use alsa::pcm::{Access, Format};

// Should be configurable after compiling in toml file
pub const DEVICE_NAMES: [&str; 2] = ["hw:0,0", "hw:1,0"];
pub const TIME_BETWEEN_RESETS_IN_S: u32 = 30;
pub const SOUND_EMITTER_TRIGGER_PIN: u8 = 2;
pub const SOUNDFOLDER_PATH: &str = "/home/pi/raw-data/";
pub const EXPERIMENT_NAME: &str = "software-test-1";

// Should be hardcoded
pub const SAMPLE_RATE: u32 = 384_000;
pub const FORMAT: Format = Format::S16LE;
pub const PCM_DEVICE_ACCESS: Access = Access::RWInterleaved;
pub const CHANNELS_PER_MIC: u16 = 1;
pub const ALSA_BUFFER_SIZE: usize = 19200; // Maybe maximum allowed by ALSA
pub const BUFFER_SIZE: usize = 384; // Maybe minimum allowed by ALSA
