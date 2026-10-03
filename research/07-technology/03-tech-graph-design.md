# Designing TCE’s technology graph

## Executive recommendation

**Build a graph of practical capabilities, not a sequence of civilization-wide upgrades.** Its 150–250 authored nodes should describe things people can learn to do—fire durable containers, operate rotary machinery, keep accounts—and connect to executable recipes, construction methods, occupations, and services.

Keep three distinctions fundamental:

1. **Discovery is not adoption.** Knowing a method does not supply its materials, skilled workers, infrastructure, or political permission.
2. **A capability is not one product.** The same capability should support several applications; many products should combine several capabilities.
3. **Different discovery orders are not enough.** Societies must also find different combinations worth using.

The most useful synthesis for TCE is **Civilization’s readable prerequisite grammar, Old World’s contingent opportunities, Stellaris’s context-sensitive availability, and Victoria 3’s production-method tradeoffs**—but with knowledge carried by people and organizations rather than owned by a national research queue.

The comparisons below distinguish documented mechanics and developer findings from my proposed TCE design. Historical development-diary numbers are identified as such, not presented as current, expansion-complete inventories.

---

## 1. What the existing games demonstrate

### Civilization: readable dependencies, but broad bundles and predictable destinations

**Civilization IV already uses a graph rather than a simple tree.** Its prerequisites include AND relationships—Compass requires Sailing and Iron Working—and OR relationships—Animal Husbandry can follow Hunting or Agriculture. Nodes bundle heterogeneous rewards: units, buildings, improvements, civic options, and actions. Soren Johnson described selecting approximately 80 important innovations and connecting them before assigning the game’s unlocks. This is useful evidence that flexible prerequisite logic and concrete content can coexist. [Game Studies](https://gamestudies.org/1201/articles/tuur_ghys)

Its limitation is not simply linearity. Even an interwoven graph can imply questionable historical necessities through long dependency chains. Ghys’s analysis shows how dependencies intended to create progression can inadvertently make particular religious or political developments necessary ancestors of unrelated later technologies. **Local plausibility of each edge does not guarantee plausibility of the entire ancestry.** [Game Studies](https://gamestudies.org/1201/articles/tuur_ghys)

Civilization VI offers two additional experiments:

**Activity-conditioned research.** Eurekas make actions relevant to technological progress. Babylon takes this especially far: its civilization ability makes Eurekas supply the entire science requirement of technologies while reducing ordinary science generation. This establishes a different way to pursue the graph, but the underlying mechanism remains completion of authored triggers, not the emergence of practical knowledge among specialists. [Steam Store](https://store.steampowered.com/app/1388850/Sid_Meiers_Civilization_VI_Babylon_Pack/?cc=us&l=malay)

**Randomized graph layout.** The August 2020 update’s Tech and Civic Shuffle rearranges costs and prerequisites within historical eras. The resulting trees are shared by all players, with nodes hidden until a prerequisite is acquired. This disrupts memorized routes, but it changes the map of progression rather than the economic meaning of its destinations. [SteamDB](https://steamdb.info/patchnotes/5469056/)

**Lesson for TCE:** adopt readable AND/OR logic and activity-sensitive opportunities. Avoid arbitrary achievement triggers that instantly grant society-wide competence, and audit transitive prerequisites—not only individual links.

### Old World: uncertainty with memory and opportunity cost

Old World’s technology deck draws **four cards**. The player chooses one; rejected cards enter a discard pile and do not return until the deck cycles. Newly unlocked technologies also enter the discard pile. This differs materially from repeatedly drawing from the entire available pool: rejecting an opportunity has a persistent consequence. Bonus cards offer immediate benefits instead of permanent technological progress and are removed after their opportunity passes, preventing them from indefinitely cluttering the deck. [DESIGNER NOTES](https://www.designer-notes.com/old-world-designer-notes-4-the-technology-deck/)

In his GDC 2022 postmortem, Johnson explicitly identified memorized “golden paths” as the problem this addressed. The design substitutes a succession of contingent choices for perfect control over a long research itinerary. [DESIGNER NOTES](https://www.designer-notes.com/my-elephant-in-the-room-part-2/)

There is also a useful counterpoint. A reader responding to Johnson’s design notes argued that a narrow frontier, large hand, and redraw abilities made the desired technology reappear quickly. This is one player’s observation, not a population-level evaluation, but it identifies a testable issue: **uncertainty depends on the ratio of available opportunities to opportunities considered**, not merely on adding randomness. [DESIGNER NOTES](https://www.designer-notes.com/old-world-designer-notes-4-the-technology-deck/)

**Lesson for TCE:** borrow temporary opportunities and consequences of passing them up, not literal cards. A visiting artisan, patronage offer, unusual material sample, or workshop collaboration can create an opportunity that later disappears for an intelligible reason.

Do not make a competent millwright forget how to investigate milling because the relevant “card” is unavailable.

### Stellaris: weighted opportunities do not eliminate optimization or runaway pacing

Stellaris’s original research design described **three parallel fields**—Physics, Society, and Engineering—with **three semi-random choices** when selecting a project. Availability was weighted by circumstances such as existing technologies, empire characteristics, and scientist traits. Some technologies came through special circumstances, including anomalies and debris. Its granularity extends from new capabilities to incremental component improvements and repeatable bonuses. These are the foundational mechanics described in the 2015 developer diary; individual weighting rules have subsequently evolved. [Reddit](https://www.reddit.com/r/paradoxplaza/comments/3uuou3/stellaris_dev_diary_11_research_technology/)

This creates contextual variation, but it does not erase the underlying optimization problem. A community-developed technology-beelining tool explicitly calculates paths toward desired technologies and identifies research to avoid. Its existence demonstrates that probabilistic availability can coexist with deliberate route optimization; it does not establish how widespread such play is. [GitHub](https://github.com/serpentskirt/stellaris-tech-beeliner)

The **December 2023–January 2024 technology beta** is the strongest concrete pacing postmortem in this comparison. Developers attributed premature exhaustion of the tree to accumulated research-speed bonuses. Their experiment reduced standard researcher output from **4 to 3 per field** and introduced tier-gating “Breakthrough” technologies. In January, they reported that roughly **80% of survey responses** considered technology too slow; nearly **70% of respondents had over 1,000 hours** in the game, so this was a highly experienced, self-selected sample. They acknowledged that simultaneous changes compounded excessively and that Breakthroughs’ diplomatic interactions caused performance problems. The revised beta removed Breakthroughs and restored earlier base costs, introducing a separate difficulty-related cost control. These were beta iterations, not a claim about today’s ruleset. [Steam Store](https://store.steampowered.com/news/posts/?appids=281990&enddate=1706788842&feed=steam_community_announcements)

**Lesson for TCE:** contextual opportunity weighting is valuable, but graph topology, discovery rate, and economic feedback require separate balancing. Adding a gate to compensate for runaway research can create stagnation without solving the source of the runaway.

### Victoria 3: the strongest model for linking knowledge to concrete consequences

Victoria 3 separates Production, Military, and Society research. Its technology design combines directed innovation with passive technological spread; literacy and institutions influence those processes. Importantly, the developers state that many technologies primarily unlock options rather than immediately improving the country. Its era structure also penalizes skipping earlier research, so it is not an era-free model despite lacking a simple calendar-date lock on each invention. [Steam Store](https://store.steampowered.com/news/posts/?enddate=1639070046&feed=steam_community_announcements)

Its **production methods** are the especially relevant contribution. The original design described **two to five method categories per building**, with one active selection in each category. Methods change inputs, outputs, and employee requirements rather than merely increasing a universal efficiency number. An iron mine with more capable pumping can extract more but requires additional fuel and appropriately qualified workers. Automation can exchange labor requirements for manufactured inputs. These are operational tradeoffs, not just prerequisites on a research screen. [Reddit](https://www.reddit.com/r/victoria3/comments/o73uqb/victoria_3_dev_diary_5_production_methods/)

That complexity creates an interface obligation. In a January 2023 discussion, a player reported that profitability predictions repeatedly changed after switching methods and concluded that simply choosing the highest method might be easier. This is evidence of one player’s interpretability problem, not proof that the economic model was defective. Nevertheless, **a meaningful tradeoff that the interface cannot explain may be experienced as an arbitrary upgrade menu**. [Steam Community](https://steamcommunity.com/app/529340/discussions/0/3765605745625745114/)

**Lesson for TCE:** adopt production-method consequences, then add physical installation, retraining, local knowledge, and actual deployment decisions. Do not copy a national unlock followed by frictionless conversion of every workshop.

### Humankind: alternative paths can coexist with strong era rails

Humankind’s official encyclopedia makes an important distinction: when a technology has multiple incoming prerequisites, **only one is required**. These are OR paths. Technologies can unlock districts, infrastructure, units, resource extraction, and even new rules such as battle reinforcement. Technologies ordinarily belong to the current or previous eras; Scientist cultures can research the next technological era. Cultural influence can also make another empire’s technologies purchasable through osmosis. [Humankind Encyclopedia](https://humankind-encyclopedia.games2gether.com/en-us/research/game-concept/technology)

The strength is accessibility through alternative paths and non-research acquisition. The limitation for TCE is that a common era structure still organizes the overall journey, while culture selection grants predefined exceptions.

A revealing community response is the *Extended Naval Combat* mod. Its author describes the late Industrial/early Contemporary portion of vanilla play as too compressed and adds technologies and intermediate units to extend that experience. This is a modder’s diagnosis and intervention, not a controlled demonstration that more nodes improve pacing. [Nexus Mods](https://www.nexusmods.com/humankind/mods/5)

**Lesson for TCE:** measure whether an invention has time to become part of everyday life before a replacement arrives. Node count is not a substitute for useful operating life.

---

## 2. What academic models contribute

### Combinatorial innovation: build things that actually work

Arthur and Polak’s model constructs new technologies by combining existing logic circuits, testing the results against functional requirements, and retaining useful improvements. Successful constructions become reusable components. Their basic experiments ran for **250,000 steps** and produced different invention sequences and repertoires across runs. These are computational experiment steps, not historical years or a game-performance benchmark. [Santa Fe Institute](https://sites.santafe.edu/~wbarthur/Papers/AP-Complexity.pdf)

The most important lesson is not “random combinations generate technology.” It is:

> **Recombination needs functional evaluation and reusable interfaces.**

For TCE, combining a water wheel and a milling mechanism should matter because they deliver compatible motion and perform useful work—not because two arbitrary prerequisite flags are true.

The limitation is equally important: circuit correctness is much easier to test automatically than the viability of an unfamiliar furnace, agricultural technique, or institution. TCE should initially generate combinations **inside authored, validated physical and procedural constraints**, rather than claim to discover arbitrary engineering.

### Multiple innovation processes produce different historical rhythms

Kolodny, Creanza, and Feldman model several processes: major innovations, associated toolkits, combinations of existing tools, and losses. Their simulations can produce gradual accumulation, bursts, and declines. They also show why the distribution of knowledge among subgroups matters: total population is not equivalent to the number of people capable of maintaining a particular practice. [DOI](https://doi.org/10.1073/PNAS.1520492112)

For TCE, this suggests separate processes for:

* opening a substantially new capability;
* adapting it into local applications;
* improving established practice.

These should not share one universal probability or cost curve.

However, their model includes strong abstractions, including some immediate dependent losses. TCE should not literally delete every downstream practice when an upstream specialist dies. Test which operating capabilities actually become unavailable.

### Connectivity can preserve or destroy useful diversity

Derex and Boyd’s experiment compared fully and partially connected groups of equal size. In their task, copying successful individuals reduced diversity in fully connected groups; partial connectivity preserved alternative solutions that enabled more complex combinations. This is evidence about an experimental setting, not a universal claim that less connectivity is always better. [Arizona State University](https://asu.elsevierpure.com/en/publications/partial-connectivity-increases-cultural-accumulation-within-group/)

For TCE, connectivity should have competing effects: access to teachers and complementary knowledge can help, while rapid imitation can concentrate experimentation on a currently successful method.

**Do not give isolation a flat innovation bonus.** Model what people encounter, retain, copy, and combine.

---

## 3. Recommended structure for TCE

Everything in this section is a proposed design, rather than a description of a tested TCE implementation.

### 3.1 Use three connected graphs

| Layer | Represents | Important property |
| --- | --- | --- |
| **Capability graph** | The 150–250 authored capabilities and routes by which they can be learned or discovered | Explicit AND/OR requirements; no era gates |
| **Implementation graph** | Recipes, facilities, components, goods, skills, and service delivery | Many-to-many relationships; resource substitutions and operating constraints |
| **Knowledge network** | People, workshops, households, schools, guilds, records, and their contacts | Local possession, imperfect transmission, specialization, and loss |

The capability graph determines **what can become possible**. The implementation graph determines **whether it can work here**. The knowledge network determines **who can make it happen**.

Do not collapse these into `country.has_technology`.

A country-level summary can be useful for the interface, but it should be derived from its people and organizations.

### 3.2 Choose nodes by operational significance

A node should represent a **learnable, reproducible capability with an identifiable practical consequence**.

Useful node concepts include ceramic vessel firing, rotary grain milling, loom weaving, lime-mortar preparation, navigational surveying, and systematic bookkeeping. These names are illustrative; their exact scopes require content research.

Apply this splitting test:

**Split capabilities when they can plausibly be learned, transmitted, deployed, or resisted independently and create different operational consequences. Merge them when their differences are merely incremental tuning of the same practice.**

For example:

| Candidate distinction | Recommended representation |
| --- | --- |
| Hand processing versus a new powered process | Potentially distinct capabilities |
| Slightly better throughput from years of practice | Mastery or quality parameter |
| A new input substitution with different costs and limitations | Recipe variant; a node only if substantial new know-how is required |
| A different decorative style | Cultural or architectural grammar |
| An important scientific principle with no implemented effect yet | Do not count it as a completed content-bearing node |
| A new administrative procedure | Valid node when it changes recordkeeping, transactions, staffing, or service capacity |

A node does not need a unique building model. It does need a **testable change in what actors can do**.

### 3.3 Author AND/OR requirements as alternative complete routes

Use an OR of AND clauses:

\[
\operatorname{eligible}(t,c)
=
\bigvee\_{r\in R\_t}
\left[
\bigwedge\_{q\in r}q(c)
\right]
\]

Here, \(t\) is a capability, \(c\) is an actual collaborating group, and each \(r\) is a complete route.

For example, an authored capability might have:

```
Local experimentation route:
    prerequisite practical skills
    AND access to suitable experimental materials
    AND a feasible testing activity

Apprenticeship route:
    access to a competent teacher
    AND sufficient communication
    AND training time and equipment

Reconstruction route:
    an informative artifact or record
    AND the skills needed to interpret it
    AND experimental capacity
```

These are not necessarily equally difficult routes.

**Do not union knowledge across an entire empire.** A potter in one town and a metalworker hundreds of kilometres away satisfy a collaborative requirement only when some interaction brings their expertise together.

Equally, do not require a learner to repeat the historical discovery chain. Learning a procedure from an immigrant specialist can bypass the route by which that specialist’s society originally invented it.

### 3.4 Separate four kinds of condition

A single `prerequisites` field is insufficient.

| Condition | Example question | Should it prevent possessing knowledge? |
| --- | --- | --- |
| **Discovery requirement** | Can this group conduct a plausible experiment? | It can prevent local invention, not necessarily learning |
| **Learning requirement** | Is there an accessible teacher, record, or artifact? | It limits this acquisition route |
| **Operating requirement** | Are the workers, tools, inputs, and site conditions available? | No; knowledge can remain unused |
| **Adoption constraint** | Is installation worthwhile, affordable, permitted, and acceptable? | No; refusal is not ignorance |

This prevents several common errors.

A society without a local ore deposit should still be able to learn a metallurgical procedure or use imported metal. Losing a fuel supply should stop production, not erase understanding. Knowing how a machine works should not automatically make it profitable.

Also distinguish **causal prerequisites from historical associations**. “This occurred earlier in one historical trajectory” is insufficient justification for a hard edge.

Every authored dependency should record whether it is supported by physical necessity, demonstrated learning dependence, a plausible alternative route, or an explicit design simplification.

### 3.5 Put resource and regional differences into opportunities and viability

Prefer conditions such as accessible fibers, fuel costs, available animals, water seasonality, transport distance, and maintenance capacity over fixed labels such as `desert_culture_technology`.

For TCE, alternative solutions should compete to deliver similar services:

| Service | Candidate approaches | Dimensions that should differentiate them |
| --- | --- | --- |
| Food preservation | Drying, smoking, salting, fermentation | Climate, fuel, salt access, labor, storage properties |
| Mechanical work | Human, animal, water, wind power | Site, reliability, feed/fuel, capital, repair skills |
| Building enclosure | Earth, timber, stone-based methods | Local materials, transport, labor, maintenance, hazards |
| Communication and records | Oral specialists, physical tallies, written records | Training, copying, storage, interpretation, institutional use |

These are proposed content families, not a universal historical order.

The design objective is **conditional advantage**. A method can be genuinely superior for one purpose without being superior everywhere.

Avoid making every alternative a cosmetic route to an identical final bonus.

---

## 4. Making every node carry concrete content

### 4.1 Require a capability contract

Before a node enters the main graph, require the following authoring fields:

| Field | Required content |
| --- | --- |
| **Capability definition** | A sentence beginning “A competent actor can…” |
| **Acquisition routes** | Local discovery, instruction, reconstruction, or combinations |
| **Knowledge carriers** | Relevant people, organizations, records, and artifacts |
| **Executable effects** | Typed references to recipes, construction operations, jobs, services, or behaviors |
| **Operating requirements** | Inputs, facilities, equipment, site properties, and staffing |
| **Costs and limitations** | Installation, maintenance, training, failure, hazards, or substitution costs |
| **Mastery** | Which aspects improve through practice and their limits |
| **Transmission** | What can be taught, copied, demonstrated, or inferred |
| **Presentation** | Observable activity, equipment, goods, or service changes |
| **Validation and provenance** | Tests, historical sources, uncertainty, and explicit simplifications |

“Unlocks another technology” is not sufficient.

“Provides +5% output” is also insufficient **when nothing explains the changed process**. Quantitative improvements are legitimate when tied to something executable—for example, less material wasted by a specific cutting procedure.

### 4.2 Example: separate water power from grain milling

Rather than one enormous “Watermill” node that includes every associated invention, separate capabilities where doing so creates reusable content.

| Capability | Direct content | Potential composition |
| --- | --- | --- |
| **Rotary grain milling** | Milling equipment, operating procedure, grain-to-meal recipe, maintenance work | Can accept compatible human, animal, or mechanical drive |
| **Water-powered rotary drive** | Wheel/race components, construction and repair work, local shaft-power service | Can drive compatible machinery |
| **Mechanical transmission**, where warranted | Couplings, gearing or other authored transmission operations | Connects particular sources and loads |

A water-powered grain mill is then an **implemented combination**, not necessarily another research node.

That does not mean assembly should be instantaneous. Each composition rule should specify whether integration is routine, requires adaptation, or needs a substantial experimental project.

For the water-powered drive, knowledge can exist anywhere. Deployment requires a suitable site, construction materials, competent workers, and access rights. Operation introduces maintenance and availability constraints. The resulting power service is local: it should not become an empire-wide production modifier.

The consequences can then emerge through the rest of TCE: changed labor demand, concentrated processing, mill ownership, disputes over access, and settlement investment. Those outcomes should not all be hardcoded into the technology reward.

### 4.3 Use many-to-many content links

The desired relationship is:

```
capabilities → compatible procedures and components
procedures + inputs + equipment + workers → outputs and services
outputs and services → further practical possibilities
```

One capability can support several industries. One building can use several capabilities. One good can have several production routes.

This is how 200 meaningful nodes can support much more than 200 visible objects without becoming either a shallow unlock list or an unmanageable catalog of tiny inventions.

**Do not create a new technology for every crop, building style, tool material, and power-source combination.** Put variation at the layer where it belongs.

### 4.4 Treat institutional techniques carefully

Bookkeeping, surveying, standardized measures, and administrative procedures can be practical capabilities. A universal chain such as “Writing → Monarchy → Democracy” should not be.

Keep political choice and institutional formation in their own systems. Capabilities can alter feasible organization without dictating its outcome.

For example, improved records could increase the scale at which an organization can manage claims or inventories. Whether that organization becomes a temple estate, merchant association, municipality, or royal administration should depend on TCE’s people and institutions.

---

## 5. Discovery, adoption, and divergence

### Discovery should emerge from bounded practical activity

Give workshops and other collaborating groups a small set of candidate investigations generated by their actual activities, observed problems, materials, contacts, and interests.

A proposed project sequence is:

```
encounter opportunity
→ form a workable proposal
→ allocate people, time, and materials
→ build or perform a trial
→ evaluate outcome
→ revise, abandon, or demonstrate
→ teach and deploy
```

Routine practice should accumulate relevant experience, but repeating an action a fixed number of times should not guarantee an unrelated invention.

Stochastic outcomes can be attached to real experimental effort. For example, a stage-completion probability can use:

\[
P(\text{stage completion})=1-\exp(-qH)
\]

where \(H\) is qualifying experimental person-hours and \(q\) is a calibrated stage-specific rate. This is an implementation option, **not an empirically established invention law**. Different stages, partial progress, material constraints, and failed designs prevent it from becoming one universal memoryless discovery roll.

Avoid translating all activity into fungible national “science.” A bookkeeping practice should not directly fund furnace experimentation unless actual organizational processes connect them.

### Knowledge should have several states

Track at least the distinctions between awareness, practical competence, demonstrated implementation, and active use. Codified records are a separate carrier, not automatically equivalent to competence.

A settlement might therefore have:

> Three competent practitioners; one operating workshop; two households experimenting; no surviving written instructions.

That is more useful to TCE than “Technology researched.”

Loss should follow the disappearance of carriers and practice. A ruined workshop, inaccessible archive, or failed apprenticeship chain can reduce capability. **Do not recursively delete descendants merely because an ancestor is no longer practiced.** Recheck their actual requirements.

### Adoption should be a local decision

An actor’s adoption evaluation can consider expected service benefit or revenue against operating costs, installation, training, risk, access restrictions, and switching costs.

For non-market households and institutions, evaluate practical objectives rather than forcing everything into profit.

Use imperfect expectations. Actors should observe nearby results, inherit habits, and sometimes misjudge a method. However, adoption friction needs causes: missing credit, uncertainty, incompatible equipment, vested interests, or scarce skills—not an unexplained “conservatism percentage.”

To prevent implausible oscillation, conversion should consume time and resources. A workshop should not switch its physical setup every day because a tiny price change alters the apparent optimum.

### Divergence must survive contact without requiring permanent isolation

The desired outcome is not that every society has a permanently exclusive branch. Knowledge can spread while deployed systems remain different.

In TCE, durable differences can come from installed capital, available skills, maintenance networks, material access, preferences, ownership, and ecological conditions. Those differences should also be reversible when circumstances change.

This is more credible than locking a population out of a technology because of its cultural identity.

### Do not display a universal technological rank

The primary interface should answer:

* What can these people do?
* Who knows how?
* Where is it being used?
* What prevents broader use?

A capability inspector might say:

> Known locally, but not deployed: construction is feasible; the proposed site lacks secure access, and the workshop cannot currently support the training cost.

Show different repertoires and production systems rather than reducing everything to “Settlement A is 17 technologies ahead.”

### Be explicit about the finite content horizon

**A finite 150–250-node graph cannot guarantee unbounded qualitative invention.** Random names and endlessly repeated percentage bonuses do not solve that.

It can support an endless world through changing deployment, recombination, institutions, architecture, migration, loss, and rediscovery. Define the authored capability ceiling separately from the simulation’s duration.

Where genuinely new technological functions are required beyond that ceiling, they need additional authored content or a more powerful validated generative system.

---

## 6. Numbers, implementation, and validation

### What the published figures establish

| Source | Figure | Interpretation |
| --- | --- | --- |
| Civilization IV designer interview | Approximately **80** selected innovations | Authoring-scale description, not a current franchise-wide count. [Game Studies](https://gamestudies.org/1201/articles/tuur_ghys) |
| Victoria 3 technology diary, December 2021 | Approximately **175** technologies targeted; many countries starting with **20–30** | An unusually close comparison to TCE’s proposed graph scale. [Steam Store](https://store.steampowered.com/news/posts/?enddate=1639070046&feed=steam_community_announcements) |
| Same Victoria 3 diary | Leading countries expected to discover roughly **one technology per year** | A game pacing target, not an estimate of historical invention frequency. [Steam Store](https://store.steampowered.com/news/posts/?enddate=1639070046&feed=steam_community_announcements) |
| Arthur–Polak experimental model | **250,000** steps in the basic experiments | Demonstrates model behavior, not simulation-years or processing speed. [Santa Fe Institute](https://sites.santafe.edu/~wbarthur/Papers/AP-Complexity.pdf) |

These figures do **not** establish an appropriate TCE discovery rate. A broad national “technology” and a narrowly defined practical capability are different units.

### Runtime: avoid scanning every person against every node

For TCE’s upper population and a 200-node graph:

\[
50{,}000\times200=10{,}000{,}000
\]

That is **10 million person–node checks per complete sweep**, before evaluating nested conditions. This is arithmetic, not a measured benchmark.

A single 256-bit capability mask for every person would use:

\[
50{,}000\times32=1.6\text{ MB}
\]

That is only the raw mask—not mastery, provenance, records, projects, relationships, or allocation overhead.

The main risk is therefore not the static graph’s size. It is unnecessary repeated evaluation and expensive social or economic queries.

Recommended implementation:

**Compile definitions.** Use stable capability IDs, typed prerequisite expressions, and reverse indexes from changed conditions to affected candidates. Keep definitions separate from world state.

**Evaluate actual frontiers.** Maintain candidates for active workshops or collaborative groups. As an illustrative workload, 500 groups considering eight candidates means 4,000 candidate evaluations—not a claim that these are the right settlement or workshop counts.

**Use event-driven invalidation.** A new teacher, material source, facility, or learned capability can invalidate the relevant cached results. Routine learning can accumulate during daily work; opportunity selection and project evaluation need not run every rendered frame.

**Preserve determinism.** Stable project identities, ordered commits, and isolated random streams should make outcomes independent of thread scheduling. UE5 should render authoritative changes, not independently decide discovery.

The sources reviewed do not provide a directly transferable Rust/UE5 timing budget for this system. Benchmark it within TCE’s actual workloads.

### Validate behavior, not just graph reachability

| Test | Failure it should expose |
| --- | --- |
| **Content coverage** | A node has no executable effect beyond opening another node |
| **Bootstrap analysis** | A facility requires a tool that can only be made using that facility |
| **AND/OR route tests** | An alternative is accidentally mandatory, or an incomplete route incorrectly succeeds |
| **Geographical scenario tests** | A resource-poor start is unintentionally blocked from essential services |
| **Prerequisite-removal tests** | An arbitrary node dominates most of the graph |
| **Adoption tests** | Every unlocked method is always adopted, or almost none are usable |
| **Lifetime tests** | New facilities become obsolete before meaningful deployment |
| **Knowledge-carrier tests** | Conquest or migration instantly transfers everything, or specialist death erases too much |
| **Divergence tests** | Worlds differ only in research order while converging on identical operating systems |

Allow genuine foundational bottlenecks. The purpose is not to make every node optional; it is to ensure that necessity has a defensible cause.

Track **first demonstration**, **first operational deployment**, and **10%, 50%, and 90% adoption** among eligible users separately. Also track dormant knowledge, abandonment, and the share of output produced by each method.

Run ablations with geography differences removed, perfect global knowledge, disabled diffusion, and identical opportunity weights. These reveal whether divergence actually comes from the intended mechanisms or merely from random delays.

### A practical production sequence

Begin with a **24-capability vertical slice**, not the full 200-node roster. This is a proposed development scope.

Choose interdependent examples across food, materials, construction, power, transport, and records. Include alternative production routes, one imported-knowledge route, one integration project, one rationally rejected technology, and one recoverable knowledge loss.

Expand toward 150–250 only after those cases work. Otherwise, the larger graph will multiply unresolved assumptions and content debt.

---

## 7. Sources and further reading

The links below prioritize developer accounts, official reference material, and original research. Community examples are valuable for identifying failure modes, but are not treated as representative surveys.

| Source | Why it matters |
| --- | --- |
| **Soren Johnson, [Old World Designer Notes #4: The Technology Deck](https://www.designer-notes.com/old-world-designer-notes-4-the-technology-deck/?utm_source=chatgpt.com)** | Detailed deck behavior, opportunity costs, bonus cards, and a useful critical reader response |
| **Soren Johnson, [GDC 2022 postmortem: talk and illustrated transcript](https://www.designer-notes.com/my-elephant-in-the-room-part-2/?utm_source=chatgpt.com)** | Explicit discussion of golden paths and the rationale for contingent research |
| **Paradox, [Stellaris Dev Diary #11, developer-posted copy](https://www.reddit.com/r/paradoxplaza/comments/3uuou3/stellaris_dev_diary_11_research_technology/?utm_source=chatgpt.com)** | Foundational weighted research choices and special acquisition routes |
| **Paradox, [Stellaris technology beta and Dev Diary #328](https://store.steampowered.com/news/posts/?appids=281990&enddate=1706788842&feed=steam_community_announcements&utm_source=chatgpt.com)** | Concrete pacing experiment, survey findings, rollback, and performance implications |
| **Paradox, [Victoria 3 Dev Diary #27: Technology](https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-27-technology.1502428/), with [Steam text](https://store.steampowered.com/news/posts/?enddate=1639070046&feed=steam_community_announcements&utm_source=chatgpt.com)** | Intended scale, research, spread, literacy, and option-unlocking philosophy |
| **Paradox, [Victoria 3 Dev Diary #5: Production Methods](https://steamcommunity.com/games/529340/announcements/detail/2981928041979769782), with [text reproduction](https://www.reddit.com/r/victoria3/comments/o73uqb/victoria_3_dev_diary_5_production_methods/?utm_source=chatgpt.com)** | Concrete links among techniques, goods, fuel, employment, and production |
| **Firaxis, [Civilization VI August 2020 update notes](https://steamdb.info/patchnotes/5469056/?utm_source=chatgpt.com)** | Archived developer notes explaining Tech and Civic Shuffle |
| **Amplitude, [Technology encyclopedia](https://humankind-encyclopedia.games2gether.com/en-us/research/game-concept/technology?utm_source=chatgpt.com) and [Eras and Cultures](https://humankind-encyclopedia.games2gether.com/en-us/fame-cultures/game-concept/eras-cultures?utm_source=chatgpt.com)** | Explicit OR semantics, era restrictions, osmosis, and Scientist exceptions |
| **Tuur Ghys, [Technology Trees: Freedom and Determinism in Historical Strategy Games](https://gamestudies.org/1201/articles/tuur_ghys?utm_source=chatgpt.com), 2012** | Comparative analysis incorporating interviews with lead designers |
| **Arthur and Polak, [The Evolution of Technology Within a Simple Computer Model](https://onlinelibrary.wiley.com/doi/10.1002/cplx.20130?utm_source=chatgpt.com), with [author manuscript](https://sites.santafe.edu/~wbarthur/Papers/AP-Complexity.pdf?utm_source=chatgpt.com)** | Functional recombination, reusable components, and divergent invention sequences |
| **Kolodny, Creanza, and Feldman, [Evolution in Leaps](https://doi.org/10.1073/pnas.1520492112), 2015** | Multiple innovation processes, toolkit growth, subgroup knowledge, and loss |
| **Derex and Boyd, [Partial Connectivity Increases Cultural Accumulation Within Groups](https://asu.elsevierpure.com/en/publications/partial-connectivity-increases-cultural-accumulation-within-group/?utm_source=chatgpt.com), 2016** | Experimental evidence on connectivity, imitation, and diversity |
| **Reference indexes: [Stellaris Technology wiki](https://stellaris.paradoxwikis.com/Technology?utm_source=chatgpt.com), [Victoria 3 Technology wiki](https://vic3.paradoxwikis.com/Technology?utm_source=chatgpt.com)** | Useful starting points for version-specific inventories; pin a ruleset before extracting balance data |

## Bottom line

**TCE’s graph should explain why a possibility exists; its people and economy should determine whether that possibility becomes history.**

A successful node changes a procedure someone performs. A successful prerequisite explains a genuine dependency or learning route. A successful discovery has identifiable participants and circumstances. A successful alternative remains useful somewhere for a reason.

That structure makes 150–250 nodes a foundation for divergent societies rather than 150–250 steps in the same race.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92850-cc04-83e9-94d1-2e18d0c5369b)
