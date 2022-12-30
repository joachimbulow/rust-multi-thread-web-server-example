use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    /// Create a new ThreadPool
    /// Minimum 1 size is needed or panics
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers: workers,
            sender,
        }
    }

    // Use a channel to send a closure to be executed in a worker
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
            
            if let Some(thread) = worker.thread.take() {    
                println!("Shutting down {}", worker.id);
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        // Move moves reciever ownership into this closure
        let thread = thread::spawn(move || loop {
            // Acquire mutex and wait for a job in the channel
            let message = receiver.lock().unwrap().recv().unwrap();
            println!("Now gonna match");
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; excuting.", id);
                    job();
                }

                Message::Terminate => {
                    break;
                }
            }
        });
        Worker {
            id: id,
            thread: Some(thread),
        }
    }
}
