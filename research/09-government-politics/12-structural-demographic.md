# Cliodynamics, secular cycles, and elite overproduction

## Research report for The Civilization Engine

**Recommendation: implement structural-demographic feedbacks, not a historical clock.** TCE should be capable of generating centuries-long expansions, mounting elite competition, fiscal crises, and clustered political breakdowns—but also successful reforms, prolonged stability, and crises unrelated to population pressure.

Structural-demographic theory, or **SDT**, is a framework within the broader field of quantitative historical research called *cliodynamics*. Its central claim is that instability becomes more likely when pressures among ordinary people, competition among elites, and weaknesses of the state reinforce one another. Goldstone developed the framework; Turchin, Nefedov, and collaborators formalized and extended it. The evidence is more convincing for some constituent mechanisms than for universal periodicity or precise political forecasting. [eScholarship](https://escholarship.org/content/qt8r85g67d/qt8r85g67d.pdf)

Turchin and Nefedov’s *Secular Cycles* examines England, France, Russia, and ancient Rome through population, living standards, elite dynamics, state finances, and instability. Here, *secular* means long-duration, not nonreligious. Their typical sequence is expansion → stagflation → crisis → depression or reconstruction, grouped into integrative and disintegrative trends. These are retrospective descriptions of interacting processes, not stages that every society must traverse on schedule. [Peter Turchin](https://peterturchin.com/books/secular-cycles)

---

## 1. Mechanisms: rules TCE could implement

The following rules are **proposed agent-based implementations** of the theory, not empirically fitted universal equations.

### 1.1 Prosperity can create delayed resource pressure

The agrarian mechanism begins with successful settlement, improved security, and growing production. Population subsequently expands. Where productive land and employment grow more slowly, households face smaller holdings, higher rents, or weaker bargaining positions. Owners of scarce assets may benefit while ordinary households lose ground. This is a conditional argument: expanding trade, productivity, or access to land can interrupt it. [eScholarship](https://escholarship.org/content/qt8r85g67d/qt8r85g67d.pdf)

**Implementation rule:** calculate household welfare from actual production, property access, prices, obligations, and consumption. Do not make population density directly generate discontent.

For a predominantly agricultural household, track:

\[
\text{available food}
=
\text{harvest}+\text{purchases}+\text{transfers}
-\text{seed}-\text{rent}-\text{tax}-\text{sales}-\text{losses}.
\]

The important outcomes are food adequacy, reserves, housing access, disposable income, and their trajectories—not population alone.

Let household formation and fertility respond through existing agents’ circumstances: available housing, marriage opportunities, surviving children, food security, and culturally defined family strategies. Existing age cohorts already generate demographic delays; do not add a second artificial “generation lag.”

**Crucial distinction:** fixed territory is not fixed carrying capacity. Irrigation, new crops, soil degradation, transport, market access, and security should change what the same land can support.

### 1.2 Wealth accumulation and elite recruitment can outrun opportunities

SDT distinguishes established elites, people seeking elite positions, and counter-elites who challenge the governing order. Elite overproduction can result from upward mobility, reproduction within privileged families, changing expectations, or contraction of desirable opportunities. It does **not** mean simply “too many rich people” or “too many graduates.” [eScholarship](https://escholarship.org/content/qt6qp8x28p/qt6qp8x28p.pdf)

**Implementation rule:** represent elite standing through concrete capabilities and institutions:

| Dimension | Possible observable basis |
| --- | --- |
| Economic power | Ownership of substantial productive assets; control of credit or trade |
| Administrative power | Offices, appointments, judicial authority |
| Coercive power | Command of soldiers, retainers, or organized armed followers |
| Religious or ideological authority | Recognized ritual offices, teaching institutions, influential communication networks |
| Social access | Patronage, lineage membership, marriage connections, eligibility for privileged roles |

A person may possess several dimensions or none. A wealthy merchant is not automatically a political incumbent; an influential priest need not be wealthy.

For each career or power domain, maintain an aspirant stock:

\[
A\_{t+1}
=
A\_t+\text{new aspirants}+\text{displaced incumbents}
-\text{appointments}-\text{career exits}-\text{deaths}.
\]

These terms should count actual people. Repeated applications must not create repeated aspirants.

### 1.3 Model positional scarcity separately from material scarcity

A society can have enough food and still have too few positions that ambitious people consider acceptable.

Use two distinct diagnostics:

\[
Q\_j=
\frac{\text{qualified unique applicants for domain }j}
{\text{openings offered in the same recruitment window}}
\]

and

\[
M=
\frac{\text{annual resources required for desired elite lifestyles}}
{\text{annual resources those households can actually command}}.
\]

The first measures access competition; the second measures maintenance pressure. Neither is itself a rebellion probability.

For \(Q\_j\), compare like with like: candidates for the same offices, military commands, guild privileges, or credentials over the same interval. When there are no openings, record a closed queue and waiting times rather than silently dividing by one.

**A TCE example:** a town doubles its number of commercially successful households, but its council remains hereditary and its officer corps retains ten commands. Economic expansion has created resources and aspirations without equivalent political access. Conflict is possible, but alternatives—new enterprises, religious careers, migration, negotiated admission—may absorb the pressure.

Do not impose a fixed maximum elite percentage. Some influential activities create additional productive or administrative capacity. Others are genuinely positional: there can be only one holder of a particular governorship.

### 1.4 Inheritance can preserve expectations while dispersing resources

For TCE, transmit assets, connections, legal eligibility, and learned expectations separately.

A privileged household with several surviving children might produce several people expecting high status without producing several equivalent estates. Partition can divide property; primogeniture can preserve an estate while leaving other children seeking alternative careers. Neither should duplicate wealth.

Suggested agent logic:

> When expected status exceeds attainable status, increase the attractiveness of alternative careers, migration, patronage, reform, and political opposition according to the person’s opportunities and beliefs.

Do not automatically turn disappointed heirs into rebels. Some should accept lower status, marry into another household, acquire new skills, or leave.

Cross-cultural research supports substantial variation in intergenerational wealth persistence, particularly between societies where productive assets are readily inherited and those where they are less important. It does not establish a universal elite fertility advantage or a single inheritance-to-rebellion coefficient. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2792081/?utm_source=chatgpt.com)

### 1.5 Elite competition becomes dangerous through organization

Disappointment is not enough. The important transition is from dissatisfied individuals to organized factions with resources, connections, and credible expectations of success.

A micro–macro SDT research proposal explicitly emphasizes connecting slow structural pressures to faster mobilization and interpersonal dynamics. It is a useful architectural direction, rather than an already validated end-to-end simulator. [eScholarship](https://escholarship.org/content/qt1xt9k875/qt1xt9k875.pdf)

**Implementation rule:** people compare several actions, not merely loyalty versus rebellion:

| Action | Relevant considerations |
| --- | --- |
| Remain loyal | Existing benefits, legitimacy, fear, expectations of future advancement |
| Seek reform | Accessible procedures, allies, anticipated concessions |
| Join a faction | Patron resources, shared goals, personal relationships |
| Migrate | Destination opportunities, travel costs, family obligations |
| Resist or rebel | Expected gains, collective strength, repression risk, opportunity costs |

Let popular organizations also form independently. Ordinary people should not require an elite manipulator before they can organize.

Elite splits matter because they can change access to money, communications, armed force, and administrative cooperation. Track those transfers explicitly. A governor withholding revenues or a commander refusing orders can be more consequential than an equal-sized increase in generalized unhappiness.

### 1.6 State weakness is a financing and coordination problem, not national poverty

Maintain separate state and private accounts. A wealthy polity can have an impoverished central government.

Ottoman fiscal research illustrates why: substantial revenues could remain with intermediaries rather than reach the central treasury. Some retained resources funded legitimate provincial services; others supported private power or extraction. Treating all nonremittance as corruption would therefore also be wrong. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/ottoman-state-finances-in-european-perspective-15001914/1C9ACC45EA3B715ACA8C2ED06BC6A510)

**Implementation rule:** distinguish:

\[
\text{tax assessed}
\rightarrow
\text{tax collected}
\rightarrow
\text{resources remitted}
\rightarrow
\text{services actually delivered}.
\]

For each resource ledger:

\[
B\_{t+1}
=
B\_t+\text{receipts}+\text{authorized borrowing}
-\text{actual payments}.
\]

Unfunded obligations become arrears, defaults, cancelled projects, or emergency extraction—not unlimited negative money.

An early agrarian government may possess grain and labor claims but lack transport or money to pay soldiers. Therefore, keep **grain, currency, and labor-service obligations distinct**, with explicit conversion and logistical constraints.

Relevant consequences include unpaid troops, delayed justice, neglected canals, failed famine relief, and loss of administrative personnel. These should influence compliance and loyalty through actual experiences.

### 1.7 Crises amplify themselves—but need not reset society

Turchin’s historical feedback work investigates how population growth and instability can affect each other with delays. Instability can depress subsequent growth through disrupted livelihoods and insecurity, making a contemporaneous population–violence correlation a poor test of the underlying mechanism. [Peter Turchin](https://peterturchin.com/publications/dynamical-feedbacks-between-population-growth-and-sociopolitical-instability-in-agrarian-states)

**Implementation rule:** feed conflict back into the ordinary simulation:

* Interrupted cultivation and trade reduce output and revenues.
* Displacement changes labor supply, rents, and food demand in receiving settlements.
* Asset destruction reduces future productive capacity.
* Defections weaken enforcement and encourage further noncompliance.

These effects should come from actual events. Do not subtract an arbitrary percentage of population or elites when a crisis flag activates.

Distinguish **regime collapse, territorial fragmentation, administrative contraction, economic decline, and settlement abandonment**. One need not imply the others.

### 1.8 Preserve genuine routes away from crisis

Reform, migration, economic diversification, and new opportunities can change the relationship between aspirants and available rewards. Davis and Feeney’s study of Britain after 1832 emphasizes several such outlets, including economic change, emigration, imperial opportunities, and state intervention. That does not mean costs disappeared: opportunities created through empire could impose violence or deprivation elsewhere. [CDL Repositories](https://repositories.cdlib.org/uc/item/2rf8c7zk)

For TCE, make stabilization costly and politically contested. Opening offices may reduce incumbents’ privileges; relief needs funding; military expansion creates recurring obligations; migration transfers people and pressures between locations.

**There should be no rule that the only way to reduce elite competition is to kill elites.**

---

## 2. Parameters: what can—and cannot—be calibrated

Three kinds of numbers must remain separate:

**Historical observations** constrain particular benchmark scenarios. **Published model coefficients** belong to particular equations and fits. **Design priors** are explicit starting assumptions for sensitivity testing.

Confidence below concerns the stated use, not a formal statistical probability.

### 2.1 Historical anchors and published coefficients

| Quantity | Value or range | Units and context | Source and confidence |
| --- | --- | --- | --- |
| Claimed secular-cycle scale | Commonly described as **200–300 years**; Turchin–Nefedov’s English intervals are **1150–1485** and **1485–1730**, or **335** and **245 years** | Retrospectively delimited historical intervals, not a universal period | Alexander 2016. Moderate confidence in the reported dating; **low confidence as a universal clock**. [eScholarship](https://escholarship.org/content/qt230872k1/qt230872k1.pdf) |
| Shorter instability oscillation | Approximately **50 years**, with US peaks around **1870, 1920, 1970** | Pattern identified in one historical violence series | Turchin 2012. A hypothesis to test, not a mandatory generational timer. [Peter Turchin](https://peterturchin.com/publications/dynamics-of-political-instability-in-the-united-states-1780-2010) |
| Qing metropolitan examination success | **6.2%** in 1691; **6.4%** in 1737; **5.2%** in 1742; **3.5%** in 1850; **5.5%** in 1890 | Successful degrees per candidate in reported examination years; **not job appointments** | Orlandi et al. 2023, Table 2. Moderate within-case confidence; the trajectory is not monotonic. [PLOS](https://journals.plos.org/plosone/article/file?id=10.1371%2Fjournal.pone.0289748&type=printable) |
| Ottoman central cash revenue | Probably **below 4% of GDP**; **below 6%** including imputed cavalry services | Early-modern central extraction, not total household taxation | Karaman & Pamuk 2010. Moderate–low: reconstructed GDP and fiscal coverage. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914) |
| Ottoman central share of total tax burden | Approximately **46% in 1527/28**, **25% in 1661/62** | Resources reaching the center relative to the estimated overall burden | Karaman & Pamuk 2010. Moderate historical estimate; useful for separating collection from remittance. [ResearchGate](https://www.researchgate.net/publication/227405066_Ottoman_State_Finances_in_European_Perspective_1500-1914) |
| Intergenerational wealth persistence | Weighted \(\beta\): foragers **0.19**, horticulturalists **0.18**, pastoralists **0.43**, agriculturalists **0.36** | Dimensionless parent–offspring association across a sample of **21 societies** | Borgerhoff Mulder et al. 2009. Moderate within-sample evidence; **not the fraction of assets bequeathed**. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2792081/?utm_source=chatgpt.com) |
| Wealth inequality in the same study | Weighted Gini: **0.25, 0.27, 0.42, 0.48**, respectively | Composite of wealth types, not directly interchangeable with modern income Gini | Borgerhoff Mulder et al. 2009. Comparative target, not fixed subsistence-system constants. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2792081/?utm_source=chatgpt.com) |
| Population-growth coefficient in a fitted demographic–fiscal model | \(r=0.013\) | **Year⁻¹**; intrinsic growth coefficient fitted to English population trends, 1086–1750 | Alexander 2016. A **model-specific fitted coefficient**, not an observed annual net growth rate or a birth-rate default. [eScholarship](https://escholarship.org/content/qt230872k1/qt230872k1.pdf) |

**Data audit:** the Qing paper’s prose gives 6.4% for 1691, whereas its table gives 154 successes among 2,500 candidates, approximately 6.2%. Its land-series prose and table also disagree on dates; I would not calibrate those dates without reconciliation. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0289748)

These sources do **not** provide transferable estimates for a universal elite population share, annual rate of aspirant formation, or increase in rebellion probability per additional aspirant.

### 2.2 Suggested TCE sensitivity ranges—not historical estimates

These are deliberately labeled **uncalibrated design priors**.

| Parameter | Initial test range | Starting value | Intended use |
| --- | --- | --- | --- |
| Status-expectation memory half-life | **5–20 years** | **10 years** | How quickly past family status loses influence absent reinforcing experiences |
| Career-strategy reconsideration interval | **1–5 years** | **2 years** | Reassessing a long-term aspiration; rejection, insolvency, and displacement can trigger earlier review |
| Routine political reconsideration interval | **1–3 months** | **1 month** | Staggered evaluation of allegiance and collective action; urgent events bypass the interval |
| Fiscal reserve headroom | **0–12 months** of obligations | **6 months** | Initial reserves **above seasonal working-capital needs**, not an automatic replenishment target |
| Direct political-network contacts | **2–12 relationships per agent** | **4** | A computational starting point for active patronage/faction links, separate from family and work networks |
| Opportunity-growth stress test | **−2 to +2 percentage points/year** | Test both signs | Experimental difference between growth in viable elite careers and aspirants; outcomes should arise from institutions in normal play |

**Source and confidence for every row:** proposed engineering choices; low empirical confidence. Sweep them rather than treating them as historical constants.

Two safeguards matter more than a precise starting value. Political noise must not dominate material and institutional differences, and calendar scheduling must not make every person reconsider loyalty on the same day.

### 2.3 Do not turn the Political Stress Index into a probability

A common SDT diagnostic is:

\[
PSI=MMP\times EMP\times SFD,
\]

where the components represent mass mobilization potential, elite mobilization potential, and state fiscal distress. Their operational definitions and normalization differ between applications. A PSI value is not intrinsically a probability or a universal collapse threshold. [eScholarship](https://escholarship.org/content/qt6qp8x28p/qt6qp8x28p.pdf)

For TCE, display the components separately. Avoid multiplication that makes any single zero erase all other risks. Most importantly, **do not use PSI to create unrest already being generated by the agents**; that would double-count the mechanism.

---

## 3. Variation across eras and world regions

Institutions and resource relationships should determine which mechanisms operate. An era or regional label should not directly change an instability coefficient.

| Context | Relevant differences | Implication for TCE |
| --- | --- | --- |
| **Foragers** | The full state–elite–fiscal model is inapplicable without a state. Comparative evidence nevertheless shows wealth transmission and inequality, rather than universal equality. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2792081/?utm_source=chatgpt.com) | Model access to resources, relationships, leadership, and exit. Do not create a treasury or bureaucratic aspirant queue merely because time has passed. |
| **Early farming** | Settlement and population boom–busts can occur without evidence establishing elite overproduction. Neolithic European demographic studies therefore support a resource-demographic module, not the entire SDT package. [Nature](https://www.nature.com/articles/ncomms3486) | Storage, land access, soil, disease, and migration can generate instability before hereditary offices or states exist. |
| **Agrarian Europe and Rome** | These provide much of the detailed comparative material in *Secular Cycles*: landholding, rents, elite reproduction and recruitment, state resources, and internal conflict. [Peter Turchin](https://peterturchin.com/books/secular-cycles) | Useful benchmark configurations, but not a universal institutional template. Vary inheritance, military obligations, office creation, and access to land. |
| **Imperial China** | SDT applications emphasize demographic pressure, competition through examination institutions, and fiscal weakness. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0289748) | Keep education, credentials, appointment eligibility, and actual employment as separate stages. |
| **Ottoman territories: Europe, western Asia, and North Africa** | Provincial intermediaries and military-fiscal arrangements affected central resources; later institutional reorganization increased central capacity. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/ottoman-state-finances-in-european-perspective-15001914/1C9ACC45EA3B715ACA8C2ED06BC6A510) | Model who retains revenues and commands force. A decentralized arrangement may work for a long time before military or administrative needs change. |
| **Pastoral settings, including African and Asian cases** | Inheritable livestock can be important to wealth persistence; land is not the only scarce productive asset. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/648561) | Use herds, water access, clientage, and redistribution where appropriate. Evidence for wealth mechanisms is stronger than evidence for one pastoral secular-cycle period. |
| **Pre-Columbian Americas** | Archaeological house-size comparisons find regional differences in inequality trajectories between Eurasia and North America/Mesoamerica. Such measures do not directly identify elite aspirants or fiscal cycles. [Nature](https://www.nature.com/articles/nature24646) | Do not infer identical elite systems from farming alone. Require evidence of offices, property, tribute, and political authority. |
| **Industrial societies** | New occupations, migration, administrative expansion, and changing political institutions can absorb pressures that a land-constrained model treats as unavoidable. Britain after 1832 is an important proposed example. [CDL Repositories](https://repositories.cdlib.org/uc/item/2rf8c7zk) | Replace a land-only constraint with changing career, housing, ownership, and political-access constraints. |
| **Modern societies** | Applying agrarian labor-supply arguments directly is disputed. Technology, globalization, institutions, and the measurement of elites substantially affect results. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0287912) | Permit elite competition without population growth. Distinguish absolute living standards, relative position, and access to influential careers. |

**Coverage limitation:** detailed, comparable, multi-century SDT tests are uneven. The evidence assembled here is substantially thinner for many African, South Asian, Southeast Asian, Oceanian, and Indigenous American institutional histories than for the principal European and Chinese cases. Cross-cultural wealth evidence should not be presented as equivalent to a validated regional theory of political cycles.

---

## 4. Stylized facts and validation targets

A correct implementation should reproduce these patterns **under appropriate conditions**, not force every world to exhibit them.

| Pattern | Evidence or theoretical status | Appropriate simulation test |
| --- | --- | --- |
| **Boom–bust dynamics need not imply regular cycles** | Shennan et al. used **13,658 radiocarbon dates across 12 European regions**; **10 regions** showed boom–bust patterns, with estimated declines often **30–60%**. Their analysis did not establish cyclicity. These are demographic proxies, not censuses. [Nature](https://www.nature.com/articles/ncomms3486) | Some constrained farming regions should grow and contract, but do not impose a repeating waveform or interpret every decline as mortality. |
| **Population and instability can be out of phase** | Lagged feedback is central to Turchin’s analyses of England, Han and Tang China, and Rome. [Peter Turchin](https://peterturchin.com/publications/dynamical-feedbacks-between-population-growth-and-sociopolitical-instability-in-agrarian-states) | Inspect lagged growth rates and causal event sequences, not only simultaneous correlations between population size and violence. |
| **Elite opportunity pressure is not simply inequality** | SDT’s elite mechanism concerns competition over positions and resources; these are distinct from a single wealth-distribution statistic. [eScholarship](https://escholarship.org/content/qt6qp8x28p/qt6qp8x28p.pdf) | Produce cases with high inequality but little aspirant congestion, and cases with congested political careers despite growing average income. |
| **Warning signals are conditional** | Downey et al. identified rising warning indicators in **7 of 9** selected Neolithic regional series. Their relevant fluctuations were roughly **400–1,000 years**, and the sample emphasized clear boom–bust cases. [DOI](https://doi.org/10.1073/PNAS.1602504113) | Increasing volatility or slower recovery may occur before endogenous crises. Sudden external shocks must remain possible without such warnings. |
| **Political contention is not all civil war** | In the US data used by Turchin and Korotayev’s 2020 assessment, peaceful demonstrations after 2010 outnumbered violent riots by about **five to one**. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0237458) | Record petitions, demonstrations, riots, coups, mutinies, and wars separately. Never validate a civil-war mechanic against an undifferentiated protest count. |
| **Crisis avoidance is a legitimate outcome** | Studies of British stability and selected historical reforms identify cases where structural pressures were altered rather than resolved only through breakdown. These are case studies, not estimates of a universal reform success rate. [CDL Repositories](https://repositories.cdlib.org/uc/item/2rf8c7zk) | Some severe-pressure worlds should reform, accommodate, or disperse tensions. Permanently forcing collapse would contradict the intended mechanism. |

For TCE’s observable world, useful outputs include landlessness, household reserves, rent burdens, elite career waiting times, patronage concentration, official arrears, troop defections, migration, and the changing composition of political events.

A visually peaceful city with rising queues, debt, and factional obstruction should be possible. So should a violent episode in a generally prosperous polity without a preceding century of demographic compression.

---

## 5. Recommended TCE representation

### 5.1 Use individuals and institutions as the source of truth

| Entity | Minimum relevant state |
| --- | --- |
| **Person** | Age, household, eligibility, skills, desired careers, perceived opportunities, allegiance, relevant experiences |
| **Household** | Assets, productive access, consumption, obligations, dependants, expected social standing |
| **Office or privileged position** | Powers, holder, eligibility, selection procedure, compensation, funding, tenure and replacement rules |
| **Enterprise or estate** | Productive assets, workers, income, ownership, obligations, expansion possibilities |
| **Faction or association** | Members, leaders, goals, resources, trusted links, commitments |
| **Government** | Resource ledgers, obligations, arrears, revenue rights, administrative personnel, coercive command relationships |
| **Settlement or district** | Population structure, production, prices, accessible land, migration links, service coverage |

Do not create a permanent hereditary `is_elite` flag. Preserve hereditary rights where institutions create them, but derive effective power from assets, offices, relationships, and command.

Likewise, do not define elites as “the richest 1%” and then attempt to measure whether their numbers are increasing: their share would be fixed by definition.

### 5.2 Keep slow accumulation and fast mobilization on different schedules

Suggested scheduling:

**Daily and event-driven:** consumption, work, travel, payments, arrests, refusals, violence, and urgent reactions.

**Seasonal:** harvests, rents, agricultural hiring, tax collection, major migration opportunities.

**Monthly, staggered:** routine political assessments, patronage recruitment, vacancy applications, faction coordination.

**At actual life events:** inheritance, household division, eligibility changes, appointments, succession, retirement, and death.

Annual or decadal aggregates are for measurement, not substitutes for these transactions.

For Rust, use sparse relationship graphs and incremental household/institution aggregates. Avoid evaluating every person against every other person. Cache relevant local and factional observations, then update them when material circumstances change.

The renderer can expose consequences already present in the kernel: abandoned works following fiscal failure, rival patron-funded buildings, crowded aspirant households, or neglected infrastructure. None requires a scripted “decline-era” architectural style.

### 5.3 Respect the population scale

At **10,000–50,000 actual people**, some elite categories will contain only a few individuals. A succession, marriage, or commander’s defection may therefore dominate local politics. Do not substitute the smooth behavior of an imperial-scale differential equation for these discrete events.

A particularly important engineering safeguard is to keep the **hardware population limit separate from ecological carrying capacity**. If births are suppressed whenever the simulation approaches its agent cap, the engine may manufacture a false Malthusian ceiling.

Either keep the simulated world’s demographic scope explicit or use a carefully accounted coarser representation outside the detailed region. Migration must move people somewhere rather than delete inconvenient population pressure.

### 5.4 Existing models and games worth studying

| Model or game | Useful contribution | Limitation |
| --- | --- | --- |
| **Turchin demographic–fiscal and elite models; Alexander’s implementations** | Small mathematical systems make feedback assumptions and sensitivity easy to inspect. | Some simplified models do not contain explicit elites at all; a good aggregate fit does not validate all proposed mechanisms. [eScholarship](https://escholarship.org/content/qt230872k1/qt230872k1.pdf) |
| **HANDY — Motesharrei, Rivas & Kalnay, 2014** | Four stocks—commoners, elites, nature, wealth—demonstrate resource and distribution feedbacks, including sustainable and collapsing outcomes. | A theoretical model, not a calibrated historical forecast; it does not supply detailed aspiration, office, or faction dynamics. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0921800914000615) |
| **Epstein civil-violence model / NetLogo Rebellion** | Local grievance, perceived arrest risk, policing, and collective-action thresholds offer a useful fast-mobilization prototype. | Hardship and legitimacy are largely external inputs in the demonstration. The official implementation also documents a rounding modification that materially changes its dynamics. [CCL](https://ccl.northwestern.edu/netlogo/models/Rebellion) |
| **Victoria 3’s published political design** | A design analogue for connecting population interests, wealth, political strength, organized groups, and revolutionary movements. | Aggregate population groups, not individual life courses; not an empirical secular-cycle model. The cited material documents its development design. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-57-the-journey-so-far) |

The strongest approach for TCE is a **hybrid**: detailed people and institutions generate slow structural changes; sparse networks generate faster collective action; aggregate models serve as diagnostic comparisons.

### 5.5 Calibration and falsification

Calibrate in layers. First validate household production, inheritance, demographic behavior, and office turnover. Then validate distributions and queues. Only afterward examine century-scale instability.

Use matched experimental scenarios rather than one preferred historical storyline:

| Experiment | What it tests |
| --- | --- |
| Productive opportunities expand as quickly as population | Whether population growth alone incorrectly causes crisis |
| Elite career opportunities expand with aspirants | Whether supposed overproduction is actually opportunity mismatch |
| Tax collection is unchanged but remittance falls | Whether fiscal weakness can emerge without aggregate impoverishment |
| Political access expands while material inequality persists | Whether accommodation operates independently of redistribution |
| External shock hits otherwise identical polities with different reserves and cohesion | Whether vulnerability modifies the consequences of the same trigger |
| Resource and political constraints are removed | Whether an accidental hidden clock still generates periodic breakdown |

A practical initial research batch would be **100 random seeds over 300–500 simulated years**, followed by targeted sweeps; these are proposed test sizes, not performance claims.

Evaluate event sequences, distributions, crisis incidence, duration, and recovery—not merely whether a spectral peak appears near 250 years. Hold out both time periods and institutional configurations. Historical observations also need an observation model: incomplete chronicles, changing reporting, and radiocarbon sampling do not expose the simulation’s complete event log.

---

## 6. Empirical support, critiques, and research sources

### 6.1 What the evidence supports

**A useful conditional framework.** Goldstone’s synthesis emphasizes interactions among state finances, elites, popular conditions, and institutional responses. This is more informative for simulation than a one-variable “poverty causes revolt” rule. It does not supply a universal crisis equation. [eScholarship](https://escholarship.org/content/qt8r85g67d/qt8r85g67d.pdf)

**Some supporting historical time-series results.** Turchin’s population–instability research finds evidence compatible with delayed feedback in selected agrarian cases. However, a small set of retrospective series is not equivalent to broad, independent, out-of-sample validation. [Peter Turchin](https://peterturchin.com/publications/dynamical-feedbacks-between-population-growth-and-sociopolitical-instability-in-agrarian-states)

**A real but limited forecasting achievement.** Turchin publicly anticipated increasing instability during the 2010s. Turchin and Korotayev’s later assessment found rising contention in US and UK series extending through 2018. This supports the direction of a broad forecast; it is not proof of exact event timing, a unique causal explanation, or a predicted civil war. The assessment was conducted by proponents. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0237458)

### 6.2 Important criticisms

**Historical fit can conceal mechanism failure.** Alexander’s English analysis found that a six-parameter demographic–fiscal model fit population approximately as well as a ten-parameter polynomial. Yet large-scale instability did not explain delayed population recovery as expected. He also changed proposed cycle boundaries, illustrating how historical periodization can influence apparent success. [eScholarship](https://escholarship.org/content/qt230872k1/qt230872k1.pdf)

**Elite measurement can become circular.** Georgescu’s 2023 industrial-society test challenges a US implementation in which elite numbers and incomes were generated from relative-wage relationships rather than independently observed. Alternative proxies substantially changed the stress-index trajectory. Those alternatives are imperfect too: income thresholds and student counts are not direct measurements of political elites. This is a serious challenge to particular operationalizations, not a demonstration that all elite competition is irrelevant. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0287912)

**Several measures must not be conflated.** Median wage divided by GDP per capita is a relative-wage indicator, not the same quantity as the labor share of national income. Neither alone establishes whether ordinary people’s absolute consumption is falling. Modern fiscal distress likewise cannot be read mechanically from debt/GDP without financing conditions and institutional capacity. [eScholarship](https://escholarship.org/content/qt6qp8x28p/qt6qp8x28p.pdf)

**Cycles may be partly imposed by model structure.** For TCE, this is a falsification problem: a model whose assumptions guarantee recurrent overshoot has demonstrated the implications of those assumptions, not that history universally obeys them. Compare it with alternative mechanisms, noncyclical baselines, and configurations where stable outcomes should exist.

**My assessment:** confidence is **moderate** that SDT identifies a useful family of interacting historical pressures, **lower** that one specification transfers across all societies, and **low** that a fixed cycle length or universal elite-overproduction threshold is justified.

### 6.3 Core reading and datasets

| Resource | Best use | Main caution |
| --- | --- | --- |
| **Turchin & Nefedov, *Secular Cycles* (2009)** | Principal comparative exposition and historical benchmark narratives | The principal cases are not a representative sample of all world societies. [Peter Turchin](https://peterturchin.com/books/secular-cycles) |
| **Goldstone, “Demographic Structural Theory: 25 Years On” (2017)** | Framework, causal interpretation, institutional qualifications | Synthesis rather than a globally calibrated parameter set. [eScholarship](https://escholarship.org/content/qt8r85g67d/qt8r85g67d.pdf) |
| **Turchin, Gavrilets & Goldstone, “Linking ‘Micro’ to ‘Macro’ Models of State Breakdown…” (2017)** | Architectural rationale for integrating structural conditions and agent mobilization | A research proposal, not a complete validated implementation. [eScholarship](https://escholarship.org/content/qt1xt9k875/qt1xt9k875.pdf) |
| **Seshat / CrisisDB** | Institution coding, power transitions, conflict and historical context | Inspect sources, coding uncertainty, temporal resolution, and coverage. [Seshat DB](https://www.seshat-db.com/variable-hierarchy/) |
| **Qing study data and R scripts** | Reproduction package, Zenodo DOI **10.5281/zenodo.7267757** | Audit underlying series before calibration. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0289748) |
| **Turchin’s US political-violence dataset** | Original study contains **1,590 events, 1780–2010**; useful for event composition and clustering | The expanded Seshat database is a different, evolving resource; recent coverage is explicitly work in progress. [Peter Turchin](https://peterturchin.com/publications/dynamics-of-political-instability-in-the-united-states-1780-2010) |
| **EUROEVOL and associated archaeological-study supplements** | Regional demographic proxies and alternative population-collapse mechanisms | Dates and settlement activity are not direct annual population counts. [Nature](https://www.nature.com/articles/ncomms3486) |
| **Clio-Infra** | Historical population, wages, output, and inequality comparisons | Consult the original series and reconstructions; coverage varies. [Clio Infra](https://clio-infra.eu/) |
| **World Inequality Database** | Modern income and wealth distributions | Distributional data do not directly count political elites or aspirants. [WID - World Inequality Database](https://wid.world/methodology/) |

### Bottom line for TCE

Build a society in which **families accumulate assets and expectations, people compete for specific opportunities, institutions collect and spend real resources, and factions mobilize through actual relationships**.

Then allow those processes to relieve or intensify one another. Long cycles should be an emergent possibility—alongside adaptation, irregular shocks, and durable stability—not a schedule the world is compelled to follow.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928d4-82c8-83ea-ba37-7f01a03ace43)
