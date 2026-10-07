# ADR-0011: Execution modes: Detailed and Accelerated

Status: Accepted
Date: 2026-10-05
Milestone: M3c
Amended: 2026-10-07, to match what was built. Slice W: §4's household view (built at a
household's first decision of the day) and leisure blocks (a drawn run of sessions, not a fixed
stretch), §5's Gate B as `civ-host consistency`

## Context

Plan §4.4 sets two speed modes. Detailed (pause, 1×, 3×, 10×) runs the full sub-daily
simulation. Accelerated (60×, 600×, Max) resolves each agent's day "statistically in one daily
step". Both call the same decision functions, and switches happen at day boundaries. The target
is at least one in-game year per real minute at 50k agents (M8), and a mode-consistency test
keeps the two together. M3c's demo lives fifty years in Accelerated mode and passes the §4.7
dashboard of five worlds × fifty years.

Reports read: 01-03 (multi-rate simulation and time acceleration), with 16-03 on validation.
Their main points:

- **Read "one statistical step per day" as one external request to advance to the next day,**
  not as a ban on events inside the day (01-03, executive recommendation).
- **Keep one authoritative model at every speed,** and preserve the state that makes the next
  day predictable: plans, active segments, trips, inventories, residual clocks (§4.1). Each
  optimised rule declares whether it is exact relative to the reference rule, or approximate
  within a declared domain (§4.1).
- **A day is an advance horizon, not one indivisible operation;** switch at day boundaries
  first (§4.2). Advance in bounded chunks so the host stays responsive (§4.4).
- **Two gates.** Gate A, exact equivalence: advance(a + b) ≡ advance(a); advance(b), across
  modes, partitions and save/load. Gate B, statistical equivalence: margins fixed before the
  results are seen (§4.5).
- **Key randomness by semantic identity,** never by frame, worker, speed setting or number of
  calls to advance (§4.5).

A probe measured the kernel on 2026-10-05 (a band of 40, one simulated year in about 14 s):

- `Population::decide` takes about 93 % of the time.
  - Most decisions are short leisure choices: rest 38 %, the hearth 20 %, play 15 %, meals 13 %.
  - Each decision rebuilds household and settlement facts: the household copied (34 % of the
    time), travel fields recomputed (27 %), and options scored (27 %).
- Walks, step effects, needs (already in closed form) and the day's work in `on_day` take
  7–9 %.
- The scheduler already delivers cadence boundaries before the events of the same instant.

## Decision

### 1. One authoritative state

- A mode is how the kernel advances, implied by the speed: 1×–10× is Detailed; 60×, 600× and
  Max are Accelerated. It is a setting, not world state.
- Both modes share activities, pending events, trips, stores and lazily settled needs.
- Nothing Accelerated-only is saved, and every save is a valid Detailed state.
  - Caches, such as a household view, are pure functions of saved state, built in a fixed
    order with no random draws, and are never saved.

### 2. A day is an advance horizon

- Accelerated mode advances whole days, midnight to midnight, and processes the same events
  inside each day.
- It stops, publishes, saves and switches only at midnight.
  - The host may work in smaller chunks to stay responsive.
  - Detailed mode may stop at any minute.
- A switch takes effect at the next midnight: after the land and the day's work (`on_day`),
  before that day's first event.
- An activity running at midnight finishes as planned. It is never restarted or redrawn.

### 3. Determinism

- Every draw is keyed by seed, purpose and semantic identity: person, household, field, day or
  counter. No draw is keyed by speed, worker, wall time or the number of advance calls.
- A save plus a sequence of (mode, days) reproduces exactly.

### 4. Approximations are declared

- Each optimisation is one of two kinds:
  - **Exact:** Detailed histories stay byte-identical, shown by digest.
  - **Approximate within a declared domain:** it can be switched off in tests and is covered
    by the statistical test.
- M3c's approximations, used in Accelerated mode only:
  - **A household view:** the options that do not depend on who decides (the purchase, the
    household to ask, paid work, the best place for each gathering, digging and field activity),
    each worked out when a member first needs it after midnight and kept until midnight or the
    household's own consequential step: a deposit, work on a field or a building, making,
    trying, or a choice that lays claim to something (new ground, a building or a workshop
    begun, goods to buy or ask for, paid work). A gift, a trade or paid work done refreshes every
    household's view, as goods move between them. Births and deaths come at midnight. Detailed mode works them out at every decision.
  - **Leisure blocks:** rest, the hearth and play last the run of sessions Detailed mode would
    live before choosing something else: a geometric number drawn from the chance with which the
    leisure was chosen. A block that ends so makes the next decision pass that leisure over, as
    Detailed mode's switch would. A block is cut at the next consequential boundary (when hunger
    begins, the sleep threshold, sunrise, the last light for water, the evening's start, sunset,
    the evening's end) or at the activity's longest session (`max_minutes`, a tuning value). The
    next decision is then made afresh, which leaves the run the same in distribution, as the
    draw is memoryless. As first built, a block lasted until the boundary. Gate B's first
    evaluation showed this kept people at home: 8 % less time at the hearth and 11 % less
    walking (§9, 2026-10-07).
  - Neither is saved. Both are set aside at midnight, so a save at an Accelerated midnight lives
    on as if never saved.
- Statistical samplers that replace per-event machinery come later, one subsystem at a time
  after profiling: company from co-location pools, crime (M4), fire (M6). Each passes the same
  test.

### 5. The consistency tests

- **Gate A, on every push.**
  - Thirty one-day advances equal one thirty-day advance.
  - Detailed, then Accelerated, then Detailed again reproduces exactly.
  - A save at an Accelerated midnight, loaded and continued, matches an uninterrupted run.
  - Each exact optimisation leaves Detailed histories byte-identical.
- **Gate B, nightly and on pull requests into main** (`civ-host consistency`).
  - Fixtures from at least two worlds, run several times in each mode for a year from the same
    midnight save. A test hook redraws the tie-break stream for each run.
    - As built, a river valley and a ria coast at 768² cells, with fixed identities.
    - Each lives by the minute to 1 January of its third year.
    - Five runs in each mode follow from there.
  - Aggregates of time use, food, harvest, materials, roofs, population, the Gini of goods and
    walks are compared against tolerances.
  - Tolerances are set once, before evaluation, from the spread of Detailed calibration runs,
    and recorded in §9. Widening one needs its own §9 entry.
    - As built, each aggregate's tolerance bounds the difference of the two modes' means.
    - It is the larger fixture's three standard errors of that difference at the Detailed
      spread, with a floor.
    - `--calibrate` runs eight Detailed runs from each fixture to find them.
  - The ledger, `problems()` and "nobody stuck" are checked exactly in every run.

### 6. The boundary

The wire is append-only:

- the clock carries its mode;
- 60×, 600× and Max are offered;
- Accelerated frames carry midnight positions and no trip updates, so observers must not
  interpolate trips in Accelerated mode;
- the speed is no longer clamped to 10× on load.

## Consequences

- Fifty-year runs cost minutes, not hours, while one model serves both speeds.
- Every behaviour added in M4–M8 is written once; an approximation must earn its place under
  Gate B.
- Plan §4.4's "statistically" is amended (§9): Accelerated runs the same event machinery a day
  at a time, with declared approximations.
- Forbidden:
  - state that exists only in one mode;
  - a draw keyed by speed or call count;
  - an undeclared approximation;
  - widening a tolerance without a §9 entry.

## Alternatives considered

- **A statistical day budget** (a few time slots per person per day, effects through
  hour-based functions, walks as travel-time lookups). The speed-up is larger at this scale,
  about 3–10×, but every behaviour then needs a slot twin: a second rule set for M4–M8 (plan
  risk 6). It stays available per subsystem, under Gate B.
- **Detailed mode only, unpaced.** It is byte-exact, but the 1.3–2× from exact work alone does
  not reach the dashboard's or M8's budgets.
- **Switching at any minute.** 01-03 §4.2 allows it once plans, reservations and partial totals
  are carried across. Midnight boundaries are simpler and enough for now.

## Revisit when

- M8's scale gate (50k agents) needs about 3 µs a person-day: parallel views, co-location pools
  and per-subsystem samplers.
- Unreal's bridge needs Accelerated frames between midnights.
- Mid-day switching is wanted.
