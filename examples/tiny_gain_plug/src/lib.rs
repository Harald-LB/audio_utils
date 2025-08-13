use nih_plug::prelude::*;
use std::sync::Arc;

struct TinyGainPlug {
    params: Arc<TinyGainParams>,
    previous_gain: i32,
}
#[derive(Params)]
struct TinyGainParams {
    /// We make the gain parameter an integer between -100 and 20 dB.
    #[id = "gain"]
    pub gain: IntParam,
}


impl Default for TinyGainPlug {
    fn default() -> Self {
        Self {
            params: Arc::new(TinyGainParams::default()),
            previous_gain: 0,
        }
    }
}

impl Default for TinyGainParams {
    fn default() -> Self {
        Self {
            gain: IntParam::new(
                "Gain",
                0,
                IntRange::Linear {min: -100, max: 20}, )
        }
    }
}

impl Plugin for TinyGainPlug {
    const NAME: &'static str = "TinyGainPlug";
    const VENDOR: &'static str = "Harald-LB";
    // You can use `env!("CARGO_PKG_HOMEPAGE")` to reference the homepage field from the
    // `Cargo.toml` file here
    const URL: &'static str = "https://example.com/TinyGainPlug";
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),

            aux_input_ports: &[],
            aux_output_ports: &[],

            // Individual ports and the layout as a whole can be named here. By default these names
            // are generated as needed. This layout will be called 'Stereo', while the other one is
            // given the name 'Mono' based no the number of input and output channels.
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();
    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    // This plugin doesn't need any special initialization, but if you need to do anything expensive
    // then this would be the place. State is kept around when the host reconfigures the
    // plugin. If we do need special initialization, we could implement the `initialize()` and/or
    // `reset()` methods

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let gain_db = &self.params.gain.value();
        if gain_db != &self.previous_gain {
            nih_log!("Gain changed to {}", gain_db);
            self.previous_gain = *gain_db;
        }   
        
       // for channel_samples in buffer.iter_samples() {  }

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
