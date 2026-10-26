//! Lo que se toma del sistema: modo claro/oscuro del escritorio (con seguimiento en vivo por el
//! portal XDG en Linux), las fuentes y el tamaño del texto.
//!
//! La fuente de la interfaz es Inter (incluida en el binario con la característica
//! `bundled-fonts`, activa por omisión), así que se ve igual en Linux, macOS y Windows. La libre
//! del sistema queda de respaldo, o como principal con [`FontSource::System`]; se busca solo en
//! una lista de fuentes libres por su nombre de archivo (Noto Sans, Ubuntu, DejaVu Sans,
//! Liberation Sans): nunca una de Microsoft o Apple, aunque esté instalada; por eso no se
//! pregunta a fontconfig, que puede devolver cualquiera.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

/// Nombre de la familia en negritas que instala [`install_fonts`].
pub const BOLD_FAMILY: &str = "iurefficient-bold";
/// Familia de peso medio (500): botones, etiquetas de sección, encabezados de tabla.
pub const MEDIUM_FAMILY: &str = "iurefficient-medium";
/// Familia seminegrita (600): no leídos, títulos, nombres. La negrita (700) queda para lo enfático.
pub const SEMIBOLD_FAMILY: &str = "iurefficient-semibold";
/// Familia con cifras de ancho fijo (horas, fechas, conteos en columnas).
pub const TABULAR_FAMILY: &str = "iurefficient-tabular";

/// Modo del escritorio según el portal: 0 sin saber, 1 oscuro, 2 claro.
static SYSTEM_THEME: AtomicU8 = AtomicU8::new(0);

/// ¿El escritorio está en oscuro? `None` mientras no se sepa (o fuera de Linux, o sin la
/// característica `portal`): entonces egui sigue lo que diga la ventana.
pub fn system_dark() -> Option<bool> {
    match SYSTEM_THEME.load(Ordering::Relaxed) {
        1 => Some(true),
        2 => Some(false),
        _ => None,
    }
}

/// Fija el modo del sistema a mano (pruebas, o una plataforma que lo sepa por otra vía).
pub fn set_system_dark(dark: Option<bool>) {
    SYSTEM_THEME.store(
        match dark {
            Some(true) => 1,
            Some(false) => 2,
            None => 0,
        },
        Ordering::Relaxed,
    );
}

/// Lee el modo del escritorio una vez y después cada 2 s en un hilo aparte (GNOME y KDE lo
/// publican en `org.freedesktop.appearance color-scheme`: 1 oscuro, 2 claro, 0 sin preferencia,
/// que GNOME muestra en claro). Cuando cambia, pide un cuadro nuevo a `ctx` para que
/// [`crate::theme::refresh`] lo aplique. Solo arranca un hilo aunque se llame varias veces.
/// Sin la característica `portal` o fuera de Linux no hace nada.
pub fn watch_system_theme(ctx: &egui::Context) {
    #[cfg(all(target_os = "linux", feature = "portal"))]
    {
        static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if let Some(d) = portal::dark() {
            set_system_dark(Some(d));
        }
        if STARTED.swap(true, Ordering::Relaxed) {
            return;
        }
        let ctx = ctx.clone();
        std::thread::Builder::new()
            .name("tema-del-escritorio".into())
            .spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    if let Some(d) = portal::dark()
                        && system_dark() != Some(d)
                    {
                        set_system_dark(Some(d));
                        ctx.request_repaint();
                    }
                }
            })
            .ok();
    }
    #[cfg(not(all(target_os = "linux", feature = "portal")))]
    let _ = ctx;
}

#[cfg(all(target_os = "linux", feature = "portal"))]
mod portal {
    /// Lee `color-scheme` del portal de configuración. `None` si no hay sesión o portal.
    pub(super) fn dark() -> Option<bool> {
        use zbus::zvariant::OwnedValue;
        let conn = zbus::blocking::Connection::session().ok()?;
        let call = |method: &str| {
            conn.call_method(
                Some("org.freedesktop.portal.Desktop"),
                "/org/freedesktop/portal/desktop",
                Some("org.freedesktop.portal.Settings"),
                method,
                &("org.freedesktop.appearance", "color-scheme"),
            )
        };
        // `ReadOne` (portales nuevos) da el valor; `Read` (viejos) lo da dentro de otra variante.
        let value: u32 = match call("ReadOne") {
            Ok(m) => u32::try_from(m.body().deserialize::<OwnedValue>().ok()?).ok()?,
            Err(_) => {
                let outer: OwnedValue = call("Read").ok()?.body().deserialize().ok()?;
                match &*outer {
                    zbus::zvariant::Value::Value(inner) => u32::try_from(&**inner).ok()?,
                    other => u32::try_from(other).ok()?,
                }
            }
        };
        Some(value == 1)
    }
}

/// Fuentes libres que se aceptan, en orden de preferencia: (normal, negritas), nombre exacto.
const FREE_FONTS: [(&str, &str); 5] = [
    ("NotoSans-Regular.ttf", "NotoSans-Bold.ttf"),
    ("Ubuntu-R.ttf", "Ubuntu-B.ttf"),
    ("Ubuntu-Regular.ttf", "Ubuntu-Bold.ttf"),
    ("DejaVuSans.ttf", "DejaVuSans-Bold.ttf"),
    ("LiberationSans-Regular.ttf", "LiberationSans-Bold.ttf"),
];

/// Una fuente libre del sistema: los bytes de la normal y, si está, la negrita de la misma familia.
#[derive(Clone, Debug)]
pub struct SystemFont {
    pub name: &'static str,
    pub regular: Arc<Vec<u8>>,
    pub bold: Option<Arc<Vec<u8>>>,
}

fn font_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = vec![
        PathBuf::from("/usr/share/fonts"),
        PathBuf::from("/usr/local/share/fonts"),
        PathBuf::from("/Library/Fonts"),
        PathBuf::from("C:\\Windows\\Fonts"),
    ];
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from) {
        dirs.push(home.join(".local/share/fonts"));
        dirs.push(home.join(".fonts"));
        dirs.push(home.join("Library/Fonts"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
        dirs.push(local.join("Microsoft\\Windows\\Fonts"));
    }
    dirs
}

fn read_font(path: &Path) -> Option<Vec<u8>> {
    let meta = std::fs::metadata(path).ok()?;
    if meta.len() > 30_000_000 {
        return None;
    }
    std::fs::read(path).ok()
}

/// Busca una fuente libre instalada (por nombre de archivo exacto) en las carpetas habituales.
/// El resultado se recuerda: la búsqueda en disco se hace una sola vez.
pub fn system_font() -> Option<SystemFont> {
    static FOUND: OnceLock<Option<SystemFont>> = OnceLock::new();
    FOUND.get_or_init(|| find_free_font(&font_dirs())).clone()
}

fn find_free_font(dirs: &[PathBuf]) -> Option<SystemFont> {
    for (regular, bold) in FREE_FONTS {
        for d in dirs {
            if let Some(f) = find(d, regular, 0)
                && let Some(bytes) = read_font(&f)
            {
                // La negrita suele estar junto a la normal; si no, se busca en todas las carpetas.
                let bold = f
                    .parent()
                    .map(|p| p.join(bold))
                    .filter(|p| p.exists())
                    .or_else(|| dirs.iter().find_map(|d| find(d, bold, 0)))
                    .and_then(|p| read_font(&p))
                    .map(Arc::new);
                return Some(SystemFont { name: regular, regular: Arc::new(bytes), bold });
            }
        }
    }
    None
}

/// Busca el archivo `name` bajo `dir`, hasta 4 niveles y unos miles de entradas.
fn find(dir: &Path, name: &str, depth: u32) -> Option<PathBuf> {
    if depth > 4 {
        return None;
    }
    let entries = std::fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();
    for e in entries.flatten().take(5_000) {
        let path = e.path();
        if path.file_name().is_some_and(|n| n == name) {
            return Some(path);
        }
        if e.file_type().is_ok_and(|t| t.is_dir()) {
            subdirs.push(path);
        }
    }
    subdirs.into_iter().find_map(|d| find(&d, name, depth + 1))
}

/// Bytes de un archivo de fuente: incrustados en el binario o leídos del disco (compartidos).
#[derive(Clone, Debug)]
pub enum FontBytes {
    Static(&'static [u8]),
    Shared(Arc<Vec<u8>>),
}

impl FontBytes {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            FontBytes::Static(b) => b,
            FontBytes::Shared(v) => v.as_slice(),
        }
    }

    /// ¿Es una fuente TrueType/OpenType que se puede leer y tiene letras latinas o cifras (la
    /// tabular solo trae cifras)? egui entra en pánico con un archivo dañado, así que todo lo que
    /// viene de fuera pasa por aquí antes.
    pub fn is_valid(&self) -> bool {
        use skrifa::MetadataProvider as _;
        skrifa::FontRef::new(self.as_slice()).is_ok_and(|f| {
            let cmap = f.charmap();
            (cmap.map('a').is_some() || cmap.map('0').is_some())
                && f.outline_glyphs().get(cmap.map('0').or(cmap.map('a')).unwrap_or_default()).is_some()
        })
    }

    fn font_data(&self, hinting: Option<bool>) -> egui::FontData {
        let data = match self {
            FontBytes::Static(b) => egui::FontData::from_static(b),
            FontBytes::Shared(v) => egui::FontData::from_blob(v.clone(), 0),
        };
        data.tweak(egui::FontTweak { hinting, ..Default::default() })
    }
}

/// Una familia de interfaz con sus pesos. Solo `regular` es obligatoria: los pesos que falten se
/// toman del más cercano (media → normal, seminegrita → negrita → normal).
#[derive(Clone, Debug)]
pub struct FontSet {
    /// Nombre para mostrar y para los registros («Inter», «NotoSans-Regular.ttf»…).
    pub name: String,
    pub regular: FontBytes,
    pub medium: Option<FontBytes>,
    pub semibold: Option<FontBytes>,
    pub bold: Option<FontBytes>,
    /// Cifras de ancho fijo (horas, fechas, conteos) si la fuente normal las tiene proporcionales.
    /// Basta con las cifras y la puntuación: el resto del texto sale de `regular`.
    pub tabular: Option<FontBytes>,
    /// Ajuste a la rejilla de píxeles (`None`: el de egui, activado).
    pub hinting: Option<bool>,
}

/// Inter 4.1 (rsms, OFL-1.1), la fuente incluida: Regular, Medium, SemiBold, Bold y cifras
/// tabulares (subconjunto de Inter Regular con `tnum` congelado por `assets/fonts/inter-tabular.py`).
#[cfg(feature = "bundled-fonts")]
mod bundled {
    pub(super) const REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
    pub(super) const MEDIUM: &[u8] = include_bytes!("../assets/fonts/Inter-Medium.ttf");
    pub(super) const SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Inter-SemiBold.ttf");
    pub(super) const BOLD: &[u8] = include_bytes!("../assets/fonts/Inter-Bold.ttf");
    pub(super) const TABULAR: &[u8] = include_bytes!("../assets/fonts/InterTabular-Regular.ttf");
}

impl FontSet {
    /// Una familia con un solo peso.
    pub fn new(name: impl Into<String>, regular: FontBytes) -> Self {
        FontSet { name: name.into(), regular, medium: None, semibold: None, bold: None, tabular: None, hinting: None }
    }

    /// La fuente incluida en el binario (Inter), o `None` sin la característica `bundled-fonts`.
    pub fn bundled() -> Option<FontSet> {
        #[cfg(feature = "bundled-fonts")]
        {
            Some(FontSet {
                name: "Inter".into(),
                regular: FontBytes::Static(bundled::REGULAR),
                medium: Some(FontBytes::Static(bundled::MEDIUM)),
                semibold: Some(FontBytes::Static(bundled::SEMIBOLD)),
                bold: Some(FontBytes::Static(bundled::BOLD)),
                tabular: Some(FontBytes::Static(bundled::TABULAR)),
                hinting: None,
            })
        }
        #[cfg(not(feature = "bundled-fonts"))]
        None
    }

    /// La fuente libre del sistema ([`system_font`]) como familia: normal y negrita.
    pub fn system() -> Option<FontSet> {
        system_font().map(|f| FontSet {
            bold: f.bold.map(FontBytes::Shared),
            ..FontSet::new(f.name, FontBytes::Shared(f.regular))
        })
    }

    /// Lee una familia de una carpeta (para cambiar la fuente sin recompilar): toma los archivos
    /// `.ttf`/`.otf` cuyo nombre termina en `-Regular`, `-Medium`, `-SemiBold` (o `-Semibold`),
    /// `-Bold` y, si existe, `Tabular-Regular` para las cifras. Sin una normal válida, `None`.
    pub fn from_dir(dir: &Path) -> Option<FontSet> {
        let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("ttf") || e.eq_ignore_ascii_case("otf"))
            })
            .collect();
        files.sort();
        let stem = |p: &PathBuf| p.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_ascii_lowercase();
        let pick = |suffix: &str, tabular: bool| {
            files
                .iter()
                .find(|p| {
                    let s = stem(p);
                    s.ends_with(suffix) && s.contains("tabular") == tabular
                })
                .and_then(|p| read_font(p))
                .map(|b| FontBytes::Shared(Arc::new(b)))
                .filter(FontBytes::is_valid)
        };
        let regular = pick("-regular", false)?;
        let name = files
            .iter()
            .find(|p| stem(p).ends_with("-regular") && !stem(p).contains("tabular"))
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| s.trim_end_matches("-Regular").to_string())
            .unwrap_or_default();
        Some(FontSet {
            medium: pick("-medium", false),
            semibold: pick("-semibold", false),
            bold: pick("-bold", false),
            tabular: pick("tabular-regular", true),
            ..FontSet::new(name, regular)
        })
    }
}

/// Fuentes que se registran en egui: qué familia se usó y con qué respaldo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontSource {
    /// La incluida (Inter) primero; la libre del sistema y las de egui de respaldo.
    #[default]
    Bundled,
    /// La libre del sistema primero (Noto, Ubuntu, DejaVu, Liberation), como antes de Inter.
    System,
    /// Solo las de egui (capturas de referencia idénticas en todas las máquinas).
    Egui,
}

/// Instala las fuentes de interfaz según `source` y devuelve el nombre de la familia principal
/// (`None` si quedaron las de egui). Si la pedida no está (sin `bundled-fonts`, o sin fuente libre
/// instalada), pasa a la siguiente: incluida → sistema → egui.
pub fn install(ctx: &egui::Context, source: FontSource) -> Option<String> {
    let set = match source {
        FontSource::Bundled => FontSet::bundled().or_else(FontSet::system),
        FontSource::System => FontSet::system().or_else(FontSet::bundled),
        FontSource::Egui => None,
    };
    // La libre del sistema queda de respaldo (símbolos y letras que Inter no tenga).
    let fallback = if source == FontSource::Bundled { FontSet::system() } else { None };
    install_set(ctx, set.as_ref(), fallback.as_ref())
}

/// Instala las fuentes en el contexto. Con la característica `bundled-fonts` (por omisión) la
/// principal es Inter, igual en Linux, macOS y Windows, y `use_system` solo agrega la libre del
/// sistema como respaldo; sin ella, la libre del sistema va primero si `use_system` (el
/// comportamiento anterior). Las de egui quedan siempre al final (símbolos, emoji). Registra
/// [`BOLD_FAMILY`], [`MEDIUM_FAMILY`], [`SEMIBOLD_FAMILY`] y [`TABULAR_FAMILY`]. Devuelve el nombre
/// de la fuente principal («Inter» o el archivo del sistema), si no quedaron las de egui.
pub fn install_fonts(ctx: &egui::Context, use_system: bool) -> Option<&'static str> {
    if let Some(set) = FontSet::bundled() {
        let fallback = if use_system { FontSet::system() } else { None };
        install_set(ctx, Some(&set), fallback.as_ref());
        return Some("Inter");
    }
    let found = if use_system { system_font() } else { None };
    let set = FontSet::system().filter(|_| use_system);
    install_set(ctx, set.as_ref(), None);
    found.map(|f| f.name)
}

/// Instala una familia propia (p. ej. [`FontSet::from_dir`]) con la libre del sistema y las de
/// egui de respaldo. Una fuente que no se pueda leer se ignora (nunca llega a egui).
pub fn install_font_set(ctx: &egui::Context, set: &FontSet, system_fallback: bool) {
    let fallback = if system_fallback { FontSet::system() } else { None };
    install_set(ctx, Some(set), fallback.as_ref());
}

/// Familias con nombre que registra la instalación (además de `Proportional`).
#[derive(Clone, Debug, Default)]
struct Installed {
    name: Option<String>,
}

fn installed_id() -> egui::Id {
    egui::Id::unique("iurefficient-ui/fonts")
}

/// Nombre de la familia principal instalada (`None` si son las de egui o no se instaló nada).
pub fn installed_font(ctx: &egui::Context) -> Option<String> {
    ctx.data(|d| d.get_temp::<Installed>(installed_id())).and_then(|i| i.name)
}

/// ¿Instalaron este kit ([`install_fonts`], [`install`], [`install_font_set`]) la familia con
/// este nombre? Se consulta lo pedido y no `ctx.fonts`, que entra en pánico antes del primer
/// cuadro; las familias con nombre del kit se registran siempre juntas.
pub fn has_family(ctx: &egui::Context, name: &str) -> bool {
    [BOLD_FAMILY, MEDIUM_FAMILY, SEMIBOLD_FAMILY, TABULAR_FAMILY].contains(&name)
        && ctx.data(|d| d.get_temp::<Installed>(installed_id())).is_some()
}

fn install_set(ctx: &egui::Context, set: Option<&FontSet>, fallback: Option<&FontSet>) -> Option<String> {
    let mut fonts = egui::FontDefinitions::default();
    let egui_list = fonts.families.get(&egui::FontFamily::Proportional).cloned().unwrap_or_default();
    // Respaldo común: la libre del sistema (normal y negrita) y luego las de egui.
    let mut add = |key: &str, bytes: &FontBytes, hinting: Option<bool>| -> Option<String> {
        if !bytes.is_valid() {
            return None;
        }
        fonts.font_data.insert(key.into(), Arc::new(bytes.font_data(hinting)));
        Some(key.to_string())
    };
    let (mut fb_regular, mut fb_bold) = (Vec::new(), Vec::new());
    if let Some(f) = fallback.filter(|f| set.is_none_or(|s| s.name != f.name)) {
        fb_regular.extend(add("iurefficient-fallback", &f.regular, f.hinting));
        fb_bold.extend(f.bold.as_ref().and_then(|b| add("iurefficient-fallback-bold", b, f.hinting)));
        fb_bold.extend(fb_regular.iter().cloned());
    }
    let (mut regular, mut medium, mut semibold, mut bold, mut tabular) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut name = None;
    if let Some(s) = set
        && let Some(r) = add("iurefficient-system", &s.regular, s.hinting)
    {
        name = Some(s.name.clone());
        let m = s.medium.as_ref().and_then(|b| add("iurefficient-medium", b, s.hinting));
        let b = s.bold.as_ref().and_then(|b| add("iurefficient-system-bold", b, s.hinting));
        let sb = s.semibold.as_ref().and_then(|b| add("iurefficient-semibold", b, s.hinting)).or_else(|| b.clone());
        let t = s.tabular.as_ref().and_then(|b| add("iurefficient-tabular", b, s.hinting));
        regular.push(r.clone());
        medium.extend(m.into_iter().chain([r.clone()]));
        semibold.extend(sb.into_iter().chain([r.clone()]));
        bold.extend(b.into_iter().chain([r.clone()]));
        tabular.extend(t.into_iter().chain([r]));
    }
    let chain = |head: Vec<String>, fb: &[String]| -> Vec<String> {
        let mut v = head;
        for k in fb.iter().chain(egui_list.iter()) {
            if !v.contains(k) {
                v.push(k.clone());
            }
        }
        v
    };
    fonts.families.insert(egui::FontFamily::Proportional, chain(regular.clone(), &fb_regular));
    fonts.families.insert(egui::FontFamily::Name(MEDIUM_FAMILY.into()), chain(medium, &fb_regular));
    fonts.families.insert(egui::FontFamily::Name(SEMIBOLD_FAMILY.into()), chain(semibold, &fb_bold));
    fonts.families.insert(egui::FontFamily::Name(BOLD_FAMILY.into()), chain(bold, &fb_bold));
    fonts.families.insert(egui::FontFamily::Name(TABULAR_FAMILY.into()), chain(tabular, &fb_regular));
    ctx.set_fonts(fonts);
    ctx.data_mut(|d| d.insert_temp(installed_id(), Installed { name: name.clone() }));
    // Los títulos (`TextStyle::Heading`) en seminegrita, como en macOS.
    crate::theme::use_semibold_headings(ctx);
    name
}

/// Escala del texto y de toda la interfaz (1.0 = 100 %), acotada a 80 %–200 %.
pub fn set_text_scale(ctx: &egui::Context, scale: f32) {
    ctx.set_zoom_factor(clamp_scale(scale));
}

/// La escala aceptada: 80 %–200 %; un valor inválido es 100 %.
pub fn clamp_scale(scale: f32) -> f32 {
    if scale.is_finite() { scale.clamp(0.8, 2.0) } else { 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_free_fonts_are_considered() {
        for (r, b) in FREE_FONTS {
            for f in [r, b] {
                let f = f.to_lowercase();
                assert!(!f.contains("segoe") && !f.contains("calibri") && !f.contains("arial") && !f.contains("sf"));
            }
        }
        let dir = std::env::temp_dir().join(format!("iurefficient-ui-fuentes-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("a/b")).unwrap();
        std::fs::write(dir.join("a/b/DejaVuSans.ttf"), b"x").unwrap();
        std::fs::write(dir.join("a/b/DejaVuSans-Bold.ttf"), b"y").unwrap();
        std::fs::write(dir.join("segoeui.ttf"), b"x").unwrap();
        assert_eq!(find(&dir, "DejaVuSans.ttf", 0), Some(dir.join("a/b/DejaVuSans.ttf")));
        assert_eq!(find(&dir, "NotoSans-Regular.ttf", 0), None);
        let f = find_free_font(std::slice::from_ref(&dir)).unwrap();
        assert_eq!(f.name, "DejaVuSans.ttf");
        assert_eq!(f.bold.as_deref().map(Vec::as_slice), Some(&b"y"[..]));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Dos cuadros: en el segundo egui ya cargó las fuentes pedidas en el primero.
    fn two_frames(ctx: &egui::Context, mut f: impl FnMut(&egui::Context)) {
        for _ in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| f(ui.ctx()));
            out.textures_delta.clear();
        }
    }

    #[test]
    fn install_fonts_registers_all_weights_and_never_panics() {
        let ctx = egui::Context::default();
        let name = install_fonts(&ctx, false);
        assert_eq!(name.is_some(), cfg!(feature = "bundled-fonts"));
        two_frames(&ctx, |ctx| {
            let families = ctx.fonts(|f| f.families());
            for n in [BOLD_FAMILY, MEDIUM_FAMILY, SEMIBOLD_FAMILY, TABULAR_FAMILY] {
                assert!(families.contains(&egui::FontFamily::Name(n.into())), "falta {n}");
                assert!(has_family(ctx, n));
                // Español, símbolos y una letra que Inter no tiene (sale del respaldo de egui).
                let g = ctx.fonts_mut(|f| {
                    f.layout_no_wrap(
                        "¿Año? ¡Sí! ñ á é — 10:42 €".into(),
                        egui::FontId::new(13.0, egui::FontFamily::Name(n.into())),
                        egui::Color32::WHITE,
                    )
                });
                assert!(g.size().x > 50.0);
            }
        });
        assert_eq!(installed_font(&ctx).is_some(), cfg!(feature = "bundled-fonts"));
        // Las otras fuentes: la del sistema (si hay) y solo las de egui.
        let _ = install(&ctx, FontSource::System);
        assert_eq!(install(&ctx, FontSource::Egui), None);
        two_frames(&ctx, |ctx| {
            assert!(ctx.fonts(|f| f.families().contains(&egui::FontFamily::Name(SEMIBOLD_FAMILY.into()))));
        });
    }

    #[cfg(feature = "bundled-fonts")]
    #[test]
    fn bundled_inter_is_valid_with_tabular_digits() {
        use skrifa::MetadataProvider as _;
        let set = FontSet::bundled().unwrap();
        assert_eq!(set.name, "Inter");
        for b in
            [Some(&set.regular), set.medium.as_ref(), set.semibold.as_ref(), set.bold.as_ref(), set.tabular.as_ref()]
        {
            assert!(b.unwrap().is_valid());
        }
        let advances = |bytes: &FontBytes| -> Vec<f32> {
            let f = skrifa::FontRef::new(bytes.as_slice()).unwrap();
            let m = f.glyph_metrics(skrifa::instance::Size::new(1000.0), skrifa::instance::LocationRef::default());
            "0123456789".chars().map(|c| m.advance_width(f.charmap().map(c).unwrap()).unwrap()).collect()
        };
        let tab = advances(set.tabular.as_ref().unwrap());
        assert!(tab.iter().all(|w| (w - tab[0]).abs() < 0.01), "cifras tabulares: {tab:?}");
        // Inter trae cifras proporcionales: por eso existe la familia tabular.
        let reg = advances(&set.regular);
        assert!(reg.iter().any(|w| (w - reg[0]).abs() > 1.0));
    }

    #[test]
    fn bad_font_files_are_rejected_not_panicking() {
        assert!(!FontBytes::Static(b"no es una fuente").is_valid());
        assert!(!FontBytes::Shared(Arc::new(Vec::new())).is_valid());
        let dir = std::env::temp_dir().join(format!("iurefficient-ui-familia-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Rota-Regular.ttf"), b"basura").unwrap();
        assert!(FontSet::from_dir(&dir).is_none());
        // Una familia propia con una negrita dañada: se instala sin ella y sin pánico.
        let mut set = FontSet::new("Rota", FontBytes::Static(b"basura"));
        set.bold = Some(FontBytes::Static(b"basura"));
        let ctx = egui::Context::default();
        install_font_set(&ctx, &set, false);
        two_frames(&ctx, |ctx| {
            let _ = ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    "Hola".into(),
                    egui::FontId::new(13.0, egui::FontFamily::Name(BOLD_FAMILY.into())),
                    egui::Color32::WHITE,
                )
            });
        });
        assert_eq!(installed_font(&ctx), None);
        #[cfg(feature = "bundled-fonts")]
        {
            for (n, b) in [
                ("Prueba-Regular.ttf", bundled::REGULAR),
                ("Prueba-SemiBold.ttf", bundled::SEMIBOLD),
                ("PruebaTabular-Regular.ttf", bundled::TABULAR),
            ] {
                std::fs::write(dir.join(n), b).unwrap();
            }
            std::fs::remove_file(dir.join("Rota-Regular.ttf")).unwrap();
            let set = FontSet::from_dir(&dir).unwrap();
            assert_eq!(set.name, "Prueba");
            assert!(set.semibold.is_some() && set.tabular.is_some() && set.bold.is_none() && set.medium.is_none());
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn text_scale_is_clamped() {
        assert_eq!(clamp_scale(9.0), 2.0);
        assert_eq!(clamp_scale(0.1), 0.8);
        assert_eq!(clamp_scale(1.25), 1.25);
        assert_eq!(clamp_scale(f32::NAN), 1.0);
    }

    /// Lee el modo del escritorio de verdad (necesita una sesión gráfica):
    /// `cargo test -- --ignored lee_el_tema`.
    #[cfg(all(target_os = "linux", feature = "portal"))]
    #[test]
    #[ignore]
    fn lee_el_tema_del_escritorio() {
        println!("oscuro: {:?}", portal::dark());
        assert!(portal::dark().is_some());
    }
}
