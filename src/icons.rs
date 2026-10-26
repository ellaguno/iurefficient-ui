//! Iconos vectoriales dibujados en código (nada de iconos de Microsoft, Apple ni archivos de
//! terceros). Cada icono se dibuja en una caja de 24×24 unidades escalada al rectángulo pedido, con
//! trazos de 1.6 unidades, el color del texto (`ink`) y un color de acento para lo que debe
//! resaltar.
//!
//! Los primeros vienen de IureProject (cinta y Gantt); los del gestor personal (correo, agenda,
//! contactos, tareas, asistente) se agregaron aquí con el mismo trazo.

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    // --- Proyecto (IureProject) ---
    Gantt,
    Sheet,
    TaskUsage,
    ResourceUsage,
    Board,
    Timeline,
    Indent,
    Outdent,
    /// Vínculo entre tareas (dos barras unidas). Para un enlace web: [`Icon::Hyperlink`].
    Link,
    Unlink,
    Milestone,
    InsertTask,
    Delete,
    Info,
    Notes,
    Split,
    Unsplit,
    Manual,
    Undo,
    Redo,
    Calculate,
    Level,
    ClearLevel,
    AddResource,
    RemoveResource,
    ProjectInfo,
    StatusDate,
    Cloud,
    CloudUp,
    Clock,
    ZoomIn,
    ZoomOut,
    Fit,
    Percent(u8),
    Outline,
    Critical,
    Calendar,
    New,
    Open,
    Save,
    SaveAs,
    Export,
    Back,
    Baseline,
    Tracking,
    Print,
    Network,
    ResourceGraph,
    Rename,
    Sprint,
    Flag,
    Pin,
    Priority,
    Status,
    StartDate,
    FinishDate,
    Assign,
    Contour,
    ClearContour,
    Sliders,
    Bolt,
    List,
    ClearBaseline,
    Person,
    Actual,
    MoveUp,
    MoveDown,
    Money,
    Comment,
    Attachment,
    // --- Gestor personal (IureDesk) ---
    Mail,
    Inbox,
    Send,
    Drafts,
    Archive,
    Trash,
    Junk,
    Star,
    StarFilled,
    Reply,
    ReplyAll,
    Forward,
    Contacts,
    Tasks,
    Check,
    Search,
    Settings,
    Plus,
    MoreHorizontal,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    ChevronDown,
    Sparkle,
    Sync,
    Offline,
    Bell,
    Tag,
    Folder,
    Hyperlink,
    Phone,
    Video,
    Close,
    Filter,
    Lock,
    Menu,
    Warning,
    Alert,
}

impl Icon {
    /// Todos los iconos con su nombre estable (el de `CommandSpec::icon` en las apps).
    pub const ALL: &[(&str, Icon)] = &[
        ("gantt", Icon::Gantt),
        ("sheet", Icon::Sheet),
        ("task-usage", Icon::TaskUsage),
        ("resource-usage", Icon::ResourceUsage),
        ("board", Icon::Board),
        ("timeline", Icon::Timeline),
        ("indent", Icon::Indent),
        ("outdent", Icon::Outdent),
        ("link", Icon::Link),
        ("unlink", Icon::Unlink),
        ("milestone", Icon::Milestone),
        ("insert-task", Icon::InsertTask),
        ("delete", Icon::Delete),
        ("info", Icon::Info),
        ("notes", Icon::Notes),
        ("split", Icon::Split),
        ("unsplit", Icon::Unsplit),
        ("manual", Icon::Manual),
        ("undo", Icon::Undo),
        ("redo", Icon::Redo),
        ("calculate", Icon::Calculate),
        ("level", Icon::Level),
        ("clear-level", Icon::ClearLevel),
        ("add-resource", Icon::AddResource),
        ("remove-resource", Icon::RemoveResource),
        ("project-info", Icon::ProjectInfo),
        ("status-date", Icon::StatusDate),
        ("cloud", Icon::Cloud),
        ("cloud-up", Icon::CloudUp),
        ("clock", Icon::Clock),
        ("zoom-in", Icon::ZoomIn),
        ("zoom-out", Icon::ZoomOut),
        ("fit", Icon::Fit),
        ("percent", Icon::Percent(50)),
        ("outline", Icon::Outline),
        ("critical", Icon::Critical),
        ("calendar", Icon::Calendar),
        ("new", Icon::New),
        ("open", Icon::Open),
        ("save", Icon::Save),
        ("save-as", Icon::SaveAs),
        ("export", Icon::Export),
        ("back", Icon::Back),
        ("baseline", Icon::Baseline),
        ("tracking", Icon::Tracking),
        ("print", Icon::Print),
        ("network", Icon::Network),
        ("resource-graph", Icon::ResourceGraph),
        ("rename", Icon::Rename),
        ("sprint", Icon::Sprint),
        ("flag", Icon::Flag),
        ("pin", Icon::Pin),
        ("priority", Icon::Priority),
        ("status", Icon::Status),
        ("start-date", Icon::StartDate),
        ("finish-date", Icon::FinishDate),
        ("assign", Icon::Assign),
        ("contour", Icon::Contour),
        ("clear-contour", Icon::ClearContour),
        ("sliders", Icon::Sliders),
        ("bolt", Icon::Bolt),
        ("list", Icon::List),
        ("clear-baseline", Icon::ClearBaseline),
        ("person", Icon::Person),
        ("actual", Icon::Actual),
        ("move-up", Icon::MoveUp),
        ("move-down", Icon::MoveDown),
        ("money", Icon::Money),
        ("comment", Icon::Comment),
        ("attachment", Icon::Attachment),
        ("mail", Icon::Mail),
        ("inbox", Icon::Inbox),
        ("send", Icon::Send),
        ("drafts", Icon::Drafts),
        ("archive", Icon::Archive),
        ("trash", Icon::Trash),
        ("junk", Icon::Junk),
        ("star", Icon::Star),
        ("star-filled", Icon::StarFilled),
        ("reply", Icon::Reply),
        ("reply-all", Icon::ReplyAll),
        ("forward", Icon::Forward),
        ("contacts", Icon::Contacts),
        ("tasks", Icon::Tasks),
        ("check", Icon::Check),
        ("search", Icon::Search),
        ("settings", Icon::Settings),
        ("plus", Icon::Plus),
        ("more-horizontal", Icon::MoreHorizontal),
        ("chevron-left", Icon::ChevronLeft),
        ("chevron-right", Icon::ChevronRight),
        ("chevron-up", Icon::ChevronUp),
        ("chevron-down", Icon::ChevronDown),
        ("sparkle", Icon::Sparkle),
        ("sync", Icon::Sync),
        ("offline", Icon::Offline),
        ("bell", Icon::Bell),
        ("tag", Icon::Tag),
        ("folder", Icon::Folder),
        ("hyperlink", Icon::Hyperlink),
        ("phone", Icon::Phone),
        ("video", Icon::Video),
        ("close", Icon::Close),
        ("filter", Icon::Filter),
        ("lock", Icon::Lock),
        ("menu", Icon::Menu),
        ("warning", Icon::Warning),
        ("alert", Icon::Alert),
    ];

    /// El icono de un nombre estable; `None` si el nombre no se conoce.
    pub fn named(name: &str) -> Option<Icon> {
        Icon::ALL.iter().find(|(n, _)| *n == name).map(|(_, i)| *i)
    }

    /// El nombre estable del icono (`Percent(_)` es «percent»).
    pub fn name(self) -> &'static str {
        let key = if let Icon::Percent(_) = self { Icon::Percent(50) } else { self };
        Icon::ALL.iter().find(|(_, i)| *i == key).map_or("", |(n, _)| *n)
    }
}

/// Dibuja `icon` con los colores del tema de `ui` (texto y acento).
pub fn paint_themed(ui: &egui::Ui, rect: Rect, icon: Icon) {
    let c = crate::theme::colors(ui);
    paint(ui.painter(), rect, icon, c.text, c.accent);
}

/// Renglón de menú con su icono a la izquierda (menús contextuales). Sin icono deja el hueco,
/// para que los textos queden alineados.
pub fn menu_item(ui: &mut egui::Ui, icon: Option<Icon>, label: &str) -> egui::Response {
    let id = egui::IdSalt::new("icono");
    let side = 16.0;
    let out = egui::Button::new((egui::Atom::custom(id, vec2(side, side)), label)).atom_ui(ui);
    if let (Some(icon), Some(r)) = (icon, out.rect(id)) {
        let c = crate::theme::colors(ui);
        let ink = if out.response.enabled() { c.text } else { c.text_muted };
        paint(ui.painter(), r, icon, ink, c.accent);
    }
    out.response
}

/// Botón pequeño solo con icono (y su texto para el lector de pantalla y la ayuda). El tamaño del
/// icono sigue a la densidad.
pub fn icon_button(ui: &mut egui::Ui, icon: Icon, tip: &str, enabled: bool) -> egui::Response {
    let id = egui::IdSalt::new("icono");
    let side = crate::theme::density(ui.ctx()).icon_size();
    let out = ui
        .add_enabled_ui(enabled, |ui| {
            egui::Button::new(egui::Atom::custom(id, vec2(side, side))).frame_when_inactive(false).atom_ui(ui)
        })
        .inner;
    if let Some(r) = out.rect(id) {
        let c = crate::theme::colors(ui);
        let ink = if enabled { c.text } else { c.text_muted };
        paint(ui.painter(), r, icon, ink, if enabled { c.accent } else { ink });
    }
    crate::a11y::button(&out.response, tip);
    out.response.on_hover_text(tip)
}

/// Dibuja `icon` dentro de `rect` (se usa el cuadrado centrado más grande).
pub fn paint(p: &Painter, rect: Rect, icon: Icon, ink: Color32, accent: Color32) {
    let side = rect.width().min(rect.height());
    let o = rect.center() - vec2(side, side) / 2.0;
    let k = side / 24.0;
    let at = |x: f32, y: f32| pos2(o.x + x * k, o.y + y * k);
    let s = Stroke::new((1.6 * k).max(1.0), ink);
    let sa = Stroke::new((1.8 * k).max(1.0), accent);
    let line = |pts: &[(f32, f32)], st: Stroke| {
        p.add(Shape::line(pts.iter().map(|&(x, y)| at(x, y)).collect(), st));
    };
    let bar = |x: f32, y: f32, w: f32, h: f32, c: Color32| {
        p.rect_filled(Rect::from_min_size(at(x, y), vec2(w * k, h * k)), 1.5 * k, c);
    };
    let frame = |x: f32, y: f32, w: f32, h: f32| {
        p.rect_stroke(Rect::from_min_size(at(x, y), vec2(w * k, h * k)), 2.0 * k, s, egui::StrokeKind::Middle);
    };
    let circle = |x: f32, y: f32, r: f32, st: Stroke| p.circle_stroke(at(x, y), r * k, st);
    let arc = |cx: f32, cy: f32, r: f32, a0: f32, a1: f32, st: Stroke| {
        let n = 16;
        let pts: Vec<Pos2> = (0..=n)
            .map(|i| {
                let a = a0 + (a1 - a0) * i as f32 / n as f32;
                at(cx + r * a.cos(), cy + r * a.sin())
            })
            .collect();
        p.add(Shape::line(pts, st));
    };
    let arrow_head = |x: f32, y: f32, dx: f32, dy: f32, st: Stroke| {
        // Punta en (x, y) apuntando hacia (dx, dy).
        let (px, py) = (-dy, dx);
        line(
            &[
                (x - dx * 3.0 + px * 2.5, y - dy * 3.0 + py * 2.5),
                (x, y),
                (x - dx * 3.0 - px * 2.5, y - dy * 3.0 - py * 2.5),
            ],
            st,
        );
    };
    let diamond = |x: f32, y: f32, r: f32, c: Color32| {
        p.add(Shape::convex_polygon(vec![at(x, y - r), at(x + r, y), at(x, y + r), at(x - r, y)], c, Stroke::NONE));
    };
    let plus = |x: f32, y: f32, r: f32, st: Stroke| {
        line(&[(x - r, y), (x + r, y)], st);
        line(&[(x, y - r), (x, y + r)], st);
    };
    let person = |x: f32, y: f32| {
        circle(x, y - 4.0, 3.0, s);
        arc(x, y + 6.0, 6.0, std::f32::consts::PI, 2.0 * std::f32::consts::PI, s);
    };
    let page = || {
        line(&[(6.0, 3.0), (15.0, 3.0), (19.0, 7.0), (19.0, 21.0), (6.0, 21.0), (6.0, 3.0)], s);
        line(&[(15.0, 3.0), (15.0, 7.0), (19.0, 7.0)], s);
    };
    // Arco con punta de flecha al final, en la dirección del recorrido.
    let arc_arrow = |cx: f32, cy: f32, r: f32, a0: f32, a1: f32, st: Stroke| {
        arc(cx, cy, r, a0, a1, st);
        let dir = if a1 >= a0 { 1.0 } else { -1.0 };
        let (dx, dy) = (-a1.sin() * dir, a1.cos() * dir);
        arrow_head(cx + r * a1.cos(), cy + r * a1.sin(), dx, dy, st);
    };
    // Polígono cerrado solo con trazo.
    let outline = |pts: &[(f32, f32)], st: Stroke| {
        let mut v: Vec<(f32, f32)> = pts.to_vec();
        if let Some(first) = pts.first() {
            v.push(*first);
        }
        line(&v, st);
    };
    // Polígono en estrella (cóncavo pero visible desde el centro) relleno como abanico.
    let star_fill = |cx: f32, cy: f32, pts: &[(f32, f32)], c: Color32| {
        for i in 0..pts.len() {
            if let (Some(a), Some(b)) = (pts.get(i), pts.get((i + 1) % pts.len())) {
                p.add(Shape::convex_polygon(vec![at(cx, cy), at(a.0, a.1), at(b.0, b.1)], c, Stroke::NONE));
            }
        }
    };
    let star_points = |cx: f32, cy: f32, n: usize, r_out: f32, r_in: f32| -> Vec<(f32, f32)> {
        (0..n * 2)
            .map(|i| {
                let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / n as f32;
                let r = if i.is_multiple_of(2) { r_out } else { r_in };
                (cx + r * a.cos(), cy + r * a.sin())
            })
            .collect()
    };
    let cloud = |st: Stroke| {
        use std::f32::consts::PI;
        let pts: Vec<(f32, f32)> = (0..=10)
            .map(|i| PI * 0.5 + PI * i as f32 / 10.0)
            .map(|a| (7.5 + 4.0 * a.cos(), 13.0 + 4.0 * a.sin()))
            .chain((0..=12).map(|i| PI + PI * i as f32 / 12.0).map(|a| (13.0 + 6.0 * a.cos(), 12.0 + 6.0 * a.sin())))
            .chain(
                (0..=8).map(|i| -PI * 0.5 + PI * i as f32 / 8.0).map(|a| (18.0 + 3.5 * a.cos(), 13.5 + 3.5 * a.sin())),
            )
            .chain(std::iter::once((7.5, 17.0)))
            .collect();
        line(&pts, st);
    };
    // Cápsula (eslabón) centrada en (cx, cy), con media longitud `h`, radio `r` y ángulo `ang`.
    let capsule = |cx: f32, cy: f32, h: f32, r: f32, ang: f32, st: Stroke| {
        use std::f32::consts::PI;
        let (ca, sa_) = (ang.cos(), ang.sin());
        let mut pts = Vec::with_capacity(26);
        for (end, from) in [(h, -PI / 2.0), (-h, PI / 2.0)] {
            for i in 0..=12 {
                let a = from + PI * i as f32 / 12.0;
                let (lx, ly) = (end + r * a.cos(), r * a.sin());
                pts.push((cx + lx * ca - ly * sa_, cy + lx * sa_ + ly * ca));
            }
        }
        if let Some(f) = pts.first().copied() {
            pts.push(f);
        }
        line(&pts, st);
    };
    use std::f32::consts::PI;
    match icon {
        Icon::Gantt => {
            bar(3.0, 4.0, 9.0, 4.0, accent);
            bar(8.0, 10.0, 10.0, 4.0, accent);
            bar(13.0, 16.0, 8.0, 4.0, accent);
            line(&[(12.0, 6.0), (14.0, 6.0), (14.0, 10.0)], s);
        }
        Icon::Sheet => {
            frame(3.0, 4.0, 18.0, 16.0);
            line(&[(3.0, 9.0), (21.0, 9.0)], s);
            line(&[(3.0, 14.5), (21.0, 14.5)], s);
            line(&[(9.0, 4.0), (9.0, 20.0)], s);
        }
        Icon::TaskUsage => {
            frame(3.0, 4.0, 18.0, 16.0);
            line(&[(9.0, 4.0), (9.0, 20.0)], s);
            bar(11.0, 7.0, 8.0, 3.0, accent);
            bar(11.0, 14.0, 5.0, 3.0, accent);
        }
        Icon::ResourceUsage => {
            person(7.0, 12.0);
            frame(13.0, 5.0, 8.0, 14.0);
            line(&[(13.0, 12.0), (21.0, 12.0)], s);
        }
        Icon::Board => {
            for x in [3.0, 9.5, 16.0] {
                frame(x, 4.0, 5.0, 16.0);
            }
            bar(4.0, 6.0, 3.0, 3.0, accent);
            bar(4.0, 11.0, 3.0, 3.0, accent);
            bar(10.5, 6.0, 3.0, 3.0, accent);
            bar(17.0, 6.0, 3.0, 3.0, accent);
        }
        Icon::Timeline => {
            frame(2.0, 11.0, 20.0, 7.0);
            bar(4.0, 13.0, 8.0, 3.0, accent);
            diamond(17.0, 6.0, 3.0, ink);
            line(&[(17.0, 9.0), (17.0, 11.0)], s);
        }
        Icon::Indent | Icon::Outdent => {
            for y in [5.0, 10.0, 15.0, 20.0] {
                let x0 = if y > 7.0 && y < 18.0 { 11.0 } else { 3.0 };
                line(&[(x0, y), (21.0, y)], s);
            }
            if icon == Icon::Indent {
                line(&[(3.0, 12.5), (8.0, 12.5)], sa);
                arrow_head(8.0, 12.5, 1.0, 0.0, sa);
            } else {
                line(&[(8.0, 12.5), (3.0, 12.5)], sa);
                arrow_head(3.0, 12.5, -1.0, 0.0, sa);
            }
        }
        Icon::Link | Icon::Unlink => {
            bar(2.0, 5.0, 8.0, 4.0, accent);
            bar(14.0, 15.0, 8.0, 4.0, accent);
            line(&[(10.0, 7.0), (12.0, 7.0), (12.0, 17.0), (14.0, 17.0)], s);
            if icon == Icon::Unlink {
                line(&[(8.0, 15.0), (16.0, 9.0)], Stroke::new(2.0 * k, RED));
            }
        }
        Icon::Milestone => diamond(12.0, 12.0, 8.0, ink),
        Icon::InsertTask => {
            bar(3.0, 10.0, 12.0, 5.0, accent);
            plus(18.5, 7.0, 3.5, s);
        }
        Icon::Delete => {
            line(&[(5.0, 7.0), (19.0, 7.0)], s);
            line(&[(9.0, 7.0), (9.0, 4.0), (15.0, 4.0), (15.0, 7.0)], s);
            line(&[(6.5, 7.0), (7.5, 21.0), (16.5, 21.0), (17.5, 7.0)], s);
            line(&[(10.0, 10.0), (10.0, 18.0)], s);
            line(&[(14.0, 10.0), (14.0, 18.0)], s);
        }
        Icon::Info => {
            circle(12.0, 12.0, 9.0, s);
            p.circle_filled(at(12.0, 7.5), 1.3 * k, ink);
            line(&[(12.0, 10.5), (12.0, 17.0)], Stroke::new(2.0 * k, ink));
        }
        Icon::Notes => {
            page();
            for y in [10.0, 13.5, 17.0] {
                line(&[(9.0, y), (16.0, y)], s);
            }
        }
        Icon::Split => {
            bar(2.0, 10.0, 7.0, 5.0, accent);
            bar(15.0, 10.0, 7.0, 5.0, accent);
            for x in [10.5, 12.5] {
                line(&[(x, 12.5), (x + 1.0, 12.5)], s);
            }
        }
        Icon::Unsplit => {
            bar(4.0, 10.0, 16.0, 5.0, accent);
            arrow_head(10.0, 6.0, 1.0, 0.0, s);
            arrow_head(14.0, 6.0, -1.0, 0.0, s);
        }
        Icon::Manual => {
            p.add(Shape::convex_polygon(
                vec![at(9.0, 3.0), at(15.0, 3.0), at(14.0, 10.0), at(10.0, 10.0)],
                accent,
                Stroke::NONE,
            ));
            line(&[(6.0, 11.0), (18.0, 11.0)], s);
            line(&[(12.0, 11.0), (12.0, 21.0)], s);
        }
        Icon::Undo | Icon::Redo => {
            let flip = if icon == Icon::Undo { 1.0 } else { -1.0 };
            let x = |v: f32| 12.0 + (v - 12.0) * flip;
            let pts: Vec<(f32, f32)> = (0..=14)
                .map(|i| PI * 1.05 + PI * 0.95 * i as f32 / 14.0)
                .map(|a| (x(13.0 + 7.0 * a.cos()), 14.0 + 6.0 * a.sin()))
                .collect();
            line(&pts, s);
            let (hx, hy) = pts.first().copied().unwrap_or((6.0, 14.0));
            arrow_head(hx, hy + 0.5, 0.0, 1.0, s);
        }
        Icon::Calculate => {
            arc(12.0, 12.0, 8.0, -PI * 0.15, PI * 1.2, s);
            arrow_head(19.6, 9.5, 0.3, -1.0, s);
            arc(12.0, 12.0, 3.0, 0.0, 2.0 * PI, sa);
        }
        Icon::Level | Icon::ClearLevel => {
            bar(3.0, 5.0, 8.0, 4.0, accent);
            bar(11.0, 11.0, 8.0, 4.0, accent);
            line(&[(3.0, 19.0), (21.0, 19.0)], s);
            line(&[(7.0, 10.0), (7.0, 16.0)], s);
            arrow_head(7.0, 16.0, 0.0, 1.0, s);
            if icon == Icon::ClearLevel {
                line(&[(14.0, 3.0), (21.0, 9.0)], Stroke::new(2.0 * k, RED));
            }
        }
        Icon::AddResource | Icon::RemoveResource => {
            person(9.0, 12.0);
            line(&[(15.0, 7.0), (21.0, 7.0)], sa);
            if icon == Icon::AddResource {
                line(&[(18.0, 4.0), (18.0, 10.0)], sa);
            }
        }
        Icon::ProjectInfo => {
            page();
            p.circle_filled(at(12.5, 10.0), 1.2 * k, accent);
            line(&[(12.5, 12.5), (12.5, 18.0)], Stroke::new(2.0 * k, accent));
        }
        Icon::StatusDate | Icon::Calendar => {
            frame(3.0, 5.0, 18.0, 16.0);
            line(&[(3.0, 9.5), (21.0, 9.5)], s);
            line(&[(8.0, 3.0), (8.0, 7.0)], s);
            line(&[(16.0, 3.0), (16.0, 7.0)], s);
            if icon == Icon::StatusDate {
                for y in [11.0, 15.0, 19.0] {
                    line(&[(13.0, y), (13.0, y + 2.0)], sa);
                }
            } else {
                bar(6.0, 12.0, 3.0, 3.0, accent);
            }
        }
        Icon::Cloud | Icon::CloudUp => {
            let pts: Vec<(f32, f32)> = (0..=10)
                .map(|i| PI * 0.5 + PI * i as f32 / 10.0)
                .map(|a| (7.5 + 4.0 * a.cos(), 13.0 + 4.0 * a.sin()))
                .chain(
                    (0..=12).map(|i| PI + PI * i as f32 / 12.0).map(|a| (13.0 + 6.0 * a.cos(), 12.0 + 6.0 * a.sin())),
                )
                .chain(
                    (0..=8)
                        .map(|i| -PI * 0.5 + PI * i as f32 / 8.0)
                        .map(|a| (18.0 + 3.5 * a.cos(), 13.5 + 3.5 * a.sin())),
                )
                .chain(std::iter::once((7.5, 17.0)))
                .collect();
            line(&pts, s);
            if icon == Icon::CloudUp {
                line(&[(13.0, 21.0), (13.0, 12.0)], sa);
                arrow_head(13.0, 12.0, 0.0, -1.0, sa);
            }
        }
        Icon::Clock => {
            circle(12.0, 12.0, 9.0, s);
            line(&[(12.0, 6.5), (12.0, 12.0), (16.0, 14.5)], sa);
        }
        Icon::ZoomIn | Icon::ZoomOut => {
            circle(10.0, 10.0, 6.5, s);
            line(&[(15.0, 15.0), (21.0, 21.0)], Stroke::new(2.4 * k, ink));
            line(&[(7.0, 10.0), (13.0, 10.0)], sa);
            if icon == Icon::ZoomIn {
                line(&[(10.0, 7.0), (10.0, 13.0)], sa);
            }
        }
        Icon::Fit => {
            frame(3.0, 6.0, 18.0, 12.0);
            line(&[(6.0, 12.0), (18.0, 12.0)], sa);
            arrow_head(6.0, 12.0, -1.0, 0.0, sa);
            arrow_head(18.0, 12.0, 1.0, 0.0, sa);
        }
        Icon::Percent(pc) => {
            let c = at(12.0, 12.0);
            let r = 8.0 * k;
            if pc > 0 {
                let end = -PI / 2.0 + 2.0 * PI * f32::from(pc.min(100)) / 100.0;
                let n = (f32::from(pc) / 5.0).ceil().max(2.0) as usize;
                let mut pts = vec![c];
                pts.extend((0..=n).map(|i| {
                    let a = -PI / 2.0 + (end + PI / 2.0) * i as f32 / n as f32;
                    pos2(c.x + r * a.cos(), c.y + r * a.sin())
                }));
                // Un abanico como varios triángulos (convex_polygon pide forma convexa).
                for w in pts.windows(2).skip(1) {
                    if let [a, b] = w {
                        p.add(Shape::convex_polygon(vec![c, *a, *b], accent, Stroke::NONE));
                    }
                }
            }
            p.circle_stroke(c, r, s);
        }
        Icon::Outline => {
            line(&[(4.0, 5.0), (12.0, 5.0)], s);
            line(&[(6.0, 5.0), (6.0, 17.0)], s);
            line(&[(6.0, 11.0), (10.0, 11.0)], s);
            line(&[(6.0, 17.0), (10.0, 17.0)], s);
            bar(11.0, 9.5, 9.0, 3.0, accent);
            bar(11.0, 15.5, 7.0, 3.0, accent);
        }
        Icon::Critical => {
            bar(3.0, 5.0, 9.0, 4.0, RED);
            bar(11.0, 15.0, 10.0, 4.0, RED);
            line(&[(12.0, 7.0), (14.0, 7.0), (14.0, 15.0)], s);
        }
        Icon::New => {
            page();
            plus(12.5, 14.0, 3.5, sa);
        }
        Icon::Open => {
            line(&[(3.0, 6.0), (9.0, 6.0), (11.0, 8.0), (20.0, 8.0), (20.0, 19.0), (3.0, 19.0), (3.0, 6.0)], s);
            line(&[(3.0, 19.0), (6.0, 11.0), (22.0, 11.0), (20.0, 19.0)], sa);
        }
        Icon::Save | Icon::SaveAs => {
            line(&[(4.0, 4.0), (17.0, 4.0), (20.0, 7.0), (20.0, 20.0), (4.0, 20.0), (4.0, 4.0)], s);
            frame(8.0, 4.0, 8.0, 5.0);
            bar(7.0, 13.0, 10.0, 7.0, accent);
            if icon == Icon::SaveAs {
                line(&[(14.0, 21.0), (21.0, 14.0)], Stroke::new(2.2 * k, ink));
            }
        }
        Icon::Export => {
            line(&[(10.0, 5.0), (4.0, 5.0), (4.0, 20.0), (19.0, 20.0), (19.0, 14.0)], s);
            line(&[(10.0, 14.0), (20.0, 4.0)], sa);
            line(&[(14.0, 4.0), (20.0, 4.0), (20.0, 10.0)], sa);
        }
        Icon::Baseline => {
            bar(3.0, 6.0, 14.0, 5.0, accent);
            bar(3.0, 13.0, 14.0, 4.0, Color32::from_gray(150));
            line(&[(20.0, 4.0), (20.0, 20.0)], s);
        }
        Icon::Tracking => {
            bar(3.0, 4.0, 10.0, 4.0, accent);
            bar(3.0, 9.0, 9.0, 3.0, Color32::from_gray(150));
            bar(10.0, 14.0, 11.0, 4.0, accent);
            bar(9.0, 19.0, 10.0, 3.0, Color32::from_gray(150));
        }
        Icon::ResourceGraph => {
            line(&[(3.0, 3.0), (3.0, 21.0), (21.0, 21.0)], s);
            bar(5.0, 13.0, 3.5, 8.0, accent);
            bar(10.0, 8.0, 3.5, 13.0, accent);
            bar(10.0, 5.0, 3.5, 3.0, RED);
            bar(15.0, 11.0, 3.5, 10.0, accent);
            line(&[(4.0, 9.0), (21.0, 9.0)], s);
        }
        Icon::Network => {
            frame(2.0, 4.0, 7.0, 5.0);
            frame(15.0, 3.0, 7.0, 5.0);
            frame(15.0, 15.0, 7.0, 5.0);
            line(&[(9.0, 6.5), (12.0, 6.5), (12.0, 5.5), (15.0, 5.5)], sa);
            line(&[(12.0, 6.5), (12.0, 17.5), (15.0, 17.5)], sa);
        }
        Icon::Print => {
            frame(6.0, 3.0, 12.0, 6.0);
            line(&[(4.0, 16.0), (2.5, 16.0), (2.5, 9.0), (21.5, 9.0), (21.5, 16.0), (20.0, 16.0)], s);
            frame(6.0, 13.0, 12.0, 8.0);
            line(&[(9.0, 16.0), (15.0, 16.0)], sa);
            line(&[(9.0, 18.5), (13.0, 18.5)], sa);
        }
        Icon::Back => {
            circle(12.0, 12.0, 9.0, s);
            line(&[(17.0, 12.0), (7.5, 12.0)], s);
            arrow_head(7.5, 12.0, -1.0, 0.0, s);
        }
        Icon::Rename => {
            line(&[(3.0, 20.0), (21.0, 20.0)], s);
            line(&[(6.0, 17.0), (6.5, 13.5), (16.0, 4.0), (19.5, 7.5), (10.0, 17.0), (6.0, 17.0)], sa);
            line(&[(14.0, 6.0), (17.5, 9.5)], sa);
        }
        Icon::Sprint => {
            // Una vuelta (ciclo) con una tarjeta al centro.
            arc(12.0, 12.0, 8.5, -PI * 0.35, PI * 1.45, s);
            arrow_head(10.7, 3.6, 1.0, -0.15, s);
            bar(8.5, 9.5, 7.0, 5.0, accent);
        }
        Icon::Flag => {
            line(&[(6.0, 3.0), (6.0, 21.0)], s);
            p.add(Shape::convex_polygon(
                vec![at(6.5, 4.0), at(18.0, 4.0), at(15.0, 8.5), at(18.0, 13.0), at(6.5, 13.0)],
                accent,
                Stroke::NONE,
            ));
        }
        Icon::Pin => {
            circle(12.0, 8.5, 5.0, sa);
            p.circle_filled(at(12.0, 8.5), 1.6 * k, accent);
            line(&[(12.0, 13.5), (12.0, 21.0)], s);
            line(&[(8.0, 21.0), (16.0, 21.0)], s);
        }
        Icon::Priority => {
            for (x, h) in [(4.0, 6.0), (10.0, 11.0), (16.0, 16.0)] {
                bar(x, 20.0 - h, 4.0, h, if h > 15.0 { accent } else { ink.gamma_multiply(0.55) });
            }
        }
        Icon::Status => {
            circle(12.0, 12.0, 9.0, s);
            line(&[(7.5, 12.5), (10.5, 15.5), (16.5, 8.5)], Stroke::new(2.2 * k, accent));
        }
        Icon::StartDate | Icon::FinishDate => {
            // La barra y una marca vertical en su orilla: la de comienzo o la de fin.
            let x = if icon == Icon::StartDate { 4.0 } else { 20.0 };
            bar(5.0, 9.5, 14.0, 5.0, accent);
            line(&[(x, 4.0), (x, 20.0)], Stroke::new(2.2 * k, ink));
            p.circle_filled(at(x, 4.0), 1.8 * k, ink);
        }
        Icon::Assign => {
            person(7.0, 11.0);
            bar(13.0, 14.0, 9.0, 4.0, accent);
            line(&[(13.0, 8.0), (19.0, 8.0)], s);
            arrow_head(19.0, 8.0, 1.0, 0.0, s);
        }
        Icon::Contour | Icon::ClearContour => {
            line(&[(3.0, 21.0), (21.0, 21.0)], s);
            for (x, h) in [(4.0, 5.0), (8.0, 10.0), (12.0, 14.0), (16.0, 8.0)] {
                bar(x, 20.0 - h, 3.0, h, accent);
            }
            if icon == Icon::ClearContour {
                line(&[(4.0, 4.0), (20.0, 18.0)], Stroke::new(2.0 * k, RED));
            }
        }
        Icon::Sliders => {
            for (y, x) in [(6.0, 15.0), (12.0, 8.0), (18.0, 13.0)] {
                line(&[(3.0, y), (21.0, y)], s);
                p.circle_filled(at(x, y), 2.4 * k, accent);
            }
        }
        Icon::Bolt => {
            p.add(Shape::convex_polygon(vec![at(14.0, 2.0), at(6.0, 13.0), at(12.0, 13.0)], accent, Stroke::NONE));
            p.add(Shape::convex_polygon(vec![at(12.0, 11.0), at(18.0, 11.0), at(10.0, 22.0)], accent, Stroke::NONE));
        }
        Icon::List => {
            for y in [6.0, 12.0, 18.0] {
                p.circle_filled(at(5.0, y), 1.5 * k, accent);
                line(&[(9.0, y), (21.0, y)], s);
            }
        }
        Icon::ClearBaseline => {
            bar(3.0, 6.0, 14.0, 5.0, accent);
            bar(3.0, 13.0, 14.0, 4.0, Color32::from_gray(150));
            line(&[(13.0, 11.0), (21.0, 19.0)], Stroke::new(2.0 * k, RED));
            line(&[(21.0, 11.0), (13.0, 19.0)], Stroke::new(2.0 * k, RED));
        }
        Icon::Person => {
            person(12.0, 12.0);
        }
        Icon::Actual => {
            circle(11.0, 12.0, 8.0, s);
            line(&[(11.0, 7.0), (11.0, 12.0), (14.5, 14.0)], s);
            p.circle_filled(at(18.5, 18.5), 4.0 * k, accent);
            line(&[(16.7, 18.6), (18.1, 20.0), (20.6, 17.2)], Stroke::new(1.4 * k, Color32::WHITE));
        }
        Icon::Attachment => {
            // Un clip: dos óvalos alargados, uno dentro del otro.
            let clip = |x0: f32, x1: f32, y0: f32, y1: f32, st: Stroke| {
                let r = (x1 - x0) / 2.0;
                let cx = (x0 + x1) / 2.0;
                arc(cx, y0 + r, r, std::f32::consts::PI, 2.0 * std::f32::consts::PI, st);
                arc(cx, y1 - r, r, 0.0, std::f32::consts::PI, st);
                line(&[(x0, y0 + r), (x0, y1 - r)], st);
                line(&[(x1, y0 + r), (x1, y1 - r)], st);
            };
            clip(7.0, 17.0, 2.0, 22.0, s);
            clip(10.0, 14.0, 6.0, 17.0, sa);
        }
        Icon::Comment => {
            // Globo de diálogo con dos renglones.
            line(
                &[
                    (3.0, 4.0),
                    (21.0, 4.0),
                    (21.0, 16.0),
                    (11.0, 16.0),
                    (6.0, 21.0),
                    (6.0, 16.0),
                    (3.0, 16.0),
                    (3.0, 4.0),
                ],
                s,
            );
            line(&[(7.0, 8.5), (17.0, 8.5)], sa);
            line(&[(7.0, 12.0), (14.0, 12.0)], sa);
        }
        Icon::Money => {
            // Un billete: marco, círculo al centro y dos marcas en las esquinas.
            frame(2.0, 6.0, 20.0, 12.0);
            circle(12.0, 12.0, 3.2, sa);
            p.circle_filled(at(5.5, 9.5), 1.0 * k, ink);
            p.circle_filled(at(18.5, 14.5), 1.0 * k, ink);
        }
        Icon::MoveUp | Icon::MoveDown => {
            let (y0, y1, dy) = if icon == Icon::MoveUp { (19.0, 5.0, -1.0) } else { (5.0, 19.0, 1.0) };
            line(&[(12.0, y0), (12.0, y1)], Stroke::new(2.0 * k, ink));
            arrow_head(12.0, y1, 0.0, dy, Stroke::new(2.0 * k, ink));
        }
        // --- Gestor personal ---
        Icon::Mail => {
            p.rect_stroke(
                Rect::from_min_size(at(3.0, 5.5), vec2(18.0 * k, 13.0 * k)),
                2.0 * k,
                s,
                egui::StrokeKind::Middle,
            );
            line(&[(3.8, 6.8), (12.0, 13.0), (20.2, 6.8)], s);
        }
        Icon::Inbox => {
            outline(&[(3.0, 13.0), (5.5, 5.0), (18.5, 5.0), (21.0, 13.0), (21.0, 19.0), (3.0, 19.0)], s);
            line(&[(3.0, 13.0), (8.0, 13.0), (9.5, 16.0), (14.5, 16.0), (16.0, 13.0), (21.0, 13.0)], sa);
        }
        Icon::Send => {
            outline(&[(3.0, 10.5), (21.0, 3.0), (13.5, 21.0), (11.0, 13.0)], s);
            line(&[(11.0, 13.0), (21.0, 3.0)], s);
        }
        Icon::Drafts => {
            line(&[(15.0, 21.0), (4.0, 21.0), (4.0, 3.0), (16.0, 3.0), (16.0, 7.0)], s);
            line(&[(7.0, 8.0), (13.0, 8.0)], s);
            line(&[(7.0, 11.5), (11.0, 11.5)], s);
            outline(&[(19.0, 9.0), (21.0, 11.0), (14.0, 18.0), (11.5, 18.5), (12.0, 16.0)], sa);
        }
        Icon::Archive => {
            frame(3.0, 4.0, 18.0, 5.0);
            line(&[(4.5, 9.0), (4.5, 20.0), (19.5, 20.0), (19.5, 9.0)], s);
            line(&[(10.0, 13.0), (14.0, 13.0)], sa);
        }
        Icon::Trash => {
            line(&[(4.0, 6.5), (20.0, 6.5)], s);
            line(&[(9.0, 6.5), (9.5, 3.5), (14.5, 3.5), (15.0, 6.5)], s);
            line(&[(6.0, 6.5), (7.0, 20.5), (17.0, 20.5), (18.0, 6.5)], s);
            line(&[(10.0, 10.5), (10.0, 16.5)], s);
            line(&[(14.0, 10.5), (14.0, 16.5)], s);
        }
        Icon::Junk => {
            circle(12.0, 12.0, 8.5, s);
            line(&[(6.0, 6.0), (18.0, 18.0)], sa);
        }
        Icon::Star => outline(&star_points(12.0, 12.8, 5, 9.0, 4.0), s),
        Icon::StarFilled => {
            let pts = star_points(12.0, 12.8, 5, 9.0, 4.0);
            star_fill(12.0, 12.8, &pts, accent);
            outline(&pts, Stroke::new(s.width, accent));
        }
        Icon::Reply | Icon::ReplyAll | Icon::Forward => {
            // Flecha que sale a la izquierda (o a la derecha, reenviar) y baja en curva.
            let f = if icon == Icon::Forward { -1.0 } else { 1.0 };
            let x = |v: f32| 12.0 + (v - 12.0) * f;
            let x0 = if icon == Icon::ReplyAll { 8.0 } else { 4.0 };
            let curve: Vec<(f32, f32)> = std::iter::once((x(x0), 10.0))
                .chain(
                    (0..=10)
                        .map(|i| -PI / 2.0 + PI / 2.0 * i as f32 / 10.0)
                        .map(|a| (x(14.0 + 6.0 * a.cos()), 16.0 + 6.0 * a.sin())),
                )
                .chain(std::iter::once((x(20.0), 19.0)))
                .collect();
            line(&curve, s);
            line(&[(x(x0 + 5.0), 5.0), (x(x0), 10.0), (x(x0 + 5.0), 15.0)], s);
            if icon == Icon::ReplyAll {
                line(&[(x(8.0), 5.0), (x(3.0), 10.0), (x(8.0), 15.0)], sa);
            }
        }
        Icon::Contacts => {
            frame(5.5, 3.0, 15.0, 18.0);
            circle(13.0, 9.5, 2.8, s);
            arc(13.0, 17.5, 4.5, PI, 2.0 * PI, s);
            for y in [7.0, 12.0, 17.0] {
                line(&[(3.5, y), (7.0, y)], sa);
            }
        }
        Icon::Tasks => {
            frame(3.0, 4.0, 6.5, 6.5);
            line(&[(4.5, 7.3), (5.9, 8.7), (8.2, 5.8)], sa);
            line(&[(12.5, 7.25), (21.0, 7.25)], s);
            frame(3.0, 13.5, 6.5, 6.5);
            line(&[(12.5, 16.75), (21.0, 16.75)], s);
        }
        Icon::Check => line(&[(5.0, 12.5), (10.0, 17.5), (19.0, 7.0)], Stroke::new(2.0 * k, ink)),
        Icon::Search => {
            circle(10.5, 10.5, 6.5, s);
            line(&[(15.3, 15.3), (20.5, 20.5)], Stroke::new(2.0 * k, ink));
        }
        Icon::Settings => {
            // Engrane de 8 dientes con un círculo al centro.
            let mut pts = Vec::with_capacity(33);
            for i in 0..8 {
                let a = PI / 4.0 * i as f32;
                for (da, r) in [(-0.36, 6.6), (-0.2, 9.0), (0.2, 9.0), (0.36, 6.6)] {
                    pts.push((12.0 + r * (a + da).cos(), 12.0 + r * (a + da).sin()));
                }
            }
            outline(&pts, s);
            circle(12.0, 12.0, 2.8, sa);
        }
        Icon::Plus => plus(12.0, 12.0, 7.5, s),
        Icon::MoreHorizontal => {
            for x in [5.5, 12.0, 18.5] {
                p.circle_filled(at(x, 12.0), 1.7 * k, ink);
            }
        }
        Icon::ChevronLeft => line(&[(15.0, 5.0), (8.0, 12.0), (15.0, 19.0)], s),
        Icon::ChevronRight => line(&[(9.0, 5.0), (16.0, 12.0), (9.0, 19.0)], s),
        Icon::ChevronUp => line(&[(5.0, 15.0), (12.0, 8.0), (19.0, 15.0)], s),
        Icon::ChevronDown => line(&[(5.0, 9.0), (12.0, 16.0), (19.0, 9.0)], s),
        Icon::Sparkle => {
            // Destello del asistente: una estrella de cuatro puntas grande y una pequeña.
            let big = [
                (10.0, 3.0),
                (11.8, 10.2),
                (19.0, 12.0),
                (11.8, 13.8),
                (10.0, 21.0),
                (8.2, 13.8),
                (1.0, 12.0),
                (8.2, 10.2),
            ];
            star_fill(10.0, 12.0, &big, accent);
            let small = [
                (18.5, 2.5),
                (19.3, 4.7),
                (21.5, 5.5),
                (19.3, 6.3),
                (18.5, 8.5),
                (17.7, 6.3),
                (15.5, 5.5),
                (17.7, 4.7),
            ];
            star_fill(18.5, 5.5, &small, accent);
        }
        Icon::Sync => {
            arc_arrow(12.0, 12.0, 7.5, PI * 1.08, PI * 1.86, s);
            arc_arrow(12.0, 12.0, 7.5, PI * 0.08, PI * 0.86, s);
        }
        Icon::Offline => {
            cloud(s);
            line(&[(4.0, 4.0), (20.0, 20.0)], Stroke::new(1.8 * k, ink));
        }
        Icon::Bell => {
            let mut pts = vec![(4.5, 17.0), (6.5, 14.5), (6.5, 10.5)];
            pts.extend(
                (0..=12).map(|i| PI + PI * i as f32 / 12.0).map(|a| (12.0 + 5.5 * a.cos(), 10.5 + 5.5 * a.sin())),
            );
            pts.extend([(17.5, 14.5), (19.5, 17.0)]);
            outline(&pts, s);
            arc(12.0, 18.5, 2.2, 0.0, PI, sa);
        }
        Icon::Tag => {
            outline(&[(3.0, 3.0), (11.5, 3.0), (21.0, 12.5), (12.5, 21.0), (3.0, 11.5)], s);
            circle(7.8, 7.8, 1.6, sa);
        }
        Icon::Folder => {
            outline(
                &[(3.0, 19.0), (3.0, 6.5), (4.0, 5.0), (9.0, 5.0), (11.0, 7.5), (20.0, 7.5), (21.0, 8.5), (21.0, 19.0)],
                s,
            );
            line(&[(3.0, 10.5), (21.0, 10.5)], s);
        }
        Icon::Hyperlink => {
            capsule(8.2, 15.8, 2.4, 3.2, -PI / 4.0, s);
            capsule(15.8, 8.2, 2.4, 3.2, -PI / 4.0, sa);
        }
        Icon::Phone => {
            let mut pts = vec![(18.0, 20.5)];
            pts.extend(
                (0..=14)
                    .map(|i| PI / 2.0 + PI / 2.0 * i as f32 / 14.0)
                    .map(|a| (18.0 + 14.5 * a.cos(), 6.0 + 14.5 * a.sin())),
            );
            pts.extend([
                (6.0, 3.5),
                (9.0, 3.5),
                (10.5, 8.0),
                (8.5, 10.0),
                (14.0, 15.5),
                (16.0, 13.5),
                (20.5, 15.0),
                (20.5, 18.0),
            ]);
            outline(&pts, s);
        }
        Icon::Video => {
            p.rect_stroke(
                Rect::from_min_size(at(2.5, 6.0), vec2(13.0 * k, 12.0 * k)),
                2.0 * k,
                s,
                egui::StrokeKind::Middle,
            );
            outline(&[(15.5, 10.5), (21.5, 7.0), (21.5, 17.0), (15.5, 13.5)], s);
        }
        Icon::Close => {
            line(&[(6.0, 6.0), (18.0, 18.0)], s);
            line(&[(18.0, 6.0), (6.0, 18.0)], s);
        }
        Icon::Filter => outline(&[(3.0, 4.5), (21.0, 4.5), (14.0, 12.5), (14.0, 19.5), (10.0, 21.0), (10.0, 12.5)], s),
        Icon::Lock => {
            frame(5.0, 11.0, 14.0, 10.0);
            line(&[(8.0, 11.0), (8.0, 8.0)], s);
            arc(12.0, 8.0, 4.0, PI, 2.0 * PI, s);
            line(&[(16.0, 8.0), (16.0, 11.0)], s);
            p.circle_filled(at(12.0, 15.5), 1.4 * k, accent);
        }
        Icon::Menu => {
            for y in [6.5, 12.0, 17.5] {
                line(&[(4.0, y), (20.0, y)], s);
            }
        }
        Icon::Warning => {
            outline(&[(12.0, 3.0), (21.5, 20.0), (2.5, 20.0)], s);
            line(&[(12.0, 9.0), (12.0, 14.0)], Stroke::new(2.0 * k, accent));
            p.circle_filled(at(12.0, 17.0), 1.2 * k, accent);
        }
        Icon::Alert => {
            circle(12.0, 12.0, 9.0, s);
            line(&[(12.0, 7.0), (12.0, 13.0)], Stroke::new(2.0 * k, accent));
            p.circle_filled(at(12.0, 16.5), 1.2 * k, accent);
        }
    }
}

const RED: Color32 = crate::tokens::LIGHT.danger;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_and_round_trip() {
        let mut seen = std::collections::HashSet::new();
        for (n, i) in Icon::ALL {
            assert!(seen.insert(*n), "nombre repetido: {n}");
            assert_eq!(Icon::named(n), Some(*i));
            assert_eq!(i.name(), *n);
        }
        assert_eq!(Icon::Percent(10).name(), "percent");
    }

    #[test]
    fn every_icon_paints_without_panicking() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            for (_, i) in Icon::ALL {
                for side in [0.0, 1.0, 16.0, 48.0] {
                    paint(
                        ui.painter(),
                        Rect::from_min_size(pos2(0.0, 0.0), vec2(side, side)),
                        *i,
                        Color32::BLACK,
                        Color32::BLUE,
                    );
                }
            }
            for pc in [0, 1, 50, 100, 255] {
                paint(
                    ui.painter(),
                    Rect::from_min_size(pos2(0.0, 0.0), vec2(24.0, 24.0)),
                    Icon::Percent(pc),
                    Color32::BLACK,
                    Color32::BLUE,
                );
            }
        });
        out.textures_delta.clear();
    }
}
