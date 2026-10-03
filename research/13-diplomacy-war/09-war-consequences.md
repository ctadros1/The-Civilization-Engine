# Social and economic consequences of war

## A simulation-ready report for The Civilization Engine

**The central recommendation is to model war as interacting losses of people, productive assets, access, and institutional coordination—not as one “devastation” value that decays after peace.** A settlement can rebuild its houses without recovering its population; recover its population through immigration without restoring displaced families; or regain its prewar output while remaining far below the output it would have produced without war.

The empirical literature supports both persistent damage and substantial recovery. Cross-country studies find economic losses lasting a decade or longer, while research on Japanese cities and Vietnamese districts finds considerable recovery in the *geographical distribution* of economic activity. These are different outcomes, not necessarily contradictory findings. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S016407042100001X?utm_source=chatgpt.com)

For TCE, keep separate records of **deaths and disability; displacement and missing births; destroyed and inaccessible capital; disrupted production and trade; public liabilities; and institutional and interpersonal consequences**. Their interaction should determine recovery.

---

# 1. Mechanisms: implementable causal rules

The rules below are proposed implementations. Historical findings motivate them, but their exact functional forms are modeling choices.

## 1.1 Death, injury, captivity, and demographic loss are different events

Maintain two independent classifications:

| Dimension | Categories |
| --- | --- |
| Status when affected | Combatant, civilian, captive, displaced person |
| Outcome and cause | Direct violent death; death from disease, hunger, or exposure; injury; lasting impairment; capture; disappearance; desertion |

A soldier dying of infection is an indirect military death. A civilian killed during a battle is a direct civilian death. “Casualties” must not become synonymous with deaths.

Historical records demonstrate why this matters. In the American Civil War, approximately twice as many soldiers died from disease as from battle wounds. That composition reflects the conditions of those armies; it is not a universal relationship between battle deaths and disease deaths. [National Park Service](https://www.nps.gov/articles/death-and-dying.htm)

**TCE rules.** Let the battle and siege systems create wounds, captures, and immediate deaths. Resolve subsequent survival through the ordinary health system, conditional on injury, treatment, shelter, nutrition, and infection. A surviving injured worker may require care, recover gradually, change occupation, or retain an impairment. Capture transfers control over a living person; it does not remove that person from world population.

Use a population identity:

\[
N\_{t+1}=N\_t+B\_t-D\_t+I\_t-O\_t
\]

Here, \(D\_t\) means actual deaths, including ordinary mortality. **Excess war mortality is an analytical comparison with a no-war baseline, not another category of deaths to subtract.** For the entire simulated world, internal migration cancels; for individual settlements it does not.

## 1.2 Disease follows exposure, movement, and service failure

Armies and displaced populations can create new concentrations of people, move pathogens, overload water supplies, and interrupt care. Hunger can increase vulnerability, but it should not be the mandatory explanation for every epidemic. Outram’s study of the Thirty Years’ War emphasizes the relationships among military activity, civilian flight, and disease transmission, rather than treating mortality simply as starvation following destruction. [White Rose Research Online](https://eprints.whiterose.ac.uk/id/eprint/385/1/outramq1.pdf)

**TCE rules.** Use settlement, camp, and traveling-group exposure conditions. War changes crowding, water quality, sanitation, food availability, and access to caregivers. The ordinary disease model then produces mortality.

A convenient competing-risk implementation is:

\[
p\_i(\text{death during }\Delta t)
=1-\exp\left[-\sum\_c \lambda\_{i,c}\Delta t\right]
\]

Each cause-specific hazard must be defined without duplicating the same risk in both the baseline and a “war multiplier.” Nutrition may modify infection severity rather than independently killing the same person a second time.

Do not end these mechanisms at the peace treaty. In a large African study, elevated infant mortality associated with nearby conflict remained detectable for years after the violence. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6338336/)

## 1.3 War changes births, household composition, and future labor supply

A wartime population deficit can contain deaths, emigration, and births that never occurred. Surviving households may lose providers or caregivers, postpone family formation, separate during displacement, or reorganize around relatives.

The resulting damage can extend beyond the exposed generation. Research on Nigeria’s Biafran War finds adverse adult outcomes among exposed women and effects on their children; the results also indicate that subsequent education policy could mitigate some consequences. [National Bureau of Economic Research](https://www.nber.org/papers/w23721)

**TCE rules.** Recalculate household labor, food needs, dependency, and care obligations after every death, disappearance, or separation. Derive births from the normal household and reproductive model rather than subtracting an arbitrary percentage for “war.”

Track birth cohorts. A small wartime cohort should become a small cohort of apprentices, workers, parents, and taxpayers later. Conversely, peace should not instantly replenish working-age people.

Inheritance, remarriage, adoption, and kin support should redistribute both assets and obligations. Their consequences depend on the institutions actually present.

## 1.4 Displacement is a repeated household process, not deletion and respawning

Distinguish current location, former home, legal residence, property claims, and intended destination. A household may flee repeatedly, split, return temporarily, or settle permanently elsewhere.

This is not a minor accounting issue. The Timor-Leste reconstruction estimated roughly **108,200 households experiencing 282,800 displacement events** during 1974–1999: displacement events substantially exceeded the number of affected households. [HRDAG - Human Rights Data Analysis Group](https://hrdag.org/timorlestefaqs/)

**TCE rules.** Households compare the expected safety and livelihood of staying against feasible destinations:

\[
U\_{h,d} =
\text{safety}+\text{food access}+\text{livelihood}+\text{kin support}
-\text{journey cost}-\text{housing cost}-\text{exclusion risk}
\]

Use bounded knowledge and accessible routes, not perfect global information. Very poor households may be unable to leave; sick or dependent members may slow departure. Evacuation, expulsion, and forced relocation should be separate coercive events, not voluntary utility maximization.

Receiving settlements gain people and skills but also additional demand for housing, food, and services. Model those quantities directly. Refugees should not carry an intrinsic productivity or instability penalty.

Return requires more than reduced danger: property access, viable livelihoods, family preferences, and freedom of movement matter. UNHCR cautions that recorded returns can occur under adverse or unsafe conditions; return counts therefore do not establish durable recovery. [UNHCR](https://www.unhcr.org/publications/global-trends-2024)

## 1.5 Destruction, seizure, and loss of access have different economic consequences

A burned mill is destroyed capital. A captured mill is transferred capital. An intact mill without grain, workers, or a safe road is inaccessible or underutilized capital.

The distinction is visible in modern damage assessments. The World Bank’s early-2017 Syria assessment separately estimated destroyed housing, partially damaged housing, displacement, and cumulative foregone GDP. Those quantities cannot be combined into one replacement-cost figure. [World Bank](https://www.worldbank.org/en/country/syria/publication/the-toll-of-war-the-economic-and-social-consequences-of-the-conflict-in-syria)

**TCE rules.** Give productive assets separate states for physical condition, ownership, access, staffing, and required inputs. Looted goods enter another inventory unless consumed or destroyed. Confiscation changes ownership and creates a claim or grievance; it does not automatically destroy the object.

For early agrarian TCE, prioritize:

* Seed grain, food reserves, tools, and breeding or draft animals.
* Planting and harvesting labor at the correct season.
* Irrigation, storage, mills, bridges, and other complementary infrastructure.

A missed planting window should cause a later harvest deficit. Eating seed stocks should improve present survival while reducing future production. These consequences should follow production recipes and calendars rather than an annual war penalty.

For production chains with strong complementarities, use output-equivalent bottlenecks:

\[
Y\_j=\min\left(Y\_j^{\rm labor},Y\_j^{\rm capital},
Y\_j^{\rm inputs},Y\_j^{\rm transport}\right)
\]

More substitutable activities can use smoother production functions.

## 1.6 Trade disruption propagates damage beyond the battlefield

War can interrupt exchange without physically damaging both trading partners. In the estimates of Novta and Pugacheva, official exports and imports remained substantially below their counterfactual paths a decade after conflict onset. Official trade statistics also need not capture every informal or diverted exchange. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S016407042100001X?utm_source=chatgpt.com)

**TCE rules.** Give each transport connection capacity, travel time, security, tolls, and expected losses. Merchants may reroute, reduce cargo, demand higher compensation, or stop trading. Producers then respond to actual input shortages and changed prices.

This supports asymmetric outcomes: an isolated city suffers shortages, a safer port gains traffic, and a nearby farming district may lose its market even though no army enters it. Trade recovery may remain incomplete after a road reopens because merchants, financing, or customers have relocated.

Do not simultaneously impose a large “trade disruption GDP penalty” if missing trade is already reducing production through these mechanisms.

## 1.7 War finance redistributes burdens and can undermine recovery

Tax rates, collectible revenue, and administrative capacity are separate quantities:

\[
T=\tau \times \text{taxable base}
\times \text{collection coverage}
\times \text{compliance}
\]

A higher rate can coincide with lower revenue when the tax base disappears or collection becomes impossible.

Modern evidence illustrates the financing problem: IMF research on defense buildups finds substantial near-term deficit financing and increases in public debt, especially during wartime. These are findings about specific modern fiscal systems, not borrowing capacities to give early agrarian governments. [IMF](https://www.imf.org/en/blogs/articles/2026/04/08/wars-impose-lasting-economic-costs-while-more-defense-spending-means-hard-choices)

**TCE rules.** Let institutions choose among taxes, tribute, requisition, compulsory labor, asset sales, borrowing, and monetary financing where the relevant mechanisms exist. Requisition should transfer real food, animals, or materials from identifiable owners.

Record debt as a liability with a creditor, maturity, and payment obligation. A simplified budget identity is:

\[
B\_{t+1}=(1+i\_t)B\_t+G\_t-T\_t-A\_t-S\_t-W\_t
\]

Here \(G\) is non-interest spending, \(A\) grants, \(S\) monetary financing, and \(W\) debt write-offs. Apply consistent time units and matching counterparty entries.

Debt service transfers resources and constrains choices; it does not physically burn buildings. Arrears, default, inflation, or confiscation affect different creditors and households differently. The government’s ability to finance reconstruction must remain distinct from the economy’s ability to supply builders, timber, stone, and food.

## 1.8 Institutional and social outcomes are conditional

War can encourage investment in tax administration and legal capacity when rulers expect durable benefits from common-interest spending. Besley and Persson formalize this mechanism, including complementarities between fiscal and legal capacity. It is not a theorem that every war produces a stronger state. [National Bureau of Economic Research](https://www.nber.org/papers/w13028)

The historical interpretation is contested. Centeno emphasizes how Latin American warfare could destroy institutions and reinforce internal divisions. Schenoni’s research instead highlights the importance of outcomes: victorious state-building coalitions and defeated states could follow very different postwar trajectories. [Penn State University Press](https://www.psupress.org/books/titles/978-0-271-02165-2.html?utm_source=chatgpt.com)

**TCE rules.** Track administrative capacity, legitimacy, coercive capacity, and military autonomy separately. Recruitment and emergency taxation can produce negotiated rights, stronger administration, repression, or resistance depending on bargaining power and institutions. Military rule should become more feasible when armed organizations retain cohesion and independent resources while civilian authority weakens—not simply when a war ends.

Likewise, do not translate victimization into universal hostility. A research synthesis finds that exposure to war can increase local cooperation and civic participation, sometimes in parochial forms rather than generalized trust. [National Bureau of Economic Research](https://www.nber.org/papers/w22312)

Represent targeted grievances, kin obligations, local solidarity, and trust in institutions separately. Compensation, accountability, propaganda, renewed violence, and successful cooperation should update them.

---

# 2. Parameters, quantitative anchors, and recovery rates

## 2.1 How to interpret the evidence

The following values are mainly **calibration outcomes**, not coefficients to apply directly to agents.

**H** means relatively strong evidence for the stated observation or narrowly defined result. **M** means useful evidence with important estimation or measurement qualifications. **L** means weak identification or poor generalizability. Confidence concerns the specified context; even a high-confidence observation may transfer poorly to another society.

### Population and social consequences

| Quantity and context | Quantitative anchor and units | Confidence and appropriate use |
| --- | --- | --- |
| **Military mortality composition: American Civil War** | Approximately **2 disease deaths per death from battle wounds**—roughly two-thirds of military deaths from disease. | **M–H** for broad composition. Validate the possibility of disease-dominated army mortality; do not impose this ratio on all armies. [National Park Service](https://www.nps.gov/articles/death-and-dying.htm) |
| **Infant mortality near African conflict, 1995–2015** | Birth within **50 km** of conflict associated with **5.2 additional infant deaths per 1,000 live births**, 95% CI **3.7–6.7**; approximately **7.7% above baseline**. Effects detectable to **100 km** and **8 years** afterward. | **M–H** within the study; **L** as an early-agrarian coefficient. A validation target for spatially and temporally extended indirect harm. Wagner et al., *Lancet* (2018). [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6338336/) |
| **Timor-Leste mortality, 1974–1999** | Conservative minimum-bound estimate of **102,800** conflict-related deaths: approximately **18,600 killings/disappearances** and **84,200 excess hunger/illness deaths**. Combatant killings were excluded from the estimation. | **M**; survey and statistical reconstruction, not a complete death register. Demonstrates potentially dominant indirect civilian mortality without supplying a universal multiplier. CAVR/HRDAG. [HRDAG - Human Rights Data Analysis Group](https://hrdag.org/timorlestefaqs/) |
| **Birth-count disruption: Augsburg, Thirty Years’ War** | Baptisms in **1632–1635 averaged about 65%** of their prewar level; approximately **55%** during the subsequent local respite discussed by Outram. | **M** for recorded counts; **L** for inference about individual fertility. Counts combine population loss, displacement, and changed reproduction. [White Rose Research Online](https://eprints.whiterose.ac.uk/id/eprint/385/1/outramq1.pdf) |
| **Partition of India, 1947** | Census reconstruction estimates **14.5 million arrivals**, **17.9 million departures**, and a **3.4 million accounting gap**. | **M** for reconstructed flows. The gap is not a verified death count. Useful for distinguishing movement, disappearance from records, and mortality. Bharadwaj, Khwaja, and Mian (2008). [Princeton University](https://collaborate.princeton.edu/en/publications/the-big-march-migratory-flows-after-the-partition-of-india/) |
| **Early farming: Schöneck-Kilianstädten** | Mass grave containing **at least 26 individuals**, approximately **13 adults and 13 subadults**, around 5000 BCE. | **H** for an extreme local event; **L** for annual incidence. Supports catastrophic settlement-level tails, not a routine Neolithic mortality rate. Meyer et al., *PNAS* (2015). [DOI](https://doi.org/10.1073/pnas.1504365112) |

### Economic and fiscal consequences

| Quantity and context | Quantitative anchor and units | Confidence and appropriate use |
| --- | --- | --- |
| **Housing damage: Syria, assessment as of early 2017** | **7% of housing stock destroyed; 20% partially damaged.** | **M**. Keep destroyed and damaged categories separate. National averages conceal much larger local losses. World Bank (2017). [World Bank](https://www.worldbank.org/en/country/syria/publication/the-toll-of-war-the-economic-and-social-consequences-of-the-conflict-in-syria) |
| **Foregone output: Syria, 2011–2016** | Approximately **US$226 billion cumulative GDP loss**, around **four times 2010 annual GDP**. | **M**, dependent on the counterfactual. This is accumulated foregone production, not the value of destroyed capital. [World Bank](https://www.worldbank.org/en/country/syria/publication/the-toll-of-war-the-economic-and-social-consequences-of-the-conflict-in-syria) |
| **Long-run national output: Novta–Pugacheva** | Estimated **GDP per capita about 28% lower ten years after conflict onset**. | **M**. Outcome benchmark for their sample and population-relative conflict definition; not “subtract 28% after every war.” *Journal of Macroeconomics* (2021). [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S016407042100001X?utm_source=chatgpt.com) |
| **Long-run official trade: same study** | **Exports about 58% lower; imports about 34% lower**, ten years after onset. | **M**. Relative to estimated counterfactual paths; unsuitable as independent penalties added to an already disrupted economy. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S016407042100001X?utm_source=chatgpt.com) |
| **Alternative modern output estimates: IMF, 2026** | Approximately **3% output loss at onset**, increasing to roughly **7% within five years**; losses persist a decade later. | **M**. Different conflict definition, sample, horizon, and output measure from the preceding study. Do not pool the estimates into one confidence interval. [IMF](https://www.imf.org/en/blogs/articles/2026/04/08/wars-impose-lasting-economic-costs-while-more-defense-spending-means-hard-choices) |
| **Modern defense buildups: IMF, 2026** | Typical buildup lasts nearly **3 years**, raises military spending by **2.7 percentage points of GDP**; deficits worsen about **2.6 points**, and debt rises about **7 points**, or **14 points in wartime**, within three years. | **M**. Conditional averages for defense-spending booms, not every war onset. Useful for validating modern fiscal responses. [IMF](https://www.imf.org/en/blogs/articles/2026/04/08/wars-impose-lasting-economic-costs-while-more-defense-spending-means-hard-choices) |

**Interpretation:** different studies identify different treatments and outcomes. A national output loss, a damaged-housing share, and a mortality increase are not interchangeable measures of “war severity.”

## 2.2 Recovery has several denominators

Track at least four recovery measures:

| Recovery measure | Question it answers |
| --- | --- |
| **Absolute recovery** | Has the settlement regained its prewar population, output, or usable capital? |
| **Counterfactual recovery** | Has it caught up with a plausible no-war trajectory? |
| **Spatial recovery** | Has it regained its previous share of regional population or activity? |
| **Household/cohort recovery** | Have displaced families, injured survivors, and exposed children regained their earlier opportunities? |

Davis and Weinstein’s Japanese evidence concerns the resilience of city population patterns after wartime destruction. Miguel and Roland find no negative district-level effects of bombing on several Vietnamese development indicators through **2002**. Neither finding means that victims recovered their lost lives, health, property, or opportunities, nor that the country experienced no aggregate loss. [National Bureau of Economic Research](https://www.nber.org/papers/w8517)

### Demographic recovery: a useful mathematical constraint

With no net migration, an initial population loss \(d\), and constant subsequent net growth \(r\), time to regain the old population is:

\[
t=\frac{\ln[1/(1-d)]}{\ln(1+r)}
\]

The following are **calculated scenarios, not historical growth estimates**:

| Initial population loss | At 0.5% annual net growth | At 1% | At 2% |
| --- | --- | --- | --- |
| **20%** | 44.7 years | 22.4 years | 11.3 years |
| **50%** | 139.0 years | 69.7 years | 35.0 years |

These calculations assume the growth rate can actually be sustained despite altered age structure, nutrition, and family formation. They also recover only the *old level*. Catching a growing no-war population requires faster relative growth or immigration.

This is why a severely depopulated TCE district should not automatically refill over a few peaceful years.

## 2.3 Suggested engineering priors

The ranges below are **proposed sensitivity-test settings, not empirical estimates**. Their source is this modeling recommendation; empirical confidence is **low**. They are useful for development before richer calibration exists.

| Parameter | Starting sensitivity range | Implementation condition |
| --- | --- | --- |
| Reconstruction allocation | **5–20% of annual output** | An attempted allocation; actual work remains constrained by food surplus, materials, labor, and security. |
| Safe-return probability | **5–30% per eligible displaced household per year** | Apply only when return is feasible and preferred; zero when physically or coercively blocked. |
| Recoverable-capital gap closure, for a simplified model | **5–20% of the remaining repairable gap per year** | Approximate half-times of **13.5–3.1 years**. Replace with construction jobs in detailed simulation; never run both systems simultaneously. |
| Agrarian restart test | **1–3 harvests** | Test scenario with secure access and restored seed, tools, and labor—not a guaranteed recovery time. |
| Personal event-salience half-life | **5–30 years** | A memory-sensitivity parameter, not a clinical trauma-recovery rate. Institutional commemoration and new events can renew salience. |
| Recurring displacement | **No one-event limit** | Retain identity and property claims across every move; count moves separately from people. |

For a yearly probability \(p\_y\), convert to a timestep of \(\Delta t\) years with:

\[
p\_{\Delta t}=1-(1-p\_y)^{\Delta t}
\]

Do not divide a large annual probability by the number of ticks and assume exact equivalence.

---

# 3. Variation across eras and world regions

**Use these as configurations of institutions, technology, settlement structure, and disease environments—not scripted eras.**

| Setting | Evidence and variation | Implication for TCE |
| --- | --- | --- |
| **Foragers and hunter-fisher-gatherers** | Fry and Söderberg’s sample of **21 mobile-forager societies** distinguished collective warfare from interpersonal killings, executions, and other lethal events. Separately, the **61-person Jebel Sahaba cemetery** in the Nile Valley provides evidence consistent with recurrent violent episodes, not one simple battle. These samples do not establish a universal forager war-death rate. [CARTA](https://carta.anthropogeny.org/node/308928) | Mobility, access to alternative territory, kin networks, and group fission may matter more than buildings or public debt. Destruction of social relationships and access rights can be severe even where fixed capital is limited. |
| **Early farming settlements** | Early Neolithic massacre evidence demonstrates that individual communities could suffer catastrophic losses, while providing weak denominators for estimating how often such events occurred. [DOI](https://doi.org/10.1073/pnas.1504365112) | Stored food, planting labor, seed, livestock, and settlement defensibility become critical. A few deaths can eliminate a scarce skill or leave a household unable to cultivate its land. |
| **Pre-industrial states and agrarian empires** | The Thirty Years’ War illustrates interactions among armies, flight, disease, and depressed births. China’s Taiping conflict also shows the difficulty of separating death from displacement and administrative disappearance; conventional totals are much less secure than their apparent precision suggests. [White Rose Research Online](https://eprints.whiterose.ac.uk/id/eprint/385/1/outramq1.pdf) | Model armies living partly from local resources, seasonal production, limited reserves, uneven administrative reach, and long-distance transmission. Fiscal recovery depends on rebuilding taxable activity and collection, not merely restoring the nominal tax rate. |
| **Industrial mass warfare** | The American Civil War demonstrates disease-heavy military mortality; Japanese bombing studies demonstrate substantial postwar spatial recovery. South Asian Partition demonstrates enormous population redistribution and the difficulty of reconciling census movements. [National Park Service](https://www.nps.gov/articles/death-and-dying.htm) | Mass mobilization, transport networks, concentrated industry, large fiscal claims, and demographic restructuring become more important. A rebuilt city may contain different households and ownership patterns. |
| **Modern conflicts** | African child-mortality studies, Timor-Leste’s reconstructed civilian losses, and Syria’s damage assessment show different combinations of indirect mortality, displacement, service failure, and capital loss. Biafran research demonstrates possible intergenerational consequences. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6338336/) | Medical capacity can protect people only when accessible and supplied. Dense service dependencies create opportunities for cascading failure. Refugee destinations, financial systems, education, and long-lived infrastructure complicate recovery. |

Regional diversity also applies to institutions. Recent research on areas affected by the Taiping regime identifies persistent differences in property rights, local capacity, and later outcomes. Together with the Latin American debate, this argues against assigning a single global “war strengthens the state” coefficient. [ZEW](https://www.zew.de/en/publications/stationary-bandits-state-capacity-and-the-malthusian-transition-the-lasting-impact-of-the-taiping-rebellion)

There is no well-supported universal ordering in which each later technological configuration produces a larger civilian share of deaths, faster rebuilding, or stronger postwar government.

---

# 4. Stylized facts and validation targets

A correct simulation should reproduce **conditional patterns**, not force every war to resemble the same historical average.

| Pattern | What a convincing simulation should demonstrate |
| --- | --- |
| **Indirect mortality can dominate** | A poorly supplied army or displaced population can suffer substantial disease and hunger losses even with relatively few battle deaths. Validate against the Civil War and Timor-Leste examples without fixing their ratios. [National Park Service](https://www.nps.gov/articles/death-and-dying.htm) |
| **Damage extends beyond battle locations and dates** | Nearby populations can suffer through disrupted services and exposure after fighting stops. The African infant-mortality results provide a modern spatial and temporal benchmark. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6338336/) |
| **Population loss and death counts diverge** | A town may empty through flight; another may regain population through newcomers. Census disappearance must not automatically become mortality, as the Partition reconstruction illustrates. [Princeton University](https://collaborate.princeton.edu/en/publications/the-big-march-migratory-flows-after-the-partition-of-india/) |
| **Economic loss can greatly exceed physical replacement cost** | Intact assets can remain unproductive, and missed production accumulates year after year. The Syrian assessment explicitly distinguishes housing damage from cumulative GDP loss. [World Bank](https://www.worldbank.org/en/country/syria/publication/the-toll-of-war-the-economic-and-social-consequences-of-the-conflict-in-syria) |
| **Local rebound and persistent aggregate losses can coexist** | Rebuilding attractive locations need not close national counterfactual output gaps. Japanese and Vietnamese local studies should be tested separately from cross-country macroeconomic estimates. [National Bureau of Economic Research](https://www.nber.org/papers/w8517) |
| **Institutional outcomes depend on political survival and bargaining** | Victory, defeat, administrative destruction, military autonomy, and coalition continuity should produce different trajectories—not a guaranteed postwar government upgrade. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ajps.12552?utm_source=chatgpt.com) |
| **Greater local cooperation need not mean universal reconciliation** | Some exposed agents should participate more in local mutual aid while retaining grievances against particular groups or institutions. [National Bureau of Economic Research](https://www.nber.org/papers/w22312) |

For TCE’s own testing, use **matched war and no-war runs**, preserving initial conditions and, where practical, matched random streams. Report distributions across runs.

Particularly useful experiments are a raid immediately before planting versus after harvest; destruction of an ordinary workshop versus the only functioning mill; similar casualties concentrated among different age groups; a blocked refugee route; and peace with versus without secure property access. These are proposed causal tests, not claims that historical episodes held all other variables constant.

Measure total population, age structure, consumption per person, usable capital, idle capital, displacement spells, tax receipts, debt service, and recovery relative to both the old level and the paired counterfactual.

---

# 5. Modeling recommendation for 10k–50k individual agents

## 5.1 Represent people individually, but aggregate exposure and infrastructure

| Entity | Minimum persistent state |
| --- | --- |
| **Person** | Age, household, location, skills, health/injury, military or captive status, care obligations, selected event memories |
| **Household** | Food and assets, labor availability, dependents, current shelter, home claim, destination knowledge, displacement history |
| **Building/farm** | Physical condition, usable capacity, owner, access rights, required inputs, repair job, hazardous remnants where applicable |
| **Settlement/district** | Water and sanitation service, market access, food stocks, security, local disease exposure, available repair labor |
| **Polity/institution** | Treasury, creditor-linked debt, revenue rules, collection reach, administrative staff, legitimacy, military autonomy |
| **Transport connection** | Capacity, condition, control, travel time, tolls, expected losses, closure state |
| **War event** | Time, location, participants, affected people/assets, immediate outcomes, responsibility as perceived by observers |

Store **administrative capacity through operational components**—staff, records, offices, communication, and territorial access—rather than only a scalar that increases when rulers spend money.

Use a scalar summary for decision-making or dashboards, but derive it from these components.

## 5.2 Resolve at different timescales

**Event-driven or daily:** violent events, movement arrivals, injury outcomes, food consumption, acute exposure, and inventory transfers.

**Weekly:** household migration decisions, labor reassignment, merchant routing, shelter allocation, and repair scheduling.

**Monthly or seasonal:** budgets, recruitment reviews, construction progress, planting and harvest, and institutional bargaining.

During time acceleration, batch quiet intervals but stop at consequential events: a food reserve threshold, harvest, debt maturity, route closure, or renewed attack. The proposed architecture does not require reevaluating every political preference every simulation tick.

For memory, keep a bounded set of salient personal events and group-level narratives. Avoid an all-person-pairs grievance matrix.

## 5.3 Make reconstruction a real production process

For repairable asset \(j\):

\[
R\_j=
\min\left(
R\_j^{\rm labor},
R\_j^{\rm materials},
R\_j^{\rm food\ support},
R\_j^{\rm finance}
\right)
\times s\_j\times a\_j
\]

All \(R\) terms use consistent units of restored capacity per period; \(s\_j\) and \(a\_j\) represent security and authorized access. Cap repairs at the actual repairable deficit.

This creates useful endogenous choices. Restoring a bridge may unlock several industries; restoring housing may enable workers to return; repairing irrigation may increase the food surplus available to support builders.

**Do not automatically rebuild everything in its original location.** Let households and enterprises abandon, relocate, consolidate, or change production when geography, ownership, or markets have changed.

## 5.4 Existing models and games worth borrowing from

| Model or game | Relevant mechanism | What to borrow—and what not to infer |
| --- | --- | --- |
| **Flee**, including Suleimenova, Bell, and Groen (2017) | Agent-based displacement along geographical networks, informed by conflict and refugee data. | Borrow route graphs, destination attractiveness, and destination constraints. It is not a complete endogenous model of war, health, and economic recovery. [Nature](https://www.nature.com/articles/s41598-017-13828-9) |
| **Epstein’s civil-violence model; NetLogo Rebellion** | Heterogeneous grievance, legitimacy, perceived arrest risk, and local collective dynamics. | Borrow the separation of dissatisfaction from willingness to act. Treat it as a generative model, not a validated universal rebellion probability. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC128592/) |
| **Victoria 3**, documented market-expansion design | Devastation affects infrastructure and market connections, with downstream living-standard and turmoil effects. | Borrow the infrastructure–market–welfare feedback. Replace aggregate populations and balancing parameters with TCE’s people, inventories, and buildings. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-37-market-expansion) |

For the first implementation, prioritize **household consequences, food-calendar disruption, displacement, infrastructure bottlenecks, and the fiscal ledger**. Detailed trauma dynamics, differentiated debt markets, and intergenerational transmission can follow once those foundations produce plausible outcomes.

---

# 6. Sources, datasets, and evidence limits

## 6.1 Datasets suitable for calibration

| Source | Best use | Important limitation |
| --- | --- | --- |
| **UCDP**, including Georeferenced Event Dataset **26.1** and armed-conflict series | Event location, timing, conflict actors, and estimates of direct fatalities. GED coverage begins in 1989; the armed-conflict series extends back to 1946. | Direct-event records are not total excess mortality. Pin versions, retain uncertainty estimates, and distinguish organized violence categories. Do not import modern absolute death thresholds as TCE’s definition of war. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/) |
| **Correlates of War** | Longer-run interstate and other war comparisons, starting in 1816, with dates and participant information. | Coverage endpoints differ across constituent datasets. Selection thresholds, missing values, and inconsistent historical reporting limit small-war and pre-industrial inference. [Correlates of War](https://correlatesofwar.org/data-sets/cow-war/) |
| **UNHCR population statistics and Global Trends; IDMC displacement database** | Displacement stocks, new movements, returns, origins, and destinations. | Stocks are not annual flows; movements are not unique people; legal categories differ. Return is not necessarily permanent reintegration. [UNHCR](https://www.unhcr.org/publications/global-trends-2024) |
| **Demographic and Health Surveys linked to conflict events** | Birth histories and child survival around conflict exposure, following studies such as Wagner et al. | Survivor reporting and exposure measurement require care; the study’s identification strategy should travel with any extracted coefficient. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC6338336/) |
| **CAVR/HRDAG Timor-Leste data and methods** | Combining testimony, retrospective mortality surveys, and other records; separating political killings from excess hunger and illness. | Missing survivors and incomplete social memory can produce downward bias. Combatant mortality was not comprehensively estimated. [HRDAG - Human Rights Data Analysis Group](https://hrdag.org/timorlestefaqs/) |
| **SIPRI Military Expenditure Database** | Modern military spending, including expenditure relative to GDP and government spending; current series reaches 2025, with coverage varying by country. | Spending does not directly measure mobilized labor, casualties, destruction, or borrowing. It cannot calibrate early agrarian military costs without a separate resource model. [SIPRI](https://www.sipri.org/databases/milex) |
| **Author replication packages** | Reproducing particular causal estimates and spatial damage studies. | Use corrected versions. Miguel and Roland’s Vietnam study has a **2024 corrigendum** addressing geographical-data problems; its central conclusions were largely unchanged. [Econ at Berkeley](https://emiguel.econ.berkeley.edu/wordpress/wp-content/uploads/2013/07/corrigendum-2023-09-19.pdf) |

## 6.2 Claims to flag as contested or thin

**Pre-industrial death totals are often less secure than population-loss estimates.** Outram reviews major disputes over Thirty Years’ War estimates and their underlying denominators. A dramatic fall in recorded inhabitants may mix deaths, flight, missing registrations, and suppressed births. Conventional Taiping death totals likewise should not be treated as precise calibration targets. [White Rose Research Online](https://eprints.whiterose.ac.uk/id/eprint/385/1/outramq1.pdf)

**Archaeological violence samples establish possibilities more readily than frequencies.** A massacre grave or unusually violent cemetery cannot by itself supply annual war mortality for an entire technological configuration. Preservation, site selection, and uncertain population-at-risk denominators matter. [DOI](https://doi.org/10.1073/pnas.1504365112)

**Recovery studies differ in what they identify.** A district-level bombing comparison can show limited persistent *relative local* damage while missing countrywide losses or harms experienced by people who moved away. National macroeconomic estimates face different counterfactual and conflict-selection problems. [Econ at Berkeley](https://emiguel.econ.berkeley.edu/research/the-long-run-impact-of-bombing-vietnam/)

**No reliable universal coefficient governs grievance decay, postwar military rule, refugee return, or early-agrarian reconstruction speed.** Treat these as mechanisms with explicit uncertainty and sensitivity tests, rather than hiding invented precision inside a historical-looking parameter table.

## Bottom line for TCE

**Peace should remove or reduce the causes of destruction; it should not erase their consequences.** Recovery should require surviving people to obtain food, restore access, settle claims, mobilize resources, and rebuild functioning organizations. Some places recover, some relocate, some remain smaller, and some rebuild under different owners and institutions.

The decisive test is not whether burned districts eventually look repaired. It is whether the simulation preserves the difference between **a rebuilt place, a recovered economy, and the people whose lives the war changed**.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92960-db4c-83ea-843d-a5101357f53f)
