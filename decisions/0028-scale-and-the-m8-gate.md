# ADR-0028: Scale and the M8 gate

Status: Accepted
Date: 2026-10-17
Milestone: M8a
Amends: plan §4.3's M8 row and §7's M8 scale gate; ADR-0011's first revisit trigger; ADR-0003
§2's delivery of people

## Context

Plan §7 makes M8's gate "10–50k agents at 60 fps, with Accelerated Max at ≥1 year per minute",
and plan §4.3 gives M8 "10–50k total" in "2–3 (up to 5)" settlements. The scale brief
(`docs/briefs/m8-scale.md`) reads 01-01, 01-02, 01-03, 01-04, 01-08, 01-11, 14-02 and 14-03 and
collects every speed measurement in §9. What it finds:

- **Where the kernel stands.** The last measurement, M6b slice BD (2026-10-11), lived a year of
  three settlements of 1,000 at Max in 1,447.4 s on the four-core container, the slowest day 18.1
  s; nothing has been measured since M6c and M7 landed. A year a minute is 164 ms a simulated day;
  a person-day costs about 1.3 ms against 16.4 µs allowed at 10k and 3.3 µs at 50k.
- **Cost grows faster than people.** Measured exponents run from 1.5 to 1.7 (slices X and AL):
  `find_deal`, `for_sale`, `best_giver`, `take_target`, `plan_errand` and `find_partner` scan the
  world and filter by settlement, and any trade, gift, fetch, hire or taking clears every
  household's Accelerated view. Extrapolated, 10k needs 150–190× and 50k 880–1,400×.
- **What can plausibly be won** (the brief's §2, its estimates): settlement indexes kept exact
  (1.8–2.6× at 10k), constant factors from a profile (1.5–2.5×), routing for a city (1.1–2×),
  exact parallel preparation (1.5–3.5× on the target PC), more Accelerated approximations under
  Gate B (1.5–3×): about 7–140× at 10k, 15–470× at 50k. 10k is reachable only near the top of
  every range; 50k is not, without plan §4.5's aggregate LOD, which is M9's.
- **Nothing has been measured on the target PC** (plan §4.6's Windows i9), and this container
  cannot.
- **Exactness binds.** The kernel is deterministic per world identity, Detailed and Accelerated
  agree by digests (Gate A) and statistics (Gate B), and the threaded landmark search writes each
  job's result by its index (01-01 §4.6: schedule sensitivity must be tested).

M8's other contents (ways and carts, plans, taller buildings) will add cost per person, as every
milestone since M5 has.

## Decision

### 1. The gate

- **M8's gate is 10,000 people at ≥1 simulated year a real minute** at Accelerated Max, with the
  declared approximations of ADR-0011 and those this ADR adds under Gate B: three settlements of
  about 3,334 people on the default 2,048-cell river valley, saved after founding and a first
  year, living their second year whole. It is reported as years a minute, with each day's wall
  time at p50, p95 and worst, peak memory against plan §4.6's 4 GB, and the pause latency at Max.
- **Measured where it can be.** The container measures every slice before and after; the gate's
  number is the target PC's, headless, its model and core count recorded (01-01 §2.3). Until the
  owner runs it there, the gate is reported on the container and marked not yet measured on the
  target.
- **The M8 demo's city (30k) is measured, not gated:** at Max and at 10×, each day's cost and
  what the observer sees.
- **50k at a year a minute moves to M9**, whose contents already include aggregate LOD and 100k+.
  Plan §4.3's M8 row becomes "10k gated, 30k measured"; the 60 fps half of the gate is the
  renderer's (§5).
- **If 10k misses after M8a's time-boxed slices**, the gate is amended to the figure reached, with
  the profile and the brief's arithmetic as evidence, as slices X and AL recorded theirs. It is
  not carried forward quietly, and no slice is rerun until it passes (01-11 §2.4).

### 2. Exact first

- Searches over households, sellers, givers, partners and those at a hearth are indexed by
  settlement and walked in the order today's scans walk, so every result is the same; each index
  keeps a debug check against the scan it replaces, on in every test. Pruning within a settlement
  only where a bound proves it exact (as the gathering-places search already prunes).
- A buyer weighs every seller of its market, as built. Weighing only the sellers it knows
  (ADR-0018 §4) is a model change, left to a later design with its own dashboard run.
- Every exact change shows equal digests of every save section at days 10 and 30, in both modes,
  from the 3×1,000 save and a 10k save.

### 3. The parallel contract

- One private, bounded worker pool, sized when the kernel starts (never the global pool in the
  DLL; 01-01 §1.3, §4.5). Only pure work runs on it: derived structures after a survey or at
  midnight (landmarks, travel fields, the routes of each home's habitual destinations), and the
  parts of midnight that read the world and write only a household's own record. Results are
  applied in id order; no floating-point sum is taken in the order work completes; nothing is
  keyed by worker, wall time or chunk (ADR-0011 §3).
- Gate A is run at 1, 2, 4 and 8 workers and every digest is equal.
- Decisions stay serial: each event sees the one before it.
- **Settlements side by side are M9's**, with the scheduler's tie-breaks keyed by an event's
  identity, state split by settlement and every effect between settlements a message at a
  boundary (01-04 §4.8). M8 keeps one event stream.

### 4. Routing for a city

- A* with landmark bounds stays the router, every route checked in tests against Dijkstra on the
  same snapshot (01-11 §1.5). Landmarks are built per region (each settlement's walked ground), not
  one box over the world.
- A two-level overlay with exact boundary crossings (01-08 §2D, §8 stage 2) serves long walks.
  Route times stay equal; which of two equally fast routes is taken may change once, a digest
  change recorded in §9 with times checked equal.
- **Before any mode faster than walking lands** (M8b), the bound is built from the fastest mode, or
  kept per mode, and tested with a way faster than any trail: a bound that a road breaks is not
  admissible.

### 5. People on the boundary

- People move to the delta stream ADR-0003 §2 planned (its revisit trigger passed at about 1,500
  people): the snapshot carries the people in view or a count a tile, the rest by query, the wire
  append-only.
- The host lives Max in chunks within a day, still publishing and saving only at midnight, so
  pause and commands are answered within a tick (ADR-0011 §2 allows the host smaller chunks; Gate
  A's partition tests prove it exact).
- The web observer draws people as batched instanced sprites and is measured at 60 fps from
  recorded snapshots of 1k, 5k, 10k and 20k visible (14-03 §4.6); Unreal's crowd tiers are that
  track's own, on the same records.

### 6. Accelerated at scale

- A household's view is cleared only for the parties of a transfer and their market, not the
  world's.
- Views may be built at midnight from the midnight state, in parallel (slice W's first wording);
  this changes an approximation, so Gate B judges it.
- At most one more sampler, chosen from the 10k profile (co-location company, or a run of work
  continued as leisure blocks continue rest), each switchable off in tests.
- Gate B gains fixtures of several settlements and one at 10k, tolerances fixed before any result
  is seen; no tolerance is widened without its own §9 entry (ADR-0011 §5).

### 7. Guards

- No settlement size is a target, and nothing in the model reads speed or the camera. When a
  world falls behind, the speed achieved falls and the observer says so; no causal step is skipped
  (01-02 §4.6).
- A 10k world saves and loads, its time and size recorded. The new-world dialog offers worlds of
  thousands only once the gate is met on the container.

## Slices

- **CF**: the baseline at scale, no engine change: fixtures (3×1,000; 3×3,334; one of 10k), the
  counters (each day's wall time, decisions, events, route searches and cells expanded, candidates
  a search, view reuse, memory, save size and time, snapshot bytes), a profile at 10k, measured
  on the container and logged in §9 as M8's starting point.
- **CG**: local searches, exactly (§2), with the growth exponent measured from 1k to 10k.
- **CH**: the host and frames at scale (§5).
- **CI**: constant factors from the profile, exact, time-boxed.
- **CJ**: routing for the city (§4).
- **CK**: parallel preparation (§3).
- **CL**: Accelerated at scale (§6).
- The gate itself is measured at M8's end, with M8b–M8d in, under §1.

## Consequences

- Easier: every later M8 part is built on a kernel whose cost stays local to a settlement; the
  boundary carries a city.
- Harder: indexes to keep in step, with debug checks; a worker pool whose exactness is tested at
  several sizes; a gate measured partly on a machine this container is not.
- M8's demo city is watched at whatever speed it reaches; 50k waits for M9.

## Alternatives considered

- **Keep the plan's 50k gate.** The brief's arithmetic puts it 880–1,400× away; no lever this
  ADR allows reaches it, and pretending would carry an unmet target to v1.
- **Lightweight settlements in M8** (01-04 §4.2). Plan §4.5 makes them M9's, and switching detail
  is a transaction (01-04 §4.6) that needs the same boundary synchronisation as settlements side
  by side.
- **Parallel decisions** (a phased model, 01-01 §1.1). A different model: every behaviour would
  change, and Gate A would compare against nothing.
- **Key tie-breaks by event identity now.** It changes every digest once for a benefit only M9
  draws; deferred with settlements side by side.
- **Weigh only known sellers now** (ADR-0018 §4). Faster, but a model change that would move every
  market; not a speed fix.

## Revisit when

- The target PC's measurement differs from the container's by more than the levers' ranges.
- M9 designs settlements side by side or cohort settlements.
- A mode, plan or building feature of M8 adds more than a fifth to a person-day's cost.
