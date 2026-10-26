//! Accesibilidad: nombre, tipo y estado para el lector de pantalla (AccessKit, que eframe trae
//! activado) de los controles que se dibujan a mano. Los widgets de egui ya lo dan solos.

use egui::{Response, Role, WidgetInfo};

/// Botón con su texto.
pub fn button(resp: &Response, label: &str) {
    let (enabled, label) = (resp.enabled(), label.to_string());
    resp.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, label.clone()));
}

/// Botón que se queda activo (vista actual, interruptor): se anuncia como seleccionado o no.
pub fn toggle(resp: &Response, label: &str, on: bool) {
    let (enabled, label) = (resp.enabled(), label.to_string());
    resp.widget_info(|| WidgetInfo::selected(Role::Button, enabled, on, label.clone()));
}

/// Fila de una tabla o elemento de una lista, con lo que se lee de ella.
pub fn row(resp: &Response, label: &str, selected: bool) {
    let (enabled, label) = (resp.enabled(), label.to_string());
    resp.widget_info(|| WidgetInfo::selected(Role::ListItem, enabled, selected, label.clone()));
}

/// Una zona dibujada (lista, calendario, diagrama) con su descripción.
pub fn area(resp: &Response, label: &str) {
    let (enabled, label) = (resp.enabled(), label.to_string());
    resp.widget_info(|| WidgetInfo::labeled(Role::Group, enabled, label.clone()));
}

/// Texto informativo (aviso, estado vacío).
pub fn label(resp: &Response, text: &str) {
    let (enabled, text) = (resp.enabled(), text.to_string());
    resp.widget_info(|| WidgetInfo::labeled(Role::Label, enabled, text.clone()));
}
