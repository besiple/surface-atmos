// PipeWire control interface for dynamic smart-filter parameter updates

use std::process::Command;
use crate::dsp::profile::AcousticParameters;
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
            _ => return self.find_node_id_pw_cli(),
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

    pub fn apply_parameters(&mut self, params: &AcousticParameters) -> Result<(), String> {
        let node_id = match self.cached_node_id {
            Some(id) => id,
            None => self
                .find_sink_node_id()
                .ok_or_else(|| format!("Could not find PipeWire smart filter node '{}'", self.sink_node_name))?,
        };

        let params_str = format!(
            "{{ params = [ \"eq_fl:Gain\" {:.2} \"eq_fr:Gain\" {:.2} \"channelmix.stereo-widen\" {:.2} ] }}",
            params.gain_db, params.gain_db, params.stereo_widen
        );

        let status = Command::new("pw-cli")
            .arg("s")
            .arg(node_id.to_string())
            .arg("Props")
            .arg(&params_str)
            .status();

        match status {
            Ok(s) if s.success() => {
                debug!("Applied acoustic parameters to node {}: gain={:.2}dB widen={:.2}", node_id, params.gain_db, params.stereo_widen);
                Ok(())
            }
            Ok(s) => {
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
