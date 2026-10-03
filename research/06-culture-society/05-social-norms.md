# Social norms: emergence, enforcement, and change

## A simulation-ready report for The Civilization Engine

**Recommendation:** represent a norm as a **context-specific prescription sustained by social expectations**, not as a settlement-wide morality score or a record of what most people happen to do. Keep private beliefs, perceived expectations, public behavior, informal sanctions, and legal enforcement separate. Bicchieri’s account explains conditional compliance; Young’s evolutionary models explain how locally self-reinforcing conventions emerge and persist. Together, they provide a useful foundation for TCE—but neither supplies universal numerical coefficients for human behavior. [Institute for Reproductive Health](https://www.irh.org/wp-content/uploads/2016/09/Bicchieri_MeasuringSocialNorms.pdf)

For TCE, the central feedback loop should be:

**Action → observation and interpretation → communication → changed expectations → rewards, sanctions, or reconciliation → future choices.**

Different institutions and technologies alter this loop. They should not automatically replace one historical “norm regime” with another.

---

## 1. Mechanisms: rules a simulation can implement

### 1.1 Distinguish norms from habits, incentives, and private convictions

The same observable behavior can have different causes. A citizen might return a borrowed tool because they value honesty, fear exclusion, expect reciprocal help, obey a law, or simply follow an established routine.

Bicchieri distinguishes **empirical expectations**—beliefs about what relevant others do—from **normative expectations**—beliefs about what those others think one ought to do. Socially conditional preferences depend on these expectations. Private moral convictions need not. Her experimental work with Xiao also shows that empirical and normative information can have different effects, rather than acting as interchangeable “social pressure.” [Institute for Reproductive Health](https://www.irh.org/wp-content/uploads/2016/09/Bicchieri_MeasuringSocialNorms.pdf)

**Implementable rule:** maintain separate motivations and beliefs:

| Component | Example concerning an unofficial payment |
| --- | --- |
| Material incentive | “Paying will obtain the permit sooner.” |
| Private endorsement | “Accepting this payment is wrong.” |
| Empirical expectation | “Most officials accept such payments.” |
| Normative expectation | “My colleagues think officials should accept them.” |
| Expected social consequences | “Refusing may make my colleagues distrust me.” |
| Expected legal consequences | “An inspector might prosecute me.” |

This permits **disapproved but common behavior**, **privately rejected but publicly enforced norms**, and **legal behavior that attracts social punishment**.

Do not classify every repeated activity as a norm. A useful diagnostic is counterfactual: would the agent change their choice if expectations changed while material opportunities remained constant?

### 1.2 Conventions emerge through repeated coordination

Young’s models show how conventions can emerge from adaptive behavior without central agreement. People sample previous interactions, select responses that appear advantageous, and occasionally experiment or make mistakes. Local reinforcement creates persistence; sufficiently consequential disturbances can move the population toward another convention. Which convention survives depends on the game, information structure, and adjustment process—not simply on which outcome maximizes collective welfare. [JSTOR](https://www.jstor.org/stable/2951778)

**Implementable rule:** when a recurring coordination problem has several workable solutions, agents learn from recent relevant encounters.

Examples for TCE include market days, irrigation schedules, customary bargaining divisions, and precedence at shared facilities. Once a convention becomes predictable, following it becomes individually advantageous.

Keep **coordination conventions** distinct from **cooperation norms**. Agreeing which side of a road to use does not present the same enforcement problem as contributing labor to a canal while others can free-ride.

### 1.3 Cultural evolution changes strategies without requiring genetic evolution

Axelrod’s evolutionary norms model lets strategies spread through differential success and examines how punishment and **metanorms**—sanctioning people who fail to enforce a norm—can support compliance. It also exposes a central difficulty: enforcing a norm can itself be costly, creating another opportunity to free-ride. Metanorms are a possible mechanism, not an automatic solution. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/an-evolutionary-approach-to-norms/2B829FB347BBDD0F1A8C0F325EFB6F7B)

**Implementable rule:** separate three processes:

* **Behavioral learning:** changing what one does after observing consequences.
* **Social learning:** adopting practices or judgments from trusted, successful, prestigious, or authoritative people.
* **Intergenerational transmission:** children learning from caregivers, peers, teachers, and workplace mentors.

These are proposed implementation categories, not fixed universal learning weights. TCE does not need genetic evolution to generate centuries of cultural change.

Allow the successful transmission of harmful or unequal norms. A rule can benefit its enforcers, protect an organization, or coordinate an influential coalition without benefiting everybody.

### 1.4 Informal enforcement is a choice with costs

Punishment should not occur automatically whenever an observer detects a violation.

In Fehr and Gächter’s public-goods experiment, participants sometimes paid to punish free-riders despite the absence of future material rewards from those partners. But Herrmann, Thöni, and Gächter found substantial variation across 16 participant pools, including **antisocial punishment of high contributors** that could undermine punishment’s cooperative benefits. “More punishment” is therefore not a reliable proxy for “better cooperation.” [EconWPA](https://econwpa.ub.uni-muenchen.de/econ-wp/mic/papers/0305/0305006.pdf)

**Implementable rule:** an observer chooses among ignoring, questioning, gossiping, confronting, withdrawing cooperation, seeking mediation, reporting to authorities, and coercion.

A sanction becomes more attractive when its expected benefits—protecting resources, preserving a relationship, gaining approval, satisfying conviction, deterring recurrence—outweigh its costs.

| Channel | Costs TCE should represent | How it changes incentives |
| --- | --- | --- |
| Gossip | Conversation time, credibility risk, retaliation if identified | Changes what particular listeners believe about the target |
| Public shaming | Confrontation risk, audience disagreement, damaged relationships | Makes disapproval visible and can coordinate others’ responses |
| Ostracism or partner refusal | Foregone exchange, labor, companionship, or assistance for both sides | Removes concrete opportunities rather than subtracting an abstract reputation score |
| Direct coercion | Injury, resource expenditure, retaliation, legal exposure | Imposes immediate losses and can escalate conflict |
| Mediation and restitution | Negotiation time, compensation, contributions from supporters | Repairs losses and makes renewed cooperation possible |

The table is a proposed economic representation; there are no defensible universal historical prices for these channels.

### 1.5 Gossip must transmit evidence, not global truth

Feinberg, Willer, and Schultz experimentally demonstrated that gossip can help people identify uncooperative partners and that exclusion can improve group cooperation. Their results also support allowing excluded individuals to change behavior and re-enter cooperation. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/0956797613510184)

**Implementable rule:** gossip transmits a claim containing an event, source, target, interpretation, and confidence. Listeners update selectively.

A report that “this person took grain” should not automatically mean “this person is a thief.” Listeners may disagree about ownership, emergency entitlements, witness reliability, or whether the taking was authorized.

Preserve enough provenance to distinguish ten independent reports from ten repetitions of the same rumor. Otherwise, the simulation will manufacture certainty through message duplication.

Reputation should also be **audience- and domain-specific**. The same dissident can gain standing among reformers and lose standing among officeholders. A reliable trading partner need not be considered a suitable spouse or political leader.

### 1.6 Power and dependence determine which sanctions are feasible

In a two-week experience-sampling study, Molho and colleagues collected 1,507 norm-violation reports from 257 Dutch adults. Gossip and avoidance were associated with more powerful offenders, while direct confrontation depended on circumstances including the relationship and whether the observer was personally affected. The study is observational and modern, but it supports distinguishing sanction channels rather than using one punishment probability. [Nature](https://www.nature.com/articles/s41467-020-17286-2)

**Implementable rule:** calculate retaliation risk and relationship dependence before choosing a sanction.

A tenant may privately condemn a landlord yet remain silent. Several tenants acting together may confront the landlord. A merchant with many alternative partners can exclude someone at lower cost than a household dependent on its only nearby miller.

Also allow reconciliation. Wiessner’s study of Enga customary courts found that third parties commonly helped with compensation and reintegration, while direct third-party punishment was rare. Repeated wrongdoers received less support. This is a different enforcement system from endlessly escalating fines or permanent exile. [ResearchGate](https://www.researchgate.net/publication/347420827_The_role_of_third_parties_in_norm_enforcement_in_customary_courts_among_the_Enga_of_Papua_New_Guinea)

### 1.7 Tightness is a property of expectations and tolerance—not moral quality

Gelfand and colleagues define cultural tightness through strong norms and relatively little tolerance of deviation. Their 33-country study connects this construct with perceived situational constraint and ecological and historical conditions, but its cross-sectional associations do not establish a universal causal coefficient from threat to conformity. [Science](https://www.science.org/doi/10.1126/science.1197754)

**Implementable rule:** derive tightness separately by group and domain from:

1. Agreement about the applicable prescription.
2. Perceived pressure to comply.
3. The likelihood and severity of responses to deviation.

Do not equate tightness with honesty, kindness, effective government, or low violence. A group can tightly enforce bribery, silence, hospitality, or mutual aid.

Roos and colleagues provide an evolutionary model in which threats can favor stronger adherence and punishment. Treat this as a mechanism to test: threat may raise coordination benefits, but TCE should also allow deprivation to weaken institutions or make compliance infeasible. [IDEAS/RePEc](https://ideas.repec.org/a/eee/jobhdp/v129y2015icp14-23.html)

### 1.8 Norm changes can be gradual underneath and abrupt in public

Private opinion, perceived public opinion, and public behavior can move at different speeds.

In Saudi Arabia, Bursztyn, González, and Yanagizawa-Drott found that many men underestimated other men’s support for women working outside the home. Correcting those beliefs changed employment-related decisions. This demonstrates that an apparent conservative consensus can partly reflect mistaken expectations rather than uniformly conservative private preferences. [University of Chicago Home](https://home.uchicago.edu/bursztyn/Misperceived_Norms_2020_3_6.pdf)

**Implementable rule:** allow privately sympathetic agents to remain inactive until they expect sufficient support, protection, or reciprocity. A public announcement can change these expectations without directly changing private values.

Centola and colleagues experimentally demonstrated tipping in a coordination convention, but the critical minority depended on the experimental conditions. Their result is not a universal population percentage for revolutions, moral reform, or legal change. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf)

For TCE, changes should originate through events: repeated disputes, visible successful deviations, migration, altered economic opportunities, new authorities, public commitments, institutional reform, or replacement of older cohorts. Network structure and heterogeneous willingness to move first determine whether these changes remain local or spread.

### 1.9 Law affects norms through several distinct pathways

Legal change can alter incentives, communicate an official judgment, protect people who challenge existing practices, or change opportunities. These pathways need not produce the same psychological response.

Tankard and Paluck’s study of the 2015 US same-sex-marriage ruling found increased perceived social support after the decision without corresponding change in personal attitudes in its longitudinal sample. Institutional signals can therefore shift perceived norms without immediate private conversion. [Princeton University](https://collaborate.princeton.edu/en/publications/the-effect-of-a-supreme-court-decision-regarding-gay-marriage-on-/)

Conversely, Gneezy and Rustichini’s daycare experiment found that introducing a small lateness fine increased late arrivals. A plausible interpretation is that a social obligation was reframed as a purchasable service; this is a specific finding, not a general rule that fines backfire. [Amazon Web Services, Inc.](https://s3.amazonaws.com/fieldexperiments-papers2/papers/00258.pdf)

**Implementable rule:** a legal change should separately modify legal knowledge, expected enforcement, institutional legitimacy, and beliefs about others’ approval. Never assign all four the same update.

Enforcers must also be agents. The law’s effective reach depends on reporting, evidence, staffing, adjudication, collection, favoritism, and resistance—not just the statute’s nominal penalty.

---

## 2. Parameters: empirical anchors versus simulation priors

### 2.1 Quantitative empirical anchors

These are **benchmarks in specified settings**, not worldwide defaults. “High” confidence below concerns the reported finding or experimental specification; it does not imply high confidence in transporting the number to an early agrarian settlement.

| Quantity | Value and units | Source and interpretation | Confidence and transferability |
| --- | --- | --- | --- |
| Public-goods interaction | 4 players; 20 monetary units each; each contributed unit returns 0.4 units to every player | Fehr–Gächter experiment. Useful as an isolated replication environment, not a historical production function. [EconWPA](https://econwpa.ub.uni-muenchen.de/econ-wp/mic/papers/0305/0305006.pdf) | High for experimental specification; low historical transfer |
| Costly punishment | Punisher pays 1 unit; target loses 3. **84.3%** punished at least once across six punishment periods | The percentage is not a per-violation probability or a permanent “punisher type” share. [EconWPA](https://econwpa.ub.uni-muenchen.de/econ-wp/mic/papers/0305/0305006.pdf) | High for reported setting; low universal transfer |
| Convention tipping | All 5 groups with committed minorities below 25% failed to overturn the convention; all 5 at 25–31% succeeded, with 72–100% uptake among initially uncommitted participants | Centola et al.; 194 participants across 10 experimental groups. Coordination task with specific incentives. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf) | Moderate for general tipping mechanism; low for universal threshold |
| Short-memory learning | Fitted memory of roughly 9–13 previous interactions predicted about 80% of choices | Same naming-game experiment; an interaction-count benchmark, not a number of days. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf) | Moderate within task; low cross-domain transfer |
| Cultural tightness index | Reported country scores ranged **1.6–12.3**; examples: Brazil 3.5, US 5.1, Japan 8.6, Norway 9.5, Pakistan 12.3 | Gelfand et al.; 6,823 respondents, 33 countries. These are study index scores, not sanction probabilities; samples were not nationally representative. [ResearchGate](https://www.researchgate.net/publication/51169484_Differences_Between_Tight_and_Loose_Cultures_A_33-Nation_Study) | Moderate for comparative construct; low as historical calibration |
| Correcting mistaken expectations | In a sample of 500 young married Saudi men, 87% privately supported women working outside the home. Feedback raised wives’ job-service sign-ups by about **9 percentage points** from a 23% control baseline | Bursztyn et al.; selected Riyadh sample, not every Saudi household. [University of Chicago Home](https://home.uchicago.edu/bursztyn/Misperceived_Norms_2020_3_6.pdf) | High for randomized intervention in sample; limited transfer |
| Follow-through after belief correction | Wives’ reported job applications rose from **6% to 16%** after roughly four months | Same study. Applications are not employment; do not substitute an employment effect. [University of Chicago Home](https://home.uchicago.edu/bursztyn/Misperceived_Norms_2020_3_6.pdf) | Moderate; context- and outcome-specific |
| Small-fine reversal | **10 NIS** fine for lateness of at least 10 minutes; late arrivals nearly doubled and remained elevated during four weeks after removal | Gneezy–Rustichini: 10 daycares, 20 weeks, 6 treatment centers. [Amazon Web Services, Inc.](https://s3.amazonaws.com/fieldexperiments-papers2/papers/00258.pdf) | Moderate; small number of institutional clusters |
| Legal enforcement effect | Diplomatic **unpaid parking violations fell by over 98% on average** after enforcement strengthened in 2002 | Fisman–Miguel. Outcome combines parking and payment behavior; it does not demonstrate a comparable change in private norms. [NBER](https://users.nber.org/~rdehejia/%21%40%24devo/Lecture%2011b%20Corruption/supplemental/fisman_miguel_parking.pdf) | Moderate-to-high for behavioral response; low transfer of magnitude |

The gaps matter. These sources do **not** establish a universal daily gossip rate, a universal shaming cost, a norm’s average lifetime, or an era-specific probability of obeying custom over law.

### 2.2 Proposed TCE starting ranges

The following are **engineering priors for sensitivity analysis**, not empirical estimates. Their purpose is to make uncertainty explicit and produce a tractable first implementation.

| Parameter | Suggested starting range | Units and implementation meaning | Basis and confidence |
| --- | --- | --- | --- |
| Empirical-expectation threshold | 0.2–0.9 | Fraction of weighted reference contacts believed to comply | Heterogeneous conditional-compliance design; numerical range proposed |
| Normative-expectation threshold | 0.2–0.9 | Fraction believed to endorse the prescription | Keep independent from the empirical threshold; proposed |
| Belief learning rate | 0.02–0.30 | Weight placed on one credible, substantially independent observation | Proposed; use evidence quality and uncertainty |
| Recent-event learning window | 8–32 | Informative encounters, not calendar days | Broad sensitivity range around a short-memory implementation; only the task-specific anchor above is empirical |
| Exploration or novel proposal probability | \(10^{-4}\)–\(10^{-2}\) | Per applicable deliberative decision | Proposed; not per animation frame or every routine action |
| Routine reputation-memory half-life | 30–730 | Simulated days | Proposed; distinguish ordinary breaches from severe or institutionally recorded events |
| Sanction leverage | 0.1–10 | Target’s material loss divided by enforcer’s material cost | Stress-test range; calculate concrete losses where possible rather than applying globally |
| Active norm–reference-group records | 16–32 | Records per agent | Computational prior, not a psychological capacity claim |
| Active social ties | 16–64 | Directed ties per agent | Computational prior; retain important household and institutional ties separately |
| Gossip forwarding cap | 1–3 | Recipients per selected report event | Computational throttling parameter; not an ethnographic estimate |

Do not independently randomize every parameter. For a proposed social ecology, correlate mechanisms coherently: repeated dependence may increase the value of a relationship, the cost of exclusion, and the importance of credible reputation.

**Calibrate observability from the world itself.** Visibility, shared workplaces, household composition, record-keeping, literacy, and travel should generate opportunities to learn. A blanket “95% of villagers notice theft” parameter would bypass much of the simulation.

---

## 3. Variation across eras and world regions

The appropriate historical comparison is between **social and economic configurations**, not a universal sequence from “primitive norms” to “modern law.” Recent ethnography can inform mechanisms, but it is not a direct measurement of prehistoric populations.

| Setting | Evidence and limitations | Consequence for TCE |
| --- | --- | --- |
| **Foraging societies** | Wiessner’s analysis of 308 conversations among southern African Ju/’hoansi documents norm regulation through everyday interaction, including reputational processes. It is a particular ethnographic population, not a universal forager template. [Springer](https://link.springer.com/article/10.1007/s12110-005-1000-9) | Make recurrent partners, sharing relationships, informal discussion, and the possibility of moving away important. Do not assume either universal harmony or universally severe punishment. |
| **Early farming and storage** | A comparative study of 131 largely non-industrial societies found material sanctions associated with food storage, while reputational sanctions were associated with egalitarianism and absence of storage. These associations do not directly measure the Neolithic transition. [Cambridge University Press](https://www.cambridge.org/core/journals/evolutionary-human-sciences/article/norm-violations-and-punishments-across-human-societies/F5B13F32B188E8A874A333A19E532615) | Storage creates appropriable assets and changes what can be confiscated or withheld. Land, water, and access dependencies should change enforcement incentives without a scripted “farming causes strictness” transition. |
| **Pre-industrial long-distance commerce** | Greif’s interpretation of eleventh-century Maghribi trading relations centers on reputation and coalition enforcement. Its historical interpretation has been disputed by Edwards and Ogilvie and defended by Greif. It is a valuable mechanism case, not uncontested proof of commerce operating through reputation alone. [ResearchGate](https://www.researchgate.net/publication/227724732_Contract_Enforcement_Institutions_and_Social_Capital_The_Maghribi_Traders_Reappraised?utm_source=chatgpt.com) | Permit reputational networks to cross settlement boundaries. Information delay, alternative partners, organizational membership, and access to courts should determine enforcement capacity. |
| **Small-scale horticultural communities within a state** | Among the Enga of Papua New Guinea, 333 customary court cases showed prominent restitution and reintegration alongside a separate formal judicial system. These are modern observations of institutional coexistence, not untouched pre-state history. [ResearchGate](https://www.researchgate.net/publication/347420827_The_role_of_third_parties_in_norm_enforcement_in_customary_courts_among_the_Enga_of_Papua_New_Guinea) | Local mediation and national law can coexist, compete, or divide jurisdiction. Courts need not eliminate informal enforcement or impose purely punitive outcomes. |
| **Industrialization** | Thompson’s historical analysis connects changing work discipline with industrial capitalism in England. It supplies a historically situated account, not a universal industrialization coefficient. [OUP Academic](https://academic.oup.com/past/article/38/1/56/1454624) | As a design inference, synchronized workplaces should make punctuality and attendance more consequential, even while urban anonymity weakens some neighborhood sanctions. Model the organization, not an “industrial culture” switch. |
| **Modern societies across regions** | Tightness scores vary among modern countries; Japan and Norway both scored above the US and Brazil in the original study. A separate 57-country study found differing preferences for gossip, ostracism, and physical confrontation. [ResearchGate](https://www.researchgate.net/publication/51169484_Differences_Between_Tight_and_Loose_Cultures_A_33-Nation_Study) | Modernization must not automatically produce looseness. Preserve multiple overlapping reference groups and different sanction-channel preferences. |

**The thinnest historical evidence concerns private expectations.** A burial custom, law code, or repeated architectural form can document a practice or official prescription, but it cannot by itself identify whether people privately approved, feared sanctions, or misunderstood others’ views. For early farming especially, mechanism-based scenarios are more defensible than precise “norm compliance rates.”

A further caution: present-day country differences should not become inherited ethnic traits in TCE. Generate them through institutions, experiences, networks, and social learning.

---

## 4. Stylized facts a credible simulation should reproduce

Treat these as **conditional validation targets**, not outcomes required in every generated world.

| Pattern | Empirical anchor | Appropriate simulation test |
| --- | --- | --- |
| **Private opinion, perceived opinion, and action can diverge** | Saudi belief correction changed employment-related choices despite substantial pre-existing private support. [University of Chicago Home](https://home.uchicago.edu/bursztyn/Misperceived_Norms_2020_3_6.pdf) | Hold private preferences and opportunities fixed; alter credible information about peers. Some behavior should change without direct value conversion. |
| **Costly enforcement occurs, but not everyone enforces every violation** | Costly punishment occurred in the Fehr–Gächter experiment. [EconWPA](https://econwpa.ub.uni-muenchen.de/econ-wp/mic/papers/0305/0305006.pdf) | Some agents enforce without immediate material gain; others free-ride, negotiate, or avoid danger. |
| **Punishment can harm cooperation** | Antisocial punishment varied across 16 participant pools and could erase cooperative gains. [PubMed](https://pubmed.ncbi.nlm.nih.gov/18323447/) | Allow retaliatory and conformity-enforcing sanctions against high contributors or reformers. Track net welfare after sanction costs, not just contributions. |
| **Power changes enforcement channels** | Daily-life evidence links powerful offenders with indirect rather than straightforward confrontational responses. [Nature](https://www.nature.com/articles/s41467-020-17286-2) | Increase offender power while preserving the violation. Direct confrontation should become less attractive; silence, gossip, or coalition-building may increase. |
| **Exclusion need not be permanent** | Gossip-and-ostracism experiments and Enga dispute resolution support behavioral adjustment or reintegration. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/0956797613510184) | Provide routes back through changed conduct, restitution, sponsorship, or new relationships. |
| **Collective change can be discontinuous** | Convention experiments show tipping under specified conditions. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf) | Sweep minority commitment, switching costs, memory, and network structure. Obtain different thresholds—and cases with no successful cascade. |
| **Law can change perceived norms before private attitudes** | Tankard–Paluck’s institutional-change evidence. [Princeton University](https://collaborate.princeton.edu/en/publications/the-effect-of-a-supreme-court-decision-regarding-gay-marriage-on-/) | Update public expectations and legal incentives independently of internalized endorsement. |
| **Enforcement can strongly change measured violations without proving moral conversion** | Diplomatic unpaid parking violations dropped dramatically after stronger enforcement. [NBER](https://users.nber.org/~rdehejia/%21%40%24devo/Lecture%2011b%20Corruption/supplemental/fisman_miguel_parking.pdf) | Increase enforceability while holding private norms fixed. Behavior should respond, but reversibility should remain possible. |

Also test the **measurement system**. Record actual acts, witnessed acts, complaints, convictions, public statements, and private beliefs separately. A reduction in complaints can result from intimidation or reduced trust, rather than reduced offending. This is an accounting requirement for interpreting TCE’s own outputs.

---

## 5. Modeling recommendation for individual agents and institutions

### 5.1 Use contentful norms with explicit scope

A compact proposed representation is:

```
NormTemplate
    action or obligation
    actor and target roles
    contextual conditions
    possible exceptions

NormInstance
    template
    reference group
    locally recognized interpretation
    version and origin

AgentNormState
    private endorsement
    empirical expectation
    normative expectation
    uncertainty
    conditional-compliance thresholds
    internalization strength

SocialTie
    partner
    relationship and group memberships
    trust
    domain-specific reputation
    recent evidence references

Institution
    jurisdiction and laws
    enforcement procedures
    personnel and budget
    records
    agent-specific perceived legitimacy
```

Examples of authored building blocks might be “return borrowed tools,” “contribute to shared maintenance,” or “permit emergency access to reserves.” The historical outcome emerges from which prescriptions agents propose, interpret, endorse, enforce, resist, and institutionalize.

**Ownership and entitlement must enter the context.** Taking grain from a household store, a public reserve, an abandoned building, or an employer withholding wages should not automatically trigger an identical “theft” judgment.

### 5.2 Use a bounded, multi-motive choice rule

One proposed decision model is:

\[
U\_i(a,c)=
M\_i(a,c)
+\mu\_i V\_i(a,c)
+\gamma\_i A\_{in}R\_{in}(a,c)
+\mathbb{E}\_i[S\_i(a,c)]
-\mathbb{E}\_i[L\_i(a,c)]
-H\_i(a,c).
\]

Here:

| Term | Meaning |
| --- | --- |
| \(M\_i\) | Expected material consequences: food, money, time, safety, future exchange |
| \(V\_i\) | Compatibility with private convictions |
| \(A\_{in}\) | Activation of a socially conditional preference for norm \(n\) |
| \(R\_{in}\) | Compatibility with the perceived prescription |
| \(S\_i\) | Social rewards minus informal sanction costs |
| \(L\_i\) | Expected formal legal loss |
| \(H\_i\) | Switching, effort, or habit-disruption cost |

For a socially conditional norm, an optional smooth activation is:

\[
A\_{in}=
\sigma\!\left(\frac{e\_{in}-\theta^{e}\_{in}}{w\_e}\right)
\sigma\!\left(\frac{q\_{in}-\theta^{q}\_{in}}{w\_q}\right),
\]

where \(e\) is perceived compliance, \(q\) perceived endorsement, \(\theta\) agent-specific thresholds, and \(\sigma\) the logistic function.

This is a **proposed implementation of conditionality**, not Bicchieri’s empirically fitted equation. Some agents should have weak conditional terms and strong private convictions; otherwise principled minorities cannot emerge.

A softmax choice rule can introduce imperfect decision-making:

\[
P\_i(a\mid c)=
\frac{\exp[U\_i(a,c)/\tau\_i]}
{\sum\_b\exp[U\_i(b,c)/\tau\_i]}.
\]

Normalize utility units before tuning \(\tau\). Rescaling every utility and the temperature together leaves behavior unchanged, so an isolated “realistic noise coefficient” has no meaning.

**Avoid double counting.** If exclusion’s foregone trade is already included in material consequences, do not charge the same loss again as an abstract social penalty. Expected losses guide decisions; actual resources change only when consequences occur.

### 5.3 Let sanctioners make their own decisions

Use a separate comparison for an observer:

\[
\Delta U\_i(\text{sanction})=
\text{expected future protection}
+\text{conviction satisfaction}
+\text{audience response}
-\text{direct cost}
-\text{retaliation risk}
-\text{relationship loss}.
\]

All terms are subjective estimates. A sanctioner can misjudge the audience or offender.

A crowd’s participation should then emerge from individual incentives and expectations. Do not automatically convert “several witnesses” into a coordinated punishment group.

For v1, metanorms can be restricted to explicitly demanding organizations: a watch group expecting members to report misconduct, or a guild requiring members to honor exclusions. Avoid recursively generating penalties for every failure to punish another failure to punish.

### 5.4 Update information through events

A practical sequence is:

**Resolve action → determine witnesses → let witnesses interpret it → generate selected reports → update listener beliefs → resolve responses → update relationships and opportunities.**

For a credible observation \(x\), a simple belief update is:

\[
b\_{t+1}=b\_t+\alpha\,w\_{\text{credibility}}\,(x-b\_t).
\]

Treat this as an approximation. Track uncertainty and discount repeated reports from the same origin. A direct observation of behavior updates empirical expectations more directly than it reveals private approval; public statements also require interpretation because speakers may conceal their preferences.

Children and newcomers should learn through accessible people and institutions. Neither should receive perfect knowledge of a settlement’s actual statistical distribution.

### 5.5 Make tightness an output before making it an input

Maintain a dashboard vector rather than one hidden cultural multiplier:

```
prescription agreement
perceived pressure
deviation tolerance
sanction-channel mix
public/private belief gap
within-group versus between-group disagreement
```

A settlement-level summary may be useful for the player, but agent decisions should normally use local beliefs. Otherwise, aggregate diagnostics become a telepathic information channel.

The 57-country sanction-preference study is particularly useful here: preferences for different sanction channels do not collapse neatly into one measure of general punitiveness. [InK](https://ink.library.smu.edu.sg/soss_research/3446/)

### 5.6 Keep computation local and event-driven

For the stated population, the proposed representation need not require all-to-all processing.

At 50,000 agents, **24 norm-state records × 32 bytes** require about **38.4 MB**. **32 directed ties × 32 bytes** require another **51.2 MB**. The combined **89.6 MB** is only arithmetic for packed base records; it excludes histories, indexes, allocation overhead, and other simulation systems. It is not a performance benchmark.

Use cached routine choices until meaningful information or circumstances change. Epstein’s *Learning to Be Thoughtless* explicitly models how established conventions can reduce individual computation, while disagreement reactivates attention. Its results are model demonstrations, not measured human processing budgets. [Brookings](https://www.brookings.edu/wp-content/uploads/2016/06/thoughtless.pdf)

For TCE:

* Process salient norm events, not every norm for every agent every day.
* Keep contacts and evidence sparse; summarize old routine history.
* Let Unreal consume simulation snapshots rather than determine social outcomes.

Preserve reproducible event ordering and seeded randomness so that apparent norm cascades can be debugged rather than attributed to rendering or scheduling artifacts.

### 5.7 Existing models and games worth borrowing from

| Model or system | What to borrow | What not to assume |
| --- | --- | --- |
| **Young: evolutionary conventions** | Local learning, historical contingency, stable conventions and transitions | That equilibrium selection guarantees justice or maximum welfare. [JSTOR](https://www.jstor.org/stable/2951778) |
| **Axelrod: norms and metanorms** | Separate violation, enforcement, and enforcement-of-enforcement decisions | That metanorms solve all collective-action problems. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/an-evolutionary-approach-to-norms/2B829FB347BBDD0F1A8C0F325EFB6F7B) |
| **Epstein: Learning to Be Thoughtless** | Adaptive attention and cheap routine compliance | That binary coordination captures morality, power, and contested interpretation. [Brookings](https://www.brookings.edu/wp-content/uploads/2016/06/thoughtless.pdf) |
| **Roos et al.: threat and norm strength** | Coupling ecological conditions to cooperation and enforcement incentives | That the model establishes a universal empirical threat coefficient. [IDEAS/RePEc](https://ideas.repec.org/a/eee/jobhdp/v129y2015icp14-23.html) |
| **Centola et al.: experimental convention change** | A bounded test environment for minority-driven switching | A global 25% revolution trigger. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf) |
| **Prom Week** | Authored social considerations producing context-dependent interactions and stories | Historical validation or demonstrated scalability to TCE’s population. [Aaron A. Reed](https://aaronareed.net/prom-week/) |

### 5.8 A minimal TCE scenario: bribery without a “corruption culture” switch

Consider officials who control access to a scarce service.

Initially, several accept gifts because the material benefit is attractive and discovery is unlikely. Colleagues observe successful gift-taking and revise their expectations. Some begin interpreting participation as loyalty, while clients learn that refusal produces delay. An informal practice can become socially enforced inside the office even while households condemn it.

A reforming ruler raises penalties. That alone may fail if colleagues conceal evidence and clients fear retaliation. A different intervention—reliable salaries, reduced discretion, accessible records, protected reporting, and visible successful refusals—changes several mechanisms simultaneously.

The same simulation should also permit reform to fail, enforcement to become selective, or honest officials to be ostracized. Those outcomes follow from agents and institutions, not an assigned national corruption trait.

---

## 6. Sources, datasets, and limits of the evidence

### Priority sources and reusable data

| Research resource | Best use for TCE | Main limitation |
| --- | --- | --- |
| **Bicchieri, *Norms in the Wild*; Bicchieri–Xiao, “Do the Right Thing: But Only If Others Do So”** | Defining and diagnosing conditional preferences, empirical expectations, and normative expectations | Definitions and experimental identification do not supply universal coefficients. [OUP Academic](https://academic.oup.com/book/6479) |
| **Gelfand et al. 2011, “Differences Between Tight and Loose Cultures”**, article and supplements | Comparative tightness construct and measurement questions | Modern, nonrepresentative national samples; avoid historical or ethnic extrapolation. [Science](https://www.science.org/doi/10.1126/science.1197754) |
| **Garfield et al. 2023, “Norm Violations and Punishments Across Human Societies”**, supplementary data and `violationsandpunishments` resources | Cross-cultural variation in sanction types; links to socioecology across 131 societies | Documentary presence is not incidence. Missing mentions need not mean absent practices; only selected violation domains were coded. [Cambridge University Press](https://www.cambridge.org/core/journals/evolutionary-human-sciences/article/norm-violations-and-punishments-across-human-societies/F5B13F32B188E8A874A333A19E532615) |
| **Eriksson et al. 2021, “Perceptions of the Appropriate Response to Norm Violation in 57 Societies”** | Sanction-channel preferences; 22,863 respondents | Convenience samples and judgments of appropriateness, not observed probabilities of acting. [InK](https://ink.library.smu.edu.sg/soss_research/3446/) |
| **Centola et al. 2018**, replication data and naming-game model | Micro-level validation of convention switching and learning | Short experimental task with specified incentives. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf) |
| **Bursztyn et al. 2020**, AER article and replication materials | Testing mistaken expectations and information interventions | Specific institutions, population, and outcomes. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20180975) |
| **Molho et al. 2020, “Direct and Indirect Punishment of Norm Violations in Daily Life”** | Relational and situational predictors of sanction choice | Self-reported modern observations; associations are not intervention effects. [Nature](https://www.nature.com/articles/s41467-020-17286-2) |

### Contested claims and thin evidence

**Punishment’s evolutionary interpretation remains contested.** Costly punishment in a laboratory does not by itself identify the evolutionary process that produced it, establish that motives are purely altruistic, or show that uninvolved third parties ordinarily punish in everyday life. Pedersen and colleagues provide a critical experimental examination, while ethnographic work also emphasizes mediation, mixed motives, and relationship repair. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3619453/)

**Threat does not provide a historically calibrated tightness dial.** The cross-national evidence and evolutionary models motivate a hypothesis, but they do not establish how much a famine, epidemic, or invasion should change TCE’s conformity parameters. [Science](https://www.science.org/doi/10.1126/science.1197754)

**There is no universal tipping percentage.** Coordination conventions, discriminatory norms, workplace customs, and political dissent have different payoffs, risks, audiences, and informational requirements. The experimental threshold is evidence for a possible nonlinear mechanism, not a portable constant. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf)

**Historical reconstruction is weakest where TCE most needs private mental states.** Use documentary and archaeological evidence to constrain institutions, actions, opportunities, and sanction forms. Treat precise private-belief distributions and learning rates as uncertain latent variables.

### Implementation priority

For v1, build **scoped prescriptions, separate expectations, local information, concrete partner consequences, and independent legal enforcement**. Add sophisticated metanorms, explicit norm entrepreneurs, and detailed cultural-learning biases later.

The decisive test is whether the same underlying system can produce a cooperative village, a corrupt office, a punitive faction, a tolerant trading network, and a reform that either spreads or fails—**without assigning those outcomes in advance**.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92837-4cf8-83ea-ac99-d99f697844b9)
