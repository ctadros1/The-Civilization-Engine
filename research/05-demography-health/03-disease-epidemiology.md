# Epidemiology of historical diseases

| Field | Value |
|---|---|
| ID | 05-03 |
| Needed by | M6 (Towns & their troubles) |
| Priority | High |
| Informs | PROJECT_PLAN.md §5.4 (disease) |
| Status | Not started |

**Why TCE needs this:** TCE simulates waterborne, airborne and contact diseases person by person, traced to real sources.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Epidemiology of historical diseases

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE simulates waterborne, airborne and contact diseases person by person, traced to real sources.

Research question: What are the transmission characteristics and impacts of the diseases that shaped pre-modern and industrial settlements?

Cover:
- Waterborne (cholera, typhoid, dysentery), airborne (smallpox, measles, influenza, tuberculosis), vector-borne (plague, malaria) and contact diseases
- Parameters: R0, latent and infectious periods, case fatality by age and nutrition, immunity duration
- Endemic vs epidemic dynamics; population thresholds for endemic persistence
- Agent-based epidemic modeling practice

Deliver a simulation-ready report:
1. Mechanisms: the causal drivers and processes, stated as rules a simulation could implement.
2. Parameters: quantitative values and ranges in tables, with units, sources and confidence.
3. Variation: how this differed across eras (foragers, early farming, pre-industrial, industrial, modern) and across world regions, not only Europe.
4. Stylized facts: real-world patterns a correct simulation should reproduce, with numbers where possible.
5. Modeling recommendation: how to represent this with individual agents and institutions, what to simplify, and existing models or games that already do it.
6. Sources: cite scholarly sources and datasets; flag contested claims and thin evidence.
```

## Report

Save the finished report next to this file as `03-disease-epidemiology.report.md`, then change **Status** above to Done.
