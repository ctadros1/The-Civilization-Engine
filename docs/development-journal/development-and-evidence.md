# Development and Evidence Practice

This describes the integrated `main` tree as of 2026-10-10. TCE is built as an interactive product, not as a preregistered academic study. Checks catch software defects, preserve accounting and save invariants, and reveal whether selected plausible conditions can occur. A passing test does not prove that simulated history is realistic.

## Working in vertical slices

Each milestone aims to leave a usable build: launch, create a world, observe it, save and load, then quit without data loss. A feature typically crosses the smallest necessary layers:

1. Read the project plan and research for the behavior and scope.
2. Assign the rule to its owning domain crate. Use an ADR for choices expensive to reverse; record reversible choices in the plan's decision log.
3. Model authoritative state, scheduled work, invariants, persistence and migration.
4. Expose kernel-authored facts and commands through host boundaries; keep the observer a renderer and input surface.
5. Check local rules, integration boundaries and longer-run interactions at more than one scale.
6. Record the exact build, inputs, commands, outcomes, failures and open limits in the plan and journal.

## Verification layers

The exact commands and gates in [AGENTS.md](../../AGENTS.md) and CI are authoritative. The main layers include:

| Layer | What it checks |
| --- | --- |
| Rust unit and property tests | Local rules: scheduler ordering, field and building calculations, IDs, ledgers, content validation and serialization |
| Workspace format, lint and tests | Rust/commons integration and compiler/clippy constraints |
| Save round trips | Supported saves reload and preserve authoritative section digests |
| Schema freshness | Generated Rust, TypeScript and C++ readers match FlatBuffers source schemas |
| Web tests | Message envelopes, payload interpretation, map logic and panels |
| Browser tests | Host startup, world creation, controls, panels, interactions and save/recovery flows |
| Smoke worlds | Invariants and chosen behavior thresholds over selected presets, seeds and years |
| Gate A | Exact continuation across advance boundaries, speed changes and save/load when approximations are off ([ADR-0011](../../decisions/0011-execution-modes.md) §5) |
| Gate B | Statistical comparison of Detailed and Accelerated fixture runs against fixed tolerances; current fixtures use one settlement and do not grade cross-settlement moves |
| Fifty-year dashboard | Five river-valley worlds over fifty years, grading applicable population, economy, inequality and building-failure rows; unsupported measures are reported unavailable, not silently passed |
| Population accounts | For each settlement and year, births − deaths + arrivals − departures equals resident change; the smoke and dashboard check the accounts |

Software properties and behavioral observations are different evidence. Exact conservation in a ledger transfer is an invariant. A migration, political change or famine observed in one seed is an outcome of that configuration and run. Neither alone supports a general historical claim.

## Current evidence and known limits

- M3c implements daily weather, field water, nitrogen pools, harvest history, household field planning and Accelerated-mode approximations. Its dashboard run passed four rows and left goods inequality amber; see the per-run record in [`PROJECT_PLAN.md` §9](../../PROJECT_PLAN.md#9-decisions-log).
- M4's institutions, order and political-change systems are implemented. In the latest recorded fifty-year dashboard after M4c, one world failed the food-price row. This does not imply the other rows or every run will behave the same way; the plan records counts and context.
- M5a implements multiple settlements, local contact, visits, marriage, movement, migration waves and coalition founding. The original demo's movement rate is not a current target: after river-routing corrections, its thirty-year rerun recorded 63 household moves, including 48 from a temporary settlement founded by the migration wave, and no coalition founding. Annual resident accounts balanced. The measured year for 3,000 people in three settlements was 1,179 seconds (19.6 minutes) at Max, above the ten-minute design budget.
- Gate B's current fixtures contain one settlement. It does not yet grade movement between settlements, so M5a's move outcomes are not covered by that consistency gate.
- M5b implements household price reports, cross-settlement purchases, fetching goods for resale, price-convergence records and contact-based technique/style diffusion. An earlier thirty-year comparison recorded 20 and 70 purchases, with no pair reaching the row's 30-purchase threshold and price gaps within the measured carrying-cost band. The corrected-routing rerun found that the two original roof-style demo villages could not meet; in the seed-9 run, trade instead connected Willowford with settlements its people founded. The earlier style result does not establish diffusion between the original neighboring villages under current routing.
- M5c implements claims, agreements ratified through each polity's own law, carried payments, and household and village log crossings. No agreement reached a gathering in the recorded thirty-year lived demo; a test world demonstrates a failed ratification. No household's own wades justified a standalone log crossing in the measured worlds, though selected village runs built shared crossings.
- The ten-year and fifty-year runs use selected worlds. Their passing checks do not show that every plausible world survives, every parameter is calibrated, or a model reproduces real history.

Older findings and the exact changes that produced them are in the chronological [development history](development-history.md); milestone-level values and run details are in the [project plan](../../PROJECT_PLAN.md). Do not repeat a metric without its seed, preset, commit, run count and command.

## Scope boundaries

- Modern industry, contemporary cities, Unreal rendering and modern public services are future scope.
- M5 is implemented, but the recorded demos do not establish that trade, diffusion, agreements or bridges will emerge in every world. Per-polity currency, merchant firms, war, enclosure and broader bridge systems remain future work.
- The current economic and political vocabularies are intentionally narrow. They do not model modern firms, unlimited credit, general-purpose currency minting, elections, state bureaucracies or war.
- Terrain, ecology and farming are stylized. Emmer is the main crop, livestock and broader crop cycles are absent, and many stock rates are tuning values.
- Simulation replay determinism is not a goal. World generation is reproducible from the seed and inputs on one build; snapshots preserve state and save continuation is checked separately.

## Continuing the journal

Add an entry for a meaningful landed slice or design correction, not a daily transcript:

```markdown
## YYYY-MM-DD — Milestone / slice: short subject

**Starting point:** branch/commit and current milestone state.
**Goal:** behavior or defect being addressed.
**Implementation:** authoritative state, systems and boundaries; link source files.
**Decision:** selected approach and alternatives; link an ADR or plan log.
**Evidence:** exact checks and observed results; distinguish pass/fail from interpretation.
**Open:** limitations, tuning values and follow-up work.
```

Keep metrics tied to their run conditions. Preserve failures, null outcomes and time-boxed nudges; never promote a test-only fixture to a claim about ordinary worlds. The current M5c failed-ratification example is a test world, not a lived-world result.
