# Hydrology for settlement simulation: rivers, groundwater and floods

| Field | Value |
|---|---|
| ID | 03-02 |
| Needed by | M1 → M6 (A band settles) |
| Priority | High |
| Informs | PROJECT_PLAN.md §5.4 (water) |
| Status | Not started |

**Why TCE needs this:** Wells, irrigation, floods and contaminated water sources in TCE all depend on a plausible water model.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Hydrology for settlement simulation: rivers, groundwater and floods

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Wells, irrigation, floods and contaminated water sources in TCE all depend on a plausible water model.

Research question: How should rivers, groundwater and flooding be modeled so that wells, irrigation, floods and water contamination behave realistically?

Cover:
- River discharge and seasonality from climate; floodplain extent and flood return periods
- Groundwater depth and well yields by terrain and geology; historical well depths
- Contaminant transport from latrines and cesspits to wells and rivers (distances, soil types)
- Simplified models suitable for real-time simulation

Deliver a simulation-ready report:
1. Mechanisms: the causal drivers and processes, stated as rules a simulation could implement.
2. Parameters: quantitative values and ranges in tables, with units, sources and confidence.
3. Variation: how this differed across eras (foragers, early farming, pre-industrial, industrial, modern) and across world regions, not only Europe.
4. Stylized facts: real-world patterns a correct simulation should reproduce, with numbers where possible.
5. Modeling recommendation: how to represent this with individual agents and institutions, what to simplify, and existing models or games that already do it.
6. Sources: cite scholarly sources and datasets; flag contested claims and thin evidence.
```

## Report

Save the finished report next to this file as `02-hydrology.report.md`, then change **Status** above to Done.
