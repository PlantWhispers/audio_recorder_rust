mod pcm_setup;
mod recording;
mod shared_buffer;
mod writing;

use alsa::pcm::{Access, Format};
use pcm_setup::setup_pcm;
use recording::Recorder;
use shared_buffer::SharedBuffer;
use std::fs;
use std::sync::Arc;
use std::thread;

const SAMPLE_RATE: u32 = 384_000;
const CHANNELS: u16 = 1;
const FORMAT: Format = Format::S16LE;
const ACCESS: Access = Access::RWInterleaved;
const BUFFER_SIZE: usize = 1920; // Adjust as needed
const FRAME_SIZE: usize = 1920; // Adjust as needed

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("recordings")?;

    let shared_buffer_a = Arc::new(SharedBuffer::new());
    let shared_buffer_b = Arc::new(SharedBuffer::new());

    let (pcm_a, pcm_b) = setup_pcm(
        "hw:0,0",
        "hw:1,0",
        SAMPLE_RATE,
        CHANNELS,
        FORMAT,
        ACCESS,
        BUFFER_SIZE as i64,
    )?;

    let recorder_a = Recorder::new(pcm_a, Arc::clone(&shared_buffer_a), FRAME_SIZE, 3, 'A')?;
    let recorder_b = Recorder::new(pcm_b, Arc::clone(&shared_buffer_b), FRAME_SIZE, 3, 'B')?;

    // sleep for 10 seconds
    thread::sleep(std::time::Duration::from_secs(10));

    println!("Done");

    drop(recorder_a);
    drop(recorder_b);

    Ok(())
}
