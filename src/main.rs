// === src/main.rs ===
mod cache;
mod config;
mod serial_reader;
mod serial_processor;
mod serial_utils;
mod tcp_server;
mod command;

use crate::config::{Config, RuntimeConfig};
use flume::unbounded;
use std::sync::Arc;

use anyhow::Result;

fn main() -> Result<()> {
    config::init_logging();

    // Leer el argumento de línea de comandos (opcional)
    let args: Vec<String> = std::env::args().collect();
    let config_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "config.toml".to_string()
    };

    log::info!("📄 Cargando configuración desde {}", config_path);

    let (tx_serial_write, rx_serial_write) = unbounded();
    let initial_config =
        Config::load_from_file(&config_path).expect("No se pudo cargar el archivo de configuración");

    let shared_config = Arc::new(parking_lot::RwLock::new(initial_config));
    let cache = cache::SharedCache::default();

    let runtime_config = RuntimeConfig {
        config: shared_config.clone(),
        serial_write_sender: tx_serial_write.clone(),
    };

    // ⏱️ Lanzar hilo para recargar configuración periódicamente si aplica
    config::spawn_reload_thread(&config_path, shared_config.clone());

    // ⚙️ Inicializar puerto serial
    let serial_port = shared_config.read().open_serial_port()?;
    shared_config.read().log_config();

    log::info!("✅ Inicializando escucha en puerto serial...");
    serial_reader::start_serial_reader(serial_port, cache.clone(), rx_serial_write);

    log::info!("📡 Iniciando servidor TCP...");
    tcp_server::start_tcp_server(&runtime_config, cache);

    Ok(())
}

