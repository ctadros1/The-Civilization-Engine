# Climate zones and stochastic weather generation

| Field | Value |
|---|---|
| ID | 03-03 |
| Needed by | M3 (Village economy) |
| Priority | High |
| Informs | PROJECT_PLAN.md §5.4, §6 |
| Status | Not started |

**Why TCE needs this:** Harvests, disease, fire spread and floods in TCE are driven by each world's climate and day-to-day weather.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Climate zones and stochastic weather generation

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Harvests, disease, fire spread and floods in TCE are driven by each world's climate and day-to-day weather.

Research question: How can a world's climate and daily weather be generated plausibly from its geography, including droughts, storms and year-to-year variability?

Cover:
- Deriving climate zones (Köppen) from latitude, elevation, oceans, prevailing winds and rain shadows
- Stochastic weather generators (e.g. Richardson/WGEN): daily precipitation, temperature, persistence
- Frequency and magnitude of droughts, storms and cold snaps; multi-year variability
- Parameter sets by climate type

Deliver a simulation-ready report:
1. Mechanisms: the causal drivers and processes, stated as rules a simulation could implement.
2. Parameters: quantitative values and ranges in tables, with units, sources and confidence.
3. Variation: how this differed across eras (foragers, early farming, pre-industrial, industrial, modern) and across world regions, not only Europe.
4. Stylized facts: real-world patterns a correct simulation should reproduce, with numbers where possible.
5. Modeling recommendation: how to represent this with individual agents and institutions, what to simplify, and existing models or games that already do it.
6. Sources: cite scholarly sources and datasets; flag contested claims and thin evidence.
```

## Report

Save the finished report next to this file as `03-climate-weather.report.md`, then change **Status** above to Done.
