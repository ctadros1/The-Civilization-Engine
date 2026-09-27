# Validating agent-based models without a research program

| Field | Value |
|---|---|
| ID | 16-03 |
| Needed by | M3 (Village economy) |
| Priority | Medium |
| Informs | PROJECT_PLAN.md §4.7 |
| Status | Not started |

**Why TCE needs this:** TCE needs cheap ways to confirm plausibility without turning into a research project.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Validating agent-based models without a research program

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. TCE needs cheap ways to confirm plausibility without turning into a research project.

Research question: What lightweight methods validate that an agent-based simulation behaves plausibly?

Cover:
- Pattern-oriented modeling (Grimm et al.), face validation, docking, sensitivity analysis
- Calibration methods, and which are cheap enough to use routinely
- Common validation failures
- Practical checklists

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `03-abm-validation.report.md`, then change **Status** above to Done.
