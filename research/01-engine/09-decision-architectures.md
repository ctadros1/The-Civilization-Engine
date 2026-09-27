# Decision architectures for thousands of agents: utility AI, GOAP, HTN

| Field | Value |
|---|---|
| ID | 01-09 |
| Needed by | M1 (A band settles) |
| Priority | High |
| Informs | PROJECT_PLAN.md §4.1, §4.2 |
| Status | Not started |

**Why TCE needs this:** TCE uses cheap utility decisions for every citizen and a deliberative planner for a few hundred notables; both must be fast and explainable.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Decision architectures for thousands of agents: utility AI, GOAP, HTN

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE uses cheap utility decisions for every citizen and a deliberative planner for a few hundred notables; both must be fast and explainable.

Research question: Which decision architectures scale to thousands of agents with plausible, explainable behavior, and how should a two-tier design (cheap citizens, deliberating notables) be built?

Cover:
- Utility AI (e.g. infinite-axis utility), GOAP, HTN, behavior trees: runtime cost and authoring cost
- Avoiding dithering: commitment, inertia, hysteresis; calibrating stochastic (softmax/logit) choice
- Generating human-readable explanations of choices
- Precedents: The Sims, RimWorld, Dwarf Fortress, F.E.A.R., Killzone, Crusader Kings 3 AI
- Planning over institutional moves (propose a law, form a faction) with estimated coalition support

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `09-decision-architectures.report.md`, then change **Status** above to Done.
