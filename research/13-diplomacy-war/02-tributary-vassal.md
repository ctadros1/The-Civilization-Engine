# Tributary and vassal systems: a simulation-ready report for TCE

## Executive recommendation

**Represent unequal relationships as bundles of enforceable obligations and decision rights—not as a single “vassal” status, tribute percentage, or loyalty score.**

A polity might acknowledge another ruler’s ceremonial superiority while retaining its courts and army. Another might surrender foreign-policy authority but pay no regular tribute. A third might supply substantial goods while its incumbent dynasty remained in office. Even “feudal vassalage” often concerned relationships between people and particular landholdings rather than two territorially sovereign states. Treating these arrangements as interchangeable obscures the mechanisms that sustained them. [University of Warwick](https://warwick.ac.uk/fac/arts/history/students/modules/hi294/readings/peter_perdue_the_tenacious_tributary_system_2015.pdf)

For TCE, the essential relationship is:

> **An incumbent governing coalition accepts specified constraints and obligations because the expected benefits of compliance—or costs of resistance—exceed its alternatives. The superior maintains the arrangement because its strategic and material returns exceed enforcement costs.**

That bargain can endure, become more intrusive, weaken, or dissolve. Annexation is one possible outcome, not its natural endpoint.

**Evidence notation used below:** **D** = documented obligation or event; **R** = scholarly reconstruction; **P** = proposed simulation parameter. Confidence is **high, medium, or low for the narrowly stated claim**. High confidence that a treaty demanded something does not establish that it was consistently delivered.

---

## 1. Mechanisms: how unequal relationships worked

### 1.1 Five historical configurations

| Configuration | What the superior obtained | What remained with the subordinate | Implication for TCE |
| --- | --- | --- | --- |
| **Chinese tributary diplomacy** | Recognition of precedence, missions and gifts, participation in an imperial diplomatic order; sometimes strategic cooperation. | Often extensive domestic and diplomatic agency. Different partners understood the relationship differently. | Separate ceremonial rank, diplomatic recognition, commerce, and coercive authority. “Tributary” must not automatically mean tax-paying province. |
| **European feudal vassalage** | Specified service, fidelity, counsel, and sometimes payments associated with tenure or succession. | Rights attached to estates, jurisdictions, offices, and customary privileges; obligations could be personal and geographically limited. | Attach some contracts to people and holdings, not only polity entities. Distinguish a land-tenure relationship from interstate subordination. |
| **Aztec tribute provinces and client states** | Assessed goods from tribute provinces; gifts and military assistance from some frontier clients. | Many local rulers and institutions survived conquest. | Allow fiscally productive provinces and strategically useful clients to have different obligations within one empire. |
| **Mongol overlordship** | Combinations of assessments, livestock or agricultural levies, military support, transport, provisioning, and recognition of supreme authority. | Depending on place and period, existing dynasties and local administrative structures continued. | Combine regular assessments with extraordinary requisitions and logistical services; avoid a universal “Mongol tax.” |
| **Protectorates and subsidiary arrangements** | Control or veto over foreign affairs, military access, advisers, or revenue administration. | A ruler, government, and some internal institutions could remain formally separate. | Model control domain by domain. A resident adviser with compulsory authority can matter more than a nominal tribute payment. |

These are analytical configurations, not uniform historical constitutions. Their diversity is documented by Perdue and Kang for East Asia, Reynolds for medieval Europe, Smith for Aztec fiscal organization, Smith and the *Encyclopaedia Iranica* for Mongol government, and the Pangkor and Tongan arrangements for protectorates. [University of Warwick](https://warwick.ac.uk/fac/arts/history/students/modules/hi294/readings/peter_perdue_the_tenacious_tributary_system_2015.pdf)

### 1.2 Rules a simulation can implement

The following are **modeling rules synthesized from those cases**, not estimated universal behavioral laws.

#### A. Indirect rule is a choice about administrative and strategic costs

The superior should compare at least three alternatives: leave independent, impose an unequal relationship, or administer directly.

A useful decision expression is:

\[
V\_{\text{indirect}}
=
\operatorname{PV}
\left[
\text{remittances}
+\text{strategic services}
-\text{gifts/subsidies}
-\text{monitoring}
-\text{expected enforcement costs}
\right].
\]

Compare this with the corresponding value of annexation, including occupation, salaried administration, transition disruption, and resistance.

**Implementation:** A frontier buffer can be worth retaining even when it produces little revenue. Conversely, a wealthy nearby settlement may become attractive for direct administration once the superior develops sufficient administrative capacity.

Do not let annexation become automatically cheaper merely because the subject has been dependent for many years.

#### B. The subordinate ruler’s interests differ from those of the population

An incumbent may accept burdens that harm households if the agreement secures recognition, military protection, or victory over a domestic rival. The Pangkor settlement of 1874 illustrates the connection between intervention and a disputed succession: British recognition of Abdullah accompanied acceptance of a resident with extensive authority. [National Library Board](https://www.nlb.gov.sg/main/article-detail?cmsuuid=07f0aaea-4348-4e34-947e-69448be4407f)

**Implementation:** Evaluate the bargain separately for the ruler, leading families, military retainers, religious institutions, merchants, and ordinary households. A locally unpopular arrangement may nevertheless remain stable while its beneficiaries control coercion and appointments.

#### C. Assessment, collection, and remittance are different processes

A superior’s demand does not directly remove goods from every household. Local institutions assess liabilities, collect resources, retain authorized shares, absorb losses, and forward a remainder.

**Implementation:** Maintain separate records for:

* what households owe locally;
* what local officials collect;
* what the subordinate owes its superior;
* what reaches the superior.

Authorized local retention is not automatically corruption. Unauthorized diversion is a separate behavior. This distinction is essential when interpreting Aztec estimates of the share of local tax revenue forwarded upward. [Academia](https://www.academia.edu/4618310/_The_Aztec_Empire_2015_A_paper_about_fiscal_organization_and_taxation)

#### D. Fixed obligations make bad harvests politically dangerous

Let a relationship demand a fixed grain quantity while production fluctuates. The effective burden rises during shortages even without an explicit tax increase.

**Implementation:** Permit arrears, remission, substitution, borrowing, concealment, flight, and refusal. A shortfall should first create a fiscal and political dispute—not immediately an independence war.

The superior must choose whether extracting the full amount now is worth damaging future production or undermining the local ruler who collects it.

#### E. Military obligations are contingent contracts, not a universal manpower percentage

The relevant questions are: **who serves, for how long, where, under whose command, with whose equipment, and at whose expense?**

French service records from 1272 contain obligations of different lengths, including four, ten, twenty, thirty, and forty days. Such obligations cannot be translated into “every adult serves forty days annually.” [The Isles Project](https://islesproject.wordpress.com/2007/11/26/500ce-1400-primary-sources-illustrating-european-feudalism/)

**Implementation:** Represent named retainers or eligible service units, a duration, an activation condition, geographical limits, and provisioning responsibilities. Money commutation should finance recruitment; it must not directly create soldiers.

#### F. Logistics determines the realized value of submission

Bulky staples, livestock, valuable manufactures, labor, and cash have different delivery costs. Inka fiscal organization illustrates why staple collection and consumption could be decentralized while portable valuables helped connect the imperial center to local elites. [Academia](https://www.academia.edu/30578708/Wealth_finance_in_the_Inka_empire)

**Implementation:** Tribute requires actual transport and storage. Porters eat, animals require fodder, grain spoils, escorts leave other duties, and blocked routes interrupt payment. The recipient’s usable return is the delivered quantity minus these costs—not the assessed amount.

#### G. Personal succession can reopen an institutional bargain

A deceased ruler’s successor may inherit an office without inheriting equivalent personal credibility, kin connections, or military support. But succession does not necessarily cancel every obligation.

**Implementation:** On succession, reassess recognition, disputed titles, outstanding promises, and the preferences of influential agents. Allow renewal missions, concessions, rival candidates, and conditional refusal. Avoid an automatic rebellion roll for every subject whenever a ruler dies.

#### H. Resistance is a sequence of choices, and weakness is shared information

Distinguish tax concealment, delayed payment, refusal of a military summons, local unrest, elite deposition, and armed secession. Different actors may pursue different objectives.

An overlord’s military defeat can change several subjects’ expectations simultaneously. Robin Law’s account of Oyo describes how the successful Dahomian break weakened remaining western dependencies; their behavior cannot be understood as independent annual coin flips. [dokumen.pub](https://dokumen.pub/the-oyo-empire-c1600-c1836-a-west-african-imperialism-in-the-era-of-the-atlantic-slave-trade-9780751200065.html)

**Implementation:** Distribute information about defeats, successful refusals, rival sponsorship, and failed punitive expeditions through actual communication networks. Let these events alter coalition formation and perceived enforcement probability.

---

## 2. Parameters: defensible anchors and proposed calibration ranges

### 2.1 First establish the denominator

“Tribute was 10%” is incomplete. Ten percent of **harvest, livestock stock, household income, provincial income, collected taxes, or disposable surplus** describes very different burdens.

For TCE, record at least:

\[
b\_{\text{external}}=\frac{T\_{\text{external}}}{Y},
\qquad
b\_{\text{surplus}}=
\frac{T\_{\text{external}}}{Y-C\_{\text{subsistence}}}.
\]

Here, \(Y\) is net production valued at consistent local prices, after intermediate inputs but before household consumption; \(T\_{\text{external}}\) is the interstate transfer. When \(Y\leq C\_{\text{subsistence}}\), record a **subsistence shortfall** rather than treating the second ratio as meaningful.

An illustrative—not historical—economy producing 100 units, needing 80 for subsistence, and paying 10 owes **10% of output but 50% of its subsistence surplus**. If output falls to 80 and the quota stays fixed, payment requires reducing subsistence consumption or drawing down assets.

Keep goods and person-days in their native units. Use shadow prices for comparison, not to make all obligations perfectly interchangeable.

### 2.2 Historical quantitative anchors

| Parameter and case | Value or range | Unit and denominator | Evidence and confidence | Important limitation |
| --- | --- | --- | --- | --- |
| **Aztec imperial extraction: Cuetlaxtlan** | **8.9%** | Estimated tribute/tax to the Triple Alliance divided by reconstructed provincial income | **R; medium–low.** Alfani & Carballo, 2023. | Reconstructed income, not observed national accounts. |
| **Aztec imperial extraction: Tepeyacac** | **14%** | Same denominator; highest provincial estimate reported in the study | **R; medium–low.** Same study. | A case estimate, not a normal rate for every subject. |
| **Aztec imperial extraction: Tlatelolco** | **1.3%** | Same denominator | **R; medium–low.** Same study. | Demonstrates substantial within-empire variation. [ResearchGate](https://www.researchgate.net/publication/371870855_Income_and_inequality_in_the_Aztec_Empire_on_the_eve_of_the_Spanish_conquest) |
| **Morelos city-states: upward fiscal remittance** | **23.4–39.1%** | Imperial payment divided by reconstructed locally collected tax revenue | **R; medium–low.** Smith’s reconstruction of five cases. | **Not a percentage of output or household income.** [Academia](https://www.academia.edu/4618310/_The_Aztec_Empire_2015_A_paper_about_fiscal_organization_and_taxation) |
| **Tlappa payment schedule** | **4 installments per year**; intervals **80, 100, 80, 105 days** | Delivery timing | **D/R; medium.** Documentary reconstruction discussed by Smith. | Other Aztec sources describe different schedules; do not impose one empire-wide calendar. [Academia](https://www.academia.edu/4618310/_The_Aztec_Empire_2015_A_paper_about_fiscal_organization_and_taxation) |
| **Korean tribute missions to China** | Approximately **3 missions/year** in the pattern described | Missions, not tax installments or GDP share | **R; medium.** Spruyt, 2020. | Period- and relationship-specific; mission costs and commercial benefits require separate accounts. [Cambridge University Press](https://www.cambridge.org/core/books/world-imagined/gathering-all-under-heaven/96C25F523B41BAC7CCD20B22FF70CDA3) |
| **Mongol livestock assessment reported by Juvaini** | **1 animal per 100**; herds below **100** exempt in the reported rule | Heads assessed against herd stock | **D as reported by a chronicler; medium.** Smith, 1970. | **A stock-based rule, not 1% of annual output.** Do not universalize it across Mongol territories. [Scribd](https://www.scribd.com/document/696325807/1970-John-Masson-Smith-Jr-Mongol-and-Nomadic-Taxation) |
| **French feudal service, 1272 records** | Examples spanning **4–40 days** | Service days per particular obligation | **D; high for the recorded obligations.** | Duration varies with the obligation; actual campaigns and payment arrangements require separate treatment. [The Isles Project](https://islesproject.wordpress.com/2007/11/26/500ce-1400-primary-sources-illustrating-european-feudalism/) |
| **Treaty of Bassein, 1802: subsidiary force** | At least **6,000 infantry**, with artillery | Maintained force under the agreement | **D; high for the stated term.** Treaty terms reproduced by Grant Duff. | Absolute number belongs to this historical polity, not a scalable universal quota. |
| **Bassein: financing the subsidiary arrangement** | **26 lakh rupees/year**, or **2.6 million** | Assessed annual revenue of ceded districts | **D; high for the stated assessment, lower for realization.** | Neither a GDP share nor necessarily a cash payment of that amount. [Ibiblio](https://www.ibiblio.org/britishraj/Duff3/chapter12.html) |
| **Dahomey’s tribute to Oyo** | **Annual**, reportedly paid in **November** | Payment frequency and season | **D/R; medium.** Law, 1977. | Surviving descriptions of the goods and quantities conflict; avoid a falsely precise standardized basket. [dokumen.pub](https://dokumen.pub/the-oyo-empire-c1600-c1836-a-west-african-imperialism-in-the-era-of-the-atlantic-slave-trade-9780751200065.html) |
| **Universal annual rebellion probability** | **Not established** | Armed secession onsets per dependent polity-year | **Insufficient comparable evidence in the sources reviewed.** | Selected revolts or dependency durations do not supply the required exposure denominator. |

**How to use the Aztec figures:** They support testing unequal provincial burdens, not fixing every agrarian tributary relationship between 1.3% and 14%. Alfani and Carballo’s figures depend on reconstructed populations, incomes, and fiscal allocations. Hammar’s subsequent replication reproduced the main computations but found sensitivity in estimated inequality levels under alternative assumptions; that is a warning against excessive numerical precision, not proof that every provincial tax estimate is wrong. [RWI Essen](https://www.rwi-essen.de/forschung-beratung/forschungsgruppen/prosoziales-verhalten/publikationen/detail/a-comment-on-income-and-inequality-in-25042204)

### 2.3 Proposed TCE calibration parameters

These are **engineering starting points**, not additional historical measurements.

| TCE parameter | Suggested initial setting or experiment | Status and rationale |
| --- | --- | --- |
| **Output-linked external assessment** | Start selected extractive relationships at **5% of net production**; test **1%, 2%, 5%, 10%, 15%, 20%** | **P; low empirical portability.** A sensitivity grid, not a claimed historical distribution. Fixed-goods and household-based assessments must also exist. |
| **Ceremonial relationship assessment** | A specified gift basket and mission schedule; **no default GDP percentage** | **P.** Prevents diplomatic participation from becoming generic fiscal subjection. |
| **Local revenue remitted upward** | Test **20–50% of collected local receipts** | **P**, loosely anchored by the Morelos reconstruction. Do not stack this on an external-output tax unless the accounting explicitly calls for both. |
| **Assessment revision interval** | Test **1, 3, and 5 harvests** | **P.** Creates differences between responsive reassessment and stale quotas without inventing an empirical universal. |
| **Military service duration** | Store contractual days; use **4–40 days** only for a European-tenure-inspired calibration family | Historical case anchor, not a cross-cultural default. Other arrangements need their own service rules. |
| **Political reevaluation frequency** | Event-driven, plus a **monthly** institutional review | **P; computational choice.** Payment, succession, military defeat, and summons events should trigger immediate reconsideration. |
| **Communication and enforcement delay** | Derive from routes, season, transport, and force readiness | Do **not** use an arbitrary “distance loyalty penalty” where TCE already simulates the underlying journey. |
| **Rebellion likelihood** | Produce from agent decisions and coalition capability; calibrate resulting event rates | Do not assign a purported historical annual probability without a suitable dataset. |

Stress-test these settings against food shortages, elite turnover, succession disputes, lost trade routes, and prolonged military absence. Their interaction matters more than finding one “correct” tribute percentage.

---

## 3. Variation across eras and world regions

### 3.1 Use capabilities rather than era gates

The relevant capabilities are **appropriable resources, storage, transport, military reach, recordkeeping, and institutions that reproduce authority**. They need not arrive together or in a fixed sequence.

| Context | Historically relevant variation | TCE treatment |
| --- | --- | --- |
| **Mobile foraging societies** | Durable extraction faces different constraints where people and resources can move. This is not evidence that all foragers were egalitarian. | Make exit, dispersal, and resource mobility affect the feasibility of coercive extraction. Do not unlock hierarchy only after farming. |
| **Sedentary, resource-rich foragers** | Northwest Coast societies demonstrate that nonagricultural communities could have hereditary ranking and slavery. These are not automatically equivalent to interstate vassalage. | Permit concentrated resources and inherited institutions to support hierarchy without agriculture. Donald’s evidence is ethnographic and historical, not a direct description of all prehistoric foragers. [JSTOR](https://www.jstor.org/stable/jj.5973043.8) |
| **Early farming and developing chiefdoms** | Storable and observable production can facilitate extraction, but crop type is not a universally established determinant of state formation. | Model appropriation costs directly. The cereal-appropriability thesis of Mayshar, Moav, and Pascali is influential but has received a recent robustness challenge. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/718372) |
| **Pre-industrial agrarian and pastoral states** | Different fiscal bases supported goods assessments, livestock levies, labor obligations, service tenures, and commercial revenues. | Let available production and administrative practices determine assessment bases, rather than assigning one “medieval” tax system. [Scribd](https://www.scribd.com/document/696325807/1970-John-Masson-Smith-Jr-Mongol-and-Nomadic-Taxation) |
| **Industrial-age protectorates** | Separate governments could coexist with externally controlled diplomacy, resident advisers, and financed foreign forces. | Expand the domains in which an external actor can exercise authority; retain the distinction between formal institutions and actual control. [Hansard](https://hansard.parliament.uk/commons/1876-05-26/debates/4143d2d1-96b9-4755-8e41-52698180d018/Observations) |
| **Modern asymmetric relationships** | Small states retain agency even where stronger neighbors exert substantial influence. Bhutan’s diplomatic history is a useful corrective to accounts that treat its choices as wholly externally determined. | Represent specific commitments and constraints. Do not automatically classify alliances, aid dependence, or unequal bargaining power as vassalage. [ResearchGate](https://www.researchgate.net/publication/353427975_Beyond_India_and_China_Bhutan_as_a_Small_State_in_International_Relations?utm_source=chatgpt.com) |

### 3.2 Regional differences that deserve distinct mechanisms

**The Andes:** Inka institutions mobilized rotating household labor for agriculture, construction, manufacture, and other state activities. Stored produce supported armies, officials, and laborers. This calls for a **labor-and-provisioning account**, not merely a grain tax converted into money. Earle also distinguishes staple finance from the manufacture and circulation of valuable goods. [Academia](https://www.academia.edu/30578708/Wealth_finance_in_the_Inka_empire)

**West Africa:** Oyo’s relationship with Dahomey combined a regular external payment with considerable local autonomy and an intermittent threat of intervention. Law cautions against reconstructing a comprehensive formal constitution of Oyo “rights” from isolated interventions. **Repeated successful demands need not imply an unlimited legal entitlement.** [dokumen.pub](https://dokumen.pub/the-oyo-empire-c1600-c1836-a-west-african-imperialism-in-the-era-of-the-atlantic-slave-trade-9780751200065.html)

**South and Southeast Asia:** Studies of “galactic” political formations emphasize layered centers and semi-autonomous peripheral authorities. For TCE, this supports influence organized around settlements, ports, routes, and rulers rather than exclusively around uniformly administered territorial polygons. [ResearchGate](https://www.researchgate.net/publication/396636887_Circles_of_Kings_and_Right_of_the_Port_Maritime_Violence_and_the_Galactic_Polity_in_14th_Century_Sri_Lanka)

**East Asia:** Kang emphasizes hierarchy and legitimacy; Perdue challenges the idea that one coherent “tributary system” explains the full variety of diplomatic, commercial, and military relationships. The simulation should be able to generate both stable recognition-based relations and coercive conflict without choosing one interpretation as an immutable regional rule. [Taylor & Francis Online](https://www.tandfonline.com/doi/full/10.1080/09636412.2010.524079)

---

## 4. Stability, rebellion, transitions, and stylized facts

### 4.1 What can be quantified about stability?

**Dependency duration, regime survival, payment compliance, and rebellion frequency are different variables.** A relationship can last decades despite repeated disputes; a short relationship may end through peaceful renegotiation or annexation rather than rebellion.

Selected historical spells demonstrate different outcomes:

| Relationship | Observed interval or transition | What it establishes—and does not establish |
| --- | --- | --- |
| **Dahomey under Oyo** | Settlement in **1748** to successful revolt in **1823**: approximately **75 years** | Long-lived subordination could end when enforcement capacity weakened. This does not establish 75 uninterrupted years without resistance. [dokumen.pub](https://dokumen.pub/the-oyo-empire-c1600-c1836-a-west-african-imperialism-in-the-era-of-the-atlantic-slave-trade-9780751200065.html) |
| **Tonga under British protectorship** | **1900–1970**: approximately **70 years** | Protected status could end without annexation. Tongan institutions persisted, although British powers expanded through amendments. [Office of the Historian](https://history.state.gov/countries/tonga) |
| **Korea under Japanese protectorate** | **1905–1910**: approximately **5 years** before annexation | External control could deepen rapidly into colonial rule. The duration is not an annual annexation probability. [Korean Legal Studies](https://kls.law.columbia.edu/content/korea-legal-history) |
| **Perak after Pangkor** | Agreement in **1874**, resident’s killing and intervention in **1875** | Attempts to exercise new authority could produce a rapid crisis. It is not a representative protectorate rebellion rate. [Hansard](https://hansard.parliament.uk/commons/1876-05-26/debates/4143d2d1-96b9-4755-8e41-52698180d018/Observations) |

The reviewed sources do **not** support a defensible universal annual rebellion rate spanning all these systems. Reporting, for example, “vassals rebelled with a 3–8% annual probability” would manufacture precision.

For an eventual TCE calibration dataset, define:

\[
r\_{\text{secession}}
=
\frac{\text{new armed secession episodes}}
{\text{dependent polity-years exposed to that risk}}.
\]

Code payment refusal, local revolt, dynastic replacement, and secession separately. Record repeated episodes, uncertain dates, missing observations, and competing outcomes such as annexation or negotiated release. Do not treat a chronicle’s label “rebellion” as automatically meaning a popular independence movement.

### 4.2 Generate conditional risk, not random disloyalty

A practical subordinate decision model should compare:

\[
EU(\text{resist})-EU(\text{comply}),
\]

using expected military success, punishment, external support, domestic coalition strength, fiscal burden, dynastic security, and the value of protection.

Household grievances and elite willingness should not be collapsed into one number. A hungry population may lack organization; a prosperous ruling faction may seek independence because an outside patron makes success plausible.

When an aggregate hazard is needed for fast-forward, use:

\[
P(\text{event during }\Delta t)=1-e^{-\lambda(X)\Delta t},
\]

with \(\lambda\) expressed per year and \(X\) describing the simulated circumstances. This is a **mathematical implementation choice**, not a historically estimated hazard. It prevents applying an annual probability independently every month.

### 4.3 Transition rules

**Toward annexation.** Make annexation possible when the superior both desires additional control and can staff and defend it. Pretexts, succession disputes, payment crises, or existing advisers may create opportunities, but administrative replacement must actually occur. Korea’s sequence demonstrates the need to represent increasing control before formal annexation. [Korean Legal Studies](https://kls.law.columbia.edu/content/korea-legal-history)

**Toward independence.** Allow successful resistance, negotiated release, superior withdrawal, collapse of enforcement, and replacement by another patron. Ending a relationship does not require exterminating the superior’s armies or conquering its capital.

**Toward renewed subordination.** A rebellion can end in restored payments, altered obligations, a new ruler, or a garrison. TCE should not force every conflict into annexation versus full independence.

**Toward institutional integration without annexation.** Shared courts, fiscal offices, appointment procedures, or military commands can deepen dependence while a distinct polity survives. Conversely, a formally strong claim may become practically unenforceable.

These are proposed transition mechanisms, not a compulsory sequence.

### 4.4 Stylized facts a correct simulation should reproduce

| Target pattern | Observable test |
| --- | --- |
| **One empire contains materially different dependency bargains.** | Different subjects supply different combinations of goods, soldiers, recognition, and access. Aztec tribute provinces and strategic clients provide a historical benchmark. [Academia](https://www.academia.edu/4618310/_The_Aztec_Empire_2015_A_paper_about_fiscal_organization_and_taxation) |
| **External burdens vary substantially between subjects.** | A successful calibration should permit variation comparable in direction—not necessarily exact magnitude—to the reconstructed Aztec provincial differences. [ResearchGate](https://www.researchgate.net/publication/371870855_Income_and_inequality_in_the_Aztec_Empire_on_the_eve_of_the_Spanish_conquest) |
| **Service capacity is seasonal and contract-limited.** | A summons can fail or require additional payment because service expires, not only because “loyalty” is low. [The Isles Project](https://islesproject.wordpress.com/2007/11/26/500ce-1400-primary-sources-illustrating-european-feudalism/) |
| **Nominal extraction exceeds the center’s usable receipts.** | Collection costs, retained local revenues, transport, and provisioning reduce what the superior can actually spend. [Academia](https://www.academia.edu/4618310/_The_Aztec_Empire_2015_A_paper_about_fiscal_organization_and_taxation) |
| **Dependency can last several generations without becoming annexation.** | Simulated relationships can survive for roughly seventy years or longer while retaining separate institutions, as the Oyo–Dahomey and Tongan cases illustrate. [dokumen.pub](https://dokumen.pub/the-oyo-empire-c1600-c1836-a-west-african-imperialism-in-the-era-of-the-atlantic-slave-trade-9780751200065.html) |
| **Political failures can spread across the dependency network.** | A superior’s defeat changes several subjects’ behavior, producing clustered defections rather than independent random events. [dokumen.pub](https://dokumen.pub/the-oyo-empire-c1600-c1836-a-west-african-imperialism-in-the-era-of-the-atlantic-slave-trade-9780751200065.html) |
| **Formal rank and actual decision-making power need not coincide.** | A ceremony, treaty label, or nominal title does not automatically imply control over taxation, courts, or foreign relations. [University of Warwick](https://warwick.ac.uk/fac/arts/history/students/modules/hi294/readings/peter_perdue_the_tenacious_tributary_system_2015.pdf) |

---

## 5. Modeling recommendation for TCE

### 5.1 Use a multilayer relationship graph

Store an `UnequalRelationship` between governing institutions, with optional links to particular rulers, lineages, holdings, and settlements.

| Component | Minimum contents |
| --- | --- |
| **Parties and recognition** | Superior, subordinate, recognized incumbent, contested claimants, witnesses or guarantors |
| **Fiscal obligations** | Assessment base, goods or currency, rate or quota, schedule, exemptions, revisions, arrears, remission |
| **Military obligations** | Eligible service units, number, duration, scope, command, equipment, transport, provisioning |
| **Decision rights** | Diplomacy, war declaration, appointments, succession recognition, taxation, courts, customs |
| **Superior’s commitments** | Protection, subsidies, gifts, recognition, trade privileges, dispute settlement |
| **Enforcement presence** | Resident officials, inspectors, garrisons, hostages, locally aligned factions |
| **Observed performance** | Payments received, summons honored, protection delivered, breaches, current disputes |
| **Institutional continuity** | Whether terms bind a person, dynasty, office, community, holding, or successor government |

Keep **claimed rights**, **recognized rights**, and **exercised rights** separate. The superior may claim control over a subject that has stopped paying and can no longer be compelled.

Permit multiple relationships where their domains differ. A single global hierarchy tree is convenient, but it cannot express every divided or overlapping authority. For fiscal forwarding, prevent circular accounting or explicitly track resource provenance so the same goods cannot circulate as newly created revenue.

### 5.2 Let individuals make the relationship visible

At TCE’s population scale, the daily consequences are more valuable than additional abstract diplomatic statistics.

A payment should involve households producing goods, an official assessing obligations, a local collector assembling them, and carriers delivering them. A summons should remove actual people from farms and workshops. A resident adviser should be a person who needs information, staff, housing, and protection. A succession mission should occupy envoys and consume resources.

Institutions can aggregate decisions without erasing those processes. The assessment ledger belongs to an office; its current holder may be competent, partisan, or corrupt. A ruler can order payment, but available household stocks and collection capacity determine what happens.

**Do not scale historical absolute quotas directly into a small world.** The Bassein force is a historical anchor for the structure of a subsidiary arrangement, not a sensible default for a simulation whose entire population may be only 10,000 people.

### 5.3 Computation and simplification

Use daily simulation for production, transport, food consumption, and active military service. Evaluate ordinary political relationships monthly, with immediate event triggers for payments, successions, defeats, and major breaches.

Maintain cached fiscal and military summaries for institutions and update them when relevant agents or obligations change. Relationship processing should be approximately proportional to the number of active edges, rather than requiring every person to evaluate every polity.

For v1, simplify the language of treaties into authored clause types. Do **not** simplify away the distinction between fiscal and military obligations, the identity of beneficiaries, or enforcement delays. Those distinctions generate much of the historical behavior.

A single “loyalty” value may be useful as a display summary. It should be an output of interests, relationships, and perceived alternatives—not the hidden cause of all compliance.

### 5.4 Existing models and games

| Model or game | Useful precedent | What not to import uncritically |
| --- | --- | --- |
| **Gavrilets, Anderson & Turchin, “Cycling in the Complexity of Early Societies” (2010)** | A close academic starting point: nested polities, upward resource flows, conquest, secession, and fragmentation. Experiments include tribute fractions **0.1–0.3**, spans of control **5–7**, and mean ruler durations **5–20 years**. | These are **model settings**, not universal historical estimates. Replace mechanical fragmentation assumptions with TCE’s succession institutions and actual coalitions. [sociostudies.org](https://www.sociostudies.org/almanac/articles/cycling_in_the_complexity_of_early_societies/) |
| **Axelrod, “A Model of the Emergence of New Political Actors” (1995)** | Demonstrates how local interactions can generate larger political actors rather than starting with fixed, permanent states. | Its abstract actors are not substitutes for household production, logistics, or differentiated governing institutions. [Taylor & Francis](https://www.taylorfrancis.com/chapters/oa-edit/10.4324/9780203993699-9/model-emergence-new-political-actors-robert-axelrod) |
| **Stellaris: Overlord, documented 2022 design** | Negotiable subject agreements, differentiated obligations, subsidies, and subject specialization demonstrate a useful modular contract interface. | Resource percentages and loyalty progression are game abstractions. TCE should resolve their consequences through inventories, people, institutions, and credible alternatives. [Steam Store](https://store.steampowered.com/news/posts/?appids=281990&enddate=1651072511&feed=steam_community_announcements) |

---

## 6. Sources, datasets, and evidence limits

### 6.1 Priority scholarly reading

**For the five requested systems:** Read Peter Perdue’s *“The Tenacious Tributary System”* (2015) alongside David Kang’s *“Hierarchy and Legitimacy in International Systems”* (2010); Susan Reynolds’s *Fiefs and Vassals* (1994); Michael E. Smith’s *“The Aztec Empire”* (2015); and John Masson Smith Jr.’s *“Mongol and Nomadic Taxation”* (1970). They provide both mechanisms and warnings about overgeneralizing institutional labels. [University of Warwick](https://warwick.ac.uk/fac/arts/history/students/modules/hi294/readings/peter_perdue_the_tenacious_tributary_system_2015.pdf)

**For quantitative fiscal reconstruction:** Alfani and Carballo’s *“Income and Inequality in the Aztec Empire on the Eve of the Spanish Conquest”* (2023), accompanied by its replication materials, is especially useful. Read it alongside Hammar’s 2025 replication comment rather than treating point estimates as direct observations. [PubMed](https://pubmed.ncbi.nlm.nih.gov/37365407/)

**For non-European institutional variation:** Robin Law’s *The Oyo Empire, c.1600–c.1836* (1977) and Timothy Earle’s *“Wealth Finance in the Inka Empire”* (1994) are particularly productive for implementation. Both connect political relationships to how resources actually moved. [dokumen.pub](https://dokumen.pub/the-oyo-empire-c1600-c1836-a-west-african-imperialism-in-the-era-of-the-atlantic-slave-trade-9780751200065.html)

### 6.2 Datasets and structured source material

| Resource | Best use in TCE research | Limitation |
| --- | --- | --- |
| **Seshat Global History Databank** | Comparative polity scale, institutions, hierarchy, and historical trajectories. The official data page lists **Polaris 2026**, alongside earlier releases. | Not a ready-made annual panel of tributary payments and rebellions. Retain uncertainty and source provenance. [Seshat Databank](https://seshatdatabank.info/data) |
| **D-PLACE** | Cross-cultural comparison of subsistence, political organization, and social differentiation. | Ethnographic observations come from particular places and observation periods; they are not a continuous record of ancient societies. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) |
| **Alfani–Carballo replication data, OpenICPSR project 186521, version 2** | Inspecting assumptions behind Aztec provincial income and extraction estimates; sensitivity analysis. | Reconstruction inputs and outputs, not raw observations of GDP. [ResearchGate](https://www.researchgate.net/publication/371870855_Income_and_inequality_in_the_Aztec_Empire_on_the_eve_of_the_Spanish_conquest) |
| **CShapes 2.0** | Territorial and capital changes for independent and dependent territories, **1886–2019**. | Formal territorial coding does not itself identify practical autonomy, contractual obligations, or rebellion onset. [Library of Congress](https://www.loc.gov/item/2023592015) |
| **Tribute registers, service rolls, and treaty texts** | Goods baskets, due dates, named service obligations, exemptions, and decision rights. | Prescribed liabilities are not necessarily realized payments; surviving records are uneven. Examples include the Aztec registers, French service records, and Bassein terms used above. [Academia](https://www.academia.edu/4618310/_The_Aztec_Empire_2015_A_paper_about_fiscal_organization_and_taxation) |

### 6.3 Claims to flag explicitly in the design documentation

**A single Chinese tributary system is contested.** It is an interpretive framework, not a uniform treaty constitution covering every relationship. Likewise, a single standardized European “feudal system” is an unsafe template. [University of Warwick](https://warwick.ac.uk/fac/arts/history/students/modules/hi294/readings/peter_perdue_the_tenacious_tributary_system_2015.pdf)

**Headline fiscal rules omit additional burdens.** A livestock assessment, agricultural tithe, or fixed payment does not capture provisioning, transport, military service, extraordinary requisitions, or the distribution of costs within the subordinate society. Mongol evidence is particularly clear about the coexistence of different exactions. [Scribd](https://www.scribd.com/document/696325807/1970-John-Masson-Smith-Jr-Mongol-and-Nomadic-Taxation)

**A treaty is evidence of a claimed arrangement, not necessarily consent or effective control.** Read its implementation and the perspectives of subordinate actors alongside its formal clauses. The gap between externally asserted authority and local interpretation is especially important in protectorate histories. [Hansard](https://hansard.parliament.uk/commons/1876-05-26/debates/4143d2d1-96b9-4755-8e41-52698180d018/Observations)

**The largest unresolved calibration gap is comparative rebellion exposure.** The sources provide useful obligations, fiscal reconstructions, crises, and dependency spells, but not a harmonized cross-system denominator from which to derive one annual rebellion rate.

---

## Bottom line

The best TCE implementation is a **fiscal, military, and political relationship graph grounded in individual production and institutional authority**.

Its central dynamics should be burden sharing, retained local power, protection, credible enforcement, succession, and alternative patrons. Stable dependence, negotiated autonomy, opportunistic refusal, cascading fragmentation, and annexation should then emerge from the same rules.

A tributary relationship should persist because real people can finance, legitimate, and enforce it—and fail when that arrangement stops working—not because an integration timer expires.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9294c-4408-83ea-a7d3-9ea7e47ec64d)
