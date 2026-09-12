# CLAUDE.md

Operating instructions for Claude Code and any other coding agent working in this repository.

## What this project is

`afk-agent` is a general computer-use agent that plays PC games from a natural-language prompt.
Windows 11 only. Rust core, C# and WinUI 3 shell and overlay, local Gemma 4 model.

**Current phase: specification, not implementation.**
There is no code. `docs/` is the deliverable, and it is a specification that code will later be built from.

## Read before you write

Before implementing anything, read the specification page that governs it and the decision records it cites.
The page is the brief; the code is downstream of it.

If the specification does not answer a question you need answered, that is a defect in the page.
Add it to that page's **Open questions** section, or raise it — do not resolve it silently in code.

If the specification is wrong, say so and propose the change to the page.
Do not implement something that contradicts an accepted page without changing the page in the same pull request.

## Hard constraints

These are not preferences. A change that violates one is rejected regardless of its merits.

- **No kernel driver, no code injection, no function hooking, no reading another process's memory, no modifying game files, no drawing inside a game's renderer.** (`CON-002`)
- **No cloud inference.** The model runs locally or on a host the user controls. (`CON-003`)
- **The model's weights never change.** No fine-tuning, no adapters, no training data collection. (`CON-005`)
- **Nothing the agent learns persists beyond the session.** No knowledge base, no per-game profiles, no retrieval over past sessions. Session recordings are write-only from the agent's perspective. (`CON-006`)
- **No per-game code paths or configuration required for a first run.**
- **Input synthesis happens in exactly one component.** If you are adding a second call site, stop. (`FR-ACT-008`)
- **No model-driven component may relax a safety limit.** (`INV-SAFE-001`)
- **Windows 11 only.** Do not add portability shims for platforms we do not support.

## Requirement identifiers

Every normative statement in the specification has an identifier: `FR-ACT-001`, `NFR-PERC-002`, `INV-SAFE-001`, `CON-003`.
They are defined in `docs/spec/01-requirements.md` (and constraints in `docs/spec/00-scope-and-goals.md`).

When you implement one, put its identifier in a comment at the implementation site and in an annotation on its test.
CI builds a traceability table from those markers and fails on an identifier that is referenced but never defined.

Never invent an identifier. If a requirement is missing, add it to the requirements page first.

## Repository layout

```text
docs/            the specification and all other documentation — currently the whole project
  spec/          normative pages; read these before writing code
  decisions/     architecture decision records, MADR format
  explanation/   the reasoning behind the hard parts
  reference/     factual pages; most will be generated from source
  contributing/  how to work here
styles/          Vale prose-lint rules
tools/           documentation lint scripts
.github/         CI workflows and templates
```

Source directories do not exist yet.
When they do, the layout is set by `ADR-0016`, which is accepted.

## Commands

Documentation is currently the only thing that builds.

```bash
pip install -r docs/requirements.txt
mkdocs build --strict
```

```bash
python tools/lint_frontmatter.py docs
python tools/lint_requirements.py docs
```

`--strict` is not optional: it is what catches a navigation entry pointing at a missing file, a code snippet whose source marker was deleted, and a broken relative link.

Python, Node, Vale and lychee are installed on the primary development machine and every check above runs locally.
Run them. Do not claim a build passed when it was not run.

The shell a coding agent gets is started with a reduced `PATH` that omits `dotnet`, `node`, `gh` and the real `python`, and in which `python` resolves to a Windows Store stub.
`.claude/settings.local.json` repairs it; when a tool is missing, resolve it through `tools/agent/tools.json` rather than concluding it is absent.

## Working autonomously

The maintainer has granted a standing permission to commit, push, open pull requests and merge them without asking each time.
It is bounded: branch first, keep every check green, and never force-push or push to `main` directly.

`CON-006` — nothing the agent learns persists beyond the session — governs the **product's** memory at run time.
It does not govern a build journal written by a coding agent about its own work, which is engineering history and belongs in version control like any other.
Session recordings remain write-only from the product's perspective, and that is the constraint `INV-CTX-002` protects.

## Writing documentation

Read `docs/contributing/documentation-guide.md` first. The rules that catch people out:

- **Present tense in `docs/spec/`.** Future tense is banned and Vale enforces it. Whether behaviour exists yet is carried by the page's `status` field, not by grammar.
- **Semantic line breaks.** One sentence per line. This is why line-length linting is off.
- **Front matter is mandatory** on every page and validated by CI. The `title` must match the H1 exactly.
- **Never paste code into a page.** Include it from a real source file by marker, so deleting the marker fails the build.
- **Banned words**: bot, cheat, hack, exploit, aimbot — for legal and positioning reasons. Also simply, just, easily, obviously — they insult a reader who is stuck. Also vague quantifiers in specification pages: fast, robust, reasonable, several. Write the number.

Every specification page follows the same section order: Purpose, Requirements, Design, Interfaces, Invariants, Known limitations, Open questions, Related decisions.

Every open question carries a disposition — **Blocking**, **Non-blocking** or **Limitation** — and CI fails on one that does not.
A page cannot reach `accepted` carrying a question marked Blocking or Limitation: the first has to be answered, the second has to move into Known limitations.

## Things not to do

- Do not add a cloud model SDK, in any form, for any reason.
- Do not add Node to the build. The documentation toolchain is Python; the product is Rust and C#.
- Do not add a persistent store the agent can read from.
- Do not wire session recordings back into the prompt. This is the specific change that would quietly undo the memory design, and it looks harmless.
- Do not create a decision record without an experiment behind it. One written without evidence is a guess in a smart format.
- Do not mark a specification page `accepted` while its Open questions section still contains something that blocks implementation.
- Do not force-push, and do not push to `main`. Work on a `feat/` branch, open a pull request, and merge it once every check is green.

## Language

English for everything in the repository: code, identifiers, comments, commit messages, issues, documentation.
The reasoning is in the documentation guide.
The maintainer converses in Polish; the repository does not.
