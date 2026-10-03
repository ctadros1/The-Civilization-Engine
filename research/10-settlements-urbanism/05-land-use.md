# Land use patterns and zoning history: a simulation-ready report for TCE

## Main recommendation

**Treat land use as the outcome of activities competing for accessible space under particular property rights—not as a map of predetermined residential, commercial, and industrial districts.** Keep four things separate: what physically occupies a site, what people actually do there, who has rights over it, and what institutions permit.

Historical settlements combined household production, specialized commercial locations, cultivated land, religious and administrative centers, and restrictions on particular activities. Their arrangements differed substantially: Roman fulleries could occupy advantageous urban properties despite their unpleasant reputation; Tang authorities organized enclosed wards and official markets; agricultural production remained interspersed with settlement in some Maya and Southeast Asian urban landscapes. No single concentric or segregated template captures these cases. [OUP Academic](https://academic.oup.com/book/6433/chapter/150252346)

For TCE, the useful distinction is between **economic sorting, negotiated neighbor relations, and compulsory spatial regulation**. They can produce superficially similar quarters through very different processes.

---

## 1. Mechanisms: implementable causal rules

### 1.1 Location value comes from access and productivity, but access to land is institutional

Von Thünen’s essential contribution is not a particular sequence of colored rings. It is the proposition that the surplus available to pay for land depends on production returns minus the costs of reaching consumers. With fixed yields and linear transport costs, agricultural bid rent declines linearly with distance; allowing production intensity and input costs to vary changes that relationship. Visser’s formal treatment shows why even the intensity gradient is conditional rather than universal. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/pdf/10.1111/j.1538-4632.1982.tb00065.x)

A useful simplified implementation is:

\[
R\_k(d)=p\_k y\_k-c\_k-\tau\_k y\_k d
\]

Here, \(R\_k\) is the annual residual available for land use \(k\), in money—or a common accounting equivalent—per hectare; \(p\_k\) is output price per unit; \(y\_k\) is output per hectare per year; \(c\_k\) is non-land production cost per hectare per year; and \(\tau\_k\) is transport cost per unit of output per kilometer.

**For TCE, replace straight-line distance with actual delivery cost.** Incorporate route length, load, mode, gradients, water crossings, tolls, waiting, spoilage, and return journeys. Soil, water availability, and market accessibility should enter separately rather than being collapsed into “fertility.” Agricultural location theory explicitly needs both productive differences and spatial access. [OECD](https://www.oecd.org/en/publications/farmland-conversion_ae50672e-en/full-report/component-4.html)

The resulting model should have the following conditional behavior:

* Perishable output, frequent tending, or high transported mass per hectare makes accessible land more valuable.
* Cheap river transport creates elongated supply regions rather than circular rings.
* Multiple markets produce overlapping catchments.
* Soil, irrigation, fuel availability, or protected land can interrupt an otherwise smooth gradient.

These are **model implications**, not instructions to place vegetables, woodland, grain, and pasture at fixed radii.

Crucially, **the highest economic bidder need not obtain the parcel**. A household may hold inherited cultivation rights; a lineage may allocate plots; an institution may grant occupancy; an owner may refuse subdivision. Store the economic opportunity cost even where no rent is paid, but let the tenure system determine which reallocations are possible. Comparative archaeological work on neighborhoods supports distinguishing household-scale interaction from larger administrative organization rather than assuming a uniform private-property market. [Arizona State University](https://asu.elsevierpure.com/en/publications/the-archaeological-study-of-neighborhoods-and-districts-in-ancien/)

**Synthetic unit test—not historical data:** let two activities have residuals \(100-20d\) and \(60-5d\), in arbitrary units per hectare-year. Their boundary is \(d=2.67\) km. A correct simplified solver reproduces that result. Adding a river, a second market, or unequal soils should break the circular boundary.

### 1.2 Markets, temples, gates, and waterfronts generate different kinds of centrality

A commercial location needs **customers or goods moving through it**, not merely proximity to the settlement’s geometric center. Hillier and colleagues’ pedestrian research links street configuration, movement, and land-use attraction; it offers a mechanism for feedback between accessible streets and commerce, not a universal historical coefficient. [UCL Discovery](https://discovery.ucl.ac.uk/1398/)

**Implement commercial location choice using three distinct advantages:**

**Customer access:** households can reach the seller, and passing travelers create additional demand.

**Freight access:** suppliers can unload goods cheaply. A landing place can therefore attract storage and wholesale exchange without becoming the principal shopping street.

**Institutional access:** an authority designates a market, protects transactions, collects dues, or channels people through a particular location.

For gates, bridges, and landings, calculate actual traffic. A gate can concentrate potential customers, but closure hours, tolls, congestion, or insecurity can offset that advantage. Allow commerce on either side of the boundary, according to the resulting costs and permissions.

Religious centers should create centrality through **specific activities**: gatherings, ceremonies, provisioning, employment, and institutional expenditure. At Vijayanagara, temple-centered townships combined bazaars, residences, and water infrastructure; processional streets also supported festival activity. That is a better model for TCE than a temple emitting an unconditional commercial bonus. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/241/)

Represent a market as an institution and an operating schedule, not necessarily a permanent building. A square may host trade periodically while adjacent shops operate daily.

### 1.3 Mixed use operates within buildings and across time

**Do not assign each building exactly one economic function.** A household may sleep, store grain, prepare food, manufacture goods, and sell products on the same property. Conversely, one enterprise may occupy several disconnected spaces.

Çatalhöyük provides a particularly important early-farming counterexample to modern spatial assumptions: excavated houses combined domestic, storage, and ritual functions, with roof access in closely packed building groups rather than conventional intervening streets. A universal rule requiring every dwelling to front a road would exclude this documented arrangement. [Çatalhöyük Research Project](https://www.catalhoyuk.com/site/architecture)

For TCE, model an activity bundle:

\[
\text{occupancy} =
(\text{actor},\ \text{space},\ \text{activity},\ \text{capacity},\ \text{schedule})
\]

A room can change activity over the day; a courtyard can serve several compatible activities; dangerous or equipment-intensive production can reserve space continuously.

Calculate benefits and conflicts explicitly. Combining home and workshop saves travel and rent but may increase crowding, fire exposure, noise, or storage competition. Mixed use should therefore be **available by default, not mandatory everywhere**.

### 1.4 Craft quarters can emerge without compulsory zoning

Three mechanisms should be authored separately:

| Mechanism | Implementable cause | Distinguishing evidence in the simulation |
| --- | --- | --- |
| Resource clustering | Workshops seek water, fuel, particular raw materials, or shared equipment. | Clustering weakens when resource access changes. |
| Commercial and social clustering | Nearby suppliers, customers, skilled workers, apprentices, and trusted associates improve operations. | Firms remain clustered even without a legal district. |
| Compulsory or privileged location | An authority assigns sites, limits licenses, grants a monopoly, or excludes a trade elsewhere. | A legal boundary produces an additional discontinuity. |

These are proposed mechanisms, not interchangeable labels for every historical occupational quarter. Smith’s comparative framework is useful precisely because neighborhood organization can involve both local interaction and higher-level administration. [Arizona State University](https://asu.elsevierpure.com/en/publications/the-archaeological-study-of-neighborhoods-and-districts-in-ancien/)

Make clustering benefits saturate. Otherwise every workshop of a trade will converge on one parcel. Competition for customers, land, water, and fuel should eventually offset proximity benefits.

### 1.5 Noxious trades are pushed outward only when other forces outweigh their advantages

**“Dirty industry goes outside the walls” is not a reliable universal rule.** Flohr’s research on Roman fulling finds that practical advantages and the availability of suitable property mattered substantially; an unpleasant reputation did not automatically determine peripheral location. [Mikoflohr](https://www.mikoflohr.org/blog/2013/08/13/the-world-of-the-fullo-1/)

The contrasting case is colonial Lima. De Peralta documents municipal efforts to concentrate unpleasant activities and institutions in San Lázaro across the Rímac, alongside the displacement of Indigenous residents. Protection from bad air was entangled with property, status, and colonial power—not simply an impartial efficiency calculation. [Academia](https://www.academia.edu/38313028/Mal_Olor_and_Colonial_Latin_American_History_Smellscapes_in_Lima_Peru_1535_1614)

**TCE implementation:** separate emissions into channels such as smoke, smell, noise, contaminated discharge, and ignition risk. Each channel has its own spatial propagation and consequences.

A workshop’s location decision can then compare:

\[
\Pi\_{ep} =
\text{sales}
-\text{inputs}
-\text{labor}
-\text{premises cost}
-\text{transport}
-\mathbb{E}[\text{legal losses}]
\]

Neighbors evaluate nuisance exposure separately. Their complaints affect expected legal losses only through institutions with jurisdiction and enforcement capacity.

Keep **perceived offensiveness, believed health danger, and actual physiological harm** distinct. Lima’s historical smell politics makes that separation especially important: contemporary explanations of danger were not equivalent to modern exposure assessment. [Duke University Press](https://read.dukeupress.edu/hahr/article/99/1/1/137471/Mal-Olor-and-Colonial-Latin-American-History)

A dirty activity may consequently remain central because it owns its premises, needs customer access, enjoys patronage, or faces weak enforcement. Another may relocate because politically influential neighbors can obtain an effective order.

### 1.6 Nuisance regulation is usually relational, not just territorial

Neighbor-based regulation can constrain drainage, shared walls, openings, access, waste, and obstruction without defining an entire district’s permitted uses. The London Assize of Nuisance records such disputes and remedies. Some early-fourteenth-century orders required correction within **40 days**; that is a formal deadline, not proof of actual compliance within that period. [CeSMA Birmingham](https://cesmabirmingham.wordpress.com/wp-content/uploads/2017/10/chew-and-kellaway-london-assize-of-nuisance.pdf)

Hakim’s account of Islamic building rules emphasizes harm, custom, and precedence. It also describes limited use of the *fina*, the space adjoining a building’s frontage, subject to obligations toward movement and neighbors. Such rules can generate incremental form without a comprehensive land-use plan. Their applicability nevertheless varies by place and legal tradition. [Academia](https://www.academia.edu/54837700/The_Generative_Nature_of_Islamic_Rules_for_the_Built_Environment)

Represent a regulation as:

> **Jurisdiction + protected interest + prohibited action or required condition + who may complain + remedy + enforcement procedure.**

Possible remedies include changing drainage, reducing operating hours, removing an obstruction, compensating a neighbor, improving containment, or relocating an activity. **Relocation should not be the only available response.**

Precedence also matters: an established use may possess stronger rights than a newly arrived complainant. That creates historically plausible persistence without giving every old building permanent immunity.

### 1.7 Ethnic quarters require separate mechanisms for support, exclusion, and coercion

Do not model residential segregation entirely as a preference for similar neighbors. Schelling’s model demonstrates that local preferences can generate aggregate segregation, but it does not establish that this explains a particular historical quarter. [Berkeley Statistics](https://www.stat.berkeley.edu/~aldous/157/Papers/Schelling_Seg_Models.pdf)

For TCE, distinguish:

**Network assistance:** relatives or associates provide lodging, credit, information, and access to employment.

**Institutional exclusion:** some actors cannot acquire particular rights, receive licenses, or enter an area on the same terms as others.

**Compulsory concentration or removal:** authorities assign residence, restrict movement, or displace a population.

**Economic sorting:** unequal resources constrain available premises independently of stated preferences.

Archaeological research on colonial multiethnic settlements shows why neighborhood labels should not imply internally homogeneous populations or permanently fixed identities. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4522790/)

Allow overlapping affiliations—kinship, language, religion, occupation, legal status—rather than a single immutable “ethnicity” field controlling all choices. Institutional discrimination should operate through rights, coercion, costs, and opportunities, **not intrinsic productivity differences between identity groups**.

### 1.8 Regulation and redevelopment evolve through conflict, not an era ladder

TCE should allow several regulatory forms to coexist: customary neighbor obligations, licensed activities, prescribed quarters, protected institutional precincts, street regulations, and later district-wide controls.

China provides a useful transformation rather than a timeless template. Lin describes the contrast between Tang enclosed-ward arrangements and Song commercial development; in Kaifeng, street commerce and encroachment could be accommodated through charges rather than simply eliminated. [Urbana](https://urbanauapp.org/index.php/urbana/issue/download/16/54)

**Proposed institutional rule:** a coalition seeks regulation when its expected gains exceed its political and administrative costs. Whether it succeeds depends on existing constitutional powers, competing claims, revenue needs, and enforcement resources.

Maintain separate dates for a rule’s enactment, public awareness, inspection, judgment, and execution. Changing a law changes incentives and exposure to sanctions; it does not instantaneously replace the urban fabric.

---

## 2. Parameters: documented anchors versus simulation priors

### 2.1 Historical and legal anchors

**Confidence refers to the specified case.** A clearly documented ordinance can have high evidential confidence while having very low applicability as a universal default.

| Parameter or observation | Value and units | Context and appropriate use | Source and confidence |
| --- | --- | --- | --- |
| Official ward and market organization | **108 residential wards; 2 official markets** | Tang Chang’an. Benchmark a particular administrative design, not the complete distribution of actual trade. | Lin, 2015. **High for reported arrangement; low transferability.** [Urbana](https://urbanauapp.org/index.php/urbana/issue/download/16/54) |
| Dimensions of a selected ward subset | **500–590 × 580–700 m** | Wards flanking a principal axis in Chang’an; not the dimensions of every ward. | Lin, 2015. **Medium–high reconstruction confidence.** [Urbana](https://urbanauapp.org/index.php/urbana/issue/download/16/54) |
| Length of a major commercial street | **2.5 km** | Panlou Street, Kaifeng. Demonstrates extended commercial frontage rather than commerce confined to one square. | Lin, 2015. **Medium; site-specific.** [Urbana](https://urbanauapp.org/index.php/urbana/issue/download/16/54) |
| Nuisance abatement deadline | **40 days** | Selected early-fourteenth-century London orders. Use as a legal deadline, not an observed enforcement delay. | Chew and Kellaway’s calendar. **High for the recorded prescription.** [CeSMA Birmingham](https://cesmabirmingham.wordpress.com/wp-content/uploads/2017/10/chew-and-kellaway-london-assize-of-nuisance.pdf) |
| Frontage-use strip, *fina* | Approximately **1–1.5 m** | Limited frontage use under the tradition described by Hakim, subject to access and neighbor obligations. Not a universal sidewalk standard. | Hakim, 2010. **Medium; contextual norm.** [Academia](https://www.academia.edu/54837700/The_Generative_Nature_of_Islamic_Rules_for_the_Built_Environment) |
| Extensive urban-settlement landscape | **More than 1,000 km²** | Greater Angkor’s integrated low-density settlement and water-management landscape. **Not continuously built-up area.** | Evans et al., 2007. **High for broad mapped extent; boundaries remain interpretive.** [PubMed](https://pubmed.ncbi.nlm.nih.gov/17717084/) |
| Height-district limits | **35, 50, 80 ft**, approximately **10.7, 15.2, 24.4 m** | Euclid’s 1922 ordinance, described in the 1926 litigation. Example of independently parameterized dimensional districts. | Original Supreme Court opinion. **High for stated law.** [Legal Information Institute](https://www.law.cornell.edu/supremecourt/text/272/365) |
| Tower-area exception | Upper portion covering **no more than 25% of lot area** | New York’s 1916 rule permitted a height exception subject to additional conditions. This was **not** a universal 25% building-coverage cap. | Original resolution, §9(d). **High for stated law.** [NYC.gov](https://www.nyc.gov/assets/planning/download/pdf/about/city-planning-history/1916_zoning_resolution.pdf) |
| Small-shop allowance in a residential category | Certain shops up to **150 m² floor area** | Category II low-rise residential zone in the cited Japanese MLIT brochure. A historical comparative example, not a statement of every current provision. | MLIT. **High for the brochure’s stated rule.** [Ministry of Transport and Tourism](https://www.mlit.go.jp/common/001050453.pdf) |

These sources do **not** establish a defensible worldwide tannery setback, standard agricultural-ring radius, typical percentage of mixed-use buildings, or uniform preindustrial rent gradient. Do not manufacture those constants from isolated examples.

### 2.2 Proposed TCE starting values

The following are **engineering and behavioral priors**, not measured historical distributions. Their source is the proposed model design; confidence in their historical calibration is **low/unvalidated**.

| Proposed parameter | Initial range | Units and interpretation | Calibration requirement |
| --- | --- | --- | --- |
| Nominal unloaded walking speed | **3–5** | km/h, before actor, surface, load, and weather modifiers | Check against the movement model; do not use for carts or loaded freight. |
| Routine field-travel allowance | **0.5–1.0** | Hours one way on a working day | Treat as a soft household constraint, not a hard cultivation boundary. |
| Ordinary recurring market interval | **1–10** | Days between market sessions | Author calendars separately from seasonal fairs and special gatherings. |
| Premises review interval | **30–90** | Days between routine location reviews | Household division, eviction, vacancy, and business failure should trigger additional reviews. |
| Investment evaluation horizon | **5–20** | Years of expected occupancy or benefit | Vary by tenure security and actor expectations; do not force identical discounting. |
| Effective enforcement scenarios | **0.1–0.9** | Probability that an actionable, recorded violation is remedied within a year | A sensitivity range for institutional outcomes, not a historical estimate. |
| Candidate premises considered | **16–64** | Alternatives per search episode | Computational sampling parameter; test convergence against larger samples. |
| Coarse nuisance-field resolution | **10–25** | Meters per cell where a raster is used | Numerical resolution only—not a health threshold or legal buffer. |

The first two priors imply an illustrative one-way reach of **1.5–5 km along the traversable network**. That is a derived scenario range, not an archaeological finding. Seasonal labor bottlenecks, overnight stays, canals, and delegated workers should allow different arrangements.

Agricultural yields, freight capacity, spoilage, water consumption, and fuel demand should come from TCE’s production and transport modules. **Fit the land-use pattern from those inputs; do not fit an arbitrary ring radius and then force the economy to match it.**

---

## 3. Variation across eras and world regions

The appropriate historical comparison is not a progression from “unregulated mixture” to “rational separation.” The cases below require different combinations of the same underlying mechanisms.

| Setting | Documented pattern and implication for TCE |
| --- | --- |
| **Foragers and pre-agrarian activity areas** | Archaeological fire and activity-area research identifies spatial organization around repeated tasks and hearths. Represent activity locations, access, refuse, and revisitation without presuming urban parcels or rent-paying land markets. These are not miniature modern zoning systems. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/692721) |
| **Early farming settlements** | Çatalhöyük demonstrates closely packed, roof-accessed houses combining everyday and ritual activities. Support non-street access and multifunctional domestic space; do not require a separate civic or religious building for every ritual activity. [Çatalhöyük Research Project](https://www.catalhoyuk.com/site/architecture) |
| **Roman Mediterranean cities** | Fulling premises show that specialized production could occupy existing urban properties rather than a universally segregated industrial fringe. Building suitability and conversion opportunities belong in location choice. [OUP Academic](https://academic.oup.com/book/6433/chapter/150252346) |
| **Medieval European towns** | London’s nuisance proceedings reveal regulation through particular rights, injuries, and remedies. A mixed-use street can be heavily regulated without belonging to a modern use district. [CeSMA Birmingham](https://cesmabirmingham.wordpress.com/wp-content/uploads/2017/10/chew-and-kellaway-london-assize-of-nuisance.pdf) |
| **Tang–Song China** | Enclosed wards and regulated markets contrast with later expansion of street commerce. Model gate operation, licensed locations, enforcement, and negotiated encroachment as changeable institutions—not permanent characteristics of “Chinese cities.” [Cambridge University Press](https://www.cambridge.org/core/books/an-urban-history-of-china/tangsong-transition-and-its-effects-on-chinas-imperial-urban-civilization-9071402/582195CF8CDB0BB3C4B5BC55A0811FEA) |
| **Islamic Mediterranean traditions** | Rules concerning harm, precedence, privacy, and customary frontage use can produce incremental urban form. They do not imply either an absence of regulation or one universally mandated city plan. [Academia](https://www.academia.edu/54837700/The_Generative_Nature_of_Islamic_Rules_for_the_Built_Environment) |
| **Northern Nigeria** | Hakim and Ahmed’s study of nineteenth-century Sokoto Caliphate rules and Zaria documents explicit legal guidance alongside incremental settlement change. African urbanism should not be represented as normatively unregulated by default. [ResearchGate](https://www.researchgate.net/publication/279891912_Rules_for_the_built_environment_in_19th_century_Northern_Nigeria) |
| **South Asia** | Vijayanagara’s temple-centered townships linked bazaars, residences, ceremonial routes, and water infrastructure. Commercial centrality can follow religious and political nodes rather than a single secular central business district. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/241/) |
| **Southeast Asia** | Angkor’s settlement and hydraulic landscape demonstrates that extensive, relatively low-density urban organization need not have a sharp boundary between “city” and agricultural hinterland. [PubMed](https://pubmed.ncbi.nlm.nih.gov/17717084/) |
| **Mesoamerica** | Maya and Aztec cases include productive urban land and settlement–agriculture mosaics. Cultivation inside an urban system should not automatically count as abandonment or failed densification. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0264275112001382) |
| **Colonial Latin America** | Lima shows environmental regulation operating through colonial hierarchy and displacement. Distributional consequences depend on whose property and comfort receive protection. [Academia](https://www.academia.edu/38313028/Mal_Olor_and_Colonial_Latin_American_History_Smellscapes_in_Lima_Peru_1535_1614) |
| **Industrial and modern cities** | Modern use and dimensional regulation developed through differing national traditions. Hirt’s comparison emphasizes the distinctive American commitment to purely residential and single-family zones; Japan’s categories demonstrate that formal zoning can still authorize substantial mixtures of uses. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0739456X13494242) |

These cases should be available as **institutional configurations and ecological conditions**, not ethnic architecture presets that automatically determine behavior.

---

## 4. Stylized facts and validation targets

A correct simulation should reproduce **conditional patterns and counterexamples**, rather than force every settlement toward one diagram.

### Accessible routes attract commerce—but the relationship is mediated

Measure commercial floor area, entrances, customer trips, freight deliveries, and frontage rents against network accessibility. In otherwise comparable locations, access should matter; permissions, institutional demand, and congestion should explain important exceptions. Hillier et al. provide an empirical framework for studying movement–configuration relationships, although their observations are not a direct calibration of ancient cities. [UCL Discovery](https://discovery.ucl.ac.uk/1398/)

**Test:** reroute a major connection. Some firms should relocate or lose customers, while sunk investments and other advantages keep others in place.

### Mixed use survives at finer spatial scales

Measure mixed activity separately at room, property, street, and district scales. A district containing both houses and workshops is not necessarily the same as households working within their dwellings. Çatalhöyük and Roman production premises illustrate why this distinction matters. [Çatalhöyük Research Project](https://www.catalhoyuk.com/site/architecture)

**Test:** disabling compulsory separation must permit home-based production when technically and economically feasible. It must not force every household to manufacture.

### Occupational clustering does not prove legal segregation

Track whether a craft cluster is explained by shared resources, business networks, or a jurisdictional rule. An occupational concentration index above the citywide share indicates clustering, not its cause.

**Test:** remove the legal restriction while retaining resources and networks. Some clusters should persist. Remove the resource advantage instead, and a different set should weaken.

### Nuisance location is a distribution, not an edge rule

The model must be able to reproduce both centrally located unpleasant production and politically enforced displacement. Rome and Lima provide contrasting cases. [OUP Academic](https://academic.oup.com/book/6433/chapter/150252346)

**Test:** increasing enforcement for privileged complainants should redistribute exposure, not necessarily reduce total emissions. Effective technical mitigation should sometimes preserve the original location.

### Urban systems can contain substantial productive and open land

Angkor and agrarian Mesoamerican urbanism rule out treating every unbuilt space inside a settlement boundary as awaiting construction. [PubMed](https://pubmed.ncbi.nlm.nih.gov/17717084/)

**Test:** valuable gardens, reservoirs, cultivated plots, and institutional open spaces can remain in use despite nearby building demand. Gross settlement area and built-up area must be reported separately.

### Formal rules and realized outcomes diverge

A recorded **40-day** deadline is not an instantaneous simulation command. London’s records provide prescriptions to test, not a universal compliance distribution. [CeSMA Birmingham](https://cesmabirmingham.wordpress.com/wp-content/uploads/2017/10/chew-and-kellaway-london-assize-of-nuisance.pdf)

**Test:** distinguish violations, complaints, judgments, overdue orders, negotiated remedies, and completed compliance. Institutional incapacity should create observable backlogs rather than silently disabling the law.

### Rings appear only in the controlled case

**Test:** the homogeneous, single-market, fixed-price experiment should approach the bid-rent solution. Introducing heterogeneous soils, water transport, competing markets, or nontransferable rights should produce deviations. This validates the mechanism without requiring real cities to resemble the idealization. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/pdf/10.1111/j.1538-4632.1982.tb00065.x)

---

## 5. Recommended representation for TCE

### 5.1 Minimum spatial and social model

**Use persistent sites, flexible occupancy, and explicit rights.**

A **site or parcel** stores geometry, productive attributes, access points, and tenure claims. It need not always be a surveyed private lot.

A **structure** stores physical capacity, construction, entrances, and subdividable spaces. The same structure can accommodate several households or activities.

An **occupancy record** connects an actor to an activity, space requirement, schedule, and permission status. Land-use labels displayed in the interface should be summaries of these records.

A **household or enterprise** chooses premises and allocates labor. Individuals perform the resulting journeys and tasks. This avoids making every household member independently relocate the family workshop.

An **institution** owns or administers rights, issues rules, hears disputes, and deploys enforcement resources. A temple, lineage, municipality, or ruler can use the same underlying allocation interfaces with different decision procedures.

Access must be a graph of usable connections, including doors, alleys, shared courts, stairs, roofs where appropriate, paths, gates, and waterfront connections—not merely adjacency to a road segment.

### 5.2 Separate fast behavior from slow spatial change

For v1, I recommend:

**Daily or event-driven:** movement, production, selling, loading, waste generation, exposure, market opening, and gate operation.

**Monthly or seasonal:** premises searches, leases, land allocation, crop choices, complaints, licensing, and compliance reviews.

**Project-driven:** construction, subdivision, major conversion, relocation, infrastructure extension, and institutional redesign.

These are scheduling recommendations, not measured historical frequencies.

Calculate accessibility for relevant destinations and refresh it when networks, gate rules, markets, or important facilities change. Sample a limited set of feasible premises instead of comparing every actor with every parcel every tick. Evaluate the numerical consequences of this sampling before choosing a final limit.

For nuisance, use inexpensive local kernels or coarse fields where appropriate, with directional propagation for water and wind. Reserve detailed geometry for interactions that need it, such as shared walls, doorway obstruction, and fire adjacency. **A single circular “pollution radius” should not stand in for every kind of harm.**

### 5.3 Make rights and enforcement inspectable

A useful rule schema is:

```
Rule
  jurisdiction
  target actors / activities / structures
  obligation or restriction
  exemptions and pre-existing rights
  evidence and complaint requirements
  responsible institution
  permitted remedies
  appeal procedure
```

For example, “do not discharge waste into a neighbor’s drain” is an action-based obligation; “only licensed slaughtering at designated sites” combines an activity license with a spatial condition; “this group may not acquire occupancy here” is discriminatory eligibility regulation.

Do not turn every restriction into a hard placement veto. Physical impossibility can be a hard constraint; a legal prohibition may instead create concealed use, informal payment, litigation, or eviction risk. Which response is possible depends on the authored institution.

### 5.4 Preserve scale consistency

A simulation of 10,000–50,000 people need not recreate the absolute population or footprint of every historical metropolis. Use smaller settlements, representative subsystems, or explicitly scaled comparisons.

**Do not shrink a thousand-square-kilometer settlement landscape into a tiny map while keeping ordinary walking speeds and expecting the same land-use economics.** Distances, trip durations, freight costs, and the number of consumers must remain mutually consistent.

### 5.5 Existing models and games worth borrowing from

| Model or game | Useful component | What not to assume |
| --- | --- | --- |
| **UrbanSim; Waddell and collaborators** | Disaggregated land-use, household, employment, development, and transport modeling; modular architecture. | Modern housing and property-market assumptions are not a universal representation of historical tenure. [UrbanSim](https://www.urbansim.com/academic-research) |
| **Schelling segregation models** | A clear demonstration of feedback between local choices and aggregate spatial outcomes. | They do not establish that historical segregation was voluntary or preference-driven. [Berkeley Statistics](https://www.stat.berkeley.edu/~aldous/157/Papers/Schelling_Seg_Models.pdf) |
| **ORBIS** | Network-based travel time and cost across land, river, and sea for the Roman world. | Its reconstructed network and transport assumptions are context-specific, not universal ancient coefficients. [ASIS&T](https://asistdl.onlinelibrary.wiley.com/doi/10.1002/bult.2015.1720410206) |
| **Manor Lords** | Gridless settlement placement and an explicit design interest in terrain and routes shaping settlement form. | Its developer description is a design reference, not empirical validation of autonomous land institutions. [Manor Lords](https://manorlords.com/) |
| **Foundation** | Procedural, gridless development, modular construction, and areas guiding residential growth. | Its player-painted development areas are not equivalent to endogenous legislation or negotiated property rights. [Steam Store](https://store.steampowered.com/app/690830/Foundation/) |

### 5.6 Modern zoning: add post-v1 without replacing the core model

Modern zoning should add **map-based permission and dimensional constraints** to the existing activity-and-rights system.

New York’s 1916 resolution combined use, height, and area controls. Euclid’s ordinance used separate use, height, and area classes; importantly, its use scheme was cumulative in significant respects rather than simply three mutually exclusive residential, commercial, and industrial colors. The 1926 Supreme Court decision upheld the ordinance against the broad challenge before it, not every conceivable zoning provision. [NYC.gov](https://www.nyc.gov/assets/planning/download/pdf/about/city-planning-history/1916_zoning_resolution.pdf)

Add authorable controls for permitted activities, intensity, height, setbacks, and building coverage. Keep two quantities distinct:

\[
\mathrm{FAR} =
\frac{\text{total floor area}}{\text{site area}},
\qquad
\mathrm{coverage} =
\frac{\text{building footprint}}{\text{site area}}.
\]

Also represent pre-existing nonconforming uses, exceptions, appeals, permits, and enforcement. A new district changes legal development capacity; it does not automatically create demand or demolish existing buildings.

Do not make exclusive single-family zoning the inevitable endpoint. Hirt’s comparative work and the Japanese land-use categories show why modern formal regulation can authorize very different degrees of mixture. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0739456X13494242)

---

## 6. Sources, datasets, and unresolved evidence

### 6.1 Datasets suitable for calibration

| Dataset or source collection | Appropriate use | Important limitation |
| --- | --- | --- |
| **Pompeii Bibliography and Mapping Project / associated mapping resources** | Buildings, streets, spatial references, and links to archaeological interpretation; useful for workshop access and property-level comparisons. | Functional identifications are interpretations. A preserved urban snapshot is not a longitudinal record of rent, occupancy, or regulation. [UMass Websites](https://websites.umass.edu/pbmp/about/) |
| **Hillier et al. pedestrian observations, UCL repository** | Testing relationships among street configuration, movement, and attraction. | Modern observations; transfer the testing method rather than assuming identical ancient coefficients. [UCL Discovery](https://discovery.ucl.ac.uk/1398/) |
| **Evans et al.’s Angkor mapping** | Extensive settlement structure, hydraulic connections, and distinctions between dense construction and broader urban landscape. | Mapped extent is not continuous occupancy, and all features need not belong to one simultaneous phase. [PubMed](https://pubmed.ncbi.nlm.nih.gov/17717084/) |
| **HYDE 3.2 historical land-use reconstruction** | Regional agricultural land budgets and broad settlement–hinterland consistency. | Approximately **5-arc-minute** spatial resolution; unsuitable for parcel-level zoning, and early estimates are reconstructed rather than cadastral observations. [ESSD](https://essd.copernicus.org/articles/9/927/2017/essd-9-927-2017.html) |
| **NYC PLUTO / MapPLUTO documentation** | A modern example of separating actual building characteristics, mixed-use categories, and zoning information. | Modern administrative definitions and data quality; not training data for ancient behavior. The cited dictionary is version 22v2. [NYC.gov](https://www.nyc.gov/assets/planning/download/pdf/data-maps/open-data/PLUTODD.pdf) |
| **London Assize of Nuisance calendar** | Rule types, affected relationships, remedies, and formal deadlines. | Records selected disputes, not every nuisance or every resident’s experience; complaints and surviving judgments are a biased sample. [CeSMA Birmingham](https://cesmabirmingham.wordpress.com/wp-content/uploads/2017/10/chew-and-kellaway-london-assize-of-nuisance.pdf) |

### 6.2 Core scholarly reading priorities

For the economic mechanism, begin with **Visser, “On Agricultural Location Theory” (1982)**. For neighborhood structure, use **Smith, “The Archaeological Study of Neighborhoods and Districts in Ancient Cities” (2010)**. Together they help separate spatial incentives from the institutions and interaction scales through which those incentives operate. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/pdf/10.1111/j.1538-4632.1982.tb00065.x)

For regulation and counterexamples, prioritize **Flohr, *The World of the Fullo* (2013)**; **Chew and Kellaway, *London Assize of Nuisance 1301–1431* (1973)**; and **de Peralta, “Mal Olor and Colonial Latin American History” (2019)**. Their value is that they expose actual premises, disputes, and distributional conflicts rather than only ideal city diagrams. [OUP Academic](https://academic.oup.com/book/6433/chapter/150252346)

For alternatives to compact, functionally separated urbanism, use **Evans et al. (2007)** on Angkor and **Isendahl and Smith, “Sustainable Agrarian Urbanism” (2013)**. For modern comparative regulation, use **Hirt, “Home, Sweet Home” (2013)** alongside original ordinances. [PubMed](https://pubmed.ncbi.nlm.nih.gov/17717084/)

### 6.3 What remains contested or thin

**Ideal plans are not occupation maps.** A prescribed market location, a legal district, and observed activity require different data fields. The Tang–Song material demonstrates why regulatory categories and street-level commerce cannot simply be equated. [Cambridge University Press](https://www.cambridge.org/core/books/an-urban-history-of-china/tangsong-transition-and-its-effects-on-chinas-imperial-urban-civilization-9071402/582195CF8CDB0BB3C4B5BC55A0811FEA)

**Civilizational city types can conceal weak generalization.** Abu-Lughod’s critique of the “Islamic city” model is especially important: a narrow selection of examples and repeated assumptions can harden into an apparently universal explanation. Treat the relevant legal and social mechanisms as hypotheses to test locally. [Cambridge University Press](https://www.cambridge.org/core/journals/international-journal-of-middle-east-studies/article/islamic-city-historic-myth-islamic-essence-and-contemporary-relevance/E686C6BE7F410B6C1717C992116B8195)

**Archaeological clustering does not identify its cause by itself.** A concentration of artifacts or building types may support an activity interpretation without establishing the residents’ identities, legal permissions, rents, or reasons for choosing that location. Use multiple evidence types before calibrating a causal coefficient. [Arizona State University](https://asu.elsevierpure.com/en/publications/the-archaeological-study-of-neighborhoods-and-districts-in-ancien/)

**Urban extent and land-use mixture depend on definitions.** Building footprint, occupied property, administrative district, and integrated settlement landscape are different denominators. Angkor’s extent and Pompeii’s property-level evidence should not be compared as though they measure the same thing. [PubMed](https://pubmed.ncbi.nlm.nih.gov/17717084/)

**For v1, implement access, productive opportunity, mixed occupancy, tenure, and nuisance justice.** Those mechanisms can generate commercial streets, working households, craft clusters, cultivated urban plots, and politically displaced hazards. Comprehensive zoning can then emerge as one institutional choice among others, rather than being built into the world before its inhabitants have made any laws.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928f0-bfa8-83ea-bb65-c799ff336b66)
