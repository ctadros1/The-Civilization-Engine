# Validating TCE without building a research program

**Recommendation:** build a small, headless **plausibility regression harness** around the Rust kernel. Combine exact accounting checks, a dozen explicitly scoped behavioral patterns, reproducible scenario tests, structured visual inspection, and occasional parameter screening. Keep sophisticated calibration outside the shipping application—and use it only when a specific modeling problem warrants it.

The central distinction is between **verification**—whether the implementation follows its rules—and **validation**—whether those rules adequately represent the intended phenomena. Agreement with a previous build establishes neither historical realism nor correctness by itself. ABM validation literature treats these as complementary activities rather than interchangeable tests. [Cambridge University Press](https://www.cambridge.org/core/books/abs/agentbased-models-in-economics/empirical-validation-of-agentbased-models/AD71080C74F834D347E91D651CC6F1C9)

For TCE, the useful claim is:

> “Within these environmental, technological, and population conditions, the simulation satisfies its accounting constraints, reproduces several independently motivated patterns, and responds credibly to controlled changes.”

That is more defensible—and more actionable—than “the civilization simulator is validated.”

**Evidence and version scope:** sources were checked on September 27, 2026. The implementation and run-budget recommendations below are proposals for TCE, not measurements of its kernel or promises about your hardware.

---

## 1. Options: what each method actually establishes

### 1.1 Pattern-oriented modeling: the best organizing principle

Pattern-oriented modeling, or **POM**, uses several observed patterns at different scales to guide model structure, choose between mechanisms, and constrain parameters. Its important contribution is not merely checking many statistics: it asks whether the **same underlying rules jointly explain multiple observations**. Grimm and colleagues developed the framework in ecology; Grimm and Railsback’s later synthesis explicitly connects individual adaptive behavior to population and system-level outcomes. [Science](https://www.science.org/doi/10.1126/science.1116681)

For TCE, adopt the method without adopting the workload of a research project.

Start with approximately **8–12 pattern cards**, not hundreds of dashboard indicators. Each card should record:

| Field | What TCE should record |
| --- | --- |
| Scope | Environmental conditions, available technologies, institutional assumptions, population scale |
| Observable | Precisely defined quantity, units, spatial aggregation, and observation period |
| Expected pattern | A range, relationship, distribution shape, or qualitative response |
| Evidence | Historical data, published mechanism, or explicitly labeled modeling assumption |
| Acceptance rule | What counts as a match; how seeds and uncertainty are handled |
| Role | Used for calibration, held out for validation, or diagnostic only |
| Explanation | Which mechanisms could produce the pattern—and which alternatives might also do so |

The ODD model-description protocol’s 2020 update is particularly useful here: it explicitly includes “purpose and patterns,” permits qualitative but testable expectations, and asks authors to document their evidential basis. It also warns against reporting only patterns that the completed model happened to reproduce. [JASSS](https://www.jasss.org/23/2/7.html)

**Proposed starting pattern families for TCE**

These are test-design suggestions, not universal historical laws or supplied historical calibration ranges.

| Scale | Proposed observation | Why it is useful |
| --- | --- | --- |
| Person | Feasible combinations of work, travel, sleep, and other activities | Detects plausible-looking aggregate output produced by impossible individual schedules |
| Household | Food shortages and reserves distributed across households, not just world-average food | Distinguishes abundant production from effective access and distribution |
| Settlement | Seasonal production, storage, consumption, and shortage timing under seasonal agriculture | Exposes missing storage, timing, or labor-allocation mechanisms |
| Settlement network | Trade volumes and delivery delays conditional on route costs, capacity, and accessibility | Tests the link between economic decisions and physical geography |
| Institutions | Revenue, enforcement workload, compliance, and administrative reach under specified institutions | Tests operational feasibility without prescribing monarchy, republic, or any inevitable progression |
| Technology and buildings | Adoption and construction conditional on knowledge, materials, labor, and contact opportunities | Detects instantaneous diffusion and “free” development |

The strongest tests cross scales. A settlement might have a believable food surplus while households routinely starve because ownership or distribution is broken. Conversely, everyone might survive only because the simulation silently creates food.

**Do not require every world to urbanize, industrialize, or converge on the same institutions.** For TCE, I recommend conditioning expectations on capabilities and circumstances rather than elapsed world year. A resource-poor world remaining small can be legitimate; unexplained survival without sufficient resources is a different issue.

**Cost and limitation:** POM is cheap to execute once observables exist. The main work is deciding which patterns are genuinely informative. Several variations of “population increased” are not independent evidence.

### 1.2 Invariants and property-based tests: the cheapest dependable foundation

These establish implementation consistency, not empirical realism.

For TCE, make stock-flow accounting executable:

\[
\text{closing stock}
=
\text{opening stock}
+\text{production}
+\text{imports}
-\text{consumption}
-\text{spoilage}
-\text{exports}
-\text{other recorded losses}.
\]

Apply the same discipline to population, ownership transfers, construction materials, and inventories.

Important qualifications:

* Food is not conserved when produced or consumed; **the accounting identity** is conserved.
* Money should be conserved only in fixtures that explicitly exclude issuance, destruction, and other monetary mechanisms.
* Population should reconcile with births, deaths, immigration, and emigration—not remain constant.

Add structural checks: no duplicate ownership where exclusivity is intended, no completed construction before required work, no use-after-death actions, and no information-dependent action without a valid knowledge source.

Rust’s **proptest** generates many inputs, reduces failing cases to smaller counterexamples, and can persist failures for subsequent runs. This is a strong fit for inventories, transactions, schedules, serialization, and action preconditions. [Docs.rs](https://docs.rs/proptest/latest/proptest/)

**Cost and limitation:** local assertions can be inexpensive; whole-world reconciliation is not free. Use incremental checks during normal operation and independent full audits at selected checkpoints. A checker that reuses the same erroneous accounting function can reproduce the same bug.

### 1.3 Metamorphic tests: test relationships when exact outcomes are unknowable

An emergent world rarely has a known “correct final population.” But TCE can have known relationships between carefully constructed runs.

I recommend these paired tests:

| Transformation | Expected relationship |
| --- | --- |
| Change camera, graphics quality, render frame cap, or renderer availability | Identical authoritative kernel outcome for identical logical inputs |
| Enable additional diagnostics | No change to simulated decisions or random-number consumption |
| Save, reload, then continue | Same continuation under the supported deterministic configuration |
| Remove the only transport connection | No deliveries across that disconnected boundary unless another explicitly modeled transport mode exists |
| Disable all knowledge-transfer routes in a controlled fixture | No diffusion through those routes |
| Reduce the simulation time step | Convergence of relevant outcomes within an explicitly chosen error envelope—not necessarily identical trajectories |

Scope these relations carefully. “Higher taxes always increase revenue” or “more food always increases population” are not safe universal assertions for an adaptive society.

For event hazards, a useful internal consistency check is that a constant hazard \(\lambda\) over interval \(\Delta t\) corresponds to:

\[
p(\Delta t)=1-e^{-\lambda\Delta t}.
\]

Changing the tick duration while leaving a per-tick probability unchanged changes the model. Faster headless execution should initially mean **removing rendering and waiting**, not enlarging the simulation time step.

**Cost and limitation:** generally one or a few extra runs per relation. The difficult part is specifying a relation that really follows from the fixture’s assumptions.

### 1.4 Face validation: structured inspection, not “it looks convincing”

Face validation asks whether knowledgeable observers find the model and its behavior credible. It is one component of broader validation practice, not a substitute for empirical or implementation checks. [JASSS](https://www.jasss.org/27/1/11.html)

For a solo developer, I recommend a **15–30-minute review session** at behavioral milestones:

Inspect one ordinary household, one unusually successful household, and one struggling household. Follow a small number of consequential events through their actual decision traces. Compare old and new builds without labels when practical.

Use the same questions each time:

> Could this person know this? Could they afford it? Could they reach it? Did they have enough time? Were the alternatives genuinely available? Do the resulting consequences follow from the recorded rules?

Show both random samples and automatically flagged anomalies. Inspecting only dramatic events selected for the chronicle will bias the review toward convincing stories.

AI coding agents can help locate anomalies and summarize traces. For TCE, however, an explanation should reference recorded inputs, contributions, decisions, and event identifiers. **A fluent invented rationale must not become validation evidence.**

**Cost and limitation:** low implementation cost if provenance already exists; continuing human attention is required. Familiarity and present-day cultural intuitions should not silently become universal historical assumptions.

### 1.5 Docking: compare with an independent implementation

“Docking” means aligning two models closely enough to compare their behavior. Depending on the models, the target can be numerical agreement, distributional agreement, or agreement on qualitative relationships.

The classic Axtell–Axelrod–Epstein–Cohen study found that activation scheduling mattered: updating agents with versus without replacement helped explain apparently inconsistent results. Aligning those details resolved the discrepancy. The same paper cautions that failing to detect a statistical difference is not evidence of equivalence when tests have insufficient power. [UMich Personal](https://www-personal.umich.edu/~axe/research/Aligning_Sim.pdf)

**Do not build a second civilization simulator.** Dock small, consequential subsystems:

* An independent spreadsheet-sized or Python stock-flow model against Rust inventory behavior.
* A tiny demographic fixture against an analytical expectation or independent cohort calculation.
* A slow, obvious reference implementation against an optimized matching or allocation algorithm.

Match initial conditions, boundaries, update order, units, and stochastic assumptions before interpreting disagreement.

**Cost and limitation:** potentially excellent value for critical subsystems, but expensive for entire worlds. Two implementations can agree because they share the same mistaken assumptions; docking establishes alignment, not historical truth.

### 1.6 Sensitivity analysis: find what deserves attention

Sensitivity analysis asks which inputs or structural choices materially affect outputs. It does not establish that the resulting behavior is realistic.

For TCE, use a progression:

**Local perturbations.** Change one parameter above and below the current value. This is useful for detecting disconnected parameters, reversed effects, and fragile tuning. It examines only a small neighborhood.

**Morris screening.** Evaluate one-at-a-time changes along several trajectories through parameter space. The mean absolute elementary effect, \(\mu^\*\), summarizes influence; variation in effects helps identify behavior requiring further investigation. In a stochastic model, that variation can also reflect simulation noise, so it must not automatically be labeled “interaction.” SALib provides the sampler and analysis. [Salib](https://salib.readthedocs.io/en/stable/api/SALib.analyze.html)

**Sobol analysis.** Estimate contributions to output variance, including total effects involving interactions. This is useful when you need a defensible variance decomposition, but requires substantially more runs. The standard setup also needs care with dependent inputs and stochastic output noise. [Salib](https://salib.readthedocs.io/en/stable/api/SALib.sample.html)

Include **structural sensitivity**, not only sliders: alternative decision rules, scheduler ordering, information assumptions, time step, and world boundaries.

A sharp transition is not automatically a bug. The question is whether it has a credible mechanism or is an accidental numerical threshold.

---

## 2. Calibration and computational trade-offs

### 2.1 Which calibration methods are cheap enough?

Calibration selects parameter values. Testing those values against the same observations used to select them is not independent validation; multiple parameterizations or mechanisms may explain similar aggregate outcomes. [JASSS](https://www.jasss.org/10/2/8.html)

| Method | How it works | Recommended role in TCE |
| --- | --- | --- |
| Direct anchoring | Set measurable quantities from evidence and explicit units | Default for physical quantities; do not distort them to repair unrelated aggregate behavior |
| Manual/local search | Adjust a few parameters and inspect scoped effects | Routine diagnosis; keep changes and reasons recorded |
| Random or Latin-hypercube sampling | Evaluate a bounded collection of parameter combinations | Best initial automated search; retain several acceptable configurations |
| Differential evolution or CMA-ES | Search using a population of candidate parameter vectors | Occasional use for a small, stubborn calibration problem |
| Sequential optimization, such as TPE or Gaussian-process search | Use previous evaluations to propose subsequent candidates | Consider when each run is expensive and the objective is reasonably stable |
| Approximate Bayesian computation and related inference | Compare simulated and observed summaries while accounting for parameter uncertainty | Research-grade option, not the default maintenance workflow |
| Surrogate models | Approximate simulation outputs with a cheaper learned model | Defer until repeated expensive studies justify another model that itself needs checking |

SciPy’s differential-evolution documentation explicitly warns about evaluation cost. With 20 free parameters, its documented default population and iteration settings permit **300,300 objective evaluations before polishing**—calculated from the documented formula. Averaging each objective over several worlds multiplies that workload. Do not launch optimizer defaults blindly. [SciPy Documentation](https://docs.scipy.org/doc/scipy/reference/generated/scipy.optimize.differential_evolution.html)

Optuna provides random, TPE, CMA-ES, and Gaussian-process samplers, among others. Its pruning features stop trials using intermediate results; for TCE, I would initially prune only crashes and hard violations. A world that develops slowly is not necessarily an unpromising final outcome. [Optuna](https://optuna.readthedocs.io/en/stable/tutorial/10_key_features/003_efficient_optimization_algorithms.html)

### 2.2 A practical calibration loop

My recommended starting procedure is:

1. **Freeze the specification first.** Define credible parameter bounds, fitting patterns, held-out patterns, and held-out environmental conditions.
2. **Limit the search.** Work on roughly 5–10 uncertain parameters in one subsystem or tightly coupled group.
3. **Screen cheaply.** Try 64 parameter vectors with three seeds each: **192 runs per scenario**.
4. **Confirm promising regions.** Test four retained configurations with 12 fresh seeds each: another **48 runs per scenario**.
5. **Check transfer.** Run finalists under held-out conditions and at relevant production populations.
6. **Retain a plausible set.** Prefer a reasonably broad acceptable region over a single unusually lucky optimum.

Those counts are starter budgets, not statistically sufficient sample sizes for every output. Increase replication when uncertainty prevents a decision.

A simple proposed interval-violation score is:

\[
v\_j(\theta)=
\frac{
\max\left(0,\ L\_j-\widetilde y\_j(\theta),\ \widetilde y\_j(\theta)-U\_j\right)
}{s\_j},
\qquad
J(\theta)=\sum\_j w\_jv\_j(\theta).
\]

Here, \([L\_j,U\_j]\) is a justified acceptable interval, \(\widetilde y\_j\) is a chosen ensemble summary, and \(s\_j\) gives a meaningful scale.

Use this only to rank candidates. **Hard violations must reject independently**, and the report should retain each metric’s result rather than hide everything inside \(J\). A good median must not conceal frequent collapse.

This is pragmatic constraint-based calibration, not a Bayesian posterior.

### 2.3 Run counts matter more than the optimizer’s label

The following is an **illustrative calculation**, not a TCE benchmark.

Assume 20 parameters and three stochastic repetitions per parameter point:

| Experiment | Calculation | World runs |
| --- | --- | --- |
| Central local perturbations | \((2k+1)s\) | 123 |
| 64-point parameter sample | \(64s\) | 192 |
| Morris, six trajectories | \(r(k+1)s\) | 378 |
| Sobol first and total effects, base size 256 | \(B(k+2)s\) | 16,896 |

The Sobol sampling factor follows SALib’s documented design with second-order indices disabled. Six Morris trajectories and three repetitions are screening choices here, not convergence guarantees. [Salib](https://salib.readthedocs.io/en/stable/api/SALib.sample.html)

At an **assumed** ten seconds per world and four perfectly utilized workers, those workloads would take approximately **5 minutes, 8 minutes, 16 minutes, and 11.7 hours**, respectively. Actual times include initialization, output, memory contention, and imperfect parallelism.

Measure:

\[
T\_{\text{batch}}\approx
\frac{N\_{\text{runs}}T\_{\text{run}}}{W\_{\text{effective}}}.
\]

NetLogo’s BehaviorSpace documentation explicitly notes that maximum concurrency can reduce performance and increase memory pressure. The same consideration should guide TCE’s worker count. [NetLogo 7.0.4 User Manual](https://docs.netlogo.org/behaviorspace.html)

### 2.4 A useful optimization: paired randomness

For comparing a baseline and an intervention, use paired scenarios with aligned random streams where possible.

The Starsim common-random-numbers paper reports **more than tenfold reductions in required simulation repetitions for some examples at a fixed standard error**. Benefits varied across examples and horizons. This is variance reduction in comparisons—not a tenfold improvement in simulation execution speed. The reported analysis used Starsim 1.0.1; the cited manuscript is a 2024 preprint. [arXiv](https://arxiv.org/html/2409.02086v2)

For TCE, consider stable streams associated with decision categories, logical time, and agent or event identity. A single global seed is insufficient when an added birth or branch shifts all later random draws.

Treat stream alignment as an engineering feature to test, not something guaranteed merely by using the same seed.

---

## 3. Precedents and what to borrow

| Precedent | What the source establishes | Lesson for TCE |
| --- | --- | --- |
| **Grimm et al.; POM, 2005/2012** | Multiple patterns can inform model structure, mechanism selection, and calibration | Spend effort selecting discriminating observations before choosing an optimizer. [Science](https://www.science.org/doi/10.1126/science.1116681) |
| **Axtell et al.; docking, 1996** | Apparently minor scheduling differences changed model comparisons | Record scheduler semantics as part of the model specification. [UMich Personal](https://www-personal.umich.edu/~axe/research/Aligning_Sim.pdf) |
| **Factorio, 2014 developer account** | Targeted automated tests helped prevent desynchronization bugs and enabled more confident changes despite incomplete coverage | Start with consequential, failure-prone mechanisms rather than chasing a coverage percentage. [Factorio](https://factorio.com/blog/post/fff-62) |
| **Factorio, 2019 developer account** | Logic tests missed GUI failures; the developers added tests using the full graphical interface | Headless correctness and observer-facing correctness need separate coverage. [Factorio](https://factorio.com/blog/post/fff-288) |
| **NetLogo BehaviorSpace, 7.0.4 documentation** | Reusable experiments support parameter variation, repetitions, headless execution, and exported measurements | Copy its experiment-runner workflow, not its simulation implementation. [NetLogo 7.0.4 User Manual](https://docs.netlogo.org/behaviorspace.html) |
| **Macal, MABS 2013 keynote** | Uses explicit base cases and incremental scenario changes to investigate behavior | Controlled interventions can expose mechanisms more cheaply than fitting complete histories. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/MASValidation.Macal2013.pdf) |
| **Unreal Automation Framework, 5.8** | Provides functional, input-driven, stress, and screenshot-oriented testing; it is not ideal for pure unit tests | Keep kernel tests in Rust and use Unreal automation for the actual integration. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/automation-test-framework-in-unreal-engine) |

These are different kinds of evidence. Factorio provides strong software-engineering precedent, not evidence that game developers have empirically validated simulated societies. Starsim provides a quantified methodological result, not a hardware benchmark transferable directly to TCE.

---

## 4. Recommended implementation for TCE

### 4.1 One kernel, two execution paths

Use the following proposed structure:

```
tce-kernel       Authoritative simulation crate
tce-headless     Command-line runner using that crate
tce-ffi         Windows DLL exposing a narrow C ABI
tce-validation  Fixtures, pattern definitions, comparisons, reports
Unreal tests    DLL integration, presentation, input, save/load
```

Rust supports both Rust-library and C-compatible dynamic-library outputs. Keep a narrow ABI with explicit ownership and lifetimes rather than passing Rust containers across the boundary. Unwinding behavior across foreign interfaces requires deliberate handling. [Rust Documentation](https://doc.rust-lang.org/reference/linkage.html)

The headless executable should exercise the **same simulation code, content, and settings** as the DLL. Also test the actual DLL: successful Rust-only tests do not establish that the C++ boundary is correct.

Keep Python, SALib, SciPy, and any optimizer in an external development environment. None needs to ship inside the game.

### 4.2 Make every failure reproducible

Each run should save a compact manifest containing:

* Code, content, configuration, and initial-state hashes.
* Seed and random-generator configuration.
* Simulation time-step and scheduler settings.
* Build/toolchain/platform information relevant to reproducibility.
* Scenario, intervention, observation-window, and metric definitions.

Maintain a reproducible single-thread reference mode, but also test the production execution path. Exact trace comparisons are appropriate where deterministic equivalence is promised; behavior-changing versions usually need distributional comparisons instead.

Save complete continuation state, including pending events and random-generator state. For long worlds, checkpoints plus a bounded recent-event buffer are more practical than retaining every detailed decision forever.

### 4.3 Use a small scenario bank

I recommend six initial scenario families:

| Scenario | Main purpose |
| --- | --- |
| Closed subsistence settlement | Accounting, seasonal labor, food storage, demography |
| Two settlements with complementary resources | Trade, transport, prices, specialization |
| Controlled harvest or route disruption | Adaptation, shortage propagation, recovery mechanisms |
| Unequal household resources or obligations | Distributional effects hidden by averages |
| Knowledge-transfer bottleneck | Awareness, adoption, prerequisites |
| Established late-world checkpoint | Long-lived institutions, accumulated state, aging identifiers, save/load |

Each family should contain a controlled fixture and a less constrained emergent version.

Synthetic late-world fixtures efficiently exercise particular mechanisms. They should not entirely replace worlds that actually reached late states through the simulation: the two test different things.

### 4.4 Keep runtime instrumentation bounded

For the shipping kernel, my proposed budget is **roughly 1–3% overhead for routine diagnostics**, to be measured rather than assumed.

Use event-driven counters, periodic aggregates, and targeted traces. Reconcile incremental totals against an independent census periodically. Record distributions and problematic tails—not just means.

When an invariant fails or a pattern becomes suspicious, preserve a checkpoint reference, recent causal events, and affected entity identifiers. Do not synchronously dump a full world on Unreal’s game thread.

For “endless” operation, explicitly test memory growth, event-queue growth, stale references, counter overflow, entity-ID reuse, and cumulative numerical drift.

### 4.5 Respect the hardware and frame budget

At 60 fps, the frame budget is **16.67 milliseconds**. That arithmetic does not tell us how much of it TCE can afford to spend on simulation.

I recommend:

* No calibration sweeps in the interactive game loop.
* Independent logical simulation time; rendering interpolates snapshots.
* Benchmark headless throughput with one, two, and four workers before increasing concurrency.
* Avoid each batch worker independently occupying every CPU core.
* Include 10k- and 50k-person worlds in production-scale checks.
* Profile frame time, kernel tick time, synchronization, and serialization separately.

Unreal Insights provides the appropriate engine-side timing and trace tooling. Your GPU and VRAM matter to rendering, but do not automatically accelerate a CPU-based Rust validation workload. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine)

Neither the available sources nor this report establish a TCE throughput figure on your i9. Measure simulated time per wall-clock second and peak memory on representative early, dense, and late worlds before fixing batch sizes.

### 4.6 A bounded operating cadence

The following are **proposed wall-clock ceilings**, with scenario counts and horizons adjusted to measured throughput:

| Trigger | Budget | Work |
| --- | --- | --- |
| Ordinary edit | 30–60 seconds | Unit/property tests, small accounting fixtures, critical regressions |
| Behavioral change | 5–15 minutes | Relevant scenario families, fixed seeds, paired comparisons |
| Background development batch | 1–2 hours | Fresh seeds, longer horizons, production-scale worlds |
| Behavioral milestone | 15–30 minutes of review, plus a bounded batch | Face validation and targeted sensitivity screening |
| Release candidate | Explicitly allocated soak period | Long histories, late checkpoints, actual DLL, save/load, presentation and performance |

Do not recalibrate after every patch. First compare with parameters held fixed; otherwise tuning can conceal regressions.

### 4.7 Report uncertainty without making the suite flaky

For each pattern, report **pass, fail, warning, or insufficient evidence**, together with the scope, number of worlds, effect size, and uncertainty.

Treat independent worlds—or independent baseline/intervention pairs—as the sampling units for world-level conclusions. Fifty thousand agents in one interconnected world are not fifty thousand independent replications.

A useful calculated warning: with **zero failures in 20 independent randomly sampled worlds**, the exact one-sided 95% upper bound on the underlying failure probability is

\[
1-0.05^{1/20}\approx13.9\%.
\]

Twenty clean worlds are useful smoke testing; they do not demonstrate that failures are rare. That calculation does not apply to a handpicked regression set as though it were a random sample.

Keep fixed seeds for reliable regression diagnosis and fresh seeds for broader exploration. Prespecify how warnings trigger additional runs; do not repeatedly rerun a failure until a favorable sample appears.

---

## 5. Common failures and practical checklists

### Failure modes worth designing against

| Failure | Why it misleads | Correction |
| --- | --- | --- |
| One convincing world | A selected seed may hide frequent pathological outcomes | Fixed regression seeds plus independently selected fresh seeds |
| Correct aggregates, impossible agents | Errors cancel in totals | Cross-scale patterns and sampled individual traces |
| Calibration presented as validation | The model is rewarded for matching what shaped it | Hold out patterns and environmental conditions, not only seeds |
| Wrong mechanisms compensate for one another | Excess production can conceal excessive spoilage or travel costs | Anchor local quantities and inspect intermediate flows |
| Baselines updated automatically | Broken behavior becomes the new standard | Require an explanation and explicit approval for baseline changes |
| Every unexpected outcome labeled a bug | Emergence is replaced with scripted history | Separate violated constraints from surprising but feasible outcomes |
| Small worlds treated as full-scale evidence | Important interactions or boundaries differ | Confirm at production scale; document what the simulated boundary excludes |
| A smooth average hides two incompatible outcomes | “Moderately successful” may mean half thriving, half collapsing | Show outcome distributions and collapse rates |
| Headless passes, game fails | Integration and display bugs remain untested | Actual-DLL and Unreal functional tests |
| AI-generated tests repeat AI-generated assumptions | Implementation and oracle share the same mistake | Independent checkers, source-backed expectations, deliberate fault injection |

For tests themselves, mutation testing is useful: deliberately introduce changes such as omitting an inventory decrement and verify that tests detect them. **cargo-mutants** automates variants of this technique for Rust. Use it selectively on important mechanisms rather than on every edit. [Mutants](https://mutants.rs/)

### Before accepting a new mechanism

* Its purpose, units, assumptions, and authoritative state are explicit.
* At least one exact fixture checks implementation behavior.
* At least one observable could reveal that the mechanism is inadequate.
* The expected observable is labeled empirical, mechanistic, or design-driven.
* A controlled intervention or alternative rule has been considered.

### Before accepting a behavioral change

* Compare with parameters held fixed before recalibrating.
* Inspect distributions and affected subgroups, not just world totals.
* Check fresh seeds and at least one held-out condition.
* Preserve and explain any changed baseline.
* Confirm that rendering, diagnostics, and save/load have not altered authoritative behavior.

### Before release

* Exercise the real DLL and production threading configuration.
* Include 10k/50k populations and early/late states.
* Audit memory and accumulated state over long horizons.
* Keep empirical plausibility, software correctness, and performance results separate.
* Record untested domains and known failed patterns.

---

## 6. Source guide and version applicability

| Source | What to read or reuse | Applicability |
| --- | --- | --- |
| [Grimm et al., *Pattern-Oriented Modeling of Agent-Based Complex Systems*](https://doi.org/10.1126/science.1116681); [Grimm & Railsback, *A “multi-scope” for predictive systems ecology*](https://pubmed.ncbi.nlm.nih.gov/22144392/?utm_source=chatgpt.com) | Foundational POM framework and later explanation | 2005 and 2012 papers; methodology, not software versions |
| [ODD protocol, second update](https://www.jasss.org/23/2/7.html?utm_source=chatgpt.com) | Purpose, patterns, assumptions, and model-description checklist | 2020 update |
| [Collins, Koehler & Lynch, validation-methods overview](https://www.jasss.org/27/1/11.html?utm_source=chatgpt.com) | Method taxonomy and complementary validation approaches | 2024 survey |
| [Windrum, Fagiolo & Moneta, empirical validation](https://www.jasss.org/10/2/8.html?utm_source=chatgpt.com) | Calibration, empirical comparison, and identification difficulties | 2007 paper |
| [Axtell et al., docking paper](https://doi.org/10.1007/BF01299065); [authors’ working paper](https://www-personal.umich.edu/~axe/research/Aligning_Sim.pdf?utm_source=chatgpt.com) | Alignment procedure and scheduling lesson | 1996 publication; 1995 working-paper text |
| [Macal, *On Validating Multi-Agent System Applications*](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/MASValidation.Macal2013.pdf?utm_source=chatgpt.com) | Keynote slides, including controlled base-case comparisons | MABS 2013 talk |
| [Factorio FFF-62](https://factorio.com/blog/post/fff-62?utm_source=chatgpt.com); [FFF-288](https://factorio.com/blog/post/fff-288?utm_source=chatgpt.com); [testing demonstration](https://factorio.com/blog/post/fff-186?utm_source=chatgpt.com) | Firsthand testing experience and demonstration | 2014, 2019, and 2017 accounts; not claims about the current internal suite |
| [NetLogo BehaviorSpace](https://docs.netlogo.org/behaviorspace.html?utm_source=chatgpt.com) | Experiment definitions, repetitions, headless runs, output | Documentation labeled **7.0.4**; several relevant additions date to 6.4 |
| [SALib documentation](https://salib.readthedocs.io/en/stable/?utm_source=chatgpt.com); [code](https://github.com/SALib/SALib) | Morris and Sobol sampling/analysis | Rolling stable documentation checked September 2026; pin an exact package version |
| [SciPy differential evolution](https://docs.scipy.org/doc/scipy/reference/generated/scipy.optimize.differential_evolution.html?utm_source=chatgpt.com); [Optuna optimization guide](https://optuna.readthedocs.io/en/stable/tutorial/10_key_features/003_efficient_optimization_algorithms.html?utm_source=chatgpt.com) | Optional offline calibration tools | Retrieved documentation labeled **SciPy 1.18.0** and **Optuna 5.0.0** |
| [Common-random-numbers paper](https://arxiv.org/abs/2409.02086?utm_source=chatgpt.com); [analysis code](https://github.com/starsimhub/crn_paper) | Paired stochastic comparisons and reported savings | 2024 preprint v2; reported results used **Starsim 1.0.1** |
| [proptest](https://docs.rs/proptest/latest/proptest/?utm_source=chatgpt.com); [failure persistence](https://proptest-rs.github.io/proptest/proptest/failure-persistence.html?utm_source=chatgpt.com); [cargo-mutants](https://mutants.rs/?utm_source=chatgpt.com) | Generated test cases, minimized failures, testing the tests | proptest documentation labeled **1.11.0**; cargo-mutants book is rolling documentation |
| [Rust linkage](https://doc.rust-lang.org/reference/linkage.html?utm_source=chatgpt.com); [Rust FFI guidance](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | Library outputs and ABI safety | Rolling official documentation; pin TCE’s Rust toolchain |
| [Unreal Automation Framework](https://dev.epicgames.com/documentation/en-us/unreal-engine/automation-test-framework-in-unreal-engine?utm_source=chatgpt.com); [Unreal Insights](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-in-unreal-engine?utm_source=chatgpt.com) | Integration tests and performance traces | Retrieved **UE 5.8** documentation |

## Final recommendation

The minimum valuable system is **one reproducible headless runner, exact accounting checks, a small scenario bank, approximately a dozen pattern cards, and a comparison report that can lead directly to an explanatory trace**.

Add bounded random or Latin-hypercube calibration next. Use Morris when you need to identify influential uncertainties. Reserve Sobol analysis, Bayesian calibration, and surrogate modeling for specific unresolved questions.

The aim is not to make every history predictable. It is to make **impossible behavior fail immediately, implausible behavior become visible, and surprising behavior remain explainable**.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab929a8-06ec-83ea-8bf3-c17031ebf465)
