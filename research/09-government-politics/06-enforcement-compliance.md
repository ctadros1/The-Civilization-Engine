# State capacity, enforcement, and compliance

## A simulation-ready report for The Civilization Engine

**The central recommendation is to make laws create obligations, opportunities for dispute, and institutional workloads—not automatic changes in behavior.** Whether a law matters should emerge from what authorities can observe, what officials choose to pursue, what sanctions they can actually carry out, and what citizens consider worthwhile or obligatory.

There is no defensible single percentage of “the law” that states enforce across history. Inspection coverage, detection of violations, successful adjudication, collection of penalties, and compliance are different quantities. Even “voluntary tax compliance” in official statistics means timely payment without subsequent enforcement; it does **not** mean payment motivated entirely by civic duty or made without withholding and reporting systems. [IRS](https://www.irs.gov/statistics/irs-the-tax-gap)

For TCE, the useful unit is therefore:

> **A particular obligation, imposed on a particular person or organization, in a particular place and institutional setting.**

The same government should be able to collect market tolls reliably, struggle to assess dispersed agricultural income, protect influential offenders, and enforce unpopular rules only near its administrative centers. This is a modeling synthesis of the evidence below, not a claim that every historical state followed the same pattern.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Represent capacity as several capabilities, not one score

Besley and Persson distinguish investments in fiscal and legal capacity and model why political conditions influence those investments. The important implication is that governing capability is an accumulated institutional asset, not simply the ruler’s preference for stronger enforcement. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.99.4.1218)

For TCE, use the following decomposition:

| Capability | Concrete state variables | What failure looks like |
| --- | --- | --- |
| **Fiscal** | Grain and cash reserves; assessed tax bases; collection infrastructure; reliable income streams | Officials go unpaid; expeditions cannot be supplied; collection costs exceed receipts |
| **Administrative** | Registers; record accuracy; trained personnel; communication links; assessment and case-processing capacity | Unknown households; obsolete assessments; lost evidence; contradictory orders; backlogs |
| **Coercive** | Available and loyal personnel; transport; supplies; local force advantage; custody and seizure capacity | Orders cannot be executed; officials avoid dangerous targets; local authorities resist |
| **Political authorization and accountability** | Jurisdiction; exemptions; supervisory institutions; patronage; appeals; enforcement priorities | A technically capable office is forbidden, discouraged, or bribed not to act |

The last row is deliberately separate: **“cannot enforce” and “will not enforce” must produce different explanations and different remedies.**

Store these capabilities by **institution, territory, and domain**. An efficient customs office should not automatically improve murder investigations or village land registration.

### 1.2 Authorities must acquire information

**Implementable rule:** The simulation knows the true world state, but institutions know only what reaches them through observations, records, witnesses, declarations, and other institutions.

Create distinct information channels:

* Direct observation: a patrol sees a prohibited transaction.
* Reporting: a victim, neighbor, merchant, or local leader makes a complaint.
* Administrative traces: a census entry, land register, employer report, invoice, or warehouse receipt.
* Investigation: an official spends time comparing records, questioning witnesses, or inspecting assets.

The Danish tax-audit experiment found sharply different evasion on third-party-reported and self-reported income. The difference was not adequately explained by sorting people into inherently honest employees and dishonest self-employed taxpayers. Information conditions mattered within taxpayers’ activities. [Econometrics Laboratory](https://eml.berkeley.edu/~saez/kleven-knudsen-kreiner-pedersen-saezEMA11taxaudit.pdf)

**TCE consequence:** Technology and institutional organization can increase compliance without a proportional increase in patrols. However, records should require maintenance and cooperation; inventing writing must not instantly reveal everyone’s income.

Pomeranz’s experiments involving more than 400,000 Chilean firms also found that VAT’s documentary links shaped deterrence and transmitted compliance effects upstream through trading relationships. An information system can therefore have network effects rather than merely improving each inspected firm independently. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20130393)

### 1.3 Model enforcement as a conditional sequence

For a violation, define:

\[
q\_{\text{effective}}
=
P(\text{noticed})
P(\text{investigated}\mid\text{noticed})
P(\text{proved}\mid\text{investigated})
P(\text{sanction ordered}\mid\text{proved})
P(\text{executed}\mid\text{ordered})
\]

These are **conditional probabilities**. Multiplying them does not require assuming that the stages are independent.

In the actual simulation, most stages should be produced by events and institutional decisions, rather than by five arbitrary dice rolls. The equation is useful for diagnostics and simplified off-screen resolution.

For example, entirely illustrative stage probabilities of \(0.5, 0.8, 0.5, 0.5, 0.5\) imply an effective sanction probability of only **5%**. A government that notices half of violations can still punish very few.

**Implementable rule:** Every stage consumes resources and can terminate for a recorded reason: insufficient evidence, settlement, lawful exemption, low priority, corruption, political protection, inability to locate the defendant, or inability to execute the judgment.

Do not classify every unpunished case as failure. A warning that produces remediation, a successful appeal, and a bribe-induced dismissal are substantively different outcomes.

### 1.4 Enforcement is a workload allocation problem

For institution \(j\), constrain assignments by:

\[
\sum\_{k\in\text{assigned cases}} h\_k
\leq H\_j
\]

Here \(h\_k\) includes travel, investigation, paperwork, hearing time, and execution; \(H\_j\) is available staff time after absence, training, other duties, and supervision.

**Implementable rules:**

New laws add obligations and potentially add cases. Without more resources or better information, additional duties can crowd out existing enforcement. Travel and communication delays reduce local throughput. Backlogs delay remedies and can degrade evidence.

Targeting should depend on expected harm, evidence, recoverable revenue, political priorities, and cost—not just random selection. An Indian environmental-regulation experiment found that doubling inspections through additional randomized visits produced only a small compliance improvement. The authors’ structural analysis found substantial value in the regulator’s existing targeting information. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.3982/ECTA12876)

The design implication is **not** that inspections are useless. It is that their marginal effect depends on whom they reach and what happens afterward.

### 1.5 Citizens choose among feasible responses

Start with a feasible action set. A person cannot pay grain they do not possess, obey a construction standard they do not understand, or obtain a permit from an office they cannot reach.

Then evaluate actions such as compliance, partial compliance, concealment, negotiation, appeal, bribery, relocation, and resistance:

\[
U\_i(a)
=
E[u\_i(\text{resources after }a)]
-
C\_i^{\text{effort}}(a)
-
C\_i^{\text{moral}}(a)
-
C\_i^{\text{social}}(a)
\]

Expected resource outcomes incorporate perceived detection, collectible penalties, confiscation, lost work, and other consequences. This extends the expected-utility approach of Allingham and Sandmo rather than replacing it with a universal obedience trait. [Econometrics Laboratory](https://eml.berkeley.edu/~saez/course/Allingham%26SandmoJPubE%281972%29.pdf)

For a simple risk-neutral tax example, evading an amount \(x\), with effective enforcement probability \(q\) and an additional fine \(fx\), gives:

\[
\Delta U\_{\text{evade}}
=
x-q(1+f)x
-\text{concealment costs}
-\text{moral and social costs}
\]

The \(1+f\) matters: when caught, the taxpayer repays the unpaid liability **and** the fine.

For TCE, distinguish:

**Unwillingness:** compliance is feasible but unattractive.  
**Inability:** compliance is currently infeasible.  
**Ignorance or misunderstanding:** the agent does not correctly perceive the obligation.  
**Disagreement over liability:** the agent believes the authority’s assessment is wrong.

These should generate different behavior under assistance, reminders, appeals, threats, and seizures.

### 1.6 Perceived risk differs from actual risk

**Implementable rule:** Agents act on their beliefs about enforcement, not on the simulation’s true probabilities.

Update beliefs from personal experience, known peers, official announcements, and trusted intermediaries. Weight information by credibility and similarity: an inspection of a large merchant may tell a subsistence farmer little about their own risk.

Keep inspection frequency separate from detection conditional on evasion. A taxpayer may rarely experience a conventional audit while facing a high probability that a mismatch with third-party records will be noticed. The distinction is explicit in the tax-compliance literature. [Econometrics Laboratory](https://eml.berkeley.edu/~saez/course/dwenger-kleven-rasul-rincke_aejpol2015.pdf)

Avoid revealing all government actions globally. A punishment can be effective locally but unknown elsewhere; conversely, a highly publicized case can influence many more people than were directly investigated.

### 1.7 Legitimacy, moral agreement, and norms are separate motivations

Tyler’s framework distinguishes instrumental incentives from normative commitments, including the legitimacy of authorities and judgments about the morality of conduct. It also emphasizes how people experience legal procedures, rather than treating satisfaction with the outcome as the only relevant consideration. [UC Davis Design Academy](https://www.des.ucdavis.edu/Faculty/Sabatier/Tyler1990.pdf)

Represent at least four separate beliefs:

| Belief | Example |
| --- | --- |
| **Authority legitimacy** | “This council has the right to issue binding decisions.” |
| **Law-specific moral agreement** | “Taking water beyond my allocation is wrong.” |
| **Descriptive norm** | “Most people in my village pay this levy.” |
| **Expected social reaction** | “My relatives will condemn—or support—my refusal.” |

**Implementable rule:** These beliefs enter action evaluation separately. A legitimate government can enact a disliked law. A distrusted ruler can prohibit conduct that people already reject. Local norms can support compliance or organized noncompliance.

Intrinsic motivation is not merely a theoretical residual. In a German local church-tax setting with historically no enforcement, approximately **20%** paid at least their liability. The experiment also found no automatic crowding-out of intrinsic motivation by deterrence. This is evidence for heterogeneous motivations, not a universal 20% “honest population” parameter. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fpol.20150083)

Nor should TCE award legitimacy automatically for adopting a named reform. Six coordinated community-policing experiments in Brazil, Colombia, Liberia, Uganda, Pakistan, and the Philippines found no improvement in trust or crime outcomes, with incomplete implementation and limited follow-through among the problems. This does not show that legitimacy is irrelevant; it shows that an intervention intended to build it may fail to do so. [MIT GOV/LAB](https://mitgovlab.org/research/community-policing-does-not-build-citizen-trust-in-police-or-reduce-crime-in-the-global-south/)

### 1.8 Officials are agents, too

**Implementable rule:** Inspectors, judges, tax collectors, and local leaders have incentives, relationships, information limits, and personal risks.

Their decisions can respond to salary, promotion rules, revenue bonuses, supervision, bribes, patronage, professional norms, sympathy, fear, and political directives.

In a randomized Pakistani property-tax experiment, performance incentives increased revenue but also increased reported bribery for many properties whose official taxes did not change. Greater collections and greater integrity are therefore not interchangeable outcomes. [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/5d380486-1b1c-49c0-a4b0-8972e348d876)

Separate four kinds of selective treatment:

| Type | Example | Modeling treatment |
| --- | --- | --- |
| **Lawful differentiation** | Exempt households or special jurisdictions | Changes the actual obligation |
| **Administrative triage** | Prioritizing large harms or strong evidence | Changes case selection |
| **Favoritism or capture** | Protecting a patron’s business | Changes treatment contrary to general rules |
| **Coercive limitation** | Avoiding an armed landholder | Changes feasible enforcement |

This distinction allows meaningful institutional reform: removing an exemption, adding inspectors, improving evidence, and confronting a powerful patron solve different problems.

### 1.9 Capacity can accumulate, stagnate, or be deliberately limited

Fiscal and administrative investments can reinforce one another, but rulers do not necessarily maximize the long-run capability of the state. Besley and Persson model political determinants of investment; Ma and Rubin offer a principal–agent explanation for why absolutist rulers may tolerate weak monitoring and unofficial extraction rather than construct a fully accountable fiscal administration. The latter is an interpretation of historical arrangements, not a settled universal explanation. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.99.4.1218)

**Implementable rule:** Let institutions invest in training, registers, roads, communications, oversight, and enforcement personnel. Also allow diversion, neglected maintenance, destruction of archives, loss of skilled staff, and deliberate delegation.

Maintain separate ledgers for **household burden, intermediary retention, treasury receipts, and public services**. Informal taxation research across ten countries documents substantial local contributions, including labor, that official fiscal statistics can miss. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.3.4.1)

---

## 2. Quantitative parameters and calibration evidence

### How to interpret the numbers

The tables distinguish three kinds of evidence:

**Observed or reconstructed levels** are outcomes to reproduce under comparable conditions.  
**Experimental effects** are changes caused by a particular intervention.  
**Structural-model estimates** depend on an estimated model as well as data.

Confidence refers to the finding **in its studied setting**. Transferability to another era is generally much lower.

### 2.1 Fiscal scale across historical and modern settings

| Setting | Quantitative benchmark | Interpretation and limitation | Source; confidence |
| --- | --- | --- | --- |
| **Ottoman core territories, early modern period** | Direct central cash receipts probably **below 4% of GDP**; including imputed cavalry service, **below 6%** | Reconstructed central resources, not total household extraction or tax compliance | Karaman & Pamuk, 2010. **Medium–low**, particularly GDP denominator. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914) |
| **European states, sixteenth century to 1780s** | Central revenue generally **below 5% of GDP** in the sixteenth century; commonly **5–10%** by the 1780s; Britain and the Netherlands **above 10%** | Comparative fiscal benchmarks, not an inevitable progression | Karaman & Pamuk, 2010. **Medium–low**. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914) |
| **Africa, 2023** | Average **16.1% of GDP** across 38 covered countries; range **2.9–34.0%** | Reported tax revenue; enormous within-region variation | OECD/AUC/ATAF, *Revenue Statistics in Africa 2025*. **High** for published accounting; coverage caveats. [OECD](https://www.oecd.org/en/about/news/announcements/2025/12/corporate-income-taxes-drove-up-tax-revenues-in-africa-again-in-2023.html) |
| **Asia-Pacific, 2023** | Average **19.6% of GDP** | Covered economies differ greatly in structure and institutions | OECD, *Revenue Statistics in Asia and the Pacific 2025*. **High** for published accounting. [OECD](https://www.oecd.org/en/about/news/announcements/2025/07/vat-drove-up-tax-revenues-in-the-asia-pacific-region-in-2023.html) |
| **Latin America and Caribbean, 2023** | Average **21.3% of GDP**; range **11.6–32.0%** | Tax revenue, not total state revenue or compliance | Joint regional revenue statistics, 2025. **High** for published accounting. [Inter-American Development Bank](https://www.iadb.org/en/news/tax-revenues-latin-america-and-caribbean-fell-2023-commodity-prices-weakened) |
| **OECD countries, 2023** | Average **33.9% of GDP** | Benchmark for the covered modern tax systems, not a target for every industrial society | OECD comparison reported in the joint regional statistics. **High**. [Inter-American Development Bank](https://www.iadb.org/en/news/tax-revenues-latin-america-and-caribbean-fell-2023-commodity-prices-weakened) |

**Do not directly splice these into one historical series.** Historical central receipts and modern tax aggregates differ in government coverage, inclusion of social contributions, and treatment of noncash obligations. The UNU-WIDER Government Revenue Dataset explicitly distinguishes central/general government and, where possible, resource-related revenues. [WIDER](https://www.wider.unu.edu/project/grd-government-revenue-dataset)

For TCE, fiscal ratios should be **emergent validation outputs**. They do not identify a unique enforcement probability: the same receipts can result from different statutory burdens, tax bases, exemptions, compliance, and collection arrangements.

### 2.2 Compliance and enforcement benchmarks

| Evidence | Numerical result and units | What it should calibrate | Source; confidence |
| --- | --- | --- | --- |
| **Denmark: information and income reporting** | Detectable underreporting of **positive income: 0.23% for third-party-reported income versus 17.1% for self-reported income** | Strong variation by information channel; these are income shares, not shares of people cheating | Kleven et al., 2011. **High** for detectable evasion; true hidden evasion may be greater. [Econometrics Laboratory](https://eml.berkeley.edu/~saez/kleven-knudsen-kreiner-pedersen-saezEMA11taxaudit.pdf) |
| **United States: tax year 2022** | Projected timely compliance **85.0% of true tax liability**; gross gap **$696 billion**; enforcement and other late payments **$90 billion** | Distinction between timely payment, ultimate payment, and collection after nonpayment | IRS tax-gap projections. **Medium**: modeled from underlying evidence, not a complete contemporaneous audit. [IRS](https://www.irs.gov/statistics/irs-the-tax-gap) |
| **Germany: local church tax** | Approximately **20% of individuals** paid at least their liability under the historical zero-enforcement baseline | Nonzero compliance without credible formal punishment; heterogeneous motivation | Dwenger et al., 2016. **High** for this small, unusual tax; **low** cross-context transferability. [Econometrics Laboratory](https://eml.berkeley.edu/~saez/course/dwenger-kleven-rasul-rincke_aejpol2015.pdf) |
| **Gujarat, India: environmental inspections** | Experiment **doubled inspection frequency**, with only a small compliance improvement | Marginal inspections can be weak when targeting or subsequent enforcement is limiting | Duflo et al., 2018. **High** for the experiment’s setting. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.3982/ECTA12876) |
| **Same Indian study: targeting model** | Estimated schedules ranged from **less than one inspection/year for half of plants** to roughly **ten/year for the dirtiest**; targeted inspections produced about **three times the abatement** of equal-count random inspections in the model | Highly unequal inspection intensity; value of information and discretion | Duflo et al., 2018. **Medium**: structural estimates, not a direct randomized threefold treatment effect. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.3982/ECTA12876) |
| **California: workplace safety inspections** | **409 inspected establishments and 409 matched controls**; estimated **9.4% fewer injuries** and **26% lower injury costs** in subsequent years | Inspections can change real behavior and harm, not merely reported paperwork | Levine, Toffel & Johnson, 2012. **High** within this design; external effects vary. [Harvard Business School](https://www.hbs.edu/ris/Publication%20Files/LevineToffelJohnson_2012_Science_1e9730f0-4224-4b6a-b1b6-ff0c44d11514.pdf) |
| **Punjab, Pakistan: collector incentives** | Across **482 tax units**, incentive schemes increased revenue by **9.4 log points after two years**, approximately **9.9% in levels** | Official incentives, reassessment, revenue–bribery trade-offs | Khan, Khwaja & Olken, 2016. **High** for the experiment. [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/5d380486-1b1c-49c0-a4b0-8972e348d876) |

Two numerical traps deserve explicit warnings.

First, Danish estimates expressed relative to **net** self-reported income are much larger because positive and negative components nearly cancel. The positive-income comparison above avoids that denominator problem. [Econometrics Laboratory](https://eml.berkeley.edu/~saez/kleven-knudsen-kreiner-pedersen-saezEMA11taxaudit.pdf)

Second, the Pakistani study’s description of **46% higher revenue growth** does not mean revenue levels became 46% higher. The reported level effect is 9.4 log points. [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/5d380486-1b1c-49c0-a4b0-8972e348d876)

### 2.3 Parameters without defensible universal historical estimates

The literature does not supply transferable values for “a medieval investigation succeeds with probability \(x\)” or “legitimacy increases compliance by \(y\) per point.” Use explicit engineering priors rather than presenting guesses as historical measurements.

The following is a **proposed sensitivity-test grid**, not empirical calibration:

| Parameter | Suggested test values | Units and implementation |
| --- | --- | --- |
| Inspection intensity | **0.01, 0.1, 1** | Visits per eligible unit per year; add higher intensities for repeatedly targeted establishments |
| Conditional follow-through at a bottleneck | **0.2, 0.5, 0.9** | Probability conditional on reaching that stage |
| Administrative trace coverage | **0.1, 0.5, 0.9** | Share of relevant obligations or transactions covered by usable records |
| Belief-memory half-life | **30, 180, 720** | Days; a modeling assumption to test persistence of enforcement beliefs |
| Additional monetary penalty | **0.25, 1, 3** | Multiples of the unpaid liability; hypothetical law settings, not historical defaults |

**Confidence: low for historical transfer; purpose: sensitivity analysis.** Once the institution is simulated explicitly, derive inspection intensity and follow-through from staff, assignments, evidence, and resources rather than retaining them as independent sliders.

For a Poisson inspection process:

\[
P(\text{at least one visit during }\Delta t)=1-e^{-\lambda\Delta t}
\]

That is a probability of a **visit**, not necessarily detection. Keep these separate. Also attach liabilities to assessment periods: an annual tax should not receive a fresh independent “annual audit chance” every simulation day.

---

## 3. Variation across eras and world regions

### Foragers: little formal state capacity need not mean little enforcement

Boehm’s comparative work describes informal sanctioning through criticism, gossip, ridicule, ostracism, and coalitions against domineering individuals. It also notes how incomplete detailed accounts of sanctioning can be. These ethnographic cases are not direct observations of Paleolithic society, and they should not be generalized to every foraging population. [Perpustakaan UMA](https://opac.uma.ac.id/repository/0674390318.pdf)

**TCE representation:** Begin with zero specialized state inspectors where no state exists, but retain witnesses, reputations, kin ties, collective decisions, and the possibility of expulsion or retaliation. Enforcement depends on people’s willingness to participate and their ability to coordinate against the target.

A small group can possess substantial interpersonal knowledge while lacking any machinery for ruling distant strangers. Model those as different capacities.

### Early farming: distinguish village obligations from state administration

For TCE’s starting conditions, the crucial design distinction is between **settled production** and **institutions capable of assessing and extracting from it**. A field being visible should not automatically mean that an authority knows its yield, recognizes its owner, or can collect a share.

Use seasonal assessments, named household obligations, witnesses, communal labor requests, and storage accounts as possible authored components. Require actual organization to connect them.

The evidence becomes richer in ancient state settings. Research on the fiscal capacity of Early Bronze Age Ebla and on archaic Ur examines administrative records and the extent of political institutions. Such evidence supports modeling differentiated institutions, but does not justify a single “early farming compliance rate.” [ResearchGate](https://www.researchgate.net/publication/347387927_The_Fiscal_Capacity_of_the_Ebla_State_in_the_Early_Bronze_Age_Taxation_and_Political_Structure?utm_source=chatgpt.com)

**Important boundary:** Do not initialize an ordinary early agricultural village with the administrative capabilities of a later palace-centered state.

### Pre-industrial societies: several different routes to capacity

**Andes.** The Inka illustrate why money revenue and alphabetic writing are poor universal prerequisites. Their administration organized labor obligations, resource accounting, provincial centers, and local intermediaries; khipu records supported administration. D’Altroy emphasizes that taxpayers rendered labor and expertise while the state received services and goods. [Academia](https://www.academia.edu/18988626/The_Inka_empire_fiscal_regime)

**TCE implication:** Maintain labor-days, transport obligations, stored food, and supplies as real fiscal resources. Do not convert everything into immediately fungible money. A storehouse full of grain may support nearby workers while doing little for an isolated expedition.

**China.** Ma and Rubin’s Qing-focused analysis emphasizes problems of monitoring officials and committing not to appropriate their returns. Low officially recorded revenue can coexist with powerful sovereign claims and unofficial extraction. This is one institutional explanation; avoid turning it into an immutable “Chinese state” modifier. [Chapman University Digital Commons](https://digitalcommons.chapman.edu/esi_working_papers/212/)

**Ottoman and European polities.** Tax intermediaries could collect, retain, spend, or remit resources under different arrangements. Changes in central receipts partly reflected changes in intermediation, not simply changes in taxpayer honesty. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914)

The shared modeling lesson is **layered government**: household → village or estate → district → treasury, with information and resources potentially changing at every transfer.

### Industrialization: increasing reach without uniform improvement

The long-run expansion of taxation is associated with changes in economic structure and fiscal institutions, not merely a willingness to impose higher rates. Besley and Persson discuss how development and administrative arrangements affect the ability to tax. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.28.4.99)

For TCE, make concentrated workplaces, standardized accounts, improved communication, and more capable organizations potential inputs to broader assessment and enforcement. These should be acquired capabilities rather than bonuses unlocked by an “industrial era.”

Colonial cases prevent a simple equation of extraction with effective public administration. Cogneau, Dupraz, and Mesplé-Somps find substantial fiscal extraction in the French colonial empire alongside underadministration, high public wage costs, and unequal public provision. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/fiscal-capacity-and-dualism-in-colonial-states-the-french-empire-18301962/8523B1BFB35FBA23514F84605566CDE2)

**TCE implication:** A state may be effective at compelling selected populations to pay or work while providing them with weak legal protection and few services.

### Modern societies: high aggregate capacity, persistent unevenness

The modern fiscal comparisons show large differences both between and within regions. The experimental evidence likewise shows that administrative domains within one country can behave very differently. Neither “modernity” nor a national trust coefficient adequately specifies enforcement. [OECD](https://www.oecd.org/en/about/news/announcements/2025/12/corporate-income-taxes-drove-up-tax-revenues-in-africa-again-in-2023.html)

A modern TCE state should therefore remain capable of having well-recorded formal activities beside poorly observed transactions, or technically sophisticated offices beside politically protected targets. The relevant mechanisms are information, organization, incentives, and constraints—not a date on the calendar.

---

## 4. Stylized facts a correct simulation should reproduce

These are validation targets, not requirements that every generated world match every historical example.

| Pattern | Evidence or numerical anchor | TCE validation test |
| --- | --- | --- |
| **Compliance differs sharply by observability** | Danish positive-income underreporting: **0.23% versus 17.1%** across reporting channels. [Econometrics Laboratory](https://eml.berkeley.edu/~saez/kleven-knudsen-kreiner-pedersen-saezEMA11taxaudit.pdf) | Give the same agents differently observable income sources; reporting should diverge without changing personality |
| **Some people comply without formal punishment** | Roughly **20%** complied with the German local church tax under its historical zero-enforcement baseline. [Econometrics Laboratory](https://eml.berkeley.edu/~saez/course/dwenger-kleven-rasul-rincke_aejpol2015.pdf) | Remove formal enforcement; compliance should not necessarily become zero |
| **Extra inspections have context-dependent returns** | Small compliance gains from doubled inspection frequency in Gujarat, but meaningful injury reductions in California. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.3982/ECTA12876) | Vary targeting quality, remediation costs, and follow-through; inspection effects should vary |
| **Information creates spillovers** | Chilean VAT enforcement effects propagated through business relationships. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20130393) | Inspect one firm; some connected firms should update reporting or risk beliefs |
| **Revenue and integrity can move in different directions** | Pakistani collector incentives increased revenue and reported bribery. [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/5d380486-1b1c-49c0-a4b0-8972e348d876) | Revenue bonuses should not automatically reduce corruption |
| **Low central revenue can coexist with substantial obligations** | Informal taxation includes local labor contributions; nonmonetary fiscal systems mobilize resources outside cash receipts. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.3.4.1) | Compare treasury receipts with household burden and mobilized labor; they should not be identical |
| **Institutional labels do not guarantee implementation** | Six community-policing experiments failed to generate the hoped-for trust and crime improvements. [MIT GOV/LAB](https://mitgovlab.org/research/community-policing-does-not-build-citizen-trust-in-police-or-reduce-crime-in-the-global-south/) | Enacting a reform without staffing, continuity, or follow-through should sometimes produce little change |

Also test the following **logical consequences of the proposed model**, rather than presenting them as independently estimated historical regularities:

An expanded lawbook with fixed staffing should create crowding-out or longer queues. Better detection can initially increase recorded violations even if actual violations decline. A fiscal crisis can reduce staff availability and further weaken collections. Selective protection can generate high aggregate compliance alongside visible elite impunity.

These are particularly useful stress tests because they expose hidden omniscience and automatic-compliance shortcuts.

---

## 5. Modeling recommendation for TCE

### 5.1 Recommended objects and state

| Object | Minimum useful state |
| --- | --- |
| **Person** | Resources; obligations known; perceived enforcement risk; law-specific moral agreement; legitimacy beliefs; risk preferences; relevant social ties |
| **Household or enterprise** | Liable members; production and assets; location; registration; transaction records; arrears; responsible decision-makers |
| **Law or norm** | Jurisdiction; eligibility predicate; required or prohibited conduct; assessment rule; exemptions; evidence requirements; remedies; responsible authority |
| **Institution or office** | Staff; skills; budget and supplies; records; jurisdiction; priorities; supervisory arrangements; case queue |
| **Case** | Allegation; evidence; confidence; stage; assigned personnel; deadlines; costs; decision; remedy; execution status |
| **Local social group** | Membership; shared expectations; information channels; sanctioning relationships; ability to coordinate |

Avoid storing a dense person-by-law matrix. Most people need detailed beliefs only about laws they encounter. Use domain-level defaults with sparse overrides for salient rules.

### 5.2 Separate truth, knowledge, and belief

Maintain three distinct layers:

**World truth:** what happened, what was produced, and whether conduct violated the authored rule.

**Institutional knowledge:** what an authority observed, recorded, or inferred.

**Agent belief:** what citizens and officials think is true, legitimate, feasible, and likely.

Only the first layer belongs in an omniscient debugging dashboard. Officials must act on the second; everyone chooses actions using some version of the third.

This distinction enables mistaken assessments, false accusations, concealed activity, credible complaints, and corruption without scripting each outcome.

### 5.3 Use event-driven obligations and resource-bounded case processing

A suitable kernel loop is:

```
Economic or social event occurs
    → determine applicable obligations
    → update true compliance state
    → generate observable traces and possible reports
    → institutions receive only accessible information

Institution scheduling step
    → rank actionable matters
    → assign available personnel and resources
    → advance investigations, hearings, remedies, and execution
    → retain explicit reasons for delay or closure

Agent decision step
    → construct feasible responses
    → evaluate material, moral, social, and enforcement consequences
    → choose action using bounded information

Learning and institutional update
    → revise beliefs from observed outcomes
    → update records, arrears, reputations, resources, and capacity
```

For 10k–50k agents, index laws by event type—sale, harvest, inheritance, construction, assault, border crossing—rather than checking every law against every person every tick.

Resolve ordinary administration at daily or weekly intervals where appropriate, but let immediate interventions interrupt visible activities. Seasonal levies and periodic assessments should create scheduled work rather than constant background deductions.

### 5.4 Preserve scarce resources and conservation

Collections should move goods, money, or labor—not create government resources.

A fine must be paid from something, become an arrear, or trigger another authorized remedy. Seized grain requires collection and storage. Detention consumes supervision and supplies. Officials assigned to one task cannot simultaneously perform unlimited others.

For a nonmonetary state, track the distinction between **labor promised**, **labor delivered**, and **usable output produced**. They are not interchangeable.

### 5.5 What to simplify—and what not to simplify

Simplify the legal details of routine cases into authored templates. Aggregate repetitive bookkeeping at the actual liability unit, such as the household or enterprise. Use coarse evidence categories where detailed investigation would add little gameplay or explanatory value.

Do **not** simplify away:

* Differences between detection, decision, and execution.
* Differences between inability, unwillingness, and exemption.
* Officials’ incentives and political constraints.
* Local information and unequal administrative reach.
* Non-state enforcement and noncash obligations.

Those distinctions generate most of the behavior the user wants laws to have.

### 5.6 Existing models worth borrowing from

| Model or framework | Useful component | Limitation for TCE |
| --- | --- | --- |
| **Allingham–Sandmo tax-evasion model, 1972** | Expected utility of concealment under detection and penalties | In its basic form, too narrow for norms, mistaken beliefs, administrative bottlenecks, and institutional evolution. [Econometrics Laboratory](https://eml.berkeley.edu/~saez/course/Allingham%26SandmoJPubE%281972%29.pdf) |
| **Kirchler–Hoelzl–Wahl “slippery slope” framework, 2008** | Distinguishes authority power and trust as routes to compliance | Not a universal calibrated equation; trust and power need concrete mechanisms. [Power of Taxes](https://poweroftaxes.univie.ac.at/en/state-of-the-art/slippery-slope-framework/) |
| **Duflo et al. regulatory-discretion model, 2018** | Target selection, information, and enforcement incentives | Estimated for a particular regulatory setting, not a ready-made historical government model. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.3982/ECTA12876) |
| **Epstein civil-violence model, 2002; NetLogo/Mesa implementations** | Local interaction among grievance, legitimacy, perceived arrest risk, citizens, and enforcement agents | A stylized model: global legitimacy and permanently loyal police should not be imported unchanged. [PNAS](https://www.pnas.org/doi/10.1073/pnas.092080199) |

The strongest approach is to combine these components rather than adopt any one framework wholesale.

### 5.7 Recommended sanity dashboard

Report outcomes by **law domain, location, wealth/status group, and institution**:

| Measure | Denominator |
| --- | --- |
| Obligation coverage | Eligible people, households, enterprises, or transactions |
| Compliance | Applicable obligations; also amount-weighted where meaningful |
| Inspection coverage | Eligible units over a stated period |
| Detection | Actual violations in omniscient telemetry |
| Institutional detection estimate | Investigated or sampled units—clearly labeled as selected |
| Follow-through | Cases reaching each preceding stage |
| Remedy completion | Ordered remedies |
| Collection | Assessed liability, with disputed assessments separated |
| Household burden | Formal payments + informal contributions + labor + illicit payments |
| Administrative cost | Staff time, supplies, transport, and foregone production |

Include false positives, successful appeals, warnings followed by remediation, uncollectible arrears, and case age. A dashboard showing only arrests and receipts will reward pathological enforcement strategies.

---

## 6. Sources, datasets, and limits of the evidence

### Recommended data sources

| Source | Best use | Main caution |
| --- | --- | --- |
| **UNU-WIDER Government Revenue Dataset, version 2025** | Tax/non-tax revenues; central/general government distinctions; resource-revenue comparisons | Fiscal outcomes are not direct measures of compliance or administrative quality. [WIDER](https://www.wider.unu.edu/project/grd-government-revenue-dataset) |
| **OECD Global Revenue Statistics and regional publications** | Harmonized modern fiscal benchmarks across Africa, Asia-Pacific, Latin America, and OECD countries | Preserve the publication vintage, coverage, and accounting definitions. [OECD](https://www.oecd.org/en/topics/sub-issues/global-tax-revenues.html) |
| **IRS tax-gap studies** | Decomposing nonfiling, underreporting, underpayment, timely payment, and later receipts | Projections and audit-based estimates have uncertainty; hidden violations are not directly enumerated. [IRS](https://www.irs.gov/statistics/irs-the-tax-gap) |
| **World Bank Enterprise Surveys** | Firms’ reported encounters with corruption and administrative demands | Survey populations and reported experiences do not represent every household or informal activity. [Enterprise Surveys](https://www.enterprisesurveys.org/en/data/exploretopics/corruption) |
| **Seshat: Global History Databank** | Historical presence of administrative specialists, military professionals, courts, legal institutions, and related structures | Institutional presence is not a measured enforcement probability; preserve uncertainty and underlying source notes. [Sage Journals](https://journals.sagepub.com/doi/10.1177/14747049211066600) |
| **French colonial fiscal dataset accompanying Cogneau et al.** | Historical extraction, expenditures, and administrative dualism across a non-European imperial setting | Colonial accounts require attention to coercion, unequal provision, and coverage. [ICPSR](https://www.icpsr.umich.edu/sites/jeh/view/studies/133361/versions/V1) |

For historical institutional design, the most useful starting volumes are Boehm’s *Hierarchy in the Forest* and Monson and Scheidel’s *Fiscal Regimes and the Political Economy of Premodern States*, particularly D’Altroy on the Inka and Jursa and Moreno García on the ancient Near East and Egypt. Use them to author institutional possibilities, not to assign universal era coefficients. [Perpustakaan UMA](https://opac.uma.ac.id/repository/0674390318.pdf)

### Where uncertainty matters most

**Historical enforcement probabilities are thinly measured.** Administrative texts document what institutions attempted and recorded; they rarely provide a representative denominator of all obligations and all violations. Reconstructed fiscal totals are generally firmer than reconstructed compliance rates.

**Modern causal evidence transfers imperfectly.** A letter experiment involving overdue taxes, a church tax, a factory inspection, and a confrontation with an armed landholder are not interchangeable enforcement environments. Parameterize the mechanisms that distinguish them before transferring estimated effects.

**Legitimacy is not a universally measurable physical quantity.** Tyler’s work supplies a useful conceptual structure, while field experiments show that particular efforts to improve police–citizen relations may fail. A scalar can be a computational approximation, but it should summarize heterogeneous beliefs and experiences, not replace them. [UC Davis Design Academy](https://www.des.ucdavis.edu/Faculty/Sabatier/Tyler1990.pdf)

**Capacity is not identical to benevolence, democracy, or growth.** The historical and experimental evidence includes effective extraction with weak public provision, and higher collections alongside more bribery. TCE should permit capable predation, legitimate but resource-poor administration, and competent institutions constrained by powerful interests. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/fiscal-capacity-and-dualism-in-colonial-states-the-french-empire-18301962/8523B1BFB35FBA23514F84605566CDE2)

**Bottom line:** The most important state-capacity mechanic is not a stronger compliance bonus. It is the ability to turn an obligation into usable information, a feasible decision, and a completed remedy—while citizens and officials retain their own motives, constraints, and choices.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928be-4ca0-83ea-b67f-e7ab079c0226)
