use crate::math_utils::Lerpable;
use crate::widgets::Plot1DData;
use array_macro::array;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::{
    num::Wrapping,
    sync::{atomic::Ordering, Arc, Mutex},
};

pub const DRIVE_MIN_DB: f32 = -6.0;
pub const DRIVE_MAX_DB: f32 = 36.0;

#[derive(Enum, PartialEq)]
enum ClipMode {
    Digital,
    Smooth,
    Intersample,
}

#[derive(Params)]
pub struct ClipParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    pub(super) editor_state: Arc<ViziaState>,

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

impl Default for ClipParams {
    fn default() -> Self {
        Self {
            editor_state: crate::editor::default_state(),

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

            // TODO: change pl0x
            hardness: FloatParam::new("Hardness", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_unit(" %")
                .with_value_to_string(formatters::v2s_f32_percentage(2))
                .with_string_to_value(formatters::s2v_f32_percentage()),

            drive: FloatParam::new(
                "Drive",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(0.0),
                    max: util::db_to_gain(36.0),
                    factor: FloatRange::gain_skew_factor(-6.0, 36.0),
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

nih_export_clap!(crate::Clip);
nih_export_vst3!(crate::Clip);
