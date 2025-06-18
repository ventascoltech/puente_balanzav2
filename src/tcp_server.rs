// === src/tcp_server.rs ===
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use flume::Sender;

use anyhow::{Context, Result};
use log::{info, warn};

use crate::cache::SharedCache;
use crate::config::RuntimeConfig;
use crate::command::Comando;

/// Inicia el servidor TCP y acepta conexiones entrantes.
pub fn start_tcp_server(runtime_config: &RuntimeConfig, cache: SharedCache) {
    if let Err(e) = run_server(runtime_config, cache) {
        warn!("❌ Error en el servidor TCP: {:?}", e);
    }
}

/// Ejecuta el bucle principal del servidor TCP.
fn run_server(runtime_config: &RuntimeConfig, cache: SharedCache) -> Result<()> {
    let config_guard = runtime_config.config.read();
    let listener = TcpListener::bind(config_guard.address())
        .context("No se pudo iniciar el servidor TCP")?;

    info!("🟢 Servidor TCP escuchando en {}", config_guard.address());
    drop(config_guard); // liberar el lock

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer = stream
                    .peer_addr()
                    .map(|a| a.to_string())
                    .unwrap_or_else(|_| "desconocido".to_string());
                info!("🔌 Nueva conexión desde {}", peer);

                let cache = cache.clone();
                let config = runtime_config.config.clone();
                let sender = runtime_config.serial_write_sender.clone();

                thread::spawn(move || {
                    if let Err(e) = handle_client(stream, config, sender, cache) {
                        warn!("❌ Error manejando cliente: {:?}", e);
                    }
                });
            }
            Err(e) => warn!("⚠️ Error al aceptar conexión: {}", e),
        }
    }

    Ok(())
}

/// Maneja una conexión con un cliente.
fn handle_client(
    mut stream: TcpStream,
    config: std::sync::Arc<parking_lot::RwLock<crate::config::Config>>,
    sender: Sender<Vec<u8>>,
    cache: SharedCache,
) -> Result<()> {
    let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
    let mut buffer = [0u8; 1024];

    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => {
                info!("🔌 Cliente desconectado [{}]", peer);
                break;
            }
            Ok(n) => n,
            Err(e) => {
                warn!("⚠️ Error al leer del cliente [{}]: {}", peer, e);
                break;
            }
        };

        let comando_str = String::from_utf8_lossy(&buffer[..bytes_read]).trim().to_string();
        info!("📥 Comando recibido del cliente [{}]: '{}'", peer, comando_str);

        match Comando::parse(&comando_str) {
            Some(Comando::Uno) => {
                let config_guard = config.read();
                responder_con_cache(&mut stream, &cache, config_guard.cache_duration_ms, b"NO DATA\n")?;
            }
            Some(Comando::W) => {
                manejar_comando_w(&mut stream, &config, &sender, &cache, &peer)?;
            }
            None => {
                warn!("⚠️ Comando no reconocido del cliente [{}]: '{}'", peer, comando_str);
                let _ = stream.write_all(b"Comando invalido\n");
            }
        }
    }

    Ok(())
}

/// Envía al cliente el dato de la caché si es válido, o un mensaje alternativo si no lo es.
fn responder_con_cache(
    stream: &mut TcpStream,
    cache: &SharedCache,
    timeout_ms: u64,
    no_data_msg: &[u8],
) -> Result<()> {
    let duration = Duration::from_millis(timeout_ms);
    match cache.lock().get_if_valid(duration) {
        Some(data) => {
            stream.write_all(&data).context("Error al enviar datos de caché al cliente")?;
            let texto = String::from_utf8_lossy(&data);
            info!("✅ Dato de caché enviado al cliente: {}", texto.trim_end());
        }
        None => {
            warn!("⚠️ No se encontró dato en caché válido ({} ms)", timeout_ms);
            let _ = stream.write_all(no_data_msg);
        }
    }
    Ok(())
}

/// Maneja el comando 'W': solicita dato nuevo a la báscula y espera una respuesta válida en caché.
fn manejar_comando_w(
    stream: &mut TcpStream,
    config: &std::sync::Arc<parking_lot::RwLock<crate::config::Config>>,
    sender: &Sender<Vec<u8>>,
    cache: &SharedCache,
    peer: &str,
) -> Result<()> {
    info!("📤 Solicitando peso actual a la báscula (comando 'W')...");

    sender.send(b"W".to_vec()).context("Error enviando 'W' al serial")?;

    let (w_duration_ms, w_response_timeout_ms) = {
        let c = config.read();
        (c.w_duration_ms, c.w_response_timeout_ms)
    };

    let start = Instant::now();
    let mut puntos = 0;

    loop {
        if let Some(data) = cache.lock().get_if_valid(Duration::from_millis(w_duration_ms)) {
            stream.write_all(&data).context("Error al enviar respuesta 'W' al cliente")?;
            let texto = String::from_utf8_lossy(&data);
            info!("✅ Respuesta enviada al cliente [{}]: {}", peer, texto.trim_end());
            break;
        }

        if start.elapsed() > Duration::from_millis(w_response_timeout_ms) {
            warn!("⏱️ Timeout esperando respuesta de la báscula ({} ms)", w_response_timeout_ms);
            let _ = stream.write_all(b"W TIMEOUT\n");
            break;
        }

        if puntos < 30 {
            print!(".");
            let _ = std::io::stdout().flush();
            puntos += 1;
        }

        thread::sleep(Duration::from_millis(10));
    }

    if puntos > 0 {
        println!();
    }

    Ok(())
}

