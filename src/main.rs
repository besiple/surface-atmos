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
use dsp::profile::{AcousticParameters, PostureMode};
use posture::PostureSensor;

#[derive(Parser)]
#[command(
    name = "surface-atmos",
    author = "besiple",
    version = "0.2.0",
    about = "Adaptive Stage & Studio Acoustic Compensator for Microsoft Surface Laptop Studio"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the daemon to watch physical posture and update acoustic compensation dynamically
    Daemon,
    /// Display current hardware posture and active compensation parameters
    Status,
    /// Manually switch acoustic mode (laptop, stage, studio)
    Set {
        /// Mode name: 'laptop', 'stage' (slate), or 'studio' (tablet)
        mode: String,
    },
}

fn send_notification(mode: PostureMode) {
    let title = format!("{} Surface Audio Acoustics", mode.icon());
    let body = format!("Acoustic compensation adapted to: {}", mode.display_name());
    let _ = Command::new("notify-send")
        .arg("-a")
        .arg("Surface Acoustics")
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
        Some(Commands::Daemon) | None => cmd_daemon(config),
    }
}

fn cmd_status(config: &AppConfig) {
    println!("{}", "=== Microsoft Surface Laptop Studio - Acoustic Compensator Status ===".bold().cyan());
    
    // Sensor status
    let sensor = PostureSensor::discover();
    match sensor.as_ref() {
        Some(s) => {
            let mode = s.read_mode();
            println!("  {} Posture Sensor: {} {}", "●".green(), mode.icon(), mode.display_name().bold());
        }
        None => {
            println!("  {} Posture Sensor: {}", "●".red(), "Not detected");
        }
    }

    // PipeWire status
    let mut pw = PipeWireController::new(&config.sink_name);
    match pw.find_sink_node_id() {
        Some(id) => {
            println!("  {} Smart Filter:   {} (Node ID: {})", "●".green(), config.sink_name.bold(), id);
        }
        None => {
            println!("  {} Smart Filter:   {} (Not found. Check pipewire.conf.d)", "●".red(), config.sink_name);
        }
    }

    let mode = sensor.as_ref().map(|s| s.read_mode()).unwrap_or(PostureMode::Laptop);
    let params = AcousticParameters::for_posture(mode);
    println!("  {} High-Shelf Gain: {:+.1} dB (above 3000 Hz)", "●".blue(), params.gain_db);
    println!("  {} Stereo Widening: {:.0}%", "●".blue(), params.stereo_widen * 100.0);
}

fn cmd_set(config: &AppConfig, mode_str: &str) {
    let mode = PostureMode::parse(mode_str);
    println!("Applying {} {}...", mode.icon(), mode.display_name().bold());

    let mut pw = PipeWireController::new(&config.sink_name);
    let params = AcousticParameters::for_posture(mode);

    match pw.apply_parameters(&params) {
        Ok(()) => {
            println!("{}", "Acoustic compensation updated successfully!".green());
            if config.enable_notifications {
                send_notification(mode);
            }
        }
        Err(e) => {
            eprintln!("{} Failed to apply parameters: {}", "Error:".red().bold(), e);
        }
    }
}

fn cmd_daemon(config: AppConfig) {
    println!("{}", "Starting Surface Acoustic Compensation Daemon...".bold().green());

    let sensor = match PostureSensor::discover() {
        Some(s) => s,
        None => {
            eprintln!("{}", "Error: Surface posture sensor not found in sysfs. Exiting.".red());
            std::process::exit(1);
        }
    };

    let pw = Arc::new(Mutex::new(PipeWireController::new(&config.sink_name)));
    let interpolator = Interpolator::new(config.transition_duration_ms, config.step_interval_ms);
    let current_params = Arc::new(Mutex::new(AcousticParameters::for_posture(sensor.read_mode())));

    // Initial application of profile
    {
        let mut pw_guard = pw.lock().unwrap();
        let params_guard = current_params.lock().unwrap();
        if let Err(e) = pw_guard.apply_parameters(&params_guard) {
            eprintln!("Warning: Failed to apply initial parameters: {}", e);
        } else {
            println!("Initial acoustic profile applied: {}", sensor.read_mode().display_name().cyan());
        }
    }

    let pw_clone = Arc::clone(&pw);
    let params_clone = Arc::clone(&current_params);
    let cfg = config.clone();

    sensor.watch(move |new_mode| {
        println!(">>> Posture shift detected: {} {}", new_mode.icon(), new_mode.display_name().bold());
        
        let target_params = AcousticParameters::for_posture(new_mode);
        let from_params = {
            let g = params_clone.lock().unwrap();
            *g
        };

        let pw_inner = Arc::clone(&pw_clone);
        interpolator.transition(&from_params, &target_params, |interpolated| {
            if let Ok(mut pw_guard) = pw_inner.lock() {
                let _ = pw_guard.apply_parameters(interpolated);
            }
        });

        {
            let mut g = params_clone.lock().unwrap();
            *g = target_params;
        }

        if cfg.enable_notifications {
            send_notification(new_mode);
        }
    });
}
