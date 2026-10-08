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
| Accelerated Gate A | Exact comparisons across advance partitions, speed-mode changes, and save/load continuations |
| Accelerated Gate B | Nightly statistical comparisons of declared approximations against repeated Detailed runs; tolerances and fixtures are logged in the plan |
| M3c dashboard | Five river-valley worlds × fifty years; applicable population, price, goods inequality, firm-size and lived-in building-failure rows are graded, with inapplicable measures shown as unavailable |
| Weather probes | Long generated series check annual totals, wet-day counts, temperature, snow and water-driven harvest variation; weather remains shared by the landscape, not sampled independently by each field |

The important distinction is between **software properties** and **behavioral observations**. “Goods balance exactly across a ledger transfer” is an invariant. “This seed empties its valley after a bad harvest” is an observation about one configuration and run. Neither should be presented as proof of a general historical claim.

## What the development record has shown

- M0's world smoke tests check terrain/water characteristics and save integrity on selected preset/seed combinations.
- M1 added population checks and longer runs because a terrain-only test cannot reveal whether people can reach food, water and shelter.
- M3a checks the conservation of goods as well as annual economy signals. The record showed that the two land regimes can look similar while land is plentiful, and that large founding villages can experience a severe harvest shortfall and leave. This motivated a logged M3c farming nudge.
- M3b's knowledge and structural-building work extended smoke checks to technique continuity, building condition, loads, collapse outcomes and economy. A smoke run caught implausibly frequent failures after slice P's load model; correction of when monthly storms applied and hut member sizes brought the recorded ten-year run into its expected check band. Later, caution based on village building failures was implemented and its recorded ten-year smoke showed no load failures in those selected worlds.
- M3c T recorded the dashboard before tuning: population and lived-in structural failures were outside their bands. This baseline is preserved so later changes can be compared against the original result.
- M3c U's final ten-year smoke passed its selected checks in ten worlds after narrowing weather restrictions to soil-turning work and fixing a way for households to replace a worn axe. Five worlds met the population floor; the other five were extinct or below it. The run therefore passes its configured checks without establishing broad settlement resilience.
- Weather probes over long histories check the generated climate and crop-water distribution. A sound distribution does not establish realistic farming outcomes or prove the selected parameters are calibrated.

These results describe the integrated `main` source baseline at commit [`33dbd4a`](https://github.com/ctadros1/The-Civilization-Engine/commit/33dbd4a), not calibration guarantees. See the chronological entries in [`development-history.md`](development-history.md) and the detailed per-slice evidence in [`PROJECT_PLAN.md` §9](../../PROJECT_PLAN.md#9-decisions-log).

## Known model limits at this baseline

- The world simulates a small founding society and early farming, not the project's full span through modern civilization.
- A single settlement has no connected regional destination for departures. A household that leaves is removed from the simulated world.
- Farming centers on emmer and a spring crop cycle. Weather and field water are integrated, but nitrogen-based fertility, deliberate rotations/resting, a broader crop set and livestock are not.
- Some early-world stock estimates and rates are tuning values. Deposits and pits/quarries are integrated, but geological placement rules remain authored approximations.
- Markets, workshops, property regimes and building programs cover a deliberately narrow set of early institutions.
- Unreal is not the current viewer. The kernel C ABI is ready as an integration surface, but the UE client and its rendering pipeline remain to be built.
- Simulation replay is not deterministic. World generation is reproducible per build/input; each world's weather stream is keyed by seed, landscape and day.

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
