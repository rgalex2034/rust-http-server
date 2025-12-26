use std::{
    error::Error,
    fmt::Display,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let mut workers: Vec<Worker> = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            let receiver = receiver.clone();

            workers.push(Worker::new(id, receiver));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).map_err(|error| {
            ThreadPoolError(format!(
                "ThreadPool: Could not send job to workers.\nInner: {error}"
            ))
        })
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in self.workers.drain(..) {
            println!("ThreadPool: Shutting Worker[{}]", worker.id);
            match worker.thread.join() {
                Ok(_) => continue,
                Err(_) => eprintln!("ThreadPool: Error while cleaning up Worker[{}]", worker.id),
            }
        }
    }
}

#[derive(Debug)]
pub struct ThreadPoolError(String);

impl Display for ThreadPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(error) = self;
        f.write_str(&format!("ThreadPool error: {error}"))
    }
}

impl Error for ThreadPoolError {}

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                println!("Worker[{id}]: Locking on receiver...");
                let receiver = match receiver.lock() {
                    Ok(inner) => inner,
                    Err(error) => {
                        println!("Worker[{id}]: Thread shutting down: {error}");
                        break;
                    }
                };

                println!("Worker[{id}]: Awaiting message...");
                let job = receiver
                    .recv()
                    .expect(&format!("Worker[{id}] Poisoned mutex! Panicking."));

                // Drop the receiver before calling the job.
                // This way, we release the lock, and other workers can start
                // processing other requests.
                drop(receiver);

                println!("Worker[{id}]: Awake!");
                job();
            }
        });

        Worker { id, thread }
    }
}
