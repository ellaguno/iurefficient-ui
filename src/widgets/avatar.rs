//! Avatar con iniciales en un círculo de color fijo por persona (el color sale del correo, así
//! que la misma persona tiene el mismo color en todas las apps y equipos).

use egui::{Color32, FontId, Painter, Pos2, Response, Sense, Ui, vec2};

/// Colores de los avatares: todos con contraste AA para el texto blanco.
pub const AVATAR_COLORS: [Color32; 10] = [
    Color32::from_rgb(29, 88, 208),
    Color32::from_rgb(17, 40, 76),
    Color32::from_rgb(0, 115, 105),
    Color32::from_rgb(22, 115, 62),
    Color32::from_rgb(112, 62, 176),
    Color32::from_rgb(180, 38, 32),
    Color32::from_rgb(150, 80, 0),
    Color32::from_rgb(176, 24, 92),
    Color32::from_rgb(70, 80, 96),
    Color32::from_rgb(10, 104, 145),
];

/// Hash FNV-1a de 64 bits (estable entre versiones y plataformas, a diferencia de `DefaultHasher`).
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Color del avatar de `key` (el correo; sin importar mayúsculas ni espacios alrededor).
pub fn avatar_color(key: &str) -> Color32 {
    let k = key.trim().to_lowercase();
    let i = (fnv1a(k.as_bytes()) % AVATAR_COLORS.len() as u64) as usize;
    AVATAR_COLORS.get(i).copied().unwrap_or(AVATAR_COLORS[0])
}

/// Iniciales: primera letra del primer y del último nombre; sin nombre, de la parte local del
/// correo («ana.lopez@…» → «AL»); «?» si no hay nada.
pub fn initials(name: &str, email: &str) -> String {
    let first_letter = |w: &str| w.chars().find(|c| c.is_alphanumeric());
    let from_words = |words: Vec<&str>| -> String {
        let letters: Vec<char> = words.iter().filter_map(|w| first_letter(w)).collect();
        match (letters.first(), letters.last()) {
            (Some(a), Some(b)) if letters.len() > 1 => format!("{a}{b}"),
            (Some(a), _) => a.to_string(),
            _ => String::new(),
        }
    };
    // Quita lo que va entre comillas, paréntesis o «<…>».
    let clean: String = name.chars().filter(|c| !matches!(c, '"' | '\'' | '<' | '>' | '(' | ')')).collect();
    let mut out = from_words(clean.split_whitespace().filter(|w| !w.contains('@')).collect());
    if out.is_empty() {
        let local = email.split('@').next().unwrap_or_default();
        out = from_words(local.split(['.', '_', '-', '+']).filter(|w| !w.is_empty()).collect());
    }
    if out.is_empty() { "?".to_string() } else { out.to_uppercase().chars().take(2).collect() }
}

/// Dibuja el avatar centrado en `center`.
pub fn paint_avatar(painter: &Painter, center: Pos2, diameter: f32, name: &str, email: &str) {
    let key = if email.trim().is_empty() { name } else { email };
    painter.circle_filled(center, diameter / 2.0, avatar_color(key));
    let text = initials(name, email);
    let size = (diameter * if text.chars().count() > 1 { 0.38 } else { 0.45 }).max(6.0);
    let galley = painter.layout_no_wrap(text, FontId::proportional(size), Color32::WHITE);
    painter.galley(center - galley.size() / 2.0, galley, Color32::WHITE);
}

/// Avatar como widget; el lector de pantalla lee el nombre (o el correo).
pub fn avatar(ui: &mut Ui, name: &str, email: &str, diameter: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(diameter, diameter), Sense::hover());
    paint_avatar(ui.painter(), rect.center(), diameter, name, email);
    crate::a11y::label(&resp, if name.trim().is_empty() { email } else { name });
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_from_name_or_email() {
        assert_eq!(initials("Ana López", "ana@x.mx"), "AL");
        assert_eq!(initials("ana maría de la o", ""), "AO");
        assert_eq!(initials("Juan", "j@x.mx"), "J");
        assert_eq!(initials("", "ana.lopez@x.mx"), "AL");
        assert_eq!(initials("\"Ñoño\" <n@x.mx>", "n@x.mx"), "Ñ");
        assert_eq!(initials("", ""), "?");
        assert_eq!(initials("   ", "@"), "?");
        assert_eq!(initials("李 小龍", ""), "李小");
    }

    #[test]
    fn color_is_deterministic_and_case_insensitive() {
        assert_eq!(avatar_color("Ana@X.mx "), avatar_color("ana@x.mx"));
        // Valor fijo: si cambia el hash, cambian los colores de todos los usuarios.
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        let distinct: std::collections::HashSet<_> =
            (0..200).map(|i| avatar_color(&format!("persona{i}@x.mx"))).collect();
        assert_eq!(distinct.len(), AVATAR_COLORS.len(), "se usan todos los colores");
    }

    #[test]
    fn white_text_is_readable_on_every_color() {
        for c in AVATAR_COLORS {
            let r = crate::theme::contrast::ratio(Color32::WHITE, c);
            assert!(r >= 4.5, "{c:?}: {r:.2}");
        }
    }
}
