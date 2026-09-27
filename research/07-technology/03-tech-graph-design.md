# Designing a technology graph for a simulation

| Field | Value |
|---|---|
| ID | 07-03 |
| Needed by | M3 (Village economy) |
| Priority | High |
| Informs | PROJECT_PLAN.md §5.6 |
| Status | Not started |

**Why TCE needs this:** TCE's tech graph has 150–250 nodes with no era gates, and every node must carry concrete content.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Designing a technology graph for a simulation

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE's tech graph has 150–250 nodes with no era gates, and every node must carry concrete content.

Research question: How should a technology graph be structured so that discovery is emergent, societies diverge, and every node carries concrete content?

Cover:
- Node granularity and structure in Civilization, Stellaris, Victoria 3, Humankind, Old World and academic models
- Prerequisite logic (AND/OR), alternative paths, resource- and region-dependent technologies
- Linking nodes to content (goods, recipes, buildings, services)
- Avoiding a linear 'tech race' feel

Deliver a design report:
1. How it works: the mechanics or patterns, in concrete detail.
2. What worked and what failed: developer postmortems, talks, patch notes and community analysis, with specifics.
3. Numbers: scale, performance or balance figures where available.
4. Lessons for TCE: what to adopt, adapt or avoid.
5. Sources: link talks, articles, papers, wikis and developer statements.
```

## Report

Save the finished report next to this file as `03-tech-graph-design.report.md`, then change **Status** above to Done.
