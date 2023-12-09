use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub struct SharedBuffer {
    queue: Mutex<VecDeque<Option<Vec<i16>>>>,
    condvar: Condvar,
}

impl SharedBuffer {
    pub fn new() -> Self {
        SharedBuffer {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
        }
    }

    pub fn push(&self, data: Option<Vec<i16>>) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(data);
        self.condvar.notify_one();
    }

    pub fn pull(&self) -> Option<Vec<i16>> {
        let mut queue = self.queue.lock().unwrap();
        while queue.is_empty() {
            queue = self.condvar.wait(queue).unwrap();
        }
        queue.pop_front().flatten()
    }
}

#[cfg(test)]
mod tests {
    use crate::shared_buffer::SharedBuffer;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_push_and_pull() {
        let buffer = SharedBuffer::new();
        buffer.push(Some(vec![1, 2, 3]));

        assert_eq!(buffer.pull(), Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_fifo_order() {
        let buffer = SharedBuffer::new();
        buffer.push(Some(vec![1]));
        buffer.push(Some(vec![2]));

        assert_eq!(buffer.pull(), Some(vec![1]));
        assert_eq!(buffer.pull(), Some(vec![2]));
    }

    #[test]
    fn test_concurrency() {
        let buffer = Arc::new(SharedBuffer::new());
        let buffer_clone = Arc::clone(&buffer);

        let producer = thread::spawn(move || {
            for i in 0..10 {
                buffer_clone.push(Some(vec![i]));
            }
        });

        let consumer = thread::spawn(move || {
            for _ in 0..10 {
                let data = buffer.pull();
                assert!(data.is_some());
            }
        });

        producer.join().expect("Producer thread panicked");
        consumer.join().expect("Consumer thread panicked");
    }

    #[test]
    fn test_blocking_behavior() {
        let buffer = Arc::new(SharedBuffer::new());
        let buffer_clone = Arc::clone(&buffer);

        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_secs(1));
            buffer_clone.push(Some(vec![1]));
        });

        let start_time = std::time::Instant::now();
        let pulled_data = buffer.pull();
        let duration = start_time.elapsed();

        assert!(pulled_data.is_some());
        assert!(duration.as_secs() >= 1);
    }

    #[test]
    fn test_none_values() {
        let buffer = SharedBuffer::new();
        buffer.push(None);

        assert_eq!(buffer.pull(), None);
    }

    #[test]
    fn stress_test() {
        let buffer = Arc::new(SharedBuffer::new());
        let mut handles = vec![];

        for _ in 0..100 {
            let buffer_clone = Arc::clone(&buffer);
            handles.push(thread::spawn(move || {
                for i in 0..1000 {
                    buffer_clone.push(Some(vec![i]));
                }
            }));
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        for _ in 0..100000 {
            assert!(buffer.pull().is_some());
        }
    }
}
