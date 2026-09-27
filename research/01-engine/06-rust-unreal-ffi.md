# Embedding a Rust library in an Unreal Engine 5 plugin

| Field | Value |
|---|---|
| ID | 01-06 |
| Needed by | M2 (First light in Unreal) |
| Priority | High |
| Informs | PROJECT_PLAN.md §3.1, §3.2 |
| Status | Not started |

**Why TCE needs this:** The TCE kernel is a Rust cdylib loaded by a UE plugin, running on its own threads, and it must ship in packaged Windows builds.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Embedding a Rust library in an Unreal Engine 5 plugin

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. The TCE kernel is a Rust cdylib loaded by a UE plugin, running on its own threads, and it must ship in packaged Windows builds.

Research question: What is the best-practice way to load a Rust cdylib from a UE5 C++ plugin on Windows, run it on its own threads, and ship it in packaged builds?

Cover:
- Build integration: invoking cargo from UnrealBuildTool, cbindgen headers, MSVC toolchain and CRT compatibility, PDBs
- Threading: running a simulation off the game thread; synchronizing with game and render threads; triple buffering
- Memory ownership across FFI, panics (`catch_unwind`), error propagation, crash reporting
- Reloading the DLL during editor sessions; interaction with Live Coding
- Existing projects or plugins embedding Rust in Unreal, and their lessons

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `06-rust-unreal-ffi.report.md`, then change **Status** above to Done.
