use crate::channel_messages::RecorderToWriterChannelMessage;
use crate::recording_thread::recording_thread_logic;
use crate::writing_thread::writing_thread_logic;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub struct Recorder {
    recording_thread_shutdown_signal: Arc<AtomicBool>,
    recording_thread: Option<JoinHandle<()>>,
    writing_thread: Option<JoinHandle<()>>,
}

impl Recorder {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let (sender, receiver): (
            Sender<RecorderToWriterChannelMessage>,
            Receiver<RecorderToWriterChannelMessage>,
        ) = unbounded();

        let recording_thread_shutdown_signal = Arc::new(AtomicBool::new(false));
        let recording_thread_shutdown_signal_clone = Arc::clone(&recording_thread_shutdown_signal);

        let recording_thread = {
            thread::spawn(move || {
                recording_thread_logic(sender, recording_thread_shutdown_signal_clone)
            })
        };

        let writing_thread = {
            thread::spawn(move || {
                writing_thread_logic(receiver).expect("Failed to write audio to file");
            })
        };

        Ok(Recorder {
            recording_thread_shutdown_signal,
            recording_thread: Some(recording_thread),
            writing_thread: Some(writing_thread),
        })
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        self.recording_thread_shutdown_signal
            .store(true, Ordering::SeqCst);

        if let Some(thread_handle) = self.recording_thread.take() {
            thread_handle
                .join()
                .expect("Failed to join recording thread");
        }
        if let Some(thread_handle) = self.writing_thread.take() {
            thread_handle.join().expect("Failed to join writing thread");
        }
    }
}
