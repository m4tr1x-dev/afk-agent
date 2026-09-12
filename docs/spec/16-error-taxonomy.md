---
title: Error taxonomy
description: "The error code scheme, categories, and which errors the agent can recover from itself."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, errors]
requirements: [FR-PERC-009, FR-ACT-006, FR-SKILL-001, FR-OBS-001, FR-CFG-002]
decisions: []
generated: false
---

# Error taxonomy

## Purpose

This page fixes the error scheme before any code establishes an ad-hoc one.

It is short and it is written early for a reason: error handling conventions are almost never retrofitted, and a system that grew three incompatible ones is a system where nobody can tell which failures matter.

## The code scheme

```text
AFK-<AREA>-<NNN>
```

Areas match the requirement areas in [requirements](01-requirements.md): `PERC`, `LOOP`, `ACT`, `GND`, `SKILL`, `CTX`, `GUI`, `CFG`, `SAFE`, `OBS`, `MODEL`.
Plus `SYS` for the process and platform level, which has no requirements area of its own.

Codes are assigned once and never reused.
A retired code is marked retired and left in place, because it appears in logs, in issues and in users' screenshots for as long as anyone runs an old build.

## Categories

Every error declares one, and the category determines who is expected to act.

| Category | Meaning | Who acts |
| --- | --- | --- |
| **User** | The user can fix it: a setting, a window mode, a missing file | The user |
| **Transient** | Expected to resolve on retry: a busy device, a timeout, a slow load | The agent |
| **Environment** | Something outside the agent is wrong: a driver, a missing capability, a permission | The user, with guidance |
| **Defect** | The agent is wrong | A developer |

The distinction between **user** and **defect** is the one that matters in practice.
A capture failure caused by exclusive fullscreen is a user error with a one-line fix; the same message produced because the agent mishandled a window-mode change is a defect.
`FR-PERC-009` exists precisely so those two do not look identical.

## What an error carries

```text
code              AFK-PERC-003
category          user | transient | environment | defect
message           one sentence, what happened
cause             the underlying error, if any
remedy            what to do about it, for user and environment categories
recoverable       whether the agent can continue by itself
context           session, tick, subgoal, target window
```

**The remedy is not optional for a user-category error.**
Telling a user that capture failed is not helping them; telling them the game is in exclusive fullscreen and how to change it is.

## Recovery

| Category | Agent behaviour |
| --- | --- |
| User | Pause, report, wait. No retry: retrying will fail identically. |
| Transient | Retry with backoff, up to a bounded count, then escalate to the stuck ladder. |
| Environment | Stop, report with remedy. Some of these are detectable at startup and should be. |
| Defect | Fail loudly. Record everything needed to reproduce. Do not attempt to continue. |

The last row is deliberate.
A defect means an assumption the code relies on is false, and continuing means acting on a state the system does not understand — in software that synthesises input, that is the wrong trade.

## Error handling across the process boundary

Errors cross the control pipe as codes with structured context, not as formatted strings.
The shell renders them, which means one message can be localised and one code can be searched.

A crash in the core is not an error in this sense: the shell observes the process exit, the guardian releases input, and the session ends.

## Rejected actions are not errors

`FR-ACT-006`.

An action the model proposed that failed validation is **information for the model**, not a failure of the system.

It is reported back into the context so the model can choose differently, and it is counted towards the no-progress signals.
It does not produce an error code, does not surface to the user, and does not interrupt the session.

Treating these as errors would fill the log with the ordinary operation of a constrained loop, and would train a user to ignore errors.

## Validation errors in configuration

`FR-CFG-002`.

A configuration file with an invalid value is rejected at load, naming the file, the line, the value, and the permitted range.
The agent does not start with a silently clamped setting.

A clamped safety limit is worse than a refusal to start, because the user believes a limit is in force that is not.

## Codes

The full table is generated from the error enumerations in the source and lives in the reference documentation.
This page specifies the scheme; the code list is not written by hand, because a hand-written list drifts from the enumeration within weeks.

## Open questions

1. Whether "environment" earns its own category or is a subset of "user". It is separate because the remedies differ — a driver update is not a setting change — and that may not justify a category.
2. How many transient retries before escalation, per operation class. One number is wrong for both a device timeout and a page fetch.
3. Whether a defect in a non-critical path should stop the session. Stopping is the safe answer, and it turns an unimportant failure into a ruined eight-hour run.
4. Whether error codes should be stable across major versions. Stability helps users and search engines; it also freezes a taxonomy that may prove wrong.

## Related decisions

`ADR-0027` will record the error handling and recovery policy.
It is not accepted.
