# ADR-0040: Letting rooms

Status: Accepted (2026-10-31, by Claude under the owner's standing authority)
Date: 2026-10-31
Milestone: M11 (its deferrals; ADR-0037 §7)
Amends: ADR-0030 §1 and ADR-0031 §6 (a home's reckoning counts the rent its spare floor would
bring), ADR-0007 §3 (a lease of floor beside the lease of a field), ADR-0035 (custody: a lodger's
goods kept where it sleeps)

## Context

ADR-0037 §7 deferred letting storeys as "the reason anyone builds tall" (11-14 §5.6): a frame wins
only when someone can use or let its upper floors, and in TCE every household builds and lives in
its own home. Research 08-19 §1.3–§1.5 and §5.1–§5.3 asks for rent arising from search, bargaining
and contracts over rentable units (rooms or bed-spaces), households trading space against access
and cost, tenure as a bundle of rights, every payment in the ledger, and housing decisions as
events, not daily repricing; it warns against a universal affordability ceiling (§1.4) and
against "expensive land → tall buildings" as a rule (§1.7: height from marginal profitability).
The kernel already has what this stands on: households with no roof of their own (`sheltered`),
custody of their goods with a neighbour for a fee (ADR-0035; M10d slice EG), field leases paid on
the `Rent` channel (ADR-0007 §3), each program's sleeping floor per member (`floor_base_m2`,
`floor_m2_per_sleeper`), and a home's site and size chosen in hours a year (ADR-0030 §1,
ADR-0031 §6).

## Decision

### 1. A room is spare sleeping floor in a roofed home

A household's home has spare floor when its roofed, finished buildings' sleeping floor exceeds
what its own members need (`floor_base_m2` + `floor_m2_per_sleeper` × members). A lodging
household needs `floor_m2_per_sleeper` × its members: lodgers share the hearth. No room is
partitioned or drawn; a let is a share of floor, recorded.

### 2. Who seeks a room, and who lets one

- **A household with no roof of its own** (not `sheltered`, not within days of finishing one)
  weighs, on its review day, the homes of its settlement within its reach that have spare floor
  enough, against staying as it is: a room's rent in food by its `hour_kcal`, plus the walk to its
  fields and water from there (ADR-0030 §1's hours), against what being roofless costs it now (its
  goods' faster spoiling in the open and any custody fee it pays, ADR-0035). It offers on the
  cheapest, k candidates at most (08-19 §5.2; a design prior of 8).
- **A household with spare floor lets it** at its asking rent when the offer covers it. The asking
  rent starts from a design prior (a day of the lodgers' food a person a month) and moves by its
  own experience: up a tenth after an offer it turned away for want of room, down a tenth after a
  review with spare floor and no lodger (08-19 §1.4: asking rents move with offers and vacancy, not
  daily). No affordability rule: a household that cannot pay stays roofless or shares.

### 3. The let is a contract

Saved with landlord, lodger household, floor let, rent a month, the day begun and the next
review. Monthly, the lodger pays on the `Rent` channel (the conservation check covers it); one
that cannot pay is in arrears, and after a second month leaves. Either ends it at a review: the
lodger once it has a roof of its own; the landlord when its own members need the floor. A lodging
ends at once if the building fails or burns.

### 4. What a lodger has

A lodger counts as sheltered: its goods keep under the landlord's roof, up to its share of the
building's room by floor (ADR-0035's custody path, no fee beyond the rent), and its home is the
landlord's building. Nothing else changes: it keeps its own stores, decisions and fields.

### 5. Building upward counts what the floor would let for

When a household designs a home (ADR-0030 §1, ADR-0031 §6), each design's hours a year are
lessened by the rent its spare floor would bring at the settlement's recent contract rents (the
mean of lets begun in the last year, or none if there were none), times the share of the
settlement's households that are roofless (the chance a room is taken). No let seen, nothing
counted. Height still follows from hours; nothing pushes toward it.

### 6. The boundary

Lets are a new saved section (saves bump); the inspector's words for a home and a household name
lodgers and landlords (strings, wire unchanged); content gains the asking-rent prior per
people profile (content API bumps).

### 7. Deferred

Rooms as drawn partitions, sub-letting, a landlord who does not live in the building (which the
skeleton's tall buildings would invite: a building raised to let), eviction law, rent law, and
public housing.

## Consequences

- Newly formed and newly arrived households can sleep under a roof before they build one.
- Homes with spare floor earn rent; where lets are seen, larger and taller homes become worth more
  in a household's own hours. Whether anyone builds a second or third storey to let is the test.
- A world where no household is roofless lives as before.

## Alternatives considered

- **A rental market solver over every dwelling.** 08-19 §5.2 warns against daily repricing and
  all-pairs matching; k candidates on review days is enough.
- **Lodging as a kin obligation only.** Kin already join households; strangers paying rent are the
  missing case.
- **A landlord's building for letting only (a tenement) first.** It needs a builder who never
  lives there; that waits until letting is seen to pay.

## Revisit when

- Households build storeys to let and a building raised only to let would pay (§7).
- Rents need law (caps, eviction rules) because a polity's issue points at them.
