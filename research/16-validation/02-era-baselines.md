# Historical baselines for The Civilization Engine

## Executive recommendation

**Use historical baselines as conditional ranges for outcomes, not as an era progression system.** A society should resemble historical societies with comparable food production, transport, energy, disease environments and institutions—not societies assigned the same technology label.

For TCE, the most useful checks are **joint constraints**: whether the agricultural workforce can feed the population; whether towns have sufficient hinterlands or imports; whether the state can actually support its soldiers; and whether age-specific mortality produces a plausible population structure. Matching GDP per capita while violating those constraints would not constitute a realistic world.

The evidence also has very uneven resolution. Modern population and employment statistics support relatively narrow comparisons. Ancient population, literacy, income and taxation often support only broad, disputed reconstructions. The report therefore distinguishes:

| Label | Meaning |
| --- | --- |
| **H — higher confidence** | Relatively strong statistical or administrative evidence, although definitions still matter. |
| **M — medium confidence** | Historical reconstruction with meaningful uncertainty. |
| **L — low confidence** | Sparse evidence, uncertain denominators or highly assumption-sensitive reconstruction. |
| **P — proposed modeling range** | A suggested starting range for TCE experiments—not a published empirical interval or confidence interval. |

---

## 1. Mechanisms: rules that should generate the historical patterns

The equations below are modeling recommendations. They express constraints and causal hypotheses rather than universally estimated historical relationships.

### 1.1 Food production constrains specialization

Track food physically before valuing it monetarily:

\[
F\_{\text{available}}=
F\_{\text{harvest}}+F\_{\text{imports}}
-F\_{\text{seed}}-F\_{\text{feed}}-F\_{\text{losses}}-F\_{\text{exports}}.
\]

Households must obtain their shares of this food through production, exchange, transfers or coercion. Aggregate sufficiency must not automatically prevent household starvation.

A useful accounting constraint is:

\[
f\_A \geq \max\left(0,\frac{q-m}{e\,a}\right),
\]

where:

* \(f\_A\) is agriculture’s share of annual labor in full-time equivalents;
* \(q\) is annual food consumption per person;
* \(m\) is net food imports per person;
* \(e\) is total labor full-time equivalents per resident;
* \(a\) is net food output per agricultural full-time equivalent.

Keep **output per hectare** separate from **output per worker**. A labor-intensive farming system can support dense settlement without releasing most workers into nonagricultural occupations. Foraging carrying capacity likewise depends on ecological productivity and seasonality, not merely land area; the FORGE model explicitly represents these constraints. [Nature](https://www.nature.com/articles/s41559-021-01548-3)

**Implementation:** farms choose land, crops and labor schedules; techniques change yields, labor requirements and risks separately. Specialization expands only when food, exchange opportunities and demand support it.

### 1.2 Settlement growth requires a support network

Let households evaluate settlements using expected consumption, employment, access to land, safety, social ties and obligations, minus housing and movement costs.

Cities need not arise only from wage advantages. Administrative, military, religious and trading functions can sustain concentration. Nevertheless, large nonfarming populations must be provisioned: Roman urban consumption, for example, depended on extensive interregional supply systems. [Cambridge University Press](https://www.cambridge.org/core/books/abs/cambridge-economic-history-of-the-grecoroman-world/early-roman-empire-production/D3F384EB636F47D422B2AC9EF2551F39)

**Implementation:** calculate each settlement’s food catchment, imports, transport labor and storage requirements. Do not grant a city a population capacity independently of the supporting economy.

### 1.3 Population growth emerges from age-specific demography

Use births and deaths by age, not a single “lifespan” parameter. One suitable mortality structure is a Siler-type hazard:

\[
h(x)=a\_1e^{-b\_1x}+a\_2+a\_3e^{b\_3x},
\]

representing declining childhood risk, background mortality and increasing late-life mortality. Convert the annual hazard into a daily event probability:

\[
p\_{\text{death,day}}=1-e^{-h(x)/365.2425}.
\]

Fit the parameters to an appropriate life table. Hunter-gatherer demographic research provides examples in which low life expectancy at birth coexists with substantial survival into older adulthood. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/GurvenKaplan2007pdr.pdf)

**Implementation:** apply nutrition, infection, childbirth, injury and environmental modifiers to relevant hazards. Fertility should depend on age, partnership, birth spacing and household circumstances. Neither mortality nor fertility should change automatically because an era label changed.

### 1.4 Household income matters more than a single occupation’s wage

An agent’s wage is not household income, and household income is not GDP. Historical household reconstructions can change substantially when women’s production and additional earners are included. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1468-0289.2010.00515.x)

**Implementation:** allow mixed seasonal work, household pooling, own-consumption production, rents and transfers. A farming household may also weave, transport goods or provide military service. Record both primary occupation and actual hours by activity.

### 1.5 Literacy is accumulated through institutions and cohorts

Model reading, writing and numeracy separately. Access requires teachers, learning materials, time and incentives; costs and access can differ by status, gender and location. Adult literacy can rise slowly even after children’s schooling expands because older cohorts remain in the population. UNESCO explicitly identifies cohort replacement as an important contributor to literacy change. [UNESCO](https://www.unesco.org/gem-report/en/literacy-adult-learning)

**Implementation:** schools and apprenticeships produce skills, rather than a technology unlock granting literacy to everyone. Record oral expertise separately from written literacy.

### 1.6 Fiscal capacity is not the statutory tax rate

A workable fiscal relationship is:

\[
R\_{\text{net}}=
\sum\_b B\_b\,\tau\_b\,c\_b
-\text{collection costs}
-\text{unremitted receipts},
\]

where \(B\_b\) is a taxable base, \(\tau\_b\) its formal rate and \(c\_b\) effective collection.

Distinguish what households surrender, what local collectors retain and what reaches the central treasury. Ottoman and Qing research shows why these distinctions matter; low central receipts need not mean equally low extraction from the population. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914)

**Implementation:** collection requires records, staff, access, enforcement or cooperation. Rent, religious dues, tribute, tax, requisition and corvée should remain separate obligations.

### 1.7 Military strength is constrained by finance and logistics

Use:

\[
N\_{\text{deployable}}
=\min(N\_{\text{recruited}},N\_{\text{equipped}},
N\_{\text{funded}},N\_{\text{supplied}}).
\]

A soldier’s cost includes more than pay: food, equipment replacement, transport, animals and supporting labor matter. Qing military history particularly illustrates the interaction between military organization, provincial finance and civilian logistics. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/qing-military-institutions-and-their-effects-on-government-economy-and-society-16401800/E1FDAA965C5B67C12274B2B3408AE206)

**Implementation:** distinguish full-time troops, seasonal levies, reserves, garrison personnel and campaign forces. A levy during harvest should cost more agricultural output than an equal number of service-days in a slack season.

---

## 2. Measurement contract: define the dashboard before calibrating it

The following are recommended TCE definitions. Historical comparisons should retain the source’s original denominator alongside the corresponding simulation measure.

| Metric | Recommended definition |
| --- | --- |
| **Territorial density** | Total residents / total land area. |
| **Inhabited-region density** | Residents / explicitly delimited settled landscape, including its working countryside. |
| **Built-up density** | Settlement residents / built-up footprint; distinguish this from residential parcel density. |
| **Urbanization** | Report shares in settlements of at least **2,000, 5,000 and 10,000** residents separately: \(U\_{2k},U\_{5k},U\_{10k}\). |
| **Agricultural employment** | Report both primary-occupation headcount and agriculture’s share of annual labor hours. |
| **GDP per capita** | Annual value added, including imputed own-consumed production, divided by population—not the sum of all sales. |
| **Real living standards** | Household disposable resources divided by a locally priced reference consumption basket. |
| **Literacy** | Adult reading and writing rates, with a specified age threshold; track numeracy independently. |
| **Life expectancy** | Period life expectancy from age-specific mortality; also report survival to 5, 15 and 45. |
| **Tax take** | Separate central receipts, consolidated government receipts and household extraction. |
| **Military participation** | Actual active personnel / total population, with a second measure for annual military labor-days. |

The urbanization distinction is especially important. The UN’s 2025 revision provides both national definitions and a harmonized classification of cities, towns and rural areas. Those are not interchangeable series. [World Population Prospects](https://population.un.org/wup/downloads)

---

## 3. Population density, settlement sizes and urbanization

### 3.1 Starting spatial ranges

These are **P ranges for initial experiments**, not estimates of what every society in a stage looked like. “Inhabited region” includes fields, pasture and other supporting land—not just buildings.

| Configuration | Inhabited-region density, people/km² | Settlement populations to exercise in tests | Interpretation |
| --- | --- | --- | --- |
| Mobile terrestrial foraging | **0.01–1** | Camps **20–60** | Productive aquatic environments require separate configurations. |
| Early farming | **1–30** | Villages **50–500**; larger settlements **500–2,000** | Do not prohibit exceptional settlements numbering several thousand. |
| Established agrarian systems | **10–100** | Villages **100–1,000**; towns **2,000–20,000**; cities **20,000–200,000** | Intensive irrigated landscapes can exceed the density range. |
| Industrializing systems | **No useful era-only density bound** | Towns and cities from thousands to **1 million+** | Energy, transport, migration and settlement boundaries become decisive. |
| Modern high-productivity systems | **No useful era-only density bound** | Small settlements through **10 million+** metropolitan systems | A low-density country and a dense urban economy can both be technologically advanced. |

These deliberately broad priors should be replaced by ecology-specific comparisons. The observations below show why a universal stage-density coefficient would fail.

### 3.2 Empirical spatial anchors

| Society or settlement | Reference numbers | Confidence and qualification |
| --- | --- | --- |
| **Aché, Paraguay**, ethnographic sample | **0.04 people/km²**; mean residential group approximately **43** | **M** for the sample; not a direct reconstruction of Paleolithic populations. |
| **Hadza, Tanzania**, ethnographic sample | **0.24 people/km²**; mean residential group approximately **29** | **M**; ecological and historical context matters. |
| **Ju/’hoansi, southern Africa**, ethnographic sample | **0.07 people/km²**; mean residential group approximately **17** | **M**; again, not an all-forager constant. |
| **Çatalhöyük**, peak around 6700–6500 BCE | Approximately **3,500–8,000 residents** | **L–M**; house occupancy and contemporaneity assumptions produce a wide range. |
| **Teotihuacan**, first millennium CE | Revised estimate approximately **100,000** over **18.76 km²**, about **5,300/km²** | **M**; population reconstruction from residential compounds. |
| **Greater Angkor**, medieval peak | Approximately **700,000–900,000** over **3,000 km²** | **L–M**; a dispersed urban–agrarian complex, not a compact city. |

Sources: Smith and colleagues’ comparative hunter-gatherer data; the Çatalhöyük bioarchaeological literature; Smith et al. on Teotihuacan; and Klassen et al. on Greater Angkor. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/)

The Angkor figures imply roughly **230–300 people/km² across the settlement complex**. That arithmetic does not conflict with Teotihuacan’s much higher built-up density: the denominators describe different spatial objects.

Even famous capitals remain uncertain. Published reconstructions of ancient Rome include approximately **450,000** and conventional estimates around **750,000–1 million**. A dashboard should preserve competing reconstructions rather than treat “Rome = 1 million” as a measured constant. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/population-of-ancient-rome/BACD7DF32B0B77609CD6713B8AF88882)

### 3.3 Urbanization anchors

Historical urbanization estimates depend strongly on minimum settlement size and territorial coverage.

| Place and period | Estimated share | Definition / confidence |
| --- | --- | --- |
| Europe, around **1500** | **10–11.5%** | Bairoch reconstruction, settlements above roughly 5,000; **M–L** |
| Europe, around **1700** | **12–13%** | Same broad historical framework; **M–L** |
| China, early **19th century** | **6–7.5%** | Settlements above 5,000 in the cited reconstruction; **M–L** |
| World, **1950** | **20% in cities** | UN harmonized classification; historical estimates, **M** |
| World, **2025** | **45% in cities** | UN harmonized classification; **H–M** |

The historical estimates are reported in Zinkina and colleagues’ discussion of the Bairoch series; the modern comparison comes from UN World Urbanization Prospects 2025. **The historical and modern rows are not one continuous, identically defined series.** [Sociological Studies](https://www.sociostudies.org/almanac/articles/the_nineteenth-century/)

**TCE recommendation:** use **5–15% in settlements above 5,000** as an initial agrarian-world test range, with **15–30%** as a separate commercial, administrative or import-supported configuration. These are **P ranges**, not universal historical limits. On a small map, thresholds can also produce abrupt percentage jumps.

---

## 4. Income: GDP per capita and real wages

### 4.1 Maddison benchmarks

For a reproducible comparison, fix the database release and purchasing-power basis.

The following panel uses **Maddison Project 2020 regional estimates as published in OECD’s 2021 *How Was Life? Volume II***, in **2011 international dollars per person per year**. Values are rounded; they are not current-dollar incomes. [OECD](https://www.oecd.org/en/publications/how-was-life-volume-ii_3d96efc5-en/full-report/component-7.html)

| Region | 1820 | 1870 | 1950 | 2016 |
| --- | --- | --- | --- | --- |
| Western Europe | 2,310 | 3,300 | 7,260 | 38,510 |
| Eastern Europe | 820 | 1,580 | 4,080 | 19,450 |
| Western offshoots | 2,510 | 4,650 | 14,770 | 51,670 |
| Latin America and Caribbean | 950 | 1,320 | 3,710 | 14,090 |
| East Asia | 1,090 | 990 | 1,120 | 15,700 |
| South and Southeast Asia | 930 | 850 | 1,070 | 6,990 |
| Middle East and North Africa | 970 | 1,170 | 2,390 | 18,010 |
| Sub-Saharan Africa | 800 | 800 | 1,320 | 3,490 |
| **World** | **1,170** | **1,500** | **3,350** | **14,700** |

**Confidence:** early observations are generally **L–M**, later ones **M–H**, varying by region. In particular, sparse early African evidence and imputation mean repeated values must not be interpreted as measured stagnation. [OECD](https://www.oecd.org/en/publications/how-was-life-volume-ii_3d96efc5-en/full-report/component-7.html)

For the production reference database, use **Maddison Project Database 2023**, which covers **169 countries through 2022**, and retain its country-source documentation. Do not silently combine its observations with older Maddison vintages. [University of Groningen](https://www.rug.nl/ggdc/historicaldevelopment/maddison/releases/maddison-project-database-2023?lang=en)

### What to do before credible GDP estimates exist

**Do not assign a precise Maddison-dollar GDP to a generic Neolithic village.** For such configurations, use physical and household measures instead:

| Early-society indicator | Simulation measure |
| --- | --- |
| Food adequacy | Consumption relative to age- and activity-adjusted requirements |
| Security | Household distribution of stored food and days of cover |
| Surplus | Net food available after producer households’ requirements |
| Labor burden | Annual hours needed to obtain basic consumption |
| Material consumption | Housing space, clothing, fuel, tools and livestock per person |
| Inequality | Household consumption and asset distributions |

A fixed-price output index remains useful internally. Its level should not masquerade as an independently established ancient PPP income estimate.

### 4.2 Real wages

Allen and colleagues’ “welfare ratio” compares standardized annual earnings with a basic family consumption basket. Their convention uses **250 workdays**, three adult-equivalent baskets and a small rent allowance; these are comparison assumptions, not universal household behavior. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1468-0289.2010.00515.x)

| Reconstruction | Approximate basic-family-basket purchasing power | Confidence |
| --- | --- | --- |
| London and Amsterdam laborer, around the mid-18th century | **About 4 baskets/year** | **M**, basket- and employment-sensitive |
| Oxford laborer, comparable period | **About 2.5–3** | **M** |
| Authors’ Yangzi household illustration: male agricultural earnings alone | **0.53** | **L–M**, constructed household example |
| Same illustration including wife’s textile earnings | **1.18** | **L–M**, dependent on assumed work availability |

These comparisons are contested in their representativeness: urban male wages, rural production, household earnings, rent and basket composition are not interchangeable. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1468-0289.2010.00515.x)

**TCE recommendation:** report both an occupational wage index and actual household consumption adequacy. A low male wage must not automatically classify a household as starving when other production supports it.

---

## 5. Life expectancy, literacy and occupational structure

### 5.1 Stage-oriented demographic and labor ranges

Except for the explicitly identified hunter-gatherer life-expectancy range, the following are **P initialization ranges**. They are useful search spaces, not empirical confidence intervals.

| Configuration | Agriculture’s share of annual labor | Life expectancy at birth | Literacy treatment |
| --- | --- | --- | --- |
| Foraging without cultivation | **0% agriculture**, but substantial food-acquisition work | **21–37 years** in the comparative ethnographic sample | Do not equate absence of writing with absence of knowledge. |
| Early farming | **80–95% P** | **20–35 years P** | Zero written literacy where no script exists; otherwise model specialist access. |
| Predominantly agrarian state | **60–90% P** | **25–40 years P** | Potentially a small minority; no reliable universal ancient percentage. |
| Commercial preindustrial economy | **35–65% P** | **25–45 years P** | Strong occupational, regional and institutional variation. |
| Industrializing economy | **15–60% P** | **35–65 years P** | Allow education and health transitions to occur at different times. |
| High-productivity modern economy | **1–10% P** | **70–85+ years P** | High literacy is a possible institutional outcome, not an automatic technology effect. |

These configurations are not dates. A contemporary economy can retain a large agricultural workforce, and a preindustrial economy can have substantial nonfarm employment.

### 5.2 Low life expectancy does not mean adults die at thirty

In Gurven and Kaplan’s comparative hunter-gatherer sample, life expectancy at birth spans approximately **21–37 years**. Across their hunter-gatherer cases, expected remaining life at age 45 averages about **21 years**. Sweden in 1751–1759 also had life expectancy at birth around **34**, with about **20 additional years** expected at age 45. These are population life-table estimates, not a rule that every survivor reaches the same age. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/GurvenKaplan2007pdr.pdf)

A simulation that has almost no older adults because its “life expectancy” is 30 is therefore misinterpreting the statistic.

### 5.3 Regional mortality transition

The OECD historical reconstruction reports the following **decadal averages**, not single-year observations:

| Region | 1900s: life expectancy at birth, years | 1950s: years |
| --- | --- | --- |
| Western Europe | 46.3 | 68.2 |
| Eastern Europe | 35.8 | 62.5 |
| Latin America and Caribbean | 29.4 | 53.5 |
| East Asia | — | 47.4 |
| South and Southeast Asia | 24.5 | 41.0 |
| Middle East and North Africa | — | 43.5 |
| Sub-Saharan Africa | — | 37.9 |
| **World** | **30.8** | **50.8** |

Confidence is generally **M**, lower where registration was incomplete; missing values mean insufficient coverage, not zero. The same historical compilation puts world adult literacy around **12% in 1820**, itself a rough reconstruction. [OECD](https://www.oecd.org/content/dam/oecd/en/publications/reports/2014/10/how-was-life_g1g43e0f/9789264214262-en.pdf)

For a modern anchor, UN World Population Prospects 2024 estimated global life expectancy at **73.3 years in 2024**. [United Nations](https://www.un.org/sustainabledevelopment/blog/2024/07/press-release-wpp2024/)

### 5.4 Literacy anchors and limitations

| Reference | Number | Confidence / interpretation |
| --- | --- | --- |
| World adult literacy, reconstructed **1820** | **About 12%** | **L–M**, sparse historical evidence |
| Global adult literacy, **UNESCO 2026 data release** | **88%** | **H–M**, underlying country observation years vary |
| Global youth literacy, same release | **93%** | **H–M**, youth and adult populations differ |

The modern figures are release-level indicators, not a claim that every country conducted a literacy assessment in 2026. [UNESCO Institute for Statistics](https://www.uis.unesco.org/en/news/2026-education-data-refresh)

For ancient and medieval societies, keep reading, writing, signatures, school attendance and mastery of an administrative script as different measures. Where evidence is thin, **“unknown” is preferable to a universal 5% literacy default**.

### 5.5 Agricultural employment anchors

| Society / period | Agricultural share | Confidence and denominator |
| --- | --- | --- |
| Britain, **1759**, historical reconstruction | **37%** | **M**, labor force; industry approximately **34%** |
| World, **2022** | **26.2%** | **H–M**, employment; approximately **892 million people** |
| Africa, **2022** | **48%** | **H–M**, employment |
| Europe, **2022** | **5%** | **H–M**, employment |

Britain’s historical estimate comes from Broadberry, Campbell and van Leeuwen; modern figures are FAO estimates. The historical headcounts and modern employment definitions are not identical to TCE’s proposed annual-hours measure. [IDEAS/RePEc](https://ideas.repec.org/a/eee/exehis/v50y2013i1p16-27.html)

This is a particularly important sanity check: **“preindustrial = 90% farmers” is not a valid universal rule**, while “modern = almost no farmers” describes only some contemporary economies.

---

## 6. Tax take and army size

### 6.1 Fiscal reference numbers

| Government / period | Revenue relative to output | Confidence and scope |
| --- | --- | --- |
| Most European states, **16th century** | **Below 5%** | **M–L**, reconstructed central revenue/GDP |
| Many European states, **1780s** | **5–10%** | **M**, central revenue/GDP |
| Britain and Netherlands, **1780s** | **Above 10%** | **M**, central revenue/GDP |
| Early Qing China | **Around 4%** | **L–M**, formal government revenue/output |
| Qing China before the Opium War | **Around 1%** | **L–M**, formal revenue; not total household extraction |

European comparisons come from Karaman and Pamuk. Qing figures follow Taisu Zhang’s reconstruction. The historical GDP denominators and boundaries of recorded revenue introduce substantial uncertainty. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914)

For modern comparison, the OECD’s **2025 African revenue-statistics release**, reporting **2023**, gives:

| Country group | Average tax/GDP |
| --- | --- |
| Africa, 38-country sample | **16.1%** |
| Asia and Pacific | **19.6%** |
| Latin America and Caribbean | **21.3%** |
| OECD | **33.9%** |

These are country-group averages of tax revenue, including social contributions where applicable, rather than population-weighted historical central-treasury ratios. The African sample ranges from **2.9% to 34.0%**, demonstrating how misleading one “modern tax rate” would be. [OECD](https://www.oecd.org/en/publications/revenue-statistics-in-africa-2025_8d3bf3af-en.html)

**Modeling implication:** distinguish three outputs:

\[
\text{household extraction}
\neq
\text{consolidated public revenue}
\neq
\text{central treasury receipts}.
\]

Also keep borrowing separate from revenue. A wartime budget can exceed current taxes without implying unlimited sustainable capacity.

### 6.2 Military reference numbers

| Case | Reference quantity | Confidence / caveat |
| --- | --- | --- |
| Roman Empire, approximately early 2nd century CE | About **380,000 legionaries and auxiliaries** | **M**; ground forces, not an all-service modern definition |
| Roman population denominator, Antonine period | Approximately **55–65 million** | **L–M** |
| Derived Roman ground-force share | Approximately **0.6–0.7% of population** | Arithmetic from uncertain, approximately contemporary reconstructions |
| Qing China, mid-18th century | Possibly **800,000 troops**: approximately **200,000 Banner** and **600,000 Green Standard** personnel | **M–L**; establishment estimates do not equal a single deployable field army |
| United States, peak **1945** | Approximately **12.3 million military personnel** | **H–M**; peak concurrent strength, not cumulative wartime service |

Sources: Hassall’s Roman army reconstruction and the Cambridge Roman demographic synthesis; Dai on Qing military institutions; and the National WWII Museum’s home-front statistics. [Cambridge University Press](https://www.cambridge.org/core/books/abs/cambridge-ancient-history/army/962EFC9627C4FEE8D98199246D8476D9)

For initial TCE stress tests, I recommend these **P ranges**, measured against the entire population:

| Military configuration | Active full-time personnel / population |
| --- | --- |
| Ordinary standing-force scenario | **0.2–1% P** |
| Heavily militarized scenario | **1–3% P** |
| Short-duration mass-mobilization stress test | **3–10% P** |

These are **scenario settings, not historical universal limits**. Nonstate communities may have few full-time soldiers but substantial part-time fighting capacity. Conversely, a large military establishment may contain personnel unavailable for campaigning.

For systematic comparisons after 1816, use the **Correlates of War National Material Capabilities dataset, version 7.0**, which covers **1816–2022**. Normalize personnel and population units before calculating participation ratios. [Correlates of War](https://correlatesofwar.org/data-sets/national-material-capabilities/)

---

## 7. Variation: use configurations rather than regional bonuses

### Foragers and early farmers

Do not model all foragers as equally sparse or all farming as an immediate improvement in individual welfare. The ethnographic density differences and exceptional scale of Çatalhöyük already show that subsistence category alone is insufficient. Use ecological productivity, seasonal scarcity, mobility, storage and disease exposure as separate variables. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/)

### East and Southeast Asia

The evidence permits combinations such as extensive agrarian settlement, very large urban–agrarian complexes and low formal fiscal receipts. Angkor’s dispersed form should not be generated by simply enlarging a compact European-style town. Qing fiscal outcomes should likewise not be reduced to “strong ruler means high taxes”: explanations emphasize both administrative incentives and political commitments concerning taxation. [DOI](https://doi.org/10.1126%2Fsciadv.abf8441)

### Europe and the Mediterranean

Commercialization can precede industrial power, as Britain’s eighteenth-century occupational structure demonstrates. Large ancient cities also require an interregional economy rather than modern industry. These are separate paths toward substantial specialization, not evidence for a single obligatory sequence. [IDEAS/RePEc](https://ideas.repec.org/a/eee/exehis/v50y2013i1p16-27.html)

### The Americas

Teotihuacan provides a strong counterexample to treating large, dense urbanism as requiring European institutional or technological packages. Its residential compounds also suggest that household organization and building form should influence population density directly. [Cambridge University Press](https://www.cambridge.org/core/journals/ancient-mesoamerica/article/apartment-compounds-households-and-population-in-the-ancient-city-of-teotihuacan-mexico/1BED268FF26F27FD2A64B38851EF32D0)

### Africa

Modern employment and fiscal data show enormous differences from—and within—other regional groupings. Early GDP evidence is much thinner. TCE should therefore avoid converting missing historical observations into low fixed productivity, weak institutions or an assumed absence of complex settlement. [FAOHome](https://www.fao.org/statistics/highlights-archive/highlights-detail/employment-indicators-2000-2022-%28september-2024-update%29/)

**Recommended design principle:** represent regional variation through soils, rainfall, disease ecology, domesticates, transport opportunities, household systems and institutions. Region names should select historical comparison cases, not supply inherent productivity multipliers.

---

## 8. Stylized facts and concrete sanity checks

| Pattern to reproduce | Quantitative reference or test | Dashboard interpretation |
| --- | --- | --- |
| **Specialization requires food support.** | Verify the food/labor inequality and actual import flows. | A food-deficit city should consume stocks, attract supply, shrink or suffer—not remain unaffected. |
| **Urbanization is not simply industrialization.** | Historical European and Chinese urban shares differ substantially before industrialization. | Do not require factories before towns emerge. [Sociological Studies](https://www.sociostudies.org/almanac/articles/the_nineteenth-century/) |
| **Low life expectancy can coexist with older adults.** | Hunter-gatherer \(e\_0\) approximately **21–37**, but substantial remaining life at 45. | Flag age structures that eliminate most older adults solely through a “lifespan” setting. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/GurvenKaplan2007pdr.pdf) |
| **Agricultural employment can fall while output rises.** | FAO reports falling global agricultural employment share alongside strongly increasing agricultural value added since 2000. | Check productivity and output, not employment share alone. [FAOHome](https://www.fao.org/newsroom/detail/fao-statistical-yearbook-2024-reveals-critical-insights-on-the-sustainability-of-agriculture-food-security-and-the-importance-of-agrifood-in-employment/) |
| **Commercial preindustrial societies need not be overwhelmingly agricultural.** | Britain’s reconstructed agricultural labor share: **37% in 1759**. | Allow trade and nonfarm production to reduce the farm share before steam power. [IDEAS/RePEc](https://ideas.repec.org/a/eee/exehis/v50y2013i1p16-27.html) |
| **State receipts and subject burden can diverge.** | Historical centralization and tax-intermediary arrangements matter. | Display retained fees, local spending and treasury receipts separately. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914) |
| **Military establishments are a support burden, not free population conversion.** | Roman standing ground forces around **0.6–0.7%** of population provide one useful anchor. | Audit food, wages, equipment and removed civilian labor together. [Cambridge University Press](https://www.cambridge.org/core/books/abs/cambridge-ancient-history/army/962EFC9627C4FEE8D98199246D8476D9) |
| **Literacy changes by cohort and access.** | Modern youth literacy exceeds adult literacy. | A school expansion should improve incoming cohorts before transforming the entire adult population. [UNESCO Institute for Statistics](https://www.uis.unesco.org/en/news/2026-education-data-refresh) |

Do not make every seed reproduce a target percentage. A plausible world may lie outside a historical comparison range for an identifiable reason. The useful alert is **“outside the reference range, with no supporting mechanism”**, not merely “outside the range.”

---

## 9. Modeling recommendation for TCE’s scale

### 9.1 Keep persons individual; organize production and obligations above them

| Simulation level | Recommended responsibilities |
| --- | --- |
| **Person** | Age, health, skills, relationships, labor availability, consumption requirements and institutional memberships |
| **Household** | Food pooling, stores, housing, land access, dependents, work allocation and transfers |
| **Production unit** | Farms, workshops and firms with physical inputs, equipment, inventories and output |
| **Institution** | Property rights, taxes, rents, schools, military service, enforcement and public spending |
| **Spatial system** | Land productivity, water, fuel, transport costs, disease exposure and settlement footprints |

For a first implementation, simplify occupational labels before simplifying household accounting. Mixed work and consumption pooling are more important to these baselines than hundreds of named professions.

### 9.2 Respect the 10k–50k population boundary

Consider an illustrative **50,000-person closed world**:

| Assumption | Consequence |
| --- | --- |
| Regional density **25 people/km²** | Approximately **2,000 km²** of territory |
| Urbanization **10%** | **5,000 urban residents total** |
| One town of 5,000 at **10,000 people/km²** | **0.5 km²** built-up footprint |
| **0.45 annual labor equivalents per resident** | **22,500 labor equivalents** |
| Agriculture uses **75%** of that labor | **16,875 agricultural labor equivalents** |
| **500 full-time troops** | **1% of total population** |
| **2,500 levies serving two months** | Approximately **417 annual labor equivalents**, before seasonal effects |

These are arithmetic examples, not a reconstruction of a particular society.

A million-person imperial capital cannot fit inside a closed 50,000-person world. TCE must choose between a genuinely regional society and an explicitly represented external world.

**Recommended compromise:** keep nearby people individual, but represent external regions with aggregate households or sectoral accounts. External food, migrants, soldiers and taxes must have origins and constraints. Avoid an unlimited off-map market that silently makes impossible local configurations viable.

### 9.3 Validation architecture

Use three classes of tests:

**Hard accounting checks.** Population identities, food balances, inventories, labor time and fiscal accounts must reconcile.

**Conditional historical comparisons.** Compare a world against several compatible reference cases, using the same settlement definition, fiscal scope and demographic measure.

**Stress-response checks.** Test harvest failure, a blocked trade route, recruitment, taxation changes and sanitation investment. Outcomes should propagate through the modeled mechanisms rather than trigger unrelated scripted penalties.

For small populations, attach uncertainty to observed rates. A settlement with few births cannot provide a stable annual infant-mortality estimate. Pool age-specific events over a rolling window while retaining the raw counts. Suggested initial reporting windows of **10–25 years are P settings**, to be adjusted for event volume and how rapidly conditions change.

### 9.4 Existing models worth borrowing from

| Model | Relevant precedent | What not to assume |
| --- | --- | --- |
| **Village Ecodynamics Project** | Household agents, agricultural production, resource procurement, exchange and specialization in reconstructed Pueblo landscapes | It is a geographically and historically bounded model, not an all-era society simulator. [JASSS](https://www.jasss.org/16/4/4.html) |
| **MayaSim — Heckbert, 2013** | Interacting settlement growth, agriculture, environmental change and trade networks | Its exploratory aggregate mechanisms are not a validated model of individual daily life. [JASSS](https://jasss.soc.surrey.ac.uk/16/4/11.html) |
| **FORGE — Zhu and colleagues, 2021** | Ecological and seasonal constraints on hunter-gatherer population density | It does not supply a general institutional or urban-development model. [Nature](https://www.nature.com/articles/s41559-021-01548-3) |

The most useful borrowing is their **separation of mechanisms, environmental inputs and observable outcomes**, rather than copying their resulting population curves.

---

## 10. Reference data to maintain

Store each benchmark as a record with its definition, period, spatial boundary, source vintage, uncertainty and applicability conditions.

| Domain | Recommended source |
| --- | --- |
| Long-run GDP | **Maddison Project Database 2023**, with original country-study references retained. [University of Groningen](https://www.rug.nl/ggdc/historicaldevelopment/maddison/releases/maddison-project-database-2023?lang=en) |
| Historical spatial population | **HYDE 3.2**, used as a reconstructed gridded model—not independent ground truth for every ancient locality. [DOI](https://doi.org/10.5194/ESSD-9-927-2017) |
| Modern population and settlements | **UN World Population Prospects 2024** and **World Urbanization Prospects 2025**; preserve national versus harmonized definitions. [United Nations](https://www.un.org/sustainabledevelopment/blog/2024/07/press-release-wpp2024/) |
| Employment and structural change | **FAO/ILOSTAT** and the **GGDC–UNU-WIDER Economic Transformation Database**. [FAOHome](https://www.fao.org/statistics/highlights-archive/highlights-detail/employment-indicators-2000-2022-%28september-2024-update%29/) |
| Education | **UNESCO Institute for Statistics**, with indicator age group and underlying observation year. [UNESCO Institute for Statistics](https://www.uis.unesco.org/en/news/2026-education-data-refresh) |
| Fiscal capacity | **OECD regional revenue-statistics datasets**, supplemented by society-specific historical fiscal studies. [OECD](https://www.oecd.org/en/publications/revenue-statistics-in-africa-2025_8d3bf3af-en.html) |
| Military establishments | **Correlates of War NMC v7.0** for 1816–2022; earlier periods require individual reconstructions. [Correlates of War](https://correlatesofwar.org/data-sets/national-material-capabilities/) |

The thinnest areas remain **early-farming GDP, ancient literacy rates, comparable premodern agricultural labor shares, and total extraction or military participation where records cover only part of the system**. Keep those benchmarks explicitly provisional.

**For TCE v1, prioritize food and labor accounting, household consumption, age-specific demography, settlement support networks and fiscal–military logistics. Make those mechanisms produce the reference numbers; use history to reject implausible combinations, not to dictate a predetermined civilization path.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556-tce-research/c/6ab929a5-0e10-83ea-955a-d6911a774369)
