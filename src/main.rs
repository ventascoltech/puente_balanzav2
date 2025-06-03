mod config;
mod serial_utils;
mod serial_listener;
mod tcp_server;
mod cache;

use clap::{Arg, Command};
use std::sync::Arc;
use std::time::Duration;
use serial_utils::{configurar_puerto_serial, parse_data_bits, parse_parity, parse_stop_bits};
use cache::Cache;
use config::Config;
use std::sync::mpsc;


fn main() {
    env_logger::init();

    let matches = Command::new("puente_balanzav1")
        .arg(Arg::new("serial_port").long("serial-port").default_value("/dev/ttyS0"))
        .arg(Arg::new("tcp_port").long("tcp-port").default_value("2029"))
        .arg(Arg::new("baud_rate").long("baud-rate").default_value("9600"))
        .arg(Arg::new("data_bits").long("data-bits").default_value("8"))
        .arg(Arg::new("parity").long("parity").default_value("None"))
        .arg(Arg::new("stop_bits").long("stop-bits").default_value("1"))
        .arg(Arg::new("cache_duration_ms").long("cache-duration-ms").default_value("1000"))
        .arg(Arg::new("command_w_cache_duration_ms").long("w-duration-ms").default_value("500"))
        .arg(Arg::new("max_wait_response_w_ms").long("w-response-timeout-ms").default_value("750"))

        .get_matches();

    let serial_port = matches.get_one::<String>("serial_port").unwrap();
    let tcp_port = matches.get_one::<String>("tcp_port").unwrap();
    let baud_rate = matches.get_one::<String>("baud_rate").unwrap().parse().unwrap();
    let data_bits = parse_data_bits(matches.get_one::<String>("data_bits").unwrap());
    let parity = parse_parity(matches.get_one::<String>("parity").unwrap());
    let stop_bits = parse_stop_bits(matches.get_one::<String>("stop_bits").unwrap());
    let timeout = Duration::from_millis(100);
    let cache_duration = Duration::from_millis(matches.get_one::<String>("cache_duration_ms").unwrap().parse().unwrap());

    let (tx_serial_write, rx_serial_write) = mpsc::channel::<Vec<u8>>();
    let config = Config {
        serial_port: serial_port.clone(),
        tcp_port: tcp_port.parse().unwrap(),
        baud_rate,
        data_bits,
        parity,
        stop_bits,
        timeout,
        cache_duration_ms: matches.get_one::<String>("cache_duration_ms").unwrap().parse().unwrap(),
        command_w_cache_duration_ms: matches.get_one::<String>("command_w_cache_duration_ms").unwrap().parse().unwrap(),
        max_wait_response_w_ms: matches.get_one::<String>("max_wait_response_w_ms").unwrap().parse().unwrap(),
        serial_write_sender: Some(tx_serial_write.clone()),
    };

    let addr = format!("0.0.0.0:{}", config.tcp_port);

    let cache = Cache::new();
    //let cache = Cache::new(cache_duration); //para hacerlo persistenta
    let serial = configurar_puerto_serial(serial_port, baud_rate, data_bits, parity, stop_bits, timeout)
        .expect("No se pudo configurar el puerto serial");

    let arc_cache = Arc::new(cache);
    serial_listener::start_serial_listener(serial, arc_cache.clone(), rx_serial_write);
    tcp_server::start_tcp_server(&addr, arc_cache.clone(), config);
//    serial_listener::start_serial_listener(serial, cache.clone(), rx_serial_write);
//    serial_listener::start_serial_listener(serial, cache.clone());
//    tcp_server::start_tcp_server(format!("0.0.0.0:{}", tcp_port), cache);
//    tcp_server::start_tcp_server(&addr, cache, config);
}
