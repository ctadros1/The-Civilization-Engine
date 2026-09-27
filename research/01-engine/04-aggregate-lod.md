# Aggregate cohort models and promoting/demoting agents

| Field | Value |
|---|---|
| ID | 01-04 |
| Needed by | M9 (Aggregate LOD & scale) |
| Priority | Medium |
| Informs | PROJECT_PLAN.md §4.5 |
| Status | Not started |

**Why TCE needs this:** Scaling past ~100k people, and any future time-skip feature, needs settlements that can run as calibrated aggregates and switch back to individuals.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Aggregate cohort models and promoting/demoting agents

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Scaling past ~100k people, and any future time-skip feature, needs settlements that can run as calibrated aggregates and switch back to individuals.

Research question: How can a settlement switch between individually simulated people and an aggregate cohort model calibrated from the agent simulation, and back again?

Cover:
- Metamodels, surrogates and emulators of agent-based models; equation-free and heterogeneous multiscale methods
- Cohort-component and compartment representations of populations with economic and ideological attributes
- Population synthesis: generating plausible individuals from distributions (IPF, Bayesian networks)
- Keeping named characters consistent across modes; games that do this (Dwarf Fortress off-site history, X4 out-of-sector simulation, Mount & Blade)
- Using the same machinery for multi-decade time skips

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `04-aggregate-lod.report.md`, then change **Status** above to Done.
