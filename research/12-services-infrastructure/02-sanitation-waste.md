# Sanitation and waste management for The Civilization Engine

## Executive conclusion

**Model sanitation as a chain of material transfers and human exposures—not as a citywide cleanliness score or a circular “pollution radius.”** The essential chain is:

**Production → deposition → containment → collection → treatment or reuse → environmental transport → exposure → infection → further shedding.**

This follows the end-to-end approach of modern sanitation-health guidance, but it also accommodates historical systems: household pits, communal latrines, commercial night-soil collection, agricultural recycling, street-cleaning obligations, and sewers that discharge untreated waste. A system can succeed at one stage while failing at another. [World Health Organization](https://www.who.int/publications/i/item/9789241514705)

For TCE, the most important distinction is **removal versus neutralization**. A collector can clean a courtyard while contaminating a field; a sewer can protect one neighborhood while endangering a downstream water intake; a well-maintained pit can reduce direct contact while threatening a hydraulically connected well. These are not exceptional edge cases—they are central design problems identified in sanitation research. [IADB Publications](https://publications.iadb.org/en/sfd-promotion-initiative-cap-haitien-haiti)

The recommendations below distinguish **observed evidence**, **engineering guidance**, and **proposed simulation assumptions**. Historical evidence is much stronger for the existence and organization of sanitation systems than for universal collection costs, disease rates, or safe separation distances.

---

## 1. Mechanisms: rules a simulation can implement

### 1.1 Separate waste quantity, resource value, and infectiousness

Use several material streams rather than a single “waste” resource:

| Stream | Suggested properties | Why keep it separate? |
| --- | --- | --- |
| Feces | Wet and dry mass, nutrients, pathogen-specific shedding | Infectiousness depends on the producer’s infection state, not just mass. |
| Urine | Liquid volume, nitrogen, phosphorus, potassium, possible contamination | Important hydraulic and fertilizer load; should not automatically inherit the same pathogen composition as feces. |
| Household wastewater | Water, organic matter, contamination acquired during use | Adds transport volume and can redistribute fecal contamination. |
| Organic refuse | Food scraps, plant material, animal remains | Can decompose, attract animals, and contribute fertilizer without necessarily carrying human enteric pathogens. |
| Ash, rubble, broken objects | Bulk, recoverable materials, obstruction potential | Can fill pits and drains without behaving like sewage. |
| Animal manure | Nutrients, organic matter, host-specific pathogens | Connects husbandry, agricultural fertility, and zoonotic risks. |

This is a **proposed TCE schema**. Its empirical foundation is that excreta composition varies substantially with food and fluid intake, while historical refuse streams could be dominated by economically useful materials such as coal ash. A kilogram of refuse is therefore not a stable unit of either infection risk or disposal cost. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26246784/)

**Implementation rule:** attach pathogen loads to infected or carrier agents’ excretion events. Do not spontaneously generate a particular infection merely because a midden becomes large. Any environmental reservoir or outside introduction should be an explicit part of that pathogen’s ecology.

### 1.2 A toilet is an interface; its destination determines much of its effect

Distinguish a latrine’s user-facing features from what happens beneath or beyond it. A private privy, communal toilet, or household chamber pot can feed a pit, removable container, drain, sewer, field, or surface dump.

Pit latrines generally separate people from exposed feces, but they are not necessarily treatment systems. Structural lining can prevent collapse without making a pit watertight. Liquid infiltration, solids accumulation, groundwater conditions, and the ability to empty or replace the pit all matter. [Water Pathogens](https://www.waterpathogens.org/book/pit-toilets-latrines)

**Implementation rules:**

* Choose a deposition location using travel time, urgency, access rights, fees, privacy, safety, cleanliness, and habit.
* Facility availability is not equivalent to use. An inaccessible, overflowing, costly, or unpleasant toilet should lose users.
* Model children’s waste disposal through caregivers as well as independent toilet use.
* Distinguish **usable capacity** from nominal excavation volume.
* When a pit fills, the owner must pay for emptying, construct another pit, reduce use, or tolerate overflow and alternative deposition.

Urban crowding makes replacement increasingly difficult: net sludge accumulation and lack of land can turn a workable household technology into an unresolved municipal service problem. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4743102/)

### 1.3 Contamination follows several different networks

A useful representation is the following pathway table. The final column gives proposed TCE behavior.

| Pathway | Physical mechanism | TCE implication |
| --- | --- | --- |
| **Hands, containers, household surfaces** | Waste handling contaminates objects subsequently involved in eating or water storage. | Track household and occupational contact, not only outdoor pollution. |
| **Groundwater** | Leachate moves through soil and aquifers; attenuation depends on material, saturation, travel time, and organism. | Use directed source-to-well connections with delays and attenuation. |
| **Surface runoff and flooding** | Rain mobilizes deposits; floodwater enters pits, courtyards, drains, and water sources. | Activate transfers through drainage catchments during rain and floods. |
| **River or canal transport** | Upstream discharge moves toward downstream abstractions. | Sewer outlets and drinking-water intakes must share a flow network. |
| **Food and agricultural soil** | Contaminated fertilizer, irrigation, handling, or preparation creates ingestion or soil-contact exposure. | Connect field treatment, crop type, harvest timing, markets, and cooking. |
| **Animals and insects** | Waste provides habitat or opportunities for mechanical transfer; some diseases have additional host or vector requirements. | Keep vector habitat distinct from generic “disease points.” |

Multiple-pathway exposure assessment is important empirically: SaniPath studies find that dominant routes differ between neighborhoods, and food pathways can outweigh drinking-water exposure. Groundwater research likewise finds that pit-related contamination cannot be explained reliably by distance alone. [PubMed](https://pubmed.ncbi.nlm.nih.gov/32530933/?utm_source=chatgpt.com)

**For wells, preserve four distinctions:** groundwater direction, vertical separation above the highest water table, subsurface permeability or fractures, and surface ingress at the wellhead. A nearby protected well can outperform a more distant but poorly protected or strongly connected source. A distance threshold should be a siting rule people believe or enforce—not the underlying disease physics.

### 1.4 Collection is a logistics service with a destination

Waste collection requires containers, access, labor, loading, travel, unloading, and somewhere to put the material. Removing a load from a household should transfer it to a worker or vehicle—not delete it.

Historical collection could be commercially attractive. In nineteenth-century London, dust-yard operators recovered saleable fractions from ash-rich household refuse. In East Asian night-soil systems, agricultural demand helped support collection and transport of human excreta. Neither arrangement required that disposal be primarily a tax-funded municipal service. [PubMed](https://pubmed.ncbi.nlm.nih.gov/19121575/)

**Implementation rule:** let the value of recoverable nutrients and materials offset collection costs. When fertilizer demand is strong and transport cheap, collectors may pay for access to waste. When its resale value falls or hauling distance rises, households or governments must pay for removal.

This produces a useful emergent transition: a once-profitable recycling system can become an expensive public-health obligation without any change in human waste production.

### 1.5 Sewers relocate loads; treatment changes them

Represent a sewer as a conveyance network with hydraulic capacity, slope or pumping requirements, blockage, leakage, maintenance, and an outlet. A connection does not itself destroy pathogens or nutrients.

Stormwater and wastewater networks also interact with rainfall. The EPA’s SWMM explicitly models runoff, pollutant buildup and wash-off, pipe networks, surcharge, and flooding—precisely the processes a simplified TCE drainage system should preserve. [US EPA](https://www.epa.gov/water-research/storm-water-management-model-swmm)

**Implementation rules:**

* Add household discharge to the receiving drain or sewer.
* Transfer it to the outlet unless leakage, overflow, settling, or treatment intervenes.
* Allow refuse and sediment to reduce conveyance capacity.
* Make drain clearance and sewer maintenance recurring work.
* Allow new water supplies or flushing practices to overload older containment systems through increased liquid inflow.

A cleaner street and a dirtier downstream river should be a possible outcome of the same infrastructure investment.

### 1.6 Agricultural reuse creates both a nutrient cycle and an exposure cycle

Human excreta contain substantial recoverable nutrients. Urine is especially useful as a rapidly available nitrogen source, while fecal material also contributes phosphorus and organic matter. Application should follow crop nutrient demand rather than a fixed “compost gives +20% yield” bonus. [SLU](https://research.slu.se/en/publications/guidelines-on-use-of-urine-and-faeces-in-crop-production/)

But fertilizer value and pathogen safety are separate properties. Storage, temperature, moisture, pH, treatment, crop handling, and exposure determine the latter. An actively used pit continually receives fresh material; its contents cannot all be assigned the age of the pit itself. [Water Pathogens](https://www.waterpathogens.org/book/pit-toilets-latrines)

**Implementation rules:**

* Track material in age cohorts or batches.
* Preserve nutrient mass through collection and treatment, with explicit losses and transfers.
* Let application increase crop growth only when the relevant nutrients are limiting.
* Transfer viable pathogens to agricultural soil or crops separately.
* Allow different risks for workers, consumers of raw produce, and consumers of cooked foods.
* Do not treat “aged,” “dry,” or “composted” as synonymous with sterile.

For soil-transmitted helminths, maturation matters as well as survival: WHO describes roughly **2–4 weeks** for eggs to become infective under favorable conditions. Fresh contamination can therefore create delayed rather than immediate risk. [World Health Organization](https://www.who.int/news-room/questions-and-answers/item/soil-transmitted-helminths)

### 1.7 Institutions govern access, externalities, and who performs dangerous work

Historical sanitation was not simply absent until germ theory. Research on late-medieval England and Scandinavia describes cooperation between civic authorities and residents in maintaining streets and gutters. Such arrangements combined infrastructure with obligations governing behavior. [ResearchGate](https://www.researchgate.net/publication/23295919_Cooperative_Sanitation_Managing_Streets_and_Gutters_in_Late_Medieval_England_and_Scandinavia)

For TCE, plausible institutional rules include household frontage-cleaning duties, designated disposal grounds, restrictions on dumping into watercourses, pit-maintenance obligations, collection concessions, user fees, and publicly financed street cleaning. These should require jurisdiction, labor, enforcement, and compliance—not merely enactment.

Keep the workers visible. Modern assessments document sanitation workers’ exposure to infection, injury, hazardous working conditions, and stigma. Model occupational risks and restrictions through working conditions and social institutions, never as intrinsic properties of a population group. [World Health Organization](https://www.who.int/publications/m/item/health-safety-and-dignity-of-sanitation-workers)

A government may initially act because of smell, blocked streets, disputes, or visible illness. **Beliefs about the cause of disease should be separate from the actual transmission model.**

---

## 2. Parameters: defensible values, ranges, and limits

**Confidence key:** **H** = strong support for the stated measurement or formal requirement; **M** = useful evidence or guidance with substantial context dependence; **L** = thin evidence or weak transferability. High confidence in a modern measurement does **not** imply high confidence when applying it to an ancient population.

### 2.1 Production and accumulation

| Parameter | Quantitative anchor | Units | Appropriate use and limitations | Confidence / source |
| --- | --- | --- | --- | --- |
| Fresh fecal production | **128** median | g wet mass/person/day | Central starting point; adjust for age, diet, and illness. | **H** for compiled median; **M** historical transfer. Rose et al., 2015. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26246784/) |
| Fecal dry solids | **29** median | g dry mass/person/day | Useful for separating water from material that degrades or accumulates. | **H/M**. Rose et al. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26246784/) |
| Urine production | **1.42** median | L/person/day | Hydraulic load; varies with fluid intake, environment, and exertion. | **H/M**. Rose et al. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26246784/) |
| Defecation frequency | **1.20** average reported | events/person/day | Schedule anchor, not a deterministic once-daily requirement. | **M**. Rose et al. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26246784/) |
| Pit sludge accumulation | **40–90** design range | L/person/year | **Net accumulated volume**, not raw excretion volume; cleansing materials and conditions matter. | **M**. Nakagiri et al., 2016. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4743102/) |
| Modern household refuse: Kampala example | **0.43–0.98** parish means, low- to high-income | kg/person/day | One-week measurements from **103 households**; useful modern comparison, not an agrarian default. | **M**, low historical transfer. `solidwastekampala`. [Open Wash Data](https://openwashdata.org/) |
| Historical collection productivity and cost | **No defensible universal rate identified** | loads/worker-day; labor-days; money | Derive from containers, access, haulage, distance, wages, and destination costs. | **L** across eras and regions. |

**Calculated TCE scale check:** applying the feces and urine medians uniformly gives approximately **1.28–6.4 metric tonnes of wet feces daily** and **14.2–71 m³ of urine daily** for 10,000–50,000 people. These are arithmetic planning scenarios, not measurements of historical cities; a realistic age and diet distribution changes them.

**Calculated pit example:** a pit with **2 m³ usable sludge capacity**, serving **six people**, lasts about:

\[
\frac{2}{6\times0.09}=3.7
\quad\text{to}\quad
\frac{2}{6\times0.04}=8.3\text{ years}.
\]

Do not subtract decomposition again when using a **net** accumulation rate. Conversely, a watertight tank receiving all liquid needs a hydraulic filling model and can fill much faster.

### 2.2 Contamination, maturation, and treatment

These numbers are **simulation anchors, not practical assurances that a particular real-world installation is safe**.

| Parameter | Quantitative evidence or guidance | Interpretation for TCE | Confidence / source |
| --- | --- | --- | --- |
| Common pit–water-source horizontal setback | **30 m** appears frequently in reviewed guidance | A regulatory heuristic, **not a guaranteed safe radius**. | **M**. Graham & Polizzotto, 2013. [ResearchGate](https://www.researchgate.net/publication/236070782_Pit_Latrines_and_Their_Impacts_on_Groundwater_Quality_A_Systematic_Review) |
| Vertical separation | Common guidance around **1.5–2 m** between pit base and highest groundwater | Use seasonal maximum water table, not only current conditions. | **M**, site-dependent. Graham & Polizzotto. [ResearchGate](https://www.researchgate.net/publication/236070782_Pit_Latrines_and_Their_Impacts_on_Groundwater_Quality_A_Systematic_Review) |
| Observed bacterial-indicator migration in reviewed studies | From **under 1.5 m to about 25 m** in particular investigations | Demonstrates variability; **25 m is not an upper physical limit**, and indicators are not every pathogen. | **M**, heterogeneous studies. Graham & Polizzotto. [ResearchGate](https://www.researchgate.net/publication/236070782_Pit_Latrines_and_Their_Impacts_on_Groundwater_Quality_A_Systematic_Review) |
| Soil-transmitted helminth egg maturation | Approximately **2–4 weeks** in favorable conditions | Separate newly deposited eggs from infective eggs; do not apply this delay to all pathogens. | **H** for broad life-cycle description; **M** for timing. WHO. [World Health Organization](https://www.who.int/news-room/questions-and-answers/item/soil-transmitted-helminths) |
| Resting pit or stored fecal material | **1–2 years** is an important storage timescale in guidance | Risk reduction, not a universal sterilization deadline; resistant organisms may persist. | **M**. Global Water Pathogen Project. [Water Pathogens](https://www.waterpathogens.org/book/pit-toilets-latrines) |
| Controlled in-vessel or static-aerated sludge composting | **≥55°C for ≥3 days** | Modern U.S. pathogen-reduction process criterion; not the behavior of an unmanaged midden. | **H** as a formal criterion. 40 CFR 503, Appendix B. [eCFR](https://www.ecfr.gov/current/title-40/chapter-I/subchapter-O/part-503/appendix-Appendix%20B%20to%20Part%20503) |
| Controlled windrow sludge composting | **≥55°C for ≥15 days**, with **≥5 turnings** | Turning and temperature history matter; a warm core alone is insufficient evidence of whole-batch treatment. | **H** as a formal criterion. [eCFR](https://www.ecfr.gov/current/title-40/chapter-I/subchapter-O/part-503/appendix-Appendix%20B%20to%20Part%20503) |
| Sludge pasteurization | **≥70°C for ≥30 minutes** | Another treatment benchmark, not a complete statement of regulatory compliance or reuse safety. | **H** as a formal criterion. [eCFR](https://www.ecfr.gov/current/title-40/chapter-I/subchapter-O/part-503/appendix-Appendix%20B%20to%20Part%20503) |

**Do not assign one universal pathogen half-life.** Temperature, moisture, sunlight, matrix, and organism can change persistence profoundly. Use pathogen-specific parameters and sensitivity tests rather than interpreting the storage values above as hard survival cutoffs.

### 2.3 Nutrient recovery

An illustrative dataset reported in SEI’s *Practical Guidance on the Use of Urine in Crop Production* concerns people aged **10 years and older in rural Limpopo, South Africa**. Values below are annual excretion, not necessarily nutrients ultimately delivered to roots. Confidence is **M** as a context-specific calibration anchor. [SEI](https://www.sei.org/mediamanager/documents/Publications/SEI-Book-Stenstrom-PracticalGuidanceOnTheUseOfUrineInCropProduction.pdf)

| Stream | Nitrogen | Phosphorus | Potassium |
| --- | --- | --- | --- |
| Urine | **3.56 kg/person/year** | **0.34 kg/person/year** | **1.26 kg/person/year** |
| Feces | **0.42 kg/person/year** | **0.24 kg/person/year** | **0.21 kg/person/year** |
| Total | **3.98 kg/person/year** | **0.58 kg/person/year** | **1.47 kg/person/year** |

For an additional agronomic scale check, urine-use guidance suggests that one person’s annual urine can fertilize roughly **300–400 m²** at example nitrogen application rates of **50–100 kg N/ha**. This is neither a universal application rate nor the area needed to feed that person. Diet, crop demand, collection losses, storage, and application method all change the result. [SLU](https://research.slu.se/en/publications/guidelines-on-use-of-urine-and-faeces-in-crop-production/)

**Recommended nutrient rule:** calculate fertilizer value from recoverable nutrients and local crop demand. Reduce delivered nutrients through explicit losses; reduce viable pathogens through a separate treatment calculation.

---

## 3. Variation across eras and world regions

The historical record supports **multiple configurations**, not an inevitable sequence from “primitive pits” to “advanced sewers.”

| Context | Evidence and characteristic arrangements | Implication for TCE |
| --- | --- | --- |
| **Foragers and repeatedly occupied camps** | The sources reviewed do not justify one universal forager sanitation regime or numerical disease baseline. Mobility, duration of occupation, and repeated use of particular places need to be distinguished. | Treat camp relocation and deposition patterns as behavioral possibilities. Do not grant all foragers automatic cleanliness or assume all were highly mobile. |
| **Early farming: Çatalhöyük, Anatolia** | Research documents middens containing mixtures of domestic waste, ash, and fecal material. Whipworm was identified in two human coprolites; that demonstrates infection, not population prevalence. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/710134) | Repeated occupation creates persistent waste reservoirs and parasite transmission opportunities without requiring a large city. |
| **Indus urbanism, third millennium BCE** | Archaeological syntheses describe household sanitation, pits or sumps, and covered brick drains in Harappan settlements. Their presence should not be equated with universal household coverage or modern wastewater treatment. [DOI](https://doi.org/10.5194/hess-24-4691-2020) | Covered drainage and organized maintenance can emerge early where construction skills, water, density, and institutions support them. |
| **Roman Mediterranean, including North Africa and western Asia** | Latrines, sewers, and water infrastructure coexisted with archaeological evidence of intestinal parasites. Evidence of parasite presence does not establish that infrastructure had no benefit. [Cambridge University Press](https://www.cambridge.org/core/journals/parasitology/article/human-parasites-in-the-roman-world-health-consequences-of-conquering-an-empire/6464BDBB5D4B8EC0B08C503B6ECD1B7B) | Infrastructure should reduce particular routes while leaving others—food, handling, soil, or untreated discharge—open. |
| **China, especially later imperial periods** | Agronomic and historical research describes human excrement as an agricultural resource, with commercial collection particularly developed in later periods. Terminology in early texts can be ambiguous: “manure” or refuse does not always establish human night-soil use. [Hep Journals](https://journal.hep.com.cn/sel/EN/10.1007/s42832-024-0226-6) | Strong fertilizer demand can sustain urban–rural nutrient trade. Do not require municipal sewer construction before dense urban life is possible. |
| **Japan, early modern through twentieth century** | Collection and agricultural reuse remained important long after sophisticated cities developed. The Japan Sanitation Consortium reports sewer coverage of only about **6% in 1958**, followed by major changes as agricultural demand for night soil declined. [Japan Sanitation Consortium](https://jsanic.org/wp/home2/sanitation-asia-pacific/history-sanitation-japan/) | Sanitation technology and sanitation service organization can change at different speeds. Fertilizer substitution can destabilize an established collection economy. |
| **Amazonia: Upper Xingu and related dark-earth research** | Research combining archaeology and contemporary Indigenous practices supports intentional enrichment of some settlement soils with organic residues, ash, and charcoal. This does not establish that all dark earth was deliberately created or that all inputs were human feces. [Alice](https://www.alice.cnptia.embrapa.br/alice/handle/doc/1156858) | Model domestic refuse as a potential soil resource, but keep fertility, waste composition, and infection risk separate. |
| **Late-medieval England and Scandinavia** | Street and gutter management involved cooperation between authorities and residents, rather than either complete municipal provision or complete neglect. [ResearchGate](https://www.researchgate.net/publication/23295919_Cooperative_Sanitation_Managing_Streets_and_Gutters_in_Late_Medieval_England_and_Scandinavia) | Household obligations, collective maintenance, and civic enforcement can arise before modern scientific explanations of disease. |
| **Industrializing European and North American cities** | London’s ash-rich waste supported recovery markets; water and sewer investments elsewhere became major components of public-health infrastructure. These were distinct material and institutional transitions. [PubMed](https://pubmed.ncbi.nlm.nih.gov/19121575/) | Fuel changes alter refuse composition; water networks alter wastewater volumes; cities can outgrow profitable informal recovery arrangements. |
| **Modern cities across Africa, Asia, and the Americas** | Sewerage, onsite containment, emptying services, and informal arrangements coexist. Sanitation-chain assessments explicitly distinguish containment from transport, treatment, and final disposal. [IADB Publications](https://publications.iadb.org/en/sfd-promotion-initiative-cap-haitien-haiti) | Do not force every modern settlement into a sewer-only endpoint. Well-managed decentralized systems and badly managed sewers must both be possible. |

**The transferable variables are density, land availability, hydrology, agricultural demand, haulage costs, labor organization, and political capacity—not calendar date or a civilization-wide sanitation level.**

---

## 4. Stylized facts and quantitative validation targets

These are patterns a credible simulation should be able to reproduce. They are not coefficients to paste directly into every world.

### Water-source membership can matter more than residential proximity

In John Snow’s comparison during the first seven weeks of London’s 1854 cholera epidemic, the reported rates were approximately **315 cholera deaths per 10,000 houses** supplied by Southwark and Vauxhall versus **37 per 10,000 houses** supplied by Lambeth—roughly an eight- to ninefold contrast. **The denominator was houses, not people.** This historical observational comparison is best used to validate source-linked outbreak geography, not as a universal contaminated-water multiplier. [John Snow Museum](https://epi-snow.ph.ucla.edu/Stream3_GrandExperiment_d.html)

**TCE target:** households using the same source can share risk even when separated geographically; adjacent households using different sources can experience different outcomes.

### Water protection and waste disposal can be complementary

Alsan and Goldin’s Massachusetts analysis for **1880–1920** found that clean water and effective sewerage were complementary and together accounted for approximately **one-third of the decline in log under-five mortality** in their setting. [Claudia Goldin](https://goldin.scholars.harvard.edu/publications/watersheds-infant-mortality-role-effective-water-and-sewerage-infrastructure)

**TCE target:** closing one transmission route may produce limited gains while other major routes remain open. Combined interventions can outperform isolated ones.

### Sanitation interventions help, but their effects are heterogeneous

The 2023 Cochrane review included **51 studies**. Across cluster-randomized trials, the pooled all-age diarrhea prevalence risk ratio was **0.85**, with a **95% confidence interval of 0.76–0.95**. Pooling randomized and non-randomized designs gave **0.74**, with **0.67–0.82** confidence limits. Evidence certainty and effects varied by intervention and setting. These are diarrhea outcomes—not equivalent reductions in all-cause mortality. [Cochrane](https://www.cochrane.org/evidence/CD013328_interventions-improve-sanitation-preventing-diarrhoea)

**TCE target:** adding toilets should not impose an unconditional mortality bonus. Actual use, coverage, emptying, treatment, and alternative exposures determine the result.

### Food exposure can remain important after drinking water improves

SaniPath assessments across **45 neighborhoods in ten cities** found substantial variation in exposure patterns, with food pathways commonly dominant in the studied low- and lower-middle-income settings. These measurements used **E. coli as an indicator of fecal contamination**, not direct counts of every disease-causing organism. [DOI](https://doi.org/10.1016/j.scitotenv.2021.151273?utm_source=chatgpt.com)

**TCE target:** safe wells alone do not necessarily eliminate enteric infection when food preparation, markets, fertilizer, or irrigation remain contaminated.

### Impressive infrastructure can coexist with persistent parasites

Roman archaeological evidence records intestinal parasites despite sophisticated water and sanitation infrastructure. However, archaeological presence/absence data cannot reliably establish population-wide infection rates or the counterfactual without that infrastructure. [Cambridge University Press](https://www.cambridge.org/core/journals/parasitology/article/human-parasites-in-the-roman-world-health-consequences-of-conquering-an-empire/6464BDBB5D4B8EC0B08C503B6ECD1B7B)

**TCE target:** sanitation should change the mixture and intensity of risks, not necessarily drive all infections to zero.

### Recycling systems can fail because their economics change

Japan’s historical reliance on night-soil collection, followed by declining agricultural demand, illustrates that disposal obligations can remain while resource revenues disappear. [Japan Sanitation Consortium](https://jsanic.org/wp/home2/sanitation-asia-pacific/history-sanitation-japan/)

**TCE target:** introducing alternative fertilizer, increasing hauling distance, or changing land use can create a sanitation crisis unless fees, public funding, or treatment capacity adapt.

---

## 5. Recommended representation in TCE

The following is a **proposed architecture**, not an already validated historical model.

### 5.1 Keep people individual; aggregate waste physically

| Entity | Minimum useful state |
| --- | --- |
| **Person** | Excretion budget, infection and shedding state, facility choices, water-source use, food consumption, hygiene behavior, occupational contacts. |
| **Household or workplace** | Occupants and visitors, waste storage, water-storage contamination, access to toilets and collection, payment capacity. |
| **Latrine, pit, or tank** | Usable volume, sludge and liquid stores, permeability, structural condition, user load, overflow state, material-age cohorts. |
| **Collector and vehicle** | Capacity, route, access constraints, load composition, destination, payment, contact protection. |
| **Midden or dump** | Composition, age, moisture, drainage connection, organic decomposition, pathogen stores, animal access. |
| **Treatment facility or batch** | Temperature history, residence time, mixing or turning, throughput, process failures, nutrients retained and pathogens remaining. |
| **Field** | Nutrient pools, application history, viable pathogens, crop characteristics, worker contact, harvest destinations. |
| **Water source and drainage network** | Water storage or flow, upstream inputs, well protection, groundwater connections, pathogen concentration. |
| **Institution** | Ownership, jurisdiction, budget, service contracts, duties, inspection, sanctions, political support. |

A citizen should remain traceable as a contributor and exposed individual, but there is no need to create persistent entities for every fecal deposit or microorganism. Combine loads into household, facility, field, or environmental stores.

Preserve **where excretion occurs**. Assigning every person’s output to their home ignores workplaces, markets, public latrines, travel, and labor camps.

### 5.2 Use a mass-conserving transfer ledger

For each material component, record:

\[
\text{next stock}
=
\text{current stock}
+\text{inputs}
-\text{physical exports}
-\text{transformations}.
\]

“Collected” is an export to a cart. “Discharged” is an export to a river or field. Treatment can reduce viable pathogens while retaining much of the nutrient mass. Organic decomposition changes chemical forms rather than making every constituent disappear.

This ledger should support a player-facing explanation such as:

> This well received contamination from Pit 184 after groundwater rose. Four households drank from it. Two infections subsequently contributed additional waste to the same catchment.

That causal trace is more useful than a neighborhood label such as “sanitation: poor.”

### 5.3 Represent groundwater through sparse, directional connections

For a first version, use exact facility coordinates plus a coarse hydrological representation. A **10–25 m surface grid** is a reasonable *engineering starting assumption to benchmark*, not an empirically established optimal resolution.

Precompute a limited set of plausible source-to-receptor connections. Each connection carries a travel time and a capture or attenuation factor. Recompute relevant connections when wells, major earthworks, drainage, or groundwater conditions change.

An illustrative arrival-load approximation is:

\[
A\_{w,p}(t)
=
\sum\_s
f\_{sw,p}\,
L\_{s,p}(t-\tau\_{sw})\,
e^{-k\_p\tau\_{sw}}
+
B\_{w,p}(t)
\]

where:

* \(A\_{w,p}\): arrival of viable pathogen \(p\) at well \(w\), organisms/day;
* \(L\_{s,p}\): pathogen load leaving source \(s\), organisms/day;
* \(f\_{sw,p}\): fraction reaching that receptor before the separately modeled decay;
* \(\tau\_{sw}\): travel time, days;
* \(k\_p\): approximate inactivation rate, per day;
* \(B\_{w,p}\): direct wellhead contamination, organisms/day.

Add arrivals to an explicit water store and obtain concentration from **organisms divided by liters**. Withdrawal, overflow, and flushing carry organisms out with the water.

This approximation needs calibration; a constant exponential decay rate will not fit every pathogen or environment. Its advantage is that it preserves direction, delay, attenuation, and local wellhead failure without requiring a full groundwater solver.

Ensure routing fractions and exports conserve mass. Do not send the same complete load independently to every nearby well.

### 5.4 Calculate exposure before infection

For each pathogen, add doses acquired through water, food, hands, soil, and relevant work activities. Then evaluate a pathogen-specific infection function.

An exponential function,

\[
P(\text{infection})=1-e^{-r\_p D\_{i,p}},
\]

is a possible modeling choice for pathogens and data compatible with that assumption. It is **not** a universal biological law: \(r\_p\), alternative dose-response functions, immunity, and host susceptibility require pathogen-specific calibration.

Also separate **infection, symptoms, severe illness, and death**. Otherwise every sanitation improvement becomes an implausibly immediate mortality reduction.

For an initial disease implementation, retain at least two distinct environmental behaviors:

**Rapid fecal–oral transmission:** infection creates shedding, environmental exposure causes new infections, and illness can spread through shared water or food.

**Persistent soil-associated infection:** deposition, maturation, survival, and later soil or crop exposure occur on different timescales.

Add other pathogen ecologies when the disease module can support their actual routes. Do not use a rat-population meter as a substitute for all sanitation-related illness.

### 5.5 Make sanitation an institutional and economic choice

A collector’s decision can use an expected margin:

\[
\text{margin}
=
\text{collection payments}
+\text{recovered-material sales}
-\text{labor}
-\text{transport}
-\text{treatment/disposal costs}
-\text{expected penalties}.
\]

All terms should emerge from TCE’s existing goods, transport, labor, land, and legal systems.

Useful consequences follow naturally:

* Valuable night soil attracts collection near profitable farms.
* Remote or poor households may remain unserved.
* Illegal dumping becomes attractive when legitimate disposal is costly and enforcement weak.
* Private homeowners can maintain facilities while shared spaces remain neglected.
* Landlords and tenants can disagree over who pays for emptying.
* Downstream settlements can demand restrictions on upstream discharge.
* Public collection can persist after resale value collapses, but only with adequate funding and labor.

A regulation should have **a target behavior, responsible actor, detection probability, consequence, and enforcement cost**. Passing a law should not directly clean the environment.

### 5.6 Simplify computation, not the causal chain

For 10k–50k people, I would begin with daily aggregated waste production and facility updates, event-driven collection, and additional hydrological updates during significant rainfall or flooding. Reuse citizens’ existing daily schedules to estimate where exposure occurs.

Avoid per-frame microbial simulation, all-pairs pit-to-well searches, and individual refuse-item pathfinding. Retain individual infection states, facility failures, source membership, collection jobs, and destinations.

The first playable version should include **pits and containers, collection, middens, fields, directional water contamination, and two contrasting pathogen behaviors**. Detailed sewer hydraulics, industrial toxic waste, and complex vector ecology can follow without changing the fundamental architecture.

### 5.7 Existing models and games worth borrowing from

| Model or game | Useful component | Important limitation |
| --- | --- | --- |
| **SFD: excreta-flow diagrams** | Accounts for containment, emptying, transport, treatment, and disposal; excellent template for a sanitation diagnostic screen. | A planning/accounting framework, not a dynamic disease simulator. [IADB Publications](https://publications.iadb.org/en/sfd-promotion-initiative-cap-haitien-haiti) |
| **SaniPath** | Combines environmental contamination and human contact behavior across multiple routes. | Fecal-indicator exposure is not automatically pathogen-specific infection probability. [PubMed](https://pubmed.ncbi.nlm.nih.gov/32530933/?utm_source=chatgpt.com) |
| **EPA SWMM** | Runoff, drainage networks, pollutant wash-off, surcharge, flooding, and maintenance-related hydraulic problems. | Best used for offline benchmarks or reduced-model development, not as an agent disease engine. [US EPA](https://www.epa.gov/water-research/storm-water-management-model-swmm) |
| **Farthest Frontier** | Its documented compost-yard loop connects collection from homes and livestock to disease pressure and field fertility. | Borrow visible jobs and logistics; its gameplay relationships are not epidemiological calibration data. [Farthest Frontier](https://www.farthestfrontier.com/guide/gameplay/farming/) |

Before balancing gameplay, test conservation and causality: a collection interruption, a rising water table, a protected versus breached wellhead, a sewer outlet moved upstream of an intake, and raw versus effectively treated fertilizer should each produce understandable, different outcomes.

---

## 6. Sources, datasets, and uncertainty

### Most useful research foundations

The strongest technical starting points are **Rose et al. (2015), *The Characterization of Feces and Urine*** for production; **Graham and Polizzotto (2013), *Pit Latrines and Their Impacts on Groundwater Quality*** for contamination; **Nakagiri et al. (2016)** for pit performance and filling; and **Richert et al. (2010), *Practical Guidance on the Use of Urine in Crop Production*** for nutrient recovery. Their numbers should retain the population, environmental, and methodological qualifications described above. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26246784/)

For historical interpretation, use archaeological studies alongside institutional history: **Ledger et al. on Çatalhöyük**, **Mitchell on Roman parasites**, **Jørgensen on medieval sanitation**, and **Shirai, Leisz, and Kyuma on night-soil agriculture**. For quantified health effects, use **Alsan and Goldin** and the **Bauza et al. Cochrane review**, rather than inferring mortality directly from the presence of drains or toilets. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/abs/parasite-infection-at-the-early-farming-community-of-catalhoyuk/71D77AF9D6895DE334D76721AFED65BF)

### Datasets and structured evidence

| Resource | Best use for TCE | Main restriction |
| --- | --- | --- |
| **WHO/UNICEF Joint Monitoring Programme—JMP** | Modern service levels, urban–rural differences, and inequality in access. | Service classification is not direct measurement of pathogen exposure; do not backcast modern relationships into antiquity. [WashData](https://washdata.org/data) |
| **World Bank, What a Waste 3.0** | Modern waste quantities, composition, collection, financing, and disposal arrangements. | Modern material consumption differs radically from early agrarian economies. [World Bank](https://www.worldbank.org/en/publication/what-a-waste) |
| **openwashdata, including `solidwastekampala`** | Inspectable household-scale observations for testing waste-generation heterogeneity. | Short observation windows and local samples limit generalization. [Open Wash Data](https://openwashdata.org/) |
| **SaniPath studies and associated outputs** | Exposure differences by pathway, neighborhood, and age-related behavior. | Indicator-organism data require additional assumptions to become disease models. [PubMed](https://pubmed.ncbi.nlm.nih.gov/34718001/?utm_source=chatgpt.com) |
| **City-specific SFD reports** | Realistic combinations of onsite sanitation, collection failures, treatment, and disposal. | Often combine administrative records, interviews, and estimates; preserve their uncertainty. [IADB Publications](https://publications.iadb.org/en/sfd-promotion-initiative-cap-haitien-haiti) |

### Claims to treat cautiously

**Archaeological detection is not prevalence.** Coprolites, latrine deposits, and burial soils have different preservation and sampling biases. Two positive samples cannot establish the infection rate of a settlement, and failure to detect a parasite does not establish its absence. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/abs/parasite-infection-at-the-early-farming-community-of-catalhoyuk/71D77AF9D6895DE334D76721AFED65BF)

**A law is not evidence of universal compliance.** Institutional records reveal obligations, disputes, and attempts at governance more readily than the daily cleanliness of every street. Avoid turning the existence of sanitation ordinances into a binary “regulated therefore clean” variable. [ResearchGate](https://www.researchgate.net/publication/23295919_Cooperative_Sanitation_Managing_Streets_and_Gutters_in_Late_Medieval_England_and_Scandinavia)

**Early night-soil claims and universal recycling narratives require scrutiny.** Historical terminology can blur human excrement, animal manure, and general refuse. Fertility gains do not establish pathogen safety, and a valuable recycling trade does not prove that every neighborhood was served. [J-STAGE](https://www.jstage.jst.go.jp/article/sanitation/advpub/0/advpub_00002/_article/-char/en)

**The largest remaining calibration gaps are historical labor productivity, collection prices, actual service coverage, and pathogen-specific transport and survival in particular environments.** The reviewed evidence supports local anchors and mechanism-based sensitivity analysis—not one reliable parameter set for all societies.

**The priority for TCE is therefore straightforward: track where each load goes, whether it remains infectious, who encounters it, and who has an incentive—or obligation—to move or treat it.** With those mechanisms intact, clean streets, contaminated wells, valuable fertilizer, sanitation markets, public services, and institutional crises can arise from the same underlying simulation rather than from scripted historical stages.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9292c-9f54-83ea-9d97-cc5129b499d7)
