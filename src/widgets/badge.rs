//! Insignias: contadores (no leídos) y etiquetas cortas en forma de píldora.

use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Ui, vec2};

use crate::tokens::Palette;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BadgeKind {
    /// Acento (no leídos).
    #[default]
    Accent,
    /// Gris (totales, borradores).
    Neutral,
    Danger,
    Success,
    Warning,
}

/// Texto de un contador: «0»…«999», y «999+» de ahí en adelante.
pub fn count_text(count: u32) -> String {
    if count > 999 { "999+".to_string() } else { count.to_string() }
}

/// Relleno y texto de una insignia.
pub fn badge_colors(kind: BadgeKind, c: &Palette) -> (Color32, Color32) {
    // Sobre los colores claros del tema oscuro va texto oscuro.
    let on_strong = if c.dark { Color32::from_rgb(16, 18, 22) } else { Color32::WHITE };
    match kind {
        BadgeKind::Accent => (c.accent, c.on_accent),
        BadgeKind::Neutral => (c.selection_inactive, c.text),
        BadgeKind::Danger => (c.danger, on_strong),
        BadgeKind::Success => (c.success, on_strong),
        BadgeKind::Warning => (c.warning, on_strong),
    }
}

/// Dibuja una insignia centrada en `center`; devuelve su rectángulo.
pub fn paint_badge(painter: &Painter, center: Pos2, text: &str, kind: BadgeKind, colors: &Palette, size: f32) -> Rect {
    let (fill, ink) = badge_colors(kind, colors);
    let galley = painter.layout_no_wrap(text.to_string(), crate::theme::medium_font(painter.ctx(), size), ink);
    let h = size + 5.0;
    let w = (galley.size().x + 10.0).max(h);
    let rect = Rect::from_center_size(center, vec2(w, h));
    painter.rect_filled(rect, h / 2.0, fill);
    painter.galley(rect.center() - galley.size() / 2.0, galley, ink);
    rect
}

/// Insignia con texto.
pub fn badge(ui: &mut Ui, text: &str, kind: BadgeKind) -> Response {
    let colors = crate::theme::colors(ui);
    let size = crate::theme::density(ui.ctx()).small_size();
    let galley = ui.painter().layout_no_wrap(text.to_string(), crate::theme::medium_font(ui.ctx(), size), colors.text);
    let h = size + 5.0;
    let w = (galley.size().x + 10.0).max(h);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    paint_badge(ui.painter(), rect.center(), text, kind, &colors, size);
    crate::a11y::label(&resp, text);
    resp
}

/// Contador (no se dibuja nada si es 0).
pub fn count_badge(ui: &mut Ui, count: u32, kind: BadgeKind) -> Option<Response> {
    (count > 0).then(|| badge(ui, &count_text(count), kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_are_capped() {
        assert_eq!(count_text(0), "0");
        assert_eq!(count_text(999), "999");
        assert_eq!(count_text(1000), "999+");
        assert_eq!(count_text(u32::MAX), "999+");
    }

    #[test]
    fn badge_text_is_readable() {
        use crate::theme::contrast::ratio;
        for p in [crate::tokens::LIGHT, crate::tokens::DARK] {
            for k in [BadgeKind::Accent, BadgeKind::Neutral, BadgeKind::Danger, BadgeKind::Success, BadgeKind::Warning]
            {
                let (fill, ink) = badge_colors(k, &p);
                assert!(ratio(ink, fill) >= 4.5, "{k:?} oscuro={}: {:.2}", p.dark, ratio(ink, fill));
            }
        }
    }
}
