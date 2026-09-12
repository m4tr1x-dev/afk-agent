---
title: Release process
description: "Versioning, changelog, artefacts, and what a release has to satisfy before it goes out."
status: draft
owner: m4tr1x-dev
created: 2026-09-12
last_reviewed: 2026-09-12
applies_to: unreleased
tags: [contributing, process]
requirements: []
decisions: []
generated: false
---

# Release process

**Not applicable yet.** There is no build to release.

This page fixes the process before the first release rather than after it, because a release process invented under pressure is a release process with holes in it.

## What is versioned

Four things, and they do not share a number.

| Artefact | Scheme | Why separate |
| --- | --- | --- |
| The application | Semantic versioning | The user-visible thing |
| The inter-process protocol | Its own major and minor | Outlives an application version; a shell and a core may differ |
| The skill manifest schema | Its own major and minor | Third parties would depend on it |
| The documentation site | Tracks the application | Published per release |

Giving the protocol its own version is not pedantry.
A shell reconnecting to a core from a different build has to know whether it can, and an application version does not answer that.

## Before a release

- Every automated check green, on a clean clone.
- The safety manual tests performed and recorded **for this build**. Not "performed recently".
- Frame-rate impact measured on reference hardware.
- The changelog's unreleased section non-empty, and written for someone deciding whether to upgrade.
- Specification pages whose subsystems shipped moved to the archive, with their reference pages authoritative.
- Every open question on a shipped specification page either resolved or restated as a known limitation in the release notes.

That last item is the one that gets skipped, and it is the one that turns an unfinished design into a documented limitation rather than a surprise for a user.

## The changelog

Keep a Changelog format, semantic versioning.

Written for a user, not from the commit history.
A `Documentation` subsection per release covers user-visible documentation changes; typos do not belong in it.

## Artefacts

One installable package containing the executables.

Model weights are not bundled.
Whether bundling would be permissible is a licensing question answered in the notices file — the reason not to is size and user choice rather than permission.

## After a release

Tag it, publish the documentation for that version, open the next unreleased section in the changelog.

## What a release does not do

It does not change a non-goal, relax a safety limit, or enable something that was off by default.

Those are decision records, made deliberately and reviewed, not release-note entries discovered by a user after upgrading.
