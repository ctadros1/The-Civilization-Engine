# Technology diffusion, adoption, and loss

## A simulation-ready report for The Civilization Engine

**TCE should model technology as a maintained capability of people, workshops, and institutions—not as a permanent property of a civilization.** Hearing about a technique, performing it competently, organizing its production, and using it widely are different achievements. They should spread at different speeds and fail for different reasons.

This distinction is supported by a major finding in diffusion economics: countries’ delays in first adopting technologies have converged, while their intensity of use has diverged. A technology can therefore “arrive” without substantially transforming an economy. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fmac.20150175)

For TCE, the central causal sequence should be:

**Contact → awareness → learning and experimentation → viable production → adoption and expansion → reproduction of expertise.**

Every transition can stall. Production can stop while expertise survives; expertise can disappear while descriptions survive; and local knowledge can vanish while remaining available elsewhere. The architecture and numerical starting values recommended below are design proposals, distinguished from the historical evidence.

---

## 1. Mechanisms: rules a simulation can implement

### 1.1 Separate information, skill, production capability, and use

“Knowing a technology” is too coarse a state. At minimum, represent the following separately:

| Layer | What it means | Example |
| --- | --- | --- |
| Awareness | An agent knows that something exists and has beliefs about its usefulness | A farmer has seen an unfamiliar plough |
| Operational skill | Someone can use the product or perform a procedure | A worker can operate a loom |
| Production and maintenance expertise | Someone can manufacture, repair, or reproduce the technique | A specialist can build and tune that loom |
| Organizational capability | The necessary workers, tools, inputs, finance, and coordination can be assembled | A workshop can sustain loom production |
| Adoption and intensity | The technique is actually used, and at a measurable scale | Looms account for a growing share of textile output |
| Recoverable knowledge | Instructions, trained outsiders, artifacts, or surviving traditions offer a route to reconstruction | A closed workshop retains drawings and an experienced former worker |

This decomposition follows the distinction between information and *absorptive capacity*: prior related knowledge helps people and organizations recognize, assimilate, and apply knowledge acquired from others. Access to information is not equivalent to the ability to exploit it. [Joseph Mahoney's Home Page](https://josephmahoney.web.illinois.edu/BA545_Fall%202022/Cohen%20and%20Levinthal%20%281990%29.pdf)

**Implementation rule:** determine a settlement’s capabilities from its accessible people, facilities, records, and supply networks. Do not permanently set `civilization.knows_iron = true`.

Keep adoption decisions at the appropriate level. A household adopts a crop; a workshop adopts a production process; an irrigation association adopts a maintenance regime. Individual people supply the beliefs, decisions, labor, and expertise behind those collective actions.

### 1.2 Contact spreads opportunities, not complete capabilities

Different channels transfer different things.

**Trade.** Imported objects expose potential users to functions, styles, and performance. They may provide models for copying, but not necessarily reveal hidden production steps. Dutch merchants supplied Japanese porcelain producers with physical models for export designs, while the establishment of Japanese porcelain production also depended on skilled potters, kilns, and suitable clay. [The Metropolitan Museum of Art](https://www.metmuseum.org/ru/essays/edo-period-japanese-porcelain)

**Rule:** attach different information yields to seeing an object, watching its use, observing its manufacture, and receiving supervised instruction. Buying a steel tool should not automatically teach steelmaking.

**Migration.** Skilled migrants carry practiced routines and teaching capacity. Hornung’s study connects Huguenot settlement in Prussia following 1685 with textile-manufacturing productivity in 1802, finding substantial long-term effects of skilled immigration. The relevant transfer is embodied in people and their subsequent economic activity, not merely increased population. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.104.1.84)

**Rule:** migrants retain their actual skills. Whether those skills establish a local industry depends on employment, inputs, equipment, permission, and opportunities to train successors.

**Conquest and captured artisans.** War can relocate expertise, but it can also destroy the communities and production systems sustaining it. The movement of Korean potters to Japan during the invasions of the 1590s helped transfer kiln and ceramic expertise; Japanese porcelain production subsequently developed where appropriate resources and production arrangements existed. [The Metropolitan Museum of Art](https://www.metmuseum.org/ru/essays/edo-period-japanese-porcelain)

**Rule:** conquest moves or removes actual people and assets. Capturing artisans is not a research-point award. Their survival, treatment, cooperation, ability to communicate, and access to a functioning workshop determine the result. Account for losses at the place of origin as well as potential gains at the destination.

**Espionage.** Information quality and recipient capability matter more than a steady stream of generic intelligence. Glitz and Meyersson’s study of East German industrial espionage finds that relatively few high-quality information transfers drove much of the estimated productivity benefit, with larger effects in sectors already closer to the technological frontier. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20171732)

**Rule:** intelligence should deliver specific, sometimes incomplete instructions or observations. Related expertise determines whether recipients can interpret and test them. Most acquisitions need not produce a breakthrough.

### 1.3 Distance matters through networks

Comin, Dmitriev, and Rossi-Hansberg examine 20 technologies across 161 countries over 140 years. Their model and evidence indicate that distance from early technological leaders impedes diffusion, but that this effect weakens as additional places adopt and become sources themselves. Diffusion is therefore multicentered, not permanently radiating from one birthplace. [CEPR](https://cepr.org/voxeu/columns/heavy-technology-process-technological-diffusion-over-time-and-space)

**Rule:** use actual contact networks: household ties, workplaces, markets, itinerant specialists, ports, migration chains, and institutional correspondence. A remote port with regular shipping may receive a technique before a geographically closer but disconnected valley.

Where TCE already simulates traders and travelers, **do not apply a second generic distance penalty to the knowledge they carry**. Distance has already reduced their contact frequency or increased their travel cost.

For contacts outside the explicitly simulated network, a distance or travel-time kernel is a useful approximation:

\[
K\_{ab}=\exp(-t\_{ab}/\tau)
\]

Here \(t\_{ab}\) is generalized travel time and \(\tau\) is an authored interaction scale. Include occasional long-distance links; otherwise the simulation will produce unrealistically smooth fronts.

### 1.4 Learning requires practice, feedback, and relevant examples

Foster and Rosenzweig’s study of India’s Green Revolution finds learning effects from both farmers’ own experience and their neighbors’ experience. Experience improved the management and profitability of unfamiliar high-yielding varieties. This is more specific than simply copying an adopter’s choice. [REV. CHRIS JOHNSON FOSTER](https://adfdell.pstc.brown.edu/papers/hyv4.pdf)

**Rule:** observations should update beliefs about yield, cost, reliability, and required inputs. Practical work should improve execution. These are separate updates.

A useful minimal representation is a believed payoff plus uncertainty. An agent should discount observations from substantially different soils, climates, scales of production, or input conditions. Failure can indicate either an unsuitable technique or poor execution; agents need not diagnose that distinction correctly.

Do not make every additional adopter produce an identical positive influence. Bandiera and Rasul find an inverse-U relationship between sunflower adoption and the number of adopting family members and friends in northern Mozambique. Strategic waiting for others to learn is one possible explanation; the study does not establish one universal mechanism. [Homepages UCL](https://www.homepages.ucl.ac.uk/~uctpimr/research/tech_ej.pdf)

**Rule:** allow agents to wait for better evidence, share experimentation costs, or free-ride on others’ trials. Count independent experiences, not repeated retellings of the same success.

### 1.5 Adoption is an economic and social decision

For TCE, evaluate adoption against the incumbent technique, not against doing nothing:

\[
\Delta U =
\text{expected operating advantage}
-\text{capital, training, and switching costs}
-\text{risk premium}
-\text{expected sanctions}
+\text{status and coordination benefits}
\]

Use comparable units: either discounted lifetime utility or annualized benefits and costs. Capital affordability remains a separate constraint; positive lifetime returns do not conjure up credit.

This structure should permit several outcomes without labeling them backwardness: a technique is unsuitable locally, too risky for a vulnerable household, profitable only at larger scale, unavailable because of missing inputs, or not yet worth replacing an existing asset. The agricultural learning studies provide empirical foundations for treating information and expected returns jointly rather than assuming exposure is sufficient. [REV. CHRIS JOHNSON FOSTER](https://adfdell.pstc.brown.edu/papers/hyv4.pdf)

**Rule:** adoption opportunities occur at meaningful decision points—planting, workshop establishment, equipment replacement, rebuilding, or a major contract. A daily simulation does not require daily investment reconsideration.

### 1.6 Resistance should arise from particular interests and institutions

**Guilds are not uniformly pro- or anti-technology.** Epstein emphasizes apprenticeship, transferable skills, and institutional support for training. Ogilvie emphasizes exclusion, market privileges, and restrictions that could impede entry and competition. The balance is historically contested and varies by institution and context. [Stanford University](https://web.stanford.edu/~avner/Greif_228_2005/Epstein%201998%20Guild.pdf)

**Rule:** split a guild into functions: teaching, certification, mutual assistance, entry restrictions, production rules, and political influence. An institution can preserve expertise while obstructing outsiders or labor-saving alternatives.

Elites should oppose a technique when they expect it to weaken their income, authority, military position, or control over information—not because “elites dislike progress.” Their response can include restricting finance, licensing workshops, blocking apprenticeships, imposing penalties, or adopting the technique themselves and monopolizing it.

Religious resistance likewise needs specificity. Ottoman printing is a warning against a civilization-wide religious penalty. Dipratu’s study rejects the evidentiary basis for famous blanket early prohibitions while arguing that religious concerns remained important. Müteferrika’s first publication in 1729 displayed extensive official and religious endorsements; support and restrictions coexisted, including exclusions of specified religious subjects. [Academia](https://www.academia.edu/113734522/Ottoman_Endorsements_of_Printing_in_18th_Century_Istanbul)

**Rule:** beliefs and institutions can prohibit particular uses, demand certification, legitimize adoption, or preserve and teach knowledge. Enforcement requires resources. A prohibition should generate clandestine practice, relocation, negotiation, or noncompliance where incentives and circumstances support them—not perfect instantaneous suppression.

### 1.7 Knowledge loss is usually a broken reproduction process

Represent at least four distinct conditions:

| Condition | What remains | Appropriate recovery path |
| --- | --- | --- |
| Production suspended | Skilled people remain, but production is uneconomic or infeasible | Restore demand, inputs, facilities, or permission |
| Expertise endangered | Few practitioners remain and succession is inadequate | Recruit apprentices or attract skilled migrants |
| Local practical loss | No accessible practitioner can perform a critical step | Reconstruct from records, experiment, or import expertise |
| Unrecoverable locally | No adequate practitioner, description, or usable external connection remains | Independent rediscovery or renewed outside contact |

Henrich’s influential model shows how imperfect transmission and a shrinking pool of interacting social learners can produce losses under specified assumptions. But **the Tasmanian case is contested evidence for that mechanism**, not a validated population threshold. [Cambridge University Press](https://www.cambridge.org/core/journals/american-antiquity/article/demography-and-cultural-evolution-how-adaptive-cultural-processes-can-produce-maladaptive-lossesthe-tasmanian-case/8CD08CC61E6FACF59EC02659B2BDA43C)

Vaesen and colleagues argue that bone points are the only artifact category demonstrably abandoned in the relevant Tasmanian record, probably around 4,000 years ago, and that retained crafts and cultural practices were not uniformly simpler. They also find mixed support for population-size explanations across comparative studies. This challenges both the archaeological inference and the practice of equating artifact counts with total cultural knowledge. [TU/e Pure](https://pure.tue.nl/ws/files/19898077/00_PNAS_online.pdf)

**Rule:** model the number and connectivity of qualified teachers for particular skills. Do not delete technologies whenever total population falls below a universal cutoff.

Post-Roman Britain illustrates a different route. Fleming documents a profound contraction of Roman-style material production around the end of Roman rule, including wheel-thrown, kiln-fired pottery, alongside disruption of urban and economic organization. Her interpretation involves substantial losses of practical skills, although the timing and extent of knowledge loss cannot simply be read from the disappearance of objects. [University of Pennsylvania Press](https://www.pennpress.org/9780812252446/the-material-fall-of-roman-britain-300-525-ce/)

A suitable **TCE hypothesis to test**, rather than a scripted historical event, is:

> Large customers and distribution networks disappear → specialized production becomes unviable → workshops close → training stops → remaining practitioners leave or die → restarting becomes much harder.

The initial failure is economic and organizational; subsequent generations can turn it into a knowledge failure.

### 1.8 Texts preserve knowledge, but do not eliminate tacit requirements

Collins documents a roughly 20-year delay before Russian sapphire measurements were repeated in the West, partly attributable to missing tacit knowledge. Personal contact helped; better reporting could also have helped. The lesson is neither “books are sufficient” nor “books are useless.” [Sage Journals](https://journals.sagepub.com/doi/10.1177/030631201031001004)

**Rule:** records should preserve particular components: dimensions, ingredients, sequences, explanations, test procedures, and troubleshooting. Their usefulness depends on completeness, language, literacy, accessibility, and the reader’s related skills.

Treat oral teaching, repeated practice, and institutional routines as memory systems too. For TCE, writing should change the cost and resilience of transmission, not mark the transition from “no knowledge storage” to “knowledge storage.”

---

## 2. Parameters: evidence anchors and proposed starting values

### 2.1 Empirical anchors

These quantities measure different processes. They should **not** be pooled into a single “diffusion speed.”

Confidence below concerns both the reported result and its suitability for transfer into TCE.

| Quantity | Reported value and units | Source and interpretation | Confidence / transfer limit |
| --- | --- | --- | --- |
| National adoption lag after invention | **45 years on average** | Comin–Hobijn: modeled adoption paths for 15 technologies in 166 countries over two centuries. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.100.5.2031) | Strongly documented result; model-dependent. Not a household adoption interval or prehistoric default |
| Spatial interaction decay | An additional **1,000 km corresponds to a 73% reduction** in the modeled probability of meeting an adopter | Median structural estimate reported by Comin–Dmitriev–Rossi-Hansberg. [CEPR](https://cepr.org/voxeu/columns/heavy-technology-process-technological-diffusion-over-time-and-space) | Model-dependent historical macro estimate; poor direct portability to small ancient settlements |
| European Neolithic expansion front | **0.6–1.3 km/year**, 95% interval | Pinhasi–Fort–Ammerman, using dates from **735 sites** in Europe, Anatolia, and the Near East. [PLOS](https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.0030410) | Useful regional benchmark; not the velocity of messages, travelers, or all agricultural innovations |
| Adoption association with an additional adopting contact | **+5.4 percentage points** at the network mean; marginal relationship turns negative above roughly **10 adopting contacts** | Bandiera–Rasul’s fitted sunflower-adoption relationship in Mozambique. [Homepages UCL](https://www.homepages.ucl.ac.uk/~uctpimr/research/tech_ej.pdf) | Context-specific conditional association, not a causal per-contact transmission probability |
| Information barrier despite an introduction program | **48% of 96 nonadopters** reported not knowing production techniques | Same Mozambique study; respondents could give multiple reasons. [Homepages UCL](https://www.homepages.ucl.ac.uk/~uctpimr/research/tech_ej.pdf) | Useful distinction between general exposure and practical knowledge; small, specific sample |
| Espionage contribution | Without espionage, the East/West German productivity ratio was estimated to be **13.3% lower** at the Cold War’s end | Glitz–Meyersson counterfactual using information flows from 1970–1989. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20171732) | Structural/counterfactual result—not a universal productivity bonus for spies |
| Tacit transfer delay | Approximately **20 years** | Collins’s sapphire-replication case. [Sage Journals](https://journals.sagepub.com/doi/10.1177/030631201031001004) | Well-defined case; no claim that this is a typical learning duration |

The reviewed evidence does **not** establish a universal annual forgetting probability, minimum population needed to retain technology, or apprenticeship duration applicable across crafts. Those require technology-specific authoring and sensitivity tests.

### 2.2 S-curves: useful diagnostics, unsuitable scripts

The Bass model represents cumulative first adoption in a fixed potential market:

\[
\frac{dF}{dt}=(p+qF)(1-F)
\]

Here \(F\) is the fraction that has adopted, \(p\) represents external influence, and \(q\) represents influence associated with existing adopters. Both have units of inverse time. Bass’s model is an important aggregate benchmark, not a complete account of expertise, institutional resistance, or loss. [Of (im)possible interest](https://pdodds.w3.uvm.edu/files/papers/others/1969/bass1969a.pdf)

For a logistic curve,

\[
\frac{dF}{dt}=rF(1-F),
\qquad
T\_{10\rightarrow90}=\frac{4.394}{r}.
\]

Thus, **as a mathematical illustration**, \(r=0.05\)–\(0.5\ \text{year}^{-1}\) gives approximately **88–8.8 years** between 10% and 90% adoption. These are derived values, not estimated historical defaults.

In TCE, fit these curves to outputs. Do not force agents to follow them. Heterogeneous costs and thresholds can generate similar aggregate shapes; an S-curve alone cannot identify imitation as the cause.

Also track active use separately from cumulative first adoption. Abandonment, substitution, and changing eligibility can make use fall or plateau even though cumulative “ever adopted” never decreases.

### 2.3 Initial implementation priors

**Every numerical value in this table is a proposed authoring or stress-test range, not an empirical estimate.** They are starting points for experiments, with low historical confidence until calibrated to a particular technology.

| Parameter | Proposed starting range | Units and implementation |
| --- | --- | --- |
| Adoption reconsideration | **1–2/year** for seasonal farm decisions; **4–12/year** for workshop opportunities | Reviews per year; replace with actual planting or investment events where possible |
| Effective instruction/practice requirement | **20–200 hours** for a narrow, observable procedure; **500–5,000 hours** for a specialized process stage | Learner-hours; full crafts can require several stages. Do not equate these with legal apprenticeship terms |
| Simultaneous apprentices | **1–3** per teacher | People; teaching consumes actual time and can reduce production |
| Reliability of borrowed local payoff evidence | **0.1–1.0** relative to a comparable own trial | Dimensionless weight, reduced by environmental mismatch, poor observation, or low trust |
| Coverage of a technical record | Test **0, 0.5, and 1.0** of explicitly authored critical steps | Fraction of documented steps—not a direct probability of successful reconstruction |
| Loss of unused execution proficiency | Start at **zero automatic erasure**; stress-test decay rates of **0.02 and 0.1/year** for performance only | Preserve awareness and conceptual knowledge separately; mortality and failed succession remain distinct |
| Practitioner replacement | Test expected successor counts of **0.5, 1, and 2** per practitioner | Dimensionless diagnostic derived from training activity, not a population-size gate |

The most important authoring task is assigning **relative difficulty and dependence on instruction** across techniques. An easily observed construction detail should not have the same transmission requirements as diagnosing a concealed process failure.

---

## 3. Variation across eras and regions

The following are analytical contexts, not technology-era gates for TCE. Different transmission systems can coexist in the same world.

| Context | Evidence and characteristic constraints | TCE emphasis |
| --- | --- | --- |
| **Foragers and small-scale societies** | The Tasmanian debate demonstrates the danger of treating preserved artifacts as the entire knowledge system. Oceania research also distinguishes population from intergroup contact; Kline–Boyd’s supportive comparison involves only ten island societies. [TU/e Pure](https://pure.tue.nl/ws/files/19898077/00_PNAS_online.pdf) | Kinship, teaching relationships, mobility, and cross-community access. Measure specialist redundancy rather than assigning a cultural-complexity score |
| **Early farming expansion** | The Near East–Europe evidence supports a broad, slow agricultural expansion front, but this combines demographic movement and establishment of viable farming communities—not just communication. [PLOS](https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.0030410) | Move crops, livestock, skills, and households separately. Ecological suitability and establishment costs filter adoption |
| **Preindustrial specialization** | European guilds could organize training and restrict access. East Asian ceramics show transfers involving artisans, kilns, resources, patronage, and export demand. [Stanford University](https://web.stanford.edu/~avner/Greif_228_2005/Epstein%201998%20Guild.pdf) | Workshops, craft lineages, merchants, courts, and religious institutions become important knowledge holders and gatekeepers |
| **Industrialization** | Cross-country adoption delays were substantial and heterogeneous; newer technologies in Comin–Hobijn’s sample arrived more quickly. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.100.5.2031) | Larger capital commitments, complementary infrastructure, specialized workers, and equipment-replacement decisions |
| **Modern systems** | Arrival can become widespread without convergence in intensity. Advanced scientific work can still require personal transfer of practical know-how. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fmac.20150175) | Rapid information access alongside expensive implementation, distributed expertise, organizational learning, and maintenance bottlenecks |

Several regional comparisons are especially useful.

**India and Mozambique:** both show why farming knowledge should remain local and experiential even when improved techniques are publicly promoted. Their studies concern different crops, institutions, and informational environments; one country’s regression coefficient should not become a universal “farmer imitation” parameter. [REV. CHRIS JOHNSON FOSTER](https://adfdell.pstc.brown.edu/papers/hyv4.pdf)

**China–Korea–Japan:** ceramic diffusion is better represented as the assembly of transferred skills, local resources, and demand than as one state receiving a complete national technology. [The Metropolitan Museum of Art](https://www.metmuseum.org/ru/essays/edo-period-japanese-porcelain)

**Ottoman lands:** endorsement, restrictions on particular categories of publication, and institutional sponsorship belong in the model simultaneously. An invariant religious resistance coefficient would erase the historical mechanism. [Academia](https://www.academia.edu/113734522/Ottoman_Endorsements_of_Printing_in_18th_Century_Istanbul)

**Indigenous North America:** archaeological and biomolecular research places horses in the northern Rockies earlier than older written-source narratives suggested; an Idaho specimen dates to the 1630s. Indigenous exchange networks were active agents of dispersal, not passive endpoints awaiting direct European contact. [Idaho State University](https://www.isu.edu/imnh/imnh-collections-and-research/research-rewrites-history-of-the-horse-in-idaho.html)

**Oceania:** relationships between population, connectivity, and technical repertoires remain informative but contested. Moreover, laboratory research by Derex and Boyd finds that partial connectivity can preserve alternative approaches and support subsequent recombination better than full connectivity. Faster homogenization is therefore not necessarily better long-run innovation. That laboratory result is a modeling hypothesis to test, not a measured universal historical advantage. [PubMed](https://pubmed.ncbi.nlm.nih.gov/20392733/)

---

## 4. Stylized facts and validation targets

A credible simulation should reproduce a **family of outcomes**, not one perfect S-curve.

| Pattern to reproduce | Validation test | Evidence anchor |
| --- | --- | --- |
| **Arrival and scale are distinct** | Compare first feasible local use, adopter share, and output per eligible person. Some places should acquire a technique yet use little of it | Convergence in arrival lags can coexist with divergence in intensity. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fmac.20150175) |
| **Diffusion has multiple centers** | Early proximity matters, but secondary hubs progressively reduce dependence on the original inventor | Spatial diffusion evidence. [CEPR](https://cepr.org/voxeu/columns/heavy-technology-process-technological-diffusion-over-time-and-space) |
| **Long delays and rapid takeoffs coexist** | Track distributions of arrival lags and 10–90% adoption intervals, rather than imposing one duration | The 45-year national average conceals substantial technological and country variation. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.100.5.2031) |
| **Exposure does not guarantee adoption** | After demonstrations, some eligible agents should still lack competence, confidence, resources, or expected benefits | The Mozambique study separates awareness campaigns from knowledge of production. [Homepages UCL](https://www.homepages.ucl.ac.uk/~uctpimr/research/tech_ej.pdf) |
| **Expert migration can have lasting effects** | Establish a migrant-led workshop, then remove its founders after successors mature. Effects should persist when institutions reproduce expertise | Huguenot textile evidence extends into the following century. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.104.1.84) |
| **Production can collapse before knowledge disappears** | Remove major customers or inputs while retaining workers. Output should stop first; expertise should become endangered later | Post-Roman Britain motivates this mechanism, without fixing its exact timetable. [University of Pennsylvania Press](https://www.pennpress.org/9780812252446/the-material-fall-of-roman-britain-300-525-ce/) |
| **Institutional effects can change sign** | The same guild may increase trained-worker supply while restricting competing establishments | Competing guild interpretations. [Stanford University](https://web.stanford.edu/~avner/Greif_228_2005/Epstein%201998%20Guild.pdf) |
| **Connectivity has more than one consequence** | Compare diffusion speed, diversity of variants, and resilience—not merely average expertise | Derex–Boyd’s experimental result. [Arizona State University](https://asu.elsevierpure.com/en/publications/partial-connectivity-increases-cultural-accumulation-within-group/) |

For controlled TCE tests, compare worlds with identical population and resources but different networks. Then vary teacher access, apprentice completion, demand, credit, prohibitions, archive survival, and migration **one mechanism at a time**.

Crucially, a successful recovery test should identify *why* recovery occurred. Reopening trade, restoring a workshop, retrieving a text, and importing a skilled person should not all function as the same generic research boost.

---

## 5. Modeling recommendation for TCE

### 5.1 A small number of explicit state types

Use authored technology nodes, but make operational knowledge more granular than the displayed node.

```
TechnologyDefinition
    alternative prerequisite sets
    recipes and critical process stages
    required skills, tools, materials, and environment
    observability and documentation properties
    substitutes and locally variable performance

PersonSkill
    technique/stage identifier
    execution proficiency
    teaching capability
    relevant practice and instruction history

WorkshopCapability
    qualified people by required role
    equipment, inputs, and operating scale
    apprentices and scheduled teaching
    current production and failure causes

KnowledgeRecord
    covered steps and instructions
    language, accessibility, and physical condition
    provenance and demonstrated reliability
```

A public technology node can therefore be “known locally” while displaying **no current production**, **one remaining teacher**, or **missing furnace construction expertise**.

Do not average away bottlenecks. Five partly trained workers do not necessarily substitute for one person competent in a critical diagnostic step. Conversely, a team can collectively possess a process that no single individual understands completely.

### 5.2 Use hazards for exposure, decisions for investment

A lightweight awareness process is:

\[
\lambda^{A}\_{ik}
=
\sum\_{j\in C\_i} f\_{ij}\,v\_{jk}\,c\_{ij}\,a\_{jk}
+
m\_{ik}.
\]

Here \(f\) is contact frequency, \(v\) is the visibility of relevant knowledge, \(c\) is communication effectiveness, \(a\) indicates whether the contact can supply awareness, and \(m\) represents mediated sources such as instruction or records.

Convert rates into timestep probabilities consistently:

\[
P(\text{event during }\Delta t)=1-e^{-\lambda\Delta t}.
\]

This avoids making diffusion speed accidentally depend on the simulation timestep.

For actual adoption, use the household or organization’s decision process. A simple probabilistic implementation is:

\[
\lambda^{\text{trial}}\_{ik}
=
\nu\_i\,G\_{ik}\,
\sigma\!\left(\frac{\Delta U\_{ik}}{s\_i}\right),
\]

where \(\nu\_i\) is the frequency of genuine opportunities, \(G\_{ik}\) is a feasibility indicator, \(\sigma\) is a logistic choice function, and \(s\_i\) controls decision noise or heterogeneity.

Feasibility for a trial can be weaker than feasibility for commercial production: an agent may experiment with small quantities before financing a workshop. Learning then follows actual instruction, practice, and observed outcomes—not elapsed time alone.

These equations are recommended engineering approximations, not fitted historical laws.

### 5.3 Make institutional continuity measurable

For a particular skill, monitor expected replacement:

\[
R^{\text{succ}}\_k=b\_kT\_ks\_k,
\]

with \(b\_k\) = apprentices started per teaching year, \(T\_k\) = expected remaining teaching years, and \(s\_k\) = probability of completion **and retention in the relevant network**.

This is a branching-process-style diagnostic. Values below one indicate a succession problem under the simplified assumptions; values above one do not guarantee survival.

Also track correlated vulnerability. Five teachers in one workshop are less resilient to a single fire or expulsion than five accessible teachers in independent establishments.

As a purely illustrative calculation, if each practitioner independently had a 20% chance of being lost during a specified interval, losing all five would have probability \(0.2^5=0.032\%\). Independence is the crucial assumption; shared disasters can invalidate that apparent protection.

### 5.4 Distinguish local loss from global loss

A settlement should query its **accessible knowledge network**, not just residents. Nearby instructors, seasonal visitors, foreign schools, and interpretable records can preserve a recovery route.

Political borders should affect access through actual restrictions and relationships. Annexation should not merge all expertise instantly, and independence should not erase knowledge acquired earlier.

Maintain separate flags for:

* no current production;
* no local qualified practitioner;
* no accessible qualified practitioner;
* no adequate known reconstruction route.

These distinctions give migration, diplomacy, archives, and institutional rebuilding different roles.

### 5.5 Scale through sparse, event-driven updates

For 10k–50k people, avoid evaluating every person against every technology every day.

Maintain sparse personal skills and awareness records, workshop rosters, and reverse indexes from each technique to practitioners and institutions. Update work-related proficiency through production events; evaluate major adoption seasonally or when opportunities arise; audit endangered expertise when practitioners die, migrate, retire, or stop teaching.

For illustration, 50,000 agents with eight compact skill entries each produce 400,000 entries. At 16–32 bytes per entry, that is approximately **6.4–12.8 MB of payload**, before indexes and container overhead. This is a storage calculation, not a performance benchmark.

Population scaling needs particular care. If an agent represents several people, applying individual mortality directly to the sole represented specialist can create artificial extinction. If every agent is literally one person, the small population should genuinely constrain the number of simultaneous specialist occupations—but not through an arbitrary technological ceiling.

### 5.6 What to simplify in the first version

Preserve people, apprenticeship, workshop feasibility, local payoff learning, institutional restrictions, and distinct loss states.

Simplify records into coverage of authored process steps rather than simulated prose. Use a few confidence levels rather than full belief distributions. Represent organizational know-how through role coverage and accumulated workshop experience rather than detailed team cognition. Model institutional policies explicitly, but their negotiations can initially use the broader political system’s existing decision rules.

A hypothetical ceramic-production sequence shows the intended result:

> Traders bring unfamiliar vessels. Local potters become interested but cannot reproduce the firing process. A migrant specialist establishes a workshop with a patron’s support and trains apprentices. A later fuel shortage closes production without immediately deleting expertise. If teaching continues, recovery remains straightforward; if the practitioners disperse and incomplete notes are all that remain, restarting requires reconstruction or renewed contact.

No era transition or scripted historical event is needed.

### 5.7 Existing models worth borrowing

**Bass (1969):** use as an aggregate adoption benchmark, not the agent decision rule or a loss model. [Of (im)possible interest](https://pdodds.w3.uvm.edu/files/papers/others/1969/bass1969a.pdf)

**Foster–Rosenzweig (1995):** borrow learning from own and neighboring experience, with economic returns improving as execution improves. This is especially suitable for TCE’s agricultural base. [REV. CHRIS JOHNSON FOSTER](https://adfdell.pstc.brown.edu/papers/hyv4.pdf)

**Comin–Dmitriev–Rossi-Hansberg (2012):** borrow spatial transmission through multiple emerging sources. Recalibrate its macro-scale relationships rather than inserting country-level coefficients into individual encounters. [CEPR](https://cepr.org/voxeu/columns/heavy-technology-process-technological-diffusion-over-time-and-space)

**Henrich (2004):** borrow imperfect transmission and limited access to skilled models as an optional loss mechanism. Do not treat its Tasmanian interpretation as settled validation. [Cambridge University Press](https://www.cambridge.org/core/journals/american-antiquity/article/demography-and-cultural-evolution-how-adaptive-cultural-processes-can-produce-maladaptive-lossesthe-tasmanian-case/8CD08CC61E6FACF59EC02659B2BDA43C)

**Derex–Boyd (2016):** use to design experiments on connectivity, diversity, and recombination. Keep the distinction between faster dissemination and greater eventual innovation. [Arizona State University](https://asu.elsevierpure.com/en/publications/partial-connectivity-increases-cultural-accumulation-within-group/)

A combination of these mechanisms fits TCE better than a single borrowed diffusion system.

---

## 6. Sources, datasets, and limits of the evidence

### Priority calibration resources

| Resource | Coverage and best use | Important limitation |
| --- | --- | --- |
| **Comin & Hobijn, CHAT dataset (2009)** | Published compilation covering over 100 technologies, over 150 countries, and observations since 1800; useful for heterogeneous national adoption and use trajectories. [National Bureau of Economic Research](https://www.nber.org/papers/w15319) | Unbalanced coverage; physical-use measures differ across technologies. Not a preindustrial teaching dataset |
| **Comin & Hobijn (2010), AER, “An Exploration of Technology Diffusion”** | National adoption-lag estimation; article includes a replication package. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.100.5.2031) | Inferred adoption timing depends on the model |
| **Comin & Mestieri (2018), AEJ: Macroeconomics** | Separate arrival from intensity; replication package available with the article. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fmac.20150175) | National averages conceal within-country networks and organizations |
| **Pinhasi, Fort & Ammerman (2005), PLOS Biology** | Radiocarbon-based reconstruction of an agricultural expansion front. [PLOS](https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.0030410) | Earliest surviving evidence is not an exact invention or adoption timestamp |
| **Banerjee et al. (2013), Science, “The Diffusion of Microfinance”** | Network-based diffusion study and replication materials; useful for separating dissemination from participation. [MIT Economics](https://economics.mit.edu/sites/default/files/2022-08/science.1236498.pdf) | Adoption of a financial institution is an analogy, not a direct estimate of craft-skill transmission |
| **Bandiera & Rasul (2006), Economic Journal** | Individual adoption, social ties, and reported barriers in a specific agricultural introduction. [Homepages UCL](https://www.homepages.ucl.ac.uk/~uctpimr/research/tech_ej.pdf) | Social similarity and shared circumstances complicate causal interpretation |

The historical case literature should complement, not be flattened into, those datasets: **Epstein and Ogilvie** for guilds; **Hornung** for skilled migration; **Dipratu** for Ottoman printing; **Fleming** for post-Roman material production; **Henrich and Vaesen et al.** for the demographic-loss debate; and **Collins** for tacit knowledge. Their roles and disagreements are identified above.

### Evidence cautions that should shape validation

**Observed objects are not direct readings of people’s minds.** Imports can appear without local manufacturing expertise. Conversely, the absence of surviving products need not prove that all relevant knowledge disappeared.

**Aggregate curves do not identify a unique mechanism.** Different combinations of learning, affordability, heterogeneity, infrastructure, and institutional constraints can produce similar trajectories. Validate against intermediate states—teachers, workshops, trials, inputs, and adoption reasons—not only final penetration.

**Resistance is not synonymous with error.** TCE should allow rational delay, distributional conflict, genuine value disagreements, and locally unsuitable techniques alongside mistaken beliefs and coercive exclusion.

**Loss claims need a specified object.** What disappeared: output, a manufacturing stage, trained practitioners, an institution, a written tradition, or the ability to recover the technique? Those are different historical propositions and different simulation events.

**The strongest design principle is therefore continuity of reproduction:** technologies persist when people and institutions continue to make them usable and teachable. In TCE, discovery should open a possibility; a functioning social and economic system must turn that possibility into a durable capability.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9284e-1934-83e9-a834-ab0728a91ad3)
