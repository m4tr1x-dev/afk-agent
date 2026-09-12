//! The schema, as parsed.
//!
//! Deliberately small. The catalogue this describes is a few hundred
//! declarations — roughly ten commands, twelve events, five queries, twenty
//! skills, twenty actions, an error list and a configuration tree — and a type
//! system large enough to express anything would be a type system nobody reads.

use serde::Deserialize;

/// One schema file.
///
/// `schema_version` keeps its prefix although the struct is already `Schema`:
/// the name is the TOML key a human writes, and renaming the field to please a
/// lint would put the two spellings out of step for no reader's benefit.
#[derive(Debug, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct Schema {
    /// Bumped when the shape of this file changes, not when its contents do.
    pub schema_version: u32,
    /// Closed sum types. The message envelope is one of these.
    #[serde(default)]
    pub union: Vec<Union>,
    /// Tools the model may invoke. These are what the grammar is built from.
    #[serde(default)]
    pub tool: Vec<Tool>,
}

/// A closed sum type: the envelope, the tool result, the verification outcome.
#[derive(Debug, Deserialize)]
pub struct Union {
    /// The type's name, used verbatim in both languages.
    pub name: String,
    /// What it is for, copied into both languages' documentation comments.
    #[serde(default)]
    pub doc: String,
    /// Its variants.
    #[serde(default)]
    pub variant: Vec<Variant>,
}

/// One variant of a union.
#[derive(Debug, Deserialize)]
pub struct Variant {
    /// The variant's name.
    pub name: String,
    /// The wire tag.
    ///
    /// Explicit, assigned once, never reused — the same discipline as a
    /// requirement identifier and for the same reason. A decoder that meets an
    /// unknown tag skips the frame, which is what "older clients ignore unknown
    /// events" means in practice, and that only works if a tag never changes
    /// what it denotes.
    pub tag: u16,
    /// What this variant reports.
    #[serde(default)]
    pub doc: String,
    /// Its fields, in wire order.
    #[serde(default)]
    pub fields: Vec<Field>,
}

/// One field of a variant.
#[derive(Debug, Deserialize)]
pub struct Field {
    /// The field's name, in snake case. Each language renames as it prefers.
    pub name: String,
    /// Its type, from the small set in `type_name`.
    #[serde(rename = "type")]
    pub kind: String,
    /// What it carries.
    #[serde(default)]
    pub doc: String,
    /// The version that added it.
    ///
    /// Present means appended after the variant shipped, which a decoder that
    /// runs out of bytes fills with the type's default. That is the whole of
    /// "a new optional field is a minor bump".
    #[serde(default)]
    pub since: Option<String>,
}

/// A tool the model may invoke.
#[derive(Debug, Deserialize)]
pub struct Tool {
    /// The name the grammar admits.
    pub name: String,
    /// What it does and when to reach for it.
    ///
    /// `07-skills.md`: "The description is a prompt." It is what the model reads
    /// when deciding whether this is the right tool, so it is generated into the
    /// grammar's documentation rather than left in a comment.
    #[serde(default)]
    pub doc: String,
    /// Its parameters, in declaration order.
    #[serde(default)]
    pub param: Vec<Param>,
}

/// One parameter of a tool.
#[derive(Debug, Deserialize)]
pub struct Param {
    /// The parameter's name.
    pub name: String,
    /// Its type. `mark` and `cell` are the enumerated kinds: the grammar fills
    /// them from the current observation, every tick.
    #[serde(rename = "type")]
    pub kind: String,
    /// What it means.
    #[serde(default)]
    pub doc: String,
    /// Permitted values, for `enum`.
    #[serde(default)]
    pub values: Vec<String>,
}

/// Map a schema type to its Rust spelling.
///
/// Returns `None` for a type the schema does not define, which is an error
/// worth reporting by name rather than emitting something that will not compile.
#[must_use]
pub fn rust_type(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "u8" => "u8",
        "u16" => "u16",
        "u32" => "u32",
        "u64" => "u64",
        "i32" => "i32",
        "i64" => "i64",
        "f64" => "f64",
        "bool" => "bool",
        "string" => "String",
        "mark" => "MarkId",
        "cell" => "CellId",
        _ => return None,
    })
}

/// Map a schema type to its C# spelling.
#[must_use]
pub fn csharp_type(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "u8" => "byte",
        "u16" => "ushort",
        "u32" => "uint",
        "u64" => "ulong",
        "i32" => "int",
        "i64" => "long",
        "f64" => "double",
        "bool" => "bool",
        "string" => "string",
        "mark" => "MarkId",
        "cell" => "CellId",
        _ => return None,
    })
}

/// `snake_case` to `PascalCase`, for the C# side.
#[must_use]
pub fn pascal(name: &str) -> String {
    name.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
