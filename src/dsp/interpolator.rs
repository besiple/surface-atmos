// Smooth slew-rate interpolator for glitch-free EQ transitions

use std::thread;
use std::time::Duration;
use crate::dsp::profile::{StereoGains, NUM_BANDS};

pub struct Interpolator {
    pub transition_duration_ms: u64,
    pub step_interval_ms: u64,
}

impl Default for Interpolator {
    fn default() -> Self {
        Self {
            transition_duration_ms: 250,
            step_interval_ms: 25,
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

    pub fn transition<F>(&self, from: &StereoGains, to: &StereoGains, mut apply_fn: F)
    where
        F: FnMut(&StereoGains),
    {
        let steps = (self.transition_duration_ms / self.step_interval_ms).max(1) as usize;

        for step in 1..=steps {
            let t = step as f32 / steps as f32;
            // S-curve smoothstep interpolation: t * t * (3.0 - 2.0 * t)
            let factor = t * t * (3.0 - 2.0 * t);

            let mut current = StereoGains {
                left: [0.0; NUM_BANDS],
                right: [0.0; NUM_BANDS],
                stereo_widen: from.stereo_widen + (to.stereo_widen - from.stereo_widen) * factor,
            };

            for i in 0..NUM_BANDS {
                current.left[i] = from.left[i] + (to.left[i] - from.left[i]) * factor;
                current.right[i] = from.right[i] + (to.right[i] - from.right[i]) * factor;
            }

            apply_fn(&current);

            if step < steps {
                thread::sleep(Duration::from_millis(self.step_interval_ms));
            }
        }
    }
}
