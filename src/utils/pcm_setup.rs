/// PCM Setup Module
///
/// Handles the setup of PCM (Pulse-code modulation) devices for audio recording. It configures the devices
/// based on predefined settings from the `config` module.
use crate::config::{
    ALSA_BUFFER_SIZE, BUFFER_SIZE, CHANNELS_PER_MIC, DEVICE_NAMES, FORMAT, PCM_DEVICE_ACCESS,
    SAMPLE_RATE,
};
use alsa::{
    pcm::{HwParams, PCM},
    Direction,
};
use std::error::Error;

pub fn setup_pcm() -> Result<[PCM; 2], Box<dyn Error>> {
    // TODO: Find devices automatically based on specs
    let pcm_a = PCM::new(DEVICE_NAMES[0], Direction::Capture, false)?;
    let pcm_b = PCM::new(DEVICE_NAMES[1], Direction::Capture, false)?;

    {
        let hwp = HwParams::any(&pcm_a)?;
        hwp.set_channels(CHANNELS_PER_MIC.into())?;
        hwp.set_rate(SAMPLE_RATE, alsa::ValueOr::Nearest)?;
        hwp.set_buffer_size(ALSA_BUFFER_SIZE as i64)?;
        hwp.set_period_size(BUFFER_SIZE as i64, alsa::ValueOr::Nearest)?;
        hwp.set_format(FORMAT)?;
        hwp.set_access(PCM_DEVICE_ACCESS)?;
        pcm_a.hw_params(&hwp)?;
        pcm_b.hw_params(&hwp)?;
    }

    Ok([pcm_a, pcm_b])
}
