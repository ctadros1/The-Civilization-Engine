*Ōhaeawai pā, 1845: protected firing positions, flanking angles, and a substantial timber palisade illustrate why fortifications should not follow a simple “wood → stone → obsolete” progression.* [NZ History](https://nzhistory.govt.nz/media/photo/plan-ohaeawai-pa)

# Siege warfare: a simulation-ready model for TCE

## Executive recommendation

**Model a siege as a contest between two supply systems, several engineering projects, and competing political decisions—not as a wall-health bar followed by a starvation countdown.**

For TCE, the decisive variables should be:

* **Access:** which people, goods, messages, and reinforcements can still enter or leave.
* **Sustainability:** food, water, ammunition, health, money, and labor available to **both** sides.
* **Local military conditions:** whether attackers can reach, suppress, breach, and hold particular defensive positions.
* **Willingness to continue:** expectations of relief, acceptable surrender terms, command authority, and the credibility of promises.

Historical evidence supports keeping these mechanisms separate. Vicksburg survived failed assaults before surrendering after a 47-day siege. Leningrad endured catastrophic civilian mortality without being captured. Early-modern commanders sometimes chose costly attacks over slower engineering because campaign time mattered to them. None of these outcomes can be represented adequately by fortification strength alone. [National Park Service](https://www.nps.gov/vick/learn/historyculture/index.htm)

The equations and implementation rules below are **proposed TCE abstractions**, not fitted historical laws. Numerical evidence is distinguished from reconstruction and from parameters that still require calibration.

---

## 1. Mechanisms: how a siege proceeds

### 1.1 Preparation and investment: control access, not a magic perimeter

Before the enemy arrives, settlements should be able to call in troops, move provisions, admit refugees, evacuate vulnerable residents, repair defenses, and contest the approaches. Once fighting begins, these actions continue wherever routes remain open.

Represent the settlement and its surroundings as a **transport-and-control graph**. Important nodes include gates, bridges, landing places, reservoirs, wells, granaries, mills, and nearby defensive positions. Roads, canals, rivers, coastal routes, and paths are edges.

For resource \(k\) moving along route \(r\), an aggregate approximation is:

\[
Q^{\mathrm{delivered}}\_{r,k}
=
Q^{\mathrm{dispatched}}\_{r,k}(1-p^{\mathrm{intercept}}\_{r,k}),
\]

subject to route capacity, available transport, and control of intermediate nodes.

In individual-agent mode, apply interception to actual travelers or convoys. Goods must end up delivered, captured, destroyed, abandoned, or returned—not disappear through rounding.

**Important implementation consequences:**

A force outside one gate does not automatically blockade the whole settlement. A land army cannot automatically close a navigable river. Cliffs can reduce the frontage requiring surveillance. An imposing wall without guards does not reliably prevent small parties from crossing.

Masada’s archaeological reconstruction is particularly useful here: its surrounding works combined walls, camps, towers, and natural terrain. Their function was movement control and military positioning, not an uniformly impassable barrier. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-archaeology/article/roman-siege-system-of-masada-a-3d-computerized-analysis-of-a-conflict-landscape/32C59BE59ACD3E9A91C95F947DFD271E)

### 1.2 Blockade: distinguish stored supplies from continuing access

Maintain the ordinary economic balance for every important resource:

\[
S\_{k,t+1}
=
S\_{k,t}
+P\_{k,t}
+I\_{k,t}
-C\_{k,t}
-L\_{k,t},
\]

where \(P\) is production, \(I\) successful arrivals, \(C\) actual consumption, and \(L\) spoilage, destruction, or other losses. Consumption cannot exceed available stock; unmet demand must become an explicit deprivation state.

**Do this separately for households, merchants, institutions, and military stores.** A city with adequate aggregate grain may still have starving residents who cannot obtain it. Conversely, officials can keep a garrison supplied while civilian consumption collapses.

Food also requires processing and access. Grain in a warehouse is not equivalent to prepared meals when milling capacity, fuel, water, transport, or labor is missing. Livestock can provide emergency food, but slaughtering draft animals should impair transport and later cultivation.

Water requires a separate balance. Model **flow, storage, quality, and collection capacity**. An internal well is not an infinite-water bonus: its yield, maintenance, accessibility, and recharge matter. A large cistern is a finite reserve, not a renewable source.

The importance of access rather than aggregate availability is explicit in humanitarian water guidance, which distinguishes source output from household consumption and warns that queues and unequal access can leave needs unmet despite apparently sufficient supply. [AHA Centre](https://ascend.ahacentre.org/wp-content/uploads/2024/05/ADM.TEC_.028.2-WASH-Learner-Guide-2nd-Edition.pdf)

### 1.3 The besieger must also survive

Apply the same food, water, health, and transport systems to the attacking army and its followers.

Its available labor must be divided among guarding the perimeter, escorting supplies, foraging, building works, operating weapons, maintaining camp, treating casualties, and resting. Those assignments compete.

This produces several useful emergent effects:

**More soldiers need not mean faster success.** Reinforcements help only to the extent that additional combat power, surveillance, or construction capacity exceeds their logistical burden.

**Foraging should deplete a real landscape.** Nearby stores and livestock are finite. As they are exhausted, collection trips become longer and more exposed.

**Seasonality should arise from the economy.** Weather affects roads and work; harvest obligations affect mobilized farmers; fodder availability affects animals. Do not impose an universal “siege season.”

**Strategic deadlines can outweigh casualty aversion.** A commander facing an approaching relief army, expiring service obligations, or another campaign objective may attack prematurely. Ostwald’s study of the War of the Spanish Succession documents precisely this tension between methodical engineering and commanders’ preference for speed. [Google Books](https://books.google.com/books/about/Vauban_Under_Siege.html?id=-rZX8-tgjrUC)

### 1.4 Assault: calculate local access and exposure

An assault should be resolved at particular fronts, not by comparing the total attacking army with the total population behind the walls.

For each front, track:

| Component | What it controls |
| --- | --- |
| Approach geometry | How many attackers can approach and how long they remain exposed |
| Obstacle crossing | Whether ladders, gates, ramps, bridges, or breaches provide access |
| Protected firing positions | How many defenders can fight while minimizing exposure |
| Flanking positions | Whether adjacent defenders can fire into an approach or crossing |
| Reserve routes | How quickly either side can reinforce the contested point |
| Cohesion and fatigue | Whether troops continue, halt, retreat, or lose organization |
| Intelligence | What commanders believe about defenses, reserves, and damage |

The key distinction is between **soldiers present**, **soldiers able to engage**, and **soldiers able to pass through the obstacle**. Thousands of attackers behind a narrow access point are not thousands simultaneously fighting at its head.

A simple combat module can calculate expected casualties from engaged shooters, rate of attack, target exposure, protection, and ammunition, while the movement module limits access. It need not reproduce every projectile.

**A breach creates an opportunity, not automatic capture.** The breach may remain covered, obstructed, or defended from another position. At Vicksburg on 22 May 1863, Union troops briefly penetrated one defensive work, but the broader assault failed, with more than 3,000 Union casualties. That is a useful counterexample to “penetration equals victory.” [National Park Service](https://www.nps.gov/vick/learn/historyculture/secondassault.htm)

### 1.5 Engineering: model work, materials, and interference

Treat ramps, batteries, approach works, siege engines, mines, countermines, and repairs as construction projects with requirements already familiar to TCE’s building system.

For a volume-based project:

\[
\text{progress/day}
=
\min
\left(
\frac{\text{effective workers}}{\text{labor-days per m}^{3}},
\text{work-face capacity},
\text{material throughput},
\text{haulage capacity}
\right).
\]

All terms on the right must be expressed in compatible volume-per-day units.

This prevents an entire army from accelerating one narrow tunnel without limit. Soil, rock, groundwater, timber availability, tool quality, expertise, and exposure should change project requirements.

**Mining needs its own mechanism.** Its purpose is to compromise foundations or create access. Progress depends on ground conditions and knowledge of the structure; defenders can detect, counterwork, reinforce, or attack the project. Keep this at the project-and-foundation level rather than simulating detailed underground structural physics.

The Lachish reconstruction demonstrates the scale involved: the authors’ preferred reconstruction of the Assyrian ramp requires roughly 9,700 m³ of stone. It is an engineering reconstruction, not a surviving account of measured daily production. [Huji](https://huji.org.ar/wp-content/uploads/2021/11/Garfinkel-et-al.-2021-Constructing-the-Assyrian-Siege-Ramp-at-Lachish.pdf)

### 1.6 Defense is active

Defenders should choose among repairs, counterworks, reinforcement, rationing, evacuation, negotiation, and **sallies**—attacks launched out of the defended position.

Sallies can disrupt equipment, labor, or supplies, but risk losing scarce experienced defenders outside their protection. Their value therefore depends on the target and the ability to return, not a fixed daily chance of damaging enemy siege equipment.

Repair competes with combat duty and civilian necessities. A wall that can be restored overnight while the attackers struggle to obtain ammunition presents a different problem from an identical wall without labor or materials.

Medieval evidence includes both successful mining and successful resistance: Rochester fell after its defenses were undermined in 1215, while Dover resisted sieges in 1216 and 1217. These are examples of different mechanisms and outcomes, not evidence for one universal castle multiplier. [English Heritage](https://www.english-heritage.org.uk/learn/story-of-england/medieval/siege-warfare/)

### 1.7 Deprivation and disease: gradual loss of capability, unequal suffering

Connect siege conditions to TCE’s existing nutrition and disease systems rather than adding a special “siege mortality” percentage.

Food deficits should affect work capacity, fatigue recovery, resistance to illness, and eventually mortality. Water shortage and sanitation failure require separate pathways. Infection should depend on pathogens, exposure, and susceptibility—not appear simply because a siege has lasted long enough.

Keep wounded, ill, exhausted, absent, and dead soldiers distinct. A force may lose operational capacity without equivalent numbers of deaths.

Modern emergency nutrition guidance emphasizes that malnutrition and deficiencies interact with disease and can obstruct recovery long after the immediate shortage. That supports linking the siege model to persistent individual health rather than resetting survivors when hostilities end. [World Health Organization](https://www.who.int/publications/i/item/food-and-nutrition-needs-in-emergencies)

### 1.8 Negotiation and termination: use institutions, not one morale bar

Maintain separate preferences and beliefs for the commander, garrison, ruling council, local elites, and civilian groups with political influence.

A defender’s decision should compare the expected consequences of continuing with the expected consequences of surrender. Those expectations depend on remaining supplies, prospects of relief, military damage, political obligations, and whether promises will be honored.

The attacker likewise compares continued siege, assault, negotiated settlement, bypass, and withdrawal.

Allow distinct terminal outcomes:

| Outcome | Required state change |
| --- | --- |
| Negotiated capitulation | An authorized agreement is accepted and implemented |
| Capture by assault | Attackers secure sufficient positions and defeat or displace remaining resistance |
| Betrayal or internal takeover | People with actual access or authority change control |
| Breakout or evacuation | Defenders leave through a feasible route |
| Relief | Outside forces reopen access or compel the besieger to withdraw |
| Attacker abandonment | Costs, supply failure, threats, or changed objectives make continuation unattractive |
| Political settlement | A wider agreement ends the siege without necessarily transferring the city |

A settlement may surrender with supplies and walls intact. It may also continue resisting after severe civilian deprivation. **Fear of an untrustworthy victor can make resistance stronger, not weaker.**

---

## 2. Defender advantages by fortification type

**There is no defensible universal table saying “palisade = ×2, stone wall = ×5, star fort = ×10.”** Such numbers conflate weapons, garrison quality, terrain, frontage, supply, and attacker behavior.

Instead, fortifications should modify several independently measurable properties.

| Fortification type | Principal advantages | Failure modes and TCE representation |
| --- | --- | --- |
| **Ditch, bank, and palisade** | Delays approach; interrupts formations; separates attackers from defenders; can be constructed from local materials | Track ditch geometry, crossing points, timber condition, gates, visibility, and repair labor. Fire effectiveness must depend on actual conditions, not a universal timber vulnerability |
| **Thick earth or earth-cored wall** | Large mass, protected positions, and substantial material that must be displaced | Track thickness, slope, facing, drainage, foundations, and weapon-specific damage. Earth is not simply inferior masonry |
| **High masonry wall with towers** | Difficult vertical access; observation; protected firing; gates concentrate movement | Track height, usable firing positions, tower coverage, structural damage, and foundations. Damage matters where attackers can exploit it |
| **Concentric defenses or city plus citadel** | More than one defensible boundary; fallback positions; opportunities to contest captured outer works | Represent separate control zones, stores, gates, and routes. Capturing the outer town need not capture the citadel |
| **Artillery-oriented bastioned fortification** | Overlapping fire across approaches, protected gun positions, and defenses adapted to bombardment | Requires trained crews, ammunition, maintained works, and sufficient coverage. Suppressed or undermanned sectors lose much of their value |
| **Entrenched perimeter or detached forts** | Defensive depth, dispersed positions, and protected approaches between works | Represent mutually supporting nodes, communications, and gaps rather than a continuous wall object |
| **Modern urban defensive area** | Numerous buildings, covered routes, internal strongpoints, and distributed positions | Control is patchy; infrastructure and supply networks matter alongside combat positions. Capturing landmarks is not equivalent to controlling every district |

The European transition toward lower, thicker, artillery-oriented works is one historical adaptation, not a universal developmental sequence. Andrade’s comparative work emphasizes that substantial Chinese earth-based walls created different problems for artillery than relatively thin masonry defenses. His broader explanation of divergent military development should be treated as an argument to test, not a deterministic rule. [JSTOR](https://www.jstor.org/stable/j.ctvc77j74)

A particularly useful counterexample is Ōhaeawai: its approximately **3 m inner timber palisade**, protected positions, and flanking arrangements remained formidable under the artillery actually employed. The existence of cannon alone did not settle the encounter. [NZ History](https://nzhistory.govt.nz/media/photo/plan-ohaeawai-pa)

For TCE, derive the effective defender advantage from **exposure reduction, crossing throughput, fire coverage, and reserve access**, then test the resulting outcomes. Do not multiply total defender strength by a fort “level.”

---

## 3. Quantitative parameters and calibration anchors

### Evidence labels

**O:** documented observation or explicit planning standard.  
**R:** reconstruction dependent on assumptions.  
**P:** proposed simulation parameter or test setting.

Confidence concerns the stated quantity. **High confidence in a modern humanitarian standard does not imply high confidence that historical populations consumed that amount.**

### 3.1 Physical and logistical quantities

| Quantity | Value or range | Units | Evidence, confidence, and appropriate use |
| --- | --- | --- | --- |
| Population-average food-planning benchmark | **2,100** | kcal/person/day | **O; high as a humanitarian benchmark.** Not an active soldier’s requirement or a universal historical ration. A UNHCR/WFP food-assistance specification uses this balanced-basket target. [UNHCR Innovation Marketplace](https://im.unhcr.org/uga/wfp.html) |
| Basic water intake through drinking and food | **2.5–3** | L/person/day | **O; high as a planning reference, conditional on physiology and climate.** Not total water demand. [AHA Centre](https://ascend.ahacentre.org/wp-content/uploads/2024/05/ADM.TEC_.028.2-WASH-Learner-Guide-2nd-Edition.pdf) |
| Basic water including cooking and hygiene | **7.5–15** | L/person/day | **O; high as an emergency reference.** Excludes many additional livestock, institutional, and productive uses. The same guidance targets at least 15 L for household drinking, cooking, and hygiene. [AHA Centre](https://ascend.ahacentre.org/wp-content/uploads/2024/05/ADM.TEC_.028.2-WASH-Learner-Guide-2nd-Edition.pdf) |
| Masada surrounding walls, towers, and camps | **≈26,700** | m³ of stone | **R; medium.** Reconstructed original volume, not simply the surviving visible remains. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-archaeology/article/roman-siege-system-of-masada-a-3d-computerized-analysis-of-a-conflict-landscape/32C59BE59ACD3E9A91C95F947DFD271E) |
| Masada construction labor assumption | **2–3** | labor-days/m³ | **R; low–medium for transfer.** Assumes disciplined labor and locally abundant stone; unsuitable as a universal construction constant. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-archaeology/article/roman-siege-system-of-masada-a-3d-computerized-analysis-of-a-conflict-landscape/32C59BE59ACD3E9A91C95F947DFD271E) |
| Masada construction-time estimate | **11–16** with **5,000 workers** | calendar days | **R; medium under the authors’ assumptions.** Time for surrounding wall-and-camp construction, **not total siege duration**. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-archaeology/article/roman-siege-system-of-masada-a-3d-computerized-analysis-of-a-conflict-landscape/32C59BE59ACD3E9A91C95F947DFD271E) |
| Lachish assault-ramp volume | **≈9,700** | m³ | **R; medium.** Preferred geometric reconstruction; alternative reconstructions differ. Use to check the scale of construction and transport demands. [Huji](https://huji.org.ar/wp-content/uploads/2021/11/Garfinkel-et-al.-2021-Constructing-the-Assyrian-Siege-Ramp-at-Lachish.pdf) |
| Ōhaeawai inner palisade height | **≈3** | m | **O; medium–high for this case.** A geometry anchor, not the standard height of all pā or palisades. [NZ History](https://nzhistory.govt.nz/media/photo/plan-ohaeawai-pa) |

Adjust individual food and water requirements through the existing physiology model. Do not treat reduced rations as costless consumption efficiency.

### 3.2 Durations and outcomes

These are **calibration cases, not a representative global distribution**.

| Case or sample | Duration or numerical result | Outcome and interpretation | Confidence |
| --- | --- | --- | --- |
| **Syria and the Jazīra, 1097–1192** | Brosset assembled **730 sieges from 51 narratives**; his “model siege” ends within **45 days** | Successful attacker outcome through assault or surrender. **The 45 days is not a published mean, median, or maximum for all 730 events.** [Lancaster University research directory](https://research.lancaster-university.uk/en/publications/siege-warfare-in-medieval-syria-and-the-jaz%C4%ABra-1097-1192/) | Medium: thesis abstract available; underlying analysis not inspected |
| **Rochester, 1215** | **More than one month** | Capture following undermining of defenses; illustrates engineering interacting with continued resistance. [English Heritage](https://www.english-heritage.org.uk/learn/story-of-england/medieval/siege-warfare/) | Medium–high |
| **Dover, 1216 and 1217** | **Two separate unsuccessful sieges** | Useful failed-siege cases; do not combine them into one uninterrupted duration. [English Heritage](https://www.english-heritage.org.uk/learn/story-of-england/medieval/siege-warfare/) | High for outcome |
| **Vicksburg, 1863** | **47 days** | Surrender after initial assaults failed; the wider campaign lasted much longer. [National Park Service](https://www.nps.gov/vick/learn/historyculture/index.htm) | High |
| **Vicksburg assault, 22 May 1863** | **More than 3,000 Union casualties** | A costly failed assault. No casualty probability should be inferred without the relevant participating-force denominator. [National Park Service](https://www.nps.gov/vick/learn/historyculture/secondassault.htm) | Medium–high |
| **Leningrad, 1941–1944** | **Almost 900 days**; Reid estimates **700,000–800,000 civilian deaths** | The city was not captured. Food deliveries and later a land corridor changed conditions during the siege; it was not a constant, perfectly sealed blockade. [The History Reader](https://www.thehistoryreader.com/military-history/siege-leningrad-deadliest-city-blockade-human-history/) | High for broad chronology; medium for mortality estimate and boundaries |

**Do not calibrate ordinary sieges to famous multi-year cases alone.** Equally, do not prevent long sieges when partial resupply, durable defenses, political commitment, and an attacker able to remain in place make them possible.

### 3.3 Proposed sensitivity grid—not historical estimates

The following settings are useful for testing mechanisms before fitting regional historical cases. All are **P; low empirical confidence until calibrated**.

| Variable | Suggested test values | Units | Purpose |
| --- | --- | --- | --- |
| Initial usable food reserve | 14, 30, 60, 120, 240 | days of initial demand | Test dependence on preparation and season |
| Continuing food supply | 0, 0.1, 0.25, 0.5, 0.9 | share of current demand/day | Test blockade leakage and internal production |
| Attacker-to-garrison headcount | 0.5, 1, 2, 4, 8 | ratio | Test local geometry versus total numerical strength; **not an assault rule** |
| Attacker labor available for engineering | 0.25, 0.5, 0.75 | share of effective personnel | Test opportunity costs of guarding, supplying, and building |
| Exposure relative to an unprotected position | 0.1, 0.3, 0.6, 1.0 | dimensionless | Stress-test cover; eventually replace with geometry- and weapon-specific estimates |
| Defender belief that terms will be honored | 0.1, 0.5, 0.9 | probability | Test negotiation, reputation, and resistance |

In a normal simulation, stocks and labor allocations should **emerge from existing inventories and decisions**. These values belong in test scenarios, not in hidden supplies created when a siege starts.

### 3.4 Worked example: why leakage matters

For an illustrative settlement of **10,000 people**, using the 2,100 kcal benchmark gives demand of **21 million kcal/day**. Suppose usable reserves contain **1.89 billion kcal**, equivalent to 90 days at that demand.

With constant population, no spoilage, and continuing food supply equal to fraction \(s\) of demand:

\[
T\_{\mathrm{stock\ exhaustion}}=\frac{90}{1-s}.
\]

Thus:

| Continuing supply | Time until those reserves are exhausted |
| --- | --- |
| 0% of demand | 90 days |
| 25% | 120 days |
| 50% | 180 days |
| 90% | 900 days |

These are arithmetic consequences of the assumptions, **not predicted surrender dates**.

Under complete food isolation, admitting 2,500 additional people without additional provisions instead reduces the original reserve to **72 days**.

At 7.5–15 L/person/day, the original population requires **75–150 m³ of basic water daily**. A protected renewable source and a finite reservoir therefore have fundamentally different strategic implications.

---

## 4. Variation across eras and regions

Use the following as **capability combinations and environmental conditions**, not mandatory eras or civilizational bonuses.

| Setting | Historical evidence | TCE implication |
| --- | --- | --- |
| **Foragers and sedentary hunter-fishers** | Amnya in western Siberia had defensive ditches, banks, and palisades around **6000 BCE**, among hunter-gatherers. Fortification cannot be locked behind agriculture or state formation. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/worlds-oldestknown-promontory-fort-amnya-and-the-acceleration-of-huntergatherer-diversity-in-siberia-8000-years-ago/90559E4105F93528A6552B36C7236259) | Defense becomes attractive when people value a fixed location, stored resources, or infrastructure. Where relocation is cheap, flight can compete with prolonged defense |
| **Early farming communities** | Enclosure and fortification evidence requires contextual interpretation; the Amnya study explicitly challenges a simple farming-to-fortification sequence. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/worlds-oldestknown-promontory-fort-amnya-and-the-acceleration-of-huntergatherer-diversity-in-siberia-8000-years-ago/90559E4105F93528A6552B36C7236259) | Let fixed stores, fields, household buildings, and labor organization increase the value and feasibility of defense. Do not infer sophisticated siege machinery from the existence of a settlement ditch |
| **Ancient Near East** | Lachish demonstrates substantial organized siege construction. Comparative scholarship traces a repertoire including ramps, rams, mining, and other methods rather than one universal technique. [Huji](https://huji.org.ar/wp-content/uploads/2021/11/Garfinkel-et-al.-2021-Constructing-the-Assyrian-Siege-Ramp-at-Lachish.pdf) | Separate knowledge of a technique from the ability to mobilize specialists, materials, transport, and protected labor |
| **Medieval Islamic and Frankish eastern Mediterranean** | Brosset’s corpus places Muslim belligerents centrally and treats logistics and sallies alongside assault and surrender. [Lancaster University research directory](https://research.lancaster-university.uk/en/publications/siege-warfare-in-medieval-syria-and-the-jaz%C4%ABra-1097-1192/) | Avoid a model derived exclusively from western European castles or Crusader narratives |
| **China** | Andrade emphasizes large earth-based walls and a distinct interaction between fortification and gunpowder weapons. [JSTOR](https://www.jstor.org/stable/j.ctvc77j74) | Wall mass, construction, and available artillery matter more than a generic gunpowder-era breach bonus |
| **South Asia** | Rajasthan’s hill forts integrate natural terrain, inhabited areas, and extensive water-harvesting systems. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/247/) | Model protected catchments and reservoirs, difficult approaches, and internal civilian economies—not only rampart strength |
| **West Africa** | Kano’s earthen walls and gates protected a substantial urban and trading center. The UNESCO record describes successive expansion rather than one static fortification. [UNESCO World Heritage Centre](https://whc.unesco.org/en/tentativelists/5171/) | Earth construction and gate-controlled commerce belong in the main fortification system. Closing access also interrupts livelihoods and revenue |
| **Mesoamerica** | Tenochtitlan’s island setting included **three causeways and two aqueducts**. Its urban defenses cannot be understood through a continuous-wall model alone. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/tenochtitlan) | Lakes, causeways, boat transport, and water infrastructure must be first-class strategic features |
| **Māori warfare** | Ōhaeawai combined timber defenses with protected firing and flanking arrangements against firearm-equipped attackers. [NZ History](https://nzhistory.govt.nz/media/photo/plan-ohaeawai-pa) | Permit rapid adaptation using local materials; do not require expensive masonry for an effective defense against guns |
| **Early-modern European states** | Formal engineering systems coexisted with commanders who departed from prescribed methods. [Google Books](https://books.google.com/books/about/Vauban_Under_Siege.html?id=-rZX8-tgjrUC) | Professional engineers offer plans and forecasts; institutions and commanders decide whether to follow them |
| **Industrial warfare** | Vicksburg’s prepared earthworks resisted assaults despite extensive bombardment and numerical pressure. [National Park Service](https://www.nps.gov/vick/learn/historyculture/vicksburgsiege.htm) | Firepower can strengthen field defense as well as attack. Rail, river transport, ammunition, and entrenched positions belong in the same logistical model |
| **Modern warfare** | The ICRC identifies loss of electricity, water treatment, sewage systems, and medical services as major siege consequences. [ICRC](https://www.icrc.org/en/document/protection-civilian-population-during-sieges-what-law-says) | Add infrastructure dependencies and distributed urban defense without replacing the underlying access-and-supply system |

This allows the same world to contain an earth-walled commercial city, a water-secure hill fortress, a lightly defended agricultural town, and a sophisticated field fortification simultaneously.

---

## 5. Civilians, surrender, sack, and recovery

### 5.1 Civilian experience should emerge through households

A city-wide food total is insufficient for TCE’s visible daily life. Track which households can buy food, draw rations, collect water, access shelter, or obtain care.

Proposed behavior rules should allow people to change consumption, sell assets, borrow, seek patronage, share with kin, hide stocks, steal, relocate within the city, or attempt to leave. None should be triggered for everyone simultaneously.

Refugees bring both needs and capabilities. Some bring provisions, money, skills, or livestock; others arrive without resources. Assigning every refugee a fixed negative modifier misses those differences.

The central distributional question is **who controls scarce necessities and who has a recognized claim to them**. For validation, compare nutrition and survival across wealth, occupation, household structure, and political access—not just average consumption.

### 5.2 Make surrender a contract

Historical capitulation and the treatment of stormed places operated within particular legal and customary traditions. Daly’s study of western Europe from 1660–1815 describes highly regularized siege customs alongside claims that storming a town permitted killing its garrison and sacking it. That regional historical convention must not become a universal or automatic rule. [Cambridge University Press](https://www.cambridge.org/core/books/civility-barbarism-and-the-evolution-of-international-humanitarian-law/sieges-and-the-laws-of-war-in-europes-long-eighteenth-century/C25E3D38894DB100CAEE8077DF4D99DC)

A TCE surrender agreement should specify, as applicable:

| Contract dimension | Possible negotiated provisions |
| --- | --- |
| Military personnel | Departure, disarmament, captivity, exchange, or service obligations |
| Political authority | Annexation, tribute, retained local institutions, or replacement of officials |
| Property and livelihoods | Protection, confiscation, indemnities, requisitions, or taxation |
| Civilian safety | Protected residence, evacuation, food access, and family unity |
| Enforcement | Hostages, guarantees, staged transfer of gates, witnesses, or third-party mediation |

These are proposed composable fields, not a claim that every historical society used the same treaty menu.

Implementation matters after acceptance. A commander may promise protection but lack control over troops. Individual units may obey, loot, retaliate, or be restrained. Reputation should update from the actual treatment of people, not merely from the signed agreement.

### 5.3 Do not use one “sacked city” percentage

Represent aftermath through separate events affecting real people and assets: killing, injury, imprisonment, displacement, looting, burning, destruction of equipment, and—where the simulated institutions practice it—enslavement.

Keep **physical destruction**, **population loss**, **political replacement**, and **economic disruption** separate. A settlement can change rulers with limited damage, suffer destructive fighting without wholesale demographic replacement, or lose its economic base even when many buildings remain.

Tenochtitlan provides an extreme reconstruction case: the Metropolitan Museum’s account describes the devastated city being razed after its fall and Mexico City built over it. That is a valid possible trajectory, not the default aftermath of capture. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/tenochtitlan)

Recovery should follow surviving assets and institutions. Missing seed, draft animals, tools, skilled workers, transport, or secure property claims can disrupt later production. Food imports and restored water may improve survival immediately while housing and economic recovery take much longer.

### 5.4 Modern humanitarian law is a separate institutional layer

For modern-capability societies, distinguish what the engine can physically simulate from what institutions permit. Current international humanitarian law prohibits starvation of civilians as a method of warfare; civilians who remain in a besieged area retain protection, and obligations concerning evacuation and humanitarian relief continue. A claim of targeting the garrison does not justify indiscriminate deprivation of civilians. [ICRC](https://www.icrc.org/en/document/protection-civilian-population-during-sieges-what-law-says)

---

## 6. Stylized facts a correct simulation should reproduce

| Pattern | Validation test |
| --- | --- |
| **Duration is not determined by wall strength alone** | Hold geometry constant and vary supplies, leakage, relief prospects, and attacker endurance. Outcomes should diverge substantially |
| **Short and prolonged sieges coexist** | Do not fit every siege to spectacular long cases; equally, sustained partial resupply must permit long resistance. The regional medieval corpus and Leningrad should not require contradictory engine rules. [Lancaster University research directory](https://research.lancaster-university.uk/en/publications/siege-warfare-in-medieval-syria-and-the-jaz%C4%ABra-1097-1192/) |
| **Assault losses can arrive in sharp bursts** | A failed assault can cause much greater one-day losses than preceding days of blockade; Vicksburg supplies a documented example. [National Park Service](https://www.nps.gov/vick/learn/historyculture/secondassault.htm) |
| **A damaged or penetrated defense can remain militarily effective** | Capturing a crossing or outer work must not automatically transfer the settlement |
| **Severe civilian mortality need not cause military capitulation** | Civilian survival, garrison capability, and political decisions must be distinct; Leningrad is a demanding test case. [The History Reader](https://www.thehistoryreader.com/military-history/siege-leningrad-deadliest-city-blockade-human-history/) |
| **Terrain and materials interact with weapons** | A well-designed timber-and-earth defense can outperform an unsuitable masonry one against the weapons actually available. [NZ History](https://nzhistory.govt.nz/media/photo/plan-ohaeawai-pa) |
| **Construction time depends on mobilization and throughput** | Large numbers of workers help large projects, but work-face and transport constraints create diminishing returns |
| **Occupation does not reset the city** | Survivors retain injuries, debts, losses, displacement histories, and relationships; destroyed productive assets continue to matter |

Also check the model’s **negative predictions**: an army with no feasible supply system should not maintain an indefinite blockade, and a settlement with an uncontrolled high-capacity supply route should not experience the same deprivation as a completely isolated one.

---

## 7. Recommended implementation for 10k–50k agents

### 7.1 Add a siege coordinator, not a separate miniature economy

The siege system should coordinate existing systems rather than replace them.

| Layer | Minimum useful state |
| --- | --- |
| **Individuals and households** | Location, role, stocks or entitlements, health, fatigue, dependents, relationships, displacement |
| **Military units** | Actual members, assigned front, equipment, ammunition, cohesion, orders, supply claims |
| **Institutions** | Store ownership, ration policy, command authority, requisition powers, negotiation authority, applicable norms |
| **Spatial defense graph** | Walls, crossings, firing positions, access routes, internal strongpoints, control |
| **Engineering projects** | Required materials, labor, specialist needs, remaining work, exposure, interference |
| **Siege-level state** | Participants, objectives, blockade routes, offers, relief information, beliefs, event chronology |

Keep the military supplied through actual economic transfers. Requisitioning food should remove it from someone’s inventory and create political or financial consequences.

### 7.2 Use multiple update scales

A reasonable starting architecture is:

**Daily economic and household updates** for food balances, ration distribution, work allocation, health progression, and political reassessment.

**Event-driven route updates** when control, weather, transport availability, or orders change.

**Short tactical steps during active combat**, with squads or front-level cohorts for engagement calculations and casualties assigned to their actual constituent people.

These are engineering recommendations, **not measured performance claims**. Profile the Rust implementation before fixing tick lengths.

Avoid recomputing paths for every person every frame. Cache route restrictions and invalidate affected paths when a gate closes, a breach opens, or a district changes control. Keep detailed combat processing localized to active fronts.

### 7.3 Simplify geometry, not conservation or agency

For v1, omit full structural finite-element analysis, individual projectile simulation, and continuously simulated underground tunnels.

Retain wall height and thickness, materials, foundations, ditch dimensions, protected positions, access throughput, and a small set of damage states. A “breach” should alter traversability and exposure in both the kernel and the renderer.

Offscreen or fast-forward resolution should use the **same resource balances, project progress, and decision rules**. It may aggregate computations, but should not substitute a separate outcome table.

Do not scale a historical siege’s duration linearly to TCE’s smaller population. Preserve per-person needs, resource flows, and relevant geometry. A smaller city also has a different perimeter-to-population relationship.

### 7.4 Make beliefs imperfect

Commanders should receive reports with delay and uncertainty about enemy supplies, casualties, defensive layouts, and relief forces.

Track true state separately from believed state. Otherwise the AI will surrender at mathematically optimal moments and attack precisely when success is certain—removing much of the historical uncertainty the model is intended to reproduce.

### 7.5 Dashboard outputs

The most informative dashboard is not “siege progress: 63%.” It should show:

| Domain | Suggested measurements |
| --- | --- |
| Supply | Food reserve at current consumption; successful inbound flow; attacker supply endurance |
| Distribution | Nutrition and water access by household group; military versus civilian allocation |
| Military | Effective personnel; local access points; ammunition; defensible fallback positions |
| Engineering | Work remaining, throughput constraints, repair-versus-damage balance |
| Politics | Pending offers, authority disputes, perceived relief prospects, confidence in guarantees |
| Human consequences | Deaths by cause, wounded, displaced, captive, and missing people |
| Recovery | Housing, water, seed, livestock, tools, labor, and market access remaining |

### 7.6 Existing models and games worth borrowing from

| Model or game | Useful element | What not to inherit |
| --- | --- | --- |
| **Masada 3D construction analysis** | Converts geometry into material quantities and labor requirements | Its inferred work rates and completion times are assumptions to test, not observed universal constants. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-archaeology/article/roman-siege-system-of-masada-a-3d-computerized-analysis-of-a-conflict-landscape/32C59BE59ACD3E9A91C95F947DFD271E) |
| **Stronghold / Stronghold HD** | Connects visible construction, food production, civilian rationing, and siege equipment | The manual explicitly excludes soldiers from granary rations, abstracting their food into recruitment expenditure. TCE should instead preserve a physical military supply chain. [Steam CDN](https://cdn.akamai.steamstatic.com/steam/apps/40950/manuals/Stronghold%20HD%20Manual%20-%20English.pdf?t=1430150646) |
| **This War of Mine** | Centers civilian scarcity, household survival, shelter, and difficult resource choices | It is not a complete model of urban provisioning, fortification, or institutional capitulation. Use it as a reference for civilian consequences, not siege statistics. [Steam Store](https://store.steampowered.com/app/282070/This_War_of_Mine/) |

**Recommended v1 priority:** access control, two-sided logistics, local assault geometry, labor-based engineering, and institutional surrender. These provide the largest causal return before detailed siege-engine varieties or elaborate structural damage.

---

## 8. Sources, datasets, and evidence limits

### Core scholarly sources

| Source | Best use | Important limitation |
| --- | --- | --- |
| **Thomas Brosset, *Siege Warfare in Medieval Syria and the Jazīra (1097–1192)* (PhD, 2025)** | Large regional event corpus and alternative to a Europe-centered narrative | Full thesis embargoed until **17 June 2030**; only its abstract was available here. No independently recomputed duration distribution is presented. [Lancaster University research directory](https://research.lancaster-university.uk/en/publications/siege-warfare-in-medieval-syria-and-the-jaz%C4%ABra-1097-1192/) |
| **Jamel Ostwald, *Vauban under Siege* (Brill, 2007)** | Systematic comparison of prescribed siegecraft with operational practice | Regional and period-specific; an inventory of early-modern sieges is not a global sample. [Google Books](https://books.google.com/books/about/Vauban_Under_Siege.html?id=-rZX8-tgjrUC) |
| **Hai Ashkenazi et al., “The Roman siege system of Masada: a 3D computerized analysis of a conflict landscape” (2024)** | Geometry, surrounding works, construction quantities, and labor reconstruction | Work rates and original dimensions contain assumptions. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-archaeology/article/roman-siege-system-of-masada-a-3d-computerized-analysis-of-a-conflict-landscape/32C59BE59ACD3E9A91C95F947DFD271E) |
| **Yosef Garfinkel et al., “Constructing the Assyrian Siege Ramp at Lachish” (2021)** | Integration of archaeology, photogrammetry, texts, and iconography | Alternative ramp reconstructions exist; reconstructed building time is not chronicle-observed time. [Huji](https://huji.org.ar/wp-content/uploads/2021/11/Garfinkel-et-al.-2021-Constructing-the-Assyrian-Siege-Ramp-at-Lachish.pdf) |
| **Henny Piezonka et al., “The world’s oldest-known promontory fort…” (2023)** | Hunter-gatherer fortification and non-linear technological/social development | Archaeological defenses do not supply an event-level siege-duration distribution. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/worlds-oldestknown-promontory-fort-amnya-and-the-acceleration-of-huntergatherer-diversity-in-siberia-8000-years-ago/90559E4105F93528A6552B36C7236259) |
| **Tonio Andrade, *The Gunpowder Age* (Princeton, 2016)** | China–Europe comparison and weapon–fortification interaction | Broad causal interpretation should not become a deterministic technology rule. [JSTOR](https://www.jstor.org/stable/j.ctvc77j74) |
| **Gavin Daly, “Sieges and the Laws of War in Europe’s Long Eighteenth Century” (2024)** | Surrender customs, sack, and civilian exposure | Western European scope; not a universal law of historical warfare. [Cambridge University Press](https://www.cambridge.org/core/books/civility-barbarism-and-the-evolution-of-international-humanitarian-law/sieges-and-the-laws-of-war-in-europes-long-eighteenth-century/C25E3D38894DB100CAEE8077DF4D99DC) |
| **Anna Reid, *Leningrad* (2011)** | Civilian experience, unequal access, mortality, and changing supply conditions | Mortality totals depend on sources and population boundaries. [The History Reader](https://www.thehistoryreader.com/military-history/siege-leningrad-deadliest-city-blockade-human-history/) |

### What remains thin or contested

**A global “average siege duration,” universal attacker success probability, and universal sack mortality rate are not supported by the evidence assembled here.** The useful quantities are conditional: on place, period, objective, supplies, defenses, and how the event is defined.

For a TCE calibration dataset, record the first hostile action, start of effective supply disruption, start of formal engineering, interruptions, capitulation, and final evacuation separately. Distinguish a long campaign containing several siege episodes from one continuous investment.

Record casualty categories and denominators explicitly. Dead, wounded, prisoners, missing soldiers, and civilian excess deaths are not interchangeable.

Keep source uncertainty in the data: date intervals, force-size ranges, competing accounts, and reconstruction assumptions. Do not silently turn narrative estimates into exact observations.

Finally, treat the model as a set of **testable mechanisms**. The strongest target is not reproducing one famous siege exactly; it is producing plausible short capitulations, failed blockades, costly assaults, prolonged partially supplied resistance, and varied civilian aftermaths **from the same conserved resources, spatial rules, and institutions**.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9295d-4c00-83ea-8c77-21314292ffc3)
