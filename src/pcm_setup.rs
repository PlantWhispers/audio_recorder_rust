use alsa::{
    pcm::{HwParams, PCM},
    Direction,
};
use std::error::Error;

use crate::{ACCESS, ALSA_BUFFER_SIZE, CHANNELS, FORMAT, SAMPLE_RATE};

pub fn setup_pcm(a_device: &str, b_device: &str) -> Result<(PCM, PCM), Box<dyn Error>> {
    // TODO: Find devices automatically based on specs
    let pcm_a = PCM::new(a_device, Direction::Capture, false)?;
    let pcm_b = PCM::new(b_device, Direction::Capture, false)?;

    {
        let hwp = HwParams::any(&pcm_a)?;
        hwp.set_channels(CHANNELS.into())?;
        hwp.set_rate(SAMPLE_RATE, alsa::ValueOr::Nearest)?;
        hwp.set_buffer_size(ALSA_BUFFER_SIZE as i64)?;
        hwp.set_format(FORMAT)?;
        hwp.set_access(ACCESS)?;
        pcm_a.hw_params(&hwp)?;
        pcm_b.hw_params(&hwp)?;
    }

    Ok((pcm_a, pcm_b))
}
