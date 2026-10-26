//! Avisos en línea (banner) y notificaciones pasajeras (toast) con un botón de acción opcional
//! («Deshacer», «Reintentar»). Mensajes en lenguaje humano: «No se pudo conectar con Gmail —
//! revisa tu internet», no el código del servidor.

use egui::{Align, Color32, Context, Layout, Rect, Response, RichText, Sense, Stroke, Ui, pos2, vec2};

use crate::icons::{self, Icon};
use crate::lang::pick;
use crate::tokens::Palette;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Severity {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl Severity {
    /// Color de la franja y del icono.
    pub fn color(self, c: &Palette) -> Color32 {
        match self {
            Severity::Info => c.info,
            Severity::Success => c.success,
            Severity::Warning => c.warning,
            Severity::Error => c.danger,
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Severity::Info => Icon::Info,
            Severity::Success => Icon::Check,
            Severity::Warning => Icon::Warning,
            Severity::Error => Icon::Alert,
        }
    }

    /// Nombre para el lector de pantalla.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Info => pick("Information", "Información"),
            Severity::Success => pick("Done", "Hecho"),
            Severity::Warning => pick("Warning", "Atención"),
            Severity::Error => pick("Error", "Error"),
        }
    }
}

/// Lo que hizo el usuario con el aviso.
#[derive(Debug)]
pub struct BannerResponse {
    /// Pulsó el botón de acción.
    pub action: bool,
    /// Pulsó cerrar.
    pub dismissed: bool,
    pub response: Response,
}

/// Aviso en línea, a todo el ancho.
#[derive(Clone, Debug)]
pub struct Banner<'a> {
    severity: Severity,
    text: &'a str,
    title: Option<&'a str>,
    action: Option<&'a str>,
    dismissible: bool,
}

impl<'a> Banner<'a> {
    pub fn new(severity: Severity, text: &'a str) -> Self {
        Banner { severity, text, title: None, action: None, dismissible: false }
    }
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
    pub fn action(mut self, label: &'a str) -> Self {
        self.action = Some(label);
        self
    }
    pub fn dismissible(mut self, yes: bool) -> Self {
        self.dismissible = yes;
        self
    }

    pub fn show(self, ui: &mut Ui) -> BannerResponse {
        let c = crate::theme::colors(ui);
        let tone = self.severity.color(&c);
        let fill = crate::theme::blend(c.surface, tone.gamma_multiply(if c.dark { 0.16 } else { 0.09 }));
        let mut action = false;
        let mut dismissed = false;
        let inner = egui::Frame::new()
            .fill(fill)
            .stroke(Stroke::new(1.0, crate::theme::blend(c.surface, tone.gamma_multiply(0.35))))
            .corner_radius(crate::tokens::radius::CARD as u8)
            .inner_margin(egui::Margin { left: 12, right: 8, top: 8, bottom: 8 })
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                    icons::paint(ui.painter(), r, self.severity.icon(), tone, tone);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.dismissible
                            && icons::icon_button(ui, Icon::Close, pick("Dismiss", "Cerrar"), true).clicked()
                        {
                            dismissed = true;
                        }
                        if let Some(label) = self.action
                            && ui.button(RichText::new(label).color(tone).strong()).clicked()
                        {
                            action = true;
                        }
                        ui.with_layout(Layout::left_to_right(Align::Center).with_main_wrap(true), |ui| {
                            if let Some(t) = self.title {
                                ui.label(RichText::new(t).strong().color(c.text));
                            }
                            ui.add(egui::Label::new(RichText::new(self.text).color(c.text)).wrap());
                        });
                    });
                });
            });
        let label = match self.title {
            Some(t) => format!("{}: {t}. {}", self.severity.label(), self.text),
            None => format!("{}: {}", self.severity.label(), self.text),
        };
        crate::a11y::label(&inner.response, &label);
        BannerResponse { action, dismissed, response: inner.response }
    }
}

/// Una notificación pasajera.
#[derive(Clone, Debug)]
struct Toast {
    id: u64,
    severity: Severity,
    text: String,
    action: Option<String>,
    seconds: f32,
    /// Momento (reloj de egui) en que desaparece; se fija al mostrarla por primera vez.
    until: Option<f64>,
}

/// Pila de notificaciones pasajeras abajo al centro. Guárdala en la app.
#[derive(Clone, Debug, Default)]
pub struct Toasts {
    items: Vec<Toast>,
    next: u64,
}

/// Cuántas notificaciones se ven a la vez (las más nuevas).
const MAX_TOASTS: usize = 4;

impl Toasts {
    /// Agrega una notificación que dura `seconds` (con acción, conviene ≥ 8 s). Devuelve su id.
    pub fn push(&mut self, severity: Severity, text: impl Into<String>, action: Option<&str>, seconds: f32) -> u64 {
        self.next = self.next.wrapping_add(1);
        let seconds = if seconds.is_finite() { seconds.clamp(1.0, 120.0) } else { 6.0 };
        self.items.push(Toast {
            id: self.next,
            severity,
            text: text.into(),
            action: action.map(str::to_string),
            seconds,
            until: None,
        });
        if self.items.len() > 32 {
            self.items.remove(0);
        }
        self.next
    }

    pub fn dismiss(&mut self, id: u64) {
        self.items.retain(|t| t.id != id);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Dibuja las notificaciones y quita las vencidas. Devuelve el id de la que tuvo su acción
    /// pulsada (y la cierra).
    pub fn show(&mut self, ctx: &Context) -> Option<u64> {
        let now = ctx.input(|i| i.time);
        for t in &mut self.items {
            if t.until.is_none() {
                t.until = Some(now + f64::from(t.seconds));
            }
        }
        self.items.retain(|t| t.until.is_none_or(|u| u > now));
        if self.items.is_empty() {
            return None;
        }
        if let Some(next) = self.items.iter().filter_map(|t| t.until).reduce(f64::min) {
            ctx.request_repaint_after(std::time::Duration::from_secs_f64((next - now).clamp(0.05, 120.0)));
        }
        let mut clicked = None;
        let mut closed = Vec::new();
        let start = self.items.len().saturating_sub(MAX_TOASTS);
        egui::Area::new(egui::Id::unique("iurefficient-ui/toasts"))
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::CENTER_BOTTOM, vec2(0.0, -24.0))
            .interactable(true)
            .show(ctx, |ui| {
                ui.set_width(440.0);
                let c = crate::theme::colors(ui);
                for t in self.items.iter().skip(start) {
                    let tone = t.severity.color(&c);
                    let frame = egui::Frame::new()
                        .fill(c.surface)
                        .stroke(Stroke::new(1.0, c.separator))
                        .corner_radius(crate::tokens::radius::CARD as u8)
                        .shadow(ui.visuals().popup_shadow)
                        .inner_margin(egui::Margin { left: 14, right: 8, top: 8, bottom: 8 });
                    let r = frame.show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let (ir, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                            icons::paint(ui.painter(), ir, t.severity.icon(), tone, tone);
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if icons::icon_button(ui, Icon::Close, pick("Dismiss", "Cerrar"), true).clicked() {
                                    closed.push(t.id);
                                }
                                if let Some(a) = &t.action
                                    && ui.button(RichText::new(a).color(c.accent).strong()).clicked()
                                {
                                    clicked = Some(t.id);
                                }
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    ui.add(egui::Label::new(RichText::new(&t.text).color(c.text)).wrap());
                                });
                            });
                        });
                    });
                    // Franja de color a la izquierda.
                    let rr = r.response.rect;
                    ui.painter().rect_filled(
                        Rect::from_min_max(
                            pos2(rr.left() + 1.0, rr.top() + 6.0),
                            pos2(rr.left() + 4.0, rr.bottom() - 6.0),
                        ),
                        2.0,
                        tone,
                    );
                    crate::a11y::label(&r.response, &format!("{}: {}", t.severity.label(), t.text));
                    ui.add_space(6.0);
                }
            });
        if let Some(id) = clicked {
            closed.push(id);
        }
        self.items.retain(|t| !closed.contains(&t.id));
        clicked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toasts_expire_and_are_bounded() {
        let mut t = Toasts::default();
        for i in 0..40 {
            t.push(Severity::Info, format!("n{i}"), None, f32::NAN);
        }
        assert_eq!(t.len(), 32);
        let id = t.push(Severity::Error, "x", Some("Reintentar"), 5.0);
        assert_eq!(t.len(), 32);
        t.dismiss(id);
        assert_eq!(t.len(), 31);
        let ctx = Context::default();
        let mut input = egui::RawInput { time: Some(0.0), ..Default::default() };
        let mut out = ctx.run_ui(input.clone(), |ui| {
            t.show(ui.ctx());
        });
        out.textures_delta.clear();
        assert_eq!(t.len(), 31);
        input.time = Some(100.0);
        let mut out = ctx.run_ui(input, |ui| {
            t.show(ui.ctx());
        });
        out.textures_delta.clear();
        assert!(t.is_empty(), "vencidas a los 6 s");
    }
}
