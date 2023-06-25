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
        // the post-gain normally decreases linearly as the input
        // drive increases linearly. This GAMMA parameter is an exponent
        // to this relationship, which moves the mid-point up and down.
        // This is exacly like the gamma transform from image processing,
        // but scaled so that the effect only happens for drive > 2.
        const GAMMA: f32 = 0.45;

        // compensation coefficient that makes the output peak be 0 dBFS when
        // the input peak is 0 dBFS, regardless of drive. This is useful as
        // a blank slate for designing a second compensation coefficient
        // as a function of drive
        let compensate_to_unity = 1.0_f32
            .max(dsp::var_hard_clip(self.drive, self.hardness))
            .recip();

        // compensation coefficient that applies a gamma transformation
        // to the reciprocal of drive to derive the post gain, adjusting
        // the gamma value changes the midpoint loudness of the drive
        // parameter while leaving the extremes unaffected
        let drive_recip = self.drive.recip();
        let drive_recip_2 = 2.0 * drive_recip;
        let compensate_gamma = drive_recip_2.max(drive_recip_2.powf(GAMMA)) / 2.0;

        compensate_to_unity * compensate_gamma
    }

    fn pre_gain(&self) -> f32 {
        self.pre_gain * self.drive / self.threshold
    }

    fn post_gain(&self) -> f32 {
        self.threshold * self.drive_compensation() * self.post_gain
    }
}

impl MonoProcessor for MainDistortionProcessor {
    fn step(&mut self, x: f32) -> f32 {
        let y = self.pre_gain() * x;
        let y = dsp::var_hard_clip(y, self.hardness);
        let y = self.post_gain() * y;
        x.lerp(y, self.mix)
    }

    fn process_simd_4(&mut self, x: wide::f32x4) -> wide::f32x4 {
        let y = self.pre_gain() * x;
        let y = dsp::var_hard_clip_simd_4(y, self.hardness);
        let y = self.post_gain() * y;
        x.lerp(y, self.mix)
    }

    fn reset(&mut self) {}

    fn initialize(&mut self) {}
}
