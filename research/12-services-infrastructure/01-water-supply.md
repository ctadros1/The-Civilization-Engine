# Water supply through history: a simulation-ready report for TCE

**The central design recommendation is to model water service as a chain—source, extraction, conveyance, storage, distribution, household access—not as a building-level bonus or an era progression.** A settlement can have abundant water but poor household access; extensive pipes but intermittent supply; or reliable wells with better drinking water than a prestigious aqueduct. Modern monitoring likewise distinguishes infrastructure from service: safely managed water must be accessible, available when needed, and uncontaminated. [WashData](https://washdata.org/monitoring/drinking-water)

For TCE, the important historical transition is often **from spending labor to obtain each container to obtaining water conveniently at home**. Networks can make that transition possible, but ownership, maintenance, pressure, price, and household connections determine who actually benefits. Eighteenth-century London illustrates the distinction: extensive commercial pipe networks still supplied many houses for only a few hours on selected days, filling basement cisterns from which servants carried water upstairs. [ResearchGate](https://www.researchgate.net/publication/260210492_London%27s_water_supply_before_1800_and_the_roots_of_the_networked_city?utm_source=chatgpt.com)

## 1. Mechanisms: rules the simulation can implement

### 1.1 Keep five different quantities separate

Use distinct accounting for:

| Quantity | Meaning in TCE | Common mistake to avoid |
| --- | --- | --- |
| Physiological intake | Water ingested through beverages and food | Treating all household water as drinking water |
| Household water delivered | Water brought into the dwelling for drinking, cooking, washing, and other uses | Assuming this includes bathing or laundry performed elsewhere |
| Off-site domestic use | Washing at rivers, fountains, bathhouses, or communal facilities | Treating households with little carried water as necessarily never washing |
| Gross system supply | Water entering an aqueduct or distribution network | Dividing this by population and calling the result household consumption |
| Productive and institutional use | Livestock, irrigation, brewing, construction, workshops, baths, gardens, hospitals, firefighting | Hiding these demands inside a universal per-person allowance |

This distinction is especially important for antiquity. A hydraulic reconstruction estimated approximately **121,000 m³/day** reaching Rome through the Anio Novus in its final preserved operating condition. That is one aqueduct’s reconstructed flow, not a measurement of household consumption or even of Rome’s total supply. Ancient flow estimates also depend on uncertain hydraulic assumptions. [Enlighten Publications](https://eprints.gla.ac.uk/129852)

**Implementation rule:** Household demand should be assembled from activities and fixtures. Settlement demand is the sum of those demands plus independently modeled workshops, animals, public facilities, conveyance losses, and deliberate overflow.

### 1.2 Sources supply finite, seasonal water

**Surface sources.** An intake draws from the hydrology model’s actual river, lake, or spring flow. Upstream withdrawals reduce downstream availability; drought, flood turbidity, and damaged intakes alter usable supply.

**Wells.** A well connects a settlement to groundwater; it does not create groundwater. Extraction is constrained by water-table position, aquifer replenishment, water stored in the shaft, lifting capacity, and access congestion. Modern hand-dug-well guidance explicitly distinguishes the shaft’s storage from recharge and identifies groundwater depth and ground stability as construction constraints. [WaterAid](https://www.wateraid.org/us/sites/g/files/jkxoof291/files/technical-brief-hand-dug-wells.pdf)

**Qanats.** A gently graded underground gallery intercepts groundwater and brings it to an outlet by gravity. The useful combination is a water-bearing formation, sufficient elevation difference, and maintainable ground—not simply “desert technology.” [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1506/)

A suitable groundwater balance is:

\[
G\_{t+1}=G\_t+R\_t-W\_t-Q\_{\mathrm{qanat},t}-Q\_{\mathrm{spring},t}
+\text{net lateral flow}
\]

Here all terms are volumes over the same timestep. Springs and qanats must draw from the same groundwater accounting as wells; otherwise TCE will inadvertently duplicate water.

**Emergent consequence:** A new deep-pumping operation can undermine older shallow wells or qanats without physically damaging them. Falling groundwater levels and competition from pumped wells are documented threats to qanat systems. [Springer](https://link.springer.com/article/10.1186/s40068-015-0039-9)

### 1.3 Storage solves timing problems, not necessarily supply problems

A cistern can store roof runoff, courtyard runoff, spring water, aqueduct deliveries, or purchased water. Its value depends on both **refill opportunities** and **the longest likely interruption**.

For a rain-fed cistern:

\[
V\_{t+1}=\operatorname{clamp}
\left(V\_t+P\_t A c+Q\_{\mathrm{in}}\Delta t-D\_t-E\_t-S\_t,\;0,\;V\_{\max}\right)
\]

where \(P\) is rainfall depth, \(A\) catchment area, \(c\) collection efficiency, \(D\) withdrawals, \(E\) evaporation, and \(S\) seepage. Overflow should leave the tank and enter drainage or groundwater accounting.

**Calculated example, not a historical observation:** A 100 m² roof receiving 500 mm/year, with 80% collection efficiency, captures 40 m³/year. That averages about 110 L/day—enough in annual terms for five people using 20 L/person/day. But a 120-day rainless period requires **12 m³ of usable storage**, before reserves and losses. The Texas rainwater-harvesting manual uses this same catchment-and-storage approach and reports typical installer assumptions of 75–90% collection efficiency. [Texas Water Development Board](https://www.twdb.texas.gov/publications/brochures/conservation/doc/RainwaterHarvestingManual_3rdedition.pdf)

**Implementation rule:** Evaluate reliability against seasonal sequences, not annual rainfall alone. A wet climate can still produce severe dry-season shortages.

### 1.4 Conveyance depends on elevation, geometry, and condition

Distinguish two network types:

| Network type | Implementable constraint |
| --- | --- |
| Open or free-surface channels | Flow depends on gradient, cross-section, roughness, and downstream conditions. Channels, tunnels, and aqueduct bridges can be different structures carrying the same water. |
| Full, pressurized pipes | Flow depends on hydraulic head, pipe resistance, fittings, and pressure limits. Pipes may descend and rise again; the pipe itself need not slope continuously downhill. |

EPANET provides an established reference implementation for pressurized networks, including pumps, valves, pressure-dependent delivery, water age, and quality transport. It is not a substitute for a surface-water or open-channel model. [US EPA](https://www.epa.gov/water-research/epanet)

**Implementation rule:** Store each segment’s dimensions, elevation, material, condition, and connection type. Do not assign “one aqueduct = water for 5,000 people.”

For pumped water, the minimum energy accounting follows:

\[
E=\frac{0.002725\,H}{\eta}\quad\mathrm{kWh/m^3}
\]

where \(H\) is total head in metres and \(\eta\) overall efficiency. This is a physical derivation from gravitational potential energy, not a historical productivity estimate. Human, animal, waterwheel, steam, and electric lifting should supply different amounts of work through different machines.

### 1.5 The last distance to the household creates much of the burden

A source’s attractiveness should depend on walking distance, queueing, filling time, price, reliability, access rights, and perceived quality.

One useful generalized cost is:

\[
c\_{hj}=p\_j+
\frac{w\_h}{v\_{\mathrm{load}}}
\left(\frac{2d\_{hj}}{u\_h}+t\_{\mathrm{queue},j}+t\_{\mathrm{fill},j}\right)
+\pi\_{hj}
\]

Here \(p\_j\) is price per litre, \(w\_h\) the household’s value of collection time, \(v\_{\mathrm{load}}\) load size, and \(\pi\_{hj}\) a perceived risk penalty. Perception must be separate from actual contamination.

**Calculated collection example:** A five-person household using 100 L/day needs five trips with a 20 L container. At 500 m each way, 4 km/h walking speed, and five minutes drawing and queueing, collection consumes about **100 minutes/day**. Moving the outlet to 100 m reduces this to about **40 minutes/day**. The improvement saves an hour without adding any source water.

Commercial delivery substitutes someone else’s transport labor and equipment. In late-twentieth-century Tanzania, documented vendors used pushcarts carrying six to eight 20 L containers; vendors expanded as piped service became less reliable. This is a useful observed analogue for TCE’s carrier businesses, not a claim that ancient carriers used identical equipment. [IIED](https://www.iied.org/sites/default/files/pdfs/migrate/9081IIED.pdf)

**Implementation rule:** Schedule real fetching and delivery trips. A fountain can have ample inflow yet inadequate access because too few people can fill vessels simultaneously.

### 1.6 Infrastructure and access rights are separate systems

An outlet should have an owner and an access policy independently of its hydraulic connection.

Represent at least household ownership, neighborhood membership, public access, purchased delivery, subscriptions, and fractional or time-based shares. In Oman, aflaj water allocation includes timed turns monitored with sundials and sluices; some water and land shares are reserved to finance management and maintenance. This is a concrete alternative to both universal municipal supply and unrestricted private ownership. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1207/)

For TCE, an institution might fund construction through taxation, subscriptions, a patron’s expenditure, compulsory labor, a cooperative’s contributions, or an endowment. Those are design options, not mandatory stages of development.

A project becomes politically attractive when some combination of avoided fetching labor, higher land values, productive use, health expectations, prestige, and security outweighs its costs **for the people able to authorize it**. Benefits need not be distributed evenly.

**Implementation rule:** A physically connected household is not automatically entitled to water, and a household with rights is not guaranteed that water will arrive.

### 1.7 Maintenance consumes labor and sometimes interrupts service

Separate maintenance into inspection, cleaning, leak repair, structural work, replacement of lifting equipment, and management of access or allocation.

Frontinus explicitly distinguished repairs that could proceed while water flowed from repairs requiring an aqueduct shutdown. He advised staggering interruptions and avoiding peak-demand periods. His account also identifies encrustation, damaged lining, leakage, weather, and poor original workmanship as problems. [Waters of Rome](https://waters.iath.virginia.edu/frontinus.html)

**Implementation rule:** Maintenance should restore specific properties—cross-section, watertightness, structural condition, machinery reliability—not refill a generic durability bar. Deferring it can save resources today while increasing shortages later.

---

## 2. Parameters: quantitative anchors and their limits

**Evidence labels:** **E** = observation or documentary evidence; **R** = reconstruction; **G** = engineering or service-planning guidance; **P** = proposed simulation prior.

**Confidence:** **High** means a strong anchor for the stated setting; **Medium** means assumptions or limited representativeness matter; **Low** means principally a calibration choice. High confidence in a local observation does **not** imply high confidence in transferring it across societies.

### 2.1 Household quantities

| Setting or parameter | Value | Evidence and confidence | Appropriate use |
| --- | --- | --- | --- |
| Very difficult access | Often **<5 L/person/day** collected; over 1 km or over 30 minutes collection time | WHO 2003 service framework; **G, Medium** | Severe access constraint, not a sustainable target |
| Basic access | Usually **≤20 L/person/day**; approximately 100–1,000 m or 5–30 minutes | WHO 2003; **G, Medium** | Calibration for carried household water |
| Intermediate access | About **50 L/person/day** with a tap on the plot or very nearby | WHO 2003; **G, Medium** | Convenient but not necessarily fully plumbed households |
| Multiple taps, continuous service | **100+ L/person/day** | WHO 2003; **G, Medium** | Higher-service starting point, not a universal modern norm |
| Drinking plus food preparation allowance | **7.5 L/person/day** in the report’s planning calculation | WHO 2003; **G, Medium** | Includes preparation; **not** a claim that everyone drinks 7.5 L |
| Tanzania, unpiped study households, 1997 | Mean **18.6 L/person/day**, \(n=61\) | *Drawers of Water II*; **E, High for sample** | Empirical carried-water benchmark |
| Tanzania, piped study households, 1997 | Mean **80.2 L/person/day**, \(n=131\) | Same study; **E, High for sample** | Household connection benchmark with substantial variation |
| United States, domestic use, 2015 | **82 US gal/person/day ≈310 L/person/day** | USGS; **E, High nationally** | Includes residential outdoor use; not a global or indoor-only average |

The WHO figures are **service-level benchmarks from the 2003 report**, not measured prehistoric or medieval averages. Its 2020 second edition revisits quantity, access, reliability, and affordability; neither should be converted into a rigid historical era table. [Programme Solidarité Eau](https://bdd.pseau.org/outils/ouvrages/who_domestic_water_quantity_service_level_and_health_2003.pdf) The Tanzania values come from the report’s household comparison, while the USGS figure uses a broader domestic-use definition. [IIED](https://www.iied.org/sites/default/files/pdfs/migrate/9081IIED.pdf)

**Recommended TCE initialization:** Where evidence is absent, use **10–30 L/person/day for hand-carried household supply** and **30–100 L/person/day for convenient on-plot supply** as **P, Low-confidence historical-transfer ranges**. These are starting distributions anchored to access evidence, not estimates of “what Neolithic people used.” Then let activities, source-side washing, prices, containers, fixtures, and shortages determine realized use.

### 2.2 Structures, containers, and hydraulic capacity

| Parameter | Quantitative anchor | Evidence, confidence, and transfer limits |
| --- | --- | --- |
| Southern African ostrich-eggshell water container | About **1 L/container**; ethnographic accounts describe **8–10 per family** in one !Kung setting | Lander & Russell’s archaeological/ethnographic synthesis; **E, Medium**. Storage and transport evidence, not daily consumption. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7343145/?utm_source=chatgpt.com) |
| Hand-dug-well depth | Approximately **5 to >20 m**; depths beyond 30 m possible | WaterAid technical guide; **G, Medium**. Modern manual-well analogue, strongly geology-dependent. [WaterAid](https://www.wateraid.org/us/sites/g/files/jkxoof291/files/technical-brief-hand-dug-wells.pdf) |
| Hand-dug-well dimensions | Example excavation diameter **1.5 m**, lined internal diameter **1.2 m** | Same guide; **G, Medium**. Do not transfer its concrete construction recipe to early societies. [WaterAid](https://www.wateraid.org/us/sites/g/files/jkxoof291/files/technical-brief-hand-dug-wells.pdf) |
| Qanat gallery | Approximately **0.8–1.0 m wide**, **1.2–2.0 m high** | Nasiri & Mafakheri 2015; **G/synthesis, Medium**. Useful working-space geometry. [Springer](https://link.springer.com/article/10.1186/s40068-015-0039-9) |
| Qanat access-shaft spacing | Approximately **20–50 m** | Same source; **Medium**. Actual spacing depends on ground and construction practice. [Springer](https://link.springer.com/article/10.1186/s40068-015-0039-9) |
| Roof collection efficiency | **75–90%** | Texas manual; **G, Medium**. Reduce for unsuitable roofs, poor gutters, intentional first-flush diversion, or overflow. [Texas Water Development Board](https://www.twdb.texas.gov/publications/brochures/conservation/doc/RainwaterHarvestingManual_3rdedition.pdf) |
| Corriental reservoir, Tikal | Approximately **58,000 m³** | Archaeological estimate; **R, Medium**. A major urban reservoir, not a household cistern template. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7582844/) |
| Pompeian fountain-supply pipes | Estimated **0.1–2.5 L/s** across 47 proposed connections | Monteleone, Crapper & Motta 2023; **R, Medium**. Hydraulic reconstruction, not measured ancient use. [Academia.edu](https://independent.academia.edu/DrMariaMonteleone?utm_source=chatgpt.com) |
| Anio Novus terminal flow | **1.4 ±0.4 m³/s**, approximately **121,000 ±35,000 m³/day** | Keenan-Jones et al. 2015; **R, Medium**. Site- and operating-condition-specific. [Enlighten Publications](https://eprints.gla.ac.uk/129852) |

**Do not assign a universal well yield from depth alone.** A deeper well can still have poor recharge; a shallow well in favorable material can perform well. Nor should a fountain’s entire continuous discharge count as collected household water: some may overflow or serve secondary uses.

### 2.3 Construction, operation, and financial costs

Historical costs are best represented as **labor, materials, lifting/haulage, specialist work, and institutional expenditure**, rather than converted into present-day dollars.

| Cost or duration anchor | Value | Interpretation and confidence |
| --- | --- | --- |
| Manual open excavation, soft soil | **5.0 m³/worker-day** | ILO recommended labor-based construction norm; **G, Medium** |
| Manual excavation, medium / hard / very hard ground | **3.5 / 3.0 / 2.0 m³/worker-day** | Same guidance; modern tools and task conditions, not ancient shaft productivity |
| Manual excavation, rock category | **0.8 m³/worker-day** | Same guidance; rock type and method still matter greatly |
| Aqua Marcia project and repairs to older aqueducts | **180 million sestertii appropriated** | Frontinus’ documentary report; **E, Medium**. A combined works appropriation, not Marcia alone |
| Roman aqueduct maintenance establishment | Approximately **240 public +460 imperial enslaved workers** | Frontinus; **E, Medium**. Core crews, not all labor associated with Rome’s water economy |
| Tamagawa canal, Edo | About **43 km** excavated in **8 months**, 1653; urban supply followed in 1654 | Tokyo water authority’s historical account; **E, Medium**. Workforce unknown; not a reusable km/month productivity rate |
| London domestic water charge, eighteenth century | Typically **20–40 shillings/year**, described as around **10% of a laborer’s annual wage** | Tomory 2015; **E, Medium**. Contracts and households varied |
| Moshi, Tanzania, study-period retail example | About **5 Tanzanian shillings/20 L** at kiosks; roughly **double** delivered by vendors | *Drawers of Water II*; **E, High for reported setting**, low transferability |

Sources: ILO excavation tables; Frontinus §§7 and 116–117; Tokyo’s Tamagawa history; Tomory’s archival study; Tanzania field report. [International Labour Organization](https://wwwex.ilo.org/dyn/asist/asistdocs.downloadfile?p_filename=F170469982%2FTechinical+Brief+No.2+-+Productivity+Norms+for+la.pdf)

**Calculated construction example:** A 1 km pipe trench, 0.5 m wide and 1 m deep, requires 500 m³ of excavation. At 2–5 m³/worker-day, excavation alone takes **100–250 worker-days**. Pipe production, transport, joints, shoring, dewatering, backfilling, access disputes, and supervision are additional. Applying that rate directly to a deep well or confined qanat gallery would be unjustified.

For authored recipes, use:

\[
L\_{\mathrm{construction}}=
L\_{\mathrm{excavation}}+
L\_{\mathrm{lining}}+
L\_{\mathrm{haulage}}+
L\_{\mathrm{assembly}}+
L\_{\mathrm{survey}}+
L\_{\mathrm{water\ control}}
\]

Calendar duration must additionally reflect limited work fronts, specialist availability, seasonal labor, funding interruptions, and material delivery.

**Maintenance-cost recommendation:** Avoid a universal “2% of capital cost annually” rule for all historical waterworks. Generate work orders from sediment volume, leaking joints, damaged lining, worn ropes, failed pumps, and inspection distances. The qanat literature specifically notes inadequate documentation for generalizable construction-time and cost estimates. [Springer](https://link.springer.com/article/10.1186/s40068-015-0039-9)

Keep **physical resource costs separate from financial transfers**. Compulsory or enslaved labor is not economically free: it consumes time, subsistence, supervision, and foregone production even when no wage is paid.

---

## 3. Variation across eras and world regions

### 3.1 Eras should change available techniques and institutions—not biological demand

| Broad setting | Historically grounded pattern | TCE treatment of per-capita use |
| --- | --- | --- |
| Foragers | Portable storage and water caching can matter alongside direct access to sources. Southern African evidence includes personally owned eggshell containers and buried reserves. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7343145/?utm_source=chatgpt.com) | Use access-based demand. Do not infer total use from the amount transported back to camp. |
| Early farming | Permanent settlements can invest in wells and skilled lining construction. Four early Neolithic German wells contained oak timbers dated **5469–5098 BCE**, demonstrating sophisticated woodworking without metal tools. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0051374) | Carried-water priors where applicable; nearby private wells can support better access without a city network. |
| Pre-industrial towns and states | Wells, imported water, fountains, reservoirs, private connections, and carriers can coexist. Mohenjo-daro provides both private-building wells and street-accessible examples. [Harappa](https://www.harappa.com/indus6/publicwell78.html) | Use a distribution across service levels, not one town-wide number. |
| Industrializing settlements | Connection expansion can precede continuous delivery. Early London networks still required household storage and internal carrying. [ResearchGate](https://www.researchgate.net/publication/260210492_London%27s_water_supply_before_1800_and_the_roots_of_the_networked_city?utm_source=chatgpt.com) | Pipe access, service hours, pressure, indoor fixtures, and wastewater arrangements become separate variables. |
| Modern settlements | Reliable household connections coexist globally with collection from distant or unreliable sources. JMP estimated **74%** global safely managed coverage in 2024—not universal access. [UNICEF DATA](https://data.unicef.org/resources/jmp-report-2025/) | All earlier service states remain possible; technology availability does not guarantee service. |

### 3.2 Regional cases that should shape the building grammars

**South Asia: wells are not the only Indus solution.** Mohenjo-daro’s building-integrated and accessible wells contrast with Dholavira’s elaborate water-conservation and reservoir arrangements in an arid setting. TCE should allow different water systems within broadly similar craft and social capabilities because local hydrology differs. Do not infer identical household access from a shared “Indus technology” label. [Harappa](https://www.harappa.com/indus6/publicwell78.html)

**Iran and Oman: shared groundwater works require allocation institutions.** Qanats make groundwater available at the surface without continuous pumping. Omani aflaj demonstrate that flow can be allocated through time shares, with dedicated revenues for upkeep. Model shareholders, maintainers, and allocation schedules separately; do not assume all gravity irrigation systems have the same ownership rules. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1506/)

**Saharan North Africa: knowledge is infrastructure.** UNESCO’s documentation of foggara water measurers in Algeria describes specialist knowledge for calculating shares and maintaining distribution devices. This supports a TCE occupation whose disappearance can impair allocation and maintenance even while the tunnels remain physically present. [UNESCO ICH](https://ich.unesco.org/en/RL/knowledge-and-skills-of-the-water-measurers-of-the-foggaras-or-water-bailiffs-of-touat-and-tidikelt-01274)

**Maya lowlands: abundant rainfall can coexist with storage dependence.** At Tikal, reservoir management was crucial. Zeolite and quartz deposits at Corriental have been interpreted as evidence of deliberate filtration; that interpretation does not establish a known pathogen-removal percentage. Other reservoirs contain evidence of mercury contamination and potentially harmful cyanobacteria. Reservoirs within one city therefore need separate quality histories. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7582844/)

**Japan: substantial networks do not require industrial pipes.** Edo’s Tamagawa system combined a long open channel with underground stone and wooden conduits. Its roughly 43 km intake-to-city route exploited about 92 m of elevation difference. Give timber conduits a viable place in the technology graph rather than making metal pipe manufacture the prerequisite for every network. [Waterworks Bureau Tokyo](https://www.waterworks.metro.tokyo.lg.jp/kouhou/meisho/tamagawa)

**Europe: commercial networks were not an automatic response to technological possibility.** Tomory’s study finds initially slow uptake of New River household connections, followed by major expansion and reorganization. TCE should distinguish an invention, a financially viable provider, and households deciding to subscribe. A newly available connection should not instantly convert all eligible buildings. [ResearchGate](https://www.researchgate.net/publication/260210492_London%27s_water_supply_before_1800_and_the_roots_of_the_networked_city?utm_source=chatgpt.com)

---

## 4. Water quality: represent mechanisms, not a single “purity” score

A compact model should distinguish **microbial contamination, suspended material, dissolved chemicals/salinity, and storage or distribution condition**. WHO’s drinking-water guidelines use hazard identification and management across the supply chain rather than assuming a source type guarantees safety. [World Health Organization](https://www.who.int/publications/i/item/9789240045064)

| Problem | Causal pathway to represent | Appropriate response in TCE |
| --- | --- | --- |
| Fecal contamination | Waste reaches an intake, well, vessel, or distribution system | Source protection, waste separation, appropriate treatment, cleaner collection and storage |
| Flood-related turbidity | Runoff introduces suspended sediment and associated contaminants | Intake interruption, settling, filtration, alternate sources |
| Geological contaminants | Aquifer chemistry produces unsafe dissolved substances | Testing knowledge, alternative aquifers, source switching, suitable treatment |
| Storage contamination | Dirty collection vessels, runoff, animal access, nutrient loading, or poor maintenance | Covers, drainage, cleaning, protected catchments, treatment |
| Treatment mismatch | An intervention addresses one hazard but not another | Hazard-specific effectiveness rather than a universal percentage improvement |

Protected-looking groundwater is not inherently safe. A field study around Dodowa, Ghana, found microbial contamination in sampled hand-dug wells and boreholes; the result is local evidence of a pathway, not a claim about all wells. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5923772/) Geological quality also matters: WHO’s provisional arsenic guideline is **10 µg/L**, and groundwater contamination is documented across numerous regions. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/arsenic)

**Do not make boiling or disinfection remove every contaminant.** CDC explicitly distinguishes microbial treatment from water contaminated by toxic chemicals, which boiling or disinfection does not make safe. [CDC](https://www.cdc.gov/water-emergency/about/index.html)

For historical behavior, agents should act on their **knowledge and observations**, not hidden simulation truth. Clear, pleasant-tasting water may be preferred without being microbiologically safe. Conversely, an institution may improve protection through customary rules without possessing germ theory.

For disease calculations, exposure should depend on the concentration in water actually ingested and a pathogen-specific response. Limited washing water can create an additional hygiene pathway. Do not translate each litre below a target into a fixed number of deaths.

---

## 5. Stylized facts a correct simulation should reproduce

| Pattern | Quantitative or historical target | Test for TCE |
| --- | --- | --- |
| Convenient access produces much higher household use | In the Tanzania 1997 sample, piped households averaged **4.3 times** unpiped use | Comparable households should diverge when collection cost changes, without an era transition |
| Service can deteriorate despite installed infrastructure | Mean piped use in the study fell from **141.8 to 80.2 L/person/day** between the late 1960s and 1997—about **43%** | Underfunding, interruptions, and demand growth must be able to reverse service gains |
| Partial blockage can materially reduce capacity | Anio Novus modeling found even minimal travertine could reduce maximum flow by about **25%** | Condition must affect hydraulics before total structural failure |
| Water and sanitation can be complementary | Massachusetts evidence attributes about **one-third of the decline in log under-five mortality** over 1880–1920 to their combined expansion | More water alone should not always produce the full health benefit |
| Collection labor is socially unequal | WHO/UNICEF reported women and girls collected water in **7 in 10 households** without on-premises supplies | Allocate work through household roles and institutions, not a biologically fixed female-only task |
| “Piped” is not synonymous with continuous or effortless | Historical London households could receive water only a few hours on selected days | Cisterns and carrying labor should remain useful after connection |
| Redundancy can be worth more than maximum capacity | Different sources and storage can cover maintenance interruptions or seasonal failures | Households and institutions should sometimes retain older wells after constructing a network |

Sources: Tanzania household survey; Anio Novus reconstruction; Alsan and Goldin’s Massachusetts analysis; WHO/UNICEF collection-burden report; Tomory’s London study. These are local or conditional findings, not portable universal coefficients. [IIED](https://www.iied.org/sites/default/files/pdfs/migrate/9081IIED.pdf)

Two additional **derived tests** are especially useful.

First, at **50,000 people**, household use of 20 L/person/day requires **1,000 m³/day**; 100 L/person/day requires **5,000 m³/day**. These exclude workshops, animals, irrigation, and losses. They provide a scale check for authored wells and channels.

Second, queueing should become severe when arrivals approach filling capacity. Adding another outlet or drawing position can improve access without changing the aqueduct. Conversely, adding trunk capacity may accomplish little when the bottleneck is a locked gate, an unaffordable fee, or a single drawing station.

---

## 6. Recommended representation for TCE

### 6.1 State model

The following is a proposed implementation, rather than a claim that historical societies represented water this way.

| Entity | Minimum useful state |
| --- | --- |
| Source | Hydrology reference, elevation/head, available quantity, seasonal behavior, quality, extraction permissions |
| Storage | Capacity, current volume, covered/open state, catchment, quality, water-age approximation, leakage |
| Conveyance segment | Endpoints, geometry, material, free-surface/pressurized type, condition, flow limit, current flow |
| Outlet | Drawing positions, filling rate, opening hours, access policy, queue, price |
| Household | Containers, stored water, planned uses, known sources, source perceptions, connection rights |
| Carrier enterprise | Workers, vessels, carts/animals, purchase sources, delivery contracts, route plans |
| Water institution | Assets, revenues, obligations, staff, access rules, maintenance backlog, allocation schedule |
| Specialist | Relevant craft knowledge, experience, tools, apprenticeship links |

**Households should plan water acquisition; individual agents should perform it.** This avoids every person independently deciding to fetch the same household’s supply, while retaining visible daily life.

When collecting, reserve an outlet slot or join its queue, but withdraw water only when filling actually occurs. Containers and household tanks then carry real volume and quality state. Interruptions should cause replanning, rationing, purchases, or use of an inferior fallback source.

### 6.2 Simulation timescales

A practical division for 10k–50k agents is:

| Process | Proposed update strategy |
| --- | --- |
| Household activities, arrival, queueing, filling | Discrete events |
| Distribution allocation | Per connected system, on meaningful demand/control changes or a coarse simulation interval |
| Storage volume | Integrated from flows and event withdrawals |
| Groundwater and seasonal source changes | Coarser hydrology timestep, with extraction accumulated between updates |
| Water quality | Lumped source/storage/network updates, plus contamination events |
| Asset inspection and maintenance | Scheduled jobs and condition events |
| Investment decisions | Periodic institutional evaluation, plus crises and petitions |

The renderer does not need a hydraulic solve every frame. It needs outlet state, water levels, visible flows, people carrying vessels, construction progress, and repair activity.

### 6.3 Simplifications worth making

For the first version, use a **capacity-constrained gravity network** for channels, an aggregated groundwater representation, and explicit household transport. Introduce pressure-dependent network solving when pumps, multiple reservoirs, or substantial elevation differences make fixed capacities misleading.

Model quality as a small vector or set of hazard classes. Track mixing in reservoirs and containers; use water-age bins where necessary. Do not simulate individual microbes or full three-dimensional groundwater transport.

Keep disease, water demand, sanitation, and health knowledge connected but separate. A water institution may know that a source is unreliable without knowing why it causes illness.

For maintenance, calculate a backlog in actual jobs and workdays. A neglected channel should gradually lose capacity through deposition or leakage, with occasional disruptive failures. Preserve the possibility that a settlement knowingly tolerates this because harvest labor, war, or fiscal crisis has higher immediate priority.

### 6.4 Technology should unlock components and skills

Rather than a sequence of “well → cistern → aqueduct → pipes,” use interacting capabilities:

* **Containers and lifting devices** change load size and extraction productivity.
* **Excavation, lining, waterproofing, and surveying** enable different wells, tanks, tunnels, and channels.
* **Pipe fabrication, joints, valves, and pumps** improve conveyance and control.
* **Treatment knowledge and operating capacity** address particular quality hazards.
* **Measurement, accounting, and allocation institutions** make larger shared systems governable.

These should be alternative and complementary paths. A cistern does not require an aqueduct; a wooden pipe network does not require industrial metallurgy; and a well can remain valuable after both exist.

### 6.5 Existing models and games to borrow from

| Reference | What it already represents | Best use for TCE |
| --- | --- | --- |
| **EPANET** | Pressurized-network hydraulics, pumps, valves, water quality, water age, pressure-dependent delivery | Reference solver and validation cases for Rust network behavior. [US EPA](https://www.epa.gov/water-research/epanet) |
| **Water Network Tool for Resilience—WNTR** | Network disruption and resilience analysis, including leaks and damage scenarios | Offline stress tests and sensitivity analysis; not an individual daily-life model. [EPA GitHub Pages](https://usepa.github.io/WNTR/) |
| **Workers & Resources: Soviet Republic** | Water/sewage networks, consumption and flow overlays, source pollution and treatment decisions | Infrastructure legibility and feedback between supply and waste handling. [Soviet Republic](https://www.sovietrepublic.net/post/report-for-the-community-45) |
| **Timberborn** | Terrain-dependent water management, dams, storage, and drought preparation | Readable seasonal risk and storage decisions; do not import fictional hazard cycles as historical mechanisms. [Steam Store](https://store.steampowered.com/app/1062090/Timberborn/) |

TCE’s distinctive requirement is to connect these physical systems to **actual labor, unequal rights, household choices, and institutions that must keep functioning**.

---

## 7. Research base, datasets, and uncertainty

### Most useful source groups

**Household quantity and access:** Howard and Bartram’s *Domestic Water Quantity, Service Level and Health* supplies the service-access framework; the 2020 edition updates the review. Mujwahuzi’s Tanzania volume of *Drawers of Water II* supplies observed household quantities, costs, service interruptions, and carrier practices. Use the latter as contextual empirical calibration—not as a substitute for measurements of ancient populations. [Programme Solidarité Eau](https://bdd.pseau.org/outils/ouvrages/who_domestic_water_quantity_service_level_and_health_2003.pdf)

**Historical hydraulics and institutions:** Frontinus supplies administrative evidence; Keenan-Jones and colleagues and Monteleone and colleagues supply hydraulic reconstructions. Tomory supplies an archival account of commercial network development. Their different evidence types should remain distinguishable in the parameter database. [Waters of Rome](https://waters.iath.virginia.edu/frontinus.html)

**Archaeological diversity:** Tegel and colleagues document early timber wells; Lander and Russell document organic containers; Tankersley and colleagues and Lentz and colleagues investigate Tikal’s filtration interpretation and contamination evidence. These are particularly useful checks against Europe-only technological narratives. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0051374)

### Datasets suitable for calibration

| Dataset or source | Useful variables | Important limitation |
| --- | --- | --- |
| WHO/UNICEF **JMP**, including the 2000–2024 release | Source type, access, availability, quality, and inequality | Coverage is not litres consumed or network hydraulic capacity. [UNICEF DATA](https://data.unicef.org/resources/jmp-report-2025/) |
| **USGS water-use estimates** | Domestic quantities and public/self-supplied categories | Modern US context; outdoor residential use matters. [USGS](https://www.usgs.gov/mission-areas/water-resources/science/domestic-water-use) |
| **Drawers of Water** studies | Household quantities, collection behavior, costs, changes over time | Specific East African sites and survey methods. [IIED](https://www.iied.org/sites/default/files/pdfs/migrate/9081IIED.pdf) |
| **Water Point Data Exchange—WPdx** | Water-point locations and operational status | Standardized planning capacities are not measurements of every source’s actual yield. [WaterPoint Data](https://www.waterpointdata.org/2021/10/07/wpdx-launches-new-rehabilitation-priority-tool/) |

### Claims that should remain flagged

**Ancient household litres per day:** Evidence is thin. Infrastructure dimensions, nominal allocations, and reconstructed flow do not directly reveal ordinary household consumption.

**Construction costs:** Excavated volume is comparatively tractable; ancient work rates, financing, interruptions, and specialist labor are much less certain. Modern manual-work norms are analogues, not historical measurements.

**Longevity:** A surviving ancient structure demonstrates survival, not continuous service without reconstruction or maintenance.

**Treatment effectiveness:** Tikal’s proposed filtration system is important evidence of practice, but it does not supply a defensible universal disinfection coefficient. Sediment contaminants likewise are not direct measurements of the concentration people drank. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7582844/)

**Institutional causation:** Shared hydraulic works should not force a particular government form. The model should permit cooperation, patronage, commercial provision, coercion, and conflict to emerge around the same physical technology.

**Bottom line:** For v1, prioritize finite sources, household containers, fetching time, queues, seasonal storage, access rights, and maintenance jobs. These make a modest village well socially and economically consequential. Larger networks should extend that same system—not replace it with a city-wide water-coverage bonus.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92929-132c-83ea-8d0e-f7442b640c14)
