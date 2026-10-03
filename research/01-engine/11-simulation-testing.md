# Pragmatic testing for The Civilization Engine

**Recommendation:** build TCE around a headless Rust test harness, exact checks for local rules and accounting, property-based tests on small worlds, and **three to five bounded world scenarios** that detect catastrophic regressions. Add a thin layer of tests for the actual Windows DLL and Unreal integration. Do **not** make exact century-long histories, population-growth curves, or frequent statistical significance tests the foundation of CI.

The essential distinction is:

> **Test whether the simulation obeys its rules rigorously. Test whether those rules produce believable civilizations separately.**

This parallels the simulation literature’s distinction between *verification*—implementing the intended model correctly—and *validation*—whether the model adequately represents its intended subject. Metamorphic testing helps with both, but does not remove that distinction. [NIST](https://www.nist.gov/publications/metamorphic-testing-continuum-verification-and-validation-simulation-models)

The design below is tailored to TCE. Proposed case counts and CI times are starting budgets, **not measured performance claims**. Sources and tool documentation were checked as of **September 27, 2026**.

---

## 1. Options: what to test, and how

### 1.1 The useful techniques are complementary

My assessment for TCE is:

| Technique | How it works | Best application in TCE | Main cost or limitation |
| --- | --- | --- | --- |
| Example-based unit tests | Assert known results for deliberately constructed inputs | Transactions, reservations, calendars, inheritance, legal transitions | Requires choosing meaningful cases |
| Property-based testing | Generate many inputs or command sequences and check general rules | Inventory, generational IDs, jobs, event queues, save/load | Good generators and useful failure reduction require work |
| Metamorphic testing | Transform an input or execution and assert a relationship between outputs | Save/load continuation, execution chunking, irrelevant-component isolation | The relationship needs explicit preconditions |
| Differential testing | Compare an optimized implementation with a simple independent reference | Routing, spatial queries, allocation, incremental caches | Reference implementation must genuinely be independent |
| Semantic snapshots | Compare a canonical, human-readable representation with an approved baseline | Compiled content, explanations, small deterministic scenarios | A snapshot can faithfully preserve a bug |
| Bounded world tests | Run a few selected worlds and check invariants and operational thresholds | Cross-system integration and catastrophic regressions | Cannot establish broad statistical reliability |
| Statistical assertions | Check frequencies, distributions, or estimated differences | Small stochastic mechanisms with known expectations | False alarms, weak power, correlated observations |
| Fuzzing and concurrency/UB tools | Search malformed inputs, unsafe operations, or thread interleavings | Save parsers, FFI adapters, buffer handoff | Specialized tooling; unsuitable for an entire large world |

The tool should follow the failure mode. A resource-accounting error does not need a statistical test. A use-after-free does not need a population-growth benchmark. A questionable social outcome may need model review rather than a code assertion.

### 1.2 Make invariants the backbone

For TCE, I would maintain an explicit **invariant catalogue**, with each invariant identifying its scope, permissible exceptions, and diagnostic output.

**Accounting.** For each good, over a defined interval:

\[
Q\_{\mathrm{end}} =
Q\_{\mathrm{start}}
+ Q\_{\mathrm{produced}}
+ Q\_{\mathrm{imported}}
- Q\_{\mathrm{consumed}}
- Q\_{\mathrm{exported}}
- Q\_{\mathrm{destroyed}}.
\]

Include goods in warehouses, personal inventories, transport, and reserved stock without double-counting them. Production and destruction must be explicit ledger events. “The total amount of goods never changes” would be an incorrect property.

Apply the same discipline to population:

\[
P\_{\mathrm{end}} =
P\_{\mathrm{start}}+\text{births}+\text{immigration}
-\text{deaths}-\text{emigration}.
\]

Money requires the rules of TCE’s monetary system: minting, credit creation, taxation, and debt cancellation may intentionally change particular totals.

**Identity and ownership.** Live references resolve to the intended generation of an entity. Destroying and reusing a slot cannot turn an old citizen reference into a reference to a different person. Mutually exclusive claims cannot coexist; shared ownership follows the authored rules.

**Transactions and activities.** A reservation cannot be consumed twice. Cancelling a job releases its reservations exactly once. Completion delivers its outputs exactly once. A failed transaction leaves either the old state or a defined recovery state, not half a transfer.

**Time and lifecycle.** Events cannot accidentally execute twice or move simulation time backward. Birth, death, succession, job completion, and scheduled obligations remain consistent across saves and time acceleration. Test dates near large year counts and integer boundaries directly instead of waiting through centuries.

**Content and geometry.** Every referenced primitive exists; recipes have valid units; technology prerequisites are well formed; building footprints and road connections satisfy their declared geometric contracts. Test legal transitions against each authored constitution—not against an assumed universal political system.

These are proposed TCE contracts. They should hold regardless of which civilization emerges.

For cost control, use cheap incremental checks at transaction boundaries and periodic full audits. If a full audit traverses agents and relationships, its cost is roughly \(O(N+E)\); invoking it after every individual update would turn a useful safeguard into excessive repeated work. Crucially, the full audit should independently recompute totals rather than merely trust the counters it is checking.

### 1.3 Property-based testing: generate small, meaningful worlds

`proptest` is a strong Rust fit because it supports composable generators, shrinking, and persisted failures. Its documentation specifically recommends retaining regression seeds, while warning that changes to a generation strategy can change what an old seed produces. [Docs.rs](https://docs.rs/proptest/latest/proptest/)

Start with **small state machines**, not randomized 50,000-person worlds. Examples include:

* A few households, inventories, and jobs receiving sequences of reserve, transfer, cancel, complete, save, and load commands.
* A small settlement with births, deaths, ownership transfers, and entity-slot reuse.
* A road graph receiving additions, closures, and route queries, checked against a simple reference implementation.

The valuable failure is a reduced sequence such as:

> Reserve grain → cancel job → delete worker → reuse worker slot → complete stale job → grain duplicated.

That is more actionable than “seed 812 became unstable after 73 years.”

Construct valid starting states directly. Excessive rejection filtering can concentrate testing on trivial cases; the original QuickCheck paper demonstrates this problem and advocates inspecting generated-case distributions and using suitable generators. [Tufts Computer Science](https://www.cs.tufts.edu/~nr/cs257/archive/john-hughes/quick.pdf)

For TCE, add **coverage conditions** to the harness: the test must actually execute transfers, complete jobs, encounter cancellations, and reuse IDs. A conservation property that passes because no production ever occurred is insufficient.

My suggested starting point is 64–128 bounded cases per important property, adjusted after measuring runtime. Persist the seed, but turn significant failures into explicit minimized fixtures so their meaning survives generator changes.

### 1.4 Metamorphic testing: powerful when the relationship is justified

Metamorphic testing is useful when an exact final answer is unavailable but a relationship between executions is known. The simulation-specific literature also cautions that relations used for validation embody assumptions about the model; a failed relation may expose a bad assumption rather than bad code. [NIST Applications](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=931851)

Good candidate relations for TCE are:

| Proposed relationship | Required qualification |
| --- | --- |
| Save/load preserves semantic state | Compare meaningful state, not memory addresses or serialization layout |
| Save/load plus continuation matches uninterrupted execution | Use a deterministic execution scope; preserve relevant RNG state, queues, and pending work |
| Advancing 100 fixed ticks equals advancing 40 then 60 | Tick size, command timing, and scheduling boundaries must remain identical |
| Enabling an observer or renderer does not alter simulation decisions | Observer work must not consume simulation RNG or change logical input timing |
| An isolated component behaves the same alone and alongside another | There must be no shared market, global resource, environmental coupling, or shared random-draw dependency |
| Reordering unordered content declarations changes nothing | Declaration order must not be part of the model’s semantics or tie-breaking |

Avoid plausible-sounding but unjustified macro-properties such as “more initial food must produce a larger population after 100 years.” Migration, conflict, congestion, fertility, and institutional feedback could reverse the outcome.

Similarly, translating a map is only invariant if geography, weather, boundaries, and coordinate-dependent rules permit it. Renaming entities may alter results when IDs participate in tie-breaking or random-stream assignment.

For TCE’s detailed versus daily-resolution modes, require accounting and structural invariants in both. Do not assert identical histories unless the two algorithms are mathematically equivalent. Approximation quality belongs in a small, separately scoped comparison suite.

### 1.5 Differential testing: spend effort on cheap independent oracles

For optimized systems, retain a slow implementation that is obviously correct on small inputs:

**Routing:** compare path cost with Dijkstra on a small graph. Compare costs and validity, not necessarily an identical path when several optimal paths exist.

**Spatial indexing:** compare indexed queries with a brute-force scan.

**Incremental state:** compare cached totals, connectivity, or eligibility sets with a complete recomputation.

**Resource allocation:** compare the optimized implementation with exhaustive enumeration on tiny problems where that is feasible.

The independence matters. Calling the same implementation through two wrappers provides little protection against shared mistakes.

---

## 2. Regression testing without exact world reproducibility

### 2.1 Separate stochasticity from accidental nondeterminism

An emergent world can be stochastic yet exactly repeatable under a fixed seed and execution order. Conversely, a fixed seed does not control thread scheduling, hash iteration, floating-point reduction order, or changed random-draw consumption.

TCE does not need to promise identical centuries across every CPU and compiler. I would nevertheless provide a **repeatable diagnostic mode**:

* Fixed logical time and explicitly timestamped inputs.
* Fixed seed and recorded RNG implementation/version.
* Stable scheduling for small scenarios, initially with one worker.
* Reuse of production model code rather than a separate simplified simulator.

Then run the actual parallel implementation as a separate test target. Race freedom and transaction correctness remain mandatory even when whole-world bitwise identity does not.

Testing more than one worker configuration is worthwhile: Factorio documented a map-generation desynchronization whose reproduction depended on core count, with a defect originating in 2017 finally resolved during 2.0 development in 2024. [factorio.com](https://factorio.com/blog/post/fff-415?utm_source=chatgpt.com)

### 2.2 Use three different kinds of regression oracle

**Exact assertions for logical state.** Entity counts, ledger identities, reservation ownership, event execution counts, and integer-valued transactions should not receive broad tolerances merely because the surrounding simulation is stochastic.

**Numerical tolerances for numerical algorithms.** For finite values, an explicit comparison can be:

\[
|a-b| \leq \mathrm{atol}
+\mathrm{rtol}\max(|a|,|b|).
\]

Choose tolerances by quantity and scale. Metres, probabilities, and annual production need different error budgets. Check finiteness first. A single global epsilon is not a specification, and a large tolerance should not conceal a unit-conversion error.

**Operational envelopes for emergent behavior.** Use broad, scenario-specific assertions such as “the controlled supply chain completes work,” “all failed deliveries reach a declared terminal state,” or “queues drain after a temporary obstruction is removed under guaranteed service capacity.”

These envelopes are engineering acceptance rules, not statistical proof that arbitrary worlds behave correctly.

### 2.3 Snapshot semantics, not entire histories

`insta` supports snapshot review and redaction of unstable fields. Its redaction facilities can also sort semantically unordered collections. [Docs.rs](https://docs.rs/insta/latest/insta/)

For TCE, snapshot:

**Compiled content:** resolved recipes, prerequisites, policies, and style definitions.

**Small deterministic scenarios:** a transaction ledger, a succession sequence, or a job’s state transitions.

**Explanations:** why an agent selected a particular action in a controlled context.

**Save inspection summaries:** entity types, references, schema metadata, and selected meaningful values.

Do not snapshot arbitrary pointer values, wall-clock timestamps, full unordered memory dumps, or decades of emergent events. Sort only collections whose order is irrelevant; sorting a decision queue could hide a scheduling bug.

Rounded numeric snapshots are also not a substitute for tolerance assertions: nearly identical values can fall on opposite rounding boundaries.

Most importantly, **an AI coding agent should not automatically accept new snapshots or widen thresholds to make its patch pass**. Baseline changes are specification changes requiring review.

### 2.4 Keep statistical assertions local

Statistical assertions are most useful where there is a known stochastic contract: a sampler, hazard conversion, weighted choice, or random-selection mechanism.

Prefer deterministic boundary tests first. For a Bernoulli decision with probability \(p\), inject controlled uniform draws and test the threshold behavior. Test the probability calculation independently from the RNG wiring. Then, where useful, add a bounded distribution check over many cheap draws.

For a genuine binomial sampling check, choose an acceptance interval from the specified \(n\), \(p\), and an explicit false-alarm allowance. Do not choose an arbitrary “within 5%” threshold. For whole-world output, three or five runs are usually too little evidence for a useful narrow distributional gate.

There are several traps:

**Multiple assertions compound false alarms.** If 20 independent tests each falsely reject correct behavior with probability 0.05, the chance of at least one false rejection is:

\[
1-0.95^{20}\approx64.2\%.
\]

**Daily observations are not independent worlds.** Measuring one world on 100 successive days does not create 100 independent replications.

**Confidence intervals and acceptance bands answer different questions.** An interval for the mean is not a prediction interval for the next individual world.

**Failure to detect a difference is not evidence of equivalence.** A low-powered comparison can miss an important regression.

My recommendation is therefore **no routine whole-world significance-test gate initially**. For a substantial economic or demographic algorithm change, an occasional small paired comparison can be diagnostic: report effect sizes against a predefined practically important margin, and allow the result to be inconclusive. Keep that outside the everyday merge gate.

Never use “retry until green” as the statistical policy. A rerun may help diagnose a failure, but preserve the first failure and its artifacts.

### 2.5 A practical four-world suite

I would begin with these fixtures:

| Fixture | Suggested construction | Required checks |
| --- | --- | --- |
| **Viable agrarian settlement** | Roughly 200 people; deliberately sufficient resources; bounded seasonal run | Accounting, working production chain, food consumption, valid jobs and homes |
| **Disruption scenario** | Similar small settlement with a closed route, cancelled work, resource loss, or leader death | Correct cancellation/recovery, no duplication, valid succession, declared blocked states |
| **Mature-world continuation** | Curated checkpoint with several institutions, technologies, settlements, and pending obligations | Loadability, valid references, continued execution, save/load integrity |
| **Scale fixture** | Bounded 10k-person run in suitable CI; 50k-person run on the target PC | Structural invariants, queue behavior, bounded temporary memory, throughput diagnostics |

Every fixture should have a fixed horizon and a documented reason to exist.

Do not require population growth in the disruption world: decline may be legitimate. Do not declare every waiting agent deadlocked: waiting for unavailable resources can be valid. Check bounded progress only where the fixture guarantees that progress is possible.

A mature checkpoint exercises mature state, but does not prove that centuries of gradual accumulation are safe. Retain one occasional longer continuation run for that purpose, rather than multiplying seeds and parameter combinations.

**Four successful worlds are a smoke test, not a reliability estimate.**

---

## 3. Precedents: what developers actually document

### Factorio: accumulate small regressions, then trust automation

Wube described a still-small automated suite in **2014, during 0.11 development**, focused on difficult code and desynchronization problems. By **2017, during 0.15 development**, the suite ran continuously and notified developers when changes broke tests—including unexpectedly unrelated systems. The latter article includes an automated-test demonstration. [Factorio](https://factorio.com/blog/post/fff-62)

Factorio also illustrates the authoring trade-off: one developer reported a five-minute fix followed by two hours spent building the regression test. That is an anecdote about maintenance investment, not a benchmark. [Factorio](https://www.factorio.com/blog/post/fff-71)

Its save/load desynchronization reports are particularly relevant to TCE: cached unit-group speed, serialization, and later changes to modifiers interacted in ways that broke continuation consistency. **A save that loads successfully is not necessarily a save that preserves future behavior.** [Factorio](https://www.factorio.com/blog/post/fff-340)

**Lesson for TCE:** convert discovered failures into small permanent cases, and test interactions and continuation—not just individual functions.

### Dwarf Fortress: controlled arenas; distinguish DFHack’s documented CI

In historical **Dwarf Fortress Talk #9**, Tarn Adams discusses controlled arena situations with matching equipment and skills, and the arena’s use for testing mods and mechanics. This supports the value of a compact, inspectable mechanism-testing environment; it does not establish Dwarf Fortress’s present internal CI architecture. [Bay 12 Games](https://www.bay12games.com/media/df_talk_9_transcript.html)

**DFHack is a separate community project**, but provides a strong integration precedent. Its **53.16-r1 documentation** describes tests executed inside a real running Dwarf Fortress on Windows and Linux, separate release-configuration builds, and native type-size comparisons. [DFHack Documentation](https://docs.dfhack.org/en/stable/docs/dev/github-workflows.html)

**Lesson for TCE:** have both a fast headless “arena” and a smaller suite that exercises the real host process.

### Paradox: failing saves and controlled AI observations

The **Stellaris 3.0–3.3-era developer diaries** give concrete examples rather than a public company-wide testing blueprint.

Diary **#227** compares AI economic behavior in developed galaxies and identifies performance optimizations that accidentally excluded some buildings and districts from consideration. [Reddit](https://www.reddit.com/r/Stellaris/comments/q38rhh/stellaris_dev_diary_227_looking_after_the_ai/)

Diary **#235** explains that the team examined submitted savegames and checked that changes handled their failures. It also describes testing AI on Ensign difficulty to examine baseline behavior without difficulty bonuses, and a resource-dependent decision deadlock involving food and consumer goods. The accessible sources are reposts of the developer-authored diaries. [Reddit](https://www.reddit.com/r/Stellaris/comments/r763j1/stellaris_dev_diary_235_ai_feedback_and_future/)

**Lesson for TCE:** curate actual failing saves, isolate the mechanism being evaluated, and distinguish behavioral improvement from bonuses that mask weaknesses. These sources do **not** establish a particular automated statistical CI regime.

### ABM frameworks and simulation projects

**Mesa’s current source tests** check time advancement, stopping conditions, seeded RNG behavior, agent registration and removal. A specific regression preserves an empty agent-type collection after extinction because models rely on querying it. These are exact framework contracts underneath stochastic models. [GitHub](https://github.com/mesa/mesa/blob/main/tests/test_model.py)

**NetLogo 7.0.4 BehaviorSpace** supports seeded, bounded, headless model runs and metric collection. It is an experimentation tool rather than an automatic correctness oracle. For TCE, borrow the configurable runner and metrics interface—not its parameter-sweep workflow as a default development process. [CCL](https://ccl.northwestern.edu/netlogo/docs/behaviorspace.html)

**SUMO’s developer documentation** describes output-file regression testing with TextTest, nightly execution, and shared GUI/non-GUI tests. It explicitly warns that reference outputs are not automatically correct. [Eclipse SUMO](https://sumo.dlr.de/docs/Developer/Tests.html)

**Lesson:** exact low-level contracts, reusable runners, and reviewed reference outputs are well-established approaches. The evidence does not imply that realistic simulation requires large experiment campaigns on every change.

---

## 4. Recommended implementation for TCE

### 4.1 Keep almost all correctness testing outside Unreal

Use one shared kernel implementation with three interfaces:

```
tce-core       Rust model and algorithms; reusable library
tce-headless   Scenario runner, invariant audits, diagnostics
tce-ffi        Windows DLL boundary used by Unreal
```

The headless runner should take a scenario, seed, logical horizon, worker count, and audit configuration. It should return a nonzero exit code on failure and produce machine-readable diagnostics.

For each failure, preserve the commit and content fingerprints, toolchain/build configuration, seed and RNG metadata, worker count, last good checkpoint, input-command log, failing invariant values, and a short recent event trace. A seed alone is not enough for a nondeterministic failure.

Make the command sequence and checkpoint easy to load into a small Unreal inspection scene. That creates a direct route from automated detection to visual diagnosis.

### 4.2 Use optimized checked builds and actual shipping builds

A useful Cargo profile is:

```
[profile.simcheck]
inherits = "release"
debug = 1
debug-assertions = true
overflow-checks = true
```

Then run:

```
cargo test --workspace --locked --profile simcheck
```

Cargo supports custom inherited profiles. Release defaults disable debug assertions and overflow checks, so an explicit checked profile is useful for running meaningful simulation workloads without relying solely on unoptimized debug execution. [doc.rust-lang.org](https://doc.rust-lang.org/cargo/reference/profiles.html)

Also test actual release artifacts. **Cargo’s normal test harness ignores the configured panic strategy and requires unwinding**, so `cargo test --release` does not demonstrate the behavior of a shipped `panic=abort` DLL. Use a separately launched executable to load and exercise that DLL. [Rust Documentation](https://doc.rust-lang.org/cargo/reference/profiles.html)

### 4.3 Test the DLL boundary as a product interface

Create a small MSVC C++ executable that loads the DLL and exercises:

**Lifecycle:** create a world, submit inputs, advance it, acquire and release snapshots, save/load, destroy it, and repeat.

**Contract handling:** wrong ABI version, unsupported options, insufficient valid buffers, and stale generation-checked handles where the API promises detection.

**Concurrency:** consumer delays, shutdown while workers are active, and snapshot replacement while older snapshots remain borrowed.

**Failure behavior:** injected internal errors and panics in subprocesses, with the expected exit or error result.

Do not fuzz arbitrary invalid pointers into an API whose safety contract requires valid pointers. Fuzz the safe byte-oriented adapter or valid owned buffers instead.

The Rustonomicon documents why this boundary matters: a Rust panic encountering a non-unwinding boundary aborts, foreign exceptions entering Rust through an inappropriate boundary are unsafe, and `catch_unwind` only catches unwinding panics. [Rust Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

For TCE, choose and document one panic policy. If an entry point catches a panic, mark affected world state unusable unless recovery is explicitly established. Do not continue from a partially mutated world merely because the exception was caught.

### 4.4 Keep Unreal tests narrow

UE **5.8’s Automation Test Framework** supports engine-dependent unit, feature, content, and screenshot tests. Epic defines its “Smoke” category as tests taking **under one second**. TCE’s multi-world runs should therefore remain a separate headless suite rather than being labelled Unreal smoke tests. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/automation-test-framework-in-unreal-engine)

Inside Unreal, concentrate on loading the packaged DLL, ABI agreement, snapshot ownership, entity creation/destruction, interpolation across snapshot changes, pause/resume, and clean shutdown.

Include at least one **packaged-build** test. Editor success is not the acceptance criterion for a DLL that must ship.

A no-render integration run cannot establish 1440p rendering performance. Keep the real GPU/frame-pacing check separate.

### 4.5 Add specialized tools only where they pay

**Miri:** use it on small unsafe Rust components, such as handle storage or custom buffer ownership. It is an interpreter with substantial platform/FFI limitations, not a way to run the complete Unreal integration. [GitHub](https://github.com/rust-lang/miri)

**Loom:** use it for small synchronization protocols—particularly snapshot publication, cancellation, and ownership transfer. It explores thread interleavings using its instrumented synchronization types; its scope should be a tiny protocol, not thousands of simulated people. [Docs.rs](https://docs.rs/loom/latest/loom/)

**Coverage-guided fuzzing:** prioritize save parsing, migration, authored-content parsing, and byte-level interchange. Current Rust Fuzz documentation includes Windows MSVC support with AddressSanitizer; the older blanket advice that `cargo-fuzz` cannot run on Windows is outdated. Nightly and the documented native toolchain setup are still relevant. [Rust Fuzz](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html)

Start with ordinary tests and `proptest`. Add these tools when an unsafe or parser-heavy boundary exists, rather than building an elaborate testing platform in advance.

### 4.6 CI on Windows: a small mandatory lane

GitHub documents Rust build/test workflows and provides explicit Windows runner labels. Pin TCE’s Rust toolchain and dependencies; do not treat a moving hosted image as a reproducible development environment. [GitHub Docs](https://docs.github.com/en/actions/tutorials/build-and-test-code/rust)

A baseline workflow can be:

```
name: Rust correctness

on: [push, pull_request]

permissions:
  contents: read

jobs:
  kernel:
    runs-on: windows-2022
    timeout-minutes: 15
    env:
      RUST_BACKTRACE: "1"
      INSTA_UPDATE: "no"

    steps:
      - uses: actions/checkout@v6

      # Commit rust-toolchain.toml with an exact tested Rust version.
      - name: Toolchain
        run: |
          rustup show active-toolchain
          rustup component add rustfmt clippy

      - name: Formatting
        run: cargo fmt --all -- --check

      - name: Lints
        run: cargo clippy --workspace --all-targets --locked -- -D warnings

      - name: Checked optimized tests
        run: cargo test --workspace --locked --profile simcheck

      - name: Shipping artifacts
        run: cargo build --workspace --locked --release

      - name: Failure artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-diagnostics
          path: test-artifacts/
          if-no-files-found: ignore
```

This is the **base lane**: place bounded kernel scenarios in its test suite, and add the C++ DLL-loader invocation once that harness exists. Building the DLL alone does not test it. Configure the harness to write the diagnostic directory used above.

The action majors shown follow the referenced documentation; pin reviewed actions to immutable commit SHAs in the maintained workflow. Cache dependencies/build output with keys that distinguish toolchain, target, profile, and relevant configuration. GitHub also warns that self-hosted runners can be persistently compromised by untrusted workflow code. Do not expose the development PC’s Unreal runner to arbitrary pull requests. [GitHub Docs](https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions)

My proposed cadence is:

| Lane | Work | Initial budget target |
| --- | --- | --- |
| Per edit | Relevant unit/property tests | Tens of seconds after compilation |
| Every PR | Windows checked tests, bounded worlds, DLL loader | Aim for 5–10 minutes warm |
| Trusted integration/release | Packaged UE test, 10k/50k workload, real rendering | Deliberately bounded, roughly 10–20 minutes initially |
| Occasional targeted run | Longer continuation, fuzzing, unsafe/concurrency checks | Explicit time box; not a seed campaign |

Adjust these after measurements. Limit simultaneous world runs: a parallel test harness launching several internally parallel kernels can oversubscribe the machine.

### 4.7 Performance evidence and hardware implications

Two useful published measurements illustrate tooling gains, not TCE throughput.

DFHack’s maintainers report caching reducing test-fix iteration from **20 minutes to one minute**, and emergency release builds from **45 minutes to five minutes**. Those are project-reported workflow results, not controlled cross-project benchmarks. [DFHack Documentation](https://docs.dfhack.org/en/stable/docs/dev/github-workflows.html)

Nextest’s published benchmark shows approximately **1.37×–3.38×** faster test execution across the listed projects. However, it used **Rust 1.66, Linux, and a Ryzen 9 7950X**, excluded compilation, and selected the minimum of later runs. It is not a Windows/TCE prediction. Nextest is worth considering when test scheduling becomes a measured bottleneck; it will not make one expensive world simulation intrinsically faster. [Nextest](https://nexte.st/docs/benchmarks/)

On the specified i9/64 GB/4070 Ti machine, measure TCE and Unreal **together**. At 60 fps the total frame interval is about **16.67 ms**, but that does not imply every citizen should update at 60 Hz.

Track game-thread handoff time, simulation lag and throughput, snapshot backlog, frame-time tails, GPU time, and CPU/GPU memory. Test both widely distributed populations and a densely visible crowd. These are different workloads.

Use the target machine for performance acceptance; hosted CI is better suited to correctness, generous timeout checks, and algorithmic work counters. Neither the hardware specifications nor the cited benchmarks establish a safe testing-overhead percentage for TCE.

### 4.8 Rules for AI-assisted development

Require each meaningful fix to come with a failing example that passes after the fix. For critical invariants, deliberately break the relevant operation once—omit a debit, skip reservation release, or allow stale-handle reuse—and verify that the test detects it.

Keep oracle logic independent of the implementation being changed. Require review for snapshot updates, disabled tests, widened tolerances, and altered smoke thresholds.

**The most valuable testing asset is a growing corpus of understood failures, not a growing count of random worlds.**

---

## 5. Sources and version applicability

The citations above link the supporting evidence. These are the most useful entry points for implementation and further reading.

| Source | Version/date and relevance |
| --- | --- |
| [Claessen & Hughes, *QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs*](https://www.cs.tufts.edu/~nr/cs257/archive/john-hughes/quick.pdf?utm_source=chatgpt.com) | ICFP 2000. Foundational paper; especially useful on generator design and trivial-case bias. [Tufts Computer Science](https://www.cs.tufts.edu/~nr/cs257/archive/john-hughes/quick.pdf) |
| [Raunak & Olsen, *Metamorphic Testing on the Continuum of Verification and Validation of Simulation Models*](https://www.nist.gov/publications/metamorphic-testing-continuum-verification-and-validation-simulation-models?utm_source=chatgpt.com) | 2021. Simulation-specific treatment of metamorphic relations and their assumptions. [NIST](https://www.nist.gov/publications/metamorphic-testing-continuum-verification-and-validation-simulation-models) |
| [Factorio FFF #62](https://factorio.com/blog/post/fff-62?utm_source=chatgpt.com), [#186, including demonstration](https://factorio.com/blog/post/fff-186?utm_source=chatgpt.com), [#415](https://factorio.com/blog/post/fff-415?utm_source=chatgpt.com) | 2014/0.11 development; 2017/0.15 development; 2024/2.0 development. Historical engineering precedents, not assertions about an unchanged 2026 pipeline. |
| [Dwarf Fortress Talk #9 transcript](https://www.bay12games.com/media/df_talk_9_transcript.html?utm_source=chatgpt.com) and [DFHack workflows](https://docs.dfhack.org/en/stable/docs/dev/github-workflows.html?utm_source=chatgpt.com) | Historical pre-Premium talk; separate DFHack documentation checked at **53.16-r1**. |
| [Stellaris diary #227](https://www.reddit.com/r/Stellaris/comments/q38rhh/stellaris_dev_diary_227_looking_after_the_ai/?utm_source=chatgpt.com) and [#235](https://www.reddit.com/r/Stellaris/comments/r763j1/stellaris_dev_diary_235_ai_feedback_and_future/?utm_source=chatgpt.com) | Developer-authored 2021 diaries, accessible reposts; principally the **3.0–3.3** period. |
| [Mesa model tests](https://github.com/mesa/mesa/blob/main/tests/test_model.py?utm_source=chatgpt.com), [NetLogo BehaviorSpace](https://docs.netlogo.org/behaviorspace.html), [SUMO tests](https://sumo.dlr.de/docs/Developer/Tests.html?utm_source=chatgpt.com) | Mesa moving `main` as inspected; NetLogo **7.0.4**; current SUMO documentation describing TextTest-based regression infrastructure. |
| [proptest](https://docs.rs/proptest/latest/proptest/?utm_source=chatgpt.com), [insta](https://docs.rs/insta/latest/insta/?utm_source=chatgpt.com), [Loom](https://docs.rs/loom/latest/loom/?utm_source=chatgpt.com) | Documentation checked at **1.11.0**, **1.48.0**, and **0.7.2**, respectively. Pin tested versions in TCE rather than depending on `latest`. [Docs.rs](https://docs.rs/proptest/latest/proptest/) |
| [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html?utm_source=chatgpt.com), [Rust FFI guidance](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com), [Miri](https://github.com/rust-lang/miri?utm_source=chatgpt.com), [Rust Fuzz setup](https://rust-fuzz.github.io/book/cargo-fuzz/setup.html?utm_source=chatgpt.com) | Current documentation checked in September 2026; pin a tested nightly for nightly-dependent tooling. |
| [GitHub’s Rust CI guide](https://docs.github.com/en/actions/tutorials/build-and-test-code/rust?utm_source=chatgpt.com), [UE Automation Test Framework](https://dev.epicgames.com/documentation/en-us/unreal-engine/automation-test-framework-in-unreal-engine?utm_source=chatgpt.com) | Current GitHub documentation; Epic page explicitly identifies **Unreal Engine 5.8**. |

**Bottom line:** start with the headless runner, an invariant catalogue, small generated state-machine tests, four curated world fixtures, and an actual DLL lifecycle test. That combination gives TCE a practical correctness foundation while leaving genuinely emergent history free to vary.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927c3-642c-83ea-a1e0-6c31397ef1d3)
