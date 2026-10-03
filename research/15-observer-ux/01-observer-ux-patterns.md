# Observer UX for The Civilization Engine

## Executive recommendation

**Build TCE as an observatory with a connected history browser—not as a management game with its controls removed.**

The core interaction should be:

**Notice something → inspect it → understand its circumstances → follow the people involved → return later to see what changed.**

For TCE, I would prioritize five connected capabilities: a readable world view, causal inspectors, persistent watchlists, a searchable chronicle, and time controls that preserve context. Overlays and dashboards should support this loop rather than become a separate spreadsheet application.

The strongest precedents contribute different pieces. Cities: Skylines connects spatial patterns to individual citizens; Dwarf Fortress makes generated history explorable; RimWorld gives mechanical events personal significance; Victoria 3 exposes population-level explanations and demonstrates how misleading presentation can undermine them; Crusader Kings 3 provides contextual learning through nested tooltips; Songs of Syx shows the importance of population-group inspection at large scales. These are complementary patterns, not a controlled ranking of which game has the “best UX.” [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/citizen-simulation-lifepath)

**The critical architectural decision is to record explanations in the simulation kernel.** A beautiful inspector cannot reliably explain a migration, failed construction project, or political reversal when the renderer receives only the resulting state.

The recommendations below distinguish documented implementations and developer reports from proposed TCE behavior. Numerical TCE budgets are starting hypotheses, not measured performance.

---

## 1. What makes observation rewarding?

### Legibility requires relationships, not merely visible numbers

A useful foundation is Shneiderman’s information-visualization framework: overview, zoom, filter, details on demand, relationships, interaction history, and extraction. The frequently repeated “overview first” formulation is only part of it. Preserving an investigation’s navigation history and letting users connect related objects matter just as much. Here, *interaction history* means retracing searches and selections—not the simulation’s historical timeline. [UMD Computer Science](https://www.cs.umd.edu/~ben/papers/Shneiderman1996eyes.pdf)

For TCE, that translates into five observer questions:

| Observer question | Interface responsibility |
| --- | --- |
| What is happening here? | Show activities, spatial patterns, and current conditions. |
| Why is this happening? | Expose recorded decisions, constraints, and relevant changes. |
| Who does it affect? | Connect population statistics to groups, households, and people. |
| What changed since I last looked? | Provide interval comparisons and a persistent chronicle. |
| What should I watch next? | Offer relevant subjects without taking control of the simulation or camera. |

The enjoyable loop should alternate between **discovery, explanation, attachment, and return**. An observer might notice a busy crossing, discover that merchants are avoiding another route, follow a merchant household, and later recognize that household’s influence in a new settlement.

This is a design proposal, not a claim that every interesting event needs a complete explanation. TCE should preserve room for curiosity. The important distinction is between **a question that remains open because the world is complicated** and **a question the interface makes impossible to investigate**.

### Explanations should answer “why this?” and “why not that?”

A CHI 2009 study by Lim, Dey, and Avrahami recruited **211 online participants** to examine explanations of relatively simple intelligent systems. “Why” explanations improved understanding and trust; “why not” explanations also helped, but were more difficult to understand in some measures. This supports offering both explanation types, not assuming that a longer explanation is automatically better. The study was not a game-retention experiment. [CMU School of Computer Science](https://www.cs.cmu.edu/~byl/publications/lim_chi09.pdf)

For TCE, the corresponding questions are concrete:

“Why did this household leave?”  
“Why did the neighboring household stay?”  
“Why has this workshop stopped?”  
“Why has the settlement not adopted this technique?”

A generic encyclopedia entry about migration or technology diffusion cannot substitute for an explanation of the actual case.

---

## 2. What the six games demonstrate

### Cities: Skylines — connect the map to an individual life

**Mechanics.** The series’ info views organize city conditions spatially. The sequel’s citizen documentation shows a useful connection between scales: a happiness overlay summarizes conditions geographically, while citizen and household inspection exposes contributing factors. Its Follow feature pins citizens to a list and starts a Lifepath Journal containing events such as graduation, employment changes, marriage, and relocation. Importantly, the documented journal starts **when the citizen is followed**, not at birth. [Skylines Paradox Wiki](https://skylines.paradoxwikis.com/Info_views)

**What works.** The pattern gives an aggregate a human referent: a colored neighborhood can become a household with an address, occupation, and changing circumstances. This is a stronger observer interaction than presenting either a heatmap or a biography alone.

**Limitations and failures.** The journal’s starting point creates a retrospective blind spot: discovering an interesting older citizen does not necessarily expose their earlier life. Separately, a 2024 community discussion around Economy 2.0 included complaints that the economic simulation was insufficiently transparent. That is qualitative evidence of a pain point, not a representative measurement of player opinion. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/citizen-simulation-lifepath)

**For TCE:** adopt spatial-to-person navigation, but preserve a compact lifetime history for everyone. Watching someone should increase notification priority and optional detail retention—not determine whether they have a past.

Also separate **Focus**, **Follow camera**, and **Subscribe to events**. A player may want updates about a family without having the camera track its members.

### Dwarf Fortress Legends — make generated history navigable

**Mechanics.** Legends presents categories of historical objects, individual entries, event histories, and links between figures, places, civilizations, and artifacts. Highlighted references open related entries in tabs. However, the documented native workflow does not allow entering Legends while a fortress or adventurer remains active in that world; retirement or a separate timeline/copy is needed. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Legends)

**What improved.** Bay 12’s January 2022 development log described replacing older navigation with browser-like links and tabs. It also described a curated world-generation readout: approximately **one important event per second**, selected from the most recent **1,000 events**. Splitting world-generation routines improved input responsiveness without making the underlying simulation faster. Those are two distinct successes: selecting what to show and keeping controls responsive while computation continues. [Bay 12 Games](https://www.bay12games.com/dwarves/)

**Community response.** LegendsViewer-Next adds paginated object tables, maps, family trees, and saved-world access. Its workflow requires exported XML; fuller maps and information require an additional DFHack export. This demonstrates a useful exploration pattern, but also the friction of making historical analysis an external operation. [GitHub](https://github.com/Kromtec/LegendsViewer-Next)

**For TCE:** integrate a Legends-like browser directly into live observation. Every person, institution, place, and important event should have a stable address. A historical reference must remain usable after its subject dies, dissolves, changes name, or disappears from the rendered world.

### RimWorld — make mechanical consequences personally meaningful

**Mechanics.** RimWorld connects characters’ backgrounds, relationships, circumstances, moods, injuries, and capabilities. Hunger, fatigue, surroundings, bereavement, and other experiences have mechanical consequences. This creates understandable connections between events and individuals rather than treating people solely as interchangeable economic units. [RimWorld](https://rimworldgame.com/)

Tynan Sylvester’s [GDC 2017 talk, *RimWorld: Contrarian, Ridiculous, and Impossible Game Design Methods*](https://www.gdcvault.com/play/1024232/-RimWorld-Contrarian-Ridiculous-and?utm_source=chatgpt.com), frames the project as a story generator and discusses choosing—and deliberately omitting—features around that purpose. [GDC Vault](https://www.gdcvault.com/play/1024232/-RimWorld-Contrarian-Ridiculous-and)

**Concrete UX improvements.** The 1.6 announcement documents search expanding into colonists’ equipment, containers, and the world map. A “Jump to…” control lists colonies, caravans, and quest locations. These changes reduce the need to remember where something is before inspecting it. [Ludeon Studios](https://ludeon.com/blog/page/2/)

**The important mismatch.** RimWorld’s storyteller deliberately controls incident selection and pacing. Its official description explicitly characterizes raids, thunderstorms, and visitors as events dealt into the story. TCE’s unscripted premise should not inherit that mechanism unchanged. [RimWorld](https://rimworldgame.com/)

**For TCE:** borrow personal continuity, intelligible consequences, and search. Add an **attention director that selects existing events**, not a story director that causes convenient disasters. A city-wide portrait bar is also the wrong scaling strategy: a small watched cast should provide personal continuity within a much larger population.

### Victoria 3 — presentation can accidentally teach the wrong model

**Mechanics and correction.** Dev Diary 61 describes removing red/green treatment from market supply-demand balances because it encouraged players to make everything green, even when that was not an economically meaningful goal. Reserve displays gained historical direction indicators so players did not have to watch a static bar to infer whether it was growing. These are unusually explicit examples of visual presentation producing the wrong interpretation. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/victoria-3-dev-diary-61-data-visualization)

**Further improvements.** Dev Diary 74 documents notification settings, more selective relevance rules, and benchmarks reporting roughly **50% fewer notifications for most countries**. It also describes surfacing the **five most important purchased goods** for population groups, improving population income/spending information, and fixing nested-tooltip placement. Community mods—including Visible Pop Needs and Practical Heatmaps—informed changes. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-74-ux-improvements)

**What worked:** expose the consequential part of a calculation close to the subject being inspected, rather than requiring repeated trips to another screen.

**What failed:** visual conventions implied objectives the simulation did not actually have; broad notifications competed with important information; traversing explanations could itself be awkward.

**For TCE:** distinguish *direction*, *magnitude*, and *desirability*. Population growth, food prices, elite wealth, centralization, and migration should not automatically become green or red. Show who benefits, who loses, and relative to which baseline.

### Crusader Kings 3 — keep conceptual learning close to the question

**Mechanics.** CK3’s nested tooltips allow an explanation to contain further explorable concepts. The original developer diary describes timer-based and explicit-action locking, an encyclopedia, contextual advice, and a separation between actionable alerts, issues, and informational notifications. It also reports playtesting with players of different experience levels. The diary text is available in a community compilation. [Scribd](https://www.scribd.com/document/500476375/Ck3-First-Version-FINAL-1)

**What works.** A player can investigate an unfamiliar concept without abandoning the current object. In his firsthand UX analysis, Philip Ardeljan praises the way this matches a chain of questions, while identifying pointer-boundary failures, accessibility, and excessive nesting as risks. His suggested depth of two or three is a design suggestion, not an experimentally established limit. [Philip Design](https://philip.design/blog/tooltips-in-tooltips/)

**For TCE:** use short contextual explanations, explicit pinning, keyboard access, and a clear “Open full explanation” action. Promote a deep investigation into a stable inspector instead of accumulating an indefinitely deep stack of hover panels.

Adapt the notification taxonomy, but not its ruler-centric assumption. In TCE, an event may deserve attention because it is historically or personally interesting—even when the observer cannot “fix” it.

### Songs of Syx — use population groups without losing individuals

**Mechanics.** Songs of Syx advertises tens of thousands of individually simulated citizens and slaves. Its documented fulfillment interface groups information into seven domains and supports inspecting the whole population or a species. Population views expose demographic changes; access and service views distinguish needs such as food, housing, and amenities. The older wiki is useful as documentation of this interface pattern, not as a guarantee of current balance values. [Steam Store](https://store.steampowered.com/app/1162750/Songs_of_Syx/)

**Concrete iteration.** The V53 developer update made the farthest zoom level usable for normal interactions, added named hotspots on the minimap, and enlarged/reworked the UI. These changes specifically improve orientation across settlement scales. [itch.io](https://songsofsyx.itch.io/songs-of-syx/devlog/170956/refinement-mod-support)

**Remaining friction.** A 2025 community discussion includes players describing initial confusion and relying on a getting-started video before the interface became understandable. This is anecdotal evidence of an onboarding problem, not a measured failure rate. [Quarter To Three Forums](https://forum.quartertothree.com/t/songs-of-syx-i-m-sort-of-fascinated-with-the-game-fantasy-city-builder/163488)

**For TCE:** adopt group-level inspection and named places. Adapt grouping to occupation, household resources, settlement, affiliation, migration history, or other actually simulated attributes. Do not transplant fantasy-species preference tables into fixed rules about human cultural groups.

---

## 3. Recommended TCE interface

### 3.1 A quiet world view, with overlays organized around questions

The default view should make ordinary activity worth watching: workers arrive, markets fill, construction progresses, roads become established, and neighborhoods change. Do not turn every person into an icon or make permanent warning badges the dominant visual feature.

Organize overlays by observer questions rather than internal subsystem names:

| Lens | Primary question | Useful representations |
| --- | --- | --- |
| Livelihoods | Who is struggling, and where? | Household purchasing power, unmet needs, employment, access to essentials. |
| Movement and exchange | What connects these places? | Selected routes, actual journeys, trade volumes, delays, migration flows. |
| Society and institutions | Who belongs to what, and who has influence? | Affiliations, jurisdictions, ownership, support distributions. |
| Change | What is different from the selected past date? | New buildings, abandonment, population change, institutional and technological changes. |

Start with a small number of excellent lenses. Each should have a visible title, scope, units, legend, time basis, and population denominator.

A food lens must distinguish **food exists nearby**, **food can be reached**, **the household is permitted to obtain it**, and **the household can afford it**. Combining those into one green “coverage” radius would hide the interesting simulation.

Show one principal filled overlay at a time, with an optional supporting layer such as routes or jurisdiction boundaries. Keep selections recognizable when the overlay changes. Offer fixed comparison scales; silently renormalizing every map view can make a worsening world look unchanged.

Use labels, symbols, and patterns alongside color. “Unknown,” “not applicable,” and “zero” need visibly different treatments.

### 3.2 One inspector structure across people, buildings, and institutions

Use a persistent side inspector rather than unrelated modal windows. Its first screen should answer:

**What is it? What is it doing? What changed? What explains the current situation?**

For a person, show identity and context, current activity and destination, household/workplace links, important conditions, and recent meaningful events. For a workshop, show current production, inputs, workers, bottlenecks, ownership, and recent changes. For an institution, show membership, jurisdiction, resources, decisions, and relevant relationships.

Keep the initial explanation short. Expand from a sentence into contributors, then into detailed records. For example:

> **The workshop is idle.** Its next production step requires timber. The last delivery did not arrive.

Selecting “timber” opens stocks and suppliers. Selecting “delivery” opens the journey record. Selecting the supplier reveals whether the problem was production, affordability, route access, or a different constraint.

The same object should be reachable from the world, search results, a chart, or a historical event. Back/forward navigation should restore the previous selection, filters, and scroll position.

Do not hide a relevant concept merely because it is unavailable. Show “No court exists here” or “This technique is known locally but not in use,” with explanations. Conversely, do not permanently display maritime logistics in a landlocked village.

### 3.3 An explanation contract between Rust and the UI

For important decisions, the kernel should expose a compact, structured record:

| Field | Why the observer needs it |
| --- | --- |
| Decision time and actor | Prevents explaining an old action using the actor’s current state. |
| Goal and chosen action | Distinguishes intention from observed movement. |
| Relevant perceived information | Shows what the actor knew or believed at the time. |
| Considered alternatives and blockers | Supports “why not?” without inventing alternatives. |
| Contributing rule inputs | Makes the explanation inspectable. |
| Related events and outcomes | Connects decisions to subsequent consequences. |

Not every action requires a full retained decision tree. Prioritize decisions that cross meaningful boundaries: relocating, changing occupation, adopting a technique, joining an institution, investing, abandoning a project, or imposing a policy.

Use three explicit explanation categories:

**Recorded reason:** the decision system used these inputs.  
**Observed relationship:** these quantities changed together.  
**Estimated alternative:** a separate calculation suggests another outcome under stated assumptions.

Do not present all three as equally certain causation. An actor’s recorded justification is also not automatically an exhaustive explanation of the wider social outcome.

A useful research precedent is PolicyExplainer, which combines agent-state, trajectory, and question-driven explanations. Its small user study favored visual explanations over a verbose text baseline, but the participants and tasks were specialized. The relevant lesson is to connect explanations to visible state and behavior, not to turn every inspector into a paragraph generator. [arXiv](https://arxiv.org/html/2104.02818v2)

### 3.4 Follow cameras should preserve attachment and orientation

Support several observation targets: **person, household, workplace, institution, and place**. A market square or workshop can remain interesting across generations even when individual residents change.

For a person-follow camera, retain a stable viewing direction where practical, smooth changes in movement, handle occlusion, and avoid repeatedly snapping around corners. Provide a small location/context indicator. A close-follow mode should not make the player lose their understanding of the surrounding settlement.

Offer three distinct actions:

**Focus** moves the camera once. **Follow** keeps tracking. **Watch** subscribes to meaningful changes.

Following should not make agents more successful, more active, or more historically important. Any higher rendering detail must remain separate from simulation fidelity.

When someone dies or leaves the represented area, preserve the inspector and history. Offer continuation through a household, workplace, descendant, or destination—but do not silently choose a replacement protagonist.

An optional guided tour can suggest existing scenes. It should disclose why a scene was selected and remain easy to interrupt. Avoid ranking solely by casualties, population size, or political power; otherwise quiet lives and small settlements will disappear from the observer experience.

### 3.5 A chronicle, not an unfiltered event log

Provide a timeline at world, settlement, institution, household, and individual scope. At long ranges, show episodes and intervals: wars, shortages, migrations, construction programs, institutional lifetimes. At short ranges, expand into particular decisions and events.

Place selected metrics on a shared time axis. Clicking an event should highlight its participants and recorded consequences. Temporal proximity alone should not create a causal arrow.

Make **“What changed since I last looked?”** a first-class operation. It should compare the selected dates and report changes to watched subjects, local conditions, buildings, institutions, and important relationships.

Separate three capabilities honestly:

| Capability | What it actually promises |
| --- | --- |
| Historical inspection | Read retained past records and aggregates. |
| Historical reconstruction | Display a past state supported by saved snapshots/deltas. |
| Replay or rewind | Re-execute or reproduce the intervening simulation under a defined reproducibility contract. |

An archive of yearly population totals cannot support an accurate replay of a particular person’s walk. Mark reconstructed or unavailable detail clearly, and keep “Return to live” unmistakable.

Retain navigation into extinct settlements and dissolved institutions. History should not break because the current-world entity was deleted.

### 3.6 Comparison dashboards should explain differences, not rank civilizations

Let players pin a few settlements, groups, institutions, or historical dates and compare the same measures. Favor aligned charts and tables over many unrelated gauges.

Support both calendar-time comparison and explicitly labeled relative comparisons, such as years since settlement founding. Show totals alongside rates and relevant distributions. A rising mean income should not conceal a poorer lower-income group or a population-composition change.

For group comparisons, distinguish:

* **Fixed membership:** what happened to the people originally selected?
* **Changing membership:** what happened to everyone currently belonging to this group?

Those answer different questions.

Avoid a universal civilization score or fixed-era progress ladder. Compare particular outcomes—food security, mobility, institutional reach, knowledge retention—while preserving trade-offs and reversals.

---

## 4. Time controls and notifications are part of the observation model

### Time needs to match the scale of the question

TCE should support three temporal experiences:

**Daily life:** watch tasks, encounters, travel, and work.  
**Development:** observe seasons, projects, and demographic change.  
**History:** follow decades through summaries and retained milestones.

These need not be three separate simulation models. They are different presentation and pacing modes. At high acceleration, do not imply that every journey remains visually continuous and readable. Keep detailed daily-life viewing available at an appropriate speed.

Show both requested speed and achieved progress, preferably in meaningful units such as simulated days per real minute. A responsive camera is not evidence that the simulation is keeping up.

Add **Run until…** predicates: the next harvest, a project’s completion, a watched household’s relocation, an institutional change, or a chosen date. Evaluate predicates in the kernel rather than relying only on periodically refreshed UI values, so transient crossings are not missed. Every wait needs cancellation and an optional maximum horizon.

Allow inspection to freeze the displayed snapshot without necessarily stopping the world, but label the snapshot’s date. Never quietly mix values from different times in an explanatory breakdown.

### Notifications should allocate attention, not demand intervention

Use a hierarchy:

| Delivery | Appropriate content |
| --- | --- |
| Chronicle only | Routine events outside watched subjects. |
| Digest | Related developments grouped over an interval or around an episode. |
| Toast | A significant change to a watched subject or a selected category. |
| Pause/slowdown | Explicitly opted-in conditions where the observer wants to witness the transition. |

Group notifications by a common episode where supported. One disrupted route can produce delivery failures, price changes, workshop stoppages, and migration pressure; five disconnected warnings would conceal the connection.

Use relevance, persistence, novelty, and user interest—not just magnitude. Add cooldowns and threshold hysteresis so a fluctuating value does not repeatedly announce the same condition.

Victoria 3’s approximately 50% notification reduction is a useful precedent for changing *selection rules*, rather than merely shrinking toast widgets. It is not a universal target for TCE. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-74-ux-improvements)

Keep dismissal separate from deletion. Every toast should remain discoverable in the chronicle, including what it meant at the time. Ordinary watched milestones—a birth, completed apprenticeship, or first successful harvest—should be eligible alongside crises.

---

## 5. Numbers, performance, and evidence limits

### Quantitative precedents

| Source | Reported figure | What it supports—and what it does not |
| --- | --- | --- |
| Victoria 3, UX Diary 74 | Roughly **50% fewer notifications** for most countries in developer benchmarks | Relevance filtering can substantially reduce volume; no published equivalent improvement in comprehension. |
| Dwarf Fortress, January 2022 development log | About **1 highlighted event/second**, selected from the latest **1,000 events** | Curation can coexist with much larger event throughput; this is not a simulation-speed benchmark. |
| Songs of Syx official description | **Tens of thousands** of individually simulated citizens/slaves | Relevant scale precedent, not proof of equivalent UE5 performance or observer comprehension. |
| Lim, Dey, Avrahami, CHI 2009 | **211 participants** | Controlled evidence for explanations in simple intelligent systems, not century-scale simulation play. |
| PolicyExplainer study | **10** graduate participants; all preferred its visual explanations to the text baseline | Encouraging evidence for linked visual explanations, with a small, specialized sample. Baseline task abandonment prevents a clean speedup claim. |

Sources: developer diaries and official description; the two research papers. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-74-ux-improvements)

These sources do not establish a safe maximum number of simultaneous overlays, inspectors, or notifications for TCE. Those require testing in the actual implementation.

### The history-storage problem is substantial even before rich narration

Assume a steady population of 50,000, a 365-day year, and just one retained event per person per day:

\[
50{,}000 \times 365 \times 100 = 1.825\text{ billion events}
\]

At an illustrative **64 bytes per event**, that is **116.8 GB per simulated century**, before strings, indexes, snapshots, and attachments.

Therefore, distinguish:

**Recent detail:** a bounded rolling record of fine-grained activity.  
**Persistent milestones:** compact lifetime, institutional, and settlement histories.  
**Aggregates:** progressively coarser time series and distributions.  
**Archives:** older detail moved out of the active working set.

Watching can increase detail retention, but everyone needs a basic history before becoming interesting. Endless simulation also needs an explicit archive policy; even milestones cannot grow forever in active memory. Any discarded detail should appear as unavailable, not be replaced with confident invented narration.

### Proposed implementation budgets

Use Rust-side aggregates and queryable snapshots, not per-frame scans initiated by individual widgets. Send keyed changes to the presentation layer; virtualize long tables; cancel obsolete queries; and keep history queries off the simulation’s critical path.

Initial budgets worth testing:

| Operation | Proposed starting target |
| --- | --- |
| Cached selection or inspector feedback | ≤ **100 ms**, 95th percentile |
| Typical indexed aggregate query | ≤ **250 ms**, 95th percentile |
| Selected entity’s textual status refresh | **5–10 Hz** maximum where useful |
| City-level displayed metrics | **1–2 Hz** unless a specific interaction needs more |
| Deep archive query | Asynchronous, cancelable, with visible scope and progress |

These are wall-clock presentation targets, not kernel tick rates. Faster refresh is not always more legible: numbers and sorted rows should remain stable while the player is reading or interacting.

---

## 6. What to build first—and how to judge it

### Adopt, adapt, avoid

| Decision | TCE priority |
| --- | --- |
| **Adopt** linked inspectors, searchable entities, stable history links, map-to-person navigation | Foundational; implement before multiplying dashboards. |
| **Adapt** nested tooltips, population-group panels, watchlists, event curation | Preserve context, scale to many people, and remove assumptions that the player governs everything. |
| **Avoid** forced dramatic incidents, universal red/green judgments, city-wide portrait bars, unbounded notification feeds, unverifiable causal narration | These conflict with TCE’s premise or undermine trust. |

### Build one complete investigation before a broad interface

A strong first vertical slice is a livelihood-and-migration chain:

The observer notices a settlement changing, opens the livelihood lens, selects an affected household, inspects a recorded decision, follows its journey, and later finds the destination and consequences in the chronicle.

The UI should succeed whether the household leaves, stays, changes work, receives assistance, or never encounters a crisis. The test fixture may be controlled; the shipped history should not be scripted to guarantee a satisfying sequence.

Next add institutional continuity and comparisons. Guided tours and elaborate automated summaries should come after the underlying navigation and evidence are reliable.

### Test understanding and enjoyment separately

For usability tests, ask players to locate a bottleneck, explain a decision, find a contrasting case, recover a watched subject, and compare two dates. Score explanations against the kernel’s retained evidence—not against the tester’s preferred narrative.

For observer enjoyment, allow unstructured time and ask what subjects players chose, whether they returned to them, and what changed their expectations. Measure interruption burden, loss of orientation, and whether players can tell an accurate story afterward. A high click count may mean curiosity or confusion; a quiet session may mean absorption or boredom.

A particularly useful test is a delayed return: after a substantial simulated interval, can the player reconnect with a household, place, or institution and understand its transformation?

**The success criterion is not that TCE exposes everything. It is that a player can find something worth caring about, investigate it without losing context, and later recognize how the world changed.**

---

## Selected source links

| Area | Sources |
| --- | --- |
| Information visualization | Shneiderman, [*The Eyes Have It*](https://www.cs.umd.edu/~ben/papers/Shneiderman1996eyes.pdf?utm_source=chatgpt.com) — exploration tasks, linked views, and navigation history. |
| Explanation research | Lim, Dey, and Avrahami, [*Why and Why Not Explanations Improve the Intelligibility of Context-Aware Intelligent Systems*](https://www.cs.cmu.edu/~byl/publications/lim_chi09.pdf?utm_source=chatgpt.com); [*Why? Why not? When? Visual Explanations of Agent Behaviour in Reinforcement Learning*](https://arxiv.org/html/2104.02818v2?utm_source=chatgpt.com). |
| Cities: Skylines | [Cities: Skylines II: Citizen Simulation & Lifepath](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/citizen-simulation-lifepath?utm_source=chatgpt.com); [Cities: Skylines info-view wiki](https://skylines.paradoxwikis.com/Info_views?utm_source=chatgpt.com). |
| Dwarf Fortress | [Bay 12 development log](https://www.bay12games.com/dwarves/?utm_source=chatgpt.com), especially January 22, 2022; [Legends documentation](https://dwarffortresswiki.org/index.php/Legends?utm_source=chatgpt.com); [LegendsViewer-Next](https://github.com/Kromtec/LegendsViewer-Next?utm_source=chatgpt.com). |
| RimWorld | [Official design description](https://rimworldgame.com/?utm_source=chatgpt.com); [Sylvester’s GDC talk](https://www.gdcvault.com/play/1024232/-RimWorld-Contrarian-Ridiculous-and?utm_source=chatgpt.com); [Ludeon’s June 2025 update material](https://ludeon.com/blog/page/2/?utm_source=chatgpt.com). |
| Victoria 3 | [Dev Diary 61: Data Visualization](https://www.paradoxinteractive.com/games/victoria-3/news/victoria-3-dev-diary-61-data-visualization?utm_source=chatgpt.com); [Dev Diary 74: UX Improvements](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-74-ux-improvements?utm_source=chatgpt.com). |
| Crusader Kings 3 | [Dev Diary 16: Tutorials, Tooltips, and Encyclopedias](https://forum.paradoxplaza.com/forum/threads/ck3-dev-diary-16-tutorials-and-tooltips-and-encyclopedias-oh-my.1345581/); Ardeljan’s [*Tooltips in tooltips*](https://philip.design/blog/tooltips-in-tooltips/?utm_source=chatgpt.com). |
| Songs of Syx | [Official description](https://songsofsyx.com/?utm_source=chatgpt.com); [V53: Refinement & Mod Support](https://songsofsyx.itch.io/songs-of-syx/devlog/170956/refinement-mod-support?utm_source=chatgpt.com); [historical fulfillment-interface documentation](https://songsofsyx.com/wiki/index.php/Fulfillment?utm_source=chatgpt.com). |

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92992-b2f4-83ea-b9dd-1afc25db30bb)
