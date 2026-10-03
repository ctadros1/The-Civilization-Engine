# How discoveries happen: a simulation-ready model for TCE

## Executive recommendation

**Model invention as local, resource-consuming search by people who possess different knowledge—not as the accumulation of a civilization-wide research currency.** Model adoption, transmission, and retention separately from invention.

For TCE, the central unit should be a **practice or capability**: a method of selecting seed, preparing fuel, lining a furnace, organizing accounts, measuring angles, or constructing a joint. People encounter problems through work, modify familiar procedures, observe unexpected results, and combine knowledge acquired from others. Institutions change who can attempt these activities, what resources they receive, and whether their results circulate.

The research supports this distributed view, but does **not** establish a universal relationship such as “twice the population produces twice the technological progress.” Recombination, access to knowledgeable people, incentives, and transmission all matter; their importance depends on the task and institutional setting. [MICHAEL MUTHUKRISHNA](https://www.michael.muthukrishna.com/wp-content/uploads/2016/08/muthukrishna_henrich_2016.pdf)

The resulting architecture should distinguish:

> **Something is physically possible → someone conceives an approach → someone makes it work → others can reproduce it → some users adopt it → the capability survives.**

These are different events. A society can know about a technique without being able to reproduce it, reproduce it without finding it worthwhile, or lose the ability while preserving descriptions of it.

---

# 1. Mechanisms: causal processes to implement

## 1.1 Innovation searches a structured space, not a list of historical milestones

The combinatorial account treats inventions as arrangements of existing capabilities, occasionally supplemented by a new capability. Youn and colleagues’ analysis of US patents supports the importance of recombination, while also showing that invention is not a uniform random search through all possible combinations. Some combinations are repeatedly refined; most conceivable combinations are never used. Patent classifications are an imperfect proxy for the underlying technologies. [arXiv](https://arxiv.org/html/1406.2938v1)

**TCE rule:** Represent technology as a **typed capability graph with multi-input recipes**, rather than a single prerequisite tree.

A recipe should specify:

| Component | Example |
| --- | --- |
| Material requirements | Suitable clay, fuel, ore, timber, fiber |
| Operations | Crush, heat, reduce, join, rotate, ferment, measure |
| Performance requirements | Temperature, purity, strength, precision, pressure |
| Knowledge requirements | Recognize a useful material; execute a sequence reliably |
| Alternative routes | Different fuels, tools, materials, or construction methods |
| Observable outcomes | Output quality, failure rate, labor requirement, durability |

Use **AND requirements** where several capabilities are genuinely necessary, and **OR alternatives** where different technical routes can accomplish the same function.

Do not require a historically named technology merely because it happened earlier in one historical sequence. Prerequisites should express material or informational dependencies, not a disguised chronology.

The **adjacent possible** is then the set of changes an agent can plausibly consider using accessible capabilities. It is local: a technique known in a distant city is not automatically available to a village craftsperson.

A useful candidate generator has three modes:

* **Variation:** Change a parameter or component of a familiar procedure.
* **Recombination:** Apply a known operation, material, or organizational arrangement in a different context.
* **Observation:** Notice an unexpected result during ordinary work or a deliberate experiment.

These are proposed implementation categories, not empirically established universal percentages.

## 1.2 Separate invention from successful engineering and economic innovation

A promising observation is not yet a reliable production process. Penicillin provides a well-documented example: Fleming’s discovery occurred in **1928**, Oxford clinical use followed in **1941**, and wartime industrial production required additional work on organisms, fermentation, purification, and manufacturing coordination. [American Chemical Society](https://www.acs.org/education/whatischemistry/landmarks/flemingpenicillin.html)

**TCE rule:** Give each practice several independent state variables rather than one “discovered” flag:

| Dimension | Meaning |
| --- | --- |
| Awareness | The agent has heard of the possibility |
| Procedural knowledge | The agent knows an approximate method |
| Execution skill | The agent can perform the necessary operations |
| Evidence | The agent has observed successes and failures |
| Reproducibility | The method works repeatedly under specified conditions |
| Adoption | A household, workshop, or institution actually uses it |
| Dissemination | Other people have acquired some of the knowledge |

This permits believable intermediate states: rumors of superior metal, an unreliable prototype, a usable process kept secret, a published recipe that local craftspeople cannot execute, or an abandoned invention whose economics later improve.

**An explanation and a working practice should also be separate.** In TCE, an agent may execute an effective process while holding an incorrect explanation of why it works. Conversely, understanding a principle should not automatically confer manufacturing competence.

## 1.3 Multiple discovery emerges from shared antecedents

Independent discovery is compatible with individual creativity. Several people can encounter similar problems while possessing similar tools and background knowledge. Historical scholarship on simultaneous invention documents this pattern, but such collections are selected examples—not an unbiased estimate of how frequently all inventions are independently duplicated. [OUP Academic](https://academic.oup.com/psq/article-abstract/37/1/83/7258140)

**TCE rule:** Never remove an invention opportunity from other communities merely because a world-first has occurred.

Maintain separate records for:

* First successful occurrence anywhere.
* First local occurrence.
* Independent invention.
* Learned reproduction.
* Imported technique adapted to local conditions.

Shared prerequisite changes should raise several communities’ discovery hazards at once. This produces clustered discoveries without scheduling them.

Keep **actual contributions and public credit** separate. An apprentice can solve a practical problem while a master, ruler, or institution receives recognition.

## 1.4 Population matters through effective participation and access

Kremer’s influential model links population and technological change: more people can generate more ideas, while improved technology supports more people. Its long-run argument concerns a feedback between demography and technology, not a directly estimated bonus that can simply be attached to every additional simulated citizen. [OUP Academic](https://academic.oup.com/qje/article-abstract/108/3/681/1881850)

Laboratory experiments have found advantages for larger groups in maintaining or improving complex cultural tasks. However, archaeological applications of demographic explanations remain contested: population size, connectivity, environmental demands, and the assumptions of transmission models are difficult to disentangle. [Nature](https://www.nature.com/articles/nature12774)

**TCE rule:** Let population affect invention through identifiable channels:

1. More people performing relevant work.
2. More specialists and complementary skills.
3. More potential teachers and replacements for lost experts.
4. Larger markets that can justify development costs.
5. Greater—or sometimes more restricted—access to other people’s knowledge.

A population of 20,000 does not supply 20,000 inventors for every domain. A metallurgical problem may have only three suitably equipped workshops and a handful of qualified practitioners.

**Avoid double-counting population.** Once invention opportunities are summed across actual qualified people or projects, do not multiply the result by population again.

## 1.5 Connectivity helps transmission but can reduce search diversity

Derex and Boyd’s experiment found that partially connected groups could develop complex solutions that fully connected groups did not. Rapid access to successful solutions encouraged convergence, while partial separation preserved different approaches that could later be combined. This is evidence for a mechanism, not a universal claim that isolation is optimal. [Arizona State University](https://asu.elsevierpure.com/en/publications/partial-connectivity-increases-cultural-accumulation-within-group/)

**TCE rule:** Use several overlapping networks:

* Household and kinship ties.
* Workshop and apprenticeship ties.
* Markets, travel, and migration.
* Institutional membership.
* Documents and correspondence.

Local clusters preserve specialized practices; occasional connections introduce unfamiliar approaches.

Do not implement information exchange as “all citizens share all discoveries.” Transmission should require contact, attention, permission, comprehension, and sometimes supervised practice.

Also distinguish **the movement of an artifact from the movement of competence**. A merchant can bring an unfamiliar object to town without bringing someone capable of making it.

## 1.6 Surplus enables experimentation, but incentives determine its direction

Historical apprenticeship models emphasize that productive knowledge depends on costly learning and access to skilled masters. Work on research funding likewise finds that incentives and tolerance for early failure can change the novelty of research pursued. These findings support modeling the allocation of time and risk, rather than treating knowledge production as free. [Northwestern Faculty](https://faculty.wcas.northwestern.edu/mdo738/research/delaCroix_Doepke_Mokyr_QJE_2018.pdf)

**TCE rule:** Experimentation must consume actual resources:

* Working time displaced from current production.
* Materials and fuel.
* Access to tools, buildings, and test sites.
* Teaching or collaboration time.
* A financial or subsistence buffer.
* Exposure to failure, injury, reputational loss, or punishment.

Need should influence **which problems attract attention**, not automatically increase the probability of solving them. Hunger may motivate a search for new food sources while simultaneously eliminating the time and materials available for experimentation.

Specialization should improve repeated observation and execution within a domain, but create dependence on suppliers and other specialists.

Rewards should include profit, survival, prestige, ritual value, military advantage, administrative control, and curiosity—not just aggregate productivity.

## 1.7 Institutions alter opportunities, incentives, and circulation

Institutional effects are conditional. Guilds could organize training while also restricting entry; secrecy and patents were alternative strategies whose usefulness differed by industry. Research on guilds explicitly disagrees about their balance of benefits and costs. [OUP Academic](https://academic.oup.com/qje/article-abstract/133/1/1/3950283)

**Recommended institutional implementation:**

| Institution | Implementable mechanisms | Important counter-effect |
| --- | --- | --- |
| **Patronage** | A patron funds people, tools, and trials; selects topics; grants a funding horizon; rewards useful or prestigious results | Dependence on one patron can redirect work, suppress inconvenient findings, or destroy a program when support ends |
| **Guild or craft association** | Training contracts, quality standards, shared facilities, mutual assistance, entry restrictions, and control of practice | Better internal transmission can coexist with exclusion of outsiders and restrictions on competing methods |
| **Secrecy** | Restrict demonstrations, documents, apprentices, and employee mobility; permit leakage and reverse engineering | Protects returns but reduces wider recombination and makes knowledge vulnerable to the loss of a few holders |
| **Patents or exclusive privileges** | Temporary rights, disclosure obligations, licensing, enforcement costs, and disputes | Potentially increases appropriability while raising access costs or obstructing follow-on work |
| **Academies, schools, and laboratories** | Maintain archives, instruments, specialist roles, teaching, tests, and reputational rewards | Can concentrate resources around prestigious methods or incumbent authorities |
| **Censorship and repression** | Topic-specific sanctions on teaching, publication, travel, or institutional membership | Encourages concealment, relocation, and selective adoption rather than erasing all practical knowledge |

Historical work on Ottoman printing illustrates how political legitimacy can affect the acceptance of particular technologies. It should not be reduced to a blanket rule that a religion or civilization rejects innovation. [Chapman University Digital Commons](https://digitalcommons.chapman.edu/economics_articles/108/)

These institutions should themselves arise through agent decisions. A master trains an apprentice when expected benefits justify the cost; patrons finance projects that serve their objectives; organized producers seek privileges when they can coordinate politically. None needs an era trigger.

## 1.8 Adoption is a separate selection process

A technique can be known and technically feasible without being worthwhile locally. Allen’s study of the spinning jenny in Britain, France, and India argues that relative wages and capital costs affected its profitability. The broader high-wage explanation of industrialization is contested, but the underlying modeling lesson is valuable: the same machine can have different economic value in different settings. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/industrial-revolution-in-miniature-the-spinning-jenny-in-britain-france-and-india/C635E725AAFC87D7F59030A2F478D886)

**TCE rule:** Adoption should depend on perceived—not omniscient—net benefit:

\[
\text{Perceived value}
=
\text{expected future gains}
-
\text{setup costs}
-
\text{learning costs}
-
\text{risk costs}.
\]

Adoption additionally requires liquidity, access to inputs, legal permission, and any necessary infrastructure.

Agents update beliefs through their own trials, demonstrations, trusted contacts, and observed users. Permit rational non-adoption, failed adoption, adaptation, and later abandonment.

Knowledge retention should then depend on **living practitioners, active teaching, usable records, and maintained equipment**. A society’s technological capabilities need not be monotonic.

---

# 2. Parameters and rates

## 2.1 Empirical anchors—not universal constants

**Confidence below refers to the usefulness of the observation for calibration.** A precisely reported result can still have low transferability to another period, region, or technological domain.

| Quantity | Value or range, with units | Evidence and confidence | Appropriate TCE use |
| --- | --- | --- | --- |
| New combinations among patent classifications | Approximately **60% new combinations / 40% reuse of existing combinations**, in the study’s 2014 analysis | Youn et al.; US patent records, 1790–2010. **Medium:** classification-based proxy; figures refer to that analysis. [arXiv](https://arxiv.org/html/1406.2938v1) | Check that refinement and recombination coexist. **Not** a universal invention probability |
| Group size in a cultural-complexity experiment | Groups of **2, 4, 8, and 16 people** | Derex et al. 2013. **Medium:** controlled experimental evidence, limited task scope. [OSF Files](https://files.osf.io/v1/resources/86erf/providers/osfstorage/57ee13b6594d9001f491ad47?action=download&direct=&version=1) | Test effects of access to multiple demonstrators; do not infer a civilization-scale elasticity |
| Crop domestication timescale | **Several centuries to millennia** for many domestication traits | Fuller et al. 2014. **Medium–high:** archaeological trait trajectories, uneven preservation and sampling. [PubMed](https://pubmed.ncbi.nlm.nih.gov/24753577/) | Model many generations of cultivation and selection, not a single “agriculture discovered” event |
| European Neolithic expansion speed | Approximately **0.6–1.3 km/year** | Pinhasi, Fort, and Ammerman 2005. **Medium:** reconstructed spatial front. [PLOS](https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.0030410) | Benchmark coupled migration, settlement, and practice transmission—not pure information speed |
| Early modern English apprenticeship requirement | **7 years** stipulated; many apprentices did not complete that period | Historical evidence discussed by de la Croix, Doepke, and Mokyr. **High** for the institution; **low** as a universal learning duration. [Northwestern Faculty](https://faculty.wcas.northwestern.edu/mdo738/research/delaCroix_Doepke_Mokyr_QJE_2018.pdf) | Separate contractual duration from achieved competence |
| Innovations without patents | **89%** of British exhibits at the **1851** world’s fair were unpatented | Moser 2012; broader dataset exceeds 8,000 British and American exhibits. **High** for the sample; fairs are selective. [RCNi Company Limited](https://www.journals.uchicago.edu/toc/jle/2012/55/1) | Patent counts must not stand in for all innovation |
| International technology-adoption lag | Mean **45 years after invention**, across **15 technologies and 166 countries** | Comin and Hobijn 2010. **Medium–high:** model-estimated adoption timing with substantial variation. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.100.5.2031) | Calibrate heterogeneous diffusion delays. This is **not** time until every household adopts |
| Penicillin discovery-to-use sequence | **1928 → 1941 clinical use → 1944 large-scale wartime supply**: roughly **13 and 16 years** | ACS/RSC historical account. **High** for chronology; one unusually well-documented case. [American Chemical Society](https://www.acs.org/education/whatischemistry/landmarks/flemingpenicillin.html) | Distinguish observation, therapeutic development, and production scale-up |
| Knowledge migration and host-country invention | Approximately **31% higher US patenting** in chemical fields associated with German Jewish émigrés, relative to comparison fields | Moser, Voena, and Waldinger 2014. **Medium–high:** context-specific quasi-experimental evidence. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.104.10.3222) | Migration can transfer networks and expertise, not merely labor |
| Local effects of higher-education institutions | **62% more patents/year** after college establishment relative to runner-up counties; only **12%** of local patents attributed to alumni or faculty | Andrews 2023. **Medium–high:** specific historical comparison design. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fpol.20200320) | Institutional effects include attraction and wider local interactions, not only direct researcher output |
| Rising effort at a technological frontier | More than **18-fold** increase in effective semiconductor research effort relative to the early 1970s to sustain Moore’s-law progress | Bloom et al. 2020. **Medium:** dependent on measurement and domain. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20180338) | Some frontiers become more demanding; do not impose a universal decline in inventiveness |

The last point remains actively disputed at the aggregate level. A **2026 working paper** by Fort and colleagues finds flat or rising patent-based research productivity under alternative firm-level measures. It is not yet equivalent to a settled consensus, and it does not directly negate every domain-specific result in Bloom et al. **TCE should allow heterogeneous research difficulty, not hard-code a universal “ideas get harder” law.** [CEPR](https://cepr.org/publications/dp21555)

## 2.2 How often should major discoveries occur?

**There is no defensible universal preindustrial rate in the evidence assembled here of “one major discovery per X people-years.”**

Three measurement problems matter.

First, archaeological dates usually identify surviving evidence, not the first conception of an idea. Second, “an invention” can mean a minor process improvement or an entire technological system. Third, preserved and patented innovations are a selected subset of actual innovation. The patent and archaeological studies above illustrate these different observation processes. [arXiv](https://arxiv.org/html/1406.2938v1)

For TCE, record four distinct rates:

| Rate | Denominator |
| --- | --- |
| Experimental activity | Qualified experiment-hours per year |
| Local technical improvement | Improvements per active practitioner-year or workshop-year |
| New functional capability | Successful capability introductions per eligible project-year |
| Diffusion and adoption | New informed users and new adopters per exposed eligible user-year |

**Calendar time should emerge from opportunity, effort, and transmission.** A capable community can wait a long time because nobody funds the relevant trials; another can make rapid progress after migration brings a missing skill.

## 2.3 Proposed starting parameters

The following are **engineering priors for initial testing, not historical estimates**. Their empirical confidence is low. They make a first implementation tunable without pretending that the literature supplies universal coefficients.

| Parameter | Initial value and sensitivity range | Purpose |
| --- | --- | --- |
| Routine experimentation time | Start at **1%** of working time; test **0–5%** | Small-scale modifications during productive work |
| Protected experimentation time for funded specialists | Start at **25%**; test **10–50%** | Dedicated trial activity after subsistence, maintenance, and other duties |
| Active technical mentors per learner | Start at **2**; test **1–4** | Access to alternative demonstrations; every mentor interaction consumes time |
| Cross-community share of technical interactions | Start at **5%**; test **0–20%** | A sensitivity parameter for bridging otherwise local knowledge networks |
| Repeat successes before a method is considered locally reliable | Start at **3**; test **2–10 trials** | Separate lucky success from reproducible practice; vary by task risk and observability |
| Independent reproductions required by a formal validation institution | Start at **1**; test **0–3 other practitioners or teams** | A configurable institutional norm, not a universal requirement for discovery |
| Conditional search difficulty, \(E\_{50}\) | Test **\(10^3,\ 10^4,\ 10^5\)** qualified experiment-hours for selected subproblems | A logarithmic calibration grid—not recommended historical values for every technology |

The most important parameter is \(E\_{50}\): **the median search exposure needed to resolve a particular subproblem when its necessary conditions are present**. It must be authored or calibrated by task. “Improve a familiar tool” and “develop a reliable new engine system” should not share one value.

---

# 3. Variation across periods and regions

These categories describe historical settings. **They should not become runtime eras or civilization traits.**

| Setting | Historical evidence | Simulation implication |
| --- | --- | --- |
| **Foragers** | Pottery at Xianrendong in China dates to approximately **20,000–19,000 calendar years BP**, in a foraging context. Pottery therefore cannot universally require farming or cities. [PubMed](https://pubmed.ncbi.nlm.nih.gov/22745428/) | Permit sophisticated material practices among mobile populations. Model skill transmission, seasonal activity, local resources, and portable versus fixed equipment |
| **Early farming societies** | Archaeological domestication research identifies parallel trajectories in different world regions and protracted changes in crop traits. [PubMed](https://pubmed.ncbi.nlm.nih.gov/24753577/) | Agricultural innovation should include repeated cultivation, selection, exchange, and local adaptation—not only conscious invention by a specialist |
| **Preindustrial African production systems** | Research on African ironworking documents varied production methods and the importance of reconstructing materials and operational sequences. Questions about origins and transmission remain distinct from identifying how particular processes worked. [Academia](https://www.academia.edu/35578564/African_Iron_Production_and_Iron_Working_Technologies_Methods) | Support multiple furnace, fuel, and ore-processing traditions. Do not impose one European sequence or treat every difference as a technology-level deficit |
| **Pre-Columbian South America** | A roughly **3,000-year-old copper mask** from the Argentinian Andes demonstrates early metalworking in a funerary context and complicates overly centralized accounts of its development. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/ancient-metalworking-in-south-america-a-3000yearold-copper-mask-from-the-argentinian-andes/80E3CFE81BC10CFFA5602230A16B40DF) | Ritual, status, and aesthetic demands can sustain technically demanding work without immediate agricultural or military productivity gains |
| **Abbasid scholarly environments** | Gutas’s study of the eighth- to tenth-century translation movement in Baghdad emphasizes political and social support for the transfer and transformation of learned knowledge. [Routledge](https://www.routledge.com/Greek-Thought-Arabic-Culture-The-Graeco-Arabic-Translation-Movement-in-Baghdad-and-Early-Abbasaid-Society-2nd-4th5th-10th-c/Gutas/p/book/9780415061339) | Translation is active knowledge work: patrons, multilingual specialists, texts, teaching, and receiving expertise all matter |
| **Late imperial China** | Elman describes selective appropriation of foreign astronomy and mathematics rather than wholesale acceptance of the accompanying theology. [JSTOR](https://www.jstor.org/stable/j.ctv1pncqmd) | Communities can adopt useful components while rejecting associated claims, institutions, or identities. Cultural distance should not be a single all-purpose transmission penalty |
| **Industrial settings** | The economic attractiveness of machinery differed across Britain, France, and India in Allen’s analysis; later manufacturing systems also required complementary engineering and organizational changes. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/industrial-revolution-in-miniature-the-spinning-jenny-in-britain-france-and-india/C635E725AAFC87D7F59030A2F478D886) | Let wages, energy, scale, input quality, capital access, and maintenance skills select among techniques |
| **Modern research systems** | Evidence from research funding, higher education, and international diffusion shows that organized resources and knowledge circulation matter, while adoption remains uneven across countries. [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/8ad96999-7c50-42d8-aedf-778272c766de) | Add laboratories, specialized careers, formal records, and large projects when their supporting capabilities exist—not when a date is reached |

The common mechanism is **unequal access to opportunities and capabilities**, not an intrinsic ranking of peoples.

An especially important distinction is between **frontier invention** and **catch-up innovation**. A settlement can advance rapidly by acquiring and adapting existing methods without producing many world-firsts. Its inhabitants are still solving real problems: selecting suitable variants, substituting local materials, learning execution, and reorganizing production.

---

# 4. Stylized facts a successful simulation should reproduce

These are validation targets, not scripts to force into every world.

| Pattern | Testable TCE output |
| --- | --- |
| **Refinement and recombination coexist** | Most activity should not consist of entirely new fundamental capabilities; familiar procedures should receive repeated modifications. Patent evidence offers one context-specific comparison. [arXiv](https://arxiv.org/html/1406.2938v1) |
| **Population effects are conditional** | Increasing qualified practitioners, teacher access, or complementary skills should affect outcomes differently from adding people with no relevant access. Laboratory findings and archaeological disputes both argue against one universal population coefficient. [Nature](https://www.nature.com/articles/nature12774) |
| **Different network structures produce different trajectories** | At matched effort, some clustered networks should preserve useful diversity, while better-connected networks should often spread known methods faster. Partial connectivity must not be automatically superior for every task. [Arizona State University](https://asu.elsevierpure.com/en/publications/partial-connectivity-increases-cultural-accumulation-within-group/) |
| **The order of capabilities can differ** | At least some worlds should produce pottery before agriculture, consistent with the Chinese archaeological case. [PubMed](https://pubmed.ncbi.nlm.nih.gov/22745428/) |
| **Invention and widespread use can be far apart** | Track prototype dates, first local use, and adoption intensity separately; the 45-year cross-country mean is one benchmark, not a required delay. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.100.5.2031) |
| **Institutions have indirect effects** | A school or academy can affect surrounding workshops through migration, contacts, and training—not only through the output of its paid staff. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fpol.20200320) |
| **Recorded innovation is incomplete** | Patents, public credit, and formal publications must be observational layers rather than the simulation’s complete inventory of innovation. [RCNi Company Limited](https://www.journals.uchicago.edu/toc/jle/2012/55/1) |
| **A technical success can require another development process to scale** | Output, cost, reliability, and supply should continue changing after the initial success, as in penicillin manufacturing. [American Chemical Society](https://www.acs.org/education/whatischemistry/landmarks/flemingpenicillin.html) |

Additional **model-consistency tests** should check that knowledge can be lost when its carriers disappear, that an unfunded project consumes no invisible research effort, and that receiving a document does not automatically create execution skill.

Run controlled ablations across many seeds. Hold initial capabilities and resources constant while changing one mechanism: teacher access, travel, secrecy, funding, or exclusion. Compare distributions of waiting times, independent inventions, failed trials, adoption, and knowledge loss.

Do not calibrate only to a single historical sequence. A model that always recreates that sequence has failed TCE’s divergence requirement even if its average dates look convincing.

---

# 5. Recommended individual-agent implementation

## 5.1 Minimal data model

Five object types are sufficient for a useful first version.

| Object | Minimum state |
| --- | --- |
| **Person** | Domain skills, partial procedural knowledge, beliefs about performance, available time, contacts, institutional access |
| **Practice or recipe** | Required capabilities, material constraints, adjustable parameters, alternative routes, outcome evaluator |
| **Experiment or development project** | Participants, proposed change, resources committed, work completed, observations, failures, unresolved subproblems |
| **Institution** | Budget, membership rules, topic priorities, teaching capacity, disclosure rules, records, sanctions |
| **Knowledge carrier** | Practitioner, document, artifact, or facility; information carried, accessibility, condition, and provenance |

Keep recipe definitions shared and immutable where possible. Store sparse per-person knowledge rather than copying the entire technology catalog into every agent.

**Institutional membership should grant access opportunities, not instant possession of every member’s knowledge.**

## 5.2 Discovery loop

A practical loop is:

```
Ordinary work or observation reveals a problem or surprising result.
An agent proposes a change using accessible knowledge.
The agent seeks collaborators, permission, tools, and funding.
A concrete experiment reserves time and consumes materials.
The world evaluates what happened.
Participants update procedural knowledge and beliefs.
Successful methods face reproduction and adoption decisions.
Teaching, travel, documents, and artifacts spread partial information.
Deaths, migration, neglect, and institutional failure affect retention.
```

The proposer need not know whether the experiment is physically feasible. The simulation’s material rules determine that.

**Do not reroll the same impossible configuration until it randomly works.** Identical trials may provide additional evidence where outcomes are noisy, but physically impossible recipes should remain impossible. Search must change a configuration, acquire a missing capability, or resolve genuine uncertainty.

## 5.3 A usable reduced-form discovery hazard

Where detailed engineering search would be too expensive, approximate the chance of resolving subproblem \(j\) in project \(p\) over one update interval as:

\[
P\_{pj}
=
1-\exp\left[
-\frac{\ln 2}{E\_{50,j}}
G\_{pj}E^{\mathrm{eff}}\_{pj}
\right].
\]

Where:

* \(E^{\mathrm{eff}}\_{pj}\) is **actual qualified experiment effort**, in person-hours.
* \(E\_{50,j}\) is conditional median search exposure, also in person-hours.
* \(G\_{pj}\) is a feasibility gate: zero if a necessary physical condition is absent, one otherwise.

Skill fit, tools, and complementary expertise determine effective effort or the applicable difficulty—not a stack of unlimited bonuses.

This is a **proposed reduced-form model**, not an equation estimated by the historical literature. Use it for unresolved search among plausible approaches. Use authored material and process rules for the outcome of a specified experiment.

### Worked example

Suppose an authored subproblem has:

\[
E\_{50}=10{,}000\ \text{qualified experiment-hours}.
\]

Five eligible workshops each devote 100 qualified hours per year. Their combined exposure is 500 hours/year. Under a stationary, independent-search approximation:

* The median waiting time to at least one success is **20 years**.
* The chance of at least one success by 40 years is **75%**.
* Doubling eligible effort halves the median waiting time **only while other conditions remain unchanged**.

These numbers illustrate the model; they are **not historical estimates**.

This is not a research bar: no guaranteed completion occurs at 10,000 hours, no effort is freely transferable between unrelated problems, and the knowledge and resources remain attached to specific projects.

For several independent projects:

\[
P(\text{at least one success})
=
1-\prod\_p(1-P\_{pj}).
\]

Do not add another population multiplier. Roll a collaborative project once, not once per participant plus once for the team. Complementary work should be limited by missing specialties and coordination, not just total headcount.

## 5.4 Make institutional effects operate through the loop

A patron’s contribution should purchase protected time, equipment, or materials.

A school should increase access to instruction and records.

A scientific institution should improve testing, comparison, and communication.

A guild should alter entry, apprenticeship, cooperation, and secrecy.

A patent should alter disclosure and expected returns.

This makes institutional effects inspectable: the simulation can explain *why* a discovery happened through its event history rather than reporting that a settlement had “+25% innovation.”

For example:

> A potter encounters repeated kiln failures. A visiting craftsperson introduces a different refractory material. A merchant finances trials. An apprentice identifies a workable mixture. Other workshops observe its performance, but only those facing sufficiently high fuel costs adopt it immediately.

That is a causal history generated from people, constraints, and decisions.

## 5.5 Computation for 10k–50k people

Use production events and contact events to generate opportunities. Avoid checking every person against every unknown recipe.

Recommended implementation shortcuts:

* **Reverse prerequisite indexes:** Reconsider relevant candidates when a person or workshop gains a capability.
* **Bounded candidate sets:** Search a small neighborhood of familiar operations and materials.
* **Sparse networks:** No all-to-all communication or pairwise recombination.
* **Cached workshop capabilities:** Invalidate when staff, equipment, access, or inputs change.
* **Daily or weekly discovery updates:** These do not need to run at rendering frequency.
* **Aggregate only where attribution remains recoverable:** A batch calculation may select a successful project probabilistically, but must still identify actual participants, resources, and observations.
* **Deterministic event logs and random streams:** Preserve reproducibility for debugging and historical explanations.

For legal and organizational innovations, reuse the same experimentation and transmission framework, but evaluate outcomes against agents’ **conflicting objectives and changing circumstances**. A law is not a physical recipe with one objectively optimal output.

## 5.6 The population-scale constraint

TCE’s 10k–50k people constitute a plausible regional population, not a modern worldwide research system.

Choose explicitly between a closed region with a limited frontier and a region connected to separately modeled external populations. External exchange, where included, should have carriers, costs, information limits, and provenance—not timed gifts from an invisible global technology tree.

Similarly, a finite authored capability catalog can generate many divergent histories, but not literally unlimited new physical principles. Parameterized recipes and recombination greatly expand possibilities; they do not eliminate the limits of the authored world model.

## 5.7 Existing models and games worth borrowing from

| Model or game | Useful component | What TCE must add or avoid |
| --- | --- | --- |
| **Arthur and Polak, “The evolution of technology within a simple computer model”** | Technologies assembled from components become components for further technologies | Their computational technology space is not a complete human economy; add situated agents, resources, teaching, and adoption. [Santa Fe Institute](https://www.santafe.edu/research/results/working-papers/the-evolution-of-technology-within-a-simple-comput) |
| **Tria et al., “The dynamics of correlated novelties”** | A model in which encountering novelty expands the space of further possibilities | Empirical applications include words, music, and other cultural sequences—not a calibrated historical invention clock. [Nature](https://www.nature.com/articles/srep05890) |
| **Derex and Boyd’s network experiments** | Local learning, diversity preservation, and recombination between groups | Do not universalize the best network structure from one task. [Arizona State University](https://asu.elsevierpure.com/en/publications/partial-connectivity-increases-cultural-accumulation-within-group/) |
| **de la Croix, Doepke, and Mokyr’s apprenticeship model** | Access to masters and the organization of training as drivers of knowledge transmission | Borrow the mechanisms, not a fixed civilizational classification of apprenticeship regimes. [OUP Academic](https://academic.oup.com/qje/article-abstract/133/1/1/3950283) |
| **Dwarf Fortress, documented 2015 implementation** | Individually tracked knowledge, cross-branch prerequisites, scholars, teachers, books, and knowledge loss; the developer log describes more than 300 simple innovations | This is a historical implementation reference: the log explicitly noted limited practical effects on production then. It is not evidence of a complete present-day innovation economy. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2015.html) |

**Recommended v1 priority:** production-linked experimentation, partial transmission, adoption decisions, and knowledge loss. Add elaborate patent systems, academies, and large research organizations only after those foundations work.

---

# 6. Sources, datasets, and evidence limitations

The papers cited throughout provide mechanisms and specific observations. The following sources are particularly useful for building calibration and validation datasets.

| Source or dataset | What to extract | Main limitation |
| --- | --- | --- |
| **Cross-country Historical Adoption of Technology—CHAT** | Adoption trajectories for more than 100 technologies across more than 150 countries, with historical coverage extending back to 1800 | Primarily diffusion and use, not a census of invention events. [National Bureau of Economic Research](https://www.nber.org/research/data/cross-country-historical-adoption-technology) |
| **USPTO PatentsView** | Inventor networks, organizations, locations, classifications, and citation relationships | Patenting propensity, classification, and disambiguation affect observed patterns; unpatented innovation is absent. [USPTO](https://www.uspto.gov/ip-policy/economic-research/patentsview) |
| **D-PLACE** | Cultural, linguistic, environmental, and geographic variables for comparative analysis | Ethnographic observations are not timeless descriptions or independent samples; account for shared history and spatial dependence. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) |
| **Archaeological datasets underlying domestication and diffusion studies** | Trait trajectories, dated sites, uncertainty intervals, and spatial fronts | First surviving evidence is not necessarily the first invention; migration and cultural transmission can be mixed. [PubMed](https://pubmed.ncbi.nlm.nih.gov/24753577/) |
| **AEA replication materials for Comin–Hobijn, Moser–Voena–Waldinger, Andrews, and Bloom et al.** | Reproduce study-specific outcomes before using them as calibration targets | Each identifies a particular setting and measurement strategy, not a universal institutional multiplier. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.100.5.2031) |

### What is well supported, and what remains uncertain?

**Strongest foundations:** invention frequently builds on existing capabilities; knowledge transmission is costly and uneven; successful prototypes and widespread use are distinct; institutions affect incentives and access; recorded patents capture only part of innovation.

**Contested generalizations:** a universal population–innovation elasticity; one optimal network density; the overall benefits of guilds or patents; single-cause explanations of industrialization; and an inevitable secular decline in the productivity of ideas.

**Thinnest quantitative evidence:** universal preindustrial invention hazards, the fraction of ordinary work devoted to experimentation, cross-cultural rates of tacit-knowledge loss, and transferable numerical effects of censorship or patronage. These should remain explicit calibration parameters rather than acquire false precision.

The central design principle is:

> **People carry knowledge. Networks make combinations accessible. Institutions allocate opportunities. Experiments test possibilities. Local conditions select adoption. Teaching and turnover determine what survives.**

That structure can generate both cumulative progress and believable divergence without requiring a research bar, a predetermined sequence of eras, or an assumption that every world must eventually reach the same destination.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9284b-79f0-83e9-b0c1-3ebd3f54e530)
