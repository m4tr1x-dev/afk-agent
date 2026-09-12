---
title: Configuration
description: "Configuration format, layering, every setting, and the migration policy."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [spec, configuration]
requirements: [FR-CFG-001, FR-CFG-002, FR-CFG-003, INV-CFG-001, INV-SAFE-001, CON-006]
decisions: []
generated: false
---

# Configuration

## Purpose

This page specifies what can be configured, where the values come from, and what cannot be configured at all.

Two rules shape everything here.

**Nothing is required for a first run** (`FR-CFG-003`).
Zero setup is the product claim, and a configuration file that must be written before the software works would contradict it.

**There are no per-game profiles.**
A profile keyed to a game is a per-game profile whatever it is called, and [non-goals](19-non-goals.md) excludes them.
The word "profiles" appears in this page's history and not in its content.

## Layering

`FR-CFG-001`.

```text
built-in defaults
  <- user configuration file
     <- session overrides, set when starting a session
        <- command-line arguments
```

Later layers win per setting, not per file.
A user who changes one value does not restate the rest.

The effective configuration is recorded in the session recording, because a setting that was in force is part of understanding what happened.

## Location

| What | Where |
| --- | --- |
| User configuration | Under the user's application data directory |
| Session recordings | Under the user's application data directory, separate |
| Model weights | User-chosen, recorded by path |
| Logs | Alongside recordings, rotated |

Nothing is written to the installation directory, so the software does not need write access to where it was installed and does not need elevation (`CON-007`).

## Validation

`FR-CFG-002`.

Every setting declares a type, a default, and a permitted range.
A file with an invalid value is **rejected at load**, naming the file, the line, the value and the permitted range.

The agent does not start with a silently clamped setting.

!!! danger "A clamped safety limit is worse than a refusal to start"

    A user who set an action-rate limit and had it silently raised believes a
    limit is in force that is not.

    They will leave the agent running overnight on that belief.

## The settings

Grouped by area. The generated reference carries the exact types, defaults and ranges; this page states what exists and why.

### Model

| Setting | Purpose |
| --- | --- |
| Endpoint | Local host started by the agent, or a host the user controls |
| Weight path | Where the model file is |
| Context length | The main memory dial |
| Tactical image cost | Latency against grounding accuracy |
| Deliberative image cost | |
| Reasoning depth | For the deliberative cadence |

The endpoint being a setting rather than an assumption is what allows the model to run on another machine on the user's network, which is the best answer when the game needs all of the graphics memory.

### Capture

| Setting | Purpose |
| --- | --- |
| Target selection | How the window is chosen |
| Capture rate | Frames per second requested |
| Reasoning resolution | What the model is shown |
| Detection resolution | What the extractors run on |
| Change thresholds | The four-way classifier's boundaries |

### Cadences

| Setting | Purpose |
| --- | --- |
| Reflex rate | The hard-deadline loop |
| Tactical rate | How often the agent decides |
| Deliberative interval | Heartbeat between plan revisions |

### Safety

Every setting here has a **ceiling that configuration cannot raise** (`INV-CFG-001`).

| Setting | Configurable | Ceiling |
| --- | --- | --- |
| Session wall-clock budget | Yes, downwards | Hard maximum |
| Actions per minute | Yes, downwards | Hard maximum |
| Maximum key hold duration | Yes, downwards | Hard maximum |
| Panic key binding | Yes | Cannot be unset |
| User-present pause | **No** | Always on |
| Foreground guard | **No** | Always on |
| Denied key list | Additions only | Cannot be shortened |
| Text entry towards the game | Yes, off by default | |
| Chat region capture | Yes, off by default | |
| Dry-run mode | Yes | |

The "no" rows are the point of `INV-SAFE-001`.
A configuration file is model-reachable the moment a skill can write one, so anything that must hold regardless of what the model does cannot be a setting.

### Recording

| Setting | Purpose |
| --- | --- |
| Enabled | Recording on or off |
| Keyframe policy | How often a frame is kept |
| Disk quota | |
| Retention period | |
| Transmission | Off, and requires explicit action to change |

### Research

| Setting | Purpose |
| --- | --- |
| Enabled | Whether the agent may reach the network at all |
| Search backend | |
| Sources per research call | |
| Network calls per session | |

Disabling research entirely is supported.
The agent then cannot look anything up, which is a real capability loss and the user's decision to make.

## Session overrides

Set when starting a session, not persisted:

goal, target window, session budget, dry-run, recording on or off.

They are not persisted deliberately.
Persisting a session's settings is how per-game configuration arrives: first it is remembered, then it is keyed by which game, and then the agent needs setting up before it works.

## Migration

The configuration file carries a schema version.

| Situation | Behaviour |
| --- | --- |
| Older version | Migrated forward, a backup kept, the migration logged |
| Newer version | Refuse to start, and say which version is needed |
| Unknown setting | Warn, ignore, preserve on rewrite |
| Removed setting | Warn once, drop on next write |

Preserving unknown settings matters when a user moves a file between builds.
Silently discarding what a newer build wrote is a good way to lose someone's configuration.

## What deliberately cannot be configured

| Not configurable | Why |
| --- | --- |
| Per-game anything | [Non-goals](19-non-goals.md) |
| Disabling the foreground guard | `INV-ACT-001` |
| Disabling user-present detection | `FR-SAFE-002` |
| Raising a safety ceiling | `INV-SAFE-001` |
| Shortening the denied-key list | `INV-ACT-003` |
| Persisting anything the agent learned | `CON-006` |
| Sending screen contents anywhere | `CON-003` |

## Open questions

1. Whether the format should be TOML or JSON. TOML is more comfortable to hand-edit and comments survive a rewrite; JSON has schema tooling. Comments surviving is the stronger argument, since the file is hand-edited.
2. Whether cadence rates should be configurable at all, or derived from measured latency. Exposing them invites configurations that violate the budgets; deriving them makes a slow machine silently different.
3. Where the hard ceilings themselves are recorded. They are in the code, and a user cannot see what they are without reading the source.
4. Whether a user should be able to add to the denied-key list per session rather than globally. It sounds useful and it is the shape of thing that becomes a per-game profile.
5. Whether disabling recording should also disable the blame histogram. The histogram is derived from events rather than frames, so probably not, and the two are easy to conflate.

## Related decisions

`ADR-0022` will record the configuration format and layering.
It is not accepted.
