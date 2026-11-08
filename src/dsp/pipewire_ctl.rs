// PipeWire control interface for dynamic filter-chain parameter updates

use std::process::Command;
use crate::dsp::profile::{StereoGains, NUM_BANDS};
use log::{debug, error, warn};

pub struct PipeWireController {
    sink_node_name: String,
    cached_node_id: Option<u32>,
}

impl PipeWireController {
    pub fn new(sink_node_name: &str) -> Self {
        Self {
            sink_node_name: sink_node_name.to_string(),
            cached_node_id: None,
        }
    }

    pub fn find_sink_node_id(&mut self) -> Option<u32> {
        let output = match Command::new("pw-dump").arg("Node").output() {
            Ok(out) if out.status.success() => out.stdout,
            _ => {
                debug!("pw-dump failed, falling back to pw-cli ls Node");
                return self.find_node_id_pw_cli();
            }
        };

        let json_val: Result<serde_json::Value, _> = serde_json::from_slice(&output);
        if let Ok(serde_json::Value::Array(nodes)) = json_val {
            for node in nodes {
                if let Some(props) = node.get("info").and_then(|i| i.get("props")) {
                    if let Some(name) = props.get("node.name").and_then(|n| n.as_str()) {
                        if name == self.sink_node_name {
                            if let Some(id) = node.get("id").and_then(|i| i.as_u64()) {
                                let node_id = id as u32;
                                self.cached_node_id = Some(node_id);
                                return Some(node_id);
                            }
                        }
                    }
                }
            }
        }

        self.find_node_id_pw_cli()
    }

    fn find_node_id_pw_cli(&mut self) -> Option<u32> {
        let output = Command::new("pw-cli")
            .arg("ls")
            .arg("Node")
            .output()
            .ok()?;

        let text = String::from_utf8_lossy(&output.stdout);
        let mut current_id = None;

        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("id ") {
                if let Some(id_part) = line.split(',').next() {
                    if let Some(id_str) = id_part.strip_prefix("id ") {
                        current_id = id_str.trim().parse::<u32>().ok();
                    }
                }
            } else if line.contains(&format!("node.name = \"{}\"", self.sink_node_name)) {
                if let Some(id) = current_id {
                    self.cached_node_id = Some(id);
                    return Some(id);
                }
            }
        }

        None
    }

    pub fn apply_gains(&mut self, gains: &StereoGains) -> Result<(), String> {
        let node_id = match self.cached_node_id {
            Some(id) => id,
            None => self
                .find_sink_node_id()
                .ok_or_else(|| format!("Could not find PipeWire sink node '{}'", self.sink_node_name))?,
        };

        // Construct PipeWire SPA pod parameters string for Props:
        // '{ params = [ "eq_fl_00:Gain" 3.0 "eq_fr_00:Gain" 3.0 ... ] }'
        let mut params_str = String::with_capacity(1024);
        params_str.push_str("{ params = [ ");

        for i in 0..NUM_BANDS {
            params_str.push_str(&format!(
                "\"eq_fl_{:02}:Gain\" {:.2} \"eq_fr_{:02}:Gain\" {:.2} ",
                i, gains.left[i], i, gains.right[i]
            ));
        }

        // Add stereo widening if supported by the adapter
        params_str.push_str(&format!("\"channelmix.stereo-widen\" {:.2} ", gains.stereo_widen));
        params_str.push_str("] }");

        let status = Command::new("pw-cli")
            .arg("s")
            .arg(node_id.to_string())
            .arg("Props")
            .arg(&params_str)
            .status();

        match status {
            Ok(s) if s.success() => {
                debug!("Successfully applied DSP gains to node {}", node_id);
                Ok(())
            }
            Ok(s) => {
                // If failed, clear cache and retry once
                warn!("pw-cli failed with status {}, retrying node discovery...", s);
                self.cached_node_id = None;
                if let Some(new_id) = self.find_sink_node_id() {
                    let retry = Command::new("pw-cli")
                        .arg("s")
                        .arg(new_id.to_string())
                        .arg("Props")
                        .arg(&params_str)
                        .status();
                    if retry.map(|r| r.success()).unwrap_or(false) {
                        return Ok(());
                    }
                }
                Err(format!("pw-cli exited with error code {:?}", s.code()))
            }
            Err(e) => {
                error!("Failed to execute pw-cli: {}", e);
                Err(format!("Failed to execute pw-cli: {}", e))
            }
        }
    }
}
