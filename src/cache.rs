use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Cache {
    inner: Arc<Mutex<Option<(Vec<u8>, Instant)>>>, 
}

impl Cache {
    pub fn new() -> Self {
    //pub fn new(duration: Duration) -> Self {   // esta la opcion de recibirlo aqui para hacerlo
    //persitente el parametro Duration
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set(&self, data: Vec<u8>) {
        let mut lock = self.inner.lock().unwrap();
        *lock = Some((data, Instant::now()));
    }

    pub fn get_if_valid(&self, duration: Duration) -> Option<Vec<u8>> {
        let lock = self.inner.lock().unwrap();
        if let Some((data, timestamp)) = &*lock {
            if timestamp.elapsed() <= duration {
                return Some(data.clone());
            }
        }
        None
    }
}
