# Bounded rationality and plausible human decision-making for TCE

## Executive recommendation

**Build citizens who are competent in familiar situations, limited in what they know, slow to reconsider successful routines, and selectively inconsistent—not optimizers with a random “stupidity” penalty.**

For TCE, I recommend this decision sequence:

**Recognize the situation → check urgent constraints → continue a routine or commitment → search a few familiar alternatives → accept something satisfactory → deliberate more carefully only when stakes, disappointment, or unfamiliarity justify it.**

This combines Simon’s bounded rationality with research on adaptive heuristics, habitual behavior, and experience-based learning. Simple decision rules can perform well when they exploit the structure of a familiar environment; their limitations become visible when circumstances change. [OUP Academic](https://academic.oup.com/qje/article-abstract/69/1/99/1919737)

An essential distinction is between **behavioral bias** and **a reasonable response to circumstances**. A farmer declining a profitable innovation may lack insurance, seed reserves, trustworthy information, or the ability to survive its failure. Ghanaian field experiments show that removing uninsured agricultural risk can change investment substantially without changing anyone’s underlying psychology. [National Bureau of Economic Research](https://www.nber.org/papers/w18463?utm_source=chatgpt.com)

The strongest initial implementation would therefore emphasize **limited knowledge, habits, satisficing, household constraints, and social learning**. Add prospect theory and present bias selectively, with domain-specific calibration.

---

## 1. Mechanisms: rules that agents can implement

### 1.1 Limited consideration and satisficing

An optimizing agent evaluates every feasible alternative and selects the maximum. A satisficing agent searches until it finds an alternative that meets an aspiration level. Simon’s contribution is not simply “choose imperfectly”: the search procedure, information limitations, and stopping rule are part of the model. [OUP Academic](https://academic.oup.com/qje/article-abstract/69/1/99/1919737)

**TCE implementation**

Maintain an aspiration level for each major domain: food security, work returns, housing, social acceptance, and so forth. Generate candidates from what the person actually knows: previous actions, visible opportunities, recommendations, and learned practices.

Evaluate candidates in a meaningful order—familiarity, proximity, trusted recommendation, or expected usefulness—and stop at the first satisfactory option:

\[
\text{accept }a \quad\text{when}\quad
\widehat V\_i(a)\geq A\_{i,d}
\]

A simple proposed aspiration update is:

\[
A\_{i,d,t+1}=A\_{i,d,t}+\eta\_A\left(y\_{i,d,t}-A\_{i,d,t}\right)
\]

Here \(y\) is an experienced outcome, not the best outcome available anywhere in the world.

Keep **aspirations separate from physiological requirements**. Someone may become accustomed to poor housing; the body should not adapt its minimum nutritional requirements downward merely because expectations fall.

When no candidate satisfies the aspiration, the person can broaden the search, ask for advice, accept the least-bad known option, or revise expectations. Which response occurs should depend on urgency, available time, and previous success with searching.

**Expected simulation consequence:** people persist with adequate jobs, suppliers, fields, and routines despite better alternatives elsewhere. Disappointment produces bursts of search rather than constant optimization.

### 1.2 Heuristics should exploit local expertise

Fast-and-frugal heuristics use limited information in structured ways—for example, consulting cues in order of their predictive usefulness and stopping when a sufficiently informative cue distinguishes the alternatives. Such methods are not equivalent to random guessing. [PubMed](https://pubmed.ncbi.nlm.nih.gov/8888650/)

**TCE implementation**

Give agents a small repertoire of context-specific rules, such as:

| Situation | Example rule |
| --- | --- |
| Familiar food acquisition | Return to a recently productive location unless travel costs or depletion have changed substantially. |
| Choosing an unfamiliar supplier | Prefer a supplier endorsed by someone whose recommendations have worked before. |
| Evaluating an agricultural practice | First reject practices incompatible with local soil, water, or available labor; then compare expected yield. |
| Facing an unfamiliar administrative choice | Follow a trusted default unless there is a salient reason not to. |
| Emergency | Use a recognized response immediately rather than undertake an exhaustive comparison. |

These are **proposed rule templates**, not universal ethnographic laws.

Let experience improve cue reliability within a domain. A highly competent farmer need not be equally competent at evaluating loans, unfamiliar diseases, or political promises. Do not represent education or technological development as a universal reduction in decision noise.

### 1.3 Subjective beliefs, limited samples, and recency

Separate the world’s true probabilities from agents’ beliefs about them.

A particularly important finding is the **description–experience gap**. In experiments, people choosing from explicitly described probabilities often behave as though rare events receive excessive weight. People learning through sampled experience can instead behave as though rare events receive too little weight, partly because samples are small and recent observations matter disproportionately. [Sage Journals](https://journals.sagepub.com/doi/10.1111/j.0956-7976.2004.00715.x)

**TCE implementation**

Store compact outcome memories, tagged by activity, context, source, and age. A cheap belief update is:

\[
\widehat q\_{t+1}=(1-\eta\_B)\widehat q\_t+\eta\_B y\_t
\]

Use this as an engineering approximation, not as a claim that people literally perform exponential smoothing.

Distinguish three information channels: personal experience, reports from other people, and explicit records or descriptions. A repeatedly retold story should not count as several independent observations.

For scale, a genuinely independent event with a 2% probability will be absent from approximately **67% of samples containing only 20 observations**, since \(0.98^{20}\approx0.67\). This is a mathematical illustration: apparent neglect of rare danger can arise without adding a special “ignore danger” bias.

**Important:** simulate outcomes using the world’s probabilities. Apply imperfect beliefs only to decisions.

### 1.4 Habit, inertia, and status quo bias are different mechanisms

Repeated behavior in stable contexts can become habitual; diary research distinguishes such behavior from more actively considered actions. Habit acquisition is gradual and variable rather than governed by a universal number of days. [DOI](https://doi.org/10.1037/0022-3514.83.6.1281)

**TCE implementation**

Represent a habit as an association between a context and an action:

```
context: morning + household location + workday
action: eat stored meal, collect tools, walk to customary workplace
```

A lightweight acquisition rule is:

\[
H\_{n+1}=H\_n+a\_H(1-H\_n)
\]

where \(n\) counts relevant repetitions in a sufficiently similar context. This is a proposed approximation to asymptotic acquisition.

Allow a habit to execute without a new utility comparison when its cues match, it remains feasible, and no urgent interruption applies. Reconsideration can be triggered by repeated failure, relocation, changed household responsibilities, a salient opportunity, or a broken resource link.

Keep three sources of persistence distinct:

| Mechanism | Representation |
| --- | --- |
| **Habit** | The situation retrieves an action without extensive evaluation. |
| **Real switching costs** | Moving, retraining, searching, losing access, or reorganizing work consumes resources. |
| **Status quo/default effects** | The existing or preselected option affects consideration, interpretation, or reference points. |

Defaults can operate through perceived endorsement, endowment, and ease. Their effects vary substantially across settings. [Cambridge University Press](https://www.cambridge.org/core/journals/behavioural-public-policy/article/when-and-why-defaults-influence-decisions-a-metaanalysis-of-default-effects/67AF6972CFB52698A60B6BD94B70C2C0)

**Avoid double-counting:** a habit bonus, switching penalty, status quo bonus, and loss-aversion penalty should not all be fitted independently to explain the same observed persistence.

### 1.5 Prospect theory: evaluate changes relative to a reference point

Cumulative prospect theory distinguishes gains from losses relative to a reference point, allows diminishing sensitivity, and uses nonlinear probability weighting. These are separate ingredients, not a single “risk aversion” parameter. [Springer](https://link.springer.com/article/10.1007/BF00122574)

A useful starter value function is:

\[
v(z)=
\begin{cases}
z^\alpha,& z\geq0\\
-\lambda(-z)^\alpha,&z<0
\end{cases}
\qquad
z=\frac{x-r\_{i,d}}{s\_d}
\]

Here \(r\_{i,d}\) is the reference outcome, \(s\_d\) a fixed meaningful domain scale, \(\alpha\) curvature, and \(\lambda\) relative sensitivity to losses.

For probability weighting, one established specification is:

\[
w(p)=
\frac{p^\gamma}
{\left[p^\gamma+(1-p)^\gamma\right]^{1/\gamma}}
\]

The familiar Tversky–Kahneman calibration uses \(\alpha=0.88\), \(\lambda=2.25\), and weighting parameters of 0.61 for gains and 0.69 for losses. These are classic experimental estimates, not biological constants. [Springer](https://link.springer.com/article/10.1007/s11166-022-09391-y)

**TCE implementation**

Use reference dependence for consequential choices involving losses of familiar income, property, status, or expected harvests. Do not apply full prospect theory to every movement or household chore.

Choose reference points explicitly. “Last year’s harvest,” “normal household consumption,” and “what was promised” can yield different decisions. For the first version, use one reference rule per domain rather than letting several competing reference points operate simultaneously.

Use **cumulative, rank-dependent weights**, separately for gains and losses. Transforming each outcome’s probability independently and then normalizing is not cumulative prospect theory.

Finally, keep **absolute survival conditions** outside the reference point. Losing half of an abundant harvest and losing half of an already inadequate harvest cannot have identical physical consequences.

### 1.6 Present bias and inconsistent plans

Present bias means that an immediately experienced cost or benefit receives extra weight relative to future ones. It is distinct from ordinary impatience, uncertainty about receiving a payment, or needing resources now.

Experimental results depend on the domain. Augenblick, Niederle, and Sprenger found substantially stronger present bias for real effort than for monetary allocations in their study. [Haas School of Business](https://faculty.haas.berkeley.edu/ned/Working_Over_Time.pdf)

A tractable formulation is:

\[
D(\Delta)=
\begin{cases}
1,&\Delta=0\\
\beta e^{-r\Delta},&\Delta>0
\end{cases}
\]

where \(r\) is the long-run discount rate and \(\beta<1\) captures the additional present–future discontinuity.

**TCE implementation**

An agent may plan to repair a roof next week, but postpone when the effort becomes immediate. A person who recognizes this tendency may prefer a commitment: an agreed work party, a deadline, resources set aside for a purpose, or an obligation carrying social consequences.

Define “now” using the activity’s consumption or action window, not the render frame. Apply \(\beta\) once to the future stream at a decision, **not once for every elapsed day**.

Model spoilage, mortality, liquidity, and counterparty reliability separately. Otherwise, impatience will absorb missing economic mechanisms.

### 1.7 Subsistence risk: model vulnerability before preferences

Uninsured risk can make a lower-average-return activity preferable. Conversely, a household facing an otherwise unavoidable shortfall may rationally accept a dangerous gamble. Neither response requires a change in innate risk preference.

**Proposed illustrative test**

Suppose a household must obtain 100 food units before a deadline:

| Available resources | Safe activity | Risky activity, equal probabilities | Safety-first implication |
| --- | --- | --- | --- |
| No additional reserves | 90 | 60 or 120 | The safe activity always misses the threshold; the gamble sometimes succeeds. |
| 20 units already stored | 110 total | 80 or 140 total | The safe activity meets the threshold; the gamble risks failure. |

The production options have the same expected return. Reserves alone reverse which option best avoids shortfall.

For the actual simulation, use a smooth relationship between shortages and harm rather than treating nutrition as a single cliff. Track household dependents, seed requirements, storage losses, correlated harvest risks, borrowing, and assistance.

Do not expect a monetary lottery coefficient to explain all farming behavior. Field evidence includes both substantial risk aversion and risk-seeking lottery choices among small-scale agricultural populations. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.2307/1240194)

---

## 2. Parameters: empirical anchors and engineering priors

### 2.1 Empirical estimates

Confidence below concerns **usefulness for TCE**, not whether the published number was transcribed correctly. “Moderate” generally means a useful task-specific anchor; transport to ancient societies or different activities remains uncertain.

| Quantity | Estimate and units | Source and scope | Confidence / limitation |
| --- | --- | --- | --- |
| Prospect-theory curvature | \(\alpha\_{\text{gain}}=\alpha\_{\text{loss}}=0.88\), dimensionless | Classic Tversky–Kahneman calibration. [Springer](https://link.springer.com/article/10.1007/s11166-022-09391-y) | Moderate as a benchmark; low as a universal coefficient. |
| Classic loss aversion | \(\lambda=2.25\), dimensionless | Same calibration. [Springer](https://link.springer.com/article/10.1007/s11166-022-09391-y) | Useful for reproducing that model, not a mandatory population mean. |
| Broad pooled loss aversion | Mean \(\lambda=1.955\); 95% interval **1.820–2.102** | Brown et al. (2024): **607 estimates from 150 articles**, spanning several disciplines. [Benny](https://benny.aeaweb.org/articles?id=10.1257%2Fjel.20221698) | Moderate; heterogeneous definitions and estimation contexts. |
| Loss aversion in re-estimated risky-choice data | Pooled \(\lambda=1.31\); 95% CI **1.10–1.53** | Walasek, Mullett & Stewart (2024): **17 studies, 19 datasets**; individual mixed-gamble choices. [Warwick Research Archive Portal](https://wrap.warwick.ac.uk/id/eprint/185745/13/1-s2.0-S0167487024000485-main.pdf) | Moderate; parameter identification and data quality are important limitations. |
| Classic probability weighting | \(\gamma\_{\text{gain}}=0.61\), \(\gamma\_{\text{loss}}=0.69\), dimensionless | Tversky–Kahneman weighting function above. [Springer](https://link.springer.com/article/10.1007/s11166-022-09391-y) | Moderate for described lotteries; weak justification for all experience-based decisions. |
| Present bias over effort | Combined \(\beta=0.888\), SE **0.033** | Augenblick et al. (2015), real-effort tasks, 80 participant clusters in the reported specification. [Haas School of Business](https://faculty.haas.berkeley.edu/ned/Working_Over_Time.pdf) | Moderate; student sample and specific tasks. |
| Present bias over money | \(\beta=0.988\), SE **0.009**, for three-week-delay specification | Same study, 75 participant clusters; other specifications differ. [Haas School of Business](https://faculty.haas.berkeley.edu/ned/Working_Over_Time.pdf) | Moderate; warns against transferring effort estimates directly to money. |
| Habit acquisition | Median approximately **66 days** to 95% of fitted asymptote; range **18–254 days** | Lally et al. (2010), daily health-related behaviors; 96 volunteers followed for 12 weeks. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/ejsp.674?undefined=&utm_campaign=how-long-does-it-take-to-create-a-habit&utm_medium=organic&utm_source=blog) | Moderate for the acquisition pattern; low for universal timing. Long fitted times extend beyond observation. |
| Default effect | Standardized effect \(d=0.68\), 95% CI **0.53–0.83** | Jachimowicz et al. (2019), **58 studies**, pooled \(n=73{,}675\). [Cambridge University Press](https://www.cambridge.org/core/journals/behavioural-public-policy/article/when-and-why-defaults-influence-decisions-a-metaanalysis-of-default-effects/67AF6972CFB52698A60B6BD94B70C2C0) | Strong evidence that defaults can matter; substantial contextual variation. **Not a utility bonus or percentage-point effect.** |
| Risk attitude in particular small-scale populations | Mean certainty equivalent / expected value: **1.40 Mapuche**, **1.37 Sangu**, **0.79 Huinca** | Henrich & McElreath (2002), particular monetary gambles in Chile and Tanzania. Values above 1 indicate risk-seeking in that task. [Amazon Web Services, Inc.](https://s3.amazonaws.com/fieldexperiments-papers2/papers/00066.pdf) | Useful counterexample to universal peasant risk aversion; low transfer beyond these tasks and samples. |

**Do not use a confidence interval around a pooled mean as the distribution of individuals.** A mean near 1.96 with a relatively narrow interval does not imply that nearly everyone’s loss-aversion coefficient lies between 1.82 and 2.10.

The two 2024 loss-aversion syntheses are not interchangeable. One aggregates a broad literature of reported estimates; the other re-estimates a more restricted class of risky-choice datasets. Their disagreement is a reason to test alternative calibrations, not to average their point estimates mechanically. [Benny](https://benny.aeaweb.org/articles?id=10.1257%2Fjel.20221698)

### 2.2 Suggested starting values for TCE

**These are engineering priors for sensitivity analysis, not measured historical distributions.** They are deliberately modest and should be replaced where domain-specific validation supports something better.

| Parameter | Starter | Sensitivity range | Units and intended use |
| --- | --- | --- | --- |
| Loss sensitivity \(\lambda\) | 1.5 | 1.0–2.5; include 0.8 and 3.0 stress tests | Dimensionless; consequential reference-dependent choices. |
| Prospect curvature \(\alpha\) | 0.88 | 0.6–1.1 | Dimensionless; allow linear and mildly convex alternatives in testing. |
| Weighting curvature \(\gamma\) | 0.8 | 0.6–1.0 | Dimensionless, **for the specified weighting function**. |
| Experience-based probability weighting | Identity: \(w(p)=p\) | Compare against nonlinear weighting | Let limited sampling and recency generate distortions first. |
| Present bias for effort / immediate consumption \(\beta\) | 0.9 | 0.8–1.0 | Dimensionless; vary by person and domain. |
| Pure long-run discount rate \(r\) | 0.05 | 0–0.20 | Per year, continuously compounded; a design prior, not an ancient discount-rate estimate. |
| Aspiration updating \(\eta\_A\) | 0.05 | 0.01–0.20 | Fraction of the outcome–aspiration gap per relevant observation. |
| Belief updating \(\eta\_B\) | 0.10 | 0.02–0.30 | Fraction of new evidence incorporated per observation. |
| Considered alternatives | 6 maximum | 3–8 | Candidates per deliberation; a computational choice, not a working-memory law. |
| Stored outcome examples | 16 | 8–32 | Recent or salient examples per active domain. |
| Habit acquisition \(a\_H\) | 0.044 | 0.012–0.153 | Per eligible repetition, **derived** from 66 and 18–254 repetitions to 95% asymptote under the proposed equation. |
| Unstructured lapse probability | 0 | 0–0.02 | Per decision event; introduce only if structured mechanisms leave unexplained inconsistency. |

The habit conversion follows:

\[
a\_H=1-0.05^{1/n\_{95}}
\]

This does **not** estimate habit extinction. Nor does it imply that a seasonal practice needs 66 years to become habitual: the original evidence concerns frequently repeated behaviors, and seasonal routines also arrive through instruction and social learning.

For comparison experiments, run conventional expected-utility agents with CRRA coefficients such as \(0,1,2\). Treat those as alternative sensitivity cases, not an empirically established range for all subsistence populations. Avoid applying CRRA curvature and prospect curvature sequentially to the same outcome without a clear interpretation.

### 2.3 Logit models and realistic noise

For a considered set \(C\_i\), a useful residual choice model is:

\[
P\_i(a)=
\frac{\exp[(V\_i(a)-V\_{\max})/\tau\_i]}
{\sum\_{b\in C\_i}\exp[(V\_i(b)-V\_{\max})/\tau\_i]}
\]

Subtracting \(V\_{\max}\) preserves probabilities while improving numerical stability.

**There is no empirically meaningful universal value of \(\tau\).** Utility coefficients and the noise scale are identified only relative to each other: multiplying both by the same constant leaves behavior unchanged. In the standard random-utility derivation, alternative-specific errors are independent Gumbel variables. [Econometrics Laboratory](https://eml.berkeley.edu/books/choice2nd/Ch03_p34-75.pdf)

For two alternatives with utility difference \(\Delta V\):

\[
P(\text{higher-valued option})=
\frac{1}{1+\exp(-\Delta V/\tau)}
\]

The following values are mathematical consequences, not empirical human error rates:

| Utility advantage \(\Delta V/\tau\) | Probability of choosing the higher-valued option |
| --- | --- |
| 0 | 50.0% |
| 0.5 | 62.2% |
| 1 | 73.1% |
| 2 | 88.1% |
| 3 | 95.3% |
| 4 | 98.2% |

**Calibration recommendation:** define an interpretable benchmark utility gap \(\Delta V\_\*\), then fit the probability of selecting the better option in a matched task:

\[
\tau=\frac{\Delta V\_\*}{\ln[p/(1-p)]}
\]

An 80%, 90%, or 95% selection rate corresponds respectively to \(\tau/\Delta V\_\*=0.721,\ 0.455,\ 0.340\). Those rates must come from data or an explicitly chosen design target.

For a provisional game calibration, choosing a clearly preferable, familiar option around **90–95% of the time** is a reasonable *test setting*, not a universal empirical claim. Near-indifferent choices should be much less consistent.

Three implementation precautions matter:

**Preserve stable preferences.** Give agents durable tastes and contextual states before adding residual randomness. Different people choosing differently is not the same as one person making inconsistent choices.

**Draw at decision events, not every tick.** Once someone chooses a meal, work task, or destination, maintain the commitment until completion or a meaningful interruption.

**Control alternative duplication.** Multinomial logit has restrictive substitution behavior; mixed logit permits richer taste variation and correlations. For TCE, grouping similar alternatives or choosing hierarchically can prevent adding twenty nearly identical purchases from drawing an implausibly large share away from unrelated activities. [Econometrics Laboratory](https://eml.berkeley.edu/books/choice2nd/Ch06_p134-150.pdf)

---

## 3. Variation across historical settings and world regions

### 3.1 Change environments and institutions, not the cognitive species

There is no defensible empirical table assigning a distinct loss-aversion coefficient to ancient farmers, medieval townspeople, and modern workers. The available evidence is much better at identifying **context dependence** than reconstructing ancient preference parameters.

An important archaeological constraint is that substantial food-storage infrastructure existed in the Jordan Valley about **11,000 years ago**, before fully domesticated crops. Long-horizon provisioning should therefore not be unlocked only by later technology or a supposed transition to “modern rationality.” [PNAS](https://www.pnas.org/doi/10.1073/pnas.0812764106)

The table below gives **recommended modeling hypotheses**, not measured historical coefficients.

| Setting | What should change in TCE | What should not be assumed |
| --- | --- | --- |
| **Foraging societies** | Configure local ecological knowledge, mobility, sharing arrangements, food storability, and the variance of returns. Decisions can rely heavily on practiced recognition and remembered outcomes. | Universal impulsiveness, absence of planning, or absence of storage. Present-day foragers are not unchanged representatives of prehistoric people. |
| **Early farming** | Make planting calendars, seed retention, delayed harvests, storage, and correlated weather risks central. Learned seasonal routines can encode long-term consequences without continuous calculation. | That slow adoption necessarily means irrational conservatism, or that every household evaluates independent crop risks. |
| **Pre-industrial agrarian and urban settings** | Vary access to markets, credit, apprenticeship, records, customary rights, and trusted intermediaries. Let occupational expertise and institutional obligations shape consideration sets. | A single European village model, universal isolation, or universally weak numerical competence. |
| **Industrial settings** | Where employers, schedules, regular wages, and specialized tasks develop, let these reorganize routines, deadlines, and payment timing. | That scheduled work eliminates procrastination or that factory employment automatically changes underlying \(\beta\) or \(\lambda\). |
| **Modern settings** | Add formal insurance, extensive records, large choice sets, administrative defaults, and decision aids when institutions support them. | That abundant information produces exhaustive optimization or removes habit and default effects. |

For TCE’s unscripted progression, implement these as properties of institutions, livelihoods, and technologies. An early polity can have sophisticated records and provisioning; a technologically advanced region can still have insecure income and little effective insurance.

### 3.2 What worldwide evidence actually supports

**Rural India.** Binswanger’s experiments found that at high real payoffs, respondents were generally moderately risk-averse. Wealth differences did not produce a strong, simple explanation of the observed risk preferences. This supports modeling costly downside risk, but not assigning all poor households an identical risk coefficient. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.2307/1240194)

**Chile and Tanzania.** Henrich and McElreath found risk-seeking monetary choices among Mapuche and Sangu participants. In one comparison, **78% of Mapuche participants versus 20% of US student participants** selected a 20%-chance high-payoff gamble over a sure amount with the same expected value. The authors explicitly discuss the difficulty of transferring lottery behavior to agricultural practice. [Amazon Web Services, Inc.](https://s3.amazonaws.com/fieldexperiments-papers2/papers/00066.pdf)

**Hadza communities in Tanzania.** Apicella and colleagues found no endowment effect in more isolated communities but an effect among communities with greater exposure to markets and wider society. This challenges a universal ownership multiplier. Because exposure was not randomly assigned, it should not be encoded as proof that market contact alone causes the effect. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.104.6.1793)

**Vietnam.** Tanaka, Camerer, and Nguyen linked experimentally elicited preferences to household surveys. Associations differed between household and village income: higher village income was associated with greater patience and lower loss aversion, while household income relationships were not identical. This favors contextual, multilevel calibration over a single poverty-to-risk-aversion equation. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.100.1.557)

**Ghana.** Randomized agricultural insurance and cash interventions identify the importance of uninsured risk. Insurance changed investment and production choices; payout experiences and social information also affected subsequent demand. Represent institutions as changing the available risk distribution and beliefs, not merely lowering a psychological “fear” variable. [National Bureau of Economic Research](https://www.nber.org/papers/w18463?utm_source=chatgpt.com)

**Across countries.** The Global Preferences Survey covers approximately **80,000 people in 76 countries** and reports substantial within-country heterogeneity—larger than between-country heterogeneity. Its standardized risk and patience measures are valuable comparative targets, but they are not directly interchangeable with structural \(\lambda\), \(\beta\), or CRRA coefficients. [National Bureau of Economic Research](https://www.nber.org/papers/w23943)

Finally, avoid a permanent cognitive penalty for poverty. Carvalho, Meier, and Wang found that before-versus-after-payday differences among low-income US households did not produce corresponding differences across several risk, cognitive, and real-effort measures; monetary impatience could reflect liquidity. This does not settle the consequences of chronic deprivation, but it rules out treating every temporary shortage as a general collapse of decision competence. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20140481)

---

## 4. Stylized facts and validation targets

A correct simulation should reproduce **conditional patterns**, not force every population to match every experiment.

| Pattern | Empirical or mathematical anchor | TCE validation test |
| --- | --- | --- |
| **Routine acquisition is gradual and heterogeneous** | The Lally study’s fitted acquisition times varied widely; one missed opportunity did not materially derail acquisition. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/ejsp.674) | Repeat a context-linked activity, introduce occasional interruptions, and test smooth acquisition rather than an abrupt “habit formed” switch. |
| **Defaults change participation without changing material payoffs** | Meta-analytic \(d=0.68\), with substantial heterogeneity. [Cambridge University Press](https://www.cambridge.org/core/journals/behavioural-public-policy/article/when-and-why-defaults-influence-decisions-a-metaanalysis-of-default-effects/67AF6972CFB52698A60B6BD94B70C2C0) | Hold payoffs constant, vary the default and its perceived authority, and measure switching. Do not target a universal 68% increase. |
| **Plans can change when effort becomes immediate** | In the first block of Augenblick et al., average early-task allocation fell from **26.59 to 24.12** when work became immediate. [Haas School of Business](https://faculty.haas.berkeley.edu/ned/Working_Over_Time.pdf) | Compare planned and realized work under unchanged incentives, separating present bias from unexpected interruptions. |
| **Some people value commitment** | **59%** selected a costless commitment option in that effort experiment. [Haas School of Business](https://faculty.haas.berkeley.edu/ned/Working_Over_Time.pdf) | Allow voluntary commitments; test whether agents prone to postponement sometimes choose them rather than universally preferring flexibility. |
| **Rare-event responses depend on information format** | Experience-based and description-based choices can differ in the direction of rare-event weighting. [Sage Journals](https://journals.sagepub.com/doi/10.1111/j.0956-7976.2004.00715.x) | Give equivalent underlying risks through repeated experience versus explicit descriptions. Compare beliefs and choices. |
| **Loss framing can change risk-taking** | Prospect theory predicts different responses across gains, losses, and probability levels. [Springer](https://link.springer.com/article/10.1007/BF00122574) | Reproduce matched lottery tasks before applying framing to property, harvests, or political losses. |
| **Agricultural conservatism is not a universal lottery preference** | The Indian and Chilean/Tanzanian studies give contrasting task-specific results. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.2307/1240194) | The same basic architecture should support conservative livelihood practices alongside willingness to gamble in another domain. |
| **Insurance can increase investment without changing personality** | Ghanaian randomized interventions support this mechanism. [National Bureau of Economic Research](https://www.nber.org/papers/w18463?utm_source=chatgpt.com) | Add credible insurance, keeping preferences fixed. Adoption should respond where downside exposure was binding. |
| **Choice sensitivity depends on utility differences** | Binary logit gives 73.1%, 88.1%, and 95.3% at gaps of \(1\tau,2\tau,3\tau\). | Unit-test the implementation and calibrate \(\tau\) only after fixing the utility scale. |
| **Population diversity remains large within cultures** | GPS evidence shows substantial within-country heterogeneity. [National Bureau of Economic Research](https://www.nber.org/papers/w23943) | Cultural means should not determine every individual’s behavior; simulated distributions should overlap substantially. |

In addition, run **engineering validity tests** that are not empirical claims: changing simulation speed should not change choices; duplicating an alternative should not radically redirect behavior; agents should not switch destinations continuously; and routines should eventually reopen when their prerequisites disappear.

---

## 5. Modeling recommendation for individual agents and institutions

### 5.1 Use a layered architecture

The following is a proposed TCE architecture:

```
on_decision_event(agent, domain, context):

    update_urgent_needs_and_household_constraints()
    observe_accessible_information()
    retrieve_relevant_routine_and_commitment()

    if current_commitment_is_valid_and_not_urgently_interrupted:
        continue_commitment()

    elif matching_habit_is_strong_and_feasible
         and no_reconsideration_trigger:
        execute_habit()

    else:
        candidates = generate_small_known_candidate_set()

        if uncertainty_is_high:
            add_options_from_trusted_relevant_peers()

        remove_physically_impossible_options()

        if no_option_avoids_serious_shortfall:
            consider_best_available_rescue_or_least_harmful_options()

        search_in_context_specific_order()

        if low_stakes_and_satisfactory_option_found:
            choose_first_satisfactory_option()
        else:
            evaluate_small_number_of_plausible_outcomes()
            apply_domain_appropriate_value_and_time_model()
            choose_using_calibrated_residual_choice_rule()

        create_or_update_commitment()

on_outcome_event:
    update_beliefs_and_prediction_error()
    update_aspirations_and_reference_points()
    reinforce_context_action_associations()
    record_salient_success_or_failure()
```

The important distinction is between **deliberation**, **execution**, and **learning**. A person can follow a complicated learned agricultural sequence without repeatedly solving the whole seasonal planning problem.

### 5.2 Represent a small amount of persistent decision state

In addition to TCE’s existing needs, personality, relationships, and possessions, retain:

| State | Purpose |
| --- | --- |
| Domain aspirations and reference points | Explain satisfaction, disappointment, and gain/loss framing. |
| Context-linked routines | Produce coherent daily and seasonal behavior. |
| Current commitments | Prevent continuous replanning and support long projects. |
| Outcome summaries and a few salient episodes | Support learning without retaining every mundane event. |
| Domain-specific uncertainty and expertise | Distinguish unfamiliarity from general incompetence. |
| Trusted information sources | Connect bounded decisions to social networks. |

For heterogeneity, begin with broad, overlapping person-level tendencies, then allow domain experience and institutions to modify behavior. Do not infer that the same person must have identical risk attitudes toward food, status, injury, and money.

### 5.3 Institutions should alter the decision problem

For TCE, institutions can act through several concrete channels:

| Institutional feature | Proposed mechanism |
| --- | --- |
| Shared storage or credible assistance | Changes the consequences of household shortfalls. |
| Insurance or enforceable contracts | Changes downside exposure and counterparty reliability. |
| Records, calendars, and accounts | Externalizes memory and makes delayed obligations more visible. |
| Apprenticeship and customary practice | Transfers useful routines without requiring each learner to rediscover them. |
| Defaults and standard procedures | Shapes what people consider and what appears endorsed. |
| Work parties, obligations, and earmarked resources | Provides coordination and possible commitment devices. |

These effects need not benefit everyone equally. A rule may help one household plan while constraining another; authority, bargaining power, and enforcement should determine who can change it.

Avoid making an institution a perfectly informed optimizer standing above the population. Its officials and members should have their own incomplete information, routines, interests, and disagreements.

### 5.4 Keep computation event-driven

The following are **engineering starting points**, not measured cognitive limits:

Use roughly **3–8 candidates** and **3–5 outcome scenarios** for ordinary consequential decisions. Permit **1–3 stages of explicit look-ahead**, with learned plans and terminal estimates representing longer consequences. Three planning stages can mean planting–harvest–storage; they need not mean three minutes.

Routine execution should be cheap. Reconsideration should occur when needs cross thresholds, a plan fails, a resource disappears, new information arrives, or a scheduled review becomes due.

For illustration, 50,000 agents making two substantial decisions per simulated day, with six candidates and four scenarios, require **2.4 million candidate-scenario evaluations per simulated day**. That is an arithmetic workload estimate, not a runtime guarantee.

Keep randomness reproducible with keys such as:

```
world_seed + agent_id + decision_counter + decision_domain
```

Update learning by actual observations and habits by relevant repetitions. Neither should accelerate because the renderer runs faster or the simulation uses smaller ticks.

### 5.5 Existing models worth borrowing from

| Model | Relevant contribution | What to borrow—and what not to assume |
| --- | --- | --- |
| **CONSUMAT — Jager, Janssen and collaborators** | Selects among repetition, imitation, deliberation, and social comparison using satisfaction and uncertainty. | Borrow the mode-switching architecture. Its thresholds and mode assignments are modeling assumptions requiring calibration, not established universal laws. [Marco Janssen](https://marcojanssen.info/wp-content/uploads/2016/08/2003_Diffusion_processes_in_demographic_transitions.pdf) |
| **Instance-Based Learning Theory — Gonzalez, Lerch & Lebiere** | Decisions develop through remembered situation–action–outcome instances, retrieval, and feedback. | Borrow compact experience-based evaluation and learning. Benchmark memory and retrieval costs before reproducing a full cognitive model for every citizen. [KiltHub](https://kilthub.cmu.edu/articles/Instance-Based_Learning_in_Dynamic_Decision_Making/6571193/1) |
| **Random-utility and mixed-logit models** | Provide interpretable stochastic choice and structured heterogeneity. | Borrow the choice layer and estimation methods, not the assumption that agents consider every available alternative. [Econometrics Laboratory](https://eml.berkeley.edu/books/choice2nd/Ch06_p134-150.pdf) |

A publicly documented **“Consumats on a network”** implementation by Marco Janssen is available through the CoMSES Computational Model Library. It is a useful reference implementation for the first architecture. [CoMSES Net](https://www.comses.net/codebases/?tags=consumat+model)

### 5.6 Calibrate mechanisms separately before combining them

Begin with limited information, real costs, and household constraints. Add routines and satisficing next. Only then add reference dependence, present bias, and residual noise.

Run ablations: remove one mechanism at a time and measure what changes. Otherwise, a model may fit slow technology adoption equally well through excessive loss aversion, missing insurance, inflated switching costs, weak information, or overly persistent habits.

Prefer the least complicated mechanism set that reproduces both individual choices and aggregate patterns. A long list of biases is not evidence of greater realism.

---

## 6. Sources, datasets, and remaining uncertainty

### Recommended calibration resources

| Resource | Best use | Caution |
| --- | --- | --- |
| **Global Preferences Survey**, Falk et al. | Broad distributions, within-country heterogeneity, cross-country comparisons, and validated survey instruments. Data and documentation are available through the project. [Global Preferences Survey](https://gps.econ.uni-bonn.de/) | Standardized indices are not structural prospect-theory or discounting parameters. |
| **Brown et al. (2024) replication package** | Inspect heterogeneity in reported loss-aversion estimates and study designs. [Benny](https://benny.aeaweb.org/articles?id=10.1257%2Fjel.20221698) | Do not turn a pooled mean into an individual distribution. |
| **Tanaka, Camerer & Nguyen (2010) replication materials** | Joint risk/time elicitation and links to household conditions in Vietnam. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.100.1.557) | Preserve the original functional forms and elicitation tasks. |
| **Augenblick and collaborators’ experimental data** | Reproduce planned versus immediate effort, monetary allocations, and commitment choices. Author-hosted materials include data and appendices. [Haas School of Business](https://faculty.haas.berkeley.edu/ned/) | Task-specific student evidence, not a historical population sample. |
| **Apicella et al. (2014) replication package** | Investigate heterogeneity in ownership effects across Hadza communities. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.104.6.1793) | Market exposure is not a randomized treatment. |
| **Carvalho, Meier & Wang replication data** | Separate liquidity-related monetary choices from broader claims about cognition and poverty. [ICPSR](https://www.icpsr.umich.edu/sites//view/studies/116156) | Anticipated payday fluctuations do not represent every form of deprivation. |

The foundational reading sequence is **Simon (1955), “A Behavioral Model of Rational Choice”; Gigerenzer and Goldstein (1996), “Reasoning the Fast and Frugal Way”; Tversky and Kahneman (1992), “Advances in Prospect Theory”; and Train’s *Discrete Choice Methods with Simulation***. Together they cover search, heuristics, risky valuation, and stochastic choice, but they should not be treated as competing complete theories of the entire person. [OUP Academic](https://academic.oup.com/qje/article-abstract/69/1/99/1919737)

**The largest uncertainties are historical transfer, cross-domain transfer, reference-point selection, and parameter identification.** Habit acquisition evidence does not supply an extinction rate for every routine. Loss aversion does not have a settled universal magnitude. Monetary impatience does not automatically reveal present bias in consumption. Observed persistence does not uniquely identify status quo bias. The relevant studies themselves demonstrate these limits. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1002/ejsp.674)

For TCE, the most useful consequence is practical: **make institutions, information, resources, habits, and expectations explain most behavior; use psychological coefficients to explain what remains.** That should produce citizens whose mistakes are understandable, whose routines make daily life coherent, and whose decisions can change with experience without requiring a scripted march toward perfect rationality.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92809-6180-83ea-aacb-57a0e51a7809)
