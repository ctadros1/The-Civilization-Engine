# The urban mortality penalty and sanitation history

## A simulation-ready report for The Civilization Engine

**TCE should make cities dangerous when population, waste production, and connectivity outgrow effective public-health capacity—not simply when population crosses a threshold.** Water supply, sewage disposal, household practices, crowding, and disease introductions should determine the penalty. Institutions can then reduce it through several technological and organizational paths.

Two distinctions are essential. **A city can have higher mortality than the countryside while still producing more births than deaths.** And **a sewer or waterworks is not itself a health improvement:** the outcome depends on water quality, actual coverage, maintenance, household use, and where the waste goes. Historical research provides particularly strong evidence for the importance of functioning, complementary water and sewerage systems. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.12964)

The numbers below are divided into **historical calibration targets**, **estimated intervention effects**, and **explicit modeling choices**. They should not be treated as interchangeable coefficients.

---

## 1. Mechanisms: what made cities deadlier?

### 1.1 Exposure, rather than city size, is the central causal variable

Historical urban disadvantages included concentrated human and animal waste, opportunities for infection, and frequent introductions through trade and migration. Feeding practices and migrants’ previous disease exposure also mattered. These mechanisms varied substantially between places. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.12964)

For TCE, implement the following causal pathways.

| Mechanism | Implementable rule | Important qualification |
| --- | --- | --- |
| **Contaminated drinking water** | Infected people shed pathogens. Unsafe disposal transports some shedding into wells, streams, reservoirs, or distribution systems. Agents acquire infection from the water they actually consume. | A shared source can expose many households simultaneously. Distance to an infected person is not the relevant distance; connection to their contaminated water source is. |
| **Fecal contamination outside drinking water** | Track contamination of food preparation, household storage, hands, soil, and relevant agricultural produce. Sanitation changes the fraction of excreta safely contained, removed, or treated. | Clean municipal water does not eliminate exposure through food, storage, or the surrounding environment. |
| **Household and institutional crowding** | Transmission depends on infectious occupants, time spent together, ventilation, and household or venue characteristics. | Use people per room or shared-air exposure, not an undifferentiated “urban density” multiplier. |
| **Connectivity and migration** | Travelers can introduce pathogens; newcomers bring their own immune histories. Seasonal migration and commercial traffic change introductions and contact networks. | Do not give migrants an intrinsic disease penalty. Susceptibility should depend on prior exposure, nutrition, and living conditions. |
| **Unequal access** | Households choose among sources using price, collection time, reliability, and perceived quality. Poorer households may remain dependent on unsafe sources after infrastructure exists. | Citywide infrastructure coverage is not equivalent to universal use or equal protection. |
| **Infrastructure overload and failure** | Waste inflow, treatment demand, or drainage demand exceeding effective capacity creates overflow, untreated discharge, or service interruption. | Effective capacity depends on labor, materials, maintenance, and operating inputs—not just installed capacity. |
| **Other urban hazards** | Keep respiratory infection, industrial pollution, occupational injury, and locally relevant vector exposure separate from enteric disease. | Sanitation should not automatically cure every source of excess mortality. |

These are proposed simulation rules. Their water-and-waste foundation is consistent with WHO’s exposure framework, while environmental-reservoir models and household/venue epidemic models provide implementable precedents. [World Health Organization](https://www.who.int/health-topics/water-sanitation-and-hygiene-wash)

### 1.2 Water supply and sewage disposal can be complements

A clean-water system can be undermined by contaminated household storage, damaged pipes, or fecal contamination elsewhere. Conversely, toilets and sewers can move waste away from homes while discharging it into somebody else’s water source. The relevant endpoint is therefore **safe separation of excreta from people throughout the system**, not the number of sanitation buildings.

Massachusetts research finds strong complementarity between clean water and effective sewerage. Mexico’s chlorination experience likewise shows that degraded pipes and inadequate sanitation can attenuate benefits. These findings argue against independent, additive bonuses such as “waterworks −20% mortality; sewer −20% mortality.” [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/700766)

**Recommended rule:** represent the entire chain:

\[
\text{excretion}
\rightarrow
\text{containment}
\rightarrow
\text{collection}
\rightarrow
\text{transport}
\rightarrow
\text{treatment or disposal}.
\]

Each stage has capacity, leakage, access, and maintenance. Failure at a downstream stage can defeat investment upstream.

### 1.3 Sanitation is also an institutional problem

The historical alternatives were not limited to modern municipal utilities. In eighteenth-century Edo, agricultural demand created markets for human waste, with disputes over collection rights, prices, and contractual arrangements. A 1789 petition involved 1,016 villages. This provides a concrete precedent for sanitation-related services emerging through property rights and commercial incentives. It does **not** establish a quantified mortality benefit from night-soil collection. [CiNii](https://cir.nii.ac.jp/crid/1390001205136876416)

For TCE, allow waste removal to emerge through municipal employment, household contracts, landlord obligations, agricultural buyers, communal duties, or combinations of these. Model their actual performance. Exporting untreated waste may improve one neighborhood while creating exposure elsewhere; WHO identifies untreated excreta and wastewater as pathways to contaminated water and food. [World Health Organization](https://www.who.int/health-topics/water-sanitation-and-hygiene-wash)

### 1.4 There is no defensible universal decomposition

Do not assign “40% of the urban penalty to water, 30% to crowding, and 30% to migration.” Those contributions depend on the local disease mix and interact. Even the share of twentieth-century American mortality decline attributable to water interventions remains disputed between major empirical studies. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20190034)

For simulation analysis, estimate contributions through counterfactual runs: improve water while holding other systems fixed, then improve sanitation, then both. Report the interaction rather than forcing all contributions into independent percentages.

---

## 2. Quantitative parameters and calibration targets

### 2.1 Keep the outcome measures distinct

For TCE’s statistics, distinguish:

* **Infant mortality:** deaths before age one per 1,000 live births.
* **Under-five mortality probability:** probability of dying before age five, conventionally expressed per 1,000 live births.
* **Age-specific death rate:** deaths divided by person-time in that age group.
* **Life expectancy at birth:** a life-table measure, not the average age of people who happened to die.

The last distinction matters historically: Chadwick’s use of average age at death could confound mortality with population composition. It should not be imported into TCE as a life-expectancy estimate. [Cambridge University Press](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/DB9628F9922A0EB9CC7988BD4CB0246A/S0025727300068721a.pdf/edwin-chadwick-and-the-poverty-of-statistics.pdf)

**Confidence labels below:** high means well-supported for the stated setting and measure; moderate means useful but affected by historical reconstruction or observational identification. Neither implies high confidence when transferring the value to another society.

### Historical mortality benchmarks

| Setting | Quantitative observation | Appropriate use in TCE | Confidence and limitation |
| --- | --- | --- | --- |
| **London, mid-eighteenth century** | Infant mortality approximately **300–400 per 1,000 births**, versus approximately **180 nationally**. | A severe, poorly protected metropolitan scenario should be capable of producing very high infant mortality. | **Moderate.** London-to-national comparison, not a matched urban–rural estimate. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.12964) |
| **Selected small English towns, 1675–1749** | Towns of roughly **2,000–3,000 inhabitants** had infant mortality **209–270 per 1,000**; some remote rural parishes were below **100**. | Serious urban penalties must be possible below TCE’s largest settlement sizes. | **Moderate.** Selected contrasts, not representative national averages. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.12964) |
| **United States, around 1900** | Urban infant mortality was approximately **50% higher** in the relevant registration comparisons; convergence occurred by approximately the **1930s**. | Industrial-era benchmark for a substantial but reversible infant penalty. | **Moderate.** Historical registration coverage limits generalization. [IZA Docs](https://docs.iza.org/dp4548.pdf) |
| **Ireland, before the 1940s** | Urban infant mortality approximately **50% higher**; the penalty disappeared by the **mid-1950s**. | Different countries should reach convergence at different times. | **Moderate.** Descriptive chronology is stronger than attribution to one reform. [IZA Docs](https://docs.iza.org/dp4548.pdf) |
| **Two Nairobi informal settlements, 2003–2010** | Infant mortality declined **83 → 57 per 1,000**; under-five mortality **113 → 79 per 1,000**. Mortality remained disadvantaged relative to comparison populations. | Permit improvement without eliminating severe neighborhood inequality. | **Moderate–high** for these monitored settlements; not an estimate of sanitation’s isolated effect. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4158907/) |
| **United States, 2019** | Age-adjusted **natural-cause mortality at ages 25–54** was **43% higher in rural than metropolitan areas**. | The sign of the urban penalty must be able to reverse. | **High** for this particular comparison; not all ages, all causes, or a 2026 estimate. [Economic Research Service](https://www.ers.usda.gov/publications/108701) |

**Interpretation:** these observations are calibration targets, not reasons to impose a fixed urban mortality ratio. A settlement with suitable exposure pathways should be able to reproduce them; another settlement with different water, housing, disease ecology, and institutions need not.

### 2.2 Measured effects of sanitary interventions

| Intervention and setting | Estimated effect | What the number measures | Confidence and transfer warning |
| --- | --- | --- | --- |
| **Water supply comparison, London cholera epidemic, 1854** | **315 versus 37 cholera deaths per 10,000 houses** supplied by Southwark & Vauxhall versus Lambeth—about **8.5-fold**. | Outbreak-specific deaths using **houses**, not people, as the denominator. | **High** for the recorded contrast; not an annual all-cause mortality multiplier. [Data Science at UChicago](https://data8.datascience.uchicago.edu/chapters/02/2/snow-s-grand-experiment.html) |
| **Filtration, 25 major US cities, 1900–1940** | Approximately **11–12% lower infant mortality** in Anderson, Charles, and Rees. | Estimated change in infant mortality associated with filtration. | **Moderate.** Historical panel identification and intervention coding matter. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20190034) |
| **US filtration, alternative analysis** | Cutler and Miller’s revised estimate attributes **38% of the observed total mortality decline** in their sample to filtration. | **Share of a historical decline**, not a 38% reduction in each person’s mortality risk. | **Contested magnitude.** Their sample and coding differ from the preceding study. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20190711) |
| **Water plus sewerage, Boston-area Massachusetts, 1880–1920** | Together explain approximately **one-third of the decline in log under-five mortality**. | Contribution to the historical decline; the two investments were complementary. | **Moderate–high** within the setting; not a generic 33% mortality bonus. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/700766) |
| **Purified water access, Tokyo, 1921–1937** | Model attributes **41.3%** of crude-death-rate improvement and **34.9%** of child-death-rate improvement to clean water. | Modeled shares of improvement, using spatial evidence. | **Moderate.** Observational attribution; the child measure is not infant mortality. [Springer](https://link.springer.com/article/10.1007/s11698-016-0148-3) |
| **Urban chlorination, Mexico, program beginning 1991** | Coverage rose **58% → over 90% within 18 months**; childhood diarrheal mortality fell an estimated **45–67%**. | **Diarrheal mortality**, not all-cause child mortality. | **Moderate–high** for the studied program; effects depended on infrastructure condition. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fpol.20180764) |
| **Citywide sanitation program, Salvador, Brazil** | Adjusted childhood diarrhea prevalence fell **22%**, with **95% confidence interval 19–26%**. | **Morbidity**, not mortality; before-and-after child cohorts. | **Moderate.** Useful exposure benchmark, not a death-risk coefficient. [ResearchGate](https://www.researchgate.net/publication/5854166_Effect_of_city-wide_sanitation_programme_on_reduction_in_rate_of_childhood_diarrhoea_in_northeast_Brazil_assessment_by_two_cohort_studies) |

The American disagreement should remain visible in calibration. Anderson and colleagues found no detectable contribution from several other studied interventions in their specification; Cutler and Miller defend a substantial filtration contribution after corrections. This is a dispute over historical attribution and magnitude, not evidence that contaminated water is harmless. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20190034)

### 2.3 Engineering and operational parameters

WHO service-level benchmarks are useful for designing water-access behavior, but they describe modern service conditions, not universal ancient consumption.

| Parameter | Useful value or range | Units | Status |
| --- | --- | --- | --- |
| Basic-access household water quantity | Often **unlikely to exceed 20** | Liters/person/day | WHO’s 2003 service-level benchmark. Not a biological safety threshold. |
| Intermediate access | Approximately **50** | Liters/person/day | Typically associated with water available on the plot. |
| Optimal access | **100 or more** | Liters/person/day | Higher-service domestic supply benchmark. |
| Basic-access collection time | Approximately **5–30** | Minutes per collection round trip | Useful for household labor and source-choice modeling. |

These benchmarks come from Howard and Bartram’s WHO report; WHO published an updated assessment in 2020 emphasizing quantity, access, reliability, and other service characteristics. [ResearchGate](https://www.researchgate.net/publication/252913246_Domestic_Water_Quantity_Service_Level_and_Health)

At **50,000 people × 50 liters/person/day**, domestic demand is **2.5 million liters/day**, or **2,500 m³/day**. That is a calculation for a selected service target, not an estimate of what an early agrarian city necessarily consumed.

For pathogen timing, parameters must remain disease-specific. As one example, WHO gives cholera’s incubation period as **12 hours–5 days**, with infected people potentially shedding bacteria in feces for **1–10 days**. Do not assign those values to every enteric infection. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/cholera)

---

## 3. Variation across eras and regions—and the migration question

### 3.1 There was no single global sanitation sequence

| Context | What the evidence supports | Modeling implication |
| --- | --- | --- |
| **Foragers and mobile camps** | Comparative forager demography documents substantial variation; contemporary hunter-gatherer observations are not direct measurements of Paleolithic populations. | Do not assign a “forager urban penalty.” Model residence duration, camp reuse, source contamination, and contact with other groups. Mobility can reset local waste accumulation in the model without making people disease-free. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1728-4457.2007.00171.x?utm_source=chatgpt.com) |
| **Early farming and permanent settlements** | Çatalhöyük provides evidence of close contact with refuse, animal waste, and contaminated surroundings. Skeletal findings do not yield a trustworthy urban–rural mortality ratio. | Permanent residence should create accumulating exposure, but do not translate skeletal lesion prevalence directly into death probability. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1904345116) |
| **Ancient cities in the Americas** | Sediment analysis at Tikal’s Corriental reservoir supports an interpretation of quartz-and-zeolite filtration. No associated mortality reduction has been measured. | Allow empirical water-management innovation before modern science. Archaeological infrastructure is evidence of capability, not a numerical efficacy coefficient. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7582844/) |
| **Preindustrial East Asia** | Edo’s documented waste markets show sanitation-related removal organized through agricultural demand and property rights. | Permit commercial and customary solutions, including systems that improve local cleanliness while leaving other transmission routes open. [CiNii](https://cir.nii.ac.jp/crid/1390001205136876416) |
| **Industrializing Europe and North America** | Historical mortality transitions were uneven; US urban mortality deterioration or stagnation preceded sustained improvement from the later nineteenth century. | Growth, disease conditions, housing, infrastructure, and reform should evolve separately rather than follow one technological-era switch. [National Bureau of Economic Research](https://www.nber.org/papers/h0134) |
| **Industrializing Japan** | Interwar Tokyo offers quantitative evidence linking expanded purified-water access to mortality improvement. | The same broad mechanisms can operate under different institutional and urban-development histories. [Springer](https://link.springer.com/article/10.1007/s11698-016-0148-3) |
| **Later urbanization in Latin America and Africa** | Mexico and Salvador show sizable intervention benefits; Nairobi shows that urban averages can conceal persistent disadvantaged settlements. | Model service inequality and maintenance within cities, not simply national “development level.” [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fpol.20180764) |

**The evidence gap is important:** this literature does not support a defensible table of matched urban–rural mortality multipliers for every ancient civilization, African region, South Asian society, or historical Chinese city. It is better to expose uncertainty than populate TCE with apparently precise regional constants.

### 3.2 Snow: source-specific exposure, not a generic cleanliness score

Snow’s 1854 water-company comparison is especially useful for simulation design. Households supplied by two companies could inhabit overlapping urban environments while receiving water of very different quality. The observed cholera mortality contrast therefore points toward **water-supply membership as a causal exposure variable**. [Data Science at UChicago](https://data8.datascience.uchicago.edu/chapters/02/2/snow-s-grand-experiment.html)

For TCE, this suggests a powerful validation scenario: two intermingled groups with similar crowding and incomes, but different water sources. A correct waterborne model should generate different attack rates without requiring visibly separate “healthy” and “unhealthy” districts.

The famous pump episode is valuable history, but the company comparison is the more useful quantitative calibration example here.

### 3.3 Chadwick: reform required organization as well as engineering

Chadwick’s **1842 report** and the **1848 Public Health Act** helped frame drainage, refuse removal, water supply, and public-health administration as coordinated responsibilities. The institutional lesson is that recognizing a problem, authorizing action, financing it, and operating a service are separate achievements. [Parliament UK News](https://www.parliament.uk/about/living-heritage/transformingsociety/towncountry/towns/tyne-and-wear-case-study/about-the-group/public-administration/the-1848-public-health-act/)

Represent these as separate TCE capabilities. A council may understand that one district is unhealthy but lack revenue, jurisdiction over an upstream discharge, construction labor, or the political support to charge property owners.

Also distinguish knowledge from measurement: mortality records can be incomplete, and misleading statistics can produce mistaken beliefs. Chadwick’s average-age-at-death comparisons are a historical warning against granting institutions perfect demographic information. [Cambridge University Press](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/DB9628F9922A0EB9CC7988BD4CB0246A/S0025727300068721a.pdf/edwin-chadwick-and-the-poverty-of-statistics.pdf)

### 3.4 How long did cities depend on immigration?

**Not all cities depended on immigration until modern sanitation.** Many European cities were demographic sinks during the seventeenth and eighteenth centuries, but by approximately **1800**, cities in Britain and parts of northwestern Europe were generally capable of natural increase. British urban–rural life-expectancy convergence came much later, in the **1930s**. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.12964)

These are different transitions:

\[
\Delta N = B-D+I-O
\]\[
r\_{\text{natural}}=\frac{B-D}{N}.
\]

A positive urban mortality penalty does not imply \(B<D\). Nor does positive current natural increase necessarily establish long-run replacement under a stable age structure.

**Illustrative TCE settlement—not a historical estimate:**

| Quantity | Before mortality improvement | After improvement |
| --- | --- | --- |
| Population | 50,000 | 50,000 |
| Birth rate, held constant for illustration | 35/1,000/year | 35/1,000/year |
| Death rate | 45/1,000/year | 25/1,000/year |
| Births | 1,750/year | 1,750/year |
| Deaths | 2,250/year | 1,250/year |
| Natural change | **−500/year** | **+500/year** |
| Net immigration needed merely to avoid decline | **500/year** | **0** |

Before improvement, net immigration of 1,250 produces growth of 750 people annually despite the natural deficit. After improvement, unchanged immigration produces much faster growth—and may overwhelm the infrastructure again.

Consequently, TCE should separately record **migration required to offset natural decrease**, **migration attracted by opportunities**, and **natural increase**. A macroeconomic modeling precedent explicitly connects urban mortality decline to both population growth and migration incentives. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fmac.20170189)

---

## 4. Stylized facts a correct simulation should reproduce

Treat these as validation tests rather than scripted outcomes.

| Test | Required emergent pattern |
| --- | --- |
| **Small settlements can be dangerous** | A contaminated source or poor waste handling can produce high mortality without a metropolis-sized population. The small-town historical benchmarks above should be attainable. |
| **Network position matters** | People supplied by the same contaminated water network cluster in risk even when they are not nearest neighbors. Snow’s comparison provides a concrete test. [Data Science at UChicago](https://data8.datascience.uchicago.edu/chapters/02/2/snow-s-grand-experiment.html) |
| **Benefits are not simply additive** | Clean water plus effective sewage management can outperform piecemeal improvements; degraded distribution can weaken treatment benefits. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/700766) |
| **Improvement can be unequal** | Citywide mortality can fall while poorly served neighborhoods retain a substantial disadvantage. Nairobi provides a longitudinal example. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4158907/) |
| **Population growth and health can diverge** | A city can grow despite natural decrease, or remain less healthy than its hinterland despite positive natural increase. This follows from the demographic accounting above. |
| **Different causes respond differently** | An enteric-disease intervention should not create an identical percentage reduction in every cause of death. The empirical studies measure different cause-specific and all-cause outcomes. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20190034) |
| **Small populations experience epidemic extinction** | Acute infections should sometimes disappear locally and return through introductions. Bartlett’s classic measles analysis illustrates persistence requirements on the order of hundreds of thousands under its assumptions—not a threshold below which outbreaks cannot occur. [OUP Academic](https://academic.oup.com/jrsssa/article-pdf/120/1/48/49739351/jrsssa_120_1_48.pdf) |
| **The penalty can reverse** | Well-served cities can become healthier than rural areas, while deprived urban neighborhoods remain exceptions. [Economic Research Service](https://www.ers.usda.gov/publications/108701) |

One additional conservation test is particularly important: **exporting untreated sewage must not delete its downstream consequences**. That is a requirement of the proposed exposure model, not a claim that every historical sewer project worsened downstream mortality.

---

## 5. Recommended TCE implementation

### 5.1 Represent a small number of real transmission processes

For the first implementation, I recommend four separable disease systems: an environmentally transmitted enteric system, an acute close-contact system, a chronic respiratory system, and an optional vector/parasite system appropriate to local ecology.

These are **computational categories**, not four interchangeable pathogens. Each instantiated disease still needs its own immunity, latency, infectiousness, environmental survival, and fatality parameters.

A minimal state architecture is:

| Entity | State worth retaining |
| --- | --- |
| **Person** | Age, nutritional condition, relevant immunity, infection stage, severity, household, routine venues, water actually consumed, travel history. |
| **Household** | Water source choices, storage contamination, sanitation practice, collection time, affordability, occupancy, food-preparation conditions. |
| **Venue** | Occupants and duration, infectious prevalence, crowding, ventilation or contact characteristics. |
| **Water/waste node** | Volume or storage capacity, flow links, pathogen load, treatment performance, leakage, service interruptions. |
| **Institution/operator** | Revenue, operating expenditure, workers, materials, maintenance backlog, construction projects, jurisdiction, observations and beliefs. |

### 5.2 Use an environmental mass balance

Codeço’s cholera model is a useful precedent for coupling infection dynamics to an aquatic reservoir. TCE should borrow that structure, not transplant one pathogen’s fitted coefficients to all diseases. [Springer](https://link.springer.com/article/10.1186/1471-2334-1-1)

For pathogen \(p\) in water node \(w\), a proposed formulation is:

\[
\frac{dM\_{wp}}{dt}
=
S\_{wp}
+\sum\_v Q\_{vw}C\_{vp}
-\sum\_v Q\_{wv}C\_{wp}
-\delta\_{wp}M\_{wp}
-R\_{wp},
\qquad
C\_{wp}=\frac{M\_{wp}}{V\_w}.
\]

Here \(M\) is pathogen load, \(S\) incoming shedding and contamination, \(Q\) water flow, \(V\) water volume, \(\delta\) environmental decay, and \(R\) treatment removal. Use a consistent unit system and a nonnegative numerical update.

At household consumption, apply treatment and possible recontamination:

\[
C\_{\text{consumed},p}
=
10^{-L\_p}C\_{\text{source},p}
+C\_{\text{distribution intrusion},p}
+C\_{\text{household storage},p}.
\]

\(L\_p\) is pathogen-specific log removal. Treatment performance should depend on the actual technique, operation, and condition.

A simple dose-response option is:

\[
P\_{i,p}(\text{infection in day})
=
1-\exp\left[-s\_{i,p}\left(
\kappa\_p\,v\_iC\_{\text{consumed},p}
+H\_{\text{food},i,p}
+H\_{\text{contact},i,p}
\right)\right].
\]

Here \(v\_i\) is ingested water volume, \(s\_{i,p}\) susceptibility, and each term inside the outer parentheses is an integrated daily exposure hazard. This is a proposed model form, **not a universally validated dose-response law**.

Most importantly, **fecal waste should not spontaneously create a pathogen absent from the world**. Infection requires introduction, an existing carrier population, or an appropriate environmental or animal reservoir.

### 5.3 Separate deaths from infections—and avoid double counting

Use infection severity and age-specific fatality to produce disease deaths. Keep unrelated background mortality separate.

A common error would be to give agents a historical all-cause mortality schedule and then add the infections responsible for part of that schedule. That counts some deaths twice. Instead, fit the residual background hazard jointly with explicit causes:

\[
P\_i(\text{death during }\Delta t)
=
1-\exp\left[-\Delta t\sum\_k h\_{ik}\right].
\]

Assign the cause consistently with the competing hazards.

Water infrastructure should normally reduce exposure first. Treatment access can separately reduce the chance that infection becomes fatal. This preserves the distinction between preventing disease and surviving it.

### 5.4 Make reform a continuing organizational achievement

Separate **capital construction** from **operations and maintenance**. A completed aqueduct, drain, or treatment facility should require whatever workers, materials, fuel, power, cleaning, and repair its design entails.

I recommend that institutions choose projects from imperfect observations: unusual deaths, repeated household complaints, disrupted trade, burial records, source-specific illness patterns, or visible overflow. Their decisions can depend on fiscal capacity, perceived benefits, political influence, and competing priorities.

Do not require modern germ theory before permitting useful source protection, waste removal, or empirical filtration. Conversely, do not make scientific understanding sufficient for implementation.

The resulting feedback can be:

\[
\text{population growth}
\rightarrow
\text{overload}
\rightarrow
\text{illness and pressure for action}
\rightarrow
\text{investment}
\rightarrow
\text{lower mortality}
\rightarrow
\text{faster population growth}.
\]

This is a proposed emergent feedback, not a prescribed sequence every city must follow.

### 5.5 Explicit starting values for sensitivity testing

The following are **design experiments, not historical estimates**. Their purpose is to reveal whether the mechanics behave sensibly.

| Variable | Suggested initial test grid | Interpretation |
| --- | --- | --- |
| Household service coverage | **0%, 25%, 50%, 75%, 100%** | Allocate spatially and by access, not only at random. |
| Demand/effective-capacity ratio | **0.5, 0.8, 1.0, 1.2, 2.0** | Tests spare capacity, full utilization, and overload. |
| Operating availability | **50%, 75%, 90%, 100%** | Compare chronic interruption with reliable service. |
| Pathogen removal | **0, 1, 2, 3, 4 log₁₀** | Mathematical exposure reductions; not claims about particular ancient filters. |
| Project completion delay | **1, 5, 10 years** | Tests demographic growth during construction. |
| Reporting interval | Daily events; annual summaries; **10–20-year** pooled comparisons | Avoid calibrating a small settlement to one noisy year. |

Do **not** supply a universal mortality coefficient per additional person per hectare. Calibrate contacts, contamination, and capacity instead.

### 5.6 Performance at 10,000–50,000 agents

Use venue occupancy buckets and water-network aggregation rather than all-pairs contacts. Each day, agents contribute infectiousness to their households and venues; shared exposure summaries then produce individual hazards. Environmental loads update per network node.

This makes the main workload approximately proportional to the number of agents, venue memberships, and network links—not \(N^2\). Epidemiology should run in the Rust kernel independently of camera position, rendering distance, or frame rate.

Retain enough provenance to explain events: the household’s water source, the contaminated network, the responsible failure, and the institution’s response. Exact person-to-person attribution may be probabilistic, but the source pathway should remain inspectable.

### 5.7 Existing models and games worth borrowing from

| Precedent | What to borrow | What not to assume |
| --- | --- | --- |
| **Codeço, 2001: aquatic-reservoir cholera model** | Coupled human infection and environmental pathogen dynamics. | Its assumptions or coefficients apply to every waterborne disease. [Springer](https://link.springer.com/article/10.1186/1471-2334-1-1) |
| **FRED, Grefenstette et al., 2013** | Synthetic populations, households, workplaces, schools, and venue-based transmission. | It is a complete historical sanitation model. [Springer](https://link.springer.com/article/10.1186/1471-2458-13-940) |
| **Jedwab and Vollrath, 2019** | Interaction between mortality decline, natural population growth, migration, and urbanization. | A macroeconomic model supplies individual water-exposure mechanics. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fmac.20170189) |
| **Workers & Resources: Soviet Republic** | Infrastructure and service logistics as an operational system rather than a building-radius bonus. | Its gameplay outcomes are validated historical demographic estimates. [Steam Store](https://store.steampowered.com/app/784150/Workers__Resources_Soviet_Republic/) |

---

## 6. Sources, datasets, and limits of confidence

### Most useful evidence and data for calibration

| Resource | Best use | Main limitation |
| --- | --- | --- |
| **Davenport, “Urbanization and mortality in Britain, c. 1800–50” (2020)** | Historical variation and chronology; includes discussion of Woods’s England and Wales decennial cause-of-death data, UK Data Service study **3552**. | Britain is unusually well documented. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.12964) |
| **Haines, “The Urban Mortality Transition in the United States, 1800–1940” (2001)** | Long-run American urban mortality transition. | Early geographic and registration coverage are incomplete. [National Bureau of Economic Research](https://www.nber.org/papers/h0134) |
| **Alsan–Goldin; Anderson–Charles–Rees; Cutler–Miller** | Water/sewerage effects, competing specifications, and replication exercises. Their publication pages provide data or replication materials. | Intervention timing, denominators, and identifying assumptions affect estimates. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/700766) |
| **Ogasawara, Shirota, and Kobayashi, interwar Tokyo study** | Spatial access differences, municipal records, waterworks statistics, and consistent-boundary analysis. | Modeled historical attribution rather than randomized exposure. [Springer](https://link.springer.com/article/10.1007/s11698-016-0148-3) |
| **Bhalotra et al., Mexico chlorination study** | Large-scale disinfection effects and infrastructure-condition heterogeneity; replication package available. | Modern treatment context and disease composition differ from ancient societies. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fpol.20180764) |
| **Kenya DHS and Nairobi Urban Health and Demographic Surveillance System, used by Kimani-Murage et al.** | Urban–rural comparisons, neighborhood inequality, births, deaths, and migration. | Urban categories conceal substantial internal variation. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4158907/) |
| **Human Mortality Database** | Age- and sex-specific mortality schedules for well-recorded populations. | Mostly not a ready-made dataset of ancient or urban–rural mortality contrasts. [Human Mortality Database](https://www.mortality.org/) |

### Claims to keep explicitly uncertain

**Ancient infrastructure does not establish mortality effects.** A drain, reservoir, or filter demonstrates a practice or capability. Without demographic evidence, it does not establish the percentage of deaths prevented. Tikal and Çatalhöyük are informative about mechanisms but cannot supply precise urban mortality multipliers. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7582844/)

**Historical effect sizes are not automatically portable.** A reduction in childhood diarrheal mortality under a twentieth-century disinfection program is not an all-cause mortality reduction for an early farming town. Differences in baseline disease, other exposure routes, and available treatment matter. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fpol.20180764)

**Aggregate mortality can mislead.** TCE’s calibration should compare ages, causes, household conditions, and stable geographic populations—not merely citywide crude death rates. Its internal records should preserve births, deaths, migration, and person-time even when simulated institutions possess only imperfect versions of those statistics.

### Bottom line for TCE

**Make the urban mortality penalty an outcome of exposure and service failure. Make its reduction an outcome of maintained, accessible, coordinated systems.** Population growth should increase the demands placed on those systems, but a large city need not be unhealthy, and a small settlement need not be safe.

That design can produce demographic sinks, thriving but deadly cities, commercially organized sanitation, unsuccessful reforms, neighborhood inequality, and eventual urban health advantages—without scripting a European sequence or assigning a permanent “city mortality” modifier.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92824-b440-83ea-982c-9503a936832d)
