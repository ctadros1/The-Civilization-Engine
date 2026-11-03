# Backlog C design brief: salvage, the sewage farm, and mending sewers

**Scope.** What ADR-0038 §6 and ADR-0041 §4 left: salvage trades beyond what a recipe already
takes; the sewage farm; and sewers that wear, block and break, and are mended. Toxicity of
residues, covering and burning tips, and hog feeding stay deferred (no harm model, no beasts).
Written 2026-11-01 by Claude under the owner's standing authority. The decisions that are
expensive to reverse are drafted in ADR-0044 (proposed).

**Read:** 12-10 §1.2, §1.6, §1.7, §1.10, §2.2, §3, §4, §5.6, §5.8, §5.10, §6; 12-02 §1.5–§1.7,
§2.3, §4, §5.5; 12-08 (public works) §1.4, §2.1–§2.3, §4.3–§4.7, §5.1–§5.4. Citations are to report
sections. Where the reports give no figure, the value below is a **design prior** and says so.

## What the code already has

- **Heaps and tips** (`civ-agents/src/person.rs` `Midden`; `polity.rs` `Tip`): a decaying share
  (`kg`) and ash (`ash_kg`), nothing else. Residues (`ash`, `slag`, `sludge`, ...) are goods offered
  by nobody, since nothing gives them an own cost (`value.rs`).
- **Tools wear away** (`make.rs::wear`): "a tool worn out is gone", with its metal. An iron axe
  takes 1.2 kg of bar, a hoe 0.8, a sickle 0.4 (`content/core/recipe/forge_iron_*.toml`).
- **The refuse round** (`population/rounds.rs`): a fee a call of the keeper's hours at its
  `hour_kcal`, paid in food on the `Refuse` channel; a home joins only where a law asks it to carry.
- **Sewers** (`population/sewers.rs`, `civ-land/src/sewers.rs`, ADR-0041): laid by `lay_sewer` as
  common work; connections served in order up to `capacity_kg_day`; `Sewer.broken` is saved but
  nothing sets it; `life_years` (50) is read only by the laying forecast. A settling tank splits
  the day's load into `sludge` and the rest; loads held at `Node::Sewer` (code 7).
- **Fields and soil** (`civ-land/src/soil.rs`): `manured(n_kg_ha)` feeds the pools; each harvest
  records whether season or soil (`Limit::Soil`) held it back. The heap's dung carries 1.0 kg of N
  a person a year at 0.5 kg a person a day (`content/core/people/early_farmers.toml`).
- **Precedents for wear and mending:** a well's lining loses `loss_per_year` × wetness and falls
  in at its quality; relining is weighed as hours spared against labour (`population/wells.rs`).
  A ditch has `silt_m`, which nothing raises yet (`civ-land/src/ditches.rs`).

## 1. Salvage (12-10 §1.6, §5.6; 12-02 §5.5)

**What the research says.**

1. **Salvage is a trade whose sign can flip** (12-10 §1.6, §4). London's dust contractors paid for
   the right to collect while brickmakers bought the fine ash ("soil") and cinders ("breeze");
   within a generation parishes paid them, because the brickfields had been built over. Neither
   the refuse nor the work changed; the haul to the buyer did.
2. **Its worth follows the prices of what it replaces** (§1.6): rags fell to wood pulp; a Paris
   container rule was accused of halving ragpickers' income; the Zabbaleen recycle about 80 % of
   what they collect against 25 % for contractors (M).
3. **Who picks is set by what an hour brings elsewhere** (12-02 §5.5's margin; §1.7: keep the
   workers visible, never as a property of a group).
4. **What a coal town's refuse held was mostly ash** (12-10 §2.1: 57–91 %), so the bulk salvage
   was brick material; metal, rags and bones were small fractions of value (§1.2's table).
5. **No sorting rates, picker earnings or ash shares in brick bodies were found** (12-10 §6: "no
   collection productivity was found"). Every rate below is a design prior.

**Causal rules (proposed).**

- **S1. What is thrown away stays somewhere.** A tool worn down leaves, with its wear, the scrap
  its content names (`[scrap]`: a good and kilograms a whole tool). A household's goes on its heap
  as a **discard**; a works' goes to its store, as residues do. A heap and a tip hold discards by
  good beside their decaying share and ash. Carrying, tipping and a round's load move them whole;
  a sewer, manuring and the destructor do not take them.
- **S2. Picking is a way to get a good.** A member may sort a heap or a tip it may use (its own
  heap; its polity's open tip) at `sort_kg_h` of what lies there an hour, taking out the discards
  and ash it wants. Its own cost of a good by picking is hours over kilograms: `1 / (sort_kg_h ×
  share of that good at the site)`, plus the walk. That gives salvage an own cost, so it enters
  `value.rs`, wants and the market like any gathered good: no salvage price is authored.
- **S3. Picking pays where its hours are cheap or its goods dear.** A household picks to use or to
  sell when picking's cost in its hours is below its other ways to the good, or below what the
  market pays at its `hour_kcal`. Households whose hour brings little food (poor land, no tools)
  reach the threshold first. Nothing names a poor class.
- **S4. Salvage has uses only through recipes.** `good/scrap_iron` is welded into bar
  (`forge_scrap`); ash replaces part of the clay in `fire_bricks_with_ash`. Where no smith or
  brickmaker wants them, nobody picks.
- **S5. A round may sort its load.** A posted round's keeper may sort its cart before tipping and
  keep what it wants. The fee becomes `(run hours + sorting hours) × hour_kcal + fuel − worth of
  what it keeps`, over the homes. It may fall below zero: the keeper then pays each home on the
  same `Refuse` channel, and a home joins with no law asking. This is 12-10 §4's sign flip.

| Parameter | Value | Source; confidence |
|---|---|---|
| Ash share of household refuse, coal town | 57–91 % | 12-10 §2.1; M (emerges from fuels, not set) |
| Scrap a worn iron tool leaves | half the bar it took (axe 0.6 kg, hoe 0.4, sickle 0.2) | design prior |
| Welding scrap to bar | 1.1 kg scrap and 4 kg charcoal a kg of bar, 2 h | design prior, inside forging's 1–5 kg charcoal and 2–10 h a kg (07-05 §2.3, as `forge_iron_axe` cites) |
| Sorting rate | 100 kg of heap or tip an hour, by hand | design prior (12-10 §6: none found) |
| Ash in a brick body | a quarter of the clay replaced, kg for kg | design prior (12-10 §1.6 names the trade, not the share) |
| Who may pick a tip | any household of the polity whose tip it is | design prior |

**Not built here:** rags, paper and glass (no cloth or glass in the vocabulary; 12-10 §5.10 defers
them); bones (no butchery gives them, and nothing uses them); wood from ruins (demolition, a
separate item; 12-08 §5.4); picking another household's heap; the law's round sorting.

## 2. The sewage farm (12-10 §1.7, §2.2; 12-02 §1.6, §2.3; ADR-0041 §4)

**What the research says.**

1. **A farm is a field receiving load, limited by area** (12-10 §1.7): Berlin about 19,000 acres
   for 1.5 million (about 0.5 ha a hundred people); Freiburg 500 ha for over 50,000 (at most 1 ha a
   hundred); both M, the ratios the report's arithmetic.
2. **Nutrients and pathogens are separate properties** (12-02 §1.6): application raises a crop
   only where that nutrient limits it; pathogens go to the soil separately; aged is not sterile.
3. **Application rates** (12-02 §2.3, M): 50–100 kg N/ha in urine-use guidance; one person's urine
   serves 300–400 m².
4. **Farms gave way to treatment and left their soils behind** (12-10 §3, §4: Berlin's peak of
   10,000 ha in 1928, 1,250 ha by 1992, contaminated soils).

**Causal rules (proposed; ADR-0041 §4 made concrete).**

- **F1. Where a farm can be.** A field any of whose cells lies beside the sewer's route below its
  lowest connection (so the whole flow passes it) may take the flow. The order at the outfall is:
  settling (if a tank), then the farm up to its limit, then the reach. Mass and every load are
  conserved.
- **F2. Its limit is its area.** At most `farm_people_ha` people's discharge a hectare: in kernel
  units 100 × 0.5 kg = 50 kg a day of decaying share a hectare. Beyond it the rest goes on to the
  reach.
- **F3. The yield comes from the soil, not a bonus.** What the field takes enters
  `soil.manured()` at the heap's nitrogen a kilogram (`MiddenParams::n_per_kg`). At the limit that
  is 100 kg N/ha a year, the top of 12-02 §2.3's 50–100. The crop rises only where its last
  harvest was held back by the soil.
- **F4. Its risk is the field's place.** The day's disease loads go to a new contagion node,
  `Node::Field` (code 8, appended), which washes and seeps from the field's centre by the links a
  tip uses. A well downslope within reach takes load; the reach below takes less.
- **F5. Who decides.** The field's household takes the flow, for nothing, in a year after a harvest
  `Limit::Soil` held back, and stops when the soil no longer limits it. People do not tie sickness
  to a field (suspicion tallies water sources only), so the farm runs on even when its wells
  sicken. 12-02 §1.7: beliefs about cause stay separate from transmission.

| Parameter | Value | Source; confidence |
|---|---|---|
| Area limit | 100 people's discharge a hectare | ADR-0041 §4 design prior; checked against 12-10 §2.2's 0.5–1 ha/100 (M) |
| Nitrogen delivered at the limit | 100 kg N/ha·yr | arithmetic on the core's 1.0 kg N a person-year; 12-02 §2.3's 50–100 (M) |
| Water with the flow | not carried | design simplification: sewers carry kilograms of decaying share, not water |

**Not built here:** produce eaten raw and farm workers' exposure (no food or handling route
exists); helminth maturation (no helminths); the metals left in farm soils (no harm model;
ADR-0038's revisit); selling sewage to farmers; sludge spread as dung (plan §7's "Not yet").

## 3. Sewers that wear, block and break (12-02 §1.5; 12-08 §1.4, §2.1, §4, §5)

**What the research says.**

1. **A sewer has blockage, leakage and maintenance** (12-02 §1.5): refuse and sediment reduce its
   capacity; clearance and maintenance are recurring work.
2. **No universal decay rate exists** (12-08 §2.1): deterioration follows material, drainage,
   load and history. Use three processes: a stock of obstruction, a condition, and failure
   conditional on condition (§5.2).
3. **Routine care does not substitute for renewal** (§4.3), and an unsuitable repair cures nothing
   (§5.3): clearing does not mend a collapse; relining does not clear silt.
4. **A break near a critical link costs more** (§4.7). **Snapshot functioning is not reliability**
   (§4.5): record outage durations, not just working or broken.
5. **Maintenance needs an organisation of its own** (§1.4); its work is limited by the scarcest
   input, and lost revenue can lead to a growing backlog (§5.4). Abandonment can be rational.
6. **No sewer deposition, exfiltration or clearing rates were found** in 12-02, 12-10 or 12-08
   (12-08 §2.2's ditch siltation is itself an assumption). Every rate below is a design prior.

**Causal rules (proposed).**

- **M1. Deposit.** Each day `deposit_share` of what enters stays in the sewer as deposit (mass
  only; its pathogens are taken to pass with the flow, a simplification). Its effective capacity
  is `capacity × (1 − deposit / (block_kg_m × length))`. Connections beyond it are unserved and
  heap again, in reverse order of connecting: **backing up** shows as heaps regrowing at those
  homes, and their loads reach wells by the heap's existing links. At the full deposit the sewer
  is blocked (cause `Blocked`, appended).
- **M2. Wear by cell.** Each month every cell of the route loses `1 / life_years` of its soundness,
  times its ground's wetness and the month's rain (the wells' factors, `condition::WET_WEAR`). A
  cell whose loss reaches 1 collapses: the sewer is broken there. Connections above the break back
  up (`Broken`); those below it still flow. A break near the outfall cuts more homes than one near
  the head.
- **M3. Failure is seen only when it shows** (12-08 §5.1). The polity learns of a fault when a
  connection backs up for want of capacity it once had, or at a collapse. No inspection in v1.
- **M4. Mending is common work under the law that laid it.** While the `lay_sewer` law is in force,
  the polity's review weighs a fault as it weighed laying: the hours the homes above it would be
  spared over the sewer's life left (carrying under laws, deaths it believes avoided) against the
  work and materials. If worth it, it asks each household an equal share (or pays it by
  `pay_work`), as laying did. **Clearing** removes the deposit at `clear_h_t` hours a tonne, and
  the deposit becomes `sludge` in the keeper's store. **Relining** a collapsed cell rebuilds its
  length at laying's hours and materials a metre and sets its loss to 0. Neither cures the
  other's fault.
- **M5. Without the law, nobody mends.** A sewer whose law ended keeps carrying until it blocks or
  breaks, and then its homes heap as before. No polity is forced to keep it.
- **Leakage is deferred.** It needs a contagion node at the worn cell and a rate the research does
  not give; contamination from failure shows through backing up in v1.

| Parameter | Value | Source; confidence |
|---|---|---|
| Deposit share of what enters | 0.02 | design prior |
| Deposit that blocks a metre | 20 kg/m | design prior |
| Clearing | 10 h a tonne | design prior, near ADR-0010 §2's 8 h/m³ of digging |
| Wear | 1 / `life_years` a year at average wetness (core 50 years) | ADR-0041 content prior; the wells' wetness factors |
| Relining a collapsed cell | laying's hours and materials for its length | rule (12-08 §4.3, §5.3) |

At these priors a sewer of 500 m at its 1,000 kg a day gathers 20 kg a day and blocks in about 500
days unless cleared: about 100 hours a year of clearing at full flow (arithmetic on the priors).

## 4. What is measured

These are stylized facts to check, not targets to tune toward.

| Fact | Research | Test |
|---|---|---|
| Mass closes | 12-10 §4 invariants | Every kilogram of worn iron ends as scrap on a heap, a tip, a cart or bar again; picking, farming and clearing conserve mass and loads |
| Salvage's sign follows prices and haul | 12-10 §4 | With a smith or brickmaker near the tip, a posted round's fee falls, and may go below zero; take the buyer away and it rises, with the refuse unchanged |
| The poor pick first | 12-10 §1.6; 12-02 §5.5 | Of two households at one tip, the one with the lower `hour_kcal` picks at a share where the other does not |
| Nothing is picked without a use | 12-10 §5.6 | A world with no forging or brick firing picks nothing |
| A farm feeds only a hungry field | 12-02 §1.6 | A farm's grain rises where `Limit::Soil` held it, and not where the season did |
| The farm's area check | 12-10 §2.2 | Hectares a hundred connected people on farms taking the whole flow: about 1 (0.5–1 is plausible) |
| A cleaner river, a dirtier field | 12-02 §1.5 | With a farm, the reach below the outfall takes less load and a well beside the field more |
| Outages are recorded | 12-08 §4.5 | Each connection's days unserved a year, by cause |
| Care is not renewal | 12-08 §4.3 | A cleared sewer still collapses at its wettest cell |
| A critical break costs more | 12-08 §4.7 | A collapse at the outfall end unserves every connection; one at the head, only those above it |
| An ended law leaves a failing sewer | 12-08 §5.4 | With the law repealed, deposit and wear go unmended, and heaps regrow |
| Guards | ADR-0038, ADR-0041 | Without worn iron tools, tips or sewers, every lived figure is unchanged |

## 5. Slices

Each is about a day's work and ends green on the ten smoke worlds.

1. **Discards and scrap (S1, S4's scrap).** Tool `[scrap]` content and its validation; `make.rs`
   `wear` returns what it wore, so the household's heap or the works' store takes the scrap; heaps
   and tips gain discards; `middens.rs`, `tips.rs` and `rounds.rs` carry them; `good/scrap_iron`
   and `recipe/forge_scrap` with its activity. Areas: `civ-agents/src/make.rs`, `person.rs`
   (`Midden`), `polity.rs` (`Tip`, `RoundCall`), `population/{middens,tips,rounds}.rs`,
   `civ-content` (the tool table), `civ-sim/src/persist` (the households, tips and rounds
   sections), `content/core/good`, `recipe`, `activity`. **Tests:** a new `salvage` target in
   `civ-sim/tests/`: an iron axe worn through leaves 0.6 kg on its heap; carrying, tipping and a
   round move it exactly; a destructor's draw and a sewer leave it; save and load exact; the
   `residues`, `tips`, `rounds`, `iron`, `metal` and `roundtrip` targets.
2. **Picking and selling (S2–S5).** A `pick` behaviour and activity at a heap or tip; picking as a
   way to get a good in own costs; `recipe/fire_bricks_with_ash`; a posted round's sorting and a
   fee that may fall below zero. Areas: `civ-agents/src/params.rs` (`Behavior`), `decide.rs`,
   `value.rs`, `population/market.rs` (offers of picked goods), `population/rounds.rs`,
   `content/core/activity`, `recipe`. **Tests:** in `salvage`: picking draws exactly what it takes
   from a tip; its cost falls as the tip's share rises; the lower-`hour_kcal` household picks first;
   a brickmaker takes ash only where picking beats digging clay; a round's fee changes sign when a
   buyer appears; `market`, `firms`, `rounds`, `tips`.
3. **The sewage farm (F1–F5).** A sewer's farm field (saved), its limit, nitrogen to the soil and
   loads to `Node::Field`. Areas: `population/sewers.rs`, `polity.rs` (`Sewer`, `SewerDef`
   `farm_people_ha`), `contagion.rs` (code 8), `population/contagion.rs` (the field washing and
   seeping as a tip), `civ-land/src/soil.rs` (read only), `civ-content` (the `[sewer]` table),
   `civ-sim/src/persist`, the services and outbreak words in `civ-sim/src/frames`. **Tests:** in
   `sewers`: mass and loads conserved through tank, farm and reach; the limit by area; nitrogen
   into the pools; a soil-limited field's grain rises and a season-limited one's does not; a well
   beside the field takes load; no field below the last connection, no farm; save and load exact;
   `contagion`, `soil`, `wells`, `sickness`.
4. **Wear, deposit and mending (M1–M5).** Deposit and effective capacity; wear by cell monthly;
   collapse and the break; days unserved by cause; the polity's weighing and the common work of
   clearing and relining. Areas: `population/sewers.rs`, `population/wells.rs` (`wetness_at`,
   shared), `condition.rs`, `polity.rs` (`Sewer`, `SewerCause`, `SewerConnection`),
   `population/polity.rs` (the review), `civ-content` (`deposit_share`, `block_kg_m`, `clear_h_t`),
   `civ-sim/src/persist`. **Tests:** in `sewers`: deposit lowers capacity and backs up the last
   connected first; a cell at loss 1 breaks the sewer there and only the homes above it heap;
   clearing does not mend a collapse, relining does not clear; mending asked under the law and not
   after it ended; sludge from clearing conserved; outage days recorded; save and load exact;
   `upkeep`, `wells`, `polity`, `roundtrip`.

## 6. Guards

- No cleanliness score, recycling rate, salvage price, sewer condition stat or yearly decay
  percentage anywhere (12-02 §5.1; 12-10 §5.10; 12-08 §2.1).
- No era, date or quota: salvage, farms and mending happen where prices, soils and laws make them
  worth it, and a world where none of that holds lives as before.
- Slices 1 and 4 change saved sections; each records whether lived figures moved and says so.

**Open questions.** Should a tip's open access be a law's term (who may pick, at what charge)?
Should a polity weigh inspection (12-08 §5.6) now, or only after a demo shows sewers failing unseen?
