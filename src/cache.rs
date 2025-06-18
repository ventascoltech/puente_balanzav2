// === src/cache.rs ===
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::serial_utils::sanitize_log_data;

pub type SharedCache = Arc<Mutex<Cache>>;

pub struct Cache {
    data: Option<(Vec<u8>, Instant)>,
}

impl Cache {
    /// Crea una nueva instancia vacía
    pub fn new() -> Self {
        Self { data: None }
    }

    /// Establece nuevos datos con su timestamp
    pub fn set(&mut self, data: Vec<u8>) {
        self.data = Some((data, Instant::now()));
    }

    /// Devuelve una referencia a los datos si no han expirado
    pub fn get_if_valid(&self, duration: Duration) -> Option<&[u8]> {
        self.data
            .as_ref()
            .and_then(|(d, t)| (t.elapsed() <= duration).then(|| d.as_slice()))
    }

    /// Verifica si los datos actuales siguen siendo válidos
    pub fn is_valid(&self, duration: Duration) -> bool {
        self.data
            .as_ref()
            .map_or(false, |(_, t)| t.elapsed() <= duration)
    }

    /// Retorna una representación de los últimos datos (para debugging/logs)
    pub fn debug_last_value(&self) -> String {
        match &self.data {
            Some((data, t)) => format!(
                "Último valor: {:?} (hace {:?})",
                sanitize_log_data(data),
                t.elapsed()
            ),
            None => "Cache vacía".to_string(),
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

