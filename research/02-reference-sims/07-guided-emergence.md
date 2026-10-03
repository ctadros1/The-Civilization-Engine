# Guided emergence: authoring the vocabulary without authoring history

## Recommendation

**TCE should guide the space of possible behavior and the observer’s attention—not the sequence of historical outcomes.** Its strongest combination would be persistent causal simulation, context-sensitive agent decisions, reusable institutional procedures, and a separate layer that finds and explains interesting developments.

The six games illustrate importantly different meanings of “emergent narrative.” Dwarf Fortress builds interacting systems and persistent history; RimWorld deliberately directs incidents; Caves of Qud generates historical accounts without fully simulating their causes; Wildermyth casts evolving characters into substantially authored stories. These are complementary design references, not interchangeable architectures. [GameAIPRO](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter41_Simulation_Principles_from_Dwarf_Fortress.pdf)

For TCE, the useful distinction is:

| Authored element | Example for TCE | Compatibility |
| --- | --- | --- |
| **An action or procedure** | People can propose a common granary, contribute resources, appoint a custodian, and dispute withdrawals. | Fully compatible: vocabulary. |
| **A decision bias** | A household facing repeated shortages considers collective storage more often. | Compatible when grounded in its experience and capabilities. |
| **A historical outcome** | Every settlement establishes communal storage after its third famine. | A plot rule disguised as emergence. |
| **An observation pattern** | Detect that several households pooled grain and later disputed its control. | Fully compatible: interpreting events without causing them. |

The goal should be **surprising beforehand, understandable afterward**. Neither maximum randomness nor maximum detail guarantees that.

The discussion below distinguishes historical developer descriptions from current implementations. Published figures are mostly content and balance parameters; they do **not** establish that the same systems scale to 50,000 continuously active citizens.

---

## 1. How the six approaches work

### Dwarf Fortress: persistent consequences, selectively detailed mechanisms

**Mechanism.** Dwarf Fortress’s world generation builds environmental conditions, then simulates civilizations establishing settlements, trading, and fighting. Tarn Adams’s design advice is especially relevant: model interacting underlying factors rather than directly producing the desired visible result. His example is deriving biomes from temperature, rainfall, elevation, and drainage instead of placing biomes as isolated categories. He also warns against variables that have no meaningful visible effect. [GameAIPRO](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter41_Simulation_Principles_from_Dwarf_Fortress.pdf)

Persistence then gives otherwise ordinary events significance. The world records historical changes across games; players can revisit retired fortresses and meet previous characters. Local systems expose thoughts, needs, injuries, institutions, and material interactions. A consequence can therefore survive the immediate incident and become part of somebody else’s experience. [Bay 12 Games](https://www.bay12games.com/dwarves/features.html)

**Where the designers intervened.** Recent fixes show that “let the simulation run” is not the whole philosophy:

* **53.07, December 2025:** added positive thoughts after defeating a siege and prevented items thrown at visitors during tantrums from automatically escalating into lethal combat.
* **53.16, August 2026:** stopped patrons ordering drinks from every tavern keeper; added intoxication-sensitive restraint; disabled the family-contact need until isolated residents had ways to contact distant relatives. [Bay 12 Games](https://www.bay12games.com/dwarves/)

These are revealing interventions. They repair excessive feedback, missing restraint, and needs without achievable responses. They do not prescribe a particular fortress’s fate.

**What worked—and what failed.** The transferable success is durable identity and consequence. The documented failures show how an individually reasonable-looking rule can become unreasonable when duplicated across many agents or combined with another subsystem.

**For TCE:** adopt persistence and causal composition, not exhaustive detail. A citizen’s frustration should have several possible responses—negotiation, withdrawal, relocation, adaptation—not just another increment toward violence. Every major need should have an achievable response under some conditions. When no response exists, that is usually a missing mechanism, not desirable tragedy.

The strongest Dwarf Fortress lesson is: **repair the causal model before adding an event that compensates for its failure.**

### RimWorld: an incident director feeding a smaller social simulation

**Mechanism.** RimWorld separates external incident selection from colonists’ local lives. Its official description explicitly assigns raids, weather incidents, and visiting traders to the storyteller. Cassandra, Phoebe, and Randy represent different approaches to pressure and pacing. Inside those incidents, backgrounds, abilities, relationships, injuries, and mood determine what happens to particular people. [RimWorld](https://rimworldgame.com/)

This distinction matters: the storyteller need not write a betrayal or heroic recovery directly. It supplies circumstances, while the colony’s state produces consequences. Tynan Sylvester’s GDC presentation frames the entire design around generating stories, including deliberately omitting features that do not improve that experience. [GDC Vault](https://www.gdcvault.com/play/1024232/-RimWorld-Contrarian-Ridiculous-and)

**Where the rails are.** Ludeon’s 1.2-era documentation explicitly describes three separable influences: wealth, population, and adaptation to losses. Adaptation increases challenge while the player avoids losses. Custom difficulty also introduced a wealth-independent alternative whose threat progression could be controlled by elapsed time. This is deliberate experience management, not an impartial model of surrounding societies. [Steam Store](https://store.steampowered.com/news/posts/?appids=294100&enddate=1625426156&feed=steam_community_announcements)

**A concrete balancing failure.** The same release documentation explains that larger raids previously brought larger rewards because loot followed the attackers’ equipment and inventories. Ludeon introduced a separately tuned core raid reward to avoid higher difficulty automatically providing bigger rewards. Pressure generation and economic reward had unintentionally reinforced one another. [Steam Store](https://store.steampowered.com/news/posts/?appids=294100&enddate=1625426156&feed=steam_community_announcements)

**For TCE:** borrow the separation between circumstances and personal consequences, but not the protagonist-centered difficulty controller.

A drought generated by climate conditions is compatible with TCE. A neighboring polity attacking because it wants resources, believes it can win, and can supply an army is compatible. A raid becoming stronger because the observed settlement accumulated wealth is not sufficient explanation by itself.

TCE can still have pacing—but primarily **pacing of observation**: when one settlement is quiet, surface an apprenticeship, migration, infrastructure project, or political dispute elsewhere. Do not punish a successful city merely because its stability reduces spectacle.

### Crusader Kings III: authored institutions, differentiated decisions, and corrective event systems

**Mechanism.** CK3 combines continuity through characters and dynasties with authored political structures and event content. Its relationship writing explicitly revisits personal history, humiliation, affection, and inter-house grievances rather than treating each encounter as independent. [Steam Store](https://store.steampowered.com/app/1158310/Crusader_Kings_III/)

The most useful TCE evidence is **Dev Diary #104, August 2022**. Developers identified several failures: economic behavior was too similar across rulers; too much money went toward warfare; fragmented realms often failed to reunify. They introduced differentiated economic archetypes, staged investment priorities, and more purposeful diplomatic expansion. Rulers would more frequently consider vassalization and improve relationships with plausible targets. [Steam Store](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1661862855&feed=steam_community_announcements)

This is guided emergence through **competence and preference**, not a command that a particular kingdom must form.

The diary also documented a straightforward implementation error: would-be religious defenders checked the wrong character, so the intended defense behavior never activated. Before changing historical balance, the designers needed to make an existing mechanism work. [Steam Store](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1661862855&feed=steam_community_announcements)

**A different intervention: harm events.** In **Dev Diary #129, 2023**, the designers acknowledged that avoiding frustrating random deaths had overcorrected toward predictable, safe rulers. Harm events reintroduced death and incapacity, usually with advance warning and cooldowns. This was an explicitly authored correction to the experience of succession. [Dev Trackers](https://devtrackers.gg/crusader-kings/p/23b2584b-dev-diary-129-post-release-update-extra-content)

Community reactions expose the trade-off. Some players welcomed unexpected succession crises; others objected to arbitrary incapacitation and long periods with little agency. These are firsthand experience reports, not reliable estimates of event frequency or representative surveys. [Reddit](https://www.reddit.com/r/CrusaderKings/comments/14f9cpb/stop_hating_on_harm_events/)

**For TCE:** preferentially copy the economic and diplomatic repairs, not the corrective catastrophe deck.

Give cautious, ambitious, generous, and status-seeking people different decision weights—but ensure that all can recognize basic opportunities. A society that never builds infrastructure may lack financing or coordination, rather than insufficient “builder personalities.”

Use physical and social risk to generate uncertainty. Do not target unusually successful people with misfortune simply to destabilize their societies.

### Caves of Qud: coherent historical accounts without a complete historical simulation

**Mechanism.** Jason Grinblat’s GDC explanation is unusually explicit: Qud avoided a full historical simulation by generating historical events and rationalizing them afterward. This is a system for producing evocative accounts, not evidence that the described motives actually drove simulated decisions. [GDC Vault](https://www.gdcvault.com/play/1024990/Procedurally-Generating-History-in-Caves)

In the described generator, sultans, places, and items are entities. Events modify their state, and replacement grammars produce short historical passages. Persistent properties—alliances, cultural domains, recurring themes—connect otherwise separate events. A relationship with frogs, for example, can supply the explanation that a military action defended frogs. Historic sites, relics, and cults make the generated accounts tangible in the playable world. [GDC Vault](https://gdcvault.com/play/mediaProxy.php?sid=1025379)

**Where the authors provide structure.** Event vocabulary, grammatical possibilities, recurring motifs, and the deliberately narrow biographical scope do much of the work. The system avoids requiring every lifetime to follow a conventional narrative arc; coherence comes partly from recurring entities and associations. [GDC Vault](https://gdcvault.com/play/mediaProxy.php?sid=1025379)

**What worked—and the boundary.** This is a successful scope reduction for mythic history. It should not be misdiagnosed as a failed attempt at causal simulation. But its central shortcut would violate TCE’s promise if applied to authoritative history.

**For TCE:** adopt the delivery mechanisms and reverse the causal direction:

> Qud-like account generation: select event → update context → explain it.  
> TCE history: agents act for recorded reasons → consequences occur → generate an account.

Reuse real people, places, objects, and symbols across accounts. Let a bridge, inherited workshop, festival, or ruined granary carry historical meaning.

Also distinguish **what happened**, **what people believe happened**, and **what a chronicler claims happened**. A ruler may commission a self-serving account, but the engine must not confuse that account with its own causal record.

### Wildermyth: authored scenes, state-sensitive casting, lasting character change

**Mechanism.** Wildermyth is not primarily a simulation discovering complete plots. Its developers explained that even generic procedural campaigns use handwritten event differences and personality-sensitive dialogue. Fully generated, consistently voiced comic narratives were outside their chosen scope. [Steam Community](https://steamcommunity.com/app/763890/discussions/0/2647504242056635823/)

Events are selected using encounter type and matching conditions such as personality, relationships, and character hooks. Characters occupy roles within an authored scene. Those scenes can then change characters and their subsequent possibilities. [Wildermyth](https://wildermyth.com/wiki/Event)

Its historical event-writing guidelines explain the intended legibility: establish who is present, where they are, and what is at stake; let roles reinforce ongoing personalities; show the characters visually; keep text brief. An example couples a supernatural encounter to persistent biography and mechanical changes rather than disposable flavor text. The page explicitly labels itself an early design document, not a current specification. [Wildermyth](https://wildermyth.com/wiki/Event_design_philosophy)

**Where the rails are—and why.** Authored scenes preserve voice and dramatic shape. The selection system manages repetition across campaigns. Repeatable “return to a site” events deliberately avoid large permanent rewards so that players are not encouraged to farm them. [Wildermyth](https://wildermyth.com/wiki/Event)

The larger continuity mechanism is equally important: heroes age, change, develop relationships, and enter a Legacy from which favorites can return in later campaigns. [Wildermyth](https://wildermyth.com/press/sheet.php?p=wildermyth)

**For TCE:** borrow contextual presentation and persistent consequences, not authored plot delivery.

A council meeting can be summarized with the actual chair, creditor, dissenter, and affected household. But do not assign someone the role of “dissenter” just because a scene needs conflict.

Apply anti-repetition weights to **which events receive prominent presentation**, not to whether recurring real events occur. The fifth grain shortage may be historically decisive even if the first four already appeared in the chronicle.

### Ultima Ratio Regum: constrained generators and authored detail integrated into the world

**Mechanism.** Mark R. Johnson’s GDC presentation rejects a single universal generation strategy. In its examples, mansions are predominantly handmade with variation because many functional requirements must fit into small spaces. Cathedrals permit more algorithmic construction because larger spaces and fewer strict constraints offer greater freedom. Religious objects mix several algorithms with rare handmade cases. [GDC Vault](https://media.gdcvault.com/gdceurope2015/Johnson_Mark_HandmadeDetailIn.pdf)

This is authored structure **inside** procedural generation, rather than a fixed quest sequence laid over arbitrary generated terrain. Johnson reported that few players distinguished the two kinds of content, but that is developer observation, not a controlled study. [GDC Vault](https://media.gdcvault.com/gdceurope2015/Johnson_Mark_HandmadeDetailIn.pdf)

**A concrete unresolved challenge.** In a July 2026 development entry, Johnson discussed generating picture clues from actual or generatable building layouts. Similar interiors created an information problem: a picture of common chairs, walls, and windows might not identify a useful location. Cultural and religious distinctions could help. The post concerns work toward 0.11, including incomplete integration—not proof of a finished, validated detective system. [Dr Mark R Johnson](https://www.markrjohnsongames.com/2026/07/05/ultima-ratio-regum-0-11-update-63-picture-clues-generation-1-and-soundtrack-news/)

**For TCE:** author constrained subassemblies where they save effort, while keeping their selection and construction grounded in local decisions.

A plausible house can use authored room adjacencies, structural constraints, and roof rules. Its builder, materials, dimensions, financing, and later extensions can still emerge. Hand-authoring a functional component does not script the building’s history.

The stronger warning is that **visual variety is not necessarily meaningful variety**. Architectural differences become legible when they express something consequential: material availability, craft tradition, patronage, household organization, institutional use, or inherited modifications.

---

## 2. What worked and what failed: the recurring patterns

The cases suggest a useful diagnosis framework. The following is a design synthesis, rather than a claim that every game experienced every failure.

| Failure mode | What it would look like in TCE | Better intervention |
| --- | --- | --- |
| **Runaway positive feedback** | One shortage produces unrest, reduced production, more shortage, and universal collapse. | Add plausible buffers, alternative responses, recovery mechanisms, and limits on repeated effects. |
| **Predictable sameness** | Different names and maps repeatedly produce the same institutions and dominant strategy. | Introduce consequential differences in resources, information, coordination costs, inherited commitments, and preferences. |
| **Agents cannot use available systems** | Resources and rules permit cooperation, but nobody proposes or organizes it. | Repair action discovery, communication, planning, and responsibility assignment. |
| **Randomness mistaken for depth** | Personalities and alliances change so rapidly that nothing can be anticipated. | Preserve commitments and identity; let change follow experience and incentives. |
| **Invisible causality** | Rich events occur, but the observer sees only statistics and disconnected notices. | Store provenance, connect recurring entities, and present causal summaries. |
| **Authored content becomes recognizable** | Repeated scenes reveal the same scenario beneath different names. | Expand mechanically meaningful combinations; manage presentation repetition without suppressing real recurrence. |

Three distinctions are especially important.

**Cosmetic diversity is not behavioral diversity.** Different roof colors or government names matter less than different ownership arrangements, succession procedures, resource dependencies, or methods of resolving disputes.

**Stable does not mean broken.** A successful agrarian society can remain relatively stable. TCE should not treat the absence of conquest, urbanization, or revolution by an arbitrary date as failure.

**Authored constraints can increase useful emergence.** Constraints eliminate incoherent combinations and make consequences interpretable. The question is not whether a rule restricts possibility; every rule does. The question is whether it preserves multiple plausible routes and outcomes.

---

## 3. Numbers: what the published evidence actually supports

| System and source | Reported figure | Interpretation for TCE |
| --- | --- | --- |
| **Qud, 2018 history-generator presentation** | Five generated sultans; approximately 12 event iterations per life in the described process. | Rich-seeming history can come from a compact event structure. This is not a simulated-population benchmark. [GDC Vault](https://gdcvault.com/play/mediaProxy.php?sid=1025379) |
| **CK3 harm-event design, 2023** | Foreboding carried a 50% chance of follow-up harm within 4–8 years; described cooldowns were 50 years for the player and 30 years per AI house. | Explicit temporal controls moderated an authored disruption system. These are historical design parameters, not recommended TCE settings. [Dev Trackers](https://devtrackers.gg/crusader-kings/p/23b2584b-dev-diary-129-post-release-update-extra-content) |
| **CK3 AI diagnosis, 2022** | Developers reported Christian conquest of the Middle East in at least 90% of observer games reaching the end date before the discussed corrections. | A repeatable macro-outcome can arise from a missing or broken mechanism. The diary gives no sample count or confidence interval. [Steam Store](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1661862855&feed=steam_community_announcements) |
| **Wildermyth event selection** | An event occurrence reduces its next-campaign selection weight to one quarter; an entire campaign without it doubles the weight toward its default. | Simple recency weighting can reduce visible repetition. It is not a cure for limited underlying behavior. [Wildermyth](https://wildermyth.com/wiki/Event) |
| **URR, July 2026 development account** | More than 2,000 generative grids were available to the discussed infrastructure. | Authored procedural vocabulary can become substantial. Content count does not establish meaningful diversity or runtime throughput. [Dr Mark R Johnson](https://www.markrjohnsongames.com/2026/07/05/ultima-ratio-regum-0-11-update-63-picture-clues-generation-1-and-soundtrack-news/) |

There is also relevant experimental evidence for **story sifting**: selecting interesting sequences from events that already occurred. *Select the Unexpected* tested a statistical ranking method on one simulation run containing **20 characters, 24 action types, and 1,000 events**. Three raters compared 15 story pairs; they preferred the heuristic-selected story in **38 of 45 judgments, or 84.4%**. This is encouraging but small-scale evidence—not validation for a 50,000-person historical world. [Max Kreminski](https://mkremins.github.io/publications/StU_ICIDS2022.pdf)

### Two scale calculations for TCE

These are illustrative engineering calculations, **not measured game performance**.

At 50,000 people, retaining 20 directed social links per person means **one million links**, rather than approximately **2.5 billion ordered person-pairs**. That strongly favors sparse, local relationship structures over universal relationship evaluation.

History storage is potentially more dangerous. Assuming 365-day years, even **one 64-byte event per person per day** produces approximately **116.8 GB over a century** at 50,000 people, before indexes or text.

Consequently, TCE needs different retention policies for routine activity, significant changes, and durable historical evidence. “Endless history” cannot mean “keep every low-level event forever in an immediately queryable graph.”

---

## 4. Lessons for TCE: a concrete design

### A. Author actions and institutions as causal building blocks

The authored vocabulary should specify **what actors can attempt, what conditions permit it, what it costs, and what changes when it succeeds or fails**.

For an institutional action, that means more than a name and probability. A useful definition should cover eligible participants, required authority or consent, resource commitments, information requirements, decision procedure, enforcement, and exit or dissolution.

For example, a common granary is not a single “communalism” switch. It can involve separate primitives for contribution, storage, access rights, stewardship, accounting, sanctions, and dispute resolution. Different combinations can produce household cooperation, an elite-controlled store, a temple institution, or a civic service.

Similarly, do not make technological change merely a timer plus prerequisite list. The authored vocabulary can include experimentation, demonstration, apprenticeship, imitation, equipment requirements, and incentives to adopt. Whether those combine into sustained innovation remains contingent.

**Preserve the distinction between opportunity and outcome.** Making a council possible is vocabulary. Ensuring that a council forms is a historical intervention.

### B. Keep the authoritative simulation separate from the narrative selector

Recommended architecture:

**Agent decisions → state changes → structured historical records → story detection → presentation**

The story detector should ordinarily have **read-only access**. It may discover a coalition, highlight a failed project, or summarize a migration. It should not create a rival because a promising narrative lacks one.

The *Felt* research provides a practical precedent: story patterns become queries over events and relationships, including temporal and causal connections. Its authors also developed a `whyNot` debugging facility to identify which conditions prevent a pattern from matching. That is directly useful for diagnosing apparently absent emergence. [Max Kreminski](https://mkremins.github.io/publications/Felt_SimpleStorySifter.pdf)

TCE need not adopt that research implementation wholesale. A smaller indexed event system may be sufficient. Important event records should nevertheless retain:

* Stable participant, institution, object, and place identifiers.
* The action, result, and material state changes.
* Relevant triggering events and recorded intentions.
* Whether any development fallback or runtime intervention influenced the action.

That last field should be inspectable in development tools and exportable with histories.

**Do not manufacture motives afterward.** When the engine knows an actor intended to secure food, say so. When it only knows that food insecurity preceded an action, distinguish correlation from a recorded reason.

### C. Make history legible at several scales

TCE should not expect observers to remember 50,000 people. It should offer continuity through a manageable selection of households, workplaces, institutions, places, and objects.

At the **personal scale**, preserve recognizability: occupation, relationships, habits, important commitments, injuries, property, and a short biography.

At the **institutional scale**, show how a rule or organization developed: founding participants, resources, authority, disputes, modifications, and beneficiaries.

At the **settlement scale**, expose changes in dependencies: where food comes from, who controls infrastructure, which groups cooperate, and which institutions maintain order.

Every highlighted development should answer four questions:

**What changed? Why now? Why these participants? What persists afterward?**

A headline such as “Council establishes a toll” is weak. A useful explanation connects it to the bridge, the financing agreement, the supporters, the households paying, and the future holder of the right.

The interface should support a short summary followed by progressive disclosure—not force all causal detail into a paragraph. A history browser should permit movement from person to institution to place, and from an outcome back to its contributing events.

### D. Use material traces, not just event text

Qud and Wildermyth demonstrate different ways of making an event recur in later experience: through worldly evidence or continuing character change. [GDC Vault](https://gdcvault.com/play/mediaProxy.php?sid=1025379)

For TCE, prioritize consequences that remain visible:

A workshop retains an apprentice’s connection to its founder. A neighborhood’s layout reflects earlier property boundaries. A repaired bridge carries the insignia of its financier. A festival commemorates a particular disaster. An inherited office preserves an arrangement originally created for a temporary task.

These are proposed TCE mechanisms, not claims about historical inevitability.

Cultural consistency should itself have causes. Builders learn from builders; patrons copy prestigious forms; migrants bring practices; institutions commission recognizable symbols. Avoid assigning an entire civilization an instantly synchronized visual identity that changes everywhere at once.

### E. Interpret the time-box as an engineering rule, not a historical deadline

The phrase “nudge only when emergence fails after a time-box” needs a precise operational meaning.

**A development time-box is useful. A mandatory in-world progress deadline is dangerous.**

“No city formed by year 100” is not enough to justify intervention. First ask whether city formation was materially plausible, whether anyone benefited from it, whether coordination was possible, and whether agents could perceive and pursue the opportunity.

Use the following escalation order:

| Diagnosis | First response |
| --- | --- |
| The development happened but nobody noticed. | Improve detection and presentation. Do not change simulation behavior. |
| Agents need something but have no means to pursue it. | Add or repair the missing action, communication channel, institution, or planning step. |
| The action exists but is never considered. | Fix candidate generation, evaluation frequency, or eligibility mistakes. |
| Agents consistently choose a pathological option. | Examine costs, rewards, information, feedback, and alternative responses. |
| A plausible opportunity remains vanishingly rare after the time-box. | Consider a bounded initiative bias among genuinely eligible actors. Log it. |
| Progress requires creating resources, people, or institutions without causes. | Treat it as an explicit scenario mode or redesign the mechanism—not an invisible baseline correction. |

The CK3 investment and diplomacy changes are particularly relevant here: they repaired how agents used available possibilities rather than directly assigning successful successor kingdoms. [Steam Store](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1661862855&feed=steam_community_announcements)

A permissible fallback might increase the chance that a qualified, aggrieved citizen proposes a meeting. It should not create consensus, guarantee adoption, waive costs, or determine what the meeting decides.

Natural stochasticity remains legitimate. Floods, disease exposure, crop variability, and accidents need not be fully predictable. Their distributions and consequences should be grounded in the modeled world, rather than selected because the observer has gone too long without drama.

### F. Test one causal chain before building an “interestingness director”

A useful initial vertical slice would connect **infrastructure, finance, household welfare, and institutional change**.

For example, the following is an illustrative possible history—not a scripted target:

> A flood destroys a bridge. Grain carriers reroute, raising transport costs in one district. A warehouse owner extends credit to affected households and finances repairs. Supporters negotiate a toll concession. The concession persists after the financier dies, and its inheritance later becomes politically contested.

This uses authored primitives but does not require an authored “rise of oligarchy” plot.

In another run, households might finance a shared crossing, a council might collect contributions, traffic might move permanently elsewhere, or the district might decline. Those alternatives must remain genuine possibilities.

The test should establish that the agents can perceive opportunities, make commitments, pay costs, coordinate, and leave durable changes. Only afterward should presentation systems identify the development as a story.

### G. Make surprise relative to context—not synonymous with disaster

The *Select the Unexpected* approach ranks a story relative to other matches of the same general pattern. Its discussion notes that peaceful conflict resolution could be unusual in a violent simulation, just as violence could be unusual in a peaceful one. [Max Kreminski](https://mkremins.github.io/publications/StU_ICIDS2022.pdf)

For TCE, this suggests a presentation score combining **consequence, contextual unusualness, continuity, and explanatory confidence**. It should not simply count deaths, wars, betrayals, or revolutions.

A long collaboration between rivals may be more surprising than another assassination. An institution that quietly survives several successions may be more significant than a brief revolt.

Keep some presentation capacity for ordinary life and broad trends. Otherwise, a read-only story selector can still misrepresent the world by showing only exceptional violence.

### H. Validate diversity, causality, and cost separately

Use a small, repeatable collection of headless worlds during normal development, with deeper investigations reserved for specific failures.

**Causal checks:** Can important outcomes be traced to actual actions, resources, authority, and information? Does disabling the narrative layer leave simulation behavior unchanged?

**Capability checks:** In constructed situations where cooperation, migration, investment, or reform is feasible, can agents attempt it without intervention?

**Diversity checks:** Do worlds differ in consequential structures, or only names and surface details? Distinguish legitimate convergence under similar conditions from universal convergence caused by one dominant strategy.

**Legibility checks:** After observing a short sequence, can a tester explain what changed and cite the evidence? Compare their explanation with the recorded causes.

**Runtime checks:** Benchmark event production, indexing, historical queries, and presentation separately at 10k, 25k, and 50k people. Do not run every narrative query against every person’s lifetime history on every update.

The acceptance criterion is not that every world becomes dramatic. It is that **plausible opportunities can produce multiple outcomes, consequences persist, and the observer can understand the important ones**.

---

## 5. Source guide

These are the most useful starting points for implementation and design review.

| Topic | Sources | Best use |
| --- | --- | --- |
| **Dwarf Fortress** | Tarn Adams, [*Simulation Principles from Dwarf Fortress*](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter41_Simulation_Principles_from_Dwarf_Fortress.pdf?utm_source=chatgpt.com); [official development log](https://www.bay12games.com/dwarves/?utm_source=chatgpt.com); [official features](https://www.bay12games.com/dwarves/features.html?utm_source=chatgpt.com). | Causal decomposition, selective detail, persistence, and concrete feedback/behavior repairs. |
| **RimWorld** | Tynan Sylvester, [GDC 2017 design talk](https://www.gdcvault.com/play/1024232/-RimWorld-Contrarian-Ridiculous-and?utm_source=chatgpt.com); [official game description](https://rimworldgame.com/?utm_source=chatgpt.com); [1.2-era official release notes](https://store.steampowered.com/news/posts/?appids=294100&enddate=1625426156&feed=steam_community_announcements&utm_source=chatgpt.com). | Story-generator philosophy, deliberate incident direction, adaptation, and economic balancing. |
| **Crusader Kings III** | [Dev Diary #104: AI AI AI!](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1661862855&feed=steam_community_announcements&utm_source=chatgpt.com); [Dev Diary #129 developer text, mirrored by DevTrackers](https://devtrackers.gg/crusader-kings/p/23b2584b-dev-diary-129-post-release-update-extra-content?utm_source=chatgpt.com); [firsthand community debate on harm events](https://www.reddit.com/r/CrusaderKings/comments/14f9cpb/stop_hating_on_harm_events/?utm_source=chatgpt.com). | Competence versus personality, missing mechanisms, uncertainty, and loss of agency. |
| **Caves of Qud** | Jason Grinblat, [GDC 2018 talk](https://www.gdcvault.com/play/1024990/Procedurally-Generating-History-in-Caves?utm_source=chatgpt.com) and [slides](https://gdcvault.com/play/mediaProxy.php?sid=1025379&utm_source=chatgpt.com). | Replacement grammars, recurring motifs, historical artifacts, and the distinction between accounts and causal history. |
| **Wildermyth** | [Developer explanation of procedural campaigns](https://steamcommunity.com/app/763890/discussions/0/2647504242056635823/?utm_source=chatgpt.com); [event-selection wiki](https://wildermyth.com/wiki/Event?utm_source=chatgpt.com); [historical event-design guidelines](https://wildermyth.com/wiki/Event_design_philosophy?utm_source=chatgpt.com). | Role matching, authored scenes, persistent outcomes, brevity, and repetition management. |
| **Ultima Ratio Regum** | Mark R. Johnson, [*Handmade Detail in a Procedural World*, GDC Europe 2015 slides](https://media.gdcvault.com/gdceurope2015/Johnson_Mark_HandmadeDetailIn.pdf?utm_source=chatgpt.com); [July 2026 picture-clue development account](https://www.markrjohnsongames.com/2026/07/05/ultima-ratio-regum-0-11-update-63-picture-clues-generation-1-and-soundtrack-news/?utm_source=chatgpt.com). | Constrained generation, authored subassemblies, and meaningful versus merely visible variation. |
| **Story recognition research** | Kreminski and colleagues, [*Felt: A Simple Story Sifter*](https://mkremins.github.io/publications/Felt_SimpleStorySifter.pdf?utm_source=chatgpt.com); [*Select the Unexpected: A Statistical Heuristic for Story Sifting*](https://mkremins.github.io/publications/StU_ICIDS2022.pdf?utm_source=chatgpt.com). | Readable histories from event records, debugging absent patterns, and preliminary evidence for contextual surprise ranking. |

**Bottom line:** TCE does not need an invisible novelist ensuring that history advances through interesting chapters. It needs agents with usable options, systems with plausible feedback and recovery, consequences that endure, and an observer interface capable of discovering the history those systems actually produced.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927e0-e6dc-83e9-95e4-e61a0d4d7a94)
