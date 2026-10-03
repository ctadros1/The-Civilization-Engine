# GlassBox: lessons for The Civilization Engine

## Executive assessment

**GlassBox is a warning about choosing the wrong simulation entities—not evidence that agent-based simulation is inherently unworkable.**

Its central abstraction was an effective resource-flow system: buildings stored quantities, rules transformed them, and mobile agents transported them between places. However, the shipping game did not consistently represent citizens as enduring people with particular homes and employers. Its developers explicitly described that omission as a performance and game-design trade-off. [GlassBox GDC 2012 Slides](https://drive.google.com/file/d/1ZFDjvndYZCJsRujC2tmjCOG12JUrxAY3/view) [Gamer Horizon](https://gamerhorizon.com/2013/04/05/simcity-one-month-later/)

For TCE, distinguish three problems that are often conflated:

| Problem | What needs fixing |
| --- | --- |
| A commuter returns to an available house rather than their own home. | **Persistent identity and relationships.** |
| Numerous vehicles pursue the same remaining vacancy or service request. | **Assignment and capacity reservation.** |
| Vehicles choose poor routes, cannot leave garages, or mishandle intersections. | **Route costs and movement execution.** |

Better pathfinding cannot supply a missing home relationship. Persistent identity cannot, by itself, prevent ten workers from claiming one vacancy. Both can be correct while intersection handling remains broken.

**Recommendation:** adopt GlassBox’s compositional rules, explicit resource transfers, and causal visualization. Do not adopt interchangeable citizens, destination selection disguised as routing, or population statistics that obscure what is actually simulated.

---

## 1. How GlassBox worked

### 1.1 The foundation: quantities and transformations

Andrew Willmott’s GDC 2012 presentation is the clearest primary description of the architecture. It predates release, so it describes the framework and intended mechanisms—not necessarily every detail of the final or subsequently patched game. Willmott also notes that the engine initially targeted possible mobile games before development shifted toward SimCity; that history does **not** establish that its later problems were caused by a mobile-first origin. [Andrew Willmott](https://www.andrewwillmott.com/talks/inside-glassbox)

The slides describe these components:

| Component | Concrete meaning |
| --- | --- |
| **Resources** | Integer quantities held in capacity-limited bins. Examples include coal, water, money, electricity, labor, pollution, and happiness. |
| **Units** | Stateful objects such as houses and factories, with resource bins and spatial footprints. The architecture could also represent people as units. |
| **Maps** | Uniform grids of resource bins representing environmental properties: deposits, vegetation, pollution, land value, and desirability. |
| **Globals** | Quantities associated with the simulation as a whole. |
| **Rules** | Operations that consume, produce, or transfer resources. A rule applies only when its complete result is valid. |
| **Agents** | Mobile resource carriers emitted by units and delivered to other units by transport handlers. |
| **Paths and zones** | Paths provide transport networks; zones run rules that create, upgrade, downgrade, or remove units. |

Units could run rules against their own bins, global bins, covered map cells, or nearby units. Map rules also supported operations such as diffusion and wind-driven advection. This was a general simulation construction kit, rather than a separate hard-coded implementation for every city subsystem. [GlassBox GDC 2012 Slides](https://drive.google.com/file/d/1ZFDjvndYZCJsRujC2tmjCOG12JUrxAY3/view)

A useful feature was **transactional rule application**. A production rule should not consume its inputs and then discover that the output has nowhere to go. GlassBox checked whether the whole operation was valid. Its demonstration factory combined material consumption, product creation, pollution, and audiovisual feedback. One example ran every **10 simulation ticks**—an illustrative rule interval, not a published engine-wide frequency. [GlassBox GDC 2012 Slides](https://drive.google.com/file/d/1ZFDjvndYZCJsRujC2tmjCOG12JUrxAY3/view)

For TCE, this is directly relevant to authored building blocks. A mill, kiln, workshop, or administrative office can expose explicit prerequisites and effects without embedding every behavior in bespoke engine code.

### 1.2 “Agent” did not necessarily mean “person”

The decisive implementation detail is that transport agents were deliberately lightweight. The slides say they carried resource bins but did **not** execute the unit-rule system, because there could be tens of thousands of them. Transport handlers managed their movement and delivery. [GlassBox GDC 2012 Slides](https://drive.google.com/file/d/1ZFDjvndYZCJsRujC2tmjCOG12JUrxAY3/view)

For example, the presentation’s work-trip rule transfers **two units of people** from a building into an agent traveling by car toward a work destination. Consequently:

* One transport agent could carry multiple units of a resource.
* A moving agent was not necessarily one citizen.
* An agent carrying people did not necessarily establish a persistent person record.

This distinction also applies to nonhuman agents carrying utilities or goods. Counting GlassBox agents is therefore not equivalent to counting individually simulated inhabitants. [GlassBox GDC 2012 Slides](https://drive.google.com/file/d/1ZFDjvndYZCJsRujC2tmjCOG12JUrxAY3/view)

Conceptually, a journey was:

> A source unit emits a carrier → a transport handler delivers it to a compatible destination → resource quantities enter that destination’s simulation.

That is a good abstraction for a shipment. It is insufficient for a person unless the shipment also refers to an enduring individual whose household, possessions, obligations, and history survive arrival.

Importantly, **the framework did not make persistence impossible**. Its unit model could represent people. The problem was how the shipping city simulation used the abstractions, not a theorem that resource-based engines cannot contain persistent individuals.

### 1.3 Routing: shared fields toward eligible destinations

GlassBox’s GDC routing description is more sophisticated than the familiar criticism that it “just chose the shortest road.”

It used a **virtual distance field**, based on D\* Lite-style wavefront updates, to calculate the cost of reaching the nearest suitable sink from network vertices. Agents followed the direction of lower cost. The slides explicitly state **“No per-agent routing info”**, and identify sink capacity, congestion, and speed limits as possible cost modifiers. [GlassBox GDC 2012 Slides](https://drive.google.com/file/d/1ZFDjvndYZCJsRujC2tmjCOG12JUrxAY3/view)

A simplified interpretation—not recovered Maxis source code—is:

\[
D(v)=\min\_{s\in\text{eligible destinations}}
\left[\operatorname{routeCost}(v,s)+\operatorname{destinationPenalty}(s)\right].
\]

Rather than independently calculating a complete route for every carrier, the system could share destination information across many carriers.

The underlying incremental-search idea is sound: D\* Lite reuses information from previous searches when relevant costs change. The original algorithm is not a prescription to give citizens interchangeable homes. [AAAI](https://cdn.aaai.org/AAAI/2002/AAAI02-072.pdf)

The semantic problem appears when the destination means:

> “An available home of an appropriate category.”

That is fundamentally different from:

> “The dwelling belonging to this person’s household.”

An excellent implementation of the first request will still behave implausibly when presented as the second.

### 1.4 What the architecture did well

GlassBox put several valuable ideas together: object-local simulation logic, data-driven definitions, hotloading, resource availability checks, spatial fields, and visual effects triggered by successful simulation operations. Its stated goal was **“What You See Is What You Sim”**: show underlying events rather than unrelated visual approximations. [GlassBox GDC 2012 Slides](https://drive.google.com/file/d/1ZFDjvndYZCJsRujC2tmjCOG12JUrxAY3/view)

My assessment is that these remain strong foundations for TCE. A workshop should visibly stop because its actual labor or materials are unavailable. A delivery should correspond to an actual shipment. Construction should reflect a real project’s progress.

The qualification is crucial: **a truthful depiction of a resource transfer is not necessarily a truthful depiction of a human life.**

---

## 2. What failed—and what the developer evidence establishes

### 2.1 Citizens lacked enduring homes and employment

Lead designer Stone Librande acknowledged that Sims did not own particular houses or have permanent employment. Names and appearance attributes were not tracked persistently either. He explained that the team prioritized performance and more Sims over what it considered additional micro-level detail. However, he also said that happiness, money, sickness, and education persisted and traveled with Sims. It would therefore be inaccurate to say that absolutely no citizen-related state survived. [Gamer Horizon](https://gamerhorizon.com/2013/04/05/simcity-one-month-later/)

Contemporary firsthand analysis described evening commuters entering the first available residential buildings appropriate to their wealth, sometimes filling driveways in sequence. Following a particular visible Sim did not provide a continuous biography beyond the immediate journey. [Gamer Horizon](https://gamerhorizon.com/2013/04/05/simcity-one-month-later/)

**Root cause:** the simulation preserved quantities and some attributes, but not the relationships needed to explain an individual’s life.

For TCE, a home is not merely somewhere with spare residential capacity. It connects a person to a household, property or occupancy rights, possessions, dependents, and neighbors. Likewise, work can connect someone to land, equipment, skill development, customers, contractual obligations, and political interests.

Those connections are not optional cosmetic detail in a civilization simulation. They are mechanisms through which yesterday affects tomorrow.

### 2.2 Destination selection produced flocking

The most revealing later fix concerns **reservation**, not faster shortest-path search.

In his explanation of the changes planned for Update 7, transportation engineer Alexander Harkness described allowing vehicles to claim capacity at a destination well before arrival. Fully claimed destinations could then be removed from the routing map for other vehicles. Previously, vehicles could converge on the same destination and only change their decision later. [PCGamesN](https://www.pcgamesn.com/simcity/simcity-update-7-brings-further-improvements-traffic-no-more-rubbish-truck-clumping)

The corresponding update notes targeted convergence across numerous vehicle types, including fire, police, garbage, recycling, buses, freight, and ordinary civilian traffic. They also specified that freight trucks should leave factories only when a valid delivery destination existed. [SimsNetwork](https://www.simsnetwork.com/news/2013/08/07/update-7-brings-more-traffic-fixes)

A minimal example explains the failure:

Suppose a destination has ten available places and 100 agents can all see those ten places. If availability changes only when agents arrive, many can begin traveling toward the same opportunity. Local decisions are individually understandable, but their combined result is wasteful.

With reservations, the first ten successful claims exhaust the capacity before the vehicles arrive. Later agents consider other destinations.

**Engineering diagnosis:** this is an allocation problem coupled to routing. It is not evidence that the shortest-path algorithm itself is incapable of finding good paths.

For services, the same distinction applies to assigning one crew to a small incident rather than sending every idle crew toward the currently most attractive request.

### 2.3 Route costs and local traffic behavior also needed work

The movement layer had genuine defects and restrictive rules. Harkness described lane selection for turns, overtaking, restrictions on U-turns, and prohibitions on crossing avenue medians to enter buildings. Those rules could force longer journeys; upgrading a road to an avenue was not automatically beneficial. Emergency vehicles had also needed changes so that they could reserve entry space outside garages rather than wait indefinitely for a gap. [BeyondSims](https://beyondsims.com/2013/03/interesting-blog-posting-about-traffic-on-simcitys-page/)

Patch 1.7, released March 19, 2013, explicitly addressed congestion and intersection behavior, allowed emergency vehicles to use empty lanes around intersection traffic, and gave emergency and delivery vehicles priority when leaving garages. [GameSpot](https://www.gamespot.com/articles/new-simcity-update-addresses-traffic-issues/1100-6405594/)

These are separate layers:

> Choose the purpose → choose the destination → secure any required capacity → calculate a route → execute movement.

A city can fail at any one of them. Calling every failure “bad AI” conceals where the repair belongs.

Also, the pre-release architecture already allowed congestion costs. The defensible historical conclusion is that the shipped behavior needed tuning and implementation changes—not that GlassBox’s architecture could only minimize physical distance. [GlassBox GDC 2012 Slides](https://drive.google.com/file/d/1ZFDjvndYZCJsRujC2tmjCOG12JUrxAY3/view)

### 2.4 The visual promise exceeded the underlying social model

GlassBox made movement consequential and inspectable, but that increased the significance of inconsistencies.

My interpretation is that a visibly identifiable commuter invites longitudinal questions: Where do they live? Who depends on them? Why are they changing jobs? A simulation can answer “this carrier has delivered labor” without being able to answer those questions.

Ocean Quigley’s GDC 2013 session explicitly framed SimCity’s art as both an illusion of a living city and a way to communicate the underlying simulation. That combination is powerful, but it makes mismatches between presentation and underlying state especially important. [GDC Vault](https://gdcvault.com/play/1019107/Building-SimCity-Art-in-the)

For TCE, an inspectable person should therefore be a real persistent entity before the renderer supplies a face, nameplate, or follow-camera.

### 2.5 What the talks and post-release explanations actually cover

| Evidence | What it establishes |
| --- | --- |
| **Willmott, GDC 2012** | The resource/unit/rule framework, lightweight transport agents, shared destination fields, and visualization philosophy. |
| **Moskowitz, GDC 2013** | A development methodology based on building systems, discovering interesting behavior, and connecting simulation to tools and feedback. |
| **Quigley, GDC 2013** | Art and authoring techniques for a composable city whose appearance communicates simulation. |
| **March 2013 developer statements and Patch 1.7** | The intentional persistence trade-off and concrete traffic execution repairs. |
| **August 2013 Update 7 explanation and notes** | Earlier destination claims and broader anti-flocking changes. |
| **October 2013 “State of SimCity”** | The unsuccessful investigation into larger cities and the decision to stop that work. |

The 2013 GDC session descriptions are useful context, but they should not be treated as comprehensive launch-failure postmortems. I verified their published abstracts, not the complete recordings. The more specific failure evidence comes from developer statements, patch notes, and firsthand observations. [GDC Vault](https://gdcvault.com/play/1017948/Exploring-SimCity-A-Conscious-Process)

---

## 3. Numbers: what can—and cannot—be inferred

### City area and regional scale

The city plot was **2 km × 2 km**, or **4 km²**, with larger-scale play organized around multiple cities rather than one contiguous metropolis. Producer Jason Haber described the dimensions before release; Quigley described city size as a performance decision made for mainstream PCs. [Gamer](https://www.gamer.ne.jp/news/201212140083/)

A December 2012 developer AMA advertised hundreds of thousands of simultaneous agents, thousands of buildings, and regions containing **16 active cities**. The same AMA explained that most population was inside buildings rather than moving on streets. These are developer scale claims—not reproducible benchmarks demonstrating hundreds of thousands of persistent people or 16 fully detailed cities running continuously on one PC. [Reddit](https://www.reddit.com/r/IAmA/comments/14umm1/we_are_the_simcity_dev_team_from_maxis_amaa/)

### Larger maps were investigated, not merely withheld

In October 2013, Maxis studio general manager Patrick Buechner said the team had spent months investigating larger cities, including terrain, routing, and GlassBox’s processing of larger spaces. He reported that acceptable performance could not be achieved for most players within the engine’s constraints, and that work on larger cities was being stopped. Some resulting optimizations would still benefit the existing game. [ModDB](https://www.moddb.com/games/simcity-2013/news/state-of-simcity)

This is evidence of a real limitation of that implementation and its target hardware. It does **not** establish a universal population or map-size ceiling for agent-based simulation.

### Displayed population was not the underlying population count

Community reverse-engineering exposed a UI function named `GetFudgedPopulation`. It left counts up to 500 unchanged, applied a nonlinear transformation above that, and used an **8.25 multiplier** above an underlying count of 40,845. [Arqade](https://gaming.stackexchange.com/questions/109072/what-is-the-formula-for-converting-population-to-residential-agents-workers-sh)

Calculated examples from that function:

| Underlying count supplied to the function | Displayed population |
| --- | --- |
| 500 | 500 |
| 10,000 | 59,829 |
| 50,000 | 412,500 |

The function is evidence of display inflation. It does not, by itself, specify how many mobile objects were instantiated simultaneously, nor how many persistent identities existed. Therefore, comparisons such as “SimCity simulated 400,000 people, so TCE’s 50,000 must be easy” are invalid. [Arqade](https://gaming.stackexchange.com/questions/109072/what-is-the-formula-for-converting-population-to-residential-agents-workers-sh)

### A useful local throughput figure

Harkness reported **4–5 times greater intersection throughput** with traffic lights than with all-way stops in developer testing. This concerns one traffic-control comparison, not a 4–5× increase in overall simulation performance. [BeyondSims](https://beyondsims.com/2013/03/interesting-blog-posting-about-traffic-on-simcitys-page/)

That figure is nevertheless instructive: movement rules and intersection capacity can matter as much as the speed of route computation.

**Evidence gap:** the material reviewed does not establish a reliable hard agent cap, per-agent memory footprint, milliseconds per simulation tick, or reproducible throughput benchmark suitable for predicting TCE’s performance.

---

## 4. What TCE should do differently

The recommendations below are architectural proposals for TCE, not claims that GlassBox implemented them.

### 4.1 Make a person an enduring entity; make a journey temporary

The authoritative Rust state should distinguish:

| Record | Lifetime and purpose |
| --- | --- |
| **Person** | Enduring identity, life status, capabilities, needs, relationships, and personal history. |
| **Household** | Membership, shared resources, dependents, and occupancy arrangements. |
| **Commitment or role** | Employment, cultivation rights, apprenticeship, office, military duty, or another obligation. |
| **Activity** | The person’s present undertaking and its expected completion or interruption conditions. |
| **Journey** | A temporary movement record connecting that person to an exact destination. |
| **Render representation** | A disposable visual representation of authoritative simulation state. |

Entering a building should end or suspend a journey—not destroy the person.

Likewise, leaving a building should schedule movement for existing people—not manufacture new anonymous residents from a population count.

A useful invariant is:

> Every living person exists exactly once and has one authoritative location or in-transit state, whether or not they are visible.

Residence and physical presence must be separate. An absent household member does not make their bed available to a stranger.

For early agrarian TCE, avoid replacing GlassBox’s simplification with an equally rigid modern employment model. People may combine seasonal cultivation, household production, paid work, care, and public obligations. Persistent **roles and commitments** are more general than a single permanent `employer_id`.

### 4.2 Separate choosing a destination from getting there

For a new activity, use this sequence:

> Determine the need → consider eligible opportunities → choose a destination → acquire the required commitment or reservation → start the journey → validate and commit arrival.

Different activities require different semantics.

Returning home usually references an existing household dwelling. Going to work references an existing role and current worksite. Finding a new home or employer is a distinct decision. Seeking emergency shelter can legitimately search for the nearest acceptable available place.

A destination reservation should identify its claimant, resource or capacity, quantity, validity period, and cancellation conditions. Admission must be atomic: two parallel decisions cannot both obtain the final place.

But do not make everything an expiring reservation. A dwelling right, landholding, apprenticeship, or office is a durable relationship. A shop queue position or unloading slot is temporary capacity.

This distinction prevents a subtler version of the GlassBox problem: treating people’s established social arrangements as vacancies that must be reclaimed every morning.

### 4.3 Preserve shared routing without sharing away individuality

Persistent destinations do not require a unique expensive planner for every citizen.

TCE should share work wherever the result remains valid: cached corridors between areas, common routes to major sites, destination-region searches, and local path segments. The individual journey still retains its exact target and its reason for traveling.

For example:

> These 300 people can share the route toward a neighborhood, while retaining 180 distinct final household destinations.

The optimization boundary is the route calculation, not the identity of the traveler or the meaning of the trip.

Replanning also needs two distinct operations:

**Route replanning:** the bridge closed; find another way to the same destination.

**Activity replanning:** the destination is no longer usable; choose a different activity, workplace, market, or shelter under explicit rules.

A blocked road should not silently reassign someone’s home.

### 4.4 Add stability and limited knowledge to decisions

My recommendation is to avoid having every person reevaluate every opportunity against the same perfectly current global score.

Instead, ordinary decisions should depend on established commitments, known opportunities, personal preferences, travel expectations, and switching costs. Reconsideration should occur at meaningful times: unemployment, household change, repeated shortages, seasonal transitions, or a sufficiently attractive new opportunity.

This produces a straightforward explanation for stable behavior:

> “She continues working here because the wage is acceptable, the commute is familiar, and changing work has a cost.”

It also permits justified instability:

> “The harvest failed twice, the household has relatives elsewhere, and migration now offers a better prospect.”

Not every person needs sophisticated deliberation. Durable relationships plus inexpensive activity selection can provide continuity. Notables can deliberate more deeply while using the same validated action and commitment system.

### 4.5 Reuse resource rules, but retain domain-specific meaning

GlassBox’s common resource framework is worth adapting. However, TCE should not force every domain into anonymous interchangeable quantities.

For production, use typed inventories, capacities, prerequisites, and transactions. For labor, distinguish a person’s availability from their identity and commitments. For disease, retain the affected person and relevant exposure history. For property and offices, retain ownership, authority, succession, and provenance.

A single scalar called “legitimacy” may help summarize a situation, but should not replace the institutions and relationships that cause political behavior.

Similarly, transport abstractions should match their domain. A cart shipment, water flow, electricity supply, and human journey need not share identical movement rules merely because all move something between locations.

The goal is **common infrastructure without erased meaning**.

### 4.6 Optimize update frequency before removing persistence

Persistent identity does not imply that every person must think or move every render frame.

An indoor worker can retain a scheduled completion time. Needs can advance according to elapsed simulation time. Decisions can be staggered. A distant journey can retain its route, progress, capacity effects, and expected arrival without requiring full visual locomotion.

For TCE’s Rust/Unreal split, the kernel should remain authoritative. Creating, removing, or simplifying a visual representation should not change a person’s household, work, possessions, or progress.

At high simulation speeds, coarser movement must still preserve the consequences that matter: journey duration, transport capacity, blocked connections, late arrival, and missed activities. Hiding a traveler is not permission to deliver them instantly.

Illustrative arithmetic helps separate storage from computation:

| Assumption—not a measured TCE result | Consequence at 50,000 people |
| --- | --- |
| 256 bytes of core per-person state | 12.8 MB |
| 1 KiB of core per-person state | 51.2 MB |
| Four journeys per person per simulated day | 200,000 journeys per day |
| One simulated day per wall-clock minute | About 3,333 journey starts per second |

Those memory figures exclude routes, indexes, relationships, inventories, and history. They demonstrate only that an identity record is not inherently enormous.

The long-horizon calculation is more sobering: even **one event per person per day**, across 50,000 people and 100 years of 365 days, produces **1.825 billion event executions**. Fast-forward performance depends heavily on what must actually be processed—not merely how compactly names and home IDs are stored.

For endless worlds, keep active state bounded and distinguish compact historical records from high-volume transient logs. Do not retain every movement sample forever.

### 4.7 Treat settlement boundaries as transfers, not respawning

GlassBox’s framework described boxes communicating through packages, including region-level organization. That is a useful decomposition pattern, but it does not automatically provide continuous identity across simulations. [GlassBox GDC 2012 Slides](https://drive.google.com/file/d/1ZFDjvndYZCJsRujC2tmjCOG12JUrxAY3/view)

When TCE eventually partitions settlements or runs some at lower detail, moving a person between them should preserve the same identity, relationships, property claims, and relevant history.

A transfer must not leave one copy behind while creating another at the destination. Nor should a named migrant become an anonymous population increment that is later reconstructed as a different person.

### 4.8 Test the promises players can inspect

A small set of targeted tests can catch the most relevant GlassBox-style failures before the city becomes complicated.

| Test | Required outcome |
| --- | --- |
| **Follow 100 people for 30 days** | Identity, household, roles, possessions, and relationships remain continuous unless an explicit event changes them. |
| **One remaining vacancy, many applicants** | No over-allocation; unsuccessful applicants receive a clear result rather than all traveling to claim it. |
| **Several simultaneous service requests** | Assignments respect required effort and available crews rather than sending every crew to one target. |
| **Close a bridge mid-journey** | The route changes, or the activity fails explicitly; the person’s home and identity do not change. |
| **Empty the streets into buildings** | Population remains accounted for; indoor people are not destroyed. |
| **Change camera position and render detail** | Authoritative simulation outcomes remain unaffected by visual interest management. |
| **Death, migration, or household dissolution** | Roles, possessions, occupancy, dependents, and outstanding reservations are reconciled consistently. |

The UI should expose the same causal records used by the simulation: destination, purpose, commitment, reservation, expected arrival, and interruption reason.

A plausible explanation should come from actual state—not a narrative invented after an inexplicable action.

---

## 5. Source guide

Several original SimCity blog addresses no longer returned usable articles during this research. Where necessary, the links below point to contemporaneous reproductions of developer statements or official notes; those are distinguished from firsthand community analysis.

### Architecture and developer talks

| Source | Why it matters |
| --- | --- |
| [Andrew Willmott — Inside GlassBox, GDC 2012](https://www.andrewwillmott.com/talks/inside-glassbox?utm_source=chatgpt.com) | Author’s page linking the original slides and demonstration videos. The principal architectural source. |
| [Dan Moskowitz — Exploring SimCity: A Conscious Process of Discovery, GDC 2013](https://gdcvault.com/play/1017948/Exploring-SimCity-A-Conscious-Process?utm_source=chatgpt.com) | Simulation development, discovery-based iteration, player tools, and feedback. |
| [Ocean Quigley — Building SimCity: Art in the Service of Simulation, GDC 2013](https://gdcvault.com/play/1019107/Building-SimCity-Art-in-the?utm_source=chatgpt.com) | Composable city art and communicating underlying state. |
| [Maxis developer AMA, December 2012](https://www.reddit.com/r/IAmA/comments/14umm1/we_are_the_simcity_dev_team_from_maxis_amaa/?utm_source=chatgpt.com) | Direct developer answers about scale, agent counts, rendering, and design constraints. |

### Failures, repairs, and scale

| Source | Why it matters |
| --- | --- |
| [Gamer Horizon — SimCity: One Month Later](https://gamerhorizon.com/2013/04/05/simcity-one-month-later/?utm_source=chatgpt.com) | Reproduces Librande’s persistence explanation and supplies firsthand behavioral observations. |
| [Alexander Harkness — Under the Hood of SimCity’s Traffic System, reproduced by BeyondSims](https://beyondsims.com/2013/03/interesting-blog-posting-about-traffic-on-simcitys-page/?utm_source=chatgpt.com) | Detailed vehicle, lane, intersection, and emergency-access mechanics. |
| [Patch 1.7 notes, reproduced by GameSpot](https://www.gamespot.com/articles/new-simcity-update-addresses-traffic-issues/1100-6405594/?utm_source=chatgpt.com) | Concrete March 19 traffic and garage-exit changes. |
| [Harkness’s Update 7 reservation explanation, quoted by PCGamesN](https://www.pcgamesn.com/simcity/simcity-update-7-brings-further-improvements-traffic-no-more-rubbish-truck-clumping?utm_source=chatgpt.com) | The clearest explanation of why earlier destination claims prevent flocking. |
| [Update 7 notes, reproduced by SimsNetwork](https://www.simsnetwork.com/news/2013/08/07/update-7-brings-more-traffic-fixes?utm_source=chatgpt.com) | Broad vehicle-assignment changes and valid freight destinations. |
| [Patrick Buechner — State of SimCity, reproduced on ModDB](https://www.moddb.com/games/simcity-2013/news/state-of-simcity?utm_source=chatgpt.com) | The larger-city investigation and its performance-related cancellation. |
| [Arqade — Population conversion reverse-engineering](https://gaming.stackexchange.com/questions/109072/what-is-the-formula-for-converting-population-to-residential-agents-workers-sh?utm_source=chatgpt.com) | Original community analysis containing the population display function. |

### Algorithmic reference and orientation

[Koenig and Likhachev — D\* Lite, AAAI 2002](https://cdn.aaai.org/AAAI/2002/AAAI02-072.pdf?utm_source=chatgpt.com) is the primary algorithmic reference for the incremental-search family named in the GlassBox slides. The [SimCity (2013) Wikipedia entry](https://en.wikipedia.org/wiki/SimCity_(2013_video_game)?utm_source=chatgpt.com) is useful as a chronology and source index, rather than as an implementation specification.

---

## Final design judgment

TCE should retain GlassBox’s strongest idea: **visible events should correspond to real causal changes in simulation state.**

But TCE’s foundational entities must be different. Households, people, land rights, work relationships, offices, and commitments must endure independently of the transport and rendering systems.

**Optimize the journey, the update schedule, and the visual representation. Do not optimize away the person.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927ca-7aa0-83e9-8628-c642d4ad92f0)
