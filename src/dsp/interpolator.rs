// Smooth slew-rate interpolator for glitch-free acoustic transitions

use std::thread;
use std::time::Duration;
use crate::dsp::profile::AcousticParameters;

pub struct Interpolator {
    pub transition_duration_ms: u64,
    pub step_interval_ms: u64,
}

impl Default for Interpolator {
    fn default() -> Self {
        Self {
            transition_duration_ms: 200,
            step_interval_ms: 20,
        }
    }
}

impl Interpolator {
    pub fn new(duration_ms: u64, interval_ms: u64) -> Self {
        Self {
            transition_duration_ms: duration_ms,
            step_interval_ms: interval_ms.max(10),
        }
    }

    pub fn transition<F>(&self, from: &AcousticParameters, to: &AcousticParameters, mut apply_fn: F)
    where
        F: FnMut(&AcousticParameters),
    {
        let steps = (self.transition_duration_ms / self.step_interval_ms).max(1) as usize;

        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            // S-curve smoothstep interpolation
            let factor = t * t * (3.0 - 2.0 * t);

            let current = AcousticParameters {
                gain_db: from.gain_db + (to.gain_db - from.gain_db) * factor,
                stereo_widen: from.stereo_widen + (to.stereo_widen - from.stereo_widen) * factor,
            };

            apply_fn(&current);

            if step < steps {
                thread::sleep(Duration::from_millis(self.step_interval_ms));
            }
        }
    }
}
