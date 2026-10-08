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

M3b went on to complete slices Q and R: deposits became part of new worlds and household extraction, earthworks and pits changed the surface, and building style spread through copying. The main-tree baseline documented here includes all of M3b.

## 2026-10-05 — M3c S and T: time modes and a long-run baseline

Slice S added Accelerated speeds at 60×, 600× and Max. Accelerated advances a whole day at a time and changes mode only at midnight, using the same event machinery and authoritative state. Gate A checks compare advance partitions, mode changes and save/load continuations. Exact performance work was measured against state digests so speed improvements could be distinguished from changed histories.

Slice T added `civ-host dashboard`, which runs five river-valley worlds for fifty years and reports the applicable population, food-price, goods-inequality, workshop-size and building-failure measures. Its initial, untuned baseline missed the population and lived-in building-failure bands. That baseline remains evidence to explain, not a result tuned away.

## 2026-10-05 — M3c U: weather reaches farms, buildings and the observer

Slice U replaced one annual climate multiplier with a daily weather stream for each landscape. Seed, landscape and day identify the stream, so both speeds see the same regional weather while the two landscapes have separate histories. Rain persistence and amounts, temperature anomalies, slow wet/dry periods and snow now feed a reference soil-water balance and per-field crop water. Harvests respond to water received, and wild plant growth follows soil moisture.

The integration crossed several layers. Breaking and preparing ground and sowing wait for frozen, snow-covered or wet ground; other field tasks continue. Farm plans use the long-run share of workable days instead of seeing future weather. Month-level rain affects building wear; storm days and snow contribute roof loads; grain asking prices respond to household stores. The observer now shows conditions on the clock, weather and season information on the map, monthly/yearly summaries in a panel, annual extremes in the chronicle, and water status on growing fields. Weather and field water are saved; the boundary reached wire 1.24 and content API 24, with save schema 24.

The final ten-year smoke for U passed the selected checks across ten worlds. Five worlds met the population floor, while four bands died out and another ended below it. Earlier smoke iterations exposed failures from over-restricting wet-weather work and from a tool-replacement loop; those causes were investigated and corrected. The remaining fragility is open work for field soils and subsequent tuning, not evidence that weather or farming is calibrated. Slice V (soil nutrients and fertility) and W (Accelerated-mode approximations, tuning and demo) remain after this baseline.

## Development pattern that emerged

The project now develops in small, reviewable vertical slices:

1. Read the milestone's research briefs and current plan.
2. Put a cross-cutting, expensive-to-reverse choice in an ADR; keep tuning values and reversible implementation choices in the plan's decision log.
3. Add the model in the owning Rust domain crate, then connect it through `civ-sim`, saves and wire payloads as needed.
4. Give the observer the model's facts and commands. Keep rendering and wording out of simulation authority.
5. Add unit and integration checks, then run the relevant smoke worlds and preserve notable outcomes, failures and nudges in the plan.
6. Update the README and technical notes so implemented, partial and planned work remain distinct.

The most important process result is a growing evidence trail: source code, tests, smoke records, decisions and docs point to the same feature boundaries. The journal should continue to record both what improved and what a run exposed as a limitation.
