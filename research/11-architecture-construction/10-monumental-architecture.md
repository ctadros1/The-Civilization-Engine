# Monumental architecture: why societies build big

## Executive conclusion

**For TCE, a monument should be an ordinary construction project with extraordinary collective meaning, mobilization requirements, and institutional persistence—not a special building that automatically produces prestige.**

Large monuments do not require kings, centralized states, or even farming. Archaeological research documents monumental construction among hunter-gatherers, early pastoralists, and communities without clear evidence of pronounced hierarchy. Conversely, royal monuments can demonstrate the ability to command resources without demonstrating popular consent. These are different social outcomes and should remain separate in the simulation. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1002/gea.21430)

The central causal loop to model is:

**Shared purpose or political ambition → coalition and resource commitments → construction and public participation → audience interpretations → changes in cooperation, authority, rivalry, and future investment.**

Construction also changes the coalition itself. The work can build administrative capacity, specialist communities, and enduring obligations—not merely consume a surplus that already existed. Excavations of the Giza workers’ settlement make this organizational dimension particularly visible. [AERA](https://aeraweb.org/projects/lost-city/)

---

## 1. Mechanisms: rules TCE can implement

### 1.1 Monumentality is relative, not a fixed size category

Trigger’s influential interpretation treats conspicuous expenditure of energy as a demonstration of social power. But monumental significance cannot be reduced to cubic metres: difficult transport, exceptional workmanship, restricted materials, prominent placement, and association with sacred or ancestral events can matter independently of mass. Costly-signaling research likewise emphasizes what particular audiences can infer from a construction. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/00438243.1990.9980135)

**Implementation:** retain several independent attributes:

| Attribute | Suggested representation | Why it matters |
| --- | --- | --- |
| Relative investment | Project labor divided by local ordinary-building labor | A structure can be exceptional in a village without being exceptional in an imperial capital. |
| Physical prominence | Height above surroundings, visible area, approach visibility | Determines who encounters it and from where. |
| Craft distinction | Specialist hours, precision, ornament complexity, material rarity | Allows modest-sized but prestigious buildings. |
| Symbolic association | Deity, ancestor, dynasty, victory, community, institution | Determines whose identity or authority it represents. |
| Public accessibility | Gathering capacity, access restrictions, usable facilities | Distinguishes collective meeting places from exclusive monuments. |
| Historical depth | Remembered events, burials, rebuilding episodes, continuity of use | Allows significance to accumulate without continual enlargement. |

These are proposed simulation variables, not an empirically established universal monumentality index. **Do not collapse them into one number until a particular agent evaluates them.**

### 1.2 Support several motives simultaneously

Religious commitment, political competition, and collective cooperation are not mutually exclusive explanations. Kantner and Vaughn interpret pilgrimage at Chaco and Nasca as potentially supporting cooperation among people who were not close kin; Glatz and Plourde examine monuments as instruments of political competition in Late Bronze Age Anatolia. Neither interpretation implies that all monuments have the same function. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416511000535)

For TCE, give a proposed project a **motive vector**, rather than one purpose:

* Sacred obligation, burial, or ancestral commemoration.
* Practical services and collective gathering.
* Patronal reputation, political authority, and competition.
* Community identity, historical memory, and—in appropriate economies—commercial returns.

Individual supporters can value different components. A ruler seeks dynastic memory, priests seek a suitable sanctuary, merchants anticipate visitors, masons seek wages, and households seek ritual participation.

A useful decision structure is:

\[
U\_{i,j} =
V^{sacred}\_{i,j}
+V^{service}\_{i,j}
+V^{identity}\_{i,j}
+V^{status}\_{i,j}
+V^{network}\_{i,j}
-C^{time}\_{i,j}
-C^{goods}\_{i,j}
-C^{risk}\_{i,j}.
\]

These are **model utility terms**, not historically measured coefficients. Voluntary contributions depend on this utility relative to alternatives. Compulsory contributions require a separate mechanism involving obligations, enforcement, evasion, and resistance.

### 1.3 Costly signals need an audience—and an identifiable cost-bearer

The useful part of costly-signaling theory is not “expensive things produce legitimacy.” It is that an observable action can supply information about a quality that is otherwise difficult to verify.

A monument might suggest that its sponsors possess:

| Claimed quality | Potentially informative observation | Important alternative interpretation |
| --- | --- | --- |
| Resource-commanding capacity | Sustained deliveries and a large workforce | Resources were borrowed, confiscated, or obtained from outside patrons. |
| Commitment to a deity or community | Repeated contributions and maintenance | The sponsor shifts most costs onto others. |
| Organizational competence | Reliable progress, provisioning, and safe construction | A finished façade conceals failures elsewhere. |
| Collective solidarity | Broad participation across households | Participation was compulsory or excluded important groups. |
| Cultural distinction | Exceptional design and craftsmanship | Expertise belongs to imported artisans rather than local institutions. |

Research on Maya display explicitly considers **cost shifting**, making it a useful corrective to the assumption that monumental expenditure always honestly reveals the patron’s own sacrifice. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0278416522000897)

**Implementation:** store the sponsor, payer, labor provider, designer, and beneficiary separately. Agents should infer costs from what they observe; they should not have access to the simulation’s true construction ledger.

Most importantly, distinguish:

**Perceived power ≠ perceived competence ≠ moral legitimacy ≠ affection.**

The same palace can increase fear of a ruler while reducing willingness to support that ruler voluntarily.

### 1.4 Collective construction can precede centralized government

Aguada Fénix is especially important here. Its excavators identified immense early construction without the pronounced elite representation familiar from some other Mesoamerican centres. Their later research describes a much larger ritual landscape, built approximately **1050–700 BCE**, with axes extending **9 km and 7.5 km**. The interpretation is communal monumental activity, not a demonstration that a fully developed monarchy must already have existed. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/10216759/)

**Implementation:** permit projects to originate from councils, ritual associations, clans, neighborhood groups, or temporary inter-settlement coalitions.

Coordination can arise through pledges, reciprocal labor, scheduled gatherings, and recognized ritual leadership. Successful coordination can subsequently strengthen an institution. Do not require an advanced government technology before collective earthworks become possible.

However, absence of obvious elite residences or imagery is **not proof of complete equality or wholly voluntary participation**.

### 1.5 Surplus matters, but timing and mobilization matter just as much

A project needs more than aggregate wealth. It needs resources in usable places and seasons: food at the worksite, transport animals or boats when available, specialists with compatible schedules, and materials delivered in construction order.

Giza’s provisioning involved a regional supply system, rather than simply feeding workers from immediately adjacent farmland. Medieval cathedral construction similarly separated year-round activities such as some stone cutting from weather-sensitive masonry work. [AERA](https://aeraweb.org/pyramids-and-protein/)

**Implementation:** distinguish:

* **Potential surplus:** what remains after essential production and consumption.
* **Mobilizable surplus:** what institutions can actually collect or purchase.
* **Delivered surplus:** what reaches the project in usable condition.

A prosperous but politically fragmented community may fail to mobilize a project. A poorer but well-coordinated coalition may succeed at a smaller one.

Seasonal labor is not free labor. Workers still have household production, repair, childcare, transport, and other opportunities.

### 1.6 Financing determines control and vulnerability

Different financing arrangements produce different political and construction dynamics. Cathedral chapters raised money from congregations and other religious income sources; St Paul’s rebuilding received an earmarked coal tax. Ottoman religious endowments supported urban institutions and services. These are distinct mechanisms, not interchangeable versions of a treasury payment. [Durham World Heritage Site](https://www.durhamworldheritagesite.com/learn/architecture/cathedral/construction)

| Financing mechanism | TCE rule | Characteristic vulnerability |
| --- | --- | --- |
| Household gifts or labor pledges | Commit specified goods, money, or work periods | Contributions fall when confidence, harvests, or affiliation deteriorate. |
| Patron’s estate income | Transfer rents or estate output into a project account | Patron death, succession disputes, or estate losses. |
| Taxation or tribute | Institution creates an enforceable revenue claim | Collection costs, evasion, opposition, competition with military expenditure. |
| Compulsory labor service | Households owe labor, substitutes, or an equivalent payment | Agricultural disruption, flight, unequal burdens. |
| Endowment | Assets generate revenue reserved for construction or operation | Asset seizure, falling income, diversion of funds. |
| Credit or commercial investment | Current spending creates future payment claims | Debt service and disappointing future revenue. |

A project may combine all of these. Construction finance and operating finance should also be separate: a sponsor can afford the building but fail to endow its upkeep.

### 1.7 Completion is not a single event

Large religious complexes can have an operational core, later towers, additional shrines, replacement roofs, and changing plans. The Chola temple record explicitly preserves later additions and continued ritual use; cathedral construction histories likewise contain successive campaigns rather than one uninterrupted job. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/250/)

**Implementation:** represent a project as a graph of independently usable phases. A sanctuary, burial chamber, terrace, or assembly court can become functional before the entire intended complex is finished.

Track at least four dates: **proposal, work commencement, first use, and completion of the current plan**. A later enlargement is a new project phase, not evidence that the original building took centuries of continuous labor.

### 1.8 Maintenance and rebuilding can be the institution’s purpose

At Ise, the prescribed cycle rebuilds sanctuaries every **20 years**, with preparations and associated rituals extending over approximately **eight years**. At Djenné, annual replastering renews both the mosque’s protective surface and community participation. Longevity therefore need not mean retaining the same physical fabric. [Ise Jingu](https://www.isejingu.or.jp/en/ritual/index.html)

**Implementation:** separate physical condition from continuity of identity. An institution can preserve “the same” sacred place through repeated replacement, while a physically intact abandoned building can lose its former significance.

---

## 2. Quantitative parameters: costs, labor, and duration

### 2.1 How to interpret the estimates

Use these confidence labels:

**H:** strong documentation for the stated quantity and scope.  
**M:** reasoned reconstruction with significant uncertainty.  
**L:** strongly assumption-dependent or incompletely documented.  
**P:** proposed simulation parameter, not a historical estimate.

A historical *person-day* is not automatically an eight-hour day. Preserve the source’s unit and assumptions; convert to person-hours only when workday duration is specified or explicitly assumed.

Also distinguish **total work**, **simultaneous workforce**, and **elapsed time**. Multiplying a reported peak workforce by the entire construction chronology usually produces a misleading total.

### 2.2 Empirical anchors

| Project or institution | Quantitative anchor | What it measures—and excludes | Confidence and source |
| --- | --- | --- | --- |
| **Aguada Fénix main plateau, Mexico** | **3.2–4.3 million m³** of artificial fill; **10–13 million person-days**; principal construction dated **1000–800 BCE** | Published 2020 reconstruction of the plateau. Labor derives from assumed excavation and transport rates. The dating window is not 200 years of continuous employment. It is not a cost estimate for the entire landscape described in 2025. | **M** volume; **L–M** labor. Inomata et al. 2020. [PasoLibre](https://pasolibre.grecu.mx/wp-content/uploads/2020/07/41586_2020_2343_opt.pdf) |
| **Giza pyramid workforce, Egypt** | One AERA provisioning model considers **8,000–10,000 workers** | A workforce assumption used to investigate food supply—not a census, a total labor estimate, or an established peak for Khufu’s pyramid. | **L–M** as a workforce anchor. Redding and Hunt/AERA. [AERA](https://aeraweb.org/pyramids-and-protein/) |
| **Great Pyramid of Khufu** | Original height **146.5 m**; Egyptian antiquities authority gives **10–20 years** as an estimated construction duration | Monument dimensions are much firmer than construction chronology or workforce reconstruction. | **H** height; **L–M** duration. [Egyptian Monuments](https://egymonuments.gov.eg/en/monuments/the-great-pyramid) |
| **Salisbury Cathedral, England** | Main body built **1220–1258: 38 years** | Excludes later tower and spire work. Useful as a campaign-duration anchor, not a total person-day estimate. | **H** chronology for the stated scope. [Smarthistory](https://smarthistory.org/salisbury-cathedral/) |
| **St Paul’s Cathedral, London** | Main rebuilding **1675–1711**; directly hired general labor reached approximately **30,000 person-days/year** in **1705–1709** | Annual payroll evidence for one labor category. Excludes the complete labor input of skilled trades and subcontractors. | **H** within the surviving accounting scope. Paker, Stephenson and Wallis 2023. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/job-tenure-and-unskilled-workers-before-the-industrial-revolution-st-pauls-cathedral-16721748/E7593739FE3F57B9900E813D7D160175) |
| **Brihadisvara Temple, Thanjavur, India** | Construction possibly inaugurated **1003–1004**, consecrated **1009–1010**: roughly **six years** | A substantial royal temple could reach consecration within one reign and a relatively short campaign. Later additions are separate. | **M–H** chronology; inauguration qualified by the source. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/250/) |
| **Süleymaniye complex, Istanbul** | Construction **1550–1557**; reported total **53,841,000 silver akçe** | İnalcık compares the total with approximately **one-tenth of annual public revenues in 1527–1528**. This is an earlier fiscal comparator, not 10% of GDP or 10% of revenue every construction year. | **M–H** reported accounts; **M** cross-year burden comparison. [Archnet](https://www.archnet.org/sites/2024) |
| **Eiffel Tower, Paris** | **2 years, 2 months, 5 days**; **150–300 onsite workers**, **150 factory workers**, **7,300 tonnes of iron** | Demonstrates industrial prefabrication and concentrated assembly. Crew figures do not include the full upstream mining, ironmaking, transport, or design workforce. | **H** documented project figures. [La tour Eiffel](https://www.toureiffel.paris/en/the-monument/history) |
| **Sydney Opera House, Australia** | Construction **1959–1973**, approximately **14 years**; estimated cost rose from **£A3.5 million in 1959** to **£A13.7 million by mid-1962** | The roughly **3.9-fold** increase compares early estimates, not initial and final actual cost. The building was still unfinished. | **H** chronology and reported estimates. [Sydney Opera House](https://www.sydneyoperahouse.com/our-story/construction-begins) |

**There is no defensible universal “cathedral cost” or “pyramid cost.”** Different estimates count different boundaries: a main structure, its entire precinct, temporary works, quarrying, provisioning, decoration, or subsequent additions.

Architectural energetics is most useful when construction is decomposed into operations rather than converted directly from visible size into workers. Abrams and Bolland explicitly connect this approach to operations management. [Springer](https://link.springer.com/article/10.1023/A%3A1021921513937)

### 2.3 Rates and organizational parameters worth importing

| Parameter | Quantitative value | Recommended use | Confidence/source |
| --- | --- | --- | --- |
| Earth excavation productivity | **2.6 m³/person-day** | Reproduce the Aguada Fénix estimate as a benchmark scenario; vary with soil, tools, and working conditions. | **L–M:** an adopted reconstruction rate. [PasoLibre](https://pasolibre.grecu.mx/wp-content/uploads/2020/07/41586_2020_2343_opt.pdf) |
| Earth transport productivity | **0.384 m³/person-day**, approximately **500 kg/person-day**, at an assumed average **500 m** carry | A case-specific carrying model—not a universal hauling coefficient. | **L–M:** Aguada Fénix assumptions. [PasoLibre](https://pasolibre.grecu.mx/wp-content/uploads/2020/07/41586_2020_2343_opt.pdf) |
| Combined excavation and transport | Approximately **3.0 person-days/m³** | Derived as \(1/2.6+1/0.384\). Excludes additional operations not represented by those two rates. | **Derived**, inheriting the above uncertainty. |
| Seasonal workforce variation | January headcount approximately **60% of July** | Benchmark for one construction labor market; derive TCE seasonality from climate and task mix. | **H**, St Paul’s series. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/job-tenure-and-unskilled-workers-before-the-industrial-revolution-st-pauls-cathedral-16721748/E7593739FE3F57B9900E813D7D160175) |
| Annual attendance at one employer | Median **145 days**; approximately **200 days** among workers active for more than two years | Useful for intermittent employment. These are not estimates of all employment across all employers. | **H**, St Paul’s series. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/job-tenure-and-unskilled-workers-before-the-industrial-revolution-st-pauls-cathedral-16721748/E7593739FE3F57B9900E813D7D160175) |
| Ritual rebuilding cycle | **20 years**, with approximately **8 years** of preparations and ceremonies | An authored institutional rule, independent of physical collapse. | **H**, Ise. [Ise Jingu](https://www.isejingu.or.jp/en/ritual/index.html) |
| Protective surface renewal | **Annual** replastering | Recurring material and communal labor demand, not complete reconstruction. | **H**, observed Djenné practice. [AP News](https://apnews.com/article/991059d6df7b671ce9c9f92a2f247d05?utm_source=chatgpt.com) |

The Aguada rates illustrate an important mechanism: **approximately 87% of the combined excavation-and-carrying labor comes from transport**. That percentage is calculated from this particular model; it is not a universal ancient construction cost share.

### 2.4 Compute real cost before monetary cost

For each task, distinguish:

\[
L\_{\text{project}}=
L\_{\text{extraction}}
+L\_{\text{processing}}
+L\_{\text{transport}}
+L\_{\text{temporary works}}
+L\_{\text{assembly}}
+L\_{\text{finishing}}
+L\_{\text{support}}.
\]

Support includes activities such as administration, tool repair, cooking, and maintaining temporary accommodation.

However, **avoid double counting**. Either price purchased materials and count direct project labor, or expand the material supply chains into embodied labor and resources. Do not count both the purchase price and every upstream input as additional social costs.

Likewise, workers need food whether or not they build monuments. The economic burden includes displaced production, transport, and any additional provisioning requirements—not automatically the entire food consumption of every worker as a new net cost.

For comparisons across societies, retain three separate measures:

\[
\text{Labor burden}=
\frac{\text{annual project labor}}
{\text{annual available labor}},
\]\[
\text{Fiscal burden}=
\frac{\text{annual project expenditure}}
{\text{annual institutional revenue}},
\]\[
\text{Household burden}\_i=
\frac{\text{contribution demanded from household }i}
{\text{its disposable resources}}.
\]

These denominators answer different questions. A modest fiscal burden can coexist with severe burdens on particular households.

### 2.5 Scale checks for TCE

**Illustrative scenario, not a historical estimate:** a population of 10,000 contains 5,000 eligible participants, each contributing 20 days annually. That supplies **100,000 person-days/year**. A project requiring one million person-days then needs **ten fully active years**, before considering task bottlenecks, interruptions, or additional support.

For a pyramid-scale program, **10,000 workers × 200–300 days/year × 20 years = 40–60 million person-days**. This is a transparent planning scenario, **not a confidence interval for Khufu’s pyramid**.

The implication is not that large monuments are impossible at TCE’s scale. It is that regional labor and food support must be genuinely represented. A monument should not silently create an off-map workforce or an unaccounted hinterland.

---

## 3. Variation across societies, eras, and regions

The following are comparative patterns and mechanism hypotheses, **not fixed stages every society passes through**.

| Context | Relevant evidence and motives | TCE implications |
| --- | --- | --- |
| **Foragers** | Poverty Point demonstrates substantial public construction among hunter-gatherers. Research on Mound A argues for rapid building and situational leadership rather than assuming a centralized state. | Permit episodic aggregation and collective earthworks without farming, taxation, or kingship. Permanent settlement size need not equal the participating population. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1002/gea.21430) |
| **Early farmers and pastoralists** | Lothagam North, Kenya, was a monumental cemetery used by early herders, with an estimated minimum of **580 burials** and no clear mortuary hierarchy. Early Mesoamerica provides large communal ceremonial landscapes. | Burial obligations, shared identity, ecological uncertainty, and regional networks can motivate projects. Avoid equating monumental scale with an elite palace economy. [Stony Brook University](https://researchconnect.stonybrook.edu/en/publications/a-monumental-cemetery-built-by-eastern-africas-first-herders-near/) |
| **Preindustrial kingdoms and empires** | Giza links royal afterlife projects to regional provisioning. Chola temples connect kingship, religious institutions, and enduring worship. Süleymaniye combines imperial representation with a larger institutional complex. | Model rulers, temples, estates, and tax systems as overlapping resource holders. Projects can compete with warfare while also creating durable institutional infrastructure. [AERA](https://aeraweb.org/projects/lost-city/) |
| **Preindustrial corporate towns and religious communities** | Cathedral chapters, congregations, bishops, and paid craftspeople could all participate. Ise shows a different institutional solution: recurring renewal rather than maximizing survival of original fabric. | Allow multiple patrons, restricted funds, negotiated contributions, and specialist migration. Long continuity can reside in institutions and practices. [Durham World Heritage Site](https://www.durhamworldheritagesite.com/learn/architecture/cathedral/construction) |
| **Industrial societies** | The Eiffel Tower joined exhibition publicity, technical achievement, and national commemoration. Its relatively small onsite workforce depended on factory production and precise prefabrication. | Shift constraints toward capital goods, industrial supply chains, design, and assembly logistics. More height need not mean more onsite workers. [La tour Eiffel](https://www.toureiffel.paris/en/the-monument/history) |
| **Modern societies** | Sydney Opera House illustrates civic cultural ambition, uncertain design, political conflict, and prolonged construction. Modern commemorative landscapes also contain large numbers of comparatively small monuments. | Distinguish cultural facilities, memorials, governmental display, and commercial landmarks. Include publicity, professional project organizations, political oversight, and contested historical meanings. [Sydney Opera House](https://www.sydneyoperahouse.com/our-story/utzon-departs-the-house) |

The major regional lesson is **institutional diversity**, not a different universal construction coefficient for each civilization. The same physical capability can support a royal tomb, communal platform, living temple, repeatedly renewed wooden sanctuary, or civic exhibition structure.

### What can actually be said about frequency?

The literature supports occurrence across many forms of social organization. It does **not** provide a reliable global table saying that a foraging society builds a monument every X years, or that a kingdom builds Y times more monuments than a council-governed settlement.

Two datasets show both the possibilities and the limitations:

**Mesoamerica:** Inomata and colleagues identified **478 formal rectangular or square complexes**, probably dating approximately **1050–400 BCE**, across the surveyed Olmec and western Maya regions. This supports regional proliferation and transmission of architectural formats. It does not establish 478 independent polities or a known annual founding rate. [Nature](https://www.nature.com/articles/s41562-021-01218-1)

**Modern United States:** the National Monument Audit’s searchable study set contains **48,178 records from 42 data sources**. Its authors explicitly state that it is not a complete census. Its conventional monuments also differ substantially from the enormous construction projects emphasized in archaeological energetics. [Monument Lab](https://monumentlab.com/audit)

For TCE, measure frequency as **new starts per 100 settlement-years or polity-years**, separately by function and scale. Also record enlargement, replacement, abandonment, and demolition. Do not infer founding rates from surviving monument stock without accounting for preservation and chronological uncertainty.

---

## 4. Stylized facts a convincing simulation should reproduce

1. **Some very large projects emerge without strongly centralized rulers.**  
   Monumental investment must not be locked behind monarchy or bureaucracy. Equally, it should not prove that its builders were entirely egalitarian. [PubMed](https://pubmed.ncbi.nlm.nih.gov/32494009/)
2. **Construction occurs in campaigns and can remain useful between them.**  
   At Ceibal, successive floors and construction phases support repeated episodes. The study’s hypothetical 20-year building cycle is a modeling assumption, not an observed universal schedule. TCE should generate bursts, pauses, additions, and partial use. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0221943)
3. **A small stable core can supply much of the labor despite substantial turnover.**  
   At St Paul’s, approximately **12%** of the studied general laborers—those serving more than five years—provided **over 60%** of labor days. Projects should retain experienced crews rather than perpetually recruit interchangeable workers. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/job-tenure-and-unskilled-workers-before-the-industrial-revolution-st-pauls-cathedral-16721748/E7593739FE3F57B9900E813D7D160175)
4. **Visible scale is a poor guide to onsite headcount or completion time.**  
   The Eiffel Tower’s documented **150–300 onsite workers** depended on factory preparation. An advanced economy can produce an enormous structure with fewer visible workers while employing many people upstream. [La tour Eiffel](https://www.toureiffel.paris/en/the-monument/history)
5. **Institutions can outlive patrons and buildings can outlive their original political setting.**  
   The Chola temples preserve continued worship and later additions across major changes in their surroundings. TCE should allow ownership, patronage, and meaning to change independently of physical survival. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/250/)
6. **Maintenance can be socially productive rather than merely a tax on prestige.**  
   Ise’s 20-year renewal and Djenné’s annual replastering should be representable as recurring collective events that also consume real resources. [Ise Jingu](https://www.isejingu.or.jp/en/ritual/index.html)
7. **Public reactions can be divided and can change over time.**  
   The Eiffel Tower attracted organized artistic opposition before becoming widely celebrated. A correct model must permit admiration, hostility, indifference, and later reinterpretation—not a uniform completion bonus. [La tour Eiffel](https://www.toureiffel.paris/en/the-monument/history)
8. **Ambition can outrun knowledge and budgets.**  
   Sydney Opera House’s escalating estimates and political disputes show why institutional commitment should not guarantee timely completion. Delay can arise from technical uncertainty and governance, not just missing stone or money. [Sydney Opera House](https://www.sydneyoperahouse.com/our-story/construction-begins)

---

## 5. Recommended TCE implementation

### 5.1 Represent projects, institutions, and meanings separately

A practical data division is:

| Entity | Minimum state |
| --- | --- |
| **Construction project** | Plan version, task graph, bills of materials, labor by skill, deliveries, contracts, work fronts, forecast and actual costs |
| **Patron coalition** | Members, pledged contributions, control rights, credit allocation, disagreements, enforcement powers |
| **Owning institution** | Assets, revenue claims, obligations, succession rules, ritual calendar, maintenance policy |
| **Physical complex** | Components, structural condition, usable capacity, visibility, access rules, completed and unfinished phases |
| **Agent interpretation** | Association with sponsor or group, perceived importance, remembered participation, perceived burden, approval or resentment |

Keep the construction project separate from the enduring complex: one temple can accumulate many projects over centuries.

### 5.2 Let proposals emerge from ordinary agent goals

Useful proposal triggers include a growing congregation, crowded gathering space, a ruler’s concern for burial or succession, a remembered victory, an anniversary, competition with another settlement, or an existing structure’s deterioration.

These should be **agent interpretations of events**, not scripted historical milestones.

A proposal becomes actionable when a coalition authorizes it and establishes credible commitments. Agents need not forecast correctly. Overoptimistic patrons should sometimes begin projects that later stall.

Religious motivation should have its own value in the decision model. Do not implement every sacred project as political manipulation with decorative religious language.

### 5.3 Reuse normal construction—but preserve bottlenecks

For task \(s\), express all limiting rates in the same units of completed work per day:

\[
\dot Q\_s =
\min\left(
R\_{\text{crew}},
R\_{\text{material delivery}},
R\_{\text{lifting}},
R\_{\text{available work fronts}}
\right).
\]

This prevents adding 5,000 workers to a task that only has room for fifty.

The monument’s grammar should generate both permanent components and required temporary works: ramps, scaffolding, centering, lifting positions, access roads, workshops, and storage. Otherwise, monumental construction becomes implausibly easy precisely where ordinary construction rules should become most restrictive.

**Do not simply enlarge every dimension of a successful small building.** Generate larger courts, additional bays, thicker supports, different roof systems, and phased precincts through the same capability constraints used elsewhere in TCE.

This task-based recommendation follows the logic of architectural energetics and operations-management approaches, rather than a single “monument construction speed” multiplier. [Springer](https://link.springer.com/article/10.1023/A%3A1021921513937)

### 5.4 Model political effects through observed experiences

A simple update structure is:

\[
\Delta A\_{i,p} =
E\_{i,j}
\left[
\text{identity fit}
+\text{valued services}
+\text{perceived competence}
+\text{fairness of contribution}
-\text{experienced burden}
-\text{perceived failure}
\right],
\]

where \(A\_{i,p}\) is agent \(i\)’s evaluation of patron \(p\), and \(E\_{i,j}\) represents exposure to project \(j\).

This is a **proposed behavioral model**, not an estimated historical equation.

Exposure can come from seeing the building, attending ceremonies, working on it, paying toward it, receiving its services, or hearing about it. Credit may go to the ruler, deity, local community, architect, or institution. Different agents should attribute it differently.

Use diminishing returns. Ten nearly identical monuments should not yield ten times the influence. Novelty, rivalry, accumulated sacred association, and actual use should matter.

### 5.5 Treat upkeep, abandonment, and reuse as competing decisions

After initial use begins, institutions choose between maintenance, enlargement, replacement, reduced operation, and abandonment. Successors can continue a predecessor’s project, alter its dedication, appropriate its imagery, dismantle it, or leave it unfinished.

Do not make abandonment automatically mean social collapse. It can be a sensible reallocation, a political repudiation, a changed ritual practice, or loss of a particular revenue source. Conversely, continuing a prestigious project during scarcity can be politically attractive but economically damaging.

The model should produce those outcomes from competing commitments rather than choosing one universal monument lifecycle.

### 5.6 Initial tuning values: keep assumptions visibly separate

These are **engineering starting points for sensitivity testing**, not historical averages.

| Parameter | Initial test range | Status |
| --- | --- | --- |
| Ordinary voluntary construction contribution | **0–30 person-days per eligible participant per year** | **P.** Evaluate against household opportunity costs; test larger episodic gatherings separately. |
| Specialist annual availability | **150–250 workdays/person/year** | **P.** Scheduling prior, not a universal preindustrial work year. |
| Institutional budget review | Every **1–3 months** | **P.** Computational and behavioral cadence. |
| Procurement/crew productivity uncertainty | Initially test **0.5–2×** the benchmark rate | **P.** Sensitivity envelope, not a confidence interval. |
| Planned material or provisioning buffer | **1–3 months** of the relevant project requirement | **P.** Risk-management policy; institutions can choose less or miscalculate. |
| Construction progress update | Daily, with weekly task-allocation review | **P.** Suitable separation between visible work and higher-level planning. |

Prefer endogenous policies over permanent constants. A temple with reliable estate income should choose differently from a temporary coalition dependent on a single harvest.

### 5.7 Performance at 10k–50k agents

Keep people individual, but aggregate work into task batches rather than simulating every stone as an autonomous object.

Use household-level contribution commitments, named specialist crews, and institution-level procurement contracts. Daily work packets can debit actual agents’ time and produce quantities of completed work. UE5 can display the corresponding deliveries, scaffolding changes, and component completion.

Update most interpretations at relevant events—attendance, taxation, visible milestones, accidents, dedication—rather than recalculating every citizen’s reaction to every monument each tick.

### 5.8 Existing models and games worth borrowing from

**Architectural energetics:** Abrams and Bolland provide the strongest methodological foundation for translating construction into labor operations. Borrow task decomposition and explicit scope, not apparent precision in uncertain ancient totals. [Springer](https://link.springer.com/article/10.1023/A%3A1021921513937)

**Comparative archaeological models:** the Ceibal study demonstrates how to combine volumes, construction phases, productivity assumptions, and alternative participating populations. Its explicit warning that true labor requirements may lie outside the calculated low–high estimates is worth adopting directly in TCE’s calibration documentation. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0221943)

**Foundation:** its modular monument construction, production chains, and distinct institutional progression paths are useful design precedents. Borrow composition from ordinary parts and the connection between architecture and institutions; TCE should make initiation and patronage autonomous rather than solely player-directed. [Steam Store](https://store.steampowered.com/app/690830/Foundation/)

**Pharaoh / Pharaoh: A New Era:** use it as a precedent for making monument building a central economic objective within a settlement simulation. For TCE, replace predetermined mission objectives with proposals generated by institutions and people. [Steam Store](https://store.steampowered.com/app/1351080/Pharaoh_A_New_Era/?utm_source=chatgpt.com)

---

## 6. Sources, datasets, and unresolved questions

### Highest-value research foundations

| Source | Why it matters for TCE |
| --- | --- |
| **Trigger (1990), “Monumental architecture: A thermodynamic explanation of symbolic behaviour,” *World Archaeology*** | Foundational argument connecting exceptional expenditure and social power. Treat it as an explanatory theory, not a universal causal law. [Taylor & Francis Online](https://www.tandfonline.com/doi/abs/10.1080/00438243.1990.9980135) |
| **Glatz & Plourde (2011), “Landscape Monuments and Political Competition in Late Bronze Age Anatolia,” *BASOR*** | Explicit costly-signaling treatment of political competition and landscape placement. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.5615/bullamerschoorie.361.0033) |
| **Kantner & Vaughn (2012), “Pilgrimage as costly signal,” *Journal of Anthropological Archaeology*** | Cooperation, religious participation, and interaction beyond immediate kin groups. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416511000535) |
| **Abrams & Bolland (1999), “Architectural Energetics, Ancient Monuments, and Operations Management”** | The construction-accounting framework: operations, labor, scheduling, and organization. [Springer](https://link.springer.com/article/10.1023/A%3A1021921513937) |
| **Vroom (2010), *Financing Cathedral Building in the Middle Ages*** | Detailed treatment of revenue institutions and the practical problem of sustaining construction. [Amsterdam University Press](https://www.aup.nl/en/book/9789089640352/financing-cathedral-building-in-the-middle-ages) |
| **Inomata and colleagues, Ceibal and Aguada Fénix studies, 2019–2025** | Quantitative reconstruction, changing architectural scope, communal construction, and regional architectural transmission. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0221943) |

### Data sources suitable for calibration work

**St Paul’s payroll reconstruction.** Paker, Stephenson and Wallis’s study provides named workers and long-run employment histories. Its replication archive is identified as **DOI 10.3886/E182784V1**. It is especially valuable for retention, intermittent attendance, and seasonality; it does not represent the entire construction workforce. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/job-tenure-and-unskilled-workers-before-the-industrial-revolution-st-pauls-cathedral-16721748/E7593739FE3F57B9900E813D7D160175)

**Olmec–Maya ceremonial-complex database.** The 2021 regional study makes its archaeological-site database available through the University of Arizona repository. It is suitable for spatial clustering, plan-family diffusion, and relationships between regional interaction and architectural form. [Nature](https://www.nature.com/articles/s41562-021-01218-1)

**National Monument Audit.** Useful for commemorative subjects, patronage questions, and the distinction between surviving records and a complete monument population. Its record categories should not be merged uncritically with ancient architectural complexes. [Monument Lab](https://monumentlab.com/audit)

**Published construction accounts and archaeological supplements.** Ottoman construction accounts, cathedral fabric accounts, and the Ceibal/Aguada supplements are valuable inputs, but require explicit extraction of scope, units, and assumptions before entering a common parameter database. [Scribd](https://www.scribd.com/document/920820864/Halil-%C4%B0nalc%C4%B1k-Cemal-Kafadar-Suleyman-the-Second-and-His-Time-Gorgias-Press-2019)

### Where the evidence remains thin or contested

**Ancient workforce totals** are generally less secure than physical dimensions. Different assumptions about productivity, carrying distance, natural substrate, participation, and work seasons can change estimated burdens substantially.

**Political interpretation** is not uniquely recoverable from size. A large platform can reflect centralized command, broad cooperation, or a mixture. Expensive construction establishes neither consent nor coercion by itself.

**Frequency by society type** remains inadequately standardized. The available regional surveys and monument inventories lack a common denominator of comparable societies, observation periods, and preservation probabilities.

**Net social returns** are also difficult to quantify. Buildings may supply services, coordination, identity, and livelihoods, but there is no transferable empirical coefficient such as “one million person-days produces 15 legitimacy.”

**The strongest TCE design is therefore a resource-constrained, coalition-driven construction system with audience-specific consequences.** Let grandeur emerge from accumulated commitments, skills, institutions, and meanings. The physical building should be the visible result of that history—not a shortcut around it.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9291e-4538-83ea-a33f-9531e7a8a11c)
