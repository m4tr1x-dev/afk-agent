//! Emit the Rust side of the contract.

use std::fmt::Write as _;

use crate::ast::{Schema, rust_type};

/// Generate a Rust module for one schema file.
pub fn emit(schema: &Schema, stem: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "//! Generated from `contract/schema/{stem}.toml`. Do not edit."
    );
    let _ = writeln!(out, "//!");
    let _ = writeln!(
        out,
        "//! Regenerate with `just codegen`. Continuous integration"
    );
    let _ = writeln!(
        out,
        "//! regenerates and compares, so a hand edit here fails the build"
    );
    let _ = writeln!(
        out,
        "//! rather than surviving until it contradicts the other language."
    );
    let _ = writeln!(out);

    for union in &schema.union {
        if !union.doc.is_empty() {
            let _ = writeln!(out, "/// {}", union.doc);
        }
        let _ = writeln!(out, "#[derive(Debug, Clone, PartialEq)]");
        let _ = writeln!(out, "pub enum {} {{", union.name);
        for variant in &union.variant {
            if !variant.doc.is_empty() {
                let _ = writeln!(out, "    /// {}", variant.doc);
            }
            let _ = writeln!(out, "    /// Wire tag {}.", variant.tag);
            if variant.fields.is_empty() {
                let _ = writeln!(out, "    {},", variant.name);
                continue;
            }
            let _ = writeln!(out, "    {} {{", variant.name);
            for field in &variant.fields {
                if !field.doc.is_empty() {
                    let _ = writeln!(out, "        /// {}", field.doc);
                }
                let kind = rust_type(&field.kind).unwrap_or("()");
                let _ = writeln!(out, "        {}: {kind},", field.name);
            }
            let _ = writeln!(out, "    }},");
        }
        let _ = writeln!(out, "}}");
        let _ = writeln!(out);

        let _ = writeln!(out, "impl {} {{", union.name);
        let _ = writeln!(out, "    /// The wire tag for this value.");
        let _ = writeln!(out, "    #[must_use]");
        let _ = writeln!(out, "    pub fn tag(&self) -> u16 {{");
        let _ = writeln!(out, "        match self {{");
        for variant in &union.variant {
            let pattern = if variant.fields.is_empty() {
                ""
            } else {
                " { .. }"
            };
            let _ = writeln!(
                out,
                "            Self::{}{pattern} => {},",
                variant.name, variant.tag
            );
        }
        let _ = writeln!(out, "        }}");
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "}}");
        let _ = writeln!(out);
    }

    out
}
