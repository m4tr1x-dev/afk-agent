# Contributing

The most valuable contribution to this project today is **reading the
specification and saying where it is wrong**.

There is no code yet. Everything is still cheap to change, and a mistaken
assumption caught on a page costs an afternoon while the same assumption caught
in a subsystem costs a fortnight.

Full guidance is in [docs/contributing/](docs/contributing/index.md). This file
is the short version.

## Ways to contribute

- **Review a specification page.** Especially action and input, grounding and
  verification, and safety and limits, which carry the most risk. Use the
  "Specification change" issue template.
- **Challenge a decision.** Every decision record has a Validation section
  naming what would make us revisit it. If you think one of those conditions
  already holds, open an "Architecture decision proposal" issue.
- **Answer an open question.** The [known-good matrix](docs/known-good-matrix.md)
  lists what is unresolved, and each entry names the experiment that settles it.
  Several need only the hardware and an afternoon.
- **Write documentation.** Read the
  [documentation guide](docs/contributing/documentation-guide.md) first; most of
  its rules are enforced by CI.

## Before writing code

Nothing is ready to implement until the decisions that govern it are accepted.
The blocking set is listed in the [decision index](docs/decisions/index.md).

Each blocking decision is preceded by a timeboxed experiment. A decision record
written without one is a guess in a smart format, and it will be sent back.

## Pull requests

One logical change per pull request, with a commit message in
[Conventional Commits](https://www.conventionalcommits.org/) form.

Any change to behaviour updates the matching specification or reference page in
the same pull request. CI tells you when a generated reference page is stale; it
cannot tell you when a hand-written one is, which is what review is for.

Fill in the template honestly, including the verification section. "CI passed"
is not a verification for a behavioural change.

## Proposing an architecture change

Open an issue, not a pull request. Architecture is discussed as an issue and
recorded as a decision record before it is implemented.

## Language

English for everything in the repository: code, identifiers, comments, commit
messages, issues and documentation. The reasoning is in the documentation guide.

## Code of conduct

Be decent. Disagree about the work, not about the person.

Report anything that needs reporting to the address in [SECURITY.md](SECURITY.md).
