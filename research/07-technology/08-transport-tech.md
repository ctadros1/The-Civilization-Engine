# Transport and navigation technology for The Civilization Engine

## Design conclusion

**TCE should model transport as a production system, not a movement-speed bonus.** A journey consumes working time, food or fuel, vehicle capacity, infrastructure, and sometimes lives or cargo. Technologies change those requirements—and the routes that are feasible.

The most important historical distinction is between **moving quickly, moving a heavy load cheaply, and delivering it reliably**. These are different achievements. Roman transport reconstructions, for example, put sea freight far below road freight in cost, while distinguishing sharply between upstream and downstream river movement. They do not imply that every ship arrived sooner than every wagon. [ifo Institut](https://www.ifo.de/DocDL/cesifo1_wp7740.pdf)

For TCE, the central quantity should therefore be:

\[
\boxed{\text{Delivered transport capacity}
=\frac{\text{useful cargo delivered}}{\text{complete operating cycle}}}
\]

The operating cycle includes loading, outward travel, unloading, waiting, provisioning, and repositioning—not just the visible journey.

**Evidence notation:** **E** denotes observations or historical records; **R** denotes scholarly or engineering reconstructions; **P** denotes a proposed TCE parameter. Confidence is **H/M/L**, meaning high, medium, or low for the stated use. A well-documented modern animal-transport observation can still be only a medium-confidence analogue for antiquity. Ranges below are not statistical confidence intervals.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Capacity is constrained by several different things

Keep separate limits for mass, volume, stability, structural strength, and route compatibility:

\[
\sum\_i m\_i \leq Q\_{\text{payload}},
\qquad
\sum\_i V\_i \leq V\_{\text{usable}}
\]

Here, passengers, provisions, fuel, and commercial cargo all consume payload. Bridge limits apply to **vehicle tare plus payload**; vessel draft depends on **total displacement**, not commercial cargo alone.

Consequently, wool, grain, timber, stone, and passengers should not use identical “cargo slots.” Wool may fill a hold before reaching its weight limit. Stone may reach the weight limit first. Long timber requires appropriate stowage or rafting. An animal’s nominal carrying capacity should decrease with poor condition, badly fitted equipment, and difficult terrain.

**Implementation rule:** give each transport asset a physical specification, then calculate usable cargo for the actual journey. Do not store one invariant “100 units” capacity.

### 1.2 Wheels substitute infrastructure and traction for carrying effort

For a wheeled vehicle, a useful first approximation is:

\[
F\_{\text{required}}
\approx Mg\left(C\_{rr}\cos\theta+\sin\theta\right)
\]

where \(M\) is the mass being pulled, \(C\_{rr}\) rolling resistance, and \(\theta\) slope. Sustainable speed is then limited by available drawbar power, animal gait, traction, and braking.

This produces an important nonlinear effect. In an **illustrative calculation**, a cart with \(C\_{rr}=0.03\) requires roughly \(0.03Mg\) on level ground but \(0.08Mg\) on a 5% ascent: about **2.7 times the pulling force**. A modest hill can therefore matter more than a modest improvement in wheel construction.

Model two-wheel carts and four-wheel wagons differently. Two-wheel vehicles require load balancing around the axle; four-wheel vehicles require workable steering and generally more construction and maintenance. Neither requires iron wheels as a universal prerequisite.

Harness improvements should improve fit, force transmission, maneuverability, and injury rates—not suddenly make animal traction possible. Experimental and archaeological work rejects the familiar claim that ancient horse harness necessarily choked horses and prevented effective pulling. The geographical origin and interpretation of the full horse collar are also contested. [Academia](https://www.academia.edu/116311848/The_origin_of_the_horse_collar_2022)

### 1.3 Water transport has high capacity, but strong route and weather constraints

Represent waterways as **directed, seasonally changing edges**.

For paddled or powered vessels, current alters speed over the ground. For sailing vessels, use a simple wind-performance lookup by wind strength and angle. Towing and poling require their own operating modes: a towpath, accessible banks, suitable depth, or additional labor.

A downstream route may permit cheap rafting but no economical return of the same craft. A river may be navigable in one season and too shallow or dangerous in another. Coastal water can connect two places economically even when their direct land separation is small.

Do not impose a universal “ocean navigation requires compass” gate. Practitioners of Pacific wayfinding document navigational systems using stars, swell patterns, winds, birds, and landfall strategies without magnetic instruments. These are demanding learned skills, not random drifting. [Hokule'a Archive](https://archive.hokulea.com/navigate/navigate.html)

### 1.4 Journey speed is not underway speed

Calculate:

\[
T\_{\text{delivery}}
=
T\_{\text{movement}}
+T\_{\text{rest}}
+T\_{\text{handling}}
+T\_{\text{queues}}
+T\_{\text{weather}}
+T\_{\text{administration}}
\]

Land carriers usually cannot turn an animal’s walking speed into 24-hour daily progress. Ships can continue overnight with appropriate crews, but may wait for weather, tides, or safe harbor access.

Navigation improvements should primarily reduce **route error, uncertainty, forced waiting, and grounding exposure**. They need not make the hull physically faster. Zhu Yu’s early-twelfth-century account explicitly combines sun and star observations, compass use in obscured conditions, and examination of seabed material; it also describes seasonal departure and return winds. [Wikisource](https://zh.wikisource.org/wiki/%E8%90%8D%E6%B4%B2%E5%8F%AF%E8%AB%87/%E5%8D%B7%E4%BA%8C)

### 1.5 Animals consume transport capacity—and require support while idle

Model an animal as a living productive asset with maintenance requirements, condition, training, and seasonal work availability.

An FAO account of Bactrian camels reports approximately **6–12 kg of dry matter consumed daily in summer**, alongside long-distance load and travel estimates. Such requirements are ecology-specific, but demonstrate why a caravan cannot simply carry unlimited additional provisions without sacrificing commercial cargo. [FAOHome](https://www.fao.org/4/x1700t/x1700t05.htm)

Allow three provisioning strategies: carry supplies, purchase them en route, or forage. They exchange different costs. Foraging consumes time and local vegetation; purchasing depends on settlements and prices; carrying supplies reduces saleable payload.

For an illustrative TCE caravan, a 200 kg payload animal carrying ten days of 8 kg/day fodder has already allocated 80 kg to its own feed before human supplies or water. That is an accounting example, not a universal camel prescription.

### 1.6 Costs should arise from complete trips and local prices

A carrier’s expected trip cost can be represented as:

\[
C\_{\text{trip}} =
C\_{\text{labor}}+
C\_{\text{feed/fuel}}+
C\_{\text{wear}}+
C\_{\text{capital}}+
C\_{\text{handling}}+
C\_{\text{fees}}+
E[C\_{\text{uninsured loss}}]
\]

Insurance premiums can substitute for some uninsured risk; do not count both the full expected loss and its insured replacement.

For auditing transport productivity:

\[
c\_{\text{cycle}} =
\frac{C\_{\text{complete cycle}}}
{\sum\_j Q\_jD\_j}
\]

The denominator includes laden legs, while the numerator includes empty repositioning. Freight prices can additionally reflect competition, bargaining, monopolies, urgency, and willingness to pay.

**Separate private payments from social resource costs.** A toll is income to an institution, not destroyed labor. Conversely, a publicly maintained road is not resource-free simply because the traveler pays no toll.

### 1.7 Infrastructure is a complementary network, not a blanket modifier

A useful road ending at an impassable river may accomplish little. A ferry or bridge can unlock the whole route. A larger ship is useless where the approach channel or quay cannot accommodate it.

| Infrastructure | Required simulation properties | Principal bottleneck |
| --- | --- | --- |
| Track or road | Width, surface, drainage, slope, maintenance, permissions | Seasonal passability and vehicle resistance |
| Bridge or ferry | Load/width limits, crossing time, queue, flood vulnerability | Crossing capacity and continuity |
| Landing or port | Approach depth, shelter, berths, loaders, storage, repair | Handling throughput and vessel access |
| Navigable river | Current, depth, rapids, banks, towpath access | Directional feasibility and season |
| Canal and locks | Water supply, depth, chamber dimensions, gates, maintenance | Water availability and lock queues |
| Waystation | Food, fodder, water, lodging, replacement animals | Provisioning and relay availability |

These are recommended state variables, not universal historical dimensions.

Infrastructure investment should occur when expected benefits justify its construction and upkeep **to an actor able to finance and organize it**. Benefits may be commercial, military, fiscal, or communal. China’s Grand Canal illustrates the combination of engineering, grain transport, storage, and state administration rather than an isolated canal invention. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1443/)

### 1.8 Freight, armies, migration, and commuting need different objectives

For **freight**, optimize expected margin subject to capacity, spoilage, and deadlines.

For **armies**, movement depends on the formation and supply train, not the fastest rider. Camps, regrouping, provisioning, and military priorities belong in the itinerary.

For **migration**, households carry people, dependent animals, tools, and selected possessions. Allow them to abandon or sell bulky property.

For **commuting**, use the person’s daily time budget. At an assumed walking speed of 4 km/h, a 30-minute one-way budget gives approximately a 2 km route distance. Better bridges and street connectivity may enlarge accessible employment without changing walking speed.

For **messages**, permit relays. Relay infrastructure allows different people or animals to carry successive stages; it must not confer the same speed on an entire caravan.

---

## 2. Parameters: capacities, speeds, and costs

### 2.1 Physical operating presets for v1

The following are **recommended starting distributions**, not averages for entire civilizations. “Travel-day progress” includes ordinary daily rest and short stops, but excludes exceptional multi-day weather waits and long terminal delays.

| Mode | Useful cargo per carrier/vehicle | Underway speed | Travel-day progress | Basis and confidence |
| --- | --- | --- | --- | --- |
| Human porter | 15–30 kg | 3–5 km/h | 15–25 km/day | P/M; rural-transport analogues |
| Pack donkey, pony, or mule | 50–150 kg, species-dependent | 3–5 km/h | 20–30 km/day | P/M; comparative animal-transport evidence |
| Pack llama | 20–40 kg | 3–4 km/h | 15–25 km/day | P/L–M; Andean travel estimates |
| Bactrian pack camel | 150–240 kg | 4–6 km/h | 30–40 km/day | E/M for the reported husbandry context |
| Handcart or wheelbarrow | 80–180 kg | 3–5 km/h | 10–20 km/day | P/M; surface and design sensitive |
| Early wooden ox cart | 0.3–1.0 t | 2–4 km/h | 10–25 km/day | P/M; lower than some improved-cart analogues |
| Improved horse/mule wagon | 0.5–2.0 t | 4–6 km/h | 20–35 km/day | P/M; team size and road quality matter |
| Working dugout/canoe | 0.1–0.7 t | 3–6 km/h | 15–35 km/day | P/L; hull-specific |
| Small river barge | 5–50 t | 2–5 km/h | 20–40 km/day | P/L–M; towing/current assumptions essential |
| Small sailing coaster | 4–50 t | 4–11 km/h | 40–120 km/day | P/M; weather and operating practice dominate |
| Larger preindustrial merchant ship | 50–500 t | 6–12 km/h | 70–200 km/day | P/L–M; an authoring band, not a universal ship class |

Several anchors constrain these presets:

* Sieber’s comparative rural-transport table gives **70–150 kg at 3–5 km/h for pack animals**, **500 kg at 5–7 km/h for horse/donkey carts**, and **1,000 kg at 3–5 km/h for ox carts**. These are useful analogues, but modern wheels and roads must not be silently back-projected into early agrarian societies. Its “range” column is **not** a measure of daily progress. [Niklas Sieber](https://www.niklas-sieber.de/Publications/Transporting_Yield.pdf)
* FAO reports Bactrian loads of **150–240 kg**, summer travel of **4–6 km/h**, and **30–40 km/day** on journeys lasting weeks. [FAOHome](https://www.fao.org/4/x1700t/x1700t05.htm)
* An Andean archaeological study uses approximately **20 km/day for llama caravans**, with rest days, drawing on ethnohistorical research. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/beyond-exotic-goods-wari-elites-and-regional-interaction-in-the-andes-during-the-middle-horizon-ad-6001000/7191A5FCBFAA14804C010573A8DC759A)
* The Viking Ship Museum estimates about **4 t of cargo** for the eleventh-century coastal trader Skuldelev 3. That is cargo, not displacement. [Vikingeskibsmuseet](https://www.vikingeskibsmuseet.dk/en/visit-the-museum/exhibitions/the-five-viking-ships/skuldelev-3)
* The *Jewel of Muscat* reconstruction, based on a ninth-century sewn ship, achieved approximately **4–6 knots in moderate winds**; tank tests suggested comfortable cruising around 6 knots. Reconstruction choices and modern interventions limit direct historical inference. [Smithsonian APA Center](https://asia.si.edu/wp-content/uploads/2023/06/shipwrecked-08-vosmer.pdf)

Heavy vehicles should be additional recipes, not automatic upgrades to every wagon: British evidence indicates some freight wagons carried **6–7 tons by about 1800**, supported by improved vehicles, teams, and roads. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/transport_revolution_cehmbjuly92013_forweb.pdf)

### 2.2 Historical freight costs: retain the original context

There is no defensible single currency conversion that makes an ancient porter, a Roman ship, an English canal boat, and a modern truck directly comparable. Preserve historical nominal prices alongside a resource-based TCE cost calculation.

#### Roman comparative benchmark

A recent Roman-network reconstruction uses:

| Mode | Relative transport cost |
| --- | --- |
| Sea | 1 |
| Downriver | 5 |
| Upriver | 10 |
| Road | 52 |

These are **R/M**, substantially grounded in Diocletian’s price edict and reconstruction assumptions. The edict supplies legal maximum prices, not a representative sample of actual transactions. Use the ordering and rough magnitude as a validation case, not immutable Roman—or universal—multipliers. [ifo Institut](https://www.ifo.de/DocDL/cesifo1_wp7740.pdf)

#### England and Wales: reconstructed network parameters

Alvarez-Palau, Bogart, and collaborators reconstruct freight parameters for 1680 and 1830. The following conversions assume the original “ton” is the British long ton: **1 long-ton-mile ≈ 1.635 metric tonne-km**. Values exclude the additional fixed charges discussed below. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/Multimodal%20model%201680%20and%201830%20EJ%20replication.pdf)

| Mode | 1680: historical pence/ton-mile | 1680: pence/metric t-km | 1830: historical pence/ton-mile | 1830: pence/metric t-km |
| --- | --- | --- | --- | --- |
| Coastal sea | 0.211 | 0.129 | 0.168 | 0.103 |
| Inland waterway | 1.00 | 0.612 | 2.00 | 1.223 |
| Better road, level | 9.97 | 6.10 | 7.50 | 4.59 |
| Poorer road, level | 11.20 | 6.85 | 9.87 | 6.04 |

**Confidence: R/M.** These are historically informed model inputs, not comprehensive national tariff schedules. Road gradients add further costs. In 1830 the reconstruction also assigns a **22.9 pence/ton seaport fee** and a **13.9 pence/ton transshipment charge** in the relevant circumstances. Fixed costs must not be mistaken for per-distance costs. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/Multimodal%20model%201680%20and%201830%20EJ%20replication.pdf)

The higher reconstructed inland-waterway rate in 1830 is not evidence that boats physically deteriorated: route composition, improvements, charges, and market conditions differed.

#### Early industrial rail

| Benchmark | Original nominal rate | Converted nominal rate | Confidence |
| --- | --- | --- | --- |
| Britain, rail, c.1845 | 0.15 shillings/ton-mile | ≈1.10 pence/metric t-km | R/M |
| Britain, rail, c.1865 | 0.10 shillings/ton-mile | ≈0.734 pence/metric t-km | R/M |

Conversions again assume long tons. These nominal prices should not be directly compared across centuries without a price or wage deflator. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/transport_revolution_cehmbjuly92013_forweb.pdf)

#### Rural small-scale transport

Sieber reproduces comparative cost estimates for two different operating scenarios:

| Mode | 50 km, good-road scenario | 5 km, poor-road scenario |
| --- | --- | --- |
| Headloading | $1.05/t-km | $1.54/t-km |
| Pack donkey | $0.77/t-km | $1.16/t-km |
| Ox cart | $0.11/t-km | $0.20/t-km |
| Medium truck | $0.11/t-km | $0.47/t-km |

These are **dated scenario estimates in the source’s US-dollar accounting**, not present-day prices. Distance and road quality change together, so they cannot identify the effect of road quality alone. They illustrate why a truck’s engineering advantage may disappear on short, poorly utilized services. **R/M.** [Niklas Sieber](https://www.niklas-sieber.de/Publications/Transporting_Yield.pdf)

### 2.3 A resource-based cost system for TCE

A useful cross-era measure is **worker-days per delivered tonne-km**. The following are calculations from explicitly assumed operating arrangements:

| Illustrative arrangement | Loaded throughput per travel day | Direct travel labor | With equal-time empty return |
| --- | --- | --- | --- |
| Porter: 25 kg, 20 km/day, one worker | 0.5 t-km | 2.00 worker-days/t-km | 4.00 |
| Eight pack animals: 75 kg each, 25 km/day, one handler | 15 t-km | 0.067 | 0.133 |
| Ox cart: 0.8 t, 20 km/day, one driver | 16 t-km | 0.063 | 0.125 |
| Canoe: 0.5 t, 30 km/day, two workers | 15 t-km | 0.133 | 0.267 |
| Coaster: 20 t, 80 km/day, five crew | 1,600 t-km | 0.0031 | 0.0063 |

**All values are P, not historical measurements.** They exclude feeding, animal rearing, handling, repairs, idle time, capital, and losses. Their purpose is to make those omitted costs visible rather than bury them inside arbitrary modal multipliers.

The pack-animal and cart examples also show why carts need not immediately dominate: handling ratios, local surfaces, load size, and animal ownership can offset the apparent payload advantage.

### 2.4 Post-v1 operating templates

These are **P/L–M scenario bands** for authoring assets, not measured global averages.

| Asset family | Cargo payload template | Underway speed template | Additional requirements |
| --- | --- | --- | --- |
| Mid-/late-nineteenth-century freight train | 100–1,000 t/train | 15–40 km/h | Track, locomotives, fuel, water, workshops, dispatch |
| Nineteenth-century steam freighter | 500–5,000 t | 12–22 km/h | Boilers, engines, fuel storage, repair facilities |
| Modern heavy freight train | 1,000–10,000 t/train | 30–80 km/h | Strong track/bridges, signaling, terminals |
| Modern road freight vehicle | 1–25 t/vehicle | 30–80 km/h | Roads, fuel distribution, tires, repair |
| Modern diesel cargo ship | 1,000–200,000 t | 18–30 km/h | Deep approaches, appropriate terminals, bunkering |

Never apply the upper end to every route or vehicle. Terminal dwell may dominate door-to-door performance.

For a modern cost anchor, ATRI reported **$2.270 per vehicle-mile in 2023 marginal truck operating costs**. At an assumed 20 t payload, that calculates to approximately **$0.071/t-km** while loaded, or **$0.141/t-km** with an equal-distance empty return at the same vehicle-mile cost. These are derived operating-cost examples, not quoted freight tariffs or full social costs. [LinkedIn](https://www.linkedin.com/posts/alex-leslie-2432651b7_supplychain-truckingdata-activity-7211777779837648897-Hkgu)

---

## 3. Variation across eras and world regions

| Setting | Historical pattern | Consequence for TCE |
| --- | --- | --- |
| **Forager waterways and coasts** | Boats predate agriculture in some regions. The Pesse dugout is dated to roughly 7920–6470 BCE; later Neolithic La Marmotta boats include a 10.43 m dugout. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0299765) | Water transport must be available without farming, metal tools, cities, or states. A forager settlement can have excellent aquatic mobility. |
| **Early agrarian Africa and Southwest Asia** | Donkey domestication has an inferred African origin around 5000 BCE; domestication and the subsequent development of transport practices are separate questions. [Science](https://www.science.org/doi/10.1126/science.abo3503) | Species availability, training, fodder, and equipment should determine transport options—not a generic “animal husbandry” unlock granting every animal. |
| **Eurasian steppe** | Genomic evidence places the major expansion of the ancestry of modern domestic horses around the late third millennium BCE; this is not proof that every earlier horse population was unused by humans. [Nature](https://www.nature.com/articles/s41586-021-04018-9) | Distinguish breeding populations, riding skills, packing, and vehicle traction. Do not require chariots before riding, or stirrups before mounted travel. |
| **Andes** | Qhapaq Ñan linked approximately 30,000 km of roads and associated facilities, reaching its greatest extent in the fifteenth century. Its engineering included stairs, bridges, drainage, and paving. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1459/) | An extensive state transport network need not be wagon-compatible. Give roads explicit mode compatibility and storage/service functions. |
| **China** | Grand Canal sections originated by the fifth century BCE, were integrated into a major network in the seventh century CE, and exceeded 2,000 km of artificial waterways by the thirteenth century. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1443/) | Large-scale bulk transport and hydraulic engineering are preindustrial possibilities. Canal administration and maintenance can sustain capitals independently of rail. |
| **Indian Ocean and adjacent seas** | Sewn-plank shipbuilding supported long-distance navigation. Historical accounts also describe voyages scheduled around seasonal winds rather than continuously interchangeable departures. [Smithsonian APA Center](https://asia.si.edu/wp-content/uploads/2023/06/shipwrecked-08-vosmer.pdf) | Permit sophisticated ships without iron-fastened hulls. Model seasonal service windows and stocks accumulated while awaiting departure. |
| **Pacific islands** | Ocean navigation could depend on expert oral knowledge rather than instruments or written charts. [Hokule'a Archive](https://archive.hokulea.com/navigate/navigate.html) | Navigators and teaching relationships are transport infrastructure. Losing a small group of experts can close practical routes without destroying boats. |
| **Africa’s varied environments** | Animal traction has markedly different histories across North Africa, Ethiopia, and other parts of the continent; later adoption in some regions cannot describe Africa as a whole. [Animal Traction Network](https://www.animaltraction.net/Improving92/Overviewpapers/ImprovingOverview1PStarkey.pdf) | Model local animal ecology, terrain, disease exposure, and institutions. Do not assign continental “transport levels.” |
| **Industrial and modern networks** | Rail expansion in colonial India reduced trade costs and regional price gaps and increased trade and real incomes in Donaldson’s analysis. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20101199) | New trunk transport should change market access. It should not remove the need for collection, delivery, warehouses, and local movement. |

A correct technological history is therefore branching: canoe networks, camel routes, stepped mountain roads, canals, sailing systems, and wheeled haulage can develop in different combinations.

---

## 4. Stylized facts and validation targets

These are better tests than reproducing a predetermined chronology.

| Pattern to reproduce | Quantitative or structural test |
| --- | --- |
| **Water can make distant places economically close.** | In a Roman-like benchmark, recover sea ≪ downstream river < upstream river ≪ road freight costs; compare against the 1:5:10:52 reconstruction without imposing it globally. [ifo Institut](https://www.ifo.de/DocDL/cesifo1_wp7740.pdf) |
| **Fixed handling costs preserve short-haul land transport.** | For equal route lengths, water becomes cheaper only after \(D^\*=H/(c\_{\rm road}-c\_{\rm water})\), where \(H\) is its extra handling cost. This is an accounting identity, not an estimated historical threshold. |
| **Road improvements can improve reliability as much as speed.** | British winter road freight rates before 1750 could be 30–50% above summer rates; by the early nineteenth century that premium was much smaller. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/transport_revolution_cehmbjuly92013_forweb.pdf) |
| **Faster services need not imply faster animals.** | British stagecoach journey speeds in one reconstruction rose from about 1.96 mph around 1700 to 7.96 mph around 1820—approximately 3.2 to 12.8 km/h. Test roads, staging, vehicle design, and organization together. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/transport_revolution_cehmbjuly92013_forweb.pdf) |
| **Engineering can raise payload by removing bottlenecks.** | Shen Kuo describes a Chinese lock improvement after which the initial grain-boat load rose from 300 to 400 *shi*, with larger vessels subsequently used. Preserve the relative increase; do not convert *shi* to tonnes without establishing the applicable unit and commodity. [Wikisource](https://zh.wikisource.org/wiki/%E5%A4%A2%E6%BA%AA%E7%AD%86%E8%AB%87/%E5%8D%B712) |
| **Network improvements integrate markets.** | After a route improvement, price gaps and transport-intensive production costs should generally fall where competition and supply allow it; trade should redirect. Donaldson’s Indian evidence is a strong empirical comparison. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20101199) |
| **Large infrastructure does not require wheeled freight.** | A mountain polity should be able to sustain a road-and-storehouse network while retaining porters and pack animals, as the Andean example demonstrates. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1459/) |
| **Technology does not guarantee commercial success.** | A larger or faster asset should lose money when underutilized, poorly matched to terminals, or burdened by empty returns. Test this as a model implication, not an exceptional scripted failure. |

Also test **seasonal inventories**. Closing a route should induce advance stocking where merchants can finance it, price increases where stocks run low, and substitution toward alternative routes. A permanently constant flow is a warning sign.

---

## 5. Modeling recommendation and technology graph

### 5.1 Architecture for 10,000–50,000 individual people

Use three connected layers.

**Physical network.** Roads, trails, crossings, navigable water, and sea-route segments carry geometry, capacity, seasonality, and permissions.

**Transport services.** Caravans, ferry crossings, barges, ships, relay stations, and later scheduled trains offer actual departure times and finite capacity. Their owners hire workers, buy supplies, maintain assets, and choose whether to operate.

**People and consignments.** Every traveler remains an individual agent. A caravan object can coordinate fifty people and twenty animals without erasing their identities. Cargo remains owned, located inventory throughout the trip.

Route choice should minimize an agent-specific generalized cost:

\[
G = \text{money paid}
+\text{value of time}\times T
+\text{expected damage/loss}
+\text{discomfort or danger preference}
\]

A trader moving stone, a wealthy passenger, a migrating household, and an army should choose differently.

For performance, cache routes by destination region, mode, season, and major restrictions. Recalculate when conditions change materially, not every simulation frame. Handle distant journeys through scheduled arrival and intermediate events; retain continuous visible movement near the camera. Route knowledge belongs to agents and institutions, so the routing system must distinguish **a physically available route from one a traveler knows**.

### 5.2 Technology-node semantics

Each node should distinguish:

```
knowledge prerequisites
material/component prerequisites
skills required to build and operate
facilities required
environmental operating conditions
ownership/access conditions
goods, recipes, buildings and services unlocked
```

Use AND/OR logic. A bridge might require **carpentry OR masonry OR fiber engineering**, depending on its recipe. An ocean voyage may require **expert wayfinding OR an instrument-and-chart navigation package**, with different reliability profiles.

Dates below are **historical evidence or reference ranges, never simulation gates**. “Unknown” means no defensible first-invention date is assigned here. Some rows use a secure later attestation rather than pretending it marks the invention.

### 5.3 Land transport and infrastructure nodes

| Node | Prerequisites | Approximate appearance or attestation | Concrete unlocks |
| --- | --- | --- | --- |
| **L01 Carrying equipment** | Cordage; basketry, leather, or wood | Prehistoric; multiple regions; first unknown | Packs, baskets, carrying poles, load frames; porter recipes |
| **L02 Sleds and drag transport** | Woodworking + lashings | Prehistoric; first unknown | Sled/travois variants; dragging heavy goods on suitable surfaces |
| **L03 Trained pack animals** | Available domestic species + handling + load equipment | Donkey domestication inferred c.5000 BCE, Africa; packing chronology separate | Species-specific pack services, saddles, panniers, animal-training work |
| **L04 Yoked animal traction** | Trained animals + woodworking + harness craft | Prehistoric agrarian Eurasia; precise first use uncertain | Draught teams, yokes, traction for carts and agricultural implements |
| **L05 Wheel-and-axle vehicles** | Accurate woodworking + wheel/axle bearings; human OR animal traction | c.3500–3300 BCE, Europe and Southwest Asia | Solid wheels, axles, grease use, carts, wheelwright work |
| **L06 Four-wheel wagon systems** | L05 + chassis construction + steering solution | Late fourth millennium BCE vehicles include four-wheel forms; designs vary | Higher-capacity wagons, steering assemblies, wagon repair |
| **L07 Light spoked wheels** | Advanced wheelwright joinery + suitable timber | Around 2000 BCE, Eurasian chariot contexts | Light vehicles and chariots; lower wheel mass, not automatic heavy freight |
| **L08 Riding and mounted transport** | Suitable trained animal + rider skill | Horse-based mobility expanded strongly around 2200–2000 BCE, Eurasia | Mounted travel, courier services, riding equipment |
| **L09 Single-animal shaft harness** | L05 + fitted harness + balancing/braking arrangements | Western Han China; distinct Roman systems by first century CE | One-animal carts and carriages; shafts and breeching recipes |
| **L10 Improved collars and traces** | Harness/leather/fiber craft + animal-specific fitting | Medieval Eurasia; European depictions around 800 CE; origin disputed | Better heavy-draught configurations, tandem teams, reduced injury |
| **L11 Wheelbarrows and handcarts** | L05 + balanced small chassis | Han-period China is a conventional early wheelbarrow reference; precise origin remains provisional | Human-powered wheeled freight, construction hauling |
| **I01 Engineered roads** | Surveying/layout + earthwork + drainage; optional stonework | Ancient, multiple centers; no unique first assigned | Drained tracks, surfaced roads, causeways, maintenance services |
| **I02 Bridges and engineered crossings** | Site knowledge + carpentry OR masonry OR fiber engineering | Ancient and prehistoric traditions; Andean network is a later non-wagon example | Beam, arch, suspension, and other crossing recipes with distinct limits |
| **I03 Waystations and relays** | Storage + provisioning organization + dependable personnel | Ancient institutions in several regions; no unique first assigned | Inns, fodder stores, depots, replacement animals, messenger relays |
| **I04 Freight landings and ports** | Water access + handling labor + storage; optional quays | Ancient riverine and maritime societies; first varies by facility | Landings, wharves, warehouses, boat repair, customs and pilot services |

The wheel chronology is based on archaeological comparison rather than a secure single-center invention story. Horse and donkey dates come from genomic research and should not be confused with first surviving transport equipment. [ResearchGate](https://www.researchgate.net/scientific-contributions/Janusz-Kruk-2092714761)

For harness nodes, Brownrigg’s research supports separating single draught, collars, traces, and their different functions. The conventional China-to-Europe collar narrative should remain a contested interpretation, not a compulsory diffusion path. [Academia](https://www.academia.edu/116311848/The_origin_of_the_horse_collar_2022)

### 5.4 Watercraft, waterways, and navigation nodes

| Node | Prerequisites | Approximate appearance or attestation | Concrete unlocks |
| --- | --- | --- | --- |
| **W01 Rafts and bundled craft** | Buoyant timber/reeds + binding | Prehistoric; first unknown | Rafts, reed craft, timber rafting, one-way delivery |
| **W02 Dugout canoes** | Large timber + adze/axe work; controlled burning optional | Eighth–seventh millennia BCE, European surviving examples; not necessarily world origin | Dugouts, paddles, fishing and local cargo boats |
| **W03 Skin/bark framework boats** | Frame construction + sewing + waterproof covering | Prehistoric regional traditions; first unresolved | Lightweight boats, portable watercraft, alternative timber requirements |
| **W04 Planked hull construction** | Plank production + sewing OR joinery/fasteners + sealing | Ancient; secure mortise-and-tenon example at Uluburun, c.1320 BCE, is not the first | Built hulls exceeding single-tree constraints; boatyard recipes |
| **W05 Sailing rigs** | Suitable hull + spars + cordage + woven/mat sail + seamanship | Ancient Egypt/Southwest Asia; late-fourth/third-millennium BCE conventional range | Sails, masts, rigging, wind-powered transport |
| **W06 Outriggers and double hulls** | W02 OR W04 + crossbeam/lashing design | Pre-European Austronesian/Pacific traditions; exact first dates uncertain | Stabilized canoes, multihull variants, different cargo/stability tradeoffs |
| **W07 Managed river haulage** | Boats + river knowledge; towpath/animals where used | Ancient, multiple river systems | Towing, poling, towpaths, hauling crews, directional freight services |
| **W08 Navigation canals** | Earthworks + surveying + water supply + maintenance organization | Ancient; Chinese Grand Canal sections by fifth century BCE | Artificial waterway links, cuts, embankments, canal administration |
| **W09 Chamber/pound locks** | W08 + watertight gates/chamber + controlled water flow | Medieval China; early-eleventh-century improvements documented by Shen Kuo | Elevation changes, reduced overland hauling, lock operators and queues |
| **W10 Improved rudder systems** | Hull-specific steering design + joinery/metalwork as applicable | Early imperial China and later distinct medieval European arrangements; definition-sensitive | Stern-rudder variants; altered steering labor and hull compatibility |
| **W11 Compartmented hulls** | W04 + internal partitions + effective sealing | Medieval Chinese shipbuilding; exact priority needs ship-specific verification | Watertight compartments; reduced consequences of some hull damage |
| **W12 Advanced/mixed rigs** | W05 + experienced sailmakers and sailors | Multiple ancient/medieval traditions; no single ladder | Alternative sail plans and combinations; different wind-performance curves |
| **N01 Local pilotage** | Accumulated environmental knowledge | Prehistoric; worldwide | Known channels, crossings, tidal windows, pilots |
| **N02 Oceanic wayfinding** | N01 + trained observation and route memory | Pre-instrument oceanic traditions; exact origin unknown | Open-water routes, star/swell knowledge, navigator apprenticeship |
| **N03 Sailing directions and charts** | Route knowledge + record-making appropriate to culture | Ancient written sailing directions; multiple later chart traditions | Transferable route information, hazard records, chart production |
| **N04 Magnetic marine compass** | Magnetized needle + float/pivot + interpretation skills | China, documented by Zhu Yu in 1119 | Compass goods; bearings when celestial observations are obscured |
| **N05 Instrumental latitude** | Angle-measuring instruments + astronomy + practical training | Developed through medieval/early-modern astronomical and maritime traditions | Latitude observations, instrument making, astronomical tables |
| **N06 Reliable longitude methods** | Precision timekeeping **OR** astronomical observations plus accurate tables; skilled calculation | Marine timekeeper demonstration: Britain, 1761–1764; alternative astronomical methods coexist | Chronometers, navigation tables, lower positional uncertainty |

The strongest material anchors here are the dated dugouts, Uluburun’s surviving hull construction, and experimental sewn-ship work. These demonstrate alternative construction paths; they do not justify treating all “ancient boats” as one capacity class. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0299765)

For Chinese hydraulic and nautical branches, Needham remains a foundational reference, but individual priority claims require care. Shen Kuo and Zhu Yu provide directly relevant historical accounts of locks and navigation. [Google Books](https://books.google.com/books/about/Science_and_Civilisation_in_China_Volume.html?id=l6TVhvYLaEwC)

Harrison’s successful trials are particularly useful for TCE’s distinction between prototype and deployment: the existence of a workable timekeeper did not instantly equip every ship. [Royal Museums Greenwich](https://www.rmg.co.uk/stories/topics/harrisons-clocks-longitude-problem)

**Recipe example:** a sewn-plank ship should consume planks, extensive cordage, caulking/sealing materials, spars, sails, and specialist labor—not require iron nails. The *Jewel of Muscat* reconstruction used over **120 km of coconut cordage** and more than **37,000 drilled holes**. Because modern tools and construction compromises were involved, its schedule is not a direct ancient labor estimate. [Smithsonian APA Center](https://asia.si.edu/wp-content/uploads/2023/06/shipwrecked-08-vosmer.pdf)

### 5.5 Post-v1 nodes

| Node | Prerequisites | Historical reference | Concrete unlocks |
| --- | --- | --- | --- |
| **P01 Rail-guided haulage** | Wheel/axle craft + track construction + rights of way | Pre-steam mining railways in early-modern Europe | Wagonways, rails, guided wagons; animal haulage remains possible |
| **P02 Steam locomotion** | P01 + boilers/engines + precision metalworking + fuel/water services | Trevithick’s 1804 Penydarren locomotive; commercial networks later | Locomotives, tenders, depots, train services |
| **P03 Marine steam propulsion** | Shipbuilding + boilers/engines + paddle or screw systems | Late-eighteenth-/early-nineteenth-century development | Steam vessels, engine rooms, bunkering and boiler repair |
| **P04 Large iron/steel ships** | Industrial plate production + joining methods + docks + engineering | *Great Britain*, 1843, is an important iron screw-ship benchmark | Metal hull recipes, larger assets, dry-dock requirements |
| **P05 Internal-combustion road transport** | Engines + fuels + transmission + vehicle production and repair | Benz motorcar patent, Germany, 1886; commercial freight develops later | Motor vehicles, fuel depots, garages, road-freight services |
| **P06 Standardized intermodal systems** | Compatible load units + terminals + lifting machinery + institutional coordination | Industrial/modern development; standards are not one isolated invention | Container-like units, rapid transfer, terminal specialization |

The industrial examples are milestones, not claims that one inventor supplied every prerequisite. Museum and manufacturer records anchor the 1804, 1843, and 1886 examples. [Museum Wales](https://museum.wales/collections/online/object/00200027-ecc6-31a5-98b5-50dde6d8a885/Trevithicks-Penydarren-locomotive-model/?field0=string&field1=with_images&page=99&value0=back&value1=1)

### 5.6 What to simplify—and what not to simplify

For v1, simplify sailing to wind-performance tables, rivers to directed seasonal edges, and vehicle motion to load/slope/surface functions. Use discrete handling jobs rather than simulating every lifted sack.

Do **not** simplify away cargo location, empty returns, animal feed, seasonal closures, terminal capacity, or ownership. Those mechanisms create much of the historical economic geography.

The smallest useful initial set is porters, locally available pack animals, an ox-cart family, dugouts, barges, and sailing coasters, supported by tracks, crossings, landings, storage, provisioning, and repair. Advanced navigation and canals can then expand this system without replacing its accounting model.

### 5.7 Existing models and games worth studying

| Model/game | Relevant pattern | What TCE should borrow or avoid |
| --- | --- | --- |
| **ORBIS** | Multimodal historical routing with different travel costs and seasonal conditions | Borrow directed, mode-dependent networks. Treat its reconstructions as scenario assumptions, not timeless constants. [Journal of Digital Humanities](https://journalofdigitalhumanities.org/1-3/modeling-networks-and-scholarship-with-orbis-by-elijah-meeks-and-karl-grossner/) |
| **MATSim** | Large numbers of individual travelers and dynamic daily transport simulation | Borrow separation of traveler demand from network movement and scalable agent-based transport architecture. TCE needs additional freight, animal, and institutional systems. [MATSim](https://matsim.org/) |
| **OpenTTD** | Explicit transport assets and networks moving passengers and cargo across several modes | Useful for readable routes, transfers, and asset utilization. Its game economy should not be used as historical calibration. [GitHub](https://github.com/openttd/openttd) |

---

## 6. Sources, datasets, and remaining uncertainty

### Priority research and calibration sources

| Source | Best use | Main limitation |
| --- | --- | --- |
| **Alvarez-Palau, Bogart et al., *Multi-modal models for pre-steam English and Welsh freight transportation systems*** | Network construction, road gradients, freight rates, port/transshipment charges | Reconstructed regional benchmarks rather than universal prices. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/Multimodal%20model%201680%20and%201830%20EJ%20replication.pdf) |
| **Bogart, *The Transport Revolution in Industrializing Britain*** | Wagon capacity, stagecoach speeds, freight prices, seasonal road improvements | Britain-centered compilation; observations differ in coverage. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/transport_revolution_cehmbjuly92013_forweb.pdf) |
| **Flückiger et al., *Roman Transport Network Connectivity and Economic Integration*** | Ancient effective transport distance and integration | Tariff evidence and modeled costs must be distinguished from actual transactions. [ifo Institut](https://www.ifo.de/DocDL/cesifo1_wp7740.pdf) |
| **Donaldson, *Railroads of the Raj* (2018)** | Testing trade costs, price convergence, and income effects; replication materials | A particular historical and institutional setting, not a universal railway bonus. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20101199) |
| **Sieber, *Transporting the Yield*** | Small-scale freight, pack animals, carts, and utilization-sensitive comparisons | Modern rural analogues; scenario assumptions matter. [Niklas Sieber](https://www.niklas-sieber.de/Publications/Transporting_Yield.pdf) |
| **FAO, Bactrian camel husbandry account** | Species-specific loads, daily travel, feeding, and ecological requirements | Not representative of every camel population or historical caravan. [FAOHome](https://www.fao.org/4/x1700t/x1700t05.htm) |
| **Gibaja et al. (2024), La Marmotta boats** | Early hull construction, dated evidence, woodworking prerequisites | Archaeological dimensions do not directly supply commercial payloads. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0299765) |
| **Vosmer, *The Jewel of Muscat*** | Construction recipes, material dependencies, sailing experiments | Reconstruction choices and modern interventions. [Smithsonian APA Center](https://asia.si.edu/wp-content/uploads/2023/06/shipwrecked-08-vosmer.pdf) |
| **Brownrigg (2022), *The Origin of the Horse Collar*** | Harness mechanics, competing origin claims, critique of older reconstructions | Iconography and terminology leave some chronological ambiguity. [Academia](https://www.academia.edu/116311848/The_origin_of_the_horse_collar_2022) |
| **Polynesian Voyaging Society practitioner documentation** | Non-instrument navigation, expertise, teaching, and route knowledge | Modern documentation and revival practice are not direct measurements of every prehistoric tradition. [Hokule'a Archive](https://archive.hokulea.com/navigate/navigate.html) |

For downloadable historical geography, Bogart’s research page identifies datasets for roads, turnpikes, stagecoaches, ports, coastal sailing, and navigation lights. These are especially valuable for testing whether TCE’s network algorithms produce plausible access patterns. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/test_researchpage/)

For post-v1 freight prices, the US Bureau of Transportation Statistics publishes modal revenue-per-ton-mile series. Its documentation warns of differences in coverage and definitions; notably, the truck series largely represents **less-than-truckload** service, so it should not be treated as an interchangeable truckload benchmark. [Bureau of Transportation Statistics](https://www.bts.gov/content/average-freight-revenue-ton-mile)

### Where the evidence is thin or contested

**First invention dates are often weaker than surviving-object dates.** Basic rafts, carrying equipment, pilotage, and many organic hull traditions have poor preservation. Several graph dates above are explicitly reference ranges or secure attestations rather than independently established first inventions.

**Ancient payloads and operating costs are frequently reconstructed.** Displacement, deadweight, cargo capacity, and volumetric tonnage must not be mixed. Large famous ships are not representative of ordinary local freight.

**Harness and rig histories are not simple progress ladders.** Avoid a universal collar multiplier or the assumption that one sail shape replaced all others. Vessel, rig, wind regime, cargo, and seamanship work together.

**Risk needs scenario-specific calibration.** The evidence assembled here does not justify one universal annual shipwreck probability, caravan theft rate, or bridge-failure rate. Start with explicit low/medium/high hazard scenarios and test sensitivity rather than presenting invented percentages as historical measurements.

**The largest remaining calibration gaps are net payloads for particular canoe and barge designs, preindustrial construction and maintenance labor, and route-specific losses.** Archaeological hull studies, experimental voyages, and local freight accounts are the appropriate next evidence—not a single global transport-cost table.

**For v1, the decisive success criterion is this:** people and institutions should discover that particular combinations of animals, craft, routes, services, and knowledge make some movements profitable and others impossible. Ports, depots, market towns, caravan routes, and regional specialization should then emerge from those decisions—not from an era label or a scripted trade bonus.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9285e-0ab8-83ea-8f14-523f7ad55225)
