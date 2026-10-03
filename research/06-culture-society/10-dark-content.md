# Presenting dark historical content in The Civilization Engine

## Executive recommendation

**TCE should make atrocities understandable without making them spectacles, and represent victims as continuing people rather than expended resources.** The strongest design combines explicit descriptions of coercion, identifiable decision-makers, individual lives, and persistent consequences. Graphic detail is optional; those elements are not.

For TCE, I recommend three separate controls: **what can happen in the world, how it is presented, and when the interface demands attention**. A presentation filter should not silently change history. Conversely, a world-generation setting that excludes slavery must actually prevent the institution, not merely hide its name.

The central lesson from historical-game scholarship is that rules themselves make arguments: what counts as progress, whose interests matter, and which actions are rewarded all shape the interpretation of history. Procedural emergence does not remove that authorship. McCall’s historical-problem-space framework is particularly useful here: examine the player’s role, objectives, available actions, resources, and obstacles—not just the accompanying prose. [Game Studies](https://gamestudies.org/2003/articles/mccall)

This report uses **version-specific examples**, rather than assuming every installment or patch behaves alike. Developer statements establish intentions; patch notes establish documented changes; community discussions establish particular interpretations. None alone establishes that a representation is ethically successful for its whole audience.

---

## 1. How the main approaches work—and what worked or failed

### Victoria: make slavery an institution, not just a production modifier

**Mechanics.** Victoria 3’s original slavery design connects coerced labor to employment, ownership, consumption, political power, and law. Enslaved populations can fill eligible agricultural jobs without receiving wages; their upkeep is purchased by the employing building. The immediate financial gains principally accrue to owners. Different laws distinguish abolition, debt slavery, hereditary slavery with international trade, and a legacy system divided between free and slave states. The developers also anticipated resistance and unequal circumstances after emancipation, rather than treating legal abolition as instant social equality. These are descriptions of Victoria’s model, not universal historical classifications. [Paradox Plaza Admin Forum](https://admin-forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-15-slavery.1490983/)

**What worked.** The representation gives slavery beneficiaries and political defenders. Abolition therefore changes a distribution of wealth and power, rather than merely replacing an outdated technology.

The most instructive evidence comes from subsequent changes:

* **Update 1.2, March 2023:** interest groups stopped immediately forgetting their previous slavery positions after abolition. Restorationist civil wars were also changed so that restoring slavery could actually re-enslave relevant populations.
* **Update 1.12, December 2025:** a Slave Revolt movement was added to represent self-emancipation, connecting existing revolt content to a more explicit political actor. This expanded an existing resistance concept; it was not the game’s first acknowledgement of resistance. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-78-update-1-2-changelog)

These are valuable corrections to **institutional continuity and actor agency**. They are not evidence that every player interpreted the system as intended.

**What remained contentious.** In the 2021 diary’s Steam discussion, one commenter welcomed including slavery rather than erasing it, while another objected to its presentation as a “profession.” Both reactions matter: including an institution can improve historical coverage while its database and interface categories still misrepresent what it is. This is anecdotal reception, not a representative survey. [Steam Community](https://steamcommunity.com/app/529340/eventcomments/5020940479392632912/)

**TCE lesson:** Adopt the political economy and persistent aftermath. Improve the ontology: **farmer is an occupation; enslavement is a coercive status or relationship**. A person should retain skills, relatives, aspirations, and identity through enslavement, sale, escape, and emancipation.

### Crusader Kings: make perpetrators psychologically legible—but do not confuse their psychology with morality

**Mechanics.** Crusader Kings III connects harmful actions to character traits. Archived game scripts show torture producing stress gain for compassionate characters and stress relief for sadistic characters, alongside dread-related effects. This makes behavior contingent on the acting character rather than governed by one universal moral meter. [GitHub](https://github.com/jesec/ck3-mod-base/blob/master/base/game/common/scripted_effects/00_interaction_effects.txt)

The 2025 Raid Intents developer diary makes another approach explicit: destructive raiding can generate prestige and dread while destroying buildings and development. It also describes defensive investments that reduce those effects. The design represents terror as an instrument with beneficiaries, victims, and countermeasures. [Dev Trackers](https://devtrackers.gg/crusader-kings/p/fc9ebefa-dev-diary-165-tributaries-confederations)

**What worked.** Personality can interrupt frictionless optimization. The same action is not equally easy for every ruler, and named characters provide an intelligible source of responsibility.

**What can fail.** The player can still interpret another person’s suffering as a tool for managing the ruler’s internal resources. A strategy guide, for example, rates Compassionate poorly partly because it obstructs tyrannical play and makes hostile decisions stressful. That is one guide’s optimization judgment—not proof that most players prefer cruelty—but it reveals a reading the mechanics permit. [Pro Game Guides](https://progameguides.com/crusader-kings/ck3-traits-tier-list-crusader-kings-3/?utm_source=chatgpt.com)

The design problem is not that a cruel character experiences satisfaction. It is that **the perpetrator’s satisfaction can become the most salient measure of the event**.

**TCE lesson:** Adopt personality-dependent willingness, fear, loyalty, and refusal. Keep them separate from victim outcomes. A perpetrator’s low stress does not imply low harm; popular approval does not imply legitimacy. Because TCE simulates ordinary people, it can extend the specificity usually reserved for rulers to the families affected by their decisions.

### Europa Universalis: the risks of converting people into map improvements

**Mechanics.** Europa Universalis IV represents “Slaves” as a provincial trade good, alongside ordinary commodities. This exposes slavery’s place in trade while presenting it through an economic abstraction. [EU4 Commands](https://eu4commands.com/trade-good?utm_source=chatgpt.com)

Its **Expel Minorities** feature provides a particularly concrete example of how incentives change meaning. The **1.30 patch, June 2020**, states that expulsion no longer changes the origin province’s culture or religion, instead transferring development from the origin to the colony. The change removed the earlier combination in which expulsion could help produce both domestic homogeneity and colonial growth. The patch notes establish the change; they do not establish that ethical criticism motivated it. [Game Update Notifier](https://gameupdatenotifier.com/g/europa-universalis-iv/v/1-30)

**What the reception reveals.** A contemporary Steam discussion asks why repeated expulsion no longer changes a province’s culture. The responses explain the patch, with dissatisfaction focused on the loss of that strategic function. This illustrates how players can evaluate persecution primarily through its efficiency at producing a desired map state. [Steam Community](https://steamcommunity.com/app/236850/discussions/0/2524779067023646892/)

EU V’s documented trade-good description now explicitly says that trading enslaved people physically moves populations between markets. That is a meaningful distinction from a commodity-only representation. Nevertheless, a May 2026 player discussion asks whether disappearing enslaved populations had moved, been freed, or died. The discussion demonstrates a traceability question—not proof of a simulation bug. [Paradoxpedia](https://paradoxpedia.com/eu5/goods/slaves_goods/?utm_source=chatgpt.com)

**TCE lesson:** Make demographic transformations auditable. A declining population category must resolve into actual transitions: death, movement, emancipation, reclassification, or another recorded cause. Also distinguish voluntary assimilation, conversion, coerced conversion, expulsion, and killing; one “culture conversion” operation cannot communicate all of them adequately.

### Civilization: explicit condemnation can coexist with strong optimization incentives

**Mechanics.** Civilization IV’s manual describes Slavery as a labor civic unlocked by Bronze Working that permits sacrificing population to complete production. Community strategy documentation describes the familiar “whipping” conversion: on Normal speed, one population point provides 30 base production, while a use adds a temporary unhappiness penalty. A population point is an aggregate game unit, not one individually simulated person. [Steam CDN](https://cdn.akamai.steamstatic.com/steam/apps/3900/manuals/manual.pdf)

The interface’s oppression-related unhappiness wording makes disapproval explicit. Yet extensive strategy guides teach the mechanic as production management. That combination is revealing: **condemnatory text does not cancel the argument made by an efficient conversion rule**. [CivFanatics](https://civfanatics.com/civ4/strategy/empire-management/vocum-sineratio-the-whip/)

**A different controversy concerns the whole game framework.** In 2018, Cree headman Milton Tootoosis criticized Civilization VI’s representation of the Cree within an expansion-and-conquest framework and called for greater consultation. His comments also acknowledged potential awareness benefits. The criticism should not be flattened into “all Cree opposed the game” or “there was no Indigenous participation”; it concerned who was consulted and what the rules implied about the represented society. McCall discusses the broader problem of placing different societies inside common player goals and progress measures. [PCGamesN](https://www.pcgamesn.com/civilization-vi/civ-6-poundmaker-cree-nation)

**TCE lesson:** Adopt transparent consequences, but avoid treating human beings as instantly redeemable production. Consultation must address **objectives, institutions, and causal assumptions**, not only names, clothing, music, and dialogue. For generated cultures, review the authored building blocks for the same assumptions.

### RimWorld: individual continuity creates powerful stories, but optimization can overwhelm the people

**Mechanics.** RimWorld distinguishes its original trade in captives from the **Ideology** expansion’s colony-level forced labor. The expansion connects slavery to belief-dependent approval, suppression, coercive equipment, escape, and rebellion. Because the affected pawns remain identifiable individuals, enslavement can continue an existing character story rather than replacing a person with an anonymous resource. [RimWorld](https://rimworldgame.com/ideology/)

In his **GDC 2017 talk**, Tynan Sylvester describes RimWorld as a story generator and emphasizes selective simulation: omission can leave room for players to imagine the resulting story. This is a developer account of a storytelling method, not a study demonstrating empathy or responsible reception. [GDC Vault](https://gdcvault.com/browse/gdc-17/play/1024232)

**What worked.** Recognizable people, relationships, and consequences support stories that persist beyond a single choice. The game makes room for contrasting player-authored interpretations rather than one prescribed narrative.

**What can fail.** The documented mechanics include an 85% work-speed factor, no recreation need for enslaved pawns, and suppression-dependent rebellion. These are legible balance rules, but they risk encouraging the reading that sufficiently controlled people are simply another efficient labor configuration. **An absent recreation need is particularly dangerous to import into a realistic human simulation as a claim about wellbeing.** [RimWorld Wiki](https://rimworldwiki.com/wiki/Slavery)

Patch **1.3.3076** added an alert for a likely slave rebellion. This improved predictability, but an interface that explains victims mainly through rebellion risk still presents them chiefly as a management problem. [RimWorld Wiki](https://rimworldwiki.com/wiki/Slavery)

**TCE lesson:** Adopt continuity of personhood and relationships. Do not equate compliance with contentment, or make rebellion the only recognizable form of agency. Family care, preserving traditions, negotiating, seeking allies, escaping, and pursuing ordinary ambitions should also remain visible.

### Dwarf Fortress: preserve histories—and distinguish emergent tragedy from defective rules

**Mechanics.** Dwarf Fortress’s Legends mode links historical figures, civilizations, places, artifacts, and events. Individual entries expose relationships and events, with hyperlinks connecting the records. It also distinguishes revealing world history from discovering it through play. This is a useful precedent for giving both a world-level account and a person-level account of the same history. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Legends)

**What worked.** A world does not end when a particular fortress fails. Historical records allow consequences to outlast the immediate management session.

**The patch history is especially relevant to TCE.** Bay 12’s **December 18, 2025** update stopped objects thrown at visitors by tantruming dwarves from escalating into lethal combat. Its **March 4, 2026** update stopped dwarves forgetting rarely encountered friends and grudges after a few years. These changes illustrate two different risks: excessive escalation and insufficient persistence. An emergent outcome can be memorable while still arising from a rule the developer considers defective. [Bay 12 Games](https://www.bay12games.com/dwarves/)

**A caution about community legends.** The notorious “mermaid farm” story is often repeated as a verified atrocity-producing exploit followed by an ethical intervention from the developer. A retrospective investigation found no concrete confirmation that the proposed farm actually worked or that the developer changed the game in response to it. Treat it as evidence of community imagination and discourse—not a confirmed developer postmortem. [GamesRadar+](https://www.gamesradar.com/as-dwarf-fortress-heads-to-steam-players-remember-the-worst-thing-its-community-ever-did/)

**TCE lesson:** Adopt linked historical records. Validate the causal rules independently of how compelling their stories are. Do not make accidental escalation, broken needs, or forgotten relationships masquerade as a theory of human behavior.

---

## 2. What player-reception research actually supports

The strongest evidence reviewed here concerns general game reception, not long-running, population-scale atrocity simulations. It supports useful hypotheses, but not a single experimentally validated recipe for TCE.

### Framing changes moral interpretation

Hartmann and Vorderer’s *It’s Okay to Shoot a Character* used **84 participants across two consecutive experiments**. In one experiment, providing a justification for violence reduced guilt and negative affect without producing a corresponding significant increase in enjoyment. The study does not establish long-term changes in political attitudes, and its short action-game sessions differ substantially from TCE. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1460-2466.2009.01459.x)

**Design implication:** Do not let the authoritative narrator automatically justify atrocities with phrases such as “necessary pacification.” A faction can make that claim, but the interface should attribute it to the faction. Removing graphic imagery is not a substitute for examining justification, victim identity, and responsibility.

### Meaningful engagement is not the same as enjoyment

Oliver and colleagues studied **512 respondents**, randomly assigning them to recall either a meaningful or a fun gaming experience. This was a **recall study, not an experiment assigning people to play different games**. Meaningful experiences were associated with greater appreciation, with insight and relatedness playing important roles. [digitalcommons.butler.edu](https://digitalcommons.butler.edu/ccom_papers/145/?utm_source=chatgpt.com)

**Design implication:** Evaluate comprehension, emotional significance, trust, and perceived respect—not only enjoyment, retention, or willingness to continue.

A player choosing to pause after a massacre is not necessarily evidence of a failed experience. Conversely, continuing for hours is not evidence that the portrayal was understood responsibly.

### Warnings provide choice, not a demonstrated protective shield

A meta-analysis of content warnings synthesized **12 articles and 144 effects**. It found no meaningful average reduction in distress after exposure or improvement in comprehension; warnings increased anticipatory distress on average. These findings concern the studied warning interventions, not every possible combination of notice, content filtering, and user control. [DOI](https://doi.org/10.1177/21677026231186625)

**Design implication:** Use specific notices to support informed choice, coupled with actual controls. Do not promise that a warning makes exposure psychologically safe.

A related boundary comes from the US Holocaust Memorial Museum’s educational guidance: it warns against simulations that leave learners believing they now know what it was like to suffer or participate in the Holocaust. That guidance concerns Holocaust education, not a blanket prohibition on historical strategy games, but its warning against **false experiential authority** is relevant. TCE should not claim to reproduce what being enslaved, persecuted, or bereaved “really feels like.” [United States Holocaust Memorial Museum](https://www.ushmm.org/teach/fundamentals/guidelines-for-teaching-the-holocaust)

---

## 3. Numbers: useful anchors, and what not to infer from them

### Documented mechanics and research scales

| Example | Documented figure | What it tells TCE—and what it does not |
| --- | --- | --- |
| Civilization IV, Normal speed | **30 base production per population point**; **10-turn unhappiness** from a use | A concrete example of converting people into output with temporary consequences. These are balance constants, not historical estimates. [CivFanatics](https://civfanatics.com/civ4/strategy/empire-management/ways-into-production/) |
| RimWorld slavery | **0.85× global work speed**; baseline rebellion mean time between events of **45 in-game days**, modified by conditions | Shows how coercion becomes an optimization system. The rebellion figure is a stochastic baseline, not a fixed schedule or empirical rate. [RimWorld Wiki](https://rimworldwiki.com/wiki/Slavery) |
| Hartmann–Vorderer experiments | **84 unique participants**, participating in both experiments | Useful evidence about short-term framing effects; insufficient to establish reception across TCE’s audience. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1460-2466.2009.01459.x) |
| Oliver and colleagues | **512 respondents** | Supports distinguishing meaningful appreciation from fun; does not establish that a particular portrayal causes empathy. [digitalcommons.butler.edu](https://digitalcommons.butler.edu/ccom_papers/145/?utm_source=chatgpt.com) |
| Content-warning meta-analysis | **12 articles; 144 effects** | Supports caution about protective claims, not abandonment of informed-choice controls. [DOI](https://doi.org/10.1177/21677026231186625) |

There is no defensible numerical conversion from these studies to “the ethical amount of suffering,” “the correct atrocity frequency,” or an optimal universal warning threshold.

### TCE storage and performance implications

The reviewed sources do **not** establish a performance budget for 50,000 individually simulated humans with UE5 presentation. Population aggregates, named historical figures, and actively simulated local characters are not interchangeable benchmarks.

For TCE, the following is a **design calculation**, not measured performance:

\[
50{,}000\ \text{people}\times365\ \text{days}\times100\ \text{years}
=1.825\ \text{billion person-day records}.
\]

At an assumed **64 bytes per record**, that is **116.8 GB**, before indexes, strings, relationships, or rendering data.

Therefore, retain **meaningful changes**, not complete daily snapshots. Store one incident with participant references rather than duplicating its full description into every biography. Preserve core life events for everyone, including ordinary people, while summarizing repetitive activity. Dwarf Fortress’s documentation independently warns that very long histories can produce gigabyte-scale XML exports, although that is not a directly comparable benchmark. [Dwarf Fortress Wiki](https://dwarffortresswiki.org/index.php/Legends)

Measure simulation-update time, incident-classification cost, history-query latency, and memory growth separately. A smooth renderer can conceal an increasingly expensive historical archive.

---

## 4. Lessons for TCE: a recommended design

The following is a proposed architecture and editorial policy, rather than a claim that these choices have already been validated together.

### A. Model coercive relationships separately from jobs, culture, and personality

A person should have distinct representations for **occupation, claimed or recognized status, practical freedoms, institutional affiliations, and coercive relationships**.

For example, someone can be a skilled mason, a parent, a member of a religious community, and an enslaved person at the same time. Enslavement should constrain choices; it should not replace the rest of the character.

Avoid one universal “unfree” boolean. Relevant distinctions may include restrictions on movement, compulsory labor, transferable ownership claims, hereditary status, detention, and the ability to form or maintain households. Their combinations can support different institutions without requiring a fixed historical sequence.

Likewise, separate:

**What an institution permits; what an actor believes; what a person can actually do; and what harm occurs.**

A village assembly, household, lineage, military group, or religious organization can impose coercion before a centralized state exists. Conversely, no technological stage should automatically require slavery or persecution.

Do not encode cruelty as an intrinsic characteristic of a generated ancestry or ethnicity. Institutions and beliefs should be capable of disagreement, variation, and change within every community.

### B. Simulate consequences without rigging a moral happy ending

Do not force every perpetrator to suffer an immediate economic penalty. Instead, account for **who gains, who pays, and over what period**.

For a coercive institution, the interface should distinguish owners’ income, victims’ consumption and freedom, enforcement demands, family disruption, displacement, health consequences, political opposition, and inherited advantage. These should connect to TCE’s underlying economy and demography rather than exist only as a generic “atrocity penalty.”

Nor should resistance be the only cost the model recognizes. A person can suffer severe deprivation without rebelling.

After reform, change the relevant rights and enforcement rules immediately where appropriate, but do not erase biographies, property disparities, missing relatives, distrust, or displaced populations. Victoria’s explicit fixes to post-abolition political continuity provide a useful precedent. [paradoxinteractive.com](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-78-update-1-2-changelog?utm_source=chatgpt.com)

### C. Build the presentation around a structured incident record

Use structured simulation records as the source of truth. Narrative text should describe those records, not invent additional causes or witnesses.

An incident record should connect its time and place, affected people, direct actors, issuing authority or orders, observed actions, relevant state changes, and visibility or uncertainty. Distinguish recorded orders from inferred motives.

Then expose three levels:

| Level | Recommended content | Purpose |
| --- | --- | --- |
| **Event summary** | What happened, where, affected population, responsible actor when known | Immediate comprehension without spectacle |
| **Community consequences** | Deaths, flight, household disruption, property changes, institutional responses | Explain the event beyond the moment of violence |
| **Individual histories** | Named people, prior lives, relationships, survival, death, subsequent movement | Preserve personhood and continuity |

An illustrative notification—not a historical example—might read:

> **Rivergate: killing of detained civilians**  
> The River Guard killed 83 detained residents. Another 214 residents fled the settlement.  
> Recorded orders targeted members of the Eastbank faith.  
> **Inspect events · Affected people · Community aftermath**

This should not automatically zoom the camera onto bodies or accompany the event with a triumphant sound.

Do not report “you killed 83 people” when autonomous agents acted without the player’s direction. Attribute responsibility to the simulated actors. Any player responsibility should follow the actual control model—not an assumed role as omnipotent ruler.

### D. Maintain two accounts: underlying facts and in-world claims

TCE can support an omniscient analytical view and an in-world information view, but they should never be silently mixed.

A government may describe forced expulsion as “resettlement.” Its chronicle can preserve that claim as an attributed statement. The analytical interface should identify coercion when the simulation establishes it.

Where information is incomplete, say so. An observer might know that people disappeared without knowing whether they fled, were detained, or died. Preserve the underlying transitions for later discovery rather than substituting arbitrary uncertainty into the simulation itself.

Use plain language: **enslaved people, forced expulsion, detained civilians, confiscation, killing**. Avoid bureaucratic euphemisms as the default narrator’s vocabulary. Also avoid automatically applying a more specific atrocity label solely because a casualty count crossed a threshold; the available evidence must support what the label implies.

### E. Represent ordinary life and recovery, not only spectacular suffering

The people affected should exist before the event becomes newsworthy. Their records should include work, friendships, household activity, aspirations, and cultural participation.

Afterward, show survival and action as well as injury: care, escape, mutual aid, rebuilding, reunification, remembrance, political organizing, and refusal. Do not impose an identical personality change or permanent behavioral outcome on every survivor.

Give chronic coercion interface visibility even when there is no dramatic incident. A world where slavery continues quietly should not appear harmless simply because the notification system only recognizes rebellions and massacres.

Similarly, do not reward the UI for finding ever more extreme events. Atrocity frequency should arise from the model and its scenario constraints, not from a story-selection system’s need to keep raising the stakes.

### F. Separate world rules, presentation, and attention

A single “mature content” checkbox is inadequate.

| Control layer | What it changes | Recommended behavior |
| --- | --- | --- |
| **World constraints** | Which institutions or actions can occur | Chosen before generation; recorded in the save; honestly described as a constrained world |
| **Presentation settings** | Graphic detail, audio, bodies, descriptive intensity | Non-graphic default; changes exposure without changing simulation outcomes |
| **Attention settings** | Automatic camera movement, notifications, pause behavior, detail expansion | No forced close-ups; configurable alerts; incidents remain available in history |

The distinction must be explicit:

**“Hide graphic violence” does not mean “prevent civilian killing.”**  
**“Exclude slavery from this world” must mean more than hiding slavery-related text.**

World constraints require checks throughout generation, AI decisions, scripted building blocks, migration, ownership changes, and save loading. When changing an existing world’s rules, explain the consequences or create a clearly identified branch; do not silently erase past events.

A content notice should name the relevant subjects without sensational examples. It should lead to useful settings, not merely require acknowledgement before showing the same material.

For a first release, I would exclude sexual violence as an enacted or graphically presented system rather than add it for completeness. Any later historical reference would need a separate scope, review, and ratings decision. “Realism” is not a sufficient specification for including every possible harm.

### G. Treat ratings as a content-and-interaction problem, not a gore slider

ESRB’s physical-game process explicitly considers **context, reward systems, and player control**, and requires disclosure of the most extreme relevant content, including pertinent locked-out content. Digital-only products follow a different questionnaire-based IARC process, with review and correction mechanisms. A default-off option therefore should not be treated as a ratings loophole. [ESRB Ratings](https://www.esrb.org/ratings/ratings-process/)

PEGI likewise considers the nature of violence, including violence against defenseless characters. Its discrimination descriptor concerns stereotypes likely to encourage hatred and carries PEGI 18; the mere historical depiction of discrimination is not the same criterion. [PEGI](https://pegi.info/what-do-the-labels-mean)

Two concrete cases demonstrate why inference from subject matter alone is unreliable:

**Crusader Kings III’s Xbox listing is Teen**, with Mild Violence, Suggestive Themes, Drug Reference, and Language. Its existence is evidence against assuming that every game containing cruel actions necessarily receives an adult classification—not evidence that TCE will receive Teen. [xbox.com](https://www.xbox.com/en-US/games/store/crusader-kings-iii/9mz8rzsd0nfq)

**RimWorld received R 18+ on Australian review on April 20, 2022**, with advice for high-impact themes and drug use. The review considered fantasy context, stylized presentation, and negative consequences of drug use. It was not simply a ruling that slavery requires a particular category. [Classification Government Australia](https://www.classification.gov.au/about-us/media-and-news/media-releases/rimworld-classified-r-18)

For TCE, seek classification guidance using examples of **the most extreme reachable generated situations**, not just a typical peaceful seed. Include interface text, camera behavior, available player actions, rewards, sound, and content settings. The final category cannot be determined reliably from this design brief alone.

### H. Test for understanding, not just tolerance

Use review sessions that distinguish several questions: Can participants explain what happened? Can they identify the responsible actor? Do they understand who benefited and who was harmed? Can they distinguish narrator statements from institutional propaganda? Do they understand what their content settings change?

Include historians and appropriately compensated cultural or subject-matter reviewers alongside ordinary players. Do not require personal disclosure of traumatic experience as a qualification for participating.

For the simulation itself, establish regression tests around:

**Causal integrity.** Deaths, transfers, emancipation, and displacement reconcile with actual person records.

**Constraint integrity.** Disabled content cannot return through an alternative mechanic or a generated institution’s renamed version.

**Presentation independence.** Changing text intensity, camera options, or audio does not consume simulation randomness or alter the world’s outcome.

**Perspective consistency.** Swapping generated group identities does not change the narrator’s sympathy, evidential standards, or vocabulary.

**Persistence.** Regime changes, save/load cycles, and distant migration do not accidentally erase relationships or consequences.

**Tone.** Notifications, achievements, tooltips, localization, and marketing do not inadvertently celebrate an event that the main interface treats seriously.

Dwarf Fortress’s fixes to unwanted lethal escalation and disappearing grudges illustrate why this testing belongs in the simulation layer, not only in editorial review. [Bay 12 Games](https://www.bay12games.com/dwarves/)

### Recommended launch boundary

Before allowing an atrocity to emerge in a release build, TCE should be able to answer:

**Who did what to whom, under which institution or decision, what changed, and what happened afterward?**

If the engine can produce the harm but cannot preserve those answers, defer that content rather than place a disclaimer over an unexplained event.

The most valuable combination to adopt is **Victoria’s institutional causality, Crusader Kings’ identifiable actors, RimWorld’s continuity of individual lives, and Dwarf Fortress’s linked histories**—with a presentation policy that avoids turning those lives into trophies or making graphic exposure compulsory.

---

## 5. Source guide

The inline citations support the individual claims. These links are the most useful starting points for implementation and review.

**Institutional mechanics and revisions:** Paradox’s [Victoria 3 slavery diary](https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-15-slavery.1490983/?utm_source=chatgpt.com), [Update 1.2 changelog](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-78-update-1-2-changelog?utm_source=chatgpt.com), and [1.12 developer patch notes, mirrored by SteamDB](https://steamdb.info/patchnotes/21114983/). Together they show original intent, institutional-memory corrections, and expanded self-emancipation mechanics.

**Strategy-game incentives:** [CK3 Raid Intents developer diary, mirrored by DevTracker](https://devtrackers.gg/crusader-kings/p/fc9ebefa-dev-diary-165-tributaries-confederations), [EU4 1.30 developer changelog mirror](https://gameupdatenotifier.com/g/europa-universalis-iv/v/1-30?utm_source=chatgpt.com), the [Civilization IV manual](https://cdn.akamai.steamstatic.com/steam/apps/3900/manuals/manual.pdf), and [CivFanatics’ production analysis](https://civfanatics.com/civ4/strategy/empire-management/ways-into-production/). Read the last as expert community strategy, not developer ethical intent.

**Emergent storytelling:** Tynan Sylvester’s [GDC 2017 RimWorld talk](https://gdcvault.com/browse/gdc-17/play/1024232?utm_source=chatgpt.com), the [official Ideology description](https://rimworldgame.com/ideology/), [RimWorld’s documented slavery mechanics](https://rimworldwiki.com/wiki/Slavery), [Dwarf Fortress Legends documentation](https://dwarffortresswiki.org/index.php/Legends), and [Bay 12’s developer log](https://www.bay12games.com/dwarves/).

**Reception and interpretation:** [Hartmann and Vorderer on moral disengagement](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1460-2466.2009.01459.x?utm_source=chatgpt.com), [Oliver and colleagues on meaningful entertainment](https://digitalcommons.butler.edu/ccom_papers/145/), the [content-warning meta-analysis](https://doi.org/10.1177/21677026231186625?utm_source=chatgpt.com), and [McCall’s historical-problem-space framework](https://gamestudies.org/2003/articles/mccall).

**Controversies and evidential caution:** The [Cree representation controversy](https://www.pcgamesn.com/civilization-vi/civ-6-poundmaker-cree-nation) concerns the game’s governing framework as well as consultation. The [mermaid-farm retrospective](https://www.gamesradar.com/as-dwarf-fortress-heads-to-steam-players-remember-the-worst-thing-its-community-ever-did/) is useful for separating verified mechanics from community legend.

**Ratings and educational boundaries:** [ESRB’s ratings process](https://www.esrb.org/ratings/ratings-process/?utm_source=chatgpt.com), [PEGI’s criteria](https://pegi.info/what-do-the-labels-mean?utm_source=chatgpt.com), the [Australian RimWorld review decision](https://www.classification.gov.au/about-us/media-and-news/media-releases/rimworld-classified-r-18?utm_source=chatgpt.com), and [USHMM’s teaching guidelines](https://www.ushmm.org/teach/fundamentals/guidelines-for-teaching-the-holocaust).

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92844-88ec-83ea-b495-5a6eb7e38d67)
