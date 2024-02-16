mod config;
pub mod recorder;
pub mod utils;
use std::sync::Arc;
use std::thread;
use std::{path::PathBuf, sync::atomic::AtomicBool};

use clap::{App, Arg};
use crossbeam::channel::{unbounded, Receiver, Sender};
use recorder::channel_messages::RecorderToWriterChannelMessage;

use crate::{
    config::{DEFAULT_DEVICE_NAMES, DEFAULT_FILE_DURATION, DEFAULT_SOUND_EMITTER_TRIGGER_PIN},
    utils::pcm_setup::setup_pcm,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command_line_arguments = App::new("Plantwhispers Recorder")
            .version("1.0")
            .author("Simon Puschmann <imnos>")
            .about("Autonomous audio recorder for plant research.")
            .arg(
                Arg::with_name("experiment_name")
                    .short("n")
                    .long("experiment-name")
                    .value_name("EXPERIMENT_NAME")
                    .help("Sets the name of the current experiment")
                    .takes_value(true)
                    .required(true),
            )
            .arg( Arg::with_name("path")
                .short("p")
                .long("path")
                .value_name("PATH")
                .help("Sets the path to the folder where the sound files will be stored")
                .takes_value(true)
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
    let experiment_name: PathBuf = command_line_arguments
        .value_of("experiment_name")
        .unwrap()
        .parse()
        .expect("Name of the experiment did not parse to a valid path");

    // Optional arguments with defaults
    let sound_path: PathBuf = command_line_arguments
        .value_of("path")
        .unwrap_or("./")
        .parse()
        .expect("Path did not parse to a valid path");
    let destination_path = sound_path.join(experiment_name);
    let device_names: Vec<&str> = command_line_arguments
        .value_of("device_names")
        .unwrap_or(DEFAULT_DEVICE_NAMES)
        .split(';')
        .map(|s| s.trim())
        .collect();
    // TODO: Test
    // TODO: Check if device names are valid
    let file_duration_in_seconds = command_line_arguments
        .value_of("file_duration")
        .unwrap_or(DEFAULT_FILE_DURATION)
        .parse::<u32>()
        .expect("Time between resets should be an unsigned integer");
    let trigger_pin = command_line_arguments
        .value_of("emmiter_pin")
        .unwrap_or(DEFAULT_SOUND_EMITTER_TRIGGER_PIN)
        .parse::<u8>()
        .expect("Trigger pin should be an unsigned integer");

    // Logic

    let (tx, rx): (
        Sender<RecorderToWriterChannelMessage>,
        Receiver<RecorderToWriterChannelMessage>,
    ) = unbounded();

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_signal_clone = Arc::clone(&shutdown_signal);
    let pcm_devices = setup_pcm(device_names).unwrap();
    let mut sound_emitter = utils::hc_sr04::HcSr04SoundEmitter::new(trigger_pin).unwrap();
    let emitt_sound = move || sound_emitter.emit_sound();
    // TODO: TEST sound emitter!

    let _recorder_thread = {
        thread::spawn(move || {
            recorder::recording_thread::recording_thread_logic(
                tx,
                shutdown_signal_clone,
                pcm_devices,
                file_duration_in_seconds,
                emitt_sound,
                destination_path,
            );
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

    shutdown_signal.store(true, std::sync::atomic::Ordering::SeqCst);

    _recorder_thread
        .join()
        .expect("Failed to join recording thread");
    _writer_thread
        .join()
        .expect("Failed to join writing thread");

    Ok(())
}
