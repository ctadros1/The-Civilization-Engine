# Urban governance for The Civilization Engine

## Central finding

**Model urban government as a negotiated distribution of powers—not as a progression from village elder to mayor to democratic city council.**

Three dimensions must remain separate: **autonomy from outside rulers; participation within the town; and capacity to deliver services**. A town can possess substantial privileges while excluding most inhabitants from government. Conversely, associations can organize important urban activities under an imperial administration without constituting an independent municipality. Research on European participative institutions, Chinese guilds, and modern local autonomy supports treating these as distinct institutional features. [WU Wirtschaftsuniversität Wien](https://research.wu.ac.at/de/publications/participative-political-institutions-in-pre-modern-europe-introdu/)

For TCE, the essential distinction is therefore:

> **Who may decide, who actually influences the decision, who pays, who performs the work, and who can overturn the arrangement?**

The mechanisms below are proposed implementations informed by the evidence. Numerical historical observations are separated from unestimated simulation settings.

---

## 1. Mechanisms: how urban government emerges and operates

### 1.1 Organize around problems before creating a municipality

Urban institutions need not originate in a comprehensive constitution. Wickham’s study of Italian communes emphasizes contingent institutional development amid the breakdown of existing authority. Chinese evidence likewise shows associations forming through mutual assistance, fundraising, meeting places, rules, and eventual official recognition. [OUP Academic](https://academic.oup.com/princeton-scholarship-online/book/15824)

**Implementable rule:** when a recurring problem affects an identifiable group, members may organize a collective response.

For an individual \(i\), participation can depend on:

\[
\Delta U\_i =
E[\text{benefit from cooperation}]
-\text{contribution}
-\text{coordination cost}
-\text{political risk}.
\]

Benefits and costs should be individual-specific. Merchants might value safer market access; adjoining households might value drainage; landowners might support a wall but resist a tax on property.

An association becomes durable when it acquires some combination of recurring contributions, common assets, recognized procedures, membership sanctions, and people willing to administer it. A temporary works committee can subsequently become a ward body or council. It can also dissolve when its task ends.

**Do not make written records a prerequisite for collective decisions.** In the proposed model, records instead improve institutional memory, accountability, and the ability to preserve an agreement beyond its original participants.

### 1.2 Charters exchange specified rights for specified obligations

A charter is not equivalent to democracy or independence. In medieval England, *farm grants* transferred aspects of fiscal and judicial administration to borough communities while preserving payments to the crown. Angelucci, Meraglia, and Voigtländer connect these arrangements to the advantages of local administration and later representation over extraordinary taxation. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20200885)

**Implement charters as bundles of clauses**, each with its own jurisdiction and enforcement arrangements. Possible clauses include authority to select officials, collect named revenues, operate courts, regulate markets, hold corporate property, or undertake defensive works. Obligations may include annual remittances, military assistance, hospitality, extraordinary contributions, or an initial payment.

A particularly concrete example is John’s **1199 London charter**: citizens could select and remove their sheriffs while owing an annual **£300 farm for the sheriffwicks of London and Middlesex**. The citizens were liable if the sheriffs failed to satisfy the obligation. This was neither the entire city budget nor a universal municipal tax rate. [Semantic Scholar](https://pdfs.semanticscholar.org/0bd8/8975943e3ce74a7acdf543b530eb132a7f9f.pdf)

For TCE, separate:

* the right to set a tax from the right to collect it;
* local retention of receipts from remittance obligations;
* ordinary obligations from extraordinary wartime demands.

A town may control collection without controlling rates, or select officers whose decisions remain appealable to an outside court.

### 1.3 Town–ruler bargaining: implement Tilly conditionally

Tilly’s distinction between coercion-intensive, capital-intensive, and capitalized-coercion settings concerns the resources available to rulers and their relationships with those controlling them. Where rulers needed resources held by organized capitalists, bargaining became important. These were historically situated configurations, not three government types that leaders simply selected. [UC San Diego Pages](https://pages.ucsd.edu/~bslantchev/courses/ps240/06%20Domestic%20Organizations%20and%20International%20Behavior/Tilly%20-%20Coercion%2C%20capital%20and%20European%20states%20%5BCh%201%2C3%2C6%5D.pdf)

**Proposed bargaining rule:** a ruler delegates when the expected return from agreement exceeds the return from direct administration, coercive extraction, or an alternative intermediary.

A minimal fiscal comparison is:

\[
F-M \geq T\_D-C\_D-L\_D
\]

where all terms refer to the same period:

| Term | Meaning |
| --- | --- |
| \(F\) | Expected remittance under the proposed agreement |
| \(M\) | Cost of supervising that agreement |
| \(T\_D\) | Expected collections under direct administration |
| \(C\_D\) | Direct administrative and collection costs |
| \(L\_D\) | Expected losses from resistance, evasion, flight, disruption, and enforcement |

The town’s participating coalition must separately prefer the agreement. Do not calculate only aggregate “town welfare”: wealthy creditors, artisans, landowners, and poorer residents may face different outcomes.

**Conditions favoring concessions in this model:** difficult-to-monitor commerce; expensive direct administration; credible collective withholding; useful town militias; competition among potential protectors; or urgent demand for a loan.

**Conditions favoring coercion or tighter control:** accessible alternative revenues; a reliable garrison; divided local interests; appropriable assets; or elites willing to administer extraction for the ruler.

Thus, war should not automatically generate municipal liberty. Depending on these conditions, it can produce concessions, indebtedness, occupation, or destruction.

### 1.4 Make agreements self-enforcing, not magically binding

Greif’s analysis of late-medieval Genoa treats political arrangements as systems whose participants must have incentives to sustain them. That is a better starting point than assuming that a charter permanently changes an autonomy statistic. [JSTOR](https://www.jstor.org/stable/j.ctv131bwhx)

**Implementation:** each powerful participant periodically compares compliance with feasible deviations. A ruler’s temptation to revoke privileges can rise after acquiring troops or alternative financing. A town’s incentive to withhold payment can rise when protection fails or the ruler becomes vulnerable.

Credibility should depend on concrete mechanisms: adjudication, recognized custom, archives, influential guarantors, organized resistance, and the consequences of damaging future cooperation.

Treat succession, default, military defeat, and disputed appointments as opportunities to renegotiate—not automatic triggers of revocation. Existing agreements should provide a focal point, but actors may disagree about their interpretation.

### 1.5 External freedom does not determine internal inclusion

Wahl’s historical database separately codes **guild participation in councils, participative election procedures, and institutionalized burgher representation**. These are not interchangeable measures. Chinese associations likewise could possess autonomy while maintaining restrictive membership and leadership arrangements. [WU Wirtschaftsuniversität Wien](https://research.wu.ac.at/de/publications/participative-political-institutions-in-pre-modern-europe-introdu/)

For every governing body, specify:

| Institutional question | TCE representation |
| --- | --- |
| Who belongs to the political community? | Citizenship or corporate-membership rules |
| Who can vote? | Eligibility predicates, separate from residency |
| Who can hold office? | Age, status, membership, wealth, service, or other authored requirements |
| Who nominates candidates? | Assembly, guild, ward, incumbent council, ruler, or mixed procedure |
| How are positions filled? | Election, lot, co-option, appointment, inheritance, or combinations |
| Who can dismiss or review officeholders? | Defined body and procedure, plus effective enforcement capacity |

Track **residents, citizens, voters, eligible candidates, and officeholders as different populations**.

A guild constitution should not confer representation on everyone employed in a trade. Membership rules determine whether masters, journeymen, apprentices, migrants, or other workers participate.

### 1.6 Guilds can coordinate production and exclude competitors

The economic interpretation of European guilds remains disputed. S. R. Epstein emphasizes apprenticeship, transferable skills, and technical diffusion. Ogilvie’s broader analysis emphasizes exclusion, privilege, political bargaining, and restrictions on competition. Neither interpretation justifies a universal “guild efficiency bonus.” [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/craft-guilds-apprenticeship-and-technological-change-in-preindustrial-europe/4B18A7808BACBFA40D76475FBCB665E0)

**Implement the constituent practices separately:** apprenticeship contracts, certification, mutual insurance, market information, common purchasing, entry fees, exclusive trading rights, and restrictions on production or employment.

Political consequences then emerge from those practices. A guild that collects dues and maintains reliable membership records can mobilize efficiently. The same organization may seek council seats to protect members, improve infrastructure, or exclude outsiders.

Changing the balance of those activities should change outcomes without changing an institution’s name.

### 1.7 Municipal offices should specialize with workload

Ancient Athens already distinguished officials responsible for roads, markets, weights and measures, grain supplies, contracts, and audits. Nineteenth-century English urban administration, meanwhile, often involved separate bodies for municipal affairs, poor relief, and public health. Neither case resembles a timeless, unified mayoral department. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.3.3.html)

**Recommended functional modules—not universal historical job titles:**

| Function | Responsibilities to simulate | Necessary resources or information |
| --- | --- | --- |
| Executive and deliberation | Convene meetings, set agendas, negotiate agreements, execute decisions | Meeting time, communication, procedural authority |
| Treasury and records | Assess liabilities, collect receipts, maintain accounts, pay obligations | Clerks, records, secure storage, payment access |
| Courts and dispute resolution | Hear disputes, validate transactions, impose or review sanctions | Adjudicators, witnesses, evidence, enforcement |
| Watch and defense | Patrol, guard gates, respond to disorder, organize defense | Personnel, schedules, equipment, command |
| Market supervision | Inspect measures and goods, manage stalls, adjudicate market disputes | Inspectors, standards, market access |
| Works and environmental management | Maintain roads, drains, water facilities, walls, and public spaces | Labor, materials, technical knowledge, maintenance finance |
| Relief and communal provision | Distribute food or support, administer endowed facilities | Revenue, stores, beneficiary rules, administrators |

Create an additional office or employee when a recurring task exceeds existing capacity and a coalition will fund it. Allow unpaid, rotating, fee-supported, part-time, and salaried arrangements.

Separately track **officeholders and workers**. Ten inspectors do not imply ten people perform every associated cleaning, carrying, guarding, or construction task.

### 1.8 Services require real finance and real production

Waqf institutions demonstrate that urban services could be funded through endowed property rather than a general municipal treasury. Kuran documents provision including water, education, food distribution, and other facilities, while also discussing constraints imposed by endowment arrangements. Actual flexibility varied; founder restrictions should therefore be modeled as legal rules, not an immutable cultural characteristic. [Sites@Duke Express](https://sites.duke.edu/timurkuran/files/2016/10/waqf-2001-1.original.pdf)

For each service, distinguish **financier, governing body, operator, and beneficiaries**. They need not be the same institution.

Use a conserved municipal ledger:

\[
\begin{aligned}
\Delta \text{cash}={}&
\text{taxes}+\text{fees}+\text{rents}+\text{transfers}
+\text{donations}+\text{borrowing}\\
&-\text{wages}-\text{operations}-\text{maintenance}
-\text{construction}-\text{remittances}
-\text{interest}-\text{principal}.
\end{aligned}
\]

Borrowing creates a corresponding liability. Transfers move resources between institutions; they do not create output. In-kind dues and labor obligations should use separate physical ledgers.

A useful proposed production rule is:

\[
\text{service output}
=\min(\text{facility capacity},\text{staffed capacity},\text{input capacity})
\times \text{operational reliability}.
\]

Consequently, a fountain without maintenance, a watch without personnel, or a granary without grain delivers little regardless of its nominal institutional status.

---

## 2. Parameters: historical anchors and proposed tuning ranges

### 2.1 Evidence-based calibration anchors

**Confidence refers to the particular observation, not its applicability everywhere.** High confidence in a recorded rule does not establish perfect compliance with that rule.

| Quantity | Observed value, units, and context | Appropriate simulation use | Confidence and source |
| --- | --- | --- | --- |
| English boroughs receiving farm grants | **51.0% of royal boroughs; 3.9% of mesne boroughs**, in the authors’ medieval English analysis; sample **554 boroughs**, including 145 royal and 409 mesne | Test whether ownership and delegation incentives produce substantial differences. **Not annual grant probabilities.** | High for reported sample; medium for generalization. Explicitly the **March 2020 research slides**, preceding the 2022 article. [UCLA Anderson School of Management](https://www.anderson.ucla.edu/faculty_pages/nico.v/Research/AMV_Slides.pdf) |
| Fixed charter remittance | **£300/year**, London and Middlesex sheriffwicks, charter of **1199** | Demonstrates a fixed remittance coupled with local officer selection and collective liability | High for charter provision; no defensible universal conversion to a tax/output ratio. [Semantic Scholar](https://pdfs.semanticscholar.org/0bd8/8975943e3ce74a7acdf543b530eb132a7f9f.pdf) |
| Large council and rotating executive | Athens: **500 councillors**, **50 from each tribe**; the presiding tribal group served **35–36 days** | Separate full council membership from the smaller body handling current business | High for the constitutional description; implementation in practice may differ. *Athenian Constitution*, §43. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.2.2.html) |
| Extremely short presidency | Athens: presiding officer served **one night and day**, without repeating that office | Support very short ceremonial or procedural offices without replacing the whole administration | High for described rule. §44. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.2.2.html) |
| Specialized boards | Athens: **5 road commissioners**; boards of **10** for several inspection functions; grain commissioners expanded from **10 to 35** | Administrative specialization and changes in staffing; **not officials-per-capita estimates** | High for textual report. §§50–54. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.3.3.html) |
| Reconstructed governing council | Tlaxcallan: approximately **50–200 members** | A plausible case-specific large collective governing body, rather than a universal council-size range | Medium–low: reconstruction using ethnohistorical and archaeological evidence. [Frontiers](https://www.frontiersin.org/journals/political-science/articles/10.3389/fpos.2022.832440/full) |
| Federation of associations | All-Hankou Guild Confederation: **more than 100 guilds** | Institutions can federate through organizational representation; member organizations are not individual councillors | Medium: historical reconstruction summarized by Moll-Murata. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/chinese-guilds-from-the-seventeenth-to-the-twentieth-centuries-an-overview/61A96EB7FF67CE0BE8073C36DDB049CD) |
| Collective civic fundraising | Sakai, **1535**: **114 people**, each contributing **1 kanmon**, for an enclosure-wall repair associated with the meeting complex | Project-specific subscriptions by named contributors; **not a 114-seat council** | High for the document as reported by Sakai’s archaeological presentation. [Sakai City](https://www.city.sakai.lg.jp/kanko/rekishi/bunkazai/bunkazai/tenjikai/chanoyu/kaisho.html) |
| Modern municipal revenue composition | RBI sample of **201 Indian municipal corporations**, actual **2017–18** accounts: own revenue **64.5%** of revenue receipts; reported central/state transfers **34.8%** | An example of substantial own revenue coexisting with intergovernmental dependence | High for reported accounts; coverage caveat: transfer reporting was incomplete for some corporations. [City Finance](https://www.cityfinance.in/assets/images/homepage/spotlight/rbi-report-on-municipal-finances.pdf) |
| Modern expenditure composition | Same sample/year: approximately **70% recurrent expenditure; 30% capital expenditure** | Separate operation and upkeep from construction; do not treat a building boom as sustained service capacity | High for this sample, not a modern-world norm. [City Finance](https://www.cityfinance.in/assets/images/homepage/spotlight/rbi-report-on-municipal-finances.pdf) |

The strongest numerical evidence concerns **specific rules, organizations, and accounts**. It does not establish a cross-civilizational council-size law, a standard preindustrial municipal budget, or a population threshold for self-government.

### 2.2 Proposed simulation settings—not historical estimates

These are initial experimental ranges. Their source is the modeling recommendation in this report, and their empirical confidence is **low/unestimated**.

| Parameter | Initial range or domain | Unit | How to use it |
| --- | --- | --- | --- |
| Working executive size | **6–24** | Seats | Compare coordination costs and concentration of power; allow smaller or larger authored constitutions |
| Representative council size | **20–200** initially; support **500+** | Seats | Do not apply to plenary assemblies or force proportionality to population |
| Ordinary rotating-office term | **3–12** | Months | One scenario family only; separately support daily, annual, lifetime, hereditary, and at-pleasure offices |
| Realized collection ratio | Stress-test **0.4–0.95**; legal domain **0–1** | Collected/assessed revenue | Prefer to generate from collector capacity, concealment, compliance, and exemptions |
| Desired treasury reserve | **1–6** | Months of recurring expenditure | A decision target, not guaranteed cash; allow shortages and hoarding |
| Maintenance funding | Stress-test **0.5–1.25** | Funding/estimated engineering requirement | Connect accumulated shortfalls to deterioration; spending above need should not create unlimited improvement |
| Routine political reevaluation | **30–90** | Simulation days | Numerical scheduling choice; crises, vacancies, and proposals trigger immediate relevant decisions |

**Do not assign a universal remittance percentage.** Derive it from bargaining and existing obligations. Likewise, calculate franchise size from eligibility rules rather than sampling a generic “medieval voter share.”

For comparative scenarios, express costs in local wages, staple quantities, labor-days, and shares of recurring receipts. Preserve historical monetary examples as evidence of contractual structure, not as directly portable prices.

---

## 3. Variation across eras and regions

### 3.1 Era differences are differences in constraints—not mandatory stages

| Setting | Evidence and institutional possibilities | TCE implication |
| --- | --- | --- |
| **Foragers** | Boehm documents collective practices that constrain would-be dominant individuals. This is evidence about political organization, not evidence that all foragers were egalitarian or possessed municipal councils. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/204166) | Support task leadership, assemblies, withdrawal, and collective sanctions before formal offices. A city-government module need not yet exist. |
| **Early farming and aggregation** | Archaeological approaches emphasizing heterarchy caution against inferring a single ranked authority from social complexity. Exact constitutions are often much less recoverable than buildings or settlement patterns. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/abs/10.1525/ap3a.1995.6.1.1) | Allow household, neighborhood, ritual, and works organizations to overlap. Increasing population raises coordination demands but does not choose the political solution. |
| **Preindustrial urbanism** | Athens, Roman municipal law, European communes, and Asian associations demonstrate different combinations of corporate action and superior authority. Roman municipal charters are especially important counterexamples to equating local institutions with independence. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.3.3.html) | Permit councils, appointed officials, courts, and associations simultaneously. Physical travel, information, and revenue collection constrain effective authority. |
| **Industrial urbanism** | English experience included elected municipal corporations after the **1835 reform**, alongside separate poor-relief and public-health institutions. Reform did not instantly create a single, universally inclusive municipal system. [House of Commons Library](https://commonslibrary.parliament.uk/research-briefings/cbp-10233/) | Expand specialized services when problems, capabilities, and funding justify them. Institutional consolidation can follow conflict or failure rather than occur automatically. |
| **Modern systems** | The Local Autonomy Index distinguishes multiple dimensions of self-rule; OECD/UCLG fiscal work records substantial variation in subnational organization and finance. [European Commission](https://ec.europa.eu/regional_policy/information-sources/publications/studies/2021/self-rule-index-for-local-authorities-in-the-eu-council-of-europe-and-oecd-countries-1990-2020_en) | Elections, taxation, spending discretion, staffing, and central supervision remain separate. Modernization does not eliminate intergovernmental bargaining. |

### 3.2 European forms should overlap rather than become exclusive regime classes

**Communes and Italian city-states.** Communal government emerged through local collective organization and changing relations with existing authorities. It was not necessarily founded with a complete republican blueprint. Use the commune as a coalition and institutional-development process; use *city-state* for a polity whose governing center is a city, keeping its territory and subordinate settlements separate from the built-up area. Wickham provides the strongest source here for contingent emergence. [OUP Academic](https://academic.oup.com/princeton-scholarship-online/book/15824)

**Chartered towns.** Represent a charter as a recognized agreement over particular powers and obligations. Multiple charters can accumulate, confirm, clarify, or conflict. A town need not have all its privileges in one founding document; English municipal history includes both chartered rights and claims resting on established custom. [House of Commons Library](https://commonslibrary.parliament.uk/research-briefings/cbp-10233/)

**Guild-governed towns.** Guild participation is a rule about access to government, not a complete description of the constitution. Model which organizations have seats, how their delegates are selected, and which inhabitants fall outside their membership. Wahl’s database is useful precisely because it distinguishes guild participation from other participative features. [WU Wirtschaftsuniversität Wien](https://research.wu.ac.at/de/publications/participative-political-institutions-in-pre-modern-europe-introdu/)

**Imperial cities.** The distinctive arrangement was direct subjection to the emperor rather than an intervening territorial lord, while retaining an urban governing corporation. Such cities remained part of a larger political order; “free” did not mean absence of obligations or unrestricted participation. Their status could also be disputed rather than cleanly binary. [Wikipedia](https://de.wikipedia.org/wiki/Heiliges_R%C3%B6misches_Reich?utm_source=chatgpt.com)

For TCE, these labels can coexist: a city might be an imperial city externally, guild-influenced internally, and governed through a collection of chartered privileges. Its label should summarize the underlying rules, not generate them.

### 3.3 Non-European cases change the model, not merely its vocabulary

**China: association alongside imperial administration.**  
Moll-Murata distinguishes occupational associations from native-place associations, while showing considerable overlap in their functions. Such bodies organized common property and mutual assistance and sought recognition from local administration. They could complement, cooperate with, or sometimes displace parts of official urban administration. **TCE implication:** local collective capacity can be substantial without municipal sovereignty. Membership should arise through occupation, migration, origin, and networks—not only territorial residence. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/chinese-guilds-from-the-seventeenth-to-the-twentieth-centuries-an-overview/61A96EB7FF67CE0BE8073C36DDB049CD)

**Islamic and Ottoman settings: separate property-based service institutions.**  
A waqf could dedicate revenue-producing assets to specified purposes, administered through its own arrangements rather than the general town budget. This permits extensive provision without a European-style city corporation. Abu-Lughod also warns against constructing one timeless “Islamic city” from a narrow set of examples. **TCE implication:** model endowments, religious institutions, local associations, and superior authorities separately; do not encode a civilization-wide absence of urban organization. [Sites@Duke Express](https://sites.duke.edu/timurkuran/files/2016/10/waqf-2001-1.original.pdf)

**South India: distinguish town, village, and territorial assemblies.**  
Tamil epigraphic research distinguishes the **ūr**, **sabhā**, **nagaram**, and **nāḍu** rather than treating every local assembly as the same institution. The nagaram is directly relevant to commerce and towns. Karashima also criticizes interpretations that overstate the democratic character of selected assembly rules while neglecting social structure. **TCE implication:** give assemblies different membership and territorial scopes; an impressive selection procedure does not establish universal eligibility. [FAS](https://fas.org.in/wp-content/uploads/2026/02/S7_Karashima_Epigraphical_Study_of_Ancient_and_Medievil_Villages_in_the_Tamil_Country.pdf)

**Japan: civic organization embedded in religious and commercial space.**  
Sakai’s archaeological presentation identifies the **kaigōshū** with self-government in the southern district from the late fifteenth century and associates their deliberation with a shrine-temple meeting complex. The documented repair subscription illustrates organized collective finance. **TCE implication:** meeting places, ritual obligations, commercial leadership, and political administration can share people and buildings. Do not require a purpose-built town hall before municipal action becomes possible. [Sakai City](https://www.city.sakai.lg.jp/kanko/rekishi/bunkazai/bunkazai/tenjikai/chanoyu/kaisho.html)

**Mesoamerica: collective government without a European ancestry.**  
Research on Tlaxcallan reconstructs a substantial governing council with responsibilities extending to adjudication, diplomacy, warfare, and supervision of officials. The extent and meaning of its “democracy” remain interpretive questions; the evidence should not be reduced to either modern democracy or concealed monarchy. **TCE implication:** support collective government through local institutions and coalition structures independently of any European institutional lineage. [Frontiers](https://www.frontiersin.org/journals/political-science/articles/10.3389/fpos.2022.832440/full)

**West Africa: urban complexity need not imply a palace-centered state.**  
The McIntoshes’ work on Jenne-jeno and the Middle Niger challenges expectations that urbanism must be organized around a citadel or royal center. This supports a wider space of possible urban organizations, but **absence of a palace is not proof of elections or equality**. **TCE implication:** permit urban clusters with multiple centers of authority and specialization; archaeological uncertainty should not be converted into a confidently invented constitution. [Taylor & Francis](https://www.taylorfrancis.com/chapters/edit/10.4324/9780203754245-38/cities-without-citadels-understanding-urban-origins-along-middle-niger-susan-keech-mcintosh-roderick-mcintosh)

---

## 4. Stylized facts a correct simulation should reproduce

These are **conditional validation targets**, not events to script into every world.

| Pattern | Test for TCE |
| --- | --- |
| **The identity and capacity of the overlord matter.** Medieval English farm grants differed sharply between royal and mesne boroughs. [UCLA Anderson School of Management](https://www.anderson.ucla.edu/faculty_pages/nico.v/Research/AMV_Slides.pdf) | Matched towns under rulers with different administrative costs should sometimes obtain different rights. Reproducing exactly 51.0% and 3.9% is appropriate only for a specifically calibrated English scenario. |
| **Participation is multidimensional.** Guild seats, electoral procedures, and burgher representation varied separately. [WU Wirtschaftsuniversität Wien](https://research.wu.ac.at/de/publications/participative-political-institutions-in-pre-modern-europe-introdu/) | Municipal autonomy must not mechanically increase the voter share, competition for office, and service equality together. |
| **Large councils can coexist with small operational bodies.** Athens distinguished 500 councillors from the presiding group of 50. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.2.2.html) | Deliberative breadth need not require every member to perform every administrative task. |
| **Services have multiple providers.** Endowments could finance urban provision outside a municipal treasury. [Sites@Duke Express](https://sites.duke.edu/timurkuran/files/2016/10/waqf-2001-1.original.pdf) | Some towns should provide functioning services despite a small council budget; municipal expenditure alone must not measure total provision. |
| **Institutions can lose autonomy while organizational names persist.** Chinese guild-to-association reforms illustrate changes in supervision and autonomy. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/chinese-guilds-from-the-seventeenth-to-the-twentieth-centuries-an-overview/61A96EB7FF67CE0BE8073C36DDB049CD) | Constitutional and practical powers should change independently of labels and buildings. |
| **Political forms can reverse under internal and external pressure.** The *Athenian Constitution* describes coercively imposed oligarchic change involving domestic factions and outside power. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.2.2.html) | A settlement can remain large and commercially important while its franchise or autonomy contracts. |
| **Fiscal dependence is compatible with local organization.** The RBI example combines own revenues with substantial transfers. [City Finance](https://www.cityfinance.in/assets/images/homepage/spotlight/rbi-report-on-municipal-finances.pdf) | A council can remain operational yet have major projects or priorities constrained by an outside financier. |

Also test **negative cases**: prosperous towns that never gain broad participation, autonomous towns that administer services badly, and appointed administrations that deliver services competently. These are necessary counterexamples to overly deterministic rules.

---

## 5. Modeling recommendation for 10k–50k individual agents

### 5.1 Use institutions as durable entities, not substitute populations

A compact proposed representation is:

```
Institution
  members and membership rules
  assets, treasury, liabilities
  offices and decision procedures
  recognized powers
  territorial and personal jurisdiction
  obligations to other institutions

Office
  holder
  selection and dismissal procedure
  eligibility requirements
  term
  powers and workload
  remuneration
  review or audit procedure

AuthorityGrant
  grantor and grantee
  specific power or function
  geographic/person scope
  conditions and obligations
  amendment, expiry, and revocation procedure
```

This is compatible with Crawford and Ostrom’s institutional grammar: identify **who** a rule concerns, what they **may/must/must not** do, the action, its conditions, and any sanction. The same framework can represent formal rules, norms, and shared strategies without pretending that all are enforced identically. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/grammar-of-institutions/7D37CD3BC5ED2D9FD57D2EE292958F47)

Keep a **settlement**, its **municipal corporation**, and its **overarching polity** as different objects. Courts and service institutions may cover different territories or classes of people.

### 5.2 Keep political behavior attached to people

Each relevant agent needs wealth and income exposure, residence, legal status, occupation, organizational memberships, personal ties, reputation, office history, and experienced burdens or benefits.

Do not assign every merchant one permanent preference. A merchant exporting food, one importing grain, and one financing the ruler can take different sides on the same policy.

Institutions may maintain aggregate statistics, but their decisions should come from authorized officeholders and participating members. “The town” should never know every inhabitant’s preferences or maximize total welfare automatically.

### 5.3 Use event-driven political computation

At this scale, avoid all-pairs political reasoning.

| Cadence | Proposed work |
| --- | --- |
| Daily | Service labor, collection encounters, inspections, patrols, hearings, construction |
| Periodic fiscal cycle | Accounts, payroll, liabilities, maintenance needs, arrears |
| Proposal-triggered | Identify affected groups, solicit support, negotiate, deliberate, vote |
| Event-triggered | Vacancies, succession, default, scandal, military threats, charter disputes |
| Slower institutional review | Membership changes, eligibility disputes, constitutional amendments |

Cache household, guild, ward, and taxpayer summaries. Update them when membership or economic circumstances change. Mobilize broader networks only when an issue matters to them.

This retains persistent individual careers and relationships without requiring 50,000 agents to reconsider every ordinance every day.

### 5.4 Simplify institutional variety without flattening it

The minimum useful implementation is not a catalogue of dozens of historical office names. Start with **one ruler–town relationship; a council; membership-based associations; a fiscal ledger; and several assignable functions**.

Add appointments, elections, lot, and co-option as reusable procedures. Let the same functional office be called a bailiff, commissioner, elder, or magistrate according to local convention.

The highest-value detail is the connection between **powers, obligations, resources, and people**. Architectural and naming variety can be generated from that institutional state.

### 5.5 Existing models and games: borrow components, not complete explanations

| Model or game | Useful component | Limitation for TCE |
| --- | --- | --- |
| **Crawford–Ostrom institutional grammar; nADICO** | Structured, composable rules and nested institutional statements | A representation framework, not a calibrated theory of urban emergence. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/grammar-of-institutions/7D37CD3BC5ED2D9FD57D2EE292958F47) |
| **Ostrom, Tiebout, and Warren’s metropolitan-government analysis** | Multiple governing centers and different arrangements for different services | An institutional framework rather than a historical individual-agent simulator. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/organization-of-government-in-metropolitan-areas-a-theoretical-inquiry/00789721D92451B3D13E6904C72D7F8A) |
| **Greif’s Genoa analysis** | Self-enforcing political arrangements and strategic incentives | A focused analytical model, not a complete municipal economy. [JSTOR](https://www.jstor.org/stable/j.ctv131bwhx) |
| **Joshua Epstein’s civil-violence ABM** | Local interaction between grievance, perceived risk, and authority | Useful for a protest or repression subsystem; it does not generate councils, budgets, or charters. [PNAS](https://www.pnas.org/doi/10.1073/pnas.092080199?utm_source=chatgpt.com) |
| **Eco** | Player-authored constitutions, elections, laws, and government procedures | Human players supply political reasoning; automatic law enforcement should not become TCE’s model of compliance. [Eco - English Wiki](https://wiki.play.eco/en/Government) |
| **Frostpunk 2** | Council bargaining and conflict among groups with evolving priorities | Its designed faction framework is useful interface inspiration, not evidence that historical urban interests fall into fixed categories. [Polygon](https://www.polygon.com/gaming/451037/frostpunk-2-interview-jakub-stokalski-11-bit-games) |

---

## 6. Source priorities, datasets, and limits

### Recommended research backbone

**For bargaining and European institutional development:** Charles Tilly, *Coercion, Capital, and European States*; Chris Wickham, *Sleepwalking into a New World*; and Angelucci, Meraglia, and Voigtländer, “How Merchant Towns Shaped Parliaments,” *American Economic Review* **112(10), 3441–3487, 2022**. Use Tilly for hypotheses, Wickham for institutional emergence, and Angelucci and colleagues for a geographically specific empirical test—not as three interchangeable sources of universal coefficients. [UC San Diego Pages](https://pages.ucsd.edu/~bslantchev/courses/ps240/06%20Domestic%20Organizations%20and%20International%20Behavior/Tilly%20-%20Coercion%2C%20capital%20and%20European%20states%20%5BCh%201%2C3%2C6%5D.pdf)

**For non-European institutional diversity:** Moll-Murata on Chinese guilds; Karashima and colleagues on Tamil assemblies and the nagaram; Kuran on waqf provision read alongside Abu-Lughod’s critique of the “Islamic city”; Fargher and colleagues on Tlaxcallan; and the McIntoshes on Middle Niger urbanism. Their evidentiary foundations differ substantially: organizational records, inscriptions, legal arrangements, archaeology, and retrospective accounts should not be treated as equivalent measurements. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/chinese-guilds-from-the-seventeenth-to-the-twentieth-centuries-an-overview/61A96EB7FF67CE0BE8073C36DDB049CD)

### Datasets worth implementing against

| Dataset or evidence collection | Coverage | Main use and caution |
| --- | --- | --- |
| **Wahl, “Participative Political Institutions in Pre-modern Europe”** | Original database: **104 Central European cities, 800–1800** | Separate guild participation, election procedures, and burgher representation. Regional selection is not global representativeness. [WU Wirtschaftsuniversität Wien](https://research.wu.ac.at/de/publications/participative-political-institutions-in-pre-modern-europe-introdu/) |
| **Angelucci–Meraglia–Voigtländer historical borough data** | English boroughs, charters, parliamentary representation, and later institutional outcomes | Best suited to reproducing a defined historical setting and testing delegation mechanisms. Use version-consistent data and results. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20200885) |
| **Local Autonomy Index 2.0** | **57 countries, 1990–2020; 11 variables grouped into seven dimensions** | Useful design vocabulary for separating powers. Do not apply modern expert-coded scores directly to ancient cities. [European Commission](https://ec.europa.eu/regional_policy/information-sources/publications/studies/2021/self-rule-index-for-local-authorities-in-the-eu-council-of-europe-and-oecd-countries-1990-2020_en) |
| **OECD/UCLG World Observatory, 2022 edition** | **135 countries** | Useful modern institutional and fiscal comparison. Subnational totals can include states or provinces, not just municipalities. [Subnational Finance Observatory](https://www.sng-wofi.org/) |
| **RBI, Report on Municipal Finances, 2022** | Municipal-corporation accounts, with a usable sample of **201 corporations** | Separate actual accounts from revised and budget estimates; inspect reporting gaps before calibration. [City Finance](https://www.cityfinance.in/assets/images/homepage/spotlight/rbi-report-on-municipal-finances.pdf) |

### What remains uncertain

There is no defensible single coefficient linking urban population, trade, or wealth to municipal autonomy. The reviewed literature is also insufficient to supply universal preindustrial staffing ratios, tax burdens, service coverage, or franchise shares. These should remain **scenario-calibrated or endogenous quantities**, rather than invented historical constants.

The principal interpretive risks are to mistake legal provisions for actual practice; surviving records for representative samples; corporate participation for universal inclusion; and archaeological absence for proof of a particular constitution. Guild efficiency, waqf flexibility, and the democratic interpretation of Tlaxcallan are especially important areas where alternative scholarly interpretations should remain visible. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/craft-guilds-apprenticeship-and-technological-change-in-preindustrial-europe/4B18A7808BACBFA40D76475FBCB665E0)

**Bottom line:** TCE’s towns should become self-governing when organized people acquire and sustain particular powers—not when a settlement reaches a prescribed size. The same machinery should then allow them to fund services, exclude outsiders, negotiate with rulers, federate, fall into debt, lose privileges, or replace their own governing coalition. Municipal history can emerge from those changing relationships without requiring a scripted sequence of regimes.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928da-2d14-83ea-ab9c-0e5f306ae167)
