# Public works: organization, financing, and maintenance

## Executive recommendation

For TCE, **model infrastructure as a continuing relationship between an asset, its users, and an organization—not as a construction purchase followed by a fixed upkeep charge.**

Separate who authorizes a project, who supplies resources, who builds it, who controls access, and who must maintain it. Those roles need not belong to the same institution. Roman aqueduct administration employed dedicated maintenance establishments; English parishes imposed statutory road labor; Andean communities renew a suspension bridge through recurring collective work; Islamic endowments attached services to income-producing property. These are different ways of making future work happen, not simply different names for a treasury. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Frontinus/De_Aquis/Bennett/2%2A.html)

The most important physical distinction is equally fundamental: **gradual wear, reversible obstruction, component failure, and catastrophic damage are different processes.** A canal can stop delivering water while its masonry remains intact. A regularly resurfaced road can survive indefinitely as an institutionally maintained system without any original wearing surface surviving. Conversely, good routine upkeep does not guarantee survival under an exceptional flood. [Frontiers](https://www.frontiersin.org/journals/built-environment/articles/10.3389/fbuil.2020.00003/full)

---

## 1. Mechanisms: rules TCE can implement

### 1.1 Who organized and paid?

“Public infrastructure” should describe a service or shared consequence, not a single ownership category. TCE should allow overlapping community, religious, municipal, royal, and commercial arrangements.

| Arrangement | Historical mechanism | Simulation rule |
| --- | --- | --- |
| **Household and user associations** | Beneficiaries organize contributions and operating rules. Research on Nepalese irrigation found that farmer-managed systems could outperform agency-managed systems despite less sophisticated physical infrastructure. Community management was not necessarily egalitarian or conflict-free. [Sage Journals](https://journals.sagepub.com/doi/10.3233/HSM-1994-13305) | Define a beneficiary group, contribution rule, decision procedure, monitoring arrangement, and sanctions. Water access, reputation, fines, or future assistance can depend on contribution. |
| **Corvée and statutory labor** | Authorities require eligible households to supply work, substitutes, animals, or carts. England’s 1555 highway statute combined parish supervisors, specified workdays, differentiated obligations, and penalties. [Constitution Society](https://constitution.org/1-History/sech/sech_078.htm) | Issue dated obligations against households. Attendance competes with agriculture, household work, and wage employment. Track exemptions, substitution, enforcement, and arrears. |
| **Liturgies and elite benefaction** | Athenian liturgies assigned expensive public responsibilities to wealthy people; naval service is especially well documented. These were not synonymous with all infrastructure gifts. Kaiser models the interaction between public obligations, private wealth information, and incentives to contribute or evade. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/athenian-trierarchy-mechanism-design-for-the-private-provision-of-public-goods/831A8086B9E8EA9309DD2332D16D777A) | Distinguish an **assigned elite obligation** from a **voluntary donation**. Sponsors value honor, influence, property appreciation, and service benefits. Construction sponsorship does not automatically establish maintenance funding. |
| **Municipal or sovereign expenditure** | Officials commission works using public revenues and supervise—or fail to supervise—delivery. Pliny’s correspondence about Nicomedia describes abandoned aqueduct projects, a request for engineering expertise, and an imperial demand to investigate wasted expenditure. [ToposText](https://topostext.org/work/198) | Maintain project appropriations separately from completed work. Require land access, competent designers, procurement, and inspections. Councils and rulers may favor prestige or supporters rather than the highest social return. |
| **Earmarked tolls and trusts** | English turnpike trusts obtained authority to collect tolls and borrow against revenues. Their investment exceeded the earlier parish provision in Bogart’s study. Medieval towns also used dedicated levies such as murage for walls. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/did-turnpike-trusts-increase-transportation-investment-in-eighteenthcentury-england/B22083C49AA94D6102E90AEF86DCF355) | Attach revenue rights to an asset or network. Revenue depends on actual traffic and collection. Allow borrowing, leakage, toll avoidance, and diversion to alternative routes. |
| **Concessions** | A public authority grants an operator contractual rights and obligations. Modern concession guidance explicitly addresses the temptation to defer major renewals near contract expiry, using inspections, residual-life requirements, and financial security. [Federal Highway Administration](https://www.fhwa.dot.gov/ipd/p3/toolkit/publications/model_contract_guides/core_toll_concession_0814/) | Specify term, tariff rights, service obligations, renewal responsibilities, enforcement, and reversion. Maintenance incentives depend on remaining revenue, sanctions, and the condition required at handover. |
| **Religious endowments, including waqf** | A founder dedicates income-producing property to specified purposes. Its earnings support services and associated expenses under an administrator and governing rules. The revenue-producing property itself must remain productive. [Sites@Duke Express](https://sites.duke.edu/timurkuran/files/2016/10/waqf-2001-1.original.pdf) | Create a ring-fenced portfolio of farms, shops, rents, or other claims. Pay administration and property upkeep before distributing available service funding according to the deed. Do not generate a guaranteed annual yield from an abstract “endowment” flag. |

Two distinctions prevent major modeling errors.

**First, financing is not the same as ultimate funding.** A loan makes resources available now but creates future claims on taxes, tolls, rents, or other receipts. A concession similarly changes contractual responsibility; it does not remove resource costs.

**Second, unpaid labor is not costless labor.** In TCE, corvée consumes people’s time, food, tools, transport capacity, and potentially lost harvest output. Its fiscal attraction can coexist with substantial household losses. Keep compulsory public labor, slavery, hired work, and voluntary cooperation as distinct labor relationships: their legal status and incentives should not be collapsed into one efficiency modifier.

### 1.2 Collective action depends on who benefits and who can refuse

A useful infrastructure association must solve two different problems: **providing and maintaining the shared facility**, and **allocating its benefits**. Irrigation makes the distinction especially clear: contributing to a canal does not settle who receives scarce water. Nepalese irrigation research and Mosse’s work on South Indian tanks show why physical infrastructure and the organization of users must be modeled together. [Sage Journals](https://journals.sagepub.com/doi/10.3233/HSM-1994-13305)

A suitable TCE participation rule is:

\[
\text{support}\_{i,p}
=
\text{expected benefit}\_{i,p}
+\text{status/norm reward}\_{i,p}
-\text{contribution burden}\_{i,p}
-\text{expected unfairness}\_{i,p}.
\]

This is a **proposed behavioral decomposition**, not a historically estimated equation. Its terms should depend on concrete circumstances:

An upstream farmer may expect water even without contributing. A downstream farmer may refuse because the allocation rule seems unjust. A merchant may support a bridge while opposing a toll that diverts customers elsewhere. A household may support canal cleaning in the slack season but resist the identical obligation during harvest.

Consequently, increasing coercion should not automatically improve project performance. It may raise immediate attendance while worsening avoidance, trust, political opposition, or household production.

### 1.3 Decisions should proceed through institutions, not an omniscient planner

Use the following project lifecycle.

**Proposal.** Requests arise from observed problems: long water queues, repeated wagon delays, flooded fields, a damaged gate, or complaints from influential residents. Institutions can also propose prestige and strategic projects without a service crisis.

**Authorization.** The relevant assembly, council, proprietor, foundation administrator, or ruler chooses among maintenance, repair, replacement, expansion, and doing nothing. Different institutions weight beneficiaries differently. They also have imperfect information about condition and future demand.

**Commitment.** A project receives a resource plan, work calendar, land rights, responsible organizer, and maintenance arrangement. Permit construction without a credible maintenance arrangement—but let that omission produce consequences rather than silently supplying free upkeep.

**Execution and acceptance.** Crews deliver measurable quantities. Inspectors assess completion and hidden defects. Recorded expenditure and actual delivered resources are separate.

**Operation and reconsideration.** Users experience the service. Complaints, failures, inspections, and revenue results can cause rule changes, additional levies, institutional replacement, or abandonment.

The distinction between spending and delivery has strong empirical support. In Olken’s randomized study of more than 600 Indonesian village road projects, increasing announced government audit probability from 4% to 100% reduced “missing expenditures” by about **8 percentage points**. That measure compared official expenditure with independent engineering estimates; it is not a universal corruption rate or a direct measure of every form of theft. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/517935)

### 1.4 Maintenance needs an organization of its own

Give recurring upkeep its own workers, accounts, inspections, and authority. Do not assume that a successful construction mobilization persists indefinitely.

Frontinus describes distinct Roman maintenance establishments, specialized duties, and workers diverted to private tasks. His administrative response included specifying work and checking its execution. This is a useful historical precedent for modeling the gap between nominal staffing and effective maintenance effort. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Frontinus/De_Aquis/Bennett/2%2A.html)

For TCE, an institution’s practical maintenance capacity should be limited by the scarcest required input:

\[
\text{work completed}
=
\min
\left(
\text{available labor capacity},
\text{tool/material capacity},
\text{specialist capacity},
\text{accessible workfront capacity}
\right).
\]

A treasury full of money cannot repair a bridge without carpenters, timber, access, and authority to close it.

---

## 2. Parameters, maintenance cycles, and costs

### 2.1 How to interpret the evidence

The tables distinguish:

* **Documented practice or observation:** what a particular source reports.
* **Prescribed rule or engineering allowance:** an obligation or planning input, not necessarily actual performance.
* **TCE assumption:** a proposed value requiring sensitivity testing.

“High confidence” below means confidence in the **scoped source statement**, not confidence that its number transfers to every society.

### Organization and maintenance cycles

| Parameter and setting | Value and units | Evidence type; confidence | Appropriate use in TCE |
| --- | --- | --- | --- |
| English parish highway labor, **1555 statute** | **4 days/year**, **8 hours/day**; therefore **32 prescribed hours** for a qualifying labor obligation | Legal prescription; **high** for the rule, low for realized compliance | A specific statutory-labor template, not a universal corvée burden. The law also specified substitutes and differentiated cart obligations. [Constitution Society](https://constitution.org/1-History/sech/sech_078.htm) |
| English parish road organization, same statute | **2 surveyors per parish** | Legal prescription; **high** | Demonstrates supervision as a separate obligation. Do not scale mechanically by population. [Constitution Society](https://constitution.org/1-History/sech/sech_078.htm) |
| Roman aqueduct maintenance establishments, Frontinus | **240 public + 460 imperial personnel = 700** | Contemporary administrative report; **medium–high** for reported establishment | A system-specific staffing anchor; not 700 workers continuously repairing, nor a general workers-per-kilometer ratio. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Frontinus/De_Aquis/Bennett/2%2A.html) |
| Q’eswachaka bridge, Peru | **Annual renewal**; **4 communities**, **2 master builders**, **3 principal construction days** | Documented living practice; **high** | Separate household rope preparation from the three principal assembly days. Annual replacement can coexist with long institutional continuity. [UNESCO ICH](https://ich.unesco.org/en/RL/knowledge-skills-and-rituals-related-to-the-annual-renewal-of-the-q-eswachaka-bridge-00594) |
| Dujiangyan irrigation, China | **Annual repair regime** | Historical documentary study; **medium–high** for the recurring institution | Schedule major maintenance around water conditions and agricultural needs. Do not infer an unchanged annual labor bill across centuries. [Xhu Academy](https://aj.xhu.edu.cn/xhdxxbzskb/en/article/doi/10.19642/j.issn.1672-8505.2018.04.002) |
| Great Mosque of Djenné, Mali | **Annual replastering campaigns** | UNESCO conservation account; **high** for this building | A material-specific recurring task. The same account records structural weakness despite annual surface care. [UNESCO World Heritage Centre](https://whc.unesco.org/en/news/574) |
| USACE levee inspection program, **2013 guidance** | Routine inspection **annually**; comprehensive inspection **every 5 years** | Program prescription; **high**, jurisdiction- and date-specific | A modern administrative benchmark, not a universal physical maintenance interval. [NWO](https://www.nwo.usace.army.mil/Media/Fact-Sheets/Fact-Sheet-Article-View/Article/487691/levee-safety-action-classification/) |

### Physical deterioration and resource costs

| Parameter and setting | Value and units | Evidence and confidence | Transfer limits |
| --- | --- | --- | --- |
| Ordinary-soil excavation allowance, ILO labor-based road manual for Timor-Leste, **2016** | **1–1.5 m³ per unskilled workday** | Engineering task-rate allowance; **medium** as a planning analogue | Add setting-out, supervision, handling, and other tasks separately. Soil, tools, haul distance, and work conditions matter; this is not a measured ancient-world average. [International Labour Organization](https://www.ilo.org/sites/default/files/wcmsp5/groups/public/%40asia/%40ro-bangkok/%40ilo-jakarta/documents/publication/wcms_470657.pdf) |
| Gravel loss, Scenic Rim, Queensland study, **2020** | Annualized medians approximately **48.7 mm/year** for existing material and **7.9 mm/year** for modified material | Field comparison; **medium** for these sites | Some observations were extrapolated to a year. This is motor-road evidence, not a universal road-decay range or a randomized estimate of a sixfold material benefit. [Frontiers](https://www.frontiersin.org/journals/built-environment/articles/10.3389/fbuil.2020.00003/full) |
| Borehole-and-handpump capital cost, IRC/WASHCost **2012 benchmarks** | **US$20–61 per person**, in **2011 dollars** | Multi-country program benchmarks; **medium** | Modern installed-service costs, not the price of an ancient hand-dug well. [IRC Wash](https://www.ircwash.org/news/wash-numbers-latest-cost-benchmarks-economic-returns-and-handwashing-0) |
| Recurring borehole-and-handpump expenditure, same benchmarks | **US$3–6 per person/year**, in **2011 dollars** | Lifecycle expenditure benchmark; **medium** | Includes more than routine repairs: operation, capital maintenance, and support. Do not treat it as a physical depreciation percentage. [IRC Wash](https://www.ircwash.org/news/wash-numbers-latest-cost-benchmarks-economic-returns-and-handwashing-0) |
| Marcius’s Roman waterworks appropriation, reported by Frontinus | **180 million sesterces** | Ancient reported appropriation; **medium** as a textual cost record, low for comparative calibration | Covered repairs to older channels as well as the new supply. It is neither a clean Aqua Marcia unit cost nor verified final expenditure. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Frontinus/De_Aquis/Bennett/1%2A.html) |
| Nicomedia’s first abandoned aqueduct attempt, Pliny | Approximately **3.3 million sesterces** reported spent | Contemporary correspondence; **medium** | Illustrates costly noncompletion. Numerical readings/translations vary; do not build precise cross-project cost ratios from it. [ToposText](https://topostext.org/work/198) |

**There is no defensible universal “public infrastructure loses X% condition each year” parameter in this evidence.** Material, drainage, foundation, loading, local weather, construction quality, and intervention history must determine deterioration. Even modern road models require local calibration. [Taylor & Francis Online](https://www.tandfonline.com/doi/full/10.1080/10298436.2011.606320)

### 2.2 A construction-cost model suitable for an agrarian economy

Use bills of quantities before money:

\[
L\_{\text{construction}}
=
\sum\_k \frac{Q\_k}{p\_k},
\]

where \(Q\_k\) is the quantity of task \(k\), and \(p\_k\) is its productivity in quantity per person-day.

Keep excavation, hauling, timber preparation, stone dressing, masonry, surveying, and supervision as separate tasks. Convert them into monetary expenditure only when an institution actually purchases those inputs. An in-kind economy can organize the same work through food allocations, labor duties, material contributions, and reciprocal obligations.

**Illustrative ditch project—not a historical observation.**

Assume a ditch 1,000 m long requires removal of an average 0.5 m² cross-section:

\[
V = 1{,}000 \times 0.5 = 500\ \mathrm{m^3}.
\]

Using the ILO ordinary-soil allowance only as a starting analogue gives approximately **333–500 unskilled person-days**. This excludes separate surveying, supervision, difficult excavation, lining, long-distance spoil disposal, and interruptions. Twenty effectively deployed excavators would therefore require roughly **17–25 working days for that task alone**.

Suppose the simulation’s sediment model subsequently deposits **50 m³/year**. That amount is an **assumption**, not a sourced historical siltation rate. At the same provisional productivity, removing it requires another **33–50 person-days/year**.

The essential relationship is inspectable: maintenance demand emerges from material accumulation and task productivity. It is not an arbitrary percentage of the original cash price.

### 2.3 Derive replacement cycles from physical state

For a gravel surface:

\[
T\_{\text{resurface}}
\approx
\frac{h\_{\text{current}}-h\_{\text{minimum}}}
{\text{expected annual thickness loss}}.
\]

An **illustrative** 50 mm usable thickness reserve at 8 mm/year loss gives about 6.25 years before the threshold. That calculation is not a prediction for ancient roads: it shows how a replacement cycle should follow from thickness and locally calibrated wear.

For a canal, calculate when sediment accumulation reduces useful capacity below demand. For an earthen wall, schedule protective surface repairs separately from structural reconstruction. For a well, distinguish the lifting apparatus, lining, drainage apron, and water source. “Repairing the well” should not restore groundwater that is absent.

---

## 3. Variation across eras and world regions

These are institutional possibilities, **not a required progression from communal work to centralized government to private enterprise**.

| Setting | What the evidence shows | Implication for TCE |
| --- | --- | --- |
| **Foraging and non-cereal food-producing societies: southeastern Australia** | Budj Bim’s Gunditjmara aquaculture landscape includes channels, weirs, and dams associated with eel management over at least **6,600 years**. This is evidence of long-lived engineering knowledge and landscape management, not untouched structures lasting that long. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1577) | Do not require cereal agriculture, cities, metal tools, or a tax bureaucracy before collective infrastructure becomes possible. |
| **Early farming: central Europe** | Neolithic wooden wells demonstrate sophisticated carpentry in farming settlements around the sixth millennium BCE. Construction evidence is much firmer than reconstruction of who commanded, owned, or financed every well. [University of Freiburg Public Relations](https://www.kommunikation.uni-freiburg.de/pm-en/press-releases-2012/pm.2012-12-19.366-en) | Small projects can depend on specialist knowledge without a large state. Leave ownership and organization variable rather than inferring monarchy from engineering complexity. |
| **Pre-industrial Mediterranean** | Athenian assigned elite services and Roman administrative waterworks represent different relationships between wealth, civic obligation, authority, and specialist labor. Municipal works could also be unfinished and investigated for waste. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/athenian-trierarchy-mechanism-design-for-the-private-provision-of-public-goods/831A8086B9E8EA9309DD2332D16D777A) | Patronage, taxation, compulsory services, and public employment should coexist rather than replace one another wholesale. |
| **East Asia: Dujiangyan** | Annual repair became a durable institution involving planned timing, labor organization, and funding arrangements. [Xhu Academy](https://aj.xhu.edu.cn/xhdxxbzskb/en/article/doi/10.19642/j.issn.1672-8505.2018.04.002) | Technical durability includes a recurring administrative calendar. A ruler can inherit an asset and its maintenance obligations rather than merely its construction achievement. |
| **South Asia: Nepal and southern India** | User-managed irrigation can perform well, but “the village community” is not necessarily homogeneous, autonomous, or harmonious. Mosse particularly challenges simplified accounts of traditional collective tank management. [Sage Journals](https://journals.sagepub.com/doi/10.3233/HSM-1994-13305) | Represent unequal landholding, water position, political authority, and contribution rights within communities. Avoid a universal communal-cooperation bonus. |
| **Islamic societies, including Ottoman North Africa** | Waqf linked services to endowed property. Research on Ottoman Algiers documents adaptations through perpetual leases and property exchanges rather than absolute institutional immobility. [Sites@Duke Express](https://sites.duke.edu/timurkuran/files/2016/10/waqf-2001-1.original.pdf) | Endowments can endure political succession but may face rigid purposes, weak income, or administrative conflict. Flexibility should depend on rules and adjudication, not a fixed cultural penalty. |
| **Andes** | The Qhapaq Ñan combined an imperial network with local knowledge and custodianship; Q’eswachaka shows recurring community renewal and specialist leadership at bridge scale. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1459) | Central coordination and local maintenance can be complementary. An empire-wide road system need not employ a centrally paid crew for every segment. |
| **West Africa: earthen civic and religious buildings** | Djenné demonstrates recurring communal surface protection alongside the need for occasional structural consolidation and reconstruction. [UNESCO World Heritage Centre](https://whc.unesco.org/en/news/574) | Surface upkeep, social ritual, and specialist structural work are distinct but connected activities. |
| **Industrializing societies** | English and Welsh turnpikes illustrate investment financed through toll rights and debt, with landowners and merchants involved in administration. The financial structure differed from both ordinary parish obligations and a modern tax-funded highway agency. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/did-turnpike-trusts-increase-transportation-investment-in-eighteenthcentury-england/B22083C49AA94D6102E90AEF86DCF355) | Introduce borrowing and increasingly specialized management when supporting institutions exist—not at a fixed historical date. |
| **Modern systems** | Contemporary arrangements include public inspection, private concessions, community-operated water points, and external technical support. More elaborate equipment does not eliminate unreliable service. [Federal Highway Administration](https://www.fhwa.dot.gov/ipd/p3/toolkit/publications/model_contract_guides/core_toll_concession_0814/) | Technology changes the required inputs and failure modes. It does not automatically solve financing, accountability, or repair logistics. |

---

## 4. Stylized facts a correct simulation should reproduce

### 4.1 A durable system can contain short-lived components

Q’eswachaka’s annual renewal is the clearest example. Institutional continuity and component longevity are different variables. A simulation that replaces material repeatedly while preserving location, rights, skills, and social meaning can reproduce this; one that assigns the bridge a single construction date and lifespan cannot. [UNESCO ICH](https://ich.unesco.org/en/RL/knowledge-skills-and-rituals-related-to-the-annual-renewal-of-the-q-eswachaka-bridge-00594)

### 4.2 Better organization can outweigh more sophisticated infrastructure

The Nepal irrigation findings support outcomes in which a well-organized user association outperforms a technically superior but poorly operated system. This should be possible, not guaranteed: neither “state” nor “community” should receive an unconditional efficiency advantage. [Sage Journals](https://journals.sagepub.com/doi/10.3233/HSM-1994-13305)

### 4.3 Routine care does not substitute for major renewal

UNESCO reported structural weakness at Djenné despite annual replastering; exceptional rain in November 2009 caused part of a tower to collapse during a restoration campaign. TCE should therefore permit an asset to receive regular surface maintenance while accumulating a structural problem that requires different skills and resources. [UNESCO World Heritage Centre](https://whc.unesco.org/en/news/574)

### 4.4 Construction expenditure can produce little or no service

Nicomedia’s abandoned aqueduct attempts show that large expenditure need not create operational infrastructure. Projects need completion thresholds, engineering competence, and opportunities for redesign, salvage, and cancellation—not simply a progress bar determined by money spent. [ToposText](https://topostext.org/work/198)

### 4.5 Snapshot functionality is not reliability

A 2025 study examined **1,682 handpumps in ten sub-Saharan African countries**. About **85%** operated at the survey visit, but operation, breakdown history, downtime, pumping effort, and output were distinct indicators. This was a particular cross-sectional dataset, not a universal national failure rate. [PLOS](https://journals.plos.org/water/article?id=10.1371%2Fjournal.pwat.0000271)

TCE should therefore record delivered service and outage duration, not just “working/broken.”

### 4.6 Accountability can change delivered quality

The Indonesian audit experiment’s approximately **8-percentage-point reduction in missing expenditure** provides a strong causal example of oversight affecting project delivery. Use it as a validation case for incentives and monitoring, not as a fixed audit bonus applied everywhere. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/517935)

### 4.7 Networks can fail nonlinearly

Penny and colleagues’ Angkor model shows how flow changes, erosion, sedimentation, and network structure can create cascading damage. It does **not** establish that Angkor declined simply because maintenance stopped. For TCE, the relevant target is that damage to a critical connection can impose much larger consequences than damage to an isolated segment. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6192684/)

---

## 5. Recommended representation for TCE

### 5.1 Keep the institutional model richer than the engineering model

For 10k–50k agents, the most valuable detail is usually **who notices, decides, contributes, refuses, supervises, and repairs**. Engineering can initially use a small number of physically meaningful states.

| Entity | Minimum useful state |
| --- | --- |
| **Asset** | Location/network links, purpose, beneficiaries, access rights, owner, operator, maintainer, design capacity, component list |
| **Component** | Material and quantity, structural condition, obstruction or wear stock, exposure, construction quality, repair recipe |
| **Institution** | Members and authority, decision rule, revenue rights, resource inventories, obligations, reserves, debts, contracts |
| **Household/person** | Eligibility for obligations, beneficiary status, skills, seasonal availability, property interests, trust, compliance history |
| **Work order** | Task, target component, quantities, deadline/season, crew, tools, materials, transport, acceptance criteria |
| **Information record** | Last inspection, reported defects, estimated condition, uncertainty, actual condition accessible only to the simulation |

An institution should be assembled from rules. For example, a canal association might have household voting, acreage-based labor contributions, elected supervisors, and water sanctions. A temple foundation might own rental property, appoint an administrator, and fund the same physical maintenance recipes.

This avoids authoring a separate maintenance engine for every governmental or religious form.

### 5.2 Use three physical processes

**Obstruction and accumulated material**

\[
S\_{t+\Delta t}
=
\max\left[
0,\;
S\_t+
\left(D\_t-E\_t-R\_t\right)\Delta t
\right],
\]

where \(S\) is sediment or debris volume, \(D\) deposition, \(E\) natural removal, and \(R\) removal by maintenance.

**Gradual deterioration**

\[
q\_{t+\Delta t}
=
\operatorname{clip}
\left[
q\_t-d\_t\Delta t+r\_t,\;0,\;1
\right],
\]

where \(q\) is component condition, \(d\_t\) depends on material, environment, loading, and protection, and \(r\_t\) is the effect of completed repair.

**Conditional failure**

\[
P(\text{failure}\mid
\text{load},q,\text{foundation},\text{design},\text{exposure}).
\]

Do not apply an identical independent annual failure chance to every asset. A levee’s principal danger is conditional on hydraulic loading and its vulnerabilities. The USACE risk framing similarly distinguishes loading, performance, and consequences. [NWO](https://www.nwo.usace.army.mil/Media/Fact-Sheets/Fact-Sheet-Article-View/Article/487691/levee-safety-action-classification/)

For a first implementation, use simple monotone response curves rather than full structural mechanics. Preserve the causal inputs so later calibration does not require redesigning the social system.

### 5.3 Convert physical changes into actual service consequences

A proposed v1 mapping is:

| Asset | Routine tasks | Service loss before total collapse |
| --- | --- | --- |
| **Road** | Clear drains and vegetation; fill holes; restore surface and crossings | Lower travel speed, higher hauling effort, seasonal impassability |
| **Well** | Repair lifting equipment and lining; clear access and drainage defects | Longer collection time, reduced delivery, intermittent outage; water quality tracked separately |
| **Wall or gate** | Repair protective surfaces, joints, drainage, doors, and timberwork | Reduced resistance at weak points; impaired access control |
| **Levee** | Inspect and repair erosion, low spots, penetrations, and associated drainage works | Increasing conditional flood risk, not necessarily a visible daily service penalty |
| **Canal** | Remove sediment and vegetation; repair banks, control structures, and lining | Lower capacity, altered allocation, tail-end shortages |

These are **recommended abstractions**, not claims that all historic maintainers used the same task classifications.

Never let an unsuitable repair cure a different problem. Dredging cannot repair a breached bank; resurfacing cannot fix an unstable foundation; repairing a pump cannot make a poorly yielding aquifer productive. Physical investigations of African handpump boreholes identify hydrogeology and component condition as distinct contributors to functionality. [PubMed](https://pubmed.ncbi.nlm.nih.gov/36041625/)

### 5.4 Let maintenance backlogs and institutional failure emerge

Maintain a queue of required work, measured in task quantities and person-days:

\[
\text{backlog}\_{t+1}
=
\max(0,\text{backlog}\_t+\text{new work}\_t-\text{completed work}\_t).
\]

The queue should prioritize according to institutional behavior, not a universal optimal scheduler. One institution may protect critical bottlenecks; another may respond only to complaints; another may favor a patron’s district.

A plausible endogenous deterioration sequence is:

> Lost revenue → postponed work → poorer service → reduced production or use → weaker revenue → larger backlog.

This is a **simulation mechanism to test**, not an inevitable historical trajectory. Institutions must also be able to recover through extraordinary contributions, renegotiated rights, outside aid, cheaper repairs, changed service standards, or replacing failed leadership.

Abandonment should sometimes be rational. An obsolete bypassed road or a settlement’s unused wall need not absorb maintenance forever. Demolition and material reuse should compete with preservation.

### 5.5 Make accounts resource-consistent

Keep separate balances for money, grain, materials, and other goods. Labor obligations are dated claims on time, not perpetual labor inventory.

When corruption occurs, model the destination: resources may become private wealth, be diverted to another building, or be wasted through poor execution. Do not simply delete a universal corruption percentage from the economy.

Keep fiscal and economic accounting separate. Wages are an institutional expense and household income; foregone farming is an opportunity cost. Adding both indiscriminately to the same “social cost” measure double-counts part of the burden.

### 5.6 Initial calibration and sensitivity settings

Where historical estimates are weak, expose assumptions rather than disguising them as facts.

| TCE setting | Suggested initial experiment | Status |
| --- | --- | --- |
| Seasonal inspection | One scheduled inspection per major maintenance season, plus inspections after damaging events | **Design assumption**, informed by recurring historical maintenance and modern inspection practice |
| Deterioration uncertainty | Test **0.5×, 1×, and 2×** each locally selected baseline | **Sensitivity range**, not an empirical global distribution |
| Missed upkeep | Compare **0, 1, and 3 missed maintenance cycles** | **Stress test** |
| Repair reserve | Compare no reserve with **1 and 3 years** of expected routine resource requirements | **Policy experiment**, not a historical norm |
| Labor compliance | Test **50%, 75%, and 100%** attendance before endogenous enforcement and legitimacy effects | **Behavioral sensitivity**, not estimated corvée compliance |
| Information | Compare timely inspection with condition information delayed by **one maintenance cycle** | **Institutional experiment** |

The important outputs are service delivered, household burden, distribution of benefits, unfinished projects, repair backlogs, outage durations, and institutional survival—not just average asset condition.

### 5.7 Performance and fast-forward

Use daily scheduling for visible crews and resource deliveries, but update slow deterioration on weekly, monthly, or seasonal events. Floods, collapses, and critical outages remain immediate events.

Store roads and canals as maintenance segments rather than one institution-facing asset per rendered tile. Cache beneficiary groups and network dependencies. Recompute them when routes, water allocation, ownership, or settlement patterns change—not for every citizen every day.

In fast-forward, aggregate crew output and contributions while preserving the same resource accounting, seasonal constraints, project completion thresholds, and shock events. Avoid replacing all of this with “subtract annual upkeep”: doing so would remove the very mechanisms that make institutional failure emerge.

These are architectural recommendations, **not benchmarked performance claims**.

### 5.8 Existing models and games worth borrowing from

| Reference | Useful element | What not to assume it supplies |
| --- | --- | --- |
| **Ostrom, Lam, and Lee’s Nepal irrigation research** | Joint representation of users, operating rules, contributions, and institutional performance | A universal numerical formula for cooperation. [Sage Journals](https://journals.sagepub.com/doi/10.3233/HSM-1994-13305) |
| **Lansing and Kremer’s Bali water-temple model** | Coupling decentralized coordination to ecological and agricultural consequences | A complete model of construction finance or maintenance accounting. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/abs/10.1525/aa.1993.95.1.02a00050) |
| **Penny et al.’s Angkor network model** | Erosion–sedimentation feedback and network vulnerability | Full hydraulic or political simulation; the published model explicitly simplifies processes and fixes network topology. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6192684/) |
| **World Bank RONET and HDM-4 tools** | Comparing maintenance strategies, condition, budgets, user costs, and revenue requirements | Direct applicability of modern motor-vehicle relationships to pedestrians, carts, or pack animals. [World Bank Data Catalog](https://datacatalog.worldbank.org/search/dataset/0065669/road-network-evaluations-tools-ronet) |
| **Songs of Syx** | A useful game abstraction: maintenance requires janitor workers and material supply | Endogenous public finance, contested obligations, or a validated historical deterioration model. The referenced wiki documents the mechanic, not current balance values. [Songs of Syx](https://www.songsofsyx.com/wiki/index.php/Janitor) |

---

## 6. Sources, datasets, and remaining uncertainty

### Most useful evidence for implementation

| Source | Best use |
| --- | --- |
| **Frontinus, *De aquaeductu*, especially §§116–126; Pliny, *Letters* 10.37–38** | Administrative roles, dedicated labor, diversion of resources, supervision, project noncompletion, and reported ancient expenditure. These are contemporary accounts, but not neutral modern audits. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Frontinus/De_Aquis/Bennett/2%2A.html) |
| **Ostrom, Lam & Lee (1994), “The Performance of Self-Governing Irrigation Systems in Nepal”; Mosse (1998), “Making and Misconceiving Community in South Indian Tank Irrigation”** | Institutional mechanisms and safeguards against idealizing either centralized or community management. [Sage Journals](https://journals.sagepub.com/doi/10.3233/HSM-1994-13305) |
| **Kaiser (2007), “The Athenian Trierarchy”; Bogart (2005), “Did Turnpike Trusts Increase Transportation Investment in Eighteenth-Century England?”** | Assigned elite public obligations, toll authority, borrowing, and investment incentives. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/athenian-trierarchy-mechanism-design-for-the-private-provision-of-public-goods/831A8086B9E8EA9309DD2332D16D777A) |
| **Kuran (2001), “The Provision of Public Goods under Islamic Law”; Hoexter (1997), “Adaptation to Changing Circumstances…”** | Endowment commitment and constraints, read alongside documented legal and administrative adaptation in Ottoman Algiers. [Sites@Duke Express](https://sites.duke.edu/timurkuran/files/2016/10/waqf-2001-1.original.pdf) |
| **Olken (2007), “Monitoring Corruption: Evidence from a Field Experiment in Indonesia”** | A causal calibration case for oversight; the journal provides linked replication materials. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/517935) |
| **McManus et al. (2025), “Alternative Indicators of Handpump Functionality”** | Service reliability indicators and an explicitly linked dataset, **DOI: 10.15139/S3/J3IVUG**. [PLOS](https://journals.plos.org/water/article?id=10.1371%2Fjournal.pwat.0000271) |
| **ILO labor-based road manuals; IRC/WASHCost benchmarks** | Task-based costing and separation of initial expenditure, recurring operation, renewal, and support. [International Labour Organization](https://www.ilo.org/sites/default/files/wcmsp5/groups/public/%40asia/%40ro-bangkok/%40ilo-jakarta/documents/publication/wcms_470657.pdf) |
| **World Bank RONET / Road User Charges tools** | Inspectable budget and maintenance scenario models. These are modeling tools, not historical observational datasets. [World Bank Data Catalog](https://datacatalog.worldbank.org/search/dataset/0065669/road-network-evaluations-tools-ronet) |

### Claims to treat cautiously

**Construction evidence does not uniquely identify government.** A sophisticated early well, canal, or landscape cannot by itself establish centralized coercion. Budj Bim and user-managed irrigation are particularly important checks against making large-scale coordination synonymous with a state. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1577)

**“Traditional community” is not a sufficient explanation.** Rules may embody unequal rights, contested obligations, and interventions by outside authorities. Mosse’s work is a direct warning against treating an idealized village collective as a timeless historical unit. [Digital Library of the Commons](https://dlc.dlib.indiana.edu/dlc/items/8614a15a-ba4a-4289-81b2-1b4a8e23922a)

**Waqf rigidity is a debated explanatory claim, not a universal outcome.** Kuran emphasizes constraints on adaptation; Hoexter documents mechanisms that adjusted endowed property to changing circumstances. TCE should make the relevant legal and administrative flexibility explicit. [Sites@Duke Express](https://sites.duke.edu/timurkuran/files/2016/10/waqf-2001-1.original.pdf)

**Maintenance failure is not a sufficient explanation for every collapse.** Extreme loading, poor siting, design limitations, warfare, and institutional disruption need separate causal paths. The Angkor analysis is evidence for a particular network vulnerability, not proof of a single cause of urban abandonment. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6192684/)

**Cross-era prices and universal decay rates remain the weakest calibration area.** For an agrarian start, prioritize quantities, person-days, transport, seasonality, and resource obligations. Use money as the settlement’s accounting system develops, rather than converting a modern borehole price or an ancient monumental appropriation directly into a universal construction cost.

**Bottom line:** TCE’s most convincing public works will emerge when people must repeatedly organize real work. Long-lived infrastructure should be an achievement of continued repair, financing, knowledge, and cooperation—not an intrinsic property awarded when construction finishes.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92944-4598-83ea-bcce-12fe424ea6d4)
