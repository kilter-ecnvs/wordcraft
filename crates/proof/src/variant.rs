//! US and British spellings of the same word (`assets/spelling/en-us-gb.txt`): English (United
//! States) flags the British form (`colour`), English (United Kingdom) the US one (`color`), and
//! suggests the other. English with no region, or another region, accepts both.

use std::collections::HashMap;
use std::sync::OnceLock;

const PAIRS: &str = include_str!("../../../assets/spelling/en-us-gb.txt");

/// Endings derived from each listed pair. Words ending in `l` take only the consonant ones:
/// their -ed/-ing forms double the l differently (travelled), so the list spells those out.
const SUFFIXES: [&str; 10] = ["", "s", "ly", "ful", "less", "ed", "ing", "er", "ers", "able"];
const CONSONANT_SUFFIXES: usize = 5;

/// `base` + `suffix`, a final `e` dropped before a vowel ending (centre → centred, analyse →
/// analysing).
fn inflect(base: &str, suffix: &str) -> String {
    match base.strip_suffix('e') {
        Some(stem) if suffix.starts_with(['a', 'e', 'i', 'o', 'u']) => format!("{stem}{suffix}"),
        _ => format!("{base}{suffix}"),
    }
}

struct Maps {
    us_to_gb: HashMap<String, String>,
    gb_to_us: HashMap<String, String>,
}

fn maps() -> &'static Maps {
    static M: OnceLock<Maps> = OnceLock::new();
    M.get_or_init(|| {
        let mut m = Maps { us_to_gb: HashMap::new(), gb_to_us: HashMap::new() };
        for line in PAIRS.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let mut it = line.split_whitespace();
            let (Some(us), Some(gb)) = (it.next(), it.next()) else { continue };
            let n = if us.ends_with('l') || gb.ends_with('l') { CONSONANT_SUFFIXES } else { SUFFIXES.len() };
            for suffix in SUFFIXES.iter().take(n) {
                let (u, g) = (inflect(us, suffix), inflect(gb, suffix));
                m.us_to_gb.entry(u.clone()).or_insert_with(|| g.clone());
                m.gb_to_us.entry(g).or_insert(u);
            }
        }
        m
    })
}

/// The British spelling of a lowercase US-only word (`color` → `colour`).
pub fn british_of(us: &str) -> Option<&'static str> {
    maps().us_to_gb.get(us).map(String::as_str)
}

/// The US spelling of a lowercase British-only word (`colour` → `color`).
pub fn us_of(gb: &str) -> Option<&'static str> {
    maps().gb_to_us.get(gb).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_and_their_inflections() {
        assert_eq!(british_of("color"), Some("colour"));
        assert_eq!(british_of("colors"), Some("colours"));
        assert_eq!(british_of("centered"), Some("centred"));
        assert_eq!(british_of("centering"), Some("centring"));
        assert_eq!(british_of("analyzed"), Some("analysed"));
        assert_eq!(british_of("traveled"), Some("travelled"));
        assert_eq!(us_of("neighbours"), Some("neighbors"));
        assert_eq!(us_of("aluminium"), Some("aluminum"));
        // Shared spellings are never one region's only.
        for w in ["enrolled", "program", "organize", "organise", "grey", "theatre", "the"] {
            assert_eq!((british_of(w), us_of(w)), (None, None), "{w}");
        }
    }
}
