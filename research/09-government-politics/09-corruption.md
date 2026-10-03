# Corruption, patronage and clientelism

## A simulation-ready research report for The Civilization Engine

**The strongest design is to model corruption as the diversion of particular resources, decisions and enforcement processes—not as a civilization-wide “corruption percentage.”** Bribe-taking, theft from a granary, patronage appointments and capture of a court have different opportunities, beneficiaries and consequences. They should sometimes reinforce one another, but need not move together.

Two empirical findings make this distinction essential. Auditing Indonesian road projects reduced missing expenditure without eliminating it. Incentive pay for Pakistani tax collectors increased government revenue **and** reported bribe payments. A society can therefore become better at collecting taxes while remaining predatory, or improve one administrative function while corruption migrates elsewhere. [MIT Economics](https://economics.mit.edu/research/publications/monitoring-corruption-evidence-field-experiment-indonesia)

For TCE, the central feedback loop should be:

**Control over resources or decisions → opportunities for private advantage → individual choices → protection and patronage networks → altered appointments, enforcement and economic opportunities.**

Reform works by interrupting particular links in that loop. The equations and implementation defaults below are recommendations, not claims that historians have estimated universal behavioral constants.

---

## 1. Mechanisms: what agents and institutions should actually do

### 1.1 Distinguish the behavior, its legality and its consequences

Historical institutions often authorized arrangements that modern bureaucracies prohibit. An office could be purchasable property, and a tax collector could legally retain part of collections. Conversely, an action could comply with formal rules while systematically benefiting a ruling network. Research on venal offices and clientelism makes a simple legal/illegal classification inadequate. [NDL Scholarship](https://scholarship.law.nd.edu/ndlr/vol100/iss1/4/)

Use three separate judgments: **Does the action violate an applicable rule? Do relevant people consider it legitimate? What material and political effects does it have?**

| Type | Operational definition for TCE | Distinction that matters |
| --- | --- | --- |
| **Extortionary bribery** | An official demands a private payment to provide an otherwise lawful entitlement or avoid manufactured obstruction. | The payer may be a victim rather than a willing partner. |
| **Collusive bribery** | A private benefit induces an official to overlook a violation, alter a judgment or grant an unauthorized advantage. | Both parties may gain while outsiders bear the cost. |
| **Embezzlement** | Someone entrusted with resources diverts them from their authorized destination. | Separate theft from spoilage, accounting errors and administrative expenses. |
| **Nepotism** | Kinship changes an appointment or allocation decision relative to the institution’s stated criteria. | A relative is not necessarily incompetent; measure the actual selection distortion. |
| **Patronage** | Jobs, contracts, protection or privileges are allocated to cultivate loyalty. | It can be authorized by a polity’s constitution rather than clandestine. |
| **Clientelism** | Particular benefits are exchanged, contingently, for political support or service. | Targeted assistance alone is not clientelism; the reciprocal political obligation matters. |
| **Institutional capture** | A network gains durable influence over appointments, rules, oversight or adjudication. | Capture can operate through both legal arrangements and illegal transactions. |

These categories should overlap. A steward might embezzle grain, distribute some to clients, pay a superior for protection and appoint a nephew to falsify records. The important simulation object is that **chain of actions**, not a single label attached to the steward.

### 1.2 Generate opportunities from institutional structure

Create a corruption opportunity only when an agent possesses something another agent values or controls resources entrusted by others.

Examples include discretion over irrigation turns, tax assessments, military exemptions, court judgments, admission to an apprenticeship, public purchasing or the release of stored grain. Large flows increase the potential prize; opaque records make diversion harder to demonstrate; restricted alternatives strengthen the official’s bargaining position. These mechanisms follow the principal–agent and market-organization approaches developed by Shleifer and Vishny and subsequent microeconomic research. [Andrei Shleifer](https://shleifer.scholars.harvard.edu/sites/g/files/omnuum10626/files/shleifer/files/corruption.pdf)

**Implementable rule:** each administrative transaction exposes only the actions permitted by its powers and information structure. A clerk who merely records a payment cannot independently sell a judicial acquittal. An official with appointment authority can create opportunities for accomplices.

Do not give every citizen an independent daily probability of “being corrupt.” People without entrusted resources or gatekeeping power have different opportunities, even when their preferences are identical.

### 1.3 Model the individual decision with expected benefits and credible losses

A useful risk-neutral starting point is:

\[
\Delta U\_i
=
B\_i+K\_i-C\_i-M\_i
-\widehat p\_i\left(F\_i+V\_i\right)
\]

The agent chooses the corrupt action when its expected utility exceeds the available alternatives.

Here, \(B\_i\) is the immediate private benefit; \(K\_i\) the value of helping kin, rewarding clients or satisfying a patron; \(C\_i\) the cost of arranging and concealing the action; and \(M\_i\) the agent’s internal or anticipated social cost. \(\widehat p\_i\) is the **perceived** probability of consequential punishment. \(F\_i\) includes restitution, confiscation and other sanctions; \(V\_i\) is the continuation value lost through dismissal or exclusion.

All terms must be in comparable utility or consumption-equivalent units. A prison term cannot simply be added to a quantity of grain.

This formulation has several desirable implications:

* Reliable pay can increase what an official stands to lose, but only when dismissal is credible.
* Patron protection can make a well-paid official willing to accept a bribe.
* A household emergency can change the value of a payment without changing the person’s enduring norms.
* Future access to office can discourage conspicuous theft today—even when that future access is itself valuable partly because of corrupt opportunities.

The wage mechanism is plausible, but the evidence does **not** support a universal salary-to-corruption elasticity. Olken and Pande’s review emphasizes the limited evidence and the interaction between compensation and monitoring. [MIT Economics](https://economics.mit.edu/research/publications/corruption-developing-countries)

For citizens, model **pay, refuse, complain, seek another provider, evade, delay or migrate**. A bribe demand is constrained by affordability and alternatives, not merely by the official’s desired income.

### 1.4 Separate inspection, proof and punishment

Use an enforcement chain:

\[
p\_{\mathrm{punishment}}
=
p\_{\mathrm{exposure}}
\,
p\_{\mathrm{proof}\mid\mathrm{exposure}}
\,
p\_{\mathrm{sanction}\mid\mathrm{proof}}
\]

Exposure may originate from an audit, a witness, a rival, a discrepancy or a public failure. Proof depends on records, investigative skill and evidentiary rules. Sanction depends on courts, political protection and actual enforcement.

For illustration, auditing every account, obtaining usable proof in one-quarter of corrupt cases and imposing sanctions in one-tenth of proven cases produces only a **2.5% punishment probability**. “Universal auditing” is not “certain punishment.”

Agents should estimate this chain from **observed, sufficiently resolved cases**, gossip and institutional reputation. They should not know the simulation’s true probabilities.

Also distinguish actual impartiality from nominal independence. An “independent auditor” appointed, paid and dismissible by the same patron as the suspect may provide little effective separation.

### 1.5 Allow corruption to become self-reinforcing—but not inevitably

Andvig and Moene’s *How Corruption May Corrupt* formalizes the possibility of multiple self-fulfilling corruption equilibria. Persson, Rothstein and Teorell emphasize a related collective-action problem: reform may fail when everyone expects others, including supposed monitors, to participate in the system. Their evidence is institutional and qualitative rather than a universal estimated tipping threshold. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/016726819090053G?utm_source=chatgpt.com)

For TCE, implement reinforcement through local mechanisms:

**Learning:** successful unpunished extraction lowers perceived risk.

**Protection:** superiors who receive benefits suppress investigations.

**Recruitment:** incumbents prefer reliable collaborators, changing who enters office.

**Social pressure:** refusal can mean exclusion from a workplace or patronage network.

**Reduced transaction costs:** trusted collaborators make secret exchanges easier to arrange.

These mechanisms can generate persistent high- and low-corruption states under similar formal rules. However, **bistability should be an emergent possibility, not an imposed law**. Some configurations should produce gradual change or mixed pockets of administrative integrity.

### 1.6 Distinguish serial gatekeepers from competing alternatives

Shleifer and Vishny identify an important difference between officials controlling **complementary approvals** and officials offering **substitute routes**. When a trader must satisfy several independent gatekeepers, each can demand payment without fully considering how the combined burden destroys trade. Genuine alternative providers can instead constrain demands. Coordinated extraction can reduce this particular inefficiency without becoming fair or non-corrupt. [Andrei Shleifer](https://shleifer.scholars.harvard.edu/sites/g/files/omnuum10626/files/shleifer/files/corruption.pdf)

**Implementable rule:** represent which approvals are jointly necessary and which offices can substitute for one another. Do not treat “more offices” or “decentralization” as automatically better or worse.

This allows a captured but coordinated administration to sustain commerce better than a fragmented collection of predatory checkpoints—while still transferring wealth and excluding outsiders.

### 1.7 Make clientelism a repeated, imperfectly observed exchange

Clientelism is not adequately represented by “give gift, receive loyalty points.” Its distinguishing features are particularistic allocation and contingent reciprocity; the literature also stresses substantial variation across democratic and non-democratic settings. [SSRN](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=1839105)

A client should compare expected access to assistance, employment or protection against service obligations and alternatives. A patron should compare the cost of benefits against expected political, military or administrative support.

For TCE:

* Benefits can be food during shortages, legal protection, tenancy, employment or access to scarce inputs.
* Obligations can be public support, turnout, labor, military service or cooperation in appointments.
* Compliance is observed imperfectly. Secret voting must not become perfectly visible to patrons.
* Brokers can maintain relationships the principal cannot personally monitor.

Dependence should increase when alternative insurance or impartial service provision is unreliable. That is a **modeling hypothesis to test**, not an assumption that poor citizens inherently prefer corrupt government.

Wantchekon’s Benin experiment provides evidence that locally targeted political appeals can outperform broad policy appeals in particular settings. It does not establish a universal price for buying votes. [Princeton University](https://www.princeton.edu/~lwantche/Clientelism_and_Voting_Behavior_Wantchekon)

### 1.8 Route consequences through the economy and enforcement system

**A bribe is usually a transfer, not the destruction of its monetary value.** The social losses arise from altered behavior: resources never delivered, harmful rules evaded, useful activity blocked, effort spent arranging protection, or productive investment displaced.

For a hypothetical granary with 100 units, if 12 are stolen and 3 spoil, only 85 remain available for authorized distribution. But the 12 stolen units still exist in somebody’s inventory unless consumed, destroyed or exported.

Use the same principle elsewhere:

| Channel | Mechanism to implement |
| --- | --- |
| Public services | Actual materials, labor and payments determine delivered output. |
| Enforcement | Bribed or protected officials change investigation, judgment and sanction decisions. |
| Investment | Expected extraction and uncertainty change projects’ expected returns. |
| Entry and competition | Connected incumbents obtain advantages or impose obstacles on rivals. |
| Human capital | Appointment and apprenticeship rules change who receives training and responsibility. |
| Discovery and diffusion | Experimentation budgets, expected rewards and access to inputs change; successful techniques may still face adoption barriers. |
| Political stability | Beneficiaries gain loyalty; excluded or victimized agents accumulate grievances from what they observe. |

Do not add a second generic productivity penalty after these losses have already occurred.

---

## 2. Quantitative evidence and calibration parameters

### 2.1 Empirical benchmarks

These are **study-specific estimates or institutional rules**, not transferable constants. “High confidence” below means relatively strong identification within the studied setting, not high confidence that the same effect applies to an agrarian city-state.

Percentage points are abbreviated **pp**.

| Evidence | Quantitative result and units | Interpretation for TCE | Confidence |
| --- | --- | --- | --- |
| **Olken 2007, Indonesian village road projects** | In **608 villages**, announced audit probability increased from about **4% to 100% per project**. Missing expenditure fell by roughly **8 pp**; approximately **20% remained missing** under universal auditing. The full-sample average was about **24%**, not the untreated baseline. | Audit coverage and effective punishment must be separate. Engineering discrepancies are a leakage measure, not individually proven theft. [MIT Economics](https://economics.mit.edu/research/publications/monitoring-corruption-evidence-field-experiment-indonesia) | **High**, randomized intervention; imperfect outcome measurement. |
| **Avis, Ferraz and Finan 2018, Brazilian municipalities** | A previous audit reduced subsequent measured corruption by **8% relative**, and increased subsequent legal-action likelihood by **20% relative**. | Oversight can affect later behavior and the enforcement chain, not just recover current losses. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/699209) | **High**, randomized audit assignment. |
| **Khan, Khwaja and Olken 2016, Pakistan** | Across **482 property-tax circles**, incentive schemes increased revenue by **9.4 log points** after two years—approximately **10% in levels**. Reported going bribe rates rose by about **32%**. | Revenue, taxpayer burden and corruption can increase together. The paper’s “46% higher growth” is not a 46% increase in revenue levels. [MIT Economics](https://economics.mit.edu/research/publications/tax-farming-redux-experimental-evidence-performance-pay-tax-collectors) | **High** for revenue effects; bribe measures are reported rather than directly observed transactions. |
| **Xu 2018, British colonial administration** | Connected governors obtained about **10% higher salaries** through better postings; during patronage, connections were associated with about **4% lower annual colonial revenue** within a posting. Relevant gaps disappeared after the **1930** appointment reform. | Protection can change the behavior of the same official; patronage costs are not solely selection of incompetent people. Colonial revenue is not a welfare measure. [Guo Xu](https://guoxu.org/docs/EmpireJMP_Xu.pdf) | **Moderate–high**, historical quasi-experimental evidence. |
| **Wantchekon 2003, Benin** | In the analyzed noncompetitive districts, participating candidates’ mean vote shares were **84%** in clientelist-message villages, **69%** in public-policy-message villages and **74%** in controls. | Use as a context-specific political-response benchmark, not a cash-bribe elasticity. Treatments concerned campaign platforms and particularistic promises. [Princeton University](https://www.princeton.edu/~lwantche/Clientelism_and_Voting_Behavior_Wantchekon) | **Moderate**, field experiment with a small, selected cluster sample and implementation limitations. |
| **Bandiera, Prat and Valletti 2009, Italy** | **83% of estimated procurement waste** was classified as passive waste rather than active private-benefit seeking. | Expensive purchasing is not synonymous with corruption. Administrative ability and incentives need their own representation. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.99.4.1278) | **Moderate–high** for the study’s decomposition; not a worldwide spending share. |
| **Mauro 1995, cross-country growth** | A **one-standard-deviation improvement** in the corruption index was associated with approximately **0.8 pp higher annual per-capita growth**. | A historical macroeconomic association, not an appropriate direct gameplay coefficient. [UWA Microeconomics Blog](https://uwamicroeconomics.wordpress.com/wp-content/uploads/2013/03/corruption-and-growth-mauro-1995.pdf) | **Low for causal transfer**; identification and robustness are contested. |

**Do not average these numbers.** Their denominators differ: project spending, subsequent audit findings, tax revenue, reported bribe rates and voting outcomes.

### 2.2 What the evidence supports about growth and innovation

The growth literature supports investigating serious economic costs, but not a universal mapping from “corruption score” to GDP growth. Mauro’s influential estimate is particularly unsuitable as a simulation constant: subsequent work by Shaw, Katsaiti and Jurgilas identifies weak-instrument problems in that identification strategy. [United Arab Emirates University](https://research.uaeu.ac.ae/en/publications/corruption-and-growth-under-weak-identification)

Innovation evidence is more useful when decomposed into mechanisms. Akcigit, Baslandze and Lotti link Italian firms, workers, politicians and patents. Politically connected firms show stronger survival and growth in employment and revenue, **without corresponding productivity growth**; market leaders are particularly politically connected and less innovation-intensive. Their close-election design strengthens the firm-growth findings, while the broader innovation patterns should not all be interpreted as independently randomized effects. [INPS Official Site](https://www.inps.it/content/dam/inps-site/pdf/inpscomunica/workinps-papers/7708KEY-14_workinps_papers_ottobre_2018_salome_baslandze.pdf)

Fang, Lerner, Wu and Zhang study China’s post-2012 anticorruption campaign and research subsidies. Following relevant official departures, subsidy allocation became more responsive to innovation-related merit and more predictive of subsequent innovation. The identification relies on particular departure patterns, not random assignment of the entire national campaign. [PubsOnLine](https://pubsonline.informs.org/doi/10.1287/mnsc.2022.4611)

**Recommended translation:** let corruption alter research funding, access to skilled people, entry conditions and appropriability of returns. Do not reduce every inventor’s discovery probability merely because their polity has a high corruption label. Likewise, distinguish **inventing a technique** from **getting permission, capital or materials to deploy it**.

### 2.3 Starting parameters where evidence does not identify universal values

The following are **proposed engineering priors for sensitivity testing**. Their confidence as historical estimates is **unestimated**. They are deliberately separate from the empirical table.

An “accounting episode” means one completed project or one annual office account; choose and record which unit applies.

| Parameter | Initial value | Sensitivity range | Unit and interpretation |
| --- | --- | --- | --- |
| Audit coverage | 0.05 | 0.01–1.00 | Probability an accounting episode receives an audit. |
| Usable evidence conditional on audit of a corrupt episode | 0.40 | 0.10–0.90 | Probability; determined eventually by records, skill and concealment. |
| Sanction conditional on usable proof | 0.50 | 0.05–0.95 | Probability; should become endogenous to courts and protection. |
| Dismissal conditional on sanction | 0.75 | 0.10–1.00 | Probability; institutional punishment rule. |
| Recoverable diverted assets | 0.50 | 0–1.00 | Fraction of identified diverted assets recoverable at enforcement. |
| Feasible diversion ceiling for a tested transaction class | 0.15 | 0–0.50 | Fraction of entrusted flow; not a mandatory amount stolen. |
| Upward patron payment | 0.20 | 0–0.50 | Fraction of illicit proceeds transferred to a protector. |
| Effective half-life of old risk evidence | 1 | 0.25–5 | Years; a proposed belief-memory parameter. |

The diversion ceiling should eventually depend on physical observability and control. It makes little sense to permit the same fraction to disappear from a sealed, jointly counted store and from loosely supervised construction.

Make wages, office horizons, legal fees and appointment rules **institutional inputs**, not universal corruption constants. Appointment for one season, renewable service and hereditary ownership imply different continuation values.

For time-based events, use hazards or correctly converted probabilities. A 5% annual event probability is not a 5% daily probability.

---

## 3. Historical and regional variation—and what reforms changed

### 3.1 Use structural conditions rather than an era progression

| Setting | Historically relevant variation | TCE representation |
| --- | --- | --- |
| **Foragers** | Woodburn describes immediate-return societies, including Hadza and several African and Asian cases, where mobility, access to resources and anti-dominance practices limit dependency. He explicitly distinguishes them from more unequal delayed-return foragers; his generalizations also have important gender limitations. [Libcom](https://files.libcom.org/files/EGALITARIAN%20SOCIETIES%20-%20James%20Woodburn.pdf) | Few permanent administrative rents where offices and stores are absent. Model favoritism and attempted dominance, but do not invent a bureaucratic corruption rate. |
| **Early farming** | Storage, delayed returns and durable asset rights create dependencies absent from some immediate-return systems. The transition is not equivalent to the automatic appearance of a state. [Libcom](https://files.libcom.org/files/EGALITARIAN%20SOCIETIES%20-%20James%20Woodburn.pdf) | As a modeling inference, generate opportunities around common stores, land allocation, irrigation and labor organization only when those institutions actually arise. |
| **Pre-industrial states** | Purchased offices, tax contracts, customary fees and incompletely funded administrations created different boundaries between public duties and private income. France and Qing China illustrate substantially different arrangements. [NBER](https://users.nber.org/~confer/2006/SEGs06/white1.pdf) | Specify the authorized income of each office, who bears administrative costs, and which charges are unauthorized. |
| **Industrializing and colonial administrations** | Merit reform and patronage could coexist across different branches of the same imperial system. Xu’s colonial evidence extends well beyond the nineteenth-century British reform proposals. [Guo Xu](https://guoxu.org/docs/EmpireJMP_Xu.pdf) | Reform particular appointment pipelines and oversight relationships rather than declaring the entire state “modern and clean.” |
| **Modern administrations** | Formal bureaucracies still exhibit leakage, politically connected firms and clientelism; modern monitoring and payment systems can change particular channels. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20141346) | Greater administrative scale increases both transaction volumes and potential monitoring capacity. Neither outcome should automatically dominate. |

This approach accommodates regional diversity without assigning innate corruption coefficients to cultures or populations.

### 3.2 Sale of offices: financing, property rights and capture

In France, office-holding could combine salaries, fees, privileges and property rights. The **1604 paulette** permitted secure transmission in return for an annual payment of **one-sixtieth of the office’s official value**, approximately **1.67% per year**. This is a legal institutional parameter, **not a measured corruption rate**. [NBER](https://users.nber.org/~confer/2006/SEGs06/white1.pdf)

For TCE, model an office’s purchase price as the discounted value of expected income, status and resale or inheritance rights. A cash-constrained ruler may sell offices to obtain immediate finance; purchasers acquire a stake in preserving the arrangement.

Do **not** make the purchase price mechanically cause later extortion because the buyer “must recover it.” Once paid, that price is sunk. Better mechanisms are selection of buyers expecting lucrative rents, outstanding debt-service obligations, authorized fees, and the incentives created by tenure and resale.

Spanish American evidence adds a network mechanism. Guardado finds that purchased positions in oversight councils were associated with greater value of subordinate offices, consistent with weaker supervision and collusion. The documented networks involved territories now including Peru, Bolivia, Ecuador and Chile and could persist after sales ended. [Cambridge University Press](https://www.cambridge.org/core/books/abs/venal-origins-of-development-in-spanish-america/captured-administration/91B87A11390ABF3AAF753CDBDF707161)

**Simulation implication:** selling an oversight office can increase expected rents—and therefore bid prices—throughout the chain below it.

### 3.3 Tax farming: delegation is not automatically theft

Tax farming assigns collection rights to a contractor in exchange for an agreed payment or revenue arrangement. Historical work covers Ottoman territories, medieval Egypt and Mughal India. Ottoman records include advance payments, guarantors and multi-year contracts; one documented 1454 mining-revenue contract lasted **three years**. [SSRN](https://papers.ssrn.com/sol3/Delivery.cfm/SSRN_ID3076104_code2805858.pdf?abstractid=3076104&mirid=1)

Represent three distinct quantities:

\[
\text{legal taxpayer liability},\qquad
\text{actual gross collection},\qquad
\text{treasury remittance}
\]

The difference between collection and remittance is not necessarily embezzlement: it may include authorized compensation, collection expenses and contractual profit.

The proposed behavioral trade-off is straightforward. A residual claimant has incentives to collect effectively, but may over-assess, coerce or conceal collections when oversight is weak. Short horizons can encourage extraction at the expense of future production. Longer horizons can reward preservation of the tax base, but also allow entrenched local control.

Compare tax farming against the state’s **actual alternative**, not a costless, perfectly honest bureaucracy. The Pakistan experiment is especially useful because it demonstrates how collection incentives can improve revenue while worsening private extraction. [MIT Economics](https://economics.mit.edu/research/publications/tax-farming-redux-experimental-evidence-performance-pay-tax-collectors)

### 3.4 Qing China: fund the administration rather than merely forbidding fees

Madeleine Zelin’s *The Magistrate’s Tael* documents the fiscal problem behind eighteenth-century Qing reforms: formal revenue arrangements did not adequately fund local administration, encouraging reliance on informal charges and intermediary networks. Yongzheng-era reforms sought to regularize meltage surcharges and finance official allowances and administrative expenses through more controlled arrangements. [UC Press E-Books Collection](https://publishing.cdlib.org/ucpressebooks/view?brand=ucpress&chunk.id=d0e4349&docId=ft4k4005k7&toc.depth=1&toc.id=d0e4349)

This suggests a crucial TCE mechanism: **an office has operating costs as well as a salary**. Clerks, transport, recordkeeping and investigations require resources. Prohibiting unofficial charges without replacing the funding can reduce service provision or drive the same charges underground.

A reform should therefore be able to convert some formerly informal revenue into explicit, reviewable fees. Legality changes, but legitimacy and efficiency should still depend on who pays, how predictable the charges are and what services are delivered.

### 3.5 British civil service reform: change recruitment and careers

The **1854 Northcote–Trevelyan report** criticized appointments driven by family or political influence and advocated recruitment and promotion arrangements aimed at competence and effort. The Civil Service Commission followed in **1855**. These were institutional changes, not proof that all British patronage disappeared at once. [Civil Servant](https://www.civilservant.org.uk/library/1854_Northcote_Trevelyan_Report.pdf)

Xu’s separate evidence concerns the **1930 reform of colonial gubernatorial appointments**, where a more formal appointment process removed previously observed connection advantages. The distinction matters: different offices changed under different reforms. [Guo Xu](https://guoxu.org/docs/EmpireJMP_Xu.pdf)

For TCE, merit reform should alter the candidate pool, selection weights, promotion criteria and protection from arbitrary patronal dismissal. It should require people capable of administering examinations or assessing performance. An examination can also select the wrong skills or favor those with access to schooling; “exams adopted” should not automatically set corruption to zero.

### 3.6 Hong Kong’s ICAC: enforcement, prevention and public cooperation

Established in **1974**, the ICAC combined investigation and enforcement, corruption prevention, and community education. Its creation addressed, among other problems, corruption embedded in the police and routine public-service access. The institutional lesson is broader than harsher punishment: an external investigative organization, redesign of vulnerable procedures and credible channels for public cooperation were combined. [ICAC](https://www.icac.org.hk/en/about/history/index.html)

Ray Yep’s archival account cautions against a single heroic-reformer explanation. Earlier institutional changes, public pressure and political bargaining mattered; the **1977 partial amnesty** illustrates the compromises involved in maintaining a viable reform coalition. This is evidence about a historical process, not a clean estimate of the isolated effect of “an anticorruption agency.” [CityUHK Scholars](https://scholars.cityu.edu.hk/en/publications/the-crusade-against-corruption-in-hong-kong-in-the-1970s-governor)

**TCE translation:** an agency needs investigators, records access, enforceable authority, operational resources and protection from the networks it investigates. Giving an institution an “independent” name should do nothing by itself.

### 3.7 What tends to work—and what can backfire

The most defensible reform package is **credible oversight plus workable administration**. Audits must lead to evidence and consequences; officials need lawful operating resources; appointment reform must change actual selection; citizens need usable alternatives and reporting channels.

Technology can support this package but is not a substitute for it. Muralidharan, Niehaus and Sukhtankar’s randomized rollout of biometric payment infrastructure in India produced faster, more predictable payments and lower leakage without reducing program access in that trial. Treat this as evidence for a particular payment-system redesign, not for biometrics universally. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20141346)

Participation alone can be insufficient. In the Indonesian road experiment, stronger grassroots participation did not significantly reduce average total missing expenditure; effects depended on what villagers could observe and influence. [MIT Economics](https://economics.mit.edu/research/publications/monitoring-corruption-evidence-field-experiment-indonesia)

Allow reforms to fail through captured monitors, excessive administrative burdens, selective enforcement, displacement into other forms of favoritism or inadequate replacement of the services formerly supplied through patrons.

---

## 4. Stylized facts and validation tests

A correct simulation should support the following patterns. These are **conditional validation targets**, not outcomes every generated world must reproduce.

| Pattern | Validation exercise |
| --- | --- |
| **Auditing reduces some leakage without guaranteeing its elimination.** | Reproduce the direction and approximate magnitude of the Indonesian audit response in a matched scenario; retain imperfect evidence and sanctioning. [MIT Economics](https://economics.mit.edu/research/publications/monitoring-corruption-evidence-field-experiment-indonesia) |
| **Fiscal capacity and predation can rise together.** | Increase collector rewards and test whether treasury receipts and bribe demands can both increase, as in Pakistan. [MIT Economics](https://economics.mit.edu/research/publications/tax-farming-redux-experimental-evidence-performance-pay-tax-collectors) |
| **Protection changes behavior, not just personnel selection.** | Hold an official’s skills fixed while changing patron access; test behavior within the same office, motivated by Xu’s evidence. [Guo Xu](https://guoxu.org/docs/EmpireJMP_Xu.pdf) |
| **Connected firms can prosper without becoming more productive.** | Track firm size, survival, productivity and innovation separately rather than using sales growth as technological progress. [INPS Official Site](https://www.inps.it/content/dam/inps-site/pdf/inpscomunica/workinps-papers/7708KEY-14_workinps_papers_ottobre_2018_salome_baslandze.pdf) |
| **Waste and corruption are not interchangeable.** | An honest but poorly managed procurement office should be able to overpay; a corrupt one may still bargain effectively on some purchases. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.99.4.1278) |
| **Corruption can move when one channel is blocked.** | Audit material purchases while leaving appointments or beneficiary selection weakly monitored; test substitution rather than automatic elimination. [MIT Economics](https://economics.mit.edu/research/publications/corruption-developing-countries) |
| **Capture propagates through oversight relationships.** | Give a network control of an appellate or appointment body and test whether subordinate offices become more valuable and protected. [Cambridge University Press](https://www.cambridge.org/core/books/abs/venal-origins-of-development-in-spanish-america/captured-administration/91B87A11390ABF3AAF753CDBDF707161) |
| **Reporting and true incidence can diverge.** | Improve reporting access while holding behavior fixed. Complaints should rise mechanically; citizens’ perceptions need not equal the hidden true rate. Perception research warns against conflating these measures. [MIT Economics](https://economics.mit.edu/research/publications/corruption-developing-countries) |

Also run conservation tests. Every diverted unit must have a destination, and every administrative expense must be distinguished from diversion. Separately test the model with network reinforcement disabled: this reveals whether persistent corruption is caused by institutions, learned expectations or an accidentally hard-coded trait.

---

## 5. Recommended agent and institution representation

### 5.1 Minimal state

| Object | State worth storing |
| --- | --- |
| **Person** | Assets, income needs, relevant skills, risk preferences, norms, kin obligations, office tenure, perceived enforcement and a bounded set of salient experiences. |
| **Political relationship** | Patron, client or broker; benefits received; obligations; observed reliability; protection capacity; dependency and exit options. |
| **Office** | Powers, jurisdiction, appointment and removal rules, authorized income, workload, operating costs, controlled resources and reporting requirements. |
| **Institution** | Real budget and reported accounts, monitoring resources, record quality, appeal routes, and who controls each enforcement stage. |
| **Transaction** | Authorized entitlement, actual delivery, private transfer, altered decision, witnesses, records and concealment effort. |
| **Case** | Allegation, evidence, investigator, adjudicator, delay, interference and eventual outcome. |

Avoid a universal `corrupt_person: bool`. A person can refuse theft from communal stores while favoring relatives in appointments, or comply when independently monitored while extracting rents under patronal protection.

### 5.2 Event pipeline

A practical sequence is:

```
Administrative opportunity occurs
→ determine lawful entitlement and responsible decision-makers
→ identify feasible deviations and potential partners
→ evaluate actor and counterparty choices
→ transfer actual resources and execute the actual decision
→ create truthful, incomplete or falsified institutional records
→ expose some information through witnesses, audits or consequences
→ resolve investigation and sanction events
→ update memories, relationships, appointments and future choices
```

Keep **world truth, official records and personal knowledge separate**. The renderer may show a furtive grain transfer, but only nearby or informed agents should learn about it.

### 5.3 Capture should be control of a chain, not a threshold on a meter

A network that controls collection but not independent auditing and adjudication has a different position from one controlling all three. Likewise, controlling appointments can be more consequential than receiving many small bribes.

Compute useful institutional diagnostics: who can appoint or dismiss the auditor, who can halt a case, who hears appeals, and whether independent evidence can reach a sanctioning authority.

The UI can then infer descriptions such as:

> “Tax collection is effective, but assessments favor the governor’s clients.”

or:

> “The treasury is well audited; hiring remains dominated by three families.”

Those are more informative than “corruption: 43%.”

### 5.4 Computational simplifications for 10k–50k agents

Use **event-driven opportunities and sparse networks**. Taxes occur with collection cycles; appointments occur at vacancies; contracts occur when institutions buy or build. There is no need for every person to consider bribing every other person each day.

Restrict partner search to people with relevant access, known brokers, kin and existing contacts. Maintain detailed political relationships chiefly where they matter. Recompute network-level diagnostics periodically or after major appointments, not every simulation tick.

A reasonable implementation target is work proportional to the number of relevant relationships and actual transactions, rather than all possible person-pairs. Benchmark this rather than assuming a particular millisecond budget.

Initially simplify bribes into a common valuation unit while transferring actual goods, money or labor obligations. Use richer bargaining only where it changes behavior. Start with **a granary steward, a tax collector and a judge**: together they expose diversion, bargaining and protection without requiring a complete modern procurement system.

### 5.5 Existing models worth adapting

**Ross Hammond’s *Endogenous Transition Dynamics in Corruption: An Agent-Based Computer Model*** is directly relevant. It uses heterogeneous agents and repeated interactions to explore transitions between corrupt and honest behavior. Its spontaneous transitions depend on model assumptions; they are not evidence that real corrupt systems inevitably clean themselves up. [Brookings](https://www.brookings.edu/wp-content/uploads/2016/06/ross.pdf)

A **NetLogo replication by Valery Dzutsati** is available through CoMSES. It is useful as an inspectable prototype and for testing replication, but the repository identifies the submission as not peer reviewed. [CoMSES Net](https://www.comses.net/codebases/4520/releases/1.1.0/)

For TCE, borrow heterogeneous preferences, local information and repeated interaction. Replace generic random pairings with actual jurisdiction, transactions, household needs and patronage relationships. The historical economy should generate the encounters.

---

## 6. Sources, datasets and limits of the evidence

### Data suited to calibration

| Source | Useful variables | Main limitation |
| --- | --- | --- |
| **World Bank Enterprise Surveys** | Firms’ reported bribe requests and experiences across utilities, permits, licenses and taxes; firm characteristics and performance. | Standard surveys largely concern eligible formal private firms. The bribery-incidence indicator is the percentage experiencing at least one request across six transaction types—not the fraction of national income stolen. [DataBank](https://databank.worldbank.org/metadataglossary/world-development-indicators/series/IC.FRM.BRIB.ZS) |
| **V-Dem** | Disaggregated political and institutional measures, with expert coding and measurement-model uncertainty. Version 16 was published in March 2026. | These are not transaction-level leakage percentages; use the codebook and uncertainty estimates rather than treating scores as precise physical quantities. [V-Dem](https://www.v-dem.net/data/the-v-dem-dataset/) |
| **Quality of Government Expert Survey** | Merit recruitment, tenure, political interference and impartiality. Its 2020 wave covers 117 countries using 996 experts. | Expert assessments of bureaucratic structure and behavior, not direct observation of every office. [Göteborgs universitet](https://www.gu.se/en/quality-government/qog-data/data-downloads/qog-expert-survey) |
| **Study-specific audits and transaction records** | Project leakage, subsequent legal action, tax receipts, appointment outcomes and service delivery. | Stronger measurement of a narrow process, usually with limited external validity. The Indonesian, Brazilian and Pakistani studies are especially useful starting benchmarks. [MIT Economics](https://economics.mit.edu/research/publications/monitoring-corruption-evidence-field-experiment-indonesia) |
| **Historical fiscal and appointment records** | Office prices, legal fees, contract duration, remittances, personnel and oversight relationships. | Surviving records are selective. Their administrative categories do not map cleanly onto modern corruption definitions. Zelin and Guardado demonstrate productive ways to analyze them. [UC Press E-Books Collection](https://publishing.cdlib.org/ucpressebooks/view?brand=ucpress&chunk.id=d0e4349&docId=ft4k4005k7&toc.depth=1&toc.id=d0e4349) |

### What is well supported

There is strong evidence that particular opportunities and incentives matter, that monitoring can change behavior, and that political connections can alter allocation and administrative performance. The empirical studies above justify building those mechanisms explicitly.

### What remains contested or thin

There is no defensible universal corruption rate for “early farming,” “medieval government” or an entire world region. Long-run aggregate growth coefficients are much less securely identified than many local administrative effects. The causal effects of broad national reform packages are difficult to separate from concurrent political and economic changes.

Kin preference, patronage and clientelism also require careful classification: support for a relative, a political ally or a needy household is not sufficient by itself to establish illicit diversion. The relevant entitlement, institutional purpose and contingent exchange must be identified.

**Bottom line for TCE:** let corruption redistribute real resources, alter specific decisions and reshape the networks that control future decisions. Let anticorruption emerge from funded administration, credible oversight, alternative access and changing political coalitions. Growth, enforcement, discovery and legitimacy should then respond to what those agents and institutions actually do—not to a universal penalty attached to a corruption label.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928cc-5780-83ea-8566-6941cec961ee)
