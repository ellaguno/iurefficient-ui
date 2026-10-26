//! Estado vacío: icono grande, título, explicación y una acción («No hay mensajes — Actualizar»).

use egui::{RichText, Sense, Ui, vec2};

use crate::icons::{self, Icon};

#[derive(Clone, Debug)]
pub struct EmptyState<'a> {
    icon: Icon,
    title: &'a str,
    body: Option<&'a str>,
    action: Option<&'a str>,
}

impl<'a> EmptyState<'a> {
    pub fn new(icon: Icon, title: &'a str) -> Self {
        EmptyState { icon, title, body: None, action: None }
    }
    pub fn body(mut self, body: &'a str) -> Self {
        self.body = Some(body);
        self
    }
    pub fn action(mut self, label: &'a str) -> Self {
        self.action = Some(label);
        self
    }

    /// Dibuja centrado en el espacio disponible. Devuelve `true` si se pulsó la acción.
    pub fn show(self, ui: &mut Ui) -> bool {
        let c = crate::theme::colors(ui);
        let mut clicked = false;
        let h =
            64.0 + 40.0 + if self.body.is_some() { 40.0 } else { 0.0 } + if self.action.is_some() { 44.0 } else { 0.0 };
        let top = ((ui.available_height() - h) / 2.0).max(8.0);
        ui.vertical_centered(|ui| {
            ui.set_max_width(ui.available_width().min(360.0));
            ui.add_space(top);
            let (r, resp) = ui.allocate_exact_size(vec2(64.0, 64.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 32.0, c.surface_alt);
            ui.painter().circle_stroke(r.center(), 32.0, egui::Stroke::new(1.0, c.separator));
            icons::paint(ui.painter(), r.shrink(18.0), self.icon, c.text_muted, c.accent);
            crate::a11y::label(&resp, self.title);
            ui.add_space(12.0);
            ui.label(RichText::new(self.title).font(crate::theme::semibold_font(ui.ctx(), 16.0)).color(c.text));
            if let Some(b) = self.body {
                ui.add_space(4.0);
                ui.label(RichText::new(b).color(c.text_muted));
            }
            if let Some(a) = self.action {
                ui.add_space(12.0);
                clicked = super::primary_button(ui, a).clicked();
            }
        });
        clicked
    }
}
