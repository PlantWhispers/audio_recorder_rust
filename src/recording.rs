use crate::shared_buffer::SharedBuffer;
use alsa::pcm::PCM;
use std::error::Error;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

pub struct Recorder {
    is_recording: Arc<Mutex<bool>>,
    shutdown_signal: Arc<Mutex<bool>>,
    condvar: Arc<Condvar>,
    thread_handle: Option<JoinHandle<()>>,
}

impl Recorder {
    pub fn new(
        pcm_device: PCM,
        shared_buffer: Arc<SharedBuffer>,
        frame_size: usize,
        time_between_resets_in_s: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let is_recording = Arc::new(Mutex::new(false));
        let shutdown_signal = Arc::new(Mutex::new(false));
        let condvar = Arc::new(Condvar::new());

        let thread_handle = {
            let is_recording_clone = Arc::clone(&is_recording);
            let samples_between_resets =
                time_between_resets_in_s * pcm_device.hw_params_current().unwrap().get_rate()?;
            let shutdown_signal_clone = Arc::clone(&shutdown_signal);
            let condvar_clone = Arc::clone(&condvar);
            thread::spawn(move || {
                Self::record_thread_logic(
                    pcm_device,
                    shared_buffer,
                    frame_size,
                    is_recording_clone,
                    samples_between_resets,
                    shutdown_signal_clone,
                    condvar_clone,
                )
            })
        };

        Ok(Recorder {
            is_recording,
            shutdown_signal,
            condvar,
            thread_handle: Some(thread_handle),
        })
    }

    fn record_thread_logic(
        pcm_device: PCM,
        shared_buffer: Arc<SharedBuffer>,
        frame_size: usize,
        is_recording: Arc<Mutex<bool>>,
        samples_between_resets: u32,
        shutdown_signal: Arc<Mutex<bool>>,
        condvar: Arc<Condvar>,
    ) {
        let pcm_io = pcm_device.io_i16().unwrap();
        let mut buffer = vec![0i16; frame_size];

        loop {
            {
                let mut is_recording = is_recording.lock().unwrap();
                while !*is_recording && !*shutdown_signal.lock().unwrap() {
                    is_recording = condvar.wait(is_recording).unwrap();
                }

                if *shutdown_signal.lock().unwrap() {
                    break;
                }
            }

            while *is_recording.lock().unwrap() {
                pcm_device.reset().unwrap();
                for _ in 0..(samples_between_resets / frame_size as u32) {
                    match pcm_io.readi(&mut buffer) {
                        Ok(_) => shared_buffer.push(Some(buffer.clone())),
                        Err(err) => {
                            eprintln!("Error while recording: {}", err);
                            break;
                        }
                    }
                }
                shared_buffer.push(None);
                println!("Resetting PCM")
            }
        }
    }

    pub fn start(&self) {
        let mut is_recording = self.is_recording.lock().unwrap();
        *is_recording = true;
        self.condvar.notify_one();
    }

    pub fn stop(&self) {
        let mut is_recording = self.is_recording.lock().unwrap();
        *is_recording = false;
        self.condvar.notify_one();
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        *self.shutdown_signal.lock().unwrap() = true;
        self.stop();

        if let Some(thread_handle) = self.thread_handle.take() {
            thread_handle
                .join()
                .expect("Failed to join recording thread");
        }
    }
}
