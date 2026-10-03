# Urban fires and firefighting: a simulation-ready report for TCE

## Executive finding

**TCE should model frequent opportunities for small fires, followed by several distinct opportunities for containment—not periodically select a city to suffer a “great fire.”** Ignition, household intervention, building involvement, interbuilding spread, and organized suppression need separate mechanisms.

Historical evidence does not support one transferable annual fire probability for a “medieval house.” Cities with superficially similar hazards had substantially different experiences. Garrioch’s research, for example, finds that early-modern Paris avoided the repeated conflagrations experienced elsewhere partly through local conditions and effective intervention by inhabitants, before a permanent brigade became the decisive institution. His broader European research rejects the idea of an essentially unchanged fire regime lasting until modern construction. [Monash University](https://research.monash.edu/en/publications/why-didnt-paris-burn-in-the-seventeenth-and-eighteenth-centuries/)

The strongest quantitative evidence comes from different sources for different questions: historical catalogs for large-fire frequency, household surveys for otherwise invisible small fires, experiments for spread, and modern incident records for response. **These are complementary—not interchangeable—calibration targets.**

---

## 1. Mechanisms: rules a simulation can implement

### 1.1 Generate ignitions from activities, not from population alone

Give each relevant activity an exposure-dependent chance of starting an **unintended fire outside its intended containment**. A working hearth is not already a building fire.

A suitable proposed rule is:

\[
P(\text{ignition during step})
=1-\exp\left[-\sum\_a \lambda\_a\,\Delta H\_a\,m\_a\right]
\]

Here, \(\lambda\_a\) is the baseline hazard per activity-hour, \(\Delta H\_a\) is the exposure accumulated during the step, and \(m\_a\) combines equipment condition, fuel placement, supervision, maintenance, and environmental conditions.

Represent cooking, heating, lighting, baking, brewing, smithing, kiln work, hot-ash disposal, and combustible storage separately. Later technologies introduce electrical, machinery, and liquid-fuel hazards. Historical research emphasizes that changing industry, consumption, and lighting altered fire exposure: economic development was not simply a continuous reduction in risk. [Monash University](https://research.monash.edu/en/publications/playing-with-fire-love-of-light-and-nocturnal-shadows/)

**Recommended causal details:**

* An occupied cooking fire produces ignition exposure but also places someone nearby to notice and suppress it.
* A chimney or enclosed stove changes several hazards: exposed flame, sparks, accumulated deposits, surrounding construction, and maintenance requirements.
* Crowding increases the number of ignition activities and potential witnesses simultaneously.
* Deliberate burning should originate in agent decisions. Earthquake-related ignitions should be generated jointly with damaged structures, obstructed streets, and disrupted water—not as unrelated random events.

Post-earthquake urban-fire models explicitly incorporate damage to building components, supporting this coupled treatment rather than independent disaster rolls. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1193/1.4000154)

### 1.2 Separate ignition from escape and major loss

Use an incident progression such as:

**Unintended ignition → locally controlled or growing → compartment/building involvement → exposure of neighbors → neighborhood fire.**

Maintain different counters for:

| Counter | Meaning |
| --- | --- |
| Unintended ignitions | All fires outside intended containment, including those immediately extinguished |
| Escaped fires | Fires beyond available household control |
| Attended incidents | Fires to which an organized service responds |
| Building losses | Buildings substantially damaged or destroyed |
| Conflagrations | Connected incidents involving many buildings |

This distinction is empirically important. The English Housing Survey found that only **25% of reported household fire experiences were extinguished by the fire and rescue service**. That is an extinguishment share, **not** the fraction reported to firefighters; some attended fires may already have been put out. [GOV.UK](https://assets.publishing.service.gov.uk/media/5b45c43ae5274a3764524d8f/Fire_and_Fire_Safety.pdf)

For TCE, create both a complete kernel event log and an incomplete social record. The simulation may know that hot ash ignited a storehouse; residents may only know that the building was found burning.

### 1.3 Model building assemblies, not a single “flammability” number

At minimum, distinguish:

| Component | Relevant simulation properties |
| --- | --- |
| Roof covering | Ignitability, combustible mass, gaps, exposed edges, ember retention |
| Frame and floors | Combustible structural mass, protected versus exposed members |
| Walls and linings | Heat transmission, combustible lining, integrity, openings |
| Contents | Fuel quantity, arrangement, ignition sensitivity, burning intensity |
| Connections | Shared roof spaces, party-wall breaches, attached sheds, balconies |
| Condition | Moisture, maintenance, deterioration, previous damage |

A masonry shell can contain a severe contents-and-roof fire. Conversely, a combustible structure is not certain to ignite from every nearby flame. Experiments in informal settlements also demonstrate that thin metal cladding does not make a dwelling fireproof when its contents and linings burn and its enclosure deforms. [SFPE](https://www.sfpe.org/publications/periodicals/sfpeeuropedigital/sfpeeurope18/issue18feature1)

Represent fire intensity, remaining fuel, structural damage, and smoke separately. Extinguishment stops further burning; it does not restore consumed beams or damaged roofing.

### 1.4 Keep three interbuilding spread routes

A reduced urban model should support:

**Connected-fuel spread:** flames move through attached combustible structures or shared voids.

**External heat exposure:** radiation and wind-driven flames/plumes expose nearby surfaces and openings.

**Firebrand spread:** burning fragments travel downwind and may ignite receptive roofs or accumulated debris.

Himoto and Tanaka’s physics-based urban model is a useful precedent for separating building fire behavior from external radiation, wind-driven exposure, and firebrand transport. [KURENAI](https://repository.kulib.kyoto-u.ac.jp/items/4c53ee14-4de8-4bdb-a67f-585e545a8f9e)

For a receiving component \(j\), a computationally inexpensive proposed formulation is:

\[
P\_j(\text{ignition})=1-\exp[-h\_j(t)\Delta t]
\]

The hazard \(h\_j\) should depend on **accumulated exposure**, not distance alone. Store a small thermal or exposure state that rises under heating and falls with cooling or wetting. Otherwise, many individually insufficient exposures never combine, or a single momentary exposure causes unrealistically immediate ignition.

Do not make all three routes equally important everywhere. Researchers reported no obvious firebrand contribution in some small informal-dwelling experiments; the dominant route depends on the assembly and incident. [SFPE](https://www.sfpe.org/publications/periodicals/sfpeeuropedigital/sfpeeurope18/issue18feature1)

### 1.5 Weather changes several systems simultaneously

Wind should make exposure directional, alter flames and firebrand transport, and influence which side of a fire is defensible. Fuel moisture should respond to recent weather rather than flip instantly between “wet” and “dry.”

Seasonality must emerge from local combinations. In Edo, dry air, strong winter and early-spring winds, and domestic heating combined to create hazardous conditions. A universal summer fire season would therefore be wrong. [Cambridge University Press](https://www.cambridge.org/core/journals/itinerario/article/abs/fires-and-recoveries-witnessed-by-the-dutch-in-edo-and-nagasaki-the-great-fire-of-meireki-in-1657-and-the-great-fire-of-kanbun-in-1663/159DF4BF9A24DA094D6850ECB415277E)

For simulation purposes, couple drought to both combustible dryness and available firefighting water. This creates a plausible compound failure: fires become harder to stop precisely when wells, streams, or cistern replenishment are least adequate.

### 1.6 Firebreaks reduce exposure; they do not create immunity

A road, courtyard, canal, cleared strip, or demolished building removes some fuel connections. Its effectiveness still depends on width, adjacent building height, openings, wind, and firebrand exposure.

Experimental separation distances can help calibrate local heat transfer, but **there is no universally safe “three-meter firebreak.”** Laboratory-derived separation thresholds change with the materials and exposure configuration being tested. [Springer](https://link.springer.com/article/10.1007/s10694-020-01075-w)

Treat emergency demolition as an agent task requiring tools, access, time, coordination, and permission. Its economic cost occurs even when successful. This is particularly relevant to Edo’s historically important demolition-based firefighting. [Kaigai Shobo](https://www.kaigai-shobo.jp/files/fireserviceinjapan_eng/20221001_History-Japan%28T-Y%29_eng.pdf)

### 1.7 Firefighting is a supply chain

A brigade’s effective water application is bounded by its weakest stage:

\[
q\_{\text{applied}}
=\eta\,
\min(q\_{\text{source}},q\_{\text{delivery}},q\_{\text{pump}})
\]

Here, \(\eta\) represents losses and the fraction reaching useful targets. Pump output should depend on operating crew, fatigue, equipment condition, and delivery head.

For individual bucket carriers:

\[
q\_{\text{delivery}}
=\frac{N\,V\_{\text{bucket}}}{T\_{\text{round trip}}}
\]

For a stationary relay, use the throughput of its slowest filling, passing, or emptying stage instead.

**Illustrative calculations—not historical measurements:** twenty carriers moving eight liters each on a two-minute circuit deliver **80 L/min before losses**. A **5 m³** cistern supplying **400 L/min** without replenishment lasts only **12.5 minutes**.

These relationships make wells, access lanes, storage reservoirs, spare buckets, and replacement pumping crews meaningful. Buying a better pump alone need not improve performance.

### 1.8 Model the complete response clock

\[
t\_{\text{effective intervention}}
=t\_{\text{detection}}
+t\_{\text{alarm}}
+t\_{\text{assembly}}
+t\_{\text{travel}}
+t\_{\text{setup}}
\]

Several responses can proceed in parallel: a resident throws water while a neighbor raises the alarm and a watchman summons additional help.

Modern English statistics measure the interval from the emergency call to the first appliance’s arrival. They exclude the preceding discovery delay and do not measure when useful water first reaches the fire. [GOV.UK](https://www.gov.uk/government/statistics/detailed-analysis-of-fires-england-year-ending-march-2026/detailed-analysis-of-fires-and-response-times-to-fires-attended-by-fire-and-rescue-services-england-year-ending-march-2026)

Therefore, a station’s nominal coverage radius should never substitute for actual notification, staffing, route access, and setup.

---

## 2. Quantitative parameters and calibration benchmarks

**Confidence convention:** **H** means strong evidence for the stated population or experiment; **M** means a credible historical reconstruction, survey, or limited experiment; **L** means thin evidence or an explicitly proposed simulation prior. High confidence in an observation does not imply high transferability to other settings.

### 2.1 How often did fires occur?

| Setting and measure | Observed value | Appropriate use and limitations | Confidence |
| --- | --- | --- | --- |
| Edo, 1601–1867: cataloged fires | **1,798**, equivalent to **6.7 recorded fires/year** | City-level historical record, not a census of small ignitions | M |
| Edo, same period: large fires | **49**, equivalent to **0.184/year**, or one per **5.4 years** on average | A long-period mean, not a regular recurrence clock | M |
| Osaka and Kyoto, same comparison | **6** and **9** large fires respectively | Demonstrates substantial regional variation; populations and exposure are not standardized | M |
| Nagasaki, 1633–1868, including suburbs | **232**, approximately **1 recorded fire/year** | Boundary and reporting differences prevent direct household-risk comparison | M |

These Japanese figures come from *Fires and Recoveries Witnessed by the Dutch in Edo and Nagasaki*. Annualized values above are calculations from the published counts. **Do not use 49/1,798 as an unbiased probability that an ignition becomes a large fire:** small incidents are much less likely to appear in historical records. [Cambridge University Press](https://www.cambridge.org/core/journals/itinerario/article/abs/fires-and-recoveries-witnessed-by-the-dutch-in-edo-and-nagasaki-the-great-fire-of-meireki-in-1657-and-the-great-fire-of-kanbun-in-1663/159DF4BF9A24DA094D6850ECB415277E)

| Modern benchmark | Observed value | Appropriate use and limitations | Confidence |
| --- | --- | --- | --- |
| English Housing Survey, 2016–17 report: households experiencing a fire in the previous year | Approximately **1%**, or **10 affected households per 1,000 household-years** | Self-reported experience, including small fires; affected households are not identical to incident counts | M |
| Same survey: fires extinguished by fire and rescue services | **25%** | Evidence that household action and self-extinguishment matter; not brigade attendance rate | M |
| United States, 2023 residential-building fires | **344,600 estimated fires** | A national attended-fire benchmark, not all unintended ignitions | H |
| US residential causes, 2023 | Cooking **167,800**; heating **27,900**; electrical malfunction **23,700** | Approximately **48.7%, 8.1%, 6.9%** of the total; do not backcast these shares into preindustrial societies | H |

Sources: English Housing Survey; US Fire Administration residential-fire estimates and cause tables. [GOV.UK](https://assets.publishing.service.gov.uk/media/5b45c43ae5274a3764524d8f/Fire_and_Fire_Safety.pdf)

**Missing parameter:** the reviewed evidence does not establish a defensible universal annual ignition rate per early-farming or preindustrial household. That must remain a calibration uncertainty rather than a fabricated historical constant.

### 2.2 Spread and suppression benchmarks

| Quantity | Evidence-based value | Interpretation for TCE | Confidence |
| --- | --- | --- | --- |
| Full-scale mock informal settlement | **20 dwellings**, **1–2 m** separation, wind **15–25 km/h** or **4.2–6.9 m/s** | A high-connectivity, wind-driven test case | H for experiment |
| Spread in that experiment | Four dwellings were ignited simultaneously; fire involved the entire settlement within approximately **5 minutes** | **Not** five minutes from one accidental spark to twenty houses | H |
| Local development in the same test | Temperatures exceeded **1,000°C within about one minute** of a dwelling igniting; downwind neighbors could ignite within another minute | Suitable as a rapid-development stress test, not a generic house curve | H |
| Separation study: material-dependent thresholds | Approximately **2.14 m** for structural timber/cardboard and **3.14 m** across the tested material set | Specific experimental/model conditions; not universal planning setbacks | M |
| Separation study: incident radiation | Approximately **36 kW/m² at 1 m**, falling to **5 kW/m² at 4 m** in the studied configuration | Useful for checking distance attenuation, not a universal inverse-distance formula | M |
| Preliminary informal-dwelling fuel-load measurements | Approximately **400–500 MJ/m²** in the cited sample | Initial contents-fuel benchmark; not appropriate for every historic dwelling | M |
| Newsham manual fire engine, eighteenth century | **4–12 pump operators**; reported maximum **160 gallons/min** | Roughly **600–750 L/min**, depending on gallon interpretation; sustained field throughput uncertain | M description; L transfer |
| Controlled modern residential fireground experiment | Four-person crews completed 22 essential tasks **30% faster than two-person crews**, **25% faster than three-person crews** | Calibrate task parallelism, not a universal extinguishment-probability bonus | H for test |

Sources: de Koker et al.; Wang et al.; the researchers’ SFPE synthesis; Society of Antiquaries equipment documentation; NIST residential fireground experiments. [Edinburgh Research](https://www.research.ed.ac.uk/en/publications/20-dwelling-large-scale-experiment-of-fire-spread-in-informal-set/)

A manual pump’s quoted performance excludes the additional people needed to bring water, move equipment, manage access, and perform other tasks. In TCE, pump operators and water carriers must come from the same finite labor pool.

### 2.3 Response and containment

| England, year ending March 2026 | Value |
| --- | --- |
| Mean call-to-first-arrival time, primary fires | **9 min 25 sec** |
| Mean for dwelling fires | **8 min 15 sec** |
| Primary fires in predominantly urban fire-authority areas | **8 min 21 sec** |
| Primary fires in predominantly rural fire-authority areas | **12 min 09 sec** |
| Dwelling-fire responses within ten minutes | **78%** |
| Dwelling fires classified as having no fire damage / item-only / room-only / larger damage | **29% / 34% / 22% / 14%**, rounded |

These are modern reference distributions, not targets for ancient brigades. Area classifications are at fire-authority level. Containment reflects the entire system—occupants, construction, detection, fixed protection, and responders—not firefighters’ isolated causal effect. [GOV.UK](https://www.gov.uk/government/statistics/detailed-analysis-of-fires-england-year-ending-march-2026/detailed-analysis-of-fires-and-response-times-to-fires-attended-by-fire-and-rescue-services-england-year-ending-march-2026)

### 2.4 Great-fire footprint benchmarks

| Event | Burned area | Other scale indicators | Calibration lesson |
| --- | --- | --- | --- |
| London, 1666 | **436 acres ≈176 ha ≈1.76 km²** | Approximately **13,200 houses**; four-fifths of the **City of London**, not modern Greater London | Connected urban fuel and constrained streets can produce enormous cumulative loss |
| Chicago, 1871 | **2,124 acres ≈860 ha ≈8.60 km²** | Approximately **17,500 buildings**, over roughly **36 hours** | Industrial-era institutions do not eliminate extreme wind/drought conflagrations |
| Bukit Ho Swee, Singapore, 1961 | Approximately **40 ha =0.40 km²** | Approximately **16,000 homeless**, **4 deaths** | Dense timber-and-zinc development, difficult access, and water limitations can defeat an organized service |

Sources: London Museum’s Great Fire project; US National Weather Service; Singapore’s National Heritage Board. Unit conversions are calculated. [The Great Fire of London](https://www.fireoflondon.org.uk/story/streets-and-buildings/)

Do not divide these areas by duration to obtain a universal spread speed. A fire develops an irregular perimeter, crosses obstacles, starts secondary fronts, and spends different lengths of time consuming individual buildings.

### 2.5 Explicit starting priors where evidence is insufficient

The following are **proposed sensitivity-test settings, not measured historical ranges**:

| Parameter | Initial exploration range | Implementation |
| --- | --- | --- |
| Unintended household-fire frequency | **1–100 per 1,000 household-years** | Explore logarithmically; use **10** only as an order-of-magnitude reference anchored to the modern household survey |
| Detection, occupied and awake | **0–3 min**, with a concealed-fire tail | Derive from perception, room occupancy, and fire location |
| Detection, sleeping or poorly observed | **5–30+ min** | No hard maximum; some fires remain unnoticed until external involvement |
| Volunteer assembly after alarm | **2–15 min** | Prefer actual agent travel and equipment collection over a fixed delay |
| Carried bucket water | **5–10 L** | Vessel capacity, spillage, fatigue, and terrain |
| Equipment setup | **1–10 min** | Explicit positioning, filling, hose or nozzle preparation |
| Active-incident numerical step | **1–5 seconds** | An engineering starting point; verify convergence |

All have **L confidence as historical parameters**. They are useful because they expose uncertainty rather than hide it.

For scale, a hypothetical settlement of **5,000 households** at **10 unintended fires per 1,000 household-years** generates about **50 small incidents annually**. How many become building losses should emerge from detection and suppression. How many spread further should emerge from geometry, materials, and weather.

---

## 3. Variation across societies, regions, and firefighting institutions

### 3.1 Do not implement eras as fire-risk multipliers

| Context | Appropriate interpretation |
| --- | --- |
| **Foragers** | Usually a camp-fire problem rather than an urban-fire problem. For TCE, fuel arrangement, aggregation size, vegetation, shelter spacing, and mobility are more meaningful than an “era” coefficient. This is a mechanistic inference; comparable annual camp-fire statistics were not established here. |
| **Early farming** | Permanent buildings and stored resources create persistent exposure. Archaeological burned settlements document destruction but do not automatically identify accidental ignition: deliberate burning and other causes remain important interpretive problems. |
| **Preindustrial cities** | Householder intervention, watches, bells, water stores, manual engines, demolition, military personnel, and specialized brigades could coexist in different combinations. There was no single progression shared by all cities. |
| **Industrial cities** | Equipment and organization could improve while urban scale and combustible commercial inventories increased. Chicago demonstrates that “industrial” does not mean conflagration-proof. |
| **Modern settlements** | Compartmentation, alarms, automatic suppression, motorized response, and reliable water are separate capabilities. Unequal access to them allows very different fire regimes within the same country or city. |

The archaeological caution is illustrated by research at Neolithic Mursalevo; the historical variation by Garrioch; and the persistence of rapid settlement-scale spread by contemporary full-scale experiments. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/10.1002/2017JB015190)

### 3.2 Regional cases that matter for TCE

**Roman Anatolia: technical capacity could be constrained by politics.** After a destructive fire at Nicomedia, Pliny proposed a brigade of **150 men**. Trajan rejected the association because of concern about political factions, favoring equipment provision and mobilization of householders and the public. A polity can therefore know how to organize firefighting yet refuse a permanent association for political reasons. [Roman Letters](https://romanletters.org/letters/pliny_younger/10033/)

**Japan: prevention and demolition were integral to firefighting.** Edo’s arrangements included distinct samurai and townspeople’s organizations, with demolition and roof-level work important to containment. These institutions were not simply European pump brigades with different uniforms. Their capabilities fit the building stock, labor organization, and available means of creating breaks in combustible development. [Kaigai Shobo](https://www.kaigai-shobo.jp/files/fireserviceinjapan_eng/20221001_History-Japan%28T-Y%29_eng.pdf)

**Ottoman Istanbul: firefighting could be embedded in military organization.** The Istanbul fire service’s historical account identifies a Janissary-associated pump organization during **1714–1826**. TCE should allow brigades to belong to military, neighborhood, occupational, religious, private, or municipal institutions rather than assume a modern department from the outset. [İtfaiye Müdürlüğü](https://itfaiye.ibb.gov.tr/en/fires-in-istanbul.html)

**Paris: inhabitants themselves were consequential.** Garrioch’s comparison identifies local conditions and residents’ firefighting as important in explaining why the city did not repeatedly burn on the scale of some counterparts. This argues against making ordinary people nearly useless until a specialist service is founded. [Monash University](https://research.monash.edu/en/publications/why-didnt-paris-burn-in-the-seventeenth-and-eighteenth-centuries/)

**Singapore and South Africa: modern date does not imply modern protection.** Bukit Ho Swee’s fire occurred in 1961, while the South African experiments were conducted recently. Their relevance lies in assembly, density, access, and supply—not in treating contemporary communities as substitutes for historical cultures. Modern synthetic contents also limit direct transfer of their burning behavior to agrarian housing. [Roots](https://www.roots.gov.sg/stories-landing/stories/bukit-ho-swee-fire/story)

### 3.3 Insurance brigades: avoid the familiar caricature

Do **not** encode “uninsured building = firefighters refuse to extinguish it” as the default historical rule.

Published evidence reviewed by Sillitoe includes instructions to attend fires beyond a company’s own insured properties, cooperation between brigades, and variation in requirements to guarantee expenses. London’s insurance-supported joint brigade dates to **1833**, followed by the publicly funded Metropolitan Fire Brigade in **1866**. The review is a synthesis of published evidence, not a newly assembled archival dataset. [Tom Scott](https://www.tomscott.com/corrections/firemarks/)

The better TCE variables are contractual obligations, reciprocity, reimbursement, rescue duties, neighborhood exposure, and command priorities. A company may rationally fight an uninsured fire because it threatens insured neighbors.

### 3.4 Make improvements separable and reversible

A proposed capability graph should distinguish:

**Household vessels and routines → shared equipment → watches and alarms → trained crews → manual pumping → improved distribution → mechanized pumping → dispatch and automatic protection.**

This is **not** a mandatory technological sequence. Inspection, demolition authority, collective financing, communication, and reliable water can develop independently.

Budgets, maintenance, training, and trust should decay when neglected. A purchased engine without an operating crew is equipment, not protection.

---

## 4. Stylized facts a correct simulation should reproduce

| Pattern | Validation implication |
| --- | --- |
| **A substantial small-fire population exists beneath official incident totals.** Household surveys capture many fires not extinguished by organized services. | The kernel should generate numerous locally controlled incidents without forcing every one into a dramatic citywide alarm. [GOV.UK](https://assets.publishing.service.gov.uk/media/5b45c43ae5274a3764524d8f/Fire_and_Fire_Safety.pdf) |
| **Similar technological periods produce different city-level fire regimes.** Edo’s catalog contrasts sharply with Osaka and Kyoto; Paris supplies another important counterexample. | Climate, construction, urban connections, and local institutions must outweigh a single era modifier. [Cambridge University Press](https://www.cambridge.org/core/journals/itinerario/article/abs/fires-and-recoveries-witnessed-by-the-dutch-in-edo-and-nagasaki-the-great-fire-of-meireki-in-1657-and-the-great-fire-of-kanbun-in-1663/159DF4BF9A24DA094D6850ECB415277E) |
| **Spread can outrun organized response.** The twenty-dwelling experiment progressed from four initial burning dwellings to full involvement in minutes. | Early local intervention and prevention must sometimes matter more than an additional distant engine. [Edinburgh Research](https://www.research.ed.ac.uk/en/publications/20-dwelling-large-scale-experiment-of-fire-spread-in-informal-set/) |
| **Additional personnel have task-dependent value.** NIST’s controlled results show substantial crew-size effects. | Extra people should help when they enable simultaneous or adequately staffed tasks, not provide unlimited additive suppression. [NIST](https://www.nist.gov/news-events/news/2010/09/nist-residential-fire-study-education-kit-now-available) |
| **Water and access can undermine an existing service.** Bukit Ho Swee illustrates these operational constraints. | A nearby brigade should sometimes fail because it cannot deliver enough water to the relevant fronts. [Roots](https://www.roots.gov.sg/stories-landing/stories/bukit-ho-swee-fire/story) |
| **Burned area is not a fixed proxy for deaths.** Edo/Tokyo historical research found that the relationship varied, including no clear area–fatality correlation in its post-Meiji analysis. | Model occupancy, warning, evacuation, refuge access, smoke, and collapse separately from property loss. [J-STAGE](https://www.jstage.jst.go.jp/article/aije/69/575/69_KJ00004079073/_article/-char/en) |

A useful **diagnostic**, rather than an extra game rule, is:

\[
R\_{\text{fire}}
=\mathbb{E}[\text{new buildings ignited by one burning building before containment or burnout}]
\]

When local conditions keep this below one, incidents tend to die out. When it exceeds one across a connected district, growth can outrun finite crews. Weather, suppression, and gaps alter it during the incident. This provides an emergent transition into conflagration without a scripted catastrophe trigger.

---

## 5. Recommended implementation for 10k–50k agents

### 5.1 Use a sparse building-exposure graph

Represent buildings—or compartments within important large buildings—as nodes. Edges describe attached fuel connections and potential external exposure.

Precompute geometry that changes infrequently: separation, facing surfaces, openings, roof relationships, and shielding. Rebuild only affected neighborhoods when construction, demolition, or collapse changes them. Use a spatial index for downwind firebrand targets rather than checking every building against every other building.

For ordinary structures, one interior zone plus separately vulnerable roof and exterior components is a reasonable initial abstraction. Reserve multiple compartments for warehouses, workshops, apartment blocks, palaces, and other consequential structures.

### 5.2 Maintain compact, causally distinct state

| Entity | Minimum useful state |
| --- | --- |
| Building | Fuel stores, assembly types, moisture, openings, exposure state, burning stage, integrity |
| Incident | Known and actual origin, active fronts, weather, committed resources, alarm state |
| Water source | Stored volume, recharge, draw limit, accessibility, contamination or damage state where relevant |
| Equipment | Capacity, crew requirement, condition, mobility, setup tasks |
| Agent | Knowledge, location, role, carried load, fatigue, willingness, perceived danger |
| Institution | Roster, financing, jurisdiction, training, priorities, contracts, demolition authority |

Avoid a single building “fire health” value. Water should reduce burning or exposure; damage should follow consumed or weakened components.

### 5.3 Make agents perform the work

Residents and responders should choose among notifying others, rescuing dependents, suppressing a small fire, carrying water, pumping, moving equipment, protecting exposures, demolishing, salvaging property, and evacuating.

A proposed action score can combine expected benefit, personal danger, duty, kinship, property interest, equipment availability, and social pressure. No individual needs omniscient knowledge of the whole fire.

Use short-lived job reservations and reassignment. A distant agent who has claimed a bucket task should not prevent a nearby available person from doing it. Crew tasks should begin only when their genuinely necessary roles are staffed.

### 5.4 Preserve conservation and bottlenecks

Enforce these invariants:

* Water cannot be applied before it is obtained and delivered.
* One agent cannot simultaneously carry water, pump, and rescue.
* Fuel consumed cannot burn again.
* Cooling does not reverse structural destruction.
* Failed routes change travel and supply, not merely visual animation.

A reduced suppression model can use experimentally calibrated growth-and-cooling curves instead of computational fluid dynamics. Preserve the relationship between available fuel, ventilation, exposure, and useful water even when the internal calculation is simplified.

### 5.5 Keep inactive settlements cheap

Accumulate ignition exposure through the existing activity system. Activate detailed fire stepping only around actual incidents and threatened buildings. Use finer time resolution during rapid growth and coarser updates during slow smoldering.

Rust should own authoritative fire state, agent jobs, water accounting, and deterministic random streams. Unreal should render those outcomes, not determine whether off-screen buildings burn. Changing camera position or simulation speed must not change the physical result.

### 5.6 Learn from existing models

| Model | What to borrow | What not to assume |
| --- | --- | --- |
| **Himoto & Tanaka, 2008, physics-based urban fire spread** | Building-scale fire behavior coupled through radiation, wind-driven exposure, and firebrands | Its validation does not establish universal parameters for every architectural tradition. [KURENAI](https://repository.kulib.kyoto-u.ac.jp/items/4c53ee14-4de8-4bdb-a67f-585e545a8f9e) |
| **Li & Davidson, 2013, urban fire simulation with suppression** | Joint treatment of ignition, spread, fire-department resources, and water constraints | A scenario’s ignition count or wind setting is not a historical constant. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0379711213001604) |
| **RoboCup Rescue Agent Simulation** | Limited perception, communication, road access, task coordination, water resupply, and multi-agent response | Competition abstractions—including protected facility types and simplified ignition—should not become TCE physics. [Robo Rescue](https://roborescue.github.io/rcrs-server/rcrs-server/index.html) |

These provide more defensible implementation precedents than copying a commercial game’s hidden fire-risk percentage.

### 5.7 Calibrate separate stages, then test the whole system

First calibrate ignition and reporting. Next calibrate single-building development and pairwise spread. Then calibrate task times and water delivery. Only afterward tune settlement-scale loss distributions.

Useful controlled tests include identical settlements with one changed variable: roofing, spacing, wind, watch coverage, source reliability, crew availability, or demolition permissions. Compare **incident counts, escaped-fire fractions, buildings lost, response distributions, water use, and labor-hours**, not just total annual damage.

Hold some historical events and experimental configurations out of fitting. Matching one famous fire by adjusting several unconstrained parameters is not validation.

---

## 6. Source priorities, datasets, and unresolved evidence

### Core research and data stack

| Source | Best use |
| --- | --- |
| **Garrioch, 2019, “Towards a fire history of European cities,” Urban History**; DOI **10.1017/S0963926818000275** | Historical variation and the changing relationship between urban life and fire. [Cambridge University Press](https://www.cambridge.org/core/journals/urban-history/article/abs/towards-a-fire-history-of-european-cities-late-middle-ages-to-late-nineteenth-century/DCF986C2797AAFB0A9B6F07DE549E501) |
| **Garrioch, 2019, “Why Didn’t Paris Burn…?”, French Historical Studies**; DOI **10.1215/00161071-7205197** | Counterexample to technologically deterministic accounts; inhabitants’ contribution to control. [Monash University](https://research.monash.edu/en/publications/why-didnt-paris-burn-in-the-seventeenth-and-eighteenth-centuries/) |
| **Nishida, Tsujimoto & Tokunaga, 2003–2004, Edo/Tokyo studies** | Historical fire-size and mortality reconstruction. One series explicitly uses burned area **over 1,653 m²**, illustrating why dataset thresholds must be preserved. [J-STAGE](https://www.jstage.jst.go.jp/article/aijt/9/17/9_KJ00004655666/_article/-char/en) |
| **de Koker et al., 2020, Fire Technology**; DOI **10.1007/s10694-019-00945-2** | Full-scale settlement-spread validation. [Edinburgh Research](https://www.research.ed.ac.uk/en/publications/20-dwelling-large-scale-experiment-of-fire-spread-in-informal-set/) |
| **Wang et al., separation study**; DOI **10.1007/s10694-020-01075-w** | Material-specific exposure and ignition thresholds. [Springer](https://link.springer.com/article/10.1007/s10694-020-01075-w) |
| **Wang & Rush material dataset**; DOI **10.7488/ds/2599** | Cone-calorimeter measurements for combustible materials found in informal settlements; useful inputs for reduced models. [SFPE](https://www.sfpe.org/publications/periodicals/sfpeeuropedigital/sfpeeurope18/issue18feature1) |
| **NIST Technical Note 1661, Report on Residential Fireground Field Experiments** | Staffing, task timing, and coordinated operational performance. [NIST](https://www.nist.gov/news-events/news/2010/09/nist-residential-fire-study-education-kit-now-available) |
| **English Housing Survey; English FIRE statistical tables; USFA residential estimates** | Small-fire prevalence, incident response and damage distributions, and modern cause composition. [GOV.UK](https://assets.publishing.service.gov.uk/media/5b45c43ae5274a3764524d8f/Fire_and_Fire_Safety.pdf) |

### What remains uncertain

**Historical ignition denominators are the largest gap.** Famous-fire catalogs cannot reveal how many small fires householders stopped. Population totals do not repair that missing observation process.

**Historical response-time distributions are thin.** The evidence reviewed here does not support a universal median time-to-water for bucket brigades, Roman watches, or early-modern insurance services. Use explicit task simulation and publish those parameters as assumptions.

**Equipment descriptions are not operational trials.** Museum-reported maximum pump output should not become sustained effective application, especially without water-supply and fatigue constraints.

**Archaeological destruction is not automatically accidental fire.** Preservation, deliberate burning, warfare, and abandonment complicate frequency estimates. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/10.1002/2017JB015190)

**Modern experiments transfer mechanisms better than complete parameter sets.** Roof construction, synthetic contents, ventilation, and household storage differ from historical settings. Separation thresholds and rapid-development curves must retain their experimental context. [Springer](https://link.springer.com/article/10.1007/s10694-020-01075-w)

**Global coverage remains uneven.** This evidence base is strongest for Europe, Japan, selected Ottoman material, North America, Singapore, and South African experiments. It does not justify pretending that comparable premodern annual rates are established for South Asia, Latin America, or most African regions.

**Bottom line:** build fire as an activity-driven combustion and exposure system, and firefighting as a constrained collective-action system. Then a great fire becomes the outcome of actual fuel connections, adverse conditions, delayed intervention, and exhausted capacity—not a date on the simulation’s disaster calendar.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9292e-e4f4-83ea-a7ed-c9419c8ef6ff)
