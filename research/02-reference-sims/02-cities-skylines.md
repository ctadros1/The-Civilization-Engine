# Cities: Skylines 1 and 2: simulation internals and lessons for TCE

## Executive assessment

**The most useful lesson for TCE is to separate three promises: persistent people, believable activity, and economically consequential activity.** A simulation can fulfill the first, approximate the second, and still break the third.

Cities: Skylines 1 deliberately favored individually identifiable citizens over realistic building populations. Cities: Skylines 2 added richer household finances, production chains, destination choices, and service behavior. However, developer fixes and modding evidence show how errors between these systems—rather than a complete absence of simulation—can produce implausible results. Examples include resource transactions failing to deliver their inputs, garbage-route estimation modifying a truck’s load, and shopping decisions ignoring distance. [Sims 4 Network](https://sims4network.wixsite.com/sims4network/single-post/2015/01/07/cities-skylines-dev-diary-7-simulation?utm_source=chatgpt.com)

For TCE, the right target is **fewer people with stronger causal consistency**, not the largest population counter. Adopt persistent identities, data-oriented processing, explicit production and service requests, and generalized travel costs. Avoid silent traffic suppression, invisible economic rescue mechanisms, and interfaces that imply successful service merely because a building is nearby.

**Version scope:** Research is current through September 27, 2026. The latest official patch located was **1.6.2f1, “Autumn Breeze,” September 15, 2026**, under developer Iceflake. Historical findings below are dated rather than presented as current defects. This is a source-based investigation, not a fresh benchmark or exhaustive audit of proprietary game code. [Steam Community](https://steamcommunity.com/app/949230/announcements/?l=english)

---

## 1. How the citizen simulations work

### C:S1: persistent citizens, simplified urban capacity

The original simulation diary describes citizens with names, ages, homes, and workplaces or student status. They travel to work, shops, and leisure destinations, using walking, cars, and public transportation. It also explicitly acknowledges deliberately low residential occupancy: a high-rise might contain only twelve households because individually simulating more residents was considered too expensive. [Sims 4 Network](https://sims4network.wixsite.com/sims4network/single-post/2015/01/07/cities-skylines-dev-diary-7-simulation?utm_source=chatgpt.com)

This establishes an important distinction:

**A citizen is a persistent resident, but the population count is not a count of simultaneously active pedestrians or vehicles.**

For engineering purposes, distinguish durable population records, relationships to buildings and households, current activities, and instantiated travelers. TCE should preserve that separation even when every person’s day is economically accounted for.

C:S1’s demographic and education rules are consequential but highly stylized. Mod author algernon documents that vanilla immigrant education depends on the residential building’s level, while age restrictions can prevent older immigrants from acquiring missing early schooling. This creates a feedback loop between housing development and workforce qualifications that is convenient for a city builder, but not a convincing model of knowledge transmission. [Steam Community](https://steamcommunity.com/workshop/filedetails/discussion/2027161563/3193618785998756176/)

**Death waves are partly a cohort-generation problem.** Lifecycle Rebalance Revisited changes immigrant age distributions and mortality behavior to reduce synchronized deaths. Its author also emphasizes that existing demographic imbalances take time to work through the population: changing birth or death rules does not immediately repair a malformed age pyramid. [Steam Community](https://steamcommunity.com/sharedfiles/filedetails/?id=2027161563)

### Daily schedules are a separate design problem

The original game’s activity model should not be mistaken for a faithful calendar of work shifts, school hours, weekends, and family time.

The **Real Time** mod is revealing because its additions include scheduled work and school, shifts, commute-time estimates, lunch breaks, vacations, weather-sensitive behavior, and cancellation of excessively delayed trips. Its documentation recommends populations around **65,500 or below** for the best experience, citing game limits and performance. That is the mod author’s practical envelope, **not C:S1’s total population limit**. [GitHub](https://github.com/dymanoid/RealTime)

The architectural lesson is that **persistent identity does not automatically produce believable temporal behavior**. Employment can be assigned correctly while the visible daily rhythm remains artificial.

### C:S2: households become meaningful economic actors

C:S2’s documented lifecycle has four stages—child, teen, adult, senior—and five education levels. Households can contain multiple generations; citizens form and leave households, study, work, become sick, retire, and die. Education decisions consider prospective financial benefits rather than simply filling available seats. Health and well-being contribute to happiness, and illness can interrupt work and generate healthcare demand. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/citizen-simulation-lifepath)

Travel preferences also vary: teenagers emphasize monetary cost, adults time, and seniors comfort. This is a useful example of inexpensive heterogeneity: agents do not need elaborate planners to make different choices when the same alternatives are scored with different weights. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/citizen-simulation-lifepath)

**What worked:** household-level decision-making gives housing, education, consumption, and employment a common economic context.

**What TCE should change:** use household budgets and obligations, but do not make modern nuclear families, wage employment, formal schooling, or universal retirement assumptions foundational. TCE needs households, kin groups, workshops, estates, and institutions whose rules can differ by culture and technology.

---

## 2. Traffic: destination choice, routing, local driving, and disappearance

### C:S1’s destination selection and road routing are different operations

The C:S2 traffic diary provides a useful retrospective comparison: C:S1 selected destinations using straight-line proximity, then calculated a route through the actual road network. It generally retained that route despite later congestion, recalculating when changes invalidated it. **“Nearest destination” therefore did not mean vehicles drove without road-network pathfinding.** [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai)

This distinction explains a common class of bad outcomes: a service provider or shop can be geographically close but operationally distant because of a river, highway interchange, one-way streets, or disconnected access.

TM:PE’s scope illustrates what players found missing from the base traffic model: stronger control over lane use, parking, junctions, restrictions, and signals. Its maintained code and changelog also show that routing correctness depends on subtle details such as short road segments, lane connections after upgrades, and interactions between vehicle behavior and traffic-light state. [GitHub](https://github.com/CitiesSkylinesMods/TMPE)

### C:S2 uses a richer generalized travel cost

C:S2’s documented pathfinding weighs **time, money, comfort, and behavior**, including mode changes and parking. Its local traffic logic adds lane changes, overtaking, reactions to obstructions, and emergency-vehicle behavior. These are separate layers: better lane selection does not guarantee good destination choice, and a sensible destination does not guarantee an uncongested route. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai)

For TCE, model a trip as:

> Purpose → candidate destinations → mode and route → reservation or commitment → movement → arrival or explicit failure.

Each stage needs its own failure reason and instrumentation. Otherwise, “bad traffic AI” becomes an undiagnosable label for several unrelated systems.

### Despawning is a simulation policy, not just rendering

C:S1’s traffic mods explicitly provide **disable-despawn controls**, including vehicle-type filters. Their documentation also distinguishes removal caused by broken network conditions from ordinary traffic-management behavior. Thus, a disappearing vehicle is not sufficient evidence of one specific cause. [GitHub](https://raw.githubusercontent.com/CitiesSkylinesMods/TMPE/master/CHANGELOG.md)

C:S2 also retains removal and recovery mechanisms. September 2026 patch notes explicitly discuss cars despawned because parking was unavailable. [Steam Community](https://steamcommunity.com/app/949230/announcements/?l=english)

For TCE, keep three meanings separate:

| Operation | Meaning for TCE |
| --- | --- |
| Render culling | Stop drawing the person or cart; simulation continues unchanged. |
| Activity cancellation | The person abandons a trip and receives an explicit new state. |
| Entity removal | The person emigrates, dies, or otherwise leaves the modeled population. |

**Do not let despawning a traveler silently complete the traveler’s economic task.** A missing cart must not count as delivered grain merely because its movement object was removed.

### Traffic volume itself can be reduced as population grows

The original **Traffic Simulation Adjuster** repository documents a population-related traffic-reduction coefficient. In the version targeted by that mod, **4 was vanilla, 0 disabled reduction, and 10 increased it**; the author warned that removing reduction could substantially affect performance. These are parameter values, not percentages or a verified description of every later build. [GitHub](https://raw.githubusercontent.com/tduck973564/TrafficSimulationAdjuster/master/README.md)

This matters when evaluating scale claims. A larger population does not necessarily imply proportionally more trips or equally detailed daily activity.

For TCE, reducing visible animation is acceptable. Reducing actual commuting, trade, or service demand merely because the population passed a threshold would change the society being simulated.

---

## 3. Services: why a green overlay can mislead

### C:S1 already combined proximity effects with actual dispatch

The original developer diary explicitly separated a nearby fire station’s positive effect on residents from the ability of its fire engines to reach an incident through traffic. A neighborhood could benefit from proximity while actual response remained constrained by roads. [Sims 4 Network](https://sims4network.wixsite.com/sims4network/single-post/2015/01/07/cities-skylines-dev-diary-7-simulation?utm_source=chatgpt.com)

The **TransferManagerCE** author describes a shared request-matching mechanism: buildings post typed offers or requests—for example, a sick-person request—and compatible providers are matched. The mod changes matching behavior and exposes outstanding offers and matches for inspection. This is a valuable window into the machinery underneath service icons and freight movements. [Steam Community](https://steamcommunity.com/workshop/filedetails/?id=2804719780)

A generic matching system is reusable, but different tasks need different priorities. An ambulance, flour shipment, garbage pickup, and job search should not all be optimized by the same simple distance heuristic.

### C:S2 makes the distinction explicit

C:S2’s service design separates **passive coverage effects** from **service actually consumed or dispatched**. Coverage propagates along roads and can improve neighborhood conditions. Vehicles and visitors can operate beyond that coverage area. District assignments restrict operational service areas; they do not magically extend a building’s passive bonus throughout an entire district. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/city-services-districts-policies)

Service efficiency depends on staffing, qualifications, worker conditions, and utilities. Consequently, nominal capacity and nearby placement do not guarantee adequate throughput. Imported services offer another distinction: they can provide operational assistance without supplying the same local coverage benefits. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/city-services-districts-policies)

**For TCE, replace a single “coverage” measure with four inspectable quantities:**

* **Eligibility:** who may use the healer, granary, school, court, or fire brigade?
* **Reachability:** can the recipient or provider physically complete the trip?
* **Capacity:** are staff, supplies, equipment, and time available?
* **Outcome:** how long did service take, and was the need resolved?

A service can be nearby but unaffordable, accessible but overloaded, or fully staffed but missing medicine. Those differences should appear in the simulation and interface.

---

## 4. Economy: real mechanisms, deliberate simplifications, and broken links

### C:S1’s core is a spatial production-and-delivery loop

The original design describes industry acquiring raw materials, producing goods, and delivering them to commercial buildings for sale to residents and tourists. Local inputs are preferred when available; otherwise, outside connections supply them, with trucks, rail, and ships connecting the economy to transport infrastructure. [Sims 4 Network](https://sims4network.wixsite.com/sims4network/single-post/2015/01/07/cities-skylines-dev-diary-7-simulation?utm_source=chatgpt.com)

However, built form is not a reliable measure of economic capacity. **Realistic Population** exists specifically to revise implausible household and workplace counts in buildings. For TCE, building appearance, floor area, household capacity, storage, and productive capacity should derive from compatible authored parameters rather than independent visual and economic numbers. [Steam Community](https://steamcommunity.com/sharedfiles/filedetails/?id=2025147082)

### C:S2 expands the accounting model

The documented economy includes household and company money, inventories, wages, rent, utilities, shopping, production, transport costs, profitability, and bankruptcy. Households send members shopping when supplies run low; companies acquire inputs and sell outputs. Offices participate through immaterial resources as well as employment. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/economy-production)

But two major abstractions qualify the realism claim:

**First, resource base prices are fixed.** Transport and other costs affect profitability, but this is not a general market-clearing price system.

**Second, the launch design deliberately included stabilizers and redistribution.** The developer described a forgiving, self-balancing economy, including subsidies and virtual redistribution of money. These were game-design choices, not necessarily programming mistakes. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/economy-production)

For TCE, the useful distinction is between an explicit institution and a hidden safety mechanism. A village granary distributing reserves is a simulated institution. An undocumented transfer that prevents households from becoming poor is a safety mechanism that can erase the causes of migration, conflict, and institutional change.

### What modders found underneath the presentation

Infixo’s early **RealEco / Economy Rebalance** documentation identified concrete implementation and balance problems:

| Historical finding | Why it matters |
| --- | --- |
| Per-citizen consumption reportedly rose roughly **5–6× between 30,000 and 300,000 population**. | Per-capita behavior can acquire an unintended dependence on global population. |
| Some commercial firms paid for resources without receiving them. | A transaction can appear financially valid while failing physically. |
| Restrictions on buying directly from industrial firms also affected offices, obstructing access to immaterial goods. | Shared classifications can unintentionally break another sector. |

These are findings about the builds examined by the mod author, **not proof that the same defects remain in September 2026**. The mod’s chosen profit reductions and replacement businesses are proposed remedies, not independent evidence that those exact balance settings are correct. [GitHub](https://raw.githubusercontent.com/Infixo/CS2-RealEco/master/README.md)

The defensible diagnosis is therefore not “the entire economy was fake.” It is that **a substantial model contained broken causal links, opaque balancing rules, and poorly communicated outcomes**.

---

## 5. What went wrong with C:S2 technically?

### DOTS was a scaling strategy, not a guarantee

Paavo Huhtala’s launch-era inspection identified **Unity 2022.3.7, DOTS/ECS, Burst compilation, and approximately 1,200 systems**. It also found a custom rendering integration using `BatchRendererGroup`, rather than simply relying on the standard Entities Graphics path. [paavohtl's blog](https://blog.paavo.me/cities-skylines-2-performance/)

The intended architecture is sensible: store simulation data separately from heavyweight scene objects, process groups of similar entities, and distribute work across CPU cores.

But data-oriented processing only makes the chosen work cheaper. It does not prevent excessive destination searches, repeated failed requests, synchronization stalls, or a model that continually invalidates its own decisions.

### Rendering and simulation failures were different problems

In one inspected launch-era frame, Huhtala measured approximately **121 million input vertices and 36 million rasterized triangles across rendering passes**. Those are aggregate GPU-work counts—not unique visible city polygons. The investigation found excessive geometry work and problems involving detail selection, culling, shadows, and other rendering passes. [paavohtl's blog](https://blog.paavo.me/cities-skylines-2-performance/)

The first hotfixes corroborate that diagnosis: they changed LOD behavior, optimized fog, depth of field and global illumination, and addressed building-related stutters. Subsequent patches improved shadow culling and reduced unnecessarily large citizen textures. [SteamDB](https://steamdb.info/patchnotes/12538376/)

This is why “the simulation is slow” and “the frame rate is low” must be separate bug categories.

### The developer’s retrospective: unproven engine capabilities

In a March 2026 original interview, Colossal Order’s Mariina Hallikainen said the team had overestimated engine capabilities. The accompanying explanation identified engine instability, missing HDRP interpolators, and inadequate support for long-running ECS jobs, requiring additional implementation work. This is evidence about dependencies and integration risk—not proof that Unity or ECS is inherently unsuitable. [PC Gamer](https://www.pcgamer.com/games/sim/cities-skylines-2-boss-says-they-completely-overestimated-the-unity-engines-capabilities/)

For TCE, substituting Rust and UE5 does not remove that class of risk. The simulation-to-renderer bridge, animation, world streaming, shadows, and packaged Windows build must be demonstrated together at target scale.

---

## 6. What was fixed—and what those fixes reveal

The following chronology is more informative than treating “launch problems” as one undifferentiated failure.

| Date and version | Documented problem or change | Engineering lesson |
| --- | --- | --- |
| **October 26, 2023 — 1.0.11** | LOD/render-resolution changes, visual optimizations, and building spawn/level-up stutter fixes. | Rendering complexity and simulation event bursts need separate budgets. [SteamDB](https://steamdb.info/patchnotes/12538376/) |
| **November 2, 2023 — 1.0.12** | Resource-consumption bug, storage-aware input ordering, company profitability, bankruptcy behavior, and incorrect college/university eligibility counts addressed. | Validate transactions, capacity constraints, and UI denominators independently. [SteamDB](https://steamdb.info/patchnotes/12596549/) |
| **November 9, 2023 — 1.0.13** | Garbage trucks accidentally added load while estimating garbage along a route. Lane-merging behavior and unsafe highway U-turn penalties also changed. | A planning query must not mutate authoritative resource state. [SteamDB](https://steamdb.info/patchnotes/12645681/) |
| **January 31, 2024 — 1.0.19** | Industrial taxes changing sign, resource data after loading, and warehouse issues fixed. A school-search cooldown reduced repeated path requests when schools were unavailable; synchronization waits were also reduced. | Failed searches and dependency barriers can dominate runtime. [Steam Store](https://store.steampowered.com/news/posts/?enddate=1706707491&feed=steam_community_announcements) |
| **June 2024 — Economy 2.0** | Government subsidies removed, service costs revised, imported services made more explicit, and economic control reworked. | This was partly redesigning incentives and difficulty, not merely correcting arithmetic. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/dev-diary-economy-part-one) |
| **February 18, 2026 — 1.5.4f1** | Death timing concentrated overnight was corrected; checks increased from **4 to 16 per citizen per game day**. An Easy Mode bug meant approximately **80% never died of old age**. | Calendar handling and demographic probabilities require explicit tests. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/patch-notes-first-frost) |
| **June 22, 2026 — 1.6.0f1** | Shopping had ignored distance in favor of stock. Trip searches gained purpose-specific bounds; housing searches became less frequent. Failed trips received explicit fallback behavior. | Improve candidate selection and stop repeatedly searching impossible alternatives. [Steam Community](https://steamcommunity.com/app/949230/announcements/?l=english) |
| **September 15, 2026 — 1.6.2f1** | Garbage trucks reserve capacity for intended targets; spending decisions use income rather than wealth; young adults delay departure until housing is affordable. | Reservation policy and household transition rules have citywide effects. [Steam Community](https://steamcommunity.com/app/949230/announcements/?l=english) |

Economy 2.0 also removed the **virtual landlord**, made tenants contribute to upkeep, revised rent and building-condition behavior, and changed how financial distress was assessed. The developer warned that existing saves would need time to rebalance. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/dev-diary-economy-part-two)

That last point matters greatly for TCE: a corrected rule does not instantly repair inventories, employment assignments, building conditions, or demographic cohorts produced by the previous rule.

### What worked despite these failures

C:S1’s mods demonstrate the value of exposing substantial simulation behavior to modification: schedules, population capacity, service matching, and road behavior can be replaced or inspected rather than merely reskinned. [GitHub](https://github.com/dymanoid/RealTime)

C:S2’s richer household and service models remain useful design references. The failure was not “too much detail” in the abstract; the evidence points to a combination of unfinished integration, expensive searches, incorrect transitions, and insufficiently transparent balancing. That is my synthesis of the developer explanations, patches, and original mod investigations—not a claimed internal postmortem finding. [PC Gamer](https://www.pcgamer.com/games/sim/cities-skylines-2-boss-says-they-completely-overestimated-the-unity-engines-capabilities/)

---

## 7. Numbers: what can responsibly be inferred about scale?

Some widely repeated figures mix population capacity, moving instances, rendered objects, and actual throughput. Keep them separate.

| Verified figure | Interpretation |
| --- | --- |
| C:S1: **16,384 vehicle-buffer slots**, **32,768 parked-vehicle slots** | Constants explicitly identified in More Vehicles source. They are not citizen-population limits. [GitHub](https://raw.githubusercontent.com/dymanoid/MoreVehicles/master/src/MoreVehicles/Constants.cs) |
| More Vehicles: **65,536 vehicle slots** | A modded buffer capacity, not evidence that every PC can simulate that many vehicles smoothly. [GitHub](https://raw.githubusercontent.com/dymanoid/MoreVehicles/master/src/MoreVehicles/Constants.cs) |
| Real Time: approximately **65,500 population** recommended | A practical recommendation for that more demanding scheduling mod. [GitHub](https://github.com/dymanoid/RealTime) |
| C:S2: no predecessor-style hard agent cap advertised | Hardware-limited scalability is still bounded, and does not guarantee constant detail per person. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai) |

There is no defensible conversion from these numbers to “TCE will run 50,000 people at a particular speed.” The workloads differ, and I did not find a controlled primary comparison holding city topology, activity fidelity, hardware, and graphics settings constant.

For TCE, benchmark **completed agent-days, route requests, economic transactions, and simulation-time advancement**, not population alone.

---

## 8. Recommended design for TCE

The recommendations below are design proposals for your Rust/UE5 architecture, not descriptions of either Skylines game.

### A. Preserve people; instantiate movement and visuals separately

A person’s identity, household membership, relationships, health, knowledge, obligations, and possessions should remain authoritative in Rust.

An activity may create a trip. A trip may create a visible pedestrian or cart. Neither should own the person’s existence.

UE5 should receive presentation state and submit commands, never decide whether someone worked, ate, inherited land, or delivered goods. Running headless, zoomed out, or focused on another settlement should not change those outcomes.

Use stable generational handles, explicit lifecycle transitions, and bounded active tables. Preserve historical identity after death without retaining every expired activity object.

### B. Make movement consequential without simulating every animation

For an agrarian society, attendance and delivery are too important to remain decorative.

A farmer contributes productive time only when the activity can actually occur. A market purchase must reserve or transfer a real quantity. A grain shipment should distinguish reserved stock, stock in transit, received stock, loss, and cancellation.

A useful transaction contract is:

> Reserve → dispatch → transport → receive → settle, with explicit cancellation and loss rules.

This does not require simulating every footstep. A long journey can advance through scheduled events or road segments. What must survive abstraction is elapsed time, resource ownership, capacity use, risk, and arrival status.

### C. Optimize how often decisions happen before optimizing their arithmetic

Do not ask every person to find a better house, job, school, or market every tick.

Use scheduled reconsideration plus meaningful invalidations: unemployment, eviction, household formation, a new accessible market, major price changes, or a blocked route. Failed searches should back off until either a deadline or a relevant change.

For perspective, **50,000 agents reconsidering once per minute means about 833 reconsiderations per second**. Updating all of them at 10 Hz means 500,000 visits per second. These are arithmetic workload examples, not measured performance predictions.

For expensive searches, record candidates examined, graph nodes expanded, result quality, failure reason, and retry frequency. A fast search repeated unnecessarily is still a bad system.

### D. Use purpose-specific matching and reservations

An urgent medical request should prioritize expected response time and treatment capability. A grain shipment should consider quantity, price, transport capacity, spoilage, and trust. A job search should include skills, commuting feasibility, household obligations, and expected returns.

A shared request framework is useful, but its scoring policy should be task-specific.

Reservations must include quantity, owner, expiration, and cancellation behavior. Opportunistic work—such as a cart collecting goods along its route—must not consume capacity promised to its primary destination.

### E. Build a physical economy before adding elaborate finance

Start with land, labor-time, nutrition, fuel, tools, storage, transport, and production recipes. Add ownership, exchange, credit, taxation, and redistribution as explicit rules.

Early societies need not have universal wages or currency. A household may pool output; an estate may claim rent in grain; an institution may collect tribute and distribute food. All can use the same resource and obligation accounting underneath.

Every unexplained source or sink should be visible. Imports require an external producer or an explicitly modeled aggregate outside economy. Emergency assistance should name its provider and exhaustible reserves.

### F. Make demographic rates independent of update frequency

Define births, mortality, illness, and recovery in simulation-time units, not “chance per system update.”

For a constant hazard \(\lambda\) over an interval \(\Delta t\), use:

\[
p(\text{event during }\Delta t)=1-e^{-\lambda\Delta t}.
\]

Changing update frequency then need not change the intended event rate. Variable hazards require integration or suitable smaller intervals.

Stagger evaluations to avoid artificial midnight bursts. Generate immigrants as plausible households with varied ages and histories. Track age pyramids, fertility, mortality, household formation, and schooling transitions over long runs.

For TCE’s centuries-long worlds, calendar semantics must also be explicit. A visible day, an agricultural season, and a demographic year cannot silently use incompatible clocks.

### G. Keep the useful distinctions in the interface

A citizen inspector should explain something like:

> “Missed two work periods because the bridge was impassable. Household grain reserves fell below seven days. A purchase request is waiting for transport capacity.”

That explanation should come from recorded simulation facts, not a narrative model inventing causes afterward.

Service inspectors should expose queue length, oldest unresolved request, staff availability, missing supplies, reserved capacity, response-time distribution, and failed trips.

Economic inspectors should distinguish **demand for a resource**, **demand for a producer**, and **demand for additional building space**. Those are not interchangeable quantities.

### H. Validate causal chains with small, adversarial worlds

Before expanding content, test several compact scenarios:

| Scenario | Required invariant or observable consequence |
| --- | --- |
| Destroy the only bridge to a grain market | Deliveries stop or reroute; goods do not appear through cancellation. |
| Fill a clinic while demand remains high | Requests queue, seek alternatives, or expire explicitly. |
| Remove all suitable schools | Search retries remain bounded; agents retain a clear reason for failure. |
| Interrupt a transaction at every stage | Money, goods, and reservations remain reconcilable. |
| Run the same demographic model at different update frequencies | Rates remain consistent within defined tolerances. |
| Load a save after changing an economic rule | State migration and subsequent rebalancing are distinguishable and inspectable. |

Finally, test the complete Rust-to-UE5 build at **50,000 people early**. Measure CPU simulation throughput separately from render frame time, memory, synchronization stalls, and accelerated-time behavior. Rust cannot rescue an excessive search workload, and UE5 cannot make an unbounded visual workload inexpensive.

---

## 9. Source guide

The most useful starting points are the following. Official design diaries document intended mechanics; patch notes document acknowledged defects and changes; mod repositories document version-specific implementation findings.

| Source | Best use |
| --- | --- |
| [C:S1 Developer Diary 7: Simulation — archived developer text](https://sims4network.wixsite.com/sims4network/single-post/2015/01/07/cities-skylines-dev-diary-7-simulation) | Persistent citizens, deliberately reduced occupancy, freight, and proximity versus dispatched services. |
| C:S2 official diaries: [Citizen Lifepath](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/citizen-simulation-lifepath?utm_source=chatgpt.com), [Traffic AI](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai?utm_source=chatgpt.com), [Economy](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/economy-production?utm_source=chatgpt.com), [Services](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/city-services-districts-policies) | Intended system mechanics and explicit comparisons with C:S1. |
| [“Tapping the Entity Component System for Cities: Skylines II” — Unite 2024](https://www.youtube.com/watch?v=nEkIyWhvq3o) | Developer talk on the ECS architecture. |
| [Paavo Huhtala’s launch-era performance investigation](https://blog.paavo.me/cities-skylines-2-performance/?utm_source=chatgpt.com) | Original technical inspection of architecture and rendering workload. |
| [March 2026 developer interview](https://www.pcgamer.com/games/sim/cities-skylines-2-boss-says-they-completely-overestimated-the-unity-engines-capabilities/?utm_source=chatgpt.com) | Retrospective discussion of engine assumptions and integration risk. |
| [Economy 2.0, Part One](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/dev-diary-economy-part-one?utm_source=chatgpt.com) and [Part Two](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/dev-diary-economy-part-two?utm_source=chatgpt.com) | Economic redesign, landlord removal, upkeep, and existing-save transitions. |
| [Real Time](https://github.com/dymanoid/RealTime?utm_source=chatgpt.com), [Lifecycle Rebalance](https://github.com/algernon-A/Lifecycle-Rebalance-Revisited), [TransferManagerCE](https://github.com/Sleepy334/TransferManagerCE?utm_source=chatgpt.com), [TM:PE](https://github.com/CitiesSkylinesMods/TMPE?utm_source=chatgpt.com) | C:S1 schedules, demographics, service matching, and traffic implementation. |
| [RealEco](https://github.com/Infixo/CS2-RealEco) and [Traffic Simulation Adjuster](https://github.com/fesdonomist/TrafficSimulationAdjuster) | Original C:S2 modding findings; read with their historical version context. |
| [First Frost patch notes](https://www.paradoxinteractive.com/games/cities-skylines-ii/news/patch-notes-first-frost?utm_source=chatgpt.com) and [official announcement archive](https://steamcommunity.com/app/949230/announcements/?l=english&utm_source=chatgpt.com) | Demographic corrections and the latest simulation changes. |

**Bottom line:** borrow Skylines’ separation of persistent population from active presentation, but give TCE a stricter causal contract. A person may be cheap to update and a journey cheap to display; neither should become economically imaginary when the system is busy.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927cd-03a0-83ea-877d-210d9be2dbc9)
