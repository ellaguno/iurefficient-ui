//! Atajos de teclado dibujados como teclas («Ctrl» «K»), para la paleta, los menús y los campos.

use egui::{FontId, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, pos2, vec2};

use crate::tokens::Palette;

/// Partes de un atajo: «Ctrl+Shift+K» → [«Ctrl», «Shift», «K»]. Un «+» suelto es una tecla.
pub fn split_shortcut(text: &str) -> Vec<&str> {
    if text == "+" {
        return vec!["+"];
    }
    let mut out: Vec<&str> = text.split('+').filter(|s| !s.trim().is_empty()).map(str::trim).collect();
    if text.ends_with("++") {
        out.push("+");
    }
    out
}

/// Dibuja el atajo alineado a la derecha en `right_center`; devuelve el ancho usado.
pub fn paint_shortcut(painter: &Painter, right_center: Pos2, text: &str, font: &FontId, colors: &Palette) -> f32 {
    let keys = split_shortcut(text);
    let mut x = right_center.x;
    let pad = 4.0;
    let h = font.size + 6.0;
    for key in keys.iter().rev() {
        let galley = painter.layout_no_wrap((*key).to_string(), font.clone(), colors.text_muted);
        let w = (galley.size().x + pad * 2.0).max(h);
        let rect = Rect::from_min_size(pos2(x - w, right_center.y - h / 2.0), vec2(w, h));
        painter.rect(rect, 4.0, colors.surface_alt, Stroke::new(1.0, colors.separator), egui::StrokeKind::Inside);
        painter.galley(rect.center() - galley.size() / 2.0, galley, colors.text_muted);
        x -= w + 3.0;
    }
    right_center.x - x
}

/// Un atajo como widget (ocupa su lugar en el `Ui`).
pub fn keycap(ui: &mut Ui, text: &str) -> Response {
    let font = crate::theme::small_font(ui.ctx());
    let colors = crate::theme::colors(ui);
    let keys = split_shortcut(text);
    let width: f32 = keys
        .iter()
        .map(|k| {
            let w = ui.fonts_mut(|f| f.layout_no_wrap((*k).to_string(), font.clone(), colors.text_muted).size().x);
            (w + 8.0).max(font.size + 6.0) + 3.0
        })
        .sum::<f32>()
        - 3.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(width.max(0.0), font.size + 6.0), Sense::hover());
    paint_shortcut(ui.painter(), pos2(rect.right(), rect.center().y), text, &font, &colors);
    crate::a11y::label(&resp, text);
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_split_into_keys() {
        assert_eq!(split_shortcut("Ctrl+Shift+K"), vec!["Ctrl", "Shift", "K"]);
        assert_eq!(split_shortcut("Ctrl++"), vec!["Ctrl", "+"]);
        assert_eq!(split_shortcut("+"), vec!["+"]);
        assert_eq!(split_shortcut("F2"), vec!["F2"]);
    }
}
