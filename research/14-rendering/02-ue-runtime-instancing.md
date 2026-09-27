# Runtime assembly of Nanite modular kits with ISM/HISM

| Field | Value |
|---|---|
| ID | 14-02 |
| Needed by | M2 (First light in Unreal) |
| Priority | High |
| Informs | PROJECT_PLAN.md §6 |
| Status | Not started |

**Why TCE needs this:** Every TCE building is assembled at runtime from instanced Nanite kit meshes.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Runtime assembly of Nanite modular kits with ISM/HISM

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Every TCE building is assembled at runtime from instanced Nanite kit meshes.

Research question: What are the best practices and limits for assembling tens of thousands of buildings at runtime from Nanite kit meshes with instanced static meshes in UE 5.8?

Cover:
- ISM vs HISM vs instanced actors; instance counts per component; cost of adding and removing instances
- Per-instance custom data for materials (weathering, color, window lights)
- Culling and HLOD for runtime-generated content; World Partition interaction
- Memory and GPU budgets

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `02-ue-runtime-instancing.report.md`, then change **Status** above to Done.
