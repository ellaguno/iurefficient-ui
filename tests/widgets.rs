//! Pruebas de interacción (egui_kittest) de la paleta de comandos y de la lista virtual, y
//! capturas de referencia en `tests/snapshots/` (regenerar con `UPDATE_SNAPSHOTS=1 cargo test`).
//! Se usan las fuentes de egui (no las del sistema) para que las capturas sean iguales en todas
//! las máquinas.

use std::time::Instant;

use egui::{Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use iurefficient_ui::list::{Column, ListState, ListView, RowStyle, SortDir};
use iurefficient_ui::palette::{CommandPalette, PaletteItem};
use iurefficient_ui::{Density, Icon, ThemeMode, lang, theme};

/// Las capturas usan wgpu; dos dispositivos a la vez fallan en algunos controladores, así que las
/// pruebas que renderizan se turnan.
static GPU: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn setup(ctx: &egui::Context, mode: ThemeMode, density: Density) {
    lang::set_lang(lang::Lang::Es);
    theme::apply(ctx, mode, density);
}

// ---------------------------------------------------------------------------------------------
// Paleta

struct PaletteApp {
    started: bool,
    palette: CommandPalette,
    items: Vec<PaletteItem>,
    chosen: Vec<String>,
}

fn palette_app() -> PaletteApp {
    PaletteApp {
        started: false,
        palette: CommandPalette::default(),
        items: vec![
            PaletteItem::new("mail.new", "Nuevo mensaje")
                .icon(Icon::Mail)
                .shortcut("Ctrl+N")
                .hint("Correo")
                .keywords("new message"),
            PaletteItem::new("mail.reply", "Responder")
                .icon(Icon::Reply)
                .shortcut("Ctrl+R")
                .hint("Correo")
                .keywords("reply"),
            PaletteItem::new("mail.reply_all", "Responder a todos")
                .icon(Icon::ReplyAll)
                .shortcut("Ctrl+Shift+R")
                .hint("Correo"),
            PaletteItem::new("mail.archive", "Archivar").icon(Icon::Archive).shortcut("E").hint("Correo"),
            PaletteItem::new("offline", "Trabajar sin conexión").icon(Icon::Offline).enabled(false),
        ],
        chosen: Vec::new(),
    }
}

fn palette_harness(mode: ThemeMode) -> Harness<'static, PaletteApp> {
    Harness::builder().with_size(vec2(720.0, 420.0)).build_ui_state(
        move |ui, app: &mut PaletteApp| {
            if !app.started {
                app.started = true;
                setup(ui.ctx(), mode, Density::Comfortable);
            }
            ui.label("Fondo de la aplicación");
            app.palette.handle_shortcut(ui.ctx());
            if let Some(id) = app.palette.show(ui.ctx(), &app.items) {
                app.chosen.push(id);
            }
        },
        palette_app(),
    )
}

#[test]
fn palette_opens_with_ctrl_k_filters_and_runs_with_enter() {
    let mut h = palette_harness(ThemeMode::Light);
    h.run();
    assert!(!h.state().palette.is_open());
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    h.run();
    assert!(h.state().palette.is_open());
    // El campo tiene el foco: se escribe directo.
    h.event(egui::Event::Text("resp".into()));
    h.run();
    assert_eq!(h.state().palette.query(), "resp");
    // Los dos «Responder» y nada más.
    assert!(h.query_by_label("Responder, Correo").is_some());
    assert!(h.query_by_label("Responder a todos, Correo").is_some());
    assert!(h.query_by_label("Archivar, Correo").is_none());
    h.key_press(Key::ArrowDown);
    h.run();
    assert_eq!(h.state().palette.cursor(), 1);
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().chosen, vec!["mail.reply_all".to_string()]);
    assert!(!h.state().palette.is_open(), "se cierra al elegir");
}

#[test]
fn palette_click_escape_and_disabled_items() {
    let mut h = palette_harness(ThemeMode::Light);
    h.state_mut().palette.open();
    h.run();
    h.get_by_label("Archivar, Correo").click();
    h.run();
    assert_eq!(h.state().chosen, vec!["mail.archive".to_string()]);
    // Los deshabilitados no se eligen, ni con clic ni con Enter.
    h.state_mut().palette.open_with("sin conex");
    h.run();
    h.get_by_label("Trabajar sin conexión").click();
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().chosen.len(), 1);
    assert!(h.state().palette.is_open());
    h.key_press(Key::Escape);
    h.run();
    assert!(!h.state().palette.is_open());
}

#[test]
fn palette_snapshots() {
    let _gpu = GPU.lock().unwrap_or_else(|e| e.into_inner());
    let mut results = egui_kittest::SnapshotResults::new();
    for (name, mode) in [("palette_light", ThemeMode::Light), ("palette_dark", ThemeMode::Dark)] {
        let mut h = palette_harness(mode);
        h.state_mut().palette.open_with("r");
        h.run();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}

// ---------------------------------------------------------------------------------------------
// Lista virtual

const ROWS: usize = 200_000;

struct ListApp {
    started: bool,
    density: Density,
    state: ListState,
    columns: Vec<Column>,
    /// Filas pintadas en el último cuadro y la mayor vista.
    painted: usize,
    max_painted: usize,
    last_sort: Option<(usize, SortDir)>,
    activated: Option<usize>,
}

fn list_app(density: Density) -> ListApp {
    ListApp {
        started: false,
        density,
        state: ListState::default(),
        columns: vec![
            Column::new("", 24.0).fixed().not_sortable(),
            Column::new("De", 160.0),
            Column::new("Asunto", 200.0).grow(),
            Column::new("Fecha", 80.0).right(),
        ],
        painted: 0,
        max_painted: 0,
        last_sort: None,
        activated: None,
    }
}

fn list_harness(density: Density, mode: ThemeMode) -> Harness<'static, ListApp> {
    Harness::builder().with_size(vec2(640.0, 400.0)).build_ui_state(
        move |ui, app: &mut ListApp| {
            if !app.started {
                app.started = true;
                setup(ui.ctx(), mode, app.density);
            }
            let mut painted = 0;
            let out = ListView::new("prueba", &mut app.columns, ROWS).label("Mensajes").show(
                ui,
                &mut app.state,
                |i| i as u64 + 1_000,
                |row| {
                    painted += 1;
                    row.style(RowStyle {
                        unread: row.index.is_multiple_of(3),
                        flagged: row.index.is_multiple_of(7),
                        muted: false,
                    });
                    if row.index.is_multiple_of(7) {
                        row.icon(0, Icon::Flag);
                    }
                    row.text(1, &format!("Fila {}", row.index));
                    row.text(2, &format!("Asunto {}", row.index));
                    row.text_muted(3, "10:42");
                },
            );
            app.painted = painted;
            app.max_painted = app.max_painted.max(painted);
            if out.sort.is_some() {
                app.last_sort = out.sort;
            }
            if out.activated.is_some() {
                app.activated = out.activated;
            }
        },
        list_app(density),
    )
}

fn selected(h: &Harness<'_, ListApp>) -> Vec<u64> {
    h.state().state.selection.keys().map(|k| k - 1_000).collect()
}

#[test]
fn list_of_200k_rows_only_paints_visible_rows_and_is_fast() {
    let mut h = list_harness(Density::Dense, ThemeMode::Light);
    h.run();
    // 400 px de alto con filas de 22 px: unas 17 filas, nunca las 200 000.
    assert!(h.state().painted > 10 && h.state().painted < 40, "pintadas: {}", h.state().painted);
    h.get_by_label("Fila 2, Asunto 2, 10:42").click();
    h.run();
    assert_eq!(selected(&h), vec![2]);
    // 60 cuadros bajando de página en página: el tiempo no depende del número de filas.
    let t = Instant::now();
    for _ in 0..60 {
        h.key_press(Key::PageDown);
        h.step();
    }
    let elapsed = t.elapsed();
    h.run();
    assert!(h.state().max_painted < 40, "máximo pintadas en un cuadro: {}", h.state().max_painted);
    assert!(elapsed.as_secs_f32() < 5.0, "60 cuadros tardaron {elapsed:?}");
    // Fin: la última fila queda seleccionada y visible.
    h.key_press(Key::End);
    h.run();
    assert_eq!(selected(&h), vec![ROWS as u64 - 1]);
    assert!(
        h.query_by_label(&format!("Fila {}, Asunto {}, 10:42", ROWS - 1, ROWS - 1)).is_some(),
        "se desplazó al final"
    );
    // Ctrl+A con 200 000 filas.
    let t = Instant::now();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    assert_eq!(h.state().state.selection.len(), ROWS);
    assert!(t.elapsed().as_secs_f32() < 5.0, "seleccionar todo tardó {:?}", t.elapsed());
    // Inicio.
    h.key_press(Key::Home);
    h.run();
    assert_eq!(selected(&h), vec![0]);
}

#[test]
fn list_mouse_and_keyboard_selection() {
    let mut h = list_harness(Density::Compact, ThemeMode::Light);
    h.run();
    h.get_by_label("Fila 1, Asunto 1, 10:42").click();
    h.run();
    h.get_by_label("Fila 4, Asunto 4, 10:42").click_modifiers(Modifiers::SHIFT);
    h.run();
    assert_eq!(selected(&h), vec![1, 2, 3, 4]);
    h.get_by_label("Fila 2, Asunto 2, 10:42").click_modifiers(Modifiers::COMMAND);
    h.run();
    assert_eq!(selected(&h), vec![1, 3, 4]);
    // Teclado: el cursor está en 2; Mayús+↓ extiende desde el ancla (2).
    h.key_press_modifiers(Modifiers::SHIFT, Key::ArrowDown);
    h.run();
    assert_eq!(selected(&h), vec![2, 3]);
    h.key_press(Key::ArrowUp);
    h.run();
    assert_eq!(selected(&h), vec![2]);
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().activated, Some(2));
    // Ordenar: clic en el encabezado alterna la dirección.
    h.get_by_label("Asunto").click();
    h.run();
    assert_eq!(h.state().last_sort, Some((2, SortDir::Ascending)));
    h.get_by_label("Asunto, orden ascendente").click();
    h.run();
    assert_eq!(h.state().last_sort, Some((2, SortDir::Descending)));
    assert_eq!(h.state().state.sort, Some((2, SortDir::Descending)));
}

#[test]
fn list_snapshots() {
    let _gpu = GPU.lock().unwrap_or_else(|e| e.into_inner());
    let mut results = egui_kittest::SnapshotResults::new();
    for (name, density, mode) in [
        ("list_dense_light", Density::Dense, ThemeMode::Light),
        ("list_comfortable_dark", Density::Comfortable, ThemeMode::Dark),
    ] {
        let mut h = list_harness(density, mode);
        h.run();
        h.get_by_label("Fila 1, Asunto 1, 10:42").click();
        h.run();
        h.get_by_label("Fila 3, Asunto 3, 10:42").click_modifiers(Modifiers::SHIFT);
        h.run();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
