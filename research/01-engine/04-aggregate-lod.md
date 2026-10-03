# Reversible settlement aggregation for The Civilization Engine

**Engineering report — evidence reviewed through September 27, 2026**

## Executive recommendation

TCE should **aggregate computation without discarding the persistent population**.

The best fit is a **stochastic cohort model layered over lightweight person, household, institution, and asset records**. Keep expensive daily activity simulation local; calculate routine off-screen flows in batches; continue to represent consequential events—births, deaths, migration, succession, construction, technological discoveries—as changes to identifiable entities.

Use three simulation modes:

1. **Detailed individuals:** movement, activities, local interactions, and visible daily life.
2. **Lightweight individuals:** persistent people and households, but event-driven or daily updates without detailed movement.
3. **Cohort-driven settlements:** routine transitions calculated for groups, with explicit individuals for exceptional or history-sensitive interactions.

Calibrate **conditional transition rates and missing interaction effects** from the detailed simulation. Do not begin with a black-box model that predicts an entire settlement decades ahead.

The central distinction is between **preserving history** and **reconstructing plausible detail**. Population synthesis can recover a population matching distributions; it cannot recover deleted marriages, debts, friendships, or ownership. TCE should preserve those facts rather than attempt to regenerate them.

---

## 1. Options: what the main techniques actually provide

### 1.1 The mathematical contract: aggregation is not reversible by itself

Let \(X\) be a detailed settlement, \(C=R(X)\) its aggregate description, and \(H\) the historical and identity information retained separately. A lifting operation constructs:

\[
X'=L(C,H,\xi),
\]

where \(\xi\) supplies randomness for unspecified detail.

The important consistency condition is:

\[
R(L(C,H,\xi))=C
\]

for hard constraints such as population and inventory counts, with explicit tolerances for approximate distributional constraints. This restriction/lifting formulation is a standard foundation of equation-free computation. It does **not** imply that lifting recovers the original microscopic state. [arXiv](https://arxiv.org/html/physics/0209043v1)

For example, “400 adults, 120 children, 200 households” does not identify which adults are married, which children belong to them, or who owns the granary. Many incompatible settlements satisfy those totals.

TCE therefore needs three separate guarantees:

* **Historical continuity:** retained identities, relationships, property, and completed events never change merely because resolution changes.
* **Statistical consistency:** detailed and aggregate modes produce sufficiently similar distributions of relevant future outcomes.
* **Presentation continuity:** reconstructed locations and activities agree with the current time, buildings, jobs, and ongoing events.

Exact counterfactual trajectory equality—“the same person would have died on the same day had the settlement remained detailed”—is a much stronger requirement and should not be the default acceptance criterion.

### 1.2 Technique comparison

The complexity and maturity assessments below are engineering judgments for TCE, not measured performance rankings.

| Technique | How it works | Principal benefit | Main limitation | Fit for TCE |
| --- | --- | --- | --- | --- |
| **Lightweight individual simulation** | Retains every person but skips movement and processes only daily or scheduled changes. | Easiest identity continuity and debugging. | Still pays for individual or household updates. | Essential baseline and fallback. |
| **Cohort-component / compartment model** | Stores counts in demographic or behavioral states and advances flows between them. | Cost can depend mainly on occupied cohorts rather than people. | Missing correlations and history can invalidate transition rates. | Recommended aggregate core. |
| **Weighted representative agents** | A smaller agent population carries weights representing multiple people. | Reuses some individual rules and heterogeneity. | Weighting changes discreteness, variance, and interaction structure. | Useful experimentally; poor identity backbone. |
| **Learned surrogate or emulator** | Learns outputs or state transitions from detailed simulation runs. | Cheap evaluation after training. | Extrapolation, constraint violations, and long-rollout drift. | Use small, constrained submodels first. |
| **Equation-free integration** | Lifts coarse states, runs short microscopic bursts, restricts results, and projects forward. | Uses the existing detailed simulator without deriving every coarse equation. | Requires suitable coarse variables and rapidly relaxing omitted detail. | Excellent diagnostic approach; risky initial runtime architecture. |
| **Heterogeneous multiscale method** | A macro solver requests missing rates or constitutive information from micro simulations. | Keeps an interpretable macro structure while using micro-level evidence. | Designing reliable micro/macro coupling is substantial work. | Strong architectural inspiration. |

Cohort-component methods are established demographic machinery; weighted-agent rescaling has operational examples such as Covasim; machine-learning calibration surrogates and equation-free/HMM methods have substantial research precedents. None automatically supplies TCE’s economic, political, architectural, and identity semantics. [Census](https://www2.census.gov/programs-surveys/popproj/technical-documentation/methodology/methodstatement23.pdf)

### 1.3 Cohort-component and compartment models

A demographic cohort-component model advances age cohorts using births, deaths, and migration. The U.S. Census Bureau’s 2023 methodology provides a concrete, documented implementation of that general structure. TCE should borrow the accounting structure—not modern demographic assumptions. [Census](https://www2.census.gov/programs-surveys/popproj/technical-documentation/methodology/methodstatement23.pdf)

For TCE, extend the idea into a **stochastic transition network**:

\[
\mathbf N\_{t+\Delta}=\mathbf N\_t+S\mathbf K.
\]

Here, \(\mathbf N\) contains integer cohort populations; each column of \(S\) describes a transition; and \(\mathbf K\) contains sampled event counts.

Transitions might include becoming an apprentice, changing occupation, joining a faction, recovering from illness, emigrating, or dying. Production and consumption need corresponding stock-flow accounting rather than being hidden inside a population-growth equation.

The difficult part is choosing the state representation. A settlement-wide mean wealth and mean ideology are inadequate when outcomes depend on concentration or polarization. Two equally wealthy settlements can differ radically when one has independent farmers and the other has indebted tenants.

**Recommended extension:** retain selected joint distributions and organizational structure, not every possible cross-product of attributes. Keep households, landowners, workshops, political organizations, and influential individuals explicit where they mediate important decisions.

### 1.4 Surrogates: distinguish calibration tools from runtime replacements

An **output surrogate** approximates something like:

\[
\text{parameters}\rightarrow\text{growth rate, mortality, or probability of collapse}.
\]

That is useful for exploring and calibrating the detailed simulation.

A **runtime state emulator** must instead approximate the next-state distribution conditional on the current population, resources, institutions, environment, and relevant history. It must also remain stable when repeatedly applied.

Lamperti, Roventini, and Sani demonstrate machine-learning surrogates for **parameter-space exploration and calibration**, including an endogenous-growth ABM. Their results are not a demonstration of preserving individual histories during long-horizon runtime replacement. [arXiv](https://arxiv.org/html/1703.10639v2)

For TCE, start with lookup tables, small regression models, or compact decision-tree models for specific quantities: effective work completion, conditional migration, contact intensity, or apprenticeship success. Keep resource conservation and event eligibility outside the learned model.

A neural state emulator is an option later, but adds training infrastructure, uncertainty estimation, constrained decoding, and difficult failure diagnosis.

### 1.5 Equation-free and heterogeneous multiscale methods

**Equation-free computation** follows a cycle: construct detailed realizations consistent with coarse state; allow initialization artifacts to relax; run short microscopic simulations; measure coarse evolution; then take a larger coarse step. Its attraction is reusing the detailed simulator. Its crucial assumption is that the retained coarse variables adequately describe the slow dynamics. [arXiv](https://arxiv.org/html/physics/0209043v1)

For TCE, this is most valuable as a **closure test**: do settlements with the same proposed aggregate description subsequently behave similarly? Long-lived feuds, ownership concentration, or network segregation may prevent the omitted detail from “relaxing away.”

**HMM** starts with a macro-level solver and uses microscopic calculations to supply missing information. The original formulation explicitly exploits scale separation where available. [Princeton University](https://collaborate.princeton.edu/en/publications/the-heterogeneous-multiscale-methods)

A practical TCE adaptation would keep explicit demographic and material accounting while using selected detailed runs to estimate, for example, how a particular settlement layout limits hauling or market access. That is safer than projecting every political and economic variable forward from a short burst.

### 1.6 Population synthesis: IPF, household fitting, and Bayesian networks

Population synthesis solves **plausible reconstruction**, not historical recovery.

**Iterative proportional fitting, or IPF**, repeatedly adjusts a seed contingency table to match specified marginals. For TCE, the seed should come from detailed TCE settlements or authored household templates. Multiplicative fitting cannot create missing support from a zero seed cell; distinguish genuinely impossible combinations from combinations merely absent in the sample. Ordinary IPF also does not, by itself, solve simultaneous household/person consistency. Household-aware methods such as IPU, hierarchical fitting, and generalized raking address that additional problem. [JASSS](https://www.jasss.org/24/2/5.html)

**Bayesian networks** represent a joint distribution through conditional dependencies. They can generate a person’s attributes conditional on household and settlement context, rather than drawing occupation, wealth, age, and housing independently. Sun and Erath’s population-synthesis work develops this approach and discusses hierarchical household generation and subsequent fitting to zonal controls. [ResearchGate](https://www.researchgate.net/publication/282815687_A_Bayesian_network_approach_for_population_synthesis)

For TCE, combine the strengths:

**Generate plausible household/person candidates → fit to aggregate controls → integerize → validate relationships and resource allocations.**

Sampling from a probabilistic model does not guarantee exact finite-population totals. Conversely, matching marginals does not guarantee realistic joint structure. Integerization and constrained allocation are separate engineering steps, not cosmetic rounding.

For an initial implementation, **resampling valid household templates from TCE runs is simpler than learning a large Bayesian network**. Use synthesis only for genuinely unspecified information; never use it to overwrite established family or property records.

---

## 2. Trade-offs and performance evidence

### Published numbers—and what they measure

| Evidence | Reported result | Appropriate interpretation |
| --- | --- | --- |
| **Lamperti et al., 2017 preprint v2** | Approximately **500×** faster prediction for the Brock–Hommes calibration task and **3,750×** for the Island-model task. | Fast prediction of calibration outputs; not an end-to-end, identity-preserving settlement simulation speedup. Training-data generation remains a cost. [arXiv](https://arxiv.org/html/1703.10639v2) |
| **Covasim, 2021 paper** | Roughly **7 million simulated person-days per CPU-second**, single-core on an **i9-8950HK**; about **1 KB per agent**. | Evidence that array-based individual simulation can already be efficient. Its epidemic workload is much narrower than TCE’s. [PLOS](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1009149) |
| **PopulationSim v0.10.0 release notes** | **5×–10×** improvement from rewriting list-balancer functions with Numba. | A maintainer-reported speedup for particular synthesis routines—not overall population generation, promotion latency, or simulation throughput. [GitHub](https://github.com/ActivitySim/populationsim/releases) |

**No source reviewed provides an end-to-end benchmark for TCE’s full combination of demographics, economic production, ideological change, persistent families, settlement reconstruction, and UE rendering.** Assigning a universal “100× aggregation speedup” would not be justified.

### Where the computational savings come from

For the proposed design, aggregate cost depends on occupied cohorts, modeled interactions, household/institution work, and events applied to persistent identities. Dense cohort-to-cohort mixing can become quadratic; materializing many historical events can dominate even when routine population updates are cheap.

Keeping individual records therefore does not imply updating every individual every simulation tick. Equally, preserving every birth and inheritance means a century of history cannot always be generated in constant time.

Switching overhead matters. If detailed and coarse modes cost \(T\_f\) and \(T\_c\) per simulated day, and conversion costs \(C\_{\downarrow}+C\_{\uparrow}\), aggregation becomes worthwhile only after approximately:

\[
D>\frac{C\_{\downarrow}+C\_{\uparrow}}{T\_f-T\_c}
\]

days of coarse residence.

This argues for **hysteresis, minimum residence times, and preloading**, rather than changing simulation mode whenever the camera crosses a boundary.

---

## 3. Precedents: what shipped systems actually establish

### Dwarf Fortress: the closest population-resolution precedent

Tarn Adams’s December 2013 development diary explicitly describes non-historical populations moving among three resolutions: active play, abstract building/dungeon populations, and abstract site populations. It also describes individual members being elevated to historical status. The work was motivated partly by population duplication bugs. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2013.html)

The 2014 release history shows why semantic parity matters: version **0.40.16** addressed attackers always winning post-world-generation non-player battles. That is a historical example of a coarse-resolution rule producing fundamentally different outcomes—not a statement about current Dwarf Fortress behavior. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2014.html)

**Lesson for TCE:** preserve historically important entities across resolutions, but treat conversion bookkeeping and cross-resolution rule equivalence as first-class systems. Dwarf Fortress is not evidence that arbitrary aggregate statistics can reconstruct a complete prior population.

### X4: reduced-attention rules need their own validation

X4 provides a precedent for continuing object-level simulation with different attention-dependent calculations. Egosoft’s **4.10** release notes include changes to low-attention capital-ship combat and hazardous regions operating when the player is absent. [Steam Store](https://store.steampowered.com/news/posts/?appids=392160&enddate=1639052654&feed=steam_community_announcements)

A particularly useful developer-confirmed 2020 incident concerned ships initially built without weapons and equipped later: Egosoft identified that sequence as a case needing a fix for low-attention combat. The surrounding investigation concerned stale firing-range information. [Egosoft Forum](https://forum.egosoft.com/viewtopic.php?t=425872)

**Lesson for TCE:** reduced-mode coefficients and caches must be invalidated when underlying capabilities change. A settlement gaining bridges, tools, laws, or a new production process must not continue using obsolete aggregate assumptions.

X4 is a precedent for attention-dependent simulation, **not evidence of a learned demographic cohort model with reversible person synthesis**.

### Mount & Blade: persistent heroes alongside counted populations

Bannerlord’s **1.2.12** API exposes a `TroopRoster` with troop counts, wounded counts, experience, separate hero totals, and conversion to a flattened roster. Its `Hero` API retains parents, spouses, children, and owned workshops. [Bannerlord API](https://apidoc.bannerlord.com/v/1.2.12/class_tale_worlds_1_1_campaign_system_1_1_roster_1_1_troop_roster.html)

This demonstrates a useful representational split: **counted ordinary populations plus persistent exceptional people**.

**Lesson for TCE:** the pattern is appropriate for named political actors and military leaders. But a troop-count representation is not sufficient when every ordinary resident must retain a continuous biography.

### Scientific and open-source precedents

Covasim’s dynamic rescaling shows how weighted representatives can control simulation cost, while documenting problems with event granularity and stochastic variability. Its resampling deliberately changes which agent records represent the population. That is precisely why it should not be copied as TCE’s identity-preservation mechanism. [PLOS](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1009149)

PopulationSim is a more relevant implementation reference for the **reconstruction pipeline**: balancing, household/person controls, and integerization. Its documentation and release history are useful starting points, but the published documentation currently labels itself **0.5.1**, while the repository exposes **v0.10.0**; pin the implementation being studied. [ActivitySim](https://activitysim.github.io/populationsim/)

---

## 4. Recommended architecture for TCE

The following is a proposed engineering design, not a claim that an existing engine implements it wholesale.

### 4.1 Keep a persistent core independent of resolution

Maintain lightweight records for every established person: stable identity, birth/death information, household, important relationships, persistent traits, occupation/skill state, and references to consequential events.

Keep household and institutional records separately: membership, property, stores, debts, obligations, offices, factions, laws, and production organizations. Retain buildings, land parcels, transport connections, unique objects, and construction projects in all modes.

**Memory is not the strongest argument for deleting identities.** As an illustrative allocation—not a measured TCE layout—a 256-byte base record costs approximately 25.6 MB for 100,000 people or 256 MB for one million. Relationships, indexes, inventories, and history add overhead, but these figures support testing persistent records before adopting irreversible compression.

The likely architectural saving comes from eliminating expensive activity planning and spatial interaction work, not from eliminating a person’s ID.

Use generational handles for runtime storage, but never reuse a historical identity as though it were the same person. Historical IDs and reusable storage slots should be different concepts.

### 4.2 Make resolution a property of execution, not existence

**Detailed mode** runs activity selection, movement, spatial contacts, and local work.

**Lightweight mode** retains people and households while resolving a day or scheduled event without simulating its entire spatial sequence. This is both the fallback mode and the reference against which cohort benefits should be measured.

**Cohort mode** calculates routine event counts for groups, then applies identity-sensitive results to eligible records. Keep membership indexes so that aggregate events can select actual people without scanning the entire settlement.

For example, if a cohort produces three migration events, select three eligible residents or households, debit the source, create traveling entities, and credit the destination only on arrival. Do not decrement one town today and independently synthesize unrelated immigrants elsewhere tomorrow.

Initially switch whole settlements. Add district-level aggregation only after settlement boundaries and traveler accounting are reliable.

### 4.3 Represent the variables that determine behavior

A useful initial representation is:

| Domain | Aggregate information | Information that should remain explicit |
| --- | --- | --- |
| Demography | Birth cohorts, life stages, household composition, health-state counts | Established parentage, pregnancies, deaths, dependents |
| Economy | Available labor by skill, production capacity, food-access groups, selected wealth distributions | Ownership, debts, contracts, inventories, scarce specialists |
| Ideology | Faction membership, belief mixtures, commitment distributions, selected wealth/belief correlations | Leaders, organizations, important commitments and conflicts |
| Interaction | District/workplace mixing and travel-access summaries | Households, important social edges, travelers, exceptional encounters |
| Built environment | Capacity, accessibility, utilization, maintenance demand | Parcels, buildings, streets, construction state, architectural recipes |

Avoid a giant Cartesian product. Even a modest combination of 20 age groups, two demographic categories, 12 occupations, five wealth bands, six ideological groups, and four health states creates **57,600 possible cells**.

Prefer sparse occupied groups and adaptive subdivision. Split a cohort when its members demonstrably require different transition probabilities—not merely because another attribute exists.

**Preserve rare capabilities explicitly.** Three master smiths are not well represented by “average smithing skill 0.003.” The same applies to unique knowledge carriers, disputed heirs, and people whose relationships connect otherwise separate communities.

### 4.4 Use stochastic, constrained transitions—not independent averages

Calculate event counts with finite-population sampling. Under approximately constant competing hazards \(\lambda\_r\), the probability of the first event being type \(r\) within \(\Delta\) is:

\[
p\_r=\frac{\lambda\_r}{\Lambda}\left(1-e^{-\Lambda\Delta}\right),
\qquad
\Lambda=\sum\_r\lambda\_r.
\]

Sampling mutually exclusive outcomes together prevents independently selecting the same resident to die, migrate, and change occupation during the same unresolved interval. Where multiple sequential transitions matter, shorten the interval or process the relevant events explicitly.

Preserve duration information. Aging, pregnancy, apprenticeship, and construction should not all become memoryless transitions. Track start dates, stages, or scheduled completion events.

Draw shared shocks at their appropriate scope. One drought should affect related farms together; independently drawing a “drought effect” for every household would alter the variance and spatial correlation of outcomes.

For economic production, conserve inputs and outputs through a common transaction system. Allocate constrained labor and materials before committing production. Avoid sampling impossible production and then silently clamping negative stocks: the repair can introduce systematic bias.

### 4.5 Preserve named characters without giving them special physics

A persistent or “pinned” person should remain individually addressable, not immune to aggregate hazards.

There are two coherent implementations:

**Explicit exceptions:** remove the person from cohort sampling and evaluate their events individually using compatible rates.

**Identity-preserving cohort membership:** retain the person in the cohort, but select actual member IDs when events occur.

Never combine the two for the same event channel. Otherwise the named character may be counted twice—or excluded from mortality entirely.

Pinning also need not activate expensive movement. A distant ruler can remain an explicit, event-driven individual.

Births must acquire identity and necessary family links when they become causally relevant. A child who inherits property during a thirty-year skip cannot wait until the final reconstruction to receive parents. Appearance details may remain unmaterialized; legally or historically consequential facts cannot.

### 4.6 Make promotion and demotion transactions

**Demotion.** Reach a safe simulation boundary. Finish or serialize ongoing activities, reservations, shipments, pregnancies, and construction. Separate explicit exceptions, build the cohort summaries, and verify counts and ledgers. Only then transfer authority from the detailed executor.

**Promotion.** Bring the settlement’s persistent records to the promotion timestamp. Restore established residents first. Synthesize only unspecified population/detail, subject to the remaining controls. Assign housing, jobs, resources, and locations consistently with retained buildings and the current time.

**Commit.** Construct the new execution representation privately, validate it, and swap authority atomically. The outgoing representation must stop contributing at exactly that point.

Useful transition invariants include:

\[
N\_{\text{settlement}}
=
N\_{\text{cohort-managed}}
+
N\_{\text{explicit}},
\]

with the two sets disjoint; one ownership claim per uniquely owned asset; and exactly one debit/credit sequence per transfer.

Do not run a “warm-up day” after promotion that consumes food, advances disease, or earns income a second time. Initialization may reconstruct routine phases and positions, but any genuine simulation time must be accounted for once.

Preload promotion incrementally to avoid a frame hitch, while retaining one authoritative executor until the transition commits.

### 4.7 Calibration: learn from TCE, then test what was lost

**Build a headless experimental harness first.** Run detailed settlements across population sizes, geography, seasons, resource shortages, inequality, political structures, disease, migration, and technological capabilities. Include actual saved settlements and deliberately constructed stress cases.

Split training and validation by whole worlds and regimes, not adjacent time windows from the same run.

**Test closure before fitting a sophisticated model.** Find detailed settlements with similar retained aggregate states, then compare their future distributions under the same external conditions. If they diverge because of hidden ownership or social structure, add that state or retain more explicit simulation. A more powerful predictor cannot recover information absent from its inputs.

**Fit conditional processes rather than an entire historical trajectory.** Examples include apprenticeship completion given instructors and work availability; migration given food access and destinations; or work completion given layout and transport capacity. Share authored eligibility rules and production recipes between modes.

Use controlled interventions in detailed experiments—changing a road, rationing rule, or labor allocation—not only passive observations. A predictor that reproduces ordinary runs may still respond incorrectly to a new law.

**Validate at several horizons.** Check days, seasons, years, and decades. Evaluate distributions, tails, correlations, event frequencies, and intervention responses—not only mean population and total production. Set explicit acceptable effect sizes; “no statistically significant difference” alone is not an equivalence criterion.

Add repeated round-trip tests, including rapid switching, save/load during coarse execution, mixed-mode trade, extinction, mass migration, inheritance, and content updates.

Finally, version each fitted model with the simulation rules and authored content it assumes. A new technology or legal mechanism can invalidate old calibration. Outside a validated regime, reduce the coarse timestep or fall back to lightweight individuals.

**Calibration to the detailed model establishes internal consistency, not historical realism.** Those are separate validation tasks.

### 4.8 Use the same machinery for multidecade skips

A time skip should change the scheduler and resolution policy, not invoke a separate history generator.

Advance toward the next relevant boundary: harvest, delivery, stock depletion, succession, construction completion, or another scheduled event. Resolve stochastic events within those intervals, update persistent records, then reassess the next interval.

Do not jump across thirty years using one average growth rate. Doing so would miss feedback such as a failed harvest causing migration, which removes skilled labor, which delays irrigation, which worsens the next harvest.

Keep settlements synchronized at interaction boundaries. A shipment must not leave a settlement in year 120 and arrive in another settlement that is still executing year 115 without an explicit synchronization scheme.

For history, store consequential events and causal references; do not log every skipped meal. Use checkpoints and a documented retention policy so centuries of history do not create unbounded high-detail logs.

As an illustrative budget, thirty 365-day years contain 10,950 daily steps. A measured **whole-world** aggregate step of 1 ms would imply roughly 11 seconds; 10 ms implies roughly 110 seconds, before promotion, saving, and other overhead. Benchmark that quantity directly.

### 4.9 Rust DLL, Unreal Engine 5.8, and the target hardware

Keep the Rust kernel authoritative and expose a narrow C ABI. Rust’s `cdylib` output is intended for dynamic libraries consumed by other languages. Use versioned interfaces, opaque handles, fixed-width fields, and explicit ownership rules rather than passing Rust containers across the boundary. Contain panics and C++ exceptions rather than allowing them to cross unexpectedly. [Rust Documentation](https://doc.rust-lang.org/reference/linkage.html)

Use an Unreal C++ adapter for DLL loading, dependency staging, commands, and snapshots; Epic documents the third-party-library integration path in its **5.8** documentation. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine)

Run simulation work independently of rendering and publish immutable snapshots. MassEntity can organize data-oriented frontend entities, but it is not itself an aggregate demographic model. Do not create a heavyweight Unreal object for every off-screen resident merely to mirror Rust state. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-entity-in-unreal-engine)

For the specified machine, start with CPU cohort execution and reserve GPU capacity for rendering. Leave CPU headroom for Unreal rather than saturating every available worker.

The 60 fps target provides **16.67 ms per frame overall**, not a 16.67 ms simulation allowance. Measure integrated frame-time tails and promotion hitches alongside headless simulated-days-per-second. No reviewed benchmark establishes 60 fps at 1440p for TCE’s eventual scene complexity.

### 4.10 Implementation order for a solo developer

Build the persistent identity/ledger layer and transition invariants first. Then implement lightweight daily individuals as the reference fallback.

Next, aggregate one narrow subsystem—such as routine food consumption and production—while running detailed/coarse comparison tests. Add demographic transitions and constrained reconstruction only after that boundary is reliable.

Finally, expand into ideology and institutional change, then multidecade skips. Give AI coding agents bounded modules with executable invariants; keep the cross-mode semantic contract under one coherent design.

**Do not make the first milestone “train a settlement neural network.” Make it “switch a settlement repeatedly without changing who exists, who owns what, or how much food is present.”**

---

## 5. Sources and version scope

These are the most useful primary references to keep alongside the implementation.

| Reference | Version/date and reason to consult |
| --- | --- |
| [Kevrekidis et al., *Equation-Free Multiscale Computation*](https://arxiv.org/abs/physics/0209043?utm_source=chatgpt.com) | 2002 preprint; restriction, lifting, microscopic bursts, and coarse integration. |
| [E and Engquist, *The Heterogeneous Multiscale Methods*](https://collaborate.princeton.edu/en/publications/the-heterogeneous-multiscale-methods/?utm_source=chatgpt.com) | 2003, *Communications in Mathematical Sciences*; macro/micro coupling and scale separation. |
| [Lamperti, Roventini, and Sani, *Agent-Based Model Calibration using Machine Learning Surrogates*](https://arxiv.org/html/1703.10639v2?utm_source=chatgpt.com) | 2017 preprint v2 used here; calibration methodology and scoped speedup results. |
| [U.S. Census Bureau, 2023 projection methodology](https://www2.census.gov/programs-surveys/popproj/technical-documentation/methodology/methodstatement23.pdf?utm_source=chatgpt.com) | 2023; concrete cohort-component accounting. |
| [Sun and Erath, *A Bayesian network approach for population synthesis*](https://doi.org/10.1016/j.trc.2015.10.010) | 2015; conditional population generation and household extensions. |
| [*Generating a Two-Layered Synthetic Population for French Municipalities*](https://www.jasss.org/24/2/5.html?utm_source=chatgpt.com) | JASSS 2021; household/person fitting and integerization comparisons. |
| [PopulationSim documentation](https://activitysim.github.io/populationsim/?utm_source=chatgpt.com) · [code and releases](https://github.com/ActivitySim/populationsim/releases?utm_source=chatgpt.com) | Documentation labeled 0.5.1; v0.10.0 release examined separately. |
| [Covasim paper](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1009149&utm_source=chatgpt.com) · [code](https://github.com/institutefordiseasemodeling/covasim) | 2021 paper supplies the cited rescaling method and benchmarks; do not assume current code has identical defaults. |
| [Dwarf Fortress 2013 diary](https://www.bay12games.com/dwarves/dev_2013.html?utm_source=chatgpt.com) · [2014 releases](https://www.bay12games.com/dwarves/dev_2014.html?utm_source=chatgpt.com) | Historical implementation evidence; particularly December 15, 2013 and release 0.40.16. |
| [X4 developer-confirmed low-attention issue](https://forum.egosoft.com/viewtopic.php?t=425872&utm_source=chatgpt.com) · [4.10 release archive](https://store.steampowered.com/news/posts/?appids=392160&enddate=1639052654&feed=steam_community_announcements&utm_source=chatgpt.com) | 2020 incident and 2021 release; evidence of reduced-mode correctness hazards, not current balance formulas. |
| [Bannerlord TroopRoster](https://apidoc.bannerlord.com/v/1.2.12/class_tale_worlds_1_1_campaign_system_1_1_roster_1_1_troop_roster.html?utm_source=chatgpt.com) · [Hero](https://apidoc.bannerlord.com/v/1.2.12/class_tale_worlds_1_1_campaign_system_1_1_hero.html?utm_source=chatgpt.com) | API version 1.2.12; counted troops versus persistent characters. |
| [MassEntity overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-entity-in-unreal-engine?utm_source=chatgpt.com) · [third-party integration](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com) | UE 5.8 documentation reviewed. |
| [Rust linkage](https://doc.rust-lang.org/reference/linkage.html?utm_source=chatgpt.com) · [Rust FFI](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | Current official documentation reviewed September 2026; pin TCE’s compiler and dependencies separately. |
| [GDC: *Practices in Procedural Generation*](https://www.gdcvault.com/play/1023372/Practices-in-Procedural?utm_source=chatgpt.com) | Tanya Short and Tarn Adams, GDC 2016; supplementary design talk, not a cohort-performance benchmark. [GDC Vault](https://www.gdcvault.com/play/1023372/Practices-in-Procedural) |

**Bottom line:** TCE’s most credible path is a persistent world with interchangeable execution models. Preserve identities and causal facts; aggregate routine work; synthesize only what was genuinely unspecified; and make statistical disagreement an observable, tested property rather than something hidden by the camera.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92762-5fd8-83ea-b9e7-ea3e47c7e197)
