# ADR-0045: Food that carries what its maker shed

Status: Accepted (2026-11-18). Decided by Claude under standing owner authority (2026-11-18):
option B, as the smallest design that covers what research 05-07 grounds, costs nothing where
nobody sheds, and only appends codes to saves 220, where option A would rewrite every store path
and grow every save for provenance nothing yet needs.
Date: 2026-11-18
Milestone: M6a's deferral (ADR-0021's "Revisit when": "Goods carry batch identity, making food a
route")
Amends: ADR-0021 §4 (a fifth way loads move: with food), ADR-0006 (a good's stock
may carry a load)

## Context

Shedding where people work is complete except for one route (plan §7, backlog E). Research 05-07
found no rate for people who work or visit together catching enteric disease from each other by
hands or tools. What it did find is that workplace and communal outbreaks of cholera, typhoid and
shigellosis are **common-source** events, mostly food handled by a carrier:
- a carrier who prepared a luncheon infected 18 of 310, an attack rate of 5.8 %;
- a sweet shop's food handler kept serving while ill;
- one workplace's shigella came from food left in a break room, and another's from a cafeteria
  worker back from travel.

Research 05-03 §1 names the mechanism: "infected people contaminate specific water sources,
drainage paths, hands or food batches", and "a contaminated well and an infectious cook create
different outbreak patterns".

ADR-0021 deferred this route because goods carry no identity. The route needs three things:
1. A shedder who **prepares** food passes a load to what they prepared.
2. The load **goes with that food** when it is sold, given or handed out.
3. Whoever **eats** it takes a dose in proportion to what they ate.

What must not happen, for two reasons:
- **No double count inside the household.** The household hazard (`household_hazard`, a tuning
  value) already stands for spread within a home, and 05-07 §1.1 puts most of it on shared food
  and water. So the route must matter where food *leaves* the household that prepared it.
- **Nothing creates a pathogen** (ADR-0021 §4).

**How food moves today.** I read the code and measured what I could:
- **Stores are kilograms by good, one number each.** Holders are household `stores: Vec<f64>`,
  and the same type on polities (common store), firms, consignments in transit, custody,
  factions and agreements. Of the 77 goods, 11 are food: bread, dried fish, dried meat, fish,
  flour, grain, groats, meat, provisions, seed grain and wild plant food.
- **The code that touches stores is spread out.** I measured 386 `.stores` references in 53
  files of civ-agents, and 42 direct writes (`stores[g] +=`, `-=`, `=`) in 22 files.
- **Eating is per person, with no shared meal and no cook.** Each person who chooses Eat takes kcal
  from the household's pooled stores, in keeping order (`Population::eat`, `eat_from`).
  "Cooked" goods (`Eaten::Cooked`) need only firewood in the store; nobody prepares a meal.
- **Preparing food is making it.** The recipes are `bake_bread`, `bake_bread_oven`, `dry_fish`,
  `dry_meat`, grinding to flour or groats, and provisions. They are made at home (`make_work`)
  or for a firm (`firm_make`), by a named person in a work step.
- **Food leaves a household by these paths:**
  - market trades settled at the seller's door (`settle_trade`);
  - the common store's levy and relief (`give_relief`);
  - gifts and debts in goods;
  - takings;
  - inheritance (stores pass to an heir);
  - consignments in transit;
  - a firm's output sold from its stock.
- **Cost of a day (measured).** `bench_village --people 1000 --days 5 --max` lived 720 ms a day
  on average on the container (p50 667 ms, p95 903 ms, 0.23 years a minute, 157 MB peak). People
  spent 1.26 hours a person-day eating and 0.44 hours making.
- **Cost of the options (estimated, not measured).** The number of eating and making acts a day
  was not instrumented. Changing no code, the figures below are estimates from the structure of
  the code.
- **The budget.** ADR-0028's "Revisit when" reopens scale when a feature "adds more than a fifth
  to a person-day's cost". In a world with no disease, the dashboard's worlds, the cost should be
  nothing.

## Options

### A. Full per-batch provenance

Every holder's stock of a food good becomes a list of batches: kilograms, the day made or got, and
where and from whom it came. Each batch keeps its own load per disease. Eating takes oldest first,
and transfers split batches.

- **Covers** every route that food could ever carry: provenance for an observer's trace ("the
  bread from Ada's bakery"), spoilage by batch, and traceability for later laws.
- **Code cost:** every store path that touches food. That is all 42 direct writes, the
  bulk-settling (`settle_stores`, spoilage by half-life) and every transfer, across all seven
  holder types. It is the largest change since the ledger (ADR-0006).
- **Save format:** every holder's stores section gains a batch list per food good. A household's
  saved stores grow from 77 numbers to 77 plus a few batches per food good, estimated at 10–40
  entries.
- **Performance (estimate):** every store operation in every world becomes a list operation, with
  or without disease. Spoilage loops over batches; eating and selling split and merge them. The
  cost falls on the dashboard's worlds too. Store operations' share of a day was not profiled, so
  the size against the fifth is unknown. That uncertainty is itself a cost.
- **Lost** on: "the smallest grounded design first" (plan §1). 05-07 grounds one route (prepared
  food from a shedder), not traceability.

### B. A load on a holder's stock of a prepared food (recommended)

No batches. As stored water already does, a holder's stock of a good carries a load per disease
(new contagion nodes keyed by holder and good). Its concentration is the load over the stock's
kilograms, mixed well within the stock (ADR-0021 §4's mixing in stored water).

- **Made:** when a shedder of a water-route disease makes an edible recipe output in a work step,
  a load goes onto the output's stock in the holder's store. The holder is their household, or
  the firm they work for. Its size is a design prior: a share of the day's shedding by the hours
  of the step, or by kilograms made. 05-07 gives no transfer figure, so it is calibrated against
  §2's 5.8 % luncheon attack rate.
- **Moves:** a transfer of food that the route handles (market trades, relief, firm sales) moves
  its share of the load, by kilograms.
- **Reconciled, never created:** any other path that lowers a stock lowers its load in
  proportion, as if the load left with the food unseen (eaten by a path the route does not
  dose, spoiled, taken). A path that raises a stock dilutes it, because what comes in is clean. Each node
  keeps the kilograms it was last reconciled at, so unhooked paths can never make a dose.
- **Dosed:** eating `kg` of a good takes `kg / stock` of its load and puts that dose on the eater
  (the dose and the draw of ADR-0021 §5's exposure). The exception is a load **made by the
  eater's own household**, which moves but is not dosed. The household hazard already stands
  for spread within the home. So load the household's people made is kept on a node of its own
  ("made here"), and load that came from elsewhere on another ("got").
- **Code cost:** new hooks at the making step, at eating, at three transfers (trade, relief, firm
  sale), and at reconciling. Two contagion node kinds and an acquisition kind (`Acquired::Food`)
  are appended. All other store paths are untouched.
- **Save format:** none new beyond appended node codes and an acquisition code. Contagion loads
  are already saved generically by node, each with its reconciled kilograms; append only, saves
  220 while unreleased.
- **Performance (estimate):** in a world with no shedding, nothing. The hooks test for an empty
  food-load map first, as the sewer's leak and slice 1's hours do. With an outbreak, there is one
  map lookup per eating act and per hooked transfer of a loaded stock. That is hundreds of
  nanoseconds against a 720 µs person-day, far under the fifth.
- **Covers** 05-07's common-source events: a carrier baking for a firm whose bread is bought
  around the settlement, and a household selling or handing out what it made. It does not cover
  provenance across many hands.

### C. A per-meal exposure event, no stored food

Nothing is stored. When a shedder makes food, the day's makers are noted, as slice 1 notes hours.
Anyone who eats that day from a store that a maker's work went into takes a hazard.

- **Code cost:** smallest, with no new saved state beyond a daily note.
- **Save format:** a daily vector, as `worked` is.
- **Performance:** negligible.
- **Lost** on the evidence. Food keeps for days, bread for a week, and dried fish and meat for
  months, and it is sold. 05-07 §2's outbreaks are of food *carried to others* (a luncheon, a
  shop's sweets). A same-day event either never reaches the buyer or must re-derive "which store
  got the maker's work". That is option B without its conservation. Inside the household it
  double counts with `household_hazard`.

## Decision

**Option B**, built in slices, the smallest grounded first (accepted 2026-11-18).

## Slices (if accepted)

1. **A firm's prepared food.**
   - *Built:* a shedder who makes bread (or another edible output) for a firm loads the firm's
     stock. The firm's sales move the load to the buyer's "got" node. A buyer who eats from it
     takes a dose (`Acquired::Food`, naming the firm). Unhooked paths reconcile, never create.
     Saved.
   - *Tests:*
     - A carrier baker's load arrives in a buyer's household, and the buyer's members take
       doses in proportion to what they eat.
     - Nothing goes to a household that bought nothing.
     - Loads plus doses plus what reconciling lost equal what was made (conservation), across
       spoilage and an unhooked transfer.
     - It saves and loads exactly.
     - A world with no shedder is byte-identical in every section.
     - It fails without the hook.
2. **A household's own prepared food leaving it.**
   - *Built:* the "made here" node. Market trades and gifts move it to the buyer's "got".
   - *Tests:*
     - The household's own members take no dose from their own bread.
     - A neighbour who buys it does.
     - Household exposure counts are unchanged where nothing leaves.
3. **The common store, relief and feasts.**
   - *Built:* the levy takes its load share into the store, and relief hands it out.
   - *Tests:* relief from a store a carrier's food went into doses those relieved.
4. **The observer and calibration.**
   - *Built:* the incidents panel names food sources ("bread from Ada's household's bakery").
     The load-per-hour prior is set against 05-07 §2's 5.8 % luncheon attack rate, measured
     in a probe world and logged as `NUDGE:`/tuning.
   - *Tests:* the panel's words, and the probe's attack rate within the recorded tolerance.
5. **Later, behind the slices:** clean hands (`core:technique/clean_hands`, already in the
   content) lowering what a maker passes to food; food handlers kept from work by a law (a polity
   change, outside these slices).

## Consequences

- **Easier:** outbreaks from a carrier baker or a market stall become possible, as 05-07 reports
  them. Nothing is scripted: who works, who buys and who eats is people's.
- **Harder:** a second kind of mixing to keep conserved (food beside water), and reconciling for
  every unhooked path.
- **Forbidden by the design:**
  - dosing a household's own people from its own food;
  - creating a dose on any unhooked path;
  - any choice reading a food load (ADR-0021 §5: no choice reads a source's load).

## Alternatives considered

- **A. Full provenance:** lost on cost against what 05-07 grounds, and on its cost to every
  world without disease.
- **C. Per-meal events:** lost on the evidence (food is kept and carried) and on double counting
  within households.
- **A co-worker contact hazard:** lost in 05-07 §4, since there is no figure and it double counts
  shared sources.

## Revisit when

- A law or an observer view needs where a particular food came from beyond one holder (then
  option A).
- Meals become shared events with a cook (then the "made here" node may dose a household's own
  people, and `household_hazard` is re-fitted).
- A disease that passes by food but not by water is authored.

## Slice 1 as built (2026-11-18)

- **Making:** `firm_make` calls `Population::food_made` once a session's output is in the
  workshop's stock. For each water-route disease its maker sheds today and will tomorrow,
  `food_share` × the day's rate × the session's hours ÷ 24 goes onto the edible outputs by their
  kilograms. The core's `[contamination] food_share` is 0.1, content API 233, a design prior to be
  calibrated in slice 4. The amount is taken ahead from the next midnight's shedding (`owed`),
  never more than that day's, and recorded as `How::Handled` (code 6).
- **Moving:** `transfer`, through which trades, gifts, wages and draws from a workshop all pass,
  moves a loaded stock's load by the kilograms moved, for any holder.
- **Eating:** `eat` doses the eater by the share of each loaded stock they ate. The dose goes to
  `Contagion::eaten`, which the midnight's exposures draw as `Acquired::Food` (code 35).
- **Reconciling:** each node keeps its last reconciled kilograms (`food_kg`). A stock found lower
  loses its load in proportion; a stock found higher is diluted. No dose is made.
- **The node:** `Node::Food { holder, good }`, code 10, its number the holder's id with the good
  above bit 48.
- **Saves:** `food_kg`, `owed` and `eaten` are appended to the contagion section, each absent while
  empty.
- **A world with no disease:** every save section's digest is the same as before the slice, in
  `bench_village` runs of 300 people for 30 days, and of two groups of 600 for 40 days at Max.
  Only the content and the meta that names it differ.
- **Left for slice 2:** the "made here" node. A workshop's owners who draw its loaded bread are
  dosed as any buyer is.

## Slice 2 as built (2026-11-18)

- **The node:** `Node::MadeFood { holder, good }`, code 11, holds what a holder's own people made.
  That is a household's own baking (`make_work`), or its workshop's when one of its people made
  it.
- **Moving:** within the household and its own workshop the load stays made here. To any other
  holder it becomes `Food`, food got from elsewhere.
- **Eating:** both loads fall by the share eaten. Only `Food` doses the eater.
- **A world with no disease:** every save section's digest matches the build before, but `meta`,
  which holds the wall-clock time the world was made.
