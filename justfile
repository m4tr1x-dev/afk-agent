# The task runner. `just --list` is the entry point for both toolchains.
#
# ADR-0016 wants a contributor working on the overlay not to have to learn the
# other toolchain, and `just --list` is the literal answer to that: one command
# that names every task in the repository regardless of which half it belongs to.
#
# Two properties decided this over a script:
#
# * It propagates exit codes correctly by default. A PowerShell script that
#   forgets `$LASTEXITCODE` produces a green build that should have been red,
#   which is the most expensive kind of bug a build system can have.
# * Recipe dependencies are declarative, so `just ci` cannot silently skip a
#   step somebody removed from the middle of a script.
#
# One deliberate omission: `lint-docs` is the only recipe that touches Node, and
# nothing reachable from `build` or `test` depends on it. CLAUDE.md forbids Node
# in the build; the documentation toolbox is not the build.

# cmd.exe rather than sh, because Git Bash's `sh` is not reliably on PATH on a
# Windows machine and a task runner that works only in one terminal is worse
# than no task runner. Every recipe below is a single command with no shell
# syntax, so this changes nothing except which program runs it.
set windows-shell := ["cmd.exe", "/c"]

# Show the tasks. This is what `just` alone does.
default:
    @just --list --unsorted

# ---------------------------------------------------------------- contract ---

# Output is committed rather than written to a build directory: the
# documentation workflows run on a hosted Linux runner with neither cargo nor
# msbuild, and `mkdocs build --strict` has to find the reference pages on disk.

# Regenerate every contract artefact from the schema.
codegen:
    cargo run --quiet --manifest-path contract/codegen/Cargo.toml -- --schema contract/schema --out-docs docs/reference/contract

# The highest value per second in the pipeline. A schema edited without
# regenerating leaves two well-formed files describing different protocols, and
# nothing else notices.

# Fail if a committed artefact no longer matches the schema.
codegen-check:
    python tools/check_codegen.py

codegen-fmt:
    cargo fmt --manifest-path contract/codegen/Cargo.toml --all --check

codegen-clippy:
    cargo clippy --manifest-path contract/codegen/Cargo.toml --all-targets --locked -- -D warnings

codegen-test:
    cargo test --manifest-path contract/codegen/Cargo.toml --locked

# -------------------------------------------------------------------- rust ---

fmt:
    cargo fmt --all --check

# Clippy does not link. A missing `#[link]` attribute passes here and fails the
# build, which is why `test` is not optional after a green `clippy`.

# Lint the product workspace with warnings denied.
clippy:
    cargo clippy --workspace --all-targets --locked -- -D warnings

test:
    cargo test --workspace --locked

doc:
    cargo doc --workspace --no-deps --locked

deny:
    cargo deny check

# --------------------------------------------------------------------- c# ----

dotnet-fmt:
    dotnet format --verify-no-changes

dotnet-build:
    dotnet build -c Release

dotnet-test:
    dotnet test -c Release --no-build

# ---------------------------------------------------------- documentation ---

docs:
    mkdocs build --strict

frontmatter:
    python tools/lint_frontmatter.py docs

requirements:
    python tools/lint_requirements.py docs --source crates src tests tools

layout:
    python tools/lint_layout.py

questions:
    python tools/lint_open_questions.py

call-sites:
    python tools/check_call_sites.py

generated:
    python tools/check_generated.py

# The only recipe that touches Node, and nothing depends on it.
lint-docs:
    npx --yes markdownlint-cli2 "**/*.md"

spell:
    npx --yes cspell "docs/**/*.md" "experiments/**/*.md" "*.md"

prose:
    vale --minAlertLevel=error docs/ CLAUDE.md

# --------------------------------------------------------------- aggregate ---

# Everything that runs without a game, a graphics processor or a person.
ci: fmt clippy test doc codegen-fmt codegen-clippy codegen-test codegen-check docs frontmatter requirements questions layout call-sites generated lint-docs spell prose

# The gate the autonomous build advances on. Stricter than `ci`: it records the
# test count and refuses a drop.

# Run the autonomous build's gate.
gate:
    python tools/agent/gate.py
