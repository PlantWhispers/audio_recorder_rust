use alsa::pcm::{Access, Format};

pub const DEVICE_NAMES: [&str; 2] = ["hw:0,0", "hw:1,0"];
pub const SAMPLE_RATE: u32 = 384_000;
pub const CHANNELS_PER_MIC: u16 = 1;
pub const FORMAT: Format = Format::S16LE;
pub const PCM_DEVICE_ACCESS: Access = Access::RWInterleaved;
pub const ALSA_BUFFER_SIZE: usize = 19200; // Adjust as needed
pub const BUFFER_SIZE: usize = 384; // Adjust as needed
pub const TIME_BETWEEN_RESETS_IN_S: u32 = 10;
pub const N_OF_BUFFERS_PER_FILE: u32 = TIME_BETWEEN_RESETS_IN_S * SAMPLE_RATE / BUFFER_SIZE as u32;
pub const SOUND_EMITTER_TRIGGER_PIN: u8 = 2;
