# Military organization and logistics for The Civilization Engine

## Executive recommendation

**Model an army as an institution that reallocates existing people, equipment, animals, and transport capacity—not as a population percentage converted into combat units.** Recruitment determines who owes or accepts service; households and workshops determine what they can bring; supply networks determine where they can remain; contracts and political authority determine how long they stay.

Keep five quantities separate:

| Quantity | Meaning in TCE |
| --- | --- |
| **Service-liable population** | People whom institutions may call upon, whether or not they are currently serving. |
| **Mustered strength** | Actual people who have reported, including those awaiting equipment or training. |
| **Deployable strength** | Personnel sufficiently equipped, trained, and healthy to leave their present duties. |
| **Field combatants** | Personnel assigned to fighting roles in a particular force. |
| **Total supported population** | Combatants plus drivers, porters, craftspeople, medical workers, servants, dependents, and other accompanying people. |

These distinctions matter historically: surviving figures can describe establishments, recruitment registers, garrisons, or battlefield forces rather than interchangeable measures of “army size.” Military labor also extended far beyond combatants, particularly in transport-intensive campaigns. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/why-military-institutions-matter-for-chinese-history-circa-6001800/2C7D4281853676920863FA46801BB0E0)

The recommendations below distinguish **documented quantities**, **historical reconstructions**, and **proposed simulation priors**. The last category supplies starting values for experimentation, not claims about universal historical behavior.

---

# 1. Mechanisms: implementable causal rules

## 1.1 Recruitment systems should be combinations of rules

Do not implement a progression in which levies automatically become militias, then mercenaries, then professionals. Instead, compose military institutions from eligibility, obligation, compensation, equipment ownership, training, command, and discharge rules. Historical systems combined these dimensions: Roman soldiers could have equipment costs deducted from pay; English expeditionary contracts specified numbers, wages, service periods, and divisions of proceeds; Qing compensation distinguished service categories and campaign allowances. [The Latin Library](https://www.thelatinlibrary.com/historians/polyb/polybius5.html)

| Institutional arrangement | Implementable recruitment and maintenance rules | Principal constraint to represent |
| --- | --- | --- |
| **Levy** | Assign obligations to persons, households, villages, estates, or other corporate groups. Permit exemptions, substitutes, geographic limits, and maximum service periods. | Compliance and the household’s ability to lose labor. |
| **Militia** | Maintain a register and recurring training obligations. Members normally retain civilian occupations; equipment may be privately owned or stored communally. | Readiness requires actual training time and maintained equipment. |
| **Professional force** | Military service is a continuing occupation supported by pay, rations, housing, land income, or combinations of these. | Recurring revenue and the opportunity cost of maintaining personnel outside civilian production. |
| **Mercenary company** | Contract with an existing organization containing identifiable people, officers, equipment, liabilities, and expectations of payment. | Availability, contract credibility, recruitment networks, and competing employers. |
| **Retainers or military households** | Tie service to a patron, hereditary status, land entitlement, household registration, or revenue assignment. | Whether assigned resources still support the required service, and to whom personnel are loyal. |

These are **overlapping attributes**, not mutually exclusive unit types. A professional soldier may be conscripted; a militia may receive campaign wages; a mercenary company may include inexperienced recruits.

For TCE, a service obligation should contain at least:

`issuer, liable_entity, eligibility_rule, required_role, equipment_provider, compensation, start_condition, duration, geographic_scope, exemptions, discharge_conditions`.

The distinction between potential manpower, a remunerated standing force, and the largest fielded army is also explicitly recognized in Seshat’s military coding framework. [Seshat DB](https://www.seshat-db.com/codebook?utm_source=chatgpt.com)

## 1.2 Mobilization is a process, not a switch

**Recommended state sequence:**

`notified → preparing → traveling to muster → inspected → training/organizing → assigned → deployed → discharged → returned`

At every stage, time, resources, and compliance can be limiting.

A person’s response should depend on perceived compensation, coercion, communal expectations, danger, household food security, and trust in the organizer. Decentralized recruitment need not mean unorganized recruitment: research on Turkana pastoralist warfare finds cooperation supported by community sanctions without a centralized state commanding the force. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3136302/)

Implement the following:

**Household feasibility.** Before departure, assess whether dependents can obtain food and whether critical tasks have substitutes. Institutions may override this constraint, but the resulting hardship remains in the simulation.

**Physical muster.** Count present people and usable equipment separately from promised quotas. A register is a claim about resources, not the resources themselves.

**Collective readiness.** Individual weapon skill, experience with particular comrades, and formation training should be different states. Replacements may restore headcount without immediately restoring cohesion.

**Command relationships.** Assign units to actual commanders with appointments and loyalties. Orders should travel through available communication systems; distant commanders should act on imperfect information.

**Finite service.** Expiring obligations, unpaid compensation, deteriorating household conditions, and campaign losses should affect willingness to remain. They should not cause a whole force to vanish automatically on a single date.

## 1.3 Equipment and pay require two separate accounts

Maintain both:

\[
\text{Fiscal expenditure}
=
\text{pay}+\text{procurement}+\text{transport payments}
+\text{allowances}+\text{debt service}
\]

and:

\[
\text{Real resource burden}
=
\text{labor diverted}+\text{goods consumed}
+\text{capital lost}+\text{productive opportunities forgone}
\]

These are related but **must not be added together indiscriminately**. Paying a soldier transfers purchasing power; feeding that soldier consumes goods. Counting both the wage and everything purchased with it as separate resource destruction would double-count costs.

Recommended rules:

* Equipment is a durable asset with ownership, condition, compatibility, and repair requirements.
* Procurement places orders on real workshops; money cannot bypass material shortages or production time.
* Self-equipping soldiers shift the initial burden to households rather than eliminating it.
* Horses, carts, clothing, footwear, and replacement parts have separate lifecycles.
* Arrears are outstanding claims, not instantly deleted expenses.
* Captured and returned equipment can be repaired, reissued, sold, or retained according to institutional rules.

For cross-world comparisons, report equipment costs in **local labor-days**, and military compensation relative to **local civilian earnings or subsistence costs**. Do not convert ancient and modern nominal currencies into a supposedly universal “gold cost.”

## 1.4 Supply must conserve goods and include support personnel

For commodity \(k\) at location \(n\):

\[
S\_{n,k,t+1}
=
S\_{n,k,t}
+\text{arrivals}
+\text{production}
-\text{consumption}
-\text{departures}
-\text{losses}.
\]

Ownership changes, such as purchases or requisitions, should transfer existing stocks rather than create supplies.

At minimum, distinguish staple food, water, animal feed, equipment replacements, and consumable military supplies. Later technologies add their own fuel, ammunition, maintenance, and spare-part demands.

Food also has a **preparation state**. Grain, flour, bread, and ready-to-eat rations should not be interchangeable inventory labels. Roth’s reconstruction of Roman provisioning emphasizes the time and equipment required to turn issued grain into meals, often at small-group level. [StudyRes](https://studyres.com/doc/806491/the-logistics-of-the-roman-army-at-war--264-b.c.)

For local procurement:

\[
Q\_{\text{acquired}}
=
\min
\left(
Q\_{\text{accessible}},
Q\_{\text{collectable with available labor}},
Q\_{\text{transportable}}
\right).
\]

Separate purchase, taxation in kind, compulsory requisition, and plunder. Each changes ownership, civilian welfare, and political relations differently.

**Foraging must draw from something real.** Use household stores, livestock, crops, and pasture biomass. Harvested food is finite; seed grain has a future production role; pasture requires time and regrowth. Two armies cannot independently extract the same supplies.

Merchant transport should be a valid alternative to government-owned convoys. Research on Mughal campaigning highlights the importance of Banjara carriers and merchants in provisioning armies. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/22308075231201909)

## 1.5 Transport capacity depends on the whole delivery cycle

Represent roads, paths, rivers, sea routes, bridges, ports, and—when available—railways and motor roads as a **seasonal, capacity-constrained network**.

A carrier’s useful output depends on payload, travel time, loading and unloading, its own consumption, and the return journey. For a simplified self-supplied shuttle:

\[
\bar q
=
V\frac{\max(0,P-cT)}{T},
\]

where \(V\) is the number of carriers, \(P\) their individual payload, \(T\) the round-trip cycle in days, and \(c\) the daily mass of supplies each carrier must carry for its own operation.

This is a bookkeeping approximation, not a universal transport law: local grazing, intermediate purchases, return cargo, and different transport modes change the calculation.

Marching should likewise use a daily time budget:

\[
24\text{ h}
=
\text{movement}+\text{rest}+\text{feeding}
+\text{loading}+\text{camp work}+\text{delays}.
\]

The important speed is **whole-force progress**, not the walking speed of the first person. Columns, baggage, crossings, and unloading create queues. ORBIS provides a useful precedent for making travel depend on network, mode, season, time, and cost rather than straight-line distance. [Stanford History Department](https://history.stanford.edu/publications/orbis-stanford-geospatial-network-model-roman-world)

## 1.6 Mobilization should affect specific economic tasks

Avoid a universal modifier such as “mobilization reduces production by 20%.”

Instead, remove actual work hours from actual tasks. The departure of a farmer during a slack period is different from departure during sowing or harvesting. The loss of a specialist, seed stock, or draft animal may constrain later production even after surviving soldiers return.

Ugaritic recruitment records illustrate the connection between military demands and village populations; Vidal also discusses evidence of attacks on grain and agricultural production. These are useful mechanisms, but the surviving material does not establish a universal output-loss coefficient. [Academia](https://www.academia.edu/30809644/Military_conscription_in_Ugarit)

Recommended economic effects include:

**Substitution:** other household members, hired labor, neighboring households, or institutional work parties may take over tasks—but they already have occupations and time constraints.

**Sectoral bottlenecks:** recruiting the only qualified smith or wagon builder can reduce military supply itself.

**Capital depletion:** taking agricultural animals or consuming seed grain affects subsequent seasons.

**Distributional effects:** suppliers and transport workers can gain while taxed households lose. Qing campaigns sometimes expanded commercial activity and transport connections, demonstrating that military demand need not harm every locality or occupation equally. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/qing-military-institutions-and-their-effects-on-government-economy-and-society-16401800/E1FDAA965C5B67C12274B2B3408AE206)

**Demobilization:** return travel, injuries, missing equipment, accumulated debt, and altered household circumstances should delay complete economic recovery.

---

# 2. Parameters: evidence, units, and confidence

**Confidence notation:** **H** = strong evidence for the stated case or specification; **M** = scholarly reconstruction or context-sensitive estimate; **L** = particularly uncertain denominator, coverage, or generalization. Confidence in a historical case does not imply confidence in transferring its value to another society.

## 2.1 Military participation: useful anchors, not universal ceilings

Percentages below use **total population**, except where explicitly stated.

| Historical case | Quantity and approximate share | Interpretation and confidence |
| --- | --- | --- |
| **Late Bronze Age Ugarit** | A general recruitment register records **955+ men**, probably around **1,000**; against a reconstructed **11,000–14,000 rural inhabitants**, approximately **7–9%**. | **M/L.** Rural denominator excludes the capital; register is not proof of simultaneous field deployment. Derived from Vidal’s reconstruction. [Academia](https://www.academia.edu/30809644/Military_conscription_in_Ugarit) |
| **Tang China** | Reconstructed soldier/population ratios **1:117–1:70**, or **0.85–1.43%**. | **M/L.** Aggregate estimates, not one directly observed field army. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/why-military-institutions-matter-for-chinese-history-circa-6001800/2C7D4281853676920863FA46801BB0E0) |
| **Qing China, eighteenth century** | **800,000 soldiers / 300 million inhabitants ≈ 0.27%**. | **L for precise comparison.** Robinson explicitly presents this as a back-of-the-envelope calculation; force and population estimates are not a tightly matched census pair. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/why-military-institutions-matter-for-chinese-history-circa-6001800/2C7D4281853676920863FA46801BB0E0) |
| **France, late Louis XIV** | Approximately **1:50 = 2%**. | **M/L.** Comparative historical estimate. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/why-military-institutions-matter-for-chinese-history-circa-6001800/2C7D4281853676920863FA46801BB0E0) |
| **Prussia, 1740** | Approximately **1:27 = 3.7%**. | **M/L.** Illustrates a much more militarized establishment, not a preindustrial biological limit. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/why-military-institutions-matter-for-chinese-history-circa-6001800/2C7D4281853676920863FA46801BB0E0) |
| **United States, 1939** | **334,473 military personnel / 130.88 million population ≈ 0.26%**. | **H for counts; M for exact date alignment.** All services, not field combatants. [National WWII Museum](https://www.nationalww2museum.org/students-teachers/student-resources/research-starters/research-starters-us-military-numbers) |
| **United States, 1945** | **12.21 million personnel / 139.93 million population ≈ 8.7%**. | **H/M.** Wartime service strength, not cumulative enlistments or combat strength. [National WWII Museum](https://www.nationalww2museum.org/students-teachers/student-resources/research-starters/research-starters-us-military-numbers) |

Military budget shares have a different denominator again. Robinson reports preliminary estimates around **70% of government expenditure** for several Chinese dynasties. That is **not 70% of economic output**. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/why-military-institutions-matter-for-chinese-history-circa-6001800/2C7D4281853676920863FA46801BB0E0)

**TCE implication:** treat population-share values as validation observations. Recruitment and supply processes should produce them; they should not directly generate soldiers.

## 2.2 Pay, equipment, and transport costs

| Case | Documented value | What the value does—and does not—measure |
| --- | --- | --- |
| **Roman Republic, Polybius 6.39** | Infantry **2 obols/day**; centurion twice that; cavalry **1 drachma/day**. | **H as a textual schedule, M as realized compensation.** Food, clothing, and additional arms could be deducted from Roman soldiers’ pay. [The Latin Library](https://www.thelatinlibrary.com/historians/polyb/polybius5.html) |
| **English northern-France expedition, 1415** | Archer **6 pence/day**; man-at-arms **12 pence/day**, with additional arrangements for some categories. | **H.** Contract wages, not total employer cost. Contracts specified **12 months** of service. [Agincourt 600](https://www.agincourt600.com/2015/03/24/27-april-3-may-1415-further-military-preparations-and-indentures/?utm_source=chatgpt.com) |
| **English procurement, 1415** | **£25 for 1,000 lance shafts**: calculated **6 pence per shaft**, equal to one archer’s daily wage. | **H.** Wooden shafts **without iron heads**, not complete weapons or equipment sets. [Agincourt 600](https://www.agincourt600.com/2015/03/24/27-april-3-may-1415-further-military-preparations-and-indentures/?utm_source=chatgpt.com) |
| **Qing regular cash compensation** | Ordinary bannerman **4 taels/month**; Green Standard soldier **1–2 taels/month**. | **H/M.** Cash schedules, not complete household compensation. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/qing-military-institutions-and-their-effects-on-government-economy-and-society-16401800/E1FDAA965C5B67C12274B2B3408AE206) |
| **Qing campaign supplement** | Green Standard troops received another **1–2 taels/month**, while regular stipends continued to families. | **H/M.** Campaign service could approximately double the cash component. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/qing-military-institutions-and-their-effects-on-government-economy-and-society-16401800/E1FDAA965C5B67C12274B2B3408AE206) |
| **Qing equipment allowance, late eighteenth century** | **6–10 taels** for Green Standard troops; **20 taels** for Manchu bannermen. | **H/M.** Deployment-readiness subsidy, **not the price of an entirely new kit**. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/qing-military-institutions-and-their-effects-on-government-economy-and-society-16401800/E1FDAA965C5B67C12274B2B3408AE206) |
| **White Lotus campaign, 1796–1805** | Mule hire reportedly reached **12 taels/mule/month**. | **M.** Scarcity and administrative manipulation; roughly an infantryman’s annual basic cash stipend, not a normal rental tariff. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/qing-military-institutions-and-their-effects-on-government-economy-and-society-16401800/E1FDAA965C5B67C12274B2B3408AE206) |
| **U.S. Civil War rifle-muskets** | Approximately **$12–18** at Springfield Armory versus **$18–20** from private production in the cited comparison. | **M.** Historical production-cost comparisons are imperfect; excludes full personal equipment and upkeep. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/17416124.2017.1293882) |
| **New Mexico Volunteers, U.S. Civil War** | Basic pay **$13/month**, with a **three-year** enlistment and separate bounty arrangements. | **H for this documented case.** Pay promised was not necessarily pay received on time. [National Park Service](https://home.nps.gov/foun/learn/historyculture/new-mexico-volunteers.htm) |

The industrial comparison puts one rifle-musket at roughly **0.9–1.5 months of the cited basic pay**. This is an illustrative calculation, not a full soldier-cost estimate.

For TCE, the useful lesson is that **initial equipment, recurring compensation, and transport scarcity can have very different magnitudes**. A campaign should not be priced merely as “number of soldiers × weapon cost.”

## 2.3 Food, animals, movement, and service duration

| Parameter | Evidence-based anchor | Confidence and implementation caution |
| --- | --- | --- |
| **Roman grain ration** | Roth reconstructs approximately **0.85 kg wheat/person/day**. | **M.** Conversion from ancient volume measures; other reconstructions differ. Grain mass is not bread mass. [StudyRes](https://studyres.com/doc/806491/the-logistics-of-the-roman-army-at-war--264-b.c.) |
| **Roman total ration** | Approximately **1.1–1.33 kg/person/day** in Roth’s detailed reconstruction. | **M.** Reconstructed food bundle, not an all-army universal and not additional drinking water. [StudyRes](https://studyres.com/doc/806491/the-logistics-of-the-roman-army-at-war--264-b.c.) |
| **Modern U.S. MRE reference** | Approximately **1,250 kcal/meal**; three meals give **3,750 kcal/day**. A nominal 22-lb case of 12 implies approximately **2.5 kg/day** of packaged rations. | **H as a product/planning reference.** Not an invariant physiological requirement; excludes additional drinking water. [Defense Logistics Agency](https://www.dla.mil/Troop-Support/Subsistence/Operational-rations/mre/) |
| **Horse feed intake** | Approximately **1.5–3% of body mass/day as dry matter**. | **H for the broad husbandry range; M in historical application.** Workload, feed quality, body size, and condition matter. [Extension Horses](https://horses.extension.org/common-feeding-programs-for-horses/) |
| **Horse drinking water** | A **454-kg horse** commonly requires about **38–45 L/day**, with higher demands in heat or activity. | **H/M.** A baseline, not an upper limit. [University of Minnesota Extension](https://extension.umn.edu/agriculture/animals-and-livestock/horse/ten-things-you-should-know-about-feeding-the-mature-horse) |
| **Grazing time** | Guidance ranges from **6–10 hours on adequate pasture** to approximately **11–15 hours** under other intake assumptions. | **M.** Do not collapse these into a guaranteed feeding rate; pasture quality is decisive. [Utah State University Extension](https://extension.usu.edu/equine/research/equine-nutrition-forages) |
| **Feed moisture** | Hay approximately **90% dry matter**; fresh pasture often **20–30%**. | **H/M.** Store dry matter and transported mass separately. [Utah State University Extension](https://extension.usu.edu/equine/research/equine-nutrition-forages?utm_source=chatgpt.com) |
| **Pack-mule cargo, medieval Aragon** | A common cited load was **200 lb ≈ 91 kg**. | **M.** Case-specific payload, not a universal maximum for every animal and terrain. [deremilitari.org](https://deremilitari.org/2014/05/the-town-in-service-of-war-in-the-medieval-crown-of-aragon/) |
| **Trained foot-march reference** | U.S. Army FM 21-18, 1966: approximately **32 km in eight hours**, or **4 km/h**. | **H as doctrine.** Not an observed mean for mixed historical armies over entire campaigns. [Wayback Machine](https://dn710301.ca.archive.org/0/items/FM21-181966/FM21-181966.pdf) |
| **Self-provisioning obligation, medieval Aragon** | Examples ranged from **3 days to 3 months**, after which royal provisioning obligations could apply. | **M.** Demonstrates institutional variation; not a universal “feudal service period.” [deremilitari.org](https://deremilitari.org/2014/05/the-town-in-service-of-war-in-the-medieval-crown-of-aragon/) |

For animals other than horses, use species-specific physiology, working load, and environmental tolerance rather than copying horse coefficients.

For motorized and industrial forces, do **not** assign a universal ammunition or fuel demand per soldier. Calculate these from equipment inventories, operating hours, movement, training, and the activities being simulated.

## 2.4 Suggested initial calibration values

These are **proposed test settings**, not historical estimates or enforced caps.

| Variable | Suggested initial envelope | Use |
| --- | --- | --- |
| Permanent on-duty personnel | **0.2–1% of total population** | A low-to-moderate agrarian starting configuration; institutions may sustain much more or none. |
| Sustained mobilized personnel, including assigned support | **1–5% of total population** | Test against household production, finances, and route capacity. |
| Brief emergency local mobilization | **5–15% of total population** | Stress test for days or weeks, not an automatically sustainable expedition. |
| Mixed foot-and-baggage column | **15–25 km per actual travel day** | Initial movement prior; reduce through terrain, queues, weather, fatigue, and daily work. |
| Personal food reserve | **3–10 days** | Must occupy real payload and decline through consumption. |

For a **50,000-person polity**, the first two envelopes correspond to **100–500 permanent personnel** and **500–2,500 mobilized personnel**, respectively. These are calculated scenario sizes. They should emerge only when the simulated institutions and economy can support them.

Do not assign numerical historical confidence to these priors. Validate and revise them against selected cases.

---

# 3. Variation across eras and regions

The following are alternative institutional and ecological configurations, **not mandatory developmental stages**.

| Setting | Historically important distinction | TCE representation |
| --- | --- | --- |
| **Mobile foragers** | A study of 21 societies emphasizes variation in lethal aggression and distinguishes interpersonal killings from war. It does not provide a universal army/population ratio or a direct census of prehistoric armies. | Use actual participants, social obligations, travel, and household subsistence. Do not infer a fixed standing force or universal war-party size. [PubMed](https://pubmed.ncbi.nlm.nih.gov/23869015/) |
| **Early farming communities** | Comparable quantitative army records are thin in the evidence assembled here. Later Bronze Age village recruitment cannot simply be projected backward into the Neolithic. | Make household labor, food storage, collective obligation, and distance primary. Treat Ugarit as a later example of administratively organized village recruitment, not a universal early-farming template. [Academia](https://www.academia.edu/30809644/Military_conscription_in_Ugarit) |
| **Ancient Mediterranean and Near Eastern states** | Citizen obligations, allied contingents, state distributions, deductions from pay, and differentiated equipment arrangements could coexist. | Permit multiple service and supply contracts inside one army. A force need not have one uniform employer–employee relationship. [The Latin Library](https://www.thelatinlibrary.com/historians/polyb/polybius5.html) |
| **Preindustrial European kingdoms and towns** | Municipal resources, hired carriers, royal funding, and contractual contingents could interact. Service duration and responsibility for provisioning varied. | Give towns and corporate bodies their own quotas, treasuries, stores, and transport contracts. [deremilitari.org](https://deremilitari.org/2014/05/the-town-in-service-of-war-in-the-medieval-crown-of-aragon/) |
| **Chinese imperial systems** | Qing garrison compensation and campaign supplements connected deployed soldiers to families remaining at home. | Separate garrison maintenance, deployment expenditure, and household transfers rather than treating mobilization as a single wage multiplier. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/qing-military-institutions-and-their-effects-on-government-economy-and-society-16401800/E1FDAA965C5B67C12274B2B3408AE206) |
| **South Asia** | Mughal warfare depended on extensive noncombatant labor and commercial provisioning; environmental conditions shaped operations. | Model carriers, markets, camp work, and regional weather. A universal European-style “summer campaigning season” is inappropriate. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/0968344520918615) |
| **Andean states** | Inka research distinguishes staple finance from wealth finance and documents storage as part of political-economic organization. | Allow institutions to mobilize labor and stored goods without requiring a cash treasury. Stocks and obligations can finance armies directly. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/203249) |
| **African pastoral societies and Inner Asian settings** | Turkana force raids in one study averaged **315 warriors**, median **248**, despite decentralized political organization. Comparative Eurasian research emphasizes the interaction of state mobilization and different ecological resource bases. | Separate the capacity to coordinate people from bureaucratic centralization. For mounted forces, track herds, remounts, pasture, and animal time budgets. Turkana are pastoralists, not a proxy for all foragers or Africa. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3136302/) |
| **Industrial and modern systems** | Mass armaments production can greatly expand equipment availability; modern service populations can change dramatically between peace and major war. | Extend existing production and transport systems rather than replacing them with abstract manpower. Add training establishments, reserves, specialized maintenance, and equipment-specific consumables. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/17416124.2017.1293882?utm_source=chatgpt.com) |

**Campaign seasons should emerge from intersecting calendars.** In TCE, evaluate crop tasks, food availability, pasture, rainfall, road conditions, river transport, temperature, and service obligations independently. A season favorable for marching may be unfavorable for grazing or household labor. Nath’s environmental approach to Mughal warfare is particularly useful for avoiding a single global campaign calendar. [OUP Academic](https://academic.oup.com/book/32277)

---

# 4. Stylized facts and quantitative validation

## 4.1 Worked logistics example

Consider a **synthetic TCE force**, not a reconstructed historical army:

* **1,000 combatants**
* **200 accompanying support personnel**
* **300 equids**

Assume **1.2 kg of transported food per person/day**, **10 kg of dry-matter feed per animal/day**, and transported fodder at **90% dry matter**. For this scenario, assign **6 L/person/day** for drinking and cooking and **40 L/animal/day**. The human water figure is a scenario input, not a universal requirement.

| Daily requirement | Calculation | Result |
| --- | --- | --- |
| Human food | \(1,200\times1.2\) kg | **1.44 t/day** |
| Animal feed | \(300\times10\) kg dry matter | **3.00 t dry matter/day** |
| Transported fodder without grazing | \(3.00/0.90\) | **3.33 t/day** |
| Combined food and fodder without grazing | \(1.44+3.33\) | **4.77 t/day** |
| Combined food and fodder with 60% of animal dry matter grazed | \(1.44+(3.00\times0.40/0.90)\) | **2.77 t/day** |
| Water under the scenario assumptions | \(1,200\times6+300\times40\) L | **19.2 m³/day** |

Ten days of food and fodder alone therefore require about **47.7 tonnes without grazing**, or **27.7 tonnes with the specified grazing contribution**.

These calculations exclude weapons, clothing, cooking fuel, losses, and any additional rear-area carriers. Extra transport animals and drivers must themselves be added to the consumption ledger.

Three consequences follow directly:

**Animals can dominate food-transport demand.** Cavalry and baggage animals are not merely movement bonuses.

**Grazing saves cargo, not resources.** It consumes pasture and feeding time.

**Local water availability can matter more than stored-food endurance.** In this example, hauling all daily water would require much more mass than hauling food.

## 4.2 Patterns a correct simulation should reproduce

| Pattern | Test in TCE |
| --- | --- |
| **Similar populations can maintain very different forces.** | Hold population constant and vary revenue extraction, obligations, training, equipment stocks, and transport institutions. Force size should change without changing a biological “military fraction.” |
| **A recruit is not immediately an additional effective soldier.** | Add poorly equipped recruits to a supply-constrained force. Headcount rises, but deployable strength may not; ration availability per person can decline. |
| **Local defense and distant campaigning have different participation limits.** | Compare short service near households with prolonged service requiring travel and a support train. Measure household task losses and total supported mouths separately. |
| **Concentration eventually creates supply problems.** | Merge two forces using the same depot and roads. Demand doubles, but throughput should increase only when actual transport capacity does. |
| **Repeated passage depletes a region.** | Send successive forces through the same farming district. Later procurement must encounter the depleted inventories left by earlier forces. |
| **Transport improvements change feasible operations.** | Add a port, bridge, navigable connection, or reliable carrier service. Benefits should propagate through travel time and delivered quantities, not a generic combat bonus. |
| **Mobilization has uneven and delayed economic costs.** | Recruit different occupations in different seasons. Compare immediate output, subsequent harvests, household consumption, and recovery after discharge. |

These are primarily **mechanism tests derived from the proposed model**. Historical validation should then compare selected outputs with the documented cases above rather than demanding one global average.

---

# 5. Recommended representation for 10k–50k agents

## 5.1 Preserve individual identity; aggregate coordination

A soldier should remain the same person throughout recruitment, service, injury, capture, desertion, and return. Military organization changes assignments and obligations, not identity.

| Simulation object | Minimum useful state |
| --- | --- |
| **Person** | Household, occupation, skills, location, health, fatigue, service status, contract, equipment ownership. |
| **Household** | Dependents, available labor, food stocks, debts, military obligations, remittances, missing members. |
| **Unit** | Roster of person IDs, commander, collective training, orders, destination, formation state, shared supplies. |
| **Military institution** | Eligibility laws, quotas, exemptions, compensation rules, command appointments, procurement authority, treasury and arrears. |
| **Depot or distribution point** | Commodity stocks, ownership, reserved quantities, handling capacity, storage losses. |
| **Convoy** | Drivers, animals or vehicles, cargo, route, departure time, arrival estimate, capacity, operating consumption. |
| **Transport edge** | Mode, travel time, throughput, seasonal accessibility, queue, tolls, permitted users. |

Represent unit requirements as **recipes of capabilities and resources**: people with suitable training, serviceable equipment, transport, and consumables. Do not make availability a technology unlock alone.

Procurement and command decisions should use the institution’s **known or estimated** information. The underlying simulation may know all inventories, but a ruler should not automatically know that a distant convoy has been delayed.

## 5.2 Use multiple update scales

For performance, separate decision frequency from rendering frequency.

**Movement and logistics:** use departure, arrival, queue, and route-change events. People traveling in one column can share a route while retaining individual identities and positions within it.

**Consumption:** aggregate allocations at the unit or mess level, then update individual nutrition and health at the appropriate physiological interval.

**Production and household work:** retain the civilian economy’s existing task scheduler. Military assignment removes or redirects work availability.

**Pay and institutions:** process pay periods, contractual deadlines, muster reviews, and policy changes as events rather than per-frame checks.

**Rendering:** Unreal can display people, animals, carts, and camp activities from these authoritative states without running an independent economy.

Cache routes until relevant conditions change. Recompute after closures, congestion changes, seasonal transitions, or revised destinations—not for every soldier every frame.

These are architectural recommendations, **not a measured performance claim for TCE’s Rust kernel**. Benchmark crowded crossings, synchronized mobilizations, and many simultaneous convoys, not just average marching conditions.

## 5.3 Simplifications worth making

Aggregate interchangeable sacks and commodity batches, not individual grains. Use shared kitchen and camp-work tasks rather than making each soldier independently search for every input. Represent ordinary camp layouts and march formations parametrically.

For an initial implementation, prioritize:

**Actual rosters, service contracts, household labor removal, equipment inventories, food and fodder, route capacity, and delayed delivery.**

Add detailed dietary variety, elaborate rank systems, individual procurement fraud, and sophisticated military finance only when they explain behavior that the simpler model cannot reproduce.

Avoid shortcuts that undermine the central design: unlimited mercenary spawning, supplies appearing because territory is friendly, free animal feeding, immediate discharge teleportation, or fixed attrition unrelated to conditions.

## 5.4 Existing models and games to borrow from

| Reference | Useful idea | Boundary for TCE |
| --- | --- | --- |
| **ORBIS, Stanford** | Mode-, season-, time-, and cost-sensitive transport over a historical network. | Borrow the network architecture, not Roman transport parameters as universal constants. It is not a citizen-level army simulator. [Stanford History Department](https://history.stanford.edu/publications/orbis-stanford-geospatial-network-model-roman-world) |
| **Manor Lords** | Developer documentation explicitly connects militia service and casualties to the settlement’s real population and economy. | Useful precedent for visible opportunity costs and persistent people; independently implement the depth of logistics TCE needs. [Steam Store](https://store.steampowered.com/app/1363080/Manor_Lords/) |
| **Songs of Syx** | Citizen training, equipment, army supply depots, and delivery connect domestic production to deployed forces. | Useful systems reference; its balancing values are not historical measurements. Verify mechanics against the version being studied. [Songs of Syx](https://songsofsyx.com/wiki/index.php/Army) |

---

# 6. Sources, datasets, and limits of the evidence

## Core scholarly reading

**Jonathan P. Roth, *The Logistics of the Roman Army at War, 264 B.C.–A.D. 235* (1999).** Foundational for rations, preparation, animals, transport, and supply administration. Its numerical reconstructions should remain reconstructions rather than universal ancient constants. [StudyRes](https://studyres.com/doc/806491/the-logistics-of-the-roman-army-at-war--264-b.c.)

**Jordi Vidal, “Military Conscription in Ugarit” (2016).** Particularly useful for connecting recruitment registers, village geography, and reconstructed population. [Academia](https://www.academia.edu/30809644/Military_conscription_in_Ugarit)

**Sarah Mathew and Robert Boyd, “Punishment Sustains Large-Scale Cooperation in Prestate Warfare” (2011).** Original research on decentralized recruitment and cooperation among Turkana pastoralists; DOI **10.1073/pnas.1105604108**. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3136302/)

**David Robinson, “Why Military Institutions Matter for Chinese History circa 600–1800” (2017),** and **Yingcong Dai, “Qing Military Institutions and Their Effects on Government, Economy, and Society, 1640–1800” (2017).** Institutional comparisons, compensation, and fiscal context. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/why-military-institutions-matter-for-chinese-history-circa-6001800/2C7D4281853676920863FA46801BB0E0)

**Pratyay Nath, *Climate of Conquest* (2019), and “What Is Military Labour?” (2021).** Essential correctives to army models centered exclusively on fighters and European environments. [OUP Academic](https://academic.oup.com/book/32277)

**Terence D’Altroy and Timothy Earle, “Staple Finance, Wealth Finance, and Storage in the Inka Political Economy” (1985).** A foundation for modeling organized military provisioning without assuming monetary taxation; DOI **10.1086/203249**. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/203249)

**Donald Kagay, “The Town in Service of War in the Medieval Crown of Aragon” (1997),** and **Michael Raber, “Mass Production at Springfield Armory during the American Civil War” (2017).** Useful contrasting studies of municipal provisioning and industrial equipment production. [De Re Militari](https://deremilitari.org/2014/05/the-town-in-service-of-war-in-the-medieval-crown-of-aragon/)

## Datasets and documentary collections

**Correlates of War, National Material Capabilities, version 7.0:** covers **1816–2022**, with military personnel, expenditure, population, and additional capability variables. Use the personnel and population series with their codebook definitions and provenance; do not treat the composite capability index as directly deployable combat strength. No global distribution was calculated from the downloadable dataset for this report. [Correlates of War](https://correlatesofwar.org/data-sets/national-material-capabilities/)

**Seshat:** useful for institutional comparisons and military characteristics. Its distinctions among potential army, professional army, and largest fielded army are especially valuable. Retain uncertainty codes and consult the underlying source notes before treating entries as comparable measurements. [Seshat DB](https://www.seshat-db.com/codebook?utm_source=chatgpt.com)

**The Medieval Soldier database:** individual and retinue records offer examples of named personnel, occupations, contracts, and uncertainties—closer to TCE’s agent-level representation than aggregate battle-size lists. [Medieval Soldier](https://medievalsoldier.org/about/agincourt/the-english-army-in-1415/english-army-table/?utm_source=chatgpt.com)

## Where confidence should remain low

**Administrative entitlement is not actual delivery.** A pay schedule does not prove timely payment; a ration scale does not prove sufficient intake; a register does not prove presence. The New Mexico Volunteers account, for example, describes serious arrears alongside the formal pay terms. [National Park Service](https://home.nps.gov/foun/learn/historyculture/new-mexico-volunteers.htm)

**Prehistoric and early-farming army sizes are poorly constrained here.** Archaeology, later texts, and ethnographic analogy answer different questions. Absence of a register cannot be converted into either “no warfare” or a precise military participation rate.

**Army counts and population estimates often have incompatible boundaries.** Urban versus rural populations, military households versus soldiers, authorized versus actual strength, and cumulative service versus a dated stock must remain explicit.

**Physiological and transport references are conditional.** Modern horse husbandry helps establish plausible requirements, but historical animals differed in size, condition, workload, and feed. A military marching manual describes a planning standard, not an observed long-run campaign average.

**There is no defensible single global coefficient for mobilization’s economic cost, nonbattle attrition, support-personnel ratio, or campaign duration.** These are better generated by the mechanisms above and calibrated against individual cases.

**The central design choice is therefore to ask not “How many soldiers can this population produce?” but “Which people can this institution bring together, with what equipment, at whose expense, and for how long can the economy keep them there?”**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92957-f640-83e9-9334-28498f04b9ba)
