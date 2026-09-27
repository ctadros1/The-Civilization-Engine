# Save/load and schema evolution for long-lived worlds

| Field | Value |
|---|---|
| ID | 01-05 |
| Needed by | M0 (Foundations) |
| Priority | High |
| Informs | PROJECT_PLAN.md §3.5 |
| Status | Not started |

**Why TCE needs this:** TCE worlds are endless and development runs for years; saves must stay loadable, fast, crash-safe and migratable.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Save/load and schema evolution for long-lived worlds

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE worlds are endless and development runs for years; saves must stay loadable, fast, crash-safe and migratable.

Research question: What is the most robust way to save, version and migrate a large, long-lived simulation state?

Cover:
- Binary formats: FlatBuffers vs Cap'n Proto vs rkyv/bincode vs custom containers; zero-copy reads and evolution rules
- Save migration strategies in long-lived games (Factorio, Paradox titles, Dwarf Fortress, RimWorld)
- Autosave without stalling the simulation (copy-on-write, background serialization)
- Crash-safe writes on Windows (atomic rename, flush semantics, checksums)
- Save size and load-time benchmarks for 1–5 GB states

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `05-persistence-saves.report.md`, then change **Status** above to Done.
