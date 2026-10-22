# M10 design brief: mechanical power, fuel, iron at scale and the works

**Scope.** Plan §7's M10 ("Industrial content") lists "steam, factories, rail, gas light; capability rungs for industrial services; healthcare and education as full services; fiat money". Plan §5.6 says a technique enters only "together with its content footprint" and "There are no era gates"; plan §1 kept industrial content out of v1. This brief covers:

- power as a local service: water wheels first, then engines that burn fuel;
- fuel as goods (wood, charcoal, coal, coke), with heat and fitness for a process kept apart;
- iron and steel made in quantities a works can sell;
- the works: a firm with a building, prime movers, machines, shifts and hired hands;
- the preconditions of industrialisation, and how each arises from what people do, not a date.

I assume sibling M10 briefs own rail, gas light, the services' capability rungs (health and education) and fiat money. This brief needs two things from them: rail, because delivered fuel decides where an engine pays (07-12 §3.2); and gas light, because a shift after dark needs light.

Reports read: **07-06** (energy and power), **07-12** (preconditions and the Industrial Revolution), **07-05** (materials and metallurgy), **08-08** (firms), **08-10** (labour regimes), **08-03** (craft production), with **07-03** (the technology graph) and **07-01** (invention) skimmed for how techniques and discovery are modelled.

**What exists to build on** (checked in the code):

- **Recipes are arithmetic on stores** (`civ-agents/src/make.rs`, ADR-0006 §2): inputs and outputs a unit, session inputs (an oven's firewood), `unit_h`, `session_h`, `max_units`, tools worn by hours (`make::wear`), and a skill. There is no power term and no elapsed time apart from labour. A **fixed tool** (`[tool] fixed = true`: only `good/kiln.toml` and `good/oven.toml`) is one number in a household's stores, out of use until whole (`make::in_use`), never sold (`population/market.rs:141`) and never carried off. It has no place of its own.
- **Metallurgy stops at copper.** `technique/copper_smelting` (found from burning lime and firing bricks, E50 100,000 h, founders lack it), `recipe/smelt_copper` (20 kg of ore and 40 kg of firewood a kilogram; its comment: "firewood here stands for the charcoal made from it") and `recipe/cast_copper_axe`, whose axe serves for the stone one (M7d slice CE's `serves`). There is no charcoal, iron, coal or coke good, and no metalworking skill: copper is worked under `skill/pottery`, one of seven skills.
- **Fuel is burned by weight.** A household burns `fuel_kg_per_person_day` (0.7–1.5 kg by month, `people/early_farmers.toml`) from every good with `purpose = "fuel"`, in catalogue order (`Household::stores_at_time`, `person.rs` about line 1166). A second fuel good would be burned at the hearth kilogram for kilogram with firewood, whatever its heat.
- **Deposits** are content rules placed once from the seed (`civ-land/src/deposits.rs`, ADR-0010 §1): clay, stone, toolstone and copper ore (`land/temperate_valley.toml`), each with cover, thickness, quality, density and dig hours, dug as pits (`population/digging.rs`). A pit never meets ground water; only wells read the water column (ADR-0021 §3).
- **Water and wind are daily quantities.** `Land::reach_discharges` is today's discharge per reach in m³/s (`civ-land/src/lib.rs:425`); `RiverReach` keeps mean discharge, width and drainage area (`civ-world/src/lib.rs:119`); the bed is saved whole. An irrigation ditch takes at most `max_share` of its reach's flow through an intake and debits every reach below once (`civ-land/src/ditches.rs`). The weather draws each day's wind as a vector about the land profile's mean and spread (`civ-land/src/weather.rs`: parameters at lines 211–219, the day's `wind_ms` at 277).
- **Firms** (`firm.rs:259`, ADR-0006 §5): one household owns each, with stores, books, posted terms and one wage offer. As built, a firm makes one good (`found_firm` opens one a good; `plan_wage` reads only `lines.first()`, `population/firm.rs:464`); it **works with its owners' tools** and wears them (`population/firm.rs:294`), although ADR-0006 §5 gives a firm "its tools and stations"; it hires only in its own settlement, at most `max_hire_hours` (21) a week, in sessions of 60–180 daylight minutes (`activity/hired_work.toml`); a hand need not know the technique when the owners do (ADR-0008 §4); it closes `Idle`, `OwnerGone`, `Failure` or `Barred`. Its building is `building/workshop.toml` (1–4 bays, one storey, 6 m² a worker), which needs `jointed_frame`.
- **Wage work is rare.** Plan §9, "NUDGE: hired work is rare" (slice J): 27 hours hired in four three-year runs, because "People who can farm or gather for themselves do better at home". In the M8 demo no household built a frame (plan §9, slice DE), so none could build a workshop.
- **Discovery** (ADR-0008 §3; `population/knowledge.rs:240`): at each work session's end, a keyed draw for every technique whose `tried_in` lists the activity, a route of it known and its `needs` at home, at 1 − exp(−ln 2 · h / E50). `try` is aimed only by `[answers] spoilage` (`params.rs:643`); ADR-0008 §3 names "hours spent on a task" and "work waiting on a good" as problems that "wait for a technique that needs them". Far techniques come in practice from contact or the observer's `IntroduceTechnique` (`civ-host/src/protocol.rs:64`).
- **Institutions that bear on a works:** `licence_tools`, `staple_tools` and `duty_on_tools` (ADR-0026); `enclose`; plans with rent and allotment by offer (ADR-0030); `pay_work` and `raise_building`, with buildings a polity owns (ADR-0031); the nuisance rule `fire_from_homes`, which already reads "a fixed tool a fuelled recipe needs" (content API 148); the ledger's `Wage`, `Rent`, `Owner` and `Dues` channels (`ledger.rs`). Credit is food lent before a harvest (`credit.rs`), not investment.
- **Absent:** draught animals (the handcart is pulled by people, `vehicle/handcart.toml`) and textiles (no fibre, yarn or cloth good).
- **Boundary:** save schema 170 (`civ-schema/src/lib.rs:66`), wire 1.96, content API 157.

## Where the reports agree

1. **Power is a local service, not a level.** "Do not give an entire polity a permanently available power total. A person or firm must arrange access to a functioning installation" (07-06 §6.1). The chain is resource, extraction, conversion, transmission, task, benefit, and any link can bind (07-06, executive recommendation).
2. **Heat, shaft work and electricity are separate accounts** (07-06 §1.1, §6.2; 07-12 §5.2), and a fuel's fitness is not its heat: "Coke-compatible ironmaking therefore deserves a separate capability from merely recognizing and burning coal" (07-06 §1.5).
3. **Output is the tightest bottleneck:** q = min(E/e, L/ℓ, I/a, r·Δt) (07-06 §1.1). Extra power does nothing without grain; extra hands do not enlarge a kiln (08-03 §4.4); a tool speeding a quarter of the work fourfold raises the whole by about 23 % (08-03 §1.2).
4. **Factories need not be steam, and steam need not be coal.** "A factory is an establishment that coordinates workers, machinery, materials, and power" (07-12 §1.3). "Steam must not require coal specifically"; "Coke-based ironmaking is not a prerequisite for the first steam engine" (07-06 §4, critical prerequisite rules).
5. **Known, buildable, operable, worth it** are four states (07-06 §4; 07-12 §5.1; 07-03 §3.4). Adoption compares the cost of the service (07-06 §1.7; 07-12 §5.2), so one machine pays in one settlement and not another without either entering an era (07-12 §2.4).
6. **Old and new coexist.** A steam engine pumped water back to Soho's wheels (07-06 §5.5); coal was half England's energy before Newcomen and never led to steam in Xinjiang (07-06 §5.4); iron does not abolish stone and copper (07-05 §5.1); hand weaving can thrive on machine yarn (08-03 §4.5); a known technique can stay rare for centuries (07-05 §5.2).
7. **Fuel scarcity is distance and price, not a cliff** (07-06 §1.6). Charcoal keeps a third of its wood's energy at 20 % yield (07-06 §2.3); one experimental steel run implies 82–137 kg of dry wood a kilogram of product (07-05 §2.2).
8. **Not industrialising must stay possible.** "If every technology retains a fixed positive discovery probability forever, eventual discovery becomes almost certain"; allow enduring states where nobody experiments or adoption never pays (07-12 §5.6).
9. **A firm is not its building** (08-08 §1.1); its size comes from opposing forces (08-08 §1.5); a wage is weighed against the household work it displaces (08-10 §1.3) within one conserved time budget (08-10 §5.2); a factory needs "particular roles present together", and a missing one stops others (07-12 §4.2).

**What games teach to avoid:** Factorio's frictionless organisation and power scales (07-06 §6.6); Victoria 3's production methods treated as the cause rather than the interface (07-12 §5.8); unlocks by era (07-03 §1). The lesson to keep: "make the player able to see why production stopped: no wind, low pond, exhausted ox, missing charcoal, damaged gearing, unpaid operator" (07-06 §6.6).

**Where they differ, or differ from the plan:**

- **Rungs.** Plan §5.4 has a service "capability ladder unlocked by technology". The reports separate knowing from operating (point 5). I read a rung as what a provider can operate, never what a settlement knows.
- **Why machines pay.** Allen's high wages against Humphries and Schneider's poorly paid spinners (07-12 §1.1, §2.1); the reports ask to keep both as parameterisations, not a blend (07-12 §6.2). In TCE a wage is what a works offers in hours of its owners' work (`plan_wage`), so dear labour arises from land, alternatives and demand, or it does not.
- **Units.** 07-06 counts in kW and MJ; TCE values goods in hours of a household's own work (`value.rs`). I propose physical units for power and heat in content and recipes, and hours for every decision.
- **Scale.** "Do not quietly provide a small settlement with the purchasing power, inventors, and skilled suppliers of millions of unseen people" (07-12 §5.5). M9 runs 10–30 settlements and no world beyond the map, so TCE is 07-12's closed small world, calibrated on its own terms.
- **Animals.** 07-06 §2.1 and §6.5 make traction and animal mills a v1 family. TCE has no beasts, so horse gins wait.

## 1. Proposal

| Mechanism | Saved state | Driven by |
|---|---|---|
| Fuel by heat | Goods with a `[fuel]` table; nothing per holder | Recipes name the fuels they take |
| Charcoal and coke | A batch under way (start, units, its fixed tool) | A make session with elapsed days |
| Iron and steel | Goods, techniques, recipes, deposits | Ore, charcoal and smiths |
| Water power | An installation: site, intake reach, head, wheel | Today's discharge, derived daily |
| Engines | A fixed tool with a `[power]` table, held by a firm | Fuel at the works |
| Pits that flood | Water standing in a pit | Ground-water heads (ADR-0021) |
| The works | A firm's own stations, lines, shift and offers | Its books, demand and the bell |
| Toll milling | Ledger legs on an appended channel | Households weighing walk and toll |
| Answers by hours | Nothing new: hours by activity are counted | ADR-0008 §3's second problem |

**Derived, not saved:** every kW, an installation's head and flow, a works' utilisation, the settlement's "uses water power" or "burns coal" labels, and what a household forecasts.

### 1.1 Fuel as goods: heat and fitness apart

- **A `[fuel]` table on goods** with `mj_per_kg`: wood 15.6, charcoal 29.5, bituminous coal 25.8, coke 28.2, peat 9.76 (07-06 §2.3, IPCC defaults). Fitness stays with the recipe, which already names its goods: a bloomery takes charcoal; a coke furnace is another recipe under another technique (07-06 §1.5).
- **Hearths burn heat, not kilograms.** `fuel_kg_per_person_day` becomes megajoules, met from the fuel goods the household holds, cheapest to it first. Until then every new fuel is `purpose = "material"`, or the hearth eats the smith's charcoal (`person.rs`, above).
- **Charcoal:** 5 kg of wood a kilogram (20 % yield; 15–25 %, 07-05 §2.1), 20–100 hours a tonne (07-05 §2.3), and a burn that takes days whoever tends it (an earth mound's 24, 07-05 §2.1). That needs a recipe field for elapsed time, `elapsed_days`, holding its fixed tool (a mound or kiln) and delivering at the end: "Active labor is not elapsed time" (08-03 §1.1). It is the first batch primitive; drying pots and malting (08-03 §2.1) can use it later.
- **Coal and peat** are deposit rules that deplete (07-06 §1.6), dug as pits; coke is a recipe. Knowing how to burn coal is a technique of its own, cheap to find where coal shows at the surface, and leads nowhere by itself (07-06 §3.2, §5.4).
- **Woodland.** Charcoal draws the same deadwood and pole stocks as hearths and builders; their gathering rate falls with stock (`half_rate_stock_per_ha`), so dearth shows as longer walks and dearer charcoal (07-06 §1.6). At an accessible 2 t/ha a year, 10,000 people at a tonne each need 50 km² (07-06 §1.6, illustrative). Whether felling for charcoal takes standing wood is open (§5).

### 1.2 Iron and steel

- **Techniques** with their footprint: `bloomery` (07-05 I1), routes `[firing]` or `[copper_smelting]` (copper is "a soft advantage rather than an absolute requirement", 07-05 §3.2), tried in charcoal burning and smelting copper, needing iron ore and charcoal at home; `forging` (I2), bloom to bar and bar to tools; `carburizing` (I3), one steel route; and `cast_iron` (I5), which needs a powered blast (§1.5). Coke smelting (X1) is an alternative route to pig iron, never a prerequisite of anything (07-06 §4).
- **A skill, smithing,** so iron is not worked as pottery.
- **Recipes inside 07-05 §2.3's envelopes:** bloom to bar 6–20 hours and 5–30 kg of charcoal a kilogram; simple forged tools 2–10 hours and 1–5 kg of charcoal a kilogram; nails at 1–2 hours a hundred (08-03 §2.2). Iron tools `serve` for stone and copper ones, as the copper axe does.
- **Deposits:** iron ore as land-profile rules beside copper's (where it lies is a content choice: bog ore in wet ground, bodies in uplands), each with its quality; a valley may have none (07-05 §1.1: every deposit has "a small, persistent material profile").
- **What it costs in wood (my arithmetic):** a tonne of bar a year at 5–30 kg of charcoal and 5 kg of wood each is 25–150 t of wood, the increment of 12–75 ha at 2 t/ha. An ore-rich but wood-poor valley should smelt little (07-05 §5.4).

### 1.3 Power at a site

- **An installation** is a record, as a crossing or a ditch is (`civ-land/src/crossings.rs`, `ditches.rs`): a weir and headrace (a ditch, with its intake, `max_share` and Manning capacity), a wheel (a fixed tool good with a `[power]` table), and the building it drives. It has an owner as a crossing does (household or polity; a firm through its household).
- **Its power is derived each day:** P = 9.81 · Q · H · η (07-06 §1.3). Q is the least of the headrace's capacity and `max_share` of today's discharge, after ditches above it took theirs; H is the bed's drop from intake to tail less the headrace's fall; η is the wheel's (07-06 §2.2: simple wheels 20–40 %, overshot 50–70 %, gearing 70–90 %). 0.5 m³/s at 2 m and 50 % is 4.9 kW. Wheels on one headrace share its Q; a cascade reuses it at each drop (07-06 §1.3).
- **Two techniques:** `water_wheel` (a direct-drive wheel, 0.5–3 kW) and `gearing` (a vertical wheel turning stones through gears, 1–10 kW), each with its wheel good (07-06 §2.2, §4.1). `rotary_quern` already stands for rotary milling; a water mill is the combination, not a node (07-03 §4.2).
- **Recipes draw power.** A recipe gains `kwh_per_unit`: a session makes the least of what its labour, inputs, `max_units` and the works' kW times its hours allow (07-06 §1.1). A water mill at 20–100 kg an hour with one or two attendants is 0.01–0.10 hours a kilogram (08-03 §2.2) against the rotary quern's 0.25 (`recipe/grind_grain_rotary.toml`).
- **Low water is visible.** A dry month gives fewer kW and fewer units, grain waits, and households go back to their querns (07-06 §5.6). The inspector says why: "the wheel turned at a third of its power; the river was low".
- **Wind** waits (§2): a windmill needs E[v³], not E[v]³ (07-06 §1.4), though the weather keeps each day's draw that would give it.

### 1.4 Engines that burn fuel

- **An engine** is a fixed tool whose `[power]` names fuel as its source, a rated kW and an efficiency: 0.5–1.5 % atmospheric, 2–5 % with a separate condenser (07-06 §2.3). It burns 3.6 / (MJ · η) kg a kWh of any fuel the recipe of its working allows: 14.4 kg of coal at 1 %, 3.6 kg at 4 % (07-06 §2.3). Its motion is reciprocating; rotary drive is a further technique (07-06 §4.2; 07-12 §1.3).
- **Its first use is pumping.** A pit dug below the ground-water head (ADR-0021's heads) fills, and its water must be lifted before digging goes on: by hand, at hours a cubic metre, or by a pump on a wheel or an engine (1 kWh lifts 36.7 m³ through 5 m at 50 %, 07-06 §2.2). That is Newcomen's niche, "deeper workable deposits where drainage is limiting" (07-12 §1.3).
- **Where it pays (my arithmetic):** 5 kW for ten hours is 50 kWh, 720 kg of coal a day at 1 %, or about 260 t a year; at 4 %, 180 kg a day. At a pit head that is spoil; a kilometre away by handcart it is a day's hauling for several people. Engines should stand where coal is cheap until carriage (the rail brief) or efficiency moves them (07-12 §2.2's long lag; §3.2).
- **What it needs:** a boiler, cylinder, valves and a pump (07-06 §4.2), so `cast_iron` or sheet copper, `forging` and `jointed_frame`; never coal or coke by name. Better boring or condensing is a further technique that makes another engine good, not a multiplier (07-12 §1.3's improvement links).

### 1.5 Machines

- **A machine** is a fixed tool good with a `[power]` demand (kW and motion) and a `crew`, the least hands present for it to run: two at a pit saw (08-03 §2.1). Recipes name it as `burn_lime` names the kiln.
- **Families for M10**, from 07-06 §6.5: grinding (millstones), crushing (an ore stamp), pumping, and hammering and blowing (a cam-driven hammer and bellows, 07-06 §4.1's rotation-to-reciprocation), each beside its hand or animal-free alternative. A powered saw belongs with timber from logs, which content does not yet have.
- **Transmission** stays inside a building: its machines share its movers' kW, allocated in the order the firm puts its lines, cached until a mover or machine changes (07-06 §6.4). No shafts between buildings.
- **Learning** stays in skill speed and quality and in tools' life; a better design is a technique with its own good (07-12 §1.2: keep engineering limits).

### 1.6 The works: a firm with power, machines, shifts and hands

- **The firm owns its stations.** Close ADR-0006 §5's gap: a firm holds its fixed tools and machines in its own stores, its recipes wear them, its building and installation name it. A closing firm's stations pass as its building does (ADR-0009 §7).
- **Several lines and stages.** A works keeps goods in progress between its recipes (ore, bloom, bar, tools; grain, flour), each line with its terms and its wage offer.
- **A shift.** A works sets its bell, a start and a length, reviewed with its wage (08-10 §5.6: every 14–30 days), and offers hire sessions of that length. A machine runs only while its crew is there, checked when a session begins; a hand who does not come idles it (07-12 §4.2). Owners may want long shifts to spread fixed costs; hands weigh them against fatigue and home (ibid.). After dark only with light.
- **Toll milling.** A household brings grain, takes flour, and the mill keeps a share (08-03 §1.4), on an appended ledger channel; the household weighs walk and toll against hours at its quern. That is how many mills serve a village without buying its grain (07-06 §5.3).
- **Investment.** A household or firm weighs a works by 07-12 §5.2's screening in hours of its own work: goods and building hours over the stations' life, fuel, wages and tolls against the hours its output would save at the output its books and the settlement's recorded demand lead it to expect. Only alternatives it knows; forecasts from what it has seen, never the kernel's truth.
- **Finance.** A works larger than one household can provision needs pooled goods: a partnership of households (08-08 §1.3), a polity raising a mill by law against a measured loss as `raise_building` raises a granary, or an organization (ADR-0026). Which of these M10 builds is open (§5).
- **Who works.** A hand weighs the shift's pay against what the household would do with the hours (08-10 §1.3), as `best_job` does; the landless and the idle come first. Hire's lowest age stays content; children's work is never a cheap bonus (07-12 §4.2).
- **Stopping is not failing.** A works idle for want of water is dormant (08-08 §1.8); it closes `Idle` only after `idle_close_days` without a sale, as now.
- **Nuisance.** A forge or engine is a fixed tool a fuelled recipe needs, so `fire_from_homes` already reaches it; smoke and noise wait for zoning's uses (ADR-0030).

### 1.7 Preconditions that emerge

| Precondition | What makes it in TCE | What the engine records |
|---|---|---|
| Food for specialists (07-12 §1.2) | Harvests, markets, trade between settlements | Food bought by households that farm none |
| Hands for hire | Little land or little to do at home: enclosure, plans with rent, crowding | Hours hired, the wage in hours, who took it |
| Knowledge | Finding by practice, contact, upbringing | Who found what, where, from what work |
| Sites and fuel | Reaches with fall; coal, ore and wood | kW by site and month; fuel's delivered cost |
| Demand | Recorded unmet demand and terms (ADR-0006 §4) | Sales and stockouts by good |
| Capital | Goods and hours, pooled by partnership or law | Each works' cost and who bore it |
| Repair and succession | Menders and heirs who know the techniques | Works stopped for want of a mender (07-12 §5.4) |
| Rules | Licences, duties, enclosure, plans | The laws in force and who proposed them |

Nothing is called "industrial" in the engine. The observer infers words from records (plan §1, rule 1): "a valley of water mills", "a coal town", never a state.

### 1.8 Answers by hours

- **Build ADR-0008 §3's second problem.** `[answers] hours` names activities; a household that spends many of its hours at one draws a member to try toward a technique whose recipe would cut them. This is Allen's mechanism as a household's problem, and directs effort without raising its chance (07-01 §1.6; 07-12 §5.3).
- **Absence stays absence.** No draw without practice of a `tried_in` activity and the `needs` at home, so a valley without ore, coal or a fall of water finds none of what needs them. The hazard per hour of practice is still constant (07-12 §5.6), so what it gives over decades is measured, never assumed.

## 2. What to defer, and why

| Defer | To | Why |
|---|---|---|
| Windmills | When a windmill is wanted | E[v³] by site and exposure (07-06 §1.4) |
| Draught animals, gins, harness | A livestock milestone | TCE has no beasts; 07-06 §1.2 warns against bonuses |
| Textiles, spinning and weaving machines | Open question 1 | No fibre goods; 07-07 is unread here |
| Turbines and electricity | M11 | 07-06 §4.2's post-v1 branch |
| Bulk and crucible steel | Later | 07-05 X2, I7; one route suffices for M10 |
| Patents, secrecy, licences of designs | Later | 07-01 §1.7, 07-12 §4.1; privileges cover goods today |
| Coerced labour | Never as a cheap wage | 07-12 §4.2, 08-10 §1.6 |
| Joint-stock companies | After partnerships | 08-08 §5.6's build order |

## 3. Guards: risks and validation

**Plot.** No date, population or wealth opens a technique, a works or a rung. No polity holds a power total (07-06 §6.1). No productivity multiplier: every gain is a recipe's hours, inputs or power. No "Engels' pause" either way (07-12 §4.3). If nobody builds a mill in the demo's time box, nudge a prior (a find's E50, a toll's share) and log a `NUDGE:`, never a mill.

**Nobody industrialises.** This is the largest risk. Wage work is rare (plan §9, slice J), nobody built a frame in M8 (slice DE), and a works needs framing, a site, smelting, a found technique and hands. Measure before tuning: who knows each technique, works considered and why rejected, hours at querns, the wage in hours. A world where it never pays is a valid answer (07-12 §5.6).

**Accounting.** Conservation as ADR-0006 §3: charcoal's and coke's losses are recipe inputs, a toll is a ledger leg, nothing is created by power. Power and heat are not conserved goods; they are rates read each day.

**Stylized facts, as tests:**

- **A 5 kW mill** sharply cuts grinding hours and leaves farming and building alone (07-06 §5.1).
- **Drought** brings queues, flour drawn down and querns at work again (07-06 §5.6).
- **Coal without steam:** a valley with coal burns it for decades with no engine (07-06 §5.4).
- **Complement, not rebuild:** an engine may pump for a wheel (07-06 §5.5).
- **One machine, two answers:** twin settlements differing only in land per household or a reach's fall adopt differently (07-12 §2.4).
- **Fuel before ore:** a wood-poor, ore-rich settlement smelts less (07-05 §5.4).
- **Coexistence:** iron leaves stone and copper tools in use where cheaper (07-05 §5.1).
- **Binding constraints:** a second hand at a one-person machine adds nothing; at a pit saw it enables the work (08-03 §4.4).
- **Slow aggregates:** a settlement's output per worker moves with adoption, not a find (07-12 §4.1).
- **Counterfactuals:** remove the coal and keep the river; remove the last mender (07-12 §5.7).

**Compute.** ADR-0032's amendment of 2026-10-21 puts the engine at about 0.7 ms a person-day in settlements of 2,000 and 2.2 ms at 10,000, the cost growing with settlement size; decisions were about half of a Detailed day in M8a's profile (the M9 brief §1). What M10 adds:

- `decide::candidates` walks every activity at every decision (`decide.rs:913`; 79 activities today). Fifteen to twenty-five new make activities add 20–30 % to that walk. Filter a household's candidates once a day by the techniques its members know before content lands.
- `discover` loops over every technique at every session's end (`population/knowledge.rs:240`); M10 may double the 26. Index techniques by the activities they are tried in (07-01 §5.5).
- `Population::firm` is a linear search and `working_for` scans every person at each session of a firm (`population/firm.rs:23`, `:141`): a works of 50 hands at 10,000 people is about a million checks a day. Keep a count per firm as work starts and ends.
- Stores are dense by good in every household and firm (`Household.stores`, `Firm::stores`): ten to fifteen goods on 34 grow every settling and valuation by a third.
- Power is computed once a day per installation, a few dozen at most.
- **Household resolution** (ADR-0032 §2): a shift is a hire block; a works' day is its summed blocks, bounded by its power and machines, crews met in aggregate. That approximation must be listed and pass Gate C, or works must be named as holding a settlement Detailed.

**Saves.** Save, load and save exact mid-burn, mid-shift and with a flooded pit; an M9 save loads with no installations, no works and no new goods.

**Observer.** A works page (stations, lines, shift, hands, power by day and why it fell short); installations on the map with their kW; a fuel row (what each settlement burns, delivered cost); the chronicle in plain words: "Ash's mill stood idle nine days in August; the river was low."

## 4. Suggested slices

1. **Fuel and iron, no power.** `[fuel]`, hearths burning heat, `elapsed_days`, charcoal, iron ore deposits, `bloomery`, `forging`, smithing, iron tools that serve. *Tests:* conservation; a hearth never burns charcoal while firewood is cheaper; wood drawn for a tonne of bar; smoke and Gate B unchanged where nobody knows the techniques.
2. **The firm's own stations and stages.** Firms hold and wear their fixed tools; several lines with goods in progress; toll milling on an appended channel; the per-firm worker count. *Tests:* stations pass on closure; toll legs conserve; an old save loads.
3. **Water power at a site.** Installations on a headrace, daily kW from discharge, `water_wheel` and `gearing`, `kwh_per_unit`, the water mill. *Tests:* 07-06 §1.3's 4.9 kW; a dry month's queue and querns; two wheels on one headrace share it.
4. **Answers by hours and the choice to build.** `[answers] hours`; a household or polity weighs a mill in hours; partnership or law as the ADR decides. *Tests:* twin settlements adopt differently; no mill where querns cost little; the candidate filter measured on the bench.
5. **Coal, flooded pits and engines.** Coal and coke, pits below the water head, pumping by hand, wheel or engine, the atmospheric engine and rotary drive. *Tests:* coal burned without engines; an engine at a pit head pays and one far from coal does not.
6. **The works.** Machines with crews, the bell, shifts, the household-resolved block, the works page. *Tests:* a missing hand idles a two-hand machine; Gate C with a works; save mid-shift.
7. **The M10 power demo, time-boxed.** Ten settlements, some with falls and coal, some without; record who builds what and why; nudges logged.

## 5. Open questions for the designer

1. **Textiles.** Historical factories were mostly spinning; TCE has no fibre. Does M10 add cloth (07-07), or are its works mills, forges, nail shops and pits?
2. **Domestic coal.** May hearths burn coal and peat once heat is counted, as England's did (07-06 §2.4)? It changes woodland pressure everywhere.
3. **Woodland.** Does charcoal burning take standing wood, and can a habitat be cut to scrub, or does it only deplete the yearly increment?
4. **Finance.** Partnerships, a polity's mill, or an organization's: which does M10 build?
5. **Who introduces far techniques?** With no world beyond the map, are steam and coke smelting found, brought by the observer, or allowed to stay unknown in most worlds?
6. **Shifts and light.** Do shifts wait for gas light, or may a works run by daylight only?
7. **Animals.** Do horse gins and harness come with M10's power, or with livestock later?
8. **Toll's rule.** Is a toll a share of the grain set by the miller, by custom, or by law, as ADR-0026's privileges are?
9. **Household resolution.** Is a works' summed-blocks day acceptable, or does a running works hold its settlement Detailed?
10. **Units on the wire.** Does the observer speak kW and MJ, or only hours and kilograms?
11. **Flooded pits and wells.** Should a deep pit draw down nearby wells' water as pumping does?
12. **A metalworking route for copper.** Should copper move from pottery's skill to smithing?

## 6. What an ADR must decide

One ADR, "Power, fuel and the works", amending ADR-0006, ADR-0008 and ADR-0010:

- **Power as a derived rate at a site:** installations as saved records with an intake, a head and a wheel; kW recomputed daily from discharge and never saved; recipes' `kwh_per_unit` and the bottleneck rule (07-06 §1.1, §1.3).
- **Fuel by heat:** the `[fuel]` table, hearths burning megajoules, fitness kept in recipes.
- **Batches in elapsed time:** `elapsed_days`, the batch record and its saving.
- **The firm's stations and stages** (amending ADR-0006 §5): fixed tools and machines held by the firm, several lines, goods in progress, the shift, and the toll channel.
- **Pits and ground water** (amending ADR-0010): when a pit floods and how water is lifted.
- **Answers by hours** (amending ADR-0008 §3).
- **Who may own a works:** households, partnerships, polities, organizations.
- **The household-resolved contract** for a works (ADR-0032 §2).

Channels, exits, kinds and goods stay append-only; the save schema, wire and content API bump.
