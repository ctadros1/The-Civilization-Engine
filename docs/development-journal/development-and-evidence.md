# Development and Evidence Practice

TCE is built as an interactive product, not as a preregistered academic study. Checks are meant to catch software defects, preserve accounting and save invariants, and reveal whether a small set of plausible conditions can be reached. A passing test does not prove that simulated history is realistic.

## Working in vertical slices

Each milestone is intended to leave a usable build: launch, create a world, observe it, save/load, and quit without data loss. Work is split into slices that cross the smallest necessary layers rather than leaving long-lived disconnected subsystems.

A typical feature path is:

1. **Define the behavior.** Read the project plan and the relevant research. State what the current build should do and what remains out of scope.
2. **Choose ownership.** Put the rule in its domain crate. Add an ADR only when a choice is expensive to reverse; otherwise record it in §9 of the plan.
3. **Model and persist it.** Add the authoritative state, scheduled work and invariants. Decide what must be saved, what can be derived, and how old saves load.
4. **Expose it.** Add kernel-owned payloads, commands or queries. Keep the viewer as a renderer and input surface.
5. **Check it at more than one scale.** Unit tests cover local rules; integration/browser tests cover boundaries; smoke worlds expose interactions over time.
6. **Record what happened.** Update the README and plan. Preserve notable observed behavior, failures, test coverage and open limits in the journal.

## Verification layers

The repository's operating instructions in [AGENTS.md](../../AGENTS.md) are authoritative for commands and pre-commit gates. In brief, the layers are:

| Layer | What it protects |
| --- | --- |
| Rust unit/property tests | Local rules: scheduler ordering, field and building calculations, IDs, ledgers, content validation and serialization |
| Workspace format, lint and test commands | Rust/commons integration and compiler/clippy constraints |
| Save round-trip checks | Supported saves reload and preserve authoritative section digests |
| Schema generation check | Checked-in Rust/TypeScript/C++ generated readers match FlatBuffers source schemas |
| Web unit tests | Envelope, payload interpretation, shading, calendar, display helpers and panels |
| Playwright browser tests | Host startup, world creation, controls, panels, interactions and save/recovery flows |
| Smoke worlds | Behavior across selected seeds and presets, checking invariants and thresholds over time |
| M3a economy grading | Annual checks for stock changes, asks, inequality and workshop distributions, with gray/amber bands where evidence is too sparse or incomplete |
| Ten-year runs | Longer interactions among food, demography, knowledge, farming and building systems; currently nightly/PR CI also runs these by repository configuration |

The important distinction is between **software properties** and **behavioral observations**. “Goods balance exactly across a ledger transfer” is an invariant. “This seed empties its valley after a bad harvest” is an observation about one configuration and run. Neither should be presented as proof of a general historical claim.

## What the development record has shown

- M0's world smoke tests check terrain/water characteristics and save integrity on selected preset/seed combinations.
- M1 added population checks and longer runs because a terrain-only test cannot reveal whether people can reach food, water and shelter.
- M3a checks the conservation of goods as well as annual economy signals. The record showed that the two land regimes can look similar while land is plentiful, and that large founding villages can experience a severe harvest shortfall and leave. This motivated a logged M3c farming nudge.
- M3b's knowledge and structural-building work extended smoke checks to technique continuity, building condition, loads, collapse outcomes and economy. A smoke run caught implausibly frequent failures after slice P's load model; correction of when monthly storms applied and hut member sizes brought the recorded ten-year run into its expected check band. Later, caution based on village building failures was implemented and its recorded ten-year smoke showed no load failures in those selected worlds.

These are source-baseline results, not calibration guarantees. See the chronological entries in [`development-history.md`](development-history.md) and the detailed per-slice evidence in [`PROJECT_PLAN.md` §9](../../PROJECT_PLAN.md#9-decisions-log).

## Known model limits at this baseline

- The world simulates a small founding society and early farming, not the project's full span through modern civilization.
- A single settlement has no connected regional destination for departures. A household that leaves is removed from the simulated world.
- Farming centers on emmer and a spring crop cycle. Soil nutrients, a broader crop set, livestock and M3c's fuller weather/soil behavior are not implemented.
- Some early-world stock estimates and rates are tuning values. The geological deposit-placement primitive is not yet connected to normal world generation or extraction.
- Markets, workshops, property regimes and building programs cover a deliberately narrow set of early institutions.
- Unreal is not the current viewer. The kernel C ABI is ready as an integration surface, but the UE client and its rendering pipeline remain to be built.
- Simulation determinism is explicitly not a goal. Results can differ between runs; world generation is reproducible per build/input.

## Continuing the journal

Add one entry for a meaningful landed slice or design correction, rather than a daily transcript. Use this outline:

```markdown
## YYYY-MM-DD — Milestone / slice: short subject

**Starting point:** branch/commit and current milestone state.

**Goal:** the behavior or defect being addressed.

**Implementation:** authoritative state, system and boundary changes; link source files.

**Decision:** the selected approach and alternatives that mattered; link an ADR or plan log.

**Evidence:** exact tests, smoke seeds, thresholds and observed outcomes; distinguish pass/fail from interpretation.

**Open:** limitations, tuning values and follow-up work.
```

Keep exact metrics with their seed, preset, build/commit and run count. Do not carry a previous run's counts forward without rerunning it.
