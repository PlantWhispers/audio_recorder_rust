use alsa::{pcm::{PCM, HwParams, Format, Access, Frames}, Direction};
use std::error::Error;

pub fn setup_pcm(device: &str, sample_rate: u32, channels: u16, format: Format, access: Access, buffer_size: Frames) -> Result<PCM, Box<dyn Error>> {
    let pcm = PCM::new(device, Direction::Capture, false)?;
    
    {
        // Limiting the scope of HwParams
        let hwp = HwParams::any(&pcm)?;
        hwp.set_channels(channels.into())?;
        hwp.set_rate(sample_rate, alsa::ValueOr::Nearest)?;
        hwp.set_buffer_size(buffer_size)?;
        hwp.set_format(format)?;
        hwp.set_access(access)?;
        pcm.hw_params(&hwp)?;
    }

    Ok(pcm)
}
