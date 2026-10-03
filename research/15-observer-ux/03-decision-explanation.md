# Explaining TCE: a provenance system for decisions and outcomes

## Executive recommendation

**Make explanations a product of the Rust simulation kernel, not a story reconstructed afterward.** Each meaningful decision should produce a compact, structured record of what the agent considered, what it knew, which rules applied, and how the selection occurred. Each lasting outcome should retain a reference—or a self-contained summary—of the events that created it.

For TCE, I recommend three complementary mechanisms:

* **Decision receipts** for immediate questions: “Why steal rather than work?”
* **Persistent provenance for properties and institutions**: “Why is this house stone?” or “How did this law become law?”
* **A progressively disclosed explanation interface**: a sentence first, a comparison next, then a historical chain or explicitly labeled counterfactual.

The central distinction is **explaining a calculation versus explaining a history**. A tooltip showing today’s stone price cannot explain a construction choice made a century earlier. Likewise, an event log saying “law enacted” does not explain the coalition, procedure, or negotiations that produced enactment.

The evidence supports this architecture’s individual parts. It does **not** establish a ready-made, benchmarked solution for complete provenance across 50,000 agents and centuries. That scale requires an explicit retention contract and TCE-specific profiling.

---

## 1. How it works

### 1.1 Define which “why” the observer is asking

TCE should distinguish four questions internally, even when the interface presents them conversationally.

| Question | Example | Evidence required |
| --- | --- | --- |
| **Accounting:** How was this value calculated? | “Why did the population fall this year?” | Births, deaths, arrivals, departures, and their aggregation rules. |
| **Decision:** Why this option rather than another? | “Why did Mara steal rather than work?” | Considered alternatives, eligibility, scores, commitments, and selection method. |
| **Historical provenance:** How did these conditions arise? | “Why could Mara not afford food?” | Earlier employment, income, prices, purchases, losses, and relevant beliefs. |
| **Counterfactual:** What change would alter the result? | “Would cheaper grain have prevented the theft?” | A defined intervention and a rerun or analytical evaluation of the relevant model. |

These are different computational products. A dependency trace answers what the calculation used; it does not automatically establish what would have happened without a particular event.

The *Whyline* debugging project is an especially relevant precedent: it generated “why did” and “why didn’t” questions from program output and answered them using recorded runtime events and control/data dependencies. TCE can adopt that question-oriented approach without exposing source code to players. [CMU School of Computer Science](https://www.cs.cmu.edu/~NatProg/whyline.html)

### 1.2 Emit a decision receipt from the actual evaluator

Give every authored decision primitive a stable identity and an explanation contract. A food-seeking choice, hiring rule, legislative vote, construction-material selector, or succession procedure should expose the evidence it actually used.

A practical receipt would contain:

| Part | What to preserve |
| --- | --- |
| Identity | Entity ID, decision/event ID, simulation time, rule and content version. |
| Context | Current goal, parent plan, relevant commitments, actor’s information state. |
| Alternatives | Candidate actions and targets; eligibility results; selected option; retained comparisons. |
| Evaluation | Raw inputs, transformations, named contributions, interactions, clamps, priority rules. |
| Selection | Maximum-score, weighted lottery, tie-break, override, or other method; relevant random evidence. |
| Outcome linkage | What was attempted, what actually happened, and which state changes resulted. |
| Coverage | Which details were retained, summarized, recomputed, or discarded. |

**Capture inputs as they were then.** Do not read current wages, current laws, or current beliefs when explaining an old decision.

This is the same temporal problem encountered in event sourcing: replaying an old transaction requires the information available at the original time, not a fresh answer from a subsequently changed external system. Fowler’s treatment explicitly discusses preserving historical query responses and handling changes to application code. [martinfowler.com](https://martinfowler.com/eaaDev/EventSourcing.html)

The receipt should come from the evaluator itself. Avoid separately implementing “the explanation formula” in Unreal or TypeScript: two implementations will eventually disagree.

For performance, use stable numeric reason IDs and typed values in the kernel. Resolve them into localized phrases only when somebody opens an explanation.

#### Record the status of alternatives accurately

An alternative can be:

**Evaluated and worse; blocked by a prerequisite; excluded by candidate generation; or not evaluated because execution stopped early.**

These are not interchangeable.

For example, *Guild Wars 2: Heart of Thorns* used multiplicative utility considerations with early termination when a consideration scored zero. Cheap checks could therefore prevent expensive later checks from running. An explanation system layered over that architecture must not invent results for those unexecuted checks. [GameAIPRO](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter13_Choosing_Effective_Utility-Based_Considerations.pdf)

TCE should preserve candidate-generation reasons where useful: “outside known search area,” “no accessible provider,” or “not in this agent’s repertoire.” When historical evidence is unavailable, an on-demand evaluation may still help—but it must be labeled **recomputed alternative, not an option considered at the time**.

### 1.3 Explain utility differences, not merely large scores

For an additive decision model,

\[
U(a)=b(a)+\sum\_i c\_i(a),
\]

the most useful explanation of choosing \(A\) over \(B\) is often:

\[
U(A)-U(B)=\bigl[b(A)-b(B)\bigr]+\sum\_i\bigl[c\_i(A)-c\_i(B)\bigr].
\]

A large contribution to both actions may explain why either action was attractive while explaining nothing about their ordering.

#### Illustrative TCE example—not proposed balance values

Suppose both theft and paid work can ultimately obtain food:

| Contribution, in internal utility points | Steal food | Work for food | Difference favoring theft |
| --- | --- | --- | --- |
| Expected hunger relief | +60 | +60 | 0 |
| Additional future income | 0 | +20 | −20 |
| Delay before obtaining food | −5 | −45 | +40 |
| Expected detection consequences | −10 | 0 | −10 |
| Aversion to theft | −8 | 0 | −8 |
| **Total** | **37** | **35** | **+2** |

A useful explanation is:

> Mara expected theft to provide food sooner. That advantage narrowly outweighed the lost income, detection risk, and her aversion to stealing.

“Because hunger contributed +60” is incomplete: hunger contributed equally to both options.

Also show excluded alternatives separately:

> Buying food was unavailable because she lacked the required money. No charity provider was known to her.

This comparison pattern has a close research analogue in **reward-difference explanations**: Juozapaitis and colleagues compare actions through differences between semantically meaningful reward components rather than showing only their total action values. Their work also develops compact explanations containing enough positive and negative reasons to establish a preference. [Oregon State University Engineering](https://web.engr.oregonstate.edu/~erwig/papers/ExplainableRL_XAI19.pdf)

For TCE, a compact explanation can show the strongest distinguishing reasons plus an explicit “other contributions” remainder. That remainder must reconcile with the actual total.

#### Respect the actual scoring mathematics

Not every decision is additive.

For a multiplicative score, show factors and the resulting product. A zero factor may be a veto, not a large negative contribution. For strictly positive factors, log-ratios can help developer diagnostics, but they are usually inappropriate as the default player presentation.

For clamped scores, thresholds, interactions, or lexicographic priorities, preserve the evaluation stages:

> Eligible → priority class → utility calculation → commitment bonus → selection.

Do not manufacture additive “percent responsibility” from a nonlinear expression. Removing a factor and recomputing the score measures a particular sensitivity; those sensitivities need not sum to the original score.

Likewise, internal utility points are **not percentages, probabilities, or interpersonal welfare units**. A score of 80 for one person does not necessarily indicate twice the motivation of another person scoring 40.

#### Expose randomness without making it the whole explanation

When selection is stochastic, preserve the selection rule, relevant probabilities or weights, normalization information, and the random draw or reproducible stream reference.

Distinguish:

> Work had the highest selection probability, but theft was selected.

from:

> Theft had the highest evaluated utility.

Also distinguish randomness in **choosing an action** from randomness in **its consequences**. Choosing theft, being noticed, and being convicted may involve three different mechanisms.

### 1.4 Connect decisions to a versioned provenance graph

Use a compact internal graph of the form:

**Facts at a particular time → evaluation or process → committed event → resulting facts.**

The W3C PROV model provides a useful vocabulary: entities, activities, agents, usage, generation, and derivation. TCE need not store RDF or adopt the entire standard; a small typed representation can borrow these distinctions. [W3C](https://www.w3.org/TR/prov-primer/)

Useful TCE edge types include:

`used`, `believed`, `enabled`, `blocked`, `generated`, `replaced`, `aggregated_from`, and `executed_under`.

The distinction between **used** and **caused** matters. An evaluator might read a variable whose value did not affect the outcome. A historical event may contribute to an outcome without being individually necessary.

Version the facts rather than linking everything to mutable entities. “Mara’s cash at tick 410” and “Mara’s cash at tick 900” are different evidence. Temporal versioning also lets feedback processes be represented as successive events rather than as an unintelligible cycle.

Shared causes should be shared records. A drought affecting thousands of households should not be copied into thousands of full narrative chains.

#### Preserve beliefs separately from world truth

An agent may act sensibly given incorrect information:

> Mara believed the granary was closed because she heard that yesterday. It had reopened that morning, but she had not learned this.

A receipt therefore needs the input **as believed**, its source where modeled, and the time it was acquired. The observer should be able to compare the agent’s information with the actual world state.

Research on **model reconciliation** makes the related point that an explanation may need to resolve differences between the actor’s model and the recipient’s model, rather than merely expose a plan. Applying this to TCE suggests explicitly displaying missing knowledge and mistaken assumptions. [arXiv](https://arxiv.org/abs/1701.08317)

### 1.5 Give lasting properties their own provenance

A recent-decision log will not preserve the reasons behind a century-old building.

For important persistent properties, retain a compact **provenance certificate**: the originating event, decisive constraints, relevant historical values, rule version, and references to whatever deeper history remains.

**A stone house.** Preserve the material-selection decision, procurement result, and construction outcome separately. The builder may have preferred timber, failed to acquire it, and substituted stone. Later repairs may explain the present facade better than the original construction decision.

**A law.** Preserve the constitutional procedure actually used: proposal, eligibility, amendments, agenda control, voting or ruler authorization, quorum, vetoes, and enactment. For a voting polity, link individual or faction decisions to the final tally. Do not flatten the institution into one “society utility” score.

**A shrinking settlement.** Begin with exact accounting:

\[
\Delta P=\text{births}-\text{deaths}+\text{arrivals}-\text{departures}.
\]

Then link deaths and departures to relevant episodes. A count of departures whose recorded decisions included housing pressure is not automatically a causal estimate of how much population loss housing policy produced.

**An accidental collapse or failed harvest.** Record the responsible process, inputs, thresholds, and stochastic outcomes. These outcomes need mechanical explanations, not fictitious intentions.

Finally, distinguish **why the simulation did something** from **why its designers chose that rule**. Rule documentation can explain the model’s rationale, but it must not masquerade as an agent’s motive.

### 1.6 Use advanced XAI selectively

| Technique | Appropriate TCE use | Important limit |
| --- | --- | --- |
| Native contribution and rule traces | Default explanation of authored decisions. | Explain the implemented calculation, not automatically real-world causation. |
| Contrastive explanations | “Why stone rather than timber?” | Must identify the comparison and whether it was actually considered. |
| Compact sufficient explanations | Reduce a long breakdown while preserving the preference ordering. | Omitted reasons and aggregate opposition still matter. |
| Model reconciliation | Explain actions based on stale knowledge, unfamiliar norms, or different goals. | Requires a modeled information state. |
| Counterfactual reruns | Investigate a proposed change to price, law, knowledge, or opportunity. | Must define the intervention, horizon, and treatment of randomness. |
| SHAP-style feature attribution | Optional diagnostics for an opaque learned component. | Feature importance for a prediction is not a historical causal record. |

SHAP assigns features importance values for a particular prediction. That is useful when a component is genuinely opaque; it is unnecessary overhead when TCE already owns an explicit, named utility calculation. [arXiv](https://arxiv.org/abs/1705.07874)

Start counterfactual support with **local decision sensitivity**. In the illustrative theft calculation, increasing the theft penalty by more than two points, while holding everything else fixed, changes the ordering. That does not establish that a policy producing such a penalty would prevent theft throughout the town.

A full historical counterfactual needs a sandboxed simulation branch. Deterministic replay requires more than retaining a seed: ordering, inputs, implementation versions, and floating-point behavior can matter. Fiedler’s lockstep discussion illustrates how even apparently minor differences can cause replay divergence. [Gaffer On Games](https://gafferongames.com/post/deterministic_lockstep/)

---

## 2. What worked—and what failed—in existing systems

### 2.1 Victoria 3: preserve the event behind a changed attitude

**This is the closest player-facing precedent for TCE.**

In its 2024 *Diplomatic Catalysts* diary, Paradox described an older system in which a hidden progress mechanism eventually triggered reconsideration of a country’s strategic desire. The developer identified opacity as the problem: an ally might become hostile for reasons the player was never told.

The replacement tied reconsideration to recognizable occurrences such as bankruptcy, broken agreements, or ideological changes. Crucially, the attitude tooltip could still explain the reason afterward, even when the player missed the notification. [Reddit](https://www.reddit.com/r/victoria3/comments/1c743c4/victoria_3_dev_diary_113_diplomatic_catalysts/)

The accompanying community discussion exposed another distinction. A player argued that merely naming bankruptcy did not explain why it caused hostility: the message needed to communicate perceived vulnerability. The developer agreed to revise that wording. Other commenters welcomed explanations as a way to understand—and potentially counter—relationship changes. These are qualitative reactions, not a measured usability study. [Reddit](https://www.reddit.com/r/victoria3/comments/1c743c4/victoria_3_dev_diary_113_diplomatic_catalysts/)

**TCE lesson:** preserve both the triggering event and its interpretation. “Bankruptcy occurred” is an event label; “the neighboring ruler interpreted bankruptcy as an opportunity for coercion” explains its role in a decision.

### 2.2 Crusader Kings III: nested explanations and one source of truth

CK3’s tooltip design supports nested concepts, with configurable timer locking or manual middle-mouse locking. Its encyclopedia is generated from game scripts rather than maintained as a separate copy of the rules. Those are strong precedents for progressive disclosure and keeping explanations synchronized with mechanics. [Paradox Plaza Forum](https://forum.paradoxplaza.com/forum/threads/ck3-dev-diary-16-tutorials-and-tooltips-and-encyclopedias-oh-my.1345581/)

There is a useful implementation failure in the same diary: hot reload invalidated database objects still referenced by encyclopedia pages, causing crashes. The solution used regeneration callbacks and page generators rather than retaining invalid object pointers. [Paradox Plaza Forum](https://forum.paradoxplaza.com/forum/threads/ck3-dev-diary-16-tutorials-and-tooltips-and-encyclopedias-oh-my.1345581/)

The 1.5.1 patch notes also explicitly changed the presentation of liege taxes into a breakdown, illustrating continued refinement of where explanatory detail belongs. [Steam Store](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1648819844&feed=steam_community_announcements)

**TCE lesson:** generate definitions from authored rules, but use immutable recorded values for historical explanations. Reference entities and rule versions through stable handles, not live pointers embedded in UI state.

### 2.3 Victoria 3 law enactment: a probability is not enough

The historical 1.3 design diary acknowledged frustrating randomness, repeated bad rolls, cancellation exploits, and confusing tooltips. It replaced immediate enactment on a successful checkpoint with three phases, accelerated the clock, and associated enactment events with the outcome that spawned them. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-80-law-enactment-and-revolution-clock-in-13)

**TCE lesson:** show the process, not just “enactment chance.” Observers need to know what stage failed, who opposed it, what changed, and whether the latest outcome was a political decision or a random resolution. A perfectly accurate percentage can still fail to explain the institution.

These are historical lessons from that revision, not claims about the game’s current balance.

### 2.4 Dragon Age: Inquisition: retaining losing options paid off

Its Behavior Decision System evaluated behavior snippets and targets, stored their results in a summary table, and chose the highest-scoring result. The developers explicitly note that execution needed only the winner, but retaining the other evaluations in a debug-viewable table provided valuable insight during iteration. [GameAIPRO](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter31_Behavior_Decision_System_Dragon_Age_Inquisition%E2%80%99s_Utility_Scoring_Architecture.pdf)

**TCE lesson:** a winner-only record loses much of the explanatory value. Preserve at least the nearest meaningful alternatives, with a route to more detailed capture for selected agents.

The Guild Wars 2 account provides a complementary warning: cooldowns and runtime considerations were used to prevent repetitive or rapidly alternating behavior. Such stabilizers are part of the decision and should appear in its explanation, rather than being hidden behind a misleading “best action” story. [gameaipro.com](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter13_Choosing_Effective_Utility-Based_Considerations.pdf)

### 2.5 Final Fantasy XV: log cheaply, analyze elsewhere—but budget overload

The developers used typed binary logging, multithreaded double buffers, and an external aggregation/visualization pipeline. Their visualizations helped identify movement problems and unexpectedly frequent dialogue scripts. Heavy transformation into database-friendly representations happened outside the game-side logging path. [GameAIPRO](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter03_Logging_Visualization_in_FINAL_FANTASY_XV.pdf)

The implementation also documents a boundary condition: although reservation was normally nonlocking, insufficient buffer space could block until a buffer swap. [GameAIPRO](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter03_Logging_Visualization_in_FINAL_FANTASY_XV.pdf)

**TCE lesson:** “asynchronous logging” is not a complete overload policy. Decide which records must survive, which detail can be dropped, and how the UI reports lost coverage.

### 2.6 Full Spectrum Command: useful explanations have a depth ceiling

This deployed training simulation recorded timestamped AI events, reconstructed unit/task state during after-action review, and filled question-and-answer templates from those records.

Its authors identified an important limitation: the system could explain how a predefined task decomposition was applied, but not deeper reasons absent from the task knowledge. Some lower-level explanations were also removed partly because they exposed behavior inconsistent with the intended doctrine. [AAAI](https://cdn.aaai.org/IAAI/2004/IAAI04-019.pdf)

**TCE lesson:** explanations cannot reveal motives or mechanisms the simulation never represented. They will also expose modeling shortcuts. Treat that as a reason to improve or document the model—not to replace the evidence with a more flattering narrative.

### 2.7 Whyline and reward decomposition: explanation is also a design instrument

Whyline’s initial prototype lacked “why didn’t” questions; its first participant wanted precisely that capability and dismissed the tool when it was absent. In the subsequent small study, five participants used the tool and four did not. Across six comparable debugging scenarios, the reported average debugging-time improvement was 7.8-fold. The sample and iterative study design make this suggestive, not a transferable TCE productivity multiplier. [CMU School of Computer Science](https://www.cs.cmu.edu/~NatProg/papers/Ko2004Whyline.pdf)

Reward-decomposition case studies likewise exposed problems hidden by total scores: inconsistent estimates, domination by shaping rewards, and optimizer-related anomalies. The paper’s evidence was primarily practitioner-oriented case studies, not a demonstrated improvement in mass-market player comprehension. [Oregon State University Engineering](https://web.engr.oregonstate.edu/~erwig/papers/ExplainableRL_XAI19.pdf)

**TCE lesson:** use the observer explanation system during simulation development. It should reveal broken incentives before those incentives produce centuries of inexplicable history.

---

## 3. Bounded memory, scale, and performance

### 3.1 The raw storage arithmetic is unforgiving

The following are **illustrative calculations, not measured TCE workloads**.

Assume 24 recorded meaningful decisions per person per simulated day, a 128-byte compact record, and 365 days per year:

| Population | Records per simulated day | Raw storage per day | Per year | Per century |
| --- | --- | --- | --- | --- |
| 10,000 | 240,000 | 30.72 MB | 11.21 GB | 1.12 TB |
| 50,000 | 1,200,000 | 153.60 MB | 56.06 GB | 5.61 TB |

These figures exclude indexes, graph edges, snapshots, strings, and detailed candidate evaluations. Twelve alternatives with eight factors at 16 bytes per factor already require **1,536 bytes**, before metadata. A 128-byte record should therefore be understood as a compact summary, not a full decision trace.

At 50,000 people, a **256 MiB ring buffer** holds only about **1.75 simulated days** of those compact records. Detailed traces reduce that duration further.

Time acceleration matters independently of rendering. At one simulated day per real second, this example generates **1.2 million records and 153.6 MB of raw output per second**.

### 3.2 Separate bounded RAM from bounded historical storage

Bounded RAM is straightforward with chunked storage and caches. Bounded total history requires deciding what can be forgotten.

A deterministic closed simulation can, in principle, regenerate old states from compact initial conditions. That exchanges storage for replay time and depends on retaining compatible code and inputs. It is not the same as promising arbitrary historical explanations at interactive latency.

I recommend this explicit TCE contract:

> Every live entity can explain its current important behavior and defining properties. Recent events retain detailed evidence. Older events retain progressively coarser evidence, with the retained scope visible.

That is more achievable than silently promising exact explanations for every historical decision forever.

### 3.3 Use four retention tiers

| Tier | Contents | Retention policy |
| --- | --- | --- |
| **Current state** | Latest important decision and provenance of defining properties. | Retained while relevant to a live entity. |
| **Recent detail** | Candidate traces, beliefs, gates, and execution outcomes. | Byte-bounded rolling storage. |
| **Historical episodes** | Employment spells, shortages, migrations, construction projects, legislative processes. | Compact structured summaries with limited exemplars. |
| **Selected archive** | Foundings, constitutions, disasters, major institutions, observer bookmarks. | Explicit quotas or a separately growing disk archive. |

A fixed-budget recorder that discards the oldest frames is an established game-debugging pattern; David Young describes one used for AI, animation, and steering investigations in Treyarch titles. But visual scrubbing records are not automatically complete simulation checkpoints or decision explanations. [GameAIPRO](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter06_Debugging_AI_with_Instant_In-Game_Scrubbing.pdf)

Several details determine whether TCE’s version remains trustworthy:

**Protect persistent properties independently of the ring.** A building’s construction explanation should survive ordinary recent-log eviction.

**Do not retain every ancestor recursively.** A pinned law can depend on thousands of lives. Before evicting supporting detail, replace necessary boundaries with self-contained summaries containing decisive values, rule identities, and explicit omissions.

**Allocate fairly.** A disaster must not erase every quiet citizen’s latest explanation. Maintain per-entity current slots separately from global event buffers.

**Distinguish aggregates from samples.** Counts and sums can be exact for the dimensions explicitly retained. A few example households cannot answer questions about every household.

**Make loss visible.** Use separate fields for evidence coverage—exact, summarized, unavailable—and explanation method—recorded, replayed, estimated. One generic confidence percentage would blur these distinctions.

As a sizing example, eight 512-byte capsules for each of 50,000 people consume **204.8 MB** before indexes or other history. That is a useful budget calculation, not evidence that all relevant context fits into 512 bytes.

### 3.4 What the published performance numbers actually establish

Full Spectrum Command supported up to **200 entities** and generated thousands to tens of thousands of AI records in missions lasting **30 minutes to two hours**. That demonstrates practical semantic logging and after-action explanations, but at a very different scale from TCE. [AAAI](https://cdn.aaai.org/IAAI/2004/IAAI04-019.pdf)

The FFXV chapter reports transmission batches at **1–10 sends per second** and a theoretical **80 Mbps** bandwidth ceiling that its described logging did not reach. It does not provide a portable CPU-overhead figure for this architecture. [GameAIPRO](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter03_Logging_Visualization_in_FINAL_FANTASY_XV.pdf)

For TCE, benchmark capture CPU, allocation rate, bytes per semantic event, worst-case buffer pressure, query latency, and retained coverage at maximum simulation speed. Average frame rate alone will miss the relevant failure modes.

---

## 4. Lessons for TCE: what to adopt, adapt, and avoid

### 4.1 Build one explanation service with several presentations

The Rust kernel should own a read-only query interface conceptually equivalent to:

`explain(entity, subject, historical_time, comparison, detail_budget)`

The response should contain structured evidence and stable references—not only prose. Unreal and the observer panels can present the same response differently.

Epic’s Visual Logger is useful for developer-side spatial inspection: actor snapshots, text, shapes, and timeline scrubbing. Its documented snapshot behavior is frame-oriented, including overwriting previously captured actor snapshot data within the same frame. It should therefore be an optional visualization/export destination, not TCE’s authoritative provenance store. [Epic Games Developers](https://dev.epicgames.com/documentation/unreal-engine/visual-logger-in-unreal-engine?lang=en-US)

Keep explanation queries outside the simulation’s mutation path. Opening an inspector must not change scheduling, consume simulation randomness, or alter subsequent behavior.

### 4.2 Use a four-level observer interface

**First level: a causal sentence.** Include the time, actor or process, and distinguishing mechanism.

Illustrative presentations:

> Mara chose theft because it provided food sooner than paid work, narrowly outweighing the risks.

> This house was built in stone after its timber purchase failed; a local quarry supplied an affordable substitute.

> This law passed under the council’s majority procedure after an amendment changed enough members’ positions.

**Second level: a contrastive breakdown.** Show the chosen option beside one meaningful alternative. Display positive and negative differences, prerequisites, commitments, and stochastic selection separately.

**Third level: history and provenance.** Expand a reason into the events that established it. Clicking “timber purchase failed” should lead to that transaction and its supplier, not to a generic encyclopedia page about timber.

**Fourth level: a counterfactual workspace.** Offer controlled questions such as “What grain price would reverse this choice?” Distinguish a local recalculation from a world replay and distinguish a single stochastic branch from an estimated probability.

Borrow nested tooltips for definitions, but move substantial investigations into a pinned panel with breadcrumbs, a time cursor, back navigation, and map highlighting. Do not require the observer to maintain an elaborate hover chain while comparing two explanations.

Freeze the evidence time while an explanation is open, or clearly indicate that it is updating. A moving total paired with historical contributions is especially misleading.

### 4.3 Keep natural-language generation subordinate to evidence

Use deterministic templates for the default explanation layer. Templates do not imply scripted history: the events, inputs, and combinations remain emergent.

A language model could later summarize a retrieved evidence bundle, but it should not decide what the causes were. Require each factual clause to map to retained records, and prevent it from filling provenance gaps with plausible motives.

This distinction is not hypothetical. The *AI Rationalization* research deliberately generated explanations as though a human had performed an agent’s actions, with accessibility and satisfaction benefits while sacrificing absolute accuracy. That can be a legitimate interface objective, but it is not the same product as TCE’s promised causal audit trail. [arXiv](https://arxiv.org/html/1702.07826v2)

### 4.4 Make explanation support part of the content contract

For each authored building block, require a stable rule ID, version, display name, input semantics and units, prerequisite descriptions, contribution labels, and outcome/provenance hooks.

Validate these alongside the rule itself. New economic, political, or architectural primitives should not ship with an unexplained numerical modifier simply because their behavior “works.”

Preserve a distinction between:

> The agent considered this costly.

and:

> The designers’ current model assigns this a cost.

That distinction is particularly important when TCE represents norms, political legitimacy, or social conflict. Explaining the model faithfully does not validate its historical assumptions.

### 4.5 Implement three vertical slices before generalizing

Start with **theft**, **law enactment**, and **building material choice**.

Together they exercise individual utility, unavailable alternatives, beliefs, collective procedure, durable properties, procurement failures, and historical retention. Complete the route from kernel computation to player explanation for each one before instrumenting every subsystem.

The minimum acceptance tests should establish that:

* Displayed calculations reconcile with the actual evaluation, including interactions, rounding, gates, ties, and randomness.
* Old explanations survive save/load and identify the correct rule version rather than silently adopting new rules.
* Entity deletion, ID reuse, compaction, and buffer overflow do not produce false historical links.
* Inspection and capture do not change simulation outcomes.
* Missing evidence produces a visible limitation, never an invented reason.

Then test comprehension, not merely interface satisfaction. After reading an explanation, can an observer identify the decisive constraint, distinguish belief from reality, and predict which of two proposed changes would matter?

**Adopt native traces, persistent property provenance, contrasts, and progressive disclosure. Adapt recording depth to memory and historical significance. Avoid winner-only logs, unlimited ancestry retention, duplicated UI formulas, and persuasive explanations unsupported by execution evidence.**

---

## 5. Sources and further reading

The most useful starting points are the following. Developer accounts establish what their systems did; research studies provide narrower experimental evidence; community discussion identifies concrete interpretation problems.

| Area | Linked sources | Why they matter |
| --- | --- | --- |
| Historical reasons in player UI | [Victoria 3: Diplomatic Catalysts](https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-113-diplomatic-catalysts.1666267/?utm_source=chatgpt.com) · [Discussion and developer reply](https://www.reddit.com/r/victoria3/comments/1c743c4/victoria_3_dev_diary_113_diplomatic_catalysts/?utm_source=chatgpt.com) | Closest precedent for retaining the reason behind a lasting state change. |
| Nested explanations | [CK3: Tutorials, Tooltips, and Encyclopedias](https://forum.paradoxplaza.com/forum/threads/ck3-dev-diary-16-tutorials-and-tooltips-and-encyclopedias-oh-my.1345581/?utm_source=chatgpt.com) | Locking modes, generated documentation, and the hot-reload failure. |
| Political-process legibility | [Victoria 3: Law Enactment in 1.3](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-80-law-enactment-and-revolution-clock-in-13?utm_source=chatgpt.com) | Developer diagnosis of randomness and process-feedback problems. |
| Utility architecture | [Dragon Age: Inquisition’s Behavior Decision System](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter31_Behavior_Decision_System_Dragon_Age_Inquisition%E2%80%99s_Utility_Scoring_Architecture.pdf?utm_source=chatgpt.com) · [Choosing Effective Utility-Based Considerations](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter13_Choosing_Effective_Utility-Based_Considerations.pdf?utm_source=chatgpt.com) | Retained alternative scores, target selection, early-outs, and behavioral stabilizers. |
| Recording architecture | [Logging Visualization in FFXV](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter03_Logging_Visualization_in_FINAL_FANTASY_XV.pdf?utm_source=chatgpt.com) · [Instant In-Game Scrubbing](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter06_Debugging_AI_with_Instant_In-Game_Scrubbing.pdf?utm_source=chatgpt.com) | Binary capture, off-path analysis, overload, and fixed-memory recording. |
| UE tooling and talk | [Visual Logger documentation](https://dev.epicgames.com/documentation/unreal-engine/visual-logger-in-unreal-engine) · [Andy Bastable, Unreal Fest Europe 2019](https://dev.epicgames.com/community/learning/talks-and-demos/35J/the-visual-logger-for-all-your-gameplay-needs-unreal-fest-europe-2019-unreal-engine?utm_source=chatgpt.com) | Spatial debugging and a talk featuring Sea of Thieves examples. |
| Explanation through execution history | [Designing the Whyline](https://www.cs.cmu.edu/~NatProg/papers/Ko2004Whyline.pdf?utm_source=chatgpt.com) · [Full Spectrum Command XAI](https://cdn.aaai.org/IAAI/2004/IAAI04-019.pdf?utm_source=chatgpt.com) | “Why not,” time-oriented questions, templates, and limits of explanation depth. |
| Action attribution | [Reward Decomposition](https://web.engr.oregonstate.edu/~erwig/papers/ExplainableRL_XAI19.pdf?utm_source=chatgpt.com) · [SHAP](https://arxiv.org/abs/1705.07874?utm_source=chatgpt.com) | Contrastive components versus post-hoc prediction attribution. |
| Beliefs and narration | [Model Reconciliation](https://arxiv.org/abs/1701.08317?utm_source=chatgpt.com) · [AI Rationalization](https://arxiv.org/abs/1702.07826) | Different knowledge states and the distinction between fidelity and plausibility. |
| Provenance and replay | [W3C PROV Primer](https://www.w3.org/TR/prov-primer/?utm_source=chatgpt.com) · [Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html?utm_source=chatgpt.com) · [Deterministic Lockstep](https://gafferongames.com/post/deterministic_lockstep/?utm_source=chatgpt.com) | Vocabulary, historical inputs, and replay constraints. |
| Player-facing reference | [Victoria 3 diplomacy wiki](https://vic3.paradoxwikis.com/Diplomacy?utm_source=chatgpt.com) | A navigable mechanics reference; check its version annotations when comparing revisions. |

**The defining design choice is to preserve evidence when the world changes.** Once that exists, TCE can offer short explanations, detailed accounting, historical investigation, and carefully scoped counterfactuals through the same underlying system.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92998-7550-83ea-8663-e5215346ea97)
