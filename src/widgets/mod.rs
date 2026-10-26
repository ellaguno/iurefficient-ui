//! Controles pequeños reutilizables.

pub mod avatar;
pub mod badge;
pub mod banner;
pub mod empty_state;
pub mod keycap;
pub mod nav_rail;
pub mod search_field;
pub mod segmented;
pub mod split;
pub mod text;

use egui::{Response, RichText, Ui};

/// Botón principal (relleno con el acento). Uno por vista, como mucho.
pub fn primary_button(ui: &mut Ui, label: &str) -> Response {
    let c = crate::theme::colors(ui);
    let font = crate::theme::medium_font(ui.ctx(), crate::theme::density(ui.ctx()).body_size());
    ui.add(
        egui::Button::new(RichText::new(label).font(font).color(c.on_accent).strong())
            .fill(c.accent)
            .stroke(egui::Stroke::NONE),
    )
}

/// Encabezado de sección pequeño en mayúsculas suaves («CARPETAS»).
pub fn section_label(ui: &mut Ui, text: &str) -> Response {
    let c = crate::theme::colors(ui);
    let font = crate::theme::semibold_font(ui.ctx(), crate::theme::density(ui.ctx()).small_size());
    ui.label(RichText::new(text.to_uppercase()).font(font).color(c.text_muted).strong())
}
