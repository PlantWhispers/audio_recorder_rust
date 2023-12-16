mod channel_messages;
mod pcm_setup;
mod recorder;
mod recording_thread;
mod writing_thread;

use alsa::pcm::{Access, Format};
use pcm_setup::setup_pcm;
use recorder::Recorder;
use std::fs;

const SAMPLE_RATE: u32 = 384_000;
const CHANNELS: u16 = 1;
const FORMAT: Format = Format::S16LE;
const ACCESS: Access = Access::RWInterleaved;
const ALSA_BUFFER_SIZE: usize = 19200; // Adjust as needed
const BUFFER_SIZE: usize = 1920; // Adjust as needed
const TIME_BETWEEN_RESETS_IN_S: u32 = 30;
const N_OF_BUFFERS_PER_FILE: u32 = TIME_BETWEEN_RESETS_IN_S * SAMPLE_RATE / BUFFER_SIZE as u32;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("recordings")?;

    let (pcm_a, pcm_b) = setup_pcm("hw:0,0", "hw:1,0")?;

    let _recorder = Recorder::new([pcm_a, pcm_b])?;

    // wait for keybord input
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    println!("Recording stopped, writing to file... This may take a while.");
    Ok(())
}
