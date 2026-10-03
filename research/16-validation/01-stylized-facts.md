# Stylized facts for validating The Civilization Engine

## Executive recommendation

**TCE should validate conditional patterns, not enforce universal social laws.** A plausible early-agrarian world may have no meaningful city-size power law, no modern firms and no persistent childhood epidemic. Those are not necessarily failures.

For TCE’s starting conditions, I would prioritize **household consumption, food inventories and seasonality, demographic accounting, wealth transmission, and stochastic epidemic behavior**. Treat Zipf, Pareto and Gibrat as later-stage distributional diagnostics whose applicability depends on institutions, settlement structure and sample size.

The empirical literature supports this distinction: city-size exponents vary substantially with definitions and samples; food demand has comparatively strong regularities; income–fertility relationships change across development contexts; and epidemic persistence depends on population size and connectivity. [EconStor](https://www.econstor.eu/bitstream/10419/49914/1/668835885.pdf)

The dashboard should distinguish three things:

| Validation layer | What it establishes | Appropriate response |
| --- | --- | --- |
| **Accounting and logical consistency** | People, goods, ownership claims and disease states reconcile. | Hard errors when violated. |
| **Conditional empirical compatibility** | Comparable simulated populations resemble relevant observations. | Warnings with uncertainty and applicability checks. |
| **Mechanism consistency** | Changes in outcomes can be traced to the modeled causes. | Explanatory diagnostics, not a universal numerical score. |

A distribution can look realistic for the wrong reasons. Conversely, an unusual but well-explained world should remain possible.

---

## 1. Mechanisms: implement processes that can generate the patterns

The following are **recommended TCE mechanisms**, not claims that the literature has identified one uniquely correct causal model.

### Settlements: opportunity, accessibility and constraints

Let households compare expected livelihood, food security, safety, kin connections and services against housing, travel and relocation costs. Settlements attract activity through markets, specialization and shared infrastructure, while land scarcity, congestion, disease exposure and transport costs limit growth.

Implement settlement founding, migration, expansion and abandonment through those decisions. Do not transfer population between cities merely to improve a rank-size plot.

For measurement, preserve both administrative boundaries and a consistent physical or functional settlement definition. Empirical city-size results change when researchers use municipalities rather than agglomerations, or include small settlements rather than only the upper tail. Eeckhout’s analysis of all U.S. places also illustrates why a plausible upper-tail Pareto fit does not establish that the entire settlement distribution is Pareto. [EconStor](https://www.econstor.eu/bitstream/10419/49914/1/668835885.pdf)

### Wealth: accumulation, losses and institutional transmission

Track wealth through productive assets, inventories, land-use rights, financial claims and liabilities. Changes should arise from earnings, consumption, investment, asset-price changes, transfers, inheritance and losses.

Use heterogeneous opportunities and risks, not a periodically redrawn Pareto distribution. Inheritance rules, communal ownership, taxation and access to credit should change accumulation and persistence.

Crucially, do not reduce all social advantage to money. Comparative research distinguishes **material, embodied and relational wealth**—such as livestock, skills and social connections—and finds different transmission patterns across subsistence systems. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

For TCE, these should remain separate state variables even when the dashboard offers a composite measure.

### Firms: entry, growth, contraction and exit

Begin with the production units that actually exist: households, farms, workshops, partnerships, estates, guild-controlled enterprises or incorporated firms. A modern employer firm should not be the default organizational unit of an early farming society.

A workable rule is to expand employment or capacity when expected additional revenue exceeds wages, inputs, financing and organizational costs. Allow failed ventures, shrinking enterprises, changing ownership and new entrants.

Measure organization size separately by employment, assets and sales. Axtell’s U.S. results concern employer firms and show similar—but not identical—employment and receipts distributions. They do not justify assigning every historical workshop a modern corporate size distribution. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/ZipfDistributionFirmSizes.RAxtell2001.pdf)

### Food consumption: subsistence needs plus diminishing expenditure shares

Give households nutritional requirements, diminishing utility from additional staple quantities, and preferences for variety, quality, preparation and other goods.

An inexpensive implementation above subsistence is a Stone–Geary-style allocation:

\[
F=p\_f\bar q+b\_f(x-p\_f\bar q),
\]

where \(F\) is food expenditure, \(x\) is total consumption expenditure, \(p\_f\bar q\) is the cost of a subsistence food quantity, and \(0<b\_f<1\) is the fraction of expenditure above subsistence allocated to food.

This produces declining food-budget shares as expenditure rises. Below subsistence, switch to explicit rationing, substitution and nutritional consequences rather than allowing negative discretionary expenditure.

Do not equate expenditure with calories: households can spend more on food quality without proportionately increasing energy intake. African demand research explicitly distinguishes food, nutrient and calorie elasticities. [JRC Publications](https://publications.jrc.ec.europa.eu/repository/handle/JRC106061?mode=full)

### Seasonal prices: harvests, stocks and trade—not a price sine wave

For each commodity and storage location, reconcile:

\[
K\_{t+1}=(1-\delta\_t)K\_t+H\_t+M\_t-C\_t-X\_t,
\]

where stocks \(K\), harvest \(H\), imports \(M\), consumption and other domestic uses \(C\), and exports \(X\) use the same physical units. The loss rate \(\delta\_t\) applies to the specified simulation interval.

Let harvest timing change supply; storage, credit, transport and trade determine how availability evolves between harvests. Prices then emerge from transactions, bids or another explicit clearing mechanism.

Use crop- and location-specific calendars. Multiple harvests, irrigation, imports and perishability can produce very different seasonal profiles. The African evidence shows large differences between commodities rather than one universal “preindustrial seasonal amplitude.” [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5384441/)

### Fertility: age-specific decisions and constraints

Model births through age, partnership exposure, fecundity, nutrition, postpartum spacing, desired surviving children and fertility-control capabilities.

Economic circumstances can affect several channels in opposite directions: resources available for children, child labor, schooling costs, care time and alternative opportunities for parents. Do not directly apply `fertility = constant − coefficient × income`.

The modern fertility literature documents weakening or reversal of familiar income and female-employment relationships in some high-income settings, making a fixed negative coefficient particularly unsuitable for an institution-generating simulation. [Northwestern Faculty](https://faculty.wcas.northwestern.edu/mdo738/research/Doepke_Hannusch_Kindermann_Tertilt_Handbook_23.pdf)

Track **period fertility**, **completed cohort fertility** and **surviving offspring** separately.

### Crime: opportunities and constraints, not a poverty switch

Represent distinct acts—taking property, assault, homicide, fraud—and separately determine their legal classification, detection, reporting and punishment.

A proposed decision model should combine expected gain, legitimate alternatives, urgency, opportunity, guardianship, perceived sanctions, relationships and individual dispositions. Poverty can affect some of these channels without determining behavior.

Evidence linking unemployment to property crime is stronger than the corresponding evidence for violent crime in a prominent U.S. study. That is not evidence for a universal poverty–violence multiplier. [JSTOR](https://www.jstor.org/stable/10.1086/320275)

Keep actual incidents separate from recorded incidents. Otherwise better policing can misleadingly appear to create crime simply because detection rises.

### Epidemics: contacts, susceptible people and stochastic extinction

Use individual infection states and transmission opportunities in households, workplaces, markets and travel. Add environmental or vector pathways only for diseases that require them.

For a contact-associated infection hazard \(h\) over duration \(\Delta t\), a consistent event probability is:

\[
p=1-e^{-h\Delta t}.
\]

Latent periods, infectious durations, immunity, births and introductions then produce the aggregate curve. Do not generate a bell curve and distribute its cases among agents.

An individual-based framework such as Covasim is a useful architectural precedent for contact layers and disease-state transitions, although its COVID-specific parameterization is not a historical disease model. [PLOS](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1009149)

---

## 2. Quantitative reference pack

### How to interpret the numbers

These values are **output benchmarks**, not coefficients that agents should obey.

“High confidence” below means strong support for the stated result **within its reference context**, not universal validity. “Moderate” indicates substantial dependence on measurement, population or specification. Historical transferability is stated separately.

A reported cross-study range, standard error, interquartile range and confidence interval are different objects. The dashboard must preserve that distinction.

### 2.1 Cities, wealth and firms

Use one exponent convention throughout:

\[
\Pr(X\ge x\mid X\ge x\_{\min})
=\left(\frac{x}{x\_{\min}}\right)^{-\alpha}.
\]

Here \(\alpha\) is the **upper-tail complementary cumulative distribution exponent**. The corresponding rank-size relationship is \(X\_r\propto r^{-1/\alpha}\); the probability-density exponent is \(\alpha+1\). Zipf corresponds to \(\alpha=1\).

| Regularity and metric | Quantitative reference | Scope, confidence and TCE interpretation |
| --- | --- | --- |
| **City-size Zipf exponent \(\alpha\)** | Nitsch’s synthesis of **515 estimates from 29 studies**: mean **1.09**, median **1.08**, full range **0.49–1.96**. **36%** lay outside **0.8–1.2**. | **Moderate** support for approximate rank-size regularity; weak support for an exact exponent. The full range is historical research variation, not an acceptance interval. [EconStor](https://www.econstor.eu/bitstream/10419/49914/1/668835885.pdf) |
| **Modern wealth-tail exponent \(\alpha\)** | One Forbes-augmented estimation in Vermeulen’s U.S./European comparison, using a **€1 million threshold**, gives **1.39–2.03** across ten countries; U.S. **1.47**. | **Moderate**, highly dependent on upper-tail coverage, threshold and estimator. Useful for modern privately owned wealth, not a default for early agrarian holdings. [European Central Bank](https://www.ecb.europa.eu/pub/pdf/scpwps/ecbwp1692.pdf) |
| **Comparative wealth inequality** | Weighted averages of wealth-type Ginis: foragers **0.25 ± 0.04 SE**; horticulturalists **0.27 ± 0.03**; pastoralists **0.42 ± 0.05**; agriculturalists **0.48 ± 0.04**. | **Moderate** within a comparative sample of 21 populations. These are **not pooled net-worth Ginis** and must not be compared directly with modern household net-wealth Ginis. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf) |
| **Employer-firm size exponent** | U.S. 1997 employment: \(\alpha=\mathbf{1.059}\), **SE 0.054**. Receipts: \(\alpha=\mathbf{0.994}\), **SE 0.064**. | **High** confidence in the cited historical result; **moderate** portability. A benchmark for developed employer-firm systems, not all productive organizations. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/ZipfDistributionFirmSizes.RAxtell2001.pdf) |
| **Firm-size skew** | In Axtell’s 1997 positive-employment subset: median **3 employees**, mean **21.8 employees**. | A concrete illustration of many small firms and a much smaller number of large employers. Not an obligatory mean for TCE. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/ZipfDistributionFirmSizes.RAxtell2001.pdf) |
| **Gibrat’s mean-growth proposition** | In \(g=a+b\ln S+\cdots\), the size-independence hypothesis is **\(b=0\)**. | This is a **null hypothesis, not a measured universal coefficient**. Test cities and firms separately; distinguish established entities from entrants and exits. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2F0002828043052303) |
| **Firm-growth variability** | Selected U.S. analyses report \(\sigma(g\mid S)\propto S^{-\beta}\), with **\(\beta\approx0.15–0.21\)**. | **Moderate**, sample-dependent. Larger firms can have less volatile proportional growth even when mean growth is approximately size-independent. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S037843710200852X) |

**Three qualifications matter especially.**

First, approximate Gibrat behavior does not require equal growth variance. A useful dashboard should show conditional mean growth and conditional volatility separately. International firm evidence also finds non-Gaussian growth distributions; there is no reason to require one exact normal or Laplace distribution. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0378437115002393)

Second, **wealth is not income**. Fit wealth tails only to positive, consistently valued wealth above a defensible threshold. Report debtors and zero-wealth households separately rather than shifting their values upward to make logarithms possible.

Third, a Pareto-looking upper tail does not imply an entire population follows a strict Pareto law. More flexible tail models can materially change estimates of concentration. “The top 20% must own 80%” should never be a TCE invariant. [IDEAS/RePEc](https://ideas.repec.org/a/spr/joecin/v20y2022i1d10.1007_s10888-021-09514-6.html)

### 2.2 Engel’s law and seasonal prices

Define the expenditure elasticity as

\[
\epsilon\_F=\frac{d\ln F\_{\mathrm{real}}}{d\ln x\_{\mathrm{real}}}.
\]

Engel’s law concerns a declining food **budget share** as household resources rise. It does not imply that richer households spend less money on food.

| Metric | Quantitative reference | Scope and confidence |
| --- | --- | --- |
| **Broad food expenditure elasticity, lower-income countries** | **0.71–0.85**; group average **0.78**. | USDA model estimates using **144 countries, 2005 ICP data**. Category includes food, beverages and tobacco; restaurant/catering expenditure is included in food. **High** directional confidence, **moderate** coefficient portability. [Economic Research Service](https://www.ers.usda.gov/media/8559/tb-1929.pdf?v=80156) |
| **Same elasticity, middle-income countries** | **0.57–0.71**. | Same dataset and model; range across countries, not a confidence interval. [Economic Research Service](https://www.ers.usda.gov/media/8559/tb-1929.pdf?v=80156) |
| **Same elasticity, high-income countries** | **0.35–0.56**; group average **0.50**. | Not a calorie elasticity and not a pure raw-staple elasticity. [Economic Research Service](https://www.ers.usda.gov/media/8559/tb-1929.pdf?v=80156) |
| **Food share among very poor households** | Typically **56–78%** of expenditure in rural samples and **56–74%** in urban samples. | Banerjee–Duflo’s 13-country study; exceptions exist. **Moderate** as a resource-constrained household reference, not a universal subsistence threshold. [MIT Economics](https://economics.mit.edu/sites/default/files/2022-08/The%20Economic%20Lives%20of%20the%20Poor.pdf) |
| **Seasonal price gap** | Mean estimated gaps: maize **33.1**, rice **16.6**, tomatoes **60.8 log-percentage points**. | **193 markets, 13 foods, seven African countries, 2000–2012**. Strong commodity variation; **moderate** portability to similar market/storage environments. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5384441/) |

The price-gap definition deserves explicit treatment:

\[
G\_{\log}=100\,[\max\_m(s\_m)-\min\_m(s\_m)],
\]

where \(s\_m\) is the estimated seasonal component of log price. A **33.1 log-point** gap corresponds to a peak/trough price ratio of \(e^{0.331}\), or approximately a **39.2%** premium. That conversion is mathematical; the reported average gap is not automatically the arithmetic average of market-level percentage premiums.

For TCE, compare household consumption at consistent prices and account for household size. Include consumed own-production, gifts and in-kind transfers; otherwise a subsistence farmer can appear to have negligible food consumption expenditure.

Do not calibrate staple calories directly to the broad USDA category.

### 2.3 Fertility, crime and epidemics

| Metric | Quantitative reference | Scope and confidence |
| --- | --- | --- |
| **Male status and reproductive success** | Meta-analytic association **\(r=0.19\)**, from **288 associations, 46 studies, 33 nonindustrial societies**. | Status includes wealth, influence and other advantages; reproductive success has several measures. **Moderate** support for a positive association, **not an income–TFR coefficient**. [PubMed](https://pubmed.ncbi.nlm.nih.gov/27601650/) |
| **Global period total fertility rate** | UN 2024 reference: approximately **2.25 births per woman**. | Historical 2024 estimate/projection in the UN’s *World Fertility 2024* report, not a timeless target or a 2026 measurement. **High** confidence as a broad demographic reference. [ResearchGate](https://www.researchgate.net/publication/391672553_World_Fertility_2024) |
| **Regional period fertility, 2024** | Eastern/South-Eastern Asia **1.34**; Latin America/Caribbean **1.80**; Central/Southern Asia **2.24**; sub-Saharan Africa **4.26 births per woman**. | Large differences exist within “modern” societies. Regional aggregates conceal within-region variation. [ResearchGate](https://www.researchgate.net/publication/391672553_World_Fertility_2024) |
| **Replacement fertility** | Approximately **2.1 births per woman under low mortality**. | Do not apply 2.1 to high-mortality historical populations. Replacement is fundamentally a survival-adjusted reproduction condition. [ResearchGate](https://www.researchgate.net/publication/391672553_World_Fertility_2024) |
| **Unemployment and property crime** | Selected U.S. estimates imply roughly **1–2%** changes in individual property-crime rates per **1 percentage-point** unemployment change. | **Moderate** evidence in that setting; weak historical portability. Unemployment is not poverty, and violent-crime evidence is weaker. [JSTOR](https://www.jstor.org/stable/10.1086/320275) |
| **Seasonal influenza reproduction-number estimates** | Median reported \(R\): **1.28**; **IQR 1.19–1.37**. | Systematic review; estimates vary by setting and method and do not all have identical interpretations. **Moderate**, disease/context-specific. [Springer](https://link.springer.com/article/10.1186/1471-2334-14-480) |
| **1918 influenza reproduction-number estimates** | Median **1.80**; **IQR 1.47–2.27**. | Across reviewed estimates, not a parameter interval applicable to every outbreak. [Springer](https://link.springer.com/article/10.1186/1471-2334-14-480) |
| **Measles critical community size** | Approximately **250,000–400,000 people** in historical pre-vaccination analyses. | **Moderate**, conditional on demography, mixing and importation. Particularly important for TCE’s 10k–50k scale. [Science](https://www.science.org/doi/10.1126/science.275.5296.65) |

### What not to turn into a number

There is no defensible universal coefficient for “poverty causes crime” or “income reduces fertility” across all TCE societies. For ancient and early farming populations, globally comparable annual crime rates and income-conditioned fertility schedules are especially poorly supported by the sources reviewed here.

That absence should produce **“reference unavailable”**, not an invented numerical band.

---

## 3. Variation across eras and regions

These categories should select comparison data, not function as mandatory eras in the simulation.

| Context | Appropriate expectations | Comparisons to avoid |
| --- | --- | --- |
| **Foraging societies** | Examine access to food, seasonal availability, sharing, mobility and separate material/social advantages. Comparative data do not support treating foragers as uniformly equal. | Requiring cities, employer firms, monetary budget shares or a modern net-wealth tail where those objects do not exist. The comparative wealth measures are broader than cash assets. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf) |
| **Early farming** | Emphasize harvests, storage, household reproduction, land access, surplus and inheritance. Evidence for a Neolithic demographic transition supports fertility increases in relevant settings, but not one universal fertility schedule. | A single “early farming TFR” or a fixed inequality jump triggered by agriculture. Archaeological demographic evidence is often indirect. [Science](https://www.science.org/doi/10.1126/science.1208880) |
| **Preindustrial settled societies** | Examine household and estate production, local market networks, transport constraints, asset transmission and recurrent epidemic introductions. Compare settlements with historically appropriate definitions. | Modern corporate-size benchmarks and mandatory Zipf exponents. City-size research finds substantial historical and definitional variation. [EconStor](https://www.econstor.eu/bitstream/10419/49914/1/668835885.pdf) |
| **Industrializing societies** | Enable firm-size and growth diagnostics as employer organizations, credit and market integration become relevant. Track mortality and fertility separately rather than forcing an instantaneous demographic transition. | A single transition date or an assumption that all income gains immediately lower fertility. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/ZipfDistributionFirmSizes.RAxtell2001.pdf) |
| **Modern societies** | Use the richest quantitative comparison sets, matched by institutions and measurement. Large regional differences in fertility and large commodity-specific seasonal price gaps can coexist with modern technology. | Treating wealthy Western countries as the only modern reference, or assuming modernity eliminates seasonality. [ResearchGate](https://www.researchgate.net/publication/391672553_World_Fertility_2024) |

### Regional coverage is uneven, not merely European

The comparative wealth literature includes populations from Africa, the Americas, Asia and the Pacific. It is useful precisely because it examines different forms of wealth and subsistence rather than assuming one European institutional trajectory. Its sample nevertheless cannot represent every society within each category. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

Archaeological inequality research finds differences between the sampled trajectories of Eurasia and North America/Mesoamerica, but uses house-size inequality as a proxy. Africa, South America, South Asia and Oceania were underrepresented in that influential comparison. House-area Gini must therefore remain distinct from monetary wealth Gini. [Nature](https://www.nature.com/articles/nature24646)

The seasonal-price study covers Burkina Faso, Ethiopia, Ghana, Malawi, Niger, Tanzania and Uganda. This is valuable non-European evidence, but it is neither an estimate for all Africa nor direct measurement of ancient markets. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5384441/)

**Recommendation:** select reference populations through observable features—storage technology, property rights, transport connectivity, mortality, labor organization and household structure—not simply the displayed regime name or technological “era.”

---

## 4. Stylized facts the dashboard should actually check

### 4.1 Respect the size of the simulated population

**Fifty thousand people are not fifty thousand cities or firms.**

For illustration, assuming five people per household gives TCE approximately **2,000–10,000 households**. Their top 1% contains only **20–100 households**. Wealth-tail estimation is already uncertain; city-tail estimation may be effectively impossible if the world contains only a few substantial settlements.

Likewise, a hypothetical homicide rate of **2 per 100,000 person-years** implies only **one expected homicide annually** in a 50,000-person world. Zero incidents in one year is not compelling evidence of underproduction. Under a simple Poisson example, the standard deviation around an annual expectation of one is also one.

Recommended engineering safeguards—not empirical laws:

| Data condition | Proposed dashboard behavior |
| --- | --- |
| Fewer than **50 observations above the fitted tail cutoff** | Do not grade the exponent. Show descriptive ranks and concentration instead. |
| **50–199 tail observations** | Exploratory fit with prominent uncertainty. |
| At least **200 tail observations** | Permit a distributional compatibility check, still conditional on fit quality and dependence. |
| Rare events | Pool exposure over multiple years; show counts, person-years and uncertainty. |
| The same entities observed repeatedly | Do not count repeated snapshots as independent additions to cross-sectional sample size. |

A small world is not a statistically interchangeable miniature of a nation. In particular, preserving individual agents while shrinking population changes the possibilities for rare events and disease persistence.

### 4.2 Fit distributions properly

Do not use a straight-looking log–log plot or a high regression \(R^2\) as the acceptance test.

Estimate the tail cutoff, use an appropriate likelihood, assess goodness of fit, and compare alternatives such as lognormal and truncated power-law distributions. Clauset, Shalizi and Newman provide the standard methodological starting point. [SIAM](https://epubs.siam.org/doi/10.1137/070710111)

For a continuous Pareto tail with a fixed cutoff:

\[
\hat\alpha=\frac{n}{\sum\_i \ln(x\_i/x\_{\min})}.
\]

City populations and employee counts are discrete, so use an appropriate discrete likelihood or justify a continuous approximation. Bootstrap recorded observations when useful; that is statistical analysis of an existing world, not a campaign of additional simulated worlds.

Also inspect **maximum-size truncation**. A world with a finite population cannot support arbitrarily large cities or employers. An unbounded mathematical distribution is only an approximation to part of its observed range.

### 4.3 Check joint signatures, not isolated targets

The following are proposed diagnostic questions. Their interpretation depends on the mechanisms currently present.

| System | Useful observed signature | Investigate when… |
| --- | --- | --- |
| **Settlements** | Size heterogeneity accompanies differences in access, productivity, political functions and migration. | Identical settlements diverge for no recorded reason, or migration ignores modeled livelihood differences. |
| **Wealth** | Concentration, mobility and inheritance can be traced to income, ownership, losses and transfers. | Assets appear without transactions or production; concentration is maintained by an invisible balancing rule. |
| **Firms** | Entry, exit and size variation emerge from production and demand conditions. | All organizations remain identical despite heterogeneous histories, or firm growth is independent of resources and customers. |
| **Engel behavior** | Food shares and food elasticities respond to household resources after controlling for prices and composition. | Wealthier households consume proportionately unlimited staple calories, or own-produced food disappears from the accounts. |
| **Seasonality** | Stock accumulation and depletion correspond to harvests, trade and consumption. | Inventories cannot reconcile, or price changes occur without any modeled supply, demand or institutional explanation. |
| **Fertility** | Births reconcile with exposed populations, age schedules and spacing; surviving children reconcile with mortality. | A single national income coefficient overrides individual circumstances, or period TFR is confused with completed family size. |
| **Crime** | Actual offenses, reporting and enforcement are distinguishable. | The crime count changes solely because a poverty statistic crossed a scripted threshold. |
| **Disease** | Introductions, local transmission, susceptible depletion and fade-out are explicit. | New infections appear after all local infection sources vanish without a modeled introduction or reservoir. |

A price ceiling, for example, may turn food scarcity into queues and unmet demand rather than higher recorded prices. The correct diagnostic should inspect availability as well as price.

### 4.4 Use analytical disease cases as implementation tests

For a homogeneous, closed SIR model with an initially almost fully susceptible population and fixed transmission conditions:

\[
R\_{\mathrm{eff}}\approx R\_0S/N.
\]

The peak in **infectious prevalence** occurs near \(S/N=1/R\_0\). This is not generally the same moment as peak incidence.

Under the same idealized assumptions, the final infected fraction \(z\) satisfies:

\[
z=1-e^{-R\_0z}.
\]

For \(R\_0=2\), the large-outbreak solution is approximately **0.797**, whereas the susceptible-depletion threshold is **0.5**. The distinction catches a common implementation error: transmission does not instantly stop when the threshold is crossed.

These are mathematical checks for a simplified configuration, not universal shapes that every simulated epidemic must follow. Contact networks, seasonality, migration, immunity changes and stochastic fade-out change the result.

For a measles-like acute immunizing infection, TCE’s population ceiling lies well below the historical persistence benchmarks. Long uninterrupted endemic transmission therefore deserves scrutiny unless connectivity and reintroduction explain it. [Science](https://www.science.org/doi/10.1126/science.275.5296.65)

### 4.5 Use four dashboard states

I recommend **red: inconsistent**, **amber: measurable mismatch**, **gray: inapplicable or insufficient evidence**, and **green: compatible**.

“Compatible” is deliberately weaker than “validated.”

Every metric should retain its definition, units, population, denominator, observation window, applicability conditions, reference date, interval type, sample size, estimator and explanation. Avoid a single “realism percentage,” and avoid repeatedly treating \(p<0.05\) on overlapping monthly windows as independent evidence of failure.

---

## 5. Modeling recommendation for TCE

### Keep causal detail where the benchmarks depend on it

The minimum useful representation is not maximum microscopic detail. It is enough detail to preserve the relevant causal distinctions.

| Component | Retain explicitly | Safe initial simplification |
| --- | --- | --- |
| **People and households** | Age, household membership, consumption, health, productive activity, dependents and reproduction. | Shared household purchasing decisions rather than fully separate shopping optimization for every member. |
| **Ownership and institutions** | Asset ownership or use rights, inheritance, transfers, taxation and debt obligations. | A limited authored set of institutional rules, provided their consequences are actually executed. |
| **Production** | Inputs, outputs, employment or labor allocation, capacity, inventories and entry/exit. | Recipe-based production rather than detailed engineering of every operation. |
| **Markets and storage** | Commodity quantities, local prices, transport costs, losses and unmet demand. | Periodic clearing and aggregated freight movements outside the visible scene. |
| **Disease** | Individual states, contact environments and introductions. | Sampled contacts within household/workplace/market groups rather than every possible pair. |
| **Crime** | Incident type, participants, consequences, detection and enforcement. | Event-level resolution rather than minute-by-minute criminal planning. |

For wealth accounting, avoid counting a firm’s physical capital and its owners’ corresponding financial claims twice in an aggregate asset total. For money and debt, reconcile issuance, repayment and default rather than assuming the money stock must always be conserved.

### Suggested measurement cadence

These are engineering defaults to adjust after profiling:

| Cadence | Metrics |
| --- | --- |
| **Daily** | Demographic events, infection states, deaths, inventory flows and logical invariants. |
| **Monthly** | Commodity prices, consumption, food shares, employment and crime exposure. |
| **Annual** | Household wealth distributions, organization sizes, settlement sizes and mobility. |
| **Rolling multiyear windows** | Fertility schedules, rare-event rates, growth regressions and seasonal-price estimates. |

Run expensive statistical fits on snapshots or low-priority analysis tasks, not inside the agent decision loop. The Rust kernel should own measurement truth; camera position, rendering LOD and visible-agent selection must not affect economic or demographic statistics.

For price seasonality, use several complete harvest cycles and fit a parsimonious seasonal component. Short samples and selecting the highest and lowest monthly estimates can exaggerate estimated seasonality. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5384441/)

### Existing models worth borrowing from

| Model | Useful precedent | What not to infer |
| --- | --- | --- |
| **Sugarscape-style models / NetLogo Wealth Distribution** | Resource acquisition, movement, consumption and heterogeneous agent circumstances can produce inequality without directly assigning a target distribution. | A convincing Gini or wealth histogram is not evidence of realistic property institutions or a complete economy. [CCL](https://ccl.northwestern.edu/netlogo/models/WealthDistribution) |
| **Village Ecodynamics Project** | Household settlement and resource decisions connected to archaeological evidence in the U.S. Southwest. Particularly relevant to early agrarian settlement validation. | Its regional reconstruction is not a universal historical parameter set. [CoMSES Net](https://www.comses.net/codebases/2518/releases/1.1.0/) |
| **Covasim** | Individual disease states, structured contact networks, stochastic transmission and explicit interventions. | Modern COVID parameters should not be reused for historical diseases; borrow the architecture. [PLOS](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1009149) |

**My implementation order would be:** accounting and exposure denominators first; household food and storage next; demography and infection fade-out next; wealth mobility and productive organization after that; power-law diagnostics last.

This order makes the dashboard useful before the world contains enough cities or substantial employers to estimate their tails.

---

## 6. Sources, datasets and evidence limitations

The papers cited above provide the numerical reference pack. These data sources provide the strongest foundations for extending it.

| Domain | Source or dataset | Best use and main limitation |
| --- | --- | --- |
| **Urban systems** | European Commission/JRC **Global Human Settlement Layer Urban Centre Database**, plus national settlement censuses. | Harmonized modern spatial comparisons. Urban-centre definitions and population thresholds must be matched; modern “urban centre” categories cannot simply be applied to TCE villages. [Global Human Settlement](https://human-settlement.emergency.copernicus.eu/ghs_ucdb_2024.php?utm_source=chatgpt.com) |
| **Firms** | U.S. Census **Statistics of U.S. Businesses**. | Employer firms and establishments by size and industry. Excludes nonemployer businesses; published bins require suitable grouped-data methods. [Census.gov](https://www.census.gov/programs-surveys/susb.html?utm_source=chatgpt.com) |
| **Wealth** | U.S. **Survey of Consumer Finances**, European **Household Finance and Consumption Survey**, and **World Inequality Database**. | Modern wealth composition and concentration. Top-tail coverage and harmonization require care; aggregate shares alone do not identify an individual-level tail distribution. [European Central Bank](https://www.ecb.europa.eu/pub/pdf/scpwps/ecbwp1692.pdf) |
| **Household consumption and agriculture** | World Bank **LSMS**, especially **LSMS-ISA**. | Household production, own-consumption, assets and welfare in agrarian settings, including African panel surveys. Access and variable comparability differ by survey. [World Bank](https://www.worldbank.org/en/programs/lsms/initiatives/lsms-ISA?utm_source=chatgpt.com) |
| **Food demand** | USDA **International Food Consumption Patterns** and the 2005 ICP-based technical bulletin. | Cross-country demand-system benchmarks. Preserve category definitions and the historical data year. [Economic Research Service](https://www.ers.usda.gov/data-products/international-food-consumption-patterns) |
| **Demography** | UN **World Fertility Data** and the estimates underlying **World Fertility 2024**. | Age-specific and aggregate fertility comparisons; distinguish estimated schedules, projections and observed registrations. [United Nations](https://www.un.org/development/desa/pd/world-fertility-data) |
| **Crime** | UNODC **Global Study on Homicide** and associated statistics. | Contemporary lethal-violence comparisons, with attention to coverage and classification. These do not supply a universal ancient crime rate. [United Nations Office on Drugs and Crime](https://www.unodc.org/unodc/en/data-and-analysis/global-study-on-homicide.html?utm_source=chatgpt.com) |
| **Epidemics** | Disease-specific outbreak studies and the supplementary estimates in Biggerstaff et al.’s influenza review. | Reference distributions for transmission estimates, with setting and estimation assumptions retained. [Springer](https://link.springer.com/article/10.1186/1471-2334-14-480) |

### The most important evidence cautions

**Exact exponents are less robust than broad heterogeneity.** The evidence supports highly unequal sizes in many modern city and firm systems, not a universal requirement that every historical system have exponent one.

**Historical proxies are not interchangeable.** House sizes, livestock, political influence, monetary net worth and surviving offspring measure different things. A shared label such as “wealth” or “success” does not make their coefficients directly comparable.

**Macro associations are not individual decision rules.** A cross-country fertility relationship or state-level unemployment–crime estimate cannot be copied directly into each agent’s utility function. Experimental work from Liberia on crime reduction further illustrates the importance of the particular intervention and behavioral mechanism, rather than one undifferentiated economic deprivation channel. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faeri.20220427)

**Foragers and early farming societies remain the thinnest numerical domain.** The available literature provides useful comparative inequality measures, archaeological demographic evidence and regional reconstructions, but not a defensible universal bundle of city-tail exponents, firm-size distributions and annual crime rates. Those gaps should remain visible.

## Bottom line

**The strongest sanity dashboard for TCE is an explanatory accounting system with conditional empirical checks attached.**

It should establish that households consume and reproduce coherently, stocks explain scarcity, institutions explain wealth transmission, organizations have operational reasons to grow or fail, and infections spread or disappear through actual transmission processes. It should then compare measurable outcomes with appropriately matched evidence.

A world should never be forced toward Zipf, Pareto, low fertility or a predetermined crime rate merely to make the dashboard green.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab929a1-aa98-83e9-9413-7e2926816aee)
