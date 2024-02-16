use alsa::pcm::{Access, Format};

// Default values
pub const DEFAULT_DEVICE_NAMES: &str = "hw:0,0;hw:1,0";
pub const DEFAULT_FILE_DURATION: &str = "30";
pub const DEFAULT_SOUND_EMITTER_TRIGGER_PIN: &str = "2";

// Hardcoded values
pub const SAMPLE_RATE: u32 = 384_000;
pub const FORMAT: Format = Format::S16LE;
pub const PCM_DEVICE_ACCESS: Access = Access::RWInterleaved;
pub const CHANNELS_PER_MIC: u16 = 1;
pub const ALSA_BUFFER_SIZE: usize = 19200; // Maybe maximum allowed by ALSA
pub const BUFFER_SIZE: usize = 384; // Maybe minimum allowed by ALSA
