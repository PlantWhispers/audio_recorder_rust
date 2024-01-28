mod utils;
use crate::utils::channel_messages::RecorderToWriterChannelMessage::{
    self, Data, EndThread, NewFile,
};
use crate::utils::pcm_setup::setup_pcm;
use crate::N_OF_BUFFERS_PER_FILE;
use crossbeam::channel::Sender;
use rppal::gpio::Gpio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use utils::get_mic_data;
use utils::new_file_name;

pub fn recording_thread_logic(
    sender: Sender<RecorderToWriterChannelMessage>,
    shutdown_signal: Arc<AtomicBool>,
) {
    let pcm_devices = setup_pcm().unwrap();

    let gpio = Gpio::new().unwrap();
    let mut tranciver_trigger = gpio.get(2).unwrap().into_output();
    tranciver_trigger.set_low();

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

        tranciver_trigger.set_high();
        thread::sleep(Duration::from_micros(10));
        tranciver_trigger.set_low();

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
