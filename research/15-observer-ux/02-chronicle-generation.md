# Automated chronicles and emergent narrative for TCE

## Recommendation

**Build the chronicle as a read-only system that selects and explains history—not as a storyteller that makes history happen.** Keep structured events authoritative, extract meaningful sequences from them, and generate prose only after deciding what the account should contain.

For TCE, I recommend this pipeline:

```
Simulation transitions
    → typed historical events and statistical summaries
    → searchable historical archive
    → event ranking and story-pattern detection
    → evidence-backed episode plans
    → deterministic text, optionally rewritten by an LLM
    → chronicle, biographies, timelines, maps, and object histories
```

The critical distinction is between **recording**, **selecting**, and **telling**. A complete log is not necessarily readable; individually important events do not necessarily form a coherent story; fluent prose does not necessarily describe what happened. Story-sifting research explicitly addresses the middle problem: finding narratively meaningful subsets of a larger simulation history. Felt and Winnow are particularly relevant precedents. [Max Kreminski](https://mkremins.github.io/publications/Felt_SimpleStorySifter.pdf)

The architecture below is a proposed TCE design. Published measurements and documented developer experiences are identified separately.

---

## 1. How it should work

### 1.1 Record historical meaning, not every simulation operation

Do not make the chronicle consume Rust debug logs or reconstruct events from rendered activity. Give simulation systems explicit historical-event emitters.

A movement update is usually telemetry. A household leaving its village permanently is history. One purchase is usually routine. A merchant establishing the first sustained trade connection between two settlements may be historically consequential.

Use three layers:

| Layer | Contents | Intended retention |
| --- | --- | --- |
| **Operational telemetry** | Movement, task execution, individual transactions, utility evaluations | Short-lived; expanded for debugging or selected entities |
| **Historical record** | Births, migrations, institutional changes, construction milestones, discoveries, conflicts, significant decisions | Durable, subject to an explicit archival policy |
| **Narrative views** | Annual summaries, biographies, recognized episodes, generated prose | Rebuildable or cached; never the sole evidence |

A useful conceptual event schema is:

```
HistoricalEvent
  id, world_branch, schema_version
  time_start, time_end, sequence_number
  event_type
  participants_with_roles
  location_id
  facts_before_and_after
  causes_and_contributing_events
  decision_reason_reference
  episode_or_process_id
  affected_population_and_resources
  evidence_kind
  retention_class
```

Several details matter disproportionately.

**Store stable identities and historical context.** “The mayor approved the mill” must resolve to whoever held that office then, not its current occupant. Preserve former names, titles, ownership, relationships, and relevant state changes.

**Distinguish relationships between events.** `caused_by`, `enabled_by`, `responded_to`, `part_of`, and `occurred_after` are not interchangeable. A chronicle should not transform temporal adjacency into causation.

**Preserve decision evidence separately from narrative interpretation.** “The council cited grain shortages” is different from “grain shortages caused the vote,” especially when lobbying, factional interests, and procedural rules also mattered.

**Use intervals where precision is unavailable.** A statistical daily simulation step should produce “during the spring planting period,” not an invented Tuesday afternoon. Fine and coarse simulation modes should emit the same *semantic* event types at the precision they actually support.

Finally, this archive is not automatically a replay system. Reconstructing a historical ownership map is much cheaper than reproducing every historical citizen’s movement. Full simulation rewind would need additional checkpoints and replay machinery.

### 1.2 Detect processes as well as incidents

TCE’s most important developments may have no single dramatic event: increasing specialization, depopulation, concentration of landownership, adoption of stone construction, or gradual technological loss.

Add a second class of historical producers that examine statistical series and sustained state changes.

For example:

```
Individual construction events
    → settlement building-material series
    → sustained increase in stone construction
    → derived historical development
```

A derived development should retain its scope, interval, comparison baseline, calculation, and underlying series or event references. It should also distinguish observation from explanation:

> “Stone replaced timber in most new houses over the following generation.”

That is a measurable trend.

> “Residents rejected timber after losing faith in traditional architecture.”

That requires evidence about beliefs and decisions.

Use persistence requirements and hysteresis so that a fluctuating metric does not repeatedly announce “decline,” “recovery,” and “decline” around one threshold. Where causes are available, connect the trend to them; otherwise present the trend without manufacturing a causal account.

For TCE’s era-free progression, these developments can also suggest descriptive chapter headings—“The Canal Expansion” or “The Years of Outmigration”—without imposing predetermined historical stages.

### 1.3 Score salience relative to a reader’s question

There is no universally correct importance score. A marriage may be central to a biography, consequential to a succession dispute, and irrelevant to a history of irrigation.

It is useful to separate **historical significance** from **narrative relevance**. In the Indexter research tradition, salience concerns how accessible a previous event is in the audience’s memory; related work connects events through time, space, protagonists, causation, and intention. That is not the same as ranking events by casualties or economic magnitude. [Narrative](https://narrative.csail.mit.edu/cmn12/abstracts.pdf?utm_source=chatgpt.com)

For TCE, retain a feature vector rather than only one scalar:

| Feature | Proposed interpretation |
| --- | --- |
| **Impact** | Magnitude and distribution of changes to people, assets, institutions, or capabilities |
| **Persistence** | Whether consequences lasted beyond the immediate incident |
| **Novelty** | Unusualness within the relevant settlement, society, and historical context |
| **Causal importance** | Whether other consequential developments depended on this event |
| **Reversal or contrast** | A changed relationship, failed ambition, recovery, or unexpected outcome |
| **Reader relevance** | Connection to selected entities, places, topics, or bookmarked histories |
| **Explanatory value** | Whether including the event makes another event understandable |

A simple first implementation can use a weighted sum for the current view:

\[
S(e\mid v)=w\_I I+w\_P P+w\_N N+w\_C C+w\_R R+w\_U U+w\_X X
\]

The weights are editorial tuning parameters, not scientific constants. Expose the contribution breakdown in developer tools.

Three safeguards are essential.

**Normalize impact at multiple scales.** Losing ten people is catastrophic for a tiny settlement but a different kind of event in a metropolis. Retain absolute magnitude and relative impact rather than choosing only one.

**Treat rarity cautiously.** Estimate novelty within meaningful categories, with smoothing and caps. Otherwise random combinations of names, locations, and minor accidents will dominate because their exact occurrence was unprecedented.

**Select a diverse set, not merely the highest scores.** After selecting one episode, reduce the marginal value of near-duplicates. A famine should not occupy every card because its deaths, migrations, riots, price spikes, and relief measures all scored highly.

Reserve representation for everyday institutions, cooperation, care, construction, and learning. Otherwise the chronicle’s scoring rules will create a misleading impression that civilization consists entirely of rulers and disasters.

### 1.4 Sift connected stories, not just noteworthy events

Felt represents story patterns as queries over events and their participants. Its public example recognizes a guest entering a town, receiving hospitality, and subsequently being harmed by the host, with constraints linking the participants and excluding an intervening departure. The relevant events need not be adjacent in the full log. [GitHub](https://github.com/mkremins/felt)

For TCE, author **recognition patterns**, not plot scripts. A pattern says “notice when these relationships occur,” never “make the missing event happen.”

Useful families include:

| Pattern family | Recognizable structure |
| --- | --- |
| **Threat and adaptation** | Disruption → attempted response → changed outcome |
| **Institutional response** | Recurring problem → proposal → political decision → implementation |
| **Rise and loss** | Accumulation of status/resources → reversal → consequences |
| **Knowledge lineage** | Learning → practice → innovation → transmission |
| **Relationship transformation** | Cooperation/conflict → consequential interaction → changed relationship |
| **Place biography** | Founding → expansion → repurposing, abandonment, or recovery |

A political pattern might be:

```
Repeated bridge failures in settlement S
    → proposal P addresses crossing reliability
    → P is adopted or rejected
    → implementation or continued disruption follows
```

Bind the actual settlement, proposal, actors, and affected crossing. Require recorded response links where the text will claim a response. A generic bridge failure followed by an unrelated tax law is not a match.

Winnow advances this approach through incremental matching. It also highlights a temporal correctness issue: a condition associated with an event should be evaluated when that event occurs, not against the character’s later state. [Max Kreminski](https://mkremins.github.io/publications/Winnow_AIIDE2021.pdf)

For the Rust implementation, I would compile patterns into staged matchers:

* Dispatch by relevant event type and participant keys.
* Advance only potentially matching partial episodes.
* Invalidate matches when an explicit exclusion occurs.
* Expire or archive unresolved matches according to the pattern’s temporal scope.

Keep bounded summaries for long-running threads, and perform deeper retrospective searches when someone opens a relevant history. A family’s multigenerational migration story should not require keeping every possible partial match active in RAM.

Do not require every episode to have a satisfying ending. “The dispute remained unresolved” is valid history. A recognized beginning is not a prediction that the rest of an authored pattern will occur.

### 1.5 Rank episodes, then recover their necessary context

Once a pattern produces an episode, score the *episode*, not just its constituent events.

For example, the construction of a small granary might initially receive little attention. Decades later, records may show that it supplied repeated relief efforts and became the foundation of a redistribution institution. The granary’s importance has changed retrospectively.

This produces an important retention rule:

> **Low current salience is not sufficient justification for deleting a historical fact.**

After selecting an episode, retrieve the supporting context necessary to understand it: relevant prior relationships, institutional arrangements, goals, resource constraints, and consequences.

Context retrieval should have its own budget. Do not fill the prose with every ancestor event, but do not discard essential context merely because it scored poorly as a headline.

Avoid identifying stories only through a single central person. TCE needs biographies of laws, buildings, technologies, firms, neighborhoods, and settlements as much as biographies of rulers.

### 1.6 Turn selected evidence into a narrative plan

Before writing sentences, create a structured intermediate representation:

```
EpisodePlan
  focal_entities
  scope_and_time_range
  orientation_facts
  principal_changes
  supported_connections
  outcome_or_current_status
  unresolved_questions
  source_event_ids
  allowed_claims
```

This is where the system decides what the reader needs to know and in what order.

A compact narrative structure is usually enough:

**Situation → consequential change → response → outcome.**

Not every entry needs all four parts. An annual entry may be one sentence; a settlement history may have several episodes.

Consider this invented TCE example:

```
E410: Flood destroys East Bridge.
E422: Grain deliveries delayed; E410 recorded as cause.
E463: Council adopts temporary relief law;
      delivery disruption recorded among proposal reasons.
E590: Replacement bridge opens.
E650: Relief law repealed; repeal rationale not retained.
```

A supported account is:

> “The spring flood destroyed East Bridge and interrupted grain deliveries. The council adopted temporary relief measures in response to the disruption. A replacement bridge opened later that year, and the relief law was subsequently repealed.”

An unsupported account is:

> “Once the bridge restored prosperity, the council gratefully abolished the now-unnecessary law.”

The second version invents prosperity, emotion, and a repeal rationale. Elegant prose does not make those additions harmless.

### 1.7 Use templates as the dependable baseline

Templates should operate on episode plans, not concatenate one sentence per event.

A practical generator needs more than synonym substitution:

**Aggregation.** Combine repeated incidents: “Three harvests failed in five years,” while retaining access to the individual records.

**Reference management.** Introduce “Mara, the settlement’s millwright,” then use a shorter reference when unambiguous. Distinguish people with identical names.

**Temporal organization.** Use absolute dates at entry points and relative expressions within a tightly scoped passage.

**Grammar and localization.** Resolve number, tense, pronouns, titles, articles, and locale-specific word order structurally. Do not build sentences through fragile string fragments.

**Controlled variation.** Choose variants deterministically from the episode identity so that reopening a page does not continually rewrite history.

**Appropriate omission.** Do not invent observations to fill empty years. A quiet year may deserve a statistical summary—or no entry.

Templates also make a valuable testing oracle: the system should be able to produce an accurate, usable chronicle without an LLM.

### 1.8 Make LLM narration optional and downstream

The most relevant cautionary study is Méndez and Gervás’s 2023 experiment using GPT‑3.5 for story sifting. On a romantic-relations simulation, the model often summarized broadly rather than selecting a focused thread, and introduced events absent from the log. Their evolutionary sifter selected more relevant subsets but produced weaker text; they proposed combining selection with language-model realization. This was a small study of an older model, not a verdict on current models. [Computational Creativity](https://computationalcreativity.net/iccc23/papers/ICCC-2023_paper_124.pdf)

For TCE, give the model an **evidence packet**, not the entire history:

```
Selected episode plan
+ permitted factual statements
+ historical names and roles
+ supported causal links
+ uncertainties and missing information
+ style and length constraints
```

Require structured output containing sentence-level evidence references. Then validate entity IDs, dates, quantities, relationships, and references against the packet.

However, **a valid event ID is not proof that the attached sentence is supported**. A sentence can cite a real flood while inventing fear or political intent. Use claim-level checking, sampled human review, and conservative fallback to the deterministic version.

The narrator should have no simulation write access. An LLM must not invent a motive that later becomes a citizen’s actual motive.

Operationally, generate longer accounts on demand or asynchronously; cache them with their source-plan, language, style, and generator versions. Cancellation, unavailable services, and generation failure should leave the ordinary chronicle fully functional. Local or remote generation should be an explicit product choice, with remote transmission of world text opt-in.

Keep **canonical history** separate from **in-world accounts**. A priest’s partisan chronicle or a merchant’s self-serving memoir can be excellent simulation content—but must be labeled as an attributed account, not silently substituted for the historical record.

### 1.9 Browse history through several connected views

I recommend five views over the same archive:

| View | Reader’s question | Main interaction |
| --- | --- | --- |
| **Chronicle digest** | “What changed while I was away?” | Grouped episodes, significance explanation, adjustable scope |
| **Entity history** | “How did this person/place/institution become this?” | Biography, milestones, relationships, ownership and role changes |
| **Timeline** | “What happened before, during, and after this?” | Zoomable intervals and topic tracks |
| **Historical map** | “Where did change spread?” | Date slider, historical boundaries/sites/routes, linked episodes |
| **Evidence inspector** | “What supports this account?” | Source events, decision reasons, aggregates, uncertainty |

Dwarf Fortress community tooling offers a concrete reference: LegendsViewer‑Next provides paginated world-object tables, historical details, maps, and family trees. These are useful complements to prose rather than substitutes for it. [GitHub](https://github.com/Kromtec/LegendsViewer-Next)

For TCE, make every important noun navigable. A paragraph about a law should link to its proposer, institution, affected settlement, enactment, amendments, and repeal.

Preserve the browsing state: current year range, filters, selected entities, map position, and back navigation. Readers should be able to follow a person into an institution’s history and return without losing their place.

Use progressive disclosure: headline → short episode → detailed account → evidence. Keep a visible distinction between “not selected for this digest,” “not recorded,” and “no longer retained.”

History should also remain accessible through the world itself. Inspecting a bridge, house, market, or monument can show a short account of its construction, ownership, repairs, and significance. That gives the observer reasons to revisit ordinary places.

---

## 2. What worked and what failed in precedents

### Dwarf Fortress: a connected historical world, with substantial browsing costs

Legends organizes history around figures, sites, artifacts, civilizations, and other entities, with links between records. It also distinguishes available knowledge through its reveal settings. These are strong precedents for an archive that supports exploration rather than one fixed retelling. Its documented workflow has limitations: accessing Legends alongside an active fortress can involve copying and retiring a save, and XML exports do not expose every detail. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Legends)

Community viewers address navigation and presentation. The older Legends Browser explicitly added linked objects, statistics, and overviews; its maintainer also documented memory problems with large exports. [GitHub](https://github.com/robertjanetzko/LegendsBrowser)

**TCE lesson:** adopt the connected archive, but make it live, searchable, and integrated from the beginning. Do not require an export/import excursion to discover why the settlement currently looks as it does. Treat an export format as a supported public interface, not an incidental debugging dump.

### Crusader Kings II and III: annual records versus personal context

Paradox advertised CKII’s Charlemagne chronicle as an annual account inspired by the Saxon chronicle. This is a useful model for readable year-by-year presentation, but its dynasty-centered framing is narrower than TCE’s required world history. [Paradox Interactive](https://www.paradoxinteractive.com/games/crusader-kings-ii/add-ons/crusader-kings-ii-charlemagne)

CKIII’s 1.7 developer diary is more directly instructive. The developers identified a missing ability to track exactly what a character had experienced, when, and with whom. Memories added that context; relationship reasons explained how friendships and enmities formed. Memories could also inform later event content and be copied to the clipboard. Crucially, some fade, with longer retention for player-related characters. [Steam Store](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1662033633&feed=steam_community_announcements)

**TCE lesson:** adopt the “what happened, with whom, and why this relationship exists” presentation. Do not equate a character’s fallible memory with the observer’s historical archive. Their retention policies serve different purposes.

### RimWorld: history becomes tangible through artifacts

RimWorld’s Alpha 9 developer release notes describe sculptures depicting earlier colony events—including recruitment, surgery, killings, discoveries, drunkenness, and vomiting—and engraved art on high-quality weapons. This demonstrates that historical callbacks need not be grand or politically important to be memorable. [Ludeon Studios](https://ludeon.com/blog/2015/02/rimworld-alpha-9-tales-o-drunkness-released/)

Tynan Sylvester’s GDC 2017 talk frames RimWorld as a story generator and discusses how deliberate omissions leave room for players to supply meaning. That supports restraint in narration: explaining every emotional beat can remove useful interpretive space. [GDC Vault](https://www.gdcvault.com/play/1024232/-RimWorld-Contrarian-Ridiculous-and)

There is also concrete community evidence of an unmet archival need. RimStory’s author described wanting to preserve memorable colony experiences, then implemented event records and commemorations. Its documentation warned that events predating installation were not recorded; comments requested export functionality. These are specific user and developer observations, not a representative satisfaction survey. [Steam Community](https://steamcommunity.com/sharedfiles/filedetails/?id=1713190031)

**TCE lesson:** adopt historical reminders attached to objects and places, and record from the start. Adapt commemorations through explicit citizen or institutional mechanics—not by letting the observer’s chronicle create celebrations.

### Felt: expressive querying worked better than anticipating every author need

Felt’s developers initially tried to provide a library of convenient database-access functions. They found it impractical to anticipate every question an author might ask, and moved toward a real query language. The paper also describes `whyNot` debugging support for identifying failed query conditions. [Max Kreminski](https://mkremins.github.io/publications/Felt_SimpleStorySifter.pdf)

**TCE lesson:** give narrative authors a small typed pattern language with role binding, temporal constraints, and access to relevant historical state. Accompany it with positive examples, negative examples, and “why did this match/not match?” tooling. A giant collection of special-purpose Rust functions will become difficult to extend and audit.

### Winnow: incremental recognition does not eliminate scaling problems

Winnow’s reference implementation leaves partial matches in its pool unless they complete or are invalidated, so the authors explicitly discuss application-specific management of potentially unbounded growth. Its measurements also show increasing per-event cost as the pool grows. [Max Kreminski](https://mkremins.github.io/publications/Winnow_AIIDE2021.pdf)

**TCE lesson:** adopt the incremental pattern semantics, not the assumption that the reference implementation’s performance transfers to a 50,000-person simulation. Bound the active working set and benchmark pathological shared crises.

### Drama managers: useful research, wrong default responsibility

Shepherd uses incremental story sifting to guide otherwise autonomous characters toward narratively interesting choices. RimWorld’s official description likewise distinguishes its storyteller as a controller of incoming events. [AAAI Publications](https://ojs.aaai.org/index.php/AIIDE/article/view/31887)

**TCE lesson:** this is a boundary, not a feature to copy. Selecting which naturally occurring events the observer sees is compatible with TCE’s premise. Steering citizens to complete a satisfying arc is a different simulation design.

---

## 3. Numbers, storage, and performance

### 3.1 What the published evidence actually establishes

| Evidence | Reported figure | Appropriate interpretation |
| --- | --- | --- |
| **Felt authoring case study** | Four high-school interns; one day of instruction; 85 actions authored within a week | Encouraging authorability evidence, not a controlled usability or scale benchmark |
| **Winnow prototype** | 30 event types, five characters, 100 random events per run | A small incremental-matching benchmark |
| **Older Legends Browser** | Some exports exceeded 400 MB and required approximately twice their size in loaded memory | Maintainer-reported import/heap concern, not a universal ratio for history databases |

The Felt result included actions with sifting patterns. The Legends Browser figures concern that particular Java utility and export representation. [Max Kreminski](https://mkremins.github.io/publications/Felt_SimpleStorySifter.pdf)

Winnow’s reported average update times were:

| Pending matches | Mean processing time per new event |
| --- | --- |
| 10 | 13 ms |
| 50 | 50 ms |
| 100 | 93 ms |
| 500 | 460 ms |
| 1,000 | 912 ms |

The test used browser JavaScript, Firefox 87, and a 2019 MacBook Pro with a 2.6 GHz six-core i7 and 16 GB RAM; the implementation favored clarity over performance. These numbers are **not** estimates for an optimized Rust design. [Max Kreminski](https://mkremins.github.io/publications/Winnow_AIIDE2021.pdf)

I found no directly comparable published benchmark demonstrating this complete chronicle pipeline at TCE’s population and historical duration.

### 3.2 The storage arithmetic is more important than prose generation

For illustration, assume 365-day years, a constant population, and a hypothetical average **128-byte stored event before indexes and additional payloads**:

| Illustrative logging policy | Records over 100 years | Event bytes alone |
| --- | --- | --- |
| 10,000 people × one event/person/day | 365 million | 46.72 GB |
| 50,000 people × one event/person/day | 1.825 billion | 233.60 GB |
| Fixed 1,000 historical events/world/day | 36.5 million | 4.67 GB |

These are arithmetic scenarios, not measured TCE event rates or recommended capture quotas. Institutional events and unusually large records would add further storage.

Consequently, **unbounded duration, arbitrary retained detail, and fixed storage cannot all be guaranteed**.

A practical policy is bounded RAM plus a tiered disk archive:

**Recent detail:** rich events and supporting context for active histories.

**Long-term historical backbone:** identity milestones, institutional transitions, major episodes, important causal records, and aggregated trends.

**Optional deep archive:** compressed older detail that remains available when disk growth is acceptable.

Preserve referential integrity during compaction. A retained explanation should not point silently to a deleted reason. Store an explicit summary or availability marker when detailed evidence is removed.

Also distinguish retention from front-page ranking. “Not interesting enough for this year’s digest” must not mean “safe to erase.”

### 3.3 Budget for fast-forward, not only normal play

The relevant throughput is:

\[
\text{history events per wall-clock second}
=
\text{events per simulated day}
\times
\text{simulated days per wall-clock second}.
\]

For the first implementation, I would use these **provisional engineering targets**, to be revised after profiling:

| Concern | Proposed target or rule |
| --- | --- |
| Kernel hot path | Typed event emission only; no text generation or archive queries |
| CPU allocation | Initially budget history processing at no more than roughly 5% of simulation CPU time |
| Warm browsing | Aim for under 100 ms at the 95th percentile for common indexed queries |
| Active working set | Configurable hard cap on caches and partial matches |
| Overload | Delay optional sifting/narration; preserve required historical records |
| Storage failure | Surface an explicit pause/degradation decision rather than silently losing canonical history |

Benchmark long peaceful runs as well as wars and disasters. A mass migration or epidemic may create many overlapping candidate stories involving the same locations and institutions.

Cache episode plans and indexes before caching elaborate prose. A history service that answers questions quickly is more valuable than a cache of beautifully written paragraphs that are difficult to search.

---

## 4. Lessons and implementation priorities for TCE

### Adopt

**A linked, structured archive with multiple views.** One factual record should support biographies, annual chronicles, maps, building histories, and explanations.

**Read-only story sifting.** Recognize meaningful relationships in events without modifying citizens to produce preferred outcomes.

**Templates as a complete shipping path.** They provide deterministic output, localization control, predictable costs, and a fallback for optional generation.

### Adapt

**Personal memories and artifacts.** Use them to make history visible in daily life, while keeping subjective recollection separate from canonical records.

**Salience scoring.** Make it perspective-dependent and diversity-aware. A world digest, family biography, and history of architecture should select different material.

**Long-term retention.** Keep enough ordinary historical structure that later developments can make earlier events meaningful; do not preserve only yesterday’s headlines.

### Avoid

**Raw-log-to-LLM narration as the primary system.** It entangles selection, inference, and phrasing, making mistakes harder to diagnose.

**One giant global timeline or relationship graph.** It exposes data volume rather than helping people understand it.

**Mandatory dramatic closure.** Unresolved, quiet, and unsuccessful processes are part of an unscripted history.

**Generated explanations that feed back into simulation truth.** The narrator must not become an invisible author of citizens’ motives.

### Recommended development sequence

| Stage | Deliverable | Acceptance test |
| --- | --- | --- |
| **1. Historical substrate** | Typed events, stable identities, historical-state access, export | Selected entities remain explainable after death, renaming, and ownership changes |
| **2. Readable baseline** | Event templates, entity pages, filters, annual summaries | A reader can reconstruct a settlement’s major changes without external notes |
| **3. Episode selection** | Initial pattern library, salience features, deduplication | Better comprehension and less repetition than a ranked event list |
| **4. Long-duration support** | Compaction, historical maps, bounded partial matches | Century-scale tests preserve required explanations within budgets |
| **5. Optional narration** | Evidence packets, validation, caching, fallback | Prose improves readability without increasing unsupported claims |

Test quality through tasks, not only ratings: ask readers to explain why a settlement moved, identify who benefited from a law, or trace a technology’s arrival. Compare a raw timeline, ranked events, grouped episodes, and narrated episodes.

Audit factual support, missing context, repeated content, topic imbalance, retrieval time, and ability to retell the main developments accurately. A fluent chronicle that makes readers confidently misunderstand the simulation is a failure.

**The desired result is not a novel automatically written about every world. It is an explorable history that helps observers discover, understand, and retell the stories that genuinely occurred.**

---

## 5. Selected sources

| Resource | Why it is useful |
| --- | --- |
| [Kreminski, Dickinson & Wardrip-Fruin — *Felt: A Simple Story Sifter*](https://mkremins.github.io/publications/Felt_SimpleStorySifter.pdf?utm_source=chatgpt.com), with [source repository](https://github.com/mkremins/felt?utm_source=chatgpt.com) | Pattern-based selection, authoring experience, and debugging lessons |
| [Kreminski, Dickinson & Mateas — *Winnow*](https://mkremins.github.io/publications/Winnow_AIIDE2021.pdf?utm_source=chatgpt.com), with [source repository](https://github.com/mkremins/winnow?utm_source=chatgpt.com) | Incremental matching, temporal semantics, partial-match management, and benchmark |
| [Flores & Thue — *Level of Detail Event Generation*](https://rise.csit.carleton.ca/pubs/FloresThue_ICIDS_2017.pdf?utm_source=chatgpt.com) | Event salience and connections to the event-indexing model |
| [Méndez & Gervás — *Using ChatGPT for Story Sifting in Narrative Generation*](https://computationalcreativity.net/iccc23/papers/ICCC-2023_paper_124.pdf?utm_source=chatgpt.com) | Direct experiment separating relevant selection from fluent realization |
| [Deo, Chung & McCoy — *Shepherd*](https://ojs.aaai.org/index.php/AIIDE/article/view/31887?utm_source=chatgpt.com) | The contrasting approach: using sifting to steer a simulation |
| [Dwarf Fortress Legends documentation](https://dwarffortresswiki.org/index.php/Legends?utm_source=chatgpt.com) | Historical entities, navigation, knowledge, and export limitations |
| [LegendsViewer‑Next](https://github.com/Kromtec/LegendsViewer-Next?utm_source=chatgpt.com) and [Legends Browser](https://github.com/robertjanetzko/LegendsBrowser?utm_source=chatgpt.com) | Concrete browsing designs and maintainer-documented memory considerations |
| [Paradox — CKII: Charlemagne](https://www.paradoxinteractive.com/games/crusader-kings-ii/add-ons/crusader-kings-ii-charlemagne?utm_source=chatgpt.com) | Official description of the annual chronicle |
| [Paradox — CKIII Dev Diary #105](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1662033633&feed=steam_community_announcements&utm_source=chatgpt.com) | Memories, relationship reasons, retention, and clipboard export |
| [Ludeon — RimWorld Alpha 9 release notes](https://ludeon.com/blog/2015/02/rimworld-alpha-9-tales-o-drunkness-released/?utm_source=chatgpt.com) | Developer documentation of tales represented in sculptures and engravings |
| [Tynan Sylvester — GDC 2017 design talk](https://www.gdcvault.com/play/1024232/-RimWorld-Contrarian-Ridiculous-and?utm_source=chatgpt.com) | Story-generator framing and deliberate room for player interpretation |
| [RimStory documentation and discussion](https://steamcommunity.com/sharedfiles/filedetails/?id=1713190031&utm_source=chatgpt.com) | Community motivation for historical records, commemorations, and export |

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92996-4b4c-83ea-9063-06d8746be1a1)
