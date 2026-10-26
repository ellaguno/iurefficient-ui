//! Búsqueda difusa (paleta de comandos, filtros rápidos): sin importar mayúsculas ni acentos, las
//! letras de cada palabra buscada tienen que aparecer en orden, y pesa más lo que empieza el texto
//! o una palabra y lo que va seguido. Así «nm» encuentra «Nuevo mensaje» antes que «Unmark», y
//! «vin» pone «Vincular» antes que «Desvincular».
//!
//! Es programación dinámica O(palabra × texto) por elemento, con textos y palabras acotados.

/// Largo máximo del texto que se compara (en caracteres).
const MAX_TEXT: usize = 256;
/// Cuántas palabras de la búsqueda se consideran.
const MAX_TOKENS: usize = 16;
/// Largo máximo de cada palabra buscada.
const MAX_TOKEN: usize = 32;

const MATCH: i32 = 4;
const START_BONUS: i32 = 20;
const WORD_BONUS: i32 = 12;
const CONSECUTIVE_BONUS: i32 = 10;
const GAP_PENALTY: i32 = 2;

/// Minúsculas y sin acentos, para comparar como lo escribe la gente.
pub fn fold(s: &str) -> String {
    s.chars().flat_map(char::to_lowercase).map(fold_char).collect()
}

fn fold_char(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        c => c,
    }
}

/// ¿La posición `j` empieza una palabra?
fn boundary(text: &[char], j: usize) -> bool {
    j == 0 || text.get(j.wrapping_sub(1)).is_some_and(|c| !c.is_alphanumeric())
}

/// Puntaje de una palabra (ya plegada) contra un texto (ya plegado); `None` si sus letras no
/// aparecen en orden.
pub fn token_score(token: &[char], text: &[char]) -> Option<i32> {
    let token = token.get(..token.len().min(MAX_TOKEN)).unwrap_or_default();
    let text = text.get(..text.len().min(MAX_TEXT)).unwrap_or_default();
    if token.is_empty() {
        return Some(0);
    }
    let n = text.len();
    if n < token.len() {
        return None;
    }
    let bonus = |j: usize| {
        if j == 0 {
            START_BONUS
        } else if boundary(text, j) {
            WORD_BONUS
        } else {
            0
        }
    };
    // prev[j]: mejor puntaje con la letra anterior de `token` en la posición j.
    let mut prev: Vec<Option<i32>> = vec![None; n];
    let mut cur: Vec<Option<i32>> = vec![None; n];
    for (i, &tc) in token.iter().enumerate() {
        // Mejor `prev[j'] + GAP·j'` con j' ≤ j − 2 (para el salto lineal).
        let mut best_far: Option<i32> = None;
        for j in 0..n {
            if j >= 2
                && let Some(Some(v)) = prev.get(j - 2)
            {
                let cand = v + GAP_PENALTY * (j as i32 - 2);
                best_far = Some(best_far.map_or(cand, |b: i32| b.max(cand)));
            }
            let here = if text.get(j) == Some(&tc) {
                if i == 0 {
                    // Empezar tarde cuesta un poco (hasta 5).
                    Some(MATCH + bonus(j) - (j as i32).min(20) / 4)
                } else {
                    let consecutive =
                        j.checked_sub(1).and_then(|p| prev.get(p).copied().flatten()).map(|v| v + CONSECUTIVE_BONUS);
                    // prev[j'] − GAP·(j − j' − 1)  =  (prev[j'] + GAP·j') − GAP·(j − 1)
                    let far = best_far.map(|b| b - GAP_PENALTY * (j as i32 - 1));
                    let base = match (consecutive, far) {
                        (Some(a), Some(b)) => Some(a.max(b)),
                        (a, b) => a.or(b),
                    };
                    base.map(|b| b + MATCH + bonus(j))
                }
            } else {
                None
            };
            if let Some(c) = cur.get_mut(j) {
                *c = here;
            }
        }
        std::mem::swap(&mut prev, &mut cur);
        cur.iter_mut().for_each(|c| *c = None);
    }
    prev.into_iter().flatten().max()
}

/// Puntaje de `text` para la búsqueda `query` (sin plegar): cada palabra de la búsqueda tiene que
/// coincidir, en cualquier orden; a igualdad gana el texto más corto. `None` si no coincide.
pub fn score(query: &str, text: &str) -> Option<i32> {
    let t: Vec<char> = fold(text).chars().collect();
    let mut total = 0;
    for w in fold(query).split_whitespace().take(MAX_TOKENS) {
        let tok: Vec<char> = w.chars().collect();
        total += token_score(&tok, &t)?;
    }
    Some(total - (t.len().min(MAX_TEXT) as i32) / 16)
}

/// Como [`score`], pero cada palabra puede coincidir con el texto principal o, con menos peso,
/// con el secundario (palabras clave, la otra lengua, la ubicación).
pub fn score_with_extra(query: &str, text: &str, extra: &str) -> Option<i32> {
    let t: Vec<char> = fold(text).chars().collect();
    let e: Vec<char> = fold(extra).chars().collect();
    let mut total = 0;
    for w in fold(query).split_whitespace().take(MAX_TOKENS) {
        let tok: Vec<char> = w.chars().collect();
        let main = token_score(&tok, &t);
        let alt = token_score(&tok, &e).map(|s| s / 2 - 4);
        total += match (main, alt) {
            (Some(a), Some(b)) => a.max(b),
            (a, b) => a.or(b)?,
        };
    }
    Some(total - (t.len().min(MAX_TEXT) as i32) / 16)
}

/// Ordena `texts` por puntaje para `query` (de mejor a peor) y devuelve los índices que
/// coinciden, como mucho `limit`. Con la búsqueda vacía devuelve los primeros en su orden.
pub fn rank<'a>(query: &str, texts: impl IntoIterator<Item = &'a str>, limit: usize) -> Vec<usize> {
    let texts = texts.into_iter();
    if query.trim().is_empty() {
        return texts.enumerate().take(limit).map(|(i, _)| i).collect();
    }
    let mut scored: Vec<(i32, usize)> =
        texts.enumerate().filter_map(|(i, t)| score(query, t).map(|s| (s, i))).collect();
    // Estable: a igualdad de puntaje se respeta el orden original.
    scored.sort_by_key(|(s, i)| (std::cmp::Reverse(*s), *i));
    scored.into_iter().take(limit).map(|(_, i)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding_ignores_accents_and_case() {
        assert_eq!(fold("Línea BASE ñ Ç"), "linea base n c");
    }

    #[test]
    fn matches_need_letters_in_order() {
        assert!(score("linea", "Establecer línea base").is_some());
        assert!(score("base linea", "Establecer línea base").is_some(), "el orden de las palabras no importa");
        assert!(score("vuelo", "Establecer línea base").is_none());
        assert!(score("esb", "Establecer").is_some());
        assert!(score("bse", "Establecer").is_none());
        assert_eq!(score("", "algo"), Some(0));
        assert!(score("x", "").is_none());
    }

    #[test]
    fn word_starts_and_runs_rank_higher() {
        let s = |q, t| score(q, t).unwrap_or(i32::MIN);
        assert!(s("vin", "Vincular") > s("vin", "Desvincular"), "al principio cuenta más");
        assert!(s("nm", "Nuevo mensaje") > s("nm", "Unmark"), "iniciales de palabras");
        assert!(s("mark", "Unmark") > s("mark", "Make all read and keep"), "seguido gana a disperso");
        assert!(s("resp", "Responder") > s("resp", "Corresponder"));
        assert!(s("rpl all", "Reply all") > s("rpl all", "Report all labels"));
        assert!(s("arch", "Archivar") > s("arch", "Buscar archivos adjuntos"), "a igualdad, el más corto");
    }

    #[test]
    fn rank_orders_and_limits() {
        let items = ["Desvincular", "Vincular", "Nuevo mensaje", "Vista previa"];
        assert_eq!(rank("vin", items, 10), vec![1, 0]);
        assert_eq!(rank("v", items, 1).len(), 1);
        assert_eq!(rank("  ", items, 3), vec![0, 1, 2]);
        assert!(rank("zzz", items, 10).is_empty());
    }

    #[test]
    fn extra_text_matches_with_less_weight() {
        let a = score_with_extra("reply", "Responder", "reply").unwrap();
        let b = score_with_extra("reply", "Reply", "").unwrap();
        assert!(b > a);
        assert!(score_with_extra("qwe", "Responder", "reply").is_none());
    }

    #[test]
    fn hostile_input_is_bounded() {
        let long: String = "a".repeat(100_000);
        assert!(score(&long, &long).is_some());
        assert!(score("a", &long).is_some());
    }
}
