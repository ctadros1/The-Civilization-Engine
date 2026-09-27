# Day/night, seasons and weather rendering in UE5

| Field | Value |
|---|---|
| ID | 14-09 |
| Needed by | M2 (First light in Unreal) |
| Priority | Medium |
| Informs | PROJECT_PLAN.md §6 |
| Status | Not started |

**Why TCE needs this:** TCE shows day and night from M2, seasons from M3 and weather from M6.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Day/night, seasons and weather rendering in UE5

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE shows day and night from M2, seasons from M3 and weather from M6.

Research question: How should day/night, seasonal change and weather be rendered in UE5 for a large, dynamic world?

Cover:
- Sky Atmosphere, volumetric clouds, dynamic time of day with Lumen
- Seasonal foliage, snow accumulation and wetness materials
- Rain and storm effects (Niagara) and their performance
- Existing systems and plugins (e.g. Ultra Dynamic Sky)

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `09-environment-rendering.report.md`, then change **Status** above to Done.
