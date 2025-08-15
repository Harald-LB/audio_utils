//! This is a simplified version of the gain plugin example from
//! the [NIH-plug](https://github.com/robbert-vdh/nih-plug) documentation.
//! It demonstrates the use of `audio_utils::{DbToGain, TinySmoother}`.
//! 

use audio_utils::{DbToGain, TinySmoother};
use nih_plug::prelude::*;
use std::sync::Arc;

struct TinyGainPlug {
    params: Arc<TinyGainParams>,
    smoother: TinySmoother,
}
#[derive(Params)]
struct TinyGainParams {
    /// The gain parameter in decibels, ranging from -60 to +20 dB.
    #[id = "gain"]
    pub gain_db: IntParam,
}

impl Default for TinyGainPlug {
    fn default() -> Self {
        Self {
            params: Arc::new(TinyGainParams::default()),
            smoother: TinySmoother::default(),
        }
    }
}

impl Default for TinyGainParams {
    fn default() -> Self {
        Self {
            gain_db: IntParam::new("Gain", 0, IntRange::Linear { min: -60, max: 20 }),
        }
    }
}

impl Plugin for TinyGainPlug {
    const NAME: &'static str = "TinyGainPlug";
    const VENDOR: &'static str = "Harald-LB";
    const URL: &'static str = "https://example.com/TinyGainPlug";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    // Note: Setting this to true will cause the audio processing cycle to be split into multiple
    // smaller chunks, often only 1 sample long.
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;
    type SysExMessage = ();
    type BackgroundTask = ();
    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }
    fn reset(&mut self) {
        self.smoother.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
       
        let gain_linear = self.params.gain_db.value().to_gain();

        for channel_samples in buffer.iter_samples() {
            // Use TinySmoother to smooth the gain value across all samples.
            let gain_sample = self.smoother.next(gain_linear);

            for sample in channel_samples {
                *sample *= gain_sample;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for TinyGainPlug {
    const CLAP_ID: &'static str = "com.mystic-plugins-gmbh.gain";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A smoothed gain parameter example plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for TinyGainPlug {
    const VST3_CLASS_ID: [u8; 16] = *b"TinyGainPlug0001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(TinyGainPlug);
nih_export_vst3!(TinyGainPlug);
