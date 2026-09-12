# Question 9 — the solution format, and ahead-of-time compilation with the user-interface framework

**Status: both answered, and one of them contradicts what this project expected.**

Four sub-questions, all answered on 2026-09-12 on the reference hardware.

| Sub-question | Answer |
| --- | --- |
| Does the newer solution format work? | It is the **default**. `dotnet new sln` produces `Probe.slnx`, and `dotnet sln migrate` converts an old one |
| Does the user-interface framework build from the command line without a Visual Studio workload? | **Yes**, zero warnings and zero errors, from two package references |
| Does single-project packaging work from the command line? | **Yes.** A real 27 MiB installer, no `.wapproj` |
| Does ahead-of-time compilation work with the user-interface framework? | **Yes.** A 4.2 MiB native binary that runs and shows its window |

The last row is the one that matters, because the plan this work follows said to
expect a no, and `Directory.Build.props` was going to carry
`<PublishAot>false</PublishAot>` as a recorded decision rather than a default.
The decision still stands, but it now rests on a measurement instead of on
folklore, and it rests on different reasoning. See **The expectation that was
wrong** below.

## Why this probe exists

`docs/known-good-matrix.md` question 9 says "prototype both", and names the
repository layout as what it blocks. That is understated. Three things were
waiting on it:

- Whether `AfkAgent.slnx` can be the committed solution file, or whether the
  repository needs the older format for another year.
- Whether a hosted `windows-latest` runner can build the C# side at all. The
  machine has Visual Studio 2026 Insiders, a hosted runner does not, and
  `dotnet workload list` is empty on both — which reads like a problem until you
  find out the framework does not use workloads.
- Whether packaging is `.wapproj`, which needs a Visual Studio workload and does
  not build under `dotnet build`. If it were, packaging would be self-hosted
  only, forever.

## Method

Four throwaway projects in a scratch directory, nothing committed to the
repository. Each one built from a clean directory with the installed toolchain
and no Visual Studio involvement: `dotnet` on the command line only.

Exact commands, versions and transcripts are in
`results/2026-09-12-toolchain.md`.

## What was measured

### The solution format is the default, not an option

```text
dotnet --version          10.0.401
dotnet new sln --name Probe
  -> Probe.slnx
dotnet sln --help | grep migrate
  -> migrate    Generate a .slnx file from a .sln file
```

There was no choice to make. The tooling produces the new format unprompted, and
carries a converter for the old one. `AfkAgent.slnx` is simply what a solution
file is now.

### The framework builds from the command line, with no workload

```text
dotnet workload list
  -> (empty)
dotnet build -c Release
  -> Build succeeded. 0 Warning(s) 0 Error(s)
```

Two package references and nothing else:

| Package | Version |
| --- | --- |
| `Microsoft.WindowsAppSDK` | 2.4.0 |
| `Microsoft.Windows.SDK.BuildTools` | 10.0.26100.4948 |

This is the finding that unblocks hosted continuous integration for the C# half
of the product. The framework is delivered as packages; the empty workload list
is not a missing prerequisite, it is the absence of a mechanism this framework
stopped using.

**One trap, found the hard way.** The markup compiler generates the entry point.
A hand-written `Program.cs` with a `Main` collides with it and the build fails on
a duplicate entry point, which reads like a project-file error and is not one.

### Single-project packaging works from the command line

```text
dotnet publish -c Release
  -p:GenerateAppxPackageOnBuild=true
  -p:AppxPackageDir=out/
  -p:UapAppxPackageBuildMode=SideloadOnly
  -p:AppxBundle=Never
  -> out/Pack_0.0.0.1_x64_Test/Pack_0.0.0.1_x64.msix   28,286,658 bytes
```

A real installer, with the runtime dependency packages beside it for four
architectures and the sideload script the template expects. No `.wapproj`, no
Visual Studio, no workload.

That removes the constraint the plan was written around. Packaging does not have
to be self-hosted, which means a release can be built by continuous integration
and only signing needs this machine.

### Ahead-of-time compilation works

```text
dotnet publish -c Release -p:PublishAot=true
  -> bin/Release/.../win-x64/native/Probe.exe    4,385,280 bytes
```

The published directory contains **no `Probe.dll` and no
`Probe.runtimeconfig.json`**, which is what distinguishes a native binary from a
single-file bundle that still carries a runtime. The 49 libraries beside it are
the framework's own self-contained natives.

Re-verified on 2026-09-12 rather than recalled:

```text
running: pid 13000, window 'afk-agent spike', working set 87.9 MB
```

It launches, it creates its window, and the window has the title the markup
gives it. That is the whole of what "works" means for this question.

**The first attempt failed, and the failure was not the framework.**

```text
MSB3073: the command "vswhere.exe -latest ..." exited with code 9009
```

`vswhere.exe` was not on `PATH`. Adding
`C:\Program Files (x86)\Microsoft Visual Studio\Installer` fixed it and the
publish succeeded. Recorded here because the message names a Visual Studio tool
and the natural reading is "ahead-of-time compilation needs Visual Studio" — the
wrong conclusion from the right error, and exactly the kind of thing that closes
a question in the wrong direction.

## The expectation that was wrong

The plan said: *"Ahead-of-time compilation: no. The framework's history with it
was never complete, the Rust core is native anyway, and ready-to-run plus
self-contained gives most of the benefit at no risk."*

The first clause is now false on this toolchain. It compiles, it runs, and the
binary is 4.2 MiB against a 133 MiB published directory.

**The conclusion does not change, but its basis does.** Three reasons to keep
`<PublishAot>false</PublishAot>`, none of which is "it does not work":

1. **The measured benefit is small where it would be felt.** The working set was
   87.9 MiB for a window with one text block. The startup and memory savings
   ahead-of-time compilation buys are real, but the shell is not on a latency
   budget — `15-performance-budgets.md` puts the budgets on the reflex loop,
   which is Rust.
2. **The costs land on the parts of the shell that do not exist yet.** Markup
   binding, reflection over view models and anything using
   `System.Text.Json` without a source generator are the usual casualties, and
   they turn up when the fifth screen is written rather than when the flag is
   set. A spike with one window does not exercise any of them.
3. **It needs a tool that is not on the `PATH` of a clean machine.** The build
   above only worked after `vswhere.exe` was found. A hosted runner would hit the
   same failure with the same misleading message.

So the flag stays off, and `ADR-0043` records it as a decision with a revisit
trigger rather than as a default nobody chose. **A revisit is now cheap**, which
it would not have been if the answer had been no: the question becomes "does the
shell survive it", not "is it possible".

## What this changes elsewhere

| Page or record | Change |
| --- | --- |
| `docs/known-good-matrix.md` question 9 | Answered, dated, with the ahead-of-time result recorded as contradicting the expectation |
| `docs/known-good-matrix.md` framework row | The machine has 10.0.401 and nothing else; the row claiming a newer release candidate is corrected |
| `ADR-0040` | Pinning and the task runner, with the versions above as evidence |
| `ADR-0041` | Continuous integration topology: the C# side can be built on a hosted runner, which was not known before |
| `ADR-0043` | The framework version revisit trigger, and the ahead-of-time flag |

## What was not tested

Stated so that nobody reads more into this than it holds.

- **Signing.** The installer is a sideload package with a test certificate. Real
  signing needs a certificate this project does not have.
- **A hosted runner.** Everything above ran on the reference hardware, which has
  Visual Studio installed. The claim that a hosted runner can do the same rests
  on the empty workload list and the package references, which is strong but is
  an inference. `code-quality.yml` tests it for real the first time the C# side
  exists.
- **The framework under ahead-of-time compilation at scale.** One window, one
  text block. See reason 2 above.
- **Capture-border suppression for an unpackaged application**, which is matrix
  question 7 and belongs to the overlay experiment, not this one.
