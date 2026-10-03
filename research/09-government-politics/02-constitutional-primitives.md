# Decomposing governments into primitives

## A simulation-ready institutional grammar for The Civilization Engine

**Recommendation:** Build governments as **networks of positions, constituencies, jurisdictions, powers, and procedures**, not as instances of predefined regime types. Run that institutional network alongside agents’ beliefs, relationships, resources, and physical actions.

The crucial distinction is:

> **An institution specifies who is entitled or obliged to do something. It does not guarantee that anyone will comply.**

A constitutional monarch, a military dictator, and an elected president might possess some identical formal powers. Their governments differ in how officeholders are selected, whose cooperation they need, which resources they control, and what happens when someone refuses an order.

Below, I propose **six primitive families**. They can express the requested historical cases without government-specific engine code. This is a practical engineering basis, not a proof that six is the uniquely smallest possible ontology—or that the worked cases exhaust every historical institution.

---

## 1. The minimum practical institutional language

### 1.1 Six primitive families

These are semantic families; each can contain several strongly typed authoring records.

| Primitive family | Required contents | What it makes possible |
| --- | --- | --- |
| **1. Bodies and constituencies** | Membership definition; constituent persons, offices, or other bodies; representation relationships; admission and exit | Councils, assemblies, clans, parties, estates, ministries, confederacies, electorates |
| **2. Statuses and positions** | Eligibility; number of holders; entry and exit conditions; tenure; succession; remuneration; compatible and incompatible roles | Citizens, subjects, heirs, magistrates, monarchs, regents, judges, hereditary titleholders |
| **3. Jurisdictions** | Territory, persons, subject matter, time, and relationships to competing jurisdictions | Municipal autonomy, federalism, personal law, ecclesiastical jurisdiction, provincial commands, overlapping sovereignty |
| **4. Actions and powers** | Typed action; authorized actor; target; scope; delegation limits; required resources | Legislating, taxing, appointing, commanding, judging, pardoning, borrowing, granting privileges |
| **5. Procedures** | Stages, participants, information access, aggregation rules, deadlines, certification, appeals, failure paths | Elections, inheritance, examinations, trials, consensus, budgets, impeachment, constitutional amendment |
| **6. Institutional statements** | Conditions; permissions, obligations, prohibitions, or constitutive effects; issuing authority; recognition; consequences; version | Laws, customary rules, office definitions, legal rights, procedural validity, amendment rules |

**Bodies should not automatically have minds.** A council acts through its members and decision procedure. A ministry acts through officials, delegated responsibilities, and administrative routines.

**Positions must be separate from people.** One person may hold several offices; one office may have several simultaneous holders; several people may claim the same supposedly exclusive office.

**Jurisdiction must not be a single-owner map.** A person can simultaneously owe military service to one lord, taxes to another authority, and obedience to a religious court in particular matters.

### 1.2 Powers need a small, reusable action vocabulary

Start with these action groups:

| Group | Representative actions |
| --- | --- |
| Constitutive | Establish or dissolve a body; recognize a status; create an office; amend a rule |
| Personnel | Nominate, appoint, confirm, promote, suspend, remove, designate a successor |
| Legislative | Propose, deliberate, enact, veto, promulgate, repeal |
| Fiscal | Assess, collect, exempt, appropriate, spend, borrow, audit |
| Executive | Issue orders, delegate, inspect, commission work, administer assets |
| Coercive | Mobilize, command, detain, seize, enforce a judgment |
| Judicial | Hear a claim, receive evidence, determine facts, interpret, adjudicate, appeal, pardon |
| External | Negotiate, recognize another authority, conclude treaties, declare war or peace |
| Informational | Demand records, publish, conceal, petition, investigate, report |

These are **actions**, not guaranteed outcomes. “Order a levy” creates instructions and obligations; it does not instantly create soldiers.

A power should therefore resemble:

\[
\text{Power}=
(\text{holder},\text{action},\text{target},\text{jurisdiction},
\text{conditions},\text{delegability},\text{expiry})
\]

For example, authority to collect an already approved tax is distinct from authority to set its rate, exempt taxpayers, spend the proceeds, or adjudicate disputes.

### 1.3 Selection methods are procedure compositions

Do not implement:

```
selection = election | inheritance | appointment
```

Implement:

```
define eligible population
→ admit or nominate candidates
→ screen candidates
→ select, rank, or bargain
→ obtain required confirmations
→ certify the result
→ install the officeholder
```

The individual stages can use descent, seniority, examination, lot, voting, appointment, co-option, purchase, rotation, acclamation, or combinations.

Separate **eligibility**, **nomination**, **selection**, and **recognition**. A designated heir might be eligible but rejected; an examination graduate might qualify for appointment without receiving a job.

### 1.4 Checks are relationships, not bonuses

A “check” is a composition of powers and procedures:

* **Veto:** another actor can suspend or reject a transition, possibly subject to an override.
* **Accountability:** someone can obtain information, investigate, judge, and impose consequences.
* **Separation:** incompatible offices, independent appointment channels, or separate control of money and force.

A court with review authority but no access to evidence, no cooperative enforcement personnel, and judges dependent on the defendant has a different practical effect from an independently supported court. TCE should discover that difference through behavior, not through a fixed “judicial independence +20” modifier.

---

## 2. What institutional grammar and comparative data contribute

### 2.1 ADICO: useful syntax, not a complete government model

Crawford and Ostrom’s original institutional grammar decomposes statements into:

| Component | Meaning | Illustrative tax rule |
| --- | --- | --- |
| **A — Attributes** | To whom the statement applies | Registered cultivators |
| **D — Deontic** | May, must, or must not | Must |
| **I — Aim** | Required or permitted action | Deliver the assessed grain |
| **C — Conditions** | When and where it applies | After assessment, before the deadline |
| **O — Or else** | Consequence of noncompliance | Enter a specified enforcement procedure |

In the original analytical scheme, **AIC** describes a shared strategy, **ADIC** a norm, and **ADICO** a rule. This is a coding framework—not a claim that a legal provision without an explicit penalty is invalid. Its sanction may be established elsewhere. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/grammar-of-institutions/7D37CD3BC5ED2D9FD57D2EE292958F47)

ADICO alone is insufficient for TCE because many essential rules are **constitutive**: they establish that a particular event creates an officeholder, a valid judgment, citizenship, or an enforceable obligation. Institutional Grammar 2.0 explicitly extends the framework to constitutive statements and richer logical structures. [arXiv](https://arxiv.org/abs/2008.08937)

Ostrom’s broader institutional-analysis tradition also distinguishes **position, boundary, choice, aggregation, information, payoff, and scope rules**. These are valuable coverage checks: a government description that specifies offices and voting but omits information and payoffs is structurally incomplete. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/psj.12466)

**TCE translation:** use an ADICO-inspired representation for obligations and permissions, a constitutive representation for recognized status changes, and procedure graphs for temporal interaction.

### 2.2 CCP: a field inventory, not a universal ontology

The Comparative Constitutions Project systematically codes formal written national constitutions, including historical texts, principally for independent states since 1789. It is excellent for discovering overlooked distinctions in executive selection, legislatures, courts, amendment, emergency powers, rights, and territorial organization. It is not designed to cover all premodern, oral, local, or informal institutions. [Comparative Constitutions Project](https://comparativeconstitutionsproject.org/research-design/)

As checked for this report, CCP’s download page lists **Constitutional Chronology v6, updated through 2025**, and **Constitutional Characteristics v5, modified February 2025**, covering more than 200 countries. Pin the exact dataset version rather than building against an unspecified “latest” file. [Comparative Constitutions Project](https://comparativeconstitutionsproject.org/download-data/)

Use several datasets for different purposes:

| Source | Best TCE use | Main limitation |
| --- | --- | --- |
| **CCP** | Formal institutional fields and historical constitutional configurations | Written national constitutional provisions are not observed behavior |
| **V-Dem, v16** | Outcomes and practice: executive constraints, participation, appointment patterns, political competition | Expert-coded estimates have uncertainty; scores across releases are not automatically interchangeable. [V-Dem](https://v-dem.net/data/the-v-dem-dataset/) |
| **Seshat** | Long-run administrative, informational, military, and political organization | Historical uncertainty, aggregation, and uneven source coverage require explicit handling. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5777031/) |
| **D-PLACE** | Kinship, descent, local organization, stratification, and cross-cultural variation | Much underlying ethnography concerns the nineteenth and early twentieth centuries, not direct observations of prehistory. [D-PLACE](https://d-place.org/about) |

Preserve **absent**, **unknown**, **not applicable**, and **contested** as different values.

---

## 3. Mechanisms: rules the simulation can implement

The following are proposed model mechanisms. Their equations are engineering formulations, not universally estimated historical laws.

### 3.1 Access to power is a funnel

For each political process, construct separate sets:

\[
\text{population}
\supseteq \text{formally eligible}
\supseteq \text{aware and able to participate}
\supseteq \text{actual participants}
\]

Candidate selection introduces additional filters.

Participation should consume time, travel, information, and sometimes money. Two societies with identical franchise rules can therefore generate different effective participation.

**Implement:** determine eligibility from institutional rules, then determine attendance and candidacy from individual constraints and motives.

### 3.2 Compliance is a decision, not an automatic rule effect

A useful starting comparison is:

\[
\Delta U\_{\text{defy}}
=
G
-
p\_D\,p\_{E|D}\,S
-
C\_{\text{reputation}}
-
C\_{\text{norm}}
+
B\_{\text{patron}}
\]

Here \(G\) is the benefit of disobedience, \(p\_D\) perceived detection probability, \(p\_{E|D}\) perceived enforcement probability conditional on detection, and \(S\) expected sanction cost. All cost and benefit terms must use the same agent-relative utility scale.

This permits a person to obey from conviction, fear, solidarity, or dependence—and to disobey despite severe written penalties.

**Implement:** derive perceived probabilities from observations, rumors, trusted relationships, and prior encounters. Do not give agents omniscient knowledge of enforcement.

### 3.3 Administration is constrained by workload

For a monitoring or adjudication office:

\[
\text{potential cases processed}
=
\frac{\text{available staff-days}}
{\text{staff-days required per case}}
\]

Actual completion also requires travel, evidence, records, supplies, and cooperation.

**Implement:** assessments, petitions, trials, audits, and appeals enter queues. Backlogs create delay, discretion, and opportunities for bribery or selective service.

The importance of monitoring has causal support in a particular modern setting: Olken’s randomized Indonesian road-project study increased announced government audit probability from 4% to 100% and reduced measured missing expenditures by approximately eight percentage points. That supports a monitoring mechanism, not a universal corruption coefficient. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/pdf/10.1086/517935)

### 3.4 Delegation creates both capacity and agency problems

An appointer chooses a subordinate using competence, loyalty, relationships, expected revenue, and political support. The subordinate balances instructions against local incentives and personal networks.

**Implement:** delegation specifies scope, reporting obligations, renewal, revocation, and subdelegation. A governor’s official superior need not be their strongest patron.

Authoritarian power-sharing research similarly treats rulers’ relationships with governing coalitions as strategic and potentially unstable, rather than assuming that a dictator can command everything without constraint. [Illinois Experts](https://experts.illinois.edu/en/publications/power-sharing-and-leadership-dynamics-in-authoritarian-regimes/)

### 3.5 Bargaining requires agenda control and credible alternatives

Record who may introduce proposals, who sees them, who can amend them, when voting occurs, and what happens after failure.

A defeated budget might leave previous spending in place, shut down particular activities, permit temporary spending, or trigger another procedure. Those are different institutions.

**Implement:** a veto changes the feasible next steps. It should not automatically select a compromise that benefits everyone.

### 3.6 Succession produces claims before it produces a settled ruler

Death, incapacity, resignation, removal, and term expiry trigger different procedures.

**Implement:** create candidates or claimants, temporary office arrangements, recognition decisions, and attempts to gain control of treasuries, officials, and troops. Allow a regent to exercise powers without possessing the sovereign’s title.

A completed legal selection can coexist with incomplete practical installation.

### 3.7 Institutional change is itself political action

Agents should propose changes when expected benefits exceed organizational costs and risks. Repeated practices can also become shared expectations without a single founding act.

**Implement:** changes can be attempted through authorized amendment, reinterpretation, negotiated exception, persistent noncompliance, or coercive replacement. Adoption and implementation remain separate events.

Do not require “democracy technology” before people can vote, or a particular era before a ruler can appoint a council.

---

## 4. Worked historical decompositions

Each case is a **dated or explicitly bounded configuration**, not a timeless cultural template.

### 4.1 Athens: late fourth-century BCE institutions

**Composition.** Adult male citizen participation; a citizen assembly; a council selected by lot with tribal allocation; numerous annually filled magistracies; elected military officers; citizen courts; scrutiny before office and accounting afterward. Different functions used different selection mechanisms. Aristotle’s *Athenaion Politeia* describes both the council’s agenda role and extensive judicial and financial oversight. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.2.2.html)

**TCE stress test:** sortition, election, direct participation, representation by territorial subdivision, and post-office accountability must coexist. Do not give every resident political citizenship or every office the same selection method.

### 4.2 Roman Republic: the middle-Republican configuration described by Polybius

**Composition.** Two consuls; a senate with substantial financial and diplomatic responsibilities; popular assemblies with electoral and other decision powers; tribunes capable of obstructing senatorial action. Commanders depended on resources and decisions controlled elsewhere.

Polybius emphasizes mutual dependence among consuls, senate, and people, but his description is an interpretive account of a “mixed constitution,” not a neutral modern administrative manual. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Polybius/6%2A.html)

**TCE stress test:** distribute proposal, authorization, funding, command, and retrospective accountability among different actors. “Republic” must not imply a single sovereign parliament.

### 4.3 Roman Empire: the Augustan principate

**Composition.** Retained republican offices and bodies, with exceptional powers and accumulated authority concentrated around one person. Augustus presented his position through established titles and powers; Tacitus stresses military rewards, provisioning, elite accommodation, and the concentration of functions. These sources offer markedly different political interpretations. [Livius](https://www.livius.org/sources/content/augustus-res-gestae/)

**TCE stress test:** permit institutional capture without wholesale constitutional replacement. A single agent’s overlapping offices, appointments, patronage, and command relationships can transform the regime while many old records remain intact.

### 4.4 Venice: the dogal selection procedure established in 1268

**Composition.** A restricted Great Council constituency and an elaborate alternation of lottery and election. The successive selection sizes were:

\[
n\rightarrow30\rightarrow9\rightarrow40\rightarrow12
\rightarrow25\rightarrow9\rightarrow45\rightarrow11\rightarrow41
\]

The final college selected the doge. Family-related restrictions and intermediate elections attempted to complicate factional control. Mowbray and Gollmann analyze this as a selection protocol, including its properties for minorities within the privileged electorate. [ShiftLeft](https://shiftleft.com/mirrors/www.hpl.hp.com/techreports/2007/HPL-2007-28R1.pdf)

**TCE stress test:** one “elect ruler” operation is inadequate. Preserve the intermediate bodies, nomination decisions, family exclusions, and opportunities for bargaining. Procedural complexity within an oligarchy does not make participation universal.

### 4.5 Haudenosaunee Confederacy: the traditional constitutional framework

**Composition.** Named chiefly positions associated with constituent nations and clans; clan mothers with selection and removal authority; deliberation through differentiated benches rather than a flat majority vote. The Onondaga Nation describes proposals moving among elder-brother, younger-brother, and Firekeeper groupings, including reconsideration. [Onondaga Nation](https://www.onondaganation.org/government/)

**TCE stress test:** distinguish officeholding from the authority to select officeholders. Model consensus as a structured, potentially iterative process—not a 100% threshold applied to an undifferentiated assembly.

These community descriptions are important primary sources for the traditional system, but they should not be treated as proof that every detail operated identically throughout the Confederacy’s entire history.

### 4.6 Song China: civil-service organization, with a 1213 recruitment snapshot

**Composition.** Dynastic rulership; a differentiated civil service; personal rank distinct from functional assignment; competitive examinations alongside protected family-entry routes and other recruitment channels. Winston Lo’s personnel study shows why “meritocratic bureaucracy” cannot be reduced to examination-only recruitment: *yin* privilege was a recognized institutional benefit, not simply illicit nepotism. [dokumen.pub](https://dokumen.pub/an-introduction-to-the-civil-service-of-sung-china-with-emphasis-on-its-personnel-administration-9780824888084.html)

**TCE stress test:** distinguish qualification, rank, salary entitlement, actual appointment, and promotion. A person can possess prestigious credentials or rank while waiting for an administrative assignment.

### 4.7 Mongol Empire: early imperial government and succession

**Composition.** Great-khan authority, dynastic claims and elite acknowledgment, commanders and royal households, delegated territorial administration, and military organization using decimal unit categories. Succession depended on political coordination; imperial women and their households could be central to regency and succession struggles. [AFE East Asia](https://afe.easia.columbia.edu/mongols/main/transcript.pdf)

**TCE stress test:** a ruler’s death should activate household, kinship, command, and recognition networks—not merely copy authority to the eldest child.

**Evidence caution:** the existence of a single coherent “Great Yasa” law code is contested. Pochekaev, building on earlier scholarship, argues for principles and rulings rather than the conventional image of a comprehensive codified constitution. [Золотоордынское обозрение](https://goldhorde.ru/en/stati2016-4-2/)

### 4.8 Medieval monarchies: two different constructions

**England, the intended 1215 Magna Carta arrangement.** Royal authority coexisted with specified obligations and privileges. Clauses 12–14 constrained particular aids and scutage, provided exceptions, and specified summons for obtaining consent. This was not universal parliamentary control over all taxation, nor should a charter’s intended settlement be mistaken for stable implementation. [Internet History Sourcebooks](https://sourcebooks.fordham.edu/source/magnacarta.asp)

**Holy Roman Empire, Golden Bull of 1356.** A defined electoral college selected the ruler by majority, while electors retained substantial jurisdictional and economic privileges. The monarch and territorial rulers therefore possessed intersecting bundles of authority. [Internet History Sourcebooks](https://sourcebooks.fordham.edu/source/goldenbull.asp)

**TCE stress test:** hereditary and elective monarchy must both be expressible. Territorial sovereignty cannot be reduced to a uniform chain of administrative subordinates.

### 4.9 Parliamentary government: Germany’s constructive no-confidence procedure

**Composition.** Separate head-of-state and head-of-government offices; a parliamentary route to replacing the chancellor. Article 67 requires the Bundestag to elect a successor by a majority of its members and request the incumbent’s dismissal; the president must comply. At least 48 hours must separate the motion and election. [Gesetze im Internet](https://www.gesetze-im-internet.de/englisch_gg/englisch_gg.html)

**TCE stress test:** removing an executive can require agreement on a replacement. This differs from a procedure where a majority can dismiss without constructing a successor coalition. Do not apply Germany’s particular mechanism to all parliamentary systems.

### 4.10 Presidential government: the United States constitutional configuration

**Composition.** Separately selected executive and legislature; different terms; bicameral lawmaking; presidential veto and legislative override; appointments involving divided responsibilities; impeachment and removal through procedures distinct from ordinary political disagreement. [National Archives](https://www.archives.gov/founding-docs/constitution-transcript)

**TCE stress test:** losing legislative support must not automatically remove a fixed-term executive. Separately track executive selection, legislative control, ordinary legislation, appropriations, appointments, and removal.

### 4.11 One-party government: a party-state configuration

**Composition.** Parallel party and state organizations, with overlapping officeholders and distinct appointment, supervision, and command relationships. In China’s constitution as revised in 2018, party leadership is expressly recognized; the state also contains differentiated legislative, administrative, judicial, supervisory, and military organs. Party leadership is therefore not adequately represented as a wholly unwritten deviation from an otherwise unrelated constitutional structure. [Constitute Project](https://www.constituteproject.org/constitution/China_2018)

**TCE stress test:** trace who influences careers and who controls organizational resources, not just who formally signs appointments. Allow governing-coalition bargaining and personal concentration of power rather than assuming a party-state is a single actor. [Illinois Experts](https://experts.illinois.edu/en/publications/power-sharing-and-leadership-dynamics-in-authoritarian-regimes/)

### 4.12 Oromo Gadaa: an additional African coverage test

**Composition.** Generation classes with periodic transfer of political responsibilities, assemblies, and accountability practices. The literature describes an eight-year transfer cycle, with important regional variation in grades and organization. [Accountability Research Center](https://accountabilityresearch.org/the-oromo-gadaa-system-and-its-accountability-values/)

**TCE stress test:** representation and succession must support generation-class membership and rotation. “Age-based” institutions should not be reduced to simply choosing the oldest available individual.

---

## 5. Parameters: documented values versus modeling assumptions

### 5.1 Historical configuration parameters

**Confidence:** **H** = strongly documented for the specified rule or case; **M** = reconstructed or dependent on particular source classifications. Confidence in a formal rule does not imply confidence that compliance was universal.

| Parameter | Value and unit | Scope and implementation meaning | Confidence/source |
| --- | --- | --- | --- |
| Athenian council size | **500 seats; 50 per tribe across 10 tribes** | Constitutional allocation, not a universal council-to-population ratio | H; *Athenaion Politeia*. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.2.2.html) |
| Athenian assembly schedule | **4 meetings per prytany; 10 prytanies**, hence **40 scheduled meetings/year** | Scheduled opportunities, not attendance or proof every meeting occurred | H for prescribed schedule. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.2.2.html) |
| Athenian jury eligibility | **30 years minimum age** | Juror eligibility differs from other civic statuses | H. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.3.3.html) |
| Venetian final electoral college | **41 persons** | Final stage of a multistage procedure | H. [ShiftLeft](https://shiftleft.com/mirrors/www.hpl.hp.com/techreports/2007/HPL-2007-28R1.pdf) |
| Traditional Haudenosaunee chiefly positions | **50 positions** associated with the original five nations | Named institutional seats, not 50 interchangeable legislators | H for documented traditional structure; historical continuity more qualified. [National Museum of the American Indian](https://americanindian.si.edu/sites/1/files/pdf/education/HaudenosauneeGuide.pdf) |
| Haudenosaunee chiefly tenure | **Life, subject to removal** | Model conditional tenure, not an extremely long fixed term | H for documented traditional rule. [National Museum of the American Indian](https://americanindian.si.edu/sites/1/files/pdf/education/HaudenosauneeGuide.pdf) |
| Song *yin*-route recruitment, 1213 roster | **37% of executory-class officials; 53% of administrative-class officials** | Shares of specified categories, not all Chinese officials across the dynasty | M; Lo’s reconstruction. [dokumen.pub](https://dokumen.pub/an-introduction-to-the-civil-service-of-sung-china-with-emphasis-on-its-personnel-administration-9780824888084.html) |
| Mongol decimal organization | Nominal units of **10, 100, 1,000, 10,000** | Organizational categories, not guaranteed actual strengths | H for formal pattern. [AFE East Asia](https://afe.easia.columbia.edu/mongols/main/transcript.pdf) |
| Imperial electoral college, 1356 | **7 electors; majority = 4** | Specific medieval electoral constitution | H. [Internet History Sourcebooks](https://sourcebooks.fordham.edu/source/goldenbull.asp) |
| Oromo Gadaa transfer cycle | **8 years** | Rotation of governing generation classes; regional qualification required | H for cycle, M for transferring details between communities. [Accountability Research Center](https://accountabilityresearch.org/the-oromo-gadaa-system-and-its-accountability-values/) |
| German constructive no-confidence delay | **At least 48 hours** | Procedural minimum before election of successor | H. [Gesetze im Internet](https://www.gesetze-im-internet.de/englisch_gg/englisch_gg.html) |
| U.S. office terms | President **4 years**; representative **2**; senator **6** | Distinct clocks within one government | H. [National Archives](https://www.archives.gov/founding-docs/constitution-transcript) |

**Do not convert these examples into universal priors.** A range of one day to life for different offices is evidence that tenure is highly role-specific, not a useful random distribution from which to draw every office.

### 5.2 Behavioral parameters that require calibration

The following are **proposed sensitivity settings**, not historical estimates.

| Parameter | Domain or initial test settings | Unit | Calibration status |
| --- | --- | --- | --- |
| Perceived detection probability | Domain **0–1**; test **0.1, 0.5, 0.9** | Probability per relevant violation | Uncalibrated; derive from monitoring and experience |
| Perceived enforcement conditional on detection | Domain **0–1**; test **0.1, 0.5, 0.9** | Conditional probability | Uncalibrated; keep separate from detection |
| Belief update rate | Test **0.02, 0.10, 0.30** | Fraction of prediction error incorporated per observation | Engineering sensitivity grid |
| Routine political reconsideration interval | Test **7, 30, 90 days**, plus event-triggered updates | Simulation days | Scheduling experiment, not a claim about historical cognition |
| Administrative throughput | Staff-days available ÷ staff-days per case | Cases per period | Measure from modeled tasks and travel; do not assign a universal “bureaucracy efficiency” |

Whenever a quantity can be derived from actual people, resources, distances, and tasks, prefer derivation over a government-wide scalar.

---

## 6. Informal institutions and divergence from formal rules

Maintain three distinct representations:

| Representation | Example |
| --- | --- |
| **Promulgated or otherwise authoritative rule** | A governor must remit an assessed revenue |
| **Agents’ beliefs and shared expectations** | Local officials expect the governor to retain a customary portion |
| **Observed behavior** | What was actually collected, retained, transferred, concealed, or punished |

An oral rule is not automatically informal. Conversely, a written instruction does not become effective merely because the engine has stored it.

Helmke and Levitsky distinguish four relationships between informal and formal institutions. Their classification depends on whether formal institutions are effective and whether informal outcomes converge with their aims. [Weatherhead Center](https://www.wcfia.harvard.edu/sites/g/files/omnuum8891/files/wcfia/files/883_informal-institutions.pdf)

| Relationship | Illustrative TCE implementation |
| --- | --- |
| **Complementary** | Unwritten bargaining conventions help a functioning council reach agreements |
| **Accommodating** | Actors follow the letter of effective rules while using conventions to alter their practical consequences |
| **Substitutive** | Community mediation performs a function that an ineffective official court does not provide |
| **Competing** | Patronage obligations or protection arrangements contradict official appointment or enforcement rules |

These examples are suggested implementations, not claims that every council, community mediator, or patronage network fits one fixed category.

**Important boundary:** a single bribe is behavior. A widely understood expectation that particular payments purchase particular official favors can become an informal institution.

TCE should also distinguish **formal privilege** from corruption. Song *yin* recruitment is a useful warning: a family advantage can be explicitly authorized rather than an illicit violation of a meritocratic rule. [dokumen.pub](https://dokumen.pub/an-introduction-to-the-civil-service-of-sung-china-with-emphasis-on-its-personnel-administration-9780824888084.html)

---

## 7. Variation across subsistence systems, eras, and regions

Treat the following as requirements for model coverage, not a mandatory sequence of development.

| Context | What TCE must allow | What it must not assume |
| --- | --- | --- |
| **Foraging societies** | Situational leadership, collective decision-making, reputation, kinship, and potentially durable inequality | That all foragers were uniformly mobile, egalitarian, or institutionally simple; that model has been explicitly challenged. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S1090513822000447) |
| **Early farming communities** | Local councils, household authority, collective obligations, and emerging offices without a complete state apparatus | That agriculture automatically creates kingship, bureaucracy, or a particular population threshold |
| **Pre-industrial agrarian societies** | Dynastic and elective rulers, corporate privileges, oral and written law, layered jurisdictions, constrained fiscal reach | That “premodern” means one uniform feudal pyramid; the Song, Haudenosaunee, and imperial-electoral cases already contradict such a template. [dokumen.pub](https://dokumen.pub/an-introduction-to-the-civil-service-of-sung-china-with-emphasis-on-its-personnel-administration-9780824888084.html) |
| **Industrial settings** | Large administrative workloads, organized constituencies, mass information flows, and occupational political interests | That industrialization should directly set a democracy variable |
| **Modern settings** | Differentiated state organizations, parties, judicial and administrative procedures, federal or unitary arrangements | That similar constitutional organs imply similar practical distributions of power. [Constitute Project](https://www.constituteproject.org/constitution/China_2018) |

For TCE, changes in transport, recordkeeping, education, settlement scale, and organizational resources should alter the **cost and reach of institutional arrangements**, not unlock predetermined regime stages.

The requested cases are a strong first test suite, but broader coverage should also deliberately include South Asian, Islamic, Andean, and Pacific institutions. Treat those as additional validation work rather than claiming that a Europe-heavy collection plus a few exceptions establishes universality.

---

## 8. Stylized facts a correct simulation should reproduce

### 8.1 Formal complexity and actual power concentration are different dimensions

A government can retain many offices and collective bodies while practical authority concentrates around one person. The Augustan case supplies a particularly clear historical test. [Internet Classics Archive](https://classics.mit.edu/Tacitus/annals.1.i.html)

**Validation:** holding the formal institutional graph largely constant, changes in appointments, resource control, and patronage should be capable of changing effective political power.

### 8.2 Selection mechanisms commonly coexist

Athens combined lot and election; Song administration combined examinations with protected entry routes; Venice combined lotteries and repeated elections. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.3.3.html)

**Validation:** do not force a polity to possess one government-wide selection method.

### 8.3 Accountability depends on detection and implementation

The Indonesian audit experiment’s approximately **eight-percentage-point reduction in missing expenditures** demonstrates that changing monitoring can matter even without changing the underlying prohibition on misuse. It does not imply that every audit program, in every society, produces the same response. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/pdf/10.1086/517935)

**Validation:** additional monitoring should sometimes improve compliance, but its effect should depend on evidence, sanctions, capture, and opportunities for substitution.

### 8.4 Extra-legal transfers can fail

Powell and Thyne’s original dataset reports **457 coup attempts during 1950–2010**, of which **227 succeeded—49.7%**. This is a descriptive result for a defined historical sample, not the probability that any arbitrary coup or government transition succeeds. [ResearchGate](https://www.researchgate.net/publication/227574729_Global_instances_of_coups_from_1950_to_2010_A_new_dataset)

**Validation:** an attempt to seize government must be distinct from a successful transfer. Defection, recognition, and control of organizational resources should determine outcomes.

### 8.5 Administrative dimensions often grow together, without requiring one institutional form

A Seshat study examined **414 polities in 30 geographic areas**, using **51 variables grouped into nine complexity characteristics**; a first principal component accounted for roughly three-quarters of the variation. That is evidence of broad co-variation among scale and organizational features, not a law that all societies follow one political sequence. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5777031/)

**Validation:** larger, more administratively demanding polities should often develop correlated organizational capacities, while retaining multiple possible distributions of authority.

### 8.6 Formal tenure is not realized tenure

Life appointments can terminate through authorized removal; fixed terms can end through death or other procedures; succession can remain disputed. The Haudenosaunee life-but-removable formulation illustrates why tenure cannot be represented by a duration alone. [National Museum of the American Indian](https://americanindian.si.edu/sites/1/files/pdf/education/HaudenosauneeGuide.pdf)

**Validation:** store tenure conditions and exit processes, not just `term_length_days`.

---

## 9. Recommended TCE architecture

### 9.1 Separate the institutional interpreter from the physical simulation

Use two interacting systems:

**Institutional interpreter:** determines which claims, obligations, permissions, and status changes follow under a specified body of recognized rules.

**Agent/world simulation:** determines what people actually say, believe, sign, deliver, seize, refuse, or enforce.

This distinction allows an unlawful detention to occur physically while producing legal claims against its perpetrators. It also allows a legally valid appointment to fail to secure obedience.

An illustrative record—not a proposed final serialization format—is:

```
InstitutionalRule
    kind: regulative | constitutive
    participants: typed selectors
    condition: predicate expression
    jurisdiction: scope reference
    statement: permission | obligation | prohibition | counts-as
    procedure: optional procedure reference
    consequences: optional consequence procedure
    authority_claim: reference
    valid_interval: interval
    precedence_and_exceptions: references
    evidence_and_confidence: metadata
```

Procedures need explicit states for **pending, contested, failed, withdrawn, certified, and implemented**. Certification and implementation should not be synonymous.

### 9.2 Agent state

Each person needs only relevant political state: memberships, offices, qualifications, kin and patron relationships, known claims, political preferences, commitments, obligations, resources, and a limited history of observations.

Avoid an all-persons-by-all-rules belief matrix. Most people need detailed beliefs about nearby authorities and relationships, not every provision in every polity.

### 9.3 Runtime strategy for 10k–50k agents

Use event-driven updates and indexed dependencies.

A death should invalidate affected offices, inheritance claims, obligations, and patron ties. A new tax rule should notify relevant administrators and taxpayers through modeled communication. An election should enumerate its constituency when needed, rather than reevaluating every person against every political rule every day.

Compile authored predicates and procedure graphs into deterministic runtime structures. Keep stable identifiers and an event log so institutional changes can be inspected and replayed.

This is an architectural recommendation; no performance benchmark for the proposed design has been run here.

### 9.4 What to simplify

Represent political speech initially as proposals, amendments, endorsements, objections, promises, and bargaining offers. Represent routine adjudication through evidence, claims, applicable rules, and decision procedures rather than unrestricted natural-language legal reasoning.

Simplify low-salience administration, but preserve the identity of actors at consequential boundaries: who appointed someone, who approved spending, who withheld information, and who enforced an order.

Do not mechanically scale historical institutions by population. A 500-seat council is not automatically meaningful in a polity of 700 people. Separate fixed institutional slots from representational quotas and feasibility constraints.

### 9.5 Existing models and games worth borrowing from

| Model or game | Useful contribution | What not to assume |
| --- | --- | --- |
| **MAIA — Ghorbani et al., 2013** | Explicit integration of agents, roles, institutions, physical resources, and operational interaction; agents may disregard institutional rules | A conceptual modeling framework is not itself a validated universal government simulator. [JASSS](https://www.jasss.org/16/2/9.html) |
| **nADICO — Frantz et al., 2013** | Nested institutional statements and consequences | A richer grammar alone supplies neither historical content nor realistic agent behavior. [Springer](https://link.springer.com/chapter/10.1007/978-3-642-44927-7_31) |
| **Crusader Kings III** | Character-centered dynastic continuity and political relationships | Its dynastic game structure should not become TCE’s universal political ontology. [Steam Store](https://store.steampowered.com/app/1158310/Crusader_Kings_III/) |
| **Victoria 3** | Population interests, economic conditions, and government reform in one system | Its player-directed framework and historical scope differ from autonomous institutional invention. [Steam Store](https://store.steampowered.com/app/529340/Victoria_3/) |
| **Eco** | Programmable laws proposed and voted on by participants, linked to economic and ecological consequences | Human players supply much of the bargaining and institutional understanding that TCE’s agents must generate. [Steam Store](https://store.steampowered.com/app/382310/Eco/) |

The closest conceptual foundation is **MAIA plus an extended institutional grammar**, supplemented by TCE’s individual-level economy, relationships, and physical enforcement.

---

## 10. Pitfalls and validation priorities

**Syntax is not causality.** Encoding “the council controls taxation” does not explain why the ruler respects it. Supply the dependencies that make cooperation valuable or resistance credible.

**Constitutions are not complete specifications.** Customs, interpretation, precedent, selective enforcement, and ambiguous language matter. Keep interpretive decisions attributable to actors rather than silently resolved by an omniscient engine.

**Do not end enforcement in an infinite chain of rules.** Eventually, an official or group must decide whether to carry out a sanction. That decision depends on incentives, commitments, beliefs, and resources.

**Do not confuse documentary precision with behavioral certainty.** A surviving law may specify an exact threshold while evidence for attendance, bribery, coercion, or compliance remains thin. Ancient political narratives also have argumentative purposes; modern official descriptions primarily establish formal arrangements.

**Do not assume every institution is state-owned.** Clans, religious organizations, guilds, parties, military households, and other bodies can create authority and obligations that intersect with government.

**Do not prohibit historically dysfunctional combinations.** Contradictory jurisdictions, unfunded mandates, deadlocked procedures, and incompatible claims are often the phenomena the simulation needs to generate. Flag them for authors; do not automatically repair them.

The most valuable initial tests are not whether the UI displays the correct regime label. They are whether the engine can handle:

* A child inheriting a title while a regent controls its powers; rival claimants receiving different groups’ recognition.
* A legally selected official who cannot obtain records, revenue, or obedience; an unauthorized commander who can.
* An empty candidate pool, disputed membership, failed quorum, tied vote, blocked succession, or conflicting jurisdictions without inventing an automatic winner.
* A constitutional amendment that changes formal rules but produces delayed, partial, or resisted implementation.

**Bottom line:** author governments as **who may do what, to whom, through which procedure, within which jurisdiction**. Then simulate **who recognizes that authority, who supplies its resources, who obeys, and who can change the arrangement**. That separation is what lets TCE produce governments rather than merely assign them.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928b3-a6e8-83ea-8fbb-2f4a9a44fb2e)
