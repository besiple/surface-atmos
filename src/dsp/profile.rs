// Hardware-calibrated Dolby Atmos profiles for Microsoft Surface Laptop Studio 1
// Realtek ALC274 Subsystem 0x10EC121C

use serde::{Deserialize, Serialize};

pub const NUM_BANDS: usize = 20;

pub const FREQUENCIES: [f32; NUM_BANDS] = [
    47.0, 141.0, 234.0, 328.0, 469.0, 656.0, 844.0, 1031.0,
    1313.0, 1688.0, 2250.0, 3000.0, 3750.0, 4688.0, 5813.0, 7125.0,
    9000.0, 11250.0, 13875.0, 19688.0,
];

#[allow(dead_code)]
pub const Q_VALUES: [f32; NUM_BANDS] = [
    1.4; NUM_BANDS
];

// Asymmetrical internal speaker physical correction (extracted from Windows driver DAX3 XML)
pub const SPEAKER_OPT_LEFT: [f32; NUM_BANDS] = [
    3.0, 3.0, 3.0, 1.19, -2.12, -2.25, -0.88, 0.69,
    0.81, 1.62, 2.12, 2.94, 2.75, 3.06, 2.81, 2.69,
    5.31, 2.50, 3.00, 1.69,
];

pub const SPEAKER_OPT_RIGHT: [f32; NUM_BANDS] = [
    3.0, 3.0, 3.0, 1.19, -2.19, -1.69, -1.50, 0.81,
    -0.12, 1.38, 2.38, 1.69, 4.00, 4.00, 4.19, 4.38,
    5.81, 7.19, 5.44, 0.75,
];

// Normalized Dolby IEQ target curves (centered around 1 kHz = 0 dB)
pub const IEQ_BALANCED: [f32; NUM_BANDS] = [
    -2.19, -1.56, 1.62, 1.62, 0.69, -0.25, 0.0, 0.0,
    0.81, 1.31, 1.62, 1.06, 0.06, -2.06, -3.62, -5.94,
    -7.56, -10.62, -17.62, -29.69,
];

pub const IEQ_DETAILED: [f32; NUM_BANDS] = [
    -3.06, -3.56, -0.69, 1.06, -0.63, -0.25, 0.18, 0.0,
    0.68, 1.62, 1.94, 2.31, 2.25, 2.25, 0.94, -2.13,
    -5.44, -9.38, -13.94, -26.00,
];

pub const IEQ_WARM: [f32; NUM_BANDS] = [
    1.50, 3.50, 5.82, 4.94, 5.00, 2.38, 0.82, 0.0,
    0.50, 2.26, 2.32, 3.13, 0.38, -0.31, -0.62, -1.50,
    -3.24, -7.62, -13.87, -22.81,
];

// Posture acoustic compensations
pub const STAGE_COMPENSATION: [f32; NUM_BANDS] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.8,
    1.2, 1.5, 2.0, 2.8, 3.5, 3.8, 3.5, 3.2,
    2.8, 2.2, 1.2, 0.0,
];

pub const STUDIO_COMPENSATION: [f32; NUM_BANDS] = [
    1.0, 0.5, -2.0, -2.5, -1.0, 0.0, 0.0, 0.5,
    1.0, 1.5, 2.5, 3.2, 3.8, 4.2, 4.0, 3.8,
    3.5, 3.0, 2.0, 0.5,
];

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

    pub fn stereo_widen(&self) -> f32 {
        match self {
            PostureMode::Laptop => 0.15,
            PostureMode::Stage => 0.25,
            PostureMode::Studio => 0.35,
            PostureMode::LidClosed => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqProfile {
    #[serde(rename = "balanced")]
    Balanced,
    #[serde(rename = "detailed")]
    Detailed,
    #[serde(rename = "warm")]
    Warm,
}

impl EqProfile {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "detailed" => EqProfile::Detailed,
            "warm" => EqProfile::Warm,
            _ => EqProfile::Balanced,
        }
    }

    pub fn ieq_curve(&self) -> [f32; NUM_BANDS] {
        match self {
            EqProfile::Balanced => IEQ_BALANCED,
            EqProfile::Detailed => IEQ_DETAILED,
            EqProfile::Warm => IEQ_WARM,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StereoGains {
    pub left: [f32; NUM_BANDS],
    pub right: [f32; NUM_BANDS],
    pub stereo_widen: f32,
}

impl StereoGains {
    pub fn compute(posture: PostureMode, profile: EqProfile) -> Self {
        let ieq = profile.ieq_curve();
        let comp = match posture {
            PostureMode::Laptop | PostureMode::LidClosed => [0.0; NUM_BANDS],
            PostureMode::Stage => STAGE_COMPENSATION,
            PostureMode::Studio => STUDIO_COMPENSATION,
        };

        let mut left = [0.0; NUM_BANDS];
        let mut right = [0.0; NUM_BANDS];

        for i in 0..NUM_BANDS {
            // Formula: Base Speaker Optimizer + IEQ Curve + Posture Delta
            let l_raw = SPEAKER_OPT_LEFT[i] + ieq[i] + comp[i];
            let r_raw = SPEAKER_OPT_RIGHT[i] + ieq[i] + comp[i];

            // Attenuate master offset by -6 dB to preserve headroom for Virtual Bass
            left[i] = (l_raw - 6.0).clamp(-24.0, 12.0);
            right[i] = (r_raw - 6.0).clamp(-24.0, 12.0);
        }

        StereoGains {
            left,
            right,
            stereo_widen: posture.stereo_widen(),
        }
    }
}
