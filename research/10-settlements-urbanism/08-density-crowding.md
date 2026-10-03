# Urban density and crowding across history

## Executive finding

**TCE should derive density from land use, buildings, and actual occupants—not assign a maximum population density to each technological era.** The same population density can result from tightly packed single-storey houses, spacious multistorey apartments, or overcrowded rooms. Those configurations have different implications for construction, daily movement, disease, fire, and social life. Urban economics likewise treats density as the outcome of competing benefits and costs of proximity, rather than as a simple technological ceiling. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.34.3.3)

The historical evidence supports a very wide range: dispersed agricultural communities with only a few residents per hectare; villages with tens; compact preindustrial settlements and districts with hundreds; and exceptional industrial neighbourhoods exceeding a thousand. **These numbers are meaningful only when the geographical denominator is specified.** A camp footprint, residential quarter, walled city, and agricultural metropolitan region are not interchangeable.

For TCE, the most important separation is:

> **Settlement density determines spatial concentration. Household crowding determines living conditions. Building configuration and infrastructure determine how concentration translates into hazards.**

---

## 1. Define the quantities before calibrating them

### 1.1 Keep several densities, not one

I recommend the following measurement system:

| Quantity | Definition | Main use in TCE |
| --- | --- | --- |
| Regional density | Residents / total regional land | Agricultural support, settlement spacing, travel between communities |
| Gross settlement density | Residents / consistently delineated settlement footprint, including internal streets and open land | City size, neighbourhood comparisons |
| Residential parcel density | Residents / land assigned to residential or mixed residential plots | Parcel allocation and housing development |
| Residential floor-area ratio | Gross residential floor area / relevant land area | Construction intensity |
| Household crowding | Occupied usable residential m²/person, supplemented by sleepers/room | Privacy, housing stress, indoor exposure |
| Activity density | People actually present / relevant space at a particular time | Markets, workplaces, public gatherings, evacuation |

This avoids a common problem: administrative boundaries, built-up footprints, and residential areas can produce dramatically different reported densities for the same city. Measurement choices are central to the urban-density literature. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.34.3.3)

### 1.2 An accounting identity suitable for TCE

For a consistently defined settlement:

\[
D\_g=10{,}000\frac{s\,c\,h\,e\,o}{a}
\]

where:

* \(D\_g\): gross density, persons/ha.
* \(s\): fraction of settlement land in residential or mixed residential plots.
* \(c\): residential-building footprint divided by that plot area.
* \(h\): effective residential storeys—gross residential floor area divided by building footprint.
* \(e\): usable residential floor area / gross residential floor area.
* \(o\): occupied usable floor area / available usable residential floor area.
* \(a\): occupied usable residential floor area per actual resident, m²/person.

**Calculate these from summed areas, not independently averaged buildings.** For mixed-use buildings, exclude commercial floors from residential floor area. Here occupancy is an *area fraction*, not the fraction of housing units occupied.

A hypothetical configuration with \(s=0.5,\ c=0.5,\ h=2,\ e=0.8,\ o=0.9,\ a=15\) produces **240 persons/ha**. With one storey and 10 m²/person, the same land allocation produces **180 persons/ha**. These are arithmetic examples, not historical estimates.

The footprint implications for TCE are substantial:

| Residents | Gross density | Settlement area |
| --- | --- | --- |
| 10,000 | 100 persons/ha | 1 km² |
| 50,000 | 50 persons/ha | 10 km² |
| 50,000 | 100 persons/ha | 5 km² |
| 50,000 | 200 persons/ha | 2.5 km² |

Actual occupancy should remain authoritative. A building may have a customary occupancy, a legal occupancy, and a much higher actual occupancy; those should not silently become the same number.

---

## 2. Quantitative evidence across periods and regions

**Confidence notation:** **H** means a well-documented count, definition, or physical example within its stated scope; **M** means an explicit but assumption-dependent reconstruction or survey estimate; **L** means substantial uncertainty about population, boundaries, or contemporaneous occupation. These are evidence assessments, not statistical confidence intervals.

### 2.1 Historical density anchors

The following are deliberately labelled by spatial scale. **Do not average this table into a historical density curve.**

| Case and period | Persons/ha | What the denominator includes | Confidence and source |
| --- | --- | --- | --- |
| Mobile hunter-gatherer camps, ethnographic sample | **≈526** equivalent to median 19 m² of camp area/person | Immediate camp footprint, **not subsistence territory** | **M** for sampled camps; **L** as a direct prehistoric analogue. Lobo et al., 2022. [ResearchGate](https://www.researchgate.net/publication/359403463_Scaling_of_Hunter-Gatherer_Camp_Size_and_Human_Sociality) |
| Mesitas, Colombia, Regional Classic | **1.4** | Scattered farmsteads and intervening agricultural land | **L–M**. Berrey, Drennan and Peterson, 2021. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0252532) |
| Hongshan villages, northeastern China, 4500–3000 BCE | **≈15** | Dispersed village extent | **L–M**; survey-based estimate. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0252532) |
| Formative villages, Oaxaca, Mexico | **≈20**, subsequently **35–40+** | Village footprints | **L–M**; change over several centuries. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0252532) |
| Barinas, Venezuela, Late Gaván, approximately 550–1000 CE | **≈25** | Bounded villages | **L–M**. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0252532) |
| Pompeii, 79 CE | **≈130** in one recent reconstruction | Urban area | **L–M**; reconstructed population, not a surviving census. [Springer](https://link.springer.com/article/10.1007/s10816-023-09604-x) |
| Teotihuacan, Classic-period Mexico | **51.4** | Area within the reconstructed urban boundary | **M**; population assigned to mapped dwellings. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416524000345?utm_source=chatgpt.com) |
| Jenné-Jeno, Mali, Iron Age | **≈220** | Settlement/residential extent | **L–M**; estimated population around 7,300. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0252532) |
| Greater Angkor Region, Cambodia, thirteenth century | **≈2.3–3.0** | Approximately 3,000 km², including extensive agricultural and hydraulic landscapes | **L–M**; calculated from 700,000–900,000 estimated residents. **Not central-city density.** [PubMed](https://pubmed.ncbi.nlm.nih.gov/33962951/) |
| Commoner districts of early-modern Edo, Japan | **≈400–600** | Particular residential districts, not all Edo | **M**; historical estimates discussed by Tajima. [ResearchGate](https://www.researchgate.net/publication/237756418_The_marketing_of_urban_human_waste_in_the_early_modern_EdoTokyo_Metropolitan_area) |
| New York City’s Tenth Ward, 1903 | **≈1,643** | Ward area | **M–H**; conversion of the reported 665 persons/acre. An exceptional crowded district. [The Skyscraper Museum](https://www.skyscraper.org/housing-density/crowding/) |

The hunter-gatherer result is especially instructive. Small, temporary camps can have high immediate residential concentration without supporting a permanently dense town. Lobo et al.’s mobile-camp subsample contains 748 observations; its median population is 23. Camp layout, duration, surrounding territory, and relationships among households matter more than the superficial comparison with an urban hectare. [ResearchGate](https://www.researchgate.net/publication/359403463_Scaling_of_Hunter-Gatherer_Camp_Size_and_Human_Sociality)

### 2.2 Modern comparisons: density is not height

Angel, Lamson-Hall and González Blanco’s **2021 pilot study** decomposes density for ten cities. Selected results below refer to the study’s **2010–2015 urban footprints and estimates**, not current administrative-city statistics. Confidence is **M**: inputs combine spatial sampling and secondary housing information. [ResearchGate](https://www.researchgate.net/publication/350326352_Anatomy_of_density_measurable_factors_that_constitute_urban_density)

| Study city and footprint date | Gross persons/ha | Effective residential storeys | Occupied usable residential m²/person |
| --- | --- | --- | --- |
| Dhaka, 2014 | 372 | 2.5 | 11 |
| Hong Kong, 2013 | 352 | 20.5 | 14 |
| Kinshasa, 2013 | 224 | 1.1 | 4 |
| Bogotá, 2010 | 196 | 2.8 | 19 |
| Cairo, 2013 | 115 | 4.4 | 22 |
| Madrid, 2010 | 62 | 3.4 | 19 |
| Bangkok, 2015 | 48 | 1.9 | 30 |
| Minneapolis, 2014 | 10 | 1.4 | 45 |

“Effective storeys” is floor area divided by footprint, not the height of a typical landmark. The table demonstrates that similar densities can arise from very different building and occupancy patterns; these selected cases are not a representative global distribution. [ResearchGate](https://www.researchgate.net/publication/350326352_Anatomy_of_density_measurable_factors_that_constitute_urban_density)

### 2.3 Archaeological estimates require special handling

**Contemporaneity is often more important than arithmetic precision.** A settlement excavated across centuries contains houses that were never occupied together.

Kuijt and Marciniak’s 2024 reassessment estimates **600–800 people in an average year during Çatalhöyük East’s Middle phase, 6700–6500 cal BCE**, rather than the several thousand commonly inferred previously. This is a consequential methodological revision, not a reason to declare the new estimate uncontested. Nor should its population be divided casually by the entire archaeological mound to manufacture a contemporary density. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416524000047?utm_source=chatgpt.com)

A second problem is circularity. Some archaeological populations were estimated by multiplying mapped area by an assumed density. For example, **250 persons/ha** was used as an estimation coefficient in work on Middle Bronze Age Palestine. Such a value is a reconstruction input, **not independent evidence that settlements actually averaged 250 persons/ha**. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.2307/1357066)

For TCE calibration, record whether each observation is based on counted residents, reconstructed households, estimated floor space, or an assumed population-per-area coefficient.

---

## 3. Mechanisms: why settlements concentrate, spread, and build upward

The historical findings suggest the following implementable rules. The right-hand column is a **modeling recommendation**, not a claim that one equation explains every society.

| Causal process | Evidence and interpretation | Implementable TCE rule |
| --- | --- | --- |
| **Access to fields versus access to people** | Dispersed farmers shorten agricultural travel; village residents shorten social and exchange travel. These benefits trade off. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0252532) | Evaluate household locations against both field travel and access to exchange, labour partners, water, and kin. |
| **Interdependence and specialization** | Household specialization and aggregation can reinforce one another; agricultural agent models reproduce this relationship under specified assumptions. [JASSS](https://www.jasss.org/16/4/4.html?utm_source=chatgpt.com) | Frequent exchange and shared production increase the value of proximity, but only where institutions permit access. |
| **Defence and constrained expansion** | Medieval towns could densify inside walls, develop suburbs, and eventually extend their defended perimeter. Walls did not always coincide with fully occupied land. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0162678) | Compare the cost of infill and crowding with expansion, defence, and travel costs. Permit unsafe or unofficial suburbs. |
| **Unequal access to land** | Edo combined crowded commoner districts with land allocations dominated by warrior estates; land availability was institutionally segmented. [ResearchGate](https://www.researchgate.net/publication/237756418_The_marketing_of_urban_human_waste_in_the_early_modern_EdoTokyo_Metropolitan_area) | Restrict who may occupy, subdivide, inherit, lease, or redevelop each parcel. Scarcity can be political rather than geographical. |
| **Transport and purchasing power** | Density reflects the balance between accessibility, housing costs, space consumption, and transport costs. Better transport need not raise metropolitan density. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.34.3.3) | Cheaper travel makes peripheral locations more competitive; greater resources can increase desired floor and outdoor space. |
| **Infrastructure capacity** | Water supply and sewerage can jointly improve health; their effects are complementary rather than interchangeable. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/700766?utm_source=chatgpt.com) | Track delivered safe water, waste removal, drainage, and service reliability separately from population density. |
| **Inherited urban structure** | Administrative allocation and redevelopment constraints can preserve land uses that no longer match accessibility. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0094119096910979) | Existing buildings, ownership, roads, and protected uses impose switching costs; avoid instantaneous spatial equilibrium. |

### 3.1 Technology changes the feasible choices—not the desired answer

Represent building technology through capabilities: wall and frame systems, available spans, foundations, joining methods, lifting equipment, vertical circulation, water delivery, and maintenance requirements. Then let households and institutions choose among feasible forms.

This is preferable to “early agriculture: maximum two floors” or “elevators unlock all buildings above five floors.” **Shibam’s documented mud-brick houses reach seven storeys**, demonstrating that substantial residential height is possible without modern structural systems or elevators. It is evidence of possibility, not of widespread affordability. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/192?utm_source=chatgpt.com)

Height should carry several independent penalties in TCE:

* Increasing structural and construction requirements.
* Stair travel, goods carrying, and water carrying.
* Loss of usable space to walls and circulation.
* Maintenance, evacuation, and fire-response difficulty.

These are proposed engineering abstractions. Their relative costs should depend on the authored construction system; a universal “cost per extra floor” would conceal important differences.

### 3.2 Documented storey counts and form examples

| Context | Documented form | What it establishes | Confidence |
| --- | --- | --- | --- |
| Angkor’s ordinary residential landscape | A principal **raised living storey** | Large urban systems need not be vertically intensive | **M**, historical synthesis. [Frontiers](https://www.frontiersin.org/journals/human-dynamics/articles/10.3389/fhumd.2024.1347157/full) |
| Traditional Suzhou housing | Commonly **two storeys** in the cited comparison | Low-rise urban traditions can persist in developed commercial settings | **M**. [Frontiers](https://www.frontiersin.org/journals/human-dynamics/articles/10.3389/fhumd.2024.1347157/full) |
| Shibam, Yemen | Mud-brick houses **up to seven storeys** | Premodern tower housing is feasible | **H** for the documented form, not its universal prevalence. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/192?utm_source=chatgpt.com) |
| Nineteenth-century New York tenements | **Four to five storeys** in the museum’s historical account; some buildings covered up to **87% of their lots** | Coverage and occupancy, not skyscrapers, generated extreme crowding | **M–H**. [The Skyscraper Museum](https://www.skyscraper.org/housing-density/history/) |
| Fujian tulou, China | Multistorey communal buildings, some accommodating **up to 800 people** | A building can house many domestic groups around shared space | **H** for documented examples. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1113?utm_source=chatgpt.com) |

For initial content authoring, **one-storey dwellings, one-to-three-storey vernacular blocks, three-to-six-storey walk-ups, and special tower-house traditions** are useful form families. These are **proposed authoring ranges**, not historical era limits. Structural capability, cost, and demand should decide which appear.

### 3.3 How the dominant mechanisms vary over time

**Foragers:** mobility and seasonal aggregation matter alongside dwelling layout. Social groups can rearrange or split instead of permanently adding buildings. More people need not produce a proportionally tighter camp; documented camps often introduce additional spacing between household clusters as they grow. [ResearchGate](https://www.researchgate.net/publication/359403463_Scaling_of_Hunter-Gatherer_Camp_Size_and_Human_Sociality)

**Early farming:** the crucial choice is often residence near fields versus residence near other households. Agricultural technology is compatible with both dispersed settlement and compact villages; the Chinese and American cases above illustrate the variation. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0252532)

**Preindustrial urbanism:** aggregation can support dense quarters, but extensive agrarian urban landscapes also existed. Angkor should not be treated as an unusually unsuccessful attempt to become a compact European town. Low-density urbanism is also documented archaeologically in the Bolivian Amazon. [PubMed](https://pubmed.ncbi.nlm.nih.gov/33962951/)

**Industrial urbanization:** TCE should allow population inflows to outrun construction and services. Subdivision, lodgers, and high coverage can generate severe crowding in buildings only a few storeys tall, as the New York evidence demonstrates. [The Skyscraper Museum](https://www.skyscraper.org/housing-density/history/)

**Modern development:** improved construction makes height available, while improved transport makes dispersion available. Institutions, infrastructure, and household resources determine which combination occurs. Technology therefore expands the range of possible densities rather than moving every settlement along one sequence. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.34.3.3)

---

## 4. Overcrowding: measures, consequences, and hazard modeling

### 4.1 Measure crowding at the household and room level

A city’s average m²/person hides unequal allocation. An affluent household occupying a large compound does not relieve the crowding of neighbouring households.

Useful variables are:

| Measure | Interpretation | Calibration status |
| --- | --- | --- |
| Occupied usable m²/person | Available domestic space after excluding structural and circulation areas | Physical measure; no universal comfort threshold |
| Persons per habitable room | Coarse occupancy measure | HUD’s comparative work discusses **more than one person/room** as a commonly used crowding measure |
| Persons per bedroom | Sleeping concentration | **More than two persons/bedroom** is another measure examined by HUD |
| Unrelated domestic groups sharing a dwelling | Potential loss of autonomy and privacy | Meaning depends on household organization and norms |
| Ventilated room volume per sleeper and exposure duration | More relevant to respiratory exposure than city density alone | Requires building- and pathogen-specific modeling |

The room-based definitions are **administrative measurement conventions**, not biological discontinuities. A room must also be defined consistently: bedroom-only measures and all-habitable-room measures cannot be compared directly. [HUD User](https://www.huduser.gov/publications/pdf/measuring_overcrowding_in_hsg.pdf)

### 4.2 Health effects: separate transmission routes

WHO’s housing review finds substantial evidence linking household crowding with infectious disease, including tuberculosis and diarrhoeal disease. Evidence varies by outcome, and crowding is entangled with poverty and housing quality. The review does **not** supply one universal disease multiplier per additional person per hectare. [NCBI](https://www.ncbi.nlm.nih.gov/sites/books/NBK535289/)

For TCE, use three distinct mechanisms:

**Indoor respiratory transmission.** Accumulate exposure among people sharing rooms, workplaces, vehicles, or other enclosed spaces. A simple implementation can use

\[
P(\text{infection})=1-\exp(-\text{accumulated infectious dose}),
\]

with dose governed by infectious occupants, time, room volume, ventilation, and pathogen characteristics. This is a proposed abstraction; calibrate its coefficients for each disease rather than deriving them from settlement density.

**Water- and waste-mediated transmission.** Track contamination of shared sources and failures of waste separation. Historical Massachusetts evidence found that effective water and sewerage infrastructure were complementary contributors to falling child mortality. Dense settlement should therefore be capable of becoming substantially healthier without first becoming less dense. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/700766?utm_source=chatgpt.com)

**Vector and animal-associated transmission.** Use the relevant host, vector, habitat, storage, and contact conditions. Do not substitute crowded housing for all of these mechanisms.

For psychological effects, model privacy, unwanted interaction, noise, heat, and inability to withdraw. WHO finds less certain evidence for several mental-health and sleep outcomes than for some infections. Avoid a universal rule that density causes aggression or crime. [NCBI](https://www.ncbi.nlm.nih.gov/sites/books/NBK535289/)

### 4.3 Fire: ignition and spread are separate processes

A useful TCE decomposition is:

\[
\text{fire loss}
=
\text{ignition occurrence}
\times
\text{spread opportunity}
\times
\text{failure of suppression and escape}.
\]

Ignition should depend on hearths, kilns, lamps, industrial processes, and behaviour. Spread should depend on building separation, combustible surfaces, openings, wind, moisture, and fire barriers. Suppression depends on detection, water, equipment, organization, and physical access.

Full-scale informal-settlement experiments demonstrate strong sensitivity to spacing. One study identified a **3.8 m critical separation under its particular test conditions**. That is useful as a calibration experiment—not as a universal firebreak width for timber, thatch, masonry, and every wind condition. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0379711218302960)

**Implementation consequence:** maintain a building-neighbour graph. Citywide population density may correlate with fire exposure, but should not directly decide whether flames jump between two buildings.

---

## 5. Density gradients: centre to edge, with important exceptions

### 5.1 Use the exponential gradient as a diagnostic

Clark’s classic formulation is:

\[
D(r)=D\_0e^{-kr},
\]

where \(r\) is distance from the centre and \(k\) is a fitted gradient, in inverse distance units. It is a useful descriptive benchmark, not a settlement-generation law. [OUP Academic](https://academic.oup.com/jrsssa/article/114/4/490/7094988)

The mechanism is straightforward: accessible locations are desirable, but housing space there is scarce. Households and activities make different trade-offs between accessibility and space. This does not require a modern monetary land market; political privilege and social allocation can mediate the competition. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416524000345?utm_source=chatgpt.com)

Teotihuacan provides a numerical historical example. Against its **51.4 persons/ha** aggregate density, the reconstructed local surface ranges from **about 10 to 148 persons/ha using a 125 m kernel bandwidth**. The pattern is broadly monocentric, with denser residential areas near the Avenue of the Dead and lower concentrations toward the periphery. These are local estimates, not an exact centre-versus-edge pair. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416524000345?utm_source=chatgpt.com)

### 5.2 Do not require every city to have a dense geometric centre

A correct simulation should permit a palace or ceremonial centre with few residents, several commercial centres, dense port approaches, and corridors following roads. Accessibility should be measured along the actual network and to actual destinations—not merely as distance from the settlement’s founding coordinate.

Institutional exceptions are important. Bertaud and Renaud documented a historically unusual outward-rising density pattern in Soviet Moscow, associated with centrally allocated land uses and peripheral housing development. This is evidence that allocation institutions can overturn the familiar gradient, not that all planned cities must do so. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0094119096910979)

For TCE, fit gradients after growth has occurred. Examine residential density, employment density, wealth, and floor area separately. The most important centre for one household may be a market; for another, fields, a dock, a temple, or a patron’s estate.

---

## 6. Recommended TCE representation and initial parameters

### 6.1 Core entities and decisions

Use individuals for occupancy and activity, but **households as the principal housing-decision unit**.

| Entity | Minimum useful state |
| --- | --- |
| Person | Household, home, sleeping location, daily destinations, income/access rights, health |
| Household | Members, resources, social ties, desired space/privacy, housing obligations |
| Dwelling | Usable area, rooms or room groups, ventilation class, facilities, actual occupants |
| Building | Footprint, residential floor area, storeys, construction system, condition, entrances, fire properties |
| Parcel | Boundaries, access, ownership/use rights, allowed uses, subdivision constraints |
| Institution | Allocation rules, building rules, enforcement resources, public works, protected land |
| Settlement | Derived footprint, service networks, local density distributions, transport accessibility |

On births, deaths, marriage, household splitting, migration, or loss of income, recalculate the affected household’s situation. Candidate responses should include **staying, sharing, taking lodgers, partitioning rooms, extending, adding a floor, building elsewhere, and moving**.

Choose among feasible candidates using generalized costs: resources, labour, travel, social access, security, privacy, and institutional permission. A household need not optimize globally or know every vacant dwelling.

Crucially, support several allocation systems. A family compound, landlord market, communal landholding, temple allocation, and state housing authority can all use the same buildings-and-occupants representation without being forced into the same rental-market algorithm.

### 6.2 Initial density envelopes

These are **proposed low-confidence calibration envelopes**, informed by the cases above. They are not measured era averages, hard caps, or values to sample independently of geometry.

| Configuration to support | Initial gross-density envelope | Intended use |
| --- | --- | --- |
| Dispersed farmstead landscape | **1–15 persons/ha** | Homes mixed with substantial cultivated land |
| Open farming village | **15–50 persons/ha** | Gardens, work areas, substantial household spacing |
| Compact village or small town | **50–200 persons/ha** | Greater coverage and smaller plots |
| Dense low-rise town or urban district | **150–400 persons/ha** | High residential land share, coverage, or occupancy |
| Highly crowded constrained district | **400–1,500+ persons/ha** | Exceptional stress case, not a normal citywide target |

Use the historical observations to test whether these configurations are achievable under appropriate conditions. **Do not make settlements converge toward the middle of these bands.** A settlement should remain dispersed when household choices and institutions favour dispersion.

For occupied usable space, a practical initial *sensitivity sweep* is **4, 8, 15, 30, and 50 m²/person**. These are test values spanning markedly different housing conditions, not universal happiness thresholds. Let household preferences, resources, climate, and sharing arrangements determine how space is evaluated.

For rendering, floor-to-floor heights such as **2.5–3.5 m** can serve as provisional authoring values where no typology-specific data exist. Mark them as content assumptions and replace them with architectural evidence as building families are developed.

### 6.3 Update frequency and simplification

For 10k–50k people, I recommend:

**Daily or activity-driven:** update presence, shared-space exposure, service demand, and routine household changes.

**Monthly or event-triggered:** evaluate housing stress and a limited set of relocation or modification candidates.

**Seasonal or project-driven:** process construction, demolition, subdivision, infrastructure expansion, and institutional land allocation.

**During fires:** use short event-specific simulation steps for the affected building cluster.

These are computational recommendations, not historical time constants. Cache network accessibility; update only affected parcels and households. Use spatial indexing for nearby alternatives and building adjacency rather than pairwise comparison of all agents.

The minimum viable model does not require structural finite-element analysis, computational fluid dynamics, or a global housing-market equilibrium. Authored construction constraints, room-level exposure approximations, and local decision rules are enough to begin. Keep the Rust simulation authoritative: rendering distance or level of detail must never change resident counts.

### 6.4 Existing models and games worth borrowing from

| Model or game | Useful precedent | Limitation for TCE |
| --- | --- | --- |
| **UrbanSim** | Explicit households, buildings, housing units, parcels, accessibility, and development decisions | Primarily designed around modern urban systems; replace allocation assumptions where appropriate. [UrbanSim Cloud](https://cloud.urbansim.com/docs/general/documentation/urbansim.html) |
| **Village Ecodynamics Project** | Household-based settlement decisions connected to agricultural landscapes and environmental resources | Better precedent for household settlement ecology than for detailed buildings, indoor crowding, or fire. [UM Impact](https://umimpact.umt.edu/en/publications/modelling-prehispanic-pueblo-societies-in-their-ecosystems/?utm_source=chatgpt.com) |
| **Small-scale agricultural specialization models** | Demonstrate how exchange and productive differentiation can affect aggregation | Results depend on model assumptions; not a universal law of urban origins. [JASSS](https://www.jasss.org/16/4/4.html?utm_source=chatgpt.com) |
| **Cities: Skylines II, documented Economy 2.0 design** | Housing affordability and household composition feeding into residential demand | Useful game-design precedent, not historical validation; avoid treating density categories as fundamental physical entities. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/dev-diary-economy-part-one) |

---

## 7. Stylized facts and validation tests

A successful TCE implementation should reproduce **conditional patterns**, not force every city to resemble a selected historical example.

| Pattern to reproduce | Concrete validation test |
| --- | --- |
| **Density and height can diverge** | Produce high-density low-rise housing and lower-density taller development by changing land allocation, coverage, occupancy, and space consumption—not by overriding population counts. |
| **Crowding can change without construction** | Add household members or lodgers to existing dwellings; city population and crowding rise while floor area remains unchanged. |
| **Small camps and large agrarian cities occupy different spatial systems** | Keep immediate residential concentration separate from the land supporting subsistence. |
| **Within-city variation is substantial** | Report local distributions rather than only means; the Teotihuacan reconstruction provides a comparison spanning roughly 10–148 persons/ha at its stated smoothing scale. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416524000345?utm_source=chatgpt.com) |
| **Growth includes infill and outward expansion** | Permit both simultaneously, with walls, tenure, and infrastructure changing their relative costs. Medieval European evidence supports this mixed process. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0162678) |
| **Institutions can override the usual gradient** | Reserve central land or restrict redevelopment and test whether peripheral concentration emerges without an explicit “reverse gradient” rule. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0094119096910979) |
| **Infrastructure can decouple concentration from mortality** | Hold density broadly constant while improving water and waste systems; disease outcomes should change through the relevant exposure pathways. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/700766?utm_source=chatgpt.com) |
| **Fire outcomes depend on configuration** | Hold population constant while changing building gaps, combustible materials, access, or suppression capacity; losses should respond. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0379711218302960) |

Also test **decline**: after depopulation, buildings and parcel boundaries should persist. Density should fall through vacancies and reduced occupancy before every empty structure disappears. This is an important consequence of the proposed stock-and-flow representation, not a special scripted historical event.

---

## 8. Source and dataset strategy

The strongest calibration programme combines archaeological settlement maps, historically reconstructed populations, modern building measurements, and household data. No single dataset adequately covers all four.

| Source or dataset | Best use | Important limitation |
| --- | --- | --- |
| **Lobo et al. 2022, “Scaling of Hunter-Gatherer Camp Size and Human Sociality”** | Camp populations, areas, household spacing, mobile versus more permanent settlement | Ethnographic analogues are not direct observations of prehistoric populations. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/pdf/10.1086/719234) |
| **Berrey, Drennan and Peterson 2021, “Local economies and household spacing in early chiefdom communities”** | Comparative agricultural settlement configurations | Small comparative sample; several densities depend on earlier reconstructions. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0252532) |
| **Cesaretti et al. 2016, “Population-Area Relationship for Medieval European Cities”** | **173 settlements around 1300**, with population-area supplements | Regional historical uncertainties remain; useful for European calibration, not global defaults. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0162678) |
| **Blumenfeld et al. 2024, Teotihuacan urban-structure study; Klassen et al. 2021, Angkor population model** | Contrasting compact and extensive urban systems | Preserve each study’s boundaries and reconstruction assumptions. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416524000345?utm_source=chatgpt.com) |
| **Atlas of Urban Expansion, 2016 edition** | Sample of **200 cities**, urban expansion and spatial structure over approximately 1990–2015 | Contemporary large-city sample; not a household-crowding dataset. [Lincoln Institute of Land Policy](https://www.lincolninst.edu/data/atlas-of-urban-expansion/) |
| **European Commission/JRC Global Human Settlement Layer** | Matched population and built-up layers for modern spatial validation | Population is spatially allocated from source data, not individually observed at every raster cell. In **GHS-POP R2023A**, 2025 and 2030 are projections. [Global Human Settlement](https://human-settlement.emergency.copernicus.eu/ghs_pop2023.php) |
| **Reba, Reitsma and Seto 2016, “Spatializing 6,000 years of global urbanization…”** | Historical city locations and population estimates | **Not automatically a density dataset:** compatible historical footprints must be supplied separately. [Nature](https://www.nature.com/articles/sdata201634) |
| **WHO Housing and Health Guidelines, 2018; HUD Measuring Overcrowding in Housing, 2007** | Crowding definitions, health evidence, measurement choices | Do not turn administrative thresholds into universal physiological rules. [NCBI](https://www.ncbi.nlm.nih.gov/sites/books/NBK535289/) |

For each calibration record, store a date or phase, population range, area range, boundary definition, residential share where known, estimation method, and contemporaneity assumptions. Preserve competing reconstructions rather than reducing them prematurely to one authoritative number.

**Bottom line:** build TCE around persistent parcels and buildings, changing households, uneven land rights, accessibility, and infrastructure. Let density emerge from those processes. Use historical density figures to test the resulting settlements, household measures to evaluate crowding, and physical contact and building networks to generate disease and fire. That architecture can produce both a dispersed agrarian urban landscape and a tightly packed commercial town without scripting either as an inevitable stage of civilization.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928fa-8c60-83ea-8cc1-37ac9b88ad01)
