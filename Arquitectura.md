+--------------------+       TCP          +----------------+
| Cliente TCP (telnet)| <--------------> |  tcp_server.rs |
+--------------------+                   +--------+-------+
                                                   |
                                                   | (usa Cache)
                                         +---------v---------+
                                         |     cache.rs      |
                                         +---------+---------+
                                                   |
                                        (lee y escribe en tiempo real)
                                         +---------v---------+
                                         | serial_listener.rs|
                                         +---------+---------+
                                                   |
                                    +--------------v---------------+
                                    |   Puerto serial: báscula     |
                                    +-----------------------------+






Este programa en Rust es una aplicación puente llamada puente_balanzav1, que actúa como intermediario entre:

Una báscula conectada por puerto serial (por ejemplo /dev/ttyS0)

Uno o varios clientes TCP que solicitan datos de esa báscula

✅ Funcionalidades principales:
    Lee datos continuamente desde un puerto serial

    Filtra los datos relevantes (usando is_relevant_data)

    Almacena los datos relevantes temporalmente en una cache en memoria

    Responde a comandos TCP de clientes, usando la cache cuando es posible

    Envía comandos a la báscula por serial si es necesario

📦 Estructura general de los archivos
main.rs
    Punto de entrada de la aplicación.

    Carga configuración desde línea de comandos (puerto serial, TCP, baud rate, etc.).

    Inicia 2 procesos en paralelo:

    serial_listener: escucha el puerto serial.

    tcp_server: escucha conexiones TCP.

config.rs
    Define y construye una estructura Config con todos los parámetros configurables por línea de comandos.

    Usa la biblioteca clap para parsear argumentos.

    Abre el puerto serial con los parámetros apropiados.

    Provee la dirección TCP a usar (0.0.0.0:<puerto>).

serial_utils.rs
    Implementa funciones utilitarias para:

    Abrir y configurar puertos seriales (configurar_puerto_serial)

    Parsear strings a DataBits, Parity, y StopBits

serial_listener.rs
    Corre en un hilo separado.

    Lee continuamente datos del puerto serial.

    Si el dato es relevante (is_relevant_data(data)), lo guarda en la cache.

    También escucha por un canal mpsc::Receiver para enviar datos hacia el serial (por ejemplo, si el cliente TCP solicita un W).

tcp_server.rs
    Escucha conexiones TCP.

    Cada cliente se maneja en un hilo aparte.

    Comandos válidos:

    1: solicita el último dato válido desde la cache (con timeout).

    W: solicita un nuevo dato:

    Si ya hay uno en la cache reciente, lo usa.

    Si no, manda W por serial, espera la respuesta y la devuelve (con timeout).

cache.rs
    Implementa un pequeño sistema de cache con duración configurable:

    set(data): guarda el dato con Instant::now()

    get_if_valid(duration): devuelve el dato si no ha expirado

⚙️ Comportamiento típico
    Se lanza el programa: ./puente_balanzav1 --serial-port /dev/ttyUSB0 --tcp-port 2029

    El listener de serial se activa:

    Si recibe un dato relevante (ej. un peso válido), lo guarda en cache

    Un cliente TCP se conecta y envía:

    "1": el programa revisa la cache y responde con el dato si es reciente

    "W": si no hay dato reciente, el programa pide uno nuevo por serial y espera la respuesta
