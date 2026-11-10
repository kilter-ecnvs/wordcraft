//! Results from an external checker (LanguageTool, #575).
//!
//! An external checker runs somewhere else (a server) and answers later, so its results are kept
//! here, keyed by the exact paragraph text they were computed for and the language they were
//! checked in. A paragraph whose text has a
//! stored result in its language shows those issues instead of the built-in checks (the built-in checks); any edit
//! changes the text, so a stale result can never land on the wrong words — the paragraph falls
//! back to the built-in checks until the checker answers for the new text.
//!
//! Each stored result has a [`stamp`] (a counter that only rises) that layout folds into its
//! per-paragraph cache key, so a result arriving redraws only its own paragraph.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{OnceLock, RwLock};

use wordcraft_proof::Issue;

/// At most this many paragraphs are remembered; past it the store starts over (results are
/// asked for again as paragraphs are seen).
pub const MAX_ENTRIES: usize = 20_000;

/// At most this many issues are kept per paragraph.
pub const MAX_ISSUES: usize = 1_000;

#[derive(Default)]
struct Store {
    /// (Text, language) → (stamp, issues). The same text in two languages has two results.
    map: HashMap<(String, String), (u64, Vec<Issue>)>,
    next: u64,
}

fn store() -> &'static RwLock<Store> {
    static S: OnceLock<RwLock<Store>> = OnceLock::new();
    S.get_or_init(|| RwLock::new(Store::default()))
}

/// Entries in the store, readable without the lock (the hot path asks this every layout).
static LEN: AtomicUsize = AtomicUsize::new(0);

/// Remember the issues an external checker found in `text` checked in `language`. Issues outside the text, empty, or
/// not on character boundaries are dropped; the rest are sorted by start.
pub fn put(text: &str, language: &str, issues: Vec<Issue>) {
    let mut issues: Vec<Issue> = issues
        .into_iter()
        .filter(|i| i.start < i.end && i.end <= text.len() && text.is_char_boundary(i.start) && text.is_char_boundary(i.end))
        .take(MAX_ISSUES)
        .collect();
    issues.sort_by_key(|i| (i.start, i.end));
    let mut s = store().write().unwrap_or_else(|e| e.into_inner());
    let key = (text.to_string(), language.to_string());
    if s.map.len() >= MAX_ENTRIES && !s.map.contains_key(&key) {
        s.map.clear();
    }
    s.next = s.next.saturating_add(1);
    let stamp = s.next;
    s.map.insert(key, (stamp, issues));
    LEN.store(s.map.len(), Ordering::Relaxed);
}

/// The stored issues for exactly `text` checked in `language`, if any.
pub fn get(text: &str, language: &str) -> Option<Vec<Issue>> {
    if LEN.load(Ordering::Relaxed) == 0 {
        return None;
    }
    store().read().unwrap_or_else(|e| e.into_inner()).map.get(&(text.to_string(), language.to_string())).map(|(_, v)| v.clone())
}

/// Whether a result is stored for exactly `text` checked in `language`. A result in another
/// language (the paragraph's language changed) doesn't count: it is checked again, and kept
/// beside the other.
pub fn has(text: &str, language: &str) -> bool {
    get_stamp(text, language) != 0
}

fn get_stamp(text: &str, language: &str) -> u64 {
    if LEN.load(Ordering::Relaxed) == 0 {
        return 0;
    }
    store().read().unwrap_or_else(|e| e.into_inner()).map.get(&(text.to_string(), language.to_string())).map(|(s, _)| *s).unwrap_or(0)
}

/// Which stored result applies to `text` in `language` (0: none). Changes whenever a new result
/// is stored for it, so layout can key its cache on it.
pub fn stamp(text: &str, language: &str) -> u64 {
    get_stamp(text, language)
}

/// Is `code` shaped like a language code (`en`, `en-US`, `de-DE-x-simple-language`)?
pub fn is_language_code(code: &str) -> bool {
    (2..=35).contains(&code.len())
        && code.starts_with(|c: char| c.is_ascii_alphabetic())
        && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && !code.ends_with('-')
}

/// The language code to check a paragraph in: `mode` is the setting (`auto`, `document` or a
/// code), `para` the paragraph's own language tag.
pub fn language_for(mode: &str, para: Option<&str>) -> String {
    let code = match mode.trim() {
        "" | "auto" => None,
        "document" => para.map(str::trim),
        c => Some(c),
    };
    code.filter(|c| is_language_code(c)).map(str::to_string).unwrap_or_else(|| "auto".to_string())
}

fn mode() -> &'static RwLock<String> {
    static M: OnceLock<RwLock<String>> = OnceLock::new();
    M.get_or_init(|| RwLock::new("auto".to_string()))
}

/// Set the language setting the app checks with, so layout asks for the same result.
pub fn set_mode(m: &str) {
    *mode().write().unwrap_or_else(|e| e.into_inner()) = m.to_string();
}

/// The language the checker uses for a paragraph tagged `para` under the current setting.
pub fn wanted(para: Option<&str>) -> String {
    language_for(&mode().read().unwrap_or_else(|e| e.into_inner()), para)
}

/// Whether anything is stored (cheap; lets callers skip work when no external checker runs).
pub fn is_empty() -> bool {
    LEN.load(Ordering::Relaxed) == 0
}

/// Forget every stored result (the checker was turned off or its settings changed).
pub fn clear() {
    let mut s = store().write().unwrap_or_else(|e| e.into_inner());
    s.map.clear();
    LEN.store(0, Ordering::Relaxed);
}

// craft-proof keeps its user-dictionary and option helpers private, so the few the external
// results need live here.

fn ignored() -> &'static RwLock<std::collections::HashSet<String>> {
    static I: OnceLock<RwLock<std::collections::HashSet<String>>> = OnceLock::new();
    I.get_or_init(Default::default)
}

/// "Ignore All" also hides the word's external spelling marks (call beside `wordcraft_proof::ignore_word`).
pub fn ignore(w: &str) {
    ignored().write().unwrap_or_else(|e| e.into_inner()).insert(w.trim().to_lowercase());
}

/// Whether `w` is in the user dictionary or ignored, any case, edge punctuation trimmed.
fn is_user_word(w: &str) -> bool {
    let w = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'' && c != '’').to_lowercase();
    !w.is_empty()
        && (ignored().read().unwrap_or_else(|e| e.into_inner()).contains(&w)
            || wordcraft_proof::user_dictionary().iter().any(|u| u.to_lowercase() == w))
}

/// The external checker's issues for exactly `text` checked in `language`, if it has answered for it,
/// without spelling issues on words the user added or ignored. `None`: use the built-in checks.
pub fn check(text: &str, language: &str) -> Option<Vec<Issue>> {
    let found = get(text, language)?;
    Some(found.into_iter().filter(|i| i.kind != wordcraft_proof::IssueKind::Spelling || !text.get(i.start..i.end).is_some_and(is_user_word)).collect())
}

/// Whether `opts` says to leave the spelling issue at `a..b` of `text` unmarked (Ignore words in
/// UPPERCASE, words with numbers, Internet and file addresses): an external checker doesn't know these options.
pub fn spelling_ignored_by_options(text: &str, a: usize, b: usize, opts: &wordcraft_proof::Options) -> bool {
    let Some(w) = text.get(a..b) else { return false };
    let w = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'').trim_matches('\'');
    let ts = text.get(..a).and_then(|t| t.char_indices().rev().find(|(_, c)| c.is_whitespace())).map_or(0, |(i, c)| i + c.len_utf8());
    let te = text.get(b..).and_then(|t| t.find(char::is_whitespace)).map_or(text.len(), |i| b + i);
    let token = text.get(ts..te).unwrap_or("");
    w.chars().count() <= 1
        || !w.chars().any(char::is_alphabetic)
        || (opts.ignore_numbers && w.chars().any(|c| c.is_ascii_digit()))
        || (opts.ignore_uppercase && w.chars().any(char::is_uppercase) && !w.chars().any(char::is_lowercase))
        || is_user_word(w)
        || (opts.ignore_internet && (token.contains("://") || token.contains('@') || token.starts_with("www.")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordcraft_proof::{IssueKind, Options, ProofLang};

    fn issue(start: usize, end: usize, kind: IssueKind) -> Issue {
        Issue { start, end, kind, message: "m".into(), suggestions: vec!["x".into()], lang: ProofLang::En }
    }

    #[test]
    fn results_apply_to_their_exact_text_and_language() {
        let text = "Zqxv external checker teh text 575.";
        put(text, "en-US", vec![issue(26, 30, IssueKind::Grammar), issue(22, 25, IssueKind::Spelling), issue(0, 4, IssueKind::Spelling), issue(5, 5, IssueKind::Grammar), issue(30, 999, IssueKind::Grammar)]);
        let got: Vec<(usize, usize)> = check(text, "en-US").unwrap_or_default().iter().map(|i| (i.start, i.end)).collect();
        assert_eq!(got, [(0, 4), (22, 25), (26, 30)]);
        assert!(check(text, "de-DE").is_none());
        assert!(check("Zqxv external checker teh text 575!", "en-US").is_none());
        ignore("Zqxv");
        assert_eq!(check(text, "en-US").unwrap_or_default().len(), 2);
    }

    #[test]
    fn spelling_options_apply_to_external_issues() {
        let text = "NASA b2b see https://exmple.org now";
        let on = Options::default();
        let off = Options { ignore_uppercase: false, ignore_numbers: false, ignore_internet: false };
        for (a, b) in [(0, 4), (5, 8), (13, 19)] {
            assert!(spelling_ignored_by_options(text, a, b, &on), "{a}..{b}");
            assert!(!spelling_ignored_by_options(text, a, b, &off), "{a}..{b}");
        }
        assert!(!spelling_ignored_by_options(text, 9, 12, &on));
    }
}
