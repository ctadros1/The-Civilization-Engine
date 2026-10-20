# M8 design brief: scale, hierarchical pathfinding and crowds

**Scope.** Plan §7's M8 ("The growing city (v1)") lists "Hierarchical pathfinding", "Crowd LOD at scale" and "**Scale gate: 10–50k agents at 60 fps, with Accelerated Max at ≥1 year per minute**"; its demo is "a 30k-person planned capital next to an organic port town, with traffic, in the same world". Plan §4.3 sets M8 at "10–50k total" in "2–3 (up to 5)" settlements. §4.4 puts the target "at 50k agents" ("A century is then under two hours"). §4.5 keeps "everyone ... simulated in full through v1", with the cohort model in M9. §4.6 sets 60 fps at 1440p on the 4070 Ti, "Kernel memory: ≤ 4 GB at 50k agents" and "Kernel frame publish: ≤ 2 ms of UE game-thread time to consume". This brief covers:

- where the kernel stands, with every speed measurement the plan records, and the gap to the gate;
- which changes could close how much of the gap, and which keep histories exact (Gate A) or need Gate B;
- hierarchical pathfinding for walking now, and for the roads, carts and boats M8 adds;
- what the kernel owes a renderer for 60 fps and crowd LOD (the Unreal client is a separate track);
- how to measure the gate, and what the plan should do if M8 cannot meet it.

Transport, traffic, planning and grammar v3 have their own briefs. This one names where they add cost.

**What exists to build on** (checked in the code):

- **One event loop** (`civ-core/src/scheduler.rs`, `Scheduler<E>`). It is one `BinaryHeap` ordered by time, phase, tie-break and sequence, with cadence boundaries delivered finest first. "The random tie-break exists so that equal claims are not won by whoever was scheduled first". The tie-break comes from one `Rng64` stream, drawn as each event is scheduled (`tiebreak: self.tiebreak.next_u64()`), so how equal-time events are ordered depends on the order the whole world scheduled them in. `Sim::advance_minutes` (`civ-sim/src/lib.rs`) runs every event and each midnight's work (`land.advance_to_day`, `people.on_day`) on one thread, and calls `refresh_nav` after every event.
- **Modes.** `Mode::of_speed`: any speed above `MAX_DETAILED_SPEED` (10×) is Accelerated, and Max is `f32::INFINITY`. `Approximations { leisure_blocks, household_view }` (`civ-agents/src/population.rs`) are both off in Detailed mode. Any `Ask`, `Trade`, `Fetch`, `Hire` or `Take` step anywhere clears every household's view in the world (`self.views.clear()`).
- **A decision** (`Population::decide`). It clones the household (`self.household(hh_id).cloned()`), fills `Facts` from about twenty fact functions (gathering, petition, poll, campaign, watch, carry, crossing, well, levee, enclosure, ditch, band, fire, tend and others), then scores the options. People make about 14.5 decisions a person-day (plan §9, slice W, measured at 65 people).
- **Searches that are local in their result and global in their cost.** Each of these goes through every household in the world and keeps those of the right settlement: `find_deal` and `for_sale`'s `on_offer` (`population/market.rs`), `best_giver` (`population.rs`), `take_target` (`population/crime.rs`) and `plan_errand`. `find_partner` (`population/life.rs`) goes through every living person. While anything is burning, `fire_facts` (`population/fire.rs`) goes through everyone, up to three times for each threatened building, at every decision. `hearth_company` (`population/ties.rs`) returns everyone at the hearth for each session. ADR-0018 §7 says such searches look within the settlement "so cost grows with the settlements rather than the world". The results do; the cost does not.
- **Walking** (`civ-world/src/nav.rs`). A* runs on the 8 m grid with Tobler speeds. Its bound is admissible because "a fully worn trail restores the full speed and never beats it". Each settlement has one travel field (a Dijkstra) from its hearth. Landmark bounds (ALT, `LANDMARKS` 8) are computed over one box that holds all worn ground in the world. They are rebuilt after each monthly survey on up to `LANDMARK_THREADS` (4) scoped threads, "each the same whichever thread runs it". That is the only work in a simulated day that runs on more than one thread. Routes are cached by (from, to) cell in two generations of `ROUTE_CACHE_MAX` (50,000), emptied at each survey. rayon is used only by civ-world's terrain generation (`refine.rs`, `routing.rs`, `lem.rs`).
- **The bench** (`civ-sim/examples/bench_village.rs`). Its flags include `--people`, `--groups`, `--known`, `--max`, `--exact`, `--only leisure|view`, `--load`, `--save-to`, `--profile-after` and `--digest`, which hashes every save section. Every exact speed change since slice L has been proven with it.
- **The host** (`civ-host/src/engine.rs`). At Max it lives "day after day for most of the tick (at least one)" (`RUN_AHEAD_SHARE` 0.8) and publishes a snapshot at most every 100 ms (`SNAPSHOT_INTERVAL`). `frames/people.rs`'s `person_briefs` puts every living person, sorted, into every snapshot. ADR-0003 §2 planned to move trips "to the ordered delta stream" from M2, with a revisit trigger at "about 64 KB (roughly 1,500 people)". No delta stream for people exists. Each ticker frame, the web observer redraws every person as a vector circle in one PixiJS `Graphics` (`web/src/map/view.ts`, `drawPeople`).
- **The gates.** Gate A runs on every push (ADR-0011 §5). Gate B runs nightly and on pull requests into main, on fixtures of 768 cells with tens of people. `civ-host dashboard` lives five worlds on four threads, one world to a thread.

## Where the reports agree

Eight points recur across 01-01, 01-02, 01-03 and 01-08, with support from 01-04, 01-11, 14-02 and 14-03. I treat them as invariants.

1. **Stop work before adding threads.** The order is: stop unnecessary work, bound queries, improve layout, parallelise, specialise (01-02 §2.3). Make traversal and spatial queries parallel-friendly before building a scheduler (01-01 §3, BioDynaMo). Build the serial model first and parallelise second (01-01 §4.7).
2. **Cost should follow changes, not population.** Separate continuing an activity from reconsidering it (01-02 §2.2). "50,000 citizens should not imply 50,000 fresh shortest-path searches" (01-08, executive recommendation). Choosing a destination and finding a route to it are separate steps (01-08 §1; 01-02 §4.4).
3. **Bounded candidates, not scans.** "'Find food' should not mean 'scan all buildings and pathfind to each'". At the same time, "Engine indexes must not accidentally give people omniscient knowledge" (01-02 §1.5). Search a bounded one-to-many set (01-08 §4).
4. **One model at every speed.** Approximations are declared and tested, and their tolerances depend on world state, not on the speed chosen (01-03 §4.1, §4.2, §4.5). When work piles up, let the achieved speed fall rather than loosen the model (01-03 §4.2; 01-02 §4.6; 01-01 §4.4).
5. **Threads must not decide outcomes.** "race-free does not mean order-independent" (01-01, executive recommendation). Key draws by stable identity, never by worker or call count (01-01 §4.2; 01-03 §4.5). Admit results at declared boundaries, never in the order tasks finish (01-01 §4.3; 01-08 §8, stage 1).
6. **Presentation LOD is not simulation LOD.** "Do not use camera visibility to decide whose economic behavior is skipped" (01-01 §4.4; 01-02 §4.6). Rust owns identity, position, route and activity (14-03 §4.1).
7. **The frame never waits on the simulation.** 60 fps gives 16.67 ms per frame, which "does not require a simulated day to finish inside that frame" (01-03 §2.4; 01-04 §4.9). Use immutable snapshots, and never let the game thread wait (14-03 §4.2; 01-01 §4.5). "Advance in bounded work chunks so pause and input remain responsive" (01-03 §4.4).
8. **Measure on the target hardware, with real workloads.** Use saved worlds of 10k and 50k people, sweep thread counts from 1 to 24, and record the CPU model (01-01 §4.7; 01-02 §4.8; 01-11 §2.5, §4.7). No published benchmark matches TCE's workload (01-01 §2.1; 01-02 §2.1; 01-03 §2.3; 14-03, executive recommendation).

**What games and models teach to avoid:**

- a synthetic benchmark where every agent thinks: Factorio measured 14.9× on a synthetic save and 9.5 % on a playtest save (01-01 §3);
- turning an ECS or GPU benchmark into a citizen count (01-02 §2.1);
- refresh periods that change with the speed setting, as in Micropolis (01-03 §3.4);
- a lock on every person or a thread per settlement (01-01, executive recommendation);
- letting a wall-clock deadline decide who gets updated (01-01 §4.4);
- unnecessary pathfinding, which Cities: Skylines II had to patch out (01-08 §6);
- assuming aggregation gives a universal 100× (01-04 §2);
- a second movement simulation inside the renderer (14-03 §4.1);
- accepting new baselines or widening thresholds so a patch passes (01-11 §2.3).

**Where they differ, or differ from the plan:**

- **The gate's population.** §4.4 says 50k. §7 says 10–50k. The demo needs 30k in one city, plus a port town.
- **Full detail.** §4.5 and ADR-0018 §7 ("Lower detail for settlements nobody watches is not built") keep everyone in full through v1. 01-04 §4.2 makes "lightweight individuals" the fallback and the reference for any cohort model. §1's arithmetic below says 50k at a year a minute needs M9's machinery.
- **Phases against an event loop.** 01-01 §4.2 and 01-02 §4.2 recommend explicit phases in which agents decide in parallel from a snapshot. TCE decides one event at a time on live state. Changing that changes who sees what, and "'Everybody reads the previous tick' is itself a modeling choice" (01-01 §1.1). It would not be exact; it would be a different model. I keep the event loop in M8 and parallelise around it.
- **Determinism.** Plan rule 6 says "Determinism is not a goal". ADR-0011 §3 says "A save plus a sequence of (mode, days) reproduces exactly", and every speed slice since L has been proven by digests. The reports ask only for a repeatable diagnostic mode (01-01 §4.2; 01-11 §2.1). For speed work I keep the project's stricter rule: results are identical whatever the worker count.
- **Parallel preparation.** Slice AL's design cites "01-03 §4.4" for parallel preparation. The Victoria 3 account of "parallel preparation followed by serial updates" is in 01-03 §3.1. §4.4 has workers "calculate independent proposals or homogeneous batches from consistent inputs".
- **60 fps.** §4.6's target is for Unreal, and the Unreal client is not built (README). 14-03's executive recommendation calls 10,000 visible citizens "a reasonable prototype target, not a verified performance result". M8's kernel can only deliver its own share of that target.
- **Faster modes break today's bounds.** nav.rs's bound holds because nothing is faster than a worn trail. 01-08 §2A: "Adding a faster connection, increasing maximum travel speed, or changing mode permissions can invalidate them". M8's roads, carts and boats do exactly that.
- **Walks measured from the hearth.** Since slice AL, a household's walks are timed on its settlement's travel field, from the hearth. In a city of 30k that misplaces walking costs. The question belongs to the planning brief, but its likely fix, travel fields per district, is a routing structure.

## 1. The gap, measured

| Date, slice | Conditions | Result |
|---|---|---|
| 2026-10-04, L | Founding spring of 200, Detailed | 248 → 85 ms a day; the first year 53 → 39 s, exact |
| 2026-10-05, S | 40 and 125 people from saves, Detailed | 14.9 → 11.1 and 37.6 → 30.9 s a year, exact |
| 2026-10-07, W | 65 people, 120 days | 57.2 → 44.2 ms a day with both approximations (23 % faster), about 16 s a year |
| 2026-10-07, before X | 1,000 at Max | 653–811 ms a day, "about four minutes a simulated year"; route searches 35 % |
| 2026-10-07, X | 1,000 at Max, exact | 485 s a year against a 2-minute goal, **not met**; 2,000 cost 3.2× 1,000; route searches 86 % on the slowest days |
| 2026-10-09, M5a design | One village at Max, 768 cells | 3,000 at 4,105 ms a day over the first 10 days, 5.2× 1,000's 785 ms |
| 2026-10-09, AL | 3×1,000 at Max, seed 2, 1,024 cells | 1,550 → 1,179 s a year (19.6 min) against a 10-minute budget, **not met**; one settlement of 1,000 took 273 s; 8,000 (3×2,667 on 2,048 cells) took 14.6 s a day over the first 30 days, "about 89 minutes a year at that rate" |
| 2026-10-10, AY | 3×1,000, `--known --max` | 1,286.5 s a year, about 9 % slower than AL |
| 2026-10-11, BD | 3×1,000 (the entry says "a year at full detail") | 1,447.4 s a year; the slowest day, 308, took 18.1 s with 3,164 people |

Every run was a single run on the four-core development container. None was on the Windows PC (an i9 of the 13th generation; 01-01 §2.3 asks that the exact model be recorded). Since BD, M6b's later steps, M6c and M7a–d have landed, and §9 records no 3×1,000 bench for them. Today's figure is unknown and probably higher: what landed between AL and AY added 9 %, and what landed between AY and BD 12.5 %.

**The gap (my arithmetic, from the table):**

- **The day budget.** A year in a minute is 164 ms a simulated day, whatever the population. Today a day of 3×1,000 takes 3.97 s on average, and the slowest took 18.1 s.
- **The cost of a person-day.** The gate allows 16.4 µs at 10k, 5.5 µs at 30k and 3.3 µs at 50k (ADR-0011's revisit trigger says "about 3 µs a person-day"). Today 1,447.4 s over about 1.1 million person-days is about 1.3 ms. At a constant cost per person-day the factors needed are about 80×, 240× and 400×. Converted to decisions with 01-02 §2.2's D = N·d·A/86,400 at d = 14.5, the gate needs 0.88 million decisions a second at 10k and 4.4 million at 50k. Today about 11,000 a second are made, about 90 µs each with all other work included.
- **The cost per person grows with the population.** Measured exponents run from 1.5 (3,000 against 1,000 in one village) to 1.7 (2,000 against 1,000). Extrapolating, not measuring: 10k in three settlements would take about 8,800–11,200 s a year, about 150–190× the budget. One city of 30k alone would take 55,000–108,000 s, assuming a single settlement of 1,000 now costs about 335 s (AL's 273 s grown by the 23 % the 3×1,000 bench grew from AL to BD). Five settlements of 10k would take about 53,000–84,000 s, 880–1,400× the budget.
- **Daily life at scale.** At 10× a day has 90 s; at 1× it has 900 s. Detailed mode costs about 1.3× Accelerated (slice W). Extrapolated, a day of 10k in Detailed takes 31–40 s, which 10× allows. A city of 30k takes 200–390 s, so 3× at best. At 50k even 1× is at risk. Watching the city at 10× fails before Max does.

## 2. Where the time can come from

| Lever | Proof | Plausible factor (my estimate) | Basis |
|---|---|---|---|
| Measure at 10k and profile | none needed | none by itself | Nothing has been measured above 8,000 people, or since BD |
| Searches indexed by settlement, pruned only where provably exact | Gate A | Removes the excess over linear: 1.8–2.6× at 10k, 4–9× at 50k | Exponents 1.5–1.7; ADR-0018 §7 |
| Exact constant-factor work from the profile | Gate A | 1.5–2.5× | Slices L (~3×), S (1.22×, 1.34×), X (1.24×), AL (1.66× over days 1–60) |
| Routing for a city (§2.4) | Equal route times; one digest change for ties | 1.1–2× of the whole | Routes were 46 % of a day after surveys before ALT (AL) and 86 % on slow days (X); unknown since |
| Exact parallel preparation on a bounded pool | Gate A at any worker count | 1.3–2× on 4 cores, 1.5–3.5× on the i9 | Amdahl with a serial event loop (01-01 §2.2); a fully parallel model reached 4.45× on 8 threads (01-01 §2.1) |
| More Accelerated approximations | Gate B | 1.5–3× | W gave 1.3×; ADR-0011's alternatives put a "statistical day budget" at "about 3–10×", kept "available per subsystem, under Gate B" |
| The i9 against the container | Measure | Unknown | Never measured, so I count nothing for it |
| Lightweight or cohort settlements | Gate B and a new contract | Large; unknown | 01-04 §2: no universal 100×; plan §4.5 defers it to M9 |

**What it adds up to.** Multiplying rows two to six at 10k gives about 7–140×, against the 150–190× the extrapolation needs. Multiplication is itself optimistic: once serial work is cut, a smaller share of what remains can run in parallel (01-01 §2.2). The 10k gate is within reach only if every range comes in near its top and the i9 helps. At 50k the same levers give about 15–470× against 880–1,400×, so 50k at a year a minute is not plausible in M8 without M9's lower detail. M8's own contents (carts, boats, congestion, taller buildings, planning) will add cost per person, as AY and BD did.

### 2.1 Measure at scale first

**Reports.** Benchmark saved worlds of 10k and 50k: dispersed farms, a dense capital, market contention, mass replanning and churn (01-01 §4.7). Measure four things: decisions a second in the handlers alone, events a second end to end, simulated days a second, and frame times taken together with throughput. Also count path searches and nodes expanded, candidates per query, allocations and stalls in the tail (01-02 §4.8). Keep a 10k fixture in CI and a 50k run on the target PC (01-11 §2.5), and accept performance on the target machine (01-11 §4.7).

**My proposal.**

- **Fixtures.** Keep 3×1,000 for continuity. Add three groups of 3,334 on the default 2,048-cell map, and one settlement of 10k standing in for the capital. Each is saved after it has been founded and lived into a year.
- **What to record.** A year at Max, and 30 days through harvest, the slowest weeks (slice X). Record wall time per day with its p50, p95 and the worst day; decisions; events; route searches and cells expanded (`nav::cells_searched` already counts them); candidates per search; how often household views are reused; peak memory; save and load time and size; snapshot bytes and build time.
- **Where.** On the container and on the i9, the model and core count recorded.

### 2.2 Local searches, exactly

**Reports.** Give agents shared search structures so they search less (01-02 §3, Songs of Syx's 16×16 chunks with destination summaries). Use service and resource indexes for decisions (01-02 §1.5). "Partition by measured work, not equal map area. One dense capital may be more expensive than many rural regions" (01-02 §4.3). Compare each cache against a full recomputation (01-11 §1.5).

**My proposal.**

- **Keep indexes by settlement, in the order today's scans go**, so every result comes out the same: households, sellers holding offers, givers, unpartnered adults by sex, and who is at each hearth. Add a check in debug builds, on in every test, that each index agrees with the scan it replaces (slice AL's precedent).
- **Within a settlement, prune only where provably exact.** For example, take sellers nearest first and stop once the walk alone costs more than the best saving found, as the gathering-places search already does (plan §9, slice S).
- **Show that the cost no longer grows faster than the people.** Measure 1k, 3k and 10k from saves, and prove each change by digests at days 10 and 30.
- **Knowing a few sellers is a model change.** Whether a buyer in a city weighs every seller (the model as built) or only those it knows (ADR-0018 §4) is a question for the design, with a dashboard rerun. Cities: Skylines II found that some speed gains "may require changes to simulation behavior" (01-02 §3).

### 2.3 Constant factors

**Reports.** Keep the hot state compact and the common paths free of allocation (01-02 §2.3). Fitting in memory is not the same as being cheap to reach repeatedly (01-02 §1.1). Watch the populations of things that are not people (01-02 §3, Dwarf Fortress's items).

**My proposal.** Work from the 10k profile, time-boxed as slices X and AL were. Likely candidates, none of them measured yet:

- the household cloned at each decision;
- fact functions worked out before any option asks for them (make them lazy, as `hh_view`'s cells already are);
- `fire_facts` going through everyone where the household's members would do;
- each hearth session returning everyone present;
- records that only ever grow (the chronicle, reports, ties) where they are read hot.

### 2.4 Hierarchical pathfinding

**Reports.**

- Routing work ≈ Q·C_q plus the costs of metric and topology updates (01-08 §1). Optimise bursts and repeated work before the last microsecond of a query.
- ALT is "excellent initial production router" (01-08 §2A). A portal overlay that keeps every boundary crossing stays exact; HPA*'s sampled entrances do not (01-08 §2D, §2F). Reverse trees serve popular destinations (01-08 §2G).
- Adopt an accelerator only when it wins the whole workload: Q(C_A* − C_acc) > the cost of keeping it (01-08 §3, §8 stage 3).
- Treat the traveller's state as (location, mode, vehicle) (01-08 §5). "At high time acceleration, preserve the route-demand and capacity model" (01-08 §4).
- Sixteen directed landmark tables over 100,000 vertices take 12.8 MB (01-08 §7).

**My proposal.**

- **Keep A* with ALT as the router**, and every route checked in tests against Dijkstra on the same snapshot (01-11 §1.5).
- **Landmarks per region.** Build one set over each settlement's walked ground, not one box over the whole world. My arithmetic: at 64 B a cell, one box over the default 2,048² map is about 268 MB.
- **Finish what slice AL left unbuilt.** Size the route cache by population, and add reverse travel fields where a destination (a hearth, a well, a market door) draws enough searches to pay for one. Measure at 10k first.
- **Add a two-level overlay** for long walks (between settlements, errands, bands): cells of the grid with exact boundary crossings (01-08 §8 stage 2). Route times stay the same; which of two equally fast routes is walked may change, a one-time change of digests checked for equal times, as in X and AL.
- **Before carts or roads land:** routing state by mode, and bounds built from the fastest mode, or one table per mode. Test with a road faster than any trail.
- **Travel fields by district** for a city, if the planning brief wants walks timed from homes rather than the hearth. It is a model change, so the dashboard runs again.
- **Congestion** belongs to the transport brief: smoothed and bucketed by time (01-08 §4), and the same at every speed (01-03 §4.3).

### 2.5 Parallel and exact

**Reports.**

- Use a private, bounded Rayon pool, never the global one in a DLL. Use chunk jobs (try 64, 256 and 1,024), not a task per person (01-01 §1.3, §4.5).
- Budget threads for the whole process: 8, 12 and 16 workers interactive; 20, 24 and 32 headless (01-01 §4.4).
- Test sensitivity to the schedule by varying thread counts and chunk sizes (01-01 §4.6). A Factorio desynchronisation showed up only on some core counts (01-11 §2.1).
- Regional queues that run ahead need "a proven boundary rule" (01-02 §4.2). Keep settlements synchronised at their interaction boundaries (01-04 §4.8).

**My proposal.**

- **One private pool, sized at start.** Only pure work runs on it, and its results are applied in id order. No floating-point reduction is summed in the order work completes. `nav::landmarks` moves onto the pool.
- **What may run in parallel, exactly.**
  - Derived structures after a survey or at midnight: landmarks, travel fields per settlement, routes for each home's habitual destinations. Within a survey a route is a function of its ends, so a route found early is the same route. This must be shown, including that scratch scores reused on one thread do not leak into a result.
  - The parts of each midnight that read the world and write only the household's own record.
- **Gate A at 1, 2, 4 and 8 workers**, with digests equal across all four.
- **Decisions stay serial.** Each event sees the one before it.
- **Running settlements side by side is not for M8.** It needs:
  - tie-breaks keyed by an event's identity rather than drawn from one stream (a one-time change of digests and of the scheduler's saved state);
  - state split by settlement;
  - every effect between settlements turned into a message at a boundary: visits, errands, one polity over several settlements, shared seats, bands.

  That is the same synchronisation M9's cohort settlements need (01-04 §4.8).

### 2.6 Accelerated mode at scale

**Reports.** Approximate one subsystem at a time, after profiling, under declared tolerances (01-03 §4.6, §4.5). An exact endpoint sampler is not an exact day resolver (01-03 §1.2). Draw checkpoints from both modes (01-03 §4.5). ADR-0011 §4 already names "company from co-location pools, crime (M4), fire (M6)" as later samplers.

**My proposal.**

- **Narrow the views' invalidation.** Today any trade, gift, fetch, hire or taking clears every household's view in the world. At 10k that happens constantly, so the approximation stops saving anything. Clear the views of the parties and of their settlement's market instead.
- **Build every household's view at midnight, in parallel, from the midnight state.** This is slice W's original wording, "A household view built at midnight". The view is already an Accelerated-only approximation, so this changes the approximation, and Gate B judges it.
- **One more sampler, chosen from the 10k profile**: company from co-location, or continuing a run of work as leisure blocks continue rest.
- **Add Gate B fixtures** of several settlements and one at scale, with tolerances fixed before evaluation. Any saving that depends on N must be judged where N is large.

### 2.7 The host, the frame and crowds

**Reports.**

- A 64-byte record for each of 50,000 citizens is 3.2 MB, 64 MB/s at 20 Hz. Send snapshots at 10–30 Hz and interpolate, and "Do not let the game thread wait" (14-03 §4.2). The bridge should take about 1 ms of the game thread (14-03 §4.5); render-bridge mutation 0.5–1 ms (14-02 §5.5).
- Choose tiers by projected size, and let "A cap ... demote representation quality, not randomly delete conspicuous citizens". At extreme acceleration, choose a visualisation policy (14-03 §4.3; 01-02 §4.5, "timelapse").
- Keep each person's appearance seed so it survives re-instancing (14-03 §4.4).
- Send buildings as a change stream with revisions, coalesced at acceleration (14-02 §7.1, §7.2).
- Prove the renderer on recorded trajectories without the simulation (14-03 §4.6).

**My proposal.**

- **Max in chunks of a day.** The host advances some minutes at a time within each tick's budget and still publishes and saves only at midnight. Gate A's partition property already proves this exact, and ADR-0011 §2 allows that "The host may work in smaller chunks". Pause and commands are then answered within a tick, not after an 18-second day.
- **People on the delta stream**, as ADR-0003 §2 planned, because its revisit trigger passed at about 1,500 people. The snapshot carries the people in view, or a count for each tile, and the rest comes by query.
- **Measure what a frame costs at 10k and 50k**: bytes, build time, and the time to consume it, against plan §4.6's 2 ms.
- **The web observer as M8's stand-in for crowd LOD.** Batch people into instanced sprites rather than redrawing vector circles each frame. Measure 60 fps at 1k, 5k, 10k and 20k visible from a recorded snapshot (14-03 §4.6). Unreal's crowd tiers are that track's own work, but they use the same records.

## 3. What to defer, and why

| Defer | To | Why |
|---|---|---|
| Lightweight or cohort settlements | M9 | Plan §4.5; 01-04 §4.6 makes switching a transaction; the same synchronisation as settlements side by side |
| Settlements run side by side | M9 | Needs keyed tie-breaks, state split by settlement and messages at boundaries (§2.5) |
| A phased model with parallel decisions | Not planned | It is a different model (01-01 §1.1); every behaviour would change |
| CCH or hub labels | When ALT and the overlay lose | 01-08 §3, §8 stage 3; maintenance can cost more than queries save |
| GPU simulation | Not planned | The GPU renders first (01-01 §4.4; 01-03 §4.4) |
| Time skips | M9's decision | Plan §4.5 and §8, open question 1 |

## 4. Guards

**Exactness.**

- Every exact change shows equal digests of every save section from the 3×1,000 save and a 10k save, at days 10 and 30, in both modes.
- Parallel changes do so at 1, 2, 4 and 8 workers.
- Routing changes show equal route times against Dijkstra. Where ties change, the one-time digest change is recorded in §9.
- Indexes keep a debug check against the scan they replace.
- Nothing is keyed by worker, wall time or chunk (ADR-0011 §3).

**Gate B.** No tolerance is widened without its own §9 entry (ADR-0011 §5). A new approximation can be switched off in tests and comes with a fixture where its saving is large.

**Plot.** Nothing in the model depends on speed or camera (01-01 §4.4; 01-03 §3.4). No settlement size is a target. When a world falls behind, the achieved speed falls and the observer says so; no causal step is skipped (01-02 §4.6).

**Usable.**

- Pause and commands are answered within a tick at Max.
- A 10k world saves and loads, with the time and size recorded.
- Peak memory is measured against plan §4.6's 4 GB.
- The new-world dialog offers worlds of thousands only once the agreed gate is met (M5 design: "offers no worlds of thousands until the scale budget is met").

**Time-boxed.** Each speed slice has a goal that is "a measure, not a gate" (slice X). One that misses its goal is recorded as X and AL were, never rerun until it passes (01-11 §2.4).

## 5. Suggested slices

Each lands on its own, is measured before and after, and keeps exactness or names the Gate B change it makes.

1. **The baseline at scale.** No change to the engine. Add the fixtures and the counters of §2.1. Run on the container and the i9, with a callgrind profile at 10k, and measure the web observer's frame time from a recorded snapshot of 10k and 50k. Log the result in §9 as M8's starting point.
2. **Local searches, exactly.** Settlement indexes in scan order with debug checks, and exact pruning where the bound holds. Gate A digests; measure the growth exponent from 1k to 10k.
3. **The host and frames at scale.** Max in chunks within a day, people on the delta stream (wire append-only), and batched drawing in the web observer. Measure pause latency, snapshot bytes and fps. Gate A's partition tests.
4. **Constant factors.** The profile's top items, exact, within a fixed time box.
5. **Routing for the city.** Landmarks per region, the route cache sized by population, reverse fields where searches pay for them, the two-level overlay, and bounds that faster modes cannot break, ready for transport. Dijkstra oracle; one-time digest change for ties, recorded.
6. **Parallel preparation.** The private pool and the pure derived work of §2.5. Equal digests at 1, 2, 4 and 8 workers, and the speed-up recorded on both machines.
7. **Accelerated at scale.** Narrower view invalidation, views built at midnight in parallel, and one sampler from the profile. Gate B with fixtures of several settlements and one at scale.
8. **The gate.** Measured on the i9, after M8's transport and buildings are in, under the ADR's definition. If it is not met, record what was reached and amend as the ADR says.

## 6. Open questions for the designer

1. **Population.** Does the gate name 10k, 30k (the demo) or 50k (§4.4)?
2. **Machine.** Is it measured on the i9 headless, or with Unreal running, whose threads take a share (01-01 §4.4)?
3. **World.** Which world and stretch counts: three settlements of 3,334, the demo's city, a whole year, or the harvest weeks?
4. **Approximations.** Does the gate allow Accelerated's declared approximations, and how many more?
5. **Ties.** May speed work change digests once, to key tie-breaks by event identity, so that settlements can run side by side later?
6. **Lower detail.** Are lightweight settlements (01-04 §4.2) allowed in M8, against plan §4.5?
7. **60 fps.** Is the web observer M8's measure of 60 fps, with Unreal measured on its own track?
8. **Responsiveness.** What pause latency at Max is acceptable, and what is the worst day the host may live in one piece?
9. **Seeing sellers.** Should a buyer in a city weigh every seller (as built) or those it knows (ADR-0018 §4)?
10. **Appearance seed.** Is it saved now, for crowds (14-03 §4.4), or when Unreal needs it?

## 7. What an ADR must decide

One ADR, "Scale and the M8 gate", amending ADR-0011's revisit trigger and ADR-0003 §2's delivery:

- **The gate's definition.** Population, machine, fixture, stretch of the year, approximations allowed, and how it is measured: p50 and worst day, years a minute, memory, pause latency, frame cost.
- **The fallback**, decided before the slices run. My recommendation:
  - M8 keeps 10k at ≥1 simulated year a real minute on the i9 at Max as its gate;
  - the demo's 30k is measured at Max and at 10×, not gated;
  - 50k at a year a minute moves to M9, whose contents already include aggregate LOD and 100k+.

  If after the time-boxed slices even 10k misses, amend the gate to the figure reached, with the profile and §1's arithmetic as evidence, as slices X and AL recorded theirs. Do not quietly carry the target forward.
- **The parallel contract.** A private bounded pool and its size, pure work only, application in id order, digests equal at every worker count.
- **People on the boundary.** The delta stream and the per-person record, plus what the snapshot carries at scale.
- **Routing with faster modes.** Admissible bounds per mode, and the overlay's exactness: equal times, with ties allowed to change once.
- **Keyed tie-breaks.** Whether the scheduler's tie-breaks are keyed by event identity now, or with M9's settlements side by side. I would defer it.
