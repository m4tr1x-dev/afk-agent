# Toolchain spikes, 2026-09-12 — transcripts

Four throwaway projects, nothing committed. Everything below ran on the
reference hardware with `dotnet` on the command line and no Visual Studio
process involved.

## Environment

| | |
| --- | --- |
| Operating system | Windows 11 Pro 26200 |
| Framework | .NET SDK 10.0.401, the only one installed |
| Visual Studio | 2026 Insiders 18.11 (installed; not used by any command here) |
| Windows SDK | 10.0.26100 |
| Workloads | none |

```text
> dotnet --version
10.0.401

> dotnet workload list
Installed Workload Id      Manifest Version      Installation Source
-------------------------------------------------------------------
(none)
```

## 1. The solution format

```text
> dotnet new sln --name Probe
The template "Solution File" was created successfully.

> ls
Probe.slnx

> cat Probe.slnx
<Solution>
</Solution>

> dotnet sln --help
  ...
  migrate                Generate a .slnx file from a .sln file.
```

**The new format is the default.** No flag was passed and no choice was offered.
The converter exists for repositories that still carry the old one.

## 2. The user-interface framework, built from the command line

Project file, in full:

```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>WinExe</OutputType>
    <TargetFramework>net10.0-windows10.0.26100.0</TargetFramework>
    <TargetPlatformMinVersion>10.0.19041.0</TargetPlatformMinVersion>
    <RootNamespace>AfkAgent.Spike</RootNamespace>
    <Platforms>x64</Platforms>
    <RuntimeIdentifier>win-x64</RuntimeIdentifier>
    <UseWinUI>true</UseWinUI>
    <Nullable>enable</Nullable>
    <WindowsPackageType>None</WindowsPackageType>
    <WindowsAppSDKSelfContained>true</WindowsAppSDKSelfContained>
    <PublishAot>false</PublishAot>
    <EnableMsixTooling>true</EnableMsixTooling>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Microsoft.WindowsAppSDK" Version="2.4.0" />
    <PackageReference Include="Microsoft.Windows.SDK.BuildTools" Version="10.0.26100.4948" />
  </ItemGroup>
</Project>
```

```text
> dotnet build -c Release
Build succeeded.
    0 Warning(s)
    0 Error(s)
```

### The entry-point collision

First attempt carried a hand-written `Program.cs`:

```text
error CS0017: Program has more than one entry point defined.
```

The markup compiler generates `Main` from `App.xaml`. Deleting `Program.cs`
fixed it. The message is accurate and the cause is not where a reader looks
first, because nothing in the project file mentions an entry point.

## 3. Single-project packaging

```text
> dotnet publish -c Release ^
    -p:GenerateAppxPackageOnBuild=true ^
    -p:AppxPackageDir=out/ ^
    -p:UapAppxPackageBuildMode=SideloadOnly ^
    -p:AppxBundle=Never

> ls out/Pack_0.0.0.1_x64_Test/
Add-AppDevPackage.ps1
Add-AppDevPackage.resources/
Dependencies/
Install.ps1
Pack_0.0.0.1_x64.msix

> ls -l out/Pack_0.0.0.1_x64_Test/Pack_0.0.0.1_x64.msix
28,286,658 bytes

> ls Dependencies/
arm64/  win32/  x64/  x86/     each containing Microsoft.WindowsAppRuntime.2.msix
```

No `.wapproj`. No workload. The runtime dependency packages are laid out for
four architectures and the sideload script is generated alongside.

**Not tested: signing.** This is a test-certificate sideload package.

## 4. Ahead-of-time compilation

### First attempt, and why it failed

```text
> dotnet publish -c Release -p:PublishAot=true
MSB3073: the command ""vswhere.exe" -latest -prerelease -products * ..."
exited with code 9009.
```

Code 9009 is "command not found". `vswhere.exe` lives in
`C:\Program Files (x86)\Microsoft Visual Studio\Installer` and that directory
was not on `PATH`.

This is an environment fault, not a framework limitation. Worth writing down
because the message names a Visual Studio tool, and the obvious reading —
"ahead-of-time compilation requires Visual Studio" — would have closed the
question in the wrong direction with a plausible-looking reason.

### Second attempt

```text
> set PATH=%PATH%;C:\Program Files (x86)\Microsoft Visual Studio\Installer
> dotnet publish -c Release -p:PublishAot=true

> ls -l bin/Release/net10.0-windows10.0.26100.0/win-x64/native/
Probe.exe      4,385,280 bytes
Probe.pdb     18,255,872 bytes
```

Evidence that this is a native binary rather than a bundle carrying a runtime:

```text
> Test-Path publish/Probe.dll
False
> Test-Path publish/Probe.runtimeconfig.json
False
> (Get-ChildItem publish -Filter *.dll).Count
49                      # the framework's self-contained natives, not managed assemblies
```

### It runs

Re-verified on 2026-09-12, by launching the published binary and reading the
process back:

```text
> $p = Start-Process -FilePath publish/Probe.exe -PassThru
> Start-Sleep -Seconds 6; $p.Refresh()
running: pid 13000, window 'afk-agent spike', working set 87.9 MB
```

The window title comes from the markup, so this is the framework starting and
composing, not merely a process that failed to exit.

### Sizes, for the record

| | |
| --- | --- |
| Native binary | 4,385,280 bytes |
| Published directory, whole | 133 MiB |
| Working set, one window with one text block | 87.9 MiB |

The published directory is dominated by the framework's self-contained natives,
which ahead-of-time compilation does not shrink. The saving is in the managed
half only.
