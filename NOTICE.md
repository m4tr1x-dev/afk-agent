# Notices and third-party licensing

The code and documentation in this repository are MIT licensed — see [LICENSE](LICENSE).

That covers what this repository contains.
It does not cover the model weights the software loads, the inference runtime it links, or the frameworks it builds against, and those are licensed separately by their publishers.

This file exists because the distinction is easy to miss and expensive to get wrong.

## Model weights

**Not distributed with this repository.**
They are downloaded at first run, from the publisher's distribution channel, by the user.

The project targets the Gemma 4 family. Its model card states the weights are published under **Apache 2.0**, which permits redistribution.

Two practical consequences:

- Bundling weights in an installer would be permissible rather than prohibited. Whether to do so is a separate question — the files are large, and downloading them lets the user pick a variant that fits their hardware.
- Downstream users are not required to accept a bespoke set of model terms before running the software.

> **Verify before relying on this.**
> The licence above was read from the model card on 2026-09-12 and is recorded in the [known-good matrix](docs/known-good-matrix.md).
>
> Model publishers have shipped weights under bespoke terms before, and a licence stated on a card is not always the licence in the repository's own licence file.
> Confirm both before writing any installer code that redistributes weights, and record the date you checked.

## Runtime components

None of these are vendored; they are fetched by the respective package managers at build time.
Their licences apply to your build, not to this repository.

| Component | Role | Licence |
| --- | --- | --- |
| Inference runtime | Loading and running the model | To be recorded when selected in `ADR-0006` |
| Windows App SDK and WinUI 3 | Application shell and overlay | Microsoft, per its own terms |
| .NET runtime and libraries | The C# side | MIT |
| Rust crates | The core | Predominantly MIT or Apache 2.0, recorded per crate |
| NuGet packages | The C# side | Recorded per package |

A complete dependency licence report is generated as part of the release process, which is `ADR-0028` and not yet written.

## Documentation toolchain

Used to build the documentation site, not shipped with the product.

| Component | Licence |
| --- | --- |
| MkDocs | BSD |
| Material for MkDocs | MIT |
| PyMdown Extensions | MIT |

## Attribution

No third-party fonts, icons or artwork are bundled at present.
When any are, they are listed here with their licence and required attribution, before they are committed.

## What this repository does not contain

Stated explicitly, because a reader may reasonably wonder:

- No model weights.
- No game files, assets or trademarks.
- No proprietary Microsoft redistributables.
- No captured screen content.

Game names appearing in the documentation are used descriptively, to identify the software being discussed.
No affiliation with or endorsement by any game publisher is claimed or implied.
