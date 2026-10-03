# Modeling battles and casualties for The Civilization Engine

## Executive recommendation

**Use a spatially constrained, stochastic casualty model coupled to a separate model of cohesion, withdrawal, and capture.** Let victory emerge from those processes and the participants’ objectives—not from a preliminary “winner” roll followed by predetermined losses.

Three findings are especially important. Constant-coefficient Lanchester equations fit some major historical battle datasets poorly. Casualty-only rules for deciding when armies break have also failed empirical tests. And catastrophic operational losses can consist predominantly of prisoners rather than deaths. These findings support separating **physical attrition, organizational defeat, and the consequences of defeat**. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/nav.10101)

For TCE, the appropriate abstraction is therefore:

**Contact and exposure → injuries and disruption → decisions and cohesion changes → withdrawal or collapse → pursuit, capture, and medical outcomes.**

The historical numbers below are calibration targets, not universal constants. Proposed implementation parameters are explicitly distinguished from measured or reconstructed values.

---

## 1. Mechanisms and mathematical models

### 1.1 Define what a “casualty” means before fitting anything

TCE should distinguish at least three kinds of outcome:

| Dimension | Recommended states | Why it must remain separate |
| --- | --- | --- |
| **Health** | Uninjured, wounded by severity, recovering, permanently impaired, dead | An injured person can survive, recover, or die later. |
| **Availability and custody** | Fighting, withdrawing, separated, deserted, captured, returned | A person can leave the fighting without being injured. A wounded person can also be captured. |
| **Organizational outcome** | Objective achieved, attack repulsed, orderly withdrawal, formation broken, force dispersed or surrendered | Defeat need not mean destruction, and remaining alive need not mean remaining an effective army. |

Historical administrative categories do not map perfectly onto biological states. For example, the modern US study discussed below distinguishes killed in action from died of wounds partly by whether the casualty reached a treatment facility. The Dupuy capture study also documents complications when wounded personnel become prisoners or are reported as missing. **Preserve actual agent states internally; translate them into source-specific reporting categories only for comparison.** [PubMed](https://pubmed.ncbi.nlm.nih.gov/30916730/)

### 1.2 Lanchester laws: useful limiting cases, not a complete battle engine

Let \(A\) and \(B\) be effective combatant numbers, and let \(\alpha\) and \(\beta\) describe their respective effectiveness.

**Aimed-fire, square-law model**

\[
\frac{dA}{dt}=-\beta B,\qquad
\frac{dB}{dt}=-\alpha A
\]

This gives the invariant:

\[
\alpha A^2-\beta B^2=\text{constant}.
\]

Its defining assumption is that additional combatants can continue contributing effective attacks against the opposing force. Under those assumptions, numbers have a strongly increasing value.

**Frontage-limited, paired-combat model**

A simple alternative is:

\[
\frac{dA}{dt}=-\beta E,\qquad
\frac{dB}{dt}=-\alpha E,
\qquad E=\min(A,B,K),
\]

where \(K\) is the number of simultaneous fighting positions. This produces a linear invariant:

\[
\alpha A-\beta B=\text{constant}.
\]

Additional personnel behind the fighting line initially provide reserves rather than additional attacks. Another formulation conventionally associated with the linear law uses area-fire terms proportional to \(AB\); it has the same linear invariant but different timing and coefficient units. Thus, the name “linear law” does not uniquely specify the underlying contact process. [Praxis](https://lru.praxis.dk/Lru/microsites/hvadermatematik/hem3download/kap6_projekt_6_2_ekstra_Turkes_rapport_Monterey_2000.pdf)

A mathematical implication worth preserving: **doubling numbers in the square-law model doubles initial attack output, not quadruples it**. The squared advantage concerns the model’s cumulative contest, not instantaneous damage.

Do not assign “linear” to ancient societies and “square” to modern ones automatically. Select contact rules from frontage, visibility, targeting, range, and weapon employment.

**Empirical assessment.** Lucas and Turkes’s *Fitting Lanchester Equations to the Battles of Kursk and Ardennes* found that several formulations fitted similarly, none of the basic square, linear, or logarithmic laws consistently performed well, and no constant-coefficient version fitted particularly well. Different fitting procedures could produce substantially different apparent best models. This argues against treating an estimated exponent or effectiveness coefficient as a universal historical law. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/nav.10101)

**TCE use:** retain Lanchester models as analytical benchmarks and automated tests. They are useful for checking that unrestricted concentration and constrained frontage behave differently. They should not determine battle termination.

### 1.3 Dupuy’s QJM and TNDM: borrow the factor structure cautiously

Dupuy’s **Quantified Judgement Model**, and its successor the **Tactical Numerical Deterministic Model**, aggregate weapons and personnel, then apply adjustments for posture, opposition, terrain, weather, surprise, and other conditions. Their casualty calculations are empirical, multiplicative models rather than direct simulations of each encounter. [Dupuy Institute](https://dupuyinstitute.org/2019/12/26/how-attrition-is-calculated-in-the-qjm-vs-the-tndm/)

Their strongest lesson for TCE is methodological: equipment counts alone are insufficient. However, distinguish documented model coefficients from measured causal effects:

| Documented legacy coefficient | Value | Meaning—not an observed universal rate |
| --- | --- | --- |
| QJM post-1900 attacker baseline | **2.8% per day** | Starting percentage before other factors |
| QJM post-1900 defender baseline | **1.5% per day** | Starting percentage before other factors |
| TNDM personnel-loss factors | **0.04 attacker; 0.06 defender** | Inputs multiplied by personnel and further adjustments |
| TNDM complete-surprise factor | **2.5×** | Model adjustment, not proof that surprise universally multiplies casualties by 2.5 |

These are documented in Lawrence’s account of the models; the TNDM version described there was not intended for pre-WWII applications. Confidence is high that these were model coefficients, but low that they transfer directly to TCE’s early agrarian battles. [Dupuy Institute](https://dupuyinstitute.org/2019/12/26/how-attrition-is-calculated-in-the-qjm-vs-the-tndm/)

A particular danger is the **Combat Effectiveness Value**, or CEV. Some TNDM procedures infer it from mission accomplishment, advance, and casualty performance. Using that battle-derived value to “predict” the same battle introduces outcome information into the input. My recommendation is to estimate persistent organizational qualities from prior behavior, training, command practices, and supply—not from the outcome being predicted, and never from an immutable national or ethnic bonus. [Dupuy Institute](https://dupuyinstitute.org/2019/09/18/validating-a-combat-model-part-iii/)

### 1.4 Other quantified approaches

For TCE, three alternatives are useful:

| Approach | What it adds | Appropriate role |
| --- | --- | --- |
| **Stochastic attrition processes** | Integer losses and substantial variation in small engagements | Core casualty sampling |
| **Networked attrition models** | Forces can affect only enemies connected through feasible engagement relationships | Basis for sectors, range, visibility, and flanks |
| **Statistical outcome models** | Estimate win probabilities from pre-battle characteristics | External calibration benchmark, not a substitute for persistent battle consequences |

Networked Lanchester work explicitly explores how interaction structure changes combat dynamics, but theoretical results from such models are not themselves validation against historical battles. [arXiv](https://arxiv.org/abs/2105.06104)

### 1.5 Recommended casualty-generation process

The following is a **proposed TCE formulation**, not a published historical law.

Partition the battlefield into engagement sectors, with persistent tactical groups assigned to them. For group \(j\), calculate the number actually able and willing to participate:

\[
E\_j=\min(N\_{\text{available},j},K\_{\text{usable},j}).
\]

Here \(K\) is weapon- and situation-dependent: melee frontage, firing positions, visible targets, or another relevant capacity.

For attacks from group \(j\) against group \(i\), define a casualty-hazard budget:

\[
\Lambda\_{j\rightarrow i}
=
\kappa\_{ji} E\_j w\_{ji} v\_{ji} x\_i s\_{ji}.
\]

The terms are:

* \(\kappa\_{ji}\): baseline casualty-producing intensity per active attacker-hour for this weapon–target interaction.
* \(w\_{ji}\): allocation of the attacking group’s effort; allocations must not exceed its available effort.
* \(v\_{ji}\): acquisition and visibility.
* \(x\_i\): target exposure.
* \(s\_{ji}\): protection and weapon–target compatibility.

For \(N\_i\) eligible people with approximately equal exposure:

\[
h\_i=\frac{\sum\_j\Lambda\_{j\rightarrow i}}{N\_i},
\qquad
C\_i\sim\operatorname{Binomial}
\left(N\_i,1-e^{-h\_i\Delta t}\right).
\]

When exposure differs substantially, sample by exposure class or person rather than spreading casualties uniformly across the group. No eligible targets means no attack casualties.

This formulation provides bounded probabilities and integer outcomes. It also makes time units explicit. Ammunition, weapon availability, and feasible contact constrain the hazard before sampling.

**Compute both sides’ attacks from the same pre-step state, then apply results together.** Otherwise, array iteration order can accidentally give one side an unearned first-strike advantage.

For volleys, explosions, or simultaneous formation shocks, use correlated events or a shared intensity disturbance. Independent, identical person-level rolls are a useful baseline, but should not be the only source of variation.

### 1.6 Model morale, cohesion, suppression, and fatigue separately

A useful distinction is:

**Morale:** willingness to continue.  
**Cohesion:** ability to act together.  
**Suppression:** immediate inhibition under threat.  
**Fatigue:** declining physical and cognitive capacity.

RAND’s *Will to Fight* research treats combat persistence as a combination of individual, group, organizational, and contextual influences, and emphasizes the limited empirical validation of many existing simulation treatments. [RAND Corporation](https://www.rand.org/content/dam/rand/pubs/research_reports/RR2300/RR2341/RAND_RR2341.pdf)

Avoid “the unit always breaks at 30% casualties.” Helmbold’s empirical examination found important contradictions even for a broader class of models in which opposing forces independently draw casualty breakpoints from generally applicable distributions. Randomizing a universal casualty threshold does not resolve the underlying problem. [RAND Corporation](https://www.rand.org/content/dam/rand/pubs/reports/2007/R772.pdf)

A proposed TCE alternative is a threat-dependent transition hazard:

\[
q\_i=\frac{\operatorname{softplus}(z\_i)}{\tau},
\qquad
P(\text{break during }\Delta t)=1-e^{-q\_i\Delta t}.
\]

Let \(z\_i\) depend on recent local losses, neighboring flight, flank isolation, command disruption, fatigue, cohesion, and commitment. Evaluate it under credible threat, not continuously in peaceful conditions.

Use both personal and shared information. A person need not know the army-wide casualty percentage to notice their immediate group disintegrating.

The coefficients remain calibration parameters. Existing evidence does not justify a universal numerical weight for “leader killed” or “friend fled.”

### 1.7 Withdrawal, pursuit, and capture are separate processes

Implement at least three exit modes:

**Ordered withdrawal:** coordination and defensive activity remain possible.  
**Rout:** movement becomes less coordinated and formations disperse.  
**Surrender:** organized resistance ceases and custody changes.

For fleeing individuals or groups, model the next event through competing hazards—escape, capture, or further injury:

\[
P(\text{some event})=
1-e^{-(\lambda\_e+\lambda\_c+\lambda\_w)\Delta t}.
\]

Choose the event in proportion to its hazard. An injury then enters the medical model; capture changes custody without erasing health state.

These hazards should depend on actual reachability. Pursuers require a route, time, stamina, and available personnel. Darkness, obstacles, intact rearguards, refuge, and bottlenecks change the result. Pursuit also competes with regrouping, guarding captives, rescuing wounded, and taking property.

The historical capture evidence below supports treating collapse and encirclement as qualitatively different from ordinary exchanges. It does **not** support a universal “losers suffer another 20%” rule. [Dupuy Institute](https://www.dupuyinstitute.org/pdf/e-4epw1and2final.pdf)

### 1.8 Translate “force multipliers” into mechanisms

For TCE, implement the following channels rather than stacking generic bonuses:

| Factor | Recommended causal representation |
| --- | --- |
| **Terrain** | Visibility, traversable paths, movement cost, frontage, available cover, and retreat connectivity |
| **Fortification** | Protected firing positions, obstacles, restricted access, breach state, and safe internal movement |
| **Technology** | Range, attack opportunities, protection, mobility, reliability, ammunition demand, detection, and communication |
| **Leadership** | Information interpretation, order delay, coordination, reserve commitment, succession, and trust |
| **Surprise** | Delayed detection and response, unfavorable initial posture, temporary loss of usable frontage or cover |
| **Training and cohesion** | Maintaining formation, using equipment, executing withdrawals, rallying, and replacing casualties |
| **Logistics** | Available ammunition, fatigue, hunger, transport, replacement equipment, and treatment capacity |

Avoid counting one effect repeatedly. For example, a fortification’s protection should not reduce exposure and then receive an additional blanket “defense bonus” calibrated from the same protection advantage.

At Ōhaeawai, Māori fortification combined palisades, protected firing arrangements, and internal shelter. That is a more informative model template than simply assigning “fort: ×2 defense.” [NZ History](https://nzhistory.govt.nz/media/photo/plan-ohaeawai-pa)

---

## 2. Quantitative parameters and calibration targets

### 2.1 Empirical anchors

**Confidence concerns the stated observation within its context—not transferability to every society.** None of these rows establishes a global casualty-rate distribution.

| Context and source | Quantitative finding | Denominator and period | Confidence and limitation |
| --- | --- | --- | --- |
| Classical Greek hoplite battles; Krentz, 1985 | Winners: approximately **5% dead on average**, commonly **3–10%** | Reconstructed winning force, per battle | **Low–moderate:** uncertain ancient strengths and reports; selected corpus |
| Same corpus | Losers: approximately **14% dead on average**, commonly **10–20%** | Reconstructed defeated force, per battle | **Low–moderate:** not all premodern warfare; these are deaths, not all injuries |
| Turkana force raids; Mathew and Boyd, 2011 | **1.1% killed per raid; 1.3% when combat occurred** | Participating warriors; **47 force raids** | **Moderate:** interview-based sample; contemporary pastoralists using firearms |
| Kursk division-level engagements; Lawrence’s published compilation | German mean losses: **0.99%/day attacking**, **0.68% defending**; Soviet: **3.25% attacking**, **4.31% defending** | Reported personnel losses relative to force strength; **192 engagements** | **Moderate:** campaign-specific group means, not individual exposure hazards |
| Ōhaeawai assault, 1845 | **40 killed and 70 wounded among 250 attackers**—**44% casualties** | Assault party, in minutes | **Moderate:** one exceptional local assault, not the entire army |
| US combat casualty care, Afghanistan and Iraq, 2001–2017; Howard et al., 2019 | Case fatality fell **20.0% → 8.6%** in Afghanistan; **20.4% → 10.1%** in Iraq | Deaths among battle-injury casualties; **56,763 casualties** in the study | **Moderate–high descriptive confidence:** specialized medical system; observational comparison |

Sources: Krentz’s reconstructed table and discussion; Mathew and Boyd’s raid study; Lawrence’s division-level summaries; New Zealand’s official historical account; Howard and colleagues’ trauma-system analysis. [GRBS](https://grbs.library.duke.edu/article/download/5321/5325)

The central warning is the denominator. **“One percent per division-day,” “one percent per raid,” and “one percent of exposed people per hour” are different quantities.** An army-wide average can coexist with devastating losses in one assault group.

### 2.2 Outcomes, captures, and winner–loser asymmetry

The following are selected outcome categories from the Dupuy Institute’s compilation of **195 division-level engagements**. Percentages are mean reported daily rates relative to each side’s own strength. Captures are included within the operational casualty accounting, not additional deaths.

| Outcome category | Engagements | Attacker casualties/day | Defender casualties/day | Defender captured/day |
| --- | --- | --- | --- | --- |
| Failed attack | 54 | **2.98%** | **2.62%** | **0.34%** |
| Successful attack | 71 | **1.20%** | **2.96%** | **0.92%** |
| Defender penetrated | 33 | **0.83%** | **6.40%** | **2.98%** |
| Defender enveloped | 8 | **1.20%** | **36.00%** | **30.43%** |

Source: Lawrence and Anderson, *Enemy Prisoner of War/Civilian Internee Capture Rate Study*, Table 7. Confidence is **moderate for descriptive patterns, lower for precise transferable rates**, particularly the eight envelopment cases. [Dupuy Institute](https://www.dupuyinstitute.org/pdf/e-4epw1and2final.pdf)

Use these as **conditional output checks**, not input multipliers. The model should produce increased capture when escape routes and organized resistance fail; it must not consult the future outcome label to assign casualties.

### 2.3 Numerical starting settings for TCE

The following are **engineering proposals**, not historical measurements.

| Parameter | Initial value or sweep | Units | Basis and confidence |
| --- | --- | --- | --- |
| Persistent tactical/social group | **10–40** | People | Proposed aggregation scale; not a claim about universal historical unit size |
| Ordinary combat integration step | **10–60** | Seconds | Starting performance range; shorten during rapid transitions |
| Maximum ordinary continuous-event probability per step | **0.02–0.05** | Probability | Numerical accuracy target; verify by convergence tests |
| Recent-loss memory | **2–10** | Minutes | Proposed morale sensitivity sweep; low empirical confidence |
| Shared effectiveness uncertainty | **0.2–0.5** | Log-scale standard deviation | Proposed uncertainty sweep, not an observed historical distribution |
| Eligible exposure | **0–1**, derived from situation | Fraction of roster | Logical bounds; do not impose a universal exposed fraction |

For mean-one multiplicative uncertainty, use:

\[
M=\exp(\sigma Z-\sigma^2/2),\qquad Z\sim N(0,1).
\]

Draw persistent group- or engagement-level uncertainty rather than re-rolling all underlying competence every tick.

**Calibrating an exposure hazard: an illustrative calculation.** Suppose a reference engagement produces 2% casualties across the roster over two hours, while 40% of the roster is exposed. Under a simple constant-hazard assumption:

\[
0.02=0.4(1-e^{-2h})
\quad\Rightarrow\quad
h\approx0.0256\ \text{per exposed person-hour}.
\]

This is an algebraic example, not an estimated historical rate. Assuming everyone was exposed would produce a different estimate. Consequently, casualty rates cannot identify weapon lethality without assumptions about exposure and duration.

For terrain, leadership, surprise, and rout lethality, the evidence reviewed does **not** establish defensible universal numerical coefficients. Preserve scenario-specific parameters and sensitivity ranges rather than manufacturing precision.

---

## 3. Variation across eras and world regions

### Foragers: do not infer battle probabilities from skeletal trauma

Reanalysis of **61 individuals** at Jebel Sahaba in the Nile Valley found evidence consistent with repeated episodes of violence rather than a single catastrophic battle. Healed and unhealed injuries help establish repeated exposure, but they do not provide the sizes of opposing forces, the number of encounters, or a casualty probability per encounter. [PubMed](https://pubmed.ncbi.nlm.nih.gov/34045477/)

**TCE implication:** allow ambushes, raids, pursuit, revenge attacks, and encounters that end without sustained fighting. Do not require two formally organized armies. Archaeological trauma should inform cumulative violence checks, not directly set a battle-damage coefficient.

### Early farming: distinguish community attacks from pitched battles

The Early Neolithic Schöneck-Kilianstädten mass grave contains at least **26 people killed**, with evidence of severe violence. Such evidence demonstrates the possibility of catastrophic community-level attacks, but cannot establish an ordinary casualty rate for opposing field forces. [DOI](https://doi.org/10.1073/pnas.1504365112)

**TCE implication:** attacks on settlements need explicit civilian presence, refuge, evacuation, and captive-taking processes. A massacre should not be represented merely as an exceptionally high army attrition roll.

### Preindustrial armies: frontage, reserves, cohesion, and escape conditions

Krentz’s Greek evidence provides one useful calibration family, not a universal ancient template. Its substantial winner fatalities also caution against assuming that almost everyone killed in every premodern battle died only after fleeing. [GRBS](https://grbs.library.duke.edu/article/download/5321/5325)

**TCE implication:** distinguish sustained fighting losses from pursuit losses. Mounted pursuit, intact retreat routes, formation depth, protected positions, and the ability to rotate or reinforce fighting groups should change outcomes through the shared mechanism set.

For the Americas, Hassig’s analysis of Aztec warfare emphasizes logistical constraints, terrain, agricultural manpower demands, and political control. His treatment of the Spanish conquest also places indigenous alliances centrally rather than explaining outcomes through European weapons alone. **Model coalition reliability, access, supply, and political objectives alongside equipment.** [University of Oklahoma Press](https://www.oupress.com/9780806127736/aztec-warfare/)

### Nonstate organization is not synonymous with weak coordination

The Turkana study documents large cooperative raids and sanctions against failures to participate effectively without requiring a centralized state command structure. But the observed forces were contemporary pastoralists with firearms—not a direct proxy for prehistoric foragers. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3136302/)

**TCE implication:** age associations, reputation, kinship, councils, customary sanctions, and voluntary leadership can generate combat cooperation. A “state present” Boolean should not determine cohesion.

### Industrial warfare: distinguish administrative strength from local fighting strength

The Kursk data show considerable variation between campaign participants and postures. Treating all personnel in a division as equally exposed would erase the distinction between local fighting and the wider organization. [Dupuy Institute](https://dupuyinstitute.org/2023/04/25/average-losses-per-day-in-division-level-engagements-on-the-eastern-front-in-1943/)

Pacific experience also contradicts a simple technology ladder. At Ōhaeawai, a locally adapted defensive system imposed severe losses on an assaulting colonial force. The relevant interaction was between fortification, exposure, weapons, and the assault decision—not a universal superiority attached to the attacker’s society. [NZ History](https://nzhistory.govt.nz/war/northern-war/ohaeawai)

### Modern warfare: injury incidence and survival can move separately

Howard and colleagues associate improved survival with changes in combat casualty care, including transport and treatment. This supports giving medical institutions their own effects rather than changing only a weapon’s “kill percentage.” The estimates should not be generalized to every modern military or civilian population. [PubMed](https://pubmed.ncbi.nlm.nih.gov/30916730/)

For TCE’s unscripted development, use **capability bundles**, not era switches: concentrated or dispersed formations, reconnaissance, communications, protected mobility, projectile performance, supply, and medical care. Societies can develop these unevenly.

**Coverage limitation:** the accessible numerical evidence remains disproportionately European and US military. The African and Pacific cases broaden mechanism testing, but do not supply representative preindustrial casualty distributions for Africa, Asia, or the Americas.

---

## 4. Stylized facts a correct simulation should reproduce

### Defeat and destruction must not be interchangeable

The historical break-point tests caution against reducing battle termination to a generally applicable casualty threshold. TCE should permit withdrawal, failed objectives, and loss of cohesion at widely varying loss fractions—not merely at different draws around one standard threshold. [RAND Corporation](https://www.rand.org/content/dam/rand/pubs/reports/2007/R772.pdf)

### Winner–loser asymmetry should vary with the type of defeat

The Greek reconstruction suggests appreciable losses on both sides, while the WWII capture compilation shows much sharper operational asymmetry when defenders are enveloped. A plausible simulator should produce both costly victories and victories dominated by enemy surrender or dispersal. [GRBS](https://grbs.library.duke.edu/article/download/5321/5325)

### Local catastrophes should coexist with modest whole-force averages

Ōhaeawai’s 44% assault-party casualties must be reproducible without requiring 44% losses across every supporting or uncommitted person. Validate the distribution of losses across groups, not only the battle total. [NZ History](https://nzhistory.govt.nz/war/northern-war/ohaeawai)

### Small engagements should produce lumpy outcomes

As a mathematical example, 50 exposed people with an independent 2% casualty probability produce one expected casualty, but about a **36% probability of no casualties**:

\[
P(C=0)=0.98^{50}\approx0.364.
\]

A deterministic “one casualty every time” rule destroys important variation at TCE’s village-war scale. Correlated events can widen that variation further.

### Medical development should change deaths without necessarily changing injuries

A simulator should be able to match changes in case fatality while holding injury incidence approximately constant. The US trauma-system results provide a useful context-specific test of this separation. [PubMed](https://pubmed.ncbi.nlm.nih.gov/30916730/)

### Numerical advantage should depend on usable contact

As a model-consistency test, adding reserves behind an unchanged bottleneck should not immediately multiply attacks. The extra force should matter through rotation, endurance, maneuver, and replacement. Conversely, where more personnel genuinely gain usable engagement opportunities, concentration should increase immediate output.

### Casualties and duration must be fitted jointly

This is an identification requirement: an army’s willingness and ability to remain engaged affects how long it is exposed. A casualty model fitted independently of stopping behavior can compensate for premature retreats with inflated lethality—or for overly persistent fighting with implausibly weak weapons.

---

## 5. Recommended TCE implementation

### 5.1 Keep individuals authoritative; aggregate interactions

Use three layers:

| Layer | Persistent information | Recommended resolution |
| --- | --- | --- |
| **Person** | Health, equipment, skill, experience, relationships, location, custody, service obligations | Individual identity and consequential events |
| **Tactical group** | Membership, leader, cohesion, fatigue, current order, frontage, perceived threats | Statistical combat and behavioral transitions |
| **Army or coalition** | Command relationships, objectives, reserves, supply, retreat plan, political constraints | Organizational decisions and battle termination |

Groups should be formed from actual institutions and social relationships, not regenerated as interchangeable numerical packets for every battle. Their computational size can be bounded without claiming that every society historically used the same formation.

Calculate attack events on a sparse contact graph. Resolve sampled casualties onto eligible agent IDs. That avoids testing every person against every enemy every tick while preserving real demographic consequences.

### 5.2 A minimal update sequence

```
1. Snapshot current positions, orders, readiness, health, and custody.
2. Update detection, communication, and feasible engagement contacts.
3. Allocate frontage, targets, ammunition, and active participants.
4. Generate attack and disruption events for both sides.
5. Apply injuries, equipment effects, and immediate losses simultaneously.
6. Update perceived local conditions, fatigue, cohesion, and command state.
7. Resolve continued fighting, ordered withdrawal, rout, rally, or surrender.
8. Move groups and resolve reachable pursuit and capture.
9. Update rescue, treatment, separation, and regrouping.
10. Evaluate objectives and whether organized battle continues.
```

Do not require every engagement to pass through every phase. A surprise attack may end before a stable front forms; a failed probe may disengage before melee.

### 5.3 Preserve the aftermath

A sampled casualty must affect an actual person. Carry injuries into recovery, impairment, and delayed mortality. Carry capture into custody, release, escape, or other institutionally determined outcomes.

Field control should affect access to wounded people and abandoned equipment. Army dissolution should create returning survivors, separated groups, prisoners, and deserters—not simply delete the losing unit.

Connect these events to households, labor availability, dependants, leadership succession, military experience, public support, and compensation obligations. Keep battle injury separate from campaign disease, starvation, and deliberate violence against civilians or captives.

In Unreal, visualize authoritative events from the Rust kernel. A rendered death must not become an additional demographic death, and camera distance must not change the simulation’s random decisions.

### 5.4 What to simplify

For a first version, abstract individual projectile trajectories, detailed formations below the chosen tactical scale, and moment-to-moment targeting decisions. Preserve the elements most likely to change the outcome: contact, exposure, obstacles, suppression, cohesion, command, reserves, escape routes, and supply.

For fast-forward, integrate accumulated exposure and event hazards, but subdivide whenever contact, orders, or morale state changes materially. Do not extrapolate an unchanged casualty rate through an entire battle after one side begins fleeing.

These are architectural recommendations, not measured performance claims. Benchmark representative battles before choosing final group sizes and update intervals.

### 5.5 Existing games and research simulations

| Implementation | Relevant feature | Lesson for TCE |
| --- | --- | --- |
| **Close Combat**, particularly later titles examined by RAND | Individual and group behavior involving morale, cohesion, fatigue, leadership, and experience | Visible soldiers should be able to cease effective participation without first exhausting health |
| **ISAAC/EINStein research simulations** | Agent interactions with behavioral states and local relationships | Useful precedent for emergent local dynamics; not sufficient historical validation |
| **RAND experiments using IWARS and NetLogo** | Explored how human-factor assumptions change outcomes | Test sensitivity to behavioral assumptions, rather than hiding them |
| **Field of Glory II** | Formation-scale tactical units; generals affect combat and morale | Demonstrates a legible intermediate scale between individual duels and whole-army resolution |

RAND explicitly treats its human-factor simulation experiments as exploratory rather than a completed, empirically validated model. The game examples likewise demonstrate implementation choices, not proof of historical accuracy. [RAND Corporation](https://www.rand.org/content/dam/rand/pubs/research_reports/RR2300/RR2341/RAND_RR2341.pdf)

### 5.6 Validation plan

Fit and test the simulator against **distributions**, not just average casualties or the winner of famous battles.

Use pre-battle information to predict outcomes. Hold out entire campaigns or wars, so that days from the same engagement do not appear in both training and testing. Fit casualty generation and termination together, with uncertain strengths and reporting conventions represented explicitly.

Report win-probability calibration, duration distributions, fatalities and injuries separately, captures, losses by exposed subgroup, zero-casualty encounters, and the upper tail of catastrophic outcomes. Check conditional results for orderly withdrawals, routs, and encirclements.

Add engineering invariants: no duplicated people, no negative strengths, bounded probabilities, no attacks through impossible contacts, no extra casualties caused by rendering, and convergence when time steps are reduced. Splitting a computational batch without changing physical organization should not change expected outcomes.

---

## 6. Sources, datasets, and remaining uncertainty

### Priority sources and their proper uses

| Source | Best use | Main limitation |
| --- | --- | --- |
| **Lucas and Turkes, 2004**, *Naval Research Logistics*, “Fitting Lanchester Equations to the Battles of Kursk and Ardennes” | Empirical evaluation of aggregate attrition equations | Two major WWII campaign datasets; not a complete alternative theory |
| **Helmbold, 1971**, RAND R-772, *Decision in Battle* | Testing casualty-breakpoint assumptions | Establishes failures more clearly than a universally validated replacement |
| **Krentz, 1985**, *Greek, Roman, and Byzantine Studies* 26:13–20 | Premodern winner–loser fatality anchors | Reconstructed, selective ancient evidence |
| **Mathew and Boyd, 2011**, *PNAS*, “Punishment Sustains Large-Scale Cooperation in Prestate Warfare” | Raid losses and nonstate cooperation | Contemporary interview sample with firearms |
| **Lawrence and Anderson, 2000**, EPW/CI capture study | Capture, outcome, and aggregation effects | Selected WWII engagements; reporting inconsistencies |
| **Howard et al., 2019**, *JAMA Surgery*, combat casualty-care analysis | Separating injury incidence from survival | Observational evidence from a particular military medical system |

These are the principal empirical studies underlying the report. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/nav.10101)

### Dataset selection

The **Dupuy Institute’s DUWAR databases**, together with the Kursk and Ardennes data discussed in the modeling literature, are the most directly relevant starting points for force strengths, posture, engagement outcomes, and casualties. Confirm access, licensing, coding definitions, and missingness before building a calibration pipeline. This report uses published analyses and tables; it does not claim an independent reanalysis of the underlying databases. [Dupuy Institute](https://dupuyinstitute.org/products/duwar-databases/)

**UCDP’s Georeferenced Event Dataset** provides geographically and temporally disaggregated organized-violence events. It is useful for broader event patterns, but it is not a ready-made dataset of opposing tactical strengths and minute-by-minute battle outcomes. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/)

**Correlates of War** is useful for war-level comparisons and broader demographic checks. Its war-level records should not be treated as individual battle observations. [Correlates of War](https://correlatesofwar.org/data-sets/COW-war/)

### What remains genuinely uncertain

The reviewed evidence does not establish universal values for morale collapse, leadership effectiveness, terrain protection, pursuit lethality, or preindustrial wounded-to-killed ratios. Ancient reports, archaeological assemblages, modern interviews, and administrative casualty records each measure different things.

The practical response is to preserve those distinctions: use context-specific calibration, explicit uncertainty, hierarchical parameters where data permit, and sensitivity tests where they do not.

**For TCE v1, prioritize local exposure, integer casualties, cohesion and withdrawal, reachable pursuit, and persistent wounds and captives.** Those mechanisms can generate both limited encounters and disastrous defeats without making annihilation the default—or concealing the battle inside an arbitrary winner–loser multiplier.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9295a-eba4-83e9-884f-a1c31c837a64)
