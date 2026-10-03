# Poor relief, charity and welfare

## A simulation-ready report for The Civilization Engine

**The central design recommendation is to model relief as claims on real food, income, shelter and care—not as a policy that directly subtracts famine deaths or unrest.** Outcomes should depend on who qualifies, who receives assistance, whether supplies reach them, and whether the assistance addresses their actual needs.

Three findings are especially important for TCE. Informal support can exist without a state bureaucracy. Substantial formal relief existed before industrialization. And assistance does not have a universal negative effect on work: the strongest modern evidence rejects that simple assumption. [Human Generosity Project](https://www.humangenerosity.org/what-are-need-based-transfers/)

The report below distinguishes **historical observations**, **estimated intervention effects**, and **proposed simulation parameters**. They serve different purposes: historical spending helps bound plausible institutions, whereas intervention effects are better used as validation checks than as universal coefficients.

---

## 1. Mechanisms: implementable causal rules

### 1.1 Poverty is a household resource shortfall, not simply unemployment

Begin with households that have obligations and resources. Someone can be employed but unable to support several dependants; a household can possess land but lack food before harvest; an elderly person can have money but nobody available to provide care.

Research among Shiwiar forager-horticulturalists documents the importance of support during illness and injury. Research among Tsimane communities identifies exchanges of food, labor, childcare and sick care—not merely cash-equivalent transfers. These findings favor a multidimensional support system. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/ajpa.10325)

**Implementation.** For a specified accounting period, such as one week, calculate:

\[
D\_h=\max(0,M\_h-R\_h)
\]

Here, \(M\_h\) is the cost of household \(h\)’s minimum consumption basket and \(R\_h\) is its accessible disposable resources, including food already owned. Do not count productive assets as spendable income until the household actually sells them.

Track **unmet care hours separately**. Money and grain are imperfect substitutes for nursing, supervision, cooking, water collection and assistance with movement.

Assess need at household level, but allocate consumption and care to individuals. Otherwise, a household-average calorie measure can conceal deprivation among children, dependants or disfavored members.

### 1.2 Kin and neighbors provide insurance—but correlated disasters can overwhelm it

Support can follow several distinct principles: helping relatives, reciprocating earlier assistance, responding to immediate need, honoring social obligations, or maintaining a valued relationship. Need-based transfers should not automatically create collectible debts. The Human Generosity Project distinguishes such transfers from debt-based exchange in settings including Maasai pastoralism. [Human Generosity Project](https://www.humangenerosity.org/what-are-need-based-transfers/)

**Implementation.** A household requests assistance from reachable social contacts. Each potential donor evaluates the request against its own protected consumption reserve:

\[
T\_{ih}\leq \min(D\_h,S\_i)
\]

where \(S\_i\) is the donor’s available surplus after its reserve. Willingness determines how much of this feasible transfer occurs.

Store gifts, loans and reciprocal obligations as different relationship events. A loan can produce later repayment demands; a gift can produce gratitude or reputational expectations without becoming a financial liability.

**The crucial failure mode is shared exposure.** A broken leg affects one household; a drought can affect every household in its support network. A Maasai-inspired agent-based model finds that transfer rules and shock structure change network survival. That is a useful mechanism, but its simulated survival rates are not historical mortality estimates. [Springer](https://link.springer.com/article/10.1007/s10745-021-00273-6)

For TCE, geographically dispersed relatives or trading partners should sometimes provide better insurance than additional neighbors exposed to identical weather.

### 1.3 Charity converts wealth, obligations and reputation into assistance

Religious or civic charity can be financed by current donations, mandatory contributions, bequests, or income from endowed property. These funding arrangements behave differently.

Ottoman foundations illustrate the distinction: endowed buildings and property financed continuing services, while foundation documents specified beneficiaries and distributions. Beneficiaries were not always simply “the poorest”; staff, students and travelers could have explicit claims. [OpenEdition Journals](https://journals.openedition.org/beo/907?lang=en)

**Implementation.** Separate an institution’s **capital assets** from its **spendable income**:

\[
\text{available funds}
=\text{opening cash}+\text{donations}+\text{rents}+\text{subsidies}
\]

An endowed field generates rent or produce; it does not replenish the treasury magically. Harvest failure, tenant default, confiscation or asset destruction can interrupt the foundation’s income.

Donation decisions can depend on donor resources, perceived need, religious obligations, public recognition and trust in administrators. This permits both sincere generosity and status-seeking philanthropy without assigning every donor the same motive.

### 1.4 Eligibility, take-up, approval and delivery are separate processes

A person can need assistance without qualifying; qualify without knowing it; apply and be rejected; or be approved but receive nothing.

England’s post-1834 reforms demonstrate that access conditions were substantive policy instruments. The workhouse test deliberately made assistance unattractive, while restrictions on support outside institutions sought to reduce claims and costs. Nevertheless, support outside workhouses continued for many sick and elderly people. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70035)

**Implementation.** Give each institution rules for residence, membership, income, assets, age, incapacity and work obligations. Then process actual applications.

For needy people, effective coverage can be decomposed as:

\[
C=
P(\text{eligible}\mid\text{needy})
P(\text{apply}\mid\text{eligible, needy})
P(\text{approved}\mid\text{apply})
P(\text{delivered}\mid\text{approved})
\]

These are conditional probabilities, not assumed-independent multipliers.

Keep **adequacy** separate from coverage: receiving one small meal counts as receipt, not as elimination of need. Also distinguish a discretionary gift from a legal entitlement. An unpaid entitlement can create arrears, complaints and resentment.

### 1.5 Public granaries combine storage, logistics and political allocation

Public granaries can stabilize prices, lend seed or consumption grain, sell at subsidized prices, or distribute food freely. These are different interventions.

In Qing China, local reserves operated alongside centrally directed disaster assistance. Shiue finds patterns consistent with local storage incentives being weakened where central rescue was more available, although this is historical observational evidence rather than an experiment. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/local-granaries-and-central-government-disaster-relief-moral-hazard-and-intergovernmental-finance-in-eighteenth-and-nineteenthcentury-china/80F2E2CEBBEA518A60D40E2116312898)

**Implementation.** Maintain a physical inventory:

\[
S\_{t+1}=S\_t+\text{purchases}+\text{repayments}+\text{arrivals}
-\text{grants}-\text{sales}-\text{departures}-\text{losses}
\]

Grain loans create receivables; subsidized sales recover some money; free grants do neither. Include storage deterioration, stock rotation, transport capacity, delivery delays and competing claims.

Do not label all state food stocks “welfare.” Inka storage research emphasizes the mobilization of staples for the political economy of the state, including administrative and labor requirements. The existence of warehouses does not establish universal poor relief. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/203249)

### 1.6 Cash helps only through the goods and services it can command

Sen’s entitlement approach distinguishes total food availability from people’s ability to acquire food through production, employment, exchange and transfers. It does not imply that aggregate shortages are irrelevant. [OUP Academic](https://academic.oup.com/book/32827)

**Implementation.** A cash grant increases purchasing power, not the food inventory. Its consequences depend on market response.

With accessible imports, additional demand can attract supplies. With a closed market and fixed stocks, it mostly changes who can purchase the remaining food and may increase prices. Nonrecipients can consequently become worse off.

Food distribution transfers a physical resource but still requires procurement. Buying the entire relief ration from the same shortage-stricken market can displace other purchasers unless additional supplies arrive.

Tax remission and debt postponement should also exist, but they are not equivalent to food grants: someone who owes no tax or possesses no harvest may obtain little immediate benefit from them.

### 1.7 Relief works have both a transfer side and a production side

Public employment supplies earnings and potentially useful infrastructure. It also consumes time, energy, materials and supervision.

Evidence from India shows why a single “work incentive” coefficient is inadequate. Imbert and Papp find that public employment displaced some private employment and raised private wages. A different experiment improving program implementation found increases in both private earnings and employment through broader market effects. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.20130401)

**Implementation.** A worker compares the offer with alternatives:

\[
\text{net household benefit}
=\text{payment}
-\text{foregone earnings}
-\text{travel and care costs}
-\text{extra food requirement}
\]

Record the resulting infrastructure separately from the wage transfer. A road is useful only if the work, materials and maintenance actually occur.

An employment-only relief system should fail to protect people who cannot work unless it includes another channel. Caregivers may also be unable to attend even when physically healthy.

For means-tested grants, calculate the marginal return to earnings explicitly. A benefit withdrawn dollar-for-dollar can reduce financial incentives to work, but improved nutrition, liquidity and health can work in the opposite direction.

### 1.8 Famine prevention requires protecting consumption and recovery capacity

Famine scholarship treats mortality as more than immediate starvation: food access, disease, displacement and public action interact. Consequently, expenditure alone is a poor predictor of deaths prevented. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjel.45.1.5)

**Implementation.** Relief should influence the existing physiology and disease systems through actual intake, care and exposure. It should not directly modify a “famine mortality” percentage.

Useful pathways include preventing prolonged energy deficits, permitting rest during illness, maintaining shelter and hygiene, and reducing desperate migration. Longer-term protection also matters: a household that keeps seed, tools or breeding animals may recover next season; one that sells everything to survive may remain dependent.

Keep emergency survival and recovery assistance distinct. Seed grain cannot feed a family and be planted at the same time.

For elderly agents, determine work and care capacity from health and abilities rather than making everyone unproductive at one age. Institutional pension eligibility can nevertheless use an age threshold.

### 1.9 Relief changes unrest through several channels, not a universal pacification bonus

Historical research associates poor relief with food-riot patterns, but estimated relationships vary across periods and identification is difficult. Stronger recent evidence links English welfare cuts to increased property crime, particularly during the agricultural off-season; that is not equivalent to an estimated effect on rebellion. [IDEAS/RePEc](https://ideas.repec.org/p/iza/izadps/dp7398.html)

**Implementation.** Separate:

* **Material distress:** hunger, homelessness, debt and inability to support dependants.
* **Perceived injustice:** exclusion, favoritism, broken promises and unequal burdens.
* **Collective action capacity:** organization, information, leadership and physical ability to participate.

Relief can reduce desperation while increasing resentment among excluded neighbors or heavily taxed contributors. Extremely weak people may become less capable of collective action even as deprivation worsens. Model that possibility rather than making riot probability increase indefinitely with hunger.

### 1.10 Political motives determine whose suffering receives attention

Relief can express moral obligation, religious merit, civic solidarity and genuine compassion. It can also preserve workers and taxpayers, reward a constituency, demonstrate sovereignty or prevent disorder.

Augustus’s own account publicized distributions to large groups of Roman recipients, illustrating the political visibility of provision. Germany’s introduction of old-age insurance likewise combined welfare objectives with concern about worker loyalty and socialism. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Augustus/Res_Gestae/3%2A.html)

**Implementation.** Let decision-makers weigh social obligations, constituency pressure, fiscal cost, labor needs and reputational consequences. Information matters: suffering that reaches councils, courts, religious leaders or public debate should affect decisions differently from suffering that remains unseen.

Do not make “democracy” automatically prevent famine. Represent the underlying channels—public information, opposition, accountability and pressure—which Drèze and Sen emphasize in their analysis of Indian famine prevention. [OUP Academic](https://academic.oup.com/book/2070/chapter/141987885)

---

## 2. Parameters: historical anchors and quantitative evidence

**Confidence labels:** **High** means well-defined within the documented setting; **Moderate** means reconstructed, context-sensitive or dependent on identification assumptions; **Low** means sparse or weakly transferable evidence. High confidence in a documented rule does not imply high confidence in its actual enforcement.

### 2.1 Spending, coverage and institutional scale

| Quantity and setting | Value, units and date | Appropriate use in TCE | Confidence and source |
| --- | --- | --- | --- |
| Formalized relief, selected European cases | Western Netherlands, c.1760: **2.9–3.3% of GDP**; England, c.1790: **2.1–2.3%**; England, c.1850: **1.2–1.5%**, annually | Bounds for substantial preindustrial provision, not a universal historical average | **Moderate–low:** reconstructions contain interpolations and estimates. Van Bavel & Rijpma. [ResearchGate](https://www.researchgate.net/publication/279519651_How_important_were_formalized_charity_and_social_spending_before_the_rise_of_the_welfare_state_A_long-run_analysis_of_selected_western_European_cases_1400-1850) |
| Subsistence-equivalent capacity, western Netherlands | Spending equivalent to supporting **8.9–9.9% of the population**, c.1760 | Check the relationship between expenditure and supportable consumption | **Moderate–low:** **not an observed recipient headcount**. [ResearchGate](https://www.researchgate.net/publication/279519651_How_important_were_formalized_charity_and_social_spending_before_the_rise_of_the_welfare_state_A_long-run_analysis_of_selected_western_European_cases_1400-1850) |
| Qing granary stock | Approximately **1.5 billion liters of husked-rice equivalent**, or **7.5 L/person**, in the 1750s; roughly **3% of annual adult consumption** | Physical reserve calibration | **Moderate:** reconstructed aggregate stock, not annual relief expenditure. Shiue. [ResearchGate](https://www.researchgate.net/publication/231743676_Local_Granaries_and_Central_Government_Disaster_Relief_Moral_Hazard_and_Intergovernmental_Finance_in_Eighteenth-_and_Nineteenth-Century_China) |
| Ottoman institutional feeding | Süleyman’s Damascus foundation reportedly fed approximately **500–600 people/day**, following its establishment in 1554–60 | Scale of a major staffed kitchen | **Moderate:** institutional case, not population-wide coverage or annual unique recipients. Boqvist. [OpenEdition Journals](https://journals.openedition.org/beo/907?lang=en) |
| Zakat on qualifying monetary wealth | Common rule: **2.5% per lunar year** on qualifying wealth above the relevant threshold and holding period | An authored contribution rule | **High for this rule; low for historical collection:** not 2.5% of GDP, income or every asset category. [Islamic Relief Worldwide](https://islamic-relief.org/news/zakat/) |
| Early statutory old-age insurance | German scheme introduced in **1889**, pension age **70**; lowered to **65 in 1916** | Eligibility thresholds can be institution-specific | **High:** legal-history benchmark, not the age at which people ceased working. [Social Security Administration](https://www.ssa.gov/history/age65.html) |
| Modern noncontributory safety-net spending | Approximately **1.5% of GDP annually**, average across developing and transition-country observations in the World Bank’s **2018** report | Modern assistance benchmark | **Moderate:** heterogeneous programs and observation years; not all social spending. [World Bank](https://www.worldbank.org/en/topic/socialprotectionandjobs/publication/the-state-of-social-safety-nets-2018) |
| Modern broad social protection | **12.9% of GDP**, excluding health; **52.4% of people** covered by at least one benefit, principally **2023** observations | Upper institutional-complexity comparison | **High for reported definitions:** includes much more than poverty relief; coverage does not establish adequacy. ILO 2024–26 report. [International Labour Organization](https://www.ilo.org/resource/news/social-protection-plays-key-role-countering-climate-change-impact-countries) |
| Emergency food planning reference | **2,100 kcal/person/day** for a standard general-assistance basket | Population-level ration planning benchmark | **High as a planning reference**, not an identical biological requirement for every agent. WFP. [World Food Programme](https://www.wfp.org/news/critical-funding-shortage-forces-wfp-slash-food-rations-refugees-tanzania) |

The European estimates do not capture all informal family support. Their ranges are not statistical confidence intervals. They should not be interpreted as showing that societies with less recorded spending necessarily provided less total assistance. [ResearchGate](https://www.researchgate.net/publication/279519651_How_important_were_formalized_charity_and_social_spending_before_the_rise_of_the_welfare_state_A_long-run_analysis_of_selected_western_European_cases_1400-1850)

**Stock-to-duration conversion.** A reserve equal to 3% of annual consumption corresponds arithmetically to about **11 population-wide consumption-days**, or approximately **110 days for 10% of the population**, before losses and transport constraints. A seemingly small national reserve can therefore sustain a targeted minority for months; it cannot feed everybody through a prolonged failure.

### 2.2 Effects: validation targets, not portable multipliers

| Study or episode | Quantitative finding | Interpretation and confidence |
| --- | --- | --- |
| England and Wales after the 1834 reforms; Green et al., 2026 | Estimated cuts imply **8–10% higher mortality at ages 1–4** and **2–4% lower rural life expectancy at birth** | **Moderate:** historical reconstruction and identification assumptions. Relative changes, not percentage points; not a famine-only estimate. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70035) |
| New Poor Law and crime; Melander & Miotto, 2023 | Estimated **2,700 additional property crimes/year**, approximately **17.2% above prereform levels** | **Moderate:** particularly associated with winter vulnerability. Do not apply this percentage to riots or civil war. [University of Warwick](https://warwick.ac.uk/fac/soc/economics/research/centres/cage/news/03-07-25-the_impact_of_welfare_reforms__lessons_from_history/) |
| Cash-transfer evaluations; Banerjee et al., 2017 | **Seven randomized evaluations in six countries** found no systematic discouragement of work | **High within studied programs; limited historical transferability.** This does not establish an exactly zero effect for every transfer design. [OUP Academic](https://academic.oup.com/wbro/article/32/2/155/4098285) |
| Improved Indian public-employment implementation; Muralidharan et al., 2023 | Beneficiary household earnings **+14%**; poverty **−26% relative**; **86% of income gains** came from nonprogram income | **High for the experimental setting**, not a general effect of creating public employment. Includes market spillovers. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.3982/ECTA18181) |
| Maharashtra drought relief, 1972–73 | Nearly **5 million workers** at peak; almost **1 billion person-days** during the twelve months before July 1973 | **Moderate:** large-scale historical case, not randomized evidence. Employment counts do not equal total household beneficiaries. [OUP Academic](https://academic.oup.com/book/2070/chapter/141987885) |
| Universal famine mortality or unrest elasticity | **No defensible single value** from this evidence | Leave uncalibrated rather than inventing “each 1% of GDP prevents X% of deaths.” |

### 2.3 Proposed initial sensitivity ranges

These are **design scenarios**, not claimed historical estimates. They are intended to expose which mechanisms matter before attempting narrower calibration.

| Parameter | Initial values or range | Units and purpose | Evidential status |
| --- | --- | --- | --- |
| Annual formal relief budget | **0, 0.5, 1, 2, 3%** | Share of annual output; test absent through substantial provision | Designer choices, loosely bounded by historical cases |
| Intended shortfall replacement | **25%, 50%, 100%** | Fraction of the assessed minimum-basket deficit | Designer choices; actual delivery remains resource-constrained |
| Opening emergency reserve | **30, 90, 180** | Ration-days for the intended recipient population | Designer choices, not population-wide days |
| Administrative response lag | **0, 7, 30** | Days from recognized need to authorization | Designer choices; add physical transport time separately |
| Abstracted application take-up | **50%, 80%, 100%** | Eligible needy households applying, where behavior is not modeled individually | Sensitivity assumptions, not cross-cultural estimates |
| Benefit withdrawal rate | **0, 0.5, 1.0** | Benefit reduction per additional unit of earnings | Authored policy alternatives |
| Protected donor reserve | **0, 7, 30** | Days of donor-household consumption, bounded by actual storage | Designer choices; storage-poor societies cannot hold imaginary reserves |
| Shared-shock correlation | **0, 0.5, 0.9** | Correlation among support-network members’ shocks | Experimental stress tests; ideally derived from geography and livelihoods |

**Worked TCE stock example.** Suppose a settlement has 10,000 people and intends to issue 0.5 kg of grain daily to 20% of them for 90 days:

\[
10{,}000\times0.20\times90\times0.5
=90{,}000\text{ kg}
\]

It must deliver **90 tonnes**. With an assumed 10% combined loss before receipt, it needs **100 tonnes initially**. This is a grain component, not a complete diet. Both ration size and loss rate here are illustrative assumptions.

---

## 3. Variation across eras and regions

The following are **institutional configurations**, not a mandatory progression. Kin support, charitable foundations, contributory funds and public relief can coexist.

| Setting | Characteristic support arrangements | Implication for TCE |
| --- | --- | --- |
| **Foraging and mixed forager-horticultural communities: East Africa and Amazonia** | Food sharing and exchanges of labor and care operate through social relationships, including but not limited to kin. Contemporary ethnography documents these arrangements. [Human Generosity Project](https://www.humangenerosity.org/what-are-need-based-transfers/) | Model repeated interactions and household vulnerability. Do not treat contemporary communities as unchanged replicas of prehistory. |
| **Early farming: the Levant** | Archaeological granaries at Dhraʿ demonstrate substantial storage around 11,000 years ago, including before fully domesticated agriculture. Storage architecture alone does not identify rights to emergency distributions. [DOI](https://doi.org/10.1073%2Fpnas.0812764106) | Storage technology enables pooling, but ownership and eligibility must emerge separately. “Granary built” must not automatically mean “public welfare established.” |
| **Ancient urban states: Rome** | Augustus reported slightly more than **200,000 public-grain recipients** in connection with a distribution in 2 BCE. This is a politically presented urban recipient count, not empire-wide poverty coverage. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Augustus/Res_Gestae/3%2A.html) | Allow status-based provisioning of a politically important population, rather than assuming all food subsidies are means-tested. |
| **Andean agrarian empire** | Inka storage was embedded in staple finance and state mobilization. Its administrative and political functions cannot be reduced to charity. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/203249) | Soldiers, officials, work crews and distressed households can compete for the same public stores. |
| **Late imperial China** | Local and central authorities combined reserves and disaster assistance; incentives between levels of government affected provision. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/local-granaries-and-central-government-disaster-relief-moral-hazard-and-intergovernmental-finance-in-eighteenth-and-nineteenthcentury-china/80F2E2CEBBEA518A60D40E2116312898) | Give districts their own stocks and incentives. Central rescue can supplement local provision while potentially weakening preparation. |
| **Islamic and Ottoman settings** | Zakat, voluntary alms and endowed foundations are distinct institutions. Ottoman kitchens could prioritize named beneficiaries, with access shaped by foundation rules rather than need alone. [Islamic Relief Worldwide](https://islamic-relief.org/news/zakat/) | Separate religious contribution rules, voluntary giving and asset-funded services. Avoid a single generic “religious charity” percentage. |
| **South Asian religious provision** | Sikh langar provides a community-kitchen model rooted in service and shared provision. Its resources include donated food and volunteer labor. [OUP Academic](https://academic.oup.com/edited-volume/34369/chapter-abstract/291508358) | A kitchen can be broadly accessible yet constrained by cooks, fuel, utensils and donations. |
| **Preindustrial Europe** | Church, civic, endowed and tax-funded provision supported people through institutions and assistance at home. The balance varied and provision did not rise monotonically. [ResearchGate](https://www.researchgate.net/publication/279519651_How_important_were_formalized_charity_and_social_spending_before_the_rise_of_the_welfare_state_A_long-run_analysis_of_selected_western_European_cases_1400-1850) | Institutions can expand, contract, merge or lose their funding without a change of technological “era.” |
| **Industrial societies** | Contributory social insurance introduced eligibility based on employment, contributions and age, alongside existing assistance. Germany’s early pension system illustrates this distinct logic. [Social Security Administration](https://www.ssa.gov/history/age65.html) | A contributor with an entitlement and a destitute noncontributor are different cases. Insurance need not reach the poorest automatically. |
| **Modern famine prevention and welfare** | Public employment, transfers and broader social protection coexist with household support. Comparative famine-prevention research includes India and African cases such as Botswana and Cape Verde. [OUP Academic](https://academic.oup.com/book/2070/chapter/141987885) | Model administrative capacity, imports and public pressure, not a modernity flag that abolishes famine. |

**Regional evidence is uneven.** Comparable national spending estimates are much easier to obtain for some European and modern states than for early agrarian societies or informal support networks. That is a documentation difference, not proof that welfare mattered only where accounting records survive.

---

## 4. Stylized facts a credible simulation should reproduce

### Small institutions can matter greatly without covering everyone

The supportable-population estimates above show why modest aggregate budgets can be important to a vulnerable minority. In TCE, a relief fund should be capable of substantially reducing deprivation while leaving most residents as nonrecipients. Equally, broad nominal coverage can coexist with inadequate benefits. The ILO’s “at least one benefit” indicator should never be interpreted as comprehensive protection. [ResearchGate](https://www.researchgate.net/publication/279519651_How_important_were_formalized_charity_and_social_spending_before_the_rise_of_the_welfare_state_A_long-run_analysis_of_selected_western_European_cases_1400-1850)

### Food production can collapse without producing proportionate famine mortality

Maharashtra provides an integrated test case. Drèze and Sen report that 1972–73 cereal production was only **47% of its 1967–68 level**. Public employment sustained purchasing power, assistance also served people unable to work, and private imports helped replenish supplies. They interpret the response as averting famine despite substantial hardship—not as proof that nobody suffered or that every excess death was prevented. [OUP Academic](https://academic.oup.com/book/2070/chapter/141987885)

A TCE reconstruction should therefore require **earnings or grants, reachable markets and incoming food together**. Cash alone in a sealed settlement should not reproduce that result.

### Household misfortune and regional catastrophe require different insurance scales

Need-based support networks should perform better against isolated losses than against synchronized losses. The same generous community can cope well with illness and fail during a widespread harvest shock. Network-model results support testing this distinction explicitly. [Springer](https://link.springer.com/article/10.1007/s10745-021-00273-6)

### Work responses should differ by recipient, program and labor market

A recovering sick worker may work more after assistance; a parent may spend more time caring for a child; a public worker may leave private employment; higher reservation wages may alter employers’ behavior. Modern evidence includes both private-employment displacement and positive broader earnings effects. A universal “welfare causes idleness” rule would fail these comparisons. [OUP Academic](https://academic.oup.com/wbro/article/32/2/155/4098285)

### Retrenchment can reduce expenditure while worsening outcomes

English evidence links relief cuts to worse childhood health and more property crime. Separately, Clark and Page find no evidence for the significant economic gains often attributed to the 1834 reform. Spending reduction therefore cannot be the simulation’s sole measure of institutional success. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70035)

### Timing and distribution should matter as much as the nominal budget

As a **model implication**, two otherwise identical settlements spending the same amount should have different outcomes when one intervenes before households exhaust food and productive assets, while the other intervenes after prolonged deprivation. Likewise, two identical granaries should yield different protection when one serves nearby healthy claimants and the other has delivery arrangements for distant or incapacitated residents.

---

## 5. Modeling recommendation for individual agents and institutions

### 5.1 Use households for claims, individuals for consequences

A compact representation is sufficient:

| Entity | State worth retaining |
| --- | --- |
| **Person** | Health and work capacity; actual intake; care needs; household; memberships and legal status; knowledge of assistance; trust and grievance |
| **Household** | Food, income, liquid assets, productive assets, debt, dependants, care capacity, consumption shortfall and support-network links |
| **Relief institution** | Treasury, stocks, endowed assets, funding sources, staff, delivery capacity, jurisdiction, eligibility rules, benefit formula, claim queue and rationing policy |
| **Relief claim** | Applicant, assessed need, requested resource, legal basis, application date, decision, promised amount, delivered amount and reason for rejection or delay |

This avoids creating a separate “poor population” abstraction disconnected from TCE’s persistent people. Households move into and out of need as births, illness, aging, employment, harvests and asset losses occur.

### 5.2 Build institutions from reusable policy components

Use a small set of composable elements:

**Funding:** donations, earmarked taxes, general revenue, endowment rents or member contributions.

**Assistance:** cash, grain, prepared meals, shelter, care hours, paid work, loans or tax remission.

**Eligibility:** assessed need, residence, membership, contribution history, age, incapacity or legal status.

**Allocation:** equal rationing, proportional shortfall coverage, priority to severe need, first-come service or politically favored categories.

**Conditionality:** none, work attendance, entry into an institution, or another authored requirement.

These combinations can generate kin funds, religious kitchens, municipal relief, grain banks, occupational insurance and pensions without hard-coding particular civilizations.

### 5.3 Separate decisions from physical execution

A recommended update cycle is:

```
Assess household food and care shortfalls.
Collect support requests and institutional claims.
Allocate donor surpluses and institutional resources.
Authorize payments, distributions, care visits and work offers.
Execute purchases, transport, cooking, care and employment.
Record actual individual consumption and received services.
Update health, assets, obligations, trust and unmet claims.
```

Collect competing requests before allocating scarce resources. Otherwise, simulation iteration order can accidentally determine who survives.

Daily consumption and health continue daily. Routine claim reassessment can be weekly; reserves and budgets can be reviewed seasonally. Emergency triggers should permit faster reassessment. These schedules are engineering choices, not historical findings.

Use ordinary people and logistics for delivery: kitchen workers cook, carts transport grain, caregivers travel, and recipients spend time collecting benefits. Visible daily life then expresses the welfare system rather than merely decorating it.

### 5.4 Keep the computation sparse

Use bounded active support networks rather than checking every household against every other household. Index institutions by jurisdiction and membership, and reassess claims when relevant circumstances change.

The intended computational structure is approximately linear in people, active support links and claims, subject to the rest of the economy and pathfinding systems. That is an architectural objective, not a performance guarantee.

Preserve the details most likely to change outcomes: resource conservation, household composition, access, physical delivery, care burdens and major eligibility exclusions. Initially simplify legal disputes, detailed bookkeeping and individual donor psychology.

### 5.5 Validate with paired counterfactual worlds

Run identical weather, disease and trade shocks under different institutions: informal support only; discretionary charity; targeted grants; reserve releases; and public works combined with assistance for nonworkers.

Measure **person-days below the consumption floor**, unmet care, age-specific excess deaths, distress asset sales, debt, migration, hunger-gap reduction, delivery delays and fiscal cost. Distinguish migration from death and authorized assistance from received assistance.

For labor, record private work, public work, unpaid care and total output separately. For politics, distinguish theft, protest and rebellion.

The decisive question is not whether a welfare setting raises an abstract happiness score. It is whether the institution changes the causal sequence from shock to deprivation, asset loss, illness and political response.

### 5.6 Existing models and games worth borrowing from

**Maasai risk-pooling agent-based models.** Campennì, Cronk and Aktipis’s *Need-Based Transfers Enhance Resilience to Shocks* is directly relevant to network support, alternative transfer rules and correlated shocks. Borrow the experimental structure, not its numerical outcomes as historical constants. [Springer](https://link.springer.com/article/10.1007/s10745-021-00273-6)

**The Survival Game.** This research-linked game models resource sharing and survival in a Maasai-inspired setting. It is useful for making support-network decisions legible to players and for exploring need-based transfers versus repayment expectations. [Scott Claessens](https://scottclaessens.github.io/projects/3_survivalgame/)

**EUROMOD and SOUTHMOD.** Tax-benefit microsimulation provides a useful pattern for encoding eligibility, contributions, benefit withdrawal and household disposable resources. These models do not supply TCE’s dynamic food production, disease or agent behavior; they offer an institutional rules architecture to combine with those systems. [Microsimulation](https://www.microsimulation.pub/articles/00075)

---

## 6. Sources, datasets and limits of the evidence

### Recommended research spine

**Long-run institutional scale:** Van Bavel and Rijpma, *How Important Were Formalized Charity and Social Spending before the Rise of the Welfare State?* supplies reconstructed spending and subsistence-equivalent comparisons. Shiue, *Local Granaries and Central Government Disaster Relief*, supplies the Chinese local–central institutional comparison. [ResearchGate](https://www.researchgate.net/publication/279519651_How_important_were_formalized_charity_and_social_spending_before_the_rise_of_the_welfare_state_A_long-run_analysis_of_selected_western_European_cases_1400-1850)

**Famine mechanisms:** Sen’s *Poverty and Famines*, Drèze and Sen’s *Hunger and Public Action*, and Ó Gráda’s *Making Famine History* provide complementary treatments of food access, public intervention and famine history. Read them together rather than treating either food availability or distribution as a complete explanation. [OUP Academic](https://academic.oup.com/book/32827)

**Effects and contested assumptions:** Banerjee et al. address cash transfers and work; Imbert and Papp and Muralidharan et al. address labor-market effects of public employment; Green et al., Melander and Miotto, and Clark and Page address different consequences of English poor-law reform. Green et al. appears in the **2026** journal issue and was first published online in **July 2025**. [OUP Academic](https://academic.oup.com/wbro/article/32/2/155/4098285)

### Datasets and accounting frameworks

| Resource | Most useful variables | Main limitation |
| --- | --- | --- |
| **World Bank ASPIRE** | Assistance coverage, adequacy, targeting, spending and household-survey incidence | Definitions and survey years vary; accounting-based poverty effects are not necessarily causal estimates. [World Bank](https://www.worldbank.org/en/data/datatopics/aspire/about) |
| **ILO World Social Protection Database and report tables** | Coverage by benefit category and social-protection expenditure | Broad protection is not synonymous with poverty relief; health treatment differs between indicators. [International Labour Organization](https://www.ilo.org/publications/flagship-reports/world-social-protection-report-2024-26-universal-social-protection-climate) |
| **National Transfer Accounts** | Age profiles of consumption, labor income and public/private transfers | Useful for lifecycle financing, but accounting flows do not identify causal behavioral responses. [National Transfer Accounts](https://www.ntaccounts.org/web/nta/show) |
| **Historical granary and relief reconstructions** | Reserve stocks, institutional finance and local–central relationships | Incomplete geography, changing units, selective survival of records and uncertain comparability. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/local-granaries-and-central-government-disaster-relief-moral-hazard-and-intergovernmental-finance-in-eighteenth-and-nineteenthcentury-china/80F2E2CEBBEA518A60D40E2116312898) |

### Claims to keep explicitly uncertain

**Informal coverage and spending are poorly quantified across deep history.** There is no defensible global percentage of forager or early-farming output devoted to supporting the poor, sick and elderly. Archaeology can establish storage or survival with impairment more readily than it can establish funding rules or population coverage.

**Norms are not realized provision.** A religious contribution rule, foundation deed or legal entitlement establishes an intended obligation. It does not show how much was collected, who was excluded or what arrived.

**Malthusian and labor-disincentive claims remain context-dependent.** Household formation, fertility, wage bargaining and labor supply should respond through their own mechanisms. The English reform literature does not justify a universal dependency or fertility multiplier. [IDEAS/RePEc](https://ideas.repec.org/p/iza/izadps/dp7398.html)

**Famine effects require counterfactuals.** Places with the worst crises often receive the most assistance, producing misleading correlations between relief and mortality. Conversely, successful prevention can leave little visible evidence of the catastrophe avoided.

**Modern causal estimates have limited historical portability.** An experiment with cash, functioning markets and substantial administrative capacity establishes possibilities and mechanisms; it does not identify the same numerical effect for an isolated early agrarian settlement.

**Bottom line for TCE:** preserve the chain from **resources → rights and requests → actual delivery → individual consumption and care → survival, work and political response**. That structure can generate effective mutual aid, selective charity, successful famine prevention, coercive work relief, exclusionary welfare and institutional failure without scripting any of them.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928e2-d648-83ea-b683-08836b98f936)
