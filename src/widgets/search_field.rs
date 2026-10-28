//! Campo de búsqueda: lupa, texto, botón para borrar y el atajo que lo enfoca («Ctrl+F»).
//! Esc lo vacía.

use egui::{Key, KeyboardShortcut, Response, Sense, Stroke, Ui, vec2};

use crate::icons::{self, Icon};
use crate::lang::pick;

pub struct SearchField<'a> {
    text: &'a mut String,
    id_salt: egui::Id,
    hint: Option<String>,
    shortcut: Option<KeyboardShortcut>,
    width: Option<f32>,
}

impl<'a> SearchField<'a> {
    pub fn new(text: &'a mut String) -> Self {
        SearchField {
            text,
            id_salt: egui::Id::unique("iurefficient-ui/search"),
            hint: None,
            shortcut: None,
            width: None,
        }
    }
    /// Necesario si hay más de un campo de búsqueda en la misma vista.
    pub fn id_salt(mut self, salt: impl egui::AsIdSalt) -> Self {
        self.id_salt = egui::Id::unique(salt);
        self
    }
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
    /// Atajo que enfoca el campo; se muestra como tecla mientras está vacío y sin foco.
    pub fn shortcut(mut self, shortcut: KeyboardShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }
    pub fn desired_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Dibuja el campo; la respuesta es la del texto (`changed()` también al borrar).
    pub fn show(self, ui: &mut Ui) -> Response {
        let c = crate::theme::colors(ui);
        let d = crate::theme::density(ui.ctx());
        let id = ui.make_persistent_id(self.id_salt);
        if let Some(sc) = self.shortcut
            && ui.input_mut(|i| i.consume_shortcut(&sc))
        {
            ui.memory_mut(|m| m.request_focus(id));
        }
        let focused = ui.memory(|m| m.has_focus(id));
        let mut cleared = false;
        if focused && !self.text.is_empty() && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            self.text.clear();
            cleared = true;
        }
        let width = self.width.unwrap_or_else(|| ui.available_width()).min(ui.available_width());
        let h = d.control_height();
        let fill = ui.visuals().text_edit_bg_color();
        let stroke = if focused { Stroke::new(1.5, c.focus) } else { Stroke::new(1.0, c.separator) };
        let hint = self.hint.unwrap_or_else(|| pick("Search", "Buscar").to_string());
        let shortcut_text = self.shortcut.map(|s| ui.ctx().format_shortcut(&s));
        let mut out = egui::Frame::new()
            .fill(fill)
            .stroke(stroke)
            .corner_radius(crate::tokens::radius::CONTROL as u8)
            .inner_margin(egui::Margin { left: 8, right: 4, top: 0, bottom: 0 })
            .show(ui, |ui| {
                ui.set_width(width - 12.0);
                ui.set_height(h - 2.0);
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let side = d.icon_size().min(16.0);
                    let (r, _) = ui.allocate_exact_size(vec2(side, side), Sense::hover());
                    icons::paint(ui.painter(), r, Icon::Search, c.text_muted, c.text_muted);
                    let trailing = if !self.text.is_empty() {
                        side + 12.0
                    } else if let Some(t) = &shortcut_text
                        && !focused
                    {
                        12.0 + 9.0 * t.chars().count() as f32
                    } else {
                        0.0
                    };
                    let te = ui.add(
                        egui::TextEdit::singleline(self.text)
                            .id(id)
                            .hint_text(hint.as_str())
                            .frame(egui::Frame::NONE)
                            .margin(vec2(2.0, 0.0))
                            // Todo el alto del campo con el texto (y la pista) centrado en vertical:
                            // si no, egui lo deja arriba de un rectángulo más alto que la línea y
                            // se ve subido y cortado respecto al marco.
                            .align(egui::Align2::LEFT_CENTER)
                            .min_size(vec2(0.0, (h - 2.0).max(0.0)))
                            .desired_width((ui.available_width() - trailing).max(20.0)),
                    );
                    if !self.text.is_empty() {
                        let clear = icons::icon_button(ui, Icon::Close, pick("Clear search", "Borrar búsqueda"), true);
                        if clear.clicked() {
                            self.text.clear();
                            cleared = true;
                            te.request_focus();
                        }
                    } else if let Some(t) = &shortcut_text
                        && !focused
                    {
                        super::keycap::keycap(ui, t);
                    }
                    te
                })
                .inner
            })
            .inner;
        if cleared {
            out.mark_changed();
        }
        out
    }
}
