# Multi-rate simulation and time acceleration

| Field | Value |
|---|---|
| ID | 01-03 |
| Needed by | M3 (Village economy) |
| Priority | High |
| Informs | PROJECT_PLAN.md §4.4 |
| Status | Not started |

**Why TCE needs this:** TCE switches between full sub-daily detail at low speeds and one statistical step per day at high speeds; the two must not diverge.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Multi-rate simulation and time acceleration

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE switches between full sub-daily detail at low speeds and one statistical step per day at high speeds; the two must not diverge.

Research question: How can one simulation switch between detailed sub-daily resolution and statistical daily resolution without its outcomes drifting apart?

Cover:
- Multi-rate and multi-scale scheduling techniques in the simulation literature
- Resolving an agent's day statistically (sampling outcomes) vs simulating it; sources of bias and variance
- How games handle fast-forward: Paradox tick rates, Dwarf Fortress history generation, Cities: Skylines speeds, SimCity
- Interactions that only exist at fine resolution (disease contacts, traffic, crime opportunities) and how to preserve them statistically
- Methods to test consistency between modes

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `03-multi-rate-time.report.md`, then change **Status** above to Done.
