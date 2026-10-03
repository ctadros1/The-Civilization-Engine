*Reconstructed fifteenth-century Iroquoian longhouse at Crawford Lake, Ontario. This is a reconstruction, not a surviving building or an excavation plan. Its shared interior illustrates why “one building = one household” is an unsuitable default.* [Royal Ontario Museum](https://www.rom.on.ca/magazine/reading-natures-record)

# Housing typologies across cultures and eras

## A simulation-ready report for The Civilization Engine

**TCE should generate housing from household organization, activities, resources, land constraints, hazards, and building traditions—not from an era-to-house lookup table.** Historical examples are best treated as locally documented combinations of these variables.

The requested types are not mutually exclusive categories. A longhouse can be raised on piles; a courtyard compound can contain several dwellings; a row house can become a tenement through subdivision. Even roundhouses could be pile-supported, as at Late Bronze Age Must Farm in Britain. Consequently, TCE should classify a building along several independent dimensions rather than assign it one immutable type. [ANU Press](https://press.anu.edu.au/downloads/press/p129191/html/ch04.html)

The main quantitative limitation is occupancy: archaeological plans often establish dimensions more securely than they establish how many people lived there. **Do not manufacture precise ancient floor-area-per-person figures by dividing every excavated footprint by an assumed household of five.**

---

## 1. Mechanisms: what determines house size and layout?

### 1.1 First separate buildings, dwellings, and households

For TCE, use these definitions:

| Entity or measurement | Recommended meaning |
| --- | --- |
| **Building** | A physical structure with its own construction and deterioration history. |
| **Dwelling unit** | Spaces allocated to a resident group, including rights to shared spaces and facilities. |
| **Household** | A domestic resource-sharing group. It need not equal a nuclear family or occupy exactly one building. |
| **Compound** | A spatial grouping of buildings, courts, yards, and service structures. |
| **Cooking group** | People regularly sharing food preparation; potentially narrower or broader than the household. |
| **Occupancy** | Actual people using a space during a specified interval, distinguished from usual residents. |

These distinctions have direct ethnographic support. Among the Iban, the *bilik* is both a family apartment and a continuing domestic group, while the longhouse includes a shared gallery. Other ethnographic cases distribute a household’s activities or members among several structures. [ANU Press](https://press.anu.edu.au/downloads/press/p129191/html/ch04.html)

Record at least five area measures separately: **external footprint, net enclosed floor area across all storeys, covered-open space, unroofed outdoor space, and dedicated production/storage/animal space**. Retain the original area definition attached to every historical measurement.

For a chosen, consistent definition of domestic floor area:

\[
a\_h=\frac{A\_{\mathrm{exclusive},h}+\sum\_s w\_{hs}A\_{\mathrm{shared},s}}{N\_h}
\]

Here \(N\_h\) is usual resident population and \(w\_{hs}\) allocates a shared space without double-counting it. Keep outdoor access as a separate measure; a courtyard is valuable but is not equivalent to an enclosed winter room.

Also distinguish a household-weighted mean from a person-weighted mean. Larger households receive more weight in the latter; these can yield substantially different descriptions of “the average household.” [Pew Research Center](https://www.pewresearch.org/short-reads/2019/10/01/the-number-of-people-in-the-average-u-s-household-is-going-up-for-the-first-time-in-over-160-years/)

### 1.2 Implementable causal rules

The rules below are recommendations for TCE. The historical evidence supports their direction and plausibility, **not universally calibrated coefficients**.

| Driver | Evidence and interpretation | Rule TCE could implement |
| --- | --- | --- |
| **Household organization** | Iban longhouses combine separate domestic groups with collective circulation and social space. Shared construction does not imply undifferentiated ownership. [ANU Press](https://press.anu.edu.au/downloads/press/p129191/html/ch04.html) | Birth, marriage, adoption, death, and household fission change claims on space. Responses include crowding, adding a bay, assigning another structure, or moving—not necessarily constructing a new detached house. |
| **Mobility and expected residence duration** | Porčić’s cross-cultural comparison of 11 mobile and 35 sedentary societies found lower house-area-to-household-size ratios among mobile groups. [Sage Journals](https://journals.sagepub.com/doi/10.1177/1069397111423889) | Compare the labor cost of durable construction with expected future use. Frequent moves favor portable components, smaller enclosed spaces, and outdoor activity areas. Seasonal return can justify maintaining a substantial house. |
| **Material and construction systems** | Early Neolithic timber houses show repeated structural arrangements and relatively constrained widths despite varying lengths. [ResearchGate](https://www.researchgate.net/publication/275951209_Architecture_of_the_Linearbandkeramik_settlement_at_Balatonszarszo-Kis-erdei-dulo_in_central_Transdanubia) | Generate feasible spans and bays from available timber, joints, posts, masonry, and roofing. Expand by adding supported modules rather than freely stretching a roof mesh. |
| **Production and storage** | Houses can be productive and public institutions, not merely private sleeping containers. Excavated Swahili stonehouses demonstrate connections between household space and wider economic and political activity. [Pure York](https://pure.york.ac.uk/portal/en/publications/the-public-life-of-the-swahili-stonehouse-14th-15th-centuries-ad/) | Add activity demand for food processing, craft work, trade, storage, guests, and dependents. Derive storage from inventories and storage technology rather than a fixed percentage of bedrooms. |
| **Access, privacy, and social rank** | Research on Chinese courtyard houses examines spatial hierarchy through access relationships, not merely room counts. [MDPI](https://www.mdpi.com/2071-1050/11/6/1582) | Give spaces permission rules and access depth. Guest reception, household ritual, intimate activities, and service work can require different routes or degrees of separation. Preferences belong to institutions and learned norms, not immutable ethnic attributes. |
| **Wealth and status** | In Teotihuacan, elite compounds devoted much more space to common areas than intermediate-status compounds. Greater wealth did not simply produce proportionately more individual rooms. [Cambridge University Press](https://www.cambridge.org/core/journals/ancient-mesoamerica/article/apartment-compounds-households-and-population-in-the-ancient-city-of-teotihuacan-mexico/1BED268FF26F27FD2A64B38851EF32D0) | Let discretionary investment buy larger courts, reception spaces, decoration, durable materials, servants’ accommodation, or additional properties—not just increased sleeping area. |
| **Parcel constraints and tenure** | UrbanSim provides an explicit modeling precedent for interactions among households, buildings, parcels, prices, and development feasibility. It is a modern model, not evidence that all historical allocation operated through markets. [UrbanSim Cloud](https://cloud.urbansim.com/docs/general/documentation/urbansim.html) | Under land pressure, compare subdivision, shared walls, extra storeys, smaller units, and relocation. Depending on institutions, allocation may be decided by households, landlords, corporate kin groups, religious bodies, or officials. |
| **Adaptation and reuse** | At Ostia, paired apartments could be separated by blocking connecting doors. Housing arrangements could change without replacing the surrounding structure. [Ostia Antica](https://www.ostia-antica.org/regio3/9/9.htm) | Preserve the existing building when occupancy changes. Permit doors to be blocked or opened, rooms to change use, and units to merge or split. Require viable access and structural support. |

### 1.3 Climate and materials: conditional advantages, not automatic bonuses

| Environment or constraint | Appropriate mechanism | Important qualification |
| --- | --- | --- |
| **Hot, arid conditions** | Evaluate shading, solar exposure, thermal mass, night ventilation, and access to cooler protected spaces. | Do not attach a universal cooling bonus to courtyards or thick earth walls. Field/modeling work in Upper Egypt found important differences among spaces, including exposed courtyards. [MDPI](https://www.mdpi.com/2075-5309/15/24/4450) |
| **Warm, humid conditions** | Reward effective airflow, protection from driving rain, shaded working areas, and construction that can dry. | A courtyard can perform differently in humid conditions; studies in Colima explicitly assess the interaction of form and local climate. [DOI](https://doi.org/10.1186/s40494-022-00820-4) |
| **Hot summers and cold winters** | Evaluate summer and winter performance separately. Allow seasonal opening, screening, and room use. | A Chongqing courtyard-house study found trade-offs: features beneficial in summer could reduce useful winter solar gains. [DOI](https://doi.org/10.3390%2Fen12061042) |
| **Cold conditions** | Model heat loss through the envelope, exposure, infiltration, and the amount of space actually heated. | Inuvialuit sod-house documentation includes protected entrance passages and raised sleeping platforms. Such arrangements are functional layouts, not just visual styles. [Herschel](https://herschel.preserve.ucalgary.ca/sites/inuvialuit-sod-house/) |
| **Flooding** | Compare floor elevation with flood depth; separately model foundation failure, scour, and lateral loads. | Thai structural research shows that raising a house does not make it flood-proof: connections and columns remain vulnerable. [Thai-Journal Online](https://ph02.tci-thaijo.org/index.php/ennrj/article/view/223488) |
| **Limited construction materials** | Price the actual bill of materials, skilled labor, transport, repairs, and replacement. | Material substitution should alter structural options and maintenance needs. Documentation of Asante buildings records changes from traditional materials to metal roofing and concrete. [Smithsonian Institution](https://www.si.edu/object/archives/components/sova-eepa-1973-001-ref32975) |

**Modeling implication:** climatic fitness should emerge from physical attributes and seasonal use. “Desert culture,” “tropical culture,” and “northern culture” should not directly select a single house form.

---

## 2. Parameters: documented dimensions, occupancy, and proposed priors

### Confidence notation

**H:** strong documentation for the stated local observation or survey statistic.  
**M:** reconstructed, adjusted, based on a limited sample, or definition-sensitive.  
**L:** weakly constrained inference, especially occupancy inferred from architecture.  
**P:** proposed simulation parameter—not an empirical estimate.

A high-confidence site measurement can still be a low-confidence parameter for an entire culture or era.

### 2.1 Dimensions and structural units

These are **calibration examples, not global “typical house” ranges**. Calculated areas are identified explicitly.

| Type and context | Documented quantity | Interpretation for TCE | Confidence and source |
| --- | --- | --- | --- |
| **Early farming longhouse: Biskupice, southern Poland, Linearbandkeramik** | House 3 reconstructed at approximately **20–25 m × 7 m**; rectangular envelope **140–175 m²**, calculated. | Calibrate a timber-post building’s envelope. Do not interpret the entire footprint as living space or infer its occupants from size alone. | **M**; excavation-based study. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S2352409X25000951) |
| **Roundhouses: Glastonbury Lake Village, Iron Age Britain** | Diameters approximately **5–8 m**; circular plan areas approximately **20–50 m²**, calculated. | Useful circular-envelope range for this context; net usable space depends on walls, posts, and internal installations. | **M**; archaeological reconstruction documentation. [South West Heritage Trust](https://swheritage.org.uk/avalon-archaeology/our-story/iron-age-roundhouse/) |
| **Larger roundhouses: Chisenbury Midden, Britain** | Two houses approximately **11 m diameter**; circular envelope approximately **95 m²**, calculated. | A counterexample to treating roundhouses as uniformly tiny. | **M**; excavators’ report. [Wessex Archaeology](https://www.wessexarch.co.uk/news/round-houses-found-chisenbury-midden) |
| **Haudenosaunee longhouse** | Common width approximately **20 ft / 6.1 m**; described compartments approximately **20 ft / 6.1 m long**, accommodating two family groups on opposite sides of the aisle. | Repeated module approximately **37 m²**, including circulation—not 37 m² of private space per family. | **M** for generalization; New York State Museum. [New York State Museum](https://www.nysm.nysed.gov/mohawk-haudenosaunee-village/haudenosaunee-longhouse) |
| **Iban longhouses, Paku–Saribas, Borneo** | Sample of **33 longhouses**: **6–39 bilik-family units**, mean **16.5**. | Sample the number of domestic units independently of household size. These are family-unit counts, not people. | **H** for the ethnographic sample; **L** for extrapolation. [ANU Press](https://press.anu.edu.au/downloads/press/p129191/html/ch04.html) |
| **Pit houses: Jōmon Japan** | An archaeological overview describes ordinary oval/circular examples approximately **3–4 m long**. | Length is not necessarily diameter; do not turn this automatically into a circular floor area. | **M**; archaeological synthesis. [Sainsbury Institute](https://dig.sainsbury-institute.org/orjach/earliest-villages-cemeteries/) |
| **Large pit structure: Sannai Maruyama** | Largest example approximately **32 m × 9.8 m**, with reported floor area approximately **250 m²**. | The bounding rectangle overstates the actual floor. Possible communal/workshop functions make ordinary household occupancy assumptions unsuitable. | **H/M** dimensions; **L** function and occupancy. [特別史跡「三内丸山遺跡」](https://sannaimaruyama.pref.aomori.jp/english/about/remains-dwellings/) |
| **Raised Thai house** | Official architectural overview describes floors commonly raised **2 m or more**. | A regional elevation parameter; not a universal minimum for stilt houses. Undercrofts and platforms require separate activity areas. | **M**; documented architectural overview. [Thailand Foundation](https://thailandfoundation.or.th/th/ruean-thai-understanding-traditional-thai-houses/) |
| **Courtyard house: Mohenjo-daro, House VIII** | Central courtyard approximately **32 ft square**, or **9.75 × 9.75 m ≈ 95 m²**, calculated. | This is **open courtyard area**, not house floor area. Marshall interpreted the house as an upper-status example. | **M**; published excavation description. [Harappa](https://www.harappa.com/blog/what-was-ancient-indus-house) |
| **Roman apartments: Case a Giardino, Ostia, second century CE** | Published ground-floor apartment area approximately **220 m²**; connected pairs approximately **440 m²**. | Affluent apartments existed. “Insula” should not automatically generate cramped poor housing. | **M**; archaeological building catalog. [Ostia Antica](https://www.ostia-antica.org/regio3/9/9.htm) |
| **Row-house units: Edo nagaya, Japan** | Described units **2.7 × 3.6 m** and **3.6 × 3.6 m**: approximately **9.7 and 13.0 m²**, calculated. | Compact dwellings with substantial time-sharing of space. Areas here use the dimensions, not the museum page’s inconsistent conversion of *tsubo*. | **M**; museum reconstruction documentation. [江戸東京博物館](https://edo-tokyo-museum.or.jp/en/p-exhibition/) |
| **Industrial tenement: 97 Orchard Street, New York, built 1863** | **5 storeys**, initially **22 apartments**, each about **325 ft² / 30.2 m²**, with **3 rooms**. | Distinguish building scale from unit scale; treat 30.2 m² as reported apartment area, not a precisely harmonized modern net-area measurement. | **H** reported configuration; **M** area comparability. [tenement.org](https://www.tenement.org/explore/97-orchard-street/) |

### 2.2 Floor area per person and household size

A foundational warning comes from Barton McCaul Brown’s 1987 reanalysis of “Naroll’s constant.” Brown proposed approximately **6 m² per person** as a cross-cultural nonindustrial population-estimation benchmark, rather than Naroll’s earlier **10 m²**. This is neither a biological minimum nor a universal historical mean. It is particularly unsafe when outdoor domestic space, multiple structures, or shared facilities are omitted. [Sage Journals](https://journals.sagepub.com/doi/10.1177/106939718702100101)

| Context | Household or resident count | Area measure | Area per person | Confidence and qualification |
| --- | --- | --- | --- | --- |
| **Rural Iran, landed households, twentieth-century ethnographic data reanalyzed by Brown** | Mean **6.8 people** | Adjusted residential area **37.6 m²** | Approximately **5.5 m²/person** | **M/L**: adjustment for wall thickness was estimated from only two plans. [Sage Journals](https://journals.sagepub.com/doi/10.1177/106939718702100101) |
| **Same Iranian study, landless households** | Mean **5.7 people** | Adjusted residential area **31.3 m²** | Approximately **5.5 m²/person** | **M/L**: same measurement limitation. Larger total houses did not imply more space per person. [Sage Journals](https://journals.sagepub.com/doi/10.1177/106939718702100101) |
| **97 Orchard: Rogarshevsky family in 1908; Baldizzi family, 1928–1935** | **8** and **4** residents, respectively | About **30.2 m²** per apartment | Approximately **3.8** and **7.5 m²/person**, calculated | **H/M** local family histories and reported unit size; not neighborhood averages. [tenement.org](https://www.tenement.org/explore/97-orchard-street/) |
| **United States, 1790 and 2010** | Mean **5.79** and **2.58 people/household** | No matched floor-area measurement used here | Not estimated | **M** for long-run comparability; these are national historical aggregates, not class-specific house occupancies. [Pew Research Center](https://www.pewresearch.org/short-reads/2019/10/01/the-number-of-people-in-the-average-u-s-household-is-going-up-for-the-first-time-in-over-160-years/) |
| **England, 2023–2024** | Mean **2.2 people/household**; mortgagors **2.7**, private renters **2.3**, outright owners **1.8** | Separate dwelling-stock statistics below | Not estimated by dividing unmatched aggregates | **H** survey estimates for the stated definitions and period. [GOV.UK](https://www.gov.uk/government/statistics/chapters-for-english-housing-survey-2023-to-2024-headline-findings-on-demographics-and-household-resilience/chapter-1-profile-of-households-and-dwellings) |
| **Hong Kong, 2021** | Varies by household | Census-based domestic living-space measure | Median approximately **16 m²/person** | **H** for the published statistic; a median, not a mean. [DevB](https://www.devb.gov.hk/en/sdev/press/index_id_11447.html) |

For modern class/tenure differentiation, England’s 2023 housing-stock estimates give mean usable dwelling areas of **66 m² for social renting, 75 m² for private renting, and 110 m² for owner occupation**, against **96 m² overall**. These stock statistics include occupied and vacant dwellings, so dividing them by the household means above would mix populations. Tenure is also not identical to social class. [GOV.UK](https://www.gov.uk/government/statistics/chapters-for-english-housing-survey-2023-to-2024-headline-findings-on-demographics-and-household-resilience/chapter-1-profile-of-households-and-dwellings)

For ancient status differentiation, Teotihuacan supplies an especially useful warning:

| Teotihuacan category | Mean total compound area per identified dwelling | Common area as share of compound |
| --- | --- | --- |
| High-status | **1,572.9 m²** | **70.8%** |
| Intermediate-status | **391.1 m²** | **38.5%** |
| Temple housing | **334.5 m²** | **36.5%** |

These are compound-level allocations, **not indoor apartment areas**. Common space includes patios, courts, light wells, and porticos. Occupancy remains dependent on demographic assumptions. [Cambridge University Press](https://www.cambridge.org/core/journals/ancient-mesoamerica/article/apartment-compounds-households-and-population-in-the-ancient-city-of-teotihuacan-mexico/1BED268FF26F27FD2A64B38851EF32D0)

### 2.3 Proposed starting parameters for TCE

Where evidence is insufficient, use explicit authored priors rather than disguised historical estimates.

A useful initial demand function is:

\[
A\_{\mathrm{desired}}=a\_0+bN+A\_{\mathrm{production}}+A\_{\mathrm{status}}
\]

The shared base \(a\_0\) introduces economies of sharing; additional members need not require a complete duplicate kitchen or entrance. This is a **proposed model**, not a fitted universal law.

| Proposed parameter | Initial sensitivity range | Meaning |
| --- | --- | --- |
| Shared domestic base, \(a\_0\) | **4–10 m²/household** | Core shared activity space in a compact dwelling. **P** |
| Marginal domestic area, \(b\) | **3–7 m²/resident** | Additional desired space, before production and status additions. **P** |
| Example central setting | \(a\_0=6\), \(b=4.8\) | Five residents desire **30 m²** before additions. **P** |
| Initial household-size distribution | Concentrate initially around **3–7 residents**, but allow one-person households and a substantial larger-household tail | An initialization choice only; thereafter derive membership from demography and institutions. **P** |
| Conditional size variation | Log-area standard deviation **0.25–0.5** | An initial test range within a narrowly defined context—not one distribution for all houses. **P** |
| Housing reassessment | **Monthly**, plus demographic, economic, and disaster events | Computational scheduling choice. **P** |

These settings should not be imposed on the Edo, Ostia, communal-longhouse, or elite-compound examples merely to make them fit. Nor should unmet desired space prohibit residence: agents can crowd, share, defer construction, or use outside space.

---

## 3. Room programs and variation across eras and regions

### 3.1 Generate activity spaces before assigning room names

A room program is better represented as **activities, installations, access rights, and temporal schedules** than as a mandatory list of “bedroom, kitchen, bathroom, living room.”

The following are starting grammar patterns. Where the archaeological evidence does not identify exact functions, the proposed zoning should remain an alternative rather than be presented as a reconstruction.

| Form | Historically informed program | Proposed extension or conversion rule |
| --- | --- | --- |
| **Haudenosaunee longhouse** | Central passage and hearths; sleeping/storage platforms along the sides; storage toward the ends. Museum documentation describes individual platform compartments and additional storage above and below. [New York State Museum](https://nysm.nysed.gov/mohawk-haudenosaunee-iroquois-longhouse) | Add a repeated longitudinal module, subject to household affiliation, structural support, and circulation. Do not widen every time residents increase. |
| **Iban longhouse** | Private *bilik* units, shared covered *ruai* gallery, and open *tanju* platforms used for activities including drying. These have different access and ownership relationships. [ANU Press](https://press.anu.edu.au/downloads/press/p129191/html/ch04.html) | Add another domestic bay while extending collective circulation. Preserve the distinction between family-controlled and shared space. |
| **Roundhouse** | A proposed flexible central activity area, hearth where evidenced, and peripheral working, sleeping, or storage locations. Excavation may constrain posts and hearths more securely than room functions. [South West Heritage Trust](https://swheritage.org.uk/avalon-archaeology/our-story/iron-age-roundhouse/) | Add a separate structure, change internal installations, or reconstruct a larger roof. Avoid treating a circular envelope as endlessly stretchable. |
| **Pit or earth-sheltered house** | Inuvialuit examples include a central occupied space, raised sleeping platforms, and a protected entrance passage. These are not interchangeable with every Japanese pit-house tradition. [Herschel](https://herschel.preserve.ucalgary.ca/sites/inuvialuit-sod-house/) | Modify entrances and platforms, enlarge where drainage and structure permit, or change to a different seasonal dwelling. |
| **Courtyard house or compound** | Courts connect suites, service areas, reception spaces, and circulation. Chinese layouts can encode access hierarchy; Swahili examples caution against equating the interior with an exclusively private sphere. [MDPI](https://www.mdpi.com/2071-1050/11/6/1582) | Add a wing or another court; partition suites; share or privatize access. Infill should reduce outdoor space and potentially compromise light and movement. |
| **Row house / nagaya** | Edo examples combine an entrance cooking area with a multipurpose living space; bedding and possessions are moved aside rather than requiring permanent dedicated rooms. [江戸東京博物館](https://edo-tokyo-museum.or.jp/en/p-exhibition/) | Add rear accommodation, divide a unit, connect adjacent units, or build upward where feasible. Street-facing trade is optional, not universal. |
| **Insulae and tenements** | Model a circulation system serving multiple units, with services either shared or private. At 97 Orchard, kitchens became sleeping spaces at night, while a front room could accommodate garment production. [tenement.org](https://www.tenement.org/explore/97-orchard-street/) | Subdivide or merge units, alter access, and retrofit services. Distinguish residents from visiting workers and customers. |
| **Portable or seasonal shelter** | Portable sleeping/protection space can coexist with outdoor cooking and processing. Smithsonian Inuvialuit documentation records summer skin tents and tents associated with winter-house entrances. [Smithsonian Institution](https://www.si.edu/es/object/model-summer-single-lodge%3Anmnhanthropology_8349504) | Pack, move, repair, or seasonally reuse components; maintain separate permanent caches or winter structures when appropriate. |

For visible daily life, the same floor patch may support food preparation at one time, socializing at another, and sleeping at night. Permanent installations constrain this flexibility; movable mats, bedding, screens, and work equipment enable it.

### 3.2 Variation through time: no universal architectural ladder

**Foragers and mixed-subsistence communities.** Mobility matters more than a simple forager/farmer label. Jōmon evidence includes substantial pit structures, while Labrador Inuit archaeological research documents winter sod houses, summer tents, and changes from smaller to communal winter houses. TCE therefore needs seasonal residence and aggregation, not “foragers always use tiny temporary huts.” [特別史跡「三内丸山遺跡」](https://sannaimaruyama.pref.aomori.jp/english/about/remains-dwellings/)

**Early farming.** The Linearbandkeramik cases show that substantial timber buildings appeared in early agricultural settings. Yet their large envelopes do not establish large co-resident families: structural bays, storage, production, possible upper-level use, and uncertain room functions complicate interpretation. Keep building dimensions and demographic assumptions independently adjustable. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S2352409X25000951)

**Pre-industrial urban and agrarian societies.** There was no convergence on one house type. Indus and Chinese traditions offer courtyard arrangements; Southeast Asia includes raised houses and longhouses; Mesoamerica includes multi-dwelling compounds; and Roman cities include substantial apartments. These traditions should contribute different grammar rules without becoming exclusive regional templates. [Harappa](https://www.harappa.com/blog/what-was-ancient-indus-house)

**Industrial societies.** Dense rental housing did not invent either apartments or shared buildings. What TCE should model is changes in employment, demand for urban accommodation, finance, construction capabilities, and service provision. The documented New York tenement also shows that home production could persist inside industrial-era urban housing rather than disappearing automatically when factories became available. [tenement.org](https://www.tenement.org/explore/97-orchard-street/)

**Modern societies.** Do not equate “modern” with detached houses, small households, or continuously increasing space. Hong Kong’s 2021 median and England’s tenure-specific figures illustrate different outcomes. Even household size can reverse direction: the U.S. mean increased from **2.58 in 2010 to 2.63 in 2018**. [DevB](https://www.devb.gov.hk/en/sdev/press/index_id_11447.html)

### 3.3 Regional coverage that should influence the grammar

**West Africa:** Yoruba courtyard compounds demonstrate that African housing cannot be reduced to isolated round huts. Asante traditions likewise include buildings organized around courts. However, the surviving UNESCO-listed Asante examples are principally shrines; they should not supply an unqualified size distribution for ordinary homes. [DOI](https://doi.org/10.1016/j.foar.2024.07.015)

**East African coast:** Fourteenth- and fifteenth-century Swahili stonehouses require spaces for social display, economic interaction, and domestic activities. Their inhabitants’ access to space was not necessarily equal; archaeology also investigates work and dependency within these households. [Pure York](https://pure.york.ac.uk/portal/en/publications/the-public-life-of-the-swahili-stonehouse-14th-15th-centuries-ad/)

**East Asia:** Chinese courtyard grammars and Japanese row-house interiors require quite different treatments of circulation, enclosure, and room flexibility. “East Asian house” would therefore be too coarse even before considering rural–urban, class, climatic, and chronological variation. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S2095263520300522)

**Southeast Asia:** Elevation, linked platforms, and shared galleries can be as important as enclosed room area. A raised building should offer usable undercroft and platform spaces rather than consist merely of an ordinary house mesh lifted above the ground. [Thailand Foundation](https://thailandfoundation.or.th/th/ruean-thai-understanding-traditional-thai-houses/)

**The Americas:** Iroquoian longhouses, Inuit seasonal housing, and Teotihuacan compounds represent very different arrangements of domestic groups and collective space. They are not successive steps along a common technological scale. [New York State Museum](https://www.nysm.nysed.gov/mohawk-haudenosaunee-village/haudenosaunee-longhouse)

---

## 4. Stylized facts a correct simulation should reproduce

These are validation targets for appropriately matched scenarios—not expectations that every generated society must show every pattern.

| Pattern | Quantitative or documentary target | Simulation test |
| --- | --- | --- |
| **Building size and household size are distinct distributions.** | The Iban sample spans **6–39 domestic units per longhouse**. [ANU Press](https://press.anu.edu.au/downloads/press/p129191/html/ch04.html) | Buildings can contain many households without turning those residents into one enormous family. |
| **Crowding can change without rebuilding.** | The approximately **30 m²** Orchard Street units accommodated documented households of **4 or 8**. [tenement.org](https://www.tenement.org/explore/97-orchard-street/) | Doubling residents approximately halves area per person while the structure remains unchanged. |
| **Status affects allocation, not only total area.** | Teotihuacan common-space shares: **70.8% elite versus 38.5% intermediate**. [Cambridge University Press](https://www.cambridge.org/core/journals/ancient-mesoamerica/article/apartment-compounds-households-and-population-in-the-ancient-city-of-teotihuacan-mexico/1BED268FF26F27FD2A64B38851EF32D0) | Wealth can enlarge shared/reception space rather than add bedrooms in a fixed ratio. |
| **Structural modules constrain shape.** | Haudenosaunee descriptions emphasize approximately **6.1 m width** and repeated longitudinal compartments. [New York State Museum](https://www.nysm.nysed.gov/mohawk-haudenosaunee-village/haudenosaunee-longhouse) | Growth often changes length or module count more than width. |
| **Room use varies through the day.** | Edo documentation shows movable bedding and possessions within compact multipurpose spaces. [江戸東京博物館](https://edo-tokyo-museum.or.jp/en/p-exhibition/) | Sleeping capacity depends on schedules and installations, not exclusively on objects tagged “bedroom.” |
| **Seasonal dwellings coexist.** | Labrador evidence distinguishes winter sod houses and summer tents. [Memorial University of Newfoundland](https://www.mun.ca/labmetis/sodhouse/fkbg3.html) | One household can maintain more than one residence and shift occupancy seasonally. |
| **House form is not a climatic or technological label.** | Must Farm combined roundhouse construction with piles in Bronze Age Britain. [Historic England](https://historicengland.org.uk/whats-new/research/back-issues/must-farm-timber-platform/) | Round, raised, timber, and multi-unit should be combinable attributes. |
| **Archaeological counts exceed simultaneous occupation.** | Sannai Maruyama’s numerous house remains accumulated through different occupation phases. [特別史跡「三内丸山遺跡」](https://sannaimaruyama.pref.aomori.jp/english/about/restored-pit/) | Compare excavated building totals with simulated accumulated remains, not with a single year’s occupied housing stock. |

A further modeling target is **path dependence**: a newly constructed house and a repeatedly inherited, divided, and extended house should look different even when they accommodate similar populations.

---

## 5. Modeling recommendation for TCE

### 5.1 Represent housing as spatial relationships plus rights

A suitable core model is:

```
Person
  ↔ Household / cooking group / kin group

Household
  ↔ Occupancy agreement
  ↔ Exclusive spaces + rights to shared spaces

Compound / estate
  → Buildings + courtyards + yards + service structures

Building
  → Storeys → Spaces → Activity locations and installations

Building
  → Structural system + materials + condition + modification history

Household / institution
  → Ownership, construction authority, maintenance obligations
```

An occupancy agreement can be customary rather than contractual. It should express who may sleep, cook, store goods, receive visitors, or pass through a space—and who can change those permissions.

Do not give every household unrestricted access to all area inside a compound. Servants, dependents, tenants, guests, and owners may have different permitted spaces even when all count toward resident or daytime population.

### 5.2 Use an orthogonal building grammar

Instead of an enum containing mutually exclusive `Longhouse`, `StiltHouse`, and `CourtyardHouse`, combine:

| Grammar dimension | Example values |
| --- | --- |
| Support and ground relationship | Ground-bearing; pile-supported; partially excavated |
| Plan geometry | Circular; oval; rectilinear; composite |
| Structural repetition | Single span; repeated bays; post grid; load-bearing cells |
| Aggregation | Detached; shared-wall row; linked pavilions; courtyard compound |
| Vertical organization | One floor; loft; multiple storeys |
| Domestic organization | Single household; several domestic units; communal residence |
| Allocation | Owner occupation; rental; corporate kin ownership; institutional provision |
| Activity mix | Primarily domestic; craft-producing; commercial; agricultural; ceremonial |

**Typology names can then be inferred for the UI**, while construction and behavior use the underlying attributes.

A generator should follow:

**housing need → activity program → access graph → structural modules → parcel fitting → openings and environmental checks → cost and permissions → construction.**

Generate several feasible alternatives, not one perfectly optimized design. Select using agent preferences, available builders, local precedent, and institutional approval.

### 5.3 Make household activity—not nominal room count—the capacity constraint

Each space should expose capabilities such as:

`weather_protection`, `thermal_condition`, `sleeping_surface`, `cooking_installation`, `smoke_venting`, `daylight`, `storage_volume`, `work_clearance`, `privacy`, and `permitted_users`.

Then calculate separate capacity constraints:

* People sleeping simultaneously.
* Meals prepared per interval.
* Storage volume and preservation conditions.
* Workplaces available during production.
* Access and circulation under peak occupancy.
* Availability of water, waste disposal, and washing facilities.

These are proposed engineering abstractions. A single “housing capacity = 5 people” property would hide precisely the variation the historical evidence requires.

### 5.4 Model changes to existing housing

The essential actions are not only **build** and **upgrade**, but also **extend, partition, merge, reroof, add a platform, add a storey, convert a room, rent out space, repair, abandon, and reuse**.

Each action should retain costs and constraints from existing geometry. A blocked doorway may solve a privacy problem while worsening circulation. Courtyard infill may accommodate newcomers while reducing usable outdoor space. A new household may gain a unit through partition rather than a new building.

Institutional rules should decide who can authorize these changes. Inheritance, tenancy, kin membership, and public regulation can therefore produce architectural change without scripted historical transitions.

### 5.5 Keep the simulation affordable

For 50,000 people, an assumed average of five people per household implies approximately **10,000 households**; it does not imply 10,000 buildings. The following are proposed implementation choices:

Use event-driven layout generation only when construction or spatial modification occurs. Maintain cached room adjacency, access permissions, capacity, and environmental summaries. Reassess housing periodically and when relevant events occur.

For ordinary buildings, use a small thermal model with a few zones rather than computational fluid dynamics. Distinguish roof, wall, ground, and ventilation effects; account for the spaces actually occupied or heated.

Keep the Rust state authoritative. UE5 can expand the same structural and activity data into detailed meshes, furniture, bedding, and animation locations. Fine navigation and prop placement can depend on visual importance without changing household membership or ownership rules.

### 5.6 Existing models and games worth borrowing from

| Model or precedent | Useful contribution | What must be added or changed |
| --- | --- | --- |
| **UrbanSim** | Explicit households, buildings, parcels, residential choice, and development feasibility. [UrbanSim Cloud](https://cloud.urbansim.com/docs/general/documentation/urbansim%20parcel%20model.html) | Add customary rights, collective ownership, household production, historical construction, and room-level daily life. |
| **Village Ecodynamics Project** | Archaeologically grounded household–landscape simulation and resource constraints in pre-industrial agricultural societies. [CoMSES Net](https://www.comses.net/codebases/2518/releases/1.1.0/) | Add individual occupancy, detailed buildings, and spatial domestic activity. |
| **Merrell, Schkufza & Koltun, 2010, “Computer-Generated Residential Building Layouts”** | Generation from architectural programs, probabilistic relationships, and layout optimization. [Vladlen Koltun](https://vladlen.info/publications/computer-generated-residential-building-layouts/) | Replace contemporary room-program assumptions with culture- and institution-specific activity programs. |
| **Historical Beijing courtyard and hutong grammars** | Hierarchical procedural rules connecting rooms, courtyards, houses, and urban fabric. [DOI](https://doi.org/10.1016/j.foar.2022.12.004) | Couple the grammar to demography, ownership, resources, and incremental adaptation. |
| **Manor Lords** | Flexible residential plots and backyard production are useful gameplay precedents. [Hooded Horse](https://wiki.hoodedhorse.com/Manor_Lords/Burgage_Plot) | Borrow plot adaptability, not a fixed progression in which prosperity automatically selects a predetermined house level. |

None of these should be treated as validation of the entire proposed TCE housing system. They supply complementary components.

---

## 6. Sources, datasets, and uncertainty

### 6.1 Core scholarly sources

| Source | Why it matters |
| --- | --- |
| **Brown, Barton McCaul. 1987. “Population Estimation From Floor Area: a Restudy of ‘Naroll’s Constant.’”** | Foundational critique of cross-cultural area-to-population conversion. [Sage Journals](https://journals.sagepub.com/doi/10.1177/106939718702100101) |
| **Porčić, Marko. 2012. “Effects of Residential Mobility on the Ratio of Average House Floor Area to Average Household Size.”** | Directly tests an important source of variation in area per person. [Sage Journals](https://journals.sagepub.com/doi/10.1177/1069397111423889) |
| **Sather, Clifford. “Posts, Hearths and Thresholds: The Iban Longhouse as a Ritual Structure,” in *Inside Austronesian Houses*.** | Connects physical modules with domestic groups, collective spaces, and social organization. [ANU Press](https://press.anu.edu.au/downloads/press/p129191/html/ch04.html) |
| **Smith et al. 2019. “Apartment Compounds, Households, and Population in the Ancient City of Teotihuacan, Mexico.”** | Separates compounds from dwellings and quantifies status-related spatial differences. [Cambridge University Press](https://www.cambridge.org/core/journals/ancient-mesoamerica/article/apartment-compounds-households-and-population-in-the-ancient-city-of-teotihuacan-mexico/1BED268FF26F27FD2A64B38851EF32D0) |
| **Wynne-Jones, Stephanie. 2013. “The Public Life of the Swahili Stonehouse, 14th–15th Centuries AD.”** | Counters the assumption that houses were exclusively private domestic spaces. [Pure York](https://pure.york.ac.uk/portal/en/publications/the-public-life-of-the-swahili-stonehouse-14th-15th-centuries-ad/) |
| **Porčić. 2010. “House Floor Area as a Correlate of Marital Residence Pattern: A Logistic Regression Approach.”** | Finds that area alone is an unreliable basis for reconstructing residence rules. [Sage Journals](https://journals.sagepub.com/doi/10.1177/1069397110378839) |

### 6.2 Datasets and archives for calibration

| Resource | What to extract | Main limitation |
| --- | --- | --- |
| **UN Household Size and Composition Database 2022** | Household-size distributions and composition by country and observation year. It assembles **1,059 sources for 196 countries**, with observations spanning **1960–2021**. [United Nations](https://www.un.org/development/desa/pd/node/3587) | The release year is not the observation year; definitions and dates differ. |
| **English Housing Survey** | Usable floor area, dwelling type, tenure, household composition, and outdoor space. [GOV.UK](https://www.gov.uk/government/statistics/chapters-for-english-housing-survey-2023-to-2024-headline-findings-on-demographics-and-household-resilience/chapter-1-profile-of-households-and-dwellings) | Household and dwelling-stock samples must be distinguished. |
| **Hong Kong census housing statistics** | Dense modern housing and per-capita space benchmarks. [DevB](https://www.devb.gov.hk/en/sdev/press/index_id_11447.html) | Preserve the measure’s definition and whether it is a median or mean. |
| **IPUMS NHGIS** | Historical U.S. aggregate housing/population tables and geographic data. [NHGIS](https://www.nhgis.org/) | Aggregate geography is not a house-level architectural dataset. |
| **Sannai Maruyama archaeological archive** | Feature records, plans, and construction variation; the archive supports structured searching and CSV output. [特別史跡「三内丸山遺跡」](https://sannaimaruyama.pref.aomori.jp/sanmaru_search/en/cat_buildings/) | Phase, function, preservation, and reconstruction uncertainty are essential. |
| **Çatalhöyük project database** | Context-level archaeological records for building histories and spatial interpretation. [Çatalhöyük Research Project](https://catalhoyuk.com/tr/content/veri-tabani) | Requires substantial archaeological interpretation before becoming a simulation parameter table. |

### 6.3 Claims that need explicit uncertainty

**Occupancy is usually weaker evidence than geometry.** Room counts, hearths, and footprint area are imperfect indicators of domestic groups. Unknown upper floors, outdoor activities, storage, and noncontemporary occupation can produce large errors. Keep alternative occupancy scenarios rather than a single “archaeologically proven” capacity.

**Room labels can be hypotheses.** At Sannai Maruyama, even roof coverings are reconstructed in several alternatives. A plausible interpretation should therefore be stored as one candidate configuration, not silently promoted to a universal building rule. [特別史跡「三内丸山遺跡」](https://sannaimaruyama.pref.aomori.jp/english/about/restored-pit/)

**Survival and excavation are selective.** Monumental, stone-built, and unusual structures cannot automatically supply ordinary-house distributions. The Asante shrine example is a particularly clear category mismatch to avoid. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/35)

**Area is not a reliable social classifier by itself.** It cannot securely identify household composition, descent rules, or rank without additional evidence. Porčić’s residence-pattern analysis explicitly cautions against drawing strong social conclusions from floor area alone. [Sage Journals](https://journals.sagepub.com/doi/10.1177/1069397110378839)

**The thinnest evidence here is globally comparable ancient household occupancy, ordinary historical housing-size distributions across much of Africa, and universal material-lifetime or thermal-performance coefficients.** The report provides documented cases, not comprehensive population distributions for every region and era. Those gaps should remain visible in TCE’s calibration records.

### Recommended v1 scope

Implement **ground, pit, and pile support systems; circular and rectilinear structural modules; repeated bays; shared walls; courtyard links; and stairs**. Couple those components to **household membership, activity schedules, access rights, construction capabilities, and incremental modification**.

That foundation can generate all eight requested typologies—and historically plausible hybrids—without scripting an architectural sequence. **The crucial historical behavior is not that a house becomes larger when its occupants become richer; it is that people renegotiate, reuse, divide, extend, and inhabit space differently as their circumstances and institutions change.**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928f4-1d3c-83ea-a1a4-e5a949c49551)
