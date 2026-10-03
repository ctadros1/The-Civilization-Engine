# Academic simulations of societies and states

## Design report for The Civilization Engine

**The strongest approach for TCE is to borrow mechanisms, not entire academic models.** Build the population’s material life around household production, storage, movement, and exchange; build institutions around recurring coordination problems; and use historical datasets to test the resulting patterns. Do not turn a statistical measure of “social complexity” into an agent objective or a civilization-level progression bar.

The central distinction is between **demonstrating a possible mechanism**, **reproducing an observed pattern**, and **establishing a historical cause**. The models below achieve these to different degrees. Artificial Anasazi’s replication literature is particularly instructive: reproducing an archaeological population curve did not establish that the simulated household behavior was its principal explanation. [JASSS](https://jasss.soc.surrey.ac.uk/12/4/13.html)

My recommendation is a **multilevel simulation with persistent individuals underneath comparatively inexpensive household, settlement, and institutional models**. That preserves TCE’s visible daily life without requiring every person to deliberate about the entire society.

---

## 1. How the models work

### Sugarscape: local resource competition produces aggregate economic patterns

Sugarscape is a family of progressively extended models rather than one immutable implementation. Its foundational loop places heterogeneous agents on a resource landscape: agents inspect nearby locations, move, harvest, consume, accumulate reserves, and potentially die. More elaborate versions add reproduction, cultural exchange, trade, conflict, and disease. A formal analysis of the original model family documents these extensions and their relationship to the simpler resource-competition core. [JASSS](https://jasss.soc.surrey.ac.uk/12/1/6/appendixB/EpsteinAxtell1996.html?utm_source=chatgpt.com)

A concrete, inspectable example is NetLogo’s **Immediate Growback** version. Agents look horizontally and vertically within their individual vision ranges, select an unoccupied location with the most sugar, prefer nearer destinations when otherwise tied, harvest, and pay their metabolic cost. Patches replenish to their maximum each tick. This is deliberately a small decision rule, not a planner. [CCL](https://ccl.northwestern.edu/netlogo/models/Sugarscape1ImmediateGrowback)

The **Wealth Distribution** version changes the resource dynamics: sugar regenerates incrementally, and agents differ in their ability to find and retain it. The resulting distribution is unequal even without an elaborate financial system. However, this implementation replaces dead agents with new ones, holding population constant; its random maximum ages of **60–100 ticks are model ticks, not human years**. Its wealth distribution is therefore conditional on a particular resource and demographic system, not a validated universal distribution. [CCL](https://ccl.northwestern.edu/netlogo/models/Sugarscape3WealthDistribution)

**Reusable mechanism:** bounded local choice plus heterogeneous circumstances can produce meaningful inequality without scripting wealthy and poor classes.

**TCE adaptation:** preserve differences in location, knowledge, household obligations, assets, and access rights. Avoid making permanently assigned “ability” the principal explanation of inequality. Land ownership, inheritance, bargaining power, taxation, and unequal exposure to shocks should also change opportunities.

### Artificial Anasazi: households respond to reconstructed agricultural conditions

Artificial Anasazi models ancestral Pueblo settlement and population dynamics in Arizona’s Long House Valley. Its important methodological move was to place agents in a landscape reconstructed from archaeological and environmental evidence, rather than an arbitrary resource map. The original study compared simulated population growth and decline with the archaeological sequence. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC128597/)

In the documented replication, an agent represents a **five-person household**, with an age, food reserves, a residence, and agricultural access. Each annual step calculates harvest, checks survival, estimates next year’s food availability, relocates households expecting inadequate food, allows household formation, updates water sources, and advances household age. The harvest forecast is essentially last year’s experience. Households interact indirectly through competition for available land, rather than through a developed exchange economy. [CCL](https://ccl.northwestern.edu/netlogo/models/ArtificialAnasazi)

The useful causal structure is:

**Changing yields → reserve depletion → relocation or household failure → changing settlement distribution.**

**TCE adaptation:** use food reserves and expected future sufficiency to generate migration pressure, but implement them through real people and households of changing composition. A household should not disappear because its abstract household-age counter expires, nor should a new household materialize with a complete family. Migration also needs remembered destinations, information received from others, travel costs, and land rights.

### Village Ecodynamics: ecology, exchange, and collective organization

Kohler and colleagues’ Village Ecodynamics Project is a broader research program connecting archaeological reconstruction with ecological, economic, and political models. It is not simply “Artificial Anasazi at a larger population.” Its published modeling program includes subsistence change, social organization, and depopulation across multiple model variants. [UM Impact](https://umimpact.umt.edu/en/publications/modelling-prehispanic-pueblo-societies-in-their-ecosystems/)

In the *Village* model, households choose where to live, farm, hunt, and gather water and fuel. Subsistence activities have time and calorie costs; resource availability changes with climate and human use. Households can exchange maize and meat through reciprocal relationships, although the described implementation does not barter across those two resource currencies. [SFI Edu](https://sfi-edu.s3.amazonaws.com/sfi-edu/production/uploads/sfi-com/dev/uploads/filer/00/e4/00e460a1-37b7-4355-a41f-3940276e434a/11-07-025.pdf)

The leadership extension adds a public-goods problem. Households can participate in hierarchical or nonhierarchical arrangements. Leaders collect contributions and undertake coordination or enforcement; leadership can become advantageous when mutual monitoring becomes expensive as groups grow. This makes paid leadership a possible outcome of cooperation rather than requiring a king at initialization. [SFI Edu](https://sfi-edu.s3.amazonaws.com/sfi-edu/production/uploads/sfi-com/dev/uploads/filer/00/e4/00e460a1-37b7-4355-a41f-3940276e434a/11-07-025.pdf)

Later work, *How to Make a Polity*, couples hierarchy with warfare and compares simulated organization against habitation and ritual-group evidence. Its authors argue that some central Mesa Verde communities were organized into village-spanning polities rather than universally autonomous villages. That is an interpretation supported by their model and archaeological comparisons, not recovery of a uniquely proven historical political map. [Cambridge University Press](https://www.cambridge.org/core/journals/american-antiquity/article/how-to-make-a-polity-in-the-central-mesa-verde-region/CD9B2FD33E18A52F9D6C69F285EA6775)

**TCE adaptation:** this is the closest fit for an early-agrarian institutional foundation. Let people establish organizations because particular activities—shared storage, irrigation maintenance, defense, adjudication—benefit from coordination. Give those organizations costs, beneficiaries, opponents, and failure conditions.

### Cioffi-Revilla’s MASON models: a progression from households to political organization

**MASON is the simulation framework, not the societal theory.** Its Java core supports custom agent models with optional visualization, and its current documentation includes facilities for combining agent-based and discrete-event simulation. TCE should borrow the separation of model and display, not assume that using MASON’s concepts requires embedding Java. [Department of Computer Science](https://people.cs.gmu.edu/~eclab/projects/mason/)

#### HouseholdsWorld: bounded decisions within a social and ecological hierarchy

HouseholdsWorld models pastoral households, herds, camps, and clans. Weather affects forage; forage affects livestock and household prospects; households relocate and camps split or merge. Its location decisions use sequential filtering of candidates according to ecological and social considerations, rather than exhaustive global optimization. This is especially relevant to TCE: recognizable behavior does not require a household to search every possible destination or solve a general planning problem. [ResearchGate](https://www.researchgate.net/publication/226786134_The_MASON_HouseholdsWorld_Model_of_Pastoral_Nomad_Societies)

Cioffi-Revilla’s methodological account advocates **progressive modeling**: establish a simpler ecological and household model, then add social aggregation and political processes. Its political theory links challenges and opportunities to recognition, collective action, and organizational consequences. The important qualification is that this causal structure is an authored hypothesis; increasing organization after successful collective action is not an assumption-free discovery. [JASSS](https://jasss.soc.surrey.ac.uk/13/1/7.html)

#### MASON Hierarchies: tribute, redistribution, control limits, and separation

MASON Hierarchies represents nested political organization over an ecological and household foundation. Resources move upward through political relationships and can be redistributed downward. Leadership has limits on effective control; military contributions, redistribution, and dissatisfaction influence conquest, rebellion, and separation. Political organization is represented explicitly as a hierarchy rather than as a country-level “stability” number. [Center for Social Complexity](https://socialcomplexity.gmu.edu/wp-content/uploads/2017/03/Cioffi-Revilla-et-al.Hierarchies-2015.pdf)

**TCE adaptation:** represent offices, subordinate organizations, obligations, and resource transfers as actual records. Cache aggregate military and fiscal capacity, but retain the relationships that produced them. A subordinate settlement’s refusal to supply labor should alter both material capacity and political relationships.

#### RebeLand: government fails when problems outrun its ability to respond

RebeLand starts with an existing polity. Its citizens earn, spend, consume, and evaluate their circumstances; city and state authorities collect resources, address issues, and provide responses. Rebels offer competing policies or organization. The authors’ presentation makes **issue burden versus government capacity** a central feedback: different combinations of problems, response, and opposition produce stability, survival under pressure, or government failure. It is a model of governance and instability, not an explanation of the first state’s origin. [Semantic Scholar](https://pdfs.semanticscholar.org/3946/7857b927e646669623dd2ec4d2bd3cba2904.pdf)

**TCE adaptation:** use explicit queues of disputes, repairs, provisioning requests, and security problems. This creates a cheap, legible reason for administrative expansion: a larger society can produce more work than existing offices can handle.

### Mathematical polity models: growth and collapse through a small set of relationships

Gavrilets, Anderson, and Turchin’s *Cycling in the Complexity of Early Societies* represents villages as a spatial network and polities as hierarchical networks. Villages transfer tribute upward; leaders have a limited number of direct subordinates; conquest changes membership; rebellion can remove a subtree of settlements. The model produces repeated political growth and collapse. Its behavior is especially sensitive to how reliably greater power wins conflicts and to the chief’s expected tenure. [Sociological Studies](https://www.sociostudies.org/almanac/articles/cycling_in_the_complexity_of_early_societies/)

Crucially, some instability is explicitly supplied: a chief’s death can trigger separation of subordinate communities. Thus “succession destabilizes this model” is not independent evidence that every society must fragment on succession. [Sociological Studies](https://www.sociostudies.org/almanac/articles/cycling_in_the_complexity_of_early_societies/)

**TCE adaptation:** take the sparse political graph and contested obligations. Replace automatic fragmentation with succession rules, rival claimants, institutional continuity, local incentives, and the practical costs of leaving.

---

## 2. What worked, what failed, and how convincing the validation is

### Reproducing a pattern is useful—but the pattern must discriminate between explanations

| Model or research program | What it reproduced or explained | What limits the conclusion |
| --- | --- | --- |
| **Sugarscape** | Unequal accumulated wealth from simple local resource acquisition. | A skewed distribution is a weak discriminator: many mechanisms can produce one. The population and resource rules substantially shape the result. [CCL](https://ccl.northwestern.edu/netlogo/models/Sugarscape3WealthDistribution) |
| **Artificial Anasazi** | Much of the observed population trajectory and environmentally constrained occupation. | Janssen’s replication found that calibrated environmental carrying capacity explained much of the apparent success; agents added comparatively limited explanatory improvement. Environmental conditions alone did not explain complete abandonment. [JASSS](https://jasss.soc.surrey.ac.uk/12/4/13.html) |
| **VEP leadership model** | A plausible route to larger groups with leadership, with some correspondence to the archaeological sequence. | It diverged substantially after about AD 900 and during late-thirteenth-century depopulation. The authors identified limitations in environmental reconstruction and omission of coercive intergroup competition. [SFI Edu](https://sfi-edu.s3.amazonaws.com/sfi-edu/production/uploads/sfi-com/dev/uploads/filer/00/e4/00e460a1-37b7-4355-a41f-3940276e434a/11-07-025.pdf) |
| **Later VEP polity modeling** | Compared political-organization hypotheses against settlement and ritual-group patterns across three areas. | Archaeological proxies constrain possible organizations but do not uniquely identify institutions, intentions, or political boundaries. [Cambridge University Press](https://www.cambridge.org/core/journals/american-antiquity/article/how-to-make-a-polity-in-the-central-mesa-verde-region/CD9B2FD33E18A52F9D6C69F285EA6775) |
| **MASON Hierarchies** | Illustrative sequences of federation, conflict, and political change resembling historical narratives. | The chapter explicitly describes its results as illustrative; historical narrative comparison is weaker than held-out quantitative prediction. [Center for Social Complexity](https://socialcomplexity.gmu.edu/wp-content/uploads/2017/03/Cioffi-Revilla-et-al.Hierarchies-2015.pdf) |
| **Polity-cycling model** | Recurrent aggregation and fragmentation under varying control and conflict assumptions. | It is a stylized mechanism experiment, not a forecast of particular historical dates. [Sociological Studies](https://www.sociostudies.org/almanac/articles/cycling_in_the_complexity_of_early_societies/) |

The archaeological evidence itself deserves attention. VEP’s historical-ecology work used occupation histories from **3,176 habitation sites** and more than **1,700 construction timbers**, together with agricultural-productivity estimates. These are substantial constraints, but they are not an annual census of identified individuals. [Cambridge University Press](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/FDC2EE1ED3E2DB39B186E92FCAFC8A6E/S0002731600040531a.pdf/historical-ecology-in-the-mesa-verde-region-results-from-the-village-ecodynamics-project.pdf?utm_source=chatgpt.com)

For TCE, my inference is that validation should compare **several independent outputs simultaneously**: settlement locations, population trajectories, settlement-size distributions, household reserves, migration, inequality, and organizational scale. A model that matches population but produces implausible journeys and relationships is not sufficient for an observer-facing simulation.

### The most useful “postmortems” are replication studies and author accounts

**Artificial Anasazi: preserve the executable research claim.** Janssen encountered mismatches between available code and published descriptions, disappearing access paths, and multiple implementation versions. The study also distinguished a good selected run from performance across repeated runs. The lesson is concrete: archive the code revision, input data, parameters, seeds, and comparison procedure together—not merely a paper or an attractive screenshot. [JASSS](https://jasss.soc.surrey.ac.uk/12/4/13.html)

**Sugarscape: visual structure can be an implementation artifact.** NetLogo’s immediate-regrowth documentation explicitly notes unintended spatial layering caused by the movement and regeneration rules. Kehoe’s formal specification separately identifies ambiguous or missing rules and distinguishes synchronous from asynchronous updating. Attractive organization on screen is therefore something to investigate, not automatically celebrate as social emergence. [CCL](https://ccl.northwestern.edu/netlogo/models/Sugarscape1ImmediateGrowback)

**VEP: reproduction requires the environment as well as the agents.** Its archived model release depends on a substantial external dataset and recommends a prepared virtual machine. The release record does not supply a detailed commercial-style patch history. That is a reminder to package runnable reference scenarios, not just simulation source files. [CoMSES Net](https://www.comses.net/codebases/2518/releases/1.1.0/)

**Kohler’s own limitation statement is directly relevant to TCE.** In a 2015 interview, he distinguished the field’s success in modeling behavior from its weaker treatment of culture and cultural change. He also emphasized that people act, while high-level complexity variables are analytical summaries of those actions. That supports treating architecture, legal traditions, and cultural meaning as additional TCE design work—not as features already solved by an ecological household model. [simulatingcomplexity](https://simulatingcomplexity.wordpress.com/2015/08/12/tim-kohler-the-nine-questions/)

---

## 3. Seshat: what data exists, what it establishes, and how TCE should use it

### Available data

As checked for this report, Seshat provides publication-specific replication datasets, a curated **Polaris-2026** spreadsheet release, and a live API. The spreadsheet separates collections such as general information, social complexity, and military technology. The official site also links Python tools for downloading variables and constructing temporal sequences. **Equinox-2020 remains available but is not the newest advertised snapshot.** [Seshat Databank](https://seshatdatabank.info/data)

Seshat records numerical estimates and ranges, along with states such as present, absent, unknown, inferred present, and inferred absent. Entries include explanatory material and scholarly provenance. Its original geographically stratified sample has since expanded; it should not be interpreted as a complete, uniformly observed census of historical societies. [Seshat Databank](https://seshatdatabank.info/how-we-work)

For TCE, I would organize an import and comparison layer as follows:

| Historical data family | Corresponding TCE measurement |
| --- | --- |
| Population and territorial scale | Resident population, controlled territory, largest settlement |
| Organizational structure | Number and depth of offices, settlement hierarchy, subordinate organizations |
| Governance and legal institutions | Actual jurisdiction, adjudication mechanisms, rules in force, personnel |
| Information and economic organization | Recordkeeping capabilities, communication reach, exchange and payment arrangements |
| Infrastructure and military capabilities | Built assets, maintenance burden, logistical reach, capabilities actually available |

This is a proposed mapping, not a recommendation to reproduce a single composite score.

A particularly useful detail in Seshat’s institutional coding is that **a formal legal code need not be written**: an established, uniformly transmitted oral system can qualify. TCE should therefore avoid equating “no writing technology” with “no formal law.” [Seshat DB](https://seshat-db.com/sc/formal_legal_codes_all/?utm_source=chatgpt.com)

### What the statistical work found

The widely cited 2018 analysis examined **414 societies from 30 regions**, spanning approximately **10,000 years**, using **51 variables grouped into nine characteristics**. One principal component captured roughly three-quarters of the observed variation. In other words, several measures of social scale, governance, economy, and information systems tended to vary together. These figures describe that study’s sample—not the present databank’s total coverage. [DASH](https://dash.harvard.edu/entities/publication/73120379-39d5-6bd4-e053-0100007fdf3b)

That is evidence of structured co-variation. It does **not** establish that societies follow a mandatory ladder, that all dimensions move together in every case, or that a single hidden “complexity” quantity causes their institutions.

A 2020 follow-up found a phased relationship: growth in polity scale, then stronger information-processing and economic organization, then further growth in scale. The authors explicitly reject inevitability. For TCE, this suggests a testable mechanism: communication and administration can become bottlenecks, and overcoming them can make larger organizations viable. It does not justify an automatic population threshold that unlocks bureaucracy. [Penn State](https://pure.psu.edu/en/publications/scale-and-information-processing-thresholds-in-holocene-social-ev/)

### Important cautions

**Unknown must remain different from absent.** The 2019 Seshat-based paper claiming that complex societies preceded moralizing gods was retracted in 2021 following challenges concerning missing-data treatment; the retraction explicitly references that critique. This is a caution about inference and analysis choices, not a reason to discard the entire databank or assume the opposite historical conclusion has been established. [PubMed](https://pubmed.ncbi.nlm.nih.gov/34234342/?utm_source=chatgpt.com)

**A coded unit is not always a sovereign state.** Seshat sometimes uses “quasi-polities” for culturally similar small communities lacking common jurisdiction. Instantiating such a record as one unified government would manufacture political organization the source does not claim existed. [Seshat Databank](https://seshatdatabank.info/how-we-work)

**Historical observations and simulation truth have different resolutions.** A century-scale sequence cannot validate daily commuting or the precise month when an office formed. Compare aggregated simulation output with the corresponding historical observation window, preserving uncertainty.

**Use snapshots in the development pipeline, not live historical data at runtime.** Pin the dataset version and file hash; preserve original coding and provenance; transform it into a separate comparison schema. Audit permissions for any material redistributed with the game. This keeps a later database correction from silently changing TCE’s balance.

---

## 4. Numbers and real-time feasibility

### Reported model scales—not interchangeable performance benchmarks

| Model / study | Reported configuration or scale | What the number actually means |
| --- | --- | --- |
| **Artificial Anasazi replication** | **80 × 120 cells**, each **100 × 100 m**: **96 km²**. Annual simulation, AD **800–1350**; reconstructed peak roughly **250 households**. | Hundreds of household agents, not thousands of individually navigating citizens. [JASSS](https://jasss.soc.surrey.ac.uk/12/4/13.html) |
| **VEP leadership implementation** | **200 × 227 cells**, each **4 ha**, about **1,800 km²**; starts with **200 households** in AD 600; reported analysis includes **36 runs**. | A landscape-and-household model with annual subsistence processes. [SFI Edu](https://sfi-edu.s3.amazonaws.com/sfi-edu/production/uploads/sfi-com/dev/uploads/filer/00/e4/00e460a1-37b7-4355-a41f-3940276e434a/11-07-025.pdf) |
| **MASON Hierarchies** | Parameter exploration includes **16, 64, and 256 clans**. | These are social aggregates; they are not a benchmark for equivalently many rendered individuals. [Center for Social Complexity](https://socialcomplexity.gmu.edu/wp-content/uploads/2017/03/Cioffi-Revilla-et-al.Hierarchies-2015.pdf) |
| **Gavrilets–Anderson–Turchin polity model** | **37, 61, or 91 villages**, **1,000-year runs**; spans of control **5–7**, tribute fractions **10–30%**. | Small, inexpensive political networks used for parameter experiments. [Sociological Studies](https://www.sociostudies.org/almanac/articles/cycling_in_the_complexity_of_early_societies/) |
| **Seshat 2018 study** | **414 societies**, **51 variables**, **30 regions**. | An empirical comparison dataset, not a simulated-agent count. [DASH](https://dash.harvard.edu/entities/publication/73120379-39d5-6bd4-e053-0100007fdf3b) |

**I did not find a comparable end-to-end benchmark demonstrating 50,000 persistent people, daily routing, resource logistics, and UE5 rendering in these sources.** Their reported scales support the usefulness of the abstractions, not a specific Windows frame rate.

### Which mechanisms should be inexpensive?

The following are **engineering estimates from the proposed data structures**, not published TCE benchmarks.

| Mechanism | Proposed computational form | Expected cost characteristic |
| --- | --- | --- |
| Household reserves, needs, production accounts | Compact household records; updates on consumption or production events | Linear in households or affected records |
| Seasonal agricultural output | Field/cell update with weather, soil, labor, and maintenance inputs | Linear in cultivated fields |
| Reciprocal aid and exchange | Bounded partner lists; local offers and requests | Approximately linear in active relationships |
| Migration choice | A small candidate set from memory, kin, and known settlements | Bounded work per reconsideration |
| Tribute and administrative capacity | Sparse institutional graph; cached aggregates | Linear in active organizational links |
| Public projects | Contributor records, resource commitments, work progress | Linear in participants and construction events |
| Global partner matching or unrestricted destination search | Compare every household with every other household or location | Potentially prohibitive; avoid as the default |

For an illustrative **50,000-person** world, an assumed mean household size of five gives **10,000 households**. Checking 16 known partners for each household requires **160,000 candidate checks per exchange pass**, rather than approximately **50 million unordered household pairs**. The household-size assumption is for estimating workload, not a proposed universal demographic constant.

The larger issue is time acceleration. One decision per person per simulated day entails:

\[
50{,}000 \times 365 = 18.25\text{ million decisions per simulated year}.
\]

A century entails **1.825 billion** such decisions, before pathfinding, production, social encounters, and rendering.

At one simulated day per real second, that is 50,000 decisions per second. At one simulated year per real second, it is 18.25 million. **A model that is comfortably real-time at observation speed may be far too expensive for century-scale fast-forward.**

---

## 5. Lessons for TCE: adopt, adapt, and avoid

### Adopt a multilevel world, but maintain one material reality

I recommend four linked levels:

**People** retain identity, relationships, skills, health, obligations, and activities. **Households** coordinate shared reserves, dependents, labor, housing, and land access. **Settlements** contain infrastructure and shared services. **Institutions** hold jurisdiction, membership, assets, obligations, offices, and procedures.

These must be different views of the same underlying world. A household’s harvest should come from work on identifiable fields. A tax transfer must leave one inventory and enter another. A government’s labor levy must remove time from the same people’s other activities.

A useful accounting identity is:

\[
S\_{t+1}=S\_t+\text{production}+\text{received}
-\text{consumed}-\text{transferred}-\text{spoilage}.
\]

Seed commitments, reserves, and already-promised deliveries should constrain spendable stock. Otherwise the political model can appear healthy while consuming resources that the daily-life simulation never produced.

### Adapt ecological models into uncertainty, not environmental destiny

Use yields, reserves, and expected sufficiency to create pressures—not predetermined outcomes.

A shortfall should open several possibilities: draw down stores, request help, borrow, change production, reduce commitments, move, or contest distribution. Which option is available should depend on relationships and institutions.

Include both **local variation** and **regionally correlated shocks**. As a design consequence, exchange can protect a household against an isolated loss without becoming a magical solution when all its partners suffer together.

Likewise, relocation should be constrained by knowledge. Let a household know a few kin destinations, markets, previous residences, and reported opportunities. Expanding that knowledge through contact is more credible—and cheaper—than continuously evaluating the entire map.

### Make institutions solve specific problems

Do not author “village → chiefdom → state” as the underlying progression.

Instead, author a vocabulary of organizational rules: membership, contribution obligations, beneficiary rights, appointment procedures, sanctions, dispute resolution, succession, exit, and dissolution.

A plausible TCE sequence might be:

A group builds a shared granary. Repeated allocation disputes create a custodian role. The custodian needs records and assistants. Contributors disagree over access during a bad harvest. Some propose elected oversight; others support hereditary control; another group leaves.

That sequence is a **design example**, not a historical inevitability. The authored content supplies the possible actions and institutional arrangements; circumstances and decisions determine which occur.

Crucially, the public benefit must come from something real. A coordination rule should not simply multiply everyone’s food. It should enable construction, reduce waste, improve scheduling, maintain irrigation, or organize a transfer that the material simulation actually executes.

### Give administration a workload and a reach

RebeLand’s capacity-versus-issues idea translates well into an inexpensive mechanism:

\[
B\_{t+1}=\max(0,\ B\_t+\text{new cases}-\text{completed cases}),
\]

where \(B\) is an administrative backlog.

Cases could include boundary disputes, theft complaints, maintenance requests, provisioning failures, and appeals. Offices have personnel, work rates, information, priorities, and travel constraints.

This creates reasons to delegate, standardize procedures, keep records, or establish local offices. It also creates failure modes more specific than “stability fell below 30.”

I would keep **service performance, legitimacy, coercive capacity, and coalition support separate**. A government can be competent but distrusted, popular but fiscally weak, or militarily strong but administratively ineffective.

### Separate succession from institutional destruction

An office, its incumbent, and the coalition supporting it should be distinct entities.

When a leader dies, evaluate who is eligible, who recognizes the procedure, what assets and obligations persist, and which groups have both an incentive and the capacity to defect.

This preserves political drama without importing a model assumption that every succession must shatter the state. Conversely, an institution should not survive indefinitely merely because its country entity still exists.

Avoid representing every relationship as part of a single tree. A tributary hierarchy can be a tree while kinship, trade, religious membership, and military alliances remain overlapping networks.

### Keep culture and technology outside a forced era system

For TCE, I would model technologies as capabilities and practices carried by people and organizations: knowledge, tools, production methods, training relationships, and reproducible procedures.

Innovation should involve authored possibilities with requirements and costs, while adoption depends on usefulness, access, compatibility, risk, and social transmission. Losing practitioners or institutions can reduce effective capability even when a chronicle records that the society once possessed it.

Architecture needs a related but separate mechanism: available materials, construction knowledge, labor organization, functional requirements, and learned design preferences. None of the reviewed model families supplies a ready-made, empirically validated generator of arbitrary legal systems and architectural cultures. Kohler’s own discussion of the behavior–culture gap reinforces the need to treat this as a distinct design layer. [simulatingcomplexity](https://simulatingcomplexity.wordpress.com/2015/08/12/tim-kohler-the-nine-questions/)

### Use multirate scheduling rather than making everyone think every frame

The following is a proposed starting schedule, to be adjusted by profiling:

| Process | Suggested trigger |
| --- | --- |
| Rendering and motion interpolation | Render frame |
| Activity completion, arrivals, deliveries, resource transfers | Simulation events |
| Household reserve review and routine commitments | Daily or when conditions materially change |
| Crop planning, harvest, major relocation reconsideration | Seasonal or event-driven |
| Institutional budgets, policy review, relationship maintenance | Monthly or triggered by significant problems |
| Historical summaries and cross-world metrics | Annual or coarser |

Acceleration should change how much detail is displayed and how work is scheduled—not grant off-screen agents free travel, instant construction, or unearned production. Logical journeys can be represented by departure, capacity use, and arrival events without rendering every intermediate position.

Also decide explicitly what **50,000 people means geographically**. A fully enumerated regional society and a representative sample of an empire are different models. Do not compare the former directly against the largest Seshat polities while silently treating each person as thousands of people.

### Make causality inspectable

For important decisions, preserve a small structured explanation record:

> Household moved because expected reserves fell below its target; its preferred destination was unavailable; a relative reported accessible land elsewhere.

For institutional changes, retain the proposal, supporters, opponents, binding constraints, and resulting obligations.

These records should be generated from the actual decision inputs, not invented retrospectively by a narration layer. They will help players understand emergence and help developers distinguish plausible adaptation from bugs.

### Validate mechanisms without tuning every world toward the same history

I recommend a small permanent reference suite alongside procedural worlds.

First, test invariants: resource conservation, valid membership, valid succession, no duplicate ownership, and consistent transfers.

Second, compare mechanisms against simpler baselines. Does exchange improve resilience relative to no exchange? Does the full household model explain more than environmental capacity alone? Does a proposed administrative system improve useful outcomes rather than merely adding offices?

Third, compare several historical-style patterns at the appropriate resolution, using parameter ranges and multiple seeds. Hold out some environmental sequences or regions rather than repeatedly tuning against the same curve.

Finally, keep **historical plausibility tests separate from entertainment targets**. A world need not develop a state to be correct. But a world in which cooperation never pays, all institutions instantly collapse, or every seed converges to the same arrangement probably exposes a design problem.

---

## 6. Linked source guide

The sources below are the most useful starting points for implementation and critical reading.

| Resource | Why read it |
| --- | --- |
| [Sugarscape model analysis](https://jasss.soc.surrey.ac.uk/12/1/6/appendixB/EpsteinAxtell1996.html?utm_source=chatgpt.com) and [NetLogo wealth model](https://ccl.northwestern.edu/netlogo/models/Sugarscape3WealthDistribution?utm_source=chatgpt.com) | Small, inspectable rules and the assumptions behind their aggregate outputs. |
| [Kehoe, *The Specification of Sugarscape*](https://arxiv.org/abs/1505.06012?utm_source=chatgpt.com) | Replication ambiguity, rule definition, and update semantics. |
| [Axtell et al., original Artificial Anasazi study](https://doi.org/10.1073/pnas.092080799) | The original archaeological modeling claim. |
| [Janssen, *Understanding Artificial Anasazi*](https://jasss.soc.surrey.ac.uk/12/4/13.html?utm_source=chatgpt.com) | The highest-priority methodological postmortem in this report. |
| [VEP leadership paper](https://doi.org/10.1142/S0219525911003256) and [author preprint](https://sfi-edu.s3.amazonaws.com/sfi-edu/production/uploads/sfi-com/dev/uploads/filer/00/e4/00e460a1-37b7-4355-a41f-3940276e434a/11-07-025.pdf?utm_source=chatgpt.com) | A concrete bridge from household subsistence to collective organization. |
| [Crabtree et al., *How to Make a Polity*](https://doi.org/10.1017/aaq.2016.18) | Political organization tested against multiple archaeological patterns. |
| [MASON documentation](https://people.cs.gmu.edu/~eclab/projects/mason/?utm_source=chatgpt.com) and [Hierarchies chapter](https://socialcomplexity.gmu.edu/wp-content/uploads/2017/03/Cioffi-Revilla-et-al.Hierarchies-2015.pdf?utm_source=chatgpt.com) | Framework architecture and explicit political relationships. |
| [RebeLand author presentation](https://pdfs.semanticscholar.org/3946/7857b927e646669623dd2ec4d2bd3cba2904.pdf?utm_source=chatgpt.com) | Governance capacity, issue accumulation, and instability feedbacks. |
| [Gavrilets–Anderson–Turchin polity model](https://escholarship.org/uc/item/5536t55r?utm_source=chatgpt.com) | Compact conquest, tribute, control, and fragmentation mechanisms. |
| [Seshat data](https://seshatdatabank.info/data?utm_source=chatgpt.com), [methods](https://seshatdatabank.info/how-we-work?utm_source=chatgpt.com), and [2018 complexity analysis](https://doi.org/10.1073/pnas.1708800115) | Data access, coding discipline, and comparative structural patterns. |
| [Information-processing thresholds paper](https://doi.org/10.1038/s41467-020-16035-9) | A basis for exploring administrative and informational bottlenecks. |
| [Kohler interview](https://simulatingcomplexity.wordpress.com/2015/08/12/tim-kohler-the-nine-questions/?utm_source=chatgpt.com) and [VEP video conversations](https://crowcanyon.org/vep/ch1/ch1q01.html?utm_source=chatgpt.com) | Developer perspective and accessible explanations of the research program. |

**Recommended implementation order:** household food and labor accounting; storage and relocation; bounded reciprocal exchange; one material collective project; competing governance arrangements; administrative workload; then inter-settlement political relationships. Add each layer only when its contribution can be distinguished from the simpler model beneath it.

That approach gives TCE a foundation for surprising history without making surprise itself the explanation.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927de-7d50-83e9-ade2-905996e0deb7)
