use crate::Result;
use super::ThreadPool;


pub struct SharedRayonThreadPool {
    pool : rayon::ThreadPool
}


impl ThreadPool for SharedRayonThreadPool {
    fn spawn<F>(&self, f: F)
        where F: FnOnce() + Send + 'static,
    {
        self.pool.install(f);    
    }

    fn new(num : u8) -> Result<Self> {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(num.into()).build().unwrap();
        Ok(SharedRayonThreadPool{ pool } )
    }
}