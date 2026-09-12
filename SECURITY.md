# Security

## Reporting a vulnerability

Report privately through [GitHub's security advisory form](https://github.com/m4tr1x-dev/afk-agent/security/advisories/new).
Do not open a public issue for a vulnerability.

Expect an acknowledgement within seven days.
There is no release yet, so there is no patch timeline to promise; what there is instead is a commitment to fix the specification before the defect is built.

## Supported versions

None. The project has not released.

| Version | Supported |
| --- | --- |
| unreleased | Not applicable |

## What this software does, stated plainly

You should know this before running it, and a security policy is the right place to say it.

**It captures your screen.**
Specifically, one window that you select — not the whole desktop — but that window's contents pass through the perception pipeline, are described to a language model, and may be written to a session recording on disk.

If the game window shows a password, a private message, an account name, or anything else you would not put in a screenshot, the agent sees it and the recording keeps it.

**It synthesises keyboard and mouse input.**
While a session is running, the software can type and click. The interlocks that bound this are specified in [safety and limits](docs/spec/12-safety-and-limits.md): input is delivered only while the target window has focus, a panic key halts everything, and the agent pauses the moment you touch the keyboard.

**Everything stays local.**
The model runs on your machine. Screen contents are not transmitted anywhere. The only network traffic the agent generates is from the research skills, which send search queries and fetch web pages — never anything derived from your screen.

**Session recordings are yours.**
Local by default, subject to a disk quota and a retention period, deletable in one action, and never uploaded without an explicit opt-in.

## Out of scope

**Terms-of-service violations are not security vulnerabilities.**
Reports that the software can be used to automate a game that prohibits automation will be closed. That is what it does; see [terms of service and anti-cheat](docs/explanation/anti-cheat-and-terms-of-service.md).

**Detection by anti-cheat systems is not a vulnerability.**
Synthesised input is detectable by design. We do not accept reports that it can be detected, and we do not accept patches that make it less detectable.

**We do not accept evasion contributions.**
Not obfuscation of the injected-input marker, not timing camouflage, not process-name hiding. This is a permanent position, recorded in [non-goals](docs/spec/19-non-goals.md).

## What we consider a genuine report

Roughly, anything that breaks one of these:

- Input reaching a window other than the target. (`INV-ACT-001`)
- Input remaining held after a stop, a panic, a focus loss or a crash. (`INV-SAFE-002`)
- The panic key failing to take effect, or failing while the core is unresponsive. (`FR-SAFE-001`)
- A safety limit being raised, disabled or bypassed by anything a model produced. (`INV-SAFE-001`)
- Content on screen or in a fetched web page causing the agent to take an action it was not instructed to take.
- Screen contents, recordings, logs or prompts leaving the machine without explicit opt-in.
- A skill escaping its declared permissions.
- Weights or configuration being loaded without integrity verification.

The last two categories and the prompt-injection one are analysed in the [threat model](docs/spec/14-threat-model.md).

## Prompt injection

Worth calling out separately, because it is the least intuitive risk here and it is real.

The agent reads text from the screen and from web pages it fetches.
Both are attacker-controlled in the general case: another player's chat message, a wiki page anyone can edit.

Such text is data, never instruction.
The specification requires it to enter the context inside a delimited untrusted block, and requires that no skill invocation can change a safety setting, register a skill, or raise a budget.
Chat text is excluded from the observation by default.

If you find a path by which observed or fetched text changes what the agent is permitted to do, that is a report we want.

## Hardening

If you are being careful:

- Run with session recording off unless you need it.
- Keep text entry disabled, which is the default.
- Use dry-run mode on a game you have not run the agent against before.
- Set a wall-clock budget. An unattended session with no time limit is an unattended session with no time limit.
- Do not run it on an account you would mind losing.
