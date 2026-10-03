# Bridge engineering through history: a simulation-ready model for TCE

## Executive recommendation

**Represent a bridge as a maintained transport service built from spans, supports, approaches, and institutions—not as a technology with one fixed tonnage limit.**

For TCE, four properties should remain separate:

| Property | What it determines |
| --- | --- |
| **Structural capacity** | Whether the deck, main members, connections, and foundations can withstand the actual loads. |
| **Usability** | Whether a person, animal, or vehicle can negotiate the width, surface, gradient, steps, and movement. |
| **Throughput** | How many travelers can cross without queues or unsafe concentrations of load. |
| **Availability** | Whether flooding, repair, ice, damage, or institutional failure closes the crossing. |

This separation matters historically. The Inka built substantial suspension bridges from plant fibers, but sustained them through frequent organized replacement. Conversely, masonry could survive for centuries while still requiring extensive repairs and dependable revenue. Neither “organic” nor “stone” translates directly into weak, strong, temporary, or permanent. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/question/how-did-inka-keep-their-suspension-bridges-safe.html)

**The strongest evidence concerns geometry, construction methods, and particular projects. Historical working-load ratings and comparable labor costs are much thinner.** Accordingly, the report distinguishes:

* **Observed:** documented dimensions, accounts, or practices.
* **Engineering analogue:** modern information useful for mechanics, not automatically transferable to ancient workmanship.
* **Proposed prior:** an explicit starting value for TCE, not an estimated historical average.

The numerical defaults below are for simulation development, not real-world bridge design.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Choose a crossing system, not just a bridge type

At a potential crossing, generate competing proposals: a ford, stepping stones, ferry, seasonal bridge, permanent bridge, or detour.

For each proposal, estimate:

\[
\text{lifetime generalized cost}
=
\text{construction}
+\text{maintenance}
+\text{travel and waiting}
+\text{expected disruption and losses}.
\]

This is a **recommended decision model**, not a claim that historical builders performed formal cost-benefit analysis. Different agents should perceive different components: merchants value reliable freight access, households value fields and markets, rulers value military movement and control, and religious institutions may value pilgrimage or charitable provision.

Site selection should consider channel width **at flood stage**, bank stability, foundation material, current direction, debris, approaches, navigation, and available building materials. A narrow channel may reduce superstructure cost while creating difficult foundations or high water velocities. Modern scour guidance explicitly treats bridge hydraulics, bed behavior, foundations, and river instability as interconnected problems. [Ponce](https://ponce.sdsu.edu/hec-18-scour.pdf)

**Implementable rule:** allow a more expensive bridge at a better site to outperform a cheaper bridge on the shortest route.

### 1.2 Separate clear span from total crossing length

A long crossing does not imply advanced long-span engineering. Repeating short spans over many supports is a different achievement from crossing a deep gorge without intermediate supports.

Herodotus describes Xerxes’ two Hellespont bridges as supported by **360 and 314 ships**, with anchors, cables, decking, and openings for navigation. Treat those numbers as an ancient literary account, not a surveyed engineering inventory; the important structural point is the multiplication of floating supports. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Herodotus/7A%2A.html)

**Implementable rule:** store an ordered sequence of spans and supports. Extending a trestle adds short bays, foundations, labor, and hydraulic obstruction—not a larger unsupported-span capability.

### 1.3 Capacity follows geometry and load paths

For a simply supported beam under uniform load \(w\) and a central point load \(P\):

\[
M\_{\max}=\frac{wL^2}{8}+\frac{PL}{4}.
\]

Here \(L\) is span, \(w\) includes relevant distributed dead and live loads, and \(P\) is a concentrated load. Bending stress is \(M/Z\), where \(Z\) depends on member shape.

The consequences are useful without a full structural solver:

* At constant distributed loading, doubling span multiplies bending demand by **four**.
* Doubling rectangular beam depth multiplies bending section modulus by **four**, with width unchanged.
* Connections, deck planks, local crushing, buckling, or foundations can fail before the main beam reaches its bending limit.

Timber trusses redirect much of the loading into axial forces, but introduce numerous connections and bracing requirements. The Forest Service manual specifically notes the fabrication and maintenance burden associated with their many members and joints. [Minnesota Department of Transportation](https://www.dot.state.mn.us/bridge/pdf/insp/USFS-TimberBridgeManual/em7700_8_chapter02.pdf)

**Implementable rule:** compute separate margins for deck, main structure, connections, lateral stability, and supports. The weakest relevant margin governs; do not average them into one forgiving health score.

For masonry arches, use a geometry-sensitive stability model or precomputed capacity surface rather than “stone strength × area.” Simplified equilibrium models are actively used to investigate historic arch bridges, including structures carrying railway traffic. [Scipedia](https://www.scipedia.com/public/Olivieri_et_al_2021a)

### 1.4 Suspension bridges exchange intermediate supports for cable and anchorage demands

For an idealized parabolic cable carrying a uniformly distributed vertical load:

\[
H \approx \frac{wL^2}{8f},
\]

where \(H\) is horizontal tension and \(f\) is sag. The approximate tension at the anchorage is:

\[
T \approx \sqrt{H^2+(wL/2)^2}.
\]

These are analytical approximations for generating simulation behavior, not complete design checks.

Increasing sag reduces horizontal tension but produces steeper approaches and a less convenient walking surface. A structurally adequate bridge may still be unsuitable for carts or difficult for particular animals.

The Inka example demonstrates that substantial suspension spans do **not** require industrial metal cables: Smithsonian documentation describes braided grass-and-reed bridges spanning as much as **45 m**. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/question/how-did-inka-road-cross-rivers.html)

**Implementable rule:** natural-fiber suspension requires appropriate cordage, braiding knowledge, anchorage construction, erection skill, and renewal capacity—not ironworking or an era gate.

### 1.5 Pontoon capacity is buoyancy plus deck strength plus mooring strength

For each floating support:

\[
W\_{\text{dead}}+W\_{\text{live}}
<
\rho\_{\text{water}}gV\_{\text{allowable displacement}}.
\]

Preserve a reserve rather than allowing boats to load to their gunwales. Separately check deck bending, uneven load distribution, moorings, wind and current forces, and the connection between floating bridge and fixed approaches.

**Implementable rule:** boats are recoverable capital. A ruler can assemble a crossing rapidly by requisitioning an existing fleet, but this removes boats from fishing, transport, or military service. A “cheap” pontoon bridge may therefore have a large opportunity cost.

### 1.6 Flood failure should emerge through several pathways

Do not implement flooding as a generic annual probability that removes a random bridge. Distinguish:

| Process | Simulation effect |
| --- | --- |
| **Bed degradation and contraction scour** | Lower the riverbed or erode the narrowed opening. |
| **Local scour** | Remove supporting material around piers and abutments. |
| **Debris accumulation** | Increase obstruction and loads; alter the effective opening. |
| **Deck inundation** | Introduce drag, uplift, and impact loads not present during ordinary use. |
| **Approach erosion** | Close the route even when the main span survives. |
| **Channel migration** | Undermine a bank or leave the bridge spanning the wrong channel. |

HEC-RAS separates contraction, pier, and abutment scour; its bridge-scour example also warns that long-term bed changes require separate assessment. [HEC](https://www.hec.usace.army.mil/confluence/rasdocs/rasappguide/latest/bridge-scour-example-11)

A useful optional approximation for noncohesive-bed pier scour is the HEC-18 form:

\[
\frac{y\_s}{y\_1}
=
2K\_1K\_2K\_3
\left(\frac{a}{y\_1}\right)^{0.65}
Fr\_1^{0.43},
\qquad
Fr\_1=\frac{v\_1}{\sqrt{gy\_1}}.
\]

Here \(a\) is pier width, \(y\_1\) approach depth, and the \(K\) factors describe selected geometric and bed conditions. Use this as a bounded approximation within its assumptions, not a universal formula for rock, cohesive soil, ice, and every debris condition. [Ponce](https://ponce.sdsu.edu/onlinescourhec18.php)

**Important implementation detail:** an equilibrium scour estimate is a target depth, not additional erosion to subtract every tick. A simple transient approximation is:

\[
\dot y\_s=(y\_{s,\mathrm{eq}}-y\_s)/\tau\_s,
\]

during erosive conditions. Model deposition separately. Refilled scour holes can conceal earlier undermining; visible bed recovery should not automatically restore damaged foundations. [U.S. Geological Survey Water Resources](https://water.usgs.gov/ogw/bgas/scour/)

### 1.7 Construction and maintenance are staged production processes

Represent construction as tasks with prerequisites:

**site preparation → foundations or anchors → temporary works → main structure → deck and parapets → approaches → opening.**

An arch may need substantial temporary support before it becomes self-supporting. A suspension bridge needs a method of getting its first lines across. A completed span without usable approaches is not an operational crossing. The documented Sri Lankan footbridge case includes precisely these distinctions between manufacture, abutments, erection equipment, installation, and approach completion. [gTKP](https://www.gtkp.com/document/supplement-a/)

Maintenance should replace particular components, not merely replenish abstract health. Japan’s Kintai Bridge provides a particularly useful precedent: Ren and Koshihara reconstruct repeated disassembly, inspection, reuse of sound timber, replacement of decayed pieces, and transmission of craft knowledge through rebuilding. [ResearchGate](https://www.researchgate.net/publication/316920925_A_Study_on_the_Construction_History_of_Kintai_Bridge_in_Japan)

**Implementable rule:** maintenance simultaneously affects structural condition, material demand, employment, and the survival of technical knowledge.

---

## 2. Parameters: spans, loads, labor, time, and upkeep

### 2.1 Initial bridge-type envelopes

The following are **proposed proposal-generation ranges**, not measured historical distributions or hard physical ceilings. Confidence is **low for the numerical envelopes**, but higher for the structural distinctions. Generate outside them only when larger materials, better connections, specialist knowledge, and site conditions justify it.

Lengths are individual clear spans unless explicitly described otherwise.

| Type | Proposed initial envelope | Initial traffic role | Principal constraints |
| --- | --- | --- | --- |
| **Stepping stones** | **0.3–0.7 m clear stepping gaps** | Pedestrians; ability-dependent | Submergence, slippery surfaces, displaced stones; not a beam structure |
| **Single-log crossing** | **2–8 m** | Single-file pedestrians in the simplest template | Trunk dimensions, defects, end support, rolling, usable walking surface |
| **Clapper bridge** | **1–3 m per stone slab** | Pedestrians or pack traffic where deck geometry permits | Available slabs, stone bending, bearing area, low flood clearance |
| **Timber beam bridge** | **3–10 m** | Foot, pack, or cart traffic according to design | Member depth, number of stringers, transverse load sharing, joints |
| **Timber trestle** | **3–8 m per bay**; repeat as needed | Road or foot traffic | Piles/bents, foundation depth, bracing, debris interception |
| **Timber truss** | **10–40 m** | Road or foot traffic | Skilled joints, tension continuity, bracing, erection and inspection |
| **Natural-fiber suspension** | **10–45 m** | Foot traffic initially; animal admission is a separate design choice | Cable quality, sag, anchors, deck movement, frequent renewal |
| **Stone or brick arch** | **3–20 m routinely proposed; 20–40 m specialist proposals** | Foot, pack, or vehicles according to width and capacity | Abutment thrust, foundations, ring geometry, centering, masonry quality |
| **Pontoon/boat bridge** | **2–6 m deck bays between supports**; crossing length resource-limited | Foot, animal, or vehicles according to deck and buoyancy | Fleet, moorings, navigation, water-level variation, storms |

The ranges deliberately overlap. They should produce alternatives rather than a linear upgrade chain.

**Evidence anchors and exceptions:**

| Documented case or engineering source | Quantitative evidence | Confidence and interpretation |
| --- | --- | --- |
| Forest Service timber manual | Log-beam bridges commonly **20–60 ft / 6.1–18.3 m**; sawn-lumber beams commonly **15–25 ft / 4.6–7.6 m** | **High as a modern engineering analogue.** Large logs and engineered assemblies exceed the simplest early templates. [Minnesota Department of Transportation](https://www.dot.state.mn.us/bridge/pdf/insp/USFS-TimberBridgeManual/em7700_8_chapter02.pdf) |
| Same manual, advanced timber trusses | Parallel-chord trusses described up to about **250 ft / 76 m** | **High for documented capability; not a primitive-truss default.** [Minnesota Department of Transportation](https://www.dot.state.mn.us/bridge/pdf/insp/USFS-TimberBridgeManual/em7700_8_chapter02.pdf) |
| Zhaozhou Bridge, China, completed conventionally in **605 CE** | Approximately **37.4 m main span**, **7.3 m rise** | **High for geometry.** A major preindustrial segmental arch; not an ordinary village bridge. [American Society of Civil Engineers](https://www.asce.org/about-civil-engineering/history-and-heritage/historic-landmarks/zhaozhou-bridge) |
| Tarr Steps, England | **17 spans**, approximately **54 m total length** | **High for recorded configuration.** Total length must not be mistaken for slab span. [Exmoor National Park](https://www.exmoor-nationalpark.gov.uk/exmoor-for-everyone/things-to-do/tarr-steps) |
| Inka suspension bridges | Smithsonian describes spans up to **45 m** | **Medium–high for historical capability; no standardized load rating supplied.** [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/question/how-did-inka-road-cross-rivers.html) |
| 1915 Çanakkale Bridge, Türkiye | **2,023 m main span** | **High, operator documentation.** Illustrates the radically different industrial supply chain behind modern long-span bridges. [1915 Çanakkale](https://www.1915canakkale.com/en-us) |

### 2.2 Loads: use explicit load cases rather than historical “tonnage by type”

For TCE, an agent’s mass, carried goods, animal mass, cart mass, cargo, and axle configuration should generate the actual loading.

Useful **proposed test bodies**, not historical averages, are:

| Test case | Suggested model load | What to check |
| --- | --- | --- |
| Person with baggage | **1 kN**, approximately 100 kg mass-equivalent | Local deck and whole-span response |
| Loaded large pack animal | **6 kN**, approximately 610 kg | Hoof loads, surface, width, behavior and span response |
| Loaded cart | **20 kN gross**, approximately 2 t | Axles and wheels separately; draught animals separately |
| Heavy wagon | **60 kN gross**, approximately 6 t | Axle spacing, load distribution, bridge occupancy |
| Pedestrian crowd | Sweep **1–5 kPa** | Distributed load, congestion, deck behavior; not a historical design standard |

For an actual modern reference, the Forest Service’s Catherine Creek replacement drawing specifies a **90 psf / 4.31 kPa pedestrian load**, a **30-inch / 0.762 m diameter** ponderosa-pine stringer, and **35 ft / 10.67 m between bearing centers**. That last measurement is explicitly a bearing-center span, not a verified clear opening. [US Forest Service](https://www.fs.usda.gov/media/267521)

Three rules prevent misleading results:

**First, gross vehicle mass is insufficient.** A short slab can be governed by one wheel or axle even when the complete vehicle is mostly off the bridge.

**Second, separate structural survival from acceptable use.** Excessive movement can deter people or animals before collapse. Modern footbridge engineering explicitly considers pedestrian-induced vibration and comfort. [JRC Publications](https://publications.jrc.ec.europa.eu/repository/handle/JRC53442)

**Third, derive the displayed load restriction from condition and uncertainty.** “One wagon at a time,” “unload before crossing,” and “pedestrians only” are more useful early-world controls than pretending every community knows a precise failure load.

### 2.3 Construction cost: price the bill of quantities

Use:

\[
C\_{\mathrm{build}}
=
\sum\_m Q\_m p\_m
+\sum\_s L\_s w\_s
+C\_{\mathrm{transport}}
+C\_{\mathrm{temporary\ works}}
+C\_{\mathrm{land/access}}.
\]

Here \(Q\_m\) is purchased material quantity, \(L\_s\) site labor by skill, and \(w\_s\) its wage or opportunity cost.

Avoid double-counting quarrying, logging, lime burning, or rope making when those costs are already embedded in purchased materials. For communal labor, substitute foregone production and provisioning costs rather than setting wages to zero.

Medieval bridge accounts involved much more than visible stonework: Harrison identifies large requirements for timber, lime, chalk, metals, and labor, together with substantial maintenance obligations. [OUP Academic](https://academic.oup.com/book/6206/chapter/149832753)

#### Provisional work budgets for TCE

These are **low-confidence authored starting budgets**, not historical reconstructions. They assume materials delivered to the site or workshop, ordinary access, hand-tool fabrication where relevant, and no unusually difficult underwater foundations. One person-day here means **eight productive worker-hours**.

| Example project | Proposed fabrication and site labor | Indicative active duration with stated crew |
| --- | --- | --- |
| Stepping-stone crossing, 10 m waterway | **10–60 person-days** | **2–12 working days**, 5 workers |
| Simple log footbridge, 5 m | **5–30 person-days** | **1–6 working days**, 5 workers |
| Clapper crossing, three 2 m bays | **30–200 person-days** | **3–20 working days**, 10 workers |
| Timber road beam bridge, 8 × 3 m | **80–400 person-days** | **8–40 working days**, 10 workers |
| Timber trestle, 30 × 3 m | **300–1,500 person-days** | **15–75 working days**, 20 workers |
| Timber truss, 25 × 3 m | **600–3,000 person-days** | **30–150 working days**, 20 workers |
| Fiber suspension, 30 × 1 m, existing sound anchors | **100–600 person-days**, including rope fabrication | **5–30 crew-equivalent working days**, 20 workers |
| Masonry arch, 10 × 3 m | **1,500–6,000 person-days** | **50–200 crew-equivalent working days**, 30 workers |
| Pontoon crossing, 50 m, suitable boats already available | **100–600 person-days** | **5–30 working days**, 20 workers |

The duration column is labor divided by crew, **not a guaranteed calendar schedule**. Crew limits, task dependencies, curing, river diversion, harvest labor demands, missing materials, and funding gaps can lengthen it substantially. New suspension anchors and construction of the pontoon fleet are additional projects.

For implementation, replace these lump sums progressively with quantities and task productivity. They are most useful initially as order-of-magnitude guardrails.

#### Documented cost and time anchors

| Project | Recorded quantities, time, or expenditure | What the comparison supports |
| --- | --- | --- |
| **Q’eswachaka renewal, Peru** | Smithsonian describes bridge rebuilding in **three days** | Rapid final assembly is possible within a mature renewal system; it does not price original anchors or all prior material preparation. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/video/bridge-qeswachaka.html) |
| **Medieval London Bridge** | Stone bridge construction **1176–1209**, approximately **33 years** | Major crossings can span generations; do not extrapolate village-bridge schedules to monumental urban projects. [City Bridge Foundation](https://www.citybridgefoundation.org.uk/assets/documents/Bridge-House-Estates-Annual-Report-2017-2018.pdf) |
| **Iron Bridge, England** | Construction began **1777**, completed **1779**, opened **1781**; **30 m span**, **378 tons of iron as reported**; about **£6,000**, versus **£3,200 estimated** | Distinguish construction, opening, and budget. The reported final cost was roughly **1.9 times** the estimate. These are contemporary pounds, not modern purchasing-power equivalents. [English Heritage](https://production.english-heritage.org.uk/visit/places/iron-bridge/history/) |
| **Sri Lankan modular steel footbridge, 2004 report** | Nominal **17 m span**, approximately **24 m² deck**; **US$7,483**, reported **US$312/m²**; itemized work totals **380 work-days** | A rare project-level benchmark. The account assigns **US$4,105** to labor/supervision/overhead and **US$3,378** to the other cost column, including transport—about **55% versus 45%**, not a universal ratio. [gTKP](https://www.gtkp.com/document/supplement-a/) |

### 2.4 Maintenance parameters and failure state

| Component or practice | Evidence or proposed parameter | Confidence |
| --- | --- | --- |
| Inka fiber-bridge replacement | Often **every 1–2 years**; Q’eswachaka’s documented practice is annual | **Medium–high**, historical/institutional account. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/question/how-did-inka-keep-their-suspension-bridges-safe.html) |
| Untreated log bridges | Forest Service manual describes typical temporary lives of **10–20 years**, dependent on conditions | **High as an engineering analogue**, poor as a universal preindustrial lifespan. [Minnesota Department of Transportation](https://www.dot.state.mn.us/bridge/pdf/insp/USFS-TimberBridgeManual/em7700_8_chapter02.pdf) |
| Kintai rebuilding | Study reconstructs almost **16 equivalent complete rebuildings** over approximately **342 years**, averaging roughly **20 years**, with irregular actual schedules | **Medium–high**; a maintained tradition, not survival of original timber. [ResearchGate](https://www.researchgate.net/publication/316920925_A_Study_on_the_Construction_History_of_Kintai_Bridge_in_Japan) |
| Inspection scheduling | Start with **seasonal inspection plus mandatory post-flood inspection** | **Proposed rule**, not a historical universal |
| Suspension sag/span ratio | Explore **0.08–0.15** in initial templates | **Proposed geometry prior**; check cable tension and usability |
| Masonry rise/span ratio | Explore **0.2–0.5** initially | **Proposed geometry prior**; Zhaozhou is near the lower end, but this is not a stability guarantee |
| Financial reserve | Target **1–3 years of expected routine maintenance**, with separate disaster funding | **Proposed institutional prior**, not a measured historical norm |

Do not assign stone an automatic lifespan of “500 years.” Separate persistent masonry from replaceable paving, joints, drainage, parapets, approaches, and foundation protection.

Also avoid a fixed global historical collapse rate. A bridge exposed to frequent destructive floods is not comparable to one on stable rock above ordinary flood levels.

---

## 3. Variation across eras and world regions

### 3.1 Eras change available capabilities—not mandatory bridge sequences

| Context | Appropriate interpretation for TCE |
| --- | --- |
| **Foraging societies** | Make local logs, stones, cordage, rafts, and temporary crossings available through relevant practical skills. Do not invent a universal first bridge date or precise prehistoric load schedule; the evidence is inadequate for that. |
| **Early farming** | Settled demand and coordinated woodworking can support substantial infrastructure without metal tools or states. The Sweet Track’s timbers were felled in **3807/3806 BCE**; it was a raised wetland walkway, not a long unsupported river span. [Historic England](https://historicengland.org.uk/listing/the-list/list-entry/1014831) |
| **Preindustrial societies** | Sophisticated masonry, timber, cordage, boats, and organized labor produce several viable technological paths. Inka fiber suspension and Chinese stone arches demonstrate that advanced crossings need not share a material sequence. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/question/how-did-inka-road-cross-rivers.html) |
| **Industrial societies** | Large-scale metal production, fabrication, transport, and capital mobilization expand the feasible design space. Iron Bridge is useful as a transition case: its castings incorporated techniques adapted from carpentry rather than appearing as fully standardized modern fabrication. [English Heritage](https://production.english-heritage.org.uk/visit/places/iron-bridge/history/) |
| **Modern societies** | Engineered timber and small rural bridges coexist with kilometer-scale suspension bridges. Industrial materials do not make every older method economically obsolete. [Minnesota Department of Transportation](https://www.dot.state.mn.us/bridge/pdf/insp/USFS-TimberBridgeManual/em7700_8_chapter02.pdf) |

### 3.2 Regional cases that should shape the model

**Andes: fiber technology plus recurring collective labor.**  
The significant innovation is a complete system: locally available fibers, braided cables, skilled erection, stone anchorage, and obligations to rebuild. The historical Inka system required the communities served by bridges to maintain them. This is a strong model for infrastructure whose reliability depends on political and communal continuity rather than monetary maintenance contracts. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/question/how-did-inka-keep-their-suspension-bridges-safe.html)

**China: masonry arches and sophisticated wooden arches are separate traditions.**  
Zhaozhou’s open spandrels reduce solid obstruction and accommodate floodwater. In Fujian and Zhejiang, UNESCO documents wooden arch construction through beam weaving and mortise-and-tenon craft knowledge, transmitted through masters and apprentices. Do not force these structures into a generic European triangulated-truss technology. [American Society of Civil Engineers](https://www.asce.org/about-civil-engineering/history-and-heritage/historic-landmarks/zhaozhou-bridge)

**Japan: rebuilding preserves both infrastructure and competence.**  
Kintai’s repairs were affected by disasters, available finance, and changing construction practices. The study reports that the approximately fifty-year interval before a modern rebuilding left only two surviving carpenters who had participated in the preceding one. TCE should therefore allow long periods without major construction to weaken a specialized craft community even when drawings survive. [ResearchGate](https://www.researchgate.net/publication/316920925_A_Study_on_the_Construction_History_of_Kintai_Bridge_in_Japan)

**Northeastern India: living structures require a different condition model.**  
Research on Khasi and Jaintia living-root bridges describes guided *Ficus elastica* roots, root fusion, and adaptive growth that can strengthen a structure over time. This is not well represented by universal monotonic decay. Use a slow establishment phase followed by growth, maintenance, damage, and tree-health dynamics. Precise bridge ages and development times should remain case-specific. [Nature](https://www.nature.com/articles/s41598-019-48652-w)

**East Africa: relative prices can favor masonry in a modern economy.**  
The Kasese District manual, based on work in western Uganda during **2009–2013**, explicitly identifies low labor costs and expensive industrial materials as conditions favoring stone arches. It also reports feasibility in Uganda, Tanzania, and Rwanda. For TCE, technology adoption should respond to local prices and skills rather than an automatic stone-to-steel replacement rule. [Climate Technology Centre & Network](https://www.ctc-n.org/sites/default/files/resources/531db02b-7db4-4306-9680-55c40a000075.pdf)

**Europe: permanent crossings often depended on permanent income.**  
Rochester’s medieval benefactors endowed the bridge institution with property whose income funded repairs. London’s bridge institution accumulated taxes, rents, and bequests. Both are precedents for infrastructure ownership that extends beyond the founding builder or ruler. [The Rochester Bridge Trust](https://rbt.org.uk/estate/)

---

## 4. Tolls and financing

### 4.1 Distinguish who supplies capital, who works, and who maintains

TCE should support several institutional arrangements, which can coexist:

| Arrangement | Capital and labor | Long-term vulnerability |
| --- | --- | --- |
| **Household or village cooperation** | Labor contributions, materials, communal provisioning | Participation declines or disputes over obligations |
| **Ruler or military project** | Taxation, requisition, compulsory labor, treasury | War priorities, succession, revenue disruption |
| **Charitable or religious endowment** | Donations and endowed property | Lost rents, appropriation, weak administration |
| **Merchant or toll association** | Subscriptions or credit, repaid from users | Low traffic, rival crossings, collection costs |
| **Public treasury or concession** | Tax revenue, borrowing, contracted operation | Fiscal stress, deferred maintenance, contested tariffs |

These are proposed institutional templates grounded in the documented diversity: Inka service obligations, Rochester’s property endowments, London’s mixed revenues, and Iron Bridge’s toll collection. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/question/how-did-inka-keep-their-suspension-bridges-safe.html)

### 4.2 Make toll revenue endogenous

Use:

\[
R\_{\mathrm{net}}
=
\sum\_c \tau\_c Q\_c(\tau\_c)
+R\_{\mathrm{endowment}}
+R\_{\mathrm{tax}}
-C\_{\mathrm{collection}}
-C\_{\mathrm{maintenance}}
-C\_{\mathrm{debt}}.
\]

Traffic \(Q\_c\) depends on tolls because travelers can change routes, use a ferry, wait for low water, reduce trips, or abandon trade.

A simple **illustrative calculation**: annual maintenance and administration costing 100 wage-days requires an average toll of **0.01 wage-day per crossing** at 10,000 paying crossings. At 2,000 crossings it requires **0.05 wage-day**—before repaying construction. This does not prescribe a historical tariff; it demonstrates why traffic and funding structure matter.

Allow exemptions, resident contributions, military demands, and differential vehicle charges to emerge from institutions. Exemption should not be universal: Iron Bridge’s posted rules explicitly included soldiers, baggage wagons, mail coaches, and the royal family among toll-paying users. [English Heritage](https://production.english-heritage.org.uk/visit/places/iron-bridge/7-things-you-didnt-know/)

**Critical rule:** opening a bridge should not guarantee a maintenance budget. A society may successfully mobilize a spectacular construction campaign and then fail to sustain ordinary repairs.

---

## 5. Stylized facts a correct simulation should reproduce

### 5.1 Short-span solutions remain useful after long-span techniques appear

The adoption of trusses or suspension should not eliminate beam bridges, ferries, or stepping stones. The test is whether their whole-life cost fits the site and traffic—not whether they are technologically older. The contemporary East African stone-arch program is a particularly clear counterexample to a universal replacement sequence. [Climate Technology Centre & Network](https://www.ctc-n.org/sites/default/files/resources/531db02b-7db4-4306-9680-55c40a000075.pdf)

### 5.2 More supports reduce span demands but create hydraulic liabilities

A cheap many-bay crossing should sometimes be more flood-vulnerable than an expensive bridge with fewer supports. However, pier shape and alignment matter: Kintai hydraulic experiments found differences in water-level rise and scour behavior among shapes and orientations, rather than one simple “pointed pier is safe” rule. [Nature](https://www.nature.com/articles/s40494-021-00576-3)

### 5.3 Floods can move surprisingly massive components

Exmoor National Park reports that Tarr Steps slabs weighing up to **2 tonnes** have been swept as much as **50 m downstream**. Mass alone should not confer immunity to flood damage. [Exmoor National Park](https://www.exmoor-nationalpark.gov.uk/exmoor-for-everyone/things-to-do/tarr-steps)

### 5.4 Centuries of service can mean many generations of replacement

A bridge can remain socially “the same bridge” while its deck, cables, or timber superstructure are renewed repeatedly. Preserve the asset’s name, institution, and route identity separately from component manufacture dates. Kintai and Q’eswachaka provide contrasting examples. [ResearchGate](https://www.researchgate.net/publication/316920925_A_Study_on_the_Construction_History_of_Kintai_Bridge_in_Japan)

### 5.5 Capacity deteriorates nonlinearly with section loss

For an ideal circular beam, bending section modulus is proportional to diameter cubed. Losing **10% of effective diameter** leaves approximately **73% of original bending capacity**, before considering cracks or other defects:

\[
0.9^3=0.729.
\]

This is an analytical model property, not an empirical universal decay rate. It is a useful regression test against an overly linear health model.

### 5.6 Rare hazards become important over long lives

Under the illustrative assumption of independent annual events, a flood with **1% annual exceedance probability** has approximately a **39.5% chance of occurring at least once in fifty years**:

\[
1-(1-0.01)^{50}\approx0.395.
\]

That is flood occurrence, **not bridge failure probability**. Flood magnitude, exposure, and structural vulnerability must remain separate. Shared floods should also create correlated damage across a river basin rather than independent bridge disasters.

### 5.7 Completion and operation are different economic achievements

TCE should produce unfinished crossings, bridges without adequate approaches, temporary ferry substitutions during repair, and institutions that keep old crossings usable despite limited new construction. The documented separation of manufacture, installation, supervision, and approaches in the Sri Lankan case makes a useful construction-state benchmark. [gTKP](https://www.gtkp.com/document/supplement-a/)

---

## 6. Recommended implementation, reference models, and evidence limits

### 6.1 A compact Rust representation

Use five linked records:

```
Crossing
  river_reach, route_connections, approaches, alternative_crossings

Span
  structural_system, geometry, members_or_capacity_template,
  deck, connections, condition, actual_loads

Support
  support_type, foundation_material, embedment_or_anchor_geometry,
  scour_state, settlement, structural_condition

BridgeInstitution
  owner, maintenance_obligations, treasury, endowment,
  toll_schedule, inspection_policy, repair_queue

ConstructionOrRepairProject
  tasks, dependencies, material_stocks, assigned_workers,
  required_skills, temporary_works, funding
```

Store **true condition separately from believed condition**. Inspectors, builders, and travelers should discover defects imperfectly. Repair decisions depend on reports and reputation; physics depends on the actual structure.

Agents should contribute through ordinary TCE systems: quarrying, logging, rope making, carpentry, masonry, hauling, financing, toll collection, inspection, and apprenticeship. Do not create a generic bridge-worker population detached from the economy.

### 6.2 What to simplify

For 10k–50k agents, use analytical formulas and precomputed response tables rather than solving a full structural model every frame.

**Recommended update schedule:** check occupancy and load when agents enter, leave, or materially change position; update gradual deterioration daily or less often; update scour and flood forces during hydrological events; update institutional budgets on their existing economic schedule.

For timber trusses and arches, precompute capacity against geometry, material quality, connection quality, support movement, and a few representative load patterns. Keep a local deck check so a heavy wheel can break flooring without destroying the entire bridge.

Sample workmanship and hidden defects when components are built. Do not repeatedly reroll “construction quality” each day. If residual uncertainty requires a hazard process, use a timestep-consistent form such as \(1-e^{-\lambda\Delta t}\).

Rust should determine closure, damage, casualties, and network effects. Unreal should visualize those outcomes, not independently decide whether a simulated bridge survives.

### 6.3 Existing models and games worth borrowing from

| Reference | Useful contribution | What not to import uncritically |
| --- | --- | --- |
| **HEC-RAS / HEC-18** | Bridge hydraulics and separate scour mechanisms; useful for offline calibration | A full river solver is unnecessary for every bridge tick. Match the equations and software version: documentation identifies differences between implemented methods and later HEC-18 procedures. [HEC](https://www.hec.usace.army.mil/confluence/rasdocs/ras1dtechref/6.0/estimating-scour-at-bridges) |
| **LimitState:RING and Sheffield masonry-bridge research** | Geometry-sensitive arch stability and reduced structural analysis | Professional assessment inputs and assumptions are not automatically available to early-world builders. [Google Sites](https://sites.google.com/sheffield.ac.uk/ccsmithresearchpages/masonry-arch-bridges) |
| **Membrane Equilibrium Analysis** | Another reduced approach tested against a monitored historic railway bridge | Simplification still needs validation against representative cases. [Scipedia](https://www.scipedia.com/public/Olivieri_et_al_2021a) |
| **OpenTTD** | Clear player-facing bridge categories, construction choices, length and speed constraints | Its type/year/length tables are game rules, not historical engineering data. [OpenTTD Wiki](https://wiki.openttd.org/en/Manual/Base%20Set/Bridge%20types) |
| **Poly Bridge** | Legible structural behavior under moving vehicle loads and repeatable physics-based testing | It is a bridge-building puzzle reference, not evidence for historical costs or institutional maintenance. [App Store](https://apps.apple.com/ci/app/poly-bridge-3/id1586851637?l=en-GB&platform=ipad) |

The appropriate synthesis for TCE is **reduced structural mechanics + river hazards + an ordinary agent economy + persistent maintenance institutions**.

### 6.4 Core scholarly sources and datasets

| Source | Best use |
| --- | --- |
| **Ritter, M. A. (1990), *Timber Bridges: Design, Construction, Inspection, and Maintenance*** | Structural vocabulary, engineered span ranges, timber detailing and upkeep. Use modern observations as analogues, not ancient averages. [Forest Products Laboratory](https://www.fpl.fs.usda.gov/documnts/misc/em7700_8_intro.pdf) |
| **Arneson, Zevenbergen, Lagasse and Clopper (2012), *Evaluating Scour at Bridges*, HEC-18, fifth edition** | Foundations, flood exposure, scour mechanisms and countermeasures. [Ponce](https://ponce.sdsu.edu/hec-18-scour.pdf) |
| **Ren and Koshihara (2017), “A Study on the Construction History of Kintai Bridge in Japan,” DOI 10.3130/jaabe.16.255** | Rebuilding, component reuse, finance-related delays, and craft transmission. [ResearchGate](https://www.researchgate.net/publication/316920925_A_Study_on_the_Construction_History_of_Kintai_Bridge_in_Japan) |
| **Sato (2021), “Rationality in pier geometry of Kintaikyo Bridge from viewpoint of river engineering”** | Experimental evidence for hydraulic trade-offs in historic pier geometry. [Nature](https://www.nature.com/articles/s40494-021-00576-3) |
| **Ludwig et al. (2019), “Living bridges using aerial roots of Ficus elastica—an interdisciplinary perspective,” DOI 10.1038/s41598-019-48652-w** | A global alternative to conventional dead-material structures; adaptive growth and root fusion. [Nature](https://www.nature.com/articles/s41598-019-48652-w) |
| **I. T. Transport (2004), *Footbridges*, Supplement A** | Itemized construction costs, work-days, fabrication, erection, and supervision for one documented project. [gTKP](https://www.gtkp.com/document/supplement-a/) |
| **FHWA National Bridge Inventory** | Modern geometry, material, condition and related inventory data. For historical downloads, note that **2025 was the last submission year using the old Coding Guide ASCII format**; schema transitions need explicit handling. [Federal Highway Administration](https://www.fhwa.dot.gov/bridge/nbi/ascii.cfm) |
| **Rochester Bridge Trust estate records** | Long-run maintenance-finance research; digitized rent information covers **1577–1914**, with older account material also identified. [The Rochester Bridge Trust](https://rbt.org.uk/estate/) |

### 6.5 Contested claims and thin evidence

**First appearances are particularly unreliable for simple bridges.** A surviving stone crossing is not necessarily prehistoric, and an archaeological absence of timber or fiber is not proof of technological absence. Tarr Steps is a useful warning: the National Park gives its first documentary mention as Tudor and does not establish the popular prehistoric dating. [Exmoor National Park](https://www.exmoor-nationalpark.gov.uk/exmoor-for-everyone/things-to-do/tarr-steps)

**Historic load ratings are frequently inferred rather than measured.** “Carried carts,” “carried an army,” and “still standing” do not establish a safe working tonnage. Modern retrospective calculations require dimensions, material condition, support behavior, and a stated safety framework.

**Survival is not untreated service life.** Repaired monuments and continuously renewed crossings are selected survivors. Their age should not calibrate the deterioration rate of abandoned bridges.

**Comparable global cost data remain a major gap in the material reviewed.** I did not find a defensible dataset jointly reporting preindustrial geometry, verified working loads, complete labor inputs, and flood exposure across regions. The proposed labor budgets should therefore be treated as calibration candidates, while documented project accounts remain case-specific anchors.

**For TCE, the central historical pattern is not “better technology produces permanent bridges.” It is that geometry, local resources, river behavior, traffic demand, and the capacity to organize repeated work jointly determine whether a crossing exists—and continues to exist.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92915-5b98-83ea-ade9-6c67fa4d6f76)
