# Personality, values, and ideology for The Civilization Engine

## Executive recommendation

**Keep the Big Five as TCE’s five broad personality traits, but do not make them carry the entire behavioral model.** Add a compact value system, distinguish temporary states from lasting dispositions, and derive political positions from the issues and institutions that actually exist in each world.

For an initial implementation, I recommend **five personality traits plus five value contrasts**, with separate residual variation in risk preferences. The first additional personality variable worth considering is **Honesty–Humility**, because cooperation and agreeableness do not adequately distinguish an accommodating person from an honest one. Research comparing personality models finds a substantial advantage for Honesty–Humility when predicting workplace exploitation and misconduct—although that evidence is not a universal model of criminal behavior. [Tilburg University Research Portal](https://research.tilburguniversity.edu/en/publications/a-meta-analysis-of-the-relations-between-personality-and-workplac/)

The proposed five-value compression below is **an engineering synthesis, not a validated universal ten-dimensional model of human beings**. Its components have empirical foundations; their particular combination needs simulation testing.

The central design principle is:

> **Personality affects how an agent pursues goals; values affect which outcomes matter; beliefs affect what the agent expects; institutions determine what is possible and costly.**

No fixed “left–right gene,” national personality template, or personality-to-occupation lookup is necessary.

---

## 1. Mechanisms: what to represent and how it should affect behavior

### 1.1 What the competing frameworks contribute

| Framework | What it measures | Best use in TCE | Main limitation |
| --- | --- | --- | --- |
| **Big Five** | Openness, Conscientiousness, Extraversion, Agreeableness, Neuroticism | Broad behavioral style: exploration, persistence, sociability, interpersonal accommodation, sensitivity to negative affect | Broad traits omit important narrower differences. Their measured five-factor structure does not transfer uniformly across populations. [Chapman University Digital Commons](https://digitalcommons.chapman.edu/esi_pubs/216/) |
| **HEXACO** | Honesty–Humility, Emotionality, Extraversion, Agreeableness, Conscientiousness, Openness | Particularly useful for exploitation, fairness, greed avoidance, anger, and forgiveness | It is **not simply the Big Five plus honesty**. Emotionality and Agreeableness divide some content differently from Big Five Neuroticism and Agreeableness. [HEXACO](https://hexaco.org/scaledescriptions) |
| **Schwartz values** | Ten broad motivational priorities, arranged around a circular structure of compatible and conflicting goals | The strongest starting vocabulary for goals that apply across occupations and political systems | Values are **relative priorities**, not ten independent virtues. Compressing the circle into two axes loses distinctions. [ScholarWorks](https://scholarworks.gvsu.edu/orpc/vol2/iss1/11/) |
| **Moral Foundations** | Different kinds of moral concern and justification | Tags on laws, rituals, punishments, and disputed actions | Do not assume universal, independent biological modules. The newer MFQ-2 measures **care, equality, proportionality, loyalty, authority, and purity**; this is not the original five plus liberty. [NSF Public Access Repository](https://par.nsf.gov/biblio/10531651-morality-beyond-weird-how-nomological-network-morality-varies-across-cultures) |
| **Authority–liberty; hierarchy–equality** | Acceptance of coercive conformity and unequal social standing | Separate preferences about obedience, domination, political rights, and rank | Obeying established authority and wanting one’s group to dominate others are distinguishable motives, emphasized respectively by authoritarianism and social-dominance research. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/10478400903028540) |
| **Left–right** | A historically and institutionally organized bundle of political positions | A summary of an existing political field, useful for the interface | Not a timeless psychological axis. In a study covering **99 nations**, culturally and economically right-wing attitudes more commonly correlated negatively than positively. [Cambridge University Press](https://www.cambridge.org/core/journals/british-journal-of-political-science/article/are-cultural-and-economic-conservatism-positively-correlated-a-largescale-crossnational-test/83AFEDEA5E004CF23631C5388E7C9F67) |

Schwartz’s ten basic values are **self-direction, stimulation, hedonism, achievement, power, security, conformity, tradition, benevolence, and universalism**. Their two major contrasts are **openness to change versus conservation**, and **self-transcendence versus self-enhancement**. Neither contrast is equivalent to a Big Five trait with a similar name. [ScholarWorks](https://scholarworks.gvsu.edu/orpc/vol2/iss1/11/)

### 1.2 Recommended compact value representation

Use the following five contrasts initially. The last column identifies the **proposed simulation interpretation**, not an experimentally established coefficient.

| Value contrast | Underlying distinction | Proposed consequences |
| --- | --- | --- |
| **Change ↔ continuity** | Experimentation and self-direction versus maintaining familiar practices | Willingness to try techniques, revise customs, migrate, or preserve inherited arrangements |
| **Others’ welfare ↔ personal advancement** | Self-transcendence versus self-enhancement | Weight placed on helping, shared welfare, achievement, prestige, and personal gain |
| **Equal standing ↔ ranked standing** | Preference for equal status versus accepted hierarchy | Responses to hereditary office, unequal legal rights, privileged estates, and status distinctions |
| **Freedom from coercion ↔ normative obedience** | Permission to dissent versus enforcing prescribed conduct | Responses to compulsory rituals, censorship, compulsory service, and restrictions on personal conduct |
| **Broad moral concern ↔ in-group priority** | How far obligations extend beyond family and recognized communities | Treatment of strangers, migrants, rival settlements, conquered people, and distant beneficiaries |

**These axes should not be initialized as five demonstrably independent psychological factors.** There is overlap: tradition, authority, security, and hierarchy can be correlated, but they need not move together.

Several distinctions are particularly important:

A person can prefer familiar customs while opposing compulsory observance. Another can support an egalitarian community while demanding strict conformity to its rules. A generous family member can remain indifferent to strangers. A status-seeking reformer can oppose hereditary rank because it blocks personal advancement.

Also preserve **categorical rules alongside continuous values**. “Fairness means equal shares,” “fairness means reward proportional to contribution,” and “fairness means meeting need” are different authored rules. A single equality slider cannot represent all three. Similarly, purity concerns require culturally learned objects and prohibitions; a universal “purity amount” does not specify what is considered impure. The distinction between equality and proportionality is explicitly represented in MFQ-2. [NSF Public Access Repository](https://par.nsf.gov/biblio/10531651-morality-beyond-weird-how-nomological-network-morality-varies-across-cultures)

### 1.3 Convert dispositions into conditional choices, not fixed outcomes

The following is a proposed behavioral mapping, informed by the evidence discussed in Section 2.

| Behavioral domain | Useful disposition inputs | Essential contextual inputs | Implementation rule |
| --- | --- | --- | --- |
| **Risk-taking** | Trait-linked tendencies plus an independent risk residual | Expected returns, perceived probabilities, existing wealth, dependents, possibility of ruin | Evaluate the distribution of consequences. Do not equate curiosity, desperation, and financial risk tolerance. |
| **Crime and exploitation** | Non-exploitation/Honesty–Humility, self-control, anger, status motives | Opportunity, need, victims, detection, sanctions, legitimacy, accomplices | Separate calculated exploitation, impulsive aggression, and violations of locally specific laws. |
| **Entrepreneurship** | Exploration, persistence, networking, risk preference | Capital, knowledge, market access, household insurance, property rules | Model recognizing an opportunity, attempting it, financing it, and succeeding as separate stages. |
| **Political participation** | Weak personality influences, values, political interest | Franchise, participation cost, mobilization, efficacy, perceived stakes | Traits modestly change participation propensity; they do not decide whether an institution permits participation. |
| **Policy support** | Values and identity | Expected household effects, beliefs about effectiveness, legitimacy, coalition cues | Evaluate concrete proposals; derive ideological alignment afterward. |
| **Conformity** | Discomfort with conflict, continuity preference, obedience values | Uncertainty, audience, dependence on others, sanctions, perceived consensus | Distinguish copying for information, outward compliance, and genuine internalization. |

Risk preferences should retain variation beyond the Big Five. Frey and colleagues administered **39 risk measures to 1,507 people** and found a general risk factor alongside domain-specific variation; behavioral tasks and self-reports were not interchangeable. Therefore, “high Openness = high risk-taking everywhere” is a poor compression. [pmc.ncbi.nlm.nih.gov](https://pmc.ncbi.nlm.nih.gov/articles/PMC5627985/?utm_source=chatgpt.com)

Values likewise do not translate into actions with a single universal strength. Bardi and Schwartz found that value–behavior relationships differed across value domains; social expectations can also make behavior less revealing of personal priorities. TCE should therefore allow two agents with different private values to perform the same publicly required act. [PubMed](https://pubmed.ncbi.nlm.nih.gov/15189583/?utm_source=chatgpt.com)

### 1.4 A common decision rule

For a feasible action \(a\), use a utility calculation such as:

\[
U\_i(a)=
M\_i(a)
+\mathbf v\_i^\top \mathbf f\_i(a)
+R\_i(a)
-C\_i(a).
\]

Here:

* \(M\_i(a)\) is expected material and household benefit, including urgent needs.
* \(\mathbf f\_i(a)\) is the agent’s **believed** effect on morally and socially relevant outcomes.
* \(R\_i(a)\) contains relationship, reputation, and status consequences.
* \(C\_i(a)\) contains effort, risk, punishment, and other costs.

Personality modifies components of this calculation and the search process: which opportunities are noticed, how much planning occurs, how conflict is anticipated, and how strongly threats are experienced.

Use stochastic selection among plausible alternatives, but retain **stable individual residual preferences**. Otherwise an agent whose modeled scores are average will behave like a different person every day.

For example, stealing food can become attractive because starvation increases its immediate benefit. That need not lower the agent’s conscientiousness, erase concern for others, or permanently change their ideology.

### 1.5 Make ideology an emergent compression

Represent policy proposals in terms of their actual effects: ownership, taxation, inheritance, representation, movement, worship, punishment, access to resources, and obligations to outsiders.

An agent can then support redistribution because of material interest, solidarity, fear of insecurity, or loyalty to a redistributing ruler. Those mechanisms can produce the same vote without producing the same broader ideology.

Research on authoritarianism and social dominance provides a useful distinction between security-oriented support for conformity and competition-oriented support for group hierarchy. Cross-national political research also shows why cultural traditionalism must be allowed to coexist with economic redistribution. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/10478400903028540)

**Implementation:** periodically summarize the leading dimensions of disagreement among active proposals and factions. Call them “left” and “right” only when such labels acquire an in-world meaning. Before that, “hereditary privilege versus elected office” or “local autonomy versus central levy” may be far more legible.

---

## 2. Parameters: empirical estimates and proposed simulation settings

### Reading the tables

**\(r\)** is an observed correlation. **\(\rho\)** below denotes a correlation corrected for measurement attenuation in the cited meta-analysis. **SD** means standard deviation.

These are **calibration constraints, not causal multipliers**. An observed \(r=-0.18\) does not mean a one-point trait change causes an 18% behavioral change.

Confidence refers to the finding **within its studied domain**. Transfer to ancient societies or TCE’s invented institutions generally deserves lower confidence.

### 2.1 Heritability, stability, and developmental evidence

| Quantity | Estimate or range | Scope and source | Confidence and use |
| --- | --- | --- | --- |
| **Personality heritability** | Approximately **40% of variance** overall; **47%** in twin designs, **22%** in family/adoption designs | Vukasović & Bratko, 2015: 62 effect sizes, over 100,000 participants. These are method-dependent estimates, **not a confidence interval**. [pubmed.ncbi.nlm.nih.gov](https://pubmed.ncbi.nlm.nih.gov/25961374/?utm_source=chatgpt.com) | High that genetic differences contribute; moderate for any single universal percentage |
| **Historical estimates of rank-order stability** | \(r\approx .31\) in childhood, .54 in college years, .64 around age 30, .74 at ages 50–70 | Roberts & DelVecchio, 2000; comparisons adjusted to a **6.7-year interval**. [PubMed](https://pubmed.ncbi.nlm.nih.gov/10668348/) | Useful magnitude references; do not adopt its late-life plateau uncritically |
| **Updated age pattern of stability** | Stability rises early; little evidence of further increase after approximately **25 years** | Bleidorn et al., 2022: rank-order analyses covered **189 studies, 178,503 participants**. [PubMed](https://pubmed.ncbi.nlm.nih.gov/35834197/) | High for broad early-life stabilization; not evidence that adults stop changing |
| **Childhood value heritability** | Self-transcendence **29%**, self-enhancement **47%**, conservation **31%**; no estimated genetic contribution to openness-to-change in this sample | Uzefovsky et al.: **174 pairs of seven-year-old Israeli twins**. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/sode.12155) | Low as a general TCE parameter; illustrates age-, construct-, and population-specific results |
| **Adult value change** | Generally moderate-to-high rank stability; mean changes often small or inconsistent | Schuster et al., 2019: **19 publications, 25 samples**. No universal annual update coefficient follows from this literature. [Arizona State University](https://asu.elsevierpure.com/en/publications/intra-individual-value-change-in-adulthood-a-systematic-literatur/) | Moderate; supports slow change rather than immutability |
| **Personality change around life events** | Generally small, event-specific effects; work-related transitions more consistently associated with change than romantic transitions | Bühler et al.: **44 studies, 89 samples, 121,187 participants**. [Sage Journals](https://journals.sagepub.com/doi/10.1177/08902070231190219) | Moderate; does not justify a fixed “trauma changes all traits by X” rule |

**Rank-order stability and mean-level change are different.** Everyone in a cohort could become somewhat more emotionally stable while largely preserving their ordering relative to one another. The updated longitudinal meta-analysis found particularly consistent increases in emotional stability, but does not support unlimited, identical maturation trajectories. [PubMed](https://pubmed.ncbi.nlm.nih.gov/35834197/)

### 2.2 Behavioral associations

| Outcome and predictor | Quantitative finding | Interpretation for TCE |
| --- | --- | --- |
| **Political conservatism: Openness** | \(r=-.18\) | Weak association, not a political identity assignment. |
| **Political conservatism: Conscientiousness** | \(r=+.10\) | Weaker still. Both estimates come from **73 studies, N=71,895**, Sibley et al., 2012. [researchgate.net](https://www.researchgate.net/publication/256933817_Personality_and_political_orientation_Meta-analysis_and_test_of_a_Threat-Constraint_Model?utm_source=chatgpt.com) |
| **Voting intentions and behavior: Big Five** | Typical associations approximately **\(r=.05\)–.10** | A recent meta-/mega-analysis found that only lower Neuroticism predicted actual voting in its pooled behavioral analysis; intentions had broader trait associations. Meta-analysis: **17 studies, N=65,036**. [PubMed](https://pubmed.ncbi.nlm.nih.gov/42006738/) |
| **Workplace deviance: Honesty–Humility** | Observed \(r=-.394\); corrected \(\rho=-.482\), 95% CI **[−.529, −.436]** | A relatively substantial association, but with workplace misconduct—not all crime. |
| **Workplace deviance: Big Five Conscientiousness** | Observed \(r=-.306\); corrected \(\rho=-.372\) | Useful for reliability and misconduct calibration. |
| **Workplace deviance: Big Five Agreeableness** | Observed \(r=-.294\); corrected \(\rho=-.362\) | Relevant, but not interchangeable with honesty. These three estimates are from Pletzer et al., 2019; source and measurement differences limit comparisons. [Vrije Universiteit Amsterdam](https://research.vu.nl/ws/files/104470376/A_meta_analysis_of_the_relations_between_personality_and_workplace_deviance.pdf) |
| **Entrepreneurial status: combined Big Five** | Multiple correlation **\(R=.37\)** | Entrepreneurs versus managers differed on several traits; Extraversion did not show a significant overall difference. Occupational selection is not a causal success formula. Zhao & Seibert, 2006. [PubMed](https://pubmed.ncbi.nlm.nih.gov/16551182/) |
| **Six-month risk-measure stability** | Average self-reported propensity \(r\approx .68\); behavioral tasks \(r\approx .46\); latent general risk factor \(r\approx .85\) | Measurement method matters. These are not three estimates of an identical observable behavior. Frey et al., 2017. [pmc.ncbi.nlm.nih.gov](https://pmc.ncbi.nlm.nih.gov/articles/PMC5627985/?utm_source=chatgpt.com) |
| **War exposure and experimental cooperation** | In-group association approximately **+.24–.25 SD**; out-group approximately **+.04 SD**, not statistically significant | Bauer et al., 2016. Exposure definitions differ across studies; this is **not** a per-battle modifier or evidence of increased out-group hostility. [Econ at Berkeley](https://emiguel.econ.berkeley.edu/assets/miguel_research/76/Can-War-Foster-Cooperation_for_NBER.pdf) |

The political coefficients are particularly useful safeguards against overpowered personality. In a simple bivariate calculation, \(r=.10\) corresponds to only **1% shared variance**, and \(r=.18\) to approximately **3.2%**. A simulated political system in which personality almost perfectly predicts allegiance would greatly exceed those empirical relationships.

For entrepreneurship, the modern meta-analysis found higher Conscientiousness and Openness, and lower Neuroticism and Agreeableness, among entrepreneurs relative to managers. Those averages should not become occupational gates: substantial individual overlap and between-study heterogeneity remain. [PubMed](https://pubmed.ncbi.nlm.nih.gov/16551182/)

### 2.3 Proposed engineering defaults—not measured human constants

These are starting settings for sensitivity tests. Their units are explicit so that they remain consistent under time acceleration.

| Parameter | Suggested initial setting | Status and rationale |
| --- | --- | --- |
| Broad personality score | Reference-population mean **0**, SD **1** | Numerical convention; preserve a fixed reference rather than restandardizing each settlement annually |
| Displayed value contrast | **−1 to +1**, derived from an unconstrained latent score | Numerical convention; do not repeatedly hard-clip the underlying distribution |
| Additive inherited fraction in a simplified trait model | Start **0.40**; test **0.20–0.50** | Modeling assumption loosely anchored to personality genetics; empirical broad heritability is not necessarily additive heritability |
| Adult personality persistence target | Approximately **\(r=.6\)–.8 over 5–7 years** | Broad calibration envelope, not a universal trait-specific estimate |
| Ordinary value-learning rate in youth | **0.05–0.20 year\(^{-1}\)** toward a weighted social target | Engineering prior; allows formative learning without cloning parents |
| Ordinary adult value-learning rate | **0.005–0.03 year\(^{-1}\)** | Engineering prior; beliefs can change much faster than core priorities |
| Generic permanent trait shift from an isolated adverse event | **0 by default** | Add persistent effects only through a specified, tested event mechanism; do not accumulate automatic personality damage |
| Uncalibrated biological trait differences between cultures | **0 by default** | An absence-of-evidence policy, not a claim that every observed population distribution is identical |

For a learning rate \(\lambda\), convert elapsed time to an update fraction using:

\[
\alpha(\Delta t)=1-e^{-\lambda\Delta t}.
\]

This avoids making value transmission stronger merely because the kernel switches from annual to monthly updates. Discrete encounters and shocks still require explicit handling.

---

## 3. Variation across life courses, experiences, eras, and regions

### 3.1 Life-course and experience mechanisms

**Childhood and adolescence.** Allow greater instability, learning, and dependence on family and peers. Values should not suddenly appear at adulthood: the Israeli twin study detected structured priorities at age seven. That finding supports early development, not importing its exact heritabilities into every society. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/sode.12155)

**Adulthood.** Preserve substantial continuity while allowing changing roles, relationships, and accumulated experiences to alter behavior and dispositions. A new job can immediately change routine and social exposure without immediately changing a broad trait. Longitudinal life-event evidence supports small, heterogeneous associations rather than uniform transformations. [Sage Journals](https://journals.sagepub.com/doi/10.1177/08902070231190219)

**War and trauma.** Separate fear, grief, perceived danger, trust in particular people, group identity, and broad personality. They are not the same variable. Exposure to violence in a Burundi study was associated with greater altruism toward neighbors, greater risk-seeking, and higher discount rates. This is an important counterexample to “trauma makes everyone cautious and antisocial,” but it is not a universal effect of violence or a randomized experiment on war exposure. [Wageningen Research Portal](https://research.wur.nl/en/publications/violent-conflict-and-behavior-a-field-experiment-in-burundi/)

**Prosperity and security.** Let improved material conditions change feasible choices, perceived threats, and socialization environments. Do not implement “wealth increases, therefore everyone becomes liberal.” An analysis of **406,185 respondents in 76 national cultures, 1981–2022**, found global value divergence, especially in tolerance and self-expression, alongside regional convergence. [Nature](https://www.nature.com/articles/s41467-024-46581-5?code=47b99fea-b1b5-458f-b9d0-9f69a8fabfa1&error=cookies_not_supported)

A practical consequence is to distinguish:

\[
\text{observed behavioral change}
=
\text{state change}
+\text{changed opportunities}
+\text{belief updating}
+\text{lasting disposition change}.
\]

Only the last term belongs in the personality vector.

### 3.2 Era-sensitive modeling without era-specific personalities

The following are **scenario-design implications**, not reconstructed psychological averages for historical populations.

| Social setting | What the evidence can support | What TCE should vary |
| --- | --- | --- |
| **Foraging and horticultural communities** | Studies of contemporary small-scale societies show diverse sharing, bargaining, and social expectations. They are analogues with their own histories, not preserved prehistoric minds. [Cambridge University Press](https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/economic-man-in-crosscultural-perspective-behavioral-experiments-in-15-smallscale-societies/6EFFD9263D9A5F2FE5DE9DE8FBBA4988) | Repeated interaction, household dependence, sharing obligations, mobility, reputation, and opportunities to leave or form another group |
| **Early farming and pastoralism** | Comparative research across **21 populations** links wealth transmission and inequality to the importance of material wealth and institutions. Domestication alone does not explain the differences. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2792081/?utm_source=chatgpt.com) | Storage, heritable land and livestock, defensible resources, household bargaining, and emerging interests in inheritance rules |
| **Pre-industrial agrarian states and towns** | Research on eight past and present intensive-agricultural societies identifies land and other material wealth as important channels of persistent inequality. This is evidence about social structure, not a medieval personality distribution. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/pdfplus/10.1086/648658?utm_source=chatgpt.com) | Legal status, patronage, occupational access, religious and civic obligations, dependence on landlords or employers, and exposure to strangers |
| **Industrial settings** | Modern longitudinal evidence can help model occupational transitions, but it does not establish an “industrial conscientiousness bonus.” [Sage Journals](https://journals.sagepub.com/doi/10.1177/08902070231190219) | Workplace discipline, schooling, urban networks, collective organization, household insurance, and new routes into enterprise |
| **Modern settings** | Surveys and panels offer substantially better measurement, while revealing persistent heterogeneity and divergent value trajectories. [Nature](https://www.nature.com/articles/s41467-024-46581-5?code=47b99fea-b1b5-458f-b9d0-9f69a8fabfa1&error=cookies_not_supported) | Mass political communication, broad institutions, specialized careers, larger identity networks, and competing sources of authority |

These transitions need not occur in that order in TCE. The implementable causes are **resource ownership, opportunities, social networks, institutions, and experiences**, not the era label.

### 3.3 Cross-cultural distributions: the important warnings

**First, structural similarity is not guaranteed.** Research among Tsimane forager-horticulturalists in Bolivia did not robustly recover the conventional Big Five structure. This does not show that curiosity or sociability are absent; it questions whether the same questionnaire-derived factor structure is an invariant description. [Chapman University Digital Commons](https://digitalcommons.chapman.edu/esi_pubs/216/)

**Second, apparent differences can be measurement differences.** Laajaj et al. examined **94,751 respondents in 23 low- and middle-income countries**. Common Big Five measures performed poorly in many face-to-face surveys, contrasting with stronger validity in internet samples from the same countries. Response styles, interviewer interactions, and education affected interpretation. Do not turn an unvalidated national questionnaire mean into a population-generation coefficient. [PubMed](https://pubmed.ncbi.nlm.nih.gov/31309152/)

**Third, retain large within-population variation.** The Global Preference Survey measured economic preferences in approximately **80,000 people across 76 countries** and found greater heterogeneity within countries than between countries. Cultural context matters without making everyone in a culture alike. [Harvard Business School](https://www.hbs.edu/ris/Publication%20Files/Quarterly%20Journal%20of%20Economics_269d889b-69bb-4412-a7bf-e5dfa7d7bff7.pdf)

**Fourth, norms are a group property distinct from average personality.** Gelfand et al.’s **33-nation** study distinguishes tighter cultures—with stronger norms and less tolerance of deviance—from looser ones. For TCE, represent norm clarity, monitoring, sanctions, and tolerance directly rather than assigning every member a high obedience trait. Associations with ecological threat do not justify a deterministic “danger produces authoritarian culture” rule. [Science](https://www.science.org/doi/10.1126/science.1197754)

---

## 4. Stylized facts a credible simulation should reproduce

These are validation goals, not events to script into every world.

| Pattern | Target or diagnostic |
| --- | --- |
| **People are recognizable across years, but not frozen** | Match broad longitudinal stability magnitudes while allowing both individual and cohort change. Personality should not reroll after every crisis. [PubMed](https://pubmed.ncbi.nlm.nih.gov/10668348/) |
| **Personality predicts tendencies better than destinies** | Modern political associations should remain broadly weak—around ( |
| **Cooperation depends on social setting** | Small-scale bargaining studies include average offers around a quarter of the stake in some communities and over half in others. Reproduce sensitivity to local expectations, not one universal “fair offer.” [Santa Fe Institute](https://sites.santafe.edu/~bowles/bbs_final.pdf) |
| **In-group cooperation and universal benevolence can diverge** | War-exposure findings support increased cooperation toward an in-group without an equivalent measured increase toward outsiders. Do not automatically convert this into hostility. [Econ at Berkeley](https://emiguel.econ.berkeley.edu/assets/miguel_research/76/Can-War-Foster-Cooperation_for_NBER.pdf) |
| **Ideological bundles vary** | Cultural traditionalism and economic redistribution must be able to coexist. A permanently fixed global left–right alignment would fail the cross-national evidence. [Cambridge University Press](https://www.cambridge.org/core/journals/british-journal-of-political-science/article/are-cultural-and-economic-conservatism-positively-correlated-a-largescale-crossnational-test/83AFEDEA5E004CF23631C5388E7C9F67) |
| **Culture does not eliminate individuality** | Substantial within-culture dispersion should remain, consistent with the economic-preference evidence. [Harvard Business School](https://www.hbs.edu/ris/Publication%20Files/Quarterly%20Journal%20of%20Economics_269d889b-69bb-4412-a7bf-e5dfa7d7bff7.pdf) |
| **Prosperity does not force global value convergence** | Different development paths should remain possible; reproduce conditional change, not compulsory movement toward one endpoint. [Nature](https://www.nature.com/articles/s41467-024-46581-5?code=47b99fea-b1b5-458f-b9d0-9f69a8fabfa1&error=cookies_not_supported) |

Add two **engineering invariants**: population trait variance must not collapse over generations, and changing enforcement must be able to change public behavior without instantly changing everyone’s private values.

---

## 5. Modeling recommendation: agents, inheritance, institutions, and performance

### 5.1 Keep six kinds of state separate

| Layer | Example contents | Proposed update schedule |
| --- | --- | --- |
| **Broad personality** | O, C, E, A, N | Slow developmental updates; explicit persistent-event mechanisms |
| **Values** | The five contrasts; commitments to particular rules | Socialization and consequential experiences |
| **Beliefs** | Whether an authority is competent, whether theft will be detected, whether a crop technique works | Observations, communication, outcomes |
| **Temporary states** | Hunger, fatigue, fear, grief, anger | Fast physiological and affective updates |
| **Relationships and identity** | Trust toward a person, household obligations, membership, recognized outsiders | Interaction- and event-driven |
| **Capabilities and institutional position** | Skills, assets, legal status, rights, obligations | Economic, educational, political, and legal events |

This decomposition is more important than finding a supposedly perfect fifth value axis.

For example, an agent can oppose a ruler because they dislike coercion, believe the ruler incompetent, resent a tax, belong to an excluded group, or hold a personal grievance. A single “rebel tendency” would hide these distinct mechanisms.

### 5.2 Implement inheritance without cloning parents

Heritability describes variation in a studied population under particular conditions. It is not “40% of this person is genetic,” nor an instruction to copy 40% of parental questionnaire scores. Estimates also depend materially on research design. [pubmed.ncbi.nlm.nih.gov](https://pubmed.ncbi.nlm.nih.gov/25961374/?utm_source=chatgpt.com)

A simple **explicitly hypothetical additive model** is:

\[
T\_i(a)=\mu(a)+\sqrt q\,G\_i+\sqrt{1-q}\,E\_i(a),
\]

where \(G\) and \(E\) each have unit variance in the reference population, and \(q\) is the chosen model variance fraction.

Under unrelated, randomly mating parents:

\[
G\_{\text{child}}
=
\frac{G\_{\text{mother}}+G\_{\text{father}}}{2}
+\epsilon,
\qquad
\epsilon\sim\mathcal N(0,1/2).
\]

The innovation term preserves variance. **Simply averaging parental scores repeatedly would make later generations increasingly similar.**

Under these simplified assumptions, with independent environments, parent–child phenotypic correlation is \(q/2\). Thus \(q=.40\) yields correlation approximately **.20**, not .40. This is a mathematical property of the proposed model, not a claim about every observed family relationship.

Assortative mating, related parents, correlated environments, and multiple correlated traits require corresponding covariance adjustments.

Political-attitude twin studies also find genetic contributions, but that does not imply genes encoding particular parties or historical policy packages. Hatemi et al.’s analysis involved over **12,000 twin pairs in five democracies**; even its results differed for explicit left–right self-placement. Transfer beyond those settings requires caution. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4038932/)

For TCE, transmit values through **family interaction, peers, institutions, and lived outcomes**, with predispositions affecting receptiveness. Do not inherit an ideology identifier.

### 5.3 Use social transmission that permits both persistence and disagreement

A serviceable value update is:

\[
\mathbf v\_i(t+\Delta t)
=
\mathbf v\_i(t)
+
\alpha\_i
\left[
\sum\_j w\_{ij}\mathbf v\_j(t)-\mathbf v\_i(t)
\right]
+
\Delta\mathbf v\_{\text{experience}}
+
\boldsymbol\epsilon\_i.
\]

The weights should depend on actual relationships and encounters: attention, trust, prestige, dependency, perceived success, and membership.

This equation alone is insufficient. Add competing sources, incomplete attention, enduring commitments, and occasional new interpretations. Otherwise averaging drives the population toward bland consensus.

Also distinguish **influence on beliefs** from **influence on values**. A respected farmer demonstrating a new technique may change beliefs about its productivity without making an observer generally more favorable to novelty.

### 5.4 Institutions should act through several mechanisms

For the initial kernel, institutions should modify:

**Feasibility:** who can own land, enter an occupation, vote, move, inherit, or marry.

**Payoffs:** taxes, compensation, access to common resources, and expected penalties.

**Information and expectations:** what people observe, believe others accept, and expect authorities to enforce.

**Socialization and selection:** schools, households, apprenticeships, religious organizations, migration, and recruitment into offices.

These mechanisms can create different public cultures from similar initial personality distributions. They also let policy effects be explained to the player: “complies because detection is likely” differs from “believes the rule is right.”

Do not define crime as a permanent personality output. Record the underlying act, its victims and consequences, its legal classification, and whether it was detected. The same act can be legal under one institution and prohibited under another.

### 5.5 What to simplify for 10k–50k agents

Five traits and five value scores stored as 32-bit floats cost **40 bytes per agent**, or **2.0 MB for 50,000 agents**, excluding all other state. Adding one honesty-related scalar costs another **0.2 MB**. The psychological state vector itself does not require aggressive compression.

Recommended simplifications are computational rather than conceptual:

* Evaluate a small set of feasible actions rather than every imaginable action.
* Cache habitual choices and stable evaluations; reconsider them when needs, prices, relationships, or institutions change.
* Update values and personality slowly, while beliefs and temporary states remain event-driven.

Do not run a new psychological inference process for every citizen every frame. Conversely, do not remove distinctions such as beliefs versus values merely to save a few floats.

**The strict five-trait version has a known blind spot:** Big Five Agreeableness is not a complete substitute for non-exploitation. Preserve that limitation explicitly, or add an H-like narrow disposition. Calling the resulting hybrid “HEXACO” would be inaccurate.

### 5.6 Existing models and games worth borrowing from

| Model or game | Useful pattern | What not to assume |
| --- | --- | --- |
| **HUMAT — Jager et al., 2025** | Integrates motives, cognitive dissonance, individual/social decision strategies, persuasion, and changing social networks. Particularly relevant to adoption, norm conflict, and collective change. [JASSS](https://www.jasss.org/28/1/4.html) | Its architecture is not a universally validated parameterization of human societies. Borrow modular mechanisms and test their contribution. |
| **RimWorld: Ideology** | Beliefs expressed through precepts, social roles, rituals, and reactions to concrete practices. This fits TCE’s authored vocabulary of institutions and customs. [Steam Store](https://store.steampowered.com/app/1392840/RimWorld__Ideology/) | A legible game belief system is not evidence that real ideologies are internally consistent packages, or that all members react identically. |

A useful development sequence is to compare **Big Five only**, **Big Five plus the two main Schwartz contrasts**, **the proposed five-value model**, and **the same model with an honesty-related disposition**.

Test whether each addition improves held-out behavior—not merely whether the expanded model can fit its calibration cases. Particularly important tests are new institutions, unfamiliar resource constraints, and social networks different from those used during tuning.

---

## 6. Sources, datasets, and uncertainty priorities

### 6.1 Most useful data sources

| Source | What to extract for TCE | Important limitation |
| --- | --- | --- |
| **World Values Survey / European Values Study** | Joint distributions of concrete attitudes, institutional confidence, religiosity, economic preferences, and differences between cohorts and regions. Joint trend files support historical comparisons within the survey period. [World Values Survey](https://www.worldvaluessurvey.org/WVSContents.jsp) | Repeated cross-sections do not automatically identify how particular individuals changed. Question comparability matters. |
| **Global Preference Survey; Falk et al., 2018** | Risk preference, patience, positive and negative reciprocity, altruism, and trust. Useful for distributions and covariances that personality traits alone do not supply. [Harvard Business School](https://www.hbs.edu/ris/Publication%20Files/Quarterly%20Journal%20of%20Economics_269d889b-69bb-4412-a7bf-e5dfa7d7bff7.pdf) | Modern survey measures are not universal utility functions, and national averages should not replace individual variation. |
| **SOEP-based longitudinal research** | Connections among personality, life goals, work, and household circumstances. For example, Buchinger et al.’s work examines co-development of life goals and Big Five traits across adulthood and old age. [DNB](https://d-nb.info/134079506X/34?utm_source=chatgpt.com) | A national panel provides temporal depth, not global historical representativeness. |
| **MFQ-2 research — Atari et al., 2023** | Moral distinctions and cross-cultural variation in their associations. The development research covered **8,672 participants in 25 populations**. [NSF Public Access Repository](https://par.nsf.gov/biblio/10531651-morality-beyond-weird-how-nomological-network-morality-varies-across-cultures) | Coverage does not establish universality or historical invariance. |
| **Small-scale society experiments and comparative wealth studies** | How bargaining, sharing, institutions, and resource ownership vary outside modern industrial settings. [Cambridge University Press](https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/economic-man-in-crosscultural-perspective-behavioral-experiments-in-15-smallscale-societies/6EFFD9263D9A5F2FE5DE9DE8FBBA4988) | Often small samples; experimental interpretations and contemporary histories matter. These are not direct measurements of ancient populations. |

For the conceptual foundation, prioritize **Schwartz (2012)** on value structure, **Bleidorn et al. (2022)** on stability and change, **Vukasović and Bratko (2015)** on personality heritability, **Pletzer et al. (2019)** on misconduct, and **Malka, Lelkes, and Soto (2019)** on the organization of ideology. Their most relevant results and citations are incorporated above.

### 6.2 Claims that should remain flagged in the implementation

**Well supported within studied settings:** substantial individual personality differences; partial heritability; substantial adult continuity; imperfect trait–behavior prediction; distinguishable value priorities; context-sensitive political organization.

**Supported but transport-sensitive:** specific risk, entrepreneurship, voting, and misconduct coefficients; measured national differences; life-event associations; war-related changes in cooperation.

**Thin, contested, or unsuitable as fixed constants:** ancient population personality means; universal annual value-change rates; a single trauma response; universal biological moral modules; an invariant left–right personality axis; genetic explanations of cultural differences.

The highest-value investment for TCE is therefore not a larger inventory of trait labels. It is a **small, interpretable disposition model embedded in changing material circumstances, relationships, beliefs, and institutions**. That combination can produce continuity in individual character while allowing the same kinds of people to build very different societies.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927fa-3358-83ea-8835-b60acdd08219)
