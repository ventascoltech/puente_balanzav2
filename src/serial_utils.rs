use serialport::{SerialPort, DataBits, Parity, StopBits, SerialPortBuilder};
use std::time::Duration;
use std::io::{self};
use log::{info, error};

pub fn configurar_puerto_serial(
    serial_port: &str,
    baud_rate: u32,
    data_bits: DataBits,
    parity: Parity,
    stop_bits: StopBits,
    timeout: Duration,
) -> io::Result<Box<dyn SerialPort>> {
    let builder: SerialPortBuilder = serialport::new(serial_port, baud_rate)
        .data_bits(data_bits)
        .parity(parity)
        .stop_bits(stop_bits)
        .timeout(timeout);

    match builder.open().map_err(|e| io::Error::new(io::ErrorKind::Other, e)) {
        Ok(port) => {
            info!("Puerto serial configurado con éxito: {:?}, {:?}, {:?}", data_bits, parity, stop_bits);
            Ok(port)
        }
        Err(e) => {
            error!("Error al configurar el puerto serial: {:?}", e);
            Err(e)
        }
    }
}

pub fn parse_data_bits(bits: &str) -> DataBits {
    match bits {
        "5" => DataBits::Five,
        "6" => DataBits::Six,
        "7" => DataBits::Seven,
        _ => DataBits::Eight,
    }
}

pub fn parse_parity(parity: &str) -> Parity {
    match parity.to_lowercase().as_str() {
        "odd" => Parity::Odd,
        "even" => Parity::Even,
        _ => Parity::None,
    }
}

pub fn parse_stop_bits(bits: &str) -> StopBits {
    match bits {
        "2" => StopBits::Two,
        _ => StopBits::One,
    }
}