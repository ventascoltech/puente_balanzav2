// === src/serial_reader.rs ===
use std::io::{Write};
use std::thread;
use std::time::Duration;

use anyhow::Context;
use flume::{Receiver, Selector};
use log::{debug, info, warn};
use serialport::SerialPort;

use crate::cache::SharedCache;
use crate::serial_processor::ensamblar_y_filtrar_datos;
use crate::serial_utils::sanitize_log_data;

/// Inicia el hilo de lectura desde el puerto serial.
pub fn start_serial_reader(
    mut serial: Box<dyn SerialPort>,
    cache: SharedCache,
    rx_serial_write: Receiver<Vec<u8>>,
) {
    thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        let mut partial_data = Vec::new();

        info!("🟡 Hilo de lectura serial iniciado. Esperando datos de la báscula...");

        loop {
            // Esperar comandos del canal con timeout
            match Selector::new()
                .recv(&rx_serial_write, |msg| msg)
                .wait_timeout(Duration::from_millis(50))
            {
                Ok(Ok(comando)) => {
                    if let Err(e) = serial
                        .write_all(&comando)
                        .and_then(|_| serial.flush())
                        .with_context(|| {
                            format!(
                                "Error al enviar comando serial: {}",
                                sanitize_log_data(&comando)
                            )
                        })
                    {
                        warn!("⚠️ {}", e);
                    } else {
                        info!(
                            "📤 Comando enviado al puerto serial: {}",
                            sanitize_log_data(&comando)
                        );
                    }
                }
                Ok(Err(e)) => {
                    warn!("⚠️ Error al recibir comando: {:?}", e);
                }
                Err(e) => {
                    warn!("❌ Error en selector: {:?}", e);
                }
            }

            // Leer datos del puerto serial
            match serial.read(&mut buffer) {
                Ok(bytes_read) if bytes_read > 0 => {
                    let recibidos = &buffer[..bytes_read];
                    debug!("📥 Bytes leídos (crudo): {}", sanitize_log_data(recibidos));

                    match ensamblar_y_filtrar_datos(recibidos, &mut partial_data) {
                        Some(msg) => {
                            info!("✅ Dato completo de báscula recibido: {}", sanitize_log_data(&msg));
                            cache.lock().set(msg);
                        }
                        None => {
                            debug!("🧩 Fragmento acumulado: {}", sanitize_log_data(&partial_data));
                        }
                    }
                }
                Ok(_) => {
                    // No se leyó nada
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // Timeout esperado, continuar
                }
                Err(e) => {
                    warn!("❌ Error al leer del puerto serial: {:?}", e);
                }
            }
        }
    });
}

