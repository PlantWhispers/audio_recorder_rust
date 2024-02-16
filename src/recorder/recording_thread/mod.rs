mod utils;
use super::channel_messages::RecorderToWriterChannelMessage::{self, Data, EndThread, NewFile};
use crate::config::{BUFFER_SIZE, SAMPLE_RATE};
use crossbeam::channel::Sender;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::SystemTime,
};
use utils::get_mic_data;

pub fn recording_thread_logic<F: FnMut()>(
    sender: Sender<RecorderToWriterChannelMessage>,
    shutdown_signal: Arc<AtomicBool>,
    pcm_devices: [alsa::pcm::PCM; 2],
    file_duration: u32,
    mut emitt_sound: F,
    destination_folder: PathBuf,
) {
    pcm_devices[0].start().unwrap();
    let pcm_ios = pcm_devices
        .iter()
        .map(|device| device.io_i16().unwrap())
        .collect::<Vec<_>>();
    let n_of_buffers_per_file = file_duration * SAMPLE_RATE / BUFFER_SIZE as u32;

    // Main recording loop
    'main_recording_loop: while !shutdown_signal.load(Ordering::SeqCst) {
        let file_name = format!(
            "{}.wav",
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        let file_path = destination_folder.join(file_name);

        sender.send(NewFile(file_path)).unwrap();

        pcm_devices[0].reset().unwrap();

        emitt_sound();

        for _ in 0..n_of_buffers_per_file {
            let data = {
                (
                    get_mic_data(&pcm_devices[0], &pcm_ios[0]),
                    get_mic_data(&pcm_devices[1], &pcm_ios[1]),
                )
            };
            match data {
                (Ok(a), Ok(b)) => {
                    sender.send(Data([a, b])).unwrap();
                }
                _ => continue 'main_recording_loop,
            }
        }
    }
    sender.send(EndThread).unwrap();
}
