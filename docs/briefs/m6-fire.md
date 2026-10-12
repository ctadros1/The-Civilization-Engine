# M6 design brief: fire, hazards and masonry

**Scope.** Plan §7's M6 ("Towns & their troubles") includes "**Fire.**", a "Stone/brick/tile kit", "Levees and embankments as public-works earthworks, which people can propose after floods", "Earthworks for farming, moved from M3c: drainage and irrigation ditches, and farm terraces (ADR-0012 §7)", "Floods, storms and fire damage feed structural demand; a collapse can prompt building-code proposals" and "God tools: fire, plague, flood, drought, storm". Its demo includes "a great fire leads to a proposed masonry code", and M6 is to prove "the incident-plus-aggregate service UX". Plan §4.3 sets 3–8k people in 2–3 settlements for M5–M7, so this brief is about villages and small towns of a few hundred to a few thousand people, most of whom live in thatched huts today. It covers:

- how fires start (hearths, ovens, crafts, lightning) and spread between buildings (materials, spacing, wind, dryness), how people fight them, and what fire does to buildings, goods and people;
- how a community comes to want a building rule, and what codes historically required;
- masonry, brick and tile as building systems;
- floods and storms as hazards from the weather the kernel already generates;
- levees and embankments as works a polity decides, and farm ditches and terraces;
- the god tools for fire, flood and storm.

I assume sibling M6 briefs own war and the fortification kit; water, sewage and disease (wells, contamination, plague); and weather visuals. This brief needs one thing from the water work: a river whose flow changes with the weather (§1.8). If that brief owns it, §1.8 is an input to it.

**What exists to build on** (checked in the code):

- **Buildings have parts, condition and loads.** `civ_land::buildings::{Building, GroupCondition, GroupState, BuildingState}` keep each component group's quality, loss, state and dates (ADR-0009 §4). `civ_grammar::GroupKind` has posts, tie beams, loft, floor and raised-floor joists, roof frame, covering, infill and floor. ADR-0009 §3 names a mass-wall group that the enum does not have yet. `civ-agents/src/structure.rs` (`margin`, `Loads`, `Timber`) checks bending and buckling with `(1 − x)³` and `(1 − x)⁴`; `population/loads.rs` weighs every building each day, and a failure kills by the NUDGE shares (`Cause::Collapse`, `ChronicleKind::BuildingFailed`). `caution.rs` (`CautionParams`) keeps each settlement's failures per technique, fading by half over 8 years.
- **Two grammars, one wall kind.** The hut grammar is frozen at version 1: thatch at 30 kg/m², a 45–55° pitch, a hearth part (`core:building/hut`). The frame grammar (`civ-grammar/src/frame.rs`) reserves `WALL_KIND` and `FOOTING`, but version 1 knows only wattle and daub on posts set in the ground. Frame dwellings have a hearth; stores do not.
- **Spacing.** A plot is the box its roof covers (`build::plot_rect`), and plots keep only `PLOT_GAP_CM` = 100 cm from each other, 4 m from the settlement's hearth (`HEARTH_GAP_M`). Eaves can stand 1 m apart.
- **Fire in daily life, with no fire as a hazard.** Households burn firewood by month (`population::fuel_per_day`; `early_farmers.toml`: 0.7–1.5 kg a person a day). They bake at the hearth or in a clay oven, a tool that stays where it is built (`core:good/oven`, `fixed = true`), and fire pots in an open fire (12 kg of wood a session, README slice Q). There is no kiln, lime, brick or tile good, technique or recipe.
- **Water at hand.** People fetch water from the nearest water cell, 15 L a trip (`carry_water_l`), and keep 1.5 days of 20 L a person (`water_target_days`), about 150 L at home for five (my arithmetic). Pots exist as goods.
- **Weather without wind.** `civ_land::weather::{Weather, WeatherDay, WeatherParams}` give daily rain, temperature, snow by height band and a reference soil bucket, keyed `[seed, weather, landscape, day]`. `Weather::storm` gives one gust load (kPa on a roof's plan) and its day for each month, the same for every roof. There is no daily wind, wind direction, humidity or fuel dryness. `ChronicleKind::Weather` notes months that stood out.
- **Rivers that never rise.** A reach keeps only its mean annual discharge (`civ_world` `discharge_m3s`); rivers above `ford_max_discharge_m3s` cannot be waded. Floodplains are carved terrain (`civ-world/src/floodplain.rs`), and `civ_world::terrain::height_above_drainage` exists (deposits use it). Field water drains away beyond the bucket's capacity: nothing waterlogs (`weather.rs`), and fields are sited by habitat, not slope (`farm.rs`).
- **Earthworks are records.** `civ_land::earth::EarthKind::{Platform, Pit, Spoil}` (append only, `EARTH_VERSION` 1) with delta tiles; digging at 8 h/m³, stone at 12. ADR-0010 §3 refuses earthworks on or beside water and forbids "drainage changed silently by an earthwork".
- **Public works by law.** `polity.rs` has ten `PolicyKind`s, among them `Curfew` (kept by choice, no sanction in v0) and `BuildCrossing`, and ten `IssueKind`s, among them `Fords`. `content/core/policy/works_crossing.toml` asks each household an equal share of the work, given by choice, with nothing following a share not given; `population/crossings.rs` (`PublicSite`, `begin_public_crossing`, `crossing_gives_way`) caps a day's crew and, when a crossing fails, drops whoever is on it (`Cause::Fell`) and makes every walker across it stop and decide again. ADR-0009 §9 leaves scour to M6.
- **Night eyes and god tools.** The watch walks rounds at night and sees only where it is (`population/watch.rs`). God tools arrive as host requests (`PlaceDeposit`, `SpawnFamily` in `civ-host/src/protocol.rs`) and influences (`InfluenceKind`), each chronicled. Saves are at schema 71, content API 68, wire 1.60 (README).

## Where the reports agree

Eight points recur across 12-03, 12-07, 11-09, 11-06, 11-08, 03-07, 12-08 and 11-12. I treat them as invariants.

1. **No disaster calendar.** There is no defensible ignition rate per early-farming household (12-03 §2.1, "Missing parameter"), no universal hazard table (03-07, executive recommendation) and no ancient collapse baseline (11-06 §2.1). Fires come from activities (12-03 §1.1) and floods from weather and routing: "Do not trigger a 'century flood' timer" (03-02 §1.2; 03-07 §2.1). A great fire is an outcome, "not a date on the simulation's disaster calendar" (12-03 §6, bottom line).
2. **Stages and counts kept apart.** Ignition, escape beyond the household, building involvement, neighbours and conflagration (12-03 §1.2); event, intensity, exposure, damage and recovery (03-07, executive recommendation); vulnerability, trigger and propagation (11-06 §1.1); an incident and its report (12-07 §1.2).
3. **Damage falls on components.** "Avoid a single building 'fire health' value" (12-03 §5.2). Fire reduces member sections and lets floors and roof ties fail before walls (11-06 §1.2). Putting a fire out restores nothing burned (12-03 §1.3, §5.4). A second hazard acts on what the first left (03-07 §5.2).
4. **People do the work, bounded by the scarcest input.** Water applied is the least of source, delivery and pumping (12-03 §1.7); responders are taken from other work (12-07 §1.4); crews have bottlenecks (11-12 §1.2), and so does upkeep (12-08 §1.4). Ordinary inhabitants matter: Paris did not burn partly because of them (12-03 §3.2).
5. **No material ladder.** Materials are production chains and assemblies (11-08, executive conclusion). Noncombustible walls do not make a building fireproof (11-06 §1.2; 11-08 §2.5). Good performance is conditional (11-06 §4); rubble and adobe masonry are the most vulnerable class in earthquakes (03-07 §2.2 A).
6. **A disaster opens politics; it does not pass laws.** "Disasters create political opportunities, not automatic reforms" (11-09 §1.2). Rules redistribute costs, so their details are fought over (11-09 §1.5; Lima, 11-06 §3.6). Rules change new work and leave old stock (11-09 §1.7, §4). Do "not automatically award 'centralization' after a flood" (03-07 §5.4).
7. **Knowledge travels.** An institution acts only on information that reached it (12-07 §1.3). An inquiry's attributed cause differs from the actual one, and remedies come from known techniques (11-09 §1.4). Public distrust spreads wider than the technical lesson (11-06 §5.6).
8. **Works are kept, or they fail.** Authorizing, building, controlling and maintaining are separate roles (12-08, executive recommendation). A levee's danger is conditional on its hydraulic loading (12-08 §5.2; 11-12 §4.2). Who benefits and who pays differ (12-08 §1.2), and "a levee can protect one community while shifting water toward another" (11-12 §5.3).

**What games and reference models teach to avoid:**

- A storyteller that sends weather incidents: borrow "the separation between circumstances and personal consequences, but not the protagonist-centered difficulty controller" (02-07 §1, RimWorld). Hazards must not be "selected because the observer has gone too long without drama" (02-07 §4 E).
- "A commercial game's hidden fire-risk percentage" and competition shortcuts such as protected facility types (12-03 §5.6, RoboCup Rescue).
- Passive hazard reduction near a station: replace it with "actual inspections, training, maintenance, or behavior changes" (12-07 §6.4, Cities: Skylines II); a green coverage overlay can mislead (02-02 §3).
- Abstract timers: from Caesar III's prefects borrow visible responders and mission states, not their timings (12-07 §6.4).
- Laws that enforce themselves, as in Eco (11-09 §5.6). Dwarf Fortress's cave-ins are design inspiration, not a safety model (11-06 §5.8); Timberborn is a presentation reference, not a labour source (11-12 §5.6).

**Where they differ, or differ from the plan:**

- **The fire ladder.** Plan §5.4 gives "none → bucket brigade (a norm) → organized watch → hand-pump engines → piped water and hydrants", each rung setting response speed, suppression and coverage. 12-03 §3.4 says the capability graph "is **not** a mandatory technological sequence", and 12-07 §4 favours "combinable institutions".
- **Who makes codes, and when.** Plan §5.1 gives codes to a planning institution (M8) and says "masonry becomes mandatory after a fire, but only if someone proposes it and it passes"; M6 wants a code without one. 11-09 calls proposals only after fires or collapses "historically too narrow" (main finding; §1.3: complaints, founding, borrowing, disease).
- **Lightning.** Plan §5.4 names it as a source. No report gives a lightning ignition rate; 14-09 §4.1 only says lightning damage must follow kernel state.
- **Time step.** 12-03 §2.5 proposes 1–5 seconds for an active fire (L confidence, "verify convergence"); the kernel's base tick is a minute.
- **How long a disaster is remembered.** 11-09 §2.3 gives 2–20 years (low); 03-07 §5.4 a 10–30-year timescale (D). Caution's 8-year half-life sits in the first.
- **Digging rates.** 11-12 §2.4's prior is 1.0 m³ of bank a day (0.5–2.0); 12-08 §2.1's ILO allowance 1–1.5 m³ an unskilled workday; 11-04 §2.2 0.25–0.8 pd8/m³, about 1.25–4 m³ a day (my arithmetic). The kernel uses 8 h/m³.
- **Bounded earthworks.** Plan §5.1 says "ditches and levees change local drainage only". 11-12 §1.6 warns that "embanking a lowland without providing drainage can trap rainfall behind the levee", and §5.3 that a levee shifts water to others. The two fit only if routing keeps floodplain storage.
- **Storeys.** 11-05 §2.2 offers 1–2 storeys for earth, 1–3 for timber and 1–5 for masonry as low-confidence priors; ADR-0009 builds 1–2.

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| Ignition | Nothing until a fire starts; then a fire record: origin as the kernel knows it, start, cause | Hours of hearth, oven, pottery and kiln fires; dryness; the observer's tool |
| Spread | Per fire: each building's fire stage and fuel left, and an exposure store per threatened building | Gaps, coverings and walls, wind, dryness |
| Noticing and fighting | Ordinary activities and walks; water carried and roofs stripped, on the fire record | Sight and earshot; ties, property at risk, water at home and at the stream |
| Fire damage | Char as loss in `GroupCondition`; goods on an appended ledger channel; a new cause of death | Fire stages; the existing daily load check |
| Dryness and wind | Two numbers on `Weather`: fuel dryness, and the day's wind | Rain, temperature, the month's storm |
| Masonry kit | New goods, a kiln (fixed tool), techniques, a grammar version with wall and covering kinds | ADR-0008 knowledge, recipes, ADR-0009 checks |
| Building rules | Laws of an appended policy kind: a rule template, its parameters, the day it binds | Issues `burned` and `gave_way`; each household's forecast |
| River flow and floods | Per world a runoff store; per building and field the last flood's depth and days | Daily rain and melt; floodplain storage by height above drainage |
| Storms | The month's storm, now with a direction; roofs open to rain | Weather |
| Banks (levees) | Earthwork kind `bank`: a line of segments, crest, condition, breaches | A polity's law; borrow earth; flood loading |
| Farm ditches | Earthwork kind `ditch`: a line, cross-section, gradient, sediment | A household or a law; field water; floods |
| God tools | Each event's origin, natural or intervention; the chronicle | The observer |

**Derived, not saved:** the exposure graph between buildings, R_fire (12-03 §4), river stage, flood extent, return levels, and every household's forecast of a rule.

### 1.1 Ignition: people's fires escape

**Reports.**

- Ignition is a hazard per activity-hour, `1 − exp(−Σ λ·ΔH·m)`, with cooking, heating, lighting, baking, smithing, kiln work, hot ash and storage kept apart. "A working hearth is not already a building fire." An attended fire both exposes and places someone near to notice; crowding adds ignitions and witnesses; deliberate burning comes from agents' decisions (12-03 §1.1).
- No early-farming rate exists (12-03 §2.1). As a prior, explore 1–100 unintended household fires per 1,000 household-years on a log scale, with 10 as an order-of-magnitude reference from the English Housing Survey (12-03 §2.5, L; §2.1: about 1 % of households a year, M). No reliable ignition rate by roofing material exists (11-08 §2.5).
- Fuel moisture responds to recent weather, and seasonality must emerge: Edo's fires came with dry winter winds and heating, not summer (12-03 §1.5).
- The kernel may know hot ash lit a store while "residents may only know that the building was found burning" (12-03 §1.2).

**My proposal.**

- **Fires in use carry the hazard.** Each household's hearth burns while its firewood is drawn, and each oven, pottery or kiln firing for its hours. A content rate per fire-hour, scaled by dryness and by whether anyone awake is home, sets ignitions; at average dryness the hearth alone gives about 10 per 1,000 household-years (a design prior inside 12-03 §2.5's range). At that rate a village of 20 households sees one ignition in five years, and a town of 300 about three a year (my arithmetic).
- **Most ignitions die at once.** If someone awake is in the building or beside it, they put it out, and only the kernel's count records it (12-03 §1.2's first counter). An ignition with nobody awake there grows.
- **Draws are keyed** by building and hour, so the same day lived at any speed starts the same fires (ADR-0011 §3; 12-03 §5.5).
- **Lightning and arson** are not in v0: no rate for the first (open question 1); the second belongs with force and war.

### 1.2 Spread between buildings

**Reports.**

- Three routes: connected fuel, external heat exposure, and firebrands. The hazard should rest on "accumulated exposure, not distance alone", a small store that rises under heating and falls with cooling or wetting. Routes differ by assembly: some small experiments showed no firebrand contribution (12-03 §1.4).
- Model the roof covering, frame, walls, contents, connections and condition apart (12-03 §1.3). One interior zone with a separately vulnerable roof suffices for ordinary buildings, on a sparse exposure graph rebuilt only when buildings change (12-03 §5.1).
- A full-scale test: 20 dwellings 1–2 m apart in 4.2–6.9 m/s of wind; four were lit together and the whole settlement burned within about 5 minutes; a dwelling passed 1,000 °C within about a minute and downwind neighbours could ignite within another (12-03 §2.2, H for the experiment). Separation thresholds of about 2.14 m and 3.14 m, and radiation falling from about 36 kW/m² at 1 m to 5 kW/m² at 4 m, hold for their configurations (M).
- There is "no universally safe 'three-meter firebreak'" (12-03 §1.6). Below an R_fire of one, fires die out; above it, across a connected district, they outrun crews (12-03 §4).

**My proposal.**

- **A building's fire** passes from its covering alight to fully involved to burnt out, consuming its covering, timber and the combustible goods under it at authored rates. Steps are one-minute events scheduled only while a fire burns; a test at half a minute checks that results converge.
- **Exposure.** Every building within a reach of a burning one (10 m, a tuning value) keeps an exposure store: heat by the gap between footprints, falling as 12-03 §2.2's 1–4 m figures do, stretched downwind by the day's wind and cut by water thrown on it. A thatch roof ignites at a low threshold, a tile roof only at its eaves and openings, a mass wall not at all.
- **Firebrands** are folded into the downwind stretch for thatch in v0 and become a route of their own later.
- **The risk is real today.** Huts 1 m apart under dry thatch are the experiment's configuration, so R_fire above one in wind is likely. That is the plausible part; §3 calibrates it.

### 1.3 Noticing, alarm and fighting

**Reports.**

- The clock runs detection, alarm, assembly, travel and setup, with steps in parallel (12-03 §1.8; 12-07 §1.1). Priors: 0–3 minutes to notice when awake, 5–30+ asleep, 2–15 to assemble, 5–10 L a vessel, 1–10 minutes to set up (12-03 §2.5, all L).
- Delivery is `N·V/T`: twenty carriers of 8 L on a two-minute circuit give 80 L/min before losses (12-03 §1.7, illustrative). Crews of four did essential tasks 30 % faster than crews of two, so more people help by tasks, not linearly (12-03 §2.2; 12-07 §1.7).
- Notification is physical: someone shouts or runs, and a lookout changes detection, not crews (12-07 §1.3). Residents rescue, carry water, protect neighbours, demolish, salvage or flee, one task each at a time (12-03 §5.3, §5.4).
- Demolition needs tools, access, time, coordination and permission, and costs even when it works (12-03 §1.6); it was central in Edo (12-03 §3.2). In 1579 Istanbul ordered households to keep ladders and water barrels (12-07 §4).
- Early farming: household help, shared equipment and obligations, with field work by day and water access setting readiness (12-07 §4). "Spread can outrun organized response" (12-03 §4).

**My proposal.**

- **Who notices.** Those inside and awake notice at once; sleepers wake after a draw in 12-03 §2.5's 5–30 minutes; anyone passing within sight (a tuning distance) notices; the watch on its rounds sees what it passes.
- **The shout.** Whoever notices shouts. Those within earshot (a tuning distance) hear, their activity ends, and they decide again, as walkers do when a crossing gives way.
- **Choices, scored as any.** Each who knows chooses: wake and lead out their household, carry water, strip a threatened roof of its thatch, carry goods out, pull down a building, or keep away. Kin, ties, their own property's exposure and danger weigh as in other choices; nobody is ordered.
- **Water is real.** Carriers draw first on household stores (about 150 L for five) and then walk to the nearest water cell with pots of 5–10 L. Water applied cools exposure stores and, above an authored rate, puts out a burning covering.
- **Pulling down** one's own building is a choice. Pulling down another's needs its household's leave, or a law granting the power (open question 6).
- **No brigade office in v0.** A rule may oblige households to keep water and a ladder (§1.7).

### 1.4 What fire does to buildings, goods and people

**Reports.**

- Exposed timber chars at about 0.8 mm/min at first and 0.6 mm/min later; a 200 mm member exposed on four faces for an hour keeps a 128 mm core, about 41 % of its area and 26 % of its section modulus (11-08 §2.5, an illustration; 11-06 §2.2). Earth walls do not burn, but the timber they carry does; brick and stone crack and lose strength; a tile covering and the timber under it need separate fire states (11-08 §2.5).
- Model occupancy, warning, evacuation, refuge, smoke and collapse apart, as burned area is no proxy for deaths (12-03 §4). Keep destroyed stocks, lost flows and rebuilding apart (03-07 §2.4). Recovery envelopes (1–8 weeks to shelter, 1–12 months to repair) are design priors to check against, not to impose (03-07 §2.5, D).

**My proposal.**

- **Parts burn as parts.** A burning covering is lost and the roof opens. While a building is fully involved, its rafters, beams and posts char on exposed faces at 11-08 §2.5's rate, turned into loss of section in `GroupCondition`. The daily check then fails them by ADR-0009 §5's rules. Wattle burns and daub stays; a mass wall shows cracking after a full burn (an authored loss).
- **Goods.** What lies under a burning roof burns at authored shares by good (grain partly, firewood wholly, pots not), on an appended ledger channel. Goods carried out survive.
- **People.** Those asleep inside and never woken die of a new appended cause, *burned*. Those under a roof or posts that give way die by the existing collapse shares. A household left homeless is taken in, rebuilds, or asks relief, as households already can.
- **The record.** A new chronicle kind tells where it began as the kernel knows, how many buildings burned, who fought it and who died. What people know is their own: "found burning at Wren's" unless someone saw it start.

### 1.5 Dryness and wind in the weather

**Reports.** Fuel moisture should follow recent weather, wind should make exposure directional, and drought should dry fuel and lower firefighting water together (12-03 §1.5). Weather needs a separate gust variable, as daily mean wind cannot drive rapid fire spread (03-03 §1.4). Storm rain belongs to the precipitation model, not added on top (03-03 §2.4).

**My proposal.**

- **A dryness store** per world, filled by dry warm days and emptied by rain (a design prior), saved as one number with the weather.
- **A daily wind,** speed and direction, drawn as an AR(1) about a prevailing direction the land profile names, from its own key so that rain and temperature draws stay as they are. On the month's storm day the wind is the storm's.
- **Shown** on the clock and in the weather panel, so a reader can see why a fire ran east.

### 1.6 Masonry, brick and tile as building systems

**Reports.**

- **Chains.** Brick: prepare clay, mould, dry, fire, cool, grade; stone: remove overburden, detach, rough-size, dress; lime: calcine, slake, mix, cure; tiles need their own breakage and shape parameters (11-08 §1.1). Production quality persists into the building (11-08 §1.2). A covering sets pitch, overlap and dead load, so changing it regenerates the roof's demand (11-08 §1.3), and cost emerges from the whole chain (11-08 §1.4).
- **Making.** Moulding takes 15–25 person-hours per 1,000 bricks, 19–33 with digging (M/L); green bricks dry 3 days, then at least a week; a small wood-fired kiln fires 4–5 days; adobe dries a month or more, with about 10 % rejected (11-08 §2.2). Plain fired brick takes 10–80 person-hours a tonne and 0.2–1 kg of dry wood a kilogram; quicklime 10–60 person-hours a tonne and 1–5 kg of wood a kilogram (07-05 §2.3, authored ranges, low). Wood holds about 15.6 MJ/kg (07-06 §2.3). Quarrying 8–80 person-hours a cubic metre of accepted stone and dressing 4–40 per square metre are proposed envelopes (11-08 §2.2, P).
- **Laying.** Brick masonry 1.5–4 pd8/m³, rubble 3–6, adobe laying 1–3, tile installation 0.15–0.35 pd8/m² (11-04 §2.2, low to medium). One recipe takes 494 bricks and 0.25 m³ of mortar a cubic metre (11-04 §2.3).
- **Strength.** Rubble masonry 1.0–2.0 MPa and solid brick in lime mortar 2.6–4.3 MPa as assemblies, not units (11-05 §2.3; 11-08 §2.1). An adobe wall's height over thickness is fairly stable below about 4–6 and poorer above 9–12 (11-05 §2.5).
- **Durability and weight.** Clay tiles often last about 100 years; thatch wants significant mending every 2–3 years (11-08 §2.4). At 65 kg/m², tiles on 100 m² of plan at 45° weigh about 9.2 t (11-08 §4).
- **History.** Stone construction, sun-dried earthen units and lime burning are all documented in Neolithic communities; kilns came gradually; fired bricks are secure around 3000 BCE (07-05 §3.1, §3.3; 11-08 §3.1).

**My proposal.**

- **Content, not code, for the vocabulary:** goods `mud_brick`, `brick`, `roof_tile` and `quicklime`; a `kiln` that stays where built, like the oven; techniques for walling in stone, moulding and laying mud brick, firing brick and tile, and burning lime (07-05's L3, E1, C2–C3 and B1), learnt and found under ADR-0008. Which ones founders bring is open question 3.
- **Batches.** A kiln firing is a batch recipe with fuel per firing, a break share and its days; green bricks dry as elapsed time and a wet day spoils a share (a design prior).
- **Grammar.** A frame version 2 expands `WALL_KIND` 1–3 (rubble, mud brick, fired brick) as load-bearing walls on a stone footing (`FOOTING` 1), and a covering kind (thatch or tile). An appended `GroupKind::MassWall` is checked by height over thickness and crushing, wears at its foot by wetness as daub does, and weakens while wet (11-08 §5.2). The hut stays frozen.
- **Cost emerges.** A hut of 3 m radius has about 54 m² of roof at 45° (my arithmetic). Tiled at 65 kg/m² it carries about 3.5 t against 1.6 t of thatch, and at 07-05's envelope for brick its tiles take 0.7–3.5 t of firewood to fire (my arithmetic), against about 1.9 t a household of five burns in a year (my arithmetic). The rafters must carry it under the existing checks.
- **Fire caution.** Each settlement remembers buildings lost to fire by covering and wall kind against their building-years, as caution does for failures. A household's design weighs the loss it believes likely, so tile and stone can spread with no law at all.

### 1.7 Building rules: how a village comes to want a code

**Reports.**

- The chain runs: experienced problems, competing explanations, proposed remedies, authorization, affordable choices, imperfect enforcement (11-09, main finding). Liability is a separate capability from prevention (11-09 §1.1).
- Attention rises with deaths, displaced households, destroyed wealth, interrupted commerce and proximity to influential people; responses include punishment, rebuilding help, works, ritual, restrictions or nothing (11-09 §1.2).
- **What codes required.** Rome after 64 CE: street layout, height, materials, water and firefighting equipment (11-09 §1.2). London after 1666: brick or stone outer walls with wooden parts still allowed, house classes of 2, 3 or 4 storeys, sworn surveyors, and a party wall the neighbour paid half of at 6 % interest (11-09 §1.2, §1.5, §2.1). Edo: fire-resistant street fronts, less inside the blocks (11-09 §1.7). Istanbul: poorer owners kept timber with fire precautions (11-09 §1.5); in 1849, 10.5 m for timber and 15 m for masonry (11-09 §2.1). Fidenae after a collapse: tested ground and a 400,000-sesterce qualification (11-09 §2.1; 11-06 §3.2). Trajan refused Nicomedia a 150-man brigade for fear of factions (12-03 §3.2). Lima's elite resisted height limits (11-06 §3.6).
- Rule families: material and assembly, dimensions and load, separation, neighbour relations, escape and operation, siting, process; a requirement to achieve a property differs from one naming a technique (11-09 §5.2). Each rule needs an applicability predicate and activation mode: "Never rebuild the entire city when a law passes" (11-09 §1.7). A rule to keep equipment creates an inventory obligation, not a permanent bonus (11-09 §1.8).
- Compliance is a choice weighing risk, reputation, expected sanction and cost (11-09 §5.3); sanction needs inspection, detection and execution (11-09 §1.6). For early farming villages, start from households, shared obligations, customary dispute resolution and craft traditions (11-09 §3). "A rejected stringent proposal followed by an affordable partial rule is a valuable emergent outcome" (11-09 §1.5).

**My proposal.**

- **Two issues** (appended): `burned`, when a household lost a building to fire or its members saw one burn, within memory; `gave_way`, when they saw or heard of a building giving way. Each opens moves and weighs toward none (ADR-0013 §5).
- **One policy kind,** `building_rule` (appended), whose templates are content: (a) new roofs within a set distance of another building must not burn; (b) new walls of named kinds; (c) a least gap between new buildings; (d) each household keeps water and a ladder, renewed; (e) no fire left burning with nobody awake; (f) after a collapse, a cap on what lofts carry. Each template names a property or a technique (11-09 §5.2) and binds new building from the day the law takes effect.
- **Who proposes.** Someone holding the issue who knows the technique the rule needs (11-09 §1.4), when their household's forecast gains. A household forecasts the fire loss it believes avoided against what its next building would cost under the rule. A household that builds nothing soon pays nothing yet, so grandfathering lowers opposition (11-09 §1.7).
- **Attribution** comes from what people saw. A fire nobody saw start is blamed on where it was first seen burning.
- **Compliance is chosen,** weighing the norm that the gathering binds, with no sanction in v0, as for `Curfew`. A tile roof is easy to see, so a later law may let the watch or a case enforce it (open question 5). A neighbour whose house burns from a noncompliant roof may hold a grievance under ADR-0016's rule of harm, expectation and blame.
- **Why this house is stone** (plan §2): the readout names the rule it was built under, derived from its start day and the law's history, or the caution and taste that moved its builders.

### 1.8 Floods from the weather

**Reports.**

- Water balances: precipitation equals evapotranspiration, exports and storage change, and the upstream catchment may lie off the map (03-02 §1.1). Flood extent comes from the inflow, the channel's capacity, connected topography and storage, never "everything within a radius of the river"; record depth, duration and velocity; treat levees as hydraulic connections (03-02 §1.2).
- A 100-year threshold is crossed in about 26 % of thirty-year runs and 63.4 % of centuries (03-02 §1.2). Bankfull recurrence had a median of 1.4 years at forty Ohio sites, low as a universal prior (03-02 §2.1).
- Damage depends on depth above the floor, duration, velocity, debris and scour; floods spoil stores, take bridges, delay planting and cut access (03-07 §1.1). JRC curves give 0.33–0.49 of maximum damage at 0.5 m and 0.49–0.71 at 1 m, as aggregate envelopes only; raised and ground floors, and stored grain, need their own functions (03-07 §2.2 B).
- Bridges fail by undermining (11-06 §1.2); 52.9 % of 503 recorded US bridge failures were hydraulic (11-06 §2.1). Wet earth weakens reversibly and erodes irreversibly (11-08 §5.2). Katrina killed about 1 % of the exposed, a case, not a rate (03-07 §2.4).

**My proposal.**

- **A runoff store** per world, fed by the weather's daily rain and melt reaching the ground (`DayWater::input_mm`) and draining by a recession (a design prior). Each reach's flow is its mean discharge times the store against its long-run mean (my simplification of 03-02 §1.1). One source then gives floods, low flow in droughts, fords that open and close, and water for irrigation.
- **Stage and extent.** Stage follows Manning's relation on each reach's hydraulic-geometry channel. Water above bankfull fills that reach's floodplain compartment through a storage curve precomputed from its cells' height above drainage, so extent is limited by volume and connection. Daily steps; flash floods wait.
- **Effects.** Fords close above `ford_max_discharge_m3s`, and deep cells stop being walkable. Water above a building's floor spoils goods on that floor (raised stores and lofts keep theirs), wets daub and earth walls, and soaks post feet. A field under water for days loses a share of its standing crop (a design prior: no report gives emmer's). A crossing over a reach in flood is checked for scour by a margin its bridge system authors.
- **People leave water.** A household whose floor is wet goes to kin or the hearth until it falls. No drowning roll in v0 (open question 9).
- **Memory.** Each household remembers the deepest water at its home and fields, fading as caution does. The chronicle names the flood, its peak and what it wetted.

### 1.9 Storms

**Reports.** A storm brings wind and rain together; "roof loss increases rain damage" (03-07 §1.1). No "cyclone every x years per coastal village" is defensible (03-07 §2.1). Wind pressure is `q = ½ρV²`: about 0.54, 1.50 and 2.94 kPa at 30, 50 and 70 m/s (03-07 §2.2 C). Check wind for pressure and uplift at connections (11-05 §5.4). One event is evaluated once, never rerolled (11-06 §5.2), and disasters cause joint failures (12-07 §5).

**My proposal.** Keep ADR-0012 §5's monthly storm as the world's storm. Give it a direction, let it drive spread on its day, and add uplift on coverings: thatch held by its quality is torn off above an authored load, so the roof leaks and goods spoil by the existing rules. Today every failure of a lived-in building on the dashboard was a hut roof in a storm (README), so the band must be checked again.

### 1.10 Levees and banks as public works

**Reports.**

- A levee's volume is `L(HC + zH²)` (11-12 §1.5). A 100 m levee 1.5 m high with a 1 m crest and 2:1 sides is 600 m³ compacted, 667 m³ of bank and 800 m³ loose; it takes about 506 worker-days and 28 working days for a fixed crew of twenty, haul-limited (11-12 §2.5 C). Fill goes in 0.15–0.20 m layers (11-12 §2.3).
- Failure comes by overtopping, piping or slope instability, at a hazard set by load, duration, material, foundation and damage, never a universal rate (11-12 §4.2). Upkeep fights crest settlement, erosion, animal damage and seepage (11-12 §4.1). In 1825 the Netherlands recorded 22 dyke breaches and 379 deaths (11-12 §4.3).
- Protection can increase future exposure (03-07 §4; 11-12 §4.4). Unpaid labour is not costless (12-08 §1.1); support depends on benefit, burden, fairness and season (12-08 §1.2). Major defensive works take 1–10 years to recover (03-07 §2.5, D).

**My proposal.**

- **An issue,** `flooded` (appended): a household's home, store or field stood in water within memory, as its members saw.
- **A policy kind,** `build_bank` (appended), shaped like `build_crossing`. The sponsor names a line between the river and the wetted homes or fields; its crest is the highest water remembered plus a freeboard (a design prior), and its earth comes from a borrow ditch on the river side (ADR-0010: no material without a source). Work is asked in equal shares, given by choice, with a crew cap per segment (11-12 §1.2's narrow fronts).
- **Forecasts.** A household weighs the losses its remembered floods cost it against its share of hours. Those never flooded oppose, so whether a bank passes depends on the custom and who came.
- **An earthwork kind,** `bank` (appended): a line of segments below the grid (11-12 §5.1), each with crest height, settlement, erosion and a breach state. Water reaches cells behind it only over the crest or through a breach, and rain behind it ponds unless drained.
- **Failure is conditional.** Overtopping erodes a segment by depth and duration; piping grows with days of head against its quality. Nobody must mend a polity's bank unless a law asks (12-08 §1.4); a neglected bank is a finding, not a bug.

### 1.11 Farm ditches and terraces

**Reports.**

- Terraces and drainage help only where their constraint binds; broad-bed drainage raised wheat 25 % at one site and 131 % at another, by waterlogging (03-04 §2.8, M). "There is no defensible universal 'terrace = +X% grain' coefficient" (ibid.).
- A canal needs an intake, connected reaches, head and an outlet; a drain needs somewhere lower (11-12 §1.6). Forming canals moves 0.8–1.2 m³ a worker-day; gradients run about 0.05–0.2 % and sides 1.5:1; Manning's n is 0.025 for an average canal and 0.040 for a weedy one (11-12 §2.2, §2.3), which cuts capacity to 62.5 % (11-12 §4.4). A 100 m ditch of 40 m³ takes 33–50 worker-days (11-12 §2.5 A).
- A hectare at 5 mm a day needs 50 m³ a day net (03-02 §1.5); surface application is about 60 % efficient (03-02 §2.1); irrigation without drainage brings waterlogging and salt (03-02 §1.5).
- Terraces come as progressive bunds or excavated benches (11-12 §1.7): 150–350 worker-days a hectare for *fanya juu* (11-12 §2.2) and 400–520 for one bench design (11-12 §2.5 D).

**My proposal.**

- **Irrigation ditches first.** Water binds in dry years (the M3c demo's dry year halved the harvest, README). A household, or several by agreement, or a law, digs a ditch from an intake to its fields at a gradient of at least 0.05 %. In a dry spell its people choose to water a field, adding to its root zone what the ditch delivers times the efficiency, so the gain comes only through ADR-0012 §2's water response. Sediment and weeds raise n; upkeep clears them.
- **Drainage ditches only with waterlogging.** Add saturated days on fields a flood wetted, or on low flat ground after heavy rain (a design prior), and let a drain shorten them. Without that, defer drains.
- **Terraces deferred.** Fields have no slope effect on runoff or erosion, and nobody farms slopes yet (ADR-0012 §7 still holds).
- **An earthwork kind,** `ditch` (appended): a line with cross-section, gradient and a sediment stock. ADR-0010 §3's refusal beside water needs amending for intakes, with drainage changed openly.

### 1.12 God tools for fire, flood and storm

**Reports.** "A god-created storm should use the same wind, flood, evacuation, and repair systems as a naturally generated storm. Only its origin differs", and an explicit origin flag keeps interventions out of recurrence records (03-07 §1.3). Plan §2 logs every use in the chronicle.

**My proposal.**

- **Fire:** set a building's covering alight. **Flood:** add water to a reach's runoff for some days, so routing decides what floods. **Storm:** a storm today of a chosen strength and direction, with its loads, wind and spread. Each runs the ordinary machinery.
- **People cannot tell.** They remember a god's flood as any flood. Statistics (return levels, the dashboard, Gate B) exclude interventions by their origin.
- Drought and plague belong to the weather and disease work.

## 2. What to defer, and why

| Defer | To | Why |
|---|---|---|
| Brigades, pumps, fire towers, insurance companies | Towns (M7–M8) | Villages fight fires by household and neighbour (12-07 §4); a crew needs equipment and people together (12-03 §3.4) |
| Lightning | Until a source gives a rate | No report gives one (open question 1) |
| Arson and fire as a weapon | M6's war brief | Deliberate burning comes from agents' decisions (12-03 §1.1) |
| Firebrands as their own route; wildfire in vegetation | Later | Routes differ by assembly (12-03 §1.4); vegetation fuel is not modelled |
| Builders' liability, inspectors, permits | M7 courts | Liability is a separate capability (11-09 §1.1); households build their own |
| Zoning, street widths, height classes | M8 planning | Plan §5.1, §7 |
| Terraces; erosion and salt | When slopes are farmed | 03-04 §2.6–2.8; ADR-0012 §7 |
| Sub-daily flash floods, sediment and channel change | Later | 03-02 §1.1–1.2 |
| Earthquakes | M8 | Plan §7; masonry's class A vulnerability is the cost then (03-07 §2.2 A) |
| Drought and plague god tools | Weather and disease work | Plan §2 |

## 3. Risks and validation

**Plot.** Watch for tuning spread until a great fire happens, timers on fires or floods, and issue→rule weights ("a fire makes a masonry code"). The demo's code must come from who lost what, who knows a noncombustible system, and what the gathering's custom decides. After a week without a proposal following a large fire, nudge a prior (the weight households give losses they saw, or memory's half-life), never an event, and log a `NUDGE:` (plan §1, rule 3).

**Every village burns, or none does.** With huts 1 m apart, R_fire may exceed one on most windy dry days, and villages would burn out. Calibrate in 12-03 §5.7's order: ignition and reporting, one building, pairs, task times and water, and only then settlement losses. Answer through response and spacing, not by lowering spread below the evidence. Burned settlements in the archaeological record do not by themselves identify accidental fire (12-03 §3.1; 11-06 §3.1), so no village-level burn rate exists to aim at.

**Leaking truth.** Test that no forecast reads the kernel's fire cause, river stage or future weather (03-03 §5.2), and that an institution acts only on what reached it (12-07 §6.6, causality).

**Scale.** Fires are rare and local. Step only burning and threatened buildings (12-03 §5.5), rebuild the exposure graph only when buildings change (12-03 §5.1), and add fire and flood aggregates to Gate B (ADR-0011 §5).

**Tests from the reports.**

- **Small fires dominate** (12-03 §2.1, §4): most ignitions are put out by whoever is there. Count ignitions, escaped fires, buildings lost and conflagrations apart (12-03 §1.2), and sweep the rate over 1–100 per 1,000 household-years (12-03 §2.5).
- **The 20-dwelling test** (12-03 §2.2): 20 huts 1–2 m apart in 4.2–6.9 m/s of wind with four lit should all burn within minutes. The same village with 4 m gaps, tile roofs or no wind should lose fewer buildings, with no gap a guarantee (12-03 §1.6).
- **One changed variable** (12-03 §5.7): roofing, spacing, wind, watch or water; compare incidents, escaped shares, buildings lost, water used and hours.
- **Seasonality emerges** from dryness and wind, not a fire season (12-03 §1.5). Burned area does not predict deaths (12-03 §4).
- **Char** (11-08 §2.5): a 200 mm member keeps a 128 mm core after an hour of exposure.
- **Codes** (11-09 §4): new and old cohorts stay visibly different; at a 1 % yearly replacement, 61 % of the old stock remains after 50 years, at 2 % 36 %; a legal patchwork is not corruption.
- **Floods** (03-02 §1.2, §4): annual maxima give return levels; a 100-year threshold is crossed in about 26 % of thirty-year runs. Wetted buildings fall inside 03-07 §2.2 B's envelopes in aggregate.
- **Shared events** (11-06 §4): failures cluster in floods (112 of 503 bridge failures in 1993); floods take crossings.
- **Works** (11-12 §2.5 A, C; §4.4): a 100 m bank takes about 500 worker-days and stays haul-limited; a weedy ditch carries 62.5 % of a clean one.
- **Saves:** save → load → save is exact in the middle of a fire or a flood, and a god's fire keeps its origin across a save.

**Observer.** Fire as an incident panel: when it started, who saw it, who came and did what, water carried, and what burned. The map shows burning and burnt buildings, flood extent and depth, banks and ditches. Coverage is measured, not drawn as a radius (12-07 §1.8). The chronicle speaks plainly: "Fire at Wren's hut on a dry windy night took nine roofs; 14 neighbours carried water; Ada's child died."

**Choices that are expensive to reverse.** Plan §1 allows about three ADRs a milestone, and war and water want theirs. I suggest one ADR for hazards: fire records and their counts, the runoff store and flood memory, wind and dryness on `Weather`, origin on events, and the causes *burned* and maybe *drowned*. Masonry would amend ADR-0009 (frame version 2, `MassWall`, char as loss), and banks and ditches ADR-0010 (lines, intakes, drainage changed openly). Policy, issue, earthwork, group and cause kinds are append-only.

## 4. Open questions for the designer

1. **Lightning.** Plan §5.4 names it, and no report gives a rate. Include it with a flagged design prior, or defer?
2. **Wind.** Add a daily wind series to the weather (new keyed draws, saves change), or use only the storm day's?
3. **Founders' repertoire.** Do founders know walling in stone or mud brick, or must it come by finding, contact or the god tool? The demo depends on it.
4. **Rule templates.** Which of §1.7's (a)–(f) belong in M6? Should a rule name a property ("does not burn") or a technique ("tile")?
5. **Enforcement.** Compliance by choice alone, as `Curfew`, or may the watch see a new noncompliant roof and bring a case under ADR-0015?
6. **Pulling down.** May a person pull down another's building in a fire without leave, and does a law grant that power?
7. **The dashboard.** Do fire losses get a row of their own, apart from §4.7's structural failures?
8. **Rivers by season.** Does this brief or the water brief own the runoff store that floods, fords and irrigation share?
9. **Drowning.** Are deep flood cells simply unwalkable, or may people be caught, with a cause *drowned*?
10. **Banks.** Equal shares of work, as for crossings, or shares by land behind the line?
11. **Drainage.** Add waterlogging so drains have value, or defer drains with terraces?
12. **God tools.** Should people remember a god's fire or flood as any other, with only statistics excluding it?
