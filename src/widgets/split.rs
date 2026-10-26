//! Paneles con el estilo de la suite: riel, barra lateral, lista y lectura, separados por líneas
//! finas (sin sombras). Envuelven a `egui::Panel` / `CentralPanel` con los marcos del tema.

use egui::{Frame, InnerResponse, Margin, Ui};

/// Papel del panel (decide el fondo y los márgenes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaneKind {
    /// Riel de módulos (fondo secundario, sin márgenes).
    Rail,
    /// Carpetas, filtros (fondo secundario).
    Sidebar,
    /// Lista o contenido principal (superficie).
    Content,
    /// Panel de lectura o detalle (superficie, con margen).
    Detail,
    /// Barra de herramientas o de estado.
    Bar,
}

/// Marco de un panel de ese tipo.
pub fn frame(ui: &Ui, kind: PaneKind) -> Frame {
    let c = crate::theme::colors(ui);
    match kind {
        PaneKind::Rail => Frame::new().fill(c.surface_alt).inner_margin(Margin::symmetric(6, 0)),
        PaneKind::Sidebar => Frame::new().fill(c.surface_alt).inner_margin(Margin::symmetric(8, 8)),
        PaneKind::Content => Frame::new().fill(c.surface),
        PaneKind::Detail => Frame::new().fill(c.surface).inner_margin(Margin::same(16)),
        PaneKind::Bar => Frame::new().fill(c.surface).inner_margin(Margin::symmetric(8, 4)),
    }
}

/// Panel a la izquierda; `width` es el inicial y `range` lo que se puede estirar (el riel no
/// cambia de ancho).
pub fn left<R>(
    ui: &mut Ui,
    id: &str,
    kind: PaneKind,
    width: f32,
    range: std::ops::RangeInclusive<f32>,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    side(egui::Panel::left(id.to_string()), ui, kind, width, range, add)
}

/// Panel a la derecha (panel contextual).
pub fn right<R>(
    ui: &mut Ui,
    id: &str,
    kind: PaneKind,
    width: f32,
    range: std::ops::RangeInclusive<f32>,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    side(egui::Panel::right(id.to_string()), ui, kind, width, range, add)
}

fn side<R>(
    panel: egui::Panel,
    ui: &mut Ui,
    kind: PaneKind,
    width: f32,
    range: std::ops::RangeInclusive<f32>,
    add: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let f = frame(ui, kind);
    let panel = if kind == PaneKind::Rail {
        panel.exact_size(width).resizable(false)
    } else {
        let (lo, hi) = (range.start().min(*range.end()), range.end().max(*range.start()));
        panel.default_size(width.clamp(lo, hi)).size_range(lo..=hi).resizable(true)
    };
    panel.frame(f).show(ui, add)
}

/// Barra arriba (herramientas, ≤ 6 acciones + «…»).
pub fn top_bar<R>(ui: &mut Ui, id: &str, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let f = frame(ui, PaneKind::Bar);
    egui::Panel::top(id.to_string()).frame(f).show(ui, add)
}

/// Barra abajo (estado, sincronización).
pub fn bottom_bar<R>(ui: &mut Ui, id: &str, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let f = frame(ui, PaneKind::Bar);
    egui::Panel::bottom(id.to_string()).frame(f).show(ui, add)
}

/// Lo que queda al centro.
pub fn center<R>(ui: &mut Ui, kind: PaneKind, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let f = frame(ui, kind);
    egui::CentralPanel::default().frame(f).show(ui, add)
}
