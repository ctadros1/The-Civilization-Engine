# Runtime-generated terrain in packaged UE5 builds

| Field | Value |
|---|---|
| ID | 14-01 |
| Needed by | M2 (First light in Unreal) |
| Priority | High |
| Informs | PROJECT_PLAN.md §6, §7 (spike S1) |
| Status | Not started |

**Why TCE needs this:** Spike S1 must decide how Unreal renders TCE's procedurally generated terrain and applies small earthwork edits at runtime.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Runtime-generated terrain in packaged UE5 builds

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Spike S1 must decide how Unreal renders TCE's procedurally generated terrain and applies small earthwork edits at runtime.

Research question: What is the best way to render a procedurally generated 16×16 km terrain in a packaged Unreal Engine 5.8 build, with small local edits streamed in at runtime?

Cover:
- Landscape runtime APIs (what works outside the editor), dynamic mesh terrain, Virtual Heightfield Mesh, Mesh Terrain, third-party options (e.g. Voxel Plugin)
- LOD, collision, runtime virtual texture materials, foliage and PCG compatibility
- Applying small local edits without hitches
- Performance on RTX 4070 Ti-class GPUs; shipped examples

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `01-ue-runtime-terrain.report.md`, then change **Status** above to Done.
