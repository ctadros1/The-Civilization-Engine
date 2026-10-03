# Agricultural and food technology for The Civilization Engine

## Executive recommendation

**Model agriculture as a network of locally viable production systems—not a universal ladder from digging stick to hoe to plough.** For TCE, technological change should alter specific constraints: the area people can cultivate, the crops they can establish, water reliability, soil fertility, processing labor, storage losses, and the fraction of production available after seed and animal feed.

The central quantities should be:

| Quantity | What it reveals |
| --- | --- |
| **Net edible output per physical hectare per year** | Land productivity, including fallow and multiple cropping. |
| **Net edible output per person-day** | Labor productivity, including processing and transport—not just field work. |
| **Seasonal labor requirements** | Whether planting, weeding, irrigation, and harvesting can actually be completed on time. |
| **Output variability and reserve requirements** | Whether a system survives drought, pests, interrupted trade, or consecutive poor harvests. |
| **Soil, water, and livestock balances** | Whether apparent prosperity is sustainable or consumes its productive base. |

These measures can move in different directions. A labor-intensive irrigated system may support more people per hectare without freeing more workers for other occupations. Animal traction may greatly reduce cultivation labor without proportionately increasing harvest per hectare. FAO’s farm-power comparisons illustrate the importance of distinguishing those effects. [VTechWorks](https://vtechworks.lib.vt.edu/bitstreams/30b9f35b-2715-4acb-895d-638f9c206d21/download)

**Evidence limitation:** there is no defensible worldwide table assigning a single historical yield to each implement. Below, historical measurements and reconstructions are separated from **proposed TCE calibration ranges**. The latter are starting assumptions to test, not archaeological estimates.

---

# 1. Mechanisms: rules the simulation can implement

## 1.1 Domestication is a continuing biological process

Cultivation, domestication, and intensive agriculture are different developments. People can manage wild stands, plant morphologically wild seeds, and store substantial harvests before crops acquire a fully domesticated combination of traits. Archaeobotanical evidence often indicates changes unfolding over centuries or millennia rather than a single invention event. [PubMed](https://pubmed.ncbi.nlm.nih.gov/24753577/)

**Implementation rule:** separate four states:

**Access to a species → knowledge of propagation → an adapted cultivated population → a particular farming system.**

A community might know how to grow rice but lack an appropriate variety, sufficient water control, or enough labor to establish it successfully.

Represent crop populations with a small trait vector: growing-season length, temperature and moisture tolerances, day-length sensitivity, seed retention, dormancy, harvestable fraction, storage behavior, and disease resistance. Selection changes these traits gradually. Do not make every improvement beneficial in every environment.

For an early-agrarian TCE start, initialize some already-cultivated populations. Subsequent discovery can produce locally adapted varieties, new crop introductions, and new uses without requiring every settlement to repeat the entire domestication process.

## 1.2 Tools primarily change tasks and feasible environments

The useful distinctions are mechanical:

| Implement | Principal action | Appropriate simulation effect |
| --- | --- | --- |
| Digging stick | Opens planting holes and loosens small areas. | Low capital requirement; suitable for small plots, roots, and difficult terrain. |
| Hoe | Breaks surface soil, removes weeds, forms mounds and beds. | Flexible cultivation and weeding; substantial human labor requirement. |
| Ard | Scratches or opens furrows without fully inverting the soil. | Faster animal-powered preparation; sometimes repeated or crosswise passes. |
| Moldboard plough | Cuts and turns a soil slice. | Better burial of vegetation and different drainage effects, but greater draft requirements and site dependence. |

The heavy plough’s historical significance in northern Europe depended particularly on soils and agricultural geography; it should not be treated as a universal replacement for lighter implements. [European Historical Economics Society](https://ehes.org/wp/EHES_70.pdf?utm_source=chatgpt.com)

**Implementation rule:** implements modify operation rates, draft demand, effective depth, weed control, and terrain restrictions. Yield changes arise through those mechanisms—not through an automatic “plough = +30% food.”

An expensive plough can be inferior to a hoe on small terraced plots. An ard can remain economically attractive where a moldboard provides little additional benefit.

## 1.3 Agriculture is constrained by deadlines, not just annual labor totals

A household may possess enough annual labor but still fail because several fields require harvesting during the same short interval.

**Implementation rule:** every operation has:

`earliest_start`, `preferred_window`, `latest_useful_date`, `labor_remaining`, and `delay_response`.

Examples include soil preparation when ground is workable, sowing after suitable rainfall, weeding before competition becomes severe, and harvesting before lodging or shattering. The crop should continue developing while labor is unavailable.

This also makes tools, hired labor, animal-sharing arrangements, staggered crop calendars, and migration valuable without arbitrary economic bonuses. The same seasonal bottleneck is explicitly represented in *Farthest Frontier*: simultaneous harvests require more workers than staggered ones. [Farthest Frontier](https://www.farthestfrontier.com/guide/gameplay/farming/)

## 1.4 Soil fertility must obey a material balance

Manure does not create nutrients. It redistributes nutrients consumed by livestock and returns only part of them to fields. Legumes can add biologically fixed nitrogen, but they do not create phosphorus or potassium; removing grain or fodder also removes nutrients.

Archaeological isotope evidence places substantial manuring within Neolithic European agriculture, making “fertilization” inappropriate as an exclusively late or industrial discovery. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1305918110)

**Implementation rule:** maintain a simplified field balance:

\[
N\_{t+1}=N\_t+\text{fixation}+\text{mineralization}+\text{imports}
-\text{harvest exports}-\text{leaching}-\text{other losses}.
\]

Use corresponding balances for phosphorus and potassium, plus a slower soil-organic-matter state.

Distinguish fresh manure mass from nutrient content and immediately available nutrients. Rothamsted’s Broadbalk experiment, for example, records manure applications separately from their estimated nitrogen contribution; its treatments are useful calibration data, not a universal historical prescription. [Rothamsted ERA](https://www.era.rothamsted.ac.uk/Broadbalk)

This produces important economic tradeoffs: straw can become fodder, bedding, fuel, construction material, or a soil amendment, but cannot perform all those roles simultaneously.

## 1.5 Irrigation is a family of technologies

Do not implement one generic “irrigation” modifier.

| System | What it changes | Principal constraint or risk |
| --- | --- | --- |
| Flood-recession cultivation | Uses moisture remaining after inundation. | Flood timing, extent, and recession rate. |
| Basin irrigation | Retains and distributes floodwater. | Embankments, leveling, coordinated release. |
| Gravity canals | Moves water from a higher source to fields. | Gradient, discharge, maintenance, upstream withdrawals. |
| Wells and lifting devices | Raises water to otherwise inaccessible fields. | Labor or power, lift height, aquifer supply. |
| Tanks and reservoirs | Moves water through time rather than merely across space. | Storage capacity, evaporation, sedimentation. |
| Qanats | Conduct groundwater through gently sloping underground galleries. | Suitable geology, surveying, excavation, maintenance. |
| Paddy systems | Manage shallow water and field levels for rice cultivation. | Bunds, inlet/outlet control, establishment labor. |
| Drainage and raised beds | Remove excessive water and improve root aeration. | Outlet elevation and recurring maintenance. |

Early Peruvian canals demonstrate small-scale organized irrigation associated with mixed farming and foraging, without evidence of a centralized bureaucracy. Qanat systems likewise combine engineering with continuing management arrangements. **Water control should require coordination, not necessarily a state.** [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1288011/)

For a lifting device, enforce the physical relationship:

\[
P\_{\text{input}}\geq \frac{\rho gQH}{\eta},
\]

where \(Q\) is water flow, \(H\) is lift height, and \(\eta\) is efficiency. This prevents a small water-lifting mechanism from irrigating unlimited land.

## 1.6 Rotation changes several things at once

A rotation is a sequence of crops, treatments, and resting periods—not a fertility percentage.

**Implementation rule:** track the actual sequence and its consequences: nutrient exports and returns, weeds, crop-specific pathogens, soil cover, fodder supply, water demand, and competing labor calendars. APSIM provides an existing model framework for interacting crops, soil, and management sequences. [APSIM](https://www.apsim.info/)

Several distinctions matter:

* A two-field arrangement with half the land cropped differs from a three-field arrangement with two-thirds cropped. At identical harvest yield, that alone raises annual cropped area by **one-third**, not yield per harvested hectare.
* A legume crop is not equivalent to a legume green manure. Harvesting and exporting the crop changes the nutrient balance.
* Two rice crops are not produced by a “double-cropping bonus.” They require two viable growing windows, appropriate varieties, water, seed, and labor.

The arithmetic is straightforward; the feasibility should emerge from local conditions.

## 1.7 Animals are capital, consumers, and biological populations

Domestication did not deliver meat, abundant milk, wool, traction, and transport simultaneously. Animal management followed different pathways, and later specializations depended on breeding and husbandry. [Annual Reviews](https://www.annualreviews.org/content/journals/10.1146/annurev-ecolsys-110512-135813)

**Implementation rule:** distinguish animals by age, sex, reproductive state, body condition, and training.

A working ox requires maintenance food even outside ploughing season. A lactating animal requires a reproductive cycle. Slaughter creates immediate food but removes future breeding, milk, transport, or traction capacity.

Give livestock several possible feed sources—pasture, stubble, hay, crop residues, and grain—with different opportunity costs. A crop-livestock system must account for its grazing and fodder land, not only the hectares producing human food.

## 1.8 Processing and storage determine usable food

A harvest is not necessarily ready to eat. Cleaning, threshing, dehusking, grinding, cooking, preservation, and transport consume labor, fuel, water, and materials.

Rice processing is a particularly clear example: paddy, brown rice, and milled rice are different goods, and drying, storage, and milling practices affect both recovery and quality. FAO also notes that grain scattered during processing may become poultry feed rather than disappear from the household economy. [FAOHome](https://www.fao.org/4/t0567e/T0567E0h.htm)

**Implementation rule:** use transformation recipes with co-products:

| Input chain | Outputs to retain |
| --- | --- |
| Harvested cereal → threshing/winnowing | Grain, straw, chaff, losses. |
| Grain → milling/sieving | Meal or flour, bran, losses. |
| Oilseed → pressing | Oil, press cake. |
| Milk → coagulation/draining | Curd or cheese, whey. |
| Grain → malting/brewing | Beverage, spent grain, processing losses. |
| Root crop → preparation/drying | Prepared food, peelings or residues, water loss. |

Fermentation and drying do not create calories. Additional product mass may be water; reduced mass may reflect drying rather than nutritional destruction.

Store food in lots with moisture, age, pest exposure, container type, and quality. Separate gradual deterioration from disasters such as fire, flooding, or a severe infestation.

## 1.9 Institutions control access and incentives

Water rights, grazing access, labor exchanges, land tenure, milling fees, seed loans, and collective stores affect whether known techniques are adopted.

**Implementation rule:** a household compares expected food, income, labor demands, risk, and obligations—not simply maximum physical yield. An innovation can be rejected because its benefits accrue to a landlord, because the household lacks seed or credit, or because failure would consume its reserves.

Lansing and Kremer’s Bali model is especially relevant: interactions between water allocation, pest control, and cropping coordination can generate organized agricultural patterns without a single optimizing controller. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/abs/10.1525/aa.1993.95.1.02a00050)

Keep **physical surplus**, **household entitlement**, and **elite extraction** separate. Rent and tax transfer food between agents; they are not spoilage.

---

# 2. Parameters: measurements, reconstructions, and proposed defaults

## 2.1 Measurement conventions

Use these definitions consistently:

\[
R\_{\text{seed}}=\frac{\text{gross harvested quantity}}{\text{quantity planted}}
\]\[
Y\_{\text{annual land}}=
\frac{\text{annual usable output}}{\text{physical land supporting the system}}
\]\[
P\_{\text{labor}}=
\frac{\text{usable output}}{\text{person-days across the stated production stages}}.
\]

For grain, specify moisture basis. For rice, specify paddy versus milled rice. For potatoes and other roots, distinguish fresh mass from dry matter or edible energy.

A seed ratio is meaningful only when the quantities are comparable. It is not a cross-species productivity ranking: a small-seeded cereal can have a large ratio without providing extraordinary food per hectare.

**Confidence notation:** **H** = strong for the stated observation or physical relationship; **M** = reconstruction or context-dependent estimate; **L** = weak transferability or deliberately provisional assumption.

## 2.2 Empirical anchors

| Parameter and context | Value | How to use it | Confidence and source |
| --- | --- | --- | --- |
| English wheat, 1250–1299 | **8.71 bushels/acre, net of seed** | Approximately **0.59 t/ha** under a 60 lb/bushel conversion assumption. Not gross harvest. | **M**: historical reconstruction; conversion approximate. |
| English wheat, 1400–1449 and 1750–1799 | **5.89** and **17.26 bushels/acre, net of seed** | Approximately **0.40** and **1.16 t/ha** under the same assumption. Useful evidence against smooth, monotonic progress. | **M**: same reconstruction. [Academia](https://www.academia.edu/385400/HISTORICAL_NATIONAL_ACCOUNTS_FOR_BRITAIN_1300_1850_SOME_PRELIMINARY_ESTIMATES) |
| Central Russian wheat, selected eighteenth-century decades | Harvest/seed ratios **3.0–4.3** | A case-specific constraint on seed deductions and surplus. | **M**: compiled historical evidence. [Williams College](https://web.williams.edu/Economics/wp/nafzigerMicroLivingStandards_WilliamsWorkingPaper_Nov2007.pdf) |
| Primary tillage, hand labor versus animal traction in a FAO African farm-power comparison | About **500 versus 60 human-hours/ha** | Operation-specific labor comparison—not total farming labor or an ancient yield estimate. | **M** for the comparison; **L** for direct prehistoric transfer. [VTechWorks](https://vtechworks.lib.vt.edu/bitstreams/30b9f35b-2715-4acb-895d-638f9c206d21/download) |
| Philippine “low-technology” rice example, 1980s | **2.5 t paddy/ha; 84 person-days and 14 animal-days/ha** | About **29.8 kg paddy/person-day**, before downstream processing. | **M** case estimate. It used IR36 rice and urea: **not a pre-industrial observation**. [FAOHome](https://www.fao.org/4/t0567e/T0567E03.htm) |
| General seasonal crop-water needs | Wheat/barley/oats **450–650 mm**; maize **500–800 mm**; paddy rice **450–700 mm**; potato **500–700 mm** | Broad crop-water requirements; not canal withdrawals. Rainfall, conveyance, land preparation, and seepage must be treated separately. | **M** agronomic guidance. [FAOHome](https://www.fao.org/4/s2022e/s2022e02.htm) |
| Grain moisture before storage | Approximately **14% for maize** and **13% for sorghum** in the cited guidance | Useful condition thresholds, not guarantees of zero loss. | **M**, crop and environment dependent. [FAOHome](https://www.fao.org/4/T0522E/T0522E03.htm) |
| Rice post-harvest losses in FAO’s synthesis | Storage-stage losses **2–6%**; whole-chain estimates **10–almost 40%** | Do not reinterpret these as universal annual granary-loss rates. Stage boundaries and storage duration matter. | **M** historical synthesis of field estimates; **L** transfer to ancient societies. [FAOHome](https://www.fao.org/4/t0567e/T0567E0h.htm) |
| Philippine official paddy-to-rice milling conversion, updated in 2025 | **63%** | A modern mass-accounting reference demonstrating that paddy tonnage is not edible-rice tonnage. | **H** as the published conversion; **L** as an ancient-milling default. [Philippine Statistics Authority](https://psa.gov.ph/content/psa-board-approves-updated-milling-recovery-rate-630-percent-palay-rice) |

The English figures come from a **preliminary national-accounts reconstruction**, not direct observations of a representative national sample in each period. Preserve that qualification in TCE’s research metadata.

## 2.3 Proposed TCE calibration envelopes

**Everything in the following table is a modeling proposal, with low initial confidence—not a sourced estimate of a historical society.** The empirical anchors above constrain plausibility but do not establish these exact intervals.

Here, a person-day is normalized to **eight hours**. Field labor includes establishment, cultivation, harvesting, and primary threshing, but excludes household cooking, milling, routine animal care, and long-distance transport. Historical sources’ original “days” may use different conventions.

| Farming-system archetype | Gross harvest per harvested hectare | Planting requirement | Field person-days/ha/crop | Implied harvest/seed ratio | Gross output per field person-day |
| --- | --- | --- | --- | --- | --- |
| Low-input, hand-cultivated wheat/barley | **0.5–1.3 t grain** | **100–200 kg seed** | **80–160** | **2.5–13** | **3–16 kg grain** |
| Similar environment and inputs, animal-tilled wheat/barley | **0.5–1.3 t grain** | **100–200 kg seed** | **35–80** | **2.5–13** | **6–37 kg grain** |
| Well-maintained, manured temperate cereal system | **1.0–2.0 t grain** | **120–200 kg seed** | **40–90** | **5–17** | **11–50 kg grain** |
| Low-input millet/sorghum | **0.4–1.2 t grain** | **5–20 kg seed** | **50–110** | **20–240** | **4–24 kg grain** |
| Hand-cultivated maize | **0.8–2.0 t grain** | **15–30 kg seed** | **60–120** | **27–133** | **7–33 kg grain** |
| Labor-intensive wet rice | **1.5–3.0 t paddy** | **30–60 kg seed** | **100–220** | **25–100** | **7–30 kg paddy** |
| Potato cultivation | **5–12 t fresh tubers** | **1–2 t planting tubers** | **80–160** | **2.5–12** | **31–150 kg fresh tubers** |

The identical hand- and animal-tilled cereal yield ranges are deliberate: **traction changes labor capacity first**. Subsequent yield differences should follow actual differences in timeliness, weed control, land selection, and management.

The last two columns show mathematical extremes, not likely joint outcomes. Do not independently draw every parameter from a uniform distribution; drought, poor soil, difficult terrain, and inadequate labor should produce correlated outcomes.

For cassava, taro, yams, bananas, and tree crops, create separate profiles. Their propagation units, time to harvest, harvest flexibility, and water content make a cereal-style seed ratio misleading.

### Converting labor productivity into worker-year output

Do not simply multiply kilograms/person-day by 365.

A worker’s annual output must emerge from the area successfully managed through all critical windows, minus time spent on animal care, processing, transport, maintenance, childcare or other obligations, and non-agricultural work.

For example, **a proposed scenario** of 2 ha at 1 t/ha and 50 field person-days/ha gives 2 t gross grain from 100 field days. It does not establish a 2 t disposable surplus: seed, losses, processing, animal inputs, and household consumption still have to be deducted.

## 2.4 Worked food-budget example

Consider a deliberately simplified scenario:

* One physical hectare, with two-thirds cropped each year.
* Gross grain yield of 1,000 kg per harvested hectare.
* Seed reservation of 150 kg per harvested hectare.
* Subsequent losses of 10%.
* Flour extraction of 85%.

Then:

\[
\text{flour per physical ha-year}
=\frac23(1000-150)(0.90)(0.85)
=433.5\text{ kg}.
\]

At an illustrative accounting value of 3,400 kcal/kg and an adult-equivalent requirement of 2,400 kcal/day, that is approximately **1.68 adult-equivalent food-years**.

Those energy values are scenario assumptions, not a dietary prescription. The calculation also excludes other foods, grazing land, animal feed, and fuel. Bran remains a co-product rather than automatically becoming waste.

The important result is architectural: **“one tonne harvested” is not “one tonne available to support specialists.”**

---

# 3. Worldwide development and variation

## 3.1 Domestication centers and packages

Dates below are approximate. They indicate early cultivation, domestication processes, or securely established complexes—not that all listed crops appeared together.

| Region | Approximate early development | Important crops and animals | Implication for TCE |
| --- | --- | --- | --- |
| **Southwest Asia** | Cultivation and domestication processes especially **c.9500–8000 BCE** | Einkorn/emmer wheat, barley, pulses, flax; sheep, goats, cattle, pigs through distinct management histories. | A cereal–pulse–livestock pathway, not the definition of agriculture everywhere. [Botanical Society of America](https://bsapubs.onlinelibrary.wiley.com/doi/10.3732/ajb.1400145) |
| **Northern China** | Millet cultivation developing through roughly **eighth–sixth millennia BCE** | Broomcorn and foxtail millet; pig husbandry; later additions to the crop package. | Summer-rainfall dry farming requires different calendars from Mediterranean winter cereals. [ResearchGate](https://www.researchgate.net/publication/272434383_Crops_cattle_and_commensals_across_the_Indian_Ocean?utm_source=chatgpt.com) |
| **Yangtze and southern China** | Rice exploitation, cultivation, and domestication across a long early-Holocene sequence; major domestication changes during roughly **6500–4000 BCE** | Rice and associated wetland resources; subsequently diverse rice systems. | Rice cultivation must not imply fully engineered paddy agriculture from its beginning. [Science](https://www.science.org/doi/10.1126/science.1166605) |
| **South Asia** | Farming at Mehrgarh from about **7000 BCE**; multiple later local domestication trajectories | Introduced wheat/barley combined with zebu and indigenous pulses, millets, and other crops. | South Asia is a set of interacting centers, not merely an eastern extension of the Fertile Crescent. [UCL Discovery](https://discovery.ucl.ac.uk/1456411/1/Fuller_Anthropology%20General%20article%2030%20August%202014%20%281%29.pdf) |
| **New Guinea** | Early cultivation evidence; clear mounded cultivation at Kuk around **5000–4500 BCE**, with later ditched systems | Bananas, taro, and other vegetatively propagated plants. | Horticulture, drainage, and mounding can support development without cereals or plough animals. [UW Faculty](https://faculty.washington.edu/plape/pacificarchaut12/Denham%20et%20al%202003.pdf) |
| **African savannas** | Sorghum domestication evidence in eastern Sudan by approximately **3500 BCE**; domesticated pearl millet in Mali by around **2500 BCE** | Sorghum, pearl millet, cowpea and other regional crops; livestock combinations vary. | Drought adaptation and seasonal risk matter more than a temperate cereal technology ladder. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0305440320301801) |
| **Other African centers** | Multiple trajectories; early dates are unevenly resolved | African rice, finger millet, teff, enset, yams, oil palm and other local crops. | Author several African crop profiles; do not reduce the continent to imported wheat or later maize. [Botanical Society of America](https://bsapubs.onlinelibrary.wiley.com/doi/10.3732/ajb.1400145) |
| **Mesoamerica** | Early squash cultivation; maize evidence in the Balsas region by about **6750 BCE**, with later development of broader packages | Maize, beans, squash, chili, amaranth, agave and others; turkey domestication later. | Productive farming and urban societies need no large draft-animal prerequisite. [Smithsonian Research Online](https://repository.si.edu/handle/10088/14850) |
| **Andes** | Multiple Holocene developments, with established tuber, grain, and camelid economies long before the Inca | Potatoes and other tubers, quinoa and related grains; llamas, alpacas, guinea pigs. | Altitude, frost, storage, herding, and exchange between ecological zones are central. Exact domestication dates differ among species. [Annual Reviews](https://www.annualreviews.org/content/journals/10.1146/annurev-ecolsys-110512-135813) |
| **Southwestern Amazonia** | Cultivation evidence for cassava and squash around **10,000 years ago** | Root crops, squash, and managed landscapes; later diverse agroforestry systems. | The evidence concerns cultivation; do not automatically infer fully domesticated morphology or a cereal-like economy. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7250647/) |
| **Eastern North America** | Indigenous domestication over approximately **5000–3700 BP** | Squash, sunflower, sumpweed, goosefoot; maize became important later. | Local agriculture can precede adoption of a subsequently dominant imported staple. [DOI](https://doi.org/10.1086/659645) |

### Animal packages need their own chronology

Do not grant every early farming society cattle, horses, chickens, and sheep at initialization. Genomic work places the major expansion of the ancestry underlying modern domestic horses around **2200 BCE**. A major 2022 reassessment identified the earliest unambiguous domestic chickens in its dataset in Thailand around **1650–1250 BCE**, while also prompting debate over earlier evidence. [Nature](https://www.nature.com/articles/s41586-021-04018-9)

For TCE, biological availability should be a separate requirement from knowledge of husbandry.

## 3.2 Crop movement was neither instantaneous nor purely productivity-driven

European Neolithic spread provides one quantified benchmark: a study using 735 archaeological sites estimated a continental-scale advance of approximately **0.6–1.3 km/year**. This describes a population-and-farming frontier, not the speed at which an individual seed shipment or idea travels. [PLOS](https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.0030410)

Other exchanges followed different networks. African sorghum, pearl millet, and several pulses reached South Asia through prehistoric exchanges, with important transfers around the second millennium BCE. Later Indian Ocean exchanges moved Asian crops toward Africa; the timing and routes of bananas, taro, and yams remain less securely resolved. [ResearchGate](https://www.researchgate.net/publication/272434383_Crops_cattle_and_commensals_across_the_Indian_Ocean?utm_source=chatgpt.com)

Introduced plants also passed through changing social roles: novelty, prestige product, garden crop, and eventually staple. A crop’s initial attraction need not be its maximum caloric productivity. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/00438243.2012.729404?utm_source=chatgpt.com)

**Implementation rule:** transmission requires propagules and people capable of establishing them. Adoption then depends on adaptation, cuisine, processing knowledge, land access, risk, and demand. Knowledge alone should not teleport a viable crop into every settlement.

## 3.3 Variation across broad periods

These are analytical comparisons, **not proposed TCE era gates**.

| Context | Typical technical possibilities | What not to assume |
| --- | --- | --- |
| **Foraging and pre-domestication cultivation** | Plant management, grinding, cooking, storage, selective harvesting. | Food processing and fixed stores are not necessarily consequences of fully domesticated agriculture. Bread-like remains around 14,400 years old and predomestication granaries demonstrate this. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1801071115) |
| **Early farming** | Locally adapted crop packages, hand cultivation, household processing, herding in some regions. | A complete cereal–plough–cattle package or an immediate disappearance of wild foods. [PubMed](https://pubmed.ncbi.nlm.nih.gov/24753577/) |
| **Pre-industrial intensification** | Traction, engineered water control, terraces, rotations, manure handling, specialized milling and preservation. | All regions follow the same sequence, or irrigation requires centralized government. [UChicago ISAC](https://isac.uchicago.edu/sites/default/files/uploads/shared/docs/Publications/OIS/ois13.pdf) |
| **Industrial transition** | Increasing mechanical power, transport integration, manufactured inputs, and more systematic experimentation. | Labor-saving machinery and yield-increasing agronomy are the same change. [VTechWorks](https://vtechworks.lib.vt.edu/bitstreams/30b9f35b-2715-4acb-895d-638f9c206d21/download) |
| **Modern systems** | Highly specialized cultivars, powered operations, managed nutrients and water, sophisticated processing chains. | Modern experimental yields or “traditional” twentieth-century farms can be projected unchanged into prehistory. [FAOHome](https://www.fao.org/aquacrop/overview/en) |

---

# 4. Technology graph: concrete nodes, prerequisites, and unlocks

I recommend approximately **40–45 reusable capability nodes**, with crop species, cultivars, recipes, and regional variants authored separately. That is large enough to support divergent agricultural histories without allocating a separate discovery node to every edible plant.

The catalog below contains **45 nodes**. Dates are historical authoring anchors, not research gates. “Unresolved” means this review does not establish a defensible worldwide first appearance. Several practices almost certainly predate their surviving documentation.

Prerequisites describe **TCE implementation choices**, not claims about one obligatory historical sequence. Carpentry, stoneworking, ceramics, metallurgy, surveying, and power transmission can be shared with other graph branches.

## 4.1 Biological management

| ID and node | Prerequisites and conditions | Approximate historical anchor | Concrete unlocks |
| --- | --- | --- | --- |
| **B01 — Seed cultivation and selection** | Plant knowledge; viable seed; workable ground. | Southwest Asia, especially **c.9500–8000 BCE**, with independent trajectories elsewhere. | Sown fields; reserved seed; crop populations; selection trials. [PubMed](https://pubmed.ncbi.nlm.nih.gov/24753577/) |
| **B02 — Vegetative propagation** | Suitable plants; living planting material; planting knowledge. | Prehistoric tropical agriculture; strong Kuk cultivation evidence by **c.5000 BCE**. | Tuber gardens, banana stands, cutting nurseries; non-seed propagation. [UW Faculty](https://faculty.washington.edu/plape/pacificarchaut12/Denham%20et%20al%202003.pdf) |
| **B03 — Managed breeding and herding** | Suitable animals; handling knowledge; feed and water. | Southwest Asian livestock management, **ninth–eighth millennia BCE**, with distinct developments elsewhere. | Breeding herds, pens, slaughter management, hides and dung. [Annual Reviews](https://www.annualreviews.org/content/journals/10.1146/annurev-ecolsys-110512-135813) |
| **B04 — Dairying** | B03; lactating animals; collection and handling. | Milk exploitation documented in **seventh-millennium BCE Anatolia**. | Milk, dairy vessels, lactation management; prerequisite for dairy processing. [Nature](https://www.nature.com/articles/nature11698) |
| **B05 — Perennial horticulture** | Propagation knowledge; suitable trees/vines; secure access over multiple years. | Major Southwest Asian/Mediterranean developments during the **fifth–third millennia BCE**; species-specific beginnings vary. | Orchards, vineyards, long-lived planted assets, fruit and oil-bearing crops. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/10041234/1/10.1007%252Fs00334-017-0659-2.pdf) |

## 4.2 Cultivation and harvest tools

| ID and node | Prerequisites and conditions | Approximate historical anchor | Concrete unlocks |
| --- | --- | --- | --- |
| **T01 — Digging-stick cultivation** | Woodworking; human labor. | Prehistoric; worldwide first appearance unresolved. | Planting holes, root harvesting, small cultivated plots. |
| **T02 — Hoes and bed formation** | Hafting/toolmaking; wood, stone, shell, or later metal. | Early farming in multiple regions; no single origin established. | Hoe cultivation, weeding, mounds, garden beds. |
| **T03 — Specialized cereal harvesting** | Suitable cutting edges or harvesting tools; harvest knowledge. | Predomestication cereal exploitation and early farming. | Sickles or regional equivalents; faster cutting; grain-plus-straw harvest. |
| **T04 — Threshing and winnowing** | Harvested seed crops; suitable work area; labor or animal power. | Prehistoric; precise first appearance unresolved. | Clean grain, straw and chaff separation, threshing floors. |
| **T05 — Trained draft animals and yokes** | B03; suitable mature animals; harness woodworking; feed surplus. | Secure ancient Eurasian use; important developments by the **fourth millennium BCE**. | Traction service, animal-powered operations, shared teams. [ResearchGate](https://www.researchgate.net/publication/249007174_The_Secondary_Products_Revolution_the_past_the_present_and_the_future) |
| **T06 — Ard cultivation** | T05; plough frame and share; suitable terrain. | Ancient Southwest Asia, securely established by the **fourth millennium BCE**; earlier claims vary. | Furrow preparation, expanded cultivation capacity. [ResearchGate](https://www.researchgate.net/publication/249007174_The_Secondary_Products_Revolution_the_past_the_present_and_the_future) |
| **T07 — Durable metal working edges** | Metallurgy; tool repair; suitable design. | Bronze- and Iron-Age developments, with large regional differences. | Metal hoe blades and ploughshares; altered wear, repair, and soil-working costs. |
| **T08 — Moldboard ploughing** | Strong draft system; suitable plough construction; workable field geometry. | Early imperial Chinese antecedents; medieval northern European development and spread, broadly **first millennium CE**. Exact invention chronology is debated. | Soil inversion; different weed and drainage management; higher draft demand. [European Historical Economics Society](https://ehes.org/wp/EHES_70.pdf?utm_source=chatgpt.com) |
| **T09 — Controlled row sowing and seed drills** | Seed metering or guided placement; tool construction; compatible field preparation. | Ancient Asian precedents; later regional elaborations. The first appearance of each mechanism requires separate dating. | More controlled depth and spacing; reduced seed waste; row-based cultivation. |

T01–T04 and T07 are intentionally given broad or unresolved dates rather than invented “firsts.” Their functionality matters more to the simulation than assigning a spurious universal invention year.

## 4.3 Water and land engineering

| ID and node | Prerequisites and conditions | Approximate historical anchor | Concrete unlocks |
| --- | --- | --- | --- |
| **W01 — Managed flood basins** | Earthworks; floodplain knowledge; collective or household maintenance. | Ancient Nile and Mesopotamian systems; origins differ between natural recession farming and engineered basins. | Embanked basins, controlled inundation and release. |
| **W02 — Gravity canals** | Excavation; gradient assessment; accessible elevated water source. | Neolithic Southwest Asian systems; early Peruvian canals also documented, including evidence approximately **5,400 years old** and earlier possibilities. | Intakes, distribution canals, irrigation turns. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1288011/) |
| **W03 — Wells and counterweighted lifts** | Excavation or accessible water; ropes, containers, lever construction. | Ancient Southwest Asia and Egypt; shaduf-type devices by the **second millennium BCE**. | Small-scale lifted irrigation and watering points. |
| **W04 — Continuous lifting machinery** | Mechanical transmission; suitable power source; W03-type water access. | Ancient Mediterranean and Asian developments; several mechanisms, not one invention. | Bucket chains, lifting wheels, screw variants, higher throughput. [Academia](https://www.academia.edu/435243/Machines_power_and_the_ancient_economy) |
| **W05 — Qanat construction** | Surveying, tunneling, maintenance organization; appropriate aquifer and gradient. | Iranian plateau, approximately **first millennium BCE**; FAO gives around **800 BCE** as a development anchor. | Underground water galleries, access shafts, gravity-fed oases. [FAOHome](https://www.fao.org/giahs/giahs-around-the-world/iran-qanat-irrigated-systems/en) |
| **W06 — Tanks and agricultural reservoirs** | Earthworks or masonry; catchment and outlet control. | Ancient systems in several regions, notably South Asia; no single worldwide first date. | Seasonal water storage, tank-fed fields, sediment-management tasks. |
| **W07 — Agricultural drainage** | Ditches or raised ground; functioning outlet. | Kuk mounded cultivation **c.5000–4500 BCE** and ditched systems **c.2400–2000 BCE** provide clear anchors. | Drained gardens, drainage networks, reclaimed wet ground. [UW Faculty](https://faculty.washington.edu/plape/pacificarchaut12/Denham%20et%20al%202003.pdf) |
| **W08 — Bunded and leveled rice fields** | Rice cultivation; earthworks; water and drainage control. | Neolithic lower Yangtze developments; construction and domestication need separate chronologies. | Managed paddies, field gates, controlled establishment regimes. [Royal Society Publishing](https://royalsocietypublishing.org/rstb/article/372/1735/20160429/30377/Geographic-mosaics-and-changing-rates-of-cereal) |
| **W09 — Terracing** | Earthmoving; retaining structures where needed; maintenance. | Independent ancient traditions; global first appearance unresolved. | Cultivable slopes, erosion control, deeper retained soil, terrace irrigation where feasible. |
| **W10 — Raised-field wetland agriculture** | Bed construction; channels; repeated organic-material or sediment management. | Pre-Columbian American systems, including Andean raised fields and later central Mexican chinampas; local dates vary. | Raised beds, canal access, wetland cultivation and associated maintenance. |

W01, W03, W06, W09, and W10 warrant regional archaeological sub-records before assigning narrower “first appearance” dates. A general comparative starting point is the research collected in *Irrigation in Early States: New Directions*. [UChicago ISAC](https://isac.uchicago.edu/sites/default/files/uploads/shared/docs/Publications/OIS/ois13.pdf)

## 4.4 Fertility and cropping systems

| ID and node | Prerequisites and conditions | Approximate historical anchor | Concrete unlocks |
| --- | --- | --- | --- |
| **S01 — Managed fallow** | Land access; observation of vegetation and soil recovery. | Prehistoric; no securely dated worldwide invention. | Resting parcels, fallow grazing, shifting field schedules. |
| **S02 — Manure collection and composting** | Collectable organic material; transport; handling and decomposition knowledge. | Manuring documented in Neolithic Europe by the **sixth millennium BCE**; other regional histories differ. | Manure heaps, compost areas, field amendment recipes. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1305918110) |
| **S03 — Cereal–legume sequences and mixtures** | Appropriate crop stocks; compatible calendars; retained residues where required. | Ancient practices in multiple regions; first systematic rotations are difficult to date. | Rotation plans, mixed stands, green-manure variants. |
| **S04 — Coordinated multi-field rotation** | Shared scheduling where fields or grazing are interdependent; suitable winter/spring crops. | Medieval European three-field arrangements, broadly **c.800–1200 CE** as a development window rather than one invention. | Multi-year field schedules; coordinated stubble grazing and fallow. |
| **S05 — Sequential multiple cropping** | Short enough crop cycles; water, temperature, seed, and labor for each crop. | Long regional histories; major medieval East Asian elaborations. First appearance is locality- and crop-specific. | Two or more sequential crop cohorts per physical field-year. |
| **S06 — Fodder crops, hay, and ley systems** | Suitable forage plants; cutting and storage; livestock demand. | Ancient fodder practices; important mixed-farming elaborations in early-modern northwestern Europe. | Hay stores, forage fields, winter feeding, grass–arable rotations. |

Treat these as **management-plan capabilities**. Their quantitative effects should be computed from crop, soil, and herd balances rather than assigned from their historical labels.

## 4.5 Food processing and storage

| ID and node | Prerequisites and conditions | Approximate historical anchor | Concrete unlocks |
| --- | --- | --- | --- |
| **P01 — Pounding and grinding** | Suitable stones or wooden implements. | Pre-agricultural; first appearance varies by process. | Mortars, pestles, saddle querns, meal, dehusking services. |
| **P02 — Prepared cereal foods** | Edible grain; preparation; fire and suitable cooking method. | Bread-like remains in Jordan by approximately **12,450 BCE**. | Porridges, flatbreads and other recipes; household cooking tasks. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1801071115) |
| **P03 — Drying and smoking** | Appropriate weather or fuel; racks, shelter, handling knowledge. | Prehistoric; worldwide first appearance unresolved. | Dried grain, fruit, meat and fish; smokehouses or drying shelters. |
| **P04 — Salting, brining, and controlled food fermentation** | Salt or suitable fermentation inputs; vessels; process knowledge. | Ancient practices with product-specific, often uncertain origins. | Salted foods, pickles, fermented staples and storage recipes. |
| **P05 — Dedicated bulk stores** | Construction; dry or otherwise suitable storage conditions; pest management. | Predomestication granaries at Dhra’, approximately **9350–9225 BCE**. | Granaries, raised floors, bins, reserve management. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0812764106) |
| **P06 — Brewing and beverage fermentation** | Fermentable substrate; suitable preparation; containers; process knowledge. | Jiahu mixed fermented beverages, approximately **7000–6600 BCE**, provide a secure early example—not a universal first beer date. | Malt or other substrate preparation, brewing vessels, beverages and spent-grain co-products. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0407921102) |
| **P07 — Cheese making** | B04; coagulation and separation knowledge; suitable containers or strainers. | Evidence from Poland in the **sixth millennium BCE**. | Cheese, curds, whey, dairy storage recipes. [Nature](https://www.nature.com/articles/nature11698) |
| **P08 — Fruit and oilseed pressing** | Suitable crops; crushing and separation equipment. | Ancient Mediterranean and Southwest Asian traditions; dates vary by crop and press mechanism. | Oil, juice, press cake, press workshops. |
| **P09 — Rotary milling** | Appropriate millstones; bearings and rotary drive. | **First millennium BCE Mediterranean** development. | Rotary querns, animal-driven mills, altered milling labor costs. [Academia](https://www.academia.edu/435243/Machines_power_and_the_ancient_economy) |
| **P10 — Water-powered milling** | P09-compatible grinding; water-power engineering; dependable flow. | Late first-millennium BCE/early first-millennium CE Mediterranean evidence. | Watermills, millraces, milling services and maintenance work. [Academia](https://www.academia.edu/435243/Machines_power_and_the_ancient_economy) |
| **P11 — Wind-powered milling** | Rotary machinery; suitable wind regime; structural engineering. | Medieval Sistan/eastern Iranian tradition; precise origin claims require caution. | Windmills, wind-dependent milling capacity. [UNESCO World Heritage Centre](https://whc.unesco.org/en/tentativelists/6192/) |
| **P12 — Alkaline maize processing** | Maize; alkaline material; water, heat, and processing knowledge. | Pre-Columbian Mesoamerica; earliest dating remains uncertain. | Nixtamal, masa, associated foods; changed nutritional availability and processing requirements. [CIMMYT](https://www.cimmyt.org/news/what-is-nixtamalization/) |
| **P13 — Cassava preparation and detoxification** | Cassava; variety-appropriate processing; tools, water, and/or heat. | Indigenous tropical American traditions; earliest secure process chronology unresolved here. | Distinct safe-food recipes, starch or flour, presses or graters where appropriate. |
| **P14 — Environmental freeze-drying** | Suitable tubers; repeated freezing and drying conditions; processing knowledge. | Pre-Columbian Andes; first appearance unresolved. | Chuño-like preserved products; climate-dependent seasonal processing. |
| **P15 — Concentrated sweeteners and sugar processing** | Sugar-bearing crops; extraction, heating, and concentration. | Ancient South Asian and later wider Eurasian traditions; precise stages and first crystallization dates require separate verification. | Syrups, concentrated sweeteners, sugar-processing equipment and fuel demand. |

The uncertain dates in P13–P15 should remain nullable metadata rather than being converted into invented precision. Their recipe functionality can still be implemented.

### Graph logic

Keep **knowledge prerequisites**, **material access**, and **environmental feasibility** separate.

For example:

```
Water-powered milling:
  knowledge: rotary milling AND water-power engineering
  materials: millstones AND structural materials
  site: usable hydraulic head AND sufficient seasonal flow
  operation: competent labor AND functioning equipment
```

“Has discovered watermills” should not imply that a desert settlement can operate one.

---

# 5. Stylized facts and validation targets

A correct simulation should reproduce the following patterns without prescribing historical dates.

| Pattern | Quantitative or observable test |
| --- | --- |
| **Seed deductions strongly constrain low-yield systems.** | At harvest/seed ratios of **2, 4, and 10**, reserving the original seed quantity leaves **50%, 75%, and 90%** of gross harvest before other deductions. |
| **Labor-saving and land-saving changes diverge.** | A traction intervention should substantially reduce cultivation labor in appropriate conditions without requiring an equal proportional increase in yield/ha. Compare the FAO operation-level benchmark. [VTechWorks](https://vtechworks.lib.vt.edu/bitstreams/30b9f35b-2715-4acb-895d-638f9c206d21/download) |
| **Production can rise through greater cropping frequency.** | Moving from half to two-thirds of land cropped yields **33% more cropped area**, holding harvest yield constant. Multiple cropping requires complete additional production cycles. |
| **Technological histories are not monotonically improving.** | Soil exhaustion, lost animals, interrupted maintenance, and altered labor incentives can reduce output despite retained knowledge. The English reconstructed yield series itself is non-monotonic. [Academia](https://www.academia.edu/385400/HISTORICAL_NATIONAL_ACCOUNTS_FOR_BRITAIN_1300_1850_SOME_PRELIMINARY_ESTIMATES) |
| **Processing can precede domestication.** | Worlds should permit prepared cereal foods and substantial stores before a complete domesticated crop package, consistent with the Jordanian evidence. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1801071115) |
| **Trade transmits both productive organisms and problems.** | Crop introductions should sometimes arrive with weeds or commensal pests; biological exchange is not an unqualified productivity bonus. [ResearchGate](https://www.researchgate.net/publication/272434383_Crops_cattle_and_commensals_across_the_Indian_Ocean?utm_source=chatgpt.com) |
| **Coordination matters independently of formal state capacity.** | Small communities should sometimes maintain irrigation successfully; larger systems should face maintenance and allocation conflicts. Early Peruvian canals are an important counterexample to mandatory hydraulic states. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1288011/) |
| **An agricultural “package” can be transformed by an introduced crop.** | New staples should sometimes alter land use, diets, and settlement potential without an accompanying tool revolution. Research on the Old World adoption of potatoes provides a historical example to examine. [OUP Academic](https://academic.oup.com/qje/article-abstract/126/2/593/1868756) |
| **Harvest crises propagate across years.** | Eating seed, slaughtering draft animals, or abandoning canals should improve immediate survival while potentially reducing future production. This follows directly from the model’s stock balances. |

Also test spatial correlation. A drought must affect neighboring fields together; independently drawing each field’s yield will give large settlements unrealistically reliable harvests through averaging.

---

# 6. Recommended TCE representation

## 6.1 Entities and responsibilities

| Entity | Recommended state |
| --- | --- |
| **Person** | Skills, health and work capacity, knowledge, relationships, obligations, current task, time budget. |
| **Household or production group** | Food and seed claims, pooled assets, labor plan, land access, debts, risk tolerance, adoption decisions. |
| **Field parcel** | Area, terrain, soil states, root-zone water, drainage, crop cohort, operations completed, ownership/access. |
| **Crop population** | Species, local variety traits, propagation method, quality, disease status, seed or planting-stock availability. |
| **Herd** | Animals or cohorts by age, sex, reproductive state, condition, training, feed demand. |
| **Processing or storage building** | Capacity, throughput, power, condition, owner, workforce, inventories, operating recipes. |
| **Institution** | Water rights, grazing rules, collective labor, store access, fees, sanctions, allocation procedure. |

Keep people individually simulated while aggregating plants into field cohorts. Individual plants are unnecessary for the economic and historical behavior requested.

## 6.2 Crop model

For v1, use a lightweight daily crop model rather than a flat annual yield roll.

A practical structure is:

\[
\text{daily growth}
=\text{potential growth}
\times \min(f\_{\text{water}},f\_{\text{nutrients}})
\times f\_{\text{temperature}}
\times f\_{\text{competition}}.
\]

Accumulate biomass, then convert it to harvestable material with a crop- and condition-dependent harvest fraction. Establishment failure and harvest losses should be separate from growth.

This is a deliberate simplification: stresses interact, and the factors are not independent causal estimates. Calibrate whole-system behavior rather than multiplying a collection of unrelated “historical bonuses.”

**AquaCrop** is a useful reference for water-limited crop production; **APSIM** is useful for crop–soil–management interactions. Use them to inform or fit a reduced model, not as unmodified historical truth. Modern cultivar parameters cannot simply become Neolithic parameters. [FAOHome](https://www.fao.org/aquacrop/overview/en)

## 6.3 Scheduling and performance

For a 10k–50k-person simulation, I recommend:

* Daily or event-triggered field growth and task updates.
* Seasonal household planning with replanning after major shocks.
* Crop cohorts per managed parcel, rather than per plant.
* Water-network calculations by connected irrigation system.
* Batched processing and storage-lot updates.
* Individual movement and visible work driven by the resulting task assignments.

These are architectural recommendations, not benchmarked performance claims.

The Rust kernel should own the authoritative material and task state. UE5 should render sowing, hoeing, water lifting, threshing, milling, herding, and hauling from that state rather than maintain a competing agricultural simulation.

## 6.4 What to simplify—and what not to simplify

**Simplify detailed chemistry before simplifying seasonal labor.** A root-zone water bucket and a few nutrient stocks will usually contribute more to historical plausibility than elaborate plant biochemistry attached to generic annual “farmer jobs.”

**Simplify crop diversity through templates, not through identical crops.** Wheat, millet, rice, maize, potato, cassava, taro, and banana can share a growth framework while retaining different propagation, harvest, water, storage, and processing behavior.

**Do not simplify all storage into one decay rate.** Keep at least commodity type, moisture condition, protection, duration, and catastrophic risk.

**Do not simplify knowledge into instantaneous access.** A mill needs a miller, a draft team needs trained animals, and a new staple needs viable propagules and preparation knowledge.

## 6.5 Existing models and games worth borrowing from

| Reference | Useful component | What not to copy uncritically |
| --- | --- | --- |
| **FAO AquaCrop** | Transparent connection between water, growth, and harvest. | Modern parameters and assumptions of relatively uniform managed fields. [FAOHome](https://www.fao.org/aquacrop/overview/en) |
| **APSIM** | Crop sequences, soil water and nutrient interactions, management events. | Complexity beyond TCE’s required resolution. [APSIM](https://www.apsim.info/) |
| **Lansing–Kremer Bali model** | Coordination emerging from water and pest interactions. | Treating one institutional explanation as universal across irrigation societies. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/abs/10.1525/aa.1993.95.1.02a00050) |
| **Farthest Frontier** | Legible rotation planning, field maintenance, seasonal labor conflicts. | Its requirement to route grain through windmills and bakeries is inappropriate for TCE’s early food history; household grinding and other grain foods must remain possible. [Farthest Frontier](https://www.farthestfrontier.com/guide/gameplay/farming/) |
| **Vintage Story** | Visible crop requirements, soil nutrients, seeds, climate sensitivity. | Its simplified nutrient-consumption and replenishment rules should not substitute for material balances. [Vintage Story Wiki](https://wiki.vintagestory.at/Farming) |

---

# 7. Sources, datasets, and remaining uncertainty

## Highest-value research foundations

The strongest foundation is the combination of **archaeobotany and zooarchaeology for sequences**, **historical accounts for production**, and **agronomy for mechanisms**.

Fuller and colleagues’ comparative domestication research is useful for gradual change and multiple trajectories; Larson and Fuller for animal-management pathways; Denham and colleagues for New Guinea; Piperno and colleagues for early maize; and the regional African studies for sorghum and pearl millet. None should be treated as a single universal chronology. [PubMed](https://pubmed.ncbi.nlm.nih.gov/24753577/)

For processing, the granary, bread, Jiahu beverage, and cheese studies provide unusually concrete archaeological anchors. They are especially valuable for preventing false prerequisite chains. [PNAS](https://www.pnas.org/doi/10.1073/pnas.0812764106)

## Datasets to use in calibration

| Dataset or source | Best use | Main limitation |
| --- | --- | --- |
| **Campbell’s English crop-yield database, 1211–1491** | Crop-specific variation, annual fluctuations, historical seed and yield accounting. | Institutional and geographical selection; not a global sample. [Queen's University Belfast](https://pure.qub.ac.uk/en/datasets/three-centuries-of-english-crop-yields-1211-1491/) |
| **Rothamsted e-RA / Broadbalk** | Long-run nutrient, manure, soil, management, and yield relationships. | One experimental setting with changing treatments and cultivars; not an ancient farm. [Rothamsted ERA](https://www.era.rothamsted.ac.uk/Broadbalk) |
| **FAOSTAT crop production, area, and yield series** | Modern regional contrasts and upper/lower plausibility checks. | National aggregation; harvested area differs from physical agricultural land. [FAOHome](https://www.fao.org/faostat/en/) |
| **HYDE 3.2** | Broad historical land-use scenarios and spatial consistency checks. | Reconstructed land use, not direct observation of ancient field productivity. [ESSD](https://essd.copernicus.org/articles/9/927/2017/) |
| **FAO farm-power and post-harvest studies** | Operation-level labor, processing stages, storage conditions, loss accounting. | Modern observations often contain improved crops or manufactured inputs. [VTechWorks](https://vtechworks.lib.vt.edu/bitstreams/30b9f35b-2715-4acb-895d-638f9c206d21/download) |

## Claims that should remain explicitly uncertain

**First appearances:** perishable tools, fermentation, fallow, terracing, detoxification, and several forms of water control have uneven archaeological visibility. Earliest surviving evidence is not an invention date.

**Historical labor productivity:** accounts frequently omit domestic processing, women’s and children’s work, animal maintenance, transport, or seasonal unemployment. An apparently precise “people fed per farmer” estimate may hide most of the relevant labor.

**Regional yield comparisons:** medieval European account series are more readily quantified than many tropical and American systems. That asymmetry in documentation must not become an assumption that other regions had simpler or less productive agriculture.

**Large historical explanations:** irrigation did not invariably produce despotism; population pressure did not invariably precede domestication; and a new implement did not independently cause an entire agricultural transformation. Eastern North American domestication and early Peruvian irrigation are particularly useful checks against overly deterministic models. [DOI](https://doi.org/10.1086/659645)

**Bottom line:** TCE’s agricultural history will become convincing when innovations change what people must physically do, what resources they can access, and which risks they can withstand. A hoe, a canal, a new bean variety, a granary, and a watermill should each produce a different kind of economic transformation—not five versions of the same food bonus.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92853-4bc4-83ea-a0eb-fa767590dbfb)
