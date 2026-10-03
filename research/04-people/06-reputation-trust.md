# Reputation, trust, reciprocity and grievance in TCE

## Executive recommendation

**Build this as a system of private beliefs, social obligations and unresolved disputes—not a single “opinion” score.** An agent should be able to distrust someone’s honesty, respect their competence, remain loyal to them, fear them, and still believe that they owe compensation.

The strongest design implication is to separate three timescales: **immediate emotional activation, durable knowledge about relationships, and socially transmitted accounts of past wrongs.** Evidence supports reputation-dependent cooperation, context-dependent trust repair and intergenerational legacies of violence. It does **not** establish a universal daily decay rate for grievances. Long-term persistence can reflect repeated reminders, continuing losses, institutions and family narratives rather than uninterrupted personal anger. [Wharton Faculty Platform](https://faculty.wharton.upenn.edu/wp-content/uploads/2014/06/Promises-and-Lies.pdf?utm_source=chatgpt.com)

For TCE, I recommend:

> **Events create evidence and claims; people communicate interpretations; relationships influence choices; institutions change the available remedies.**

The empirical results below are calibration targets. The equations, memory budgets and numerical defaults explicitly marked **proposed** are engineering choices, not fitted laws of human psychology.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Keep the important social states separate

Use these distinctions in the simulation schema:

| State | Meaning in TCE | Example |
| --- | --- | --- |
| **Reputation belief** | What this observer believes about another person’s characteristics or conduct | “She usually repays loans.” |
| **Trust** | Expected acceptable behavior in a particular interaction, together with uncertainty | “I will leave my tools with her, but not ask her to negotiate for me.” |
| **Affection and loyalty** | Attachment and willingness to preserve or support a relationship | “He is unreliable, but he protected my family.” |
| **Obligation** | A socially recognized expectation of repayment, assistance or performance | “Her household helped with our harvest.” |
| **Grievance** | A perceived wrongful harm that remains insufficiently addressed | “He took our field, and the council refused to hear us.” |
| **Fear** | Expected harm from resistance, defection or exposure | “I obey the chief because his followers can hurt me.” |

These are proposed implementation categories. Their separation prevents several simulation errors: obedience becoming affection, compensation becoming forgetting, generosity implying competence, and loyalty implying agreement.

### 1.2 Update from interpreted actions, not merely outcomes

An interaction should produce an event containing an action, outcome, circumstances and evidence about responsibility.

For example, three failed deliveries should not necessarily have identical consequences:

* A flood destroyed the shipment: lower confidence in delivery, little evidence of dishonesty.
* The merchant negligently lost it: lower confidence in competence or diligence.
* The merchant sold it elsewhere and lied: lower confidence in honesty, plus a possible compensation claim.

This distinction matters experimentally. Schweitzer, Hershey and Bradlow found that trustworthy subsequent behavior could restore trust after untrustworthy conduct, but recovery was poorer when the original conduct also involved deception. Their result concerns the experimental observation period—not proof that deception makes recovery permanently impossible. [Wharton Faculty Platform](https://faculty.wharton.upenn.edu/wp-content/uploads/2014/06/Promises-and-Lies.pdf?utm_source=chatgpt.com)

**Implementation rule:** calculate perceived responsibility and diagnosticity separately from material loss. A costly accident can create a practical problem without creating the same grievance as intentional betrayal.

Likewise, an inexpensive act can be highly informative: keeping a confidence, refusing an advantageous betrayal, or helping when nobody influential is watching.

### 1.3 Direct reciprocity: remember partners, but allow mistakes and exit

In repeated cooperation, today’s action changes the value of future interaction. In Nowak’s simplified donation-game treatment, direct reciprocity can support cooperation when

\[
w>\frac{c}{b},
\]

where \(w\) is the probability of another encounter, \(c\) is the helper’s cost and \(b\) the recipient’s benefit. This is a theoretical benchmark under specified strategies and assumptions—not a universal behavioral threshold. [ResearchGate](https://www.researchgate.net/publication/6641993_Five_Rules_for_the_Evolution_of_Cooperation)

**TCE rules:**

Maintain a compact history of assistance, fulfilled commitments and exploitation. Make cooperation more attractive when future interaction is valuable, but include alternatives: use another partner, demand collateral, reduce the amount entrusted, or stop interacting.

Avoid literal one-strike tit-for-tat. Distinguish an observed defection from an uncertain failure, and permit renewed cooperation after convincing repair. Otherwise ordinary observation errors can manufacture endless retaliation.

Do not make all assistance an exact debt ledger. Author separate exchange norms: immediate payment, balanced favors, need-based household assistance, hospitality and patronal support. Agents can disagree about which norm applied; that disagreement itself can generate grievance.

### 1.4 Indirect reciprocity: behavior toward others changes treatment of you

Under indirect reciprocity, a person gains or loses cooperation because of how they treated third parties. Nowak’s corresponding simplified benchmark is

\[
q>\frac{c}{b},
\]

with \(q\) representing the probability that reputation is known. Again, this is model-specific. In richer models, assessment rules, mistakes, private information and communication structure change the result. [ResearchGate](https://www.researchgate.net/publication/6641993_Five_Rules_for_the_Evolution_of_Cooperation)

The central implementation question is **what observers consider justified**.

| Observer’s assessment rule | Helping someone considered “bad” | Refusing someone considered “bad” |
| --- | --- | --- |
| **Image scoring** | Positive | Negative |
| **Simple standing** | Positive | Acceptable |
| **Stern judging** | Negative | Acceptable |

These are useful theoretical alternatives, not an evolutionary sequence of moral sophistication. Under stern judging, helping a condemned person can damage the helper’s reputation; under simple standing, it need not. Modern indirect-reciprocity models also show why disagreement over reputations can destabilize cooperation and why gossip can help coordinate assessments. [ResearchGate](https://www.researchgate.net/figure/Assessments-of-the-donor-either-G-or-B-for-good-or-bad-for-different-social-norms_tbl1_380103281)

**TCE rule:** evaluate conduct through the observer’s norms and information. The same assistance can be interpreted as generosity, favoritism or betrayal. Never give every citizen automatic access to the simulator’s authoritative “good person” label.

### 1.5 Gossip: transmit reports, interpretations and source credibility

An experience-sampling study of 309 people documented 5,284 gossip events over ten days. Reports were commonly face-to-face, dyadic and based on first-hand experience; recipients updated reputational judgments and reported different intentions toward the targets. This supports gossip as an ordinary social-information channel, not just malicious rumor. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8487731/)

**Proposed report structure:**

`event reference + target + claim + source + origin + confidence + interpretation`

Transmit reports through actual encounters, household conversation, workplaces, markets, ceremonies and travelers. Select content by relevance, novelty, emotional importance and expected usefulness to the listener.

Crucially, **ten retellings of one allegation are not ten independent observations**. Track provenance sufficiently to discount shared origins. Repetition may increase salience without adding equivalent evidentiary weight.

Allow credibility to be domain-specific: a reliable observer of farm work may be a partisan source on factional disputes. Contradictory reports should produce uncertainty or disagreement, not automatic averaging into objective truth.

### 1.6 Kin, community and strangers: different priors, not different species

Group membership influences cooperation, but the effect is not equivalent to universal hostility toward outsiders. Balliet, Wu and De Dreu’s meta-analysis found greater cooperation with ingroup members, with much of the difference attributable to **ingroup favoritism rather than additional outgroup derogation**. [ResearchGate](https://www.researchgate.net/publication/265648934_Ingroup_Favoritism_in_Cooperation_A_Meta-Analysis)

Nor should early communities be represented as isolated clusters of close relatives. Hill and colleagues’ study of 32 contemporary hunter-gatherer societies found substantial co-residence among people who were not close genetic kin. Contemporary foragers are informative comparisons, not direct observations of prehistory. [Academia.edu](https://independent.academia.edu/BarrySHewlett)

For TCE, distinguish:

**Known-person trust:** accumulated evidence about an individual.

**Community-based expectations:** expectations associated with shared institutions, identities and mutual contacts.

**Generalized trust:** a prior about unfamiliar people.

**Institution-backed willingness to transact:** accepting an interaction because guarantees or remedies limit the downside.

A stranger with a respected guarantor can therefore receive more credit than a dishonest cousin. A distrusted trader can remain commercially usable under advance payment.

### 1.7 Grievance and revenge: perceived responsibility plus unresolved stakes

Create a grievance when an agent believes that a wrongful harm occurred, attributes responsibility, and considers the response inadequate.

A grievance should refer to:

`harm + responsible party + violated expectation + demanded remedy + unresolved portion`

Revenge is only one possible response. The action set should include confrontation, withdrawal of cooperation, public accusation, mediation, compensation demands, legal action, relocation and retaliation.

Ethnographic evidence cautions against treating every dispute as an opportunity for impartial third-party punishment. Fitouchi and Singh’s comparative analysis of Kiowa, Mentawai and Nuer material emphasizes victim- and kin-centered responses, compensation and mediation. It also documents substantial cost-inflicting retaliation, so “customary justice is always restorative” would be equally misleading. [Toulouse Capitole Publications](https://publications.ut-capitole.fr/48547/1/singh_48547.pdf)

**Proposed escalation rule:** retaliation creates a new event and potentially a new grievance. Each side may interpret its own violence as justified redress and the opponent’s response as a fresh wrong.

Collective liability should be an institutional or cultural rule, not automatic kinship physics. Relatives may join revenge, refuse it, pressure the offender to compensate, or contribute to settlement because they share the risks.

### 1.8 Loyalty and faction formation

For TCE, make loyalty a partly independent relationship investment rather than the positive end of grievance.

Protection, repeated assistance, shared commitments and dependable advocacy can increase the value of a relationship. Conflicting loyalties should be possible: kin against employer, religious community against ruler, patron against neighborhood.

**Proposed faction rule:** shared grievance supplies a reason to organize, but faction formation also requires communication, trusted organizers, expected efficacy and tolerable risk. Do not spawn a faction merely because enough agents have high anger scores.

Similarly, a ruler who suppresses complaints may reduce visible unrest while increasing fear and leaving grievances unresolved.

---

## 2. Parameters: empirical targets versus engineering defaults

### 2.1 Empirical calibration targets

“High confidence” below means reasonably strong support for the stated result **in the studied setting**. It does not mean the value can be transferred unchanged to an early agrarian village.

| Quantity | Estimate or range, with units | Source and interpretation | Confidence and portability |
| --- | --- | --- | --- |
| **Amount entrusted in experimental trust games** | Mean **50% of initial endowment**; study-level means **22–89%**; SD across study means **12 percentage points** | Johnson & Mislin’s meta-analysis covered 162 replications and more than 23,000 participants. These are transfer amounts, not probabilities of trusting. [Noel D. Johnson](https://noeldjohnson.github.io/assets/papers/trust-games-a-meta-analysis-paper.pdf) | High for summarized experiments; moderate–low for historical transfer |
| **Amount returned by trust-game recipients** | Mean **37% of amount received**; study-level means **11–81%**; SD **11 percentage points** | Denominator is the recipient’s received transfer, after the experimental multiplier where applicable. Not directly comparable to the sender’s fraction. [Noel D. Johnson](https://noeldjohnson.github.io/assets/papers/trust-games-a-meta-analysis-paper.pdf) | High within the compilation; protocol-dependent |
| **Ingroup advantage in cooperation** | Standardized mean difference **\(d=0.32\)**; 95% CI **0.27–0.38** | Balliet et al. A standardized effect, **not 32 percentage points**. Effects vary across interaction structures. [ResearchGate](https://www.researchgate.net/publication/265648934_Ingroup_Favoritism_in_Cooperation_A_Meta-Analysis) | High for an average experimental tendency; moderate transfer |
| **Observed gossip channels** | **68.4% face-to-face**, **74.9% first-hand source**, **73.7% dyadic** | Dores Cruz et al.; 5,284 sampled reports. Sampling does not justify treating total reports divided by person-days as the actual daily gossip rate. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8487731/) | Moderate; one contemporary population |
| **Third-party dispute responses in Kiowa cases** | Intervention to mediate/stop conflict: **37.4%**; enacted third-party punishment: **6.6%**, among **91 cases** | Fitouchi & Singh. Historical case coding, not a random sample of all everyday disputes. [Toulouse Capitole Publications](https://publications.ut-capitole.fr/48547/1/singh_48547.pdf) | Moderate for this corpus; low universal portability |
| **Historical homicide compensation scale** | **40 head of cattle** in the cited Nuer arrangement | A local, historically situated compensation convention involving kin—not a universal livestock price for homicide. [Toulouse Capitole Publications](https://publications.ut-capitole.fr/48547/1/singh_48547.pdf) | Moderate historical confidence; very low portability |
| **Effect of dispute-resolution training in rural Liberia** | Land disputes resulting in property destruction: **−5.1 percentage points**, from **19%** comparison prevalence, at **one year** | Randomized program evaluation; approximately **27% relative reduction**. Conditional on the reported dispute measure, not a reduction in all violence by 27%. [J-PAL](https://www.povertyactionlab.org/evaluation/community-education-preserve-peace-rural-liberia) | High causal evidence for this intervention; moderate–low transfer |
| **Reconciliation and trust toward former rebels, Sierra Leone** | Estimated increase **7.3 percentage points**, SE **3.6 points**, against **32.8%** control mean | Cilliers et al. Domain-specific trust outcome; not equivalent to a universal increase in generalized trust. [Bilal Siddiqi Research](https://bilalsiddiqi.com/wp-content/uploads/2019/01/cilliers-dube-siddiqi-2016-science-reconciling-after-civil-conflict.pdf) | Moderate for this particular outcome; context-specific |

**How to use these:** reproduce the experimental task or institutional scenario inside a small test simulation. Do not set every citizen’s “trust” to 0.50 because subjects transferred half an endowment. Trust-game behavior combines beliefs with preferences and experimental incentives; the meta-analysis also finds substantial protocol effects. [Noel D. Johnson](https://noeldjohnson.github.io/assets/papers/trust-games-a-meta-analysis-paper.pdf)

### 2.2 Proposed numerical defaults for an initial implementation

These are **sensitivity-analysis ranges**, not empirical estimates. Their empirical confidence is low until TCE is calibrated.

| Parameter | Proposed starting range | Units | Purpose and caution |
| --- | --- | --- | --- |
| Active, detailed directed relationships | **32–128** | Targets per agent | Adaptive capacity; permit exceptions for merchants, leaders and mediators |
| Individually retained salient event references | **64–256** | References per agent | Pin unresolved claims and life-defining events; compress routine interactions |
| Compressed reputation reports | **1–5** | Reports per socially active person-day | Simulation work budget, not an estimate of all human gossip |
| Ordinary hearsay evidence weight | **0.1–0.5** relative to a clear independent observation of **1.0** | Dimensionless | Modify by source reliability, independence and diagnosticity |
| Action, observation and assessment error | Sweep **0–0.20** separately | Probability per relevant operation | Stress-test cooperation; do not merge the three errors |
| Minor emotional activation half-life | Sweep **1–7** | Days | Applies to activation, not the claim or memory’s factual content |
| Severe-event activation half-life | Sweep **7–90** | Days | Include reactivation; this is deliberately a broad tuning range |
| Routine reputation-evidence half-life | Sweep **30–365** | Days | Discount stale predictive evidence; durable exceptional evidence can be exempt |
| Unresolved serious claim expiry | **No automatic expiry** | Event-conditioned | Closure depends on settlement, abandonment, loss of standing or other modeled events |
| Narrative adoption | **0–1**, context-dependent | Probability per meaningful exposure | Do not use a universal inheritance fraction per generation |

Use local subsistence requirements, labor time, livestock and asset values to calculate compensation affordability. Do not convert the Nuer example into a globally fixed “40 cattle = forgiveness” rule.

A low-cost first version should expose relatively few behavioral parameters. Start with evidence weighting, future relationship value, obligation strength, danger sensitivity, remedy credibility and narrative reinforcement. Add further axes only when a specific validation failure requires them.

---

## 3. Variation across eras and world regions

### 3.1 Change social conditions, not an “era trust bonus”

| Setting | Evidence and historical interpretation | Implication for TCE |
| --- | --- | --- |
| **Foragers** | Ju/’hoansi research documents norm enforcement through social discussion and criticism; wider forager comparisons show that co-residence is not exclusively close-kin organization. Neither finding establishes universal peacefulness. [Springer](https://link.springer.com/article/10.1007/s12110-005-1000-9) | Emphasize repeated interaction, sharing expectations, reputation and feasible exit. Allow mobility and partner replacement to compete with retaliation. |
| **Early farming** | Direct measurements of trust, grievance duration and interpersonal beliefs are unavailable for prehistoric first farmers. Archaeology cannot supply a defensible daily grievance-decay coefficient. | Treat the following as **mechanistic hypotheses**: fixed fields and stored wealth increase dispute stakes; inheritance creates continuing claims; lower mobility can make avoidance more costly. Test rather than hard-code these consequences. |
| **Pre-industrial towns and states** | Jha’s South Asian research links medieval Hindu–Muslim economic complementarities and institutions for sharing exchange gains to a sustained legacy of tolerance. This is evidence against “different religion automatically means hostility.” [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/trade-institutions-and-ethnic-tolerance-evidence-from-south-asia/534E0018C1431E7A7615B4FAD26DEB3E) | Support brokers, guarantors, merchant organizations and cross-community interests. Trade should promote cooperation when gains and protection are credible—not automatically on every contact. |
| **Industrializing societies** | Institutional economic analysis emphasizes how formal and informal rules change the costs and feasibility of exchange; this is a mechanism, not evidence of an inevitable rise in personal trust. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.5.1.97&utm_source=chatgpt.com) | Add larger organizations, records, contractual remedies and occupational reputations while retaining kin, neighborhood and patronage networks. Impersonal transactions can expand without everyone becoming more trusting. |
| **Modern societies, including weak-state settings** | Liberia’s dispute-resolution experiment and Papua New Guinea’s adaptations of customary institutions show that contemporary peace can depend on hybrid local arrangements, not simply replacement of custom by centralized law. [J-PAL](https://www.povertyactionlab.org/evaluation/community-education-preserve-peace-rural-liberia) | Let customary and state jurisdictions coexist, cooperate or compete. Technology should alter communication and coercive capacity without guaranteeing legitimacy or reconciliation. |

The Papua New Guinea study is particularly useful for TCE: Wiessner and Pupu analyzed **501 recent wars and 129 customary court sessions**. Destructive changes associated with firearms were followed by adaptations in local peace-making institutions. Institutional responses were neither instantaneous nor predetermined by technology. [Arizona State University](https://asu.elsevierpure.com/en/publications/toward-peace-foreign-arms-and-indigenous-institutions-in-a-papua-/)

### 3.2 Quantitative regional variation: use distributions, not stereotypes

The Global Preferences Survey collected data from more than **80,000 people in 76 countries**, within the 2012 Gallup World Poll. Its regional averages illustrate variation in reported trust and reciprocity. Values below are standard-deviation units relative to the world individual mean—not percentages or behavioral probabilities. [Harvard Business School](https://www.hbs.edu/ris/Publication%20Files/Quarterly%20Journal%20of%20Economics_269d889b-69bb-4412-a7bf-e5dfa7d7bff7.pdf)

| Regional grouping used in the study | Trust | Positive reciprocity | Negative reciprocity |
| --- | --- | --- | --- |
| Western Europe | +0.10 | +0.06 | +0.04 |
| Eastern Europe | −0.07 | −0.02 | +0.10 |
| United States, Canada, Australia | +0.23 | +0.16 | +0.02 |
| South and East Asia | +0.04 | +0.07 | +0.11 |
| North Africa and Middle East | +0.23 | +0.07 | +0.08 |
| Sub-Saharan Africa | −0.33 | −0.34 | −0.11 |
| South America | −0.10 | −0.08 | −0.16 |

Source: Falk et al., Table III. [Harvard Business School](https://www.hbs.edu/ris/Publication%20Files/Quarterly%20Journal%20of%20Economics_269d889b-69bb-4412-a7bf-e5dfa7d7bff7.pdf)

Only **8.2% of measured individual trust variation**, **12.0% of positive-reciprocity variation**, and **7.0% of negative-reciprocity variation** was between countries. Most variation was within countries; measurement noise is included in that decomposition. [Harvard Business School](https://www.hbs.edu/ris/Publication%20Files/Quarterly%20Journal%20of%20Economics_269d889b-69bb-4412-a7bf-e5dfa7d7bff7.pdf)

**Design implication:** cultural context should shift distributions and learned expectations, not assign identical personalities to everyone in a region. These contemporary averages should not initialize prehistoric ancestors.

Measurement also matters. Delhey, Newton and Welzel’s 51-country analysis found that respondents differ in whom they understand “most people” to include. A broad-sounding survey question does not always measure an equally broad radius of trust. [DOI](https://doi.org/10.1177/0003122411420817)

Therefore, measure simulated trust separately toward family, familiar nonkin, neighbors, unfamiliar locals and outsiders. Do not infer generalized trust from willingness to help relatives.

---

## 4. How long grievances persist

### 4.1 Individual repair: evidence for change, not a universal clock

McCullough and colleagues followed **337 recently harmed people over approximately three weeks**. Conciliatory gestures were associated with forgiveness and reduced anger through changes in perceived relationship value and exploitation risk. This supports a model in which repair changes the expected future relationship—not merely one in which time automatically subtracts anger. The study’s three-week window is **not** an estimated grievance half-life. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1405072111)

For TCE, distinguish:

**Cooling down:** immediate activation diminishes.

**Reassessment:** new evidence changes judgments of intent, danger or trustworthiness.

**Settlement:** a recognized claim is satisfied, withdrawn or otherwise closed.

**Forgiveness:** the motivation to retaliate or avoid changes.

These can occur separately. An agent can settle a debt without restoring affection, forgive an insult while remembering it, or stop fighting because defeat is likely.

### 4.2 Reconciliation can improve cooperation without erasing suffering

Cilliers, Dube and Siddiqi studied a reconciliation intervention across **200 Sierra Leonean villages**, surveying **2,383 people**. It improved forgiveness and social-capital outcomes but also worsened measures of psychological health. In a subset observed at both **9 and 31 months**, positive and negative effects persisted. [Bilal Siddiqi Research](https://bilalsiddiqi.com/wp-content/uploads/2019/01/cilliers-dube-siddiqi-2016-science-reconciling-after-civil-conflict.pdf)

**Simulation implication:** do not implement a reconciliation ceremony as “remove all negative memories.” It can change public commitments, social access and expectations while leaving distress or private mistrust.

The converse also matters: an emotionally calmer population may still have unresolved property loss, exclusion or insecurity.

### 4.3 Intergenerational persistence is transmitted and renewed

Lupu and Peisakhin’s study of Crimean Tatar families connects the severity of experiences associated with the **1944 deportation** to political identity and attitudes across **three generations**, surveyed roughly **70 years later**. This is a context-specific legacy, not a universal transmission coefficient. The article should be read with its published 2023 erratum. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ajps.12327)

Nunn and Wantchekon connect historical slave-trade exposure to contemporary mistrust in Africa, considering both inherited beliefs and institutions. Such research supports long-lived historical effects but does not isolate a literal centuries-long emotional memory in individuals. Historical identification and the separation of cultural from continuing institutional mechanisms remain important qualifications. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.101.7.3221)

**Proposed transmission mechanism:**

Children encounter accounts through caregivers, relatives, ceremonies, public commemorations and current disputes. They adopt some combination of factual beliefs, group identification and moral interpretation. Later experiences can reinforce or contradict these.

A child may also inherit a legal compensation claim or lost-property claim. That is different from inheriting the deceased person’s anger.

### 4.4 Use three different persistence processes

| Layer | Recommended persistence | What ends or changes it |
| --- | --- | --- |
| **Emotional activation** | Decays and can be reactivated | Time without reminders, safety, repair, new shocks |
| **Relationship evidence and unresolved claims** | Selective, potentially lifelong | New evidence, compliance, settlement, abandonment, changed relevance |
| **Collective narrative** | Potentially multigenerational, but requires carriers | Transmission failure, reinterpretation, competing identities, changed institutions and experience |

The critical rule is **no automatic permanent ethnic hatred and no automatic generational reset**.

---

## 5. Modeling recommendation for 10,000–50,000 agents

### 5.1 Minimal architecture

Use four cooperating components rather than a large psychological simulation for every person.

**A shared event store** records significant occurrences once: actors, affected people, location, time, resources, witnesses and causal context.

**Sparse directed relationship records** store observer-specific evidence, attachment, loyalty, obligations and fear.

**Dispute objects** store claims, parties, representatives, remedies, rulings and compliance.

**Narrative records** store socially transmitted accounts and which groups or people endorse them.

The authoritative world state and agent beliefs must remain separate. A court can rule incorrectly. An agent can sincerely believe a false allegation. Two groups can remember the same event differently.

### 5.2 A cheap reputation-learning approximation

For a predictive domain such as repayment, maintain positive and negative evidence counts \(a\) and \(b\), with positive prior counts \(a\_0,b\_0\).

A proposed update is:

\[
a' = a\_0+(a-a\_0)2^{-\Delta t/h}+w y
\]\[
b' = b\_0+(b-b\_0)2^{-\Delta t/h}+w(1-y)
\]\[
\hat p=\frac{a'}{a'+b'}.
\]

Here \(y\in[0,1]\) is the interpreted degree of success, \(w\) combines credibility, independence and diagnosticity, and \(h\) is an evidence half-life in days.

This is a **Beta-inspired engineering approximation**, not a claim that people perform Bayesian inference. With correlated reports and changing partners, it is not an exact Bayesian posterior.

Keep uncertainty available to the action selector. “Probably reliable, based on one report” should support a smaller commitment than “probably reliable, based on years of dealings.”

Discounting old evidence should move beliefs toward a prior and reduce confidence. It should **not** close an unresolved theft claim or erase a known killing.

For the first implementation, use a few domains—honesty, task reliability, willingness to help and dangerousness—rather than dozens of traits.

### 5.3 Grievance dynamics and response selection

For each active grievance, retain durable claim state separately from transient activation:

\[
A(t+\Delta t)=
\operatorname{clamp}
\left[
A(t)2^{-\Delta t/h\_A}
+\text{new provocation}
+\text{reminder activation},
0,1
\right].
\]

Then compare available responses by expected benefit, relationship consequences, danger, norms and remedy credibility.

Where a stochastic action hazard is useful, convert it to a timestep probability correctly:

\[
P(\text{action in }\Delta t)=1-e^{-\lambda\Delta t},
\]

with \(\lambda\) in actions per day and \(\Delta t\) in days. Gate it on opportunity and capacity. This is a numerical implementation rule, not an empirical revenge-rate model.

Do not add an unresolved grievance to emotional activation every simulation tick. That would manufacture perpetual rage. Instead, trigger reminders through meaningful encounters, anniversaries, threatened assets, public discussion or renewed noncompliance.

### 5.4 Make institutions resolve cases, not apply mood bonuses

A dispute-resolution institution should expose:

| Component | Necessary state |
| --- | --- |
| **Access** | Who can bring claims; distance, fees, delay, representation |
| **Fact-finding** | Witness access, evidence quality, error and partiality |
| **Authority** | Jurisdiction, constituency-specific legitimacy, enforcement capacity |
| **Remedies** | Compensation, restitution, apology, guarantees, separation, sanctions |
| **Compliance** | Ability to pay, guarantors, installments, monitoring, default |
| **Closure** | Who accepts the outcome, what remains disputed, conditions for reopening |

A useful case lifecycle is:

`claim → hearing/mediation → proposed settlement or ruling → compliance → closure or renewed dispute`

Compensation should transfer actual resources. A fine paid to a treasury does not restore the victim’s missing cattle. An apology should not be accepted automatically from a repeatedly deceptive source. An institution that favors a powerful household can settle one case while damaging its standing with others.

For reconciliation, author several separable outputs: material repair, acknowledgment, future protection and public recognition of closure. TCE can then generate partial successes rather than binary “feud ended” events.

### 5.5 Runtime and memory budget

Make this **event-driven**, with lazy timestamp-based decay.

At 50,000 agents and an average of 64 directed relationship records, there are **3.2 million records**. At an illustrative **64 bytes per compact record**, that is **204.8 MB**. Another 128 memory references per agent at 24 bytes each would require **153.6 MB**.

These are arithmetic budgeting examples, not measured Rust performance. They exclude event storage, indexing, institutions, faction membership, allocator overhead and rendering.

Avoid all-pairs updates. Most work should scale with meaningful events, reports and active disputes. Preserve exceptionally important relationships outside ordinary eviction rules. Aggregate mundane repeated interactions into sufficient statistics rather than retaining every greeting.

For debugging and player legibility, expose a short causal explanation:

> “Refuses unsecured credit: two unpaid deliveries, one independent warning, and no trusted guarantor.”

For factions:

> “Joined the petition because the land claim remains unresolved, the organizer previously helped the household, and retaliation is considered unlikely.”

These explanations should come from actual state and evidence provenance, not invented narrative text.

### 5.6 Existing models and games worth borrowing from

| Model or game | Reusable feature | Limitation for TCE |
| --- | --- | --- |
| **Direct/indirect reciprocity models** | Small, controlled tests of repeated interaction, reputation access, norms and error | Abstract games do not provide a complete model of households, property or political history. [ResearchGate](https://www.researchgate.net/publication/6641993_Five_Rules_for_the_Evolution_of_Cooperation) |
| **Versu — Evans & Short** | Social practices specify roles and available actions; autonomous agents select responses. This suits authored mediation, accusation and reconciliation building blocks. | A social-storytelling architecture, not an empirically calibrated 50,000-person civilization model. [UK Computer Science](https://cs.uky.edu/~sgware/reading/papers/evans2014versu.pdf) |
| **Crusader Kings III: Friends & Foes** | Named house feuds, identifiable provocations, cessation and residual hostility make conflict legible across family history. | Borrow presentation and explicit relationship history, not mandatory event chains or automatic inherited enemies. The developer’s 2022 diary is the relevant design reference. [Reddit](https://www.reddit.com/r/CrusaderKings/comments/x79rz4/pc_dev_diary_106_a_fistful_of_friends_foes/) |

---

## 6. Stylized facts and validation tests

A correct simulation should reproduce **conditional patterns**, not merely a desired average amount of cooperation.

| Pattern to reproduce | Validation experiment |
| --- | --- |
| **Repeated interaction can sustain cooperation where isolated encounters do not.** [ResearchGate](https://www.researchgate.net/publication/6641993_Five_Rules_for_the_Evolution_of_Cooperation) | Hold resources and preferences fixed; vary future encounter opportunities. |
| **Reputation can support cooperation beyond direct partners, but private disagreement matters.** [arXiv](https://arxiv.org/abs/2312.10821) | Vary report access, source overlap and assessment error independently. Check for both cooperative and fragmented outcomes. |
| **Ingroup advantages need not imply active hostility to outsiders.** [ResearchGate](https://www.researchgate.net/publication/265648934_Ingroup_Favoritism_in_Cooperation_A_Meta-Analysis) | Compare help to ingroup members, identified outsiders and unclassified strangers. Do not require outsider aggression to produce preferential help. |
| **Deception can damage repair more than equivalent nondeceptive failure.** [Wharton Faculty Platform](https://faculty.wharton.upenn.edu/wp-content/uploads/2014/06/Promises-and-Lies.pdf?utm_source=chatgpt.com) | Compare matched material losses, then identical repair histories, with and without a prior lie. |
| **Mediation can reduce destructive escalation without eliminating disputes.** [J-PAL](https://www.povertyactionlab.org/evaluation/community-education-preserve-peace-rural-liberia) | Introduce an accessible, credible mediator. Track disputes separately from violence and property destruction. |
| **Public reconciliation and private distress can move differently.** [Bilal Siddiqi Research](https://bilalsiddiqi.com/wp-content/uploads/2019/01/cilliers-dube-siddiqi-2016-science-reconciling-after-civil-conflict.pdf) | Track cooperation, avoidance, claim closure and activation as separate outcomes. |
| **Historical legacies can persist across generations without identical descendants.** [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ajps.12327) | Vary narrative exposure, migration, intermarriage, continuing deprivation and countervailing experiences. |
| **Cross-community cooperation can grow around credible shared gains.** [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/trade-institutions-and-ethnic-tolerance-evidence-from-south-asia/534E0018C1431E7A7615B4FAD26DEB3E) | Give groups complementary economic roles, then vary gain-sharing and protection rather than contact alone. |

Add engineering invariants: repeated copies of one rumor must not create unlimited independent evidence; settlement payments must conserve resources; innocent descendants must not acquire autobiographical memories; and results should remain approximately invariant to timestep changes.

For long runs, monitor the distribution of dispute duration, recurrence after settlement, reputation disagreement, partner replacement and coalition size. A system in which every grievance eventually becomes war is no more credible than one in which every grievance disappears after thirty days.

---

## 7. Sources, datasets and evidence limits

### Most useful resources for calibration

| Resource | Best use |
| --- | --- |
| **Johnson & Mislin, “Trust games: A meta-analysis”** | Benchmark distributions under explicitly recreated experimental protocols. [Noel D. Johnson](https://noeldjohnson.github.io/assets/papers/trust-games-a-meta-analysis-paper.pdf) |
| **Global Preferences Survey; Falk et al., “Global Evidence on Economic Preferences”** | Joint variation in trust, positive reciprocity and negative reciprocity; within- versus between-country heterogeneity. [Harvard Business School](https://www.hbs.edu/ris/Publication%20Files/Quarterly%20Journal%20of%20Economics_269d889b-69bb-4412-a7bf-e5dfa7d7bff7.pdf) |
| **World Values Survey documentation and microdata** | Trust toward different categories of people and institutions; country-wave comparisons with weights and questionnaire checks. [World Values Survey](https://www.worldvaluessurvey.org/WVSDocumentationWV7.jsp) |
| **Dores Cruz et al., “Gossip and reputation in everyday life”** | Communication channels, information sources and reputational consequences in ordinary social settings. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8487731/) |
| **Fitouchi & Singh; Wiessner’s ethnographic research** | Dispute sequences, kin involvement, mediation, compensation and limits of punishment-centered models. [Toulouse Capitole Publications](https://publications.ut-capitole.fr/48547/1/singh_48547.pdf) |
| **Liberia and Sierra Leone intervention studies** | Causal tests of institutional changes, including partial success and unintended consequences. [Chris Blattman](https://chrisblattman.com/documents/research/2014.ImprovingOrder%26PropertyRights.APSR.pdf) |
| **Nunn & Wantchekon; Lupu & Peisakhin** | Long-term historical and family transmission mechanisms, with identification and measurement caveats. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.101.7.3221) |

### Where the evidence remains thin or contested

**Universal numerical psychology is the largest gap.** The literature reviewed here does not supply defensible global values for “anger lost per day,” “trust gained per favor,” or “fraction of a parent’s grievance inherited.” Treat such numbers as explicit model hypotheses.

**Experimental generalization is limited.** Monetary tasks are valuable controlled tests, but protocol-sensitive transfers are not direct measurements of how agrarian households handle food insecurity, land disputes or homicide. Historical ethnographic case collections likewise lack the denominator needed to infer population-wide annual dispute rates. [Noel D. Johnson](https://noeldjohnson.github.io/assets/papers/trust-games-a-meta-analysis-paper.pdf)

**Long-lived effects do not identify a single transmission mechanism.** Family teaching, persistent disadvantage, institutions and renewed political cues can coexist. Do not label every historical association “cultural memory” and stop modeling the material conditions. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.101.7.3221)

**Use corrected research versions.** Both the 2024 mechanistic gossip paper and the intergenerational political-violence paper have published corrections; exact model equations or outcome claims should be taken from the corrected versions. [PNAS](https://www.pnas.org/doi/10.1073/pnas.2420897121)

### Bottom line

**TCE does not need a realistic number for how quickly all humans forgive. It needs realistic reasons why particular people cooperate, distrust, demand redress, remain loyal, settle, or retaliate.**

A sparse system of observer-specific evidence, differentiated relationships, persistent claims, actual communication and imperfect institutions can generate those outcomes without scripting the history—and without pretending that one reputation score explains social life.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92801-ec14-83ea-a021-42b6872dc137)
