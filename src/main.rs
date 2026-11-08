use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process::Command;
use std::sync::{Arc, Mutex};

mod config;
mod dsp;
mod posture;

use config::AppConfig;
use dsp::interpolator::Interpolator;
use dsp::pipewire_ctl::PipeWireController;
use dsp::profile::{EqProfile, PostureMode, StereoGains, FREQUENCIES, NUM_BANDS};
use posture::PostureSensor;

#[derive(Parser)]
#[command(
    name = "surface-atmos",
    author = "besiple",
    version = "0.1.0",
    about = "Dolby Atmos DSP and adaptive posture acoustics daemon for Microsoft Surface Laptop Studio"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the daemon to watch physical posture and update DSP filters dynamically
    Daemon,
    /// Display current hardware posture, active EQ profile and PipeWire sink status
    Status,
    /// Manually switch acoustic mode (laptop, stage, studio)
    Set {
        /// Mode name: 'laptop', 'stage' (slate), or 'studio' (tablet)
        mode: String,
    },
    /// Switch EQ target profile (balanced, detailed, warm)
    Profile {
        /// Profile name: 'balanced', 'detailed', or 'warm'
        name: String,
    },
}

fn send_notification(mode: PostureMode) {
    let title = format!("{} Surface Dolby Atmos", mode.icon());
    let body = format!("Acoustic profile switched to: {}", mode.display_name());
    let _ = Command::new("notify-send")
        .arg("-a")
        .arg("Surface Atmos")
        .arg("-i")
        .arg("audio-speakers")
        .arg(&title)
        .arg(&body)
        .spawn();
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    let config = AppConfig::load_or_default();

    match cli.command {
        Some(Commands::Status) => cmd_status(&config),
        Some(Commands::Set { mode }) => cmd_set(&config, &mode),
        Some(Commands::Profile { name }) => cmd_profile(&name),
        Some(Commands::Daemon) | None => cmd_daemon(config),
    }
}

fn cmd_status(config: &AppConfig) {
    println!("{}", "=== Microsoft Surface Laptop Studio - Dolby Atmos Status ===".bold().cyan());
    
    // Sensor status
    let sensor = PostureSensor::discover();
    match sensor.as_ref() {
        Some(s) => {
            let mode = s.read_mode();
            println!("  {} Posture Sensor: {} {}", "●".green(), mode.icon(), mode.display_name().bold());
        }
        None => {
            println!("  {} Posture Sensor: {}", "●".red(), "Not detected (using fallback mode)");
        }
    }

    // PipeWire status
    let mut pw = PipeWireController::new(&config.sink_name);
    match pw.find_sink_node_id() {
        Some(id) => {
            println!("  {} PipeWire Sink:  {} (Node ID: {})", "●".green(), config.sink_name.bold(), id);
        }
        None => {
            println!("  {} PipeWire Sink:  {} (Not found. Run install.sh to setup)", "●".red(), config.sink_name);
        }
    }

    println!("  {} EQ Profile:     {}", "●".blue(), config.profile.to_uppercase().bold());
    println!("  {} Crossfade Slew: {} ms", "●".blue(), config.transition_duration_ms);

    println!("\n{}", "Active 20-band EQ Curve (Left / Right):".bold());
    let mode = sensor.as_ref().map(|s| s.read_mode()).unwrap_or(PostureMode::Laptop);
    let gains = StereoGains::compute(mode, config.eq_profile());

    for i in 0..NUM_BANDS {
        let f_str = if FREQUENCIES[i] >= 1000.0 {
            format!("{:>4.1}k", FREQUENCIES[i] / 1000.0)
        } else {
            format!("{:>4.0} ", FREQUENCIES[i])
        };
        println!(
            "  [{:>2}] {:>5} Hz | L: {:>+6.2} dB | R: {:>+6.2} dB",
            i, f_str, gains.left[i], gains.right[i]
        );
    }
}

fn cmd_set(config: &AppConfig, mode_str: &str) {
    let mode = PostureMode::parse(mode_str);
    println!("Applying {} {}...", mode.icon(), mode.display_name().bold());

    let mut pw = PipeWireController::new(&config.sink_name);
    let gains = StereoGains::compute(mode, config.eq_profile());

    match pw.apply_gains(&gains) {
        Ok(()) => {
            println!("{}", "Acoustic DSP gains updated successfully!".green());
            if config.enable_notifications {
                send_notification(mode);
            }
        }
        Err(e) => {
            eprintln!("{} Failed to apply gains: {}", "Error:".red().bold(), e);
        }
    }
}

fn cmd_profile(profile_str: &str) {
    let profile = EqProfile::parse(profile_str);
    let mut config = AppConfig::load_or_default();
    config.profile = format!("{:?}", profile).to_lowercase();

    let path = AppConfig::config_path();
    if let Ok(toml_str) = toml::to_string_pretty(&config) {
        let _ = std::fs::write(&path, toml_str);
        println!("Switched default EQ profile to: {}", config.profile.to_uppercase().bold().green());
    }

    // Apply to current posture
    let sensor = PostureSensor::discover();
    let mode = sensor.map(|s| s.read_mode()).unwrap_or(PostureMode::Laptop);
    let gains = StereoGains::compute(mode, profile);

    let mut pw = PipeWireController::new(&config.sink_name);
    let _ = pw.apply_gains(&gains);
}

fn cmd_daemon(config: AppConfig) {
    println!("{}", "Starting Surface Atmos Adaptive Acoustic Daemon...".bold().green());

    let sensor = match PostureSensor::discover() {
        Some(s) => s,
        None => {
            eprintln!("{}", "Error: Surface posture sensor not found in sysfs. Exiting.".red());
            std::process::exit(1);
        }
    };

    let pw = Arc::new(Mutex::new(PipeWireController::new(&config.sink_name)));
    let interpolator = Interpolator::new(config.transition_duration_ms, config.step_interval_ms);
    let current_gains = Arc::new(Mutex::new(StereoGains::compute(
        sensor.read_mode(),
        config.eq_profile(),
    )));

    // Initial application of profile
    {
        let mut pw_guard = pw.lock().unwrap();
        let gains_guard = current_gains.lock().unwrap();
        if let Err(e) = pw_guard.apply_gains(&gains_guard) {
            eprintln!("Warning: Failed to apply initial gains: {}", e);
        } else {
            println!("Initial acoustic profile applied: {}", sensor.read_mode().display_name().cyan());
        }
    }

    let pw_clone = Arc::clone(&pw);
    let gains_clone = Arc::clone(&current_gains);
    let cfg = config.clone();

    sensor.watch(move |new_mode| {
        println!(">>> Posture shift detected: {} {}", new_mode.icon(), new_mode.display_name().bold());
        
        let target_gains = StereoGains::compute(new_mode, cfg.eq_profile());
        let from_gains = {
            let g = gains_clone.lock().unwrap();
            g.clone()
        };

        let pw_inner = Arc::clone(&pw_clone);
        interpolator.transition(&from_gains, &target_gains, |interpolated| {
            if let Ok(mut pw_guard) = pw_inner.lock() {
                let _ = pw_guard.apply_gains(interpolated);
            }
        });

        {
            let mut g = gains_clone.lock().unwrap();
            *g = target_gains;
        }

        if cfg.enable_notifications {
            send_notification(new_mode);
        }
    });
}
