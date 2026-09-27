# Designing data-dense panels for a simulation observer

| Field | Value |
|---|---|
| ID | 15-04 |
| Needed by | M1 (A band settles) |
| Priority | Medium |
| Informs | PROJECT_PLAN.md §3.2 |
| Status | Not started |

**Why TCE needs this:** TCE's inspectors, charts and timelines are web panels that must stay fast and readable with live data.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Designing data-dense panels for a simulation observer

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE's inspectors, charts and timelines are web panels that must stay fast and readable with live data.

Research question: How should data-dense panels (tables, charts, timelines, network views) be designed for a live simulation observer, and which web libraries fit?

Cover:
- TypeScript charting libraries that handle large, live series; table virtualization
- Visual patterns for dense game UIs (Paradox games, EVE Online, Factorio)
- Accessibility and UI scaling
- Real-time update strategies

Deliver a design report:
1. How it works: the mechanics or patterns, in concrete detail.
2. What worked and what failed: developer postmortems, talks, patch notes and community analysis, with specifics.
3. Numbers: scale, performance or balance figures where available.
4. Lessons for TCE: what to adopt, adapt or avoid.
5. Sources: link talks, articles, papers, wikis and developer statements.
```

## Report

Save the finished report next to this file as `04-data-dense-ui.report.md`, then change **Status** above to Done.
