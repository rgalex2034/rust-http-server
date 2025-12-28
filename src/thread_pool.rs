use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, SendError},
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

    pub fn execute<F>(&self, f: F) -> Result<(), SendError<Box<dyn FnOnce() + Send + 'static>>>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job)
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
                    Err(poisoned_error) => {
                        eprintln!(
                            "Worker[{id}]: Poisoned mutex! Some previous request panicked. Ignoring error."
                        );
                        poisoned_error.into_inner()
                    }
                };

                println!("Worker[{id}]: Awaiting message...");
                let job = receiver.recv().expect(&format!(
                    "Worker[{id}] Thread shutting down. Sender closed the channel."
                ));

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
