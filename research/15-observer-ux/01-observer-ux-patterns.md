# Observer UX for complex simulations

| Field | Value |
|---|---|
| ID | 15-01 |
| Needed by | M1 (A band settles) |
| Priority | High |
| Informs | PROJECT_PLAN.md §2 |
| Status | Not started |

**Why TCE needs this:** TCE's player mostly watches and inspects, so legibility is the product.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Observer UX for complex simulations

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE's player mostly watches and inspects, so legibility is the product.

Research question: What makes complex simulations legible and enjoyable to observe, and which UI patterns work best?

Cover:
- Info views and overlays, inspectors, follow cameras, timelines, comparison dashboards
- Examples: Cities: Skylines info views, Dwarf Fortress Legends, RimWorld, Victoria 3, Crusader Kings 3, Songs of Syx
- Managing information overload; progressive disclosure
- Time controls and notifications

Deliver a design report:
1. How it works: the mechanics or patterns, in concrete detail.
2. What worked and what failed: developer postmortems, talks, patch notes and community analysis, with specifics.
3. Numbers: scale, performance or balance figures where available.
4. Lessons for TCE: what to adopt, adapt or avoid.
5. Sources: link talks, articles, papers, wikis and developer statements.
```

## Report

Save the finished report next to this file as `01-observer-ux-patterns.report.md`, then change **Status** above to Done.
