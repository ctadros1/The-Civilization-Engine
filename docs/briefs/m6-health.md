# M6 design brief: water, sickness and health

**Scope.** Plan §7's M6 ("Towns & their troubles") lists "**Water, sewage & disease**", "Incident panels, coverage maps" and, among its god tools, "plague" and "drought". Its demo includes "a cholera outbreak traced to a well", and it is to prove "the incident-plus-aggregate service UX". Plan §5.4 sketches the service: "a hydrology-driven contamination field", "water sources on a ladder from well to cistern to aqueduct to piped water", "agent-level SEIR per disease template (waterborne, airborne, contact)" and "mortality from nutrition and healer availability", with people deciding "whether to build and fund wells ...; hygiene norms; quarantine policy; where to live". This brief covers, for villages and small towns of a few hundred to a few thousand people (plan §4.3 sets 3–8k people in 2–3 settlements for M5–M7):

- where people get water (rivers, lakes, springs, wells), and who digs and keeps a well;
- what they do with waste, and how it reaches what they drink;
- how infectious disease spreads, and how sickness, recovery and death go in a person;
- how people in the world trace an outbreak to its source from what they saw and heard;
- the remedies people try, and how a polity responds through its law pipeline;
- what a drought does to water, and the plague and drought god tools.

Sibling briefs own fire, floods, levees and farming earthworks (`m6-fire.md`), war (`m6-war.md`), and the incident panels, coverage maps and god-tool previews (`m6-observer.md`). This brief needs from them floodwater that can enter a well, and one river flow that changes with the weather, which `m6-fire.md` §1.8 proposes (§1.9 below).

**What exists to build on** (checked in the code):

- **Water is a walk to the nearest river or lake.** `nearest_water` (`population.rs`) picks, from the home's travel field, the reachable cell closest in walking time that lies next to a river or lake (not the sea). `Behavior::FetchWater` (`decide.rs`, `WaterOption`) walks there, fills for 5 minutes, by daylight, at most 45 minutes away (`content/core/activity/fetch_water.toml`), and deposits at home. `Household.water_l` drains at a fixed 20 L a person a day (`water_at_time`, `person.rs`; `HouseholdParams`: 15 L a trip, 1.5 days kept). Nobody drinks as such; a dry household only feels `Reason::WaterShortage`.
- **No groundwater, springs or wells exist.** `WorldMap` (`civ-world`) holds a water class per cell (land, river, lake, ocean), lakes from a water balance and river reaches with a fixed mean annual discharge, `Q = A·runoff` (`rivers.rs`). `terrain::height_above_drainage` gives each cell's height above its river; `found.rs` scores sites by it (flood risk) and by distance to water. ADR-0012's "Revisit when" names "M6's water: rivers by season, floods, wells, ditches and terraces".
- **Weather** (`civ-land/src/weather.rs`): one daily series per world, with a slow monthly AR(1) anomaly (`Weather::slow`) that carries "droughts and wet spells across seasons", and a reference soil bucket where, in `root_zone_day`, "what the zone cannot hold drains away", uncounted. `Climatology` is computed when a world is made or loaded and never saved.
- **Waste is a midden.** Each household keeps a `Midden` (`person.rs`): 0.5 kg a person a day of ash, food waste, sweepings and dung (the people profile's `[midden]`), carried to fields by `manure_field`. Nothing records where anyone defecates.
- **Death.** `death_today_scaled` (`demography.rs`) splits a day's chance between a Siler baseline (the Hadza fit in `[mortality]`: ₁q₀ 216, e₀ 34) and hunger, as competing risks. `Cause` (`history.rs`, append only) has `Unspecified` ("illness or accident"), `Starvation`, `Childbirth`, `Collapse`, `Violence` and `Fell`. The README: "Disease, injury and cold are not modelled apart from the life table."
- **Kept from work.** A blow keeps someone from work for days (`Harm`, `hurt_until`, `crime.rs`). `Facts::hurt` (`decide.rs`) leaves them only sleep, eating, fetching water, rest, company, asking and gatherings.
- **Truth apart from belief.** Incidents are the kernel's truth; a `Belief` records its `Source` (saw, told, noticed, did) and its `origin` (ADR-0015 §1). Claims are shared and immutable, and `Heard` records travel through the household at midnight and companions at the hearth (`word.rs`, `population/word.rs`; ADR-0016 §3). `uses.rs` logs the places a household works and the outsiders it saw there, fading by a half-life; `views.rs` holds evidence for and against, fading to a prior.
- **The law pipeline** (`polity.rs`). Ten `PolicyKind`s, the last `BuildCrossing`: a public work asking each household an equal share of labour, where "nothing follows a share not given" (`content/core/policy/works_crossing.toml`). `Curfew` is a prohibition "whose keeping is each person's choice". Ten `IssueKind`s, the last `Fords` ("what its members walked"). "No issue carries a weight toward a policy" (`polity.rs`, after ADR-0013 §5). A household already weighs a log bridge by the walking its recorded wades would save over the log's life against its work (`fords.rs` keeps the wades; slice AW).
- **God tools.** `InfluenceKind` (`influence.rs`): whisper, ideology, agitator, bless, curse, wave. Bless and curse move a person's own draws "for illness or accident and for finding things out", never hunger's part of a death (`influence.rs`; ADR-0016 §5).
- **Ground and parts.** Earthworks are records; pits are dug at 8 h/m³ (ADR-0010 §2). Posts rot at the foot by the site's wetness (ADR-0009 §4). The dashboard's "Epidemics" row is grey: "no disease until M6" (`civ-host/src/dashboard.rs`).

## Where the reports agree

Eight points recur across 12-01, 12-02, 05-03, 05-04, 05-05 and 03-02. I treat them as invariants.

1. **Routes, not bonuses.** Water is a chain from source to household, "not ... a building-level bonus or an era progression" (12-01, opening); sanitation is transfers and exposures, "not ... a citywide cleanliness score or a circular 'pollution radius'" (12-02, executive conclusion); care is "not ... a universal mortality bonus attached to healer numbers" (05-05, opening); water and sewers are not additive bonuses (05-04 §1.2).
2. **No infection without a source.** "Do not spontaneously generate a particular infection merely because a midden becomes large" (12-02 §1.1); waste "should not spontaneously create a pathogen absent from the world" (05-04 §5.2). In a closed world an acute disease dies out and returns only by an explicit introduction (05-03, executive recommendation; §4.3).
3. **Exposure follows what people drink, touch and share.** "Distance to an infected person is not the relevant distance; connection to their contaminated water source is" (05-04 §1.1). Households on one source share risk though apart (12-02 §4). Routes accumulate before one draw (05-03 §7.2).
4. **People act on what they observe, never on truth.** Clear water may be preferred without being safe, and customary protection needs no germ theory (12-01 §4). "Beliefs about the cause of disease should be separate from the actual transmission model" (12-02 §1.7). "A citizen can know that several neighbors have diarrhea without knowing which well or pathogen caused it" (05-03 §7.1). Theory is neither required nor sufficient (05-04 §5.4; 05-05 §1.2).
5. **Infection, symptoms, infectiousness, work and death are separate**, with distributed durations and death hazards by stage (05-03 §1.1, §7.3, §7.6; 12-02 §5.4). "Do not reroll a whole-episode case-fatality probability every day" (05-05 §5.4).
6. **Deaths are not counted twice.** An all-cause life table already holds infectious deaths (05-01 §1.3; 05-03 §3.4; 05-04 §5.3; 05-05 §5.2).
7. **Water and waste are conserved and shared.** A well "does not create groundwater" and springs draw on the same account (12-01 §1.2); "do not give each well an independent renewable supply" (03-02, executive recommendation). Waste moved is waste somewhere else (12-02 §5.2; 05-04 §4).
8. **A law acts only through work and behaviour.** "Passing a law should not directly clean the environment" (12-02 §5.5). Recognizing, authorizing, financing and operating are separate achievements (05-04 §3.3). Restrictions also cost trade, trust and evasion (05-05 §1.7).

**What reference simulations teach to avoid:**

- Borrow Workers & Resources' legibility, not its outcomes as estimates (12-01 §6.5; 05-04 §5.7); Timberborn's readable drought storage, not its "fictional hazard cycles" (12-01 §6.5) nor its groundwater (03-02 §5.4); Farthest Frontier's visible compost jobs, not its relationships as calibration (12-02 §5.7). A commercial game's "displayed disease behavior often omits or conceals its epidemiological assumptions" (05-03 §8.1).
- Shortcuts the reports name: "one aqueduct = water for 5,000 people" (12-01 §1.4); "+20% water" upgrades (03-02 §5.3); "waterworks −20% mortality; sewer −20% mortality" (05-04 §1.2); a rat-population meter (12-02 §5.4); "infected for ten days, then immune for one year" (05-03 §1.3); "hunger doubles every disease risk" (05-03 §3.4); "healers reduce all mortality by 5%" (05-05 §2.2); "drought → famine → revolt" (03-03 §5.2); an epidemic spawned by every disaster (03-07 §1.2).

**Where they differ, or differ from the plan:**

- **SEIR templates.** Plan §5.4's "SEIR per disease template" is what 05-03 warns against: "several disease mechanisms ... not one generic disease system with different mortality settings" (executive recommendation; §1.3). The reports differ on how many to build first: five persistence mechanisms (05-03, bottom line), four computational categories (05-04 §5.1), or at least two environmental behaviours, fast fecal–oral and slow soil-borne (12-02 §5.4, §5.6).
- **A ladder of sources.** Plan §5.4's "well to cistern to aqueduct to piped water" against 12-01 §6.4: "Rather than a sequence ..., use interacting capabilities".
- **Healer availability.** Plan §1 keeps healthcare, education, power and waste each "a single abstract stat (healer availability, literacy/skill rate)", and §5.4 has "mortality from nutrition and healer availability". 05-05 §5.1 keeps such a stat only as access to care whose effect is condition-specific; 05-03 §3.4 rejects a universal nutrition multiplier.
- **A contamination field** is fine as transport but not as a radius: "travel time, not a safe circle" (03-02 §1.6).
- **Plague.** Bubonic plague "is not ordinarily a human-to-human respiratory epidemic" and needs a rodent–flea system (05-03 §1.2); medieval transmission is contested (§5.3). The plan's plague tool cannot mean that disease in M6.
- **Groundwater resolution.** "An aggregated groundwater representation" (12-01 §6.3); 64–128 m head cells with 5–20 m contamination paths (03-02 §5.1); a 10–25 m grid "to benchmark" (12-02 §5.3).
- **Setbacks.** 30 m appears in guidance, with indicator migration "from under 1.5 m to about 25 m" (12-02 §2.2); 15–30 m guidance, attenuation "within 5–15 m" and detections "around 20–25 m" (03-02 §2.4). Both call distance a heuristic, not physics.
- **Digging rates.** 12-01 §2.3's modern norms (5.0 m³ a worker-day in soft soil down to 0.8 in rock) are faster than the kernel's 8 h/m³ (ADR-0010 §2), and 12-01 warns against applying them "to a deep well".
- **The dashboard row.** Plan §4.7's "waterborne epidemics cluster on contaminated sources" is graded on five unattended worlds, but a closed world has no disease to cluster (05-03 §4.3).

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| Water sources | Wells: site, depth, diameter, lining parts, cover and curb, water column, owner, state | Digging and upkeep as chosen work; the aquifer |
| Shallow aquifer | Head per coarse cell | Drainage from the weather's soil bucket; wells; rivers |
| A household's water | Litres and a load per disease in them; the sources its people know | Fetching trips |
| Waste and contamination | A load per disease in each midden, well and river reach | Infected people's excretion; rain; floods |
| Infection | Episodes: person, disease, stage and its ends, immunity until, and the acquisition record (day, route, source) | Exposure by water, household, care and company |
| Sickness seen and told | Appended claim kinds (sickness in a household; someone holds that a source sickens) and hearing records; water sources as `uses.rs` places | Seeing someone abed or a death; the hearth; others seen at a source |
| Care | Hours of tending, on the episode | A household's choices |
| Laws | Appended issue and policy kinds, with ADR-0013's whole history | Proposals and gatherings |
| Interventions | `Influence` records with the disease, target and period | The observer |

**Derived, not saved:** springs, each reach's flow on the day, the aquifer's material priors (from map and seed), a person's suspicion, outbreak groupings and every panel aggregate. **Size:** an outbreak of 300 cases at about 40 B an episode is 12 KB; an 8 km map at 128 m is about 3,900 head cells, 16 KB as `f32` (my arithmetic; the 40 B is a guess).

### 1.1 Where people get water

**Reports.**

- Water has "quantity, reliability, accessibility, and quality", and "a river-adjacency bonus conflates" them (10-01 §1.2).
- Access sets use. WHO's basic access is "usually ≤20 L/person/day" at 100–1,000 m or 5–30 minutes; beyond 1 km or 30 minutes, often under 5 L (guidance, medium). The report's priors are 10–30 L hand-carried and 30–100 L on the plot (proposed, low confidence) (12-01 §2.1). Piped Tanzanian households used 4.3 times what unpiped ones did (12-01 §5).
- A five-person household using 100 L a day spends about 100 minutes a day on it at 500 m and 40 at 100 m (calculated, 12-01 §1.5). Perceived risk is a term of its own, apart from actual contamination (§1.5). Households plan and individuals carry; an interruption brings rationing or "an inferior fallback source" (§6.1).
- Access rights are apart from the connection (12-01 §1.6), and old wells are kept for redundancy (12-01 §5).

**My proposal.**

- **A choice among known sources.** `nearest_water` becomes a choice among the sources a household's people know: river and lake points, springs, its own well and wells it may use. Each is scored by the walk, the wait and lift at the source, and the household's own suspicion of it (§1.6), as 12-01 §1.5's perceived-risk term. Sources become an appended `Place` kind in `uses.rs`, so households log where they draw and whom they saw there.
- **Use flexes with access.** The fixed 20 L becomes a target that falls as the trip's cost rises, within 12-01 §2.1's 10–30 L prior for carried water (a tuning curve). Drinking and cooking come first: below WHO's 7.5 L planning allowance (12-01 §2.1; not an intake figure), fetching outranks anything that can wait, from the nearest source even if suspected. A share of the litres is drunk; it carries the dose (§1.4).
- **Rights, v0.** A household's well serves its own people and its kin's households; a polity's well serves its members; others ask leave (open question 3).

### 1.2 Groundwater, springs and wells

**Reports.**

- Store groundwater as a head and let depth emerge; "Do not assign a comparable universal groundwater-depth table by terrain type" (03-02 §1.3, §2.2). Priors: sand aquifer K 0.1–100 m/day, Sy 0.15–0.30 (medium); clay matrix 10⁻⁷–10⁻⁴ m/day (low); fractured rock 0.001–10 m/day (low) (03-02 §2.2).
- A well is shaft storage plus inflow, and "depth alone does not determine yield"; a 1.2 m well with 2 m of water holds about 2.26 m³ (03-02 §1.4). Wellhead integrity (lining, cover, apron, drainage) is a route of its own (§1.4).
- Hand-dug wells run 5 to more than 20 m (guidance, medium; 12-01 §2.2). A Chilean sample of 49 ran 1.48–10.1 m, mean 4.78 m, and drew 0.02–3 m³ a day, mean 0.46, "not maximum hydraulic yield" (high for that sample). Prehistoric Yangtze wells were mostly under 3 m; one late-Neolithic well was about 11 m (03-02 §2.3).
- Early Neolithic timber-lined wells date to 5469–5098 BCE (12-01 §3.1). "Available timber, excavation stability and local water depth should govern feasibility", and water knowledge can be social memory (03-02 §3).
- Maintenance restores specific properties, not a durability bar (12-01 §1.7).

**My proposal.**

- **A shallow aquifer on coarse cells** (03-02 §5.1's 64–128 m). Each cell keeps a head. Its K and Sy are drawn by seed within 03-02 §2.2's priors by landform: valley-floor alluvium as sand, slopes of bare rock as fractured rock, uplands as silt. Recharge is what the reference soil cannot hold, which `root_zone_day` now discards. Cells exchange with neighbours and with river cells, and wells draw on them. When a world is made, or a pre-M6 save loads, the heads start from `height_above_drainage` and spin up over some years of the landscape's climatology.
- **Springs are derived** (my proposal; the reports say little beyond one shared account, 12-01 §1.2): a land cell off the river network where the head stands above the ground, flowing what the aquifer sheds there. They dry when heads fall.
- **A household digs a well as it weighs a log bridge.** Once a year it compares the walking and waiting its fetching would save over a well's life with the hours to dig and line one, at the depth it expects from what it knows: wells it saw dug and where they met water, pits that met water, its height above the river it draws from. Founders know a well-digging technique with timber lining (ADR-0008), at a share that is a design prior. At 8 h/m³, a 1.5 m shaft 5 m deep is about 8.8 m³ and 71 hours before lining (my arithmetic). A shaft meets water where the true head is, or not: a dry one is deepened or given up, and its depth becomes knowledge.
- **A well is an ADR-0010 pit with ADR-0009 lining parts** that rot by wetness. Its yield is its water column, refilled by inflow from its cell's head (03-02 §1.4's formula as a check). A draw takes from the column; an empty one means waiting or another source. 300 people at 20 L need 6 m³ a day, about 13 times the Chilean wells' mean use (my arithmetic).
- **Keeping a well** is chosen work: mending its lining, clearing silt, making a cover and a raised curb. Each part closes or opens a route (§1.3). A polity's well comes by law (§1.8).

### 1.3 Waste, and how it reaches water

**Reports.**

- The chain runs from production through deposition, containment and transport to exposure, and "removal versus neutralization" is the key distinction (12-02, executive conclusion).
- Deposition is a choice by travel, privacy and habit; children's waste goes through caregivers; where excretion occurs must be kept (12-02 §1.2, §5.1). People produce a median 128 g of wet feces, 1.42 L of urine and 1.20 events a day (high for the compiled medians, medium for historical transfer, 12-02 §2.1). At Çatalhöyük, middens mixed domestic waste, ash and fecal material (12-02 §3).
- Pathways: hands and containers, groundwater, runoff and floods, rivers, food and soil, animals (12-02 §1.3). For wells: groundwater direction, vertical separation, permeability and "surface ingress at the wellhead" (§1.3). In one Bangladesh study, aquifer contamination was less frequent than contamination of collected and stored water (03-02 §1.6).
- Travel time, not a circle: at a gradient of 0.01 and porosity 0.25, 30 m takes 75 days at K 10 m/day and 7.5 days at 100 (calculated, 03-02 §1.6). Use sparse links with travel time and capture, and "do not send the same complete load independently to every nearby well" (12-02 §5.3). There is no universal pathogen half-life (12-02 §2.2; 03-02 §2.4).

**My proposal.**

- **Where it goes.** A person's excretion goes to their household's midden when at or near home, and to open ground where they work otherwise. A load per disease follows their shedding, never the heap's size. Pits and latrines wait (§2).
- **Four routes in v0.** (a) *Runoff:* on a day of heavy rain, a share of each midden's load reaches wellheads and banks just downslope, far less into a covered, curbed well. (b) *Groundwater:* sparse links from middens to wells and river cells downgradient on the head field, each with a travel time and decay per disease. (c) *Rivers:* loads move reach by reach with the day's flow, diluted and decaying, as 05-04 §5.2's mass balance. (d) *Storage:* each fetched vessel's load mixes into the household's water. Floodwater into wells comes from the floods brief.
- **Concentration** is load over litres (12-02 §5.3; 05-04 §5.2). Decay rates and the runoff share are tuning values, logged as such.
- **Every move of a load is recorded,** so the panel can say what 12-02 §5.2 asks: "This well received contamination from Pit 184 after groundwater rose."

### 1.4 Diseases and how they spread

**Reports.**

- Diseases need their own branches: acute immunizing, carrier, persistent and partially immune (05-03 §1.3). Routes: water and food; fecal–oral by caregiving, toileting and food (Shigella is "not exclusively 'dirty river disease'"); shared air indoors, not "every person within an outdoor radius" (05-03 §1.2).
- Natural history (05-03 §2.1). Cholera: incubation 12 hours–5 days; shedding 1–10 days, including in infections "with few or no symptoms"; protection "at least 3–10 years" (high for timing, medium for immunity). Shigella: symptoms in 1–2 days, about 7 days ill, shedding up to two weeks after. Smallpox: incubation 10–14 days (7–19). Smallpox's R₀ is 3.5–6 in initially susceptible populations (§2.2); about 30 % of variola major cases died (§3.2). No portable R₀ exists for cholera, typhoid or dysentery (low, §2.2).
- Integrate exposure over routes into one draw, 1 − exp(−ΣH) (05-03 §7.2). The exponential dose-response is "a possible modeling choice", not a law (12-02 §5.4).
- The measles critical community size is 250,000–500,000, yet "a village of 300 susceptible people can therefore experience a devastating outbreak" (05-03 §4.1). Imports are explicit; a closed world lets a pathogen go extinct (§4.3).

**My proposal.**

- **A content kind `disease`:** its routes; the three clocks as lognormal or gamma distributions (05-03 §7.3); the share with symptoms; severity by age; shedding by stage; decay outside the body; immunity; and death hazards by stage. Authored first: **cholera** (water; the demo) and **bacillary dysentery** (water and household contact), and, if time allows, **smallpox** (home and hearth), to test extinction. Values come from 05-03 §2.1 and §3.2 where given. Dose scales are tuning values, calibrated on attack patterns.
- **Exposure is daily and per person:** water drunk from the household's store; contact with shedding members of the household, more for a carer; for air-borne templates, the sick at home and the hearth company as sampled today. One draw a day.
- **Introductions come only from the observer** (§1.10). A disease with nobody infected and no load left is gone.

### 1.5 Sickness, recovery and death in a person

**Reports.**

- Keep infection, symptoms, infectiousness and work capacity apart (05-03 §1.1). Sample durations in a biologically possible order (§7.3). Prefer stage-specific death hazards, so care given after onset still changes survival (§7.6).
- Cholera's untreated clinical fatality "can reach approximately 50%", and adequate rehydration brings it "below 1%" (high for the contrast, medium to low for transfer, 05-03 §3.2). Shigella's 15 % among hospitalized *S. dysenteriae* type 1 patients is an upper benchmark only (§3.2).
- Nutrition acts on susceptibility, progression, recovery and survival separately, and illness worsens nutrition (05-03 §3.4). For exceptional events, add "excess mortality relative to baseline, not another copy of ordinary disease mortality" (05-01 §1.3).

**My proposal.**

- **An episode per infection:** exposed, then infectious and ill by the template's clocks, then recovered (immune until a day) or dead.
- **The ill are abed.** They are kept from work through `Facts::hurt`'s gate, eat less (a tuning value) and do not fetch water; their household fetches for them.
- **Death in the severe stage.** A severe share by age, and by nutrition only where a template authors it. While severe, a daily death hazard, lowered on days care is given (§1.7). Append `Cause::Disease`, the episode naming its template ("died of the flux, 34"). Steps are daily, as plan §4.4 says; 05-03 §7.5 would keep subdaily events for fast deterioration, which I accept as a first approximation.
- **No double count in v0.** Introduced outbreaks are excess mortality over the Siler baseline (05-01 §1.3). If a disease becomes endemic in measured worlds, the residual is refitted (§3).
- **Bless and curse** move the target's own severe-stage death draws, as they move the baseline today (ADR-0016 §5 names illness), never infection.

### 1.6 What people see, and tracing a source

**Reports.**

- Keep an acquisition record per infection; for a well fed by several people, the defensible source is "well 17, contamination mixture during days 120–123" (05-03 §7.4). What people believe is a record apart from the true state (§7.1).
- Institutions act on "unusual deaths, repeated household complaints, ... burial records, source-specific illness patterns, or visible overflow" (05-04 §5.4), and misleading statistics produce mistaken beliefs (§3.3).
- In Snow's comparison, houses supplied by two companies had 315 against 37 cholera deaths per 10,000 houses in seven weeks, about 8.5-fold; "the denominator was houses, not people" (12-02 §4; 05-04 §2.2, high for the contrast). Supply membership is "a causal exposure variable" (05-04 §3.2).
- "Abandoning a source can result from experience rather than a scripted scientific era" (03-02 §5.3).

**My proposal.**

- **Truth.** Each infection keeps its acquisition record: day, route (water, household, care, company, observer) and source (a well, a river point, a household, an influence). No choice reads it (ADR-0015 §1).
- **What is seen.** Illness is seen by the household and by anyone who enters the home (visiting, tending, asking); a death by the household and its kin. Seeing makes an appended `ClaimKind::Sickness` (a household had sickness from a day), told as any claim with its origin.
- **Who draws where.** Each household logs its own draws and the households it saw drawing at the same source that day (`uses.rs`). A person also knows the usual source of households they have ties with (a design prior).
- **Suspicion is a tally of one's own records.** At the household's weekly review, or on hearing of sickness, each adult counts, over a recent window, the households they know that draw at a source and had sickness, against those that draw elsewhere. They hold a suspicion when the first share exceeds the second by a ratio over a minimum count (tuning values; Snow's contrast is an orientation, not a target). The same tally runs for contact with sick households. A suspicion is told as an appended claim kind, "someone holds that this source sickens", with its counts; one origin's retellings count once (ADR-0016 §3).
- **Plain words.** The inspector and chronicle say, for example: "Ada stopped drawing at Wren's well: of 9 households they know that draw there, 6 had the flux this month, against 1 of 11 that draw elsewhere." Beside it, the incident panel's truth: "31 of 44 cases since 3 June drank from Wren's well, fed by the midden of Bo's household after rain on 1 June."

### 1.7 Remedies people try

**Reports.**

- Household care is the foundation: an ill person creates care tasks that take someone's labour, and there is no separate "nursing nutrition bonus" (05-05 §1.1). Early farmers relied on "household care, with specialization varying locally" (05-05 §3.1).
- An unvalidated remedy defaults to "no established curative effect" (05-05 §1.3), RR 1.00. Organized supportive care starts at RR 0.95, tested over 0.85–1.00 (a low-confidence prior), for care-sensitive episodes and care actually given (05-05 §2.2). Access and effectiveness are separate (§5.1).
- Rehydration chiefly changes cholera survival (05-03 §1.4). In 1971, 3,703 refugee patients had 3.6 % fatality, a demonstration unit's 1,190 had 1 %, and severe cases also had intravenous fluids (05-05 §2.1). "A counterfactual society could discover an effective, simple fluid-replacement protocol earlier" (05-05 §3.2).
- Isolation, quarantine and cordons act at different points; "restricting human movement does not automatically remove contaminated water" (05-05 §1.7); "even perfect isolation performs poorly when recognition occurs late" (§5.5). Vitruvius reports Salapia moved "to a healthier location" (10-01 §1.2).

**My proposal.**

- **Tending the sick** is an appended behaviour. A member spends hours with the ill, bringing water and food. It costs work and exposes the carer. A severe day with care lowers the death hazard by the template's supportive effect (05-05 §2.2's 0.95 prior for enteric templates). A household all abed has no carer unless kin or those who regard it come.
- **Fluid replacement** is a far technique (ADR-0008): found rarely or introduced by the observer. Where a knower tends, cholera's severe-stage hazard falls toward 05-03 §3.2's treated figure. Other remedies are content with RR 1.00 unless evidence is authored.
- **Avoiding a source.** A household that suspects a source draws elsewhere if it knows another, paying the walk, and may cover its own well.
- **Keeping away.** Someone who suspects contact keeps from a sick household's home and the hearth.
- **Moving away.** A death at home, or a suspicion, is an event at which a household reconsiders where to live (ADR-0018 §5), weighing the sickness it believes here and at places it knows. Movers carry infection with them (05-04 §1.1).

### 1.8 The polity's response

**Reports.**

- Plausible rules include "household frontage-cleaning duties, designated disposal grounds, restrictions on dumping into watercourses", each requiring "jurisdiction, labor, enforcement, and compliance—not merely enactment"; a government may first act because of "smell, blocked streets, disputes, or visible illness" (12-02 §1.7).
- A regulation needs "a target behavior, responsible actor, detection probability, consequence, and enforcement cost" (12-02 §5.5).
- A water project is attractive when its benefits outweigh its costs "for the people able to authorize it" (12-01 §1.6). "Public quarantine should not require a physician profession" (05-05 §4); compliance shares of 0.50, 0.80 and 0.95 are stress tests, and emergent compliance is preferred (05-05 §2.2).

**My proposal.**

- **Issues (appended):** `sickness` (a household had sickness lately, as its members know); `source_suspected` (someone holds a suspicion; it opens moves to them, as `Overruled` does); `water_far` (a household's fetching walks are long, as its members walked them, like `Fords`).
- **Policies (appended):**
  - `close_source`: nobody may draw at a named source. Keeping it is each person's choice, weighing the gathering's word as a curfew is. A higher level may set the watch at it or have the polity fill it as an earthwork.
  - `dig_well`: a well built together at a named site, as `BuildCrossing`: an equal share of work asked, nothing following a share not given, owned by the polity and open to its people.
  - `keep_apart`: a household with someone sick keeps to its plot, and others keep from it, until some days after the last is well (quarantine kept by choice).
- **Stances** use each household's own forecast in today's units: hours walked or worked, against days of work its members would lose to the sickness it believes the source or contact brings (its own tally), a death counted as `w_death` days (a tuning value). `close_source` bears security +, autonomy −; that weighs what people hold dear.
- **Legible mistakes stay possible.** A quarantine passed against a waterborne outbreak costs work and does little (05-05 §1.7).

### 1.9 Drought and water

**Reports.**

- Meteorological, agricultural and hydrological drought differ; soil water responds fast, while groundwater, streamflow and storage have "longer memory" (03-03 §1.6). Drought depletes "soil water, streams, reservoirs, and groundwater on different timescales" and "is a persistent state, not an instantaneous attack" (03-07 §1.1).
- Judge storage against seasonal sequences, not annual totals (12-01 §1.3). "Do not silently replenish groundwater" (03-02 §5.2). Later deaths come through the existing nutrition and disease systems, and an epidemic is not spawned by rule (03-07 §1.2).
- A god tool's event uses the same systems as a natural one; "Store an explicit `origin = natural | intervention` flag" (03-07 §1.3).

**My proposal.**

- **Drought reaches water through the aquifer and rivers.** Recharge stops, heads fall, columns shorten, shallow wells go dry and springs stop. Each reach's flow on the day is its mean scaled by a smoothed runoff store against its long-run mean, the one store `m6-fire.md` §1.8 proposes for floods; low flow concentrates loads (§1.3). That brief lets fords open and close with it; I would keep fords on mean discharge in M6, since changing routing changed every world in M5 (open question 7).
- **People respond with what exists:** longer walks, queues at fewer sources, less water used, deepening a well, a polity's well, moving. Concentration rises as volume falls (05-04 §5.2), so an outbreak may follow a drought through crowding at fewer sources, never by rule.
- **The drought god tool** holds the weather's slow anomaly at a chosen dry value for 3–24 months. It is an appended `Influence` with its origin; the months lived under it are marked; `Climatology` already never reads lived months.

### 1.10 The plague god tool and the incident panel

**Reports.**

- God tools submit perceptible events or physical changes, one record per recipient, every use logged (ADR-0016 §5); an origin flag keeps interventions out of natural records (03-07 §1.3).
- Plan §5.4: "Incidents are sim entities with a lifecycle ... this cholera case traces to this well. The dashboards ... are computed from the incident log." Keep "the household's water source, the contaminated network, the responsible failure, and the institution's response" (05-04 §5.6).

**My proposal.**

- **Bring a sickness:** the observer picks a disease and either a person (infected from today) or a source (a load placed there). An appended `InfluenceKind`; the acquisition route says "observer". What follows is people's. A source target makes the source the observer's choice, so the demo should start from a person (`m6-observer.md`, open question 4).
- **The incident panel.** Each case is an incident: the episode, its acquisition record and its course. An outbreak is a grouping derived from the log (one disease, cases linked by source or contact within its clocks). Aggregates: the epidemic curve, attack rates by source and household, deaths by age, care given. Three columns stay apart, as ADR-0015 §7 does: what happened, what people believe, what the polity did.
- **The coverage map** shows each home's walk to its sources, wells' water columns, and the sources people suspect.

## 2. What to defer, and why

| Defer | To | Why |
|---|---|---|
| Rodent–flea plague, malaria, vectors | Later | They need animal and vector systems (05-03 §1.2); the plague tool offers authored templates meanwhile |
| Typhoid carriers, TB, reactivation | After M6 | The episode keeps room for those branches (05-03 §1.3); carrier fractions need separate validation (§6) |
| Soil helminths | Later | Eggs mature over 2–4 weeks (12-02 §2.2) and burden more than kill; 12-02 §5.6's second behaviour waits |
| Latrine pits, night-soil collection, drains | When towns need them | Nothing yet moves a household to dig one but a belief or a law; pits fill at 40–90 L a person a year (12-02 §2.1) |
| Cisterns, aqueducts, pipes, boiling, filtration | Stone kit and later | Capabilities, not a ladder (12-01 §6.4); boiling does not remove every hazard (12-01 §4) |
| Food batches and markets as routes | Later | Food may dominate exposure (12-02 §4), but goods carry no batch identity yet |
| Healers as a calling, hospitals, midwifery, immunization | M10 healthcare | Plan §7; 05-05 §1.5–1.6 |
| Cordons; complaints against upstream discharge | With force and relations | 05-05 §1.7; 12-02 §5.5 ("Downstream settlements can demand restrictions on upstream discharge") |
| Imports from off the map | M9 regions | The off-map account holds no stock (ADR-0018 §3); 05-03 §4.3 wants an external population |
| Hygiene norms | After an outbreak is lived | ADR-0016 §4's norm templates fit, once suspicions exist to move them |
| Nitrate, salts, arsenic | Later | 03-02 §1.6 |
| Lasting conditions (scars, disability) | Later | 05-03 §3.4 |

## 3. Risks and validation

**Plot.** Watch for issue→policy weights ("sickness makes a well closure"), suspicion thresholds tuned until the demo's well is named, disease that appears without an introduction, and outbreaks after floods by rule. The demo's trace must come from people's tallies. After a week without a lived trace, nudge a prior (say, the minimum count for a suspicion), never an event, and log a `NUDGE:` (plan §1, rule 3).

**Nothing to trace.** Villages are sited near rivers (`found.rs`), so households may never dig wells; a large river dilutes any load; and if everyone draws at one point, a tally finds no contrast. Measure the funnel before tuning: wells dug, households per source, cases by source, suspicions held, sources avoided, laws proposed and decided. A move that is never available is a bug, not a preference (M5's lesson).

**Leaking truth.** Test that no choice reads an acquisition record, a source's load or another person's tally; that a household that heard nothing suspects nothing; and that one teller's retellings count once.

**Double counting.** Introduced outbreaks add to the Siler baseline by design (§1.5). If a disease persists in measured worlds, refit the residual by running the disease systems in a reference world (05-01 §1.3). 05-05 §5.1's example shows why effects look small: care reaching 60 % of patients and cutting their risk by 20 % turns 50 deaths into 44, which cuts all deaths by only 3 % if those episodes cause a quarter of them (the report's illustration).

**Scale.** A year of 3,000 people in three settlements takes about 20 minutes at Max against a budget of ten (README). Disease adds work in proportion to households, sources and links each day, not pairs of people (05-03 §7.5; 05-04 §5.6); the aquifer is a few thousand cells. Add cases, sources and water use to Gate B (ADR-0011 §5) with wide tolerances.

**Tests from the reports.**

- **Source membership** (12-02 §4; 05-04 §3.2, §4): two intermingled groups of households, alike but for their well; the contaminated well's users fall ill many times as often. Snow's 8.5-fold contrast is an orientation.
- **No source, no case** (12-02 §1.1; 05-04 §5.2; 05-03 §6): a world without an introduction never has a case, whatever its middens; no negative counts and no double death.
- **Extinction** (05-03 §4.1, §4.3; 05-04 §4): an outbreak in a village of a few hundred ends and does not return without a new introduction. Measure a reproduction number by introducing infection into replicated reference villages (05-03 §2.2).
- **Care changes deaths, not cases** (05-03 §6): with fluid replacement known, cholera deaths fall toward 05-03 §3.2's "below 1%" while incidence holds.
- **Wellhead and storage** (03-02 §4): a cover and curb lower what users drink while the aquifer is unchanged; storage can be dirtier than the aquifer (03-02 §1.6).
- **Geology** (03-02 §4): equal loads at equal distances differ in sand and in fractured rock; 03-02 §1.6's 75 against 7.5 days.
- **Not additive** (12-02 §4; 05-04 §4): closing one route leaves others open; quarantine barely touches a waterborne outbreak (05-05 §1.7).
- **Access and use** (12-01 §1.5, §5): a household with a well on its plot uses more water than one walking 500 m; fetching times near 12-01's 100 and 40 minutes a day.
- **Groundwater accounting** (03-02 §4): 50 mm of recharge at Sy 0.20 raises the head 0.25 m; a broad dug well yields its column at once but refills slowly; wells on one cell compete.
- **Drought lags** (03-03 §1.6): after a dry spell, soil dries first, wells fail later and recover later.
- **Small places can be deadly** (05-04 §2.1, §4): English towns of 2,000–3,000 had 209–270 infant deaths per 1,000 against under 100 in some remote parishes; such levels should be attainable, never targeted.
- **Conservation and causality** (05-04 §4; 12-02 §5.2, §5.7): every load's moves balance; a rising water table and a breached wellhead each give understandable, different outcomes.
- **Unchanged without it:** a world with no wells and no disease lives as before, digest for digest, and save → load → save stays exact.

**Observer.** The incident panel (§1.10); an inspector that shows a person's episodes, what they saw and heard, and their suspicions with counts; the map's sources, wells and water columns; a chronicle in plain words: "Wren's household dug a well 4.2 m deep and met water at 3.1 m." "The gathering at Ashford closed Wren's well, 21 for and 6 against; most who opposed draw there and suspect nothing."

**Choices that are expensive to reverse.** Plan §1 allows about three ADRs a milestone, and war and fire will want theirs. I suggest one ADR for health and water: infection episodes and acquisition records as truth that no choice reads, with sickness claims and suspicions apart; wells and the aquifer as saved state, with loads conserved on links; `Cause::Disease` with the template on the episode; the content kinds `disease` and `well`; and appended claim, issue, policy and influence kinds. Wells amend ADR-0009 and ADR-0010, as crossings did. Rivers by season amend ADR-0012.

## 4. Open questions for the designer

1. **Plague.** Which authored diseases does the tool offer? Is "plague" a menu name, with *Yersinia* deferred (05-03 §1.2)?
2. **The dashboard row.** Should each of the five worlds receive one declared introduction at a fixed day, logged as the observer's, so the epidemics row can be graded? Or should the row stay grey until imports exist (M9)?
3. **Well rights.** Who may draw from a household's well in v0: its kin, those it regards, anyone who asks?
4. **Founders' knowledge.** Do founders know well-digging and lining, or must a world find or be given it?
5. **Valuing a death.** Should forecasts price a believed death as days of work (`w_death`), or leave fear to values alone?
6. **Drought tool.** Hold the weather's slow anomaly, as proposed, or let rain fall to a stated share for stated months, as `m6-observer.md` §1.5 proposes? The second is easier to preview; the first keeps wet-day persistence and temperature coherent.
7. **Rivers by season and walking.** Keep fords on mean discharge in M6, as proposed, or let low water open fords?
8. **The baseline.** Should explicit disease deaths start replacing part of the Siler baseline now, or only once a disease is measured as endemic?
9. **Suspicion's holder.** Per person, as proposed, or per household?
10. **Contagion tallies.** Is a tally of contact with the sick in v0, so people can be wrong in a second way, or only source tallies?
11. **Bless and curse.** Should they reach severe-stage death draws, as proposed, and nothing else in an episode?
