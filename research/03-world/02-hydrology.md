# Hydrology for The Civilization Engine

## Executive recommendation

**Use a coupled, mass-conserving model of catchments, river reaches, floodplain storage and shallow groundwater, with finer contaminant transport around settlements.** Do not simulate water molecules, and do not give each well an independent renewable supply.

For TCE, the essential relationships are:

**Weather → soil moisture and recharge → river discharge and groundwater levels → accessible water → human withdrawals and waste → downstream and groundwater consequences.**

I recommend preserving five features even in the first implementation: shared groundwater levels; finite well inflow and storage; floods constrained by terrain connectivity and water volume; contaminant travel time rather than a fixed pollution radius; and contamination through the wellhead and household storage, not only through the aquifer.

The report below separates **empirical measurements**, **engineering reference values** and **proposed TCE parameters**. The last category supplies implementation starting points, not claims about universal natural conditions. Historical well-depth evidence and pathogen attenuation are particularly unsuitable for a single global default.

---

# 1. Mechanisms: rules the simulation can implement

## 1.1 Climate must produce a water balance, not just a seasonal river multiplier

Represent precipitation as water entering a catchment. Partition it among interception, soil storage, evapotranspiration, rapid runoff, groundwater recharge and, where relevant, snow storage.

A useful invariant, expressed in volumes over an accounting interval, is:

\[
P+\text{water imports}
=
ET+\text{water exports}+\Delta S.
\]

Here, storage includes snow, soil, groundwater, channels, lakes and constructed reservoirs. Moving water from an aquifer into an irrigation canal is an internal transfer, not new supply.

A practical implementation can borrow the conceptual-store approach of rainfall–runoff models such as **GR4J**, whose four parameters describe production storage, an exchange term, routing storage and routing time. However, GR4J is not a physical groundwater-head model: adding a separate aquifer beneath it without revising the accounting can count groundwater contributions twice. [webGR](https://webgr.inrae.fr/eng/tools/hydrological-models/daily-hydrological-model-gr4j)

**Proposed TCE rules:**

* Rain exceeding the soil’s infiltration capacity produces rapid runoff; saturated soil can also produce runoff even under relatively gentle rainfall.
* Water remaining in the root zone supports vegetation and evaporates. Excess drainage recharges groundwater after a delay.
* Groundwater contributes slower river flow, allowing streams to persist between storms.
* Snow accumulates and melts separately from rainfall. Warm rain on existing snow can therefore produce a different event from the same rainfall over dry ground.
* Regional weather must have spatial coherence: upstream storms can flood a settlement that received little local rain.

Generate weather regimes with seasonality and persistence, rather than independent daily random rainfall. Fit these regimes against regional hydrographs and climate observations; the **Caravan** dataset supplies catchment attributes, meteorological forcing and discharge records for that purpose. [Nature](https://www.nature.com/articles/s41597-023-01975-w)

Daily hydrological updates are adequate for much of the annual water balance, but intense events require subdaily rainfall and routing. Otherwise a one-hour downpour and twenty-four hours of drizzle become incorrectly equivalent. Rapid rainfall-driven rises are central to flash flooding. [USGS](https://www.usgs.gov/mission-areas/water-resources/science/usgs-flood-information?page=1&qt-science_center_objects=0&utm_source=chatgpt.com)

### The upstream catchment can extend beyond the rendered world

A local map should receive explicit upstream boundary flows from a larger, cheaply simulated catchment graph.

For illustration, a **100 km² catchment** with **800 mm/year precipitation**, **500 mm/year evapotranspiration**, no net annual storage change and no other transfers produces approximately:

\[
\overline Q
=\frac{100\times10^6\times0.300}{31{,}557{,}600}
\approx0.95\ \mathrm{m^3/s}.
\]

This is a calculated example, not a regional observation. It prevents an important failure: generating a very large river from a small local rainfall budget.

---

## 1.2 River discharge becomes flooding through channel capacity and connectivity

Give each river reach a cross-section, bed elevation, roughness, water volume and connections to neighboring reaches.

For relatively steady, unobstructed flow, Manning’s equation provides a useful relationship:

\[
Q=\frac{1}{n}A\_c R\_h^{2/3}S\_f^{1/2},
\]

where \(A\_c\) is wetted cross-sectional area, \(R\_h\) hydraulic radius and \(S\_f\) friction slope. Roughness depends on channel material, vegetation and obstructions; official HEC-RAS reference values span substantially different conditions. [HEC](https://www.hec.usace.army.mil/confluence/rasdocs/ras1dtechref/6.6/basic-data-requirements/geometric-data/energy-loss-coefficients)

**Do not use this equation as the entire river solver.** Backwater from dams, downstream water levels and floodplain exchanges requires water-surface-aware routing. A local-inertial or similar simplified hydraulic scheme is more appropriate there; LISFLOOD-FP provides a relevant implementation reference. [GMD](https://gmd.copernicus.org/articles/18/9827/2025/)

For TCE, flood extent should emerge from:

\[
\text{inflow hydrograph}
+\text{channel capacity}
+\text{connected topography}
+\text{available storage}.
\]

Precompute elevation–area–volume curves for floodplain compartments. Water enters them through overtopped banks, channels, breaches or culverts, then drains through those same connections.

This avoids two common shortcuts: flooding everything within a radius of the river, and instantly filling every low cell regardless of whether water can reach it. Coupled channel–floodplain models explicitly represent such exchanges. [HEC](https://www.hec.usace.army.mil/confluence/rasdocs/r2dum/6.3/development-of-a-2d-or-combined-1d-2d-model/connecting-2d-flow-areas-to-1d-hydraulic-elements?scroll-versions%3Aversion-name=6.3)

**Proposed additional rules:** maintain distinct river flooding, rainfall ponding and groundwater emergence. Record maximum depth, duration and approximate velocity for damage calculations. Represent levees, bridge openings and culverts as hydraulic connections even when narrower than the flood grid. For a centuries-long simulation, update channel capacity occasionally from a coarse sediment budget rather than freezing river geometry forever.

### Return periods are probabilities, not event schedules

A “100-year flood” means **1% annual exceedance probability** under the specified statistical conditions. It does not mean a flood happens once every hundred years or that one cannot follow another immediately. USGS also emphasizes that estimated flood magnitudes change as records and conditions change. [USGS](https://www.usgs.gov/water-science-school/science/100-year-flood)

Under a stationary, independent-annual approximation:

\[
P(\text{at least one exceedance in }N\text{ years})
=1-(1-1/T)^N.
\]

Thus a 100-year threshold has a calculated **26.0% chance of exceedance in thirty years**, and **63.4% in a century**.

Generate floods from weather and routing; estimate return periods from the resulting annual maxima. Do not trigger a “century flood” timer.

---

## 1.3 Groundwater depth is an evolving hydraulic state

Store groundwater **head**, \(h\), rather than a fixed “groundwater depth” assigned by biome:

\[
d\_{\mathrm{water}}=z\_{\mathrm{ground}}-h.
\]

Groundwater behavior depends on recharge, aquifer geometry, permeability and boundaries. Groundwater divides need not coincide with surface drainage divides; a regional USGS model of Wisconsin’s Rock River basin illustrates why subsurface flow cannot simply follow the terrain’s river network. [USGS](https://www.usgs.gov/publications/simulation-regional-ground-water-flow-system-and-ground-watersurface-water-interaction)

For TCE, a useful shallow-aquifer approximation is:

\[
S\_{y,i}A\_i\frac{dh\_i}{dt}
=
R\_iA\_i
+\sum\_j C\_{ij}(h\_j-h\_i)
+q\_{\mathrm{river},i}
-W\_i
-ET\_{\mathrm{gw},i}A\_i.
\]

The terms are drainable storage, recharge, groundwater exchange, river exchange, pumping and groundwater evapotranspiration. Neighbor conductance can be approximated by:

\[
C\_{ij}=K\_{ij}\frac{b\_{ij}w\_{ij}}{\ell\_{ij}},
\]

using effective saturated thickness \(b\), interface width \(w\), separation \(\ell\) and hydraulic conductivity \(K\). This is a proposed reduced implementation of the control-volume approach used by groundwater models such as MODFLOW. Use SI units internally. [USGS](https://www.usgs.gov/software/modflow-6-usgs-modular-hydrologic-model)

Two distinctions matter:

**Specific yield is not porosity.** Specific yield measures water released by gravity drainage; some pore water remains retained. Effective porosity, used for transport velocity, is another parameter again. [The Groundwater Project](https://books.gw-project.org/hydrogeologic-properties-of-earth-materials-and-principles-of-groundwater-flow/chapter/specific-yield-and-specific-retention/)

**A river can gain or lose water.** Compare river stage with adjacent groundwater head, subject to riverbed conductance and available water. Pumping can capture water that would otherwise have reached the river, or induce additional river leakage. Effects can be delayed, so stopping pumping need not immediately restore streamflow. [USGS](https://www.usgs.gov/publications/streamflow-depletion-caused-groundwater-pumping-fundamental-research-priorities)

Generate connected geological units, not independently randomized soil cells. A sand lens over clay can support perched water; fractured rock can transmit water through narrow pathways. Karst should have optional conduit connections rather than merely an unusually large uniform conductivity. Published field transport evidence shows that fractures and karst behave differently from well-filtering matrix materials. [PubMed](https://pubmed.ncbi.nlm.nih.gov/19549931/)

---

## 1.4 Wells need construction depth, hydraulic inflow and stored water

A well should contain at least:

| Component | Simulation meaning |
| --- | --- |
| Shaft or bore depth and intake interval | Which water-bearing material the construction reaches |
| Shaft water volume | Water immediately available before further aquifer inflow |
| Hydraulic connection | Replenishment rate as groundwater and well water levels change |
| Lifting equipment | Labor, power, throughput and attainable operating depth |
| Construction integrity | Lining, cover, apron, drainage and routes for dirty surface water |

Large-diameter dug wells can store water during periods of low use and replenish between withdrawals. WaterAid’s technical guidance explicitly treats this storage function and the importance of protected upper construction. [WaterAid](https://www.wateraid.org/us/sites/g/files/jkxoof291/files/technical-brief-hand-dug-wells.pdf)

**Depth alone does not determine yield.** A deeper hole in poorly permeable material may add little supply. Conversely, a shallow well in productive alluvium may refill quickly.

For a homogeneous, unconfined aquifer under steady radial-flow assumptions, a useful diagnostic is:

\[
Q\approx
\frac{\pi K(H^2-h\_w^2)}
{\ln(r\_e/r\_w)}.
\]

Here \(H\) and \(h\_w\) are saturated thicknesses above the aquifer base away from and inside the well; \(r\_e\) and \(r\_w\) are influence and well radii. Use this as a subgrid approximation or test case—not a universal yield formula.

For an illustrative \(H=10\) m, \(h\_w=9\) m, \(r\_e=50\) m and \(r\_w=0.5\) m, the calculated hydraulic inflow is approximately **\(12.96K\) m³/day** when \(K\) is in m/day. Conductivities of 0.01, 1 and 10 m/day therefore produce radically different potential inflows.

**Potential inflow is not sustainable supply.** The shared aquifer, neighboring abstractions, captured streamflow and long-term recharge still constrain withdrawals. [USGS](https://www.usgs.gov/publications/streamflow-depletion-caused-groundwater-pumping-fundamental-research-priorities)

A separate storage example: a 1.2 m internal-diameter well containing a 2 m water column holds about **2.26 m³**. It can appear productive during a short collection period even when its sustained refill rate is modest.

---

## 1.5 Irrigation must compete with other water uses

Calculate crop demand from weather and growth stage:

\[
ET\_c=K\_cET\_0.
\]

Irrigation supplies the deficit remaining after effective rainfall and accessible root-zone moisture. FAO’s reference values vary with crop, development stage and climate; a single annual “water need” is insufficient for scheduling irrigation. [FAOHome](https://www.fao.org/4/s2022e/s2022e07.htm)

For illustration, **1 hectare receiving 5 mm/day requires 50 m³/day net**. A hypothetical community of 100 people using 20 liters/person/day uses 2 m³/day. Those household assumptions are illustrative, not an estimate of ancient consumption; the calculation demonstrates how quickly irrigation can dominate a settlement’s water budget.

Gross diversion should account for conveyance and field application:

\[
I\_{\mathrm{gross}}
=
\frac{I\_{\mathrm{net}}}
{\eta\_{\mathrm{conveyance}}\eta\_{\mathrm{application}}}.
\]

However, inefficient delivery does not mean every lost liter disappears. Canal seepage may recharge groundwater, and excess application may return downstream. FAO distinguishes these efficiencies; TCE should preserve the destination of each loss. [FAOHome](https://www.fao.org/4/t7202e/t7202e08.htm)

Track salt separately from water: evapotranspiration removes water while leaving salts behind; drainage and leaching export salts. Without adequate drainage, irrigation can create waterlogging and salinity rather than monotonically improving farmland. [FAOHome](https://www.fao.org/4/t0234e/t0234e03.htm)

---

## 1.6 Contamination needs multiple pathways and different substances

A single “pollution” scalar cannot adequately represent drinking-water risk, crop salinity and sewage effects.

For the first version, I recommend three separate families:

| Family | Minimal representation |
| --- | --- |
| Enteric pathogens | Viable organism counts or infectious-unit equivalents, with pathogen-specific survival |
| Dissolved nitrogen | Mobile nitrogen loading; add separate ammonium/nitrate pools when transformation chemistry matters |
| Salts | Conserved dissolved mass, removed through exports rather than microbial decay |

For pathogens, the decisive distinction is the **route to the consumer**:

**Pit → unsaturated soil → groundwater → well**,  
**pit or contaminated ground → surface runoff → river or wellhead**, and  
**water source → collection vessel → household storage**.

A two-year Bangladesh study found that contamination measured in the aquifer was less frequent than contamination associated with collected and stored water. This makes well construction and handling separate causal systems, not cosmetic upgrades. [ResearchGate](https://www.researchgate.net/publication/319928194_Accepted_Manuscript_The_public_health_significance_of_latrines_discharging_to_groundwater_used_for_drinking)

### Model travel time, not a safe circle

Groundwater pore-water velocity is approximately:

\[
v=\frac{Ki}{n\_e},
\qquad
t\_{\mathrm{travel}}\approx\frac{L}{v},
\]

where \(i\) is hydraulic gradient and \(n\_e\) effective porosity.

A calculated illustration with \(i=0.01\), \(n\_e=0.25\) and a 30 m path gives:

| Conductivity | Approximate velocity | Travel time |
| --- | --- | --- |
| 10 m/day | 0.4 m/day | 75 days |
| 100 m/day | 4 m/day | 7.5 days |

The same setback can therefore imply very different travel times even before fractures, dispersion, filtration or pumping are considered.

Transport contaminant mass along water fluxes. Apply pathogen inactivation over time and attachment/removal according to the traversed medium. **Do not combine an empirical total attenuation coefficient with a separate decay term unless the coefficient excludes that decay**, or removal will be counted twice.

Pang’s analysis of field and intact-core studies found relatively weak microbial removal in structured clay, coarse gravel, fractured rocks and karst compared with several finer or strongly weathered materials. In 26 of 87 analyzed datasets, removal rates decreased with distance rather than remaining constant. Thus “clay filters well” is unsafe shorthand when cracking or preferential flow is present. [PubMed](https://pubmed.ncbi.nlm.nih.gov/19549931/)

A review of pit-latrine studies reports substantial attenuation over short distances in some settings, but contamination detections farther away in others. Common **15–30 m setbacks** and **1.5–2 m vertical separation** from the water table are conditional screening guidance—not universal guarantees. [ResearchGate](https://www.researchgate.net/publication/236070782_Pit_Latrines_and_Their_Impacts_on_Groundwater_Quality_A_Systematic_Review)

Nitrate should not disappear at a pathogen mortality rate. Nor should boiling be represented as universal purification: WHO notes that it does not remove nitrate and can concentrate it as water evaporates. [WHO CDN](https://cdn.who.int/media/docs/default-source/wash-documents/water-safety-and-quality/chemical-fact-sheets-2022/nitrate-and-nitrite-fact-sheet-2022.pdf?sfvrsn=a65406e)

Finally, deeper groundwater is not inherently chemically safe. A study in the southwestern Bengal Basin documented arsenic-contaminated groundwater below 150 m and attributed it to a lithologically controlled deep-flow system. For TCE, geogenic hazards should be independent of nearby sewage. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2019GL084767?utm_source=chatgpt.com)

---

# 2. Quantitative parameters and confidence

**Confidence refers to applicability as well as measurement quality.** A reliable observation from one aquifer is not a reliable global default.

## 2.1 Hydrology and irrigation reference values

| Parameter | Value or range | Units | Interpretation and confidence |
| --- | --- | --- | --- |
| Reference evapotranspiration, humid conditions at 15–25°C | 3–4 | mm/day | FAO indicative climatic range; **medium**, not a substitute for weather-based calculation. |
| Reference evapotranspiration, arid conditions at 15–25°C | 7–8 | mm/day | FAO indicative range; **medium**. |
| Wheat seasonal crop water requirement | 450–650 | mm/season | Indicative crop evapotranspiration, not gross diversion; **medium**. |
| Grain maize crop coefficient | 0.40 initially; 1.15 mid-season | dimensionless | Illustrates growth-stage variation; **medium** transferability. |
| Field application efficiency | Surface 60%; sprinkler 75%; drip 90% | fraction or % | FAO indicative values; **medium**. These are not historical-era constants. |

Sources: FAO crop-water guidance and irrigation-efficiency guidance. [FAOHome](https://www.fao.org/4/s2022e/s2022e07.htm)

| Hydraulic parameter | Reference range | Units | Interpretation and confidence |
| --- | --- | --- | --- |
| Manning roughness: clean, straight natural channel | 0.025–0.033 | s·m\(^{-1/3}\), SI convention | Engineering starting range; **medium** until calibrated. |
| Clean, winding channel | 0.033–0.045 | same | **Medium**. |
| Weedy channel with pools/sluggish sections | 0.050–0.080 | same | **Medium**. |
| Short-grass floodplain | 0.025–0.035 | same | **Medium**; vegetation changes seasonally. |
| Dense summer brush on floodplain | 0.070–0.160 | same | **Medium-low** transferability. |
| Observed bankfull recurrence, Ohio study | 1.01–9.7; median 1.4 | years | Forty gauged sites; **high** for that study, **low** as a universal prior. |

Sources: HEC-RAS roughness tables; Sherwood and Huitger’s USGS bankfull study. The latter is a useful warning against making bankfull discharge universally equal to a 1.5-year flood. [HEC](https://www.hec.usace.army.mil/confluence/rasdocs/ras1dtechref/6.6/basic-data-requirements/geometric-data/energy-loss-coefficients)

## 2.2 Proposed geological priors—not measured universal ranges

Hydraulic conductivity spans many orders of magnitude, and similarly named materials can differ greatly. The following are deliberately broad **TCE starting priors**, selected within the much wider literature envelopes. Sample conductivity logarithmically and create spatially connected deposits. [The Groundwater Project](https://books.gw-project.org/hydrogeologic-properties-of-earth-materials-and-principles-of-groundwater-flow/chapter/hydraulic-conductivity-values-for-earth-materials/)

| Generated material | Proposed \(K\) | Proposed specific yield \(S\_y\) | Confidence as a TCE prior |
| --- | --- | --- | --- |
| Intact clay-rich matrix | \(10^{-7}\)–\(10^{-4}\) m/day | 0.01–0.05 | Low; add separate cracks where appropriate |
| Silty deposit | 0.001–0.1 m/day | 0.03–0.15 | Low–medium |
| Sand aquifer | 0.1–100 m/day | 0.15–0.30 | Medium as a broad starting envelope |
| Sandy gravel/open alluvium | 10–1,000 m/day | 0.15–0.30 | Medium-low; fines can change behavior substantially |
| Weathered/fractured bedrock | 0.001–10 m/day equivalent bulk value | 0.005–0.05 | Low; explicit connected fractures may be necessary |

The specific-yield priors are narrower modeling subsets of published compilations, not the full observed ranges. Fresh crystalline rock, weathered rock and fracture networks must not receive interchangeable storage values. [The Groundwater Project](https://books.gw-project.org/hydrogeologic-properties-of-earth-materials-and-principles-of-groundwater-flow/chapter/specific-yield-and-specific-retention/)

**Do not assign a comparable universal groundwater-depth table by terrain type.** Generate geology and aquifer boundaries, spin up the water balance, and let the depth emerge.

## 2.3 Wells: measurements, historical evidence and analogues

| Evidence | Quantitative anchor | What it supports—and does not |
| --- | --- | --- |
| WaterAid hand-dug-well guidance | Commonly 5 to more than 20 m deep; some exceed 30 m; an example finished diameter is 1.2 m | **Medium-high** as a contemporary low-technology analogue. Not an ancient depth distribution. [WaterAid](https://www.wateraid.org/us/sites/g/files/jkxoof291/files/technical-brief-hand-dug-wells.pdf) |
| South-central Chile hand-dug-well study | 49 wells: 1.48–10.1 m depth, mean 4.78 m | **High** for that sample; demonstrates strong local variation. [SciELO](https://www.scielo.cl/scielo.php?pid=S0718-58392014000200014&script=sci_arttext) |
| Same Chile study | Observed abstraction 0.02–3 m³/day, mean 0.46 m³/day | Actual use, **not maximum hydraulic yield**. [SciELO](https://www.scielo.cl/scielo.php?pid=S0718-58392014000200014&script=sci_arttext) |
| African groundwater assessment | Appropriately sited boreholes supplying 0.1–0.3 L/s feasible across many areas; opportunities above 5 L/s more restricted | Domestic handpump supply and irrigation potential are different resource questions. **Medium** regional screening confidence. [Nora](https://nora.nerc.ac.uk/id/eprint/17892/) |
| Prehistoric Yangtze coastal-plain wells | Most reported examples less than 3 m deep | **Medium** archaeological synthesis; not representative of all China. [ResearchGate](https://www.researchgate.net/publication/265014693_Wells_in_China) |
| Exceptional late-Neolithic Chinese construction | A reported timber-supported well approximately 11 m deep | Demonstrates capability, not typical depth or yield. **Medium**. [ResearchGate](https://www.researchgate.net/publication/265014693_Wells_in_China) |

## 2.4 Contamination parameters that deserve explicit uncertainty

| Quantity | Numerical anchor | Confidence and implementation |
| --- | --- | --- |
| Latrine–well separation | Guidance commonly around 15–30 m | **Low as a predictive cutoff**. Calculate pathways instead. |
| Pit bottom above seasonal maximum water table | Guidance often 1.5–2 m | Conditional unsaturated-zone protection, not guaranteed safety. |
| Reported microbial influence | Some studies show strong attenuation within 5–15 m; detections around 20–25 m also occur | Heterogeneous indicators, geology and designs; **low global transferability**. [ResearchGate](https://www.researchgate.net/publication/236070782_Pit_Latrines_and_Their_Impacts_on_Groundwater_Quality_A_Systematic_Review) |
| Nitrate drinking-water benchmark | 50 mg/L as nitrate ion, approximately 11.3 mg/L as nitrogen | **High** confidence in the WHO benchmark; not an ancient institutional standard or a binary biological threshold. [WHO CDN](https://cdn.who.int/media/docs/default-source/wash-documents/water-safety-and-quality/chemical-fact-sheets-2022/nitrate-and-nitrite-fact-sheet-2022.pdf?sfvrsn=a65406e) |
| Pathogen inactivation time, \(T\_{90}\) | No defensible universal value | Calibrate by organism, temperature and medium. Do not invent one “sewage half-life.” |
| Groundwater transport porosity | Use an independently authored parameter | Do not automatically substitute specific yield. |

The final two rows are deliberate omissions of false precision. A simulator can expose these parameters without pretending that one number is well established for every pathogen or aquifer.

---

# 3. Variation across eras and world regions

**Keep the physical laws stable; vary infrastructure, energy, knowledge, access and institutions.** These should be composable capabilities, not a globally synchronized technology ladder.

| Setting | Evidence or relevant pattern | Implication for TCE |
| --- | --- | --- |
| Foraging and mobile societies | Australian First Nations water knowledge includes locating and using dispersed water sources, including sources not obvious from visible streams. Official curriculum documentation draws on this recorded knowledge. [Australian Curriculum](https://www.australiancurriculum.edu.au/support-resources/background-information/science_teacher_background_information_AC9SFH01_E3) | Water knowledge can be social memory. Seasonal travel, access rights and knowledge loss matter; “no cities” does not mean “no water management.” |
| Early farming: central Europe and China | European timber wells dated 5469–5098 BCE show sophisticated woodworking. Chinese Neolithic wells exhibit different depths and support structures. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0051374) | Early communities can construct durable wells without metal machinery. Available timber, excavation stability and local water depth should govern feasibility. |
| Arid pre-industrial regions: Iran | Qanats intercept groundwater and convey it by gravity through underground tunnels; UNESCO documents associated maintenance and allocation institutions. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1506/) | Model a qanat as a groundwater drain plus a graded conveyance network, with excavation and maintenance costs—not an unusually powerful well. |
| Tropical seasonal cities: Maya lowlands | Research at Tikal identifies zeolite and quartz in the Corriental reservoir and interprets them as a filtration system. Ancient treatment performance is inferred, not experimentally measured. [Nature](https://www.nature.com/articles/s41598-020-75023-7?error=cookies_not_supported) | Reservoir storage and seasonal reliability can dominate even in a tropical setting. Permit filtration technologies without granting modern disinfection performance. |
| Irrigated Asian landscapes: Bali | Subak combines canals, tunnels, weirs, rice terraces and cooperative water institutions. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1194/) | Coordination can emerge through local organizations rather than requiring centralized state ownership. Represent shared schedules and maintenance obligations. |
| Low-technology wells in African and South American settings | Borehole potential varies strongly across Africa; Chilean dug-well depths and use vary locally. [Nora](https://nora.nerc.ac.uk/id/eprint/17892/) | “Village well” is not one hydraulic asset type. Distinguish reliable domestic supply from water-intensive agricultural development. |
| Industrializing settlements | Pumping links groundwater withdrawals to later stream depletion; urban drainage systems introduce pipes, outfalls and concentrated discharges. [USGS](https://www.usgs.gov/publications/streamflow-depletion-caused-groundwater-pumping-fundamental-research-priorities) | Mechanization changes withdrawal scale. Sewer construction can relocate pollution downstream unless treatment is also provided. |
| Modern intensive agriculture and supply | USGS observations beneath irrigated cropland distinguish higher nitrate in younger groundwater from older recharge, demonstrating land-use and travel-time effects. [USGS](https://www.usgs.gov/publications/using-age-tracers-and-decadal-sampling-discern-trends-nitrate-arsenic-and-uranium?utm_source=chatgpt.com) | Keep groundwater age and chemical legacy. Improved practices should not instantly erase contamination already in transit. |

### Regional differences should be generated explicitly

For a world generator, I would author combinations of climate, geology and infrastructure rather than continental stereotypes:

**Monsoonal alluvial plain:** concentrated seasonal recharge, flood exposure and shallow domestic wells.

**Arid mountain front:** episodic runoff, alluvial aquifers, springs or gravity interception, and strong competition over limited recharge.

**Humid fractured or karst terrain:** abundant rainfall but potentially rapid contaminant transport and uneven well productivity.

**Snow-fed basin:** water storage in snow, melt-season flows and downstream irrigation demand that may peak later.

These are proposed generative templates, not claims that every landscape in the named regions behaves alike. Validate them against appropriate regional observations.

---

# 4. Stylized facts a correct simulation should reproduce

These tests should combine exact accounting checks with qualitative patterns and selected empirical comparisons.

| Test world or intervention | Expected outcome |
| --- | --- |
| Add 50 mm of net recharge to an isolated aquifer with \(S\_y=0.20\), holding all other fluxes at zero | Calculated water-table rise of 0.25 m. This is a storage-equation unit test. |
| Pump several wells from one aquifer | Drawdown and competition emerge from shared head. Streamflow effects may lag pumping. [USGS](https://www.usgs.gov/publications/streamflow-depletion-caused-groundwater-pumping-fundamental-research-priorities) |
| Compare a broad dug well with a narrow borehole under weak recharge | Immediate stored supply differs from sustained inflow. A short pumping test must not create an infinite daily yield. [WaterAid](https://www.wateraid.org/us/sites/g/files/jkxoof291/files/technical-brief-hand-dug-wells.pdf) |
| Irrigate 100 ha at a 5 mm/day net deficit | Calculated net requirement of 5,000 m³/day, before delivery losses. Domestic collection capacity should not automatically satisfy it. |
| Repeat stationary 100-year-threshold flood trials | Approximately 26% of thirty-year runs contain at least one exceedance, given independent annual events. |
| Put equal pollutant loads equal distances from wells in different geological settings | Outcomes can differ substantially. An Indian comparison found markedly different sanitation effects in Kolkata alluvium and Indore’s basalt setting. [Springer](https://link.springer.com/article/10.1007/s10661-011-1965-2) |
| Repair a contaminated wellhead without changing the aquifer | Delivered-water quality can improve even though regional groundwater chemistry is unchanged. Keep the pathways separate. [ResearchGate](https://www.researchgate.net/publication/319928194_Accepted_Manuscript_The_public_health_significance_of_latrines_discharging_to_groundwater_used_for_drinking) |
| Improve household storage practices | Exposure can fall without relocating wells or latrines. Source quality and consumed-water quality are not identical. [ResearchGate](https://www.researchgate.net/publication/319928194_Accepted_Manuscript_The_public_health_significance_of_latrines_discharging_to_groundwater_used_for_drinking) |
| Increase pumping power without increasing recharge | More water can be extracted initially, but depletion and external effects increase; technology does not manufacture renewable water. [USGS](https://www.usgs.gov/publications/streamflow-depletion-caused-groundwater-pumping-fundamental-research-priorities) |

Additional TCE-specific regression cases should include a levee with a narrow breach, an upstream flood during local dry weather, a seasonally inundated latrine, canal seepage that benefits neighboring wells, and a pumping well that changes which contaminant plume it captures.

---

# 5. Modeling recommendation for a real-time Rust kernel

## 5.1 Use different resolutions for different questions

The following are **proposed engineering starting points**, not published TCE benchmarks.

| Subsystem | Representation | Starting spatial and temporal scale |
| --- | --- | --- |
| External catchments | Graph of rainfall–runoff stores | Catchment units roughly 0.1–10 km²; daily background updates, subdaily storm routing |
| Local soil and shallow aquifer | Conservative finite-volume grid | 64–128 m groundwater cells; hours to daily under ordinary conditions |
| Rivers | Reach graph with stored volume and hydraulic connections | Reaches roughly 100–500 m, split at important structures and junctions |
| Floodplains | Connected storage compartments or active 2D grid | 16–64 m cells, retaining finer terrain-derived storage and barriers |
| Settlement contamination | Local transport grid or mass-carrying travel-time packets | Approximately 5–20 m refinement, or subgrid paths through coarser hydraulic fields |
| Wells, pits, tanks, canals and fields | Persistent entities exchanging water and contaminant mass | Event-driven operations with aggregated hydraulic updates |

The distinction between flow and transport resolution is crucial. **A 64 m cell that instantly mixes a latrine and well cannot resolve a 10–30 m contamination question**, even if its regional groundwater head is accurate.

A practical option is coarse hydraulic heads plus local, mass-conserving contaminant trajectories. Use spatial indexing and capture zones; do not check every citizen against every pollution source.

For an illustrative **16 × 16 km** map, a 64 m grid contains **62,500 cells**, while 128 m gives **15,625**. At an assumed 128 bytes per cell, the former uses about **7.6 MiB of raw cell state**. This excludes solver workspaces, geometry, transport histories and entities; it says nothing by itself about runtime.

Flood hydraulics needs adaptive substeps. Explicit wave propagation is constrained approximately by:

\[
\Delta t \lesssim
\frac{\Delta x}{|u|+\sqrt{gd}}.
\]

LISFLOOD-FP’s range of simplified and fuller hydraulic solvers is useful precisely because solver choice changes cost and applicable flow conditions. Do not advance a fast flood by an hour merely because ordinary agent decisions use hourly scheduling. [GMD](https://gmd.copernicus.org/articles/18/9827/2025/)

## 5.2 Keep the coupling conservative

I recommend this update contract:

**Weather changes stores; infrastructure requests transfers; flow solvers approve physically available transfers; contaminant transport uses those exact water fluxes; agents receive the resulting quantities and concentrations.**

Requests should reserve or ration supply, rather than allowing several agents to consume the same remaining bucket of water. A well withdrawal must reduce well storage and ultimately the aquifer or its contributing boundary flows. A canal leak must enter another store or become a documented loss.

Keep water and contaminant budgets inspectable by subsystem and globally. Use double precision for accumulated volumes and mass accounting, nonnegative-store safeguards, and explicit records of treatment removal or chemical transformation.

For accelerated time, reduce spatial detail or aggregate quiet periods. Do not silently replenish groundwater when an area becomes inactive or unloads from the renderer.

## 5.3 Individual people should interact with water infrastructure, not solve hydrology

A person’s water decision can depend on travel time, queues, price, access rights, known reliability, taste, appearance and learned health associations.

The kernel knows actual concentration; the person has **imperfect beliefs**. This enables plausible behavior without giving early societies laboratory knowledge. A visibly clean source can remain hazardous, and abandoning a source can result from experience rather than a scripted scientific era.

Institutions can own or regulate wells, allocate irrigation turns, organize canal cleaning, inspect pits, maintain levees and pay for treatment. Track who benefits and who bears downstream effects. An upstream diversion and a neighboring pump become sources of conflict through physical consequences rather than an arbitrary diplomacy modifier.

Technologies should modify concrete capabilities: excavation support, intake protection, lift energy, storage losses, filtration, disinfection, measurement or coordination. Avoid upgrades that merely grant “+20% water” without identifying the additional source.

For observer legibility, retain causal explanations such as:

> “This well supplied less water because the aquifer fell below part of its intake after two dry seasons and increased upstream pumping.”

Or:

> “The river flood entered through the damaged well apron; the surrounding aquifer was not the main contamination route.”

## 5.4 Existing models and games: what to borrow

| Reference | Useful contribution | Boundary |
| --- | --- | --- |
| **GR4J / INRAE hydrological models** | Cheap catchment storage and rainfall–runoff routing | Not a literal well-depth or groundwater-plume model. [webGR](https://webgr.inrae.fr/eng/tools/hydrological-models/daily-hydrological-model-gr4j) |
| **MODFLOW 6** | Reference solutions for groundwater, wells, boundaries and solute transport | Use for offline comparison and selected validation cases; full 3D modeling is not necessary everywhere in TCE. [USGS](https://www.usgs.gov/software/modflow-6-usgs-modular-hydrologic-model) |
| **LISFLOOD-FP** | Conservative inundation, simplified hydraulics and channel/floodplain approaches | Choose the solver appropriate to backwater, flow speed and discontinuities. [University of Bristol](https://www.bristol.ac.uk/geography/research/hydrology/models/lisflood/) |
| **HEC-RAS** | Cross-sections, structures, roughness and 1D/2D coupling | Engineering reference, not automatically a suitable runtime dependency. [HEC](https://www.hec.usace.army.mil/confluence/rasdocs/r2dum/6.3/development-of-a-2d-or-combined-1d-2d-model/connecting-2d-flow-areas-to-1d-hydraulic-elements?scroll-versions%3Aversion-name=6.3) |
| **EPA SWMM** | Urban runoff, drainage networks, outfalls and water-quality routing | Most useful when dense urban infrastructure develops. [US EPA](https://www.epa.gov/water-research/storm-water-management-model-swmm) |
| **Timberborn** | Readable interaction among drought, storage, dams and irrigation | A gameplay and visualization reference, not validation of groundwater or pathogen physics. [Steam Store](https://store.steampowered.com/app/1062090/_Timberborn/?curator_clanid=27189131&l=english) |

**First implementation priority:** catchment water balance, a shared shallow aquifer, river/flood storage, wells, irrigation transfers and separate surface-versus-groundwater contamination pathways.

Defer deep confined aquifers, density-dependent saltwater intrusion, detailed sediment chemistry and full 3D groundwater until a demonstrated simulation need justifies them.

---

# 6. Data sources, calibration and evidence limits

## Useful datasets

| Dataset or reference | Best use in TCE | Important limitation |
| --- | --- | --- |
| **Caravan**, Kratzert et al. | Regional rainfall–runoff behavior, seasonality and discharge comparisons | Select comparable catchments; do not average incompatible climates together. [Nature](https://www.nature.com/articles/s41597-023-01975-w) |
| **ERA5-Land**, Copernicus | Hourly weather sequences and climate variability | Reanalysis is a modern climate analogue, not a reconstruction of an arbitrary ancient world. [Climate Data Store](https://cds.climate.copernicus.eu/datasets/reanalysis-era5-land?tab=overview) |
| **HydroATLAS / HydroSHEDS** | River networks, catchments and environmental attributes | Useful for regional relationships rather than individual well prediction. [HydroSHEDS](https://www.hydrosheds.org/hydroatlas) |
| **SoilGrids 2.0**, Poggio et al. | Soil texture, bulk properties and uncertainty for near-surface modeling | Its soil profiles do not supply deep aquifer geology. [Soil](https://soil.copernicus.org/articles/7/217/2021/) |
| **GLHYMPS 2.0**, Huscroft et al. | Geological permeability priors and distinction between unconsolidated material and bedrock | Intrinsic permeability \(k\) is not hydraulic conductivity \(K\): convert using fluid properties. Do not treat map values as site measurements. [AGU Journals](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1002/2017GL075860) |
| **USGS stream and groundwater studies** | Flood frequency, well response, stream depletion and regional validation cases | Their strongest evidence is local; transfer mechanisms more confidently than fitted coefficients. [USGS Publications](https://pubs.usgs.gov/sir/2005/5153/) |

## Where the evidence is thin or contested

**Historical depths:** surviving excavations are selective, and original ground level, water level and usage are not always recoverable. The retrieved evidence supports particular regional examples—not a global distribution of “ancient well depths.” Modern hand-dug wells are useful construction analogues only when clearly labeled as such.

**Sanitation setbacks:** the literature does not justify a universal distance that makes a well safe. Studies differ in geology, seasonal conditions, source attribution and whether they measure indicators or pathogens. Use distances as constraints or institutional rules, while retaining physical transport beneath them.

**Microbial removal:** neither filtration nor mortality has a universal coefficient. Prefer a small number of explicitly uncertain material–organism classes and sensitivity tests over highly precise-looking constants.

**Flood frequency:** a fitted return period depends on the climatic and engineered landscape being assessed. Over a centuries-long simulation, retain changing hazard distributions rather than a permanently valid “100-year flood line.”

**Performance:** the proposed grid sizes are plausible starting designs, not demonstrated benchmarks on TCE’s target PC. Profile hydraulic events, transport refinement and long-term state retention separately from citizen AI.

## Bottom line

**Simplify spatial resolution and solver dimensionality, not the causal connections.**

TCE does not need a research-grade hydrological model everywhere. It does need water to come from somewhere, withdrawals to affect shared stores, floods to occupy connected volume, and contamination to take a physically plausible route to the person who drinks it.

Those constraints are sufficient to produce many of the outcomes the project needs: reliable and unreliable wells, irrigation disputes, drought-delayed failures, damaging floods, beneficial infrastructure, pollution externalities and institutions that arise because people must manage the same water.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927e5-de14-83ea-bcd5-f728e833ae1d)
