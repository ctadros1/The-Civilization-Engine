# Human needs and motivation for The Civilization Engine

## Executive recommendation

**Use a hybrid of physiological regulation, psychological needs, learned preferences, and persistent goals—not a Maslow hierarchy and not a collection of identical, linearly draining meters.**

For TCE, I recommend four interacting layers:

| Layer | What it represents | Appropriate dynamics |
| --- | --- | --- |
| **Physical condition** | Energy reserves, hydration, sleep pressure, fatigue, temperature exposure, pain | Resource balances, physiological accumulation and recovery, escalating impairment |
| **Psychological experience** | Relatedness, autonomy, competence, perceived security | Responses to ongoing circumstances and events; satisfaction and active frustration tracked separately |
| **Social identity and commitments** | Relationships, reputation, obligations, values, caregiving, belonging to groups | Persistent relationships and memories, socially interpreted experiences |
| **Goals and action selection** | Obtaining food, finishing a roof, raising children, mastering a craft, gaining office | Anticipation, plans, habits, learning, and bounded comparison of feasible actions |

This is a **proposed synthesis**, not a single experimentally validated model. Its components have substantially different evidential strength. Physical requirements can be anchored quantitatively; psychological constructs have useful empirical support; universal rates such as “autonomy falls by 3% per hour” do not follow from the cited research.

The central design distinction is:

> **Needs influence what people value. Their resources, relationships, beliefs, obligations, and institutions determine what they can do about those needs.**

---

## 1. Which models are useful?

### Maslow: useful vocabulary, poor scheduling algorithm

Maslow’s original 1943 paper was more qualified than a rigid pyramid suggests. It acknowledged exceptions, partial satisfaction, and actions serving several motives simultaneously. It also described its framework as a research proposal substantially informed by clinical experience. Its illustrative percentages of need satisfaction were explicitly arbitrary—not measurements suitable for simulation parameters. [Psych Classics](https://psychclassics.yorku.ca/Maslow/motivation.htm)

The stronger reason not to implement a strict hierarchy is empirical. **Tay and Diener’s 2011 study across 123 countries** found that different forms of need fulfillment were associated with different aspects of well-being, with those associations largely independent of whether other needs were fulfilled. Basic needs mattered, but social connection and respect did not become irrelevant simply because material conditions were poor. This is observational evidence, not a set of causal action-selection coefficients, but it makes strict “unlock the next need” rules a poor default. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21688922/)

**TCE rule:** severe physical deficits should usually become increasingly urgent, without setting every social, moral, or long-term motive to zero.

### Self-determination theory: the strongest organizing framework for psychological needs

Self-determination theory, or **SDT**, distinguishes three basic psychological needs:

* **Autonomy:** experiencing one’s actions as voluntary or endorsed.
* **Competence:** experiencing effectiveness and opportunities to develop mastery.
* **Relatedness:** experiencing connection, care, and belonging.

Autonomy is not synonymous with individualism or independence. A person can willingly embrace demanding family obligations; another can experience nominally well-paid work as coercive. SDT also distinguishes externally controlled motivation from goals that become personally endorsed through internalization. [Self Determination Theory](https://selfdeterminationtheory.org/SDT/documents/2000_RyanDeci_SDT.pdf)

A particularly useful refinement is **satisfaction versus frustration**. Having little opportunity for mastery is not the same experience as repeated humiliation. Limited contact is not equivalent to rejection. In Chen and colleagues’ cross-cultural study, need satisfaction and frustration showed distinguishable associations with well-being and ill-being. Their four-country study included **1,051 university students in Belgium, China, Peru, and the United States**—valuable cross-cultural evidence, but not a representative sample of humanity across history. [Self Determination Theory](https://selfdeterminationtheory.org/wp-content/uploads/2015/01/2014_Chen-et-al._need-satisfaction.pdf)

**TCE rule:** a supportive workshop, an indifferent workshop, and an abusive workshop must produce different experiences even when wages and calories are identical.

SDT should not become another exhaustive list of human motives. Its contemporary theoretical treatment discusses the criteria for qualifying something as a basic psychological need and the status of proposed additions. Status, novelty, meaning, and morality should not automatically be assigned the same empirical standing as its three core needs. [Springer](https://link.springer.com/article/10.1007/s11031-019-09818-1)

### Complementary models

| Model | What TCE should borrow | Important limitation |
| --- | --- | --- |
| **Homeostatic reinforcement learning** | Actions can acquire value because they reduce anticipated physiological deficits; learned predictions allow preparation before a deficit becomes critical. | Keramati and Gutkin’s formal model is a computational account of regulation, not a validated complete model of human social life. [eLife](https://elifesciences.org/articles/04811) |
| **Goal-setting theory** | Goal commitment, feedback, perceived attainability, and intermediate milestones; successful people can adopt new goals rather than merely return to equilibrium. | Evidence about performance on specified tasks does not directly determine lifelong ambitions or political aspirations. [Stanford Medicine](https://med.stanford.edu/content/dam/sm/s-spire/documents/PD.locke-and-latham-retrospective_Paper.pdf) |
| **Habit formation** | Repetition in a stable context can reduce deliberation and create persistent routines. | Habit strength is behavior- and context-specific; there is no universal “habit forms in 21 days” constant. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/ejsp.674) |
| **Consumat-style agents** | Combine repetition, imitation, social comparison, and deliberation instead of assuming continuous optimization. | These are useful simulation architectures; reproducing a commons dilemma does not validate every underlying psychological rule. [Groningen Research Portal](https://research.rug.nl/en/publications/behaviour-in-commons-dilemmas-homo-economicus-and-homo-psychologi/) |

**Recommended division of labor:** physiological control handles maintenance; SDT handles the quality of lived experience; relationships and values provide social commitments; goals and habits organize behavior through time.

---

## 2. Simulation mechanisms

The following rules are **implementation recommendations informed by the evidence**, rather than equations directly estimated from a universal human-needs dataset.

### 2.1 Separate objective condition from subjective experience

Do not store only `hunger`, `happiness`, and `social`.

At minimum, distinguish:

**Condition → perception → motivation → feasible action → outcome → memory.**

A person may be physically safe but afraid, hungry but unaware of an available food store, socially surrounded but rejected, or financially poor while strongly connected to others. The simulation should preserve those distinctions.

### Recommended state decomposition

| Domain | State to represent | Update and behavioral rule |
| --- | --- | --- |
| **Food** | Short-term satiety; longer-term energy reserves; nutritional adequacy elsewhere in the health model | Eating changes satiety and supplies energy. Work consumes energy. A recently filled stomach must not erase accumulated malnutrition. |
| **Water** | Water balance and perceived thirst | Intake includes beverages and food; losses depend on activity and environment. Drinking requires a reachable, permitted source. |
| **Sleep and exertion** | Sleep pressure, circadian phase, physical fatigue | Sleep and physical rest are not interchangeable. Sitting can reduce exertional fatigue without fully resolving sleep pressure. |
| **Exposure and bodily distress** | Thermal exposure, wetness, injury, pain | Shelter, clothing, fire, shade, and treatment change actual exposure or symptoms—not an abstract shelter meter alone. |
| **Safety** | Immediate danger; beliefs about future danger and resource reliability | Escape urgent hazards; avoid or mitigate anticipated hazards through route choice, storage, alliances, or migration. |
| **Relatedness** | Recent social experience; durable relationships; rejection events | Contact quality, familiarity, reciprocity, and belonging matter. Mere proximity should not guarantee satisfaction. |
| **Autonomy** | Experienced volition; coercion and blocked-choice events | The same duty can be endorsed by one person and resented by another. Track values, alternatives, and perceived coercion. |
| **Competence** | Task success, skill development, perceived efficacy, persistent failure | Reward meaningful progress and suitable challenges. Keep perceived competence distinct from actual skill. |
| **Respect and status** | Recognition within relevant groups; humiliation; rank where relevant | Distinguish being respected from outranking others. A society should not require half its members to be permanently miserable. |
| **Purpose, identity, and care** | Enduring goals, roles, attachments, dependents, obligations | Generate commitments and plans. Do not make purpose a universal tank that empties every afternoon. |
| **Stimulation and enjoyment** | Activity preferences, recent repetition, opportunities for play or exploration | Optional activities can lose novelty; different people prefer different forms of stimulation. |

For a visible daily-life simulation, elimination and hygiene can be additional routines. **Hygiene should affect exposure, discomfort, and social judgments rather than cause death when an arbitrary meter reaches zero.** Likewise, reproduction and intimacy belong in preferences, relationships, and life-course decisions—not in a universal lethal maintenance meter.

### 2.2 Integrate physical quantities in physical units

For energy reserves:

\[
E\_{t+\Delta t}
=
E\_t+\text{absorbed energy}-\text{energy expenditure}.
\]

A practical task-based expenditure calculation is:

\[
\Delta E\_{\text{task}}
=
BMR\_{\text{day}}\times PAR\_{\text{task}}\times\frac{\Delta t\_{\text{hours}}}{24},
\]

where the task’s physical activity ratio, \(PAR\), expresses expenditure relative to basal metabolism. The daily time-weighted average corresponds to physical activity level, \(PAL\). FAO’s energy-requirement framework provides an appropriate reference structure. [FAOHome](https://www.fao.org/4/y5686e/y5686e07.htm)

Keep the health consequences of prolonged deprivation outside the immediate action scorer. Otherwise a sufficiently attractive celebration can accidentally “compensate” for lethal dehydration in a weighted happiness sum.

For sleep, a cheap starting point is the normalized homeostatic component of the two-process model:

\[
S\_{t+\Delta t}=
\begin{cases}
1-(1-S\_t)e^{-\Delta t/\tau\_w}, & \text{awake},\\[3pt]
S\_t e^{-\Delta t/\tau\_s}, & \text{asleep}.
\end{cases}
\]

A circadian component changes the propensity to fall asleep and wake. Classic parameterizations use approximately **18.2 hours for waking accumulation** and **4.2 hours for sleeping dissipation**, but these are model constants—not required sleep durations or universally fitted individual values. [arXiv](https://arxiv.org/html/1311.1734v3)

### 2.3 Psychological needs should respond to circumstances, not merely elapsed time

For an experience state \(s\_k\in[0,1]\), a useful authored approximation is:

\[
s\_k(t+\Delta t)
=
s\_k(t)+
\left(1-e^{-\Delta t/\tau\_k}\right)
\left[q\_k(t)-s\_k(t)\right].
\]

Here, \(q\_k(t)\) represents the quality of the person’s **ongoing circumstances**.

This distinction is important. A secure family relationship can continue contributing to belonging while its members work separately. A craftsperson can experience competence during routine productive work without requiring a special “gain competence” interaction.

Maintain active frustration separately:

\[
f\_k(t+\Delta t)
=
e^{-\Delta t/\tau\_{f,k}}f\_k(t)
+
\text{new frustrating experiences}.
\]

Coercion, rejection, humiliation, and repeated failure can create such experiences. The separation between satisfaction and frustration is research-informed; **these particular update equations and their time constants are not empirically established**. [Self Determination Theory](https://selfdeterminationtheory.org/wp-content/uploads/2015/01/2014_Chen-et-al._need-satisfaction.pdf)

### 2.4 Score consequences, not immediate meter changes

A suitable conceptual action value is:

\[
Q\_i(a)=
\mathbb{E}\_i\left[
\int\_0^H e^{-\rho\_i t}u\_i(s\_t\mid a)\,dt
+
e^{-\rho\_i H}V\_i(s\_H)
\right]
-C\_{\text{switch}}.
\]

The expectation is based on the agent’s beliefs—not omniscient world state.

The implementation should include travel, waiting, effort, resource consumption, danger, failure probability, obligations, and the consequences of postponing other activities. Compare alternatives over a **common horizon**, including what happens after the candidate action ends. Otherwise brief actions can dominate simply because they complete quickly.

For physical discomfort, an authored convex penalty is useful:

\[
D\_{\text{physical}}
=
\sum\_k w\_{ik}\,[\max(0,d\_{ik})]^{p\_k},
\qquad p\_k>1.
\]

This makes worsening deprivation increasingly urgent without imposing a universal need order. Actual incapacitation and mortality remain physical constraints.

**Example:** a moderately hungry parent may finish feeding a child before eating. A dangerously dehydrated worker should usually abandon an ordinary task to obtain water. A person with no accessible water cannot solve the problem merely by assigning drinking an enormous score.

### 2.5 Give agents commitments and anticipation

A needs-only greedy agent will eat the seed grain, abandon roofs at the first distraction, and never complete a long apprenticeship.

Represent persistent goals with:

* an intended outcome and beneficiary;
* expected benefits, costs, and feasibility;
* intermediate tasks and deadlines;
* commitment strength and conditions for abandonment.

Goal-setting research supports the usefulness of commitment, feedback, and intermediate progress; it also distinguishes pursuing goals from merely reducing immediate discomfort. [Stanford Medicine](https://med.stanford.edu/content/dam/sm/s-spire/documents/PD.locke-and-latham-retrospective_Paper.pdf)

In TCE, planting, child-rearing, saving for tools, seeking recognition, and maintaining a marriage should therefore survive ordinary fluctuations in hunger or mood. Emergencies may interrupt them; an interruption need not erase the plan.

### 2.6 Allow joint satisfaction and prevent repetitive exploits

A shared meal can supply calories, contact, belonging, pleasure, and social recognition during the same interval. Do not require separate actions for each benefit or double-count the elapsed time.

Conversely, apply saturation and context checks. Greeting the same person repeatedly should not provide unlimited belonging; repeatedly viewing the same ornament should not generate unlimited purpose. Optional rewards can habituate, but basic physical resources must not cease to work because they are familiar.

---

## 3. Quantitative parameters

### 3.1 Evidence-backed anchors

**Confidence refers to the stated finding or reference model, not to its transferability into every historical society.** None of the psychological observations below identifies a universal need-decay coefficient.

| Quantity | Value and units | Source and interpretation | Confidence |
| --- | --- | --- | --- |
| **Daily activity multiplier** | Light: **1.40–1.69**; moderate: **1.70–1.99**; vigorous: **2.00–2.40 × BMR** | FAO/WHO/UNU reference categories. For an illustrative BMR of 1,500 kcal/day, this spans about 2,100–3,600 kcal/day. Assign activity from actual time use. [FAOHome](https://www.fao.org/4/y5686e/y5686e07.htm) | High for the reference framework; medium for an unmeasured historical workload |
| **Adult water turnover** | Approximately **1–6 L/day** across the reported adult range; substantial variation | Yamada et al., 2022: **5,604 people in 26 countries**. Turnover is not equivalent to drinking-water requirement: food and metabolic water also contribute. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC9764345/) | High that requirements vary; medium for an individual prediction |
| **Water turnover relative to energy expenditure** | **1.4 ± 0.4 mL/kcal**, adult mean ± SD | Useful as a cross-check, not a universal formula replacing climate, activity, and body-composition effects. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC9764345/) | Medium |
| **Sleep-pressure constants** | Waking \(\tau\_w\approx18.2\) h; sleeping \(\tau\_s\approx4.2\) h | Classic two-process model parameterization. These govern state evolution, not the number of hours an agent should sleep. [arXiv](https://arxiv.org/html/1311.1734v3) | Medium as a modeling starting point |
| **Sleep in three nonindustrial populations** | **5.7–7.1 h asleep/day**; sleep periods **6.9–8.5 h**; about **1 h seasonal difference** | Yetish et al., 2015: Hadza, San, and Tsimane. These are observations in particular contemporary populations, not optimal-sleep prescriptions or universal ancestral values. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26480842/) | Medium; limited population coverage |
| **Acute social deprivation** | **10 h isolation**, **40 participants** | Tomova et al., 2020 found increased social craving after isolation. This demonstrates responsiveness; it does **not** establish a ten-hour refill requirement. [Nature](https://www.nature.com/articles/s41593-020-00742-z) | Medium for the experiment; low for general decay-rate inference |
| **Habit development** | **18–254 days** to model-estimated 95% automaticity | Lally et al., 2010 followed 96 volunteers for 12 weeks, with sufficient data for 82. The long estimates were fitted extrapolations, not 254-day follow-ups. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/ejsp.674) | Medium for context-dependent habit development; low as a universal distribution |

Do not turn observed sleep duration into recommended sleep, water turnover into drinking volume, or an experimental deprivation interval into a psychological time constant.

### 3.2 Explicit TCE calibration priors

The following ranges are **authored starting values proposed here**. Their source is this report’s design synthesis; they are **unfitted and low-confidence**.

For exponential processes, \(\tau\) is an *e-folding time*, not a half-life; half-life is approximately \(0.693\tau\).

| Parameter | Initial exploration range | Units | Intended interpretation | Source/confidence |
| --- | --- | --- | --- | --- |
| Short-term satiety relaxation | **2–6** | Hours, \(\tau\) | Convenience layer for meal-seeking; separate from energy reserves | TCE design prior; unfitted |
| Recent psychological-context smoothing | **1–7** | Days, \(\tau\) | Smooth perceived autonomy, competence, or social experience toward ongoing conditions | TCE design prior; unfitted |
| Ordinary positive/negative affect residue | **1–24** | Hours, \(\tau\) | Prevent instant mood resets after routine events; not a model of grief or mental illness | TCE design prior; unfitted |
| Physical-discomfort convexity | **2–4** | Dimensionless exponent | Increase urgency as normalized deficits worsen | TCE design prior; unfitted |
| Initial individual salience variation | **0.5–2.0 ×** population baseline | Relative multiplier | Seed heterogeneity without assuming identical priorities | TCE design prior; unfitted |
| Routine strategic goal review | **7–90** | Days | Weekly-to-seasonal reconsideration, with event-triggered exceptions | TCE design prior; engineering choice |
| Routine action reconsideration | **5–30** | Simulated minutes | Reconsider only when appropriate; commitments and events override this cadence | TCE design prior; engineering choice |

**Do not assign a universal decay constant to trust, purpose, grief, moral commitment, or reputation.** Start with event-driven changes and persistent context. Add generic forgetting only where it solves a demonstrated modeling problem, and test its consequences separately.

Fit these priors against multiple outcomes simultaneously: time use, unmet needs, completed projects, relationships, consumption, and response to shocks. Matching a single happiness average is insufficient.

---

## 4. Scarcity, individual differences, and historical variation

### 4.1 What people prioritize under scarcity

**Material needs become more pressing, but people do not become pure calorie maximizers.** Banerjee and Duflo’s analysis of poor households in 13 countries reported food expenditure shares of roughly **56–78% in rural samples and 56–74% in urban samples**. These figures use historical poverty definitions and survey settings, not present-day universal thresholds. They establish competing demands on budgets, not that every remaining purchase expresses a freely chosen psychological preference. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2638067/)

**Liquidity constraints can look like changing preferences.** Carvalho, Meier, and Wang’s randomized survey-timing study around payday found greater present bias in monetary choices before payday, but not in real-effort choices. It found no corresponding differences in cognitive performance, risk-taking, decision quality, or heuristic judgments. A money-choice change therefore should not automatically become a permanent change in patience or intelligence. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20140481)

**Cognitive effects of scarcity are context-dependent.** Mani and colleagues reported impaired cognitive performance under financial concerns and among the same farmers before versus after harvest. Together with the payday study’s null cognitive results, this argues for testing temporary, situation-dependent burdens—not encoding poverty as a fixed intelligence penalty. [PubMed](https://pubmed.ncbi.nlm.nih.gov/23990553/?utm_source=chatgpt.com)

**Hunger does not automatically eliminate cooperation.** Häusser and colleagues’ four studies, totaling **795 participants**, found that acute hunger did not consistently undermine prosocial behavior; effects depended on the decision setting. Short-term hunger experiments do not establish behavior during prolonged famine, but they reject a simple universal “hungry means selfish” rule. [Nature](https://www.nature.com/articles/s41467-019-12579-7)

For TCE, represent scarcity primarily through **resource access, uncertainty, opportunity costs, and the expected consequences of delay**. Proposed coping actions include drawing down stores, borrowing, seeking relatives’ help, changing work, postponing investment, selling assets, or migrating. Theft and conflict should compete with those alternatives under specific beliefs and institutions—not appear automatically when hunger crosses a threshold.

Area-wide crop failure should also differ from one household losing its harvest. A local sharing network can redistribute an idiosyncratic loss; it cannot manufacture food when everyone’s supply fails. That is a resource-accounting constraint, not a personality change.

### 4.2 Individual differences: separate preferences from circumstances

Use separate fields for:

| Component | Examples | Why separation matters |
| --- | --- | --- |
| **Physiology** | Body size, age, health, workload | Different consumption requirements are not different moral priorities. |
| **Preferences and values** | Sociability, achievement, risk tolerance, care, tradition | These change the attractiveness of outcomes. |
| **Beliefs and expectations** | Trust in a granary, perceived danger, expected success | Two equally hungry people can make different choices because they predict different outcomes. |
| **Capabilities and access** | Skills, wealth, mobility, rights, relationships | An unavailable action should not be treated as a rejected preference. |
| **Commitments** | Children, debts, promises, occupational roles | Obligations can outweigh immediate personal comfort. |
| **Experience** | Learned routines, successes, losses, coercion | Behavior should change without constantly rerolling personality. |

Falk and colleagues’ **Global Preferences Survey**, covering about **80,000 people in 76 countries**, found that country membership accounted for only about **10% of measured preference variation**. Most variation was within countries, although measurement error is part of that variation. This strongly cautions against giving everyone in a region the same economic personality. [Harvard Business School](https://www.hbs.edu/ris/Publication%20Files/Quarterly%20Journal%20of%20Economics_269d889b-69bb-4412-a7bf-e5dfa7d7bff7.pdf)

For TCE, sample individuals hierarchically—community, household, individual—and let experience modify beliefs and habits. Do not make geography an immutable personality preset.

Likewise, keep **need satisfaction** distinct from **stated preference for a need**. Chen and colleagues found that need-strength measures did not explain away the associations between fulfillment and well-being. “This citizen says autonomy is unimportant” should not automatically imply immunity to coercive treatment. [Self Determination Theory](https://selfdeterminationtheory.org/wp-content/uploads/2015/01/2014_Chen-et-al._need-satisfaction.pdf)

### 4.3 Differences across eras and world regions

**Change constraints, satisfiers, and institutions more aggressively than the underlying biological architecture.** Evidence for historical workloads and health is much stronger than evidence for precise ancient psychological weights.

| Setting | Evidence and limits | Representation in TCE |
| --- | --- | --- |
| **Foraging and mixed subsistence: Africa and the Americas** | Hadza, San, and Tsimane sleep observations show that nonindustrial sleep need not begin at sunset and varies seasonally. Contemporary communities are not direct measurements of prehistoric populations. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26480842/) | Let environment, activity, social schedules, and season influence sleep opportunities. Do not impose one “forager routine.” |
| **Foraging–farming transitions: Philippines** | Among contemporary Agta, greater agricultural engagement was associated with reduced leisure, especially through changes in women’s work. This is a particular comparison, not proof that every farming transition had the same effect. [Nature](https://www.nature.com/articles/s41562-019-0614-6) | Track subsistence, domestic labor, care, travel, and leisure separately. Examine who bears additional work. |
| **Early farming: southwest Asia** | Çatalhöyük, approximately **7100–5950 BCE**, provides evidence of changing disease burdens, workload, mobility, and density through its occupation. Skeletal indicators do not directly reveal satisfaction or motivation. [PubMed](https://pubmed.ncbi.nlm.nih.gov/31209020/) | Settlement growth can alter health, labor, crowding, and support networks simultaneously. |
| **Early farming: China** | At Yangshao-period Baligang, approximately **4000–3000 BCE**, researchers interpreted skeletal changes as declining mobility/workload while dental caries increased. This is an important counterexample to a single global farming trajectory. [Anthropol](https://www.anthropol.ac.cn/EN/abstract/abstract2252.shtml) | Permit labor-saving change and worsening dietary health to coexist. Technology should not move every welfare variable in one direction. |
| **Pre-industrial societies across regions** | Cross-cultural datasets provide evidence about subsistence, residence, kinship, and social organization, but not calibrated hourly motivation curves. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) | Vary crop calendars, storage, rights, household organization, obligations, and routes to recognition. Avoid a universal “medieval European” baseline. |
| **Industrialization** | Thompson’s historical account emphasizes changes from task-oriented work toward clock discipline in British industrial capitalism. It is a regional historical analysis, not a universal timetable for industrial societies. [OUP Academic](https://academic.oup.com/past/article-abstract/38/1/56/1454624) | Model shifts, punctuality sanctions, wages, commuting, and control over work schedules. Do not introduce new biological needs at an industrial technology threshold. |
| **Modern societies** | Global values and preferences surveys expose substantial variation in trust, religion, priorities, and economic preferences; modern surveys do not directly identify ancient values. [World Values Survey](https://www.worldvaluessurvey.org/) | Use these data to test heterogeneity and institutional effects, not to assign timeless regional personalities. |

The same underlying desire for competence might be expressed through tracking animals, weaving, irrigation management, scholarship, factory work, or engineering. **The available paths and their social interpretation are historical; the simulation need not invent a new competence need for each era.**

---

## 5. Stylized facts and validation targets

A plausible motivation model should reproduce **joint patterns**, not merely keep average need meters above a threshold.

| Target | Empirical anchor | Test |
| --- | --- | --- |
| **Work changes resource requirements** | FAO activity categories span roughly **1.4–2.4 × BMR**. [FAOHome](https://www.fao.org/4/y5686e/y5686e07.htm) | Heavy work should change food demand and feasible daily activity, rather than only produce a mood penalty. |
| **Water demand is heterogeneous** | Measured turnover varies substantially with activity, environment, and body characteristics. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC9764345/) | Hot, strenuous work should not have the same water budget as resting in mild conditions. |
| **Sleep is rhythmic but not identical** | The three-population study found **5.7–7.1 hours asleep**, with seasonal differences. [PubMed](https://pubmed.ncbi.nlm.nih.gov/26480842/) | Match comparable scenarios without forcing every citizen to sleep simultaneously or declaring the shortest duration optimal. |
| **Material deprivation does not erase social experience** | Associations between fulfillment and well-being persisted across different needs in **123 countries**. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21688922/) | Poor citizens should retain relationships, values, pride, and commitments. |
| **Poor-household budgets are not 100% food** | Reported food shares were approximately **56–78%**, depending on sample. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2638067/) | Compare consumption composition under matched prices and obligations; do not force every society toward those percentages. |
| **Routine formation is heterogeneous** | Habit-development estimates ranged **18–254 days** in the cited study. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/ejsp.674) | Repeated contexts should yield routines at varying rates; a single missed repetition should not necessarily erase them. |
| **Scarcity effects are not universal personality changes** | Payday differences appeared in monetary impatience without corresponding cognitive or real-effort differences. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20140481) | Distinguish liquidity, task context, beliefs, and trait preferences. |

Add adversarial **engineering tests**, clearly separate from empirical targets:

A hungry agent must not improve nutrition by praying. A coercive institution must not become universally beloved because it provides good meals. Citizens must sometimes finish valuable projects despite minor discomfort. A household must not consume seed grain merely because planting has delayed benefits. A social network must not create food through circular transfers.

Finally, compare detailed and accelerated simulation modes. Daily aggregation should preserve consumption, sleep, exposure, completed work, and relationship outcomes—not merely reproduce the same end-of-day happiness average.

---

## 6. Recommended architecture, institutions, and game precedents

### 6.1 A practical agent loop

Use a layered decision process:

```
Integrate physical state and ongoing task consequences.
Update beliefs and salient experiences from new events.
Check emergencies, incapacity, and task interruption conditions.
Continue a viable commitment, or generate a small feasible action set.
Compare predicted consequences; select with bounded variation.
Reserve resources and execute.
Update outcomes, memories, relationships, skills, and habits.
```

Action definitions should expose preconditions, duration, resource requirements, exertion, expected consequences, participants, permissions, and interruption rules. These become TCE’s authored vocabulary; circumstances determine which actions matter.

Use persistent preferences or modest choice noise rather than rerolling a wholly different personality at every decision. Preserve task continuation unless circumstances materially change.

### 6.2 Institutions should alter opportunities and expectations

For emergent institutions, the motivation system needs more than individual utility scores.

| Institution | Mechanisms to implement |
| --- | --- |
| **Household** | Shared resources, caregiving, relationships, bargaining, and unequal access where applicable. Do not assume household income is distributed equally. |
| **Granary, credit, or mutual aid** | Actual stocks, contributions, claims, eligibility, repayment, rationing, and reliability. Perceived security should depend on credible access. |
| **Workshop or apprenticeship** | Productive tasks, instruction, feedback, progression, compensation, and control over methods. Work can provide competence and belonging as well as income. |
| **Association, festival, or religious group** | Coordination of participants, shared meanings, obligations, recognition, and inclusion or exclusion. Benefits depend on beliefs and relationships. |
| **Government and law** | Enforcement, protection, extraction, procedural treatment, and access to dispute resolution. Material provision, fear, endorsement, and loyalty should remain distinguishable. |

A proposed emergence mechanism is:

**recurrent problem → proposed arrangement → recruitment and bargaining → commitments and contributions → performance → revised trust and participation.**

For example, repeated storage losses might motivate a shared granary, but adoption should depend on expected protection, contribution costs, control of access, and confidence in its managers. There should be no automatic “population reaches 300, therefore government appears” rule.

Needs help explain why people seek arrangements. They do not, by themselves, specify how coalitions negotiate or institutions persist. TCE still needs explicit coordination, enforcement, and social-learning mechanisms.

### 6.3 What to borrow from existing games

These are **design precedents, not empirical validation**.

| Game/model | Useful mechanism | What not to copy as human science |
| --- | --- | --- |
| **The Sims** | Historical Maxis design documents describe objects advertising the motive consequences of interactions, with action selection considering current motives, distance, and availability. This is a strong precedent for data-authored action affordances. [Don Hopkins](https://donhopkins.com/home/TheSimsDesignDocuments/Ch21-Happy.pdf) | Prototype motive scales and compressed timing are gameplay parameters. Do not make possessions the universal solution to purpose, respect, or belonging. These documents describe historical designs, not verified current Sims internals. |
| **RimWorld** | Its official description combines needs, mood, relationships, backgrounds, health, and preferences; Ideology adds belief-dependent practices, roles, and rituals. Borrow the distinction between material conditions and how individuals interpret them. [RimWorld](https://rimworldgame.com/) | The storyteller deliberately organizes dramatic events. Neither its event pacing nor dramatic behavioral outcomes should be treated as an empirical model for TCE’s unscripted history. |
| **Dwarf Fortress** | The 2015 developer log describes personality-dependent needs, actions satisfying several needs, and separate effects involving stress and focus. This supports individualized satisfiers and consequences beyond a single happiness number. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2015.html) | Fantasy-specific values, need timers, and skill effects are not human parameters. Borrow architecture, not numeric tuning. |
| **Consumat** | Multiple decision modes—habit, imitation, comparison, deliberation—provide a computationally useful alternative to perpetual optimization. [Groningen Research Portal](https://research.rug.nl/en/publications/behaviour-in-commons-dilemmas-homo-economicus-and-homo-psychologi/) | Treat its scenario results as model results, not proof that its decision rules apply universally. |

### 6.4 Scaling to 10k–50k people

Use compact hot-state arrays, sparse relationship storage, cached routines, and event-triggered reconsideration. Sleeping citizens and people successfully executing long tasks should not repeatedly score every possible action.

As an engineering starting point, shortlist perhaps **8–32 feasible actions** when a decision is actually required. That range is a performance-design choice, not a psychological finding.

The raw storage for 32 single-precision values across 50,000 agents is approximately **6.4 MB**, before relationships, plans, histories, indexing, and allocation overhead. The scalar needs vector is therefore unlikely to be the main scaling problem; unconstrained searches, pathfinding, and all-pairs social evaluation deserve closer attention. This is a storage calculation, not a throughput benchmark.

Keep rendering independent from decision frequency. Animation does not require continuous replanning.

---

## 7. Sources, datasets, and remaining uncertainty

The cited literature supports the constructs and empirical anchors above. For calibration, prioritize datasets that expose behavior and circumstances rather than only global happiness scores.

| Source or dataset | Useful variables | Main caution |
| --- | --- | --- |
| **Basic Psychological Need Satisfaction and Frustration Scale** | Separate satisfaction/frustration measures for autonomy, competence, and relatedness | Questionnaire scores are not utility weights or hourly decay rates. [Self Determination Theory](https://selfdeterminationtheory.org/basic-psychological-need-satisfaction-and-frustration-scale/) |
| **Global Preferences Survey; Falk et al., 2018** | Patience, risk-taking, reciprocity, altruism, trust | Useful for heterogeneity; does not directly measure all TCE needs. [Harvard Business School](https://www.hbs.edu/ris/Publication%20Files/Quarterly%20Journal%20of%20Economics_269d889b-69bb-4412-a7bf-e5dfa7d7bff7.pdf) |
| **World Values Survey** | Values, trust, religion, attitudes, reported well-being | Modern survey responses require contextual interpretation. [World Values Survey](https://www.worldvaluessurvey.org/) |
| **Multinational Time Use Study and American Time Use Survey** | Activity episodes, work, care, travel, leisure, reported sleep-related time | Harmonization and activity definitions matter; diary sleep time is not automatically physiological sleep. [Time Use Research](https://www.timeuse.org/mtus) |
| **World Bank Living Standards Measurement Study** | Consumption, income, agriculture, household characteristics, shocks | Household totals alone cannot identify each member’s access or priorities. [World Bank](https://www.worldbank.org/en/programs/lsms) |
| **D-PLACE; Kirby et al., 2016** | Cultural, linguistic, geographical, and ecological information for over **1,400 societies** | Use dated, contextual observations; audit sampling and coding biases and non-independence between societies. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) |
| **Physiological and archaeological studies cited above** | Resource requirements, sleep, health, mobility, workload proxies | Stronger for physical conditions than subjective motives; archaeology does not directly reveal utility functions. |

### What remains contested or thinly evidenced

**The exact universal inventory of needs remains unsettled.** SDT offers a well-developed account of three psychological needs, but that does not establish that all human motivation reduces to three variables. [Springer](https://link.springer.com/article/10.1007/s11031-019-09818-1)

**Psychological time constants and tradeoff weights remain the largest calibration gap for this application.** The cited cross-cultural surveys, experiments, and task studies do not jointly identify a universal conversion between calories, autonomy, respect, and purpose. Any such conversion in TCE is a model assumption.

**Historical motivation is particularly underdetermined.** Material remains and comparative ethnography can constrain opportunities, workloads, health, and institutions far better than they constrain an ancient individual’s precise preferences. The appropriate response is explicit uncertainty and sensitivity testing—not a single authoritative set of era-specific personality weights.

**The recommended first implementation is therefore modest:** sound physical accounting; three psychological experience dimensions; durable relationships; contextual respect; a small set of persistent commitments; learned routines; and institutions that change actual access and expectations.

That combination can produce people who prepare, endure, cooperate, resent, learn, care, and pursue ambitions—without pretending that human life is either a rigid pyramid or a set of refillable bars.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927f4-c124-83ea-a279-47d33bb23924)
