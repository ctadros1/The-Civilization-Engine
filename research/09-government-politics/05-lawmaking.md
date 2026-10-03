# How laws are made: a simulation-ready model for TCE

**The strongest design is a constitution-configured procedure operating on actual people—not a government-type modifier to a law’s probability of passing.** Separate the ability to propose a measure, secure agenda time, change its text, authorize it, authenticate it, bring it into force, and communicate it.

A useful default sequence is:

**Issue → proposal → agenda admission → deliberation and amendment → authorization → promulgation → commencement → implementation.**

But this must be a configurable graph, not a universal linear pipeline. A decree can bypass an assembly; an emergency measure can operate before legislative ratification; a council can return a proposal for revision; and a customary rule can acquire authority without a single enactment event. The historical cases below require these distinctions. [Pure](https://www.pure.ed.ac.uk/ws/files/20012342/OHAGL_Canevaro_4.pdf)

The report distinguishes **documented institutional rules**, **observed outcomes in particular datasets**, and **proposed simulation assumptions**. The evidence is much stronger for who formally possessed a power than for universal estimates of drafting time, persuasion rates, or ancient legislative output.

---

## 1. Mechanisms: implementable rules of lawmaking

### 1.1 Separate problems, proposals, and authoritative rules

For TCE, an economic or social problem should not immediately create a bill. Give it several possible responses: private adjustment, litigation, petitioning, customary agreement, administrative action, or legislation.

Represent three different objects:

| Object | Meaning | Example |
| --- | --- | --- |
| **Issue** | A condition attracting concern | Downstream households receive insufficient irrigation water. |
| **Proposal** | A particular actor’s proposed response | Establish a water-allocation rotation and appoint inspectors. |
| **Authoritative rule** | A prescription recognized through an accepted source of authority | The assembly’s enacted allocation rule, a ruler’s order, or a recognized local custom. |

A useful conceptual foundation is Crawford and Ostrom’s institutional grammar: distinguish the actors addressed, prescription, action, conditions, and possible sanction. But legislative procedure also requires **constitutive rules** specifying which offices and acts can create other rules. A sanction-bearing prohibition and a rule empowering a council are not interchangeable objects. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/grammar-of-institutions/7D37CD3BC5ED2D9FD57D2EE292958F47)

**Implementation rule:** an issue generates proposals only when someone has sufficient motivation, information, resources, and access to an authorized sponsor. Unaddressed issues should remain possible even when many inhabitants dislike the status quo.

### 1.2 Proposal rights and agenda rights are different powers

Model separately the rights to:

* Petition or submit information.
* Introduce a formal proposal.
* Schedule it for deliberation or decision.

Someone can possess the first two without the third. This makes **nondecision** an important political outcome: a proposal may never encounter an explicit rejecting vote.

Legislative bargaining models show why recognition and proposal rules matter: the actor allowed to present an offer can influence which bargain is available. An empirical investigation of US transportation spending found support for the qualitative importance of committee proposal power, while some precise quantitative predictions performed less well. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/bargaining-in-legislatures/2636DB331955032B9421DB0EB25467CC)

**Implementation rule:** give agenda controllers a queue-selection policy based on their preferences, procedural obligations, coalition commitments, urgency, and expected success. Allow mandatory scheduling rules, petition thresholds, deadlines, and minority agenda rights to constrain that discretion.

A powerful institution may therefore exercise influence through **what never reaches a vote**, not merely through recorded votes.

### 1.3 Deliberation should change information, proposals, and relationships separately

For simulation purposes, distinguish three processes:

**Information exchange** changes beliefs about consequences. An engineer explains that a proposed canal cannot carry the promised volume.

**Bargaining** changes the offer. Upstream households accept a rotation in exchange for help maintaining their channels.

**Social pressure or coercion** changes willingness to speak, attend, or oppose. A dependent tenant may privately dislike a measure while publicly accepting it.

These should not become one generic “persuasion” variable. Parliamentary bargaining research provides a basis for modeling proposal revision, coalition formation, and procedural advantage, but it does not supply universal coefficients for converting speeches into opinion change. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/bargaining-in-legislatures/2636DB331955032B9421DB0EB25467CC)

**Implementation recommendation:** evaluate proposals using expected material effects, values, obligations to allies or patrons, procedural legitimacy, and anticipated risks. Let speeches carry bounded pieces of information or commitments rather than directly adding approval points.

Also distinguish an **advisory council** from a **consenting council**. Advisory objections can reduce confidence, legitimacy, or implementation effort without creating a legal veto.

### 1.4 Attendance and voting require explicit aggregation rules

Do not model “the assembly” as every eligible person appearing automatically.

For TCE, participation should depend on travel, work schedules, weather, safety, compensation, mobilization, and issue importance. The body’s constitutional rules then determine how those who participate affect the outcome.

Each decision rule needs at least:

| Field | Necessary distinction |
| --- | --- |
| Eligibility | Who may attend, speak, propose, amend, or vote? |
| Quorum | How many must be present for valid business? |
| Denominator | All members, those present, or those present and voting? |
| Aggregation | Individual votes, territorial units, estates, clans, or successive benches? |
| Threshold | Plurality, majority, supermajority, or a specified consensus procedure? |
| Abstention and ties | Do abstentions affect passage? Who resolves a tie? |
| Visibility | Open declaration, secret ballot, acclamation, or recorded division? |

These distinctions are not cosmetic. Roman tribal voting aggregated through tribes, while Haudenosaunee deliberation proceeds through structured group consultations. Neither is accurately represented by counting every participant’s vote in one undifferentiated pool. [Springer](https://link.springer.com/article/10.1007/s10602-026-09503-9)

**Consensus should mean a specified collective acceptance procedure, not identical private preferences.** An agent may support, tolerate, abstain from obstructing, or actively object. Which of those states blocks a decision must depend on the institution.

### 1.5 Model veto players as required consents—not as a count of institutions

Tsebelis’s central distinction is between actors whose agreement is necessary to change the status quo and actors who merely influence the process. Policy stability depends on the configuration of these actors, including their preferences and internal organization. Additional veto players can be redundant or “absorbed” rather than adding an independent constraint. [LSA Technology Services](https://sites.lsa.umich.edu/tsebelis/wp-content/uploads/sites/246/2015/03/decision_making_in_political_systems_1995_bjps.pdf)

For TCE, distinguish:

**Legal vetoes:** a required chamber, ruler, electorate, or council can refuse authorization.

**Political vetoes:** an actor can credibly destroy the governing coalition, withhold essential implementation, or otherwise make the proposal untenable.

**Delay powers:** an actor can postpone action without permanently preventing it.

A minister threatening resignation is not automatically an indispensable veto player: that depends on whether the government can replace the minister or coalition partner.

A useful **modeling formulation**, rather than an empirical equation, is:

\[
F\_r(q)=
\left\{x:
x\text{ is admissible under route }r
\quad\text{and every required decision gate approves }x
\right\}
\]

Here, \(q\) is the policy that results from inaction, and \(x\) is a proposed alternative. With several legally available routes:

\[
F(q)=\bigcup\_r F\_r(q).
\]

An ordinary assent route and a veto-override route therefore have different gates. A constitutionally authorized decree may be another route, but only for permitted subjects and circumstances.

**Important consequence:** disagreement does not always imply gridlock. Two actors with different ideal policies may both prefer the same compromise to the existing policy. Conversely, a status quo located between their preferences may be difficult to change.

Tsebelis’s empirical study of important labor legislation in 15 Western European countries during 1981–1991 supports the relevance of veto-player configurations, but it should not be converted into a universal percentage penalty per additional institution. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/veto-players-and-law-production-in-parliamentary-democracies-an-empirical-analysis/4B60A0AE4081473F82B0C41D512A397D)

### 1.6 Separate administrative congestion from political gridlock

A bill can wait because nobody will approve it, because nobody will schedule it, or because the institution lacks time to complete the work. Those are different mechanisms.

For each drafting or deliberative resource, maintain a workload queue:

\[
Q\_{t+1}=\max(0,\;Q\_t+W\_t-H\_t),
\]

where \(Q\) is outstanding work, \(W\) is newly admitted work, and \(H\) is completed work, measured in compatible units such as staff-hours.

Keep at least **drafting capacity** and **collective meeting time** separate. A polity can have abundant scribes but few assembly days, or frequent meetings but little technical competence.

This yields different remedies:

| Bottleneck | Potential remedy |
| --- | --- |
| Agenda exclusion | Replace the gatekeeper, create mandatory scheduling, or change priorities. |
| Incompatible required consents | Revise the proposal, change the coalition, or change the procedure. |
| Drafting congestion | Add competent staff or simplify the measure. |
| Meeting congestion | Add sittings, delegate routine decisions, or prioritize business. |
| Implementation incapacity | Fund personnel, transport, records, and enforcement. |

For measurement, distinguish **bill passage rates** from **unresolved policy demand**. Binder’s gridlock research used salient agenda issues rather than treating every introduced bill as an equally meaningful demand. Subsequent reanalysis showed that some substantive conclusions depend on how legislators’ preferences are made comparable across chambers and time. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/dynamics-of-legislative-gridlock-194796/ED0A880C049DB34F2BDFA9162A72011C)

**TCE recommendation:** record both event-level delays and the fraction of a predefined cohort of salient issues that remains unresolved. Do not call every period with few new laws a crisis.

### 1.7 Promulgation, commencement, knowledge, and enforcement are separate

A law needs an authoritative identity: which text or oral formulation was approved, by whom, under which procedure?

It then needs a commencement rule. South Africa’s Constitution illustrates the distinction particularly clearly: presidential assent and signature make a bill an Act; publication must follow promptly; commencement occurs upon publication or on a date specified by the Act. [Justice](https://www.justice.gov.za/constitution/chp04.html)

Finally, people must learn about it. Icelandic law-speaking demonstrates that public oral transmission can itself be an institutional responsibility, rather than an imperfect substitute for a modern gazette. [Þjóðgarðurinn á Þingvöllum](https://www.thingvellir.is/en/education/history/loegberg/)

**Implementation recommendation:** store separate states for:

`authorized → authenticated → legally effective → communicated → operationally implemented`

These states need not advance simultaneously or uniformly across territory. A remote official may enforce an outdated version. A merchant may hear about a change before a farmer. A law may be legally effective despite widespread ignorance; whether ignorance excuses a violation is another legal rule.

Do not allow the engine’s immediate access to the new rule to give every agent immediate knowledge of it.

---

## 2. Parameters: empirical anchors and explicit design assumptions

### Confidence notation

**H:** directly documented formal rule or official count.  
**M:** scholarly reconstruction or a result from a particular empirical study.  
**D:** proposed design/testing assumption, not a historical estimate.

High confidence in a written rule does **not** establish perfect historical compliance.

### 2.1 Institutional parameters with defensible historical or contemporary values

| Context | Parameter and value | Units and interpretation | Confidence and source |
| --- | --- | --- | --- |
| Late-fourth-century Athens | Council of **500**; **50** from each of ten tribes | Council members; selection by lot | **H**, Aristotle, *Athenaion Politeia* 43. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.2.2.html) |
| Same | **4 assemblies per prytany**, giving **40 scheduled assemblies annually** | Meetings, not laws enacted | **H** for the described calendar. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.2.2.html) |
| Roman tribal assembly | **35 tribes**; a majority is **18 unit votes** | Tribal votes, not equal-weight individual votes across the entire electorate | **H/M** for structure and reconstruction. [Springer](https://link.springer.com/article/10.1007/s10602-026-09503-9) |
| Late Republican Rome | Notice associated with **three market days** | A calendar-relative requirement; conversion into a fixed modern day count is disputed | **M**; Lintott’s studies of *trinundinum* and *nundinae*. [Cambridge University Press](https://www.cambridge.org/core/journals/classical-quarterly/article/abs/trinundinum/D309663E4A51A4C8B8027BC93DD57549) |
| Icelandic Commonwealth | National assembly held **annually**; lawspeaker chosen for **3 years** | Meeting frequency and office term | **H/M**. [Library of Congress Blogs](https://blogs.loc.gov/law/2016/05/thingvellir-northern-europes-first-parliament/) |
| Icelandic oral publication | Complete law recitation over **3 years**; procedural provisions repeated annually | Publication cycle, not the duration needed to enact each law | **H/M**. [Þjóðgarðurinn á Þingvöllum](https://www.thingvellir.is/en/education/history/loegberg/) |
| Haudenosaunee original five-nation structure | **50 chiefships**, organized into **3 deliberative benches** | Offices and stages of collective consultation; not a simple 50-person majority chamber | **H** for the structure described by Onondaga Nation; historical practice requires contextual interpretation. [Onondaga Nation](https://www.onondaganation.org/government/) |
| Tenth-century Uttaramerur, South India | **30 wards**; specified committees serve **360 days** | Territorial nomination structure and committee term, not universal adult representation | **H/M**, inscription in an older published translation. [GeoCities](https://www.geocities.ws/ifihhome/articles/uttaramerur.html) |
| Ordinary UK bill procedure | **5 principal stages in each chamber**, including **3 readings** | Procedural stages; not five votes or five fixed-duration sessions | **H**; exceptional routes exist. [GOV.UK](https://www.gov.uk/government/publications/guide-to-making-legislation/guide-to-making-legislation-html--2) |
| China’s NPC Standing Committee | Generally **3 deliberations**; bills circulated **7 days** before the session | Normal procedure with shorter and longer exceptions | **H**, official January 2026 explanation. [NPC](https://en.npc.gov.cn.cdurl.cn/2026-01/14/c_1155040.htm?utm_source=chatgpt.com) |
| US Senate, ordinary legislation | Cloture generally requires **3/5 of senators chosen and sworn**, normally **60 of 100** | Ending debate; **not** the ordinary final-passage threshold | **H**. [U.S. Senate](https://www.senate.gov/about/powers-procedures/filibusters-cloture.htm) |
| Brazil, provisional measures | Initial validity **60 days**, extendable once for another **60**; clock pauses during congressional recess | Constitutional deadline for an immediately effective measure awaiting legislative approval | **H**. [Portal da Câmara dos Deputados](https://www2.camara.leg.br/comunicacao/assessoria-de-imprensa/guia-para-jornalistas/medida-provisoria?utm_source=chatgpt.com) |
| Switzerland, optional federal referendum | **50,000 signatures within 100 days**, or a request by **8 cantons** | Trigger for a referendum on an eligible parliamentary enactment | **H**. [CH Info](https://www.ch-info.swiss/en/direkte-demokratie/abstimmungen) |
| Switzerland, constitutional popular initiative | **100,000 signatures within 18 months** | Proposal channel for constitutional change; approval requires popular and cantonal majorities | **H**. [CH Info](https://www.ch-info.swiss/en/direkte-demokratie/abstimmungen) |

These are **case configurations**, not values to average into a universal constitution.

### 2.2 Observed output and delay benchmarks

| Observation | Quantitative result | Correct use in TCE | Confidence and source |
| --- | --- | --- | --- |
| Chinese legislative delay in Truex’s study | **48%** of laws missed their legislative-plan period; about **12%** took **more than 10 years** to pass | Validate the possibility of long delays under authoritarian rule; do not treat these as timeless Chinese probabilities | **M**, context-specific research dataset. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/0010414018758766) |
| US 117th versus 118th Congress | **362 versus 274 public laws**, respectively | Closed two-year legislative cohorts show substantial output variation; the latter count is about **24.3% lower**, calculated from the official totals | **H** for counts; these are not measures of policy importance. [Congressional Budget Office](https://www.cbo.gov/publication/59112) |
| Swiss optional referendums through December 31, 2025 | **91 of 219** challenged proposals rejected: approximately **41.6%** | Conditional rejection among measures reaching this referendum route—not rejection of 41.6% of all legislation | **H** for official counts; percentage calculated. [CH Info](https://www.ch-info.swiss/en/direkte-demokratie/abstimmungen) |

The US counts attach to congressional cohorts, including enactments signed shortly after the relevant Congress ended; they should not be mistaken for calendar-year totals.

### 2.3 Starting assumptions where history does not provide general estimates

The following are **engineering test ranges**, not empirical confidence intervals.

| TCE parameter | Suggested initial exploration | Units | Status |
| --- | --- | --- | --- |
| Small deliberative council | **5–15**, with sensitivity tests from **3–30** | Members | **D** |
| Active issues receiving serious attention per small institution | **3–10** | Issues concurrently worked on | **D** |
| Drafting effort, simple measure | **1–10** | Person-days of competent drafting or equivalent preparation | **D** |
| Drafting effort, complex multi-clause measure | **10–100** | Person-days | **D** |
| Candidate compromises considered per bargaining round | **3–8** | Alternative packages | **D**, computational bound |
| Broad policy-preference dimensions | **6–12**, supplemented by clause-specific material effects | Dimensions | **D**, representation choice |

These assumptions should be varied substantially during testing. Derive meeting intervals, travel delays, and participation costs from TCE’s actual calendars, geography, livelihoods, and infrastructure rather than assigning each era a universal number.

**Do not invent a universal “laws per thousand inhabitants per year” parameter.** A village’s customary adjustment, a royal administrative instruction, and a modern omnibus statute are not commensurable units.

---

## 3. Variation across eras and world regions

### 3.1 Foragers and early farming communities: rules without legislative sessions

Wiessner’s research among the Ju/’hoansi examines norm enforcement through conversation, criticism, ridicule, and social sanctions. This supports a mechanism in which expectations are maintained and contested through repeated interaction without requiring a formally introduced bill or sovereign promulgation. It is evidence about a particular society, not proof that all foragers had the same political organization. [Arizona State University](https://asu.elsevierpure.com/en/publications/norm-enforcement-among-the-juhoansi-bushmen-a-case-of-strong-reci/)

**TCE representation:** allow a customary prescription to accumulate recognition through repeated application, dispute settlement, and endorsement by influential participants. Recognition can be uneven, contested, and reversible.

For the earliest farming villages, the evidence assembled here does not justify precise frequencies of legislative meetings, majority thresholds, or universal household voting. Treat councils, household bargaining, ritual authority, and stronger personal leadership as alternative configurations to investigate—not mandatory steps in an evolutionary ladder.

In early Near Eastern states, royal normative acts coexisted with other legal materials and practices. Scholarship cautions against imposing modern classifications too neatly: an administrative act could contain rules that modern analysts would label civil or criminal law. [University of Lodz Journals](https://czasopisma.uni.lodz.pl/Iuridica/article/view/17777)

**TCE implication:** writing should enable recordkeeping, copying, comparison, and authoritative versions. It should not automatically turn all norms into a comprehensive statutory code.

### 3.2 Athens: popular participation with procedural differentiation

Fourth-century Athenian lawmaking distinguished ordinary assembly decrees from the making or alteration of general laws. Canevaro reconstructs a sequence involving preliminary authorization, publication of proposals, repeated public reading, and consideration by *nomothetai*, with attention to conflicting existing laws. The composition and precise institutional identity of the *nomothetai* remain disputed. [Pure](https://www.pure.ed.ac.uk/ws/files/20012342/OHAGL_Canevaro_4.pdf)

The council’s agenda preparation and the assembly calendar were themselves structured institutions, not spontaneous gatherings of the whole population whenever someone wanted a rule. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.2.2.html)

**TCE implication:** one polity needs different routes for different kinds of enactment. “Direct democracy” is not a single unrestricted majority-vote operation.

### 3.3 Roman Republic: office-controlled proposals and grouped popular votes

Roman citizens could accept or reject proposals, but authorized magistrates and tribunes controlled formal introduction. Public discussion did not give ordinary participants an unrestricted right to amend the proposal during the voting assembly. Tribal voting aggregated decisions through voting units. Tribunician intervention could obstruct proceedings before enactment; it should not be modeled simply as a modern president vetoing a completed statute. The Senate’s considerable influence likewise did not make it an ordinary second chamber required to pass identical text. [Springer](https://link.springer.com/article/10.1007/s10602-026-09503-9)

**TCE implication:** distinguish public discussion from the decision meeting, preserve proposer control over text where applicable, and implement grouped voting explicitly. An institution can be politically dominant without occupying a mandatory formal approval node.

### 3.4 Norse things: public gatherings with differentiated legal roles

The Icelandic Althing combined a broad gathering with more restricted institutional functions. The law council, rather than every person present acting as an equal legislator, held lawmaking responsibilities. The lawspeaker occupied a specialized office associated with legal knowledge and public declaration. [Library of Congress Blogs](https://blogs.loc.gov/law/2016/05/thingvellir-northern-europes-first-parliament/)

**TCE implication:** model the gathering place, deliberative body, adjudicative functions, and publication office separately. A large visible assembly can contain spectators, litigants, supporters, traders, and legislators with different rights.

The annual meeting cycle also creates a natural distinction between ordinary business awaiting the next gathering and actions that some other authority can take between meetings.

### 3.5 Imperial China: accumulated norms and bureaucratic processing

Research on Tang and Song legal materials describes interaction among codes, administrative regulations, and imperial edicts; Song practice included compiling edicts alongside inherited legal materials. That is a model of layered rule production, not a ruler replacing the entire legal system with each decision. [Institute of History and Philology](https://www1.ihp.sinica.edu.tw/en/Publications/LegalHistoryStudy/992/Article/306)

**TCE implication:** support several classes of enactments, compilation and consolidation work, and conflicts between older general rules and newer specific instructions. An imperial decision still requires people to formulate, transmit, interpret, and operationalize it.

Do not infer that formal supremacy makes bureaucratic information, interpretation, or compliance irrelevant.

### 3.6 South India: royal authority and local corporate rulemaking

The tenth-century Uttaramerur inscription records detailed arrangements for village committees, including restricted eligibility, selection procedures, terms, and accountability. Its enactment context combines a royal communication, local assembly action, official involvement, and an inscription. Eligibility was socially and economically restricted; this is not evidence for universal adult village democracy. [GeoCities](https://www.geocities.ws/ifihhome/articles/uttaramerur.html)

**TCE implication:** local bodies can possess meaningful rulemaking authority while operating within a larger monarchy. A constitution should be able to express royal authorization plus local procedural autonomy, rather than allocating all legislative power to either the center or the village.

### 3.7 Islamic and Ottoman settings: multiple producers of authoritative norms

Ottoman legal history requires distinguishing juristic interpretation, sovereign enactments, and the work of courts and administrative bodies. Atçıl describes the development of relations between sultans and scholars, including scholars’ integration into state structures. Tamdoğan’s research on eighteenth-century Ottoman records identifies several venues of legal authority and petitioning, with regional variation. [DOI](https://doi.org/10.1093/oso/9780192888341.003.0006)

**TCE implication:** some disputes over “making law” are actually disputes over interpretation, jurisdiction, or whether a ruler is entitled to legislate on that subject. Religious jurists should not automatically become either a modern legislative chamber or passive servants of executive will.

### 3.8 Haudenosaunee and African deliberative traditions

Onondaga Nation describes a structured process in which the Firekeepers introduce matters, Older and Younger Brothers deliberate through their respective groupings, differences are reconciled, and the matter returns to the Firekeepers. Clan mothers also have a role in selecting leaders. This is not well captured by either a single chief’s decree or a flat majority ballot. [Onondaga Nation](https://www.onondaganation.org/government/)

**TCE implication:** allow consensus within sub-bodies, sequential referral, and return-for-reconsideration rules.

Research discussing Tswana *kgotla* and southern African community politics shows how public deliberation can test and sustain chiefly authority rather than simply transmit decisions from a chief to subjects. These traditions are documented in historically specific and sometimes contemporary settings; they should not be projected unchanged into all precolonial Africa. [Cambridge University Press](https://www.cambridge.org/core/journals/africa/article/neotraditional-authority-contested-the-corporatization-of-tradition-and-the-quest-for-democracy-in-the-topnaar-traditional-authority-namibia/C1568071AEF8CDF173CFC84AFBA10FE5)

**TCE implication:** public acceptance can constrain a leader without every participant holding a formal individual veto.

### 3.9 Industrial-era transitions: changing who controls an existing procedure

Industrialization does not require replacing assemblies with one predetermined parliamentary form. Britain illustrates changes to representation within an existing legislative structure: the 1867 Reform Act approximately doubled the electorate in England and Wales from one million to two million men, while substantial exclusions remained. [Parliament UK News](https://www.parliament.uk/about/living-heritage/evolutionofparliament/houseofcommons/reformacts/from-the-parliamentary-collections/collections-reform-acts/great-reform-act111/?utm_source=chatgpt.com)

The 1832 reform struggle also illustrates an institutional escape from obstruction: the prospect of creating additional peers helped induce abstentions that allowed the bill through the Lords. The composition of a veto-holding body could itself become part of the bargain. [Parliament UK News](https://www.parliament.uk/about/living-heritage/evolutionofparliament/houseofcommons/reformacts/overview/reformact1832/?utm_source=chatgpt.com)

**TCE implication:** franchise, representation, office eligibility, and appointment powers should be amendable rules. Political change can occur through modifying who occupies a gate, not only by abolishing it.

### 3.10 Modern systems: different combinations of the same components

In an ordinary UK legislative route, bills proceed through readings, committee scrutiny, further amendment, and approval of matching text by both chambers before assent. Government control of legislative business is an important part of the process, separate from its ability to win a final vote. [GOV.UK](https://www.gov.uk/government/publications/guide-to-making-legislation/guide-to-making-legislation-html--2)

China’s NPC Standing Committee uses circulation, sponsor explanations, group deliberation, committee review, and repeated consideration. Its official description includes exceptions allowing either abbreviated or extended processing. One-party government does not eliminate a multi-stage legislative procedure. [NPC](https://en.npc.gov.cn.cdurl.cn/2026-01/14/c_1155040.htm?utm_source=chatgpt.com)

Brazil’s provisional measures demonstrate a different ordering: executive issuance produces immediate legal effects, followed by legislative consideration and possible amendment or rejection. Expiration also creates questions about the legal relationships formed while the measure operated. [Portal da Câmara dos Deputados](https://www2.camara.leg.br/comunicacao/assessoria-de-imprensa/guia-para-jornalistas/medida-provisoria?utm_source=chatgpt.com)

Switzerland adds citizen-triggered challenges and constitutional initiatives to representative lawmaking. The potential for a referendum can influence a proposal before any referendum actually occurs. [CH Info](https://www.ch-info.swiss/en/direkte-demokratie/abstimmungen)

**The general lesson is combinatorial:** decree authority, representative deliberation, popular ratification, and constitutional review can coexist in the same polity.

---

## 4. Stylized facts and validation targets

Use two kinds of validation: **historical case benchmarks** and **controlled counterfactual tests of the implemented mechanisms**.

| Pattern to reproduce | Validation target |
| --- | --- |
| **Formal access differs from actual influence.** | Hold preferences constant and change proposal or agenda rights. The available proposals and distribution of outcomes should change—not merely the final vote count. Legislative bargaining research provides the benchmark mechanism. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/bargaining-in-legislatures/2636DB331955032B9421DB0EB25467CC) |
| **Additional institutions do not always add independent vetoes.** | Adding an aligned or redundant approval gate may increase processing time without changing the set of acceptable policies. Divergent indispensable actors should constrain change more strongly. [LSA Technology Services](https://sites.lsa.umich.edu/tsebelis/wp-content/uploads/sites/246/2015/03/decision_making_in_political_systems_1995_bjps.pdf) |
| **Authoritarian systems can experience prolonged legislative delay.** | A calibrated Chinese case should permit missed planning windows and decade-long processing, rather than automatically granting the ruler rapid enactment of every desired policy. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/0010414018758766) |
| **Institutional calendars create waiting independently of disagreement.** | An annual assembly and a frequently meeting council should produce different delay distributions even with identical preferences and competent staff. The Icelandic and Athenian calendars provide contrasting configurations. [Library of Congress Blogs](https://blogs.loc.gov/law/2016/05/thingvellir-northern-europes-first-parliament/) |
| **Raw enactment counts vary and are not equivalent to policy accomplishment.** | Track laws, substantive clauses changed, issues resolved, and implementation separately. The US congressional counts are a volume benchmark, not a quality ranking. [Congressional Budget Office](https://www.cbo.gov/publication/59112) |
| **Referendum outcomes are selected, not random samples of all legislation.** | First simulate which measures attract a qualifying challenge; only then compare rejection among challenged measures with the Swiss benchmark. [CH Info](https://www.ch-info.swiss/en/direkte-demokratie/abstimmungen) |
| **Procedural thresholds apply to particular stages.** | A measure can possess a final-passage majority but lack the votes required to end debate. Do not substitute the Senate cloture threshold for its ordinary passage rule. [U.S. Senate](https://www.senate.gov/about/powers-procedures/filibusters-cloture.htm) |
| **Legal validity and practical operation can diverge.** | A newly effective rule need not be universally known or fully implemented. Publication and commencement must remain separately testable events. [Justice](https://www.justice.gov.za/constitution/chp04.html) |

Additional **model-consistency tests** should verify that adding scribes relieves drafting congestion but does not magically reconcile incompatible preferences; that changing the status quo can unlock a previously impossible compromise; and that an expiring emergency measure makes inaction consequential.

Avoid calibrating to one headline “average time to pass a law.” Measure the distribution of time spent at each stage, including proposals that never reach enactment.

---

## 5. Modeling recommendation for individual agents and institutions

### 5.1 Use a small set of versioned objects

| Object | Essential contents |
| --- | --- |
| **Norm or law** | Scope, affected actors, permissions/obligations/prohibitions, sanctions where relevant, authority, priority, commencement, expiry, and version history |
| **Proposal** | Sponsor, issue, clauses, amendments, expected consequences, current procedural state, approvals, and deadlines |
| **Institutional body** | Actual members, selection rules, location, calendar, agenda offices, quorum, aggregation rule, resources, and jurisdiction |
| **Office** | Powers attached to the role, current occupant, term, appointment/removal rule, and delegation permissions |
| **Procedure** | A directed graph of stages, conditions, alternatives, and reconsideration loops |
| **Authority record** | Which actor or institution recognizes the act as valid, and why |
| **Agent legal knowledge** | Rules and versions known, confidence, source, and time learned |

The constitution should contain **rules for selecting procedures**, not just one procedure:

```
select procedure using:
    subject matter
    territorial scope
    initiating authority
    existing delegations
    emergency conditions
    constitutional hierarchy
```

Ordinary regulations, taxes, constitutional changes, emergency orders, and local bylaws may follow different paths. A constitutional amendment must also specify what happens to proposals already in progress.

### 5.2 Let the same population supply legislators, petitioners, and implementers

A council member should be the same person who owns land, has relatives, depends on patrons, fears violence, and must find time to attend.

Use persistent individual differences and relationships, but calculate policy reactions from actual consequences. A landholder may favor collective irrigation while opposing the proposed financing mechanism. Two members of the same faction may diverge because their holdings or obligations differ.

Do not make an institution a separate omniscient personality. Its behavior should result from its occupants, procedures, information, and resources.

### 5.3 Make legislation event-driven

For 10k–50k agents, the expensive mistake would be continuously simulating every possible political conversation.

**Recommended approach:**

* Schedule proposal submissions, meetings, deadlines, messages, and publication as discrete events.
* Recompute an agent’s assessment when the proposal, relevant circumstances, or available information changes.
* Generate a bounded set of amendments and compromises instead of searching all possible laws.

Cache material effects for groups sharing relevant circumstances, while retaining individual votes and politically important exceptions. Parties and kin groups can coordinate behavior through relationships and commitments without becoming perfect blocs.

This is an architectural recommendation, not a measured Rust performance claim.

### 5.4 Connect the political model to visible daily life

Use TCE’s existing simulation systems to make procedure observable:

A petitioner visits a patron; a councillor misses a meeting during harvest; a messenger carries a summons; a scribe prepares copies; a public reader announces a decision; an official arrives to implement it.

These should not all be decorative animations after an abstract approval roll. Attendance, communication, and labor should affect the process where the model represents them.

Conversely, the simulation need not resolve every sentence of a speech. A deliberative exchange can be an event containing an argument, information item, amendment, promise, or threat.

### 5.5 A worked example: an irrigation law

Suppose TCE has generated a settlement with a household assembly, an elected canal council, and a chief authorized to issue temporary emergency orders.

A drought creates an **issue**. Downstream farmers petition the council. The council’s chair can schedule a proposal but cannot enact a permanent allocation rule.

The council drafts a rotation. Upstream households object. A compromise adds shared maintenance labor and compensation. Some farmers miss the assembly because travel and harvest work are costly. The assembly votes under its actual quorum and household-representation rules.

If the proposal passes, the settlement authenticates and announces it. An inspector still needs time and authority to implement the schedule.

If the assembly is not due to meet before the crops fail, the chief may issue a temporary order—but only if the constitution supplies that route. The order’s expiry and ratification requirements create further political decisions.

**No “chiefdom lawmaking modifier” is necessary.** Urgency, access, attendance, interests, compromise, delegated power, and enforcement generate the outcome.

### 5.6 Existing models and games worth borrowing from

| Model or game | Useful component | What TCE should not inherit uncritically |
| --- | --- | --- |
| **Tsebelis’s veto-player framework** | Required consents, policy stability, and institutional redundancy | A blanket penalty based only on the number of chambers or parties. [LSA Technology Services](https://sites.lsa.umich.edu/tsebelis/wp-content/uploads/sites/246/2015/03/decision_making_in_political_systems_1995_bjps.pdf) |
| **Baron–Ferejohn legislative bargaining** | Recognition, proposer advantage, coalition offers, amendment rules | An assumption that all politics concerns dividing a single transferable resource under perfect strategic reasoning. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/bargaining-in-legislatures/2636DB331955032B9421DB0EB25467CC) |
| **Crawford–Ostrom institutional grammar** | Structured representation of prescriptions and institutional statements | Treating the grammar alone as a complete executable model of legal authority and process. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/grammar-of-institutions/7D37CD3BC5ED2D9FD57D2EE292958F47) |
| **Victoria 3’s documented version 1.3 enactment design** | Staged enactment, interest-group pressure, setbacks, and political demands | Replacing actual voting and procedure with a universal progress-and-chance mechanism. This comparison concerns the documented 2023 design, not an assertion about the current version. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-80-law-enactment-and-revolution-clock-in-13) |
| **Eco’s constitution and government system** | Configurable relationships between constitutional rules, laws, and government powers | Assuming that a rule’s existence should physically prevent every prohibited action. The referenced documentation is version-sensitive community documentation. [Eco - English Wiki](https://wiki.play.eco/en/Government) |

**Best first implementation:** build decree, advisory council, binding council, assembly, representative chamber, and popular-ratification components; combine them through procedure graphs. Add drafting and meeting capacity, a small bargaining model, and local dissemination. Expand the authored procedures rather than adding a separate simulator for every named regime.

---

## 6. Sources, datasets, and limits of inference

### 6.1 Data sources most useful for calibration

| Source | Best use | Important limitation |
| --- | --- | --- |
| **Comparative Constitutions Project** | Written constitutional powers, institutional arrangements, and constitutional chronology; the 2026 chronology release extends through 2025 | Formal constitutional provisions are not observations of day-to-day compliance. Coverage is not a substitute for ancient or customary constitutional history. [Comparative Constitutions Project](https://comparativeconstitutionsproject.org/download-data/) |
| **Comparative Agendas Project** | Comparing attention and government activity using shared policy-topic classifications | Country coverage and available activities differ; it is not a uniform global registry of all failed proposals. [Comparative Agendas](https://www.comparativeagendas.net/) |
| **IPU Parline** | Modern parliamentary structure, membership, organization, and operating characteristics | Variables have different coverage; institutional profiles do not replace bill-level histories. [IPU Parline](https://data.ipu.org/) |
| **Official legislative histories and enactment registers** | Introduction, committee consideration, amendments, passage, signature, and publication dates | Bills can be duplicated, bundled, split, or enacted through another vehicle. Track proposal families and text versions. [National Archives](https://www.archives.gov/federal-register/laws/118-second-session) |
| **Ancient texts, inscriptions, and institutional histories** | Constructing historically specific procedural test cases | Surviving prescriptions often reveal what should happen, not how often it actually happened. [Pure](https://www.pure.ed.ac.uk/ws/files/20012342/OHAGL_Canevaro_4.pdf) |

For the theoretical core, the most directly useful works are **Crawford and Ostrom, “A Grammar of Institutions” (1995); Tsebelis, “Decision Making in Political Systems” (1995) and “Veto Players and Law Production” (1999); and Baron and Ferejohn, “Bargaining in Legislatures” (1989)**. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/grammar-of-institutions/7D37CD3BC5ED2D9FD57D2EE292958F47)

For evaluating claims about throughput, pair **Binder, “The Dynamics of Legislative Gridlock, 1947–96” (1999)** with **Chiou and Rothenberg’s 2008 reconsideration**, and include **Truex, “Authoritarian Gridlock?” (2020; first published online in 2018)** rather than assuming competitive democracies uniquely suffer legislative obstruction. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/dynamics-of-legislative-gridlock-194796/ED0A880C049DB34F2BDFA9162A72011C)

### 6.2 Where uncertainty matters most

**Ancient procedural reconstruction.** The exact organization of Athenian *nomothesia* and the calendar interpretation of Roman notice requirements remain contested. Encode alternative reconstructions rather than concealing uncertainty behind one precise constant. [Pure](https://www.pure.ed.ac.uk/ws/files/20012342/OHAGL_Canevaro_4.pdf)

**Prescriptions versus practice.** An inscription can establish a formal eligibility rule without demonstrating that every appointment followed it. Uttaramerur is especially valuable as a detailed institutional document, but the translation used here is an older scholarly text reproduced on a mirror, not a new critical edition. [GeoCities](https://www.geocities.ws/ifihhome/articles/uttaramerur.html)

**Ethnographic extrapolation.** Evidence from Ju/’hoansi, Tswana, or contemporary Indigenous institutions should inform possible mechanisms, not stand in for all prehistoric or non-European societies. [Arizona State University](https://asu.elsevierpure.com/en/publications/norm-enforcement-among-the-juhoansi-bushmen-a-case-of-strong-reci/)

**Comparability of output.** Counting enactments cannot by itself distinguish productive government from symbolic legislation, technical consolidation, delegation, or repeated attempts to enforce an existing rule. TCE should therefore expose several measures rather than one “legislative efficiency” score.

**Bottom line:** model lawmaking as **controlled access to authoritative decisions, constrained by bargaining, institutional calendars, work capacity, and communication**. Let regime labels describe the resulting configuration. Do not let those labels determine the procedure.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928bb-ca0c-83e9-b6fb-6aa593fce36d)
