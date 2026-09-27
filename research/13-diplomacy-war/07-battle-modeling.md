# Modeling battles and casualties

| Field | Value |
|---|---|
| ID | 13-07 |
| Needed by | M6 (Towns & their troubles) |
| Priority | High |
| Informs | PROJECT_PLAN.md §5.5 |
| Status | Not started |

**Why TCE needs this:** TCE resolves battles statistically but with visible, realistic consequences.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Modeling battles and casualties

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE resolves battles statistically but with visible, realistic consequences.

Research question: How can battle outcomes and casualties be modeled statistically with realistic results?

Cover:
- Lanchester laws and empirical tests; Dupuy's QJM and other quantified models
- Casualty rates for winners and losers; pursuit; morale collapse
- Force multipliers: terrain, fortifications, technology, leadership, surprise
- Wargame and game implementations

Deliver a simulation-ready report:
1. Mechanisms: the causal drivers and processes, stated as rules a simulation could implement.
2. Parameters: quantitative values and ranges in tables, with units, sources and confidence.
3. Variation: how this differed across eras (foragers, early farming, pre-industrial, industrial, modern) and across world regions, not only Europe.
4. Stylized facts: real-world patterns a correct simulation should reproduce, with numbers where possible.
5. Modeling recommendation: how to represent this with individual agents and institutions, what to simplify, and existing models or games that already do it.
6. Sources: cite scholarly sources and datasets; flag contested claims and thin evidence.
```

## Report

Save the finished report next to this file as `07-battle-modeling.report.md`, then change **Status** above to Done.
