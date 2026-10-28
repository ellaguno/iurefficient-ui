//! Elección de la tipografía de la interfaz: la incluida (Inter), la del escritorio (en GNOME,
//! `org.gnome.desktop.interface font-name` por el portal XDG) o una familia libre instalada
//! (IBM Plex Sans, Source Sans 3, Noto Sans…), con su búsqueda por nombre de familia en las
//! carpetas de fuentes habituales y la opción de «texto nítido».
//!
//! La búsqueda mira los nombres de archivo (`IBMPlexSans-SemiBold.otf`, `UbuntuSans[wdth,wght].ttf`,
//! `Cantarell-VF.otf`…): sin fontconfig, que es C. Las fuentes variables se registran con el eje
//! `wght` en 400/500/600/700. Nunca se toma una fuente de Microsoft o Apple aunque sea la del
//! escritorio: entonces se pasa a Cantarell, Ubuntu o Noto Sans.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use crate::system::{FontBytes, FontSet};

/// Familias libres que se ofrecen si están instaladas, en este orden.
pub const OFFERED_FAMILIES: [&str; 3] = ["IBM Plex Sans", "Source Sans 3", "Noto Sans"];

/// Respaldo de «Fuente del sistema» si la del escritorio no se encuentra o no es libre.
const DESKTOP_FALLBACKS: [&str; 5] = ["Cantarell", "Ubuntu Sans", "Ubuntu", "Noto Sans", "DejaVu Sans"];

/// Familias que no se cargan nunca (no son libres).
const PROPRIETARY: [&str; 12] = [
    "segoe",
    "calibri",
    "arial",
    "helvetica",
    "sf pro",
    "san francisco",
    "tahoma",
    "verdana",
    "lucida grande",
    "consolas",
    "cambria",
    "sfns",
];

/// Qué tipografía usar.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FontChoice {
    /// Inter, incluida en el binario (por omisión).
    #[default]
    Bundled,
    /// La del escritorio (o un respaldo libre).
    Desktop,
    /// Una familia instalada por su nombre («IBM Plex Sans»).
    Family(String),
}

impl FontChoice {
    /// Clave para guardar en las preferencias: `inter`, `system` o `family:<nombre>`.
    pub fn key(&self) -> String {
        match self {
            FontChoice::Bundled => "inter".into(),
            FontChoice::Desktop => "system".into(),
            FontChoice::Family(f) => format!("family:{f}"),
        }
    }

    /// Lee una clave de [`FontChoice::key`]; lo desconocido es Inter.
    pub fn from_key(key: &str) -> FontChoice {
        match key.trim() {
            "system" => FontChoice::Desktop,
            k => match k.strip_prefix("family:").map(str::trim) {
                Some(f) if !f.is_empty() && f.len() <= 100 => FontChoice::Family(f.to_string()),
                _ => FontChoice::Bundled,
            },
        }
    }
}

/// ¿Texto nítido? Ver [`set_crisp_text`].
static CRISP: AtomicBool = AtomicBool::new(false);

/// ¿Está activo el texto nítido?
pub fn crisp_text() -> bool {
    CRISP.load(Ordering::Relaxed)
}

/// Texto nítido: sin posiciones de subpíxel (cada letra cae en un píxel entero: más nítido, en
/// especial con escalado fraccionario) y con ajuste horizontal a la rejilla de píxeles. Cambia los
/// estilos actuales; el ajuste horizontal se aplica al volver a instalar las fuentes
/// ([`install_choice`] lo hace).
pub fn set_crisp_text(ctx: &egui::Context, crisp: bool) {
    CRISP.store(crisp, Ordering::Relaxed);
    ctx.all_styles_mut(|s| apply_text_options(&mut s.visuals));
}

/// Pone en `v` las opciones de texto de [`set_crisp_text`] (lo llama `theme::visuals`).
pub fn apply_text_options(v: &mut egui::Visuals) {
    v.text_options.subpixel_binning = !crisp_text();
    v.text_options.font_hinting = true;
}

/// Ajuste a la rejilla de cada fuente según [`crisp_text`].
pub(crate) fn hinting_target() -> egui::epaint::text::HintingTarget {
    use egui::epaint::text::{HintingTarget, SmoothHinting};
    if crisp_text() {
        HintingTarget::Smooth(SmoothHinting {
            light: false,
            symmetric_rendering: false,
            preserve_linear_metrics: false,
        })
    } else {
        HintingTarget::default()
    }
}

/// Separa «Ubuntu Sans Bold 11» en familia y tamaño (`("Ubuntu Sans", Some(11.0))`): el formato
/// de Pango que usa GNOME. Se quitan los estilos del final.
pub fn parse_font_name(raw: &str) -> (String, Option<f32>) {
    let mut words: Vec<&str> = raw.split_whitespace().take(20).collect();
    let mut size = None;
    if let Some(last) = words.last()
        && let Ok(s) = last.trim_end_matches("px").parse::<f32>()
    {
        if s.is_finite() && s > 0.0 && s < 200.0 {
            size = Some(s);
        }
        words.pop();
    }
    const STYLES: [&str; 14] = [
        "regular",
        "book",
        "normal",
        "medium",
        "semibold",
        "demibold",
        "bold",
        "italic",
        "oblique",
        "light",
        "thin",
        "heavy",
        "black",
        "condensed",
    ];
    while words.len() > 1
        && words.last().is_some_and(|w| STYLES.contains(&w.to_ascii_lowercase().trim_end_matches(',')))
    {
        words.pop();
    }
    (words.join(" ").trim_end_matches(',').trim().to_string(), size)
}

/// La fuente del escritorio (familia y tamaño), si se puede saber. En GNOME la da el portal
/// (`org.gnome.desktop.interface font-name`); en KDE, `~/.config/kdeglobals` (`font=`).
pub fn desktop_font() -> Option<(String, Option<f32>)> {
    #[cfg(all(target_os = "linux", feature = "portal"))]
    if let Some(raw) = crate::system::portal_string("org.gnome.desktop.interface", "font-name") {
        let (f, s) = parse_font_name(&raw);
        if !f.is_empty() {
            return Some((f, s));
        }
    }
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let path = home.join(".config/kdeglobals");
    if std::fs::metadata(&path).ok()?.len() > 1_000_000 {
        return None;
    }
    let text = std::fs::read_to_string(path).ok()?;
    let line = text.lines().take(10_000).find(|l| l.starts_with("font="))?;
    let mut parts = line.trim_start_matches("font=").split(',');
    let family = parts.next()?.trim().to_string();
    let size = parts.next().and_then(|s| s.trim().parse::<f32>().ok()).filter(|s| s.is_finite() && *s > 0.0);
    (!family.is_empty()).then_some((family, size))
}

/// ¿Es una familia que no se carga (de Microsoft o Apple)?
pub fn is_proprietary(family: &str) -> bool {
    let f = family.to_ascii_lowercase();
    PROPRIETARY.iter().any(|p| f.contains(p))
}

/// Peso de un archivo de la familia.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Weight {
    Regular,
    Medium,
    SemiBold,
    Bold,
    /// Fuente variable con eje de peso.
    Variable,
}

/// Archivos de fuente encontrados (la búsqueda en disco se hace una vez).
fn index() -> &'static [PathBuf] {
    static INDEX: OnceLock<Vec<PathBuf>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut out = Vec::new();
        for d in crate::system::font_dirs() {
            walk(&d, 0, &mut out);
        }
        out
    })
}

/// Junta los `.ttf`/`.otf` bajo `dir` (hasta 5 niveles y 50 000 archivos).
fn walk(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
    if depth > 5 || out.len() > 50_000 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten().take(10_000) {
        let path = e.path();
        if e.file_type().is_ok_and(|t| t.is_dir()) {
            walk(&path, depth + 1, out);
        } else if path
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(|x| x.eq_ignore_ascii_case("ttf") || x.eq_ignore_ascii_case("otf"))
        {
            out.push(path);
        }
    }
}

fn normalize(s: &str) -> String {
    s.chars().filter(char::is_ascii_alphanumeric).map(|c| c.to_ascii_lowercase()).collect()
}

/// Peso del archivo `file` si es de la familia `family` (normalizada), sin cursivas.
fn classify(family: &str, file: &Path) -> Option<Weight> {
    let stem = file.file_stem()?.to_str()?;
    let lower = stem.to_ascii_lowercase();
    if lower.contains("italic") || lower.contains("oblique") {
        return None;
    }
    let (head, axes) = match stem.split_once('[') {
        Some((h, a)) => (h, Some(a)),
        None => (stem, None),
    };
    let (name, style) = head.split_once('-').unwrap_or((head, ""));
    let mut name = normalize(name);
    let mut variable = axes.is_some_and(|a| a.contains("wght"));
    if let Some(n) = name.strip_suffix("vf") {
        name = n.to_string();
        variable = true;
    }
    if name != family {
        return None;
    }
    let style = normalize(style);
    if style == "vf" || variable {
        return Some(Weight::Variable);
    }
    match style.as_str() {
        "" | "regular" | "r" | "book" | "normal" | "upright" => Some(Weight::Regular),
        "medium" | "m" => Some(Weight::Medium),
        "semibold" | "demibold" | "sb" => Some(Weight::SemiBold),
        "bold" | "b" => Some(Weight::Bold),
        _ => None,
    }
}

fn read(path: &Path) -> Option<FontBytes> {
    if std::fs::metadata(path).ok()?.len() > 30_000_000 {
        return None;
    }
    let b = FontBytes::Shared(Arc::new(std::fs::read(path).ok()?));
    b.is_valid().then_some(b)
}

/// Busca una familia instalada por su nombre («IBM Plex Sans», «Ubuntu Sans»). `None` si no está,
/// si no tiene un peso normal que se pueda leer o si es de Microsoft o Apple.
pub fn find_family(family: &str) -> Option<FontSet> {
    find_family_in(family, index())
}

fn find_family_in(family: &str, files: &[PathBuf]) -> Option<FontSet> {
    if family.trim().is_empty() || is_proprietary(family) {
        return None;
    }
    let norm = normalize(family);
    let mut found: Vec<(Weight, &PathBuf)> = files.iter().filter_map(|p| classify(&norm, p).map(|w| (w, p))).collect();
    found.sort_by(|a, b| a.1.cmp(b.1));
    let path = |w: Weight| found.iter().find(|(x, _)| *x == w).map(|(_, p)| p.as_path());
    if let Some(regular) = path(Weight::Regular).and_then(read) {
        return Some(FontSet {
            medium: path(Weight::Medium).and_then(read),
            semibold: path(Weight::SemiBold).and_then(read),
            bold: path(Weight::Bold).and_then(read),
            ..FontSet::new(family.trim(), regular)
        });
    }
    let var = path(Weight::Variable).and_then(read)?;
    Some(FontSet { variable_weight: true, ..FontSet::new(family.trim(), var) })
}

/// Las familias de [`OFFERED_FAMILIES`] que están instaladas.
pub fn available_families() -> Vec<String> {
    static FOUND: OnceLock<Vec<String>> = OnceLock::new();
    FOUND
        .get_or_init(|| {
            let files = index();
            OFFERED_FAMILIES
                .iter()
                .filter(|f| {
                    let n = normalize(f);
                    files.iter().any(|p| matches!(classify(&n, p), Some(Weight::Regular | Weight::Variable)))
                })
                .map(|f| (*f).to_string())
                .collect()
        })
        .clone()
}

/// La familia del escritorio como [`FontSet`] (o el primer respaldo libre instalado), con su nombre.
pub fn desktop_font_set() -> Option<FontSet> {
    let wanted = desktop_font().map(|(f, _)| f);
    wanted.iter().map(String::as_str).chain(DESKTOP_FALLBACKS).find_map(find_family)
}

/// Instala la tipografía elegida (con la libre del sistema y las de egui de respaldo) y devuelve
/// el nombre de la familia que quedó. Si la elegida no está, queda Inter.
pub fn install_choice(ctx: &egui::Context, choice: &FontChoice) -> Option<String> {
    let set = match choice {
        FontChoice::Bundled => None,
        FontChoice::Desktop => desktop_font_set(),
        FontChoice::Family(f) => find_family(f),
    };
    match set {
        Some(s) => {
            crate::system::install_font_set(ctx, &s, true);
            Some(s.name)
        }
        None => crate::system::install(ctx, crate::system::FontSource::Bundled),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choice_keys_round_trip() {
        for c in [FontChoice::Bundled, FontChoice::Desktop, FontChoice::Family("IBM Plex Sans".into())] {
            assert_eq!(FontChoice::from_key(&c.key()), c);
        }
        assert_eq!(FontChoice::from_key("basura"), FontChoice::Bundled);
        assert_eq!(FontChoice::from_key("family:"), FontChoice::Bundled);
        assert_eq!(FontChoice::from_key(&format!("family:{}", "x".repeat(500))), FontChoice::Bundled);
    }

    #[test]
    fn pango_font_names() {
        assert_eq!(parse_font_name("Ubuntu Sans 11"), ("Ubuntu Sans".into(), Some(11.0)));
        assert_eq!(parse_font_name("Cantarell Bold 10.5"), ("Cantarell".into(), Some(10.5)));
        assert_eq!(parse_font_name("Noto Sans"), ("Noto Sans".into(), None));
        assert_eq!(parse_font_name("Source Sans 3 12"), ("Source Sans 3".into(), Some(12.0)));
        assert_eq!(parse_font_name(""), (String::new(), None));
        assert_eq!(parse_font_name("Bold"), ("Bold".into(), None));
    }

    #[test]
    fn file_names_are_classified_by_family() {
        let w = |fam: &str, f: &str| classify(&normalize(fam), Path::new(f));
        assert_eq!(w("IBM Plex Sans", "/f/IBMPlexSans-Regular.otf"), Some(Weight::Regular));
        assert_eq!(w("IBM Plex Sans", "/f/IBMPlexSans-SemiBold.otf"), Some(Weight::SemiBold));
        assert_eq!(w("IBM Plex Sans", "/f/IBMPlexSans-Italic.otf"), None);
        assert_eq!(w("IBM Plex Sans", "/f/IBMPlexSansCondensed-Regular.otf"), None);
        assert_eq!(w("Noto Sans", "/f/NotoSans-Bold.ttf"), Some(Weight::Bold));
        assert_eq!(w("Noto Sans", "/f/NotoSansArabic-Regular.ttf"), None);
        assert_eq!(w("Noto Sans", "/f/NotoSansMono-Regular.ttf"), None);
        assert_eq!(w("Ubuntu Sans", "/f/UbuntuSans[wdth,wght].ttf"), Some(Weight::Variable));
        assert_eq!(w("Ubuntu Sans", "/f/UbuntuSans-Italic[wdth,wght].ttf"), None);
        assert_eq!(w("Ubuntu", "/f/Ubuntu-R.ttf"), Some(Weight::Regular));
        assert_eq!(w("Ubuntu", "/f/Ubuntu-B.ttf"), Some(Weight::Bold));
        assert_eq!(w("Cantarell", "/f/Cantarell-VF.otf"), Some(Weight::Variable));
        assert_eq!(w("Source Sans 3", "/f/SourceSans3VF-Upright.otf"), Some(Weight::Variable));
        assert_eq!(w("Source Sans 3", "/f/SourceSans3-Medium.otf"), Some(Weight::Medium));
        assert_eq!(w("DejaVu Sans", "/f/DejaVuSans.ttf"), Some(Weight::Regular));
    }

    #[test]
    fn proprietary_families_are_never_loaded() {
        assert!(is_proprietary("Segoe UI") && is_proprietary("SF Pro Text") && is_proprietary("Arial"));
        assert!(!is_proprietary("Ubuntu Sans") && !is_proprietary("Inter"));
        assert!(find_family_in("Segoe UI", &[PathBuf::from("/f/SegoeUI-Regular.ttf")]).is_none());
        assert!(find_family_in("", &[]).is_none());
    }

    #[cfg(feature = "bundled-fonts")]
    #[test]
    fn finds_static_and_variable_families_in_a_folder() {
        let dir = std::env::temp_dir().join(format!("iurefficient-ui-familias-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        let inter = include_bytes!("../assets/fonts/Inter-Regular.ttf");
        let bold = include_bytes!("../assets/fonts/Inter-Bold.ttf");
        std::fs::write(dir.join("sub/Prueba Sans-Regular.ttf"), inter).unwrap();
        std::fs::write(dir.join("sub/PruebaSans-Bold.ttf"), bold).unwrap();
        std::fs::write(dir.join("Variable[wght].ttf"), inter).unwrap();
        std::fs::write(dir.join("Rota-Regular.ttf"), b"basura").unwrap();
        let mut files = Vec::new();
        walk(&dir, 0, &mut files);
        let s = find_family_in("Prueba Sans", &files).unwrap();
        assert!(s.bold.is_some() && s.medium.is_none() && !s.variable_weight);
        let v = find_family_in("Variable", &files).unwrap();
        assert!(v.variable_weight);
        assert!(find_family_in("Rota", &files).is_none());
        // Se instala sin pánico y queda con su nombre.
        let ctx = egui::Context::default();
        crate::system::install_font_set(&ctx, &v, false);
        for _ in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui.label(egui::RichText::new("Hola").font(crate::theme::bold_font(ui.ctx(), 13.0)));
            });
            out.textures_delta.clear();
        }
        assert_eq!(crate::system::installed_font(&ctx).as_deref(), Some("Variable"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn crisp_text_changes_text_options() {
        let ctx = egui::Context::default();
        set_crisp_text(&ctx, true);
        assert!(!ctx.global_style().visuals.text_options.subpixel_binning);
        set_crisp_text(&ctx, false);
        assert!(ctx.global_style().visuals.text_options.subpixel_binning);
    }

    /// Lee la fuente del escritorio de verdad: `cargo test -- --ignored fuente_del_escritorio`.
    #[test]
    #[ignore]
    fn fuente_del_escritorio() {
        println!("escritorio: {:?}", desktop_font());
        println!("instaladas: {:?}", available_families());
        println!("conjunto: {:?}", desktop_font_set().map(|s| (s.name, s.variable_weight)));
    }
}
