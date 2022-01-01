use std::thread;
use crossbeam_channel::{unbounded, Sender, Receiver}; 

use crate::Result;
use super::ThreadPool;
use log::{debug, error};



pub struct SharedQueueThreadPool
{ 
    tx : Sender<Box<dyn FnOnce() + Send + 'static>>
}

struct TaskReceiver (Receiver<Box<dyn FnOnce() + Send + 'static>>);

impl Drop for TaskReceiver {
    fn drop(&mut self) {
        // thread is dropped, try to create a new thread
        if thread::panicking() {
            // if thread is panicking , try to recreate the thread
            let rx = TaskReceiver(self.0.clone());
            if let Err(e) = thread::Builder::new().spawn(move || run_task(rx)) {
                debug!("failed to spawn new thread on panick {}", e);
            }
        }
    }
}


impl ThreadPool for SharedQueueThreadPool {
    fn spawn<F>(&self, f: F)
        where F: FnOnce() + Send + 'static,
    {
        self.tx
            .send(Box::new(f))
            .expect("Channel send error");
    }

    fn new(num : u8) -> Result<Self> {
        let (tx, rx) = unbounded::<Box<dyn FnOnce() + Send + 'static>>();
        for _i in 0..num {
            let rx = TaskReceiver(rx.clone());
            thread::spawn(move || run_task(rx));
        }

        Ok(SharedQueueThreadPool { tx })
    }
}


fn run_task(rx : TaskReceiver) {
    loop {
        match rx.0.recv() {
            Ok(task) => {
                task()
            },
            Err(_e) => {
                debug!("Thread exits");
            }
        }
    }
}