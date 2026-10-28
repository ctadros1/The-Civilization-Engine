# ADR-0036: Electricity and the telegraph

Status: Proposed
Date: 2026-10-28
Milestone: M11
Amends: ADR-0033 §3 (power at a site may drive a dynamo), ADR-0034 §8 and §9 (a second carrier on
the mains' pattern; light by wire as a light source), ADR-0029 (a motor vehicle as a mode whose
speed comes from its power)

## Context

Plan §7's M11 lists electricity and a power grid, motor vehicles, transit, telecom, modern
architecture primitives and waste. Two briefs (`docs/briefs/m11-power.md`, `m11-movement.md`)
read 07-13 and 12-09 against the code. M10 already built the parts electricity needs: power at a
site by P = ρgQHη (ADR-0033), engines that burn fuel for work, a carrier laid along ways with
connections at buildings, served in order by capacity and paid by the meter (ADR-0034 §8), and
light that changes the evening (ADR-0034 §9). The reports ask for knowledge, equipment, service and
use kept apart (07-13 §1.1), power and energy apart with explicit shortage (07-13 §1.3; 12-09
§5.3), radial distribution first (12-09 §5.2), service records rather than penalties (12-09 §5.4),
and messages that travel only between offices (07-13 §1.7). Modern architecture and waste have no
report yet and are left for a later part.

## Decision

### 1. Parts

M11a electricity (generation, the wire, dispatch, light and power by wire, who builds); M11b
engines that move and transit; M11c the telegraph; M11d the demo. Modern architecture and waste
are designed when their research exists.

### 2. Generation

A dynamo is a tool good set at a site that already has power: a water installation (ADR-0033 §3)
or an engine burning fuel. Its output in an hour is the drive's power that hour times its
efficiency, capped by its rating, and nothing when its keeper's crew is short (ADR-0033's crews).
Steam turbines, alternating current, gas turbines, nuclear plant and photovoltaics are far
techniques: content may define them, nobody finds them, the observer may introduce them.

### 3. The wire

A line is a record like a gas main: cells along ways with their keepers' leave, laid by work and a
material a metre at a time, carrying once whole. It has a rating (kW) and a resistance per metre;
losses are I²R at the world's one direct-current voltage (07-13 §1.3). Lines from one station form
a tree (radial, 12-09 §5.2); there is no meshed network and no max-flow routing.

### 4. Dispatch and service

Each served hour the station's output is shared down its tree: a line passes what its rating
allows less its losses, connections are served in the order made, and one left unserved keeps its
cause (station short, line full, line broken, unpaid). In a household-resolved settlement dispatch
runs for the day's served hours together, as crews and gas evenings are met in aggregate (ADR-0032
§2). A building keeps a rolling record of power asked and delivered and its hours served. No energy
is stored and none goes negative.

### 5. Use

Light by wire is a light source at a connected building, read where light is read now (ADR-0034
§9). Power by wire feeds a recipe that already takes power, up to the connection's share. Nothing
else reads a connection: no bonus for being connected.

### 6. Who builds and pays

A household firm with a drive and the technique weighs a station and a line by what the
connections along it would repay, an anchor use (a powered recipe at its own works) counted; a
polity may build one by law as it lays a line together. Use is paid by the meter on a new ledger
channel; a connection that cannot pay is cut off until it can.

### 7. Engines and vehicles

A compact fuel engine is a tool good with shaft power and the fuel it burns; set where power is
used, it is a drive as a wheel is. A motor vehicle is a vehicle whose speed and load on a way come
from its power over its pull, the way's grade and roughness, burning fuel by its work. Transit is a
vehicle's posted runs between stops on ways, reusing ADR-0034 §4–§5's runs, timetables and fares.

### 8. The telegraph

An office is a building use with an operator who can write and knows the code; a wire between two
offices is laid as a line is. A message handed in is delivered at the other office after the
operator's queue and carried on from there by someone who walks it. It carries only what carriers
carry now (price reports, news, a polity's report) and enters the existing hearing paths.

### 9. The boundary

Stations, lines, connections, service records, offices and messages are saved entities in new
sections, empty where nobody knows the techniques; the save schema, wire and content API bump.

## Consequences

- A world where nobody knows the new techniques lives exactly as before (M10's gate clause kept).
- Whether anyone electrifies follows from what a station's connections would repay; most worlds
  will have no station, and a works may be served before any home.
- A local line can be full while its station has power to spare (12-09 §4).

## Alternatives considered

- **A coverage radius.** Cheap, but it erases capacity, losses and shortage (07-13 §1.3).
- **A meshed grid with power flow from the start.** Truer for interconnected systems, but nothing
  TCE builds first needs it, and max flow is not an electrical model (12-09 §5.2).
- **Telegraph as instant settlement-wide knowledge.** It would make an office omniscient (07-13
  §1.7).

## Revisit when

- Two stations' trees meet and people would interconnect them.
- An hour is too coarse to show an evening peak or an outage's effect.
- Far techniques are found in lived worlds often enough to need their own chains.
