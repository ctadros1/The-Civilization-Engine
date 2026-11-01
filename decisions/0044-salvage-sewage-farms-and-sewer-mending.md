# ADR-0044: Salvage, sewage farms and mending sewers

Status: proposed (draft)
Date: 2026-11-01
Milestone: M11 (its deferrals; ADR-0038 §6, ADR-0041 §4)
Amends: ADR-0038 §2–§4 (heaps and tips hold discards; a round may sort and its fee may fall below
zero), ADR-0041 §2 and §4 (a sewer wears, silts and breaks at a cell; the farm made concrete)

## Context

ADR-0038 §6 deferred "salvage trades beyond what a recipe already takes"; ADR-0041 §4 named the
sewage farm without building it, and plan §7 lists "mending a broken sewer (nothing breaks one
yet)" as not yet. The design is in `docs/briefs/backlog-c-salvage-sewage.md`. Its research:
12-10 §1.6 (salvage is a trade whose sign follows prices and haul; London's dust contractors went
from paying to being paid), §5.6 (salvage goods with uses in existing recipes); 12-10 §1.7, §2.2
and 12-02 §1.6, §2.3 (a farm is a field limited by area; nutrients and pathogens are separate);
12-02 §1.5 and 12-08 §2.1, §4.3–§4.7, §5.1–§5.3 (blockage, wear and conditional failure, no
universal decay rate, care is not renewal, outages recorded). The kernel's heaps and tips hold only
a decaying share and ash; worn tools vanish with their metal; residues have no own cost, so nobody
offers them; `Sewer.broken` is saved and never set.

Three things here are expensive to reverse: what a heap and a tip hold (saved in the households
and tips sections), a sewer's saved condition, and a contagion node code. Payments are not: they
stay on the `Refuse` channel.

## Decision

### 1. What is thrown away: discards

- A tool good may name `[scrap]` (content): a good and kilograms a whole tool. As it wears, its
  holder gains that share of the scrap: a household on its heap, a works in its store (as
  residues, ADR-0038 §2).
- A heap and a tip hold **discards**: a short saved list of (good, kilograms) beside the decaying
  share and ash. One list, not one field for each good, so that rags, bones or glass are later
  content only. Carrying, tipping and a round's load move discards whole and record them; a sewer,
  manuring and the destructor take none.
- The core adds `good/scrap_iron`, iron tools' scrap (half the bar each took, a design prior) and
  `recipe/forge_scrap`, which welds scrap into bar.

### 2. Picking is a way to get a good

- A member may sort its own heap or its polity's open tip at a content rate (100 kg an hour, a
  design prior), taking the discards and ash its household wants. The good's own cost by picking
  is the hours over the kilograms its share of the site yields, plus the walk. Salvage then enters
  own costs, wants and the market as any gathered good. **No price, recovery rate or salvage
  share is authored.**
- Salvage has uses only through recipes: `forge_scrap`, and `fire_bricks_with_ash` (a quarter of
  the clay as ash, a design prior). Where nobody forges or fires bricks, nobody picks.
- Who picks follows what an hour brings a household elsewhere (`hour_kcal`). Nothing names a
  class of picker.

### 3. The round's fee may change sign, on the same channel

- A posted round's keeper may sort its load before tipping. Its fee a call is its run and sorting
  hours at its `hour_kcal`, with fuel, less the worth of what it keeps, over the homes. Below zero,
  the keeper pays each home the difference on `Refuse`, and a home joins with no law asking. The
  saved fee keeps its field; a negative value is now meaningful (the rounds section's version
  bumps).
- **No new ledger channel.** The direction of a payment on `Refuse` is the payer's, as on every
  channel. The law's round (`collect_refuse`) does not sort in this ADR.

### 4. The sewage farm

- A field with a cell beside a sewer's route below its lowest connection may take the flow. At the
  outfall, settling comes first (ADR-0041 §4), then the farm up to its area's limit (`[sewer]`
  `farm_people_ha`, 100 a hectare, ADR-0041's prior), then the reach.
- What the farm takes enters the field's soil as dung at the heap's nitrogen a kilogram; the crop
  answers through the soil's own limits, never a bonus.
- Its disease loads go to **`Node::Field`** (contagion code 8, appended), which washes and seeps
  from the field's place by the links a tip uses.
- The field's household takes the flow, for nothing, after a harvest the soil held back, and stops
  when the soil no longer does. Suspicion is unchanged: people tally water sources, not fields.
- The sewer record saves its farm field and the day's kilograms to it.

### 5. A sewer wears, silts and breaks; the polity mends it

- **Deposit:** a share of what enters stays as deposit kilograms (mass only), lowering effective
  capacity in proportion to what the length holds before it blocks (design priors). Unserved
  connections heap again, the last connected first: backing up is heaps regrowing.
- **Wear by cell:** each month each route cell loses `1 / life_years` times its ground's and the
  month's wetness (the wells' factors). A cell at loss 1 collapses; connections above it back up,
  those below still flow.
- **Saved:** the deposit, each cell's loss, the broken cell (replacing the meaning of `broken`),
  and each connection's days unserved this year by cause. `SewerCause` gains `Blocked` (code 3).
- **Mending:** the polity learns of a fault only when a connection backs up for want of capacity
  it once had, or at a collapse. While the law that laid the sewer is in force, its review weighs
  the fault as laying was weighed and asks the work as common work (equal shares, or `pay_work`).
  Clearing turns deposit into `sludge` in the keeper's store; relining rebuilds a collapsed cell
  at laying's hours and materials a metre. Neither mends the other's fault. With the law ended,
  nobody mends.

### 6. The boundary

The households, tips, rounds and sewers sections change meaning and bump their versions (or
`SAVE_SCHEMA_VERSION`, as is current when built), with explicit migrations: empty discards, zero
deposit and loss, no farm. Contagion code 8 is appended. New `[scrap]`, `[sewer]` terms and the
`pick` behaviour bump the content API. The wire is unchanged: words in the polity, services and
outbreak pages tell farms, faults and pickers; drawing them waits.

### 7. Deferred

Rags, paper, glass and bones (no goods or uses; 12-10 §5.10); picking others' heaps; the law's
round sorting; a tip's access as a law's term; produce and workers' exposure on a farm (no food or
handling route); metals in farm soils (no harm model); selling sewage; sludge spread as dung;
leakage through worn walls (a node and a rate the research lacks); inspection; a sewer rate paid by
connected homes (it would need a channel).

## Consequences

- Iron no longer vanishes with a worn tool; where iron is scarce, as in M11d's demo, scrap is a
  second way to bar.
- A smith or brickmaker near a tip can make a round pay its homes; take the buyer away and the
  same refuse costs them again.
- A sewer becomes a standing cost. A polity that stops mending gets its heaps back.
- A farm can sicken a well beside a field while the river below the outfall carries less.
- Worlds with no worn iron tools, tips or sewers live as before; their saves still migrate.

## Alternatives considered

- **Salvage as a fixed share recovered automatically, or a salvage price.** Plot by another name;
  12-10 §1.6 says worth follows prices of substitutes and haul.
- **One field for each salvage good on heaps and tips (`scrap_kg`).** Cheaper now, but each new
  salvage good would be a save change.
- **A `Salvage` or `Sewer` ledger channel.** Nothing a channel would show that `Refuse` and common
  work do not; channels are append-only.
- **A sewer condition that falls a fixed share a year, or a random failure chance.** 12-08 §2.1 and
  §5.2: no universal rate; failure follows condition and place.
- **One condition for the whole sewer.** It cannot make a break at the outfall cost more than one
  at the head (12-08 §4.7).
- **Leakage now.** No rate in the research; backing up already carries failure's contamination.

## Revisit when

- Cloth, paper or glass exist: their scrap is content on §1's list.
- A food or handling route exists: farm produce and pickers' exposure.
- A demo shows sewers failing unseen for years: inspection as a law's term.
- A non-pathogen harm model exists: farm soils and tips get a dose (ADR-0038's revisit).
