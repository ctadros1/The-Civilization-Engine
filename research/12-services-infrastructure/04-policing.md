# Policing and public order: a simulation-ready report for TCE

## Executive conclusion

**Model policing as a combination of social relationships, available labor, information, coercive power, and institutional incentives—not as a building that reduces crime within a radius.**

Historical order did not depend exclusively on professional police. Roman authorities combined soldiers, officials, and local participation; eighteenth-century London mixed public officers with privately motivated investigators; contemporary South Africa demonstrates continued coexistence between state police, community organizations, and commercial security. These are overlapping arrangements, not successive stages that necessarily replace one another. [OUP Academic](https://academic.oup.com/book/6853)

For TCE, the essential distinction is between **what happened, what people know, what authorities record, and what authorities decide**. An institution can suppress violence but tolerate elite theft, recover property without prosecuting anyone, obtain a conviction against an innocent person, or report fewer crimes because victims have stopped seeking help. Modern reporting and clearance statistics measure different parts of this process; neither directly measures all underlying harm. [Bureau of Justice Statistics](https://bjs.ojp.gov/document/cv23.pdf)

The strongest quantitative evidence below concerns particular modern jurisdictions and interventions. **There is no defensible universal officer ratio, historical clearance rate, or “professional policing” effectiveness multiplier established by this review.** Use historical cases to specify mechanisms and modern measurements to test particular configurations—not to assign automatic era bonuses.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Distinguish harmful events, prohibited acts, and disputes

**Recommended TCE rule:** maintain separate classifications for material harm, locally prohibited conduct, and contested obligations.

A stolen grain sack, an unpaid debt, a retaliatory assault, an unauthorized assembly, and an accusation of ritual wrongdoing should not enter an identical “crime” process. The polity’s laws determine jurisdiction, admissible accusations, protected persons, and authorized remedies. The simulation independently retains the underlying event.

This separation is historically important. Charleston’s colonial watch, for example, enforced restrictions on enslaved people’s movement alongside other watch duties. Its success at enforcing those restrictions was not equivalent to providing equal public safety. [Charleston County Public Library](https://www.ccpl.org/charleston-time-machine/medieval-roots-charleston-night-watch)

Generate incidents through ordinary activities and relationships: accessible goods, absent guardians, disputes over obligations, interpersonal grievances, predatory organizations, and perceived opportunities. Do not require a permanent occupational class of “criminals.”

### 1.2 Community enforcement operates through relationships and coordinated action

Boehm’s comparative ethnographic work documents criticism, ridicule, disobedience, ostracism, expulsion, and—in exceptional cases—collective killing as sanctions in some foraging societies. These mechanisms could constrain domineering individuals without a standing police organization. The evidence concerns particular ethnographic societies, not a uniform reconstruction of prehistoric humanity. [Perpustakaan UMA](https://opac.uma.ac.id/repository/0674390318.pdf)

**Recommended TCE rules:**

* Knowledge of misconduct travels along kinship, residential, work, and exchange relationships.
* A sanction becomes credible when enough relevant people will support it—not merely when an abstract reputation score crosses a threshold.
* Kin backing can protect victims, shield offenders, or mobilize retaliation.
* Mediation and compensation can terminate a dispute when the parties and their supporters accept the settlement.

A household’s willingness to intervene should depend on the relationship, expected retaliation, opportunity cost, and confidence that others will participate. This permits both effective collective action and situations in which everyone privately disapproves but nobody moves first.

### 1.3 Specialization emerges from workload, finance, and political interests

Qing county administration illustrates why a small official establishment need not imply negligible enforcement capacity: magistrates depended on clerks, runners, servants, and locally embedded personnel. Those intermediaries supplied knowledge and practical capacity, while also creating opportunities for manipulation. [JSTOR](https://www.jstor.org/stable/j.ctt1tfjc28)

**Recommended TCE rule:** households and governing coalitions establish specialized positions when the expected benefits exceed their perceived costs. Benefits may include safer markets, protection of stores, tax collection, suppression of rivals, or reduced demand on household labor.

Support multiple financing arrangements:

| Arrangement | Direct cost | Incentive problem to model |
| --- | --- | --- |
| Rotating household watch | Time taken from production and rest | Absence, substitution, unequal exemptions |
| Tax-funded officers | Wages, equipment, premises, supervision | Patronage, fiscal shortfalls, competing priorities |
| Merchant or neighborhood subscriptions | Contributions from members | Protection concentrated on subscribers |
| Private retainers and guards | Employer expenditure | Loyalty to the employer rather than general law |
| Fees, recoveries, rewards, fines | Payments linked to actions or outcomes | Neglect of unprofitable cases; fabrication or over-enforcement |

These are proposed institutional options, not a claim that every society used every arrangement.

### 1.4 Patrol effectiveness comes from encounters and deployment

A patrol should affect places it actually visits, people who observe it, and incidents to which it can respond. Fixed posts at gates or markets, scheduled night rounds, escorts, mobile patrols, and investigations consume different kinds of labor.

Experimental evidence distinguishes deployment strategies. The Kansas City experiment found no statistically significant differences in its principal crime and public-attitude measures between tested levels of routine preventive motor patrol. Philadelphia’s targeted foot-patrol experiment, by contrast, found a reduction in violence in treated high-crime locations. Neither result establishes a universal effect for all policing. [Policing Institute](https://www.policinginstitute.org/wp-content/uploads/2015/07/Kelling-et-al.-1974-THE-KANSAS-CITY-PREVENTIVE-PATROL-EXPERIMENT.pdf)

**Recommended TCE rules:** assign officers to explicit tasks and routes. Let potential offenders sometimes notice patrols, change timing, abandon an attempt, choose another target, or proceed regardless. Track displacement rather than assuming every prevented incident disappears.

Response time should be decomposed:

\[
T\_{\text{response}}
=
T\_{\text{discovery}}
+
T\_{\text{notification}}
+
T\_{\text{queue}}
+
T\_{\text{travel}}.
\]

A fast mounted officer does not help if nobody reports the incident until morning.

### 1.5 Investigation is an information-constrained workflow

RAND’s study of American criminal investigation in the 1970s found that many solved cases already contained identifying information at initial reporting. Much investigative work involved locating people, documenting cases, processing evidence, and preparing matters for prosecution—not reconstructing an unknown offender through prolonged detective work. Treat this as evidence from that institutional setting, not as a timeless percentage. [RAND Corporation](https://www.rand.org/content/dam/rand/pubs/reports/2007/R1776.pdf)

**Recommended TCE rule:** create specific leads, not a generic progress bar.

A lead might identify a witness, possession of stolen goods, a known antagonist, an informant, a physical trace, or a potentially linked incident. Each permits particular tasks. Cases with no actionable leads normally wait for new information rather than becoming solvable through unlimited expenditure.

Separate:

**Identification → location → apprehension → hearing or trial → disposition.**

Knowing the offender is not the same as being able to arrest them. An officer may lack force, jurisdiction, political backing, or access to the suspect’s protectors.

Evidence should have provenance and reliability. Several witnesses repeating the same rumor are not several independent observations. Coercion or political pressure can produce a formally acceptable accusation without improving its truth.

### 1.6 Private investigators and thief-takers are useful—and incentive-sensitive

London’s eighteenth-century enforcement arrangements included paid Bow Street runners and other “runners” whose income depended more heavily on rewards. Historical research records both useful investigative activity and reputations for corruption around parts of this system. The public/private distinction was not clean. [London Lives](https://www.londonlives.org/book/chapter6.html)

**Recommended TCE rule:** give private enforcement contracts explicit payment conditions: recovery, identification, capture, conviction, or continuing protection.

Different contracts should produce different behavior. A recovery specialist may negotiate a return without arrest. A conviction bounty may encourage useful investigation, but also pressure to identify a convenient defendant. A guard may deter theft from one warehouse while ignoring the neighboring street.

Do not give private policing an intrinsic efficiency or corruption modifier. Let selection, payment, competition, legal authority, and oversight determine outcomes.

### 1.7 Corruption consists of actions, not one institutional percentage

Model at least distinct possibilities for accepting payment to overlook wrongdoing, extorting payment for a service, protecting associates, diverting payroll or equipment, and falsifying evidence or records.

A useful proposed decision structure is:

\[
U\_{\text{corrupt act}}
=
\text{illicit benefit}
+\text{patron benefit}
-\text{expected sanction}
-\text{expected loss of position}
-\text{personal/social disapproval}.
\]

All terms should be expressed on the agent’s utility scale. Detection depends on witnesses, independent oversight, competing officials, records, and political protection.

**Higher pay should not automatically eliminate corruption.** A study of Ghana’s 2010 police salary doubling found increased bribery extraction on the studied trucking routes rather than a reduction. This is a context-specific quasi-experimental finding, not evidence that raising salaries generally increases corruption. It does show why wages alone are an inadequate reform mechanism. [World Bank](https://www.worldbank.org/en/events/2018/06/12/dime-seminar-do-higher-salaries-lower-petty-corruption-a-policy-experiment-on-west-africas-highways)

### 1.8 Public cooperation and coercive capacity are separate assets

An institution needs sufficient force to carry out decisions, but also information and cooperation. Model confidence separately among social groups and neighborhoods: the same force may be trusted by merchants, feared by laborers, and subordinated to a particular lineage.

Do not attach an automatic trust bonus to a “community policing” policy. Coordinated experiments in Brazil, Colombia, Liberia, Pakistan, the Philippines, and Uganda found no average improvement in the central crime or trust outcomes from the studied programs. Implementation and surrounding institutional constraints matter. [PubMed](https://pubmed.ncbi.nlm.nih.gov/34822276/)

Soldiers should therefore be usable for policing without becoming interchangeable with specialist investigators. Roman evidence demonstrates military involvement in public order; TCE can represent differences through training, local knowledge, command objectives, and permissible force rather than an absolute soldier/police divide. [OUP Academic](https://academic.oup.com/book/6853)

---

## 2. Quantitative parameters and calibration anchors

### Interpretation and confidence

**High confidence** below means a clearly defined measurement or well-documented prescription in its stated setting. **Medium confidence** indicates a context-sensitive estimate, limited observation, or important methodological qualifications. Neither label implies reliable transfer to another century or society.

Distinguish **authorized positions, actual employees, full-time equivalents, officers on duty, and officers performing the relevant task**.

### 2.1 Staffing, watch obligations, and corruption exposure

| Parameter | Value and units | Context and source | Confidence and appropriate use |
| --- | --- | --- | --- |
| Prescribed medieval night watch | **6 men per city gate; 12 per borough; 4 or 6 per town** | England, Statute of Winchester, 1285. Watch prescribed between sunset and sunrise, from Ascension to Michaelmas. [Wikisource](https://en.wikisource.org/wiki/Statute_of_Winchester) | **High as prescription; low as observed staffing.** Not full-time employees or a population ratio. |
| Reported urban night rounds | **3 rounds/night**, around **21:00, 00:00, 03:00** | A traveler’s account of Mughal policing, discussed by Hakeem, Haberfeld, and Verma. [ResearchGate](https://www.researchgate.net/publication/297926142_Police_and_the_Administration_of_Justice_in_Medieval_India) | **Medium–low.** Useful schedule example, not an empire-wide standard. |
| Full-time sworn staffing | **2.41 officers/1,000 residents**, approximately **1:415** | U.S. state and local law-enforcement agencies, 2018; excludes federal personnel. [Bureau of Justice Statistics](https://bjs.ojp.gov/media/67846/download) | **High for this census definition.** Includes functions beyond street patrol. |
| Actual police strength | **1.558 personnel/1,000 residents** | India, State/Union Territory police, 1 January 2020. [Ministry of Home Affairs](https://www.mha.gov.in/MHA1/Par2017/pdfs/par2021-pdfs/rs-24032021/3266.pdf) | **High for reported administrative count.** Remit differs from U.S. categories. |
| Authorized police strength | **1.954 personnel/1,000 residents** | Same Indian return: actual staffing was approximately **79.7%** of sanctioned staffing, calculated from the reported ratios. [Ministry of Home Affairs](https://www.mha.gov.in/MHA1/Par2017/pdfs/par2021-pdfs/rs-24032021/3266.pdf) | **High.** Useful example of vacancies separating budgets from capacity. |
| Police bribery exposure | **28% of respondents with police contact** reported paying a bribe during the preceding 12 months | Global Corruption Barometer—Africa 2019. Country results shown ranged from **3% in Cabo Verde to 75% in DRC**. [Corruption Watch](https://www.corruptionwatch.org.za/wp-content/uploads/2019/07/GCB-Africa-2019-Full-report-WEB.pdf) | **Medium–high as survey evidence.** Not the proportion of corrupt officers or a per-encounter probability. |

India’s Ministry of Home Affairs explicitly states that there is no universal standard or UN recommendation fixing an optimal police staffing level. TCE should likewise avoid a universal “required officers per thousand” rule. [Ministry of Home Affairs](https://www.mha.gov.in/MHA1/Par2017/pdfs/par2021-pdfs/rs-24032021/3266.pdf)

For ancient and early agrarian societies, the central quantitative gap is not merely missing headcounts. It is missing **comparable denominators, duties, attendance, auxiliary labor, and territorial coverage**. A precise-looking ancient ratio assembled from incompatible estimates would be worse than an explicitly uncertain labor budget.

### 2.2 Reporting and clearance rates

The U.S. National Crime Victimization Survey estimated that **44.7% of nonfatal violent victimizations** and **29.9% of household property victimizations** were reported to police in 2023. Its population and offense definitions differ from police-recorded crime systems; it excludes homicide. [Bureau of Justice Statistics](https://bjs.ojp.gov/document/cv23.pdf)

For the following table, *clearance* is an administrative outcome, not necessarily a conviction or correct attribution. FBI rules allow clearance by arrest or specified exceptional means. Annual clearances can also concern offenses recorded in earlier years. All values below are for participating agencies in the **2019 U.S. Uniform Crime Reporting system**. [Federal Bureau of Investigation](https://ucr.fbi.gov/crime-in-the-u.s/2019/crime-in-the-u.s.-2019/topic-pages/clearances)

| Offense | Offenses cleared by arrest or exceptional means | Confidence |
| --- | --- | --- |
| Murder and nonnegligent manslaughter | **61.4%** | High within reporting definition |
| Aggravated assault | **52.3%** | High within reporting definition |
| Rape | **32.9%** | High within reporting definition |
| Robbery | **30.5%** | High within reporting definition |
| Larceny-theft | **18.4%** | High within reporting definition |
| Burglary | **14.1%** | High within reporting definition |
| Motor-vehicle theft | **13.8%** | High within reporting definition |

Source for all table values: FBI, *Crime in the United States, 2019*, clearance statistics. [Federal Bureau of Investigation](https://ucr.fbi.gov/crime-in-the-u.s/2019/crime-in-the-u.s.-2019/topic-pages/clearances)

**Implementation implication:** do not assign one clearance probability to all crimes. Identification opportunities, reporting, priority, offender mobility, evidentiary requirements, and allocated effort must differ.

Do not multiply the 2023 survey reporting rates by the 2019 police clearance rates and present the result as a measured probability of solving all actual crimes. The years, populations, classifications, and counting procedures do not match.

### 2.3 Patrol and intervention evidence

| Intervention or parameter | Quantitative result | Interpretation for TCE |
| --- | --- | --- |
| Routine preventive motor patrol, Kansas City, 1972–73 | **15 beats**, divided among reactive, normal, and roughly **2–3× normal patrol visibility** conditions; no statistically significant differences in principal measured outcomes | Do not make ordinary patrol volume a guaranteed linear crime-reduction factor. Context and experimental limitations matter. [Policing Institute](https://www.policinginstitute.org/wp-content/uploads/2015/07/Kelling-et-al.-1974-THE-KANSAS-CITY-PREVENTIVE-PATROL-EXPERIMENT.pdf) |
| Targeted foot patrol, Philadelphia, 2009 | **60 treated and 60 control hotspots**; approximately **23% lower violent crime relative to controls** over the study period | A calibration case for concentrated patrol in high-violence places, not a permanent universal modifier. [Jerry Ratcliffe](https://www.jerryratcliffe.net/phila-foot-patrol-experiment) |
| Duration of hotspot visits | Koper’s analysis identified roughly **11–15 minutes** as an effective visit duration, with diminishing additional benefit | **Medium confidence and highly context-specific.** A possible modern patrol scheduling test, not a medieval watch constant. [Office of Justice Programs](https://www.ojp.gov/library/publications/just-enough-police-presence-reducing-crime-and-disorderly-behavior-optimizing) |
| Community-policing reform packages | **Six country experiments**, without average improvement in the central crime/trust outcomes | Test implementation pathways; policy labels alone should have no effect. [PubMed](https://pubmed.ncbi.nlm.nih.gov/34822276/) |

### 2.4 Proposed tuning ranges—not historical estimates

These are **engineering sensitivity ranges** for initial experiments. They should remain visibly separate from evidence-derived parameters.

| TCE parameter | Initial test range | Suggested treatment |
| --- | --- | --- |
| Paid order-enforcement staffing | **0, 0.5, 1, 2, 4 FTE/1,000 residents** | Test grid, not an optimal range or era requirement; allow values outside it. |
| Effective annual working time | **1,200–2,400 hours/FTE/year** | Start at **1,800**, then derive from calendars, illness, leave, and attendance. |
| Share allocated to patrol | **0.2–0.7** | Other time goes to guards, escorts, cases, court attendance, administration, and training. |
| Reporting probability for otherwise comparable incidents | **0.2, 0.5, 0.8** | Sensitivity tests only; production behavior should emerge from incentives and relationships. |
| Patrol crew size | **1–4 people** | Test safety, intimidation, coverage, and mobilization trade-offs. |

A crucial staffing calculation is:

\[
N\_{\text{continuously patrolling}}
=
\frac{N\_{\text{employed}}\times H\_{\text{effective annual}}\times f\_{\text{patrol}}}
{8{,}760}.
\]

**Illustrative TCE calculation:** a population of 50,000 with 2 officers per thousand has 100 officers. At 1,800 effective annual hours and 60% patrol allocation, that supports only **12.3 continuously patrolling officer-equivalents**—approximately six two-person patrols averaged across the year.

This arithmetic is not an empirical staffing recommendation. It demonstrates why 100 employees must not appear simultaneously on the streets.

---

## 3. Variation across eras and world regions

The following are institutional examples, not a compulsory development sequence.

| Setting | How order was maintained and investigated | What TCE should preserve |
| --- | --- | --- |
| **Foragers and small mobile communities** | In some ethnographic cases, peer sanctions and coordinated opposition constrained misconduct without permanent police. Arrangements differed substantially between societies. [Perpustakaan UMA](https://opac.uma.ac.id/repository/0674390318.pdf) | High interpersonal knowledge can coexist with weak centralized coercion. Exit, kin support, and collective participation matter. |
| **Early farming settlements** | Evidence rarely supports numerical reconstruction of police staffing or clearance. For TCE, treat protection of stores, livestock, fields, and water access as a **modeling hypothesis about demand for enforcement**, not proof of a universal historical office. | Begin with household obligations, mediation, and occasional guards. Let specialization depend on workload and political organization. |
| **Roman imperial settings** | Soldiers, governors, local officials, and inhabitants all participated in maintaining order; arrangements varied across the empire. [OUP Academic](https://academic.oup.com/book/6853) | A garrison, municipal officials, and local intermediaries can jointly provide enforcement without a modern police department. |
| **Medieval Islamic Egypt** | Lev’s research distinguishes judicial courts, complaint-handling institutions, police, and market regulation. “Justice” was distributed across several authorities. [JSTOR](https://www.jstor.org/stable/10.3366/j.ctv10kmdrv) | Different offices can hear different complaints, with overlapping authority and opportunities for appeal or conflict. |
| **Imperial China** | Qing magistrates depended on locally knowledgeable clerks and runners. Earlier, Song Ci’s **1247** inquest manual demonstrates sophisticated written guidance for investigating deaths. [JSTOR](https://www.jstor.org/stable/j.ctt1tfjc28) | Administrative intermediaries and forensic knowledge are separate capabilities. Neither requires industrialization or European-style policing. |
| **Mughal South Asia** | Urban kotwals employed watchmen and informants; military district authorities addressed larger-scale disorder, while village watch arrangements relied on local support. Sources mix prescriptions with descriptions of practice. [ResearchGate](https://www.researchgate.net/publication/297926142_Police_and_the_Administration_of_Justice_in_Medieval_India) | Distinguish city patrol, village guardianship, and district coercive expeditions. Information can travel through neighborhood representatives. |
| **Medieval and early-modern Europe** | Watch obligations, constables, magistrates, private prosecution, and reward-seeking detection overlapped. London policing was developing substantially before the nineteenth-century metropolitan force. [OUP Academic](https://academic.oup.com/book/47797) | Rotating duties can become paid specialties without instantly replacing older institutions. |
| **Colonial Charleston, North America** | The town watch enforced enslaved people’s curfew and movement restrictions; enforcement duties are documented in late-seventeenth-century legislation. [Charleston County Public Library](https://www.ccpl.org/charleston-time-machine/medieval-roots-charleston-night-watch) | Institutions can enforce status hierarchy and provide unequal protection. This case should not be generalized into a single origin story for all American policing. |
| **Industrial-era Britain and Japan** | London’s Metropolitan Police was founded in **1829**. Japan’s transition from Tokugawa to Meiji government transformed punishment and state authority through a different institutional history. [Met Police](https://www.met.police.uk/SysSiteAssets/media/downloads/force-content/met/careers/careers/transferring-officers-pcdc/metropolitan-police-service-information-pack.pdf?_t_hit.id=Cds_Soh_Web_Models_Media_GenericMedia%2F_9f5cef3e-628d-4e10-b577-e6cb7dfd3701_en-GB&_t_hit.pos=8&_t_id=yGpdaJFC3Qh3I46bznJ2QA%3D%3D&_t_q=bwv&_t_tags=language%3Aen%2Csiteid%3A9a7e26e6-9ba2-42bf-80ec-103507e5aec9%2Candquerymatch&_t_uuid=8LrvLWpKRmyaKT9%2Fiz3SKQ&utm_source=chatgpt.com) | Salaried organization, administrative standardization, and changed jurisdiction should be separable reforms—not one universal modernization package. |
| **Modern African settings** | Baker documents autonomous citizen enforcement, organized community responses, and commercial private security alongside state police in South Africa. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-modern-african-studies/article/living-with-nonstate-policing-in-south-africa-the-issues-and-dilemmas/9A654292BE5DF5164BF406FF76CF841B) | Formal state capacity does not eliminate private or communal provision. Providers may cooperate, compete, or protect different constituencies. |

**Cross-era design principle:** unlock capabilities, not historical labels. A polity might possess professional soldiers, literate court records, and skilled death investigators while retaining rotating neighborhood watches. Another might fund many uniformed officers but lack reliable records, public cooperation, or independent oversight.

---

## 4. Stylized facts a correct simulation should reproduce

### A. Few permanent officers need not mean absence of order

Informal sanctions can operate without professional personnel, while modern systems can retain extensive non-state provision. Therefore a zero-paid-police settlement should not automatically become lawless; its outcomes should depend on social organization and the kinds of conflicts it faces. [Perpustakaan UMA](https://opac.uma.ac.id/repository/0674390318.pdf)

### B. Recorded crime can rise while underlying safety improves

The reporting gap documented by victimization surveys leaves substantial room for changes in official counts without equivalent changes in victimization. [Bureau of Justice Statistics](https://bjs.ojp.gov/document/cv23.pdf)

**Model-derived prediction:** when complaint costs fall or confidence improves, more existing incidents may become visible. Conversely, intimidation or refusal to record complaints can make official statistics improve while harm remains unchanged.

### C. Clearance differs greatly between offense types

The 2019 U.S. contrast between approximately **61% homicide clearance and 14% burglary clearance** is a useful modern validation target. It should emerge from offense-specific information, priorities, and workflows—not from separate arbitrary “difficulty” constants alone. [Federal Bureau of Investigation](https://ucr.fbi.gov/crime-in-the-u.s/2019/crime-in-the-u.s.-2019/topic-pages/clearances)

### D. More staffing can help without benefiting every outcome equally

Research by Chalfin and colleagues finds that larger U.S. police forces reduced homicide while also increasing arrests for some low-level offenses, with unequal burdens across racial groups. A single “policing effectiveness” score would conceal these divergent outcomes. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faeri.20200792)

### E. Deployment matters, and effects are not necessarily permanent

TCE should be capable of reproducing both weak effects from changing routine patrol and meaningful reductions from concentrated hotspot patrol. It should also permit adaptation or loss of deterrence after deployment ends. [Policing Institute](https://www.policinginstitute.org/wp-content/uploads/2015/07/Kelling-et-al.-1974-THE-KANSAS-CITY-PREVENTIVE-PATROL-EXPERIMENT.pdf)

### F. Spatial concentration should emerge—but not be hardcoded

Targets, travel routes, land uses, and recurring contacts can concentrate incidents. However, research re-examining crime concentration shows that sparse counts alone can create apparently dramatic concentration statistics. Compare simulated patterns with an appropriate randomized baseline; do not require “half of crime on 5% of streets” everywhere. [Department of Criminology](https://crim.sas.upenn.edu/working-papers/re-examining-law-crime-concentration-between-and-within-city-evidence)

### G. Security, legitimacy, and institutional success can diverge

A force may improve market security while imposing discriminatory movement controls or protecting an employer’s interests. Historical Charleston and contemporary plural policing illustrate why institutional objectives and beneficiaries must remain explicit. [Charleston County Public Library](https://www.ccpl.org/charleston-time-machine/medieval-roots-charleston-night-watch)

### H. Small settlements produce unstable annual statistics

As an **arithmetic illustration**, a 10,000-person settlement at 10 homicides per 100,000 people per year has only one expected homicide annually. Under a simple Poisson assumption, approximately 37% of years would contain none.

That is not a proposed historical homicide rate. It shows why TCE should validate rare outcomes across many seeds and long periods, rather than interpret one village-year’s clearance percentage as institutional performance.

---

## 5. Modeling recommendation for TCE

### 5.1 Use ordinary agents with institutional roles

An officer, watchman, magistrate, informer, guard, or mediator should remain a normal person with relationships, needs, employment, beliefs, and material constraints.

| Object | Minimum useful state |
| --- | --- |
| **Person** | Relationships, reputation, relevant skills, employer/patron, known incidents, willingness to report or intervene |
| **Enforcement institution** | Jurisdiction, objectives, protected constituencies, authority, funding, recruitment, oversight, evidence rules |
| **Roster and assignment** | Actual attendance, task, location/route, equipment, crew, scheduled relief, remaining work time |
| **Incident** | Simulation-ground-truth participants, actions, location, losses, traces, and potential observers |
| **Case or dispute** | Allegations, known leads, statements with provenance, suspects, assigned tasks, institutional decisions |
| **Sanction or settlement** | Restitution, obligations, confinement, exclusion, punishment, appeal, compliance, collateral consequences |

Do not encode “corrupt police” as a separate species of agent. An otherwise diligent officer might protect a sibling, obey a patron, refuse a dangerous assignment, or accept an occasional bribe. Conversely, an institution with predatory objectives can contain personally honest employees.

### 5.2 Maintain separate truth and knowledge layers

This is the highest-value architectural decision.

```
Incident or accusation
    → observation, discovery, or rumor
    → private/community response or formal complaint
    → institutional recording
    → actionable leads and assigned tasks
    → beliefs about identity and responsibility
    → apprehension, mediation, or adjudication
    → sanction, settlement, dismissal, or unresolved case
    → material, relational, and political consequences
```

An accusation can enter without a corresponding offense. An actual offense can remain unknown. A case can be administratively closed without locating the offender, recovering the property, or correctly attributing responsibility.

Track those outcomes independently. Otherwise the simulation cannot represent wrongful punishment, compensation without prosecution, or fabricated institutional success.

### 5.3 Make investigation consume specific work

For an assigned investigative task, a simple proposed outcome model is:

\[
P(\text{useful result})
=
1-\exp(-\kappa qh),
\]

where \(h\) is officer-hours, \(q\) is a dimensionless measure of actionable lead quality, and \(\kappa\) is a task-specific productivity coefficient per officer-hour.

This is an **engineering abstraction, not an estimated historical law**.

Apply it only to tasks that actually exist. If there is no witness to interview, no trail to follow, and no record to search, additional investigator-hours should not generate information from nothing. New tips or linked discoveries can reopen the case.

Give outputs both usefulness and reliability. An interview can produce a persuasive false allegation; an examination can produce a correct observation that the prevailing court does not accept.

### 5.4 Represent technological and institutional capabilities independently

Recommended capability dimensions include:

**Notification and transport:** messengers, signals, mounts, vehicles, communications networks.

**Memory and identification:** local familiarity, written complaints, searchable registers, standardized descriptions, cross-jurisdiction information exchange.

**Evidence production:** trained inquests, examination tools, laboratories, evidence preservation and processing capacity.

**Administrative capacity:** dependable wages, rosters, supervision, procurement, case tracking, independent review.

**Legal authority:** power to compel attendance, search, detain, mediate, adjudicate, or request outside assistance.

The historical Chinese inquest tradition is a reminder that sophisticated investigative knowledge can precede modern police organization by centuries. [JSTOR](https://www.jstor.org/stable/10.3998/mpub.19945)

### 5.5 Preserve plural authority and unequal coverage

Allow multiple institutions to claim the same event: a household seeks restitution, a merchant association demands recovery, a magistrate considers a public offense, and a ruler treats the incident as disobedience.

For each institution, use an explicit priority function such as:

\[
\text{priority}
=
w\_H(\text{harm})
+w\_V(\text{victim standing})
+w\_R(\text{regime interest})
+w\_S(\text{solvability})
+w\_P(\text{expected payment})
-\text{cost}.
\]

The weights are institutional policy and political outcomes, not universal moral judgments. Keep a separate player-facing account of harm so that favorable official statistics do not erase victims.

### 5.6 Simplify computation, not causal structure

For 10,000–50,000 agents, the recommended implementation is sparse and event-driven:

* Use existing movement and contact systems to generate observation opportunities; avoid all-person comparisons.
* Process gossip and reporting through selected relationship edges and actual encounters.
* Keep detailed cases for serious, contested, or politically consequential incidents; aggregate repetitive low-impact work where individual detail adds little.
* Update assignments and investigations on task events or hourly/daily schedules; evaluate institutional budgets and staffing less frequently.

These are implementation recommendations, not performance benchmarks.

Visible daily life should come from the same assignments: taking over a gate post, making a night round, escorting a complainant, questioning a witness, conveying a prisoner, searching for a named person, attending a hearing, or collecting an assessed contribution.

### 5.7 Existing models and games worth borrowing from

| Precedent | Useful contribution | Important limitation |
| --- | --- | --- |
| **Epstein, “Modeling Civil Violence” (2002)** | Links grievance, perceived legitimacy, local enforcement, and participation in unrest. Useful for crowd and rebellion dynamics. [PubMed](https://pubmed.ncbi.nlm.nih.gov/11997450/) | Not a model of routine investigation, evidence, or private disputes. |
| **Short et al., “A Statistical Model of Criminal Behavior” (2008)** | Agent movement and changing target attractiveness can generate burglary hotspots and repeat patterns. [ResearchGate](https://www.researchgate.net/publication/242451455_A_STATISTICAL_MODEL_OF_CRIMINAL_BEHAVIOR) | Stylized urban burglary setting; not a general theory of all historical offending. |
| **Ensign et al., “Runaway Feedback Loops in Predictive Policing” (2018)** | Demonstrates how enforcement-generated observations can redirect enforcement and amplify biased data. [Proceedings of Machine Learning Research](https://proceedings.mlr.press/v81/ensign18a.html) | Primarily a feedback and allocation model, not a complete policing institution. |
| **Dwarf Fortress, documented justice/intrigue systems** | Named conspirators, interrogation reports, and discovered organizational relationships illustrate persistent institutional knowledge. [Bay 12 Games](https://www.bay12games.com/dwarves/) | A game-design precedent, not empirical validation or a staffing calibration source. |

For a first implementation, prioritize **rostered labor, social reporting, lead-based cases, and explicit institutional objectives**. Advanced forensic detail is less important initially than preventing officers from knowing things nobody has told or shown them.

---

## 6. Sources, evidence limitations, and calibration strategy

### Recommended evidence base

| Source family | Best use | Main caution |
| --- | --- | --- |
| **BJS staffing censuses and National Crime Victimization Survey** | Staffing definitions, workforce composition, reporting gaps, offense-specific victimization | Dated U.S. benchmarks; police counts and victimization surveys measure different populations and processes. [Bureau of Justice Statistics](https://bjs.ojp.gov/library/publications/census-state-and-local-law-enforcement-agencies-2018-statistical-tables) |
| **FBI Uniform Crime Reporting clearance data** | Offense-specific administrative outcomes | Clearance is not conviction, accuracy, or a same-year incident-cohort probability. [Federal Bureau of Investigation](https://ucr.fbi.gov/crime-in-the-u.s/2019/crime-in-the-u.s.-2019/topic-pages/clearances) |
| **Indian MHA/BPR&D administrative returns** | Actual versus authorized staffing and territorial variation | Duties and included personnel differ across jurisdictions and international datasets. [Ministry of Home Affairs](https://www.mha.gov.in/MHA1/Par2017/pdfs/par2021-pdfs/rs-24032021/3266.pdf) |
| **Transparency International/Afrobarometer surveys** | Experienced bribery, institutional perceptions, reporting constraints | Contact-conditioned exposure, survey error, different national contexts; not direct observation of every transaction. [Corruption Watch](https://www.corruptionwatch.org.za/wp-content/uploads/2019/07/GCB-Africa-2019-Full-report-WEB.pdf) |
| **Historical institutional studies** | Fuhrmann on Rome; Ch’ü on Qing administration; Lev on medieval Egypt; Beattie on London; Botsman on Japan | Rich mechanisms but uneven coverage, changing terminology, and limited numerical comparability. [OUP Academic](https://academic.oup.com/book/6853) |
| **Primary prescriptions and investigative texts** | Statute of Winchester; Song Ci’s inquest manual | Evidence of prescribed arrangements and available knowledge—not automatic evidence of compliance or effectiveness. [Wikisource](https://en.wikisource.org/wiki/Statute_of_Winchester) |

### Claims to treat as contested or thin

**Prehistoric reconstruction is especially uncertain.** Ethnographic analogies identify possible mechanisms but do not establish universal institutions, staffing levels, or crime rates for early farmers and foragers.

**Professionalization is not a single measurable treatment.** It can simultaneously change pay, recruitment, territorial control, recordkeeping, political authority, and complaint access. Attribute effects to those components rather than to the label.

**Historical trial and punishment records are selected outputs.** They are valuable for reconstructing accusations, incentives, relationships, and procedure, but cannot by themselves reveal the denominator of all offenses. London Lives is particularly useful for studying those institutional processes. [London Lives](https://www.londonlives.org/book/chapter6.html)

**Modern causal estimates have limited transferability.** A Philadelphia deployment experiment is more informative about concentrated foot patrol than about a village watch; a Ghana highway-bribery study is more informative about one salary intervention than about the entire political economy of corruption.

### Final modeling recommendation

Build public order around **who will act, what they know, what they are authorized to do, whom they serve, and how much work they can actually perform**.

Then evaluate several outcomes separately: harm, recovery, dispute settlement, correct attribution, wrongful punishment, coercive abuse, reporting, response time, and institutional legitimacy. That architecture can generate a cooperative village watch, a predatory guard force, a capable magistrate’s office, or a professional police organization from the same underlying systems—without scripting any of them as history’s inevitable destination.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92931-b97c-83e9-a8ef-337d97e231f0)
