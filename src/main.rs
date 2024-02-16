mod config;
pub mod recorder;
pub mod utils;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;

use clap::{App, Arg};
use crossbeam::channel::{unbounded, Receiver, Sender};
use recorder::channel_messages::RecorderToWriterChannelMessage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Plantwhispers Recorder")
            .version("1.0")
            .author("Simon Puschmann <imnos>")
            .about("Autonomous audio recorder for plant research.")
            .arg(
                Arg::with_name("destination")
                    .short("d")
                    .long("destination")
                    .value_name("DEST_PATH")
                    .help("Sets the sound file destination path")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("device_names")
                    .long("device-names")
                    .value_name("DEVICE_NAMES")
                    .help("Sets ALSA device names, separated by selicolon(;). Devices can be found using `arecord -l`.")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("file_duration")
                    .long("file-duration")
                    .value_name("FILE_DURATION_IN_SECONDS")
                    .help("Sets the file duration in seconds before a new file is created. Default is 30 seconds.")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("emitter_pin")
                    .long("emmiter-pin")
                    .value_name("SOUND_EMITTER_TRIGGER_PIN")
                    .help("Sets the sound emitter trigger pin number. Default is 2.")
                    .takes_value(true),
            )
            .get_matches();

    // Mandatory argument
    let destination_path = matches.value_of("destination").unwrap();

    // Optional arguments with defaults
    let device_names: Vec<&str> = matches
        .value_of("device_names")
        .unwrap_or("hw:0,0;hw:1,0")
        .split(';')
        .map(|s| s.trim())
        .collect();
    // TODO: Check if device names are valid
    let file_duration_in_seconds = matches
        .value_of("file_duration")
        .unwrap_or("30")
        .parse::<u32>()
        .expect("Time between resets should be an unsigned integer");
    let trigger_pin = matches
        .value_of("emmiter_pin")
        .unwrap_or("2")
        .parse::<u8>()
        .expect("Trigger pin should be an unsigned integer");

    // Use the arguments as needed
    println!("Destination Path: {}", destination_path);
    println!("Device Names: {:?}", device_names);
    println!("Time Between Resets: {}", file_duration_in_seconds);
    println!("Trigger Pin: {}", trigger_pin);

    // Logic

    let (tx, rx): (
        Sender<RecorderToWriterChannelMessage>,
        Receiver<RecorderToWriterChannelMessage>,
    ) = unbounded();

    let recorder_shutdown_signal = Arc::new(AtomicBool::new(false));
    let recorder_shutdown_signal_clone = Arc::clone(&recorder_shutdown_signal);

    let _recorder_thread = {
        thread::spawn(move || {
            recorder::recording_thread::recording_thread_logic(tx, recorder_shutdown_signal_clone)
        })
    };

    let _writer_thread = {
        thread::spawn(move || {
            recorder::writing_thread::writing_thread_logic(rx).expect("Writing thread failed");
        })
    };

    // wait for keybord input
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    println!("Recording stopped, writing to file... This may take a while.");

    recorder_shutdown_signal.store(true, std::sync::atomic::Ordering::SeqCst);

    _recorder_thread
        .join()
        .expect("Failed to join recording thread");
    _writer_thread
        .join()
        .expect("Failed to join writing thread");

    Ok(())
}
