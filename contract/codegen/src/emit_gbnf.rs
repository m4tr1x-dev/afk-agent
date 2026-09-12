//! Emit the grammar that constrains model output.
//!
//! `ADR-0013`: output is restricted to a grammar regenerated every tactical
//! tick, describing exactly what is valid at that moment. This emitter produces
//! the half that does not change within a session — the tool names and their
//! parameter shapes — and leaves a hole where the enumerations go.
//!
//! The hole is the point. A reference to a mark that is not on screen has to be
//! impossible rather than unlikely, and that is only true if the alternation is
//! rebuilt from the marks actually present. `afk-grammar` splices them in at run
//! time; this file is the part worth caching.
//!
//! Both halves come from the same declarations as the tool types, so the grammar
//! and the validator cannot disagree about what a tool takes.

use std::fmt::Write as _;

use crate::ast::Schema;

/// Generate the invariant portion of the grammar.
#[must_use]
pub fn emit(schema: &Schema) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# Generated from the tool declarations. Do not edit.");
    let _ = writeln!(out, "#");
    let _ = writeln!(
        out,
        "# The invariant half of the grammar. `mark` and `cell` are left"
    );
    let _ = writeln!(
        out,
        "# undefined on purpose: afk-grammar fills them each tick from the"
    );
    let _ = writeln!(
        out,
        "# marks actually present, which is what makes an unavailable target"
    );
    let _ = writeln!(out, "# inexpressible rather than merely unlikely.");
    let _ = writeln!(out);

    let roots: Vec<String> = schema
        .tool
        .iter()
        .map(|t| format!("{}-call", t.name))
        .collect();
    let _ = writeln!(out, "root ::= {}", roots.join(" | "));
    let _ = writeln!(out);

    for tool in &schema.tool {
        if !tool.doc.is_empty() {
            let _ = writeln!(out, "# {}", tool.doc);
        }

        let mut parts = vec![terminal(&format!("{{\"tool\":\"{}\"", tool.name))];
        for param in &tool.param {
            parts.push(terminal(&format!(",\"{}\":", param.name)));
            parts.push(format!("{}-{}", tool.name, param.name));
        }
        parts.push(terminal("}"));
        let _ = writeln!(out, "{}-call ::= {}", tool.name, parts.join(" "));

        for param in &tool.param {
            let rule = rule_for(param);
            if !param.doc.is_empty() {
                let _ = writeln!(out, "# {}: {}", param.name, param.doc);
            }
            let _ = writeln!(out, "{}-{} ::= {rule}", tool.name, param.name);
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "unsigned ::= [0-9]+");
    let _ = writeln!(out, "signed ::= {}? [0-9]+", terminal("-"));
    let _ = writeln!(
        out,
        "decimal ::= {}? [0-9]+ ({} [0-9]+)?",
        terminal("-"),
        terminal(".")
    );
    let _ = writeln!(
        out,
        "quoted-string ::= {0} ([^\"\\\\] | {1} .)* {0}",
        terminal("\""),
        terminal("\\")
    );
    out
}

/// The right-hand side for one parameter.
fn rule_for(param: &crate::ast::Param) -> String {
    match param.kind.as_str() {
        // Left undefined here; spliced in per tick.
        "mark" => "mark".to_owned(),
        "cell" => "cell".to_owned(),
        "enum" => param
            .values
            .iter()
            .map(|value| terminal(&format!("\"{value}\"")))
            .collect::<Vec<_>>()
            .join(" | "),
        "string" => "quoted-string".to_owned(),
        "bool" => format!("{} | {}", terminal("true"), terminal("false")),
        kind if kind.starts_with('i') => "signed".to_owned(),
        kind if kind.starts_with('f') => "decimal".to_owned(),
        _ => "unsigned".to_owned(),
    }
}

/// Wrap a literal as a GBNF terminal, escaping what the grammar reserves.
///
/// Tool calls are JSON, so most terminals contain a quotation mark, and writing
/// those by hand in the emitter is where a grammar acquires the kind of typo
/// that shows up as a model producing valid-looking rubbish.
fn terminal(literal: &str) -> String {
    let mut out = String::with_capacity(literal.len() + 2);
    out.push('"');
    for character in literal.chars() {
        if character == '"' || character == '\\' {
            out.push('\\');
        }
        out.push(character);
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_escapes_the_characters_gbnf_reserves() {
        assert_eq!(terminal("}"), "\"}\"");
        assert_eq!(terminal("\""), "\"\\\"\"");
        assert_eq!(terminal("\\"), "\"\\\\\"");
    }

    #[test]
    fn a_tool_call_opens_with_its_own_name() {
        let schema: Schema = toml::from_str(
            "schema_version = 1\n\
             [[tool]]\n\
             name = \"click\"\n\
             [[tool.param]]\n\
             name = \"mark\"\n\
             type = \"mark\"\n",
        )
        .expect("the fixture parses");
        let grammar = emit(&schema);
        assert!(grammar.contains("root ::= click-call"));
        assert!(grammar.contains("click-call ::= \"{\\\"tool\\\":\\\"click\\\"\""));
        // Deliberately undefined: the tick supplies it.
        assert!(grammar.contains("click-mark ::= mark"));
        assert!(!grammar.contains("\nmark ::="));
    }

    #[test]
    fn a_negative_number_is_expressible() {
        // `look` takes a signed delta and half its range is turning left. A
        // grammar that drops the sign loads, runs, and silently refuses every
        // left turn, which is the sort of defect that reads as a bad model.
        let schema: Schema = toml::from_str(
            "schema_version = 1\n\
             [[tool]]\n\
             name = \"look\"\n\
             [[tool.param]]\n\
             name = \"dx\"\n\
             type = \"i32\"\n",
        )
        .expect("the fixture parses");
        let grammar = emit(&schema);
        assert!(grammar.contains("signed ::= \"-\"? [0-9]+"));
    }

    #[test]
    fn the_string_rule_excludes_both_characters_it_escapes() {
        // A character class missing one backslash is invisible in every
        // tool-call assertion above and wrong in exactly one case.
        let schema: Schema = toml::from_str(
            "schema_version = 1\n\
             [[tool]]\n\
             name = \"say\"\n\
             [[tool.param]]\n\
             name = \"text\"\n\
             type = \"string\"\n",
        )
        .expect("the fixture parses");
        let grammar = emit(&schema);
        assert!(grammar.contains("([^\"\\\\] | \"\\\\\" .)*"));
    }

    #[test]
    fn an_enum_becomes_a_closed_alternation() {
        let schema: Schema = toml::from_str(
            "schema_version = 1\n\
             [[tool]]\n\
             name = \"click\"\n\
             [[tool.param]]\n\
             name = \"button\"\n\
             type = \"enum\"\n\
             values = [\"left\", \"right\"]\n",
        )
        .expect("the fixture parses");
        let grammar = emit(&schema);
        assert!(grammar.contains("click-button ::= \"\\\"left\\\"\" | \"\\\"right\\\"\""));
    }
}
