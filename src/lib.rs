#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![feature(generic_const_exprs)]
use array_macro::array;
use dsp::MonoProcessor;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::{
    num::Wrapping,
    sync::{atomic::Ordering, Arc, Mutex},
};
use widgets::Plot1DData;

use crate::math_utils::Lerpable;

mod dsp;
mod editor;
mod math_utils;
mod params;
mod widgets;

/// size of each "batch" of samples taken from a channel at a time, independent
/// of host's buffer size.
const MAX_BLOCK_SIZE: usize = 64;

/// hardcoded supported number of channels (basically every DAW immaginable
/// supports stereo lmao)
const NUM_CHANNELS: usize = 2;

const PEAK_METER_DECAY_MS: f64 = 650.0;

/// skips computing expensive GUI calculations in audio loop
const GUI_REFRESH_RATE: f32 = 60.0;

pub struct Clip {
    params: Arc<params::ClipParams>,

    // === widgets ===
    // TODO: consider RwLock instead
    // TODO: consider a deadlock-free alternative
    plot: Arc<Plot1DData<128>>,

    // === processors ===
    dc_blocker: [dsp::DCBlock; NUM_CHANNELS],

    // === config ===
    peak_meter_decay_weight: f32,
    gui_refresh_period: usize,

    // === volatile state ===
    // *could* overflow if leaving the plugin running for 24 hours on a 32-bit system
    frame_counter: Wrapping<usize>,
}

impl Default for Clip {
    fn default() -> Self {
        Self {
            params: Arc::new(params::ClipParams::default()),
            plot: Arc::new(Plot1DData::new()),
            dc_blocker: array![dsp::DCBlock::default(); NUM_CHANNELS],
            peak_meter_decay_weight: 1.0,
            gui_refresh_period: 800,
            frame_counter: Wrapping(0),
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
        editor::create(
            self.params.clone(),
            self.plot.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.

        self.plot.xlim.0.store(0.0, Ordering::Relaxed);
        self.plot.xlim.1.store(1.0, Ordering::Relaxed);
        self.plot.ylim.0.store(0.0, Ordering::Relaxed);
        self.plot.ylim.1.store(1.0, Ordering::Relaxed);

        self.peak_meter_decay_weight = 0.25f64
            .powf((f64::from(buffer_config.sample_rate) * PEAK_METER_DECAY_MS / 1000.0).recip())
            as f32;

        self.gui_refresh_period = (buffer_config.sample_rate / GUI_REFRESH_RATE).trunc() as usize;

        context.set_latency_samples(0);

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
                if self.params.bypass.value() {
                    return ProcessStatus::Normal;
                }

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
                const CALIBRATION: f32 = 0.931_950_8;
                const INV_CALIBRATION: f32 = 1.0 - CALIBRATION;
                let drive_compensation =
                    dsp::var_hard_clip(drive, hardness) * CALIBRATION + drive * INV_CALIBRATION;

                // TODO: put this somewhere else
                let main_nonlinearity = |x| {
                    let dry = x;
                    let mut y = x;
                    y *= pre_gain;
                    y *= drive;
                    y /= threshold;
                    y = dsp::var_hard_clip(y, hardness);
                    y *= threshold;
                    y /= drive_compensation;
                    y *= post_gain;
                    y = dry.lerp(y, mix);
                    y
                };

                let mut in_amp_sum = 0.0;
                let mut out_amp_sum = 0.0;

                for (chan, sample) in channel_samples.into_iter().enumerate() {
                    // TODO: branchless?
                    if dc_block {
                        *sample = self.dc_blocker[chan].step(*sample);
                    }

                    in_amp_sum += *sample;
                    *sample = main_nonlinearity(*sample);
                    out_amp_sum += *sample;
                }

                if !self.params.editor_state.is_open() {
                    return ProcessStatus::Normal;
                }

                let in_amp = in_amp_sum.abs() / block.channels() as f32;
                let out_amp = out_amp_sum.abs() / block.channels() as f32;

                // TODO: this is really really bad for performance, almost on purpose
                //       because I want to track the performance gains when optimizing
                let mut plot_in_amp = self.plot.in_amp.load(Ordering::Relaxed);
                let mut plot_out_amp = self.plot.out_amp.load(Ordering::Relaxed);
                plot_in_amp = if in_amp > plot_in_amp {
                    in_amp
                } else {
                    plot_in_amp * self.peak_meter_decay_weight
                        + in_amp * (1.0 - self.peak_meter_decay_weight)
                };
                plot_out_amp = if out_amp > plot_out_amp {
                    out_amp
                } else {
                    plot_out_amp * self.peak_meter_decay_weight
                        + out_amp * (1.0 - self.peak_meter_decay_weight)
                };
                self.plot.in_amp.store(plot_in_amp, Ordering::Relaxed);
                self.plot.out_amp.store(plot_out_amp, Ordering::Relaxed);

                if self.frame_counter.0 % self.gui_refresh_period == 0 {
                    self.plot.plot_function(main_nonlinearity);
                }
                self.frame_counter += 1;
                dbg!(self.frame_counter);
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
