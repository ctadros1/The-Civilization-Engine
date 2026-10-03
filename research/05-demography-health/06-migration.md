# Migration: why and how people move

## A simulation-ready research report for The Civilization Engine

**The central recommendation is to model migration as a household decision constrained by resources, relationships, information, and institutions—not as a population flow toward whichever settlement has the highest “attractiveness.”** People may want to leave but lack the means; they may send one worker while retaining a farm; and successful pioneers can make later migration accessible to poorer households. These distinctions are central to both migration theory and empirical evidence. [SpringerLink](https://comparativemigrationstudies.springeropen.com/articles/10.1186/s40878-020-00210-4)

For TCE, distinguish five processes: **ordinary changes of residence; seasonal and circular movement; marriage and household formation; collective settlement founding; and forced displacement or coerced relocation.** They can share routing and accounting infrastructure, but should not share a single departure probability.

There is no defensible universal “pre-industrial migration rate.” Historical records often measure village turnover, birthplace, or residence at two distant dates—not annual departures. Even modern migration estimates change dramatically with the geographical boundaries used. The report therefore separates **observed benchmarks**, **derived accounting relationships**, and **proposed simulation defaults**. [CAMPOP](https://www.campop.geog.cam.ac.uk/blog/2024/08/22/stuck-in-the-mud/)

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Establish what counts as migration

Maintain separate fields for an agent’s **physical location, usual residence, household membership, legal settlement, and property claims**. A seasonal worker can be physically absent while remaining economically attached to the origin household. A refugee can lose access to a home without immediately relinquishing its ownership.

For each settlement, population accounting should satisfy:

\[
\Delta N = B-D+I-O
\]

People in transit must remain somewhere in the world’s population ledger. In a closed simulated world, migration redistributes existing people; it does not create them. An off-map population reservoir must therefore be explicit.

Record both **person-moves** and **distinct movers**. Someone leaving, returning, and departing again produces three movements but is one person. Also distinguish migration from an administrative reclassification: a village becoming a town does not mean its residents migrated.

### 1.2 Ordinary migration and chain migration

The following are proposed implementation rules grounded in the cited mechanisms.

| Mechanism | Rule for TCE | Evidence and qualification |
| --- | --- | --- |
| **Expected livelihood, not advertised wages** | Compare expected household consumption after employment risk, rents, taxes, food prices, and moving costs. For farmers, evaluate usable land and expected harvests rather than a wage. | Harris–Todaro explains why urban migration can coexist with unemployment: migrants respond to expected earnings, not guaranteed employment. [JSTOR](https://www.jstor.org/stable/1807860) |
| **Willingness differs from ability** | Track desire to leave separately from the ability to finance travel, survive unemployment, arrange care, and cross controlled territory. Severe deprivation can increase desire while reducing ability. | The aspirations–capabilities framework explicitly distinguishes these dimensions. [SpringerLink](https://comparativemigrationstudies.springeropen.com/articles/10.1186/s40878-020-00210-4) |
| **Life-course triggers** | Reconsider residence after marriage, leaving a parental household, entering service or apprenticeship, inheritance, eviction, widowhood, or loss of employment. | Comparative research across 27 countries connects migration age profiles to the timing of life-course transitions. A universal “move at age 25” rule is inappropriate. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1728-4457.2014.00671.x) |
| **Household risk diversification** | Permit one member to work elsewhere while others maintain land, care, and local claims. Evaluate whether origin and destination livelihoods fail together. | South Indian evidence links marriage migration and geographically dispersed family ties to consumption smoothing against agricultural risk. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/261633) |
| **Chain migration** | A successful migrant can provide information, lodging, loans, introductions, and transport assistance. These reduce specific costs and uncertainties for connected people. | Mexican evidence supports network-driven changes in who can afford to migrate. Do not model the network merely as an unexplained attraction multiplier. [King Center on Global Development](https://kingcenter.stanford.edu/publications/working-paper/network-effects-and-dynamics-migration-and-inequality-theory-and) |
| **Networks are helpful, not mandatory** | Permit migration through employers, recruiters, markets, public information, or exploratory travel without pre-existing relatives. | London apprenticeship records, 1600–1749, show that typical apprentices had no identifiable kin or common-origin connection to their masters, although access favored families with resources. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/networks-in-the-premodern-economy-the-market-for-london-apprenticeships-16001749/54A1634D065C5DFD5F47A56E27562CAA) |
| **Care arrangements constrain departure** | Check who will feed children, maintain the dwelling, and support dependent relatives after departure. Transfers and substitute carers can unlock migration. | South African longitudinal evidence finds that pensions facilitated working-age labor migration through both resources and childcare supplied by older household members. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fapp.1.1.22) |
| **Institutions regulate access** | Represent departure restrictions, admission, land allocation, work eligibility, relief eligibility, and expulsion as distinct rules with enforcement—not a single open/closed border. | Historical English legal settlement and poor-relief administration could lead to forced removal; modern asylum restrictions also affect later employment. [UCL Discovery](https://discovery.ucl.ac.uk/1522141/1/GISRUK_Abstract_new.pdf) |

**Implementation implication:** migration should normally select a *moving party*, not independently roll a die for every household member. Nevertheless, households should not be perfectly unified actors: disagreement, abandonment, refusal, and coercion can change who actually leaves.

### 1.3 Village fission and daughter settlements

**Do not split every village when it reaches 150 inhabitants.** Ethnographic and archaeological evidence supports relationships between group size, organizational difficulty, conflict, and splitting, but not a universal threshold. Alberti’s Hutterite compilation includes colonies splitting at sizes from **104 to 251 people**. Its comparison of colonies “at” and “after” fission is not an estimate of annual splitting risk. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3953443/)

Bandy’s research around Lake Titicaca treats fission as an alternative to developing institutions capable of coordinating larger communities. The costs of leaving, the availability of land, and integrative institutions can change whether a growing village divides. The archaeological population estimates are indices inferred from settlement evidence, not exact headcounts. [Academia](https://www.academia.edu/6447491/Fissioning_Scalar_Stress_and_Social_Evolution_in_Early_Village_Societies)

A useful TCE founding process is:

**Dissatisfaction or opportunity → coalition formation → site investigation → resource commitments → departure → establishment or failure.**

Implement that process through the following requirements:

* **A coherent coalition:** related households, followers of a leader, a faction, or people sharing a practical project—not a random half of the population.
* **A feasible destination:** water, usable land, access routes, defensibility, and permission or the ability to contest existing claims. Uncultivated land need not be unclaimed.
* **A viable establishment plan:** labor, tools, seed, shelter, food until the next reliable supply, and arrangements for children and other dependents.

These are proposed rules, not universal historical prerequisites. A hamlet near its parent can rely heavily on exchange and assistance; an isolated colony needs much more redundancy.

Keep parent–daughter relationships alive. Founding a new settlement should not automatically erase kinship, trading relationships, ritual obligations, or political allegiance.

A useful **mathematical sanity check**, rather than an empirical fission schedule, is:

\[
T\_{\text{doubling}}=\frac{\ln 2}{g}
\]

A daughter settlement starting at half the parent’s pre-split size would take approximately **139 years at 0.5% annual net growth**, or **69 years at 1%**, to regain that size without other changes. Frequent splitting therefore requires immigration, higher net growth, smaller departing groups, or causes other than gradual population accumulation.

### 1.4 War and famine: different departure processes

**War displacement should respond to locally perceived danger and escape possibilities.** In TCE, trigger emergency reconsideration when violence approaches, a settlement is attacked, food access is deliberately blocked, or an authority orders evacuation or expulsion. Destination choice should emphasize reachable safety, relatives, protection, and relief—not simply wages.

Allow movement in stages: nearby refuge, onward relocation, return, and renewed flight. Network-based refugee models such as Flee explicitly represent settlements, conflict locations, camps, and routes; their usefulness lies partly in representing these intermediate movements. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5645318/)

**Famine migration should respond to expected access to food, not only current hunger.** Households may send workers, seek relatives, sell assets, seek relief, or move together. These choices need not occur in a fixed sequence. Starvation can also make travel impossible, while war, restrictions, and dangerous routes can prevent access to otherwise available food. The famine–migration literature remains too fragmented to support a universal “harvest loss of X causes Y% to leave” function. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/disa.12420)

Do not make successful escape equivalent to successful survival. The 1994 influx of Rwandan refugees into North Kivu was followed by devastating disease mortality. In TCE, this should emerge from contaminated water, crowding, exposure, malnutrition, and inadequate treatment—not from a generic “refugee mortality modifier.” [PubMed](https://pubmed.ncbi.nlm.nih.gov/7646638/)

Forced transport, enslavement, deportation, and expulsion should be initiated by the coercing actor or institution. They are not ordinary migration choices with unusual utility weights.

### 1.5 Integration and effects on both ends

Migration changes more than headcounts. The following are recommended feedbacks rather than fixed outcome bonuses.

At the **origin**, remove the actual workers, consumers, carers, skills, and political participants who leave. Continue real transfers of food, goods, or money through surviving relationships. Loss of labor can relieve competition for land, but can also undermine a household’s cultivation or a community’s shared work. Mexican network research also cautions against assuming a single inequality effect: early and mature migration systems can affect access differently. [King Center on Global Development](https://kingcenter.stanford.edu/publications/working-paper/network-effects-and-dynamics-migration-and-inequality-theory-and)

At the **destination**, newcomers bring both labor supply and demand. Their consequences should depend on housing, land, food supply, skills, and institutions. A Rwanda study estimated positive local-income effects from refugees receiving cash assistance, but these were **model-based results under specific aid and market conditions**, not a universal migrant multiplier. [PubMed](https://pubmed.ncbi.nlm.nih.gov/27325782/)

Represent integration on separate dimensions: **livelihood matching, social relationships, language or practical knowledge, and formal rights**. These need not advance together. Swiss evidence associates an additional year of asylum waiting with a **4–5 percentage-point reduction in subsequent employment**. Historical research on Irish famine migrants finds substantial intergenerational convergence, but not immediate or complete economic assimilation. [Immigration Policy Lab](https://immigrationlab.org/publication/when-lives-are-put-on-hold-lengthy-asylum-processes-decrease-employment-among-refugees/)

---

## 2. Quantitative parameters and calibration benchmarks

**Confidence below concerns the stated setting.** “High” does not mean a result transfers unchanged to ancient farming villages. “Moderate” generally indicates reconstruction, selection problems, or model dependence.

### 2.1 Movement and settlement-pattern benchmarks

| Quantity and setting | Value, denominator, and interval | Appropriate simulation use | Confidence and source |
| --- | --- | --- | --- |
| **Village out-migration: Clayworth, England** | **38% of initial residents left over 1676–1688**; population remained approximately stable, **401 → 412** | Test substantial turnover without large net population change. Not exclusively rural–urban movement. | **Moderate**; historical household reconstruction summarized by CAMPOP. [CAMPOP](https://www.campop.geog.cam.ac.uk/blog/2024/08/22/stuck-in-the-mud/) |
| **Village out-migration: Cogenhoe, England** | **38% left over 1618–1628**; population **185 → 180** | A second stable-population/high-turnover case. Not a directly observed annual hazard. | **Moderate**; same research synthesis. [CAMPOP](https://www.campop.geog.cam.ac.uk/blog/2024/08/22/stuck-in-the-mud/) |
| **Rural–urban movement: England and Wales** | Of **18,740 matched males living in rural areas in 1851**, **4,387, approximately 23%, lived in a city in 1881** | A 30-year endpoint benchmark. Includes only successfully linked survivors; misses intervening returns and overseas departures. | **Moderate**; Long, 2005. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/ruralurban-migration-and-socioeconomic-mobility-in-victorian-britain/E6E208B771B71A3808A13BEE5D4E8392/share/26c4e26be82e8baa701441bbfea1140fba455980) |
| **Adult migrant stocks in industrial cities, 1851** | Born outside the city: **London 54%; Manchester–Salford 72%; Liverpool 77%**, among adults aged **20+** | Validate cumulative newcomer shares, not annual inflow probabilities or exclusively rural origins. | **High for census description**; Davenport’s compilation. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7611108/) |
| **Industrial-era long-distance departure rates** | Peak annual emigration: **Italy 10.8 per 1,000 population** in the 1910s; **Guangdong at least 9.6 per 1,000** in the 1920s | Demonstrates comparable orders of magnitude outside Europe. These are long-distance departure estimates, not rural–urban rates. | **Moderate**; McKeown’s reconstruction. [eScholarship](https://escholarship.org/content/qt4t49t5zq/qt4t49t5zq_noSplash_95f73f6a685393fac7d7641da9e81507.pdf) |
| **Modern internal migration and geographical scale: Canada, 2006** | Five-year migration **2.9% between 13 provinces/territories**, versus **15.0% between 4,916 subdivisions** | Validate the observation method. Changing settlement boundaries can change the measured rate several-fold. | **High within census definitions**; Bell and Charles-Edwards. [ResearchGate](https://www.researchgate.net/publication/305612958_Cross-national_comparisons_of_internal_migration_An_update_on_global_patterns_and_trends) |
| **Forager residential group size** | Across **32 societies, 5,067 individuals**: mean experienced band size **28.2 adults** | A residential-group benchmark, not a migration probability, total-population count, or minimum viable settlement size. | **High for compiled observations; low for direct prehistoric transfer**; Hill et al., 2011. [ResearchGate](https://www.researchgate.net/publication/279286429_Co-residence_patterns_in_hunter-gatherer_societies_show_unique_human_social_structure) |
| **Communal-colony size at fission** | Hutterite observations include **104–251 people at splitting** | Test heterogeneous splitting sizes; do not convert this range into a universal annual hazard. | **Moderate; low cross-context transfer**; Alberti, 2014. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3953443/) |
| **Early agricultural expansion front** | **0.6–1.3 km/year**, reported **95% confidence interval**, from **735 archaeological sites** | Regional frontier-expansion benchmark for the European/Near Eastern study setting. Not walking speed or household travel distance. | **Moderate**; Pinhasi, Fort, and Ammerman, 2005. [PLOS](https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.0030410) |

### 2.2 Crisis, household, and integration benchmarks

| Quantity and setting | Value and units | Appropriate simulation use | Confidence and source |
| --- | --- | --- | --- |
| **Seasonal migration response: Bangladesh** | A treatment offering **$8.50 upfront plus $3 at the destination** raised households sending a seasonal worker from **36% to 58%**, a **22-percentage-point increase** | Strong evidence that liquidity, risk, and experimentation can constrain migration. Do not transplant the dollar amount into another economy. | **High within experiment**; Bryan, Chowdhury, and Mobarak, 2014. [Innovations for Poverty Action](https://poverty-action.org/sites/default/files/publications/Under-investment%20in%20a%20Profitable%20Technology.pdf) |
| **Irish famine-era arrivals in the United States** | **More than 500,000 arrivals during 1846–1850** | A large, multi-year crisis migration benchmark; not the total Irish exodus over every famine-related interval. | **Moderate–high for recorded arrivals**; Collins and Zimran, 2019. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0014498319300403) |
| **Rwandan refugee influx, July 1994** | **500,000–800,000 people** entered North Kivu; **almost 50,000 died during the first month after the influx** | Test extreme bursts and destination-system overload. This is an exceptional crisis, not a typical mortality parameter. | **Moderate**, including substantial enumeration uncertainty; Goma Epidemiology Group, 1995. [PubMed](https://pubmed.ncbi.nlm.nih.gov/7646638/) |
| **Institutional delay and integration: Switzerland** | **One additional year waiting for asylum determination → 4–5 percentage points lower subsequent employment** | A context-specific test of institutional obstruction, separate from language or social familiarity. | **High within study design; low historical transfer**; Hainmueller, Hangartner, and Lawrence, 2016. [Immigration Policy Lab](https://immigrationlab.org/publication/when-lives-are-put-on-hold-lengthy-asylum-processes-decrease-employment-among-refugees/) |

### 2.3 How large should rural–urban flows be?

For a closed rural–urban region with fixed boundaries, let:

* \(u\) be the urban population share;
* \(g\_u\) be the urban population growth rate;
* \(n\_u=b\_u-d\_u\) be urban natural increase;
* \(M\_u\) be net migration into urban settlements.

Then:

\[
\frac{M\_u}{N\_u}=g\_u-n\_u
\]

and the corresponding net withdrawal from the rural population is:

\[
\frac{M\_u}{N\_r}
=
\frac{u}{1-u}(g\_u-n\_u)
\]

These identities provide better calibration checks than assigning a universal historical migration rate.

**Illustrative calculations—not historical estimates:**

| Assumed region of 10,000 people | Urban share | Urban natural increase/year | Urban growth/year | Required net rural–urban movement |
| --- | --- | --- | --- | --- |
| Small urban sector with a mortality deficit | 10% | −1.0% | +0.5% | **15 people/year:** 15 per 1,000 urban residents, or 1.67 per 1,000 rural residents |
| Rapidly expanding urban sector | 30% | +1.0% | +3.0% | **60 people/year:** 20 per 1,000 urban residents, or 8.57 per 1,000 rural residents |

Gross inflows and outflows can be much larger than these net transfers.

Also, **do not make every historical city a permanent demographic sink**. Davenport finds that large nineteenth-century English industrial cities could have births exceeding deaths despite retaining a mortality disadvantage relative to rural areas. Migration could therefore support rapid expansion rather than merely replace deaths. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7611108/)

---

## 3. Variation across eras and regions

These should become **institutional and technological configurations**, not date-triggered migration regimes.

| Setting | Historically important variation | Implication for TCE |
| --- | --- | --- |
| **Foraging societies** | Hill and colleagues document flexible residence: either sex may disperse or remain, and residential groups contain many people who are not close genetic relatives. Their sample includes societies from several continents. [ResearchGate](https://www.researchgate.net/publication/279286429_Co-residence_patterns_in_hunter-gatherer_societies_show_unique_human_social_structure) | Separate movement of an entire camp from changes in its membership. Permit residence with relatives of either partner, friends, and more distant connections. Do not hard-code exclusively patrilocal bands. |
| **Early farming and expanding settlement systems** | Archaeological evidence supports slow regional agricultural expansion, while Titicaca evidence connects settlement division and increasing social integration. The European expansion estimate does not establish a universal rate for farming’s spread everywhere. [PLOS](https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.0030410) | Make establishment costs, land access, crop suitability, and organizational capacity decisive. Frontier advance emerges from demographic growth, founding, abandonment, and adoption—not annual map painting. |
| **Pre-industrial agrarian and urban systems** | English village reconstructions reveal substantial movement, and the wider historical literature challenges a simple transition from immobile peasants to mobile industrial workers. Service, marriage, military activity, and temporary work require distinct accounting. [CAMPOP](https://www.campop.geog.cam.ac.uk/blog/2024/08/22/stuck-in-the-mud/) | Trigger movement through contracts, household formation, inheritance, and institutions. Most well-measured examples here are European; they are not universal coefficients for Africa or Asia. |
| **Industrial-era global migration** | McKeown reconstructs approximately **55–58 million** movements from Europe to the Americas, **48–52 million** from India/southern China toward Southeast Asia and the wider Indian Ocean/Australasian region, and **46–51 million** through northern Asian systems, roughly **1840–1940**. These are uncertain movement totals, not unique permanent settlers. [eScholarship](https://escholarship.org/content/qt4t49t5zq/qt4t49t5zq_noSplash_95f73f6a685393fac7d7641da9e81507.pdf) | Transport, recruitment, credit, frontier opportunities, and political regimes can generate very large Asian as well as European migration systems. “Industrial era” does not imply every migrant becomes an urban factory worker. |
| **Modern agrarian, industrial, and service economies** | Bangladesh illustrates seasonal experimentation; South Africa illustrates household financing and childcare; Mexico illustrates network development. These mechanisms differ from a model in which all family members permanently follow a wage gap. [Innovations for Poverty Action](https://poverty-action.org/sites/default/files/publications/Under-investment%20in%20a%20Profitable%20Technology.pdf) | Keep circular migration and geographically divided households even after transport improves. Better communications can alter information without abolishing family obligations or institutional barriers. |

Gender selection should arise from institutions and opportunities. For example, female migrants outnumbered males in many English towns because of demand for domestic servants, while South Indian marriage migration provides a different mechanism producing gendered flows. Neither supports a universal “young men migrate, women stay” rule. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7611108/)

---

## 4. Stylized facts a correct simulation should reproduce

| Pattern | Validation test |
| --- | --- |
| **Stable settlement size can conceal extensive turnover.** | Reproduce cases resembling Clayworth: nearly unchanged population alongside **38% departure of initial residents over 12 years**. Count deaths separately. [CAMPOP](https://www.campop.geog.cam.ac.uk/blog/2024/08/22/stuck-in-the-mud/) |
| **Migration concentrates around life transitions rather than occurring uniformly across age.** | Compare age profiles conditional on marriage, household formation, education, and employment transitions—not merely overall migration totals. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1728-4457.2014.00671.x) |
| **The poorest are not automatically the most mobile.** | A household with greater migration pressure can remain while a slightly better-resourced household leaves. Subsidizing the initial move should sometimes unlock migration, as in Bangladesh. [SpringerLink](https://comparativemigrationstudies.springeropen.com/articles/10.1186/s40878-020-00210-4) |
| **Corridors develop through experience and relationships.** | Successful first moves should alter later destination choices and access among connected households, rather than producing an immediate settlement-wide information update. [King Center on Global Development](https://kingcenter.stanford.edu/publications/working-paper/network-effects-and-dynamics-migration-and-inequality-theory-and) |
| **Growing cities can consist largely of newcomers.** | Under comparable conditions and definitions, reproduce adult outside-born shares on the order of **54–77%** seen in selected English cities in 1851. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC7611108/) |
| **Settlement expansion is much slower than travel.** | Agents may complete a journey quickly while the agricultural settlement frontier advances only around **a kilometre annually** over long periods in the relevant archaeological comparison. [PLOS](https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.0030410) |
| **Displacement is bursty and can relocate danger rather than eliminate it.** | Stress-test arrivals that overwhelm food, water, and shelter systems; mortality should emerge through those systems. The North Kivu crisis is an extreme benchmark. [PubMed](https://pubmed.ncbi.nlm.nih.gov/7646638/) |
| **Economic integration can span generations.** | Permit substantial progress without imposing either immediate equality or a permanent inherited migrant penalty; Irish famine-migrant research provides a historical comparison. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0014498319300403) |

For every comparison, generate a **synthetic observation using the historical study’s definitions**. A five-year residence comparison, an outside-born population share, and a complete event log should yield different statistics.

---

## 5. Modeling recommendation for individual agents and institutions

### 5.1 Recommended decision architecture

Use a **household planner with individual participants**, plus separate collective and emergency processes.

Each person needs residence history, relationships, skills, possessions, obligations, legal status, and health state. Each moving party needs a purpose, organizer, membership, intended destination, known alternatives, budget, route, and return conditions.

Ordinary migration can follow four stages:

**Consider → compare feasible plans → prepare → travel.**

A plan should include “stay” and alternatives such as “send one worker,” “move the household,” “join relatives,” or “participate in a founding coalition.”

A usable proposed destination score is:

\[
V\_{hj}
=
\beta\_c\Delta E[U(\text{consumption})]
+\beta\_s\Delta\text{safety}
+\beta\_k\Delta\text{support}
+\beta\_a\Delta\text{autonomy}
-\beta\_m\text{switching costs}
\]

Here \(h\) is the household and \(j\) a destination or plan. The quantities must be normalized consistently. Actual travel expenditure belongs in the household budget; avoid charging it again as an unexplained distance penalty.

Expected consumption should include own-production, employment probabilities, fallback work, transfers, and local prices. The coefficients are **calibration parameters**, not established human constants.

A bounded-choice rule can select among known alternatives:

\[
P(j\mid h)=
\frac{\exp(V\_{hj}/T\_h)}
{\sum\_{k\in C\_h}\exp(V\_{hk}/T\_h)}
\]

Use remembered or reported conditions, not omniscient knowledge. The temperature \(T\_h\) controls choice dispersion; its value depends on utility scaling and must be fitted.

For continuous-time consideration hazards:

\[
P(\text{consider during }\Delta t)=1-e^{-\lambda\Delta t}
\]

This prevents changing the simulation timestep from silently changing annual behavior. Do not independently apply overlapping departure rolls for hunger, unemployment, and grievance; combine their effects or treat them as competing triggers.

**Emergency movement should bypass ordinary review delays.** Under immediate danger, first screen for plausible escape and survival, then compare secondary benefits. Otherwise a sufficiently large wage advantage can absurdly compensate for imminent destruction in a linear score.

### 5.2 Founding should be a project, not a spawn event

A founding coalition should reserve or acquire resources, investigate sites, resolve claims, establish shelter and production, and potentially fail. Until the project succeeds, its members remain identifiable people with continuing obligations.

Allow partial outcomes: some households withdraw; a temporary camp becomes permanent; the daughter accepts the parent’s protection; or the founders return after a failed harvest.

Do not treat the new settlement as genetically or economically closed. A small daughter hamlet can remain part of a much larger marriage, exchange, and mutual-assistance network.

### 5.3 Physical movement and destination feedback

Use a persistent journey state. Consumption, fatigue, illness, weather exposure, theft, interception, and births or deaths continue during travel through the existing simulation systems.

At arrival, migrants must encounter actual housing, employment, land claims, relief, and social contacts. Housing shortage should generate crowding, encampments, price increases, or onward movement—not an arbitrary population-cap rejection.

Update origin and destination opportunities after migration. Without these feedbacks, the first slightly favorable city can absorb the entire world.

Return decisions should depend on surviving property rights, family ties, restored security, destination success, and the cost of another move. Do not classify “temporary” and “permanent” as immutable personality traits.

### 5.4 Proposed initialization and sensitivity settings

**Every number in this table is a design choice, not an empirical historical estimate.** Use them to initialize and stress-test TCE, then fit observed outcomes.

| Setting | Proposed starting point | Sensitivity range or alternative |
| --- | --- | --- |
| Ordinary household review | Every **6 months**, plus major life events | Every **3–12 months** |
| Emergency review | **Daily**, and immediately after direct threat events | Event-driven where possible |
| Candidate destinations per review | **12** | **8–16**, combining nearby, connected, and exploratory options |
| Peaceful inter-settlement migration benchmark | Initially examine runs near **2 completed moves per 100 residents/year** | Sweep **0.5–10 per 100/year**; exclude seasonal trips and forced displacement. This is an output benchmark, not a quota. |
| Initial daughter-settlement coalition | **10 households** | Test **5–20 households**, with viability determined by resources and support rather than a hard population minimum |
| Founding provisions | Budget to the next reliable food supply | Test **6–12 months** for isolated crop-dependent projects; shorten where trade, gathering, or parent support is credible |
| Acute-displacement stress tests | Compare scenarios with **10%, 50%, and 90%** of an exposed population attempting departure | Vary threat duration, escape routes, and support; attempted departure must not guarantee escape |

Do **not** initialize universal coefficients for “migration response to a 10% wage gap,” “annual famine flight,” or “years until assimilation.” The evidence does not identify transferable constants at that level.

### 5.5 What to simplify at 10k–50k agents

Use settlement-level caches for known wages, prices, vacancies, relief, land availability, and threats. Keep individual differences in beliefs and access, but avoid evaluating every person against every settlement every day.

Store a small number of consequential destination connections rather than a complete migration-relevant social graph. Evaluate ordinary plans infrequently; preserve daily physical movement and emergency responses.

For a first implementation, omit explicit smugglers, recruitment contracts, and detailed language learning. Preserve the essentials: **household coordination, capabilities, rights, connections, physical journeys, and destination feedback**.

### 5.6 Existing models worth adapting

| Model | What to borrow | What not to assume |
| --- | --- | --- |
| **Harris–Todaro, 1970** | Expected earnings and employment risk; migration can persist alongside urban unemployment. | Its two-sector equilibrium is not a complete model of families, routes, land rights, or forced displacement. [JSTOR](https://www.jstor.org/stable/1807860) |
| **Axtell et al., Long House Valley, 2002** | Household agents, reconstructed crop productivity, and spatial settlement decisions constrained by subsistence. | An environmental/production model alone is not a sufficient explanation of all settlement abandonment or political relocation. [ResearchGate](https://www.researchgate.net/publication/11360901_Population_growth_and_collapse_in_a_multiagent_model_of_the_Kayenta_Anasazi_in_Long_House_Valley) |
| **Simini et al., radiation model, 2012** | Intervening opportunities and aggregate origin–destination benchmarks. | A mobility model’s performance on commuting or aggregate flows does not establish permanent household-migration parameters. [Nature](https://www.nature.com/articles/nature10856) |
| **Suleimenova, Bell, and Groen, Flee, 2017** | Network routing through conflict locations, settlements, and camps; changing routes and facilities. | The original application conditions on externally supplied refugee totals to generate agents. It does not independently explain how many people decide to flee. TCE must derive departures from existing people. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5645318/) |

Flee also illustrates a calibration warning: some original movement probabilities and speeds were selected heuristically. They should not be transplanted into a pre-industrial world. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC5645318/)

---

## 6. Sources, datasets, and remaining uncertainty

### Priority calibration sources

| Source or dataset | What to extract |
| --- | --- |
| **IPUMS International** | Census microdata linking migration measures to age, sex, household composition, and work. The **MIGRATE5** documentation is particularly useful for five-year residence measures and country comparability. [IPUMS International](https://international.ipums.org/international-action/variables/MIGRATE5) |
| **IMAGE research program; Bell and colleagues** | Cross-national migration intensity and the effects of geographic scale. Use harmonized definitions before comparing populations. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/j.1728-4457.2015.00025.x) |
| **Long’s linked 1851–1881 census sample** | Rural–urban endpoint transitions, occupational selection, and explicit discussion of survivor and linkage bias. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/ruralurban-migration-and-socioeconomic-mobility-in-victorian-britain/E6E208B771B71A3808A13BEE5D4E8392/share/26c4e26be82e8baa701441bbfea1140fba455980) |
| **Hill et al. and Bandy** | Residential-group composition and archaeological settlement trajectories, respectively. These constrain group organization and settlement patterns more securely than annual departure hazards. [ResearchGate](https://www.researchgate.net/publication/279286429_Co-residence_patterns_in_hunter-gatherer_societies_show_unique_human_social_structure) |
| **IDMC displacement reporting** | Separate displacement events from people remaining displaced. For example, the **end-2024 stock was 83.4 million internally displaced people**, while **45.8 million disaster-displacement movements occurred during 2024**; repeated movements are not unique persons. These are fixed historical benchmarks, not a claim about the latest total. [PreventionWeb](https://www.preventionweb.net/news/number-internally-displaced-people-tops-80-million-first-time) |
| **Bryan–Chowdhury–Mobarak and Ardington–Case–Hosegood** | Empirical tests of whether finance, risk, experience, and care arrangements alter household migration decisions. [Innovations for Poverty Action](https://poverty-action.org/sites/default/files/publications/Under-investment%20in%20a%20Profitable%20Technology.pdf) |

The core theoretical and historical readings are **de Haas, “A Theory of Migration” (2021); Rosenzweig and Stark, “Consumption Smoothing, Migration, and Marriage” (1989); McKenzie and Rapoport, “Network Effects and the Dynamics of Migration and Inequality” (2007); and Lucassen and Lucassen, “The Mobility Transition Revisited” (2009)**. Together they help prevent reduction of migration to wages, individual optimization, or a European industrial-era novelty. [SpringerLink](https://comparativemigrationstudies.springeropen.com/articles/10.1186/s40878-020-00210-4)

### Where evidence is thin or contested

**Prehistoric annual flows are weakly identified.** Settlement distributions, reconstructed populations, and modern ethnographic analogies do not provide complete migration histories. Agricultural expansion can involve both movement and adoption. Treat archaeological frontier speeds and settlement sizes as outcome constraints, not direct agent probabilities. [PLOS](https://journals.plos.org/plosbiology/article?id=10.1371%2Fjournal.pbio.0030410)

**Historical samples are selective.** Linked censuses omit many deaths, emigrants, and unlinked individuals; court or apprenticeship records represent particular groups. A thirty-year residence comparison is not a count of every intervening move. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/ruralurban-migration-and-socioeconomic-mobility-in-victorian-britain/E6E208B771B71A3808A13BEE5D4E8392/share/26c4e26be82e8baa701441bbfea1140fba455980)

**Famine has no universal migration elasticity.** Politics, conflict, assistance, household resources, and escape opportunities can alter both direction and magnitude. A sufficiently severe crisis can simultaneously produce mass flight and immobilize those least able to survive a journey. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/disa.12420)

**Good local causal evidence is not a universal coefficient.** The Bangladesh incentive and Swiss asylum-delay estimates identify important mechanisms, but their numerical effects should remain scenario-specific. Fit parameter ensembles to several targets—flows, age profiles, household splitting, return, migrant stocks, and settlement persistence—rather than selecting one “correct” parameter set from a single aggregate rate. [Innovations for Poverty Action](https://poverty-action.org/sites/default/files/publications/Under-investment%20in%20a%20Profitable%20Technology.pdf)

**For TCE, the most important architectural choice is this: people should move because a feasible plan becomes preferable to their current arrangements, while collective projects and coercion can change those arrangements.** With real resource costs, relationships, rights, and journeys, migration can generate village turnover, urban growth, diaspora connections, daughter settlements, abandonment, and return without scripting any historical trajectory.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9282a-50a4-83ea-84be-e4dc33e34aa5)
