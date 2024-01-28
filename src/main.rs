mod config;
pub mod recorder;
pub mod utils;

use recorder::Recorder;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("recordings")?;

    let _recorder = Recorder::new()?;

    // wait for keybord input
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    println!("Recording stopped, writing to file... This may take a while.");
    Ok(())
}
