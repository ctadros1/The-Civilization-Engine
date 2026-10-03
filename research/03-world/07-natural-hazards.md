# Natural hazards for The Civilization Engine

## Executive recommendation

**Use separate physical generators for the seven hazards, but one shared consequence system:**

\[
\text{event}
\rightarrow
\text{local intensity}
\rightarrow
\text{exposure and component failure}
\rightarrow
\text{injury, displacement, and production loss}
\rightarrow
\text{recovery and adaptation}.
\]

This is the essential separation used by catastrophe-risk systems such as CLIMADA: a hazardous event is not itself a disaster; consequences depend on what is exposed and how vulnerable it is. For TCE, the important extension is to make exposure, vulnerability, and recovery emerge from individual activity, construction, inventories, transport, and institutions. [GMD](https://gmd.copernicus.org/articles/12/3085/2019/)

The most consequential design choices are:

* **Generate spatially and temporally correlated events, not independent disaster rolls for settlements.**
* **Damage construction features, not “ancient” or “modern” buildings as undifferentiated categories.**
* **Let famine, epidemics, migration, and political reactions emerge through intermediate mechanisms rather than attaching them directly to disaster magnitude.**

There is no defensible universal table saying that a settlement experiences an earthquake every *x* years or loses *y* percent of its population to a flood. The report below therefore separates **measured physical quantities and case-specific estimates** from **proposed TCE initialization values**.

**Confidence notation:** **H** = well-established physical relationship or well-documented measurement; **M** = useful empirical estimate with substantial contextual variation; **L** = sparse, indirect, or historically uncertain evidence; **D** = proposed design prior, not an empirical estimate. Confidence applies to the stated quantity in its original context, not automatically to its transfer across regions or eras.

---

## 1. Mechanisms: implementable causal rules

### 1.1 A common event and consequence model

Represent each event with a source, start time, duration, geographic footprint, and several intensity fields. A cyclone, for example, should contain wind, rainfall, and coastal-water-level fields—not a single “strength” that independently causes three unrelated disasters.

For each exposed building, field, road, water source, and person, evaluate:

\[
\text{impact}\_{e,j}
=
f\!\left(
\text{intensity}\_{e,j},
\text{duration}\_{e,j},
\text{asset or person attributes}\_{j},
\text{protective actions}\_{j}
\right).
\]

For buildings, distinguish **physical damage**, **habitability**, **repair requirements**, and **collapse**. A flooded house may be structurally standing but unusable; a roofless workshop may retain machinery but stop production. This distinction follows the separation between intensity, damage, and exposed value in established risk models. [GMD](https://gmd.copernicus.org/articles/12/3085/2019/)

The proposed causal rules for each hazard are:

| Hazard | Implementable mechanism | Essential dependencies and secondary effects |
| --- | --- | --- |
| **Flood** | Rainfall, snowmelt, upstream inflow, or coastal surge produces water levels and flows. Damage depends on depth above floor, duration, velocity, debris, and foundation erosion. | Contaminated wells, spoiled stores, drowned livestock, bridge loss, delayed planting, and disrupted access. Defenses change the footprint and may fail; do not merely reduce damage by a flat percentage. |
| **Earthquake** | A fault event generates spatially varying shaking. Local ground conditions and construction determine damage. | Aftershocks strike already weakened structures; slope failure, fire, and coastal tsunami require their own physical conditions. Magnitude is not local shaking intensity. |
| **Storm** | A moving weather system produces wind and precipitation fields; coastal storms additionally interact with tide, bathymetry, and shoreline shape. | Roof loss increases rain damage; roads and communications fail; coastal water can contaminate agricultural land. Tropical cyclones, extratropical storms, and localized convective storms need different event footprints. |
| **Drought** | Persistent precipitation and evaporation anomalies deplete soil water, streams, reservoirs, and groundwater on different timescales. | Crop-stage-specific stress, reduced pasture, livestock sales or deaths, diminished transport, depleted seed stocks, and food-price increases. Drought is a persistent state, not an instantaneous attack. |
| **Wildfire** | Ignition encounters sufficiently dry, connected fuel. Spread responds to wind, slope, vegetation, and suppression. | Ember transport creates new ignitions ahead of the main front. Buildings can spread fire to other buildings. Evacuation routes and available water matter independently of burned area. |
| **Locusts** | Suitable rain and soil conditions permit breeding; crowding changes behavior; mobile swarms track winds and vegetation. | Damage follows the swarm’s feeding footprint. Reproduction and migration create regional outbreaks lasting far longer than one village encounter. |
| **Volcanism** | A particular volcanic system enters an eruptive episode with a style and magnitude. Generate separate ash, lava, pyroclastic-flow, and lahar footprints. | Rain remobilizes deposits into lahars after the eruption. Ash affects roofs, water, crops, and transport; large eruptions can also create distant climatic effects. |

The corresponding evidence bases are USGS/JRC flood research, USGS earthquake guidance, cyclone-impact studies, FAO crop and locust models, NIST fire reconstruction, and volcanic-impact databases. These support the mechanisms, but not a single globally interchangeable parameterization. [JRC Publications](https://publications.jrc.ec.europa.eu/repository/handle/JRC105688)

### 1.2 Keep primary and secondary mortality separate

A useful agent-level decomposition is:

\[
P(\text{death}) =
P(\text{direct fatal exposure})
+
P(\text{survive direct exposure})
P(\text{later excess death}\mid\text{post-event conditions}).
\]

The second term should use TCE’s existing nutrition, disease, shelter, and injury systems. Avoid adding an arbitrary “famine mortality” percentage to everyone in a drought footprint.

Likewise, **do not automatically spawn an epidemic after a disaster**. Post-disaster infection risk depends particularly on displacement, crowding, safe water, sanitation, underlying health, and the pathogens already circulating. Bodies of people killed by physical trauma are not, by themselves, a general epidemic generator. [CDC](https://wwwnc.cdc.gov/eid/article/13/1/06-0779_article)

### 1.3 Natural events and god tools should share consequences

A god-created storm should use the same wind, flood, evacuation, and repair systems as a naturally generated storm. Only its origin differs.

Store an explicit `origin = natural | intervention` flag. Otherwise, intervention events will contaminate the historical records agents or debugging tools use to estimate natural recurrence.

---

## 2. Parameters: recurrence, damage, casualties, and recovery

### 2.1 Frequency–magnitude distributions

#### Return periods are probabilities, not schedules

For annual exceedance probability \(p\), the conventional return period is \(T=1/p\). Under stationary, independent annual trials:

\[
P(\text{at least one exceedance in }n\text{ years})
=1-(1-p)^n.
\]

Thus a “100-year flood” has a **1% annual exceedance probability**, not a rule preventing another flood for 100 years. Calculated probabilities over a century are **63.4%** for a 100-year threshold and **18.1%** for a 500-year threshold. These calculations cease to describe the process adequately when climate, catchments, defenses, or exposure change. [USGS](https://www.usgs.gov/special-topics/water-science-school/science/floods-and-recurrence-intervals)

For a continuous-time Poisson process, use \(P(N\_{\Delta t}\ge1)=1-e^{-\lambda\Delta t}\). Annual probability and annual event rate are approximately, but not exactly, interchangeable for rare events.

| Hazard | Recommended frequency–magnitude representation | Quantitative anchors and interpretation | Confidence |
| --- | --- | --- | --- |
| **Floods** | Generate discharge from weather and hydrology; calibrate its upper tail against local return levels. For a reduced generator, fit annual maxima or threshold exceedances. | US practice includes **log-Pearson III** modeling of annual peak discharge. Useful calibration thresholds are **10%, 2%, 1%, and 0.2% annual exceedance**, corresponding to 10-, 50-, 100-, and 500-year return levels. These are calibration points, not globally fixed flood magnitudes. [USGS](https://www.usgs.gov/publications/guidelines-determining-flood-flow-frequency-bulletin-17c) | H for definitions; M for fitted tails |
| **Earthquakes** | Regional Gutenberg–Richter occurrence model, fault-specific maximum magnitude, and a separate shaking model. Add clustered aftershocks rather than drawing all events independently. | \(\log\_{10}N(M\ge m)=a-bm\), with **\(b\) often near 1**: roughly ten times fewer events for each magnitude increment. The activity parameter \(a\), magnitude ceiling, and catalog completeness must be regional. [USGS](https://www.usgs.gov/publications/capturing-uncertainty-seismicity-observations-earthquake-rate-estimates-implications) | H for broad scaling; M/L for local long-period rates |
| **Storms** | Seasonal track or weather-system occurrence, followed by intensity, size, speed, rainfall, and coastal-water fields. Fit separate families for tropical and nontropical storms. | NOAA’s **IBTrACS** supplies global tropical-cyclone tracks. Wind observations use differing averaging conventions; a one-minute sustained wind and a short gust are not interchangeable inputs. No global “cyclone every \(x\) years per coastal village” is defensible. [NCEI](https://www.ncei.noaa.gov/products/international-best-track-archive) | H for catalog observations; M for synthetic tails |
| **Droughts** | Persistent climate anomalies coupled to water storage; classify drought afterward by duration and accumulated deficit. | For a normally standardized SPI, thresholds **−1, −1.5, −2** have lower-tail probabilities of approximately **15.9%, 6.7%, 2.3%**. These are probabilities of index values, **not independent drought-event frequencies**. Overlapping accumulation windows are correlated. [Climate Data Guide](https://climatedataguide.ucar.edu/climate-data/standardized-precipitation-index-spi) | H for index mathematics; M for impacts |
| **Wildfires** | Ignition plus fuel–weather spread. If sampling final areas statistically, use a truncated or tapered heavy-tailed distribution calibrated by fire regime. | LANDFIRE distinguishes historical fire-return regimes of **≤35 years**, **35–200 years**, and **>200 years**, also separated by severity. These describe landscape fire regimes, not the interval between city-destroying fires. Empirical fire-area distributions can resemble truncated power laws, but exponents and cutoffs vary. [Google for Developers](https://developers.google.com/earth-engine/datasets/catalog/LANDFIRE_Fire_FRG_v1_2_0) | M; historical reconstructions are model-based |
| **Locusts** | State transitions among recession, outbreak, upsurge, and plague, driven by breeding habitat, population density, migration, and control. | FAO finds **no fixed periodicity** of Desert Locust plagues. Favorable conditions can permit about **20-fold multiplication per generation of roughly three months**; this is a favorable-condition potential, not an unconditional growth rate. [FAOHome](https://www.fao.org/locust-watch/resources/frequently-asked-questions-%28faqs%29-about-locusts/?utm_source=chatgpt.com) | H for nonperiodicity; M for growth potential |
| **Volcanic events** | Volcano-specific or volcano-class eruption rates, with a magnitude distribution conditioned on system type and eruptive state. | A Southeast Asian model estimated decadal probabilities of at least one event in **VEI classes 4, 5, 6, 7, 8** of approximately **near 100%, 60%, 15%, 1.2%, 0.1%**, respectively. These apply to the **whole region**, not one volcano; large-event estimates depend strongly on incomplete records and proxy classifications. [Springer](https://link.springer.com/article/10.1007/s00445-014-0893-8) | M for common classes; L for extreme tails |

**TCE rule:** attach recurrence parameters to a **catchment, fault system, storm region, fire regime, breeding region, or volcano**. A settlement receives events by intersecting their footprints.

Do not run a rainfall-driven flood model and independently roll “random floods” on top of it. The same applies to drought and wildfire weather: shared climate must create their correlations.

---

### 2.2 Building damage functions

#### A. Earthquake vulnerability: construction matters more than era

A practical prototype can use the European Macroseismic Scale, EMS-98, which combines local intensity with building vulnerability classes. Its construction mapping includes approximately:

| Construction archetype | Typical starting vulnerability class | Important TCE attributes |
| --- | --- | --- |
| Rubble stone or adobe masonry | **A**, most vulnerable | Wall cohesion, reinforcement, wall-to-roof ties |
| Ordinary unreinforced manufactured-brick masonry | **B** | Mortar, bonding, connections, openings |
| Massive dressed-stone masonry | **C**, with variation | Geometry, construction quality, floor and roof connections |
| Timber construction | **D**, with broad variation | Joinery, bracing, anchorage, roof mass |
| Confined or reinforced masonry | **D**, with variation | Confinement continuity and workmanship |

These are starting archetypes, not guarantees for every building of that material. The EMS construction chart explicitly allows ranges within types. [ICGC Web Pro](https://icgc-web-pro.s3.eu-central-1.amazonaws.com/produccio/s3fs-public/2025-03/sismo_estructures_segons_vulnerabilitat_ems98_en_f.png)

A compact intensity–damage calibration is:

| Vulnerability | EMS intensity VIII | EMS intensity IX |
| --- | --- | --- |
| **A** | Many buildings reach very heavy damage; a few are destroyed | Many are destroyed |
| **B** | Many reach heavy damage; a few very heavy damage | Many reach very heavy damage; a few are destroyed |
| **C** | Many reach moderate damage; a few heavy damage | Many reach heavy damage; a few very heavy damage |
| **D** | A few reach moderate damage | Many reach moderate damage; a few heavy damage |

Here, EMS uses approximate categories: **“few” ≈1–15%, “many” ≈15–55%, “most” ≈55–100%**. These are deliberately imprecise descriptive bands, not precise probability intervals. Damage grade 5 means destruction; grade 4 is not synonymous with total collapse. [ICGC](https://www.icgc.cat/en/European-Macroseismic-Scale-1998-EMS-98)

For a more continuous implementation, a **proposed** fragility wrapper is:

\[
P(DS\ge k\mid x)
=
\Phi\left(\frac{\ln x-\ln\theta\_k}{\beta\_k}\right),
\]

where \(x\) is a positive physical intensity such as spectral acceleration, \(\theta\_k\) the median threshold for damage state \(k\), and \(\beta\_k\) the dispersion. Calibrate those parameters rather than guessing a universal acceleration-to-damage conversion.

**Implementation detail:** assign persistent construction-quality variation when a building is built. Do not reroll its strength every simulation tick.

#### B. Flood damage: depth is useful, but insufficient

JRC’s global flood-damage database supplies normalized depth–damage functions. Two published residential curves illustrate the scale and variability:

| Water depth, m | JRC Asia curve | JRC South/Central America curve |
| --- | --- | --- |
| 0.5 | 0.33 | 0.49 |
| 1.0 | 0.49 | 0.71 |
| 2.0 | 0.72 | 0.95 |
| 3.0 | 0.87 | 0.98 |

These values are **fractions of each source curve’s maximum damage**, not probabilities of collapse and not necessarily fractions of the full replacement value of every building. The curves synthesize particular studies and asset assumptions; they do not establish intrinsic continental vulnerability. [JRC Publications](https://publications.jrc.ec.europa.eu/repository/bitstream/JRC105688/global_flood_depth-damage_functions__10042017.pdf)

For TCE, use these as aggregate calibration envelopes, then separate:

\[
L\_\text{flood}
=
L\_\text{structure}
+
L\_\text{contents}
+
L\_\text{stored food}
+
L\_\text{production interruption}.
\]

A proposed building model should distinguish raised versus ground-level floors, water-sensitive versus water-resistant walls, exposure duration, flow forces, and scour. Stored grain and workshops need their own damage functions rather than inheriting a residential one.

#### C. Storm wind: a useful aggregate curve, not a universal building curve

Eberenz and colleagues use an established tropical-cyclone impact-function family:

\[
f(V)=\frac{z^3}{1+z^3},
\qquad
z=\frac{\max(V-V\_0,0)}{V\_{50}-V\_0}.
\]

One reference parameterization uses **\(V\_0=25.7\ \mathrm{m/s}\)** and **\(V\_{50}=74.7\ \mathrm{m/s}\)**, with one-minute sustained wind at 10 m. Calculated damage fractions are approximately **10.9% at 50 m/s**, **42.5% at 70 m/s**, and **50% at 74.7 m/s**. [NHESS](https://nhess.copernicus.org/articles/21/393/2021/)

**Important limitation:** this is an aggregate economic-impact function. Regional calibration matters greatly, and reported cyclone losses can include effects for which wind is only a proxy. Do not add this complete loss estimate to separately calculated complete surge and flood losses. [NHESS](https://nhess.copernicus.org/articles/21/393/2021/?utm_source=chatgpt.com)

For individual TCE buildings, model roof anchorage, span, openings, wall connections, and debris exposure. The physical relationship \(q=\tfrac12\rho V^2\) explains why wind load rises rapidly: using \(\rho=1.2\ \mathrm{kg/m^3}\), dynamic pressure is about **0.54, 1.50, and 2.94 kPa at 30, 50, and 70 m/s**. These are calculated free-stream pressures, not final design loads.

#### D. Volcanic ash: use mass loading

| Parameter | Quantitative range | Confidence |
| --- | --- | --- |
| Dry deposited ash bulk density | **500–1,500 kg/m³** | H as a broad range |
| Wet deposited ash bulk density | **1,000–2,000 kg/m³** | H as a broad range |
| Load from 0.10 m wet ash | **0.98–1.96 kPa**, calculated | H |
| Load from 0.30 m wet ash | **2.94–5.89 kPa**, calculated | H |

Use \(q=\rho gh\). USGS notes that ash thicknesses exceeding approximately **100 mm**, and more commonly **300 mm**, can cause roof collapse, depending on roof characteristics and ash density. Wide-span, low-pitched roofs are particularly vulnerable; wetting can greatly increase loading. These are warning ranges, not universal collapse thresholds. [USGS Volcanoes](https://volcanoes.usgs.gov/volcanic_ash/density_hardness.html)

This creates a valuable cross-hazard interaction: a roof may survive dry ash and fail when rain arrives before cleaning.

#### E. Wildfire: ignition and burning, not a smooth universal damage percentage

For a building, distinguish exposure to direct flame, radiant heat, embers, and burning neighbors. A proposed time-consistent ignition model is:

\[
P(\text{ignition during }\Delta t)
=
1-\exp\left[-\int\_{\Delta t}r\_\text{ignition}(t)\,dt\right].
\]

The rate depends on combustible materials, roof and opening details, local fuel, moisture, and fire exposure. Once ignited, burning duration and suppression determine damage.

A critical empirical benchmark is the 2018 Camp Fire: NIST documented winds up to **22 m/s** and spotting up to **6.3 km**. A model in which fire spreads only to immediately adjacent vegetation cells cannot reproduce that event’s propagation mechanisms. [NIST](https://www.nist.gov/publications/case-study-camp-fire-fire-progression-timeline)

---

### 2.3 Crop and pasture loss

#### Drought

A useful reduced crop-water response is:

\[
1-\frac{Y\_a}{Y\_m}
=
K\_y\left(1-\frac{ET\_a}{ET\_m}\right),
\]

with yield \(Y\), evapotranspiration \(ET\), and a crop- and condition-specific response coefficient \(K\_y\).

A FAO compilation reports a whole-season maize coefficient of **0.74 under the particular sprinkler-irrigated conditions studied**. Under that fitted relationship, a 25% evapotranspiration deficit implies an approximately **18.5% yield reduction**. This is an example calibration, not a universal maize constant; growth stage and management matter. [FAOHome](https://www.fao.org/4/y3655e/y3655e03.htm)

At a much larger scale, Lesk, Rowhani, and Ramankutty found that drought and extreme-heat disasters reduced national cereal production by roughly **9–10%** on average in their 1964–2007 analysis. Local fields can be devastated even when national losses are much smaller, so do not use a national mean as a village damage ceiling. [Nature](https://www.nature.com/articles/nature16467)

**Recommended implementation:** compute water stress during crop development, not only at harvest. Preserve seed requirements, planting windows, irrigation access, and the possibility of abandoning a field.

#### Locusts

| Parameter | Empirical range or value | Confidence |
| --- | --- | --- |
| Dense Desert Locust swarm population | **40–80 million adults/km²** | M |
| Adult fresh-food consumption | Approximately **2 g/adult/day** | M |
| Dense-swarm consumption, calculated | Approximately **80–160 tonnes of fresh vegetation/km²/day** | M |
| Daily movement | Can reach approximately **150 km/day** | M |

These figures come from FAO guidance. Consumption is **fresh vegetation mass**, not grain-equivalent calories. The commonly quoted “food for 35,000 people” comparison should not become a calorie conversion in the simulation. [FAOHome](https://www.fao.org/clcpro/faq/en?utm_source=chatgpt.com)

A suitable proposed mass-balance rule is:

\[
B\_\text{consumed}
=
\min\left(
B\_\text{accessible},
N\_\text{locusts}\,c\,\Delta t
\right).
\]

Allocate feeding among crops, pasture, and wild vegetation. Translate damaged leaves, growing points, flowers, and grain into subsequent yield through crop state. An encounter may cause anything from negligible loss to destruction of the current harvest, but **complete harvest loss must emerge from exposure and feeding**, not from the presence of one swarm marker.

---

### 2.4 Casualty and economic-loss anchors

The following are **case-specific observations or model outputs**, not globally representative fatality rates.

| Hazard | Quantitative impact anchor | Correct use in TCE |
| --- | --- | --- |
| **Flood** | A study of New Orleans flooding after Katrina estimated mortality around **1% of the exposed population**. Depth, proximity to breaches, and population vulnerability mattered. | A severe-event calibration case, not “all floods kill 1%.” The denominator is exposed people, not the entire city. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1539-6924.2008.01190.x) |
| **Earthquake** | The Jaiswal–Wald empirical model’s India example gives approximately **0.019% mortality at MMI VII** versus **4% at MMI IX**. | Demonstrates highly nonlinear intensity response. These are fitted population-exposure rates, not death probabilities conditional on building collapse, and not timeless national traits. [USGS Earthquake Hazards](https://earthquake.usgs.gov/static/lfs/data/pager/Jaiswal_%26_Wald_%282010%29_Empirical_Fatality_Model.pdf) |
| **Storm** | A Bangladesh study reports approximately **139,000 deaths in the 1991 cyclone**, versus approximately **4,200 in Cyclone Sidr in 2007**. Warning, shelters, and preparedness improved substantially. | Mortality can change greatly without eliminating the physical hazard. These were different storms, so the comparison is not a controlled estimate of one intervention. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3302549/) |
| **Storm—economic loss** | The estimated economic impact of Cyclone Pam in Vanuatu in 2015 was equivalent to approximately **64% of annual GDP**. | A whole-island economy can lose an enormous amount relative to annual output. This does **not** mean GDP itself fell by 64%. [DFAT](https://www.foreignminister.gov.au/minister/julie-bishop/media-release/further-support-vanuatu) |
| **Drought** | Approximately **9–10% national cereal-production reduction** in the disaster sample discussed above. | Food mortality must depend on reserves, distribution, purchasing power, imports, livestock, and subsequent harvests—not directly on this percentage. [Nature](https://www.nature.com/articles/nature16467) |
| **Wildfire** | The Camp Fire caused **85 fatalities** and destroyed **more than 18,000 structures**. | Structural destruction and mortality are related but distinct outcomes; evacuation changes the latter without necessarily saving buildings. [NIST](https://www.nist.gov/publications/case-study-camp-fire-fire-progression-timeline) |
| **Locusts** | No defensible general “deaths per swarm” coefficient. FAO describes damage principally through crops, pasture, livelihoods, and food security. | Treat as a production and food-access shock, not a direct mass-casualty attack. [FAOHome](https://www.fao.org/newsroom/story/eLocust3-solving-an-age-old-problem-with-new-technology/?utm_source=chatgpt.com) |
| **Volcanism** | The 1985 Nevado del Ruiz lahars caused roughly **25,000 deaths** across affected settlements; Armero was approximately **46 km from the volcano**. | A simple short-radius volcano damage circle misses deadly valley-following hazards. [SpringerLink](https://appliedvolc.biomedcentral.com/articles/10.1186/s13617-017-0067-4) |

**Economic bookkeeping should distinguish three quantities:**

1. **Destroyed stocks:** buildings, grain, livestock, tools, stored materials.
2. **Foregone flows:** missed harvests, idle workshops, interrupted trade, lost labor.
3. **Recovery expenditures:** labor and materials used to rebuild.

Do not count the replacement cost of a bridge, the actual rebuilding expenditure, and all lost trade as three interchangeable measures of destroyed wealth. In early TCE worlds, report physical losses and labor requirements before converting them into whatever prices the economy happens to have.

---

### 2.5 Recovery: multiple clocks, not a settlement-wide timer

Recovery research distinguishes emergency response, restoration, reconstruction, and longer-term change. Population recovery may remain incomplete even after substantial rebuilding; displacement and return are socially uneven. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0605726103?utm_source=chatgpt.com)

The literature does **not** support one transferable recovery-time table spanning ancient villages and modern cities. The following are therefore explicitly **D: proposed initialization and stress-test envelopes**, to be replaced by the outcomes of TCE’s labor, material, crop, and transport systems.

| Recovery process | Proposed test envelope | Conditions that should lengthen or prevent recovery |
| --- | --- | --- |
| Emergency shelter and basic local access | **1–8 weeks** | Continuing hazard, severe injury, blocked roads, scarce timber or fabric |
| Repair of repairable dwellings and workshops | **1–12 months** | Labor shortage, unavailable materials, seasonal work conflicts |
| Major rebuilding of housing stock | **3–36 months** | Widespread destruction, poor transport, depleted household resources |
| Major irrigation, defensive, or transport works | **1–10 years** | Coordination failure, fiscal shortage, loss of specialists |
| Annual-crop production after a one-off shock | **Next successful crop cycle; test 3–12 months** | Missed planting window, no seed, damaged irrigation, repeated drought or locust invasion |
| Herds and perennial productive assets | **Test 3–15 years** | Breeding constraints, immature replacements, lack of fodder or investment |
| Population and institutional recovery | **No mandatory completion time** | Permanent migration, land loss, changed trade routes, political fragmentation |

Two empirically grounded persistence rules are especially important. Desert Locust plagues can span many years rather than one growing season, and volcanic deposits can generate secondary lahars years after an eruption. Recovery may therefore be interrupted by the continuation of the same regional episode. [FAOHome](https://www.fao.org/locust-watch/resources/frequently-asked-questions-%28faqs%29-about-locusts/?utm_source=chatgpt.com)

A proposed repair rule is:

\[
\text{daily repair progress}
=
\min(
\text{available skilled work},
\text{delivered materials},
\text{site access},
\text{organizational capacity}
),
\]

with each term expressed in equivalent units of completed work. Recovery time then becomes an output rather than a hazard-specific constant.

---

## 3. Variation across eras and world regions

### 3.1 Model capabilities rather than chronological vulnerability multipliers

The following is a **capability-based modeling framework**, not a claim that every society passed through the same stages.

| Social/technological setting | Exposure and vulnerability to represent | Available responses to represent |
| --- | --- | --- |
| **Mobile foragers** | Fewer immovable assets may reduce rebuilding burdens, but local destruction of water and food resources can make an area unusable. Mobility helps only when destinations are reachable and accessible. | Moving camps, resource substitution, territorial knowledge, kin-based refuge, exchange networks |
| **Early farming** | Fixed fields, seasonal harvests, seed stocks, livestock, and dwellings concentrate risk. Losing one storehouse may be more consequential than losing several huts. | Dispersed stores, mixed crops, relocation, reciprocal support, small irrigation and drainage works |
| **Pre-industrial towns and states** | Dense construction, specialized occupations, dependence on canals, bridges, roads, ports, and grain markets | Public granaries, maintenance obligations, organized labor, tax relief, reconstruction, rationing, migration, trade |
| **Industrial societies** | More productive construction and transport, but greater dependence on concentrated networks and specialized equipment | Rail or maritime imports, mechanized repairs, organized firefighting, engineered defenses, insurance |
| **Modern societies** | Strong potential protection, alongside costly exposed assets and network cascades | Forecasting, evacuation, building codes, public health, engineered redundancy, large-scale finance and logistics |

Two historical findings are especially useful for avoiding simplistic disaster scripts.

**Persistence is common alongside disruption.** Archaeological research in the Willaumez Peninsula of Papua New Guinea documents repeated occupation and substantial cultural continuity across a long sequence of volcanic disturbances. Torrence argues that exchange relationships may have helped affected populations find refuge. The broad evidence supports resilience and repeated return; specific claims that an eruption caused a particular cultural innovation remain interpretations rather than universal causal rules. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S1040618214002535?utm_source=chatgpt.com)

**Distant hazards can interact with institutions.** Manning and colleagues link volcanic forcing, suppressed Nile summer flooding, and episodes of unrest and political change in Ptolemaic Egypt. This supports a pathway from external climate shock through harvest conditions and social pressures—not a rule that a bad flood automatically causes revolt or state collapse. [Nature](https://www.nature.com/articles/s41467-017-00957-y)

For later societies, Bangladesh provides a particularly clear counterexample to “hazard magnitude determines deaths”: warning and shelter institutions substantially changed outcomes without removing cyclone exposure. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3302549/)

### 3.2 Geographic clustering

Use geography to determine which source processes exist and how their footprints propagate.

| Geographic setting | Hazard combinations to emphasize | Procedural implication |
| --- | --- | --- |
| **Subduction margins and volcanic arcs** | Earthquakes, volcanic events, landslides, coastal tsunami | Generate tectonic source systems, not arbitrary dangerous mountains. Volcanism and earthquake exposure overlap imperfectly. |
| **Large floodplains and deltas** | River floods, waterlogging, coastal surge where relevant | Trace upstream catchments and low-lying connected terrain. Multiple settlements can fail together. |
| **Tropical-cyclone coasts and islands** | Wind, heavy rain, surge, landslides, trade interruption | Storm tracks affect regions; small islands may have little undamaged hinterland. |
| **Semi-arid breeding regions in Africa, Arabia, and southwest Asia** | Drought and Desert Locust episodes | Couple occasional favorable breeding rains to later mobile crop hazards; do not equate locust risk with the driest moment. |
| **Seasonally dry, fuel-connected landscapes** | Drought–fire combinations | Burnability requires both fuel and drying. An extremely arid, sparsely vegetated cell is not automatically the most fire-prone. |
| **Valleys below volcanoes** | Lahars and sediment-laden floods | Route hazards down drainage networks, including areas far beyond the near-vent zone. |
| **Downwind volcanic regions** | Ashfall and, for some large eruptions, climatic effects | Sources can lie outside the playable map. Wind direction matters more than simple radial distance for ash. |

Relevant calibration sources include Smithsonian GVP for volcanic systems, IBTrACS for tropical-cyclone tracks, FAO for species-specific locust ranges, and fire-regime research distinguishing climatic and fuel controls. **Do not place Desert Locust ecology everywhere:** Australia, southern Africa, and other regions have different locust species and outbreak ecologies. [Smithsonian Volcanoes](https://volcano.si.edu/)

For a settlement-scale world, a lightweight **off-map regional environment** is indispensable. Storms, droughts, ash, upstream floods, and locusts should not originate only inside the camera-accessible terrain.

---

## 4. Stylized facts a correct simulation should reproduce

These are validation targets derived from the evidence above, not instructions to replay particular disasters.

| Pattern | Testable implication for TCE |
| --- | --- |
| **Rare does not mean regularly spaced.** | A stationary 1%-annual-exceedance threshold is crossed in approximately 63% of 100-year trials, with some trials containing multiple exceedances and others none. |
| **Magnitude and damage are different quantities.** | The same earthquake magnitude produces different settlement losses with distance, ground conditions, and construction. |
| **Damage rises nonlinearly.** | Increasing shaking, wind, or ash loading through structural thresholds can produce a much larger change in damage than the same intensity increment below those thresholds. |
| **Evacuation can save lives without saving assets.** | A warning-and-shelter intervention should be able to reduce deaths while leaving substantial building and crop losses. |
| **Protection can change future exposure.** | Defended floodplain development may reduce frequent losses while increasing assets exposed to rare failures. Socio-hydrological models reproduce this feedback. [HESS](https://hess.copernicus.org/articles/17/3295/2013/) |
| **Local agricultural devastation can coexist with modest regional averages.** | Do not force every field to lose the same fraction, or interpret national 9–10% production effects as a village-level cap. |
| **Hazards are spatially and temporally connected.** | Neighboring settlements share droughts; storms combine wind and flood; rain can follow ash deposition; aftershocks encounter damaged buildings. |
| **Secondary hazards can outlast and outrange the initial event.** | Valley settlements tens of kilometers from a volcano remain vulnerable to lahars, including after the eruption ends. |
| **Recovery is unequal and sometimes incomplete.** | Some households repair quickly, some return later, some move permanently; rebuilding a capital stock need not restore the previous population or institutions. |
| **Disaster does not imply cultural collapse.** | Refuge, exchange, adaptation, rebuilding, and institutional continuity must remain possible outcomes alongside abandonment and fragmentation. |

A particularly valuable falsification test is to simulate the **same physical event against several counterfactual settlements**: identical population, but different roof connections, grain-storage arrangements, evacuation access, and public works. The model should explain the differences through those mechanisms, not an opaque “resilience score.”

---

## 5. Modeling recommendation for TCE

### 5.1 State to retain

At the **regional level**, retain climate persistence, catchment wetness, fault activity, vegetation fuel and moisture, volcanic state and deposits, and locust populations.

At the **building and infrastructure level**, retain material and connection properties, elevation, roof geometry, occupancy capacity, maintenance, existing damage, and repair requirements.

At the **household level**, retain accessible food, seed, livestock, tools, shelter alternatives, debts or obligations, and refuge relationships.

At the **individual level**, reuse location, mobility, health, occupation, dependents, knowledge, perceived risk, and current activity. Hazard exposure should normally be read from shared spatial fields, not calculated by an expensive independent physical simulation for every person.

At the **institutional level**, represent concrete capacities: grain reserves, treasury, mobilizable labor, transport, warning coverage, shelter places, medical capacity, and the ability to enforce or maintain construction and land-use rules.

### 5.2 Use two damage representations together

Use **component damage** for simulation: destroyed roof, cracked wall, spoiled grain, broken bridge span, blocked canal.

Use **aggregate loss functions** for calibration and fast previews: expected damage by intensity and building type.

Do not confuse the two. Aggregate curves are useful for checking whether a thousand-building district is implausibly invulnerable; component states are what create TCE’s visible reconstruction, shortages, and household choices.

For successive hazards, apply damage to the **remaining asset and its current condition**. A house should not lose 80% of its original value independently to wind and then another 80% to flood without an explicit combined-damage interpretation.

### 5.3 Multi-rate execution

The following are proposed engineering choices, not measured performance benchmarks:

| Process | Suggested resolution strategy |
| --- | --- |
| Regional event occurrence | Event queue or coarse periodic updates; sample exact event times |
| Weather, crop water, fuel moisture | Daily baseline, with sub-daily forcing during important episodes |
| Active flood, fire, evacuation | Temporarily refine affected areas and agents |
| Building damage | Evaluate intensity history or peak/duration summaries; update when thresholds or states change |
| Repair and food logistics | Daily work and inventory accounting |
| Institutional adaptation and demographic recovery | Weekly to seasonal decisions, with immediate emergency actions |

Use spatial indexing to query only affected assets. For distant or accelerated simulation, retain hazard summaries sufficient to reproduce exposure: maximum depth, time above thresholds, peak wind, ash accumulation and wetting, or fire-arrival time.

**Fast-forward must preserve opportunity to act.** A 12-hour warning cannot become zero warning merely because the simulation advanced in daily steps. Likewise, a persistent exposure must not become a fresh independent damage roll every time step.

### 5.4 Institutions should act through resources and incentives

A granary reduces famine only if food survives, can be transported, and is allocated. An evacuation system reduces casualties only if warnings arrive, people believe them, routes remain open, and shelter capacity exists.

Represent choices such as leaving with livestock, staying to protect possessions, accepting strangers, repairing a canal versus private houses, or withholding grain through ordinary agent and institutional decision systems.

Risk memory can be a simple decaying state—for example, an exponential memory with a **provisional 10–30-year timescale, D**, modified by personal experience, transmitted accounts, and institutional recordkeeping. Do not treat that range as a measured universal human constant.

Crucially, do not automatically award “centralization” after a flood or “rebellion” after a drought. Public works may strengthen coordination; failed relief may destroy legitimacy; migration may reduce the tax base. Those outcomes should follow the institutional model.

### 5.5 Existing models worth borrowing from

| Model or framework | Reusable contribution | What not to import uncritically |
| --- | --- | --- |
| **OpenQuake / GEM** | Separation of seismic sources, shaking, exposure, vulnerability, and risk; a strong reference for earthquake-model architecture. [OpenQuake](https://docs.openquake.org/oq-engine/latest/manual/) | Modern building inventories or locally calibrated vulnerability parameters as universal ancient values |
| **CLIMADA** | Event footprints, exposure data, impact functions, and probabilistic event-loss calculations. [GMD](https://gmd.copernicus.org/articles/12/3085/2019/) | Treating an aggregate economic-loss function as an agent-level injury or collapse model |
| **JRC flood-damage framework** | Inspectable depth–damage curves and asset-category distinctions. [JRC Publications](https://publications.jrc.ec.europa.eu/repository/handle/JRC105688) | Continental averages as inherent regional building properties |
| **FARSITE** | A reference for spatial fire-growth modeling; useful when designing a simplified wind–terrain–fuel spread system. [US Forest Service R&D](https://research.fs.usda.gov/treesearch/4617?utm_source=chatgpt.com) | Full operational-model complexity when only a reduced real-time approximation is needed |
| **FAO crop-water response models** | Water-deficit-to-yield relationships, with crop and management dependence. [FAOHome](https://www.fao.org/4/y3655e/y3655e03.htm) | One coefficient for every crop, growth stage, and farming system |
| **Di Baldassarre et al.’s socio-hydrological model** | Feedback between flood experience, defenses, settlement, and subsequent risk. [HESS](https://hess.copernicus.org/articles/17/3295/2013/) | Treating one stylized social-response equation as a complete theory of institutions |

### 5.6 Validation strategy

Validate in layers rather than tuning everything against final population counts.

First, generate long synthetic regional histories and check frequency–magnitude curves, seasonality, spatial clustering, and duration distributions. Second, expose fixed test settlements and check damage against the selected vulnerability envelopes. Third, activate evacuation, logistics, repair, and demographic response.

Use sensitivity ensembles for poorly known historical parameters. A model that matches one historical death toll only after choosing a particular uncertain evacuation rate is not independently validated.

Maintain separate tests for **physical hazard**, **direct damage**, **casualties**, **food access**, and **recovery**. Otherwise, an overly destructive hazard model can be accidentally “corrected” by implausibly generous relief.

---

## 6. Sources, calibration priorities, and thin evidence

### Core datasets and technical references

| Source | Best use |
| --- | --- |
| **USGS Bulletin 17C — England et al. (2018), *Guidelines for Determining Flood Flow Frequency*** | Flood-frequency fitting, historical and censored information, and uncertainty. [USGS](https://www.usgs.gov/publications/guidelines-determining-flood-flow-frequency-bulletin-17c) |
| **Huizinga, de Moel & Szewczyk (2017), JRC, *Global Flood Depth-Damage Functions*** | Quantitative flood-damage envelopes and asset definitions. [JRC Publications](https://publications.jrc.ec.europa.eu/repository/handle/JRC105688) |
| **EMS-98; Jaiswal & Wald (2010), *An Empirical Model for Global Earthquake Fatality Estimation*** | Construction vulnerability, intensity–damage patterns, and highly uncertain casualty estimation. [GFZ](https://www.gfz.de/en/EMS-98%20%20-%20%20European%20Macroseismic%20Scale%201998) |
| **NOAA IBTrACS; Eberenz et al. (2021), *Regional Tropical Cyclone Impact Functions*** | Cyclone tracks, intensity conventions, and regional calibration of economic impacts. [NCEI](https://www.ncei.noaa.gov/products/international-best-track-archive) |
| **LANDFIRE; Archibald et al. (2013), *Defining Pyromes*; NIST Camp Fire reconstruction** | Fire-regime diversity, landscape calibration, and a detailed settlement-impact case. [LANDFIRE](https://landfire.gov/fire-regime/frg) |
| **FAO crop-water guidance and Locust Watch; Lesk et al. (2016)** | Crop response, locust biology and geography, and aggregate agricultural-loss checks. [FAOHome](https://www.fao.org/4/y3655e/y3655e03.htm) |
| **Smithsonian Global Volcanism Program; Whelley et al. (2015); Brown et al. (2017)** | Volcano catalogs, regional eruption-frequency estimates, and fatality mechanisms and distances. [Smithsonian Volcanoes](https://volcano.si.edu/) |

### Where confidence is weakest

**Ancient casualty counts and reconstruction costs.** Surviving records are selective, population denominators are uncertain, and many losses were never recorded. Do not calibrate an ancient settlement to a precise fatality percentage merely because a chronicle gives a large round number.

**Transferring modern vulnerability backward.** Modern empirical curves embed occupancy, construction quality, contents, infrastructure, and response systems. The physical mechanisms transfer more reliably than the numerical loss ratios.

**Extreme tails.** Rare volcanic eruptions, the largest floods, and the largest earthquakes are precisely the events for which short historical records are least informative. Preserve parameter uncertainty rather than presenting one recurrence estimate as known. The Southeast Asian eruption study explicitly relies on incomplete histories and proxy information. [Springer](https://link.springer.com/article/10.1007/s00445-014-0893-8)

**Universal fire scaling.** Heavy-tailed burned-area distributions are useful, but neither one exponent nor one claim of self-organized criticality should govern every biome and human land-use regime. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0167278905003532)

**Environmental determinism.** Claims that a drought or eruption “caused civilization collapse” often compress a long chain of contested mechanisms. The Egyptian and Papua New Guinean cases support investigating those chains, not scripting their conclusions. [Nature](https://www.nature.com/articles/s41467-017-00957-y)

**Recovery times.** The proposed timing ranges in this report are test inputs, not universal historical estimates. The defensible model is resource-constrained recovery with the possibility of permanent change.

---

## Recommended first implementation

For TCE’s first complete version, implement **weather-linked floods and droughts, a fuel-based fire model, regional earthquake events with construction-class damage, and explicit household food and shelter losses**. Add storm tracks, locust population regimes, and volcano-specific processes through the same event interface.

Spend the most modeling effort on **who was exposed, what physically failed, what inventories survived, and who could obtain help**. Those mechanisms can produce an abandoned hamlet, a successfully evacuated town, a famine without much structural damage, or a rebuilt city with different institutions—without authoring any of those outcomes in advance.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927f3-88e0-83ea-88f7-fb8d55430f24)
