//! Lo que se toma del sistema: modo claro/oscuro del escritorio (con seguimiento en vivo por el
//! portal XDG en Linux), la fuente libre instalada y el tamaño del texto.
//!
//! La fuente se busca solo en una lista de fuentes libres por su nombre de archivo (Noto Sans,
//! Ubuntu, DejaVu Sans, Liberation Sans): nunca una de Microsoft o Apple, aunque esté instalada;
//! por eso no se pregunta a fontconfig, que puede devolver cualquiera.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

/// Nombre de la familia en negritas que instala [`install_fonts`].
pub const BOLD_FAMILY: &str = "iurefficient-bold";

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

/// Instala las fuentes en el contexto: la libre del sistema primero (si `use_system` y se
/// encuentra) y las de egui como respaldo (símbolos, emoji). Registra siempre la familia
/// [`BOLD_FAMILY`] (con la negrita libre, o la normal si no hay). Devuelve el nombre del archivo
/// de la fuente del sistema usada, si se usó una.
pub fn install_fonts(ctx: &egui::Context, use_system: bool) -> Option<&'static str> {
    let mut fonts = egui::FontDefinitions::default();
    let found = if use_system { system_font() } else { None };
    let proportional = fonts.families.get(&egui::FontFamily::Proportional).cloned().unwrap_or_default();
    let mut bold_list = proportional.clone();
    if let Some(f) = &found {
        fonts
            .font_data
            .insert("iurefficient-system".into(), Arc::new(egui::FontData::from_owned((*f.regular).clone())));
        if let Some(list) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            list.insert(0, "iurefficient-system".into());
        }
        bold_list.insert(0, "iurefficient-system".into());
        if let Some(b) = &f.bold {
            fonts
                .font_data
                .insert("iurefficient-system-bold".into(), Arc::new(egui::FontData::from_owned((**b).clone())));
            bold_list.insert(0, "iurefficient-system-bold".into());
        }
    }
    fonts.families.insert(egui::FontFamily::Name(BOLD_FAMILY.into()), bold_list);
    ctx.set_fonts(fonts);
    found.map(|f| f.name)
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
