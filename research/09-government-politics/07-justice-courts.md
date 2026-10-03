# Justice systems, courts, and punishment

## A simulation-ready report for The Civilization Engine

**The central recommendation is to model justice as a branching process for handling grievances—not as “crime → arrest → punishment.”** People may ignore an injury, negotiate, demand compensation, mobilize relatives, seek mediation, initiate a prosecution, retaliate, or approach competing authorities. Arrest is only one possible intervention.

Also, **customary adjudication and remembered precedents should exist from TCE’s beginning**. Professional judges, permanent courthouses, written records, and appellate hierarchies can develop later. Oral Somali *xeer*, for example, incorporates precedents, while Hindu legal thought connects judicial decisions with established social practice. Neither requires the modern common-law institutional package. [HD](https://hdcentre.org/wp-content/uploads/2016/07/StatelessJusticeinSomalia-July-2005.pdf)

Do not make justice follow an inevitable progression from violent custom to humane bureaucracy. A comparative study of **131 largely nonindustrial societies** found substantial variation in punishment and no support for simply concentrating harsh punishment in the least politically complex societies. Its observations describe particular ethnographic settings, not a chronological reconstruction of humanity’s past. [Cambridge University Press](https://www.cambridge.org/core/journals/evolutionary-human-sciences/article/norm-violations-and-punishments-across-human-societies/F5B13F32B188E8A874A333A19E532615)

Throughout this report, **historical observations**, **formal prescriptions**, and **proposed simulation parameters** are distinguished.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 The basic dispute-processing system

A useful minimum process is:

```
Incident or perceived injury
    → recognition and attribution
    → ignore / avoid / negotiate / retaliate / seek a forum
    → settlement OR investigation and adjudication
    → dismissal / acquittal / liability or conviction
    → remedy or sentence
    → review / pardon / enforcement / evasion / default
    → changes in relationships, assets, security, and institutional reputation
```

Negotiation can recur at several stages. A civil claim need never involve detention. A public prosecution may continue after the victim accepts compensation, but only where the polity’s rules authorize that.

The following are **proposed implementation rules**, informed by the historical cases and measurements below.

| Mechanism | Implementable rule | Consequences to allow |
| --- | --- | --- |
| **Recognition of a wrong** | Compare an observed event with the agent’s understood norms and recognized rights—not directly with the kernel’s complete legal database. | Unrecognized claims, disagreement over whether harm was wrongful, conflicting community norms. |
| **Attribution and accusation** | Construct beliefs about responsibility from observations, testimony, reputation, and inference. Keep actual responsibility separate. | Mistaken accusations, concealment, strategic accusations, uncertainty. |
| **Standing and jurisdiction** | A forum checks who may initiate proceedings, against whom, over which matters, and in which territory or community. | Excluded claimants, overlapping authorities, disputes about the correct forum. |
| **Choice of response** | Compare expected remedy, protection, relationship effects, costs, delay, and retaliation risk across available responses. | Private bargaining, avoidance, forum shopping, or abandonment despite a strong claim. |
| **Collective responsibility** | Where authorized, make kin groups, households, patrons, or associations contribute to compensation or guarantee appearance. | Social insurance, pressure on offenders, disputes within the paying group. |
| **Settlement** | Seek an agreement acceptable to the parties whose consent matters under local rules. Agreements can include property, payment schedules, apologies, guarantees, or separation. | Peace without agreement about guilt; powerful relatives imposing a settlement on an individual. |
| **Adjudication** | Apply the forum’s evidence rules and decision procedure using the decision-makers’ information and incentives. | Acquittals, inconsistent decisions, favoritism, reasoned judgments, wrongful convictions. |
| **Sentence selection** | Select a permitted sanction bundle using offense, intent, status, relationship, prior record, and legally authorized discretion. | Different treatment of apparently similar harms. |
| **Enforcement** | Require an actual payer, collector, guard, escort, community coalition, or other enforcement mechanism. | Unpaid awards, escape, partial performance, renegotiation, substitution of penalties. |
| **Institutional feedback** | Update beliefs from experienced fairness, outcomes, delay, collection, and retaliation. | Increased use of trusted forums; withdrawal from predatory or ineffective ones. |

### 1.2 Compensation is not simply a fine

Represent at least three distinct transfers:

**Restitution** returns an item or repairs a loss. **Compensation** transfers resources to an injured party or that party’s recognized group. **A fine** transfers resources to an authority.

These transfers have different consequences. Compensation can reduce a victim group’s incentive to retaliate; a treasury fine may leave that grievance untouched. Both can be imposed in the same case.

In the Somali arrangements documented by André Le Sage, compensation obligations can fall on a *diya*-paying group rather than the offender alone. This makes collective liability a mechanism for both financing settlements and disciplining members. It is not equivalent to an individual damages award collected by a state. [HD](https://hdcentre.org/wp-content/uploads/2016/07/StatelessJusticeinSomalia-July-2005.pdf)

**TCE implementation:** create an obligation with explicit debtors, beneficiaries, guarantors, installments, and default rules. An award should not instantly create money. An insolvent offender may need relatives, a patron, asset sales, or renegotiation; the governing institution determines what happens when those fail.

### 1.3 Adjudication is a choice of decision procedure, not a technology tier

Separate **who decides**, **how evidence is evaluated**, and **how a verdict is aggregated**.

Possible decision-makers include a mutually accepted mediator, elders, a ruler, a magistrate, a professional judge, a lay panel, or a jury. Appointment, inheritance, election, rotation, and lottery are separate selection primitives.

Classical Athens provides a particularly useful counterexample to a simple developmental ladder: its citizen courts used selection by lot and large panels, while eligibility remained restricted. The *Athenian Constitution* also describes juror payment and procedural safeguards around assignment and voting. [Internet Classics Archive](https://classics.mit.edu/Aristotle/athenian_const.3.3.html)

For a panel, store the individual members’ beliefs and relationships, but aggregate them with a simple authored procedure: unanimity, majority, qualified majority, or a chair’s final decision. Do not assume that a larger panel automatically supplies independent information; several jurors may share the same rumor or patron.

### 1.4 Oaths, ordeals, and evidence

An evidentiary system determines what counts as persuasive or legally sufficient. It need not match modern factual investigation.

For ordeals, distinguish:

* belief in supernatural judgment;
* willingness to undergo the procedure;
* control of the procedure and interpretation by authorities;
* the community’s acceptance of the result.

Leeson’s “Ordeals” proposes a belief-mediated selection mechanism: willingness to undergo an ordeal can convey information when participants believe it reveals guilt. This is a theoretical explanation, **not an established estimate of ordeal accuracy**. TCE should not implement ordeals as supernatural access to the kernel’s truth state. [Chicago Unbound](https://chicagounbound.uchicago.edu/jle/vol55/iss3/8/)

**Recommended abstraction:** each forum has a small set of admissible evidence categories and thresholds. Individual adjudicators then interpret the admissible evidence with imperfect knowledge, credibility judgments, and possible pressure. Procedural acceptability and factual correctness must remain separate outputs.

### 1.5 Codes and precedent are independent dimensions

Avoid a `CommonLaw | CivilLaw` switch as the universal representation.

Instead, give a legal system separate values for:

| Dimension | What it controls |
| --- | --- |
| **Written coverage** | How much of the applicable law is recorded and accessible. |
| **Authority hierarchy** | Which sources override others: enactments, religious texts, custom, ruler’s orders, decisions. |
| **Precedent authority** | Whether earlier decisions are binding, persuasive, or merely informative. |
| **Interpretive discretion** | How freely adjudicators distinguish cases or reinterpret rules. |
| **Record reach** | Whether decisions are known locally, across a jurisdiction, or throughout a polity. |
| **Revision procedure** | Who may alter a rule or overturn an interpretation. |

Judicial lawmaking also need not converge toward a single efficient answer. Gennaioli and Shleifer’s formal models distinguish refinement of precedent from outright overruling; their overruling model can produce persistent instability. Empirically, Niblett, Posner, and Shleifer tracked **465 appellate decisions** concerning one commercial rule and found divergent development rather than convergence to a stable rule. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/pdf/10.1086/511996?utm_source=chatgpt.com)

---

## 2. Quantitative parameters and calibration anchors

### How to read the evidence

**H** means strong evidence for the stated population or legal text. **M** means useful but qualified evidence, such as a survey or localized historical reconstruction. **L** means thin evidence. Confidence does **not** imply that a value transfers to other societies.

A tariff can be well attested as a prescription while its actual collection rate remains unknown.

### 2.1 Sanctions: prescriptions and observed outcomes

| Parameter or observation | Value, units, and scope | Recommended use | Confidence and source |
| --- | --- | --- | --- |
| **Status differentiation in wergild** | Mercian legal values: **200 shillings for a ceorl; 1,200 for a thegn**, a **6:1 ratio**. | Example of an explicitly status-dependent compensation schedule. These are historical accounting units, not modern sterling. | **H** for cited legal values; **L** for realized payment. Bosworth–Toller’s entry cites the underlying laws. [Linguistics Research Center](https://lrc.la.utexas.edu/books/asd/dict-S?utm_source=chatgpt.com) |
| **Somali homicide compensation benchmarks** | Commonly stated *xeer* benchmarks: **100 camels for a male victim; 50 for a female victim**, with variation between arrangements. | Group-funded compensation with socially unequal valuation. Do not impose this schedule on other African societies. | **M**, institutional field report; prescription rather than measured average settlement. [HD](https://hdcentre.org/wp-content/uploads/2016/07/StatelessJusticeinSomalia-July-2005.pdf) |
| **Status/property-based bodily-injury remedy** | Hammurabi §199 prescribes **half the enslaved person’s value** for specified injuries. | Demonstrates that the beneficiary and basis of valuation can differ fundamentally from modern personal damages. | **H** for the text, not enforcement frequency. [Avalon Project](https://avalon.law.yale.edu/ancient/hamcode.asp) |
| **Execution conditional on capital conviction, Old Bailey, 1750–1759** | **329 executions / 480 condemned = 68.5%**. Calculated from Devereaux’s annual database counts. | A local calibration of the sentence-to-execution stage—not the probability that a crime causes execution. | **H/M**, reconstructed archival series. [PubPub](https://assets.pubpub.org/539i8e0a/71625490165601.pdf) |
| **Same measure, Old Bailey, 1810–1819** | **169 / 1,552 = 10.9%**, calculated in the same way. | Shows that realized punishment can change greatly without equating every capital sentence with execution. Nonexecution does not imply release. | **H/M**, same source. [PubPub](https://assets.pubpub.org/539i8e0a/71625490165601.pdf) |
| **Modern fine frequency, Germany, 2024** | **506,500**, approximately **80% of 632,100 convictions**, received a fine under general criminal law. | A modern fine-heavy benchmark, not a universal modern punishment mix. | **H**, official judicial statistics. [Destatis](https://www.destatis.de/DE/Presse/Pressemitteilungen/2025/12/PD25_452_24311.html?utm_source=chatgpt.com) |
| **German imprisonment-sentence category, 2024** | Approximately **85,700 convictions, or 14%**, involved imprisonment under general criminal law or military detention; the remaining **6%** used juvenile law. | A sentencing classification. Do not interpret 14% as the share immediately admitted to prison. | **H**, official statistics. [Destatis](https://www.destatis.de/DE/Presse/Pressemitteilungen/2025/12/PD25_452_24311.html?utm_source=chatgpt.com) |
| **Global detention awaiting sentence, 2023** | Approximately **3.7 million of 11.7 million prisoners: 31% unsentenced**. | Calibrate remand separately from sentenced imprisonment. This is a population stock, not an annual admissions share. | **H/M**, UN global estimates. [UNSD](https://unstats.un.org/sdgs/report/2025/goal-16/) |

**Important limitation:** these do not establish an “ancient,” “medieval,” or “modern” universal punishment distribution. Reliable percentages require a specified jurisdiction, offense mix, population, and denominator. In particular, the incidence of fines, corporal penalties, exile, and forced labor cannot responsibly be inferred from how prominently a code discusses them.

### 2.2 Access, cost, staffing, and delay

| Parameter or observation | Value and units | Interpretation for TCE | Confidence and source |
| --- | --- | --- | --- |
| **Incidence of everyday legal problems** | **49% of surveyed people** experienced at least one problem during the preceding **two years**; WJP surveys in **101 countries/jurisdictions**, 2017–2018. | A broad legal-needs benchmark, not a crime rate or annual case-arrival rate. | **M**, cross-country survey. [World Justice Project](https://worldjusticeproject.org/our-work/research-and-data/global-insights-access-justice-2019) |
| **Advice and third-party involvement** | Among people with a legal problem, **29% sought advice** and **17% approached an authority or third party for mediation/adjudication**. | Most problems do not automatically become court cases. The 17% includes noncourt processes. | **M**, same survey. [World Justice Project](https://worldjusticeproject.org/our-work/research-and-data/global-insights-access-justice-2019) |
| **Financial access difficulty** | WJP reports **16%** found obtaining the necessary money difficult or nearly impossible. Its process questions concern problems whose process had ended, including abandonment. | Validate affordability barriers and abandonment; this is not a fee schedule. | **M**, survey measure with a restricted process denominator. [World Justice Project](https://worldjusticeproject.org/our-work/research-and-data/global-insights-access-justice-2019) |
| **Professional judicial staffing, Europe, 2022** | **Median 17.6; mean 21.9 professional judges per 100,000 inhabitants**. | A modern institutional-scale benchmark, not an agrarian requirement. | **H** for reporting jurisdictions. [Council of Europe](https://rm.coe.int/cepej-evaluation-report-part-1-en-/1680b272ac) |
| **First-instance disposition-time indicator, Europe, 2022** | Medians: **133 days criminal; 239 civil/commercial litigious; 292 administrative**. | Match like-for-like case categories. These are stock/flow indicators, not observed mean waiting times. | **H**, CEPEJ methodology. [Council of Europe](https://rm.coe.int/cepej-evaluation-report-part-1-en-/1680b272ac) |
| **Judicial independence measurement** | V-Dem expert response categories run **0–4**, from always to never merely following government wishes in politically salient cases. | Useful dimensions and historical comparisons; do not convert ordinal scores directly into probabilities. | **M**, expert-coded, model-based measurement. [V-Dem](https://v-dem.net/documents/55/codebook.pdf) |

CEPEJ’s indicator is:

\[
DT=365\frac{\text{pending cases at year end}}{\text{cases resolved during the year}}
\]

It is not the number of judge-days required per case, nor necessarily the elapsed time experienced by the median litigant. [Council of Europe](https://rm.coe.int/cepej-evaluation-report-part-1-en-/1680b272ac)

### 2.3 Explicit design priors where historical measurements are inadequate

The following are **author-proposed starting ranges**, not historical estimates. Their purpose is sensitivity testing and plausible scheduling.

| Parameter | Initial experimental range | Units and implementation |
| --- | --- | --- |
| Informal mediation effort | **0.25–2** | Facilitator-person-days per attempt; participants incur their own time costs. |
| Routine adjudication effort | **0.25–3** | Adjudicator-person-days per case, excluding waiting and travel. |
| Complex adjudication effort | **3–20** | Adjudicator-person-days; use a right-skewed distribution. |
| Interval between scheduled sessions | **7–90** | Calendar days for intermittently sitting forums; emergency hearings are separate. |
| Rehearing/review effort | **0.5–2 times** initial effort | Depends on whether review is documentary or repeats evidence gathering. |
| Sensitivity levels for enforceability | **0.1, 0.5, 0.9** | Conditional probability of obtaining the ordered remedy; ultimately replace with agent/resource mechanics. |

All six have **low empirical confidence as universal values**. They should remain visibly marked as design parameters.

For monetary burdens, report both nominal value and **days of local household consumption or disposable earnings**. Avoid converting historical shillings or camels into a single timeless currency.

---

## 3. Variation across eras and world regions

### 3.1 Era comparison: capabilities, not an upgrade ladder

| Setting | Attested institutional and punishment patterns | Modeling implications and evidence limits |
| --- | --- | --- |
| **Forager and other small-scale societies** | Ethnographic records document reputational, material, physical, and lethal sanctions in differing combinations. | Model mobility, social dependence, coalition formation, and the possibility of exclusion. Do not use a single “forager justice” package or infer prehistoric event frequencies from modern ethnography. [Cambridge University Press](https://www.cambridge.org/core/journals/evolutionary-human-sciences/article/norm-violations-and-punishments-across-human-societies/F5B13F32B188E8A874A333A19E532615) |
| **Early farming communities** | Direct evidence for everyday procedures and punishment frequencies is especially thin. Later village ethnography is an analogy, not a record of the first farmers. | As a design hypothesis, persistent land boundaries, stored goods, inheritance, and repeated neighborhood interactions should create demand for adjudication. Mark the rates and institutional sequences as uncertain. |
| **Preindustrial states and cities** | Written rules, local custom, status-dependent remedies, lay adjudication, corporal punishment, exile, labor penalties, and capital punishment could coexist. Ancient Athens also used imprisonment in connection with unpaid public debt and penalties. | Do not reserve detention for industrial societies or assume written law eliminates customary forums. [Avalon Project](https://avalon.law.yale.edu/ancient/hamcode.asp) |
| **Industrial and imperial expansion** | Penal transportation and labor systems connected punishment with colonization, migration, and economic projects across several empires. | A prison-centered domestic system can coexist with distant labor colonies and different treatment of subject populations. Transportation is more than deleting a person from the map. [Bloomsbury Publishing](https://www.bloomsbury.com/9781350000674?utm_source=chatgpt.com) |
| **Modern systems** | Noncustodial penalties can dominate convictions, while detention before sentence remains substantial. Customary and community-based processes can coexist with formal courts. | Keep fines, probation-like restrictions, remand, imprisonment, and consensual settlement as distinct mechanisms. Modernity should not force a single sanction mix. [Destatis](https://www.destatis.de/DE/Presse/Pressemitteilungen/2025/12/PD25_452_24311.html?utm_source=chatgpt.com) |

The economic feasibility of a penalty should be computed rather than attached to an era. Long-term confinement requires continuing food, shelter, custody, and administration. Exile requires the ability to exclude someone from a territory or community. Forced labor requires supervision and a usable work assignment. These are **simulation resource constraints**, not claims that institutions always minimize costs.

### 3.2 Regional institutional examples

**Mesopotamia: written rules and unequal legal status.**  
Hammurabi’s legal collection differentiates remedies according to status and relationship. Its provisions are excellent templates for conditional legal rules, but poor evidence for the frequency with which those rules were invoked. Treat the collection as a normative source, not a complete administrative dataset or proof of uniformly applied modern-style codification. [Avalon Project](https://avalon.law.yale.edu/ancient/hamcode.asp)

**China: graded punishment plus administrative review.**  
The Qing system’s conventional five punishments comprised light and heavy bamboo beating, penal servitude, exile, and death. Serious cases moved through supervisory levels; capital cases were reviewed before the emperor’s final decision. Some death sentences entered an annual autumn-review process rather than immediate execution. This is a strong example of elaborate review within an imperial hierarchy—not proof of independent constitutional courts. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-asian-studies/article/abs/wrongful-treatment-of-prisoners-a-case-study-of-ching-legal-practice/993F8FAB38AF2C84D4E0834E9465201A)

The Tang code’s “Ten Abominations” also shows why TCE needs offenses against politically and culturally defined relationships: loyalty to ruler, family hierarchy, and ritual obligations could determine legal seriousness. A universal offense system based only on physical injury and property loss would miss this. [AFE East Asia](https://afe.easia.columbia.edu/main_pop/ps/ps_china-zhangsun-wuji-great-tang-code.htm)

**South Asia: legal texts, social practice, and plural authority.**  
Donald Davis’s account of Hindu law emphasizes household and social relationships, not merely commands issued by a centralized state. His treatment of *vyavahāra* connects legal procedure to both ordinary practice and prior judicial decisions. TCE should therefore allow a legally sophisticated society without requiring one monopolistic court system, and should not treat a Sanskrit normative text as a uniformly enforced pan-Indian statute. [Cambridge University Press](https://www.cambridge.org/core/books/spirit-of-hindu-law/E738FB6987860874ECB74BC155D283BA)

**Islamic and Ottoman settings: different legal tracks for different claims.**  
Rudolph Peters distinguishes legal domains with different procedural and evidentiary rules. In the law of homicide and injury, the victim or heirs could control initiation of the private claim. Retaliation and compensation therefore cannot be modeled as interchangeable state sentences. Prescribed offenses and penalties also need to be distinguished from discretionary punishment and governmental practice. A single automatic “religious law punishment table” would erase these differences. [Cambridge University Press](https://www.cambridge.org/core/books/abs/crime-and-punishment-in-islamic-law/conclusion/5D3CDCF3593FBD8D86373D38029FACBE)

**Europe: status tariffs, citizen adjudication, and changing mercy.**  
Mercian wergild and Athenian juries illustrate institutional combinations separated by place, date, and political structure—not consecutive upgrades. Later, Old Bailey execution data show why the administration of mercy must be modeled separately from the formal sentencing schedule. Changing actual outcomes is possible through discretion, pardons, and substitution, not just legislative amendment. [Linguistics Research Center](https://lrc.la.utexas.edu/books/asd/dict-S?utm_source=chatgpt.com)

**Africa: customary settlement can both restrain and reproduce coercion.**  
A randomized community-education intervention in Liberia treated **86 of 246 towns**, training approximately **15% of adults** in alternative dispute-resolution practices. After one year, the study found more resolution of land disputes and less violence, but also increased extrajudicial punishment. This is useful causal evidence against assuming that stronger informal institutions are uniformly benign. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/how-to-promote-order-and-property-rights-under-weak-rule-of-law-an-experiment-in-changing-dispute-resolution-behavior-through-community-education/63D2C071E48C25A2F5861D4248E59A90)

**Indigenous North America: coexistence rather than replacement.**  
The Navajo Nation’s judicial materials distinguish consensual peacemaking from court adjudication and describe pathways connecting them. Some matters requiring legal orders can be filed in court and referred to peacemaking, with agreements subsequently incorporated into orders. This supplies an institutional composition example, not a template for all Indigenous societies or an unchanged survival from prehistory. [Navajo Courts](https://courts.navajo-nsn.gov/FAQs.htm)

**Oceanic and other colonial settings: punishment as population movement.**  
Clare Anderson’s comparative work places penal transportation within a worldwide history of forced migration and empire. For TCE, a distant penal settlement should affect destination labor supply, land appropriation, family separation, mortality exposure, and possible settlement after release. Those effects belong in migration and economy systems as well as justice. [University of Birmingham](https://research.birmingham.ac.uk/en/publications/a-global-history-of-convicts-and-penal-colonies/?utm_source=chatgpt.com)

---

## 4. Access, costs, delays, judicial independence, and review

### 4.1 Access is a sequence of barriers

An agent needs more than a courthouse within walking distance. TCE should test:

**Recognition → standing → knowledge of a forum → affordability → safe participation → ability to prove the claim → enforceable remedy.**

Failure at any point can prevent effective access.

A useful **decision-model proposal** is:

\[
U\_i(f)=
\hat p\_{if}(\text{success})
\hat p\_{if}(\text{enforcement})
V\_i(\text{remedy})
+V\_i(\text{security and recognition})
-C\_{if}
\]

where:

\[
C\_{if}=
\text{fees}+\text{travel}+\text{lost work}
+\text{delay cost}+\text{expected retaliation cost}.
\]

The probabilities are the agent’s beliefs, not omniscient values. Personality, anger, obligation, honor, and loyalty can modify or override this calculation.

Financial hardship should operate through cash flow. A household can own substantial land but lack the food or money to sustain repeated travel. A formally inexpensive case can still be unaffordable during harvest or after detention of its main earner.

WJP’s observed pattern—widespread problems but relatively little third-party intervention—provides a calibration target for this distinction between having a claim and pursuing adjudication. [World Justice Project](https://worldjusticeproject.org/our-work/research-and-data/global-insights-access-justice-2019)

### 4.2 Delay should emerge from the process

For forum \(f\), track:

\[
B\_{t+1}=B\_t+A\_t-H\_t-S\_t-W\_t,
\]

where \(B\) is pending cases, \(A\) new filings, and \(H,S,W\) are mutually exclusive closures through adjudication, settlement, and withdrawal.

Schedule actual service work, adjournments, missing witnesses, travel, and review. If arrivals persistently exceed service capacity, the backlog grows. This is a queueing consequence, not an additional historical assumption.

For a forum meeting every \(s\) days, uniformly timed arrivals imply an average **calendar wait of \(s/2\)** before the next session, before adding congestion. Seasonal closures or infrequent circuits therefore matter even when individual hearings are brief.

Record both **time to closure** and **whether the claim was resolved**. A case abandoned after years should not count as successful justice simply because it left the docket.

### 4.3 Judicial independence is not one scalar

Use an institutional vector:

| Dimension | Agent-level implementation |
| --- | --- |
| **Appointment dependence** | Who selects the adjudicator, and what loyalty or debt results? |
| **Removal and career security** | What happens after an adverse decision against a powerful actor? |
| **Financial dependence** | Who controls salary, court funds, fee income, and collection? |
| **Social dependence** | Kin, patrons, local elites, professional peers, and religious authorities. |
| **Decisional autonomy** | Whether the adjudicator follows their own interpretation in salient cases. |
| **Enforcement autonomy** | Whether an adverse judgment will actually be obeyed or executed. |

V-Dem usefully separates high-court and lower-court independence and includes measures of government compliance with judicial decisions. Its scores are indicators of observed political institutions, not direct parameters such as “a 40% chance of bribery.” [V-Dem](https://v-dem.net/documents/55/codebook.pdf)

**Design implication:** an independent court can be powerless; an effective court can be politically dependent; a court independent of the ruler can still favor local elites.

### 4.4 Review requires several separate primitives

Distinguish **rehearing**, **appeal**, **automatic sentence confirmation**, **pardon or commutation**, **review of administrative conduct**, and **constitutional review**.

Specify who can trigger review, what it can examine, whether it suspends enforcement, and what remedies the reviewer can order. An appeal may challenge facts; another may address only procedure or law. A pardon need not declare the original conviction wrong.

The Qing case demonstrates that mandatory supervisory review and delayed execution can operate inside a centralized imperial system. Review should therefore not unlock only when a polity adopts democratic or separation-of-powers institutions. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-asian-studies/article/abs/wrongful-treatment-of-prisoners-a-case-study-of-ching-legal-practice/993F8FAB38AF2C84D4E0834E9465201A)

---

## 5. Stylized facts a correct simulation should reproduce

| Pattern | Observable TCE test |
| --- | --- |
| **Disputes greatly outnumber adjudicated cases.** Modern WJP evidence puts third-party mediation/adjudication at **17%** among people experiencing a legal problem. | Count perceived injuries, claims, advice, settlements, filings, and judgments separately. More filings can result from improved access rather than more underlying harm. [World Justice Project](https://worldjusticeproject.org/our-work/research-and-data/global-insights-access-justice-2019) |
| **Equal physical harm need not produce equal remedies.** Status-dependent schedules can differ greatly, as in the **6:1** Mercian example. | Compare outcomes after holding harm constant and varying legally relevant status. [Linguistics Research Center](https://lrc.la.utexas.edu/books/asd/dict-S?utm_source=chatgpt.com) |
| **Sentences are not completed punishments.** | Preserve separate records for prescribed, imposed, commuted, attempted, and completed sanctions; reproduce changing execution shares without changing every offense definition. [PubPub](https://assets.pubpub.org/539i8e0a/71625490165601.pdf) |
| **Modern criminal justice need not be predominantly imprisonment.** Germany’s 2024 convictions were **80% fines**. | A fine-heavy modern polity should be possible without treating most convictions as prison admissions. [Destatis](https://www.destatis.de/DE/Presse/Pressemitteilungen/2025/12/PD25_452_24311.html?utm_source=chatgpt.com) |
| **Detention is partly produced by procedure.** Approximately **31%** of the global prison population was unsentenced in 2023. | Slower processing or stricter release decisions should enlarge remand populations even at unchanged conviction rates. [UNSD](https://unstats.un.org/sdgs/report/2025/goal-16/) |
| **More severe penalties are not a universal substitute for enforcement.** Nagin’s review finds stronger, more consistent deterrence evidence for certainty—especially apprehension—than severity. | Increasing a penalty should have limited effects when agents consider apprehension unlikely, do not know the rule, or act impulsively. Do not impose one universal deterrence elasticity. [DOI](https://doi.org/10.1086/670398) |
| **Informal institutional improvement can have mixed effects.** | Mediation can reduce violent disputes while also strengthening coercive community sanctions; track both rather than one “justice quality” score. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/how-to-promote-order-and-property-rights-under-weak-rule-of-law-an-experiment-in-changing-dispute-resolution-behavior-through-community-education/63D2C071E48C25A2F5861D4248E59A90) |
| **Precedent does not guarantee convergence.** | Different jurisdictions can retain distinct interpretations under similar conditions, rather than inevitably discovering one optimal rule. [National Bureau of Economic Research](https://www.nber.org/papers/w13856?utm_source=chatgpt.com) |

---

## 6. Recommended representation for 10k–50k agents

### 6.1 Minimum data model

Use six principal entity types:

| Entity | Essential fields |
| --- | --- |
| **Incident** | Actual participants, actions, location, time, harm, intent, observable traces. Keep hidden truth separate from legal knowledge. |
| **Claim/case** | Claimants, accused/respondents, alleged norms, forum, evidence references, requested remedy, procedural stage, deadlines. |
| **Forum** | Jurisdiction, access rules, personnel, selection method, calendar, budget, authority sources, evidence rules, decision procedure, review links. |
| **Evidence item** | Origin, observer, content, reliability, custody, who knows it, admissibility. |
| **Judgment/precedent** | Material fact tags, interpreted rule, decision, authority, jurisdiction, date, later treatment. |
| **Sanction/obligation** | Type, amount or duration, beneficiaries, responsible parties, enforcement actor, progress, default and substitution rules. |

Each person needs links to relevant cases, outstanding obligations, legal status, procedural knowledge, perceived forum reliability, and significant grievances. Avoid giving every person a complete memory of every proceeding.

### 6.2 Represent punishment as a bundle

Do not compress sanctions into a single severity number. A sentence may combine:

```
return property
pay victim compensation
pay treasury fine
perform labor
remain in custody
leave or avoid a territory
lose an office or other legal privilege
undergo a corporal penalty
face execution
```

These produce distinct effects on health, assets, employment, family support, mobility, reputation, and political relationships.

For forced labor, count supervision, transport, sustenance, work productivity, escape, and injury rather than assuming costless output. For exile, specify who recognizes the exclusion and what happens at neighboring jurisdictions. For confinement, distinguish remand from punishment and legal sentence length from actual time served.

A useful accounting identity is:

\[
P\_{t+1}=P\_t+\text{admissions}-\text{releases}-\text{escapes}-\text{deaths}.
\]

Under approximately steady conditions, average custody population equals admission flow multiplied by average stay. Thus a larger prison population need not imply more crimes or more admissions.

### 6.3 Let institutions emerge from demand and political support

Use **capability and incentive conditions**, not dates:

* Repeated unresolved conflicts create demand for dependable arbitration.
* Individuals or coalitions supply adjudication when authority, compensation, prestige, or political control makes it attractive.
* Repeated business supports specialization and record keeping.
* Territorial expansion makes local delegation and review more valuable.
* Budget failure, war, factional conflict, or loss of confidence can fragment the system again.

These are proposed generative rules, not a claim that every real society followed this sequence.

Crucially, demand does not guarantee creation. A ruler may suppress an independent forum; elites may prefer inaccessible justice; potential judges may lack protection. Institutional change needs supporters, resources, and a means of overcoming opposition.

### 6.4 Computational simplification

Run justice **event by event**, not through daily universal legal reasoning. Updates are needed when an incident becomes known, a claim is filed, a session occurs, new evidence appears, a payment falls due, or an enforcement action happens.

Use structured rules and indexed precedents rather than unrestricted text reasoning. A precedent search can retrieve a few cases sharing offense tags, relationship categories, and jurisdiction. Oral transmission can preserve a small, selective set of remembered decisions; writing expands persistence and reach.

For small settlements, adjudicators should usually be agents with other occupations. Even the modern European median staffing rate corresponds to only **about 1.8–8.8 professional judges for populations of 10k–50k**, a derived illustration of scale—not a recommended staffing ratio for early agrarian worlds. [Council of Europe](https://rm.coe.int/cepej-evaluation-report-part-1-en-/1680b272ac)

### 6.5 Existing models and games worth borrowing from

| Model or game | Useful component | What not to assume |
| --- | --- | --- |
| **Dwarf Fortress** | Its official development material describes witness information, interrogations, stolen artifacts, and links among people and organizations. Useful inspiration for evidence-bearing social events. | Its justice mechanics are not a validated reconstruction of historical courts. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2019.html) |
| **Eco** | Authored legal rules and endogenous political decisions demonstrate composable legislation. | Its automatic rule enforcement is precisely what TCE should replace with knowledge, enforcement agents, discretion, and evasion. [Eco](https://www.play.eco/) |
| **Sigmund et al., “Social learning promotes institutions for governing the commons” (2010)** | Models competition between individual punishment and collectively funded punishment institutions, including second-order free riding. | Stylized cooperation payoffs do not supply historical trial procedures or calibrated sentence frequencies. [Nature](https://www.nature.com/articles/nature09203) |
| **Gennaioli–Shleifer legal-evolution models** | Distinguishing precedents, costly legal change, and policy-motivated adjudicators. | Their results depend on assumptions; do not hard-code inevitable improvement through precedent. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/pdf/10.1086/511996?utm_source=chatgpt.com) |

For validation, use paired simulation runs that change one institutional feature: fee relief, additional court sessions, collective liability, automatic capital review, salary security, or release before trial. Compare access, delay, enforcement, distributional effects, and violence—not only conviction counts.

---

## 7. Source strategy, datasets, and remaining uncertainty

### A practical research and calibration stack

| Source | Best use |
| --- | --- |
| **Garfield et al. (2023), “Norm violations and punishments across human societies,” with its open data** | Cross-cultural diversity of sanction types and institutional associations. Presence/absence is not event frequency. [Cambridge University Press](https://www.cambridge.org/core/journals/evolutionary-human-sciences/article/norm-violations-and-punishments-across-human-societies/F5B13F32B188E8A874A333A19E532615) |
| **Old Bailey Online, Digital Panopticon, and Devereaux (2016), “The Bloodiest Code”** | Link accusation, conviction, sentence, and later fate. Account for changing reporting and digitization practices. [The Digital Panopticon](https://blog.digitalpanopticon.org/seeing-things-differently-visualising-patterns-of-data-from-the-old-bailey-proceedings/) |
| **World Justice Project, Global Insights on Access to Justice 2019** | Legal needs, advice, forum use, abandonment, and access barriers. Use country-level data before a global default. [World Justice Project](https://worldjusticeproject.org/our-work/research-and-data/global-insights-access-justice-2019) |
| **CEPEJ, 2024 evaluation using 2022 data** | Court staffing, case flows, and disposition indicators with explicit definitions. [Council of Europe](https://rm.coe.int/cepej-evaluation-report-part-1-en-/1680b272ac) |
| **UN prison statistics and national sentencing statistics such as Destatis** | Separate prison stocks from sanctions imposed and from admissions. [UNSD](https://unstats.un.org/sdgs/report/2025/goal-16/) |
| **V-Dem judiciary indicators and codebook** | Independence, government compliance, and review as separate institutional dimensions. [V-Dem](https://v-dem.net/documents/55/codebook.pdf) |
| **Peters; Davis; Harrison; Le Sage; Anderson** | Islamic criminal law, Hindu legal thought, Qing review, Somali customary institutions, and global penal transportation respectively. These provide institutional mechanisms rather than universal rates. [Cambridge University Press](https://www.cambridge.org/core/books/crime-and-punishment-in-islamic-law/B6DB73FB474660E44B8B1EE27D75770F) |

### Where caution matters most

**Punishment frequencies before comprehensive administrative records remain unevenly documented.** Spectacular executions, severe offenses, and elite disputes are more visible than routine settlements. The correct response is jurisdiction-specific calibration—not assigning invented percentages to whole eras. The Old Bailey project itself emphasizes changes in what proceedings recorded and what their digital markup makes countable. [The Digital Panopticon](https://blog.digitalpanopticon.org/seeing-things-differently-visualising-patterns-of-data-from-the-old-bailey-proceedings/)

**Normative texts are not administrative outcomes.** The uncertainty is often not whether a tariff existed, but who invoked it, how officials interpreted it, whether payment occurred, and which disputes never reached the record.

**Cross-cultural comparisons do not provide a ready-made chronology.** Ethnographic institutions have their own histories; living customary systems should not be treated as untouched specimens of an earlier developmental stage. The comparative punishment dataset is best used to widen the space of possible institutions, not to script their order. [Cambridge University Press](https://www.cambridge.org/core/journals/evolutionary-human-sciences/article/norm-violations-and-punishments-across-human-societies/F5B13F32B188E8A874A333A19E532615)

**Some attractive explanations remain contested or weakly quantified.** Ordeal accuracy, universal deterrence effects, automatic efficiency gains from precedent, and predictable improvements from formalization should all remain hypotheses or parameterized mechanisms, not guaranteed laws of TCE.

### Implementation priority

**Build claims, bargaining, evidence, and enforceable obligations first. Add specialized courts and review as institutions that organize those existing processes.** That architecture allows a village compensation agreement, an imperial capital-case review, and a modern fine-heavy justice system to emerge from the same primitives—while preserving the central historical distinction between what the law says, what authorities decide, and what actually happens.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928c6-f3dc-83e9-87d8-3d9126058ba7)
