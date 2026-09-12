---
title: ADR-0043 — Target .NET 10, and keep ahead-of-time compilation off
description: Ship on the long-term support release, with a stated condition for moving; keep native compilation off although it was measured to work.
status: accepted
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
date: 2026-09-12
deciders: [m4tr1x-dev]
consulted: []
informed: []
applies_to: unreleased
tags: [adr, dotnet, packaging]
requirements: []
decisions: [ADR-0016, ADR-0039]
affects: [src, .github]
spec: [docs/spec/09-gui-and-overlay.md]
evidence:
  - global.json
  - Directory.Build.props
  - experiments/09-toolchain/findings.md
  - experiments/09-toolchain/results/2026-09-12-toolchain.md
  - docs/known-good-matrix.md
supersedes: null
superseded_by: null
generated: false
---

# ADR-0043 — Target .NET 10, and keep ahead-of-time compilation off

## Context and problem statement

Two framework questions have been open since the known-good matrix was written, and they are usually answered together because they are usually confused with each other.

The first is which framework release to target. Version 10 is long-term support; version 11 is at release candidate with general availability scheduled, and is standard-term support. The argument that has been made for 11 is that C# 15 has union types, and the message envelope is a sum type in both languages.

The second is whether to compile the user-interface framework ahead of time. The plan this work follows said to expect no, on the grounds that the framework's history with native compilation was never complete.

Neither question had been measured. Both are now.

## Decision drivers

- **The product ships an installer to users.** A long-term support release is the default for that, and departing from it needs a reason.
- **The C# side does not start until M2, and the shell not until M7.** Deciding now for a capability needed later buys nothing.
- **Only 10.0.401 is installed on the reference hardware**, and nothing else.
- **A representation choice is not a framework choice.** This is the driver that dissolves the union-type argument, and it only became available once `ADR-0039` existed.
- **Ahead-of-time compilation is a shape, not a switch.** Its costs land on code that does not exist yet.

## Considered options

For the framework: target 10 now; target 11 now; target 10 and move at general availability.

For native compilation: on; off; off with a recorded revisit.

## Decision

**Target .NET 10, and keep `<PublishAot>false</PublishAot>`.**

### On the framework version

Four reasons, in order of weight.

1. **The known-good matrix says 10 is long-term support and 11 is standard-term.** A product that ships an installer belongs on the long-term release unless a capability forces otherwise.
2. **The only stated reason for 11 dissolves under `ADR-0039`.** The union is a *representation*, and the representation belongs to the generator. It emits a closed record hierarchy today, with a private constructor and an exhaustive `Match`; when language-level unions reach a supported channel, an emitter flag changes and **not one hand-written line does**. The C# consumer writes `Match` either way.
3. **Version 11 is a release candidate today**, and the C# side starts in M2.
4. **The machine has 10.0.401 and nothing else.**

**The condition for moving, stated so it can be checked rather than felt:** when .NET 11 reaches general availability **and** the stable Windows App SDK channel declares support for it, one pull request bumps `global.json`, every `TargetFramework`, `LangVersion`, and the generator's union flag. Nothing else changes, which is the point.

### On ahead-of-time compilation

**It works.** Measured on 2026-09-12: `dotnet publish -p:PublishAot=true` produced a 4,385,280-byte native binary, with no assembly and no runtime configuration file beside it, which launches and composes its window at an 87.9 MiB working set. The plan expected a no and the plan was wrong.

The first attempt failed with `MSB3073` naming `vswhere.exe`, which was a missing `PATH` entry rather than a framework limitation — and is exactly the error that would have closed this question in the wrong direction with a plausible-looking reason.

**The flag still stays off**, on three grounds, none of which is "it does not work":

1. **The benefit is small where it would be felt.** The shell is not on a latency budget. `docs/spec/15-performance-budgets.md` puts the budgets on the reflex loop, which is Rust and already native. Startup time for a window the user opens once per session is not where this product is judged.
2. **The costs land on code that does not exist yet.** Markup binding, reflection over view models and serialisation without a source generator are the usual casualties, and they appear when the fifth screen is written rather than when the flag is set. A spike with one window and one text block exercises none of them.
3. **It needs a tool absent from a clean machine's `PATH`.** A hosted runner would hit the same failure with the same misleading message, and diagnosing it from a workflow log is considerably worse than diagnosing it here.

`<PublishReadyToRun>true</PublishReadyToRun>` with `<SelfContained>true</SelfContained>` is the shipping shape. It captures most of the startup benefit at none of the risk.

The difference this measurement makes is to the *cost of changing our minds*. Had the answer been no, revisiting would have meant waiting on somebody else's roadmap. It is now a question about our own code — does the shell survive it — which is a question we can answer whenever we choose.

## Consequences

### Good

- The product ships on a long-term support release, which is what an installer handed to users deserves.
- The union-type argument is answered permanently rather than deferred, because the answer is architectural.
- The native-compilation question is closed with a measurement and a stated basis, so it will not be reopened as folklore in six months.
- `Directory.Build.props` carries the flag explicitly, which makes it a decision rather than a default nobody chose.

### Bad

- C# 14, not 15. Any language feature in 15 that would genuinely help is unavailable until the move, and we will only discover which ones those are by wanting them.
- Standing on a release candidate's older sibling means the move is future work with its own fallout, arriving at a time we do not choose.
- Keeping native compilation off leaves a measured 4.2 MiB binary on the table in favour of a 133 MiB published directory. The saving is real; it is judged not to matter here, and that judgement could be wrong if start-up time turns out to be what users notice.

### Neutral

- The generator's C# emitter has a union flag that is currently always off. Dead configuration until the move, and cheap to carry.

## Validation

Two conditions, both checkable.

**Move to .NET 11 when both hold:** general availability has shipped, and the stable Windows App SDK channel declares support. Not one without the other — a framework the interface library does not support is not a framework this product can use.

**Revisit native compilation if either holds:**

- **Shell start-up exceeds two seconds from launch to first interactive frame** on the reference hardware. That is the number a user notices, and it is the only one that would justify the migration cost.
- **The published directory exceeds 250 MiB** and installer size becomes a distribution complaint. Native compilation does not shrink the framework's own self-contained libraries, which dominate, so this would need measuring before acting rather than assuming.

If neither holds by M7, the flag stays off and this record is confirmed rather than revisited.

## Pros and cons of the options

### Target 10 now

- Good, because it is long-term support, installed, and supported by the interface library today.
- Bad, because C# 15 is unavailable, and the move is deferred work.

### Target 11 now

- Good, because it is where the language is going and the move happens once.
- Bad, because it is standard-term support for a product that ships an installer.
- Bad, because it is a release candidate and the C# side does not start for two milestones.

### Native compilation on

- Good, because it works, and the binary is 4.2 MiB against 133 MiB.
- Bad, because the costs are unmeasured on the code that matters, which does not exist yet.
- Bad, because it needs a tool that is not on a clean `PATH`, with a message that misattributes the cause.

### Native compilation off, with a revisit

- Good, because the decision is recorded with its evidence and its reopening condition.
- Bad, because carrying an emitter flag and a props line for a thing we are not doing is a small ongoing cost.

## More information

- `experiments/09-toolchain/` holds the measurements, including the transcripts and the sizes.
- `ADR-0039` is what makes the union question a flag. Without it, the argument for .NET 11 would be considerably stronger.
- `ADR-0040` pins the versions this record chooses.
- Two neighbouring results from the same experiment are recorded there rather than here: the interface framework builds from the command line with no workload installed, and single-project packaging works from the command line. Both bear on `ADR-0041` rather than on this decision.
