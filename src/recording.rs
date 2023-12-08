use crate::shared_buffer::SharedBuffer;
use alsa::pcm::PCM;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub fn record_audio(pcm: PCM, shared_buffer: Arc<SharedBuffer>, frame_size: usize, duration_secs: u64) {
    let io = pcm.io_i16().expect("Failed to open PCM device");
    let mut buffer = vec![0i16; frame_size];

    let start = Instant::now();
    let duration = Duration::from_secs(duration_secs);

    while Instant::now().duration_since(start) < duration {
        match io.readi(&mut buffer) {
            Ok(_) => shared_buffer.push(buffer.clone()),
            Err(err) => {
                eprintln!("Error while recording: {}", err);
                break;
            }
        }
    }
    shared_buffer.set_recording_finished();
    println!("Finished recording")
}
