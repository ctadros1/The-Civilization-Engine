# Traffic flow and assignment models for simulations

| Field | Value |
|---|---|
| ID | 12-06 |
| Needed by | M8 (The growing city (v1)) |
| Priority | High |
| Informs | PROJECT_PLAN.md §5.4 |
| Status | Not started |

**Why TCE needs this:** TCE needs realistic congestion for tens of thousands of agents in real time, plus an aggregate mode for fast-forward.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Traffic flow and assignment models for simulations

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE needs realistic congestion for tens of thousands of agents in real time, plus an aggregate mode for fast-forward.

Research question: Which traffic models give realistic congestion for 50k agents in real time, with an aggregate mode for fast-forward?

Cover:
- Microscopic, mesoscopic and macroscopic models; BPR volume-delay functions; dynamic traffic assignment
- Queue-based models (e.g. MATSim's queue simulation)
- How SimCity, Cities: Skylines 1 and 2 and Transport Fever model traffic, and their problems
- Performance numbers

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `06-traffic-modeling.report.md`, then change **Status** above to Done.
