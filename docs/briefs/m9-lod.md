# M9 design brief: aggregate level of detail, many settlements and time skips

**Scope.** Plan §7's M9 ("Aggregate LOD & scale") lists a "cohort model, 10–30 settlements, 100k+
agents; **decide time skips**; climate change god tool". Plan §4.3 sets M9+ at "100k+ via aggregate
LOD" in 10–30 settlements. §4.5 sketches the aggregate model as cohorts ("age × sex × occupation ×
wealth × ideology bins") with "individuals synthesized from the cohort distributions" when a
settlement is focused, notables persisting in both. §8's first open question is whether to keep
time skips. ADR-0028 moved 50k at a year a minute here, and its amendment records the gate reached
by M8: 10,000 in one settlement at 53.5 s a simulated day (0.0031 years a real minute) on the
container. This brief covers:

- where a day's time goes today, and why no change to how an event runs reaches 100k;
- what the research asks of aggregation (01-04), of speed and skipping (01-03) and of climate
  change (03-03);
- a resolution per settlement that keeps every person, and what it must answer to;
- many settlements, time skips and the climate tool;
- guards, suggested slices and the questions an ADR must decide.

Reports read: **01-04** (aggregate cohort models and promoting and demoting agents, the M9 report),
**01-03** (multi-rate time, for skips), **03-03** (climate and weather, for the climate tool), with
**16-03** on validation through ADR-0011's gates.

**What exists to build on** (checked in the code):

- **One event loop.** `Sim::advance_minutes` (`civ-sim/src/lib.rs`) runs one scheduler: each
  person's activity is a list of steps (walk, work, deposit, wait, paddle, haul, handle), and each
  step's end is an event. `Population::decide` runs when a person's steps run out and scores 38
  behaviours (`params.rs`'s `Behavior`) over `Facts`, keeping the top eight and drawing one by a
  keyed softmax (`decide::choose`). Effects land in `end_step`, one function per behaviour
  (`field_work`, `build_work`, `make_work`, `eat`, `land.harvest`, `settle_trade` and the rest).
- **Midnight** (`Cadence::Day`): the land's day (weather, water, soils, wild stocks), then
  `Population::on_day`'s thirty-odd steps: fields, the polity's day (gatherings, cases, word,
  elections, factions), crime, bands, land reviews, buildings' wear and checks, fires, wells,
  crossings, ditches, markets' posted terms, credit, mints, and `live_day` (births, deaths,
  sickness, moving, partnering).
- **Accelerated mode runs every event.** It stops the clock only at midnight and declares two
  approximations under Gate B (ADR-0011 §4): the household view (options that do not depend on who
  decides, kept from a household's first decision until midnight or its own consequential step) and
  leisure blocks. Nothing is aggregated.
- **Settlements are indexed, not separated.** `Household.settlement`, `Population::households_in`
  (debug-checked against a scan), the travel field per settlement, markets, polities and standing
  per settlement. People, households, fields and buildings are world-wide tables.
- **Between settlements:** visits to another hearth, fetching by price report, carrying an
  agreement's dues, marriage through ties, moving and founding, word at the hearth with visitors,
  trespass, bands and raids, polities holding several settlements.
- **Gates.** Gate A (exact, every push) and Gate B (`civ-host consistency`: two fixtures, five runs
  a mode, ten aggregates with tolerances fixed from Detailed calibration: people, work hours,
  leisure, sleep, walking, food days, harvest, materials, roofed share, Gini of goods).

## Where the reports agree

1. **Aggregate computation, not the population** (01-04, executive recommendation). Keep persistent
   records of people, households, institutions and assets; resolve routine work in batches; keep
   consequential events (births, deaths, migration, succession, construction, discovery) as changes
   to identified records. "Population synthesis can recover a population matching distributions; it
   cannot recover deleted marriages, debts, friendships, or ownership."
2. **Resolution is a property of execution, not existence** (01-04 §4.2). Detailed individuals,
   lightweight individuals (persistent people resolved by the day or by events, without movement)
   and cohort-driven settlements (routine transitions counted by group, applied to chosen members).
   "Initially switch whole settlements."
3. **The first milestone is the switch, not a model** (01-04 §4.10): "switch a settlement repeatedly
   without changing who exists, who owns what, or how much food is present." Then lightweight
   individuals as the reference fallback; then one narrow subsystem aggregated (food), compared
   against Detailed; then demography; then ideology and institutions; then skips.
4. **Promotion and demotion are transactions** (01-04 §4.6): at a safe boundary, with invariants
   (everyone in exactly one executor, one owner per asset, one debit and credit per transfer), and
   no warm-up day that eats or earns twice.
5. **Share the rules between modes** (01-04 §4.7; 01-03 §4.1): eligibility rules and recipes are
   the same; a coarse mode declares what it approximates and is tested against the detailed one at
   several horizons, on distributions and intervention responses, not only means. Calibration to
   the detailed model is internal consistency, not realism.
6. **Draw shared shocks at their scope, conserve stocks, never clamp** (01-04 §4.4): a drought
   strikes related farms together; production is allocated before it is committed.
7. **Keep resolution switches rare** (01-04 §2): conversion pays only after about
   (C↓ + C↑)/(T_f − T_c) days of coarse residence, so use hysteresis and a minimum residence.
8. **A skip is the same machinery with another scheduler** (01-04 §4.8; 01-03 §4.2): advance to
   the next relevant boundary, resolve, reassess; never one average growth rate over thirty years;
   keep settlements synchronised where they interact.
9. **Climate change is specified forcing** (03-03 table "Social/technological context"): change the
   background climate as an input, not as an era flag; a settlement does not warm a planet.

**What the precedents teach to avoid:** a coarse rule that changes outcomes wholesale (Dwarf
Fortress 0.40.16, attackers always winning off-screen battles); stale coefficients after a
capability changes (X4's ships built without weapons); weighted representative agents as an
identity backbone (Covasim's resampling).

## 1. Why no change to the event loop reaches 100k

The 10k day's last profile (M8a, slices CI–CL) put about half in deciding, a seventh in starting
work at the hearth (company), about an eighth in route searches, and under 1 % in midnight's own
work. People make about 14.5 decisions a person-day. A year a minute at 100k is 1.6 µs a
person-day; today a person-day costs about 5 ms. The levers ADR-0028 listed (indexes, constant
factors, routing, parallel preparation, approximations) together reached 1.3–2× in M8. A
lightweight individual that still decides each activity and only skips the walk saves the route
share and part of the company: 1.3–1.5×. The gap is three orders of magnitude, so the coarse mode
must decide less often, not decide more cheaply.

## 2. A day resolved per household

The cheapest resolution that keeps every person and every rule is **a household's day resolved at
midnight**: the household's view (already built for Accelerated mode) and its members' facts are
scored once with the same `decide::candidates`, and the members' waking hours are shared among the
behaviours in proportion to the choice probabilities the softmax would give, after the hours that
needs fix (sleep, meals, water). Each block's effect is applied through the behaviour's own effect
function, with the minutes, target and goods the event path would have used, and with the travel
time read from the settlement's travel field instead of a route.

- **Cost** grows with households and behaviours chosen, not with steps: about one scoring a
  household-day against about 70 decisions for a household of five.
- **What it keeps:** identities, households, stores, the ledger's channels, fields, buildings,
  projects, laws, offices, ties and beliefs; midnight's work unchanged; the same priors and recipes.
- **What it approximates, declared:** the order of a day's activities, who meets whom (company drawn
  from the settlement's pool once a session-day, not a hearth roll per session), the walk itself
  (no wear laid step by step: a household's ways worn by its travel minutes along its habitual
  routes, or not at all, decided in the ADR), contention within the day (two households wanting
  one seller's last sack resolved in household id order at midnight), and fire and other
  per-minute incidents (none start in a household-resolved settlement, or they start at a declared
  hour).
- **What it cannot do yet:** behaviours that are journeys to another place and time-critical joint
  work (a fire's dousing, a band, a visit, a fetch, a gathering's attendance). Each needs a stated
  contract: either resolved as a block with its effect (attendance counted, a fetch's goods
  arriving the next midnight), or not taken while household-resolved, listed and tested.
- **Parallel and exact:** each household's plan reads the world and writes nothing; plans are
  applied serially in household id order. That is ADR-0028 §2's pattern for midnight preparation,
  so thread count cannot change an outcome (Gate A at 1, 2, 4 and 8 workers).

Cohorts proper (counts by group, members chosen when an event happens) are the third mode. They are
cheaper again only where households are many and alike. Whether they are needed is measured after
the household day is built: if 100k does not reach its figure with every distant settlement
household-resolved, cohorts are the next lever, not the first.

## 3. Switching

- **At midnight only**, as ADR-0011 already switches modes. Demotion ends the day's open activities
  as midnight finds them (most people asleep at home), banks loads and places everyone at home;
  promotion starts each person's first decision at their next waking, as founding does.
- **A run setting, not world state.** Which settlements are household-resolved is chosen by the
  host (the observer's focus, with hysteresis and a minimum residence of days) or by the bench; a
  save is valid in either resolution, as every save is a valid Detailed state today (ADR-0011 §1).
- **Invariants checked at every switch in tests:** the same living people with the same households,
  the same owners of every field, building, good and claim, the ledger's totals unchanged, nobody
  in two executors, and a settlement switched down and up at the same midnight lives on as if never
  switched (Gate A).

## 4. Between settlements

A household-resolved settlement is lived at midnight; a Detailed one through the day. Where they
meet, the boundary is midnight: a Detailed visitor finds the hearth's company drawn from the
residents' pool; a Detailed buyer at a household-resolved seller buys from its stores as they stand
(the seller's day already applied); a household-resolved buyer's fetch from elsewhere is resolved
as a block whose goods arrive at the next midnight. Moving, marriage and founding are already
midnight's. Word, opinion and ties move at the hearth through the drawn company.

## 5. Many settlements

Worlds today are founded with up to five groups. M9 needs 10–30 settlements: founding many groups
on a larger map (setup, as ADR-0018's groups are), the bench founding 100k, memory per person
measured (plan §4.6: 4 GB at 50k; M8 measured 0.92 GB at 10k, which would be about 9 GB at 100k),
saves of that size measured, and the observer and wire carrying each settlement's resolution.

## 6. Time skips

Plan §8 asks whether to keep them. 01-04 §4.8 and 01-03 §4.2 answer how: the same machinery, every
settlement household-resolved, Accelerated Max, stopping at midnight when the observer asks or a
date is reached, every consequential event recorded as it happens. Nothing is generated after the
fact, so a skip is just a fast run and stays deterministic and saveable. The decision left is
whether the observer is offered "skip to" as its own command; the cost is the household day's.

## 7. The climate tool

03-03 asks for climate change as specified forcing: a change in the background (monthly
temperature normals and precipitation amounts) applied as an input and ramped over years, not an
era flag and not caused by a settlement. The weather generator (ADR-0012) draws each day from the
landscape's monthly normals and a slow anomaly; an offset to the normals and a factor on wet-day
amounts and wet-day share, recorded as an influence (ADR-0022 §13) with its ramp, changes every
consequence (harvest factor, snow, water, dryness) through the existing chain. The harvest factor's
long-run mean, drawn once a world is made or loaded, must stay the unforced climate's, or a warmer
climate would be renormalised away.

## 8. Guards

- No state that exists in one resolution only; a household-resolved settlement saves the same
  sections as a Detailed one.
- No draw keyed by resolution, worker or call count; every draw keyed by seed, purpose and identity.
- A coarse rule declares what it approximates and passes a consistency gate against Detailed before
  it is the default anywhere.
- Coefficients are not fitted and frozen: the household day calls the live scoring, so a new law,
  technique or building changes it at once (X4's lesson).
- Nothing synthesised overwrites a record; nobody is created or removed by a switch.

## 9. Suggested slices

1. **The switch that changes nothing**: resolution per settlement as a run setting, the midnight
   transaction, invariants, a many-settlement bench; a household-resolved settlement may at first
   live a declared minimal day only in tests.
2. **The household day for the core behaviours**: sleep, eat, water, gather, farm, build, make,
   leisure, ask, trade within the settlement; company from the pool.
3. **The rest of the behaviours**, each resolved or listed as not taken: public works, offices and
   gatherings, taking, tending, hiring, carts and boats, fetching, carrying, visiting, fire.
4. **The consistency gate and parallel planning**: household-resolved against Detailed on Gate B's
   aggregates and on intervention responses; exact at any thread count; measured.
5. **Many settlements in the observer**: focus decides resolution with hysteresis; the wire carries
   it; 10–30 settlements founded; 100k measured.
6. **Time skips, the climate tool and the M9 demo.**

## 10. Open questions for the designer

1. Does a household-resolved settlement lay path wear (its trails would freeze otherwise)?
2. Which behaviours are not taken in household resolution, and is any of them consequential enough
   to keep a settlement Detailed while it is pending (a fire, a gathering called, a band out)?
3. Is resolution chosen by the observer's focus alone, or may the observer pin a settlement?
4. Does the consistency gate compare at a year only, or also over ten years (01-04 §4.7's horizons)?
5. Is "skip to" a command of its own, or Accelerated Max with every settlement household-resolved?
6. Does the climate tool act on the whole map (one weather series a world) or by region?
7. Are cohorts built in M9, or only if the household day misses its measure?
8. What is M9's scale figure: 100k at a year a minute is 1.6 µs a person-day on the container,
   which nothing measured suggests; is the gate set from the household day's measurement?

## 11. What an ADR must decide

The resolutions and what each keeps; the household day's rule (scoring once, sharing hours, effects
through the behaviours' own functions); the switch transaction and its invariants; the contract at
the boundary between resolutions; which behaviours are not taken; the consistency gate; time skips;
the climate tool's input; and M9's scale figure.
