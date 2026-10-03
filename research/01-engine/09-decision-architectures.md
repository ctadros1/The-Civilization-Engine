# Decision architectures for TCE

## Executive recommendation

**Use event-driven utility selection plus persistent jobs for ordinary citizens, and utility-selected goals plus a bounded HTN planner for notables. Both tiers should share the same action definitions, beliefs, execution system, resource reservations, and explanation records.**

Do not give every citizen a general-purpose planner, and do not make the notable planner a separate simulation that can directly command other people’s decisions. A notable should initiate requests, contracts, proposals, and commitments; other agents should accept, refuse, negotiate, or ignore them.

For TCE, I would divide responsibility this way:

| Responsibility | Recommended mechanism |
| --- | --- |
| What matters now? | Utility scoring, constrained by emergencies and existing commitments |
| What ordinary activity should I undertake? | Utility selection among a bounded set of available jobs and interactions |
| How do I carry out that activity? | Persistent, interruptible job state machine; small behavior-tree fragments where helpful |
| How do I achieve a multi-step political or economic objective? | Bounded, total-order HTN planning over authored methods |
| Can I improvise a previously unauthored action sequence? | Optional, narrowly scoped GOAP search—not a requirement for the first implementation |

The main scaling opportunity is **avoiding unnecessary decisions and expensive candidate queries**, rather than finding a universally fastest selector. That conclusion is consistent with F.E.A.R.’s emphasis on cached sensing, Unreal’s event-driven behavior trees, and Project Highrise’s separation of action selection from execution. [AAAI Publications](https://ojs.aaai.org/index.php/AIIDE/article/download/18724/18501)

**Evidence scope:** This report uses sources available as of **September 27, 2026**. Historical game implementations are identified by their documented version or publication date. Proposed TCE budgets below are engineering starting points, not measured performance claims.

---

# 1. Options: what each technique actually provides

These techniques overlap, but they do not answer exactly the same question. Utility usually selects a desirable action or goal; planning constructs a sequence; a behavior tree or state machine controls execution.

## Utility AI and Infinite Axis Utility

Utility AI evaluates available choices using contextual scores: hunger relief, expected earnings, travel burden, danger, social obligations, personality, and so forth. It then selects the highest-scoring choice or samples among sufficiently attractive alternatives.

The **Infinite Axis Utility System–style implementation documented for Guild Wars 2: Heart of Thorns** associates each decision with a decision-score evaluator. Individual considerations pass through response curves, and their outputs are multiplied. A zero consideration rejects the choice, allowing cheap checks to eliminate expensive evaluations early. “Infinite axis” concerns extensible considerations—not an infinite search space or long-horizon planning. [GameAIPRO](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter13_Choosing_Effective_Utility-Based_Considerations.pdf)

For TCE, utility is a good fit for choosing between activities such as harvesting, eating, visiting someone, accepting paid work, attending an assembly, or continuing the current job. It can include predicted consequences without searching through an entire action sequence.

Its weakness is that a useful intermediate action can look unattractive in isolation. Buying timber is costly now but enables a profitable workshop later. A job template or planner must supply that connection.

## Goal-Oriented Action Planning: GOAP

GOAP describes actions through **preconditions, effects, and costs**, then searches for a sequence that achieves a goal. A common implementation uses A\* over compact symbolic states. F.E.A.R. demonstrated how this could replace a large collection of hand-connected behavior transitions with reusable action building blocks. [GameDevs](https://www.gamedevs.org/uploads/three-states-plan-ai-of-fear.pdf)

For example:

> Need stored grain → obtain grain → acquire containers → transport grain → deposit it.

The main attraction is compositional flexibility: the designer need not author every permitted sequence. The main danger is the action model. Missing preconditions, inaccurate effects, or unrealistic costs can produce perfectly valid plans for an incorrect model.

GOAP also does not automatically decide which goal is worth pursuing. That still needs arbitration, often through priorities or utility.

## Hierarchical Task Networks: HTN

An HTN decomposes an abstract task through authored **methods** until it reaches executable primitives.

For example, “improve winter food security” might have methods for increasing production, negotiating imports, organizing household contributions, or proposing a communal reserve. Each method has applicability conditions and subordinate tasks.

Unlike an ordinary behavior tree, an HTN planner can apply hypothetical action effects while checking whether later tasks will be feasible. The practical total-order forward-decomposition approach described by Troy Humphreys was used in *Transformers: Fall of Cybertron*. [GameAIPRO](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter12_Exploring_HTN_Planners_through_Example.pdf)

HTN trades some unrestricted improvisation for explicit know-how. That is advantageous for a solo developer: institutional competence is easier to inspect as “these are the available ways to organize a reserve” than as an arbitrary search through every possible political action.

However, HTN is not automatically cheap or complete. Alternative methods, recursion, parameter binding, and backtracking can still create large searches.

## Behavior trees: BTs

Behavior trees compose selectors, sequences, conditions, and running actions. They work particularly well for procedural behavior:

> Reserve tool → walk to workshop → perform work → release tool → report completion.

They are not inherently planners: an ordinary sequence executes actions rather than proving the sequence against a simulated future state.

Nor must a BT restart from its root every frame. Unreal’s implementation is explicitly event-driven, with blackboard changes and observers triggering relevant transitions. Consequently, “BT means expensive per-frame polling” is not a fair comparison. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/behavior-tree-in-unreal-engine---overview)

For TCE, use BT ideas for reusable execution fragments, not one enormous tree encoding every aspect of citizenship and politics.

---

# 2. Trade-offs: performance, complexity, and evidence

## Comparative engineering costs

Here, \(A\) means the number of **bound action–target candidates**, not merely action types; \(K\) is considerations per candidate.

| Technique | Runtime characteristics | Authoring and maintenance cost | Best TCE role |
| --- | --- | --- | --- |
| **Utility** | Approximately \(O(AK)\) per reconsideration when inputs are bounded-cost lookups. Candidate generation can dominate. | Easy to add choices; difficult to keep scores, response curves, and durations comparable. | Citizen choices; notable goal selection |
| **GOAP** | Search cost depends on branching, depth, grounding, heuristic quality, and state representation. Failed searches and tail latency matter greatly. | Reusable operators, but demanding precondition/effect modeling and debugging of unintended plans. | Small improvisational subproblems |
| **HTN** | Usually constrains search through domain knowledge; alternative methods and backtracking can still be expensive. | More explicit procedural knowledge to author; good structural explanations and predictable competence boundaries. | Notable projects and institutional procedures |
| **BT / job state machine** | Cost follows visited conditions and active tasks; expensive leaves remain expensive regardless of the tree. | Straightforward initially; large trees accumulate duplicated branches and priority interactions. | Execution, interruption, recovery |

The runtime descriptions are conditional, not universal rankings. Orkin’s planning architecture and Jacopin’s optimization chapter both emphasize representation and supporting calculations: a cheap symbolic planner can become expensive when predicates invoke navigation, object searches, or allocation-heavy binding. [AAAI Publications](https://ojs.aaai.org/index.php/AIIDE/article/download/18724/18501)

Two authoring pitfalls deserve emphasis:

**Normalized scores are not automatically comparable.** A “0.8” for hunger satisfaction and a “0.8” for expected income need an explicit calibration convention. Likewise, multiplying more factors generally lowers a score: \(0.8^2=0.64\), whereas \(0.8^6\approx0.26\). Adding considerations can therefore change behavior even when every consideration looks favorable. The multiplication itself is documented in the Heart of Thorns implementation; the numerical consequence follows directly. [GameAIPRO](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter13_Choosing_Effective_Utility-Based_Considerations.pdf)

**A planner’s correctness is only relative to its model.** A plan that assumes requesting support guarantees support is not politically intelligent; it is exploiting a false effect.

## What published performance evidence establishes

| Primary source | Reported result | What it does **not** establish |
| --- | --- | --- |
| **Killzone 2 multiplayer-bot presentation, 2009** | Approximately **500 plans/s**, **8,000 decompositions/s**, and **24,000 branch evaluations/s** in the documented multiplayer workload. The slide lists 14 bots, up to 10 turrets, up to 6 drones, and squads. | These are workload counters, not a portable milliseconds-per-agent benchmark or evidence for 50,000 planners. |
| **Project Highrise, Game AI Pro 3, 2017** | Describes an implementation meeting its goal of **1,000 simulated/displayed NPCs at 60 fps** on contemporary desktop hardware. Its early propositional planner benefited strongly from plan caching. | A 2D management simulation is not a benchmark for 50,000 detailed UE characters. |
| **Building a Better Centaur, GDC 2015** | Reports modular utility-based content across hundreds of agent types and dozens of actions, with improved processing and authoring efficiency. | The presentation description does not provide a reproducible TCE-sized throughput benchmark. |

Sources: Guerrilla’s measured counters, Zubek’s implementation report, and Lewis/Mark’s talk description. [Guerrilla Games](https://www.guerrilla-games.com/media/News/Files/GAIC09_Killzone2Bots_StraatmanChampandard.pdf)

I did **not** find a public, apples-to-apples benchmark comparing these four architectures on a TCE-like workload and your processor class. The available evidence supports architectural choices, but not a defensible claim that “HTN costs X microseconds” or “utility supports Y citizens” independently of content.

## A useful TCE cost model

Estimate work in **simulation time**, not renderer frames:

\[
C\_{\text{day}}
=
N\_cD\_cc\_c + N\_nD\_nc\_n
\]

Here \(N\) is population, \(D\) is decisions or planning requests per simulated day, and \(c\) is measured CPU time per request.

An **illustrative sensitivity calculation**, not a benchmark:

| Assumption | Citizens | Notables |
| --- | --- | --- |
| Population | 50,000 | 300 |
| Requests per simulated day | 32 | 2 |
| Assumed cost per request | 2 μs | 0.5 ms |
| CPU time per simulated day | 3.2 seconds | 0.3 seconds |

That is **3.5 CPU-seconds per simulated day for decision work alone**. At ten simulated days per real second, it becomes 35 CPU-seconds per real second before routing, production, markets, and rendering.

The exact assumptions need measurement. The important conclusion is mathematical: **time-slicing smooths work but does not eliminate it**. Event-based scheduling can eliminate redundant work; simply spreading repeated polling across frames cannot. Graham’s event-simulation chapter makes this distinction explicitly. [GameAIPRO](https://www.gameaipro.com/GameAIProOnlineEdition2021/GameAIProOnlineEdition2021_Chapter02_Efficient_Event_Based_Simulations.pdf)

---

# 3. Precedents and what TCE should learn from them

## The Sims: expressive preferences and committed interactions

Graham’s utility chapter describes motive scoring, response curves, and action selection in the Sims family. It specifically notes that *The Sims Medieval* makes a new decision when its interaction queue is empty, continuing the current interaction until completion or failure. Richard Evans’s Sims 3 talk describes personality expressed through data-driven social interactions and production rules, supported by visualization tools. [GameAIPRO](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter09_An_Introduction_to_Utility_Theory.pdf)

**TCE lesson:** plausible daily life does not require continuous deep planning. Stable preferences, useful affordances, and completed interactions can provide much of the apparent intelligence. Invest in inspecting motives and candidate scores.

## RimWorld: job granularity is an AI design decision

The official **Alpha 16 release notes, December 2016**, explicitly identify batching cleaning and harvesting jobs as an improvement to both efficiency and sensible behavior. This is useful architectural evidence even without treating RimWorld as a pure example of utility, GOAP, or HTN. [Steam Store](https://store.steampowered.com/news/posts/?appids=294100&enddate=1495648434)

**TCE lesson:** “harvest this plot” or “deliver this batch” is often a better decision unit than repeatedly selecting the next individual plant or item. Do not confuse incident-generation AI with the mechanism that executes a person’s work.

## Dwarf Fortress: allocation and interruption matter as much as preference

Bay 12’s December 2014 development log describes replacing independent job-to-dwarf assignment with a job-posting/application process. The surrounding changes address priorities, interruption, and workers being diverted from useful ongoing work. These are documented historical changes around the 0.40.20 period, not a claim about every current subsystem. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2014.html)

**TCE lesson:** a population of individually sensible selectors still needs coherent allocation of scarce jobs, tools, workstations, and service capacity. Reservations, queues, and interruption rules are part of the decision architecture.

## F.E.A.R.: modular plans need supporting memory and sensing

F.E.A.R.’s architecture combined planning with a small execution state machine. Orkin’s accompanying paper emphasizes caching expensive sensing and navigation-related calculations in working memory rather than performing them repeatedly inside planning. [GameDevs](https://www.gamedevs.org/uploads/three-states-plan-ai-of-fear.pdf)

**TCE lesson:** preserve context across activity changes, and keep costly world queries outside the search loop. Do not port only the planner and omit the architecture that made it practical.

## Killzone and DECIMA: HTN remains a mature production approach

Killzone 2’s presentation documents explicit plan monitoring and “continue” branches that retain an existing plan rather than producing twitchy behavior under frequent replanning. Guerrilla’s **June 2026 DECIMA presentation** still describes HTN-based high-level decisions, backtracking, and in-game decomposition debugging. [Guerrilla Games](https://www.guerrilla-games.com/media/News/Files/GAIC09_Killzone2Bots_StraatmanChampandard.pdf)

**TCE lesson:** author not only how to start a plan, but why to continue it, when it becomes invalid, and what failed. Plan visualization is production tooling, not optional polish.

## Crusader Kings III: personality modifies competence rather than replacing it

The official **2022 AI diary associated with the 1.7 update** describes economic archetypes—Warlike, Cautious, Builder, and Unpredictable—alongside efforts to improve investment, domain consolidation, and diplomacy. The developers explicitly sought competent progress without universally optimal or out-of-character behavior. The same update’s memory and relationship-reason features exposed more of the historical context behind relationships. [Steam Store](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1662468472&feed=steam_community_announcements)

**TCE lesson:** provide competent procedures, then let personality and interests influence which are pursued. Preserve reasons and consequential memories. Do not infer from these diaries that CK3 uses a generic GOAP or HTN architecture throughout.

Two additional precedents are particularly relevant. **Project Highrise abandoned its fast prototype planner for authoring/control reasons**, illustrating that expressiveness can exceed what a design actually needs. **Comme il Faut/Prom Week separates an initiator’s desire from a responder’s acceptance**, an essential distinction for TCE’s politics. [GameAIPRO](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter34_1000_NPCs_at_60_FPS.pdf)

---

# 4. Recommended TCE design

The following is a proposed architecture, not a description of an existing engine.

## 4.1 Build one action system, not two incompatible AIs

Both tiers should operate through the same primitives.

| Shared element | Required information |
| --- | --- |
| **Action definition** | Stable identifier; target parameters; applicability; duration and costs; resource requirements; execution handler; modeled outcomes; explanation labels |
| **Running job** | Actor and targets; current step; start/deadline; reservations; progress; interruption policy; parent project |
| **Belief/context view** | Known opportunities, relationships, prices, risks, and institutional rules—with freshness and provenance |
| **Project** | Goal; chosen method; pending tasks; commitments; assumptions; review conditions |
| **Decision record** | Considered choices; score contributions; commitment effects; selected option; execution outcome |

Keep **predicted effects** separate from **committed world changes**. Search may predict that a completed delivery increases a stockpile. It must not add goods while considering the delivery.

Notables remain citizens. Hunger, family obligations, travel, and work still constrain them. Their planner produces projects and proposed activities; the ordinary activity arbitrator schedules those activities against other obligations.

Allow an ordinary citizen to receive temporary deliberative capacity when circumstances warrant it—for example, organizing an irrigation repair or initiating a faction. Treat “notable” primarily as a computational allocation policy, not a rule that only elites can originate consequential action.

## 4.2 Make citizen decisions event-driven and locally bounded

A citizen should reconsider when an action completes or fails, a relevant need threshold is crossed, an obligation becomes due, a material opportunity changes, or a danger requires interruption.

For slowly changing values, calculate the next relevant crossing rather than repeatedly decrementing every value and rescoring every activity. This follows the event-based approach described by Graham: derive state on demand where possible and schedule meaningful transitions. [GameAIPRO](https://www.gameaipro.com/GameAIProOnlineEdition2021/GameAIProOnlineEdition2021_Chapter02_Efficient_Event_Based_Simulations.pdf)

Start with a bounded candidate set—perhaps **8–24 action–target pairs**, each with **4–8 considerations**—and measure whether these limits omit important behavior. These are initial content budgets, not established optimal values.

Candidate generation should use local indexes:

* Household and workplace obligations.
* Nearby services and job postings.
* Known social contacts and institutional opportunities.
* Existing commitments and a small exploration allowance.

Do not enumerate every building, person, market, and law for every citizen. Keep social relationships sparse rather than maintaining a fully populated citizen-by-citizen matrix.

Use cached travel estimates while ranking candidates. Request an actual route after shortlisting or selection. A “cheap utility scorer” that performs twelve path searches is not cheap.

**Scarcity requires a commit stage.** Multiple agents may prefer the same remaining sack of grain or workstation. Parallel selection should produce proposals; allocation then validates and commits reservations. Losing agents should receive a meaningful failure reason and back off or choose an alternative—not retry the identical impossible choice every tick.

Local job boards can belong to households, enterprises, settlements, or factions. The allocation mechanism need not become an omniscient central government that decides everyone’s goals.

## 4.3 Use utility scores with explicit semantics

For ordinary actions, begin with an additive, inspectable score:

\[
U\_i(a)=
V\_{\text{need}}
+V\_{\text{material}}
+V\_{\text{social}}
+V\_{\text{obligation}}
-C\_{\text{time}}
-C\_{\text{risk}}
-C\_{\text{switch}}.
\]

This is a design convention, not a claim that human preferences are truly additive.

Separate **physical impossibility** from **undesirability**. A person without a boat cannot sail; a person forbidden to cross a border might still attempt it while accepting punishment risk. Hard-gating every illegal action would prevent rebellion, smuggling, and institutional change.

Use an explicit emergency rank before ordinary scoring when appropriate. Rank-then-weight selection has a documented precedent in dual-utility reasoning. It is easier to audit than hoping an arbitrarily large danger coefficient always defeats every combination of leisure benefits. [GameAIPRO](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter03_Dual-Utility_Reasoning.pdf)

Be especially careful with action duration. Score predicted consequences over a consistent horizon or use explicit duration/opportunity costs. Otherwise, repeated tiny rewards can defeat productive long activities. Dividing every score by duration is not a universal solution either: it can unfairly suppress delayed investments.

### Commitment, inertia, and hysteresis

Give each activity an interruption contract:

**Routine work** continues to a useful boundary. **Promises and projects** persist across individual jobs. **Emergencies** can interrupt immediately. **Failed or stale work** eventually times out.

A practical switching rule is:

\[
U(\text{new})-C\_{\text{switch}}
>
U(\text{continue})+H,
\]

where \(H\) is a hysteresis margin. Apply switching costs once; do not also hide the same cost inside an unexplained “loyalty to current action” bonus.

Compare **remaining** value, not historical expenditure. Continuing because abandoning a half-built wall would waste usable progress can be sensible; continuing merely because much time was already spent is a different behavioral assumption.

Use different enter/exit thresholds for need states, safe interruption points, and minimum commitment durations. These mechanisms must have escape conditions: commitment without failure detection produces stuck agents.

### Calibrating stochastic choice

For acceptable alternatives, softmax gives:

\[
P(a)=
\frac{\exp((U(a)-U\_{\max})/\tau)}
{\sum\_b\exp((U(b)-U\_{\max})/\tau)}.
\]

The temperature \(\tau\) controls sensitivity to utility differences. Logit’s dependence on utility scale and its substitution assumptions are covered in Train’s discrete-choice treatment. [Econometrics Laboratory](https://eml.berkeley.edu/books/choice2nd/Ch03_p34-75.pdf)

For two alternatives separated by \(\Delta U\),

\[
P(\text{better})=\frac{1}{1+\exp(-\Delta U/\tau)}.
\]

Therefore, a desired probability \(p\) implies:

\[
\tau=\frac{\Delta U}{\ln(p/(1-p))}.
\]

For example, making a 0.2-point advantage win 90% of the time gives \(\tau\approx0.091\). This is a calibration illustration, not an empirical estimate of human behavior.

Prefer persistent variation in traits, habits, information, and obligations over continual random noise. **Sample once per genuine decision**, not every update.

Watch for duplicate-option bias. Ten equivalent “sit on bench” candidates can collectively attract more probability than one “eat” candidate merely because more benches were enumerated—the same structural issue illustrated by logit’s red-bus/blue-bus example. Group activity intent before choosing its target, or explicitly model correlated alternatives. [Econometrics Laboratory](https://eml.berkeley.edu/books/choice2nd/Ch03_p34-75.pdf)

Finally, provide a valid fallback when no candidate is feasible: wait, seek information, request assistance, or search farther. Never let an empty or all-rejected candidate set silently become an arbitrary action.

## 4.4 Give notables bounded, receding-horizon HTN planning

Use this pipeline:

> Generate relevant goals → score goals → examine a few applicable methods → produce a partial plan → execute to a checkpoint → observe → revise.

An initial implementation should favor **total-order forward decomposition**, with limits on method alternatives, target bindings, recursion, and search expansions. Start by profiling budgets such as **256–2,048 expansions per planning request**; predicate costs and domain structure will determine whether those numbers are useful.

Maintain a compact planning state containing only relevant facts. Do not clone a settlement, economy, or full social graph at each branch. Humphreys’s HTN description explicitly advocates an abstract world state containing what the planner needs rather than a complete representation of the game world. [GameAIPRO](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter12_Exploring_HTN_Planners_through_Example.pdf)

Track which assumptions a plan depends on: office ownership, a road’s availability, a contract, a proposal version, an expected supporter. Reconsider when those change, not when any world property changes.

A planning budget that expires should yield a distinguishable result:

> Valid partial plan; no plan found within budget; genuinely infeasible under the checked methods.

Those are not equivalent.

**Authored methods should describe competence, not historical outcomes.** “Obtain food through production, exchange, aid, or coercion” is reusable know-how. “After 100 years, establish feudal government” is a scripted trajectory.

Likewise, a planner cannot discover an unknown technology merely because the simulation contains its eventual effects. Innovation needs separate hypothesis, experimentation, learning, and knowledge-transfer mechanics. Those mechanics can expose new actions and methods when discoveries actually occur.

Add GOAP only after a concrete case demonstrates that missing sequence combinations—not missing methods, bad goals, or poor beliefs—are the limiting factor.

## 4.5 Model institutional planning as uncertain interaction

Institutional actions should include proposing rules, convening meetings, requesting endorsements, negotiating concessions, recruiting members, making promises, voting, and enforcing decisions.

Represent institutions explicitly: membership, offices, authority, procedures, treasury, quorum, vetoes, enforcement capability, and legitimacy. A proposal changes particular rule parameters; it should not require a bespoke planner action for every imaginable law.

### Do not make another person’s consent an action effect

“Request endorsement” can create an outstanding request. It cannot directly set `other_person_supports = true`.

The responder evaluates the request through their own preferences, obligations, beliefs, and commitments. Comme il Faut provides a useful precedent: an initiator’s desired social effect and the responder’s acceptance are distinct, and rejection can produce different consequences. [GameAIPRO](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter43_An_Architecture_for_Character-Rich_Social_Simulation.pdf)

For TCE, plan through uncertain interactions using checkpoints and contingent continuations. Do not build an enormous joint search over every actor’s possible future actions.

### Estimate support from the proposer’s beliefs

A simple initial support model is:

\[
p\_{ij}(L)=
\sigma\!\left(
\frac{\widehat{\Delta U}\_{ij}(L)}{T\_j}
\right),
\]

where \(\widehat{\Delta U}\_{ij}\) is proposer \(i\)’s estimate of how supporter \(j\) values approving rather than rejecting proposal \(L\).

That estimate can combine perceived material interests, values, relationships, faction pressure, credible promises, and expected enforcement. It should use **the proposer’s information**, not unrestricted access to the target’s internal utility calculation.

Distinguish uncertainty from randomness. A proposer may be uncertain about a person whose underlying preference is stable. That does not justify rerolling that person’s loyalty every simulation tick.

### Calculate passage probability, not just expected votes

Consider an illustrative seven-member council requiring four votes. There are two certain supporters, two certain opponents, and three uncertain members, each independently supporting with probability 0.6.

Expected support is \(2+3(0.6)=3.8\), but the probability of passage is:

\[
P(\text{at least two of three})
=
3(0.6)^2(0.4)+(0.6)^3
=
0.648.
\]

If one uncertain member becomes a genuinely committed third supporter, passage requires at least one of the remaining two:

\[
1-(0.4)^2=0.84.
\]

These are consequences of the stated assumptions, not calibrated political predictions.

For small councils with conditionally independent votes, a simple probability dynamic program can calculate the threshold distribution exactly. If factions vote together, use joint scenarios or shared latent variables instead; independent probabilities would misrepresent that dependence. Apply quorum, abstention, veto, and weighted-vote rules separately.

Score a plan by its expected outcomes:

\[
Q(\pi)=
P\_sV\_s+(1-P\_s)V\_f
-C\_{\text{concessions}}
-C\_{\text{effort}}
-C\_{\text{delay}}.
\]

Failure can have political value or cost; a failed proposal might signal allegiance, expose opposition, or damage prestige. Make those effects explicit rather than assuming failure always has zero value.

Bound coalition construction. Shortlist plausible partners and compare a few strategies using marginal improvement in passage probability relative to concession cost. Avoid exhaustive enumeration of every possible coalition.

After passage, continue simulating compliance and enforcement. **Winning a vote is not the same event as successfully changing society.**

## 4.6 Produce explanations from decision records

Generate explanations from the computation that actually occurred—not from a later narrative guess.

For a sampled decision or important event, retain the relevant inputs, candidates, rejection reasons, score contributions, commitment penalties, random selection information, planner method, assumptions, and execution result.

Useful outputs include:

> “Continued harvesting to complete the promised delivery. Eating was attractive, but leaving now would miss the delivery window.”

> “Sought Mira’s endorsement because she was considered persuadable and her commitment substantially improved the proposal’s chance of passing.”

> “The grain purchase failed because another buyer reserved the available stock before the transaction committed.”

Distinguish **chosen**, **reserved**, **started**, and **completed**. An explanation of selection is not proof that the action happened.

Show the strongest relevant alternative, not merely the largest positive score component. Label unexamined alternatives as unexamined. Label estimated support as a belief. A budget-limited planner must not report that all other plans were impossible.

Use reason codes and templates, materializing text on demand. Keep bounded routine traces and durable records for consequential institutional and personal events. Retaining every citizen’s full decision trace over centuries would defeat the purpose of a compact simulation.

CK3’s relationship reasons and CiF’s stored social-event context are useful precedents for exposing causal history, although neither is a substitute for logging TCE’s actual decision computation. [Steam Store](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1662468472&feed=steam_community_announcements)

## 4.7 Fit the Rust kernel and UE5.8 boundary

Keep authoritative decision-making inside Rust, with a bounded worker pool and batched state publication. Do not create one heavyweight Unreal AI controller or behavior tree merely to mirror each simulated person.

A suitable kernel phase structure is:

> Immutable decision snapshot → parallel proposals → conflict resolution → committed changes → renderer snapshot.

This makes shared-market and reservation conflicts explicit. Stable event identifiers, reproducible random streams, and inspectable commit ordering are valuable for debugging even when bit-identical simulation is not a project requirement.

Use a C-compatible DLL boundary with explicit ownership, lengths, and lifetimes. Keep Rust containers and Unreal object ownership on their respective sides; prevent unwinding across an incompatible FFI boundary and stop worker threads before unloading. Rust’s FFI guidance and Epic’s third-party-library documentation cover the relevant boundary constraints. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

For the hardware target, reserve CPU capacity for Unreal, routing, and other simulation systems rather than allocating every logical processor to deliberation. Keep the GPU focused on presentation unless profiling identifies a suitable batch workload.

UE5.8’s experimental MetaHuman Collections demonstrates a Mass-orchestrated crowd approach with different representations at different distances. It is not evidence that 50,000 full-fidelity people will render at 60 fps on an RTX 4070 Ti. Treat simulation throughput and visual crowd throughput as separate acceptance tests. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

Under overload, reduce simulation speed before silently suppressing important decisions. Camera visibility should change presentation cost, not political intelligence or access to economic opportunity.

## 4.8 Make authoring and testing part of the architecture

Compile validated text definitions into shared indexed data. Authoring records should expose score units, response curves, parameter types, applicability, durations, modeled outcomes, interruption rules, and explanation labels.

Lint for missing handlers, invalid references, unreachable methods, uncontrolled recursion, inconsistent resource accounting, impossible durations, and search-cost cycles. Hot reload should retain the definition version used by an active job or migrate it at a safe boundary—not reinterpret a half-completed action under silently changed semantics.

For AI coding agents, require each new action or method to arrive with a fixture demonstrating success, failure, interruption, and resource cleanup.

Build the implementation in three increments:

1. **Citizen vertical slice:** food, work, sleep, travel, social interaction, reservations, and explanations.
2. **One institutional scenario:** a small assembly, a proposal, endorsement requests, voting, and enforcement.
3. **Generalization and scale:** additional methods, heterogeneous institutions, content tooling, and 50,000-agent stress tests.

Test headlessly at 10,000, 25,000, and 50,000 citizens. Measure whole decision latency, candidate-query cost, planner expansions, allocations, failed reservations, abandoned jobs, and simulation days per real second.

Also test behavioral outcomes: oscillation, unmet needs, impossible commitments, repeated failed plans, work starvation, and whether predicted coalition support matches realized outcomes. Include synchronized shocks—fire, food shortage, succession, lost transport links—because ordinary-day averages hide the worst decision bursts.

Tarn Adams’s simulation-design advice is particularly appropriate here: build an understandable model iteratively and introduce complexity where it has meaningful effects, rather than attempting to design the complete simulation in advance. [GameAIPRO](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter41_Simulation_Principles_from_Dwarf_Fortress.pdf)

---

# 5. Sources, code, and version applicability

The references below are the most useful starting points for implementation. Historical sources explain specific shipped systems; they should not be interpreted as documentation of every later patch.

| Area | Primary source | Applicable version / date |
| --- | --- | --- |
| Utility foundations | [Graham — An Introduction to Utility Theory](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter09_An_Introduction_to_Utility_Theory.pdf?utm_source=chatgpt.com) | Game AI Pro, 2013; Sims-family examples |
| Utility feature design | [Lewis — Choosing Effective Utility-Based Considerations](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter13_Choosing_Effective_Utility-Based_Considerations.pdf?utm_source=chatgpt.com) | 2017; Guild Wars 2: Heart of Thorns implementation |
| Utility at production scale | [Lewis and Mark — Building a Better Centaur](https://www.gdcvault.com/play/1021848/Building-a-Better-Centaur-AI?utm_source=chatgpt.com) | GDC 2015 |
| Personality authoring | [Evans — Modeling Individual Personalities in The Sims 3](https://www.gdcvault.com/play/1012804/Modeling-Individual-Personalities-in-The?utm_source=chatgpt.com) | GDC 2010 |
| GOAP architecture | [Orkin — Agent Architecture Considerations for Real-Time Planning in Games](https://ojs.aaai.org/index.php/AIIDE/article/view/18724?utm_source=chatgpt.com) and [Three States and a Plan](https://www.gamedevs.org/uploads/three-states-plan-ai-of-fear.pdf?utm_source=chatgpt.com) | F.E.A.R.; 2005 paper and 2006 talk |
| Practical HTN | [Humphreys — Exploring HTN Planners through Example](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter12_Exploring_HTN_Planners_through_Example.pdf?utm_source=chatgpt.com) | 2013; Fall of Cybertron approach |
| HTN workload and stability | [Guerrilla — Killzone 2 Multiplayer Bots](https://www.guerrilla-games.com/read/killzone-2-multiplayer-bots?utm_source=chatgpt.com) | 2009; presentation includes workload counters |
| Current production HTN | [Guerrilla — HTN Introduction and Application in DECIMA](https://www.guerrilla-games.com/read/from-byrd-box-to-debug-boxes-htn-introduction-and-application-in-decima?utm_source=chatgpt.com) | June 2026 |
| Simulation scheduling | [Graham — Efficient, Event-Based Simulations](https://www.gameaipro.com/GameAIProOnlineEdition2021/GameAIProOnlineEdition2021_Chapter02_Efficient_Event_Based_Simulations.pdf?utm_source=chatgpt.com) | Game AI Pro Online, 2021 |
| Small-team scale precedent | [Zubek — 1000 NPCs at 60 FPS](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter34_1000_NPCs_at_60_FPS.pdf?utm_source=chatgpt.com) | Project Highrise; 2017 |
| Social interaction model | [Mateas and McCoy — An Architecture for Character-Rich Social Simulation](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter43_An_Architecture_for_Character-Rich_Social_Simulation.pdf?utm_source=chatgpt.com) | CiF / Prom Week; 2013 |
| Stochastic choice | [Train — Discrete Choice Methods with Simulation](https://eml.berkeley.edu/books/choice2.html?utm_source=chatgpt.com) | Second edition, 2009; especially Chapter 3 |

### Open-source implementations worth studying

| Project | Value for TCE | Version caveat |
| --- | --- | --- |
| [big-brain](https://github.com/zkat/big-brain?utm_source=chatgpt.com) | Rust/Bevy utility scoring, action state, and cancellation patterns | GitHub was archived in October 2025 **because development moved to Codeberg**. The archived README’s Bevy 0.16 compatibility is not a verified current compatibility statement. |
| [Fluid HTN](https://github.com/ptrefall/fluid-hierarchical-task-network?utm_source=chatgpt.com) | Compact total-order HTN concepts, partial planning, decomposition logging, pooling hooks | C# reference implementation; inspected repository snapshot, not a Rust dependency |
| [CrashKonijn GOAP](https://github.com/crashkonijn/GOAP?utm_source=chatgpt.com) | Job-system integration and GOAP visualization | Unity implementation; inspected README installs tag 3.1.2 |
| [BehaviorTree.CPP](https://github.com/BehaviorTree/BehaviorTree.CPP?utm_source=chatgpt.com) | Asynchronous actions, execution logging, visualization, and typed dataflow | Inspected README identifies 4.9; C++17, not a drop-in Rust component |

These descriptions and version caveats come from the projects’ own documentation. [GitHub](https://github.com/zkat/big-brain)

For the engine boundary, use Epic’s [UE5.8 behavior-tree overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/behavior-tree-in-unreal-engine---overview?utm_source=chatgpt.com), [third-party library integration guide](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com), and the [Rustonomicon FFI chapter](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com).

**Bottom line:** TCE’s scalable unit should be a **persistent, meaningful activity**, not a continuously reconsidered animation or tiny task. Its deliberative unit should be a **bounded project grounded in the actor’s beliefs**, not an unrestricted search over the whole civilization. Utility plus shared execution plus selective HTN gives the strongest fit to those requirements while keeping authoring, debugging, and explanations manageable.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927bd-b018-83ea-b441-7df74d44291c)
