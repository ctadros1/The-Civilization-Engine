# ADR-0034: Rail and gas light

Status: Accepted
Date: 2026-10-22
Milestone: M10
Amends: ADR-0029 §1, §2, §3, §6 and §7 (a rail mode, engines as pullers, runs as carriers, track
graded and built, journeys composed from stations), ADR-0019 §3 (consignments carried by runs),
ADR-0015 (sight by light), ADR-0022 (ignitions from burners and works); carries out ADR-0028 §4
(landmarks per region, decided and never built)

## Context

Plan §7's M10 lists "rail, gas light". The brief (`docs/briefs/m10-rail-gas.md`) reads 07-08,
12-09, 12-05, 12-06, 12-03 and 12-04 against the code. Its findings:

- **Routing is the cost already.** ADR-0028 §4 decided landmarks per region, but the code builds
  one set over the bounding box of all walked ground and rebuilds it whenever walking costs change
  anywhere: 30 % of a day in many small settlements (ADR-0032, amendment of DN).
- **Ways cannot tell modes apart for wear.** `Ways::holds` keeps every way's wear whatever its
  modes, so a railway would become a full-speed footpath.
- **Journeys by vehicle and between settlements are not taken** in household-resolved days
  (`resolution.rs`).
- **Light acts on nothing yet.** Sight is 30 m day and night; no good makes light; fire is worked
  only at home.
- **Rail began as wagonways** at pits and quarries, pushed or pulled, before locomotives (07-08
  §4); its value is payload and time on a fixed line. Gas light came from coal at gasworks, with
  holders, mains and connections; it lengthened working evenings and changed where people were at
  night (12-09 §2, 12-04).

## Decision

### 1. Routing groundwork first

Before any rail: landmarks are built per settlement over its own walked ground, as ADR-0028 §4
decided, and kept until walking costs change there; a change elsewhere does not throw them away.
Route times are unchanged (landmarks only bound searches); a route may differ only among equally
fast ones, so the change is proven against route times and digests of everything else. Ways keep
wear by mode: a way's walking surface wears and holds only where people walk it.

### 2. Track

A `RAIL` mode on ways. Track is a way of grid cells under a turn rule (no turn sharper than the
spec's) and a ruling grade, graded and built as ADR-0029 §6's ways are, with bridges that carry
wheels (ADR-0020's crossings by spec). Lines are open ground: walkers cross anywhere in M10.

### 3. Rolling stock and pullers

Wagons are vehicles of the `rail` mode (ADR-0029 §2). A wagonway's wagons are pushed by people;
an engine (ADR-0033 §7, on wheels) is a puller with a power and a fuel, its work as ADR-0033's
power. Draught animals wait for husbandry.

### 4. Runs

A run is a line's event chain: it leaves a station, holds block sections of single track and
passing loops, and arrives; nothing is despawned. Riders and consignments of many owners ride a
run (amending ADR-0019 §3).

### 5. Stations, timetables, fares

A station is a building on a line. Its operator posts a timetable (runs a day, set at the
operator's review from the last period's riders and freight) and fares and freight rates as posted
terms, as any firm posts terms. A ride is chosen as any trip: by its hours and fare against
walking or carting. Journeys are composed from walking fields to and from stations; rail is never
in the walking search, and its revisions are kept apart from walking's.

### 6. Who builds a line

A line is built and owned by a household firm, an organization by stake, or a polity
(`build_line` as a public work, ADR-0019's works); a franchise over a route is a privilege
(ADR-0026).

### 7. Household resolution

A ride is a block: a household-resolved person's day may include a ride, applied as the run's
seat and fare. A line does not hold its settlement Detailed.

### 8. Gas

A gasworks makes gas from coal (coke left over, ADR-0033 §1); holders store it; mains carry it to
connections, checked for capacity by district each day; an interruption has a cause (works down,
coal short, main broken). Gas, holders and mains are records; gas is accounted in energy content.

### 9. Light

Light is vocabulary before gas: an oil or fat lamp burns a fuel good for hours of light. A place
is dark or lit by what burns there. Sight has a dark distance (`dark_sight_m`), equal to day sight
until a world first has lamps, so no world's digests change before then; light restores sight
where it falls. Light changes what scores in the evening (work, company, taking), not when people
sleep. Burners and works are ignitions as hearths are (ADR-0022). Street lighting is a public work
with lamplighters' work.

### 10. The boundary

Lines, runs, stations, mains and lit places are on the wire, append-only; the save schema, wire and
content API bump.

## Consequences

- The routing work speeds every many-settlement world before rail exists.
- Whether a line is built, and where, follows from what it carries and what it saves in hours;
  most worlds will not have one.
- Night becomes a place-by-place condition, so taking, watches and evening work answer to light.

## Alternatives considered

- **Rail in the walking search.** Every search would grow; composition from stations keeps walking
  searches as they are.
- **A centreline geometry below the grid.** Truer curves at the cost of a second geometry; cells
  with a turn rule keep one grid.
- **Darkness on now.** It would change every world's digests for a lamp nobody yet has.

## Revisit when

- Composed journeys miss rides a walker would plainly take.
- District capacity is too coarse for gas pressure to explain an interruption.
- Maps need edges that lead off the map for rail to matter beyond payload.
