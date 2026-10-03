# Settlement builders with individual citizens: lessons for The Civilization Engine

**The strongest design for TCE is a combination, not an enlarged version of any one game:** Foundation’s demand-driven residential placement, Manor Lords’ adaptable household plots, Ostriv’s physically staged construction, Farthest Frontier’s seasonal production constraints, and the routing and data-oriented techniques documented by Banished and Songs of Syx.

The key distinction is **organic appearance versus autonomous development**. Foundation lets citizens place houses inside designated zones; Manor Lords lets the player shape economically meaningful plots. Neither mechanism, by itself, decides who owns land, finances expansion, authorizes a street, or bears construction risk. Those are the additional decision-making systems TCE needs. [Polymorph Games](https://www.polymorph.games/presskit/)

A second distinction matters just as much: **persistent individuals do not require continuous, expensive deliberation**. Songs of Syx’s developer describes compact storage and hierarchical pathfinding, while Banished’s developer documents the value of inexpensive walking-distance estimates before committing to detailed routes. These are directly applicable to TCE’s Rust kernel. [Reddit](https://www.reddit.com/r/pcgaming/comments/eoisok/my_first_game_colony_sim_citybuilder_with_up_to/)

This report uses evidence available through **September 27, 2026**. Historical patches are identified as such: they establish what a developer changed and why, not necessarily today’s exact balance. Performance figures below are developer reports or explicitly labeled player observations—not independently reproduced benchmarks.

---

## 1. How the six games work—and where their designs break down

### Foundation: organic settlement form through constrained local autonomy

**Mechanism.** Foundation separates important player decisions from residential detail. The player places workplaces and designates residential areas; villagers construct their own houses within those areas and generate paths through movement. Major buildings use a modular construction system rather than a single fixed footprint. This produces irregular neighborhoods without requiring the player to position every home. The official description explicitly combines workplace placement, naturally generated paths, residential zoning, and modular monuments. [Polymorph Games](https://www.polymorph.games/presskit/)

The useful pattern is not “randomly place houses.” It is **give local agents freedom inside a constrained development envelope**. Workplace locations establish destinations; permitted residential land provides a feasible search space; individual placements create the finer-grained settlement.

**What worked.** The division of control produces a recognizable settlement without making every structure an explicit design task. For TCE, this suggests separating a household’s housing decision from a settlement’s land-use rules. A council, landlord, customary practice, or household claim could supply the constraints currently supplied by the player.

**What failed or was deliberately excluded.** Foundation is an important negative example for TCE’s demographic ambitions. In its 2023 roadmap reassessment, Polymorph shelved aging and families, housing decay, and graveyards. The developer cited unwanted complexity and micromanagement, including village populations becoming larger than originally anticipated. Thus, its organic urbanism should not be mistaken for a demonstrated multigenerational society model. [polymorph.games](https://www.polymorph.games/foundation/news/2023/02/?utm_source=chatgpt.com)

Its hauling fixes are more directly reusable. Before update **1.9.5**, transporters reserved quantities before arriving, but those quantities did not adapt well to changing resource availability. The revised algorithm dynamically adjusted carried quantities, reducing inefficient transport and improving coordination between workplaces. [Polymorph Games](https://www.polymorph.games/foundation/news/2023/07/11/the-1-9-5-scalability-update-is-now-available/)

**TCE lesson.** Adopt constrained, locally generated housing and path formation. Add household persistence, property rights, relocation costs, and demographic continuity. For hauling, a reservation must remain valid—or be revised—as the world changes; it cannot be an unquestioned promise created several minutes earlier.

### Manor Lords: the household plot as an economic and architectural unit

**Mechanism.** Burgage plots combine housing, expandable living capacity, and backyard production within a flexible boundary. Sufficient width permits additional living space; sufficient backyard space permits an extension. The documented family unit contains **three people**. Some extensions retain the household’s availability for other employment, while artisan extensions remove inhabitants from the general worker pool and dedicate them to that business. [Hooded Horse](https://wiki.hoodedhorse.com/Manor_Lords/Burgage_plot)

This is especially valuable for TCE because a parcel is not merely a rectangle under a building. It can contain residence, garden, workshop, storage, and future expansion. Architecture and household economics therefore affect one another.

However, **the player still draws the plots and selects development**. The burgage system solves flexible settlement geometry and mixed residential production more than autonomous land development.

**Logistics failure and redesign.** Update **0.8.024** is a useful case study in making an economy understandable. It restricted marketplace-stall operation to storage workers, exposed storage-to-market connections, and tied market capacity to active stalls and assigned families. Previously, stall ownership could be confusing—for example, workers in unrelated professions could operate stalls selling household produce. The same patch added configurable overstock buffers, accounted for goods in transit in supply calculations, and addressed misleading supply conditions. [Hooded Horse](https://wiki.hoodedhorse.com/Manor_Lords/0.8.024_-_Update_4)

That patch also illustrates competition between production and construction: it added warnings when construction materials were exported or reserved for crafting, including logs taken by a sawpit. It changed milling and bread-production quantities to reduce mill congestion and prevented milling when storage was full. [Hooded Horse](https://wiki.hoodedhorse.com/Manor_Lords/0.8.024_-_Update_4)

**TCE lesson.** Adopt the **persistent, mixed-use parcel**, but replace player-selected upgrades with household and enterprise decisions. Do not copy fixed “family equals three workers” arithmetic into a demographic simulation.

The market redesign supports an equally important principle: **every delivery responsibility should have a clear owner**. Household shopping, retail replenishment, workshop input collection, and long-distance freight are different jobs. Allowing every occupation to perform all of them opportunistically can create a busy-looking town whose labor allocation is impossible to explain.

### Ostriv: physical construction exposes the real cost of geometric freedom

**Mechanism.** Ostriv combines gridless placement and organic dirt roads with production and construction that visibly occupy workers and space. Its official description emphasizes freedom from grids and angles, rather than treating the settlement as a set of interchangeable square lots. [Ostriv](https://ostrivgame.com/home/)

A particularly revealing developer account concerns pavement. In Alpha 5, certain freeform structures were restricted to **convex four-sided shapes**. Pavement construction generated a grid of work points inside the area, and workers traveled from the stone stack to those points. Because the shape was convex, the implementation could use direct internal connections rather than repeatedly solve general paths. There could be **hundreds of work points**. [Ostriv](https://ostrivgame.com/alpha-5-patch-9-hotfix-55/)

The developer’s May 2026 discussion of more general, concave shapes shows why “just allow more vertices” is not a small feature. It affects triangulation, selection outlines, picking, overlap checks, boundary markers, and construction access. Concavity also requires additional local pathfinding because direct lines from the material stack may cross an intervening building. Bounding-box rejection was added before more expensive polygon tests. **This was development discussion for the expanded system, not evidence that every described feature had shipped in Alpha 5.** [Ostriv](https://ostrivgame.com/alpha-5-patch-9-hotfix-55/)

**Production details.** An early patch changed tanning into a **six-month batch process** requiring bark, water, salt, and lime. It introduced production-only water access to avoid industrial consumption draining household wells, allowed well-use restrictions, and avoided fetching very small input quantities. The same patch fixed doubled wood consumption when two carpenters were employed—a concrete warning about shared production state. [Ostriv](https://ostrivgame.com/alpha-1-patch-7/)

**TCE lesson.** Borrow physical staging and spatially grounded labor, but establish geometry restrictions deliberately. A parcel grammar that guarantees access and bounded complexity is more valuable than unrestricted polygons that occasionally produce impossible buildings.

Also separate **processing duration from labor duration**. A six-month tanning batch should not automatically consume six months of uninterrupted worker attention. TCE’s authored recipes should distinguish preparation, unattended processing, inspection, finishing, and storage requirements.

### Banished: a small survival economy with an unusually useful routing postmortem

**Mechanism.** Banished makes people the central productive resource: they age, work, form families, and die. Housing supports population growth, but houses alone do not create residents. Its economy has no money or skill tree; available resources determine what can be built, and visiting merchants enable barter. Those choices create a compact survival model rather than a simulation of firms and institutions. [Steam Store](https://store.steampowered.com/app/242920?curator_clanid=27511314&l=polish)

The demographic feedback is important. A contemporary player account describes repeated cycles of elderly occupants, compensating house construction, population growth, and subsequent starvation. This is an anecdotal report, not a measured demographic model, but it identifies the kind of delayed feedback TCE must handle over generations. [PistonHeads](https://www.pistonheads.com/gassing/topic.asp?f=108&h=0&t=1379868)

**The routing failure.** Banished originally selected nearby accessible destinations using straight-line distance. A workplace or food source across a river could therefore look close despite a long detour to a bridge. Running full detailed searches for every candidate was too expensive.

The developer instead built a coarse graph of connected areas, splitting chunks where obstacles disconnected their interiors. Coarse A\* supplied walking-distance estimates; a short-lived area-pair cache reused them. Local construction changes required local graph updates. The developer reported the coarse search as **60–100 times faster** than detailed pathfinding on a map with roughly **250,000 fine nodes**. Importantly, this was a destination-scoring solution; using the coarse graph to constrain the final detailed route was discussed as a future idea. [Shining Rock Software](https://shiningrocksoftware.com/2013-11-21-more-bugs-pathfinding-problems/)

**TCE lesson.** Before optimizing how an agent walks somewhere, ensure it has chosen a sensible destination. Household relocation, employment, shopping, and material sourcing should use estimated travel costs, not Euclidean proximity.

Adapt rather than copy Banished’s reassignment behavior: a shorter commute should compete with tenure, kinship, rent, workshop ownership, and moving costs. Otherwise, your world may be economically efficient but socially implausible.

### Songs of Syx: large populations through constrained work and data-oriented implementation

**Mechanism and scale.** Songs of Syx explicitly targets settlements with tens of thousands of individually simulated citizens and slaves. Its public positioning also includes large battles, but **battle-unit counts must not be treated as equivalent to a continuously simulated civilian population**. [Steam Store](https://store.steampowered.com/app/1162750/Songs_of_Syx/)

An early developer discussion provides unusually specific implementation evidence. Jake described a non-object-centric layout intended to keep related data close in memory, hierarchical pathfinding, and a **768 × 768** tile map. He reported that pathfinding consumed about **70% of workload**, with thousands of paths found per second. Battle decisions were partly delegated to groups, reducing individual deliberation. He also described work divided among rendering, audio, logic, and AI logic, with performance still strongly dependent on clock speed. These are historical implementation statements, not verified descriptions of the current build. [reddit.com](https://www.reddit.com/r/pcgaming/comments/eoisok/my_first_game_colony_sim_citybuilder_with_up_to/)

This is a stronger precedent for TCE than the vague claim that “simple graphics let it simulate more people.” The documented gains concern **memory layout, hierarchy, and how much thinking each entity performs**.

**Logistics at city scale.** Community accounts show that distribution design can consume a substantial share of the economy. One player reported approximately **1,300 warehouse workers out of 4,800 workers**. Replies described using loading/unloading stations, shorter warehouse collection ranges, and local workshop supply to reduce long-distance collection work. Another reported roughly **9.5% of a 5,700-person population** in logistics. These are different cities, configurations, and denominators—not a controlled before/after comparison or a universal staffing target. [Reddit](https://www.reddit.com/r/songsofsyx/comments/1vvdsy9/how_to_reduce_warehouse_workers/)

**TCE lesson.** Separate local collection from bulk transport. Do not allow every warehouse worker to become an unrestricted, citywide courier. At increasing scale, a settlement needs distribution structure: collection points, freight routes, destination stores, and local delivery.

Also borrow the division between individual identity and shared planning. A person can remain persistent while a workshop schedules its jobs or a freight organization assigns its carriers.

### Farthest Frontier: seasonal capacity and preservation, not just recipe ratios

**Mechanism.** Farthest Frontier makes agricultural production depend on preparation, soil, fertility, weeds, rocks, crop timing, and rotations. Its official guide describes **three-year crop rotations**, initial field sizes from **5 × 5 to 12 × 12**, and preparation that can take years. It specifically recommends staggering activities because overlapping harvests create labor peaks. [Farthest Frontier](https://www.farthestfrontier.com/guide/gameplay/farming)

Its food chains distinguish long-lived reserves from immediately edible output. Grain is stored, milled into flour, and baked into bread; later stages have progressively shorter useful storage lives. Mills require heavy tools, initially obtained through trade. Storage conditions, preserving, and containers affect losses. The result is a meaningful difference between “the town harvested enough” and “the town can keep people fed throughout the year.” [Farthest Frontier](https://www.farthestfrontier.com/guide/gameplay/food)

Construction has a straightforward contract: villagers first deliver the required resources, then builders complete the building. Roads improve travel speed. This is easier to reason about than construction that advances independently of supply, though it offers less stage-by-stage flexibility than the model proposed below for TCE. [Farthest Frontier](https://www.farthestfrontier.com/guide/gameplay/buildings)

**A revealing balance correction.** In the v0.9.3 development announcement, Crate changed a game day from **five to eight real seconds at 1×**. The developer explained that balancing needs and productivity against travel distance had become difficult without making villagers move absurdly quickly. The same announcement reported up to a **90% framerate improvement** in internal tests of towns exceeding **2,000 villagers**, with explicit hardware and settlement caveats. [Crate Entertainment Forum](https://forums.crateentertainment.com/t/v0-9-3-and-beyond/136812)

**TCE lesson.** Travel time, calendar duration, consumption, and productivity must be calibrated together. Time compression is part of the economic model, not merely a presentation setting.

---

## 2. Numbers worth retaining—and what they actually establish

| Evidence | Documented figure | Appropriate interpretation for TCE |
| --- | --- | --- |
| Banished coarse destination-distance searches | **60–100× faster** than fine searches; approximately **250,000** fine nodes | Strong evidence for cheap destination scoring and local invalidation—not a whole-engine speedup. [Shining Rock Software](https://shiningrocksoftware.com/2013-11-21-more-bugs-pathfinding-problems/) |
| Songs of Syx early engine discussion | Advertised up to **30,000 subjects**; **768²** map; about **70%** of workload in pathfinding | A credible architectural precedent, but not a current, hardware-controlled benchmark. [reddit.com](https://www.reddit.com/r/pcgaming/comments/eoisok/my_first_game_colony_sim_citybuilder_with_up_to/) |
| Songs of Syx V53 developer report | Approximately **10%** performance improvement | Evidence of iterative optimization; the post provides insufficient methodology for comparison with TCE. [itch.io](https://songsofsyx.itch.io/songs-of-syx/devlog/170956/refinement-mod-support) |
| Farthest Frontier v0.9.3 development tests | Up to **90%** more FPS at **2,000+ villagers** | Performance can improve substantially after launch; no evidence here for linear scaling to 50,000. [Crate Entertainment Forum](https://forums.crateentertainment.com/t/v0-9-3-and-beyond/136812) |
| Farthest Frontier calendar rebalance | **5 → 8 seconds per day** at 1× | A 60% longer day changed the time available for work and travel. [Crate Entertainment Forum](https://forums.crateentertainment.com/t/v0-9-3-and-beyond/136812) |
| Ostriv’s early tannery design | **Six-month** batch | Recipes need elapsed-time and working-time semantics, not only input/output ratios. [Ostriv](https://ostrivgame.com/alpha-1-patch-7/) |

I did not find a comparable, controlled population-and-framerate benchmark across all six games. Foundation’s references to thousands of moving parts, large player settlements, and minimum system requirements should not be converted into a reliable citizen capacity estimate.

For TCE, report **simulated days per real second**, population, route-request rate, and high-percentile tick time alongside rendered FPS. A town can render smoothly while its simulation fails to maintain the selected time acceleration.

---

## 3. Common failure modes: distinguish economic hardship from software pathology

### Starvation spirals are often delivery spirals

The recurring risk is not simply insufficient annual food production. Manor Lords’ marketplace redesign, Foundation’s transport-reservation changes, and Songs of Syx’s warehouse discussions all point toward a different problem: resources exist, but the organization of labor prevents them from reaching consumers reliably. [Hooded Horse](https://wiki.hoodedhorse.com/Manor_Lords/0.8.024_-_Update_4)

For TCE, a plausible feedback loop is:

**Food deliveries fall → people spend more time seeking food or lose productive capacity → harvest and transport work decline → deliveries fall further.**

That is a useful emergent crisis when caused by real constraints. It is not useful when caused by an orphaned reservation or an agent repeatedly choosing an inaccessible store.

Model and diagnose three separate shortages: **insufficient production, insufficient transport capacity, and insufficient access or purchasing power**. They can look identical from the household’s perspective but demand different responses.

### Growth creates demand before it creates productive capacity

New households need food, fuel, housing, and services immediately; fields and orchards may require substantial lead time. Farthest Frontier explicitly makes agricultural establishment costly and delayed, while Banished’s population experience illustrates demographic overshoot. [Farthest Frontier](https://www.farthestfrontier.com/guide/gameplay/food)

TCE should therefore make expansion decisions depend on expected future conditions—not merely current warehouse totals. A large post-harvest stockpile is not necessarily a permanent surplus.

### Shared resources create hidden conflicts

Construction, workshops, heating, and trade may all compete for the same timber. Water may serve both households and industry. The Manor Lords construction warnings and Ostriv’s industrial-water changes expose these conflicts directly. [Hooded Horse](https://wiki.hoodedhorse.com/Manor_Lords/0.8.024_-_Update_4)

Do not solve every conflict with a universal, invisible priority order. In TCE, rights and priorities should belong to households, firms, customary rules, or governments. A workshop taking water from a village well might be a legitimate political problem—not necessarily an engine bug.

### Tiny numerical or lifecycle errors can stop an entire chain

Ostriv fixed construction becoming stuck when fewer than one nail remained to be supplied. That is a small implementation detail with a large visible consequence. [Ostriv](https://ostrivgame.com/alpha-5-patch-1-hotfix-2/)

TCE needs exact semantics for divisible versus indivisible goods, rounding, partial delivery, cancellation, and completion. “Almost complete forever” should not be an emergent feature.

---

## 4. Recommended TCE design

The following is a proposed synthesis, not a claim that any one reference game implements the complete architecture.

### A. Make settlement growth a chain of accountable decisions

Use this sequence:

**Need or opportunity → proposal → land access → financing/resource commitment → work orders → construction → occupancy or operation.**

A household proposes a home because it is overcrowded. A miller proposes a mill because expected throughput justifies the investment. A council proposes a bridge because recurring crossings impose a collective cost.

Each proposal needs an actor, expected benefit, resource budget, acceptable delay, and abandonment rule. This replaces the player’s invisible coordination without requiring every citizen to run a global city planner.

Local knowledge should matter. Households can know nearby rents and vacancies; merchants can estimate freight demand; officials can commission surveys. Avoid giving everyone instantaneous knowledge of the entire world.

### B. Preserve parcels through changes in buildings and ownership

Borrow Manor Lords’ mixed-use plot, but give it a longer life than any one house. A parcel should retain its identity through inheritance, subdivision, annexation, fire, demolition, and rebuilding.

Separate **legal geometry** from **physical geometry**. The claimed boundary, building footprint, usable yard, public frontage, entrance, and easement are different objects.

Authored building components should declare constraints: minimum support, permitted spans, materials, access clearance, adjacency requirements, and construction prerequisites. A settlement can then create architectural variation without generating structurally or logistically impossible combinations.

Initially, support a restricted set of robust parcel operations. Ostriv’s experience is a good reason to validate access and polygon complexity before adding arbitrary concavity. [Ostriv](https://ostrivgame.com/alpha-5-patch-9-hotfix-55/)

### C. Separate worn paths from engineered roads

Foundation’s movement-generated paths are an excellent visual precedent, but TCE needs an explicit distinction between **a trail emerging through use** and **a road requiring collective investment**. [Polymorph Games](https://www.polymorph.games/presskit/)

A useful research basis is Helbing, Keltsch, and Molnár’s active-walker model. Pedestrians alter the ground; improved walking conditions attract subsequent movement; vegetation recovery weakens unused trails. The paper reproduces features such as the bundling of initially separate paths. This is a proposed implementation basis for TCE, not evidence of Foundation’s internal algorithm. [arXiv](https://arxiv.org/pdf/cond-mat/9805158)

For TCE, maintain a bounded wear field influenced by passage, ground conditions, and regrowth. Let it affect both appearance and modest travel costs. Persistent traffic can then create an investment opportunity for clearing, drainage, widening, paving, or bridging.

Use hysteresis: route choices and road status should not oscillate whenever traffic fluctuates. Also allow inherited, inconvenient streets to persist. An endlessly reoptimized street network erases the physical history TCE is trying to generate.

### D. Treat hauling as production with its own capacity

A recipe should not be considered productive merely because its inputs exist somewhere.

A useful carrier-capacity estimate is:

\[
\text{delivery rate}
=
\frac{\text{usable load}}
{\text{loading}+\text{loaded travel}+\text{unloading}+\text{return travel}+\text{queueing}}
\]

This is a planning approximation; return loads and multi-stop routes require extensions. Its purpose is to expose the labor and time consumed by distribution.

Build three cooperating layers: **local collection**, **bulk transport**, and **last-mile delivery**. A workshop porter can collect from a nearby depot; a cart can move bulk material between districts; household members or market workers can handle final distribution. A village need not possess all three initially.

Use input and output buffers with minimum and maximum targets. Stop production when output has nowhere to go. Consolidate deliveries, but do not wait indefinitely for a perfect load when a critical recipient is running out.

Most importantly, assign delivery ownership once. A workshop and warehouse should not both dispatch workers for the same requirement unless the request explicitly allows partial fulfillment.

### E. Make reservations first-class, inspectable state

Represent a shipment or material commitment with explicit references to the source inventory, destination capacity, quantity, carrier or task, priority, and expiration conditions.

Maintain distinct values for **on hand, reserved, in transit, and expected**. Expected goods must not become consumable merely because a delivery was promised.

Reservations need lifecycle rules. When a carrier dies, a bridge collapses, a construction project pauses, or ownership changes, commitments must be released or reassigned. Long-running tasks should revalidate their assumptions.

For parallel Rust updates, let workers propose claims against a stable view, then resolve conflicting claims at a controlled commit stage. Do not allow two production jobs to consume the same stock simply because they read it before either writes its result.

Foundation’s transporter fix is evidence for revalidation; Ostriv’s two-carpenter resource bug is evidence for testing shared consumption. [Polymorph Games](https://www.polymorph.games/foundation/news/2023/07/11/the-1-9-5-scalability-update-is-now-available/)

### F. Build structures as dependency graphs, not a single percentage

Use stages such as site preparation, foundations, frame, roof, enclosure, and equipment installation. Each stage requires its own materials, labor, tools, skills, and reachable work locations.

Completion should mean that **both material and work requirements have been satisfied**. Allow useful partial completion only where authored: a roofed shelter may be habitable before finishing, while a mill without its mechanism cannot operate.

Do not reserve every material for every future stage immediately. That can immobilize scarce stock across hundreds of unfinished projects. Prefer commitments around the active stage and a limited look-ahead window.

For visualization, let authoritative milestones drive scaffolding, material stacks, structural components, and worker tasks. Near the camera, expose individual operations; farther away, use coarser construction states. A nail need not be a physics object for nail supply to be causally real.

The desired rule is: **visual detail may be approximate; construction causality may not be**.

### G. Preserve individuals while reducing unnecessary computation

Use compact hot-state storage for frequently accessed data and colder storage for biography, relationships, possessions, and historical records. Evaluate needs and choices when relevant conditions change rather than rerunning all decisions every frame.

Use hierarchical travel estimates to select jobs and destinations, then generate detailed routes only for chosen trips. Cache appropriate shared results, invalidate affected regions after geometry changes, and back off repeated impossible requests.

Songs of Syx provides the scale precedent; Banished provides the particularly valuable lesson that destination selection itself deserves a cheap routing layer. [Reddit](https://www.reddit.com/r/pcgaming/comments/eoisok/my_first_game_colony_sim_citybuilder_with_up_to/)

On the Unreal side, keep the Rust kernel authoritative. UE’s MassEntity offers data-only fragments, chunked archetypes, and batch processors that are suitable building blocks for large representations. It does not automatically solve TCE’s economy or justify duplicating citizen decision-making in Unreal. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-entity-in-unreal-engine)

A persistent citizen should not require a permanently active heavyweight actor. Conversely, hiding someone must not erase their consumption, employment, travel, or existence.

### H. Treat fast-forward as a separate engineering requirement

Consider an illustrative workload—not a benchmark:

* 50,000 people;
* ten meaningful state transitions per person per simulated day;
* one simulated year per real minute.

That already implies approximately:

\[
50{,}000 \times 10 \times 365 / 60
\approx 3.0\text{ million transitions per second}.
\]

This excludes buildings, goods, animals, politics, and history.

Consequently, event-driven design alone does not guarantee century-scale acceleration. Predictable intervals may need analytical advancement or batching: stored-food deterioration, uninterrupted processing, and travel along unchanged routes are candidates. Interruptions must still be handled correctly.

Keep one consistent simulation calendar. Acceleration should change how quickly it is processed, not silently change travel speed relative to hunger, seasons, or construction. Farthest Frontier’s day-length rebalance demonstrates how tightly those quantities interact. [Crate Entertainment Forum](https://forums.crateentertainment.com/t/v0-9-3-and-beyond/136812)

---

## 5. What to implement first

I would prioritize **a small autonomous settlement that survives its own decisions** before maximizing citizen count or geometric freedom.

| Prototype | What it should demonstrate |
| --- | --- |
| **Household–food–construction loop** | Households create demand; agents establish production and housing; deliveries consume labor; seasonal shortages have understandable causes. |
| **Distribution and reservation stress test** | Cancellation, death, full stores, blocked roads, and competing construction orders do not duplicate goods or leave permanent orphaned tasks. |
| **Growing-settlement scale test** | The same mechanisms run at 10k, 25k, and 50k people, with measured route demand and simulated-time throughput—not only FPS. |
| **Multigenerational continuity test** | Aging, inheritance, replacement workers, parcel reuse, maintenance, and abandoned structures remain viable without player intervention. |

The observer interface should be developed alongside these systems. Clicking a stalled building should reveal **which material, which source, which carrier, and which blockage**. Clicking a hungry household should distinguish “no food produced,” “delivery delayed,” and “cannot access or afford available food.”

That information is not merely debugging output. It is the foundation of TCE’s historical storytelling: the bridge failed, deliveries stopped, a workshop closed, and a household moved.

---

## 6. Source guide

### Developer accounts and official documentation

**Banished:** Luke Hodorowicz’s [“More bugs! Pathfinding problems!”](https://shiningrocksoftware.com/2013-11-21-more-bugs-pathfinding-problems/?utm_source=chatgpt.com) is the highest-value technical account here: destination selection, connected-area abstraction, caching, and measured search improvement.

**Foundation:** The [official press kit](https://www.polymorph.games/presskit/?utm_source=chatgpt.com) documents the placement model. The [February 2023 roadmap reassessment](https://www.polymorph.games/foundation/news/2023/02/?utm_source=chatgpt.com) explains demographic features being shelved. The [1.9.5 release notes](https://www.polymorph.games/foundation/news/2023/07/11/the-1-9-5-scalability-update-is-now-available/?utm_source=chatgpt.com) document the transporter correction.

**Manor Lords:** The publisher-hosted [burgage plot wiki](https://wiki.hoodedhorse.com/Manor_Lords/Burgage_plot?utm_source=chatgpt.com) explains mixed-use plots and household specialization. [Update 0.8.024](https://wiki.hoodedhorse.com/Manor_Lords/0.8.024_-_Update_4?utm_source=chatgpt.com) is a particularly useful logistics and market-design changelog.

**Ostriv:** The developer’s [May 11, 2026 geometry/construction discussion](https://ostrivgame.com/alpha-5-patch-9-hotfix-55/?utm_source=chatgpt.com) exposes the implementation cost of freeform construction. [Alpha 1 Patch 7](https://ostrivgame.com/alpha-1-patch-7/?utm_source=chatgpt.com) provides concrete production, water-allocation, and shared-resource fixes.

**Songs of Syx:** The [developer’s early technical Q&A](https://www.reddit.com/r/pcgaming/comments/eoisok/my_first_game_colony_sim_citybuilder_with_up_to/?utm_source=chatgpt.com) contains the storage, pathfinding, workload, and threading statements. The [V53 development update](https://songsofsyx.itch.io/songs-of-syx/devlog/170956/refinement-mod-support?utm_source=chatgpt.com) documents a subsequent optimization pass.

**Farthest Frontier:** The official [farming guide](https://www.farthestfrontier.com/guide/gameplay/farming/?utm_source=chatgpt.com), [food guide](https://www.farthestfrontier.com/guide/gameplay/food/?utm_source=chatgpt.com), and [building guide](https://www.farthestfrontier.com/guide/gameplay/buildings/?utm_source=chatgpt.com) describe production constraints. [“V0.9.3 and beyond!”](https://forums.crateentertainment.com/t/v0-9-3-and-beyond/136812?utm_source=chatgpt.com) explains the time-scale correction and reported performance gains.

### Community evidence and supplemental engineering material

The Songs of Syx discussion [“How to reduce Warehouse Workers?”](https://www.reddit.com/r/songsofsyx/comments/1vvdsy9/how_to_reduce_warehouse_workers/?utm_source=chatgpt.com) is useful firsthand evidence of logistics labor costs, with the comparability limitations noted above.

For road emergence, read Helbing, Keltsch, and Molnár, [*Modelling the Evolution of Human Trail Systems*](https://arxiv.org/abs/cond-mat/9805158?utm_source=chatgpt.com), published in *Nature* in 1997, with the author manuscript on arXiv.

For UE implementation, Epic’s [MassEntity overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-entity-in-unreal-engine?utm_source=chatgpt.com) and [*City Sample: Unpacked* video course](https://dev.epicgames.com/community/learning/courses/wy4/unreal-engine-city-sample-unpacked/Kkx/unreal-engine-introduction-to-city-sample-unpacked?utm_source=chatgpt.com) are relevant supplemental material, not settlement-economy postmortems.

**Bottom line:** borrow the visible, local mechanisms that make these settlements convincing, but replace their player-supplied coordination with accountable household, enterprise, and institutional decisions. TCE’s hardest requirement is not merely making 50,000 people move. It is ensuring that their movement, production, construction, and failures remain causally consistent when nobody is there to reorganize the town.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927dc-1b04-83ea-9be2-fa96856a5b37)
