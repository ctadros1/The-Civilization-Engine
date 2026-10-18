# ADR-0029: Ways, modes and traffic

Status: Accepted
Date: 2026-10-17
Milestone: M8b
Amends: ADR-0004 (ways with identities), ADR-0010 (an appended earthwork kind), ADR-0019 §3
(goods in transit), ADR-0028 §4 (the bound a faster mode cannot break), plan §5.4's transport row

## Context

Plan §7's M8 lists transport and traffic (road hierarchy, congestion, commuting, boats), and plan
§5.4 has walk, cart, pack animal and boat as v1's modes, with "edge capacity with time-bucketed
congestion" and "aggregate flow assignment" in Accelerated mode. The transport brief
(`docs/briefs/m8-transport.md`) reads 12-05, 12-06, 10-07, 07-08, 01-08, 11-12, 11-07 and 13-13
and finds nine points they share: a mode is a production system and not a speed bonus; a road is
a corridor, a service and a bundle of rights; congestion lives at bottlenecks; nothing vanishes in
a jam; routing is informed but imperfect; each mode has its own network; works make upkeep;
fidelity never depends on the camera or the speed; capabilities, not eras.

What the kernel has: walkers on an 8 m grid at Tobler's speed, a worn trail giving full speed and
a monthly survey tracing trails with no identity (ADR-0004); log footbridges and a polity's
crossings (M5c); gates in palisades (ADR-0023 §6); earthworks that change no one's walk
(ADR-0010, ADR-0022 §11); each river reach's daily discharge and stage (M6a slice AY); loads of
one good, at most 20 kg a person, and nothing held in transit (ADR-0019 §3). There is no wheel,
no boat and no domestic animal. M8a measured routing as about an eighth of a day at 10k
(slice CJ) and closed with the kernel some 130× from its gate.

## Decision

### 1. Ways are records once someone works on them

- Trails stay as they are: wear, the monthly survey and traced polylines are unchanged, so a
  world in which nobody works on a way keeps its digests.
- The first work on a line (clearing, laying, a passing place) or a plan's reserved corridor
  (M8c) makes a **way**: a saved line record below the grid, as a bank is (ADR-0022 §11), with an
  identity that survives surveys and loads, segments split where slope, surface, width or keeper
  change, three widths (corridor, built, usable), a surface, a grade profile, a condition, a
  keeper and the modes it serves. A trail never becomes a way by use alone.
- A kept way holds its cells' trail factor at full while its keeping lasts; untended, its wear
  fades as a trail's does.
- The **hierarchy** (lane, street, main way, way between settlements) is a label derived each
  survey from use (trips and loads by mode over the last 90 days, what the way joins) and build
  (widths, surface), shown and never read by behaviour.

### 2. Modes come from techniques and made things

- A **vehicle** is a durable good with a physical spec in content: payload (kg and volume),
  running width, rolling resistance for wheels or draft for a boat, and the puller it needs. It
  is made by a recipe, owned, stands where it was left, and wears as tools do. Using one needs no
  technique; making one does, and the technique spreads as every technique does (ADR-0008).
- **M8's modes are within reach of today's content:** the dugout (`wood_shaping`, an axe, a
  large log; a design prior of 400 kg payload) and the handcart (a wheel technique found by
  trying; 150 kg, pulled by a person). Both priors sit where 12-05 §2.3's and 07-08 §2.1's tables
  overlap and are marked so.
- **No beasts in M8.** Pack animals and ox carts need herds, fodder, breeding, training and
  death; without a husbandry design they cannot appear and nothing stands in for them. The gap is
  recorded; plan §5.4's pack animal waits for it.

### 3. Goods in transit

- A trip by boat or cart, and any trip that lasts past the day, carries its goods in a
  **consignment**: a ledger holder with an owner, a carrier, an origin and a destination, its
  goods spoiling by their own clocks and staying where the carrier stops, falls or sinks. A
  porter's hand load within a day stays in the hand, as now. This amends ADR-0019 §3 for those
  trips.
- **Mode choice is hours against hours:** the household weighs, for each vehicle it can use, the
  complete chain (fetch the vehicle, load, travel by that mode, wait, unload, return it) against
  the porter's trips the load would take, in hours of its own work.

### 4. Congestion at bottlenecks, and nothing vanishes

- **Bottlenecks are facilities:** crossings (by deck width), open gates, landings (berths and
  handling positions) and way segments narrower than their traffic. Everything else flows free,
  as today; walkers still pass through one another on open ground.
- A trip is **legs between bottlenecks**: planned free-flow at departure, its arrival at a
  bottleneck an event, passing at the facility's next free slot in its direction, first by
  arrival with a tie-break keyed by trip, the rest of the trip re-timed as a new revision
  (ADR-0003 §2). A full waiting space holds the next arrival short of it.
- **Carts meet by clearance:** on a single-cart segment opposing carts wait at a passing place or
  the end; a cart that cannot pass turns back or unloads to porters, and its household remembers
  why. A density–speed factor applies only on segments narrower than their walkers, below 0.7
  persons a square metre none.
- A trip waits, turns back, reroutes or gives up; its consignment stays owned and its hours are
  lost. Nothing despawns or is delivered unseen.
- Plan §5.4's time-bucketed edge capacity is not built: queues at bottlenecks set elapsed time,
  and what households remember of waits informs their next choice.

### 5. Boats on reaches

- A reach is **navigable today** where its stage at today's discharge exceeds the boat's draft.
  Speed over the ground is the paddler's speed plus or minus the reach's mean velocity at that
  stage; upstream where that is not positive, the boat does not go. Low water closes reaches;
  there is no season by calendar.
- Any bank cell beside a navigable reach is a bare **landing**; a built landing is a work with
  berths and handling positions. Loading and unloading are people's work at a content rate (a
  design prior near 10 kg a worker-minute).
- A boat at a bank is the first way across a river too big to wade. Its owner carries their own
  household; carrying others for a fee waits for an agreement design.
- *Amended (2026-10-18, M8b slice CO, second part):* upstream a boat keeps to the slack water
  along the banks, where it meets a share of the reach's mean velocity (a content prior per
  vehicle, `slack_share`; the dugout's 0.4); downstream it rides the mean. Measured against the
  mean alone, every generated river was closed upstream: the river valley's trunk falls 0.2-5 %
  and runs at 1.1-3 m/s at its mean flow, past any paddler's 1.2 m/s, so no boat could ever be
  brought back. With the share, 9 of 12 river valley worlds and 9 of 12 ria coast worlds tried
  have a way by water both ways between the banks nearest their two camps (in one, both camps'
  nearest bank is the same), the slower way from about as long to nine times the faster, and
  the steepest reaches stay closed.

### 6. Works on ways

- A household records, by segment or site, the hours its trips lost to grade, width, a missing
  crossing for its cart and waits, fading as wades do (`civ_agents::fords`).
- A household works on its own approach; a polity builds by an appended `build_way` policy,
  shaped as `build_crossing` and `build_bank` are (an issue raised where recorded losses would
  repay the work over its life, a forecast in each household's hours, equal shares, payable by
  `pay_work`).
- A work may lay a segment (cleared ground, shaped earth, gravel, a timber causeway, as recipes),
  widen it within its corridor, add a passing place, a crossing that takes wheels, a landing, or
  **ease a grade**. Grading is an earthwork of an appended kind (amending ADR-0010): a segment's
  target profile, volumes kept in bank, loose and compacted states, staged with the haul cycle
  earthworks already charge. The grade a cart feels is the way's own profile; routing's ground
  is unchanged.
- **Upkeep only by law,** as for banks (ADR-0022 §11): a frontage duty is one bundle such a law
  may name. **Tolls** are an appended policy on the duty machinery (ADR-0026), late in M8b.

### 7. Routing for several modes

- Walkers stay on the grid, exact, as M8a left them.
- Carts route on the way graph with grid legs from door to way, boats on the reach graph,
  directed; transfers are at landings and where a vehicle stands. Each is checked against a
  Dijkstra oracle on the same snapshot.
- Way and reach routes are keyed to the way revision and today's navigability, not to the wear
  survey.
- **The admissible bound** (ADR-0028 §4): a search's heuristic uses the fastest speed any mode in
  its state can reach, landmarks built per mode on free-flow times. Tested with a way faster than
  any trail: a bound a mode breaks is not admissible.

### 8. Accelerated mode

- Accelerated mode lives the same bottleneck events (ADR-0011 §1). If they cost too much, cohorts
  that share a leg and a five-minute departure bucket pass together, a declared approximation
  switched off in tests and judged by Gate B. No flow assignment; plan §5.4's row is amended.

### 9. The boundary

- `GetWays` with a ways revision in the snapshot (as the crossings' and the paths' revisions
  are): segments by widths, surface, condition and derived hierarchy, works in progress.
- Boats and carts drawn with their loads; landings; the Traffic lens on the coverage query (delay
  by place and hour); waits worth telling as incidents; the person inspector names the mode, the
  load and each wait.

## Slices

- **CM**: ways as records (identity, segments, three widths, keeper, kept wear), the derived
  hierarchy, saves, `GetWays` and the Ways layer.
- **CN**: bottlenecks for walkers: crossings and gates as facilities, trips as legs, first-come
  passage with waiting space, waits remembered, the Traffic lens, wait incidents.
- **CO**: boats: the dugout, navigability by stage, directed speeds, landings and handling,
  consignments in transit, ferrying one's own across, fishing, fetching and owed goods by boat.
- **CP**: handcarts on ways: the wheel technique and handcart, passability by width, grade and
  deck, the way graph with door legs, mode choice by the complete chain, clearance and turning
  back.
- **CQ**: works on ways: recorded losses, household works and `build_way`, laying, widening,
  passing places, cart crossings, landings, grading as an earthwork, upkeep by law, tolls.
- **CR**: where to live weighs the walk; the dashboard's travel time by occupation and purpose,
  mode shares and waits.

## Consequences

- Easier: goods can move in bulk where people make the means; delay is visible where it happens;
  ways become things that can be planned (M8c), built up (M8d) and fought over.
- Harder: a ledger holder in motion, queues at facilities with spillback, two more route graphs,
  an earthwork kind with three volume states. Every slice must show a world without ways, carts
  or boats keeping its digests.
- The handcart and dugout carry little against an ox cart; prices between villages may stay
  apart for want of beasts, and that is the honest result until husbandry exists.

## Alternatives considered

- **Time-bucketed edge capacity (plan §5.4).** Rejected after 12-06 §2.1: it double-counts with
  queues and lets delay live everywhere instead of at the places it happens.
- **Aggregate flow assignment in Accelerated mode (plan §5.4).** Rejected: ADR-0011 has
  Accelerated mode live the same events, and 12-06 §8 rejects swapping physics in fast-forward.
- **Ways from use alone.** Rejected: a trail used enough is still a trail until someone works on
  it, or the engine would author roads by count (10-07 §1.6).
- **A minimal beast in M8.** Rejected for now: a herd's fodder, breeding and idle support is a
  design of its own; a fake animal would carry an outcome.
- **Wet ground slowing walkers.** Deferred: it would give ways a value to walkers and seasons to
  routes, and change every world's digests; revisited with drainage.
- **Earthworks in routing's ground.** Rejected: a way's own profile carries its grade; changing
  routing's elevation would change every walker's routes for no behaviour M8 needs.

## Revisit when

- A husbandry design lands (beasts, pack animals, ox carts).
- Carts congest enough that vehicle classes and time windows are worth a law.
- Bottleneck events cost more than a tenth of an Accelerated day.
- Drainage is designed (wet ground and ways).
