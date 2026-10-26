//! Paleta de comandos (`Ctrl+K`, `⌘K` en Mac): se escribe y aparece lo que coincide (búsqueda
//! difusa, sin importar acentos ni mayúsculas). Flechas para moverse, Enter para elegir, Esc o un
//! clic fuera para cerrar. No sabe nada de la app: recibe elementos `{id, label, hint, shortcut,
//! icon}` y devuelve el `id` elegido.
//!
//! ```no_run
//! # fn f(ctx: &egui::Context, pal: &mut iurefficient_ui::palette::CommandPalette) {
//! use iurefficient_ui::{Icon, palette::PaletteItem};
//! let items = [
//!     PaletteItem::new("mail.new", "New message").icon(Icon::Mail).shortcut("Ctrl+N"),
//!     PaletteItem::new("mail.archive", "Archive").icon(Icon::Archive).hint("Mail"),
//! ];
//! pal.handle_shortcut(ctx);
//! if let Some(id) = pal.show(ctx, &items) {
//!     // despachar el comando `id`
//! }
//! # }
//! ```

use egui::{Align2, Context, Key, KeyboardShortcut, Modifiers, Rect, Sense, Stroke, pos2, vec2};

use crate::icons::{self, Icon};
use crate::lang::pick;
use crate::{fuzzy, theme};

/// Un elemento que se puede buscar y elegir.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaletteItem {
    /// Lo que devuelve [`CommandPalette::show`] al elegirlo (id del comando, de la carpeta…).
    pub id: String,
    /// Texto principal, en el idioma de la interfaz.
    pub label: String,
    /// Texto secundario a la derecha (dónde está: «Correo», «Carpeta»…). También se busca.
    pub hint: String,
    /// Atajo de teclado que se muestra («Ctrl+Shift+R»).
    pub shortcut: Option<String>,
    pub icon: Option<Icon>,
    /// Palabras extra que también encuentran el elemento (la etiqueta en el otro idioma, sinónimos).
    pub keywords: String,
    /// Los deshabilitados se ven, pero no se pueden elegir.
    pub enabled: bool,
}

impl PaletteItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        PaletteItem { id: id.into(), label: label.into(), enabled: true, ..Default::default() }
    }
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn keywords(mut self, keywords: impl Into<String>) -> Self {
        self.keywords = keywords.into();
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Índices de `items` que coinciden con `query`, del mejor al peor (como mucho `limit`). Con la
/// búsqueda vacía, los primeros en su orden (quien llama los ordena por uso reciente).
pub fn search(query: &str, items: &[PaletteItem], limit: usize) -> Vec<usize> {
    if query.trim().is_empty() {
        return (0..items.len().min(limit)).collect();
    }
    let mut scored: Vec<(i32, usize)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, it)| {
            let extra = format!("{} {}", it.keywords, it.hint);
            // Los deshabilitados bajan un poco, a igualdad.
            let penalty = if it.enabled { 0 } else { 6 };
            fuzzy::score_with_extra(query, &it.label, &extra).map(|s| (s - penalty, i))
        })
        .collect();
    scored.sort_by_key(|(s, i)| (std::cmp::Reverse(*s), *i));
    scored.into_iter().take(limit).map(|(_, i)| i).collect()
}

/// Estado de la paleta. Guárdalo en la app; ábrela con [`CommandPalette::open`] o el atajo.
#[derive(Clone, Debug)]
pub struct CommandPalette {
    open: bool,
    query: String,
    cursor: usize,
    focused: bool,
    scroll_to_cursor: bool,
    /// Cuántos resultados se calculan (los que no caben se ven desplazando).
    pub max_results: usize,
    /// Texto de ayuda del campo; `None` usa el de siempre.
    pub placeholder: Option<String>,
    /// Resultados de la última búsqueda: (búsqueda, número de elementos, índices).
    cache: Option<(String, usize, Vec<usize>)>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        CommandPalette {
            open: false,
            query: String::new(),
            cursor: 0,
            focused: false,
            scroll_to_cursor: false,
            max_results: 50,
            placeholder: None,
            cache: None,
        }
    }
}

/// Alto de una fila de resultado.
const ROW: f32 = 34.0;
/// Filas visibles antes de desplazar.
const VISIBLE_ROWS: f32 = 10.0;
/// Ancho de la paleta.
const WIDTH: f32 = 580.0;

impl CommandPalette {
    /// El atajo que la abre y cierra: Ctrl+K (⌘K en Mac).
    pub const SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::K);

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open_with("");
    }

    /// Abre con una búsqueda ya escrita.
    pub fn open_with(&mut self, query: &str) {
        self.open = true;
        self.query = query.to_string();
        self.cursor = 0;
        self.focused = false;
        self.cache = None;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn toggle(&mut self) {
        if self.open { self.close() } else { self.open() }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    /// Índice (en la lista de resultados) del elemento resaltado.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Abre o cierra con [`Self::SHORTCUT`]. Devuelve `true` si lo consumió.
    pub fn handle_shortcut(&mut self, ctx: &Context) -> bool {
        let hit = ctx.input_mut(|i| i.consume_shortcut(&Self::SHORTCUT));
        if hit {
            self.toggle();
        }
        hit
    }

    fn results(&mut self, items: &[PaletteItem]) -> Vec<usize> {
        if let Some((q, n, r)) = &self.cache
            && *q == self.query
            && *n == items.len()
        {
            return r.clone();
        }
        let r = search(&self.query, items, self.max_results.max(1));
        self.cache = Some((self.query.clone(), items.len(), r.clone()));
        r
    }

    /// Dibuja la paleta si está abierta y devuelve el `id` elegido (y entonces se cierra).
    pub fn show(&mut self, ctx: &Context, items: &[PaletteItem]) -> Option<String> {
        if !self.open {
            return None;
        }
        let results = self.results(items);
        let n = results.len();
        let (down, up, pgdn, pgup, enter, esc) = ctx.input_mut(|i| {
            (
                i.consume_key(Modifiers::NONE, Key::ArrowDown),
                i.consume_key(Modifiers::NONE, Key::ArrowUp),
                i.consume_key(Modifiers::NONE, Key::PageDown),
                i.consume_key(Modifiers::NONE, Key::PageUp),
                i.consume_key(Modifiers::NONE, Key::Enter),
                i.consume_key(Modifiers::NONE, Key::Escape),
            )
        });
        let last = n.saturating_sub(1);
        let page = VISIBLE_ROWS as usize - 1;
        let before = self.cursor;
        if down {
            self.cursor = if self.cursor >= last { 0 } else { self.cursor + 1 };
        }
        if up {
            self.cursor = if self.cursor == 0 { last } else { self.cursor - 1 };
        }
        if pgdn {
            self.cursor = (self.cursor + page).min(last);
        }
        if pgup {
            self.cursor = self.cursor.saturating_sub(page);
        }
        self.cursor = self.cursor.min(last);
        if self.cursor != before {
            self.scroll_to_cursor = true;
        }

        let colors = theme::ctx_colors(ctx);
        let mut chosen: Option<String> = None;
        let mut close = esc;
        let frame = egui::Frame::new()
            .fill(colors.surface)
            .stroke(Stroke::new(1.0, colors.separator))
            .corner_radius(crate::tokens::radius::WINDOW as u8)
            .shadow(ctx.global_style().visuals.window_shadow)
            .inner_margin(egui::Margin::same(6));
        let modal = egui::Modal::new(egui::Id::unique("iurefficient-ui/palette"))
            .area(
                egui::Modal::default_area(egui::Id::unique("iurefficient-ui/palette"))
                    .anchor(Align2::CENTER_TOP, vec2(0.0, 72.0)),
            )
            .backdrop_color(egui::Color32::from_black_alpha(if colors.dark { 120 } else { 64 }))
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_width(WIDTH);
                // Campo de búsqueda con la lupa.
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    let (r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                    icons::paint(ui.painter(), r, Icon::Search, colors.text_muted, colors.accent);
                    let hint = self.placeholder.clone().unwrap_or_else(|| {
                        pick("Type a command or search…", "Escribe un comando o busca…").to_string()
                    });
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.query)
                            .hint_text(hint)
                            .frame(egui::Frame::NONE)
                            .desired_width(f32::INFINITY)
                            .margin(vec2(4.0, 8.0))
                            .font(egui::FontId::proportional(16.0)),
                    );
                    if !self.focused {
                        resp.request_focus();
                        self.focused = true;
                    }
                    if resp.changed() {
                        self.cursor = 0;
                        self.scroll_to_cursor = true;
                    }
                });
                let sep = ui.cursor().min.y + 2.0;
                ui.painter().hline(ui.max_rect().x_range(), sep, Stroke::new(1.0, colors.separator));
                ui.add_space(6.0);
                if results.is_empty() {
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(pick("Nothing matches", "Nada coincide")).color(colors.text_muted),
                        );
                    });
                    ui.add_space(10.0);
                }
                egui::ScrollArea::vertical().max_height(ROW * VISIBLE_ROWS).auto_shrink([false, true]).show(ui, |ui| {
                    for (k, &idx) in results.iter().enumerate() {
                        let Some(it) = items.get(idx) else { continue };
                        let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
                        crate::a11y::row(
                            &r,
                            &if it.hint.is_empty() { it.label.clone() } else { format!("{}, {}", it.label, it.hint) },
                            k == self.cursor,
                        );
                        if k == self.cursor {
                            ui.painter().rect_filled(rect, 6.0, colors.selection);
                            if self.scroll_to_cursor {
                                ui.scroll_to_rect(rect, None);
                                self.scroll_to_cursor = false;
                            }
                        } else if r.hovered() {
                            ui.painter().rect_filled(rect, 6.0, colors.hover);
                        }
                        let ink = if it.enabled { colors.text } else { colors.text_muted };
                        if let Some(icon) = it.icon {
                            icons::paint(
                                ui.painter(),
                                Rect::from_center_size(pos2(rect.left() + 18.0, rect.center().y), vec2(18.0, 18.0)),
                                icon,
                                ink,
                                if it.enabled { colors.accent } else { ink },
                            );
                        }
                        let small = egui::FontId::proportional(12.0);
                        let mut right = rect.right() - 10.0;
                        if let Some(sc) = &it.shortcut {
                            right -= crate::widgets::keycap::paint_shortcut(
                                ui.painter(),
                                pos2(right, rect.center().y),
                                sc,
                                &small,
                                &colors,
                            ) + 10.0;
                        }
                        let hint_w = if it.hint.is_empty() {
                            0.0
                        } else {
                            let g = ui.painter().layout_no_wrap(it.hint.clone(), small.clone(), colors.text_muted);
                            let w = g.size().x.min(180.0);
                            ui.painter()
                                .with_clip_rect(Rect::from_x_y_ranges(right - w..=right, rect.y_range()))
                                .galley(pos2(right - w, rect.center().y - g.size().y / 2.0), g, colors.text_muted);
                            w + 12.0
                        };
                        let left = rect.left() + 38.0;
                        let label_w = (right - hint_w - left).max(0.0);
                        let galley =
                            crate::widgets::text::elided(ui, &it.label, egui::FontId::proportional(14.0), ink, label_w);
                        ui.painter().galley(pos2(left, rect.center().y - galley.size().y / 2.0), galley, ink);
                        if r.clicked() && it.enabled {
                            chosen = Some(it.id.clone());
                        }
                    }
                });
                // Pie con la ayuda del teclado.
                ui.add_space(4.0);
                ui.painter().hline(ui.max_rect().x_range(), ui.cursor().min.y, Stroke::new(1.0, colors.separator));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(pick(
                            "Arrows to move · Enter to run · Esc to close",
                            "Flechas para moverte · Enter para elegir · Esc para cerrar",
                        ))
                        .size(11.5)
                        .color(colors.text_muted),
                    );
                });
            });
        if modal.backdrop_response.clicked() {
            close = true;
        }
        if enter && let Some(it) = results.get(self.cursor).and_then(|&i| items.get(i)).filter(|it| it.enabled) {
            chosen = Some(it.id.clone());
        }
        if chosen.is_some() || close {
            self.open = false;
        }
        chosen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<PaletteItem> {
        vec![
            PaletteItem::new("mail.new", "Nuevo mensaje").keywords("new message compose"),
            PaletteItem::new("mail.unmark", "Unmark"),
            PaletteItem::new("mail.reply", "Responder").keywords("reply"),
            PaletteItem::new("mail.reply_all", "Responder a todos").keywords("reply all"),
            PaletteItem::new("x.disabled", "Responder después").enabled(false),
        ]
    }

    #[test]
    fn search_ranks_by_fuzzy_score() {
        let it = items();
        let ids = |q: &str| search(q, &it, 10).into_iter().map(|i| it[i].id.as_str()).collect::<Vec<_>>();
        assert_eq!(ids("nm").first(), Some(&"mail.new"));
        assert_eq!(ids("resp").first(), Some(&"mail.reply"));
        assert_eq!(ids("reply all").first(), Some(&"mail.reply_all"));
        assert_eq!(ids("compose"), vec!["mail.new"]);
        assert_eq!(ids("").len(), 5);
        assert!(ids("zzz").is_empty());
        assert_eq!(*ids("resp").last().unwrap(), "x.disabled", "los deshabilitados al final a igualdad");
    }
}
