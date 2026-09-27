# SimCity (2013) GlassBox: lessons from an agent-based city sim

| Field | Value |
|---|---|
| ID | 02-01 |
| Needed by | M1 (A band settles) |
| Priority | High |
| Informs | PROJECT_PLAN.md §4 |
| Status | Not started |

**Why TCE needs this:** GlassBox is the best-known failure of an agent-based city simulation; TCE must avoid its non-persistent, implausible citizens.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: SimCity (2013) GlassBox: lessons from an agent-based city sim

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. GlassBox is the best-known failure of an agent-based city simulation; TCE must avoid its non-persistent, implausible citizens.

Research question: How did SimCity (2013)'s GlassBox engine simulate its city, and why did its agents behave implausibly?

Cover:
- Resource/agent/unit model; city size and simulation scale limits
- Developer GDC talks and postmortems
- Specific failures (citizens without persistent homes/jobs, nearest-destination pathing) and their root causes
- What a simulation with persistent individuals must do differently

Deliver a design report:
1. How it works: the mechanics or patterns, in concrete detail.
2. What worked and what failed: developer postmortems, talks, patch notes and community analysis, with specifics.
3. Numbers: scale, performance or balance figures where available.
4. Lessons for TCE: what to adopt, adapt or avoid.
5. Sources: link talks, articles, papers, wikis and developer statements.
```

## Report

Save the finished report next to this file as `01-simcity-glassbox.report.md`, then change **Status** above to Done.
