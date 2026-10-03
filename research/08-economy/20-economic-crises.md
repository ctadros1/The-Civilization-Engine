# Pre-industrial economic crises: a simulation-ready report for TCE

## Core recommendation

**Model crises as failures of particular economic functions—not as a single “recession” state.** A settlement can have enough food but distribute it disastrously; maintain functioning markets while people starve; possess valuable assets but lack acceptable means of payment; or suffer falling commodity prices that bankrupt heavily indebted merchants. Historical famine and financial-crisis research documents these distinctions. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/famine-and-market-in-ancien-regime-france/A01CEF4F777ADEB4CEAFF68AAABA93CC)

For TCE, the essential interacting systems are **food production and storage, household access to resources, payment instruments, credit obligations, and transport networks**. Crises should emerge when shocks and obligations overwhelm the buffers in these systems.

A failed harvest should not automatically cause famine. A debasement should not automatically cause hyperinflation. A merchant failure should not automatically close every bank. Conversely, a government’s declaration of relief should not create food, transport capacity, or creditworthiness.

The report below distinguishes **historical observations and reconstructions** from **proposed simulation parameters**. The latter are starting points for sensitivity testing, not estimates of universal pre-industrial behavior.

---

## 1. Mechanisms: implementable causal rules

### 1.1 Harvest crises: the interaction of shocks, stocks, and commitments

The relevant shock is not simply “this year’s harvest was poor.” It is a shortfall relative to the food, seed, fodder, and contractual requirements that must be met before the next replenishment.

Crop failures can also be correlated across crops and settlements. Historical English records distinguish autumn-sown and spring-sown grains, whose exposure to adverse weather differed; sometimes spring crops replaced destroyed winter crops. In India, reconstructed droughts show why neighboring regions could simultaneously lose their ability to assist one another. [British Association for Holistic Science](https://www.bahs.org.uk/crop-yields-database/the-data/)

**Production rule.** Determine output from planted area, growing-season conditions, available labor, seed quality, draft power, and harvesting capacity. Feed the same underlying weather into affected farms rather than drawing an independent harvest shock for every household.

**Inventory rule.** Every physical stock has an owner, location, quality, and permitted uses:

\[
G\_{t+1}=G\_t+H\_t+R\_t-C\_t-S\_t-F\_t-O\_t-L\_t
\]

Here \(G\) is stored grain; \(H\), harvest receipts; \(R\), purchases and transfers received; \(C\), human consumption; \(S\), seed; \(F\), fodder; \(O\), sales, taxes, and transfers out; and \(L\), physical losses. Use consistent mass units and account for each transfer at both ends.

**Commitment rule.** Separate fixed obligations from proportional ones. A fixed grain rent consumes a larger fraction of a bad harvest; a share rent automatically shares some production risk. This distinction appears explicitly in the provisions on agricultural leases in Hammurabi’s law collection. Those provisions establish legal possibilities, not proof of uniform enforcement. [Avalon Project](https://avalon.law.yale.edu/ancient/hamcode.asp)

An illustrative calculation shows the amplification. Suppose normal output is four units, with one reserved for seed and one owed as a fixed grain payment. Two remain for consumption. A 20% harvest loss reduces output to 3.2, leaving only 1.2 for consumption: **a 40% reduction in disposable food**.

**Carryover rule.** Distinguish:

* food held to bridge the ordinary interval between harvests;
* precautionary stocks carried beyond that interval;
* seed and fodder that can be consumed only by sacrificing future production.

A granary full immediately after harvest is not necessarily a large emergency reserve. Conversely, do not assume pre-industrial grain could never survive multiple years: direct evidence from English farms in 1750–1850 documents such storage, although carryover quantities were generally small. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13133)

**Damage-persistence rule.** Households may survive by eating seed, selling tools, slaughtering draft animals, or abandoning land. Subsequent good weather does not repair these losses automatically. A simple planting constraint is:

\[
A\_{\text{planted}}
=\min\left(
A\_{\text{available}},
\frac{\text{seed available}}{\text{seed required per area}},
\frac{\text{labor available}}{\text{labor required per area}}
\right)
\]

Draft power, irrigation, and tools can modify the requirements or impose additional constraints.

### 1.2 Famine: food availability and access are different variables

**Allocate food through actual ownership, purchasing power, obligations, and relief eligibility.** Do not divide settlement-wide food equally among residents unless an institution actually performs that redistribution.

Landless workers, small cultivators who buy food seasonally, artisans, dependents, and substantial grain sellers can experience the same price increase very differently. Research on medieval England documents food prices outrunning wages; parish-level evidence from nineteenth-century Finland finds that income distribution materially affected famine mortality. [Taylor & Francis Online](https://www.tandfonline.com/doi/full/10.1080/03044181.2023.2250952)

Implement several linked responses:

**Consumption adjustment.** Households substitute foods, reduce diet quality, reduce consumption, request assistance, borrow, and sell assets according to available options.

**Demand transmission.** Increased food expenditure reduces purchases of clothing, tools, construction, and services. Producers of those goods may reduce employment, creating a second group unable to afford food.

**Unequal gains and losses.** A grain owner may gain from scarcity while a neighboring laborer loses. A producer’s outcome depends on their net market position, harvest loss, and obligations—not their occupation label alone.

**Health feedback.** Accumulate nutritional deficits over time and connect them to work capacity and susceptibility to disease. Avoid converting one missed meal directly into mortality. Historical famine deaths included substantial epidemic components; research on Finland in 1695–1697 identifies disease outbreaks behind the spring and summer mortality peak. [Taylor & Francis Online](https://www.tandfonline.com/doi/full/10.1080/03468755.2014.937740)

**Market behavior.** Allow precautionary retention, speculation, emergency sales, and forced liquidation. Do not assume merchants invariably withhold grain during famine. Studies of French crises conclude that markets provided some mitigation, while research on medieval English institutions documents retention and exclusion that could worsen access. These findings concern different settings; neither supports a universal “markets always save” or “markets always cause famine” rule. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/famine-and-market-in-ancien-regime-france/A01CEF4F777ADEB4CEAFF68AAABA93CC)

For an analytical benchmark, consider constant-elasticity demand:

\[
\frac{P\_1}{P\_0}
=\left(\frac{Q\_1}{Q\_0}\right)^{-1/\eta}
\]

where \(\eta>0\) is the magnitude of demand elasticity. With \(\eta=0.3\), a 20% supply reduction implies a price multiplier of approximately **2.10**.

This is a sensitivity illustration, not a universal famine-price formula. In TCE, income changes, substitutions, storage releases, rationing, and imports should determine the actual outcome. Historical reconstructions themselves are highly sensitive to the elasticity assumed. [ResearchGate](https://www.researchgate.net/publication/24140339_Market_Segmentation_and_Famine_in_Ancien_Regime_France)

### 1.3 Debasement and monetary crises: distinguish metal, accounting units, and acceptance

For metallic currencies, store at least:

\[
\text{fine-metal content per coin}
=\text{coin mass}\times\text{fineness}
\]

Keep this separate from the coin’s legal denomination and market exchange value. For paper or deposit money, track issuer liabilities, redemption terms, and acceptance.

**Fiscal rule.** A government facing a financing gap can raise taxes, borrow, delay payments, requisition goods, sell assets, or alter money issuance. These choices should have different distributional and credibility consequences.

**Acceptance rule.** Agents value a currency according to where it is accepted, whether taxes can be paid with it, expected redemption, transaction costs, and beliefs about future purchasing power. Persistent deterioration can encourage substitution into foreign coin, metal, goods, or another issuer’s liabilities.

**Contract rule.** Record whether obligations are denominated in local money, a weight of silver, grain, or another indexed unit. Inflation can reduce the real burden of fixed nominal debts while leaving grain debts unchanged. Monetary reform can therefore help one group and harm another.

The Ottoman debasement of **1585–1586 reduced the akçe’s silver content by 44%**. Holding the silver price of goods constant would imply about \(1/0.56=1.79\) times as many coins per purchase. That is an accounting benchmark, **not a prediction of immediate 79% consumer-price inflation**. Pamuk also warns that official mint standards need not describe the actual circulating coin mix. [Ataturk Institute](https://ata.bogazici.edu.tr/sites/ata.boun.edu.tr/files/faculty/sevket.pamuk/pamuk_price_revolution.pdf)

Paper money is not inherently a modern-stage institution, nor inherently unstable. Guan, Palma, and Wu’s study of Yuan China finds nearly half a century of comparatively moderate inflation, followed eventually by severe monetary deterioration; their evidence associates excessive issuance particularly with military fiscal pressure. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13305)

**Validation implication:** display nominal food prices, food prices in a relatively stable accounting standard, and real wages separately. Otherwise a monetary crisis and a physical food shortage become indistinguishable.

### 1.4 Credit and banking collapses: liquidity, solvency, and connected obligations

Credit crises do not require modern banks. Households, landlords, merchants, partnerships, governments, and deposit-taking institutions can all become interdependent through promises to deliver goods or money.

Represent every material obligation as a claim with a debtor, creditor, amount, denomination, maturity, enforcement rules, and—where relevant—collateral or guarantors.

Two conditions must remain separate:

\[
\text{Equity}
=\text{value of assets}-\text{liabilities}
\]

**Insolvency** means asset value is insufficient to cover liabilities. **Illiquidity** means the entity cannot meet payments currently due, even though its longer-term assets may cover them.

A practical propagation sequence is:

**Loss or suspicion → funding withdrawal or failed rollover → asset sales → lower collateral values → additional losses and calls for payment.**

The northern European crisis of 1763 is an important pre-industrial example: interlocking credit relationships and leverage produced distress sales and liquidity contagion. It was not simply a modern-style retail deposit run transplanted into the eighteenth century. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1162/1542476042813887)

Implement three distinct transmission channels:

| Channel | Implementable rule |
| --- | --- |
| Direct default | An unpaid claim reduces the creditor’s cash receipts and possibly asset value. |
| Shared collateral | Distress sales change prices used to value similar assets held by other borrowers. |
| Funding confidence | News about relevant counterparties changes renewal willingness, collateral requirements, or withdrawal behavior. |

Not every default should cause immediate liquidation. Permit negotiated extensions, partial repayment, guarantor payment, seizure, partnership dissolution, and legal stays. Unlimited partnership liability or personal enforcement should be institutional settings, not silently replaced with modern limited liability.

Sovereign exposure is one possible source of concentrated losses, not a universal explanation. Hunt’s reassessment of the Bardi and Peruzzi challenges the familiar claim that enormous loans to Edward III alone explain their failures. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/new-look-at-the-dealings-of-the-bardi-and-peruzzi-with-edward-iii/2AD04842FACDAE9D0A660EDCD72EC097)

### 1.5 Trade disruption: buffering and contagion use the same network

Model trade links with **capacity, journey time, loss risk, financing requirements, and political permissions**. A positive price difference is insufficient to produce imports when ships, carts, working capital, or safe passage are unavailable.

The same connection can have opposite effects under different shocks:

| Situation | Expected mechanism |
| --- | --- |
| One settlement has a bad harvest | Imports can buffer the shortage, while raising prices in supplying settlements. |
| Most connected settlements have correlated failures | The network has little surplus to redistribute. |
| An importing city is blockaded | Its prices rise; stranded exporters may experience falling prices and incomes. |
| A major merchant loses finance | Otherwise profitable cargoes may not be purchased or dispatched. |
| Authorities prohibit exports | Local access may improve temporarily while neighboring shortages worsen. |

These are model implications to test, not guarantees under every institutional arrangement. Research on the Revolutionary and Napoleonic Wars demonstrates that trade disruption generated substantial, geographically uneven economic losses even before modern industrial supply chains. [Cambridge University Press](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/B5D21C47E53307E78358803D4695FCE8/S1740022806000076a.pdf/worldwide_economic_impact_of_the_french_revolutionary_and_napoleonic_wars_17931815.pdf)

Keep **common exposure** distinct from **contagion**. Two towns experiencing the same drought are not evidence that distress traveled between them. For debugging, replay a shock with selected trade or credit links disabled while holding weather constant.

### 1.6 Policy responses and recovery

Historical interventions included conditional debt relief, public food storage, and emergency credit. Hammurabi’s §48 provides for relief from grain repayment in a harvest-failure year. Qing China developed extensive civilian granary institutions. Tacitus reports three-year interest-free public lending secured on land during the Roman crisis of AD 33; this is a historical account of a policy, not a modern audited evaluation. [Avalon Project](https://avalon.law.yale.edu/ancient/hamcode.asp)

Translate these possibilities into budget-constrained actions:

| Response | Immediate effect in TCE | Necessary limitation or possible adverse effect |
| --- | --- | --- |
| Grain release or free rations | Transfers existing food to buyers or eligible recipients. | Stock location, transport, spoilage, exclusion, and replenishment matter. |
| Cash relief or public employment | Raises recipients’ purchasing power. | Does not itself increase physical supply; work requirements can exclude the incapacitated. |
| Tax or rent remission | Leaves resources with affected households. | Transfers the financing shortfall to landlords or government. |
| Import facilitation and transport protection | Reduces barriers to obtaining outside supplies. | Requires an external surplus and usable transport capacity. |
| Price controls and compulsory marketing | Changes legal terms of exchange. | Requires procurement, enforcement, or rationing when supply does not meet demand. |
| Debt extensions or write-downs | Reduces immediate debtor pressure. | Delays or destroys creditor income and can transmit losses. |
| Emergency lending | Bridges a liquidity shortage against acceptable claims or collateral. | Cannot make worthless assets sound; losses need an eventual bearer. |
| Seed, livestock, and infrastructure replacement | Restores productive capacity. | Competes with immediate consumption and other public spending. |

Public relief should also affect incentives. Shiue finds that Chinese provinces receiving more frequent central disaster relief tended to maintain lower local granary stocks, interpreting this as a self-insurance incentive problem. Model that as a possible institutional response, not proof that relief is generally harmful. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/local-granaries-and-central-government-disaster-relief-moral-hazard-and-intergovernmental-finance-in-eighteenth-and-nineteenthcentury-china/80F2E2CEBBEA518A60D40E2116312898)

**Recovery should be a vector, not a timer.** Track the normalization of food consumption, food stocks, prices, production, employment, credit, household assets, and population separately. A settlement may regain low grain prices while its poorest households remain indebted or dispossessed. Another may recover through migration and a smaller population rather than restoration of its former scale.

---

## 2. Quantitative calibration

### 2.1 Historical anchors

**Confidence refers to the specific observation, not its transferability to every society.** High means relatively well-documented records or policy terms; medium means reconstruction or meaningful measurement uncertainty; low means especially thin coverage or strong interpretive dependence.

| Quantity and setting | Value or range | Units and interpretation | Confidence and source |
| --- | --- | --- | --- |
| Harvest-failure frequency, nineteenth-century Finland | Approximately once every **3 years** | Failures of varying severity; northern areas faced greater risk. **Not famine frequency.** | Medium; Voutilainen, *Income inequality and famine mortality*. [onlinelibrary.wiley.com](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13095?utm_source=chatgpt.com) |
| Finnish crisis, 1867–1868 | Regional harvest destruction **over 50%**; grain prices roughly **2×**; real wages **−25%** | Different indicators of one predominantly agrarian crisis; price comparison is autumn 1866 to spring 1868. | Medium; same study. [onlinelibrary.wiley.com](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13095?utm_source=chatgpt.com) |
| French famine, 1693–1694 | About **6%** of population | Estimated famine death toll; not a normal crisis severity. | Medium; Ó Gráda and Chevet. [ResearchGate](https://www.researchgate.net/publication/24140339_Market_Segmentation_and_Famine_in_Ancien_Regime_France) |
| French demographic response, 1694 | Deaths **+98.8%**; baptisms **−26.0%** | Changes against the study’s 1680–1692 baseline; **doubling deaths does not mean half the population died**. | Medium; reconstructed national totals. [ResearchGate](https://www.researchgate.net/publication/24140339_Market_Segmentation_and_Famine_in_Ancien_Regime_France) |
| Famine-report years, Mughal Empire | About **10 of 75 years**, 1555–1630; approximately **half of years**, 1630–1670 | Famine recorded somewhere in the empire, not local annual probability. | Low–medium; Morshed’s documentary compilation, reported in a research summary. [LSE Blogs](https://blogs.lse.ac.uk/economichistory/2022/03/16/climate-change-and-rebellion-in-mughal-india/) |
| Indian drought and famine cluster, 1630–1632 | **3 consecutive years**; drought affected roughly **half of India** at its 1632 peak | Historical crisis chronology combined with reconstructed drought extent; drought area is not lost crop area. | Medium; Mishra and Aadhar, 2021. [Nature](https://www.nature.com/articles/s41612-021-00219-1) |
| Ottoman akçe, 1585–1586 | **−44%** fine-silver content | Monetary-standard change, not directly measured inflation. | High for the standard; actual circulation less certain. [Ataturk Institute](https://ata.bogazici.edu.tr/sites/ata.boun.edu.tr/files/faculty/sevket.pamuk/pamuk_price_revolution.pdf) |
| Roman emergency lending, AD 33 | **3 years**, **0% interest**, land security worth **2×** loan | Reported rescue terms, equivalent to a maximum 50% loan-to-collateral ratio under the stated valuation. | Medium as historical testimony; Tacitus, *Annals* 6.17. [Internet History Sourcebooks](https://sourcebooks.web.fordham.edu/ancient/tacitus-annals.asp) |
| Grain-price reversal preceding the 1763 crisis | **−30%**, November 1762–May 1763 | Berlin and Hamburg grain prices; a collateral/inventory-price shock, not a harvest-loss measure. | Medium–high; Quinn and Roberds. [FRASER](https://fraser.stlouisfed.org/files/docs/historical/frbatl/wp/frbatl_wp_2012-08.pdf) |
| Amsterdam failures, August–September 1763 | **38 firms** | Recorded bankruptcies; many smaller firms reopened within months after settlements. Activity was moving toward normal by October, at a lower level. | High for documented chronology; Quinn and Roberds. [FRASER](https://fraser.stlouisfed.org/files/docs/historical/frbatl/wp/frbatl_wp_2012-08.pdf) |
| Revolutionary/Napoleonic trade disruption | US **5–6%**; France **3–4%**; Britain **1.7–1.8%** | Estimated **annual welfare losses** from the study’s model. **Not measured GDP contractions.** | Medium, model-dependent; O’Rourke. [Cambridge University Press](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/B5D21C47E53307E78358803D4695FCE8/S1740022806000076a.pdf/worldwide_economic_impact_of_the_french_revolutionary_and_napoleonic_wars_17931815.pdf) |

These anchors show why a universal rule such as “a major crisis every twenty years” would be misleading. Even within one environment, bad harvests could recur frequently while famine remained exceptional. Financial-crisis incidence additionally depends on whether the relevant credit institutions and exposures exist.

### 2.2 Proposed starting parameters—not historical estimates

Use the following as a deliberately broad initial sensitivity envelope. Replace them with fitted local or institutional distributions as TCE’s crop, household, and finance models mature.

| Parameter | Proposed initial range | Units | Evidence status and calibration approach |
| --- | --- | --- | --- |
| Ordinary settlement-level harvest variability | **0.10–0.25** | Standard deviation of log harvest deviations after controlling for area and trend | **Design prior, low confidence.** Fit crop-specific records; do not apply independently to every person. |
| Severe harvest stress test | **20–50% below normal** | Fraction of harvest | **Scenario range**, not an assigned event frequency; informed by the scale of documented failures above. |
| Regional shock correlation | **0.2–0.9** | Pairwise correlation | **Design prior.** Fit geography and weather; distant or ecologically different partners should not receive identical values. |
| Annual shock persistence | **0–0.6** | AR(1) coefficient | **Design prior.** Calibrate to the climate process, not famine records alone. |
| Desired precautionary food carryover | **0–90** | Days of household consumption beyond ordinary seasonal needs | **Design prior.** Actual attainment depends on wealth, expectations, and storage costs. |
| Public reserve target | **30–180** | Days of consumption for the intended recipient population | **Design prior.** This is not necessarily the entire population and is not automatically funded. |
| Dry-grain storage loss | **2–15%** | Fraction per year | **Design prior.** Separate routine deterioration from floods, fire, pests, and processing losses. |
| Short-run demand-elasticity magnitude | **0.2–0.6** | Dimensionless | **Analytical sensitivity range**, not a universal measured elasticity; comparable values have been debated in historical reconstructions. [ResearchGate](https://www.researchgate.net/publication/24140339_Market_Segmentation_and_Famine_in_Ancien_Regime_France) |
| Merchant-credit maturity | **30–180** | Days | **Design prior.** Agricultural loans should instead follow production and repayment calendars. |
| Leveraged intermediary balance-sheet ratio | **2–8** | Assets/equity | **Design prior for testing**, not an estimated historical banking norm. |
| Emergency loan collateral discount | **10–50%** | Deduction from assessed collateral value | **Design prior.** Interacts with valuation uncertainty, enforceability, and political favoritism. |

Three implementation cautions matter more than any particular default.

First, convert annual rates correctly. An annual loss fraction \(d\) corresponds to a daily rate \(1-(1-d)^{1/365}\), not the same percentage applied daily.

Second, do not impose statistical yield variation on top of a weather-and-agronomy model that already generates it. These targets are for checking the resulting distribution.

Third, **do not parameterize recovery as “the crisis lasts two years.”** Recovery duration is an output of surviving assets, institutions, subsequent harvests, and access to outside resources.

---

## 3. Variation across economic settings and world regions

### 3.1 Developmental setting: capabilities, not era switches

| Setting | Main vulnerabilities | Main buffers and TCE implications |
| --- | --- | --- |
| Predominantly foraging societies | Failure of particular food resources, illness of productive members, exclusion from territory, and disruption of sharing relationships. | Model mobility, dietary breadth, and social transfers. Sharing is widespread in studied forager and forager-farmer populations, but its rules and motivations vary. [Cambridge University Press](https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/abs/to-give-and-to-give-not-the-behavioral-ecology-of-human-food-transfers/45EBF9C9619A612ECCB77B6F81449F7D) |
| Early farming communities | Seasonal concentration of food supply, seed requirements, local crop dependence, and stock destruction. | Storage and diversified production can buffer risk, but ownership determines access. Storage must not be locked behind domesticated agriculture: granaries in the Jordan Valley preceded full domestication. [DOI](https://doi.org/10.1073%2Fpnas.0812764106) |
| Pre-industrial agrarian states and commercial societies | Harvest shocks interact with rent, taxation, urban provisioning, warfare, currencies, and credit chains. | Enable mechanisms according to actual institutions. Public granaries, private merchants, kin networks, and creditors can coexist rather than replace one another in a fixed sequence. [University of Michigan Press](https://press.umich.edu/Books/N/Nourish-the-People) |
| Industrializing economies | Wage dependence, transport dependence, commercial inventory exposure, and increasingly extensive financial connections coexist with agrarian vulnerability. | Do not switch famine off when factories appear. Nineteenth-century Finland is a useful example of severe agrarian vulnerability within an industrializing world. [onlinelibrary.wiley.com](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13095?utm_source=chatgpt.com) |
| Modern economies | Financial and employment crises can occur without an initial local food-production failure; war and institutional breakdown can still produce famine. | Transport, public health, relief capacity, and financial backstops alter the transmission mechanisms. Their existence and effectiveness—not the calendar date—should govern resilience. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjel.45.1.5) |

An epidemic also differs from a harvest shock. The loss of people can reduce aggregate production while changing land availability and bargaining power in ways that eventually improve some survivors’ incomes. Post-Black Death wage research therefore cautions against treating aggregate output, population, and welfare per survivor as interchangeable recovery measures. [Cambridge University Press](https://www.cambridge.org/core/journals/european-review-of-economic-history/article/abs/black-death-and-the-origins-of-the-great-divergence-across-europe-13001600/CD25EC53A0EF219087EE5E8F5AB8F275)

### 3.2 Regional institutional and ecological contrasts

**Mesopotamia.** Agricultural obligations could explicitly distinguish fixed rents, harvest shares, and crop-failure contingencies. This supports implementing contractual risk allocation early, without requiring a modern banking system. [Avalon Project](https://avalon.law.yale.edu/ancient/hamcode.asp)

**China.** Yuan paper currency and Qing civilian granaries illustrate different capacities: monetary administration and food-risk management. Granary effectiveness depended on regional organization and incentives, not merely the presence of a “granary building.” Avoid making either paper money or substantial state relief uniquely European or modern achievements. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13305)

**South Asia.** Monsoon-related spatial covariance matters greatly. A nearby settlement may be commercially connected yet suffer the same drought. Mughal research also links famine periods to migration, fiscal difficulty, and changing political relationships, although documentary chronologies do not identify a simple, uncontested causal chain. [Nature](https://www.nature.com/articles/s41612-021-00219-1)

**Japan.** Regional hazards differed: cold conditions were particularly important in the northeast, drought in western regions, and floods affected wider areas. Political and social arrangements mediated these hazards. Give crops and regions different weather-response functions rather than a single national “rice failure” probability. [The Asia-Pacific Journal](https://apjjf.org/2016/14/selden-1)

**Southeast Africa.** Historical work emphasizes varied responses mediated by food systems, livelihood assets, social networks, ritual, and political power. Evidence is insufficient to attribute every major settlement or state transformation to climate. TCE should permit migration, changes in livelihood, and political reorganization—not only mortality or technological rescue. [White Rose Research Online](https://eprints.whiterose.ac.uk/id/eprint/97803/)

**The Andes and southwestern North America.** Inka storage and staple finance demonstrate that serious provisioning and fiscal systems need not revolve around money markets. Archaeological household models of ancestral Pueblo communities demonstrate how spatially varying agricultural potential can influence settlement movement and population. These are distinct systems and should not be combined into one generic “American” economic package. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/203249)

The appropriate world-generation inputs are therefore crop portfolios, transport geography, property rights, reserve institutions, obligations, and enforcement—not regional personality modifiers.

---

## 4. Stylized facts and validation targets

The following are **proposed tests derived from the historical mechanisms**. Numerical anchors refer to the cases in Section 2; they should not become mandatory outcomes for every world.

| Pattern to reproduce | Test for TCE | Failure signal |
| --- | --- | --- |
| Bad harvests are more frequent than famines. | Repeated moderate harvest failures should sometimes be absorbed by reserves, trade, or transfers. | Every negative yield shock causes mass death, or no shock ever does. |
| The same production loss can have very different effects. | Hold weather fixed; vary initial stocks, poverty, fixed obligations, and relief access. | Mortality depends almost entirely on aggregate harvest percentage. |
| Scarcity prices are nonlinear. | Compare identical harvest losses with ample versus nearly exhausted carryover. | Prices respond with the same fixed multiplier regardless of stocks. |
| Food access is unequal. | Compare net food sellers, landless workers, artisans, and favored relief recipients. | Every household consumes the settlement average. |
| Crises affect births and household formation as well as deaths. | Test temporary fertility and marriage responses and their lags; compare with the French demographic anchor. | Population loss consists only of immediate starvation deaths. |
| Trade both buffers and transmits shocks. | Test a local failure, a regional failure, and a blockade using the same network. | Connectivity is always beneficial or always harmful. |
| Credit crises can begin with falling asset prices. | Apply a commodity-price reversal to differently leveraged merchant networks. | Financial crises occur only after food prices rise. |
| Monetary and physical crises differ. | Change the monetary standard without changing harvests, then change harvests without changing money. | Both experiments generate indistinguishable “inflation” and welfare effects. |
| Recovery has multiple speeds. | Track prices, consumption, assets, credit, and population separately. | Everything returns to baseline simultaneously after an event timer expires. |
| Institutional behavior can change after repeated rescue. | Allow agents and officials to revise reserves and risk-taking after interventions. | Relief has no behavioral consequences, or always mechanically destroys self-insurance. |

The stock constraint is especially important. Competitive-storage models can generate asymmetric price movements and occasional sharp upward spikes because inventories cannot fall below zero; they do not require an exogenous “price panic” event to generate every extreme. [IDEAS/RePEc](https://ideas.repec.org/p/nbr/nberwo/3439.html)

### Measurement definitions

Use several crisis indicators rather than one composite score:

**Food stress:** unmet consumption needs, number of affected people, cumulative deficit, stock cover, and seed consumption.

**Livelihood stress:** food purchasing power, employment days, distress asset sales, debt service, and loss of land or tools.

**Financial stress:** missed payments, rejected renewals, write-downs, collateral discounts, and failed intermediaries.

**Demographic stress:** excess deaths, missing births, migration, and household dissolution, recorded separately.

For UI purposes, a possible *design convention* is to flag severe food stress when at least 10% of residents receive under 80% of their modeled requirements for fourteen days. This is **not a historical estimate or an official famine classification**. Keep the underlying measures visible so the label never becomes the mechanism.

### Validation procedure

Calibrate ordinary production and price variability first, then test extremes. A model fitted only to famous catastrophes will likely make normal life implausibly unstable.

Use held-out locations or periods where possible. Compare distributions of shock size, price changes, crisis duration, affected population share, and recovery—not whether a simulation reproduces particular historical dates.

Maintain separate counterfactual tests for weather, institutions, trade links, and credit links. A correlation between neighboring crises is not sufficient evidence that the contagion mechanism is correct.

Finally, audit physical and financial consistency. Food cannot be transferred twice, taxes do not disappear from the economy, a loan creates matching claims and obligations, and debt relief must change the creditor’s position as well as the debtor’s.

---

## 5. Recommended representation for TCE

### 5.1 Agent and institution architecture

Keep **people individual**, but place most economic accounting at the household, enterprise, estate, or institution level.

| Entity | Minimum crisis-relevant state |
| --- | --- |
| Person | Nutritional condition, health, work capacity, household membership, skills, migration options. |
| Household | Food and money stocks, production access, dependents, obligations, assets, assistance relationships. |
| Farm or estate | Crop areas, seed, labor demand, draft power, expected output, storage, rents and taxes. |
| Merchant or partnership | Inventories by location, cargoes, working capital, receivables, guarantees, debt maturities. |
| Lender or bank | Assets, liabilities, liquid funds, collateral valuations, exposure concentrations, funding terms. |
| Government, temple, or relief institution | Treasury, usable food stocks, eligibility rules, commitments, transport capacity, reporting quality. |
| Market and transport node | Offers, transactions, prices, unfilled demand, route capacity, delays, and restrictions. |

This allows one household’s bread purchase to affect its members individually without requiring each child to maintain a separate financial balance sheet.

**Do not enable banking simply because the simulation has reached a date.** Enable relevant financial behavior when agents have created enforceable claims, transferable instruments, deposit relationships, or institutions that fund long-lived assets with short-lived liabilities.

### 5.2 Scheduling and computational simplifications

Use daily updates for food consumption, health, inventory losses, travel, and urgent events. Clear ordinary markets on their actual market schedules. Process debt through a maturity queue instead of scanning every contract every tick.

Aggregate money by currency and issue class rather than individual coins. Aggregate similar contracts by counterparty and maturity window when doing so preserves their legal and financial consequences.

A small number of food categories can suffice initially, but distinguish their growing seasons, storage characteristics, nutritional contribution, and response to shocks. Three nominally different cereals that all fail in the same weather are not three independent sources of resilience.

Retain sparse trade, assistance, and credit networks. Avoid all-pairs household trading and universal information about distant prices or failures.

Where only a city-region is simulated in detail, external hinterlands should have finite stocks, production, demand, and transport links. Otherwise “imports” become an unlimited rescue mechanism that conceals flaws in the local economy.

### 5.3 Institutional decisions should be endogenous

Governments and other institutions should respond to observations available to them: price reports, requests for assistance, tax arrears, visible hunger, creditor complaints, and political pressure.

Their actions should depend on reserves, expected fiscal costs, administrative competence, social obligations, factional interests, and beliefs. Officials can underestimate shortages, favor particular groups, refuse costly intervention, or exhaust their reserves protecting a politically important city.

Do not give them perfect knowledge of true regional food availability or future harvests. Equally, do not force them to be irrational to produce crises: an individually defensible decision can impose costs on another settlement or social group.

### 5.4 Existing models worth adapting

| Model or research program | Reusable component | Limitation |
| --- | --- | --- |
| **Deaton and Laroque, “On the Behaviour of Commodity Prices” (1992)** | Nonnegative storage, intertemporal arbitrage, nonlinear price responses. | Not a complete household famine model; income distribution and institutions need adding. [Professor Sir Angus Deaton](https://deaton.scholar.princeton.edu/publications/behaviour-commodity-prices) |
| **Schnabel and Shin, “Liquidity and Contagion: The Crisis of 1763” (2004)** | Leverage, connected credit, asset sales, and liquidity contagion. | A specialized financial setting, not a template for every agrarian settlement. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1162/1542476042813887) |
| **Eisenberg and Noe, “Systemic Risk in Financial Systems” (2001)** | Consistent clearing of interconnected obligations and default cascades. | Adapt debt priority, legal liability, and timing; do not silently impose modern institutional assumptions. [PubsOnLine](https://pubsonline.informs.org/doi/10.1287/mnsc.47.2.236.9835) |
| **Axtell and colleagues, Long House Valley model (2002)** | Household settlement choices linked to spatial agricultural potential and nutritional constraints. | Reproduction of one archaeological setting does not establish a universal explanation of societal collapse. [Johns Hopkins University](https://pure.johnshopkins.edu/en/publications/population-growth-and-collapse-in-a-multiagent-model-of-the-kayen-7/) |
| **Aktipis, Cronk, and de Aguiar, Maasai risk-pooling model (2011)** | Assistance requested according to need and provided according to capacity; effects on herd survival. | An ethnographically informed pastoral model, not an ancient famine-frequency dataset. [Arizona State University](https://asu.elsevierpure.com/en/publications/risk-pooling-and-herd-survival-an-agent-based-model-of-a-maasai-g/) |

**Recommended build order:** first make harvests, household access, stocks, obligations, and transport work together. Add monetary plurality and credit-network detail afterward. Otherwise elaborate banking mechanics may coexist with an unrealistic food economy that either never experiences scarcity or collapses every few harvests.

---

## 6. Sources, datasets, and limits of the evidence

### 6.1 Data suitable for calibration

| Source | Useful material | Main caution |
| --- | --- | --- |
| **Medieval Crop Yields Database**, British Agricultural History Society | More than **34,000** dated yield observations, approximately **31,000** for grain; useful for crop covariance and harvest variation. | Predominantly demesne records, uneven geography and coverage; distinguish gross harvest from output net of seed. [British Association for Holistic Science](https://www.bahs.org.uk/crop-yields-database/the-data/) |
| **Allen–Unger Global Commodity Prices Database** | Commodity-price series from the late Middle Ages onward, including locations beyond Europe; original and standardized monetary measures. | Local units, quality, market type, and currency conversions require inspection. [ScienceDirect](https://www.sciencedirect.com/org/science/article/pii/S2452366623000269) |
| **Alfani and Ó Gráda, “The timing and causes of famines in Europe” (2018)** | Famine chronology covering approximately **1250–2017**, useful for clustering and changing incidence. | European coverage; event counts depend on geographic units, definitions, and documentary visibility. [Nature](https://www.nature.com/articles/s41893-018-0078-0) |
| **NOAA drought-atlas collections** | Monsoon Asia, Old World, and other regional reconstructions for spatially correlated environmental stress. | Drought indices are not crop yields, and proxy coverage and uncertainty vary. [NCEI](https://www.ncei.noaa.gov/products/paleoclimatology/drought-variability) |
| **Guan, Palma, and Wu’s Yuan-China replication material** | Money issues, prices, warfare, taxation, disasters, and population; underlying series cover **1260–1355**. | Some variables are reconstructed; estimated money stocks are not independently observed balances. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13305) |
| **Quinn and Roberds’ 1763 study and appendices** | Bank balances, payments, exchange conditions, and crisis chronology. | A particularly sophisticated commercial-financial center, not representative of all pre-industrial credit. [FRASER](https://fraser.stlouisfed.org/files/docs/historical/frbatl/wp/frbatl_wp_2012-08.pdf) |
| **Claridge and Langdon’s storage work; Brunt and Cannon’s farm records** | Physical storage arrangements and actual storage/sales behavior. The medieval study examines **315 cases in 97 communities**. | Administrative storage and later commercial-farm storage are different samples; neither directly gives a universal household reserve distribution. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1468-0289.2010.00564.x) |

For broader interpretation, especially useful starting works are Ó Gráda’s **“Making Famine History” (2007)**, Will and Wong’s **Nourish the People (1991)**, and the famine, monetary, and financial studies identified throughout this report. They address different layers of the problem rather than supplying interchangeable crisis parameters. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjel.45.1.5)

### 6.2 Contested claims and thin evidence

**Famine causes are not a choice between “nature” and “society.”** Production shocks, purchasing power, warfare, and institutional decisions can be jointly necessary. Claims about a European transition around 1710 are interpretations of a particular chronology, not a global technological boundary. [Nature](https://www.nature.com/articles/s41893-018-0078-0)

**Mortality, population decline, and missing population are different quantities.** Deaths, reduced births, and migration can all affect population comparisons. For example, the Finnish literature describes a roughly 10% famine-era population shortfall involving both mortality and reduced births; treating that figure as a directly measured starvation death rate would be wrong. [onlinelibrary.wiley.com](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13095?utm_source=chatgpt.com)

**Historical “storage costs” are not necessarily physical spoilage.** Financing, buildings, handling, risk, and quality changes are separate components. Direct records also show that drying could improve the sale value of stored wheat. Do not infer kilograms lost directly from the seasonal increase in grain prices. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.13133)

**Single-cause financial stories are especially hazardous.** Surviving chronicles may exaggerate royal debts or simplify intertwined commercial failures. Treat famous episodes as tests of mechanisms and exposure structures, not as evidence for scripted triggers. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/new-look-at-the-dealings-of-the-bardi-and-peruzzi-with-edward-iii/2AD04842FACDAE9D0A660EDCD72EC097)

**Numerical evidence is uneven across regions.** European price and estate records are unusually extensive. Archaeology, administrative histories, and oral evidence elsewhere often identify mechanisms more securely than annual probabilities or precise death totals. The correct response is to preserve uncertainty and test parameter ranges—not to fill every gap with an English default. [British Association for Holistic Science](https://www.bahs.org.uk/crop-yields-database/the-data/)

---

## Bottom line for TCE

The most useful crisis engine is a set of **finite resources, unequal claims, dated obligations, constrained networks, and fallible institutional decisions**.

With those systems connected, the simulation can generate a localized hunger episode that trade resolves; a famine despite continued market activity; a monetary disturbance without food scarcity; a merchant collapse that freezes distant credit; a relief effort that succeeds but depletes future protection; or a recovery that restores output while permanently changing ownership and settlement patterns.

Crises then become consequences of what people and institutions have built—and recovery becomes something they must actually accomplish.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928ab-2164-83ea-a0c3-8f25d21c029a)
