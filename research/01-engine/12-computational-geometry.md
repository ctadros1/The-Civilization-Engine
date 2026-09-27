# Computational geometry for parcels, footprints and roofs

| Field | Value |
|---|---|
| ID | 01-12 |
| Needed by | M1 (A band settles) |
| Priority | Medium |
| Informs | PROJECT_PLAN.md §5.1 |
| Status | Not started |

**Why TCE needs this:** Plots, building footprints, roofs and road offsets are polygons the kernel must manipulate robustly.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Computational geometry for parcels, footprints and roofs

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Plots, building footprints, roofs and road offsets are polygons the kernel must manipulate robustly.

Research question: Which computational geometry techniques and Rust libraries handle land parcels, building footprints, roofs and road offsets robustly?

Cover:
- Robust polygon boolean operations, offsetting and simplification; floating-point robustness
- Straight skeleton for hip and gable roofs; medial axis
- Parcel subdivision and polygon splitting
- Rust crates (geo, i_overlay, others): maturity and performance

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `12-computational-geometry.report.md`, then change **Status** above to Done.
