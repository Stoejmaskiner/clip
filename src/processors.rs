use crate::dsp;
use crate::dsp::MonoProcessor;
use crate::math_utils::Lerpable;

#[derive(Clone)]
pub(super) struct MainDistortionProcessor {
    pub pre_gain: f32,
    pub post_gain: f32,
    pub drive: f32,
    pub threshold: f32,
    pub hardness: f32,
    pub mix: f32,
}

impl MainDistortionProcessor {
    const CALIBRATION: f32 = 0.931_050_8;
    const INV_CALIBRATION: f32 = 1.0 - Self::CALIBRATION;
    pub fn new() -> Self {
        Self {
            pre_gain: 1.0,
            post_gain: 1.0,
            drive: 1.0,
            threshold: 1.0,
            hardness: 0.0,
            mix: 1.0,
        }
    }

    fn drive_compensation(&self) -> f32 {
        dsp::var_hard_clip(self.drive, self.hardness) * Self::CALIBRATION
            + self.drive * Self::INV_CALIBRATION
    }

    fn pre_gain(&self) -> f32 {
        self.pre_gain * self.drive / self.threshold
    }

    fn post_gain(&self) -> f32 {
        self.threshold / self.drive_compensation() * self.post_gain
    }
}

impl MonoProcessor for MainDistortionProcessor {
    fn step(&mut self, x: f32) -> f32 {
        let y = self.pre_gain() * x;
        let y = dsp::var_hard_clip(y, self.hardness);
        let y = self.post_gain() * y;
        x.lerp(y, self.mix)
    }

    fn reset(&mut self) {}

    fn initialize(&mut self) {}
}
