
mod naive;
mod shared_queue;

pub use self::naive::NaiveThreadPool;
pub use self::shared_queue::SharedQueueThreadPool;
use crate::Result;


pub trait ThreadPool {
    // ALWAYS put reference for struct method, to prevent moving the object
    fn spawn<F>(&self, f: F)
        where F: FnOnce() + Send + 'static,;
    
    fn new(num : u8) -> Result<Self> where Self: Sized;
}


