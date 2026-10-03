# Historical demography for The Civilization Engine

**TCE should generate population growth from age-specific survival, reproductive exposure, birth spacing, and household decisions—not assign an “era growth rate,” a fixed lifespan, or a lifetime birth quota.** The same farming technology can coexist with very different demographic regimes because disease environments, marriage systems, infant feeding, inequality, migration, and reproductive norms differ. Recent comparative work also finds substantial overlap between the fertility of different subsistence groups, rather than a clean forager–farmer divide. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC10907265/)

The most useful architecture is therefore a **biological demographic core coupled to endogenous institutions and material conditions**. Historical observations provide calibration targets; they should not become scripts that force a settlement through a predetermined demographic transition.

Throughout this report, **high, medium, and low confidence refer to the strength and transferability of evidence**, not statistical confidence intervals. A well-measured twentieth-century population can be a high-confidence observation but a low-confidence analogue for prehistoric people.

---

## 1. Mechanisms: rules the simulation should implement

### 1.1 Mortality is an age-dependent risk, not a scheduled death age

The essential demographic quantities are different:

| Quantity | Definition | Interpretation for TCE |
| --- | --- | --- |
| \(e\_0\) | Life expectancy at birth under a specified mortality schedule | An output of all age-specific death risks |
| \(e\_{15}\) | Expected **remaining** years at age 15 | Expected attained age at death is \(15+e\_{15}\) |
| \({}\_1q\_0\) | Probability of dying before age 1, conditional on live birth | Infant mortality |
| \({}\_5q\_0\) | Probability of dying before age 5, conditional on live birth | Includes infant deaths; do not add it to infant mortality |
| \({}\_{45}q\_{15}\) | Probability of dying between ages 15 and 60, conditional on surviving to 15 | A useful measure of adult mortality |
| \(h(a)\) | Instantaneous mortality hazard at age \(a\) | The quantity used to generate individual deaths |

Life expectancy at birth can be low while substantial numbers of adults survive into later life. Conversely, surviving childhood does **not** guarantee reaching old age. Gurven and Kaplan’s comparative life tables illustrate both points:

| Population or reference | \(e\_0\), years | \(e\_{15}\), remaining years | \(e\_{45}\), remaining years |
| --- | --- | --- | --- |
| Hadza | 34 | 42.5 | 24.2 |
| Aché, forest period | 37 | 38.5 | 21.1 |
| Sweden, 1751–1759 | 34 | 40.0 | 20.0 |

Thus, the Hadza estimate implies an expected attained age of **57.5 for someone already aged 15**, not 34—and not automatically 70. These are particular reconstructed schedules, not universal preindustrial values. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/GurvenKaplan2007pdr.pdf)

**Implementation rule.** For a constant hazard over a time step of length \(\Delta t\),

\[
P(\text{death during step})=1-e^{-h\Delta t}.
\]

If importing a one-year conditional death probability \(q\_a\), the corresponding daily probability is

\[
p\_{\mathrm{day}}=1-(1-q\_a)^{1/365.2425}.
\]

Use finer intervals during infancy. At minimum, distinguish neonatal life, the remainder of infancy, ages 1–4, ages 5–14, and subsequent adult ages. Do not distribute first-year mortality uniformly across childhood.

A common implementation error is treating infant mortality and under-five mortality as independent risks. The correct conditional probability of dying between the first and fifth birthdays is

\[
{}\_4q\_1=\frac{{}\_5q\_0-{}\_1q\_0}{1-{}\_1q\_0}.
\]

### 1.2 Fit childhood and adult mortality separately

**A single life-expectancy slider is insufficient.** Two populations can have similar \(e\_0\) but different infant, childhood, and adult mortality. Coale–Demeny model life tables provide alternative age patterns; newer approaches, including the log-quadratic model discussed in the IUSSP estimation manual, can use child and adult mortality information together. The Coale–Demeny family names—North, South, East, West—are pattern labels, not rules assigning mortality by ethnicity or map location. [Demographic Estimation Tools](https://demographicestimation.iussp.org/content/introduction-model-life-tables)

For TCE, choose one of two approaches:

**Table-based core:** import sex-specific, age-specific survival schedules, then calibrate their response to conditions.

**Smooth core:** use a Siler-type hazard,

\[
h(a)=A e^{-Ba}+C+D e^{Ea},
\]

with a declining early-life component, background component, and increasing late-life component. These are mathematical components, not literal diagnoses.

For a reproducible low-technology test case, the published Hadza fit is

\[
h(a)=0.351e^{-0.895a}+0.011+0.00000670e^{0.125a},
\]

where age is in years and the hazard is per year. Integrating this fitted curve gives approximately **216 infant deaths per 1,000 live births**, **358 deaths before age five per 1,000**, and \({}\_{45}q\_{15}\approx0.447\). These are **calculations from that fitted model**, not additional observed measurements. It is a useful regression test, not a universal “tribal mortality” preset. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/GurvenKaplan2007pdr.pdf)

### 1.3 Explicit disease and background mortality must not double-count deaths

This matters especially because TCE will simulate infections individually.

An empirical all-cause life table already includes deaths from infections, injuries, childbirth, and other causes. Applying that table and then adding a full disease simulation will generally kill too many people.

**Recommended accounting rule:**

\[
h\_{\text{total},i}
=
h\_{\text{residual},i}
+\sum\_c h\_{c,i},
\]

where explicitly simulated causes replace the corresponding portion of the calibration population’s mortality. Estimate the residual by running the explicit disease and injury systems in a reference environment and fitting the remaining hazard to the target life table.

For exceptional events, add **excess exposure or excess mortality relative to baseline**, not another copy of ordinary disease mortality.

Maintain an auditable cause ledger. “Unspecified background mortality” is preferable to inventing a precise disease explanation unsupported by the model.

### 1.4 Fertility should emerge from reproductive states

Use a state machine:

**Not exposed or not fecund → exposed and fecund → pregnant → pregnancy outcome → postpartum state → potentially fecund again.**

The transition rates should depend on age, reproductive history, partner availability, health, infant feeding, intentions, and available fertility-control methods. Bongaarts’s proximate-determinants framework is valuable because it separates biological reproductive capacity from exposure through marriage or partnership, contraception, abortion, and postpartum infecundability. [JSTOR](https://www.jstor.org/stable/1972149?googleloggedin=true)

A practical conception model is

\[
\lambda\_{\text{conception},i}
=
\lambda\_0(a\_i)
\times E\_i
\times F\_i
\times C\_i
\times H\_i,
\]

where the modifiers represent reproductive exposure, current fecundity, fertility-control effects, and health or energetic condition. Their definitions must avoid counting the same mechanism twice.

Two particularly important traps:

**An all-women age-specific fertility rate already includes women who are unpartnered or otherwise not exposed.** Multiplying it by a marriage fraction again understates births.

**An observed birth rate already reflects pregnancy duration and birth spacing.** Using it as a conception probability, then imposing pregnancy and postpartum delays, also understates births. Either simulate the reproductive process and calibrate its output, or use empirical birth hazards directly—not both without recalibration.

### 1.5 “Natural fertility” does not mean maximum fertility

In historical demography, natural fertility usually means the absence of deliberate **parity-dependent limitation**: couples are not systematically stopping because they have reached a chosen number of children. It does not imply continuous exposure, immediate conception after birth, no abstinence, or no culturally regulated spacing. [JSTOR](https://www.jstor.org/stable/1972149?googleloggedin=true)

Breastfeeding and reproductive energetics are especially important. Among !Kung studied by Konner, reported birth spacing was about **44 months**. Among intensively breastfeeding Amele women in Papua New Guinea, a prospective study found median postpartum amenorrhea of **11.3 months**, conception around **19 months postpartum**, and an interbirth interval around **28 months**. Breastfeeding duration alone therefore cannot determine the next birth date. [PubMed](https://pubmed.ncbi.nlm.nih.gov/12278620/)

**TCE rule:** track lactation, the nursing child’s survival, maternal energy balance, and postpartum reproductive recovery separately. A child’s death can both change parents’ intentions and terminate breastfeeding; those are distinct pathways.

For computational economy, very early unrecognized pregnancy losses can be absorbed into effective fecundability. Recognized miscarriages, stillbirths, and live births can remain explicit events.

### 1.6 Household economics and institutions change exposure and intentions

Do not connect prosperity to fertility with one universally signed coefficient.

In one setting, improved resources allow earlier household formation and more surviving births. In another, education, housing costs, delayed partnership, or the cost of raising children reduce births. Historical comparisons of France and England also show that the response of mortality to economic pressure varied over time and place; a single universal “Malthusian elasticity” is not well supported. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/life-under-pressure-france-and-england-16701870/E27A0F72DE0C55E06B7C9ABD15BAB697)

Recommended causal pathways are:

* **Land, housing, and employment access → household formation and reproductive exposure.**
* **Food access and disease → fecundity, pregnancy outcomes, and survival.**
* **Child survival expectations, schooling, care obligations, and reproductive norms → desired timing and family size.**
* **Knowledge, autonomy, institutions, and available methods → ability to implement those intentions.**

These are implementation pathways to calibrate, not a claim that every society weighs them identically.

---

## 2. Quantitative parameters and calibration ranges

### 2.1 Broad demographic environments

The following are **proposed calibration search ranges**, synthesized from the evidence—not observed global means, historical limits, or independently interchangeable inputs. The underlying observations are much stronger for recent populations than for early farming societies. Comparative small-scale data, historical life tables, national reconstructions, and modern UN estimates provide the reference points. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6600907/)

| Calibration environment | \(e\_0\), years | Infant deaths per 1,000 live births | Deaths before age 5 per 1,000 live births | TFR, births per woman | Confidence and interpretation |
| --- | --- | --- | --- | --- | --- |
| Small-scale foraging analogue | 21–37 | 100–300 | 200–450 | 4–7 | Medium for studied populations; low for direct prehistoric transfer |
| Early farming, substantial disease and nutritional stress | 20–35 | 150–300 | 300–500 | 5–8 | Low; deliberately broad working envelope |
| Established preindustrial agrarian society | 25–40 | 150–300 | 250–500 | 4–7 | Medium for selected documented populations; lower elsewhere |
| Industrializing or early mortality-transition society | 35–65 | 50–200 | 70–300 | 3–6 | Medium; combines different positions within a transition |
| Low-mortality, post-transition society | 75–85 | 2–10 | 3–15 | 0.8–2.2 | High for availability of empirical analogues; these bounds are a design envelope, not universal limits |

**Do not independently sample every column.** Select or fit a coherent mortality schedule, calculate its \(e\_0\), and then combine it with a reproductive system. A schedule with a particular infant mortality does not permit an arbitrary adult survival curve.

The farming row is especially provisional. The Neolithic demographic transition literature supports increased fertility in many agricultural transitions, but cemetery evidence does not justify a single globally precise Neolithic TFR or life expectancy. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21798934/)

### 2.2 Reproductive and sex-ratio parameters

| Parameter | Value or working range | Units and denominator | Source and confidence |
| --- | --- | --- | --- |
| Sex ratio at birth | Default **105**; natural reference values roughly **103–107** | Male live births per 100 female live births | Chao et al. 2019; high for broad reference range, not a fixed value for every population. [PubMed](https://pubmed.ncbi.nlm.nih.gov/30988199/) |
| Fertility across 75 natural-fertility populations | Mean **5.98**, SD **1.40** | TFR, births per woman | Gurven and Davison 2019 compilation; medium because populations and observation methods differ. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6600907/) |
| Routine low-technology birth-spacing search range | **24–48** | Months between live births | Proposed calibration envelope, anchored by the field studies below; medium as a starting range, not a universal distribution. [PubMed](https://pubmed.ncbi.nlm.nih.gov/12278620/) |
| Agta first live birth | **20.14** | Mean age in years | Goodman et al. 1985; medium for the studied sample, low for other populations. [PubMed](https://pubmed.ncbi.nlm.nih.gov/3985568/) |
| Agta interbirth interval | **2.85 years**, about **34.2 months** | Conditional on the previous infant surviving until the next birth | Same study; medium. The conditioning matters. [PubMed](https://pubmed.ncbi.nlm.nih.gov/3985568/) |
| Amele postpartum amenorrhea | Median **11.3** | Months after delivery | Worthman et al. 1993; medium-to-high for this prospective study. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-biosocial-science/article/abs/attenuation-of-nursingrelated-ovarian-suppression-and-high-fertility-in-wellnourished-intensively-breastfeeding-amele-women-of-lowland-papua-new-guinea/4A76FD866D333D3328CE279C1F68A30A) |
| Miscarriage risk, ages 25–29 | **9.8%** | Recognized pregnancies in the study | Norwegian registers, 2009–2013; high locally, low for direct ancient transfer. [BMJ](https://www.bmj.com/content/364/bmj.l869%20) |
| Miscarriage risk, ages 45+ | **53.6%** | Same denominator | Same study; use chiefly to constrain the age gradient, not ancient absolute risk. [BMJ](https://www.bmj.com/content/364/bmj.l869%20) |
| Historical English maternal mortality | **1,050 → 750 → 500** | Maternal deaths per 100,000 live births, for 1700–1750, 1750–1800, 1800–1850 | Parish-based estimates discussed by Chamberlain; medium, with substantial historical ascertainment uncertainty. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1633559/) |

The maternal mortality ratio is not exactly the probability of death per pregnancy: its denominator is live births, while maternal deaths can arise from pregnancies ending otherwise. A simplified delivery-only model may approximate it as risk per live birth, but a full reproductive model should preserve the distinction.

For scale, a hypothetical 1% death risk at each of six deliveries gives a cumulative risk of

\[
1-0.99^6 \approx 5.9\%.
\]

This illustrates the significance of repeated exposure without implying that childbirth killed most mothers.

### 2.3 Age-specific fertility schedules

For five-year age groups,

\[
TFR=5\sum\_a ASFR\_a,
\]

when ASFR is expressed as births per woman-year. Divide rates expressed per 1,000 woman-years by 1,000 first.

The following schedules are **synthetic test fixtures**, not historical estimates:

| Maternal age | High fertility | Moderate, later-entry fertility | Delayed, low fertility |
| --- | --- | --- | --- |
| 15–19 | 80 | 20 | 5 |
| 20–24 | 240 | 150 | 25 |
| 25–29 | 270 | 230 | 90 |
| 30–34 | 250 | 220 | 130 |
| 35–39 | 200 | 170 | 85 |
| 40–44 | 130 | 90 | 23 |
| 45–49 | 30 | 20 | 2 |
| **Resulting TFR** | **6.0** | **4.5** | **1.8** |

*ASFR units: live births per 1,000 woman-years.*

These are useful for testing the demographic accounting layer before implementing endogenous reproductive decisions. In the final simulation, they should be **outputs against which the conception–pregnancy–spacing system is checked**, not direct conception probabilities.

Also track completed cohort fertility. Period TFR can fall when people postpone births even if their eventual family size falls less: birth timing and lifetime birth number are not the same phenomenon. [Knowledge Commons](https://knowledgecommons.popcouncil.org/departments_sbsr-pgy/248/?utm_source=chatgpt.com)

---

## 3. Variation across eras and regions

### 3.1 Foragers were not a single low-fertility regime

The contrast between “foragers with few children” and “farmers with many children” is too rigid. Hadza research reports TFR around **6.1**; the Agta study reports completed parity of **6.53**. These are different demographic measures, but both demonstrate that substantial fertility is compatible with foraging. [Cambridge University Press](https://www.cambridge.org/core/books/demography-and-evolutionary-ecology-of-hadza-huntergatherers/57B133BBAED33BC9231410D6A91794BD/listing)

The correct TCE variables are mobility constraints, childcare, workload, food reliability, breastfeeding, reproductive health, and partnership patterns. “Forager” should describe an economic strategy, not directly set a birth multiplier.

### 3.2 Agricultural transitions can increase births without improving survival

Settlement and food production can change energetic conditions, reproductive exposure, mobility, and childcare. At the same time, settlement can alter infection exposure and diet. The Neolithic demographic transition therefore need not be a transition to healthier individual lives; increased fertility can outweigh substantial mortality. The direction and timing vary, and archaeological inference remains method-sensitive. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21798934/)

**TCE implication:** agriculture should modify specific mechanisms. It should not automatically grant a longevity bonus, and the same transition can produce a population increase alongside worsened health indicators.

### 3.3 Regional anchors

| Region and setting | Documented anchor | What it should change in the model |
| --- | --- | --- |
| Southern Africa, !Kung | Reported spacing around **44 months** and lifetime fertility around **4.7 births**. [PubMed](https://pubmed.ncbi.nlm.nih.gov/12278620/) | Long spacing can restrain fertility without modern contraception. |
| East Africa, Hadza | TFR around **6.1** in Blurton Jones’s demographic synthesis. [Cambridge University Press](https://www.cambridge.org/core/books/demography-and-evolutionary-ecology-of-hadza-huntergatherers/57B133BBAED33BC9231410D6A91794BD/listing) | Do not assign uniformly low fertility to foragers. |
| Philippines, Agta | First live birth about **20.1 years**, completed parity **6.53**, conditional spacing about **34 months**. [PubMed](https://pubmed.ncbi.nlm.nih.gov/3985568/) | Model reproductive timing and spacing rather than a subsistence label. |
| Papua New Guinea, Amele | Intensive breastfeeding with amenorrhea about **11 months** and birth spacing around **28 months**. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-biosocial-science/article/abs/attenuation-of-nursingrelated-ovarian-suppression-and-high-fertility-in-wellnourished-intensively-breastfeeding-amele-women-of-lowland-papua-new-guinea/4A76FD866D333D3328CE279C1F68A30A) | Feeding practices and energetic condition interact; breastfeeding is not a binary contraceptive switch. |
| Qing China, selected lineages | A study of five lineages found moderate marital fertility but no clear evidence of parity-dependent early stopping or extended spacing. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/aehr.12269) | Competing interpretations of historical fertility control should remain alternative calibrations. |
| South Asia, Nepal | A 1987 proximate-determinants analysis estimated average birth intervals around **36 months**, with breastfeeding important. [PubMed](https://pubmed.ncbi.nlm.nih.gov/3624296/) | Early or widespread union does not imply uninterrupted childbearing. |
| Japan, 1891–1898 | Official life-table estimates: \(e\_0\) **42.8 years for males**, **44.3 for females**. [Ministry of Health, Labour and Welfare](https://www.mhlw.go.jp/www1/english/database/lifetb99_8/part1.html) | Low-technology or early-industrial populations need not share the same mortality as stressed agrarian populations. |
| Japan, postwar | TFR fell from **4.54 in 1947 to 2.04 in 1957**. [Nippon Zaidan](https://nippon.zaidan.info/seikabutsu/1996/00147/contents/050.htm) | Fertility transitions can be much faster than a centuries-long technology progression. |
| Mexico, Old Colony Mennonites | A 1967 census-based study reported median live births of **9.5 among women older than 45**. [PubMed](https://pubmed.ncbi.nlm.nih.gov/2227913/) | Strong pronatal norms and sustained exposure can produce much larger families than a generic agrarian default. |
| London, eighteenth century | Infant feeding and familial wealth interacted; wealth was not a simple, uniformly protective factor. [Taylor & Francis Online](https://www.tandfonline.com/doi/full/10.1080/1081602X.2019.1580601) | Health effects should pass through actual care, feeding, and exposure pathways. |

These are **anchors, not continent-wide presets**. The evidence is particularly thin for many ancient African, American, Oceanian, and Asian populations. Substituting a European parish schedule for missing evidence is a modeling assumption and should be recorded as such.

### 3.4 Industrialization and the demographic transition

The usual transition involves mortality declining before fertility has fully adjusted, producing rapid natural increase. But industrialization is neither a necessary immediate trigger nor a reliable clock. Bongaarts documents fertility decline in poor, substantially agricultural settings and emphasizes differences in preferences and family-planning provision. In his selected-country comparison, fertility-transition onset averaged about **1995 in sub-Saharan Africa versus 1975 elsewhere**; these are sample-based averages, not dates all countries followed. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/j.1728-4457.2016.00164.x)

For TCE, let sanitation, infection control, maternal care, schooling, childrearing costs, social insurance, and reproductive practices spread at different rates. Mortality and fertility can stall, reverse, or become decoupled.

A modern reference point is the UN’s **2024 estimate of global TFR at 2.25 and life expectancy at 73.3 years**. Those global averages combine populations at very different demographic positions and should not be used as one modern-world parameter set. [United Nations](https://www.un.org/development/desa/pd/node/4354)

### 3.5 Sex ratios and adult causes of death

Start with the sex ratio at birth, then let age-specific survival and migration generate the adult sex ratio. Do not enforce a 50:50 adult population. Sex-biased migration, warfare, maternal mortality, differential care, and other exposures belong in the relevant behavioral and health systems—not in an unexplained culture-wide mortality coefficient.

Adult causes also require age and context. In Gurven and Kaplan’s pooled small-scale sample, reported deaths at ages 15–59 were approximately **61% illness, 9% degenerative causes, 13% accidents, and 17% violence**. Classification and between-population variation were substantial; these are not global prehistoric shares. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/GurvenKaplan2007pdr.pdf)

| Demographic environment | Adult causes the model should be able to represent |
| --- | --- |
| Small-scale societies | Infections, injuries, interpersonal or collective violence, reproductive causes, and chronic disease |
| Dense preindustrial settlements | Endemic and epidemic infections, reproductive causes, injuries, occupational exposures, and deprivation-related vulnerability |
| Industrializing settlements | Continued infectious mortality alongside workplace hazards, pollution, and changing chronic-disease risks |
| Low-mortality populations | Stronger concentration of deaths at older ages, with cardiovascular disease, cancers, and other noncommunicable causes important |

For a quantitative industrial anchor, Cambridge historical demographers report that tuberculosis accounted for about **15% of all deaths in Britain during 1848–1872**. That is an all-age share—not a probability that 15% of adults die each year. For later regimes, WHO’s cause-specific estimates are preferable to carrying forward historical cause shares. [Cambridge University](https://www.cam.ac.uk/stories/covid19-the-long-view?utm_source=chatgpt.com)

---

## 4. Population growth, Malthusian feedback, and validation facts

### 4.1 Growth must be demographic accounting

For every settlement and the world,

\[
\Delta N=B-D+I-E.
\]

Keep natural increase separate from migration. A growing city may have an unfavorable birth–death balance but receive enough migrants to expand.

Under constant fertility and mortality schedules, a useful diagnostic is the net reproduction rate:

\[
NRR=p\_f\int l\_f(a)f(a)\,da,
\]

where \(p\_f\) is the female share of live births, \(l\_f(a)\) is female survival from birth to age \(a\), and \(f(a)\) is age-specific fertility.

An NRR of one means one generation of women replaces itself under those schedules. The intrinsic growth rate satisfies the Euler–Lotka equation:

\[
1=p\_f\int e^{-ra}l\_f(a)f(a)\,da.
\]

These equations are **diagnostics**, not instructions to force observed annual growth to equal \(r\). Transient age structure, migration, shocks, and stochasticity all matter.

### 4.2 Replacement fertility is not universally 2.1

Let \(\bar S\_f\) be female survival averaged across the fertility schedule. Approximately,

\[
TFR\_{\mathrm{replacement}}
\approx
\frac{1}{p\_f\bar S\_f}.
\]

With 105 male births per 100 female births, \(p\_f\approx0.4878\).

| Fertility-weighted female survival | Approximate replacement TFR |
| --- | --- |
| 0.45 | 4.56 |
| 0.60 | 3.42 |
| 0.98 | 2.09 |

These are calculated examples. **Survival to age five is not the correct survival term**: deaths after childhood and during reproductive ages also affect replacement.

A useful synthetic transition test holds TFR at six while increasing fertility-weighted survival from 0.35 to 0.55. NRR rises from about **1.02 to 1.61**. Assuming a 30-year generation interval, the approximation

\[
r\approx\frac{\ln NRR}{30}
\]

changes from about **0.08% to 1.59% annual intrinsic growth**. The population accelerates without any fertility bonus or direct growth modifier.

### 4.3 Slow historical growth does not imply low reproductive capacity

Gurven and Davison discuss estimated global growth of approximately **0.072% annually between 7,000 and 500 years before present**, compared with mean observed twentieth-century growth around **1.1% among foragers** in their compilation. Long-run historical averages and recent local observations are therefore very different quantities. Their explanation involves changing conditions and demographic catastrophes, but the exact contribution of each remains inferential. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6600907/)

An unusually rapid documented natural-increase regime was the Hutterites: Laing reports **41.5 per 1,000 annually in 1950**, or about **4.15%**. This is exceptional, but it demonstrates why “preindustrial people can only grow at 0.1%” is not a biological rule. [Wayne State Digital Commons](https://digitalcommons.wayne.edu/humbiol/vol52/iss2/13/)

For context, constant annual growth of 0.1%, 1%, and 3% implies doubling times of approximately **693, 69, and 23 years**, respectively.

### 4.4 Malthusian pressure should act through households

A useful TCE feedback loop is:

\[
\text{population and land use}
\rightarrow
\text{output, rents, wages, and food prices}
\rightarrow
\text{household resources}
\rightarrow
\text{formation, fertility, survival, and migration}.
\]

Do not turn total population directly into mortality. People should respond to actual scarcity, unequal access, disease exposure, or inability to establish a household.

Trade, storage, agricultural improvement, redistribution, and migration can weaken local constraints. Conversely, a food-producing region can contain hungry households if access is unequal. The model should therefore distinguish total production from household consumption.

### 4.5 Stylized facts a correct simulation should reproduce

| Validation pattern | Expected result |
| --- | --- |
| **Low \(e\_0\) with surviving elders** | Substantial childhood mortality can coexist with adults living into their sixties and beyond; the conditional life table must reproduce the chosen reference. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/GurvenKaplan2007pdr.pdf) |
| **Fertility heterogeneity** | A mean near six must not give every woman six children. Exposure, losses, sterility, spacing, and survival should produce a distribution. |
| **Spacing responds to physiology and behavior** | Long breastfeeding does not imply one fixed interval; the Amele and !Kung anchors should be representable by different mechanisms. [PubMed](https://pubmed.ncbi.nlm.nih.gov/12278620/) |
| **Mortality decline creates a growth surge** | The synthetic survival-change experiment above should increase births relative to deaths without changing TFR. |
| **Delayed births affect period indicators** | Postponement can depress period TFR more than completed cohort fertility. [Knowledge Commons](https://knowledgecommons.popcouncil.org/departments_sbsr-pgy/248/?utm_source=chatgpt.com) |
| **Sex ratios vary by age** | Birth ratios, differential survival, and migration should generate different child, working-age, and elderly ratios. |
| **Crises are correlated** | Epidemics and harvest failures should produce clustered household and settlement outcomes, not merely more independent random deaths. |
| **Age structure affects crude rates** | An older low-hazard population can have a higher crude death rate than a younger high-hazard population. Test age-specific mortality as well as deaths per resident. |

Also expect sampling noise. In a synthetic population of 10,000 with 400 births and 350 deaths annually, independent Poisson variation gives a standard deviation of about **27 people** in annual natural increase. One poor year does not necessarily indicate a faulty demographic model; shared shocks can make variability larger.

---

## 5. Recommended TCE representation

### 5.1 Individual and institutional state

The demographic core needs relatively little persistent state per person:

| Level | Minimum useful state |
| --- | --- |
| Individual | Birth date; reproductive characteristics; health and persistent frailty; parents; household; partnership/exposure; reproductive stage; pregnancy due date; parity; surviving offspring; postpartum and lactation state |
| Household | Members and dependants; food access; labor and care capacity; housing; land or employment access; wealth; reproductive intentions |
| Settlement and institutions | Water and sanitation; care access and quality; markets and storage; housing and land institutions; schooling; social insurance; marriage and inheritance rules; reproductive knowledge and method access |

Keep social gender roles and institutions distinct from the biological variables needed for pregnancy and sex-specific mortality. Institutional restrictions can change without rewriting biology.

### 5.2 Update cadence and event handling

**Recommended design:** use monthly reproductive decisions with dated pregnancy outcomes, daily or event-driven health updates, and integrated mortality hazards. Household formation and migration can be evaluated less frequently or when relevant conditions change.

Age should advance on the same demographic clock as pregnancies, harvests, and institutional change. Compressing visual daily life is compatible with realism; silently making a pregnancy last a different fraction of a lifetime is not.

Do not assign an immutable death date at birth. A scheduled candidate event can be efficient, but its distribution must respond when hazards change.

### 5.3 Calibration order

First calibrate the **mortality-only subsystem** against \(q\_1\), \(q\_5\), adult mortality, \(e\_0\), and conditional adult expectancy.

Next calibrate **reproductive outcomes** against ASFR, TFR, completed parity, childlessness, age at first birth, and birth-interval distributions. Only then couple births and deaths to household resources, disease transmission, and institutions.

Finally test endogenous change: frontier settlement, urban crowding, sanitation adoption, epidemic shocks, improved child survival, delayed household formation, and fertility-control diffusion.

A model that matches total population but misses age structure or birth spacing is not demographically calibrated.

### 5.4 Initialization and uncertainty

Initialize a plausible age distribution, kinship structure, and mixture of pregnant, postpartum, and other reproductive states. Starting every adult newly partnered and immediately fecund creates an artificial baby boom.

Use multiple random seeds and compare distributions, not one trajectory. Preserve both individual heterogeneity and shared shocks. Where evidence is weak, test alternative demographic regimes rather than concealing uncertainty inside one preferred parameter value.

### 5.5 The 50,000-agent constraint

A genuinely growing population cannot remain below a fixed cap indefinitely without something changing.

At constant growth of 1% annually, 10,000 people become 50,000 in approximately **161 years**; at 2%, about **80 years**. This is arithmetic, not a forecast.

TCE therefore needs an explicit architectural choice: finite habitable space with endogenous constraints and migration; an off-map population representation; or a distinction between fully simulated individuals and aggregate populations. A hidden fertility reduction near the agent cap would undermine the demographic model.

### 5.6 Existing models worth studying

| Model | What to borrow | What not to assume |
| --- | --- | --- |
| **SOCSIM / rsocsim** | Individual demographic event histories, kinship construction, marriages, and synthetic population initialization | It generally takes demographic schedules as inputs; it does not itself explain their economic and institutional origins. [MPIDR](https://www.demogr.mpg.de/de/publikationen_datenbanken_6118/publikationen_1904/software/rsocsim_socsim_microsimulation_r_package_7744) |
| **FPsim** | Explicit reproductive states, contraception, pregnancy outcomes, postpartum processes, and maternal/infant outcomes | Contemporary calibrations are not prehistoric biology or universal social behavior. [Nature](https://www.nature.com/articles/s44294-023-00001-z) |
| **Artificial Anasazi / Long House Valley models** | Coupling agricultural productivity, household needs, spatial settlement, and demographic outcomes | Reproducing one archaeological sequence does not establish a universal demographic mechanism. Replication and sensitivity work are important. [Johns Hopkins University](https://pure.johnshopkins.edu/en/publications/population-growth-and-collapse-in-a-multiagent-model-of-the-kayen-7?utm_source=chatgpt.com) |

For TCE, the strongest combination is **SOCSIM-style demographic bookkeeping, FPsim-style reproductive processes, and an explicit household–environment economy**.

---

## 6. Source priorities and limits of the evidence

### Core sources and datasets

| Source | Best use |
| --- | --- |
| **Coale–Demeny model life tables; UN model life tables; IUSSP Tools for Demographic Estimation** | Coherent age patterns when detailed mortality observations are unavailable; comparison of child and adult mortality shapes. [Demographic Estimation Tools](https://demographicestimation.iussp.org/content/introduction-model-life-tables) |
| **Human Mortality Database** | High-quality age- and sex-specific mortality and life tables for covered populations; distinguish period from cohort tables. [Human Mortality Database](https://www.mortality.org/) |
| **Human Fertility Database** | Period and cohort fertility, age schedules, and birth-order patterns for covered populations. [Human Fertility Database](https://www.humanfertility.org/) |
| **UN World Population Prospects** | Broad international demographic comparisons from 1950 onward; retain the release version and distinguish estimates from projections. [United Nations](https://www.un.org/development/desa/pd/node/4354) |
| **UN Inter-agency Group for Child Mortality Estimation** | Neonatal, infant, and child mortality comparisons, including uncertainty and data-quality limitations. [UNICEF](https://www.unicef.org/press-releases/progress-reducing-child-deaths-slows-49-million-children-under-five-die-2024) |
| **WHO Global Health Estimates** | Cause-specific mortality for later demographic environments. [World Health Organization](https://www.who.int/data/gho/data/themes/mortality-and-global-health-estimates) |
| **Field demography and historical reconstruction** | Gurven–Kaplan, Blurton Jones, Goodman et al., Worthman et al., parish reconstruction, and lineage studies provide mechanisms and local calibration anchors unavailable from modern national series. |

### Claims to treat cautiously

**Prehistoric mortality estimates are not direct vital statistics.** Cemetery samples are shaped by burial selection, preservation, age-estimation errors, and population growth. A large proportion of child skeletons does not translate mechanically into a known child mortality probability. This has been a central methodological dispute in paleodemography. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0047248482800237)

**Recent foragers are not unchanged prehistoric populations.** Their demography is observed under particular ecological, political, disease, and contact conditions. Use them as informative analogues, not a universal ancestral baseline. Comparative work on the forager population paradox explicitly shows the difficulty of projecting recent observed growth backward. [eScholarship](https://escholarship.org/uc/item/6401j91k)

**Agriculture did not produce one demographic package.** Fertility increases are better supported than a universal numerical change in life expectancy, and recent comparative research finds considerable overlap between subsistence categories. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21798934/)

**Historical fertility control is contested in some settings.** Qing lineage research illustrates disagreement over whether moderate fertility reflected deliberate limitation or other mechanisms. Missing women, infants, and unmarried people also complicate inference from genealogies. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/aehr.12269)

**There is no single demographic-transition coefficient.** Education, reproductive preferences, institutions, survival, and method access interact differently across settings. Avoid encoding a deterministic fertility decline at a particular technology level or income. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/j.1728-4457.2016.00164.x)

**Bottom line:** make mortality schedules and reproductive processes internally coherent; let institutions, resources, and disease change their causes; and validate against multiple demographic observables. Then population growth, stagnation, aging, recovery, and decline can emerge without being authored as era-specific outcomes.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92820-da70-83ea-9177-d308ef4dc49a)
