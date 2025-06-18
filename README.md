# puente_balanzav2

Aplicación Rust para escuchar datos de una balanza a través de un puerto serial y exponerlos por TCP.

---

## 📦 Requisitos

- Rust >= 1.70
- Linux/macOS/Windows
- Balanza conectada por puerto serial

---

## ⚙️ Configuración

El sistema carga la configuración desde línea de comandos o archivo `~/.config/puente_balanzav2/config.toml`.

Puedes generar uno automáticamente al primer uso o crear manualmente:

```toml
serial_port = "/dev/ttyS0"
tcp_port = 2029
baud_rate = 9600
data_bits = "8"
parity = "None"
stop_bits = "1"
timeout_ms = 100
cache_duration_ms = 1000
w_duration_ms = 500
w_response_timeout_ms = 750



# puente_balanzav1

`puente_balanzav1` es una aplicación en Rust que actúa como puente entre una báscula conectada por puerto serial y clientes remotos conectados por TCP.

## Funcionalidad principal

- Lee datos automáticamente desde el puerto serial.
- Filtra y almacena temporalmente datos relevantes en memoria (`cache`).
- Acepta conexiones TCP y responde comandos (`1` y `W`) con datos de la báscula.
- Usa configuraciones flexibles desde línea de comandos.
- Registra eventos y errores mediante `log` y `env_logger`.

## Comandos TCP soportados

- `1`: Solicita el último dato válido disponible en `cache`. Si no hay, espera brevemente.
- `W`: Envía `"W\r\n"` al puerto serial y espera una respuesta antes de reenviarla al cliente.

## Configuración por argumentos

```bash
--serial-port /dev/ttyS0
--tcp-port 2029
--baud-rate 9600
--data-bits 8
--parity None
--stop-bits 1
--cache-duration-ms 1000
--w-duration-ms 500
--w-response-timeout-ms 750

