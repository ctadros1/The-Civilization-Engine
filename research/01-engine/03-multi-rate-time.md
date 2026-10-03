# Multi-rate simulation and time acceleration for TCE

**Engineering report — sources checked through September 27, 2026**

## Executive recommendation

**Build one authoritative stochastic simulation, with multiple ways to execute it—not separate “detailed” and “daily” rule sets that are calibrated to produce similar averages.**

For TCE, the strongest architecture is an **event-driven, multi-rate kernel with persistent individual state**, accelerated by analytical updates, cached activity plans, and carefully bounded statistical batches. Low-speed observation and high-speed advancement should consume the same authoritative decisions, travel constraints, resource transactions, and random events.

Interpret “one statistical step per day” as **one external request to advance to the next day**, not as a prohibition on processing important events inside that day.

That distinction is essential. A worker who dies at breakfast must not produce a full day’s output. An afternoon grain delivery cannot satisfy a morning purchase. A disease exposure cannot propagate backward through an earlier contact. A daily resolver must preserve such dependencies, whether explicitly or through an equivalent joint sampler.

**My recommendation is to make simulation speed independent of simulation accuracy.** First accelerate the same model by eliminating unnecessary work. Introduce speed-specific approximations only where measured distributional differences are acceptable—and do not describe those approximations as preserving the same individual history.

---

# 1. Options: what can actually remain equivalent?

## 1.1 Define “the two must not diverge”

There are three substantially different requirements:

| Requirement | Meaning | Appropriate expectation |
| --- | --- | --- |
| **Same individual history** | Given the same initial state, randomness, and commands, the same people experience the same consequential events. | Achievable with shared event semantics, coupled randomness, and genuinely equivalent batching. |
| **Same outcome distribution** | Individual histories may differ, but their conditional probabilities, correlations, and outcome distributions agree. | Appropriate for validated stochastic approximations. |
| **Similar aggregate averages** | Population, production, or mortality averages look similar. | Too weak for TCE: inequality, extinction, innovation, and political histories can still be wrong. |

The mathematical difficulty is **closure**. Suppose two detailed states have the same daily summary, but different futures because one has a queue outside a granary and the other does not. A daily transition depending only on that summary cannot reproduce both futures correctly. Research treating agent-based models as Markov chains formalizes this through projection and *lumpability*: an aggregated state is not automatically sufficient to predict its own evolution. [arXiv](https://arxiv.org/abs/1108.1716)

For TCE, the practical implication is:

> **Coarsen execution time before coarsening people or discarding causal state.**

Keep identities, household membership, current activities, disease timing, resource ownership, reservations, and important relationships even when an entire day is resolved in one API call.

An exact daily transition exists in principle for a fully specified stochastic model. The problem is that sampling it cheaply may require solving essentially the same interactions that the fine simulation performs.

## 1.2 Main techniques

The suitability judgments below are engineering recommendations for TCE, not published benchmark rankings.

| Technique | How it works | Fidelity, performance, and maturity |
| --- | --- | --- |
| **Fixed semantic multi-rate scheduling** | Different systems run at fixed intervals in simulation time: activity changes when needed, budgets daily, institutional reviews less often. Fast-forward executes the same schedule faster. | Low implementation risk. Preserves the existing model when update times and ordering remain unchanged. Does not eliminate intrinsically expensive events. |
| **Discrete-event execution and lazy updates** | Schedule arrivals, completions, state changes, and stochastic events; analytically advance unchanged quantities between them. | Best first optimization for TCE. Saves idle polling without necessarily approximating outcomes. Dependency invalidation and dense event traffic are the main costs. |
| **Statistical batching / tau-leaping** | Hold event rates approximately constant over an interval and sample counts rather than process every event separately. | Potentially large savings for frequent events. Usually approximate; requires bounded-resource handling, adaptive intervals, and exact fallback. |
| **Multi-rate integration and operator splitting** | Integrate fast and slow continuous processes on different internal schedules, exchanging states, forcing, or accumulated fluxes. | Mature for differential equations. Useful for soil moisture, spoilage, environmental fields, and related continuous quantities—not a general solution for discrete social decisions. |
| **Quasi-steady or slow-scale averaging** | Replace a rapidly mixing process by its conditional equilibrium distribution while advancing slower variables. | Powerful when genuine timescale separation exists. Dangerous near bottlenecks, crises, transitions, or persistent social memory. |
| **Equation-free / coarse projective integration** | Run short detailed bursts, measure coarse trends, project those trends forward, then reconstruct detailed states. | Established research approach, but reconstruction and lost correlations make it a poor first authoritative architecture for TCE. |
| **Adaptive local resolution** | Keep sensitive regions or interactions detailed while using cheaper representations elsewhere. | Potentially useful later. Boundary ownership, conservation, cross-region effects, and switching consistency create substantial engineering complexity. |

Several literature connections are particularly useful:

**Discrete-event stochastic simulation.** The temporal Gillespie method demonstrates that event-driven simulation can preserve a specified stochastic process on a changing contact network while avoiding repeated unsuccessful event checks. Its Poisson-process formulation is exact under its stated assumptions; extensions to non-Markovian processes require additional qualifications. [PLOS](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1004579)

**Tau-leaping.** Here an event channel with current rate \(a\_j(X)\) is approximated over interval \(\Delta\) by a Poisson event count with mean \(a\_j(X)\Delta\). Freezing rates is the approximation. The literature distinguishes strong, trajectory-level error from weak, distributional error; improved integration order under particular assumptions is not a blanket guarantee for arbitrary agent logic. [arXiv](https://arxiv.org/html/0909.4790v3)

**Multi-rate numerical integration.** SUNDIALS ARKODE’s multirate methods explicitly account for coupling between slow and fast processes. Their relevance is the architecture: one requested output interval can contain many internal integration steps. They should not be treated as a drop-in integrator for arbitrary agent decisions. [SUNDIALS](https://sundials.readthedocs.io/en/latest/arkode/Mathematics_link.html)

**Slow-scale reduction.** Gillespie and colleagues’ work on stochastic model reduction emphasizes that removing a fast process is a conditional mathematical approximation, not simply “update this subsystem less frequently.” [PubMed](https://pubmed.ncbi.nlm.nih.gov/19222263/)

**Equation-free methods.** Cisternas and colleagues use short individual-based simulation runs to perform coarse integration and analysis. For TCE, I would use this family of methods for offline exploration or calibration before considering it for authoritative gameplay. [arXiv](https://arxiv.org/abs/nlin/0310011)

### A distinction worth preserving

An **exact endpoint sampler is not necessarily an exact day resolver**.

For example, correctly sampling whether someone ends the day healthy or sick does not necessarily preserve their hours worked, contacts made, food consumed, or treatment received. The required transition must include every accumulated quantity that affects other systems—not just the person’s final label.

---

# 2. Trade-offs: accuracy, bias, variance, and throughput

## 2.1 Sampling an agent’s day correctly

### Use integrated hazards, not probability multiplied by time

For an event with instantaneous conditional rate \(\lambda(t)\), define accumulated hazard

\[
H=\int\_t^{t+\Delta}\lambda(s)\,ds.
\]

Under the corresponding survival model,

\[
P(\text{event during the interval})=1-e^{-H}.
\]

This is the basis of integrated-rate stochastic event scheduling. It is not generally valid to multiply a fine-step probability by the number of steps. [PLOS](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1004579)

For example, when an hourly conditional probability remains identical across 24 hours, the daily probability is

\[
p\_{\text{day}}=1-(1-p\_{\text{hour}})^{24},
\]

not \(24p\_{\text{hour}}\).

For implementation, `-expm1(-H)` evaluates \(1-e^{-H}\) accurately for small \(H\).

A useful TCE design is to retain a **residual integrated-hazard clock** for each relevant stochastic channel. Draw an exponential threshold once, consume hazard as time advances, and carry the remaining threshold across update boundaries. Switching modes should not redraw the person’s pending fate.

### Handle competing events jointly

Suppose death, illness, and departure are competing first events with constant rates \(\lambda\_1,\lambda\_2,\lambda\_3\). Let

\[
\Lambda=\lambda\_1+\lambda\_2+\lambda\_3.
\]

The probability of any event before the interval ends is \(1-e^{-\Lambda\Delta}\); conditional on that first event occurring, its cause has probability \(\lambda\_k/\Lambda\).

After it occurs, recompute the subsequent state and rates. Independent daily rolls followed by arbitrary conflict resolution can create a different stochastic process.

### Choose distributions that match the mechanism

A binomial count is appropriate for a finite set of conditionally independent, exchangeable trials. A Poisson count describes a different mechanism. Neither automatically represents heterogeneous people competing for finite supplies.

Tau-leaping literature specifically addresses problems such as negative populations and excessive event counts, including bounded alternatives and hybrid handling of critical reactions. TCE’s corresponding failures are negative grain, duplicated money, multiple buyers receiving the same item, or more successful workers than available jobs. [PubMed](https://pubmed.ncbi.nlm.nih.gov/16108628/)

**Do not sample unconstrained outcomes and repair them by clamping afterward.** The repair changes the distribution.

## 2.2 The main sources of drift

### Lost chronology

Two days can have identical totals and different consequences.

Consider a household with no morning food and a delivery at sunset. A daily calculation that pools all arrivals before all consumption implicitly allows the household to use future resources.

The same issue occurs with labor, travel, disease progression, credit, guarding, and conflict. Preserve causal order wherever an earlier outcome changes later opportunities.

### Lost covariance and shared shocks

Independent person-day draws erase common causes: weather, household exposure, workplace closure, festivals, market shortages, or a single road blockage.

As a mathematical illustration, because \(1-e^{-H}\) is concave,

\[
E[1-e^{-H}] \leq 1-e^{-E[H]}.
\]

Thus, under this hazard model, replacing heterogeneous exposure with its mean overestimates average infection probability. Matching mean exposure is not sufficient.

Likewise, replacing stochastic production by expected production suppresses variance. That can remove famines, bankruptcies, and migration waves even when average annual output matches.

### Lost memory

Disease duration, unemployment duration, relationship history, and time spent waiting may affect future behavior. Redrawing daily from a memoryless distribution can destroy those dependencies.

For non-exponential waiting times, retain elapsed time, the appropriate conditional duration state, or a previously sampled completion time.

### Threshold and feedback errors

Small short-term errors can cross consequential thresholds: a household sells its land, a settlement starves, a leader loses support, or a first technology becomes viable.

Matching one-day averages therefore does not establish consistency over centuries. Conversely, different individual trajectories do not alone prove statistical bias. The acceptance criterion must distinguish those two issues.

## 2.3 What published performance evidence actually supports

| Precedent | Reported performance | What it does—and does not—establish |
| --- | --- | --- |
| **Covasim, 2021 paper** | Approximately **7 million simulated person-days/second on one Intel i9-8950HK core**, with roughly **1 KB per agent**. | Demonstrates very high throughput for array-oriented daily individual simulation. It does not include TCE’s explicit movement, general economy, or open-ended institutions. [PLOS](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1009149) |
| **Temporal Gillespie, 2015** | Typically **10–100× faster than rejection sampling** on the empirical temporal-network cases studied. | Strong evidence for replacing repeated event checks with suitable event-driven algorithms—not a whole-engine speedup estimate. Read the paper with its **2019 pseudocode correction**. [PLOS](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1004579) |
| **SUMO mesoscopic simulation** | Documentation reports **up to 100× faster** than microscopic simulation. | An upper-end documentation claim, not a fixed reproducible benchmark for TCE. The mesoscopic traffic model changes resolution and assumptions; speed does not establish equivalence. [Eclipse SUMO](https://sumo.dlr.de/docs/Simulation/Meso.html) |

These are useful precedents, but there is **no verified benchmark here for TCE’s complete workload on the specified PC**.

## 2.4 Translate acceleration goals into a daily budget

The following are calculations, not measured performance. Assuming 365-day years:

| Average wall time per simulated day | Days advanced per second | Time to simulate 100 years |
| --- | --- | --- |
| 50 ms | 20 | 30.4 minutes |
| 10 ms | 100 | 6.1 minutes |
| 1 ms | 1,000 | 36.5 seconds |

A century in one minute requires approximately **608 simulated days/second**, or **1.64 ms per day**.

At 50,000 people, that century contains **1.825 billion person-days**.

The potential savings from removing polling are substantial: a hypothetical minute-by-minute loop performs 72 million person checks per day, versus 50,000 daily checks. But even eight meaningful transitions per person still produce 400,000 transitions per day. Those are work counts, not speedup predictions.

Also separate two budgets:

**60 fps permits 16.67 ms per rendered frame. It does not require a simulated day to finish inside that frame.** A simulation worker can advance independently while Unreal renders the latest completed snapshot.

---

# 3. Precedents: what shipped systems demonstrate

## 3.1 Paradox: fixed semantic ticks and multi-rate tasks

Paradox’s February 2023 Victoria 3 performance diary documents these simulation ticks:

| Game, as described in that diary | Simulation time per tick |
| --- | --- |
| Crusader Kings III | One day |
| Europa Universalis IV | One day |
| Hearts of Iron IV | One hour |
| Victoria 3 | Six hours |

Victoria 3 additionally schedules yearly, monthly, weekly, daily, and regular tasks, with explicit dependencies. The diary describes expensive weekly work, parallel preparation followed by serial updates, and sorting to prevent execution-order inconsistencies. These observations apply to the **Victoria 3 1.1–1.2 development period**, not every 2026 build. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-76-performance)

**Lesson for TCE:** a speed control need not change the simulation’s mathematical resolution. Different subsystems can already have different semantic cadences, independent of wall-clock speed. The diary does not establish a separate statistical high-speed world model.

## 3.2 Dwarf Fortress: abstraction is useful, but not evidence of equivalence

Bay 12’s December 2013 development logs describe three resolutions: objects in active play, abstract building/dungeon populations, and abstract site populations. They also discuss transferring populations between representations, population-duplication problems, and differences between world-generation advancement and continued advancement of an existing world. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2013.html)

**Lesson for TCE:** historical simulation and local activity can coexist at different resolutions, but the difficult part is preserving identity, ownership, and meaningful historical consequences during transitions.

Dwarf Fortress is therefore an important architectural precedent—not proof that its historical and locally detailed processes sample the same transition distribution. The cited evidence is historical development documentation, not a claim about current Premium-edition internals.

## 3.3 Cities: Skylines: speed settings are not an approximation contract

The launch-era **Cities: Skylines I** manual labels its speeds **1×, 2×, and 4×**. These are documented nominal controls, not a guarantee about achieved speed under load or every later patch’s internals. [Steam CDN](https://cdn.akamai.steamstatic.com/steam/apps/255710/manuals/CitiesSkylines-UserManual_EN.pdf)

The June 2023 **Cities: Skylines II** traffic developer diary describes consequential individual pathfinding, lane decisions, rerouting, and multicore processing. It does not document an equivalent daily statistical replacement for those interactions. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai)

**Lesson for TCE:** accelerating detailed traffic and statistically replacing traffic are different engineering problems. A faster speed button is not evidence that the second problem has been solved.

## 3.4 SimCity / Micropolis: a particularly useful counterexample

The public **SimCity-derived Micropolis** `simulate.cpp` does two distinct things.

First, it throttles execution according to the selected speed. Second, it changes certain subsystem refresh periods with that speed. The inspected code uses crime-scan periods of **1, 8, and 18 simulation cycles**, and pollution/terrain/land-value scan periods of **2, 7, and 17 cycles**. It also distributes simulation work across 16 phases. [Micropolis Web](https://micropolisweb.com/doc/simulate_8cpp_source.html)

This is not simply “run the identical process faster.” Some information becomes staler relative to simulation time at higher speeds.

**Lesson for TCE:** phase-based scheduling is useful, but tying model refresh intervals to the speed selector introduces an obvious consistency risk. The code establishes the scheduling difference; it does not quantify its eventual gameplay bias.

For a separate SimCity generation, Andrew Willmott’s **GDC 2012 “Inside GlassBox”** is a relevant architecture talk for the engine behind SimCity 2013. It should not be conflated with the Micropolis code lineage. [Andrew Willmott](https://www.andrewwillmott.com/talks/inside-glassbox)

## 3.5 Open-source simulation: borrow specific mechanisms

**SUMO** preserves individual vehicles in its mesoscopic model while replacing detailed vehicle dynamics with queue-based movement through road segments. Its implementation is a strong reference for retaining trips and congestion without simulating every acceleration maneuver. It is not evidence of exact live micro/meso switching. [Eclipse SUMO](https://sumo.dlr.de/docs/Simulation/Meso.html)

**JumpProcesses.jl** provides useful examples of rates separated from state-changing effects, dependency-aware event algorithms, and multiple stochastic scheduling strategies. That separation is more relevant to TCE than adopting Julia as the production runtime. [SCIML Documentation](https://docs.sciml.ai/JumpProcesses/stable/jump_types/)

---

# 4. Recommended architecture for TCE

## 4.1 Keep one authoritative model at every speed

I recommend this separation:

```
Authored behavioral and physical rules
                  ↓
Persistent Rust world state
                  ↓
Shared event / hazard / transaction machinery
                  ↓
Execution strategy:
    scheduled events + analytical advances + validated batches
                  ↓
Timestamped snapshots and event records
                  ↓
Unreal presentation and interaction
```

The crucial boundary is between **consequential simulation** and **visual realization**.

A resident’s destination, departure time, arrival delay, work interval, purchases, and exposure opportunities can all be genuine sub-daily simulation. Their gait cycle and incidental animation need not be.

For strict speed independence, the low-speed view should reveal the authoritative activity process—not replace it with a different causal process. For example, decorative local avoidance should not create infections that only exist while the camera is nearby.

Where detailed geometry genuinely determines an outcome, retain that geometry in the authoritative model or explicitly validate its replacement. Do not silently demote consequential physics to decoration.

### Preserve the state that makes the next day predictable

At minimum, I would retain:

| Domain | Persistent information |
| --- | --- |
| People and relationships | Stable identity, household, workplace, important social edges, behavioral state |
| Activities | Current plan, active segment, elapsed progress, next interruption/completion |
| Economy | Ownership, balances, inventories, reservations, debts and pending transfers |
| Movement | Individual trip, route, departure/arrival state, relevant queue position |
| Stochastic processes | Residual hazard clocks, duration state, semantic event counters |
| Daily accounting | Accumulated work, consumption, exposure, travel, and partial-day totals |

This avoids destructive conversion between a “real person” and an unrelated daily statistical record.

### Give authored building blocks an execution contract

Each rule should declare its prerequisites, state dependencies, rate units, resource transfers, interruption conditions, and batching assumptions.

For example, “harvest grain” should specify whether work can proceed while hungry, what happens when storage fills, which inputs are reserved, and what invalidates the completion estimate.

Require every optimized implementation to state whether it is **exact relative to the reference rule** or **approximate within a declared domain**. A newly authored technology or law should invalidate affected caches and approximation assumptions, rather than silently using parameters calibrated for an earlier society.

## 4.2 Make a day an advance horizon, not one indivisible operation

A conceptual scheduler—not production code—looks like this:

```
advance_until(target_time):
    while simulation_time < target_time:
        boundary = earliest of:
            target_time
            scheduled consequential event
            known rate/dependency change
            resource or capacity boundary
            permitted approximation horizon

        advance eligible processes to boundary
        resolve events and constrained transactions at boundary
        commit a consistent state
        invalidate affected predictions
```

Use three categories of work:

| Category | Examples | Policy |
| --- | --- | --- |
| **Exactly skippable or integrable** | Aging; constant-rate progress; analytical decay; scheduled completion | Batch aggressively until a dependency changes. |
| **Statistically batchable** | Numerous routine transitions under stable conditions | Use a validated sampler with explicit bounds and resource constraints. |
| **Chronology-sensitive** | Resource exhaustion, contagious transitions, fire spread, pivotal decisions, conflicting transactions | Process causal boundaries explicitly or use an equivalent joint method. |

“Rare” is not sufficient to justify averaging. A first invention or a founder’s death may require little computational work but have enormous historical consequences.

Also, an exactly scheduled event can still have an incorrectly approximated rate. Audit its dependencies, not just its scheduler.

### Switching modes

Initially, implement switching at day boundaries. Then support arbitrary boundaries while preserving the active plan, reservations, partial totals, and residual random clocks.

Never restart someone’s day because the user slowed down. Never reroll unfinished work because the user accelerated.

**The safest default is that approximation tolerances depend on world state, not selected speed.** Under a crisis, allow achieved acceleration to fall rather than silently loosening the model.

## 4.3 Preserve interactions that disappear under daily aggregation

### Disease: retain contact structure and timing where consequential

For TCE, I would represent disease-relevant contact through persistent household/work networks plus time-bounded occupancy at markets, institutions, and public places.

Daily accumulated exposure can be sufficient in some regimes. A study using measured conference contacts found that daily networks retaining contact durations could reproduce important SEIR outcomes for the timescales studied, whereas more homogeneous representations lost accuracy. This is conditional evidence, not permission to average all diseases into one daily citywide mixing rate. [arXiv](https://arxiv.org/abs/1108.4841)

Retain repeated contacts, exposure heterogeneity, infectiousness timing, and shared venue events.

Chronology can matter even when total contact durations match: B meeting C before being infected by A differs from B meeting C afterward. Long latent periods can make that distinction irrelevant for a particular day; a faster process may require internal subdivisions.

The same activity records should support disease, rumor transmission, and social interaction. Separate subsystems should not invent mutually inconsistent populations at the same location.

### Traffic: preserve capacity and arrival consequences

Use **individual trips over a mesoscopic network** as the initial authoritative model. Preserve departure windows, route choice, finite capacity, queues, spillback where needed, and arrival/completion events.

Do not replace congestion by a fixed average commute penalty. The same number of daily trips can either fit comfortably or overwhelm a bridge, depending on timing.

Workers should begin work when their authoritative travel process gets them there. Emergency response, purchases, and encounters should consume those same arrival times.

At low speed, render movement consistent with this model. Add more detailed traffic physics only after deciding whether it is presentation or a change in authority.

### Crime: model opportunity overlap, not only daily totals

For TCE, a reasonable proposed abstraction is to integrate opportunities over simultaneous offender, victim, property, and guardian states.

The relevant quantity resembles

\[
\int O(t)V(t)[1-G(t)]\,dt,
\]

not a product of daily averages. An offender present in the morning and a victim present at night do not constitute an encounter.

A sampled incident should identify a feasible place, time, participants, and property. Consequential effects—injury, stolen goods, guard response, retaliation—must update the remaining day.

This preserves opportunities statistically without requiring every visually close pedestrian pair to trigger an expensive query.

## 4.4 Rust DLL and Unreal Engine 5.8 integration

Keep the domain kernel in a **headless Rust crate**, with a thin DLL interface. Unreal should not own or independently update authoritative person state.

Use a versioned C ABI, opaque world handles, and explicitly laid-out transfer structures. Do not pass Rust `Vec`, `String`, references, or trait objects across the interface. Allocate and free buffers on the same side, and prevent Rust panics or C++ exceptions from unwinding across an incompatible ABI boundary. Rust’s FFI documentation covers these representation and unwinding constraints. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

For Unreal’s side, use documented DLL loading and packaged-build staging. The UE 5.8 integration documentation covers `FPlatformProcess::GetDllHandle` and `RuntimeDependencies`; a DLL that works in the editor is not automatically staged correctly in a packaged build. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine)

My implementation choices would be:

**Ownership and threading.** One simulation coordinator owns authoritative progression. Parallel workers calculate independent proposals or homogeneous batches from consistent inputs. Resolve conflicting transfers before committing. Avoid making every agent a concurrent writer to shared markets.

**Presentation.** Publish immutable, timestamped snapshots through double or triple buffering. Unreal consumes them without holding the world lock throughout rendering. Use a data-oriented presentation layer, potentially including MassEntity, rather than a heavy independently ticking Character for every resident. MassEntity’s documented role is data-oriented entity processing; using it as an adapter does not require making it the simulation authority. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-entity-in-unreal-engine)

**Responsiveness.** Advance in bounded work chunks so pause and input remain responsive. Assign player commands explicit simulation timestamps; do not accidentally defer every command to the next midnight.

**Hardware allocation.** Start with CPU execution. Reserve the RTX 4070 Ti primarily for rendering rather than assuming a branch-heavy civilization kernel will benefit from GPU migration. Benchmark Rust and Unreal together, with a bounded worker pool to avoid oversubscription.

**Lifetime safety.** Stop workers and release outstanding snapshot references before unloading the DLL. Treat hot reload as a separate engineering feature, not something Unreal Live Coding automatically solves for Rust-owned world state.

The hardware is a useful target configuration, but **60 fps at 1440p cannot be guaranteed without profiling the actual scene, representations, animation, lighting, and simulation contention**.

## 4.5 Test consistency with two separate gates

### Gate A: exact execution equivalence

For optimizations claimed to preserve individual history, start from the same world and command timeline, then run:

* Detailed advancement, accelerated advancement, and mixed sequences.
* Different advance-call partitions, including mid-day boundaries.
* Save/load, different rendering rates, and different camera positions.

Compare canonical authoritative state and consequential event records at common simulation times. Include hidden state such as queues, clocks, reservations, and partial-day accumulators—not merely visible totals.

The key metamorphic property is:

\[
\operatorname{advance}(a+b)
\equiv
\operatorname{advance}(a);\operatorname{advance}(b),
\]

under the same semantic randomness and input timing.

Use strict numerical tolerances where floating-point evaluation prevents a legitimate bitwise contract. A deterministic headless reference remains valuable even when cross-machine bitwise reproducibility is not a product requirement.

Also enforce conservation: transfers balance, inventories remain valid, people are uniquely owned, births/deaths/migration reconcile, and an agent cannot perform more activity than elapsed time permits.

### Gate B: statistical equivalence

For approximations, compare ensembles against the reference model. Define acceptance margins **before** evaluating results.

For example, “mean annual food differs by less than 1%” and “outbreak probability differs by less than two percentage points” are possible product-defined criteria—not universal scientific standards.

A nonsignificant difference is not evidence of equivalence. Equivalence testing instead asks whether uncertainty in the difference fits inside prespecified acceptable bounds. Lakens’ practical treatment of TOST explains this distinction. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5502906/)

Test more than means:

| Domain | Important comparisons |
| --- | --- |
| Demography | Mortality distribution, household survival, settlement extinction |
| Economy | Wealth distribution, shortages, bankruptcies, output variance |
| Disease | Outbreak probability, peak burden, final size, clustering |
| Movement | Arrival-delay distribution, peak queues, inaccessible destinations |
| Social development | Innovation waiting times, adoption paths, regime-transition frequencies |

Begin with a pilot ensemble, then size experiments for the differences and rare-event probabilities that matter. A few dozen seeds cannot certify extremely rare historical failures.

### Couple randomness deliberately

Use independently addressable random streams keyed by semantic identity: world seed, subsystem, person or pair, event family, and occurrence.

Counter-based generators such as Random123’s Philox and Threefry provide a suitable foundation for addressable parallel randomness. [The Salmons](https://www.thesalmons.org/john/random123/)

Do **not** key randomness by frame number, worker index, speed setting, or number of calls to `advance`.

Counter-based RNG alone does not make different algorithms equivalent. A coarse Poisson draw and several fine draws require a consistent coupling or conditional splitting strategy. Simply reusing the same seed does not preserve the same underlying event process.

### Test difficult states and long horizons

Include scarce grain, one infectious arrival, a closed bridge, a nighttime crime opportunity, a leader dying mid-day, abrupt migration, and newly introduced technologies or laws.

Draw test checkpoints from **both** modes. Otherwise, the fast mode may enter states that the validation suite never examined.

For debugging, rerun short intervals from checkpoints, compare event traces, and locate the first consequential difference. For performance, report median and tail day times alongside frame times and achieved days per second.

Finally, convergence between modes is not validation against reality. An approximate fine reference can be wrong too. Test its own timestep sensitivity and behavioral assumptions.

## 4.6 Development sequence for a solo developer

Build the headless reference, conservation ledger, and replayable tests first. Add exact event scheduling, analytical updates, and data-oriented storage before introducing approximate daily samplers. Then integrate Unreal snapshots and measure the combined workload.

Only after profiling should you approximate one expensive subsystem at a time.

For AI coding agents, make acceptance criteria explicit: an analytical unit case, a conservation test, a partition/switching test, dependency-invalidation cases, and a benchmark. “Compiles and looks plausible” is not enough for a multi-rate kernel.

---

# 5. Linked sources and version applicability

These are the most useful starting points for implementation. Rolling documentation and source snapshots should be pinned to specific revisions when TCE adopts them.

| Source | Applies to / why it matters |
| --- | --- |
| [Banisch, Lima & Araújo — *Agent Based Models and Opinion Dynamics as Markov Chains*](https://arxiv.org/abs/1108.1716?utm_source=chatgpt.com) | 2011 preprint. Micro-to-macro projection and conditions for a closed coarse process. |
| [Anderson, Ganguly & Kurtz — *Error analysis of tau-leap simulation methods*](https://arxiv.org/html/0909.4790v3?utm_source=chatgpt.com) | arXiv v3, 2012; published analysis. Strong versus weak error and stochastic time-change formulations. |
| [Cao, Gillespie & Petzold — avoiding negative populations in tau-leaping](https://pubmed.ncbi.nlm.nih.gov/16108628/?utm_source=chatgpt.com) | 2005 paper. Critical events, finite populations, and safer batching. |
| [Gillespie et al. — *The subtle business of model reduction for stochastic chemical kinetics*](https://pubmed.ncbi.nlm.nih.gov/19222263/?utm_source=chatgpt.com) | 2009 paper. Conditions and hazards of slow-scale reduction. |
| [Cisternas et al. — *Coarse-grained analysis of stochastic individual-based models*](https://arxiv.org/abs/nlin/0310011?utm_source=chatgpt.com) | 2003 preprint. Equation-free and coarse projective methods. |
| [Temporal Gillespie algorithm](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1004579&utm_source=chatgpt.com) and [2019 correction](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1007190&utm_source=chatgpt.com) | 2015 paper with supplementary code; implement with the correction, not the original pseudocode alone. |
| [Stehlé et al. — high-resolution contact patterns and infection spread](https://arxiv.org/abs/1108.4841?utm_source=chatgpt.com) | 2011 study. Evidence for, and limitations of, daily weighted contact representations. |
| [SUNDIALS ARKODE mathematics](https://sundials.readthedocs.io/en/latest/arkode/Mathematics_link.html?utm_source=chatgpt.com) | Rolling documentation inspected in 2026. Multi-rate integration and coupling. |
| [JumpProcesses.jl documentation](https://docs.sciml.ai/JumpProcesses/stable/jump_types/?utm_source=chatgpt.com) | Rolling stable documentation inspected in 2026. Rates, effects, dependency graphs, and event algorithms. |
| [Covasim paper](https://journals.plos.org/ploscompbiol/article?id=10.1371%2Fjournal.pcbi.1009149&utm_source=chatgpt.com) and [code](https://github.com/InstituteforDiseaseModeling/covasim) | 2021 benchmark; article discusses v2.1.1. Daily individual simulation and array-oriented implementation. |
| [SUMO mesoscopic documentation](https://sumo.dlr.de/docs/Simulation/Meso.html?utm_source=chatgpt.com) | Meso available publicly since SUMO 0.26.0; rolling documentation inspected in 2026. Individual vehicles with queue-based dynamics. |
| [Victoria 3 Dev Diary #76](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-76-performance?utm_source=chatgpt.com) | February 22, 2023; Victoria 3 1.1–1.2 period. Tick structure, dependencies, and performance engineering. |
| [Bay 12 development logs](https://www.bay12games.com/dwarves/dev_2013.html?utm_source=chatgpt.com) | December 2013 entries. Historical evidence on multiple population resolutions and transition problems. |
| [Cities: Skylines I manual](https://cdn.akamai.steamstatic.com/steam/apps/255710/manuals/CitiesSkylines-UserManual_EN.pdf?utm_source=chatgpt.com) and [Cities: Skylines II traffic diary](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/traffic-ai?utm_source=chatgpt.com) | Launch-era 2015 manual; June 2023 developer account. Keep the games and version contexts separate. |
| [Micropolis simulation source](https://micropolisweb.com/doc/simulate_8cpp_source.html?utm_source=chatgpt.com) and [repository](https://github.com/SimHacker/MicropolisCore?utm_source=chatgpt.com) | Public SimCity-derived source inspected in 2026; no immutable revision established for the Doxygen snapshot. |
| [Andrew Willmott — *Inside GlassBox*](https://www.andrewwillmott.com/talks/inside-glassbox?utm_source=chatgpt.com) | GDC 2012 talk, with presentation links. Separate SimCity 2013 architecture reference. |
| [Random123](https://www.thesalmons.org/john/random123/?utm_source=chatgpt.com) and [code](https://github.com/DEShawResearch/random123?utm_source=chatgpt.com) | SC 2011 research and continuing implementation. Counter-based random-number generation. |
| [Lakens — *Equivalence Tests: A Practical Primer*](https://pmc.ncbi.nlm.nih.gov/articles/PMC5502906/?utm_source=chatgpt.com) | 2017 paper. Prespecified equivalence margins and appropriate statistical interpretation. |
| [UE third-party library integration](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com), [MassEntity overview](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-mass-entity-in-unreal-engine?utm_source=chatgpt.com), and [Rust FFI guidance](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | UE 5.8 documentation and current Rust documentation inspected in 2026. Pin the Rust toolchain and ABI in the project. |

## Bottom line

**TCE’s best route to consistent acceleration is not to replace an agent’s simulated day with an independently sampled summary. It is to represent that day with the smallest set of activities, events, constraints, and accumulated quantities that determine its consequences.**

Execute those same semantics at every speed. Skip idle work, integrate safe intervals, batch genuinely equivalent operations, and retain internal event boundaries where causality demands them.

A day can be the unit of advancement and reporting. **It should not be the universal unit of causality.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab926b4-fc68-83ea-80e8-78c84f1fcf77)
