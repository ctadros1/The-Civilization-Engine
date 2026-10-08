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

**Open:** nobody asked the store for relief in these runs, because nobody ran short. The government panel (Z2), leaving as a choice (Z3), offices and succession (Z4), labels (Z5) and the notables' gate with the demo (Z6) are next.

## Development pattern that emerged

The project now develops in small, reviewable vertical slices:

1. Read the milestone's research briefs and current plan.
2. Put a cross-cutting, expensive-to-reverse choice in an ADR; keep tuning values and reversible implementation choices in the plan's decision log.
3. Add the model in the owning Rust domain crate, then connect it through `civ-sim`, saves and wire payloads as needed.
4. Give the observer the model's facts and commands. Keep rendering and wording out of simulation authority.
5. Add unit and integration checks, then run the relevant smoke worlds and preserve notable outcomes, failures and nudges in the plan.
6. Update the README and technical notes so implemented, partial and planned work remain distinct.

The most important process result is a growing evidence trail: source code, tests, smoke records, decisions and docs point to the same feature boundaries. The journal should continue to record both what improved and what a run exposed as a limitation.
