use crate::shared_buffer::SharedBufferMessage::{self, Data, EndThread, NewFile};
use crate::writing::write_audio;
use alsa::pcm::PCM;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::SystemTime;
use std::{error::Error, vec};

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
        let mut mics = pcm_devices
            .iter()
            .map(|device| {
                let pcm_io = device.io_i16().unwrap();
                Microphone {
                    pcm_device: device,
                    pcm_io,
                    buffer: vec![0i16; frame_size],
                }
            })
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
                for mic in &mut mics {
                    match mic.pcm_io.readi(&mut mic.buffer) {
                        Ok(_) => {}
                        Err(err) => {
                            mic.pcm_device.try_recover(err, false).unwrap();
                            continue 'outer;
                        }
                    }
                }
                sender
                    .send(Data([mics[0].buffer.clone(), mics[1].buffer.clone()]))
                    .unwrap();
            }
        }
        sender.send(EndThread).unwrap();
    }
}
struct Microphone<'a> {
    pcm_device: &'a PCM,
    pcm_io: alsa::pcm::IO<'a, i16>,
    buffer: Vec<i16>,
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
