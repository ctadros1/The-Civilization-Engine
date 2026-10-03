# Ideology formation and opinion dynamics

## A simulation-ready report for The Civilization Engine

**Recommendation:** Model ideology as a combination of **policy preferences, causal beliefs, values, group attachments, and institutional commitments**—not as one left–right number. Use confidence-sensitive social influence for ordinary opinion change, material experience to change what people consider important, institutions to preserve and package ideas, and a separate decision process for public allegiance and political action.

The research offers useful mechanisms and several experimentally measured effects, but **not a universally validated set of political-learning coefficients**. Many opinion-dynamics models reproduce consensus or polarization without establishing that their individual update rules describe real political behavior. Evidence is strongest for contemporary experiments and surveys; transferring numerical coefficients to ancient or small-scale societies requires substantial uncertainty. [JASSS](https://www.jasss.org/20/4/2.html)

The distinction throughout this report is between **empirical benchmarks**, **mathematical model properties**, and **proposed TCE design parameters**.

---

## 1. Mechanisms: implementable causal rules

### 1.1 Separate the components of ideology

For TCE, an individual’s political state should contain several distinguishable components:

| Component | Example | Function in the simulation |
| --- | --- | --- |
| **Material interests** | A tenant expects a land reform to reduce rent. | Generates perceived gains and losses from proposed institutions. |
| **Causal beliefs** | “The shortage was caused by hoarding, not a poor harvest.” | Determines which policies or authorities receive credit and blame. |
| **Values** | Security, autonomy, reciprocity, status, obligations beyond kin. | Evaluates outcomes that are not reducible to personal consumption. |
| **Issue preferences** | Support for hereditary office, communal storage, or equal legal standing. | Supplies positions on concrete proposals. |
| **Group attachments** | Identification with a lineage, settlement, occupation, religion, or faction. | Shapes trust, loyalty, and perceived collective interest. |
| **Public expression and action** | Privately opposing a ruler while publicly complying. | Determines observable support, participation, and mobilization. |

These are proposed state variables, not claims that all societies recognize the same categories. Cross-cultural values research provides useful starting dimensions, but its categories should not become a timeless, exhaustive taxonomy of human motivation. Schwartz’s framework explicitly distinguishes values from attitudes, beliefs, and norms. [ScholarWorks](https://scholarworks.gvsu.edu/orpc/vol2/iss1/11/)

**Rule:** Allow inconsistent and incomplete combinations. A person may favor communal grain reserves, hereditary leadership, religious pluralism, and unequal inheritance simultaneously. Do not force every issue position to agree with a global ideology score.

Also distinguish **no formed opinion** from **a moderate opinion**. Survey instability is not necessarily genuine ideological volatility: research using multiple indicators finds substantially more preference stability than noisy single-question measurements suggest. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/strength-of-issues-using-multiple-measures-to-gauge-preference-stability-ideological-constraint-and-issue-voting/16F6AF97F7B71AA0112EC9ADF78B553A)

### 1.2 Material interests change incentives—but interpretation mediates the result

**Rule:** When a household’s economic or legal position changes, recompute its perceived consequences of relevant policies. Relevant inputs include property, occupation, debt, taxation, access to commons, dependence on patrons, exposure to violence, and expectations for children.

Use **perceived counterfactual welfare**, not omniscient optimization. An agent can misunderstand who benefits, misattribute a crisis, or prioritize a group’s position over immediate personal consumption.

A useful empirical example is the allocation of land titles to squatters near Buenos Aires. Di Tella, Galiani, and Schargrodsky found that receiving titles changed beliefs in a more market-compatible direction. Institutions can therefore change political beliefs, rather than merely reflecting preferences that existed beforehand. This does not establish a universal “property ownership causes conservatism” coefficient. [OUP Academic](https://academic.oup.com/qje/article/122/1/209/1924727)

**TCE implementation:** Express material consequences in comparable units, such as the expected change in household consumption, security, or control over productive assets. Map those consequences into policy evaluations through each agent’s beliefs.

A harvest failure should first increase food-related salience and dissatisfaction. Whether it produces demands for redistribution, stronger executive authority, religious renewal, migration, or no political response depends on available explanations and organizations.

### 1.3 Identity affects both whom people believe and what they want

**Rule:** The same proposal can receive different evaluations depending on its perceived sponsor. Group attachment should influence source trust, anticipated treatment under a policy, and the reputational cost of accepting an opposing group’s position.

Cohen’s experiments demonstrated that party endorsements could dominate policy content in participants’ evaluations. Their scope was contemporary partisan judgment, not all political decisions everywhere, but they establish why material-interest calculations alone are insufficient. [PubMed](https://pubmed.ncbi.nlm.nih.gov/14599246/)

For TCE, use **overlapping identities**. A citizen can share a workplace with one group, a lineage with another, and ritual obligations with a third. Do not collapse these into a single permanent faction.

Keep identity attachment separate from ideological agreement. This permits:

* Followers retaining allegiance while disagreeing with leaders on particular policies.
* Coalitions spanning different doctrines.
* A faction splitting over succession, exclusion, or patronage despite little initial policy disagreement.

These are architectural possibilities; their frequencies should emerge from local institutional conditions rather than fixed global probabilities.

### 1.4 Social influence is selective, heterogeneous, and often weak

A practical update for an agent’s support \(x\_{ik}\in[0,1]\) for proposition \(k\) is:

\[
x'\_{ik}=(1-a\_{ik}-b\_{ik})x\_{ik}+a\_{ik}m\_k+b\_{ik}z\_{ik},
\]

where \(m\_k\) is an encountered message, \(z\_{ik}\) is the agent’s current experience-and-values-based evaluation, and \(a\_{ik},b\_{ik}\geq0\), with \(a\_{ik}+b\_{ik}\leq1\).

Make the social weight conditional:

\[
a\_{ik}
=\eta\_i\,
s\_{ik}\,
T\_{ij,k}\,
G(|m\_k-x\_{ik}|,c\_{ik})\,
N\_{im}.
\]

Here, \(\eta\_i\) is susceptibility, \(s\_{ik}\) is salience, \(T\_{ij,k}\) is topic-specific trust, \(G\) is a confidence/disagreement gate, and \(N\_{im}\) discounts redundant information.

**Rules:**

1. Exposure does not guarantee attention; attention does not guarantee acceptance.
2. Strongly held positions usually require more convincing evidence or experience.
3. Agreement can increase confidence without changing position.
4. Trust is topic-specific: a successful farmer need not be an authority on succession law.
5. Repeated copies of the same argument should not count as independent evidence.

Moussaïd and colleagues’ experiments support heterogeneous responses and the importance of relative confidence and disagreement. However, they studied numerical factual estimates, and explicitly identified transfer to emotional or political questions as a further research challenge. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0078433)

For a first implementation, use a **smooth gate**, not a hard rule that people beyond a particular ideological distance can never influence one another.

### 1.5 Networks determine exposure; institutions determine persistence

**Rule:** Draw influence opportunities from actual social life: households, workplaces, neighborhood meetings, rituals, markets, military service, migration, and communication institutions.

Keep two processes separate:

**Selection:** people preferentially form or maintain some relationships.

**Influence:** interaction changes opinions within existing relationships.

A network of like-minded people does not, by itself, reveal which process created the similarity. The distinction is central to assessing social-influence models. [JASSS](https://www.jasss.org/20/4/2.html)

Institutions should hold durable doctrine records, teaching practices, offices, resources, and membership rules. A doctrine then survives through replacement of its human carriers rather than through immortal founders.

**TCE rule:** Institutional capacity changes transmission opportunities. A school, priesthood, guild, court, or newspaper does not directly add a fixed percentage to ideological support. It creates repeated exposure, trusted relationships, coordination opportunities, and incentives for public conformity.

### 1.6 Polarization is several different phenomena

Measure these separately:

| Dimension | Suitable diagnostic |
| --- | --- |
| Policy disagreement | Distance between group mean positions on specific issues. |
| Extremity | Distance from a substantively defined reference position—not automatically the population mean. |
| Bimodality | Whether the distribution forms two separated concentrations. |
| Issue alignment | Correlation between positions on otherwise distinct issues. |
| Group hostility | Negative affect or discriminatory expectations toward another group. |
| Network segregation | Assortativity and frequency of cross-group interactions. |

Research on American public opinion illustrates why sorting into parties and increasing consistency across issues must be distinguished from everyone becoming more extreme. The historical pattern itself can change, so diagnostics should be measured rather than assumed. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/590649)

**Mechanisms to implement:** selective trust, selective argument exchange, social sorting, coordinated elite messages, exclusion, and feedback from conflict.

Do **not** make “hearing disagreement causes movement in the opposite direction” the default. Mäs and Flache demonstrated a mechanism, supported by a group-discussion experiment, through which argument exchange and homophily can produce polarization without negative influence. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0074516)

Modern media experiments also discourage a single “echo chamber multiplier.” Reducing like-minded Facebook exposure produced no detectable change on several polarization measures in one large experiment, while reranking content expressing antidemocratic attitudes and partisan animosity changed affective polarization in another. These manipulated different properties of the information environment. [Princeton University](https://collaborate.princeton.edu/en/publications/like-minded-sources-on-facebook-are-prevalent-but-not-polarizing/)

### 1.7 Generational replacement and individual conversion are different processes

**Rule:** Children acquire initial dispositions through caregivers, peers, and institutions, with variation rather than exact copying. Continue allowing adult revision.

Longitudinal American research supports greater susceptibility during late adolescence and early adulthood, but not a simple rule that resistance to change increases continuously throughout life. Family transmission also varies with the consistency and political engagement of the family environment. [PubMed](https://pubmed.ncbi.nlm.nih.gov/2778632/)

Do not assign younger cohorts an automatic ideological direction. Their formative environment may favor hierarchy, redistribution, religious revival, national independence, or accommodation. Research on Algeria identified distinctive attitudes associated with coming of age under a particular political regime, illustrating the importance of historical context rather than universal generational labels. [JSTOR](https://www.jstor.org/stable/pdf/3521589.pdf)

Track population change as:

\[
\Delta\text{opinion stock}
=
\text{changes among continuing members}
+\text{opinions of entrants}
-\text{opinions of exits}.
\]

Entrants include migrants and people entering the measured political population; exits include deaths and out-migration.

**Arithmetic example, not an empirical rate:** replacing 1% of an adult population with entrants whose mean position differs by 0.20 changes the population mean by 0.002, before any existing adult changes their mind.

Age, period, and birth cohort are mathematically linked. Do not treat a cross-sectional age difference as proof of either lifelong aging effects or permanent cohort effects.

### 1.8 Public support and mobilization require separate decisions

**Rule:** Choose public expression using private preference, expected sanctions, local audience, loyalty, and perceived support. Choose costly action using those variables plus resources, expected efficacy, and organizational access.

Kuran’s preference-falsification theory explains how apparently stable regimes can encounter sudden public opposition without a corresponding overnight conversion of the population. It is a mechanism for investigating such discontinuities, not a universal prediction that repression inevitably produces collapse. [Scholars@Duke](https://scholars.duke.edu/person/t.kuran/scholarly-works/journal-articles)

For TCE, public demonstrations can update perceptions of support and reduce uncertainty. Conversely, punishment can suppress expression without restoring belief.

**Never use a universal population-share trigger for revolution.** Coordination experiments establish that tipping points can exist, but their location depends on incentives, memories, networks, and commitment. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf)

---

## 2. Opinion-dynamics models: what to borrow and what not to infer

| Model | Implementable rule | Useful behavior | Empirical fit and principal limitation |
| --- | --- | --- | --- |
| **DeGroot** | \(x(t+1)=Wx(t)\), with nonnegative, row-normalized influence weights. | Diffusion, convergence, unequal influence through network position. | A useful baseline for repeated social learning. Under suitable connectivity and aperiodicity conditions it converges to consensus; that is a mathematical property, not evidence that political societies should agree. |
| **Friedkin–Johnsen** | Combine network averaging with attachment to prior positions. | Persistent disagreement and heterogeneous susceptibility. | Better starting point for durable opinions. In TCE, replace permanently frozen initial anchors with slowly changing experience-based anchors. |
| **Bounded confidence: Hegselmann–Krause / Deffuant** | Average only sufficiently similar opinions; update neighborhoods synchronously or pairs asynchronously. | Consensus, multiple clusters, fragmentation. | Captures selective influence, but a universal confidence radius is not empirically established. Perfectly isolated, frozen clusters can be artifacts. |
| **Voter model** | Copy a randomly selected neighbor’s discrete state. | Neutral diffusion, drift, cluster coarsening. | Excellent null model. Without mutation or persistent opposing agents, a finite connected population eventually reaches consensus under standard assumptions. It lacks interests, arguments, and commitment. |
| **Axelrod cultural dissemination** | Interaction becomes more likely with shared traits; interaction copies a differing trait. | Local convergence alongside persistent cultural boundaries. | Useful for multiple cultural dimensions and similarity-dependent contact; not a validated model of entire political ideologies. |
| **Argument-exchange models** | Transmit considerations supporting or opposing positions, then reevaluate. | Bundled beliefs, confidence, polarization without automatic repulsion. | Closer to ideological reasoning, but needs authored argument content and more state. |

The DeGroot, Friedkin–Johnsen, and bounded-confidence formulations are presented together by Hegselmann and Krause; the voter-model limitations are explicit in Redner’s treatment. Axelrod and Mäs–Flache provide the multidimensional and argument-based alternatives. [JASSS](https://www.jasss.org/5/3/2.html)

**What does direct testing show?** Chandrasekhar, Larreguy, and Xandri tested social-learning models in experiments in India and Mexico. Their framework distinguishes Bayesian and DeGroot-like learners rather than assuming one learning rule for everyone. Such evidence is useful for factual learning on networks, but learning an objectively correct hidden state is not the same problem as choosing whose interests a constitution should protect. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.3982/ECTA14407)

**Recommended combination:** Friedkin–Johnsen-like persistence, confidence-sensitive influence, sparse argument memory, and endogenous material/identity-based evaluations. Keep voter and pure averaging models as inexpensive comparison cases.

---

## 3. Quantitative parameters and calibration ranges

### 3.1 Empirical benchmarks

These are **targets for reproducing particular experimental or observational settings**, not coefficients to copy indiscriminately into TCE.

Confidence below refers first to the original setting; historical portability is assessed separately.

| Quantity | Measured value and units | Scope and source | Confidence and appropriate use |
| --- | --- | --- | --- |
| Response to one peer estimate | **53% retained**, **43% compromised**, **4% adopted** the other estimate | 59 participants × 15 factual questions = 885 responses; Moussaïd et al., 2013. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0078433) | **Moderate** for this task; **low** portability to political convictions. Supports heterogeneous response types. |
| Ordinary general-election persuasion | Best pooled estimate **approximately 0 percentage points** in candidate choice | 40 existing plus 9 original field experiments; Kalla & Broockman, 2018. Early measured effects sometimes decayed. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/minimal-persuasive-effects-of-campaign-contact-in-general-elections-evidence-from-49-field-experiments/753665A313C4AB433DBF7110299B7433) | **High** for the studied campaign setting; does not imply zero long-run ideological influence or zero turnout effects. |
| Committed-minority tipping point | Approximately **25%** of participants | Naming-convention experiment: 194 recruited subjects, 10 groups; Centola et al., 2018. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf) | **Moderate** within that coordination task; **very low** as a general political threshold. |
| Reduced like-minded exposure | Approximately **one-third reduction**; **23,377 users**; no measurable effects on eight preregistered attitude outcomes | Facebook experiment during the 2020 US election; Nyhan et al., 2023. [Princeton University](https://collaborate.princeton.edu/en/publications/like-minded-sources-on-facebook-are-prevalent-but-not-polarizing/) | **High** for that intervention; not proof that media never polarize. |
| Reranking hostile political content | Approximately **2 thermometer points** of affective change on a roughly 100-point scale | Increased/decreased exposure to antidemocratic and partisan-animosity content; Piccardi et al., 2025. [Dryad](https://datadryad.org/dataset/doi%3A10.5061/dryad.hmgqnk9tj) | **Moderate–high** for this intervention; no justified centuries-long extrapolation. |
| Immediate depolarization effect | Mean **5.4 points**, standard error **0.4**, on a 0–100 thermometer | Meta-analysis reported by Holliday et al., 2025. [PNAS](https://www.pnas.org/doi/10.1073/pnas.2508827122) | **Moderate–high** for sampled interventions; useful for modeling short-lived affect changes. |
| Persistence of depolarization | Approximately **75% of the initial effect decayed after one week**; most change disappeared in follow-ups beyond two weeks | Follow-up evidence from 20 interventions in the same study. [PNAS](https://www.pnas.org/doi/10.1073/pnas.2508827122) | **Moderate**; do not apply this decay to deep values, identities, or every intervention. |

The contrast between rapid affect changes and durable political orientations argues for **several timescales**, not one opinion half-life.

There is no defensible universal numerical weight for “material interests versus identity versus values,” no cross-era annual ideology-conversion rate, and no established number of exposures required to create a new political doctrine.

### 3.2 Proposed TCE starting priors

**Every numerical range in this table is a design prior, not a historical estimate.** They are intended for sensitivity analysis and subsequent calibration. All issue positions use a \([0,1]\) scale.

| Parameter | Suggested default | Initial sensitivity range | Units and interpretation | Source/status |
| --- | --- | --- | --- | --- |
| Politically meaningful exposure rate | 0.5 | 0.05–5 | Episodes/person/week; derive actual opportunities from activity and institutions | **Design; low empirical confidence** |
| Base social-learning coefficient \(\eta\) | 0.03 | 0.005–0.15 | Fraction of message–opinion distance per eligible episode, before trust/salience gates | **Design**, informed qualitatively by heterogeneous experimental responses |
| Soft disagreement scale \(\epsilon\) | 0.25 | 0.10–0.60 | Normalized issue distance over which receptivity declines | **Design**; bounded-confidence theory supplies the mechanism, not this range |
| Ordinary experience-anchor adjustment | 2 | 0.25–10 | Years to halve distance toward a persistently favored new evaluation | **Design**; applies under sustained experience |
| Slow value adjustment | 20 | 5–50 | Years to halve distance toward a persistently favored new value configuration | **Design**; not spontaneous decay toward neutrality |
| Youth susceptibility multiplier | 2 | 1–3 | Relative to baseline adult susceptibility | **Design magnitude**; impressionable-years mechanism has empirical support |
| Broad developmental window | 12–30 | Vary onset/end | Years of age; smooth rather than abrupt transitions | **Design boundaries**, not a universal historical age interval |
| Preference for similar optional contacts | Odds multiplier 2 | 1–5 | Relative odds among otherwise comparable optional contacts | **Design**; do not apply to unavoidable household/work relationships |
| Explicit negative influence | 0 | 0–0.02 in stress tests | Fraction of opinion distance moved away per qualifying hostile interaction | **Design**; disabled in baseline |
| Local coordination threshold | Heterogeneous, centered near 0.5 | 0.1–0.9 | Perceived weighted local support share required for a specified action | **Design**; not a global revolution threshold |

For material, value, and identity weights, sweep normalized combinations over the full simplex rather than selecting one purportedly universal mixture. Permit variation by person, issue, and situation.

**Time conversion matters.** With a constant accepted message and coefficient \(a=0.03\), halving the initial distance requires:

\[
n\_{1/2}=\frac{\ln(0.5)}{\ln(1-0.03)}\approx23
\]

qualifying exposures. This is mathematical arithmetic, not an observed persuasion law. Trust gates, competing messages, and experience anchors alter the result.

Consequently, exposure frequency and learning strength must be calibrated jointly. A convincing-looking annual trend cannot identify both independently.

---

## 4. Variation across societies and the historical emergence of new ideologies

### 4.1 Use institutional conditions, not era-specific psychological species

| Setting | Historically relevant variation | TCE representation and caution |
| --- | --- | --- |
| **Foragers** | Some societies maintain egalitarian relations through active criticism, ridicule, coalition-building, and withdrawal of cooperation. Wiessner’s Ju/’hoansi study examined **308 conversations** involving norm enforcement. This is not evidence that all foragers were identical or egalitarian. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/204166) | Emphasize personal reputation, sharing obligations, autonomy, and limits on domination. Political norms can exist without formal parties or written doctrine. |
| **Early farming** | Archaeological evidence shows divergent trajectories of material inequality across Eurasia and the Americas; farming did not generate one uniform social outcome. Houses and burials are indirect evidence, not opinion surveys. [Nature](https://www.nature.com/articles/nature24646) | Make storage, land rights, inheritance, household dependence, and local authority salient. Treat reconstructed beliefs as uncertain scenarios. |
| **Pre-industrial states** | Political thought could be sophisticated and geographically extensive before modern mass literacy. Warring States thinkers debated political order; Ashoka promoted an empire-wide moral project through royal institutions and inscriptions. [OUP Academic](https://academic.oup.com/hawaii-scholarship-online/book/29959) | Allow courts, teachers, religious organizations, itinerant specialists, and oral transmission to carry doctrines. Written communication increases durability and reach, not the basic capacity for political thought. |
| **Industrializing societies** | Explicit programs linking property, labor, class, and political organization became important forms of ideological production. The 1848 Communist Manifesto is primary evidence of one such program, not proof that all wage workers shared it. [Marxists Internet Archive](https://www.marxists.org/archive/marx/works/1848/communist-manifesto/) | New workplaces, associations, communication systems, and state demands can change both interests and networks. Do not automatically turn workers socialist or owners liberal. |
| **Modern mass societies** | Surveys find relationships between economic development and values alongside substantial cultural persistence. Such associations do not establish an inevitable sequence from wealth to secularism or liberal democracy. [Sage Journals](https://journals.sagepub.com/doi/10.1177/000312240006500103) | Model education, security, media, institutional performance, and historical inheritance separately. Permit reversals and different combinations. |

These settings can coexist in one world. An industrial port may exchange people and ideas with agricultural villages governed through kinship institutions.

### 4.2 How new ideologies arise

For TCE, treat ideological innovation as **recombination plus organization**, rather than random invention of a complete doctrine.

A candidate ideology should combine an explanation of current problems, a moral justification, a proposed institutional arrangement, a constituency, and a claim about legitimate authority. Its authors must draw from concepts they have encountered or developed through experience.

Historical cases illustrate distinct pathways:

**Warring States China: competition over political order.**  
Pines traces enduring imperial ideas to debates before political unification. The lesson is that doctrines can precede the institutions they justify and can be selected through competition among patrons, advisers, and states. It is not that warfare mechanically selects one philosophy. [OUP Academic](https://academic.oup.com/hawaii-scholarship-online/book/29959)

**Mauryan South Asia: a ruler-supported moral program.**  
Ashoka’s presentation of *dharma* attempted to provide a common ethical orientation across a heterogeneous empire. For simulation purposes, this is a route in which an existing state promotes and institutionalizes a doctrine, rather than a grassroots movement first winning majority support. Royal inscriptions establish the official message more securely than its reception by every subject. [Yale University Press](https://yalebooks.yale.edu/book/9780300270006/ashoka/)

**The European Reformation: communication and organizational opportunity.**  
Rubin’s research links the earlier presence of printing to later Protestant adoption at the city level. The transferable mechanism is that cheaper reproduction can aid a challenge to established authority. It does not justify “printing unlocked → religious revolution,” and the causal interpretation depends on the historical identification strategy. [Chapman University Digital Commons](https://digitalcommons.chapman.edu/economics_articles/99/)

**The Haitian Revolution, 1791–1804: excluded people transform political claims.**  
Dubois documents how enslaved and free people of African descent, colonial actors, warfare, and revolutionary politics interacted in the destruction of slavery and creation of Haiti. New political possibilities emerged through struggle and coalition changes—not merely by copying European philosophical labels. [JSTOR](https://www.jstor.org/stable/j.ctv322v50g)

**Tanzania’s Ujamaa: translation and selective reconstruction.**  
Nyerere’s 1962 formulation articulated socialism in a locally framed moral vocabulary. This is useful evidence of how political actors construct a doctrine, but not independent proof that their account of traditional society was historically universal or that citizens accepted the program. [Julius Nyerere Foundation](https://www.juliusnyerere.org/resources/view/ujamaa_-_the_basis_of_african_socialism_julius_k._nyerere)

**Derived TCE rule:** When existing explanations repeatedly fail, actors with access to alternative concepts may propose a new combination. Survival then depends on communication, credibility, resources, coalition usefulness, and institutional adoption.

Allow doctrines to split, merge, change emphasis, or survive under new names. Do not require a new ideology to be entirely novel on every dimension.

---

## 5. Stylized facts and validation targets

A correct simulation should reproduce **conditional patterns**, not the same political history in every seed.

| Pattern to reproduce | Testable simulation requirement |
| --- | --- |
| **Stability alongside occasional rapid change** | Established commitments persist under ordinary life; major experiences and institutional shifts can change them. A single conversation should not routinely reverse a settled identity. Compare micro-responses and campaign-effect benchmarks in §3. |
| **Incomplete ideological alignment** | Citizens can share some positions while disagreeing on others. Increased issue alignment and increased extremity must remain distinguishable. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/590649) |
| **Confidence can change before position** | Reassurance strengthens commitment without necessarily moving a numerical opinion. Contradiction can create doubt before conversion. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0078433) |
| **Polarization without automatic repulsion** | The baseline can produce separated groups through selective interaction and argument exchange, with negative influence disabled. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0074516) |
| **Different economic positions do not imply perfectly separated political camps** | Material-interest-only simulations should differ from simulations containing identity, values, and perceived consequences. Land-title and party-cue findings supply distinct empirical checks. [OUP Academic](https://academic.oup.com/qje/article/122/1/209/1924727) |
| **Population change without mass conversion** | Holding incumbent adults’ beliefs fixed can still change the aggregate through entry, exit, and migration. Report this separately from within-person revision. |
| **Public discontinuity without equally abrupt private change** | Reduced fear or new information about support can produce a participation cascade while private preferences change little. This is a test of the Kuran mechanism, not a universal empirical law. [Scholars@Duke](https://scholars.duke.edu/person/t.kuran/scholarly-works/journal-articles) |
| **Institutional inheritance** | A doctrine can outlive its founder when teaching, succession, and resources persist; removing those supports can weaken transmission. |
| **Context-dependent tipping points** | Changing action costs, network structure, or commitment changes the critical mass. A hard-coded 25% trigger fails this test. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf) |
| **Affect and action need not move together** | A modest improvement in intergroup warmth should not automatically produce constitutional compromise or eliminate violence. Depolarization research finds uncertain downstream effects. [PNAS](https://www.pnas.org/doi/10.1073/pnas.2508827122) |

For statistical validation, compare distributions, transition rates, within-person persistence, intergenerational resemblance, network assortativity, and the frequency and duration of faction splits. Matching only an aggregate polarization curve leaves too many mechanisms unidentified.

---

## 6. Recommended TCE architecture, calibration, and sources

### 6.1 Minimal viable representation

A practical first implementation would store:

**Per person:** four to eight slow motivational weights; a sparse set of roughly six to twelve active issue records; confidence and salience for each; a few group attachments; perceived institutional legitimacy; and short memories of consequential events and arguments.

**Per relationship:** trust, interaction context, familiarity, and recent influence history. Kinship and work relationships should not vanish merely because opinions diverge.

**Per institution:** doctrine, membership, officers, resources, teaching or broadcasting capacity, public positions, sanctioning powers, and succession rules.

**Per faction:** a coalition of people and institutions, a policy program, organizers, resources, commitments, and internal disagreements.

Those counts are **engineering choices**, not measured cognitive limits.

An ideology should exist as a structured record such as:

> Problem explanation → moral commitments → institutional proposals → supporting groups → legitimacy narrative.

The text shown to players can be generated from this structure. The simulation should operate on proposition IDs and relationships, not free-form prose.

### 6.2 A computationally economical update cycle

**Daily life generates events.** Relevant encounters, punishments, disputes, employment changes, rituals, and witnessed failures place items in a bounded political-event queue.

**Weekly or monthly processing updates political state.** Only salient issues and affected relationships need reevaluation. Major shocks can trigger immediate processing.

**Institutions periodically select messages and proposals.** Evaluate a limited number of candidates against constituencies and organizational constraints, rather than making every person search every possible doctrine.

**Annual demographic accounting separates replacement from conversion.** Preserve named characters’ histories and institutional transmission chains.

Use asynchronous, reproducibly seeded updates to avoid artificial population-wide oscillations. Rendering frequency should not determine the frequency of persuasion.

As a rough engineering estimate, 50,000 people with 16 cached directed influence links produce 800,000 links. At 16 bytes per compact link, that is about **12.8 MB before container overhead**. Two processed influence events per person per simulated week produce 100,000 events. These are workload estimates, not a claim about achieved frame rate.

### 6.3 Constitutional politics requires power, not just opinion aggregation

Do not enact whichever constitution has the highest population-average support.

Instead, let constitutional outcomes depend on the participating coalition, legal standing, property control, military organization, bargaining commitments, and capacity to enforce a settlement. Excluded groups may have strong preferences without representation.

This is a modeling recommendation: **opinion determines some demands and loyalties; institutions determine whose demands can become rules**.

Also preserve a gap between adopting a constitution and accepting its legitimacy. A negotiated rule can stabilize behavior before becoming widely valued, or remain formally in force while losing support.

### 6.4 What to simplify first

Omit exact Bayesian reasoning, unrestricted natural-language debate, a fully dynamic global friendship network, and hundreds of simultaneously active issues.

Retain the distinctions whose removal changes the causal story: interests versus beliefs; private versus public support; identity versus issue agreement; exposure versus acceptance; and demographic replacement versus conversion.

Keep counterfactual baselines available. Compare the hybrid against material-interest-only agents, pure DeGroot averaging, a voter model, and bounded confidence. Ablate family transmission, identity, organizational memory, and network selection one at a time.

### 6.5 Existing models and games worth examining

**Axelrod’s cultural dissemination model** is a compact reference for multidimensional similarity and cultural boundaries. **Epstein’s civil-violence model** is useful for separating grievance from overt rebellion under perceived risk, but it is not a model of ideological content or constitutional bargaining. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002797041002001)

**RimWorld: Ideology** provides a useful design example of belief bundles affecting roles, rituals, preferences, and social life. **Victoria 3** provides a reference for connecting socioeconomic conflict with political and institutional change. These are design precedents, not empirical validation, and neither should supply timeless ideological categories for TCE. [RimWorld](https://rimworldgame.com/ideology/)

### 6.6 Datasets and source strategy

| Source | Best use in TCE research | Main limitation |
| --- | --- | --- |
| **World Values Survey** | Joint distributions of values, institutional confidence, religiosity, and policy attitudes; cross-national comparisons. [World Values Survey](https://www.worldvaluessurvey.org/WVSDocumentationWV7.jsp) | Mostly repeated cross-sections, not trajectories of the same individuals. Translation and measurement comparability require attention. |
| **American National Election Studies** | Panel persistence, party attachment, issue positions, affect, and measurement models. [ANES](https://electionstudies.org/data-center/) | One unusually well-documented contemporary political setting, not a universal template. |
| **Afrobarometer and Arab Barometer** | Political trust, governance evaluations, identity, and policy attitudes outside Europe and North America. [Afrobarometer](https://www.afrobarometer.org/data/) | Coverage, questionnaire content, and political conditions vary across places and rounds. |
| **Asian Barometer and AmericasBarometer/LAPOP** | Comparative political orientations in Asian and American societies. [Asian Barometer](https://www.asianbarometer.org/) | Validate the meaning of apparently similar concepts before pooling coefficients. |
| **D-PLACE** | Ethnographic variation in subsistence, kinship, social organization, language, and environment. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) | Institutional/cultural coding, not individual opinion panels; related societies are not independent observations. |
| **Seshat** | Historical institutional configurations and long-run sequences for scenario comparison. [eScholarship](https://escholarship.org/uc/item/9qx38718) | Sparse and interpretive historical evidence; official institutions should not be equated with mass belief. |
| **Experimental replication data** | Reproduce local response distributions before extrapolating; the Centola and Piccardi projects provide relevant experimental resources. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf) | Laboratory or platform environments differ sharply from settlement politics. |

Use a **measurement layer** when comparing simulation outputs to surveys. A latent belief is not identical to one questionnaire response; public reporting can contain uncertainty, wording effects, and fear.

### 6.7 Claims to treat cautiously

**Universal ideological axes:** useful for summarizing a particular population, but unsuitable as immutable human coordinates. Let issue bundles and their correlations emerge.

**Automatic modernization:** associations between security, development, and values do not establish a compulsory historical sequence. [Sage Journals](https://journals.sagepub.com/doi/10.1177/000312240006500103)

**Universal committed-minority thresholds:** the experimental 25% result is explicitly context-dependent. [Network Dynamics Group](https://ndg.asc.upenn.edu/wp-content/uploads/2018/06/Centola-et-al.-2018-Science.-Tipping-Point.pdf)

**Uncritical formative-experience coefficients:** the widely cited *Growing up in a Recession* paper was retracted in 2023 because its original findings could not be replicated, likely owing to a coding error. It should not supply a recession-to-lifelong-ideology parameter. [The Review of Economic Studies](https://www.restud.com/retraction-of-growing-up-in-a-recession/)

**Ancient mass opinion reconstructed from elite texts:** a ruler’s inscription or philosopher’s surviving work establishes an articulated doctrine more securely than its prevalence among ordinary people.

---

## Bottom line

TCE’s most productive abstraction is not “people move around an ideology map.” It is:

**People experience conditions, interpret them through inherited and encountered ideas, trust some messengers, attach themselves to groups, and decide whether expressing or acting on a belief is worthwhile. Institutions preserve some resulting combinations and give them political force.**

Build those processes separately, connect them through daily life and institutional change, and calibrate their observable consequences. That architecture can generate conformity, pluralism, entrenched factions, reform, schism, and revolution without prescribing which ideology must appear next.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92834-96cc-83e9-b7e7-a863bee36a48)
