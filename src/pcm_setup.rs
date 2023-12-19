use alsa::{
    pcm::{HwParams, PCM},
    Direction,
};
use std::error::Error;

use crate::{ACCESS, ALSA_BUFFER_SIZE, BUFFER_SIZE, CHANNELS, DEVICE_NAMES, FORMAT, SAMPLE_RATE};

pub fn setup_pcm() -> Result<[PCM; 2], Box<dyn Error>> {
    // TODO: Find devices automatically based on specs
    let pcm_a = PCM::new(DEVICE_NAMES[0], Direction::Capture, false)?;
    let pcm_b = PCM::new(DEVICE_NAMES[1], Direction::Capture, false)?;

    {
        let hwp = HwParams::any(&pcm_a)?;
        hwp.set_channels(CHANNELS.into())?;
        hwp.set_rate(SAMPLE_RATE, alsa::ValueOr::Nearest)?;
        hwp.set_buffer_size(ALSA_BUFFER_SIZE as i64)?;
        hwp.set_period_size(BUFFER_SIZE as i64, alsa::ValueOr::Nearest)?;
        hwp.set_format(FORMAT)?;
        hwp.set_access(ACCESS)?;
        pcm_a.hw_params(&hwp)?;
        pcm_b.hw_params(&hwp)?;
    }

    Ok([pcm_a, pcm_b])
}
