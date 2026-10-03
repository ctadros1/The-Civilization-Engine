# Epidemiology of historical diseases: a simulation-ready report for TCE

## Executive recommendation

**Build several disease mechanisms on a shared infection framework—not one generic disease system with different mortality settings.** Measles, typhoid, tuberculosis, malaria and scabies persist for fundamentally different reasons. Their differences must exist in the simulation’s state transitions, environmental systems and contact rules, not merely in their names.

The most important consequence of TCE’s scale is this: **when 10k–50k people constitute the entire epidemiologically connected population, measles should usually disappear after an outbreak.** Recurrent epidemics then require susceptible cohorts to accumulate *and* infection to return. A larger, partly abstracted external population is a defensible source of introductions; an invisible permanent infected citizen is not. Historical estimates of measles persistence thresholds are substantially above TCE’s population range, although those thresholds depend strongly on seasonality and connectivity. [bpb-us-e1.wpmucdn.com](https://bpb-us-e1.wpmucdn.com/sites.psu.edu/dist/0/175317/files/2023/12/rural-urban-gradient.pdf)

The strongest evidence concerns disease routes and approximate biological timings. The weakest concerns universal “ancient” fatality rates, historical nutrition multipliers, and reproduction numbers transferable between different settlements. The report therefore distinguishes **observed benchmarks, fitted estimates and proposed implementation choices**.

---

## 1. Mechanisms: rules the simulation should implement

### 1.1 Separate three clocks—and distinguish infection from illness

Store three different quantities:

| Quantity | Definition | Why it matters |
| --- | --- | --- |
| **Incubation period** | Infection → first symptoms | Determines recognition and changes in behavior. |
| **Latent period** | Infection → becoming infectious | Determines when transmission can begin. |
| **Infectious period** | Interval during which transmission is possible | Can begin before symptoms and continue after apparent recovery. |

These are not interchangeable. Influenza can spread before symptoms; Shigella can be shed after diarrhea ends; scabies can spread during a lengthy symptom-free interval. Conversely, inactive tuberculosis is not contagious. [CDC](https://www.cdc.gov/pinkbook/hcp/table-of-contents/chapter-12-influenza.html)

**Implementation rule:** infection state, symptom state, infectiousness and work capacity should be separate variables.

### 1.2 Transmission must follow a plausible route

| Mechanism | Implementable rule | Necessary distinctions |
| --- | --- | --- |
| **Shared air** | Infectious people contribute exposure to occupied rooms; susceptible occupants accumulate exposure according to duration, ventilation and pathogen-specific shedding. | Measles and TB need indoor shared-air exposure. Do not treat every person within an outdoor radius as equivalent to a household member. Measles can remain transmissible in enclosed air after the source leaves. [CDC](https://www.cdc.gov/pinkbook/hcp/table-of-contents/chapter-13-measles.html) |
| **Water and food** | Infected people contaminate specific water sources, drainage paths, hands or food batches. Other people become exposed only through relevant ingestion events. | Typhoid has a human reservoir; cholera can also involve aquatic environmental persistence. A contaminated well and an infectious cook create different outbreak patterns. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/cholera) |
| **Fecal–oral contact** | Caregiving, toileting, food preparation and contaminated objects connect shedders to ingestion events. | Shigella is not exclusively “dirty river disease”: household and caregiver transmission matter. [CDC](https://www.cdc.gov/shigella/causes/index.html) |
| **Mosquito transmission** | Mosquitoes acquire infection from infectious humans, survive parasite development, then infect other humans through subsequent bites. | Malaria requires compatible vectors, suitable habitat and temperature, and the correct parasite stages. An infected person is not immediately infectious to mosquitoes. [CDC](https://www.cdc.gov/dpdx/malaria/index.html) |
| **Rodent–flea transmission** | Maintain rodent infection and flea exposure at settlement or landscape-patch level; generate human spillover from this system. | Bubonic plague is not ordinarily a human-to-human respiratory epidemic. Pneumonic plague requires a separate respiratory branch. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/plague) |
| **Prolonged skin contact** | Weight household, sleeping and intimate-care contacts heavily; distinguish ordinary from highly infectious crusted scabies. | For ordinary scabies, contaminated bedding is less important than prolonged skin contact; it becomes more important with crusted infestation. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/scabies) |
| **Sexual and congenital transmission** | Use partnership/contact events and mother-to-child transmission, with infectiousness determined by disease stage. | Syphilis does not spread through ordinary market proximity; fetal outcomes require a separate pregnancy pathway. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/syphilis) |

A sewer should therefore **move waste somewhere**, not apply a universal health bonus. Protected water, wastewater disposal, food handling and household hygiene should change different exposure pathways.

### 1.3 Clinical recovery is not always pathogen clearance

Use disease-specific branches:

* **Acute immunizing infection:** exposure → infectious illness → recovery with durable immunity.
* **Carrier infection:** acute illness or asymptomatic infection → continued shedding → clearance or chronic carriage.
* **Persistent/reactivating infection:** initial infection → long-lived internal state → later disease.
* **Repeated exposure with partial immunity:** infections recur, but their probability of causing severe illness changes with exposure history.

These distinctions are essential for measles, typhoid, TB and malaria respectively. A common “infected for ten days, then immune for one year” template cannot reproduce their dynamics. [CDC](https://www.cdc.gov/pinkbook/hcp/table-of-contents/chapter-13-measles.html)

### 1.4 Institutions act through mechanisms, not era modifiers

For TCE, represent institutions as providers of maintained services: clean water, waste removal, nursing, food support, isolation facilities, vaccination and treatment. Their effects should depend on coverage, supplies, access, compliance and delay.

Different services affect different outcomes. Rehydration principally changes cholera survival; isolation changes opportunities for respiratory transmission; nutrition support can change TB disease incidence. These are not interchangeable “medical technology” bonuses. [CDC](https://www.cdc.gov/yellow-book/hcp/travel-associated-infections-diseases/cholera.html)

---

## 2. Quantitative parameters

### Reading the tables

**H — high confidence:** established route or broad biological timing.  
**M — moderate confidence:** credible, but strongly dependent on population, case definition or fitted model.  
**L — low confidence:** historical transfer, duration or effect size is poorly constrained.  
**P — proposed implementation choice:** useful for TCE, but not an empirical historical estimate.

Ranges below are generally **clinical ranges, not uniform sampling distributions or 95% confidence intervals**. A high-confidence modern incubation range does not imply equally high confidence in an ancient fatality estimate.

### 2.1 Natural history: timing, persistence and immunity

| Disease | Incubation: infection → symptoms | Latency and infectiousness | Recovery, persistence and immunity | Confidence and sources |
| --- | --- | --- | --- | --- |
| **Cholera** | **12 hours–5 days**. | Fecal shedding commonly lasts **1–10 days**, including infections with few or no symptoms. Do not equate diarrhea duration with the complete infectious window. | Natural symptomatic infection can confer substantial protection for **at least 3–10 years** in studied settings; protection is not a universal sterilizing timer across strains/serogroups. | **H** timing; **M** immunity and historical transfer. WHO; Ryan and colleagues. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/cholera) |
| **Typhoid fever** | **6–30 days**. | Untreated illness can last about **one month**; shedding may extend into convalescence. | CDC estimates **1–4% of treated patients** become chronic carriers, defined by shedding for **≥12 months**. This is not a directly measured pre-antibiotic carrier probability. Reinfection is possible; a fixed immunity duration is poorly supported. | **H/M** timing and carriage; **L** universal immunity duration. [CDC](https://www.cdc.gov/yellow-book/hcp/travel-associated-infections-diseases/typhoid-and-paratyphoid-fever.html) |
| **Bacillary dysentery: Shigella** | Symptoms commonly begin in **1–2 days**. | Illness commonly lasts about **7 days**; stool shedding can continue for **up to two weeks after symptoms end**. | Retain convalescent shedding. Do not give lifelong immunity to the entire Shigella genus; species and serotypes matter. | **H** broad timing; **L/M** simplified cross-protection. [CDC](https://www.cdc.gov/shigella/about/index.html) |
| **Amoebiasis: Entamoeba histolytica** | Usually **2–4 weeks**, sometimes later. | Asymptomatic intestinal infection can transmit through cyst shedding; symptomatic illness is not a sufficient indicator of infectiousness. | Only about **10–20%** of infected people become ill. Use an asymptomatic intestinal-carriage branch, with invasive disease as a separate outcome. No well-supported universal immunity timer. | **H/M** presentation; **L** portable persistence and immunity parameters. [CDC](https://www.cdc.gov/amebiasis/about/index.html) |
| **Smallpox** | Usually **10–14 days**, reported range **7–19 days**. | Not contagious during incubation. Infectiousness begins around the prodrome/rash transition and continues until scabs separate, roughly **3–4 weeks after rash onset**. | Recovery gives prolonged protection; lifelong protection is a reasonable **P** simplification for ordinary simulation horizons. Do not leave recovered people as chronic carriers. | **H** broad timing; **M/P** simplified immunity. [CDC](https://www.cdc.gov/smallpox/signs-symptoms/index.html) |
| **Measles** | About **11–12 days to prodrome**, **14 days to rash**. | Infectious approximately **4 days before to 4 days after rash**. A latent period around **10 days** is a useful derived central value, not an independently fixed clock. | Natural recovery normally gives lifelong immunity. No human chronic-carrier state. | **H** timing and immunity. [CDC](https://www.cdc.gov/pinkbook/hcp/table-of-contents/chapter-13-measles.html) |
| **Influenza** | **1–4 days**, average about **2 days**. | Adults commonly infectious from about **1 day before symptoms through 5–7 days afterward**; children can shed for **10 days or longer**. | Immunity depends on strain, prior infection and antigenic change. Avoid a universal “immune for exactly one year” rule. | **H** broad timing; **M/L** long-horizon immune simplification. [CDC](https://www.cdc.gov/pinkbook/hcp/table-of-contents/chapter-12-influenza.html) |
| **Tuberculosis** | No single useful acute incubation timer. Infection can precede active disease by months or years. | Inactive infection is noninfectious. Infectious pulmonary disease may persist for a long time; untreated active disease averaged roughly **3 years to death or self-cure** in historical evidence—not three years of constant shedding. | Approximately **5–10%** of untreated infected people develop active disease during life under conventional reference conditions. Model progression, regression and reactivation separately. | **H** distinction; **M** lifetime progression and historical duration. [CDC](https://www.cdc.gov/tb/about/inactive-tuberculosis.html) |
| **Bubonic plague** | Typically within the broader plague range of **1–7 days**. | Ordinary bubonic cases should not automatically transmit by shared air. Subsequent pneumonic involvement changes the route. | Persistence primarily requires the animal–flea system. A universal duration of human post-infection protection is poorly constrained. | **H** route; **M/L** simplified course and immunity. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/plague) |
| **Pneumonic plague** | Can be very short; general clinical descriptions and historical fitted distributions differ. | One historical analysis fitted lognormal **latent mean 4.3 days, SD 1.8**, and **infectious mean 2.5 days, SD 1.2**, using untreated cases. Treat these as a specific empirical model, not universal constants. | Acute, rapidly lethal without treatment; do not substitute the bubonic transmission mechanism. | **M**, historical sample and model assumptions matter. Gani & Leach, 2004. [CDC](https://wwwnc.cdc.gov/eid/article/10/4/03-0509_article) |
| **Malaria: falciparum/vivax archetypes** | Commonly **7–30 days**, depending on species and other conditions; delayed presentations also occur. | Human-to-mosquito infectiousness follows gametocyte development, not symptom onset alone. Mosquito parasite development commonly takes **9–18 days**, strongly temperature-dependent. | Maintain blood-stage infection and acquired partial protection. For **vivax**, add dormant liver stages and later relapses; recovery from one fever episode is not sterilizing immunity. | **H** stage structure; **M/L** compact duration and immunity functions. [CDC](https://www.cdc.gov/malaria/hcp/clinical-features/index.html) |
| **Scabies** | First symptoms usually take **4–8 weeks**. | Transmission can occur before symptoms. Infestation persists without effective clearance; this is not a short febrile infection. | Reinfestation is common. Mites usually survive only **2–3 days away from human skin**, limiting ordinary environmental persistence. | **H/M**. [CDC](https://www.cdc.gov/scabies/about/index.html) |
| **Syphilis** | **10–90 days**, often about **3 weeks** to the initial lesion. | Stage-dependent sexual infectiousness, especially early disease; primary lesions persist for weeks. Latent infection must not mean equally infectious for decades. | Untreated infection can persist for years; tertiary manifestations may occur **10–30 years** later. Reinfection remains possible. | **H/M** staging; **L** one-number lifetime outcome models. [CDC](https://www.cdc.gov/mmwr/volumes/73/rr/rr7301a1.htm) |

**Dysentery is a syndrome, not one pathogen.** Historical records bearing that label cannot automatically calibrate Shigella, amoebiasis and other diarrheal infections as though they were biologically identical.

### 2.2 Reproduction numbers: calibration targets, not pathogen constants

\(R\_0\) describes transmission in a specified susceptible population and environment. It changes with contact patterns, housing, water systems, vectors and the method of estimation. The effective reproduction number additionally reflects existing immunity and interventions. Consequently, a directly observed epidemic growth rate is not itself \(R\_0\). [CDC](https://wwwnc.cdc.gov/eid/article/25/1/17-1901_article)

| Disease/context | Quantitative benchmark | Appropriate use in TCE | Confidence |
| --- | --- | --- | --- |
| **Measles** | Conventional \(R\_0\) benchmark **12–18**; the broader literature contains substantially wider estimates. | Reference calibration for a highly susceptible, connected population—not a mandatory value for every village. [PubMed](https://pubmed.ncbi.nlm.nih.gov/28757186/) | **M** portability |
| **Smallpox** | Historical initially susceptible populations: estimated \(R\_0\) **3.5–6**. | Benchmark for corresponding contact conditions and negligible prior immunity. Gani & Leach, 2001. [Nature](https://www.nature.com/articles/414748a) | **M** |
| **Seasonal influenza** | Review median reproduction estimate **1.28**, interquartile range **1.19–1.37**. | The review includes different kinds of reproduction estimates; **do not relabel every estimate as pure \(R\_0\)**. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4169819/) | **M** |
| **1918 influenza, community settings** | Median **1.80**, interquartile range **1.47–2.27**; confined settings were higher. | Pandemic reference scenario, not a universal influenza default. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4169819/) | **M** |
| **Pneumonic plague** | Historical fitted mean **1.3 secondary cases**, offspring variance **3.1**. | Reproduce heterogeneous, often small transmission chains; do not make every case infect exactly 1.3 people. [CDC](https://wwwnc.cdc.gov/eid/article/10/4/03-0509_article) | **M** |
| **Falciparum malaria** | One analysis of **121 African populations** estimated values from around **1 to over 3,000**. | A warning against a portable constant. Results depend on biting heterogeneity, finite-population effects and how vector generations are counted. Prefer infectious bites/person/year and parasite prevalence as calibration targets. [PLOS](https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.0050042) | **M** within model; **L** portability |
| **Cholera, typhoid, dysentery** | No single historical range sufficiently portable across water/food systems. | Estimate transmission in the actual settlement network; calibrate contaminated-source attack patterns and incidence. | **L** universal values |
| **TB, scabies, syphilis** | No single historical range sufficiently portable across progression and contact structures. | Calibrate active-disease incidence, household/partnership spread, infectious duration and persistence jointly. | **L** universal values |

For TCE, measure a reproduction number by introducing infection into replicated susceptible reference populations and recording secondary infections. Do not translate a literature value into a daily infection probability without accounting for how contacts and infectiousness are represented.

---

## 3. Fatality, age, nutrition and lasting impacts

### 3.1 Use the correct denominator

Keep these separate:

\[
\text{IFR}=\frac{\text{deaths}}{\text{all infections}},\qquad
\text{CFR}=\frac{\text{deaths}}{\text{defined clinical cases}}.
\]

Hospitalized-case fatality and severe-case fatality have still narrower denominators. A large fatality estimate among severe patients must not be applied to every exposed or infected citizen.

### 3.2 Mortality benchmarks

| Disease | Quantitative evidence | Modeling interpretation | Confidence |
| --- | --- | --- | --- |
| **Cholera** | Untreated clinical case fatality can reach approximately **50%**; timely adequate rehydration can reduce it below **1%**. | Branch mild/asymptomatic infection from severe dehydration. Care availability and delay should strongly change survival. Neither figure is a universal infection-fatality ratio. [CDC](https://www.cdc.gov/yellow-book/hcp/travel-associated-infections-diseases/cholera.html) | **H** treatment contrast; **M/L** historical population transfer |
| **Typhoid** | Pre-antibiotic CFR **>10%**; prompt appropriate care generally reduces CFR below **1%**. | A weeks-long illness with substantial untreated mortality, not merely a brief productivity penalty. [CDC](https://www.cdc.gov/yellow-book/hcp/travel-associated-infections-diseases/typhoid-and-paratyphoid-fever.html) | **M** |
| **Shigella dysentery** | WHO reports CFR as high as **15% among hospitalized patients with S. dysenteriae type 1**. | An upper benchmark for a particularly dangerous species and selected severe cases—not “15% of all dysentery infections.” [WHO IRIS](https://iris.who.int/bitstreams/ae354d78-f155-4d9b-98e8-831f76ae2a22/download) | **M** for that denominator; **L** generic transfer |
| **Amoebiasis** | Most infections do not produce clinical illness; invasive intestinal and extraintestinal disease are different outcomes. | No defensible single historical CFR for all infections was established here. Model invasive disease separately. [CDC](https://www.cdc.gov/amebiasis/about/index.html) | **L** universal CFR |
| **Smallpox** | Variola major historically killed approximately **30% of unvaccinated cases**. | Keep variola major separate from milder smallpox variants. Pregnancy and disease form matter; malignant disease is disproportionately associated with children. [CDC](https://www.cdc.gov/smallpox/hcp/clinical-signs/index.html) | **H/M** aggregate; **L** universal age curve |
| **Measles** | Modeled 2019 LMIC CFR, ages 0–34: **1.32% in community settings**, versus **5.35% in hospital settings**. | These are modern, setting-specific benchmarks—not ancient mortality constants. [PubMed](https://pubmed.ncbi.nlm.nih.gov/36925172/) | **M** |
| **Influenza** | Historical accounts of 1918 report clinical CFR exceeding **2.5%**. | Do not use this for ordinary seasonal influenza. The 1918 age distribution was unusually severe among young adults. [CDC](https://wwwnc.cdc.gov/eid/article/12/1/05-0979_article) | **M/L** historical denominators |
| **Pulmonary TB** | Untreated, HIV-negative, smear-positive disease: approximately **70% ten-year case fatality**, with study estimates **53–86%**. Smear-negative, culture-positive disease was estimated nearer **20%**. | These are outcomes of **active pulmonary disease**, not outcomes of all TB infections. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0017601) | **M** |
| **Bubonic / pneumonic plague** | Untreated bubonic CFR approximately **30–60%**; untreated pneumonic disease approaches **100%**. | Separate clinical forms and treatment timing. High lethality does not imply exceptionally high respiratory transmissibility. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/plague) | **H/M** |
| **Malaria** | No universal untreated IFR is appropriate. In the African AQUAMAT trial, mortality among children already hospitalized with severe malaria was **8.5%** in the artesunate arm. | This illustrates how serious the severe branch is even with treatment; it is **not** an all-infection or historical fatality parameter. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21062666/) | **H** trial result; **L** historical transfer |
| **Scabies** | Ordinary infestation is primarily a morbidity and secondary-infection problem. | Model itching, impaired sleep and skin complications rather than a large direct epidemic death probability. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/scabies) | **H** qualitative; **L** historical attributable mortality |
| **Syphilis in pregnancy** | Untreated, late-treated or incorrectly treated maternal infection is associated with **50–80% adverse birth outcomes**. | This combines several outcomes; it does **not** mean 50–80% fetal mortality. Separate stillbirth, neonatal death, congenital infection and other adverse outcomes. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/syphilis) | **M** aggregate |

### 3.3 A usable age-specific benchmark: measles

The Sbarra et al. model estimates the following community CFRs for **LMICs in 2019**. These are valuable for testing whether an age-risk function behaves plausibly, but transporting them into a pre-industrial population requires substantial additional uncertainty. [PubMed](https://pubmed.ncbi.nlm.nih.gov/36925172/)

| Age | Estimated CFR | 95% uncertainty interval |
| --- | --- | --- |
| Under 1 year | **3.03%** | 2.89–3.16% |
| 1–4 years | **1.63%** | 1.58–1.68% |
| 5–9 years | **0.84%** | 0.80–0.87% |
| 10–14 years | **0.67%** | 0.64–0.70% |

A similarly credible **disease × age × nutritional status × historical region** matrix is not available for all the diseases requested. Supplying one with precise coefficients would create false confidence.

Instead, use disease-specific functions:

\[
P(\text{severe disease}\mid\text{infection})
=f\_d(\text{age, nutrition, pregnancy, immunity, comorbidity})
\]\[
P(\text{death}\mid\text{clinical course})
=g\_d(\text{severity, reserves, care quality, treatment delay}).
\]

These are **proposed model structures**, not fitted universal equations.

### 3.4 Nutrition must affect different processes separately

A useful causal benchmark comes from the **RATIONS randomized trial in India**: nutritional supplementation of household contacts reduced TB incidence by approximately **39% for all forms** and **48% for microbiologically confirmed pulmonary TB**. That is evidence about developing disease—not a 39–48% reduction in fatality among existing TB patients. [pubmed.ncbi.nlm.nih.gov](https://pubmed.ncbi.nlm.nih.gov/37567200/)

For TCE, distinguish nutrition’s effects on susceptibility, progression, recovery and survival. Also allow illness to worsen nutrition through lost work, impaired intake and care demands. Do not implement a universal “hunger doubles every disease risk” coefficient.

Mortality is not the complete burden. Smallpox can leave scars and visual impairment; malaria involves anemia and pregnancy risks; scabies can produce consequential secondary skin disease. Represent such outcomes as persistent conditions rather than additional deaths. [CDC](https://www.cdc.gov/smallpox/hcp/clinical-signs/index.html)

**Integration warning:** historical all-cause life tables already include infectious deaths. Adding these disease deaths on top of an unchanged historical mortality schedule would double-count them. Calibrate the remaining background mortality accordingly.

---

## 4. Endemic versus epidemic dynamics

### 4.1 Persistence thresholds are conditional, not gates

The classic measles **critical community size** is often placed around **250,000–500,000 people** in particular historical settings. This describes the approximate population scale at which uninterrupted local persistence becomes likely—not the minimum population capable of suffering an outbreak. Strong seasonal forcing can raise the relevant scale considerably; analyses of Niger discuss persistence difficulties even at much larger population sizes. [bpb-us-e1.wpmucdn.com](https://bpb-us-e1.wpmucdn.com/sites.psu.edu/dist/0/175317/files/2023/12/rural-urban-gradient.pdf)

A village of 300 susceptible people can therefore experience a devastating outbreak. It simply cannot be assumed to maintain measles indefinitely afterward.

For TCE, distinguish:

| Pattern | What sustains it? | Expected simulation behavior |
| --- | --- | --- |
| **Acute local epidemic** | A susceptible population plus an introduction | Rapid growth, susceptible depletion, then possible extinction |
| **Recurrent imported epidemics** | External infection plus births/migration rebuilding susceptibility | Quiet intervals interrupted by introductions |
| **Carrier-supported recurrence** | Persistent shedders | Illness can recur after an apparently disease-free interval |
| **Reactivation-supported disease** | Long-lived infection within people | New clinical cases without a recent exposure event |
| **Vector/reservoir-supported transmission** | Compatible host–vector or environmental ecology | Human incidence can persist without a large human population |

### 4.2 Demography supplies susceptibles; it does not guarantee persistence

**Illustrative calculation—not a historical estimate:** suppose a settlement has 20,000 people and 40 births per 1,000 people annually. That produces 800 newborns per year.

Even if approximately 800 people eventually contracted an immunizing disease each year, an eight-day infectious period would imply only

\[
800\times\frac{8}{365}\approx17.5
\]

infectious people on average. Seasonal troughs can be much lower. Such averages do not prevent stochastic extinction.

This is why matching a long-run annual incidence total is insufficient: the model must reproduce troughs, introductions and the distribution of outbreak sizes.

### 4.3 Implement imports explicitly

For a larger world, maintain an external population representation containing population size, pathogen prevalence and travel links. Travelers should arrive with sampled disease stages; contaminated goods or vectors require their own plausible routes.

For a genuinely closed world, permit permanent extinction of pathogens lacking a remaining reservoir. Do not spontaneously recreate measles because enough children have been born. TB reactivation, typhoid carriage and malaria relapse are different processes, supported by persistent infection states. [CDC](https://www.cdc.gov/tb/about/inactive-tuberculosis.html)

---

## 5. Variation across eras and world regions

### 5.1 Model ecological configurations, not fixed eras

The following are **mechanism-based scenario expectations**, not claims that every society passed through an identical epidemiological sequence.

| Settlement configuration | Drivers to represent | Likely implications |
| --- | --- | --- |
| **Mobile foragers and small isolated groups** | Small contact networks, mobility, intermittent aggregation, local animal/vector ecology | Acute human-only infections have difficulty persisting locally. Isolation does not prevent infections with animal reservoirs, long duration or intimate transmission. |
| **Early permanent farming** | Repeated use of water sources, accumulated waste, food storage, changing vector habitat, births and seasonal gatherings | Persistent fecal contamination and some vector habitats can emerge without a large city. Different agricultural practices can increase or decrease particular exposures. |
| **Pre-industrial villages and cities** | Shared wells, food markets, dense housing, servants/caregivers, migration, religious gatherings, trade | Local outbreaks become connected; carrier and water systems can sustain transmission. Large cities can supply infection to smaller settlements. |
| **Early industrial growth** | Crowded housing and workplaces, rapid migration, increasingly centralized water distribution | Centralization can distribute contamination widely before effective treatment and waste separation. More connectivity need not initially mean better health. |
| **Later industrial and modern configurations** | Protected water, functioning sewerage, vaccination, adequate nutrition, effective treatment and surveillance | Different diseases decline through different mechanisms. Unequal access and service breakdowns can preserve or restore risk. |

These implications follow from the route and persistence distinctions above. They should be generated by infrastructure, activity and ecology rather than a global year-based infection modifier.

### 5.2 Regional differences that matter

**West African/Sahelian seasonality.** Niger measles data show why European school-term or winter seasonality should not be universal. Seasonal movement between towns and agricultural areas changes contact intensity; the rural–urban pattern can dominate a simple temperature rule. [bpb-us-e1.wpmucdn.com](https://bpb-us-e1.wpmucdn.com/sites.psu.edu/dist/0/175317/files/2023/12/rural-urban-gradient.pdf)

**South Asia.** Cholera and typhoid scenarios should distinguish river/coastal ecology, water distribution and household food transmission. For influenza, district-level historical work on British India provides a non-European reference for the spatial diffusion of the 1918 pandemic. Nutrition–TB links also have direct experimental evidence from India. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/cholera)

**East and Southeast Asian cities.** Pneumonic-plague model evidence includes outbreaks outside Europe, including Mukden and Rangoon. This supports testing caregiver and household transmission under specific urban conditions, rather than deriving all plague behavior from the European Black Death. [CDC](https://wwwnc.cdc.gov/eid/article/10/4/03-0509_article)

**Tropical Africa, Asia and the Americas.** Malaria should depend on parasite species, competent vectors, habitat, temperature and prior exposure—not a continent tag. Falciparum and vivax need different natural histories, especially because vivax relapse changes apparent seasonality and persistence. [CDC](https://www.cdc.gov/malaria/hcp/clinical-features/index.html)

**Pacific and other resource-poor tropical communities.** Scabies can be a substantial endemic burden rather than a rare incidental ailment. WHO reports child prevalence ranging from **5% to 50%** in some resource-poor settings. This is a modern ecological comparator, not a direct estimate for ancient Pacific societies. [World Health Organization](https://www.who.int/news-room/fact-sheets/detail/scabies)

For American or other historically isolated scenarios, treat pathogen presence and introduction history as explicit initial conditions. Settlement size alone should not conjure every Old World epidemic disease.

### 5.3 Two major historical uncertainties

**Medieval plague transmission remains contested.** Human ectoparasite models have fitted some Second Pandemic mortality curves better than competing models, but methodological critiques argue that those comparisons provide weaker support than originally claimed. Keep competing transmission scenarios available; do not encode “all medieval plague was rat fleas” or “all was human lice” as settled fact. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5819418/)

**Aggregate catastrophe estimates conceal geographic variation.** Pollen-based research on the Black Death finds substantial spatial differences in land-use change. These are indirect demographic proxies, not direct measurements of infection or case fatality. They argue against forcing a uniform continent-wide population loss into every local outbreak. [Nature](https://www.nature.com/articles/s41559-021-01652-4)

---

## 6. Stylized facts a correct simulation should reproduce

Use these as **separate validation experiments**, not as universally simultaneous targets.

| Validation target | Evidence or numerical benchmark | Expected TCE result |
| --- | --- | --- |
| **Measles reaches most susceptible close contacts** | Secondary attack rates can exceed **90%** in close-contact settings. | Strong household transmission without making every outdoor encounter equally dangerous. [CDC](https://www.cdc.gov/pinkbook/hcp/table-of-contents/chapter-13-measles.html) |
| **Long-established measles becomes predominantly a childhood infection** | In the pre-vaccine United States, more than **90%** had immunity from infection by age 15; epidemic cycles commonly occurred every **2–3 years**. | Age at infection and periodicity emerge from births, mixing and immunity—not a rule that only children can catch measles. [CDC](https://www.cdc.gov/pinkbook/hcp/table-of-contents/chapter-13-measles.html) |
| **Symptoms and transmission end at different times** | Shigella shedding can persist **two weeks beyond symptoms**. | A recovered-looking food handler can remain epidemiologically important. [CDC](https://www.cdc.gov/shigella/causes/index.html) |
| **Rare carriers can matter disproportionately** | Typhoid chronic carriage occurs in an estimated **1–4% of treated patients**. | A small number of long-lived shedders can connect outbreaks across time; validate the historical carrier fraction separately. [CDC](https://www.cdc.gov/yellow-book/hcp/travel-associated-infections-diseases/typhoid-and-paratyphoid-fever.html) |
| **Lethality and contagiousness are different** | Pneumonic plague: fitted \(R\_0\) about **1.3**, despite extremely high untreated fatality. | Many short chains, occasional larger outbreaks and strong effects of contact patterns. [CDC](https://wwwnc.cdc.gov/eid/article/10/4/03-0509_article) |
| **TB is slow but demographically consequential** | Untreated active smear-positive disease has high mortality over years, not days. | Chronic work loss, households affected repeatedly, and deaths after long disease courses. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0017601) |
| **Malaria cannot respond instantaneously to weather** | Parasite development within mosquitoes commonly requires **9–18 days**. | Habitat changes and rainfall affect future transmission through vector growth, infection and survival delays. [CDC](https://www.cdc.gov/dpdx/malaria/index.html) |
| **Care changes mortality without necessarily preventing exposure** | Cholera CFR can fall below **1%** with adequate prompt care. | Mortality and incidence can move differently when treatment improves. [CDC](https://www.cdc.gov/yellow-book/hcp/travel-associated-infections-diseases/cholera.html) |
| **Pandemic age patterns can differ from seasonal ones** | The 1918 influenza pandemic had unusually high young-adult mortality. | Strain/history-specific severity rather than a permanent influenza age curve. [CDC](https://wwwnc.cdc.gov/eid/article/12/1/05-0979_article) |

Also test basic structural invariants: no infection without a source or persistent internal state; no negative susceptible counts; no double death; and no artificial survival of an extinct transmission chain.

---

## 7. Recommended agent-based implementation

### 7.1 Shared framework, disease-specific state machines

The following is a **proposed TCE architecture**:

| Component | Suggested contents |
| --- | --- |
| **Person health** | Age, pregnancy, nutritional reserves, relevant chronic conditions, immune history |
| **Infection episode** | Pathogen/strain, acquisition time, route/source, clinical stage, infectiousness schedule, scheduled transitions |
| **Persistent infection** | Typhoid carrier status, TB infection/progression state, malaria blood/liver stages |
| **Place exposure** | Occupancy schedule, ventilation, contaminated water/food, sanitation connections |
| **Vector/reservoir patch** | Vector abundance and infection stages, habitat, temperature, relevant animal infection |
| **Institution service** | Capacity, access, supplies, detection delay, treatment/isolation coverage |
| **Observation record** | What citizens and authorities believe, separate from the simulation’s true disease state |

Do not make agents or institutions omniscient. A citizen can know that several neighbors have diarrhea without knowing which well or pathogen caused it.

### 7.2 Integrate exposure rather than performing arbitrary daily rolls

For person \(i\), pathogen \(d\), and time interval \(\Delta t\):

\[
P\_{i,d}(\text{infection})
=
1-\exp\!\left[-\sum\_r H\_{i,d,r}\right],
\]

where \(H\_{i,d,r}\) is the **dimensionless integrated infection hazard** from route \(r\).

For a constant route-specific hazard rate \(\lambda\), \(H=\lambda\Delta t\). This makes the meaning of the time step explicit. Accumulate routes before drawing the infection event, rather than permitting multiple independent acquisitions of the same infection in one update.

A compact shared-air approximation could use:

\[
\frac{dC}{dt}
=
\frac{\text{infectious emissions}}{\text{room volume}}
-
\text{removal rate}\times C.
\]

For water, use a mass-balance analogue with shedding inputs, flow, dilution and loss of viability. These are proposed approximations; their coefficients must be calibrated consistently with the model’s units.

### 7.3 Sample durations with distributions

Avoid memoryless daily recovery rolls for every disease. The shapes of latent and infectious-period distributions affect epidemic dynamics and inferred intervention effectiveness. Wearing, Rohani and Keeling demonstrate why these assumptions matter. [PLOS](https://journals.plos.org/plosmedicine/article?id=10.1371%2Fjournal.pmed.0020174)

Gamma or lognormal durations are practical defaults when supported by data. For an arithmetic mean \(m\) and standard deviation \(s\), a lognormal distribution can be parameterized as:

\[
\sigma^2=\ln\left(1+\frac{s^2}{m^2}\right),
\qquad
\mu=\ln(m)-\frac{\sigma^2}{2}.
\]

The historical pneumonic-plague means and standard deviations above therefore provide a directly implementable example. Do not independently sample symptom and infectiousness clocks in ways that create biologically impossible ordering.

### 7.4 Preserve source tracing without inventing certainty

Store an acquisition record identifying a person, room, water node, food batch or vector cohort.

For a well contaminated by several people, the defensible source may be **“well 17, contamination mixture during days 120–123”**, not one uniquely identifiable person. TCE may maintain probabilistic ancestry internally, but a mixed environmental exposure should not be presented as certain individual attribution.

### 7.5 Practical performance choices for 10k–50k people

These are engineering recommendations, not measured TCE benchmarks:

* Process infectious people and affected places rather than scanning all person pairs.
* Represent mosquitoes and fleas as patch-level cohorts, not individually rendered agents.
* Use scheduled stage transitions and within-day exposure blocks; retain subdaily events for fast deterioration.
* Keep chronic progression separate from acute exposure updates.
* Preserve the epidemiological population when rendering detail changes.

FRED provides a relevant precedent for synthetic populations linked through households, schools and workplaces, with disease propagation through activity locations. Its modern demographic assumptions should not be imported wholesale into pre-industrial societies. [DOI](https://doi.org/10.1186/1471-2458-13-940)

### 7.6 Calibrate mortality dynamically

Care delivered after illness begins should still change survival. Prefer stage-specific mortality hazards or revisable clinical-course probabilities over an irreversible “will die” flag drawn at infection.

For each reference scenario, jointly fit incidence, timing, age distribution, severity and deaths. Fitting only total deaths allows compensating errors—for example, too many infections combined with an unrealistically low fatality probability.

Separate uncertainty **between runs**—such as unknown historical nutrition effects—from random variation **within runs**, such as which contact becomes infected. OpenMalaria’s distinction between stochastic, parameter and model uncertainty is useful here. [GitHub](https://github.com/OpenMalaria-Org/openmalaria/wiki)

---

## 8. Existing models and datasets worth using

### 8.1 Models

| Model or study | What to borrow | Main caution |
| --- | --- | --- |
| **FRED — Grefenstette et al., 2013** | Synthetic people, activity locations, household/workplace structure, epidemic event logging | Modern census and activity assumptions are not historical defaults. [DOI](https://doi.org/10.1186/1471-2458-13-940) |
| **OpenMalaria** | Individual human infection, heterogeneous exposure, immunity, vector dynamics, alternative model structures | Use as a methodological and validation reference, not a drop-in Rust subsystem. [GitHub](https://github.com/OpenMalaria-Org/openmalaria/wiki) |
| **Chao, Halloran & Longini, 2011: cholera vaccination modeling in Haiti** | Spatially structured individual-based cholera modeling and intervention comparisons | Parameters are tied to a particular epidemic and environmental setting. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3084143/) |
| **Gani & Leach, 2004: pneumonic-plague outbreak model** | Empirical stage distributions and heterogeneous secondary-case counts | Not a complete model of bubonic plague or medieval reservoir ecology. [CDC](https://wwwnc.cdc.gov/eid/article/10/4/03-0509_article) |

These scientific models are more useful calibration references than commercial games, whose displayed disease behavior often omits or conceals its epidemiological assumptions.

### 8.2 Data sources

| Source | Useful targets | Limitations |
| --- | --- | --- |
| **Project Tycho** | Historical reported infectious-disease time series; seasonality, epidemic timing and changes around interventions | Reported cases are not all infections; reporting and diagnostic practices change. [Tycho](https://www.tycho.pitt.edu/) |
| **International Infectious Disease Data Archive — IIDDA** | Classic infectious-disease series for testing epidemic models | The archive is now available through a repository; inspect dataset-specific documentation rather than assuming consistent coverage. [GitHub](https://github.com/canmod/iidda) |
| **Death by Numbers: London Bills of Mortality** | Weekly spatial and cause-of-death patterns, **1603–1752** | Digitization is ongoing; historical cause labels are not laboratory diagnoses. [Death by Numbers](https://deathbynumbers.org/) |
| **Malaria Atlas Project** | Parasite prevalence, geographical variation and transmission-related spatial data | Modern ecological comparator, not a reconstruction of ancient malaria burden. [Malaria Atlas Project](https://malariaatlas.org/) |
| **Ferrari et al., Niger measles study** | Non-European seasonality, rural–urban coupling and persistence | Published study data and setting-specific assumptions; verify raw-data availability separately. [bpb-us-e1.wpmucdn.com](https://bpb-us-e1.wpmucdn.com/sites.psu.edu/dist/0/175317/files/2023/12/rural-urban-gradient.pdf) |
| **Pre-chemotherapy TB cohorts synthesized by Tiemersma et al.** | Untreated disease duration and long-term outcomes | Historical selection, diagnosis and follow-up differ across cohorts. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0017601) |
| **Sbarra et al. measles CFR study** | Age-, location- and setting-specific mortality calibration | Modeled modern estimates, with incomplete coverage of displaced and other exceptional populations. [PubMed](https://pubmed.ncbi.nlm.nih.gov/36925172/) |

### Evidence gaps to retain explicitly

The largest unresolved calibration issues are pathogen-specific mortality in early societies; age-by-nutrition interactions; historical asymptomatic and carrier fractions; environmental survival under specific water conditions; and the exact mechanisms behind particular historical epidemic curves.

A sensible parameter record should therefore include **value/distribution, units, disease stage, denominator, population, treatment context, source, confidence and transportability notes**. “Historical CFR = 10%” is not an adequate record.

## Bottom line

For TCE, prioritize **five distinct persistence mechanisms**: acute immunizing outbreaks, environmental transmission, chronic carriers, latent/reactivating infection, and vector-mediated infection.

Make transmission follow actual activities and sources; make disease severity respond to the person and available care; and let local extinction occur. With those foundations, changing settlement density, infrastructure, food security, trade and institutions can generate different epidemiological regimes **without hard-coded eras, disease rates or population-growth penalties**.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92822-18b0-83ea-aaae-2a739cfc8dd9)
