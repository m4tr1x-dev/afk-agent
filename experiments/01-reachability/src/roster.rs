//! The roster, and the refusal.
//!
//! `ADR-0025` excludes competitive multiplayer, and the terms of some titles
//! prohibit automation outright. Both rules lived in prose, and a rule of that
//! kind is one somebody eventually works around at two in the morning with a
//! good reason — not through bad faith, but because the rule is somewhere else
//! and the probe is right here.
//!
//! So this refuses. `tests/roster.toml` carries `synthesis_allowed` per title,
//! and a probe that synthesises input will not start against a title where it
//! is false, or against a title the roster does not list at all.
//!
//! **Unlisted is a refusal, not a default.** A roster that silently permits
//! anything it has not heard of permits everything, since a typo in a name is
//! indistinguishable from a title nobody considered.
//!
//! The parser is deliberately small and hand-written. This crate has one
//! dependency — the real `afk-input`, because a probe with its own input path
//! would validate something the product does not send — and a supply-chain
//! decision is not worth making for eight keys.

use std::path::Path;

/// Why a title may not be used.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// The roster does not list it.
    Unlisted,
    /// The roster lists it and forbids synthesis.
    Forbidden {
        /// What the roster says about its terms.
        terms: String,
    },
    /// The roster could not be read.
    Unreadable(String),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unlisted => write!(
                f,
                "this title is not in tests/roster.toml. Add it with a \
                 synthesis_allowed value before pointing a probe at it; an \
                 unlisted title is refused rather than assumed"
            ),
            Self::Forbidden { terms } => write!(
                f,
                "tests/roster.toml sets synthesis_allowed = false for this \
                 title (terms: {terms}). The probe will not synthesise input \
                 against it"
            ),
            Self::Unreadable(why) => write!(f, "tests/roster.toml: {why}"),
        }
    }
}

/// Decide whether synthesis is permitted against `title`.
///
/// # Errors
///
/// Returns the reason it is not.
pub(crate) fn permits_synthesis(roster: &Path, title: &str) -> Result<(), Refusal> {
    let text =
        std::fs::read_to_string(roster).map_err(|error| Refusal::Unreadable(error.to_string()))?;
    match entry(&text, title) {
        None => Err(Refusal::Unlisted),
        Some(entry) if entry.allowed => Ok(()),
        Some(entry) => Err(Refusal::Forbidden { terms: entry.terms }),
    }
}

/// One roster row, reduced to what the refusal needs.
struct Entry {
    allowed: bool,
    terms: String,
}

/// Find a title's row.
///
/// Reads `[[title]]` blocks, taking `name`, `synthesis_allowed` and `terms`.
/// Anything else in the file is ignored, including the multi-line `notes`
/// strings, because nothing here needs them.
fn entry(text: &str, title: &str) -> Option<Entry> {
    let mut in_block = false;
    let mut name = String::new();
    let mut allowed = false;
    let mut terms = String::new();
    let mut in_multiline = false;

    for line in text.lines() {
        let trimmed = line.trim();

        // A triple-quoted value spans lines and can contain anything,
        // including something that looks like a key. Skip its body wholesale.
        if in_multiline {
            if trimmed.ends_with("\"\"\"") {
                in_multiline = false;
            }
            continue;
        }

        if trimmed == "[[title]]" {
            if in_block && name == title {
                return Some(Entry { allowed, terms });
            }
            in_block = true;
            name.clear();
            allowed = false;
            terms.clear();
            continue;
        }

        if !in_block || trimmed.starts_with('#') {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        if value.starts_with("\"\"\"") && !value.trim_end().ends_with("\"\"\"") {
            in_multiline = true;
            continue;
        }

        let unquoted = value.trim_matches('"');
        match key {
            "name" => unquoted.clone_into(&mut name),
            "terms" => unquoted.clone_into(&mut terms),
            "synthesis_allowed" => allowed = value == "true",
            _ => {}
        }
    }

    if in_block && name == title {
        Some(Entry { allowed, terms })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    const ROSTER: &str = r#"
schema_version = 1

[[title]]
name = "xonotic"
synthesis_allowed = true
terms = "silent"
notes = """
Multi-line prose that mentions synthesis_allowed = false, because a note about
another title's status is exactly the sort of thing a naive parser reads as a
key and this one must not.
"""

[[title]]
name = "roblox"
synthesis_allowed = false
terms = "prohibits"
notes = "Capture only."
"#;

    #[test]
    fn a_permitted_title_is_permitted() {
        let found = entry(ROSTER, "xonotic").expect("xonotic is listed");
        assert!(found.allowed);
    }

    #[test]
    fn a_forbidden_title_is_forbidden() {
        let found = entry(ROSTER, "roblox").expect("roblox is listed");
        assert!(!found.allowed);
        assert_eq!(found.terms, "prohibits");
    }

    #[test]
    fn an_unlisted_title_is_not_found() {
        // Unlisted has to be a refusal rather than a default. A roster that
        // silently permits what it has not heard of permits everything,
        // because a typo is indistinguishable from an omission.
        assert!(entry(ROSTER, "some-game-nobody-added").is_none());
    }

    #[test]
    fn prose_inside_a_multi_line_note_is_not_read_as_a_key() {
        // The xonotic note contains the literal text `synthesis_allowed =
        // false`. A parser that reads it flips a permitted title to forbidden,
        // which fails safe — and would still be a bug, because the next such
        // note might flip one the other way.
        let found = entry(ROSTER, "xonotic").expect("xonotic is listed");
        assert!(found.allowed, "the note's prose leaked into the parse");
    }

    #[test]
    fn the_last_block_in_the_file_is_reachable() {
        // Off-by-one: a parser that only emits an entry when it meets the
        // *next* [[title]] header loses the last one, and the last one is
        // whatever was added most recently.
        assert!(entry(ROSTER, "roblox").is_some());
    }
}
