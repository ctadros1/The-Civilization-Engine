# Agent-based macroeconomics for The Civilization Engine

**Engineering assessment, current to September 27, 2026**

## Executive recommendation

**Build a small, stock-flow-consistent economy with local adaptive rules—not a wholesale port of any one research model.** For TCE, the strongest combination is:

* **JAMEL and related stock-flow-consistent models** for accounting discipline and the interaction between production, incomes, consumption, and finance.
* **EURACE and the CATS/BAM lineage** for bounded search, inventory-driven production, posted prices, vacancies, and adaptive wages.
* **Keynes-meets-Schumpeter models** for investment, technological heterogeneity, and the connection between demand and innovation.
* **Axtell’s endogenous firms** for people founding, joining, leaving, and reorganizing productive organizations.

These are complementary precedents, not interchangeable implementations. In particular, an economy assembled from their mechanisms **does not inherit their empirical validation**. The original studies validate particular combinations of rules, parameters, institutions, and initial conditions. EURACE’s documentation work explicitly emphasizes how much those implementation details matter. [Springer](https://link.springer.com/article/10.1007/s00191-018-0594-0)

The central distinction is between **three meanings of stability**:

| Requirement | What TCE should enforce |
| --- | --- |
| **Computational correctness** | No duplicated goods, unfunded payments, invalid ownership, overflow, or corrupted employment relationships. |
| **Economic viability under benign conditions** | A feasible settlement should not collapse merely because every firm overreacts to one bad sales period. |
| **Historical plausibility** | Harvest failures, unemployment, bankruptcies, unequal development, and even societal collapse remain possible when their causes are present. |

The first should be guaranteed by construction. The second should be established through experiments. The third should not be “fixed” with invisible money injections or automatic replacement firms.

---

# 1. Options: what the main models actually offer

## 1.1 Model families and their suitability

### EURACE / Eurace@Unibi: the richest operational reference

EURACE is a family rather than one immutable model. The Eurace@Unibi branch represents heterogeneous households, firms, banks, and public institutions interacting across markets and regions. Its detailed documentation covers decision rules, interaction protocols, and balance sheets; the 2019 publication also discusses reproducible execution through a virtual appliance. [Springer](https://link.springer.com/article/10.1007/s00191-018-0594-0)

The **2011 v1.0 manual** contains particularly useful operational mechanisms: inventory replenishment under uncertain demand, vacancies calculated from desired employment, limited job applications, reservation wages adjusted for commuting, and wage-offer increases when vacancies remain unfilled. Its labor-market protocol runs through bounded matching rounds rather than solving for a market-clearing wage. [Noah NRW](https://noah.nrw/ubbihs/download/pdf/5127756)

**What to borrow:** separation of production planning, input acquisition, hiring, sales, and financial settlement; persistent employment relationships; local search frictions.

**What not to import wholesale:** its institutional machinery and calibration. A model organized around established firms, banking, unemployment payments, and regional policy is not an appropriate starting state for an agrarian village.

**Assessment:** the best reference manual, but too large to be TCE’s initial implementation specification.

### JAMEL: the strongest accounting and coordination foundation

JAMEL is an open-source Java framework for monetary agent-based economies. Its public repository identifies stock-flow consistency as a core concern and is licensed under GPL-3.0. [GitHub](https://github.com/pseppecher/jamel)

Two results are especially relevant. In Seppecher’s 2012 model, greater wage flexibility can destabilize an otherwise viable economy: falling wages weaken demand and contribute to a deflationary contraction. This is a **model-specific result**, not proof that every wage reduction causes depression. [Cambridge University Press](https://www.cambridge.org/core/journals/macroeconomic-dynamics/article/abs/flexibility-of-wages-and-macroeconomic-instability-in-an-agentbased-computational-model-with-endogenous-money/890FC15C5058C2C03459231FD6213010)

A later multisector model by Seppecher, Salle, and Lavoie uses heterogeneous markups over unit costs that evolve under market pressure. It produces coordination among interdependent sectors without an equilibrium price solver, including adjustment following technology shocks. [OUP Academic](https://academic.oup.com/icc/article-abstract/27/6/1045/5035120?utm_source=chatgpt.com)

**What to borrow:** explicit production and payment sequences; balance-sheet consistency; adaptive, heterogeneous pricing; household income feedback.

**What not to import wholesale:** modern monetary institutions or an assumption that one wage or markup rule works across all forms of organization.

**Assessment:** the closest conceptual foundation for TCE’s market economy.

### CATS, BAM, and Mark I: simple rules with important qualifications

The original **CATS—complex adaptive trivial system—models** emphasized financial fragility, capital accumulation, firm entry and exit, and aggregate dynamics. They should not be confused with a fully developed household shopping and physical logistics simulation. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0167268102001385?utm_source=chatgpt.com)

The later **BAM/Mark I lineage**, associated with *Macroeconomics from the Bottom-up*, provides more directly implementable interactions among workers, firms, and banks in labor, goods, and credit markets. A contemporary independent implementation, BAM Engine, explicitly identifies the 2011 book’s baseline as its starting point. [PyPI](https://pypi.org/project/bamengine/)

CATS also has unusually concrete validation work: Bianchi and colleagues initialized a model from **6,422 Italian nonfinancial firms** and compared simulated outcomes with observations over **1996–2001**. Results were less satisfactory for the smallest and largest firms. This is evidence about that application—not a general accuracy score for CATS economies. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0167268107001722?utm_source=chatgpt.com)

**What to borrow:** inexpensive adaptive rules, explicit credit constraints, heterogeneous firms, and failure propagation.

**Main caution:** identify the exact paper and variant. “Implement CATS” is not a sufficiently precise coding task.

### Mark-0: a valuable diagnostic model, not a complete economy

Mark-0 deliberately simplifies the Mark I approach, including aggregating households. It demonstrates that small changes in hiring/firing asymmetry and financial constraints can move the economy between qualitatively different regimes, including high employment, unemployment, and endogenous crises. [arXiv](https://arxiv.org/html/1307.5319)

**What to borrow:** the practice of mapping a model’s behavioral regimes before calibration.

**What not to borrow:** its aggregation as a substitute for TCE’s individual households, products, geography, and material flows.

### Keynes-meets-Schumpeter, or K+S: the best growth-and-investment reference

K+S connects demand-driven business fluctuations with innovation-driven growth. The 2010 model emphasizes complementarities between demand generation and technological change; the 2013 credit-augmented version adds banking and monetary institutions. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S016518891000148X?utm_source=chatgpt.com)

Its productive structure distinguishes **machine-producing firms** from **consumption-good firms**. Innovation changes available equipment, while investment and production decisions depend on economic conditions. The credit extension studies how financial conditions and income distribution affect recessions and recovery. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0165188913000213?utm_source=chatgpt.com)

**What to borrow:** heterogeneous tools and capital vintages, replacement versus expansion investment, innovation and imitation, and demand-sensitive adoption.

**What not to borrow wholesale:** a two-sector industrial structure or continuously operating research-and-development departments.

**Assessment:** excellent for TCE’s later technological development, but excessive as the first implementation of farming and workshops.

### Axtell’s endogenous firms: the best organizational reference

Axtell models firms as organizations emerging from the decisions of individual people. Agents adjust effort, move between firms, and establish new ones. Persistent individual adjustment can coexist with stable aggregate distributions. The working paper reports simulations at the scale of **120 million workers**. [SSRN](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=2827059)

The limitation is crucial: the related Emergent Firms literature explicitly distinguishes this organizational model from a complete production-consumption economy. Output sharing, worker utility, and mobility are modeled, but the sale of output and the source of consumption goods are not a fully closed macroeconomic circuit. [Springer](https://link.springer.com/article/10.1007/s10614-020-10064-8)

**What to borrow:** person-founded firms, teams, effort, mobility costs, social recruitment, and organizational dissolution.

**What not to borrow directly:** treating a firm’s theoretical output as automatically realized monetary revenue.

**Assessment:** an organizational module to place inside TCE’s economy—not the economy itself.

---

## 1.2 Firm heuristics that fit TCE

The rules below are **proposed TCE engineering rules**, informed by those precedents. Their numerical settings require calibration.

### Demand estimation: distinguish demand from sales

Maintain separate quantities for:

**Sales**, **budget-backed purchase requests**, **stockout time**, **unfilled commitments**, and **repeat customers**.

A seller that sells its final loaf at noon has not observed the full day’s potential demand. Conversely, ten shoppers each checking six bakeries have not created sixty independent orders.

Use a smoothed demand estimate:

\[
\hat d\_{t+\Delta}
=
(1-\alpha)\hat d\_t+\alpha d\_t^{\mathrm{observed}},
\qquad
\alpha=1-e^{-\Delta/\tau\_d}
\]

Here, demand is measured in **units per simulated day**, and \(\tau\_d\) is a memory time in days.

The time-based coefficient matters: changing simulation speed or update frequency should not silently make firms more reactive.

For farms, use a **seasonal forecast**, not merely an exponential average of recent harvest sales.

### Inventories and production: use replenishment targets

A useful starting policy is:

\[
I^\star
=
\text{forecast demand over replenishment lead time and review interval}
+
\text{safety stock}
\]\[
Q^{\mathrm{requested}}
=
\max(0,I^\star-I^{\mathrm{position}})
\]

Inventory position includes usable stock, confirmed incoming supply, and existing backorders, with every commitment counted exactly once.

Then constrain production by the actual recipe:

\[
Q^{\mathrm{feasible}}
=
\min
\left(
Q^{\mathrm{labor}},
Q^{\mathrm{materials}},
Q^{\mathrm{equipment}},
Q^{\mathrm{land/season}},
Q^{\mathrm{working\ capital}}
\right)
\]

This is not a generic “productivity multiplier.” A smith without iron cannot compensate by hiring more smiths.

**Separate inventory policies by purpose.** Retail stock, seed grain, winter fodder, fuel, and production inputs need different horizons. Seed reserves should normally be protected, but desperate households may consume them—with consequences for the next harvest.

### Prices: prefer a cost anchor with a restrained scarcity response

Three practical options are available:

| Technique | Mechanism | Strength | Main failure mode |
| --- | --- | --- | --- |
| **Adaptive markup** | Posted price follows estimated unit cost plus an evolving margin. | Understandable; works across production chains. | Bad cost accounting can generate a price spiral. |
| **Inventory feedback** | Short stocks raise prices; excess stocks lower them. | Responds directly to local scarcity. | Delayed replenishment plus aggressive updates causes oscillation. |
| **Price/quantity switching** | Adjust either production or price depending on sales and relative price. | Cheap and interpretable. | Can depend too heavily on an unrealistically informative market average. |

The Mark I-derived four-case rule is instructive: strong sales at a relatively high price encourage more production; strong sales at a relatively low price encourage a price increase; weak sales at a low price encourage less production; weak sales at a high price encourage a price cut. [arXiv](https://arxiv.org/html/1307.5319)

For TCE, I recommend **adaptive markup plus modest inventory feedback**:

\[
p^\star=(1+m)\bar c\,e^{\beta e\_I}
\]\[
\ln p\_{\mathrm{new}}
=
\ln p+
\operatorname{clip}
\left[
\lambda(\ln p^\star-\ln p),
-r\Delta,
+r\Delta
\right]
\]

Here, \(\bar c\) is smoothed normal unit cost, \(m\) is the firm’s markup, and \(e\_I\) is a bounded inventory-shortage signal. The parameter \(r\) limits adjustment per unit of simulated time.

**Do not calculate normal unit cost as this week’s total costs divided by this week’s near-zero output.** That turns temporary idleness into enormous prices, which suppress sales further.

Normal price-setting should also be distinct from clearance sales and liquidation. A failing firm must sometimes sell below its historical cost.

### Wages and hiring: contracts, vacancies, and hysteresis

Calculate desired labor from planned feasible production, then compare it with contracted labor.

Raise an offered wage when a vacancy remains unfilled **and the expected production can support the wage**. Do not raise wages indefinitely when the real obstacle is unavailable housing, missing skills, impassable transport, or absent inputs.

Use different employment arrangements for permanent workshop staff, apprentices, household labor, casual day labor, and seasonal harvest work.

For ordinary adjustments, apply **hysteresis**: require a persistent signal before reversing a recent hiring or firing decision. A two-person workshop should not alternate between one and two workers every day because an unrounded target fluctuates around 1.5.

However, smoothing must not authorize unpaid payroll forever. Insolvency and genuine seasonal endings remain valid reasons for rapid contraction.

The wage-deflation mechanism documented in JAMEL is a warning against using economy-wide wage cuts as the default cure for unemployment. [Cambridge University Press](https://www.cambridge.org/core/journals/macroeconomic-dynamics/article/abs/flexibility-of-wages-and-macroeconomic-instability-in-an-agentbased-computational-model-with-endogenous-money/890FC15C5058C2C03459231FD6213010)

### Search and matching: bounded, local, persistent

Use remembered suppliers and employers, supplemented by a small amount of exploration.

A shopping or job-search decision should inspect a bounded set of plausible alternatives, not every firm in the world. Rank candidates by relevant combinations of delivered price, travel time, reliability, quality, skill fit, contractual terms, and relationships.

Execute matching through a small number of rounds:

**Quote or vacancy → application/order → reservation → acceptance → settlement.**

Reservations prevent two households from buying the same sack or two employers from simultaneously acquiring the same worker’s full labor time.

Do not make the lowest price win every purchase. That creates brittle winner-take-all markets and eliminates the persistent relationships that make bounded search meaningful.

---

## 1.3 Household heuristics

**Use households as budget-sharing and provisioning units while retaining individuals as workers, dependents, owners, and decision participants.** This reduces duplicated decisions without replacing people with population cohorts.

A practical household controller should:

1. Forecast essential consumption and predictable obligations.
2. Allocate accessible money and goods between immediate needs and precautionary reserves.
3. Substitute among acceptable products according to delivered cost and utility.
4. Purchase discretionary goods only from the remaining budget.

Buffer targets should depend on exposure to unemployment, harvest timing, dependents, storage possibilities, and access to kin support—not a universal savings percentage.

The controller should permit distress behavior: drawing down reserves, selling assets, seeking temporary work, requesting assistance, or migrating. It should not preserve a discretionary savings target while children starve.

For early agrarian TCE, households must also produce, store, exchange, and consume their own output. **Wage employment is one livelihood, not the universal economic primitive.**

---

# 2. Trade-offs: complexity, maturity, and performance

## 2.1 Engineering comparison

The rankings below are my assessment of **implementation burden for TCE**, not published comparative scores.

| Approach | Relative implementation burden | Research maturity | Best use in TCE |
| --- | --- | --- | --- |
| Lightweight inventory/markup economy | Low–medium | Mechanisms have substantial precedent; the particular combination is new. | Initial market economy. |
| JAMEL-style monetary economy | Medium | Established research framework and papers. | Accounting, incomes, expenditure, production timing. |
| CATS/BAM-style economy | Medium | Long research lineage; variants differ materially. | Transparent adaptive rules and later credit dynamics. |
| Full Eurace@Unibi-style system | High | Extensive documentation and policy applications. | Reference library for specific subsystems. |
| K+S-style growth economy | High | Established growth/business-cycle research. | Later investment and technological change. |
| Axtell-style organizational dynamics | Medium as a module | Substantial firm-dynamics research. | Formation, mobility, effort, and dissolution. |

**Complexity is not only CPU cost.** Each additional institution increases the number of causal interactions a solo developer must diagnose. A bank, tax system, benefit rule, bankruptcy process, and investment market can each be individually reasonable while their timing creates an unintended feedback loop.

## 2.2 What the available scale evidence does—and does not—show

| Evidence | Reported scale or setup | Interpretation |
| --- | --- | --- |
| **Axtell endogenous firms** | **120 million workers** reported in the working paper. | Strong evidence that simple individual organizational decisions can scale. Not a rendered, spatial, goods-and-finance economy benchmark. [SSRN](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=2827059) |
| **CATS validation** | **6,422 firms**, initialized from historical data for a 1996–2001 comparison. | An empirical-validation sample, **not a frame-time benchmark**. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0167268107001722?utm_source=chatgpt.com) |
| **krABMaga benchmark suite** | Configurations from **1,000 to 128,000 agents**, generally **200 steps**, with repeated runs. | Useful framework benchmarking precedent; workloads are not equivalent to TCE’s economy. [Krabmaga](https://krabmaga.github.io/benchmarks/) |
| **BAM Engine** | Public cross-framework comparison harness and behavioral-equivalence checks are documented. | Useful for learning how to compare implementations without changing model behavior. I could not verify its committed timing results through the accessible pages. [GitHub](https://github.com/kganitis/bam-engine) |

I did not find a directly comparable benchmark establishing that any of the five principal model families delivers **50,000 spatial individuals plus a UE5.8 renderer at 60 fps on your hardware**. Treat population counts, simulated periods per second, and rendered frame rates as different measurements.

## 2.3 TCE’s computational scaling

With local indexes and bounded search, a purchasing pass can be approximately:

\[
O(HGk)
\]

where \(H\) is the number of purchasing households, \(G\) the product categories considered, and \(k\) the suppliers inspected.

For illustration, **50,000 buyers × 8 categories × 6 candidates = 2.4 million quote evaluations per full purchasing round**. That is a workload calculation, not a measured runtime—and household purchasing should require fewer buyers than the total population.

The dangerous alternatives are all-to-all matching, repeated global sorting, per-agent pathfinding for every quote, and unrestricted negotiation rounds.

**Keep the economy on the CPU initially.** My recommendation is to spend the RTX 4070 Ti’s resources on rendering. Financial settlement and irregular ownership changes are poor places to introduce GPU synchronization and debugging complexity before profiling proves a need.

---

# 3. Precedents: code, engines, and shipped games

## 3.1 Research code worth studying

**JAMEL** is useful for tracing complete monetary transactions and production decisions through source code. Its GPL-3.0 license needs explicit review before code reuse; studying its mechanisms and embedding its implementation are different decisions. [GitHub](https://github.com/pseppecher/jamel)

**BAM Engine 0.10.2**, released July 9, 2026, is an independent Python implementation of the 2011 BAM baseline. It uses parallel NumPy arrays and an entity-component-system organization, with sequential goods matching where order matters. It also includes validation and calibration tooling. The project marks its pre-1.0 API as changeable. This is a useful executable reference, not evidence of a mature production-game dependency. [PyPI](https://pypi.org/project/bamengine/)

**JAS-mine MacroABM** provides accessible Java code inspired by K+S. Its documentation explicitly calls it a reinterpretation rather than an exact port. That is an important precedent: two implementations with similar labels may embody different economic assumptions. [GitHub](https://github.com/jasmineRepo/MacroABM)

**krABMaga** offers a Rust ABM framework with headless simulation support and optional visualization. It is worth inspecting for scheduling and experiment organization. For TCE, I would still favor a narrow custom kernel over adopting a second general-purpose visualization architecture alongside Unreal. [GitHub](https://github.com/krABMaga/krABMaga?utm_source=chatgpt.com)

## 3.2 Games: useful lessons, not macroeconomic validation

### Victoria 3: economic heuristics require repeated correction

Paradox’s **Update 1.2** documentation changed wage behavior to condition increases on profitability and employment circumstances, and reductions or layoffs on deficits and cash reserves. It also changed GDP accounting to subtract intermediate input costs. These are concrete examples of a shipped economic simulation revising both behavioral rules and accounting definitions. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-78-update-1-2-changelog?utm_source=chatgpt.com)

**Lesson for TCE:** expose the precise reason for each wage adjustment, layoff, and reported aggregate. “Profitability fell” is not enough; show whether the cause was input prices, demand, transport, wages, or utilization.

These are **historical 1.2 rules**, not a claim about the current 2026 implementation.

### X4: Foundations: production chains and autonomous logistics are viable game content

Egosoft describes thousands of ships and stations carrying out tasks, factions trading and expanding independently, and interconnected production chains. This is a strong precedent for making autonomous economic activity visible and consequential. It is not a demonstration of a complete household labor-and-consumption macroeconomy. [Steam Store](https://store.steampowered.com/app/392160/X4_Foundations/)

**Lesson for TCE:** physical logistics should be economically authoritative. A profitable order is not fulfilled until the required goods can actually reach their destination.

### Songs of Syx: individual-scale settlement simulation is feasible for a small team

The developer describes a one-person studio and a city-builder simulating thousands of citizens and soldiers, with production disruptions capable of cascading into collapse. The accessible Steam listing still identifies it as Early Access. [Steam Store](https://store.steampowered.com/app/1162750/Songs_of_Syx/?utm_source=chatgpt.com)

**Lesson for TCE:** this is encouraging evidence for population scale and visible logistics under tight development constraints. It does not establish the feasibility of the same population with TCE’s rendering, autonomous ownership, and institution-generation requirements.

---

# 4. Recommended TCE design, stabilizers, and validation

## 4.1 Make accounting stricter than behavior

Agents may be mistaken, shortsighted, biased, or badly informed. **The transaction system must not be.**

Maintain two connected but distinct systems:

**Physical accounts:** goods, land access, equipment, work in progress, transport cargo, reservations, spoilage, consumption, and recipe transformations.

**Financial accounts:** cash or tokens, deposits, loans, obligations, ownership shares, payments, and explicit default losses.

Stock-flow-consistent modeling provides a methodological precedent for making real and financial interactions coherent; Caiani and colleagues’ benchmark is a useful companion to JAMEL here. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0165188915301020?utm_source=chatgpt.com)

Recommended invariants include:

| Invariant | Required behavior |
| --- | --- |
| No double spending | Reserve spending capacity before accepting concurrent commitments. |
| No duplicated goods | A shipment or sale transfers ownership/location; it does not copy inventory. |
| No free production | Consume recipe inputs and labor capacity before recognizing output. |
| No inconsistent debt | Every financial claim has the appropriate corresponding obligation. |
| No disappearing assets at bankruptcy | Transfer, liquidate, abandon, or destroy assets explicitly. |
| No fictitious income from repricing | Inventory revaluation is not a cash receipt. |
| No double-counted employment | An individual’s contracted time cannot exceed available time without an explicit overtime mechanism. |

Do **not** enforce constant money as a universal invariant. An institution may legitimately issue money or credit; what matters is that creation, repayment, and write-off have explicit rules and counterpart entries.

Similarly, internal accounting consistency does not require ancient households to possess historically anachronistic bookkeeping knowledge. The kernel’s records and the agents’ information are separate.

## 4.2 Stabilize decision processes, not outcomes

| Instability | Typical mechanism | Recommended response |
| --- | --- | --- |
| **Price oscillation** | All sellers react strongly to the same lagged shortage. | Stagger reviews, smooth observations, limit adjustment rates, model delivery lead times. |
| **Production collapse after stockouts** | Firms interpret constrained sales as weak demand. | Record stockouts and budget-backed unmet requests. |
| **Wage-price spiral** | Wage and price controllers chase one another without productivity or affordability checks. | Different review clocks, contractual stickiness, normal-cost estimates, financially feasible offers. |
| **Deflationary demand spiral** | Income reductions suppress spending, causing further layoffs and reductions. | Permit reserves, alternative livelihoods, transfers, and relief where institutions support them—not automatic universal bailouts. |
| **Credit cascade** | Defaults damage creditors, which contract lending to otherwise viable producers. | Explicit exposure limits, loss allocation, restructuring, and institution-dependent resolution. |
| **Supply-chain deadlock** | Every producer waits for another producer’s output or payment. | Seed initial inventories, permit funded advances, and model working capital and fallback recipes. |
| **Winner-take-all monopoly** | All buyers instantly select one marginally cheaper seller. | Bounded information, switching costs, capacity limits, reliability, geography, and entry. |
| **Endless compounding** | Productivity or prices grow by a fixed amount every tick. | Express rates in simulated time; require resources and opportunities for innovation and adoption. |

Mark-0’s regime changes are a warning that gentle-looking local rules can still produce abrupt macroeconomic transitions. Rate limits are useful engineering controls, but they are not a proof of system-wide stability. [arXiv](https://arxiv.org/html/1307.5319)

**Recovery must have an economic mechanism.** Workers can leave a failed workshop and farm only if land, skills, season, tools, and access rights allow it. A new mill can replace a failed monopoly only if someone can acquire the site, capital, labor, and knowledge.

Many research experiments simplify entry or keep an agent population fixed. TCE should not silently replace bankrupt firms with free, fully equipped clones.

## 4.3 Proposed initial tuning ranges

These are **low-confidence starting ranges for experiments**, not estimated historical constants. They apply to ordinary market workshops and households; agriculture, casual labor, and crises need their own policies.

| Parameter | Initial exploration range | Purpose |
| --- | --- | --- |
| Supplier candidates per purchase | 4–8 | Bound search while allowing competition. |
| New job candidates per search event | 3–8 | Avoid global labor-market scans. |
| Demand-estimation memory | 14–56 simulated days | Filter short-lived sales noise. |
| Ordinary price-review interval | 3–14 simulated days | Prevent every transaction from repricing the economy. |
| Ordinary price adjustment limit | 1–5% per review | Explore sensitivity without immediate jumps. |
| Persistent mismatch before ordinary staffing reversal | 1–4 reviews | Reduce hire/fire oscillation. |
| Routine working-capital target | 1–3 production/payroll cycles | Cover the delay between paying inputs and receiving revenue. |
| Nonseasonal safety stock | 0.5–2 replenishment lead times of demand | Absorb delivery uncertainty. |

A grain store intended to last until the next harvest does **not** use the last row. Nor should emergency scarcity necessarily obey the same price-change limit as routine market conditions.

## 4.4 Runtime architecture for Rust and Unreal Engine 5.8

**Keep one authoritative economy in Rust.** Unreal should submit commands and consume snapshots, not maintain a second implementation of money, inventories, or employment.

A suitable flow is:

```
Unreal input
    → timestamped command queue
    → Rust simulation and transaction commit
    → immutable presentation snapshot
    → Unreal representation and interpolation
```

Use an opaque-handle C interface with explicit ABI versions, fixed-layout data, and ownership rules. Keep allocation and deallocation paired on the same side, and do not allow Rust panics to unwind across an ordinary C boundary. Epic’s third-party integration documentation covers DLL loading and packaging; the Rustonomicon documents the relevant FFI constraints. [Epic Games Developers](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine)

Recommended implementation choices:

**Data layout.** Store frequently accessed economic fields in compact arrays. Use stable IDs with generation counters rather than cross-module pointers.

**Scheduling.** Daily provisioning, weekly business reviews, seasonal farming, and infrequent investment should have different clocks. Spread decision dates across agents instead of running an entire economy at midnight.

**Parallelism.** Begin with a deterministic single-threaded transaction commit. Parallelize read-only calculations and intent generation first. Deterministically resolve competing intents before mutation.

**Reproducibility.** Key randomness to stable identities and event counters; do not let thread scheduling determine who receives the final sack of grain.

**Long runs.** Keep bounded rolling statistics in memory. Stream or compact older history; otherwise centuries of transaction logging will dominate memory.

At **60 fps, the complete frame budget is 16.67 ms**. As an initial engineering target—not a measured result—allocate **less than 1 ms at the 99th percentile to main-thread economic snapshot consumption**. Benchmark the worker separately in simulated days per real second at several time accelerations.

Do not assume that an economically cheap person is visually cheap. UE5.8’s documented crowd workflows transition between high-fidelity Actors and lower-fidelity instanced representations. That supports a presentation-LOD approach, but does not guarantee your 1440p frame target. [Unreal Engine](https://www.unrealengine.com/news/unreal-engine-5-8-is-now-available)

## 4.5 Calibration: match mechanisms and distributions, not one GDP curve

### What the literature establishes

The models reproduce different subsets of observed regularities:

| Model family | Relevant reported evidence |
| --- | --- |
| **Eurace@Unibi** | Benchmark configurations reproduce persistent output fluctuations, differing consumption/investment amplitudes, firm heterogeneity, and labor-market patterns including the Beveridge relationship. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/roie.12317?utm_source=chatgpt.com) |
| **JAMEL-related models** | Decentralized multisector price coordination and adjustment to technology shocks; other versions generate wage-deflation and deleveraging crises. [OUP Academic](https://academic.oup.com/icc/article-abstract/27/6/1045/5035120?utm_source=chatgpt.com) |
| **CATS** | Firm-output and growth-distribution validation against a specified Italian firm sample, with limitations at size extremes. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0167268107001722?utm_source=chatgpt.com) |
| **K+S** | Joint innovation, growth, business-cycle, and financial dynamics; results depend on the particular baseline and extension. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S016518891000148X?utm_source=chatgpt.com) |
| **Axtell / Emergent Firms** | Organizational dynamics, mobility, and inequality mechanisms, but not full goods-market closure. [Springer](https://link.springer.com/article/10.1007/s10614-020-10064-8) |

**Reproducing a stylized fact is not unique identification of its cause.** Several different mechanisms can generate skewed firm sizes or volatile investment. TCE needs micro-level checks as well as aggregate resemblance.

### Recommended calibration sequence

**First: physical feasibility.** Verify labor requirements, food availability, production lead times, storage, transport, and seasonal constraints before tuning prices.

**Second: isolated behavioral tests.** Test one household, one shop, one employer, and one supply chain under controlled conditions. A profitable, fully supplied bakery should not autonomously choose permanent inactivity.

**Third: parameter-space exploration.** Map prosperous, oscillatory, stagnant, and collapsing regimes across demand memory, adjustment rates, buffers, search intensity, and credit restrictions. Preserve failed runs in the results.

**Fourth: simulated moments.** Compare distributions and time-series characteristics: stockout frequency, household reserves, firm survival, utilization, price dispersion, real consumption, unemployment spells, and sectoral labor shares. Normalize discrepancies by meaningful tolerances rather than allowing whichever statistic has the largest units to dominate.

**Fifth: held-out worlds.** Validate on unseen seeds, settlement sizes, transport networks, and resource distributions. Calibrating one fertile river valley does not validate an isolated mountain settlement.

Machine-learning surrogates can later reduce the cost of parameter exploration. Lamperti, Roventini, and Sani demonstrate this approach, but a surrogate should accelerate experiments—not replace checking candidate settings in the actual simulator. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0165188918301088?utm_source=chatgpt.com)

For TCE’s early societies, prioritize **food security, seasonal labor bottlenecks, storage, relative prices, and household resilience**. Do not force a modern inflation target, industrial growth rate, or unemployment relationship onto an agrarian economy.

### Minimum stress-test suite

Use a compact set of synthetic scenarios with explicit expected mechanisms: a harvest shock, loss of the sole input supplier, transport interruption, a large employer’s failure, a productivity improvement, migration, and an institutional change.

For each, record the causal chain from event to inventories, production, incomes, purchases, prices, and recovery. Also run a frozen-technology, benign-environment baseline for centuries. Unexplained trend growth or collapse in that baseline is a diagnostic signal.

At 10k–50k people, be cautious about fitting asymptotic firm-size laws: some worlds will contain too few substantial firms to estimate a meaningful tail.

## 4.6 A solo-developer implementation order

**Stage 1 — Viable subsistence.** Household production, consumption, seasonal stocks, ownership, and transfers. No banking required.

**Stage 2 — Local markets.** Posted prices, bounded shopping, funded settlement, workshops, wages, and failure. Establish a stable benign baseline.

**Stage 3 — Endogenous organization.** Founding, recruitment, apprenticeships, ownership changes, household enterprises, and migration.

**Stage 4 — Capital and knowledge.** Tools, capacity, replacement investment, recipe adoption, and technology diffusion.

**Stage 5 — Institutional finance.** Credit, intermediaries, taxes, public spending, insolvency rules, and alternative monetary arrangements when the simulated society supports them.

For AI coding agents, define the transaction invariants and model rules as versioned specifications. Require regression tests for every behavioral change. A coding agent should not “fix” a recession by weakening conservation checks, granting free credit, or changing the demand formula without an explicit model decision.

---

# 5. Sources, code, and version scope

These are the most useful implementation references. Model dates identify the version of the argument being discussed; they are not claims that the research stopped at that date.

| Reference | Version/date and recommended use |
| --- | --- |
| [Eurace@Unibi v1.0 User Manual](https://noah.nrw/ubbihs/content/titleinfo/5127756?utm_source=chatgpt.com) | **2011.** Detailed market protocols, functions, and accounting checks. |
| [Dawid et al., *Macroeconomics with heterogeneous agent models*](https://link.springer.com/article/10.1007/s00191-018-0594-0?utm_source=chatgpt.com) | Published online **2018**, journal volume **2019**. Best EURACE documentation/reproducibility reference. |
| [JAMEL source](https://github.com/pseppecher/jamel?utm_source=chatgpt.com) | Public Java repository; **GPL-3.0**. Pin a commit rather than treating the branch as a stable model version. |
| [Seppecher, wage flexibility and instability](https://doi.org/10.1017/S1365100511000447) | **2012.** Wage adjustment, endogenous money, and destabilizing demand feedback. |
| [Seppecher, Salle & Lavoie, *What drives markups?*](https://academic.oup.com/icc/article-abstract/27/6/1045/5035120?utm_source=chatgpt.com) | **2018.** Multisector adaptive pricing and coordination. |
| [Bianchi et al., CATS validation](https://www.sciencedirect.com/science/article/abs/pii/S0167268107001722?utm_source=chatgpt.com) | **2008**, empirical comparison for **1996–2001**. Useful validation precedent. |
| [Delli Gatti et al., *Macroeconomics from the Bottom-up*](https://doi.org/10.1007/978-88-470-1971-3) | **2011.** BAM baseline; distinguish it from earlier CATS variants. |
| [Gualdi et al., *Tipping points in macroeconomic Agent-Based models*](https://arxiv.org/abs/1307.5319) | **2013–2014 preprint versions.** Mark I/Mark-0 rules, pseudocode, and regime analysis. |
| [Dosi, Fagiolo & Roventini, K+S baseline](https://www.iris.sssup.it/handle/11382/302310?utm_source=chatgpt.com) | **2010.** Innovation, demand, investment, and growth. |
| [Dosi et al., credit-augmented K+S](https://www.sciencedirect.com/science/article/pii/S0165188913000213?utm_source=chatgpt.com) | **2013.** Financial extension; not identical to the baseline. |
| [Axtell, *Endogenous Dynamics of Multi-Agent Firms*](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=2827059&utm_source=chatgpt.com) | **2016 working-paper posting.** Organizational formation and large-scale worker simulation. |
| [Applegate & Janssen, *Job Mobility and Wealth Inequality*](https://link.springer.com/article/10.1007/s10614-020-10064-8?utm_source=chatgpt.com) | Published online **2020**. Extension and explicit discussion of the Emergent Firms model’s scope. |
| [Caiani et al., AB-SFC benchmark](https://doi.org/10.1016/j.jedc.2016.06.001) and [corrigendum](https://doi.org/10.1016/j.jedc.2017.06.006) | **2016**, corrected **2017**. Accounting, calibration, and validation methodology. |
| [BAM Engine](https://pypi.org/project/bamengine/0.10.2/) and [source](https://github.com/kganitis/bam-engine?utm_source=chatgpt.com) | **0.10.2, July 9, 2026.** Independent executable reference; MIT license; pre-1.0 API. |
| [JAS-mine MacroABM](https://github.com/jasmineRepo/MacroABM?utm_source=chatgpt.com) | **2016/2018 implementation lineage.** K+S-inspired reinterpretation, not an exact replication. |
| [krABMaga](https://github.com/krABMaga/krABMaga?utm_source=chatgpt.com) and [benchmarks](https://krabmaga.github.io/benchmarks/?utm_source=chatgpt.com) | Repository and benchmark documentation inspected in **2026**. Rust scheduling and experiment infrastructure. |
| [Lamperti et al., machine-learning calibration surrogates](https://www.sciencedirect.com/science/article/abs/pii/S0165188918301088?utm_source=chatgpt.com) | **2018.** Later-stage acceleration of calibration experiments. |
| [Herbert Dawid’s WIFO talk](https://www.wifo.ac.at/en/event/260944/?utm_source=chatgpt.com) | **September 24, 2019.** Talk listing and abstract on Eurace@Unibi policy analysis; not a verified video recording. |
| [Epic third-party library integration](https://dev.epicgames.com/documentation/en-us/unreal-engine/integrating-third-party-libraries-into-unreal-engine?utm_source=chatgpt.com) and [Rust FFI](https://doc.rust-lang.org/nomicon/ffi.html?utm_source=chatgpt.com) | **UE5.8 documentation** and live Rust documentation inspected in **2026**. DLL integration and ownership boundaries. |

**Bottom line:** TCE’s most defensible foundation is a physically grounded household economy with exact transactions, local information, restrained adaptive controls, and explicit recovery mechanisms. Add endogenous firms and technological investment to that foundation. The goal is not an economy that never suffers—it is an economy whose prosperity, crises, and recoveries can all be explained by inspectable rules rather than numerical accidents.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92876-5ef8-83ea-95ac-33754cb58f1a)
