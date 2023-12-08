use std::sync::{Mutex, Condvar};
use std::collections::VecDeque;

pub struct SharedBuffer {
    queue: Mutex<VecDeque<Vec<i16>>>,
    condvar: Condvar,
    is_recording_finished: Mutex<bool>,
}

impl SharedBuffer {
    pub fn new() -> Self {
        SharedBuffer {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
            is_recording_finished: Mutex::new(false),
        }
    }

    pub fn push(&self, data: Vec<i16>) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(data);
        self.condvar.notify_one();
    }

    pub fn pull(&self) -> Option<Vec<i16>> {
        let mut queue = self.queue.lock().unwrap();
        while queue.is_empty() {
            if *self.is_recording_finished.lock().unwrap() {
                return None; // Return None if recording is finished and buffer is empty
            }
            queue = self.condvar.wait(queue).unwrap();
        }
        queue.pop_front()
    }

    pub fn set_recording_finished(&self) {
        let mut finished = self.is_recording_finished.lock().unwrap();
        *finished = true;
        self.condvar.notify_all(); // Wake up all waiting threads
    }

}
