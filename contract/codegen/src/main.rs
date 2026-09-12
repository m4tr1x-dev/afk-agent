//! Generate every artefact the cross-language contract owes, from one schema.
//!
//! Four outputs, and the reason this is a project-owned generator rather than
//! an off-the-shelf one is that no off-the-shelf tool produces the last two:
//!
//! | Output | Why it cannot be hand-written |
//! | --- | --- |
//! | Rust types | Hand-written types on two sides of a boundary drift |
//! | C# types | The same, from the other side |
//! | A reference page | It has to match the project's front-matter schema exactly |
//! | A grammar | `ADR-0013` regenerates it every tick; it must come from the same file as the tools |
//!
//! The alternatives were weighed and each fails on the same two columns.
//! Protobuf has no sum type in either language's idiom and emits neither a page
//! nor a grammar. `TypeSpec` covers four of the five and is built on a runtime
//! `CLAUDE.md` forbids in this build. The catalogue is small and closed, so
//! writing four emitters against a typed tree costs days, and maintaining two
//! bespoke ones against somebody else's tree costs forever.
//!
//! ```text
//! contract-codegen --schema contract/schema --out-rust crates/afk-contract/src/generated
//!                  --out-csharp src/AfkAgent.Contract/Generated
//!                  --out-docs docs/reference --out-grammar crates/afk-grammar/src/generated
//! ```
//!
//! Every output is written into the repository rather than into a build
//! directory. The documentation jobs run on a hosted Linux runner with neither
//! cargo nor msbuild, so `mkdocs build --strict` has to find the reference pages
//! already on disk — and a reviewer should see a wire-format change in the diff,
//! which is the whole argument for one repository in `ADR-0016`.

mod ast;
mod emit_csharp;
mod emit_docs;
mod emit_gbnf;
mod emit_rust;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ast::Schema;

/// Where each artefact goes.
struct Targets {
    schema: PathBuf,
    rust: Option<PathBuf>,
    csharp: Option<PathBuf>,
    docs: Option<PathBuf>,
    grammar: Option<PathBuf>,
}

fn main() -> ExitCode {
    let mut targets = Targets {
        schema: PathBuf::from("contract/schema"),
        rust: None,
        csharp: None,
        docs: None,
        grammar: None,
    };

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut take = || args.next().map(PathBuf::from);
        match arg.as_str() {
            "--schema" => {
                if let Some(path) = take() {
                    targets.schema = path;
                }
            }
            "--out-rust" => targets.rust = take(),
            "--out-csharp" => targets.csharp = take(),
            "--out-docs" => targets.docs = take(),
            "--out-grammar" => targets.grammar = take(),
            "--help" | "-h" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument: {other}\n\n{USAGE}");
                return ExitCode::from(2);
            }
        }
    }

    match run(&targets) {
        Ok(count) => {
            println!("contract: {count} artefact(s) written");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("contract: {error}");
            ExitCode::from(1)
        }
    }
}

const USAGE: &str = "\
contract-codegen [--schema DIR] [--out-rust DIR] [--out-csharp DIR]
                 [--out-docs DIR] [--out-grammar DIR]

Reads every .toml under --schema and writes the artefacts for whichever
outputs are named. Naming none parses and validates without writing, which is
what a schema-only check wants.
";

fn run(targets: &Targets) -> Result<usize, String> {
    // The reference page records the schema it was generated from, and that
    // page is published. An absolute path here would commit one machine's
    // directory layout to a public site and make the page unreproducible
    // anywhere else, which the freshness check would then report as staleness
    // on every other machine.
    if targets.docs.is_some() && targets.schema.is_absolute() {
        return Err(format!(
            "--schema {} is absolute, and --out-docs records it on the page. \
             Run from the repository root and give a relative path.",
            targets.schema.display()
        ));
    }

    let mut written = 0;

    for path in schema_files(&targets.schema)? {
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let schema: Schema =
            toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;

        validate(&schema, &path)?;

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("{}: has no usable file name", path.display()))?;

        if let Some(dir) = &targets.rust {
            write(dir, &format!("{stem}.rs"), &emit_rust::emit(&schema, stem))?;
            written += 1;
        }
        if let Some(dir) = &targets.csharp {
            let name = ast::pascal(stem);
            write(
                dir,
                &format!("{name}.g.cs"),
                &emit_csharp::emit(&schema, stem),
            )?;
            written += 1;
        }
        if let Some(dir) = &targets.docs {
            write(
                dir,
                &format!("{stem}.md"),
                &emit_docs::emit(&schema, stem, &path),
            )?;
            written += 1;
        }
        // A schema with no tools has no grammar, which is not an omission: the
        // event catalogue is not something the model is allowed to utter.
        if let Some(dir) = &targets.grammar
            && !schema.tool.is_empty()
        {
            write(dir, &format!("{stem}.gbnf"), &emit_gbnf::emit(&schema))?;
            written += 1;
        }
    }

    Ok(written)
}

fn schema_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(format!("{}: no .toml schema files", dir.display()));
    }
    Ok(files)
}

/// Reject a schema that would generate code nobody could compile, and say which
/// declaration is at fault.
///
/// A generator that emits whatever it was given turns a typo in a schema into a
/// compiler error in generated code, which is the wrong place to read it.
fn validate(schema: &Schema, path: &Path) -> Result<(), String> {
    let where_ = path.display();

    if schema.schema_version != 1 {
        return Err(format!(
            "{where_}: schema_version {} is not understood by this generator",
            schema.schema_version
        ));
    }

    for union in &schema.union {
        if union.doc.is_empty() {
            return Err(undocumented(&where_, &union.name));
        }
        let mut seen: Vec<u16> = Vec::new();
        for variant in &union.variant {
            if variant.doc.is_empty() {
                return Err(undocumented(
                    &where_,
                    &format!("{}::{}", union.name, variant.name),
                ));
            }
            if seen.contains(&variant.tag) {
                return Err(format!(
                    "{where_}: {}::{} reuses tag {}. Tags are assigned once and \
                     never reused, because a decoder that meets an unknown one \
                     skips the frame rather than guessing.",
                    union.name, variant.name, variant.tag
                ));
            }
            seen.push(variant.tag);

            let mut seen_optional = false;
            for field in &variant.fields {
                if field.doc.is_empty() {
                    return Err(undocumented(
                        &where_,
                        &format!("{}::{}.{}", union.name, variant.name, field.name),
                    ));
                }
                if ast::rust_type(&field.kind).is_none() {
                    return Err(format!(
                        "{where_}: {}::{}.{} has unknown type {:?}",
                        union.name, variant.name, field.name, field.kind
                    ));
                }
                // A field added after the fact is filled from the default when a
                // decoder runs out of bytes, which only works while every such
                // field is at the end.
                if field.since.is_some() {
                    seen_optional = true;
                } else if seen_optional {
                    return Err(format!(
                        "{where_}: {}::{}.{} has no `since` but follows a field \
                         that does. Appended fields go last, or an older decoder \
                         reads the wrong bytes.",
                        union.name, variant.name, field.name
                    ));
                }
            }
        }
    }

    for tool in &schema.tool {
        // "The description is a prompt", says 07-skills.md: this text is what
        // the model reads when deciding whether this is the right tool, so an
        // undocumented tool is a tool nobody can choose correctly.
        if tool.doc.is_empty() {
            return Err(undocumented(&where_, &tool.name));
        }
        for param in &tool.param {
            if param.doc.is_empty() {
                return Err(undocumented(
                    &where_,
                    &format!("{}.{}", tool.name, param.name),
                ));
            }
            if param.kind == "enum" {
                if param.values.is_empty() {
                    return Err(format!(
                        "{where_}: {}.{} is an enum with no values",
                        tool.name, param.name
                    ));
                }
            } else if !matches!(param.kind.as_str(), "mark" | "cell")
                && ast::rust_type(&param.kind).is_none()
            {
                return Err(format!(
                    "{where_}: {}.{} has unknown type {:?}",
                    tool.name, param.name, param.kind
                ));
            }
        }
    }

    Ok(())
}

/// Report a declaration that carries no documentation.
///
/// Three things depend on the `doc` text and none of them can invent it: the
/// reference page is generated from it, the Rust field inherits it and the
/// workspace refuses an undocumented public field, and a tool's description is
/// what the model reads when choosing between tools. Reporting the omission
/// here names the declaration; reporting it downstream names a generated file
/// nobody is allowed to edit.
fn undocumented(where_: &dyn std::fmt::Display, what: &str) -> String {
    format!(
        "{where_}: {what} has no `doc`. The reference page is generated from \
         this text and the generator will not invent it."
    )
}

fn write(dir: &Path, name: &str, body: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join(name);
    std::fs::write(&path, body).map_err(|e| format!("{}: {e}", path.display()))
}
