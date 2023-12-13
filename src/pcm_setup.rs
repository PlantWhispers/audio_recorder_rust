use alsa::{pcm::{PCM, HwParams, Format, Access, Frames}, Direction};
use std::error::Error;

pub fn setup_pcm(a_device: &str, b_device: &str, sample_rate: u32, channels: u16, format: Format, access: Access, buffer_size: Frames) -> Result<(PCM, PCM), Box<dyn Error>> {
    let pcm_a = PCM::new(a_device, Direction::Capture, false)?;
    let pcm_b = PCM::new(b_device, Direction::Capture, false)?;

    {
        // Limiting the scope of HwParams
        let hwp = HwParams::any(&pcm_a)?;
        hwp.set_channels(channels.into())?;
        hwp.set_rate(sample_rate, alsa::ValueOr::Nearest)?;
        hwp.set_buffer_size(buffer_size)?;
        hwp.set_format(format)?;
        hwp.set_access(access)?;
        pcm_a.hw_params(&hwp)?;
        pcm_b.hw_params(&hwp)?;
    }

    Ok((pcm_a, pcm_b))
}
