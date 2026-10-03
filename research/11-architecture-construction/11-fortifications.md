# Fortifications: forms, costs, and effectiveness

## A simulation-ready report for The Civilization Engine

**The central recommendation is to model fortifications as systems that control movement and buy time—not as buildings that add a universal defense bonus.** Their value depends on the defended space, available people, supplies, terrain, weapons, maintenance, and the political organization operating them.

TCE should distinguish three outcomes: **preventing entry, sustaining resistance, and retaining control of the settlement**. A defense can succeed at one and fail at another. At Ōhaeawai in 1845, Māori defenders repelled a costly assault but subsequently evacuated; the British then occupied an empty fortification. “Assault repulsed” and “site eventually occupied” are different events. [NZ History](https://nzhistory.govt.nz/war/northern-war/ohaeawai)

The evidence below uses four labels: **E—observed or recorded; R—reconstructed; D—derived mathematically; P—proposed simulation parameter.** Confidence is assessed separately. A precisely calculated result can still depend on a weak historical assumption.

---

## 1. Mechanisms: rules TCE can implement

### 1.1 Why communities fortify

Do not require agriculture, cities, or a state before fortifications become possible. The Amnya sites in western Siberia had banks, ditches, and palisades around **6000 BCE**, among hunter-gatherer communities. Their relationship to concentrated, storable resources is a plausible interpretation; year-round occupation and the precise motives behind construction remain uncertain. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/worlds-oldestknown-promontory-fort-amnya-and-the-acceleration-of-huntergatherer-diversity-in-siberia-8000-years-ago/90559E4105F93528A6552B36C7236259)

A suitable **proposed investment rule** is:

\[
\text{Build when perceived avoided losses}
+\text{access-control benefits}
+\text{political benefits}
>
\text{construction, upkeep, and opportunity costs}.
\]

Evaluate this through particular decision-makers, not an omniscient settlement optimizer. A ruler may protect a citadel and treasury while leaving peripheral households exposed. Farmers may prefer a refuge enclosure; merchants may favor a circuit protecting warehouses and market access.

Allow competing responses: fortify, relocate, disperse, negotiate, form an alliance, maintain a field force, or accept losses. Fortification should therefore be common under some conditions without becoming mandatory everywhere.

**Archaeological caution:** an enclosure is not automatically a military installation. Boundary-making, ritual, livestock control, and defense can overlap. Parkinson and Duffy emphasize this interpretive problem; Keeley and colleagues argue that combinations such as defended entrances, projecting positions, and particular ditch forms provide stronger military evidence than enclosure alone. [Academia](https://www.academia.edu/49176670/Fortifications_and_Enclosures_In_European_Prehistory_A_Cross_Cultural_Perspective)

### 1.2 Represent forms as combinations of components

The following are **proposed mechanics**, not measured combat multipliers. They translate physical form into simulation behavior; the historical calibration examples follow in Section 2.

| Component | What it should do in TCE | Costs and vulnerabilities to represent |
| --- | --- | --- |
| **Palisade** | Block ordinary movement; slow crossing; screen and protect occupied positions. A platform or bank may be needed for defenders to see and fight over it. | Suitable logs, felling, transport, post setting, bracing, and replacement. Track combustible material and moisture-dependent deterioration. |
| **Ditch** | Add a descent, exposed crossing, and ascent; obstruct access to the wall foot. Its benefit depends on profile, approach terrain, and defensive coverage. | Excavation, spoil handling, drainage, slumping, and sediment clearance. A wet moat requires an actual water source and water balance. |
| **Earth rampart** | Raise the fighting position and provide protective mass. Permit timber, stone, or brick facing without changing the core into “solid masonry.” | Earth volume, shaping, compaction, retaining structures, drainage, and occupied land. Erosion and slope failure are distinct from damage by weapons. |
| **Stone or brick wall** | Obstruct entry; support protected positions where designed to do so; resist damage according to geometry and material layers. | Foundation, facing, core, mortar, lifting, scaffolding, access stairs, and parapets. Local damage can create a breach without destroying the whole circuit. |
| **Tower or projecting platform** | Improve observation and coverage of adjacent approaches. Its value comes from position, visibility, access, and occupants. | Additional structure, floors, stairs, roofing, and guards. An empty tower supplies no weapon fire. |
| **Gate complex** | Create a controlled crossing with opening schedules, inspection, queues, toll collection, and security decisions. | Doors, fittings, approaches, guard accommodation, staffing, and maintenance. Gates can be heavily defended rather than automatically being the weakest section. |
| **Bastioned system** | Combine projecting fighting positions, mutually supporting sectors, substantial protective mass, ditches, and outer works. | Extensive earthmoving, specialized layout, land clearance, platforms, drainage, and continuing adaptation to weapons. Do not price it as an ordinary wall with decorative points. |

The historical significance of defended entrances and projecting positions is supported by cross-cultural archaeological comparison. Gunpowder-era Venetian and Vauban systems also demonstrate why the relevant object is an integrated defensive layout rather than an isolated wall type. [Springer](https://link.springer.com/article/10.1007/s10814-006-9009-0)

**Conserve materials.** Excavated ditch soil can supply a nearby rampart. Do not charge for excavating the same material twice, but do charge separately for moving, spreading, and compacting it. Preserve the distinction between in-situ soil volume, loose spoil, and compacted fill.

**Use terrain locally.** A cliff can reduce the constructed barrier needed along one sector without eliminating observation or access requirements. A river can hinder approach while simultaneously providing a supply route. The cost and defensive role of a sector should follow its actual surroundings.

### 1.3 Construction is a supply-chain project

Represent construction as a task network:

**survey and acquire land → prepare access → obtain materials → establish foundations → build barriers and platforms → install gates → provision and staff.**

Tasks can overlap, and incomplete works can provide partial benefits. They should not provide the full benefit of a completed enclosure.

For each task, daily progress is limited by the smallest of **available skilled labor, delivered materials, tools and transport, and usable work-front capacity**. Increasing the nominal workforce cannot overcome a missing quarry road or an undelivered gate.

Maintain two cost accounts:

| Account | What it measures |
| --- | --- |
| **Real resource cost** | Worker-hours, material quantities, transport capacity, fuel, food, and land diverted from other uses. |
| **Financial expenditure** | Purchased materials, wages, contracts, transport charges, compensation, and administration. |

Do not add all upstream labor costs again when they are already embodied in purchased materials. Likewise, compulsory labor is not economically free: households lose work time and still require subsistence.

Hwaseong’s construction records are especially useful here because they distinguish materials, workers, procurement, transportation, and project administration rather than reporting only a finished wall length. [Suwon Cultural Foundation](https://www.swcf.or.kr/english/?p=33)

### 1.4 Resolve sieges through competing processes

Avoid a universal attacker-to-defender ratio or “fortification level × siege duration” rule. Instead, compare several evolving processes:

| Process | Relevant state |
| --- | --- |
| **Entry or breach** | Accessible approaches, obstacle geometry, occupied defensive positions, engineering progress, local damage. |
| **Defender endurance** | Food, water, ammunition, crowding, health, fatigue, pay, cohesion, and confidence in relief. |
| **Attacker endurance** | Supply consumption, transport losses, disease, wages, political commitment, and exposure to relief forces. |
| **Relief or escape** | Outside allies, open waterways, hidden or contested routes, and the movement of field armies. |
| **Negotiation or political failure** | Expected treatment after surrender, leadership legitimacy, divided interests, and control of gates. |

A basic food balance is:

\[
F\_{t+1}=F\_t+I\_t-Q\_t-\ell\_t,
\]

where \(F\) is usable food, \(I\) incoming supply, \(Q\) consumption, and \(\ell\) losses. Track water separately.

A rough endurance estimate is \(F/(Q-I)\) when net consumption is positive. It is **not** a surrender timer: morale, water, health, negotiations, and military events can dominate.

Model land encirclement and complete isolation separately. Also distinguish protecting the settlement’s interior from protecting its fields. A fort can remain unbreached while its inhabitants lose access to harvests and trade.

Local geometry matters more than average wall condition. A usable breach or uncontrolled gate may admit attackers even while most of the perimeter remains intact. Conversely, an unoccupied wall still remains a physical obstacle.

### 1.5 Maintenance should create work orders, not automatic global decay

Use component-specific processes:

| Component | Maintenance jobs |
| --- | --- |
| Timber | Replace compromised posts and braces; repair gates and platforms. |
| Earthworks | Restore profiles, clear drains, repair erosion and slumps. |
| Masonry | Repair joints and facing, stabilize foundations, remove damaging vegetation. |
| Ditches and moats | Remove sediment and debris; maintain drainage or water-control structures. |
| Defensive readiness | Maintain access, stores, weapons, watch schedules, and emergency procedures. |

At Notion, surveyors identify different masonry materials, erosion-prone stone, repairs, and potentially unstable construction on steep ground. Cahokia’s stockade underwent repeated reconstruction. These are reasons to model maintenance and rebuilding, not evidence for a universal annual deterioration percentage. [LSA Technology Services](https://sites.lsa.umich.edu/notionsurvey/current-projects/fortifications/)

### 1.6 Fortifications reshape growth without imposing a hard population cap

For TCE, make protected land attractive but costly. Households should weigh safety against rent, crowding, gate delays, and distance to work. This can generate dense interiors, extramural suburbs, gate-centered commerce, and eventual enlargement or removal.

There is empirical support for a broader safe-haven mechanism: Dincecco and Onorato study more than **800 European conflicts, 800–1799**, and find an association between conflict exposure and urban population growth. That is **not** a clean estimate of the causal effect of constructing walls, nor evidence that war improves overall welfare. [Springer](https://link.springer.com/article/10.1007/s10887-016-9129-4)

The geometric constraint is straightforward:

\[
L\_{\min}=2\sqrt{\pi A},\qquad L=\kappa L\_{\min},\quad \kappa\geq1.
\]

Here \(A\) is enclosed area and \(\kappa\) represents deviation from a circular perimeter. Terrain determines which portions require construction.

A circular **10-hectare** enclosure requires about **1.12 km** of perimeter; **100 hectares** requires **3.54 km**. Doubling enclosed area increases the minimum perimeter by only **41.4%**, but replacing an existing ring is a large, discontinuous investment. These are geometric results, not historical averages.

---

## 2. Parameters, construction costs, and maintenance

### 2.1 Historical anchors

These examples are deliberately heterogeneous. An attacker’s encircling wall, a defended urban core, and a territorial boundary are not interchangeable observations.

| Case or parameter | Quantitative evidence | Evidence and confidence | Appropriate use |
| --- | --- | --- | --- |
| **Cahokia stockade, North America** | Approximately **3.2 km**; estimated **15,000–20,000 logs** per construction. Logs about **0.30 m diameter**, **6.1 m total length**, set **1.2–1.5 m** into the ground. Initial construction around **1100 CE**, followed by **three rebuildings over roughly 200 years**. | **R; medium.** Archaeological reconstruction summarized by the site. | Timber procurement and rebuilding scale. The stockade enclosed the central precinct, not the entire settlement. [Cahokia Mounds](https://cahokiamounds.org/explore/) |
| **Notion, Anatolia** | Curtain approximately **2.5 m thick**; original height probably **at least 4 m**. **16 certain and 29 possible towers** identified. | Width **E; high** locally. Height and uncertain towers **R; medium/low**. | Masonry geometry and uncertainty-aware reconstruction. Do not treat all possible towers as confirmed. [LSA Technology Services](https://sites.lsa.umich.edu/notionsurvey/current-projects/fortifications/) |
| **Roman siege works at Masada** | **4,290 m** circumvallation plus **1,980 m** camp walls; estimated **26,700 m³** stone. Modeled labor **50,000–80,000 source-days**, or **11–16 days with 5,000 builders**. | Length **E**; volume and labor **R**. Medium confidence in reconstruction, lower in productivity assumptions. | A concentrated, locally supplied military construction scenario—not ordinary urban masonry. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-archaeology/article/roman-siege-system-of-masada-a-3d-computerized-analysis-of-a-conflict-landscape/32C59BE59ACD3E9A91C95F947DFD271E) |
| **Experimental masonry benchmark** | Erasmus’s experiments, as reported in the Masada study: **6.5–8.5 five-hour worker-days/m³**, equivalent to **32.5–42.5 worker-hours/m³**. | Experimental result reported through another study; **medium**, narrow scope. | A comparison point, not a universal construction rate. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-archaeology/article/roman-siege-system-of-masada-a-3d-computerized-analysis-of-a-conflict-landscape/32C59BE59ACD3E9A91C95F947DFD271E) |
| **Hwaseong, Korea** | **5.74 km** circuit, approximately **130 ha**, **four principal gates**; constructed **1794–1796**. | **E; high** for published site dimensions and chronology. | Integrated urban enclosure, terrain, water crossings, and gate layout. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/817/) |
| **Hwaseong budget** | Reported cost approximately **873,518 nyang**; adult unskilled daily wage **0.25 nyang**; construction approximately **32 months**. | Recorded figures transmitted through an official summary; **medium** pending checking against a critical edition. | Financial scale, not a direct person-day count. [Suwon Cultural Foundation](https://www.swcf.or.kr/english/?p=33) |
| **Sungbo’s Eredo, Nigeria** | Published descriptions give a circuit up to approximately **170 km**, with some ditch or bank sections reaching **10 m**. | **E/R; medium**; maximum dimensions, not average cross-sections. | Territorial enclosure and access control. Do not model it as a single densely occupied walled city. [OpenEdition Journals](https://journals.openedition.org/aaa/5124) |

**The Masada estimate excludes the siege ramp and does not establish the total siege duration.** Its construction-time estimate depends on assumed productivity and workforce availability. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-archaeology/article/roman-siege-system-of-masada-a-3d-computerized-analysis-of-a-conflict-landscape/32C59BE59ACD3E9A91C95F947DFD271E)

For Hwaseong:

\[
873{,}517.79/0.25 \approx 3.49\text{ million unskilled wage-days}.
\]

This is a **purchasing-power equivalent**, not 3.49 million days actually worked: the budget also purchased materials and other inputs. It should not be converted into modern currency or generalized into a universal cost per kilometre. [Suwon Cultural Foundation](https://www.swcf.or.kr/english/?p=33)

### 2.2 Starting parameters for TCE

The following are **explicitly provisional design inputs**. They make a first implementation possible while preserving uncertainty. They are not claimed historical distributions.

Define one standardized **worker-day as eight worker-hours**. Preserve original source-day lengths separately.

| Parameter | Initial value or sensitivity range | Evidence/confidence | Scope and warning |
| --- | --- | --- | --- |
| Hand excavation | **0.5–2 m³/worker-day** | **P; low** | Ordinary soil, short casting distance. Separate rock, deep excavation, wet ground, and long hauling. |
| Delivered earth spreading, shaping, and compaction | **0.25–1 worker-day/m³** | **P; low** | Excludes excavation and transport. Strongly dependent on technique and required finish. |
| Rough masonry, aggregate local-supply recipe | **4–10 worker-days/m³** | **P; low** | A testing band, not a global empirical average. Define included tasks; do not then charge the same upstream labor again. Fine ashlar requires a different recipe. |
| Timber replacement sensitivity tests | **5, 15, and 30 years** | **P; low** | Alternative scenarios, **not** a calibrated probability distribution. Species, moisture, exposure, and treatment should eventually determine condition. |
| Routine upkeep reserve | **0.5–3% of original direct construction labor/year** | **P; low** | Temporary budgeting approximation, not automatic loss of integrity. Damage from war or disasters is additional. Replace with condition-driven tasks. |
| Strategic food-stock target | **30–180 days of intended occupancy** | **P; low** | Institutional policy range, not a historical siege-duration distribution. |
| Continuous watch staffing | **3–4 people per continuously occupied post** | **D; high arithmetic confidence** | Follows from 6–8 duty hours/person/day, before sickness, leave, and other duties. Does not prescribe posts per metre. |
| Structural simulation sector | **10–25 m** | **P; implementation choice** | Use finer geometry at gates, corners, and breaches. Not a historical building module. |

The most important missing calibration is **task-specific labor under specified tools, materials, transport distances, and workday lengths**. A single “masonry productivity” variable will otherwise hide most of the economic differences that TCE is meant to simulate.

### 2.3 Worked construction modules

These are **derived examples using the provisional rates above**, not reconstructions of named historical works.

| Illustrative module | Quantity calculation | Labor or materials implied |
| --- | --- | --- |
| **100 m ditch**, 4 m top width, 1 m bottom width, 2 m depth | \(100\times(4+1)/2\times2=500\text{ m³}\) | **250–1,000 worker-days** for excavation alone. |
| **100 m rampart**, triangular section, 6 m base, 2 m height | \(100\times6\times2/2=600\text{ m³}\) | **150–600 worker-days** for shaping/compaction after delivery, plus acquiring and moving the earth. |
| **100 m close-set palisade**, 0.20 m diameter, 4 m total post length | Approximately **500 posts**; cylindrical volume approximately **62.8 m³** | Add bracing, wastage, felling, transport, and erection. No defensible generic labor total is supplied. |
| **100 m masonry curtain**, 4 m high, 2 m thick | \(100\times4\times2=800\text{ m³}\) | **3,200–8,000 worker-days** under the provisional aggregate recipe; excludes foundation, parapet, towers, and gates. |
| **Illustrative square tower shell**, 5 × 5 m exterior, 3 × 3 m interior, 8 m high | \((25-9)\times8=128\text{ m³}\) | Price the shell’s material recipe separately from floors, stairs, openings, roof, and parapet. |

Under those assumptions, 100 fully supplied workers could complete the **masonry task alone** in 32–80 working days. That is not a forecast for the whole project: access works, supply preparation, specialized tasks, and interruptions can dominate elapsed time.

**Gates and bastions should be priced from their bill of quantities.** Their variability makes a fixed “gate = 10% extra” or “bastion = 2,000 days” rule especially misleading.

### 2.4 Financing and opportunity cost

TCE institutions should be able to combine treasury spending, household contributions, labor obligations, purchased contracts, and dedicated gate revenues. These are alternative policy mechanisms, not requirements for every culture.

Track who bears the burden. An elite-financed citadel, a collectively maintained refuge, and a city wall funded by taxes may produce different legitimacy and compliance outcomes even when their physical construction is identical.

Also charge for **land and access**. Hwaseong’s account includes compensation and substantial procurement and transportation arrangements. A fortification budget is therefore not simply the price of stone plus masons. [Suwon Cultural Foundation](https://www.swcf.or.kr/english/?p=33)

---

## 3. Variation across eras and world regions

These are comparison categories, **not a technology ladder or scripted chronology**.

| Setting | Historical pattern and example | Implication for TCE |
| --- | --- | --- |
| **Foragers and hunter-fishers** | Amnya demonstrates substantial defensive enclosure among hunter-gatherers around 6000 BCE. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/worlds-oldestknown-promontory-fort-amnya-and-the-acceleration-of-huntergatherer-diversity-in-siberia-8000-years-ago/90559E4105F93528A6552B36C7236259) | Repeated use of a valuable place and coordinated labor can matter more than farming or formal government. |
| **Early farming communities** | European prehistoric enclosures had varied and sometimes overlapping military, social, and ceremonial functions. [Academia](https://www.academia.edu/49176670/Fortifications_and_Enclosures_In_European_Prehistory_A_Cross_Cultural_Perspective) | Allow enclosure without assuming constant warfare; infer defense from configuration and use. |
| **South Asian early cities** | Dholavira, occupied approximately 3000–1500 BCE, combined nested fortified divisions with water storage; construction included stone and mud-brick cores. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1645/) | Model internal enclosures and water infrastructure, not only one exterior ring. |
| **Mediterranean and western Asian cities** | Notion shows a masonry circuit with towers, gates, varied construction, and repairs. [LSA Technology Services](https://sites.lsa.umich.edu/notionsurvey/current-projects/fortifications/) | Local materials, terrain, contractors, and rebuilding can produce a heterogeneous wall. |
| **North American agricultural centers** | Cahokia’s large timber stockade enclosed a central precinct and was repeatedly reconstructed. [Cahokia Mounds](https://cahokiamounds.org/explore/) | Timber defenses need not imply small settlements or low organizational capacity. |
| **West African kingdoms** | Sungbo’s Eredo enclosed a territorial landscape; the capital also had its own fortifications. [OpenEdition Journals](https://journals.openedition.org/aaa/5124) | Distinguish territorial boundaries, city defenses, and inner political centers. |
| **East Asian preindustrial states** | Hwaseong combined stone, brick, gates, varied terrain, water crossings, and firearm-related defensive features. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/817/) | Recipes can hybridize. Do not force every society through the same European sequence. |
| **Early-modern artillery fortification** | Venetian and Vauban works exemplify integrated bastioned systems adapted to terrain and gunpowder warfare. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1533/) | Geometry, protective mass, engineering knowledge, and land demand change together. |
| **Māori adaptation to firearms** | Ōhaeawai combined palisades with protected trenches and underground shelter. [NZ History](https://nzhistory.govt.nz/war/northern-war/ohaeawai) | Locally available timber and earth can support sophisticated adaptation without masonry urban walls. |
| **Industrial warfare** | Fort Pulaski fell after a two-day artillery battle within a longer, approximately 50-day siege. Rifled artillery challenged its masonry defenses. [National Park Service](https://www.nps.gov/places/fort-pulaski.htm) | Weapon changes can devalue a particular construction system, not the entire concept of fortification. |
| **Industrial and modern field defense** | First World War armies dug extensive protective positions under artillery and machine-gun fire; water, collapse, and repair remained major problems. [National WWI Museum and Memorial](https://www.theworldwar.org/learn/about-wwi/trench-warfare) | Retain earthworks and shelters. For modern extensions, emphasize cover, concealment, dispersion, and protected movement rather than merely increasing city-wall height. |

This sample is geographically broad but not exhaustive. It should seed **multiple viable combinations**, not assign one permanent defensive style to each culture.

---

## 4. Stylized facts and validation targets

A correct simulation should reproduce patterns, not merely recreate famous outlines.

| Pattern | Evidence or numerical benchmark | Validation test |
| --- | --- | --- |
| **Defense can compensate for numerical inferiority without guaranteeing permanent possession.** | At Ōhaeawai, a **250-man assault** suffered **40 killed and 70 wounded**—about **44% casualties**—against a defended position held by little more than 100 fighters. The defenders later withdrew. This is an exceptional case, not a default casualty modifier. [NZ History](https://nzhistory.govt.nz/war/northern-war/ohaeawai) | Record assault outcome, losses, withdrawal, and eventual occupation separately. |
| **Threat-specific protection matters.** | Pulaski’s short bombardment phase contrasted with the longer operation surrounding it. [National Park Service](https://www.nps.gov/places/fort-pulaski.htm) | A weapon innovation should alter the effectiveness of particular materials and geometries, not add a universal attack bonus. |
| **Fortification cost scales with perimeter and cross-section, not directly with population.** | At fixed density and cross-section, ideal perimeter cost per person scales approximately as \(N^{-1/2}\). **D.** | Larger communities should gain geometric economies of scale, offset by larger projects and coordination burdens. |
| **Refugees increase both protection demand and logistical stress.** | With fixed stocks and identical consumption, doubling occupants halves nominal food endurance. **D.** | A refuge policy should affect crowding, rations, workforce, and survival—not only population count. |
| **Enclosures do not define the whole settlement.** | Cahokia’s defended center and the nested/territorial examples above show different scales of enclosure. [Cahokia Mounds](https://cahokiamounds.org/explore/) | Permit suburbs, fields, separate compounds, and fallback enclosures. |
| **Urban growth can eventually compromise the circuit.** | UNESCO records **nine openings** cut through Hwaseong’s walls to accommodate city traffic. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/817/) | Under changing threat and traffic conditions, agents should propose new gates, breaches, expansion, or removal. |
| **Safety can attract population while war destroys wealth.** | The European conflict–urbanization association supports a possible concentration mechanism, not a welfare benefit from conflict. [Springer](https://link.springer.com/article/10.1007/s10887-016-9129-4) | Measure population relocation separately from total regional welfare and mortality. |

Also test **deterrence**. A successful fortification may never be attacked. Calibrating only against recorded sieges selects places that attackers considered worth challenging and misses avoided raids, bypassed strongpoints, and abandoned attacks.

---

## 5. Recommended TCE representation

### 5.1 Data model

Use a **defensive network** linked to the settlement’s existing transport, building, labor, and political systems.

| Entity | Minimum state |
| --- | --- |
| **Enclosure** | Boundary, defended area, owner, intended beneficiaries, entrances, fallback enclosures. |
| **Defense sector** | Geometry, material layers, foundations, integrity, climb/crossing difficulty, platforms, access, visibility. |
| **Gate** | Open/closed state, passage capacity, inspections, guards, controller, schedules, damage. |
| **Defensive position** | Occupant capacity, protection, visibility, weapon compatibility, ammunition access. |
| **Construction or repair project** | Bill of quantities, task dependencies, workforce requirements, delivered stock, expenditure, completion. |
| **Siege state** | Contested approaches, supply-route status, forces, engineering progress, stocks, relief expectations, negotiations. |

Keep **physical condition**, **military readiness**, and **political control** separate. A structurally sound fort may be unprovisioned; a damaged wall may be actively defended; a gate may change hands without being demolished.

### 5.2 Individual agents and institutions

Individuals should perform visible tasks: digging, cutting timber, hauling, building, guarding, repairing, supplying, queuing, sheltering, and fighting.

Institutions should decide which tasks happen and who bears their costs. Give councils, rulers, commanders, and households different priorities and imperfect information. Guard duty competes with productive work; emergency mobilization affects food production; ration policies distribute scarcity rather than erase it.

Fortification proposals should use TCE’s existing political machinery. Relevant decisions include the proposed alignment, compulsory contributions, protected access, gate fees, emergency admission, demolition of obstructing buildings, and surrender terms.

### 5.3 Computational simplification

For 10k–50k people, maintain geometry at sector level rather than simulating individual stones.

Precompute visibility and approach connectivity, updating them when construction or damage changes the layout. Update ordinary deterioration and budgets slowly; use finer time steps only around active engagements, fires, gates, and breaches.

At low simulation detail, aggregate repetitive labor and combat exchanges while retaining the identities, locations, injuries, and resource consumption of participating agents. Transitions between levels of detail must conserve people and materials.

For siege outcomes, either use explicit event thresholds or competing event rates:

\[
P(\text{event during }\Delta t)=1-e^{-\lambda\Delta t}.
\]

The rate \(\lambda\) should respond to actual conditions. It should not be a disguised fixed probability that every siege ends after a predetermined number of days.

### 5.4 Existing models and games worth borrowing from

| Precedent | Useful pattern | What not to import as historical evidence |
| --- | --- | --- |
| **Architectural energetics and the Masada reconstruction** | Recover quantities from geometry, then apply explicit productivity assumptions and workforce scenarios. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-archaeology/article/roman-siege-system-of-masada-a-3d-computerized-analysis-of-a-conflict-landscape/32C59BE59ACD3E9A91C95F947DFD271E) | Assumed labor rates presented as measured ancient performance. |
| **Dwarf Fortress** | Persistent individual needs, material production, multilevel construction, and mechanisms interacting with physical layout. [Bay 12 Games](https://www.bay12games.com/dwarves/features.html) | Fantasy hazards or fine-grained simulation detail that does not serve TCE’s scale. |
| **Stronghold** | A visible working economy connected to castle construction and spatial siege combat. [Steam Store](https://store.steampowered.com/app/40950/Stronghold_HD/) | Gameplay prices, accelerated construction, or balance values as archaeological estimates. |
| **Seshat military-technology research** | Comparative coding of defensive technologies and their co-occurrence with institutions and other capabilities. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0258161) | Turning a technology count into a universal combat-strength index. |

For v1, prioritize **enclosure geometry, gates, material/labor costs, staffing, food and water, local breaches, relief, and urban growth**. Detailed projectile–masonry physics, individual underground excavation, and dozens of narrowly named fort types can wait.

---

## 6. Sources, datasets, and evidence limits

### Core research and calibration sources

| Source | Why it matters | Main limitation |
| --- | --- | --- |
| **Piezonka et al. (2023), “The world’s oldest-known promontory fort…”**, *Antiquity* | Early hunter-gatherer fortification and its environmental/social context. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/worlds-oldestknown-promontory-fort-amnya-and-the-acceleration-of-huntergatherer-diversity-in-siberia-8000-years-ago/90559E4105F93528A6552B36C7236259) | “Oldest known” is not proof of the first fortification ever; motives and occupation intensity remain interpretive. |
| **Keeley, Fontana & Quick (2007), “Baffles and Bastions,”** and **Parkinson & Duffy (2007), “Fortifications and Enclosures…”**, *Journal of Archaeological Research* | Complementary approaches to identifying military function and recognizing multifunctional enclosure. [Springer](https://link.springer.com/article/10.1007/s10814-006-9009-0) | Diagnostic claims require contextual testing; no single feature settles every case. |
| **Ashkenazi et al. (2024), “The Roman siege system of Masada…”**, *Journal of Roman Archaeology* | Explicit geometry-to-volume-to-workload reconstruction. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-archaeology/article/roman-siege-system-of-masada-a-3d-computerized-analysis-of-a-conflict-landscape/32C59BE59ACD3E9A91C95F947DFD271E) | Workload and construction duration depend on assumptions, not surviving labor accounts. |
| **Hwaseong Seongyeok Uigwe (1801)** and the Suwon Cultural Foundation’s account | Detailed historical construction documentation. [Suwon Cultural Foundation](https://www.swcf.or.kr/english/?p=33) | Consult the original or a critical edition before importing attendance totals and translated units. |
| **Lasisi (2023), “The Archaeology of Power…”**, *Afrique: Archéologie & Arts* | Distinguishes territorial enclosure, capital defenses, and political/ritual landscape in Ijebu. [OpenEdition Journals](https://journals.openedition.org/aaa/5124) | The cited publication is a concise research summary, not a complete engineering survey. |
| **Dincecco & Onorato (2016), “Military conflict and the rise of urban Europe,”** *Journal of Economic Growth* | Quantitative urbanization evidence and a safe-haven mechanism. [Springer](https://link.springer.com/article/10.1007/s10887-016-9129-4) | Regional study; conflict exposure is not equivalent to wall construction. |

**Seshat is the most directly reusable comparative dataset here.** Turchin et al.’s 2021 military-technology study used **373 historical polities, 35 Natural Geographic Areas, and 46 binary military-technology variables**, including fortification features. Its polity/century-scale observations are suitable for technology combinations and broad adoption patterns, not daily construction rates or siege-win probabilities. Pin the research data version and retain unknown and disputed entries rather than treating them as absences. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0258161)

### Where the evidence remains thin

**Labor and maintenance are the weakest universal parameters.** Published estimates often mix different tools, materials, transportation distances, labor organizations, and workday lengths. Rebuilding episodes do not directly reveal timber service life, and standing ruins do not prove continuous military readiness.

**Siege outcomes lack a clean global denominator in the sources assembled here.** They do not support a universal probability of capture, a standard attacker-to-defender ratio, or one typical siege duration. A useful future calibration corpus would distinguish investment, bombardment, assault, negotiated surrender, relief, withdrawal, and eventual occupation.

**Survival and reporting are selective.** Stone structures, spectacular failures, and major state projects receive disproportionate attention. Treat missing wooden defenses, unrecorded repairs, and non-attacked settlements as potential gaps rather than evidence of absence.

The resulting TCE model should allow a community to build cheaply but maintain poorly, construct magnificently but provision inadequately, defend successfully but evacuate, or prosper behind walls until those same walls obstruct its growth. **Represent defendable space and time bought at continuing cost—not a single “wall level.”**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92920-c6e4-83e9-bbfe-97efdc37e09c)
