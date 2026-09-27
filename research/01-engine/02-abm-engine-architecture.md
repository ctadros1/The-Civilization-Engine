# Architecture of large-scale agent-based simulation engines

| Field | Value |
|---|---|
| ID | 01-02 |
| Needed by | M0 (Foundations) |
| Priority | High |
| Informs | PROJECT_PLAN.md §3, §4 |
| Status | Not started |

**Why TCE needs this:** The kernel must simulate 10k–50k individual people with visible daily life and still fast-forward decades.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Architecture of large-scale agent-based simulation engines

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. The kernel must simulate 10k–50k individual people with visible daily life and still fast-forward decades.

Research question: Which engine architectures let 10k–100k individually simulated people run fast enough for real-time observation with strong time acceleration?

Cover:
- Data-oriented storage: struct-of-arrays, generational IDs, archetype ECS vs plain tables; cache behavior at 50k+ agents
- Time-stepped vs discrete-event vs hybrid scheduling; event queues for activity-based agents
- How MATSim, FLAME GPU, Repast HPC, Mesa, Dwarf Fortress, RimWorld, Songs of Syx and Cities: Skylines 2 (Unity DOTS) structure their simulations
- Spatial indexing for agent queries (uniform grids, quadtrees, R-trees)
- Benchmarks: agent decisions per second on a modern desktop CPU

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `02-abm-engine-architecture.report.md`, then change **Status** above to Done.
