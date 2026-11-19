//! La ventana sin la barra del sistema (barra de título integrada), para las apps de la suite:
//! botones de la ventana con el estilo y el orden del escritorio, arrastrar para mover, doble clic
//! para maximizar, menú de la ventana con clic derecho, bordes invisibles para cambiar el tamaño y
//! el borde de 1 px.
//!
//! Lo que va dentro de la barra (pestañas, título, búsqueda) lo pone cada app; aquí está lo que
//! es igual en todas. Salió de la barra de IureDesk (UI-HEADER).

use egui::{
    Color32, Context, CursorIcon, Id, PointerButton, Rect, ResizeDirection, Sense, Stroke, Ui, ViewportCommand,
};
use egui::{pos2, vec2};

use crate::{Icon, a11y, icons, pick, theme};

/// Un botón de la ventana.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowButton {
    Minimize,
    Maximize,
    Close,
}

/// Qué botones van a cada lado (en el orden en que se dibujan, de izquierda a derecha).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonLayout {
    pub left: Vec<WindowButton>,
    pub right: Vec<WindowButton>,
}

impl Default for ButtonLayout {
    fn default() -> Self {
        ButtonLayout::standard()
    }
}

impl ButtonLayout {
    /// Minimizar, maximizar y cerrar a la derecha (Windows, Ubuntu, KDE).
    pub fn standard() -> Self {
        ButtonLayout {
            left: Vec::new(),
            right: vec![WindowButton::Minimize, WindowButton::Maximize, WindowButton::Close],
        }
    }

    /// Lee el formato de GNOME (`org.gnome.desktop.wm.preferences button-layout`), p. ej.
    /// `appmenu:minimize,maximize,close` o `close,minimize,maximize:`. Lo de antes de `:` va a la
    /// izquierda. Se ignoran los nombres que no son botones y los repetidos. `None` si el texto
    /// está vacío, es demasiado largo o no dice nada útil.
    pub fn parse(setting: &str) -> Option<Self> {
        let s = setting.trim().trim_matches(|c| c == '\'' || c == '"').trim();
        if s.is_empty() || s.len() > 256 {
            return None;
        }
        let (l, r) = s.split_once(':').unwrap_or((s, ""));
        let mut seen: Vec<WindowButton> = Vec::new();
        let mut side = |part: &str| {
            let mut v = Vec::new();
            for name in part.split(',').take(16) {
                let b = match name.trim() {
                    "minimize" => WindowButton::Minimize,
                    "maximize" => WindowButton::Maximize,
                    "close" => WindowButton::Close,
                    _ => continue,
                };
                if !seen.contains(&b) {
                    seen.push(b);
                    v.push(b);
                }
            }
            v
        };
        let left = side(l);
        let right = side(r);
        if left.is_empty() && right.is_empty() && !s.contains(':') {
            return None;
        }
        Some(ButtonLayout { left, right })
    }
}

/// Aspecto de los botones de la ventana.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlStyle {
    /// Redondos, como GNOME/Adwaita (Linux).
    Adwaita,
    /// Rectangulares a todo lo alto, cerrar en rojo al pasar (Windows).
    Windows,
    /// macOS: los semáforos del sistema siguen ahí; solo se deja su hueco a la izquierda.
    MacOs,
}

/// La ventana sin la barra del sistema: estilo y orden de los botones.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowChrome {
    pub style: ControlStyle,
    pub layout: ButtonLayout,
}

impl WindowChrome {
    /// La de este sistema; en Linux con el orden de botones del escritorio si se pudo leer.
    pub fn for_this_os(button_layout: Option<&str>) -> Self {
        if cfg!(target_os = "macos") {
            WindowChrome { style: ControlStyle::MacOs, layout: ButtonLayout { left: Vec::new(), right: Vec::new() } }
        } else if cfg!(target_os = "windows") {
            WindowChrome { style: ControlStyle::Windows, layout: ButtonLayout::standard() }
        } else {
            WindowChrome {
                style: ControlStyle::Adwaita,
                layout: button_layout.and_then(ButtonLayout::parse).unwrap_or_default(),
            }
        }
    }

    /// La de este sistema leyendo el orden de los botones del escritorio (portal XDG, con un tope
    /// de 300 ms: si el bus tarda, el orden de siempre).
    pub fn detect() -> Self {
        WindowChrome::for_this_os(desktop_button_layout().as_deref())
    }

    /// Por nombre (`adwaita`, `windows`, `macos`), para capturas.
    pub fn named(name: &str) -> Option<Self> {
        match name {
            "adwaita" | "linux" | "gnome" => {
                Some(WindowChrome { style: ControlStyle::Adwaita, layout: ButtonLayout::standard() })
            }
            "windows" => Some(WindowChrome { style: ControlStyle::Windows, layout: ButtonLayout::standard() }),
            "macos" | "mac" => Some(WindowChrome {
                style: ControlStyle::MacOs,
                layout: ButtonLayout { left: Vec::new(), right: Vec::new() },
            }),
            _ => None,
        }
    }

    /// ¿La app dibuja borde y bordes de cambio de tamaño? (En macOS los pone el sistema.)
    pub fn owns_frame(&self) -> bool {
        self.style != ControlStyle::MacOs
    }

    /// Hueco a la izquierda para los semáforos de macOS (fuera de pantalla completa).
    pub fn left_inset(&self, ctx: &Context) -> f32 {
        if self.style == ControlStyle::MacOs && !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)) {
            68.0
        } else {
            0.0
        }
    }
}

/// `button-layout` del escritorio (GNOME) por el portal de configuración XDG.
#[cfg(all(target_os = "linux", feature = "portal"))]
fn desktop_button_layout() -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("button-layout".into())
        .spawn(move || {
            let _ = tx.send(crate::system::portal_string("org.gnome.desktop.wm.preferences", "button-layout"));
        })
        .ok()?;
    rx.recv_timeout(std::time::Duration::from_millis(300)).ok().flatten()
}

#[cfg(not(all(target_os = "linux", feature = "portal")))]
fn desktop_button_layout() -> Option<String> {
    None
}

/// Alto de la barra de título según la densidad.
pub fn title_bar_height(ctx: &Context) -> f32 {
    match theme::density(ctx) {
        crate::Density::Comfortable => 40.0,
        crate::Density::Compact => 36.0,
        crate::Density::Dense => 32.0,
    }
}

/// ¿Maximizada o en pantalla completa?
pub fn maximized(ctx: &Context) -> bool {
    ctx.input(|i| i.viewport().maximized.unwrap_or(false) || i.viewport().fullscreen.unwrap_or(false))
}

fn toggle_maximize(ctx: &Context) {
    let max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    ctx.send_viewport_cmd(ViewportCommand::Maximized(!max));
}

/// Lo que hace un botón de la ventana.
pub fn press(ctx: &Context, b: WindowButton) {
    match b {
        WindowButton::Minimize => ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),
        WindowButton::Maximize => toggle_maximize(ctx),
        WindowButton::Close => ctx.send_viewport_cmd(ViewportCommand::Close),
    }
}

/// Fondo de la barra: arrastrar mueve, doble clic maximiza, clic derecho abre el menú de la
/// ventana. Se llama antes que los controles de la barra (que tienen prioridad).
pub fn drag_area(ui: &mut Ui, rect: Rect, id: Id) {
    let ctx = ui.ctx().clone();
    let resp = ui.interact(rect, id, Sense::click_and_drag());
    a11y::area(&resp, pick("Title bar", "Barra de título"));
    if resp.drag_started_by(PointerButton::Primary) {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }
    if resp.double_clicked() {
        toggle_maximize(&ctx);
    }
    let max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    resp.context_menu(|ui| {
        if icons::menu_item(ui, Some(Icon::Minimize), pick("Minimize", "Minimizar")).clicked() {
            press(&ctx, WindowButton::Minimize);
            ui.close();
        }
        let (icon, label) = if max {
            (Icon::Restore, pick("Restore", "Restaurar"))
        } else {
            (Icon::Maximize, pick("Maximize", "Maximizar"))
        };
        if icons::menu_item(ui, Some(icon), label).clicked() {
            toggle_maximize(&ctx);
            ui.close();
        }
        ui.separator();
        if icons::menu_item(ui, Some(Icon::Close), pick("Close", "Cerrar")).clicked() {
            press(&ctx, WindowButton::Close);
            ui.close();
        }
    });
}

/// Un botón de la ventana (`h`: alto de la barra).
pub fn window_button(ui: &mut Ui, b: WindowButton, style: ControlStyle, h: f32) -> egui::Response {
    let c = theme::colors(ui);
    let is_max = maximized(ui.ctx());
    let (icon, label) = match b {
        WindowButton::Minimize => (Icon::Minimize, pick("Minimize", "Minimizar")),
        WindowButton::Maximize if is_max => (Icon::Restore, pick("Restore", "Restaurar")),
        WindowButton::Maximize => (Icon::Maximize, pick("Maximize", "Maximizar")),
        WindowButton::Close => (Icon::Close, pick("Close", "Cerrar")),
    };
    let resp = match style {
        ControlStyle::Windows => {
            let (rect, resp) = ui.allocate_exact_size(vec2(46.0, h), Sense::click());
            let close = b == WindowButton::Close;
            let hot = resp.hovered() || resp.has_focus();
            let (fill, ink) = if close && hot {
                (Color32::from_rgb(196, 43, 28), Color32::WHITE)
            } else if resp.is_pointer_button_down_on() {
                (c.pressed, c.text)
            } else if hot {
                (c.hover, c.text)
            } else {
                (Color32::TRANSPARENT, c.text)
            };
            ui.painter().rect_filled(rect, 0.0, fill);
            icons::paint(ui.painter(), Rect::from_center_size(rect.center(), vec2(15.0, 15.0)), icon, ink, ink);
            if resp.has_focus() {
                ui.painter().rect_stroke(rect.shrink(2.0), 2.0, Stroke::new(1.5, c.focus), egui::StrokeKind::Inside);
            }
            resp
        }
        ControlStyle::Adwaita | ControlStyle::MacOs => {
            let d = 24.0;
            let (rect, resp) = ui.allocate_exact_size(vec2(d, d), Sense::click());
            let a = if resp.is_pointer_button_down_on() {
                0.26
            } else if resp.hovered() {
                0.17
            } else {
                0.09
            };
            let base = if c.dark { Color32::WHITE } else { Color32::BLACK };
            ui.painter().circle_filled(rect.center(), d / 2.0, base.gamma_multiply(a));
            icons::paint(ui.painter(), Rect::from_center_size(rect.center(), vec2(15.0, 15.0)), icon, c.text, c.text);
            if resp.has_focus() {
                ui.painter().circle_stroke(rect.center(), d / 2.0 + 1.0, Stroke::new(1.5, c.focus));
            }
            resp
        }
    };
    a11y::button(&resp, label);
    resp.on_hover_text(label)
}

/// Los botones de un lado de la barra (`right`: en un layout de derecha a izquierda, se dibujan
/// al revés para quedar en su orden).
pub fn window_buttons(ui: &mut Ui, chrome: &WindowChrome, right: bool, h: f32) {
    let ctx = ui.ctx().clone();
    let list = if right { &chrome.layout.right } else { &chrome.layout.left };
    let mut order: Vec<WindowButton> = list.clone();
    if right {
        order.reverse();
    }
    let gap = if chrome.style == ControlStyle::Windows { 0.0 } else { 8.0 };
    let old = ui.spacing().item_spacing.x;
    ui.spacing_mut().item_spacing.x = gap;
    for b in order {
        if window_button(ui, b, chrome.style, h).clicked() {
            press(&ctx, b);
        }
    }
    ui.spacing_mut().item_spacing.x = old;
}

/// Bordes invisibles para cambiar el tamaño (no si está maximizada) y el borde de 1 px. Va al
/// final del cuadro para quedar encima de los paneles.
pub fn frame(ui: &mut Ui, chrome: &WindowChrome, id: Id) {
    let ctx = ui.ctx().clone();
    if !chrome.owns_frame() || maximized(&ctx) {
        return;
    }
    let r = ctx.content_rect();
    let c = theme::colors(ui);
    ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, id.with("border"))).rect_stroke(
        r,
        0.0,
        Stroke::new(1.0, c.separator),
        egui::StrokeKind::Inside,
    );
    for (i, (dir, rect, cursor)) in grips(r).into_iter().enumerate() {
        let resp = ui.interact(rect, id.with(("resize", i)), Sense::drag());
        if resp.hovered() || resp.dragged() {
            ctx.set_cursor_icon(cursor);
        }
        if resp.drag_started_by(PointerButton::Primary) {
            ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
        }
    }
}

/// Zonas de cambio de tamaño: 6 px por lado, esquinas de 12 px.
fn grips(r: Rect) -> [(ResizeDirection, Rect, CursorIcon); 8] {
    let (g, k) = (6.0, 12.0);
    let (l, t, rt, b) = (r.left(), r.top(), r.right(), r.bottom());
    let rr = Rect::from_min_max;
    [
        (ResizeDirection::North, rr(pos2(l + k, t), pos2(rt - k, t + g)), CursorIcon::ResizeNorth),
        (ResizeDirection::South, rr(pos2(l + k, b - g), pos2(rt - k, b)), CursorIcon::ResizeSouth),
        (ResizeDirection::West, rr(pos2(l, t + k), pos2(l + g, b - k)), CursorIcon::ResizeWest),
        (ResizeDirection::East, rr(pos2(rt - g, t + k), pos2(rt, b - k)), CursorIcon::ResizeEast),
        (ResizeDirection::NorthWest, rr(pos2(l, t), pos2(l + k, t + k)), CursorIcon::ResizeNorthWest),
        (ResizeDirection::NorthEast, rr(pos2(rt - k, t), pos2(rt, t + k)), CursorIcon::ResizeNorthEast),
        (ResizeDirection::SouthWest, rr(pos2(l, b - k), pos2(l + k, b)), CursorIcon::ResizeSouthWest),
        (ResizeDirection::SouthEast, rr(pos2(rt - k, b - k), pos2(rt, b)), CursorIcon::ResizeSouthEast),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use WindowButton::*;

    #[test]
    fn parses_gnome_button_layouts() {
        let l = ButtonLayout::parse("appmenu:minimize,maximize,close").unwrap();
        assert_eq!(l, ButtonLayout { left: vec![], right: vec![Minimize, Maximize, Close] });
        let l = ButtonLayout::parse("'close,minimize,maximize:'").unwrap();
        assert_eq!(l, ButtonLayout { left: vec![Close, Minimize, Maximize], right: vec![] });
        assert_eq!(ButtonLayout::parse("appmenu:close").unwrap().right, vec![Close]);
        let l = ButtonLayout::parse(" close , spacer,close,icon ").unwrap();
        assert_eq!(l, ButtonLayout { left: vec![Close], right: vec![] });
        assert_eq!(ButtonLayout::parse("appmenu:").unwrap(), ButtonLayout { left: vec![], right: vec![] });
    }

    #[test]
    fn hostile_layouts_fall_back() {
        assert!(ButtonLayout::parse("").is_none());
        assert!(ButtonLayout::parse("tonterías").is_none());
        assert!(ButtonLayout::parse(&"close,".repeat(200)).is_none());
        assert_eq!(
            WindowChrome::for_this_os(Some("")).layout.right.len(),
            if cfg!(target_os = "macos") { 0 } else { 3 }
        );
    }

    #[test]
    fn grips_cover_the_edges_without_overlap() {
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
        let g = grips(r);
        for (i, (_, a, _)) in g.iter().enumerate() {
            assert!(r.contains_rect(*a));
            for (_, b, _) in g.iter().skip(i + 1) {
                assert!(!a.intersects(*b) || a.intersect(*b).area() == 0.0);
            }
        }
    }
}
