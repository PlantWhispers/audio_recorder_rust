use crate::BUFFER_SIZE;
use alsa::pcm::{IO, PCM};
use std::error::Error;
use std::time::SystemTime;

pub fn get_mic_data(pcm_device: &PCM, pcm_io: &IO<'_, i16>) -> Result<Vec<i16>, Box<dyn Error>> {
    let mut buffer = vec![0i16; BUFFER_SIZE];
    match pcm_io.readi(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(err) => {
            if pcm_device.try_recover(err, false).is_err() {
                panic!("Failed to recover from ALSA error: {}", err);
            }
            Err(err.into())
        }
    }
}

pub fn new_file_name() -> String {
    format!(
        "recordings/{}.raw.wav",
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
}
