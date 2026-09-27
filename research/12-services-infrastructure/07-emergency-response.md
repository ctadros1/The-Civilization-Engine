# Emergency dispatch and service coverage models

| Field | Value |
|---|---|
| ID | 12-07 |
| Needed by | M6 (Towns & their troubles) |
| Priority | Medium |
| Informs | PROJECT_PLAN.md §5.4 |
| Status | Not started |

**Why TCE needs this:** TCE shows real incidents and response times; coverage must emerge from where providers are and how they travel.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Emergency dispatch and service coverage models

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE shows real incidents and response times; coverage must emerge from where providers are and how they travel.

Research question: How should service coverage and emergency response be modeled so that response times and coverage emerge from the simulation?

Cover:
- Location-set covering, maximal covering and p-median models
- Dispatch and response-time distributions; queueing
- Historical response capacities (watches, brigades)
- How games implement coverage

Deliver a simulation-ready report:
1. Mechanisms: the causal drivers and processes, stated as rules a simulation could implement.
2. Parameters: quantitative values and ranges in tables, with units, sources and confidence.
3. Variation: how this differed across eras (foragers, early farming, pre-industrial, industrial, modern) and across world regions, not only Europe.
4. Stylized facts: real-world patterns a correct simulation should reproduce, with numbers where possible.
5. Modeling recommendation: how to represent this with individual agents and institutions, what to simplify, and existing models or games that already do it.
6. Sources: cite scholarly sources and datasets; flag contested claims and thin evidence.
```

## Report

Save the finished report next to this file as `07-emergency-response.report.md`, then change **Status** above to Done.
