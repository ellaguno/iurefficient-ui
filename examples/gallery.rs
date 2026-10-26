//! Galería del kit: una maqueta de IureDesk (riel, carpetas, lista virtual de 200 000 mensajes,
//! lectura) con todos los controles, más una hoja con todos los iconos.
//!
//! Captura sin pantalla (wgpu fuera de pantalla):
//! `cargo run --example gallery -- /tmp/gallery.png [--dark] [--hc] [--density dense|compact|comfortable]
//!  [--palette[=texto]] [--icons] [--lang en|es]`
//!
//! Sin ruta abre una ventana interactiva: `cargo run --example gallery`.

use egui::{Align, Layout, RichText, Ui, vec2};
use egui_kittest::Harness;
use iurefficient_ui::list::{Column, ListState, ListView, RowStyle};
use iurefficient_ui::palette::{CommandPalette, PaletteItem};
use iurefficient_ui::widgets::nav_rail::rail_width;
use iurefficient_ui::{
    BadgeKind, Banner, Density, EmptyState, Icon, NavItem, PaneKind, SearchField, Severity, ThemeMode, Toasts, avatar,
    badge, icons, keycap, nav_rail, pick, primary_button, section_label, segmented, sidebar_item, split, system, theme,
};

const ROWS: usize = 200_000;

const SENDERS: [(&str, &str); 10] = [
    ("Ana López", "ana.lopez@acme.mx"),
    ("Juan Pérez", "juan@perez.com"),
    ("Equipo Iurefficient", "hola@iurefficient.com"),
    ("María Fernanda Ruiz", "mfruiz@despacho.legal"),
    ("Carlos Méndez", "carlos.mendez@cliente.mx"),
    ("Notificaciones", "no-reply@banco.mx"),
    ("Sofía Torres", "sofia@torres.dev"),
    ("Luis Hernández", "lhernandez@proveedor.com"),
    ("Paula Gómez", "paula.gomez@acme.mx"),
    ("Roberto Sánchez", "rsanchez@notaria12.mx"),
];

const SUBJECTS: [&str; 10] = [
    "Propuesta Q4 — revisión final antes del viernes",
    "Factura #391 — adjunto el XML y el PDF",
    "Minuta de la junta del lunes",
    "Re: Contrato de arrendamiento, cláusula 7",
    "Tu estado de cuenta de octubre está listo",
    "Invitación: Revisión de avance (jue 10:00)",
    "Fwd: Expediente 1452/2026 — acuerdo publicado",
    "Cotización de licencias para 25 usuarios",
    "¿Podemos mover la llamada a las 5?",
    "Recordatorio: firma de escritura el martes",
];

/// Un mensaje de ejemplo (se genera del índice, no se guarda).
struct Mail {
    from: (&'static str, &'static str),
    subject: &'static str,
    date: String,
    unread: bool,
    flagged: bool,
    attachment: bool,
}

fn mail(i: usize) -> Mail {
    let h = i.wrapping_mul(2_654_435_761) >> 7;
    let date = match i {
        0..=5 => format!("{:02}:{:02}", 10 - i.min(9), (i * 17) % 60),
        6..=20 => pick("Yesterday", "Ayer").to_string(),
        _ => format!("{:02}/{:02}/{}", 1 + (i / 30) % 28, 1 + (i / 900) % 12, 2026 - (i / 10_800) as i32),
    };
    Mail {
        from: SENDERS[h % SENDERS.len()],
        subject: SUBJECTS[(h / 10) % SUBJECTS.len()],
        date,
        unread: i.is_multiple_of(4) || i < 3,
        flagged: i % 17 == 3,
        attachment: i % 5 == 1,
    }
}

struct Gallery {
    started: bool,
    mode: ThemeMode,
    density: Density,
    module: String,
    folder: usize,
    search: String,
    columns: Vec<Column>,
    list: ListState,
    reversed: bool,
    palette: CommandPalette,
    items: Vec<PaletteItem>,
    toasts: Toasts,
    view: usize,
    icons_only: bool,
    show_warning: bool,
}

impl Gallery {
    fn new(mode: ThemeMode, density: Density) -> Self {
        let items = vec![
            PaletteItem::new("mail.new", pick("New message", "Nuevo mensaje"))
                .icon(Icon::Mail)
                .shortcut("Ctrl+N")
                .hint(pick("Mail", "Correo"))
                .keywords("new message compose"),
            PaletteItem::new("mail.reply", pick("Reply", "Responder"))
                .icon(Icon::Reply)
                .shortcut("Ctrl+R")
                .hint(pick("Mail", "Correo"))
                .keywords("reply"),
            PaletteItem::new("mail.reply_all", pick("Reply all", "Responder a todos"))
                .icon(Icon::ReplyAll)
                .shortcut("Ctrl+Shift+R")
                .hint(pick("Mail", "Correo"))
                .keywords("reply all"),
            PaletteItem::new("mail.forward", pick("Forward", "Reenviar"))
                .icon(Icon::Forward)
                .shortcut("Ctrl+F")
                .hint(pick("Mail", "Correo"))
                .keywords("forward"),
            PaletteItem::new("mail.archive", pick("Archive", "Archivar"))
                .icon(Icon::Archive)
                .shortcut("E")
                .hint(pick("Mail", "Correo"))
                .keywords("archive"),
            PaletteItem::new("mail.flag", pick("Flag for follow-up", "Marcar para seguimiento"))
                .icon(Icon::Flag)
                .shortcut("Ins")
                .hint(pick("Mail", "Correo")),
            PaletteItem::new("cal.new", pick("New event", "Nueva cita"))
                .icon(Icon::Calendar)
                .hint(pick("Calendar", "Calendario"))
                .keywords("event"),
            PaletteItem::new("task.new", pick("New task", "Nueva tarea"))
                .icon(Icon::Tasks)
                .hint(pick("Tasks", "Tareas"))
                .keywords("task"),
            PaletteItem::new("ai.summarize", pick("Summarize thread", "Resumir conversación"))
                .icon(Icon::Sparkle)
                .hint(pick("Assistant", "Asistente")),
            PaletteItem::new("sync.now", pick("Sync now", "Sincronizar ahora")).icon(Icon::Sync).shortcut("F9"),
            PaletteItem::new("view.dense", pick("Density: dense", "Densidad: densa"))
                .icon(Icon::List)
                .hint(pick("View", "Vista")),
            PaletteItem::new("settings", pick("Settings", "Ajustes")).icon(Icon::Settings).shortcut("Ctrl+,"),
            PaletteItem::new("offline", pick("Work offline", "Trabajar sin conexión"))
                .icon(Icon::Offline)
                .enabled(false),
        ];
        let mut toasts = Toasts::default();
        toasts.push(
            Severity::Success,
            pick("Message archived", "Mensaje archivado"),
            Some(pick("Undo", "Deshacer")),
            30.0,
        );
        let mut list = ListState::default();
        list.selection.click(1, egui::Modifiers::NONE, &|i| i as u64);
        list.selection.click(4, egui::Modifiers::COMMAND, &|i| i as u64);
        Gallery {
            started: false,
            mode,
            density,
            module: "mail".into(),
            folder: 0,
            search: String::new(),
            columns: vec![
                Column::new("", 28.0).fixed().not_sortable(),
                Column::new(pick("From", "De"), 180.0).min_width(80.0),
                Column::new(pick("Subject", "Asunto"), 300.0).grow().min_width(120.0),
                Column::new("", 24.0).fixed().not_sortable(),
                Column::new(pick("Received", "Recibido"), 92.0).right().min_width(60.0),
            ],
            list,
            reversed: false,
            palette: CommandPalette::default(),
            items,
            toasts,
            view: 1,
            icons_only: false,
            show_warning: true,
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        if !self.started {
            self.started = true;
            system::watch_system_theme(&ctx);
            system::install_fonts(&ctx, true);
            theme::apply(&ctx, self.mode, self.density);
        }
        theme::refresh(&ctx);
        self.palette.handle_shortcut(&ctx);
        if self.icons_only {
            split::center(ui, PaneKind::Detail, icon_sheet);
            return;
        }
        let items = [
            NavItem::new("mail", Icon::Mail, pick("Mail", "Correo")).badge(12).shortcut("Ctrl+1"),
            NavItem::new("calendar", Icon::Calendar, pick("Calendar", "Calendario")).shortcut("Ctrl+2"),
            NavItem::new("contacts", Icon::Contacts, pick("Contacts", "Contactos")).shortcut("Ctrl+3"),
            NavItem::new("notes", Icon::Notes, pick("Notes", "Notas")).shortcut("Ctrl+4"),
            NavItem::new("tasks", Icon::Tasks, pick("Tasks", "Tareas")).badge(3).shortcut("Ctrl+5"),
            NavItem::new("assistant", Icon::Sparkle, pick("Assistant", "Asistente")),
            NavItem::new("settings", Icon::Settings, pick("Settings", "Ajustes")).bottom(),
            NavItem::new("sync", Icon::Sync, pick("Synced 2 min ago", "Sincronizado hace 2 min")).bottom(),
        ];
        let w = rail_width(&ctx);
        split::left(ui, "rail", PaneKind::Rail, w, w..=w, |ui| {
            if let Some(id) = nav_rail(ui, &items, &self.module) {
                self.module = id.to_string();
            }
        });
        split::left(ui, "folders", PaneKind::Sidebar, 232.0, 180.0..=360.0, |ui| self.folders(ui));
        split::top_bar(ui, "toolbar", |ui| self.toolbar(ui));
        split::right(ui, "detail", PaneKind::Detail, 500.0, 340.0..=800.0, |ui| self.detail(ui));
        split::center(ui, PaneKind::Content, |ui| self.message_list(ui));
        self.toasts.show(&ctx);
        if let Some(id) = self.palette.show(&ctx, &self.items) {
            self.toasts.push(Severity::Info, format!("{}: {id}", pick("Command", "Comando")), None, 4.0);
        }
    }

    fn folders(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        SearchField::new(&mut self.search)
            .hint(pick("Search mail", "Buscar en el correo"))
            .shortcut(egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::F))
            .show(ui);
        ui.add_space(10.0);
        section_label(ui, pick("Favorites", "Favoritos"));
        let folders = [
            (Icon::Inbox, pick("Inbox", "Entrada"), 12),
            (Icon::Star, pick("Starred", "Destacados"), 0),
            (Icon::Send, pick("Sent", "Enviados"), 0),
            (Icon::Drafts, pick("Drafts", "Borradores"), 2),
            (Icon::Archive, pick("Archive", "Archivo"), 0),
            (Icon::Junk, pick("Junk", "Correo no deseado"), 5),
            (Icon::Trash, pick("Trash", "Papelera"), 0),
        ];
        ui.spacing_mut().item_spacing.y = 1.0;
        for (k, (icon, label, n)) in folders.into_iter().enumerate() {
            if sidebar_item(ui, Some(icon), label, n, k == self.folder).clicked() {
                self.folder = k;
            }
        }
        ui.add_space(10.0);
        section_label(ui, pick("Accounts", "Cuentas"));
        for (k, label) in ["ana@gmail.com", "ana@iurefficient.com"].into_iter().enumerate() {
            if sidebar_item(ui, Some(Icon::Folder), label, 0, self.folder == 10 + k).clicked() {
                self.folder = 10 + k;
            }
        }
        ui.add_space(10.0);
        section_label(ui, pick("Today", "Hoy"));
        let c = theme::colors(ui);
        for (time, what) in [
            ("10:00", pick("Progress review", "Revisión de avance")),
            ("13:30", pick("Lunch with ACME", "Comida con ACME")),
        ] {
            ui.horizontal(|ui| {
                ui.label(RichText::new(time).color(c.accent).strong());
                ui.label(RichText::new(what).color(c.text));
            });
        }
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(14.0, 14.0), egui::Sense::hover());
            icons::paint(ui.painter(), r, Icon::Check, c.success, c.success);
            ui.label(RichText::new(pick("3 tasks due today", "3 tareas para hoy")).color(c.text_muted));
        });
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for (icon, en, es) in [
                (Icon::Reply, "Reply", "Responder"),
                (Icon::ReplyAll, "Reply all", "Responder a todos"),
                (Icon::Forward, "Forward", "Reenviar"),
                (Icon::Archive, "Archive", "Archivar"),
                (Icon::Trash, "Delete", "Eliminar"),
                (Icon::Flag, "Flag", "Marcar"),
            ] {
                icons::icon_button(ui, icon, pick(en, es), true);
            }
            icons::icon_button(ui, Icon::MoreHorizontal, pick("More actions", "Más acciones"), true);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let before = self.density;
                segmented(
                    ui,
                    &mut self.density,
                    &[
                        (Density::Comfortable, Density::Comfortable.label()),
                        (Density::Compact, Density::Compact.label()),
                        (Density::Dense, Density::Dense.label()),
                    ],
                );
                if before != self.density {
                    theme::set_density(ui.ctx(), self.density);
                }
                let before = self.mode;
                segmented(
                    ui,
                    &mut self.mode,
                    &[
                        (ThemeMode::System, ThemeMode::System.label()),
                        (ThemeMode::Light, ThemeMode::Light.label()),
                        (ThemeMode::Dark, ThemeMode::Dark.label()),
                    ],
                );
                if before != self.mode {
                    theme::apply(ui.ctx(), self.mode, self.density);
                }
                if ui.button(pick("Commands", "Comandos")).on_hover_text("Ctrl+K").clicked() {
                    self.palette.open();
                }
                primary_button(ui, pick("New message", "Nuevo mensaje"));
            });
        });
    }

    fn message_list(&mut self, ui: &mut Ui) {
        let rev = self.reversed;
        let idx = move |i: usize| if rev { ROWS - 1 - i } else { i };
        let out =
            ListView::new("inbox", &mut self.columns, ROWS).label(pick("Inbox messages", "Mensajes de Entrada")).show(
                ui,
                &mut self.list,
                |i| idx(i) as u64,
                |row| {
                    let m = mail(idx(row.index));
                    row.style(RowStyle { unread: m.unread, flagged: m.flagged, muted: false });
                    let c = *row.colors();
                    if m.flagged {
                        row.icon_colored(0, Icon::Flag, c.text, c.warning);
                        row.describe(pick("flagged", "marcado"));
                    }
                    if m.unread {
                        row.describe(pick("unread", "no leído"));
                    }
                    row.text(1, m.from.0);
                    row.text(2, m.subject);
                    if m.attachment {
                        row.icon_colored(3, Icon::Attachment, c.text_muted, c.text_muted);
                    }
                    row.text_muted(4, &m.date);
                },
            );
        if let Some((col, _)) = out.sort
            && col == 4
        {
            self.reversed = !self.reversed;
        }
    }

    fn detail(&mut self, ui: &mut Ui) {
        let c = theme::colors(ui);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.label(RichText::new(SUBJECTS[0]).size(20.0).strong().color(c.text));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                avatar(ui, SENDERS[0].0, SENDERS[0].1, 36.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(SENDERS[0].0).strong().color(c.text));
                    ui.label(RichText::new(format!("{} · {}", SENDERS[0].1, pick("today 10:42", "hoy 10:42"))).color(c.text_muted));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    badge(ui, pick("Lead", "Lead"), BadgeKind::Neutral);
                    badge(ui, "ACME", BadgeKind::Accent);
                });
            });
            ui.add_space(10.0);
            Banner::new(
                Severity::Info,
                pick("Linked to lead ACME · $120k in Iurefficient.", "Vinculado al lead ACME · $120k en Iurefficient."),
            )
            .action(pick("Open", "Abrir"))
            .show(ui);
            if self.show_warning {
                ui.add_space(6.0);
                let r = Banner::new(
                    Severity::Warning,
                    pick("Couldn't connect to Gmail — check your internet.", "No se pudo conectar con Gmail — revisa tu internet."),
                )
                .action(pick("Retry", "Reintentar"))
                .dismissible(true)
                .show(ui);
                if r.dismissed {
                    self.show_warning = false;
                }
            }
            ui.add_space(12.0);
            ui.label(RichText::new(pick(
                "Hi team, attached is the final version of the Q4 proposal. Please review the pricing section and the delivery dates before Friday.",
                "Hola equipo, adjunto la versión final de la propuesta Q4. Revisen por favor la sección de precios y las fechas de entrega antes del viernes.",
            )).color(c.text));
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            section_label(ui, pick("Components", "Componentes"));
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                badge(ui, "12", BadgeKind::Accent);
                badge(ui, pick("Draft", "Borrador"), BadgeKind::Neutral);
                badge(ui, pick("Urgent", "Urgente"), BadgeKind::Danger);
                badge(ui, pick("Paid", "Pagado"), BadgeKind::Success);
                badge(ui, pick("Pending", "Pendiente"), BadgeKind::Warning);
                ui.add_space(8.0);
                keycap(ui, "Ctrl+K");
                keycap(ui, "Ctrl+Shift+R");
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                for (n, e) in SENDERS.iter().take(7) {
                    avatar(ui, n, e, 28.0);
                }
                ui.add_space(8.0);
                segmented(
                    ui,
                    &mut self.view,
                    &[(0, pick("Day", "Día")), (1, pick("Week", "Semana")), (2, pick("Month", "Mes"))],
                );
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                primary_button(ui, pick("Send", "Enviar"));
                let _ = ui.button(pick("Cancel", "Cancelar"));
                for (icon, en, es) in [(Icon::Star, "Star", "Destacar"), (Icon::Tag, "Label", "Etiquetar"), (Icon::Bell, "Remind me", "Recordarme"), (Icon::Pin, "Pin", "Fijar"), (Icon::Hyperlink, "Copy link", "Copiar enlace"), (Icon::Phone, "Call", "Llamar"), (Icon::Video, "Video call", "Videollamada")] {
                    icons::icon_button(ui, icon, pick(en, es), true);
                }
            });
            ui.add_space(8.0);
            Banner::new(Severity::Error, pick("The message could not be sent.", "No se pudo enviar el mensaje."))
                .title(pick("Outbox", "Bandeja de salida"))
                .action(pick("Details", "Detalles"))
                .show(ui);
            ui.add_space(6.0);
            Banner::new(Severity::Success, pick("All accounts are up to date.", "Todas las cuentas están al día.")).show(ui);
            ui.add_space(8.0);
            egui::Frame::new()
                .stroke(egui::Stroke::new(1.0, c.separator))
                .corner_radius(8)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(200.0);
                    EmptyState::new(Icon::Tasks, pick("Nothing due today", "Nada pendiente para hoy"))
                        .body(pick("Tasks you add or flag will show up here.", "Las tareas que agregues o marques aparecerán aquí."))
                        .action(pick("New task", "Nueva tarea"))
                        .show(ui);
                });
        });
    }
}

/// Todos los iconos con su nombre, en claro y con el acento.
fn icon_sheet(ui: &mut Ui) {
    let c = theme::colors(ui);
    ui.label(
        RichText::new(format!("{} ({})", pick("Icons", "Iconos"), Icon::ALL.len())).size(18.0).strong().color(c.text),
    );
    ui.add_space(8.0);
    let cols = 12;
    egui::Grid::new("icons").spacing(vec2(6.0, 10.0)).show(ui, |ui| {
        for (k, (name, icon)) in Icon::ALL.iter().enumerate() {
            ui.vertical_centered(|ui| {
                ui.set_width(100.0);
                let (r, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::hover());
                icons::paint(ui.painter(), r, *icon, c.text, c.accent);
                iurefficient_ui::a11y::label(&resp, name);
                ui.label(RichText::new(*name).size(11.0).color(c.text_muted));
            });
            if (k + 1).is_multiple_of(cols) {
                ui.end_row();
            }
        }
    });
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = args
        .iter()
        .find(|a| !a.starts_with("--") && !matches!(a.as_str(), "dense" | "compact" | "comfortable" | "en" | "es"))
        .cloned();
    let flag_value = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
            .or_else(|| args.iter().find_map(|a| a.strip_prefix(&format!("{name}="))).map(str::to_string))
    };
    let density = flag_value("--density").and_then(|d| Density::from_key(&d)).unwrap_or(Density::Comfortable);
    if let Some(l) = flag_value("--lang") {
        iurefficient_ui::lang::set_lang(iurefficient_ui::lang::from_tag(&l));
    }
    let mode = if args.iter().any(|a| a == "--hc") {
        ThemeMode::HighContrast
    } else if args.iter().any(|a| a == "--dark") {
        ThemeMode::Dark
    } else if out.is_some() {
        ThemeMode::Light
    } else {
        ThemeMode::System
    };
    let mut g = Gallery::new(mode, density);
    g.icons_only = args.iter().any(|a| a == "--icons");
    if let Some(q) = args.iter().find_map(|a| a.strip_prefix("--palette")) {
        g.palette.open_with(q.strip_prefix('=').unwrap_or(""));
    }

    let Some(out) = out else {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 900.0]).with_title("Iurefficient UI"),
            ..Default::default()
        };
        return eframe::run_native("iurefficient-ui-gallery", options, Box::new(|_| Ok(Box::new(App(g)))))
            .map_err(|e| e.to_string());
    };

    let size = if g.icons_only { vec2(1300.0, 1080.0) } else { vec2(1440.0, 900.0) };
    // El arnés deja 8 px de margen alrededor; la galería ocupa la ventana completa, como en eframe.
    let ppp = flag_value("--ppp").and_then(|p| p.parse::<f32>().ok()).unwrap_or(1.0).clamp(0.5, 4.0);
    let mut harness = Harness::builder().with_size(size).with_pixels_per_point(ppp).build_ui_state(
        |ui, g: &mut Gallery| {
            let full = ui.ctx().content_rect();
            ui.scope_builder(egui::UiBuilder::new().max_rect(full), |ui| {
                ui.set_clip_rect(full);
                g.ui(ui);
            });
        },
        g,
    );
    harness.run();
    // Un cuadro más para que las fuentes nuevas y el tema queden aplicados.
    harness.run_steps(2);
    let image = harness.render()?;
    image.save(&out).map_err(|e| e.to_string())?;
    println!("{out}");
    Ok(())
}

struct App(Gallery);

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
}
