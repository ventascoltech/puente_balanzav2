use std::sync::mpsc::Receiver;
use crate::cache::Cache;
use crate::tcp_server::is_relevant_data;
use std::thread;
use std::io::Read;
use std::sync::Arc;
use serialport::SerialPort;

pub fn start_serial_listener(
    mut port: Box<dyn SerialPort>,
    cache: Arc<Cache>,
    rx: Receiver<Vec<u8>>,
) {
    thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            if let Ok(data) = rx.try_recv() {
                let _ = port.write_all(&data);
            }

            match port.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let data = &buffer[..n];
                    if is_relevant_data(data) {
                        cache.set(data.to_vec());
                    }
                }
                _ => {}
            }
        }
    });
}

