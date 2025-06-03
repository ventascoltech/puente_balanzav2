use crate::cache::Cache;
use crate::config::Config;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};
use log::{info, error, trace};
use regex::Regex;

pub fn start_tcp_server(addr: &str, cache: Arc<Cache>, config: Config) {
    let listener = TcpListener::bind(addr).expect("No se pudo iniciar el servidor TCP");
    info!("Servidor TCP escuchando en: {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let cache = cache.clone();
                let config = config.clone();
                thread::spawn(move || handle_client(stream, cache, config));
            }
            Err(e) => {
                error!("Error aceptando conexión TCP: {:?}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, cache: Arc<Cache>, config: Config) {
    let mut buffer = [0u8; 1024];
    match stream.read(&mut buffer) {
        Ok(n) if n > 0 => {
            let lossy_string = String::from_utf8_lossy(&buffer[..n]);
            let trimmed = lossy_string.trim();

            if trimmed.is_empty() {
                trace!("Comando vacío o con solo espacios recibido, ignorado.");
                return;
            }

            let re_1 = Regex::new(r"^1+\s*$").unwrap();
            let re_w = Regex::new(r"^W+\s*$").unwrap();

            if re_1.is_match(trimmed) {
                let duration = Duration::from_millis(config.cache_duration_ms);
                if let Some(data) = cache.get_if_valid(duration) {
                    let _ = stream.write_all(&data);
                } else {
                    let start_wait = Instant::now();
                    loop {
                        if let Some(data) = cache.get_if_valid(duration) {
                            let _ = stream.write_all(&data);
                            break;
                        }
                        if start_wait.elapsed() > duration {
                            let _ = stream.write_all(b"Err\n");
                            break;
                        }
                        thread::sleep(Duration::from_millis(50));
                    }
                }
            } else if re_w.is_match(trimmed) {
                let duration = Duration::from_millis(config.command_w_cache_duration_ms);
                if let Some(data) = cache.get_if_valid(duration) {
                    let _ = stream.write_all(&data);
                } else {
                    if let Some(sender) = config.serial_write_sender.as_ref() {
                        if sender.send(b"W\r\n".to_vec()).is_ok() {
                            let wait_duration = Duration::from_millis(config.max_wait_response_w_ms);
                            let start_wait = Instant::now();
                            loop {
                                if let Some(data) = cache.get_if_valid(duration) {
                                    let _ = stream.write_all(&data);
                                    break;
                                }
                                if start_wait.elapsed() > wait_duration {
                                    let _ = stream.write_all(b"Err\n");
                                    break;
                                }
                                thread::sleep(Duration::from_millis(50));
                            }
                        } else {
                            error!("No se pudo enviar 'W' al puerto serial.");
                            let _ = stream.write_all(b"Err\n");
                        }
                    } else {
                        error!("Sender del puerto serial no configurado.");
                        let _ = stream.write_all(b"Err\n");
                    }
                }
            } else {
                trace!("Comando desconocido recibido: {}", trimmed);
                let _ = stream.write_all(b"Err\n");
            }
        }
        Ok(_) => {
            trace!("Cliente desconectado sin enviar datos.");
        }
        Err(e) => {
            error!("Error leyendo del cliente TCP: {:?}", e);
        }
    }
}

pub fn is_relevant_data(data: &[u8]) -> bool {
    let irrelevant_patterns = [
        &[0x18, 0x0D][..],
        &[0x02, 0x3F, 0x58, 0x0D][..],
        &[0x02, 0x3F, 0x50, 0x0D][..],
        &[0x02, 0x3F, 0x44, 0x0D][..],
        &[0x02, 0x3F, 0x41, 0x0D][..],
        b"00000",
    ];
    let ends_with_pattern = &[0x30, 0x2E, 0x30, 0x30, 0x35, 0x0D][..];
    let contains_pattern = &b"Count        Weight/kg"[..];

    if irrelevant_patterns.iter().any(|&pat| data == pat) {
        return false;
    }
    if data.ends_with(ends_with_pattern) {
        return false;
    }
    if data.windows(contains_pattern.len()).any(|w| w == contains_pattern) {
        return false;
    }
    true
}

