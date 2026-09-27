# Procedural terrain with realistic geomorphology

| Field | Value |
|---|---|
| ID | 03-01 |
| Needed by | M0 (Foundations) |
| Priority | High |
| Informs | PROJECT_PLAN.md §6 |
| Status | Not started |

**Why TCE needs this:** TCE generates each world's 16×16 km terrain (2 m heightfield) at world creation; settlements depend on plausible valleys, rivers and coasts.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Procedural terrain with realistic geomorphology

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE generates each world's 16×16 km terrain (2 m heightfield) at world creation; settlements depend on plausible valleys, rivers and coasts.

Research question: Which algorithms generate realistic 16×16 km terrain at 2 m resolution with plausible mountains, valleys, rivers, lakes and coasts, fast enough for world creation on a desktop?

Cover:
- Uplift/stream-power erosion (e.g. Cordonnier et al.), hydraulic and thermal erosion; GPU vs CPU
- River network and lake extraction, flow accumulation, depression filling
- Generation time benchmarks at 8192² resolution
- Open-source implementations (Rust or portable) and papers

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `01-terrain-generation.report.md`, then change **Status** above to Done.
