//! Tema: convierte los [`tokens`](crate::tokens) en `Visuals` y `Style` de egui y los aplica al
//! contexto. Las vistas piden los colores con [`colors`] (nunca números sueltos) y la densidad con
//! [`density`].
//!
//! ```no_run
//! # let ctx = egui::Context::default();
//! use iurefficient_ui::{theme, Density, ThemeMode};
//! theme::apply(&ctx, ThemeMode::System, Density::Compact);
//! // En cada cuadro: si el escritorio cambió de claro a oscuro, se vuelve a aplicar.
//! theme::refresh(&ctx);
//! ```

use egui::{Color32, Context, CornerRadius, FontFamily, FontId, Id, Margin, Shadow, Stroke, TextStyle, Ui, Visuals};

use crate::tokens::{self, Density, Palette, radius};

/// Tema elegido por el usuario.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ThemeMode {
    /// El del escritorio (portal XDG en Linux; el de la ventana en los demás).
    #[default]
    System,
    Light,
    Dark,
    HighContrast,
}

impl ThemeMode {
    pub const ALL: [ThemeMode; 4] = [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark, ThemeMode::HighContrast];

    pub fn label(self) -> &'static str {
        use crate::lang::pick;
        match self {
            ThemeMode::System => pick("System", "Sistema"),
            ThemeMode::Light => pick("Light", "Claro"),
            ThemeMode::Dark => pick("Dark", "Oscuro"),
            ThemeMode::HighContrast => pick("High contrast", "Alto contraste"),
        }
    }

    /// Nombre estable para guardar en preferencias.
    pub fn key(self) -> &'static str {
        match self {
            ThemeMode::System => "system",
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
            ThemeMode::HighContrast => "high_contrast",
        }
    }

    /// Lee un nombre guardado con [`ThemeMode::key`] (o los de IureProject: «claro», «oscuro»…).
    pub fn from_key(s: &str) -> ThemeMode {
        match s.trim() {
            "light" | "claro" => ThemeMode::Light,
            "dark" | "oscuro" => ThemeMode::Dark,
            "high_contrast" | "alto_contraste" => ThemeMode::HighContrast,
            _ => ThemeMode::System,
        }
    }
}

/// Lo aplicado, guardado en la memoria del contexto.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Applied {
    mode: ThemeMode,
    density: Density,
    /// Modo del escritorio que se vio al aplicar (para [`refresh`]).
    system: Option<bool>,
}

fn state_id() -> Id {
    Id::unique("iurefficient-ui/theme")
}

fn applied(ctx: &Context) -> Option<Applied> {
    ctx.data(|d| d.get_temp::<Applied>(state_id()))
}

/// Aplica tema y densidad al contexto (ambas variantes, clara y oscura, para que egui pueda
/// cambiar entre ellas si sigue al sistema por su cuenta).
pub fn apply(ctx: &Context, mode: ThemeMode, density: Density) {
    let system = crate::system::system_dark();
    let dark = match mode {
        ThemeMode::Light => Some(false),
        ThemeMode::Dark | ThemeMode::HighContrast => Some(true),
        ThemeMode::System => system,
    };
    ctx.set_theme(match dark {
        Some(true) => egui::ThemePreference::Dark,
        Some(false) => egui::ThemePreference::Light,
        None => egui::ThemePreference::System,
    });
    let dark_palette = if mode == ThemeMode::HighContrast { &tokens::HIGH_CONTRAST } else { &tokens::DARK };
    ctx.set_visuals_of(egui::Theme::Light, visuals(&tokens::LIGHT));
    ctx.set_visuals_of(egui::Theme::Dark, visuals(dark_palette));
    ctx.all_styles_mut(|s| style_for_density(s, density));
    use_semibold_headings(ctx);
    ctx.data_mut(|d| d.insert_temp(state_id(), Applied { mode, density, system }));
    // El marco de la ventana (en GNOME/Wayland lo dibuja la app) con el mismo tema.
    let frame = match dark {
        Some(true) => egui::SystemTheme::Dark,
        Some(false) => egui::SystemTheme::Light,
        None => egui::SystemTheme::SystemDefault,
    };
    ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(frame));
}

/// Vuelve a aplicar el tema si sigue al sistema y el escritorio cambió de modo. Barata: llámala
/// en cada cuadro. Devuelve `true` si cambió algo.
pub fn refresh(ctx: &Context) -> bool {
    let Some(a) = applied(ctx) else { return false };
    if a.mode == ThemeMode::System && crate::system::system_dark() != a.system {
        apply(ctx, a.mode, a.density);
        return true;
    }
    false
}

/// Cambia solo la densidad, conservando el tema.
pub fn set_density(ctx: &Context, density: Density) {
    let mode = applied(ctx).map_or(ThemeMode::System, |a| a.mode);
    apply(ctx, mode, density);
}

/// Densidad aplicada (cómoda si no se ha llamado a [`apply`]).
pub fn density(ctx: &Context) -> Density {
    applied(ctx).map_or_else(Density::default, |a| a.density)
}

/// Tema aplicado (sistema si no se ha llamado a [`apply`]).
pub fn mode(ctx: &Context) -> ThemeMode {
    applied(ctx).map_or_else(ThemeMode::default, |a| a.mode)
}

/// Los colores del tema con que se dibuja `ui`.
pub fn colors(ui: &Ui) -> Palette {
    palette_for(ui.ctx(), ui.visuals().dark_mode)
}

/// Los colores del tema actual del contexto.
pub fn ctx_colors(ctx: &Context) -> Palette {
    palette_for(ctx, ctx.global_style().visuals.dark_mode)
}

fn palette_for(ctx: &Context, dark: bool) -> Palette {
    if !dark {
        tokens::LIGHT
    } else if mode(ctx) == ThemeMode::HighContrast {
        tokens::HIGH_CONTRAST
    } else {
        tokens::DARK
    }
}

/// Fuente del texto normal según la densidad.
pub fn body_font(ctx: &Context) -> FontId {
    FontId::proportional(density(ctx).body_size())
}

/// Fuente del texto pequeño según la densidad.
pub fn small_font(ctx: &Context) -> FontId {
    FontId::proportional(density(ctx).small_size())
}

/// La familia con nombre `name` si está instalada; si no, la proporcional normal.
fn family_or_regular(ctx: &Context, name: &str) -> FontFamily {
    if crate::system::has_family(ctx, name) { FontFamily::Name(name.into()) } else { FontFamily::Proportional }
}

/// Familia en negritas (700). Si [`crate::system::install_fonts`] no se llamó, es la proporcional
/// normal. Para no leídos y títulos se prefiere [`semibold_family`], como en macOS.
pub fn bold_family(ctx: &Context) -> FontFamily {
    family_or_regular(ctx, crate::system::BOLD_FAMILY)
}

/// Fuente en negritas del tamaño pedido.
pub fn bold_font(ctx: &Context, size: f32) -> FontId {
    FontId::new(size, bold_family(ctx))
}

/// Familia de peso medio (500): botones, etiquetas de sección, encabezados de columna.
pub fn medium_family(ctx: &Context) -> FontFamily {
    family_or_regular(ctx, crate::system::MEDIUM_FAMILY)
}

/// Fuente de peso medio del tamaño pedido.
pub fn medium_font(ctx: &Context, size: f32) -> FontId {
    FontId::new(size, medium_family(ctx))
}

/// Familia seminegrita (600): no leídos, títulos, nombres.
pub fn semibold_family(ctx: &Context) -> FontFamily {
    family_or_regular(ctx, crate::system::SEMIBOLD_FAMILY)
}

/// Fuente seminegrita del tamaño pedido.
pub fn semibold_font(ctx: &Context, size: f32) -> FontId {
    FontId::new(size, semibold_family(ctx))
}

/// Familia con cifras de ancho fijo: horas, fechas y conteos alineados en columnas. Las letras
/// salen de la normal (egui no deja activar `tnum`; ver README, «Tipografía»).
pub fn tabular_family(ctx: &Context) -> FontFamily {
    family_or_regular(ctx, crate::system::TABULAR_FAMILY)
}

/// Fuente de cifras tabulares del tamaño pedido.
pub fn tabular_font(ctx: &Context, size: f32) -> FontId {
    FontId::new(size, tabular_family(ctx))
}

/// Pone los títulos (`TextStyle::Heading`) en seminegrita si la familia está instalada. Lo llaman
/// [`apply`] y la instalación de fuentes, así que el orden entre ambas no importa.
pub fn use_semibold_headings(ctx: &Context) {
    let family = semibold_family(ctx);
    ctx.all_styles_mut(|s| {
        if let Some(h) = s.text_styles.get_mut(&TextStyle::Heading) {
            h.family = family.clone();
        }
    });
}

/// Mezcla `top` (con su alfa) sobre `bottom` opaco.
pub fn blend(bottom: Color32, top: Color32) -> Color32 {
    let k = 1.0 - f32::from(top.a()) / 255.0;
    let mix = |t: u8, b: u8| (f32::from(t) + f32::from(b) * k).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(mix(top.r(), bottom.r()), mix(top.g(), bottom.g()), mix(top.b(), bottom.b()))
}

/// `Visuals` de egui a partir de una paleta.
pub fn visuals(p: &Palette) -> Visuals {
    let mut v = if p.dark { Visuals::dark() } else { Visuals::light() };
    let control = CornerRadius::same(radius::CONTROL as u8);
    let button = blend(p.surface, if p.dark { Color32::from_white_alpha(14) } else { Color32::from_black_alpha(10) });
    let hovered = blend(p.surface, if p.dark { Color32::from_white_alpha(24) } else { Color32::from_black_alpha(18) });
    let active = blend(p.surface, if p.dark { Color32::from_white_alpha(36) } else { Color32::from_black_alpha(28) });
    let border = if p.dark { blend(p.separator, Color32::from_white_alpha(10)) } else { p.separator };

    v.panel_fill = p.surface;
    v.window_fill = p.surface;
    v.faint_bg_color = p.surface_alt;
    v.extreme_bg_color = if p.dark { p.bg } else { p.surface };
    v.text_edit_bg_color = Some(if p.dark { p.bg } else { p.surface });
    v.code_bg_color = p.surface_alt;
    v.hyperlink_color = p.accent;
    v.warn_fg_color = p.warning;
    v.error_fg_color = p.danger;
    v.weak_text_color = Some(p.text_muted);
    v.window_stroke = Stroke::new(1.0, p.separator);
    v.window_corner_radius = CornerRadius::same(radius::WINDOW as u8);
    v.menu_corner_radius = CornerRadius::same(radius::CARD as u8);
    let shadow_alpha = if p.dark { 110 } else { 34 };
    v.window_shadow = Shadow { offset: [0, 6], blur: 24, spread: 0, color: Color32::from_black_alpha(shadow_alpha) };
    v.popup_shadow = Shadow { offset: [0, 3], blur: 12, spread: 0, color: Color32::from_black_alpha(shadow_alpha) };
    v.selection.bg_fill = p.selection;
    v.selection.stroke = Stroke::new(1.0, p.text);
    v.text_cursor.stroke = Stroke::new(2.0, p.accent);
    v.striped = false;
    v.slider_trailing_fill = true;

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = p.surface;
    w.noninteractive.weak_bg_fill = p.surface;
    w.noninteractive.bg_stroke = Stroke::new(1.0, p.separator);
    w.noninteractive.fg_stroke = Stroke::new(1.0, p.text);
    w.noninteractive.corner_radius = control;

    w.inactive.bg_fill = button;
    w.inactive.weak_bg_fill = button;
    w.inactive.bg_stroke = Stroke::new(1.0, border);
    w.inactive.fg_stroke = Stroke::new(1.0, p.text);
    w.inactive.corner_radius = control;
    w.inactive.expansion = 0.0;

    w.hovered.bg_fill = hovered;
    w.hovered.weak_bg_fill = hovered;
    w.hovered.bg_stroke = Stroke::new(1.0, border);
    w.hovered.fg_stroke = Stroke::new(1.5, p.text);
    w.hovered.corner_radius = control;
    w.hovered.expansion = 0.0;

    w.active.bg_fill = active;
    w.active.weak_bg_fill = active;
    w.active.bg_stroke = Stroke::new(1.0, p.focus);
    w.active.fg_stroke = Stroke::new(1.5, p.text);
    w.active.corner_radius = control;
    w.active.expansion = 0.0;

    w.open.bg_fill = hovered;
    w.open.weak_bg_fill = hovered;
    w.open.bg_stroke = Stroke::new(1.0, border);
    w.open.fg_stroke = Stroke::new(1.0, p.text);
    w.open.corner_radius = control;

    if *p == tokens::HIGH_CONTRAST {
        v.override_text_color = Some(p.text);
        v.selection.stroke = Stroke::new(2.0, p.text);
        for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active]
        {
            w.bg_stroke = Stroke::new(1.5, p.text);
            w.fg_stroke = Stroke::new(1.5, p.text);
        }
    }
    v
}

/// Espaciado y tamaños de letra de una densidad.
pub fn style_for_density(style: &mut egui::Style, d: Density) {
    let s = &mut style.spacing;
    s.item_spacing = d.item_spacing();
    s.button_padding = d.button_padding();
    s.interact_size.y = d.control_height();
    s.icon_width = d.icon_size().min(16.0);
    s.menu_margin = Margin::same(6);
    s.window_margin = Margin::same(12);
    s.combo_height = 320.0;
    let body = d.body_size();
    let heading = d.heading_size();
    style.text_styles = [
        (TextStyle::Small, FontId::proportional(d.small_size())),
        (TextStyle::Body, FontId::proportional(body)),
        (TextStyle::Button, FontId::proportional(body)),
        (TextStyle::Heading, FontId::proportional(heading)),
        (TextStyle::Monospace, FontId::monospace(body - 1.0)),
    ]
    .into();
}

#[cfg(test)]
pub(crate) mod contrast {
    use egui::Color32;

    /// Contraste WCAG 2 de `a` (que puede ser translúcido: se mezcla sobre `b`) contra `b`.
    pub(crate) fn ratio(a: Color32, b: Color32) -> f32 {
        let a = super::blend(b, a);
        fn lum(c: Color32) -> f32 {
            let ch = |v: u8| {
                let s = f32::from(v) / 255.0;
                if s <= 0.039_28 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
            };
            0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
        }
        let (x, y) = (lum(a), lum(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }
}

#[cfg(test)]
mod tests {
    use super::contrast::ratio;
    use super::*;

    /// Contraste AA (4.5:1) del texto normal y del secundario sobre todos los fondos y la selección,
    /// y 3:1 para el acento (controles, iconos) — en los tres temas.
    #[test]
    fn palettes_meet_wcag_aa() {
        let mut low = Vec::new();
        for (name, p) in [("claro", tokens::LIGHT), ("oscuro", tokens::DARK), ("alto contraste", tokens::HIGH_CONTRAST)]
        {
            let fills = [
                ("bg", p.bg),
                ("surface", p.surface),
                ("surface_alt", p.surface_alt),
                ("selection", p.selection),
                ("selection_inactive", p.selection_inactive),
                ("flag_tint", p.flag_tint),
            ];
            for (what, fg, min) in [
                ("text", p.text, 4.5),
                ("text_muted", p.text_muted, 4.5),
                ("accent", p.accent, 3.0),
                ("danger", p.danger, 3.0),
                ("warning", p.warning, 3.0),
                ("success", p.success, 3.0),
            ] {
                for (on, bg) in fills {
                    let r = ratio(fg, bg);
                    if r < min {
                        low.push(format!("{name}: {what} sobre {on}: {r:.2}"));
                    }
                }
            }
            let r = ratio(p.on_accent, p.accent);
            if r < 4.5 {
                low.push(format!("{name}: on_accent sobre accent: {r:.2}"));
            }
        }
        assert!(low.is_empty(), "contraste bajo:\n{}", low.join("\n"));
    }

    #[test]
    fn apply_stores_density_and_mode() {
        let ctx = Context::default();
        apply(&ctx, ThemeMode::Dark, Density::Dense);
        assert_eq!(density(&ctx), Density::Dense);
        assert_eq!(mode(&ctx), ThemeMode::Dark);
        assert_eq!(ctx.style_of(egui::Theme::Dark).spacing.item_spacing, Density::Dense.item_spacing());
        set_density(&ctx, Density::Comfortable);
        assert_eq!(mode(&ctx), ThemeMode::Dark);
        assert_eq!(density(&ctx), Density::Comfortable);
        assert_eq!(ThemeMode::from_key("oscuro"), ThemeMode::Dark);
        for m in ThemeMode::ALL {
            assert_eq!(ThemeMode::from_key(m.key()), m);
        }
    }
}
