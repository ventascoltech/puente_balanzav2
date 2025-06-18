use std::{fs, io::Write, sync::Arc, thread, time::Duration};

use anyhow::{Context, Result};
use log::info;
use parking_lot::RwLock;
use serialport::{DataBits, Parity, SerialPort, StopBits};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub serial_port: String,
    pub baud_rate: u32,
    #[serde(deserialize_with = "crate::serial_utils::deserialize_data_bits")]
    pub data_bits: DataBits,
    #[serde(deserialize_with = "crate::serial_utils::deserialize_parity")]
    pub parity: Parity,
    #[serde(deserialize_with = "crate::serial_utils::deserialize_stop_bits")]
    pub stop_bits: StopBits,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_cache_duration_ms")]
    pub cache_duration_ms: u64,
    #[serde(default = "default_w_duration_ms")]
    pub w_duration_ms: u64,
    #[serde(default = "default_w_response_timeout_ms")]
    pub w_response_timeout_ms: u64,
    #[serde(default = "default_tcp_address")]
    pub tcp_address: String,
    #[serde(default = "default_recargar_configuracion")]
    pub recargar_configuracion: bool,
}

fn default_timeout_ms() -> u64 { 1000 }
fn default_cache_duration_ms() -> u64 { 1000 }
fn default_w_duration_ms() -> u64 { 500 }
fn default_w_response_timeout_ms() -> u64 { 500 }
fn default_tcp_address() -> String { "0.0.0.0:2029".to_string() }
fn default_recargar_configuracion() -> bool { true }

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Error leyendo archivo de configuración {}", path))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| "Error parseando archivo TOML con serde")?;
        Ok(config)
    }



    pub fn open_serial_port(&self) -> Result<Box<dyn SerialPort>> {
        serialport::new(&self.serial_port, self.baud_rate)
            .data_bits(self.data_bits)
            .parity(self.parity)
            .stop_bits(self.stop_bits)
            // Ya no se necesita timeout, poll controla el bloqueo
            .timeout(Duration::from_secs(0))
            .open()
            .with_context(|| format!("No se pudo abrir el puerto serial {}", self.serial_port))
    }


    pub fn log_config(&self) {
        info!("📦 Configuración cargada:");
        info!("  Serial port           : {}", self.serial_port);
        info!("  Baud rate             : {}", self.baud_rate);
        info!("  Data bits             : {:?}", self.data_bits);
        info!("  Parity                : {:?}", self.parity);
        info!("  Stop bits             : {:?}", self.stop_bits);
        info!("  Timeout (ms)          : {}", self.timeout_ms);
        info!("  Cache duration (ms)   : {}", self.cache_duration_ms);
        info!("  W duración (ms)       : {}", self.w_duration_ms);
        info!("  W respuesta timeout   : {}", self.w_response_timeout_ms);
        info!("  Dirección TCP         : {}", self.tcp_address);
        info!("  Recarga configuración : {}", self.recargar_configuracion);
    }

    pub fn address(&self) -> &str {
        &self.tcp_address
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfigComparable {
    serial_port: String,
    baud_rate: u32,
    data_bits: DataBits,
    parity: Parity,
    stop_bits: StopBits,
    timeout_ms: u64,
    cache_duration_ms: u64,
    w_duration_ms: u64,
    w_response_timeout_ms: u64,
    tcp_address: String,
    recargar_configuracion: bool,
}

impl From<&Config> for ConfigComparable {
    fn from(cfg: &Config) -> Self {
        ConfigComparable {
            serial_port: cfg.serial_port.clone(),
            baud_rate: cfg.baud_rate,
            data_bits: cfg.data_bits,
            parity: cfg.parity,
            stop_bits: cfg.stop_bits,
            timeout_ms: cfg.timeout_ms,
            cache_duration_ms: cfg.cache_duration_ms,
            w_duration_ms: cfg.w_duration_ms,
            w_response_timeout_ms: cfg.w_response_timeout_ms,
            tcp_address: cfg.tcp_address.clone(),
            recargar_configuracion: cfg.recargar_configuracion,
        }
    }
}

pub struct RuntimeConfig {
    pub config: Arc<RwLock<Config>>,
    pub serial_write_sender: flume::Sender<Vec<u8>>,
}

pub fn init_logging() {
    use env_logger::Builder;
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, log::LevelFilter::Info)
        .init();
}

pub fn spawn_reload_thread(path: &str, shared: Arc<RwLock<Config>>) {
    let path = path.to_string();

    thread::spawn(move || {
        let guard = shared.read();
        let mut ultima_config = ConfigComparable::from(&*guard);
        drop(guard);

        if !ultima_config.recargar_configuracion {
            log::info!("📴 Recarga de configuración desactivada por archivo de configuración");
            return;
        }

        loop {
            thread::sleep(Duration::from_secs(5));
            match Config::load_from_file(&path) {
                Ok(nueva_config) => {
                    let nueva_comp = ConfigComparable::from(&nueva_config);

                    if !nueva_config.recargar_configuracion {
                        log::info!("📴 Recarga de configuración desactivada dinámicamente");
                        break;
                    }

                    if nueva_comp != ultima_config {
                        *shared.write() = nueva_config.clone();
                        ultima_config = nueva_comp;
                        log::info!("🔄 Configuración recargada desde {}", path);
                        nueva_config.log_config();
                    }
                }
                Err(e) => {
                    log::warn!("⚠️ Error recargando configuración: {}", e);
                }
            }
        }
    });
}

