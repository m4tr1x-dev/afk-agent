---
title: ADR-0039 — Generate the cross-language contract from one schema
description: Own a small TOML interface description and a Rust generator, rather than adopt a schema toolchain that produces three of the five artefacts the contract owes.
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, contract, tooling]
requirements: []
decisions: [ADR-0004, ADR-0013, ADR-0016]
affects: [contract, crates, src, docs]
spec: [docs/spec/10-ipc-protocol.md]
evidence:
  - contract/schema/events.toml
  - contract/codegen/src/main.rs
  - contract/codegen/src/emit_gbnf.rs
  - docs/reference/contract/events.md
  - tools/check_codegen.py
supersedes: null
superseded_by: null
generated: false
---

# ADR-0039 — Generate the cross-language contract from one schema

## Context and problem statement

The message catalogue crosses a process boundary and a language boundary at the same time.
`ADR-0002` puts the core and the shell in separate processes; `ADR-0003` and `ADR-0004` make the boundary a byte stream rather than a function call.
So the same twelve events, ten commands and five queries have to exist as Rust types and as C# types, and the two have no compiler between them.

Hand-written types on both sides of a boundary drift.
Not immediately, and not visibly: they drift on the day somebody adds a field to one side and the other side keeps decoding the old layout, which produces a running system that reports the wrong thing rather than a build that fails.

The catalogue owes five artefacts, and the last two are the ones that decide this:

| Artefact | Consumer |
| --- | --- |
| Rust types and a codec | The core, the guardian, the probe |
| C# types and a codec | The shell and the overlay |
| A reference page | The published documentation, in this project's front-matter schema |
| A grammar | `ADR-0013`, which constrains model output to the tools that exist |
| A wire encoding | Both sides, with the versioning rules `ADR-0004` states in prose |

Which of these should be generated, and from what?

## Decision drivers

- **Two languages and no shared compiler.** Anything a person can type twice will eventually be typed differently.
- **The grammar and the reference page have to come from the same declarations as the types.** `ADR-0013` says a tool the model may invoke is constrained by a grammar; if the grammar is written separately from the tool's type, the two can disagree and the disagreement is invisible until a model produces something the validator rejects.
- **`CLAUDE.md` forbids Node in the build.** This is a hard rule, not a preference.
- **`ADR-0004` says not to add a large dependency or a code-generation toolchain for its own sake.** That steer points away from this decision and has to be answered rather than ignored.
- **The catalogue is small and closed.** Roughly ten commands, twelve events, five queries, twenty skills, twenty actions, an error list and a configuration tree. It is a catalogue, not an open-ended type system.

## Considered options

1. Hand-write both sides and review carefully.
2. Protobuf, or FlatBuffers, or Cap'n Proto.
3. JSON Schema alone.
4. TypeSpec.
5. Smithy.
6. A project-owned interface description in TOML with a Rust generator.

## Decision

We choose **option 6**: `contract/schema/*.toml` describes the catalogue, and `contract/codegen/` — a Rust binary excluded from the product workspace — emits Rust, C#, the reference page and the grammar from it.

The driver that settles it is the third column of the table above.
Every off-the-shelf candidate produces at most three of the five artefacts, and the two none of them produce are the two this project made load-bearing: the grammar, because `ADR-0013` regenerates it every tactical tick from the tools that exist, and the reference page, because `ADR-0016` puts the documentation in the same repository precisely so a wire-format change shows up in review as a documentation diff.

**On the steer in `ADR-0004`.** That record warns against adding a code-generation toolchain for its own sake, and the warning is right.
This is not that. The generator is about six hundred lines in a language the project already builds, it has no runtime dependency, it is excluded from `cargo build --workspace` so it never reaches the shipped audit surface, and it exists because two of its outputs cannot be bought.
Writing four emitters against a typed tree we own costs days. Maintaining two bespoke emitters against somebody else's tree costs forever, because the tree changes on their schedule.
A reader who arrives at `ADR-0004` in a year and proposes protobuf should read this paragraph and then the fourth column of the table.

**Encoding**, which mechanises the versioning table `ADR-0004` currently states in prose:

```text
u32_le length | u16_le tag | payload
```

Fields in declaration order, with no per-field tags.
A decoder that meets an unknown tag skips `length` bytes, which is exactly what "older clients ignore unknown events" means.
A field appended after a variant ships carries `since`, and a decoder that runs out of bytes fills it from the type's default.
The generator refuses a schema that reuses a tag, and refuses one that inserts a non-`since` field after a `since` field, because both produce a decoder that reads the wrong bytes without erroring.

**Generated output is committed**, not written to `OUT_DIR`.
Three reasons, the first decisive: the documentation workflows run on a hosted Linux runner with neither cargo nor msbuild, and `mkdocs build --strict` has to find the reference pages already on disk.
Second, a reviewer sees a wire-format change in the diff.
Third, the C# build must not depend on cargo.

Freshness is checked by `tools/check_codegen.py`, which regenerates into a temporary directory and compares.

## Consequences

### Good

- One declaration produces the Rust type, the C# type, the published page and the grammar. They cannot disagree, because there is nothing to keep in step.
- The versioning rules are enforced rather than documented. A reused tag is a generator error naming the variant, not a decoding bug found in the field.
- Documentation is mandatory. The generator refuses a declaration with no `doc`, because the reference page is generated from that text and a hole in it is a hole in the published protocol.
- The C# representation is the generator's to choose. Closed record hierarchies today; a flag flips them to language-level unions when those reach a supported channel, and no hand-written line changes. This is what makes the .NET version question a flag rather than a rewrite.
- An absolute `--schema` path is refused when a page is being generated, so one machine's directory layout cannot reach a published page.

### Bad

- We own a generator. It has no community, no ecosystem and no other users, and when it has a bug the bug is ours.
- The schema language is ours too, which means a contributor learns it. It is deliberately small — six declarations and eleven types — but small is not zero.
- A wire encoding we wrote has not been attacked by anyone else. `ADR-0004` chose a byte stream between two processes we both control, which bounds the exposure, but it does not remove it.
- Generated files in the repository mean a class of pull request whose diff is mostly machine output. The freshness check makes forgetting to regenerate an error, but it does not make the diff smaller.

### Neutral

- `contract/codegen` keeps its own lockfile and its own workspace. `cargo build --workspace` continues to mean "what ships".
- The generator runs on Linux, so the freshness check is a forty-second job on a hosted runner rather than a Windows one.

## Validation

This decision is wrong if the generator becomes the thing being maintained rather than the thing doing the maintaining.

Three observable conditions, any one of which reopens it:

- **The generator exceeds roughly fifteen hundred lines**, or grows a feature no artefact needs. The argument above rests on the catalogue being small and closed; if that stops being true, so does the argument.
- **A schema change requires editing more than one emitter for a reason that is not about that emitter's language.** That is the signature of a type system that has outgrown its representation.
- **An off-the-shelf toolchain produces all five artefacts**, including a grammar in the format the model runtime accepts and a page in this project's front-matter schema, without introducing a runtime `CLAUDE.md` forbids.

The counter-condition is worth stating too, because it is the one that would tempt us: a toolchain that produces four of five and leaves the grammar hand-written is **not** sufficient. The grammar is the artefact `ADR-0013` makes safety-relevant.

## Pros and cons of the options

### Hand-write both sides

- Good, because it adds nothing to the build.
- Bad, because the failure is silent. Two independently correct files that describe different layouts produce a system that runs and reports the wrong thing.
- Bad, because the grammar would then be a third hand-written thing that has to agree with the other two.

### Protobuf, FlatBuffers, Cap'n Proto

- Good, because the encoding is somebody else's problem, and a well-attacked one.
- Bad, because `oneof` is not a sum type in either language's idiom, and the envelope is a sum type in both.
- Bad, because none of them emit a grammar or a Markdown page, so two of the five artefacts stay hand-written and the central problem is unsolved.
- Bad, because the zero-copy model FlatBuffers and Cap'n Proto are built around buys nothing here: large payloads go through shared memory, not through the pipe.

### JSON Schema alone

- Good, because it is ubiquitous and needs no toolchain.
- Bad, because `oneOf` is a poor sum type and validation is not generation.
- Bad, because it says nothing about a binary encoding, which is the thing the two processes actually exchange.

### TypeSpec

- Good, because it covers four of the five artefacts, and covers them well.
- Bad, because it is built on Node, and `CLAUDE.md` says not to add Node to the build. This is a hard rule, so the option fails on the rule rather than on its merits — which is worth recording, because on merits it was the strongest alternative.

### Smithy

- Good, because it is a mature interface description language with real code generation.
- Bad, because it runs on the JVM, which would be a third runtime in a build that currently has two.

### A project-owned schema and generator

- Good, because it produces all five artefacts, including the two nothing else produces.
- Good, because the representation is ours, which turns the C# union question into a flag.
- Good, because the versioning rules become generator errors rather than prose.
- Bad, because we maintain it, and because a contributor has one more small language to learn.

## More information

- `ADR-0004` sets the transport and states the versioning rules this generator mechanises.
- `ADR-0013` requires a grammar regenerated every tactical tick, which is why the tool declarations and the grammar have to come from the same file.
- `ADR-0016` puts documentation and code in one repository so that a protocol change is visible in review; committing the generated page is what makes that true for this protocol.
- `docs/reference/contract/events.md` is the first generated page. It is marked `generated: true` and names its source, so a hand edit is caught rather than silently overwritten.
- The encoding has been designed but not yet implemented: no codec exists at the time of writing, and the wire format above is therefore reasoned rather than measured. The first implementation lands with `afk-contract` in M2, and this record should be revisited if encoding or decoding turns out to need anything the schema cannot express.
