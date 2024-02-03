mod utils;
use super::channel_messages::RecorderToWriterChannelMessage::{self, Data, EndThread, NewFile};
use crate::{
    config::{N_OF_BUFFERS_PER_FILE, SOUND_EMITTER_TRIGGER_PIN},
    utils::{hc_sr04::HcSr04SoundEmitter, pcm_setup::setup_pcm},
};
use crossbeam::channel::Sender;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use utils::{get_mic_data, new_file_name};

pub fn recording_thread_logic(
    sender: Sender<RecorderToWriterChannelMessage>,
    shutdown_signal: Arc<AtomicBool>,
) {
    let mut sound_emitter = HcSr04SoundEmitter::new(SOUND_EMITTER_TRIGGER_PIN).unwrap();

    let pcm_devices = setup_pcm().unwrap();
    pcm_devices[0].link(&pcm_devices[1]).unwrap();
    pcm_devices[0].start().unwrap();
    let pcm_ios = pcm_devices
        .iter()
        .map(|device| device.io_i16().unwrap())
        .collect::<Vec<_>>();

    // Main recording loop
    'main_recording_loop: while !shutdown_signal.load(Ordering::SeqCst) {
        sender.send(NewFile(new_file_name())).unwrap();

        pcm_devices[0].reset().unwrap();
        sound_emitter.emit_sound();

        for _ in 0..N_OF_BUFFERS_PER_FILE {
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
