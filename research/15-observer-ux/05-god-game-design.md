# Designing indirect god-game interventions for TCE

## Recommendation

**Give the player reliable control over an intervention’s immediate effect, but not over people’s interpretation, decisions, or eventual institutions.**

A rain miracle should reliably deliver the promised water. It should not guarantee a harvest, gratitude, population growth, or political stability. A whisper should reliably enter someone’s awareness. It should not guarantee belief, obedience, or even the response the player intended.

For TCE, the strongest design is therefore **a small set of powerful interventions that enter the ordinary simulation**, rather than a large catalogue of buttons that set social outcomes. The satisfying loop should be:

**Understand a situation → change a relevant cause → observe different responses → follow the consequences.**

The precedents support different parts of this approach. *Black & White* demonstrates attachment to a being that learns; *Populous* makes environmental influence immediately legible; *WorldBox* supports open-ended experimentation; *Reus* makes provision politically consequential; *From Dust* exposes the importance of dependable autonomous execution; and *RimWorld* makes different kinds of dramatic pacing explicit. None should be copied wholesale as an autonomy model. [Game Developer](https://www.gamedeveloper.com/design/postmortem-lionhead-studios-i-black-white-i-)

The report distinguishes **documented precedent**, **qualitative reception**, and **proposed TCE rules**. The proposed balance values are prototype hypotheses, not empirically established genre constants.

---

## 1. Define autonomy before designing the tools

There are two separate design goals:

**Player agency:** the player can understand available actions and bring about meaningful consequences.

**Agent autonomy:** a person’s action is selected through that person’s normal decision process, using their needs, knowledge, relationships, values, capabilities, and institutional constraints.

These are compatible. Agency research usefully treats player agency as a relationship between what players want to do and what a computational system actually supports—not simply the number of available buttons. For TCE, the interface must establish the right expectation: *influence a society*, rather than *issue commands through decorative supernatural language*. [EIS](https://eis.ucsc.edu/papers/nwf-C7-digra09-agency.pdf)

I recommend this operational boundary:

| Intervention | Preserves TCE’s decision autonomy? | Reason |
| --- | --- | --- |
| Create rain over a watershed | Yes | Changes physical circumstances; people still choose how to respond. |
| Draw a farmer’s attention to spoiled grain | Yes | Adds a percept or concern to ordinary deliberation. |
| Reveal evidence of an official’s theft | Yes | Changes information; audiences can believe, investigate, exploit, or dismiss it. |
| Make a ruler “consider reconciliation” | Conditionally | Acceptable as an idea entering deliberation; unacceptable as a hidden command to select peace. |
| Add enough invisible utility to guarantee peace | No | The normal decision process remains cosmetically present but cannot affect the result. |
| Set a kingdom’s war state, religion, law, or government type | No | Directly writes the outcome that autonomous actors should produce. |

**Autonomy does not require arbitrary refusal.** A person who consistently accepts convincing evidence can be autonomous. A person who randomly refuses one command in five is not necessarily autonomous.

Nor does autonomy mean freedom from coercion. Destroying a settlement can drastically restrict its inhabitants’ options. The honest distinction is that the player has changed their circumstances, not secretly selected their decisions.

---

## 2. What the six precedents actually teach

### Black & White: influence becomes meaningful through a relationship

**How it works.** Its distinctive mechanism is the Creature: a persistent being influenced through demonstration, reward, and punishment, rather than exclusively through commands. Molyneux’s postmortem describes learning through reinforcement and imitation, and explicitly says the team avoided using randomness merely to make the Creature appear independent. The same account explains that village coordination was partly centralized in a Village Center to reduce computation. The celebrated Creature should therefore not be confused with an entire population of equally sophisticated learners. [Game Developer](https://www.gamedeveloper.com/design/postmortem-lionhead-studios-i-black-white-i-)

**What worked.** The important payoff was not just task automation. It was recognizing something of one’s own teaching in later autonomous behavior. Rick Lane’s retrospective describes attachment to the Creature and the satisfaction of teaching it useful activities, including resource delivery and rain miracles. [Wayback Machine](https://web.archive.org/web/20160816050101/http%3A//www.eurogamer.net/articles/2015-07-26-black-and-white-combined-the-sublime-with-the-stupid)

**What failed.** The same retrospective criticizes persistent villager demands and campaign events that remove or undermine the Creature, including its abduction and a later curse. These are two different failures: repetitive service work makes godhood feel administrative; scripted interference with a nurtured relationship makes previous investment feel disposable. This is a critic’s retrospective assessment, not a representative player survey. [Wayback Machine](https://web.archive.org/web/20160816050101/http%3A//www.eurogamer.net/articles/2015-07-26-black-and-white-combined-the-sublime-with-the-stupid)

**For TCE:** adopt persistent relationships and recognizable influence; avoid training humans as pets. A person should remember a warning, reinterpret it, tell others about it, and perhaps teach a practice to their children. Do not reward that investment by automatically kidnapping the favorite person or destroying their achievements to manufacture drama.

The especially useful adaptation is **influence that survives the original recipient**: a farming practice, a family obligation, a disputed religious account, or an institution that descendants maintain for their own reasons.

### Populous: changing the environment can be a complete control language

**How it works.** The original *Populous* couples terrain alteration to settlement and population growth, which supports greater divine power and attacks against an opposing deity’s followers. It also offers coarse behavioral controls; it is not a demonstration of unrestricted individual human autonomy. Its fundamental interaction is environmental: make land usable, and followers exploit it. [GOG.COM](https://www.gog.com/en/game/populous)

Molyneux’s GDC retrospective gives a revealing origin story: difficulty making people navigate the landscape led to letting the player raise and lower it. Rather than merely improving command execution, the design gave players a way to change the problem. [GameSpot](https://www.gamespot.com/articles/molyneux-on-building-populous/1100-6302263/)

**What worked and failed.** The environmental feedback loop is concrete and visible. However, the GOG community reviews illustrate two readings of that simplicity: some praise the original indirect-control premise, while another finds the repeated land-flattening and destruction formula shallow. Those accounts support a design tension, not a consensus that repetition necessarily ruined the game. [GOG.COM](https://www.gog.com/en/game/populous)

**For TCE:** adopt “create an opportunity and watch people exploit it.” Avoid making the player the world’s routine earthworks contractor.

Opening a viable pass, restoring a spring, or exposing useful stone can create durable strategic possibilities. Flattening every future building plot would turn an autonomous civilization into a construction queue disguised as a god game.

### WorldBox: observation and experimentation work—but some powers bypass autonomy

**How it works.** *WorldBox* combines world creation, environmental and destructive powers, and civilizations that develop and interact without continuous player instruction. This is particularly relevant to TCE’s observer-led play. [Super World Box](https://www.superworldbox.com/)

Its tools nevertheless occupy different positions on the autonomy spectrum. The documented 0.21-era change to **Spite** made a targeted kingdom wage persistent total war against the other kingdoms; **Friendship** could stop that behavior. These are direct diplomatic interventions, not merely changes to the information or incentives available to leaders. The same update introduced plots and additional war information, including duration and casualties—useful precedents for exposing processes rather than just map-color changes. [Super World Box](https://www.superworldbox.com/changelog)

**Reception exposes a genuine audience split.** In a May 2024 Steam discussion, a player explicitly wanted wars to continue longer because peaceful aftermaths were uninteresting; replies discussed using powers and settings to prevent peace. This is evidence that some players want an editable spectacle, not evidence that all simulation observers want permanent conflict. [Steam Community](https://steamcommunity.com/app/1206560/discussions/0/4365753515843767565/?l=turkish)

**For TCE:** adopt the ease of setting up situations, watching them unfold, and inspecting political history. Do not assume that a power called a *whisper* preserves autonomy.

A TCE “whisper of war” should introduce a threat interpretation, grievance, or strategic possibility. A cautious ruler might investigate, fortify, negotiate, suppress the rumor, or reject it. The interface should never promise “start a war” unless the product deliberately offers a separate outcome-editing mode.

### Reus: provision can create consequences beyond simple gratitude

**How it works.** *Reus* places four giants under the player’s control while leaving humanity outside direct control. Terrain, plants, animals, minerals, and their synergies shape development. Its central complication is that generous provision can produce greed rather than uncomplicated appreciation. [Abbey Games](https://abbeygames.com/game/reus/)

**Developer evidence.** Abbey’s June 2013 retrospective identifies giants, greed, and placement as the mechanisms that worked during development. Symbioses, aspects, projects, and additional content were then developed around them. The designer also acknowledged that some entities needed more life. This is an important distinction for TCE: a compelling resource-placement puzzle does not automatically constitute a convincing society simulation. [Reus](https://www.reusgame.com/news/)

A June 20, 2013 development update documents a concrete balance repair: greed should decrease when a village is not growing too quickly even while danger influences it. That correction matters because a negative state needs an intelligible recovery path. [Reus](https://www.reusgame.com/news/)

**Reception.** A 2017 Steam discussion titled “greed is out of control” shows players struggling with the interaction between rapid development, danger, and greed. Responses include slowing provision and using coercive countermeasures. Some community explanations are version-sensitive, so they should not be treated as an authoritative current formula. [Steam Community](https://steamcommunity.com/app/222730/discussions/0/1326718197224204442/)

**For TCE:** adopt unintended consequences, but reject **“prosperity automatically causes aggression”** as a universal social law.

Instead, let additional resources operate through actual mechanisms: who owns them, whether they can be stored, who supplies labor, whether taxes rise, and whether leaders can mobilize the surplus. A successful harvest may produce investment, leisure, tribute, migration, inequality, or conflict. It should not be a disguised countdown to punishment.

### From Dust: indirect control needs exceptionally dependable execution

**How it works.** The player manipulates terrain and natural materials to make survival and movement possible for a tribe. Its campaign and timed challenges place this manipulation within objectives rather than offering only an unrestricted long-term society sandbox. Ubisoft’s description explicitly distinguishes the story territories from additional time-sensitive challenge maps. [Steam Store](https://store.steampowered.com/app/33460/From_Dust/)

**Developer evidence.** In *The Core of From Dust*, Eric Chahi discusses making intentions and paths sufficiently predictable for players to anticipate the tribe’s behavior. Replanning and navigation were not peripheral implementation problems: they directly affected whether indirect control felt understandable. He also describes the importance of exposing the environmental simulation through direct manipulation. [Game Developer](https://www.gamedeveloper.com/design/the-core-of-i-from-dust-i-)

**What worked and failed.** Tom Francis’s PC Gamer review praises the landscape simulation and the pleasure of protecting settlements, but criticizes villagers losing track of their tasks. Because players cannot directly correct them, failures become especially aggravating under recurring disaster deadlines. The review also describes uncertainty over whether an apparent failure came from the player’s mistake or the game’s usability problems. [PC Gamer](https://www.pcgamer.com/from-dust-review/)

**For TCE:** distinguish three states in the UI:

* **Cannot act:** the route, materials, authority, or physical capacity is missing.
* **Has not acted yet:** the person is waiting, planning, coordinating, or occupied.
* **Chooses otherwise:** another obligation, preference, or belief wins.

These must not collapse into one ambiguous “ignored your intervention” outcome.

Most importantly, do not describe pathfinding bugs or broken job reservations as evidence of agent free will.

### RimWorld’s storytellers: separate preferred drama from simulation causation

**How it works.** RimWorld explicitly frames its storyteller as a selector of events such as raids, storms, and traders. Its official description characterizes Cassandra through rising tension, Phoebe through a more relaxed approach, and Randy through unpredictability. This is a mechanism for shaping the events confronting characters—not a model of how a player should communicate with autonomous people. [RimWorld](https://rimworldgame.com/)

**What worked.** Sylvester’s GDC talk describes how treating RimWorld as a story generator changed feature selection and design priorities. More concretely, the 1.2 update separated numerous playstyle parameters: players could alter threat scaling, production yields, and other settings rather than accepting one bundled interpretation of difficulty. [GDC Vault](https://www.gdcvault.com/play/1024232/-RimWorld-Contrarian-Ridiculous-and)

**What can fail for TCE.** Community criticism of wealth-based threats illustrates a particular problem: some players describe discarding wealth or avoiding decoration to manage an invisible danger calculation. Ludeon also provides a wealth-independent option, with a configurable time progression. This does not establish that wealth scaling is universally bad; it shows that an effective challenge-balancing mechanism can conflict with a player’s preferred fiction. [Reddit](https://www.reddit.com/r/RimWorld/comments/1hrd2ro/after_switching_to_wealth_independent_mode_i_can/)

**For TCE:** borrow configurable intensity and the importance of quiet intervals. Do not automatically import a hidden incident director that creates enemies because the society is prospering.

The default historical simulation should generate events from its own causes. A separate observer system can identify interesting events and bring them to the player’s attention. **It should usually select what to show, not what must happen.**

---

## 3. Numbers: what is documented, and what it does not establish

| Precedent | Documented figure | Relevance to TCE |
| --- | --- | --- |
| **Populous** | **500 worlds** in the official storefront description. | Extensive scenario variation can reuse a compact control vocabulary; this is not 500 independently simulated societies. [GOG.COM](https://www.gog.com/en/game/populous) |
| **Reus** | **4 giants** and **over 100** plants, animals, and minerals. | A small number of intervention channels can produce variety through combinations and context. [Abbey Games](https://abbeygames.com/game/reus/) |
| **Reus** | An official challenge used a **30-minute era**. | Its documented short-session balance should not be transferred directly to TCE’s centuries-long worlds. [Reus](https://www.reusgame.com/news/) |
| **From Dust** | **13** story territories and **30** additional challenge maps. | Much of its pacing evidence comes from bounded scenarios and timed challenges. [Steam Store](https://store.steampowered.com/app/33460/From_Dust/) |
| **From Dust technology** | The GDC technical description specifies **128-bit SIMD**, multithreading, and a PS3 implementation running the simulation on SPUs. | Evidence of specialized environmental-simulation engineering—not a benchmark for tens of thousands of deliberating humans. [GDC Vault](https://gdcvault.com/play/1013667/Creating-a-High-Performance-Simulation) |
| **RimWorld 1.2** | The developer describes **a couple dozen** custom playstyle settings. | Supports separating intensity, lethality, and economic generosity instead of exposing one opaque difficulty number. [Ludeon Studios](https://ludeon.com/blog/2020/08/1-2-update-with-new-quests-psycasts-gear-and-more/) |

For broad reception, the retrieved Steam snapshots showed **96% positive across 29,726 English-language reviews for WorldBox**, and **97% across 119,577 for RimWorld**. These are substantial product-level approval signals, but they cannot establish which intervention or pacing mechanism caused that approval. Reviews are self-selected and evaluate the whole product. [Steam Store](https://store.steampowered.com/app/1206560/?utm_campaign=get_steam_button&utm_medium=banner&utm_source=website)

**Evidence gap:** I found no directly comparable published benchmark for these intervention systems operating over 10k–50k individually deliberating people, and no controlled study establishing an optimal universal miracle frequency. Exact TCE cooldowns would therefore be design hypotheses, not researched constants.

---

## 4. Recommended intervention design for TCE

The remainder is a proposed design, not a description of an existing game.

### 4.1 Use three channels, with different promises

| Channel | What the player controls | What remains autonomous | Example |
| --- | --- | --- | --- |
| **Whisper / inspiration** | Recipient, topic, and the idea or concern brought to attention. | Credibility, interpretation, investigation, priority, action, and communication. | “Could winter stores be protected from damp?” |
| **Omen / public sign** | A perceptible event, its location, timing, and observable features. | Who notices, how accounts spread, what it means, and who claims authority from it. | An unusual light appears above a flood-threatened settlement. |
| **Miracle / environmental change** | A precisely described physical effect and footprint. | Resource use, ownership disputes, labor, migration, gratitude, and institutional response. | Rain replenishes soil moisture over selected fields. |

These channels should have different failure semantics.

A whisper can be **heard but rejected**. An omen can be **observed but disputed**. A miracle should not randomly fail after the interface has promised a definite physical result.

This gives the player something solid to trust without guaranteeing the larger outcome.

### 4.2 Route every social effect through ordinary cognition

The processing path should be:

**Intervention → perception → memory/belief update → consideration of alternatives → ordinary decision → action → social consequences.**

A whisper may add a hypothesis to consideration or increase attention to a relevant concern. It should not directly replace values, set loyalty, install a goal with overriding priority, or write a government decision.

A useful engineering restriction is to let god tools submit only:

**Physical changes:** environmental fields, resources, objects, injuries, or other explicitly permitted world effects.

**Perceptible events:** recipient or witness scope, content, modality, apparent source, and an event identifier.

Do not give the same interface access to operations equivalent to `set_religion`, `declare_war`, `pass_law`, or `choose_action`.

An institution must also preserve this boundary. Persuading one officeholder should not bypass a council vote, legal procedure, coalition, budget constraint, or subordinate’s refusal.

### 4.3 Make information an opportunity, not a technology unlock

For technological inspiration, the intervention should make a nearby possibility worth investigating.

A potter might become curious about a different firing arrangement. Whether that produces an innovation still depends on materials, prior knowledge, equipment, experiments, labor, and whether anyone adopts the result.

This fits TCE’s authored building blocks: the intervention can activate **search within a feasible neighborhood of the technology graph**, not grant an arbitrary node.

Similarly, architectural inspiration should influence what a patron, builder, or community considers. It should not place a completed blueprint in everyone’s mind or force construction regardless of resources and permissions.

### 4.4 Keep interpretation plural

An omen should not carry a globally authoritative tooltip into every agent’s mind.

Track at least three different propositions:

**Occurrence:** “A strange light appeared.”

**Attribution:** “A divine being caused it.”

**Prescription:** “Therefore we must abandon this settlement.”

Different people can accept different subsets. A priest might accept all three; a farmer might accept the first two but oppose relocation; a rival religious specialist might offer a different prescription.

Crucially, the player’s intended meaning is not automatically an agent belief. It enters the world only through what was actually communicated.

This creates meaningful divine influence without requiring a universal “faith” number or a single good–evil judgment shared by the whole population.

### 4.5 Stop repetition from becoming a hidden command

A probabilistic whisper becomes guaranteed obedience when the player can click it indefinitely.

Use a stable decision episode rather than a new independent acceptance roll on each click. Repeating the same proposition should generally refresh awareness, produce annoyance, or add little—not repeatedly reroll the person’s position.

Recommended protections:

**One active influence record per recipient, proposition, and decision episode.** Repetition does not stack unlimited bonuses.

**Correlated evidence remains correlated.** Five rumors originating from the same omen are not five independent confirmations.

**New context can matter.** A rejected warning can become persuasive after a crop failure, new testimony, or a change in family circumstances.

These rules preserve the possibility of sustained influence without converting patience into mind control.

### 4.6 Let material success be genuinely good

Avoid mandatory compensating consequences.

Rain need not create a new disaster. A saved child need not cause an equivalent death elsewhere. A prosperous town need not become aggressive because prosperity has exceeded a designer’s threshold.

Consequences should arise from the changed state. The saved child may later become a ruler—or remain an ordinary person. Better harvests may support public works—or be appropriated by landlords. Sometimes the outcome should simply be improvement.

Otherwise the player learns that helping is an elaborate way to trigger punishment, rather than an opportunity to understand the world.

### 4.7 Design for long-term influence, not repeated maintenance

A strong TCE intervention should often change the conditions under which people can sustain themselves.

Prefer revealing a spring over refilling every household’s water container; drawing attention to storage losses over repeatedly spawning food; enabling experimentation over annually refreshing a productivity blessing.

Temporary miracles remain useful for exceptional rescue. They should not become routine requirements for a functioning society.

**The target number of god actions required for ordinary daily maintenance should be zero.**

---

## 5. Frequency, consequences, and player reception

### Balance intervention size before intervention count

A button press is a poor unit of balance. One local warning and one continent-wide climatic alteration should not consume equivalent “actions.”

Represent intervention magnitude through:

**Reach:** people, area, or institutions directly exposed.

**Intensity:** how far the effect departs from existing conditions.

**Duration:** how long the direct effect persists.

**Irreversibility:** whether the intervention destroys options or creates permanent assets.

**Propagation:** whether it can plausibly spread through communication or institutions.

The last dimension should inform forecasts and testing, not necessarily the price. Charging an exact amount for predicted historical impact would imply foresight the simulation does not possess.

### Separate three kinds of pacing

**Physical time:** rain, injury, travel, erosion, and crop growth.

**Decision time:** the next deliberation, harvest choice, council meeting, or succession.

**Player attention time:** how frequently the observer has an interesting decision to make.

Do not solve all three with a single cooldown.

A memory’s salience can decay over relevant decision cycles. A physical effect follows world time. Interface prompts can be governed by an attention budget. Changing simulation speed should not silently change agent susceptibility or produce a mana-farming exploit.

### Start with explicit experimental ranges

These are **proposed prototype settings**, not final recommended constants:

| Parameter | Initial experiment | What to measure |
| --- | --- | --- |
| Whisper scope | **1 named recipient** | Whether individual selection creates understanding and attachment. |
| Public-sign scope | **25–100 direct witnesses**, selected spatially | Whether subsequent spread comes from social communication rather than magical broadcast. |
| Salience half-life | **1–3 relevant decision cycles** | Whether the idea remains useful without permanently dominating attention. Historical memory can last much longer. |
| Repeat stacking | **1 active record** per recipient/proposition/episode | Whether spam can force a result. |
| Tracked intervention threads | **3–5 prominently visible** | Whether players can follow consequences without losing context. |
| Frequency test arms | **4**, **12**, or unrestricted significant interventions per **30 minutes of active play** | Whether constraints improve anticipation or merely create waiting. These are study conditions, not a proposed recharge rule. |
| Routine maintenance | **0 required god actions** | Whether settlements remain genuinely autonomous. |

Start with both an unrestricted testing configuration and a constrained “stewardship” configuration. The former reveals exploits and expressive possibilities; the latter tests whether scarcity makes selection meaningful.

Do not bind power directly to population or worship in the first prototype. That would introduce a second optimization game—maximizing followers to buy more influence—before establishing whether influencing people is enjoyable in itself.

### Offer intensity preferences without secretly rewriting causality

The precedents do not point to one audience. The WorldBox discussion favors extended war; RimWorld’s wealth-scaling criticism favors a world that does not punish ordinary accumulation through hidden accounting. Both preferences are legitimate. [Steam Community](https://steamcommunity.com/app/1206560/discussions/0/4365753515843767565/?l=turkish)

For TCE, expose separate choices for intervention availability, destructive power access, notification intensity, and simulation speed. Keep the underlying historical rules visible and stable.

A dramatic variant can be offered later, but it should say clearly when it introduces external events for pacing. It should not be indistinguishable from the baseline emergent history.

---

## 6. Make consequences readable enough that refusal remains enjoyable

### Show promises and uncertainties separately

Before an intervention, the inspector should distinguish:

**Certain direct effect:** what the tool will actually change.

**Current situation:** who can perceive it and which relevant constraints are known.

**Plausible responses:** several outcomes consistent with the current state.

**Unknowns:** missing information and long causal chains.

For example:

> **Rain over the western fields**  
> Direct effect: adds the specified rainfall within the selected area.  
> Likely immediate benefit: reduced moisture stress where crops remain viable.  
> Important constraints: labor shortages and disputed irrigation access remain.  
> Social response: attribution and resource allocation are not guaranteed.

Do not display an exact “chance of founding a republic” unless it comes from a real, appropriately qualified predictive procedure.

### Expose a chain of observable milestones

After a whisper, report whether it was perceived, considered, investigated, acted upon, or communicated—not just whether the intended final outcome occurred.

A useful record might say:

> Mara considered communal grain storage. She rejected immediate construction because her household cannot spare labor. She discussed the idea with her brother, who raised it at the next village meeting.

This is a meaningful outcome even without a granary.

An unhelpful record would say:

> Whisper failed.

### Preserve provenance without overclaiming causation

Attach intervention identifiers to relevant perceptions, decisions, and resulting events. Store the actual reasons used by decision systems.

However, “this intervention was one recorded influence” is not equivalent to “this intervention caused the entire outcome.” The chronicle should distinguish direct effects, explicit agent attribution, and broader inferred consequences.

The observer system can summarize genuine records. It should not invent a convincing divine causal story after the fact.

### Follow continuity beyond favorite individuals

Allow the player to follow a person, family, settlement, practice, institution, or disputed account of a miracle.

When a recipient dies, the interface should reveal whether the influence disappeared, persisted as a memory, became a custom, or was appropriated by someone else. This is particularly important for TCE’s centuries-long timescale: the unit of attachment cannot remain only the original individual.

---

## 7. Implementation and validation at TCE’s scale

### Keep intervention processing local and event-driven

The Rust kernel should own both interventions and decisions. Unreal should display effects and submit requests, not independently apply gameplay consequences.

A local miracle should update affected environmental cells and notify relevant systems. A whisper should target an individual. An omen should discover witnesses through spatial queries; later spread should follow actual communication.

Do not trigger a fresh global deliberation pass over 50,000 people whenever the player clicks.

Bounded intervention records are affordable in principle, but that is not a full memory-system benchmark. For illustration, **50,000 people × 8 compact records × 32 bytes = 12.8 MB** of raw record storage, before indexes, allocator overhead, shared content, and retained history. The long-term archive needs a separate retention policy.

### Validate autonomy and usefulness separately

| Test | Passing condition |
| --- | --- |
| **No hidden command path** | God tools cannot directly write selected actions, laws, diplomatic outcomes, or allegiance. |
| **Counterfactual usefulness** | Paired simulations show that interventions can materially change relevant outcomes. |
| **Context sensitivity** | Different needs, beliefs, relationships, and institutions produce explainable differences in response. |
| **Anti-spam** | Repeated unchanged input does not converge to guaranteed obedience through rerolls or unlimited stacking. |
| **Competence** | Navigation and task failures are identified as implementation failures, not narrated as choice. |
| **Long-term persistence** | Some effects can survive through ordinary social mechanisms after the direct effect ends. |
| **Player comprehension** | A proposed initial target is **80% correct identification of the main response reason** in structured playtests. |
| **Performance** | Benchmark separately at **10k, 25k, and 50k** people, including mass-witness events and communication bursts. |

Use paired seeded runs for counterfactual testing, but avoid interpreting every divergence as a clean estimate of effect. Random-number consumption and cascading interactions can complicate comparisons. Repeat across multiple starting states, and inspect mechanisms alongside aggregate outcomes.

The aim is not a fixed obedience percentage. It is **substantial, understandable influence without bypassing the actors**.

### Build one complete causal chain before a large power catalogue

A strong first vertical slice would contain:

**One whisper:** concern about winter grain losses.

**One public omen:** a visible sign around an already developing environmental danger.

**One miracle:** a local, quantitatively specified rainfall event.

Test these against households with different labor supplies, beliefs, property rights, and political relationships. Include both cooperative and coercive institutions.

The milestone is not “three powers implemented.” It is that a player can observe:

> “The rain helped both villages. One expanded communal storage; the other’s landlord took the surplus. My intervention mattered, but their societies determined what it became.”

That is the distinctive promise TCE should deliver.

---

## 8. Source guide

The links below provide the most useful starting points for deeper reading and viewing. Developer claims describe intent or implementation; reviews and discussions provide qualitative reception, not controlled evidence.

| Topic | Primary material | Critique or supporting reference |
| --- | --- | --- |
| **Agency** | [Wardrip-Fruin, Mateas, Dow, and Sali: *Agency Reconsidered*](https://dl.digra.org/index.php/dl/article/view/369?utm_source=chatgpt.com) | Useful conceptual distinction between meaningful action and merely available controls. |
| **Black & White** | [Peter Molyneux’s 2001 postmortem](https://www.gamedeveloper.com/design/postmortem-lionhead-studios-i-black-white-i-?utm_source=chatgpt.com) | [Rick Lane’s retrospective, archived](https://web.archive.org/web/20160816050101/http://www.eurogamer.net/articles/2015-07-26-black-and-white-combined-the-sublime-with-the-stupid); [wiki and source bibliography](https://en.wikipedia.org/wiki/Black_%26_White_%28video_game%29?utm_source=chatgpt.com). |
| **Populous** | [GDC 2011: *Classic Game Postmortem—Populous*](https://www.gdcvault.com/play/1014633/Classic-Game-Postmortem?utm_source=chatgpt.com) | [Contemporaneous coverage of the developer talk](https://www.gamespot.com/articles/molyneux-on-building-populous/1100-6302263/?utm_source=chatgpt.com); [GOG description and player reviews](https://www.gog.com/en/game/populous?utm_source=chatgpt.com). |
| **WorldBox** | [Official game description](https://www.superworldbox.com/?utm_source=chatgpt.com); [versioned changelog](https://www.superworldbox.com/changelog?utm_source=chatgpt.com) | [Player discussion about extending wars](https://steamcommunity.com/app/1206560/discussions/0/4365753515843767565/). |
| **Reus** | [Abbey’s feature description](https://abbeygames.com/game/reus/?utm_source=chatgpt.com); [June 2013 design retrospective and patch notes](https://www.reusgame.com/news/?utm_source=chatgpt.com) | [Greed discussion](https://steamcommunity.com/app/222730/discussions/0/1326718197224204442/?utm_source=chatgpt.com); [community mechanics guide](https://steamcommunity.com/sharedfiles/filedetails/?id=225178210&utm_source=chatgpt.com), which should be checked against the relevant version. |
| **From Dust** | [Developer interview: *The Core of From Dust*](https://www.gamedeveloper.com/design/the-core-of-i-from-dust-i-?utm_source=chatgpt.com); [GDC technical presentation](https://gdcvault.com/play/1013667/Creating-a-High-Performance-Simulation?utm_source=chatgpt.com) | [Tom Francis’s PC Gamer review](https://www.pcgamer.com/from-dust-review/?utm_source=chatgpt.com). |
| **RimWorld** | [Official storyteller description](https://rimworldgame.com/?utm_source=chatgpt.com); [Sylvester’s GDC 2017 talk](https://www.gdcvault.com/play/1024232/-RimWorld-Contrarian-Ridiculous-and?utm_source=chatgpt.com); [1.2 customization announcement](https://ludeon.com/blog/2020/08/1-2-update-with-new-quests-psycasts-gear-and-more/?utm_source=chatgpt.com) | [Community criticism of wealth-based pacing](https://www.reddit.com/r/RimWorld/comments/1hrd2ro/after_switching_to_wealth_independent_mode_i_can/?utm_source=chatgpt.com). |

**Bottom line:** TCE’s godhood should be powerful enough to alter history, constrained enough that people still author their own responses, and legible enough that the player can appreciate the difference. The signature experience is not *“they obeyed me.”* It is *“I changed what was possible—and they made something of it.”*

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9299e-70d4-83e9-aae0-195b35876ed9)
