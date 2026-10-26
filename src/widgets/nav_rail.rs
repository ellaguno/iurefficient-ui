//! Riel de módulos: columna de iconos (Correo, Calendario, Contactos…) con insignias, ayuda
//! emergente y el elemento actual marcado. Los marcados `bottom` van al pie (Ajustes).

use egui::{Align, Layout, Rect, Sense, Ui, pos2, vec2};

use crate::icons::{self, Icon};
use crate::widgets::badge::{BadgeKind, count_text, paint_badge};

#[derive(Clone, Debug)]
pub struct NavItem<'a> {
    pub id: &'a str,
    pub icon: Icon,
    /// Nombre (ayuda emergente y lector de pantalla).
    pub label: &'a str,
    /// Contador (no leídos); 0 no muestra nada.
    pub badge: u32,
    /// Atajo que se muestra en la ayuda («Ctrl+1»).
    pub shortcut: Option<&'a str>,
    /// Va al pie del riel.
    pub bottom: bool,
}

impl<'a> NavItem<'a> {
    pub fn new(id: &'a str, icon: Icon, label: &'a str) -> Self {
        NavItem { id, icon, label, badge: 0, shortcut: None, bottom: false }
    }
    pub fn badge(mut self, n: u32) -> Self {
        self.badge = n;
        self
    }
    pub fn shortcut(mut self, s: &'a str) -> Self {
        self.shortcut = Some(s);
        self
    }
    pub fn bottom(mut self) -> Self {
        self.bottom = true;
        self
    }
}

/// Ancho recomendado del riel para la densidad actual (para `Panel::exact_size`).
pub fn rail_width(ctx: &egui::Context) -> f32 {
    match crate::theme::density(ctx) {
        crate::Density::Comfortable => 60.0,
        crate::Density::Compact => 52.0,
        crate::Density::Dense => 46.0,
    }
}

/// Dibuja el riel en todo el alto de `ui`. Devuelve el id pulsado, si hubo.
pub fn nav_rail<'a>(ui: &mut Ui, items: &'a [NavItem<'a>], selected: &str) -> Option<&'a str> {
    let mut clicked = None;
    let side = rail_width(ui.ctx()) - 12.0;
    ui.spacing_mut().item_spacing.y = 4.0;
    let mut one = |ui: &mut Ui, it: &'a NavItem<'a>| {
        let (rect, r) = ui.allocate_exact_size(vec2(side, side), Sense::click());
        let c = crate::theme::colors(ui);
        let on = it.id == selected;
        if on {
            ui.painter().rect_filled(rect, crate::tokens::radius::CARD, c.selection);
            let bar = Rect::from_min_max(
                pos2(rect.left() - 5.0, rect.center().y - 8.0),
                pos2(rect.left() - 2.0, rect.center().y + 8.0),
            );
            ui.painter().rect_filled(bar, 2.0, c.accent);
        } else if r.hovered() {
            ui.painter().rect_filled(rect, crate::tokens::radius::CARD, c.hover);
        }
        if r.has_focus() {
            ui.painter().rect_stroke(
                rect,
                crate::tokens::radius::CARD,
                egui::Stroke::new(2.0, c.focus),
                egui::StrokeKind::Inside,
            );
        }
        let icon_side = (side * 0.5).round();
        let ir = Rect::from_center_size(rect.center(), vec2(icon_side, icon_side));
        let ink = if on { c.accent } else { c.text_muted };
        icons::paint(ui.painter(), ir, it.icon, ink, if on { c.accent } else { c.text_muted });
        if it.badge > 0 {
            let small = crate::theme::density(ui.ctx()).small_size() - 1.0;
            paint_badge(
                ui.painter(),
                pos2(ir.right() + 2.0, ir.top()),
                &count_text(it.badge),
                BadgeKind::Accent,
                &c,
                small,
            );
        }
        let label = if it.badge > 0 { format!("{} ({})", it.label, it.badge) } else { it.label.to_string() };
        crate::a11y::toggle(&r, &label, on);
        let tip = match it.shortcut {
            Some(s) => format!("{label}   {s}"),
            None => label,
        };
        if r.on_hover_text(tip).clicked() {
            clicked = Some(it.id);
        }
    };
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.add_space(6.0);
        for it in items.iter().filter(|i| !i.bottom) {
            one(ui, it);
        }
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.add_space(6.0);
            for it in items.iter().filter(|i| i.bottom).rev() {
                one(ui, it);
            }
        });
    });
    clicked
}

/// Renglón de una barra lateral (carpeta, calendario, filtro): icono, nombre y contador, a todo
/// el ancho, con el alto de fila de la densidad.
pub fn sidebar_item(ui: &mut Ui, icon: Option<Icon>, label: &str, count: u32, selected: bool) -> egui::Response {
    let c = crate::theme::colors(ui);
    let d = crate::theme::density(ui.ctx());
    let h = d.row_height().max(24.0);
    let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::click());
    if selected {
        ui.painter().rect_filled(rect, crate::tokens::radius::CONTROL, c.selection);
    } else if r.hovered() {
        ui.painter().rect_filled(rect, crate::tokens::radius::CONTROL, c.hover);
    }
    if r.has_focus() {
        ui.painter().rect_stroke(
            rect,
            crate::tokens::radius::CONTROL,
            egui::Stroke::new(2.0, c.focus),
            egui::StrokeKind::Inside,
        );
    }
    let side = d.icon_size();
    let mut x = rect.left() + 8.0;
    if let Some(icon) = icon {
        let ir = Rect::from_center_size(pos2(x + side / 2.0, rect.center().y), vec2(side, side));
        icons::paint(ui.painter(), ir, icon, if selected { c.accent } else { c.text_muted }, c.accent);
        x += side + 8.0;
    }
    let mut right = rect.right() - 8.0;
    if count > 0 {
        let small = d.small_size();
        let text = count_text(count);
        let font = crate::theme::semibold_font(ui.ctx(), small);
        let g = ui.painter().layout_no_wrap(text, font, c.accent);
        ui.painter().galley(pos2(right - g.size().x, rect.center().y - g.size().y / 2.0), g.clone(), c.accent);
        right -= g.size().x + 8.0;
    }
    let font = if count > 0 {
        crate::theme::semibold_font(ui.ctx(), d.body_size())
    } else {
        crate::theme::body_font(ui.ctx())
    };
    let g = crate::widgets::text::elided(ui, label, font, c.text, (right - x).max(0.0));
    ui.painter().galley(pos2(x, rect.center().y - g.size().y / 2.0), g, c.text);
    let a11y = if count > 0 { format!("{label} ({count})") } else { label.to_string() };
    crate::a11y::toggle(&r, &a11y, selected);
    r
}
