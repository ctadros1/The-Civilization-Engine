# Daily routines and time use across history

## A simulation-ready report for The Civilization Engine

**Recommendation: build one household-aware activity scheduler, not separate “forager,” “medieval,” and “modern” timetables.** Let routines emerge from daylight, resource availability, production tasks, household responsibilities, access to facilities, and institutions that coordinate or compel attendance.

The historical labels remain useful for testing different configurations. They should not be executable rules. A farmer who also weaves, a factory worker supporting a rural household, and a merchant attending periodic markets need overlapping schedules—not mutually exclusive era presets.

The report below distinguishes **measured observations**, **historical reconstructions**, and **proposed simulation parameters**. Those categories should remain separate in TCE’s data files.

---

## 1. Mechanisms: what determines a person’s schedule?

### 1.1 Establish consistent time accounting first

The biggest comparison problem is what counts as work. Food procurement, paid employment, domestic production, childcare, travel, and waiting are not interchangeable categories. Agricultural measurement itself can be difficult: a randomized survey experiment in Tanzania found that end-of-season recall produced up to four times the reported hours **per person–plot** compared with weekly reporting. That discrepancy disappeared at household level because recall also omitted workers and plots. This is a warning about measurement, not a universal correction factor. [GOV.UK](https://assets.publishing.service.gov.uk/media/5fbce7af8fa8f559e77576e2/glmlic-wp015.pdf)

For TCE, use the following accounting conventions:

| Quantity | Recommended definition |
| --- | --- |
| **Primary activity time** | Exactly one primary activity at each moment; totals must equal 24 hours per agent-day. |
| **Secondary activities** | Compatible overlays such as supervising a child, talking, listening, or learning. These do not create extra hours. |
| **Production time** | Time assigned to subsistence or market production, including necessary task-related waiting. |
| **Domestic production and care** | Cooking, cleaning, collecting household necessities, maintaining clothing, and caring for dependents. Never treat these as automatically completed background processes. |
| **Attendance versus effort** | Track being on duty separately from active effort. A shopkeeper waiting for customers is working, but not continuously producing goods. |
| **Uncommitted time** | Time without an assigned obligation. It may become leisure, recovery, unemployment, waiting, or social activity; it is not automatically pleasant. |
| **Sleep opportunity versus sleep** | Distinguish the scheduled sleep interval from actual sleep, awakenings, naps, and time resting awake. |

Also store the denominator with every calibration target: **all people**, **employed people**, **participants in an activity**, **working days**, or **all calendar days**. A population-average work duration cannot be used directly as an employed adult’s shift.

### 1.2 Implementable causal rules

These are recommended mechanisms, with empirical anchors developed in Sections 2–3.

| Driver | Rule TCE should implement | Resulting behavior |
| --- | --- | --- |
| **Light, temperature, and weather** | Give outdoor tasks feasible windows and weather-dependent effort/productivity costs. Artificial lighting expands some windows at a resource cost. | Seasonal starts and finishes, weather interruptions, optional midday rests, indoor substitutions. |
| **Resource and crop timing** | Generate tasks from resource availability and biological deadlines, not from a fixed occupational work quota. | Fishing trips, harvest rushes, planting bottlenecks, irregular procurement days. |
| **Household requirements** | Generate food preparation, water, fuel, maintenance, and care obligations before allocating all adults to external work. | Shared chores, caregivers staying near home, children accompanying adults, hired help. |
| **Labor institutions** | Employers, estates, households, religious bodies, and states can impose attendance, service, or labor obligations. Model compliance and sanctions separately. | Factory shifts, compulsory labor, apprenticeships, domestic service, collective maintenance. |
| **Shared calendars** | Institutions announce recurring and exceptional events with participants, locations, and attendance windows. | Markets, worship, festivals, assemblies, synchronized rest days. |
| **Travel and access** | An activity requires a reachable destination, sufficient travel time, and permission or payment where applicable. | Nearby substitutes, combined errands, unequal access to leisure, realistic market catchments. |
| **Technology and infrastructure** | Change specific task requirements, batch sizes, labor coefficients, and locations—not a global “hours worked” multiplier. | A mill displaces household grinding; water infrastructure changes collection trips; a factory concentrates attendance. |
| **Shocks and recovery** | Illness, injury, conflict, weather damage, and shortages modify available capacity and create additional tasks. | Care burdens, repairs, interrupted routines, emergency work, or inactivity when productive opportunities disappear. |

### 1.3 Make seasonal pressure a workload calculation

For a field operation \(j\), represent required labor as:

\[
L\_j=A\_j\,\ell\_j
\]

where \(A\_j\) is area in hectares and \(\ell\_j\) is labor required in person-hours per hectare, conditional on tools, soil, method, and skill.

A useful scheduling indicator is:

\[
\text{urgency}\_j =
\frac{\text{remaining required person-hours}}
{\text{available effective person-hours before the deadline}}.
\]

The denominator must exclude time already committed to travel, care, other urgent jobs, and infeasible weather.

**Illustrative calculation—not historical data:** a harvest requires 120 remaining person-hours, but the household can supply only 80 before expected crop loss. TCE should produce a real response: seek 40 hours of outside help, postpone another task, accept losses, or change future cultivated area. Giving everyone a generic “harvest-season productivity bonus” would conceal the underlying coordination problem.

---

## 2. Quantitative parameters and time budgets

### Confidence convention

**High** means strong evidence for the stated population and measurement definition. **Medium** means a localized observational study or an archival reconstruction requiring interpretation. **Low** means a proposed initialization or a weakly constrained historical inference. None implies universal transferability.

### 2.1 Workdays, workweeks, and annual attendance

| Population or setting | Quantitative benchmark | What it actually measures | Confidence and source |
| --- | --- | --- | --- |
| **Agta, northern Philippines** | **29.2%** external work; **24.0%** domestic chores; **12.2%** direct childcare; **34.7%** leisure | Daylight observations of 142 adults across ten camps. Leisure includes daytime sleep; external work includes foraging and non-foraging activities. **Not a 24-hour budget.** | Medium; Dyble et al. (2019). [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/10091712/3/Dyble_Agta%20time%20budgets%20MS.pdf) |
| **London construction workers, early eighteenth century** | About **5.2 days worked per week**, out of six offered; often **under 150 days/year with this employer** | St Paul’s construction records. The annual figure excludes work elsewhere and household production. | Medium; Stephenson’s analysis. [EHS](https://ehs.org.uk/how-many-days-a-year-did-people-work-in-england-before-the-industrial-revolution/) |
| **Normandy, late medieval to eighteenth century** | Reconstruction suggests around **250 working days/year** in the late medieval period; **290–300** was not uncommon by the mid-eighteenth century | Regional payment records and calendar reconstruction, not a universal European average. | Medium; Chambru and Maneuvrier-Hervieu. [Springer](https://link.springer.com/chapter/10.1007/978-3-031-91930-5_7) |
| **Lowell textile mills, nineteenth-century United States** | **12–14 hours/day**, half-day Saturdays, Sunday closure; many mill women employed **9–10 months/year** | Factory schedules and employment patterns. Do not interpret the full span as uninterrupted physical effort. | Medium; Lowell National Historical Park’s documentary account. [National Park Service](https://www.nps.gov/lowe/learn/historyculture/the-mill-girls-of-lowell.htm) |
| **United States, full-time employed people, 2024** | **8.4 hours** on weekdays worked; **5.6 hours** on weekend days worked. Participation: **87%** on an average weekday, **29%** on a weekend day | Conditional work duration and daily participation, not a seven-day personal timetable. | High; BLS American Time Use Survey. [Bureau of Labor Statistics](https://www.bls.gov/news.release/archives/atus_06262025.htm) |

These figures should become **separate calibration cases**, not a historical progression curve. In particular, days recorded by one employer cannot be compared directly with reconstructed total annual workdays.

For early farming, the sources reviewed do not justify a precise worldwide workweek. TCE should derive hours from task requirements and household capacity, using explicitly uncertain starting values rather than inventing a Neolithic average.

### 2.2 Sleep, care, and age-related benchmarks

| Parameter | Quantitative evidence | Interpretation and confidence |
| --- | --- | --- |
| **Sleep in three non-industrial communities** | Mean actigraphic sleep durations **5.7–7.1 h/night**; sleep periods **6.9–8.5 h** | Hadza in Tanzania, San in Namibia, and Tsimane in Bolivia. Sleep period means onset to final awakening, not continuous sleep. Medium for transferability. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/yetishetal2015.pdf) |
| **Seasonal sleep timing** | Approximately **1 h more sleep in winter**; mean onset about **3.3 h after sunset** | Findings from those communities, not a universal human schedule or recommended sleep duration. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/yetishetal2015.pdf) |
| **Children’s food production** | About **11% of observed time at ages 7–12**, rising to **21% at 13–18** | Comparative data from 12 foraging and mixed-subsistence societies; substantial variation. Medium. [Cambridge Repository API](https://api.repository.cam.ac.uk/server/api/core/bitstreams/a5a65cbe-a02f-460c-b52b-a9218770389a/content) |
| **Children’s domestic work** | Approximately **12% → 19%** across those age groups | Observation categories, not hours of formal employment. Medium. [Cambridge Repository API](https://api.repository.cam.ac.uk/server/api/core/bitstreams/a5a65cbe-a02f-460c-b52b-a9218770389a/content) |
| **India, women aged 15–59, 2024** | **305 min/day** in unpaid domestic services, among participants | A participating-day mean, not every woman’s daily allocation. High for this definition. [Press Information Bureau](https://www.pib.gov.in/PressReleasePage.aspx?PRID=2106113) |
| **India, household caregiving, ages 15–59** | Participants: women **140 min/day**, men **74**; participation **41%** and **21.4%**, respectively | Duration and participation are distinct parameters. Do not add conditional averages into a population-wide 24-hour budget. [Press Information Bureau](https://www.pib.gov.in/PressReleasePage.aspx?PRID=2106113) |

### 2.3 An observed modern 24-hour budget

The following groups the **2024 US ATUS population averages for people aged 15 and over**, including employed and non-employed people and all days of the week.

| Primary-activity grouping | Hours/person/day |
| --- | --- |
| Recorded sleep | 9.04 |
| Other personal care plus eating | 2.00 |
| Work and work-related activities | 3.43 |
| Household activities, care, and purchasing | 3.36 |
| Education, civic, and religious activities | 0.72 |
| Leisure and sport | 5.07 |
| Communication and other activities | 0.39 |
| **Published total** | **24.00** |

The grouped entries total 24.01 because of rounding. Activity-related travel is already included. Recorded sleep includes naps and sleeplessness, so it is not directly comparable with actigraphic sleep duration. [Bureau of Labor Statistics](https://www.bls.gov/news.release/archives/atus_06262025.htm)

**Simulation implication:** reproduce population averages by mixing different kinds of people and days. Do not give every employed adult a 3.43-hour shift.

### 2.4 Proposed TCE seed budgets—not historical estimates

These are feasible **initial adult day templates**, in hours per day. They are low-confidence design priors intended for testing before task-level calibration.

“Production/duty” excludes separately recorded meals, travel, and domestic work; actual effort can occupy only part of that interval. “Sleep allocation” includes the opportunity for sleep and associated awakenings.

| Adult day type | Sleep allocation | Production / duty | Domestic work / active care | Travel | Eating / personal care | Uncommitted / social / recovery | Total |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Forager, ordinary resource day | 8 | 4 | 3 | 1 | 2 | 6 | **24** |
| Farmer, ordinary day | 8 | 6 | 3 | 1 | 2 | 4 | **24** |
| Farmer, urgent seasonal peak | 7 | 10 | 2 | 1 | 2 | 2 | **24** |
| Farmer, field-work slack season | 8.5 | 3 | 4 | 0.5 | 2 | 6 | **24** |
| Town artisan, working day | 8 | 8 | 2 | 0.5 | 2 | 3.5 | **24** |
| Early factory worker, long day | 7 | 11.5 | 1.5 | 0.5 | 1.5 | 2 | **24** |
| Modern worker, working day | 8 | 8 | 2 | 1 | 2 | 3 | **24** |

These are **not population means**, healthy-duration prescriptions, or schedules for primary caregivers.

Three implementation safeguards matter:

**Redistribute rather than erase obligations.** The farmer’s reduced domestic allocation during a peak requires another household member, purchased provision, a deferred task, or an unmet need. The factory template may require boarding-house meals or another person doing household work.

**Do not randomize columns independently.** Perturb episode timing and duration, then reconcile the complete day. Independent random draws readily produce impossible totals.

**Compute annual work from realized days.** Sum actual episodes across weather, employment, rest-day, illness, and seasonal states. Do not multiply a peak-day schedule by a nominal number of working days.

---

## 3. Variation across societies, regions, and the life course

### 3.1 Foragers: model resource-dependent episodes

Forager schedules should be generated around procurement opportunities, travel, processing, camp maintenance, and social obligations—not an obligatory daily shift.

The Agta study found less leisure in camps more engaged in agricultural and other non-foraging work, with the association especially pronounced among women. Its authors explicitly caution against treating contemporary communities as direct substitutes for prehistoric populations. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/10091712/3/Dyble_Agta%20time%20budgets%20MS.pdf)

For TCE, a procurement day should differ from a processing day, a relocation day, and a day when a valuable resource is unavailable. Storage, sharing, and successful large returns should affect subsequent trips. A household with food in store need not send every capable adult out daily.

Conversely, “not hunting” must not mean “doing nothing”: preparation, equipment maintenance, food processing, care, and social activity need visible episodes. The proposed four-hour procurement template is therefore only one possible day, not a universal forager work allowance.

### 3.2 Farming: calendars should follow crop biology and water

The following are **recommended task-calendar structures**, using regional agricultural examples as ecological anchors. Modern crop calendars can identify seasonal opportunities; they cannot establish ancient labor requirements or prove that ancient farmers used the same varieties and cropping intensity.

| Agricultural configuration | Recommended sequence and scheduling consequences | Regional anchor |
| --- | --- | --- |
| **Temperate winter cereals** | Prepare and sow in the appropriate cool/wet window; maintain fields; concentrate harvesting and post-harvest handling near maturity. Place repairs, processing, and other enterprises in remaining windows. | Türkiye’s winter wheat/barley and differently timed maize illustrate why crop-specific calendars are necessary. [FAOHome](https://www.fao.org/giews/countrybrief/country.jsp?code=TUR) |
| **Sahelian rain-fed cereals** | Rain onset opens planting opportunities; failed establishment can create replanting; weeding and later harvesting create separate demand peaks. | In Mali, the usual rainy season is May–September, while the June–August lean season can overlap agricultural work. [FAOHome](https://www.fao.org/giews/countrybrief/country.jsp?code=MLI) |
| **South Asian rice systems** | Represent land preparation, establishment method, water management, weeding, harvesting, and processing separately. Transplanting and direct seeding require different task graphs. | Bangladesh has distinct Boro, Aus, and Aman rice seasons. Multiple seasons are not proof that every field or farmer grows three crops annually. [FAOHome](https://www.fao.org/giews/countrybrief/country.jsp?code=BGD) |
| **Mesoamerican maize systems** | Use locally feasible planting and maturity windows; allow one or multiple seasonal crops only when water, temperature, varieties, and household labor permit. | Guatemala’s Primera, Segunda, and regionally later crops show substantial within-country calendar variation. [FAOHome](https://www.fao.org/giews/countrybrief/country.jsp?code=GTM) |
| **Mixed farming and pastoral variants** | Add daily herd care and water requirements, then overlay pasture movements, animal reproduction, fodder production, and crop work. | Mali’s crop and pastoral seasonal monitoring provides an example of linked but distinct seasonal systems. [FAOHome](https://www.fao.org/giews/countrybrief/country.jsp?code=MLI) |

**Do not equate “winter” with leisure.** In the model, low field demand should release time for other feasible work, not delete household needs. Whether that becomes textile production, repairs, trade, social visits, or rest should depend on available tasks and incentives.

Likewise, a second annual crop should create **another labor cycle**, potentially competing with the first crop’s harvesting and processing. It should not simply double an annual yield statistic.

For an early farming settlement, the most informative annual visualization would show **required versus available household person-hours**, by week, alongside food stocks. That makes labor bottlenecks and shortages legible without scripting a crisis.

### 3.3 Pre-industrial towns: distinguish occupations and control over time

Town life should not collapse into an eight-hour artisan schedule. A useful TCE distinction is between:

* **Task-controlled work:** completing orders, batches, repairs, or deliveries.
* **Attendance-controlled work:** being present at a workshop, household, gate, stall, or institution.
* **Opportunity-controlled work:** serving customers, obtaining casual employment, or attending a periodic market.

These are modeling categories rather than exclusive historical occupations.

Stephenson’s construction evidence is particularly useful because attendance was irregular even when six days were offered, and low annual attendance at a single site does not establish generous leisure. The model should allow workers to combine employers and other activities. [EHS](https://ehs.org.uk/how-many-days-a-year-did-people-work-in-england-before-the-industrial-revolution/)

For artisans, separate workshop opening time from each worker’s effort. For traders, include preparing goods, travel, sales, and return travel. For domestic servants, track multiple obligations spread across the household’s day rather than treating the job as one compact shift.

### 3.4 Industrialization: synchronization, not simply longer hours

Lowell provides a clear case of factory bells coordinating workers’ days. Its long hours, Saturday pattern, and seasonal employment are useful as one institution-specific configuration, not a definition of all industrial labor. [National Park Service](https://www.nps.gov/lowe/learn/historyculture/the-mill-girls-of-lowell.htm)

The transition should therefore be represented through **larger synchronized workplaces, attendance enforcement, production-process requirements, and household dependence on wages**.

The geographical comparison matters. Tsurumi’s *Factory Girls* documents women’s textile labor in Meiji Japan and its relationship to rural households and exploitative employment conditions. Industrialization cannot be modeled solely as adult men leaving otherwise self-contained households for factory jobs. [Google Books](https://books.google.com/books/about/Factory_Girls.html?id=TfsoDwAAQBAJ)

In TCE, let workers retain obligations and economic links outside their workplace. Factory attendance can displace tasks onto relatives or services, cause tasks to be deferred, or generate unmet needs.

### 3.5 Modern societies: preserve heterogeneity

A modern configuration should add possible institutions—schools, formal shifts, commuting systems, retirement provision, commercial childcare, and household technologies—rather than replace everyone’s routine with “work, shop, leisure.”

The ATUS benchmarks show both strong weekday concentration and substantial weekend work among full-time workers. A convincing town therefore needs overlapping day types, not a universal weekend shutdown. [Bureau of Labor Statistics](https://www.bls.gov/news.release/archives/atus_06262025.htm)

### 3.6 Weeks, religious observance, and festivals

**Do not hard-code a seven-day commercial cycle.** Skinner’s study of rural China describes widespread schedules with two or three market days per ten-day subdivision, alongside other arrangements. Neighboring markets could be staggered, supporting traveling traders rather than requiring every settlement to trade simultaneously. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952)

**Do not equate a religious calendar with observed days off.** Medieval canonical lists discussed in the Normandy research specify 36 holidays plus 52 Sundays, later adding four holidays. These are normative lists, not proof of 88–92 distinct, universally work-free days: overlaps, local rules, enforcement, and necessary labor matter. [Springer](https://link.springer.com/chapter/10.1007/978-3-031-91930-5_7)

**A gathering window need not imply an all-day closure.** Qur’an 62:9–10 directs believers to leave trade for Friday prayer and permits dispersal afterward. This is evidence of a normative coordination rule, not an attendance survey or a universal Friday work prohibition. [Quran](https://quran.com/id/al-jumuah/9/translations/en-sahih-international)

Implement calendars as institution-owned event rules:

`recurrence + duration + location + eligible participants + obligations + exemptions + enforcement`

A festival should simultaneously create rest or celebration for some people and additional preparation, selling, cooking, performance, security, and cleanup work for others. Resolve overlapping holidays by taking the union of affected time windows, not by subtracting nominal days independently.

### 3.7 Sleep: the segmented-sleep debate

**Segmented sleep is historically attested; its prevalence is contested.**

Ekirch argues that first and second sleep were widespread in pre-industrial Western societies, drawing on literary, medical, legal, and other documentary references. Boyce questions whether many references establish a regular two-block nightly pattern and cautions against assuming universality. Ekirch’s subsequent response defends the interpretation. Neither side supplies a reliable global percentage that TCE can assign to all pre-electric populations. [Cambridge University Press](https://www.cambridge.org/core/product/identifier/S0025727323000145/type/journal_article)

The actigraphy study of Hadza, San, and Tsimane did not find routine extended middle-of-the-night wake intervals in those samples. Its seasonal and sunset-relative findings also argue against a universal “sleep at sunset, rise at dawn” rule. Those communities do not, however, settle the historical European debate. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/yetishetal2015.pdf)

**Recommended representation:** give agents a sleep requirement, circadian preference, sleep opportunity, and interruption process. Permit consolidated sleep, planned two-block sleep, naps, and fragmented sleep. Let culture and circumstances influence episode placement without turning electricity into an automatic “segmented sleep off” switch.

For sensitivity testing, vary the prevalence of planned segmentation as an **explicit scenario assumption**, not an estimated historical fact. Preserve total sleep accounting when inserting a waking interval.

### 3.8 Meals and gathering places

Do not impose breakfast at 07:00, lunch at 12:00, and dinner at 18:00 on every culture. Instead, make shared meals potential **coordination events** constrained by food availability, preparation, work breaks, household attendance, and institutional rules. Small consumption events can coexist with one or more larger shared meals.

For a first implementation, two or three potential shared-meal windows and flexible portable-food consumption are reasonable **design options**, not historical parameter estimates. Cooking labor, waiting for a batch, eating, and cleanup should be distinct costs.

Gathering places should offer different activity bundles rather than interchangeable “social need” service:

| Place | What to represent |
| --- | --- |
| **Household hearth or campfire** | Shared meals, conversation, storytelling, performance, warmth, care, and knowledge transmission. |
| **Market and associated eating/drinking places** | Trading plus information exchange, credit relationships, meetings, refreshments, and entertainment. |
| **Temple or other ritual venue** | Scheduled observance, preparation and maintenance work, festivals, and institution-specific access. |
| **Tavern or teahouse** | Food/drink service, recurring companions, entertainment, and sometimes business. |
| **Bathing facility** | Washing plus social interaction, constrained by water, fuel, staffing, capacity, price, and rules. |

The evidence supports making social activity purposeful. Wiessner’s Ju/’hoansi research found a marked contrast between daytime and firelit conversation; stories accounted for **81% of lengthy group conversations at night**, not 81% of all nighttime activity. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1404212111) Skinner describes rural Chinese markets as social institutions with teahouses and other gathering opportunities, while Fagan’s Roman bathing history establishes baths as social spaces rather than merely washing infrastructure. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952)

### 3.9 Gender, age, and household position

Model **responsibilities and constraints**, not sex-specific fixed timetables.

India’s measured care and domestic-work differences illustrate why an external-work-only scheduler would omit a substantial and uneven burden. They do not establish an immutable division for all societies. [Press Information Bureau](https://www.pib.gov.in/PressReleasePage.aspx?PRID=2106113)

Recommended household assignment should consider ability, skill, proximity, competing earnings, relationships, bargaining power, and cultural expectations. A norm can strongly bias assignment without making alternative arrangements impossible.

Children should acquire responsibility gradually. The comparative study of 690 children and adolescents found increasing food-production and domestic-work shares with age, alongside substantial cross-cultural variation. Productive participation, play, and learning should therefore overlap rather than become a switch from “idle child” to “adult worker.” [Cambridge Repository API](https://api.repository.cam.ac.uk/server/api/core/bitstreams/a5a65cbe-a02f-460c-b52b-a9218770389a/content)

Schooling creates another institutional claim on time. In India’s 2024 survey, **89.3% of children aged 6–14 participated in learning activities**, with participants spending approximately **413 minutes** that day. This is a useful modern calibration case, not a schedule for children without that institution. [Press Information Bureau](https://www.pib.gov.in/PressReleasePage.aspx?PRID=2106113)

Older agents should shift tasks according to capability and opportunity rather than cease all contribution at a universal retirement age. Social and instructional roles can remain valuable: Wiessner describes the importance of skilled older storytellers. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1404212111)

Finally, represent legal and social status separately from occupation. For servants, dependents, bonded workers, or enslaved people, the central modeling difference is often **who controls their time and what refusal costs**, not an intrinsic preference for work.

---

## 4. Stylized facts a correct simulation should reproduce

These are **scenario-specific validation targets**. Reproducing one population should not require every generated society to converge on its schedule.

| Pattern | Validation target |
| --- | --- |
| **External work is not total labor** | In an Agta-like calibration, reproduce the separately observed external, domestic, care, and leisure shares rather than treating non-procurement time as leisure. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/10091712/3/Dyble_Agta%20time%20budgets%20MS.pdf) |
| **Actual attendance differs from offered work** | A construction-employment case should permit incomplete weeks, changing employers, and irregular annual attendance. It should not interpret every non-attendance day as voluntary rest. [EHS](https://ehs.org.uk/how-many-days-a-year-did-people-work-in-england-before-the-industrial-revolution/) |
| **Agricultural time is seasonally uneven** | Generate concentrated task demand around establishment and harvesting, with regional differences and possible overlap between agricultural work and food scarcity. [FAOHome](https://www.fao.org/giews/countrybrief/country.jsp?code=MLI) |
| **Children’s contribution rises with age** | Reproduce the observed direction and approximate scale in the 12-society comparison without eliminating play or forcing identical gender roles. [Cambridge Repository API](https://api.repository.cam.ac.uk/server/api/core/bitstreams/a5a65cbe-a02f-460c-b52b-a9218770389a/content) |
| **Shared calendars concentrate activity** | Periodic-market scenarios should show attendance peaks and merchant movement between staggered markets. [OpenEdition Journals](https://journals.openedition.org/etudesrurales/7952) |
| **Nighttime is not universally inactive** | Permit evening social activity and varied sleep timing without imposing universal nightly segmentation. [PNAS](https://www.pnas.org/doi/10.1073/pnas.1404212111) |
| **Modern work remains heterogeneous** | Match both conditional work duration and weekday/weekend participation, not just average weekly hours. [Bureau of Labor Statistics](https://www.bls.gov/news.release/archives/atus_06262025.htm) |
| **Care changes other opportunities** | A modern India calibration should reproduce unequal care participation and duration through household constraints and institutional settings. [Press Information Bureau](https://www.pib.gov.in/PressReleasePage.aspx?PRID=2106113) |

Separately enforce software invariants: every day closes at 24 hours; agents cannot occupy two locations; shared appointments have mutually compatible schedules; travel consumes time; and required care cannot be satisfied by incompatible simultaneous activities.

A useful diagnostic is the distribution of **longest uninterrupted discretionary interval**, not just total leisure. TCE can expose the difference between a person with several free hours together and one with the same total scattered between obligations.

---

## 5. Recommended representation for TCE

### 5.1 Store responsibilities at the correct level

| Entity | Minimum useful state |
| --- | --- |
| **Agent** | Age/capabilities, occupation and status, fatigue and sleep pressure, preferences, skills, current episode, location, commitments, dependents, habitual destinations. |
| **Household** | Members, food/fuel/water stocks, care requirements, domestic task queue, production assets, shared commitments, task assignments, purchasing options. |
| **Task** | Location, prerequisites, remaining effort, feasible windows, deadline, required skills/tools, allowable crew, interruptibility, outputs, compatible secondary activities. |
| **Institution or venue** | Calendar, opening windows, attendance rules, access conditions, service capacity, fees, staffing requirements, sanctions, recurring participant groups. |

Do not put “the household cooks dinner” in a decorative subsystem. Assign its labor to actual people or purchased services.

### 5.2 Use plans with flexible episodes

At the start of a day—or after a major disruption—construct a small feasible plan.

First reserve binding commitments: shifts, collective work, care, shared appointments, and necessary travel. Then allocate urgent household and production tasks. Fill remaining windows with optional work, social visits, recreation, learning, and recovery.

A bounded action score can take the form:

\[
U\_i(a)=
w\_n\Delta\text{needs}
+w\_h\Delta\text{household security}
+w\_d\text{deadline benefit}
+w\_s\text{social benefit}
+w\_b\text{habit fit}
-\text{travel cost}
-\text{fatigue cost}
-\text{switching cost}.
\]

This is a **proposed decision architecture**, not an empirically estimated universal utility function. Normalize its components before tuning weights.

Use constraints to determine what is feasible; use utility to choose among feasible actions. Otherwise a sufficiently attractive tavern can mathematically “outbid” a task even when the agent cannot physically get there.

Add inertia so people finish sensible episodes instead of switching whenever scores change slightly. Interrupt when something material changes: a storm, a crying dependent, a depleted input, a missed connection, a customer arrival, or an urgent physiological need.

### 5.3 Synchronize through shared objects

Household meals, market sessions, work crews, and ceremonies should be **shared event objects** with participants and reserved intervals.

This avoids the common failure where every agent independently decides to socialize, but potential partners arrive at different times or leave immediately. Participation should improve the attraction of an event, while capacity, costs, exclusion, and competing commitments limit attendance.

Allow recurring familiar groups and destinations. Purely random encounters can be a fallback, not the whole social scheduler.

### 5.4 Keep the visible life and economic life identical

For 10k–50k agents, my implementation recommendation is:

**Plan coarsely; execute episodes cheaply.** Use event-driven transitions and bounded candidate sets, with a fallback reconsideration interval around **5–15 simulated minutes**. That interval is an engineering starting point to benchmark, not a demonstrated throughput result.

Batch household planning and recurring institutional reservations. Cache habitual plans, but invalidate the relevant portions when circumstances change. Use task queues indexed by location, skill, and urgency instead of asking every agent to inspect every job.

Keep the Rust kernel authoritative. Unreal should render the same trips, queues, work episodes, and gatherings that consume simulation time. Camera distance must not create extra production or remove household obligations.

At high speed, aggregate or statistically resolve the same feasible episodes. Do not award a daily production total independently and then invent a visually plausible day afterward.

### 5.5 Existing models and games worth borrowing from

| System | Reusable idea | Limitation |
| --- | --- | --- |
| **ActivitySim** | Mandatory work/school tours, available time windows, household joint tours, escorting, and dependent scheduling decisions. | Its modern travel-demand coefficients are not historical behavioral parameters. Borrow the constraint structure. [ActivitySim](https://activitysim.github.io/activitysim/v1.2.0/models.html) |
| **MATSim** | Daily activity plans connected by real travel, with consistent person-level schedules and transport consequences. | Primarily a transport framework; it does not supply early-agrarian care or production mechanisms. [MATSim](https://www.matsim.org/docs/) |
| **Dwarf Fortress** | Needs expressed through meaningful destinations and group activities: taverns, temples, libraries, stories, music, and dance. | A useful design precedent, not evidence for human time-budget parameters. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2015.html) |

**Simplify individual movements within an activity, not the activity’s obligations.** It is cheaper and safer to approximate exactly how someone washes dishes than to omit cooking and cleaning from the household economy.

---

## 6. Sources, datasets, and uncertainty

### 6.1 Recommended calibration datasets

| Source | Best use in TCE |
| --- | --- |
| **American Time Use Survey microdata** | Construct modern episode sequences and conditional schedules, retaining survey weights and respondent characteristics. [Bureau of Labor Statistics](https://www.bls.gov/tus/data/datafiles-2024.htm) |
| **Multinational Time Use Study, Centre for Time Use Research** | Compare harmonized modern diaries across societies and periods rather than treating one country as universal. [Time Use Research](https://www.timeuse.org/mtus) |
| **India Time Use Survey 2024** | Test domestic work, care, learning, and participation distinctions in a large non-Western setting. [Press Information Bureau](https://www.pib.gov.in/PressReleasePage.aspx?PRID=2106113) |
| **Lew-Levy et al. comparative child time-allocation data/code** | Calibrate age gradients, task participation, and cross-society variation in subsistence childhoods. [GitHub](https://github.com/sheinalewlevy/HGC-TA) |
| **FAO crop calendars and GIEWS country briefs** | Define regional crop-season opportunities and competing seasonal tasks; combine with separately sourced historical technologies and labor coefficients. [FAOHome](https://www.fao.org/giews/countrybrief/index.jsp) |

### 6.2 Scholarly core

The most directly reusable studies are **Dyble et al. (2019)** on Agta time allocation; **Yetish et al. (2015)** on sleep in three non-industrial communities; **Lew-Levy et al. (2022)** on children’s time allocation; **Stephenson** and **Chambru–Maneuvrier-Hervieu** on archival work attendance; and **Arthi and colleagues** on agricultural labor measurement. Their quantitative findings appear above with the relevant denominators.

For social scheduling, use **Skinner’s *Marketing and Social Structure in Rural China***, **Wiessner’s “Embers of society”**, **Fagan’s *Bathing in Public in the Roman World***, and **Tsurumi’s *Factory Girls***. For segmented sleep, read **Boyce (2023)** together with **Ekirch’s response**, rather than choosing one popularized account.

### 6.3 Where confidence is weakest

The evidence reviewed is thinnest for complete 24-hour budgets of the earliest farmers; representative ancient urban populations; high-latitude and Oceanian settings; and people whose labor was poorly recorded, including many domestic and coerced workers. The historical hour estimates are particularly weighted toward European and North American records.

Precise meal clocks, leisure durations, and the prevalence of segmented sleep are also weaker candidates for universal parameters than task requirements, attendance records, or directly observed activity shares. Modern ethnography and crop calendars help construct possibilities; they do not turn those possibilities into measured prehistoric schedules.

For these areas, retain broad alternative parameter sets and test sensitivity. Report an outcome as robust only when it survives plausible changes to sleep timing, care allocation, travel requirements, seasonal labor demand, and institutional attendance.

**The highest-priority implementation is a household time ledger linked to physical tasks and shared calendars.** Once every necessary hour belongs to someone, seasonality, unequal burdens, labor shortages, crowded markets, quiet streets, and evening gatherings can emerge from the same system—without scripting what a civilization’s day must look like.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927f7-53c8-83e9-8030-b346b8b63596)
