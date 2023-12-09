use crate::shared_buffer::SharedBuffer;
use alsa::pcm::PCM;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::error::Error;

pub struct Recorder {
    pcm_device: PCM,
    shared_buffer: Arc<SharedBuffer>,
    frame_size: usize,
    is_recording: Mutex<bool>,
}

impl Recorder {
    pub fn new(pcm_device: PCM, shared_buffer: Arc<SharedBuffer>, frame_size: usize) -> Result<Self, Box<dyn Error>> {
        Ok(Recorder {
            pcm_device,
            shared_buffer,
            frame_size,
            is_recording: Mutex::new(false),
        })
    }

    pub fn start(&self, duration_secs: u64) -> Result<(), Box<dyn Error>> {
        let mut is_recording = self.is_recording.lock().unwrap();
        *is_recording = true;
        drop(is_recording);

        let io = self.pcm_device.io_i16()?;
        let mut buffer = vec![0i16; self.frame_size];
        let start = Instant::now();
        let duration = Duration::from_secs(duration_secs);

        while Instant::now().duration_since(start) < duration && *self.is_recording.lock().unwrap() {
            match io.readi(&mut buffer) {
                Ok(_) => self.shared_buffer.push(buffer.clone()),
                Err(err) => {
                    eprintln!("Error while recording: {}", err);
                    break;
                }
            }
        }
        self.stop();

        Ok(())
    }

    pub fn stop(&self) {
        let mut is_recording = self.is_recording.lock().unwrap();
        *is_recording = false;
        self.shared_buffer.set_recording_finished();
    }
}
