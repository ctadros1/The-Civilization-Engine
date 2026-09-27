# Pathfinding and routing for tens of thousands of commuters

| Field | Value |
|---|---|
| ID | 01-08 |
| Needed by | M1 → M8 (A band settles) |
| Priority | High |
| Informs | PROJECT_PLAN.md §5.4 (transport) |
| Status | Not started |

**Why TCE needs this:** Every TCE citizen plans daily trips on a road network that grows and congests; routing is likely the kernel's largest CPU cost.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Pathfinding and routing for tens of thousands of commuters

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Every TCE citizen plans daily trips on a road network that grows and congests; routing is likely the kernel's largest CPU cost.

Research question: Which routing techniques let 50k agents plan daily trips on an evolving, congestible road network cheaply?

Cover:
- Contraction hierarchies, customizable route planning, hub labels, A* variants; update costs when roads change
- Time-dependent and congestion-aware routing; route caching by origin/destination zone
- Hierarchical pathfinding in games (HPA*, flow fields); what Cities: Skylines 1/2 do and their known problems
- Multi-modal routing (walking, carts, boats; later transit)
- Rust crates and benchmark numbers

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `08-routing-at-scale.report.md`, then change **Status** above to Done.
