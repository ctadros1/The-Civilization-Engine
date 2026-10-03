# Wealth and income distribution dynamics

## A simulation-ready report for The Civilization Engine

**TCE should generate inequality from ownership, household budgets, inheritance, market opportunities, and enforceable institutions—not assign a Gini coefficient to an era.** The same productive technology can support relatively equal smallholders, concentrated landlordism, or communal ownership. Likewise, a catastrophe can reduce inequality, increase it, or merely impoverish everyone. Historical evidence supports conditional mechanisms rather than a universal progression from equality to inequality and back. [Iris](https://iris.unibocconi.it/handle/11565/4072836)

The most important distinction for the sanity dashboard is between **inequality of current living standards, inequality of accumulated property, and inequality of control over resources**. A society can be unequal in one dimension and comparatively equal in another.

---

## 1. Measurement and the inequality possibility frontier

### 1.1 Do not build one universal “inequality” statistic

For TCE, maintain the following separate measures:

| Measure | Recommended simulation definition | Main use |
| --- | --- | --- |
| **Market income** | Wages, self-employment income, rent, interest, and distributed profits, including appropriately valued production consumed by its producers | Distribution before public redistribution |
| **Disposable resources** | Current income plus transfers, less current taxes and compulsory payments | Household purchasing power and subsistence access |
| **Consumption** | Food, fuel, clothing, housing services, and other goods actually consumed, including own production and communal provision | Living standards, especially in nonmonetary economies |
| **Net private wealth** | Privately held assets and financial claims, less liabilities | Accumulation, inheritance, and economic security |
| **Resource access and control** | Land-use rights, access to commons, command over institutional assets, and legally enforceable claims on labor or output | Power that private-wealth accounts miss |
| **Archaeological proxies** | Residential floor area, storage capacity, or other separately measured physical attributes | Comparison with archaeological evidence |

Modern disposable-income statistics deduct household income taxes and social contributions and include public cash transfers. They are not interchangeable with pretax national-income estimates. Archaeological house-size Ginis measure yet another object: a proxy influenced by architecture, household composition, and sampling. [OECD](https://www.oecd.org/en/data/indicators/income-inequality.html)

**Recommended accounting conventions:**

Use households as consumption-sharing units, then calculate person-weighted inequality using per-capita or explicitly defined adult-equivalent resources. Do not give every child zero income and interpret the resulting individual-earner Gini as living-standard inequality.

For communal or temple property, keep institutional ownership separate from household access. A household with secure grazing rights is not economically equivalent to a landless household excluded from grazing, even when neither owns saleable land.

For net wealth, retain negative observations. Mathematically, the conventional Gini can exceed one when some observations are negative; with a nonpositive mean it becomes undefined or difficult to interpret. Report debt incidence and gross-asset concentration alongside it rather than silently clamping the result.

### 1.2 Milanovic’s inequality possibility frontier

Milanovic, Lindert, and Williamson ask: **how unequal could an economy be while everyone still receives subsistence?** Their inequality possibility frontier, or IPF, distinguishes absolute inequality from inequality relative to the surplus available for appropriation. [CEPR](https://cepr.org/voxeu/columns/measuring-ancient-inequality)

Let:

* \(N\) be the population;
* \(\mu\) be mean annual income per person;
* \(s\) be a common annual subsistence requirement, with \(\mu\geq s\).

Give \(N-1\) people exactly \(s\), and allocate all remaining income to one person. The resulting finite-population maximum is:

\[
G\_{\max}=\frac{N-1}{N}\left(1-\frac{s}{\mu}\right)
\]

For TCE-sized populations, the approximation is effectively:

\[
G\_{\max}\approx 1-\frac{s}{\mu}
\]

The following are **calculated examples**, not historical estimates:

| Mean income relative to subsistence, \(\mu/s\) | 1.25 | 1.5 | 2 | 3 | 5 |
| --- | --- | --- | --- | --- | --- |
| Approximate maximum income Gini | 0.20 | 0.33 | 0.50 | 0.67 | 0.80 |

Define the **inequality extraction ratio**:

\[
E=\frac{G\_{\text{income}}}{G\_{\max}}
\]

An income Gini of 0.40 implies \(E=0.80\) when mean income is twice subsistence, but only \(E=0.50\) when mean income is five times subsistence.

Thus, **a poor society can exhibit modest measured inequality while leaving its majority almost nothing above survival**. The extraction ratio is not a literal tax rate or, in general, the fraction of surplus received by elites. [CEPR](https://cepr.org/voxeu/columns/measuring-ancient-inequality)

**Implementation recommendation:** compute \(s\) from a local survival basket, not a universal monetary constant. Keep household consumption needs distinct from seed, fodder, and replacement inputs needed to sustain production.

Treat the IPF as a diagnostic, not an enforced ceiling. An apparent violation can indicate below-subsistence consumption, asset depletion, inconsistent population units, an incorrect basket, or incompatible income estimates. Do not apply this income frontier to wealth Ginis.

---

## 2. Mechanisms as implementable rules

### 2.1 Accumulation requires assets that survive, matter, and can be controlled

**Mechanism.** Inequality becomes more persistent when economically important resources are durable, transferable, and defensible. A temporary hunting windfall differs from a herd, irrigated field, urban rental building, or ownership share that repeatedly generates income.

Recent global archaeological research finds greater inequality under **land-limited** production than under **free, labor-limited** production, while governance modifies the relationship. The relevant transition is not simply “agriculture discovered,” but a change in which scarce resources constrain production and who can control them. [Iris](https://iris.unibocconi.it/handle/11565/4072836)

**TCE rules.** Give each asset type:

`durability`, `productive_service`, `transferability`, `divisibility`, `excludability`, `maintenance_cost`, and `enforcement_requirement`.

Accumulation should become easier when assets survive between harvests, generate further output, and remain under their holders’ control. Storage capacity alone should not automatically create an aristocracy: access rules, spoilage, theft, obligations to share, and political enforcement must intervene.

### 2.2 Land concentration emerges through transactions and coercion

**Mechanism.** Landowners gain bargaining power when households cannot obtain independent livelihoods elsewhere. Conversely, accessible land can improve workers’ outside options. But land abundance does not guarantee equality: elites may restrict settlement, mobility, or independent cultivation. Domar’s influential hypothesis treats coercive labor institutions as part of this land–labor relationship, not as an automatic consequence of population density. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/causes-of-slavery-or-serfdom-a-hypothesis/B6055D4D909C9D21B79BB42D2CD952C5)

**TCE rules.**

Calculate a household’s realistic outside option: available land quality, tools, settlement rights, travel costs, security, and the labor needed to establish a farm. Do not use geographic vacancy alone.

Allow concentration through purchases, distress sales, foreclosure, conquest, confiscation, and conversion of customary rights into exclusive titles. Model the legal changes separately from their enforcement.

Distinguish **ownership concentration** from **operational concentration**. A large estate rented to many tenants and a large centrally operated farm should not have identical employment, investment, or political effects.

### 2.3 Surplus, saving, and returns create cumulative advantage

**Mechanism.** Households near subsistence have little capacity to absorb losses or acquire productive assets. Households with surpluses can invest, diversify, and wait rather than sell under pressure. Administrative evidence from Norway also demonstrates substantial, persistent differences in returns across individuals—not merely different quantities of wealth. [Stanford University](https://web.stanford.edu/~pista/FGMP.pdf)

**TCE rules.**

Households first meet consumption and obligatory payments, then rebuild reserves, repay or service debt, and consider investment. Investment must require an available asset or project; money should not grow merely because it sits in a “capital” account.

Returns should arise from actual rents, profits, interest payments, and asset-price changes. Correlate risks appropriately: drought affects neighboring farms; a merchant’s shipwreck is more idiosyncratic; war affects particular routes and asset classes.

**Do not implement “\(r>g\) ⇒ increase inequality.”** Piketty explicitly rejects that as a complete account of inequality dynamics. Saving behavior, institutions, shocks, and unequal ownership mediate the relationship. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.29.1.67)

A simple mathematical check illustrates why: multiplying every household’s wealth by the same factor leaves wealth shares unchanged. Also, a 5% return with only 20% reinvested produces 1% accumulation from that return, before other changes—not 5%.

### 2.4 Inheritance preserves shocks; partition and turnover dissipate them

**Mechanism.** Inheritance transmits not only objects but also opportunities and claims. Wealth-distribution models with finite lives show how bequests and heterogeneous returns can sustain concentrated wealth across generations. [Gabriel Zucman | Professor of economics](https://gabriel-zucman.eu/files/teaching/BenhabibEtal11.pdf)

**TCE rules.**

At death, settle the estate under the applicable debt and succession rules, then allocate assets or beneficial shares to eligible heirs. Support partible inheritance, preferential heirs, spouse entitlements, institutional succession, and restrictions on alienation.

Keep **physical indivisibility** separate from **ownership divisibility**: four heirs can share an estate without physically quartering every building.

Model education, apprenticeship access, connections, and assortative marriage through observable opportunities and relationships. Do not encode inherited economic success as an immutable biological “wealth talent.”

### 2.5 Credit can finance mobility—or compound vulnerability

**Mechanism.** Borrowing constraints, entrepreneurial opportunities, and wealth-dependent investment interact in models capable of generating strongly skewed wealth distributions. The important distinction is whether a household can finance productive activity and survive adverse outcomes, not whether it possesses a generic “entrepreneur” label. [Federal Reserve Bank of New York](https://www.newyorkfed.org/medialibrary/media/research/conference/2016/woodford/benhabib_jess_1_paper)

**TCE rules.**

Lenders evaluate collateral, expected repayment, enforcement costs, and existing exposure. Borrowers can invest successfully, default, lose pledged assets, negotiate, or obtain communal support.

An adverse harvest should sometimes trigger a sequence such as:

\[
\text{reserve depletion}
\rightarrow \text{borrowing}
\rightarrow \text{distress sale}
\rightarrow \text{loss of productive capacity}
\]

That sequence is a proposed causal implementation, not an instruction that every poor borrower must become poorer. Productive credit and recovery must remain possible.

### 2.6 Institutions alter both initial access and subsequent returns

**Mechanism.** Taxation, inheritance law, education, bargaining institutions, and property rules change the distributional consequences of growth. Institutional interaction is central to the historical interpretation of inequality; technology alone does not determine a unique distribution. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.29.1.67)

**TCE rules.**

Let political power affect specific decisions: exemptions, enclosure, monopoly privileges, debt enforcement, public expenditure, and access to office. Give other groups organizational capacities that can resist or reverse those changes.

Separate statutory policy from implementation:

\[
\text{effective redistribution}
=
\text{assessed obligations}
\times \text{collection success}
-\text{administration and diversion}
\]

Transfers then require an allocation rule and actual delivery. A progressive tax schedule with widespread elite exemptions should not behave like an effectively progressive system.

### 2.7 Peaceful equalization must be possible

Latin America provides an important counterexample to the proposition that only catastrophes reduce inequality. Research on the 2000s identifies narrowing skill premiums and more progressive transfers as major contributors, with minimum wages, unionization, and changing labor supply and demand playing different roles across countries. [Center For Global Development](https://www.cgdev.org/publication/declining-inequality-latin-america-2000s-cases-argentina-brazil-and-mexico-working-paper)

**TCE rules.** Broader access to useful skills can reduce scarcity rents; collective bargaining can alter wage setting; land redistribution can broaden productive ownership; transfers can raise disposable resources. Their effectiveness should depend on financing, administrative capacity, labor demand, and implementation—not an automatic reform bonus.

### 2.8 War, plague, revolution, and collapse are different shocks

Scheidel’s *The Great Leveler* emphasizes four categories: **mass-mobilization warfare, transformative revolution, state collapse, and catastrophic pandemics**. This is a historical thesis about major leveling episodes, not evidence that every conflict or epidemic reduces inequality. [Stanford Classics](https://classics.stanford.edu/publications/great-leveler-violence-and-history-inequality-stone-age-twenty-first-century)

| Shock | Potential equalizing channels | Potential unequalizing channels | What TCE should change |
| --- | --- | --- | --- |
| **Epidemic** | Labor scarcity; improved bargaining positions; more resources per survivor | Inheritance consolidation; elite protection; coercive restrictions; unequal mortality | People, household composition, labor supply, succession events, and institutional responses |
| **War** | Destruction of concentrated assets; extraordinary taxation; inflation; political reform | Loot concentration, enslavement, displacement, debt, privileged supply contracts | Specific assets, people, claims, prices, taxes, and territorial rights |
| **Revolution** | Expropriation and redistribution; abolition of privileges | Replacement elites; selective confiscation; exclusion from redistributed resources | Ownership and authority graphs, eligibility, enforcement, and organizational control |
| **State collapse** | Disappearance of tribute claims and enforceable privileges | Loss of security and public goods; predation; local monopolies of force | Enforcement capacity and public-service provision, not just treasury balances |

The epidemic comparison is especially instructive. Alfani’s recent synthesis reports that northern Italy’s 1629–1630 plague killed roughly **35%** of the population but did not produce a Black-Death-like sustained equalization. Institutions protecting large fortunes mattered. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.70122)

**Never encode catastrophe as `wealth_gini -= amount`.** Model the losses and transfers, then measure the result. Always show deaths, consumption, and security beside inequality: equalization through devastation is not improved welfare.

---

## 3. Quantitative benchmarks, parameters, and variation

### Confidence notation

**H:** strong measurement within the stated sample or a mathematical identity.  
**M:** informative estimate with meaningful measurement or identification limitations.  
**L:** fragile reconstruction or limited transferability.  
**P:** proposed simulation sensitivity range, not a historical estimate.

Confidence below concerns the stated use, not a formal statistical confidence interval.

### 3.1 Foragers, horticulturalists, pastoralists, and farmers

Borgerhoff Mulder and colleagues studied **21 small-scale populations**. Their summary Ginis are weighted averages across embodied, relational, and material wealth measures. **They are not Ginis of monetized net worth, and these populations are not direct observations of prehistoric societies.** [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

| Economic system in the study | Weighted average Gini, ± standard error | Weighted intergenerational persistence \(\beta\), ± standard error | Confidence for TCE |
| --- | --- | --- | --- |
| Hunter-gatherer | 0.25 ± 0.04 | 0.19 ± 0.05 | M for study; L for prehistoric calibration |
| Horticultural | 0.27 ± 0.03 | 0.18 ± 0.04 | M / L |
| Pastoral | 0.42 ± 0.05 | 0.43 ± 0.06 | M |
| Agricultural | 0.48 ± 0.04 | 0.36 ± 0.05 | M |

For **material wealth specifically**, estimated persistence was **0.67 ± 0.07 among pastoralists** and **0.55 ± 0.07 among agriculturalists**. These are transmission coefficients, **not fractions of estates inherited**. Use them as output diagnostics after simulating succession and opportunities, not as direct bequest parameters. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

The practical implication is to preserve distinct wealth dimensions. A skilled but property-poor individual and an unskilled heir should not collapse into the same scalar “wealth level.”

### 3.2 Early farming: substantial variation, not an immediate inequality jump

Kohler and colleagues’ 2017 archaeological comparison found greater post-Neolithic increases in house-size inequality in Eurasia than in North America and Mesoamerica, proposing roles for traction animals and mounted elites. Those explanations concern mechanisms and sampled trajectories, not inherent continental characteristics. [Nature](https://www.nature.com/articles/nature24646)

The newer global land–labor research emphasizes production constraints and governance. Together, these findings suggest that TCE should permit long-lived relatively equal farming communities, unequal pastoral societies, and divergent urban trajectories without assigning a continent-specific Gini modifier. [Iris](https://iris.unibocconi.it/handle/11565/4072836)

### 3.3 Reconstructed historical income Ginis

The following values come from the **2007 working-paper version** of Milanovic, Lindert, and Williamson’s research. Where shown, ranges reproduce its **Gini1–Gini2 construction**, not statistical confidence intervals. Social-table assumptions and missing within-class variation create uncertainty beyond these ranges. [Munich Personal RePEc Archive](https://mpra.ub.uni-muenchen.de/5388/1/MPRA_paper_5388.pdf)

| Society and date | Reconstructed income Gini | Confidence |
| --- | --- | --- |
| Roman Empire, 14 CE | 0.364–0.394 | L–M |
| Byzantium, 1000 | 0.410–0.411 | L |
| England and Wales, 1688 | 0.449–0.450 | M |
| Mughal India, 1750 | 0.385–0.489 | L |
| Old Castile, 1752 | 0.523–0.525 | M |
| Nueva España, approximately 1790 | 0.635 | L |
| Brazil, 1872 | 0.387–0.433 | L–M |
| China, 1880 | 0.239–0.245 | L |
| British India, 1947 | 0.482–0.497 | L–M |

Source: the working paper’s Table 2. Decimal precision is retained for traceability, not because these societies’ inequality is known to three decimal places. [Munich Personal RePEc Archive](https://mpra.ub.uni-muenchen.de/5388/1/MPRA_paper_5388.pdf)

**Calibration implication:** a single “preindustrial income Gini = 0.45” default would conceal economically important variation. Select a reconstruction only after matching its population coverage, resource concept, and institutional setting.

### 3.4 Historical wealth and industrial transitions

| Case | Quantitative result | Interpretation and confidence |
| --- | --- | --- |
| German area, approximately 1600–1700 | Wealth Gini just below **0.68 → 0.62 → 0.59**, around 1600, 1650, and 1700 | Reconstructed leveling associated with war and epidemic; M for broad movement |
| Italy, approximately 1700 | Wealth Gini around **0.80** | Comparable reconstruction illustrates regional divergence; M |
| Japan, 1938–1945 | Top 1% income share approximately **20% → 6.4%** | Very large wartime compression; M–H for direction and magnitude |
| Brazil, 1998–2009 | Household per-capita income Gini **0.592 → 0.537** | Substantial non-catastrophic decline; M–H |

The German and Italian figures use analogous historical sources and estimation methods. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.70122) Japan’s income decline was concentrated during the war; subsequent reforms helped prevent reconcentration and affected wealth ownership, rather than simply causing the entire initial income decline. [WID - World Inequality Database](https://wid.world/document/moriguchi-chiaki-and-saez-emmanuel-2010-the-evolution-of-income-concentration-in-japan-1886-2005-in-atkinson-a-b-and-piketty-t-editors-top-incomes-a-global-perspective-oxford/) The Brazilian figures come from the Latin American inequality research discussed above. [Center For Global Development](https://www.cgdev.org/sites/default/files/1426568_file_Lustig_et_al_IneqLA_FINAL.pdf)

These episodes reject a mechanical Kuznets curve: industrialization and growth do not automatically produce one predictable rise-and-fall sequence.

### 3.5 Modern regional comparison

The **World Inequality Report 2026** supplies the following **2025 estimates**. These are regional distributions, not averages of national Ginis, and its income measures must not be confused with disposable-income survey measures. [World Inequality Report 2026](https://wir2026.wid.world/methodology/)

| Region | Top 10% income share | Bottom 50% income share | Confidence |
| --- | --- | --- | --- |
| Europe | 36% | 19% | M |
| East Asia | About 46% | 13% | M |
| North America and Oceania | About 46% | 13% | M |
| Latin America, Sub-Saharan Africa, and Middle East/North Africa | Approximately 55–57% across these regions | Approximately 8–11% | M |

These aggregates conceal substantial within-region variation. [World Inequality Report 2026](https://wir2026.wid.world/insight/regional-income-inequality/)

Wealth is more concentrated: the report estimates that regional top deciles own **60–74% of wealth**, while regional bottom halves own **1–5%**. Globally, the top decile owns approximately **75% of wealth**, versus **53% of income**; the bottom half owns about **2% of wealth**. **The global distribution is not a suitable target for one TCE town**, because it includes differences between countries and regions. [World Inequality Report 2026](https://wir2026.wid.world/insight/regional-wealth-inequality/)

For South Asia, the **2026 published version** of Bharti and colleagues’ India study estimates 2022–2023 top-1% shares of **23.3% of income and 40.1% of wealth**. Its authors explicitly flag data-quality limitations. These are harmonized reconstructions, not direct counts of every household. [OUP Academic](https://academic.oup.com/wber/advance-article/doi/10.1093/wber/lhag013/8675571)

For precolonial Africa, the Pacific, and many early Asian societies, this evidence base does **not** justify equally precise continent-wide income-Gini defaults. Use local archaeological or ethnographic evidence with its original units rather than filling gaps with European values.

### 3.6 Quantitative process anchors

Returns should emerge from TCE’s production and ownership systems. Nevertheless, measured returns are useful checks on whether simulated asset accumulation is implausibly fast or uniform.

Jordà and colleagues’ long-run dataset covers **16 advanced economies, 1870–2015**. The accessible working-paper table reports:

| Asset | Real arithmetic mean return | Real geometric mean return | Annual return standard deviation |
| --- | --- | --- | --- |
| Bills | 0.98%/year | 0.78%/year | 6.01 percentage points |
| Bonds | 2.50%/year | 1.94%/year | 10.74 percentage points |
| Equities | 6.89%/year | 4.64%/year | 21.94 percentage points |
| Housing | 7.05%/year | 6.61%/year | 9.98 percentage points |

Confidence is M for historical aggregate comparisons and L for transfer to early agrarian assets. These are total returns, not guaranteed annual cash yields. Aggregate housing series are also not measures of the risk faced by one household owning one building. Do not apply these rates to grain stores, oxen, or medieval land without separate calibration. [Federal Reserve Bank of San Francisco](https://www.frbsf.org/wp-content/uploads/wp2017-25.pdf)

A complementary Norwegian administrative-data study summarizes persistent **after-tax net-worth return effects** with a standard deviation of approximately **3.6 percentage points** and an intergenerational persistence coefficient around **0.14**. This is evidence against giving every owner the same return—and against making a successful return advantage perfectly hereditary. [Stanford University](https://web.stanford.edu/~pista/FGMP.pdf)

### 3.7 Proposed sensitivity parameters—not historical constants

These are starting experiments for TCE. They should not appear in the historical-data table as measured values.

| Parameter | Initial sensitivity grid | Unit and interpretation | Status |
| --- | --- | --- | --- |
| Desired household reserve | 30, 90, 180 | Days of household subsistence; constrained by storage and affordability | P |
| Sharing obligation | 0, 0.25, 0.50, 0.75 | Fraction of eligible surplus entering a defined sharing pool | P |
| Investment propensity | 0.10, 0.30, 0.60 | Fraction of discretionary resources offered to feasible investments | P |
| Largest heir’s estate share | \(1/n\) through 1 | Fraction allocated to the principal heir, conditional on legal eligibility | P |
| Effective estate levy | 0%, 10%, 25%, 40% | Collected fraction of the taxable estate after exemptions | P |
| Land accessible under communal rights | 0%, 25%, 50%, 100% | Fraction of relevant land; access rules must also be specified | P |

These ranges are deliberately broad. Their purpose is to identify which mechanisms matter and where results become unstable—not to assert that a particular society historically used the midpoint.

---

## 4. Stylized facts and validation targets

A credible simulation should reproduce **families of outcomes under appropriate conditions**, not match every historical number in every seed.

| Pattern to reproduce | Evidence anchor | TCE validation |
| --- | --- | --- |
| **Wealth can be much more concentrated than income** | Modern regional top-decile shares: 60–74% of wealth versus 36–57% of income. [World Inequality Report 2026](https://wir2026.wid.world/insight/regional-wealth-inequality/) | Confirm that labor earnings and transfers can support households with little property |
| **Similar technologies can coexist with very different inequality** | Land constraints interact with governance; historical German and Italian wealth trajectories diverged. [Iris](https://iris.unibocconi.it/handle/11565/4072836) | Hold technology fixed and vary property, access, and enforcement institutions |
| **Persistence depends on what can be transmitted** | Small-scale material-wealth persistence differs markedly across economic systems. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf) | Measure parent–child outcomes by wealth type, not just total wealth |
| **Major compression can happen rapidly and then persist** | Japan’s top-1% income share fell from about 20% to 6.4% during 1938–1945. [WID - World Inequality Database](https://wid.world/document/moriguchi-chiaki-and-saez-emmanuel-2010-the-evolution-of-income-concentration-in-japan-1886-2005-in-atkinson-a-b-and-piketty-t-editors-top-incomes-a-global-perspective-oxford/) | Allow destruction and institutional change to have different immediate and long-run effects |
| **Epidemics need not level wealth** | The seventeenth-century Italian experience differs from the Black Death. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ehr.70122) | Repeat the same mortality shock under different succession and labor-control rules |
| **Peaceful policy and labor-market changes can reduce inequality** | Brazilian and broader Latin American declines in the 2000s. [Center For Global Development](https://www.cgdev.org/sites/default/files/1426568_file_Lustig_et_al_IneqLA_FINAL.pdf) | Produce lower disposable-income inequality without requiring war, famine, or confiscation |
| **Upper tails are important, but not immutable** | Empirical tail thickness varies; stochastic accumulation models explain several possible distributions. [Academia](https://www.academia.edu/534154/The_Forbes_400_and_the_Pareto_wealth_distribution) | Track top shares and mobility alongside the Gini |

Add two **hard mathematical checks**, rather than empirical targets:

First, identical proportional growth of every wealth holding must leave wealth shares unchanged. Second, a pure transfer between domestic balance sheets must not create aggregate real assets.

---

## 5. Modeling recommendation for individual agents and institutions

### 5.1 Use a balance-sheet core with persistent ownership

The minimum useful representation is:

| Entity | Distribution-relevant state |
| --- | --- |
| **Person** | Household, age, kinship, skills, legal eligibility, employment, ownership shares |
| **Household** | Members, consumption obligations, inventories, cash, debts, access rights, reserve policy |
| **Asset or parcel** | Quantity, quality, location, owner, user, condition, encumbrances, rights regime |
| **Firm or institution** | Assets, liabilities, beneficiaries, control rights, retained earnings, distribution rules |
| **Contract** | Parties, payment obligations, collateral, maturity, seniority, enforcement jurisdiction |
| **Law or custom** | Succession, alienability, taxation, exemptions, commons access, creditor remedies, labor mobility |

For household wealth accounting, use a reconciliation identity such as:

\[
W\_{h,t+1}-W\_{h,t}
=
Y^{disp}\_{h,t}-C\_{h,t}
+K\_{h,t}+H\_{h,t}-L\_{h,t}
\]

Here \(K\) denotes net capital transfers such as bequests; \(H\) denotes holding gains or losses; and \(L\) denotes extraordinary asset losses not already recorded elsewhere.

Choose consistent treatments of depreciation and retained firm earnings. **Do not count firm assets once on the firm’s balance sheet and again as additional aggregate wealth through owners’ equity.** Household wealth can include equity, but consolidated economy-wide accounting must eliminate duplication.

Similarly, a loan creates a creditor’s claim and a debtor’s liability. A land-price increase can raise measured private wealth without increasing hectares, food output, or housing services.

### 5.2 Let observed distributions emerge; use fitted distributions only diagnostically

A useful upper-tail approximation is:

\[
\Pr(W>w\mid W\geq w\_{\min})
=
\left(\frac{w\_{\min}}{w}\right)^\alpha
\]

A smaller \(\alpha\) means a heavier upper tail. The probability-density exponent is \(\alpha+1\), so store the convention explicitly.

Klass and colleagues estimated a pooled exponent of approximately **1.49** for normalized Forbes 400 wealth over **1988–2003**, with substantial year-to-year variation. This is a narrow, exceptionally wealthy sample—not a universal exponent for farmers, towns, or all household wealth. [Academia](https://www.academia.edu/534154/The_Forbes_400_and_the_Pareto_wealth_distribution)

For an entirely Pareto-distributed positive population,

\[
G=\frac{1}{2\alpha-1},\qquad \alpha>1
\]

But **do not convert a fitted upper-tail exponent into the Gini of the whole society**. The lower distribution, zero holdings, debts, and the fraction in the tail all matter.

For testing, an \(\alpha\) grid such as **1.2, 1.5, 2, and 3** is useful as a set of synthetic distributions. It is not a claimed all-history empirical range.

### 5.3 Kesten-type accumulation is the best conceptual bridge

A stylized recurrence is:

\[
W\_{t+1}=A\_tW\_t+B\_t
\]

The multiplicative term represents retained returns and other proportional changes; the additive term represents earnings or other inflows. Under suitable assumptions, including contraction on average but some opportunities for expansion, such processes generate Pareto tails. In a standard independent-coefficient case, the tail exponent satisfies:

\[
\mathbb{E}[A^\alpha]=1
\]

In a growing economy, work with wealth normalized by an appropriate aggregate trend. \(A\) is therefore not simply the gross investment return: consumption, taxation, succession, and normalization may all affect it. [Federal Reserve Bank of New York](https://www.newyorkfed.org/medialibrary/media/research/conference/2016/woodford/benhabib_jess_1_paper)

**Recommendation:** use this literature to understand and diagnose TCE’s emergent accumulation process. Do not replace production, contracts, and inheritance with arbitrary annual wealth multipliers.

### 5.4 Kinetic exchange models are useful controls, not a complete economy

In a basic saving-exchange model:

\[
m\_i'=\lambda\_i m\_i+
\epsilon[(1-\lambda\_i)m\_i+(1-\lambda\_j)m\_j]
\]

The other participant receives the remaining exchanged amount.

Simple random exchange without saving produces an exponential money distribution; common saving propensities produce a narrower, gamma-like distribution; heterogeneous persistent saving propensities can produce a Pareto tail. These are valuable demonstrations of how distributional shape depends on micro-rules. [arXiv](https://arxiv.org/html/0709.1543v2)

However, the conserved quantity is usually **money**, not a complete system of productive wealth. Production, destruction, credit, asset valuation, demography, and legal ownership require additional mechanisms.

Use kinetic models as unit-test environments. The exponential distribution’s theoretical Gini of **0.5** is a mathematical benchmark, not a historical target.

### 5.5 Existing models worth borrowing from

| Model family | Useful component | What not to copy uncritically |
| --- | --- | --- |
| **Sugarscape / NetLogo Wealth Distribution** | Persistent agents, resource acquisition, heterogeneous constraints, Lorenz curves, and Gini monitoring | Resource “sugar” is not a complete model of property, credit, firms, or institutions. [CCL](https://ccl.northwestern.edu/netlogo/models/Sugarscape3WealthDistribution) |
| **Benhabib–Bisin–Zhu finite-life wealth models** | Stochastic returns, inheritance, taxation, and upper-tail mechanisms | Their economic environment is not automatically appropriate for an early agrarian economy. [Gabriel Zucman | Professor of economics](https://gabriel-zucman.eu/files/teaching/BenhabibEtal11.pdf) |
| **Kinetic exchange models** | Small, analytically tractable experiments revealing the effects of exchange and saving rules | Conservation of money must not substitute for conservation and production of real resources. [arXiv](https://arxiv.org/html/0709.1543v2) |

For TCE, the strongest combination is **Sugarscape-style persistent agents, explicit balance sheets and property rights, and accumulation diagnostics informed by heterogeneous-agent wealth theory**.

### 5.6 Performance and dashboard design

The following are engineering recommendations, not measured performance benchmarks.

Run physical production and consumption at the existing simulation cadence. Update contracts, ownership transfers, deaths, and succession through events. Accumulate household income flows continuously, but calculate the main distribution statistics annually or over rolling twelve-month windows.

For equally weighted, nonnegative observations sorted as \(x\_{(1)}\leq \dots\leq x\_{(N)}\):

\[
G=
\frac{2\sum\_{i=1}^{N}i\,x\_{(i)}}{N\sum\_{i=1}^{N}x\_i}
-\frac{N+1}{N}
\]

Sorting gives an \(O(N\log N)\) calculation. Use the appropriate weighted version for person-weighted household observations.

The dashboard should show, at minimum:

**Consumption Gini; disposable-income Gini; gross-asset and net-wealth inequality; top 10% and top 1% shares; landless and indebted shares; subsistence shortfall; extraction ratio; and intergenerational mobility.**

Also retain a decomposition of wealth changes into earned saving, inheritance, transfers, holding gains, and destruction. That makes unexpected inequality explainable.

At 10,000–50,000 people, the top 0.1% comprises only **10–50 people**, and fewer observations after converting to households or eligible adults. Extreme-tail estimates will therefore be noisy. Fit tails only with adequate observations, test alternative cutoffs, and compare multiple seeds; do not claim precision from a straight-looking log–log plot.

### 5.7 Validation should use paired institutional experiments

Use common initial conditions and matched random seeds to compare:

* accessible frontier land versus restricted settlement;
* equal partition versus preferential succession;
* communal insurance versus unbuffered household shocks;
* identical versus heterogeneous investment opportunities;
* nominally progressive taxation versus effective enforcement.

Run long enough for multiple generations. Examine both distribution levels and transitions: who enters the top decile, who remains there, who loses land, and how often families recover.

A stationary Gini with permanently frozen dynasties is a different society from a stationary Gini with substantial mobility. TCE should distinguish them.

---

## 6. Sources, contested claims, and a calibration data plan

### Recommended evidence to retain with the simulation

| Source or dataset | Best use | Principal limitation |
| --- | --- | --- |
| **Milanovic, Lindert & Williamson, *Pre-Industrial Inequality*; accompanying working-paper social tables** | Historical income distributions and the IPF | Sparse classes, reconstructed means, uncertain subsistence assumptions; preserve the exact version. [Stone Center](https://stonecenter.gc.cuny.edu/publications/pre-industrial-inequality-2/) |
| **Borgerhoff Mulder et al., Science 2009** | Small-scale wealth dimensions and intergenerational persistence | Weighted multi-dimensional measures, limited populations, not prehistoric observations. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf) |
| **GINI archaeological project; Kohler et al. 2017; Bogaard et al. 2025** | Housing/storage distributions and mechanisms across early societies | Residential proxies, dating, household identification, and site-selection effects. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/global-dynamics-of-inequality-gini-project-analysing-archaeological-housing-data/CBA474CDCB265DED23906F0A193BF7B3) |
| **World Inequality Database / World Inequality Report** | Long-run top shares, wealth concentration, and distributional national accounts | Harmonization and top-tail reconstruction require assumptions; series are revised. [WID - World Inequality Database](https://wid.world/) |
| **World Bank Poverty and Inequality Platform** | Household income or consumption distributions and poverty | Preserve the welfare concept; its percentile files include several underlying data types. [World Bank Data Catalog](https://datacatalog.worldbank.org/search/dataset/0063646/poverty-and-inequality-platform-pip-percentiles) |
| **UNU-WIDER WIID and WIID Companion** | Broad historical and cross-country inequality coverage | Raw and standardized observations are different products. Record the version; the retrieved release is 8 September 2026. [WIDER](https://www.wider.unu.edu/database/world-income-inequality-database-wiid) |
| **OECD Income Distribution Database** | Disposable-income definitions and modern redistribution comparisons | Not a substitute for ancient income or private-wealth measures. [OECD](https://www.oecd.org/en/data/indicators/income-inequality.html) |
| **Jordà et al.; Fagereng et al.** | Aggregate asset returns and individual return heterogeneity | Modern financial environments and different levels of aggregation. [Federal Reserve Bank of San Francisco](https://www.frbsf.org/wp-content/uploads/wp2017-25.pdf) |

### Claims that should remain explicitly qualified

**“Agriculture caused inequality.”** Too broad. Asset characteristics, land constraints, and institutions are better causal variables than an agriculture flag. Archaeological patterns support investigation of these mechanisms, not one inevitable timetable. [Iris](https://iris.unibocconi.it/handle/11565/4072836)

**“Only violence levels societies.”** Scheidel’s thesis concerns historically powerful leveling episodes. Peaceful reductions are documented, although their scale, durability, and effects on wealth versus income differ. [Stanford Classics](https://classics.stanford.edu/publications/great-leveler-violence-and-history-inequality-stone-age-twenty-first-century)

**“Returns exceeding growth inevitably produce unlimited concentration.”** This omits saving, distribution of ownership, taxation, succession, and shocks. Even Piketty’s own clarification rejects that simplification. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.29.1.67)

**“A Pareto tail proves the mechanism is correct.”** Different models can generate similar tails. Validate accounting, mobility, asset composition, and responses to institutional changes—not only a fitted exponent. [Federal Reserve Bank of New York](https://www.newyorkfed.org/medialibrary/media/research/conference/2016/woodford/benhabib_jess_1_paper)

For each empirical target, store its **resource definition, population unit, geography, date, observation method, uncertainty, and source version**. That metadata is as important as the number.

**Bottom line for TCE:** build inequality around who controls productive resources, who can obtain them, who receives their output, and what survives into the next generation. Use the Gini to summarize those processes—but use ownership histories, living standards, and mobility to determine whether the simulation is believable.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92899-4400-83ea-8119-df6e9873f9a5)
