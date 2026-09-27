# Explaining simulated decisions: the 'why' behind every outcome

| Field | Value |
|---|---|
| ID | 15-03 |
| Needed by | M1 (A band settles) |
| Priority | High |
| Informs | PROJECT_PLAN.md §2 |
| Status | Not started |

**Why TCE needs this:** Every TCE entity must answer 'why': why a law passed, why a person stole, why a house is stone.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Explaining simulated decisions: the 'why' behind every outcome

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Every TCE entity must answer 'why': why a law passed, why a person stole, why a house is stone.

Research question: How can a simulation record the provenance of decisions and outcomes so that any result can be explained to the observer?

Cover:
- Attribution in utility-based decisions; contribution breakdowns
- Provenance tracking under bounded memory
- Explainable-AI techniques applicable to game AI
- UI examples (e.g. breakdown tooltips in Paradox games)

Deliver a design report:
1. How it works: the mechanics or patterns, in concrete detail.
2. What worked and what failed: developer postmortems, talks, patch notes and community analysis, with specifics.
3. Numbers: scale, performance or balance figures where available.
4. Lessons for TCE: what to adopt, adapt or avoid.
5. Sources: link talks, articles, papers, wikis and developer statements.
```

## Report

Save the finished report next to this file as `03-decision-explanation.report.md`, then change **Status** above to Done.
