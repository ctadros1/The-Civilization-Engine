# M5 design brief: settlements, migration, roads and bridges

**Scope.** This brief covers M5's (plan §7, "M5: Neighbors") physical side:

- how 2–3 settlements come to exist in one world: founding groups at world setup, and splinter founding (when, why, who goes, where they settle);
- migration between settlements, leaving the map, and the migration-wave god tool;
- roads between settlements and travel times;
- bridges, with span and load limits per technique;
- the compute budget for 3–8k people in 2–3 settlements (plan §4.3).

Trade and currencies (`m5-trade.md`), the spread of techniques and style (`m5-diffusion.md`) and relations between polities (`m5-relations.md`) have briefs of their own.

**What exists today (checked in code).**

- `civ_land::Settlement` holds an id, name, founding date, hearth and two harvest figures. Each settlement has its own polity (ADR-0013 §1: one to one in M4, "M5's new settlements need the separate identity"), market, knowledge record and buildings.
- `found.rs::choose_site` scores 48 random candidates (`camp_candidates`) on wild food within 2 km, arable land within a 30-minute field walk, water, slope and flood height. It ignores other settlements and land already held. The observer sends 1–20 families; within 600 m of a lived-in hearth they join it, otherwise they found their own (`SPAWN_JOIN_M`).
- A household out of food weighs leaving (`leave_chance`, M4a slice Z). One that leaves is removed from the world (`life.rs::leave`), and so is an exile (`life.rs::exile`); only their records remain.
- **Walking** follows Tobler's function: 5 km/h on the flat on a trail, 0.6 of that off trail. Rivers under 3 m³/s mean discharge are waded at a quarter of walking speed; larger ones cannot be crossed (`nav.rs`). Channel width is 4·√Q, so the ford limit is about 7 m wide and the river valley's trunk (about 14 m³/s) about 15 m (my arithmetic).
- **Trails** are wear only (ADR-0004 §4). With `wear_per_walk` 0.01, a 120-day half-life and `trail_at` 0.3, about one walk every four days across a cell makes a trail and one every ten keeps it (my arithmetic, from the wear equation's steady state). A worn trail restores full speed and never beats it, so for walkers a built road adds no speed over a trail.
- Dashboard worlds are 1,024 cells (8.2 km) a side and found one band each. The settlement-size row is grey, "one settlement a world until regions (M5)".
- **Speed.** A year of 1,000 people at Max takes 485 s. A day of 2,000 costs 3.2 times a day of 1,000. Route searches take 86 % of the slowest days, because the A* bound assumes trail speed everywhere (plan §9, "Slice X").
- **Shared climate.** Every settlement in a world shares each year's climate factor, keyed by seed and year alone (plan §9, "Villages fail in their seed's bad years").

**Where the reports agree.** I treat these as invariants:

1. **Households choose.** Migration is a household choosing among feasible plans, not a flow toward the most attractive place (05-06 opening, §5.1; 10-01 core recommendation; 10-02 §2.3). Wanting to leave and being able to are separate (05-06 §1.2).
2. **Founding is a project.** A coalition drawn from real relationships founds in stages, and it can fail (05-06 §1.3, §5.2; 10-01 §1.5, §5.2; 10-02 §2.8). No population threshold splits a village (05-06 §1.3; 10-01 §1.5).
3. **No world truth.** Choices use remembered or reported conditions (05-06 §5.1; 10-01 §5.3). A route that exists is not one a traveller knows (07-08 §5.1).
4. **People are conserved.** ΔN = B − D + I − O per settlement; people in transit stay counted; an off-map reservoir is explicit (05-06 §1.1; 10-02 §2.1). Settlement sizes are outputs, never controllers (10-02 §6.2, §6.4).
5. **Separate identities.** Settlement, site and polity each have their own (10-01 §1.8, §5.1; 10-02 §6.1).
6. **Infrastructure works through the network.** It matters through the travel costs and feasibility it changes (07-08 §1.7; 10-01 §1.4; 10-02 §2.5). A crossing is a maintained service, and opening one guarantees no upkeep (11-07 opening, §4.2).
7. **One model.** One authoritative model runs at every speed and camera position. Coarsen execution time before people or causal state (01-03 §1.1, §4.1, §4.5).

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| Settlement and site | Settlement gains its parent, founding coalition and abandonment date; the site record outlives its people | Founding; the last resident leaving |
| Residence | Per person, a short history: settlement, from, until, why | Moves, marriage, founding, exile |
| Founding coalition | Organizer, member households, target site, goods set aside, stage (forming, scouting, moving, camped, established, failed), dates | Households' reviews; the organizer's recruiting |
| Known places | Per household, a few records: settlement or site, when last heard of, food and land impressions, kin there, source | Visits, kin, news claims (ADR-0016) |
| Migration accounts | Per settlement and year: births, deaths, arrivals and departures by origin and destination; moves and distinct movers | Every move |
| Crossings | Crossing (reach, spanned cells, approaches), spans in order (system, length, members, quality drawn once), supports, owner, project stage, the owner's believed condition | Building work, decay, upkeep, loads, a law |
| Migration wave | One intervention record and the households it brought | An observer command |

**Derived, not saved:** the road graph between settlements, travel times between hearths, candidate site bundles, settlement summaries for choosing a destination, and landmark tables for routing.

### 1.1 Settlements, sites and polities

**Reports.**
- A site, its community and its political identity need separate persistent identifiers. Abandonment is a process: decline, departure, reuse, disuse and reoccupation (10-01 §1.8).
- "Do not make settlement synonymous with polity" (10-01 §5.1). 10-02 §6.1 separates settlement, catchment and territory.
- Abandoned places keep their buildings, fields, roads and claims (10-02 §2.8).

**My proposal.**
- Split the site (kept after its people go) from the settlement (the community) and the polity.
- A daughter founds a polity under a copy of the custom its founders lived under when they left. Membership stays residence (ADR-0013 §1).
- Abandoned fields and huts already stay. A later group that chooses the site finds its fields cleared.

### 1.2 Founding groups at world setup

**Reports.**
- "Do not let entity update order decide which settlers receive the best land" (10-01 §5.5). Two founding groups must not both count the same field (10-01 §5.4).
- Founding cohorts are 5–20 households, for scenario initialization only. The food buffer is the time until dependable production plus 1–3 months (10-01 §2.3). For isolated crop-dependent projects, 05-06 §5.4 tests 6–12 months.

**My proposal.**
- **Groups.** The new-world dialog takes 1–3 founding groups, each with a size.
- **Joint site choice.** Each group draws candidates as `choose_site` does. A softmax then picks one joint assignment, over the summed scores of assignments whose field reaches do not overlap. Off trail, a 30-minute reach is about 1.5 km, so hearths at least about 3 km apart qualify. Three groups of 48 candidates give about 110,000 sums, which costs nothing (my arithmetic).
- **One people or strangers.** A setup option. One people share taste, hold a few kin ties across groups and know one another's sites; strangers know of no one (the relations brief's "unknown").
- **Reaching 3–8k.** At 1 % a year a population takes about 230 years to grow tenfold (10-02 §3.2), so 3–8k comes from founding large or from waves, not from growth in a 50-year run (my arithmetic). Villages of hundreds founded at once have died in their first bad harvest (plan §9 NUDGEs), so large groups need the buffers above.

### 1.3 Splinter founding

**Reports.**
- **Process.** Dissatisfaction or opportunity → coalition → site investigation → resource commitments → departure → establishment or failure (05-06 §1.3).
- **Alternatives.** Stay and intensify, negotiate institutional change, join another settlement, or found (10-01 §1.5). Fission is an alternative to building integrative institutions (05-06 §1.3, Bandy).
- **Who goes.** Related households, a leader's followers, a faction or a shared project, never a random half (05-06 §1.3). Lineal fission differs from random fission (10-01 §1.5, §4).
- **Requirements.** A feasible destination and a viable plan: labour, tools, seed, shelter and food to the next reliable supply (05-06 §1.3), tested month by month to the first dependable food (10-01 §1.1).
- **Outcomes.** A project, not a spawn: members stay identifiable, and partial outcomes (withdrawal, an outpost, return after a failed harvest) are allowed (05-06 §5.2; 10-01 §5.2). Parent–daughter ties persist (05-06 §1.3).
- **Knowledge and agreement.** Scouting reveals part of the truth, and a familiar mediocre site may win. A coalition is not the mean of its households' scores (10-01 §5.3).
- **Rate.** No report gives a founding rate or a minimum size. A daughter at half its parent's size needs 139 years at 0.5 %, or 69 at 1 %, to regain it, so frequent splitting needs immigration, small groups or causes other than growth (05-06 §1.3).

**My proposal.**
- **When.** A household's elder reconsiders residence once a year on a keyed day, and at events (05-06 §1.2: life-course triggers):
  - a household formed, or an inheritance;
  - a shortfall in the yearly field allocation, or no vacant arable land within reach;
  - a grievance against the polity still unresolved after a refused petition;
  - an outlook below subsistence.

  Founding is one plan beside staying, joining a known settlement and leaving the map.
- **Why.** Each plan is scored in the household's own units:
  - the M4a forecast (expected log of a year's food above subsistence) at the candidate site against at home;
  - autonomy, as the activation of its grievances against the polity or an office (05-06 §5.1's β_a; ADR-0016);
  - support, as regard-weighted kin who would come or stay near;
  - the cost of breaking fields and building before the first harvest, in hours of its own work.

  The advantage must hold over two consecutive reviews except in a crisis (10-01 §2.3 gives 1–3; two is a tuning value).
- **Who.** A household whose founding worth passes its threshold becomes an organizer. It recruits as a faction's organizer does: kin and households it regards, then their contacts. Each household joins only if founding with this coalition beats its own best plan. A faction's organizer (ADR-0017) gains "leave together" beside petitioning and refusing.
- **Viability, not headcount.** The coalition must hold food and seed for the wait to its first harvest plus 1–3 months (10-01 §2.3), tools, and enough adults to break the area it needs before sowing; otherwise it waits or dissolves. 05-06 §5.4 and 10-01 §2.3 both test 5–20 households, and 05-06 starts at 10.
- **Where.** Candidates are 128 m patches (10-01 §5.5: 100–250 m cells) that a member has walked or heard of. They are scored as `choose_site` scores, with land held or worked by others excluded (10-01 §5.4) and yields forecast at the 10th–30th percentile (10-01 §2.3). Walking time to the parent counts as support (05-06 §1.3: "a hamlet near its parent can rely heavily on exchange and assistance"). An abandoned site's cleared fields lower its cost (10-01 §1.7).
- **Stages.**
  1. Scouting walk.
  2. Goods set aside.
  3. The move, as trips carrying goods (07-08 §1.8).
  4. Camp. The settlement record is written at the first night in camp, so the chronicle reports a process under way (10-01 §5.2).
  5. Fields and huts.
  6. Reassessment; a household may return.
- **What goes with them.** Ties, kin, taste, the custom and knowledge. A parent can lose a technique when its last knower leaves, as it already can.

### 1.4 Migration between settlements

**Reports.**
- **Choosing.** V_hj = β_c ΔE[U(consumption)] + β_s Δsafety + β_k Δsupport + β_a Δautonomy − β_m switching costs, with a softmax over known alternatives. Travel costs go in the budget, not into a second distance penalty. Consideration hazards take the form 1 − e^(−λΔt), never as overlapping rolls (05-06 §5.1).
- **Who moves.** A moving party, not one roll per member, at life events (05-06 §1.2). The poorest are not automatically the most mobile (05-06 §4). Chain migration lowers costs for connected people (05-06 §1.2).
- **Famine and emergencies.** Famine migration follows expected access to food, with no universal function (05-06 §1.4). Emergencies bypass review delays and are reviewed daily (05-06 §5.1, §5.4).
- **Arrival.** Newcomers meet real housing, land and contacts. Shortage causes crowding or onward movement, never a cap. Without such feedback, the first slightly favourable settlement absorbs the world (05-06 §5.3).

**Where they disagree.**
- **Routine reviews.** Every 6 months, 3–12 (05-06 §5.4), against 0.05–0.20 a household-year (10-02 §3.3).
- **Destinations per review.** 12, 8–16 (05-06 §5.4), against 3–8 (10-02 §3.3). With 2–3 settlements this second disagreement hardly matters.

**My proposal.**
- **Leaving becomes a choice of destination.** Today's famine conditions stay as the emergency trigger, reviewed daily. The household compares the settlements it knows (food impressions, kin), founding (rarely viable while starving, 05-06 §1.2) and the map's edge. All settlements share a lean year, so a neighbour will often be short too (05-06 §1.2 asks whether origin and destination fail together).
- **Ordinary moves.** At §1.3's reviews, joining another settlement is compared with staying by the same terms, and only a known settlement can be joined.
- **Marriage.** Partner search includes adults of other settlements whom the person has ties with, and the couple lives in either partner's settlement. It is cheap, it is 05-06 §1.2's life-course channel, and it seeds kin across settlements for chain migration.
- **Arrival.** Newcomers camp and build as sent families do. Land comes through the property regime.
- **Information.** Visits and kin fill a household's known places. News of another settlement travels as ADR-0016 claims ("Several polities (M5) carry news between settlements"). Settlement summaries are cached (05-06 §5.5) and rebuilt at midnight.
- **Accounting.** Births, deaths, arrivals, departures and residence histories give moves and distinct movers (05-06 §1.1). The chronicle names origin and destination.

### 1.5 The migration wave

**Reports.** Plan §2 lists the tool; no report I read covers god tools. The off-map reservoir must be explicit (05-06 §1.1). Displacement is bursty and can overwhelm a destination, with mortality arising through its food and shelter (05-06 §4).

**My proposal.**
- **Arrival.** The observer picks a point. 5–50 households of one band of origin walk in from the nearest map edge over 1–7 days (both tuning values). They share a taste, hold kin ties and ties among themselves, and carry a stated number of months of provisions.
- **The boundary.** The wave is one intervention under ADR-0016 §5: a perceptible arrival, logged and traceable.
- **After arrival.** They are ordinary people. They join, found or move on by §1.3–§1.4, knowing the settlements within a tuning distance of their camp. The tool never chooses a destination or a size for any settlement. It extends `spawn_families`.

### 1.6 Roads and travel times

**Reports.**
- **Complementary network.** A road ending at an impassable river does little; a bridge or ferry can unlock the route. Infrastructure is built when its benefits justify building and keeping it to an actor able to organize that (07-08 §1.7).
- **Reliability.** Road improvements bought reliability as much as speed: English winter freight before 1750 cost 30–50 % more than summer freight (07-08 §4).
- **Persistence.** Roads are among the assets that make a place persist (10-01 §1.7).
- **Access by network.** Access is measured by network travel costs, not circles (10-01 §1.6; 08-16 §1.2), and a near settlement across a dangerous crossing can be less accessible than a far one (10-02 §2.5).
- **Before roads.** An early agrarian world must stay walkable before roads exist (01-08 §5).

**Today's model gives a built road little to do.** Off-trail walking is not slowed by weather or vegetation, there are no animals or carts (the trade brief defers them too), and a trail already gives full speed. 5 km takes about 1 hour on a trail and 1 h 40 min off it. A 16 km map's diagonal is about 4.5 hours on a trail, within 07-08 §2.1's porter day of 15–25 km (my arithmetic). So trips between settlements are day trips, and M5 needs no multi-day journey state.

**My proposal.**
- **A derived road graph.** The road graph between settlements is the monthly trail tracing (ADR-0004 §4), labelled with the settlements it joins. Travel times between hearths come from one reverse travel field per hearth over the whole map, rebuilt at each survey (01-08 §2G: reverse trees toward settlement gates).
- **M5 builds crossings (§1.7).** A kept way (cleared, drained, its wear held up) changes nothing people notice until carts, pack animals or weather-dependent ground exist. Built roads wait for carts (plan M8) and would meet ADR-0004's revisit trigger, "road edges need identities of their own".

### 1.7 Bridges

**Reports (11-07 unless noted).**
- **Four properties.** A bridge is a maintained service with four separate properties: structural capacity, usability, throughput and availability (opening).
- **Choosing a crossing.** The options are ford, stepping stones, ferry, a seasonal or permanent bridge, or a detour, chosen by lifetime generalized cost. "Allow a more expensive bridge at a better site to outperform a cheaper bridge on the shortest route." Different actors value different parts of that cost (§1.1).
- **Spans and supports.** Clear span is not crossing length; a bridge stores ordered spans and supports (§1.2).
- **Capacity.** For beams, M_max = wL²/8 + PL/4. Deck, main members, connections, lateral stability and supports each get a margin, and the weakest governs (§1.3). Suspension follows H ≈ wL²/8f and needs cordage, anchorage, erection skill and renewal; Inka fibre bridges spanned up to 45 m (§1.4).
- **Building.** Staged: site, foundations or anchors, temporary works, structure, deck, approaches. Maintenance replaces components (§1.7). A span without approaches is not a crossing (§5.7).
- **Floods.** Floods act through scour, debris and inundation, never as a generic annual probability (§1.6).
- **Loads.** Explicit load cases: 1 kN a person with baggage, 6 kN a pack animal, 20 kN a cart, crowds 1–5 kPa. The displayed restriction ("pedestrians only") comes from condition (§2.2). Limits apply to tare plus payload (07-08 §1.1).
- **Upkeep and craft.** "Opening a bridge should not guarantee a maintenance budget" (§4.2). Short spans stay useful after long ones appear (§5.1). Long gaps without rebuilding lose the craft (§3.2, Kintai).
- **Technique logic.** A bridge needs carpentry OR masonry OR fibre work, by recipe (07-08 §5.2).

**A disagreement with the plan.** Plan §5.1 authors "span and load limits per technique". 11-07's opening argues against "a technology with one fixed tonnage limit": spans are proposal envelopes, and capacity follows each bridge's geometry, members and condition. 11-07 has no "rope" type; the plan's rope is its natural-fibre suspension.

**My proposal.**
- **Systems.** Each bridge system is a content template: a span envelope (11-07 §2.1), member sizes, the techniques it needs, and work per unit (11-07 §2.3, as tuning values). Load capacity is computed per bridge. Techniques are found as ADR-0008's ordinary finds, never by date.

  | System | Span envelope | Techniques |
  |---|---|---|
  | Timber beam | Log 2–8 m; beam 3–10 m; trestle bays 3–8 m | `wood_shaping` |
  | Timber truss | 10–40 m | `jointed_frame` and a truss technique |
  | Rope (fibre suspension) | 10–45 m | Cordage: a new good and technique |
  | Stone arch | 3–20 m; 20–40 m for specialists | `stone_shaping` and a discovered arch technique |

  The Forest Service's modern log beams reach 6.1–18.3 m (11-07 §2.1): an analogue, not a default.
- **Geometry sets the options.** A fordable stream (under about 7 m) takes one log or beam. The trunk river (about 15 m) needs a pier in the channel (a trestle), a truss, a rope bridge or an arch. Which techniques can cross it follows from the river, not a gate (my arithmetic).
- **State.** 11-07 §6.1's records, reduced: a crossing (reach, spanned cells, approaches), spans and supports in order (supports keep foundation data for M6's scour), an owner and a project. Condition uses ADR-0009's component groups (capacity, demand, quality drawn once, decay by exposure). The owner's believed condition is kept apart (11-07 §6.1).
- **Loads.** Checked as people step on (11-07 §6.2). With people only, loads will rarely bind; in M5, span and decay decide (my inference).
- **Who builds and why.** A household (a log over a stream to its fields), or the polity through a public-work template in the M4 law pipeline, with labour contributed as obligations (ADR-0015). Its worth to each household is the walking its own recorded trips would save, plus the fields or settlements reachable only across it, in hours of its own work. Sites compete by lifetime cost (11-07 §1.1).
- **Building.** ADR-0009's staged construction: supports or anchors, centering for an arch, structure, deck, approaches, built by real workers.
- **Upkeep and failure.** Decay by exposure; repair replaces components, and nothing guarantees it is funded. A rope bridge needs renewal every 1–2 years (11-07 §2.4). A failure kills whoever is on the bridge and enters trust and caution for its technique (ADR-0009; plan §5.1).
- **Routing.** The bridge's cells become walkable at deck speed. A closure or failure invalidates routes hard; an opening invalidates them for quality (01-08 §4). No cached route may cross a fallen bridge (01-08 §7).

### 1.8 Compute budget

**Measured** (plan §9, "Slice X"): a year of 1,000 people at Max takes 485 s; 2,000 cost 3.16 times 1,000, roughly n^1.66 (my arithmetic, from two points only); route searches dominate the slowest days.

**My arithmetic.**

| World | Cost per settlement | One pool of people |
|---|---|---|
| 3,000 people | 3 × 1,000 ≈ 24 min a simulated year | ≈ 50 min |
| 8,000 people | 3 × 2,667 ≈ 2.1 h | ≈ 4.3 h |

Fifty years at 3,000 people would take about 20 hours a world, so the 50-year dashboard cannot run at M5's target population.

**Lower detail for unwatched settlements: the reports do not support it.**
- 01-03 recommends one authoritative model and coarsening execution time before people or causal state (§1.1). It calls adaptive local resolution "potentially useful later", with "substantial engineering complexity" (§1.2).
- Replacing stochastic production by its expectation "can remove famines, bankruptcies, and migration waves", which M5 needs (01-03 §2.2).
- Outcomes must not depend on the camera; Gate A compares camera positions (01-03 §4.1, §4.5). Dwarf Fortress shows the transition problems: identity, ownership, duplicated populations (01-03 §3.2).
- 10-02 §6.4 forbids turning one person into hundreds, or letting the performance ceiling become a demographic rule.
- The plan agrees: everyone is simulated in full through v1, with aggregate settlements in M9 (§4.5, §5.5).

**What they do support.** Semantic cadences that depend on neither speed nor focus (01-03 §1.2, §3.4), which TCE has. Staggered, infrequent strategic reviews with cached summaries (05-06 §5.5; 10-01 §5.5; 10-02 §6.3). Fewer and shared route searches (01-08 executive recommendation, §4).

**My proposal, in order.**
1. **Scope searches to the settlement.** Every per-household search (market, asking, company, deliberation) stays within the household's own settlement. Test that three settlements of 1,000 cost about three times one.
2. **Landmark bounds (ALT, 01-08 §2A).** Build them at each monthly survey on the surveyed metric, which routes use unchanged until the next survey, so the bounds hold all month.
   - Eight landmarks, both directions, u16 seconds: about 34 MB at 1,024² cells and 134 MB at 2,048².
   - If searches are 70–86 % of the time and fall fourfold, the overall gain is 2.1–2.8 times (my arithmetic).
   - It needs Slice X's one-time digest change, checked for equal route times.
3. **Reverse fields.** Per-hearth reverse travel fields for trips between settlements.
4. **Cache size.** Size the route cache by population (50,000 entries world-wide today).
5. **Parallel preparation.** Per-settlement preparation in parallel with a serial commit (01-03 §4.4), only after steps 1–4 and only if exact.

**Budget, my proposal.** A year of 3,000 people in three settlements in at most 10 minutes at Max; 8,000 measured, not targeted. The dashboard founds 2–3 bands a world, and scale is shown by 30-day benchmarks at 3k and 8k (Slice X's method).

### 1.9 Expensive to reverse

- **ADR candidate A: several settlements.** Site, settlement and polity identities; residence histories and migration accounts; coalitions as saved projects; what leaving the map means. It answers ADR-0013's trigger "Several polities exist (M5)".
- **ADR candidate B: crossings.** Bridges as structures under ADR-0009, crossing identities, route invalidation and owners. It answers ADR-0004's road trigger if built ways come in.
- **The ALT digest change** needs a §9 entry, not an ADR.

## 2. Numbers worth keeping

| Quantity | Value | Source | Status |
|---|---|---|---|
| Founding coalition | 10 households; test 5–20 | 05-06 §5.4; 10-01 §2.3 (5–20) | Design prior |
| Founding provisions | To dependable production + 1–3 months; 6–12 months if isolated | 10-01 §2.3; 05-06 §5.4 | Design prior |
| Fission sizes | Hutterite 104–251 people; Titicaca indices 157, 186, 277 | 05-06 §2.1; 10-01 §2.1 | Test ranges, not hazards |
| Daughter regains half size | 139 y at 0.5 %, 69 y at 1 % | 05-06 §1.3 | Arithmetic |
| Inter-settlement moves | ~2 per 100 residents a year; sweep 0.5–10 | 05-06 §5.4 | Output benchmark, not quota |
| Turnover, Clayworth | 38 % of residents left in 12 y; 401 → 412 | 05-06 §2.1 | Observation |
| Reviews; destinations per review | Every 6 months (3–12) and 12 (8–16), vs 0.05–0.20 a household-year and 3–8 | 05-06 §5.4 vs 10-02 §3.3 | Disagree |
| Founding search | Persistence over 1–3 reviews; 16–64 site bundles (48 today); yields at the 10th–30th percentile; field commute 15–60 min (30 in content) | 10-01 §2.3 | Design priors |
| Spacing | Hexagonal d ≈ 7.6 km for 50 km²; Skinner 7.79 km; Upper Xingu ~20 km | 10-01 §1.6, §2.1 | Geometry and regional anchors, not a law |
| Growth tenfold | 230 y at 1 %, 115 y at 2 % | 10-02 §3.2 | Arithmetic |
| Porter | 15–30 kg, 3–5 km/h, 15–25 km a day | 07-08 §2.1 | Design prior |
| Bridge spans | Beam 3–10 m; truss 10–40 m; fibre 10–45 m; arch 3–20 m | 11-07 §2.1 | Proposal envelopes |
| Test loads | Person 1 kN; pack animal 6 kN; cart 20 kN; crowd 1–5 kPa | 11-07 §2.2 | Test bodies |
| Work | Log footbridge 5 m: 5–30 person-days; beam 8 × 3 m: 80–400; truss 25 m: 600–3,000; fibre 30 m: 100–600; arch 10 m: 1,500–6,000 | 11-07 §2.3 | Low-confidence guardrails |
| Upkeep | Fibre renewed every 1–2 y; untreated logs last 10–20 y | 11-07 §2.4 | Historical; analogue |
| Geometry priors | Sag/span 0.08–0.15; rise/span 0.2–0.5 | 11-07 §2.4 | Design prior |
| Section loss | 10 % of diameter leaves 73 % of bending capacity | 11-07 §5.5 | Model property |
| Route switching | 10–20 % improvement threshold | 01-08 §4 | Parameter to try |
| Interruption tests | 7, 30, 90 days | 08-16 §2.2 | Test settings |

## 3. What to defer, and why

- **Sending one worker, seasonal and circular migration** (05-06 §1.2): there is no wage work between settlements yet.
- **Forced displacement and expulsion by an outside power** (05-06 §1.4): war is M6.
- **Exile.** An exile could become a migrant choosing a destination, but whether a settlement admits them is a polity question (Q10).
- **Built roads, carts, pack animals, boats, ferries and pontoon bridges** (07-08 §5.6; 11-07 §1.5): plan M8.
- **Flood damage to bridges** (11-07 §1.6): it waits for M6's floods. Supports keep the foundation data it needs.
- **Tolls, endowments and bridge institutions beyond the polity's store** (11-07 §4): M7.
- **Integration as separate dimensions** (05-06 §1.5): 05-06 §5.5 omits language learning at first.
- **Aggregate settlements and focus-dependent detail:** plan M9.

## 4. Risks, and how to show it works

| Failure | Report | Test or check |
|---|---|---|
| People appear or vanish | 01-03 §4.5; 05-06 §1.1 | Each person is in one settlement or in transit. Per settlement, B − D + I − O = ΔN. Internal net moves sum to the off-map net. |
| Telepathic destinations | 05-06 §5.1; 10-01 §5.3 | A household with no tie, visit or news of a settlement never chooses it |
| Random coalitions | 10-01 §1.5, §4 | Coalitions' kin and tie density beats random groups of the same size |
| The poorest always move | 05-06 §4 | A household under more pressure can stay while a better-off one leaves; provisions given unlock a move |
| One settlement absorbs all | 05-06 §5.3 | Arrivals raise land and housing pressure, and later movers choose differently |
| Order picks the best land | 10-01 §5.5 | Shuffling the founding groups' order leaves the joint choice unchanged |
| A stale route crosses a fallen bridge | 01-08 §7, §8 | A bridge fails under a cached route; a new bridge shortens cached routes |
| A shared crossing is a hidden single point | 08-16 §4; 10-02 §5 | Remove a bridge for 7, 30 and 90 days; supply and moves shift through prices and choices |
| Bridges kept automatically | 11-07 §4.2, §5.7 | Unfinished and unkept crossings occur; upkeep is chosen |
| Span or decay too forgiving | 11-07 §1.3, §5.5 | No beam spans 15 m; the section-loss test as in ADR-0009 |
| Detail depends on the camera | 01-03 §4.1, §4.5 | Gate A, following different settlements, stays byte-identical |
| Cost grows world-wide | Plan §9, Slice X | Three settlements of 1,000 cost about three times one |

**Three risks particular to this codebase.**
- **Leaving still beats everything.** All settlements share the climate factor, and asking evens out stores, so households run out together (plan §9 NUDGEs). With a free, safe exit off the map, one lean year may empty every settlement. What leaving the map promises decides this (Q2).
- **Demo pressure.** The demo (two settlements trade, prices converge, one copies the other's roof, a treaty fails) needs only founding at setup. Splinter founding is rare at village sizes (05-06 §1.3's arithmetic), so do not tune toward it. After a week without a split, nudge a prior and log a NUDGE.
- **Frame size.** ADR-0003's trigger (per-person snapshot entries past about 64 KB, roughly 1,500 people) is passed at 3–8k, so deltas are due.

**Dashboard rows** (plan §4.7), and Gate B:
- **Graded:** population accounting, as a hard check; moves per 100 residents a year, amber outside 0.5–10 (05-06 §5.4) and never red.
- **Reported:** turnover beside net change (05-06 §4); foundings, failures, abandonments and parent–daughter walking times (10-01 §4); crossings built, failed and days closed; seconds per simulated year a world; settlement sizes and P₁₂ (10-02 §2.7, see Q8).
- **Gate B** gains moves and foundings, with wide tolerances, since both are rare.

## 5. Open questions for the designer

1. **Polity.** May a hamlet stay in its parent's polity (10-01 §5.1 allows two settlements under one government), or does every daughter found its own?
2. **Leaving the map.** What does it promise? 05-06 §1.1 wants an explicit reservoir, and 05-06 §1.4 warns that escape is not survival.
3. **Reviews.** 05-06's six months, or 10-02's once in five to twenty years?
4. **Roads.** Does M5 build anything besides crossings?
5. **World size.** Should the new-world dialog offer worlds of thousands, given the NUDGE that big foundings die in their first bad year?
6. **Setup groups.** One people, or strangers?
7. **Budget.** What is M5's "within budget"? I propose 10 minutes a year for 3,000 people.
8. **Settlement-size row.** 10-01 §4 and 10-02 §5 warn against fitting distributions to few towns. Grade it, or keep it grey until M9?
9. **Climate.** Should a year's climate factor vary across a 16 km map, or is a shared lean year right for so small a region?
10. **Exile.** Does an exile become a migrant who can settle elsewhere?
11. **The wave.** How many households, from where, and do they know the map?
