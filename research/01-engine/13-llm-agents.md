# LLM-driven agents in large simulations

| Field | Value |
|---|---|
| ID | 01-13 |
| Needed by | M12 (LLM notables, multi-viewer, close-up fidelity) |
| Priority | Low |
| Informs | PROJECT_PLAN.md §4.2 |
| Status | Not started |

**Why TCE needs this:** TCE may later let an LLM drive notables' deliberation through a pluggable interface.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: LLM-driven agents in large simulations

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE may later let an LLM drive notables' deliberation through a pluggable interface.

Research question: What has been learned from LLM-driven agent societies, and how could a hybrid rule-based/LLM deliberation layer work for a few hundred notable agents?

Cover:
- Generative Agents (Park et al.), Project Sid, AgentSociety and 2024–2026 work: architectures and results
- Cost and latency at scale; local models on a 12 GB GPU shared with rendering
- Constraining outputs to typed, valid actions; caching and batching requests
- Failure modes: invalid actions, homogenized personalities, drift

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `13-llm-agents.report.md`, then change **Status** above to Done.
