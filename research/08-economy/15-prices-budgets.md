# Historical prices, wages, and household budgets

## A calibration report for The Civilization Engine

**TCE should calibrate relative prices, physical consumption, and household resource constraints—not assign a universal price list or spending pattern to each technological era.** The useful historical anchors are local: kilograms of staple food per day’s wage, the price of meat relative to grain, the value of livestock relative to a harvest, and the share of household resources needed for necessities.

Three distinctions are essential:

* **A subsistence basket is a constructed benchmark, not an observed household budget.**
* **A wage rate is not household income:** employment days, other earners, land, livestock, and payments in kind matter.
* **Cash expenditure is not total consumption:** households can produce food, fuel, clothing, and housing services themselves. Historical household reconstructions show that including these resources can substantially change the apparent standard of living. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1468-0289.2010.00515.x)

The recommendations below separate empirical observations, historical reconstructions, and proposed simulation rules.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Necessities constrain expenditure, but households do not maximize calories alone

**Rule:** Give households minimum requirements for nutrition, warmth, shelter, and clothing services. Allocate additional resources toward dietary variety, convenience, durability, comfort, and socially valued consumption.

This produces an *Engel curve*: expenditure on food can increase while its share of the budget declines. But do not model poor households as purchasing only the cheapest available calories. Banerjee and Duflo’s evidence from 13 countries includes spending on nonfood goods and social activities even among extremely poor households. Their selected rural samples typically devoted **56–78% of expenditure to food**, with important exceptions. [MIT Economics](https://economics.mit.edu/sites/default/files/publications/The%20Economic%20Lives%20of%20the%20Poor.pdf)

**TCE implication:** A richer household should not consume ten times the grain of a poorer one. It should shift toward different foods, preparation, clothing quality, housing, transport, services, and discretionary goods.

### 1.2 Calculate income from actual household activities

**Rule:** Household resources should aggregate earnings from each member, net household-enterprise income, transfers, and consumption obtained outside markets.

A useful decomposition is:

\[
Y\_h^{\mathrm{resources}}
=
Y\_h^{\mathrm{cash,net}}
+V\_h^{\mathrm{own\ consumption}}
+V\_h^{\mathrm{in\!-\!kind}}
+V\_h^{\mathrm{housing\ services}}.
\]

Maintain the cash components separately; imputed values do not pay a money tax or settle a shopkeeper’s debt.

The magnitude can be substantial. In Boter’s sample of **25 Dutch rural household budgets around 1910**, husbands’ earnings averaged **54% of total household income**, ranging from **10% to 97%**. Land contributed an average of about **25%**, again with wide variation. These are local observations, not universal household coefficients. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7687131/)

**TCE implication:** A family with a modest cash wage and access to land may consume more than a fully market-dependent family with a higher wage.

### 1.3 Prices should reflect complete production chains

**Rule:** Price processed goods from their material inputs, labor, energy, capital use, losses, transport, and market conditions.

For bread, an approximate unit-cost relationship is:

\[
c\_{\mathrm{bread}}
=
a\_g p\_{\mathrm{grain}}
+a\_f p\_{\mathrm{fuel}}
+\ell w
+c\_{\mathrm{milling}}
+c\_{\mathrm{capital}}
+c\_{\mathrm{transport}}
-v\_{\mathrm{byproducts}}.
\]

The input coefficient \(a\_g\) must account for milling extraction and baking yield. A kilogram of grain, flour, and bread are not interchangeable products.

This matters especially at low technology. Bowles’s reconstruction of early cultivation emphasizes storage, seed retention, processing, delayed returns, and risk—not merely harvesting labor. In some underlying cases, processing accounted for half or more of farming-related labor time. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3064343/)

**TCE implication:** A mill or efficient oven can improve living standards without raising agricultural yields. Conversely, cheap grain does not guarantee cheap bread when milling capacity or fuel is scarce.

### 1.4 Treat durable goods as stocks that provide services

**Rule:** Clothing, tools, houses, and livestock persist, deteriorate, and sometimes appreciate. Households demand their services; purchases occur when stocks are insufficient, worn out, or worth upgrading.

Recommended representations:

| Category | Service to represent | Important distinctions |
| --- | --- | --- |
| Clothing | Warmth, coverage, protection, status | Fabric area, weight, material, condition, repairability |
| Tools | Additional output or reduced labor | Task suitability, quality, wear, repair materials |
| Housing | Shelter, space, access to work | Ownership versus rent, crowding, location, maintenance |
| Fuel | Useful heat and light | Energy content, moisture, appliance efficiency, collection time |
| Livestock | Traction, food, offspring, materials | Species, age, sex, health, fertility, training |

Historical inventories are particularly valuable here because they record stocks of possessions rather than annual purchases. They also illustrate why an assessed used-tool value should not be treated as a new retail price. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70052)

**TCE implication:** Do not destroy a fixed fraction of everyone’s “clothing good” every day and replace it through the market. Accumulate wear, permit repair and secondhand exchange, and make replacement purchases uneven.

### 1.5 Harvests and storage should generate price seasonality

**Rule:** Maintain physical inventories and explicit harvest calendars:

\[
I\_{t+1}
=
(1-\delta\_t)I\_t
+H\_t+M\_t
-C\_t-X\_t-U\_t,
\]

where \(H\) is harvest, \(M\) imports, \(C\) consumption, \(X\) exports, and \(U\) seed, feed, and other production uses. Enforce \(I\_{t+1}\geq0\).

Storage is attractive when expected future sale proceeds exceed purchase, storage, loss, financing, and risk costs. When inventories approach zero, supply cannot be brought forward from nonexistent stocks, making prices more sensitive to additional shortages.

Competitive-storage models capture this asymmetry, but storage alone does not explain every feature of observed commodity-price persistence. Deaton and Laroque explicitly found limitations in that explanation. [National Bureau of Economic Research](https://www.nber.org/papers/w3439)

**TCE implication:** Generate seasonal prices from harvest timing, stockholding, transport, and credit. Avoid an independent “winter food price multiplier” that duplicates those effects.

### 1.6 Institutions change what households must purchase

**Proposed rule:** Property and allocation institutions should alter household rights and obligations, not merely modify a generic income number.

Examples include access to common fuel, a dwelling supplied with employment, food rations, compulsory deliveries, crop shares, public relief, and reciprocal transfers. These should change physical deliveries, market exposure, and cash requirements.

A household receiving housing and grain from an employer should not subsequently buy the same shelter and grain again. A household gathering fuel should pay the associated time cost rather than a fictitious cash bill.

### 1.7 Keep prices local and distinguish sellers from buyers

**Proposed rule:** Maintain settlement-level prices with transport and transaction costs connecting them. Households should face different effective prices depending on whether they sell farm output, buy at wholesale, buy small retail quantities, or exchange through an institution.

A useful household diagnostic is:

\[
\text{net staple position}
=
\text{staple production available}
-\text{household and production requirements}.
\]

The same price increase can benefit a net seller while harming a net buyer. Nevertheless, a household can sell shortly after harvest and purchase later; annual net position alone is insufficient. Research in Tanzania finds that seasonal prices and consumption interact differently across household groups, including seasonal caloric variation of about **10%** among poor urban households and rural net food sellers in the study. [OUP Academic](https://academic.oup.com/oep/article-abstract/68/3/736/1752693)

### 1.8 Saving must be an accounting outcome, not an era-specific percentage

**Rule:** Separate consumption, investment, borrowing, debt repayment, and changes in asset prices.

Buying a cow is an asset purchase, not ordinary food consumption. Borrowing is not income. Repaying principal is not consumption. A rise in the assessed value of land is not saving out of current income.

Poor households may save in cash, inventories, livestock, durable goods, or informal financial arrangements; lack of a bank account does not imply zero saving. [MIT Economics](https://economics.mit.edu/sites/default/files/publications/The%20Economic%20Lives%20of%20the%20Poor.pdf)

For calibration, avoid deriving a national saving rate mechanically from old budget surveys. The BLS’s 1901 worker-family figures report **$750 annual income and $769 expenditure**. The difference is approximately **−2.5% of reported income**, but it is not a reliable national household-saving estimate. It is a sample accounting discrepancy that may reflect dissaving, borrowing, and measurement differences. [Bureau of Labor Statistics](https://www.bls.gov/opub/100-years-of-u-s-consumer-spending.pdf)

---

## 2. Quantitative parameters and calibration anchors

### How to read confidence

**High** means a well-documented observation or clearly specified benchmark for the stated population. **Medium** means a reconstruction, selective sample, proxy, or substantial comparability limitation. **Low transferability** means a value may be accurately transcribed but should not be generalized to other places or periods.

These are evidence assessments, not statistical confidence intervals.

### 2.1 Relative prices of common goods

The following are **local anchors**, not a single internally consistent economy. Silver units preserve the source conversion; they are not a timeless measure of purchasing power.

| Place and period | Good | Source price | Useful relative-price measure | Evidence |
| --- | --- | --- | --- | --- |
| Strasbourg, 1745–54 | Bread | 0.693 g silver/kg | Local comparison base | Medium |
| Strasbourg, 1745–54 | Meat | 2.213 g silver/kg | **3.19 kg bread per kg meat** | Medium |
| Strasbourg, 1745–54 | Basket cloth, labeled cotton in the draft | 4.369 g silver/linear metre | **6.30 kg bread per metre** | Medium; cloth specifications matter |
| Strasbourg, 1745–54 | Fuel | 4.164 g silver/million Btu | **5.70 kg bread per GJ** of source energy | Medium; not useful heat |
| Canton, 1757 | Rice | 1.407 g silver/kg | Local staple base | Medium |
| Canton, 1757 | Beef | 2.447 g silver/kg | **1.74 kg rice per kg beef** | Medium |

Source: Allen and colleagues’ earlier working-paper tables; ratios are calculated from the listed unit prices. The paper identifies the European prices as Strasbourg averages. These tables contain proxies and should not be confused with the revised baskets in the published 2011 article. [GPIH](https://gpih.ucdavis.edu/files/Allen_et_al.pdf)

A better-documented industrial-era work-time comparison is available for the United States:

| United States, 1901 | Observed value | Derived purchasing power |
| --- | --- | --- |
| Manufacturing wage | **$0.23/hour** | Reference wage |
| Flour | **$0.13 per 5 lb** | **0.249 work-hours/kg** |
| Round steak | **$0.14/lb** | **1.34 work-hours/kg** |
| Steak relative to flour | — | **5.38 times the price per unit mass** |

These are national industry-wage and urban retail-price summaries, not matched purchases by one household. Evidence is high for the published figures and medium for the combined purchasing-power comparison. [Bureau of Labor Statistics](https://www.bls.gov/opub/100-years-of-u-s-consumer-spending.pdf)

**Calibration use:** Preserve both `price_per_physical_unit` and `work_time_per_unit`. The latter should specify occupation, location, year, and whether food or lodging accompanied the wage.

### 2.2 Livestock, cloth, and tools: useful evidence with strict limitations

A recently published archival study provides unusually explicit household-asset valuations.

| Inventory context | Item | Recorded valuation | Derived comparison |
| --- | --- | --- | --- |
| Hampshire, England, 1462 | Two cows | 10 shillings total | **5 shillings per cow on average** |
| Same inventory | One horse | 3 shillings 4 pence | **40 pence** |
| Same inventory | Three quarters of wheat | 6 shillings total | **24 pence per local quarter** |
| Same inventory | 1.5 yards of russet woollen cloth | 16 pence | **10.67 pence/yard** |
| Kent, England, 1577 | Iron bundle including an axe, rake, bill, spade, and cooking implements | 2 shillings 6 pence | **30 pence for the entire bundle** |

Thus, within the **1462 assessment**, an average cow was valued at **2.5 quarters of wheat**, and the horse at about **1.67 quarters**. Do not convert those quarters to kilograms without establishing the local measure. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70052)

**Confidence: medium for the recorded appraisals; low transferability.** These are forfeiture assessments of particular possessions, not representative market quotes for healthy animals or new tools. The iron bundle does not establish an individual axe price.

**TCE recommendation:** Use such records to test plausible *orders of magnitude and household asset composition*. Estimate new-tool prices from production recipes, and animal prices from expected services, reproduction, maintenance costs, and condition.

### 2.3 Observed household budget shares

Percentages below use each source’s expenditure denominator. **The columns are not fully harmonized across rows.**

| Population and period | Food | Fuel, light, or utilities | Housing or shelter | Clothing | Confidence |
| --- | --- | --- | --- | --- | --- |
| English agricultural laborers, 1787–96 | **77.1%**, including beer | **3.6% fuel** | **5.9% housing** | **8.4% “Other (Clothing)”** | Medium; historical sample |
| English agricultural laborers, 1830–54 | **66.3%**, including beer | **4.6% fuel** | **8.8% housing** | **15.8% “Other (Clothing)”** | Medium; historical sample |
| U.S. worker families, 1901 | **42.5%** | Not separated here | **23.3% source housing category** | **14.0%** | High locally; selective population |
| Rural India, 2023–24 | **About 47%** | **6.11% fuel and light** | Not harmonized here | **6.63% clothes, bedding, footwear** | High for survey definition |
| Urban India, 2023–24 | **About 40%** | **5.59% fuel and light** | Not harmonized here | **5.66% clothes, bedding, footwear** | High for survey definition |
| U.S. consumer units, 2023 | **12.9%** | **6.0% utilities, fuels, public services** | **20.1% shelter** | **2.6% apparel and services** | High for survey definition |

English observations are reproduced in Clark’s table from Horrell; light and soap were separately **5.0%** and **4.0%**, respectively. “Other (Clothing)” should not be silently relabeled pure clothing. [UCDavis Economics Faculty](https://faculty.econ.ucdavis.edu/faculty/gclark/papers/farm_wages_%26_living_standards.pdf) U.S. 1901 figures are from the BLS historical compilation. [Bureau of Labor Statistics](https://www.bls.gov/opub/100-years-of-u-s-consumer-spending.pdf) India’s categories come from the official HCES release and charts. [Press Information Bureau](https://pib.gov.in/PressReleasePage.aspx?PRID=2088390) The 2023 U.S. **total housing category was 32.9%**, including shelter, utilities, household operations, supplies, and furnishings; it must not be added to the shelter and utilities columns above. [Bureau of Labor Statistics](https://www.bls.gov/opub/reports/consumer-expenditures/2023/)

**Important interpretation:** These rows are examples of different populations, institutions, and accounting conventions—not a universal progression that every TCE civilization should follow.

For implementation, expose two dashboards:

\[
s\_i^{\mathrm{cash}}
=
\frac{\text{cash purchases of category }i}
{\text{all cash consumption purchases}}
\]

and

\[
s\_i^{\mathrm{total}}
=
\frac{\text{purchased + own-produced + in-kind consumption value of }i}
{\text{total consumption value}}.
\]

Compare an empirical survey only with the matching simulated definition.

### 2.4 Constructed subsistence baskets

Allen and colleagues’ published comparative work uses regional *bare-bones* baskets. These are valuable as reproducible diagnostics.

| Component | Quantity per reference person per year |
| --- | --- |
| Main staple | **155 kg oats**, **165 kg polenta**, **171 kg rice**, or **179 kg sorghum**, depending on regional basket |
| Beans or peas | **20 kg** |
| Meat or fish | **3–5 kg** |
| Oil or butter | **3 kg** |
| Cloth | **3 linear metres** |
| Soap, candles, lamp oil | **1.3 kg each** |
| Fuel | **3 million Btu ≈ 3.17 GJ** |
| Calculated dietary energy | Approximately **1,936–1,942 kcal/day** |

Their familiar wage-based welfare comparison uses **250 paid days**, **three reference-person baskets** for a model family, and a **5% addition to nonhousing basket costs for rent**:

\[
WR\_{\mathrm{Allen}}
=
\frac{250\,w\_{\mathrm{day}}}
{3\times1.05\times B\_{\mathrm{annual}}}.
\]

These are methodological conventions, not observations that everyone worked 250 days, formed the same household, or paid 5% of expenditure in rent. The rent addition corresponds to about **4.76% of the resulting total**. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1468-0289.2010.00515.x)

**TCE use:** Implement this as a historical-comparison metric alongside actual household adequacy. Do not make this basket the universal diet or treat its calorie level as adequate for every occupation and person.

### 2.5 Seasonal prices and wages

Seasonality is not the same as annual volatility. A predictable annual cycle, a harvest failure, and a monetary inflation episode require different parameters.

| Empirical target | Value | Scope and interpretation | Confidence |
| --- | --- | --- | --- |
| Maize seasonal price gap | **33.1%**, in the study’s reported convention | Across sampled African markets | Medium–high; substantial local variation |
| Rice seasonal price gap | **16.6%** | Same study | Medium–high |
| Tomato seasonal price gap | **60.8%** | Same study; highly seasonal perishables | Medium–high |
| English harvest/nonharvest money-wage ratio | **1.47–1.56** | Farm-account period averages, 1670–1850 | Medium |
| English harvest/nonharvest money-wage ratio | **1.77** | 1832 Poor Law benchmark | Medium |

Gilbert, Christiaensen, and Kaminski examine **193 markets, 13 foods, and seven African countries**. Their seasonal gaps are estimated from seasonal price models, not simply the largest observed monthly price divided by the smallest. Treat them as modern evidence for seasonal-market mechanisms—not direct estimates for ancient farming. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5384441/) Wage ratios are reported in Clark’s farm-wage study. [UCDavis Economics Faculty](https://faculty.econ.ucdavis.edu/faculty/gclark/papers/farm_wages_%26_living_standards.pdf)

For TCE validation, calculate separately:

\[
\ln p\_t = \text{trend}\_t+\text{season}\_{m(t)}+\varepsilon\_t.
\]

Measure the fitted seasonal amplitude, residual volatility, and extreme-price frequency separately. Otherwise a war or failed harvest can be mistaken for normal seasonality.

---

## 3. Variation across eras and regions

### Foragers: calibrate time, risk, and access before money

A universal monetary budget for foragers would be largely invented. Begin with food acquisition, processing, travel, sharing obligations, tool maintenance, and shelter work.

Ethnographic populations are not interchangeable with prehistoric ones. Dyble and colleagues’ study of Philippine Agta communities nevertheless provides a useful warning: greater agricultural participation was associated with less leisure, especially for women. Subsistence transitions need not immediately make each household’s working life easier. [Nature](https://www.nature.com/articles/s41562-019-0614-6)

**TCE scenario axes:** Resource density, seasonal mobility, household dependency, food-sharing networks, and the carrying costs of possessions.

### Early farming: annual surplus and labor productivity are different

Bowles’s reconstruction challenges the assumption that the first farmers necessarily obtained more food per hour than foragers. The result is uncertain and sensitive to reconstruction, but it demonstrates why processing, seed, storage, and risk must enter the comparison. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21383181/)

**TCE scenario axes:** Yield per area, labor per task, land availability, harvest concentration, crop diversity, processing technology, and storage security.

A farming household can support higher population density while working longer and remaining exposed to a concentrated harvest risk. Do not equate greater settlement output with greater individual welfare.

### Ancient and preindustrial states: distinguish exchange from assessment and rationing

Ancient evidence often records administrative prices, rations, or assessed obligations rather than representative retail transactions. Diocletian’s Price Edict is an especially important example: it concerns prescribed maximum prices, not a modern market-price survey. Allen’s Roman living-standard reconstruction is therefore useful as an explicit reconstruction, not an unquestionable price list. [OUP Academic](https://academic.oup.com/book/8619/chapter-abstract/154580697)

**TCE scenario axes:** Dependence on wages versus land, ration entitlements, access to common resources, household craft production, market distance, and obligations in money versus goods.

### Preindustrial regions should not share one consumption template

The price comparisons above already show that even the meat-to-staple ratio differs markedly by place. Regional baskets also substitute different staples rather than assuming a European bread diet everywhere. [GPIH](https://gpih.ucdavis.edu/files/Allen_et_al.pdf)

For authored content, allow households to satisfy nutritional functions using locally available grains, roots, pulses, animal foods, and preparation techniques. Cultural preferences should influence substitution without making households incapable of adaptation.

The empirical coverage is much wider than northwestern Europe. GPIH catalogs include price and wage material for **India, China, Japan, Korea, Istanbul, Egypt, southern Africa, and multiple Latin American economies**. Their density and comparability vary substantially. [GPIH](https://gpih.ucdavis.edu/Datafilelist.htm)

### Industrialization: commercialization changes measured budgets

As more household activities become purchased services, cash spending can rise without an equivalent increase in the underlying physical need.

**TCE implementation:** Let technology and organization change production recipes, household time use, and what is bought. A household may move from making clothing to purchasing it, or from processing grain at home to buying bread. Do not infer welfare solely from the resulting increase in market expenditure.

### Modern economies: income and institutions still outweigh an “era label”

Modernity does not imply a low food share everywhere: the India and U.S. observations above remain far apart. Nor does a low food share imply cheap housing. U.S. shelter alone represented **20.1%** of expenditure in 2023, considerably more than the food share. [Press Information Bureau](https://pib.gov.in/PressReleasePage.aspx?PRID=2088390)

**TCE scenario axes:** Income distribution, housing supply, tenure, transport requirements, public services, household size, and the price of services relative to manufactured goods.

---

## 4. Stylized facts and validation tests

A correct simulation should reproduce these **conditional patterns**, not force every historical outcome.

| Pattern to reproduce | Suggested TCE test |
| --- | --- |
| Food shares generally decline as resources rise | Compare otherwise similar households within the same settlement; use the budget table as scenario-specific anchors, not a global target |
| Higher income changes composition as well as quantity | Verify that dietary quality, clothing services, housing, and discretionary demand grow without proportional multiplication of staple calories |
| Wage rates and family welfare can diverge | Hold one worker’s wage constant while changing paid days, dependants, land access, and other earners |
| Seasonal pressure differs by commodity | Compare grain, perishable vegetables, livestock products, and goods with year-round production |
| Storage buffers some shocks but cannot eliminate scarcity | Reduce inventories before an identical harvest shock and test whether prices and unmet needs become more sensitive |
| Asset ownership matters separately from current expenditure | Compare families with the same cash income but different tools, livestock, clothing stocks, or housing rights |
| Households can remain poor while saving intermittently | Permit small buffer accumulation followed by dissaving, borrowing, or asset sales during shocks |
| Better daily wages need not imply proportionate gains in possessions | Validate consumption and asset stocks independently rather than deriving both mechanically from a wage index |

The empirical foundations are complementary: household budgets constrain expenditure composition; storage research constrains price dynamics; household-income reconstructions constrain resource accounting; and inventories constrain material possessions. None alone supplies a complete welfare measure. [UCDavis Economics Faculty](https://faculty.econ.ucdavis.edu/faculty/gclark/papers/farm_wages_%26_living_standards.pdf)

**A particularly useful stress test:** Give two otherwise identical households different ownership arrangements. One owns a dwelling and produces some food; the other rents and buys everything. The simulation should explain differences in cash stress, consumption, and asset accumulation without assigning different innate “wealth needs.”

---

## 5. Recommended representation for TCE

### 5.1 Simulate people, budget through households

For 10,000–50,000 individuals, use the household as the principal purchasing and stockholding unit while retaining individual labor, consumption, health, preferences, and ownership claims.

A practical state division is:

| Entity | Minimum economic state |
| --- | --- |
| Person | Household membership, capabilities, available time, job, individual requirements |
| Household | Cash, debts, inventories, durable stocks, dependants, transfers, purchase plans |
| Enterprise or household business | Inputs, outputs, labor commitments, capital, operating accounts |
| Dwelling or plot | Ownership, access rights, rent obligations, maintenance, capacity |
| Market | Prices by good and grade, offers, completed trades, unmet orders |
| Institution | Taxes, rations, relief, entitlement and enforcement rules |

Do not merge household-business input purchases with household consumption. Feed, seed, and replacement business tools must remain identifiable.

### 5.2 Use a compact demand system with explicit minimums

A useful starting point is a **linear expenditure system**:

\[
q\_i
=
b\_i+
\frac{\alpha\_i}{p\_i}
\left(E-\sum\_jp\_jb\_j\right),
\qquad
\sum\_i\alpha\_i=1.
\]

Here:

* \(b\_i\) is a minimum service requirement;
* \(E\) is the planned consumption budget;
* \(\alpha\_i\) allocates resources remaining after minimum costs.

This is a **modeling proposal**, not a claim that historical people solved this equation.

The system is computationally cheap and naturally produces declining necessity shares. However, it requires three extensions:

**Substitution within needs.** Represent “food,” “warmth,” and “clothing service” as requirements that multiple goods can satisfy. Include preparation time and equipment, not just money price.

**An explicit shortfall branch.** When \(E<\sum p\_jb\_j\), do not allow negative quantities. Allocate constrained resources, request transfers or credit, reduce discretionary purchases, and record unmet requirements.

**Durable-stock logic.** Clothing and shelter services usually come from existing stocks. The demand function should produce repair, replacement, or upgrade plans—not pretend all services are newly purchased each period.

### 5.3 Make saving a forward-looking buffer decision

A simple household policy is:

\[
\text{desired liquid buffer}
=
\text{expected cash needs until next reliable receipt}
+
\text{uncertainty allowance}.
\]

Estimate the first term from the actual calendar. A regularly paid worker and a once-yearly harvest household should not share a fixed savings rule.

For sensitivity experiments, test **0, 30, and 90 days of cash-consumption coverage** as alternative uncertainty allowances. These are **design priors**, not estimated historical norms.

Allow households to choose among cash, storable goods, debt reduction, productive assets, and durable improvements. Store expected liquidity and sale costs: a house, a cow, and a sack of grain are not equally useful for paying tomorrow’s bill.

### 5.4 Keep livestock and tools tied to productive value

For an animal, use an approximate willingness-to-pay calculation:

\[
V\_{\mathrm{animal}}
\approx
PV(\text{traction, products, offspring, other services})
-PV(\text{feed, care, mortality risk})
+\text{expected terminal value}.
\]

For tools:

\[
V\_{\mathrm{tool}}
\approx
PV(\text{additional output or labor saved})
-PV(\text{maintenance}),
\]

subject to household credit and access to substitutes.

These need not be exact discounted-cash-flow optimizers. A cached estimate and bounded heuristic are sufficient. The key is that a working ox has value beyond its meat, while a tool that cannot be used productively should not command the same household willingness to pay.

### 5.5 Use multiple update frequencies

Recommended starting schedule:

| Process | Suggested frequency |
| --- | --- |
| Eating, physical production, wear, spoilage | Daily or event-driven |
| Routine purchasing | Staggered daily/weekly household schedules |
| Employment and production-plan revision | Weekly or on major events |
| Rent and debt payments | Contract dates |
| Crop harvest and agricultural labor peaks | Seasonal events |
| Calibration statistics | Monthly, seasonal, and annual aggregates |

Keep the physical simulation authoritative. Market summaries can be aggregated, but a statistical budget share must never create goods that were not produced, imported, or withdrawn from stocks.

For price adjustment, use inventory pressure and unmet orders with a tunable response rate. Record attempted purchases as well as completed trades: otherwise the poorest households can disappear from measured demand precisely when necessities become unaffordable.

### 5.6 Relevant existing models and games

| Model or game | Useful pattern | What not to copy directly |
| --- | --- | --- |
| **Village Ecodynamics Project / “Village”** | Household agents acquire calories, protein, fuel, and water from a changing landscape; extensions include specialization and reciprocal or barter exchange | Its regional subsistence assumptions are not a universal economy |
| **Sugarscape** | Demonstrates how exchange, resource heterogeneity, and agent differences generate aggregate market behavior | Highly abstract resources are insufficient for historical household budgets |
| **Victoria 3’s documented consumption design** | Need categories, substitution among goods, and consumption patterns that change with prosperity | A scalar prosperity/wealth level should not replace TCE’s actual cash, debts, and durable possessions |

The Village specialization model is described in *JASSS*; Sugarscape in Epstein and Axtell’s *Growing Artificial Societies*. [JASSS](https://www.jasss.org/16/4/4.html) Victoria 3’s 2021 developer diary describes wealth-dependent needs and substitute goods; this is a reference to that documented design, not a claim about every detail of the current build. [Steam Community](https://steamcommunity.com/games/529340/announcements/detail/2965045886379003737)

---

## 6. Datasets, source strategy, and evidence limits

### 6.1 Datasets worth building the calibration pipeline around

| Source | What it supplies | Best use and main limitation |
| --- | --- | --- |
| **Allen–Unger Global Commodity Prices Database, hosted by IISH/IISG** | Multi-century commodity-price series, original units and currencies, and standardized price information | Relative prices and price histories; not a complete household-income database |
| **UC Davis Global Price and Income History — GPIH** | Catalog of price, wage, rent, and conversion datasets across regions | Assemble matched local panels; inspect each contribution’s definitions |
| **Horrell household budgets, as used by Clark and related research** | Historical expenditure composition | British working-household calibration; selected samples rather than universal budgets |
| **BLS historical consumer-spending compilation and Consumer Expenditure Surveys** | U.S. budgets, selected historical prices and wages, modern category detail | Industrial and modern scenarios; populations and definitions change over time |
| **India HCES 2023–24** | Rural/urban consumption categories and distributions | Non-Western modern calibration; preserve the treatment of imputed consumption |
| **Gilbert–Christiaensen–Kaminski study and replication materials** | Monthly food-price seasonality and estimation methods | Seasonal market diagnostics; not ancient-price estimates |
| **Household inventories and forfeiture datasets** | Durable possessions, livestock, stored goods, assessed values | Asset stocks and material living standards; selection and valuation biases are substantial |

The Allen–Unger database has a scholarly data description by **Robert Allen and Richard Unger (2019)** and persistent identifier **hdl:10622/3SV0BO**. [Research Data Journal](https://researchdatajournal.org/article/view/24730) GPIH’s catalog includes, among many others, **London/Southern England 1295–1914, Istanbul 1469–1914, India 1595–1930, Beijing 1738–1923, Japan 1600–1938, Mexico 1701–1813, and Cape wage material for 1837–1913**. Coverage differs by variable. [GPIH](https://gpih.ucdavis.edu/Datafilelist.htm)

The survey sources and seasonal-study documentation support the other entries. [Bureau of Labor Statistics](https://www.bls.gov/opub/100-years-of-u-s-consumer-spending.pdf) The Briggs and colleagues study supplies a recent example of systematic archival asset evidence. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70052)

The catalogs and papers were inspected for this report; this is **not** a claim that every underlying spreadsheet has been downloaded, cleaned, or reconciled.

### 6.2 Preserve provenance in every calibration record

Recommended fields:

```
place, observation_date, period_covered
commodity, grade, condition, processing_stage
original_quantity, original_unit, original_currency
price_or_value, transaction_type
retail_wholesale_ration_assessment_flag
wage_occupation, worker_category, payment_period
food_lodging_included, paid_days_observed
unit_conversion, conversion_source
observed_imputed_interpolated_flag
source_identifier, table_or_page, evidence_note
```

Do not discard original observations after conversion.

For cross-source fitting, match place, period, product, and transaction type before calculating ratios. Keep imputed observations out of volatility estimation where the imputation mechanically smooths or copies price movements.

### 6.3 The largest uncertainties

**Prehistoric monetary budgets are largely unavailable.** Use material and time constraints, with explicit reconstruction uncertainty.

**Daily male wage series incompletely describe households.** They are useful labor-market evidence, but employment, other earners, land, and in-kind resources alter the relationship to welfare. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7687131/)

**Some long price series contain extensive reconstruction.** In Allen and colleagues’ Chinese comparison, several nongrain series rely on later relative prices, and grain inputs include smoothing. Such series are more suitable for particular long-run comparisons than for estimating short-run price volatility. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1468-0289.2010.00515.x)

**Housing is especially difficult to harmonize.** Rent paid, employer-provided accommodation, owner-occupied housing services, repairs, fuel, and furnishings may be grouped differently. Modern U.S. category detail illustrates the problem directly. [Bureau of Labor Statistics](https://www.bls.gov/opub/reports/consumer-expenditures/2023/)

**Inventories measure assessed stocks, not necessarily sale prices or welfare.** Their selection and recording practices can change. Recent archival work also cautions against assuming that movements in real wages map directly onto changes in household possessions. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70052)

### Recommended calibration order

Start with a few **matched local scenarios**, rather than one global set of averages:

1. Fit physical production and household requirements.
2. Fit employment, ownership, transfers, and access to nonmarket resources.
3. Fit relative prices and household expenditure composition.
4. Fit seasonal dynamics and shock responses.
5. Validate asset accumulation, debt, and material consumption independently.

**The central design target is not “a historically correct price of bread.” It is an economy in which bread, rent, fuel, clothing, and productive assets become expensive or affordable for intelligible reasons—and in which those reasons produce historically plausible household behavior.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9289c-13dc-83ea-ad7f-3bb27d37e5b8)
