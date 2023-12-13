mod pcm_setup;
mod shared_buffer;
mod recording;
mod writing;

use pcm_setup::setup_pcm;
use shared_buffer::SharedBuffer;
use recording::Recorder;
use writing::write_audio;
use alsa::pcm::{Format, Access};
use std::sync::Arc;
use std::thread;
use std::fs;

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

    let (pcm_a, pcm_b) = setup_pcm("hw:0,0", "hw:1,0", SAMPLE_RATE, CHANNELS, FORMAT, ACCESS, BUFFER_SIZE as i64)?;

    let recorder_a = Recorder::new(pcm_a, Arc::clone(&shared_buffer_a), FRAME_SIZE, 10)?;
    let recorder_b = Recorder::new(pcm_b, Arc::clone(&shared_buffer_b), FRAME_SIZE, 10)?;

    recorder_a.start();
    recorder_b.start();

    // Spawn writing threads
    let writing_thread_a = thread::spawn(move || {
        write_audio('A', shared_buffer_a).expect("Failed to write audio to file");
    });

    let writing_thread_b = thread::spawn(move || {
        write_audio('B', shared_buffer_b).expect("Failed to write audio to file");
    });

    recorder_a.stop();
    recorder_b.stop();

    // Wait for threads to complete
    writing_thread_a.join().unwrap();
    writing_thread_b.join().unwrap();

    println!("Done");

    Ok(())
}
