# M8 design brief: ways, carts, boats and traffic

**Scope.** Plan §7's M8 ("The growing city (v1)") lists "**Transport & traffic**: road hierarchy, congestion, commuting, boats", "Hierarchical pathfinding" and "Road cut-and-fill and grading for the road hierarchy (earthworks)". Plan §5.4's transport row reads: "Road graph from path wear and plans; v1 modes are walk, cart, pack animal and boat; edge capacity with time-bucketed congestion; trips get travel times from congested costs; Accelerated mode uses aggregate flow assignment", with "Road building and upgrades (public works), tolls, planning, where to live relative to work" as the decisions. Plan §5.1 has organic layout run "path wear → desire lines → road graph → frontage plots", planned templates carry "road hierarchy widths", and road cut-and-fill is one of its bounded earthworks. This brief covers, for M8's 10–50k agents in 2–3 (up to 5) settlements (plan §4.3):

- ways as records with identities, grown from trails and from works people choose, and the hierarchy as a label derived from use and build;
- modes (walking, hand-hauled wheels, pack animals and carts, boats) that appear only when a technique and a making exist, and what each changes;
- loads and consignments carried by a mode, and the complete chain of a trip;
- congestion at the places it happens: crossings, gates, landings, narrow ways;
- commuting as trips that activities generate, and where to live relative to work;
- works on ways (laying, widening, passing places, cut-and-fill and grading) and their upkeep;
- routing only as far as modes and ways need it, and how Accelerated mode treats traffic.

The M8 planning brief covers the planning institution, plan templates and zoning; the scale brief covers crowd LOD, route-search performance and the "Accelerated Max at ≥1 year per minute" gate. The M8 demo is "a 30k-person planned capital next to an organic port town, with traffic, in the same world".

**What exists to build on** (checked in the code):

- **Walking** (`civ-world/src/nav.rs`, `NavGrid`, `NavParams`). Eight directions on 8 m cells. Speed is Tobler's `tobler_ms`, with `top_speed_kmh` 6.0, `slope_sensitivity` 3.5, `best_slope_offset` 0.05; unworn ground walks at `offtrail_factor` 0.6, fordable rivers at `wading_factor` 0.25, and rivers above `ford_max_discharge_m3s` 3.0 cannot be crossed on foot (`content/core/people/early_farmers.toml`, `[walking]`). "Travel time (what advances the clock) and route choice are the same thing here: the cost of a step is the seconds it takes." Speed scales by `walk_speed_by_age` only: what someone carries never slows them.
- **Routes.** A* (`route_with`) within `ROUTE_BUDGET` (600,000 cells), pulled straight, bounded by ALT `Landmarks` (`LANDMARKS` 8) round the walked area plus `LANDMARK_MARGIN_CELLS` (32); results `Found`, `Unreachable`, `BudgetExhausted`. The cache is keyed by (from cell, to cell), holds at most `ROUTE_CACHE_MAX` (50,000), and is emptied whenever `Land::route_revision` (the crossings' revision with the wear survey's) changes, so every month. Settlements keep a `TravelField`.
- **Wear and trails** (`civ-land/src/paths.rs`, ADR-0004 §4). `PathParams`: `wear_per_walk` 0.01, a 120-day half-life, trail at 0.3 until 0.15. A trip wears its cells when it ends; the monthly survey traces `Trail` polylines (points, mean wear) with no identity. ADR-0004's revisit: "People build or maintain roads deliberately, and road edges need identities of their own." Plan §9 (M5 design): "A worn trail already gives full walking speed, so a kept way changes nothing people notice until carts (M8)."
- **Crossings** (`civ-land/src/crossings.rs`). `Crossing` records, `CrossingOwner::{Household, Polity}`, `Collapse::{OwnWeight, UnderWalker, Scour}`, one system `core:bridge/log_beam` (2–8 m, deck walked at 0.6). Households weigh a log by the wades they recorded (`civ_agents::fords`, `Wade`); a polity builds by `build_crossing` with equal shares; `pay_work` (M7d) pays coin for hours on common works. "Every river too big to wade is wider than a log's 8 m" (plan §9, slice AW), so nothing crosses a trunk river.
- **Gates** (`civ-land/src/enclosures.rs`, `Gate`, ADR-0023 §6): a palisade's gate cells are ground while open.
- **Earthworks** (`civ-land/src/earth.rs`; ADR-0010; ADR-0022 §11). `EarthKind::{Platform, Pit, Spoil, Bank, Ditch}`, append-only; banks and ditches are lines below the grid; `GroundDelta` holds cell-mean deltas. `LeveeParams::hours_per_m3` charges digging, carrying a basket at a time and placing apart. Routes read `ctx.map.elevation`, the generated bed; the deltas reach only the observer's raster (`raster_response`). No earthwork changes anyone's walk today. Nobody mends a polity's bank unless a law asks (`works_wet_bank_kept`).
- **Rivers by the day.** `RiverReach` (cells downstream, order, discharge, width), `ReachHydraulics` (width, slope, Manning n 0.035, `stage(q)`) and each day's reach discharges (`reach_discharges`, M6a slice AY). Depth and current exist for a boat; no boat does.
- **Loads and trips.** `Load { good, kg, water_l }`: one good a person, at most `carry_kg` 20. `Trip { id, rev, depart, points, minutes }`; a replanned trip keeps its id with a new revision (ADR-0003 §2). Walkers pass through one another; nothing queues.
- **Trips that carry goods.** `fetch` (`max_walk_minutes` 120), the `Errand` (units "the least of a load, the units reported, those it can pay for..."), `walk_owed_goods`, `hired_work`. ADR-0019 §3: "nothing is held in transit in M5b", honest "while trips are under a day and nothing on the way can take, spoil or delay a load (no robbery, war, tolls or carts yet...)".
- **Content.** 34 goods (an axe, timber, stone among them), 21 techniques (`wood_shaping`, `jointed_frame`), wild `Game` and `Fish` stocks; fishing is "from the bank". There are no domestic animals, no wheel and no boat.
- **What the M5b demo found** (plan §9): prices between villages stayed apart because "one person carries one load", inside what carrying one unit costs on foot; it named "pack animals or carts (M8)" among what would carry more.
- **Speed today.** Three settlements of 1,000 live a year in 1,286.5 s at Max (plan §9, slice AY); 8,000 people about 89 minutes a year (slice AL), where route searches were 46 % of a day's work after each survey before landmarks.

## Where the reports agree

Nine points recur across 12-05, 12-06, 10-07, 07-08, 01-08 and 11-12. I treat them as invariants.

1. **A mode is a production system, not a speed bonus.** Carts and boats "primarily change payload, not necessarily speed"; a canoe moved about a tonne with one paddler "at roughly walking speed" (12-05 §1.3). Quickness, cheap heavy loads and reliable delivery are different achievements (07-08, design conclusion); compare worker-days per tonne-km (07-08 §2.3).
2. **A road is a corridor, a service and a bundle of rights, not a technology level** (10-07, opening). Function, physical design and authority are separate attributes; "Its physical standard changes only when someone can organize the work" (10-07 §1.5). Infrastructure is built when its benefits justify it "to an actor able to finance and organize it" (07-08 §1.7).
3. **Congestion lives at bottlenecks.** It arises from "conflicting uses, narrow passages, vehicle clearance, and slow handling" (12-05, opening); spend fidelity on gates, bridges, loading areas and market entrances (12-06 §7.2); intersections and gates need their own queues (10-07 §1.9). The signature is "concentration of delay at particular locations and times" (12-05 §4.1).
4. **Nothing vanishes in a jam.** Closing a gate "must not erase cargo, complete deliveries remotely, or free a worker who remains physically occupied" (12-05 §4.2); "never resolve a jam by silently converting a blocked delivery into a completed delivery" (12-06 §5).
5. **Routing is informed but imperfect.** A few familiar routes, replanning on closure or persistent delay, hysteresis (12-05 §5.3; 01-08 §4); "do not silently give every citizen a perfect live congestion map" (12-06 §3); agents need not know new roads instantly (10-07 §5.3).
6. **Each mode has its own network and access.** Steps serve walkers and not carts (10-07 §1.2); routing state is (location, mode, vehicle/access state), and "A cart cannot materialize at a remote destination" (01-08 §5); waterways are directed, seasonal edges (07-08 §1.3).
7. **Works are construction, and construction makes upkeep.** Road projects are task chains with materials and haul (10-07 §5.4); earthworks keep soil, labour and water accounts (11-12, opening; §1.2); a settlement that expands roads with fixed maintenance labour "should accumulate backlog" (10-07 §4 G).
8. **Fidelity is independent of camera and speed.** "Looking away from a bridge must not increase its economic capacity" (12-06 §8); the same journey takes the same time visible, off-screen or fast-forwarded (12-05 §4.2; 10-07 §5.5).
9. **Capabilities, not eras.** Dates are "historical evidence or reference ranges, never simulation gates" (07-08 §5.2); behavioural regimes, not calendar unlocks (12-05 §3.1); capability nodes with alternative prerequisites (11-12 §5.4).

**What games and models teach to avoid:** despawning stuck vehicles and teleporting their goods (12-06 §5, TM:PE's account of Cities: Skylines 1); SUMO's jam fallback that lets pedestrians through obstacles, "unacceptable as an invisible solution" (12-05 §5.5); freight departing without valid destinations (12-06 §5, SimCity Update 7); pedestrians passing through one another (12-06 §6, A/B Street); BPR as authoritative congestion physics (12-06, recommendation; §2.1); swapping to independent BPR delays in fast-forward (12-06 §8); mixing a presentation calendar with physical time (12-06 §5, Transport Fever 2); making cobblestone "intrinsically faster than every alternative" (10-07 §1.4); a universal "pave at X travelers/day" threshold (10-07 §1.6); a universal collar multiplier (07-08 §6).

**Where they differ, or differ from the plan:**

- **Congestion model.** Plan §5.4 has "edge capacity with time-bucketed congestion; trips get travel times from congested costs". 12-06 recommends finite-storage queues and explicit intersections, keeps BPR for "initial route costs" and coarse proposals only (§2.1), and warns that counting both is double counting. I follow 12-06: queues at bottlenecks set elapsed time; bucketed expectations only inform route choice.
- **Accelerated mode.** Plan §5.4 says it "uses aggregate flow assignment". ADR-0011 now has Accelerated mode advance "a day at a time through the same events", and 12-06 §8 makes cohort movement under "the same congestion rules" the first fast-forward mechanism, rejecting a switch to BPR. I follow ADR-0011 and 12-06.
- **Hierarchy.** Plan §5.1's planned templates carry "road hierarchy widths". 10-07 §1.5 has hierarchy "emerge from function before it becomes a design standard", and §4 A finds importance, width and paving "imperfectly correlated". A plan can reserve corridor widths; the hierarchy shown is derived.
- **Pack animals.** Plan §5.4 lists pack animals and carts among v1 modes. There is no livestock in content (M5 trade brief), and 07-08 §3 asks species availability, training, fodder and equipment to decide, "not a generic 'animal husbandry' unlock". 12-05's bottom line: "Add animals and carts when their physical and economic prerequisites exist."
- **Payloads.** 12-05 §2.3 proposes an early handcart at 150–300 kg and an early animal cart at 300–800 kg; 07-08 §2.1 gives a handcart or wheelbarrow 80–180 kg and an early wooden ox cart 0.3–1.0 t. Porters: 20–30 kg (12-05 §2.3) against 15–30 kg (07-08 §2.1). All are design priors.
- **Grade.** With rolling resistance 0.02 a 5 % ascent needs about 3.5 times the level pull (10-07 §1.2); with 0.03, about 2.7 times (07-08 §1.2). Both are illustrations.
- **Commuting.** 07-08 §1.8 uses "the person's daily time budget" (30 minutes one way at 4 km/h is about 2 km). 12-05 §1.1 reads Marchetti's constant as a hypothesis, and §2.3 allows 45–90 minutes a day only as "an optional soft-cost experiment, never a compulsory allocation or hard cap". I follow 12-05.
- **The accelerator.** 01-08 names CCH "the first advanced accelerator to evaluate" but finds a portal hierarchy "easier to adapt to frequent new roads and settlements" (executive recommendation); §8 adds a two-level overlay only "where whole-network searches remain costly". The scale brief decides; this brief asks only what modes need (§1.9).

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| A way | A line record: segments with centreline, three widths, surface, grade profile, condition, keeper, permitted modes | Work someone does on a line, or a plan's reserved corridor |
| A vehicle or boat | A durable good with a physical spec (payload kg and volume, running width, rolling resistance or draft); where it stands | A recipe gated by a technique |
| A consignment | A ledger holder: goods, owner, carrier, origin, destination | A trip that carries by a mode, or lasts past the door |
| A bottleneck | Per facility and direction: discharge capacity, waiting space, who waits in order | Crossings, gates, landings, ways narrower than their traffic |
| A landing | A bank site: berths, handling positions, owner | Built as a work; a bare bank otherwise |
| Delay remembered | Per household, by place: waits and refusals (too narrow, too steep, too shallow), fading | Its own trips |
| A grading work | An earthwork of an appended kind on a way segment: target profile, volumes bank/loose/compacted, stages | A household's or a law's choice |

**Derived, not saved:** trails (as today); the hierarchy label (lane, street, main way, way between settlements); traffic counts by segment and hour; the traffic lens; travel-time distributions; delay incidents.

**Size (my arithmetic).** 5,000 way segments for a 30k town at an assumed 128 B each is 0.64 MB. Consignments in transit at any time number in the hundreds. 01-08 §7's arithmetic, 50,000 routes of 100 four-byte edge IDs in 20 MB, bounds habitual routes if they are kept.

### 1.1 Ways: from trails to records

**Reports.**

- Keep trail definition apart from surface damage; one variable lets traffic improve a dirt road "forever" (10-07 §1.1).
- Store three widths, legal corridor, constructed and usable; widening beyond the corridor needs negotiation, purchase or encroachment, and "should not silently delete adjoining buildings" (10-07 §1.7).
- Split segments "at intersections and meaningful changes in slope, surface, drainage, responsibility, or width"; do not tie a path's width to the cell size (10-07 §5.1; 11-12 §5.1's sub-grid features).
- The renderer needs a simulation graph with "Stable edge/node IDs", a geometric specification and a render cache, exported as versioned deltas (13-13 §2.1, §6).

**My proposal.**

- **Trails stay as they are.** Wear, the monthly survey and traced `Trail` polylines are unchanged, so worlds without works keep their digests.
- **A way is a record once someone works on it.** The first work on a line (clearing, laying, a passing place) or a plan's reservation makes a way record with an identity: a line below the grid like a bank (ADR-0022 §11), segmented as 10-07 §5.1 says. This meets ADR-0004's revisit trigger.
- **A kept way holds its cells' trail factor at full** while its keeping lasts (the M5 settlements brief's "its wear held up"); untended, it fades as wear does.
- **A planned street is a legal corridor first.** A plan's grid lays corridors of a width with nothing constructed; buildings are kept off them as `works_wrought_spacing` keeps them apart, binding new work only. Building one is a work like any other.
- **The hierarchy is a label.** Derived each survey from function (trips and loads by mode over 30–90 days, 10-07 §2.4's observation window; what it joins) and from build (widths, surface), shown apart, never read by behaviour.

### 1.2 What a way is worth, by mode

**Reports.**

- Tractive force ≈ Mg(C_rr + s); agents should "sometimes choose a longer contouring route, reduce loads, add animals, or transfer goods between modes instead of requesting a steeper road's 'speed upgrade'" (10-07 §1.2). A preferred sustained grade for loaded carts of 3–5 % is a prior (10-07 §2.4).
- Corridor priors: footpath tread 0.6–1.2 m, pack path 1.5–2.5 m, single cart 2.5–3.5 m, two carts passing 4.5–6 m (10-07 §2.4).
- "A structurally adequate bridge may still be unsuitable for carts" (11-07 §1.4); a loaded cart is a 20 kN gross load case (11-07 §2.2).
- A road ending at an impassable river "may accomplish little" (07-08 §1.7). Drainage and season change the ranking of routes (10-07 §1.3, §4 C).

**What the code implies.** A walker on a worn trail is already at full speed, and wet ground does not slow anyone. A way changes nothing for walkers; it matters for wheels, for grade and for crossings.

**My proposal.**

- **Walkers keep the grid.** Their routes, speeds and wear are as today; a kept way is a trail that does not fade.
- **Wheels need a way or a worn trail.** A wheeled vehicle moves on segments whose usable width takes its running corridor and whose grade its puller can haul at its load, by 10-07 §1.2's force against a pull the content authors (a design prior); on a trail never worked, at a rough-ground rolling resistance. It crosses only where the deck's width and capacity take its gross load, which no log footbridge does.
- **Wet ground is an open question** (§5, question 2). Slowing off-way walking by soil water would give ways value to walkers and seasons to routes, and would change every world.

### 1.3 Modes from techniques and makings

**Reports.**

- Node semantics: knowledge, material, skills, facilities, environment, access, and what each unlocks; AND/OR prerequisites (07-08 §5.2). L05 wheel-and-axle needs "human OR animal traction"; L11 is wheelbarrows and handcarts; W01 rafts; W02 dugouts need "Large timber + adze/axe work" (07-08 §5.3, §5.4).
- "Water transport must be available without farming, metal tools, cities, or states" (07-08 §3, the Pesse dugout). Animals are "a living productive asset with maintenance requirements" that "require support while idle" (07-08 §1.5).
- The smallest useful set is "porters, locally available pack animals, an ox-cart family, dugouts, barges, and sailing coasters" (07-08 §5.6).

**My proposal.**

- **A vehicle is a durable good with a spec**, made by a recipe, owned, kept at a place, wearing as tools do: payload in kg and volume (07-08 §1.1), running width, rolling resistance or draft. One cart owned is one cart: it is where it was left.
- **Within reach of today's content.** A dugout (`wood_shaping`, an axe, a log of large diameter from woodland) and a raft; a handcart or wheelbarrow behind a wheel technique found by trying (ADR-0008), pulled by a person. Priors from the tables above; which table is open question 10.
- **Pack animals and ox carts need beasts.** A beast needs a herd, grazing or fodder, breeding, training and death. Without a husbandry design they cannot appear, and nothing should fake one (open question 1).
- **A mode spreads as techniques spread**: by sight and teaching (ADR-0008), carried between settlements by people. Using a vehicle needs no technique; making one does.

### 1.4 Loads, consignments and the complete chain

**Reports.**

- Freight comes from inventories: Q_order = max(0, I_target − I_available − I_committed inbound) (12-05 §1.2). Compare "the complete chain: Collect → load → travel → wait → unload → transfer or store → complete delivery", with empty returns (12-05 §1.2, §1.3).
- Moving 1 tonne is "40 loaded trips at 25 kg, two at 500 kg, or one at 1,000 kg", plus empty returns and handling (12-05 §4.2).
- Illustrative labour: a porter 2.00 worker-days per t-km (4.00 with an empty return), an ox cart 0.063 (0.125), a canoe with two 0.133 (0.267) (07-08 §2.3, all P).
- Keep person-trips, person-km, tonnes, tonne-km and labour-hours apart (12-05 §4.2).

**My proposal.**

- **A trip may carry a consignment.** Above a hand load, or on a trip that can be delayed, held up or outlast the day, the goods sit in a consignment holder of the ledger, owned throughout, spoiling by their own clocks, staying where the carrier stops or falls. This amends ADR-0019 §3 for those trips (ADR, §6).
- **Mode choice is hours against hours**, the unit of every TCE choice: the household weighs, for each mode it has, the walk to fetch the vehicle, loading, the way there by that mode, waiting, unloading and bringing it back, against the porter's trips the load would take.
- **Where carts pay (my arithmetic).** At 2,100 kcal a person a day (`daily_kcal_per_person`) and 3,340 kcal a kg of grain, a person eats about 230 kg of grain a year if all of it were grain; 30,000 people, about 6,900 t. Brought in over 30 days that is about 230 t a day, 11,500 porter loads of 20 kg, or about 1,150 handcart loads of 200 kg. Harvest, firewood and building stone are the bulk flows; tools are light, and carts change little for the M5b tool trade.

### 1.5 Boats, reaches and landings

**Reports.**

- Waterways are directed and seasonal; current alters speed over the ground; a downstream route "may permit cheap rafting but no economical return of the same craft" (07-08 §1.3).
- Navigable capacity "should vary with water and weather" (12-05 §3.2, Tenochtitlan). A dugout carries 0.1–0.7 t at 3–6 km/h (07-08 §2.1).
- A landing service takes t_setup + Q/(n r); "More boats do not increase throughput when the landing is labor-constrained"; "an empty landing without workers must not function like a fully staffed warehouse" (12-05 §1.2, §1.5). Priors: setup 2–5 minutes a stop, 5–20 kg a worker-minute (12-05 §2.3).
- Water beats land only past D* = H/(c_road − c_water), H its extra handling (07-08 §4). A route a traveller knows differs from one that exists (07-08 §5.1).
- Boats are "recoverable capital" (11-07 §1.5).

**My proposal.**

- **A reach is navigable today** where its stage at today's discharge (`ReachHydraulics::stage`, `reach_discharges`) exceeds the boat's draft, a content prior per boat. Speed over the ground is the paddler's speed plus or minus the reach's mean velocity at that stage; upstream where that is not positive, the boat does not go. Low water closes reaches, floods make them dangerous: no calendar.
- **Landings.** Any bank cell beside a navigable reach is a bare landing; a built one (a cleared bank, a jetty) is a work and a record with berths and handling positions. Loading and unloading are people's work at 12-05 §1.5's rate.
- **Ferrying.** A boat at a bank is the first way across a river too big to wade. Its owner carries its own household; carrying others for a fee is an appended agreement, later.
- **First uses come from what people do already**: fishing beyond the bank, fetching and errands along the river between settlements, owed goods carried downstream. A boat must be brought back, or left.
- **Illustration (my arithmetic).** Loading and unloading 1 t by one person at 10 kg a minute takes 200 minutes, about 0.4 eight-hour worker-days. Against a porter (2.00 worker-days a t-km) and a canoe (0.133), D* ≈ 0.4 / 1.87 ≈ 0.2 km. Against porters a loaded boat wins on almost any trip; depth, direction and owning a boat decide.

### 1.6 Bottlenecks and congestion

**Reports.**

- Two mechanisms: pedestrian crowding, where density lowers speed, and clearance failure, where "a cart cannot pass another cart" even at low density (12-05 §1.4). Weidmann's curve is relatively unconstrained below about 0.7 persons/m² and peaks near 1.2 persons/(m·s) (12-05 §2.1).
- A 2 m clear passage carries about 8,640 people an hour; one unloading position occupied ten minutes serves at most six vehicles an hour (12-05 §4.2). Finite queue space: waiting "must extend upstream rather than accumulate inside an invisible point" (12-05 §1.5).
- Between bottlenecks, expected speed and scheduled arrivals; at bottlenecks, actual finite space and service; a stalled encounter "must not end through invisible overlap" (12-05 §5.2).
- Keep storage and flow capacity apart; earliest exit = entry + L / v_free; no unlimited accumulation of unused discharge capacity; stable ordering and bounded retries; a single-threaded reference first (12-06 §7.1, §7.3).

**My proposal.**

- **Bottlenecks are facilities, not every link:** crossings (deck width, one file or two), open gates, landings (berths, handling positions), and way segments narrower than their traffic. Everything else is free-flow, as today.
- **A trip is legs between bottlenecks.** At departure a trip is planned free-flow as now; its arrival at a bottleneck is an event; it passes at the facility's next free slot in its direction, first come by arrival with a keyed tie-break, and the rest of its trip is re-timed as a new revision (ADR-0003 §2 already allows this). A full waiting space blocks the approach: the next arrival waits short of it.
- **Carts meet by clearance.** On a single-cart segment, opposing carts wait at passing places or the end; walkers pass them where width allows. A cart that cannot get through turns back or unloads to porters (12-05 §4.2's mode substitution), and the household remembers why.
- **Crowding only where it binds.** A density–speed factor on segments narrower than their traffic, from counts of walkers on them in 5-minute buckets; below 0.7 persons/m², none.
- **Nothing despawns.** A trip waits, turns back, reroutes or gives up, with its consignment still owned, and its hours are lost.
- **Will anything congest? (my arithmetic).** The 11,500 harvest loads of §1.4 over 12 daylight hours are 16 a minute, a ninth of a 2 m passage's 144. Walkers will rarely queue except at one-file crossings; carts at landings and single-cart ways will. Villages of the M5–M7 demos should show no queues at all, and say so.

### 1.7 Commuting and where to live

**Reports.**

- Generate activities, not standardized commutes: household workshops, separate workplaces, mobile occupations, provisioning, seasonal work (12-05 §1.1). At 1.2 m/s, 15 minutes is 1.08 km, and the thirty-minute distance is no maximum city radius (12-05 §1.1).
- Connect repeated transport costs to "residence, shop location, warehouses, road improvements, and new markets", their strengths calibrated, not assumed (12-05 §5.3). Measure "Full distributions of daily travel time and distance, separated by occupation and trip purpose" (12-05 §4.1).

**My proposal.**

- **No commute is authored.** Trips are what activities ask (fields, water, wood, hired work, fetches, visits), as today. A planned capital with fields beyond its streets makes long daily walks by its layout alone.
- **Where to live weighs the walk.** The yearly residence review (ADR-0018) adds the household's recorded hours walking to its own places over the past year, in hours of its own work, beside what it weighs now.
- **Measured, never capped.** The dashboard shows each settlement's daily travel time by occupation and purpose; 12-05 §2.3's 45–90 minutes is a check, not a rule.

### 1.8 Works on ways: laying, widening, cut-and-fill and grading

**Reports.**

- Candidate projects come from "repeated experienced problems": wet-season interruptions, high hauling effort, opposing carts, market obstruction, strategic vulnerability, prestige (10-07 §1.6). Evaluate against benefits, construction and upkeep, land and alternatives; the beneficiaries may not control the resources (10-07 §1.6). "There is no defensible universal 'pave at X travelers/day' threshold" (10-07 §1.6).
- Survey → clear → strip topsoil → excavate → load → transport → spread → compact → finish (11-12 §1.2); Q_crew = min of the stages, so extra diggers make queues (11-12 §1.2). Conserve dry soil mass across bank, loose and compacted volumes (11-12 §1.1); priors: non-metal digging 1.0 m³ bank a worker-day (0.5–2.0), loose/bank 1.20, compacted/bank 0.90, 3 m³ compacted spread and tamped a day (11-12 §2.4).
- A full-bench road cut is V ≈ L s W²/2: "A wider road or terrace becomes disproportionately expensive where it demands deeper cutting" (11-12 §1.5). A 1 km × 4 m × 0.15 m course is 600 m³; widening 3 to 5 m adds 67 % (10-07 §2.3).
- Maintenance: routine, periodic and emergency; "A public road with no functioning payer or labor obligation should accumulate a maintenance backlog"; corvée "is not free" (10-07 §1.8). English statute labour was 6 days a year, "not evidence of complete compliance" (10-07 §2.2). Road cuts slump, wash out and block their drains (11-12 §4.1).

**My proposal.**

- **Problems are recorded where they happen,** as wades are (`civ_agents::fords`): a household keeps, by segment or site, the hours its trips lost to grade (extra pull or a detour), width (waiting, turning back), a missing crossing for its cart, and waits at bottlenecks, fading by the same half-life.
- **Who builds.** A household its own approach; a polity by an appended `build_way` policy, shaped like `build_crossing` and `build_bank`: an issue raised where recorded losses at one site would repay the work over its life, a forecast in each household's hours, equal shares, payable by `pay_work` where coin exists. The planning institution's plan is the other route (planning brief).
- **What a work can be:** clear and lay a segment (surface: cleared ground, shaped earth, gravel, timber causeway, as recipes, 10-07 §1.4), widen it within its corridor, add a passing place, a cart-worthy crossing (a bridge system with a deck for wheels), a landing, or ease its grade.
- **Grading is an earthwork of an appended kind.** A way segment's bed: centreline, target profile, width, side slope; volumes in bank, loose and compacted states; staged as 11-12 §1.2 has it, with the haul cycle `LeveeParams::hours_per_m3` already charges (11-12 §1.3). The grade carts feel is the way's own profile, not the 8 m cells'; the cell-mean deltas go to the raster and the water, as banks' do. Whether routing's elevation gains the deltas is open question 6.
- **Illustration (my arithmetic).** A 3 m bench along a 10 % side slope is 100 × 0.10 × 3² / 2 = 45 m³ of cut per 100 m; at TCE's 8 hours a bank cubic metre (ADR-0010's digging prior, 11-12 §2.4's 1.0 m³ a day) that is 360 hours, about 45 eight-hour days, before compaction. A 5 m bench cuts 25/9, about 2.8 times as much.
- **Upkeep by law only.** As for banks, nobody keeps a polity's way unless a law asks (ADR-0022 §11); a frontage duty (12-05 §1.6's *Tabula Heracleensis* assigns street maintenance to frontage owners) is one bundle such a law may name. Condition: roughness and ruts from wheels on wet days, material lost, obstruction; repair is work done with inputs.
- **Tolls are a law and an income** (07-08 §1.6: "A toll is income to an institution, not destroyed labor"): an appended policy charging a crossing or way per load, on ADR-0026's duty machinery (its revisit names "tolls along routes"). Traffic answers by route and mode (11-07 §4.2).

### 1.9 Routing what transport needs

**Reports.**

- ALT's landmark tables must use an optimistic lower bound; "Adding a faster connection, increasing maximum travel speed, or changing mode permissions can invalidate them" (01-08 §2A).
- Separate physical networks with explicit transfer links; walking "must remain traversable before its road network is complete" (01-08 §5). Roads are a sparse graph; walking across fields may want a separate terrain hierarchy (01-08 §2F).
- Separate elapsed time from generalized cost (01-08 §4). Keep a habitual route unless invalid or a 10–20 % improvement (a parameter to try) appears; feasibility and quality invalidation are distinct (01-08 §4).
- Cache by "direction, mode, access permissions, cost profile, departure bucket, and relevant graph versions" (01-08 §4).

**What the code implies.** The A* heuristic is straight-line distance at walking's top speed. A canoe downstream at 3–6 km/h plus a current can beat 6 km/h, so a boat leg breaks admissibility unless the bound takes the fastest mode. The route cache empties every month even where nothing a route used has changed.

**My proposal.**

- **Walkers stay on the grid**, exact; the scale brief owns how searches get cheaper.
- **Carts route on the way graph,** with grid legs from door to way; boats on the reach graph, directed; transfers at landings and where a vehicle stands. Each is checked against a Dijkstra oracle on the same snapshot (01-08 §8).
- **Way and reach routes are keyed to the way revision** (works opening or failing, permissions changing) and today's navigability, not to the monthly wear survey.
- **The heuristic takes the fastest mode in a search's state;** landmarks are built per mode on free-flow times.
- **Congestion enters choice as memory.** A household's recorded waits by place feed its next choice of route, time or mode; nobody sees queues they have not met or been told of.

### 1.10 Accelerated mode

**Reports.** Three levels: individual mesoscopic, cohort/platoon under the same rules, strategic long-horizon (12-06 §8). Preserve "count, membership, destination composition, queue position, travel age, and cargo"; cohort sizes 1, 5 and 20 are experiments (12-06 §8). "Increasing the time step arbitrarily is not a valid fast-forward strategy" (12-06 §2.3).

**My proposal.** Accelerated mode lives the same bottleneck events (ADR-0011 §1, §4). If they cost too much, cohorts that share a leg and a 5-minute departure bucket pass a bottleneck together, as a declared approximation switched off in tests and held to Gate B. No flow assignment; plan §5.4's row is amended.

## 2. What to defer, and why

| Defer | To | Why |
|---|---|---|
| Pack animals and ox carts | When a husbandry design exists (open question 1) | No livestock; animals need herds, fodder and idle support (07-08 §1.5) |
| Barges, towing, sailing coasters, sea routes | M9 or a port design | Towpaths, wind tables and crews (07-08 §1.3, §2.1) |
| Ferry and carrying services as firms | After household boats | 07-08 §5.1's transport services need owners who hire and operate |
| Handling rights at landings | With guilds and privileges in a port | 12-05 §1.2: rights "inheritable and separable from the people performing" |
| Time-window access laws (vehicle class, hours) | After carts congest | 12-05 §1.6's rule tuple needs traffic to act on |
| Full drainage networks for ways | With drains (ADR-0022 §11) | 10-07 §1.3 |
| Time-dependent routing, CCH | Scale brief | 01-08 §4, §8 Stage 3 |
| Litters, riding, couriers, relays | Later | 12-05 §1.3; 07-08 §1.8 |
| A traffic or road god tool | Never | It would choose an outcome |

## 3. Risks and validation

**Plot.** No date, era or population opens a mode or a road; no way is authored; no mode carries a speed bonus by name; no road appears for a count of travellers (10-07 §1.6); no despawn or teleport (12-06 §5). If nobody makes a boat or a cart within its time box in a world that knows how, nudge a prior (what a household believes a load saves) and log a `NUDGE:`, never a vehicle (plan §1, rule 3).

**Stability.** Carts raise bulk flows; cheaper carrying can thin a village's food store into a town's, and the latest fifty-year dashboard still fails its food-price row in one world (README). Measure before tuning: trips by mode, loads, tonne-km, waits, refusals by reason, works proposed and passed.

**Stylized facts, as plausibility checks:**

- Delay concentrates at particular places and times, not citywide (12-05 §4.1).
- Person-trips stay overwhelmingly on foot while carts and boats carry most tonne-km (12-05 §4.2).
- Better cart access cuts labour for a heavy shipment and may add obstruction and a porter's last leg (12-05 §4.2).
- Water makes places economically close; downstream is cheaper than upstream (07-08 §4, the 1:5:10:52 ordering as a check, never imposed).
- Fixed handling keeps short hauls on land (07-08 §4's D*).
- Importance, width and paving correlate imperfectly; surfaces become patchworks (10-07 §4 A, B).
- A small crossing's closure sometimes matters more than kilometres of road (10-07 §4 D); more ways mean more backlog (10-07 §4 G).
- Longer hauls lower a work's daily output (11-12 §4.4).
- After a route improvement, price gaps for carried goods narrow where supply allows (07-08 §4): the M5b convergence row's test.
- The same journey takes the same time in Detailed and Accelerated modes (12-05 §4.2).

**Headless scenarios first** (12-06 §10): a single bottleneck, two merging streams, opposing flow through a narrow passage, a short segment holding a long cart, a gridlocked loop, a crossing failing during travel; each checked for conservation ("No lost people, duplicated cargo, negative occupancy, or unexplained deliveries") and against hand-worked times.

**Observer.** A Ways layer (wire `GetWays`, with a ways revision in the snapshot as `paths_rev` and the crossings' revision are): segments by width, surface and condition, the derived hierarchy, works in progress, cuts and fills in the raster. A Traffic lens on the coverage query (as the Water lens): delay by place and hour. Waits worth telling as incidents ("Nine waited 14 minutes at the ford landing"). The person inspector names the mode, the load and each wait; the household panel what its trips cost by mode. The kernel exports what 13-13 §6 asks: stable IDs, centrelines, elevation profiles, widths, surface, condition and bridge relationships as deltas.

**Compute (my arithmetic).** At linear cost, 50k people would take about six hours a year at today's 1,286.5 s for 3,000; plan §4.4 asks a minute. At a year a minute a day lasts 0.16 s, so 01-08 §1's four legs a person a day for 50k is about 1.2 million trip starts a real second (by 12-06 §1's definition, R = 525,600); at 01-08 §1's illustrative 100 μs a fresh search, even 99 % reuse leaves 1.2 core-seconds a second. Transport must add events, not searches: legs at bottlenecks cost an event each; way and reach graphs are small; a world with no ways, carts or boats must give the build-before's digests.

## 4. Suggested slices

1. **Ways as records.** The way record, its identity and three widths; ways from first work and from reserved corridors; a kept way's wear held up; the derived hierarchy label; save, load and `GetWays` with the Ways layer. Tests: identities stable over surveys and loads; a world without works keeps its digests.
2. **Bottlenecks for walkers.** Crossings and gates as facilities; trips as legs; first-come passage with waiting space and re-timed revisions; waits remembered; the Traffic lens and wait incidents. Tests: 12-06 §10's scenarios, conservation, Gate A, Gate B; the M5–M7 demo worlds report their waits (expected none).
3. **Boats.** A dugout technique, recipe and good; navigability by today's stage, directed speeds; bare and built landings with handling time; ferrying across rivers too big to wade; consignments in transit (amending ADR-0019 §3); fishing, fetching and owed goods by boat. Tests: no boat on a reach too shallow, upstream slower, goods conserved through a sinking or a stranded boat. Observer: boats on the river, landings, a boat's load.
4. **Handcarts on ways.** A wheel technique, a handcart good; cart passability by width, grade and deck; carts on the way graph with door legs; mode choice by complete-chain hours; clearance at single-cart segments and turning back. Tests: a 5 % grade costs what 10-07 §1.2 says at the content's rolling resistance; a cart never crosses a log. Observer: carts and their loads.
5. **Works on ways.** Recorded losses by site; `build_way` and household works; laying, widening, passing places, cart crossings, landings; grading as an appended earthwork kind with balanced volumes and crew bottlenecks; upkeep only by law; tolls on the duty machinery. Tests: earth balances (11-12 §1.1), a graded segment lowers the cart's recorded loss, a neglected way degrades.
6. **Where to live, and travel time measured.** The residence review weighs recorded walking; the dashboard's travel-time distributions by occupation and purpose.
7. **The demo's traffic.** With the planning and scale briefs' slices: the capital's corridors built over time, the port town's trails kept, the river between them carrying loads, and the waits reported as found.

## 5. Open questions for the designer

1. **Beasts.** Add a minimal domestic animal in M8 (herd, grazing or fodder, breeding, training, death) so pack animals and ox carts can exist, or leave them content-ready and absent? Without them the only wheels are hand-pulled.
2. **Wet ground.** Should off-way walking slow with soil water, giving ways a value to walkers and routes a season, at the cost of every world's digests?
3. **Congestion.** Legs and queues at bottlenecks, as proposed, or plan §5.4's time-bucketed edge capacity?
4. **Accelerated mode.** The same events with optional cohorts (ADR-0011, 12-06 §8), or plan §5.4's aggregate flow assignment?
5. **Ways from use.** Should a trail carrying enough loads become a way record by itself, or only when someone works on it?
6. **Ground and routing.** Should routing's elevation gain earthworks' deltas (platforms, pits, banks, grades), or should a way carry its own profile?
7. **Upkeep.** Law-only keeping, as for banks, with frontage duties as one bundle a law may name?
8. **Transit.** Consignment holders for every carrying trip, or only for boats and carts?
9. **The port.** Does the demo's port town need sea and sailing, or does a navigable trunk river with dugouts suffice?
10. **Payload priors.** 12-05 §2.3's or 07-08 §2.1's tables?
11. **Tolls and access.** Tolls in M8, and vehicle-class time windows (12-05 §1.6) as a policy kind now or later?
12. **Measures.** Which waits, mode shares and travel-time distributions go on the dashboard, and with what tolerances?

## 6. What an ADR would need to decide

One ADR, "Ways, modes and traffic", amending ADR-0004 (ways with identities), ADR-0010 (an appended earthwork kind), ADR-0019 §3 (goods in transit) and plan §5.4's transport row. It would decide:

1. **Ways:** a saved line record with identity below the grid, three widths, segments, keeper and permitted modes; how it relates to trails and wear; reserved corridors.
2. **Vehicles and boats:** durable goods with a physical spec, owned and located; modes gated by techniques only; whether beasts exist.
3. **Consignments:** a ledger holder for goods in transit, its owner, spoilage and fate when a carrier stops.
4. **Bottlenecks:** which facilities queue, trips as legs, the passage rule (first by arrival, keyed ties), waiting space and spillback, no despawn, delays remembered per household.
5. **Navigation:** navigability from each day's stage, directed speeds, landings and handling.
6. **Grading:** the earthwork kind, volumes in three states, the way's own profile, and what routing's elevation reads.
7. **Routing:** separate networks per mode with transfers, the admissible bound for the fastest mode, route revisions by graph and not by survey.
8. **Accelerated mode:** the same events, with cohorts as a declared approximation.
9. **The boundary:** `GetWays`, the ways revision and its deltas, wait incidents and the Traffic lens.
