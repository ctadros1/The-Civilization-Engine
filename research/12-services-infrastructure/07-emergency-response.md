# Emergency dispatch and service coverage for TCE

## Executive recommendation

**Use an event-driven response simulation in which incidents, information, responders, equipment, and travel are real. Treat coverage as a measured probability—not a radius attached to a building.**

Separate three systems:

| System | What it determines |
| --- | --- |
| **Incident and response simulation** | Who notices an emergency, who becomes available, who travels, when useful work begins, and what happens afterward. |
| **Coverage measurement** | The probability of receiving a particular kind of help within a given time, including the possibility of receiving none. |
| **Institutional planning** | Whether to recruit another watch, move equipment, establish a station, improve communications, negotiate mutual aid, or build a road. |

Location-set covering, maximal covering, and p-median models belong principally in the third system. They answer useful investment questions, but their basic formulations do not reproduce actual dispatch, busy crews, incomplete teams, or delayed notification. Queue-aware models address some of those omissions; explicit simulation should determine TCE’s realized outcomes. [PubsOnLine](https://pubsonline.informs.org/doi/10.1287/opre.19.6.1363)

The central historical design principle is equally important: **detection, communication, mobilization, transport, and effective intervention are separate capabilities.** A society can develop some without developing all the others.

---

## 1. Mechanisms: implementable rules

### 1.1 Represent the complete response chain

Record separate timestamps for:

**Occurrence → detection → notification → dispatch → departure → arrival → useful action → sufficient response → resolution → resource release.**

For a simple sequential response:

\[
T\_{\mathrm{effective}}=
T\_{\mathrm{detection}}+
T\_{\mathrm{notification}}+
T\_{\mathrm{decision}}+
W\_{\mathrm{resources}}+
T\_{\mathrm{mobilization}}+
T\_{\mathrm{travel}}+
T\_{\mathrm{access/setup}}
\]

Here, \(W\_{\mathrm{resources}}\) is waiting for an appropriate responder or resource bundle to become available.

This equation is an accounting framework, not a requirement that everything happen sequentially. When responders gather simultaneously, equipment travels separately, or one crew begins work while another approaches, use an event dependency graph. Readiness occurs when the necessary prerequisites have finished.

**Keep several response measures.** The first person arriving, the first water applied, the first trained responder arriving, and a rescue-capable team becoming ready can be different events. Official statistics may measure something narrower: England’s fire response series, for example, measures receipt of the call to arrival of the first vehicle—not ignition to effective firefighting. [GOV.UK](https://www.gov.uk/government/statistics/fire-and-rescue-incident-statistics-year-ending-march-2026/fire-and-rescue-incident-statistics-year-ending-march-2026--2)

**TCE rule:** never allow an arrival timestamp alone to determine incident success. Advance the incident according to the capabilities actually operating at it.

### 1.2 Generate incidents from activities; generate reports separately

For an activity or location with instantaneous incident hazard \(h(t)\), a convenient timestep-independent rule is:

\[
P(\text{incident during }\Delta t)
=1-\exp[-h(t)\Delta t]
\]

Let \(h(t)\) depend on the relevant subsystem: hearth use, material condition, industrial work, weather, violence, individual health, and so forth.

The emergency-service subsystem should then determine whether the incident is noticed and reported. These are separate events.

**Recommended implementation rules:**

* A fire can exist without a report. A report can exist without a fire.
* Witnesses need appropriate proximity, visibility, hearing, attention, or information from another person.
* Reports contain perceived location and severity, not necessarily the true values.
* Multiple reports may describe one incident; deduplicate them without assuming perfect initial identification.
* Shared shocks—storms, attacks, conflagrations—should affect multiple incident hazards and transport conditions together.

False alarms and non-fire duties can materially occupy a fire service. In England’s year ending March 2026, services attended 170,597 fires, 255,047 fire false alarms, and 215,583 non-fire incidents. A model that occupies fire crews only when something is burning would miss most attended events in that particular system. These proportions are not suitable historical defaults. [GOV.UK](https://www.gov.uk/government/statistics/fire-and-rescue-incident-statistics-year-ending-march-2026/fire-and-rescue-incident-statistics-year-ending-march-2026--2)

Where TCE already generates an injury or fire explicitly, do not generate the same demand again through a population-based background call rate.

### 1.3 Make information travel

Before centralized communications exist, notification should be a physical process: someone shouts, rings a bell, carries a message, signals from a tower, or reaches an authority.

A lookout changes the probability and timing of **detection**. A signal network changes **notification**. Neither creates an available crew.

Historical evidence supports separating these functions. Meng Yuanlao’s account of Song-era Kaifeng describes street patrol posts, elevated fire lookouts, stored equipment, and mounted messengers reporting fires to responding military bodies. That is a multi-stage information-and-response system, not an undifferentiated “fire station.” [Project Gutenberg](https://www.gutenberg.org/cache/epub/24137/pg24137-images.html)

**TCE rule:** a dispatcher may act only on information available to that institution. The player’s diagnostic interface can know more than the institution does.

Notification can also fail because the witness cannot leave, the receiving post is unattended, a signal is misunderstood, or the institution declines the request.

### 1.4 Capacity is a compatible bundle of resources

A station is infrastructure. A response unit is a temporarily assembled bundle:

\[
\text{Unit}=
\{\text{people, required skills, equipment, transport, consumables}\}
\]

Examples include a bucket party, a watch patrol, a pump team, a stretcher party, or a modern ambulance crew.

A nominal roster does not establish operational capacity. People may be working elsewhere, asleep, ill, already responding, unwilling to participate, or unable to reach the equipment.

For an on-call team, readiness depends on the arrival of enough **eligible** people, including required roles. It is not necessarily the arrival of the first \(k\) volunteers: the missing person might be the only trained operator.

**TCE rules:**

* Reserve people and equipment atomically so they cannot serve two missions simultaneously.
* Represent shared duties explicitly: the same watch members may handle fires, patrols, arrests, or gate security.
* Deduct emergency participation from ordinary work, rest, and household activities.
* Model continuous staffing in labor-hours, not headcount alone.

For example, one continuously occupied post requires 168 person-hours per week. With an assumed 40 effective hours per worker, that is **4.2 workers before additional absence allowances**. This is arithmetic illustrating the staffing problem, not a historical workweek assumption.

### 1.5 Dispatch the nearest appropriate available response—not the nearest building

A useful baseline dispatch policy is:

> Choose the highest-priority actionable incident; assign the available, permitted resource bundle with the earliest predicted time to useful action.

For candidate unit \(u\) and incident \(i\):

\[
\widehat{T}\_{ui}
=
\widehat{T}\_{\mathrm{remaining\ mobilization},u}
+
\widehat{T}\_{\mathrm{route}}(x\_u,x\_i)
+
\widehat{T}\_{\mathrm{setup},ui}
\]

Use the unit’s actual location \(x\_u\), including a patrol location or a position on a return journey.

Eligibility should depend on capability, jurisdiction, information, and institutional rules. A nearby untrained helper may be useful immediately but cannot substitute for every specialized capability.

Later institutions can add a reserve penalty:

\[
\operatorname{score}(u,i)=\widehat{T}\_{ui}
+\theta\,\operatorname{cost\ of\ leaving\ other\ areas\ uncovered}
\]

The parameter \(\theta\) represents an authored policy preference, not a universal optimum.

Do not let a high-priority incident needing an unavailable specialist block an unrelated idle resource from serving a lower-priority incident. Conversely, do not allow unrestricted preemption of work already protecting someone from immediate harm.

### 1.6 Travel on the same world network as everyone else

Use directed, mode-specific paths through roads, alleys, bridges, gates, stairs, navigable water, and building entrances.

Travel costs should incorporate the capabilities already represented in TCE’s movement model: load, slope, crowding, weather, darkness, road condition, and vehicle restrictions. Emergency priority may change traffic interactions, but it should not erase blocked bridges or make a wheeled pump fit through an impassable alley.

Keep three quantities distinct:

| Quantity | Purpose |
| --- | --- |
| Forecast travel time | Dispatch decisions. |
| Realized travel time | Actual mission progress and outcomes. |
| Access/setup time | Reaching the patient, raising ladders, establishing water delivery, or preparing tools after road arrival. |

**Do not sample an entire response time and then add physical travel to it.** That double-counts part of the delay.

### 1.7 Model useful work, replenishment, and release

At the scene, represent a small task graph rather than a single “service applied” action.

For firefighting, tasks might include obtaining water, carrying it, operating a pump, reaching an opening, rescuing occupants, and creating a firebreak. Effective water application is constrained by the bottleneck among source supply, transport, pumping, and application capacity.

Crew complementarity matters. In NIST’s residential fire experiments, four-person crews completed a set of 22 essential tasks substantially faster than two- or three-person crews. This supports task-and-role constraints, not a universal linear “each extra firefighter adds X% effectiveness” rule. [NIST](https://www.nist.gov/news-events/news/2010/04/landmark-residential-fire-study-shows-how-crew-sizes-and-arrival-times)

Define a unit’s busy period as the time until it becomes eligible for another assignment. That may include treatment, transport, handover, restocking, or recovery. Return travel counts as busy **only while the unit is not dispatchable**.

For medical incidents, distinguish first assistance from definitive treatment. For policing, distinguish arrival from resolving an ongoing threat or conducting a subsequent investigation.

### 1.8 Define coverage as a distribution

For location \(i\), incident class \(k\), time/state \(t\), and response threshold \(\tau\):

\[
C\_{ik}(\tau,t)
=
P\!\left(T^{\mathrm{effective}}\_{ik}\leq\tau
\mid i,k,t\right)
\]

Specify the action represented by “effective”: first water, trained first aid, rescue-capable crew, and so on.

Treat never receiving the specified response as a separate outcome—or as probability mass at \(T=\infty\). Otherwise, dropping unserved incidents can make a failing service appear faster.

The interface should distinguish:

* **Potential access:** travel from equipped locations assuming resources are ready.
* **Operational coverage:** probability of timely response with actual availability and dispatch.
* **Realized service:** what happened to actual incidents, including failures and outcomes.

A building near a station can have excellent potential access and poor operational coverage when its crew is occupied.

### Worked example: an early-town fire

An illustrative, not historical, scenario:

A fire is noticed after 90 seconds. A messenger covers 240 m at 2 m/s. An available pump party takes two minutes to prepare, travels 600 m at 1 m/s, and needs two minutes to establish operation.

\[
T\_{\mathrm{effective}}
=1.5+2+2+10+2
=\mathbf{17.5\ minutes}
\]

A nearby resident might begin limited bucket work much earlier. Another station is only one possible improvement: a nearer water cache, better signaling, a ready team, or a passable shortcut could change different parts of the same chain.

---

## 2. Planning models, dispatch distributions, and queues

### 2.1 Location models: what each actually optimizes

Let \(i\) index demand locations, \(j\) candidate facilities, \(w\_i\) demand or harm weights, \(t\_{ij}\) network travel times, and \(x\_j\) indicate an opened facility. Define \(a\_{ij}=1\) when \(t\_{ij}\leq S\), for a chosen threshold \(S\).

#### Location-set covering: meet a minimum access requirement everywhere

\[
\min \sum\_j f\_jx\_j
\qquad
\text{subject to }
\sum\_j a\_{ij}x\_j\geq1
\quad\forall i
\]

The classic emergency-facility formulation is associated with Toregas, Swain, ReVelle, and Bergman (1971). The equal-cost version minimizes facility count; the weighted version minimizes opening cost. [PubsOnLine](https://pubsonline.informs.org/doi/10.1287/opre.19.6.1363)

**TCE use:** an institution promises that every recognized settlement or ward should have a post within a specified potential travel time.

**Limitation:** a covered point may still receive no timely response if its only covering crew is busy. Physical infeasibility should remain possible.

#### Maximal covering: serve as much weighted demand as the budget permits

\[
\max\sum\_iw\_iy\_i
\]

subject to:

\[
y\_i\leq\sum\_j a\_{ij}x\_j,\qquad
\sum\_jx\_j\leq p
\]

Church and ReVelle’s maximal covering model chooses a limited number of facilities to cover the greatest weighted demand. [Springer](https://link.springer.com/article/10.1007/BF01942293)

**TCE use:** a council can afford only \(p\) posts and chooses which locations to protect.

**Limitation:** low-density peripheral settlements may be excluded. The hard threshold also treats locations just inside and just outside \(S\) very differently.

#### P-median: minimize average weighted travel

\[
\min\sum\_i\sum\_jw\_it\_{ij}z\_{ij}
\]

with each demand point assigned to an opened facility and exactly \(p\) facilities selected. Hakimi’s graph-location work provides foundational median-location theory. [PubsOnLine](https://pubsonline.informs.org/doi/10.1287/opre.12.3.450)

**TCE use:** minimize routine patrol travel, expected dispatch distance, or transport costs.

**Limitation:** a low mean can coexist with very poor worst-case access. Supplement it with maximum-time constraints, subgroup guarantees, or explicit penalties for unserved demand.

For all three models, distinguish **facility count** from **deployable unit count**. Opening a second garage without funding another compatible crew does not necessarily add response capacity.

### 2.2 Expected covering: include a first approximation to busyness

Daskin’s maximum expected covering model, MEXCLP, accounts approximately for units being unavailable. Under an identical, independent busy probability \(q\), a location covered by \(m\) units has:

\[
P(\text{at least one available})=1-q^m
\]

For an illustrative \(q=0.4\), one, two, and three covering units give availability probabilities of **60%, 84%, and 93.6%**. [PubsOnLine](https://pubsonline.informs.org/doi/10.1287/trsc.17.1.48)

This captures the value of backup coverage, but the independence assumption is fragile. A large incident can occupy several units together; nearby units share demand and roads; changing locations changes workloads.

**Recommendation:** use MEXCLP for inexpensive planning estimates. Estimate its busy fractions from the response simulation, and validate resulting investments through simulated dispatch.

### 2.3 Hypercube queueing: preserve spatially distinct responders

Larson’s hypercube model represents distinguishable emergency-service units, their busy/idle states, and dispatch preferences. It can estimate workloads and spatial response performance more faithfully than a single pooled queue. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/0305054874900768?utm_source=chatgpt.com)

The binary busy-state component grows as \(2^c\) for \(c\) units; at 20 units that already means 1,048,576 combinations, before richer crew states.

**TCE use:** an offline benchmark for small fleets, policy experiments, or validating approximations. Do not enumerate the full hypercube every simulation tick.

### 2.4 Queueing: staffing must cover busy time, not merely travel time

For a simplified system with incident arrival rate \(\lambda\), mean unit busy time \(E[S]\), and \(c\) interchangeable units:

\[
a=\lambda E[S], \qquad
\rho=\frac{\lambda E[S]}{c}
\]

Here, \(a\) is offered workload and \(\rho\) is average utilization.

A useful verification benchmark is an **M/M/c queue**: Poisson arrivals, exponential busy times, identical servers, and first-come-first-served dispatch. It is not a complete emergency-service model.

**Calculated example:** three units, mean busy time 45 minutes, one unit per incident.

| Utilization \(\rho\) | Probability an arrival must wait | Mean queue wait |
| --- | --- | --- |
| 0.30 | 7.0% | 1.5 min |
| 0.60 | 35.5% | 13.3 min |
| 0.80 | 64.7% | 48.5 min |
| 0.90 | 81.7% | 122.6 min |

These are Erlang-C calculations, not observed emergency-response values. They show why a system sized merely to match average demand can have disastrous delays. At \(\rho\geq1\), this unlimited-queue benchmark has no stationary finite mean wait.

TCE should instead simulate priorities, spatial dispatch, multi-unit requirements, cancellations, outside assistance, and incidents that deteriorate or become unserviceable while waiting.

### 2.5 Response-time distributions should emerge from their components

Use observed distributions to calibrate and test outputs. Sample only unresolved components.

For positive residual durations—such as treatment or equipment preparation—gamma or shifted-lognormal distributions are reasonable candidate families to test, not universal defaults. An ambulance simulation calibrated to Dutch data selected shifted lognormal distributions for scene treatment and hospital handover times. Its authors also removed the highest observations before fitting, which is a reminder that preprocessing affects the apparent tail. [Springer](https://link.springer.com/article/10.1007/s10729-026-09786-2)

Recommended rules:

* Allow a point mass at zero queue delay.
* Condition residual times on incident type, staff, equipment, and time of day.
* Preserve shared travel shocks rather than adding independent noise to every road edge.
* Do not discard long delays simply because they are inconvenient.
* Do not infer uniquely determined stage parameters from an aggregate mean response time.

---

## 3. Quantitative parameters and calibration anchors

**Confidence labels:** **H** = strong for the stated observation or controlled experiment; **M** = useful but limited by sampling or source interpretation; **L** = proposed initialization or weak transferability. Confidence in an observed number is not confidence that it applies to another society.

### 3.1 Modern empirical anchors

| Quantity | Value | Definition and context | Confidence / appropriate use |
| --- | --- | --- | --- |
| Urban EMS response | Median **6 min**; 90th percentile **12 min** | U.S. 2015 agency sample; call to scene arrival. | H for sample; M for transfer. |
| Suburban EMS response | Median **6 min**; 90th percentile **14 min** | Same study. | H for sample; M for transfer. |
| Rural EMS response | Median **13 min**; 90th percentile **26 min** | Same study. | H for sample; M for transfer. |
| England primary-fire response | Mean **9 min 25 s** | Year ending March 2026; call to first vehicle. | H for published series; not occurrence-to-action. |
| England dwelling-fire response | Mean **8 min 15 s** | Same reporting period and clock definition. | H for published series. |
| Urgent ambulance demand, Utrecht | **67,855 trips/year** for approximately **1.4 million residents** | 2021 case-study inputs. Derived rate: about **48.5 urgent trips per 1,000 person-years**. | M as a local workload anchor; not a universal incident rate. |
| Fire crew task performance | Four-person crews completed 22 tasks **30% faster than two-person crews** and **25% faster than three-person crews** | NIST residential-fire experiments, testing crews of two to five. | H for experiment; limited transfer to other equipment and tasks. |

Sources: Mell et al. (2017), Table 2, for the three U.S. rows; England’s official fire statistics for the next two; Dieleman and Jagtenberg for Utrecht; NIST for crew experiments. [Emergency Medical Services (EMS)](https://ems.emed.wisc.edu/wp-content/uploads/sites/55/2025/11/jamasurgery_mell_2017_rural_ems_responsetimes.pdf)

The U.S. response table contains 1,753,168 records but comes from a convenience sample, not a globally representative emergency system. England’s series excludes, among other cases, response times below one minute and above one hour. **Do not calibrate TCE’s extreme tail to a dataset from which extremes were deliberately removed.** [Emergency Medical Services (EMS)](https://ems.emed.wisc.edu/wp-content/uploads/sites/55/2025/11/jamasurgery_mell_2017_rural_ems_responsetimes.pdf)

Scaling the Utrecht rate mechanically to 50,000 residents gives approximately **6.6 urgent trips per day**. This is a workload-test scenario, not a recommended ancient or medieval demand rate.

### 3.2 Historical capacity anchors

| Setting | Quantitative evidence | What it supports—and does not support |
| --- | --- | --- |
| Song-era Kaifeng | Patrol posts described about **300 bu** apart, with **5 soldiers per post**. | A spatially distributed watch system. Do not convert *bu* automatically into modern meters or assume all five were continuously available. |
| Kaifeng fire lookouts | The account describes **more than 100 soldiers** housed beneath lookouts, with firefighting equipment. | Substantial organized manpower and stored equipment; not a measured simultaneous deployment strength. |
| London, 1833 | **10 private insurance-company brigades** merged into the London Fire Engine Establishment. | Institutional consolidation as a development path independent of a simple equipment upgrade. |
| Kampala training program, 2008 | A **one-day course** trained **307 lay first responders**, according to the study abstract. | A relatively small training intervention can add useful capabilities without creating a conventional ambulance fleet. |

The Kaifeng evidence is a historical narrative rather than a staffing audit: **M for institutional description, L for operational timing**. London’s institutional count is well documented. Kampala’s study supports implementation feasibility, but its follow-up and outcome limitations prevent treating training as a fixed mortality-reduction multiplier. [Project Gutenberg](https://www.gutenberg.org/cache/epub/24137/pg24137-images.html)

### 3.3 Explicitly proposed TCE initialization ranges

The following values are **engineering priors for testing**, not estimates of historical response standards. Replace them with TCE’s existing movement and work models where available.

| Input | Initial sensitivity range | How to use it |
| --- | --- | --- |
| Foot travel on a clear, approximately level route | **0.8–1.5 m/s** | Apply load, terrain, fatigue, and congestion modifiers. |
| Short messenger run | **2–3 m/s** | Limit sustained duration; do not grant indefinite running. |
| Human-pulled equipment movement | **0.5–1.2 m/s** | Require sufficient pullers and a passable route. |
| Residual preparation for a ready, on-post team | **0.5–3 min** | Exclude movement and preparation tasks already explicitly simulated. |
| Extra coordination for an on-call assembly | **1–5 min** | Add only unresolved coordination, not another allowance for members’ actual journeys. |
| Positive residual work-time variability | Coefficient of variation **0.5–1.5** | Stress-test gamma/lognormal residuals around task-derived mean durations. |
| Fleet utilization scenarios | **0.2, 0.4, 0.6, 0.8, 0.9** | Exercise queue behavior; these are not staffing targets. |

**Confidence: L for all historical transfer.** In particular, there is insufficient evidence here to prescribe a universal “medieval turnout time,” “watchmen per 1,000 residents,” or “brigade response radius.”

---

## 4. Historical and regional variation

Model variation through institutions and capabilities, not an ordered sequence that every civilization must follow.

| Setting | Evidence and institutional form | Representation in TCE |
| --- | --- | --- |
| **Foragers and small-scale mixed subsistence societies** | Research among Shiwiar forager-horticulturalists documents illness, injury, and prolonged disability in a context where care and provisioning can buffer health risks. This is ethnographic evidence, not direct observation of prehistoric dispatch. | Help can originate from nearby people and social relationships without a station. Include care, carrying, provisioning, and lost subsistence labor—not only a formal emergency mission. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1002/ajpa.10325) |
| **Early farming settlements** | The evidence reviewed does not establish general numerical response standards or a universal formal brigade. Archaeological evidence of destruction alone cannot reconstruct notification and dispatch. | Allow household assistance, communal equipment, assembly obligations, and specialized helpers as possible institutions. Let settlement layout, daytime field work, water access, and cooperation determine actual readiness. These are proposed mechanisms, not an asserted universal organization. |
| **Pre-industrial East Asia: Song Kaifeng** | Patrol posts, fire towers, equipment stores, mounted notification, and organized military response appear together in the historical account. | Separate lookout, message relay, storage, and response units. Complex emergency organization should not require industrial transport or a modern municipal-government template. [Project Gutenberg](https://www.gutenberg.org/cache/epub/24137/pg24137-images.html) |
| **Pre-industrial Ottoman Istanbul** | The fire department’s history records a 1579 order requiring household ladders and water barrels, with household participation before additional help arrived. Its tower history describes region-coded visual signals. | Combine distributed household readiness with larger responding organizations. Encode obligations, compliance, and signal interpretation. A centralized brigade need not replace household response. [İtfaiye Müdürlüğü](https://itfaiye.ibb.gov.tr/en/history-0.html) |
| **Industrializing Britain** | London’s 1833 consolidation united insurance-company brigades under a common organization, with Braidwood emphasizing training and discipline. | Institutional merger can change dispatch territories, coordination, training, and equipment sharing. Do not make institutional improvement depend solely on faster vehicles. [London Fire Brigade](https://www.london-fire.gov.uk/museum/london-fire-brigade-history-and-stories/trailblazers/james-braidwood/) |
| **Modern high-resource systems** | The U.S. response data show substantial rural–urban differences despite modern communications and vehicles. | Retain distance, fleet availability, dispatch, and final access. Modern capability should improve stages of the chain, not remove geography. [Emergency Medical Services (EMS)](https://ems.emed.wisc.edu/wp-content/uploads/sites/55/2025/11/jamasurgery_mell_2017_rural_ems_responsetimes.pdf) |
| **Modern resource-constrained urban systems: Kampala, 2008** | A program trained police, taxi drivers, and community leaders as lay first responders. Among six-month respondents, 97% reported using at least one taught skill; only 62% of the original group was followed up. | Training existing mobile people can expand first-response coverage before a dedicated fleet exists. Keep supply availability, opportunity costs, and incomplete participation explicit. Do not interpret self-reported use as a measured causal survival benefit. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0006955) |

These examples favor **combinable institutions**: household obligations, neighborhood organizations, military units, private providers, charitable responders, professional departments, and mutual-aid agreements. Their relative importance should follow TCE’s local politics, labor markets, equipment, and social relationships.

---

## 5. Stylized facts and validation targets

A correct model should reproduce the following patterns under appropriate conditions. Some are empirical targets; others are consequences the proposed mechanics should generate.

| Pattern | Evidence or expected behavior | Useful test |
| --- | --- | --- |
| **Response distributions have important tails** | In the U.S. sample, rural median response was 13 minutes versus 6 in urban areas; the corresponding 90th percentiles were 26 and 12 minutes. | Match distributions by geography, not only a citywide mean. Do not hard-code “rural = twice as slow.” [Emergency Medical Services (EMS)](https://ems.emed.wisc.edu/wp-content/uploads/sites/55/2025/11/jamasurgery_mell_2017_rural_ems_responsetimes.pdf) |
| **Availability can matter more than nominal proximity** | Queueing produces large delays at high utilization; expected-covering models explicitly value backup units. | Keep roads and station locations fixed while increasing overlapping demand. Delays and unserved incidents should rise. [PubsOnLine](https://pubsonline.informs.org/doi/10.1287/trsc.17.1.48) |
| **First arrival and effective force are different** | NIST’s crew-size experiments show that staffing changes task completion, not merely travel. | Send the same equipment with different role-complete crews; arrival may be unchanged while useful work differs. [NIST](https://www.nist.gov/news-events/news/2010/04/landmark-residential-fire-study-shows-how-crew-sizes-and-arrival-times) |
| **Coverage is direction-dependent and discontinuous at barriers** | This follows from network routing. Nearby points can have very different access across a river, gate, or impassable slope. | Remove one bridge or close one gate. The affected area should follow reachable routes, not circular distance. |
| **Nominal staffing is not simultaneous readiness** | This follows from explicit rosters, shared duties, and gathering. | Hold membership constant; change shifts, work locations, or simultaneous obligations. Operational coverage should change. |
| **More posts and more pooled capacity are not equivalent** | Decentralization can reduce travel while dividing available resources; pooling can reduce queueing while increasing travel. | Compare dispersed one-unit posts with a central multi-unit post under both light and heavy demand. |
| **Improved reporting can make recorded performance look worse** | Under separate incident/report generation, more reporting exposes demand that was previously invisible. | Introduce a signal network without adding crews. Recorded calls may rise even if true incidence is unchanged. |
| **Disasters cause joint failures** | Under shared-shock modeling, demand, staffing, communications, water, and roads may deteriorate together. | Stress-test a storm or conflagration rather than merely increasing independent incident arrivals. |

For calibration, distinguish **demand-weighted performance**, **population-weighted access**, and **harm-weighted performance**. They answer different questions and can rank investments differently.

Do not impose a universal time threshold at which every emergency suddenly becomes fatal or every building suddenly becomes unsalvageable. Let the relevant incident subsystem determine deterioration and the value of each intervention.

---

## 6. Recommended implementation, existing systems, and sources

### 6.1 A minimal but sufficient simulation structure

| Entity | Essential state |
| --- | --- |
| **Incident** | Actual location and severity; occurrence time; evolving condition; required capabilities; intervention progress; resolution. |
| **Report/message** | Sender, recipient, perceived location/severity, transmission method, creation and delivery times. |
| **Provider institution** | Jurisdiction, budget, obligations, membership, rosters, dispatch policy, facilities, mutual-aid agreements. |
| **Responder** | Actual person, location, skills, availability, fatigue, current obligation, willingness or legal duty. |
| **Equipment asset** | Location, condition, operator requirements, transportability, consumables, reservation. |
| **Mission/team** | Reserved resources, gathering plan, route, task assignments, release conditions. |

Useful mission states are:

`assembling → preparing → en_route → on_scene → transporting → handing_over → replenishing → available`

A returning unit can be available or unavailable depending on its remaining requirements. Do not assume that every unit becomes free only on reaching its original station.

**Dispatch should be triggered by events:** a report arrives, a resource becomes available, severity changes, a mission is canceled, or a route becomes unusable. Resource reservations should be committed together before movement begins.

### 6.2 Keep it economical for 10k–50k people

These are proposed engineering choices, not measured TCE benchmarks:

**Use an event queue rather than repeatedly dispatching every frame.** Store scheduled arrivals, message deliveries, shift changes, and task completions. Recalculate only when something relevant changes.

**Index potential helpers spatially.** A witness or neighborhood organizer should search nearby eligible people, not scan all citizens. Dispatch can shortlist candidates using lower-bound travel estimates, then evaluate actual routes. Expand the shortlist when needed rather than sacrificing correctness to a fixed cap.

**Reuse TCE’s routing and congestion model.** Cache estimates by movement mode and network version. Invalidate affected routes after bridge failures, gate closures, and comparable changes.

**Separate simulation precision from rendering.** Offscreen missions must preserve their actual resources, destinations, elapsed time, and scheduled arrivals. Fast-forward may abstract intermediate animation, but it must not make responders arrive sooner or become available twice.

**Compute coverage maps through sampled hypothetical incidents.** Periodically copy selected world states and run “dry” dispatch probes at representative locations. These probes must not reserve resources in the live world. Show uncertainty where samples are sparse.

Do not claim a frame-rate guarantee from population size alone. Profile incident detection, route searches, team assembly, and coverage probing separately.

### 6.3 Make institutions choose improvements

Institutions should compare feasible actions using their own information and priorities:

\[
\text{estimated investment value}
=
\frac{\text{expected reduction in weighted harm}}
{\text{annualized labor, equipment, and infrastructure cost}}
\]

The weights may reflect lives, buildings, politically important residents, member interests, or legal obligations. They need not be socially optimal.

For v1, a bounded planning heuristic is sufficient: evaluate adding staff, moving equipment, adding a post, improving a notification link, and negotiating outside assistance. Choose among affordable proposals; validate performance through subsequent experience.

A greedy covering heuristic followed by local swaps is a practical substitute for solving a large exact optimization problem every time. Keep long-term station decisions distinct from short-term dispatch.

### 6.4 What to borrow from games and operational models

| Example | Verified implementation or model feature | Lesson for TCE |
| --- | --- | --- |
| **Caesar III / Julius** | The inspected Julius source implements road-roaming prefects that switch into explicit travel-to-fire and at-fire states. Its fire handling targets burning ruins and uses abstract game timers. | Borrow visible responders and explicit mission states. Do not treat its timings or fire mechanics as a reconstruction of Roman practice. [GitHub](https://github.com/bvschaik/julius/blob/master/src/figuretype/maintenance.c) |
| **Cities: Skylines II** | The developer’s service description combines passive, road-propagated neighborhood effects with simulated vehicles and visits. Fire facilities lower nearby hazard passively; response vehicles and district assignments provide additional behavior. | Borrow road-dependent response and jurisdiction controls. Replace unexplained hazard-reduction effects with actual inspections, training, maintenance, or behavior changes. This is the published design description, not an audit of every subsequent patch. [Paradox Interactive](https://www.paradoxinteractive.com/games/cities-skylines-ii/features/city-services-districts-policies) |
| **ELASPY ambulance simulator** | An open-source discrete-event model represents ambulance dispatch, scene work, hospital handover, and return processes, with a Dutch case study. | A useful operational reference for event-based resource management. Its assumptions are not a historical model, and it does not replace TCE’s individual mobilization or information propagation. [Springer](https://link.springer.com/article/10.1007/s10729-026-09786-2) |
| **Hypercube / Desktop Hypercube applications** | Spatial queueing models have been applied to police deployment, workloads, and district design. | Useful independent checks for small modern-style fleets and patrol institutions; not a required live simulation architecture. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/0305054874900768?utm_source=chatgpt.com) |

### 6.5 Data and scholarly foundations

**Foundational planning literature**

| Source | Principal contribution |
| --- | --- |
| Toregas, Swain, ReVelle & Bergman (1971), *The Location of Emergency Service Facilities*, **Operations Research** 19:1363–1373 | Emergency-service location-set covering. [PubsOnLine](https://pubsonline.informs.org/doi/10.1287/opre.19.6.1363) |
| Church & ReVelle (1974), *The Maximal Covering Location Problem*, **Papers of the Regional Science Association** 32:101–118 | Budget-constrained demand coverage. [Springer](https://link.springer.com/article/10.1007/BF01942293) |
| Hakimi (1964), *Optimum Locations of Switching Centers and the Absolute Centers and Medians of a Graph*, **Operations Research** 12:450–459 | Graph-center and median-location foundations. [PubsOnLine](https://pubsonline.informs.org/doi/10.1287/opre.12.3.450) |
| Daskin (1983), *A Maximum Expected Covering Location Model*, **Transportation Science** 17:48–70 | Approximate coverage with busy units. [PubsOnLine](https://pubsonline.informs.org/doi/10.1287/trsc.17.1.48) |
| Larson (1974), *A Hypercube Queuing Model for Facility Location and Redistricting in Urban Emergency Services*, **Computers & Operations Research** 1:67–95 | Spatially distinguishable emergency-service queues. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/0305054874900768?utm_source=chatgpt.com) |

**Useful calibration datasets and records**

| Source | Best use | Important limitation |
| --- | --- | --- |
| **London Fire Brigade Mobilisation Records** | Individual pumping-appliance mobilizations from January 2009 onward; timing and originating-location analysis. | Do not assume mobilization records alone describe complete crew availability, all scene tasks, or every release condition. [Data London](https://data.london.gov.uk/dataset/london-fire-brigade-mobilisation-records-24r65) |
| **England FIRE1001 and associated fire statistics** | Aggregate response stages and comparisons by incident category. | Publication definitions and exclusions must be replicated when validating against published summaries. [GOV.UK](https://www.gov.uk/government/statistics/fire-and-rescue-incident-statistics-year-ending-march-2026/fire-and-rescue-incident-statistics-year-ending-march-2026--2) |
| **NEMSIS public-release research datasets** | Modern EMS case mix and response-process research, subject to release-specific access and field availability. | Reporting coverage, missingness, and selection require examination; records are not automatically a census of all emergencies. [NEMSIS](https://nemsis.org/) |
| **London Archives fire-brigade records** | Historical organizational development, staffing, stations, and incident documentation. | Historical recordkeeping is not equivalent to a modern synchronized dispatch log. [The London Archives](https://www.thelondonarchives.org/your-research/research-guides/fire-brigade-records) |

### 6.6 Evidence gaps and acceptance tests

The weakest empirical area is **historical operational timing**: detection delays, volunteer gathering distributions, on-duty fractions, scene-work rates, and unreported incidents. The sources here support several institutional mechanisms much better than they support universal numerical coefficients.

Also distinguish observed associations from intervention effects. Severe incidents may receive faster dispatch yet still have worse outcomes. Comparing raw response time with mortality or property loss does not by itself identify the benefit of faster response.

Before accepting the subsystem, require:

| Test | Required behavior |
| --- | --- |
| **Causality** | No institutional response before the institution receives information, unless its own responder directly observes the incident. |
| **Resource conservation** | No person, animal, vehicle, or equipment asset is assigned incompatibly to simultaneous missions. |
| **Travel consistency** | Closed routes and actual origins affect arrival; offscreen simulation produces the same causal timings. |
| **Queue verification** | Simplified configurations reproduce analytic queue benchmarks. |
| **Capability verification** | Arrival without required roles or equipment does not complete the corresponding task. |
| **Coverage accounting** | Unserved and late incidents remain visible; conditional averages cannot conceal them. |
| **Counterfactual validation** | With matched incident seeds, compare staffing, stations, communications, roads, and mutual aid using both response distributions and incident outcomes. |

**Recommended v1:** explicit incidents, local detection and messages, rostered capability bundles, priority dispatch from actual positions, shared routing, task-constrained intervention, and measured coverage distributions. Use covering and median models to help institutions decide where to invest—not to make help appear.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9293c-fc10-83e9-8bf5-8cabbda7c007)
