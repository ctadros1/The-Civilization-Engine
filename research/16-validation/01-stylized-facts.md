# Stylized facts for validating a society simulation

| Field | Value |
|---|---|
| ID | 16-01 |
| Needed by | M3 (Village economy) |
| Priority | High |
| Informs | PROJECT_PLAN.md §4.7 |
| Status | Not started |

**Why TCE needs this:** TCE's sanity dashboard checks worlds against real-world regularities instead of running research campaigns.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Stylized facts for validating a society simulation

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE's sanity dashboard checks worlds against real-world regularities instead of running research campaigns.

Research question: Which robust empirical regularities about societies, economies and cities should a realistic simulation reproduce, and with what numbers?

Cover:
- Zipf's law for cities, Pareto wealth tails, Gibrat's law, firm size distributions
- Crime and poverty, epidemic curves, price seasonality, fertility and income, Engel's law
- Ranges and sources for each
- How robust each regularity is across eras

Deliver a simulation-ready report:
1. Mechanisms: the causal drivers and processes, stated as rules a simulation could implement.
2. Parameters: quantitative values and ranges in tables, with units, sources and confidence.
3. Variation: how this differed across eras (foragers, early farming, pre-industrial, industrial, modern) and across world regions, not only Europe.
4. Stylized facts: real-world patterns a correct simulation should reproduce, with numbers where possible.
5. Modeling recommendation: how to represent this with individual agents and institutions, what to simplify, and existing models or games that already do it.
6. Sources: cite scholarly sources and datasets; flag contested claims and thin evidence.
```

## Report

Save the finished report next to this file as `01-stylized-facts.report.md`, then change **Status** above to Done.
