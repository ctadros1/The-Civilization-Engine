# Social networks for The Civilization Engine

## Executive recommendation

**TCE should generate relationships from shared lives—not generate a graph first and make people conform to it.** Households, work, residence, worship, travel, marriage, and political institutions should determine whom people encounter. Repeated interaction, compatibility, mutual assistance, conflict, and limited attention should then determine which encounters become durable relationships.

The strongest practical model is a **temporal, multiplex network**: *temporal* because relationships and contacts change; *multiplex* because the same people can simultaneously be relatives, coworkers, friends, creditors, and rivals. Research on rural Indian networks, for example, measures visiting, borrowing, advice, kinship, and worship separately rather than treating them as interchangeable friendships. [MIT Economics](https://economics.mit.edu/sites/default/files/2022-08/science.1236498.pdf)

Use Dunbar-like layers as an approximate pattern to test, **not a hard limit of 150 relationships or a maximum settlement size**. Communication data support unequal, layered social investment, but the precise cognitive ceiling inferred from primate comparisons is contested. [arXiv](https://arxiv.org/abs/1604.02400)

For implementation, I recommend:

**Actual encounters → selective interaction → directional relationship updates → attention competition → dormancy or reactivation**, all operating alongside persistent kinship and institutional obligations.

---

# 1. Mechanisms: implementable rules

## 1.1 Separate four things that games often conflate

The following is a proposed TCE representation, informed by the distinctions in the empirical literature.

| Layer | What it represents | Appropriate persistence | What it must not imply |
| --- | --- | --- | --- |
| **Genealogy and recognized kinship** | Parentage, siblings, marriage, adoption, socially recognized descent | Genealogy persists; recognition and obligations depend on institutions and knowledge | Every relative is liked, known personally, or encountered regularly |
| **Institutional affiliation and roles** | Household, work group, congregation, guild, military unit, patronage, office | Until membership or role changes | All members are friends or interact with every other member |
| **Personal relationships** | Familiarity, affection, trust, resentment, fear, remembered assistance | Reinforced, weakened, damaged, dormant, or reactivated | Feelings are reciprocal or reducible to one score |
| **Contact events** | Conversation, shared room, physical interaction, joint work, attendance | Minutes to days; usually aggregated or discarded afterward | Every contact creates a lasting relationship |

The practical consequence is important: **a disliked employer can transmit disease, an absent sibling can remain an obligation, and a respected patron can influence someone without being a friend.**

Kinship and friendship should therefore have different update rules. Longitudinal research finds that their maintenance and emotional dynamics differ; treating them as identical decaying edges loses that distinction. [Springer](https://link.springer.com/article/10.1007/s12110-015-9242-7)

## 1.2 Co-location creates opportunities, not automatic friendship

Feld’s *focused organization of social ties* is especially useful for TCE. A “focus” is a shared setting or activity around which interaction becomes organized: a household, workshop, neighborhood, congregation, or association. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/227352)

**Simulation rule:** Generate potential interaction partners from actual attendance and schedules.

A farm worker’s candidates might come from their household, adjacent fields, harvest crew, water source, market, and seasonal festival. A town artisan’s candidates might instead come from a workshop, customers, apprentices, neighbors, and an association.

Then distinguish three levels:

**Exposure:** “I recognize this person.”  
**Interaction:** “We spoke or worked together.”  
**Relationship-relevant experience:** “They helped me, entertained me, humiliated me, kept a promise, or demonstrated competence.”

Repeated exposure can increase familiarity without increasing affection. Compulsory work together should not automatically create close friendship.

There is causal evidence that proximity matters: a randomized seating experiment across 182 Hungarian classrooms increased mutual friendship probability from approximately 15% to 22% for adjacent pupils over a semester. That is evidence for an opportunity effect—not a universal probability of befriending a neighbor. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0255097)

**Institutional design consequence:** A new bridge, market, school, shrine, or workshop changes the social network by changing repeated opportunities, without requiring a scripted “community cohesion” bonus.

## 1.3 Apply homophily after opportunity has been determined

Homophily means that similar people are more likely to be connected. However, similarity can arise because people **meet similar people**, because they **prefer them**, or because connected people **become more similar**. The proximity experiment demonstrates that changing opportunities can create friendships across existing social differences; Hadza research also finds homophily in a small-scale society. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0255097)

For TCE, distinguish:

* **Opportunity sorting:** Residence, occupation, language, status, restrictions on movement, and institutional admission determine access.
* **Conditional preference:** Among accessible people, shared interests, values, experience, or conversational compatibility affect voluntary engagement.
* **Influence:** Interaction subsequently changes beliefs or practices.

Do not apply the same similarity bonus to every relationship type. Friendship may favor similarity, while trade, apprenticeship, marriage arrangements, and patronage may depend on **complementary resources or roles**.

A suitable proposed rule is:

\[
P(\text{engage}\mid\text{opportunity})
=
\sigma\!\left(
b\_i+\beta\_s S\_{ij}+\beta\_c C\_{ij}
+\beta\_r R\_{ij}-\beta\_h H\_{ij}
\right)
\]

Here, \(S\) is relevant similarity, \(C\) represents trusted introductions, \(R\) is role compatibility or expected benefit, and \(H\) is hostility. The coefficients require calibration; they are not universal psychological constants.

Represent discrimination through beliefs, norms, and institutional constraints—not innate coefficients attached to ancestry.

## 1.4 Triadic closure should require an actual introduction

Triadic closure occurs when two people with a common acquaintance become connected. It is a central ingredient in generative models that reproduce clustered social networks. [IDEAS/RePEc](https://ideas.repec.org/a/eee/phsmap/v371y2006i2p851-860.html)

**Simulation rule:** A trusted acquaintance sometimes brings another person into an activity, recommends them for work, arranges a meeting, or shares relevant information about them.

Do not periodically connect random friends-of-friends regardless of geography. A common acquaintance should increase either:

1. the chance of a meeting, or
2. willingness to engage after meeting.

It should not instantly create friendship.

Also preserve opportunities that do **not** pass through existing friends: migration, markets, travel, marriage, recruitment, and public events. Otherwise, closure progressively seals communities into disconnected cliques.

Weak ties deserve particular protection from indiscriminate pruning: ties between otherwise separated circles can provide access to information or opportunities unavailable within a close circle. This is the central mechanism of Granovetter’s weak-tie argument, not a claim that every weak tie is valuable. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/225469?ref=vitalcitynyc.org)

## 1.5 Make time and attention the binding resources

Give people a limited social-attention budget that competes with work, travel, care, rest, and other needs. Do not merely impose a maximum number of edges.

Longitudinal phone research finds that individuals can preserve distinctive patterns of how they allocate communication across contacts even while the identities of those contacts change. This supports heterogeneous allocation habits rather than identical relationship slots for everyone. [arXiv](https://arxiv.org/html/1204.5602v2)

**Proposed rule:** People preferentially maintain relationships that currently matter, but distribute attention differently according to personality, obligations, life stage, and circumstances.

Some maintain a concentrated inner circle; others distribute attention more broadly. A new intimate relationship can crowd out maintenance of others without deleting them immediately.

For group activities, distinguish shared presence from focused engagement. A two-hour feast can expose someone to many people, but should not award two hours of intimate bonding with every attendee. Conversely, a speech can reach many listeners without requiring a separate conversation with each.

This also solves the patron problem: **an official may have hundreds of organizational connections without having hundreds of intimate friends.**

## 1.6 Reinforce relationships through differentiated experience

Hall’s friendship-formation studies associate closer friendship with accumulated time together, while also showing that what people do together matters. These are probabilistic associations, not thresholds at which friendship automatically unlocks. [Sage Journals](https://journals.sagepub.com/doi/full/10.1177/0265407518761225)

For TCE, update different dimensions from different events:

| Experience | Suggested relationship effect |
| --- | --- |
| Repeated neutral encounters | Familiarity and recognition |
| Enjoyable voluntary interaction | Affection and willingness to seek future contact |
| Reliable help or fulfilled promises | Trust; possibly gratitude or obligation |
| Skilled performance | Domain-specific respect |
| Threat or coercion | Fear, avoidance, resentment; possibly compliance |
| Betrayal or humiliation | Trust loss, resentment, remembered grievance |
| Shared hardship | Potential bonding, but conditional on behavior during the hardship |
| Repeated unreciprocated requests | Reduced willingness to help, unless strong obligations override it |

These are **proposed authored event rules**, not estimates of universal effect sizes.

Keep attitudes directional. A may admire B while B barely remembers A. A rival may also be a respected specialist. Friendship, fear, dependence, and grievance should not cancel into one “opinion” number.

## 1.7 Model several kinds of decay

“Relationship decay” can mean reduced emotional closeness, less frequent contact, disappearance from a survey’s named contacts, or complete loss of recognition. These are different outcomes.

Research tracking life transitions finds maintenance-sensitive changes in closeness. A separate study of transient phone relationships found comparatively stable communication activity before cessation rather than a universal smooth decline. Together, these argue against one timer governing every aspect of a tie. [Springer](https://link.springer.com/article/10.1007/s12110-015-9242-7)

Use four processes:

**Salience fades:** A person becomes less likely to come to mind.

**Current closeness weakens:** Reduced rewarding interaction makes the relationship less central.

**Contact opportunities disappear:** Moving, changing work, institutional closure, or conflict interrupts interaction.

**Durable history remains:** Shared childhood, parentage, major assistance, betrayal, and unresolved obligations can survive long gaps.

A practical state machine is:

`unfamiliar → recognized → active → dormant → reactivated`

“Hostile” and “obligated” should be independent states or dimensions, not positions on that single progression.

Do not make dormant friends socially newborn when they meet again. Equally, do not guarantee that old friendship restores itself: remembered history should affect reactivation, while current compatibility and conduct still matter.

## 1.8 Couple relationships to life events and material conditions

For TCE, marriage, childbirth, apprenticeship, migration, bereavement, recruitment, retirement, and institutional collapse should alter **opportunities and demands**, rather than trigger arbitrary graph rewiring.

A longitudinal Dutch study found substantial changes in personal relationship membership while network size changed relatively little, with meeting opportunities important to emergence and discontinuation. [Utrecht University Research Portal](https://research-portal.uu.nl/en/publications/changes-in-personal-relationships-how-social-contexts-affect-the-/)

A useful implementation principle is to allow opposing effects. Scarcity might reduce leisure and travel while increasing requests for assistance. Migration might interrupt local friendship while activating distant kin or patrons. Which effect dominates should follow from circumstances, not a universal “hardship reduces friendship” multiplier.

## 1.9 Give disease, gossip, influence, and help different transmission rules

Physical-contact diaries measure a different network from friendship surveys. POLYMOD, for example, records daily conversational or physical contacts relevant to transmission, including contacts outside intimate relationships. [PLOS](https://journals.plos.org/plosmedicine/article?id=10.1371%2Fjournal.pmed.0050074)

For TCE:

**Disease** should use the relevant contact or shared-environment events, not affection scores.

**Gossip** should require knowledge, an opportunity to communicate, interest, and willingness. Store who heard what from whom; do not provide every citizen with an omniscient reputation score.

**Influence** should depend on perceived credibility, identity, dependence, and the subject involved. Awareness and adoption must be separate: the Indian microfinance research explicitly distinguishes information transmission from participation decisions. [MIT Economics](https://economics.mit.edu/sites/default/files/2022-08/science.1236498.pdf)

**Costly collective behavior** may require reinforcement from several contacts. Centola’s experiment shows that clustered networks can help such behavioral adoption even when shortcuts would help simple information travel. Do not use one contagion rule for rumors, disease, conversion, and rebellion. [Science](https://www.science.org/doi/10.1126/science.1185231)

---

# 2. Parameters: measured benchmarks versus proposed defaults

## 2.1 Empirical benchmarks

**Confidence terminology:** “High within study” means the reported measurement or experimental result is reasonably secure for that design. It does not mean high confidence in transferring the number to every era or society.

| Quantity | Reported value and units | Interpretation for TCE | Source and confidence |
| --- | --- | --- | --- |
| Approximate personal-network layers | **5, 15, 50, 150 people**, cumulative | Approximate levels of progressively weaker involvement—not 220 separate slots. Layer positions vary. | Mac Carron, Kaski & Dunbar, *Calling Dunbar’s Numbers* (2016). **Moderate** as a pattern; **low** as fixed limits. [arXiv](https://arxiv.org/abs/1604.02400) |
| Time associated with friendship development | Roughly **40–60 hours** for casual friendship; **80–100 hours** for friendship; around **200+ hours** for close friendship | Cumulative time since meeting, not additive stages; activity quality and study population matter. | Hall, *How Many Hours Does It Take to Make a Friend?* (2019; online 2018). **Moderate association; low universal transfer.** [Sage Journals](https://journals.sagepub.com/doi/full/10.1177/0265407518761225) |
| Experimental proximity effect | Mutual friendship approximately **15% → 22%** over a semester | A **7-percentage-point** effect for adjacent seating, not a daily formation probability. | Rohrer, Keller & Elwert (2021), **2,966 pupils**. **High within experiment; limited historical transfer.** [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0255097) |
| Intimate-tie persistence | **27%** still intimate after **10 years** | Loss of intimacy is not necessarily forgetting or complete loss of contact; deaths also matter. | Wellman et al., *A Decade of Network Change* (1997), **33 Toronto respondents**. **Low–moderate**, small contextual sample. [Academia](https://www.academia.edu/24723158/A_decade_of_network_change_Turnover_persistence_and_stability_in_personal_communities) |
| Daily contact count | Mean **13.4 distinct people/day** | Contact degree, not number of friends. One-day diaries do not measure the full personal network. | Mossong et al., POLYMOD (2008): **7,290 participants**, eight European countries. **High descriptive; low ancient-world transfer.** [PLOS](https://journals.plos.org/plosmedicine/article?id=10.1371%2Fjournal.pmed.0050074) |
| Forager residential-group size | Mean experienced band size **28.2 adults** | Adults, not total population; residential group, not complete friendship network. | Hill et al. (2011): **5,067 individuals**, 32 societies. **Moderate comparative confidence.** [Academia.edu](https://independent.academia.edu/BarrySHewlett) |
| Hadza network transitivity | **0.16–0.17** for campmate nominations; **0.41** for gift ties | Strong dependence on which relationship is measured and how nominations are collected. | Apicella et al. (2012), **205 adults in 17 camps**. **Moderate within design; low universal transfer.** [Human Nature Lab](https://humannaturelab.net/sites/default/files/pdf/paper/127-Social-Networks-and-Cooperation-in-Hunter-Gatherers.pdf) |
| City-size relationship | Total degree approximately \(K\propto N^{1.12}\); mean degree approximately \(\bar k\propto N^{0.12}\) | In one Portuguese phone-network specification, a tenfold population increase implies about **1.32×**, not 10×, mean degree; the latter is calculated from the exponent. | Schläpfer et al. (2014), **140 cities**, 409-day reciprocal-call window. **Moderate for that network; low preindustrial transfer.** [arXiv](https://arxiv.org/pdf/1210.5215) |

**There is no defensible universal empirical table of friendship half-lives by historical era.** The available studies measure different outcomes, populations, and communication channels. A precise-looking decay constant inferred from one of them would hide more uncertainty than it resolves.

## 2.2 Proposed starting parameters for TCE

The following are **engineering priors authored for this recommendation, not measured human constants**. Their confidence is **low until TCE is calibrated**. Use them to build a functioning prototype, then sweep and fit them against multiple observations.

| Parameter | Initial value or range | Units | Intended use |
| --- | --- | --- | --- |
| Focused relationship-maintenance budget | **45–180** | effective social minutes/person/day | Includes meaningful interaction during meals or work; excludes passive co-presence |
| Substantive update episodes | **2–8** | episodes/person/day | A computational starting range, not a cap on physical contacts |
| Introduced candidates | **5–20%** | share of new candidate encounters | Only where an actual mediator and meeting opportunity exist |
| Conditional similarity preference | **1–3×** | engagement odds multiplier | Sensitivity range after opportunity sorting; allow neutrality and role-specific differences |
| Recent-contact salience half-life | **14–60** | days | How readily a contact comes to mind |
| Familiarity half-life without exposure | **90–365** | days | Recognition strength; major shared history should modify or bypass simple fading |
| Current warmth half-life without reinforcement | **1–5** | years | Only current emotional activation; not genealogy, durable trust evidence, or obligations |
| Positive reinforcement coefficient | **0.005–0.02** | per effective social hour | Saturating increase in current warmth |
| Sensitivity sweep | **0.5× and 2×** each starting value | multiplier | Identify brittle assumptions and compensating parameter combinations |

Use distributions across people rather than one value for everyone. The relative importance of time availability, maintenance preferences, relationship history, and circumstances should be tested before adding elaborate personality-to-network coefficients.

### A minimal continuous update

For a normalized **current warmth** variable \(w\in[0,1]\), one proposed approximation is:

\[
w^-\_{ij}=w\_{ij}\,2^{-\Delta t/H\_{ij}}
\]\[
w^+\_{ij}
=
\operatorname{clip}\_{[0,1]}
\left[
w^-\_{ij}
+(1-w^-\_{ij})(1-e^{-\alpha\_i q\_{ij}d})
-h\_{ij}
\right]
\]

Here, \(\Delta t\) and half-life \(H\) use the same time unit; \(d\) is interaction duration in hours; \(q\) is nonnegative effective quality; and \(h\) is an event-specific negative shock.

As a **calculated illustration**, with no decay and \(\alpha=0.01\) per effective hour, warmth rises from zero to approximately 0.39 after 50 hours, 0.63 after 100, and 0.86 after 200. This is a tunable curve, **not a validated conversion from hours to friendship**.

Do not apply this equation indiscriminately to trust, debt, kinship, resentment, and fear. Trust should respond to relevant evidence; an unpaid obligation should follow its own institutional rules.

---

# 3. Variation across eras, regions, and settlement sizes

## 3.1 Historical variation should emerge from social organization

The evidence does not support a single progression from “kin-based village” to “individualistic city.”

| Setting | Evidence and limits | Implication for TCE |
| --- | --- | --- |
| **Foragers** | Hill et al.’s comparative study found flexible residence and many co-resident adults who were not close genetic relatives. Agta research in the Philippines identifies social organization across families, camps, and regions, rather than one isolated band. [Academia.edu](https://independent.academia.edu/BarrySHewlett) | Generate nested but permeable groups. Preserve intercamp visiting, marriage, and information routes when camps split or move. |
| **Forager risk-sharing networks** | Ju/’hoansi *hxaro* research in southern Africa documents exchange partnerships associated with visiting and assistance beyond the immediate residential group. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S109051380200096X) | Allow distant partners to remain strategically important despite infrequent contact. Local daily interaction should not erase all external connections. |
| **Early farming** | Ancient DNA from Gurgy, France, supports patrilocal and exogamous organization in that community. Fujia, eastern China, instead provides evidence of a two-clanned matrilineal community around **2750–2500 BCE**. These are site-specific reconstructions, not global templates. [Nature](https://www.nature.com/articles/s41586-023-06350-8) | Make descent, postmarital residence, inheritance, and marriage rules composable institutions. Do not hard-code farming as patrilineal. Matrilineal descent does not itself establish female political dominance. |
| **Horticultural societies of the Americas** | Walker et al. compare **34 lowland Central and South American societies**, finding substantial variation in co-resident kin composition and sex biases. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/walkeretal2013.pdf) | Different residence and marriage rules can produce different networks under broadly comparable subsistence systems. |
| **Traditional and preindustrial corporate organization** | Sangren’s study of Chinese corporate forms, using cases from Taiwan, challenges treating lineages as categorically separate from the wider range of formal associations. It is comparative institutional evidence, not a complete medieval friendship census. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-asian-studies/article/traditional-chinese-corporations-beyond-kinship/F894DFF5E875EE6D6BEF877F0BD98675) | Represent lineage groups, associations, and religious or economic bodies as organizations with overlapping memberships, property, rules, and activities—not merely enlarged families. |
| **Industrial communities** | Marttila’s study of a Finnish ironworks, **1880–1950**, finds occupation, apprenticeship, kinship, and marriage deeply intertwined. Industrial work did not simply replace kin networks with impersonal labor relations. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/monopolizing-the-property-of-skill-a-prosopographic-analysis-of-a-finnish-ironworks-community/795937FC5F122215B022410F4CAAE609) | Factories can create repeated contact while recruitment and skill transmission remain family- or patron-mediated. Let institutional change alter those mechanisms. |
| **Modern rural and urban societies** | Karnataka village research measures many overlapping relation types; urban phone studies show that larger cities can have more connections without a corresponding collapse in local clustering. [MIT Economics](https://economics.mit.edu/sites/default/files/2022-08/science.1236498.pdf) | Modernity should alter transport, institutions, and communication costs—not switch kinship off or replace community with random contacts. |

Contemporary foragers and horticulturalists are not unchanged representatives of prehistory. Ancient pedigrees reveal biological relationships and burial selection much more directly than affection, trust, or everyday friendship. Those limits should remain visible in calibration.

### The variables that should actually differ

Instead of an “era multiplier,” vary **residential mobility, travel time, marriage residence, descent rules, household composition, labor organization, institutional access, settlement layout, inequality, and communication technology**.

For example, moving after marriage changes the spatial distribution of close kin. Restrictions on entering occupations alter who meets whom. A market held periodically creates recurring cross-settlement encounters. These are proposed causal inputs—not fixed claims about every society assigned to an era.

## 3.2 Village, town, and city networks

Treat these as tendencies produced by opportunities and institutions, not population thresholds.

| Dimension | Village-like setting | Town-like setting | City-like setting |
| --- | --- | --- | --- |
| Repeated encounters | A substantial share of residents repeatedly encounter one another | Repetition within neighborhoods and occupations, plus market contacts | Repetition within selected settings; many other residents never become personally known |
| Overlap of roles | The same person may be kin, neighbor, coworker, and creditor | Some overlap; more specialized roles and associations | Potentially greater separation among work, residence, worship, and leisure circles |
| Bridging ties | Visitors, marriage partners, traders, external kin | Brokers between villages, occupations, and institutions | Bridges among locally clustered communities and across settlements |
| Knowledge | Broad recognition can coexist with unequal intimacy | Reputation becomes uneven across groups | Reputation and information become strongly localized unless institutions broadcast them |
| Model risk | Mistakenly making the village a complete friendship clique | Missing occupational and institutional overlap | Mistakenly treating urban residents as socially isolated or randomly connected |

These columns are **modeling expectations to test**, not universal measured profiles.

One distinction is mathematical: for an undirected graph,

\[
\text{density}=\frac{\bar{k}}{N-1}.
\]

Density therefore falls as population grows unless mean degree grows comparably. That does **not** imply that friends become less connected to one another. Schläpfer et al. found local clustering largely invariant with city size in their communication-network analysis. [arXiv](https://arxiv.org/pdf/1210.5215)

**TCE should therefore allow a city to contain strongly connected neighborhoods and circles while remaining globally sparse.**

---

# 4. Stylized facts and validation tests

A correct simulation should reproduce several patterns simultaneously. Matching mean degree alone is insufficient.

| Pattern | What to measure | Validation target or warning |
| --- | --- | --- |
| **Unequal relationship investment** | Ranked contact time and relationship intensity per person | A small inner set receives disproportionate attention; layer positions and concentration vary. Do not require every agent to have exactly five closest contacts. [arXiv](https://arxiv.org/abs/1604.02400) |
| **Broad degree distributions** | Degree distribution separately for friends, recognition, roles, and contacts | Allow substantial heterogeneity. Do not assume a universal power law merely because the tail looks straight on logarithmic axes. [Nature](https://www.nature.com/articles/s41467-019-08746-5.pdf) |
| **Clustering depends on relationship type** | Triangle closure, transitivity, and local clustering per layer | Reproduce the appropriate survey-defined benchmark, such as the different Hadza campmate and gift networks—not one global clustering target. [Human Nature Lab](https://humannaturelab.net/sites/default/files/pdf/paper/127-Social-Networks-and-Cooperation-in-Hunter-Gatherers.pdf) |
| **Homophily is conditional on opportunity** | Similarity of connected pairs relative to accessible candidates | Compare against people who could realistically have met, not only against the entire population. The seating experiment provides an intervention benchmark. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0255097) |
| **Kinship is neither the whole network nor irrelevant** | Kin fraction by household, camp, activity, and support relation | Residential groups can include many non-close-kin adults; kin composition differs across societies. [Academia.edu](https://independent.academia.edu/BarrySHewlett) |
| **Stable allocation can coexist with turnover** | Rank-allocation profiles and contact identity overlap through time | Individuals should be able to replace contacts without continually expanding their network or completely changing their allocation pattern. [arXiv](https://arxiv.org/html/1204.5602v2) |
| **Relationships can weaken without vanishing** | Active → dormant transitions and later reactivation | A person no longer named as intimate may still remain a casual contact or recognized acquaintance. [Academia](https://www.academia.edu/24723158/A_decade_of_network_change_Turnover_persistence_and_stability_in_personal_communities) |
| **Physical contact has its own structure** | Daily contacts, duration, age mixing, setting, recurrence | Compare to contact diaries, not friendship-layer counts. Household intergenerational contacts and age-assortative contacts both matter. [PLOS](https://journals.plos.org/plosmedicine/article?id=10.1371%2Fjournal.pmed.0050074) |
| **Urban sparsity can coexist with local cohesion** | Mean degree, density, local clustering versus settlement size | These metrics should not be forced to move together. [arXiv](https://arxiv.org/pdf/1210.5215) |
| **Diffusion depends on process** | Reach and adoption under different contact structures | A network that spreads awareness rapidly need not maximize costly adoption or coordinated action. [Science](https://www.science.org/doi/10.1126/science.1185231) |

## Build a simulated survey instrument

This is essential. Compare **observations of TCE** with observations of people, rather than comparing complete internal state with incomplete surveys.

For every empirical comparison, reproduce the source’s:

**Population boundary, nomination limits, relation definition, directionality, observation window, and missing-data pattern.**

For example, the Hadza gift task restricted what participants could allocate; it is not an unrestricted count of everyone they liked. Likewise, a one-day contact diary and a year-long phone network measure different things. [Human Nature Lab](https://humannaturelab.net/sites/default/files/pdf/paper/127-Social-Networks-and-Cooperation-in-Hunter-Gatherers.pdf)

For undirected internal diagnostics, report both:

\[
\text{transitivity}=\frac{3\,\text{triangles}}{\text{connected triples}}
\]

and mean local clustering. They are not interchangeable; the latter also depends on how agents with fewer than two neighbors are handled.

Finally, test **interventions**, not just snapshots: relocate households, open a market, close a workplace, introduce a marriage-residence rule, or interrupt travel. Several generative processes can produce similar static graphs but respond differently to these changes. Dynamic-network modeling explicitly distinguishes current tie prevalence from formation and duration. [OUP Academic](https://academic.oup.com/jrsssb/article/76/1/29/7075934)

---

# 5. Modeling recommendation for a 10k–50k-agent Rust kernel

## 5.1 Use a hybrid generative model

| Model or approach | Useful contribution | Recommended role in TCE |
| --- | --- | --- |
| **Kumpula et al.: weighted community formation** | Local weighted search, reinforcement, additional connections, and turnover generate communities and ties between them | Best conceptual starting point. Replace abstract meetings and turnover with actual activity, migration, birth, death, and institutional change. [arXiv](https://arxiv.org/abs/0708.0925) |
| **Toivonen et al.: random contacts plus their neighbors** | A compact growth model producing clustering, broad degrees, and social-network structure | Useful for initialization and controlled experiments; insufficient by itself for full life-course dynamics. [IDEAS/RePEc](https://ideas.repec.org/a/eee/phsmap/v371y2006i2p851-860.html) |
| **Affiliation-based opportunity model** | People share settings rather than independently choosing from the world population | Use as the meeting generator: households, workplaces, congregations, and events. Feld provides the organizing principle. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/227352) |
| **Separable temporal exponential random graph model—STERGM** | Models formation and dissolution separately, with estimable structural effects | Use offline for calibration and comparison where longitudinal data permit; do not substitute its fitted coefficients for universal human rules. [OUP Academic](https://academic.oup.com/jrsssb/article/76/1/29/7075934) |

**Recommended synthesis:** actual affiliation-based encounters, Kumpula-like reinforcement and introductions, heterogeneous attention budgets, persistent kin and role structures, and event-sensitive dormancy.

Avoid making unrestricted preferential attachment the backbone of friendship formation. It provides a mechanism for accumulating connections, but TCE still needs finite attention, plausible meetings, role differences, and realistic durations. Broad degree distributions alone do not establish a scale-free mechanism. [Nature](https://www.nature.com/articles/s41467-019-08746-5.pdf)

## 5.2 Store shared pair identity but directional impressions

A practical proposed pair record contains:

```
Pair identity:
    person_a, person_b
    relation flags
    history/event reference

A's view of B:
    familiarity, affection, trust
    fear, resentment
    last relevant interaction

B's view of A:
    same directional fields
```

Keep debts, employment contracts, offices, and membership obligations in their appropriate institutional systems. A relationship record can reference them instead of duplicating their complete state.

Store genealogy separately. Compute close relatedness on demand with bounded-depth ancestry queries and use explicit social membership for larger descent groups. Do not allocate an all-pairs kinship matrix.

For frequently consulted relationships, retain richer state; for dormant acquaintances, retain compact recognition and history references. **Storage tiers must not become biological relationship limits.**

## 5.3 Use event-driven updates and lazy decay

The proposed update sequence is:

1. **Routines and institutions produce attendance and encounters.**
2. **Actors select substantive engagement within time and opportunity constraints.**
3. **Events update both participants’ directional impressions and relevant obligations.**
4. **Decay is materialized when a record is accessed; periodic maintenance handles dormancy and compaction.**

Do not update every relationship every rendered frame. Do not form a complete clique whenever 300 people attend a ceremony. Sample actual interaction groups and represent shared attendance separately.

Partition social-event production by activity or place, then apply pair updates through a controlled ownership or merge phase. This prevents two concurrent encounters from overwriting one another’s changes.

The intended computational structure is approximately **proportional to processed events**, with occasional maintenance proportional to stored relationships—not all possible pairs. This is an architectural target, not a measured performance claim.

### Memory scale

For **50,000 people** and mean undirected stored degree **80–150**:

\[
E=N\bar{k}/2=2.0\text{–}3.75\ \text{million pair records}.
\]

At an **illustrative 48 bytes per shared pair record**, raw storage is **96–180 MB**, calculated before adjacency indices, event memories, allocator overhead, genealogy, or institutional state. Larger directional representations increase this figure.

That suggests sparsity can make the relationship layer manageable, but throughput and memory must be benchmarked with the actual layout.

## 5.4 Initialize a lived-in community

Do not start adults with empty social lives.

Generate households, parentage, marriage, residence, occupations, and memberships first. Then create plausible prior activity histories or run a compressed social warm-up before the playable start.

Use shared history to initialize *different* relationships: siblings need not be equally close; an apprenticeship may create respect, resentment, or both; a neighbor may be familiar without being trusted.

For century-scale operation, archive dead-person information and consequential events separately. Most relationships between two deceased people should leave the active simulation, while genealogy, inheritance claims, feuds, and culturally preserved history can remain through compact records.

## 5.5 Borrow game architecture, not game balance constants

**Comme il Faut / Prom Week**, and its adaptation **CIF-CK** for *Skyrim*, are useful precedents for persistent social state and interactions that depend on feelings, relationship context, and longer-term mood. They offer an authoring pattern for TCE’s building blocks, not an empirically validated population-network generator. [Open Universiteit research portal](https://research.ou.nl/en/publications/prom-week-meets-skyrim-developing-a-social-agent-architecture-in-/)

**Neighborly** is an inspectable town-scale social-simulation framework explicitly designed for extensibility and content authoring, drawing lessons from *Talk of the Town*. It is relevant as an architectural reference; its documentation does not establish performance at TCE’s population scale. [PyPI](https://pypi.org/project/neighborly/0.9.0/)

The useful division of labor is: **authored social actions explain what an event means; emergent encounters determine who experiences it.**

---

# 6. Sources, datasets, and evidence limits

## 6.1 Recommended calibration portfolio

No single dataset measures the complete network TCE needs. Use several complementary sources.

| Source or dataset | Best use | Important limitation |
| --- | --- | --- |
| **Banerjee et al., *The Diffusion of Microfinance*; Karnataka village network data** | Overlap among visiting, kinship, borrowing, advice, worship, and information diffusion. Data are catalogued under DOI **10.7910/DVN/U3BIHX**. | Modern rural India; partial survey coverage and household/individual definitions require careful handling. [Dataverse](https://dataverse.harvard.edu/dataset.xhtml?persistentId=doi%3A10.7910%2FDVN%2FU3BIHX) |
| **Apicella et al., *Social Networks and Cooperation in Hunter-Gatherers*** | Small-scale network clustering, reciprocity, geography, and differences between relation types | Restricted nomination tasks; a particular population and study design, not a universal ancestral network. [Human Nature Lab](https://humannaturelab.net/sites/default/files/pdf/paper/127-Social-Networks-and-Cooperation-in-Hunter-Gatherers.pdf) |
| **Mossong et al., POLYMOD** | Daily contacts and age/setting mixing | Contemporary European contact diaries; neither complete friendship data nor direct ancient-world parameters. [PLOS](https://journals.plos.org/plosmedicine/article?id=10.1371%2Fjournal.pmed.0050074) |
| **Rohrer, Keller & Elwert classroom experiment** | Causal effect of changed proximity on friendship; accompanying data/code are identified in the paper | Children in a school setting, over one semester. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0255097) |
| **Saramäki et al., *Persistence of Social Signatures in Human Communication*** | Heterogeneity in attention allocation and stability despite contact turnover | Small life-transition cohort and a particular communication channel. [arXiv](https://arxiv.org/html/1204.5602v2) |
| **D-PLACE: Kirby et al.** | Cross-cultural residence, descent, marriage, subsistence, and other institutional inputs | Not individual friendship networks; observations have dates and societies are not statistically independent. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) |
| **Ancient pedigrees and historical community studies** | Constraints on residence, kinship, marriage, recruitment, and institutions | They reveal some relations much more directly than others; missing friendships cannot be treated as absent friendships. [Nature](https://www.nature.com/articles/s41586-023-06350-8) |

Check access conditions and reuse rights before packaging empirical records into a commercial product. Calibration against published statistics and distributing underlying personal or ethnographic data are different activities.

## 6.2 Claims to keep explicitly uncertain

**The precise Dunbar ceiling is contested.** Layered investment is more defensible than a universal number of maintainable relationships inferred from brain size. [Royal Society Publishing](https://royalsocietypublishing.org/rsbl/article/17/5/20210158/62916/Dunbar-s-number-deconstructed-Dunbar-s-number)

**A universal power-law degree distribution is not established.** Fit competing distributions and examine how data collection constrains the observed tail. [Nature](https://www.nature.com/articles/s41467-019-08746-5.pdf)

**Historical friendship decay rates are poorly constrained.** Modern emotional-closeness surveys, phone cessation, and intimate-contact turnover cannot be converted into one ancient-world half-life. [Springer](https://link.springer.com/article/10.1007/s12110-015-9242-7)

**Kinship does not uniquely determine social organization.** Cross-cultural residence patterns and contrasting archaeological cases rule out a single universal family-network template. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/walkeretal2013.pdf)

**Static similarity does not establish causal correctness.** Formation rates and durations must be checked separately; otherwise excessive tie creation can be hidden by excessive deletion while producing a plausible snapshot. [OUP Academic](https://academic.oup.com/jrsssb/article/76/1/29/7075934)

## Final design judgment

For TCE, the highest-value realism will come from **credible meeting opportunities, differentiated relationships, limited attention, and remembered history**—not from a more elaborate random-graph formula.

Build the physical and institutional opportunities first. Let relationships reinforce, compete, lapse, and reactivate through actual events. Preserve external connections and differences among kinship, affection, obligation, and exposure. Then validate the resulting networks through the same incomplete observational instruments used in the empirical studies.

That gives TCE a route to villages, towns, and cities whose social structures differ because their inhabitants live differently—not because an era switch assigned them different graphs.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927fc-b7f8-83ea-8f0f-d8014d3acf7e)
