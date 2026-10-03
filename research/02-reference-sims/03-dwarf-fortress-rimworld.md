# Emergent societies and stories: lessons from Dwarf Fortress and RimWorld

## Executive conclusion

**TCE should borrow Dwarf Fortress’s continuity and RimWorld’s legibility—but not RimWorld’s authority to manufacture dramatic events.**

Dwarf Fortress generates much of its history by evolving a world containing civilizations, settlements, historical figures, offices, relationships, and artifacts. Its strength is that these things remain connected: a person belongs to an institution, holds an office, creates an object, dies, and leaves consequences that another character can encounter. However, much of its world population is represented abstractly rather than as fully active individuals. It is not evidence that tens of thousands of fortress-detail citizens can run cheaply. [Game Developer](https://www.gamedeveloper.com/design/q-a-dissecting-the-development-of-i-dwarf-fortress-i-with-creator-tarn-adams)

RimWorld explicitly combines systemic character behavior with an AI storyteller that selects incoming incidents. Its official description emphasizes generating memorable situations rather than maintaining a neutral, independently evolving world. That is highly effective for a colony game, but conflicts with TCE’s requirement that societies make their own history. [rimworldgame.com](https://rimworldgame.com/)

The architecture I recommend is:

**Autonomous simulation → persistent consequences and causal records → read-only story discovery → observer interface.**

The simulation determines what happens. The observer layer determines what deserves attention. Keeping those responsibilities separate is the most important lesson in this report.

---

## 1. Dwarf Fortress: history as a persistent, interconnected world

### World generation is more than generating terrain and writing a chronology

Tarn Adams describes world generation as first constructing geography—elevation, rainfall, temperature, drainage, rivers, biomes, and related fields—then running a loose strategy simulation involving civilizations and historical actors. History is produced by changes to the world and recorded alongside them, rather than generated solely as decorative prose. [Game Developer](https://www.gamedeveloper.com/design/q-a-dissecting-the-development-of-i-dwarf-fortress-i-with-creator-tarn-adams)

The useful conceptual model is a collection of persistent entities:

| Concept | What it represents | Why it matters for TCE |
| --- | --- | --- |
| **Civilization or organization** | A collective with membership, cultural properties, relationships, and positions. | A society must survive individual leaders and changes of territory. |
| **Site** | A geographically located settlement or other persistent place. | A city should retain its identity through conquest, abandonment, rebuilding, and cultural change. |
| **Position** | An office with responsibilities, eligibility, succession, and sometimes material requirements. | Authority should belong to an institutionally defined role, not merely a character attribute. |
| **Historical figure** | An individually tracked person or creature with relationships and a recorded life. | Identity must persist across travel, retirement, promotion, and changes in simulation detail. |
| **Artifact or cultural work** | A persistent object or creation associated with makers, owners, places, and events. | Material and cultural inheritance make history observable after its original participants die. |

These distinctions are reflected in DF’s civilization, position, historical-figure, and Legends documentation. Crucially, a civilization is not synonymous with a settlement, and a position is not synonymous with its incumbent. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Entity)

### Offices are a particularly valuable precedent

DF’s data-defined positions can specify appointment or election, succession by heir or another position, responsibilities, population requirements, and required rooms. This makes an office a bundle of institutional rules and privileges rather than just a title. However, individual responsibility tokens vary in implementation; their existence does not establish that a complete corresponding governmental subsystem exists. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Position_token)

There is also an important nuance: **not every office is fixed in advance**. Community documentation describes *variable positions* created during world generation. Human lawgiver positions, for example, can have recorded origins involving collaboration, popular support, threats, or argument. Their holders can establish further positions. This is procedural institutional formation within a bounded vocabulary—not simply selecting a ruler from a static list, but also not a general simulation of constitutional reasoning. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Variable_positions)

For TCE, adopt that separation and extend it:

An office should define jurisdiction, powers, obligations, selection, removal, succession, compensation, and enforcement. A person occupies the office for an interval. The organization, office, and succession rules remain after that person leaves.

This enables institutional stories without prewritten plots: a temporary emergency office becomes hereditary; a council refuses to recognize a successor; two institutions claim authority over the same market.

### Historical figures are not the entire population

DF’s world-scale population includes abstract populations alongside explicitly tracked historical figures. Nobles and other significant actors are individually represented; local play can introduce or promote additional individuals into that persistent historical representation. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Historical_figure)

This distinction matters enormously for TCE:

**Persistence, behavioral detail, and historical prominence are three separate decisions.**

A person can retain identity, kinship, property, and commitments without requiring continuous fine-grained processing. Conversely, an event appearing in Legends does not prove that all the movement, deliberation, production, and communication behind it were individually simulated.

TCE’s 50,000 citizens should remain individually identifiable even when their updates are infrequent. Becoming notable should increase the detail of observation and perhaps deliberation—not retroactively create their identity or resources.

### Culture provides a better model for technology than a global unlock tree

One especially instructive developer example appears in DF’s March 2015 development log. In a generated world, a human named Usmen learned from a goblin poet, developed a new poetic form in year 106, and transmitted it through a performance troupe. Forty years after his death, eight current troupe members still knew the form. The result depended on teaching, membership, movement, and survival—not merely a civilization-wide flag. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2015.html)

That pattern is directly applicable to TCE’s technologies, architectural styles, legal doctrines, and religious practices:

**Creation → acquisition by particular people → practice → teaching or copying → diffusion → possible loss.**

But do not overread DF’s precedent. Its 2015 scholarship implementation explicitly did not make discovered knowledge unlock fortress buildings or jobs. DF’s development roadmap also distinguishes existing systems from ambitions for deeper law, property, and world-economy simulation. Neither should be presented as a demonstrated solution to TCE’s entire endogenous-development problem. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2015.html)

---

## 2. How individual minds turn events into stories

### Dwarf Fortress: several interacting systems, not one happiness meter

DF’s personal needs, physiological requirements, focus, emotions, stress, personality, and memories should not be collapsed into a single variable. Community documentation explicitly distinguishes need satisfaction and focus from happiness or stress: a dwarf can be emotionally content but distracted by unmet personal needs. Needs depend on personality and values, producing different demands for activities such as socializing, worship, learning, and creating. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Need)

Its 2018 memory work illustrates a useful bounded architecture. The developer described eight short-term emotional memories and eight long-term memories, with important experiences surviving longer than minor pleasures. Remembering an event could produce further emotional consequences, and sufficiently important experiences could change traits or values permanently. The intention was that grief over a loved one should not simply disappear beneath repeated appreciation of furniture. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2018.html)

The practical story-producing loop is therefore:

**An event happens → its personal significance is evaluated → an emotion or memory changes state → later behavior changes → new events become possible.**

Relationships make the appraisal personal. A death is not merely a global “death occurred” stimulus: who died, the observer’s connection to them, and the observer’s prior experiences matter. DF’s memory rebalancing specifically distinguished significant experiences from repeated horror involving strangers’ corpses. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2018.html)

For TCE, the important design distinction is between:

* **Physiology:** hunger, fatigue, pain, illness.
* **Current affect and attention:** emotional state, distraction, acute stress.
* **Durable dispositions:** values, preferences, habits, loyalties.
* **Autobiographical memory:** selected experiences that can influence later appraisal and action.

Those layers should have different update rates and retention policies. A historical archive can remember a marriage forever without the citizen’s active decision system scanning every interaction from that marriage.

### “Strange moods” are a different mechanism

DF’s strange moods are episodic artifact-production behaviors. A dwarf claims a workshop, seeks particular materials, and pursues an artifact to the exclusion of ordinary activities. Success usually produces substantial skill advancement; failure can end in insanity. They occur in world generation as well as fortress play, although world-generation handling is more abstract. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Strange_mood)

They are effective because they couple a character, a constrained project, scarce materials, and a persistent object. **For TCE, borrow that project structure—not the supernatural trigger or guaranteed leap in expertise.**

An inventor, architect, or religious reformer could become committed to a difficult project. Whether it succeeds should depend on knowledge, resources, collaborators, institutions, and persistence.

### RimWorld: selective character differences with visible consequences

RimWorld uses a more tightly focused cast. Backstories, skills, traits, bodily capabilities, relationships, and needs make characters differently useful and differently vulnerable. Its simulation gains much of its dramatic power from consequences that change what a familiar person can do, rather than from an elaborate explicit model of literary motivation. [rimworldgame.com](https://rimworldgame.com/)

Its mood system is unusually legible. Community documentation describes a 0–100 mood value moving toward a target assembled from thoughts and other modifiers. Under ordinary thresholds, minor, major, and extreme break risks begin below 35, 20, and 5 respectively; traits can alter thresholds. These are risk bands, not guarantees that crossing a number instantly causes a particular break. Mood also changes gradually rather than immediately matching every new thought. [RimWorld Wiki](https://rimworldwiki.com/wiki/Mood)

This creates temporal structure. One unpleasant event may be tolerable; several overlapping problems can become dangerous; relief can arrive before a crisis. The observer can understand why.

Relationships are also asymmetric and event-sensitive. Community documentation describes directed opinions ranging from −100 to +100, modified by compatibility, family connections, and remembered interactions. A rescue, insult, or botched operation can affect future social behavior for different durations. “A likes B” does not require “B likes A.” [RimWorld Wiki](https://rimworldwiki.com/wiki/Social)

The Alpha 13 developer release is particularly revealing: it added an expanded family network, romantic relationships, social fights, animal bonds, and the ability for off-map characters to return as visitors, raiders, or recruits. **Recurring identity makes an encounter meaningful:** this is not just another attacker, but someone connected to an existing life. [Ludeon Studios](https://ludeon.com/blog/2016/04/alpha-13-released/)

### RimWorld’s storyteller is useful—but incompatible with TCE’s causal contract

RimWorld’s storyteller selects and schedules external incidents. Cassandra emphasizes an escalating dramatic pattern, Phoebe provides more breathing room, and Randy produces less predictable incident combinations. The resulting story emerges from authored pressure interacting with the colony’s state and the player’s decisions. [rimworldgame.com](https://rimworldgame.com/)

TCE should not copy that authority into its world simulation.

A famine should arise from harvests, reserves, distribution, purchasing power, or policy. A raid should require attackers with motives, information, organization, and resources. A succession crisis should require an actual disputed succession—not a hidden timer deciding that the observer has enjoyed too much peace.

RimWorld’s **Ideology** expansion offers another useful but bounded precedent: beliefs are composed from one to four major memes, more specific precepts, roles, rituals, and styles. This supports interacting authored cultural components, but player-configurable ideology is not equivalent to autonomous institutional evolution. [Steam Store](https://store.steampowered.com/app/1392840/RimWorld__Ideology/)

For TCE, adapt the modular representation while making adoption, enforcement, reinterpretation, and transmission consequences of agent behavior.

---

## 3. What worked—and what failed

### Success: consequences survive the original event

The strongest shared pattern is not randomness. It is **recurrence with changed meaning**.

DF’s cultural transmission example preserves an invention through later people. RimWorld’s returning off-map characters preserve relationships across separate encounters. Both demonstrate why an event matters more when the simulation can refer to it again through surviving people, objects, skills, obligations, or institutions. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2015.html)

For TCE, every major feature should answer: **What durable state does this leave behind, and which future decisions can read it?**

A revolt that changes only a history entry is weak emergence. A revolt that changes ownership, offices, loyalties, migration, and the location of a new assembly hall can shape the next century.

### Failure: more detail can create bureaucracy rather than stories

In his 2019 interview, Adams discusses an equipment-management approach involving quartermasters that became slow, fiddly, confusing, and bug-prone without sufficient narrative payoff. He also characterizes the player as the fortress’s coordinating will, distinct from dwarves pursuing personal activities. [Game Developer](https://www.gamedeveloper.com/design/q-a-dissecting-the-development-of-i-dwarf-fortress-i-with-creator-tarn-adams)

Two lessons follow.

First, **detail should justify itself through consequential choices**, not merely apparent realism.

Second, TCE cannot assume that removing the player leaves a self-organizing society behind. The player’s hidden labor—choosing construction, assigning priorities, organizing reserves, and resolving coordination failures—must be replaced by institutions and decision processes.

### Failure: emotional systems amplify repeated exposure

DF’s 2018 changes reduced the dominance of repeated similar traumatic experiences, including horror at strangers’ corpses, and introduced filtering intended to prevent nearly identical memories from occupying the whole emotional system. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2018.html)

For TCE, repeated perception should not automatically count as repeated independent trauma. Seeing the same casualty on ten successive updates is not ten unrelated bereavements.

Use event identity, category saturation, habituation where appropriate, and explicit distinctions between witnessing, remembering, and learning about an event.

### Failure: needs without achievable responses

DF 53.16, released August 5, 2026, disabled the “be with family” need until isolated residents had ways to visit, receive visits, or contact distant relatives. The same patch fixed tavern patrons ordering a drink from every keeper and drinking them all, sometimes fatally. It also addressed conflicting diplomatic assignments and dwarves becoming trapped in activities despite unmet social needs. [Bay 12 Games](https://www.bay12games.com/dwarves/)

These are excellent TCE acceptance-test cases:

A desire must have a feasible response, a recognized blocked state, or an adaptation mechanism. A goal must account for already outstanding requests. An agent must not independently satisfy the same need through multiple providers without considering the combined result.

### Failure: optimization changes semantics

RimWorld’s June 8, 2026 update fixed several low-tick-rate errors: blood-rage progression, cube withdrawal, and mech repair behaved incorrectly under reduced update frequency. It also fixed an infinite appearance-changing desire caused by ideology–gene conflicts and performance problems involving many unbuildable blueprints. [Ludeon Studios](https://ludeon.com/blog/2026/06/update-1-6-4850-released/)

These are not just implementation curiosities. They expose three general failure classes:

**Elapsed-time errors**, where “per update” is mistaken for “per simulated second”; **unsatisfiable constraint loops**, where no valid state can satisfy all preferences; and **repeated failed work**, where the same impossible task is reconsidered indefinitely.

TCE should treat those as first-class design hazards, especially when accelerating time.

---

## 4. Scale, performance, and “FPS death”

### Why population alone is a poor predictor

DF’s “FPS” commonly refers to simulation progress, separately from graphical frame rate. A world can remain visually responsive while simulated time advances painfully slowly. Community performance guidance associates slowdowns with active creatures, perception, jobs, navigation, items, environmental updates, and changing map connectivity. It is a useful inventory of suspects, but its version-migrated advice and reported percentages are not a portable benchmark for modern builds. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Maximizing_framerate)

The relevant workload is not merely “200 dwarves” or “50,000 citizens.” It includes:

**How many candidate interactions are considered, how often decisions repeat, how often paths fail, how frequently topology changes, and how much state receives unnecessary periodic work.**

A quiet settlement and a crowded evacuation can have the same population and radically different costs.

### Pathfinding is important, but not the only structural problem

Adams’s 2021 engineering interview describes A\* supported by walking-connectivity labels: if two locations belong to different reachable components, the engine can reject a route before performing an expensive search. Dynamic terrain changes can require connectivity recomputation. He also notes that walking-oriented structures do not straightforwardly solve flying movement, illustrating the compromises created by multiple locomotion modes. [Stack Overflow Blog](https://stackoverflow.blog/2021/12/31/700000-lines-of-code-20-years-and-one-developer-how-dwarf-fortress-is-built/)

For TCE, adopt the general pattern: cheap feasibility checks before expensive planning, cached failures, and mode-specific connectivity. Do not repeatedly discover that a demolished bridge still prevents the same commute.

But do not infer that routing is always the dominant cost. Local perception, job discovery, relationship processing, and global resource searches can become equally serious if implemented as repeated broad scans.

### Old threading descriptions are now misleading

DF 50.09’s June 2023 release introduced experimental multithreading alongside other optimization work. Describing every current DF subsystem as strictly single-threaded is therefore too broad. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Release_information/50.09)

More decisively, RimWorld’s 1.6 announcement in June 2025 states that pathfinding was made fully multithreaded and batched, with lighting also multithreaded. It also lists improvements to hauling, alerts, animal-pen calculations, and other recurring work. This demonstrates targeted subsystem parallelization, not proof that the entire simulation scales linearly with core count. [Ludeon Studios](https://ludeon.com/blog/2025/06/announcing-odyssey-and-update-1-6/)

### Numbers worth retaining—and their limits

| Figure | What it establishes | What it does **not** establish |
| --- | --- | --- |
| DF world-generation presets span **17×17 to 257×257 regional tiles**. | World geography is represented hierarchically at substantial regional scale. | That every local tile and inhabitant is continuously simulated in fortress detail. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/World_generation) |
| DF’s documented memory design uses **8 short-term plus 8 long-term emotional memories**. | A small active memory can support durable characterization. | A limit of 16 historical facts per person. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Memory_%28thought%29) |
| RimWorld’s usual break-risk thresholds are **35 / 20 / 5** on a 0–100 mood scale. | Explicit thresholds and inertia make accumulating pressure readable. | Psychologically validated human behavior or universal thresholds across traits. [RimWorld Wiki](https://rimworldwiki.com/wiki/Mood) |
| Ludeon reported startup falling from **18.57 seconds to 4.66 seconds** in its 1.6 comparison, despite an additional expansion. | A developer-reported improvement to startup. | A simulation-throughput benchmark or a supported colonist count. [Ludeon Studios](https://ludeon.com/blog/2025/06/announcing-odyssey-and-update-1-6/) |

The reviewed sources do **not** establish a hardware-independent population ceiling for either game. Configurable population limits, community comfort ranges, and save-specific slowdowns should not be converted into one.

### TCE’s arithmetic makes sparse processing essential

The following are workload calculations, not measured benchmarks.

At 50,000 citizens, there are:

\[
\frac{50{,}000 \times 49{,}999}{2}
=1{,}249{,}975{,}000
\]

possible unordered citizen pairs.

By contrast, an average of 20 stored directed social connections per citizen produces one million edges. That is a fundamentally different workload.

Likewise, updating every citizen at 20 Hz means one million agent visits per wall-clock second. Even an assumed five microseconds per visit consumes five CPU-seconds per second before routing, markets, buildings, history, or rendering.

The implication is not “remove individuality.” It is:

**Use sparse relationships, bounded candidate sets, event-triggered deliberation, and analytic or scheduled updates between meaningful changes.**

Fast-forward additionally requires reducing unnecessary simulated-time work, not just distributing the same work across more cores.

---

## 5. History browsing: making emergence visible

### Legends makes the world inspectable

DF’s Legends mode connects historical figures, civilizations, sites, artifacts, and other records through navigable entries and event histories. It exposes a world’s accumulated past rather than only the currently visible settlement. Its limitation for TCE’s observer experience is significant: native access is separate from an active fortress or adventure in that world, encouraging retirement or separate-copy workflows rather than seamless live investigation. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Legends)

External tools show what users want beyond chronological lists. **LegendsViewer-Next** provides interactive maps, family trees, and paginated exploration of exported history. These are useful precedents for navigating a large graph of people, places, and events without loading the whole thing into one overwhelming view. [GitHub](https://github.com/Kromtec/LegendsViewer-Next)

### RimWorld makes recent causality easier to follow

RimWorld’s History interface combines wealth, population, and average-mood graphs with incident markers. Its message history preserves recent letters and messages, with pinning for longer retention. This supports questions such as “What happened just before the colony’s mood collapsed?” rather than requiring the player to reconstruct chronology from memory. [RimWorld Wiki](https://rimworldwiki.com/wiki/Menus)

Alpha 13 also expanded character records, tale recording, and descriptions of art. That is another useful mechanism: events can reappear in a different medium, allowing the colony to refer back to itself. [Ludeon Studios](https://ludeon.com/blog/2016/04/alpha-13-released/)

TCE needs both approaches: **RimWorld’s immediate explanation and DF’s long historical reach.**

### Story discovery should be independent of event generation

Research on *story sifting* provides a direct architectural match.

**Felt** searches simulation histories for recognizable patterns involving related characters and events. A pattern can bind the same person across several events and impose temporal or relational conditions without requiring every intervening event to be narratively important. [Max Kreminski](https://mkremins.github.io/publications/Felt_SimpleStorySifter.pdf)

**Winnow** extends this direction with incremental processing of partially completed patterns. New events can start, advance, or invalidate a possible story. That supports following an emerging situation before its ending is known, rather than discovering everything retrospectively by rescanning the entire chronicle. [Max Kreminski](https://mkremins.github.io/publications/Winnow_AIIDE2021.pdf)

For TCE, use these ideas as **recognizers, not scripts**.

A recognizer might notice that a person helped found an institution, later lost authority within it, and now faces a successor they once mentored. It should not cause betrayal to complete the pattern. The situation may resolve peacefully, remain unresolved, or never become important.

### Recommended observer interface

I would build three connected views.

**The live observer** follows a limited cast of people, households, organizations, and projects. Cards should state what changed, why it matters, and what remains unresolved. Include ordinary achievement, cooperation, care, and institutional maintenance—not only violence and catastrophe.

**The “why?” inspector** connects a selected action to evidence: the actor’s goal, relevant beliefs, available options, constraints, and decisive state changes. The explanation should come from recorded decision inputs, not an invented motive added afterward.

**The historical atlas** combines maps, life timelines, genealogy, ownership chains, office succession, migration, legal changes, and building biographies. A granary should reveal who commissioned it, how it was financed, which policy created demand for it, and how its use changed.

Every view should distinguish:

**Recorded fact**, **a character’s belief**, and **the observer’s inferred interpretation**.

That distinction allows misinformation and competing historical accounts without making the underlying simulation inexplicable.

---

## 6. Concrete design recommendations for TCE

The remainder is a proposed TCE design, not a claim about either game’s implementation.

### A. Separate world state, personal memory, and history

Use three stores with different purposes.

| Store | Contents | Processing and retention |
| --- | --- | --- |
| **Authoritative state** | Living people, households, inventories, ownership, organizations, offices, current laws, buildings, commitments. | Optimized for simulation access; updated by the Rust kernel. |
| **Active personal memory** | A bounded set of emotionally or practically significant experiences, plus relationship summaries and beliefs. | Updated when relevant events occur; consulted selectively during decisions. |
| **Historical archive** | Significant events and the durable identities needed to interpret them. | Append-oriented, indexed, paginated, and loaded on demand. |

Do not use the archive as the agent’s working memory. Do not use the current-state database as the only record of the past.

A compact consequential-event record should contain an ID, simulation time, event type, participating identities, location, relevant organizations, state changes, and references to immediate causes. Record the rule or action version when useful for debugging.

Keep **world events** separate from **who witnessed or learned about them**. One theft is one event; several people can acquire different beliefs about it at different times.

### B. Record causality when state changes are committed

For a major action, preserve enough evidence to answer:

“What enabled this, what triggered it, who chose it, and what changed?”

This does not require storing a full planner trace for every meal. Use tiers:

Routine actions update counters or short-lived diagnostics. Significant personal changes create persistent events. Institutional, demographic, ownership, and major construction changes receive richer provenance.

A useful invariant is that the observer can never claim a causal link that the simulation did not record or explicitly mark as an inference.

### C. Replace the absent player with institutions

This is likely the largest design gap between TCE and its references.

Households should coordinate consumption, care, inheritance, and labor. Enterprises or cooperatives should coordinate production and investment. Governments should decide and enforce rules within explicit jurisdictions. Builders should require a client, authorization or tolerated occupation, resources, labor, and a feasible site.

Do not make all 50,000 people independently solve city planning.

Most citizens need cheap decisions inside commitments: go to an established workplace, fulfill an obligation, buy from known options, care for a dependent. Institutions and a smaller set of deliberating actors handle changes to those arrangements.

This preserves agency while avoiding an invisible central planner disguised as “the economy.”

### D. Give authored primitives operational consequences

A law should not merely modify a culture label. It should change permissions, obligations, costs, sanctions, or adjudication.

A technology should not merely set a civilization-wide unlock. It should have practitioners, required knowledge, materials, equipment, and transmission mechanisms.

An architectural style should not merely follow an era number. Its adoption should depend on available techniques, resources, patronage, imitation, institutions, and the existing built environment.

Authored primitives are compatible with unscripted history when they define **possible mechanisms**, not predetermined sequences.

### E. Bound costs without breaking continuity

For the Rust kernel, prioritize data-oriented hot state, sparse social edges, local candidate discovery, dirty-state invalidation, and batched independent work.

Needs that change smoothly can advance from elapsed simulation time. Decisions should run on schedules or relevant events. Failed plans should retain a reason and an invalidation condition rather than retrying continuously.

For Unreal, keep rendering separate from authority. A citizen leaving the camera should not lose commitments, possessions, relationships, or consequential behavior. Visual detail can decrease; causal fidelity must follow an explicit simulation policy rather than camera position.

The observer should consume snapshots and committed events. It should not independently reconstruct or modify authoritative citizen state.

### F. Establish a century-scale retention contract early

At a constant population of 50,000, retaining just one event per person per day for 100 years produces:

\[
50{,}000 \times 365 \times 100
=1.825\text{ billion events}
\]

At an assumed 100 bytes per event, that is **182.5 GB before indexes and other overhead**. Ten daily events per person would be **1.825 TB**.

Therefore, “record everything forever” is not a neutral implementation choice.

Retain births, deaths, marriages, migrations, ownership changes, offices, laws, major projects, inventions, conflicts, and other consequential changes. Aggregate routine production and consumption. Preserve richer short-term detail in a rolling window. Allow the user to pin people, institutions, places, or episodes for deeper retention.

The retention rule must be explicit: a summarized past should never silently masquerade as a complete event log.

### G. Test plausibility alongside throughput

Benchmark 10,000, 25,000, and 50,000 citizens under different interaction patterns—not merely different population counts.

The important scenarios include concentrated markets, blocked routes, harvest failure, migration, mass bereavement, political succession, and very old worlds with large archives.

Measure simulation throughput and tail latency alongside route retries, candidate-job scans, local interaction checks, archive growth, and failed-goal duration.

Then add semantic tests:

Does a slower update schedule change recovery or consumption rates? Can an agent satisfy a need through available actions? Can a person remain trapped between contradictory norms forever? Does switching rendering detail change outcomes? Can every displayed historical claim be traced to supporting records?

Those tests directly target the failure classes exposed by both games’ development histories.

### A representative TCE story—without scripting it

Consider a hypothetical sequence:

A poor harvest reduces actual stocks. Prices rise. Households with different reserves and obligations experience different pressure. A grain merchant refuses further credit to a household already in debt. An existing council debates a storage proposal. A coalition secures labor and materials for a communal granary. Its rules favor certain contributors. Years later, those rules are challenged by descendants who inherited neither the original obligations nor the original benefits.

No “famine reform” plot needs to exist.

The story emerges because production, credit, kinship, institutions, construction, and inheritance share persistent state. The observer discovers the sequence and links it to the people, decisions, and surviving building.

That is the combination worth pursuing: **a world that remembers enough to remain coherent, and an interface that reveals why its changes matter.**

---

## 7. Sources worth studying directly

### Developer talks and engineering accounts

| Source | Best use |
| --- | --- |
| [Tynan Sylvester, GDC 2017: “RimWorld: Contrarian, Ridiculous, and Impossible Game Design Methods”](https://gdcvault.com/play/1024232/-RimWorld-Contrarian-Ridiculous-and?utm_source=chatgpt.com) | RimWorld’s story-generator design philosophy. |
| [Tarn Adams, Game Developer interview, 2019](https://www.gamedeveloper.com/design/q-a-dissecting-the-development-of-i-dwarf-fortress-i-with-creator-tarn-adams?utm_source=chatgpt.com) | World generation, player coordination, and the costs of excessive detail. |
| [Tarn Adams, Stack Overflow engineering interview, 2021](https://stackoverflow.blog/2021/12/31/700000-lines-of-code-20-years-and-one-developer-how-dwarf-fortress-is-built/?utm_source=chatgpt.com) | Connectivity, pathfinding, representation, and architectural trade-offs. Treat threading statements as historical. |

### Developer logs and release evidence

| Source | Best use |
| --- | --- |
| [DF 2015 development log](https://www.bay12games.com/dwarves/dev_2015.html?utm_source=chatgpt.com) and [2018 development log](https://www.bay12games.com/dwarves/dev_2018.html?utm_source=chatgpt.com) | Cultural transmission; memories, personality change, and emotional rebalancing. |
| [DF current development log](https://www.bay12games.com/dwarves/?utm_source=chatgpt.com) | Recent fixes involving needs, coordination, pathing, and continuity. |
| [RimWorld Alpha 13 release](https://ludeon.com/blog/2016/04/alpha-13-released/?utm_source=chatgpt.com) | Relationships, returning characters, records, and tales. |
| [RimWorld 1.6 announcement](https://ludeon.com/blog/2025/06/announcing-odyssey-and-update-1-6/?utm_source=chatgpt.com) and [1.6.4850 patch](https://ludeon.com/blog/2026/06/update-1-6-4850-released/?utm_source=chatgpt.com) | Modern parallelization and concrete failures of reduced-rate processing. |

### Mechanics, history tools, and research

| Source | Best use |
| --- | --- |
| DF Wiki: [historical figures](https://dwarffortresswiki.org/index.php/Historical_figure?utm_source=chatgpt.com), [positions](https://dwarffortresswiki.org/index.php/Position_token?utm_source=chatgpt.com), [variable positions](https://dwarffortresswiki.org/index.php/Variable_positions?utm_source=chatgpt.com) | Community documentation of population abstraction and institutions. |
| DF Wiki: [Legends](https://dwarffortresswiki.org/index.php/Legends?utm_source=chatgpt.com), [performance guidance](https://dwarffortresswiki.org/index.php/Maximizing_framerate?utm_source=chatgpt.com) | History navigation and a version-sensitive checklist of performance risks. |
| RimWorld Wiki: [mood](https://rimworldwiki.com/wiki/Mood?utm_source=chatgpt.com), [social relationships](https://rimworldwiki.com/wiki/Social?utm_source=chatgpt.com), [menus/history](https://rimworldwiki.com/wiki/Menus?utm_source=chatgpt.com) | Community-documented mechanics and presentation patterns. |
| [LegendsViewer-Next](https://github.com/Kromtec/LegendsViewer-Next?utm_source=chatgpt.com) | Maps, genealogy, and large-history exploration. |
| [Kreminski, Dickinson, and Wardrip-Fruin: *Felt: A Simple Story Sifter*](https://mkremins.github.io/publications/Felt_SimpleStorySifter.pdf?utm_source=chatgpt.com) | Recognizing stories in simulation histories. |
| [Kreminski, Dickinson, and Mateas: *Winnow: A Domain-Specific Language for Incremental Story Sifting*](https://mkremins.github.io/publications/Winnow_AIIDE2021.pdf?utm_source=chatgpt.com) | Discovering and following partial stories without controlling their outcomes. |

**Bottom line:** TCE does not need every citizen to be a novelist or every subsystem to be maximally detailed. It needs persistent identities, consequential institutions, bounded minds, reliable causal records, and an observer that makes those connections visible without inventing them.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927cf-882c-83ea-9738-889e6a87300d)
