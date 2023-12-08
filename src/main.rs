mod pcm_setup;
mod shared_buffer;
mod recording;
mod writing;

use pcm_setup::setup_pcm;
use shared_buffer::SharedBuffer;
use recording::record_audio;
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

    let pcm_a = setup_pcm("hw:2,0", SAMPLE_RATE, CHANNELS, FORMAT, ACCESS, BUFFER_SIZE as i64)?;
    let pcm_b = setup_pcm("hw:3,0", SAMPLE_RATE, CHANNELS, FORMAT, ACCESS, BUFFER_SIZE as i64)?;

    let buffer_a_clone = Arc::clone(&shared_buffer_a);
    let buffer_b_clone = Arc::clone(&shared_buffer_b);

    // Spawn recording threads
    let recording_thread_a = thread::spawn(move || {
        record_audio(pcm_a, buffer_a_clone, FRAME_SIZE, 10);
    });

    let recording_thread_b = thread::spawn(move || {
        record_audio(pcm_b, buffer_b_clone, FRAME_SIZE, 10);
    });

    // Spawn writing threads
    let writing_thread_a = thread::spawn(move || {
        write_audio('A', shared_buffer_a).expect("Failed to write audio to file");
    });

    let writing_thread_b = thread::spawn(move || {
        write_audio('B', shared_buffer_b).expect("Failed to write audio to file");
    });

    // Wait for threads to complete
    recording_thread_a.join().unwrap();
    recording_thread_b.join().unwrap();
    writing_thread_a.join().unwrap();
    writing_thread_b.join().unwrap();

    Ok(())
}
