use std::thread;

use crate::Result;
use super::ThreadPool;


pub struct NaiveThreadPool;
// { 
    // num : u8 
// }


impl ThreadPool for NaiveThreadPool {
    fn spawn<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
    {
        thread::spawn(f);
    }

    fn new(_num : u8) -> Result<Self> {
        Ok(NaiveThreadPool)
    }
}