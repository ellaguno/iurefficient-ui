# Iurefficient UI

Kit de interfaz **egui 0.37** compartido por las apps de escritorio de la suite Iurefficient
(IureDesk primero, IureProject después). Rust puro, sin `unsafe`, sin `unwrap` fuera de pruebas,
textos en español e inglés, accesible (AccessKit) y sin activos de Microsoft, Apple ni Google: los
iconos se dibujan en código.

Estilo: neutro y tranquilo (grises fríos, un solo acento azul y el marino Iurefficient como
identidad), sin cintas. Tres densidades; en **Densa** las filas miden 22 px.

| Claro, cómoda | Oscuro, densa | Paleta (Ctrl+K) |
|---|---|---|
| ![](docs/gallery-light.png) | ![](docs/gallery-dark-dense.png) | ![](docs/palette-light.png) |

Más capturas en [`docs/`](docs): compacta en inglés, alto contraste, paleta oscura y la hoja de
iconos (`icons.png`, `icons-dark.png`).

## Uso

```toml
[dependencies]
iurefficient-ui = { path = "../iurefficient-ui" }   # característica `portal` activa por omisión
```

```rust
use iurefficient_ui::{system, theme, Density, ThemeMode};

// Al arrancar:
system::watch_system_theme(&ctx);          // portal XDG en Linux; no hace nada en otros sistemas
system::install_fonts(&ctx, true);         // Noto/Ubuntu/DejaVu/Liberation si están instaladas
theme::apply(&ctx, ThemeMode::System, Density::Compact);
// En cada cuadro:
theme::refresh(&ctx);                      // si el escritorio cambió de claro a oscuro
```

La galería muestra todo junto (es la mejor referencia de uso):

```sh
cargo run --example gallery                                   # ventana interactiva
cargo run --example gallery -- /tmp/g.png --dark --density dense
cargo run --example gallery -- /tmp/p.png --palette=resp      # con la paleta abierta
cargo run --example gallery -- /tmp/i.png --icons --ppp 2     # hoja de iconos al doble
```

Opciones: `--dark`, `--hc` (alto contraste), `--density comfortable|compact|dense`,
`--lang en|es`, `--palette[=texto]`, `--icons`, `--ppp N`. Sin ruta abre una ventana (eframe).

## Módulos

| Módulo | Qué da |
|---|---|
| `tokens` | `brand::{NAVY, NAVY_DARK_THEME, BLUE, GREEN}`; `Palette` con colores semánticos (`bg`, `surface`, `surface_alt`, `text`, `text_muted`, `accent`, `on_accent`, `danger`, `warning`, `success`, `info`, `selection`, `selection_inactive`, `focus`, `separator`, `hover`, `pressed`, `flag_tint`, `brand`) en `LIGHT`, `DARK` y `HIGH_CONTRAST`; `space`, `radius`, `text`; `Density { Comfortable, Compact, Dense }` con `row_height()` (34/28/22), `body_size()`, `small_size()`, `icon_size()`, `button_padding()`, `item_spacing()`, `control_height()`, `cell_padding()`, `label()`, `key()/from_key()`. |
| `theme` | `ThemeMode { System, Light, Dark, HighContrast }`; `apply(ctx, mode, density)`, `refresh(ctx)`, `set_density`, `density(ctx)`, `mode(ctx)`; `colors(ui) -> Palette`, `ctx_colors(ctx)`; `body_font`, `small_font`, `bold_font`, `bold_family`; `visuals(&Palette)`, `style_for_density`, `blend`. Prueba de contraste WCAG AA de las tres paletas. |
| `system` | `system_dark()`, `set_system_dark()`, `watch_system_theme(ctx)` (hilo único que consulta `org.freedesktop.appearance color-scheme` cada 2 s y pide un cuadro al cambiar; característica `portal`); `system_font() -> Option<SystemFont>` (solo fuentes libres, por nombre de archivo exacto, con su negrita); `install_fonts(ctx, use_system)` (registra además la familia `BOLD_FAMILY`); `set_text_scale(ctx, f)` / `clamp_scale` (80–200 %). |
| `icons` | `enum Icon` (108 iconos, cuadrícula 24×24, trazo de 1.6): los de IureProject y los del gestor personal (`Mail`, `Inbox`, `Send`, `Drafts`, `Archive`, `Trash`, `Junk`, `Flag`, `Star`/`StarFilled`, `Reply`, `ReplyAll`, `Forward`, `Attachment`, `Calendar`, `Contacts`, `Person`, `Notes`, `Tasks`, `Check`, `Search`, `Settings`, `Plus`, `MoreHorizontal`, `Chevron*`, `Sparkle`, `Sync`, `Offline`, `Bell`, `Pin`, `Tag`, `Folder`, `Hyperlink`, `Phone`, `Video`, `Close`, `Filter`, `Lock`, `Menu`, `Warning`, `Alert`). `Icon::ALL`, `Icon::named`, `Icon::name`; `paint(painter, rect, icon, ink, accent)`, `paint_themed`, `icon_button(ui, icon, tip, enabled)`, `menu_item(ui, icon, label)`. |
| `a11y` | `button`, `toggle`, `row` (rol `ListItem`), `area`, `label`: nombre, rol y estado para AccessKit de lo dibujado a mano. |
| `lang` | `Lang { En, Es }`, `lang()`, `set_lang()`, `detect()` (`LC_ALL`/`LC_MESSAGES`/`LANG`, luego `sys-locale` en Windows/macOS), `from_tag()`, `pick(en, es)`. |
| `fuzzy` | `fold` (minúsculas, sin acentos), `score`, `score_with_extra`, `token_score`, `rank`: búsqueda difusa por programación dinámica (inicio de texto y de palabra, letras seguidas, saltos penalizados; acotada a 16 palabras × 32 letras × 256 caracteres). |
| `palette` | `PaletteItem { id, label, hint, shortcut, icon, keywords, enabled }` con constructor encadenable; `CommandPalette` (`open`, `open_with`, `close`, `toggle`, `is_open`, `query`, `cursor`, `handle_shortcut` = Ctrl/⌘+K, `show(ctx, &items) -> Option<String>`); `search(query, items, limit)`. Modal con fondo atenuado, flechas/RePág/AvPág, Enter, Esc o clic fuera. |
| `list` | `ListView::new(id, &mut columns, rows).row_height(h).header(b).label(s).show(ui, &mut ListState, key_of, paint_row) -> ListResponse { sort, selection_changed, activated, context, visible, response }`. `Column` (`fixed`, `not_sortable`, `right`, `grow`, `min_width`; ancho arrastrable), `SortDir`, `ListState` (`selection`, `sort`, `scroll_to`), `Selection` por clave estable `u64` (`click`, `move_to`, `toggle_cursor`, `select_all`, `clamp`), `Row` (`style(RowStyle { unread, flagged, muted })`, `text`, `text_muted`, `text_with`, `text_in`, `icon`, `icon_colored`, `cell`, `painter`, `describe`). Solo pinta las filas visibles; prueba con 200 000 filas. |
| `widgets` | `nav_rail(ui, &[NavItem], selected) -> Option<&str>` y `rail_width(ctx)`; `sidebar_item(ui, icon, label, count, selected)`; `split::{left, right, top_bar, bottom_bar, center, frame, PaneKind}`; `badge`, `count_badge`, `paint_badge`, `BadgeKind`; `avatar(ui, name, email, d)`, `initials`, `avatar_color` (FNV-1a del correo, 10 colores AA); `Banner::new(Severity, text).title().action().dismissible().show(ui)` y `Toasts` (`push`, `show(ctx) -> Option<id>`, caducan solas); `EmptyState::new(icon, title).body().action().show(ui)`; `segmented(ui, &mut value, &[(v, label)])`; `SearchField::new(&mut s).hint().shortcut().show(ui)` (Esc vacía, botón ×, atajo como tecla); `keycap(ui, "Ctrl+K")`, `paint_shortcut`; `text::elided`; `primary_button`, `section_label`. |

Los nombres más usados se reexportan en la raíz (`iurefficient_ui::{Icon, Density, ThemeMode, pick, …}`).

## Calidad

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

- Pruebas unitarias: búsqueda difusa, selección (clic, Mayús, Ctrl, teclado), anchos de columna,
  contraste AA de paletas, insignias y avatares, iniciales, fuentes libres, caducidad de avisos.
- `tests/widgets.rs` (egui_kittest con wgpu): paleta (Ctrl+K, escribir, flechas, Enter, clic, Esc,
  deshabilitados), lista de 200 000 filas (solo ~17 filas pintadas por cuadro, 60 páginas en < 5 s,
  Fin/Inicio, Ctrl+A, Mayús/Ctrl+clic, ordenar por encabezado) y capturas de referencia en
  `tests/snapshots/` (`UPDATE_SNAPSHOTS=1 cargo test` para regenerarlas). El arnés de kittest
  verifica además que todo control tenga nombre accesible.
- Las pruebas que renderizan se turnan (un `Mutex`): algunos controladores fallan con dos
  dispositivos wgpu a la vez.

## Plan de migración de IureProject

IureProject está en egui 0.36; primero sube a **0.37** (cambios que se encontraron aquí:
`Id::new` → `Id::unique` / `IdSalt::new`, `WidgetType` → `Role` (`SelectableLabel` ya no existe:
`Role::Button` con `selected`), `Atom::custom` recibe `IdSalt`, `scroll_area::State::offset` →
`clamped_offset()`, `SidePanel`/`TopBottomPanel` → `egui::Panel::{left,right,top,bottom}(…).show(ui, …)`,
y kittest revisa nombres accesibles por omisión). Después, módulo por módulo:

| IureProject | iurefficient-ui | Notas |
|---|---|---|
| `crates/render/src/tokens.rs` (`brand`, `space`, `radius`, `text`) | `tokens` | `render` conserva su paleta del Gantt (la usa también la web, sin egui). Los `color::*` sueltos pasan a `Palette`. |
| `ui-egui/src/theme.rs` | `theme::colors(ui)` | `text→c.text`, `weak→c.text_muted`, `accent→c.accent`, `brand→c.brand`, `grid→c.separator`, `selection→c.selection`, `DANGER/SUCCESS→c.danger/c.success`, `warning→c.warning`; `highlight` (filtro «Resaltar») se queda local o usa `flag_tint`. |
| `ui-egui/src/icons.rs` | `icons` | Mismo `enum` y mismos nombres; `Icon::of_command(id)` queda en IureProject como función de una línea (`find(id).and_then(\|c\| Icon::named(c.icon))`). |
| `ui-egui/src/prefs.rs` | `system` + `theme` | `system_dark`, `watch_system_theme`, `portal_dark`, `system_font`, `find` → `system`; `polished`/`high_contrast` → `theme::visuals`; `Prefs::apply_theme` → `theme::apply(ctx, ThemeMode, Density)`. La persistencia (`preferencias.json`) se queda en la app; `ThemeMode::from_key` ya acepta «claro»/«oscuro»/«alto_contraste». Agregar la densidad a Opciones. |
| `ui-egui/src/a11y.rs` | `a11y` | Igual, más `label`; `row` ahora anuncia `ListItem`. |
| `ui-egui/src/palette.rs` | `palette` + `fuzzy` | `fold`/`score` → `fuzzy` (ranking mejor: iniciales y letras seguidas). `entries()` de IureProject construye `PaletteItem` (id = `ribbon:<act>`, `file:<page>`, `task:<uid>`, `filter:<n>`) y `run()` traduce el id devuelto a `Target`. |
| `ui-egui/src/sheet.rs` (filas, selección) | `list` (parcial) | La hoja tiene edición en celda y esquema; se reutilizan `Selection` (por `TaskUid`) y `widgets::text::elided`. Vistas de solo lectura (uso, recursos) pueden pasar a `ListView`. |
| `engine/src/lang.rs` | `lang` | El motor (L4) no puede depender de egui, así que conserva su `lang`; la UI llama a los dos `set_lang` al cambiar el idioma. |
| `ribbon.rs`, `backstage.rs`, Gantt y demás vistas | — | Se quedan en IureProject; usan `theme::colors`, `icons` y los controles pequeños (`Banner`, `segmented`, `SearchField`, `EmptyState`). |

Orden sugerido: (1) egui 0.37; (2) `tokens`/`theme`/`icons`/`a11y` (cambio mecánico, capturas
`ui_shot` antes y después); (3) `system` + densidades en Opciones; (4) paleta; (5) vistas de solo
lectura con `ListView`.

## Pendientes conocidos

- Sin columnas reordenables por arrastre ni desplazamiento horizontal en `ListView` (la columna
  `grow` absorbe el ancho); sin filas de altura variable.
- El cursor y el ancla de la selección son índices: si llegan filas nuevas arriba, la app debe
  mover el cursor (las claves seleccionadas sí se conservan).
- Precisión `f32`: listas de más de ~5 millones de filas pierden exactitud al desplazarse.
- `watch_system_theme` consulta el portal cada 2 s en vez de suscribirse a `SettingChanged`.
- Los glifos que no estén en la fuente (p. ej. flechas «↑») se ven como cajas; los textos de la
  interfaz evitan esos caracteres.

## Licencia

FSL-1.1-ALv2 ([LICENSE.md](LICENSE.md), [NOTICE](NOTICE)), titular Eduardo Llaguno Velasco.
