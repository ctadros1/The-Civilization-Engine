# ADR-0032: Resolution per settlement, the household day and time skips

Status: Accepted
Date: 2026-10-20
Milestone: M9
Amends: plan §4.5 (the aggregate model keeps every person; cohorts are a later lever, not the
first), plan §8 open question 1 (time skips kept, as fast runs), ADR-0011 §1 (a resolution per
settlement beside the speed's mode), ADR-0028 §1 and §3 (M9's scale figure; settlements side by
side replaced by parallel household planning), ADR-0012 (the background climate takes a forcing)

## Context

Plan §7's M9 lists a "cohort model, 10–30 settlements, 100k+ agents; **decide time skips**;
climate change god tool". The brief (`docs/briefs/m9-lod.md`) reads 01-04, 01-03 and 03-03, with
16-03 through ADR-0011's gates. Its findings:

- **The gap is three orders of magnitude.** M8 closed with 10,000 people in one settlement at
  53.5 s a simulated day, about 5 ms a person-day; a year a real minute at 100k is 1.6 µs. About
  half a day goes in deciding (14.5 decisions a person-day), a seventh in hearth company, an eighth
  in route searches, under 1 % in midnight's own work. A lightweight person who still decides each
  activity saves 1.3–1.5×.
- **01-04 keeps the population and aggregates computation.** Persistent people, households,
  institutions and assets; resolution as a property of execution; whole settlements switched
  first; switching as a transaction with invariants; rules shared between modes and tested; no
  synthesis over a record. Its first milestone is "switch a settlement repeatedly without changing
  who exists, who owns what, or how much food is present".
- **Plan §4.5's cohorts would delete history.** Bins of age × sex × occupation × wealth × ideology
  with people synthesised on focus cannot give back the marriages, debts, ties, beliefs, offices
  and property every system since M1 keeps by person, which 01-04 §1.1 shows no lifting recovers.
- **A skip is a fast run** (01-04 §4.8, 01-03 §4.2): the same machinery to the next boundary,
  never an average rate over decades.
- **Climate change is specified forcing** (03-03): a change to the background climate as an input,
  not an era flag.

## Decision

### 1. Two resolutions, chosen per settlement

- **Detailed** is the kernel as it is: every person's activities run as events through the day, in
  Detailed or Accelerated mode (ADR-0011).
- **Household** resolves each household's day at the midnight that begins it (§2). Its people,
  households, stores, fields, buildings, projects, claims, ties, beliefs, offices and laws are the
  same records; midnight's work (`land.advance_to_day`, `on_day`, the month and year cadences) runs
  for it unchanged.
- **Cohorts** (counts by group, members chosen when an event happens) are not built in M9 unless
  the household day misses §8's figure; then they are the next lever, designed by amendment.
- **A run setting, not world state.** A settlement's resolution is chosen by the host or the
  bench, is never saved, and takes effect at a midnight. Every save is valid in either resolution,
  as every save is a valid Detailed state (ADR-0011 §1). A save plus a sequence of (mode,
  resolutions, days) reproduces exactly.
- **Speed and resolution are separate.** A household-resolved settlement is lived at midnight at
  every speed; at 1× its people are shown where their day's blocks put them (§6), not walking.
- **Whole settlements only**, as 01-04 §4.2 asks. A settlement holds Detailed while a fire burns
  in it, a band of its polity is out or under way to it, a siege line stands about it, or a
  gathering of its polity is called and not yet held; the host's choice waits until none holds.

### 2. The household day

- **When:** at the midnight that begins the day, after `on_day`, every household of a
  household-resolved settlement is planned, then the plans are applied.
- **Planning is pure.** For each household, the household view (ADR-0011 §4) and each awake
  member's facts at the day's start are built, and `decide::candidates` scores them as it would at
  that member's first decision. Each member's waking hours, less what needs fix (sleep to the
  sleep need, meals to hunger, water fetched to thirst, all by the same needs model), are shared
  among the scored behaviours in proportion to the probabilities `decide::choose`'s softmax gives
  them, each share a block of whole minutes, rounded by largest remainder. Travel to a block's
  target takes the settlement's travel field's time and is part of the block. Nothing is written
  while planning; plans are made in parallel and are independent of thread count.
- **Applying is serial**, household by household in household id order, block by block in plan
  order. A block's effect is the behaviour's own effect function (`field_work`, `build_work`,
  `make_work`, `eat`, `land.harvest`, `deposit`, `give_food`, `settle_trade` and the rest), called
  with the block's minutes, target and goods, as `end_step` calls it; `practise`, `discover`,
  `wear_tools` and time use follow as they do there. A block whose precondition no longer holds
  when applied (the seller's stock gone to a household applied earlier, the field taken, the
  project finished) yields nothing and its minutes go to the household's next block, else to rest;
  nothing is clamped.
- **Company.** A member's hearth hours are one session drawn against the settlement's hearth pool
  of that day (the members who share hearth hours and Detailed visitors there the day before),
  through `keep_company` once a member-day; word, opinion, ideology, ties and norms move through it.
- **Path wear.** Each block that leaves home lays wear along its route as a walk would, the route
  taken from the route cache or searched once and cached (ADR-0028 §4); its travel time is still
  the travel field's.
- **Not taken while household-resolved** (listed in the code, each with a test that it is never
  chosen): `Visit`, `Fetch` beyond the settlement, `Carry`, `Band`, and the fire behaviours
  (`Douse`, `Strip`, `Salvage`, `Pull`), since no fire starts in a household-resolved settlement
  (§1 keeps a burning one Detailed; ignitions are planned only for Detailed settlements). Every
  other behaviour is resolved as a block, its effect through its own function; a slice may list a
  behaviour as not taken only with a §9 entry naming why.
- **Incidents of the day** (a taking seen, a sighting by the watch, an illness's onset) happen at a
  block's end as `end_step` makes them, at the minute the plan puts the block.

### 3. Switching

- **At a midnight**, as a mode switch is (ADR-0011 §2). Demotion: open activities end as midnight
  finds them, loads carried are banked (`deposit`), everyone is placed at home, pending step
  events are dropped. Promotion: each person begins with a first decision at their next waking,
  as `begin` starts founders, with their needs as midnight left them.
- **Invariants**, checked by a debug function the suite calls at every switch: the same living
  people, households and memberships; the same holder of every field, building, plot, claim and
  firm; every good's total over the ledger's parties unchanged; every person in exactly one
  executor; no pending event for a household-resolved person.
- **Gate A extended:** a settlement switched down and straight back up at the same midnight lives
  on exactly as if never switched; household-resolved days reproduce from a midnight save; thirty
  one-day advances equal one thirty-day advance; plans are equal at 1, 2, 4 and 8 workers.

### 4. The boundary between resolutions

The boundary is midnight. A Detailed visitor at a household-resolved hearth meets company drawn
from that hearth's pool; a Detailed buyer at a household-resolved seller buys from its stores as
the seller's applied day left them; a household-resolved fetch within reach of its settlement is a
block whose goods are in its stores when applied. Moving, marriage, founding, word at gatherings,
holds and agreements are midnight's already and are unchanged.

### 5. Gate C: the household day against Detailed

- `civ-host consistency --resolution household` runs Gate B's fixtures and aggregates with the
  fixtures' settlements household-resolved against the same fixtures Detailed (Accelerated), five
  runs each, tolerances fixed once from the Detailed spread as Gate B's are, before evaluation,
  recorded in §9. Takings, gatherings decided, laws passed and buildings begun are added as
  aggregates, and an intervention (the drought tool) must move both resolutions the same way.
- A ten-year horizon on the dashboard's smoke seeds compares population, food days and harvest,
  ungraded until its tolerances are set.
- Household resolution is not the host's default for any settlement until Gate C passes.

### 6. The observer

- **Focus decides.** In a world of at most `DETAILED_UP_TO` settlements (3) every settlement is
  Detailed, so every v1 world lives as before. Above that, the settlement the observer follows or
  centres is Detailed and others household-resolved, with a minimum residence of 30 days in
  either resolution (01-04 §2's hysteresis). The observer may pin a settlement Detailed.
- The wire carries each settlement's resolution (append-only), and a household-resolved person's
  brief carries the block they are in by the clock, drawn at its target, not interpolated.

### 7. Time skips

Kept (plan §8 question 1 answered). "Skip to" is a host command: Accelerated Max with every
settlement household-resolved, stopping at a date, at the observer's pause or at a chronicle event
the observer marks, and restoring the resolutions after. It is a fast run; nothing is generated
after the fact, and it saves and reproduces as any run does.

### 8. Many settlements and M9's figure

- Worlds may be founded with up to 30 groups on a map large enough (setup, ADR-0018).
- **M9's gate:** 100,000 people in at least ten settlements, all but one household-resolved, living
  a year at Accelerated Max at **≥ 0.1 simulated years a real minute** on the container (1.64 s a
  simulated day), with peak memory measured against plan §4.6's 4 GB at 50k and recorded at 100k.
  The target PC's figure is recorded when the owner runs it. If the gate misses after the time-boxed
  slices, it is amended to the figure reached with the profile as evidence (ADR-0028 §1's rule),
  and cohorts (§1) are designed.

### 9. The climate tool

An observer influence (ADR-0022 §13) sets a forcing on the world's background climate: an offset
in °C to every monthly temperature normal and a factor on wet-day amounts, each reached linearly
over a ramp of whole years from the day given, and held after. The weather generator (ADR-0012)
reads the forced normals; snow, water, dryness, soils and harvests follow through the existing
chain. The harvest factor's long-run mean stays the unforced climate's, so a forced climate is not
renormalised away. Forcings are saved (they are world state), recorded in the chronicle, and
compose by sum (offsets) and product (factors).

## Consequences

- Every person, debt, tie, office and building stays one record at every resolution; the observer
  can follow anyone anywhere, and a settlement brought into focus is the one that was lived.
- A household-resolved settlement costs about one scoring a household a day, not seventy decisions;
  planning runs on every core exactly.
- Every behaviour added after M9 is written once and must name how its effect is applied as a
  block, or be listed as not taken with a §9 entry.
- Forbidden: state that exists in one resolution only; a resolution saved as world state; a
  coefficient fitted and frozen in place of the live scoring; synthesising a person, tie or asset
  on a switch; household resolution as a host default before Gate C passes.

## Alternatives considered

- **Cohorts with synthesis on focus** (plan §4.5 as written). It deletes the history every system
  keeps by person; 01-04 §1.1 shows lifting cannot recover it.
- **Lightweight individuals who still decide each activity.** Saves 1.3–1.5×, three orders short.
- **A learned emulator of a settlement's year.** Extrapolation and drift, no identities, and its
  coefficients go stale whenever a law or technique changes (01-04 §1.4, X4's lesson).
- **Settlements side by side** (ADR-0028 §3's deferral): state split by settlement and messages at
  boundaries. A rewrite of every table for at most the core count; parallel planning of household
  days gets the cores without splitting state.
- **Time skips as a separate history generator.** It would be a second model to keep consistent
  with the first.

## Revisit when

- Gate C fails on an aggregate the household day cannot be brought within (cohorts or a finer
  household day).
- The gate misses by more than the household day's measured share explains.
- A behaviour's effect cannot be applied as a block without a model change.

## Amendment (2026-10-20, M9a slice DG): the household day as built

- **Effects land at midnight**, not at the minute the plan puts a block (§2's last point): the
  clock stands at midnight while a household-resolved day is lived, and records dated later in
  a day not yet lived would come before the Detailed settlements' events of that day. Scoring is
  as at sunrise (the night is lived asleep first); what follows is dated midnight.
- **Hours are shared by expected repetitions**: an option chosen with probability `p` and lasting
  `T` minutes is lived `waking · p / Σ p·T` times, so its share of hours is `p·T / Σ p·T`, as a
  run of decisions would give it.
- **Company's pool** is the settlement's residents who kept company that day; Detailed visitors
  to a household-resolved hearth meet nobody parked there (§4's pool for visitors is slice DH's).
- **Eating** is once a day, what the night and the day before used; **water** is fetched by the
  trip a member would make, until a day's use is in.

## Amendment (2026-10-20, M9a slice DH): what holds a settlement, and what waits

- **A called gathering does not hold a settlement Detailed** (§1 listed it). Coming to a
  gathering, a poll or a petition is now a block counted as an arrival there is, so a
  household-resolved settlement takes part in its polity's decisions; holding it for every
  gathering would have kept settlements Detailed most of the time. Fires and bands still hold.
- **Options moved by a boat or a cart** (paddling, hauling, handling a load) are left out of a
  household-resolved day until slice DJ gives such journeys their block; making a boat or a cart
  is shared.
- **No fire in use is tended in a household-resolved settlement**, so none escapes there: §2's
  "ignitions are planned only for Detailed settlements", as built.

## Amendment (2026-10-21, M9a slice DI): weighed three times a day

§2's "scored once" became three weighings a member-day: at sunrise for the morning, at midday for
the afternoon, and an hour after sunset for the evening until turning in. One weighing at sunrise
shared every waking hour by the morning's priorities and put work 10–16 % above Detailed and
leisure 4–7 % below it on Gate C. The later weighings put each member's needs back as they found
them. A day is still about three decisions a person, not fourteen. The night runs from turning in,
two hours after sunset, to sunrise; the day's energy is each block at its activity's level.
