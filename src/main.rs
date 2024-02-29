mod config;
mod recording_thread;
mod utils;
mod writing_thread;
use crate::recording_thread::recording_thread_logic;
use crate::utils::hc_sr04::HcSr04SoundEmitter;
use crate::writing_thread::writing_thread_logic;
use crate::{
    config::{DEFAULT_DEVICE_NAMES, DEFAULT_FILE_DURATION, DEFAULT_SOUND_TRIGGER_PIN},
    utils::pcm_setup::setup_pcm,
};
use clap::Parser;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{path::PathBuf, sync::atomic::AtomicBool};
use utils::channel_messages::RecorderToWriterChannelMessage;

#[derive(Parser, Debug)]
#[clap(
    name = "Plantwhispers Recorder",
    version = "1.0",
    author = "Simon Puschmann <imnos>",
    about = "Autonomous audio recorder for plant research."
)]

struct Args {
    #[clap(
        short = 'e',
        long = "experiment-name",
        value_name = "EXPERIMENT_NAME",
        required = true
    )]
    experiment_name: PathBuf,
    #[clap(short = 'p', long = "path", value_name = "PATH")]
    path: Option<PathBuf>,
    #[clap(
        long = "device-names",
        value_name = "DEVICE_NAMES",
        number_of_values = 2
    )]
    device_names: Option<Vec<String>>,
    #[clap(long = "file-duration", value_name = "FILE_DURATION_IN_SECONDS")]
    file_duration: Option<u64>,
    #[clap(long = "emmiter-pin", value_name = "SOUND_EMITTER_TRIGGER_PIN")]
    emitter_pin: Option<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let experiment_name = args.experiment_name;
    let sound_path = args.path.unwrap_or("/home/pi/raw-data/".parse()?);
    let destination_folder = sound_path.join(experiment_name);
    let device_names: Vec<&str> = args
        .device_names
        .as_ref()
        .map(|v| v.iter().map(AsRef::as_ref).collect())
        .unwrap_or(DEFAULT_DEVICE_NAMES.to_vec());
    let file_duration =
        Duration::from_secs(args.file_duration.unwrap_or(DEFAULT_FILE_DURATION.parse()?));
    let trigger_pin = args
        .emitter_pin
        .unwrap_or(DEFAULT_SOUND_TRIGGER_PIN.parse()?);

    // Logic
    let (sender, receiver): (
        Sender<RecorderToWriterChannelMessage>,
        Receiver<RecorderToWriterChannelMessage>,
    ) = unbounded();

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_signal_clone = Arc::clone(&shutdown_signal);
    let pcm_devices = setup_pcm(device_names).expect("The specified devices could not be set up.");
    let sound_emitter = HcSr04SoundEmitter::new(trigger_pin).unwrap();
    // TODO: TEST sound emitter!

    let _recorder_thread = {
        thread::spawn(move || {
            recording_thread_logic(
                sender,
                shutdown_signal_clone,
                pcm_devices,
                file_duration,
                sound_emitter,
                destination_folder,
            );
        })
    };

    let _writer_thread = {
        thread::spawn(move || {
            writing_thread_logic(receiver).expect("Writing thread failed");
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
