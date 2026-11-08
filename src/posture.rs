// Hardware sensor reader for Surface Laptop Studio posture states

use std::fs;
use std::path::{Path, PathBuf};
use inotify::{Inotify, WatchMask};
use crate::dsp::profile::PostureMode;
use log::{debug, info, warn};

pub struct PostureSensor {
    state_file_path: PathBuf,
}

impl PostureSensor {
    pub fn discover() -> Option<Self> {
        // Direct path on Surface Laptop Studio
        let primary = PathBuf::from("/sys/devices/platform/MSHW0123:00/01:26:01:00:01/state");
        if primary.exists() {
            info!("Found Surface posture sensor at {}", primary.display());
            return Some(Self { state_file_path: primary });
        }

        // Search in surface aggregator drivers
        let ssam_dir = Path::new("/sys/bus/surface_aggregator/drivers/surface_aggregator_tablet_mode_switch");
        if ssam_dir.exists() {
            if let Ok(entries) = fs::read_dir(ssam_dir) {
                for entry in entries.flatten() {
                    let state = entry.path().join("state");
                    if state.exists() {
                        info!("Found posture sensor via driver link at {}", state.display());
                        return Some(Self { state_file_path: state });
                    }
                }
            }
        }

        // Generic search in /sys/devices/platform/
        if let Ok(entries) = fs::read_dir("/sys/devices/platform") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.to_string_lossy().contains("MSHW") {
                    let candidate = path.join("01:26:01:00:01/state");
                    if candidate.exists() {
                        info!("Found posture sensor via platform scan at {}", candidate.display());
                        return Some(Self { state_file_path: candidate });
                    }
                }
            }
        }

        warn!("Could not automatically locate Surface posture sensor in sysfs");
        None
    }

    pub fn read_mode(&self) -> PostureMode {
        match fs::read_to_string(&self.state_file_path) {
            Ok(content) => PostureMode::parse(&content),
            Err(e) => {
                debug!("Failed to read {}: {}, defaulting to Laptop", self.state_file_path.display(), e);
                PostureMode::Laptop
            }
        }
    }

    pub fn watch<F>(&self, mut callback: F)
    where
        F: FnMut(PostureMode),
    {
        let mut inotify = match Inotify::init() {
            Ok(i) => i,
            Err(e) => {
                warn!("Failed to initialize inotify: {}, falling back to polling", e);
                self.poll_watch(callback);
                return;
            }
        };

        // Watch the parent directory because sysfs virtual files don't always emit standard file modify events
        let parent = match self.state_file_path.parent() {
            Some(p) => p,
            None => {
                self.poll_watch(callback);
                return;
            }
        };

        if let Err(e) = inotify.watches().add(
            parent,
            WatchMask::MODIFY | WatchMask::ATTRIB | WatchMask::CLOSE_WRITE | WatchMask::ACCESS,
        ) {
            warn!("Could not add inotify watch to {}: {}, using polling", parent.display(), e);
            self.poll_watch(callback);
            return;
        }

        info!("Watching posture sensor at {} for changes", self.state_file_path.display());
        let mut last_mode = self.read_mode();
        callback(last_mode);

        let mut buffer = [0u8; 1024];
        loop {
            // Read inotify events or timeout after 1 second to poll in case sysfs bypassed inotify
            let events = inotify.read_events(&mut buffer);
            if events.is_ok() {
                let current_mode = self.read_mode();
                if current_mode != last_mode {
                    info!("Posture changed: {:?} -> {:?}", last_mode, current_mode);
                    last_mode = current_mode;
                    callback(current_mode);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
            let polled_mode = self.read_mode();
            if polled_mode != last_mode {
                info!("Posture changed (polled): {:?} -> {:?}", last_mode, polled_mode);
                last_mode = polled_mode;
                callback(polled_mode);
            }
        }
    }

    fn poll_watch<F>(&self, mut callback: F)
    where
        F: FnMut(PostureMode),
    {
        let mut last_mode = self.read_mode();
        callback(last_mode);

        loop {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let current_mode = self.read_mode();
            if current_mode != last_mode {
                info!("Posture transition detected: {:?} -> {:?}", last_mode, current_mode);
                last_mode = current_mode;
                callback(current_mode);
            }
        }
    }
}
