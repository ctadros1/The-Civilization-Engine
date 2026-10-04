# ADR-0007: Claims and property regimes

Status: Accepted
Date: 2026-10-04
Milestone: M3a

## Context

Until M3a slice K, the household that works a field also holds it, and the household that built
a hut lives in it. When a household is no more, everything it had goes to one heir. M3a needs a
property regime v0, chosen for a world, that sets who holds, works, allocates, leases and
inherits fields, plots and buildings (plan §5.3, §7). Its demo runs one seed twice, as a
communal and as a household-tenure village, and the two should diverge in prices, house sizes and
the distribution of wealth. Saves will keep these claims for the life of a world, and panels will
show them.

Reports read: 08-09 (property regimes), 08-11 (allocation without markets), 08-14 (inequality),
08-19 (housing, land and rent), with 08-08, 08-10 and 04-08 from earlier slices. Their main
points:

- **Property is a bundle of rights, and different parties can hold them.** Access, withdrawal,
  management, exclusion, alienation, succession, revenue, duration and recognition can each
  belong to someone else. A landholder need not be the cultivator (08-09 §1.1).
- **Common property is not open access, and collective ownership is not collective
  production.** Families can farm separate fields on land a community holds (08-09, executive
  recommendation; §5.2's village example: the village holds the territory, households are
  assigned cultivation, crops belong to the operator, sale is prohibited, leasing is permitted,
  and household division triggers an allocation review).
- **Ownership changes nothing physical.** A change of holder must not change a field's
  fertility or yield (08-09, executive recommendation; §5.3: title changes neither create nor
  destroy land, buildings or crops).
- **Claims form through occupation, investment, membership and recognized rules.** A frontier
  may recognize first cultivation; a lineage may recognize membership and allocate cultivation
  internally (08-09 §1.2).
- **Leases are not sales.** A society may forbid permanent alienation and still allow use to be
  let. Crop shares and fixed rents spread risk differently; Operation Barga set the tenant's share
  at 75 % where the tenant supplies the non-labour inputs (08-09 §1.4, §1.6, §2.1).
- **Inheritance connects property to lasting inequality.** Supported rules include equal
  division, preferential heirs and return to an institution. The consequences should follow from
  household formation, rental and migration rather than be assumed (08-09 §1.7; 08-14 §2.4).
- **Measure several kinds of wealth, not one number.** These include land held and worked,
  goods, and house floor area (an archaeological proxy); person-weighted Ginis, top shares and
  landless shares; a decomposition of how wealth changed; and paired runs that change one rule
  at a time (08-14 §1.1, §5.6, §5.7). A transfer between households creates no wealth (§4).

## Decision

### 1. Holders and users

- Every field records two parties.
  - Its **holder** has the claim: it may allocate the field's use, let it and pass it on, as the
    regime allows. A holder is a household or a settlement. Settlements become holders of land
    claims here, as ADR-0006 §3 foresaw; they hold no goods yet.
  - Its **user** works it and keeps what it yields. A user is a household.
- The holder and the user are the same household unless the field is allocated by a settlement
  or let.
- In v0 a dwelling plot and its hut stay with the household that lives in them, under every
  regime, and pass to the household its people join. A hut is an improvement its builders made;
  giving out houses waits for a regime that needs it.
- Holding or using a field changes nothing physical. A field yields by its ground, its work and
  the year, whoever holds it.
- A field whose user is no more is **vacant**: nobody works it until it changes hands. Under
  allocation by need its settlement gives it out again; under holders, ground whose holder is no
  more either is taken up by a household short of land, which then holds it (first occupation,
  08-09 §1.2). Fields change hands at a review only between crops, so a crop and the work put
  into it stay with the household that did it; at a death a crop goes with its field, as part
  of the estate.

### 2. Regimes are content

- A **regime** is a content kind (`kind = "regime"`, ids `<pack>:regime/<name>`). It gives a
  name, a description and its rules. One regime is the default; a new world may choose another.
  The choice is fixed for the world's life in v0 and is kept in its saves.
- Its rules:
  - **Who holds broken ground:** the household that broke it, or the settlement.
  - **How use is given:** the holder works its ground (and may let it), or the settlement gives
    each household fields to work by how many it feeds: the area a household of its size plans
    to crop, the same need it plans by itself. It does so at a yearly review before the year's
    field work, and when households form or end, giving fields that are vacant or beyond what
    their household needs to the households furthest short of theirs, nearest their homes, and
    never leaving a household short of its own need.
  - **Succession:** when a household is no more, its holdings go to one heir, are divided
    among its heirs' households (whole fields, as near equal in area as they can be), or return
    to the settlement. Its heirs are the households of its last member's nearest living kin, at
    the first step out where any is found; those in its own settlement count first. Children left
    with no adult are taken in by kin with their household's land, as before.
  - **At a union:** whether a new couple's household takes a share of its families' fields, as
    it takes a share of their stores.
  - **Leasing:** whether use may be let, the holder's share of the crop and the term.
- Labels such as "communal" or "private" are content names, not states the engine switches
  between (plan §1).

### 3. Leases

- A lease lets a household other than the holder work a holder's field for a term (whole
  crop years) for a share of the grain it threshes.
- The share is paid through the ledger on a new `rent` channel when the grain comes in, so the
  conservation check covers it.
- Leases are offered by holders with more broken ground than they plan to work, and taken by
  households short of ground. Both are run-time decisions.
- A lease ends at its term, when either household is no more, or when the holder needs the field
  itself at renewal. Sales of land wait for a later slice: v0 has no land market.

### 4. Wealth measures

- The kernel measures each household by several separate quantities:
  - land held and land worked, in hectares;
  - its goods, valued at the settlement's prices (the median of its households' own costs, in
    hours of work), so households can be compared;
  - its house floor area.
- From these it reports person-weighted Ginis (of goods, land held and land worked per head;
  floor area is compared house by house, as archaeologists compare houses), the top tenth's
  share of goods, and the shares of households that hold no land and that work none.
- It keeps a yearly history of these per settlement, for the panel and the smoke checks.
- The measures are derived. They are never inputs to behaviour, except where a later slice
  names one (house size by wealth in slice L).

### 5. Saves and boundary

- Saves gain each field's holder (its user is the household saved with it), the leases, the
  regime id in the world's metadata, and each settlement's yearly wealth history (in schema 12;
  the history's section is read when present, since it is a measure and nothing depends on it).
- Saves from before slice K load under the default regime with every field held by the
  household that works it, as before.
- The wire gains the world's regime (and the regimes a new world can choose), each field's
  holder, user and lease, and a wealth query answered with each settlement's households, their
  measures and the yearly history.

## Consequences

- One seed can be lived under two regimes with everything else equal, which is the M3a demo and
  the paired experiment 08-14 §5.7 asks for.
- Inheritance, unions and households ending now consult the regime instead of passing
  everything to one heir.
- Settlements become parties with claims, which councils and law (M4) will build on.
- Forbidden: a regime that changes yields or fertility; land that appears or disappears through
  a change of claim; rent paid outside the ledger.

## Alternatives considered

- **A separate claim table, with one record per right per party** (08-09 §5.1's full `Claim`
  entity). It is more general, but v0 needs only two rights layers (holding and using) and
  leases. A holder and a user on each field keep saves and panels simple. A claims table can
  replace them when rights multiply (grazing, dues, disputes) without changing what v0 saves
  mean.
- **The regime as a parameter of the people profile.** It is simpler, but a world could not
  choose its regime, and the demo needs one seed under two regimes.
- **Regime changes during a run.** They are out of v0: they need political actors and a reform
  sequence (08-09 §1.8), which arrive with councils and law (M4).

## Revisit when

- Councils and law (M4) let a settlement change its rules: reform as an event sequence.
- Grazing, commons resources, dues or disputes need rights beyond holding and using.
- Land sales, credit or mortgages arrive.
