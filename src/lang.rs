//! Idioma de la interfaz (igual que `iureproject_engine::lang`): inglés por omisión, español si el
//! sistema está en español. Se detecta por `LC_ALL`/`LC_MESSAGES`/`LANG` y, si no hay ninguna (lo
//! normal en Windows y macOS), por la configuración regional del sistema (`sys-locale`).

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Es,
}

/// 0 sin detectar, 1 inglés, 2 español.
static LANG: AtomicU8 = AtomicU8::new(0);

/// Fija el idioma (preferencia del usuario).
pub fn set_lang(l: Lang) {
    LANG.store(if l == Lang::Es { 2 } else { 1 }, Ordering::Relaxed);
}

/// El idioma actual; la primera vez lo detecta del sistema.
pub fn lang() -> Lang {
    match LANG.load(Ordering::Relaxed) {
        1 => Lang::En,
        2 => Lang::Es,
        _ => {
            let detected = detect();
            set_lang(detected);
            detected
        }
    }
}

/// El idioma del sistema, sin mirar la preferencia guardada.
pub fn detect() -> Lang {
    let env = ["LC_ALL", "LC_MESSAGES", "LANG"].iter().filter_map(|k| std::env::var(k).ok()).find(|v| !v.is_empty());
    let tag = env.or_else(sys_locale::get_locale).unwrap_or_default();
    from_tag(&tag)
}

/// Idioma de una etiqueta como `es_MX.UTF-8`, `es-419` o `en-US`.
pub fn from_tag(tag: &str) -> Lang {
    let t = tag.trim().to_ascii_lowercase();
    if t == "es" || t.starts_with("es_") || t.starts_with("es-") || t.starts_with("es.") { Lang::Es } else { Lang::En }
}

/// Elige el texto según el idioma: `pick("Save", "Guardar")`.
pub fn pick<'a>(en: &'a str, es: &'a str) -> &'a str {
    if lang() == Lang::Es { es } else { en }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_are_recognized() {
        assert_eq!(from_tag("es_MX.UTF-8"), Lang::Es);
        assert_eq!(from_tag("es-419"), Lang::Es);
        assert_eq!(from_tag("ES"), Lang::Es);
        assert_eq!(from_tag("en_US.UTF-8"), Lang::En);
        assert_eq!(from_tag("C"), Lang::En);
        assert_eq!(from_tag("estonian"), Lang::En);
        assert_eq!(from_tag(""), Lang::En);
    }
}
