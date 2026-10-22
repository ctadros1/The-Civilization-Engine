# M10 design brief: rail and gas light

**Scope.** Plan §7's M10 ("Industrial content") lists "steam, factories, rail, gas light; capability
rungs for industrial services; healthcare and education as full services; fiat money". Other briefs
take steam and factories (the power brief), health and education, and money. This brief covers:

- railways as vocabulary: wagonways, then locomotive rail; track as a way, rolling stock as
  vehicles, stations, freight and passengers, timetables, who builds and owns a line;
- what rail changes: travel time, trade, and where people live;
- gas light: gasworks from coal, holders, mains, street and house lighting;
- what light changes: working hours, night activity, safety and the watch, fire;
- routing and cost: a rail network over the walking grid, and ADR-0032's amendment of 2026-10-21;
- guards, suggested slices, open questions and what an ADR must decide.

The power brief defines the steam engine. §1.3 says what rail needs from it and assumes nothing
more.

**Reports read:**

- In full: **07-08** (transport technology), **12-09** (utilities, with the gas parts read
  closely), **12-05** (urban mobility), **12-06** (traffic models) and **12-03** (urban fire).
- **12-04** (policing). It has no section on lighting and crime, which is a gap (§2.4).
- Skimmed: **08-12** (trade and merchants).
- For light and residence: **04-02** §1.2 and §3.7, and **08-19** §1.3 and §4.

**What exists to build on** (checked in the code):

- **Ways** (`civ-land/src/ways.rs`, ADR-0029 §1).
  - A way is a list of segments. Each segment is a list of 8 m map cells, with three widths, a
    `Surface` (cleared, shaped, gravel, causeway), a `Keeper` (nobody, a household or a polity)
    and a condition that fades. The rank is derived.
  - The modes a way serves are bits: `WALK` = 1 and `CART` = 2.
  - `Ways::holds` holds the wear of **every** way's cells at its condition, whatever its `modes`.
    So any way, a railway included, would become a full-speed footpath.
- **Vehicles.**
  - Content: `content/core/vehicle/dugout.toml` and `handcart.toml`. `VehicleMode` is
    `{Boat, Cart}` (`civ-agents/src/params.rs`).
  - A cart's `pull_n` (200 N) and `power_w` (75 W) are its puller's, and the puller is a person.
  - Carts and boats are records in `boats::Boats` (`Land.carts` reuses that type). Each is owned by
    one household (`civ-land/src/boats.rs`).
- **Wheels** (`civ-land/src/carting.rs`).
  - `Wheels::speed` uses the pull `M·g·(C + s)`, capped at the puller's walking speed.
  - `Terrain::route` is a Dijkstra search over grid cells that take wheels.
  - `Land::cart_way` collects the decks again and searches afresh each time it is called. A
    household weighing a fetch by cart runs two such searches (`population/market.rs`,
    `way_by_wheels`). Nothing caches them.
- **Boats and goods in transit.** `boating.rs` has a directed `RiverGraph`. A `Consignment`
  (`civ-agents/src/consignments.rs`) has one owning household, one carrier and one boat.
- **Bottlenecks** (`civ-agents/src/traffic.rs`).
  - Facilities are `Crossing`, `Gate` and `Narrows`, served first by arrival, and nothing
    despawns.
  - `travel::Mode` is `{Walk, Boat, Cart}`.
- **Works.**
  - `population/way_works.rs` and `PolicyKind::BuildWay` build passing places only.
  - M8b deferred widening, crossings that carry carts, landings, grading as an earthwork, upkeep by
    law, and tolls (plan §7, "M8b closed").
  - `EarthKind` has no grade kind (`civ-land/src/earth.rs`). There is one bridge system,
    `core:bridge/log_beam`.
- **Walking routes** (`civ-world/src/nav.rs`, `civ-agents/src/population.rs` `start_walk`).
  - A* is bounded by **one** `Landmarks` set (`LANDMARKS` 8) over `wear.walked_area()`'s bounding
    box plus `LANDMARK_MARGIN_CELLS` (32).
  - The set is dropped, and built again at the next search, whenever `Land::route_revision`
    changes. That revision combines the crossings, the wear survey and the travel revision.
  - Other limits: `ROUTE_BUDGET` 600,000 cells; a cache keyed by (from cell, to cell) of at least
    `ROUTE_CACHE_MAX` 50,000 routes, or 8 a person.
  - ADR-0028 §4 decided "landmarks are built per region (each settlement's walked ground), not one
    box over the world". That was never built.
- **ADR-0032's amendment of 2026-10-21 (slice DN).** In ten small settlements on a large map, "30 %
  of the time went to rebuilding route landmarks over the bounding box of all walked ground, thrown
  away whenever walking costs anywhere change". Its first lever is landmarks over each settlement's
  own walked ground. The M9 demo's slow days were the same rebuild.
- **The household day** (`population/resolution.rs`).
  - `NOT_TAKEN` lists visits, fetching, carrying dues, bands and the four fire behaviours.
  - Options moved by boat or cart (`Step::Paddle`, `Haul`, `Handle`) are left out with the note
    "wait for the journeys they make (slice DH lists them)". The plan defers them to slice DJ, but
    DJ built founding and memory, not journeys.
  - So no journey between settlements, and no vehicle journey, is lived in a household-resolved
    settlement.
- **Map scale.** `CELL_SIZE_M` is 8.0 (`civ-sim/src/lib.rs`) and `MAX_CELLS` is 4096
  (`civ-world/src/lib.rs`), so a map is at most 32.8 km a side. M9's bench used 2048 cells
  (16.4 km).
  - A fetch reaches at most 120 minutes away (`content/core/activity/fetch.toml`). It is weighed
    at off-trail pace: `tobler_ms(0)` ≈ 1.40 m/s × 0.6, so about 6 km in a straight line.
- **Night is astronomical.**
  - `needs::daylight(latitude)` sets `Facts.dark`, `daylight_left_min`, `evening` (from sunset −60
    to +240 minutes) and `until_sunrise_min`.
  - 52 activities are `daylight_only = true`, and `w_dark` (6.0) counts against outdoor work that
    would run into the dark.
  - Sleep onset follows sunset (`needs::circadian`, `bedtime_after_sunset_hours` 3.3, after
    04-02 §2.2).
  - Of the 34 goods, none is a lamp, a candle, oil, tallow, coal or iron. Nothing makes light.
- **Curfew and watch.**
  - `PolicyKind::Curfew` scores options away from home in its hours below others; the penalty
    is in `population.rs`.
  - `limit_daylight_work.toml` reads a curfew's hours against the day's light.
  - `KeepWatch` rounds are walked only while it is dark, three a night (`population/watch.rs`,
    `crime.rs`).
- **Takings.** In `population/crime.rs` (`take_from`), anyone awake within `sight_m` (30 m) sees
  the taker, day or night. Darkness matters only through who is asleep: it never shortens sight.
- **Fire** (`civ-agents/src/fire.rs`).
  - Ignitions come from a hearth (lit or banked, per hour) and from fire worked **at home**.
  - `works_wrought_worked_fires.toml` keeps such homes 10 m apart.
  - No fire is worked anywhere but a home.
- **Owners.**
  - Firms are household workshops (`firm.rs`).
  - An organization may be founded from a shared stake, and a privilege gives it a licence or a
    staple (ADR-0026 §2–§3). Pay for common works comes through `pay_work`.
  - "Carrying others for a fee waits for an agreement design" (ADR-0029 §5).
  - A plan's land assembly lists clauses per kind of claim (ADR-0030 §4).
- **Boundary.** `GetWays`, `GetVehicles` and `ways_rev` exist (`civ-schema/schema/tce_wire.fbs`).

## Where the reports agree

1. **A railway is a mode, not a speed bonus.** It moves payload cheaply and reliably.
   - "Moving quickly, moving a heavy load cheaply, and delivering it reliably" are different
     achievements (07-08, design conclusion).
   - Freight-train templates are 100–1,000 t at 15–40 km/h. They need "track, locomotives, fuel,
     water, workshops, dispatch" (07-08 §2.4).
2. **Track complements other ways.** A trunk line "should change market access. It should not
   remove the need for collection, delivery, warehouses, and local movement" (07-08 §3, industrial
   row). "Terminal dwell may dominate door-to-door performance" (07-08 §2.4).
3. **A scheduled service is a business.** Services "offer actual departure times and finite
   capacity. Their owners hire workers, buy supplies, maintain assets, and choose whether to
   operate" (07-08 §5.1). An omnibus "should emerge as a scheduled transport business" (12-05
   §5.6).
4. **Technology does not guarantee success.** "A larger or faster asset should lose money when
   underutilized" (07-08 §4).
5. **Light is a service, and gas is a chain.** The chain runs "Feedstock → gasworks → purification
   → gas holder → street mains → meter → compatible burner" (12-09 §1.4).
   - A connection is an investment of its own (12-09 §1.2).
   - Networks extend where demand pays their fixed costs (12-09 §1.3).
   - Infrastructure comes before universal uptake (12-09 §4).
6. **A building is served only when every condition holds**: "connected ∧ source available ∧
   network feasible ∧ account authorized ∧ usable quality". Failures happen at assets, not to
   persons (12-09 §1.5–§1.6).
7. **Ownership changes incentives and finance, not physics.** There is no fixed efficiency bonus
   for public or private owners (12-09 §1.7, §4).
8. **Light widens some windows at a cost and switches nothing on.**
   - "Artificial lighting expands some windows at a resource cost" (04-02 §1.2).
   - Light must not become "an automatic 'segmented sleep off' switch" (04-02 §3.7).
   - "Evening light without a feasible activity should not automatically increase income" (12-09
     §3.3).
9. **Lighting is an ignition activity of its own** (12-03 §1.1). Improvements are "separable and
   reversible" (12-03 §3.4).
10. **Capabilities, not eras.** Dates are "never simulation gates" (07-08 §5.2). Industrial
    capabilities "enable new combinations" (12-09 §3.1). Trams and buses come from "materials,
    engineering knowledge, investment, passenger demand, operating institutions, and energy supply —
    not fixed eras" (12-05 §5.6).

**What the reports teach to avoid:**

- A universal railway income bonus. Of Donaldson's 16 %, 08-12 §4 says "Do not award 16%
  automatically".
- Utilities as citywide coverage flags (12-09's recommendation and its §5.6 note on Cities:
  Skylines).
- Gas as a required step before electricity (12-09, opening).
- BPR delay counted on top of queues (12-06 §2.1).
- Jams cleared by turning a blocked delivery into a completed one (12-06 §5).

**Where they differ:**

- **Speed.** 07-08 §2.4 gives freight trains 15–40 km/h. 08-12 §2.1 gives rail "up to 600 km/day"
  as an operating capability, "not every consignment's delivery rate".
- **Cost.** 07-08 §2.2 puts British rail near 1.10 pence a t-km (c.1845), against 4.59–6.04 for
  roads and 1.223 for inland water (1830). 08-12 §2.2 has Indian rail at 1, road at 4–5 and river
  at 2–4. Both rank rail near water and well below road. Neither is a constant to impose.
- **Lighting and crime.** 12-04 is about patrols, encounters and response (§1.4, §2.1, §4 E–F).
  It says nothing on light, so any effect of light on takings must come through sight and who is
  about (§2.4).

## 1. Rail

| Mechanism | Saved state | Driven by |
|---|---|---|
| Track | A way serving `RAIL`: per segment, rail good, gauge, condition, the bed's grade profile | Work a household, firm or polity chooses |
| Rolling stock | A vehicle record (as `Boats`): wagon, carriage or engine, owner, where it stands | A recipe gated by techniques |
| A train run | Its consist, its line, its timetable slot, its position by time, the consignments and riders it carries | An operator's timetable |
| Block section | A facility: one train at a time; trains queue first by arrival | Track between stations and passing loops |
| Station | A record on a line: platform length, loops and sidings, handling positions, owner | Built as a work |
| Timetable and terms | Departures per stop, the fare, the freight rate a kg | The operator's choice |

**Derived, not saved:** train positions between events, the line's rank, traffic by section and
hour, and each station's walking fields.

### 1.1 Track: wagonways first

- **Track is a way that serves `RAIL`.** `RAIL` = 4 is appended to the mode bits. A rail segment
  carries:
  - a gauge;
  - its rails (timber, or iron from the power brief's metals);
  - its sleepers and bed;
  - its condition.

  A railway is a record like any way: begun by a first work, with an identity and a keeper
  (ADR-0029 §1).
- **Geometry below the grid.** Rail cannot turn 45° in a cell, so its line is a centreline with a
  minimum radius (a content prior per track kind), and the cells it covers are derived. 12-05
  §1.4's usable width and ADR-0029 §1's lines below the grid both apply. This is open question 2.
- **Wagonways come first, and are within reach of today's content.** Timber rails
  (`wood_shaping`) and a wagon (`wheel_and_axle`) can join a pit, a quarry or a landing to where
  the goods are used. 07-08 §5.5's P01 needs only "wheel/axle craft + track construction + rights
  of way". The only traction the world has is people (ADR-0029 §2: no beasts), so wagons are
  pushed.
- **Grading is a prerequisite.** Rail's low rolling resistance makes grade dominate (§1.2), so a
  line needs the grade earthwork that ADR-0029 §6 decided and M8b deferred: a target profile, with
  volumes in three states (11-12 via ADR-0029 §6).
- **Crossings that carry the load.** A line over water needs a bridge system that carries a
  wagon's or an engine's gross load. Only `log_beam` exists, and "a structurally adequate bridge
  may still be unsuitable for carts" (11-07 via the M8 transport brief §1.2).

### 1.2 What rail physics gives (my arithmetic)

The pull is `M·g·(C + s)`, as in `carting.rs`. No report read gives rail's `C`. As design priors:
0.002 for iron on iron, and 0.005 for timber rails.

- **Grade.** At `C` = 0.002, a 1 % grade needs (0.002 + 0.01) / 0.002 = **6 times** the level
  pull. At the handcart's way resistance of 0.02 it needs 1.5 times. Grade, not distance, is what
  makes a line costly to run, so lines will contour, cut and embank (10-07 §1.2's "longer
  contouring route").
- **People pushing.** Take a wagon carrying 2 t with a 0.5 t tare on timber rails. On the level
  it needs 2,500 × 9.81 × 0.005 ≈ 123 N, inside a person's 200 N (the handcart's `pull_n`), and
  moves at 75 W / 123 N ≈ 0.61 m/s.
  - That is about 1.2 t·m/s of load. A handcart's 150 kg at the level walking pace
    (`tobler_ms(0)` ≈ 1.40 m/s) is about 0.21, so a wagon moves some **six times** as much per
    person-hour on the level.
  - On a 1 % rise the same wagon needs 2,500 × 9.81 × 0.015 ≈ 368 N, more than a person gives.
    Pushed wagonways run level or loaded downhill, or stop.
- **An engine.** Take an illustrative engine of 20 kW (the power brief sets the real figure)
  pulling 100 t.
  - On the level at `C` = 0.002 the pull is 1,962 N, so its power would give about 10 m/s,
    capped at its top speed.
  - On a 1 % rise the pull is 11,772 N and the speed 1.7 m/s, a walking pace.
  - Adhesion caps the pull at μ·m·g on the driven wheels. With μ = 0.2 and 10 t on them, that is
    19.6 kN, enough for a train of about 166 t, engine included, up a 1 % rise.
- **The map.** A 2048-cell map is 16.4 km across. That is about 3.3 h on a level trail (about
  5 km/h), against 33 min by train at 30 km/h (07-08 §2.4's band) plus waits and terminals. At
  `MAX_CELLS` (32.8 km) it is 6.5 h against about 1.1 h.
  - **Rail on TCE's maps changes payload per worker far more than reach.** This is why rail is a
    mode and not a speed bonus (07-08, conclusion). Open question 8 asks whether M10 needs more.
- **Freight demand.** The M8 transport brief §1.4 put a town of 30,000's grain coming in at about
  230 t a day over 30 days. That is two or three 100 t trains a day, against about 1,150 handcart
  loads.

### 1.3 Rolling stock and what rail needs from the power brief

- **A wagon, carriage or engine is a vehicle** of an appended mode `rail`, a record like a cart's.
  Its spec: payload (kg and seats), tare, gauge, `C` on track, braking grade, life and handling.
  It runs only on `RAIL` segments of its gauge.
- **From the power brief, an engine as a made puller** in place of `pull_n` and `power_w` being a
  person's:
  - its sustained power and top speed;
  - its mass on the driven wheels (for adhesion);
  - its fuel (a good, kg an hour at a load) and water (litres an hour);
  - the time to raise steam, and its crew;
  - its upkeep: hours in a shed or workshop, with wear by hours run.

  It needs fuel and water at stops: tenders and water points are facilities on a line (07-08
  §2.4).
- **Making them** needs the power brief's techniques and metals (iron, boilers; 07-08 §5.5 P02:
  "boilers/engines + precision metalworking + fuel/water services"). Using one needs only that it
  is there, as ADR-0029 §2 says of every vehicle.

### 1.4 Stations, runs, timetables, single track

- **A train run is the line's own event chain, not a person's.**
  - It departs at its slot. It is timed along the line by grade and consist, with the speed
    profile worked out once per line and consist.
  - It is re-timed only where it waits: a block section, a loop, or a station's handling.
  - Its crew are people whose work is the run.
- **Block sections are bottleneck facilities** under ADR-0029 §4's rule: one train in a section,
  first by arrival, waiting at the last loop or station. Single track meets at passing loops, as
  carts meet at passing places (ADR-0029 §4's amendment). 12-06 §7.1's finite storage applies:
  nothing overlaps, and nothing despawns.
- **Stations handle** at 12-05 §1.5's `t_setup + Q/(n·r)`: freight by the people working there,
  and passengers boarding at a rate. "An empty landing without workers must not function like a
  fully staffed warehouse" (12-05 §1.2).
- **Level crossings.** Where a line crosses a worn trail or a way, a crossing is a facility that
  walkers wait at while a run passes (12-05 §1.4, clearance failure).
- **Timetables are the operator's choice**, revised at its monthly review from the loads and
  riders it saw. 12-05 §5.6 asks for both capacity and frequency: 22 seats every 15 minutes is 88
  seats an hour.

### 1.5 Freight, passengers and fares

- **Freight is consignments of many owners on one run.** This extends ADR-0029 §3's holder:
  - the carrier becomes a run, not one person and one boat;
  - each owner's goods stay owned and spoil by their own clocks;
  - goods wait at a station in its store when a run is full or cancelled.
- **A posted fare and freight rate** are terms the operator posts, paid through the ledger. TCE has
  no priced carriage yet (ADR-0029 §5). This is open question 5.
- **Choice is door to door, in hours.** A rider compares a direct walk with access + wait + ride +
  egress (12-05 §5.6, its `T` door-to-door), plus the fare in hours of their own work. A shipper
  compares the whole chain, as for boats and carts (ADR-0029 §3). Agents know the timetable once
  they have seen the station or been told of it: "a physically available route" is not "one a
  traveler knows" (07-08 §5.1; 12-06 §3).

### 1.6 Who builds and owns a line

- **A household or a firm** lays a wagonway from its own pit or landing. A firm may own wagons; an
  engine needs the capital the money brief gives.
- **An organization by stake** (ADR-0026 §2) may found a line company. A privilege law (ADR-0026
  §3) gives it the sole right to run on a line in a settlement, which is a franchise (12-09 §1.7).
- **A polity** builds by an appended `build_line` policy shaped like `build_way`: an issue raised
  where recorded losses (hauls, waits) would repay the work, forecasts in hours, equal shares or
  `pay_work`. 07-08 §1.7 says infrastructure is built "to an actor able to finance and organize
  it".
- **The ground.** A corridor across many claims is ADR-0030 §4's assembly problem. A line is an
  appended plan operation (ADR-0030 §2 lists "axial" among later operations) with its clauses, and
  "a standing home is never cleared by a plan".

### 1.7 What rail changes

- **Travel time.** Rail changes travel time only for those within walking reach of a station, and
  by the door-to-door sum. Most trips stay on foot (12-05 §4.2, person-trips against tonne-km).
- **Trade.** The fetch weighs a station pair like a boat's way. A run's capacity and fare, not a
  multiplier, narrow price gaps "where competition and supply allow" (07-08 §4; 08-12 §4, the
  no-arbitrage band). This is the M5b convergence row's test.
- **Where people live.** The residence review (ADR-0029 slice CR, `[moving] w_walk`) and plot
  choice (ADR-0030 §1) weigh travel by any mode the household uses, in hours. 08-19 §1.3: "use
  network travel time". Berlin's railway decentralization is "a historical model attribution, not
  a universal transport elasticity" (08-19 §2.1).

### 1.8 Routing and cost: rail over the walking grid

- **Rail is never an edge in the walking search.** Walking A* keeps its bound (top walking speed,
  landmarks on walking times). A faster mode inside it would break that bound, and ADR-0028 §4 and
  ADR-0029 §7 say such a bound "is not admissible".
- **A rail journey is composed, not searched.**
  - Two walking travel fields per station (to it and from it), bounded by the walk people accept,
    are built with `NavGrid::travel_field`. They are keyed to `route_revision`, as each
    settlement's field is (`population.rs`).
  - A rider's best option is the minimum, over known station pairs, of walk + wait + ride + walk.
  - Rides come from an earliest-arrival scan over the station graph's departures. Stations number
    in the tens, and the scan is recomputed when a timetable changes.
- **Track changes nothing for walkers.**
  - A `RAIL`-only way does not hold wear: `Ways::holds` gains a mode filter.
  - Its revision does not enter `route_revision`, so building track never drops the walkers'
    cache or landmarks.
  - Train runs follow fixed lines and need no search.
- **ADR-0032's amendment of 2026-10-21.** Landmarks over the whole walked box, thrown away
  whenever walking costs change, are already the main cost in many-settlement worlds.
  - Rail joins settlements, so the walked box only grows.
  - Rail can also *remove* the long walks that need whole-map bounds, but only if landmarks are
    per settlement (ADR-0028 §4, never built). Then the legs to stations are local searches.
  - So per-settlement landmarks land first (slice 1). Long walks without rail fall back to the
    straight-line bound within `ROUTE_BUDGET`, as walks outside the box do now.
- **Cart and rail routes are cached by the ways' revision.** Today's `cart_way` searches twice per
  weighing, uncached.
- **Accelerated mode** lives the same runs and sections (ADR-0029 §8).
- **The household day** needs a contract:
  - a freight consignment rides its run whatever the resolution, since a run is the line's event;
  - a ride by a household-resolved person is either a block (ride minutes from the timetable,
    goods in store at the next midnight, as a fetch within reach is under ADR-0032 §4) or not
    taken.

  Today every vehicle journey is left out (`resolution.rs`), so without a contract a rail world of
  ten settlements carries no one between nine of them. This is open question 7.

## 2. Gas light

| Mechanism | Saved state | Driven by |
|---|---|---|
| Gasworks | A building: retorts, purifier, stock of coal, coke and tar, staff, condition | A recipe (coal → gas + coke) gated by a technique |
| Holder | A building: m³ stored (and energy content), capacity, condition | Built with a gasworks or apart |
| Main | A line below the grid along ways: diameter, cells, owner, condition, leakage | A work, along a way with its keeper's leave |
| Connection | On a building: burners (street lamp, house lamps), meter, account | A household's or a polity's choice |
| Lamp round | The lamplighter's evening and morning rounds | Work, as the watch's rounds are |

**Derived:** each building's service record (12-09 §5.4: requested and delivered, hours lit,
interruptions), the night-light map, and each district's served share.

### 2.1 The chain

- **Gas is made, stored and piped** (12-09 §1.4).
  - A gasworks turns coal into gas and coke. The coke is a by-product that "affected the
    economics".
  - The holder buffers steady making against the evening peak: "do not treat gas as entirely
    instantaneous".
  - Store gas by **energy content**, not by volume (12-09 §2.3's warning).
  - The yield per tonne of coal is a content prior. No report read gives one.
- **Mains are assets, built and kept.** Their cost comes from TCE's own quantities: digging,
  pipe, restoring the street, and rights (12-09 §1.3). A main under a polity's way needs that
  keeper's leave, which is `Keeper` on the segment.
- **Service is a per-district check, not a flow solver**: capacity and pressure adequacy with
  storage and leakage (12-09 §5.2), every 15–60 simulated minutes (12-09 §5.5), and only where a
  network exists. Interruptions keep four causes apart: no gas, an overloaded main, a damaged
  asset, an account cut off (12-09 §1.5).
- **Demand comes from burners and hours** (12-09 §1.1). One example burner uses 0.142 m³ an hour,
  and two house lamps lit four hours a day use about 413 m³ a year (12-09 §2.3).
- **Street lamps (my arithmetic).** Take one lamp every 30 m (a prior) lit six hours a night. A
  kilometre of street uses 33 × 6 × 0.142 ≈ 28 m³ a night, about 10,300 m³ a year.

### 2.2 Who builds, who pays

- **Configurations, not a ladder** (12-09 §1.7):
  - a polity lights its streets as a public work (shaped like `build_way`), paid from the store
    or by `pay_work`;
  - a household firm, or an organization by stake, makes and sells gas by the meter;
  - a franchise is a privilege law giving the sole right to lay mains in a settlement's streets
    (ADR-0026 §3).
- **The utility's ledger is real**: bills and subsidy against coal, wages, upkeep and investment
  (12-09 §1.7). Extensions are weighed by expected connected demand (12-09 §1.3's NPV, in hours).
- **Households connect by choice** (12-09 §1.2): the connection and burners against the hours
  that light would give them. They keep what they have; there is nothing to keep yet (§2.3).
- **Lamplighters walk rounds**, as the watch does (`population/watch.rs`): an unlit lamp gives no
  light.

### 2.3 What light changes

- **Darkness becomes a property of a place**, no longer the world's alone. `Facts.dark` stays
  astronomical, and a person's work or path counts as dark only where no burner of theirs, or
  street lamp, lights it at that minute.
  - `daylight_only` indoor work (making, milling, baking) can then run lit into the evening.
  - Walks on lit streets are not "into darkness" for `w_dark`.
  - Fields stay dark.
- **Evening and sleep.** People may stay up because their evening activities still score, not
  because the lamp moves bedtime. Whether light shifts `needs::circadian`'s onset is open question
  12; 04-02 §3.7 warns against an automatic switch.
- **A world with no lamp at all.** Light compared with nothing overstates what gas brings. 12-09
  §1.2 compares alternatives, and §3.1 has pre-industrial lighting as "a labor-intensive purchased
  service". A rush light or an oil or fat lamp is open question 4.
- **Curfew and limits** stay laws over hours. `limit_daylight_work` keeps reading astronomical
  light: lamps do not make a curfew lawful.

### 2.4 Safety and the watch

- **Light can act only through sight and who is about**, because `take_from` uses one `sight_m`
  day and night. A dark sight distance (`dark_sight_m`) is needed, with the day distance applying
  on lit ground. 12-04 §4 F asks for concentration to emerge and not be "hardcoded", and §4 E
  allows effects to fade. A lamp is a place where takers are seen, never a crime-rate factor.
- **Turning on dark sight** below `sight_m` changes every world's digests even without gas
  (open question 3). With `dark_sight_m` = `sight_m` nothing changes until someone lights a lamp.
- **The watch** (12-04 §1.4: patrols act where they are) sees farther on lit streets by the same
  rule.

### 2.5 Fire and gas

- **Burners are ignition sources** (12-03 §1.1: lighting separate from cooking and heating). Each
  lit hour carries a hazard like `hearth_lit_per_hour`, put out at once when someone awake is by.
  In TCE a lit house adds hazard over darkness, since nothing it replaces exists.
- **A gasworks is a worked fire away from home**, which `fire.rs` does not yet have. A holder or a
  main can leak (12-09 §1.4) and, rarely, ignite. Both are hazards per asset-day by condition
  (12-09 §1.6), never a scheduled disaster.
- **Nuisance and siting** reuse ADR-0030 §6's rules: "gasworks kept from homes" is a template like
  `works_wrought_worked_fires`, proposed when a harm the kernel models has touched households.

## 3. Defer

| Defer | To | Why |
|---|---|---|
| Horse wagonways, horse trams and omnibuses | A husbandry design | No beasts (ADR-0029 §2) |
| Signalling beyond block sections, telegraph dispatch | With telegraphy (M11) | 07-09's telegraph tied to railway operations |
| Electric light and traction | M11 | 12-09 §3.1, §3.2: separate networks, no forced sequence |
| Gas cooking and heating | After lighting | 12-09 §1.2: one service at a time; keep multiple fuels |
| Pressure solver for mains | When districts outgrow the capacity check | 12-09 §5.2: validate before claiming realism |
| Sea and steamships | A port design | 07-08 §5.5 P03 |
| A god tool that lays track or lights streets | Never | It would choose an outcome |

## 4. Guards

- **Plot.** No date, era or population opens rail or gas. No line or main is authored. No mode
  carries a named speed bonus. No income or productivity bonus comes from a connection (12-09
  §3.3) or a railway (08-12 §4). If nobody builds within the time box in a world that knows how,
  nudge a prior (what a household believes a haul or a lamp saves) and log a `NUDGE:`.
- **Digests.** A world without track, gas or lamps keeps the build-before's digests. The mode
  filter on `Ways::holds` and per-settlement landmarks are each a one-time digest change, with
  route times checked equal (ADR-0028 §4).
- **Conservation.** No lost riders or duplicated cargo, no gas delivered from storage twice, no
  supply through a disconnected main (12-09 §4; 12-06 §10).
- **Stylized facts as plausibility checks:**
  - rail carries most tonne-km while person-trips stay on foot (12-05 §4.2);
  - price gaps narrow after a line opens where supply allows (07-08 §4);
  - an underused line or an unconnected main loses money (07-08 §4; 12-09 §1.3);
  - mains run beside unconnected homes (12-09 §4);
  - local gas failures coexist with an adequate works (12-09 §4);
  - delay concentrates at sections and stations (12-05 §4.1);
  - the same run takes the same time at every speed and from any camera (12-06 §8).

## 5. Suggested slices

1. **Routing ground for faster modes.**
   - Build: landmarks per settlement's walked ground (ADR-0028 §4; ADR-0032's amendment of
     2026-10-21, first lever); `Ways::holds` by mode; cart routes cached by the ways' revision;
     stations' walking fields (built, not yet used).
   - Tests: route times equal to Dijkstra's; one recorded digest change; the M9 bench's
     landmark share measured before and after.
2. **Wagonways.**
   - Build: `RAIL` track as a way with geometry below the grid; timber rails; a wagon pushed by
     people; grading as an earthwork kind (ADR-0029 §6); a household's or firm's line from a pit
     or landing.
   - Tests: §1.2's pulls at 0 and 1 %; no wagon off its gauge; earth balances; a world without
     track keeps its digests.
3. **Engines and runs.**
   - Build: the power brief's engine as a puller (fuel, water, adhesion); runs as the line's
     event chain; block sections and loops as facilities; level crossings; trains in the observer.
   - Tests: 12-06 §10's scenarios with trains (meeting on single track, a section blocked, a
     bridge failing under a run); nothing despawns.
4. **Services.**
   - Build: stations and handling; timetables; fares and rates as posted terms; consignments of
     many owners on a run; door-to-door choice for riders and shippers; `build_line`, line
     companies and franchises; the household-day contract.
   - Tests: price gaps narrow along a line in a two-settlement fixture; Gate C with a line.
5. **Gas.**
   - Build: coal → gas + coke at a works; holders; mains and connections; the district check and
     four interruption causes; who builds (polity, firm, franchise); lamplighters.
   - Tests: energy conserved through the holder, leakage, an outage by cause.
6. **What light changes, and the demo.**
   - Build: darkness by place; `dark_sight_m`; burner and works ignitions; the gasworks nuisance
     rule; a Light lens.
   - Demo: lit streets beside dark ones, as found. Measured: lit hours of work and sightings by
     hour.

## 6. Open questions for the designer

1. **Traction without beasts.** Should wagonways be pushed by people, or wait for husbandry?
2. **Track geometry.** Should it be a centreline below the grid with a minimum radius, or cells
   with a turn rule?
3. **Dark sight.** Introduce `dark_sight_m` below `sight_m` now, changing every world's digests,
   or keep them equal until lamps exist?
4. **A lamp before gas.** Should a fat or oil lamp be added as vocabulary, so gas is compared with
   something?
5. **Priced carriage.** Fares and freight rates need a sale of a service. Should it be a posted
   term now, or wait for ADR-0029 §5's agreement design?
6. **Capital.** Who may own engines and lines (a household firm, an organization by stake, a
   polity), and does joint capital wait for the money brief?
7. **Household resolution.** Is a ride a block, or not taken? Does a line hold a settlement
   Detailed?
8. **Scale.** Do M10 worlds need larger maps or edges that lead off the map for rail to matter
   beyond payload?
9. **Timetables.** Should they be set by a fixed rule (headway) or revised from demand at the
   operator's review?
10. **Track and walkers.** Are lines fenced (facilities only at level crossings) or open ground?
11. **Gas physics.** Is a capacity check per district enough for M10?
12. **Bedtime.** Does light shift sleep onset, or only what scores in the evening?

## 7. What an ADR must decide

One ADR, "Rail and gas light", amending ADR-0029 (§1 modes, §2 vehicles and pullers, §3 carriers,
§6 grading built, §7 routing by composition), ADR-0019 §3 (consignments carried by runs),
ADR-0015 (sight by light) and ADR-0022 (ignitions from burners and works). It would decide:

1. **Track:** the `RAIL` mode, track spec, geometry below the grid, grading, and bridges that carry
   wheels.
2. **Rolling stock:** the `rail` vehicle mode and the engine-as-puller contract with the power
   brief.
3. **Runs:** a run as the line's event chain, block sections, loops, level crossings, nothing
   despawned.
4. **Services:** stations, handling, timetables, fares and rates, and consignments of many owners.
5. **Routing:** rail never in the walking search; rail journeys composed from station fields;
   revisions kept apart; landmarks per settlement first.
6. **Household resolution:** the contract for rides and freight.
7. **Gas:** gasworks, holders, mains and connections as records; energy-content accounting; the
   service rule and its cadence; interruptions by cause.
8. **Light:** darkness by place, sight by light, burner and works hazards, and lamplighters' work.
9. **Ownership and finance:** `build_line` and street lighting as works, firms and organizations,
   franchises as privileges.
10. **The boundary:** lines, runs, stations, mains and lit places on the wire, append only.
