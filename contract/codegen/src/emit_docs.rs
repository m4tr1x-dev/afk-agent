//! Emit the reference page for one schema file.
//!
//! Generated rather than written, and marked as such in its front matter, so a
//! hand edit to it is caught rather than silently overwritten on the next run.
//! `docs/reference/index.md` has promised that mechanism since the first commit;
//! this is the half that makes the promise true.
//!
//! The front matter has to satisfy `tools/lint_frontmatter.py` exactly — eleven
//! base fields, `generated: true`, and `generated_from` naming the source. That
//! requirement is the whole reason the project owns its generator rather than
//! adapting somebody else's: no off-the-shelf tool emits a page in a schema
//! invented by this repository, and a page that fails the lint is a page the
//! documentation build rejects.

use std::fmt::Write as _;
use std::path::Path;

use crate::ast::{Schema, Tool, Union};

/// The day the shape of these pages was last reviewed by a person.
///
/// A generated page cannot carry a meaningful `last_reviewed` of its own: it is
/// rewritten whenever the schema moves, and stamping today's date on every run
/// would claim a review nobody performed. The honest field is the date the
/// *shape* of the page was last looked at, which is a property of this file.
const REVIEWED: &str = "2026-09-12";

/// Generate the reference page for one schema file.
#[must_use]
pub fn emit(schema: &Schema, stem: &str, source: &Path) -> String {
    let title = title_for(stem);
    // Forward slashes regardless of the host. The page is read on a hosted Linux
    // runner, where a backslash is an ordinary character in a filename.
    let source = source.display().to_string().replace('\\', "/");

    let mut out = String::new();
    front_matter(&mut out, &title, &source);
    preamble(&mut out, &title, &source);
    for union in &schema.union {
        union_section(&mut out, union);
    }
    if !schema.tool.is_empty() {
        tools_section(&mut out, &schema.tool);
    }
    // Every section ends with a blank line, which leaves two at the end of the
    // file. `markdownlint` calls that MD012 and it is right to.
    out.truncate(out.trim_end().len());
    out.push('\n');
    out
}

/// The eleven base fields `lint_frontmatter.py` requires, plus provenance.
fn front_matter(out: &mut String, title: &str, source: &str) {
    let _ = writeln!(out, "---");
    let _ = writeln!(out, "title: {title}");
    let _ = writeln!(
        out,
        "description: Generated from the contract schema - every message, its wire tag and its fields."
    );
    let _ = writeln!(out, "status: draft");
    let _ = writeln!(out, "owner: m4tr1x-dev");
    let _ = writeln!(out, "created: {REVIEWED}");
    let _ = writeln!(out, "last_reviewed: {REVIEWED}");
    let _ = writeln!(out, "applies_to: unreleased");
    let _ = writeln!(out, "tags: [reference, contract]");
    let _ = writeln!(out, "requirements: []");
    let _ = writeln!(out, "decisions: [ADR-0004]");
    let _ = writeln!(out, "generated: true");
    let _ = writeln!(out, "generated_from: {source}");
    let _ = writeln!(out, "---");
    let _ = writeln!(out);
}

/// The heading and the two paragraphs every one of these pages opens with.
fn preamble(out: &mut String, title: &str, source: &str) {
    let _ = writeln!(out, "# {title}");
    let _ = writeln!(out);
    // One sentence per line, per the documentation guide. A generated page is
    // still a page a person reads in a diff.
    let _ = writeln!(out, "Generated from `{source}`.");
    let _ = writeln!(
        out,
        "Editing it by hand does not change the protocol; it changes a file the next generator run overwrites, and continuous integration fails before that happens."
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "Wire tags are assigned once and never reused.");
    let _ = writeln!(
        out,
        "A decoder that meets a tag it does not know skips the frame, which is what lets an older process ignore a message it was never taught."
    );
    let _ = writeln!(
        out,
        "That holds only while a tag keeps meaning what it meant."
    );
    let _ = writeln!(out);
}

/// One union and its variants, one subsection each.
fn union_section(out: &mut String, union: &Union) {
    let _ = writeln!(out, "## {}", union.name);
    let _ = writeln!(out);
    if !union.doc.is_empty() {
        let _ = writeln!(out, "{}", union.doc);
        let _ = writeln!(out);
    }
    for variant in &union.variant {
        let _ = writeln!(out, "### {} (tag {})", variant.name, variant.tag);
        let _ = writeln!(out);
        if !variant.doc.is_empty() {
            let _ = writeln!(out, "{}", variant.doc);
            let _ = writeln!(out);
        }
        if variant.fields.is_empty() {
            let _ = writeln!(out, "Carries nothing beyond its tag.");
            let _ = writeln!(out);
            continue;
        }
        let _ = writeln!(out, "| Field | Type | Since | Meaning |");
        let _ = writeln!(out, "| --- | --- | --- | --- |");
        for field in &variant.fields {
            let since = field.since.as_deref().unwrap_or("1.0");
            let doc = if field.doc.is_empty() {
                "-"
            } else {
                &field.doc
            };
            let _ = writeln!(
                out,
                "| `{}` | `{}` | {since} | {doc} |",
                field.name, field.kind
            );
        }
        let _ = writeln!(out);
    }
}

/// The tool catalogue, which is also what the grammar is generated from.
fn tools_section(out: &mut String, tools: &[Tool]) {
    let _ = writeln!(out, "## Tools");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "The grammar admits these and nothing else, and it is rebuilt every tactical tick."
    );
    let _ = writeln!(
        out,
        "A parameter of type `mark` is filled from the marks in the current observation, so naming a target that is not on screen is inexpressible rather than merely unlikely."
    );
    let _ = writeln!(out);
    for tool in tools {
        let _ = writeln!(out, "### `{}`", tool.name);
        let _ = writeln!(out);
        if !tool.doc.is_empty() {
            let _ = writeln!(out, "{}", tool.doc);
            let _ = writeln!(out);
        }
        let _ = writeln!(out, "| Parameter | Type | Meaning |");
        let _ = writeln!(out, "| --- | --- | --- |");
        for param in &tool.param {
            let kind = if param.values.is_empty() {
                format!("`{}`", param.kind)
            } else {
                param
                    .values
                    .iter()
                    .map(|value| format!("`{value}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let doc = if param.doc.is_empty() {
                "-"
            } else {
                &param.doc
            };
            let _ = writeln!(out, "| `{}` | {kind} | {doc} |", param.name);
        }
        let _ = writeln!(out);
    }
}

/// The page's title, which `lint_frontmatter.py` requires to match its H1.
fn title_for(stem: &str) -> String {
    match stem {
        "events" => "Inter-process events".to_owned(),
        "commands" => "Inter-process commands".to_owned(),
        "queries" => "Inter-process queries".to_owned(),
        "actions" => "Action catalogue".to_owned(),
        "skills" => "Skill manifests".to_owned(),
        "errors" => "Error codes".to_owned(),
        "config" => "Configuration schema".to_owned(),
        other => crate::ast::pascal(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Schema {
        toml::from_str(
            "schema_version = 1\n\
             [[union]]\n\
             name = \"Event\"\n\
             [[union.variant]]\n\
             name = \"SessionState\"\n\
             tag = 1\n\
             fields = [{ name = \"state\", type = \"u8\" }]\n",
        )
        .expect("the fixture parses")
    }

    #[test]
    fn the_title_matches_the_level_one_heading() {
        // lint_frontmatter.py rejects a page where these disagree, and a
        // generator is the one author that can guarantee they never do.
        let page = emit(
            &fixture(),
            "events",
            Path::new("contract/schema/events.toml"),
        );
        assert!(page.contains("title: Inter-process events"));
        assert!(page.contains("\n# Inter-process events\n"));
    }

    #[test]
    fn the_page_declares_where_it_came_from() {
        let page = emit(
            &fixture(),
            "events",
            Path::new("contract/schema/events.toml"),
        );
        assert!(page.contains("generated: true"));
        assert!(page.contains("generated_from: contract/schema/events.toml"));
    }

    #[test]
    fn a_windows_source_path_is_written_with_forward_slashes() {
        let page = emit(
            &fixture(),
            "events",
            Path::new("contract\\schema\\events.toml"),
        );
        assert!(page.contains("generated_from: contract/schema/events.toml"));
        assert!(!page.contains('\\'));
    }

    #[test]
    fn the_page_ends_with_exactly_one_newline() {
        // MD012. Generated output is linted by the same rules as written
        // output, so a trailing blank line fails the documentation build.
        let page = emit(
            &fixture(),
            "events",
            Path::new("contract/schema/events.toml"),
        );
        assert!(page.ends_with("|\n"));
    }

    #[test]
    fn the_wire_tag_reaches_the_page() {
        let page = emit(
            &fixture(),
            "events",
            Path::new("contract/schema/events.toml"),
        );
        assert!(page.contains("### SessionState (tag 1)"));
    }
}
