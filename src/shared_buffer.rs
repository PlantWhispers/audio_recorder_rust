use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub enum SharedBufferMessage {
    NewFile(String),
    Data(Vec<i16>),
    EndOfFile,
    EndThread,
}

pub struct SharedBuffer {
    queue: Mutex<VecDeque<SharedBufferMessage>>,
    condvar: Condvar,
}

impl SharedBuffer {
    pub fn new() -> Self {
        SharedBuffer {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
        }
    }

    pub fn push(&self, message: SharedBufferMessage) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(message);
        self.condvar.notify_one();
    }

    pub fn pull(&self) -> Option<SharedBufferMessage> {
        let mut queue = self.queue.lock().unwrap();
        while queue.is_empty() {
            queue = self.condvar.wait(queue).unwrap();
        }
        queue.pop_front()
    }
}
