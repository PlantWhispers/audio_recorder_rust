use std::error::Error;
use std::sync::{Arc, Mutex, Condvar};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use alsa::pcm::PCM;
use crate::shared_buffer::SharedBuffer;

pub struct Recorder {
    is_recording: Arc<Mutex<bool>>,
    duration: Arc<Mutex<Duration>>,
    shutdown_signal: Arc<Mutex<bool>>,
    condvar: Arc<Condvar>,
    thread_handle: Option<JoinHandle<()>>,
}

impl Recorder {
    pub fn new(
        pcm_device: PCM,
        shared_buffer: Arc<SharedBuffer>,
        frame_size: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let is_recording = Arc::new(Mutex::new(false));
        let duration = Arc::new(Mutex::new(Duration::from_secs(0)));
        let shutdown_signal = Arc::new(Mutex::new(false));
        let condvar = Arc::new(Condvar::new());

        let is_recording_clone = Arc::clone(&is_recording);
        let duration_clone = Arc::clone(&duration);
        let shutdown_signal_clone = Arc::clone(&shutdown_signal);
        let condvar_clone = Arc::clone(&condvar);

        let thread_handle = thread::spawn(move || {
            let io = pcm_device.io_i16().expect("Failed to open PCM device");
            let mut buffer = vec![0i16; frame_size];

            loop {
                {
                    let mut is_rec = is_recording_clone.lock().unwrap();
                    while !*is_rec && !*shutdown_signal_clone.lock().unwrap() {
                        is_rec = condvar_clone.wait(is_rec).unwrap();
                    }

                    if *shutdown_signal_clone.lock().unwrap() {
                        break;
                    }
                }

                let start = Instant::now();
                pcm_device.reset().expect("Failed to reset PCM device");
                while *is_recording_clone.lock().unwrap() {
                    match io.readi(&mut buffer) {
                        Ok(_) => shared_buffer.push(Some(buffer.clone())),
                        Err(err) => {
                            eprintln!("Error while recording: {}", err);
                            break;
                        }
                    }

                    if Instant::now().duration_since(start) >= *duration_clone.lock().unwrap() {
                        break;
                    }
                }
                shared_buffer.push(None);
            }
        });

        Ok(Recorder {
            is_recording,
            duration,
            shutdown_signal,
            condvar,
            thread_handle: Some(thread_handle),
        })
    }

    pub fn start(&self, duration_secs: u64) {
        let mut duration = self.duration.lock().unwrap();
        *duration = Duration::from_secs(duration_secs);

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
        self.stop();

        *self.shutdown_signal.lock().unwrap() = true;
        self.condvar.notify_one();

        if let Some(thread_handle) = self.thread_handle.take() {
            thread_handle.join().expect("Failed to join recording thread");
        }
    }
}
