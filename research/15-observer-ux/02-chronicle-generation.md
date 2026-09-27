# Automated chronicles and emergent narrative from event logs

| Field | Value |
|---|---|
| ID | 15-02 |
| Needed by | M1 (A band settles) |
| Priority | High |
| Informs | PROJECT_PLAN.md §2 |
| Status | Not started |

**Why TCE needs this:** TCE writes an automatic, filterable history of each world from its event log.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Automated chronicles and emergent narrative from event logs

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE writes an automatic, filterable history of each world from its event log.

Research question: How can a simulation's event log be turned into a readable, interesting chronicle?

Cover:
- Salience scoring of events; story sifting (e.g. Kreminski et al.)
- Dwarf Fortress Legends, Crusader Kings chronicles, RimWorld tales; template-based generation
- Optional LLM narration from structured events
- UI for browsing history

Deliver a design report:
1. How it works: the mechanics or patterns, in concrete detail.
2. What worked and what failed: developer postmortems, talks, patch notes and community analysis, with specifics.
3. Numbers: scale, performance or balance figures where available.
4. Lessons for TCE: what to adopt, adapt or avoid.
5. Sources: link talks, articles, papers, wikis and developer statements.
```

## Report

Save the finished report next to this file as `02-chronicle-generation.report.md`, then change **Status** above to Done.
