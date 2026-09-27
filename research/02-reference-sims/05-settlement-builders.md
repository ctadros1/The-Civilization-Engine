# Settlement builders with individual citizens

| Field | Value |
|---|---|
| ID | 02-05 |
| Needed by | M1 (A band settles) |
| Priority | Medium |
| Informs | PROJECT_PLAN.md §5.1, §5.3 |
| Status | Not started |

**Why TCE needs this:** Banished, Songs of Syx, Manor Lords, Foundation, Ostriv and Farthest Frontier have solved organic growth, logistics and visible construction at various scales.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Settlement builders with individual citizens

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Banished, Songs of Syx, Manor Lords, Foundation, Ostriv and Farthest Frontier have solved organic growth, logistics and visible construction at various scales.

Research question: How do settlement builders with individual citizens simulate organic growth, production chains, hauling and construction, and what can TCE borrow?

Cover:
- Organic growth mechanics (Foundation, Manor Lords burgage plots), road formation
- Production chains, hauling logistics, construction progress visualization
- Scaling to thousands of citizens (Songs of Syx) and performance techniques
- Common failure modes: starvation spirals, balance issues

Deliver a design report:
1. How it works: the mechanics or patterns, in concrete detail.
2. What worked and what failed: developer postmortems, talks, patch notes and community analysis, with specifics.
3. Numbers: scale, performance or balance figures where available.
4. Lessons for TCE: what to adopt, adapt or avoid.
5. Sources: link talks, articles, papers, wikis and developer statements.
```

## Report

Save the finished report next to this file as `05-settlement-builders.report.md`, then change **Status** above to Done.
