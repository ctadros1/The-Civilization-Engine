# Criminology for The Civilization Engine

## Executive recommendation

**Build crime as an encounter-driven process with offense-specific motives, moral constraints, imperfect information, and institutional responses—not as a daily “crime roll” attached to each person.**

The strongest evidence supports responsiveness to opportunities, guardianship, and the perceived likelihood of apprehension. Evidence that increasing already substantial punishments produces large additional deterrence is much weaker. Neither finding implies that everyone is a calculating offender or that punishment never matters. [KiltHub](https://kilthub.cmu.edu/articles/journal_contribution/Deterrence_in_the_Twenty-first_Century_A_Review_of_the_Evidence/6471200)

For TCE, keep four things separate:

**An action occurred → someone considered it wrongful → an authority learned about it → an institution classified and resolved it.**

This distinction lets customary compensation, discriminatory laws, unreported abuse, false accusations, and changes in reporting emerge without confusing them with changes in underlying harm.

The confidence assessments below are my synthesis: **high** means strong evidence for the stated finding in its studied setting; **moderate** means meaningful evidence with identification or generalization limits; **low** means substantial measurement uncertainty. None denotes a universal coefficient valid across civilizations.

---

## 1. Mechanisms: what should cause an agent to offend?

### 1.1 The principal theories explain different stages

These theories are more useful as complementary mechanisms than as mutually exclusive explanations.

| Theory | Implementable mechanism | Strength of evidence and limitations |
| --- | --- | --- |
| **Rational choice / bounded choice** | Compare feasible actions—including work, exchange, asking for help, withdrawal, and doing nothing—using subjective benefits, effort, moral costs, social consequences, and expected sanctions. Use the agent’s information, not the simulation’s omniscient state. | **Strong evidence that incentives sometimes matter; weaker support for fully informed optimization.** Police and legitimate economic opportunities affect offending, but responses differ by offense and context. A utility framework is an organizing device, not proof of a single criminal decision process. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjel.20141147) |
| **Routine activity** | Generate opportunities when a potentially willing actor encounters an accessible target without effective guardianship. Daily routines determine who meets whom, which property is unattended, and who can intervene. | **Strong practical foundation for opportunity modeling.** Cohen and Felson’s original account was primarily macro-level and observational; subsequent place-based interventions strengthen the case that changing situations changes crime. It does not itself explain motivation. [UW Faculty](https://faculty.washington.edu/matsueda/courses/587/readings/Cohen%20and%20Felson%201979%20Routine%20Activities.pdf) |
| **General strain** | Loss, mistreatment, blocked goals, or perceived injustice generate distress. Whether this becomes crime depends on anger, available coping strategies, social support, and whether an offense appears to solve the problem. | **Moderate support for associations and conditional pathways; weaker identification of a unique causal mechanism.** “Poor person becomes criminal” is not an adequate implementation. Strain can also produce withdrawal, migration, help-seeking, or lawful collective action. [OUP Academic](https://academic.oup.com/edited-volume/61796/chapter/546057953) |
| **Social disorganization / collective efficacy** | Neighborhood relationships affect information exchange, supervision, willingness to intervene, and collective problem-solving. Residential instability can disrupt those capabilities. | **Strong neighborhood associations; more limited causal identification.** Sampson, Raudenbush, and Earls linked collective efficacy to lower violence, but their study does not establish a transferable “cohesion coefficient.” Poverty, diversity, or deteriorated buildings should not automatically generate crime. [Crab Archive](https://crab.rutgers.edu/users/goertzel/NeighborhoodsCrimeEarls.html) |
| **Self-control** | Vary attention to immediate rewards, impulsive responding, planning, and consideration of future consequences. Allow development and experience to change these capacities. Keep moral preferences separate. | **Strong predictive association; important parts of the original theory are contested.** Evidence does not justify an immutable trait fixed in childhood, a universal explanation of all offenses, or a literal exhaustible “self-control fuel tank.” [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8095718/) |

**Add social learning.** Associates can transmit justifications, expectations, skills, and reputational rewards. Meta-analytic evidence supports associations between offending and delinquent peers or attitudes, but peer selection and shared circumstances complicate causal interpretation. In TCE, allow both influence and friendship selection rather than assuming every correlation among associates represents contagion. [ResearchGate](https://www.researchgate.net/publication/258490902_The_Empirical_Status_of_Social_Learning_Theory_A_Meta-Analysis)

### 1.2 Use different processes for different offenses

I recommend three initial decision processes.

**Instrumental appropriation:** theft, burglary, fraud, embezzlement, and some robbery. The actor wants resources or another concrete advantage. Relevant variables include lawful alternatives, the subjective value of the gain, access, ownership norms, effort, and anticipated consequences.

**Conflict escalation:** threats, assault, retaliatory violence, and some homicide. Start with an actual interaction or remembered grievance. Possible responses should include ignoring, withdrawing, seeking allies, mediation, compensation, intimidation, and attack. An audience or relationship history can alter the perceived consequences of backing down.

**Coercive or institutional abuse:** repeated exploitation, extortion, abuse within dependent relationships, and misuse of office. Model continuing access and power over another person or resource—not merely a chance encounter with a stranger.

These are proposed architectural categories, not exclusive empirical offender types. The same agent can participate in more than one process.

**Do not equate low self-control with bad morality.** A planned appropriation can require patience and organization; a strongly impulsive person may nevertheless reject theft. Separating goals from the capacity to pursue them follows the broader self-control literature better than one “criminality” attribute. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8095718/)

### 1.3 Let consequences change future behavior

After an event, update more than a crime counter. Proposed updates include the actor’s resources, confidence, perceived apprehension risk, relationships, reputation, access to employment, and knowledge of the target. Victims may change routines, seek protection, move, or seek retaliation.

For repeated burglary specifically, evidence supports including **both persistent target differences and event-dependent risk**. A Long Beach study found that persistent differences among homes alone could not explain the observed timing of repeat victimization. [Springer](https://link.springer.com/article/10.1007/s10940-009-9068-8)

---

## 2. Quantitative evidence: deterrence and behavioral change

### 2.1 Useful estimates—and what they actually measure

These numbers are best treated as **counterfactual validation targets for comparable simulated settings**, not coefficients to paste directly into individual agents.

An elasticity of −0.3 means that a 1% increase in the intervention variable is associated with approximately a 0.3% decrease in the outcome locally.

| Intervention and setting | Quantitative result and units | Interpretation for TCE | Confidence |
| --- | --- | --- | --- |
| **Police deployment: London after the July 2005 attacks** — Draca, Machin & Witt, 2011 | Crime elasticity with respect to police activity approximately **−0.3 to −0.4**: a **10% increase in activity → roughly 3–4% less crime**. | More effective guardianship and police activity should sometimes reduce attempts. This is not a pure elasticity of apprehension probability. | **Moderate–high locally**, limited cross-era transfer. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.101.5.2157) |
| **Fixed police protection: Buenos Aires after the 1994 bombing** — Di Tella & Schargrodsky, 2004 | Approximately **75% less vehicle theft on protected blocks**, relative to controls; little corresponding effect one or two blocks away. | Visible protection can have a large, spatially concentrated effect on a particular offense. Do not extrapolate this to all crime throughout a city. | **Moderate–high locally**; unusually intensive deployment. [Harvard Business School](https://www.hbs.edu/ris/Publication%20Files/Do%20Police%20Reduce%20Crime%20Di%20Tella%20Schargrodsky_d8e0367d-38fd-42c1-a95d-2ead15772e01.pdf) |
| **Adult sentencing threshold: Florida** — Lee & McCrary, 2017 | Their calibrated model implies sentence-length deterrence elasticities **no more negative than −0.13 for young offenders**. | Large changes in potential punishment need not produce large behavioral changes. | **Moderate**; discontinuity evidence plus a model-dependent interpretation, not a universal bound. [Princeton University](https://collaborate.princeton.edu/en/publications/the-deterrence-effect-of-prison-dynamic-theory-and-evidence/) |
| **Suspended punishment following Italy’s 2006 clemency** — Drago, Galbiati & Vertova | The working-paper estimate is approximately **1.24% lower recorded recidivism per additional month of expected sentence**—a relative percentage, not percentage points. | Severity can matter when an additional sanction is salient and personally applicable. Do not extrapolate this relationship linearly to very long sentences or ordinary citizens. | **Moderate–high for this released-prisoner setting**; limited transfer. [IZA Docs](https://docs.iza.org/dp2912.pdf) |
| **Hot-spots policing meta-analysis** — Braga et al., 2019 | **65 studies and 78 tests**: a small average crime-reduction effect. Among studies examining surrounding areas, results generally favored diffusion of benefits over displacement. | Suppression should not mechanically relocate every prevented offense. Some opportunities disappear; some actors desist or choose another activity. | **High for the broad direction**, heterogeneous magnitude. [Springer](https://link.springer.com/article/10.1007/s11292-019-09372-3) |
| **Behavioral intervention: Liberia** — Blattman et al., 2023 | Randomized study of **999 high-risk men**. After ten years, an eight-week therapy intervention reduced antisocial behavior by about **0.20 standard deviations**; therapy plus cash by **0.25 SD**. | Adult behavior need not remain fixed. Changes in habits, identity, and decision-making can persist. This is not a direct coefficient for an abstract self-control stat. | **High internal validity**, uncertain transfer to other populations or historical institutions. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faeri.20220427) |

### 2.2 Certainty versus severity: the important distinctions

Nagin’s review supports a more precise statement than “certainty beats severity”: the most consistent evidence concerns **the likelihood of apprehension**, not every subsequent stage of punishment. It also emphasizes uncertainty about how institutional changes become changes in people’s perceptions. [KiltHub](https://kilthub.cmu.edu/articles/journal_contribution/Deterrence_in_the_Twenty-first_Century_A_Review_of_the_Evidence/6471200)

For TCE, distinguish:

**Opportunity prevention:** a guardian prevents access or interrupts an attempt.

**Deterrence:** anticipated consequences change a choice before the offense.

**Incapacitation:** an actor temporarily cannot reach targets because they are confined, restrained, exiled, or otherwise restricted.

**Post-sanction change:** the experience changes resources, relationships, skills, legitimate opportunities, or future behavior.

These are different causal pathways. A policy that reduces offending through confinement has not necessarily frightened anyone out of offending. Conversely, weak marginal effects of longer sentences do not show that removing all consequences would have no effect. Reviews explicitly distinguish punishment threats from the potentially different effects of punishment experience. [Econometrics Laboratory](https://eml.berkeley.edu/~jmccrary/chalfin_mccrary2017.pdf)

**Implementation implication:** do not give a punishment-severity slider a large automatic crime-reduction bonus. Represent what the punishment does, what agents know about it, and which alternatives remain available.

---

## 3. Variation across eras and regions

### 3.1 Historical homicide: useful benchmarks, unequal evidence

All rates in the following table are **homicides per 100,000 people per year**. They are not interchangeable estimates produced by one standardized measurement system.

| Setting | Estimated rate | Appropriate interpretation and confidence |
| --- | --- | --- |
| European regions, approximately **1200–1450** | Typically **15–60** | Eisner’s historical compilation. **Moderate confidence in the broad magnitude and substantial variation**, lower confidence in any particular local estimate. |
| European compilation, **1500–1549** | **20.1** | Grand average across available local estimates, not a population-weighted continental rate. |
| Same compilation, **1600–1649** | **12.0** | Same qualification. |
| Same compilation, **1700–1749** | **5.5** | Same qualification. |
| Same compilation, **1800–1825** | **3.5** | Same qualification. |
| Same compilation, **1900–1924** | **2.0** | Same qualification. |
| Same compilation, **2000–2012** | **1.0** | A benchmark from this compilation, not the rate for every modern European country. |

These figures come from Eisner’s *From Swords to Words* (2014). The long decline is much more secure than an exact continental rate for a medieval year. Coverage, recording, local population denominators, and regional trajectories differ. **Do not turn the observed decline into an automatic yearly pacification coefficient.** [ResearchGate](https://www.researchgate.net/publication/264402032_From_Swords_to_Words_Does_Macro-Level_Change_in_Self-Control_Predict_Long-Term_Variation_in_Levels_of_Homicide)

A major non-European comparison is **Qing China, 1661–1898**. Chen, Peng, and Zhu estimate annual homicide rates of **0.35–1.47 per 100,000**, rising until the early nineteenth century and declining thereafter. These are important archival estimates, but I would assign **low confidence to direct numerical comparison with European or modern survey-quality series**: record coverage, classification, and undercounting remain consequential. They should not be treated as proof that a generic “Qing-like culture” intrinsically suppresses violence. [IDEAS/RePEc](https://ideas.repec.org/a/eee/exehis/v63y2017icp8-25.html)

For a modern benchmark, UNODC’s 2023 study estimated a **2021 global rate of 5.8**, with **15.0 in the Americas** and **12.7 in Africa**. Asia, Europe, and Oceania were below the global average. These are dated benchmarks, not current-year estimates or culture-specific constants. [United Nations India](https://india.un.org/en/255398-homicide-bigger-killer-armed-conflict-and-terrorism-combined)

### 3.2 Foragers and early farming: avoid false annual rates

There is no defensible universal annual “forager crime rate” in the evidence reviewed here.

Fry and Söderberg examined lethal aggression in **21 mobile-forager societies**. More than half of recorded lethal events involved a single perpetrator, and a large share involved interpersonal or interfamilial disputes, accidents, or executions rather than warfare. Their event classifications are informative; they are **not standardized annual homicide rates**. Contemporary ethnography also cannot simply be projected backward into prehistory. [Åbo Akademi University](https://research.abo.fi/en/publications/lethal-aggression-in-mobile-forager-bands-and-implications-for-th/)

For the ancient Middle East, Baten, Benati, and Sołtysiak assembled evidence from **3,539 skeletons**, spanning roughly **12,000–400 BCE** across seven present-day countries. Their pooled sample included **323 individuals with relevant trauma, about 9.1%**. That is a skeletal-trauma proportion—not annual homicide incidence, not the share murdered, and not an individual yearly probability. The study finds substantial variation rather than a simple monotonic trend. [Diposit Digital UB](https://diposit.ub.edu/bitstreams/db2a3679-c2e3-4a53-bce5-ff4b7b8542b6/download)

Evidence from Japan’s Jōmon period also indicates geographically and temporally restricted violence rather than a uniformly violent prehistoric condition. Sample selection, preservation, and the interpretation of injuries remain important limitations. [PubMed](https://pubmed.ncbi.nlm.nih.gov/27029838/)

### 3.3 Era differences should arise from institutions and opportunities

The following are **recommended changes in modeled conditions**, not scripted stages that every world must follow.

| Social setting | Conditions to represent |
| --- | --- |
| **Mobile foragers** | Portable possessions, repeated face-to-face interaction, food-sharing obligations, kin support, reputation, withdrawal, group splitting, and informal responses to aggression. Avoid assuming either universal peace or universal warfare. |
| **Early farming settlements** | Stored harvests, livestock, boundaries, inherited claims, water access, seasonal deprivation, and more difficulty escaping a conflict by moving. Shared storage and mutual aid should also create protective possibilities. |
| **Pre-industrial towns and states** | Markets, commercial storage, strangers, servants, household production, patrons, customary mediation, courts, watchmen, and unequal access to authority. Kin institutions and formal enforcement may coexist. |
| **Industrial societies** | Wage dependence, more portable manufactured goods, changing work/home schedules, denser transport networks, specialized police and courts, and larger custodial institutions. None should guarantee falling crime. |
| **Modern societies** | Financial and organizational opportunities, mass transport, advanced security, firearms, surveillance, reporting systems, and organized illegal markets. Different offenses may move in opposite directions. |

For TCE’s agrarian starting point, **ownership, storage, relationship obligations, and dispute resolution are more important initial modules than prisons or professional detectives**.

### 3.4 Theft: better modern comparisons, much thinner long-run history

The reviewed sources do not provide a harmonized annual theft series spanning foragers, early farmers, and pre-industrial regions. Surviving prosecutions measure institutional processing as well as offending; they should not be treated as complete theft counts.

Modern victimization surveys provide more defensible benchmarks:

| Source and period | Quantitative benchmark | Unit and warning |
| --- | --- | --- |
| **ICVS, national samples, 2003–2004** | Annual burglary-with-entry prevalence: **Japan 0.9%; Mexico 3.0%**. | Percentage of households experiencing at least one event—not number of incidents. |
| **ICVS, city samples, 2001–2004** | **Hong Kong 0.6%; Rio de Janeiro 1.0%; Lima 6.8%; Maputo 12.6%; Phnom Penh 15.8%**. | Same general prevalence concept, but different survey years and city samples; these are not national estimates. |
| **United States NCVS, 2023** | Other household theft **83.1**; burglary **9.0**; motor-vehicle theft **6.1**. | Victimizations per **1,000 households per year**. These are incidence rates and use NCVS definitions. |
| **England and Wales CSEW, 1995 to year ending March 2024** | Estimated theft incidents fell from **11.6 million to 2.7 million**, approximately **77%**. | A decline in estimated counts, not a population-adjusted rate. Commercial shoplifting is outside this household survey measure. |

ICVS values are from its international comparison table; comparability is useful but imperfect. [UNICRI](https://unicri.org/sites/default/files/2021-06/ICVS2004_05report.pdf) The US figures come from *Criminal Victimization, 2023*. [Bureau of Justice Statistics](https://bjs.ojp.gov/document/cv23.pdf) The England–Wales series provides a long-run survey benchmark; ONS also notes quality limitations affecting the recent survey estimates. [Office for National Statistics](https://www.ons.gov.uk/peoplepopulationandcommunity/crimeandjustice/bulletins/crimeinenglandandwales/yearendingmarch2024)

**Design implication:** household burglary prevalence, theft incident counts, and homicide rates require separate outputs and denominators. An offense-specific model should be capable of large changes in one without forcing equivalent changes in the others.

---

## 4. Stylized facts a credible simulation should reproduce

Treat these as **conditional validation targets for comparable worlds**, not laws that every village and civilization must satisfy.

| Pattern | Evidence and numerical benchmark | Required model behavior |
| --- | --- | --- |
| **Age concentration and desistance** | Classic age–crime curves rise toward a teenage peak and then fall; timing differs by offense, cohort, and measurement. Aggregate curves need not describe each individual’s trajectory. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/449114) | Change routines, supervision, peer influence, stakes, and decision-making through the life course. Do not simply turn criminality off at a fixed birthday. |
| **Homicide is strongly sex-skewed** | UNODC’s 2021 benchmark: approximately **81% of victims and 90% of suspects were male**. This does not describe all crime categories. [United Nations India](https://india.un.org/en/255398-homicide-bigger-killer-armed-conflict-and-terrorism-combined) | A comparable modern world should reproduce the skew through its exposure, conflict, social-role, and behavioral mechanisms—not infer guilt from sex. |
| **A minority accounts for many repeated convictions** | In a Swedish population study, approximately **1% of the population accounted for 63.2% of violent-crime convictions**. [PubMed](https://pubmed.ncbi.nlm.nih.gov/24173408/) | Generate a heavy-tailed distribution of repeated involvement. This is retrospective concentration in convictions, not proof of a detectable “criminal 1%” at birth. |
| **Crime concentrates at micro-places** | Weisburd’s comparisons found **2.1–6.0% of street segments accounted for 50% of recorded crime**. [ProHIC](https://prohic.nl/wp-content/uploads/2020/11/4-LawCrimeConcentrationWeisburd.pdf) | Repeated routines, targets, and guardianship should create hot spots. Match spatial units, exposure, and observation periods before comparing percentages. |
| **Victimization repeats in space and time** | Long Beach burglary data showed excess short-interval repeats and near-repeats. Effects were evident for homes within **100 m**, but not around **3.9–4.0 km** apart. [Springer](https://link.springer.com/article/10.1007/s10940-009-9068-8) | Allow familiarity and local opportunity to matter, alongside enduring vulnerability. These distances are context-specific, not universal radii. |
| **Domestic and relational contexts matter** | In the UNODC benchmark, women constituted about **66% of intimate-partner homicide victims**, despite being a minority of homicide victims overall. [United Nations India](https://india.un.org/en/255398-homicide-bigger-killer-armed-conflict-and-terrorism-combined) | Include household and relationship-specific coercion and violence; random stranger encounters cannot reproduce the victim pattern. |
| **Most offenses are not equivalent reporting events** | US NCVS 2023: police learned of approximately **24.8% of other household thefts**, **42.2% of burglaries**, and **72.4% of motor-vehicle thefts**. [Bureau of Justice Statistics](https://bjs.ojp.gov/document/cv23.pdf) | Separate actual victimization from institutional knowledge. The incentive and ability to report should differ by offense. |

Also allow an agent to be an offender in one event, a victim in another, and a witness in a third. Those are event roles, not permanent population classes.

---

## 5. Recommended implementation for 10k–50k agents

The specification below is a proposed synthesis. Its equations are **engineering choices**, not empirically established universal laws.

### 5.1 Minimal state

| Entity | State worth retaining |
| --- | --- |
| **Individual** | Existing personality and values; offense-specific moral objections; future orientation; current needs and liquidity; grievances; relevant skills; known opportunities; perceived sanction risks; significant event memories. |
| **Relationship** | Trust, dependence, hostility, obligations, shared involvement, willingness to protect or report, and perceived retaliation risk. |
| **Household / property** | Possession and ownership claims; movable resources; access permissions; occupancy; guardianship; physical security; recent victimization and protective responses. |
| **Local institution** | Jurisdiction and law version; resources; personnel; information; case backlog; investigation and enforcement capabilities; available resolutions; errors, favoritism, and legitimacy perceptions. |

Avoid a separate “criminal personality” bundle when TCE already has needs, traits, values, relationships, and memories. Add offense-specific knowledge and beliefs only where those existing systems cannot represent them.

### 5.2 Generate opportunities from daily life

Use the activity system to produce candidate encounters. A proposed processing sequence is:

`routine activity → noticed opportunity or dispute → considered actions → choice → attempt → physical outcome → witnesses/reports → institutional response → memory updates`

An opportunity should require **awareness and access**, not merely the existence of an attractive object somewhere on the map.

Candidate evaluation can be cheap: query nearby entities and a small set of remembered targets, filter for feasibility, then evaluate a bounded number of actions. Do not have every agent inspect every building.

### 5.3 Bounded choice with a moral filter

For an admissible action \(a\), one possible score is:

\[
U\_i(a)=
G\_i(a)+R\_i(a)
-C\_i(a)-M\_i(a)-I\_i(a)
-\sum\_o \hat p\_i(o\mid a)L\_i(o)
\]

Here:

* \(G\) is the subjective resource benefit;
* \(R\) is the perceived status, revenge, or other nonmaterial benefit;
* \(C\) is effort and foregone alternatives;
* \(M\) is moral objection;
* \(I\) is expected informal social cost;
* \(\hat p(o\mid a)L(o)\) represents anticipated adverse outcomes using subjective probabilities.

All terms must use a common utility scale. The value of a stolen food portion should depend on the agent’s needs and alternatives, not only its market price.

Before scoring, some actions can be excluded by strong moral commitments or lack of awareness. Otherwise, a noisy optimizer presented with millions of opportunities may eventually make almost everyone offend.

Choice noise is a tunable representation of incomplete deliberation—not evidence that decisions are literally random. For conflict escalation, use the same framework over responses such as withdrawal, mediation, threat, and attack, with emotional state affecting attention and perceived consequences.

### 5.4 Model actual and perceived enforcement separately

For formal sanctions, decompose the process:

\[
\begin{aligned}
P(\text{sanction}\mid\text{offense})={}&
P(\text{authority learns})\\
&\times P(\text{actor identified}\mid\text{previous stage})\\
&\times P(\text{actor brought under jurisdiction}\mid\text{previous stages})\\
&\times P(\text{adverse ruling}\mid\text{previous stages})\\
&\times P(\text{ruling executed}\mid\text{previous stages}).
\end{aligned}
\]

Every factor is conditional on the preceding stages; this is not an independence assumption. Reporting and proactive observation are alternative ways an authority can learn about an event.

Let actual probabilities emerge from witnesses, social knowledge, travel, staffing, procedures, and capacity. Let **perceived probabilities** update through personal experience and communicated information.

A simple belief update is:

\[
\hat p\_{i,t+1}=(1-\alpha\_i)\hat p\_{i,t}+\alpha\_i s\_t,
\]

where \(s\_t\) is an interpreted signal. Its credibility and selection bias matter: an actor rarely observes a random sample of all offenses and outcomes.

**There are no well-established cross-era numerical values for \(\alpha\), moral-cost weights, grievance decay, or the conversion of one unit of hunger into offending propensity.** Treat these as calibration parameters, disclose their priors, and test sensitivity. The empirical elasticities above constrain the combined model, not any single weight.

To represent sentence length, discount costs through time rather than charging an immediate linear penalty for every nominal year:

\[
L\_i=\sum\_t d\_i(t)c\_i(t).
\]

This allows distant consequences to have different subjective weight from immediate ones without hard-coding “severity never matters.”

### 5.5 Make violence and lethality separate

Generate homicide through the interaction and injury systems:

\[
P(\text{death})=
P(\text{violent encounter})
\times P(\text{dangerous injury}\mid\text{encounter})
\times P(\text{death}\mid\text{injury}).
\]

Intent, weapons, force, intervention, and treatment belong in different stages. A technological change can therefore increase deaths without increasing the number of disputes.

The importance of separating means from motivation is illustrated by UNODC’s 2021 figures: firearms were involved in approximately **75% of homicides in the Americas**, compared with **17% in Europe** and **18% in Asia**. These shares are not estimates of the causal effect of firearm availability, but they demonstrate radically different lethal-event compositions. [United Nations India](https://india.un.org/en/255398-homicide-bigger-killer-armed-conflict-and-terrorism-combined)

### 5.6 Repeats, adaptation, and institutions

For burglary, maintain persistent property attributes and short-lived actor-specific familiarity. A decaying memory or attractiveness term is reasonable, but distinguish its decay parameter from an observed waiting time.

For example, Short and colleagues’ fitted Long Beach model contained repeat-event timescales of approximately **12 days and one year** in elevated-risk states. Those are **model-specific waiting-time scales**, not universal memory half-lives to copy into TCE. [Springer](https://link.springer.com/article/10.1007/s10940-009-9068-8)

Victims and institutions should adapt too. A theft can cause better storage, shared watching, relocation, demands for enforcement, or reduced trust. Otherwise, repeat-victimization feedback can create permanently doomed households.

Institutional responses should include combinations of restitution, compensation, apology, mediation, exclusion, restrictions, and custody according to local rules. Do not assume that every society resolves offenses through a modern police–court–prison sequence.

Make investigations consume time and resources. Cases can remain unresolved, accusations can be mistaken, and different people can receive different treatment. Preserve the underlying event separately so the debugging interface can distinguish actual harm, institutional belief, and legal judgment.

### 5.7 Preserve economic and temporal consistency

Theft transfers existing goods; it does not create production. Its wider costs arise through lost work, damaged property, security expenditure, fear, retaliation, and changed cooperation. Compensation transfers should likewise conserve resources.

For time-based hazards, use a timestep-consistent conversion:

\[
P(\text{event during }\Delta t)=1-e^{-\lambda\Delta t}.
\]

Here \(\lambda\) should represent a process such as an encounter or opportunity-arrival rate under the current circumstances—not a fixed authored “crime rate.”

Use event-triggered updates for encounters and reports, slower updates for beliefs and social conditions, and queues for institutional work. This is an architectural recommendation, not a claim that a particular implementation has already been benchmarked at 50,000 agents.

### 5.8 Handle rare events correctly

Small simulated populations make homicide statistically noisy.

As an **illustrative Poisson calculation**, at one homicide per 100,000 person-years:

| Population | Expected homicides per year | Probability of no homicide that year |
| --- | --- | --- |
| 10,000 | 0.10 | 90.5% |
| 50,000 | 0.50 | 60.7% |

Real violence may cluster more than a Poisson process. The calculation nevertheless shows why an annual target-enforcement mechanism would be inappropriate.

Validate homicide over pooled person-years, many seeds, and long windows. Use more frequent outcomes—disputes, threats, assaults, theft attempts, reporting, and resolutions—to diagnose the mechanisms.

### 5.9 Existing models and a game precedent

**Malleson, Heppenstall, and See’s burglary ABM** combines individual motivation with situational factors in a geographically explicit setting. It is a close architectural precedent for coupling daily activity and residential targets, though its results do not validate a general civilization model. [White Rose Research Online](https://eprints.whiterose.ac.uk/id/eprint/76813/)

**Short and colleagues’ crime hot-spot models** demonstrate how target attractiveness, repeated events, and offender movement can generate and alter spatial concentrations. Their suppression and displacement results are model results, not a guarantee that every real intervention behaves identically. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2840109/)

**Dwarf Fortress** is a useful design comparator for concealed plots, interrogation reports, and incomplete institutional information. Borrow the separation between underlying events and what authorities know—not an assumption that its crime dynamics are empirically calibrated. [Bay 12 Games](https://www.bay12games.com/dwarves/)

### 5.10 Validation should test causes as well as totals

A model can match a homicide rate for the wrong reasons. Test matched worlds in which only one mechanism changes: guardianship, punishment duration, access to aid, target security, mediation capacity, reporting confidence, or weapon lethality.

Evaluate offense-specific totals, attempt/completion ratios, age patterns, repeat involvement, victimization concentration, spatial clustering, and the gap between actual and recorded events.

**Emergence does not mean parameter-free.** Calibrate preferences, information, opportunity recognition, and institutional processes—but do not add a controller that forces each settlement back to a desired crime rate.

---

## 6. Sources, data priorities, and unresolved evidence

### A practical evidence package

| Evidence family | Sources to retain | Main use |
| --- | --- | --- |
| **Deterrence** | Nagin, *Deterrence in the Twenty-First Century*; Chalfin & McCrary, *Criminal Deterrence: A Review of the Literature*; the intervention studies in Section 2. | Distinguish apprehension, punishment, incapacitation, and contextual effect sizes. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/670398) |
| **Historical homicide** | Eisner’s 2003 and 2014 syntheses; Chen, Peng & Zhu’s Qing China study. | Long-run magnitude and variation, with archival comparability warnings. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/652229) |
| **Modern homicide** | UNODC *Global Study on Homicide 2023*, with its underlying definitions and country coverage. | Regional, demographic, relationship, and weapon-specific benchmarks. [United Nations Office on Drugs and Crime](https://www.unodc.org/documents/data-and-analysis/gsh/2023/Global_study_on_homicide_2023_web.pdf) |
| **Victimization and reporting** | ICVS; US NCVS; England–Wales CSEW. | Incidence, prevalence, repeat victimization, and underreporting. Preserve survey scope and denominators. [UNICRI](https://unicri.org/sites/default/files/2021-06/ICVS2004_05report.pdf) |
| **Prehistoric violence** | Baten, Benati & Sołtysiak and their archived dataset; region-specific archaeological studies. | Qualitative regime differences and trauma patterns—not annual crime-rate calibration. [Zenodo](https://zenodo.org/records/8010025) |

For every calibration datum, store the observation period, geography, offense definition, denominator, recording process, sample coverage, uncertainty, and whether it is a descriptive statistic, causal estimate, or fitted model parameter.

The largest unresolved areas for TCE are **absolute premodern theft incidence; comparable violence estimates for small nonstate populations; the dynamics of subjective sanction beliefs; and transferable numerical effects of norms, kinship, mediation, and legitimacy**. The mechanisms are implementable, but their coefficients should remain uncertain and adjustable.

**The central design rule is to simulate opportunities and relationships explicitly, let bounded choices generate attempts, let physical events generate harm, and let imperfect institutions discover and resolve only some of those events. Crime rates should be outputs and tests of that system—not its causes.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92809-50e0-83ea-9db6-f31847211d22)
