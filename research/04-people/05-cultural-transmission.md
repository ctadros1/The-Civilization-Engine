# Cultural evolution and social learning

## A simulation-ready report for The Civilization Engine

**Recommendation:** represent culture as a collection of **domain-specific learned beliefs, norms, preferences, and practices**, transmitted through actual relationships and institutions. Do not make a citizen’s culture a single identifier that determines an entire package of behavior.

For TCE, the most useful distinction is between three processes:

1. **Learning:** acquiring information, preferences, or skills from other people.
2. **Behavior:** deciding what to do, given beliefs, resources, social expectations, and possible sanctions.
3. **Material persistence:** retaining buildings, tools, clothing, texts, and institutions after their creators—or their creators’ preferences—have changed.

These processes need different clocks. An agent can learn about a technique immediately, require years of practice to master it, publicly follow a norm without privately endorsing it, and inhabit a house built according to a much older architectural tradition.

The evidence supports several social-learning mechanisms, but **not a universal conformity coefficient, copying fidelity, or annual cultural mutation rate**. The report therefore distinguishes **empirical measurements**, **results conditional on formal models**, and **proposed TCE parameters**.

---

# 1. Mechanisms: rules the simulation can implement

## 1.1 Represent cultural knowledge separately from cultural behavior

A useful citizen-level representation distinguishes:

| Cultural object | What to store | What not to assume |
| --- | --- | --- |
| **Belief** | Awareness, endorsement or confidence, remembered evidence, trusted sources | Hearing or repeating a statement does not establish belief. |
| **Norm** | Personal endorsement; expectations about what others do and approve; anticipated sanctions | Frequent behavior is not necessarily a moral obligation. |
| **Practice** | Known procedure, prerequisites, competence, practice history | Seeing a successful outcome does not reveal how to reproduce it. |
| **Taste** | Preferences over features, occasions, and social meanings | Preferred objects are not necessarily affordable or available. |
| **Material expression** | The actual recipe, garment, building, or symbol produced | A change in preference does not instantly replace existing objects. |

The distinction between **empirical expectations**—what people normally do—and **normative expectations**—what people believe others expect them to do—is experimentally meaningful. Bicchieri and Xiao’s experiments also show why approval and observed behavior should not be combined into one “norm strength” variable. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/bdm.621)

**TCE rule:** allow a person to know a practice without using it, use it without endorsing it, and endorse it without possessing the resources or skill to perform it.

This makes apparent contradictions intelligible: a dissenter can attend an obligatory ceremony; a poor household can prefer an expensive style; an apprentice can recognize good craftsmanship without being able to reproduce it.

## 1.2 Transmission routes: parents, peers, and other teachers

Cavalli-Sforza and Feldman’s foundational framework distinguishes **vertical transmission** from parents, **horizontal transmission** among peers, and **oblique transmission** from other members of older generations. These routes have different implications for persistence and the speed of change. [PubMed](https://pubmed.ncbi.nlm.nih.gov/7300842/)

A convenient TCE source-access model is:

\[
q\_i(k)=v\_iq\_{\text{parents}}(k)+h\_iq\_{\text{peers}}(k)+o\_iq\_{\text{other elders}}(k),
\qquad v\_i+h\_i+o\_i=1.
\]

Here, \(q\_i(k)\) is the availability of variant \(k\) to learner \(i\), **before** applying prestige, conformity, or other biases. The weights should depend on age, domain, household organization, occupation, and available relationships—not nationality or an immutable personality category.

Parents should not be the only childhood sources. Research among BaYaka hunter-gatherers documents learning through observation, practice, play, and interaction with others; teaching is particularly relevant to some social norms. Contemporary forager evidence is useful for identifying mechanisms, but is not a direct reconstruction of Paleolithic life. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6668464/)

**TCE rule:** generate learning opportunities from childcare, play groups, work, apprenticeship, worship, migration, marriage, and market encounters. Becoming older changes the source mixture; it does not switch learning off.

## 1.3 Conformity: disproportionately copying the common variant

In cultural-evolution theory, **conformist transmission means copying a common variant more often than its frequency alone predicts**. Choosing the majority option 60% of the time when it occurs in 60% of demonstrations is unbiased copying, not conformity. Experimental work emphasizes this distinction. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S1090513807000876)

One convenient frequency-dependent rule is:

\[
P\_C(k)=\frac{f\_k^\alpha}{\sum\_\ell f\_\ell^\alpha},
\]

where \(f\_k\) is the frequency among the learner’s sampled demonstrators.

* \(\alpha=1\): proportional copying.
* \(\alpha>1\): conformity.
* \(0<\alpha<1\): disproportionate attraction to rarer alternatives.

The Boyd–Richerson family of models shows how conformity can reinforce local traditions and preserve between-group differences under specified environmental and transmission conditions. This is a conditional theoretical result, not proof that conformity is always adaptive. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S109051389800018X)

For a binary trait and three independent demonstrators, mixing proportional copying with majority-of-three choice gives another useful formulation:

\[
P(A)=p+D\,p(1-p)(2p-1), \qquad 0\leq D\leq1.
\]

At \(p=0.6\), complete majority-of-three choice gives \(P(A)=0.648\). The power rule with \(\alpha=2\) instead gives approximately \(0.692\). These are **different parameterizations**, not interchangeable coefficients.

**TCE rules:**

Use the frequency among people the learner actually observes—not the true settlement-wide frequency. Count distinct demonstrators where possible: hearing one influential person’s claim repeated by ten intermediaries should not necessarily provide ten independent pieces of evidence.

Apply conformity at learning or reconsideration events. Reapplying it every simulation tick would create enormous, unintended selection pressure.

## 1.4 Prestige and payoff biases: different reasons to copy someone

**Prestige bias** uses attention, voluntary deference, or reputation as a cue to whom one should learn from. **Payoff bias** uses evidence that a behavior or practitioner succeeds.

They should be separate. Preschool experiments found that a brief cue showing others attending to a model increased learning from that model, with stronger effects when prestige and learning concerned the same domain. Prestige did not simply spill over indiscriminately between artifact use and food preferences. [Coevolution Lab](https://coevolution.fas.harvard.edu/publications/prestige-biased-cultural-learning-bystanders-differential)

Field research in Fijian villages also supports domain-specific learning networks. Advice about fishing, cultivation, and medicine was not drawn from one universal hierarchy; local access and domain-relevant cues mattered. Feedback schedules also differed: fishing outcomes can be observed much more frequently than annual crop outcomes. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3049092/)

**TCE rules:**

Give people reputations by domain: competent builder, successful cultivator, persuasive ritual specialist, fashionable patron. Wealth, coercive power, popularity, and expertise may correlate, but should not be identical variables.

For payoff learning, use **observed and noisy outcomes**. A farmer should compare yields under approximately comparable soil and weather conditions, imperfectly. Agents should not consult the engine’s true production function.

Allow prestige to substitute for difficult outcome evaluation. Conversely, repeated visible failure should sometimes erode prestige.

## 1.5 Content bias and transmission fidelity

Some information is more likely to attract attention, be remembered, or be passed onward. But **transmission, endorsement, and behavior change are distinct outcomes**.

For example, an observational study of 563,312 political tweets found approximately 20% greater diffusion per additional moral-emotional word. That is a result about message sharing in a particular communication environment, not a general probability that listeners adopt a belief. [DOI](https://doi.org/10.1073/pnas.1618923114)

Copying also need not produce unbiased random errors. In an experiment involving 21 expert potters from two Indian communities and France, reproductions of unfamiliar shapes retained individual and community-specific characteristics. Learned production habits systematically influenced reconstruction. [DOI](https://doi.org/10.1093/pnasnexus%2Fpgae055)

**TCE rules:**

Separate:

* Probability of noticing an item.
* Probability of retaining or retelling it.
* Probability of endorsing or attempting it.
* Accuracy of reconstruction or performance.

For practices, use a transition kernel \(M\_{k\ell}\): the probability that intending to learn variant \(k\) produces variant \(\ell\). Let errors depend on competence, instruction, available exemplars, and prior habits. Archaeological transmission models explicitly distinguish copying processes that amplify or reduce material variation. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416505000437)

A useful mathematical warning: if a 20-component procedure has independent per-component fidelity of 0.99, exact whole-procedure fidelity is \(0.99^{20}\approx0.818\). That is an illustration, **not an empirical estimate**. Redundancy, correction, repeated teaching, and external records can invalidate the independence assumption—and are precisely the mechanisms TCE should support.

## 1.6 Norms persist through expectations and institutions, not copying alone

A norm can influence behavior through several channels: personal commitment, expected approval, coordination benefits, exclusion, or punishment.

**TCE rule:** calculate public behavior from these components separately. Expected punishment must depend on actual observers, enforcement capacity, and willingness to sanction. Enforcement should cost someone time, resources, or social capital.

This permits public conformity alongside private disagreement. It also permits rapid behavioral change when enforcement collapses, without pretending that everyone’s private beliefs changed overnight.

Institutions should act as **organized transmission and enforcement systems**: households, councils, temples, guilds, schools, courts, and workshops can maintain canonical practices, train newcomers, certify competence, and sanction deviations. Their persistence should require people and resources rather than an immortal cultural template. The separation of personal attitudes, expectations, and observed behavior is consistent with experimental work on social norms. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/bdm.621)

## 1.7 Boundaries emerge from selective contact and selective borrowing

Axelrod’s model gives a useful baseline. Agents have multiple cultural features, interact more often when they share features, and copy one differing feature during an interaction. Local convergence can coexist with persistent regional differences. However, its zero-overlap rule can also freeze boundaries artificially. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002797041002001)

Migration models likewise show that migration does not mechanically erase cultural differences. Outcomes depend on acculturation, conformity, assortment, and the transmission processes operating within receiving communities. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0205573)

**TCE rules:**

Distinguish **people moving** from **information crossing a boundary**. A traded tool can spread without migration; a migrant can retain some practices while adopting others.

Make permeability domain-specific. Construction techniques, foods, ritual obligations, language, and marriage conventions need not cross the same boundary at the same rate.

Allow overlapping memberships and broker relationships: a merchant can participate in a foreign commercial network while retaining local ritual commitments. Hybrid forms should arise by recombining compatible modules, not only by averaging two complete “cultures.”

Do not hard-code all cross-group contact as hostile—or all contact as assimilative. Use actual histories of exchange, conflict, cooperation, and exclusion.

## 1.8 Drift, innovation, and the maintenance of complex skills

**Neutral drift** is random change in variant frequencies through finite sampling. It is not a synonym for every cultural change.

In a simple haploid Wright–Fisher copying model without selection or innovation:

\[
\operatorname{Var}(\Delta p\mid p)=\frac{p(1-p)}{N\_e}
\]

per complete model generation. This is a mathematical benchmark, not a claim that all citizens replace their culture once per biological generation.

Henrich’s 2004 model illustrates a different mechanism: imperfect learning can cause complex skills to deteriorate unless a sufficiently large interacting population supplies exceptional models. Under its particular assumptions:

\[
\Delta\bar z\approx-a+b(\gamma+\ln N),
\]

where \(\gamma\approx0.577\), \(a\) represents copying difficulty, \(b\) describes variation in learning outcomes, and \(N\) is the model’s interacting learner population. These are **model parameters in skill units**, not portable historical estimates. [Joe Henrich](https://henrich.fas.harvard.edu/sites/g/files/omnuum5811/files/henrich/files/henrich_2004-2.pdf)

The broader claim that population size explains historical technological complexity is contested, including the interpretation of the Tasmanian case. Ecology, specialization, connections, and model assumptions matter. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4843435/)

**TCE recommendation:** explicitly model access to skilled teachers, opportunities to practice, specialization, and stored knowledge. Do not implement “population falls below X, therefore lose technology Y.”

---

# 2. Parameters and measured rates

## 2.1 Empirical measurements that can constrain the model

These observations constrain particular mechanisms. They are **not universal constants to insert unchanged into an agrarian simulation**.

| Quantity | Measured value | Units and setting | Confidence and appropriate use |
| --- | --- | --- | --- |
| Prestige cue affecting model choice | More than **2×** odds in one experiment; almost **5×** in a domain-matched condition | Odds ratio; learning by 3–4-year-old children after a brief attention cue | **Moderate**, experimental but narrow. Supports domain-specific prestige, not a universal status coefficient. [Coevolution Lab](https://coevolution.fas.harvard.edu/publications/prestige-biased-cultural-learning-bystanders-differential) |
| Cross-sample difference in social learning | Mainland China versus UK: **OR 2.16**, 95% CI **1.38–3.42**, in one season; **2.25**, CI **1.38–3.73**, in another | Odds of copying in a virtual artifact-design task; 292 participants across four samples | **Moderate**, task-specific. Hong Kong and Chinese immigrants in the UK did not show the same mainland difference. This was not a direct test of strict conformity. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4262178/) |
| Local access affecting teacher choice | Same-village effects approximately **OR 1.8–2.76**, varying by domain/model | Advice nominations concerning fishing, cultivation, and medicine in Fiji | **Moderate association**. Nominated advisers are not observed transmission events; significance varies by domain. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3049092/) |
| Uptake of visibly useful components | Approximately **70%** adopted within the first **1–2 trials** | Adoption opportunities in a controlled cumulative-culture task | **Moderate**, experimentally measured; low transferability to opaque, expensive, or dangerous practices. Trials are not days. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4801235/) |
| Moral-emotional content and diffusion | Approximately **1.20×** expected retweet count per additional moral-emotional word | Count-rate ratio in an observational social-media dataset | **Moderate association; weak causal interpretation**. Calibrate retelling, not belief conversion. [DOI](https://doi.org/10.1073/pnas.1618923114) |
| General conformity strength, copying fidelity, or innovation rate | **No transferable universal estimate identified** | These depend on task, cultural domain, observation model, and time unit | Treat as uncertain parameters requiring domain-specific calibration, rather than filling the gap with a single historical constant. |

An instructive archaeological fit produced a frequency-bias parameter \(b=0.028\), with 95% highest-posterior-density interval \([-0.005,0.102]\), under stationarity, versus \(0.066\), interval \([0.015,0.134]\), without it. **Both constant-bias models fit poorly.** Positive \(b\) represented novelty bias in that paper, unlike the conformity exponent above. A fitted number is not automatically a validated mechanism. [Nature](https://www.nature.com/articles/srep39122)

## 2.2 Measured material-culture change: what the evidence actually measures

Pottery frequencies, garment dimensions, house forms, and private preferences are different observables. Their rates cannot be converted into each other without an explicit production and observation model.

| Evidence | Quantitative finding | What it can—and cannot—calibrate |
| --- | --- | --- |
| **Neolithic pottery, Merzbach Valley, Germany** | **5,804 vessels**, **36 decorative band types**, eight phases spanning approximately **180 years**. About **80%** of vessels in the final phase carried types already present in the first studied phase. | Persistence of a motif repertoire alongside frequency change. **Not** 80% copying fidelity or an individual adoption rate. [Nature](https://www.nature.com/articles/srep39122) |
| **Women’s daywear in Vogue, 1950–2013** | **2,102 images**. Reported hemline cycles included **21 years** for 1950–1971, **6 years** for 1971–1977, and **16 years** for 1977–1993. | Descriptive changes in editorial fashion imagery. Not a universal fashion clock or population wardrobe survey. The analysis used annual means and smoothing. [KoreaScholar](https://db.koreascholar.com/Article/Detail/280374) |
| **Taiwan and US dress imagery, 1966–1986** | Six dress dimensions were compared across **21 years**. Skirt-length cycles appeared in both samples; skirt-width cycles appeared only in the US sample. Other dimensions lacked apparent cycles. | Supports component-specific, regionally variable dynamics rather than one clothing-style variable. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0887302X9201000408) |
| **Domestic architecture at Çatalhöyük** | Successive house construction and repeated domestic practices provide evidence of architectural and social memory; the study does not supply a transferable annual style-transition probability. | Rebuilding can reproduce an inherited organization rather than introduce a new style. [Cambridge University Press](https://www.cambridge.org/core/journals/american-antiquity/article/abs/daily-practice-and-social-memory-at-catalhoyuk/4178335E6AEA96DE576304DF1D8E9D2E) |
| **Early twentieth-century Hanoi house designs** | Study based on **68 houses**, **248 photographs**, and **78 coded observations**, analyzing combined French and Chinese/Vietnamese-associated design influences. | Evidence relevant to hybrid architectural features. Cross-sectional observations do **not** estimate annual diffusion rates; coded rows are not independent houses. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S2590291119300014) |
| **Comparative material-culture change series** | Perreault’s fitted relationship predicts characteristic-value ratios of about **1.022 over one year** and **1.417 over 1,000 years**. | Illustrates strong dependence on observation interval. These pooled predictions do not justify compounding “2.2% cultural change per year.” [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0045150) |

### Architecture needs a stock-and-replacement model

The reviewed architectural evidence does **not** establish a broadly transferable annual “style drift” parameter. For TCE, separate:

* Preferences of patrons and craftspeople.
* Styles selected for new construction and renovation.
* Styles represented in surviving buildings.

For constant total building stock and a style-independent replacement hazard \(r\):

\[
\frac{dS\_k}{dt}=r\left(P^{\text{new}}\_k-S\_k\right),
\]

where \(S\_k\) is the fraction of existing stock in style \(k\), and \(P^{\text{new}}\_k\) is its share of replacements.

As a **purely illustrative engineering calculation**, \(r=0.02\) per year gives a stock-adjustment half-life of about **34.7 years**. That is not a historical estimate. In TCE, derive replacement from actual decay, demolition, disaster, rebuilding, and growth events. Façades and furnishings can change independently of structural layouts.

### Use explicitly defined change metrics

Two useful outputs are:

\[
r\_{\mathrm{TV}}=
\frac{1}{2\Delta t}
\sum\_k|p\_k(t+\Delta t)-p\_k(t)|
\]

for net change in a frequency distribution per year, and

\[
r\_{\log}=
\frac{|\ln \bar x(t+\Delta t)-\ln \bar x(t)|}{\Delta t}
\]

for relative change in a continuous characteristic.

Neither is an individual adoption hazard. Reversals within an interval disappear from these net measures. Compare simulations and observations at matching temporal resolutions; cultural-rate comparisons are sensitive to interval length. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0045150)

## 2.3 Proposed TCE starting parameters

**Every number below is a design prior, not an empirical estimate.** Their source is the proposed implementation, and their external confidence is **low until calibrated**. They provide a manageable initial search space.

| Parameter | Initial value | Sensitivity range | Unit and interpretation |
| --- | --- | --- | --- |
| Demonstrators sampled, \(n\) | 5 | 3–9 | Distinct eligible models per learning event |
| Reliance on social information, \(s\) | 0.6 | 0.2–0.9 | Mixture weight during reconsideration |
| Frequency exponent, \(\alpha\) | 1.3 | 0.7–2.0 | Dimensionless; below 1 permits rarity preference |
| Prestige sensitivity, \(\beta\_P\) | 0.5 | 0–1.5 | Log-weight per standard deviation of domain prestige |
| Payoff sensitivity, \(\beta\_\pi\) | 1.0 | 0–3.0 | Log-weight per standard deviation of perceived success |
| Module-level transmission fidelity, \(q\) | 0.99 | 0.90–0.999 | Probability per attempted transmission of a defined module |
| Deliberate innovation attempt rate, \(\mu\) | 0.001 | 0.0001–0.01 | Probability per eligible modification/design opportunity |
| Baseline adult taste reconsideration, \(\lambda\_T\) | 0.5 | 0.1–2.0 | Events per domain per year |
| Baseline committed-norm reconsideration, \(\lambda\_N\) | 0.05 | 0.01–0.2 | Events per norm per year; not conversion probability |

Important qualifications:

**Reconsideration is not conversion.** A review can reinforce the incumbent variant. Childhood acquisition, instruction, migration, observed failure, and major institutional changes should generate additional events rather than wait for baseline review clocks.

**Fidelity and innovation are different.** A badly made pot is not automatically a new cultural tradition. Errors can remain individual deviations; innovations become transmitted variants only when noticed, retained, and reproduced.

**Do not transfer units between studies.** A log odds ratio for a binary attention cue is not a coefficient per standard deviation of prestige. A retweet count ratio is not an adoption odds ratio.

---

# 3. Variation across eras and world regions

TCE should vary transmission through **social organization, communication infrastructure, production systems, and access to teachers**, rather than scripted era-specific psychology.

| Context | Evidence and interpretation | TCE implementation |
| --- | --- | --- |
| **Forager societies** | BaYaka evidence shows observation, practice, play, and teaching contributing differently across domains. Peers and older individuals matter alongside parents. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6668464/) | Mixed-age activity groups; learning embedded in everyday tasks; mobility changes accessible teachers. Do not equate small populations with isolated households. |
| **Early farming settlements** | Neolithic European pottery shows persistent traditions but also changes inconsistent with simple neutral copying. Anatolian house histories show repeated construction contributing to social memory. [Cambridge University Press](https://www.cambridge.org/core/journals/american-antiquity/article/ceramic-style-change-and-neutral-evolution-a-case-study-from-neolithic-europe/99C26B88AFB5FF76A2630FA5DB53AF09) | Household production, neighborhood exposure, apprenticeship, and durable buildings preserve variants. Permit novelty without requiring an industrial-style communication network. |
| **Village and apprenticeship-based production** | Fijian learning networks and Indian pottery experiments offer measured mechanisms for local access, specialized expertise, and entrenched production habits. These are contemporary studies—not direct pre-industrial time series. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3049092/) | Make craft lineages and teacher availability consequential. Market exchange can spread products faster than the embodied skills needed to manufacture them. |
| **Industrializing and colonial urban settings** | Hanoi’s architectural evidence illustrates combinations of design influences rather than complete replacement by one homogeneous style. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S2590291119300014) | Add patrons, contractors, standardized components, professional training, and communication links as they become available. Allow selective combinations of structure, ornament, materials, and symbolism. |
| **Modern high-connectivity settings** | The China–Hong Kong–UK experiment found substantial variation even among connected contemporary populations. Moral-emotional message diffusion was stronger within ideological networks, illustrating that broad communication need not erase boundaries. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4262178/) | Increase reach and potential source concentration, but retain trust, language, identity, and domain-specific credibility. High exposure need not imply consensus. |

The Chinese immigrant comparison is particularly important to interpret cautiously: it is **cross-sectional**, so it does not establish that migration itself caused a measured individual change in learning strategy. Nor does the study justify assigning an innate “Chinese conformity” parameter. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4262178/)

For regional calibration, use variables such as residence patterns, occupational specialization, language access, settlement connectivity, schooling, and institutional participation. D-PLACE provides comparative cultural, linguistic, and environmental information for more than 1,400 societies in its foundational publication, but is not an annual cultural-change dataset. [The Australian National University](https://researchportalplus.anu.edu.au/en/publications/d-place-a-global-database-of-cultural-linguistic-and-environmenta/)

---

# 4. Stylized facts and validation targets

A correct simulation should reproduce several patterns **without forcing every world toward the same historical trajectory**.

| Pattern | Quantitative or empirical anchor | Validation target for TCE |
| --- | --- | --- |
| **Prestige has limits across domains** | Preschool learning effects depended on whether prestige cues matched the learning domain. [Coevolution Lab](https://coevolution.fas.harvard.edu/publications/prestige-biased-cultural-learning-bystanders-differential) | A respected builder should not automatically become the preferred source for medicine, food preferences, and political beliefs. |
| **Useful innovations can spread quickly when evaluation and reproduction are easy** | Approximately 70% uptake within the first one or two opportunities in a controlled task. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4801235/) | Low-cost, visible improvements can diffuse rapidly; opaque techniques should respond differently. |
| **Maximum connectivity is not always best for cumulative innovation** | In one experiment, the most complex innovation appeared in **58.3% of partially connected groups** and **0% of fully connected groups**. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4801235/) | Some network structures should preserve alternative approaches long enough for recombination. Do not turn the experimental percentages into universal settlement bonuses. |
| **Tradition and change coexist** | Long-lived pottery repertoires can coexist with changing type frequencies; neutral copying alone did not explain the examined ceramic sequence. [Cambridge University Press](https://www.cambridge.org/core/journals/american-antiquity/article/ceramic-style-change-and-neutral-evolution-a-case-study-from-neolithic-europe/99C26B88AFB5FF76A2630FA5DB53AF09) | New variants should not require erasing all old motifs. A settlement can remain recognizable while changing substantially. |
| **Fashion need not have one cycle or one dominant style** | Hemline periods varied; Taiwan and US dress dimensions did not display identical patterns. [KoreaScholar](https://db.koreascholar.com/Article/Detail/280374) | Permit overlapping styles, component-specific change, reversals, and increased diversity—not a fixed “new fashion every N years” clock. |
| **Public action and endorsement respond differently** | Experiments distinguish effects of observed behavior from normative expectations. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/bdm.621) | Enforcement changes can alter public behavior without instantly rewriting private beliefs. |
| **Local convergence can coexist with persistent boundaries** | Formal cultural-diffusion and migration models produce this combination under specified mechanisms. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002797041002001) | Connected settlements should sometimes share tools or architectural components while retaining distinct rituals, identities, or obligations. |

Also include **mathematical null tests**, distinct from empirical validation. Under neutral Wright–Fisher copying at \(p=0.5\), the one-generation standard deviation is approximately **0.112 for \(N\_e=20\)** and **0.0112 for \(N\_e=2{,}000\)**. Failure to reproduce such a benchmark indicates an implementation problem before any historical calibration begins.

---

# 5. Modeling recommendation for 10k–50k agents

## 5.1 Use a small shared representation, not a giant culture vector

I recommend five interacting object types.

**Variant registry.** Store authored cultural modules and their generated combinations: prerequisites, production requirements, content cues, parent variants, and compatibility rules. A building style can combine a plan, roof form, ornament set, color convention, and symbolic elements rather than belong to one indivisible style package.

**Citizen cultural state.** Store a limited active repertoire: current variants, a few alternatives, confidence, competence where relevant, and remembered sources. Allocate richer expectation state only to consequential norms.

**Existing relationship graph.** Reuse kinship, neighborhood, workplace, apprenticeship, friendship, and institutional relationships. Cultural transmission should draw on the same daily life that carries help, gossip, conflict, and disease.

**Institutions.** Store teaching capacity, canonical practices or records, recognized experts, enforcement resources, membership requirements, and patrons. Their effects must pass through actual agents and material resources.

**Artifacts.** Preserve the features chosen when an object was made. A house changes only through a construction or renovation event, not because the owner’s cultural affiliation changed.

Culture names and aggregate profiles can be computed for presentation and analysis. They should summarize agents, not overwrite them.

## 5.2 A combined learning kernel

The following is a **proposed TCE synthesis**, not an equation claimed to come from one paper.

At a learning event for agent \(i\) in domain \(d\), sample eligible demonstrators. Construct four distributions:

\[
q\_U(k)=f\_k,
\qquad
q\_C(k)=\frac{f\_k^\alpha}{\sum\_\ell f\_\ell^\alpha},
\]\[
q\_P(k)=
\frac{\sum\_{j:x\_j=k}\exp(\beta\_P P\_{jd})}
{\sum\_j\exp(\beta\_P P\_{jd})},
\]

with an analogous \(q\_\pi(k)\) based on **perceived**, domain-appropriate success.

Mix them:

\[
q\_{\text{social}}(k)=
w\_Uq\_U(k)+w\_Cq\_C(k)+w\_Pq\_P(k)+w\_\pi q\_\pi(k),
\qquad \sum w=1.
\]

Then combine social information with individual experience:

\[
q\_{\text{candidate}}(k)
=(1-s\_i)q\_{\text{individual}}(k)+s\_iq\_{\text{social}}(k).
\]

Content can affect attention, retention, or the weighting of candidates. Reconstruction then passes through the appropriate transmission-error process. Finally, **behavior is chosen separately**, considering ability, resources, commitments, and social consequences.

Practical safeguards:

Keep the incumbent option available. Use bounded, observed success rather than the simulation’s true payoff. Standardize prestige and payoff within relevant comparison sets. Fall back to individual experience when no eligible teacher exists.

Avoid applying the same mechanism twice unintentionally—for example, counting prestige-driven exposure as strong majority evidence and then applying an additional prestige bonus without recognizing the duplication.

## 5.3 Make cultural updates event-driven

For a hazard \(\lambda\_d\), use:

\[
P(\text{review during }\Delta t)=1-e^{-\lambda\_d\Delta t}.
\]

This makes the interpretation stable when timestep length changes.

Generate additional learning events from apprenticeships, new jobs, instruction, migration, festivals, disputes, failed harvests, successful demonstrations, and construction orders. Visible daily activities can accumulate exposure without triggering a full belief update on every encounter.

A workload illustration: one review per citizen per simulated month at 50,000 citizens is approximately **1,667 reviews per day**. Five demonstrators per review implies about **8,333 demonstrator inspections per day**. This is an operation count, **not a performance benchmark**.

Likewise, 24 minimal cultural slots of 16 bytes each across 50,000 people require **19.2 MB** before relationship storage, richer beliefs, institutions, registries, allocator overhead, or histories. Full probability distributions over every variant are unnecessary; construct local choice distributions when events occur.

Use snapshot/read and commit phases for parallel updates so agents do not selectively observe half-updated cultural states.

## 5.4 Preserve knowledge without granting automatic progress

Innovation should require an opportunity, relevant knowledge, materials, time, and a viable candidate assembled from authored components. Some attempts should fail; some should improve only one objective while worsening another.

Separate:

**Knowing that a technique exists**, **knowing its procedure**, **being competent to perform it**, and **having the infrastructure to use it**.

This allows a community to retain an account of a lost craft while lacking a competent practitioner. Conversely, migration by one specialist can restore practical capacity without “unlocking an era.”

Teacher access and network structure should affect accumulation, but not through a single automatic population multiplier. Experimental evidence that partial connectivity can outperform full connectivity makes such a multiplier especially inappropriate. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4801235/)

## 5.5 Calibrate mechanisms and observation processes together

Do not fit conformity, innovation, fidelity, migration, and memory duration simultaneously to one frequency curve and assume the answer is identified.

First test micro-level behavior: teacher selection, responses to success, majority information, teaching, and sanctions. Then test population outputs: within-settlement diversity, between-settlement differences, cultural persistence, skill loss, hybridization, and cohort change.

For archaeological comparison, generate **simulated assemblages**: account for production volume, breakage, disposal, preservation, sampling, and time averaging. The frequency of excavated pots is not necessarily the frequency of living people’s preferences.

Use held-out periods or sites, multiple random seeds, and sensitivity analysis. Recent work explicitly develops computational workflows connecting cultural-evolution models to data rather than stopping at plausible-looking emergent patterns. [DOI](https://doi.org/10.1073%2Fpnas.2322887121)

## 5.6 Existing models worth reusing

| Model or implementation | What to reuse | What to change for TCE |
| --- | --- | --- |
| **Axelrod’s cultural dissemination model** | A transparent baseline for local convergence and persistent differentiation | Replace the fixed neighborhood and artificially absorbing zero-overlap boundaries with real social and geographic opportunities. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002797041002001) |
| **Mesoudi’s migration and acculturation models** | Experiments linking migration, conformity, assortment, and maintenance of group differences | Use individual migration, multiple domains, actual institutions, and event-based learning. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0205573) |
| **Mesoudi’s “Simulation models of cultural evolution in R”** | Small, inspectable implementations covering copying, mutation, transmission biases, migration, networks, and other mechanisms | Port mechanisms and regression tests to Rust; there is no need to embed R in the simulation. The repository includes a 2025 release. [GitHub](https://github.com/amesoudi/cultural_evolution_ABM_tutorial) |
| **Henrich’s skill-maintenance model** | A stress test for imperfect transmission and access to exceptional models | Treat its assumptions as an experimental scenario, not a universal law of technological history. [Cambridge University Press](https://www.cambridge.org/core/journals/american-antiquity/article/abs/demography-and-cultural-evolution-how-adaptive-cultural-processes-can-produce-maladaptive-lossesthe-tasmanian-case/8CD08CC61E6FACF59EC02659B2BDA43C) |

---

# 6. Sources, datasets, and evidence limits

## Most useful replication and comparative resources

| Resource | What it contributes |
| --- | --- |
| **Crema, Kandler & Shennan, 2016**, *Revealing patterns of cultural transmission from frequency data* | Pottery frequencies and model code: **Zenodo DOI 10.5281/zenodo.187558**. [Nature](https://www.nature.com/articles/srep39122) |
| **Mesoudi et al., 2015**, cross-cultural social-learning experiment | Trial-level experimental data: **Dryad DOI 10.5061/dryad.f5q4s**. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4262178/) |
| **Kirby et al., 2016**, D-PLACE | Comparative cultural, linguistic, geographic, and environmental context; useful for scenario construction and comparative validation. [The Australian National University](https://researchportalplus.anu.edu.au/en/publications/d-place-a-global-database-of-cultural-linguistic-and-environmenta/) |
| **Perreault, 2012**, *The Pace of Cultural Evolution* | Comparative change-rate analysis and supplementary series; particularly useful for understanding temporal-resolution effects. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0045150) |
| **Mesoudi’s cultural-evolution ABM tutorial** | Reusable model implementations and explanations; archived under **Zenodo DOI 10.5281/zenodo.5155821**. [GitHub](https://github.com/amesoudi/cultural_evolution_ABM_tutorial) |

The main theoretical foundations are Cavalli-Sforza and Feldman’s *Cultural Transmission and Evolution*; Boyd and Richerson’s transmission framework and Henrich and Boyd’s conformity models; Axelrod’s cultural dissemination model; and Henrich’s imperfect-learning model of skill maintenance. [PubMed](https://pubmed.ncbi.nlm.nih.gov/7300842/)

## Where evidence is thin or contested

**Portable coefficients are scarce.** Laboratory odds ratios, advice nominations, archaeological frequencies, and message-sharing counts measure different outcomes. None should silently become a general “chance to copy someone.”

**Material resemblance does not identify a unique transmission process.** Similar objects can arise through copying, shared constraints, inherited motor habits, centralized production, or reconstruction toward familiar forms. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416505000437)

**Population size is not a sufficient explanation of cultural complexity.** Teacher access, ecology, specialization, transmission assumptions, and connectivity must remain explicit. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4843435/)

**Architecture is the weakest area for a transferable annual rate.** The reviewed studies support persistence and hybridization, but do not justify a universal annual architectural-style mutation probability. [Cambridge University Press](https://www.cambridge.org/core/journals/american-antiquity/article/abs/daily-practice-and-social-memory-at-catalhoyuk/4178335E6AEA96DE576304DF1D8E9D2E)

**The central design principle for TCE is therefore to simulate the opportunities and consequences of learning, not cultural replacement itself.** Let people observe, choose teachers, practice, enforce expectations, preserve records, migrate, and build. Persistent traditions, fashions, hybrid styles, cultural boundaries, and occasional knowledge loss should emerge from those processes rather than from a hidden culture-conversion clock.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927ff-3c54-83ea-ad2e-12c87acd7cff)
