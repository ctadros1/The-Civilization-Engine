*Visual reference: the Met’s Damascus Room, dated 1707—an affluent reception interior, not a template for an ordinary pre-industrial home.* [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/452102)

# Interior layouts and room use: a simulation-ready report for TCE

## Executive recommendation

**Generate interiors from activities, household organization, climate, wealth, and available equipment—not from an era-specific list of modern room names.** Archaeological work on Pompeii, descriptions of Japanese houses, and documented home-work arrangements in industrial tenements all demonstrate that the same space could accommodate several activities. A room’s name, decoration, or surviving furniture does not uniquely establish its function. [eScholarship](https://escholarship.org/uc/item/8gp5c2nc)

For TCE, distinguish three things:

| Layer | What it represents | Example |
| --- | --- | --- |
| **Physical space** | Geometry, access, openings, heating, surfaces | A plastered room with raised platforms and an oven |
| **Equipment and possessions** | What activities are possible, at what quality and capacity | Bedding, grain containers, grinding equipment, lamps |
| **Current use** | What agents are doing now | Food preparation by day; gathering and sleeping later |

An equally important rendering rule is: **decide whether an interior can be seen before selecting its atlas.** Çatalhöyük houses were entered through roofs; the documented Huizhou house Yin Yu Tang has important windows facing its courtyard; Japanese paper screens and wooden shutters change both light transmission and visibility. Applying transparent, illuminated street windows to all these traditions would be a larger historical error than choosing the wrong chair. [Çatalhöyük Research Project](https://www.catalhoyuk.com/node/56)

The recommendations below separate **historical observations**, **reconstructions**, and **explicitly authored simulation defaults**.

---

## 1. Mechanisms: causal rules the simulation can implement

### 1.1 Space is allocated to activities, not permanently assigned to one function

Represent each space as supporting several activities with different suitability scores. Cooking requires an appropriate heat source; sleeping requires suitable resting space; textile work requires equipment, working clearance, and sufficient light. Some activities can coexist; others compete for the same floor area.

**Implementation rule:** agents reserve activity slots, while movable furnishings change configuration. Deploying bedding can displace dining or craft activity without changing the room’s architectural identity.

This fits both the flexible use of Pompeian domestic spaces and Morse’s nineteenth-century descriptions of Japanese bedding being brought out and put away. It avoids turning every ancient *cubiculum* into a modern bedroom. [ResearchGate](https://www.researchgate.net/publication/312231844_Venus_in_Pompeian_Domestic_Space_Decoration_and_Context)

### 1.2 Heat, smoke, and daylight organize the interior

Give rooms a thermal connection to hearths, ovens, heated platforms, and neighboring spaces. Give openings orientation, size, transmission, and shutter state.

**Implementation rule:** agents favor warmer locations in cold conditions, cooler or shaded locations in hot conditions, and better-lit locations for visually demanding work. Cooking and smoke-producing work incur ventilation penalties.

The northern Chinese **kang** combined cooking-related heat, sleeping, and other domestic functions in one installation. At Çatalhöyük, ovens, roof access, platforms, and storage formed a different spatial arrangement; its experimental reconstruction also demonstrates the importance of reflective white plaster in a dim interior. [HERO](https://hero.epa.gov/reference/2591630/)

### 1.3 Wealth buys options—not a universal furniture count

Do not implement class as “poor: three objects; rich: twelve objects.” Instead, wealth can purchase:

* More space and separation between incompatible activities.
* Better bedding, textiles, surfaces, storage, and lighting.
* Specialized equipment and costly display objects.
* The ability to reserve space for hospitality rather than continuous productive use.

These should be **alternative expenditures**, mediated by local preferences. A spacious, carefully maintained floor-seated reception room should not score as impoverished merely because it lacks European-style chairs. The Damascus reception room and Morse’s Japanese interiors offer sharply different furnishing arrangements from one another and from a Victorian parlor. [The Metropolitan Museum of Art](https://www.metmuseum.org/about-the-met/collection-areas/islamic-art/damascus-room)

**Implementation rule:** households invest when expected comfort, productivity, privacy, or prestige benefits exceed construction, heating, maintenance, and opportunity costs. Curtains, screens, and additional storage can precede structural subdivision.

### 1.4 Household composition and access rules shape layout

Rooms serve households, extended families, tenants, servants, customers, and visitors—not an abstract number of interchangeable occupants.

**Implementation rule:** assign access permissions to spaces and activity slots. Permissions may depend on household membership, tenancy, employment, age, status, ritual rules, or locally evolving gender conventions. Do not hard-code one universal public–private or male–female division.

Haudenosaunee longhouses organized multiple families around shared hearths. Yin Yu Tang accommodated a multigenerational merchant household. Meskell’s work on Deir el-Medina cautions against assigning social identities and functions too confidently from architectural arrangements alone. [New York State Museum](https://www.nysm.nysed.gov/mohawk-haudenosaunee-village/haudenosaunee-longhouse)

### 1.5 Production and retail compete with domestic space

Model the movement of inputs, unfinished goods, finished goods, waste, workers, and customers.

**Implementation rule:** arrange workspaces around this flow:

\[
\text{receiving/storage}\rightarrow\text{processing}\rightarrow
\text{finishing}\rightarrow\text{sale or dispatch}
\]

Allow stages to share a room or spill into a courtyard, porch, street frontage, or dwelling. Separation becomes attractive when congestion, fire, noise, theft, or customer traffic impose sufficient costs.

The reconstructed medieval shop from Horsham combines commercial frontage with rear working space and possible accommodation. The Tenement Museum documents garment production in a family’s front room in 1902: industrialization did not automatically remove work from homes. [Weald & Downland Living Museum](https://www.wealddown.co.uk/buildings/medieval-shop-from-horsham/)

### 1.6 Infrastructure enables equipment, but retrofits consume space

Treat water supply, drainage, fuel delivery, power, and ventilation as prerequisites for specific installations—not automatic benefits of reaching an era.

**Implementation rule:** installing equipment must find physical space and connect to required services. A retrofit can improve sanitation while reducing usable private area.

At 97 Orchard Street, the 1905 installation of hallway toilets and a fireproof shaft reduced particular apartments from 345 to 318 square feet while their three-room arrangement was retained through partition changes. [Tenement Museum Blog](https://tenement-museum.blogspot.com/2010/02/questions-for-curatorial-renovation.html)

### 1.7 Interiors accumulate and change incrementally

Track the acquisition, inheritance, repair, sale, and replacement of furnishings separately from construction.

**Implementation rule:** a new technology changes what can be acquired; it does not replace every household’s possessions immediately. Renovations alter selected components rather than switching the entire building to a new visual theme.

Yin Yu Tang preserves changes across generations rather than one frozen eighteenth-century moment. Bayleaf’s hall house also underwent later subdivision and chimney insertion. Both are useful models for layered interiors. [pem.org](https://www.pem.org/visit/yin-yu-tang)

### 1.8 Institutions purchase and schedule shared interiors

Households are not the only furnishing agents. Firms, temples, councils, schools, landlords, and other institutions can own equipment, fund lighting, regulate access, and schedule collective activity.

**Implementation rule:** use an institution’s purpose and budget to generate its room program. A council may meet above a market; a ritual institution may fund lighting beyond household affordability.

Titchfield’s market hall combined selling space, an upper meeting chamber, and a probable lock-up rather than conforming to a single-purpose modern building category. [Weald & Downland Living Museum](https://www.wealddown.co.uk/buildings/market-hall-from-titchfield/)

---

## 2. Parameters: empirical anchors and explicit simulation defaults

### 2.1 Historical and observational values

**Confidence concerns the named case, not its applicability worldwide.** “High” means a clearly documented measurement or reported configuration; “medium” includes reconstruction, interpretation, and historical synthesis. None of these rows establishes a universal distribution for an era.

| Parameter | Value and units | Scope and source | Confidence |
| --- | --- | --- | --- |
| Whole-house area | **40–120 m²; mean approximately 72 m²** | Deir el-Medina, Egyptian state-artisan settlement; not an Egyptian peasant average. Meskell, 1998. [ResearchGate](https://www.researchgate.net/publication/227331125_An_Archaeology_of_Social_Relations_in_an_Egyptian_Village) | Medium–high, local |
| Room areas within those houses | First room **8–24 m²**; second **14–26 m²**; small side rooms **3–6 m²** | Deir el-Medina. Functional names should remain uncertain. [ResearchGate](https://www.researchgate.net/publication/227331125_An_Archaeology_of_Social_Relations_in_an_Egyptian_Village) | Medium, local |
| Shared domestic heating | **2 families per hearth** | New York State Museum’s Haudenosaunee longhouse interpretation. [New York State Museum](https://www.nysm.nysed.gov/mohawk-haudenosaunee-village/haudenosaunee-longhouse) | Medium, reconstructed tradition |
| Longhouse circulation and platforms | Central aisle about **3.05 m** wide; platforms about **0.30 m** above the floor | Same interpretation; these are not whole-building dimensions. [New York State Museum](https://www.nysm.nysed.gov/mohawk-haudenosaunee-village/haudenosaunee-longhouse) | Medium |
| Room modules | Examples of **4.5-, 6-, and 8-mat rooms** | Morse’s 1886 Japanese house descriptions. Preserve mat-count modules rather than assuming one universal square-metre conversion. [Wikisource](https://en.wikisource.org/wiki/Japanese_Homes_and_Their_Surroundings/Chapter_3) | Medium, selected examples |
| Large merchant-house accommodation | **16 bedrooms**; at times **up to 30 residents across 3 generations** | Yin Yu Tang, Huizhou; an affluent extended household, not a national norm. [pem.org](https://www.pem.org/visit/yin-yu-tang) | High, named house |
| Tenement apartment area before/after retrofit | **345 → 318 ft²**, equivalent to **32.1 → 29.5 m²** | Particular apartments at 97 Orchard Street, 1905 alterations. [Tenement Museum Blog](https://tenement-museum.blogspot.com/2010/02/questions-for-curatorial-renovation.html) | High, named building |
| Workshop equipment sharing | Reported workforce of **12 men**, with **3 benches** | Witley joiner’s shop, late nineteenth/early twentieth century. This is not proof of four simultaneous interaction positions per bench. [Weald & Downland Living Museum](https://www.wealddown.co.uk/buildings/joiners-shop-from-witley/) | Medium–high, local |
| Wax-candle luminous flux | Approximately **13 lumens per candle** | Benchmark in Fouquet and Pearson’s historical lighting study; candle designs varied. [Canadian Scotobiology Group](https://www.csbg.ca/BofD/2006%20Fouquet%20-%207%20Centuries%20Light%20Energy.pdf) | Medium |
| Electric-lamp comparison | Approximately **700 lumens from a 60 W incandescent lamp** | A particular technological benchmark, not the output of all electric lights. [Canadian Scotobiology Group](https://www.csbg.ca/BofD/2006%20Fouquet%20-%207%20Centuries%20Light%20Energy.pdf) | High as benchmark |
| Sleep onset after sunset | Mean approximately **3.3 hours**; group/season means **2.5–4.4 hours** | Yetish et al.’s Hadza, San, and Tsimane field study. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/yetishetal2015.pdf) | Medium; limited populations |
| Measured sleep versus sleep period | Sleep **5.7–7.1 h/night**; sleep period **6.9–8.5 h/night** | Same study. These are distinct measures, not universal biological requirements or historical norms. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/yetishetal2015.pdf) | Medium; limited populations |

**Do not fit a worldwide “pre-industrial room size” distribution to these examples.** They differ in household definition, construction, social position, date, and preservation. Use them as localized calibration cases.

### 2.2 Lighting: distinguish output, illumination, and affordability

A lumen measures total visible-light output. Lux measures illumination on a surface. For a simplified direct-light calculation:

\[
E\_{\rm task}
\approx
\sum\_j
\frac{I\_j(\theta)\max(0,\cos\alpha\_j)}{r\_j^2}
+E\_{\rm daylight}+E\_{\rm reflected}
\]

Here \(E\) is lux, \(I\) is luminous intensity in candelas, and \(r\) is distance in metres.

**Derived illustration, not a measured historical room:** distributing 13 lumens uniformly gives approximately 1.03 candelas. A surface facing that source receives roughly **4.1 lux at 0.5 m**, **1.0 lux at 1 m**, and **0.26 lux at 2 m**, before reflections and obstructions.

The visual consequence is concentrated light around a source—not a uniformly orange room. For TCE, let agents bring fine work nearer a lamp or share a lit working area. Do not implement one rigid historical “reading threshold”; task detail and penalties are more useful than a binary rule.

The hourly economic model can remain simple:

\[
c\_{\rm light}
=q\_{\rm fuel}p\_{\rm fuel}
+\tau\_{\rm tending}w
+\frac{c\_{\rm fixture}}{H\_{\rm service}}
\]

All terms have units of currency per burning hour. Fuel is consumed from an actual household or institutional inventory.

Historically, light sources differed in price, smoke, maintenance, and infrastructure requirements. Tallow, wax, rushlights, oil, gas, and electric lighting should therefore coexist where their supply conditions permit, rather than replace one another globally. Fouquet and Pearson’s evidence is particularly useful for these economic relationships, but is predominantly British. [Canadian Scotobiology Group](https://www.csbg.ca/BofD/2006%20Fouquet%20-%207%20Centuries%20Light%20Energy.pdf)

### 2.3 Initial engine values requiring calibration

These are **authored test settings, not claimed historical measurements**.

| Engine parameter | Suggested starting value or test range | Status and intended use |
| --- | --- | --- |
| Deployable adult floor-bedding footprint | **1.5–2.5 m²/person** | Low-confidence geometric prior. Excludes circulation, storage, and working space; shared and child bedding need separate rules. |
| Small oil-lamp test configuration | Start at **10 lm**, burning **10 mL/h**; sensitivity tests at **3–30 lm** and **5–20 mL/h** | Uncalibrated engineering prior. Replace by lamp-, wick-, and fuel-specific experimental data; not a universal ancient-lamp range. |
| Ordinary virtual-room depth | Initially **2–6 m** | Atlas-authoring convenience only. Override for halls, longhouses, deep shops, and actual building geometry. |
| Distant visual-state refresh | Event-driven, with a **5–15 simulated-minute** fallback | Engineering proposal, not a performance benchmark. Close-up changes should follow actual activity events. |
| Activity capacity | Explicit object slots plus configurable floor-space capacity | Do not infer household size from chair count or equate one bench with one worker. |

For lamp calibration, Caitlin Lobl’s chapter, **“An Experimental Approach to the Study of the Roman Oil Lamp”** in the 2024 *Bloomsbury Handbook of Experimental Approaches to Roman Archaeology*, is a directly relevant experimental starting point. [Bloomsbury Publishing](https://www.bloomsbury.com/uk/bloomsbury-handbook-of-experimental-approaches-to-roman-archaeology-9781350217836/?fbclid=IwZXh0bgNhZW0CMTAAAR1pgrc4s6rzRjZBU5LMyml6RGI6m4InTDLialYakV5ubmGyZmWqlwV1iaU_aem_5C10qrJtOywlr2z6mhEgRA)

---

## 3. Variation across eras, regions, building types, and classes

### 3.1 Homes: a comparative visual vocabulary

The following are **located examples and traditions**, not universal cultural presets. In TCE, they should inform recombinable architectural practices.

| Context | Characteristic spaces, furnishings, and activities | Implication for TCE |
| --- | --- | --- |
| **Foraging societies: substantial communal houses as well as mobile shelters** | Coast Salish plank houses could combine residential, communal, and ceremonial functions; important carving and painted crest imagery occurred inside. Separately, residue analysis establishes animal-fat lighting in Late Mesolithic northern Europe. [Vancouver Heritage Foundation](https://www.vancouverheritagefoundation.org/house-styles/traditional-coast-salish-plank-houses/) | “Forager” must not automatically mean an empty tent, tiny hut, or absence of lamps. Generate mobility and material constraints separately from subsistence category. |
| **Early farming: Çatalhöyük, central Anatolia** | Roof access, plastered surfaces, raised platforms, ovens, storage spaces, and sometimes painted decoration. Platforms participated in domestic life and could have burials beneath them. [Çatalhöyük Research Project](https://www.catalhoyuk.com/node/56) | Strong low-level platforms and storage silhouettes; few modern freestanding-room categories. Do not add conventional street-facing display windows. |
| **Ancient Egypt: Deir el-Medina** | A sequence of rooms, plastered or decorated surfaces, niches and bench-like installations, small side spaces, and rear food-preparation equipment. Some light entered through high grilles. [ResearchGate](https://www.researchgate.net/publication/227331125_An_Archaeology_of_Social_Relations_in_an_Egyptian_Village) | Use different light levels along the room sequence. Keep disputed features functionally ambiguous rather than giving every raised installation a fixed ritual or sleeping purpose. |
| **Roman Mediterranean** | Domestic assemblages include stools, benches, tables, couches, braziers, oil lamps, candelabra, and containers; elite houses could have painted walls and courtyard-centered arrangements. Room functions were flexible. [Archaeological Museum of Naples](https://www.museoarcheologiconapoli.it/en/portfolio-item/domus-furnishings-from-pompeii/) | Separate elite decorative finishes from the wider object repertoire. A courtyard house is not simply a modern corridor house with Roman wallpaper. |
| **Haudenosaunee longhouses, northeastern North America** | Shared hearths along an aisle, side platforms used for sitting, sleeping and work, mats and furs, overhead or suspended storage, and managed smoke openings. [New York State Museum](https://www.nysm.nysed.gov/mohawk-haudenosaunee-village/haudenosaunee-longhouse) | Represent family territories within one large shared interior. Sleeping, storage, and social boundaries need not be solid-walled rooms. |
| **Classic Maya village households: Cerén, El Salvador** | Exceptionally preserved household buildings and contents show storage, food preparation, sleeping arrangements, and changing uses; one structure interpreted as a former dwelling became a storehouse. [IntechOpen](https://www.intechopen.com/chapters/73203) | Allow a household’s functional “interior” to span several buildings and porches. Conversion to storage should be a real use change. |
| **Yoruba courtyard compounds, Nigeria** | Documented compounds place activities such as cooking, weaving, storytelling, meetings, and dispute settlement in courtyards or verandas as well as enclosed rooms. A recent study examined six houses. [DOI](https://doi.org/10.1016/j.foar.2024.07.015) | Do not squeeze all household activity into enclosed window-visible rooms. Treat this evidence as a limited study of a continuing tradition, not an unchanged ancient norm. |
| **Northern rural China** | The kang integrates domestic heating with sleeping and other activities; cooking heat and domestic space form one system. [HERO](https://hero.epa.gov/reference/2591630/) | The heated platform is a major organizing fixture, not decorative background. Its occupancy should respond to season and temperature. |
| **Huizhou merchant housing, southern China** | Yin Yu Tang provides courtyard-facing lattice windows, carved architectural elements, inherited furnishings, and rooms altered over generations. [pem.org](https://www.pem.org/visit/yin-yu-tang) | Keep this merchant-house tradition distinct from northern kang layouts. Courtyard orientation and multigenerational occupancy matter more than a generic “Chinese” furniture set. |
| **Japan, as documented by Morse in 1886** | Mat-based room modules, movable partitions, paper screens, storage cupboards, alcoves, and bedding deployed for sleep and stored afterward. Wooden shutters could close the exterior at night. [Wikisource](https://en.wikisource.org/wiki/Japanese_Homes_and_Their_Surroundings/Chapter_3) | Render a changing floor arrangement. Paper is translucent, not transparent glazing; an orderly open floor is not evidence of poverty. |
| **South Indian merchant housing** | DakshinaChitra’s Chettiar merchant-house reconstruction includes a central courtyard and Burmese-teak-columned veranda from an 1895 house, with components from another house around 1900. [Dakshinachitra](https://www.dakshinachitra.net/tamilnadu) | Use the courtyard and timber joinery as strong cues, but tag the reference as affluent and composite. Do not distribute its finishes to every South Indian household. |
| **Ottoman Damascus: affluent reception space** | The *qa’a* combined a lower entrance zone and raised reception area with low cushioned seating, cupboards or niches, decorated wood, and stone flooring. [The Metropolitan Museum of Art](https://www.metmuseum.org/about-the-met/collection-areas/islamic-art/damascus-room) | Hospitality and display can occupy a distinct room without Western chair-and-table organization. Preserve changes in level. |
| **Late medieval/early modern England: Bayleaf** | An open hall, service rooms, and family chambers; the museum’s approximately 1540 interpretation includes tables, benches, stools, storage and bedding, informed by inventories. [Weald & Downland Living Museum](https://www.wealddown.co.uk/buildings/bayleaf-farmstead/) | Useful for a comparatively prosperous farm household, not an all-purpose “peasant interior.” Smoke, subdivision, and furnishing quality require separate variables. |
| **Industrial cities** | New York tenement rooms combined domestic life with garment work. At the prosperous end, the Museum of the Home’s 1878 room features patterned surfaces, upholstered furniture, pictures, ceramics, and a piano. [Tenement Museum](https://www.tenement.org/food-experiences/) | Industrial production can increase possessions without eliminating crowding or home work. Show class differences through usable space and equipment as well as decoration. |
| **Modern and contemporary households** | Museum of the Home reconstructions include shared accommodation, a bedroom computer workspace, a Caribbean British front room organized around a television, and a Vietnamese family’s combined living/kitchen activities. [Museum of the Home](https://www.museumofthehome.org.uk/whats-on/rooms-through-time/) | Modern interiors still vary by household organization, migration, tenancy, and work. Avoid one universal suburban nuclear-family template. |

### 3.2 Workshops and shops: identify the process before the style

For production buildings, the most useful visible distinction is often **what is being made and how**, rather than ornament.

The Horsham shop provides a particularly valuable counterexample to modern retail assumptions: its interpretation includes a broad **unglazed counter opening**, a heavy shutter closed at night, goods displayed on rails or shelves, and rear working space. The reconstructed shopfront incorporates evidence from a surviving analogue because the original front was incomplete. [Weald & Downland Living Museum](https://www.wealddown.co.uk/buildings/medieval-shop-from-horsham/)

The following are proposed TCE content recipes:

| Program | Core room/prop set | State changes worth rendering |
| --- | --- | --- |
| **Textile and garment work** | Appropriate loom or sewing equipment, work surface, fiber or cloth storage, baskets, unfinished pieces | Equipment occupied; cloth deployed; stock accumulating; bedding replacing work in a home-work room |
| **Woodworking** | Benches, tools, timber stock, partly finished components, assembly clearance | Timber arriving; active workpiece; finished goods occupying floor space |
| **Heat-intensive craft** | Process-specific hearth/furnace, fuel, tools, working surface, ventilation and waste handling | Fire active or cold; input stock; cooling products; soot and debris |
| **Food production and sale** | Oven or cooking installation, preparation surface, containers, ingredient storage, service opening | Preparation, cooking, selling, cleaning, closed frontage |
| **Small retail** | Counter or transaction edge, containers/shelves, secure stock, weighing or accounting equipment when available | Open/closed shutter; changing quantities; customer-facing display; empty stock |
| **Mechanized production** | Machines, power connections or transmission, input and output staging, circulation aisles | Active machinery; material flow; shift occupancy; maintenance shutdown |

These are **program grammars**, not claims that every tradition used the same fixtures. Connect their equipment to TCE’s actual recipes and technology graph.

### 3.3 Public buildings: purpose, access, and collective furnishing

A public building should not receive a scaled-up domestic atlas. Its spatial organization should express what people do together.

| Institutional program | Recommended spatial logic | Avoid |
| --- | --- | --- |
| **Assembly, court, or council** | Gathering area, speaking/judging position, access hierarchy, optional secure record storage | Automatically giving every early institution modern desks and a permanent chamber |
| **Religious gathering** | Authored ritual focal point or orientation, permitted circulation, appropriate floor or seating arrangements, storage and lighting | Treating every religion as a church with pews and an altar |
| **Teaching** | Instructor position, learner places, equipment and writing media supported by the local knowledge system | Unlocking rows of identical desks merely because schooling exists |
| **Bathing** | Wet and dry zones, water storage/heating, drainage, changing space | A domestic bathroom atlas repeated at public-building scale |
| **Care, lodging, or barracks** | Resting places, circulation, storage, supervision and service areas | Assuming beds, partitions, sanitation, and privacy match modern standards |
| **Administration and archives** | Reception/transaction area, working surfaces, storage appropriate to the record medium, controlled access | Showing books and paper files before the relevant goods exist |

For concrete references, Titchfield demonstrates a combined market–meeting–detention program. At a very different scale and date, Sheikh Zayed Grand Mosque’s documented prayer halls demonstrate the importance of an open carpeted congregation area, columns, lighting, and institution-specific attendance patterns. Neither should be generalized to all public buildings of its region. [Weald & Downland Living Museum](https://www.wealddown.co.uk/buildings/market-hall-from-titchfield/)

For class differentiation **within** a building, assign finishes and equipment by room role: public display, productive work, storage, service, and accommodation. Do not apply one wealth multiplier uniformly to every visible space.

---

## 4. Stylized facts a correct simulation should reproduce

### Multifunctionality persists across technological change

The target pattern is not a monotonic transition from “one primitive room” to “many modern dedicated rooms.” Pompeian houses, Japanese deployable bedding, industrial home work, and contemporary shared accommodation all provide counterexamples. Measure **activities per space** and **time spent reconfiguring it**, not just bedroom counts. [ResearchGate](https://www.researchgate.net/publication/312231844_Venus_in_Pompeian_Domestic_Space_Decoration_and_Context)

### Low furniture counts do not reliably indicate low wealth

Built-in platforms, floor-based furnishing systems, textiles, storage, and architectural decoration can carry much of an interior’s value. TCE should therefore permit both materially rich rooms with open floors and materially poor rooms crowded with tools and necessities. This is a modeling inference from the contrasting documented furnishing systems above. [New York State Museum](https://www.nysm.nysed.gov/mohawk-haudenosaunee-village/haudenosaunee-longhouse)

### Technology adoption should be uneven and persistent

Zhuang and colleagues report approximately **67 million kangs serving 175 million people in 2004**. This is an especially useful counterexample to deleting an older domestic technology because industrial or modern technologies exist elsewhere. The figures are historical estimates for that date, not current counts. [HERO](https://hero.epa.gov/reference/2591630/)

### Better infrastructure can reduce private floor area

The 97 Orchard Street example lost approximately **7.8%** of the affected apartment area during the cited sanitation retrofit—calculated from 345 to 318 square feet. The simulation should allow beneficial improvements to carry spatial costs. [Tenement Museum Blog](https://tenement-museum.blogspot.com/2010/02/questions-for-curatorial-renovation.html)

### Cheap lighting should change activity more than a cosmetic lamp swap

Fouquet and Pearson estimate that British lighting-service prices fell by **more than 3,000-fold between 1800 and 2000**, while per-capita light consumption rose approximately **6,500-fold**. These are reconstructed national series, not coefficients to apply to every society. They support modeling affordability, quantity, and duration of lighting together. [Canadian Scotobiology Group](https://www.csbg.ca/BofD/2006%20Fouquet%20-%207%20Centuries%20Light%20Energy.pdf)

### Darkness does not mean everyone immediately sleeps

The Hadza–San–Tsimane study found substantial wakefulness after sunset. Separately, Wiessner’s field research documents the social importance of firelight conversation. Nighttime scenes should therefore include gathering and conversation around limited light, not just continuous productive labor or universally sleeping populations. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/yetishetal2015.pdf)

**Segmented sleep remains a qualified modeling option.** Ekirch’s historical argument and the debate around the field studies do not justify forcing either universal two-part sleep or universal uninterrupted sleep onto all pre-industrial agents. Allow seasonal and cultural variation, with uncertain prevalence. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4763365/)

---

## 5. Modeling recommendation for agents, institutions, and fake interiors

### 5.1 Use a room graph plus activity capacities

For 10,000–50,000 agents, I recommend a compact authoritative model rather than continuous interior pathfinding everywhere:

```
Space
  geometry: area, height, shape, level
  access: connected spaces, permissions
  openings: orientation, aperture type, shutter/screen state
  environment: thermal connection, daylight, smoke, services
  affordances: supported activities and capacities

Fixture
  type, material, capabilities, owner
  placement, interaction slots
  condition, age, fuel or consumable inventory

ActivityReservation
  agent or group, space, fixture
  activity, start time, end time

InteriorVisualState
  architectural tradition
  actual furnishings and renovation layers
  current activity, occupancy, light, season
```

Keep the **physical-space graph** distinct from the **social-access graph**. A door may physically connect two rooms while entry remains prohibited to a customer or visitor.

An agent’s choice can use:

\[
U(s,a)=
B\_{\rm equipment}
+B\_{\rm comfort}
+B\_{\rm light}
+B\_{\rm social}
-C\_{\rm distance}
-C\_{\rm crowding}
-C\_{\rm conflict}
\]

Access restrictions are hard constraints; discomfort and crowding are usually soft costs. Household meals, shared heating, and some sleeping arrangements should be coordinated group decisions rather than independent random choices.

Do not impose an abrupt occupancy cutoff that makes excess residents vanish. Overcrowding should reduce comfort, privacy, circulation, or productive capacity and potentially encourage migration, partitioning, expansion, or renting additional space.

### 5.2 Let institutions own the relevant decisions

A **household** allocates sleeping places, stores food, buys lamps, and deploys bedding. A **firm** owns work equipment and schedules production. A **landlord** controls partitions and service investment. A **religious or civic institution** maintains collective rooms and pays for scheduled use.

This separates ownership from physical location: a rented workroom, shared courtyard, and family dwelling need not belong to the same decision-maker.

At distance, generate visual snapshots from these authoritative reservations. Do not run an independent “fake population” inside windows that contradicts the agent model.

### 5.3 Build atlases from separable layers

A suitable selection key is:

\[
\text{interior}=
\text{room geometry}
+\text{architectural tradition}
+\text{activity equipment}
+\text{wealth choices}
+\text{age/repair}
+\text{current state}
\]

**Avoid an era × class × building-type texture explosion.**

As an initial authoring proposal, use roughly **12–16 geometric families**, including single rooms, platform rooms, courtyard-facing rooms, partitioned suites, deep commercial bays, work bays, large halls, dormitory-like spaces, and wet/service rooms. Add compatible furnishing layers rather than baking every combination.

A room can have a small number of meaningful states—idle, working, eating/gathering, sleeping—while fire, lamp, shutter, and stock levels remain separate overlays.

For every authored room recipe, store:

| Metadata | Why it matters |
| --- | --- |
| Place/tradition and source date | Prevents accidental mixtures presented as one historical reconstruction |
| Household or institutional type | Distinguishes merchant house, laborer’s dwelling, workshop, and public hall |
| Required capabilities and goods | Prevents unsupported machines, lighting, writing media, or plumbing |
| Fixed versus movable furnishings | Supports daily rearrangement |
| Aperture compatibility | Determines whether the room can actually be seen |
| Source status | Excavated, described, reconstructed, or speculative |
| Image and asset rights | Separates reference access from permission to reuse pixels |

### 5.4 Render the opening correctly

Use distinct treatments for:

**Open aperture:** visible interior, with geometry and exposure appropriate to the view.

**Transparent glazing:** interior plus reflection and transmission; not an unconditionally bright billboard.

**Translucent paper or fabric:** transmitted light and limited silhouettes, not a clear view of furniture.

**Closed shutter or opaque screen:** no visible room.

**Roof-lit or courtyard-oriented dwelling:** do not invent street windows merely to expose an atlas.

For adjacent windows, share a room identity where appropriate. Otherwise a large hall becomes a row of implausibly identical little boxes.

Joost van Dongen’s **interior mapping** technique provides the relevant rendering foundation: intersect view rays with virtual room surfaces rather than constructing full room geometry. It solves the visual approximation, not the historical selection or occupancy problem. [Joost's Dev Blog](https://joostdevblog.blogspot.com/2018/09/interior-mapping-real-rooms-without.html)

Close to the camera, add a few depth-correct props or animated silhouettes where they materially improve the view. Keep them tied to the room’s actual activity state.

### 5.5 Existing models worth adapting

| Model or dataset | Useful component | Important limitation |
| --- | --- | --- |
| **Merrell, Schkufza & Koltun, 2010, “Computer-Generated Residential Building Layouts”** | Learning room programs and optimizing layouts under constraints | Modern residential training assumptions should not become universal historical room requirements. [Vladlen Koltun](https://vladlen.info/publications/computer-generated-residential-building-layouts/) |
| **Richardson, Thomson & Infield, 2008, domestic occupancy model** | Time-use-based stochastic occupancy for coarse simulation | Useful for aggregate validation or distant scheduling, not as a second population contradicting TCE’s individuals. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0378778808000467) |
| **Modified Swiss Dwellings, 2024** | Approximately **5,300 floor plans and 18,900 apartments** for testing modern layout algorithms | Geographically and technologically specific; unsuitable as a direct ancient-household prior. [arXiv](https://arxiv.org/abs/2407.10121) |
| **Van Dongen, interior mapping** | Window-view rendering without full interior geometry | Requires TCE’s own room metadata, historical content, and consistent activity state. [Joost's Dev Blog](https://joostdevblog.blogspot.com/2018/09/interior-mapping-real-rooms-without.html) |

---

## 6. Sources, visual references, datasets, and evidence limits

### 6.1 Visual references suitable for atlas development

These collections are useful for studying **geometry, surface treatment, furnishings, and object arrangement**. Publicly viewable does not automatically mean licensed for commercial texture reuse.

| Reference | Best use | Qualification |
| --- | --- | --- |
| **Met: Damascus Room, accession 1970.170** | Raised/lowered floor zones, cupboards, painted wood, reception arrangement | The cited collection image is marked public domain. The museum’s broad viewing opening was not the original arrangement; do not copy the gallery presentation as a house window. [The Metropolitan Museum of Art](https://www.metmuseum.org/art/collection/search/452102) |
| **Edward S. Morse, *Japanese Homes and Their Surroundings* (1886)** | Plans, sections, partitions, cupboards, bedding, and joinery drawings | Public-domain historical drawings; retain date and observer bias, and check the digitized item’s conditions. [Wikisource](https://en.wikisource.org/wiki/Japanese_Homes_and_Their_Surroundings/Chapter_3) |
| **Peabody Essex Museum: Yin Yu Tang** | Courtyard-facing rooms, latticework, furnishings, and a virtual tour | A house with multiple historical layers, not a single-period set; photographs carry rights information. [pem.org](https://www.pem.org/visit/yin-yu-tang) |
| **Çatalhöyük Research Project: experimental house** | Plaster, platforms, storage, roof access, and restrained daylight | Experimental reconstruction, not an intact excavated interior. [Çatalhöyük Research Project](https://www.catalhoyuk.com/node/731) |
| **New York State Museum: Haudenosaunee longhouse** | Shared-hearth layout, platforms, hanging and overhead storage | A documented interpretive reconstruction; preserve cultural specificity. [New York State Museum](https://www.nysm.nysed.gov/mohawk-haudenosaunee-village/haudenosaunee-longhouse) |
| **DakshinaChitra** | South Indian courtyard and veranda construction; occupational and regional house comparisons | Relocated/reconstructed buildings, sometimes assembled from multiple houses; permission is needed for protected imagery. [Dakshinachitra](https://www.dakshinachitra.net/tamilnadu) |
| **Weald & Downland: Bayleaf, Horsham shop, Witley joiner’s shop** | Domestic inventories, shuttered retail frontage, workbench layouts | Distinguish original fabric, comparative reconstruction, and replica furnishings. [Weald & Downland Living Museum](https://www.wealddown.co.uk/buildings/bayleaf-farmstead/) |
| **Museum of the Home: Rooms Through Time** | Dated industrial and modern interiors, class and migration differences | Curated reconstructions. Its future-looking room is speculative and should not enter the historical evidence set. [Museum of the Home](https://www.museumofthehome.org.uk/whats-on/rooms-through-time/) |
| **MANN: furnishings from Pompeii** | Individual seats, tables, lamps, braziers, and domestic objects | Strong prop references; survival and collecting favor some materials and households. [Archaeological Museum of Naples](https://www.museoarcheologiconapoli.it/en/portfolio-item/domus-furnishings-from-pompeii/) |
| **Archnet, MIT/Aga Khan documentation** | Plans and photographs for Muslim societies’ domestic and institutional architecture | A large contextual archive; verify rights and the date/function of each selected record. [MIT Libraries](https://libraries.mit.edu/akdc/archnet/) |

**Do not directly treat museum photographs as historical lighting measurements.** Gallery illumination, modern exposure, reconstruction choices, and altered openings change the image. The Damascus Room also illustrates how later varnish can darken surfaces that were originally brighter. [The Metropolitan Museum of Art](https://www.metmuseum.org/about-the-met/collection-areas/islamic-art/damascus-room)

### 6.2 Research and calibration sources

**Household archaeology.** Penelope Allison’s *Pompeian Households: An Analysis of Material Culture* and its digital material cover **30 houses and more than 6,000 artifacts**. This is useful for object co-occurrence and testing assumptions about room function, while remaining a selective Pompeian sample. Lynn Meskell’s 1998 **“An Archaeology of Social Relations in an Egyptian Village”** provides the Deir el-Medina measurements and a valuable critique of overconfident social interpretation. [Cotsen Institute of Archaeology](https://ioa.ucla.edu/press/pompeian-households)

**Exceptional household preservation.** Payson Sheets’s work at Cerén is particularly important for ordinary agricultural households and materials poorly represented at many other sites. Its preservation conditions should not be mistaken for the normal completeness of the archaeological record. [IntechOpen](https://www.intechopen.com/chapters/73203)

**Lighting and nighttime behavior.** Fouquet and Pearson’s 2006 **“Seven Centuries of Energy Services”** supplies long-run lighting economics; Yetish et al.’s 2015 **“Natural Sleep and Its Seasonal Variations in Three Pre-industrial Societies”** supplies measured nighttime behavior; Ekirch’s 2016 **“Segmented Sleep in Preindustrial Societies”** supplies an important counterpoint about historical interpretation. [Canadian Scotobiology Group](https://www.csbg.ca/BofD/2006%20Fouquet%20-%207%20Centuries%20Light%20Energy.pdf)

**Modern household calibration.** The DHS Indicator Explorer includes persons per sleeping room and household-service indicators. Eurostat’s `ilc_lvho03` and related tables describe rooms per person by tenure or household characteristics. Use named country-years and consistent denominators: a sleeping room and a statistically defined room are not interchangeable measures. [DHS Program](https://dhsprogram.com/data/Indicator-Explorer.cfm?indgrpid=16)

**Activity schedules.** The American Time Use Survey provides public diary data suitable for calibrating modern activity durations and variation. It is not a room-level dataset and should not be projected directly onto prehistoric households. [Bureau of Labor Statistics](https://www.bls.gov/tus/data.htm)

### 6.3 Contested claims and thin evidence

**Room function is often uncertain.** A floor plan, wall painting, or object findspot rarely establishes one exclusive activity. Deir el-Medina’s raised installations and Pompeian room labels are good cases for storing competing interpretations rather than choosing an unsupported definitive function. [ResearchGate](https://www.researchgate.net/publication/227331125_An_Archaeology_of_Social_Relations_in_an_Egyptian_Village)

**Preservation is unequal.** Portable valuables may be removed before abandonment; organic furnishings often disappear. Elite rooms and unusually preserved sites are therefore not neutral samples of everyday interiors. [IntechOpen](https://www.intechopen.com/chapters/73203)

**Modern ethnography is not a frozen image of prehistory.** Use contemporary and recent studies to establish possible mechanisms and variation, not to assign an unchanging interior to a people over millennia.

**Quantitative coverage remains uneven.** The sources assembled here support useful local dimensions, configurations, and lighting benchmarks, but not a defensible universal table of furniture counts, floor area per person, or lamp consumption by historical class. Public-building furnishing distributions and ordinary rural interiors are especially unevenly documented. Keep those parameters explicitly provisional.

## Bottom line

For TCE, **an interior should be the visible consequence of how a household or institution uses space**. A heated platform, a shuttered sales counter, bedding deployed at night, a courtyard full of work, or a lamp shared by several people can communicate more historical reality than a large library of static “era rooms.”

Make the room’s activities, possessions, access rules, and light authoritative. Let the atlas show their consequences.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92926-5ae8-83e9-9f77-afff40f4daa0)
