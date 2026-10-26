//! Control segmentado (vista Día/Semana/Mes, densidad): opciones excluyentes en una píldora.

use egui::{Rect, Response, Sense, Stroke, Ui, pos2, vec2};

/// Dibuja las opciones y cambia `value` al pulsar una. La respuesta marca `changed()`.
/// Con el foco del teclado, ← y → cambian de opción.
pub fn segmented<T: PartialEq + Copy>(ui: &mut Ui, value: &mut T, options: &[(T, &str)]) -> Response {
    let c = crate::theme::colors(ui);
    let d = crate::theme::density(ui.ctx());
    let font = crate::theme::body_font(ui.ctx());
    let h = d.control_height();
    let pad = d.button_padding().x + 4.0;
    let widths: Vec<f32> = options
        .iter()
        .map(|(_, l)| ui.painter().layout_no_wrap((*l).to_string(), font.clone(), c.text).size().x + pad * 2.0)
        .collect();
    let total: f32 = widths.iter().sum::<f32>() + 4.0;
    let (rect, mut resp) = ui.allocate_exact_size(vec2(total, h), Sense::hover());
    let painter = ui.painter_at(rect.expand(2.0));
    painter.rect(rect, h / 2.0, c.surface_alt, Stroke::new(1.0, c.separator), egui::StrokeKind::Inside);
    let mut x = rect.left() + 2.0;
    let mut changed = false;
    let current = options.iter().position(|(v, _)| v == value);
    for (k, ((v, label), w)) in options.iter().zip(&widths).enumerate() {
        let seg = Rect::from_min_size(pos2(x, rect.top() + 2.0), vec2(*w, h - 4.0));
        let id = resp.id.with(k);
        let r = ui.interact(seg, id, Sense::click());
        let on = Some(k) == current;
        if on {
            painter.rect(seg, (h - 4.0) / 2.0, c.surface, Stroke::new(1.0, c.separator), egui::StrokeKind::Inside);
        } else if r.hovered() {
            painter.rect_filled(seg, (h - 4.0) / 2.0, c.hover);
        }
        let ink = if on { c.text } else { c.text_muted };
        let g = painter.layout_no_wrap((*label).to_string(), font.clone(), ink);
        painter.galley(seg.center() - g.size() / 2.0, g, ink);
        crate::a11y::toggle(&r, label, on);
        if r.clicked() && !on {
            *value = *v;
            changed = true;
        }
        x += w;
    }
    if (0..options.len()).any(|k| ui.ctx().memory(|m| m.has_focus(resp.id.with(k)))) {
        let (left, right) = ui.input(|i| (i.key_pressed(egui::Key::ArrowLeft), i.key_pressed(egui::Key::ArrowRight)));
        let n = options.len();
        if let Some(cur) = current
            && n > 0
        {
            let next = if right {
                Some((cur + 1) % n)
            } else if left {
                Some((cur + n - 1) % n)
            } else {
                None
            };
            if let Some((v, _)) = next.and_then(|i| options.get(i)) {
                *value = *v;
                changed = true;
            }
        }
        painter.rect_stroke(rect, h / 2.0, Stroke::new(2.0, c.focus), egui::StrokeKind::Outside);
    }
    if changed {
        resp.mark_changed();
    }
    resp
}
