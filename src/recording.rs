use crate::shared_buffer::{
    SharedBuffer,
    SharedBufferMessage::{Data, EndThread, NewFile},
};
use crate::writing::write_audio;
use alsa::pcm::PCM;
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
        shared_buffer: Arc<SharedBuffer>,
        frame_size: usize,
        time_between_resets_in_s: u32,
        channel_label: char,
    ) -> Result<Self, Box<dyn Error>> {
        let shutdown_signal = Arc::new(Mutex::new(false));

        let record_thread = {
            let shared_buffer_clone = Arc::clone(&shared_buffer);
            let shutdown_signal_clone = Arc::clone(&shutdown_signal);
            let samples_between_resets = time_between_resets_in_s
                * pcm_devices[0].hw_params_current().unwrap().get_rate()?;
            thread::spawn(move || {
                Self::record_thread_logic(
                    pcm_devices,
                    shared_buffer_clone,
                    frame_size,
                    samples_between_resets,
                    shutdown_signal_clone,
                    channel_label,
                )
            })
        };

        let write_thread = {
            let shared_buffer_clone = Arc::clone(&shared_buffer);
            thread::spawn(move || {
                write_audio(shared_buffer_clone).expect("Failed to write audio to file");
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
        shared_buffer: Arc<SharedBuffer>,
        frame_size: usize,
        samples_between_resets: u32,
        shutdown_signal: Arc<Mutex<bool>>,
        channel_label: char,
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
            shared_buffer.push(NewFile(new_file_name(channel_label)));

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
                shared_buffer.push(Data([mics[0].buffer.clone(), mics[1].buffer.clone()]));
            }
        }
        shared_buffer.push(EndThread);
    }
}
struct Microphone<'a> {
    pcm_device: &'a PCM,
    pcm_io: alsa::pcm::IO<'a, i16>,
    buffer: Vec<i16>,
}

fn new_file_name(channel_label: char) -> String {
    format!(
        "recordings/{}{}.wav",
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        channel_label
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
