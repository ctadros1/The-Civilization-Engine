# Nutrition, food requirements, and famine dynamics

## Simulation-ready research report for The Civilization Engine

**The central recommendation is to model food production, food access, and nutritional condition separately.** A settlement can have adequate aggregate food while particular households starve; conversely, a poor harvest need not become a famine when reserves, trade, income support, and disease control remain effective. Historical famine research therefore supports a coupled economic, physiological, and epidemiological model—not a settlement-wide “hunger” variable. [OUP Academic](https://academic.oup.com/book/32827)

For TCE, nutritional requirements should depend primarily on **body size, age, activity, growth, pregnancy, and lactation**, rather than technological era. Historical change should alter what foods exist, how people obtain them, their workloads, storage and transport, and exposure to disease.

Throughout this report:

* **H — High confidence:** established physiological reference or well-defined measurement. Not an exact requirement for every individual.
* **M — Moderate confidence:** measured association or case study whose transfer to other populations requires calibration.
* **P — Proposed parameter:** an implementation or sensitivity-testing choice, **not an empirical historical estimate**.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Maintain physical food accounts before calculating hunger

Represent each food stock by its **owner, location, quantity, processing state, age, and condition**. Production and exchange must reconcile:

\[
S\_{t+1}=S\_t+H+I+T\_{\mathrm{in}}
-C-\mathrm{Seed}-\mathrm{Feed}-X-T\_{\mathrm{out}}-L
\]

Here, \(H\) is harvested food entering this accounting stage, \(I/X\) imports/exports, \(T\) transfers, \(C\) consumption, and \(L\) actual losses.

**Taxes, rents, requisitions, gifts, and sales transfer food; they do not destroy it.** Seed and animal feed are competing uses, not spoilage. FAO food-balance accounting explicitly distinguishes these flows. [FAOHome](https://www.fao.org/4/X9892E/X9892e02.htm)

Implementation consequences:

* Harvested paddy, milled rice, cooked rice, and spoiled rice must be different states.
* Milling creates edible grain and by-products; cooking usually adds water rather than calories.
* Grain held by a landlord, army, or merchant cannot automatically feed nearby households.
* Consumption of seed or breeding livestock supplies food now but damages future production.

The crucial diagnostic is not just **“food in settlement”**, but **“food each household can obtain before its next dependable acquisition.”**

### 1.2 Compute both healthy requirements and actual expenditure

For adults, a useful starting structure is:

\[
E\_{\mathrm{healthy}}
=BMR(M,\mathrm{age},\mathrm{physiology})\times PAL
+E\_{\mathrm{growth}}
+E\_{\mathrm{pregnancy}}
+E\_{\mathrm{lactation}}
\]

Actual expenditure can fall when people stop working, lose body mass, or undergo metabolic adaptation. That does **not** mean they have become adequately nourished. Keep a distinction between expenditure under their current debilitated condition and the intake needed for healthy functioning and recovery. Dynamic body-weight models explicitly distinguish tissue change and expenditure adaptation. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21872751/)

For TCE, derive activity from the daily task budget. A person harvesting, carrying water, nursing an infant, and walking to market should not receive the same requirement as an otherwise similar sedentary person. **Do not add “exercise calories” again when the PAL already includes that work.**

### 1.3 Allocate food through households, rights, and purchasing power

A household obtains food through own production, wages, trade, transfers, credit, common-resource access, and institutional entitlements. Model these separately.

A useful purchasing-power measure is:

\[
\text{staple purchasing power}
=\frac{\text{disposable income}}{\text{local staple price}}
\]

For pastoralists, the equivalent may be kilograms of grain obtainable per animal sold. For laborers, it is grain obtainable per day of work.

Sen’s entitlement framework explains why employment loss or worsening exchange terms can cause starvation without a proportionate decline in total food availability. It does **not** establish that production shortfalls are unimportant in every famine. [academic.oup.com](https://academic.oup.com/book/32827?utm_source=chatgpt.com)

Within households, allocation should depend on norms and circumstances: equal sharing, protection of children, prioritization of essential workers, gender hierarchy, or unequal bargaining power. Do not assume a universal allocation rule. WFP’s coping measurement explicitly includes adults restricting their own consumption so children can eat. [VAM Resource Centre](https://vamresources.manuals.wfp.org/docs/reduced-coping-strategies-index)

### 1.4 Let storage and trade create nonlinear price responses

A modest harvest loss can initially be absorbed by inventories. Once available inventories become scarce, prices can rise sharply because consumers cannot easily eliminate their need for food. Competitive-storage models reproduce this combination of ordinary fluctuations and occasional large positive price spikes. [Professor Sir Angus Deaton](https://deaton.scholar.princeton.edu/publications/behaviour-commodity-prices)

For an agent market, implement:

**Harvest expectations → desired inventories → offers and bids → physical shipments → revised expectations.**

Transport capacity and delay matter. Grain available two valleys away is not immediately available in a besieged town. Merchants should compare expected destination prices with purchase, transport, storage, and loss costs.

Avoid assigning a fixed multiplier such as “20% harvest failure causes 50% inflation.” The outcome should depend on reserves, substitutes, neighboring harvests, trade access, and household purchasing power.

### 1.5 Make coping choices preserve—or undermine—future livelihoods

Food shortages usually provoke adaptation before catastrophic mortality. Households can substitute cheaper foods, borrow, seek assistance, increase gathering, reduce portions, sell assets, or move. Household Economy Analysis distinguishes meeting immediate survival needs from preserving the assets and expenditures needed to sustain livelihoods. [FEWS NET Help Center](https://help.fews.net/fdp/livelihoods-terms?utm_source=chatgpt.com)

Use choices with explicit costs rather than a fixed famine-stage script:

| Choice | Immediate effect | Possible later consequence |
| --- | --- | --- |
| Substitute less-preferred foods | Maintains calories more cheaply | Lower diet quality or additional processing costs |
| Borrow food or money | Preserves consumption | Debt, dependency, loss of collateral |
| Sell livestock or tools | Raises purchasing power | Lower traction, milk, manure, or future earnings |
| Consume seed | Supplies calories | Less planting next season |
| Increase gathering or work | Additional food or income | Exhaustion, resource depletion, reduced care time |
| Migrate | Access to work, relatives, markets, or relief | Travel costs, separation, congestion, disease exposure |

These are **modeling relationships**, not universal sequences. A household with relatives elsewhere may migrate early; another may remain because it cannot finance travel. Assess destination opportunities, travel costs, dependents, information, and social connections individually.

### 1.6 Couple depletion, infection, work, and reproduction

Separate short-term hunger from deteriorating nutritional condition.

**Short-term:** missed meals influence motivation, fatigue, and food-seeking.

**Intermediate:** sustained deficits reduce body reserves and potentially lean tissue; illness can further reduce intake and absorption.

**Longer-term:** childhood growth, adult work capacity, reproductive function, and subsequent livelihood prospects can change.

Undernutrition is strongly associated with elevated child mortality, while historical famine mortality also involves infectious disease and disrupted living conditions. Hunger must therefore modify susceptibility or disease severity without magically creating pathogens. [journals.plos.org](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0064636)

A suitable mortality structure is:

\[
p\_i(\mathrm{death},\Delta t)
=1-\exp[-(h\_{\mathrm{baseline},i}
+h\_{\mathrm{infection},i}
+h\_{\mathrm{depletion},i})\Delta t]
\]

Use nutritional condition within the relevant hazards. **Do not additionally multiply total mortality by every available malnutrition indicator**, which would double-count overlapping risks.

Institutional behavior belongs inside this causal system. For example, research on China’s 1959–1961 famine identifies inflexible procurement as a mechanism through which productive regions could suffer high mortality. A model in which local output alone determines local consumption cannot reproduce that pattern. [Columbia Business School](https://business.columbia.edu/faculty/research/institutional-causes-chinas-great-famine-1959-1961)

---

## 2. Quantitative parameters

### 2.1 Adult energy requirements

FAO/WHO/UNU’s *Human Energy Requirements* provides the following approximate adult basal metabolic equations. \(M\) is body mass in kilograms; output is **kcal/person/day**. These are population-reference equations, not measurements of a particular individual. [fao.org](https://www.fao.org/4/y5686e/y5686e07.htm)

| Age | Male-reference BMR | Female-reference BMR | Confidence |
| --- | --- | --- | --- |
| 18–30 years | \(15.057M+692.2\) | \(14.818M+486.6\) | H reference; individual prediction uncertain |
| 30–60 years | \(11.472M+873.1\) | \(8.126M+845.6\) | H reference |
| 60+ years | \(11.711M+587.7\) | \(9.082M+658.5\) | H reference |

Multiply BMR by an activity level:

| Activity pattern | PAL, dimensionless | Example: 55 kg woman, age 30–60 | Example: 70 kg man, age 30–60 |
| --- | --- | --- | --- |
| Sedentary/light | 1.40–1.69 | About 1,810–2,180 kcal/day | About 2,350–2,830 kcal/day |
| Active/moderate | 1.70–1.99 | About 2,200–2,570 | About 2,850–3,340 |
| Vigorous/heavy | 2.00–2.40 | About 2,590–3,100 | About 3,350–4,020 |

PAL bands are FAO references; example intakes are calculated from the preceding equations. They describe whole-day activity, not the intensity of a single task. [fao.org](https://www.fao.org/4/y5686e/y5686e07.htm)

**TCE recommendation:** preserve age and body-size effects, but assign work independently of sex. Seasonal agricultural workloads should change PAL without requiring an “era multiplier.”

### 2.2 Infants, children, and adolescents

FAO infant reference requirements decline per kilogram as growth slows: approximately **110 kcal/kg/day at one month, 82 at six months, and 80 at twelve months** in its combined-feeding estimates. Feeding mode and individual growth matter; these are not a single constant for infancy. [FAOHome](https://www.fao.org/4/y5686e/y5686e05.htm)

The following childhood values are rounded reference requirements, including growth:

| Age band | Boys, kcal/day | Girls, kcal/day | Application |
| --- | --- | --- | --- |
| 1–2 years | 950 | 850 | Reference-size child |
| 3–4 years | 1,250 | 1,150 | Reference-size child |
| 6–7 years | 1,575 | 1,425 | Central activity estimate |
| 10–11 years | 2,150 | 2,000 | Central activity estimate |
| 14–15 years | 3,000 | 2,450 | Reference adolescent size/activity |
| 16–17 years | 3,325 | 2,500 | Reference adolescent size/activity |

**Confidence: H for the reference framework; M when transferred directly to historical populations.** For children aged six and older, FAO supplies approximately ±15% adjustments for lighter or heavier activity. Historical adolescents should not automatically receive these absolute values regardless of body size. [FAOHome](https://www.fao.org/4/y5686e/y5686e06.htm)

**Implementation:** interpolate age/body-size reference tables. Do not add a second growth allowance to a table value that already includes growth.

### 2.3 Protein requirements

The EFSA protein reference uses an adult average requirement of **0.66 g/kg/day** and a population reference intake of **0.83 g/kg/day**. The latter is intended to cover nearly all healthy individuals; it is not a threshold below which everyone immediately becomes protein deficient. [EFSA](https://efsa.onlinelibrary.wiley.com/doi/10.2903/j.efsa.2012.2557)

| Age | Protein reference intake, g/kg body mass/day |
| --- | --- |
| 6 months | 1.31 |
| 1 year | 1.14 |
| 2 years | 0.97 |
| 3 years | 0.90 |
| 4–10 years | Approximately 0.85–0.92 |
| 11–17 years | Approximately 0.83–0.91, age- and sex-dependent |
| Adults, including older adults | 0.83 |

**Confidence: H as healthy-population reference values.** These imply approximately **46 g/day for a 55 kg adult** and **58 g/day for a 70 kg adult**. Illness and recovery are separate conditions. [EFSA](https://efsa.onlinelibrary.wiley.com/doi/pdf/10.2903/j.efsa.2012.2557)

Count protein already present in cereals: it is incorrect to treat grain as “calories only.” But grams alone are insufficient. Amino-acid composition and digestibility affect usable protein; cereal and legume proteins can complement one another. Cooking and processing also matter. [Cambridge University Press](https://www.cambridge.org/core/journals/british-journal-of-nutrition/article/digestible-indispensable-amino-acid-score-and-digestible-amino-acids-in-eight-cereal-grains/8A1D552E0A471CFA42FDBBE3B8653AC4?utm_source=chatgpt.com)

For a compact model, calculate protein adequacy from the **mixed diet**, rather than summing independently capped “protein quality scores” for each food.

### 2.4 Pregnancy and lactation

| Condition | Additional energy | Additional protein | Source/confidence |
| --- | --- | --- | --- |
| Pregnancy, trimester 1 | +85 kcal/day | +1 g/day | FAO energy; EFSA protein; H reference |
| Pregnancy, trimester 2 | +285 kcal/day | +9 g/day | Same |
| Pregnancy, trimester 3 | +475 kcal/day | +28 g/day | Same |
| Lactation, first six months | About 675 kcal/day production cost | +19 g/day | H reference |
| Lactation after six months | Depends on milk production | +13 g/day | H reference; actual demand variable |

Pregnancy allowances assume the reference pattern of gestational tissue gain. [FAOHome](https://www.fao.org/4/y5686e/y5686e0a.htm)

For adequately nourished mothers, FAO’s first-six-month **additional dietary intake** is about **505 kcal/day**, with approximately 170 kcal/day coming from maternal reserves. Undernourished mothers should not be assumed to have those reserves available; their additional dietary requirement is correspondingly higher. [FAOHome](https://www.fao.org/4/y5686e/y5686e0b.htm)

**Accounting rule:** breast milk transfers energy to the infant. It is not free food generated independently of maternal metabolism.

### 2.5 Converting foods into nutritional quantities

The values below are approximate FAO food-balance coefficients, converted to **per kilogram of the listed product, before cooking**. They use a retail/as-purchased basis; do not apply an additional refuse deduction without first checking the product definition. [FAOHome](https://www.fao.org/4/x9892e/X9892e05.htm)

| Product | kcal/kg | Protein, g/kg |
| --- | --- | --- |
| Wheat grain | 3,340 | 122 |
| Milled rice | 3,600 | 67 |
| Maize grain | 3,560 | 95 |
| Dry lentils | 3,460 | 242 |
| Cassava flour | 3,380 | 15 |
| Shelled groundnuts | 5,670 | 257 |
| Boneless beef, reference product | 1,500 | 185 |
| Whole cow milk | 610 | 33 |
| Common vegetable oils | 8,840 | 0 |

**Confidence: H for approximate food accounting; M for a particular historical variety or preparation.** In the engine, make edible fraction, processing yield, and nutrient retention explicit. Water uptake during cooking changes kilograms per serving, not the original quantity of food energy.

**Worked grain requirement.** For an illustrative adult needing 2,500 kcal/day, with 70% supplied by cereal averaging 3,500 kcal/kg:

\[
\frac{2,500\times0.70}{3,500}=0.50\ \mathrm{kg/day}
\]

That is **182.5 kg/year consumed**. With an illustrative 15% loss between the starting stock and consumption:

\[
182.5/(1-0.15)=214.7\ \mathrm{kg/year}
\]

This is a **cleaned-grain-equivalent calculation**, not a standing-crop yield requirement. Seed, feed, milling losses, and transfers must still be accounted for at their respective stages.

### 2.6 Storage and postharvest losses

Historical loss percentages are much less secure than physiological requirements. Losses differ by crop, stage, season, weather, pests, packaging, and measurement method.

| Parameter or observation | Value | Interpretation | Confidence |
| --- | --- | --- | --- |
| Malawi farm-level study: average losses among households reporting losses | Approximately 5–12% of harvest | Across studied crops and postharvest stages; **not annual storage loss** | M |
| Households reporting any loss in that study | Less than half | Do not apply the conditional mean to every household | M |
| Fresh cassava physiological deterioration | Can begin within 24–72 hours after harvest | Quality deterioration, not instantaneous disappearance of all calories | M |
| Suggested dry-grain storage sensitivity cases | 2%, 5%, and 15% loss over one year | **Authored test cases**, not established pre-industrial averages | P |

The Malawi study covered maize, soybeans, and groundnuts and found substantial variation in which stages generated losses. It provides a useful warning against universal claims such as “traditional farmers lose 30% of all grain.” [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/1467-8489.12237) Fresh cassava’s short postharvest window makes processing and harvest scheduling especially consequential. [PubMed](https://pubmed.ncbi.nlm.nih.gov/28315989/)

For gradual loss:

\[
S(t+\Delta t)=S(t)e^{-\lambda\Delta t},
\qquad
\lambda=-\ln(1-L\_{\mathrm{annual}})/365
\]

Add separate events for infestation, flooding, fire, or improperly dried crops. Also separate **edible quantity from safety**: fungal contamination can make food hazardous without a comparable reduction in stock weight. [FAOHome](https://www.fao.org/4/x5036e/x5036E04.htm)

### 2.7 Price and mortality calibration

For an isolated, simplified market with constant-elasticity demand:

\[
P\_1/P\_0=(Q\_1/Q\_0)^{-1/\epsilon}
\]

Using **proposed sensitivity values** \(\epsilon=0.2\)–0.5, a 20% decline in available quantity produces a **calculated price increase of roughly 1.56–3.05 times**. These are algebraic stress tests, not measured universal famine responses. Do not impose this equation on top of an independently clearing agent market.

A related empirical estimate must not be confused with price elasticity: Subramanian and Deaton estimated **calorie–total-expenditure elasticities of approximately 0.3–0.5** in rural Maharashtra. Higher spending therefore did not translate one-for-one into higher calorie consumption in that setting. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/262020)

For child mortality, useful calibration evidence comes from Olofin and colleagues’ pooled analysis:

| Nutritional condition, children under five | Associated all-cause mortality hazard ratio | Confidence |
| --- | --- | --- |
| Moderate underweight: weight-for-age z-score −3 to below −2 | 2.63 | M |
| Severe underweight: weight-for-age z-score below −3 | 9.40 | M |
| Severe wasting: weight-for-height z-score below −3 | 11.63; 95% CI 9.84–13.76 | M |
| Severe stunting: height-for-age z-score below −3 | 5.48; 95% CI 4.62–6.50 | M |

The comparison category was z-score at least −1. These are observational associations from ten prospective studies, not independent causal multipliers or daily death probabilities. **Do not multiply wasting, stunting, and underweight hazard ratios together.** [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0064636)

There is no comparably defensible universal rule of the form “an adult eating X kcal/day dies after Y days.” Starting reserves, disease, fluid balance, activity, and subsequent intake make such a rule unsuitable.

---

## 3. Variation across eras and world regions

### 3.1 Use historical eras as bundles of capabilities, not nutrition presets

| Setting | Features worth representing | Avoid assuming |
| --- | --- | --- |
| Foraging | Seasonal wild plants, game, fish, honey, mobility, sharing, variable storage | A universally meat-heavy diet or uniformly enormous energy expenditure |
| Early farming | Domesticated staples, harvest concentration, seed retention, processing, mixed wild/domestic foods | Farming immediately eliminates seasonal shortages or always worsens nutrition |
| Pre-industrial agrarian societies | Staple dependence, rents and taxes, local markets, grain reserves, household food processing | Every household consumes the same basket or owns its harvest |
| Industrializing societies | Greater wage dependence, transport integration, milling and processing, changing work | More marketed food automatically means nutritionally complete diets |
| Modern systems | Highly connected supply chains, preservation, fortification, social protection, persistent inequality | National food availability equals individual consumption |

These are modeling archetypes, not a universal sequence. For example, Hadza energy-expenditure research found that greater physical activity did not translate into the simple excess over Western expenditure often assumed after accounting for body size and other factors. Contemporary foragers should nevertheless not be treated as unchanged prehistoric populations. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0040503)

Processing can improve safety and usability while also changing nutrient content. Milling and preparation therefore deserve actual recipes and nutrient-retention rules, rather than a generic “technology increases nutrition” bonus. [FAOHome](https://www.fao.org/4/W0073E/w0073e06.htm)

### 3.2 Regional food systems require different staple combinations

The following are food-system families to support, not homogeneous continental diets:

| Region or ecological setting | Important combinations to represent |
| --- | --- |
| Southwest Asia, Mediterranean, parts of Europe | Wheat/barley and other grains, pulses, oil-bearing crops, dairy and livestock |
| Northern and eastern Europe | Rye, oats, barley or wheat; pulses, animal foods; roots where historically available |
| South Asia | Rice or wheat, millet and sorghum in relevant ecologies, pulses, oils, variable dairy |
| East Asia | Rice in some systems; wheat and millet in others; legumes, vegetables, fish and variable animal foods |
| African drylands and highlands | Sorghum, millets, teff or other cereals; pulses; pastoral and agropastoral combinations |
| Humid tropical Africa | Root/tuber or plantain systems alongside grains, legumes, oils, fish and other animal foods |
| Mesoamerica and the Andes | Maize–bean systems; highland tubers and other crops; locally important animal foods |
| Amazonian and other tropical horticultural systems | Manioc, plantains and other cultivated foods combined with fish, hunting, and gathering |

FAO regional food tables document the compositional differences among these staples; the variation is substantial enough that “one unit of food” is an inadequate nutritional representation. The table does **not** imply that every listed crop was present in every region from the start of agriculture—crop availability must follow TCE’s own diffusion history. [FAOHome](https://www.fao.org/4/X6879E/X6879E04.htm)

Oceania, Arctic communities, and mobile pastoralists also require non-cereal-centered possibilities. The implementation principle is to derive available diets from the local crop, animal, marine, processing, and exchange systems—not from a global cereal quota.

### 3.3 Cereal share: distinguish measured observations from initialization choices

A single “pre-industrial cereal share” is not defensible across these systems. Useful **scenario-initialization priors**, to be replaced by locally grounded evidence where available, are:

| Authored household scenario | Cereal share of dietary energy | Status |
| --- | --- | --- |
| Strongly grain-dependent agrarian household | 60–80% | P: scenario range, not a universal historical estimate |
| Root/tuber/plantain-centered horticultural household | 0–30% | P |
| Pastoral household with variable grain exchange | 0–50% | P |

Two measured anchors show why the range must remain wide.

In the Bolivian Tsimane study, cultivated foods supplied approximately **62% of calories**, but plantains dominated that cultivated component; fish and game also contributed substantially. Men averaged about **2,736 kcal/day**, women **2,422 kcal/day**. Despite the many foods recorded, **nine items supplied 75% of calories**. This was a specific subsistence population undergoing change, not a universal ancestral diet. [DOI](https://doi.org/10.1093%2Fajcn%2Fnqy250)

In FAO’s **2023 global food-balance estimates**, cereals supplied about **42% of dietary energy**. Total dietary energy supply exceeded **3,000 kcal/person/day**, but this measures national food availability, not what each person actually ate. Neither figure is an appropriate universal household target. [FAOHome](https://www.fao.org/statistics/highlights-archive/highlights-detail/food-balance-sheets-2010-2023/)

---

## 4. Chronic undernutrition and stylized facts to reproduce

### 4.1 Work capacity is not simply proportional to yesterday’s calories

Connect work capacity to lean tissue, illness, acute fatigue, and specific deficiencies. A person with depleted reserves may preserve resting metabolism partly by reducing voluntary activity, while still being unable to sustain planting or harvesting.

Micronutrient status can matter even when calories are available. In a twelve-week trial among iron-deficient female cotton-mill workers in Beijing, iron supplementation increased mean hemoglobin from approximately **114 to 127 g/L** and reduced the physiological cost of work. That is evidence for a separate anemia pathway, not a justification for converting the percentage change in heart rate into an equal production bonus. [American Journal of Clinical Nutrition](https://ajcn.nutrition.org/article/S0002-9165%2823%2919537-2/fulltext)

**Implementation:** allow nutritionally impaired agents to choose shorter or less demanding work, and let productive output depend on task-specific capacity. Do not force all deficits into an immediate linear movement-speed penalty.

### 4.2 Childhood growth needs its own state

Track **wasting**—low weight relative to height—separately from **stunting**—low height for age. WHO provides age- and sex-specific standards and growth-velocity references suitable for generating diagnostic measures. [World Health Organization](https://www.who.int/tools/child-growth-standards?utm_source=chatgpt.com)

Longitudinal evidence from cohorts in Brazil, Guatemala, India, the Philippines, and South Africa associates early undernutrition with later stature and human-capital outcomes. It does not imply that every affected individual receives the same permanent impairment. [PubMed](https://pubmed.ncbi.nlm.nih.gov/18206223/)

For TCE, let growth depend on accumulated nutrition and illness over time, with individual variation and possible recovery. Do not shrink adult height when food runs short, or turn one hungry month into a fixed lifelong intelligence penalty.

### 4.3 Reproduction responds with lags and individual variation

Nutritional stress can reduce reproductive function, but birth counts also depend on separation, sexual exposure, pregnancy loss, and maternal survival.

Research on the Dutch famine found a roughly **two-month lag** before food deprivation affected conceptions that subsequently produced live births; recovery could begin rapidly after relief. That is an episode-specific calibration target, not a universal countdown. [Wayne State Digital Commons](https://digitalcommons.wayne.edu/humbiol/vol47/iss1/11/)

Experimental work on exercise and energy availability found **no universal energy-availability threshold** separating normal from disrupted menstrual function. Avoid rules such as “below BMI 18.5, fertility becomes zero” or transferring one athletic-study threshold to every historical woman. [PubMed](https://pubmed.ncbi.nlm.nih.gov/29023359/)

Use a smooth, heterogeneous conception modifier based on nutritional trajectory, illness, and reproductive state. Birth effects then follow naturally through gestation rather than appearing immediately when stocks fall.

### 4.4 Validation targets

| Pattern a credible simulation should reproduce | Evidence or test |
| --- | --- |
| **Highly unequal outcomes under the same harvest shock** | Compare households with different land, wages, debts, access rights, and dependents; entitlement analysis predicts distributional differences. [academic.oup.com](https://academic.oup.com/book/32827?utm_source=chatgpt.com) |
| **Price spikes concentrated around tight inventories** | Storage models generate asymmetric, occasionally large positive price movements rather than symmetric noise. [Professor Sir Angus Deaton](https://deaton.scholar.princeton.edu/publications/behaviour-commodity-prices) |
| **Child mortality rises steeply with severe wasting** | Pooled evidence gives roughly 12 times the reference mortality hazard, subject to context and confounding. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0064636) |
| **Famine mortality is not synonymous with direct starvation** | Historical Irish evidence requires disease and disrupted living conditions in the explanation. [Cambridge University Press](https://www.cambridge.org/core/journals/european-review-of-economic-history/article/abs/what-do-people-die-of-during-famines-the-great-irish-famine-in-comparative-perspective/F83691CC58D26C3D70C1A94E71FB38C4) |
| **Women need not have higher mortality simply because they have less power** | Analysis of seven extreme-mortality populations found female survival advantages, including at very young ages; social allocation can still alter outcomes. [DOI](https://doi.org/10.1073/pnas.1701535115) |
| **A severe famine can remove a substantial fraction of the population over months** | Somalia’s October 2010–April 2012 crisis was estimated to cause about 258,000 excess deaths, including 133,000 children under five: approximately 4.6% of the affected southern/central population and 10% of its under-fives. [FSNAU](https://fsnau.org/in-focus/press-release-study-suggests-258000-somalis-died-due-severe-food-insecurity-and-famine-half) |
| **Birth troughs lag the nutritional shock** | Reproductive suppression affects conceptions first; recorded live births respond later. [Wayne State Digital Commons](https://digitalcommons.wayne.edu/humbiol/vol47/iss1/11/) |
| **Recovery of supply need not instantly restore health or livelihoods** | This should emerge from depleted bodies, ongoing infections, debt, asset loss, and reduced planting capacity—not an arbitrary post-famine timer. |

The Somalia figures are reconstructed excess-mortality estimates, not complete death registration. Use their magnitude and age pattern as an episode-level benchmark, not as a universal famine death rate.

---

## 5. Recommended TCE representation

### 5.1 Minimum state worth retaining

The following is a proposed architecture, not a claim about measured performance.

| Entity | Minimum useful state |
| --- | --- |
| **Food definition** | Energy, protein/amino-acid profile, fat, selected micronutrients, edible fraction, processing recipes, spoilage behavior |
| **Food lot** | Owner, location, product/state, quantity, age, storage conditions, contamination/safety, seed viability where relevant |
| **Person** | Age, size, fat reserve, lean-mass proxy, recent intake, growth state, pregnancy/lactation, illness, deficiency states |
| **Household** | Members, allocation norms, accessible stocks, income, assets, debt, expected acquisitions, social claims |
| **Market/settlement** | Offers, prices, wages, inventories, routes, transport capacity, arrivals, information |
| **Institution** | Tax/procurement rules, granaries, release policies, ration eligibility, relief capacity, enforcement |

For 10k–50k people, aggregate food into **owner–location–product–condition cohorts**, rather than individual objects for every sack or meal ingredient. Preserve enough lot identity for differential spoilage and ownership.

### 5.2 A practical update schedule

**Daily:** spoilage, shipments, household acquisition and meal allocation, intake, actual expenditure, reserve change, illness progression, work-capacity adjustment.

**Weekly or event-driven:** household coping plans, asset sales, borrowing, migration decisions, merchant inventory expectations, institutional releases.

**Monthly or biologically appropriate intervals:** growth updates, longer-term deficiency trajectories, fertility modifiers.

Crop development and harvest remain linked to the agricultural subsystem’s actual seasons. A failed harvest should be an input to household expectations immediately, even before physical stocks run out.

Track recent intake at more than one timescale. **Seven-day and thirty-day summaries are reasonable proposed diagnostics**, but they are smoothing choices, not established biological cutoffs.

### 5.3 What to simplify—and what not to simplify

**Simplify biochemical detail before simplifying access.** Initially, energy, protein quality, a lean/fat distinction, and a few consequential micronutrient states will yield more realistic outcomes than dozens of nutrients combined with communal food pooling.

A sensible first nutrient set is energy, protein quality, fat, iron, vitamin A, thiamine, and vitamin C. These are a proposed engineering selection; expand according to foods and diseases actually represented.

Do not:

* Count protein calories twice.
* Apply “digestibility loss” again to an energy coefficient already representing metabolizable food energy.
* Treat every kilogram of lost body weight as the same energy deficit.
* Use low adult BMI thresholds for children.
* Make illness mortality, wasting mortality, and underweight mortality independent stackable penalties.
* Make a national or settlement average guarantee individual adequacy.

Hall and colleagues’ dynamic body-weight model offers a starting point for tissue and expenditure accounting, but it was not validated as a universal terminal-starvation simulator. Borrow its structure cautiously rather than extrapolating it directly to death. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21872751/)

### 5.4 Institutions should act on actual constraints

Represent several distinct interventions: food redistribution, cash or credit support, transport protection, tax suspension, seed provision, clean water, shelter, and medical care.

Their effects should arise from the modeled constraint. Cash can improve access when food can be bought or imported; it cannot create grain inside a completely isolated settlement. Seed aid can restore next season’s production without resolving today’s hunger. Granary release can protect consumers while exposing corruption, access restrictions, or depletion of future reserves.

Use **survival protection** and **livelihood protection** as different objectives. This distinction is central to Household Economy Analysis and maps well onto TCE’s interannual feedbacks. [FEWS NET Help Center](https://help.fews.net/fdp/livelihoods-terms?utm_source=chatgpt.com)

### 5.5 Existing models and games worth borrowing from

| Model/framework | Useful component | Limitation for TCE |
| --- | --- | --- |
| **Household Economy Analysis; FEWS NET livelihood baselines** | Food and income sources by wealth group, expenditures, seasonality, coping capacity | Not an individual physiology simulator. [FEWS NET](https://fews.net/topics/livelihoods?utm_source=chatgpt.com) |
| **Deaton–Laroque competitive storage model** | Inventory-dependent commodity prices and stockout dynamics | Does not itself model household malnutrition. [Professor Sir Angus Deaton](https://deaton.scholar.princeton.edu/publications/behaviour-commodity-prices) |
| **Hall et al. dynamic energy-balance model** | Body composition and metabolic adjustment | Not a historical famine mortality model. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21872751/) |
| **Village Ecodynamics Project and extensions** | Household farming, resource constraints, exchange, and specialization in archaeological settings | Simplifications such as separating maize calories from hunted protein should not be copied as nutritional truth. [JASSS](https://www.jasss.org/16/4/4.html) |
| **Vintage Story** | Visible food preservation, environmental storage conditions, processing, food categories | Game nutrition and health effects are abstractions, not validated physiological relationships. [Vintage Story Wiki](https://wiki.vintagestory.at/Food_preservation) |

**Recommended development order:** physical food accounting → household access → body reserves and work → infection interaction → reproductive and childhood consequences → richer micronutrients and institutions.

---

## 6. Sources, datasets, and uncertainty

### 6.1 Most useful datasets for implementation

| Source | Use in TCE | Main caution |
| --- | --- | --- |
| **FAO/WHO/UNU, *Human Energy Requirements* (2004); EFSA protein reference (2012)** | Age, activity, pregnancy, lactation, protein lookups | Healthy-population references are not exact individual requirements. [FAOHome](https://www.fao.org/4/y5686e/y5686e00.htm) |
| **FAO food-composition and food-balance coefficients; regional composition tables** | Product definitions and nutrient accounting | Check edible/as-purchased basis, moisture, preparation, and variety. [FAOHome](https://www.fao.org/4/x9892e/X9892e05.htm) |
| **FAOSTAT Food Balance Sheets** | National commodity mixtures and supply accounting | Availability is not intake; national annual averages conceal inequality and seasonality. [FAOHome](https://www.fao.org/statistics/highlights-archive/highlights-detail/food-balance-sheets-2010-2023/) |
| **APHLIS** | Crop-, location-, and stage-specific postharvest-loss estimates | Much of the output is modeled; it is not direct measurement of every stock or a prehistoric dataset. [JRC Publications](https://publications.jrc.ec.europa.eu/repository/handle/JRC62618) |
| **FEWS NET Livelihoods Explorer and baseline profiles** | Wealth-group food sources, expenditure, livelihood structure, seasonal calendars | Baselines are place- and reference-year-specific. [FEWS NET Help Center](https://help.fews.net/lex/about-the-livelihoods-explorer?utm_source=chatgpt.com) |
| **WHO growth standards and Global Database on Child Growth and Malnutrition** | Growth diagnostics and population wasting/stunting targets | Match age, season, survey design, and reference definitions. [World Health Organization](https://www.who.int/tools/child-growth-standards?utm_source=chatgpt.com) |
| **World Bank Food Prices for Nutrition** | Distinguish energy-sufficient, nutrient-adequate, and healthy-diet costs | Modern retail-price framework, not an ancient subsistence budget. [World Bank Data Catalog](https://datacatalog.worldbank.org/search/dataset/0061222/food-prices-for-nutrition-fpn?utm_source=chatgpt.com) |
| **FSNAU/FEWS NET famine reconstructions and IPC documentation** | Episode-level mortality and crisis diagnostics | Classification thresholds are reporting tools, not biological switches. [FEWS NET](https://fews.net/east-africa/somalia/special-report/may-2013) |

For an optional modern-style dashboard, IPC combines household deprivation, acute malnutrition, and mortality evidence. Its published famine criteria include approximately **20% of households facing extreme deprivation**, **30% child acute malnutrition by weight-for-height or approximately 15% by MUAC**, and a mortality criterion that can be met by **at least four under-five deaths per 10,000 under-five children per day**. These are joint area-level assessment criteria—not a rule that deaths begin only once a threshold is crossed. [IPCInfo](https://www.ipcinfo.org/famine-facts/)

### 6.2 Claims that should remain explicitly uncertain

**Historical cereal shares and spoilage rates are thinly measured.** Archaeological remains, accounts, and modern analogues often capture different things. Do not present a proposed 70% cereal diet or 10% annual storage loss as an established global fact.

**The relative roles of shortage and access differ among famines.** Bengal’s 1943 famine remains a major example of disagreement: Sen emphasized entitlement failure, while Tauger argued for a larger role for production shortfall. TCE should support both mechanisms and their interaction rather than encode either explanation as universal. [OUP Academic](https://academic.oup.com/book/32827)

**Undernutrition–mortality associations are not interchangeable causal coefficients.** Infection can cause both weight loss and death; wealth, care, sanitation, and displacement also matter. Use observed hazard ratios to check whether model outcomes are plausible, not as unquestioned constants.

**Adult work and fertility penalties are less precisely transferable than energy requirements.** Avoid false precision in generic productivity multipliers, infertility thresholds, starvation timers, or inherited penalties after famine.

### Bottom line

For TCE, the most convincing famine will emerge when **a real harvest shock changes inventories and expectations; prices and incomes redistribute access; households protect consumption by sacrificing assets; depleted people work and reproduce differently; and infection amplifies mortality**. Recovery should require restoring bodies, livelihoods, and institutions—not merely refilling the settlement’s food counter.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9281f-8f00-83e9-b8b2-45f86e05520a)
