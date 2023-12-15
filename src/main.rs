mod pcm_setup;
mod recording;
mod shared_buffer;
mod writing;

use alsa::pcm::{Access, Format};
use pcm_setup::setup_pcm;
use recording::Recorder;
use std::fs;
use std::thread;

const SAMPLE_RATE: u32 = 384_000;
const CHANNELS: u16 = 1;
const FORMAT: Format = Format::S16LE;
const ACCESS: Access = Access::RWInterleaved;
const ALSA_BUFFER_SIZE: usize = 19200; // Adjust as needed
const BUFFER_SIZE: usize = 1920; // Adjust as needed

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("recordings")?;

    let (pcm_a, pcm_b) = setup_pcm("hw:0,0", "hw:1,0")?;

    let recorder = Recorder::new([pcm_a, pcm_b], 3)?;

    // sleep for 10 seconds
    thread::sleep(std::time::Duration::from_secs(10));

    println!("Done");

    drop(recorder);

    Ok(())
}
