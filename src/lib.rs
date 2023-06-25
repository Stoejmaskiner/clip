#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
// #![allow(incomplete_features)]
// #![feature(generic_const_exprs)]
use array_macro::array;
use core::num;
use dsp::MonoProcessor;
use nih_plug::{buffer::ChannelSamples, prelude::*};
use nih_plug_vizia::ViziaState;
use std::{
    num::Wrapping,
    sync::{atomic::Ordering, mpsc::channel, Arc, Mutex},
};
use widgets::Plot1DData;

use crate::{dsp::var_hard_clip, math_utils::Lerpable};

pub mod dsp;
mod editor;
mod filter_coefficients;
// mod luts;
pub mod math_utils;
mod params;
mod processors;
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
    plot: Arc<Plot1DData<128>>,
    in_amp_accumulator: f32,
    out_amp_accumulator: f32,

    // === processors ===
    dc_blocker: [dsp::DCBlock; NUM_CHANNELS],
    clipper: [dsp::OversampleX4<processors::MainDistortionProcessor>; NUM_CHANNELS],
    clipper_4_viz: processors::MainDistortionProcessor,

    // === config ===
    peak_meter_decay_weight: f32,
    gui_refresh_period: usize,

    // === volatile state ===
    // *could* overflow if leaving the plugin running for 24 hours on a 32-bit system
    frame_counter: Wrapping<usize>,
}

impl Clip {
    fn param_update(&mut self) {
        let pre_gain = self.params.pre_gain.smoothed.next();
        let post_gain = self.params.post_gain.smoothed.next();
        let hardness = self.params.hardness.smoothed.next();
        let drive = self.params.drive.smoothed.next();
        let threshold = self.params.threshold.smoothed.next();
        let mix = self.params.mix.smoothed.next();

        for c in &mut self.clipper {
            let inner = &mut (*c).inner_processor;
            inner.pre_gain = pre_gain;
            inner.post_gain = post_gain;
            inner.hardness = hardness;
            inner.drive = drive;
            inner.threshold = threshold;
            inner.mix = mix;
        }

        self.clipper_4_viz.pre_gain = pre_gain;
        self.clipper_4_viz.post_gain = post_gain;
        self.clipper_4_viz.hardness = hardness;
        self.clipper_4_viz.drive = drive;
        self.clipper_4_viz.threshold = threshold;
        self.clipper_4_viz.mix = mix;
    }

    /// expensive GUI calculations, that are only run at the GUI_REFRESH_RATE
    /// (i.e. 60 fps) to save on computation. Any GUI-related tasks that need
    /// to be updated once per sample should be in `audio_update()`
    fn gui_update(&mut self) {
        // update meters, the meters slowly decay if the incoming volume is
        // lower than the past volume, or immediately increases to the new
        // volume if it is louder than the last.
        let last_in_amp = self.plot.in_amp.load(Ordering::Relaxed);
        let last_out_amp = self.plot.out_amp.load(Ordering::Relaxed);
        let new_in_amp = if self.in_amp_accumulator > last_in_amp {
            self.in_amp_accumulator
        } else {
            last_in_amp * self.peak_meter_decay_weight
                + self.in_amp_accumulator * (1.0 - self.peak_meter_decay_weight)
        };
        let new_out_amp = if self.out_amp_accumulator > last_out_amp {
            self.out_amp_accumulator
        } else {
            last_out_amp * self.peak_meter_decay_weight
                + self.out_amp_accumulator * (1.0 - self.peak_meter_decay_weight)
        };
        self.plot.in_amp.store(new_in_amp, Ordering::Relaxed);
        self.plot.out_amp.store(new_out_amp, Ordering::Relaxed);

        // reset running meter accumulators
        self.in_amp_accumulator = 0.0;
        self.out_amp_accumulator = 0.0;

        self.plot.plot_processor(&mut self.clipper_4_viz);
    }

    fn audio_update(&mut self, channel_samples: ChannelSamples) {
        for (chan, sample) in channel_samples.into_iter().enumerate() {
            // update input amp accumulator (running maximum)
            {
                let x = *sample;
                self.in_amp_accumulator = self.in_amp_accumulator.max((x).abs());
            }

            // maybe apply DC blocker
            if self.params.dc_block.value() {
                let x = *sample;
                let y = self.dc_blocker[chan].step(x);
                *sample = y;
            }

            // apply main distortion
            {
                let x = *sample;
                let y = self.clipper[chan].step(x);
                *sample = y;
            }

            // update output amp accumulator (running maximum)
            {
                let x = *sample;
                self.out_amp_accumulator = self.out_amp_accumulator.max((x).abs());
            }
        }
    }
}

impl Default for Clip {
    fn default() -> Self {
        Self {
            params: Arc::new(params::ClipParams::default()),
            plot: Arc::new(Plot1DData::new()),
            in_amp_accumulator: 0.0,
            out_amp_accumulator: 0.0,
            dc_blocker: array![dsp::DCBlock::default(); NUM_CHANNELS],
            clipper: array![dsp::OversampleX4::new(processors::MainDistortionProcessor::new()); NUM_CHANNELS],
            peak_meter_decay_weight: 1.0,
            gui_refresh_period: 800,
            frame_counter: Wrapping(0),
            clipper_4_viz: processors::MainDistortionProcessor::new(),
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

    // fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    //     editor::create(
    //         self.params.clone(),
    //         self.plot.clone(),
    //         self.params.editor_state.clone(),
    //     )
    // }

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
            .powf((f64::from(GUI_REFRESH_RATE) * PEAK_METER_DECAY_MS / 1000.0).recip())
            as f32;

        self.gui_refresh_period = (buffer_config.sample_rate / GUI_REFRESH_RATE).trunc() as usize;

        context.set_latency_samples(0);

        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.

        for d in &mut self.dc_blocker {
            (*d).reset();
        }

        for c in &mut self.clipper {
            (*c).reset();
        }
    }

    // ===== PROCESS =====================================================================
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            self.param_update();

            if !self.params.bypass.value() {
                self.audio_update(channel_samples);
                //return ProcessStatus::Tail(256);
            }

            if self.frame_counter.0 % self.gui_refresh_period == 0
                && self.params.editor_state.is_open()
            {
                self.gui_update();
            }
            self.frame_counter += 1;
        }
        ProcessStatus::Tail(256)
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
