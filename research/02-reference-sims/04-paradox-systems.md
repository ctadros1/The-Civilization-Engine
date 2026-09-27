# Grand strategy models: Victoria 3, Crusader Kings 3, Europa Universalis

| Field | Value |
|---|---|
| ID | 02-04 |
| Needed by | M3 (Village economy) |
| Priority | High |
| Informs | PROJECT_PLAN.md §5.2, §5.3 |
| Status | Not started |

**Why TCE needs this:** Victoria 3's pops, markets, interest groups and law enactment are the nearest shipped analogue to TCE's economy and politics.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Grand strategy models: Victoria 3, Crusader Kings 3, Europa Universalis

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Victoria 3's pops, markets, interest groups and law enactment are the nearest shipped analogue to TCE's economy and politics.

Research question: How do Victoria 3, Crusader Kings 3 and Europa Universalis model economies, politics, laws and characters, and which balance problems did they hit?

Cover:
- Victoria 3: pop needs, market clearing, interest groups, law enactment mechanics, known economy problems and patches
- Crusader Kings 3: succession laws, factions, schemes, character AI
- How laws and institutions are represented (enumerated options vs composable parts)
- Lessons for composable institutions enacted by agents

Deliver a design report:
1. How it works: the mechanics or patterns, in concrete detail.
2. What worked and what failed: developer postmortems, talks, patch notes and community analysis, with specifics.
3. Numbers: scale, performance or balance figures where available.
4. Lessons for TCE: what to adopt, adapt or avoid.
5. Sources: link talks, articles, papers, wikis and developer statements.
```

## Report

Save the finished report next to this file as `04-paradox-systems.report.md`, then change **Status** above to Done.
