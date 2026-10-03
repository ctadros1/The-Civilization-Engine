*The Qingming scroll’s bridge scene depicts movement and commerce sharing the same space: pedestrians cross, vendors occupy the bridge, and boats navigate below. It is evidence of activities and spatial arrangements—not a traffic-count survey.* [AFE East Asia](https://afe.easia.columbia.edu/songdynasty-module/cities-bridge.html)

# Urban mobility and congestion before industrial transport

## A simulation-ready report for The Civilization Engine

**Yes, pre-industrial cities experienced congestion. But the most useful model for TCE is not a modern road network with cars replaced by carts.** Historical street life combined movement with selling, working, unloading, ceremonies, and social interaction. Archaeological research at Pompeii also indicates that wheeled circulation depended on highly specific street geometry and access arrangements. Congestion should therefore arise from **conflicting uses, narrow passages, vehicle clearance, and slow handling**, as well as pedestrian density. [Cambridge University Press](https://www.cambridge.org/core/books/roman-street/life-in-the-street/837A59424D14E5415F9B2663EB6B35F4)

My recommendation is an **activity-based pedestrian and freight simulation on a shared, capacity-constrained network**. Keep individual travelers and shipments, but use inexpensive movement between bottlenecks. Treat markets, loading places, bridges, gates, and boat landings as explicit facilities with space, workers, schedules, and access rules.

A critical qualification: **the evidence supports mechanisms much better than universal historical traffic parameters**. We can reconstruct particular vehicles, regulations, street layouts, and transport organizations. We generally cannot specify a defensible “average medieval commute,” “Roman carts per hour,” or worldwide pre-industrial modal split.

---

## 1. Mechanisms: rules TCE can implement

### 1.1 Generate activities, not standardized commutes

The familiar **Marchetti constant** is a hypothesis about approximately an hour of daily travel expenditure, not a demonstrated requirement that everyone throughout history commuted thirty minutes each way. Marchetti’s original argument should be read alongside Mokhtarian and Chen’s review of more than two dozen studies: travel time was not constant at the individual level, and varied with household characteristics, activities, and residential environments. Even apparent aggregate stability did not establish a universal mechanism. [IIASA](https://pure.iiasa.ac.at/id/eprint/4071/)

**Implementation:** give each person an activity schedule constrained by available time, obligations, fatigue, and access. Distinguish:

| Activity arrangement | Trip-generation rule |
| --- | --- |
| Work within the household compound | No external commute; inputs, sales, water, fuel, and social visits can still generate trips. |
| Separate workshop, field, institution, or employer | Travel to the actual workplace, with occupation-specific timing. |
| Mobile occupation | Portering, selling, herding, or collecting consists partly of movement; do not count the whole working day against a commuting allowance. |
| Household provisioning | Generate trips from inventories and responsibilities; permit purchasing for several people and combining errands. |
| Temporary or seasonal work | Change destinations and departure times with construction projects, harvests, contracts, and hiring opportunities. |

These are proposed agent categories, not estimates of their historical population shares. Ancient retail scholarship supports a varied commercial landscape, rather than a single centralized market through which all purchasing must pass. [Oxford University Press](https://global.oup.com/academic/product/shopping-in-ancient-rome-9780199698219)

For a **chosen simulation walking speed of 1.2 m/s**, uninterrupted network travel reaches:

| Travel time, one way | Network distance |
| --- | --- |
| 5 minutes | 360 m |
| 15 minutes | 1.08 km |
| 30 minutes | 2.16 km |

These are calculations, not historical commuting observations. Gates, hills, bridges, queues, and indirect routes reduce the area reachable within the same time.

**Do not turn the thirty-minute distance into a maximum city radius.** An urban region can contain many local activity areas rather than requiring everyone to cross its entire extent. Angkor is an important counterexample to treating all pre-industrial urbanism as a compact walking disk: archaeological mapping identified a low-density settlement complex extending over more than 1,000 km², not a continuously dense city of that size. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0702525104)

### 1.2 Freight demand comes from goods and inventories

A household needs food; a kiln needs fuel; a construction site needs stone. These requirements should create **shipment jobs**, not decorative traffic.

A suitable rule is:

\[
Q\_{\text{order}}=\max(0,\;I\_{\text{target}}-I\_{\text{available}}-I\_{\text{committed inbound}})
\]

Assign transport only after checking ownership, payment or obligations, available workers, and feasible routes. A shipment has mass, volume, packaging, destination, deadline, and possibly perishability.

Transport choice should compare the **complete chain**:

> Collect → load → travel → wait → unload → transfer or store → complete delivery.

A water route may therefore involve a boat, landing, warehouse, and final porter journey. In early-modern Osaka, rice handling involved organized stevedores and delivery porters; rights to handling work could become inheritable and separable from the people performing it. That is an excellent historical basis for distinguishing **facility access rights, labor supply, and physical throughput**. [BPB](https://bpb-us-e1.wpmucdn.com/blogs.uoregon.edu/dist/8/12656/files/2018/06/Morishita-Stevedores-2012-2d26imx.pdf)

For TCE, the consequence is straightforward: an empty landing without workers must not function like a fully staffed warehouse.

### 1.3 Carts and boats primarily change payload, not necessarily speed

Human porters, pack animals, carts, and small boats should not form a simple ladder of progressively faster travel.

Animal transport engineering describes packing as especially useful on narrow, steep, rocky, or sandy routes. Carts can carry substantially more, but require suitable paths and equipment. Animal condition, harness, terrain, and vehicle design materially affect performance. [FAOHome](https://www.fao.org/4/x5483b/x5483b0w.htm)

Hassig’s reconstruction of transport around Tenochtitlan illustrates the distinction particularly well: a canoe could move approximately a tonne with one paddler at roughly walking speed. Its main advantage was **cargo moved per worker**, not rapid passenger movement. [Arqueología Mexicana](https://arqueologiamexicana.mx/mexico-antiguo/tenochtitlan-en-el-gran-lago)

**Implementation:** choose among modes using:

\[
C\_{\text{freight}}
=C\_{\text{labor}}+C\_{\text{animal}}+C\_{\text{vehicle}}
+C\_{\text{fees}}+C\_{\text{handling}}+C\_{\text{expected loss}}
\]

Calculate each term across the whole journey, including transfers and empty returns. A short porter journey can beat a boat shipment that requires two transfers; a heavy recurrent flow may justify a wharf and warehouse.

Litters belong in a different category: passenger comfort, status, privacy, or accessibility purchased with other people’s labor. They should consume crew-hours, not provide a free speed upgrade.

### 1.4 Streets are both routes and places of activity

Hartnett’s reconstruction of Roman street life emphasizes encounters among construction work, animals, processions, vehicles, and other activities. The relevant historical problem was broader than excessive numbers of identical travelers. [Cambridge University Press](https://www.cambridge.org/core/books/roman-street/life-in-the-street/837A59424D14E5415F9B2663EB6B35F4)

Represent a street’s **usable movement space**, not just its mapped width:

\[
W\_{\text{usable}}(x,t)
=
W\_{\text{physical}}(x)
-
W\_{\text{occupied}}(x,t)
\]

Occupied space can include stalls, stored materials, unloading vehicles, drainage channels, waste, tethered animals, and work areas. Use actual cross-sectional occupancy where possible: subtracting one average width cannot describe a staggered sequence of obstacles.

Two distinct congestion mechanisms follow:

**Pedestrian crowding:** increasing density reduces walking speed and passing opportunities.

**Clearance failure:** a cart cannot pass another cart, turn into a lane, or squeeze past a stationary load—even when pedestrian density is low.

A street may therefore remain permeable to walkers while being closed to freight vehicles. This is much more useful than assigning the entire street one congestion multiplier.

### 1.5 Handling facilities create queues

For a loading or unloading operation, a useful initial service model is:

\[
t\_{\text{service}}
=
t\_{\text{setup}}
+
\frac{Q}{n\,r}
\]

Here, \(Q\) is cargo mass, \(n\) is the number of effective workers, and \(r\) is handling productivity per worker. Reduce effective crew size when the doorway, gangplank, or storage aisle prevents simultaneous work.

This produces several desired effects without scripted congestion:

* More boats do not increase throughput when the landing is labor-constrained.
* A larger cart reduces loaded trips but occupies the unloading position longer.
* Storage beside the landing reduces handling distance.
* A queue can obstruct a junction and delay unrelated travelers.

**Finite queue space matters.** When a landing, gate approach, or alley is full, waiting must extend upstream rather than accumulate inside an invisible point.

### 1.6 Regulation changes the timing and distribution of traffic

Rome’s often-repeated “daytime cart ban” needs precision. The *Tabula Heracleensis* restricts wagons in Rome and continuously built-up suburbs from sunrise until the tenth daylight hour. It includes exceptions for specified public construction and demolition work, religious and ceremonial uses, and certain vehicles leaving after entering at night. Its dating, attribution, and precise vehicle scope should not be simplified into an empire-wide ban on all wheeled movement. The text also assigns street-maintenance responsibilities to frontage owners under official supervision. [Roman Law Library](https://droitromain.univ-grenoble-alpes.fr/Anglica/heracleensis_johnson.html)

**TCE rule representation:**

`jurisdiction + vehicle class + trip purpose + time window + exemptions + enforcement`

A restriction should make agents reschedule, transfer loads, seek permission, use another route, violate the rule, or abandon an uneconomic trip. It should not make required deliveries disappear.

Poehler’s Pompeii study reconstructs circulation from street geometry and wear evidence. It is useful for testing directional routing and restricted access, but it is a reconstruction—not a surviving traffic-count dataset or proof that every Roman town used the same system. [OUP Academic](https://academic.oup.com/book/7089)

---

## 2. Parameters: evidence, analogues, and initial simulation values

### How to read these tables

**Historical** means a documented case or scholarly reconstruction.  
**Analogue** means modern evidence useful for physical or operational calibration.  
**Proposed** means an initial TCE parameter, not an empirical historical estimate.

Confidence applies **within the stated context**. A well-documented nineteenth-century vehicle does not establish a universal ancient capacity.

### 2.1 Walking and pedestrian congestion

| Parameter | Value or range | Evidence and confidence |
| --- | --- | --- |
| Unimpeded adult walking speed | Approximately **1.2–1.4 m/s** | Modern analogue; medium confidence for transfer to ordinary historical walking. |
| Relatively unconstrained pedestrian density | Below approximately **0.7 persons/m²** | Modern analogue, not an ancient street measurement. |
| Increasing restriction of passing | Approximately **0.7–2.3 persons/m²** | Modern analogue. |
| Frequent close contact | Above approximately **2.3 persons/m²** | Modern analogue; not a safe operating target. |
| Peak specific flow | Approximately **1.2 persons/(m·s)** | Approximate reading of the referenced speed–density curve. |
| Near-jam density in that curve | Approximately **5.4 persons/m²** | Model-specific analogue; emphatically not a safety limit. |

Source: the Weidmann relationship reproduced and discussed by Seyfried and colleagues. Their paper explicitly distinguishes facilities, directional patterns, and ordinary movement from pushing or panic conditions. [arXiv](https://arxiv.org/pdf/physics/0506170)

For TCE, modify individual free speed for load, grade, surface, fatigue, mobility, and weather. Do not apply the pedestrian curve directly to a mixed street containing animals and carts.

### 2.2 Freight and passenger conveyance

| Mode | Capacity or crew | Travel speed | Evidence and confidence |
| --- | --- | --- | --- |
| Human porter, Mesoamerican comparison | Approximately **25 kg**, inferred from Hassig’s canoe/porter comparison | — | Historical reconstruction; medium-low. |
| Wheelbarrow | **100–120 kg** | **3–4 km/h** | Modern rural engineering analogue; medium. |
| Handcart | **400–700 kg** | **3–4 km/h** | Same analogue; not a default early wooden cart. |
| Pack animal, generic | **80–150 kg** | **3–5 km/h** | Same analogue; species and condition matter. |
| Animal-drawn cart | **400–2,000 kg** | **3–5 km/h** | Same analogue; upper range requires appropriate equipment and conditions. |
| Tenochtitlan-area cargo canoe | Nearly **1,000 kg**; one paddler in the cited reconstruction | Roughly walking speed | Historical reconstruction; medium-low. |
| Indian palanquin example | **4–6 bearers**, generally one passenger | Not established here | Museum collection interpretation; medium for this type, not all litters. |

The wheeled and animal figures come from Adeoti’s FAO-hosted engineering paper; the porter/canoe comparison is Hassig’s; the palanquin crew is documented by the Sarmaya collection. [FAOHome](https://www.fao.org/4/x5483b/x5483b0w.htm)

These are **not interchangeable rated payloads**. The engineering table includes later equipment and operating contexts; its upper values should not automatically be assigned to early carts. Cargo volume, axle strength, packaging, road condition, and endurance must also constrain a load.

### 2.3 Proposed TCE starting values

All values below are **design priors with low historical confidence**, intended for sensitivity testing.

| Parameter | Proposed initial value | Implementation note |
| --- | --- | --- |
| Routine porter payload | **20–30 kg** | A gameplay/calibration starting range, not a universal safe-load recommendation. |
| Early handcart payload | **150–300 kg** | Deliberately below the engineering analogue’s larger vehicles. |
| Early animal-cart payload | **300–800 kg** | Increase only with stronger vehicles, suitable animals, and roads. |
| Unencumbered free walking speed | **1.2 m/s** | Give individuals variation rather than one identical speed. |
| Footpath authored width | **0.8–1.5 m** | Geometry prior, not a historical worldwide width distribution. |
| Local street authored width | **2–4 m** | Distinguish property-to-property width from usable width. |
| Main street authored width | **4–8 m** | Wider exceptional spaces should remain possible. |
| Loading setup time | **2–5 minutes/stop** | Finding the recipient, positioning, opening storage, securing the vehicle. |
| Handling productivity | **5–20 kg/(worker·minute)** | Replace with explicit carrying cycles where handling dominates. |
| Daily destination-travel sensitivity range | **45–90 minutes/person/day** | Optional soft-cost experiment, never a compulsory allocation or hard cap. |
| Familiar route alternatives | **2–4 routes/destination class** | Proposed bounded-knowledge optimization. |

For street widths, the correct long-term calibration method is to sample particular archaeological or historical plans. A single “medieval street width” would conceal more than it explains.

**Parameters not justified by the reviewed evidence:** universal trips per person per day, percentage working at home, urban cart ownership, average historical delivery dwell time, and ancient road capacities in vehicles/hour. These should remain exposed assumptions until tied to a specific reconstructed settlement.

---

## 3. Variation across eras and regions

### 3.1 Behavioral regimes, not calendar unlocks

| Context | Evidence or boundary condition | TCE implication |
| --- | --- | --- |
| **Foragers** | A Hadza GPS study covering **2,078 person-days** reported adult daily movement averaging approximately **7.6 km for women and 12.9 km for men**. Much of this movement was productive foraging activity, not commuting. These are population-specific contemporary observations, not Paleolithic universal constants. [Nature](https://www.nature.com/articles/s41562-020-01002-7) | Food acquisition can itself be a moving activity. Do not force it into an urban travel-time budget or hard-code the observed sex difference globally. |
| **Early farming** | Direct daily movement records are thin. Rural transport studies offer analogues involving fields, markets, water, fuel, and household tasks, but are not measurements of Neolithic villages. [FAOHome](https://www.fao.org/4/x5483b/x5483b0w.htm) | Generate farm and provisioning journeys from actual plot and resource locations. Harvest transport should respond to crop volume and available labor. |
| **Pre-industrial cities** | Street commerce, portering, animal transport, and specialized handling institutions appear in different combinations across the cases below. | Congestion should emerge at shared spaces and transfer points; substantial routine movement need not be a home-to-central-workplace commute. |
| **Industrial cities** | Historical commuting studies become more informative; Pooley and Turnbull examine changing British journeys to work from 1890 onward. Their geographic and chronological scope matters. [ScienceDirect](https://www.sciencedirect.com/author/7004485181/colin-g-pooley) | Add larger concentrated workplaces, scheduled transport, fares, and increasingly differentiated residential and employment locations. |
| **Modern cities** | Travel time continues to vary by people, activities, and urban structure rather than following a universal individual constant. [ideas.repec.org](https://ideas.repec.org/a/eee/transa/v38y2004i9-10p643-675.html) | Retain activity-based behavior; add motorized networks without replacing household decision-making with one fixed commute template. |

### 3.2 Regional configurations worth reproducing

**Mediterranean and European cities.** Rome supplies unusually explicit regulation, while Pompeii supplies unusually rich physical evidence. Neither should become the template for all towns. Medieval English and Scandinavian research also shows street and gutter management as a cooperative problem involving local responsibilities, rather than simply a centrally operated road service. For TCE, separate ownership, maintenance obligations, enforcement, and actual maintenance labor. [JSTOR](https://www.jstor.org/stable/40061427)

**Song China.** The contrast between ward-based urban organization and the more open commercial streets of Song capitals matters as much as vehicle technology. Columbia’s treatment describes shops and markets extending along streets; the Qingming bridge scene shows commerce occupying a crossing itself. The modeling consequence is that institutional opening of streets and commercial frontage can change where movement concentrates without any new transport mode. [AFE East Asia](https://afe.easia.columbia.edu/songdynasty-module/cities-new.html)

**Early-modern Japan, especially Osaka.** Waterborne distribution connected boats, storehouses, stevedores, and delivery porters. Occupational rights and organizations shaped access to work. Model a wharf as an economic institution with crews, claims, and storage—not merely a water-to-road connector. A license holder and a cargo handler need not be the same person. [BPB](https://bpb-us-e1.wpmucdn.com/blogs.uoregon.edu/dist/8/12656/files/2018/06/Morishita-Stevedores-2012-2d26imx.pdf)

**South Asia.** Palanquins demonstrate how passenger transport can consume several workers while serving relatively few passengers. The Sarmaya examples connect the vehicle to both elite comfort and practical travel. TCE should allow status, protection, bodily mobility, and access restrictions to influence mode choice; minimizing travel time alone will not reproduce this transport demand. [Sarmaya](https://sarmaya.in/spotlight/song-of-the-palanquin-bearers/)

**Middle Eastern settings.** Bulliet’s *The Camel and the Wheel* argues that the relative economics of pack transport helped explain displacement of wheeled transport in parts of the region. Treat this as a historically situated thesis, not a universal rule that a particular culture “rejects wheels.” In TCE, animal availability, route geometry, maintenance costs, and shipment size should make packing competitive even after carts exist. [Google Books](https://books.google.com/books/about/The_Camel_and_the_Wheel.html?id=Vnf74PlZ7Z8C)

**Mesoamerica.** Tenochtitlan’s transport possibilities depended on lake and canal access. Hassig also describes seasonal changes in water levels and wind that could slow or interrupt navigation. Thus navigable capacity should vary with water and weather; a water-connected settlement may have broad supply access without faster walking or wheeled freight. [Arqueología Mexicana](https://arqueologiamexicana.mx/mexico-antiguo/tenochtitlan-en-el-gran-lago)

**The Andes.** Smithsonian’s Inka Road interpretation describes official users, llama caravans, and restrictions requiring ordinary travelers to obtain permission. It also emphasizes adaptation to local terrain. The lesson is not simply “pack animals instead of carts”: network access itself can be political, and substantial infrastructure can serve state movement rather than unrestricted household travel. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/question/who-used-inka-road.html)

**Southeast Asia.** Angkor demonstrates a dispersed urban configuration in which water management and settlement networks extended far beyond a compact center. Do not infer that every mapped canal was navigable or that residents routinely crossed the whole complex. Model local activity areas embedded in a larger infrastructural landscape. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0702525104)

**Africa requires an explicit evidence warning.** The Hadza study and Nigerian rural transport evidence are valuable for particular movement and transport mechanisms; neither establishes the urban traffic composition of precolonial African cities. African city-specific calibration remains a gap in this evidence set. Importing a Roman street system or European vehicle fleet would conceal that gap rather than resolve it.

---

## 4. Stylized facts and validation targets

### 4.1 Historically grounded signatures

A convincing simulation should reproduce the following **qualitative patterns**, not one universal numerical target.

| Pattern | What to measure in TCE |
| --- | --- |
| **Movement competes with street activities.** The Roman and Song evidence includes uses beyond circulation. [Cambridge University Press](https://www.cambridge.org/core/books/roman-street/life-in-the-street/837A59424D14E5415F9B2663EB6B35F4) | Delay near markets, workshops, loading positions, and events; usable width versus physical width. |
| **Vehicle circulation depends on local geometry.** Pompeii’s evidence makes route-specific access and maneuvering central. [OUP Academic](https://academic.oup.com/book/7089) | Locations inaccessible to carts but reachable by people; detours, reversing events, and transfer demand. |
| **Cargo handling is an organized production activity.** Osaka supplies a particularly clear example. [BPB](https://bpb-us-e1.wpmucdn.com/blogs.uoregon.edu/dist/8/12656/files/2018/06/Morishita-Stevedores-2012-2d26imx.pdf) | Waiting for crews, warehouse access, handling-hours per tonne, and monopolized versus open facilities. |
| **Travel behavior varies across people and activities.** Aggregate time regularities should not erase that variation. [ideas.repec.org](https://ideas.repec.org/a/eee/transa/v38y2004i9-10p643-675.html) | Full distributions of daily travel time and distance, separated by occupation and trip purpose. |
| **Regulation differentiates purposes and users.** The Roman legal text and Inka access rules are examples. [Roman Law Library](https://droitromain.univ-grenoble-alpes.fr/Anglica/heracleensis_johnson.html) | Who can use each route, when, at what cost, and with what consequences for others. |

A useful overall signature is **concentration of delay at particular locations and times**. TCE does not need universal citywide gridlock to demonstrate historically plausible congestion.

### 4.2 Numerical engineering tests

The following are **constructed tests**, not recovered historical traffic observations.

**Walking consistency.** At the proposed 1.2 m/s baseline, 1 km takes **13.9 minutes** before stops or congestion. The same journey must consume the same simulation time whether visible, off-screen, or fast-forwarded.

**Payload arithmetic.** Moving 1 tonne requires **40 loaded trips at 25 kg**, **two at 500 kg**, or **one at 1,000 kg**. Add empty returns unless a backhaul exists. Loading and unloading remain necessary.

**Pedestrian versus freight blockage.** Using an idealized specific flow of 1.2 persons/(m·s), a 2 m clear passage has an approximate theoretical throughput of **8,640 people/hour**. This is an uncluttered pedestrian reference, not historical mixed-street capacity. One unloading cart can still eliminate wheeled passage. [arXiv](https://arxiv.org/pdf/physics/0506170)

**Loading bottleneck.** One unloading position occupied for ten minutes can serve at most **six vehicles/hour**, regardless of road width. Arrivals above that rate must queue, divert, or wait elsewhere.

**Conservation under disruption.** Closing a gate should delay or reroute people and goods. It must not erase cargo, complete deliveries remotely, or free a worker who remains physically occupied.

**Mode substitution.** Better cart access should usually reduce the labor needed for a fixed heavy shipment—but can increase obstruction and require transfer to porters for the final narrow segment. This is a modeled consequence of the payload, clearance, and service rules, not an assumed universal outcome.

For calibration, keep **person-trips, person-kilometers, tonnes, tonne-kilometers, and labor-hours** separate. A city may be overwhelmingly pedestrian by passenger trips while boats dominate heavy freight movement.

---

## 5. Recommended TCE representation

### 5.1 Minimum state model

Use five connected systems:

| System | Essential state |
| --- | --- |
| **People and households** | Activities, destinations, obligations, possessions, mobility, time availability, and responsibilities for provisioning. |
| **Transport resources** | Porters, animal teams, carts, litters, boats, crews, payloads, condition, ownership, and availability. |
| **Shipments** | Origin, destination, mass, volume, packaging, ownership, deadline, reservations, and current location. |
| **Network and facilities** | Width, slope, surface, steps, clearance, turns, water depth, entrances, berths, storage, and queue space. |
| **Institutions** | Access permissions, operating hours, tolls, loading rights, maintenance duties, enforcement, and exemptions. |

Treat home workshops, storehouses, and markets as locations with actual entrances. A building’s centroid is not an adequate loading point.

### 5.2 Use a hybrid movement model

For 10k–50k people, I recommend **individual identities with mesoscopic movement and explicit local bottlenecks**:

**Between bottlenecks:** agents advance through network segments using expected speed, occupancy, and scheduled arrival events.

**At bottlenecks:** allocate actual finite space and service. Narrow crossings, corners, blocked streets, loading areas, and gates need local queues and conflict resolution.

**Where pedestrians crowd:** use a density–speed relationship, with separate treatment for opposing flows and obstacles.

**Where carts meet:** use footprints, swept turning clearance, yielding, passing places, and reversal. Do not convert carts into a fixed number of pedestrians and assume that reproduces maneuvering.

A stalled encounter needs a resolution mechanism: yielding precedence, backing up, negotiation, enforcement, or physical inability to complete the trip. It must not end through invisible overlap.

### 5.3 Keep routing informed but imperfect

Compute traveler costs from time, effort, fees, safety, and access. Compute freight costs from crew and equipment costs, handling, delay, and loss.

Use a small set of familiar feasible routes. Replan when destinations change, a route becomes unusable, or persistent delay exceeds a threshold—not every rendered frame. Introduce hysteresis so all agents do not switch simultaneously between two temporarily attractive routes.

For long-term emergence, connect repeated transport costs to decisions about residence, shop location, warehouses, road improvements, and new markets. These are proposed behavioral feedbacks; their strengths should be calibrated, not assumed.

### 5.4 Preserve the economics of time

The Rust kernel should remain authoritative. Unreal can interpolate movement and animate loading, but camera distance must not change delivery time or congestion.

For fast-forward, aggregate movement **without deleting individual commitments**. Preserve identities, shipment quantities, in-transit inventories, labor occupancy, travel times, and queues when switching detail levels.

Do not simulate a whole day’s work while agents move only a few minutes’ worth of physical distance. Explicitly define the conversion between wall-clock time, simulation time, and movement.

These architectural choices are **performance hypotheses, not a demonstrated 50k-agent benchmark**. Benchmark active travelers, path searches, contested bottlenecks, and simulated days per second alongside frame rate.

### 5.5 Models and games worth borrowing from

| Model or game | Useful ideas | Important limitation |
| --- | --- | --- |
| **MATSim** | Activity plans, network loading, queue simulation, and freight organization. Its user guide includes QSim and freight components. [MATSim](https://matsim.org/docs/userguide/) | Borrow the abstractions; do not assume a modern transport-demand setup directly represents household production or shared historical streets. |
| **SUMO pedestrian simulation** | Pedestrian interaction, lateral movement, and mixed pedestrian–vehicle testing. [Eclipse SUMO](https://sumo.dlr.de/docs/Simulation/Pedestrians.html) | Its documented jam fallback can allow pedestrians through obstacles after prolonged waiting. That is unacceptable as an invisible solution to economically meaningful TCE blockages. |
| **JuPedSim** | Reference scenarios for pedestrian bottlenecks, geometry, queues, and movement-model comparison. [Jupedsim](https://www.jupedsim.org/stable/) | Best used to calibrate local crowd behavior, not as a complete civilization-scale activity and freight economy. |
| **Ostriv** | Visible historical logistics, wagons, horse care, equipment replacement, and construction hauling. Developer notes explicitly discuss several of these dependencies. [Ostriv](https://ostrivgame.com/alpha-3-patch-1/) | A useful design reference, not evidence that historical crowd dynamics or traffic capacities have been validated. |

### 5.6 Post-v1: omnibus and tram

An omnibus should emerge as a **scheduled transport business**, not simply a larger personal carriage.

A concrete calibration case is Shillibeer’s London service, begun in **1829**: **22 passengers**, **three horses**, **four to five runs per day**, and a **one-shilling fare** in the archive account. These are service-specific observations, not a general horse-bus specification. [Google Arts & Culture](https://artsandculture.google.com/story/horse-buses-in-london-tfl-archives/GwWxYWh0IiD1Wg?hl=en)

Its agents need a route, stopping policy, departure schedule, fare collection, seats, crews, stabling, and replacement horses. Passenger choice should compare:

\[
T\_{\text{door-to-door}}
=
T\_{\text{access}}+T\_{\text{wait}}+T\_{\text{ride}}
+T\_{\text{transfer}}+T\_{\text{egress}}
\]

For perfectly regular service with random passenger arrivals, mean waiting is half the headway. Schedule-aware arrivals and irregular service invalidate that simple approximation.

A constructed service with 22 seats every fifteen minutes supplies **88 seats/hour/direction**. This illustrates why vehicle capacity and service frequency must both be represented.

Trams add track-constrained movement, infrastructure maintenance, and greater dependence on keeping the route clear. Electric operation additionally requires power and supporting technology. A London museum vehicle record documents a horse tram built in New York in **1882**, used in London, and replaced by electric operation in **1910**—a useful reminder that technologies coexist and spread through procurement rather than changing everywhere at one date. [Google Arts & Culture](https://artsandculture.google.com/asset/london-tramways-company-double-deck-horse-tram-no-284-built-by-john-stephenson-co-new-york/7wH--wEGRvbPTA)

For TCE, unlock these systems through materials, engineering knowledge, investment, passenger demand, operating institutions, and energy supply—not fixed eras.

---

## 6. Sources, datasets, and unresolved evidence

### Practical calibration resources

| Resource | Best use | What it does not establish |
| --- | --- | --- |
| **Pompeii Bibliography and Mapping Project; Pompeii Linked Open Data** | Spatial identifiers, entrances, archaeological context, and links between mapped places and evidence. P-LOD describes downloadable data and geographic interfaces. [Ancient World Institute](https://isaw.nyu.edu/library/blog/pompeii-lod-part-one) | Historical origin–destination matrices, trip frequencies, or measured congestion. |
| **Pedestrian Dynamics Data Archive, Forschungszentrum Jülich** | Modern trajectories and experiments for checking bottlenecks and pedestrian interaction. The associated data-guidance paper describes experiments involving approximately 1,000 participants. [arXiv](https://arxiv.org/abs/2303.02319) | Historical clothing, cargo, animal behavior, or mixed-use street demand. |
| **Greater Angkor mapping research** | Testing dispersed settlement and infrastructure layouts against a compact-city assumption. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0702525104) | Daily resident journeys or proof that every water feature supported navigation. |
| **TfL Corporate Archives and London Transport Museum collections** | Specific vehicles, capacities, schedules, fares, ownership, and operational change. [Google Arts & Culture](https://artsandculture.google.com/story/horse-buses-in-london-tfl-archives/GwWxYWh0IiD1Wg?hl=en) | A global industrial-transition timetable or directly portable in-game prices. |

### Core scholarly reading

The most useful foundational works are **Poehler’s *The Traffic Systems of Pompeii* (2017)** for circulation reconstruction; **Hartnett’s *The Roman Street* (2017)** for streets as occupied social spaces; and **Holleran’s *Shopping in Ancient Rome* (2012)** for retail organization. Together they help prevent a road-only interpretation of urban movement. [OUP Academic](https://academic.oup.com/book/7089)

For behavior and quantitative calibration, pair **Marchetti (1994), “Anthropological invariants in travel behavior,”** with **Mokhtarian and Chen (2004), “TTB or not TTB.”** Use **Seyfried et al. (2005), “The Fundamental Diagram of Pedestrian Movement Revisited,”** for physical movement rather than historical travel demand. [IIASA](https://pure.iiasa.ac.at/id/eprint/4071/)

For non-European institutions and configurations, particularly useful sources are **Hassig’s “Rutas y caminos de los mexicas” (2006)**, **Morishita’s “Stevedores and stevedores’ guilds” (2012)**, and **Evans et al.’s Angkor mapping study (2007)**. [Arqueología Mexicana](https://arqueologiamexicana.mx/mexico-antiguo/tenochtitlan-en-el-gran-lago)

### What remains uncertain

**Regulations are not enforcement records.** A prohibition establishes an institutional possibility, not compliance rates or its actual effect on delay.

**Images and literary complaints are not traffic surveys.** They reveal activities and perceived conflicts, but cannot reliably supply vehicles/hour, queue lengths, or average commuting time.

**Physical traces require interpretation.** Street wear and geometry constrain possible circulation; they do not uniquely reconstruct every trip.

**Modern analogues are not historical constants.** Pedestrian experiments are useful for bodies moving through space. Rural engineering tables are useful for transport capabilities. Neither supplies a complete historical operating environment.

**The largest calibration gap is demand:** who traveled, how often, for which purposes, carrying what, and at what times. TCE should expose those assumptions and test alternative household and institutional arrangements instead of hiding them behind a universal daily commute.

## Bottom line for v1

Build **walking, human hauling, inventory-triggered shipments, usable street width, finite loading queues, and rule-based access** first. Add animals and carts when their physical and economic prerequisites exist. Include boats early wherever navigable water is available: they can transform freight economics without increasing passenger speed.

The result should be a settlement where most movement is ordinary and local, but a busy market entrance, obstructed lane, understaffed landing, or restricted gate can cause a consequential traffic problem. That is a stronger foundation for TCE than either unconstrained walking agents or a modern automobile model dressed in historical assets.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92936-6c68-83e9-b2a7-6517ef9a24f8)
