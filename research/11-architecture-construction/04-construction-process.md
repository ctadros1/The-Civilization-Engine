# How buildings were built: labor, time, and organization

## Executive conclusion

**For TCE, construction should be a network of quantity-based tasks—not a fixed duration attached to a building type or historical era.** The important distinctions are between obtaining materials, processing them, transporting them, assembling the structure, and waiting for materials or unfinished work to become usable.

Those distinctions can reverse the apparent economics of construction. In Elliot Abrams’s reconstruction of a substantial Maya residence at Copán, transport accounted for about **49%** of estimated labor, manufacturing for **39%**, and final assembly for only **3.5%**. These are estimates for one building, not universal proportions, but they demonstrate why “workers applying labor to delivered stone” captures only part of the process. [Scribd](https://www.scribd.com/document/37073996/0884022544)

The recommended foundation is therefore:

**Geometry → material quantities → production and delivery tasks → skill-constrained assembly → usable stages → continuing maintenance.**

The quantitative evidence below distinguishes recorded observations, estimating norms, archaeological reconstructions, and proposed TCE defaults.

---

## 1. Mechanisms: rules a simulation can implement

### 1.1 Keep labor, elapsed time, and financial cost separate

Use three independent accounting systems:

| Quantity | Meaning | Recommended TCE representation |
| --- | --- | --- |
| **Labor expenditure** | Time contributed by all participating people | Person-hours, separated by occupation and task |
| **Elapsed construction time** | Time between milestones, including interruptions | Start, weather-tight, usable, finished, abandoned |
| **Financial expenditure** | What the commissioning household or institution pays | Material purchases, wages, transport, equipment, administration |

An owner-built house may involve little cash but substantial household labor. Conversely, a purchaser can acquire an expensive prefabricated component whose production labor occurred elsewhere. FAO’s construction-costing guidance explicitly treats family labor as an opportunity cost rather than a free resource. [FAOHome](https://www.fao.org/4/s1250e/S1250E0o.htm)

For comparisons, define **one normalized person-day, `pd8`, as eight person-hours**. This is a bookkeeping convention, not a claim about historical working days. Preserve original source units whenever day length is unknown.

Also distinguish:

* **Masonry volume:** finished wall volume, including joints—not solid stone volume or a loose pile.
* **Earth volume:** undisturbed excavation, loose transported soil, and compacted fill.
* **Framing area:** the floor area served by the frame—not wall elevation.
* **Roofing area:** actual sloping surface, including whatever eaves the design specifies.

A crew-day is not a person-day: six people working one day contribute six person-days.

### 1.2 Derive construction tasks from the building grammar

The following is a recommended construction template for TCE. It should branch by structural system rather than force every building through an identical sequence.

| Stage | Inputs and activities | Conditions for progress | Visible result |
| --- | --- | --- | --- |
| **Commission and preparation** | Claim or permission, design, estimates, labor commitments, procurement orders | Site access and an organizer | Marked plot, stakes, stored materials |
| **Groundworks** | Clearing, excavation, drainage, grading, platform construction | Suitable ground conditions; disposal or reuse route for spoil | Trenches, piles, raised platform |
| **Foundations and base** | Footings, postholes, plinth, foundation masonry | Excavations ready; required materials supplied | Established footprint |
| **Primary structure** | Posts, beams, load-bearing walls, temporary bracing | Qualified crew; safe lifting arrangements; previous work ready to carry loads | Frame or rising walls |
| **Roof and enclosure** | Rafters, battens, thatch or tiles, infill, doors | Supporting structure complete in the relevant area | Weather-tight rooms |
| **Fit-out and commissioning** | Floors, plaster, hearths, kilns, fixtures, equipment, ornament | Appropriate dryness, ventilation, and structural readiness | Habitable house or operational workshop |
| **Repair and alteration** | Replastering, reroofing, replacement members, extensions | Damage, new demand, available resources | Continuing transformation |

Permit parallel work where physically possible: roof timber can be prepared while foundations are built; bricks can dry while another building is occupied; separate wall sections can advance simultaneously.

Historical building knowledge already distinguished such work packages. The Chinese *Yingzao Fashi* organized construction into separate craft systems and specified materials, labor, and working time for particular operations, rather than treating a palace or monastery as one undifferentiated job. [University of Hawaii at Hilo](https://hilo.hawaii.edu/languages/chinese/documents/languages/chinese/Chinese_Architecture_and_Metaphor_Song_Culture_in_the_Yingzao_Fashi_Building_Manual-introchap4-Reduced.pdf)

### 1.3 Make supply chains part of construction

A masonry recipe should create demand for more than stone:

**Extraction or collection → initial shaping → transport → dressing → mortar preparation → placement → finishing.**

A timber recipe similarly distinguishes felling, moving logs, conversion into members, joint preparation, raising, and covering. Do not charge these activities twice: purchased beams already embody upstream labor, but still require delivery and installation.

Model material condition explicitly. Green timber is not always unusable: FAO describes construction with green wood as well as seasoning, whose duration varies enormously by species, section, and environment—from weeks for favorable thin softwood to a year or longer for difficult hardwood seasoning. A universal “all timber waits one year” rule would be inappropriate. [FAOHome](https://www.fao.org/4/s1250e/S1250E04.htm)

### 1.4 Constrain crews by skills and work space

Do not allow twenty helpers to substitute automatically for a mason, carpenter, or kiln specialist.

For each task, specify a productive crew composition, available working positions, and essential equipment. Additional people can prepare materials, carry loads, or work elsewhere, but cannot indefinitely increase output at one wall face.

The organizer should matter through estimating, sequencing, teaching, detecting mistakes, and arranging supplies—not merely through a generic productivity bonus. Marchand’s apprenticeship-based ethnography of Djenné describes building expertise as embedded in training, professional associations, social networks, and relationships between masters and apprentices. [Indiana University Press](https://iupress.org/9780253220721/the-masons-of-djenne/)

### 1.5 Make seasons emerge from interacting constraints

Use local weather and household obligations rather than a universal “construction season.”

Recommended rules include wet-weather limits on exposed earthwork and roofing; moisture-dependent drying; access deterioration on hauling routes; cold or excessively rapid drying affecting binder work; and competing demand for labor during planting, harvesting, herding, or fishing peaks.

Separate **passive waiting** from work. Adobe may need little attention while drying, but still occupies space and remains vulnerable. FAO recommends at least about a month of slow drying for its adobe method, whereas stabilized earth blocks require a different curing sequence. [FAOHome](https://www.fao.org/4/s1250e/S1250E06.htm)

This produces a useful emergent choice: prepare materials during one period, assemble during another, and occupy the building before every finish is complete.

### 1.6 Institutions change recruitment and coordination—not physics

Implement several organizational arrangements that can coexist within one settlement.

**Owner-building and reciprocal help.** Households contribute labor, recruit relatives or neighbors, and hire specialists selectively. Reciprocal help should create social obligations; it should not create free labor. FAO’s rural construction guidance explicitly includes family labor, communal effort, artisans, and contracting as different arrangements. [FAOHome](https://www.fao.org/4/s1250e/S1250E0n.htm)

**Master builders and craft associations.** A master estimates work, recruits a crew, supervises quality, and trains successors. A guild or analogous association can regulate admission, training, reputation, access to commissions, and working practices. Treat restrictive and cooperative effects as separate institutional parameters rather than assuming every association has the same effects.

**Institutional construction.** A temple, court, municipality, or merchant organization can maintain stores, workshops, accounting staff, and long-term crews. Its main advantage should be continuity and coordination, subject to its actual resources.

**Corvée, wage work, and enslaved labor.** Keep these legally and socially distinct. Labor obligations alter recruitment, refusal, compensation, and household burdens; they do not eliminate food requirements or skilled bottlenecks. Egyptian evidence includes both compulsory labor and privately remunerated builders, including payment in goods. It does not justify treating all ancient construction as one labor regime. [eScholarship](https://escholarship.org/content/qt6fr8p2hb/qt6fr8p2hb.pdf?v=lg)

---

## 2. Quantitative parameters

### 2.1 Evidence anchors

**Confidence:** High means the figure is clearly recorded for its stated case; Medium means useful reconstruction, analogue, or estimating norm. Neither means universally transferable.

| ID | Operation or project | Quantitative anchor | Evidence and confidence |
| --- | --- | --- | --- |
| **A1** | Manual excavation | **0.24–0.68 pd8/m³**, calculated from experiments reporting 2.6 m³ in five hours and 1.1 m³ in six hours | Experiments summarized by Ortmann and Kidder; excavation, not complete earthmoving. **Medium**. [ResearchGate](https://www.researchgate.net/publication/263213175_Building_Mound_A_at_Poverty_Point_Louisiana_Monumental_Public_Architecture_Ritual_Practice_and_Implications_for_Hunter-Gatherer_Complexity) |
| **A2** | Brick masonry, Indian estimating norm | **2.50 worker-days/m³**, including mortar preparation; about **0.72 mason-days** | CPWD 2016 analysis, foundation/plinth work. **High as a norm; medium as a productivity analogue**. [Academia](https://www.academia.edu/37038412/2016_GOVERNMENT_OF_INDIA) |
| **A3** | Random rubble masonry, Indian estimating norm | **3.21 worker-days/m³**, including mortar preparation; **1.07 mason-days** | CPWD 2016. **High as a norm; medium transferability**. [Academia](https://www.academia.edu/37038412/2016_GOVERNMENT_OF_INDIA) |
| **A4** | Clay roof-tile installation | **0.186 worker-days/m²**; supporting timber excluded | CPWD 2016 tile-roof item. **High as a norm; medium transferability**. [Academia](https://www.academia.edu/37038412/2016_GOVERNMENT_OF_INDIA) |
| **A5** | Two prehistoric-house reconstructions, Poland | Pit-house: four people over **16 days**, implying approximately **64 worker-days**. A 20 m² shelter: five people over approximately **30 days**, or **150 worker-days** | Experimental projects; historical day length and expert prehistoric productivity are not established. **Medium for the experiments; low for universal prehistoric rates**. [EXARC](https://exarc.net/sites/default/files/2025-11/Two%20Reconstructions%20of%20Prehistoric%20Houses%20from%20Torun%20%28Poland%29%20_%20The%20EXARC%20Journal.pdf) |
| **A6** | Traditional stone-and-earth house replication, Cusco region, 2023 | Approximately **440–460 person-hours**, or **55–57 pd8**, over about **nine working days** | Published 2025; skilled local builders. Component and narrative totals differ slightly. **Medium**. [Academia](https://www.academia.edu/145108875/How_to_Build_a_Drum_Shaped_House_A_Replicative_Experiment_in_Ancestral_Architecture_Cusco_Peru_) |
| **A7** | Maya residences at Copán | Modeled commoner house: **67 source person-days**. House of the Bacabs: **10,686 source person-days**, assuming the analyzed episode without material reuse | Archaeological energetics, not observed timesheets. **Medium for comparative orders of magnitude; low for precise calendar reconstruction**. [Scribd](https://www.scribd.com/document/37073996/0884022544) |
| **A8** | Salisbury Cathedral construction milestones | Foundation stones **1220**; east end completed **1225**; consecration **1258** | **High for milestones**, not a measure of continuous crew employment. [Salisbury Cathedral](https://www.salisburycathedral.org.uk/discover/history/the-cathedral-that-moved/) |
| **A9** | US single-family houses completed in 2023 | Average **8.6 months from start to completion**, plus **1.5 months from authorization to start** | Census Survey of Construction analysis. **High for this population and period**, not labor content. [Eye On Housing](https://eyeonhousing.org/2024/08/single-family-build-time-continues-to-trend-upward-for-2023/) |

The Indian coefficients retain **source worker-days**; using eight-hour shifts to map them into TCE is a modeling convention. They also use supplied industrial materials and mechanically mixed mortar—not an entirely pre-industrial production chain.

The Andean replication is particularly useful because it employed local expertise, but it was not a complete historical household economy: plastering was omitted, some support work was outside the accounting, and materials came from within roughly 3 km. Do not convert its reported loose material quantities directly into finished-wall productivity. [Academia](https://www.academia.edu/145108875/How_to_Build_a_Drum_Shaped_House_A_Replicative_Experiment_in_Ancestral_Architecture_Cusco_Peru_)

### 2.2 Recommended starting rates for TCE

**These are proposed calibration ranges, not published universal historical rates.** They are deliberately broader than the anchors above. Except where noted, they assume supplied materials, ordinary low-rise work, adequate tools, and a competent crew.

“Low” confidence here means the range is a useful initial engineering assumption that needs calibration—not that every point inside it has been measured.

| Work package | Proposed starting range | Included boundary | Basis / confidence |
| --- | --- | --- | --- |
| Hand excavation, ordinary soil | **0.25–0.8 pd8/m³ bank volume** | Digging; excludes substantial haulage | Rounded envelope around A1; **Medium–low** |
| Spread and lightly tamp fill | **0.15–0.5 pd8/m³ compacted fill** | Placement and light compaction | TCE prior; **Low**; not a structural rammed-earth wall |
| Adobe preparation and molding | **1–3 pd8/m³ of finished blocks** | Mixing and molding from supplied ingredients | TCE prior; **Low**; drying separate |
| Adobe wall laying | **1–3 pd8/m³ of wall** | Laying and ordinary jointing | TCE prior; **Low**; block manufacture separate |
| Cob wall construction | **1.5–4 pd8/m³ of wall** | Mixing and placement | TCE prior; **Low**; lift drying separate |
| Rammed-earth wall | **2–5 pd8/m³ of wall** | Mixing, form handling, ramming | TCE prior; **Low** |
| Brick masonry | **1.5–4 pd8/m³ of wall** | Laying and mortar preparation | TCE prior around A2; **Low–medium** |
| Rubble masonry | **3–6 pd8/m³ of wall** | Ordinary laying and mortar preparation | TCE prior around A3; **Low–medium** |
| Simple pole-and-lashing frame | **0.1–0.5 pd8/m² of one-storey footprint** | Assembly using prepared poles | TCE prior; **Low** |
| Traditional jointed timber frame | **0.5–2 pd8/m² of one-storey footprint** | Joint preparation and raising, including roof-support framing | TCE prior; **Low**; excludes felling, hewing, enclosure, roof covering |
| Thatch installation | **0.2–0.6 pd8/m² of actual roof** | Fixing prepared thatch | TCE prior; **Low**; collection and preparation separate |
| Clay-tile installation | **0.15–0.35 pd8/m² of actual roof** | Laying tiles on prepared support | TCE prior around A4; **Low–medium** |

**Timber framing is the weakest standardized unit in this table.** A square metre does not reveal the number of joints, beam dimensions, span, lifting difficulty, or decorative complexity. Use the area rate for early balancing, but derive mature recipes from member lengths, cross-sections, joint types, and lifts.

Likewise, do not assign a single “stonework” rate to rubble, fine ashlar, sculpture, and large monolithic members. Separate extraction, dressing, surface finish, and erection.

### 2.3 Material quantities and passive delays

| Parameter | Value or relationship | Application and confidence |
| --- | --- | --- |
| One documented brickwork recipe | **494 bricks + 0.25 m³ mortar per m³ of finished masonry** | CPWD size-and-bond-specific recipe, not all brickwork. **High for that recipe**. [Academia](https://www.academia.edu/37038412/2016_GOVERNMENT_OF_INDIA) |
| General purchasing allowance | **5–15%** for waste and breakage | FAO estimating guidance; use component-specific allowances where known. **Medium**. [FAOHome](https://www.fao.org/4/s1250e/S1250E0o.htm) |
| Adobe drying | Turn after approximately **3–4 days**; total slow drying **at least one month** | FAO method; climate-sensitive. **Medium**. [FAOHome](https://www.fao.org/4/s1250e/S1250E06.htm) |
| Stabilized-earth blocks | **7–14 days moist curing**, then approximately **2–3 weeks air drying** | Specific stabilized-earth process, not ordinary adobe. **Medium**. [FAOHome](https://www.fao.org/4/s1250e/S1250E06.htm) |
| Pressed-earth material conversion | **1.4–1.7 m³ loose dry soil per m³ of compacted blocks** | Not a universal earthwork swell factor. **Medium**. [FAOHome](https://www.fao.org/4/s1250e/S1250E06.htm) |
| Sloping roof area | \(A\_{\text{roof}}=A\_{\text{horizontal projection}}/\cos\theta\) for a uniform pitch | Geometric relationship; at 45°, multiplier **1.414** |
| Illustrative roof-covering weights | Corrugated steel **8–12 kg/m²**; clay tiles approximately **65 kg/m²** | FAO examples; product-specific and excluding the full supporting structure. **Medium**. [FAOHome](https://www.fao.org/4/s1250e/S1250E0l.htm) |

For rejected production, use \(Q\_{\text{input}}=Q\_{\text{required}}/(1-r)\). A 10% rejection rate therefore requires 11.1% extra production. Do not confuse that with adding a 10% purchasing allowance, and do not apply both to the same loss.

### 2.4 Transport can outweigh assembly

For a repeated carrying operation:

\[
H\_{\text{haul}}
=
\frac{M}{q}
\left(
\frac{d}{v\_{\text{loaded}}}
+
\frac{d}{v\_{\text{return}}}
+
t\_{\text{handling}}
\right)
\]

Here \(M\) is total mass, \(q\) load per trip, and \(d\) one-way distance.

An archaeological modeling convention uses a **22 kg load**, **3 km/h loaded travel**, and **5 km/h return travel**. These are assumptions for a specified transport model, not universal human capacities. [ResearchGate](https://www.researchgate.net/publication/361405946_Bulbs_and_Biographies_Pine_Nuts_and_Palimpsests_Exploring_Plant_Diversity_and_Earth_Oven_Reuse_at_a_Late_Period_Plateau_Site)

Using those assumptions, moving **one tonne one kilometre** requires approximately **24.2 person-hours, or 3.0 pd8**, before loading, unloading, rest, gradients, and route problems.

TCE should consequently make quarries, waterways, roads, carts, and stockpile placement economically consequential. It should not approximate distance with a minor percentage surcharge on masonry labor.

### 2.5 Financial cost shares

Do not hard-code one historical materials-versus-labor ratio.

For one CPWD brickwork item, unpacking mortar preparation gives approximately **23% identifiable labor, 72% basic purchased materials, and 5% carriage, equipment, and sundries**, before specified overhead/profit additions. This is one modern estimating example, not an ancient or worldwide building-cost distribution. [Academia](https://www.academia.edu/37038412/2016_GOVERNMENT_OF_INDIA)

Calculate TCE expenditure from local conditions:

\[
C =
\sum\_m p\_m^{\text{factory/quarry gate}} Q\_m
+
\sum\_r w\_r D\_r
+
C\_{\text{transport}}
+
C\_{\text{equipment}}
+
C\_{\text{administration}}
\]

When using delivered material prices, do not add delivery again.

Maintain a separate embodied-labor ledger. A high purchased-material share may represent labor performed by other households, workshops, or settlements. Unpaid household or compulsory labor can reduce the commissioner’s cash expenditure without reducing the community’s real resource burden.

### 2.6 Worked TCE examples

These are **calculations using the proposed defaults**, not claims that a particular historical culture built these standardized buildings.

#### Small house: approximately 30 m²

Assume a grammar produces **15 m³ of rubble masonry**, **30 m² of framed footprint**, and **50 m² of thatched roof**, including its pitch and eaves.

| Package | Calculation | Labor |
| --- | --- | --- |
| Walls | 15 m³ × 3–6 pd8/m³ | **45–90 pd8** |
| Prepared-timber frame | 30 m² × 0.5–2 pd8/m² | **15–60 pd8** |
| Thatch installation | 50 m² × 0.2–0.6 pd8/m² | **10–30 pd8** |
| Groundworks, simple floor, miscellaneous completion | Provisional allowance pending detailed quantities | **20–40 pd8** |
| **Total supplied-site work** |  | **90–220 pd8** |

Four continuously supplied workers imply a labor-only lower bound of approximately **23–55 working days**. Actual completion requires the right skills at the right times and may take longer.

This excludes quarrying, timber conversion, thatch collection, substantial transport, and elaborate finishes. Replace the provisional miscellaneous allowance with measured quantities as the grammar matures.

A small woodworking or textile workshop can use the same shell calculation. Its functional completion then depends on benches, looms, storage, or other equipment. A smithy or pottery workshop needs separate hearth, furnace, chimney, kiln, and commissioning recipes—not a universal “workshop multiplier.”

#### A 100 m boundary or defensive wall

Assume length **100 m**, height **2.5 m**, thickness **0.5 m**:

\[
V=100 \times 2.5 \times 0.5=125\text{ m³}
\]

Rubble placement requires approximately **375–750 pd8**. Ten appropriately composed workers imply a labor-only lower bound of **38–75 working days**.

For comparison, an assumed delivery requirement of **250 tonnes** carried **1 km** by the porter model above adds approximately **760 pd8** before handling. The transport system can therefore matter as much as the wall-laying crew.

#### Small temple or communal hall

Assume **200 m³ of earth platform**, **100 m³ of rubble masonry**, **200 m² of timber-framed footprint**, and **300 m² of thatch**.

Excavation and fill placement contribute roughly **80–260 pd8**; masonry **300–600**; framing **100–400**; roof covering **60–180**. Total core work is approximately **540–1,440 pd8**, excluding procurement, transport, elaborate finishes, and equipment.

Twenty workers give a labor-only lower bound of **27–72 working days**, not a guaranteed completion date. A temple with dressed façades, carved columns, complex roofing, or monumental lifting requirements is a different task inventory—not the same building with an arbitrary prestige surcharge.

---

## 3. Variation across eras and world regions

**Use technological and organizational capabilities, not era-wide speed bonuses.** These examples describe different construction systems and evidence bases; they are not successive stages every society must pass through.

| Setting | Relevant evidence or distinction | Implication for TCE |
| --- | --- | --- |
| **Forager construction, European experimental analogues** | The Polish reconstructions required substantial work even for small structures; collecting grass for one roof alone took about **60 person-hours**. [EXARC](https://exarc.net/sites/default/files/2025-11/Two%20Reconstructions%20of%20Prehistoric%20Houses%20from%20Torun%20%28Poland%29%20_%20The%20EXARC%20Journal.pdf) | Light construction need not mean negligible procurement. Mobility and expected service life should affect investment. |
| **Forager monumentality, southeastern North America** | Poverty Point’s Mound A contains approximately **238,500 m³** of fill. Rapid construction is inferred from stratigraphy; exact 30-, 60-, or 90-day schedules are modeled scenarios. [ResearchGate](https://www.researchgate.net/publication/263213175_Building_Mound_A_at_Poverty_Point_Louisiana_Monumental_Public_Architecture_Ritual_Practice_and_Implications_for_Hunter-Gatherer_Complexity) | Do not require farming, monarchy, or a permanent bureaucracy before large cooperative earthworks become possible. |
| **Early-farming configurations** | Direct work records are unavailable for prehistoric examples; reconstruction experiments and later technological analogues provide the quantitative basis used here. [EXARC](https://exarc.net/sites/default/files/2025-11/Two%20Reconstructions%20of%20Prehistoric%20Houses%20from%20Torun%20%28Poland%29%20_%20The%20EXARC%20Journal.pdf) | Allow post-built, earth-walled, and block-built alternatives. Treat inferred rates as uncertain and couple labor availability to subsistence. |
| **Ancient Egypt** | Specialized workers’ communities, private commissions, remuneration in goods, and compulsory obligations all occur in the evidence. Exceptional communities should not stand for the entire economy. [eScholarship](https://escholarship.org/content/qt6fr8p2hb/qt6fr8p2hb.pdf?v=lg) | Represent mixed recruitment and provisioning systems, not an undifferentiated “slave workforce.” |
| **Mesoamerica** | Abrams’s Copán estimates distinguish material procurement, manufacture, transport, and assembly; elite residences can involve vastly more labor than modest houses. [Scribd](https://www.scribd.com/document/37073996/0884022544) | Let platforms, finish quality, imported materials, and architectural complexity create disparities. |
| **Andean traditions** | The Cusco replication demonstrates rapid execution by a knowledgeable small crew using local materials, but with restricted project scope. [Academia](https://www.academia.edu/145108875/How_to_Build_a_Drum_Shaped_House_A_Replicative_Experiment_in_Ancestral_Architecture_Cusco_Peru_) | Local expertise and resource proximity can offset the absence of industrial machinery. |
| **Song China** | The *Yingzao Fashi*, promulgated in **1103**, combined modular carpentry with formal material and labor standards for official construction. [University of Hawaii at Hilo](https://hilo.hawaii.edu/languages/chinese/documents/languages/chinese/Chinese_Architecture_and_Metaphor_Song_Culture_in_the_Yingzao_Fashi_Building_Manual-introchap4-Reduced.pdf) | Standardization and sophisticated estimating should be discoverable institutional capabilities, not exclusively industrial technologies. |
| **West African earthen cities** | Djenné’s building trade combines specialized masonry, apprenticeship, professional networks, and architectural maintenance. [Indiana University Press](https://iupress.org/9780253220721/the-masons-of-djenne/) | Earthen construction can sustain skilled urban professions and elaborate architecture; do not equate it with temporary shelter. |
| **Medieval European monumental construction** | Salisbury’s main campaign reached consecration in 1258, but its tower enlargement and spire belong approximately to **1300–1329**. [Salisbury Cathedral](https://www.salisburycathedral.org.uk/discover/history/the-cathedral-that-moved/) | Distinguish initial usability, dedication, later enlargement, and maintenance. “Built over a century” need not mean one continuous unchanged project. |
| **Industrial and modern systems** | Prefabrication moves work into standardized production and distribution; FAO notes that factory setup and transport can offset site savings. [FAOHome](https://www.fao.org/4/s1250e/S1250E0n.htm) | Machines, standard sizes, repeat orders, and supply reliability should change individual operations and economic scale. |

Modern calendars also resist a simple technological ladder. In the US 2023 cohort, permit-to-completion averages were **8.9 months for houses built for sale** and **15.2 months for owner-built houses**. These categories differ in project characteristics as well as organization; the difference is not a clean causal estimate of owner-builder inefficiency. [Eye On Housing](https://eyeonhousing.org/2024/08/single-family-build-time-continues-to-trend-upward-for-2023/)

---

## 4. Stylized facts a correct simulation should reproduce

### Procurement can dominate visible building work

A completed structure should have a construction history extending into forests, pits, workshops, kilns, and transport routes. The Copán example shows how little of total estimated effort may occur during final assembly. [Scribd](https://www.scribd.com/document/37073996/0884022544)

**Test:** moving a material source farther away should increase transport employment and total labor without directly changing the mason’s intrinsic laying rate.

### Collective projects need not imply centralized coercion

Poverty Point supplies an important counterexample to a technology tree in which large construction becomes possible only after agriculture and state formation. Its exact workforce and schedule remain inferential. [ResearchGate](https://www.researchgate.net/publication/263213175_Building_Mound_A_at_Poverty_Point_Louisiana_Monumental_Public_Architecture_Ritual_Practice_and_Implications_for_Hunter-Gatherer_Complexity)

**Test:** sufficiently connected, motivated communities should sometimes complete major work through temporary cooperation.

### “Finished” is a sequence of milestones

Salisbury’s east end preceded consecration by decades, and later vertical expansion followed the main campaign. [Salisbury Cathedral](https://www.salisburycathedral.org.uk/discover/history/the-cathedral-that-moved/)

**Test:** households and institutions should sometimes use completed portions while other work remains unfinished. Later additions must not reset the whole building to “under construction.”

### Maintenance is continuing production

FAO’s tropical roofing guidance anticipates major grass-thatch repairs at roughly **two- to three-year intervals** under its stated conditions—not a universal thatch lifespan. [FAOHome](https://www.fao.org/4/s1250e/S1250E0l.htm)

**Test:** a mature settlement should retain demand for roofers, plasterers, carpenters, and transport even with no population growth.

### Cheap construction is not necessarily cheap occupation

This is a modeling consequence of separate construction and maintenance ledgers: a household may rationally choose a low initial investment despite higher recurring repair labor, especially when credit or secure tenure is limited.

**Test:** changing expected occupancy duration or access to finance should change material choices without requiring a cultural style change.

### Short site labor does not guarantee a short calendar

Modern completion statistics include long periods compared with any single trade’s presence on site. [Eye On Housing](https://eyeonhousing.org/2024/08/single-family-build-time-continues-to-trend-upward-for-2023/)

**Test:** delays in one specialized trade, one delivery, or a curing stage should be able to stall completion despite abundant general labor.

---

## 5. Recommended implementation for TCE

### 5.1 Core entities

Represent a building project with a **directed acyclic graph of work packages**. Each package should contain:

| Field group | Required information |
| --- | --- |
| Quantity | Unit, total quantity, installed quantity |
| Recipe | Inputs per unit; recoverable and wasted fractions |
| Labor | Required hours by role; reference tools and working conditions |
| Dependencies | Previous packages, material readiness, minimum curing or drying |
| Capacity | Working positions, crew composition, lifting or equipment limits |
| Environment | Weather restrictions, exposure damage, access requirements |
| Quality | Workmanship state, defects, rework, inspection requirements |
| Responsibility | Owner, organizer, contractor or labor obligation |
| Provenance | Source, date, evidence type, included/excluded activities, uncertainty |

Use the same schema for houses, workshops, temples, walls, repairs, and extensions. Their differences come from geometry, recipes, finish requirements, and institutions.

### 5.2 Progress should follow the limiting input

For task \(j\), let \(a\_{jr}\) be required reference person-days of role \(r\) per unit, and \(E\_{jr}\) the effective person-days supplied during the interval.

A useful rule is:

\[
\Delta q\_j =
I\_j
\min\left[
q\_{\text{remaining}},
\min\_r\frac{E\_{jr}}{a\_{jr}},
\min\_m\frac{S\_m}{b\_{jm}},
K\_j\Delta t
\right]
\]

Here \(I\_j\) indicates whether prerequisites and environmental conditions are satisfied; \(S\_m\) is usable material stock; \(b\_{jm}\) is material required per unit; and \(K\_j\) is the task’s spatial or equipment capacity.

This prevents surplus helpers from replacing missing specialists, or labor from substituting for absent roof tiles.

**Avoid double-counting inefficiency.** Store whether a source rate describes productive hours or an attended working shift. A shift-based estimating norm may already include ordinary breaks and small interruptions; reducing it again by a generic utilization factor can make construction unrealistically slow.

### 5.3 Let individual agents supply the labor

Agents should choose or be assigned actual activities: cutting timber, preparing mortar, carrying water, operating a cart, laying masonry, supervising, cooking for a work party, or training.

Their time must leave the household’s other activities. Paying a worker does not create an extra person, and conscripting a farmer does not suspend the crop calendar.

A master’s knowledge can affect feasible designs, estimated requirements, rework risk, and apprentice learning. A household without a specialist might choose a simpler structural system rather than becoming completely unable to build.

### 5.4 Aggregate work without making it invisible

For 10k–50k agents, I recommend:

* **Work packages rather than individual bricks as simulated entities.**
* **Crew-level task allocation**, with individual attendance and travel recorded.
* **Event-driven or hourly construction updates**, not structural calculations every rendering frame.
* **Local inventories and reserved deliveries**, preventing two projects from spending the same material.

UE5 can display individual workers and progressively assembled modules while the Rust kernel advances quantities. Visible activity should correspond to ledger activity, but it need not reproduce every physical hammer strike.

For v1, simplify full structural mechanics into grammar feasibility rules, temporary-support requirements, quality checks, exposure damage, and repairable defects.

### 5.5 Represent uncertainty explicitly

Separate three kinds of variation.

**Uncertain knowledge:** the baseline productivity of a technique is imperfectly known. Sample or select this when creating a scenario or calibrating a technique.

**Persistent project differences:** access, stone hardness, timber dimensions, skill, and design complexity.

**Daily variation:** weather, illness, interruptions, attendance, and delivery failures.

Do not redraw the historical productivity assumption every day. That converts uncertainty about your model into artificial noise inside the simulated world.

### 5.6 Existing models and games worth borrowing from

**Archaeological energetics.** Abrams’s work supplies the central accounting idea: infer quantities and apply operation-specific labor costs. Borrow the decomposition, but do not treat estimated totals as observed labor records. [Scribd](https://www.scribd.com/document/37073996/0884022544)

**OiKoS.** Archibald, Fitzjohn, and Wilson’s 2026 method connects reconstructed geometry, quantities, production, and labor estimation, with uncertainty analysis. It is a useful architectural-costing precedent, not an agent-based construction scheduler. Its use of historical and later estimating sources also illustrates why transferred rates need explicit provenance. [ResearchGate](https://www.researchgate.net/publication/401437283_A_New_Method_for_Studying_Ancient_Cities_Mapping_and_Estimating_Labor_and_Production_with_OiKoS)

**Workers & Resources: Soviet Republic.** Its construction offices, workers, material deliveries, and industrial supply chains are useful design precedents for separating money from physical construction capacity. TCE should borrow that logistical separation while replacing centralized player allocation with households, firms, and institutions. [Steam Store](https://store.steampowered.com/app/784150/Workers__Resources_Soviet_Republic/)

---

## 6. Source guide, datasets, and evidence limits

### Sources most useful for implementation

| Source | Best use |
| --- | --- |
| **Bengtsson and Whitaker, *Farm Structures in Tropical Climates*, FAO, 1986** | Practical material processes, construction organization, quantity estimation, earth building, timber, and roofing. Regionally useful, not a universal historical productivity dataset. [FAOHome](https://www.fao.org/4/s1250e/S1250E0o.htm) |
| **Government of India, CPWD, *Delhi Analysis of Rates*, 2016** | Explicit labor/material decomposition; especially items **6.1.1**, **7.1.1**, and **12.48**. [Academia](https://www.academia.edu/37038412/2016_GOVERNMENT_OF_INDIA) |
| **Osipowicz, Nowak, and Kuriga, 2015, “Two Reconstructions of Prehistoric Houses from Torun (Poland)”** | Small-building experiments and documentation of procurement and assembly difficulties. [EXARC](https://exarc.net/sites/default/files/2025-11/Two%20Reconstructions%20of%20Prehistoric%20Houses%20from%20Torun%20%28Poland%29%20_%20The%20EXARC%20Journal.pdf) |
| **Earle and colleagues, 2025, “How to Build a Drum-Shaped House”** | Skilled local replication in Peru; useful whole-project observations with stated scope limitations. [Academia](https://www.academia.edu/145108875/How_to_Build_a_Drum_Shaped_House_A_Replicative_Experiment_in_Ancestral_Architecture_Cusco_Peru_) |
| **Abrams, 1998, “Structures as Sites: The Construction Process and Maya Architecture”** | Phase-based labor estimation, differential architectural investment, and reuse. [Scribd](https://www.scribd.com/document/37073996/0884022544) |
| **Ortmann and Kidder, 2013, “Building Mound A at Poverty Point, Louisiana”** | Earthwork rates, construction scenarios, and the limits of inferring social organization from monument size. [ResearchGate](https://www.researchgate.net/publication/263213175_Building_Mound_A_at_Poverty_Point_Louisiana_Monumental_Public_Architecture_Ritual_Practice_and_Implications_for_Hunter-Gatherer_Complexity) |
| **Marchand, 2009, *The Masons of Djenné*** | Apprenticeship, craft organization, expertise, and the social production of earthen architecture. [Indiana University Press](https://iupress.org/9780253220721/the-masons-of-djenne/) |
| **Feng, 2012, *Chinese Architecture and Metaphor*** | Interpretation of *Yingzao Fashi* standardization and craft-specific accounting. [University of Hawaii at Hilo](https://hilo.hawaii.edu/languages/chinese/documents/languages/chinese/Chinese_Architecture_and_Metaphor_Song_Culture_in_the_Yingzao_Fashi_Building_Manual-introchap4-Reduced.pdf) |
| **US Census, Survey of Construction time-series tables** | Modern elapsed construction times, separated by authorization/start/completion and building categories; not person-day estimates. [Census.gov](https://www.census.gov/construction/nrc/data/time.html) |

### What remains thin or contested

**Cross-cultural timber-framing productivity is particularly thin in comparable units.** Published projects often mix preparation, raising, roof construction, and finishing. The proposed framing coefficients should remain low-confidence until TCE’s actual kits are timed or reconstructed analytically.

**Experiments are not direct observations of ancient work.** Participants may lack lifelong expertise, use substitute materials, work around research schedules, or omit household support. Conversely, specialist modern crews can benefit from tools, transport, or accumulated knowledge unavailable in the target setting. The Polish authors explicitly caution against straightforward transfer to prehistoric reality. [EXARC](https://exarc.net/sites/default/files/2025-11/Two%20Reconstructions%20of%20Prehistoric%20Houses%20from%20Torun%20%28Poland%29%20_%20The%20EXARC%20Journal.pdf)

**Monument labor estimates depend on reconstruction choices.** Missing superstructures, reused materials, source locations, finish assumptions, and workday definitions all matter. A modeled labor requirement does not uniquely identify workforce size, construction duration, or political coercion.

**Support labor is easily omitted.** Salisbury’s own account includes women and children carrying goods, tending animals, and supplying food and drink alongside skilled and unskilled building work. A ledger that records only people touching the structure misses part of the construction economy. [Salisbury Cathedral](https://www.salisburycathedral.org.uk/discover/history/the-cathedral-that-moved/)

**Long project histories are not continuous labor histories.** Distinguish initial campaigns, pauses, enlargement, repair, and replacement. Similarly, modern elapsed-time data cannot be converted into person-days without knowing crew attendance.

The strongest v1 implementation is therefore a small, auditable collection of work recipes with explicit boundaries and uncertainty. Let **resource access, skilled people, household availability, transport, institutional coordination, and architectural ambition** determine the resulting schedule. That will produce much more plausible—and more varied—building histories than an era-based construction-speed multiplier.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9290d-70c4-83ea-be94-bc077c6d63d8)
