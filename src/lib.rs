use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Message>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ThreadPoolBuildError;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    pub fn build(size: usize) -> Result<Self, ThreadPoolBuildError> {
        if size == 0 {
            return Err(ThreadPoolBuildError);
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(Self {
            workers,
            sender: Some(sender),
        })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender
            .as_ref()
            .unwrap()
            .send(Message::NewJob(job))
            .unwrap();
    }
}

impl std::fmt::Display for ThreadPoolBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Thread pool size must be greater than zero.")
    }
}

impl std::error::Error for ThreadPoolBuildError {}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            for _ in &self.workers {
                sender.send(Message::Terminate).unwrap();
            }
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        job();
                    }
                    Message::Terminate => {
                        break;
                    }
                }
            }
        });

        Self {
            _id: id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ThreadPool, ThreadPoolBuildError};
    use std::sync::mpsc;

    #[test]
    fn thread_pool_build_requires_positive_size() {
        assert!(matches!(ThreadPool::build(0), Err(ThreadPoolBuildError)));
    }

    #[test]
    fn thread_pool_executes_all_jobs() {
        let pool = ThreadPool::build(2).unwrap();
        let (sender, receiver) = mpsc::channel();

        for value in 0..4 {
            let sender = sender.clone();
            pool.execute(move || {
                sender.send(value).unwrap();
            });
        }

        drop(sender);

        let mut received: Vec<_> = receiver.iter().take(4).collect();
        received.sort();

        assert_eq!(received, vec![0, 1, 2, 3]);
    }
}
