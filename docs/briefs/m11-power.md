# M11 design brief: electricity and the wire

**Scope.** Plan §7's M11 ("Modern content") lists "electricity and a power grid, motor vehicles,
transit, telecom, modern architecture primitives, waste". This brief covers electricity: how it is
made, carried and used, who builds and pays for it, and what it changes. Motor vehicles, transit and
telecom are another brief's (07-13 §1.5–§1.7, §5.3–§5.4); modern architecture and waste have no M11
report yet (research index, M11) and are not covered.

**Read:** 07-13 (modern technology) §1.1–§1.4, §4, §5.1, §5.2, §5.5, §5.6; 12-09 (modern
utilities) §4, §5. Not yet read in full: 07-13 §1.5–§3 and §5.7–§6, 12-09 §1–§3 and §6 (a second
pass before the ADR). Citations are to report sections.

## What the code already has that electricity can stand on

- **Power at a site** (M10a slice DR, ADR-0033): an installation's kilowatts from its river reach
  each day by P = ρgQHη, the formula 07-13 §1.3 gives for hydropower; recipes that take power.
- **Engines that burn fuel for work** (slice DS): the atmospheric engine at a pit, by its heat.
- **A network laid by work along ways, with connections at buildings** (slice DY, ADR-0034): gas
  mains laid a metre at a time with keepers' leave, carrying once whole; homes along a main connect
  where the service costs them less than their lamp; each dusk the works serves connections in the
  order made while the main carries their rate and the works' gas lasts, and a connection left dark
  keeps its cause; paid by the meter on a ledger channel; one that cannot pay is cut off. This is
  12-09 §5.1's shape (connections on buildings, not people) and §5.4's service record, already
  built for one carrier.
- **Light changing the evening** (slice DU): work at home after dark as long as the light lasts,
  and sight after dark where light burns.
- **Firms with stations, crews and a bell** (slices DQ, DT); **laws a polity passes to build
  together** (`build_line`, `light_streets`).

## 1. What the reports ask for

1. **Separate layers** (07-13 §1.1): knowing a technique, being able to make the equipment, the
   equipment existing, service delivered, and use. Knowing gives recipes and designs; it never
   builds or equips anything by itself. TCE already works this way (vocabulary, never plot).
2. **Power, energy and availability apart** (07-13 §1.3; 12-09 §5.3): a generator's output is
   bounded by its rating, condition, staff and fuel or water; demand above supply is rationed or
   cut, never a negative store; a device needs enough power, not only enough energy.
3. **Networks grow where demand pays their fixed cost** (07-13 §1.2): an anchor customer (a mill,
   a works, a pit's pump) can justify a station before homes do (07-13 §4, Chivilingo). The same
   project is weighed by a household firm or a polity with different means.
4. **Radial distribution first** (12-09 §5.2): a tree from the station; a sweep sums downstream
   demand, a pass applies each line's limit and losses (I²R, 07-13 §1.3). No meshed grid and no
   max-flow routing.
5. **Service, not penalties** (12-09 §5.4): a building's record of power asked and delivered, the
   hours it had, interruptions and cost; consequences only through what the service does (light,
   a machine that runs or stops).
6. **Cadence** (12-09 §5.5; 07-13 §5.6): demand and dispatch every 15–60 minutes is the reports'
   prior; adoption at decision events; expansion at reviews; failures as events. Fast-forward keeps
   the evening peak (a daily profile), not an annual mean.
7. **Stylized facts** (07-13 §4; 12-09 §4): mains beside unconnected homes; production served
   before households where that pays; no universal productivity bonus per connection; service can
   stall or reverse when maintenance fails; a local overload darkens a district while the station
   has power to spare.

## 2. A first electricity, in TCE's terms

- **Generation.** A dynamo (07-13 §5.2 E03) is a tool driven by power an installation or engine
  already gives: a wheel at a fall (DR) or a fuel engine (DS) turns it. Its rating caps output;
  output is what the drive gives that day times an efficiency. A steam turbine, AC transmission,
  gas turbines, nuclear plant and photovoltaics are far techniques nobody finds (introduced by the
  observer only, as M10 decided for far techniques).
- **The wire.** A line is laid like a gas main: cells along ways with leave, by work and a
  material (copper or iron wire, a pole every few cells as a design prior), carrying once whole.
  Each line has a rating in kW and a resistance per metre; losses are I²R at the design voltage.
  Lines form a tree from one station (radial); a home connects to the line passing it.
- **Use.** Two uses first: **light** (an electric lamp at a connected building lights it as gas
  light does, without its fuel) and **motive power** (a machine whose recipe takes power, today fed
  only by a wheel at its own site, may take it from a connection instead, up to the connection's
  share). Both read only what already exists (light hours, powered recipes).
- **Dispatch.** Each hour of service the station's output is shared down the tree: each line passes
  what its rating allows less its losses, and connections are served in the order made, as gas
  mains do, those beyond what the line or station carries left dark with their cause.
- **Who builds and pays.** A household firm that holds a drive and knows the technique weighs a
  station and a line where the connections along it would repay them (DY part 4b's reckoning, with
  an anchor customer counted); a polity may build one by law as it lays a line. Use is paid by the
  meter on a new ledger channel; one that cannot pay is cut off.

## 3. Defer

Meshed grids and interconnection (E07, E10); storage and batteries; motors replacing shafts inside a
works (07-13 §1.1's reorganisation, decades long); appliances beyond light; heating and cooling;
outage causes beyond overload and an unpaid bill (correlated disasters reuse M6's hazards later);
tariffs beyond a meter price; reliability statistics.

## 4. Guards

- Nothing changes where nobody knows the new techniques: every save section's digest equal but the
  new empty ones (M10's gate clause, kept).
- No bonus to anything for being connected (07-13 §4; 12-09 §4): only light hours and power to a
  recipe that already takes power.
- No negative energy, no supply through a line not whole or not connected (12-09 §4's invariants).
- Dispatch is per hour of service and per station tree, not per person; a world with no station runs
  no dispatch at all.

## 5. Suggested slices

1. The dynamo and the electric lamp as vocabulary (goods, techniques nobody knows), a station as a
   firm's installation with output from its drive.
2. Lines laid by work along ways, carrying once whole; connections; the hourly dispatch down a tree
   with ratings and losses; the building's service record.
3. Light by wire at connected buildings; power by wire to recipes that take power.
4. Who builds: a station and line weighed by a firm with an anchor customer; a polity's law; metered
   pay and cut-off.
5. The observer: lines, stations, service and why a building is dark.

## 6. Open questions for the designer

- One voltage a world (direct current, short lines) until AC is found, or a voltage per line?
- Does a station's dispatch run hourly in Detailed settlements and as a daily profile (the evening's
  hours) in household-resolved ones, as gas evenings do now?
- Is the wire a way of its own (like track) or an attribute of an existing way (like a main)?
- Copper exists as a good (M7d); does wire need a recipe of its own, or is it a tool's material?

## 7. What an ADR must decide

The saved state of lines, stations, connections and service records (new sections, save versions);
the dispatch cadence and its place in the scheduler; the ledger channel; whether electric light
reuses gas light's evening path; and the far techniques left to the observer.
