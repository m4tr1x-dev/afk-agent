---
title: ADR-0041 — Split continuous integration across hosted and self-hosted runners
description: What runs where, why the interactive runner never sees a pull request, and which checks are forbidden a path filter.
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, ci, safety]
requirements: []
decisions: [ADR-0016, ADR-0025]
affects: [.github, tools, crates, src]
spec: [docs/spec/14-threat-model.md]
evidence:
  - .github/workflows/code-quality.yml
  - .github/workflows/repo-quality.yml
  - .github/workflows/docs-quality.yml
  - .github/workflows/docs-publish.yml
  - experiments/09-toolchain/findings.md
supersedes: null
superseded_by: null
generated: false
---

# ADR-0041 — Split continuous integration across hosted and self-hosted runners

## Context and problem statement

Some of this project's tests need a graphics processor, a running game, a desktop session and a real keyboard.
Hosted runners have none of those. The reference hardware has all of them, and it is the maintainer's own machine.

That is the whole problem in one sentence: **the tests that matter most need the machine we least want to expose.**

The repository is public. A self-hosted runner that executes code from a pull request, on an interactive desktop session, in a project whose entire purpose is synthesising keyboard and mouse input, is close to the worst configuration a self-hosted runner can have. Anyone could open a pull request.

So: which checks run where, and under what conditions does the interactive machine execute anything?

## Decision drivers

- **A public repository with an interactive self-hosted runner is a remote code execution primitive** unless something prevents pull-request code from reaching it.
- **Half the product cannot be built on Linux.** `CON-001` makes the Rust workspace Windows-only; `rust-toolchain.toml` pins one target and `afk-input` declares `user32`.
- **A check that can be skipped by touching only code is not a check.** Traceability and layout are exactly the rules a change would skip.
- **Measurements are not parallelisable.** Two jobs competing for the graphics processor produce numbers that are noise.
- **A failed interactive test can leave a key held down.** `FR-ACT-001` exists because of that failure; the runner has the same one.

## Considered options

1. Everything on hosted runners, and accept that the interactive tests are manual.
2. Everything on the self-hosted machine.
3. Split by what the check needs, with the self-hosted half kept off `pull_request`.

## Decision

**Option 3.** Four workflows, three tiers.

### Hosted Linux — `ubuntu-latest`

Everything that is neither Windows-specific nor hardware-specific:

| Workflow | Job | Why here |
| --- | --- | --- |
| `docs-quality` | The whole documentation toolbox | No compiler needed |
| `repo-quality` | Layout, call sites, traceability | Python only |
| `code-quality` | The contract generator and the freshness check | Plain Rust, no platform surface |
| `docs-publish` | The site | The generated reference pages are committed, which is why this needs no cargo |

**`repo-quality` carries no path filters, and that is the point of it.**
A pull request that adds a top-level directory and no documentation is precisely the change the layout check exists to catch, and a path filter would let it through.
The same holds for traceability: a change touching only code is the case where a requirement quietly loses its covering test.

### Hosted Windows — `windows-latest`

The product workspace: formatting, lints, tests, documentation.
Plus, once it exists, the C# side.

That the C# side can be built on a hosted runner was not known until it was measured.
`dotnet workload list` is empty on both this machine and a hosted runner, which reads like a missing prerequisite; `experiments/09-toolchain/` established that the framework is delivered as packages and does not use workloads at all.
The same experiment established that packaging works from the command line, so a release need not be built on the reference hardware either — only signed there.

**The Rust API reference moves here too.** `docs-publish` built it on the Linux runner, which worked only for as long as every crate was declarations. The first crate that genuinely calls into the platform would have failed the documentation deploy, in a workflow nobody associates with the Rust build. It is now a Windows job that uploads an artefact.

### Self-hosted, the reference hardware

Labels `[self-hosted, Windows, X64, reference-hw, gpu-rx7900xtx, interactive]`.
The safety suite, the perception suite, the grounding benchmark, the latency benches.

**Never on `pull_request`.** Nightly schedule plus `workflow_dispatch`, with an `environment:` requiring manual approval.

Four configuration points, each of which otherwise costs a day:

- **Installed through `run.cmd` at logon, in an interactive session — not as a Windows service.** `docs/spec/03-container-architecture.md` says this about the core itself: session 0 isolation makes input synthesis and window capture impossible for a service. A runner installed as a service inherits exactly that, and every test that matters fails in a way that reads as a product bug: black frames and silent no-ops, with no error.
- **`concurrency: { group: reference-hw, cancel-in-progress: false }` on every self-hosted job.** Two jobs competing for the graphics processor or the keyboard produce measurements that are noise, and the noise looks like data.
- **Defender exclusions** for the runner work directory, the Cargo home and the package cache. These roughly halve Rust link times.
- **A final step that always runs, including after failure: `afk-guardian --release-all`.** A safety test that dies halfway leaves the maintainer's machine with a key held down. That is not hypothetical; it is the failure `FR-ACT-001` exists for, and the runner is subject to it too.

## Consequences

### Good

- Pull-request code never reaches the interactive machine. The remote execution path does not exist rather than being guarded.
- The structural checks cannot be skipped, because they have no path filters and no dependency on a compiler.
- The contract freshness check is a forty-second Linux job, which makes it cheap enough to be unconditional.
- A release can be built by continuous integration; only signing needs the reference hardware.

### Bad

- The tests that most need to run on every change run nightly instead. A regression in the safety suite is found within a day rather than within a pull request, and that is a real cost accepted knowingly.
- The nightly job needs the machine logged in with the screen unlocked, which is a security posture the maintainer accepts for one machine.
- A green pull request does not mean the product works. It means nothing structural is broken. The distinction has to be stated in the contributing guide or people will read it wrong.

### Neutral

- Four workflows rather than one. Each has a distinct trigger and runner, so merging them would mean conditionals rather than fewer files.

## Validation

The self-hosted decision is wrong if the nightly gap starts letting regressions through, which is observable: **a safety-suite failure whose cause was merged more than one working day earlier.**
Two of those in a quarter means the trade is not paying, and the answer is a `pull_request_target`-free approval gate for trusted contributors rather than opening the runner.

The hosted-Windows decision is wrong if the C# build turns out to need something a hosted runner does not have. That is testable the day the first project lands, and `experiments/09-toolchain/` says it will not.

## Pros and cons of the options

### Everything hosted

- Good, because nothing executes on the maintainer's machine.
- Bad, because the safety suite, the perception suite and every latency number are then manual, and `docs/contributing/testing-strategy.md` is explicit that a safety test run when somebody remembers is not a safety test.

### Everything self-hosted

- Good, because one runner, one environment, no split to reason about.
- Bad, because a public repository would be handing arbitrary contributors an interactive shell on a machine that synthesises input. This is disqualifying rather than costly.

### Split by what the check needs

- Good, because each check runs in the cheapest place that can run it honestly.
- Bad, because there are three environments to keep working instead of one.

## More information

- `ADR-0025` excludes hooking and injection; `PresentMon` is used for frame data because it reads event tracing rather than hooking, which keeps the measurement consistent with that exclusion.
- `ADR-0040` pins what each runner installs.
- The self-hosted tier is described here but not yet configured. The workflows for it land with the safety suite in M3, and this record should be revisited if the configuration points above turn out to be incomplete.
