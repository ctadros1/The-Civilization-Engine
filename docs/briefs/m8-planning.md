# M8 design brief: planning, zoning and land value

**Scope.** Plan §7's M8 ("The growing city (v1)") lists "**Planning as politics**: planning institution, plan templates, zoning, land value". Its demo is "a 30k-person planned capital next to an organic port town, with traffic, in the same world". Plan §5.1 says "Governments decide **whether a planning institution exists**. If it does, it picks a plan template and parameters, zoning, and **building codes**", lists planned layouts ("grid, radial, linear, axial (cardo/decumanus), each with parameters: block size, road hierarchy widths, plaza frequency") and parcel subdivision, and gives people the choices "Which plot to claim or buy" and "how tall (land value, tech)". Plan §5.2 puts "Zoning and building codes" and "Land tenure" in the policy library; §3.4 has a program chosen from "library ∩ known_tech ∩ legal(zoning, codes) ∩ affordable" and says "The kernel owns the road graph, parcels and land use". This brief covers, for 10–50k people in 2–3 (up to 5) settlements (plan §4.3):

- how a plan comes to be proposed and adopted as a law, by people of the world, and what a plan is;
- reservation, land assembly, allotment and phases: what a plan does to ground people hold;
- compliance, encroachment and what a polity may do about it;
- zoning as one kind of rule among others, beside the building rules M6b built;
- land value as something derived from what people pay or would pay, never an index behaviour reads;
- density and height as consequences, and what the observer measures.

Roads, congestion, carts and boats belong to the M8 transport brief; this one names where a plan's streets become road work.

**What exists to build on** (checked in the code):

- **Laws** (`civ-agents/src/polity.rs`). There are 29 `PolicyKind`s and 24 `IssueKind`s. "No issue carries a weight toward a policy (ADR-0013 §5): an issue only makes a move available". A template (`PolicyDef`) names its kind, the issues it `answers`, the parameters a sponsor may put forward, a `question` and how it `bears` on values. A `MoveOption` carries a sponsor's choices (`levy_share`, `nominee`, `sanction`, `hours`, `body`, `site: Option<u32>`, `target`) and the forecasts `own_gain` and `followers_gain`. Laws are versioned and their history is kept (ADR-0013 §3); offices have a `Tenure` (`term_days`, `renewable`, `by_lot`) since M7a.
- **Laws with ground of their own.** The `Polity` record keeps where its works would go, "recorded when the law is proposed": `sites` (`CrossingSite`), `wells` (`WellSite`, with the `rect` its digging takes), `banks` (`BankSite`, with the `envelope` "The ground its trapezoid takes" and its borrow `pit`), `enclosures` (`EnclosureSite`: `centre`, `radius_m`, `gates`, homes `inside`, `labour_h`, `work`) and `claimed` places (`ClaimPlace` over `uses::Place`: `Patch`, `Deposit`, `Source`). Per-household reckonings are kept beside them (`bank_saves`, `enclosure_saves`) with the hours given (`bank_given`): "labour asked and labour given kept apart". Public works ask each household an equal share and "Nobody is made to work"; `PayWork` pays a good per hour from the store (`core:policy/pay_work`, `per_hour` 8 coin).
- **Public works avoid held ground.** A flood bank may lie only on ground free to dig: in M6b's measurement "Most candidate banks crossed fields or plots (1,435 of 1,582 rejected)" (plan §9, "Banks built together", slice BG). Nothing in the kernel today takes, buys or pools ground anyone holds.
- **How a household weighs a law** (`population/polity.rs`, `Forecasts::gain`). Each template's gain is in the units of `store_gain`: the change in the log of a year's food above `subsistence_share` (0.5), times `w_gain` (10) points; `kept_gain` and `levy_cost` turn kcal kept or paid into those units, and `Enclose` turns hours into kcal by each household's `hour_kcal`. Polities review every `review_days` (7).
- **Building rules** (`RuleTemplate`: `RoofsUnburnt { within_m }`, `WallsOfKinds { kinds }`, `LeastGap { gap_m }`; ADR-0022 §12). A rule binds buildings begun once it is in force; "Keeping it is each builder's choice, as a curfew's is, with no sanction in M6b". `core:policy/works_wrought_spacing` asks 3 m and "needs nothing"; a gap rule "costs nothing it reckons (the walk to a farther plot is not counted, a simplification)" (plan §9, M6b slice BH). A template may need a technique the sponsor knows (`RuleDef.needs`).
- **Plots** (`civ-land/src/buildings.rs`). `Plot { id, household, rect, use_, since }`, with `PlotUse` `Dwelling`, `Store` or `Work`: "Ground a household has claimed ... A plot outlives its building." ADR-0004 §3 gave plots a status (provisional, held, abandoned); the struct keeps none. Fields have a `holder: Party` (`Household` or `Settlement`) and a user, and an optional `Lease`; plots have only the household that claimed them.
- **Where people build is a geometric rule, not a choice** (`build.rs`, `found.rs`, `population/life.rs`). A new household's wanted point lies `NEW_HOME_STEP_M` (20 m) farther from the hearth than its parent's, moved to dry ground within the hearth's travel field of `HOME_REACH_SECONDS` (300, "Longest walk from the hearth to a home"). `build::home_site` takes the first plot found in rings of `SEARCH_STEP_M` (2 m) in 16 directions, away from the hearth first, that `plot_clear` passes (clear of fields, plots, pits, heaps and wells by `PLOT_GAP_CM`, 1 m) and that needs no levelling, within `HOME_SHIFT_M` (30 m), or `FIRST_HOME_SHIFT_M` (120 m) for a first home. No alternative is compared and nothing is paid.
- **Fields are chosen by worth** (`farm.rs`). `find_site` samples `site_candidates` (24) cells within `max_walk_minutes` (30) and keeps the one worth most by `site_value`: "grain over the work it takes across a few years ... less for the walk to a working session". It is each household's own use value of ground, in grain per hour, and nobody pays it.
- **Regimes** (`params.rs`, `RegimeDef`; ADR-0007). `LandHolder` (`Breaker`, `Settlement`), `LandUse` (`Holder`, `Need`), `Succession` (`Heir`, `Divided`, `Settlement`), `union_share`, `lease: Option<LeaseRules>` (`holder_share`, `term_years`; `core:regime/household` lets for 0.25 of the grain, a year at a time). The regime "is fixed for the world's life in v0"; no `PolicyKind` changes it. "In v0 a dwelling plot and its hut stay with the household that lives in them, under every regime", and "v0 has no land market" (ADR-0007 §1, §3).
- **The ledger** (`ledger.rs`) has 23 channels. `Rent` is "A tenant's share of a let field's grain to its holder". None pays for a plot, a site or ground taken. Credit's `Pledge` is `Field` (its use) or `Guarantor`: "land is not sold, so no mortgage" (`credit.rs`).
- **Wealth measures** (`wealth.rs`): `held_ha`, `worked_ha`, `let_ha`, `rented_ha`, `goods_h`, `floor_m2`, `roofed_m2`, `storage_kg`; "Derived, recorded once a year, and never an input to behaviour", except house size by wealth (plan §9, slice L).
- **Markets are doors.** A buyer weighs "the offer in its settlement that saves it the most hours of its own work over getting the good itself, the walk there and back ... included" (`population/market.rs`, `find_deal`). Customer access already costs hours; there is no market square or market day (plan §9, the M5b demo).
- **Organizations, privileges, forums** (ADR-0025, ADR-0026). A forum hears the kinds of law its `ForumDef.hears` names; a licence "is personal to the member; transfer and lease wait". Body memberships include `Landholders`, `Organized` and `Privileged`.
- **Ways.** Worn ground per 8 m cell becomes trail, surveyed monthly (`civ-land/src/paths.rs`); M5 built only crossings, and "a kept way changes nothing people notice until carts (M8)" (plan §9, M5 design).
- **Harms a nuisance rule could rest on**: fires worked at home carry a hazard per hour (`fire.rs`, ADR-0022 §3) and each household's midden is a node of contamination (`contagion.rs`, `Node::Midden`). Smoke, smell and noise are not modelled.
- **Scale.** Lived worlds so far are villages: the M7d demo lived three of 40. A bench of three settlements of 1,000 cost 3.25 times one (plan §9, M5a slice AL).

## Where the reports agree

Eight points recur across 10-04, 10-05, 08-19 and 10-08, with support from 08-11 and 10-07. I treat them as invariants.

1. **Planning is coordination by an institution, not a flag.** "Treat planning as an institution's ability to coordinate land, infrastructure and building rights—not as a binary 'planned city' flag" (10-04, executive conclusion). One settlement can hold a planned core, surveyed districts and informal outskirts at once (ibid.). Configurations are "available ... not an obligatory progression" (10-04 §4); comprehensive zoning should "emerge as one institutional choice among others" (10-05 §6.3).
2. **A plan is proposed by those who gain.** Evaluate it "from the sponsoring coalition's perspective, not from an omniscient city-wide welfare function" (10-04 §1.1, §6.3); regulation is sought "when its expected gains exceed its political and administrative costs" (10-05 §1.8); rule changes go through the political system (08-11 §5.8).
3. **The ideal, the reservation, the built and the surviving are four things** (10-04, executive conclusion). "A reservation for a street is not a paved street" (10-04 §1.4); a component can be "legally reserved, only partly constructed and partly occupied without permission at the same time" (10-04 §6.1). A road keeps a legal, a constructed and a usable width (10-07 §1.7).
4. **Land assembly is its own problem.** Every component names its rights-holders and "a valid acquisition mechanism" (10-04 §1.2); widening "should not silently delete adjoining buildings" (10-07 §1.7); conversion pays the old use's opportunity cost and any compensation (08-19 §1.2).
5. **Allocation is stored apart from geometry.** "Regular parcels should not automatically imply equal wealth" (10-04 §1.5); "the highest economic bidder need not obtain the parcel" (10-05 §1.1); parcel, structure, rent right and occupancy may belong to different parties, so "Never infer all four from one `owner_id`" (08-19 §1.1).
6. **Law is not self-enforcing.** Expected sanction is detection × adverse ruling × execution × (fine + demolition loss) (10-04 §1.6); "Do not turn every restriction into a hard placement veto" (10-05 §5.3); enactment, awareness, inspection, judgment and execution keep separate dates (10-05 §1.8).
7. **Land value is an outcome of bids.** "Make land value an outcome of competition over locations and permitted uses—not an independent 'desirability score'" (08-19, opening); store the opportunity cost "even where no rent is paid" (10-05 §1.1) without "inventing a cash rent" (08-19 §1.1); "a revaluation is not spendable cash unless an actual sale or financing transaction occurs" (08-19 §5.3).
8. **Density and persistence are consequences.** Density comes from "land use, buildings, and actual occupants—not ... a maximum population density to each technological era" (10-08, executive finding; §1.2's identity). Persistence follows from the cost of replacing what is built, "not timers" (10-04 §1.7); regime replacement "must not automatically regenerate the street graph" (10-04 §5.1).

**What games and models teach to avoid:** player-painted development areas, "not equivalent to endogenous legislation" (10-05 §5.5, on Foundation); a procedural road-and-lot generator as the model of the decision, when it belongs "downstream of institutions" (10-04 §6.7, on Parish & Müller); Cities: Skylines II's land-value formula and its removed landlord (08-19 §5.4); Schelling as an explanation of a quarter (10-05 §1.7); "one plaza every X meters" (10-04 §3.3); a pooled "block size" distribution (10-04 §3.1); a global foot (10-04 §1.3); a fitted ring radius (10-05 §2.2); one "pollution radius" (10-05 §5.2); era caps on storeys (10-08 §3.1); a 30 % housing ceiling (08-19 §1.4); a "planning inefficiency" penalty (08-11, executive recommendation); a paving threshold by traffic (10-07 §1.6).

**Where they differ, or differ from the plan:**

- **Whether a planning institution exists.** Plan §5.1 makes it a government's yes or no, and the institution "picks ... building codes". M6b already passes building codes with no planning institution (ADR-0022 §12; the M6 fire brief). 10-04 §6.1 lists what an institution needs (members, offices, jurisdiction, procedure, treasury, labour, survey knowledge, acquisition powers, inspection and judging), and the polity already has most of it. I treat "planning institution" as a derived label, as m7-money treats "bank".
- **Templates.** Plan §5.1's grid, radial, linear and axial each with "plaza frequency" fit 10-04 §6.2's composable operations only if a template is a composition of operations; 10-04 §3.3 rejects a plaza interval, and §3.1 a single block size.
- **Height from land value.** Plan §5.1 has "how tall (land value, tech)". 08-19's opening forbids the chain "expensive land → rich residents → tall buildings", and §1.7 derives height from marginal profit and feasibility; 10-08 §3.1 adds stair, carrying and circulation costs. In TCE nobody builds to let, so height can only follow a household's own space against the plot it can get.
- **Buying plots.** Plan §5.1 has people "claim or buy". ADR-0007 has no land market, and its revisit trigger is "Land sales, credit or mortgages arrive". 08-19 §5.3's build order puts rights and rental before investment and finance.
- **Zoning now or later.** Plan §7 lists zoning for M8. 10-05 §5.6 puts "Modern zoning" (map-based use and dimensional districts) "post-v1 without replacing the core model", and 10-05 §6.3 asks first for "access, productive opportunity, mixed occupancy, tenure, and nuisance justice".
- **Cadences.** Institutional review every 30–90 days (10-04 §3.4); premises reviews 30–90 days (10-05 §2.2); asking rents every 1–4 weeks (08-19 §2.2); a 30–90-day chronic-problem window for roads (10-07 §2.4). Candidates per search: 16–64 premises (10-05 §2.2), 10–30 dwellings (08-19 §2.2), 3–8 layouts a proposal (10-04 §3.4). All are design priors.
- **The demo names an outcome.** "A planned capital" is a result; 10-04's executive conclusion asks for "bounded templates and test cases, not a predetermined sequence". And a 30k settlement must stay at human scale: "simulate a smaller settlement or a smaller implemented portion of an ambitious plan" (10-04 §6.5; 10-05 §5.4).
- **Economic planning.** Plan §5.3's "quotas and planning by an office" is 08-11 §1.6's planning of production, not of ground. It is out of this brief.

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| A plan | A law of an appended kind; its version: template, origin, orientation, extent, the realised layout (corridors, public sites, parcels), allocation rule, phases | A sponsor's proposal answering an issue; each stance; the custom's decision |
| A reservation | Per corridor and public site: legal width and ground; separately, what is built and what is occupied | The plan in force; works proposed later |
| Land assembly | Per touched claim: the field or plot, its holder, the clause offered, the answer (consent, refusal, taken) | The plan's clause; each holder's choice |
| An allotment | Per parcel: phase, allottee, rule, day, any payment and its channel, build-by day | The keeper of the plan by the rule; households asking or offering |
| An encroachment | A building or plot on reserved ground: when, by whom, seen by whom, case, remedy | A builder's choice; a sighting; a case |
| A use rule (zoning) | Per district of a plan, the `PlotUse`s it allows; nuisance rules as appended `RuleTemplate`s | Laws, kept by choice |
| Where a household builds | Its plot, and the receipt of its choice: candidates, hours each saves, the chosen | The household's own reckoning |

**Derived, not saved:** "planned", "organic", "grid" and "planning institution" as labels; the land-value map and its gradients; every density (10-08 §1.1); the street land fraction; the share of a plan built, occupied and kept; each household's forecast of a plan.

**Size (my arithmetic).** 30,000 people at five a household is 6,000 households. A plan releasing 10–50 parcels a phase (10-04 §3.4) holds a few hundred parcels after years; at an assumed 48 B a parcel (a rectangle, allottee, rule, days, payment) that is tens of kilobytes. The cost is in choosing, not storing: §4.

### 1.1 What a plan is

**Reports.**

- Composable operations: orthogonal subdivision, axial connection, radial branching, enclosure and ward subdivision, plaza-centred subdivision, courtyard aggregation, boundary extension (10-04 §6.2). Cultural knowledge shapes "proportions, orientation preferences, acceptable public/private arrangements" and "should not predetermine the outcome" (ibid.).
- A plan version keeps sponsor, adoption date, extent, orientation, street reservations, public sites, subdivision and allocation rules, phases and amendments; "Keep plan versions after amendments" (10-04 §6.1).
- Streets cost ground: f_street = 1 − BxBy/((Bx + w)(By + w)); for 40 × 80 m blocks, 6 m streets take 19.1 % and 12 m take 33.1 % "before plazas or other public reservations" (10-04 §5.6). Keep the source unit; "Do not give every culture a global 'foot'" (10-04 §1.3, §3.2).
- Laws of the Indies: a central plaza inland, near the landing on a coast, and room for growth (10-04 §2.5). Camps, towns and field divisions are three systems (10-04 §2.2).

**My proposal.**

- **A plan is a law of an appended kind, `Plan`.** Its template is content: a composition of operations with menus a sponsor chooses from (block and plot modules in a local unit, corridor widths, an orientation rule such as toward the hearth, the river or a cardinal line, public sites by kind and size, an allocation rule, a phase size). 10-04 §3.4's 40 × 80 m blocks with 6 m streets is a neutral test fixture, not a cultural preset.
- **The layout is laid by a pure function and saved as realised.** The sponsor's proposal expands its template over the ground as it is: terrain, water, slopes too steep, and every claim it would touch. What is saved is the realised layout, as `BuildingSpec` saves a realised design (ADR-0004 §2), so a plan does not change when claims or terrain change later.
- **3–8 alternatives a proposal** (10-04 §3.4): origins, orientations and extents the template allows. The sponsor forecasts each as households will (§1.2) and puts forward the best for its own household and followers, as `MoveOption` does.
- **A plan may need knowledge.** As `RuleDef.needs` does, a template can name a technique the sponsor must know (laying out by cord and stake), spread by people (ADR-0008; 10-04 §4: planning knowledge "should diffuse through specialists, texts and political imitation").
- **Versions are kept.** An amendment is a new version by the custom's procedure; a repeal ends the reservations and leaves what was built (10-04 §5.1).

### 1.2 Who proposes, and why

**Reports.**

- ΔU_f(p) = access, revenue, security and prestige benefits − land losses, taxes, labour obligations and disruption, per faction; forecasts imperfect, so "oversized reservations, delayed phases and unfinished public projects" come "without scripted failure events" (10-04 §6.3).
- Bastides were "principally market-town investments": founders exchanged land and privileges "for future rents and market revenue" (10-04 §1.1, §2.6). Early farming settlements coordinated through kin groups "without a city-wide plan" (10-04 §4).
- No universal plan-adoption probability is established (10-04 §3.4).

**My proposal.**

- **Appended issues, each what members saw, weighing toward nothing:** *no ground*, when a household that would build found no clear ground within its reach or built farther than it wanted (a `home_site` that fell to `FIRST_HOME_SHIFT_M` or failed); *a way built over*, when a building begun on a trail lengthened the walks of those who used it; *new ground*, when the settlement was founded lately or households build beyond its built edge (Laws of the Indies' moment: nothing to assemble). The existing `burned` may also open a plan move by authoring, as `works_wrought_walls` answers both `burned` and `gave_way`. (The existing `founding` issue is a body that took the deciding, not a new settlement.)
- **Each household forecasts a plan in hours, turned into kcal by its `hour_kcal`**, as `Enclose` is: the walking it expects to save on an allotted plot over its next-best site (§1.6), if it means to build within the horizon; fire losses it believes averted by the plan's spacing (the rule forecasts' terms); payments it expects in or out (ground rent, fees, compensation); less field ground lost under reservations (its own `site_value` of that ground), its share of the survey and clearing work, and longer walks where a corridor blocks a way it uses.
- **Growth is what people have seen.** A sponsor sizes a plan from the arrivals and new households its members remember, so plans overshoot and undershoot by belief (10-04 §6.3). The polity's revenue from fees enters as a share of the store, as an annexed store does (`annex_share`).
- **Plans are reviewed on the polity's cadence,** but a plan move is weighed only when one of its issues is held, and at most every 30–90 days (10-04 §3.4).

### 1.3 Reservation before construction

**Reports.**

- Keep "legal reservation, ground clearance, drainage, road surface, installed services and adjacent occupancy" apart; release phases "when finance, labor and demand permit" (10-04 §1.4). Unbuilt portions stay "vulnerable to cancellation and occupation" (ibid.).
- A road's standard "changes only when someone can organize the work" (10-07 §1.5); upgrades "require a sponsor, not just traffic" (10-07 §1.6). More roads mean more upkeep; unfunded ones gather a backlog (10-07 §1.8, §4 G).
- A plaza is "a polygon with occupiable space", not a junction (10-07 §1.9).

**My proposal.**

- **In force, a plan reserves ground.** A builder who keeps the plan treats reserved corridors and public sites as not clear (`plot_clear` gains the plan's ground for keepers), as a keeper of a gap rule keeps its distance today. Buildings that stand stay (ADR-0022 §12; 11-09 §1.7 as the M6 fire brief reads it).
- **Nothing is built by adoption.** Clearing a corridor, digging its drains or setting its surface is a public work proposed later, asked in equal shares or paid by `PayWork`, as crossings, wells and banks are. The transport brief owns what a street is once built; this brief keeps its legal width apart from its built and usable widths (10-07 §1.7).
- **A public site is reserved ground with a named use:** a square at the hearth or landing, ground for a well, a workshop row. It is empty until someone builds or gathers there.

### 1.4 Land assembly

**Reports.**

- Acquisition mechanisms: "purchase, negotiated pooling, donation, customary allocation, conquest, confiscation or compulsory acquisition"; unresolved claims "delay construction, distort the alignment or provoke resistance" (10-04 §1.2).
- "Political seizure can override consent, but should still impose physical costs and political consequences" (08-19 §1.2). Clearance is not "an automatic welfare improvement" (08-19 §1.8, Bombay).
- A household weighs "accepting an allotment, refusing relocation" (10-04 §6.4).

**My proposal.**

- **A plan lists every claim it touches** when proposed: fields by holder and user, plots by household, wells, pits. Its template names one clause from a content menu for each kind of claim:
  - *around*: the layout bends round held ground, which is what every public work does now (§ "What exists");
  - *at the review*: under `LandHolder::Settlement` and `LandUse::Need`, the settlement gives no field on reserved ground at its yearly review (customary allocation, 08-09 §5.2 as ADR-0007 cites it);
  - *land for land*: a holder is offered vacant ground of like area within the same walk, and accepts or refuses by its own `site_value`s;
  - *paid*: the store offers goods the law names per hectare or per plot, and the holder accepts or refuses;
  - *taken*: the ground is reserved without consent once its crop is in; its holder holds a grievance against the gathering (`Blamed`), as any overruled member may.
- **Refusal bends the plan.** A component whose claims refuse is not reserved; the observer shows the gap. A plot with a standing home is never cleared by a plan; only a later law could ask that, and M8 does not offer one (§3).
- **Every payment is on the ledger** on an appended channel, so compensation never creates goods.

### 1.5 Allotment, phases and obligations to build

**Reports.**

- "Store the allocation system independently of geometry. Record owner, occupier, household membership, transfer restrictions, subdivision rights and obligations to build" (10-04 §1.5).
- Initial release 10–50 plots (10-04 §3.4). Bastides succeed or fail by "Commercial attraction and vacant-lot uptake" (10-04 §2.6). Villeréal's plots were realised at 62.5 % of their prescribed area (10-04 §5.3): permit "systematic implementation changes".
- "Queues and eligibility can partly replace highest-bid allocation" (08-19 §1.5, cooperative housing row). Identical allotments should support unequal wealth later (10-04 §5.4).

**My proposal.**

- **The plan names a keeper**, an office with a `Tenure` as M7a's offices have, who sets out stakes (hours of work, recorded), releases phases and allots parcels by the plan's rule. A keeper may favour by regard and may be paid on `Channel::Bribe`, through ADR-0026 §4's existing terms; nothing new is needed to make the office corruptible.
- **Allocation rules are content** (a menu a sponsor chooses from): *by need* (households short of a home, nearest first, as `LandUse::Need` gives fields); *by lot* (a keyed draw among those who ask); *by offer* (to whoever offers most of a good the law names, paid once into the store or as a yearly ground rent; §1.6); *by standing* (the keeper's choice). Phases open when the last is mostly taken, at the keeper's review.
- **A household chooses** to ask for a parcel, take it, or build where it would have built. An allotted parcel is a candidate in its site choice (§1.6), with its fee and its security under the law.
- **An obligation to build** is an optional clause: unbuilt by its day, the parcel returns to the keeper. Subdivision and merging of parcels wait (§3).
- **Plots become a bundle.** A parcel records its allottee and the rule it was given under; who may lease or pass it on is the regime's question (open question 2). Today's rule, that "a dwelling plot and its hut stay with the household that lives in them", stays the default.

### 1.6 Where a household builds, and what it would pay

**Reports.**

- Bid rent falls with money-equivalent travel cost: ∂b/∂d = −T′(d)/h, on "network travel time, not a fixed distance-to-city-center multiplier" (08-19 §1.3). Households trade "field travel and access to exchange, labour partners, water, and kin" (10-08 §3).
- Commerce needs customers or goods moving through, with customer, freight and institutional access apart (10-05 §1.2). Businesses bid "the additional profit obtainable there" (08-19 §1.3).
- Sample a limited set of premises (10-05 §2.2: 16–64; 08-19 §2.2: 10–30); an attractive plot "without secure possession is not equivalent to the same plot with an enforceable title" (10-04 §6.4).
- Keep asking, contract and expected rent, and collections, apart (08-19 §1.4); "Land value is measurement-dependent" (08-19 §6).

**What the code implies (checked above).** A household never compares two places to build: the ring search takes the first clear, level plot. So no household can say what one plot is worth to it over another, and nothing exists from which a land value could be derived. Fields, by contrast, are already chosen by worth.

**My proposal.**

- **Site choice becomes a choice among candidates.** A household about to build samples candidates (clear plots within its reach, allotted parcels it could ask for, parcels on offer) and scores each in hours a year: its members' walks to its fields, the hearth, its kin's homes, the water it draws and the doors it buys at, by the walking times the kernel already plans with; less levelling, any fee or ground rent, and an expected sanction where the plot breaks a law it knows (§1.7). It takes the best. Its receipt keeps each candidate's hours.
- **What it would pay is that receipt.** The hours the chosen plot saves over the next best are the household's own reservation value of it (the compensating differential of 08-19 §1.3). It is recorded with the decision, never written on the ground, and read by no other household.
- **What people pay is the ledger.** Under an allocation *by offer*, households who want a parcel offer up to their reservation value in the named good, and the payment moves on an appended channel. Ground rent is a yearly obligation (ADR-0015's record) to the store. Leases and sales, if a regime allows them (open question 2), add their own payments.
- **Illustration (my arithmetic; the trips are an assumption).** A plot 200 m nearer where a household's people go four times a day saves 4 × 400 m at 1 m/s (10-04 §3.4's walking convention), 27 minutes a day or 162 hours a year. A hut costs 450 to 1,050 hours (plan §9, slice L's NUDGE). Nearness is worth something at village scale already, before any town.
- **Later, comparables.** A household may hear what others paid for a parcel, as price reports are heard (ADR-0019), and use it as a ceiling or floor when it offers. That is people's knowledge, kept per household, not an index.

### 1.7 Compliance, encroachment and remedies

**Reports.**

- Tang authorities "repeatedly prohibited private openings through ward walls"; "The repetition is evidence that prescription and practice differed" (10-04 §1.6). Inspection intensity priors: 0.02, 0.2 and 1 per parcel-year (10-04 §3.4).
- Distinguish "unauthorized occupation, tolerated occupation and legally regularized occupation" (10-04 §5.5); Kaifeng charged rent for some infringements (10-04 §1.7).
- A rule is "Jurisdiction + protected interest + prohibited action or required condition + who may complain + remedy + enforcement procedure"; remedies besides relocation; precedence protects an established use (10-05 §1.6). London orders allowed 40 days to correct, "a formal deadline, not proof of actual compliance" (10-05 §1.6, §2.1). Backlogs should be observable (10-05 §4, "Formal rules and realized outcomes diverge").

**My proposal.**

- **Keeping a plan is a builder's choice,** weighed as a building rule's is (norm, stance, regard for the sponsor), plus 10-04 §1.6's expected sanction where the plan names one: the chance believed that someone sees (neighbours and the keeper on their rounds), that a hearing finds against it, and that the remedy is carried out, times its cost.
- **An encroachment is recorded and known by sight.** A case is brought under the plan law, heard by the gathering or by a forum whose `hears` names `Plan` (ADR-0025 §1).
- **Remedies are the template's bundle:** a fine in food to the store; removal of the building, as BD's crews take buildings down (plan §9, "BD crew demolition"); or regularisation for a yearly payment. Tolerance is no case brought. An amendment may regularise a whole row.
- **Precedence:** a use older than the plan is never an encroachment (10-05 §1.6; ADR-0022 §12's old stock).

### 1.8 Zoning: use rules and nuisance rules

**Reports.**

- Keep four things apart: what occupies a site, what is done there, who holds rights, what institutions permit (10-05, main recommendation). Mixed use "available by default, not mandatory everywhere" (10-05 §1.3).
- Clustering has three causes: resources, commercial and social ties, compulsory location; benefits must saturate (10-05 §1.4). "Dirty industry goes outside the walls" is not a rule (10-05 §1.5).
- Modern zoning adds permitted activities, intensity, height, setbacks and coverage, and keeps FAR and coverage apart, with nonconforming uses, exceptions and appeals (10-05 §5.6).

**My proposal.**

- **A plan's districts may carry the uses they allow,** from `PlotUse` (`Dwelling`, `Store`, `Work`). A keeper builds only those there; an older use stays. Zoning is a clause of a plan, so a settlement without one has none.
- **Nuisance rules are appended `RuleTemplate`s over harms the kernel models:** a building where fire is worked keeps a distance from others' homes (`fire.rs`'s hazard of fires worked at home); a midden keeps a distance from a well (`Node::Midden`). Each answers what people saw (`burned`, `sickness`), as M6b's rules do. Smoke, smell and noise wait until something models them (10-05 §1.5's channels).
- **Dimensional districts wait** (coverage, storeys, setbacks; §3), as 10-05 §5.6 suggests. The gap rule is the one dimensional rule M8 starts with.

### 1.9 Density and height follow

**Reports.**

- D_g = 10,000 s c h e o / a (10-08 §1.2); with s = 0.5, c = 0.5, h = 2, e = 0.8, o = 0.9, a = 15 it gives 240 persons/ha. Envelopes (low confidence): open farming village 15–50 persons/ha, compact village or small town 50–200, dense low-rise 150–400; "Do not make settlements converge toward the middle of these bands" (10-08 §6.2).
- Height carries structural, stair, carrying and circulation costs, and depends on the construction system (10-08 §3.1). Crowding can rise with no building (10-08 §7).

**My proposal.**

- **Height is a household's choice** between a larger footprint on the plot it can get and another storey where grammar v3 allows one, each costed in hours by the grammar's stages and the carrying it adds. A plot that is dear, scarce or allotted small makes the storey worth more. No rule reads density or land value.
- **Density is measured,** by 10-08 §1.1's quantities, never set.

## 2. Plot guards

- **No authored plan.** Content supplies operations, menus and clauses. No settlement starts planned; no culture, region or founding custom carries a template.
- **No issue weighs toward a plan** (ADR-0013 §5). "No ground" opens a move; it is never a cause.
- **No land-value index.** Behaviour reads only a household's own reckoning and the payments it made, saw or heard. The observer's map is drawn from the ledger and from receipts and is read by nothing.
- **No threshold.** No population, era, density or wealth opens planning, zoning or storeys (10-04 §3.4: no adoption probability is established).
- **No instant city.** Adoption reserves; it builds nothing, clears nothing and moves nobody. Repeal regenerates nothing (10-04 §5.1).
- **No silent taking.** Every touched claim has a clause, an answer and, for a payment, a ledger entry.
- **The demo's outcome is one of several.** A world may never plan, may plan and leave the plan empty, or may build half of it. If no plan passes after a time-boxed week, nudge only priors (the sponsor's horizon, the candidates sampled), log a `NUDGE:` (plan §1, rule 3), and say so.

## 3. What to defer, and why

| Defer | To | Why |
|---|---|---|
| Sales of land and mortgages | An ADR on land transfer; possibly post-v1 | ADR-0007 has no land market; 08-19 §5.3 puts rights and rental first; 08-19 §1.5: title does not unlock credit |
| Lodging, rooms let, subletting | With the M8 housing work on where to live | 08-19 §1.4; 10-08 §6.1; nobody lives under another's roof today |
| Coverage, storey and height districts | Post-v1 | 10-05 §5.6 |
| Subdividing and merging parcels | After allotments live | 10-04 §5.4 needs generations |
| Clearance of standing homes for a plan | Not in M8 | 08-19 §1.8; 10-07 §1.7 |
| Survey error and local units of measure | After plans pass in lived worlds | 10-04 §1.3, §3.2; §6.6: "Detailed survey instruments ... can wait" |
| Speculation and building cycles | With land sales | 08-19 §1.9 needs prices, credit and delay |
| Camps laid by force composition | With war's bands, if wanted | 10-04 §2.2 |
| A planning god tool | Never | It would choose an outcome |

## 4. Risks and validation

**Changing where everyone builds.** §1.6 replaces a geometric rule every world uses, so every world's homes move, and the smoke and dashboard must be rerun. Worlds with no plan will not give the build-before's digests once site choice lands; every later slice must.

**Footprint at scale (my arithmetic).** At 1 m/s, `HOME_REACH_SECONDS` (300) reaches 300 m, about 28 ha, or 2,800 people at 100 persons/ha. A 30,000-person settlement at 50–200 persons/ha needs 150–600 ha, a disc of 0.7–1.4 km radius (10-08 §1.2's table: 10,000 at 100 persons/ha is 1 km²). The reach must give way to a household's own walk reckoning, or no plan has room. "Do not shrink a thousand-square-kilometer settlement landscape into a tiny map while keeping ordinary walking speeds" (10-05 §5.4).

**Nobody plans.** If households build near the hearth and ground stays clear, *no ground* never arises, and plans are proposed only at foundings. That is plausible for villages (10-04 §4, early farming row); M8's growth must make ground scarce by itself.

**Compute.** Site choice runs only for a household about to build: at 6,000 households, a few hundred a year a settlement, each 16–64 candidates (10-05 §2.2) on walking fields the kernel already keeps. A plan proposal forecasts 3–8 layouts for households that hold touched claims or mean to build, not every household. Recompute only touched parcels and walking regions after an amendment or a work (10-04 §6.5).

**Stylized facts, as plausibility checks:**

- Rings only in the controlled case: with residuals 100 − 20d and 60 − 5d, the field boundary falls at 2.67 km; a river, a second market or uneven soil must break it (10-05 §1.1, §4).
- Reckoned bids fall with walking time to one destination; a second centre makes them less monocentric (08-19 §4).
- A plan reserved is not a plan built: plans stay partly empty, partly occupied without leave (10-04 §1.4, §6.1).
- Realised parcels can differ systematically from prescribed ones (10-04 §5.3).
- Streets persist after the polity that laid them ends or is annexed; later building aligns to them (10-04 §5.1).
- A successful planned core grows extensions that inherit some alignments and not others (10-04 §5.2).
- Identical allotments come to support unequal wealth (10-04 §5.4).
- Unauthorized, tolerated and regularized occupation look alike on the map and differ in the record (10-04 §5.5).
- Removing a use rule leaves some clusters standing (10-05 §4, "Occupational clustering does not prove legal segregation").
- Central premiums vary widely: no fixed gradient (08-19 §2.1, Albouy's 1.3–35.1; Toledo's median central rents about 50 % higher, direction only).
- Density and height diverge; crowding changes without construction; after decline, buildings and parcels persist (10-08 §7).

**Observer.** A plan's page shows its versions, sponsor, stances, the claims it touched and their answers, phases and allotments, and encroachments with their cases. The map overlays legal reservations, built ground and occupied ground as separate layers (10-04 §6.1). A plot's page says why it was chosen ("over 23 others: 40 minutes a day nearer its fields and its daughter's home") and what was paid for it. Land value is shown only as what it is: "paid" (with the payment), "reckoned" (a household's receipt) or "nothing recorded" (08-19 §6). Densities are shown with their denominators (10-08 §1.1).

## Suggested slices

1. **Where a household builds, as a choice.** Candidate plots scored in hours, the receipt kept, the walk-based reckoning of §1.6; the reach of `HOME_REACH_SECONDS` replaced by the household's own. Tests: the controlled single-destination world gives falling reckoned values; a saved world loads and lives on exactly. Observer: a plot's "why here" and a "reckoned" overlay labelled as such. Smoke and dashboard rerun.
2. **The plan law and its reservations.** `Plan` with content templates of composed operations, 3–8 layouts a proposal, forecasts, the appended issues, the realised layout saved with versions, keepers avoiding reserved ground, encroachments recorded. Tests: a passed plan reserves and builds nothing; a repeal leaves what stands; worlds with no plan give the previous slice's digests. Observer: reservation layers and the plan's page.
3. **Land assembly and allotment.** Touched claims with clauses and answers, compensation on an appended channel, the keeper's office, phases, allocation rules including *by offer*, ground rent as an obligation, the build-by clause. Tests: a refusal bends the plan; *by offer* records payments; an unattractive phase stays vacant. Observer: parcel pages, payments and vacancies.
4. **Compliance and cases.** Expected sanction in builders' choices, sightings, cases under a plan, forums that hear them, remedies (fine, removal, regularisation). Tests: a seen encroachment becomes a case only if brought; an unseen one changes nobody's choice. Observer: violations, complaints, findings and overdue remedies as a backlog (10-05 §4).
5. **Use and nuisance rules.** District uses on a plan; the two nuisance templates; older uses kept. Tests: disabling the rule leaves home work possible (10-05 §4, "Mixed use survives at finer spatial scales"). Observer: a district's uses and its exceptions.
6. **Land value, density and the demo's footing.** The derived map (paid, reckoned, nothing), densities by denominator, gradients fitted after growth (10-08 §5.1), the dashboard rows with no targets; a bench of the largest settlement M8 lives. Tests: no behaviour reads the map (a test that blanks it and checks digests).

## Open questions for the designer

1. **One kind or several.** One `Plan` kind whose templates compose operations, as proposed, or a kind per operation (reserve a corridor, subdivide, extend)?
2. **Transfer of plots.** May a regime allow a parcel to be leased or sold in M8? ADR-0007 would need its revisit; without it, land value shows only in fees, ground rent and compensation.
3. **Taking.** Is *taken* (reservation without consent, after the crop) vocabulary M8 should offer, or only *around*, *at the review*, *land for land* and *paid*?
4. **Site choice for everyone.** Accept that slice 1 changes every world, as m7-money's question 7 did for money's worth?
5. **The reach.** What replaces `HOME_REACH_SECONDS`: the household's own walking reckoning alone, or a plan's extent where one is in force?
6. **The keeper.** A new office per plan, or the store's keeper's power extended?
7. **Removal.** Is taking down an encroaching building a remedy a gathering may choose in M8?
8. **Zoning's scope.** District uses and two nuisance rules, with dimensional districts after v1: is that enough for plan §7's "zoning"?
9. **Knowledge.** Should a plan need a technique, and do any founders bring it?
10. **The demo.** A planned capital must come from lived worlds picked by what they did, as the M5b demo picked its worlds by their founding draw. Is a setup choice that founds a settlement under a plan acceptable vocabulary, or plot?
11. **Measures.** Which densities, payments and plan shares go on the dashboard, and with what tolerances?

## What an ADR would need to decide

- **The plan as a law with saved ground:** the `Plan` kind, the realised layout as authoritative state (as `BuildingSpec` is), versions, reservations, the save sections and their migration, and the boundary payloads for the observer and Unreal (plan §3.4).
- **Plots as a bundle of rights:** allottee, rule given under, and who may lease or pass them on, amending ADR-0007 §1's dwelling-plot rule and ADR-0004 §3's plot status; whether land is ever sold.
- **Land assembly and payment:** the clause menu, consent and refusal, appended ledger channels for fees, ground rent and compensation, and ground rent as an ADR-0015 obligation.
- **Site choice as a decision:** candidates, the receipt, the replaced reach, and the rule that land value is derived from payments and receipts and read by no behaviour.
