# Taxation and public finance for The Civilization Engine

**The most important design choice is to separate what a government demands, what people actually surrender, and what reaches a usable public treasury.** These can differ enormously. A population can face severe extraction while its central government remains chronically short of money, food, and reliable officials. Historical fiscal systems therefore should not be represented by a tax-rate slider multiplied by an omniscient estimate of national income. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914)

For TCE, taxation should emerge from **claims over particular people, assets, activities, and communities**, implemented through imperfect records, collectors, intermediaries, transport, and enforcement. Public spending then turns those collections into soldiers, administration, patronage, infrastructure, and relief—not an abstract government-effectiveness bonus.

The historical evidence supports useful calibration anchors, but not a universal “ancient,” “medieval,” or “industrial” tax rate. Below, **historical observations, reconstructions, and proposed simulation settings are explicitly distinguished**.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Maintain distinct fiscal accounts

TCE should record the following separately:

| Account | What it measures | Why the distinction matters |
| --- | --- | --- |
| **Legal assessment** | The obligation imposed under a law, custom, or contract | A land register or population roll can be inaccurate, obsolete, or manipulated. |
| **Actual household burden** | Money, goods, labor, and additional charges actually surrendered | Includes burdens that may never enter public accounts. |
| **Public revenue collected** | Receipts belonging to public institutions at every level | Some revenue is legally retained and spent locally. |
| **Central treasury receipts** | Resources actually delivered to the central authority | Not equivalent to total public revenue or total extraction. |
| **Non-tax finance** | Domain income, monopoly profits, borrowing, asset sales, mint income, tribute from outside the accounting boundary | These have different sustainability and distributional consequences. |

Add separate records for private rent, religious dues, and compulsory service. Whether a temple’s receipts count as public finance should depend on its institutional role, not a permanent “religious/private” classification.

**Consolidation is essential.** A village’s payment to a district and the district’s remittance to the capital are two transfers of the same resources, not two contributions to national tax revenue. Conversely, authorized local expenditure is not automatically corruption.

Keep labor obligations in **person-days**, goods in physical quantities, and cash by currency. A supplementary “resource-equivalent burden” can value labor at an explicit shadow wage, but it should not silently replace cash-revenue statistics. Van Waijenburg’s reconstruction of French African colonial finance finds that including compulsory labor radically changes the apparent size of early colonial budgets. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/financing-the-african-colonial-state-the-revenue-imperative-and-forced-labor/9514245BA11D58B053FBB83E4EADF565)

### 1.2 Represent tax instruments as different claims

The following are recommended rule implementations. Their effects arise from their bases, timing, and enforcement rather than a generic tax-efficiency modifier.

| Instrument | Implementable rule | Characteristic consequences |
| --- | --- | --- |
| **Tribute** | An authority imposes a periodic quota of goods, valuables, labor, or military support on another community or ruler. The subordinate authority determines collection within its jurisdiction. | Local autonomy reduces central administrative requirements but creates bargaining, under-remittance, and secession risks. |
| **Tithe or proportional produce levy** | Claim a fraction of specified production, possibly assessed at harvest or delivery. Specify whether the base is gross output, output after deductions, or an estimated normal harvest. | Revenue shares harvest risk only when the assessment follows actual production. Exemptions and collection practices can matter as much as the nominal fraction. |
| **Poll, household, or hut tax** | Charge a fixed amount for each eligible registered person, household, or dwelling. | Easy to calculate once the register exists; burdens poorer taxpayers more heavily relative to resources. Encourages concealment of eligible units and disputes over classification. |
| **Land tax** | Assess area × land quality × an administrative yield or value estimate; update the register periodically. | Fixed assessments do not automatically fall after crop failure. Improvements may escape reassessment, while obsolete assessments create uneven burdens. |
| **Corvée or labor levy** | Require eligible individuals or households to supply specified labor-days, substitutes, goods, or commutation payments. | The timing, travel, provisioning, and task matter. Ten days during harvest can cost much more than ten days in an agricultural slack season. |
| **Customs, tolls, and market dues** | Collect at an actual port, gate, bridge, market, or frontier crossing, using quantities or declared values. | Creates route substitution, concealment, bribery, inspection delays, and competition between jurisdictions. |
| **Excise, turnover, or consumption taxes** | Assess selected producers, sellers, or transactions that officials can observe. | Concentrated production sites may be easier to monitor than dispersed households. Repeated turnover taxes can accumulate along supply chains. |
| **Income, payroll, and value-added taxes** | Assess documented income or transactions, with withholding, third-party reporting, or invoice matching where institutions support them. | Administrative information becomes central. Identical nominal rates can generate very different revenue when reporting opportunities differ. |

The importance of information is not merely theoretical. Pomeranz’s experiments involving more than 400,000 Chilean firms found that transaction paper trails deterred VAT evasion and transmitted enforcement effects to suppliers. For TCE, records should therefore be **links between actual transactions and counterparties**, not a blanket national compliance percentage. [National Bureau of Economic Research](https://www.nber.org/papers/w19199?utm_source=chatgpt.com)

### 1.3 Tax farming is a collection contract, not a separate tax

A tax farmer acquires collection rights under specified terms, often providing the government with advance finance. Model the contract with:

* A territory and set of taxable claims.
* An advance, scheduled remittance, and expiration date.
* Rights to retain revenue and obligations to bear costs.
* Rules governing additional charges, coercion, complaints, and renewal.

The government gains finance and access to an intermediary’s organization. The intermediary gains an incentive to collect—but may also conceal receipts, protect associates, demand unofficial payments, or damage the future tax base. These outcomes should depend on monitoring, contract duration, access to credit, and political protection.

**Direct collection should not automatically outperform contracting.** Ottoman reformers attempted to abolish tax farming after 1839, but insufficient replacement administration caused revenue losses and a return to tax farmers. Institution replacement requires personnel and working systems, not simply passage of a law. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914)

A modern experiment illustrates the incentive problem without implying that modern Pakistan reproduces an Ottoman institution: performance incentives for property-tax collectors increased public revenue, but many taxpayers also reported higher bribes. “More revenue” and “less predatory collection” are separate objectives. [OUP Academic](https://academic.oup.com/qje/article-abstract/131/1/219/2461220)

### 1.4 Fiscal capacity is a set of capacities

Use separate institutional capabilities for:

**Identification:** Can officials find taxpayers, parcels, warehouses, employers, and ships?

**Assessment:** Can they estimate the taxable base and distinguish exemptions from concealment?

**Collection:** Can they deliver notices, transport goods, receive payments, and maintain accounts?

**Monitoring:** Can higher authorities detect diversion or abuse by collectors?

**Enforcement:** Can they obtain payment without prohibitive cost, flight, or resistance?

**Political authorization:** Can the government tax powerful groups, alter privileges, or maintain an accepted agreement over taxation and services?

This decomposition follows the broader state-capacity literature’s emphasis on administrative development, economic structure, and political incentives. Raising statutory rates cannot substitute indefinitely for these capabilities. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.28.4.99)

For each liability \(k\), let:

\[
A\_{ik}=\operatorname{Assess}\_k(\widehat B\_{ik},\text{status}\_i,\text{law}\_k)
\]

Here, \(\widehat B\) is the government’s **observed or recorded base**, not the simulation kernel’s true value \(B\). The assessment function can return money, goods, or labor obligations.

Do not collapse assessment, payment, and remittance into one operation. Each should be an event that can fail, be delayed, be appealed, or be manipulated.

### 1.5 Extraction is constrained by reproduction, not just enforcement

For an agricultural household, a useful diagnostic is:

\[
S\_i=Y\_i-I\_i-C\_i^{\min}-M\_i
\]

where \(Y\_i\) is gross production, \(I\_i\) necessary production inputs, \(C\_i^{\min}\) minimum household consumption, and \(M\_i\) maintenance or replacement requirements.

This is a **model diagnostic**, not an inviolable legal ceiling. Governments and other claimants may extract more than \(S\_i\). The consequences should appear as depleted stores, reduced consumption, debt, asset sales, deferred maintenance, migration, or mortality.

A simple illustrative stress test shows why tax form matters:

* Normal harvest: 100 units; fixed obligation: 20.
* Failed harvest: 50 units; unchanged obligation: 20.
* The obligation rises from **20% to 40% of the current harvest**.
* A genuine 20% proportional levy would instead fall to 10 units.

Also distinguish **wealth from liquidity**. A household may own land or livestock but lack the coin required on the due date. Borrowing, selling crops immediately after harvest, or selling productive assets should be possible responses.

### 1.6 Build the fiscal-military feedback loop—but allow failure

A useful causal sequence is:

> External threat → higher expenditure demand → emergency collections or borrowing → bargaining and administrative investment → improved military resources → possible expansion of the future tax base.

The alternative sequence is equally important:

> War → disrupted production and trade → falling receipts → arrears and coercive collection → declining cooperation → unpaid forces and political fragmentation.

Britain provides an illustrative fiscal-military benchmark: an official Treasury historical account places public spending’s temporary Napoleonic-era rise at roughly **12% to 23% of national income**, with public debt around **twice national income in 1815**. These are **spending and debt measures, not tax rates**. [GOV.UK](https://www.gov.uk/government/speeches/speech-by-the-permanent-secretary-to-the-treasury-sir-nicholas-macpherson-the-origins-of-treasury-control)

Do not make war automatically increase capacity. Whether new institutions survive depends on finance, administrative investment, political coalitions, and the postwar settlement.

### 1.7 Government finance needs stock-flow consistency

A simplified cash identity is:

\[
\Delta \text{Cash}
=
T+N+B\_{\mathrm{new}}+M
-G-iD-B\_{\mathrm{repaid}}
\]

Here \(T\) is tax receipts, \(N\) non-tax receipts, \(B\_{\mathrm{new}}\) new borrowing, \(M\) net monetary financing, \(G\) non-interest cash expenditure, and \(iD\) interest.

Maintain parallel physical inventories. A government with silver but no accessible grain cannot feed an army instantly. A government with grain but no acceptable payment medium may struggle to buy imported metal.

**Borrowing is financing, not revenue.** It transfers purchasing power now in exchange for future claims. Likewise, assigning a district’s taxes to a military officer finances government activity without necessarily producing a cash receipt at the capital.

---

## 2. Parameters and quantitative benchmarks

**Confidence key:** **H** = relatively well-defined recorded statistic or nominal obligation; **M** = localized reconstruction or historical synthesis; **L** = uncertain denominator, incomplete coverage, or contested inference. Confidence concerns the stated observation—not its applicability to every society.

### 2.1 Aggregate revenue and output

These entries deliberately retain different accounting boundaries.

| Setting and period | Quantitative anchor | Measurement and limitations | Confidence/source |
| --- | --- | --- | --- |
| Many European states, sixteenth century | Central taxation commonly **below 5% of GDP** | Reconstructed national output; excludes some local and non-central extraction. | **L–M**; Karaman & Pamuk. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914) |
| Many European states, 1780s | Approximately **5–10% of GDP**; Britain and Dutch Republic above **10%** | Central fiscal receipts; not a ceiling on household burdens. | **L–M**; Karaman & Pamuk. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914) |
| Ottoman Empire, early modern period | Central cash revenues **below 4% of GDP**; below **6%** when revenue assignments financing cavalry are included | Coverage changes with treatment of non-cash military finance; GDP is reconstructed. | **L–M**; Karaman & Pamuk. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914) |
| Qing China, around 1839–1842 | Official revenue at most approximately **2% of an estimated national-income denominator** | Sng–Moriguchi use a subsistence-based lower bound for income, producing an upper-bound ratio. Not a precise modern GDP estimate or total household burden. | **L**; Sng & Moriguchi. [Springer](https://link.springer.com/article/10.1007/s10887-014-9108-6) |
| African sample, 2022 | Average **16.0% of GDP**, 33 jurisdictions | Standardized tax-revenue measure; sample average, not population-weighted continental extraction. | **H within definition**; OECD. [OECD](https://www.oecd.org/en/publications/tax-policy-reforms-2025_de648d27-en/full-report/tax-revenue-context_80e66aad.html) |
| Asia-Pacific sample, 2022 | Average **19.3% of GDP**, 36 jurisdictions | Same caution about country coverage and averaging. | **H within definition**; OECD. [OECD](https://www.oecd.org/en/publications/tax-policy-reforms-2025_de648d27-en/full-report/tax-revenue-context_80e66aad.html) |
| Latin America and Caribbean sample, 2022 | Average **21.5% of GDP**, 27 jurisdictions | Same standardized comparison. | **H within definition**; OECD. [OECD](https://www.oecd.org/en/publications/tax-policy-reforms-2025_de648d27-en/full-report/tax-revenue-context_80e66aad.html) |
| OECD, 2024 | Average **34.1% of GDP**; available-country range **18.3–45.2%** | General-government taxes, including compulsory social-security contributions; preliminary 2024 country data. | **H, provisional**; OECD Revenue Statistics 2025. [OECD](https://www.oecd.org/en/about/news/press-releases/2025/12/labour-taxes-drive-oecd-tax-revenues-to-record-high-in-2024.html) |

**TCE interpretation:** These are scenario-validation anchors, not era limits. A government with modest central receipts may coexist with substantial village spending, aristocratic revenue assignments, religious dues, and compulsory labor.

For foragers and the earliest farming societies, the evidence assembled here does **not** support a comparable global tax/GDP range. Encoding one would create false precision. Where no standing extracting authority exists, “central tax take” is not the right variable; model transfers and obligations directly.

### 2.2 Instrument-level and behavioral parameters

| Parameter or observation | Value and unit | Correct use in TCE | Confidence/source |
| --- | --- | --- | --- |
| Nominal tithe | **10% of specified produce** | A canonical claim structure, not evidence that every payer surrendered exactly 10%, or that the recipient was the state. | **H for nominal definition**; UK National Archives. [National Archives](https://www.nationalarchives.gov.uk/latin/stage-1-latin/resources/stage-1-glossary-of-english-terms/) |
| Tokugawa agricultural assessments | Shogunate approximately **34%**; Aizu **50–55%** in 1637–1764; Chōshū **40%** in 1840 | Percentages of **assessed agricultural yield**, not national income and not necessarily current realized harvests. | **M**; Sng & Moriguchi. [link.springer.com](https://link.springer.com/article/10.1007/s10887-014-9108-6) |
| Mughal agrarian extraction, seventeenth century | Roughly **one-quarter to one-third**, possibly **one-half**, of agricultural produce | Habib’s broad estimate of revenue removed from the rural economy, quoted by Richards. Do not reinterpret as a measured central tax/GDP ratio. | **L; contested reconstruction**. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/abs/mughal-state-finance-and-the-premodern-world-economy/2F05763474FF90946BCB0FA8CF6D476D) |
| Contracted collectors’ advances, Maratha Malwa, eighteenth century | **One-third to one-half** of specified collections advanced | A financing parameter: collectors borrowed and advanced funds before completing collection. Not a general taxpayer rate. | **M**; Richards, citing Gordon. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/abs/mughal-state-finance-and-the-premodern-world-economy/2F05763474FF90946BCB0FA8CF6D476D) |
| English poll tax imposed in 1380 | **12 pence per eligible adult**; third poll tax in four years | Model fixed liability, eligibility, repeated demands, and collection campaigns. Convert to local wages or consumption costs only with matching data. | **H for nominal levy**; The People of 1381 project. [1381 Online](https://www.1381.online/about/about_the_revolt/) |
| Danish audit experiment | Evasion approximately **0.3%** for third-party-reported income versus **37%** for self-reported income | These are income-evasion measures, not percentages of people refusing all taxes. Demonstrates the importance of observability. | **H for study setting**; Kleven et al. [National Bureau of Economic Research](https://www.nber.org/papers/w15769) |
| Pakistan collector-incentive experiment | Revenue **9.4 log points higher** after two years—approximately **9.9% in levels** | A localized intervention effect, not a universal collector-productivity multiplier. The reported **46% higher growth rate** is not 46% higher revenue. | **H for experiment; limited external validity**; Khan, Khwaja & Olken. [OUP Academic](https://academic.oup.com/qje/article-abstract/131/1/219/2461220) |

A large assessed crop fraction can coexist with a much smaller national revenue ratio because agriculture is only part of output, assessments differ from realization, exemptions and arrears intervene, and proceeds may be distributed among multiple recipients.

### 2.3 Proposed sensitivity settings—not historical estimates

Some indispensable parameters cannot be calibrated globally from surviving evidence. Use explicit experimental settings rather than disguising guesses as scholarship.

| Model parameter | Suggested experimental values | Purpose |
| --- | --- | --- |
| Levy on **current gross harvest** | **5%, 10%, 20%, 30%** | Explore reproduction, savings, and compliance effects under otherwise identical conditions. |
| Customs rate on declared cargo value | **0%, 2.5%, 5%, 10%, 20%** | Test route diversion, inspection incentives, and revenue concentration. |
| Annual compulsory service | **0, 5, 15, 30, 60 days per eligible worker** | Stress-test seasonality, substitution, provisioning, and labor withdrawal. |
| Annual audit selection probability | **0%, 1%, 5%, 20%** | Separate audit selection from detection probability and punishment. |
| Land-register reassessment interval | **1, 5, 20 years** | Test administrative cost against assessment drift and distributional unfairness. |

**Source and confidence for this table:** authored TCE experiment design; **not empirically estimated ranges**. In the implemented economy, preferably derive realized collection reach and detection from staff time, routes, records, and inspections.

---

## 3. Variation across eras and regions, including spending

### 3.1 Foraging and early farming: do not equate sharing with taxation

For TCE, begin with distinctions between reciprocal transfers, emergency contributions, hospitality, obligatory offerings, and enforceable recurring claims. They may use similar physical goods while expressing different rights and expectations.

Agriculture makes some resources easier to assess, concentrate, and store, but it should not automatically create a state. Mayshar, Moav, and Pascali argue that the appropriability of cereals, rather than land productivity alone, helps explain state formation. A 2026 scholarly comment challenges the breadth and robustness of parts of their empirical evidence. **Use appropriability as a mechanism, not a grain-to-state unlock condition.** [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/718372)

Nor should the mechanism be restricted to Eurasian grain. Kolb’s work on ancient Hawai‘i describes the political importance of staple finance and ritual pig sacrifice—useful evidence for systems built around different productive ecologies and legitimating practices. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/abs/10.1525/ap3a.1999.9.1.89)

### 3.2 Ancient and pre-industrial systems: several fiscal organizations coexist

The useful distinction is not “primitive versus sophisticated,” but **how resources are mobilized**.

In the Andes, Inka finance combined organized labor, production, storage, and redistribution. D’Altroy and Earle distinguish **staple finance**, involving food and utilitarian goods, from **wealth finance**, involving more concentrated valuables. Their analysis makes transport, storage, and the organization of production central to political power. TCE should consequently permit a powerful government with extensive material provisioning but little coin circulation. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/203249)

In East Asia, similar agrarian production did not imply identical fiscal arrangements. Sng and Moriguchi contrast Japan’s village-level collection responsibilities with Qing administrative and monitoring problems. Their explanation emphasizes geography and principal-agent relationships; it should be treated as an influential model, not the sole accepted explanation of the divergence. [Springer](https://link.springer.com/article/10.1007/s10887-014-9108-6)

South Asian systems demonstrate that revenue assignments and elite establishments can finance armies and government without all proceeds passing through one treasury. Richards’ discussion of Mughal finance also emphasizes the relationship between revenue flows, political authority, and the ability to support officials and their followers. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/abs/mughal-state-finance-and-the-premodern-world-economy/2F05763474FF90946BCB0FA8CF6D476D)

For ancient societies more generally, avoid treating reconstructed national-income ratios as measurements of modern quality. Hopkins explicitly presented his influential Roman taxes-and-trade analysis as speculative, given the sparse evidence. Ancient settings are often better calibrated through documented obligations, provisioning requirements, and local accounts than through a single empire-wide percentage. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-roman-studies/article/abs/taxes-and-trade-in-the-roman-empire-200-bcad-400/44A06D708148DFA3A73E510A82D12E9B)

### 3.3 Colonial and postcolonial Africa: cash accounts can conceal labor extraction

Colonial fiscal burdens cannot be read solely from cash receipts. Van Waijenburg’s French African research finds that imputed labor taxation was, in most places studied, the largest component of early colonial budgets. The implication for TCE is substantial: a government can build roads cheaply **in its cash account** while imposing a large real burden on households. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/financing-the-african-colonial-state-the-revenue-imperative-and-forced-labor/9514245BA11D58B053FBB83E4EADF565)

For the longer run, Albers, Jerven, and Suesse reconstruct African fiscal development across 1900–2015, including measures expressed in labor-day equivalents. Their evidence challenges a simple narrative of uniformly stagnant fiscal capacity. It also cautions against assuming that resource revenue automatically reduces real non-resource tax collection: a falling tax-category *share* is not necessarily a falling *amount*. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/fiscal-state-in-africa-evidence-from-a-century-of-growth/347B8C8B89485D4687604C8F1C97F89B)

### 3.4 Industrialization: information and tax substitution, not just higher rates

The expansion of firms, recorded wage payments, accounting, and administrative organizations changes which tax bases are accessible. This helps explain why mass taxation can expand alongside economic development; income growth alone is not a sufficient mechanism. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.28.4.99)

Political choices can also reverse or redirect fiscal development. Britain abolished income tax in **1816** after opposition and reintroduced it in **1842**, partly to permit reductions in trade duties. The reintroduced rate was **7 pence in the pound**, approximately **2.92%**, on incomes above the stated threshold. This is a useful example of **tax substitution and institutional reversibility**, rather than a permanent technology unlock. [Parliament UK News](https://www.parliament.uk/about/living-heritage/transformingsociety/private-lives/taxation/overview/incometaxbolished/?utm_source=chatgpt.com)

### 3.5 Spending: model commitments and production, not fixed historical shares

There is no defensible universal allocation such as “all pre-industrial governments spend 70% on war.” Accounts differ in whether they include local military obligations, court establishments, religious services, assigned revenues, or debt charges.

TCE should nonetheless distinguish five major spending processes:

| Spending function | Resources and decisions to simulate | Important failure modes |
| --- | --- | --- |
| **Military and security** | Pay, food, equipment, animals, transport, fortifications, replacement, and mobilization | Cash without supplies; interrupted procurement; unpaid forces; requisitioning that shifts costs onto civilians. |
| **Court, ritual, and patronage** | Household establishments, ceremonies, gifts, artistic production, elite rewards, and officeholding | Coalition maintenance may consume resources needed elsewhere; benefits are concentrated. Do not classify every such payment as physically wasted. |
| **Administration and justice** | Officials, messengers, records, inspections, courts, collection facilities | Underfunding reduces future receipts; local dependence can weaken central monitoring. |
| **Public works** | Construction plus maintenance, skilled labor, materials, land access, and supervision | Monument completion without maintenance; forced labor scheduled during vital private work; locally harmful projects. |
| **Relief and social provision** | Food releases, transfers, tax remission, care, and eligibility decisions | Need rises when harvest-related receipts fall; inaccessible stores; favoritism; exclusion; depletion of reserves. |

Debt service deserves its own category. The Treasury historical account estimates that British **interest payments absorbed about half of public expenditure between 1820 and 1850**. Debt accumulated in war can therefore constrain a peacetime state for decades. [GOV.UK](https://www.gov.uk/government/speeches/speech-by-the-permanent-secretary-to-the-treasury-sir-nicholas-macpherson-the-origins-of-treasury-control)

For a consistently defined modern comparison, Eurostat’s **EU general-government expenditure in 2022** provides the following benchmarks:

| Function | Expenditure as share of GDP |
| --- | --- |
| Social protection | **19.5%** |
| Health | **7.7%** |
| General public services | **6.0%** |
| Economic affairs | **5.9%** |
| Education | **4.7%** |
| Public order and safety | **1.7%** |
| Defence | **1.3%** |

These are **shares of GDP, not shares of the budget**, and general public services include debt-related functions. They should not be mapped directly onto ancient categories. [European Commission](https://ec.europa.eu/eurostat/en/web/products-eurostat-news/w/ddn-20240305-1)

---

## 4. Responses, resistance, and stylized facts

### 4.1 Distinguish inability to pay from evasion

A taxpayer’s arrears should have a cause:

* Insufficient resources after a shock.
* Insufficient liquidity in the required payment medium.
* Disputed assessment or exemption.
* Deliberate concealment.
* Collective refusal.
* Payment diverted or falsely reported by an intermediary.

These causes call for different government responses. A reassessment may solve one problem; a payment extension another. More severe enforcement may fail when no resources exist to collect.

For deliberate evasion, a bounded-rational heuristic is:

\[
\text{Expected gain}
=
\text{tax avoided}
-
p\_i\bigl(\text{repayment}+\text{penalty}\bigr)
-
\text{concealment cost}
-
\text{social or moral cost}
\]

Here \(p\_i\) is the agent’s **perceived** probability of being caught, conditional on the proposed action. It should differ by income source, information available to officials, prior inspections, and observed experiences of peers.

The Danish experiment is particularly useful because it indicates that reporting opportunities can dominate broad claims about national honesty: evasion differed sharply between income visible to third parties and income reported by taxpayers themselves. [National Bureau of Economic Research](https://www.nber.org/papers/w15769)

Tax incidence should also emerge through markets. A merchant legally liable for a duty may adjust prices, margins, shipment size, or route. Do not automatically assign the entire economic burden to the legal payer—or automatically pass it all to consumers.

### 4.2 Resistance needs grievances, organization, and opportunity

Do not use:

```
if tax_rate > threshold:
    spawn_revolt()
```

Instead, let tax-related grievances respond to deterioration in consumption, arbitrary reassessment, coercive encounters, perceived unfairness, broken promises, and unequal exemptions. Mobilization then depends on communication, trusted organizers, shared institutions, expectations about others, and perceived enforcement risks.

**England, 1381.** The 1380 poll tax was followed by widespread avoidance and renewed collection efforts. The People of 1381 project documents the importance of collection commissions and attacks on officials. The lesson is that the chain from assessment to enforcement matters; the nominal rate alone does not describe the event. [1381 Online](https://www.1381.online/about/about_the_revolt/?utm_source=chatgpt.com)

**Southeastern Nigeria, 1929.** Susan Martin’s account connects women’s economic pressures and threatened autonomy with the counting of women and children by tax assessors. Women had been exempt from the direct tax imposed on men. The threatened extension of taxation, rather than an already collected uniform increase, helped precipitate revolt. TCE therefore needs beliefs about **future claims**, gendered interests, and politically meaningful exemptions—not just current disposable income. [Cambridge University Press](https://www.cambridge.org/core/books/abs/palm-oil-and-protest/production-and-protest-the-women-riot-1929/392C477E75EBC9A71E170255903D7B8C)

These cases support a modeling conclusion: assessments, censuses, and enforcement campaigns can themselves become political events.

### 4.3 Patterns a correct simulation should be capable of reproducing

The following are validation targets, not outcomes that must occur in every world.

| Pattern | Test or observable implication |
| --- | --- |
| **Low central receipts coexist with heavy local burdens** | Trace every payment and retention. A weak central treasury must not imply lightly burdened households. |
| **Fixed obligations amplify harvest shocks** | Holding policy constant, the fixed-tax/current-harvest ratio rises as output falls. Proportional levies should respond differently. |
| **Visibility changes compliance** | Third-party-reported or physically inspected bases should be harder to conceal than equally taxed unobserved bases; compare directionally with the Danish evidence. |
| **Collector incentives have multiple effects** | Stronger revenue incentives can raise collections while also increasing unofficial charges under weak oversight; compare with the Pakistan experiment. |
| **Military finance creates persistence** | War can leave debt service, new administrative organizations, or entrenched revenue claims after fighting ends. |
| **Fiscal transitions can fail** | Removing an intermediary before replacing its collection capacity can reduce receipts even when the new institution is nominally better. |
| **Resistance is clustered and contingent** | Similar fiscal burdens should produce different outcomes depending on networks, legitimacy, organization, and recent enforcement. |
| **Public benefits and burdens are uneven** | The location and recipients of spending matter; national expenditure alone should not determine every agent’s opinion. |
| **Tax composition changes with economic organization** | Formal employment and transaction records can make new bases practical without forcing old taxes to disappear immediately. |
| **Revenue categories can mislead** | A declining customs or resource share need not mean declining real revenue from that source. Track levels, shares, and price changes separately. |

For historical anchors behind these tests, use the tables above and the colonial labor, enforcement, and institutional-transition studies—not a single cross-era regression.

---

## 5. Modeling recommendation for TCE

### 5.1 Use one composable claim-and-collection system

The minimum useful architecture is:

| Entity or component | Essential state |
| --- | --- |
| **Fiscal authority** | Jurisdiction, treasury inventories, borrowing authority, laws, budgets, obligations, and political constraints |
| **Tax rule** | Base, eligibility, exemption conditions, assessment method, payment medium, calendar, collector, penalties, appeal and remission rules |
| **Taxpayer account** | Assessments, payments, arrears, disputes, exemptions, and responsible household/person/firm |
| **Administrative observation** | Observed value, source, observation date, reporting party, and reliability |
| **Collector or collection organization** | Assigned claims, working funds, routes, staff time, incentives, political relationships, and remittance obligations |
| **Collection contract** | Advance, rights, duration, territory, retention rules, guarantees, and renewal conditions |
| **Public expenditure commitment** | Recipient or project, resource requirements, payment schedule, priority, and consequences of non-performance |

A temple, town council, monarch, subordinate lord, or modern tax agency can then use the same underlying machinery with different rules and authority structures.

Keep **individual eligibility and experience** even when payment accounts are household-based. Pooling household resources should not erase who performs labor, controls income, receives an exemption, or encounters a collector.

### 5.2 Schedule fiscal work around relevant events

For 10k–50k people, avoid re-evaluating every tax rule against every agent each simulation tick.

A proposed schedule:

**Transaction events:** Customs at crossings, market dues, wage withholding, recorded sales, interest payments.

**Seasonal events:** Harvest assessment, produce delivery, labor musters, livestock counts, relief planning.

**Periodic administrative work:** Collection visits, audits, remittances, appeals, reassessment, and arrears review.

**Political events:** Budget decisions, emergency levies, exemption bargaining, contract renewal, tax resistance, and institutional reform.

Index taxpayers by jurisdiction and tax base. Collectors should process bounded queues with actual travel and service costs. Benchmark the implementation rather than assuming a historical ratio of officials to population guarantees computational feasibility.

### 5.3 Preserve imperfect information without simulating every document

The kernel may know every harvest and transaction. Authorities should not.

A compact observation record can hold:

```
subject
tax_base_type
observed_quantity_or_value
observation_date
reporting_source
confidence
```

Age and revise these records as people move, parcels change, households divide, and enterprises open or close. An audit compares claims with information the institution can actually obtain.

This provides much of the behavioral value of imperfect administration without simulating every sheet of paper, clerk conversation, or fraudulent ledger entry.

### 5.4 Make budgets compete for real resources

Governments should budget against forecasts, not guaranteed future receipts.

Prioritize commitments according to institutional choices: soldiers, court dependents, creditors, emergency food, maintenance, or political allies. Unpaid obligations should generate consequences specific to their recipients.

Relief reserves can use an explicit target:

\[
\text{desired reserve}
=
\text{eligible population}
\times
\text{daily ration}
\times
\text{coverage days}
\]

That is a design rule, not a universal historical practice. It makes provisioning legible and links storage, transport, and fiscal planning.

For public works, separate authorization, procurement, construction progress, and maintenance. A budget entry alone should not create a bridge.

### 5.5 Existing models worth adapting

**Sng–Moriguchi’s principal-agent model:** Useful for separating rulers’ fiscal interests from collectors’ behavior and for making monitoring costly. Borrow that structure; do not copy a single revolt threshold as a complete political model. [Springer](https://link.springer.com/article/10.1007/s10887-014-9108-6)

**Epstein’s civil-violence model and NetLogo’s Rebellion implementation:** Useful starting points for heterogeneous grievances, perceived enforcement risks, and decentralized activation. TCE should replace random movement and highly simplified legitimacy with persistent communities, relationships, organizations, and concrete disputes. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC128592/)

**The AI Economist / Foundation framework:** Useful as a modular example of interacting household behavior, economic activity, and government tax policy. Its optimization experiments are not historical validation, and a learned planner objective should not become TCE’s universal model of government motivation. [arXiv](https://arxiv.org/abs/2004.13332)

**TaxAI:** A research environment incorporating heterogeneous households, government, firms, and financial institutions, with reported experiments at 10,000-household scale. It is relevant to later macroeconomic extensions, not evidence that the same complexity is necessary or performant in TCE’s spatial Rust kernel. [arXiv](https://arxiv.org/abs/2309.16307)

For TCE’s initial agrarian implementation, **bounded heuristics and explicit institutions are preferable to training a universal optimal-tax planner**. Governments need motives, information, and constraints—not privileged access to the simulation’s welfare function.

### 5.6 Minimum dashboard and invariants

Track assessed liabilities, actual payments, public receipts, central remittances, collection costs, unauthorized extraction, arrears, labor-days, in-kind stocks, debt service, and benefits received.

Show distributions by wealth, locality, legal status, occupation, and politically relevant group. Aggregate revenue/GDP is insufficient.

The most valuable invariant tests are:

* Every transferred good, payment, and debt claim has a corresponding owner or counterparty.
* Consolidation eliminates internal government transfers.
* Diversion enriches an actor or funds an activity; it does not simply vanish.
* Spoilage destroys physical goods explicitly.
* Labor obligations consume actual time.
* Government spending cannot create unavailable labor or materials.
* Tax payment itself is not new production.
* Officials cannot assess an unobserved base using the kernel’s hidden truth.

---

## 6. Sources, datasets, and limits of inference

### Recommended calibration source stack

| Source | Best use |
| --- | --- |
| **Karaman & Pamuk, “Ottoman State Finances in European Perspective, 1500–1914” (2010), and associated data** | Long-run central revenues, European–Ottoman comparison, and changes in fiscal organization. The author’s research page provides the dataset and source documentation. [Google Sites](https://sites.google.com/view/kivanckaraman/home) |
| **Albers, Jerven & Suesse, “The Fiscal State in Africa: Evidence from a Century of Growth” (2023)** | African fiscal development over 1900–2015, including comparisons less dependent on uncertain historical GDP estimates. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/fiscal-state-in-africa-evidence-from-a-century-of-growth/347B8C8B89485D4687604C8F1C97F89B) |
| **Van Waijenburg, “Financing the African Colonial State” (2018), replication data** | Making compulsory labor visible in fiscal accounts. The replication dataset is available through openICPSR. [openICPSR](https://www.openicpsr.org/openicpsr/project/101190/version/V1/view) |
| **OECD Global Revenue Statistics Database** | Harmonized modern tax categories and government-level comparisons; coverage includes 141 economies and data from 1990 onward. [OECD](https://www.oecd.org/en/data/datasets/global-revenue-statistics-database.html) |
| **Eurostat government expenditure by function; OECD Government at a Glance** | Modern expenditure composition with explicit accounting categories. [European Commission](https://ec.europa.eu/eurostat/en/web/products-eurostat-news/w/ddn-20240305-1) |
| **The People of 1381 project** | Microhistorical records linking taxpayers, officials, places, collection efforts, and resistance. [1381 Online](https://www.1381.online/about/about_the_revolt/?utm_source=chatgpt.com) |

### Claims that require particular caution

**Ancient and early-modern aggregate ratios are reconstructions.** Uncertain population, output, prices, local revenues, and non-cash obligations can substantially alter the result.

**A nominal fraction is not a realized burden.** Assessed yield, current yield, gross production, net income, and national value added are different denominators.

**Normative rules are not administrative practice.** A law may describe what should be collected while surviving accounts reveal exemptions, bargaining, arrears, and unauthorized additions.

**Modern experiments identify local effects.** The Denmark, Chile, and Pakistan results are strong evidence about particular mechanisms, not ready-made universal coefficients for every historical society.

**State-formation theories remain contested.** The cereal-appropriability literature is useful for designing hypotheses, but the 2026 critique is a reason to preserve alternative institutional paths rather than script one origin story. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/740225)

**Overall recommendation:** Make fiscal outcomes emerge from **observable tax bases, enforceable claims, material collection, intermediary incentives, political bargaining, and real expenditure commitments**. Calibrate against several deliberately different historical cases. That gives TCE room to generate a cash-poor provisioning state, a heavily burdened but weakly centralized empire, a commercially financed military power, or a high-tax welfare government—without hard-coding any of them as the inevitable next era.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92896-7508-83ea-a734-3babfea680b5)
