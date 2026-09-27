# Streaming simulation state to a renderer

| Field | Value |
|---|---|
| ID | 01-07 |
| Needed by | M2 (First light in Unreal) |
| Priority | High |
| Informs | PROJECT_PLAN.md §3.1 |
| Status | Not started |

**Why TCE needs this:** Unreal must show tens of thousands of moving people and a changing city from kernel frames without stalling the game thread.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Streaming simulation state to a renderer

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. Unreal must show tens of thousands of moving people and a changing city from kernel frames without stalling the game thread.

Research question: How should 10k–50k moving agents and a continually changing city be streamed from a simulation to a renderer efficiently?

Cover:
- Snapshot + delta encoding, interest management, quantization
- Event-based movement (trip start/end on known paths) vs position streaming; interpolation and extrapolation
- Frame pacing between a simulation running at variable speed and a 60 fps renderer
- Analogous solutions in networked games and replay systems (Unreal replication/Iris, Overwatch, Rocket League, Factorio)
- CPU and memory budgets with numbers

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `07-state-streaming.report.md`, then change **Status** above to Done.
