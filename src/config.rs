use std::time::Duration;
use serialport::{DataBits, Parity, StopBits};
use std::sync::mpsc::Sender;


#[derive(Clone)]
pub struct Config {
    pub serial_port: String,
    pub tcp_port: u16,
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub parity: Parity,
    pub stop_bits: StopBits,
    pub timeout: Duration,
    pub cache_duration_ms: u64,               // para comando '1'
    pub command_w_cache_duration_ms: u64,     // para comando 'W'
    pub max_wait_response_w_ms: u64,          // tiempo máximo para esperar respuesta W
    pub serial_write_sender: Option<Sender<Vec<u8>>>,
}

