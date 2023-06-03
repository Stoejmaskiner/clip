use array_macro::array;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod dsp;
mod editor;

/// size of each "batch" of samples taken from a channel at a time, independent
/// of host's buffer size.
const MAX_BLOCK_SIZE: usize = 64;

/// hardcoded supported number of channels (basically every DAW immaginable
/// supports stereo lmao)
const NUM_CHANNELS: usize = 2;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

pub struct Clip {
    params: Arc<ClipParams>,

    // === processors ===
    dc_blocker: [dsp::DCBlock; NUM_CHANNELS],
}

#[derive(Enum, PartialEq)]
enum ClipMode {
    Digital,
    Smooth,
    Intersample,
}

#[derive(Params)]
struct ClipParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "bypass"]
    pub bypass: BoolParam,

    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "pre-gain"]
    pub pre_gain: FloatParam,

    #[id = "post-gain"]
    pub post_gain: FloatParam,

    #[id = "hardness"]
    pub hardness: FloatParam,

    #[id = "drive"]
    pub drive: FloatParam,

    #[id = "threshold"]
    pub threshold: FloatParam,

    #[id = "mix"]
    pub mix: FloatParam,

    #[id = "dc-block"]
    pub dc_block: BoolParam,
}

impl Default for Clip {
    fn default() -> Self {
        Self {
            params: Arc::new(ClipParams::default()),
            dc_blocker: array![dsp::DCBlock::default(); NUM_CHANNELS],
        }
    }
}

impl Default for ClipParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            bypass: BoolParam::new("Bypass", false),

            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            pre_gain: FloatParam::new(
                "Pre Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-18.0),
                    max: util::db_to_gain(6.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-18.0, 6.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            post_gain: FloatParam::new(
                "Post Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-18.0),
                    max: util::db_to_gain(6.0),
                    factor: FloatRange::gain_skew_factor(-18.0, 6.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            hardness: FloatParam::new("Hardness", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(2))
                .with_string_to_value(formatters::s2v_f32_percentage()),

            drive: FloatParam::new(
                "Drive",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-12.0),
                    max: util::db_to_gain(24.0),
                    factor: FloatRange::gain_skew_factor(-12.0, 24.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            threshold: FloatParam::new(
                "Threshold",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(6.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 6.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            mix: FloatParam::new("Mix", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(2))
                .with_string_to_value(formatters::s2v_f32_percentage()),

            // calibration: FloatParam::new(
            //     "Calibration",
            //     0.5,
            //     FloatRange::Linear { min: 0.0, max: 1.0 },
            // )
            // .with_unit(" dB")
            // .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            // .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            dc_block: BoolParam::new("DC Block", false),
        }
    }
}

impl Plugin for Clip {
    const NAME: &'static str = "Clip";
    const VENDOR: &'static str = "Støjmaskiner";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "panierilorenzo@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(NUM_CHANNELS as u32),
        main_output_channels: NonZeroU32::new(NUM_CHANNELS as u32),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.

        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    // ===== PROCESS =====================================================================
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for (_, mut block) in buffer.iter_blocks(MAX_BLOCK_SIZE) {
            for channel_samples in block.iter_samples() {
                // TODO: deciding whether to bypass is done once per block to keep the branching
                //       outside of the main loop
                let bypass = self.params.bypass.value();
                if !bypass {
                    let pre_gain = self.params.pre_gain.smoothed.next();
                    let post_gain = self.params.post_gain.smoothed.next();
                    let hardness = self.params.hardness.smoothed.next();
                    let drive = self.params.drive.smoothed.next();
                    let threshold = self.params.threshold.smoothed.next();
                    let mix = self.params.mix.smoothed.next();
                    let dc_block = self.params.dc_block.value();

                    // There are two main approaches to gain compensation, the first is to
                    // divide the post-clipping by the drive amount. This approaches -inf dB
                    // with infinite drive. This works well for small drive values, but
                    // overcompensates for high dirve values.
                    //
                    // The other approach is to divide the post-clipping by the clipped drive
                    // amount. This approaches +0dB with infinite drive. It undercompensates for
                    // high drive, but works well for small drives.
                    //
                    // This approach averages the two main approaches, with the weighting
                    // tuned by hand so that volume approaches -6.0dB. This is the same
                    // behavior you see in Fab Filter's "Saturn" plugin.
                    const CALIBRATION: f32 = 0.9319508;
                    const INV_CALIBRATION: f32 = 1.0 - CALIBRATION;
                    let drive_compensation =
                        dsp::var_hard_clip(drive, hardness) * CALIBRATION + drive * INV_CALIBRATION;

                    for (chan, sample) in channel_samples.into_iter().enumerate() {
                        // TODO: branchless?
                        if dc_block {
                            *sample = self.dc_blocker[chan].step(*sample);
                        }

                        let dry = *sample;

                        *sample *= pre_gain;

                        *sample *= drive;
                        *sample /= threshold;
                        *sample = dsp::var_hard_clip(*sample, hardness);
                        *sample *= threshold;
                        *sample /= drive_compensation;

                        *sample *= post_gain;

                        *sample = mix * *sample + (1.0 - mix) * dry;
                    }
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Clip {
    const CLAP_ID: &'static str = "com.stoejmaskiner.clip";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple ergonomic clipper");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // TODO: Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Clip {
    const VST3_CLASS_ID: [u8; 16] = *b"stoej-fp001-clip";

    // TODO: And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(Clip);
nih_export_vst3!(Clip);
