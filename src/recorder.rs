use crate::recording_thread::recording_thread_logic;
use crate::shared_buffer::SharedBufferMessage;
use crate::writing_thread::writing_thread_logic;
use alsa::pcm::PCM;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub struct Recorder {
    shutdown_signal: Arc<AtomicBool>,
    record_thread: Option<JoinHandle<()>>,
    write_thread: Option<JoinHandle<()>>,
}

impl Recorder {
    pub fn new(pcm_devices: [PCM; 2]) -> Result<Self, Box<dyn Error>> {
        let (sender, receiver): (Sender<SharedBufferMessage>, Receiver<SharedBufferMessage>) =
            unbounded();

        let shutdown_signal = Arc::new(AtomicBool::new(false));
        let shutdown_signal_clone = Arc::clone(&shutdown_signal);

        let record_thread = {
            thread::spawn(move || {
                recording_thread_logic(pcm_devices, sender, shutdown_signal_clone)
            })
        };

        let write_thread = {
            thread::spawn(move || {
                writing_thread_logic(receiver).expect("Failed to write audio to file");
            })
        };

        Ok(Recorder {
            shutdown_signal,
            record_thread: Some(record_thread),
            write_thread: Some(write_thread),
        })
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        self.shutdown_signal.store(true, Ordering::SeqCst);

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
