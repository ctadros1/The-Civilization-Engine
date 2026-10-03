# Settlement growth, hierarchies and urban scaling

## A simulation-ready report for The Civilization Engine

**The strongest design is to make settlement sizes emerge from household demography, migration, food provisioning, transport, specialization and political concentration—and use Zipf’s law and urban scaling as conditional validation targets, not growth rules.** The evidence supports pronounced settlement hierarchies and recurring economies of agglomeration, but not a universal rank–size slope, a universal productivity exponent, or an inevitable progression toward ever-larger cities. Measurement boundaries materially affect the apparent laws. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0183919)

For TCE, the crucial distinction is between three different phenomena:

| Phenomenon | Question it answers | Appropriate simulation treatment |
| --- | --- | --- |
| **Settlement-size distribution** | How many small and large settlements coexist? | Emergent outcome of growth, migration, founding, abandonment and integration. |
| **Urban scaling** | How do area, infrastructure or output differ between settlements of different sizes? | Emergent consequence of spatial organization, interaction and production; sometimes a useful approximation for omitted detail. |
| **Urban growth and urbanization** | How quickly do particular settlements grow, and what share of everyone lives in urban places? | Accounting outcomes of births, deaths, migration and changing settlement classifications. |

A second constraint is specific to your population budget: **a world of 50,000 actual people must contain its cities’ supporting countryside as well as their residents.** It can support a convincing regional settlement system. It cannot simultaneously contain several historically large cities and their full agrarian hinterlands without an explicitly represented external economy.

---

## 1. What the “laws” actually say

### 1.1 Zipf’s law: a useful benchmark, not a universal distribution

Use an explicit convention:

\[
N\_{(r)}=C r^{-q}
\]

Here \(N\_{(r)}\) is the population of the settlement ranked \(r\), largest first. Exact Zipf behavior means \(q=1\): the second settlement is half the first, the tenth is one-tenth, and so forth.

This is distinct from the Pareto-tail convention:

\[
\Pr(N\ge n)\propto n^{-\alpha},
\qquad \alpha=1/q.
\]

The probability-density exponent is \(1+\alpha\). Keeping these conventions separate prevents a common calibration error: importing an exponent from a paper that fitted the inverse relationship. Cottineau’s meta-analysis explicitly distinguishes and harmonizes these alternatives. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0183919)

Across **86 studies and 1,962 estimates**, Cottineau found a mean rank–size exponent of **1.025**, median **0.986**, and standard deviation **0.282**. That dispersion is variation among published estimates—not a confidence interval for a universal constant. Approximately **40% of variation** was attributable to technical choices such as settlement definition, sample selection and estimation. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0183919)

**For TCE:** expect a strongly unequal size distribution, but do not enforce \(q=1\). An approximately straight upper tail can coexist with a curved distribution of villages and hamlets.

There is a genuine dispute about the full distribution. Eeckhout’s analysis of all US places supported a **lognormal** distribution; Levy argued that the largest places possess a distinct Pareto tail; Eeckhout’s reply challenged that inference. The defensible conclusion is not “all settlements follow Zipf,” but “the appropriate distribution depends on the size range, boundaries and statistical test.” [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2F0002828043052303)

### 1.2 How a rank–size hierarchy can emerge

Gabaix’s explanation starts from approximately proportional stochastic growth: established cities experience percentage changes whose distributions are not strongly determined by initial size. Combined with suitable lower-bound, entry and normalization conditions, this can generate a Zipf-like stationary upper tail. **Multiplicative growth alone is insufficient**: over finite horizons it can instead generate a lognormal distribution. [Xavier Gabaix](https://xgabaix.scholars.harvard.edu/publications/zipfs-law-cities-explanation)

The implementation implication is important. Independent individual births and deaths are not enough to produce the same process. Their relative fluctuations tend to average out in large populations. TCE also needs **shared settlement-level influences**: a port gains traffic, a district loses access to irrigation, a craft cluster develops, a capital receives tribute, or an epidemic affects a connected population.

Apply those shocks to the actual economic and demographic mechanisms—not directly to population counts.

### 1.3 Urban scaling: compare like with like

A useful representation is:

\[
Y\_i=Y\_0(t,s)\left(\frac{N\_i}{N\_0}\right)^\beta e^{\varepsilon\_i}.
\]

\(Y\_i\) might be road length, housing units, annual output or settled area. \(s\) identifies a comparable settlement system; \(N\_0\) is a reference population; and \(Y\_0\) captures technology, institutions, prices and other system-level conditions.

Bettencourt and colleagues found three recurring patterns: some infrastructure measures are **sublinear**, basic population-serving quantities are approximately **linear**, and some socioeconomic outputs are **superlinear**. Their empirical estimates differ substantially by indicator and country. Food, water and energy consumption should not automatically receive the same economies of scale as network infrastructure. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1852329/)

For illustration, with \(\beta=1.15\), doubling population raises predicted total output by \(2^{1.15}\), but output **per person by only about 11%**, not 15%. That is arithmetic, not a recommendation to award every citizen an 11% production bonus.

Three qualifications matter:

**First, a cross-sectional relationship is not a growth equation.** A city becoming twice as populous over a century also changes its technologies, industries and institutions.

**Second, boundaries matter.** Arcaute and colleagues generated thousands of alternative city definitions for England and Wales. Many indicators were approximately linear, while nonlinear exponents varied with the definition. [Bristol Research Information](https://research-information.bris.ac.uk/en/publications/constructing-cities-deconstructing-scaling-laws/?utm_source=chatgpt.com)

**Third, aggregate advantages need not be equally distributed.** Research on within-city inequality shows that the composition of incomes can contribute substantially to measured scaling. TCE should track who receives gains rather than distributing a population-size bonus uniformly. [Nature](https://www.nature.com/articles/s41562-022-01509-1)

---

## 2. Mechanisms: rules TCE can implement

The following are **recommended implementations**, informed by the evidence rather than claimed as uniquely established historical equations.

### 2.1 Preserve the population accounting identity

For each settlement:

\[
N\_i(t+\Delta t)=N\_i(t)+B\_i-D\_i+M\_i^{in}-M\_i^{out}.
\]

Record annexation, merger, splitting and changes in the “urban” classification separately. They change reported aggregates or membership, not the number of living people.

In a closed world:

\[
\sum\_i(M\_i^{in}-M\_i^{out})=0.
\]

A town can grow despite negative natural increase. For example, an **illustrative** 1,000-person town with 35 births, 40 deaths, 55 arrivals and 20 departures gains 30 people: natural decrease of 0.5%, offset by net immigration of 3.5%.

Demographic research distinguishes natural increase, internal migration, international migration and reclassification precisely because their contributions vary substantially across historical settings. [Marron Institute](https://marroninstitute.nyu.edu/uploads/content/Demography%2C_Urbanization_and_Development_Remi_Jedwab.pdf)

### 2.2 Let provisioning constrain concentration

A settlement needs food that actually reaches it, not merely food theoretically grown somewhere in the world.

Track harvests, seed retention, animal feed, storage losses, household consumption, transport capacity, delivered prices and institutional appropriation. Urban farming should remain possible: “urban resident” must not mean “necessarily produces no food.”

A useful **accounting illustration** makes the constraint clear. Suppose rural residents collectively produce \(a\) annual person-rations per rural resident, net of production losses, while urban residents produce none. Then:

\[
a(1-u)P\ge P
\quad\Rightarrow\quad
u\le1-\frac1a.
\]

Thus:

| Illustrative net food productivity \(a\) | Maximum urban share under these assumptions |
| --- | --- |
| 1.10 person-rations per rural resident-year | 9.1% |
| 1.25 | 20.0% |
| 2.00 | 50.0% |

These are **derived examples, not historical productivity estimates**. Transport losses lower the attainable share; imports or urban agriculture raise it.

For TCE, this is preferable to an arbitrary “agricultural era permits 10% urbanization” setting. It also ensures that a capital can impoverish its hinterland through extraction without magically creating additional food.

### 2.3 Make migration a constrained household decision

Households should compare expected living conditions, not nominal wages alone:

\[
V\_{hi}
=
E[\text{consumption, housing, safety, access and social support}]
-
\text{moving costs}.
\]

Important constraints include available land or accommodation, expected employment, legal admission, kin obligations, household assets and knowledge of destinations.

Use a two-stage process: households occasionally reconsider their residence, then compare remaining in place with a limited set of known destinations. Distress, eviction, marriage, inheritance and military displacement can trigger additional reviews.

Migration need not stop when a city is unpleasant: the relevant comparison is with available alternatives. Rural impoverishment can produce urban growth without substantial prosperity. Demographic “urban push” can also expand cities independently of a new influx of rural workers. [Marron Institute](https://marroninstitute.nyu.edu/uploads/content/Demography%2C_Urbanization_and_Development_Remi_Jedwab.pdf)

### 2.4 Produce agglomeration through specific advantages

Implement the benefits of concentration through authored activities:

| Mechanism | Agent-level implementation |
| --- | --- |
| Shared fixed costs | Several households or workshops support a mill, kiln, dock, market or specialist supplier that none could support alone. |
| Better matching | Workers encounter more suitable employers; producers encounter suppliers and buyers. |
| Specialization | A specialist becomes viable when accessible demand covers setup and continuing costs. |
| Learning | Apprenticeship, observation, collaboration and worker movement transmit techniques. |
| Collective services | Institutions finance protection, water, roads, storage or adjudication. |

These provide mechanisms through which larger interacting populations can achieve increasing returns, rather than assuming the returns as an intrinsic property of headcount. Settlement-scaling theory similarly emphasizes interaction within spatial and movement constraints. [journals.plos.org](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0087902)

Do not let every added resident automatically improve every business. A larger population with poor access, weak purchasing power or exclusionary institutions may contribute little to a particular production network.

### 2.5 Let service catchments generate a hierarchy

A settlement’s market is not identical to its resident population.

Where detailed transactions are too expensive to simulate, an approximation is:

\[
D\_{ik}=\sum\_j N\_j d\_{jk}e^{-T\_{ij}/\tau\_k}.
\]

Here \(D\_{ik}\) is accessible demand for service \(k\), \(d\_{jk}\) is demand per resident, \(T\_{ij}\) is travel time, and \(\tau\_k\) controls how far customers tolerate travelling for that service.

Frequently needed services should have small catchments; infrequent specialist services can draw customers from farther away. This produces overlapping tiers of local centers, district towns and major centers without fixed settlement classes.

Use actual travel networks. A nearby settlement across a dangerous crossing may be less accessible than a distant one connected by reliable transport. Historical research on European and Middle Eastern–North African urban development identifies transport access and institutional differences as important correlates of growth. [MIT Press Direct](https://direct.mit.edu/rest/article/95/4/1418/58289/From-Baghdad-to-London-Unraveling-Urban)

### 2.6 Make concentration costly

The opposing forces should operate through experienced conditions:

Higher land demand produces subdivision, rising rents or political competition over plots. Long journeys consume work time. Crowding increases exposure opportunities. Water systems, roads and markets develop queues and maintenance requirements. Food and fuel must be obtained from more distant sources.

Allow investment and technological change to relax these constraints—but with construction time, financing, maintenance and distributional conflict.

**Do not impose a permanent “cities have negative natural increase” rule.** Gindelsky and Jedwab assembled almost 2,000 demographic observations for **142 large cities in 35 countries, 1700–1950**. They found that mortality declined faster than fertility during industrialization and urban natural increase rose, particularly in large cities. This qualifies a simple universal “industrial city as demographic sink” narrative. [Bureau of Economic Analysis](https://www.bea.gov/research/papers/2022/killer-cities-and-industrious-cities-new-data-and-evidence-250-years-urban)

### 2.7 Generate primacy through flows of resources and authority

A capital can become disproportionately large because it concentrates tax receipts, court expenditure, military payrolls, religious activity, administrative employment or privileged trading rights.

Implement those transfers and jobs directly. Also allow commercial centers to challenge political capitals, and allow political fragmentation to create several competing centers.

Do not encode “autocracy gives the capital +50% population.” Ades and Glaeser reported substantially larger primary cities under dictatorships, but later work challenges the institutional relationship. These are contested cross-country findings, not portable regime coefficients. [National Bureau of Economic Research](https://www.nber.org/papers/w4715)

Measure primacy separately from the whole distribution:

\[
P\_{12}=\frac{N\_1}{N\_2},
\qquad
S\_1=\frac{N\_1}{\sum\_{i\in\mathcal U}N\_i}.
\]

Under exact Zipf, \(P\_{12}=2\); that is a reference, not a universal boundary between ordinary and “primate” systems. Always record which settlements belong to \(\mathcal U\).

### 2.8 Permit founding, fission, failure and abandonment

A settlement system needs entry and exit, not just redistribution among permanent towns.

Founding should require a viable group, an accessible site, tools and provisions, and a way to survive until production begins. Overcrowding, kin conflict, inheritance or access to new land can motivate fission. Failure can follow provisioning breakdown, insecurity or loss of network connections.

Abandoned locations should retain buildings, fields, roads, memories and claims where appropriate. The Village Ecodynamics Project provides a useful precedent for connecting household decisions and environmental constraints to aggregation and abandonment rather than assuming settlements persist forever. [Crow Canyon](https://crowcanyon.org/projects/the-village-ecodynamics-project/)

---

## 3. Quantitative parameters and calibration anchors

**Confidence below concerns evidential support and transferability, not a numerical probability.** “High within sample” does not mean the same value should apply to an early agrarian TCE world.

### 3.1 Size-distribution and scaling parameters

All exponents are dimensionless. Intervals are reported 95% confidence intervals where supplied.

| Quantity | Estimate or range | Evidence and units of the dependent quantity | Confidence and TCE use |
| --- | --- | --- | --- |
| Rank–size exponent \(q\) | Mean **1.025**; median **0.986**; SD **0.282** | Meta-analysis of 1,962 estimates; population in persons | High confidence in substantial variation. A comparison benchmark, not a world constant. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0183919) |
| Settled-area exponent | Theoretical benchmarks **\(2/3\)** and **\(5/6\)** | Area against population; amorphous and networked settlement models | Medium as conditional theory; do not impose universally. [journals.plos.org](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0087902) |
| Medieval European area exponent | Regional and pooled estimates between **\(2/3\)** and **\(5/6\)**; intervals exclude 1 | **173 cities**, around 1300 CE; settled area | Medium: reconstructed populations and areas, four European regional samples. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0162678) |
| US housing units | **1.00 [0.99, 1.01]** | Number of housing units | High within sample; useful near-linear benchmark, conditional on household structure. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1852329/) |
| US total wages, 2002 | **1.12 [1.09, 1.13]** | Aggregate monetary wages | High within sample; low direct transfer to agrarian production. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1852329/) |
| Chinese GDP, 2002 | **1.15 [1.06, 1.23]** | Monetary output | High within sample; composition and measurement matter. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1852329/) |
| German electrical cable length | **0.87 [0.82, 0.92]** | Network length | Infrastructure-specific; not a food, energy or material-consumption exponent. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1852329/) |
| Ancient Basin of Mexico civic construction | **1.177 [1.028, 1.327]** | Annualized monument volume against population of the contributing political unit; **48 observations** | Medium for this proxy; not ancient GDP or purely voluntary exchange. [SFI Education](https://sfi-edu.s3.amazonaws.com/sfi-edu/production/uploads/sfi-com/dev/uploads/filer/39/ba/39ba0bf1-8533-4ae5-a885-7de49c0d5eee/14-11-041.pdf) |

Ortman and colleagues also found mean residential-mound area increasing with settlement population: exponent **0.190**, interval **0.083–0.298**. This suggests differences in household material circumstances, but house area remains a proxy. Multiplying it by population to create a “total output” measure does not turn it into independently observed GDP. [SFI Education](https://sfi-edu.s3.amazonaws.com/sfi-edu/production/uploads/sfi-com/dev/uploads/filer/39/ba/39ba0bf1-8533-4ae5-a885-7de49c0d5eee/14-11-041.pdf)

For a compact settlement with \(A\propto N^{5/6}\), doubling population predicts approximately **78% more area and 12% greater density**. This is a useful arithmetic consistency check—not a command to resize the settlement.

### 3.2 Growth-rate anchors

Use:

\[
g=\frac{\ln(N\_1/N\_0)}{\Delta t}
\]

for continuously compounded annual growth. This makes estimates over different intervals comparable.

| Population or system | Evidence | Annual growth equivalent | Interpretation |
| --- | --- | --- | --- |
| Historical industrial-European urban populations | Approximately **35-year doubling time** in Jedwab and colleagues’ comparison | **1.98%/year**, calculated | Aggregate urban population, not the expected growth of every town. |
| Developing-country urban populations in the same comparison | Approximately **18-year doubling time** | **3.85%/year**, calculated | Migration plus natural increase and classification effects; not a fertility parameter. |
| Dar es Salaam urban agglomeration, 1990–2018 | **1.474 million → 6.048 million** in UN estimates | **5.04%/year**, calculated | Rapid sustained growth in a particular modern agglomeration. |
| Philadelphia urban agglomeration, 1990–2018 | **4.725 million → 5.695 million** | **0.67%/year**, calculated | A much slower-growing large urban system. |

The doubling-time comparison comes from the research underlying *Demography, Urbanization and Development*; the individual-city calculations use the UN’s 2018 historical population estimates, not its then-future projections. [Marron Institute](https://marroninstitute.nyu.edu/uploads/content/Demography%2C_Urbanization_and_Development_Remi_Jedwab.pdf)

**There is no well-supported universal annual growth parameter for pre-industrial towns.** Use matched longitudinal settlement datasets where possible. Do not substitute the growth of surviving famous capitals for that of ordinary towns, failed settlements or the whole population.

For gameplay timescales, the arithmetic is revealing:

| Hypothetical sustained growth | Time to grow tenfold |
| --- | --- |
| 0.2%/year | About **1,151 years** |
| 1%/year | About **230 years** |
| 2%/year | About **115 years** |
| 4%/year | About **58 years** |

These are **constructed scenarios**, not era-specific estimates. A village becoming a major town within one lifetime requires sustained concentration or unusually rapid demographic growth; ordinary slow growth will not do it.

### 3.3 Implementation search ranges—not historical estimates

The literature does not identify universal household decision frequencies or search budgets. These should be exposed as calibration parameters.

| Parameter | Suggested initial search range | Units | Status |
| --- | --- | --- | --- |
| Routine residential reconsideration | **0.05–0.20** | Reviews per household-year | Design prior; reconsideration is not migration. |
| Known destinations evaluated per review | **3–8** | Settlements | Computational simplification; expand through information networks. |
| Routine relocation payback horizon | **1–5** | Years | Behavioral prior; emergency moves bypass it. |
| Initial sensitivity ensemble | **100–500** | Independent runs per scenario family | Engineering recommendation, not a statistical sufficiency guarantee. |

Calibrate these against **realized gross migration, household stability, founding rates and settlement survival**, rather than selecting whichever values produce the most attractive rank–size graph.

---

## 4. Variation across eras and regions

### 4.1 Foragers: residential groups are not miniature cities

Hill and colleagues’ study of **32 contemporary foraging societies** reported a mean experienced band size of **28.2 adults**. That is neither total group population including children nor a universal prehistoric camp size. It nevertheless supports modeling ordinary residential groups at a scale of tens, embedded in wider networks rather than as isolated, permanently bounded communities. [Arizona State University](https://asu.elsevierpure.com/en/publications/co-residence-patterns-in-hunter-gatherer-societies-show-unique-hu/?utm_source=chatgpt.com)

**TCE implication:** distinguish residential population, seasonal attendance and the wider social network. Do not calibrate temporary aggregation sites with a permanent-city rank–size model. “Urbanization percentage” is often an inappropriate measure for this phase.

### 4.2 Early farming: permanence does not guarantee urban scaling

Early farming can sustain more permanent aggregation without immediately producing the same institutions, movement patterns or specialization as later cities.

A recent study by Chelazzi and Lawrence, published in **August 2026**, examined house-size and settlement-area relationships in ancient Southwest Asia. Scaling was unclear in Neolithic samples and more evident in later urbanized periods. The authors’ evidence is proxy-based and uncertain; it does **not** establish that early farmers lacked all agglomeration advantages. [journals.plos.org](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0355479)

**TCE implication:** unlock opportunities through storage, dependable provisioning, collective organization, exchange and repeated interaction—not through an “early farming reached” switch that automatically assigns a superlinear exponent.

### 4.3 Pre-industrial urbanism had several spatial forms

**Central Mexico:** Ortman and colleagues analyzed more than **1,500 settlements across roughly two millennia**, finding recurrent relationships between population, area and density. This is substantial evidence that scaling patterns are not exclusively industrial phenomena. Population reconstruction remains an important uncertainty because archaeological estimates often combine settlement area with artifact or dwelling evidence. [journals.plos.org](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0087902)

**Medieval Europe:** the 173-city study found larger cities denser on average across its regional samples. This is a useful compact-city benchmark, not evidence that medieval institutions or technologies were identical across Europe. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0162678)

**Maya and Izapa:** five full-coverage regional surveys yielded a contrasting pattern: residential density generally decreased as settlements became larger. More conventional scaling appeared when researchers compared central interaction spaces or civic construction with the relevant administered populations. The interpretation emphasizes less-than-daily gathering at centers rather than universal daily mixing throughout the entire residential footprint. [University of Colorado](https://www.colorado.edu/socialreactors/sites/default/files/attached-files/smith_et_al_2020_maya_sst_laa_plus_si.pdf)

This is a particularly valuable TCE test. A dispersed farming population visiting a center periodically should not be forced into the same density law as a compact pedestrian town. **Low density is not equivalent to an absence of urban functions.**

### 4.4 Historical urbanization was regionally uneven and sometimes reversed

Urbanization requires a denominator and a definition. Xu, van Leeuwen and van Zanden’s reconstruction of China includes towns at a **2,000-person threshold** and illustrates both temporal reversal and strong internal variation:

| Region and date | Reconstructed urban share | Interpretation |
| --- | --- | --- |
| Inner China, 1205 | **12%** | Pre-industrial national-scale urbanization was appreciable. |
| Lower Yangzi, 1205 | **25%** | A highly urbanized region could differ greatly from the larger system. |
| Inner China, 1776 | **7%** | Urban share did not increase monotonically. |
| Lower Yangzi, 1776 | **19%** | Regional concentration remained substantial. |
| Inner China, 1893 | **7%** | The reconstruction remains well below modern shares. |

Confidence is **medium-to-low for exact historical levels**, higher for the importance of definitions and regional differences. A falling urban share does not necessarily mean fewer urban residents: rural population can grow faster. [KNAW](https://pure.knaw.nl/ws/files/8633887/Urbanization_in_China_ca._1100_C1900.pdf)

European and Middle Eastern–North African historical datasets provide another warning against a single trajectory. Bosker, Buringh and van Zanden track urban development across **800–1800**, examining changing transport and institutional contexts rather than treating urban growth as a common continental clock. Their large-city threshold also differs from the Chinese small-town-inclusive definition. [MIT Press Direct](https://direct.mit.edu/rest/article/95/4/1418/58289/From-Baghdad-to-London-Unraveling-Urban)

For early sub-Saharan Africa, pre-contact America and parts of ancient South Asia, globally comparable urban shares are particularly difficult to establish from uneven city inventories. **Missing population estimates must not become a simulated absence of settlements.** Global historical compilations and reconstructions are useful, but their coverage is not equivalent to a complete census of towns and villages. [Nature](https://www.nature.com/articles/sdata201634)

### 4.5 Industrialization: faster growth without identical demographic paths

Industrial-era urban growth combines changing agricultural productivity, employment concentration, transport and demographic transition. Falling mortality can accelerate growth well before fertility falls sufficiently to offset it; therefore urban growth is not simply the number of peasants moving to factory jobs. The relative contribution of these processes differs across the historical comparisons and later city-level demographic evidence. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0094119015000601)

**TCE implication:** allow technological and institutional combinations that produce fast natural increase, fast immigration, both, or neither. A sanitation improvement can increase urban population pressure before the settlement has enough housing and infrastructure to accommodate it.

### 4.6 Modern urbanization: strong regional differences and changing definitions

For a clearly dated comparison, the UN’s **2018 revision**, using national urban definitions, estimated:

| Region | Urban share in 2018 |
| --- | --- |
| Africa | **42.5%** |
| Asia | **49.9%** |
| Europe | **74.5%** |
| Latin America and the Caribbean | **80.7%** |
| Northern America | **82.2%** |
| Oceania | **68.2%** |

These are not mutually identical morphological definitions of a city. [World Population Prospects](https://population.un.org/wup/assets/WUP2018-Highlights.pdf)

The **2025 revision** introduced a globally harmonized Degree of Urbanisation presentation alongside national definitions. On this harmonized basis, approximately **45%** of the world lived in cities and **36%** in towns and semi-dense areas in 2025; the corresponding 1950 shares were **20%** and **40%**. Its city dataset covers centers with at least **50,000 inhabitants**. The 45% figure is therefore not evidence of a decline from older “urban” estimates: it is a different category. [Joint Research Centre](https://joint-research-centre.ec.europa.eu/jrc-news-and-updates/worlds-constructed-areas-expanding-twice-fast-population-1975-2025-11-18_en)

Do not equate modernity with uniform expansion. UN data include shrinking cities, while global built-up land per person has increased over recent decades. Cross-sectional densification and temporal expansion can therefore coexist. [World Population Prospects](https://population.un.org/wup/assets/WUP2018-Highlights.pdf)

---

## 5. Stylized facts and validation tests

A correct simulation should reproduce **conditional families of patterns**, not pass one universal exponent test.

| Pattern to test | Expected behavior | Important qualification |
| --- | --- | --- |
| **Unequal settlement sizes** | Numerous small settlements, progressively fewer large ones. | Compare the full distribution and upper tail separately; lognormal and Pareto alternatives both matter. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2F0002828043052303) |
| **Approximate rank–size regularity** | Some mature, integrated systems have slopes near one. | Fragmented, small or transitional systems need not. Published estimates vary substantially. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0183919) |
| **Compact-system densification** | Larger settlements can have area exponents around \(2/3\)–\(5/6\). | Do not fail low-density Maya-like scenarios for lacking this pattern. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0162678) |
| **Indicator-specific scaling** | Housing is approximately proportional to population; selected infrastructure and economic measures deviate. | Do not apply one exponent to all goods and services. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC1852329/) |
| **Heterogeneous growth** | Rapidly expanding, slowly growing and declining places coexist. | Neither initial size nor technological stage determines a single growth rate. [World Population Prospects](https://population.un.org/wup/assets/WUP2018-Highlights.pdf) |
| **Non-monotonic urbanization** | Urban share can fall while total population rises. | Track the denominator and threshold explicitly. [KNAW](https://pure.knaw.nl/ws/files/8633887/Urbanization_in_China_ca._1100_C1900.pdf) |
| **Different demographic engines** | Similar net growth can arise from very different natural increase and migration. | Validate components, not only total population. [Marron Institute](https://marroninstitute.nyu.edu/uploads/content/Demography%2C_Urbanization_and_Development_Remi_Jedwab.pdf) |
| **Aggregation and abandonment** | Settlement networks can concentrate, disperse or lose centers. | Preserve failed settlements in the historical record rather than analyzing survivors alone. [Crow Canyon](https://crowcanyon.org/projects/the-village-ecodynamics-project/) |

For statistical validation, I recommend fitting alternative distributions with likelihood-based methods and uncertainty estimates, testing sensitivity to the minimum settlement size and settlement boundary. A straight-looking log–log plot is insufficient. Small samples and arbitrary cutoffs are known sources of misleading estimates. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0183919)

For scaling, fit comparable systems and time slices. Do not pool unrelated technologies, different price levels and different area definitions into a single exponent. At TCE’s smaller scales, many outputs will be zero—there may be no specialist workshop in most villages. Use count-aware or two-stage models rather than dropping those villages from the regression.

The strongest causal checks are **counterfactual experiments**: remove a bridge, decentralize a tax flow, improve water supply, restrict migration, lower workshop setup costs, or disable knowledge transmission. The resulting changes should occur through wages, provisioning, mortality, demand and relocation—not through an explicit adjustment to the target rank–size curve.

---

## 6. Recommended representation for TCE

### 6.1 Separate settlement, catchment and polity

Maintain three distinct objects.

A **physical settlement** is a residential and built cluster. Its statistics include permanent residents, dwellings, occupied area and infrastructure.

A **functional catchment** is the network of people who regularly use a market, workplace, shrine, court or service. It can overlap other catchments and include dispersed rural households.

A **political territory** determines taxation, protection, jurisdiction and coercive claims. It may contain many settlements or divide a single built-up area.

This separation directly addresses the empirical boundary problems: administrative units, continuous built-up areas and interacting populations need not coincide. [Bristol Research Information](https://research-information.bris.ac.uk/en/publications/constructing-cities-deconstructing-scaling-laws/?utm_source=chatgpt.com)

For UI labels, combine population with observable functions. A 1,500-person administrative and trading center can play a more urban role than a larger agricultural agglomeration. For research telemetry, additionally publish settlement counts and population shares above several fixed thresholds—such as 1,000, 2,000, 5,000 and 10,000 people—without turning those thresholds into development gates.

### 6.2 Use individual agents where they change the outcome

| Level | Recommended responsibilities |
| --- | --- |
| Individual | Age, survival, fertility, skills, occupation, movement, encounters and personal ties. |
| Household | Shared consumption, assets, housing, land claims, care obligations and most relocation decisions. |
| Firm or productive group | Input purchases, production, hiring, specialization and fixed costs. |
| Building or neighborhood | Space, accessibility, water, waste, crowding and local infrastructure. |
| Settlement | Aggregate accounts, market conditions and cached statistics—not an independent population-growth controller. |
| Institution or polity | Taxation, transfers, permissions, protection, public investment and compulsory labor. |

Keep realized demographic and economic events authoritative. Settlement-level approximations should summarize or accelerate those processes, not independently create people, goods or income.

### 6.3 Simplify search and interaction, not conservation

For 10,000–50,000 people, avoid all-pairs social or settlement calculations.

Use bounded acquaintance networks, spatial neighborhoods and explicit destinations for interaction. Stagger household reviews. Cache route costs and catchments, updating them when roads, access rules or environmental conditions materially change. Evaluate slower institutional and investment decisions on slower schedules than daily movement.

Calculate output from real production or value added at consistent prices. Do not count every intermediate sale as new output or mistake local price inflation for an agglomeration benefit.

Likewise, distinguish roof area, floor area, occupied residential area and the gross settlement envelope. Multistory construction can increase floor space without the same increase in footprint.

### 6.4 The finite-world arithmetic

Consider an **illustrative** 50,000-person world in which 7,500 people—15%—live in ten central settlements.

If those ten centers happened to follow exact Zipf:

\[
\sum\_{r=1}^{10}N\_r
=
N\_1\sum\_{r=1}^{10}\frac1r,
\qquad H\_{10}\approx2.929.
\]

Therefore:

\[
N\_1\approx\frac{7,500}{2.929}\approx2,561,
\qquad N\_{10}\approx256.
\]

That is a plausible regional hierarchy, but most centers would not qualify as cities under many statistical definitions. This is not a defect; it follows from the population budget.

**Recommended scope:** simulate the whole population of a region containing villages, market centers and perhaps one or a few substantial towns. When larger external networks are necessary, represent their food, trade and migration flows explicitly as an exterior economy.

Do not silently convert one citizen into hundreds of people: doing so breaks the relationship between visible households, labor, consumption, disease exposure and settlement size. Nor should the performance ceiling become an unexplained demographic rule that suppresses births or removes residents at 50,000.

### 6.5 Existing models worth borrowing from

| Model | What it offers | What it does not provide |
| --- | --- | --- |
| **Village Ecodynamics Project** | Household-level settlement decisions, environmental reconstruction, agriculture, exchange, aggregation and abandonment. Particularly relevant to early agrarian TCE. | A universal model of urban institutions, industry or city-size distributions. [Crow Canyon](https://crowcanyon.org/projects/the-village-ecodynamics-project/) |
| **SimpopLocal** | Settlement agents whose growth and hierarchy develop through innovation, changing resource capacities and spatial diffusion. Useful for ensemble calibration of settlement systems. | Individual daily life; its calibrated population and innovation rules should not be mistaken for historical constants. [arXiv](https://arxiv.org/pdf/1905.07160) |
| **MayaSim** | Coupled agriculture, ecological change, settlement growth and trade networks; useful for studying concentration and network breakdown. | A complete individual-agent city simulation or a uniquely established explanation of Maya history. [JASSS](https://www.jasss.org/16/4/11.html) |

The most promising synthesis is **VEP-like households, Simpop-like settlement-network analysis, and MayaSim-like ecological and trade feedbacks**, with TCE’s institutions determining access, transfers and investment.

---

## 7. Sources, datasets and remaining uncertainty

### Priority calibration datasets

| Source | Best use | Main limitation |
| --- | --- | --- |
| **MetaZipf — Cottineau (2017)** | Compare rank–size estimates and assess sensitivity to definitions and methods. | A database of published estimates, not a uniform global settlement census. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0183919) |
| **Ortman and colleagues’ Basin of Mexico datasets** | Agrarian population–area scaling and archaeological construction proxies. | Population reconstruction and proxy interpretation; regional transfer is uncertain. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0087902) |
| **Cesaretti and colleagues’ medieval European dataset** | Paired population and settled area for 173 cities around 1300. | Reconstructed observations and a selected European urban sample. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0162678) |
| **Xu, van Leeuwen and van Zanden (2018)** | Chinese urbanization, small-town inclusion and regional comparisons over the second millennium. | Historical estimation and threshold sensitivity. [Hep Journals](https://journal.hep.com.cn/fec/EN/10.3868/s060-007-018-0018-9) |
| **Bosker, Buringh and van Zanden (2013)** | Long-run urban development across Europe, the Middle East and North Africa, 800–1800. | Large-city-focused coverage; not a complete village system. [MIT Press Direct](https://direct.mit.edu/rest/article/95/4/1418/58289/From-Baghdad-to-London-Unraveling-Urban) |
| **Reba, Reitsma and Seto (2016)** | Spatialized urban population observations from **3700 BCE to 2000 CE**. | Irregular snapshots and large-city selection; insufficient alone for the urbanization denominator. [Nature](https://www.nature.com/articles/sdata201634) |
| **HYDE 3.2 — Klein Goldewijk and colleagues (2017)** | Long-run population and land-use reconstruction for geographically grounded scenarios. | Much of the ancient spatial detail is reconstructed through allocation assumptions, not directly observed. [ESSD](https://essd.copernicus.org/articles/9/927/2017/) |
| **UN World Urbanization Prospects and GHSL** | Modern population trajectories, built-up patterns and explicit settlement classifications. | Definition changes, historical revisions and limited relevance of modern city thresholds to small agrarian centers. [Joint Research Centre](https://joint-research-centre.ec.europa.eu/jrc-news-and-updates/worlds-constructed-areas-expanding-twice-fast-population-1975-2025-11-18_en) |

### Claims to keep provisional

**Exact Zipf universality remains contested.** Approximately Zipf-like upper tails are common enough to be useful, but they do not establish one distribution for all settlements or all eras. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0183919)

**A universal urban exponent is not established.** Indicator choice, boundaries, social inequality, technology and regional organization affect results. The strongest claims should concern particular measurements in particular systems. [Bristol Research Information](https://research-information.bris.ac.uk/en/publications/constructing-cities-deconstructing-scaling-laws/?utm_source=chatgpt.com)

**Ancient productivity is usually inferred, not directly measured.** Larger houses or greater monument construction can reflect prosperity, extraction, household organization or institutional priorities. These are informative proxies, not interchangeable measures of welfare or GDP. [SFI Education](https://sfi-edu.s3.amazonaws.com/sfi-edu/production/uploads/sfi-com/dev/uploads/filer/39/ba/39ba0bf1-8533-4ae5-a885-7de49c0d5eee/14-11-041.pdf)

**Early global urbanization estimates are substantially less secure than modern ones.** There is no defensible table of precise, universally comparable urban shares for every region and technological stage. Use explicit thresholds and reconstruction uncertainty rather than false precision. [ESSD](https://essd.copernicus.org/articles/9/927/2017/)

### Bottom-line recommendation

Build TCE’s settlement system around **conserved people and goods, household relocation, viable specialization, costly transport, shared infrastructure and institutional transfers**. Let those mechanisms produce both concentration and dispersal.

The success criterion is not that every world draws a straight Zipf line. It is that a world’s hierarchy, density, growth and urban share are explainable from its people’s actual activities—and that comparable worlds reproduce the relevant empirical patterns without having those patterns imposed on them.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928e8-3b74-83e9-b15a-2e4ad09b0e8b)
