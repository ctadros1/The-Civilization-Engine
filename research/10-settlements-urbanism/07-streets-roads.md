# Streets and roads: design and evolution

## A simulation-ready report for The Civilization Engine

**For TCE, represent a road as a maintained physical corridor, a transport service, and a bundle of rights—not as a technology level.** Width, surface, foundation, drainage, permitted users, and maintenance can improve independently. A narrow paved market street, a broad unpaved ceremonial avenue, and an engineered mountain footpath are different solutions, not successive stages.

Two historical cases illustrate the problem with a universal upgrade ladder. Pompeii never became uniformly stone-paved: several surface technologies coexisted. Conversely, the early farming communities that built the Sweet Track around **3807/3806 BCE** constructed an approximately **2 km timber walkway** across wetlands without first developing a paved-road system. [Gwern](https://gwern.net/doc/history/2018-poehlher.pdf)

The recommendations below distinguish **observed or reconstructed historical values**, **engineering prescriptions**, and **proposed TCE defaults**. These should not be treated as interchangeable evidence.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Repeated movement creates paths—but can also destroy their usefulness

Helbing, Keltsch, and Molnár’s active-walker model demonstrates how travelers modifying the ground, and subsequently preferring modified ground, can produce shared trail systems. Its evidence concerns contemporary pedestrian paths; it is a useful mechanism, not a calibrated model of prehistoric societies. [arXiv](https://arxiv.org/pdf/cond-mat/9805158)

**TCE rule:** Maintain separate variables for *trail visibility/vegetation clearance* and *surface damage*.

Initially, repeated passage removes vegetation and makes following the route easier. Continued traffic on susceptible wet ground creates depressions, rutting, and mud. Travelers then either remain on the established route, walk alongside it, or establish a detour.

A simple proposed visibility model is:

\[
\frac{dT}{dt}=\alpha U(1-T)-\beta T
\]

Here, \(T\) is trail definition, \(U\) is passage intensity, \(\alpha\) is trampling effectiveness, and \(\beta\) is vegetation recovery. Soil damage requires a separate update; otherwise, unlimited traffic unrealistically improves a dirt road forever.

Allow abandoned parallel tracks to recover. This produces braided paths, seasonal alternatives, and surviving traces of obsolete alignments without authoring them individually.

### 1.2 Route geometry should respond to transport mode

Chacoan roads include stairways and alignments continuing directly across difficult terrain. Their builders had neither wheeled vehicles nor pack animals. In the Andes, engineered routes supported pedestrians and llama caravans, with construction adapted to local environments. These are direct counterexamples to treating every important road as a potential cart road. [National Park Service](https://www.nps.gov/chcu/learn/historyculture/chacoan-roads.htm)

**TCE rule:** Evaluate accessibility separately for pedestrians, carried loads, pack animals, sledges, carts, and later vehicles.

A pedestrian can use steps; a cart cannot. A pack animal may negotiate a narrower bend than a long wagon-and-team combination. A route suitable for an empty cart may be unsuitable for the same cart carrying stone.

For slow wheeled movement, a useful first-order physical approximation is:

\[
F\_{\text{required}}\approx Mg(C\_{rr}+s)
\]

where \(M\) is vehicle-plus-load mass, \(g\) gravitational acceleration, \(C\_{rr}\) rolling resistance, and \(s\) uphill grade as rise/run. This excludes acceleration and the animal’s own climbing effort.

For illustration—not an empirical calibration—assuming \(C\_{rr}=0.02\), a 5% ascent raises the vehicle’s required tractive force to approximately **3.5 times** its level-ground value. Descending requires a separate braking/control constraint.

Consequently, agents should sometimes choose a longer contouring route, reduce loads, add animals, or transfer goods between modes instead of requesting a steeper road’s “speed upgrade.”

### 1.3 Drainage is infrastructure, not a surface modifier

FHWA guidance separates two requirements: removing water from the running surface and removing it from beside the road. A crowned surface can still fail when roadside water enters and weakens the road body. [Federal Highway Administration](https://www.fhwa.dot.gov/clas/ctip/unpaved_roads_dust/ch_6.aspx)

**TCE rule:** Give drainage its own connected network:

* Surface runoff reaches a gutter, ditch, or adjacent ground.
* Ditches reach an outlet, cross-drain, stream, or storage area.
* Blockage or an undersized downstream component can back water up.

A ditch with no outlet should not count as functioning drainage. A road embankment crossing a watercourse without adequate openings should obstruct flow.

Track moisture-sensitive bearing strength, erosion, sediment accumulation, and local flood depth. Maintenance can clear an outlet or reshape a crown without changing the surface material.

For an initial implementation, use catchment runoff and drainage-capacity approximations rather than full fluid simulation. Update extreme events separately from ordinary daily wear.

### 1.4 Surfacing is a choice among material recipes

ILO rural-road guidance treats construction as a combination of earthworks, drainage, material supply, and workmanship. Modern pavement research likewise distinguishes the wearing surface from the supporting structure; a binder holds aggregate together but does not eliminate the need for support beneath it. [International Labour Organization](https://www.ilo.org/publications/building-rural-roads)

**Recommended TCE abstraction:**

| Surface family | Capabilities and trade-offs to represent |
| --- | --- |
| Cleared natural ground | Low initial material requirement; performance strongly dependent on soil, moisture, and vegetation. |
| Shaped or raised earth | Deliberate alignment, compaction, crown, and drainage; may remain unpaved indefinitely. |
| Gravel or broken stone | Requires suitable aggregate and replenishment; distinguish loose, poorly graded material from well-compacted mixtures. |
| Rounded cobbles, shaped stone blocks, or slabs | Different shaping labor, roughness, joint behavior, and repair requirements; do not give them identical movement bonuses. |
| Fired-clay units or ceramic fragments | Requires an appropriate production tradition; material and laying pattern need not resemble European stone paving. |
| Timber, brushwood, or causeway construction | A local solution for wet or unstable ground, with material-specific renewal requirements. |
| Bound aggregate or concrete | Additional production capabilities and supply chains; retain cracking, support, and drainage failure states. |

These are suggested simulation categories, not a universal historical taxonomy. For example, Yoruba archaeology documents ceramic pavements associated with roads, courtyards, and sacred settings, including edge-laid herringbone arrangements. [Academia](https://www.academia.edu/4255832/Potsherd_Pavements_in_Yorubaland)

**Do not make “cobblestone” intrinsically faster than every alternative.** Calculate movement from roughness, firmness, wetness, traction, and user type. The surface name selects underlying properties.

### 1.5 Hierarchy emerges from function before it becomes a design standard

The Qhapaq Ñan connected political, economic, and sacred places through major routes and subsidiary connections. Chaco demonstrates that unusually wide roads need not be explained solely by freight throughput. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1459/)

**TCE rule:** Maintain separate attributes for:

**Function:** household access, local collection, market connection, intersettlement movement, military administration, pilgrimage, ceremony.

**Physical design:** width, surface, drainage, alignment, carrying capacity.

**Authority:** household, neighboring owners, community, town, ruler, concession holder.

A road becomes functionally important when many journeys depend on it, when it connects important destinations, or when a powerful institution designates it. Its physical standard changes only when someone can organize the work.

This permits important but poor roads, excellent private approaches, and expensive ceremonial routes with little ordinary traffic.

### 1.6 Upgrades require a sponsor, not just traffic

English turnpike trusts could collect tolls, acquire land, and issue bonds. Their institutional powers helped overcome limitations of maintenance organized within individual parishes. Bogart’s research finds that the trusts increased investment rather than merely replacing equivalent parish expenditure. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/transport_revolution_cehmbjuly92013_forweb.pdf)

**TCE rule:** Generate candidate projects from repeated experienced problems:

| Experienced problem | Candidate responses—not a predetermined upgrade |
| --- | --- |
| Wet-season interruptions | Clear outlets, raise the road, add a culvert, harden a short section, or reroute. |
| High hauling effort | Improve surface/support, reduce grade, enlarge loads through better vehicles, or change transport mode. |
| Opposing carts cannot pass | Passing places, time restrictions, one-way operation, widening, or an alternative route. |
| Market obstruction | Move stalls, create loading space, enlarge a square, regulate hours, or build a bypass. |
| Strategic vulnerability | Strengthen a crossing, create a redundant route, station guards, or reserve passage rights. |
| Prestige or ceremonial demand | Improve an approach, avenue, gateway, or plaza even when commercial returns are low. |

Evaluate projects against expected benefits, construction and maintenance burdens, land acquisition, disruption, and alternatives. Then apply political decision-making: the beneficiaries may not control the resources, and those paying may not value the benefits.

For an early agrarian economy, compare labor, food, tools, materials, and foregone production directly. Money need not exist.

**There is no defensible universal “pave at X travelers/day” threshold in the evidence reviewed.** Modern low-volume-road appraisal models are useful frameworks, but their motor-vehicle traffic categories are not ancient paving rules. [SSATP](https://www.ssatp.org/publications?page=17&utm_source=chatgpt.com)

### 1.7 Rights-of-way constrain widening

Hakim’s analysis of traditional building rules emphasizes avoiding harm, respecting earlier use and property, and distinguishing public circulation from locally controlled space. These rules could regulate an apparently irregular street network without requiring a modern planning department. [ResearchGate](https://www.researchgate.net/publication/26502838_Revitalizing_Traditional_Towns_and_Heritage_Districts)

**TCE rule:** Store at least three widths:

\[
W\_{\text{legal corridor}},\quad
W\_{\text{constructed}},\quad
W\_{\text{currently usable}}
\]

A broad reserved corridor can contain a narrow track. A constructed street can lose usable width to stalls, storage, parked vehicles, rubble, vegetation, or drainage channels.

Widening beyond the recognized corridor requires negotiation, purchase, compulsory acquisition, or unlawful encroachment. Include demolition, relocation, compensation, and opposition. Making a road wider should not silently delete adjoining buildings.

Once frontage development becomes expensive to remove, actors should consider managing traffic or building another route.

### 1.8 Maintenance is a recurring collective-action problem

Roman legal evidence assigned frontage-related paving liabilities and allowed official intervention. Applying Rome’s precise arrangements to Pompeii remains an inference, although coordinated paving indicates oversight. In Andean examples, communities served by suspension bridges were required to maintain them as service to the empire. [Gwern](https://gwern.net/doc/history/2018-poehlher.pdf)

**TCE rule:** Separate:

**Routine work:** clearing drains, removing debris, filling small defects, trimming vegetation, restoring local shape.

**Periodic work:** replacing wearing material, resetting pavers, renewing timber components, rebuilding failed drainage.

**Emergency work:** reopening washouts, landslides, bridge failures, and conflict damage.

Assign responsibility explicitly. A public road with no functioning payer or labor obligation should accumulate a maintenance backlog.

Corvée is not free construction: workers still need food, tools, supervision, and time away from farms or workshops. A merchant’s voluntary repair should reflect private benefit; an official’s project may reflect wider benefit, favoritism, or prestige.

Maintenance credit should be awarded only when agents complete the required tasks with the required inputs.

### 1.9 Intersections and plazas require area, access, and competing uses

Modern junction guidance makes turning geometry dependent on the vehicles using it, rather than on road width alone. Archaeology at Pompeii also records localized work at intersections rather than only whole-street replacement. [Federal Highway Administration](https://highways.dot.gov/safety/other/road-diets/road-diet-informational-guide/4-designing-road-diet)

**TCE rule:** An intersection is not a zero-area graph point.

Represent turning envelopes, conflicting movements, waiting space, and obstructions. A long cart may fit along both adjoining streets but fail to turn between them. Gates, bridges, ferries, steep approaches, and loading places should have their own queues.

A plaza should be a polygon with occupiable space and several access points—not an oversized junction granting unlimited throughput. Assign areas to circulation, stalls, wells, monuments, gatherings, and temporary events.

For plaza formation, permit both deliberate reservation and incremental enlargement around an attraction. Give neighboring owners incentives to occupy valuable edges, while market authorities or ritual institutions may defend open space. These are proposed generative rules; the evidence does not support a single cross-cultural plaza-size formula.

---

## 2. Parameters: quantitative anchors and starting ranges

**Confidence notation:** **H** means strong support for the specified measurement or published prescription; **M** means a credible but localized reconstruction or interpretation; **L** means a modeling assumption or weak transfer between contexts. A modern engineering prescription can be H as documentation but L as an estimate of ancient practice.

### 2.1 Widths, drainage, and surface construction

| Parameter or case | Value and units | Interpretation, source, confidence |
| --- | --- | --- |
| Mohenjo-daro main streets | Approximately **9.1 m** | Width reported in Yonekura’s analysis of excavation plans; not every street. **M**. [J-STAGE](https://www.jstage.jst.go.jp/article/chirikagaku/20/0/20_KJ00003718392/_article/-char/en) |
| Mohenjo-daro small lanes | Approximately **1.12–2.13 m** | Reported range of 3 ft 8 in–7 ft. Do not impose it on all Indus settlements. **M**. [J-STAGE](https://www.jstage.jst.go.jp/article/chirikagaku/20/0/20_KJ00003718392/_article/-char/en) |
| Chaco roads inside the canyon | Average approximately **4.6 m** | NPS archaeological account. **M–H**, site-specific. [National Park Service](https://www.nps.gov/chcu/learn/historyculture/chacoan-roads.htm) |
| Chaco roads toward outlying sites | Often approximately **9.1 m** | Approximately twice the internal-road width; functions remain debated. **M**. [National Park Service](https://www.nps.gov/chcu/learn/historyculture/chacoan-roads.htm) |
| Tang Chang’an avenue between Pingkang and Xuanyang wards | **29 m** | Excavated width reported by Linda Rui Feng. A particular monumental avenue, not a typical Chinese street. **M**. [Academia](https://www.academia.edu/6260026/City_of_Marvel_and_Transformation_Chang_an_and_Narratives_of_Experience_in_Tang_Dynasty_China) |
| Public through-street minimum in rules discussed by Hakim | **7 cubits** | A normative requirement associated with passing loaded camels, not an observed universal minimum. Convert using the relevant local cubit. **M**. [ResearchGate](https://www.researchgate.net/publication/26502838_Revitalizing_Traditional_Towns_and_Heritage_Districts) |
| Clear space for two pedestrians passing or walking together | Approximately **1.5 m** | Modern FHWA human-space guidance; useful geometric reference, not a historical legal standard. **H** for guidance. [Federal Highway Administration](https://www.fhwa.dot.gov/publications/research/safety/pedbike/05085/chapt8.cfm?utm_source=chatgpt.com) |
| Gravel-road crown/crossfall | **4–6%** | FHWA handbook target; adjustments required for particular geometry and conditions. **H** as prescription. [Federal Highway Administration](https://www.fhwa.dot.gov/clas/ctip/unpaved_roads_dust/ch_6.aspx) |
| Gravel wearing course | At least **100 mm**, preferably **150 mm** in the cited guidance | Not total foundation thickness. **H** as prescription; low confidence as a historical default. [Federal Highway Administration](https://www.fhwa.dot.gov/clas/ctip/unpaved_roads_dust/ch_6.aspx?utm_source=chatgpt.com) |
| Paved-road normal crossfall | Typically **1.5–2%**; up to **2.5%** in intense rainfall | FHWA guidance for the cited modern-road context. **H** as prescription. [Federal Highway Administration](https://highways.dot.gov/safety/other/road-diets/road-diet-informational-guide/4-designing-road-diet?utm_source=chatgpt.com) |
| Modern motor-vehicle lane width | Commonly **3.05–3.66 m** | Converted from 10–12 ft in FHWA guidance; not total street width. **H** as documented practice. [Federal Highway Administration](https://highways.dot.gov/safety/other/road-diets/road-diet-informational-guide/4-designing-road-diet) |
| Maximum spacing of offshoot drainage outlets in an ILO example | At road grades **4%, 8%, 12%**: **200, 120, 40 m**, respectively | Context-specific guidance; runoff, erosion, soil, and receiving land also matter. Not a universal spacing law. **H** as prescription. [International Labour Organization](https://www.ilo.org/sites/default/files/wcmsp5/groups/public/%40asia/%40ro-bangkok/documents/genericdocument/wcms_101011.pdf) |

**Grade and crossfall are different.** A 5% longitudinal grade climbs 5 m per 100 m horizontally. A 5% crossfall drops 5 cm per metre sideways.

The large differences in this table are useful: width is influenced by purpose, institutions, and available space—not merely transport technology.

### 2.2 Labor, maintenance organization, and historical costs

| Parameter | Quantitative anchor | Confidence and transfer limits |
| --- | --- | --- |
| Manual excavation: soft / medium / hard / very hard soil | **5 / 3.5 / 3 / 2 m³ per worker-day** | ILO recommended norms informed by country data and trials. Modern hand tools and organization assumed. **H** as norms; **L–M** for historical transfer. [International Labour Organization](https://wwwex.ilo.org/dyn/asist/asistdocs.downloadfile?p_filename=F170469982%2FTechinical+Brief+No.2+-+Productivity+Norms+for+la.pdf) |
| Manual rock excavation | Approximately **0.8 m³ per worker-day** | Same ILO reference; highly sensitive to rock and tools. **M** for transfer even within manual construction. [International Labour Organization](https://wwwex.ilo.org/dyn/asist/asistdocs.downloadfile?p_filename=F170469982%2FTechinical+Brief+No.2+-+Productivity+Norms+for+la.pdf) |
| Wheelbarrow haulage | **8.5 m³/worker-day** at 0–20 m; **4.5 m³/worker-day** at 100–150 m | Recommended rates assume a well-organized route and suitable equipment. Reported trial rates were lower, especially at short distances. **H** as norms, not guaranteed output. [International Labour Organization](https://wwwex.ilo.org/dyn/asist/asistdocs.downloadfile?p_filename=F170469982%2FTechinical+Brief+No.2+-+Productivity+Norms+for+la.pdf) |
| Lengthperson maintenance assignment | Approximately **1–1.5 km per worker** | Zambia handbook organizational example. **Not** evidence that each kilometre requires one full-time worker year-round. **H** as prescription. [gTKP](https://www.gtkp.com/document/contractors-handbook-labour-based-road-works-zambia/) |
| Mobile maintenance gang | Up to **20 workers**, plus supervision | Alternative organization in the same handbook. **H** as prescription. [gTKP](https://www.gtkp.com/document/contractors-handbook-labour-based-road-works-zambia/) |
| English statutory road labor | **6 days per year** in the regime discussed by Bogart | An assessed obligation, not evidence of complete compliance or effective work. **M** for realized labor supply. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/transport_revolution_cehmbjuly92013_forweb.pdf) |
| Pompeii stone-paving work-rate reconstruction | **1.475 linear m/team-day**, at **3.5 m** width: approximately **5.2 m²/team-day** | Authors’ assumed rate, considered nearer a maximum than a minimum. Crew size is unspecified; do not convert directly to worker-days. **L**. [Gwern](https://gwern.net/doc/history/2018-poehlher.pdf) |
| Roman stone-paving monetary benchmark used in the Pompeii study | **22.5 sesterces per Roman linear foot**, approximately **76.3 sesterces/m** | Borrowed from inscriptions elsewhere; width assumptions are needed for area costs. Not a directly observed Pompeii price. **L** for transfer. [Gwern](https://gwern.net/doc/history/2018-poehlher.pdf) |
| Renewal of Andean suspension bridges | Often every **1–2 years**; annual rebuilding at Q’eswachaka | Applies to replaceable bridge components and community practice, not stone roads generally. **M–H**. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/question/how-did-inka-keep-their-suspension-bridges-safe.html) |

### 2.3 Calculate costs from quantities

For TCE, use a bill of quantities:

\[
C =
C\_{\text{labor}}+
C\_{\text{materials}}+
C\_{\text{haulage}}+
C\_{\text{tools}}+
C\_{\text{land}}+
C\_{\text{disruption}}
\]

For a layer of compacted material:

\[
V=LWt
\]

**Illustrative calculation:** A 1 km road with a 4 m-wide, 0.15 m-thick wearing course needs **600 m³ of compacted material**. Loose extraction and transported volumes will differ; foundation work, drainage, and losses are additional.

Widening an otherwise identical surface from 3 m to 5 m raises its material volume by **67%**, before land acquisition or changed drainage. This is a geometric consequence, not a historical cost estimate.

Estimate maintenance through tasks and deterioration: expected ditch-clearing volume, replacement gravel, damaged pavers, bridge renewal, and inspection effort. **Do not use one universal annual maintenance percentage of construction cost.** The reviewed evidence is too heterogeneous to justify one.

### 2.4 Explicitly proposed TCE starting priors

These are **uncalibrated design defaults**, not historical measurements.

| Proposed parameter | Initial exploration range | Purpose and confidence |
| --- | --- | --- |
| Single-file footpath tread | **0.6–1.2 m** | Starting geometry; vegetation margins remain passable where appropriate. **L**. |
| Pack-animal/shared path | **1.5–2.5 m** | Recompute clearance from animal and load envelopes. **L**. |
| Single cart running corridor | **2.5–3.5 m** | Depends on authored cart/team geometry; use passing places where necessary. **L**. |
| Comfortable two-cart passing corridor | **4.5–6 m** | Test against actual vehicle widths, loads, and margins. **L**. |
| Preferred sustained grade for loaded cart routes | **3–5%** | Routing preference, not a hard physical maximum. Derive severe penalties from traction above it. **L**. |
| Chronic-problem observation window | **30–90 simulation days** | Prevent institutions oscillating between projects after small fluctuations; bypass for emergencies. **L**. |

---

## 3. Variation across eras and world regions

Treat these as combinations of capabilities and institutions that can coexist, not calendar-locked stages.

| Context | Evidence and characteristic variation | Implication for TCE |
| --- | --- | --- |
| **Foragers and mobile communities** | Active-walker models offer a plausible physical feedback for recurring foot travel, but they do not establish prehistoric widths, formation times, or maintenance rates. [arXiv](https://arxiv.org/pdf/cond-mat/9805158) | Begin with destinations, terrain knowledge, clearance, and seasonal movement. Mark numerical calibration as weak rather than inventing a universal “forager road.” |
| **Early farming communities** | The Sweet Track demonstrates deliberate timber engineering across wetlands in the fourth millennium BCE. [Historic England](https://historicengland.org.uk/listing/the-list/list-entry/1014831) | Valuable access can justify substantial work before cities or centralized states. Early road spending may concentrate on a short obstacle rather than an entire corridor. |
| **South Asian urbanism: Indus examples** | Mohenjo-daro combined broad principal streets with much narrower lanes; proposed reconstructions of an underlying perfect planning module are more interpretive than the individual measurements. [J-STAGE](https://www.jstage.jst.go.jp/article/chirikagaku/20/0/20_KJ00003718392/_article/-char/en) | Permit planned main corridors and finer-grained local access within the same settlement. Do not use a single width or assume all roads were paved. |
| **East Asia: Tang Chang’an** | Monumental avenues coexisted with walled residential wards and timed gates. Feng contrasts this arrangement with the more dispersed commercial streets of later Kaifeng. [Academia](https://www.academia.edu/6260026/City_of_Marvel_and_Transformation_Chang_an_and_Narratives_of_Experience_in_Tang_Dynasty_China) | Model access control and time-dependent connectivity independently of physical width. Changes in law can transform circulation without rebuilding every street. |
| **Mediterranean and European preindustrial towns** | Pompeii provides evidence for heterogeneous surfaces and repairs, rather than a uniformly finished network. [Gwern](https://gwern.net/doc/history/2018-poehlher.pdf) | Streets should retain construction histories and local patches. “Roman-style” should not mean identical foundations, paving, and maintenance everywhere. |
| **North Africa, the Middle East, and other Islamic legal settings** | Hakim documents interaction between legal principles and local custom, including earlier-use rights and minimum through-street provisions. [ResearchGate](https://www.researchgate.net/publication/26502838_Revitalizing_Traditional_Towns_and_Heritage_Districts) | Irregular form can be regulated. Distinguish public through routes from neighborhood-controlled access; avoid a single universal “Islamic city” template. |
| **West Africa: Yoruba settlements** | Ceramic pavements occur in residential, road, and sacred contexts; regional archaeological dates cluster particularly within the twelfth–fifteenth centuries. [Academia](https://www.academia.edu/4255832/Potsherd_Pavements_in_Yorubaland) | Local ceramic traditions can supply paving. Ritual and political value can influence which spaces receive it, alongside practical benefits. |
| **Andean states** | UNESCO describes a Qhapaq Ñan network of approximately **30,000 km**. Smithsonian material emphasizes local adaptation and community obligations for bridge maintenance. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1459/) | Large integrated networks need not depend on wheeled transport. Distinguish state coordination from the households supplying labor. |
| **North American Southwest: Chaco** | Broad, straight routes, stairs, and sometimes discontinuous alignments support competing practical and ceremonial interpretations. [National Park Service](https://www.nps.gov/chcu/learn/historyculture/chacoan-roads.htm) | Road expenditure should not always maximize freight efficiency. Preserve uncertainty about the objective rather than treating all unusual geometry as irrational. |
| **Industrializing economies** | British road investment combined new financing arrangements with engineering changes associated with McAdam and Telford. Mechanical compaction subsequently increased construction speed. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/transport_revolution_cehmbjuly92013_forweb.pdf) | Unlock material processing, compaction equipment, survey skills, contracting, and credit separately. Industrial capabilities accelerate some tasks without eliminating land or maintenance constraints. |
| **Modern systems** | Lane allocation, pedestrian provision, junction design, and drainage are explicit engineering concerns; road-diet guidance shows that improvement can reallocate existing width rather than widen the corridor. [Federal Highway Administration](https://highways.dot.gov/safety/other/road-diets/road-diet-informational-guide/4-designing-road-diet) | “Upgrade” can mean better reliability, safer crossings, or more useful space—not necessarily higher speed or additional vehicle lanes. |

---

## 4. Stylized facts and validation targets

A correct simulation should reproduce conditional patterns, not one ideal street map.

### A. Importance, width, and paving are imperfectly correlated

The numerical anchors above span narrow Indus lanes, broad Chang’an avenues, and unusually wide Chaco roads without wheeled vehicles. Validate the **joint distribution** of function, width, surface, and transport mode—not a rigid hierarchy assigning one standard to each rank. [J-STAGE](https://www.jstage.jst.go.jp/article/chirikagaku/20/0/20_KJ00003718392/_article/-char/en)

### B. Surface histories are patchworks

At Pompeii, early stone-paving phases survived at least **60 years**, with some substantially repaired examples reaching **179 years**. These are reconstructed histories, not maintenance-free design lives. [Gwern](https://gwern.net/doc/history/2018-poehlher.pdf)

**Test:** Older towns should contain segments of different ages, repaired patches, and surviving alignments—not synchronized citywide deterioration or replacement.

### C. Drainage and season change the ranking of routes

Engineering guidance links inadequate drainage to softening, potholes, erosion, and loss of passability. [Federal Highway Administration](https://www.fhwa.dot.gov/clas/ctip/unpaved_roads_dust/ch_6.aspx)

**Test:** A short route through susceptible ground may win in dry weather and lose during rains. Repairing its surface without correcting drainage should provide only limited or temporary benefit.

### D. Short critical structures can carry disproportionate maintenance burdens

Andean bridge renewal could be annual or biennial even when the route itself persisted. [National Museum of the American Indian](https://americanindian.si.edu/inkaroad/engineering/question/how-did-inka-keep-their-suspension-bridges-safe.html)

**Test:** The closure of a small bridge, ford, or gate should sometimes matter more than deterioration along kilometres of ordinary road. Institutions should occasionally prioritize these network bottlenecks.

### E. Institutional change can improve roads without a new surface invention

By the early nineteenth century, British turnpike trusts controlled approximately **20,000 miles**, about **20% of highways for wheeled carriages** in Bogart’s account. The relevant change included financing and coordination powers. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/transport_revolution_cehmbjuly92013_forweb.pdf)

**Test:** Holding tools and materials constant, different responsibility and funding arrangements should produce different maintenance outcomes.

### F. Hauling materials constrains what gets built

The ILO’s recommended wheelbarrow rates fall from **8.5 to 4.5 m³ per worker-day** as haul distance increases from 0–20 m to 100–150 m. [International Labour Organization](https://wwwex.ilo.org/dyn/asist/asistdocs.downloadfile?p_filename=F170469982%2FTechinical+Brief+No.2+-+Productivity+Norms+for+la.pdf)

**Test:** Quarry and gravel-pit location should affect road recipes and project feasibility. Improving access to a material source should sometimes reduce the cost of subsequent construction.

### G. More infrastructure creates more recurring obligations

This is a **model prediction from the accounting structure**, not a universal fitted historical ratio.

**Test:** A settlement that expands roads while holding its maintenance labor and material supply fixed should accumulate backlog. Resource shocks should produce postponed work, selective abandonment, and concentrated protection of essential routes—not automatic maintenance of everything already built.

---

## 5. Modeling recommendation for TCE

### 5.1 Use a hybrid spatial representation

Keep four linked representations:

| Layer | What it stores |
| --- | --- |
| **Terrain/environment raster** | Soil, moisture, vegetation, slope, erosion susceptibility, and runoff contribution. |
| **Movement graph with geometric corridors** | Routes, polylines, width profiles, mode permissions, junctions, crossings, and travel costs. |
| **Parcel and rights geometry** | Claimed land, recognized corridors, frontage responsibilities, encroachments, and acquisition requirements. |
| **Infrastructure assets and work orders** | Surface layers, drains, bridges, condition, responsible institutions, materials, workers, and unfinished projects. |

Do not constrain a narrow path’s width to terrain-cell size. Render and navigate it as finer corridor geometry over the terrain.

Split segments at intersections and meaningful changes in slope, surface, drainage, responsibility, or width. Avoid creating one simulation object per paving stone.

### 5.2 Keep enough state to distinguish failure mechanisms

A minimal segment record should include:

| State group | Minimum useful fields |
| --- | --- |
| Geometry | Alignment, length, grade profile, legal width, constructed width, usable width. |
| Construction | Surface recipe, wearing-layer quantity, support quality, compaction/workmanship. |
| Condition | Roughness, rutting/deformation, material deficit, obstruction, moisture/flood state. |
| Drainage | Crown or crossfall, connected drains/outlets, blockage and capacity. |
| Use | Trips by mode, load distribution, waiting time, seasonal interruptions, route alternatives. |
| Governance | Owner, maintenance responsibility, access rights, tolls, assessed obligations, current work orders. |

A single “road health” value can summarize this for the UI, but should not be the underlying simulation.

### 5.3 Preserve individual decisions while aggregating physical wear

Every person can retain a destination, mode, load, familiarity, and journey outcome. Accumulate passage and loading statistics onto segments rather than modifying terrain independently for every footstep.

A proposed deterioration update is:

\[
\Delta D =
D\_{\text{weather}}
+
D\_{\text{traffic}}(\text{mode, load, wetness, concentration})
-
D\_{\text{completed repairs}}
\]

The functional forms need calibration. In particular, do not transplant a modern heavy-axle pavement-damage exponent onto barefoot pedestrians, hooves, or ancient carts.

For route choice, use perceived journey cost:

\[
C\_{\text{route}}=
C\_{\text{time}}+
C\_{\text{effort}}+
C\_{\text{fees}}+
C\_{\text{risk}}+
C\_{\text{unreliability}}
\]

Weights depend on agent circumstances. A trader protecting perishable goods should value reliability differently from a person making an optional social visit.

Agents need not know every newly improved road instantly. Use familiar routes plus exploration and communicated information.

### 5.4 Make construction actual economic activity

An approved road project should create a task chain:

**survey and negotiate → clear and excavate → obtain materials → haul → drain and prepare → place and compact → inspect/open.**

Some stages can overlap; others cannot. A paved surface should not appear when workers have completed only excavation.

Projects should consume storage space and obstruct movement. Materials can be redirected, workers reassigned, and funding exhausted. Partial completion should leave a physically meaningful state.

A useful autonomous sequence is: repeated cart delays generate complaints; a sponsor commissions inspection; workers identify one wet low point; the institution compares draining that section with paving the whole street; it chooses a project it can resource. Nothing requires a scripted milestone such as “population reaches 500, therefore pave.”

### 5.5 Use multiple time scales

For the stated 10k–50k population target, a reasonable starting architecture is:

| Process | Suggested scheduling |
| --- | --- |
| Individual travel and queues | Event-driven; detailed local stepping only where interactions require it. |
| Traffic/load accumulation | During trips, consolidated into daily segment statistics. |
| Ordinary deterioration and vegetation | Daily or seasonally adjusted updates. |
| Floods, washouts, major obstruction | Immediate event updates. |
| Routine maintenance dispatch | Daily or weekly institutional decisions. |
| Capital-project review | Monthly/seasonal review, plus emergency exceptions. |
| Network routing caches | Invalidate on meaningful cost or connectivity changes. |

These are implementation recommendations, not benchmarked performance claims.

Near-camera and distant simulation must conserve people, goods, travel time, and bottleneck capacity. Visual detail should not make the same bridge carry more traffic simply because the camera moved away.

### 5.6 Existing models and games worth borrowing from

| Model or game | Useful component | Limitation for TCE |
| --- | --- | --- |
| **Helbing–Keltsch–Molnár active-walker model** | Reinforcement, regeneration, and emergence of shared paths. [arXiv](https://arxiv.org/pdf/cond-mat/9805158) | Does not supply historical institutions, pavement construction, or long-run maintenance. |
| **SUMO pedestrian and junction models** | Explicit walking areas, crossings, local interaction, and bottlenecks. [Eclipse SUMO](https://sumo.dlr.de/docs/Simulation/Pedestrians.html) | Its modern network conventions need adaptation for shared streets, carts, animals, and informal crossing. |
| **World Bank/SSATP Roads Economic Decision model—RED** | Comparing low-volume-road investments through user benefits and costs. [SSATP](https://www.ssatp.org/publications?page=17&utm_source=chatgpt.com) | Borrow the appraisal structure, not motor-vehicle coefficients or modern monetary valuations. |
| **Foundation** | Gridless organic settlement presentation, agent activity, and selective road paving as a visual/gameplay reference. [Steam Store](https://store.steampowered.com/app/690830/Foundation/) | A reference for presentation and player interaction, not evidence of historically calibrated autonomous infrastructure institutions. |

**Recommended first implementation:** mode-specific passability; traffic-created paths; moisture and drainage; material-and-labor work orders; maintenance responsibility; parcel-constrained widening. Add sophisticated junction microsimulation after those systems interact correctly.

---

## 6. Sources, datasets, and evidence limits

### Highest-value research and technical references

| Source | Best use |
| --- | --- |
| **Poehler & Crowther, 2018, “Paving Pompeii: The Archaeology of Stone-Paved Streets,” *American Journal of Archaeology* 122(4):579–609. DOI: 10.3764/aja.122.4.0579.** | Repair histories, construction events, wear, and explicitly qualified cost reconstructions. [Gwern](https://gwern.net/doc/history/2018-poehlher.pdf) |
| **Helbing, Keltsch & Molnár, 1997, “Modelling the evolution of human trail systems,” *Nature* 388:47–50. DOI: 10.1038/40353.** | A generative mechanism for trails rather than decorative path placement. [Nature](https://www.nature.com/articles/40353) |
| **ILO, *Building Rural Roads*; ASIST Technical Brief No. 2, *Productivity Norms*.** | Drainage, labor-based construction, task quantities, and productivity uncertainty. [International Labour Organization](https://www.ilo.org/publications/building-rural-roads) |
| **FHWA, *Unpaved Road Dust Management: A Successful Practitioner’s Handbook*, chapter 6.** | Relationships among shape, wearing material, preparation, and drainage. [Federal Highway Administration](https://www.fhwa.dot.gov/clas/ctip/unpaved_roads_dust/ch_6.aspx) |
| **Hakim, 2007, “Revitalizing Traditional Towns and Heritage Districts,” *Archnet-IJAR* 1(3):153–166. DOI: 10.26687/archnet-ijar.v1i3.26.** | Street-related rights, local custom, and rule-governed incremental development. [ResearchGate](https://www.researchgate.net/publication/26502838_Revitalizing_Traditional_Towns_and_Heritage_Districts) |
| **Ogundiran, 2000, “Potsherd Pavements in Ilare-Ijesa, Yorubaland: A Regional Perspective,” *Nyame Akuma* 53.** | Non-European material traditions and distinctions between practical and sacred paving contexts. [Academia](https://www.academia.edu/4255832/Potsherd_Pavements_in_Yorubaland) |
| **Bogart, *The Transport Revolution in Industrializing Britain: A Survey*, and associated research.** | Financing, maintenance institutions, transport services, and network investment. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/transport_revolution_cehmbjuly92013_forweb.pdf) |

### Datasets and spatial evidence

**Historical British roads:** Bogart and collaborators provide research-linked datasets for turnpike roads, main roads, and coaching networks, including a turnpike-road map covering **1700–1838**. These are useful for testing jurisdiction, network hierarchy, and investment diffusion—not for calibrating every preindustrial society. [Social Sciences UCI](https://www.socsci.uci.edu/~dbogart/test_researchpage/)

**GRIP4:** Meijer and colleagues’ Global Roads Inventory Project provides vector roads and five-category density products. Its raster density is **5 arcminutes**, approximately 8 km near the equator, and the provider explicitly says it is **not suitable for navigation**. Use it for broad modern network comparisons, not alley geometry or ancient-route reconstruction. [Globio](https://www.globio.info/download-grip-dataset)

**Archaeological site evidence:** Excavation plans and phased pavement records are more appropriate than modern road inventories for historical width and repair validation. However, published sites are unevenly distributed, and the surviving material is not a random sample of all former routes. The Sweet Track record, for example, explicitly notes discovery and preservation biases affecting prehistoric timber trackways. [Historic England](https://historicengland.org.uk/listing/the-list/list-entry/1014831)

### What remains uncertain

**Norms are not outcomes.** A prescribed seven-cubit street or six-day labor obligation does not establish actual width or effective work.

**Archaeological lifespan is not engineering lifespan.** A surface can remain identifiable while receiving repairs, carrying reduced traffic, or operating poorly.

**Technology labels conceal variation.** “Dirt,” “gravel,” and “stone paving” cover different soils, aggregate mixtures, support structures, and workmanship.

**Global numerical calibration is thin.** The reviewed sources do not justify universal ancient traffic capacities, paving thresholds, plaza areas per inhabitant, or annual road-maintenance cost ratios. Those should remain exposed calibration parameters.

**Function can be contested.** Chaco’s road system is a particularly important warning against explaining all expensive infrastructure through ordinary transport demand alone. [National Park Service](https://www.nps.gov/chcu/learn/historyculture/chacoan-roads.htm)

The strongest design for TCE is therefore not a catalogue of era-specific road types. It is a system in which **movement changes ground; ground and weather change journey costs; people propose competing remedies; institutions mobilize resources; and every improvement creates an asset that someone must continue to maintain.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928f6-ead8-83ea-89d3-fbb17a6ac395)
