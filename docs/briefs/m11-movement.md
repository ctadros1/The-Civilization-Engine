# M11 design brief: engines that move, transit and the telegraph

**Scope.** Plan §7's M11 lists "motor vehicles, transit, telecom" beside electricity (the power
brief, `m11-power.md`). This brief covers the combustion engine as portable power, motor vehicles
on the ways, transit as posted runs, and telecommunications as messages carried between offices.
Modern architecture and waste have no M11 report yet and are not covered.

**Read:** 07-13 §1.1, §1.5–§1.7, §4, §5.1, §5.3–§5.6. Not yet read in full: 07-13 §2–§3, §5.7–§6
(a second pass before the ADR). Citations are to report sections.

## What the code already has

- **Modes from techniques on ways** (M8b, ADR-0030): walking, carts and boats, ways that serve
  modes, wear by mode (M10a slice DO); wagons and locomotives on track (M10b slices DV–DX) with
  runs timed by power over pull, block sections, stations, timetables of up to four runs a day,
  freight for a tenth and fares posted at a review, and rides taken where quicker than walking.
- **Engines that burn fuel for work** (M10a slice DS, M10b slice DW): power by heat, fuel burned by
  the work done.
- **News between settlements by carriers** (M5b): price reports, news and rumour travel with people
  who walk between settlements; a settlement hears of a refusal or a default (M10d slice EI) by
  that route.
- **Letters and records** (M10c slice EC): reading and writing as techniques; records kept in homes.

## 1. What the report asks for

1. **The engine is the component, the vehicle one use** (07-13 §1.5): an engine is a good with shaft
   power, mass, fuel it takes and upkeep; a pump, a generator, a boat or a truck combines it with
   other parts. Fuels are not interchangeable across engines.
2. **A motor vehicle needs no paved road**, but its use depends on the way's roughness, grade,
   bridges, fuel and repair (07-13 §1.6). Choice of mode is a generalized cost: money, time
   (travel, wait, access), risk (07-13 §1.6), which TCE's route times and fares already price.
3. **More road does not guarantee faster journeys** (07-13 §1.6, §4): induced travel; congestion
   at bottlenecks is M8b's already.
4. **Telecom carries messages, not omniscience** (07-13 §1.7): delivery time is access + queue +
   transmission + last mile + the recipient's availability; an office in one settlement informs
   nobody who does not come to it or receive a delivery. Telephone service does not need household
   electricity (local batteries), so the telegraph must not wait for the grid.
5. **No fixed political effect of a medium** (07-13 §1.7, §4): radio or telegraph changes reach,
   never loyalty or benevolence by a coefficient.
6. **Discovery separate from adoption** (07-13 §1.1, §4): long lags; equipment can be imported where
   trade brings it; nothing equips anyone by being known.

## 2. A first M11 movement, in TCE's terms

- **The engine.** A compact fuel engine (07-13 §5.3 C02/C04) is a tool good with shaft power and a
  fuel; set at a pump, a mill or a dynamo (the power brief), it does what M10's wheels and engines
  do, by its own fuel. Refined liquid fuel (C01) is a far technique; a gas engine can burn town gas
  from M10b's holders, which exists.
- **A motor vehicle** (C06) is a vehicle like M8b's cart, with an engine instead of a hauler: its
  speed and load on a way by its power over pull, the way's grade and roughness, its fuel burned by
  the work. It is weighed as a cart is, by the hours it spares against its making and fuel.
- **Transit** is a vehicle's posted runs along ways between stops, reusing M10b's run, timetable,
  fare and ride machinery (DX) on ways that are not track; a household firm or a polity runs it.
- **The telegraph** (T01): an office in each of two settlements, a wire between them laid like a
  line (the power brief's wire), an operator who can write (EC) and knows the code. A message
  handed in at one office is delivered at the other after the operator's queue, and from there by
  someone who walks it on. What it carries is what carriers already carry (price reports, news of
  a refusal, a polity's report), so it shortens time without adding any new kind of knowledge.

## 3. Defer

Mass production (C07) and motor-traffic engineering (C08) as rule bundles; diesel and petrol fuel
chains; the telephone (T03–T05) until the telegraph shows what its offices change; radio
broadcasting (T08) and its audiences; submarine cables, transistors, fibre and cellular (far
techniques for the observer); parking and traffic rules.

## 4. Guards

- Nothing changes where nobody knows the new techniques: every save section's digest equal but the
  new empty ones.
- A message delivered changes only what its receiver heard, through the existing hearing paths; no
  settlement-wide knowledge, no loyalty or order effect of a medium.
- A vehicle's speed comes from its power, its load and the way, not a mode constant.

## 5. Suggested slices

1. The compact engine as a tool good, burning a fuel by work; set at existing power uses.
2. The motor vehicle on ways: speed and load by power over pull, fuel by work; chosen as carts are.
3. Transit: posted runs, stops and fares on ways, by a firm or a polity.
4. The telegraph: offices, a wire between settlements, an operator's queue, delivery onward, and
   what it carries through the existing news paths.
5. The observer: vehicles, runs, offices and messages.

## 6. Open questions for the designer

- Does a motor vehicle share M8b's cart record with a power field, or need a record of its own?
- Is transit on ways the same run record as a train's on track, keyed by its way?
- Does a telegraph message keep its words (as a record) or only its effect on the hearer?
- Which settlement pairs does a telegraph wire join first, and who weighs it (the offices' firm, a
  polity, or both)?

## 7. What an ADR must decide

The engine's and the motor vehicle's records and saved state; whether transit reuses the train's
runs; the telegraph's offices, wire and message queue as saved entities; how a delivered message
enters the existing news and hearing paths; and which nodes are left to the observer.
