use crate::shared_buffer::SharedBufferMessage::{self, Data, EndThread, NewFile};
use crate::writing::write_audio;
use alsa::pcm::{IO, PCM};
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::SystemTime;

pub struct Recorder {
    shutdown_signal: Arc<Mutex<bool>>,
    record_thread: Option<JoinHandle<()>>,
    write_thread: Option<JoinHandle<()>>,
}

impl Recorder {
    pub fn new(
        // array of 2 PCM devices
        pcm_devices: [PCM; 2],
        frame_size: usize,
        time_between_resets_in_s: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let (sender, receiver): (Sender<SharedBufferMessage>, Receiver<SharedBufferMessage>) =
            unbounded();

        let shutdown_signal = Arc::new(Mutex::new(false));

        let record_thread = {
            let shutdown_signal_clone = Arc::clone(&shutdown_signal);
            let samples_between_resets = time_between_resets_in_s
                * pcm_devices[0].hw_params_current().unwrap().get_rate()?;
            thread::spawn(move || {
                Self::record_thread_logic(
                    pcm_devices,
                    sender,
                    frame_size,
                    samples_between_resets,
                    shutdown_signal_clone,
                )
            })
        };

        let write_thread = {
            thread::spawn(move || {
                write_audio(receiver).expect("Failed to write audio to file");
            })
        };

        Ok(Recorder {
            shutdown_signal,
            record_thread: Some(record_thread),
            write_thread: Some(write_thread),
        })
    }

    fn record_thread_logic(
        pcm_devices: [PCM; 2],
        sender: Sender<SharedBufferMessage>,
        frame_size: usize,
        samples_between_resets: u32,
        shutdown_signal: Arc<Mutex<bool>>,
    ) {
        let pcm_ios = pcm_devices
            .iter()
            .map(|device| device.io_i16().unwrap())
            .collect::<Vec<_>>();

        'outer: while !*shutdown_signal.lock().unwrap() {
            sender.send(NewFile(new_file_name())).unwrap();

            for pcm_device in pcm_devices.iter() {
                match pcm_device.reset() {
                    Ok(_) => {}
                    Err(err) => {
                        pcm_device.try_recover(err, false).unwrap();
                        continue 'outer;
                    }
                }
            }

            for _ in 0..(samples_between_resets / frame_size as u32) {
                let data = {
                    (
                        get_mic_data(&pcm_devices[0], &pcm_ios[1]),
                        get_mic_data(&pcm_devices[1], &pcm_ios[1]),
                    )
                };
                match data {
                    (Ok(a), Ok(b)) => {
                        sender.send(Data([a, b])).unwrap();
                    }

                    _ => {
                        continue 'outer;
                    }
                }
            }
        }
        sender.send(EndThread).unwrap();
    }
}

fn get_mic_data(pcm_device: &PCM, pcm_io: &IO<'_, i16>) -> Result<[i16; 1920], Box<dyn Error>> {
    let mut buffer = [0i16; 1920];
    match pcm_io.readi(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(err) => {
            pcm_device.try_recover(err, false)?;
            Err(err.into())
        }
    }
}

fn new_file_name() -> String {
    format!(
        "recordings/{}.wav",
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
}

impl Drop for Recorder {
    fn drop(&mut self) {
        *self.shutdown_signal.lock().unwrap() = true;

        if let Some(thread_handle) = self.record_thread.take() {
            thread_handle
                .join()
                .expect("Failed to join recording thread");
        }
        if let Some(thread_handle) = self.write_thread.take() {
            thread_handle.join().expect("Failed to join writing thread");
        }
    }
}
