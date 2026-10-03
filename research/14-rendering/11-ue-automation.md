# Automating Unreal Engine for AI agents and CI

**Engineering report for The Civilization Engine — September 27, 2026**

## Executive recommendation

**Treat Unreal as a programmable asset compiler, renderer, and integration-test host—not as a GUI that an AI agent must learn to operate.**

For TCE, I recommend this stack:

* **Rust and native tests** for simulation behavior, persistence, determinism, and accelerated history.
* **Versioned text manifests, Python orchestration, and a small C++ editor module** for importing and generating Unreal assets.
* **Unreal Automation Tests and functional test maps** for engine-level assertions, with **Gauntlet** managing packaged-game tests.
* **Unreal Build Tool and AutomationTool** for compilation, cooking, staging, and packaging on a **self-hosted Windows runner**.
* **A separate real-GPU test lane** for screenshots and performance.
* **UE 5.8’s native MCP integration** as an optional interactive interface for agents—not the authoritative CI pipeline.

The important 2026 development is that Epic now supplies an experimental MCP integration in UE 5.8. That reduces the need to build a general-purpose editor-control server, but it does not eliminate the need for deterministic generators, explicit completion conditions, or independent tests. UE 5.8 was released on June 23, 2026. [Unreal Engine](https://www.unrealengine.com/en-US/news/unreal-engine-5-8-is-now-available)

“Almost all work without the GUI” is a realistic architectural goal. “Every arbitrary editor operation has a complete, stable Python API” is not. Design TCE’s authoring conventions around the operations you can reliably compile, inspect, and test.

**Evidence boundary:** The commands below are documentation-grounded templates, not commands executed against a TCE checkout. Project-specific scripts, test groups, and asset types are identified as such. Most Epic references currently display UE 5.8; historical examples and the older Python API reference are explicitly labeled.

---

## 1. Options: which automation mechanism belongs where?

### 1.1 The main mechanisms

| Mechanism | How it works | Best role in TCE | Trade-offs and maturity |
| --- | --- | --- | --- |
| **Python editor scripting** | Python calls Unreal’s reflected editor APIs and project-exposed functions. | Manifest parsing, imports, bulk property changes, asset inspection, generation orchestration. | Low implementation cost; coverage depends on exposed APIs. Epic still labels the feature Experimental. Python is editor-side, not packaged-game scripting. |
| **C++ editor module and commandlets** | Native code implements asset operations; commandlets expose batch entry points. | Reliable generators, custom validation, unsupported Python operations, large batches. | More code and compilation, but stronger access and explicit lifecycle control. |
| **Editor Utility Blueprints/Widgets** | Editor-only Blueprint tools, optionally with custom panels. | Optional human controls over the same underlying operations. | Useful for artists, but widget-driven workflows introduce unnecessary UI dependencies into CI. Widgets remain labeled Beta. |
| **UE 5.8 Unreal MCP** | An in-editor server exposes typed tools to compatible AI clients. | Interactive inspection, experimentation, invoking project-specific tools. | Experimental, incomplete API surface, mutable editor state, and additional security considerations. |
| **Automation Tests and functional tests** | Assertions run inside Unreal, with support for asynchronous and world-based testing. | Asset, bridge, gameplay, screenshot, and integration checks. | Established infrastructure; tests must use the correct editor/client context. |
| **Gauntlet** | C# orchestration launches and monitors Unreal sessions and collects results. | Packaged-game smoke tests, soak tests, crashes, timeouts, performance scenarios. | Additional framework to learn; it manages execution rather than building the game or supplying all assertions. |
| **UBT, UAT, BuildGraph** | Compile code, execute build/cook/package operations, and optionally express them as a dependency graph. | The repeatable build pipeline. | Established production infrastructure. BuildGraph is valuable when a simple script becomes hard to maintain. |

These distinctions follow Epic’s scripting, commandlet, testing, Gauntlet, and build documentation. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/scripting-the-unreal-editor-using-python)

### 1.2 Python: use the right execution mode

There are two materially different startup paths:

```
UnrealEditor-Cmd.exe TCE.uproject -run=pythonscript -script="build_assets.py"
```

This runs the Python commandlet with reduced editor startup. It does **not automatically load the project’s normal map**.

```
UnrealEditor.exe TCE.uproject -ExecutePythonScript="build_assets.py"
```

This loads the full editor environment and runs the script after startup. It is more suitable for operations needing that environment. Epic specifically warns against using `-ExecCmds` to launch startup Python because execution can occur before the editor is ready. Python availability does not extend to cooked gameplay. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/scripting-the-unreal-editor-using-python)

For TCE, start with the commandlet path. Escalate individual operations to a full-editor process only when their actual dependencies require it.

A key design rule is **batching**. Do not start Unreal once per mesh, material, or building definition. Parse the dependency graph first, then process a coherent batch in one invocation.

### 1.3 Native commandlets: the durable automation boundary

A custom `UCommandlet` exposes a `Main(const FString& Params)` entry point. UE 5.8 also exposes commandlet exit-code controls. This is a suitable place to implement a stable contract such as:

```
-run=TCEBuildAssets -Manifest=... -OutputReport=...
-run=TCEValidateContent -OutputReport=...
```

Those names would be TCE-defined classes, not built-in commands. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UCommandlet)

Put the actual implementation in reusable services, not inside a large commandlet method. Python, a commandlet, an Editor Utility, and MCP should all call the **same asset-generation functions**.

Existing builders demonstrate the pattern: Epic supplies commandlets for World Partition HLODs, navigation, minimaps, actor resaving, and PCG generation. Some explicitly require `-AllowCommandletRendering`. “Commandlet” therefore does not necessarily mean “GPU-free.” [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/world-partition-builder-commandlet-reference)

### 1.4 Native MCP: useful, but keep it outside the trusted build path

UE 5.8’s implementation uses the `ModelContextProtocol` plugin, with toolsets enabled through `AllToolsets` or selected individual toolsets. It supports project-defined tools. Calls execute serially on Unreal’s game thread; clients should not overlap them. The default server is local-only and has no authentication. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/unreal-mcp-in-unreal-editor)

My recommendation is to expose narrow operations such as:

```
inspect_asset
validate_manifest
preview_generation_plan
generate_asset_batch
run_named_test_suite
capture_named_fixture
```

Avoid making unrestricted “execute arbitrary editor code” the normal interface. Have agents submit manifests and inspect structured results rather than improvise hundreds of stateful edits.

Use MCP for the **interactive development loop**. Have CI reproduce accepted changes through ordinary scripts or commandlets in a fresh process. Keep agent-control tooling out of Shipping builds.

---

## 2. Generate assets from text, but distinguish source from compiled output

### 2.1 A proposed TCE content architecture

I recommend four content categories:

| Category | Authoritative source | Generated or consumed output |
| --- | --- | --- |
| Simulation definitions | Versioned Rust-compatible schemas and data | Kernel configuration, validation fixtures |
| Architectural kits | DCC source files, export files, text manifests | Static meshes, material instances, kit data assets |
| Presentation definitions | Text descriptions plus curated native templates | Appearance catalogs, lighting presets, test scenes |
| Emergent worlds | Rust state, seeds, snapshots, event history | Runtime Unreal representations—not authored maps for every possible settlement |

This last distinction is essential. **TCE should compile a library of building components and construction rules, not pre-generate an asset for every future house or city.** Runtime settlement growth should assemble those components from Rust-owned state.

### 2.2 A manifest should specify intent, not reproduce editor clicks

For example, the following is a **proposed TCE schema**, not a built-in Unreal format:

```
{
  "schema_version": 1,
  "id": "kit.timber.wall.4m",
  "source": "Art/Exports/timber_wall_4m.fbx",
  "destination": "/Game/TCE/Generated/Timber/SM_Wall_4m",
  "import_profile": "static_mesh_v3",
  "expected_size_cm": [400, 20, 300],
  "nanite": true,
  "material_slots": {
    "Wall": "/Game/TCE/Materials/MI_Timber"
  },
  "sockets": [
    {
      "name": "SnapLeft",
      "location_cm": [-200, 0, 0]
    }
  ]
}
```

The generator should resolve that intent into Unreal operations. The import profile should explicitly define scale conversion, normal handling, collision policy, UV expectations, and other settings that must not depend on an import dialog’s remembered state.

A generated lock/report file should record the input hashes, generator version, import profile version, exact engine build, output package paths, and validation results.

### 2.3 What can be generated reliably?

**Data assets and catalogs.** Prefer native `UPrimaryDataAsset` subclasses for kit definitions, appearance catalogs, and other typed runtime data. Unreal’s Asset Manager supports discovery, loading, bundles, and cooking rules; data-only Blueprint subclasses are also available when inheritance is genuinely useful. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/asset-management-in-unreal-engine)

**Meshes and textures.** Use automated import tasks or explicit Interchange pipelines. Interchange separates source translation, pipeline processing, and asset creation; imports may be asynchronous. Your generator must wait for completion and inspect actual outputs. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/importing-assets-using-interchange-in-unreal-engine)

**Material instances.** Prefer a small, curated family of master materials with generated instances and parameter sets. My recommendation is to avoid generating a unique master graph for every architectural variation: it expands the surface that agents must maintain and makes validation harder.

**Maps and fixtures.** Generate standardized test maps and preview scenes through editor APIs. Use built-in builders where appropriate rather than recreating their behavior.

**Blueprints, animation graphs, Control Rigs, and Niagara systems.** Treat these as a spectrum, not a binary “scriptable or not” category. For a solo project, I would use native behavior plus curated templates and constrained parameters. Unrestricted graph synthesis should be an exceptional, separately tested authoring operation.

One concrete version trap: the **UE 5.6 `AssetImportTask` API reference** says that `destination_name` is ignored when using Interchange; naming belongs in the pipeline. It also exposes completion checks and an object retrieval method that can wait for asynchronous import. Verify these contracts against TCE’s installed 5.8 API rather than copying older examples unquestioningly. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/AssetImportTask?application_version=5.6)

### 2.4 Make generation reproducible and recoverable

I recommend the following generator contract:

1. **Validate and plan before writing.** Reject duplicate IDs, missing dependencies, invalid paths, and unsupported schema versions.
2. **Reconcile deterministically.** Update stable package paths, skip unchanged inputs, and save only changed outputs.
3. **Validate the result.** Check dimensions, sockets, collision, material slots, references, and project-specific budgets.
4. **Publish a completion record only after success.** A partially imported batch is not a successful build.

Use a dedicated generated-content subtree and prohibit two processes from modifying the same project packages concurrently. For risky operations, regenerate in a disposable checkout. Do not treat editor Undo as crash recovery.

For rebuild verification, compare **semantic exports**—properties, references, geometry statistics, and dependency lists—rather than assuming `.uasset` files will be byte-identical.

### 2.5 Prevent the classic “works in editor, missing when packaged” failure

Runtime references must be visible to the cooking strategy. Do not assume that a path written in arbitrary JSON causes the target asset to be included.

Use generated primary assets, bundles, and explicit cooking rules to make the dependency closure discoverable. Then run a packaged test that loads **every asset referenced by every production manifest**. Asset Manager discovery and cooking controls provide the underlying mechanisms. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/asset-management-in-unreal-engine)

---

## 3. Testing: separate simulation correctness, engine integration, and presentation

### 3.1 Recommended test layers

| Layer | Execution environment | What TCE should verify |
| --- | --- | --- |
| **Rust unit/property tests** | No Unreal process | Conservation rules, agent state transitions, scheduling, save/load, deterministic replay |
| **Rust scenario tests** | Standalone kernel executable | Multi-seed decades/centuries, population bounds, deadlocks, throughput, memory growth |
| **Unreal non-rendering tests** | Editor/client with an appropriate headless configuration | DLL ABI, serialization bridge, catalog resolution, lifecycle and ownership |
| **Content validation** | Editor commandlet | Asset structure, naming, dimensions, sockets, collision, missing references |
| **Rendered functional tests** | Real renderer | Visual fixtures, occupancy lighting, construction stages, appearance, UI |
| **Packaged-game tests** | Development package, later Shipping smoke test | Cooking, DLL staging, startup, save/load, actual runtime behavior |
| **Performance/soak tests** | Packaged game on controlled hardware | Frame-time distributions, stalls, VRAM, memory growth, sustained simulation |

Unreal’s Automation Test Framework provides engine-level tests, specifications, functional tests, and screenshot testing. Epic distinguishes it from lower-level unit testing and explicitly emphasizes test independence and cleanup. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/automation-test-framework-in-unreal-engine)

For TCE, this means keeping most civilization rules out of Unreal tests. Loading the engine to test a Rust food-allocation invariant is unnecessary coupling.

### 3.2 Data Validation: choose validators that actually run in CI

Unreal provides:

```
UnrealEditor-Cmd.exe TCE.uproject -run=DataValidation
```

Custom validation can live in native object validation methods or validator classes. A significant caveat is that the documented command-line path runs **C++ validation by default**; Python validators require registration and appropriate initialization. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/data-validation-in-unreal-engine)

I would put mandatory TCE validators in C++. Add a deliberately invalid fixture to a validator self-test so that CI can prove the validation system is active.

### 3.3 Automation tests versus Gauntlet

Use Automation Tests for assertions. Use Gauntlet to manage the process containing those assertions.

Gauntlet can launch sessions, monitor execution, parse logs and crashes, collect artifacts, and coordinate optional in-game test controllers. It **does not create the build** that it tests. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/gauntlet-automation-framework-overview-in-unreal-engine)

Start with a small Gauntlet suite:

* Boot, load a known Rust snapshot, run, save, reload, and exit.
* Replay a camera path through a generated settlement.
* Run a population/streaming stress scenario and report results.

Do not build an elaborate distributed test framework before these three cases work locally.

### 3.4 A zero exit code is insufficient

My proposed pass condition is:

```
process succeeded
AND expected tests were discovered
AND expected tests completed
AND no test failed
AND required artifacts exist
AND the report identifies the expected build
```

Treat “zero tests found,” missing screenshot output, timeout, stale report, and interrupted execution as failures.

Unreal can export automation results as JSON with related HTML files. Use those structured results rather than only searching logs for the word “Error.” [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/run-automation-tests-in-unreal-engine)

---

## 4. Headless and offscreen rendering for screenshot regression

### 4.1 Three modes that must not be confused

| Mode | What it means | Appropriate use |
| --- | --- | --- |
| **`-NullRHI`** | No real rendering hardware interface | Logic and supported non-rendering checks |
| **`-RenderOffScreen` with a real RHI** | Rendering without the normal onscreen presentation path | GPU-backed image capture and rendering tests |
| **Windowed interactive session** | Normal application window and desktop environment | Fallback or tests requiring actual desktop/UI behavior |

`UnrealEditor-Cmd.exe` does not by itself mean “no renderer.” Epic documents `nullrhi`, `RenderOffScreen`, and `AllowCommandletRendering` as separate controls. **Do not use `-NullRHI` for a test intended to verify rendered pixels.** [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/unreal-engine-command-line-arguments-reference)

For TCE, begin with D3D12 offscreen rendering on the actual Windows GPU runner. Make a startup probe that verifies renderer initialization, image dimensions, successful readback, and nonempty output.

Do not assume that a Windows service session behaves like a logged-in desktop. Microsoft documents interactive-process requirements for desktop UI testing and warns that remote-desktop disconnection can disrupt such tests. That does not prove every Unreal offscreen workload needs an interactive session; it does justify testing the exact runner configuration and retaining an interactive fallback. [Microsoft Learn](https://learn.microsoft.com/en-us/azure/devops/pipelines/test/ui-testing-considerations?view=azure-devops)

### 4.2 Build deterministic fixtures before comparing images

A screenshot fixture should specify:

**World state:** Rust snapshot hash, simulation tick, settlement configuration, construction/ruin states, and occupancy.

**Presentation state:** Camera transform, resolution, exposure, time of day, season, weather, animation time, scalability settings, and random seeds.

**Readiness:** Required assets loaded, relevant asynchronous work completed, and enough rendered frames for the intended temporal rendering state.

Avoid “sleep five seconds, then capture.” Prefer explicit readiness checks plus a bounded timeout. A fixed warm-up frame count can supplement those checks, but should not replace them.

Use two kinds of visual tests:

**Structural tests** simplify unstable effects to catch wrong meshes, misplaced sockets, missing materials, visibility errors, and UI layout changes.

**Rendering tests** retain the shipping rendering configuration to catch lighting, shadows, transparency, weathering, and temporal artifacts.

Unreal’s screenshot tools distinguish gameplay-oriented and rendering-oriented defaults, provide local/global tolerances, and include separate functional UI screenshot support. Those controls are useful starting points—not universal thresholds for TCE. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/screenshot-comparison-tool-in-unreal-engine)

### 4.3 Calibrate tolerance rather than hiding failures

Repeat an unchanged fixture several times before establishing its baseline. Measure the natural variation, then choose tolerances that distinguish that variation from defects.

Store the approved image, incoming image, difference image, test result, engine build, driver, RHI, settings, and fixture hash together. Keep rendering baselines tied to a defined environment; an engine or driver upgrade should trigger a deliberate baseline review.

Never let the same autonomous change automatically approve its own replacement baseline.

### 4.4 Movie Render Queue has a different job

Movie Render Queue is useful for asset contact sheets, high-quality reference shots, and offline sequences. Epic exposes programmatic queue/job configuration and render-completion results, including success status and generated file paths. Some rendering features are unavailable in runtime builds. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/movie-render-queue-in-runtime-in-unreal-engine)

For TCE, use it for **offline inspection artifacts**, not as evidence of 60 fps gameplay. An offline sequence with different sampling and temporal history is not the same workload as a real-time camera moving through a populated city.

---

## 5. Building and packaging on Windows

### 5.1 Start with a small, pinned toolchain

Use a prebuilt engine installation initially unless TCE needs engine modifications. Pin the exact UE build/hotfix, compiler, Windows SDK, Rust toolchain, plugin revisions, import profiles, and GPU driver.

The current UE 5.8 compatibility table specifies **Visual Studio 2022 17.14+ or Visual Studio 2026 18.0+**, with Visual Studio 2026 recommended for general development. This differs from older UE 5.x setup instructions. Use the installed engine’s accepted toolchain as the final authority. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/setting-up-visual-studio-development-environment-for-cplusplus-projects-in-unreal-engine)

Keep generated build products and caches outside the authoritative source set. Preserve a persistent local Derived Data Cache: UE 5.4 and later use Zen for the default local DDC, and Epic documents command-line cache filling. DDC is regenerable derived data, not a substitute for source assets or backups. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/using-derived-data-cache-in-unreal-engine)

### 5.2 Command-line structure

Unreal’s build operations separate compilation, cooking, staging, packaging, deployment, and execution. AutomationTool’s `BuildCookRun` drives these operations without the Project Launcher GUI. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/build-operations-cooking-packaging-deploying-and-running-projects-in-unreal-engine)

The following PowerShell illustrates the shape of the pipeline. `build_assets.py`, `TCE.Headless`, and `TCE.Visual` are project-defined components.

```
$ErrorActionPreference = "Stop"

$UE      = "C:\UE\UE_5.8"
$Root    = "C:\src\TCE"
$Project = "$Root\TCE.uproject"
$Cmd     = "$UE\Engine\Binaries\Win64\UnrealEditor-Cmd.exe"
$UAT     = "$UE\Engine\Build\BatchFiles\RunUAT.bat"

function Invoke-Checked {
    param(
        [string]$Executable,
        [string[]]$Arguments
    )

    & $Executable @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Executable failed with exit code $LASTEXITCODE"
    }
}

# Rust testing/building and DLL staging precede Unreal startup.

Invoke-Checked "$UE\Engine\Build\BatchFiles\Build.bat" @(
    "TCEEditor", "Win64", "Development",
    "-Project=$Project", "-WaitMutex"
)

Invoke-Checked $Cmd @(
    $Project,
    "-run=pythonscript",
    "-script=$Root\Tools\build_assets.py",
    "-unattended", "-nop4", "-nosplash"
)

Invoke-Checked $Cmd @(
    $Project,
    "-run=DataValidation",
    "-unattended", "-nop4", "-nosplash"
)

Invoke-Checked $Cmd @(
    $Project,
    "-NullRHI", "-unattended", "-nop4", "-nosplash",
    "-ExecCmds=Automation RunTest TCE.Headless;Quit",
    "-ReportExportPath=$Root\Artifacts\Headless"
)

Invoke-Checked $UAT @(
    "BuildCookRun",
    "-project=$Project",
    "-platform=Win64",
    "-clientconfig=Development",
    "-build", "-cook", "-stage", "-pak", "-iostore", "-package",
    "-archive", "-archivedirectory=$Root\Artifacts\Package",
    "-unattended", "-nop4", "-utf8output"
)
```

The automation command and report-export form are documented by Epic. The wrapper checks process exit status, but a production runner must additionally enforce timeouts, terminate orphaned child processes, parse test reports, and validate required outputs. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/run-automation-tests-in-unreal-engine)

An editor-hosted visual test invocation would look like:

```
Invoke-Checked $Cmd @(
    $Project,
    "-d3d12", "-RenderOffScreen",
    "-ResX=2560", "-ResY=1440",
    "-unattended", "-nop4", "-nosplash",
    "-ExecCmds=Automation RunTest TCE.Visual;Quit",
    "-ReportExportPath=$Root\Artifacts\Visual"
)
```

The named tests must load and drive their own fixtures. This command is not a generic screenshot generator. Follow editor tests with a **packaged-game** lane so that cooking and runtime-only differences are exercised.

### 5.3 Rust DLL integration: make it explicit and testable

For TCE, I recommend a Rust `cdylib` targeting `x86_64-pc-windows-msvc`, with a deliberately small C ABI:

```
tce_abi_version
tce_create
tce_destroy
tce_step
tce_load_snapshot
tce_save_snapshot
tce_read_presentation_snapshot
```

Use opaque handles, fixed-width fields, explicit lengths, `#[repr(C)]` where structures cross the boundary, and documented ownership. Do not pass Rust `Vec`, `String`, or C++ containers directly through the ABI. Define a no-unwinding boundary: catching Rust panics only works for unwinding panics, not `panic=abort`, and foreign exceptions require separate care. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

Build and stage the DLL **before** launching Unreal. In the Unreal module, either load it explicitly using platform DLL facilities or configure import/delay-load behavior. Declare staging through `RuntimeDependencies`. Windows DLLs must remain loose files available to the operating-system loader; they cannot simply live inside a PAK. Include their dependent libraries as well. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/integrating-third-party-libraries-into-unreal-engine)

My additional TCE safeguards would be:

* Reject ABI/build mismatches during initialization.
* Initialize the simulation explicitly, not as an unavoidable side effect of loading the plugin during asset cooking.
* Restart Unreal after kernel binary changes rather than attempting casual hot replacement.
* Join worker threads and invalidate callbacks before unloading.
* Transfer batched presentation snapshots rather than making one reflected call per citizen.

Test startup, shutdown, save/load, missing DLL, stale DLL, and repeated world creation in a packaged build—not only in the editor.

### 5.4 CI topology for a solo developer

Start with **one self-hosted Windows runner and one serialized GPU lane**, driven by the same checked-in script used locally.

| Trigger | Suggested workload |
| --- | --- |
| Every change | Rust tests, schema checks, affected compilation, targeted asset generation and validation |
| Merge candidate | Broader Unreal tests, Development package, packaged smoke test, visual fixtures |
| Scheduled integration run | Multi-seed long simulation, streaming/soak scenarios, full content checks, performance paths |
| Release candidate | Clean rebuild/cook, Shipping startup and dependency checks, reviewed visual baselines |

GitHub Actions, Azure Pipelines, or another existing orchestrator can trigger this script. Do not begin by operating a build farm.

If using a self-hosted runner, do not execute untrusted pull-request code on the developer workstation. GitHub explicitly warns that such runners can be persistently compromised and that restricting secrets alone does not isolate the host. [GitHub Docs](https://docs.github.com/en/actions/reference/security/secure-use)

BuildGraph becomes attractive once dependency handling, artifact reuse, and multiple build configurations outgrow the script. Horde is a later option: Epic uses it for Unreal and Fortnite development, but its own documentation describes opinionated workflows and integrations. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/buildgraph-for-unreal-engine)

---

## 6. Performance, complexity, and what benchmarks actually establish

### 6.1 Evidence worth using

| Evidence | Reported result | Appropriate interpretation |
| --- | --- | --- |
| **Epic multi-process cooking** | Four subprocesses reduced cooking time by roughly **40%** on a large project; small projects such as Lyra may gain little. | A useful reason to benchmark worker counts, not a guaranteed TCE speedup. |
| **2026 Unreal synthetic-data pipeline** | Cache-aware repeated scene scheduling reduced per-scene rendering time to less than half the cold-start case. | Cache locality can dominate production throughput; this is not a 4070 Ti or gameplay benchmark. |
| **Batching model** | Separate launches cost approximately \(N(S+W)\); one batch costs approximately \(S+NW\), for startup cost \(S\) and per-item work \(W\). | An analytical reason to batch operations, not a measured Unreal speed ratio. |

Epic also warns that RAM exhaustion and insufficient cores can make additional cook processes slower. Its documentation still labels multi-process cooking Beta. The 2026 rendering result comes from a much larger, specialized production system. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/using-multi-process-cooking-for-unreal-engine)

I did not find a trustworthy, directly comparable UE 5.8 benchmark for Python versus native asset generation on your exact hardware. In practice, benchmark the complete operation: import, mesh build, shader work, saving, startup, and cache state—not just Python loop overhead.

### 6.2 What to measure on TCE’s machine

For the **i9 / 64 GB / RTX 4070 Ti 12 GB** configuration, I recommend beginning conservatively:

**One Unreal content-writing process at a time.** Parallel agents can work in separate source worktrees, but asset compilation should have explicit ownership.

**One GPU regression or performance process at a time.** Do not run a local GPU model, another editor, or competing rendering job during measurements.

**Benchmark cooking with 1, 2, and 4 processes.** Record elapsed time, peak RAM, paging, and failures. Epic documents `CookProcessCount` and passing it through UAT’s `AdditionalCookerOptions`. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/using-multi-process-cooking-for-unreal-engine)

**Measure cold and warm paths separately.** A warm cache is appropriate for development iteration; cold-start behavior still matters for reproducibility and clean machines.

### 6.3 Connect automation to the 60 fps requirement

At 60 fps, the frame interval is **16.67 ms**. Automation does not make that budget easier by itself; it makes regressions visible.

For TCE, record:

* CPU and GPU frame-time distributions, especially p95/p99 and long stalls.
* Simulation ticks per second and requested time acceleration actually achieved.
* Peak and steady-state RAM/VRAM.
* Asset streaming, shader-related stalls, and world-chunk creation costs.

Use Unreal Insights traces for diagnosis and compact numeric summaries for CI comparison. Epic provides a structured tracing system and persisted trace files; a test need not depend on manually opening the profiler to decide whether it passed. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine)

Keep **fixed-tick correctness tests separate from wall-clock performance tests**. A deterministic replay can advance the correct number of ticks while taking far too long.

For initial acceptance targets, I would reserve margin rather than allowing every subsystem to consume the full frame interval—for example, targeting GPU time below roughly 14 ms in the chosen stress fixture. That is a proposed engineering target, not a demonstrated result on your machine. CPU and GPU timings overlap; do not simply add them.

Most importantly, distinguish **50,000 simulated citizens** from **50,000 fully animated, high-detail visible characters**. Your test fixtures should record both populations so that a change in visibility or representation does not masquerade as a simulation performance improvement.

---

## 7. Precedents and their transferable lessons

| Precedent | What it demonstrates | Lesson for TCE and version caveat |
| --- | --- | --- |
| **Epic: Unreal/Fortnite build infrastructure** | BuildGraph and Horde support automated builds, artifact workflows, and testing used in Epic development. | Reuse UBT/UAT and explicit build stages. The existence of a large production system does not mean a solo developer should deploy all of it. Current documentation is presented for UE 5.8. |
| **Daedalic’s test-automation plugin, used for *The Lord of the Rings: Gollum*** | Gauntlet integration, parameterized tests, simulated inputs, timeouts, performance camera paths, reports, and screenshots. | Borrow test structure and crash-resilient reporting. Its documented support is **UE 4.23–4.27**, not a drop-in UE 5.8 guarantee. |
| **CARLA rendering automation** | Separates disabled rendering from offscreen rendering with GPU-produced camera data. | Keep simulation and image production distinct. The cited instructions discuss **CARLA 0.9.12 / UE 4.26** and include Linux-specific commands. |
| **UnrealCV** | A command API controls virtual worlds and retrieves images/data from Unreal, including packaged applications. | Expose semantic operations instead of mouse gestures. The original research is UE4-era; the current repository advertises UE 5.2 and later, which is not proof that every feature is validated on 5.8. |
| **Wang et al., September 2026 synthetic-data pipeline** | Records trajectories during simulation, then launches a new process to replay them through Movie Render Queue; adds cache-aware scheduling and partial-output recovery. | Separate simulation generation from final rendering and preserve recoverable intermediate results. The paper reports a 200-GPU production cluster; it does not establish TCE performance or a Windows UE 5.8 compatibility matrix. |

These are documented implementations, but they solve different problems. Their strongest common lesson is **explicit interfaces, controlled process lifecycles, and inspectable artifacts**, not “an autonomous agent can reliably perform arbitrary editor work.” [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/buildgraph-for-unreal-engine)

A useful historical talk is Axel Riffard’s **[“Press Button, Drink Coffee: An Overview of UE4 Build Pipeline and Maintenance”](https://www.docswell.com/s/EpicGamesJapan/ZM7625-UE4_CEDEC19_BuildPipeline_En?utm_source=chatgpt.com)**, from Epic Games Japan at CEDEC 2019. Use it for pipeline structure and maintenance ideas, not contemporary UE 5.8 flag or toolchain requirements. [Docswell](https://www.docswell.com/s/EpicGamesJapan/ZM7625-UE4_CEDEC19_BuildPipeline_En)

---

## 8. Recommended implementation order

### First: establish a reproducible packaged vertical slice

Build the Rust DLL, build Unreal, generate one kit asset, validate it, package the game, load one snapshot, render one known camera view, save, reload, and exit successfully.

This proves the entire chain before expanding the content system.

### Next: make the authoring pipeline declarative

Introduce schema validation, stable IDs, import profiles, incremental generation, semantic asset reports, and a generated runtime catalog. Give the agents the exact local API definitions and a small number of project-owned entry points.

### Then: add independent regression gates

Separate kernel correctness, asset validation, visual comparison, packaged behavior, and performance. Require expected test counts and artifacts. Keep baseline approval and changes to test expectations reviewable independently of implementation changes.

### Finally: improve iteration speed

Add persistent editor/MCP workflows, larger batches, cache warming, tuned cooking parallelism, and eventually BuildGraph where measurements justify the complexity.

**The central TCE rule should be: an agent’s editor session is disposable; the checked-in inputs and reproducible pipeline are authoritative.**

---

## Source guide and version applicability

The implementation references below are the most useful starting points. The historical code and papers are linked separately because their compatibility assumptions differ.

| Area | Primary references | Applicability |
| --- | --- | --- |
| Agent interface | [Unreal MCP](https://dev.epicgames.com/documentation/unreal-engine/unreal-mcp-in-unreal-editor?utm_source=chatgpt.com) | UE 5.8; Experimental |
| Editor automation | [Python scripting](https://dev.epicgames.com/documentation/unreal-engine/scripting-the-unreal-editor-using-python?utm_source=chatgpt.com), [UCommandlet API](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/UCommandlet?utm_source=chatgpt.com) | Current UE 5.8 pages; verify installed APIs |
| Content generation | [Interchange](https://dev.epicgames.com/documentation/unreal-engine/importing-assets-using-interchange-in-unreal-engine?utm_source=chatgpt.com), [Asset Management](https://dev.epicgames.com/documentation/en-us/unreal-engine/asset-management-in-unreal-engine?utm_source=chatgpt.com), [World Partition builders](https://dev.epicgames.com/documentation/en-us/unreal-engine/world-partition-builder-commandlet-reference?utm_source=chatgpt.com) | Current UE 5.8 documentation |
| Tests | [Automation Framework](https://dev.epicgames.com/documentation/unreal-engine/automation-test-framework-in-unreal-engine?utm_source=chatgpt.com), [CLI test execution](https://dev.epicgames.com/documentation/unreal-engine/run-automation-tests-in-unreal-engine?utm_source=chatgpt.com), [Gauntlet overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/gauntlet-automation-framework-overview-in-unreal-engine?utm_source=chatgpt.com) | Current UE 5.8 documentation |
| Visual regression | [Screenshot comparison](https://dev.epicgames.com/documentation/unreal-engine/screenshot-comparison-tool-in-unreal-engine?utm_source=chatgpt.com), [command-line flags](https://dev.epicgames.com/documentation/unreal-engine/unreal-engine-command-line-arguments-reference?utm_source=chatgpt.com) | Current UE 5.8 documentation; validate exact RHI/runner |
| Builds and caching | [Build operations](https://dev.epicgames.com/documentation/unreal-engine/build-operations-cooking-packaging-deploying-and-running-projects-in-unreal-engine?utm_source=chatgpt.com), [BuildGraph](https://dev.epicgames.com/documentation/unreal-engine/buildgraph-for-unreal-engine?utm_source=chatgpt.com), [multi-process cooking](https://dev.epicgames.com/documentation/unreal-engine/using-multi-process-cooking-for-unreal-engine?utm_source=chatgpt.com) | Current UE 5.8 pages; cooking measurements are workload-specific |
| Rust integration | [Unreal third-party libraries](https://dev.epicgames.com/documentation/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com), [Rustonomicon FFI](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | UE 5.8 plus current Rust guidance |
| Historical implementation code | [Daedalic test automation](https://github.com/DaedalicEntertainment/ue4-test-automation?utm_source=chatgpt.com), [UnrealCV](https://github.com/unrealcv/unrealcv?utm_source=chatgpt.com) | UE4-specific Daedalic support; UnrealCV branch/plugin compatibility must be checked |
| Research | [UnrealCV, 2016](https://arxiv.org/abs/1609.01326?utm_source=chatgpt.com), [Unreal video-production pipeline, September 2026](https://arxiv.org/abs/2609.03557?utm_source=chatgpt.com) | Architectural evidence, not TCE hardware benchmarks |

**Bottom line:** TCE does not need a robot that knows where Unreal’s buttons are. It needs a small, well-tested content compiler and runtime bridge, with Unreal’s existing build and test infrastructure behind them. That approach makes AI-authored changes reproducible, reviewable, and replaceable without turning editor state into an undocumented dependency.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92984-68f4-83ea-9049-8049fa72d9a2)
