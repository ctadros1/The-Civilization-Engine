# Military technology through history: a simulation-ready model for TCE

## Executive conclusion

**Model military technology as a system of complementary capabilities, not a ladder of increasingly powerful units.** A weapon’s practical value depends on who can make it, who can use it, what ammunition and maintenance it requires, and the terrain, organization, and opposition it encounters.

The historical record repeatedly breaks simple technology trees. Fortified settlements existed among hunter-gatherers; cavalry existed centuries before stirrups; firearms and substantial armor coexisted for centuries; and mechanical siege engines were not simply weaker versions of cannon. These are important constraints on TCE’s prerequisite logic and combat model. [DOI](https://doi.org/10.15184/aqy.2023.164)

For TCE, separate three kinds of change:

| Kind of change | What changes in the simulation | Example |
| --- | --- | --- |
| **Material and equipment capability** | Available goods and their physical properties | Mail, crossbows, cannon, armored vehicles |
| **Military organization and skill** | What people can do together | Maintaining formations, coordinating mounted and foot troops, operating artillery |
| **Sustained operational capacity** | How many equipped people can remain effective, where, and for how long | Remount systems, ammunition supply, arsenals, transport and repair |

The recommended model below keeps those layers independent but interacting.

**Evidence convention:** **H** means strong evidence for the particular observation; **M** means reconstruction, contextual interpretation, or substantial variation; **L** means uncertain dating or weak generalizability. **Design prior** means a proposed TCE tuning value—not a measured historical parameter.

---

## 1. Mechanisms: rules the simulation should implement

### 1.1 Knowledge, manufacture, possession, and effective use are different states

A society may obtain weapons through purchase or capture without being able to manufacture them. Conversely, a workshop may understand a design but lack materials, skilled assistants, or customers. Korean matchlock history provides a particularly useful example of technology moving through trade, warfare, and local adaptation rather than a single national invention-to-adoption sequence. [American Society of Arms Collectors](https://americansocietyofarmscollectors.org/wp-content/uploads/2022/06/The-Korean-snap-matchlock-a-global-microhistory-v124-Kang.pdf)

**TCE rule:** track six separate states:

`known → locally producible → stocked → issued → competently operated → sustainably supplied`

These are not necessarily traversed in order: imports can create stocked and issued equipment before local production becomes possible.

A technology should therefore unlock a **recipe or capability**, not automatically replace every soldier’s equipment. Procurement, distribution, training, and retirement of old equipment should remain economic and institutional processes.

### 1.2 Improvements usually redistribute costs rather than eliminate them

Weapons can economize on one input while demanding more of another. Composite bows combine several materials and specialized craftsmanship; early firearms depend on different craft and ammunition systems. Qin crossbow triggers show organized batch production, but that is not the same thing as modern universal interchangeability. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/555921)

**TCE rule:** evaluate equipment through several costs:

\[
C\_{\text{fielded}} =
C\_{\text{equipment annualized}}
+C\_{\text{training}}
+C\_{\text{pay or maintenance}}
+C\_{\text{consumables}}
+C\_{\text{transport}}
+C\_{\text{repair}}
+C\_{\text{lost civilian production}}
\]

Do not select the “best” weapon by purchase price or damage alone. A force can be affordable to assemble but unaffordable to keep campaigning.

### 1.3 Training is partly inherited from civilian life

Mounted archery illustrates the importance of childhood experience and habitual practice. Historical descriptions of Mongol life connect riding and archery to early upbringing, not merely a short period of formal military instruction. That does not imply an ethnic ability: it identifies a reproducible training environment. [Royal Armouries](https://royalarmouries.org/objects-and-stories/stories/the-mongols/arms-and-armour-of-the-mongol-empire)

**TCE rule:** military skills draw on existing individual attributes:

* Hunting contributes to archery, tracking, and judging distance.
* Herding and riding contribute to mounted competence.
* Craft work contributes to equipment maintenance.
* Collective exercises contribute to formation control and coordinated action.

Keep **weapon handling**, **physical conditioning**, **formation discipline**, and **combat experience** separate. Learning to discharge a firearm is not equivalent to becoming a reliable infantry soldier.

### 1.4 Armor changes injury probability, fatigue, and coverage—not hit points

European field plate was articulated equipment in which wearers could move, rather than an immobilizing shell. Experimental work nevertheless demonstrates substantial locomotion costs. Separate experiments with a Dendra-style bronze armor reconstruction also show why “heavy” does not automatically mean unusable, although reconstructed combat protocols cannot establish actual ancient battlefield performance. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/arms-and-armor-common-misconceptions-and-frequently-asked-questions)

**TCE rule:** give armor:

`coverage by body zone × protection against threat type × condition × fit`

Then separately apply its mass, distribution, and thermal burden to fatigue.

A protected hit can produce no injury, a minor injury, or incapacitation depending on the attack and exposed location. Avoid one universal “armor percentage.” Modern protective-equipment standards likewise distinguish threats rather than treating protection as a single scalar. [CJTTEC](https://cjtec.org/compliance-testing-program/for-law-enforcement/body-armor-overview-and-history/)

### 1.5 Cavalry is a collection of capabilities, not a universal infantry counter

Mounted warfare predates paired stirrups. Saddle and stirrup developments improved the rider–horse interface, but the archaeological chronology does not support making stirrups a prerequisite for cavalry itself. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/origins-of-saddles-and-riding-technology-in-east-asia-discoveries-from-the-mongolian-altai/95BA971FD64B2A7544D4BEF6694A8E14)

**TCE rule:** distinguish mounted scouting, pursuit, raiding, mounted missile use, close combat, transport followed by dismounting, and mounted command.

Their effectiveness should depend on horse condition, rider skill, footing, visibility, formation coherence, and the opposing formation’s state. A cavalry unit should not receive an unconditional bonus against every foot soldier.

Chariots should be a **parallel vehicle branch**: wheels, terrain suitability, draught animals, crew coordination, and maintenance. They should not be a compulsory ancestor of riding.

### 1.6 Fortifications change access, exposure, and time

Bastioned defenses were designed around fields of fire and the reduction of vulnerable approaches—not simply taller or stronger walls. Māori gunfighter pā demonstrate another response to artillery: protective earthworks and sheltered positions rather than imitation of European masonry fortresses. [English Heritage](https://www.english-heritage.org.uk/visit/places/harrys-walls/history/)

**TCE rule:** represent fortifications through:

`obstacle depth, height, material, local thickness, ditch, gates, firing positions, covered approaches, shelter, garrison, water, food`

Fortification strength should therefore be partly geometric and partly organizational. An elaborate perimeter with too few defenders is not equivalent to a fully manned one.

### 1.7 A siege is several simultaneous contests

The Assyrian siege ramp at Lachish demonstrates the importance of engineering work. Research on trebuchets also cautions against assuming that every large stone-thrower was an efficient masonry-demolition machine. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ojoa.12231)

**TCE rule:** run separate processes for blockade, engineering access, bombardment, structural damage, suppression, repair, assault readiness, sorties, relief, and negotiated surrender.

The attacker and defender both consume supplies. A breach is an opportunity, not an automatic capture. A fortress can fall without a breach, and an attacker can abandon a siege despite possessing superior weapons.

### 1.8 Industrial firepower creates new bottlenecks

First World War weapons combined much higher firing capacities with enormous ammunition, transport, maintenance, and coordination demands. Early tank wireless experiments also show that possessing communication hardware did not immediately produce reliable communication procedures. [National Army Museum](https://www.nam.ac.uk/explore/weapons-western-front)

**TCE rule:** more rapid fire must increase ammunition consumption and dependence on replenishment. Better communications should reduce some delays and uncertainty only when trained operators, compatible equipment, and functioning procedures exist.

---

## 2. Parameters: what can be quantified defensibly?

### 2.1 Empirical and reconstructed anchors

These are **specific observations**, not universal values for entire eras.

| Parameter | Value and units | Appropriate TCE use | Evidence and confidence |
| --- | --- | --- | --- |
| Complete European **field plate armor** | **20–25 kg** | Equipment-mass benchmark; not total campaign load | Met curatorial synthesis. **H** for typical category, **M** across examples. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/arms-and-armor-common-misconceptions-and-frequently-asked-questions) |
| Associated helmet mass | **2–4 kg** | Head protection versus load | Same source; distinguish field from specialized tournament equipment. **H/M**. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/arms-and-armor-common-misconceptions-and-frequently-asked-questions) |
| Net energy cost of walking in tested armor | **2.1–2.3×** unarmored walking | Fatigue calibration for comparable equipment and movement | Askew, Formenti and Minetti. **H** for experiment; **M/L** for broad extrapolation. Not a movement-speed multiplier. [UNIFIND](https://expertise.unimi.it/resource/item/182803) |
| Net energy cost of running in tested armor | **1.9×** unarmored running | Separate running fatigue from walking fatigue | Same experiment and limitations. [UNIFIND](https://expertise.unimi.it/resource/item/182803) |
| Greek infantry spear example | **8–10 ft**, approximately **2.4–3.0 m** | Reach and formation-space benchmark | Met account of ancient Greek equipment. **M** across periods and formations. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/warfare-in-ancient-greece) |
| Mary Rose longbow length | **1.839–2.113 m** | Physical variation within one dated equipment assemblage | Surviving artifacts from the ship lost in 1545. **H**. [Mary Rose](https://maryrose.org/discover/collections/the-weaponry-of-the-mary-rose/longbows-and-arrows/) |
| Mary Rose reconstructed draw weight | **65–175 lbf**, approximately **289–778 N**, on the museum’s current collection page | Strength requirement and equipment variation—not direct damage | Working-copy reconstruction, not measurement of original operating force. Other reconstructions differ. **M/L**. [Mary Rose](https://maryrose.org/discover/collections/the-weaponry-of-the-mary-rose/longbows-and-arrows/) |
| Brown Bess trained firing rate | Approximately **3 shots/minute** | Upper practical cadence benchmark for this weapon context | Regimental-museum interpretation. Battle cadence varies. **M**. [Age of Revolution](https://ageofrevolution.waterlooassociation.org.uk/200-object/musket-brown-bess/) |
| Brown Bess individual aiming | Difficult beyond approximately **100 m** | Shape a hit-probability curve; do not use as a hard projectile limit | Same source. Formation targets differ from individuals. **M**. [Age of Revolution](https://ageofrevolution.waterlooassociation.org.uk/200-object/musket-brown-bess/) |
| SMLE trained firing rate | Approximately **15 rounds/minute** | Trained rifleman benchmark, before supply and suppression constraints | National Army Museum. **M** as an operational parameter. [National Army Museum](https://www.nam.ac.uk/explore/weapons-western-front) |
| Vickers machine-gun mechanical rate | **Over 600 rounds/minute** | Mechanical ceiling only | Not indefinitely sustainable battlefield expenditure. **H/M**. [National Army Museum](https://www.nam.ac.uk/explore/weapons-western-front) |
| Horse daily dry-matter intake | Approximately **1.5–3% of body mass/day** | Feed budget, modified by work, climate, and feed quality | Equine husbandry guidance; physiological anchor rather than historical ration record. **H/M**. [Utah State University Extension](https://extension.usu.edu/equine/research/caring-for-horses-in-cold-weather) |
| English army contracted daily pay, 1415 | Archer **6d/day**; man-at-arms **12d/day** | Contextual **2:1 base-pay ratio** | Indenture evidence. Excludes equipment, horses, retinues, and other costs. **H** for record; **L** as a universal ratio. [Agincourt 600](https://www.agincourt600.com/2015/03/24/27-april-3-may-1415-further-military-preparations-and-indentures/) |
| English knight’s equipment expenditure, 1374 | Entire armory **over £16**, compared with **over three years of a skilled worker’s wages** | Demonstrate possible capital burden | **Not the price of one suit of armor.** Context-specific record. **H/M**. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/arms-and-armor-common-misconceptions-and-frequently-asked-questions) |
| Fort Pulaski bombardment, 1862 | Masonry breached during approximately **30 hours** of bombardment | One validation scenario for masonry versus rifled artillery | Do not convert into a universal siege duration. **H** for event. [National Park Service](https://home.nps.gov/places/the-breeched-corner.htm) |

**Important distinction:** the approximately fivefold difference between the cited musket and rifle firing rates is **not evidence for fivefold combat effectiveness**. Accuracy, exposure, ammunition, formation, cover, and morale intervene.

#### A useful cavalry logistics calculation

For an explicitly assumed force of **1,000 riders**, **two horses per rider**, **400 kg per horse**, and dry-matter intake of **2% of body mass/day**:

\[
1{,}000 \times 2 \times 400 \times 0.02
=16{,}000\text{ kg dry matter/day}.
\]

That is **16 tonnes daily**, or **480 tonnes over 30 days**. This is a derived scenario, not a historical army ration record. Grazing can provide some intake, but doing so requires accessible forage and time; fresh-grass weight is not the same as dry-matter weight. The intake assumption lies within the husbandry range above. [Utah State University Extension](https://extension.usu.edu/equine/research/caring-for-horses-in-cold-weather)

### 2.2 Training parameters: explicitly proposed design priors

Reliable universal “hours to train a medieval soldier” figures do not exist in the evidence assembled here. Use the following only as **initial tuning ranges**, with separate instruction, conditioning, and collective practice.

| Competence | Initial TCE design prior | What completion should mean |
| --- | --- | --- |
| Basic spear-and-shield handling | **40–120 supervised hours** | Competent handling; not dependable formation behavior |
| Basic formation practice | **80–240 additional hours** | Improved spacing, response to commands, and cohesion |
| Crossbow handling | **40–120 hours** | Reliable operation and basic aiming; accuracy continues developing |
| Early firearm handling | **40–120 hours**, plus collective drill | Reliable routine handling; does not confer battle steadiness |
| Heavy-bow proficiency from an inexperienced baseline | **300–1,000+ practice hours**, plus conditioning over elapsed time | An intentionally expensive skill investment |
| Mounted military competence | **300–1,000 hours of riding experience**, plus **80–300 hours** of role-specific practice | Civilian riding can satisfy much of the first component |
| Routine artillery crew duties | **80–300 hours**, under qualified specialists | Crew competence, not qualification as an engineer or master gunner |

**All numerical ranges in this table are design priors, with low empirical confidence.** Run sensitivity tests at least at half and twice the baseline. More importantly, make proficiency continuous rather than issuing an “expert soldier” certificate when a counter reaches a threshold.

### 2.3 Cost should come from production chains

For a good \(g\), calculate local production cost from:

\[
C\_g=\sum\_i q\_i p\_i+\sum\_j h\_j w\_j+C\_{\text{fuel}}+
C\_{\text{workshop}}+C\_{\text{rejects}}.
\]

Here \(q\_i\) is an input quantity, \(p\_i\) its local price, \(h\_j\) labor hours by skill, and \(w\_j\) the corresponding labor cost.

Keep **active labor**, **elapsed production time**, and **capital tied up in production** separate. A long commission or curing period is not equivalent to continuous labor by one worker. Historical armor production also varied between individual commissions, specialized workshops, and cheaper equipment markets. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/arms-and-armor-common-misconceptions-and-frequently-asked-questions)

For weapons, armor, and vehicles, TCE should account for repair, reuse, resizing where possible, capture, resale, and salvage. These are proposed economic mechanisms; they prevent military procurement from becoming a one-time technology fee.

---

## 3. Technology graph: nodes, prerequisites, and concrete unlocks

### How to read the graph

The dates below are **approximate first-appearance ranges or explicitly identified secure benchmarks**. They are not adoption dates for the entire world. Where origins are poorly established, the table says so rather than supplying false precision.

The prerequisites are **recommended TCE dependencies**. “AND” means a required combination; “OR” means an alternative route. Existing materials, textiles, transport, and information nodes should be reused rather than duplicated inside the military branch.

Importantly, prerequisites below govern **local production or institutional reproduction**, not possession of imported equipment.

### 3.1 Weapons and protection

| Node | Historical appearance or benchmark | Recommended prerequisites | Unlocks and modeled consequences |
| --- | --- | --- | --- |
| **W01 — Basic hafted and hand weapons** | Deep prehistory; no single origin appropriate. Treat as starting capabilities rather than discoveries scheduled after farming. | Woodworking; optional lithic or bone working and binding | Clubs, thrusting spears, throwing weapons; low-complexity workshop recipes. Separate reach, handling, and material quality. |
| **W02 — Bow-and-arrow systems** | Evidence interpreted as bow-and-arrow use at Fa-Hien Lena, Sri Lanka, approximately **48,000 years ago**; not a demonstrated first global invention or first military use. **M**. [PubMed](https://pubmed.ncbi.nlm.nih.gov/32582854/) | Suitable wood or alternative bow materials AND cordage AND projectile manufacture | Bows, arrows, bowyer/fletcher roles, hunting and ranged-combat capabilities. |
| **W03 — Nonmetal shields and body protection** | Origins unresolved; multiple independent traditions. Preserve this branch without requiring metallurgy. | Woodworking OR hide working OR suitable textile construction | Shields and protective garments. Coverage, durability, heat burden, and occupied hands vary by recipe. |
| **W04 — Cast copper-alloy weapons** | Late **fourth millennium BCE**, Near East; Arslantepe provides an early sword assemblage. **M/H** for context, not an exclusive invention claim. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1622) | Copper metallurgy AND casting; alloying as available | Cast spearheads, axes, and blades; weapon casting and finishing recipes. Do not make every copper-alloy weapon “tin bronze.” |
| **W05 — Forged iron and steel weapons** | Major proliferation during the late second and first millennia BCE in western Eurasia; regional pathways and African chronologies differ. **M**. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0258161) | Iron production OR imported iron AND forging; steel-quality processes are additional nodes | Iron weapon variants and repair. Do not impose a mandatory local Bronze Age first or a universal damage increase. |
| **W06 — Composite bows** | Earlier origins debated; surviving Egyptian example dated approximately **1550–1400 BCE** is a secure benchmark. **H** for object. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/555921) | Bowmaking AND suitable composite materials AND adhesive craft | Compact composite-bow variants; specialized production and environmental sensitivity parameters. No requirement for metalworking. |
| **W07 — Crossbows and mechanical release systems** | First-millennium-BCE China; organized large-scale production securely represented by the **third-century-BCE Qin** assemblage. **H** for benchmark. [DOI](https://doi.org/10.1017/s0003598x00050262) | Bow mechanics AND woodworking AND suitable trigger manufacture | Crossbows, bolts, trigger-making and maintenance; separate spanning effort, holding/aiming, and reload cadence. |
| **W08 — Rigid metal body protection** | Dendra bronze armor, Greece, **fifteenth century BCE**, is an important surviving benchmark. **H** for artifact context. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0301494) | Sheet-metal work AND fastening AND fitted support garments | Bronze plate and other rigid protection recipes. Scale, lamellar, and large plates should be parallel constructions, not compulsory successive levels. |
| **W09 — Mail armor** | Approximately **third century BCE** in the European record; precise origin attribution remains less certain. **M**. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/32862) | Metal forming AND ring manufacture/joining AND skilled assembly | Mail garments and later gap protection; flexible coverage with substantial assembly and repair requirements. |
| **W10 — Coats of plates and brigandine-type protection** | European development particularly **thirteenth–fourteenth centuries CE**. **M**. [The Metropolitan Museum of Art](https://www.metmuseum.org/pt/essays/arms-and-armor-in-medieval-europe) | Plate manufacture AND fastening AND garment construction | Small-plate protective garments; an alternative to, not merely an inferior prerequisite for, full articulated plate. |
| **W11 — Articulated full plate** | Developed European field harness by approximately **1420–1430 CE**. **H/M**. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/arms-and-armor-common-misconceptions-and-frequently-asked-questions) | High-quality plate work AND articulation/joints AND fitting expertise | Fitted harness; approximately 20–25 kg field-equipment benchmark. Price, coverage, repair, and fatigue remain separate. |
| **W12 — Standardized military batch production** | Qin trigger production, **third century BCE**, provides a strong organizational benchmark. **H**. [DOI](https://doi.org/10.1017/s0003598x00050262) | Workshop coordination AND measurement practices AND inspection | Arsenal production batches, quality distributions, parts families, procurement contracts. Do not equate standardization with universal interchangeability. |

**Weapon families such as swords, axes, maces, long spears, and specialized polearms should mostly be recipes and equipment variants**, not separate civilizations-wide discoveries. Their relative utility should emerge from reach, handling, protection encountered, and formation constraints.

### 3.2 Mounted warfare

| Node | Historical appearance or benchmark | Recommended prerequisites | Unlocks and modeled consequences |
| --- | --- | --- | --- |
| **M01 — Light spoke-wheeled war chariots** | Approximately **2000 BCE**, Sintashta-related steppe contexts. **H/M**. [DOI](https://doi.org/10.1038/s41586-021-04018-9) | Suitable domesticated horses AND wheel/axle craft AND harness AND crew coordination | Chariot goods, teams, vehicle workshops, mobile combat platforms; terrain and maintenance constraints. |
| **M02 — Military riding** | Mounted warfare securely established by the **mid-first millennium BCE** in Eurasia; earlier riding is a separate question. **M/H**. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/origins-of-saddles-and-riding-technology-in-east-asia-discoveries-from-the-mongolian-altai/95BA971FD64B2A7544D4BEF6694A8E14) | Rideable horses AND riding skills AND workable tack | Mounted roles and remount demand. **Neither chariots nor stirrups are required parents.** |
| **M03 — Supportive saddle systems** | Several traditions; rigid wooden saddle evidence from Mongolia dates to approximately the **third–sixth centuries CE**. **H/M** for dated example. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/origins-of-saddles-and-riding-technology-in-east-asia-discoveries-from-the-mongolian-altai/95BA971FD64B2A7544D4BEF6694A8E14) | Tack-making AND woodworking/leatherwork AND accumulated riding experience | Saddle variants affecting rider stability, horse comfort, load distribution, and maintenance. |
| **M04 — Paired stirrups** | **Fourth century CE**, northeastern China/Korea; subsequent diffusion across Eurasia. **M/H**. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/origins-of-saddles-and-riding-technology-in-east-asia-discoveries-from-the-mongolian-altai/95BA971FD64B2A7544D4BEF6694A8E14) | Suitable saddle system AND strong foot-support manufacture | Improved mounting and supported riding actions. No automatic “shock cavalry,” feudalism, or fixed charge multiplier. |

A separate **remount-and-horse-care institution** should maintain horse stocks and trained personnel. That is an organizational capability, not an inevitable consequence of discovering riding.

### 3.3 Fortifications and siege equipment

| Node | Historical appearance or benchmark | Recommended prerequisites | Unlocks and modeled consequences |
| --- | --- | --- | --- |
| **F01 — Ditches, banks, and palisades** | Amnya, western Siberia, approximately **6000 BCE**, among hunter-gatherers. **H/M**. [DOI](https://doi.org/10.15184/aqy.2023.164) | Excavation tools AND timber work where needed AND coordinated labor | Fortified enclosures, defended settlement approaches, gates. **No farming prerequisite.** |
| **F02 — Engineered masonry or rammed-earth defenses** | Multiple early urban traditions; securely mature in Bronze/Iron Age societies. A universal first date is inappropriate. **M/L** for origins. | Masonry OR rammed-earth building AND organized construction | Walls, towers, parapets, gate complexes; use local geometry and materials. |
| **F03 — Protected approaches, ramps, rams, and towers** | Assyrian practice securely illustrated at **Lachish, 701 BCE**; earlier precedents exist. **H** for benchmark. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ojoa.12231) | Heavy carpentry AND earthmoving AND ropes/transport AND engineering supervision | Siege-work sites, covered equipment, approach construction; consume labor and time before an assault opportunity exists. |
| **F04 — Mechanical artillery** | Increasingly prominent in the Mediterranean from the **fourth century BCE**; designs included different energy-storage systems. **M/H**. [PubMed](https://pubmed.ncbi.nlm.nih.gov/14764855/) | Precision carpentry AND suitable tension/torsion components AND specialist design | Bolt- and stone-throwing engine families; accuracy, maintenance, suppression, and structural effects are distinct. |
| **F05 — Traction trebuchets** | Conventionally associated with China in the **later first millennium BCE**, often dated to the fourth–third centuries BCE; identification and terminology complicate exact dating. **M/L**. [JSTOR](https://www.jstor.org/stable/1291833) | Lever/frame engineering AND rope manufacture AND coordinated crews | Human-powered stone-throwers; manpower-intensive operation and transport. |
| **F06 — Counterweight trebuchets** | Secure medieval use by the **twelfth century CE** around the Mediterranean; origin and diffusion disputed. **M**. [JSTOR](https://www.jstor.org/stable/1291833) | Heavy-frame engineering AND lifting/rigging expertise | Counterweight engines and specialized crews. Model bombardment and suppression without assuming cannon-like wall demolition. [JSTOR](https://www.jstor.org/stable/27224963) |
| **F07 — Bastioned artillery fortification** | Developed in **late-fifteenth-century Italy**, with extensive later adaptation. **H/M**. [English Heritage](https://www.english-heritage.org.uk/visit/places/harrys-walls/history/) | Artillery knowledge AND earthwork/masonry engineering AND surveying | Angled bastions, protected gun positions, mutually supporting firing sectors; costs follow perimeter, earthworks, guns, and garrison. |
| **F08 — Dispersed, sheltered field defenses** | Recurrent earthwork traditions; Māori gunfighter pā of the **1840s** provide a particularly useful artillery-era benchmark. **H**. [NZ History](https://nzhistory.govt.nz/media/video/ruapekapeka-roadside-stories) | Excavation AND timber where needed AND defensive knowledge | Sheltered positions, traverses, communication routes, dispersed fighting locations. These can develop independently of European bastion design. |

**Do not require one siege-engine tradition to descend from another.** Torsion engines, traction trebuchets, and counterweight trebuchets can be alternative technical solutions with different labor and material requirements.

### 3.4 Gunpowder transition

| Node | Historical appearance or benchmark | Recommended prerequisites | Unlocks and modeled consequences |
| --- | --- | --- | --- |
| **G01 — Gunpowder technology** | Chinese development from approximately the **ninth century CE**; military formulations recorded in **1044**. **M/H**. [Gunpowder Mills](https://www.royalgunpowdermills.com/chronology-of-gunpowder) | Relevant chemical-processing knowledge AND reliable ingredient supply AND specialist handling | Abstract propellant and early gunpowder-device production chains, storage, transport, and quality variation. |
| **G02 — Gunpowder projectile weapons** | China, particularly the **thirteenth century** for securely recognizable guns; earlier “true gun” claims remain debated. **M**. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-the-royal-asiatic-society/article/abs/mongol-empire-the-first-gunpowder-empire/9128388937125B6FBA9E5F708E27AC8F) | G01 AND suitable metalworking AND projectile manufacture | Early guns and ammunition; low reliability and uneven quality can persist after invention. |
| **G03 — Large artillery and artillery transport** | Expansion during the **fourteenth–fifteenth centuries** across Eurasia; separate invention from local adoption. **M**. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0258161) | G02 AND large-scale metalworking AND heavy transport AND specialist crews | Bombards/cannon, carriages, artillery workshops, siege trains. Increased structural-damage capability accompanied by major logistical cost. |
| **G04 — Matchlock firing mechanisms** | European development in the **fifteenth century**; precursor evidence in 1411 and later practical forms. **M/H**. [American Society of Arms Collectors](https://americansocietyofarmscollectors.org/wp-content/uploads/2022/06/The-Korean-snap-matchlock-a-global-microhistory-v124-Kang.pdf) | Portable guns AND mechanism-making expertise | Matchlock variants and improved two-handed aiming; create regional alternatives rather than one universally optimal lock. |
| **G05 — Flintlock systems** | Mature “true flintlock” associated with France around **1610–1615**. **M**. [National Park Service](https://www.nps.gov/jame/learn/historyculture/history-of-armour-and-weapons-relevant-to-jamestown.htm) | Portable guns AND fine lock/spring manufacture AND ignition-material supply | Flintlock weapons; different readiness, maintenance, and ignition characteristics. |
| **G06 — Combined pike-and-firearm organization** | A major European military configuration in the **sixteenth–seventeenth centuries**, not the world’s first combined-arms warfare. **M**. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/arms-and-armor-in-renaissance-europe) | Available polearms and firearms AND collective training AND command practices | Mixed formations and role coordination. This is a **doctrine node**, not a firearm manufacturing prerequisite. |
| **G07 — Bayonet-equipped firearms** | Bayonets increasingly used during the **seventeenth century**; a surviving English plug example dates to approximately **1690**. **H/M**. [Cleveland Museum of Art](https://www.clevelandart.org/art/1916.1659) | Firearm manufacture AND blade/fitting manufacture | Combined ranged/close-combat equipment. Preserve the operational distinction between plug and later non-obstructing mountings. |

The most important transition is not “bows disappear when gunpowder is researched.” It is a changing economic and tactical balance among equipment, training, ammunition manufacture, protection, and organization. Substantial European armor persisted for more than three centuries after firearms appeared. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/arms-and-armor-common-misconceptions-and-frequently-asked-questions)

### 3.5 Industrial and modern extension nodes

These belong after TCE’s initial agrarian/pre-industrial scope, but their dependency structure should be anticipated.

| Node | Historical benchmark | Recommended prerequisites | Unlocks and modeled consequences |
| --- | --- | --- | --- |
| **I01 — Percussion ignition** | Early nineteenth-century development; US **Model 1842** percussion musket is a secure service benchmark. [National Park Service](https://www.nps.gov/fosm/learn/historyculture/1842-springfield-musket.htm) | Precision mechanisms AND specialist ignition-component manufacture | Percussion arms and conversion recipes; reliability becomes component- and supply-dependent. |
| **I02 — Practical military breechloading and cartridges** | Broad nineteenth-century development; British **Snider adoption, 1866**, illustrates conversion of existing arms. [Royal Museums Greenwich](https://www.rmg.co.uk/collections/objects/rmgc-object-2545) | Accurate metalworking AND ammunition manufacture AND compatible action design | Breechloading weapons, ammunition standards, conversion workshops; old inventories need not be discarded instantly. |
| **I03 — Smokeless magazine-rifle systems** | French **Lebel Model 1886** is a major military benchmark. [Smithsonian Institution](https://www.si.edu/object/lebel-model-188693-rifle%3Anmah_414502) | Advanced propellant industry AND precision arms manufacture AND standardized ammunition | Magazine rifles, reduced smoke signature, changed firing and ammunition-supply profiles. |
| **I04 — Automatic machine guns** | Maxim development in Britain, **1884**. [Science Museum Group Collection](https://collection.sciencemuseumgroup.org.uk/people/cp16875/hiram-stevens-maxim) | Precision mechanism manufacture AND reliable ammunition AND maintenance capability | Crew-served automatic fire; ammunition, cooling, stoppages, and crew losses constrain output. |
| **I05 — Modern artillery systems** | Mature combinations of explosive ammunition, improved guns, observation, and fire coordination by **1914–1918**; components have separate earlier histories. [National Army Museum](https://www.nam.ac.uk/explore/weapons-western-front) | Heavy precision industry AND ammunition industry AND observation/communications | Indirect-fire services, ammunition columns, repair and replacement; fragmentation, suppression, and structural damage remain distinct. |
| **I06 — Tracked armored fighting vehicles** | British combat debut, **1916**. [National Army Museum](https://www.nam.ac.uk/explore/weapons-western-front) | Engines AND transmissions/tracks AND armor manufacture AND repair/fuel system | Tanks, vehicle crews, recovery workshops; mobility is conditional on reliability, terrain, and supply. |
| **I07 — Mobile military wireless** | Experimental and operational development during the **First World War**, followed by further institutional maturation. [British Journal for Military History](https://bjmh.gold.ac.uk/index.php/bjmh/article/view/1472) | Radio technology AND power supply AND trained operators/procedures | Mobile command links; latency, failures, compatibility, and interception risks. |
| **I08 — Radar detection networks** | British demonstrations in **1935** and an operational network before the Second World War; not an exclusive global invention claim. [Bawds Eye Radar](https://www.bawdseyradar.org.uk/the-daventry-experiment/) | Radio engineering AND power AND signal interpretation AND communications | Detection stations and warning networks, not automatic engagement success. |
| **I09 — Guided weapons** | German guided bombs used operationally in **1943** provide a secure early combat benchmark. [Smithsonian Institution](https://www.si.edu/object/launch-lug-bomb-guided-fritz-x-x-1%3Anasm_A19840794001) | Guidance/control technology AND suitable delivery platform AND target observation | Guided-weapon capability; detection and platform survival remain necessary complements. |
| **I10 — Modern fiber/composite personal protection** | Mid-to-late twentieth century; major US soft-body-armor development and trials in the **1970s**. [Office of Justice Programs](https://www.ojp.gov/library/publications/technology-70s-style-nij-forefront-body-armor-research-and-development) | Advanced materials AND manufacturing control AND threat-specific testing | Modern protective equipment with threat classes, coverage, conditioning, and replacement requirements. |

These ten extension nodes are deliberately broad. A modern expansion would split artillery ammunition, aircraft, armored vehicles, sensors, communications, and guided systems into their own graphs rather than compressing them into “modern weapons.”

---

## 4. Variation across eras and world regions

### 4.1 Temporal variation

| Context | Historical pattern | Consequence for TCE |
| --- | --- | --- |
| **Foraging societies** | Projectile technology can greatly predate farming; fortified hunter-gatherer settlements are documented at Amnya. [PubMed](https://pubmed.ncbi.nlm.nih.gov/32582854/) | Do not equate foragers with technologically simple or necessarily unfortified warfare. |
| **Early farming and urban societies** | Weapon production, stored wealth, administration, and organized construction appear in different combinations; Arslantepe is one early example. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1622) | Surplus enables some military specialization, but does not automatically create a standing army or state monopoly. |
| **Pre-industrial societies** | Military systems differ in equipment ownership, pay, mounted resources, craft production, and institutional organization. English pay records and West African horse history illustrate distinct dimensions of cost. [Agincourt 600](https://www.agincourt600.com/2015/03/24/27-april-3-may-1415-further-military-preparations-and-indentures/) | Retainers, militias, hired soldiers, and state-supported forces should be alternative institutional arrangements. |
| **Industrial societies** | Rapid-firing weapons and mechanization expand dependence on ammunition, transport, repair, and communication. [National Army Museum](https://www.nam.ac.uk/explore/weapons-western-front) | Production and replenishment can matter more than the number of weapons initially stockpiled. |
| **Modern societies** | Detection, guidance, protection, and coordination become increasingly differentiated technical systems. [Bawds Eye Radar](https://www.bawdseyradar.org.uk/the-daventry-experiment/) | Model a chain of locating, communicating, reaching, and affecting a target—not a single attack statistic. |

### 4.2 Regional variation

**East Asia:** The Qin crossbow assemblage makes workshop organization and inspection important alongside weapon design. Later gun diffusion was not a one-way passage from a permanently superior West: Korean snap-matchlocks remained useful after related European military preferences had changed. [DOI](https://doi.org/10.1017/s0003598x00050262)

**The Eurasian steppe:** Horse availability, practiced riding, compact bows, and remounts form a complementary system. Model these as products of ecology, livelihoods, and institutions—not inherited cultural combat bonuses. [Royal Armouries](https://royalarmouries.org/objects-and-stories/stories/the-mongols/arms-and-armour-of-the-mongol-empire)

**South, West, and Southeast Asian connections:** Firearm technologies moved through Indian Ocean and overland networks, including Portuguese, South Asian, Chinese, Japanese, and Korean contexts. Access to merchants, migrant craftspeople, and suitable workshops should create multiple adoption routes. [American Society of Arms Collectors](https://americansocietyofarmscollectors.org/wp-content/uploads/2022/06/The-Korean-snap-matchlock-a-global-microhistory-v124-Kang.pdf)

**Europe:** European armor and fortress sequences are useful calibration cases, not a universal chronology. Mail continued to fill gaps in plate equipment; new protection did not always replace old protection wholesale. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/35368)

**Sub-Saharan Africa:** Metallurgical histories cannot be reduced to a mandatory local copper–bronze–iron progression. West African cavalry also requires attention to horse acquisition, upkeep, and environmental constraints rather than an assumption that mounted warfare was absent. [DOI](https://doi.org/10.1007/s10437-023-09545-6)

**The Americas:** Hassig’s study of Aztec warfare emphasizes logistical and political constraints, including agricultural labor demands. TCE should permit substantial military organization and imperial expansion without making Eurasian horse-and-steel packages prerequisites. [University of Oklahoma Press](https://www.oupress.com/9780806127736/aztec-warfare/)

**Oceania:** Māori adaptation to artillery through gunfighter pā is an especially valuable counterexample to a single global fortification tree. Imported weapons and locally developed defensive solutions can coexist. [NZ History](https://nzhistory.govt.nz/media/video/ruapekapeka-roadside-stories)

---

## 5. Stylized facts and validation targets

A correct simulation should reproduce these **relationships**, not replay the historical events on fixed dates.

| Test | Historical anchor or quantitative constraint | Expected emergent result |
| --- | --- | --- |
| **Fortification without agriculture** | Amnya, approximately **6000 BCE**. [DOI](https://doi.org/10.15184/aqy.2023.164) | A sufficiently settled, organized foraging community can invest in defenses. |
| **Cavalry without stirrups** | Mounted warfare precedes paired stirrups by many centuries. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/origins-of-saddles-and-riding-technology-in-east-asia-discoveries-from-the-mongolian-altai/95BA971FD64B2A7544D4BEF6694A8E14) | Removing stirrup knowledge must not remove riding, mounted missiles, or all mounted close combat. |
| **Old and new protection coexist** | Mail used alongside plate; firearms and substantial armor overlap for **over 300 years** in Europe. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/35368) | Equipment mixes persist according to role, wealth, threat, and existing inventory. |
| **Mounted forces have a large forage footprint** | The stated scenario produces **16 tonnes dry matter/day** for 2,000 assumed 400-kg horses. | Cavalry expansion eventually encounters ecological and transport constraints. |
| **Higher firing capacity is not proportional victory** | Approximately **3 versus 15 rounds/minute** in the cited musket/rifle examples. [Age of Revolution](https://ageofrevolution.waterlooassociation.org.uk/200-object/musket-brown-bess/) | Firepower gains depend on exposure, supplies, skill, and enemy behavior. |
| **Mechanical bombardment is not automatically breaching** | Trebuchet research questions cannon-like masonry destruction. [JSTOR](https://www.jstor.org/stable/27224963) | Engines can be worthwhile through suppression and pressure even when breaches are rare. |
| **Defensive adaptation changes what artillery achieves** | Pulaski’s masonry breach contrasts with sheltered earthwork responses such as Māori pā. [National Park Service](https://home.nps.gov/places/the-breeched-corner.htm) | “Artillery unlocked” must not make every fortification equally obsolete. |
| **Technical choices can diverge or reverse** | Regional persistence of Korean snap-matchlocks. [American Society of Arms Collectors](https://americansocietyofarmscollectors.org/wp-content/uploads/2022/06/The-Korean-snap-matchlock-a-global-microhistory-v124-Kang.pdf) | A locally suitable older design can outcompete a newer alternative. |
| **Advanced systems require complementary institutions** | Early tank wireless required procedural development, not merely equipment. [British Journal for Military History](https://bjmh.gold.ac.uk/index.php/bjmh/article/view/1472) | Missing operators or procedures can neutralize nominal technological superiority. |

Additional **design validation requirements** should include partial retreat, surrender, capture, repair, and equipment reuse. Battles should not routinely consume every participant, and a victorious army should still be capable of suffering a logistical defeat afterward.

---

## 6. Recommended implementation for individual agents and institutions

### 6.1 Preserve people; aggregate battlefield computation

For 10,000–50,000 people, use three linked resolutions.

| Resolution | Persistent state | Suggested responsibility |
| --- | --- | --- |
| **Individual agent** | Health, conditioning, skills, morale disposition, occupation, equipment, ownership, relationships | Training, mobilization, injury, recovery, desertion, death, demobilization |
| **Military group** | Membership, cohesion, local orders, formation, supplies, perceived threats | Movement and combat decisions |
| **Operational formation/institution** | Command relationships, procurement, replacements, transport, arsenals, stores | Campaign planning and sustained capacity |

As a **design starting point**, resolve many battlefield interactions in groups of approximately **20–100 people**, while preserving the identities of their members. Group size should vary with formation and simulation distance; it is not a claim about universal historical unit organization.

Sample individual outcomes from group-level events. This preserves consequences for households and occupations without requiring every soldier to perform all-pairs threat evaluation.

### 6.2 Represent capabilities as compatible bundles

A cavalry archer requires more than an inventory count:

```
eligible person
+ sufficient riding skill
+ sufficient bow skill and conditioning
+ compatible, serviceable bow and arrows
+ fit, usable horse
+ usable tack
+ food and forage access
+ a group and command arrangement
```

A shortage in one component should constrain that role, but the person may remain useful in another role. A dismounted rider can become a foot soldier; an artillery crew without ammunition can help move equipment or build defenses.

### 6.3 Use combat events with distinct causal stages

For ranged attacks, a useful abstract structure is:

\[
E[\text{incapacitations}]
=
N\_{\text{shots}}
\cdot P(\text{hit}\mid\text{range, visibility, exposure, skill, suppression})
\cdot P(\text{incapacitation}\mid\text{hit, threat, protection}).
\]

Calculate shot count from an **effective cadence**, capped by available ammunition and operational constraints—not the nominal mechanical rate.

For close combat, use a different interaction model incorporating contact frontage, reach, formation state, fatigue, and local numerical support. Do not pretend that a spear thrust and an artillery burst differ only in their “damage number.”

Keep suppression and morale effects distinct from physical injury. A unit may stop advancing or firing before it suffers many incapacitations.

### 6.4 Give commanders imperfect information

Each group should act on a **perceived situation** rather than direct access to all world state. Communication methods affect the age, reliability, and reach of orders and reports.

This also prevents a common simulation artifact: perfect coordination among thousands of individually autonomous agents. Better technology should improve some information channels, not remove uncertainty.

### 6.5 Make fortifications spatial, but not full structural simulations

Represent a fort as connected segments with:

```
material
height and obstacle geometry
local structural condition
defender positions
protection and shelter
access/approach difficulty
gate state
repair activity
```

Maintain settlement-level food, water, ammunition, and disease conditions separately.

A siege engine targets a segment or defended area. Damage and suppression can open opportunities; defenders can repair, reposition, sortie, or abandon a location. UE5 should display these authoritative Rust outcomes rather than determine them through incidental visual collision behavior.

### 6.6 Institutions should reproduce military capability

Create persistent entities for arsenals, training grounds, retinues, militia organizations, horse establishments, engineering corps, and later industrial procurement systems.

Their key resource is not only equipment. It is also **experienced people who can teach, inspect, organize, and repair**.

A useful skill process is:

\[
\frac{ds}{dt}
=
a\,P(t)(1-s)
-
b\,I(t)s,
\]

where \(s\) is proficiency, \(P(t)\) useful practice, and \(I(t)\) prolonged inactivity. Instructor quality, equipment access, conditioning, and prior experience modify learning.

Do not impose rapid universal skill decay. Some knowledge persists while physical conditioning, readiness, and group coordination deteriorate at different rates.

### 6.7 Suggested node schema

```
TechnologyNode
  id
  knowledge_domains
  requires_all
  requires_any
  enabling_conditions
  learned_by_roles
  discovery_opportunities
  unlock_goods
  unlock_recipes
  unlock_buildings
  unlock_services
  unlock_doctrines
  compatible_imports
  maintenance_dependencies
  evidence:
    first_attestation_range
    region
    interpretation_confidence
    adoption_notes
    source_ids
```

Crucially, `first_attestation_range` belongs to **historical provenance**, not the simulation’s eligibility logic.

### 6.8 What to borrow from existing models and games

**Dwarf Fortress:** its official feature description provides a useful precedent for persistent individuals, differentiated skills, material properties, and bodily consequences. Borrow persistence and material differentiation; avoid reproducing its full injury-detail burden for every distant TCE battle. [Bay 12 Games](https://www.bay12games.com/dwarves/features.html)

**ISAAC/EINSTein and related agent-based combat research:** these explore combat behavior emerging from local interactions rather than only aggregate attrition equations. Borrow local decision rules and emergent group behavior, but do not treat a visually plausible simulated battle as historical validation. [IAORIFORS](https://iaorifors.com/paper/37141)

**Seshat-based military-technology research:** borrow the distinction between the existence of a technology and significant uptake. Its broad comparative data are useful for checking plausible adoption patterns, not for assigning weapon damage. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0258161)

**Recommended simplification:** simulate enough detail to explain *why* a force cannot advance, fire, hold, or remain supplied. Do not simulate internal weapon mechanics, every projectile trajectory, or every structural fracture unless those details change operational decisions.

---

## 7. Sources, datasets, and evidence limits

### 7.1 Highest-value sources for continued calibration

| Source | Best use | Principal limitation |
| --- | --- | --- |
| **Turchin et al. (2021), “Rise of the war machines,” PLOS ONE**, DOI 10.1371/journal.pone.0258161 | Comparative military-technology chronology across **35 geographic areas** and ten world regions; associated data repository | Coarse coding and significant-uptake judgments; observational relationships do not establish universal causal prerequisites. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0258161) |
| **Librado et al. (2021), Nature**, DOI 10.1038/s41586-021-04018-9 | Domestic-horse population history and connections to the chariot horizon | Horse ancestry and spread do not directly reveal cavalry doctrine or training. [DOI](https://doi.org/10.1038/s41586-021-04018-9) |
| **Bayarsaikhan et al., Antiquity**, DOI 10.15184/aqy.2023.172 | Saddle and stirrup chronology | Surviving dated artifacts provide uneven regional coverage. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/origins-of-saddles-and-riding-technology-in-east-asia-discoveries-from-the-mongolian-altai/95BA971FD64B2A7544D4BEF6694A8E14) |
| **Piezonka et al. (2023), Antiquity**, DOI 10.15184/aqy.2023.164 | Fortification among hunter-gatherers | One regional archaeological record is not a universal explanation of fortification. [DOI](https://doi.org/10.15184/aqy.2023.164) |
| **Askew, Formenti and Minetti**, DOI 10.1098/rspb.2011.0816 | Measured locomotion costs of armor | Experimental equipment and participants limit extrapolation. [UNIFIND](https://expertise.unimi.it/resource/item/182803) |
| **Flouris et al. (2024), PLOS ONE**, DOI 10.1371/journal.pone.0301494 | Integrated armor/physiology reconstruction | Simulated ancient combat conditions are interpretive assumptions. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0301494) |
| **Li et al., Antiquity**, DOI 10.1017/S0003598X00050262 | Qin crossbow manufacturing organization | Funerary assemblage and workshop evidence do not directly measure field performance. [DOI](https://doi.org/10.1017/s0003598x00050262) |
| **Garfinkel et al. (2021)**, DOI 10.1111/ojoa.12231; **Fulton (2023), “Trebuchets Were Not Siege Guns, So Why Use Them?”** | Siege engineering and limits of mechanical bombardment | Reconstructions and surviving accounts require case-specific interpretation. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ojoa.12231) |
| **Mary Rose collection; Met arms-and-armor catalogs; Medieval Soldier database** | Equipment distributions, dimensions, materials, names, roles, and contracts | Survival, collecting, elite-equipment, and archival-selection biases. [Mary Rose](https://maryrose.org/discover/collections/the-weaponry-of-the-mary-rose/longbows-and-arrows/) |
| **COW Arms Technology Data v1.1** | Acquisition of **31 arms technologies**, **1816–2023** | Acquisition is not the same as quantity, readiness, or combat effectiveness. [correlatesofwar.org](https://correlatesofwar.org/data-sets/arms-technology-data-v1-0/) |
| **COW National Material Capabilities v7.0** | Annual resource and military indicators, **1816–2022** | Use component variables; the composite index is not a battlefield-strength statistic. [Correlates of War](https://correlatesofwar.org/data-sets/national-material-capabilities/) |

### 7.2 Claims that should remain flagged

**Stirrups caused feudalism or made mounted combat possible.** The chronology alone rules out the second claim; wider institutional causation cannot be encoded as a simple technical prerequisite. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/origins-of-saddles-and-riding-technology-in-east-asia-discoveries-from-the-mongolian-altai/95BA971FD64B2A7544D4BEF6694A8E14)

**Iron automatically outclassed bronze.** Separate material quality, processing, design, availability, and cost. Comparative military-technology work does not justify a universal iron-weapon damage multiplier. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0258161)

**Firearms required almost no training.** Distinguish learning a handling routine from maintaining discipline, coordinated action, and performance under combat conditions. The report’s training hours are deliberately identified as unvalidated design priors.

**A trebuchet was a medieval siege gun.** Recent specialist work explicitly challenges that equivalence. [JSTOR](https://www.jstor.org/stable/27224963)

**One successful replica test establishes historical armor superiority.** Experiments can constrain feasibility and costs, but results depend on reproduction choices and protocol. They cannot directly recover ancient casualty probabilities. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0301494)

**A weapon’s first surviving example identifies its invention.** Archaeological preservation and classification make that inference unsafe, especially for organic equipment. Keep uncertainty and “secure by” dates in the content database. [PubMed](https://pubmed.ncbi.nlm.nih.gov/32582854/)

### Final recommendation

Build TCE’s initial military model around **equipment compatibility, individual competence, group cohesion, fortification geometry, and sustained supply**. Add detailed weapon distinctions only when they change one of those systems.

The resulting history should not be “the society with the newest weapon wins.” It should be: **a society can exploit a new capability only when it can acquire, reproduce, organize, maintain, and afford the people and infrastructure that make it effective.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92863-6ac8-83ea-87f9-ef06d0b48f68)
