use crate::channel_messages::RecorderToWriterChannelMessage::{self, Data, EndThread, NewFile};
use crate::pcm_setup::setup_pcm;
use crate::{BUFFER_SIZE, N_OF_BUFFERS_PER_FILE};
use alsa::pcm::{IO, PCM};
use crossbeam::channel::Sender;
use rppal::gpio::Gpio;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

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

fn get_mic_data(pcm_device: &PCM, pcm_io: &IO<'_, i16>) -> Result<Vec<i16>, Box<dyn Error>> {
    let mut buffer = vec![0i16; BUFFER_SIZE];
    match pcm_io.readi(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(err) => {
            if pcm_device.try_recover(err, false).is_err() {
                panic!("Failed to recover from ALSA error: {}", err);
            }
            Err(err.into())
        }
    }
}

fn new_file_name() -> String {
    format!(
        "recordings/{}.raw.wav",
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
}
