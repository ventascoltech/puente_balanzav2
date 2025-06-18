use crate::serial_utils::is_relevant_data;

/// Ensambla mensajes del puerto serial terminados en 0x0D (carácter '\r').
/// Devuelve `Some(Vec<u8>)` si el mensaje completo es relevante.
/// Si el mensaje no es relevante, se descarta.
pub fn ensamblar_y_filtrar_datos(buffer: &[u8], partial_data: &mut Vec<u8>) -> Option<Vec<u8>> {
    // Acumular nuevos datos
    partial_data.extend_from_slice(buffer);

    // Buscar fin de mensaje
    if let Some(pos) = partial_data.iter().position(|&b| b == 0x0D) {
        let completo = partial_data.drain(..=pos).collect::<Vec<u8>>();
        if is_relevant_data(&completo) {
            return Some(completo);
        }
    }

    None
}


