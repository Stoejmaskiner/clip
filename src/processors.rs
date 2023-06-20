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
        /// fade between two gain matching techniques, 1.0 preserves 0dB peaks,
        /// making the signal louder as drive increases, 0.0 has constant gain and
        /// just lowers the threshold with more drive, decreasing loudness.
        ///
        /// This is tuned by hand such that at max drive the perceived loudness of an
        /// input at -6 dbFS is preserved. This cannot be computed as it is highly
        /// subjective, and it depends on context and psychoacustics. It's one of
        /// those "chef's parameters". In this case, various drum loops and full
        /// mixdowns were normalized to -6 dBFS and this parameter was fine tuned
        /// to produce an apparent equal loudness at full drive. Note that changing
        /// the range of the drive parameter will require re-calibrating.
        const CALIBRATION: f32 = 0.875;
        const INV_CALIBRATION: f32 = 1.0 - CALIBRATION;

        // TODO: the minimum and maximum value of `drive` get it in some less idiotic way
        const MAX_DRIVE: f32 = 63.095734;
        const _MIN_DRIVE: f32 = 0.5011872;

        // TODO: range-independent gamma-like transformation, you take the reciprocal of
        //       the drive, as the drive can go from 0 to infinity (clipping negative values
        //       to 0), and use that to get a [0; 1] t-value to index a LUT of some kind

        let t = self.drive.clamped_inverse_lerp(1.0, MAX_DRIVE);

        let comp = dsp::var_hard_clip(self.drive, self.hardness) * CALIBRATION
            + self.drive * INV_CALIBRATION;

        comp
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
