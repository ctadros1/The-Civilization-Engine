# Parallel simulation in Rust: throughput and correctness

| Field | Value |
|---|---|
| ID | 01-01 |
| Needed by | M0 (Foundations) |
| Priority | High |
| Informs | PROJECT_PLAN.md §3.2 |
| Status | Not started |

**Why TCE needs this:** The kernel must use every core of an i9 to simulate tens of thousands of people; exact reproducibility is not a goal, but data races and subtle update-order bugs are.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Parallel simulation in Rust: throughput and correctness

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. The kernel must use every core of an i9 to simulate tens of thousands of people; exact reproducibility is not a goal, but data races and subtle update-order bugs are.

Research question: How should a Rust simulation kernel parallelize agent updates for maximum throughput while avoiding data races, update-order artifacts and hard-to-debug behavior?

Cover:
- Parallel patterns for agent simulations: double-buffered state, spatial partitioning, job systems, rayon vs custom thread pools
- Update-order artifacts (agents seeing half-updated state) and how to avoid them
- Contention hot spots: shared markets, shared resources, spatial queries
- Debugging parallel simulation bugs without full determinism (snapshots before failures, tracing, sanitizers)
- Benchmarks: scaling from 1 to 24 threads on desktop CPUs

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `01-parallel-rust-sim.report.md`, then change **Status** above to Done.
