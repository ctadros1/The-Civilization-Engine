# Settlement site selection and founding

## Core recommendation

**Model settlement founding as a household migration and coordination problem, constrained by seasonal survival—not as choosing the tile with the highest permanent “city score.”**

Three things should remain separate:

**Site quality** is local: dependable water, buildable ground, accessible fields, shelter, and defensibility. **Situation quality** is relational: access to other settlements, transport routes, markets, and political protection. **Persistence** comes from accumulated improvements, institutions, relationships, and the difficulty of coordinating a move.

These distinctions matter historically. European city-origins research finds strong effects of physical geography and transport access, while studies of American portage towns show that places can remain important after their original transport advantage disappears. Neither implies that the initially most attractive agricultural site must become the largest city. [Utrecht University Research Portal](https://research-portal.uu.nl/en/publications/city-seeds-geography-and-the-origins-of-the-european-city-system/)

For TCE, founding should establish **costs, commitments, and opportunities**. Subsequent investment, environmental change, conflict, and network development should determine whether the settlement grows, remains a village, relocates, or disappears.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Choose a livelihood catchment before choosing the residential core

The economically relevant location is not just the land beneath the houses. It includes fields, grazing, woodland, fishing grounds, water sources, and routes connecting them.

The classic *site-catchment* approach explicitly examines resources accessible around a settlement. More operationally, the Long House Valley model distinguishes agricultural plots from residences and allows households to occupy less productive residential ground near their fields and water. [Cambridge University Press](https://www.cambridge.org/core/journals/proceedings-of-the-prehistoric-society/article/prehistoric-economy-in-the-mount-carmel-area-of-palestine-site-catchment-analysis/7AFF7937750FF5D3377D6600E3A86173)

**Implementation rule:** generate candidate **residence–resource bundles**, rather than scoring residential cells independently.

For each bundle, calculate whether the founding households can obtain sufficient food, water, fuel, shelter, and necessary materials with their available labor and technology. Resource access must respect existing ownership, common-use rights, and competing users.

A useful accounting identity is:

\[
\text{food available}
=
\text{usable stocks}
+\text{expected production}
+\text{credible imports}
-\text{seed requirements}
-\text{losses}
-\text{obligations}.
\]

Evaluate this by food type through TCE’s nutrition system, not solely as gross harvest tonnage.

Annual totals are insufficient. A site can be productive in the long run but impossible to establish immediately because the group arrives after planting, lacks seed, or cannot build shelter before winter. Test survival **month by month until the first dependable food supply**, including the labor diverted to construction.

### 1.2 Water has four separate properties: quantity, reliability, accessibility, and quality

A river-adjacency bonus conflates very different conditions. Water can be abundant but seasonal, nearby but contaminated, or physically accessible only through another community’s land. Water collection also consumes labor that cannot simultaneously be spent farming or building.

WHO’s work on domestic water explicitly distinguishes availability from accessibility and relates collected quantities to travel time and service level. These are useful engineering relationships, although modern service categories are not measurements of ancient consumption. [World Health Organization](https://www.who.int/publications/i/item/9789240015241)

**Implementation rule:** represent sources as flows and stores, with seasonal variation, access permissions, collection capacity, queues, and contamination pathways. Keep domestic, livestock, irrigation, and industrial demands separate.

A well, reservoir, or channel should change actual access and reliability—not simply add settlement capacity.

**Do not hard-code “avoid marshes.”** Historical perceptions were more discriminating. Vitruvius warned against unhealthy stagnant locations but also described acceptable coastal marsh sites with drainage and reported the relocation of Salapia to a healthier location. This is evidence about ancient planning and perceived health, not validation of his medical explanations. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Vitruvius/1%2A.html)

For TCE, separate vector habitat, contaminated water, and crowding-related transmission. Wetlands may simultaneously offer useful food, transport, materials, and protection; the Upper Xingu evidence includes extensive wetland management within inhabited landscapes. [ResearchGate](https://www.researchgate.net/publication/23222376_Pre-Columbian_Urbanism_Anthropogenic_Landscapes_and_the_Future_of_the_Amazon)

### 1.3 Defense changes the preferred settlement form, not just its location

A defensible residential core may impose longer journeys to fields and water. Dispersed homes may improve agricultural access while making mutual assistance harder.

**Implementation rule:** compare expected losses and labor costs under alternative layouts:

\[
\text{defensive benefit}
-
\text{extra travel}
-
\text{fortification maintenance}
-
\text{resource-access restrictions}.
\]

Let threats change the balance. Aggregation can become attractive during insecurity; a remote field station can become untenable even when its soil remains excellent.

Evidence from mobile pastoralists in Cameroon illustrates the importance of this qualification: resource-based distribution predictions fit some years better than years of insecurity. The authors discuss insecurity as a possible explanation, rather than establishing a universal causal coefficient. [Moritz Lab](https://mlab.osu.edu/sites/mlab.osu.edu/files/Moritz%20et%20al%202014%20Ideal%20free%20distribution.pdf)

For siege conditions, distinguish ordinary access from secure access. A hilltop with an exposed external spring is not equivalent to one with a protected well.

### 1.4 Fuel and mineral advantages depend on usable technology and transport

Resources should not provide timeless attraction bonuses. An inaccessible deposit, an unknown ore, or a fuel unsuitable for available processes need not attract settlers.

The European coal evidence is especially useful: proximity to coalfields was not associated with faster city growth before 1750 in the study, but became important after coal-using industrial technologies spread. This is a technology–geography interaction, not an eternal “coal tile” advantage. [OUP Academic](https://academic.oup.com/ej/article/131/635/1135/5955447)

**Implementation rule:** derive resource value from actual production chains:

\[
\text{resource value}
=
\text{expected sale or use value}
-\text{extraction cost}
-\text{processing cost}
-\text{delivery cost}.
\]

For wood and other renewable resources, track accessible stocks, regeneration, harvesting rights, and increasing collection distance. Allow substitution where technology permits it.

Transport should likewise be functional. A river may provide drinking water without being navigable. A crossing, landing, portage, or junction matters because it changes travel costs or concentrates exchange—not because a terrain tag supplies free commerce.

### 1.5 Village fission is a coalition process, not a population cap

Bandy’s southern Titicaca analysis interprets early village fission in terms of internal coordination problems and increasing relocation costs, not simply exhaustion of agricultural carrying capacity. He also associates the later cessation of fission with integrative institutions. The archaeological evidence is suggestive and case-specific; some proposed mother–daughter relationships remain uncertain. [ResearchGate](https://www.researchgate.net/publication/227795396_Fissioning_Scalar_Stress_and_Social_Evolution_in_Early_Village_Societies)

**Implementation rule:** households periodically compare several responses to dissatisfaction:

* Remain and intensify production.
* Negotiate institutional changes or new access rights.
* Join an existing settlement.
* Establish a daughter settlement or split into several groups.

Generate potential migrant coalitions from actual relationships. A primary study of Yanomama village splits found that lineal, family-associated fission differed substantially from random splitting. TCE should therefore not select an arbitrary half of a village’s population. [OUP Academic](https://academic.oup.com/genetics/article-abstract/98/1/179/5995107)

Candidate coalitions should consider labor composition, shared assets, interpersonal conflict, mutual assistance, and support from the parent settlement. A daughter settlement may retain ritual affiliation, exchange obligations, marriage connections, or political subordination.

**No universal “150 people means split” rule is justified.** Model grievances, coordination capacity, opportunities elsewhere, and barriers to exit. Institutional adaptation and fission should be competing possibilities.

### 1.6 Central-place theory: implement demand thresholds and travel costs, not hexagons

Christaller’s central-place framework concerns settlements supplying services to surrounding areas. Skinner’s work on rural China extends this into interconnected economic and social market systems, including periodic markets. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952)

For TCE, translate that insight into a provider-level calculation:

\[
\Pi\_{j,s}
=
\sum\_h D\_{h,j}\,P(h\text{ patronizes }s)\,m\_j
-F\_{j,s},
\]

where \(D\_{h,j}\) is household demand for service \(j\), \(m\_j\) is contribution margin per unit, and \(F\_{j,s}\) is its fixed cost over the same period.

A specialist becomes viable when accessible demand covers fixed costs. Customers choose among providers using delivered price, journey time, reliability, affiliation, and restrictions.

This produces several useful possibilities without scripting a hierarchy. Common services can survive in small catchments; rarer services require larger ones. Several villages may support a periodic market before any supports permanent specialist employment. An itinerant provider can serve multiple markets.

On a uniform plane, regularly spaced centers with hexagonal catchments obey:

\[
A=\frac{\sqrt{3}}{2}d^2,
\]

where \(A\) is catchment area and \(d\) is center-to-center spacing. Thus a hypothetical \(50\text{ km}^2\) catchment implies \(d\approx7.6\text{ km}\).

**That is geometry, not a historical spacing law.** In TCE, rivers, mountains, settlement history, political boundaries, and uneven demand should distort the pattern. Calculate market access using network travel costs, not circles.

### 1.7 Persistence comes from useful assets and coordination—not an age bonus

American portage towns persisted after portaging lost its original economic significance. Colonial Kenyan cities also persisted through major changes in settlers, traders, and railway importance. These studies demonstrate that an initial locational advantage can shape a durable settlement network. [OUP Academic](https://academic.oup.com/qje/article/127/2/587/1825072)

**Implementation rule:** persistence should arise from assets and relationships that continue to provide value: housing, cleared land, terraces, wells, roads, workshops, markets, recognized rights, and social networks.

Do not subtract historical construction expenditure merely because it was once paid. Economically relevant relocation costs are the **future services forgone, replacement costs, disruption, and coordination problems**.

Persistence must remain reversible. Michaels and Rauch’s comparison of Britain and France after Roman rule argues that stronger continuity could preserve an urban network less well positioned for later transport conditions. A reset can sometimes improve location—not merely destroy accumulated value. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.1111/ecoj.12424)

### 1.8 Abandonment should be a process with several possible meanings

Distinguish population decline, departure of permanent residents, seasonal reuse, complete disuse, nearby relocation, and later reoccupation.

**Implementation rule:** settlement decline occurs through household decisions and institutional failure. Departures can reduce market demand, maintenance labor, security, and mutual assistance, making departure more attractive to those remaining.

Do not require absolute resource exhaustion. In the Long House Valley simulation, environmental conditions could support some population after the real valley was completely depopulated around 1300 CE. That mismatch is an important warning against treating ecological capacity as a sufficient explanation of abandonment. [PNAS](https://www.pnas.org/doi/10.1073/pnas.092080799)

A settlement’s physical site, its community, and its political identity should therefore have separate persistent identifiers.

---

## 2. Parameters: empirical anchors versus design assumptions

### 2.1 Published anchors

Confidence below concerns the **specific observation or reconstruction**. Transfer to another region or subsistence system is usually less certain. Published model inputs are explicitly distinguished from empirical measurements.

| Quantity | Published value or range | Units and interpretation | Source and confidence |
| --- | --- | --- | --- |
| Water collection under “basic access” | **100–1,000 m**, or **5–30 minutes total collection time**; collected quantity generally unlikely to exceed **20 L/person/day** | Modern access category, not a minimum physiological requirement or ancient consumption estimate | Howard & Bartram, WHO 2003, Table S1. **Moderate** for the access relationship; **low** for direct ancient transfer. [ResearchGate](https://www.researchgate.net/publication/252913246_Domestic_Water_Quantity_Service_Level_and_Health) |
| Water use with intermediate access | About **50 L/person/day**, associated with on-plot access | Domestic use; does not include field irrigation or livestock | Same source. **Moderate**, strongly dependent on infrastructure and use practices. [ResearchGate](https://www.researchgate.net/publication/252913246_Domestic_Water_Quantity_Service_Level_and_Health) |
| Forager residential-group scale | Mean experienced band size **28.2 adults**; study covered **32 societies**, **5,067 individuals** | Adults, not total residents; experienced group size is not an unweighted mean of all camps | Hill et al. 2011. **Moderate** for sampled societies; **low–moderate** for prehistoric transfer. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21393537/) |
| Early village fission-associated sizes | **3.0, 3.5, and 5.0 ha**; corresponding population-index values **157, 186, 277** | Southern Titicaca case. Population indices are reconstructed from occupation area, not censuses | Bandy 2004. **Moderate** for mapped occupation areas; **low** as a universal demographic threshold. [ResearchGate](https://www.researchgate.net/publication/227795396_Fissioning_Scalar_Stress_and_Social_Evolution_in_Early_Village_Societies) |
| Rural Chinese standard-market catchment | Modal modeled case: **52.5 km²**, **7,870 people**, density **150 people/km²**; center spacing **7.79 km** | Skinner’s historical reconstruction; spacing is calculated under a regular-hexagon assumption, not directly observed regularity | Skinner, Table 1, estimates for agricultural China in 1948. **Moderate** regional anchor; **low** universal transfer. [journals.openedition.org](https://journals.openedition.org/etudesrurales/docannexe/image/7952/img-14.jpg) |
| Upper Xingu settlement clusters | Major cluster centers about **20 km** apart; estimated polity territory **≥250 km²** and population **≥2,500** | A dispersed settlement system, not one continuously built-up town | Heckenberger et al. 2008. **Moderate** for mapped spacing; **lower** for population reconstruction. [ResearchGate](https://www.researchgate.net/publication/23222376_Pre-Columbian_Urbanism_Anthropogenic_Landscapes_and_the_Future_of_the_Amazon) |
| Residence–field distance in an established ABM | **1 km** in stated residential rules; **1.6 km** in its base-case parameter table | **Published model assumptions**, not an observed universal farming radius | Axtell et al. 2002. High confidence about what the model specifies; **low** as a transferable empirical parameter. [PNAS](https://www.pnas.org/doi/10.1073/pnas.092080799) |
| Industrial resource-location effect | Study’s indexed city-growth series, 1750–1900: **721%** cumulative growth within **25 km** of coalfields versus **293%** farther away | Descriptive contrast, not an annual growth bonus or an isolated causal effect | Fernihough & O’Rourke 2021. **Moderate–high** within this historical dataset; **low** as a direct simulation coefficient. [OUP Academic](https://academic.oup.com/ej/article/131/635/1135/5955447) |

These anchors are best used to constrain **outcomes and mechanisms**, rather than copied indiscriminately into agent code.

### 2.2 A worked physical example: why water distance matters

Suppose, illustratively, that a settlement of 100 people collects 20 L/person/day, carries 20 L per trip, and walks at 4 km/h.

At a source 500 m away, each round trip covers 1 km and takes 15 minutes before filling or queuing. The settlement needs 100 trips:

\[
100\text{ trips/day}\times0.25\text{ hours/trip}
=
25\text{ person-hours/day}.
\]

Moving the source to 200 m away reduces walking to about 10 person-hours/day.

This is an **illustrative calculation**, not an archaeological estimate. Its value is that TCE can derive the attraction of a nearby source from actual daily work. The carrying capacity and speed are assumptions; the source’s flow alone does not determine the benefit.

### 2.3 Recommended initial calibration priors

The following are **proposed TCE starting ranges, not measured historical constants**. Their numerical confidence is low until tested against appropriate cases.

| Parameter | Initial range | Units | Intended use |
| --- | --- | --- | --- |
| Ordinary unladen walking speed | **3–5** | km/hour | Base movement parameter; modify for slope, load, surface, weather, age, and health |
| Preferred agricultural commute | **15–60** | minutes one-way | Soft cost range, not a prohibition on distant fields |
| Human-carried water load | **10–25** | L/trip | Adult task assumption; derive actual loads from capacity and equipment |
| New agrarian founding cohort | **5–20** | households | Scenario initialization only; replace a minimum headcount with labor, supplies, and support checks |
| Founding food buffer | Time until dependable production **plus 1–3 months** | months of household consumption | Reserve seed separately; vary with support networks and risk tolerance |
| Household planning horizon | **2–5** | years | Compare establishment costs and plausible near-term returns; avoid perfect centuries-long foresight |
| Conservative yield forecast | **10th–30th percentile** | percentile of believed local yield distribution | Sensitivity-test risk aversion; desperate groups may accept worse prospects |
| Voluntary-move persistence requirement | **1–3** | seasonal reviews | Require a sustained advantage to prevent oscillation; waive during acute danger |
| Candidate bundles evaluated | **16–64** | bundles per group/review | Engineering prior for bounded search, not an anthropological observation |

**Do not assign a universal village lifespan, fission population, marsh penalty, or settlement spacing.** Those should emerge from more elementary conditions. Initial cohorts are also not closed breeding populations: continued migration and marriage connections matter for long-run continuity.

---

## 3. Variation across subsistence systems, eras, and regions

The same decision machinery can serve all periods, but the relevant resources, transport technologies, property arrangements, and coordination costs must change.

| Context | What differs in the evidence | TCE implication |
| --- | --- | --- |
| **Foragers and hunter-fisher communities** | Residential organization can be flexible and socially mixed. California’s Channel Islands also had permanent settlements, so farming is not a necessary prerequisite for residential permanence. High- and middle-ranked resource locations were occupied early; lower-ranked habitat entered the record later. [Arizona State University](https://asu.elsevierpure.com/en/publications/co-residence-patterns-in-hunter-gatherer-societies-show-unique-hu/) | Support seasonal camps, recurrent sites, and permanent settlements. Resource concentration, storage, and mobility costs—not an “agriculture unlocked” flag—should influence permanence. |
| **Early farming villages: southern Titicaca** | Bandy’s case combines village growth, inferred fission, increasing difficulty of relocation, and stronger integrative institutions. [researchgate.net](https://www.researchgate.net/publication/227795396_Fissioning_Scalar_Stress_and_Social_Evolution_in_Early_Village_Societies) | Permit social fragmentation before food exhaustion. When accessible land becomes scarce, groups may negotiate, intensify, or develop institutions rather than automatically split. |
| **Riverine agrarian systems: Indus region** | Giosan et al. associate settlement distribution with changing fluvial conditions. After approximately **3,900 years ago**, deurbanization included reduced settlement sizes, abandonment in some locations, continuity in others, and a proliferation of smaller settlements farther east. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1112743109) | River landscapes must evolve. A favorable balance of accessible water, productive floodplain, and stable residential ground can change without a global “civilization collapse” event. |
| **Pre-industrial market systems: China** | Standard markets linked villages into larger economic and social communities, with periodic schedules as well as differences in service hierarchy. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952) | Distinguish a village, its market center, and the wider market community. A functional settlement system may contain many residential nodes. |
| **Pre-Columbian tropical landscapes: Upper Xingu** | Roads linked walled towns and smaller villages within managed agricultural, forest, and wetland mosaics. Settlement clusters were not simply compact cities surrounded by untouched wilderness. [ResearchGate](https://www.researchgate.net/publication/23222376_Pre-Columbian_Urbanism_Anthropogenic_Landscapes_and_the_Future_of_the_Amazon) | Allow dispersed urban functions, managed woodland, engineered wetlands, and overlapping resource-use areas. Do not define “city” solely by continuous dense housing. |
| **Mobile pastoralism: Logone floodplain, Cameroon** | Resource distributions and seasonal mobility are central; an open-access resource regime can support distributed adjustment, while insecurity complicates resource-based predictions. [Moritz Lab](https://mlab.osu.edu/sites/mlab.osu.edu/files/Moritz%20et%20al%202014%20Ideal%20free%20distribution.pdf) | Support seasonal residence and mobile resource rights. A recurring camp need not become a permanent village, and access institutions can matter as much as resource quantity. |
| **Industrial and colonial settlement systems** | Coal’s importance changed with technology in Europe. In Kenya, railways influenced the location of settlers, traders, and cities, and those cities persisted after the initial constellation changed. [OUP Academic](https://academic.oup.com/ej/article/131/635/1135/5955447) | Permit new transport and production systems to reorder opportunities, while existing centers retain advantages through useful investments and coordinated activity. |
| **Modern settlement expansion** | Settlement expansion does not necessarily avoid hazards. Rentschler et al. find that high-hazard settlements in East Asia expanded **60% faster** than flood-safe settlements in their post-1985 comparison. [PubMed](https://pubmed.ncbi.nlm.nih.gov/37794266/) | Hazard exposure should enter decisions alongside jobs, land access, infrastructure, prices, and perceived protection. Do not prohibit growth in risky locations merely because the simulation knows the true hazard. |

For TCE, these are **configurations of mechanisms**, not compulsory stages. A technologically sophisticated society might retain periodic markets or seasonal pastoral mobility; a non-farming society might sustain permanent settlements.

---

## 4. Stylized facts and validation targets

A correct simulation should reproduce conditional patterns—not the same map or universal village size in every world.

| Pattern to reproduce | Evidence or benchmark | Suggested validation |
| --- | --- | --- |
| **Early occupation favors attractive resource bundles, but not in perfect rank order** | On Santa Rosa Island, early permanent occupation included high- and middle-ranked locations; settlement in the studied low-ranked habitat was confined to the late Holocene, after **3,600 calibrated years BP**. Drought could change relative rankings. [Springer](https://link.springer.com/article/10.1007/s10816-015-9267-6) | Compare founding dates with **contemporaneous** resource suitability, including uncertainty and accessibility. Do not rank sites using conditions centuries later. |
| **Residential land and productive land need not coincide** | Harappan settlement geography includes stable surfaces and relationships to river valleys and floodplain margins. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1112743109) | Measure residential flood exposure, travel time to fields, and productive land lost beneath buildings. Settlements should not systematically pave their best farmland. |
| **Fission is socially structured** | The Yanomama study documents differences between lineal and random fission. [OUP Academic](https://academic.oup.com/genetics/article-abstract/98/1/179/5995107) | Compare kinship and relationship density within departing coalitions against random groups of equal size. Track subsequent contact between parent and daughter settlements. |
| **Market hierarchy and residential hierarchy differ** | Skinner’s standard-market community encompasses a catchment larger than the market town itself. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952) | Measure provider viability, customer travel time, market-day attendance, and villages per market—not merely settlement populations. |
| **Some locational advantages outlive their original cause, but not all networks are permanent** | Portage persistence and the Britain–France urban-reset comparison provide contrasting outcomes. [OUP Academic](https://academic.oup.com/qje/article/127/2/587/1825072) | Remove a transport advantage after development. Test persistence with surviving capital and networks, then compare destruction, maintenance failure, and coordinated relocation scenarios. |
| **Abandonment can occur while some local subsistence remains feasible** | The Long House Valley model’s residual population after historical depopulation is a direct warning against resource-only explanations. [PNAS](https://www.pnas.org/doi/10.1073/pnas.092080799) | Test whether lost security, exchange, or institutional support can cause departure without setting local food production to zero. |
| **Risky settlement and long-run regret remain possible** | Modern expansion into flood zones provides an observed counterexample to universal hazard avoidance. [PubMed](https://pubmed.ncbi.nlm.nih.gov/37794266/) | Compare actual hazards, household beliefs, constraints, and realized losses. Agents should sometimes choose sites that later prove costly. |

Also inspect settlement-size distributions, nearest-neighbor travel times, founding and abandonment rates, parent–daughter distances, and reoccupation frequency.

**Do not force an exact Zipf distribution.** With a small number of towns, a broad size hierarchy is a more defensible target than a precisely fitted power law. Evaluate several outcomes together; matching total population while placing settlements implausibly is not a successful calibration.

---

## 5. Modeling recommendation for TCE

### 5.1 Use households for strategic decisions and individuals for execution

A household-level planner is compatible with persistent individual agents. Its inputs should be calculated from its actual members: food needs, labor availability, skills, health, dependents, preferences, and relationships.

| Simulation object | Required state |
| --- | --- |
| **Resource patch / environmental cell** | Terrain, soils, moisture, seasonal water, renewable stocks, hazards, access rights, improvements, and transport connections |
| **Individual** | Persistent residence and relationships, physical needs, skills, task capacity, knowledge, and participation in household decisions |
| **Household** | Members, stores, tools, land rights, obligations, expected production, migration preferences, and beliefs about candidate locations |
| **Settlement** | Residential nodes, shared facilities, market functions, maintenance obligations, institutions, and membership |
| **Founding coalition** | Participating households, commitments, supplies, target bundle, travel plan, construction plan, and agreement about rights |
| **Site history** | Occupation episodes, former structures, improvements, claims, abandonment, reoccupation, and connections to predecessor communities |

Do not make “settlement” synonymous with “polity.” Two settlements may share government; one dispersed community may occupy several residential clusters.

### 5.2 Implement a staged founding process

A practical sequence is:

```
Trigger a household or institutional review.
Compare staying, adaptation, joining, and founding.
Form a coalition and assemble known candidate bundles.
Evaluate seasonal feasibility, risks, rights, and establishment costs.
Resolve competing claims and confirm commitments.
Travel, establish shelter and water access, and begin production.
Reassess survival, membership, and continued occupation.
```

This makes founding visible in daily life. People scout, transport supplies, negotiate access, build, fetch water, and plant. A “founded” notification should describe a real process already occurring, not create houses and residents instantaneously.

Keep success uncertain. Founding can stop at a temporary camp, become a dependent outpost, fail during establishment, or mature into a durable settlement.

### 5.3 Use bounded knowledge, not a global optimizer

Maintain separate **world truth** and **agent beliefs**.

Scouting reveals some water sources and resources, but perhaps not dry-season reliability. Reports can be stale. A familiar mediocre site may be preferred over a poorly understood apparently superior one.

The household decision can take the general form:

\[
V\_h(s)
=
E\!\left[\sum\_{\tau=0}^{H}\delta^\tau
u\_h(\text{consumption, leisure, safety, rights, social ties})\right]
-C\_h^{\text{move}}(s)
-\rho\_h R\_h(s).
\]

This is a proposed decision model. Express all terms in consistent utility units, and avoid counting the same risk twice through both expected outcomes and an additional penalty.

Group agreement should not reduce everything to the mean household score. A coalition whose total benefit is positive may still fail because several essential households bear most of the costs. Model bargaining, compensation, exit, or institutional coercion explicitly.

### 5.4 Derive resource capacity instead of storing a fixed settlement capacity

A settlement’s feasible population should be conditional on crops, labor, technology, trade, rights, and risk.

For a simplified agricultural check:

\[
A\_{\rm required}
=
\frac{Nq}{y\_{\rm usable}},
\]

where \(q\) is the annual quantity required per person and \(y\_{\rm usable}\) is output per hectare after relevant deductions. Apply this separately to actual food requirements and supplement it with labor, rotation, water, and storage constraints.

Then test whether the necessary land is **accessible and available**, not merely present within a radius.

Likewise, trade should relax local food constraints only when someone actually produces, transports, and sells the food. Two founding groups must not both count the same unallocated field or the same surplus shipment.

### 5.5 Keep the strategic layer inexpensive

For 10,000–50,000 individuals, the expensive mistake would be evaluating every map cell for every person every day.

A suitable initial architecture is:

**Spatial layers.** Use relatively coarse environmental cells—perhaps the proposed 100–250 m range—for candidate generation, while retaining finer building and task geometry. Represent roads, rivers, passes, and crossings in transport networks.

**Different clocks.** Update collection and consumption daily; farming and migration opportunities seasonally; major network changes when infrastructure, borders, or services change. Trigger emergency reviews immediately.

**Cached candidate bundles.** Reuse route costs and resource summaries, invalidating them when conditions change. Household-specific calculations then apply rights, skills, labor, relationships, and beliefs.

**Deterministic allocation.** Batch competing land and water claims or otherwise resolve them explicitly. Do not let entity update order decide which settlers receive the best land.

As an illustrative operation count, 50,000 people at five people per household implies 10,000 households. Evaluating 32 bundles each is 320,000 bundle evaluations per strategic review—not per rendered frame. This is an architectural estimate, not a performance benchmark.

At this explicit population scale, also avoid validating against million-person metropolitan systems without declaring how external populations and markets are represented.

### 5.6 Existing models worth borrowing from

| Model | What to borrow | Important limitation |
| --- | --- | --- |
| **Long House Valley / Artificial Anasazi**, Axtell et al. 2002 | Household stores, reconstructed annual production, and separate residence and farming decisions | Primarily a subsistence and demographic model; its failure to produce complete historical depopulation identifies missing mechanisms. [PNAS](https://www.pnas.org/doi/10.1073/pnas.092080799) |
| **Artificial Long House Valley**, individual-based extension | A bridge from household abstractions toward individual demographic processes | Do not assume the richer demography alone solves institutional or regional migration questions. [CoMSES Net](https://www.comses.net/codebases/812fb67a-761b-41dc-81ef-d1c736568f83/releases/1.0.0/) |
| **Village Ecodynamics Project** | Household settlement decisions responding to natural and social environments; archaeological calibration | Regional behavioral assumptions need reworking for TCE’s other ecologies and institutions. [CoMSES Net](https://www.comses.net/codebases/2518/releases/1.1.0/) |
| **MayaSim**, Heckbert 2013 | Coupled settlement growth, migration, trade networks, land degradation, forest succession, and hydrology | Its principal agents are settlements. The original paper presents a proof of concept requiring further refinement and archaeological calibration, not a validated general explanation of Maya history. [JASSS](https://www.jasss.org/16/4/11.html) |
| **Logone pastoral-mobility model**, Moritz and colleagues | Testing whether simple local movement rules generate larger-scale resource matching | Seasonal mobile pastoralism is not equivalent to permanent agrarian founding; information and access assumptions matter. [CoMSES Net](https://www.comses.net/codebases/4242/releases/1.0.0/) |

For TCE, the strongest combination is **household resource accounting and persistent individuals**, coupled to **settlement-level trade, institutions, and environmental feedbacks**.

---

## 6. Sources, datasets, and evidence limits

### Useful datasets

| Dataset or archive | Appropriate use | Important restriction |
| --- | --- | --- |
| **D-PLACE**, including ethnographic and forager datasets | Compare residence, subsistence, household organization, and environmental context across societies | Use each observation’s time, place, and seasonal focus. These are not timeless attributes of a culture; some source datasets explicitly contain multiple observations for one society. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) |
| **Village Ecodynamics Project / tDAR archives** | Regional settlement histories and model calibration against archaeological evidence | Inspect coverage, chronology, and population-estimation methods before treating records as complete censuses. [tDAR](https://core.tdar.org/project/425611/village-ecodynamics-project-i) |
| **HYDE 3.2**, Klein Goldewijk et al. 2017 | Broad historical population and land-use context at approximately **5 arc-minute** resolution | It is a reconstruction, not a catalogue of observed ancient villages. Use for regional constraints, not exact founding locations. [ESSD](https://essd.copernicus.org/articles/9/927/2017/essd-9-927-2017.html) |
| **World Settlement Footprint Evolution**, DLR | Modern settlement expansion, **1985–2015**, at **30 m** resolution | Detects settlement footprint, not household motives, political identity, or ancient occupation. [EOC Geoservice](https://geoservice.dlr.de/web/maps/eoc%3Awsfevolution) |
| **SoilGrids**, ISRIC | Soil properties and uncertainty for environmental benchmarking; **250 m** resolution | Contemporary predicted soil properties are not direct reconstructions of ancient fertility or yields. [ISRIC](https://www.isric.org/explore/soilgrids) |
| **HydroSHEDS** | Catchments, rivers, and lakes for testing network and landscape algorithms | A modern drainage network does not reconstruct past river courses; palaeohydrological evidence is needed for historical cases. [HydroSHEDS](https://www.hydrosheds.org/) |

For foundational reading, begin with **Christaller’s *Die zentralen Orte in Süddeutschland* (1933)** and **Vita-Finzi, Higgs and colleagues’ “Prehistoric Economy in the Mount Carmel Area of Palestine: Site Catchment Analysis” (1970)**. They provide contrasting service-network and resource-catchment perspectives. Skinner’s *Marketing and Social Structure in Rural China* supplies an especially useful bridge between economic geography and social organization. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952)

The most directly actionable empirical studies for this module are Bandy on fission, Jazwa–Kennett–Winterhalder on changing habitat rankings, Giosan and colleagues on fluvial landscapes, Bleakley–Lin and Jedwab–Kerby–Moradi on persistence, and Michaels–Rauch on urban resetting. Their value is as **different mechanism tests**, not as interchangeable sources of universal constants. [ResearchGate](https://www.researchgate.net/publication/227795396_Fissioning_Scalar_Stress_and_Social_Evolution_in_Early_Village_Societies)

### Where uncertainty remains consequential

**Founding motives are often inferred rather than observed.** Favorable terrain, occupation timing, and material similarities constrain explanations but do not uniquely reveal what founders intended. Even Bandy’s detailed analysis acknowledges ambiguity in particular mother–daughter identifications. [ResearchGate](https://www.researchgate.net/publication/227795396_Fissioning_Scalar_Stress_and_Social_Evolution_in_Early_Village_Societies)

**Settlement area is not population.** Compact villages, dispersed farmsteads, and multi-center communities require different conversion assumptions. An excavated or surveyed occupation area should not become an exact headcount.

**Ecological and institutional explanations are not mutually exclusive.** Climate can change available choices; ownership and power determine who can use them; relationships influence who moves together. A resource-only model can match aggregate population while missing the actual cause of departure.

**Central-place regularity is conditional.** Skinner’s tabulated spacings are partly geometric constructions. Treat them as structured hypotheses, not measurements proving that historical settlements formed a regular lattice. [journals.openedition.org](https://journals.openedition.org/etudesrurales/docannexe/image/7952/img-14.jpg)

**No credible universal founding-group size or settlement lifespan emerges from these sources.** Calibrate distributions by livelihood, landscape, mobility, and institutions. Test against multiple regions and retain uncertainty in archaeological dates and population reconstructions.

The resulting design principle is straightforward: **simulate access, labor, risk, relationships, and coordination, then let settlement patterns emerge.** A founding decision should make some futures easier and others harder without deciding, centuries in advance, which village becomes a city.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928e5-7c74-83e9-a1f9-1b06c2df2a60)
