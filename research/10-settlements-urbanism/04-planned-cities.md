# Planned cities and planning traditions: a simulation-ready report for TCE

## Executive conclusion

**Treat planning as an institution’s ability to coordinate land, infrastructure and building rights—not as a binary “planned city” flag or a technology that unlocks grids.** A settlement can have a planned ceremonial center, surveyed residential districts, informally subdivided outskirts and an inherited road network simultaneously. Archaeological approaches that distinguish coordination, standardization, access and monumental arrangement are more useful for TCE than the traditional planned-versus-organic classification. [Sage Journals](https://journals.sagepub.com/doi/10.1177/1538513206293713)

The essential distinction is between four things:

**The prescribed ideal → the legally reserved layout → what people actually build → what survives later.**

These can diverge substantially. Chinese capital prescriptions did not produce one invariant layout; bastide documents sometimes specified larger plots than were realized; and Washington’s plan remained influential after its designer’s dismissal. [J-STAGE](https://www.jstage.jst.go.jp/article/pjab/93/9/93_PJA9309B-04/_html/-char/en)

For TCE, the most productive architecture is therefore **institutionally selected spatial templates, implemented in phases, followed by household adaptation and costly enforcement**. The historical examples below should supply bounded templates and test cases, not a predetermined sequence of civilizations.

---

## 1. Mechanisms: rules the simulation can implement

The following are **modeling recommendations derived from the comparative evidence**, not estimated universal historical equations.

### 1.1 Planning begins when an actor can capture benefits from coordination

A ruler, council, army, religious institution or group of landowners should propose a plan when expected benefits exceed its own expected costs. Different actors value different outcomes: military deployment, land revenue, commercial frontage, ritual access, prestige, defensibility or attracting settlers.

Bastides illustrate the importance of revenue and settlement concentration: many were principally market-town investments rather than purpose-built fortresses. Washington illustrates a different objective—giving political institutions prominent, connected locations. [Cornell University Library Exhibits](https://exhibits.library.cornell.edu/bastides-collection/about/about-the-bastides)

**Implementable rule:** evaluate proposals from the sponsoring coalition’s perspective, not from an omniscient city-wide welfare function. A plan that benefits merchants may dispossess cultivators; a monumental avenue may benefit a ruler while burdening taxpayers.

### 1.2 Land assembly is a separate problem from drawing a plan

Separate the ability to propose a layout from the authority to reserve streets, redistribute parcels, acquire land and remove existing buildings.

**Implementable rule:** every planned component must identify its affected rights-holders and a valid acquisition mechanism: purchase, negotiated pooling, donation, customary allocation, conquest, confiscation or compulsory acquisition. Unresolved claims delay construction, distort the alignment or provoke resistance.

A competent surveyor working for a politically weak commission should produce a different outcome from an incompetent surveyor working for a powerful landowner.

### 1.3 Surveying establishes shared commitments

Planning requires a common orientation, measurement convention and rules for allocating space. It need not require modern instruments or universal literacy.

**Implementable rule:** a survey creates stakes, boundaries and recorded or socially recognized claims. Survey skill affects alignment error and the consistency of repeated modules. Durable records improve recovery after disputes or destruction; local witnesses and boundary markers can support smaller schemes without a bureaucracy.

Do not give every culture a global “foot.” Funo’s comparative study shows why local measurement systems matter to the interpretation of apparently similar grids. [J-STAGE](https://www.jstage.jst.go.jp/article/pjab/93/9/93_PJA9309B-04/_html/-char/en)

### 1.4 Reserve first, construct selectively

A reservation for a street is not a paved street. A proposed plaza is not an occupied market.

**Implementable rule:** maintain separate values for legal reservation, ground clearance, drainage, road surface, installed services and adjacent occupancy. Release new phases when finance, labor and demand permit—not immediately when a plan is adopted.

Plans can therefore be ambitious but mostly empty, cheaply marked out, partially serviced, or abandoned. Their unbuilt portions should remain vulnerable to cancellation and occupation.

### 1.5 Allocation rules shape society inside the geometry

The same grid can accommodate equal initial allotments, privileged estates, military contingents, rental housing or large collective residences.

Polybius describes camp space allocated by military units and their strength. Teotihuacan’s residential compounds, by contrast, contained many rooms and multiple households rather than corresponding to ordinary single-family plots. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Polybius/6%2A.html)

**Implementable rule:** store the allocation system independently of geometry. Record owner, occupier, household membership, transfer restrictions, subdivision rights and obligations to build. Regular parcels should not automatically imply equal wealth.

### 1.6 Compliance depends on detection, adjudication and execution

A law is not self-enforcing. Montgéard’s foundation involved appointing an officer to oversee orderly realization; Tang authorities repeatedly prohibited private openings through ward walls and demanded removal of encroachments. The repetition is evidence that prescription and practice differed. [University of Virginia](https://pure.uva.nl/ws/files/994564/75683_11_CH07_Planners_def_eng_vt_VERSIE2.pdf)

A useful behavioral approximation is:

\[
\text{Expected sanction}
=
P(\text{detection})
P(\text{adverse ruling}\mid\text{detection})
P(\text{execution}\mid\text{ruling})
(\text{fine}+\text{demolition loss})
\]

An agent encroaches when the expected benefit exceeds this cost, plus any reputational or normative cost. Political connections can affect adjudication and execution independently of inspection frequency.

### 1.7 Persistence and erosion should be consequences, not timers

**Implementable rule:** replacing a street or boundary incurs demolition, compensation, coordination and lost-access costs. Reusing it avoids those costs. This creates persistence even after the founding institution disappears.

Conversely, pressure for additional floor space, frontage and shortcuts encourages local modification. Institutions may demolish violations, tolerate them, legalize them or charge for them. In Song Kaifeng, authorities eventually collected rent for some roadway infringements instead of insisting on demolition—a policy change, not simply an automatic collapse of government. [Urbana](https://urbanauapp.org/index.php/urbana/issue/download/16/54)

---

## 2. The principal planning traditions

### 2.1 Hippodamian and other Greek grids

“Hippodamian” should identify a family of orthogonal arrangements, **not an invention event or evidence that Hippodamus personally designed a site**. Greek grid planning preceded him. Priene provides a useful example of coordinated streets, residential allotments and reserved civic or sacred spaces adapted to a steep site. [J-STAGE](https://www.jstage.jst.go.jp/article/pjab/93/9/93_PJA9309B-04/_html/-char/en)

Institutions could establish the subdivision while households subsequently changed buildings and property arrangements. Cahill’s study of Olynthus is especially valuable because it follows household organization within a regular urban framework rather than treating the plan as a complete description of urban life. [OUP Academic](https://academic.oup.com/yale-scholarship-online/book/15363)

**TCE translation:** generate a surveyed street-and-parcel framework with separately reserved public sites. Allow terraces, stairs, merged plots, subdivision and unequal household investment. Neither democracy nor monarchy should be a prerequisite for the geometry.

### 2.2 Roman castra and colonial towns

Keep three systems distinct: **military encampments, permanent urban settlements and agricultural cadastral divisions**.

Polybius’s camp is an organizational diagram made physical: headquarters, supplies, market, troop units, circulation routes and a defensive clearance zone. Its dimensions change with the participating forces. This is an unusually explicit example of institution-to-space translation. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Polybius/6%2A.html)

Timgad, founded under Trajan in 100 CE, supplies a contrasting urban case: an orthogonal foundation subsequently expanded beyond its original limits. By the middle of the second century, growth was already exceeding the initial framework. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/194/)

**TCE translation:** derive camps from force composition and command needs. Derive colonial towns from allotment, public-building and settlement programs. Do not automatically convert every camp into a town, or use large agricultural survey squares as residential blocks.

### 2.3 Chinese capital planning and the *Kaogong ji*

The *Kaogong ji* offered a prestigious normative arrangement of walls, gates, roads, court, ritual institutions and markets. Its short description permits competing reconstructions; it is not a complete engineering specification. Chang’an’s north-palace configuration and later capital arrangements were variants, not exact copies of one diagram. [J-STAGE](https://www.jstage.jst.go.jp/article/pjab/93/9/93_PJA9309B-04/_html/-char/en)

Tang Chang’an also combined physical planning with access regulation: wards, gates, curfews and restrictions on doors opening directly onto major streets. Exemptions for high-ranking occupants show that enforcement was socially differentiated. [Urbana](https://urbanauapp.org/index.php/urbana/issue/download/16/54)

**TCE translation:** use a capital-planning package containing spatial hierarchy, ceremonial orientation, public compounds and access rules. Let institutions reinterpret inherited principles and selectively abandon restrictions without having to erase the major streets.

### 2.4 Teotihuacan

Teotihuacan combined a strongly ordered monumental axis, a shared orientation and extensive residential compounds. However, a single comprehensive master plan and a sequence of projects following common principles are competing interpretations of its development. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/teotihuacan)

For modeling, this is important: a coherent city can result from **successive institutions respecting a durable spatial convention**, not only from one founding blueprint. Royal, collective and mixed decision structures are reasonable alternative scenarios to test rather than assigning an invented, securely identified planner.

**TCE translation:** establish an orientation convention and ceremonial network early; add large residential compounds in later phases. Give compounds internal household organization, courtyards and controlled entrances rather than treating them as oversized detached houses.

### 2.5 The Laws of the Indies

The 1573 ordinances provided a Crown-authorized framework for settlement: site selection, surveying, plazas, streets, allotments and building obligations. They placed an inland town’s principal plaza centrally, but a coastal town’s plaza near the landing place. They also anticipated future growth. [HUD User](https://www.huduser.gov/portal/sites/default/files/pdf/The-Laws-of-the-Indies.pdf)

These rules should be treated as **prescriptions whose implementation requires local institutions**, not as evidence that every colonial settlement followed one exact plan.

**TCE translation:** create a legal foundation package administered by a governor, founder or council. Its clauses can specify public reservations, allocation procedures, deadlines and frontage requirements, while local terrain and competing interests produce departures.

### 2.6 Bastides

Bastides in southwestern France were commonly organized around a commercial square, with relatively regular streets and narrow house plots. Churches often stood beside the commercial focus or one or two blocks away. Many fortifications were later additions, so “bastide” should not imply a fully fortified foundation. [Cornell University Library Exhibits](https://exhibits.library.cornell.edu/bastides-collection/about/about-the-bastides)

Founding could involve shared lordship, monasteries, royal officials and delegated administrators. Boerefijn’s evidence distinguishes founders, supervising officials and people executing the layout—roles that popular accounts often collapse into a single architect. [University of Virginia](https://pure.uva.nl/ws/files/994564/75683_11_CH07_Planners_def_eng_vt_VERSIE2.pdf)

**TCE translation:** allow a landowner–ruler–religious coalition to exchange land, privileges and administrative support for future rents and market revenue. Commercial attraction and vacant-lot uptake should determine whether the scheme succeeds.

### 2.7 Baroque and Baroque-derived radial plans

The key spatial operation is **connecting important destinations with long axes, converging avenues and designed squares**, often superimposed on a more ordinary street network.

Washington’s 1791 plan is a well-documented Baroque-derived example: prominent sites for government buildings, diagonal connections, monumental public spaces and an underlying grid. A commission supervised the work; conflict led to L’Enfant’s dismissal in 1792, but the plan continued to influence development. [Library of Congress](https://www.loc.gov/resource/hhh.dc0776.sheet/?sp=2&utm_source=chatgpt.com)

**TCE translation:** select important nodes first, then connect them through axial corridors and subdivide the residual land. This should create triangular parcels and awkward junctions as well as prestigious frontages. Political symbolism should influence the plan, but not eliminate terrain, finance or administrative conflict.

---

## 3. Parameters: dimensions, ranges and their evidential status

### Reading the tables

**P** means prescribed in a text or documented plan; **O** means observed or surveyed; **R** means reconstructed or estimated.

**High confidence** means strong evidence for that particular case or prescription—not a universal standard. **Medium confidence** indicates reconstruction, approximate conversion or limited sampling. None of these examples establishes a worldwide probability distribution.

Distinguish a **house plot**, a **street block**, a **residential compound** and an **administrative ward** in the simulation schema.

### 3.1 Blocks, plots, compounds and wards

| Case and spatial object | Quantitative anchor | Status and confidence | Appropriate use |
| --- | --- | --- | --- |
| **Miletus: reconstructed residential block module** | **100 × 180 Ionian feet**, approximately **29.4 × 52.9 m** in Funo’s account; six lots | R; medium | One Greek module, not a universal Hippodamian block. [J-STAGE](https://www.jstage.jst.go.jp/article/pjab/93/9/93_PJA9309B-04/_html/-char/en) |
| **Monpazier: documented house plot** | Approximately **8 × 20 m**, or **160 m²** | P; high for documented module, medium for conversion | Narrow-frontage market-town allotment. [University of Virginia](https://pure.uva.nl/ws/files/994580/75691_19_appendix_C_lots_def_vt_Y_.pdf) |
| **Selected bastide house plots** | Examples span approximately **7.7–10 m frontage** and **19–24 m depth** | P/R; medium | A bounded sample for one regional family; not all bastides. [University of Virginia](https://pure.uva.nl/ws/files/994580/75691_19_appendix_C_lots_def_vt_Y_.pdf) |
| **Teotihuacan: apartment compound** | Typically about **61 × 61 m**; estimated **60–100 residents** | R; medium | Multi-household compound, not street-block or nuclear-family standard. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/teotihuacan) |
| **Chang’an: wards flanking the main axis** | Reported ranges **500–590 × 580–700 m** | R; medium | Large gated administrative/residential units. [Urbana](https://urbanauapp.org/index.php/urbana/issue/download/16/54) |
| **Chang’an: wards beside the palaces** | Reported ranges **600–883 × 1,020–1,125 m** | R; medium | A second ward family; preserve the association with location. [Urbana](https://urbanauapp.org/index.php/urbana/issue/download/16/54) |
| **Chandigarh: typical modern sector** | **800 × 1,200 m**, approximately **96 ha** | P; high | Neighborhood-scale cell containing smaller streets and building plots. [Chandigarh](https://chandigarh.gov.in/about-us) |

**Implementation consequence:** do not pool these dimensions into one “block size” distribution. The differences partly describe different spatial objects, not merely cultural taste.

### 3.2 Streets, axes and public spaces

| Case and parameter | Quantitative anchor | Status and confidence | Interpretation |
| --- | --- | --- | --- |
| **Polybius: camp circulation passages** | **50 ancient feet**; larger spaces **100 feet** | P; high for text | Approximately **15 and 30 m** under an explicitly chosen 0.30 m simulation foot; not ordinary town streets. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Polybius/6%2A.html) |
| **Polybius: clearance between tents and rampart** | **200 ancient feet** | P; high for text | Approximately **60 m** under the same convention; defensive and logistical open space. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Polybius/6%2A.html) |
| **Chang’an: reported street widths** | North–south streets **42–68 m**; east–west streets **39–55 m** in the surveyed examples discussed | O/R; medium | Broad capital streets; do not transfer to small agricultural towns. [J-STAGE](https://www.jstage.jst.go.jp/article/pjab/93/9/93_PJA9309B-04/_html/-char/en) |
| **Teotihuacan: ceremonial axis** | Approximately **40 m average width**, over a described **2.4 km ceremonial stretch** | O/R; medium | The cited stretch is not necessarily the full extent of every reconstruction of the avenue. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/teotihuacan) |
| **Teotihuacan: principal orientation** | Approximately **15.5° east of north** | O; high as a broad alignment | A shared convention, not proof every structure was exactly aligned. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/teotihuacan) |
| **L’Enfant: grand avenues** | **160 ft ≈ 48.8 m** total; **80 ft ≈ 24.4 m** central carriageway | P; high | Full reservation included planted walks and pedestrian space; do not model it all as carriageway. [Early American Landscape Design](https://heald.nga.gov/mediawiki/index.php/Pierre-Charles_L%E2%80%99Enfant) |
| **Laws of the Indies: principal plaza** | Minimum **200 × 300 feet**; preferred **400 × 600**; maximum **530 × 800**, as printed in the HUD translation | P; high for prescription | Not an observed range of completed colonial plazas. [HUD User](https://www.huduser.gov/portal/sites/default/files/pdf/The-Laws-of-the-Indies.pdf) |
| **Kaogong ji: ideal capital** | **9 × 9 li**; **3 gates per side**; **9 longitudinal and 9 transverse routes**; principal route width **9 carriage tracks** | P; high for textual numbers, lower for exact reconstruction | Keep native units and interpretive choices explicit. Do not substitute the modern 500 m li. [J-STAGE](https://www.jstage.jst.go.jp/article/pjab/93/9/93_PJA9309B-04/_html/-char/en) |

**Historical-unit caution.** The HUD translation prints “feet” without, by itself, settling every metrological question. Retain the source unit in the research record and bind a TCE template to a declared local unit. The approximate Roman-camp conversions above are explicitly a simulation convention, not a claim of centimeter-level archaeological accuracy.

### 3.3 Plaza spacing

There is **no defensible universal “one plaza every X meters” rule** across these traditions.

The Indies ordinances call for additional plazas distributed through the settlement, but do not specify a fixed interval. Bastide examples often emphasize one principal commercial focus, whereas a capital’s ceremonial spaces can serve different institutions and access groups. [HUD User](https://www.huduser.gov/portal/sites/default/files/pdf/The-Laws-of-the-Indies.pdf)

For TCE, distinguish **commercial catchments**, **ceremonial assembly capacity**, **administrative forecourts** and **neighborhood common space**. A market square should respond to anticipated attendance and access; a restricted palace forecourt should not count as universally available public space.

### 3.4 Explicitly authored starting parameters

These are **engineering and sensitivity-test defaults, not historical estimates**.

| Parameter | Suggested initial test values | Purpose |
| --- | --- | --- |
| Candidate layouts evaluated per proposal | **3–8 alternatives** | Bounded institutional decision-making |
| Initial residential release phase | **10–50 plots** | Prevent instantaneous completion of large plans |
| Routine institutional review interval | **30–90 simulated days** | Keep expensive planning decisions off the daily person update |
| Local-market access target | **5–10 minutes** | At an assumed 1 m/s, **300–600 m network distance**; not straight-line spacing |
| Inspection intensity | **0.02, 0.2, 1 inspection per eligible parcel-year** | Low, medium and high enforcement-capacity experiments |
| Illustrative compact grid for testing | **40 × 80 m net blocks; 6 m streets** | A neutral test fixture, not a historical cultural preset |

There is insufficient comparative evidence to assign universal historical values to inspector productivity, annual encroachment rates, plan-adoption probabilities or the share of municipal revenue devoted to implementation.

---

## 4. Variation across eras and world regions

**These are available configurations, not an obligatory progression.**

| Context | Evidence and variation | TCE implication |
| --- | --- | --- |
| **Foragers and hunter-fisher-gatherers** | Poverty Point’s builders created six semi-elliptical ridges around a central area; the outer arrangement spans roughly **1.14 km**. This was a hunter-fisher-gatherer achievement, not an agricultural city-state’s work. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1435/) | Permit coordinated assembly landscapes before farming or bureaucratic government. Do not make this exceptional scale typical of all foragers. |
| **Early farming settlements** | Repeated house forms and aligned neighborhoods can arise through household coordination. Smith cautions that local regularity is not sufficient evidence for comprehensive central planning. [UW Faculty](https://faculty.washington.edu/plape/citiesaut11/readings/Journal%20of%20Planning%20History-2007-Smith-3-47.pdf) | Allow kin groups and neighborhood institutions to coordinate compounds, access and common space without a city-wide plan. |
| **Early urban agrarian societies** | Dholavira combined differentiated fortified sectors, ceremonial ground, reservoirs and drainage. Its configuration changed over a long occupation; it was not a frozen blueprint. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1645/) | Let water management and existing sector boundaries structure development. Do not require Greek-derived grids as the origin of planning. |
| **Pre-industrial Africa and the Americas** | Yoruba palace arrangements and Great Zimbabwe’s enclosed spaces illustrate planning through access hierarchy and compounds, not necessarily a city-wide orthogonal street system. Teotihuacan demonstrates a different, strongly oriented urban tradition. [UW Faculty](https://faculty.washington.edu/plape/citiesaut11/readings/Journal%20of%20Planning%20History-2007-Smith-3-47.pdf) | Include courtyard, enclosure and controlled-access templates alongside street grids. |
| **Pre-industrial South and East Asia** | Jaipur combined a planned commercial capital, public squares and coordinated bazaar fronts; its major infrastructure and royal/public spaces were established in **1727–1731**. East Asian capitals also transmitted and adapted planning conventions rather than independently inventing every layout. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1605/) | Planning knowledge should diffuse through specialists, texts and political imitation. Institutions can standardize public fronts while households build behind them. |
| **Industrializing cities** | Manhattan’s 1811 commission mapped **155 numbered streets and 12 avenues** across existing terrain and holdings. Subsequent development modified the proposal rather than executing it unchanged. [NYPL Digital Collections](https://digitalcollections.nypl.org/collections/commissioners-plan-of-manhattan-island-and-report-with-related-materials?filters%5Bdivision%5D=MSS&keywords=&utm_source=chatgpt.com) | Add large-scale future reservations, professional commissions and long implementation horizons. Keep existing claims and later amendments active. |
| **Modern planned capitals and neighborhoods** | Chandigarh’s sectors and Brasília’s planned capital represent neighborhood-scale circulation design and large public planning programs rather than merely wider ancient grids. [Chandigarh](https://chandigarh.gov.in/about-us) | Add transport hierarchy, infrastructure networks and sector-level service planning without making older spatial forms disappear. |

A regional name should therefore influence the **available repertoire and institutional preferences**, not force every settlement into one shape.

---

## 5. Stylized facts and validation targets

### 5.1 Plans persist at different spatial levels

A correct simulation should allow streets, plot boundaries, buildings and access regulations to change at different rates.

The Teotihuacan lidar study found that approximately **65% of the urban areas within its survey contained modern properties or features aligned within three degrees of 15° east of north**. This is not “65% of all streets,” nor a count of surviving ancient buildings. It demonstrates enduring influence of the earlier geometry. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0257550)

**Test:** regime replacement must not automatically regenerate the street graph. Buried structures and existing access can continue affecting later construction.

### 5.2 A planned core can acquire differently organized extensions

Timgad exceeded its initial framework within roughly half a century of foundation. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/194/)

**Test:** extend a successful planned settlement beyond its first boundary. New districts should inherit some alignments, negotiate new ones and sometimes develop more irregularly.

### 5.3 Prescribed dimensions and realized dimensions can differ materially

Boerefijn records Villeréal plots prescribed at **4 × 12 cannes**, but realized at **3 × 10**. Holding the unit constant, the latter is **62.5% of the prescribed area**—a calculation from the reported dimensions, not a universal compliance rate. [University of Virginia](https://pure.uva.nl/ws/files/994580/75691_19_appendix_C_lots_def_vt_Y_.pdf)

**Test:** permit systematic implementation changes, not only random centimeter-scale survey noise.

### 5.4 Regular geometry does not determine household organization

Olynthus provides evidence for household and property variation within a regular framework. A city can remain visually ordered while its social and economic organization changes. [OUP Academic](https://academic.oup.com/yale-scholarship-online/book/15363)

**Test:** after several generations, identical initial allotments should be capable of supporting unequal wealth, subdivision and plot consolidation.

### 5.5 Relaxing a plan can be an adaptive political choice

Kaifeng’s changing street economy included legal acceptance of direct street commerce and charging for some encroachments rather than always removing them. [Urbana](https://urbanauapp.org/index.php/urbana/issue/download/16/54)

**Test:** distinguish unauthorized occupation, tolerated occupation and legally regularized occupation. All three can produce similar visible geometry but different political outcomes.

### 5.6 Planning has a large land and infrastructure cost

For an ideal rectangular grid with net block dimensions \(B\_x,B\_y\) and uniform street width \(w\), the street land fraction is:

\[
f\_{\text{street}}
=
1-\frac{B\_xB\_y}{(B\_x+w)(B\_y+w)}
\]

For **40 × 80 m blocks**, increasing streets from **6 to 12 m** raises this fraction from approximately **19.1% to 33.1%**, before plazas or other public reservations.

This is a geometric calculation, not a historical statistic. It gives TCE an immediate tradeoff: larger circulation reservations consume land, clearance labor and maintenance capacity. They should not be a free aesthetic improvement.

---

## 6. Recommended representation in TCE

### 6.1 Separate institutions, plans and built geometry

Use three persistent layers.

| Layer | Essential state |
| --- | --- |
| **Planning institution** | Members and offices; jurisdiction; decision procedure; objectives; treasury; labor access; survey knowledge; acquisition powers; inspection and judicial capacity |
| **Plan version** | Sponsor; adoption date; legal extent; orientation conventions; street reservations; public sites; subdivision rules; allocation rules; implementation phases; amendments |
| **Built and occupied landscape** | Actual paths and roads; surface/drainage condition; parcels and claims; buildings; entrances; barriers; construction progress; encroachments |

A plan component should be able to be **legally reserved, only partly constructed and partly occupied without permission at the same time**. These are separate attributes, not mutually exclusive states.

Keep plan versions after amendments. They provide causal history for disputes and an intelligible UI: “This avenue was reserved by the old council, narrowed during the crisis, then legalized by the successor government.”

### 6.2 Use composable spatial operations, not complete historical city blueprints

A compact v1 repertoire could include:

| Operation | What it produces |
| --- | --- |
| **Orthogonal subdivision** | Repeated blocks or plots along one or more baselines |
| **Axial connection** | A corridor connecting important destinations or views |
| **Radial branching** | Several corridors converging on a selected node |
| **Enclosure and ward subdivision** | Perimeter boundaries, controlled entrances and internal access |
| **Plaza-centered subdivision** | Frontage allocation around an open node, followed by connecting streets |
| **Courtyard or compound aggregation** | Nested shared and private spaces without requiring through-streets |
| **Boundary extension** | A new district fitted to old roads, terrain and surviving claims |

Institutions select and combine these operations. A town can begin with compounds, acquire a market grid, gain a palace axis, and later incorporate a gated district.

Cultural knowledge should influence proportions, orientation preferences, acceptable public/private arrangements and which combinations seem legitimate. It should not predetermine the outcome.

### 6.3 Model political choice explicitly but cheaply

For each proposed plan, estimate benefits and costs to relevant factions:

\[
\Delta U\_f(p)
=
\text{expected access, revenue, security and prestige benefits}
-
\text{land losses, taxes, labor obligations and disruption}
\]

The institution’s decision procedure converts faction preferences into a decision. A ruler may override opposition but still encounter refusal, sabotage, migration or fiscal limits. A council may approve only a compromised alignment.

Use imperfect forecasts. Underestimated growth, optimistic revenues and changing political support should create oversized reservations, delayed phases and unfinished public projects without scripted failure events.

### 6.4 Give households and other occupants consequential choices

Individual people remain the simulation’s population, but many property decisions should be made at household, compound or firm level.

Relevant choices include accepting an allotment, refusing relocation, renting frontage, building incrementally, opening a door, combining plots, subdividing inheritance, occupying unused reservations or financing neighborhood improvements.

The household should consider both physical access and institutional risk. An attractive plot without secure possession is not equivalent to the same plot with an enforceable title.

### 6.5 Keep the simulation kernel authoritative

For the stated 10k–50k-person scale, my recommendation is:

* Evaluate political planning proposals infrequently and on major events; update construction and occupation through event queues.
* Use a hierarchical movement graph: settlement connections, district streets and local entrances.
* Recompute only affected parcels and navigation regions after a plan amendment or construction event.
* Let Unreal render the kernel’s authoritative geometry rather than independently deciding where streets or entrances exist.

Large historical capitals should supply **organizational principles**, not mandatory settlement sizes. Do not shrink doorways, lanes or human-scale plots merely to fit a historically larger city into the agent budget; simulate a smaller settlement or a smaller implemented portion of an ambitious plan.

### 6.6 What to simplify in v1

The highest-value simplification is to preserve **land rights, reservations, access and implementation sequence** while simplifying architectural detail.

A minimal convincing system needs: a sponsoring institution, several layout alternatives, land assembly, public-space reservation, phased allocation, household construction, enforcement and amendment.

Detailed survey instruments, individual bureaucratic paperwork and full building-code simulation can wait. Conversely, omitting land acquisition or giving plans instant enforcement would remove much of the historical mechanism the system is meant to reproduce.

### 6.7 Existing models and games worth borrowing from

| Model or game | Useful contribution | Important limitation for TCE |
| --- | --- | --- |
| **Parish & Müller, “Procedural Modeling of Cities” (2001)** | Generates roads, lots and buildings through procedural rules combining global goals and local constraints. | Primarily a geometry-generation method; the social decision to impose a pattern is not its central model. Use it downstream of institutions. [CGL ETHZ](https://cgl.ethz.ch/Downloads/Publications/Papers/2001/p_Par01.pdf) |
| **UrbanSim / Waddell’s research program** | Separates households, employment, buildings, parcels, development and externally specified constraints. | Useful for development under rules, but TCE needs an additional model explaining how historical institutions choose and change those rules. [UrbanSim Cloud](https://cloud.urbansim.com/docs/general/documentation/technical.html) |
| **Foundation** | Useful reference for gridless settlement presentation, resource-driven growth and modular monuments. | Treat it as a design reference, not historical validation. Its player-directed development is different from TCE’s autonomous planning institutions. [Polymorph Games](https://www.polymorph.games/en/?utm_source=chatgpt.com) |

The central integration task is to put **political selection and implementation between the economy and the geometry generator**.

---

## 7. Sources, datasets and limits of confidence

### Core scholarly and primary-source reading

| Source | Principal value |
| --- | --- |
| **Michael E. Smith, 2007, “Form and Meaning in the Earliest Cities.”** *Journal of Planning History* 6(1): 3–47. | Comparative framework for identifying planning without equating it with grids. [Sage Journals](https://journals.sagepub.com/doi/10.1177/1538513206293713) |
| **Nicholas Cahill, 2002, *Household and City Organization at Olynthus*.** | Household adaptation and property organization within a regular plan. [OUP Academic](https://academic.oup.com/yale-scholarship-online/book/15363) |
| **Polybius, *Histories*, Book 6, especially §§27–33.** | Explicit spatial prescriptions connecting military organization, circulation and logistics. [Penelope](https://penelope.uchicago.edu/Thayer/E/Roman/Texts/Polybius/6%2A.html) |
| **Shuji Funo, 2017, “Ancient Chinese Capital Models—Measurement System in Urban Planning.”** | Metrology and competing reconstructions; distinguish reported measurements from the author’s proposed models. [J-STAGE](https://www.jstage.jst.go.jp/article/pjab/93/9/93_PJA9309B-04/_html/-char/en) |
| **Hang Lin, 2015, “From Closed Capital to Open Metropolis.”** *Urbana* XVI. | Tang–Song changes in access, commerce, enforcement and street occupation. [Urbana](https://urbanauapp.org/index.php/urbana/issue/download/16/54) |
| **Wim Boerefijn, 2010, *The Foundation, Planning and Building of New Towns in the 13th and 14th Centuries in Europe*.** | Institutional roles and documented versus reconstructed plot dimensions. [UvA DARE](https://dare.uva.nl/id/6ac40515-d4b1-49e7-8703-60b8aee1a791) |
| **1573 settlement ordinances, in HUD’s *The Laws of the Indies, Translated*.** | Direct prescriptions; evidence of legal intent rather than automatic compliance. [HUD User](https://www.huduser.gov/portal/sites/default/files/pdf/The-Laws-of-the-Indies.pdf) |
| **Sugiyama and colleagues, 2021, “Humans as Geomorphic Agents.”** *PLOS ONE* 16(9): e0257550. | Quantitative evidence for Teotihuacan’s long-term landscape influence. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0257550) |

### Useful calibration material

| Material | Use and limitation |
| --- | --- |
| **Teotihuacan Mapping Project and the 2021 lidar study** | Compare orientations, compounds and landscape persistence. The lidar coverage is approximately **165 km²**; published supplements are available, but the raw lidar has access restrictions associated with archaeological protection. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0257550) |
| **Boerefijn’s Appendix C** | Extract plot modules with dates, local units and evidence type. Keep prescribed and reconstructed dimensions in separate fields. [University of Virginia](https://pure.uva.nl/ws/files/994580/75691_19_appendix_C_lots_def_vt_Y_.pdf) |
| **DC GIS L’Enfant Plan street-centerline dataset** | Historical-plan geometry for network comparison. Use original plan documentation—not ambiguous GIS buffer metadata—to establish street widths. [Data.gov](https://catalog.data.gov/dataset/lenfant-plan-street-centerlines) |
| **NYPL Commissioners’ Plan collection** | Original plan, report and related material for studying reservation, implementation and amendment. [NYPL Digital Collections](https://digitalcollections.nypl.org/collections/commissioners-plan-of-manhattan-island-and-report-with-related-materials?filters%5Bdivision%5D=MSS&keywords=&utm_source=chatgpt.com) |
| **Cornell’s John Reps Bastides Collection** | Photographic evidence of surviving squares, frontages and street relationships; not a random statistical sample of medieval settlements. [Cornell University Library Exhibits](https://exhibits.library.cornell.edu/bastides-collection) |

### Contested claims and thin evidence

**Geometry does not uniquely identify government.** Strong regularity supports coordinated action, but does not by itself distinguish a king, council, priesthood or coalition. Conversely, non-orthogonal layouts can embody deliberate control of access and assembly. [UW Faculty](https://faculty.washington.edu/plape/citiesaut11/readings/Journal%20of%20Planning%20History-2007-Smith-3-47.pdf)

**Ideal plans are not observations.** A textual dimension is evidence that someone prescribed it; an archaeological reconstruction is an interpretation of surviving evidence. Their uncertainty should travel with the parameter.

**The surviving sample is selective.** Monumental centers, unusually well-preserved foundations and excavated abandoned sites are easier to study than altered ordinary neighborhoods. Teotihuacan’s residential estimates, for example, rest on a much smaller excavated sample than the thousands of compounds inferred to have existed. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/teotihuacan)

**Universal planning-success or erosion rates are not established.** Calibrate those mechanisms against multiple cases rather than inventing one annual decay constant.

The resulting design principle for TCE is straightforward: **institutions propose a spatial order, acquire enough authority and resources to implement part of it, and then negotiate its survival with the people who live there.** That mechanism can generate persistent grids, altered wards, incomplete capitals and irregular extensions without scripting any particular historical outcome.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928ed-e618-83ea-94f4-79938b1227a4)
