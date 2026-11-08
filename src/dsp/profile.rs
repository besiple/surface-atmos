// Adaptive Posture Acoustic Profiles for Microsoft Surface Laptop Studio 1
// Realtek ALC274 Subsystem 0x10EC121C

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostureMode {
    #[serde(rename = "laptop")]
    Laptop,
    #[serde(rename = "slate")]
    Stage,
    #[serde(rename = "tablet")]
    Studio,
    #[serde(rename = "lid_closed")]
    LidClosed,
}

impl PostureMode {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "slate" | "stage" => PostureMode::Stage,
            "tablet" | "studio" | "canvas" => PostureMode::Studio,
            "lid_closed" | "closed" => PostureMode::LidClosed,
            _ => PostureMode::Laptop,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PostureMode::Laptop => "Laptop Mode (Standard)",
            PostureMode::Stage => "Stage Mode (Screen over keyboard)",
            PostureMode::Studio => "Studio Mode (Canvas/Tablet flat)",
            PostureMode::LidClosed => "Lid Closed",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PostureMode::Laptop => "💻",
            PostureMode::Stage => "🎬",
            PostureMode::Studio => "🎨",
            PostureMode::LidClosed => "🔒",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticParameters {
    pub gain_db: f32,
    pub stereo_widen: f32,
}

impl AcousticParameters {
    pub fn for_posture(posture: PostureMode) -> Self {
        match posture {
            // In Laptop mode, 0 dB shelf = 100% transparent bypass (pure Built-in Audio)
            PostureMode::Laptop | PostureMode::LidClosed => Self {
                gain_db: 0.0,
                stereo_widen: 0.0,
            },
            // In Stage mode (display over keyboard), +4.5 dB high-shelf pierces cleanly through the glass
            PostureMode::Stage => Self {
                gain_db: 4.5,
                stereo_widen: 0.25,
            },
            // In Studio mode (laid flat), +5.0 dB high-shelf and spatial widening
            PostureMode::Studio => Self {
                gain_db: 5.0,
                stereo_widen: 0.30,
            },
        }
    }
}
