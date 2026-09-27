# Data-driven content systems and mod-style authoring

| Field | Value |
|---|---|
| ID | 01-10 |
| Needed by | M0 (Foundations) |
| Priority | Medium |
| Informs | PROJECT_PLAN.md §3.2 |
| Status | Not started |

**Why TCE needs this:** All TCE primitives (technologies, goods, policies, offices, style elements) live in validated text data that AI agents author.

## Prompt

Copy everything in the block below into ChatGPT Pro deep research.

```text
Deep research request: Data-driven content systems and mod-style authoring

Context: I'm building The Civilization Engine (TCE), an endless, realistic agent-based simulation. Autonomous people found settlements, grow them into cities, and develop their own governments, laws, economies, technologies and architecture from authored building blocks. Nothing about history is scripted, and every world turns out differently. A Rust simulation kernel simulates 10k–50k individual people with visible daily life over decades to centuries, starting from early agrarian technology with no fixed eras. Unreal Engine 5 renders it on a Windows PC. All TCE primitives (technologies, goods, policies, offices, style elements) live in validated text data that AI agents author.

Research question: How should authored simulation primitives be stored as text data so they are validated, cross-referenced, hot-reloadable and easy to author?

Cover:
- Paradox script, RimWorld XML Defs, Dwarf Fortress raws, Factorio prototypes: strengths and pain points
- Schema validation, cross-file references, content-pack versioning and fingerprints
- RON vs TOML vs YAML vs KDL with serde in Rust; hot reload patterns
- Content linting: e.g. every technology lists what it unlocks

Deliver an engineering report:
1. Options: the main techniques and how each works.
2. Trade-offs: performance, complexity and maturity, with benchmarks where available.
3. Precedents: shipped games, engines, open-source projects and papers that do this, and what they learned.
4. Recommendation: the best fit for TCE's constraints (Rust kernel loaded as a DLL inside Unreal Engine 5.8 on Windows; i9 13th gen, 64 GB RAM, RTX 4070 Ti with 12 GB VRAM; 60 fps at 1440p; a solo developer working with AI coding agents), including pitfalls.
5. Sources: link documentation, papers, talks and code, and note which versions the information applies to (as of 2026).
```

## Report

Save the finished report next to this file as `10-data-driven-content.report.md`, then change **Status** above to Done.
