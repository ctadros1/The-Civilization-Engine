# Cities: Skylines 1 and 2 simulation internals

| Field | Value |
|---|---|
| ID | 02-02 |
| Needed by | M1 (A band settles) |
| Priority | High |
| Informs | PROJECT_PLAN.md §4, §5.4 |
| Status | Not started |

**Why TCE needs this:** Cities: Skylines is TCE's closest genre reference; its citizen, traffic and economy models and C:S2's launch problems are directly instructive.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Cities: Skylines 1 and 2 simulation internals

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Cities: Skylines is TCE's closest genre reference; its citizen, traffic and economy models and C:S2's launch problems are directly instructive.

Research question: How do Cities: Skylines 1 and 2 simulate citizens, traffic, economy and services, and what went wrong with Cities: Skylines 2?

Cover:
- Citizen model: lifecycle, households, jobs, schooling; traffic AI and despawning
- Service coverage models and their known issues
- Cities: Skylines 2's Unity DOTS architecture, performance problems, economy bugs, and fixes
- Modding-community findings on how the simulations actually work

Deliver a design report:
1. How it works: the mechanics or patterns, in concrete detail.
2. What worked and what failed: developer postmortems, talks, patch notes and community analysis, with specifics.
3. Numbers: scale, performance or balance figures where available.
4. Lessons for TCE: what to adopt, adapt or avoid.
5. Sources: link talks, articles, papers, wikis and developer statements.
```

## Report

Save the finished report next to this file as `02-cities-skylines.report.md`, then change **Status** above to Done.
