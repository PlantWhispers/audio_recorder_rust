use crate::shared_buffer::{
    SharedBuffer,
    SharedBufferMessage::{Data, EndThread, NewFile},
};
use crate::writing::write_audio;
use alsa::pcm::PCM;
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
        pcm_device: PCM,
        shared_buffer: Arc<SharedBuffer>,
        frame_size: usize,
        time_between_resets_in_s: u32,
        channel_label: char,
    ) -> Result<Self, Box<dyn Error>> {
        let shutdown_signal = Arc::new(Mutex::new(false));

        let record_thread = {
            let shared_buffer_clone = Arc::clone(&shared_buffer);
            let shutdown_signal_clone = Arc::clone(&shutdown_signal);
            let samples_between_resets =
                time_between_resets_in_s * pcm_device.hw_params_current().unwrap().get_rate()?;
            thread::spawn(move || {
                Self::record_thread_logic(
                    pcm_device,
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
        pcm_device: PCM,
        shared_buffer: Arc<SharedBuffer>,
        frame_size: usize,
        samples_between_resets: u32,
        shutdown_signal: Arc<Mutex<bool>>,
        channel_label: char,
    ) {
        let pcm_io = pcm_device.io_i16().unwrap();
        let mut buffer = vec![0i16; frame_size];

        'outer: while !*shutdown_signal.lock().unwrap() {
            shared_buffer.push(NewFile(new_file_name(channel_label)));

            match pcm_device.reset() {
                Ok(_) => (),
                Err(err) => {
                    pcm_device.try_recover(err, false).unwrap();
                    continue 'outer;
                }
            }

            for _ in 0..(samples_between_resets / frame_size as u32) {
                match pcm_io.readi(&mut buffer) {
                    Ok(_) => {
                        shared_buffer.push(Data(buffer.clone()));
                    }
                    Err(err) => {
                        pcm_device.try_recover(err, false).unwrap();
                        continue 'outer;
                    }
                }
            }
        }
        shared_buffer.push(EndThread);
    }
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
