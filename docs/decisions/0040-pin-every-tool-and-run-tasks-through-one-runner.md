---
title: ADR-0040 — Pin every tool exactly, and run every task through one runner
description: Exact versions rather than ranges for the compiler, the framework and every build tool, with a single task runner that both toolchains share.
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, tooling, build]
requirements: []
decisions: [ADR-0016]
affects: [tools, crates, src, contract]
spec: []
evidence:
  - rust-toolchain.toml
  - global.json
  - Directory.Packages.props
  - .config/dotnet-tools.json
  - justfile
  - experiments/09-toolchain/results/2026-09-12-toolchain.md
supersedes: null
superseded_by: null
generated: false
---

# ADR-0040 — Pin every tool exactly, and run every task through one runner

## Context and problem statement

This repository builds two languages with two toolchains, and it is meant to be built unattended for weeks at a time.
Both facts point at the same question from different directions.

A build that floats on `stable` fails on a morning when nothing in the repository changed.
Clippy gains lints every six weeks; the Rust coding standards run it with `-D warnings`; so a toolchain release turns somebody's unrelated pull request red, and the person who has to deal with it is whoever pushed next rather than whoever chose the policy.
The same argument applies to a documentation generator installed with `dotnet tool install -g` at workflow time, which is what `docs-publish.yml` did until this record.

The second direction is discoverability.
`ADR-0016` puts both halves in one repository so that "a contributor working on the overlay does not have to learn the other toolchain".
That promise is only kept if there is somewhere to look that lists what can be run.

Two questions, then: what pins the versions, and what lists the tasks?

## Decision drivers

- **A version bump should be a pull request, not a Tuesday.** Whoever bumps the version owns the fallout, which is where that work belongs.
- **The known-good matrix is an argument for pinning.** It records what each component was verified against. A build that resolves versions at run time makes those records describe something other than what ran.
- **Unattended operation.** Nobody is watching to notice that a failure is environmental.
- **Exit codes.** A build step that fails and reports success is the most expensive defect a build system can have.
- **One list of tasks, both toolchains.**

## Considered options

For pinning: exact versions everywhere; ranges with a lockfile; latest-and-fix-forward.

For the runner: `just`; PowerShell scripts; `make`; `cargo xtask`; nothing, and put the commands in the workflow files.

## Decision

**Pin exactly, in five files, each owning one thing:**

| File | Pins |
| --- | --- |
| `rust-toolchain.toml` | The Rust toolchain, at `1.98.1`, with one target |
| `global.json` | The .NET SDK, at `10.0.401`, with `rollForward: disable` |
| `Directory.Packages.props` | Every package version, centrally, with transitive pinning on |
| `.config/dotnet-tools.json` | Every .NET tool, at a version, restored rather than installed |
| `Cargo.lock` and `contract/codegen/Cargo.lock` | The dependency graphs, both of them |

`rollForward: disable` is the unusual one and it is deliberate.
Without it, the build silently uses a newer SDK when one is installed, which is exactly the class of difference that shows up as an unreproducible failure months later.

**Run tasks through `just`.** One `justfile` at the root, `just --list` as the entry point.

Three properties decided it over a PowerShell script, in order of weight:

1. **It propagates exit codes correctly by default.** A PowerShell script that forgets `$LASTEXITCODE` produces a green build that should have been red. That is not a hypothetical failure; it is the most common way a build system lies.
2. **Dependencies are declarative.** `just ci: fmt clippy test …` cannot silently skip a step somebody deleted from the middle of a script.
3. **It is one static binary**, installed with `cargo install just --locked`, which the machine already has a toolchain for.

**`lint-docs` is the only recipe that touches Node**, and nothing reachable from `build` or `test` depends on it.
`CLAUDE.md` forbids Node in the build; the documentation toolbox is not the build, and keeping that distinction structural rather than verbal is what stops it eroding.

**The `justfile` sets `windows-shell := ["cmd.exe", "/c"]`** and every recipe is a single command with no shell syntax.
Git Bash's `sh` is not reliably on `PATH` on a Windows machine, and a task runner that works only in one terminal is worse than no task runner.

## Consequences

### Good

- A toolchain upgrade is a pull request with its own lint fallout, reviewed by whoever chose to do it.
- `just --list` answers "what can I run here" for both toolchains in one place, which is `ADR-0016`'s stated goal made operable.
- The documentation generator is restored at a pinned version rather than installed at whatever is newest, so the published site is reproducible.
- Central package management makes two projects referencing two versions of one package impossible rather than merely unlikely.

### Bad

- Pinned toolchains go stale silently. Nothing fails, so nothing reminds anybody. A quarterly bump has to be somebody's job or it will not happen.
- `just` is a dependency contributors install. It is one `cargo install` away, and the repository already requires a Rust toolchain, but it is still a step.
- `rollForward: disable` fails loudly on a machine with a different patch release. That is the intent, and it will still be irritating the first time.
- The `cmd.exe` shell choice means a recipe cannot use a pipe or a conditional without changing the setting. Every recipe today is a single command, and the constraint is worth more than the convenience.

### Neutral

- `contract/codegen` keeps a second lockfile, because `ADR-0016` excludes it from the product workspace.

## Validation

This is wrong if pinning costs more than it saves, which would show up as either of:

- **A pin that is more than two minor versions behind and nobody noticed.** The remedy is a scheduled bump, not floating versions.
- **A recipe that has to be duplicated** because `just` cannot express it without shell syntax. One such recipe is a nuisance; three would mean the shell choice is wrong and the answer is a `sh` requirement, not a script.

The revisit trigger for the Rust pin is concrete: **bump when a lint or a language feature the project wants is only available on a newer toolchain**, and do it in a pull request that carries its own Clippy fallout.

## Pros and cons of the options

### Exact pins

- Good, because a failure is always caused by something in the diff.
- Bad, because staleness is silent.

### Ranges with a lockfile

- Good, because updates arrive without effort.
- Bad, because "without effort" means "in somebody else's pull request".

### PowerShell scripts instead of a runner

- Good, because nothing extra is installed and the platform is Windows anyway.
- Bad, because `$LASTEXITCODE` handling is manual and the failure mode is a green build.
- Bad, because the C# contributor and the Rust contributor end up with separate scripts, which is the split `ADR-0016` exists to avoid.

### `cargo xtask`

- Good, because it needs no new tool and is fully typed.
- Bad, because it makes every task a Rust build, including the documentation ones that run on a runner with no cargo.

### Commands only in the workflow files

- Good, because there is exactly one copy.
- Bad, because a contributor cannot run continuous integration locally, which is the thing that makes a red pull request expensive.

## More information

- `experiments/09-toolchain/` measured the .NET side: the framework builds from the command line from two package references, with no workload installed. Those two versions are what `Directory.Packages.props` pins.
- `ADR-0041` covers where each of these tasks runs.
- `ADR-0043` covers the framework version specifically, and the condition that reopens it.
