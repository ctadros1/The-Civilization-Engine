# Collective action, protest, and riots: a simulation-ready research report

## Core recommendation

**TCE should model grievance, mobilization, violence, and political outcomes as separate processes—not successive levels of an “unrest” meter.** People can be deeply dissatisfied without acting, join demonstrations without supporting violence, continue strikes after street protests disappear, or replace rulers without a mass uprising. Research identifies injustice, collective identity, perceived efficacy, and moral conviction as distinct motivations; organization and resources help translate those motivations into action. [Groningen Research Portal](https://research.rug.nl/en/publications/toward-a-comprehensive-and-potentially-cross-cultural-model-of-wh/)

The appropriate architecture is therefore:

> **Experienced conditions → interpreted grievances → communication and organization → participation choices → interactions and institutional responses → revised beliefs, resources, and loyalties.**

This is a feedback system, not a scripted historical sequence. Below, **empirical estimates**, **published model assumptions**, and **proposed TCE engineering choices** are explicitly distinguished.

---

## 1. Mechanisms: rules a simulation can implement

### 1.1 Hardship becomes political grievance through interpretation and blame

A failed harvest is not automatically a grievance against the ruler. People must interpret their condition as unjust, preventable, or contrary to an obligation. Conversely, people who are materially comfortable may mobilize over discrimination, religion, political exclusion, or solidarity with others. Quantitative syntheses support distinguishing injustice, identity, efficacy, and moral motivation rather than reducing participation to deprivation. [Groningen Research Portal](https://research.rug.nl/en/publications/toward-a-comprehensive-and-potentially-cross-cultural-model-of-wh/)

**TCE rule:** Store grievances by **issue and blamed actor**, not as a single quantity:

* Subsistence: food access, eviction, interrupted relief.
* Extraction: taxes, rents, compulsory labor, military levies.
* Treatment: humiliation, arbitrary punishment, unequal exemptions.
* Collective claims: religious restrictions, exclusion, threatened customs.

Separate current material harm from remembered violations. A tax may be affordable but considered illegitimate; another may be burdensome yet accepted as customary or necessary.

Scott’s research on Southeast Asian peasants emphasizes subsistence guarantees and customary obligations: threatened livelihoods become politically consequential partly through beliefs about what landlords and authorities owe people. This is an influential historical interpretation, not a universal law that peasants always prioritize subsistence over every other value. [JSTOR](https://www.jstor.org/stable/j.ctt1bh4cdk)

**Implementation consequence:** The same crop failure should produce different responses where grain relief works, officials are blamed for hoarding, obligations are negotiated, or migration remains feasible.

### 1.2 Participation requires opportunity and resources, not merely willingness

An aggrieved person may lack time, food reserves, transport, trusted companions, or protection for dependents. Organizations can provide meeting places, information, mutual assistance, and continuity. Resource-mobilization theory explicitly treats these capacities as distinct from dissatisfaction. [DOI](https://doi.org/10.1086%2F226464)

**TCE rule:** Evaluate two questions separately:

1. Does the agent consider participating worthwhile?
2. Can the agent actually participate in this action now?

Participation should consume real time and resources. Household responsibilities can prevent attendance; support from relatives or an association can make attendance feasible.

This creates a useful nonlinearity: worsening conditions can increase motivation while reducing practical capacity. Do not assume that the poorest household is always the most likely to join—or sustain—a protest.

### 1.3 Threshold distributions matter more than average discontent

Granovetter’s threshold model describes people whose willingness to act depends on how many others have acted. Aggregate outcomes depend on the **distribution and ordering of thresholds**, not merely their mean. Similar populations can therefore produce a cascade in one case and near-total inactivity in another. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/226707?utm_source=chatgpt.com)

A mathematical illustration—not an empirical estimate—makes the point. Suppose five people will join when at least **0, 1, 2, 3, and 4 others** have joined. Sequential updating activates everyone. Change the second person’s threshold from 1 to 2, and participation stops after the first person: the chain has a gap.

**TCE rule:** Give agents heterogeneous, action-specific thresholds. Let thresholds depend on perceived danger, identity, commitments, and opportunity costs rather than making them permanent “rebelliousness” traits.

Use both **absolute numbers and proportions**. One committed friend out of one known friend should not automatically provide the same reassurance as twenty committed friends out of twenty.

### 1.4 Networks transmit news and commitment differently

Hearing about an action is not the same as receiving enough reassurance to join it. Complex-contagion models distinguish information that can spread through one contact from costly behavior requiring reinforcement from several contacts. A long-distance connection may carry news efficiently while failing to carry sufficient social reinforcement. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/521848)

**TCE rule:** Maintain at least three informational states:

* The agent has heard about a grievance or proposed action.
* The agent believes particular people support it.
* The agent believes those people will actually participate.

Use existing kinship, workplace, neighborhood, religious, and patronage relationships. Contacts should differ in credibility, relevance, and exposure to the same information.

Crucially, **peer effects are not always positive**. A randomized information experiment around a Hong Kong protest found strategic substitutability: learning that more others would attend could reduce an individual’s own participation. Some agents free-ride when they think sufficient turnout is already assured. [OUP Academic](https://academic.oup.com/qje/article/134/2/1021/5298503)

For TCE, allow competing responses: reassurance, conformity, solidarity, and “they do not need me.”

### 1.5 Organizations and leaders solve specific coordination problems

Leadership is more useful as a collection of functions than as a universal charisma bonus. Research on rural Chinese protest identifies leaders who articulate claims, recruit participants, coordinate across communities, and negotiate. Some emerge from established local figures; others are ordinary petitioners frustrated by unsuccessful appeals. [Cambridge University Press](https://www.cambridge.org/core/journals/china-quarterly/article/abs/protest-leadership-in-rural-china/9B392272F3CA7A3FCE9F3AEE634D4728)

**TCE rule:** Let leadership emerge from demonstrated activity and trusted relationships. Leaders can:

* Propose a shared demand and action time.
* Connect otherwise separate groups.
* Allocate organizational resources.
* Negotiate and report outcomes.

A movement should remain vulnerable to poor coordination, conflicting demands, and unreliable leadership. But removing one leader should not always dissolve it: replacement depends on organizational redundancy.

Organizations should persist between episodes. Their treasuries, reputations, membership ties, and past agreements are part of the explanation for why one disturbance develops into a sustained campaign while another ends quickly.

### 1.6 Triggers combine material shocks with shared visibility

Food-price research supports an association between rising prices and unrest, but not a universal global price threshold. Bellemare’s analysis distinguishes price increases from volatility; Hendrix and Haggard find that political institutions condition the relationship in a study covering 55 Asian and African cities. [DOI](https://doi.org/10.1093%2Fajae%2Faau038)

For TCE, triggers should affect several variables simultaneously:

| Trigger | Material effect | Interpretive or coordinating effect |
| --- | --- | --- |
| Staple-price increase | Changes household purchasing power | Makes distribution, hoarding, or relief failures salient |
| Tax collection | Removes money, goods, or labor | Reveals exemptions, arbitrary treatment, or broken agreements |
| Conscription | Removes household labor; creates anticipated loss | Concentrates affected households around a visible demand |
| Wage arrears | Exhausts reserves and disrupts provisioning | Creates a shared claim against an identifiable employer |
| Violent incident | Injury, death, detention, or fear | Produces competing accounts of injustice and responsibility |

These are **proposed causal pathways**, not measured universal coefficients.

For food shocks, calculate **local staple affordability** and whether a household is a net buyer or seller. For taxation, calculate effective burdens after exemptions, customary rights, and services. For conscription, calculate household consequences and perceived fairness. None has a defensible cross-historical “rebellion begins above X percent” threshold.

### 1.7 Repression has opposing channels

Repression can deter participation, remove participants, damage organizations, or increase outrage. Its effects depend on interpretation and context.

A field experiment with **671 opposition supporters in Zimbabwe** found that induced fear reduced dissent and increased pessimism and risk aversion. A Ugandan survey experiment found greater stated willingness to protest after scenarios involving excessive police force. These are evidence for different mechanisms, not contradictory universal laws. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/psychology-of-state-repression-fear-and-dissentdecisions-in-zimbabwe/3F1E1A2B675D4137C7CA8B4162FDE544)

**TCE rule:** Resolve a coercive incident through four separate updates:

| Channel | Immediate simulation effect |
| --- | --- |
| Deterrence | Revised perceived probability and cost of punishment |
| Incapacitation | Actual injury, detention, displacement, or lost resources |
| Outrage and legitimacy | Changed grievance and judgment of the responsible institution |
| Information and organization | Changed communication, leadership availability, and beliefs about allies |

Do not make “backfire” an automatic multiplier. Distinguish direct victims, witnesses, trusted recipients of reports, and people who accept the authorities’ account.

Also distinguish **willingness to protest** from subsequent attendance: the Ugandan experiment measured responses to hypothetical scenarios, not realized street mobilization. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002720939304)

### 1.8 Violence is an interaction process, not a property of the whole crowd

Research on the 2011 Tottenham and Hackney riots emphasizes interactions and changing social identities rather than treating crowds as uniformly irrational or mechanically driven by deprivation. [University of St Andrews Research Portal](https://research-portal.st-andrews.ac.uk/en/publications/on-the-role-of-a-social-identity-analysis-in-articulating-structu/)

**TCE rule:** Represent violence separately for demonstrators, security personnel, countergroups, and opportunistic participants. Joining a demonstration must not silently authorize violent behavior.

Agents may differ over the acceptability of property damage, interpersonal violence, defense of companions, or compliance with dispersal. Those decisions should respond to actual encounters and group norms.

Record separately:

* Nonviolent collective action.
* Property damage.
* Violence against people.
* State violence against nonviolent participants.

This prevents a common modeling error: interpreting a violently suppressed peaceful demonstration as evidence that its participants “turned into rioters.”

### 1.9 Strikes, uprisings, and coups need different organizational machinery

**Strikes:** Model labor withdrawal through the production system. Its effect depends on which tasks stop, available replacements, inventories, bargaining arrangements, and participants’ ability to endure lost earnings. A small group at an important production dependency can matter more than a large street crowd. These are recommended economic mechanisms, not a claim that strike leverage is proportional to population share.

**Uprisings:** Sustained challenges to political authority need continuity, resources, and changes in the willingness or ability of institutions to enforce orders. Research on major resistance campaigns highlights participation, legitimacy, and shifts among security forces and other regime supporters. [MIT Press Direct](https://direct.mit.edu/isec/article/33/1/7/11935/Why-Civil-Resistance-Works-The-Strategic-Logic-of)

**Coups:** Keep a separate process involving insiders and command relationships. Powell and Thyne’s coup dataset concerns attempts by military or other state elites to unseat the executive—not simply large demonstrations. [OUP Academic](https://academic.oup.com/jpr/article-abstract/48/2/249/8365857)

A protest may influence elite calculations, but **a coup must not be the final level of a riot meter**.

---

## 2. Parameters: empirical anchors versus modeling assumptions

### 2.1 Empirical findings suitable for calibration checks

These values constrain plausible behavior. They generally **cannot be inserted directly into an agent utility function**.

| Quantity | Estimate and units | Evidence | Confidence and interpretation |
| --- | --- | --- | --- |
| Injustice–collective-action association | **r = 0.35**, 95% CI **0.30–0.39**; 65 effect sizes | Van Zomeren, Postmes & Spears, 2008 | Strong evidence of a moderate association across the included studies; not a causal probability increment |
| Efficacy–collective-action association | **r = 0.34**, CI **0.29–0.39**; 53 effect sizes | Same meta-analysis | Supports a separate efficacy channel; substantial contextual variation |
| Identity–collective-action association | **r = 0.38**, CI **0.33–0.42**; 64 effect sizes | Same meta-analysis | Supports group identification rather than grievance-only models |
| Effect of an excessive-force scenario on observers’ stated protest willingness | **13.2% → 22.0%**, a calculated **+8.8 percentage points** | Curtice & Behlendorf; Uganda, total experiment **N = 1,920** | High confidence in the reported experimental contrast; low portability to actual turnout or other societies |
| Breadth of the later motivational synthesis | **1,235 effects; 403 samples; 123,707 participants** | Agostini & van Zomeren, 2021 | Strong basis for considering injustice, identity, efficacy, and morality; not proof of culturally invariant coefficients |

The first three estimates come from the original meta-analysis’s Table 1. Its measures include different forms of collective-action outcomes; they should not be treated as three independent causal weights. The later synthesis broadens the evidence substantially but does not supply a universal simulation parameterization. [Vrije Universiteit Amsterdam](https://research.vu.nl/ws/portalfiles/portal/2391519/Van%20Zomeren%20Psychological%20Bulletin%20134%284%29%202008.pdf)

**Practical use:** After generating synthetic survey-like observations from TCE, check that grievance, efficacy, and identification have meaningful but imperfect relationships with participation. Do not force every simulated society to reproduce these exact correlations.

### 2.2 Proposed TCE parameters and sensitivity ranges

The following are **engineering priors and experimental ranges**, not historical measurements. Their purpose is to make the first implementation explicit and testable.

| Parameter | Proposed initial representation or range | Units | Source/status and confidence |  
|---|---|---|---|---|  
| Issue-specific grievance | **0–1**, with separate current harm and remembered violation | Normalized score | TCE design choice; low confidence in any universal scale |  
| Participation threshold | **0–1** for perceived support; also track absolute committed contacts | Local weighted fraction; persons | Granovetter-inspired representation; distribution must be calibrated |  
| Threshold heterogeneity | Compare **Beta(1,3), Beta(2,2), Beta(3,1)** in controlled experiments | Dimensionless distributions | Deliberately contrasting test cases, not population estimates |  
| Explicit recurrent contacts | Start with **8–32** per person, plus group memberships | Directed relationship records/person | Computational choice, not a claim about total human network size |  
| Economic and grievance update | Approximately **1 simulated day**, plus event-triggered updates | Time | Engineering resolution |  
| Active-crowd social decisions | Test **1–10 simulated minutes** | Time | Engineering resolution; separate from movement/collision integration |  
| Acute arousal memory | Sensitivity sweep of **0.5–7 days** half-life | Days | Uncalibrated; do not confuse with durable grievance |  
| Remembered unresolved violation | Sensitivity sweep of **30–365 days**, with renewed reminders | Days | Uncalibrated; institutional and inherited claims need separate persistence |  
| Strike endurance | Derive from reserves, essential spending, remaining income, and support | Days | Accounting-based rather than a universal behavioral constant |  
| Baseline action hazards | Fit separately by action, institution, and observation regime | Transitions/person-day | No justified universal cross-era value |

The memory ranges are especially uncertain. They should be exposed as sensitivity parameters, not buried as “research-backed” defaults.

For strike endurance, an initial accounting approximation is:

\[
D\_i =
\frac{\text{usable household reserves}}
{\text{essential daily costs}
-\text{remaining daily income}
-\text{reliable daily support}}.
\]

If the denominator is zero or negative, immediate resource depletion does not impose a finite deadline. Actual endurance may still be limited by fear, disagreement, obligations, or changing expectations.

### 2.3 Quantities not presently justified as universal constants

Do **not** assign empirical authority to:

* A fixed percentage of dissatisfied people required for protest.
* A universal tax, food-price, or conscription trigger.
* A fixed probability that repression causes backlash.
* A single grievance half-life.
* A universal fraction of demonstrators who become violent.
* An automatic regime-collapse threshold at 3.5% participation.

Chenoweth’s own cautionary discussion explicitly rejects treating the **3.5% observation as a law**. It notes failures above that level, including Bahrain, and that many successful nonviolent campaigns mobilized less. Organization, leadership, and sustained participation cannot be replaced by a single turnout number. [Harvard Kennedy School](https://www.hks.harvard.edu/centers/carr/publications/questions-answers-and-some-cautionary-updates-regarding-35-rule)

---

## 3. Variation across eras and regions

**Use institutional capabilities, not era labels, to unlock behavior.** Wage disputes require wage relationships; compulsory-labor resistance requires compulsory labor; a coup requires an executive and an insider organization capable of attempting replacement.

| Setting | Evidence and characteristic mechanisms | Implication for TCE |
| --- | --- | --- |
| **Foragers and mobile small-scale societies** | Boehm’s comparative argument emphasizes coalitions that restrain would-be dominators through social sanctions. This does not establish that every forager society was uniformly egalitarian. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/204166) | Allow ridicule, withdrawal of cooperation, exclusion, and group fission. Resistance to a person is not automatically rebellion against a state. |
| **Early farming communities and early states** | Evidence for everyday protest frequency is thin. Scott’s interpretation of early states stresses extraction, labor control, and alternatives to incorporation; its broader historical claims remain contested. [Yale University Press](https://yalebooks.yale.edu/book/9780300231687/against-the-grain/) | Make land attachment, stored crops, debt, compulsory service, and exit costs explicit. Do not invent an archaeological “annual revolt rate.” |
| **Preindustrial Europe** | Historical scholarship records conflicts over provisioning, taxation, customs, work, and local rights. Nicolas’s study also emphasizes that popular action could defend established practices rather than seek progressive reform. [Apple](https://books.apple.com/us/book/la-r%C3%A9bellion-fran%C3%A7aise-mouvements-populaires-et-conscience/id882963232) | Generate demands from actual obligations and institutions. A crowd may demand restoration of custom, not abolition of hierarchy. |
| **Peasant Southeast Asia** | Scott’s work on Burma and Vietnam connects subsistence insecurity with changing obligations and extraction under colonial conditions. [JSTOR](https://www.jstor.org/stable/j.ctt1bh4cdk) | Distinguish crop failure from the perceived failure of patrons or authorities to honor protective obligations. |
| **The Andes under colonial rule** | Walker’s account of the 1780 rebellion emphasizes leadership—including Micaela Bastidas—and the changing composition and fragmentation of a broad coalition. [Apple](https://books.apple.com/us/book/the-tupac-amaru-rebellion/id6476234723) | Permit coalitions to unite around a demand but split over identity, aims, or treatment of other groups. Do not assume a movement has one stable preference. |
| **Igbo communities and colonial Nigeria** | Van Allen documents women’s political institutions and collective action through meetings, market relationships, boycotts, and other sanctions. [DeepDyve](https://www.deepdyve.com/lp/taylor-francis/sitting-on-a-man-colonialism-and-the-lost-political-institutions-of-iHh37P7e0D) | Women’s organizations and economic roles must be represented directly. A generic young-male mobilization model misses important institutions. |
| **Industrializing Japan** | Lewis’s study of the 1918 rice disturbances begins with women responding to rising rice prices and traces varied urban and rural developments. [University of California Press](https://www.ucpress.edu/books/rioters-and-citizens/hardcover) | One price shock can generate different local demands, organization, policing encounters, and outcomes—not one synchronized national riot. |
| **Modern rural China** | Protest leadership can emerge through local standing or failed petitioning; leaders perform concrete recruiting, coordinating, and negotiating work. [Cambridge University Press](https://www.cambridge.org/core/journals/china-quarterly/article/abs/protest-leadership-in-rural-china/9B392272F3CA7A3FCE9F3AEE634D4728) | Model petitioning, brokerage, and failed redress as possible precursors. Modern mobilization need not begin with social media. |
| **Modern cities and authoritarian settings** | Hong Kong evidence demonstrates strategic free-riding; Zimbabwe and Uganda studies demonstrate distinct fear and outrage mechanisms. [OUP Academic](https://academic.oup.com/qje/article/134/2/1021/5298503) | Faster information does not eliminate uncertainty, free-riding, household constraints, or conflicting interpretations of repression. |

The cross-regional lesson is not that societies have different inherent “riot propensities.” They differ in obligations, organizations, political opportunities, communication systems, and credible alternatives.

---

## 4. Historical base rates and stylized facts

### 4.1 What the numbers actually measure

There is **no well-supported universal rate of protests or riots per thousand people per year across history**. Available sources count different things: incidents, demonstrations, strikes, campaigns, or attempts to replace rulers.

| Evidence base | Quantitative benchmark | Correct interpretation |
| --- | --- | --- |
| **HiSCoD**, historical conflict | Its June 2023 release contained **over 20,000 events**, approximately **1000–1870**; about **92%** concerned France and England | An uneven historical compilation, not a global census. Its definition includes at least three people from different families and threatened/actual violence or attacks on property. It is not a census of peaceful protest. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/introducing-hiscod-a-new-gateway-for-the-study-of-historical-social-conflict/0A03A5D98B74DD6E9124EA4552F765AC) |
| **U.S. major work stoppages**, BLS | **470 in 1952**, compared with **5 in 2009** | Counts stoppages involving at least **1,000 workers** and lasting at least one shift; includes strikes and lockouts. Small disputes are excluded. [Bureau of Labor Statistics](https://www.bls.gov/news.release/archives/wkstp_02212024.htm) |
| **Major resistance campaigns**, Stephan & Chenoweth | Among **323 campaigns, 1900–2006**, reported success was **53% for nonviolent** and **26% for violent** campaigns | A historical campaign-level association, not a randomized tactic effect or an event-level success probability. [CNCR](https://www.nonviolent-conflict.org/wp-content/uploads/2016/02/Why-Civil-Resistance-Works-Article.pdf) |
| **BLM-associated demonstrations, United States, 2020**, ACLED | Over **7,750 events**, May 26–August 22; **more than 93%** involved nonviolent demonstrators | A particular wave’s event distribution—not a universal riot probability or percentage of peaceful individual participants. [ACLED](https://acleddata.com/press/us-crisis-monitor-releases-full-data-summer-2020) |
| **Coup attempts**, Powell & Thyne’s original series | **457 attempts, 1950–2010**, of which **227 succeeded** | Elite/insider attempts; useful for a separate coup subsystem, not for converting protests into coups. [ResearchGate](https://www.researchgate.net/publication/227574729_Global_Instances_of_Coups_from_1950_to_2010_A_New_Dataset) |

Historical recording introduces additional problems. HiSCoD combines catalogues with differing participation thresholds, and surviving archives are uneven. **No recorded event is not equivalent to no event having occurred.** [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/introducing-hiscod-a-new-gateway-for-the-study-of-historical-social-conflict/0A03A5D98B74DD6E9124EA4552F765AC)

For TCE’s 10,000–50,000 people, national campaign statistics are particularly unsuitable for direct rescaling. A historical country-level percentage does not establish what will happen in a simulated settlement of a few thousand people.

### 4.2 Patterns a credible simulation should reproduce

**Dormant grievance and sudden change.** High discontent should sometimes coexist with inactivity. Small changes in who participates, what is publicly known, or which contacts are credible should sometimes cause large changes in turnout. This is a central implication of heterogeneous threshold models, not evidence that every outbreak follows the same cascade mechanism. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/226707?utm_source=chatgpt.com)

**Reinforcement and free-riding can coexist.** Increasing expected turnout should reassure some agents and discourage others who think participation is already sufficient. A model that allows only positive imitation fails to encompass the Hong Kong experimental finding. [OUP Academic](https://academic.oup.com/qje/article/134/2/1021/5298503)

**Violence should be concentrated, not automatically crowd-wide.** The ACLED benchmark shows that a prominent protest wave can contain many thousands of events while most demonstrators’ events remain nonviolent. Within TCE, crowd size and violent participation must therefore be separate outputs. [ACLED](https://acleddata.com/press/us-crisis-monitor-releases-full-data-summer-2020)

**Economic distress should not mechanically maximize strikes.** The very low BLS major-stoppage count in 2009 is a useful counterexample to a model where economic deterioration always produces more large strikes. The series alone does not establish the causal explanation, but it rules out that simple monotonic rule. [Bureau of Labor Statistics](https://www.bls.gov/news.release/archives/wkstp_02212024.htm)

**Repression should have heterogeneous and delayed effects.** Fear-driven withdrawal and outrage-driven willingness are both empirically supported. A correct model should permit simultaneous demobilization among some people and increased commitment among others. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/psychology-of-state-repression-fear-and-dissentdecisions-in-zimbabwe/3F1E1A2B675D4137C7CA8B4162FDE544)

**Large turnout is neither necessary nor sufficient for political success.** Campaign-level results and the cautions around the 3.5% observation require outcomes to depend on organization and institutions as well as attendance. [MIT Press Direct](https://direct.mit.edu/isec/article/33/1/7/11935/Why-Civil-Resistance-Works-The-Strategic-Logic-of)

For validation, measure **unique participants, participant-days, peak attendance, duration, geographic spread, and outcomes separately**. Ten demonstrations by the same hundred people are not equivalent to mobilizing a thousand distinct people.

---

## 5. Modeling recommendation for TCE

### 5.1 State representation

The following is a proposed compact design.

| Entity | Minimum useful state |
| --- | --- |
| **Person** | Sparse issue grievances; blamed actors; group identification; risk sensitivity; action-specific moral constraints; beliefs about support and danger; current commitments |
| **Household** | Food, money, essential costs, dependents, labor availability, support obligations |
| **Organization or movement** | Membership, demands, leadership roles, treasury, meeting arrangements, communication links, internal disagreement |
| **Authority or employer** | Resources, obligations, credibility, willingness to negotiate, actual enforcement capacity |
| **Security unit** | Members, commander, location, orders, cohesion, loyalties, fatigue and provisioning |
| **Episode** | Issue, participating organizations, locations, tactics, attendance history, concessions, injuries, arrests, and resolution |

Keep **belief, membership, and current activity** independent. An imprisoned supporter has not necessarily stopped believing in the movement. A sympathizer staying home is not necessarily an opponent. A person can remain a union member after a strike ends.

### 5.2 Local beliefs rather than omniscient turnout

For person \(i\) and movement \(m\), define:

\[
S\_{im}
=
\frac{\sum\_{j\in N\_i}w\_{ij}\,p\_{ijm}}
{\sum\_{j\in N\_i}w\_{ij}},
\]

where \(p\_{ijm}\) is person \(i\)’s belief that contact \(j\) will participate, and \(w\_{ij}\) measures that contact’s relevance.

Track uncertainty separately. An unknown commitment should remain uncertain rather than automatically becoming zero support. Distinguish promises, observed attendance, and rumors. Also track an absolute committed-contact count and perceived crowd conditions.

This representation is inspired by threshold and network research, but its exact weighting scheme is a TCE design choice. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/226707)

### 5.3 A bounded decision rule

One possible action score is:

\[
V\_{ia}
=
\beta\_g g\_{ik}
+\beta\_I I\_{im}
+\beta\_E E\_{ima}
+\beta\_M M\_{ia}
+f\_a(S\_{im})
+O\_{ima}
-C\_{ia}
-\widehat p\_{ia}D\_{ia}.
\]

Here:

* \(g\): relevant grievance.
* \(I\): identification with the movement.
* \(E\): perceived efficacy of the action.
* \(M\): moral attraction or aversion to that action.
* \(f(S)\): social influence, potentially including free-riding.
* \(O\): organizational assistance.
* \(C\): opportunity cost.
* \(\widehat pD\): perceived expected sanction cost.

All terms must be mapped onto a common utility scale. **The meta-analytic correlations in Section 2 are not these coefficients.**

First remove infeasible actions. Then use a threshold or noisy choice rule among the remaining actions. Do not redraw every belief and preference independently each minute: that produces behavioral flickering rather than commitment.

For time-step-independent stochastic transitions, use hazards:

\[
P(\text{transition during }\Delta t)
=
1-\exp(-\lambda\Delta t).
\]

For mutually exclusive actions, use competing hazards rather than independent draws that can make one person undertake several incompatible activities simultaneously. The hazard values remain calibration parameters.

### 5.4 Separate initiation, persistence, escalation, and exit

A single activation test is insufficient. Give each process its own conditions:

| Process | Important inputs |
| --- | --- |
| Initiating an action | Motivation, awareness, expected support, feasibility |
| Continuing participation | Remaining resources, commitments, fatigue, perceived progress |
| Changing tactic | Encounters, norms, organizational decisions, perceived efficacy |
| Leaving | Concessions, fear, depleted resources, disagreement, competing obligations |
| Re-entering later | Persistent membership, unresolved claims, renewed invitations, changed conditions |

Promises should affect expectations; delivered concessions should affect actual conditions and credibility. A movement may divide over whether a settlement is acceptable.

### 5.5 Existing models: what to borrow and what not to borrow

**Epstein’s civil-violence model** is an excellent minimal benchmark. Its core rules are:

\[
G=H(1-L),\qquad
P=1-e^{-k(C/A)},\qquad
N=RP,
\]

with activation when:

\[
G-N>T.
\]

\(A\) includes the prospective participant. Published settings include **\(k=2.3\)**, **\(T=0.1\)**, and uniformly distributed hardship and risk aversion. One run uses a **40×40** lattice, legitimacy **0.82**, cop density **0.04**, vision **7 lattice units**, and maximum imprisonment **30 ticks**. These are illustrative assumptions, not measured human constants. The model distinguishes private grievance from public action, but has no actual political-order replacement, no security-force defection, and no empirically measured hardship. In the presented version, longer jail terms incapacitate without adding sentence-length deterrence. Borrow the local-information mechanism, not the parameter values or political completeness. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC128592/)

**NetLogo Rebellion** and **Mesa’s Epstein Civil Violence example** provide inspectable implementations useful for testing qualitative dynamics. They are reference models, not validated historical predictors; inspect implementation licenses before commercial code reuse. [CCL](https://ccl.northwestern.edu/netlogo/models/Rebellion)

**Kim and Hanneman’s “A Computational Model of Worker Protest”** adds wage comparisons and group-dependent interpretations of support and risk. It is useful when extending beyond a generic hardship variable toward workplaces and social identity. [JASSS](https://www.jasss.org/14/3/1.html)

### 5.6 Computational simplifications

For the stated population, I recommend:

**Daily economic and organizational updates, event-driven political updates.** Recompute grievances when relevant conditions change, rather than evaluating every possible movement for every person every frame.

**Sparse relationships and sparse grievances.** At 50,000 people and 16 directed contacts each, the explicit network has approximately **800,000 contact references**—an arithmetic sizing estimate, not a performance benchmark. Organizations can supply additional shared information without constructing complete member-to-member graphs.

**Detailed interactions only where activity occurs.** Crowd decisions can use local spatial queries; distant agents can receive delayed messages through ordinary relationships and travel. Rendering visibility must not change the underlying political outcome.

**Institutional capacity rather than a universal police percentage.** Track physically available personnel, travel, custody, provisioning, and loyalty. The same nominal force can have different effective capacity under different conditions.

**Stable random streams and update-order tests.** Check that results do not depend excessively on agent array order, synchronized daily updates, or the chosen time step.

---

## 6. Sources, calibration strategy, and remaining uncertainty

### 6.1 Additional datasets worth using

| Dataset | Coverage and useful application | Main caution |
| --- | --- | --- |
| **SCAD: Social Conflict Analysis Database** | **1990–2017**; Africa plus Mexico, Central America, and the Caribbean. Useful for protests, strikes, riots, and other social-conflict episodes. [Strauss Center](https://www.strausscenter.org/ccaps-research-areas/social-conflict/database/) | News-based coverage and episode definitions affect counts and durations |
| **Mass Mobilization Data Project** | **162 countries, 1990–2018**; useful for antigovernment mobilization and state responses. [Binghamton University](https://www.binghamton.edu/political-science/massmobilization.html) | Does not represent all workplace, neighborhood, or customary disputes |
| **NAVCO 2.1** | **384 maximalist campaigns, 1945–2013**, with campaign-year observations. Useful for sustained campaigns and outcomes. [DOI](https://doi.org/10.7910/DVN/MHOXDV) | Selected major campaigns, not routine protest incidence |
| **HiSCoD** | Historical local conflict, with explicit source provenance. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/introducing-hiscod-a-new-gateway-for-the-study-of-historical-social-conflict/0A03A5D98B74DD6E9124EA4552F765AC) | Uneven geography, surviving archives, and heterogeneous inclusion thresholds |
| **BLS work-stoppage series** | Long-run, consistently defined large labor disputes. [Bureau of Labor Statistics](https://www.bls.gov/news.release/archives/wkstp_02212024.htm) | Size threshold excludes most disputes in a small simulated economy |

### 6.2 Calibrate an observation model as well as a behavior model

A useful comparison requires TCE events to be filtered according to the source being evaluated.

For a BLS-like comparison, count only sufficiently large and long work stoppages. For a campaign dataset, combine related actions into sustained political campaigns. For historical archives, model incomplete recording rather than treating every unrecorded settlement-year as peaceful.

Fit several outputs jointly: participation, duration, recurrence, resources, violence, concessions, and elite responses. Matching only the total number of outbreaks can conceal an incorrect mechanism.

Use held-out settings and controlled interventions. For example, keep average grievance fixed while changing network structure; keep a food-price shock fixed while varying household exposure and relief institutions; keep coercive harm fixed while varying who observes it and how credible accounts spread. These are proposed simulation experiments, not estimates of real-world treatment effects.

### 6.3 Where the evidence is strongest—and weakest

The strongest basis for implementation is that **motivation is multidimensional, participation is socially interdependent, organization matters, and repression operates through competing mechanisms**. The weakest basis is for universal ancient base rates, fixed memory durations, exact threshold distributions, and portable coefficients linking tax burdens or food prices directly to rebellion. The cited research supports mechanisms and conditional comparisons much better than timeless numerical constants. [Groningen Research Portal](https://research.rug.nl/en/publications/toward-a-comprehensive-and-potentially-cross-cultural-model-of-wh/)

**Recommended first implementation:** connect household burdens to issue-specific grievances; mobilize through actual relationships and organizations; keep beliefs local and uncertain; make participation consume resources; separate violence from attendance; and let authorities, employers, and security personnel respond as agents with their own obligations and loyalties.

That architecture gives TCE multiple routes to collective action—and equally important, multiple reasons why collective action does not occur—without scripting a historical sequence or manufacturing a universal rebellion rate.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556-tce-research/c/6ab92812-094c-83ea-9e30-b8e66c05ce0a)
