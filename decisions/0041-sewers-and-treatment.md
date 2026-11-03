# ADR-0041: Sewers and treatment

Status: Accepted (2026-10-31, by Claude under the owner's standing authority)
Date: 2026-10-31
Milestone: M11 (its deferrals; ADR-0038 §6)
Amends: ADR-0034 §8 (a third carrier on the mains' pattern), ADR-0021 §4 (a sewer's outfall is a
node of the contamination links), ADR-0038 §3 (sludge goes where refuse goes)

## Context

ADR-0038 §6 deferred sewers and treatment: "no sewer in the kernel; 12-10 §5.8 gives their
shape". Research 12-02 §1.5: a sewer is a conveyance network with capacity, slope, blockage,
maintenance and an outlet; "a connection does not itself destroy pathogens or nutrients"; household
discharge goes to the outlet unless leakage, overflow, settling or treatment intervenes; "a cleaner
street and a dirtier downstream river should be a possible outcome of the same infrastructure
investment". Research 12-10 §1.7 and §5.8: sewers outpaced treatment (by 1905 over 95 % of the US
urban population discharged untreated); settling splits a flow into effluent and a sludge good
(primary settling removes at least 20 % of BOD and 50 % of suspended solids); a sewage farm is a
field receiving load with an area limit (Berlin, about 19,000 acres for 1.5 million people;
0.5–1 ha per 100 people as a check); biological treatment comes later. The kernel has heaps whose
decaying share carries disease loads (`Node::Midden`), tips (`Node::Tip`), reaches downstream
(`Node::Reach`), and carriers laid along ways with connections at buildings (gas mains, ADR-0034
§8; wires, ADR-0036 §3).

## Decision

### 1. A sewer is a carrier laid by gravity

A sewer is a record like a main: cells along ways, laid a metre at a time by work and material
(brick or stone, with lime or cement), carrying once whole, from its head to an **outfall** on a
river reach. It must fall all the way, by the terrain's heights, at no less than a content
gradient (a design prior of 1 in 500); a route that would rise is refused (no pumping until an
engine is set at a lift, deferred). It has a capacity in kilograms a day, a design prior by its
section.

### 2. Connections carry a heap's decaying share

A home along a sewer connects its privy: from then each day its heap's new decaying share (not its
ash, which still heaps) and the disease loads with it go into the sewer instead of the heap, served
in the order connected while the sewer's capacity lasts; a connection left unserved heaps as
before and keeps its cause. What enters reaches the outfall that day, its loads moving whole to the
outfall's reach (recorded carried moves): the street is cleaner and the river downstream carries
it. A broken sewer (a failed segment) heaps its connections' share again until it is mended.

### 3. Who lays it and who connects

A polity lays one by law, `lay_sewer`, a `PolicyKind` appended on `lay_current`'s pattern, answering
`source_suspected` and `heaps_near`, its work asked of households or paid by `pay_work`. A household
connects when the hours carrying its heap would cost it (what `refuse_to_tip` or `clear_middens`
asks, or a round's fee) exceed a connection's one-off work; with no law asking, it connects only for
nothing (a connection is free once the sewer passes its door, a design prior).

### 4. Treatment at the outfall

- **Settling** (a technique and a fixed tool, a settling tank, at the outfall): a share of the day's
  load (a design prior of 0.5 of mass and of each disease load, 12-10 §1.7's at least half the
  suspended solids) becomes `sludge`, a good held by the tank's keeper, which goes to a tip or a
  field as refuse does (ADR-0038 §3); the rest goes on to the reach.
- **A sewage farm**: the outfall may be led onto a field instead of a reach; the field takes the
  load as dung (its nitrogen, ADR-0011's soils) and its disease loads seep by the field's own links,
  up to its area's limit (a design prior of 100 people's discharge a hectare, 12-10 §5.8's check);
  beyond it, the rest goes to the reach.
- Biological treatment, discharge standards by the reach's flow (the Royal Commission's), and
  blockage by sediment are deferred.

### 5. The boundary

Sewers, connections and tanks are saved sections; the outfall is a contagion node (appended code);
`sludge` and the settling technique are content; the observer's water layer draws sewers and
outfalls (wire bumps); saves and content API bump.

## Consequences

- A sewered street keeps its heaps light and moves its disease downstream: a village below the
  outfall may sicken where it did not; settling and farms move part of it again.
- Without the law nothing changes.

## Alternatives considered

- **A sanitation score for sewered districts.** Forbidden (12-02 §5.1).
- **Sewers that destroy load.** 12-02 §1.5: a connection does not.
- **Pumped sewers first.** Gravity suffices for the first design; a lift with an engine is a later
  part.

## Revisit when

- An outfall's reach sickens a downstream settlement and its polity objects (ADR-0020's relations).
- Engines are common enough to pump a sewer over a rise.
- A reach's dilution matters to a law (the discharge standard).
