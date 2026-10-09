# Development History

This journal condenses the main development path into dated entries. The project plan's §9 log is the more detailed record of implementation parameters, evidence, nudges and design choices. This file records why the work was staged this way and how the system changed at each stage.

## 2026-09-27 — Define the project before building the engine

The initial work established a plan of record: an endless, plausible society simulation in which people and institutions produce the story of a world. The plan set seven durable constraints: the kernel owns truth; the engine supplies primitives rather than plot; each milestone should leave a usable build; emergent behavior is time-boxed; realism means plausibility, not proof; determinism is not a goal; and ADRs are reserved for decisions that are costly to reverse.

The architecture began with three separations that still shape the repository:

1. **Simulation versus presentation.** Rust owns world state and rules. Web and Unreal clients observe it and submit commands.
2. **Authored vocabulary versus generated outcomes.** TOML content defines available activities, resources and building programs; households choose among what they can do.
3. **Research versus evidence.** Research briefs inform design; executable checks and recorded runs show what the current implementation actually does.

The target was deliberately broader than the first build: a lifetime-spanning, evolving civilization, rendered in Unreal. That remains the vision, not a description of today's implemented simulation.

## 2026-10-03 — M0: make a world inspectable and durable

M0 turned the plan into a working vertical slice. The team added a Rust workspace, a deterministic seeded terrain pipeline, water and river models, a localhost host, a TypeScript/PixiJS observer, versioned schemas, and snapshot saves. A user can make a world, explore its relief and water, run the clock, save it, and recover from an interrupted session.

Two decisions reduced future rework:

- `commons-wire` and `commons-persist` were staged as a separate, engine-neutral workspace. TCE-specific FlatBuffers schemas describe world messages and save sections.
- The web observer came first. It gives the kernel a real viewer and interaction loop before the Unreal client exists, while preserving the same transport boundary that Unreal will use.

The terrain work separated seeded world generation from simulation. A generated bed is fixed for the world's lifetime; later land changes must be recorded explicitly. That rule keeps saved terrain, generated maps and eventual Unreal reconstruction from becoming competing sources of truth.

## 2026-10-03 — M1: put people in the world

M1 added a founding band and then grew its behavior in small slices. People move on routes, satisfy needs, forage, hunt and fish, farm emmer, build huts, form families, have children, die, inherit and sometimes leave. Walking wears paths into the land. The observer can inspect a person's activity and decision receipt, send new families, run time ahead, and save and reload a world.

The first demographic and economic loop made the project testable as a simulation rather than only as a landscape viewer. Development checks moved from terrain constraints to population survival, food and water access, household activity, fields and ten-year runs. The M1 video and screenshots are in [`assets/m1/`](../../assets/m1/); they show the web build, not Unreal.

Several practical modeling boundaries were made explicit: agents are represented by tables and stable IDs rather than an engine-specific ECS; trips represent movement over time; activity decisions retain their reasons; the chronicle holds durable events; and frequently changing decision receipts are bounded rather than kept forever.

## 2026-10-03 — M2 groundwork: define a portable kernel host boundary

The kernel was packaged behind a small, versioned C interface (`civ-ffi`) so a future Unreal plugin can load it without pulling simulation code into the engine. The same kernel builds as `civ-host` for headless use. C ABI frames follow the same envelope and schema as WebSocket frames; buffers are caller-owned, ordered events are polled, and replaceable snapshots can be copied independently.

This is **kernel groundwork**, not an Unreal integration. The Unreal plugin, runtime terrain, renderer, HUD, packaging and Windows-PC development remain outstanding in this source baseline.

## 2026-10-04 — M3a: build an economy around household production

M3 was divided into M3a (village economy), M3b (knowledge and buildings), and M3c (seasons and time). M3a introduced explicit goods and recipes, tools that wear, practical skills, an accounting ledger, exchange at posted terms, firms and hired work, property regimes, leases and wealth measures. The observer gained market, workshop, land-tenure and wealth views.

The key engineering decision was to separate economic behavior from accounting invariants. People may make limited choices, but every unit of goods must still have a recorded source and destination. A money good is inferred from what settles the most payments; it is not a preselected game rule. Property regimes affect who holds and leases fields, while the simulation records actual field users and transfers.

The project recorded a meaningful limitation rather than hiding it: large villages can be created and run at full detail, but the early farming model does not yet support them through a bad harvest. Smoke runs showed large-band famine and departure. That finding became a time-boxed development nudge for M3c farming rather than a claim that the model is realistic or complete. Detailed observations live in §9 of [`PROJECT_PLAN.md`](../../PROJECT_PLAN.md).

## 2026-10-04 — M3b: carry knowledge and building state through a settlement

M3b through slice P added knowledge to individual people, learning from household work and teachers, discoveries, and techniques that gate activities. A technique can be introduced by the observer; it can also disappear from a settlement when its last knower is gone.

The hut grammar was extended with frame-building programs and derived spaces, storage and work areas. Households can choose among homes they know how to build; they can build granaries and workshops when the relevant technique and means allow. Buildings acquire part quality, wear by exposure, leak, receive upkeep and can fail under load. Settlements retain a fading record of failures; later frame structures can be sized more strongly in response.

On the `main` tree of 2026-10-04 (`fad23fa`), from which this journal was first written, slice Q had begun only with deterministic placement of deposit bodies in `civ-land`. Slices Q and R were completed on the development branch, with the M3b demo, by 2026-10-05; the next entry records them.

## 2026-10-05 — M3b slices Q and R: the ground people change, style, and the M3b demo

**Starting point:** the `main` tree at `fad23fa` (2026-10-04): slices M–P implemented and slice Q begun with deposit placement in `civ-land` (saves schema 18, wire 1.18, content API 17). Slice Q's five steps landed on 2026-10-04; slice R and the demo followed on 2026-10-05.

**Goal:** complete slice Q (deposits found by passing or digging, the god tool *place a deposit*, plot levelling, clay pits and quarries, clay crafts) and slice R (style with prestige copying), then run the milestone's demo.

**Implementation:**

- Every world lays down deposit bodies of clay, stone and flint from its seed once its founding band has arrived ([`civ-land/src/deposits.rs`](../../kernel/crates/civ-land/src/deposits.rs)). A village finds a body that shows at the surface when one of its people walks within 50 m of its edge, and a buried one when a levelled plot's cut reaches it. The observer can lay one down.
- Households level sloping plots by cut and fill, dig clay pits, stone quarries and flint pits with spoil heaps beside them, and dig each building's daub from a pit beside its plot ([`civ-land/src/earth.rs`](../../kernel/crates/civ-land/src/earth.rs), [`civ-agents/src/population/digging.rs`](../../kernel/crates/civ-agents/src/population/digging.rs)). Each earthwork is a record, and its changes to the ground are kept in tiles beside the generated bed.
- Clay crafts: pottery, 35 kg storage pots that keep food under a roof as a raised floor does, and a clay oven that bakes only once whole.
- Each household builds to its own taste in roof pitch, wall height to the eaves and overhang, moved once a year toward the new buildings its village admires ([`civ-agents/src/style.rs`](../../kernel/crates/civ-agents/src/style.rs), [`civ-agents/src/population/taste.rs`](../../kernel/crates/civ-agents/src/population/taste.rs)). The observer reads how each building was built and which it followed. `civ-host run --introduce` brings a technique to a founder as a world begins.

**Decision:** [ADR-0010](../../decisions/0010-ground-people-change.md): deposits are bodies with finite inventories, knowledge of a deposit belongs to a settlement, earthworks are records, the generated bed is never rewritten, and walking and water stay on the generated ground in M3b. Cob walls were deferred to a later mass-wall grammar. Style followed [ADR-0009](../../decisions/0009-building-components.md) §7, its rates inside research 11-02's test priors (`alpha` 0.1, `prestige_most` 3, innovation 1 %). Smokes forced two corrections: a part rebuilt after it gave way is made anew, and pots keep only what lies under a roof (NUDGE: storage pots).

**Evidence:**

- Every completed ten-year smoke of the two slices passed all ten worlds. The clay-pit step's first smoke was stopped at its fourth coast world: pots were reckoned to keep grain lying in the open, households spent their hours making them, and two of nine went without a roof from year 2 to year 6. Pots were changed before the recorded run.
- Slice Q's first smoke passed, but coast 2's buildings gave way 46 times, killing 6: rafters that gave way were rebuilt as poorly as before. With rebuilt parts made anew, one building gave way across all ten worlds.
- After the oven step, every village had daub pits, five dug 1.3 to 42 t at deposits, and six kept ten or more people (237 people in the ten worlds).
- The demo lived river-valley seeds 2, 4 and 5 (bands of 40 on 768-cell maps) 25 years each, the observer bringing jointed framing to one founder. Drying and smoking was found once in 75 village-years and lost the same year. Framing passed only to children at home, was never practised, and died with its last knower in all three. No village built a frame, so no loft was loaded; slice P's test shows loft failure and over-building. All 16 buildings begun after the first year followed an admired one. All three villages grew to 52–59 people, then emptied within a year.
- M3b closed at saves schema 22, wire 1.22 and content API 22.

**Open:** NUDGEs record that no village builds a frame and that villages of about 55 leave together. The same day a time-boxed probe of river valley 1 left its falling yields unexplained. Then, designing M3c, a recomputation of the kernel's climate draws (a check outside the repository) lined each seed's low years up with every village failure the log recorded: the yearly climate factor was keyed by seed and year, without the landscape, and households met those fixed lean years with thin reserves.

## 2026-10-05 — M3c design: speeds, the dashboard, weather and soil

**Starting point:** M3b complete (`aff6dcc`). The yearly climate factor, which scaled every harvest and the wild plants, was drawn from `[seed, climate, year]`, so both landscapes of a seed lived through the same lean years in every run (seed 1: 0.71 in year 5 and 0.42 in year 7; seed 4: five lean years running, in years 6 to 10). A band of 40 lived a simulated year in about 14 s, so fifty-year runs of five worlds would take hours.

**Goal:** plan M3c, Seasons and time: weather and seasons, soils and fertility, an Accelerated mode with a consistency test, and plan §4.7's dashboard of five worlds over fifty years. Its demo is a dry and a wet year in one village, then fifty Accelerated years that pass the dashboard.

**Implementation:** design only (`8a14ca6`), drawing on twelve research reports: [ADR-0011](../../decisions/0011-execution-modes.md), [ADR-0012](../../decisions/0012-weather-and-soil.md) and five slices in [plan §7](../../PROJECT_PLAN.md#7-milestones): S (speed and the day step), T (the dashboard), U (weather and seasons), V (soils and fertility) and W (Accelerated mode in earnest, tuning and the demo).

**Decision:**

- Order: speed first, so long runs are affordable; the dashboard second, so a baseline is logged before anything is tuned; then weather, which fixes the keying; then soils, which need the water balance; then tuning and the demo.
- ADR-0011 keeps one authoritative model at every speed. Accelerated mode (60×, 600×, Max) lives a day at a time through the same events and decisions, and stops, publishes, saves and switches mode only at midnight. Approximations must be declared, switchable in tests and covered by a statistical test. Gate A (exact equivalence) runs on every push and Gate B (statistical, its tolerances fixed before results are seen) nightly; plan §4.4's "statistically in one daily step" was amended to match. Rejected: a statistical day budget (about 3–10× faster, but a second rule set for every behaviour in M4–M8) and Detailed mode alone, unpaced (1.3–2×, short of the budgets).
- ADR-0012 draws weather daily from `[seed, weather, landscape, day]`, lets each field's water set its harvest, keeps two pools of soil nitrogen per field (slice V), and lets people plan only from what they can see. Villages may then fail in other years, or more often; the weather is never tuned to save them.
- Ditches and farm terraces moved to M6. The dashboard grades the Gini of goods amber from 0.1 to 0.3 for hoe farmers (research 08-14 §3.1 gives 0.27 ± 0.03 for horticultural populations).

**Evidence:** no run; design inputs only. A kernel profile behind ADR-0011 put about 93 % of the time in `Population::decide`, rebuilding household and settlement facts at every decision. The climate-factor recomputation is recorded in [plan §9](../../PROJECT_PLAN.md#9-decisions-log) (*Villages fail in their seed's bad years*).

**Open:** Gate B arrives with the first approximation, in slice W. M8's 50,000 agents may need statistical samplers, one subsystem at a time.

## 2026-10-05 — M3c slice S: exact speed work, saves 23 and the Accelerated speeds

**Starting point:** the M3c design (`8a14ca6`), on M3b's saves schema 22 and wire 1.22.

**Goal:** make long runs cheaper without changing any history, add the Accelerated speeds, and show both exact (Gate A).

**Implementation:**

- Exact speed work (`52a829d`): the population's indexes and caches use the engine's fast hasher; an activity's tools are listed without allocating; the search over gathering places stops once no farther place could beat the best found; and the travel field skips cells already final ([`civ-world/src/nav.rs`](../../kernel/crates/civ-world/src/nav.rs)).
- Saves schema 23 (`21397ef`) keeps worn ground exactly, each tile at its own day, with the routing view people plan on ([`civ-land/src/paths.rs`](../../kernel/crates/civ-land/src/paths.rs)). Before, a world saved, loaded and lived on parted within a day from one lived straight through.
- The day step (`ea3d628`): an advance stops after the boundaries of the minute it stops at and before that minute's events. At 60×, 600× and Max the host lives whole days; the clock stops, publishes and saves only at midnight, where a change of mode takes effect. Wire 1.23 carries the clock's mode and an infinite speed, and the observer offers six speeds.

**Decision:** exactness first ([ADR-0011](../../decisions/0011-execution-modes.md) §4): no approximation is declared yet, so a day lived at Max is exactly a day lived by the minute. A test that failed one run in 24 was run over sixty worlds with fixed identities (a new test hook, `Sim::create_for_tests`): worn daub was mended in 32 to 66 days, because mending waits for spring's fieldwork by design. The test now allows 120 days; the behaviour is unchanged.

**Evidence:**

- From one save each (seed 2 with 40 people over 90 days, seed 4 with 125 people over 60), every save section came out byte for byte as before, at 11.1 s a simulated year against 14.9 s, and 30.9 s against 37.6 s.
- Gate A's tests ([`civ-sim/tests/modes.rs`](../../kernel/crates/civ-sim/tests/modes.rs)): a world saved at a midnight, loaded and lived 29 more days ends in the same bytes as one lived straight through; thirty days lived at once, a day at a time or cut every 433 minutes end the same; three days at 10×, ten at Max and three at 10× end where sixteen at 10× do; and a world saved at an Accelerated midnight lives on as if never saved.

**Open:** Max is only as fast as the machine: about 11 s a simulated year for a band of 40 where it was measured. Accelerated frames show nobody on a trip. A world loaded within a month of its making shows no trails until its first survey.

## 2026-10-05 — M3c slice T: the fifty-year dashboard and its baseline

**Starting point:** slice S (`ea3d628`; saves schema 23, wire 1.23): a day lived at Max is exactly a day lived by the minute.

**Goal:** plan §4.7's sanity dashboard of five worlds over fifty years, and a baseline logged before anything is tuned.

**Implementation:** `civ-host dashboard` (`3dbc311`, [`civ-host/src/dashboard.rs`](../../kernel/crates/civ-host/src/dashboard.rs)) lives five river-valley worlds (seeds 1 to 5, the content's property regimes in turn, 1,024-cell maps) fifty years each, four at a time on four cores: the first month by the minute with the smoke's checks, then a day at a time at Max. Each year's end runs the checks every long run makes, now shared with the smoke (`smoke::LongRun`), and any failed check fails the dashboard. It grades population, food prices, the Gini of goods, firm sizes and failures of lived-in buildings; settlement sizes, crime, epidemics and regime labels are grey with their reasons and never count as passes. It writes a JSON report (`--json`) and keeps decade-end saves (`--keep-saves`). A nightly job in [`nightly.yml`](../../.github/workflows/nightly.yml) runs it and keeps the report.

**Decision:** thresholds only, most of them tuning values: at least 3 of 5 worlds keep 10 people at the end, and no band grows beyond max(3, 1.05^y) times its founders by year y; at most 2 failures of lived-in buildings per 1,000 lived building-years (research 11-06 §2.4's top scenario doubled); the Gini of goods amber from 0.1 to 0.3, as hoe farmers may be. Five worlds and fifty years are fixed, and a failing run is not rerun until it passes. The run is too long for pull requests until Accelerated mode is faster than Detailed.

**Evidence:** the baseline (slice T's build, still with the yearly climate factor) took 44 minutes on four cores and FAILED on two rows.

- **Population, red:** two of five worlds kept ten people, both under village fields: seed 2 (59, 46, 56, 52 and 48 people at years 10 to 50) and seed 4 (45, 49, 60, 72 and 78). The household-field worlds fell in their seeds' lean years, seed 1 to 2 people in year 6, seed 5 to 5 in year 13 and seed 3 to 9 in year 13, and lived on as 4 to 11 people. That both survivors are under village fields is confounded with their seeds.
- **Structural failures, red:** 4 failures of lived-in buildings in 1,846 lived building-years, 2.17 per 1,000 against at most 2. Buildings nobody lived in gave way 64 times.
- Food prices and firm sizes were green; the Gini was amber (0.27 in seed 2, 0.33 in seed 4); settlement sizes, crime, epidemics and regimes were grey.

**Open:** nothing was tuned against the baseline: weather (slice U) changes which years are lean, and soils (V) and the household view (W) come before any tuning. The statistical consistency test (Gate B) comes with the first approximation, in slice W.

## 2026-10-07 — M3c slice U: weather, the field's water, workable days and what the weather does

**Starting point:** slices S and T (saves schema 23, wire 1.23, content API 22), with a dashboard baseline that failed under the yearly climate factor. The steps below landed from 2026-10-05; the last four, wire 1.25, the weather in the chronicle, storms and snow on roofs and grain asks that answer stores, on 2026-10-07.

**Goal:** replace the yearly climate factor with a daily weather series per landscape that drives harvests, wild plant food, field work and wear, and show it in the observer ([ADR-0012](../../decisions/0012-weather-and-soil.md)).

**Implementation:**

- Weather in the kernel (`9ac14ab`; saves schema 24, content API 23): one daily series per world from `[seed, weather, landscape, day]` ([`civ-land/src/weather.rs`](../../kernel/crates/civ-land/src/weather.rs)). Each growing field keeps a root-zone water balance, and its harvest scales by 1 − 1.15 × (1 − water had ÷ needed), divided by the landscape's long-run mean, which is derived when a world is made or loaded and never saved. Wild plant food follows the soil water. `civ-host weather` replays a world's weather ([`civ-host/src/weather.rs`](../../kernel/crates/civ-host/src/weather.rs)).
- Wire 1.24 (`b428552`): today's weather on the clock, a `GetWeather` query behind a panel of months and years against the usual ([`web/src/weather.ts`](../../web/src/weather.ts)), and each field's water.
- A fix (`8ec687a`): a household whose axe wore out with no timber in store could never make another. [`cut_rods`](../../content/core/activity/cut_rods.toml) lets households without an axe cut thin rods and saplings with a flint flake or by hand, at a fifth of an axe's rate.
- Workable days and the farmers' view (`e98b0b8`, content API 24; narrowed in `04591f7`): snow lying, 5 mm or more of rain or snow in the day, or a daily mean below freezing keeps people from breaking, preparing and sowing ground ([`civ-agents/src/farm.rs`](../../kernel/crates/civ-agents/src/farm.rs)). Households plan spring work on the window's days that can usually be worked (81 % in the valley, 76 % on the coast), at a peak's longer hours, and judge a growing crop by its water so far.
- Wear by wetness (`7211261`): thatch, daub and the feet of posts wear as wet as each month was against its usual, from a quarter to three times the authored rate ([`civ-agents/src/condition.rs`](../../kernel/crates/civ-agents/src/condition.rs)).
- Wire 1.25 (`5bdef9c`): the kernel reports the snow line, and the observer shades the map's land for the season, a dry spell and snow ([`web/src/map/shade.ts`](../../web/src/map/shade.ts)). The tints are the observer's; the kernel supplies the month, the soil water and the snow line.
- The weather in the chronicle (`8f4ac31`, saves schema 25): on the first of each month the chronicle notes, in the kernel's words, a month whose rain and snow reached twice its usual or fell to a quarter of it, or whose mean was 3 °C or more from its normal; on the first of May a winter whose snow lay on 1.75 times its usual days or on a quarter of them; and on the first of January a year with 1.3 times its usual rain and snow or 0.7 of it ([`civ-land/src/weather.rs`](../../kernel/crates/civ-land/src/weather.rs)). `civ-host weather` lists the same notes for a seed.
- Storms and snow on roofs (content API 25): the monthly draw for each settlement that stood in for weather is retired. Once a month a storm crosses the world's landscape, one draw from the weather's key on one day of the month, log-normal about 0.25 kPa on a roof's plan (the stand-in's figures, now wind alone); and each roof carries the snow lying at its building's height, of which it keeps 0.8 when pitched at 30° or less, falling to none at 60° (tuning values; [`civ-agents/src/population/loads.rs`](../../kernel/crates/civ-agents/src/population/loads.rs)). A roof that gives way under more snow than storm is told of as giving way "under snow".
- Grain asks that answer stores (content API 26): a household's ask for food is its cost and margin times exp(−0.5 × s), `s` the years of its own need it can spare, at most one (research 08-04 §1.2: modest inventory feedback; the 0.5 a tuning value), so asks fall as good harvests fill the stores and rise after lean ones ([`civ-agents/src/population/market.rs`](../../kernel/crates/civ-agents/src/population/market.rs)).

**Decision:** dividing by the long-run mean keeps the content's yield as the average year's, so year-to-year variance is not counted twice. People see only the season so far, never future draws. Workable days first held back all field work out of doors; after the failed smoke below, a probe found that holding back weeding and reaping did most of the harm, and the model has no wet grain to make a wet harvest day cost more than the time lost, so only work that turns the soil now waits. In the first build, planning spring work at ordinary hours made coast 4's households leave in year 3 under both identities probed, so plans count on peak hours (10 a day against 6, research 04-02 §2.4; NUDGE). Emmer's authored loss for late sowing is now counted twice in part (NUDGE), left for slice W's tuning. The chronicle says what the weather was, never what it did (research 15-02 §1.2), and judges snow by the whole winter, as in deep winter snow lies all month or hardly at all.

**Evidence:**

- 500 years of the valley's weather: 803 mm a year (CV 0.22) on 139 wet days, and a harvest factor of mean 1.01 and CV 0.22 (research 08-02 §7.3: 0.20–0.35), below 0.8 in 21 % of years and below 0.6 in 3.6 % (the retired factor: 21 % and 5.5 %).
- Ten-year smoke at the weather commit: 9 of 10 worlds passed. Coast 4 failed the roof check, two of its households roofless from year 2, which a probe traced to the axe; after the fix, coast 4 roofed all nine households in its first year under both identities probed.
- Ten-year smoke at the workable-days commit: FAILED. River valley 4 sowed no field in its first 30 days (snow lay on its valley floor on 65 days of its first year), only three bands kept ten people against eight in the weather commit's smoke, and CI's month-long smoke failed on river valley 4 too.
- With workable days narrowed, all ten passed, at the edge: 5 of 10 bands kept ten people, the fewest the check allows. Coast 3, coast 5, river valley 3 and river valley 5 died out and river valley 1 ended with 3: 202 people in the ten worlds against 263 before workable days. The smoke's first-month check now asks for a sown field only where the ground could be worked on at least 15 of the first 30 days, 3 of them in the sowing window (`smoke::WORKABLE_DAYS`); river valley 4 had 11.
- Wear by wetness: over 300 years of the valley a year's wear runs from 0.59 to 1.65 of an average year's (0.51 to 1.71 on the coast).
- The chronicle's weather: over 500 years of the valley, 543 months (9 %), 69 winters (14 %) and 83 years (17 %) stood out, about 1.4 entries a year.
- Storms and snow on roofs: the year's deepest snow on the valley floor holds a median 33 mm of water and 108 mm one year in a hundred, so a hut's roof carries about 0.1 to 0.4 kPa of snow. The ten-year smoke passed all ten worlds, again with 5 of 10 bands keeping ten people (227 people in the ten worlds); two buildings gave way, killing nobody, against one before.
- Grain asks: in a probe of river valley 2 (two fixed identities, eight years) the ask stood at 1.845 hours a kg after every harvest without the term; with it, it fell from about 1.8 to 1.12 as good harvests filled the stores and, under one identity, rose after the leaner harvests of years 4 and 8. The ten-year smoke passed all ten worlds; grain was asked 6 to 16 log points more before the harvest than after, against 3 to 4 before (research 16-01 §2.2: 17-61 where markets also carry credit and risk).

**Open:** slice U is complete. The dashboard has not been run since its baseline, so how the weather moves its rows (population, the failures of lived-in buildings, food prices) is not yet measured. Survival stays at the edge: in every ten-year smoke of the slice's later steps, 5 of 10 bands kept ten people. Workable days cost the most fragile worlds. Per [plan §9](../../PROJECT_PLAN.md#9-decisions-log), survival is the dashboard's to answer: households still plan their fields on the content's yield rather than on what their fields have given, which slice V's field records change, and slice W tunes. The log records no dashboard run since the baseline.

## 2026-10-07 — M3c slice V: the soil remembers, households plan from their fields, and middens

**Starting point:** slice U complete (saves schema 25, wire 1.25, content API 26). Survival at the edge: in slice U's later ten-year smokes 5 of 10 bands kept ten people, the fewest the check allows, and `main`'s nightly fifty-year dashboard failed its population row.

**Goal:** soils and fertility ([ADR-0012](../../decisions/0012-weather-and-soil.md) §3-4): each field keeps its soil and its record, a harvest is the least of what water and soil allow, and households plan from expected yields, rest fields they can spare, break ground outside the sowing season and break long-rested fields again; middens, and manuring as a technique.

**Implementation:**

- The soil (`f51f1d3`; saves schema 26, content API 27): two pools of organic nitrogen per field, humus at 0.02 a year and fresh organic matter at 0.3 (RothC's base rates; research 03-04 §2.5), turned each first of January into the year's supply with 15 kg/ha from the air ([`civ-land/src/soil.rs`](../../kernel/crates/civ-land/src/soil.rs)). A crop takes up 0.6 of it, emmer 0.035 kg for each kg of grain; a harvest is the least of the season's and the soil's allowance (03-04 §5.2); grain and straw carried home take their nitrogen off, the rest returns; a field unsown a year grows its wild cover, which brings each pool back toward native ground's. Each field keeps its last eight harvests and what held each back. `civ-host run` reports them.
- Planning from the fields (`269edf3`, content API 28): expected yields from each field's last harvests, need as grain, the best fields cropped and the rest rested, new ground broken out of season, and ground left unsown three whole years grown over and broken again ([`civ-agents/src/farm.rs`](../../kernel/crates/civ-agents/src/farm.rs), [`civ-land/src/fields.rs`](../../kernel/crates/civ-land/src/fields.rs)).
- Middens and manuring (saves schema 27, content API 29): each household's midden grows with its members (0.5 kg a person a day holding 1 kg of nitrogen a year, below what their grain holds; research 12-02 §2.3) and wastes at a one-year half-life. Founders bring `core:technique/manuring` (07-04 §4.4), which gates carrying it in 25 kg loads to fields that have given less than new ground of their kind, the nearest first; a tenth of its nitrogen reaches the year's crop and the rest the pools (03-04 §2.3). The observer tells each field's last harvest and whether its soil held it back.

**Decision:** the model is mass-balanced and its tests check it: nothing makes nitrogen, so continuous cropping falls over decades toward what the air keeps (about 260 kg/ha), never to nothing (03-04 §4.1). People see records, never stocks. Native ground allows about 1,660 kg/ha, so the soil holds nothing back in a new village's first years and matters over decades, as the research's long experiments do. Nothing about outcomes was tuned: the second step implements the plan's own households.

**Evidence:**

- The soil's step: a ten-year smoke FAILED by one band (4 of 9 bands kept ten people; river valley 1 failed its first month on firewood). Neither was the soil's: nothing in the first month touches it, river valley 1's first month passed in eight more runs, and ten years of coast 1 and river valley 5 gave no harvest held back by the soil. CI's month-long smoke passed.
- Planning from the fields: the ten-year smoke passed all ten worlds and all ten bands kept ten people (468 people against 215 to 227 in slice U's smokes). Breaking in the autumn freed the spring: households held and cropped more ground (river valley 5: 115 fields in year 10, about 0.48 ha a person), grain about 20 to 35 % above need. The soil began to tell: supply at 77 % of native ground's by year 10, and 4 harvests held back by it in year 9. Three buildings gave way, each killing one, against one or two before.

**Open:** the fifty-year dashboard with soils: it shows whether the larger villages stay within the population row's growth bound, and how the soil's drawdown and the failures of loaded buildings move over fifty years.

## 2026-10-07 — M3c slice W: Accelerated mode's approximations and Gate B

**Starting point:** slice V complete (saves schema 27, content API 29). A day at Max was exactly a day by the minute, so the fifty-year dashboard took 33 to 76 minutes a world.

**Goal:** Accelerated in earnest ([ADR-0011](../../decisions/0011-execution-modes.md) §4-5): the household view and leisure blocks, switchable, under the statistical consistency test (Gate B).

**Implementation:**

- A profile of a village lived twenty years (65 people, 1024 cells) found most of the time outside the household's own choices. Rebuilding a settlement's walking times, a Dijkstra over the whole map, took 24 %, planning routes 12 %, the day's expected yields 8.5 %, the best purchase 8 %. The exact part: the monthly refresh of walking times now reuses them when the paths' survey and the hearth are unchanged (6 % faster, every digest unchanged).
- `Approximations` in `Ctx`, on in Accelerated mode, each switchable (`Sim::set_approximations`) ([`civ-agents/src/population.rs`](../../kernel/crates/civ-agents/src/population.rs)). The household view keeps its options' cells from a household's first decision after midnight until midnight or its own consequential step. Leisure blocks run a geometric number of sessions ([`decide::leisure_block`](../../kernel/crates/civ-agents/src/decide.rs)), cut at the next turn of the day; one that ends by a draw makes the next decision pass that leisure over.
- Gate B as `civ-host consistency` ([`civ-host/src/consistency.rs`](../../kernel/crates/civ-host/src/consistency.rs)). Two fixtures with fixed identities are each lived to 1 January of year 3. Five runs in each mode follow, each with a redrawn tie-break stream (`Sim::redraw_tiebreak_for_tests`), checked exactly as every long run is. Ten aggregates come from new time-use counters (`Population::time_use`, not saved) and the year's end. The nightly workflow runs it.

**Decision:** tolerances were fixed from a Detailed calibration before Accelerated mode was evaluated. Each is three standard errors of a difference of means at the larger fixture's spread, rounded up to a whole percent, with floors (PROJECT_PLAN §9).

**Evidence:**

- Gate B's first evaluation FAILED on walking (−12 % and −11.5 % against 9 %). Run one approximation at a time, it pointed at the first leisure blocks, which lasted until the next boundary: a choice made afresh after each block lengthened every run of the leisure first chosen, and people stayed home rather than at the hearth.
- Redrawn as the run Detailed mode would live, the blocks brought time use within 4 %, and Gate B passed: the largest differences were walking −4.4 % and the Gini of goods −5 %. No tolerance was widened.
- A twenty-year village lives about 23 % faster at Max than by the minute (44 against 57 ms a day); the household view gives about 20 % on its own, the blocks 15 %.

- Choices that lay claim to something (new ground, a building or workshop begun, goods to buy or ask for, paid work) refresh the household's view, so no member acts on a stale option (`8093d51`); Gate B still passed.
- The dashboard passed twice, in 50 and 32 minutes (76 at the slice before): four rows green and the Gini of goods amber. Failures of lived-in buildings came in at 1.10 and 1.53 per 1,000 building-years against 2, every one a hut roof in a storm, so the time-boxed tuning found nothing to tune. In the first run a village of 95 under village fields left together after a harvest 30 % short (a §9 NUDGE).
- The M3c demo: river valley seed 2's dry year 4 and wet year 5, three runs. In the dry year the harvest halved, grain stores fell by about a sixth and grain was asked at 1.36–1.42 hours a kilogram against 1.14–1.20; nobody went short. `web/e2e/m3c-demo.spec.ts` shows the village in the observer.

**Open:** walking times and routes, about a third of the time, are untouched; they are exact work to make cheaper. M4's 1,000–2,000 people need measuring before its design settles. M3c is complete.

## 2026-10-07 — M4 design: three parts, the polity, ties and standing

**Starting point:** M3c complete (`472b64d`). Settlements hold land but no goods; nobody holds an office or remembers anybody. A village under village fields left the valley together after a short harvest, at 95 people (plan §9 NUDGE). At Max, 1,000 people lived a simulated day in 653–811 ms, about four minutes a year, growing faster than the population.

**Goal:** plan M4, Councils, law and crime: 1,000–2,000 agents, notables and deliberation, constitution primitives, the law pipeline, crime and justice without courts, factions and unrest, and the god tools that touch people's minds. Its demo is five seeds from the same start, their regimes after forty years, and one law's full history.

**Implementation:** design only, from three research briefs (governance and law; crime and order; factions and unrest) drawn from about thirty reports. M4 is split into M4a (standing and the first council), M4b (crime and order) and M4c (factions and unrest). M4a has three slices in [plan §7](../../PROJECT_PLAN.md#7-milestones): X (scale), Y (ties and standing) and Z (the first council). Its two ADRs are [ADR-0013](../../decisions/0013-polity-offices-laws.md) (the polity, its offices and its laws) and [ADR-0014](../../decisions/0014-ties-standing-notables.md) (ties, standing and notables).

**Decision:**

- Every world starts from the same custom: a gathering of adults deciding by acclamation, where anyone may propose. Offices, laws and a common store arise only from proposals people back. No population threshold or timer creates an office, and no issue carries a weight toward a policy.
- Deliberation is one level of scored moves behind a `Deliberator` trait, not HTN (amends plan §4.2).
- Standing is the esteem other people's ties hold, by domain and audience. Ties are written only by recorded acts.
- Notables are a compute tier that grants nothing, checked against every adult deliberating.
- Labels are derived and never read.
- Leaving becomes a household's scored choice.

**Evidence:** a measurement, no code. At 1,000 people, a profile put 35 % of the time in route searches, partly because the route cache clears itself whole when full and monthly; 25 % in scoring candidates; 11 % in trade's search; and 5 % in checking every exposed deposit against every walk (plan §9, *Scale at 1,000 people*).

**Open:** whether 2,000 people fit a usable speed at Max is slice X's to show. M4b and M4c get their slices when the part before them is complete.

## 2026-10-08 — M4a slices X and Y: scale, ties and standing

**Starting point:** M4 designed (`705b7b0`). A year of 1,000 people at Max took about ten minutes; nobody remembered anybody.

**Goal:** slice X, make 1,000–2,000 people usable with exact speed work only; slice Y, ties written by recorded acts, standing summed from them, notables, and the observer's view of both.

**Implementation:**

- **Slice X (`ce69c6f` and the commit after).** Route searches keep Tobler's speeds per step, keep their scores side by side and read trail factors from a dense copy of the routing view. Deposits along a walk are ruled out by its box. A deal's costs are worked out only when an offer is worth pricing. The route cache keeps two generations.
- **Slice Y, first step (`b855172`).** `civ_agents::ties`: directed ties with familiarity, warmth, evidence by domain fading toward a prior, a balance of help and a reason. They are written at gifts, wages, rent, trades, learning and hearth company. Saves 28, content API 30.
- **Slice Y, second and third steps.** `civ_agents::standing`, worked out monthly and saved. The giver choice weighs regard. Wire 1.26 adds ties in the inspector and a Standing panel.

**Decision:** the hearth draws 15 % of its company from strangers and the rest from known faces. Weighting every face present let strangers fill every draw in a village of 1,000, and 227,070 ties were let go in 30 days. Influence ranks regard over all domains with warmth, not per domain.

**Evidence:**

- **Slice X:** every save section byte for byte as before over 30 days at 1,000 and 2,000 people. 1,674 to 1,354 ms a day, and 5,256 to 4,278.
- **A year of 1,000 people:** 485 s, against a goal of 120 (route searches remain most of the cost).
- **Ties without readers:** left every other section unchanged.
- **Bands of 40 after three years:** 31–34 ties a person, every adult esteemed by someone, three notables each.
- **Checks:** the smoke, Gate B and the end-to-end suite pass.

**Open:** a tighter route search bound changes ties between equally fast routes and waits for towns. The notables' gate comes with deliberation in slice Z. Gate B does not yet compare ties.

## 2026-10-08 — M4a slice Z, first step: the polity, its gathering and a common store

**Starting point:** slice Y done (`2571a66`). People remembered one another and settlements had notables, but no settlement could decide anything together.

**Goal:** ADR-0013's polity in its first form. Every world starts from one custom. A law is proposed only when someone weighs it worth proposing, a gathering decides it, people pay or do not, the store gives to those who ask, and the whole history is kept.

**Implementation:**

- `civ_agents::polity` (pure): the body as values (adults, a quorum share, acclamation), laws with their whole history, the forecast, stances, the chance of paying, and the `Deliberator` trait with a rule-based deliberator.
- `population::polity`: founding, the weekly review, proposals, the gathering decided at midnight, word going round, the levy at threshing and relief from the store.
- The `policy` content kind and the core pack's common store. The people profile's `[polity]`. A new behaviour, `attend`. Ledger channels `levy` and `relief`, with the polity as a holder. Chronicle entries for proposals and decisions. Saves 29, content API 31.

**Decisions:**

- The issue is a shortfall against the outlook: a household whose food will not last until its harvest, or a settlement that ran short. Three seeds lived five years without a settlement-wide shortage.
- A forecast is the change in the expected log of a household's year of food above subsistence, over an ordinary and a lean year. Pooling then helps those near the edge and costs those with plenty, and no issue weighs toward any policy.
- Being unable to pay (it would leave a household below subsistence) is recorded apart from keeping a levy back.

**Evidence:**

- **Three seeds over four years:** a twentieth passed in each, once after a fifth was turned down 1–18. Gatherings drew nearly every adult, and 5–43 % of the levy owed was kept back or unpayable.
- **Ten-year smoke:** all 10 worlds passed. Every village ended with a store at a twentieth, after 1–15 proposals, and four stores gave relief. Ria coast 5, which died out in year 4 before, kept 15 people.
- **Gate B:** passed. The coast fixture's Detailed runs spread more than when its tolerances were set (work ±0.27 h against ±0.15), cause not traced.
- **Tests:** an integration test proposes, gathers, decides and saves exactly. Another puts a law in force by hand, then sees the levy paid at threshing, every good accounted for, and asks answered from the store.
- **Checks:** the workspace tests and clippy pass.

**Second step, the government panel:**

- **What it shows:** `GetGovernment` (wire 1.27) carries each polity's custom, store and every law with its whole history, sentences rendered by the kernel. A Government panel shows it.
- **Evidence:** the end-to-end suite passes with a new government test.
- **Two findings:** in the one run whose stances were read in full, the law passed mostly on regard for its sponsor, and two worlds of one seed differ in their details because each draws its own identity.

**Third step, leaving weighed:**

- **What changed:** a household out of food and worn down still leaves only with no harvest near, but now at a chance scaled by what staying offers: its own food, its share of what neighbours could spare, and a common store it knows of, against the wait to its next harvest, and the year's food its fields should bring. The chance never exceeds the old one (content API 32).
- **Evidence:** a unit test of the weighing; the ten-year smoke passed. A coast village whose store held a few hundred kilograms still lost half its people in a poor year: there was little to wait on.

**Fourth step, the storekeeper:**

- **What changed:** an unkept store with food in it is an issue. A sponsor may propose the sheltered adult they regard most as its keeper, and the gathering decides as for any law. Under a keeper the store spoils as a roofed store and relief is asked at their home. When the keeper dies or leaves, the law lapses and a successor needs a new law (saves 30, wire 1.28, content API 33).
- **A deviation, recorded:** ADR-0013's general office (seats, removal, pay, powers) is not built; the first office is a law naming its holder.
- **Evidence:** an integration test puts a keeper law in force by hand, sees the store keep under the keeper's roof, moves the keeper's household away and sees the law lapse in the chronicle; the ten-year smoke passed with a keeper in every village and spoilage down from 32–40 % to 11–18 %; one coast run showed a keeper gone and a successor named; Gate B and the end-to-end suite passed.

**Found on the way:** the fifty-year dashboard failed one row, structural failures at 2.01 per 1,000 building-years against 2, within the spread of earlier runs (1.10–2.37). Its roof failures killed people where slice W's had killed nobody. A time-boxed probe showed the collapse rule unchanged: slice W's failures were mostly huts their households had left.

**Fifth step, labels:**

- **What changed:** a pure classifier works out what each polity would be called from its saved history: who may decide, what it decided in the last two years and how many came, whether one person's following carries what passes, its offices and whether one outlived its holder, and whether its levy is paid. It gives a name, modifiers, a confidence and reasons in words, shown in the Government panel (wire 1.29), the run report and the smoke. Nothing is saved.
- **Evidence:** table-driven tests over the classifier, including cases from the research (twice the people under the same institutions changes nothing; a body admitting a third of the adults is an oligarchy). An integration test labels a village every day on one copy of a save and not on another, and both end byte for byte the same. The end-to-end suite opens a label to its reasons.
- **Found on the way:** the first ten-year smoke labelled every village at the lowest confidence, because nothing had been decided in the last two years, and the classifier would have called a custom exercised years before "not yet exercised". Laws still in force now count as evidence that the procedure binds.

**Sixth step, the notables' gate:**

- **What changed:** the notables' tier became a run setting. With it, the notables and those something reached since the last review (a household running short, a gathering they came to, a law they learnt) weigh institutional moves; without it, every adult does. `civ-host notables` lives two fixture worlds three years, five times each way, and compares laws proposed, laws passed, offices filled and the share of food the polity moved, against tolerances fixed from a calibration with the tier. It runs nightly beside Gate B.
- **A failure, and its cause:** the first gate failed on offices filled (1.6 without the tier against 1.0 with it). The runs showed the tier changing who was named keeper, and with it whether the office had to pass to a successor. The tier had been built with only one of ADR-0014 §4's three ways a non-notable is reached. Built as the ADR specifies, recalibrated by the same rule, the gate passes. The cost of every adult deliberating at a village's size was nil.

**The M4a demo, and what it found:**

- **The demo:** river valley seed 2 lived five years twice from the command line and once in the observer (`web/e2e/m4a-demo.spec.ts`). In each run a gathering passed a common store at a twentieth in the lean weeks before the first harvest and named a keeper within a year; the levy was mostly paid; and in the dry year 4 the harvest halved, nobody went short, and the store gave no relief, because households' own stores carried them. The observer shows a law's whole history, the notables and the label with its reasons.
- **Found by the demo:** its first observer run voted 27 times on a fifth or a tenth before a twentieth passed, one sponsor proposing 14 of them; nobody learnt anything from a vote they had watched. Now one who came to a gathering expects no more support for the same proposal than they saw it get, for a year (content API 34). The wording of a self-nomination ("proposed themselves as keeper") was fixed too.

**M4a closed:** the fifty-year dashboard passed on the last build, four rows green and the Gini of goods amber; every world kept its band (77–94 people at year 50) and ended a council community whose storekeeper's office had outlived a holder. Carried forward: office pay and land tenure as a law, a general office primitive, slice X's speed goal, and word of a gathering spreading by mouth.

**Open:** whether a larger store would keep a village through a poor year is untested; villages chose a twentieth or a tenth. M4b, crime and order, is next.

## 2026-10-08 — M4b design, and slice AA: theft and what was seen

**Goal:** begin crime and order with the part every later slice stands on: taking as a person's choice, what happened kept apart from what people believe of it, and the first thing one household can owe another.

**Design:** from the M4 crime brief, its citations checked against the criminology, policing, justice and corruption reports. One ADR, [ADR-0015](../../decisions/0015-incidents-cases-obligations.md): incidents (the kernel's truth), cases (what a polity knows) and beliefs (what people know) are kept and saved apart, and no choice reads an incident; taking is a scored choice behind a moral filter; every sanction is an obligation. Four slices, AA to AD ([plan §7](../../PROJECT_PLAN.md#7-milestones)). The briefs are now kept in `docs/briefs/`.

**Slice AA, what changed:**

- **Taking is a choice.** Someone whose household is short may go to another household's home to take food, weighed like asking: the food a load would carry against the walk, their objection to taking, the chance they believe a taker runs of being seen and their regard for those they would take from. Above a moral filter it is not weighed at all. Each person's objection is drawn at birth and pulled toward their parents'; the chance of being seen moves with what they try and hear.
- **Arrival settles guardianship.** Someone old enough to stop them at home, or anyone they notice about, turns them back; a sleeper may wake; those they miss, and children, see them. One who turned back waits a while before trying again.
- **Truth and belief.** Every attempt is an incident only the kernel reads. People believe what they saw, were told or found missing, each belief with the witness it began with. A household finds a loss at midnight; who took travels by household and hearth, a witness telling those taken from unless they regard the taker more.
- **What follows.** A household refuses the asks of someone it believes took from it or from those it regards. One that learns who took may demand the food back; the taker's household pays from what it can spare, refuses, or leaves an arrear. The ledger gains `take` and `restitution`; the chronicle tells what was seen plainly; a Takings panel shows what happened beside what people believe.
- **Boundary:** saves schema 31, wire 1.30, content API 35.

**Found on the way:** the first lean-spell run had takers turn back only for a member at home, so evening takings were "seen" by most of the village from the hearth, and the same few people went back to a guarded store dozens of times a day. The policing report's guardianship rules (an offender notices and abandons, or changes their timing) and capable guardians fixed it; a fixed wait then put every retry at dawn or dusk, when households are up, so the wait is drawn.

**Evidence:** unit tests for the objection draw, the belief rules and the choices' points; an integration test of a well-fed village (no takings) and of a test's shared shortage (the moral filter holds, goods are conserved, beliefs trace to witnesses, a demand meant to be paid is paid, an exact save and load); the end-to-end suite reads the Takings panel. The ten-year smoke passed all 10 worlds: seven never tried to take, and on the coast one village saw five takings, each seen, demanded back and four of them paid. Gate B and the notables' gate passed.

**Open:** fed villages scarcely take; whether a lean year in a village of 1,000 makes takings common, or only attempts, is untested. Cases and their decision (AB) are next.

## 2026-10-08 — M4b slice AB: cases and their decision

**Goal:** give a village a way past a demand: a law against taking, a case brought before the gathering, a decision kept whole, and what a finding imposes as obligations, with exile.

**What changed:**

- **A law against taking.** A new policy template whose sanction bundles people propose: compensation to the household taken from and a fine to the common store, in days of the taker's household's food, and exile. Each household weighs it by what it knows: what it would recover of what it lost to takers it knows of, less what its own takers would owe at the chance it believes they are seen. Households taken from since the last review may propose it. So that nothing here reads what happened, a taker now knows what they took, and a household knows what it found missing or brought home.
- **Bringing a case.** Under such a law, known to the household's chooser, a household that learns who took from it chooses among letting it go, a demand and a case: what it would recover, times the chance it believes the gathering would find from its distinct witnesses, less the cost of bringing it. A household whose demand fails may bring a case after.
- **The hearing.** The gathering hears cases as it decides laws, and may be called for cases alone. Each who came stands by what they believe or the accounts told there, what their household stands to gain or lose, and their regard for each party; the body's rule decides, and the decision is kept whole and never repaired.
- **What a finding imposes.** Restitution, compensation and a fine to the store, obligations answered once and whole and paid from what the household can spare (`compensation` and `fine` channels); exile sends the one found from the valley, recorded as a leaving.
- **Boundary:** saves schema 32 (saves 31 still load), wire 1.31 (the case beside what happened and what people believe; a gathering's cases), content API 36.

**Found on the way:** the hearing first used the common store's forecast for each household's stake, which is flat below a year's subsistence, so in a lean spell nobody stood to gain or lose by a finding; it now counts days of food. The slice AA test failed about one run in 24 because a lean spell can pass 60 days without a single taking; both crime tests now set their situation explicitly. A first draft of the hearing's chronicle said a household was sent from the valley.

**Evidence:** unit tests for the bundle's words, the forecast's signs (those taken from gain, takers lose, exile weighs only on takers) and the content checks; an integration test that lives a law against taking through takings, cases, hearings, findings, exile and paying, with goods conserved and an exact save and load (12 runs of 12). 548 kernel tests, 130 web tests and the end-to-end suite (16 passed) pass. The ten-year smoke passed all 10 worlds three times; in the third, three coast villages passed a law against taking after a seen taking, and in two of them a case was brought, found and what it imposed paid. No world chose exile. Gate B and the notables' gate passed, the latter with M4a's figures.

**Open:** whether exile is ever chosen, and how often cases come where takings are common: fed villages scarcely take, and their takers are rarely seen. No office hears cases yet, labour-days wait for polity work, and refusing what a finding imposes has no sanction beyond its arrear until a watch exists (AC, next).

## 2026-10-08 — M4b slice AC: the watch and corruption v0

**Goal:** a village's own guard, built from what already exists: an office named by a law, a guardian who acts only where they stand, and the first corrupt act as a person's choice rather than a rate.

**What changed:**

- **The watch is an office.** A `keep_watch` law names its holder as the storekeeper's does; a household weighs it by a share of everything it found missing against as much of what its own takers took. A law now carries its template's kind, so offices are told apart without the catalog, and any office lapses when its holder dies or leaves.
- **Rounds are a choice.** The one named walks rounds of the settlement's homes at night, a stand at each, worth less with each round, until sleep wins. Standing there they are someone about: the guardianship of slice AA turns a taker back or makes them seen, and nothing else makes the watch work. Its rounds, hours and cases show in the Government panel.
- **What a watcher does with what they saw.** Once per taking: bring it before the gathering (or tell those taken from), say nothing, or ask the taker's household for food to say nothing, weighed against the watcher's objection to taking and the chance they believe they run of being found out. A household asked pays, or is brought before the gathering. A payment moves on a `bribe` channel; a quiet watcher tells nobody; what they chose shows only as truth.
- **Boundary:** saves schema 33, wire 1.32, content API 37.

**Found on the way:** a test watcher with no objection to taking also took, once their household had fed its hungry neighbours, and was found and exiled, ending the watch. A starving taker's household has nothing to pay with by the next midnight, and a watcher who has heard of many takings expects to be found out, so asking is rare even without objection.

**Evidence:** unit tests for the watch's words, forecast and content checks; integration tests that a watch walks its rounds within the night's limit, that a watcher chooses once and keeps quiet or brings a case accordingly, and that a payment moves on its channel with goods conserved (10 runs of 10). 553 kernel tests, 130 web tests and the end-to-end suite (16 passed) pass. The ten-year smoke passed all 10 worlds; one village named a watch, which walked about two hours a night for the decade and brought the one taking it saw before the gathering. Gate B and the notables' gate passed.

**Open:** whether a watch changes how often takers succeed: the smoke's villages take a few times a decade. Curfew, prohibitions, the dashboard's crime row and the M4b demo (AD) are next.

## 2026-10-08 — M4b slice AD: curfews, the crime row and the demo

**Goal:** close M4b: a prohibition in the law pipeline, plan §4.7's crime row graded by its direction, and the demo of a theft from the act to its end.

**What changed:**

- **Curfews.** A `curfew` policy people may propose against takings, at the hours its template offers. A household weighs it as a watch, at a smaller share of what it lost, less what keeping its grown members at home costs it. Under one in force, anyone who knows of it weighs keeping it against any option that would take them off their home's plot in its hours, with the levy's terms: custom, where they stood on it and their regard for its sponsor. The watch at its rounds and those at a gathering are exempt; there is no sanction, and the law counts its breaches, shown in the Government panel.
- **The crime row.** Every attempt to take records where the taker's household stood by food among its neighbours. The dashboard grades every attempt by that share, by direction only, with takings, those seen and cases beside it, ungraded; the smoke reports the share too.
- **The demo.** civ-sim's `theft_world` example sets a lean spell going (every other household loses its food) and lives it until a case has been found and settled; the M4b demo spec shows that village's takings, the case and the law. The Takings panel now keeps takings and seen attempts ahead of the many unseen attempts that turned back.
- **Boundary:** saves schema 34 (saves 33 still load), wire 1.33, content API 38.

**Found on the way:** closing a grove would be inert: fallen wood grows at a constant rate, and for game and fish no household keeps a record from which to forecast a closure, so closures are designed and not built. In the template's night hours nearly everyone is already at home asleep, so the curfew's test runs through a working day to see whether keeping it weighs. Lean spells mostly end without a case (one world in five or six), so the demo's example makes up to twelve worlds and says which it kept. A pre-existing flaky test claimed that every household keeps the seed for all its fields after the first harvest; since households may break ground for next year and rest some, the village's seed for the ground it cropped is the claim the model keeps, and the test now makes it.

**Evidence:** unit tests for the curfew's hours, being away from home, the content checks and the crime row's grading; integration tests that a known curfew keeps people in (175 samples away against 662 without) with breaches counted and an exact save and load, and that incidents carry the food measure (10 runs of 10). The kernel, web and end-to-end suites pass but for the flaky test. The ten-year smoke passed all 10 worlds: three coast villages passed a watch, a law against taking and a curfew; their takers' neighbours were richer in 0.73 of 42 attempts. The fifty-year dashboard's crime row was grey (no attempt in five valley worlds) and its structural failures red by chance (11 in 4,484 building-years). Gate B and the notables' gate passed.

**Open:** whether curfews change how often takers succeed, and the crime row's grade where the dashboard's worlds go hungry. M4c, factions and unrest, is next.

## 2026-10-08 — M4c design and slice AE: claims and grievances

**Goal:** design factions and unrest (M4c), and build its first slice: news that travels only by contact, and grievances people hold against the polity for what its acts cost them.

**Design:** from the M4 factions brief, its citations checked against the reputation, collective action, information flow, ideology, norms, revolutions and structural demography reports. Two ADRs: [ADR-0016](../../decisions/0016-grievances-claims-opinions.md) (social memory per person, sparse and sourced: grievances, shared claims with per-person hearing, opinion with anchors, god tools that touch only what can be perceived) and [ADR-0017](../../decisions/0017-factions-episodes-regime-change.md) (leader, regime and constitution told apart; factions as organizations with treasuries; attendance as a scored choice; seizure only by force or effective authority, with no roll). Six slices, AE to AJ ([plan §7](../../PROJECT_PLAN.md#7-milestones)).

**What changed:**

- **Word of mouth.** A gathering's call is a shared claim. Its sponsor, or whoever brought its cases, hears first; households tell their members at midnight and companions tell each other at the hearth, each by a keyed chance; only those who heard may come.
- **Grievances.** Each is held by one person against a party (the gathering, an office, a household), under a law whose terms it broke, with a harm in days of food, the part not yet made good, and an activation that fades and is raised only by reminders. They come from an empty store when short, a levy in a lean year that leaves a household short, a finding a household thinks wrong, a case not found or not heard, and a finding's obligation refused. Relief makes good what was held against the store; a grievance held keenly is told at the hearth.
- **The inspector** lists a person's grievances and what they have heard, with who told them, in the kernel's words.
- **Boundary:** saves schema 35 (saves 34 still load: every adult hears of a gathering already called), wire 1.34, content API 39.

**Found on the way:** the first levy grievance had no lean-year test, and the smoke showed nearly every adult in every world holding one: households rarely hold a year's food in store, so an ordinary levy counted as a wrong. The rule now needs a year the settlement's food ran short, as the polity's own issue test does. At the first `full_harm_days` (30) an empty store's harm was felt below the telling floor and could never be told; it is now 10. Draws about a gathering's call were keyed by the claim's number, which grievance claims moved; they are now keyed by the settlement and the day. News of laws and findings is not copied into claims: both already travel by contact in their own records.

**Evidence:** unit tests for claims, hearing, pruning and grievance activation; integration tests that only those who heard a gathering came and that word of it was passed on, that an empty store is held against the gathering by those who know its law and that relief makes it good, and that a levy is held against the gathering in a lean year (13 grievances) and not in an ordinary one (168 levies paid, none held); the inspector's words are read back from the wire. The kernel (563), web (136) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the schema check clean. The ten-year smoke passed all 10 worlds: 92-99 % of the members came on average and no gathering lacked a quorum; only the three hungry coast villages held grievances at the end (45 to 65 among 25 to 28 people). Gate B and the notables' gate passed.

**Open:** nothing acts on a grievance yet; whether larger villages, where word thins out, see gatherings fail for want of a quorum. Amending the custom (AF) is next.

## 2026-10-08 — M4c slice AF: amending the custom

**Goal:** let a village change who decides by its own procedure, and keep the record that says it was an amendment.

**What changed:**

- **Amendable bodies.** Membership (every adult, each household's elder, or landholders), the share who must come and the rule (more for, or two-thirds) are values a content template offers, one change at a time.
- **Who proposes, and why.** Only someone a gathering overruled within memory, or who holds a grievance against it, and only when the new rule would have decided what they saw in their household's favour: each decision is re-decided under the new body from the stances recorded there.
- **What it changes.** The present custom decides it. Once passed, only members come, stand and count; the custom keeps every version with the amendment that made it; the chronicle tells an amendment apart from a decision; the label reads the real share of adults the body admits; the Government panel lists the versions and how many may decide.
- **Boundary:** saves schema 36 (35 still loads), wire 1.35, content API 40.

**Found on the way:** with temperature 0.5 a softmax over four amendments of no gain would choose one about three times in five (worked out from the scores), and a village voted 24 to 0 for a landholders' custom that changed nothing; an amendment is now weighed only when it would have served the sponsor's household.

**Evidence:** unit tests for the two-thirds rule, the codes and the custom's history in words, and the content checks; integration tests that those a thin gathering overruled propose that the elders decide (one change, the one that would have passed what they backed), and that a passed amendment changes the body, its versions, who counts and the panel, with an exact save and load (seven runs of seven). The kernel (568), web (137) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the schema check clean. The ten-year smoke passed all 10 worlds and no village amended its custom: gatherings of about 50 are nearly unanimous. Gate B and the notables' gate passed.

**Open:** whether larger villages, with thinner gatherings, amend their customs; opinion, ideology and norms (AG) are next.

## 2026-10-08 — M4c slice AG, step one: opinion

**Goal:** give each person a view of the questions of the day that is their own, rooted in their household's lot, moved a little by the company they keep, and heard in what they say at a gathering.

**What changed:**

- **Positions.** A policy template may ask a question ("whether to keep a common store"); every adult holds a position on each, 0 against to 1 for, with a salience that is higher while a law of it is in force or before the gathering.
- **Anchors.** On the first of each month the forecast a sponsor would weigh is worked out for each household and question (now shared by the polity review and opinion), and each position is pulled toward what it makes of the policy with a two-year half-life.
- **Talk.** Companions at the hearth say where they stand now and then; a listener moves a little toward someone they regard, less the further apart they are, more when young (research 06-04's Friedkin–Johnsen priors). No negative influence yet.
- **Stances.** At a gathering, how far talk has moved someone from their household's lot weighs in their stance, and the law's record says when it did. The inspector shows where someone stands and what moved them.
- **Boundary:** saves schema 37 (36 still loads), wire 1.36, content API 41.

**Found on the way:** the first build told 14 to 36 positions a person a week, far above the report's 0.5; the chance was cut to 0.0015 and the smoke measured 0.46 to 1.85. With stances a little more spread, two worlds began amending the custom back and forth (51 and 8 amendments in ten years): slice AF's forecast had re-decided amendments, and decisions made under earlier customs, so each change gave the other side cause to undo it. It now counts only laws and cases decided under the present custom. The smoke's check of each law's body was also wrong about amended customs (it read today's body, and did not count a superseded amendment as passed); both fixed. The smoke also showed that grievances and hearing records of people who died or left are never let go: nothing acts on them, but they fill saves and the smoke's count. That is fixed separately, next.

**Evidence:** unit tests for anchoring, the pull, taking talk in and the position store; integration tests that adults hold positions anchored in their household's lot (one household, one anchor) and moved a little by talk, with an exact save and load, and that talk weighs in a stance and the record says so. The kernel (573), web (138) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the schema check clean. The ten-year smoke passed all 10 worlds: one village amended its custom once, to its elders, and none cycled; talk moved positions 0.003 to 0.007 from their anchors on average. Gate B and the notables' gate passed.

**Open:** norms, values and ideologies, the rest of slice AG.

## 2026-10-08 — M4c slice AG, step two: norms

**Goal:** turn the flat custom that "the gathering binds" into a norm each person holds their own way, and let what they believe of others come only from what others tell them.

**What changed:**

- **Norms are content.** A new kind, `norm`, names a prescription the kernel can act on; the core pack's says that what the gathering decides binds everyone.
- **Each person's state** (research 06-05): an endorsement drawn at birth and pulled toward the parents', a belief of how many households abide, and a threshold on that belief, so a principled few hold to it whatever others do and others follow the crowd.
- **What it does.** Paying a levy or keeping a curfew weighs the person's endorsement and how far others' doing it moves them, in place of the flat 1.5 points every payer weighed before.
- **What moves belief.** Only what companions at the hearth say their own household did at its last levy; a household that could not pay is not counted.
- **The inspector** says how far someone holds it, what they believe others do and whether that holds them to it.
- **Also:** grievances and hearing records of people who died or left are now let go (found by the opinion smoke).
- **Boundary:** saves schema 38 (37 still loads), wire 1.37, content API 42.

**Found on the way:** the slice AE fix went in with one line not formatted as `cargo fmt` wants; it is formatted in this step.

**Evidence:** unit tests for activation, the draws, learning and the season's count; integration tests that everyone holds a state drawn their own way, that companions' accounts move beliefs, with an exact save and load, and that a village that holds the norm and believes others pay pays a levy 0.91 of the time where one that does neither pays 0.59; content checks for the kind; the inspector's words in the web tests and the standing end-to-end spec. The kernel (580), web (139) and end-to-end (16, 6 demos skipped) suites pass, with clippy, the format check and the schema check clean. The ten-year smoke passed all 10 worlds: people believed 0.90 to 0.95 of households abide where 0.86 to 0.95 had paid, from 0.08 to 0.29 accounts a person a week; three villages amended their custom once each. Gate B and the notables' gate passed.

**Open:** values and ideologies, the rest of slice AG; the normative expectation and sanctions.

## 2026-10-08 — M4c slice AG, step three: values

**Goal:** let people weigh a law by more than what it does to their household's food, each by what they hold dear.

**What changed:**

- **Values are content.** A new kind, `value`; the core pack names three (research 06-04 §1.1): safety from want and harm, a household's say over what is its own, and giving back what was given or taken.
- **Each person** holds each between less than most and more than most, drawn at birth and like their parents', for life.
- **Templates say how a law bears** on each value (`[bears]`): a common store for safety and against a household's say, a curfew against a household's say.
- **Where it weighs:** in someone's anchor on a question, their stance at a gathering (the record says when it mattered) and how they weigh proposing a law. Nobody sees another's values.
- **States at midnight.** Everyone takes their values and norms the first midnight they are here: a three-day curfew test found that until 1 April nobody held the norm that the gathering binds, so a curfew weighed nothing in a world's first month.
- **The inspector** says what someone holds dear.
- **Boundary:** saves schema 39 (38 still loads), wire 1.38, content API 43.

**Evidence:** unit tests for the draw, heredity and points; integration tests that everyone holds each value their own way and that a village holding safety dear stands for a store nobody gains by (and against it when it does not), with the record's words and an exact save and load; content checks for the kind and for templates naming values that do not exist; the web tests and the standing end-to-end spec. The kernel (585), web (140) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the format and schema checks clean. The ten-year smoke passed all 10 worlds: values weighed in 24 to 54 % of stances. Gate B and the notables' gate passed.

**Open:** ideologies, the last of slice AG (below).

## 2026-10-08 — M4c slice AG, step four: ideologies

**Goal:** let people hold shared explanations of what goes wrong and what to do about it, carried from person to person, without the engine choosing which wins.

**What changed:**

- **Ideologies are content.** A new kind, `ideology` (research 06-04 §6.1): the problem it explains, how it tilts what its holders hold dear, the laws it proposes, its legitimacy story and how it travels. The core pack names three: common provision (food short; a store and a storekeeper), order kept by all (takings; a law against taking and a watch) and each household its own (being overruled; no program).
- **Holding one.** Some founders bring each; a child may take up a parent's; at the hearth a holder now and then speaks of one, and a listener takes it up by their trust in the teller and how well it fits what they hold dear, never when it runs against it (06-04 §1.4). The record keeps from whom.
- **What it does.** Its commitments weigh beside a holder's values on every law. Its program joins a holder's moves only when the problem it explains is before the village (06-04 §1.2). A law so proposed keeps its creed, and the Government panel says it was proposed "as one who holds to" it.
- **The inspector** lists what someone holds to, since when and from whom.
- **Boundary:** saves schema 40 (39 still loads, and everyone takes their start the next midnight), wire 1.39, content API 44.

**Two findings:** programs weighed with no problem present had holders proposing stores in villages never short of food (eleven tests broke), so programs now wait on their problem. Spreading by trust alone saturated villages, with every ideology held by 31 to 52 of about 50 people within ten years; the fit gate stopped that, and the adoption chance was calibrated to 0.3 (a design prior).

**Evidence:** unit tests for holdings, fit and the adoption chance; integration tests that founders bring ideologies and companions take them up from whom the record says, that a holder proposes what their ideology proposes and the law keeps its creed, and that a schema-39 save loads and its founders bring the same ideologies the next midnight; content checks for the kind and for unknown values and templates; the web tests and the standing end-to-end spec. The kernel (591), web (141) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the format and schema checks clean. The ten-year smoke passed all 10 worlds: each ideology held by 0 to 35 of 17 to 56 people; 15 of 29 laws were proposed by someone holding an ideology that names them, with 2 to 6 laws a village in ten years as before. The creed marks a law; it does not show the creed caused it. Gate B and the notables' gate passed.

**Slice AG is complete:** opinion, the norm that the gathering binds, values and ideologies. **Open:** slice AH, factions and episodes (ADR-0017; below).

## 2026-10-08 — M4c slice AH, step one: factions

**Goal:** let people who share a grievance organize, as their own choice, without the engine deciding who rebels or when.

**What changed:**

- **A faction** organizes those who hold grievances against one party, the gathering or an office (ADR-0017 §2; research 04-10 §1.1, §1.5). It has a founder, an organizer, members, a store on the ledger and a history.
- **Founding** needs a grievance felt keenly and the belief, from what someone has heard, that others they trust hold one against the same party too; never a count of angry people.
- **Joining, staying and leaving** are each adult's choice on a day of their own each month: their grievance, their regard for the organizer, how many of those they know belong and the dues, against a threshold of their own. Leaving needs the worth to fall well below it. Membership is kept apart from belief.
- **The store** takes dues from members' threshing up to a reserve (a new ledger channel, `dues`) and gives to a member's household short of food. The longest-standing member takes over when the organizer is gone; a faction with no members ends.
- **The inspector** says which faction someone belongs to and why; **the Government panel** lists each polity's factions.
- **Boundary:** saves schema 41 (40 still loads), wire 1.40, content API 45.

**Three findings:** unbounded dues piled up thousands of kilograms that were never given out (members' households were never short after joining), so dues now stop at a reserve; a founder's self-regard kept lone founders in factions nobody joined, and removing it made them found and quit monthly (117 foundings in one village), so a founder whose faction ended waits a year; and two coast villages that fell to 16 and 14 in one run were the draw of the world's life, not factions (the same seeds with fixed identities, factions on and off, ended within two people).

**Evidence:** unit tests for founding, belonging, the words of why someone belongs and memberships; integration tests that a shared grievance founds a faction others join for reasons the record keeps, that a member's household short of food is given from its store, and that a schema-40 save loads with no factions, each saving and loading exactly; the web tests and the standing end-to-end spec. The kernel (598), web (142) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the format and schema checks clean. The ten-year smoke passed all 10 worlds: factions formed only in the three hungry coast villages (one of 26 members in a village of 48). Gate B and the notables' gate passed.

**Open:** step two, petitions and assemblies with a demand the gathering decides; step three, refusals of a levy, damage and violence. A rare save-and-load test failure (about 1 run in 100) is narrowed to one unchosen option scoring differently after a reload, most likely a decision cache rebuilt on load; it stays open.

## 2026-10-08 — M4c slice AH, step two: petitions

**Goal:** give a faction a peaceful way to press its grievance, with coming a choice of each person's own and the answer the gathering's, never the engine's.

**What changed:**

- **Calling a petition:** at their monthly review a faction's organizer weighs calling one at the hearth once the faction has three members and nothing waits, by its members' grievance and how many of those the organizer knows belong, against a threshold of their own (ADR-0017 §3; research 09-04 §5.5, 09-05 §1.2).
- **The demand answers the party blamed** (04-10 §1.1): against the gathering, the common store at another share or none in place of the one in force; against an office, another holder in place of the one in it. A law may now name the law it replaces, and passing it supersedes that law.
- **Coming** is a scored activity for those who heard of it, on the evening it sits: their grievance, belonging or regard for the organizer, and how many of those they know belong, with free-riding for some (04-10 §1.4, §5.3).
- **The answer:** the organizer puts the demand to the gathering, which decides it as any law; for a new officeholder, regard for the one in office weighs against it, lowered by a grievance against them. A petition turned down is a new grievance to each who came (04-10 §3).
- **The Government panel** lists each polity's petitions in the kernel's words; the smoke reports them.
- **Boundary:** saves schema 42 (41 still loads), wire 1.41, content API 46.

**Two findings:** the first build let only factions against the gathering petition, and no petition was ever called, because every faction the smoke formed was against the storekeeper (an empty store is held against its keeper); the demand now follows the party blamed. Then petitioners came and abstained at the gathering (13 of one village's 15 petitions split one to one), because nothing in their stance held their grievance; a grievance against an office now lowers regard for its holder in that decision. Also fixed: a keeper no longer blames their own keeping.

**Evidence:** unit tests for coming, the stance between a holder and a replacement, and the words of a replacing law; integration tests that a faction against the gathering petitions it (43 heard of it and 26 came; the gathering ended the store's levy, 26 for and none against) and that one against the keeper petitions for another (the keeper defending their place), each saving and loading exactly, and that a schema-41 save loads; the web tests. The kernel (602), web (142) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the format and schema checks clean. The ten-year smoke passed all 10 worlds: petitions in two of the three coast villages with factions, all for another keeper (22 came to one, granted; 16 and 17 to two, one granted and one without a quorum). Gate B and the notables' gate passed.

**Open:** step three, refusals of a levy, damage and violence. The save-and-load flake from step one stays open.

## 2026-10-08 — M4c slice AH, step three: refusals of a levy

**Goal:** give a faction a second way to press its grievance where it cannot petition, without the engine deciding who resists.

**What changed:**

- **Calling a refusal:** where no petition is called, an organizer whose members blame the gathering or the store's keeper may call on them to keep back the store's levy together, openly, for a year (ADR-0017 §3; research 09-04 §5.5, 04-10 §1.9). It is worth what a petition is, less what the norms the organizer holds weigh for abiding by what the gathering decided, so a petition is always preferred where one can be called.
- **Joining** is each thresher's choice at their own threshing, for those who heard of it, with the same terms as coming to a petition; a law now counts levies kept back in a refusal apart from those kept back unannounced.
- **Word** of a refusal travels as a petition's does; the Government panel lists refusals and the smoke reports them.
- **Damage and violence move to slice AI.** Violence answers encounters (04-10 §1.8), and no one in these villages yet collects, disperses or represses; built now, it would never fire or would fire without cause.
- **Boundary:** saves schema 43 (42 still loads), wire 1.42, content API 47.

**Findings:** the first prior made a refusal cost twice what a petition does, and even with no norm held it could hardly ever be called; the cost is now a petition's, the norm being what sets them apart. One coast village ended between 16 and 49 people in the runs since keeper petitions began, against 46 to 53 before; lived from four fixed identities with petitions on and off, it ended alike (42 and 39, 45 and 45, 33 and 30, 17 and 17), so the spread is the identity's draw. One smoke run failed a coast world's first-month water check, the rare gap recorded before; nothing this step changed runs that early, and the rerun passed.

**Evidence:** an integration test in which a faction that petitioned lately calls a refusal and, at the harvest, 29 people keep back 355 kg under it, saving and loading exactly; a test that a schema-42 save loads; the web tests. The kernel (604), web (142) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the format and schema checks clean. The ten-year smoke passed all 10 worlds (on a second run; no refusal was called, five petitions were, all to replace a keeper). Gate B and the notables' gate passed.

**Slice AH is complete** for what these villages can do: factions, petitions and refusals of a levy. **Open:** slice AI (seizure and founding, with the first office of force, and damage and violence); AJ (god tools, the regimes row, the M4 demo). The save-and-load flake from step one stays open.

## 2026-10-08 — The save-and-load flake, found and fixed

**Goal:** find the rare failure (about 1 run in 100) of the tests that save a world, load it and check both go on alike.

**What was found:** living one test world forty days from 200 fixed identities reproduced it twice. Narrowed step by step: only the decision receipts differed; the choice was the same, its probability was not; the option that scored differently was breaking new ground, at another site. A household's candidate new ground was found once a day and kept in a cache the save does not hold, so a world lived straight on reused a site found before a neighbour marked out a field that day, while a loaded world found a fresh one.

**What changed:** the cached site is kept only while the land its search reads (fields, plots, earthworks, the homes it keeps clear of) is unchanged. A household also no longer holds to ground a neighbour's field has since covered. The cache of a household's planned building has the same shape; nothing showed it, and it is recorded in the plan rather than changed.

**Evidence:** 600 further identities (two seeds) continued alike, where two of the first 200 had not. The kernel suite (604) passes; the ten-year smoke passed all 10 worlds; Gate B and the notables' gate passed.

## 2026-10-08 — M4c slice AI, step one: a faction's revolt

**Goal:** let a faction take the deciding from the gathering outside its procedure, without a roll or a timer: the revolt holds only if people, and the officeholders above all, choose to stand with it.

**What changed:**

- **A program:** a faction whose members blame the gathering holds the body under which the decisions its members saw would have gone most their way (slice AF's test), of those the content's amendments offer; unlike an amendment it need not be one change away.
- **Calling a revolt:** where neither a petition nor a refusal can be called, its organizer may call on everyone to stand with that body, worth what a petition is less `revolt_cost` and the norms they hold (ADR-0017 §4).
- **Sides:** each adult who heard of it takes a side each day, with it, with the gathering or with neither, from their grievance, belonging or regard for the organizer and where those they know stood the day before, less the norms they hold and a cost for one the new body would leave out (research 04-10 §5.3, 09-11 §1.6, §2.5).
- **Holding:** once the keeper and the watch, and more adults than stand with the gathering, have stood with it seven days (09-11 §2.2), its body decides; the custom's history records a version taken, not amended, and the chronicle says so. It comes to nothing after 60 days or when its organizer or faction is gone.
- **Boundary:** saves schema 44 (43 still loads), wire 1.43 (revolts in the Government panel), content API 48.
- **Also fixed:** the plan's AJ entry, which the last plan edit had run into AI's text.

**Findings:** with `revolt_cost` 1.0 even a fully aggrieved organizer where no one held the norm could not call one (0.65 against a threshold of 0.78); it is now 0.75, above a refusal's 0.5. The first test village amended its custom by procedure before its faction's organizer reviewed, and the decision the faction minded no longer counted, so no revolt came: the legal route first. The test now uses an elders' custom, which no single amendment would have changed to the faction's good.

**Evidence:** integration tests that a revolt is called, every one of the 27 who heard of it stands with it and it holds a week later, the custom's new version naming who took it; that one whose time runs out comes to nothing; each saving and loading exactly; a test that a schema-43 save loads; the web tests. The kernel (607), web (142) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the format and schema checks clean. The ten-year smoke passed all 10 worlds with no revolt called (four petitions in three villages, all granted). Gate B and the notables' gate passed.

**Open:** steps two to four of slice AI (founding, a watch of several and coups, force and violence); slice AJ.

## 2026-10-08 — M4c slice AI, step two: founding

**Goal:** after a revolt, let the body that took the deciding weigh what it inherited, so a seizure ends in a bargain from the old constitution rather than a blank (ADR-0017 §5; research 09-11 §1.6).

**What changed:**

- **A founding window:** for `founding_days` (90, a design prior) after the custom is taken, while laws the old custom made are unweighed, a new issue, `founding`, is before the village, and every member of the new body deliberates at each weekly review.
- **Ending a law:** a new policy kind, `repeal` (content `core:policy/repeal`), opens an end to each inherited law: a store's levy at none, as a petition ends it, or any other law repealed. A repeal that passes is carried, a new law status, and the law it names is superseded.
- **Weighing an end:** each household weighs it as what the law, kept, would bring it, turned about, with what the law does to what its people hold dear turned about too. Each law is put once; what the body leaves stands.
- **A fix in passing:** a petition to end a store's levy had weighed values as if for a store; any end to a law now weighs them turned about.
- **Boundary:** saves schema 45 (44 still loads), wire 1.44 (status `carried`), content API 49. The web now says "superseded by a later law" for any law a later one replaced.

**Evidence:** an integration test in which, after a revolt holds, the new body ends both the old custom's heavy levy (22 for, 1 against) and its curfew (24 for, none against) within two weeks, saving and loading exactly; a test that a schema-44 save loads; the web tests. The kernel (609), web (142) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the format and schema checks clean. The ten-year smoke passed all 10 worlds (no revolt, so no founding; three petitions in two villages, all granted). Gate B and the notables' gate passed.

**Open:** steps three and four of slice AI (a watch of several and coups; force, damage and violence); slice AJ.

## 2026-10-08 — M4c slice AI, step three: a watch of several and coups

**Goal:** give the village its first office of force worth seizing with, and let its holders, and only they, decide whether the deciding passes to them (ADR-0017 §4; research 09-11 §1.4).

**What changed:**

- **A watch of several:** while takings continue, a further watcher may be proposed beside those in office, each adding what the others leave unguarded; every watcher walks their own rounds and sees.
- **Coups:** a watcher of several who blames the gathering weighs, at their monthly review, calling on the others to take the deciding for the watch, at their grievance and how well the others regard them, less a cost and the norms they hold.
- **Sides among the watchers alone:** each day each watcher stands with it, with the gathering or with neither, from their grievance, their regard for the caller and where the others stood; once more back it than the gathering for a week, the body becomes those who keep the watch, the custom taken, and a founding follows.
- **Boundary:** saves schema 46 (45 still loads), wire 1.45 (coups in the Government panel), content API 50 (`coup_cost`, the membership `watch`).

**Findings:** the first build left the watch template open whenever a watch was in force, and the smoke had one coast village name 18 watchers and another 28: a move worth almost nothing is still proposed now and then and passes on regard for the one it names. A further watcher now needs takings since the newest watcher was named, and a sponsor it serves; the rerun named one watch in one village and three over ten years in another. A crime test assumed a single watcher; with takings continuing in it, the village named a second, as it now may. The ideology spread test failed about 1 run in 20: for some worlds' identities the ideology fitted nobody who heard it, so the test now sets what people hold dear.

**Evidence:** integration tests that three aggrieved watchers carry a coup and the watch then decides, saving and loading exactly; that a watch of two each walk their own rounds; that a schema-45 save loads; the web tests. The kernel (612), web (142) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the format and schema checks clean. The ten-year smoke passed all 10 worlds on both runs, and Gate B and the notables' gate passed.

**Open:** step four of slice AI (force, damage and violence); slice AJ.

## 2026-10-08 — M4c slice AI, step four: force

**Goal:** let the first office of force use it, with every blow a record and a cause (ADR-0017 §3–§4; research 04-10 §1.7–1.8, 09-06 §1.3, 06-10 §4).

**What changed:**

- **Going to collect:** at their monthly review, one who keeps the watch weighs going to take what a gathering's finding owed and a household refused, worth a duty, the norms they hold and their regard for the household owed, less their regard for the household owing.
- **How they are met:** each adult of the household lets it be taken, stands in the way, or strikes, from the norms they hold, their regard for the watcher, how strongly the household refused and their grievances against the watch and the gathering, against their own keyed reluctance to strike.
- **Force and blows:** where anyone resisted, the watcher takes it by force if it is still worth it less a cost for each who did, or turns back; each who struck strikes the watcher and is struck back. A blow keeps the one struck from work for days and rarely kills, a death by violence.
- **What it leaves:** grievances against the watch, a tie act for a blow, and the encounter told in plain verbs in the chronicle and the Order panel.
- **Boundary:** saves schema 47 (46 still loads), wire 1.46 (encounters in the Order panel), content API 51 (`w_collect`, `w_harm`, `strike_threshold`, `hurt_days`, `kill_share`; the tie act `struck`).

**Findings:** no village in the ten-year smoke named a watch or refused a finding this run, so force fired only in the tests; it waits on conditions the villages rarely reach, as a coup did in step three. A test that expected two grievances found one: the grievance against the watch for taking by force and the one for a blow merge as one wrong against one party, so the test accepts either.

**Evidence:** integration tests that a watcher takes a refused finding by force against a household that struck, each blow recorded and the save loading exactly, and that a killing blow is a death by violence with its record; that a schema-46 save loads; the web tests. The kernel (615), web (142) and end-to-end (16, 6 demos skipped) suites pass, with clippy and the format and schema checks clean. The ten-year smoke passed all 10 worlds, and Gate B and the notables' gate passed.

**Open:** keeping a seized store, damage to property, fear from a blow; slice AJ (the god tools, the regimes row and the M4 demo).

## 2026-10-08 — M4c slice AJ, step one: a whisper and an ideology heard of

**Goal:** the first god tools of M4, which reach people only through what they could perceive (ADR-0016 §5; research 15-05 §4.2, §4.5, §6).

**What changed:**

- **Intervention records:** each use of a god tool is one record with its number, target and what it submitted; a repeat refreshes it and never adds another, and the chronicle logs each as one recorded influence.
- **The whisper:** a true claim the adult's settlement's word holds and they have not heard (a call still to come, or another's grievance) is placed in their hearing from no one; what they do with it is what anyone who heard it would do, and it lapses as news does.
- **An ideology heard of:** the adult weighs it by how well it fits what they hold dear, by a draw keyed to them and the ideology alone, and again at their monthly reviews while it is fresh, so only a change in them can change the answer.
- **The observer's hand:** the inspector lists each influence on a person with what came of it (heard, told to others, came, joined, stood, weighed, taken up), read from the records people keep, and offers the news and ideologies to whisper and tell.
- **Boundary:** saves schema 48 (47 still loads), wire 1.47 (two commands, the welcome's ideologies, a person's news and influences), no content API change.

**Findings:** a save-and-load test counts the save's sections and needed the new one. The first inspector form's "Tell of it" button shared its name with the technique form's; the ideology one is now "Tell them".

**Evidence:** integration tests that a whisper is heard from no one and a hundred repeats are one record and one chronicle entry, that it writes nothing but the hearing, that an ideology told of is taken up only where it fits and repeats draw nothing new, and that a change in what someone holds dear lets them take it up at a later review; a schema-47 save loads; host tests decode the commands and answer them; an end-to-end test tells an adult of an ideology from the inspector. The kernel (620), web (144) and end-to-end (17, 6 demos skipped) suites pass, with clippy and the format and schema checks clean. The ten-year smoke passed all 10 worlds, and Gate B and the notables' gate passed.

**Open:** step two (the agitator, blessings and curses) and step three (the regimes row and the M4 demo).

## 2026-10-08 — M4c slice AJ, step two: the agitator, blessings and curses

**Goal:** the last two god tools of M4: a newcomer holding an idea, and luck in a person's own material draws (ADR-0016 §5).

**What changed:**

- **The agitator:** a map tool sends one adult newcomer, in a household of their own, holding the ideology chosen and knowing nobody, where a family would go; the chronicle tells it as one recorded influence, and the inspector says how many took it up from them.
- **Blessings and curses:** for a month to five years, a person's own chance of illness or accident (not hunger's) and of finding a technique out are scaled their way or against them, by up to half or double. The same draw is checked against the chance without it, so each death spared or brought and each find brought or cost is exact; it is counted on the record and told in the chronicle. A second blessing refreshes the first; a curse ends a blessing.
- **A household of one:** a family's making now allows a lone adult, with no couple, partner or union.
- **Boundary:** saves schema 49 (48 still loads), wire 1.48 (`SendAgitator`, `Bless`), no content API change.

**Findings:** none in the build; the tests needed the ideology's own name from content ("order kept by all").

**Evidence:** integration tests that an agitator joins the village alone, holding the ideology and knowing nobody, and that with a raised hazard blessings spared deaths and curses brought them, every turned draw told, and a blessing ending a curse; a schema-48 save loads; host tests decode and answer the commands; the end-to-end test blesses an adult and sends an agitator with the map tool. The kernel (624) and web (145) suites pass, with clippy and the format and schema checks clean. The end-to-end suite (17, 6 demos skipped) passes; the ten-year smoke passed all 10 worlds, and Gate B and the notables' gate passed.

**Open:** step three (the regimes row and the M4 demo).

## Development pattern that emerged

The project now develops in small, reviewable vertical slices:

1. Read the milestone's research briefs and current plan.
2. Put a cross-cutting, expensive-to-reverse choice in an ADR; keep tuning values and reversible implementation choices in the plan's decision log.
3. Add the model in the owning Rust domain crate, then connect it through `civ-sim`, saves and wire payloads as needed.
4. Give the observer the model's facts and commands. Keep rendering and wording out of simulation authority.
5. Add unit and integration checks, then run the relevant smoke worlds and preserve notable outcomes, failures and nudges in the plan.
6. Update the README and technical notes so implemented, partial and planned work remain distinct.

The most important process result is a growing evidence trail: source code, tests, smoke records, decisions and docs point to the same feature boundaries. The journal should continue to record both what improved and what a run exposed as a limitation.
