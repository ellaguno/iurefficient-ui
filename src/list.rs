//! Lista/tabla virtual densa para cientos de miles de filas (bandeja de correo, contactos, tareas):
//! alto de fila fijo (de la densidad), columnas que se estiran y ordenan con un clic en el
//! encabezado, selección múltiple con ratón (clic, Mayús, Ctrl) y teclado (↑ ↓ Inicio Fin RePág
//! AvPág, Ctrl+A, Ctrl+Espacio, Enter), y un pintor por fila. Solo se dibujan las filas visibles:
//! el costo por cuadro no depende del número de filas.
//!
//! La selección se guarda por **clave estable** (`key_of(fila) -> u64`, p. ej. el id del mensaje),
//! así sobrevive a reordenar o a que lleguen filas nuevas; el cursor y el ancla son índices.
//!
//! ```no_run
//! # fn f(ui: &mut egui::Ui, state: &mut iurefficient_ui::list::ListState, cols: &mut [iurefficient_ui::list::Column]) {
//! use iurefficient_ui::list::{ListView, RowStyle};
//! let subjects = vec!["Hola"; 200_000];
//! let out = ListView::new("bandeja", cols, subjects.len()).show(ui, state, |i| i as u64, |row| {
//!     row.style(RowStyle { unread: row.index % 3 == 0, ..Default::default() });
//!     row.text(0, subjects[row.index]);
//! });
//! if let Some((col, dir)) = out.sort { /* reordenar */ }
//! # }
//! ```

use std::collections::BTreeSet;
use std::ops::Range;

use egui::{
    Align, Color32, CursorIcon, FontId, Id, Key, Modifiers, Painter, Rect, Response, Sense, Stroke, Ui, pos2, vec2,
};

use crate::icons::{self, Icon};
use crate::tokens::Palette;

/// Una columna. El ancho lo cambia el usuario arrastrando el borde del encabezado; guarda las
/// columnas en la app para conservarlo.
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub title: String,
    pub width: f32,
    pub min_width: f32,
    pub resizable: bool,
    pub sortable: bool,
    /// `Align::Max` alinea a la derecha (fechas, tamaños).
    pub align: Align,
    /// Se queda con el ancho sobrante (normalmente el asunto).
    pub grow: bool,
}

impl Column {
    pub fn new(title: impl Into<String>, width: f32) -> Self {
        Column {
            title: title.into(),
            width,
            min_width: 24.0,
            resizable: true,
            sortable: true,
            align: Align::Min,
            grow: false,
        }
    }
    pub fn min_width(mut self, w: f32) -> Self {
        self.min_width = w.max(0.0);
        self
    }
    /// Ancho fijo (iconos de estado).
    pub fn fixed(mut self) -> Self {
        self.resizable = false;
        self
    }
    pub fn not_sortable(mut self) -> Self {
        self.sortable = false;
        self
    }
    pub fn right(mut self) -> Self {
        self.align = Align::Max;
        self
    }
    pub fn grow(mut self) -> Self {
        self.grow = true;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SortDir {
    Ascending,
    Descending,
}

impl SortDir {
    pub fn toggled(self) -> SortDir {
        match self {
            SortDir::Ascending => SortDir::Descending,
            SortDir::Descending => SortDir::Ascending,
        }
    }
}

/// Cuántas claves puede agregar una sola selección por rango (protege de rangos absurdos).
const MAX_RANGE: usize = 2_000_000;

/// Selección múltiple: claves seleccionadas, ancla (inicio de Mayús+clic) y cursor (la fila con el
/// foco del teclado).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection {
    keys: BTreeSet<u64>,
    anchor: Option<usize>,
    cursor: Option<usize>,
}

impl Selection {
    pub fn is_selected(&self, key: u64) -> bool {
        self.keys.contains(&key)
    }
    /// Claves seleccionadas, en orden.
    pub fn keys(&self) -> impl Iterator<Item = u64> + '_ {
        self.keys.iter().copied()
    }
    pub fn len(&self) -> usize {
        self.keys.len()
    }
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }
    pub fn anchor(&self) -> Option<usize> {
        self.anchor
    }
    pub fn clear(&mut self) {
        self.keys.clear();
        self.anchor = None;
        self.cursor = None;
    }

    /// Solo esta fila (clic normal).
    pub fn select_only(&mut self, row: usize, key: u64) {
        self.keys.clear();
        self.keys.insert(key);
        self.anchor = Some(row);
        self.cursor = Some(row);
    }

    fn add_range(&mut self, a: usize, b: usize, key_of: &dyn Fn(usize) -> u64) {
        let (lo, hi) = (a.min(b), a.max(b));
        for r in (lo..=hi).take(MAX_RANGE) {
            self.keys.insert(key_of(r));
        }
    }

    /// Clic con modificadores: Ctrl/⌘ alterna la fila, Mayús selecciona desde el ancla (con Ctrl
    /// además conserva lo que había), sin nada selecciona solo esa.
    pub fn click(&mut self, row: usize, mods: Modifiers, key_of: &dyn Fn(usize) -> u64) {
        if mods.shift {
            let anchor = self.anchor.or(self.cursor).unwrap_or(row);
            if !mods.command {
                self.keys.clear();
            }
            self.add_range(anchor, row, key_of);
            self.anchor = Some(anchor);
            self.cursor = Some(row);
        } else if mods.command {
            let k = key_of(row);
            if !self.keys.remove(&k) {
                self.keys.insert(k);
            }
            self.anchor = Some(row);
            self.cursor = Some(row);
        } else {
            self.select_only(row, key_of(row));
        }
    }

    /// Mueve el cursor a `row` con el teclado: con `extend` (Mayús) selecciona del ancla a `row`;
    /// con `keep` (Ctrl) solo mueve el cursor; sin nada selecciona solo esa fila.
    pub fn move_to(&mut self, row: usize, extend: bool, keep: bool, key_of: &dyn Fn(usize) -> u64) {
        if extend {
            let anchor = self.anchor.or(self.cursor).unwrap_or(row);
            self.keys.clear();
            self.add_range(anchor, row, key_of);
            self.anchor = Some(anchor);
            self.cursor = Some(row);
        } else if keep {
            self.cursor = Some(row);
        } else {
            self.select_only(row, key_of(row));
        }
    }

    /// Alterna la fila del cursor (Ctrl+Espacio).
    pub fn toggle_cursor(&mut self, key_of: &dyn Fn(usize) -> u64) {
        if let Some(c) = self.cursor {
            let k = key_of(c);
            if !self.keys.remove(&k) {
                self.keys.insert(k);
            }
            self.anchor = Some(c);
        }
    }

    /// Todas las filas (Ctrl+A).
    pub fn select_all(&mut self, rows: usize, key_of: &dyn Fn(usize) -> u64) {
        self.keys.clear();
        if rows > 0 {
            self.add_range(0, rows - 1, key_of);
            self.anchor = Some(0);
            self.cursor = Some(self.cursor.unwrap_or(0).min(rows - 1));
        }
    }

    /// Ajusta cursor y ancla si la lista se achicó.
    pub fn clamp(&mut self, rows: usize) {
        let fix = |v: &mut Option<usize>| {
            if let Some(x) = v {
                if rows == 0 {
                    *v = None;
                } else if *x >= rows {
                    *x = rows - 1;
                }
            }
        };
        fix(&mut self.cursor);
        fix(&mut self.anchor);
    }
}

/// Estado de una lista: guárdalo en la app junto a sus columnas.
#[derive(Clone, Debug, Default)]
pub struct ListState {
    pub selection: Selection,
    /// Columna y dirección del orden actual (la flecha del encabezado).
    pub sort: Option<(usize, SortDir)>,
    scroll_to: Option<usize>,
    offset_y: f32,
    viewport_h: f32,
}

impl ListState {
    /// Desplaza lo justo para que se vea la fila `row` en el próximo cuadro.
    pub fn scroll_to(&mut self, row: usize) {
        self.scroll_to = Some(row);
    }
    /// Desplazamiento vertical actual (en puntos).
    pub fn offset(&self) -> f32 {
        self.offset_y
    }
}

/// Estilo de una fila de correo/tarea: no leído en negritas con una marca de acento, con bandera
/// tintada, o atenuado (terminado, silenciado).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RowStyle {
    pub unread: bool,
    pub flagged: bool,
    pub muted: bool,
}

/// Lo que recibe el pintor de cada fila visible.
pub struct Row<'a> {
    pub index: usize,
    pub selected: bool,
    /// Es la fila del cursor del teclado.
    pub cursor: bool,
    pub rect: Rect,
    ui: &'a Ui,
    painter: Painter,
    cells: &'a [(Rect, Align)],
    colors: &'a Palette,
    font: FontId,
    bold: FontId,
    small: FontId,
    style: RowStyle,
    label: String,
    pad: f32,
}

impl Row<'_> {
    pub fn colors(&self) -> &Palette {
        self.colors
    }
    /// Pintor recortado a la lista.
    pub fn painter(&self) -> &Painter {
        &self.painter
    }

    /// Fija el estilo de la fila; pinta ya el tinte de bandera y la marca de no leído.
    pub fn style(&mut self, s: RowStyle) {
        self.style = s;
        if s.flagged && !self.selected {
            self.painter.rect_filled(self.rect, 0.0, self.colors.flag_tint);
        }
        if s.unread {
            let r = Rect::from_min_max(
                pos2(self.rect.left() + 1.0, self.rect.top() + 3.0),
                pos2(self.rect.left() + 4.0, self.rect.bottom() - 3.0),
            );
            self.painter.rect_filled(r, 1.5, self.colors.accent);
        }
    }

    /// Rectángulo de la celda `col` sin su margen.
    pub fn cell(&self, col: usize) -> Rect {
        self.cells.get(col).map_or(Rect::NOTHING, |c| c.0)
    }

    fn ink(&self) -> Color32 {
        if self.style.muted { self.colors.text_muted } else { self.colors.text }
    }

    /// Texto normal en la celda (negritas si la fila no está leída).
    pub fn text(&mut self, col: usize, text: &str) {
        let font = if self.style.unread { self.bold.clone() } else { self.font.clone() };
        let ink = self.ink();
        self.text_with(col, text, font, ink);
    }

    /// Texto secundario (fecha, vista previa).
    pub fn text_muted(&mut self, col: usize, text: &str) {
        let font = if self.style.unread { self.font.clone() } else { self.small.clone() };
        let ink = if self.style.unread { self.colors.text } else { self.colors.text_muted };
        self.text_with(col, text, font, ink);
    }

    /// Texto con fuente y color propios, recortado con «…» al ancho de la celda.
    pub fn text_with(&mut self, col: usize, text: &str, font: FontId, color: Color32) {
        let Some(&(rect, align)) = self.cells.get(col) else { return };
        self.text_in(rect, align, text, font, color);
    }

    /// Texto en cualquier rectángulo (filas de dos líneas).
    pub fn text_in(&mut self, rect: Rect, align: Align, text: &str, font: FontId, color: Color32) {
        if rect.width() <= 1.0 {
            return;
        }
        let galley = crate::widgets::text::elided(self.ui, text, font, color, rect.width());
        let x = if align == Align::Max { rect.right() - galley.size().x } else { rect.left() };
        let y = rect.center().y - galley.size().y / 2.0;
        self.painter.with_clip_rect(rect.intersect(self.painter.clip_rect())).galley(pos2(x, y), galley, color);
        if !self.label.is_empty() {
            self.label.push_str(", ");
        }
        self.label.push_str(text);
    }

    /// Icono centrado en la celda, con los colores del texto y el acento.
    pub fn icon(&mut self, col: usize, icon: Icon) {
        let (ink, accent) = (self.ink(), self.colors.accent);
        self.icon_colored(col, icon, ink, accent);
    }

    pub fn icon_colored(&mut self, col: usize, icon: Icon, ink: Color32, accent: Color32) {
        let Some(&(rect, _)) = self.cells.get(col) else { return };
        // Los iconos usan la celda completa, sin el margen del texto.
        let rect = rect.expand2(vec2(self.pad, 0.0));
        let side = (self.rect.height() - 8.0).clamp(8.0, 18.0).min(rect.width() - 4.0);
        let r = Rect::from_center_size(rect.center(), vec2(side, side));
        icons::paint(&self.painter, r, icon, ink, accent);
    }

    /// Agrega texto a lo que lee el lector de pantalla de esta fila (lo dibujado con `text*` ya
    /// se agrega solo).
    pub fn describe(&mut self, text: &str) {
        if !self.label.is_empty() {
            self.label.push_str(", ");
        }
        self.label.push_str(text);
    }
}

/// Lo que pasó en la lista en este cuadro.
#[derive(Debug)]
pub struct ListResponse {
    /// Respuesta del cuerpo (para `context_menu`, foco…).
    pub response: Response,
    /// El usuario pidió ordenar por esta columna y dirección (ya guardado en `state.sort`).
    pub sort: Option<(usize, SortDir)>,
    pub selection_changed: bool,
    /// Doble clic o Enter sobre esta fila.
    pub activated: Option<usize>,
    /// Clic secundario sobre esta fila (ya seleccionada).
    pub context: Option<usize>,
    /// Filas dibujadas.
    pub visible: Range<usize>,
}

/// La lista. Se construye en cada cuadro.
pub struct ListView<'c> {
    id: Id,
    columns: &'c mut [Column],
    rows: usize,
    row_height: Option<f32>,
    header: bool,
    label: String,
}

impl<'c> ListView<'c> {
    pub fn new(id_salt: impl egui::AsIdSalt, columns: &'c mut [Column], rows: usize) -> Self {
        ListView {
            id: Id::unique(("iurefficient-ui/list", id_salt)),
            columns,
            rows,
            row_height: None,
            header: true,
            label: crate::lang::pick("List", "Lista").to_string(),
        }
    }
    /// Alto de fila propio (p. ej. dos líneas); por omisión, el de la densidad.
    pub fn row_height(mut self, h: f32) -> Self {
        if h.is_finite() && h >= 8.0 {
            self.row_height = Some(h);
        }
        self
    }
    pub fn header(mut self, show: bool) -> Self {
        self.header = show;
        self
    }
    /// Nombre de la lista para el lector de pantalla («Mensajes de Entrada»).
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Anchos efectivos: la columna que crece se queda con lo que sobra.
    fn widths(&self, total: f32) -> Vec<f32> {
        let mut w: Vec<f32> = self.columns.iter().map(|c| c.width.max(c.min_width)).collect();
        let fixed: f32 = self.columns.iter().zip(&w).filter(|(c, _)| !c.grow).map(|(_, w)| *w).sum();
        let growers = self.columns.iter().filter(|c| c.grow).count();
        if growers > 0 {
            let each = ((total - fixed) / growers as f32).max(0.0);
            for (c, x) in self.columns.iter().zip(w.iter_mut()) {
                if c.grow {
                    *x = each.max(c.min_width);
                }
            }
        }
        w
    }

    pub fn show(
        self,
        ui: &mut Ui,
        state: &mut ListState,
        key_of: impl Fn(usize) -> u64,
        mut paint_row: impl FnMut(&mut Row<'_>),
    ) -> ListResponse {
        let ctx = ui.ctx().clone();
        let density = crate::theme::density(&ctx);
        let colors = crate::theme::colors(ui);
        let row_h = self.row_height.unwrap_or_else(|| density.row_height());
        let pad = density.cell_padding();
        let font = crate::theme::body_font(&ctx);
        let small = crate::theme::small_font(&ctx);
        let bold = crate::theme::bold_font(&ctx, font.size);
        let rows = self.rows;
        state.selection.clamp(rows);
        let key_of: &dyn Fn(usize) -> u64 = &key_of;
        let mut sort_event = None;
        let mut selection_changed = false;
        let mut activated = None;
        let mut context = None;

        let total_w = ui.available_width();
        ui.spacing_mut().item_spacing.y = 0.0;
        let widths = self.widths(total_w);
        let left = ui.cursor().left();

        // --- Encabezado ---
        if self.header && !self.columns.is_empty() {
            let hh = row_h.max(24.0);
            let (hrect, _) = ui.allocate_exact_size(vec2(total_w, hh), Sense::hover());
            let p = ui.painter_at(hrect);
            p.rect_filled(hrect, 0.0, colors.surface_alt);
            p.hline(hrect.x_range(), hrect.bottom() - 0.5, Stroke::new(1.0, colors.separator));
            let mut x = left;
            let n = self.columns.len();
            for (i, w) in widths.iter().enumerate() {
                let Some(col) = self.columns.get(i) else { break };
                let cell = Rect::from_min_size(pos2(x, hrect.top()), vec2(*w, hh));
                let r = ui.interact(
                    cell,
                    self.id.with(("header", i)),
                    if col.sortable { Sense::click() } else { Sense::hover() },
                );
                if r.hovered() && col.sortable {
                    p.rect_filled(cell, 0.0, colors.hover);
                }
                let sorted = state.sort.filter(|(c, _)| *c == i).map(|(_, d)| d);
                let arrow_w = if sorted.is_some() { 14.0 } else { 0.0 };
                let inner = Rect::from_min_max(
                    pos2(cell.left() + pad, cell.top()),
                    pos2(cell.right() - pad - arrow_w, cell.bottom()),
                );
                let title_font = FontId::new(small.size, crate::theme::bold_family(&ctx));
                let g = crate::widgets::text::elided(ui, &col.title, title_font, colors.text_muted, inner.width());
                let tx = if col.align == Align::Max { inner.right() - g.size().x } else { inner.left() };
                p.galley(pos2(tx, inner.center().y - g.size().y / 2.0), g, colors.text_muted);
                if let Some(d) = sorted {
                    let icon = if d == SortDir::Ascending { Icon::ChevronUp } else { Icon::ChevronDown };
                    let ar = Rect::from_center_size(pos2(cell.right() - pad - 6.0, cell.center().y), vec2(12.0, 12.0));
                    icons::paint(&p, ar, icon, colors.text_muted, colors.accent);
                }
                let dir_text = match sorted {
                    Some(SortDir::Ascending) => crate::lang::pick(", sorted ascending", ", orden ascendente"),
                    Some(SortDir::Descending) => crate::lang::pick(", sorted descending", ", orden descendente"),
                    None => "",
                };
                let title = if col.title.is_empty() {
                    format!("{} {}", crate::lang::pick("Column", "Columna"), i + 1)
                } else {
                    col.title.clone()
                };
                crate::a11y::button(&r, &format!("{title}{dir_text}"));
                if col.sortable && r.clicked() {
                    let d = match state.sort {
                        Some((c, d)) if c == i => d.toggled(),
                        _ => SortDir::Ascending,
                    };
                    state.sort = Some((i, d));
                    sort_event = Some((i, d));
                }
                // Separador y asa para cambiar el ancho.
                if i + 1 < n {
                    p.vline(cell.right() - 0.5, hrect.y_range().shrink(5.0), Stroke::new(1.0, colors.separator));
                }
                if col.resizable && !col.grow {
                    let handle = Rect::from_center_size(pos2(cell.right(), cell.center().y), vec2(8.0, hh));
                    let hr = ui.interact(handle, self.id.with(("resize", i)), Sense::drag());
                    if hr.hovered() || hr.dragged() {
                        ctx.set_cursor_icon(CursorIcon::ResizeColumn);
                    }
                    if hr.dragged()
                        && let Some(c) = self.columns.get_mut(i)
                    {
                        let dx = hr.drag_delta().x;
                        c.width = (c.width.max(c.min_width) + dx).clamp(c.min_width, 4000.0);
                    }
                }
                x += w;
            }
        }

        // --- Cuerpo ---
        let body_id = self.id.with("body");
        let has_focus = ctx.memory(|m| m.has_focus(body_id));
        // Teclado (antes de dibujar, para que el cuadro ya muestre el cambio).
        if has_focus && rows > 0 {
            ctx.memory_mut(|m| {
                m.set_focus_lock_filter(
                    body_id,
                    egui::EventFilter { tab: false, horizontal_arrows: true, vertical_arrows: true, escape: false },
                )
            });
            let page = ((state.viewport_h / row_h).floor() as usize).saturating_sub(1).max(1);
            let events = ui.input(|i| i.events.clone());
            for e in events {
                let egui::Event::Key { key, pressed: true, modifiers, .. } = e else { continue };
                let cur = state.selection.cursor();
                let last = rows - 1;
                let target = match key {
                    Key::ArrowDown => Some(cur.map_or(0, |c| (c + 1).min(last))),
                    Key::ArrowUp => Some(cur.map_or(0, |c| c.saturating_sub(1))),
                    Key::Home => Some(0),
                    Key::End => Some(last),
                    Key::PageDown => Some(cur.map_or(0, |c| (c + page).min(last))),
                    Key::PageUp => Some(cur.map_or(0, |c| c.saturating_sub(page))),
                    _ => None,
                };
                if let Some(t) = target {
                    state.selection.move_to(t, modifiers.shift, modifiers.command && !modifiers.shift, key_of);
                    state.scroll_to = Some(t);
                    selection_changed = true;
                } else if key == Key::A && modifiers.command {
                    state.selection.select_all(rows, key_of);
                    selection_changed = true;
                } else if key == Key::Space && modifiers.command {
                    state.selection.toggle_cursor(key_of);
                    selection_changed = true;
                } else if key == Key::Enter {
                    activated = state.selection.cursor();
                }
            }
        }

        let mut area = egui::ScrollArea::vertical().id_salt(self.id.with("scroll")).auto_shrink([false, false]);
        if let Some(t) = state.scroll_to.take() {
            let y = t as f32 * row_h;
            let vh = state.viewport_h.max(row_h);
            let off = state.offset_y;
            if y < off {
                area = area.vertical_scroll_offset(y);
            } else if y + row_h > off + vh {
                area = area.vertical_scroll_offset(y + row_h - vh);
            }
        }
        let mut visible = 0..0;
        let mut body_resp: Option<Response> = None;
        let out = area.show_viewport(ui, |ui, viewport| {
            let content_h = rows as f32 * row_h;
            ui.set_height(content_h);
            let top = ui.max_rect().top();
            let left = ui.max_rect().left();
            let width = ui.max_rect().width();
            let first = ((viewport.min.y / row_h).floor().max(0.0) as usize).min(rows);
            let last = ((viewport.max.y / row_h).ceil().max(0.0) as usize).min(rows);
            visible = first..last;
            let view_rect = Rect::from_min_size(pos2(left, top + viewport.min.y), vec2(width, viewport.height()));
            let resp = ui.interact(view_rect, body_id, Sense::click());
            // Clic: qué fila.
            let row_at = |p: egui::Pos2| -> Option<usize> {
                let r = ((p.y - top) / row_h).floor();
                (r >= 0.0 && (r as usize) < rows).then_some(r as usize)
            };
            let mods = ui.input(|i| i.modifiers);
            if (resp.clicked() || resp.secondary_clicked() || resp.double_clicked())
                && let Some(r) = resp.interact_pointer_pos().and_then(row_at)
            {
                resp.request_focus();
                if resp.secondary_clicked() {
                    if !state.selection.is_selected(key_of(r)) {
                        state.selection.select_only(r, key_of(r));
                        selection_changed = true;
                    }
                    context = Some(r);
                } else if resp.double_clicked() {
                    activated = Some(r);
                } else {
                    state.selection.click(r, mods, key_of);
                    selection_changed = true;
                }
            } else if resp.clicked() {
                resp.request_focus();
            }
            let hovered_row = resp.hover_pos().and_then(row_at);
            let focused = resp.has_focus();
            let painter = ui.painter().clone();
            // Celdas (x relativas) una sola vez.
            let mut xs = Vec::with_capacity(widths.len());
            let mut x = left;
            for (w, c) in widths.iter().zip(self.columns.iter()) {
                xs.push((x, *w, c.align));
                x += w;
            }
            let mut cells: Vec<(Rect, Align)> = Vec::with_capacity(xs.len());
            for i in visible.clone() {
                let y = top + i as f32 * row_h;
                let rect = Rect::from_min_size(pos2(left, y), vec2(width, row_h));
                let key = key_of(i);
                let selected = state.selection.is_selected(key);
                let is_cursor = state.selection.cursor() == Some(i);
                if selected {
                    painter.rect_filled(rect, 0.0, if focused { colors.selection } else { colors.selection_inactive });
                } else if hovered_row == Some(i) {
                    painter.rect_filled(rect, 0.0, colors.hover);
                }
                cells.clear();
                cells.extend(
                    xs.iter()
                        .map(|&(cx, w, a)| (Rect::from_min_max(pos2(cx + pad, y), pos2(cx + w - pad, y + row_h)), a)),
                );
                let mut row = Row {
                    index: i,
                    selected,
                    cursor: is_cursor,
                    rect,
                    ui,
                    painter: painter.clone(),
                    cells: &cells,
                    colors: &colors,
                    font: font.clone(),
                    bold: bold.clone(),
                    small: small.clone(),
                    style: RowStyle::default(),
                    label: String::new(),
                    pad,
                };
                paint_row(&mut row);
                let label = std::mem::take(&mut row.label);
                if focused && is_cursor {
                    painter.rect_stroke(
                        rect.shrink(0.5),
                        2.0,
                        Stroke::new(1.0, colors.focus),
                        egui::StrokeKind::Inside,
                    );
                }
                // Nodo de accesibilidad de la fila (solo las visibles).
                let rr = ui.interact(rect, body_id.with(i), Sense::hover());
                crate::a11y::row(&rr, if label.is_empty() { "—" } else { &label }, selected);
            }
            body_resp = Some(resp);
        });
        state.offset_y = out.state.clamped_offset().y;
        state.viewport_h = out.inner_rect.height();
        let response = body_resp.unwrap_or_else(|| ui.interact(Rect::NOTHING, body_id, Sense::hover()));
        let n_sel = state.selection.len();
        crate::a11y::area(
            &response,
            &format!(
                "{}, {} {}, {} {}",
                self.label,
                rows,
                crate::lang::pick("items", "elementos"),
                n_sel,
                crate::lang::pick("selected", "seleccionados")
            ),
        );
        ListResponse { response, sort: sort_event, selection_changed, activated, context, visible }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(i: usize) -> u64 {
        i as u64 * 10
    }

    fn sel(s: &Selection) -> Vec<u64> {
        s.keys().collect()
    }

    #[test]
    fn click_shift_and_ctrl() {
        let mut s = Selection::default();
        s.click(3, Modifiers::NONE, &k);
        assert_eq!(sel(&s), vec![30]);
        s.click(6, Modifiers::SHIFT, &k);
        assert_eq!(sel(&s), vec![30, 40, 50, 60]);
        assert_eq!((s.anchor(), s.cursor()), (Some(3), Some(6)));
        // Mayús otra vez: el ancla se queda; el rango se rehace.
        s.click(1, Modifiers::SHIFT, &k);
        assert_eq!(sel(&s), vec![10, 20, 30]);
        // Ctrl alterna y mueve el ancla.
        s.click(9, Modifiers::COMMAND, &k);
        assert_eq!(sel(&s), vec![10, 20, 30, 90]);
        s.click(2, Modifiers::COMMAND, &k);
        assert_eq!(sel(&s), vec![10, 30, 90]);
        // Ctrl+Mayús agrega un rango sin quitar lo demás.
        s.click(4, Modifiers::COMMAND | Modifiers::SHIFT, &k);
        assert_eq!(sel(&s), vec![10, 20, 30, 40, 90]);
        s.click(7, Modifiers::NONE, &k);
        assert_eq!(sel(&s), vec![70]);
    }

    #[test]
    fn keyboard_moves() {
        let mut s = Selection::default();
        s.move_to(0, false, false, &k);
        s.move_to(1, true, false, &k);
        s.move_to(2, true, false, &k);
        assert_eq!(sel(&s), vec![0, 10, 20]);
        s.move_to(5, false, true, &k);
        assert_eq!(sel(&s), vec![0, 10, 20], "Ctrl solo mueve el cursor");
        s.toggle_cursor(&k);
        assert_eq!(sel(&s), vec![0, 10, 20, 50]);
        s.select_all(4, &k);
        assert_eq!(sel(&s), vec![0, 10, 20, 30]);
        assert_eq!(s.cursor(), Some(3), "el cursor queda dentro");
        s.clamp(2);
        assert_eq!(s.cursor(), Some(1));
        s.clamp(0);
        assert_eq!(s.cursor(), None);
    }

    #[test]
    fn grow_column_takes_the_rest() {
        let mut cols =
            vec![Column::new("A", 50.0), Column::new("B", 100.0).grow(), Column::new("C", 60.0).min_width(80.0)];
        let lv = ListView::new("t", &mut cols, 0);
        assert_eq!(lv.widths(500.0), vec![50.0, 370.0, 80.0]);
        assert_eq!(lv.widths(10.0)[1], 24.0, "nunca menos que su mínimo");
    }
}
