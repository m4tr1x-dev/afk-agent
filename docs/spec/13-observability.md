---
title: Observability
description: "Structured logging, the metric catalogue, session recording, and redaction."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, observability]
requirements: [FR-OBS-001, FR-OBS-002, FR-OBS-003, FR-OBS-004, NFR-OBS-001, INV-OBS-001, INV-CTX-002, FR-GND-008]
decisions: []
generated: false
---

# Observability

## Purpose

The agent runs while nobody is watching.
Every interesting failure therefore happens unobserved, and the only way to understand one afterwards is from what the agent recorded at the time.

This page specifies what it records, at what cost, and who may read it.

## Three outputs

| Output | For | Lifetime |
| --- | --- | --- |
| **Structured events** | Diagnosis, metrics, the blame histogram | Rotated logs |
| **Metrics** | Budgets, live display, trend | Aggregated per session |
| **Session recording** | Review by the user, replay by a developer | Quota and retention bound |

All three are local by default and none leave the machine without explicit opt-in (`FR-OBS-004`).

## Structured events

`FR-OBS-001`.

Every event carries the session, the tick, the active subgoal, a monotonic timestamp, and a severity.

| Event | Emitted on |
| --- | --- |
| `session.started` / `session.ended` | Lifecycle, with the reason for ending |
| `tick.reflex` | Sampled, not every tick — see cost below |
| `observation.built` | Element count, scene class, change class, phase timings |
| `model.called` / `model.returned` | Cadence, image cost, token counts, latency |
| `action.proposed` | The action, before validation |
| `action.rejected` | With the reason, and whether it was reported back to the model |
| `action.executed` | With the resolved target and the grounding path used |
| `verification.outcome` | Confirmed, refuted or inconclusive, with the evidence |
| `failure.attributed` | The blame category (`FR-GND-008`) |
| `skill.invoked` / `skill.returned` | Skill, duration, outcome |
| `context.compacted` | What survived, what was summarised, what was dropped |
| `safety.triggered` | Which interlock, and what it did |
| `budget.exhausted` | Which budget |
| `error` | An error code from [the taxonomy](16-error-taxonomy.md) |

`safety.triggered` is never sampled and never dropped.
Whatever else is lost under load, a record of an interlock firing is not.

## Metrics

Derived from events and shown live in the shell.

| Metric | Why it matters |
| --- | --- |
| Reflex tick duration, 95th percentile | The hard deadline |
| Model latency per cadence | The budget that is not yet measured |
| Tokens per tick | Context growth, prefix reuse working or not |
| Prefix reuse rate | Directly, whether the layout rules are being honoured |
| Verification outcome distribution | The agent's own success rate |
| Grounding hit rate per path | What the arbiter is acting on |
| Actions per minute | Runaway detection |
| Graphics memory headroom | How close the game is to being starved |
| Subgoal completion rate | Whether the plan is working |
| Blame histogram | What to fix next |

The prefix reuse rate is worth calling out.
It is the one metric that catches a violation of the byte-identical prompt rule, and that violation is invisible in every other respect — the agent behaves correctly and costs several times more per tick.

## Session recording

`FR-OBS-002`, `INV-OBS-001`.

Contains everything needed to replay: keyframes, observations, assembled prompts, model outputs, sampling parameters and seeds, executed actions, verification outcomes, and compaction results.

Compaction results are stored rather than re-derived, because compaction is a model call and replaying it would diverge.

Frames are stored as keyframes rather than every frame.
A full-rate recording of an overnight session is hundreds of gigabytes and adds nothing over the keyframes plus the observation stream.

`INV-OBS-001` requires a recording to be readable by the replay harness of the same version.
A recording nobody can read is disk usage.

### The agent never reads a recording

`INV-CTX-002`.

Recordings are output.
Feeding them back into the prompt would reintroduce cross-session memory, and it is the specific small change by which this design would be quietly undone — see [context and research](08-context-and-research.md).

## Cost

`NFR-OBS-001`.

Instrumentation that breaks the loop it measures is worse than none.

- Reflex-tick events are **sampled**, not emitted per tick. The 95th percentile over a session does not need every sample.
- Serialisation happens off the reflex thread.
- Keyframe encoding happens on a worker.
- The telemetry ring the overlay reads is lock-free and may drop frames. A display that falls behind must never be able to block the core.

## Privacy

`FR-OBS-003`, `FR-OBS-004`.

Recordings are images of the user's screen, written while they are away.

| Control | Default |
| --- | --- |
| Recording enabled | On, with keyframes only |
| Transmission | Never, without explicit opt-in |
| Disk quota | Bounded; oldest keyframes dropped first |
| Retention | Bounded; expired recordings deleted |
| Purge | One action, immediate |

**Redaction** applies to logs and errors: window titles are kept, recognised text is not written to logs at severity levels below debug, and no recognised text appears in an error message.

Frames cannot be redacted meaningfully, since the agent does not know which part of a screen is sensitive.
The mitigations are the quota, the retention period, and window-scoped capture — the user's other windows were never in frame.

### Crash reports

Opt-in, per report, never automatic.

A report contains the error, the stack, the configuration, and the recent event stream.
It does **not** contain frames, recognised text, or the note block, because all three may contain the user's own content.

## Interfaces

| Direction | Interface |
| --- | --- |
| Out | Events to a rotated local log |
| Out | Metrics over the control pipe, for the shell |
| Out | Telemetry over the shared ring, for the overlay |
| Out | Recordings to the session directory |
| In | The replay harness reads recordings |

## Open questions

1. What reflex sampling rate preserves a useful 95th percentile. Too sparse and the tail disappears; too dense and the instrumentation is in the budget it measures.
2. Whether keyframe selection should follow the change classifier or run on its own schedule. Following it means an idle session records almost nothing, which is correct and also means a failure during a quiet stretch has no frames.
3. Whether the note block belongs in the recording. It is the clearest record of what the agent believed, and it is also user content by definition.
4. Whether recordings should be encrypted at rest.
5. How a user reviews a session without the shell. There is currently no answer, and a recording that needs the application to read it is a recording the user does not really own.

## Related decisions

`ADR-0023` will record the logging, tracing and recording format, and `ADR-0031` crash reporting and privacy.
Neither is accepted.
