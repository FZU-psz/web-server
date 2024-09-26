use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let mut workers = Vec::with_capacity(size);

        let (sender, recevier) = mpsc::channel();
        let recevier = Arc::new(Mutex::new(recevier));
        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&recevier)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutdow worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            };
        }
    }
}
struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn new(id: usize, recevier: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = recevier.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {id}, got job ");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected");
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
