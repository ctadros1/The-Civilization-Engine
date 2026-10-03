# How architectural styles evolve and spread

## A simulation-ready report for The Civilization Engine

**Recommendation: model architectural change as the selective transmission of building practices, expressed through individual construction projects—not as cultures periodically acquiring new visual skins.**

For TCE, a “style” should be a recognizable distribution of combinations: structural systems, spatial arrangements, roofs, openings, proportions, ornament, materials and symbols. Different people and institutions can prefer different combinations within the same culture. Builders may know techniques that patrons cannot afford; patrons may admire buildings that local workshops cannot reproduce.

The most important distinction is between **three clocks**:

1. **Exposure:** when people encounter a design or learn that it exists.
2. **Implementation:** when someone can finance, authorize and competently construct an adaptation.
3. **Built-stock change:** when enough construction and renovation have occurred to alter the settlement’s appearance.

These clocks can diverge dramatically. A monumental commission can introduce a conspicuous novelty without transforming ordinary housing. Conversely, a settlement can replace buildings frequently while deliberately preserving their form—as the approximately twenty-year rebuilding cycle at Japan’s Shinto shrine of Ise demonstrates. [Ise Jingu](https://www.isejingu.or.jp/en/ritual/index.html)

**Evidence assessment:** historical dates, building phases and identifiable transmission networks provide useful calibration anchors. Evidence is much thinner for universal annual rates of taste change, prestige effects or hybridization. The report therefore separates **historical observations**, **mathematical consequences** and **proposed simulation parameters**.

---

## 1. Mechanisms: implementable rules and historical cases

### 1.1 The causal rules

The following are proposed implementations of mechanisms supported by the cases below, not statistically fitted historical laws.

| Mechanism | Rule TCE could implement | Important qualification |
| --- | --- | --- |
| **Selective exposure** | Add designs to an agent’s known repertoire through observed buildings, travel, migration, trade, pilgrimage, apprenticeship and institutional connections. Later technologies can introduce drawings, catalogues and mass media. | Seeing a feature does not imply knowing how to construct it. |
| **Prestige-biased copying** | Increase the probability of considering an exemplar when its patron, builder or institution is admired by the decision-maker. Make prestige specific to social group and domain. | Prestige is not identical to wealth or coercive power. A ruler can compel construction without becoming an admired model. |
| **Craft transmission** | Transfer construction recipes through supervised work, apprenticeship, specialist migration and collaboration. Require appropriate competence for demanding structural combinations. | A copied silhouette or decorative arch does not confer the ability to build a load-bearing vault. |
| **Patronage and competition** | Wealthy households, merchants, governments and religious institutions fund projects that express their goals and rivalries. Successful projects attract further commissions and trainees. | Different patrons may compete through size, antiquity, austerity, ornament, ritual correctness or foreign associations—not just novelty. |
| **Functional and material selection** | Filter candidate designs through climate, available materials, plot geometry, structural requirements, labor and financing. | Similar constraints can independently produce similar buildings; resemblance is not necessarily evidence of diffusion. |
| **Religious and ideological selection** | Apply requirements to circulation, orientation, gathering spaces, visibility, iconography and permitted uses. Let institutions sponsor, suppress or reinterpret particular forms. | Do not assign each religion a mandatory roof, dome or wall material. |
| **Selective hybridization** | Recombine compatible elements from several known repertoires, retaining features that satisfy local functions, identity and construction capabilities. | Mixing is structured. It need not be a random, equal contribution from two supposedly “pure” parent styles. |
| **Construction opportunities and persistence** | Reconsider designs when a funded project occurs: new construction, extension, damage repair, replacement or conversion. Preserve untouched components and phases. | Changing preferences alone must not regenerate existing buildings. |

Prestige-biased transmission has a theoretical foundation in Henrich and Gil-White’s distinction between freely conferred prestige and dominance. However, their work does **not** provide an architectural copying coefficient that can simply be imported into TCE. [ResearchGate](https://www.researchgate.net/publication/11954021_The_Evolution_of_Prestige_Freely_Conferred_Deference_as_a_Mechanism_for_Enhancing_the_Benefits_of_Cultural_Transmission)

A complementary hypothesis comes from **complex contagion**: adoption of a costly or uncertain practice may require reinforcement from several trusted sources, rather than one distant contact. This is a useful model for expensive building choices, but remains an architectural modeling analogy rather than a measured universal law. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/521848)

### 1.2 Romanesque to Gothic: recombination, patrons and mobile expertise

The east-end rebuilding at Saint-Denis in **1140–1144** is a pivotal Gothic exemplar, not an instantaneous invention of every Gothic component. Its importance lies in combining pointed arches, rib vaults, slender supports and a particular treatment of interior space and light. Craft mobility also matters: the Metropolitan Museum documents sculptural workshops moving from Reims to Bamberg, carrying workmanship and motifs between major commissions. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/gothic-art)

Canterbury illustrates the interaction of **opportunity and finance**. The fire of **1174** created a rebuilding occasion, while the cathedral’s pilgrimage economy helped support ambitious enlargement. The resulting building retained multiple chronological layers rather than becoming uniformly “Gothic” at one moment. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/496)

Cologne illustrates another clock. Construction began in **1248**, but completion came in **1880**, following long interruptions and renewed commitment to the medieval design. The elapsed 632 years are neither a normal construction duration nor a diffusion rate; they demonstrate how institutional memory and revival can reconnect widely separated building campaigns. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/292/)

**TCE rule:** prestigious projects should create unusually visible exemplars and specialist employment. Diffusion follows patron and workshop connections. Damage, new funding and institutional ambition determine when another settlement can act on that influence.

Do not encode “Gothic replaces Romanesque because technology reaches level X.” Encode feasible structural combinations, particular architectural ambitions and the projects through which those combinations become familiar.

### 1.3 Islamic architecture: common religious purposes, multiple regional traditions

Early Islamic architecture did not emerge from an empty design space. Umayyad patrons and craftspeople drew on late-antique, Byzantine and Sasanian traditions while creating buildings for new political and religious purposes. This is a case of institutional change reorganizing an existing repertoire. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-art-of-the-umayyad-period-661-750)

The subsequent record is strongly regional:

**China:** Nancy Shatzman Steinhardt’s research documents mosques that combine Chinese architectural compounds, courtyards and gate forms with Islamic requirements and inscriptions. A building can preserve a locally familiar architectural language while changing its orientation, ritual organization and religious meaning. Her study considered roughly seventy surviving mosques predating the twentieth century. [Penn Today](https://penntoday.upenn.edu/news/east-asian-art-prof-documents-early-chinese-mosques)

**The Swahili coast:** Kilwa’s architecture developed through African urban institutions and Indian Ocean commerce, using materials including coral stone and lime. Its Great Mosque originated in the eleventh century and was enlarged in the thirteenth. Trade connections and locally situated construction traditions belong in the explanation together. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/144/)

**Iberia:** the Mudéjar architecture of Aragon combined Islamic-derived brickwork and glazed decoration with other architectural traditions under Christian rule. Its development from the twelfth into the early seventeenth century makes religious identity an inadequate predictor of the entire architectural repertoire. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/378/)

Flood’s *Objects of Translation* additionally emphasizes movement through trade, gifts, migration, appropriation and conflict in medieval South Asian encounters. Hybridization should therefore not be equated with peaceful or equal cultural exchange. [AAE Portal](https://aaeportal.com/publications/-18546/objects-of-translation-material-culture-and-medieval-hindu-muslim-encounter)

**TCE rule:** transmit religious requirements separately from construction traditions. A newly established religious institution should recruit from the builders and materials actually accessible to it. Conquest can change patronage and ownership, move craftspeople, reuse buildings or disrupt production—but should not automatically replace every local recipe.

### 1.4 Buddhist architecture: institutional networks and transformations of spatial form

Buddhist transmission across Asia carried institutions, ritual practices and sacred concepts, but not one invariant building type. Korean temple architecture, for example, developed several arrangements, including one-pagoda/one-hall, one-pagoda/three-hall and paired-pagoda configurations. Temples could be complexes of **five to sixty buildings**, making the arrangement of an ensemble as important as the appearance of any individual structure. Political changes also altered the resources and locations available to monasteries. [Smithsonian APA Center](https://asia.si.edu/research/essays/buddhist-architecture-in-korea/)

Hōryū-ji provides evidence for building knowledge transmitted through Korea to Japan and locally adapted. UNESCO identifies **48 monuments**, including **11 dating to the late seventh or eighth century**. Their survival demonstrates the importance of protection and maintenance; it is not an estimate of the normal lifespan of an unattended wooden building. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/660/)

At Borobudur, the eighth- to ninth-century ensemble expresses Buddhist concepts through a distinctive monumental organization. UNESCO interprets it as combining Buddhist ideas with local ancestor-related traditions; that interpretation is useful, but should be treated as an interpretation of meaning rather than a precisely measured causal variable. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/592/)

**TCE rule:** religious networks should transmit programs and relationships—relic accommodation, ritual circulation, assembly, residence, pilgrimage—alongside selected motifs. Local workshops then realize those requirements through feasible construction systems. The result can be recognizable religious continuity without visual uniformity.

### 1.5 Colonial architecture: unequal power, local work and new meanings

The four UNESCO-listed Baroque churches of the Philippines illustrate adaptation by Chinese and Filipino craftspeople of European church forms to local materials, decorative repertoires and seismic conditions. Their massing and buttressing were not merely ornamental departures from a European model. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/677/)

The Jesuit missions of Chiquitos in Bolivia combined a missionary settlement program with local construction and craftsmanship, including prominent timber structures. Here the transferred object was partly an **institutional and spatial arrangement**, not simply a façade style. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/529/)

Mumbai’s Chhatrapati Shivaji Terminus, constructed in **1878–1888**, combined Victorian Gothic Revival design with Indian architectural references and craftsmanship. It demonstrates hybridization within a highly organized colonial infrastructure commission. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/945/)

Asmara adds two further mechanisms. Its colonial development combined modernist architecture with segregated planning. Subsequently, the inherited city acquired significance within Eritrean national identity. A building’s social meaning can therefore change without corresponding replacement of its physical form. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1550/)

**TCE rule:** distinguish the commissioner, designer, builders, users and political authority. They may have different preferences and unequal freedom. Store the social interpretation of a building separately from its geometry.

---

## 2. Parameters: rates, regional lag and uncertainty

### 2.1 What can actually be measured?

Do not collapse the following into one “adoption date”:

* first documented awareness or imported object;
* first experimental building;
* first locally reproducible construction;
* majority of new commissions;
* majority of standing buildings or floor area.

For TCE calibration, record the event type and its population of reference. “A cathedral started using a design” and “ordinary houses commonly used it” are fundamentally different observations.

### Historical anchors

**Confidence refers to the stated observation, not to its suitability as a universal parameter.**

| Observation | Quantitative anchor | What it can calibrate | Confidence and source |
| --- | --- | --- | --- |
| Saint-Denis east-end campaign | **1140–1144; approximately 4 years** | A major, well-funded construction phase | **High** for chronology; not the duration of an entire style transition. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/gothic-art) |
| Canterbury rebuilding after fire | **1174**, approximately **30 years after 1144** | A later major commission adopting related architectural possibilities | **High** for the event; the offset is not a national adoption rate. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/496) |
| Cologne cathedral | Begun **1248**, completed **1880**; **632 elapsed years** | Interrupted projects, design persistence and revival | **High**; do not treat elapsed time as continuous construction. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/292/) |
| Tudor Revival in Washington, Missouri | National stylistic period **1890–1940**; surveyed local examples **late 1920s–early 1940s** | Approximately **35–40 years** between the national period’s start and local onset in this inventory | **Moderate** as a regional-lag proxy: surviving local buildings, not an awareness survey. [NPGallery](https://npgallery.nps.gov/pdfhost/docs/NRHP/Text/64500319.pdf) |
| Ise shrine renewal | Approximately **20 years per rebuilding cycle** | Deliberate preservation of form through material replacement | **High** for the institutional practice; Shinto, not Buddhist. [Ise Jingu](https://www.isejingu.or.jp/en/ritual/index.html) |
| Mumbai railway terminus | **1878–1888; 10 years** | A large hybrid architectural commission | **High** for project chronology. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/945/) |
| Tel Aviv’s White City | Principal development from the **early 1930s to the 1950s**, roughly **20–30 years** | Rapid formation of a substantial new architectural ensemble | **High** for the broad period; not a measured individual-adoption curve. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1096/) |

**Interpretation:** these cases support a model in which conspicuous adoption can occur over decades, while construction campaigns and inherited fabric extend across much longer periods. They do **not** justify a single global “architectural diffusion speed,” especially in kilometres per year.

### 2.2 Proposed starting parameters for TCE

The table below contains **engineering test priors, not historical estimates**. Their purpose is to expose behavior during prototyping and sensitivity analysis. The historical evidence above supports including the mechanisms, not these particular numerical settings.

| Parameter | Initial sensitivity range | Unit / application | Source and confidence |
| --- | --- | --- | --- |
| Preference adjustment after a meaningful encounter, \(\alpha\) | **0.02–0.20** | Fractional adjustment per independent, relevant exposure | **TCE proposal; low empirical confidence** |
| Maximum prestige weighting of an exemplar | **1–3×** a neutral exemplar | Relative consideration weight; observer-specific | **TCE proposal; low** |
| Acquisition of a demanding unfamiliar craft recipe | **3–10** | Years of supervised practice; an imported competent team can bypass local training delay | **TCE proposal; low**; not a universal apprenticeship duration |
| Attempt to introduce an unusual, previously unused feasible combination | **0.1–3%** | Probability per commission | **TCE proposal; low**; not a technology-discovery rate |
| Opportunity to reconsider replaceable ornament | **5–20** | Years between opportunities, conditional on need and funding | **TCE proposal; low**; not mandatory renovation |
| Existing-stock replacement | **0.5–2%** | Fraction of standing floor area replaced per year in test scenarios | **TCE proposal; low**; replace with endogenous demolition and rebuilding when available |

Two safeguards matter more than the exact starting values.

First, **repeatedly walking past the same building must not count as an unlimited series of independent persuasive encounters**. Familiarity should saturate.

Second, a separate “hybridization probability” is optional. Hybridization can emerge from choosing compatible components from several known repertoires. This is preferable to periodically forcing cultures to merge.

### 2.3 Why stock turnover creates long visual lag

Consider one specified trait or style bundle. Let:

* \(x\): its share of standing floor area;
* \(q\): its share of newly constructed floor area;
* \(r\): annual replacement rate of existing floor area;
* \(g\): annual net growth rate of total floor area.

Under **constant nonnegative growth, style-neutral replacement and no renovation**, a simple stock model gives:

\[
\frac{dx}{dt}=(r+g)(q-x).
\]

Suppose every new building immediately adopts the trait: \(q=1\), starting from \(x=0\). Then:

\[
x(t)=1-e^{-(r+g)t},
\qquad
t\_{50}=\frac{\ln 2}{r+g},
\qquad
t\_{90}=\frac{\ln 10}{r+g}.
\]

These are **calculated scenario results**, not historical building-lifetime estimates:

| Replacement \(r\) | Net growth \(g\) | Years until 50% of stock has the trait | Years until 90% |
| --- | --- | --- | --- |
| 0.5%/year | 0%/year | **139** | **461** |
| 1%/year | 0%/year | **69** | **230** |
| 2%/year | 0%/year | **35** | **115** |
| 0.5%/year | 2%/year | **28** | **92** |

At 1% replacement and no growth, even immediate universal adoption in new construction changes only about **39% of the stock after fifty years**.

**Implication for TCE:** apparent architectural conservatism can arise without conservative tastes. Conversely, a rapidly growing settlement can look stylistically transformed without demolishing most older buildings.

In the full model, replacement should depend on age, condition, damage, rent, ownership and financing. Ornament, façades, roofs and structural cores should have different opportunities for alteration. Restoration can reproduce an old recipe rather than count as adoption of a new one.

---

## 3. Variation across social conditions and world regions

The rows below are comparative historical settings, **not eras that TCE should unlock**.

| Setting | Historically relevant variation | Modeling consequence |
| --- | --- | --- |
| **Foragers** | Mobility varies substantially. Northern Japan’s Jōmon record includes sedentary hunter-fisher-gatherer settlements and substantial ritual places; foraging does not imply temporary shelters everywhere. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1632/) | Derive architectural persistence from mobility, storage, seasonality, tenure and ritual investment—not the label “forager.” |
| **Early farming communities** | At Çatalhöyük, the eastern settlement has **18 levels dated approximately 7400–6200 BCE**, including closely packed, roof-accessed houses. Long settlement sequences do not prove unchanged architectural taste. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1405/) | Give households and local building groups strong roles. Preserve plot and rebuilding histories; allow domestic form to change without a state or professional architect. |
| **Pre-industrial urban societies** | Courts, religious establishments, merchants and specialist workshops can support long-distance connections. Kilwa and Chinese mosques demonstrate very different local realizations of connected religious and commercial worlds. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/144/) | Use institutional and trade networks alongside geographic proximity. Keep religious, structural and decorative transmission partly independent. |
| **Industrializing societies** | The Washington, Missouri survey links architectural change to rail-distributed publications and milled lumber: information and practical building inputs became more accessible together. [NPGallery](https://npgallery.nps.gov/pdfhost/docs/NRHP/Text/64500319.pdf) | Improvements to transport should affect both exposure and feasibility. Catalogues alone should not eliminate local material and skill bottlenecks. |
| **Modern societies** | Tel Aviv illustrates migrant professional knowledge and climatic adaptation; Asmara illustrates concentrated colonial investment and segregated planning. Neither is explained by autonomous fashion alone. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1096/) | Add professional networks, standardized products and powerful commissioning organizations when those institutions emerge. Preserve local constraints, political inequality and inherited stock. |

**Avoid continent-level personality parameters** such as “Asia is conservative” or “Europe innovates quickly.” For TCE, the explanatory variables should be specific: workshop connections, patron resources, political authority, ritual requirements, migration, transport and available materials.

The same settlement can also have several tempos simultaneously: conservative sacred architecture, fashionable elite houses, rapidly changing commercial frontages and inexpensive dwellings repaired using familiar techniques.

---

## 4. Stylized facts and validation targets

A correct simulation need not reproduce Gothic cathedrals in a particular year. It should reproduce the following **conditional patterns** when comparable mechanisms are present.

| Pattern to reproduce | Evidence or numerical anchor | A useful TCE test |
| --- | --- | --- |
| **Adoption is geographically uneven rather than a smooth expanding circle.** | The project offsets and local housing lag in Section 2 are different kinds of delay; they cannot be collapsed into one diffusion rate. | Give a distant town strong workshop and patron connections. It should sometimes adopt before a nearer but weakly connected settlement. |
| **Local coherence can coexist with wider diversity.** | In the Washington survey’s **pre-1870 study group, 93%** of buildings were red brick with ornamental brick cornices. This describes selected surveyed survivors, not all original buildings. [NPGallery](https://npgallery.nps.gov/pdfhost/docs/NRHP/Text/64500319.pdf) | Shared materials and builders should create coherent districts without requiring identical household preferences. |
| **Hybrids preserve meaningful combinations.** | Chinese mosques can retain local compound forms while meeting Islamic religious requirements. [Penn Today](https://penntoday.upenn.edu/news/east-asian-art-prof-documents-early-chinese-mosques) | Imported ritual requirements should alter relevant spaces and symbols without randomly replacing unrelated roof or structural systems. |
| **Physical renewal and stylistic turnover are independent.** | Ise’s approximately **20-year** renewal practice preserves prescribed form. [Ise Jingu](https://www.isejingu.or.jp/en/ritual/index.html) | A tradition-preserving institution should sustain a recognizable design through many complete material replacements. |
| **Old fabric and old designs can persist through institutional continuity.** | Hōryū-ji preserves structures from the late seventh and eighth centuries; Cologne’s completion returned to a much older design. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/660/) | Support maintenance, interrupted projects and revival. Do not make design knowledge or visual form expire automatically with age. |
| **Growth changes the apparent speed of style change.** | In the derived example, adding 2% annual growth to 0.5% replacement reduces the half-transformation time from about **139 to 28 years**. | Hold project preferences constant and vary growth. New districts should respond more rapidly than established neighborhoods. |
| **Political meaning can change without rebuilding.** | Asmara’s colonial inheritance subsequently acquired Eritrean national significance. [UNESCO World Heritage Centre](https://whc.unesco.org/en/list/1550/) | Change ownership, commemoration and interpretation independently from meshes and construction recipes. |

For evaluation, record at least three separate outputs: **preferences**, **new-project choices**, and **standing-stock composition**. Matching only the final skyline could conceal an incorrect transmission mechanism.

---

## 5. Recommended TCE representation and execution

### 5.1 Represent a repertoire, not a single “culture style”

A building recipe should contain several linked layers:

| Layer | Example contents | Principal constraints |
| --- | --- | --- |
| **Construction** | Timber framing, masonry system, vault or truss type, foundation method | Materials, specialist knowledge, structural compatibility |
| **Spatial program** | Rooms, courts, halls, circulation, ritual orientation, relationship between buildings | Household needs, institution, use, plot |
| **Visible form** | Roof geometry, openings, massing, proportions, façade organization | Construction system, climate, preferences |
| **Surface and symbolism** | Color, carving, inscriptions, decorative motifs, emblems | Craft capability, identity, doctrine, budget |
| **Execution quality** | Finish, precision, ornament density, material quality | Labor, wealth, supply and maintenance |

**Style labels should summarize recurring combinations.** They should not determine all five layers in advance.

For example, a settlement might develop a recognizable preference for shaded courtyards, narrow openings and painted lintels across several wall materials and building uses. A new religious institution might retain the structural and climatic package while modifying circulation, orientation and inscriptions.

Require compatibility between components. A roof must have valid support; an opening must be compatible with its wall system; an ensemble’s circulation must serve its intended program. Hybridization should operate within those constraints.

### 5.2 Put different kinds of memory in different entities

| Entity | State worth storing |
| --- | --- |
| **Individual resident** | A compact preference profile; social and religious affiliations; a few salient exemplars; judgments about relevant people and institutions |
| **Household or patron** | Space requirements, resources and financing, tenure, project intentions, preferred builders, desired public expression |
| **Builder or workshop** | Known recipes, proficiency, apprenticeship relationships, team capacity, supplier connections and completed work |
| **Institution** | Ritual or political requirements, resources, authority, favored exemplars, geographic connections, maintenance commitments and design memory |
| **Building** | Component recipes, construction phases, dates, condition, ownership, use, alterations and transmission provenance |

Do not initialize guilds, monasteries or architectural professions merely because the architectural subsystem needs them. Early settlements can use household and work-group knowledge; specialized organizations should emerge from the broader simulation.

Likewise, do not put all construction knowledge into a cultural technology flag. A society can retain knowledge that a technique exists while losing the local specialists capable of executing it.

### 5.3 Make a commission the main architectural decision event

A practical project-selection model is:

\[
\Pr(b\mid i,\text{project})
\propto
\mathbf{1}[b\in\mathcal{B}\_{i,\text{project}}]
\exp\!\left(\frac{U\_i(b)}{T\_i}\right),
\]

where \(\mathcal{B}\_{i,\text{project}}\) is the set of **known, feasible candidate designs** and \(T\_i\) controls choice variability.

An illustrative utility function is:

\[
U\_i(b)=
w\_p\,\text{prestige}
+w\_f\,\text{functional fit}
+w\_r\,\text{ritual/identity fit}
+w\_l\,\text{local familiarity}
+w\_n\,\text{novelty}
-w\_c\,\text{cost burden}
-w\_u\,\text{uncertainty}.
\]

This is a **proposed decision model**, not a historical regression.

Normalize the scores and make costs relative to the project’s financing. Some requirements are hard constraints; others are preferences or expected penalties. An unenforced prohibition should not necessarily eliminate a candidate, while physical impossibility should.

An event sequence can then be:

1. A household or institution develops a building need and obtains resources.
2. It identifies available builders and a small set of candidate recipes.
3. Builders adapt candidates to the site and reject infeasible combinations.
4. Patron and builder select a design.
5. Actual work installs components over time, potentially with interruptions.
6. The completed or partially completed project becomes an exemplar.
7. Its reception, performance and patronage affect subsequent commissions and learning.

This creates feedback without forcing progress. A successful project may spread a design; a collapse, funding failure, unpopular patron or religious prohibition may weaken its appeal.

### 5.4 Distinguish taste innovation from technical innovation

A novel combination of familiar ornament and proportions is not equivalent to discovering a new structural technique.

For v1, distinguish:

**Recombination:** a builder proposes a new combination of already available, compatible components.

**Adaptation:** a known recipe changes dimensions, material details or arrangement to fit a site and program.

**Technical innovation:** a newly viable construction method expands the feasible set. This should interact with your technology, experimentation and skill systems.

**Revival:** an old known recipe becomes desirable again. It may require recovering skills or importing specialists; an archived drawing alone need not suffice.

This separation prevents the grammar from creating technically impossible buildings merely because an agent has a strong preference for novelty.

### 5.5 Rust and Unreal division of responsibility

For the stated architecture, **Rust should own design decisions and building history**. Unreal should realize the installed state.

A useful boundary is:

```
Construction decision
  -> validated recipe and component plan
  -> funded construction / alteration events
  -> installed component state
  -> Unreal instance additions, removals or replacements
```

Store original and altered phases rather than overwriting a building’s recipe wholesale. A new porch, replaced roof and retained wall core should remain distinguishable.

For 10k–50k people, the following are sensible engineering choices to benchmark:

* Accumulate meaningful exposure through existing travel and social events.
* Update preferences periodically or after significant encounters, not every frame.
* Keep a small bounded set of salient exemplars per person; **5–20** is a prototype storage choice, not an empirical claim.
* Store detailed recipes principally in workshops and institutions.
* Evaluate architecture only when a real project or alteration occurs.
* Aggregate settlement culture from agent distributions; never overwrite individuals with the settlement average.

Use deterministic random streams for decisions and preserve recipe provenance. This makes it possible to inspect a building and explain: “commissioned by this household, adapted by this workshop, influenced by these exemplars, constrained by these materials.”

### 5.6 Existing models and games: what to borrow

| Model or system | Useful contribution | What TCE must add or avoid |
| --- | --- | --- |
| **Axelrod, “The Dissemination of Culture” (1997)** | A baseline for local convergence and persistent differences using interacting agents with multiple cultural features. A public NetLogo implementation is available through CoMSES. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002797041002001) | Add nonlocal networks, unequal influence, craftspeople, feasibility and durable buildings. Similarity-driven copying alone is insufficient. |
| **Henrich and Gil-White’s prestige account (2001)** | A reason to distinguish admired models from coercive authorities and to make social learning selective. [ResearchGate](https://www.researchgate.net/publication/11954021_The_Evolution_of_Prestige_Freely_Conferred_Deference_as_a_Mechanism_for_Enhancing_the_Benefits_of_Cultural_Transmission) | Architecture-specific influence weights still require calibration. Do not equate prestige mechanically with wealth. |
| **Centola and Macy’s complex-contagion model (2007)** | A way to represent adoption that requires several reinforcing sources. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/521848) | Apply selectively to costly or uncertain choices. It should not prevent simple awareness or motif copying through a single contact. |
| **Crusader Kings III: documented Royal Court hybrid-culture design** | Separates cultural components and allows hybrid aesthetics, including selecting architecture from a parent culture. [Reddit](https://www.reddit.com/r/CrusaderKings/comments/o5m2cg/dev_diary_65_one_culture_is_not_enough/) | Borrow component separation, not instant culture-wide visual assignment. The documented automatic combination of parental innovations is inappropriate for TCE’s local craft capabilities. |
| **CityEngine** | Demonstrates rule-driven generation of architectural geometry from adjustable inputs. [Esri](https://www.esri.com/en-us/arcgis/products/arcgis-cityengine/overview) | It is a geometry-production reference, not a model explaining why a patron or culture chooses those inputs. |

**Recommended v1 priority:** first implement durable building phases, local craft capabilities and project budgets; then prestige copying and institutional networks; then richer hybridization, revival and explicit style naming. This should produce more credible historical texture than beginning with sophisticated taste drift applied to instantly replaceable buildings.

---

## 6. Sources, datasets and limits of the evidence

### 6.1 A practical calibration corpus

| Source or dataset | Useful content | Limitations |
| --- | --- | --- |
| **Mapping Gothic France**, hosted by Columbia | Building imagery, spatial context and historical information for comparing major medieval projects and phases. [Media Center Image Database](https://mcid.mcah.columbia.edu/mapping-gothic) | Strong monumental and regional selection; not a denominator for ordinary-house adoption. |
| **D-PLACE — Kirby et al. (2016)** | The original release covers **more than 1,400 societies**, combining cultural, environmental and linguistic information. Useful for comparative associations involving house forms and social conditions. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) | Mostly comparative snapshots, not architectural diffusion histories. Related societies are not independent observations. |
| **WikiChurches — Barz and Denzler (2021)** | **9,485 images**, architectural labels and a smaller set of feature annotations. Useful for evaluating whether generated features and combinations remain visually recognizable. [arXiv](https://arxiv.org/abs/2108.06959) | Image classification data, not a reliable time series of construction decisions or population preferences. |
| **Building Technology Heritage Library**, Association for Preservation Technology International | Historical catalogues, plan books and technical publications: evidence of designs and products being offered. [APT International](https://www.apti.org/building-technology-heritage-library) | Publication establishes availability or promotion, not actual local adoption. |
| **Sheals and Snider, *Historic Resources of Washington, Missouri* (1999)** | A local architectural inventory useful for ordinary-building chronology, regional lag and combinations of vernacular forms with fashionable details. [NPGallery](https://npgallery.nps.gov/pdfhost/docs/NRHP/Text/64500319.pdf) | Surviving and surveyed properties introduce selection bias. |
| **UNESCO site dossiers and associated inventories** | Construction phases, materials, institutional histories, alterations and conservation records across regions. | Monumental selection and heritage interpretation; cross-check uncertain dates and distinguish original fabric from restoration. |

For the case-study interpretation, particularly useful scholarly starting points are **Nancy Shatzman Steinhardt’s *China’s Early Mosques* (2015)**, **Finbarr Barry Flood’s *Objects of Translation* (2009)**, and **Kim Bongryol’s account of Korean Buddhist architecture**. They help replace broad labels such as “Islamic influence” or “Buddhist style” with specific builders, forms, institutions and transformations. [Penn Today](https://penntoday.upenn.edu/news/east-asian-art-prof-documents-early-chinese-mosques)

### 6.2 What remains contested or thinly evidenced

**Universal rates are not established.** The sources support particular chronologies and transmission mechanisms, not a single defensible annual probability of architectural innovation, copying or hybridization. Keep those parameters exposed and test alternative settings.

**Chronology is usually stronger than causal weighting.** A building can be dated reasonably well while the relative contributions of prestige, liturgy, structural experimentation and political competition remain interpretive. Avoid turning a persuasive historical narrative into a measured coefficient.

**Survivorship is severe.** Monumental masonry, protected religious buildings and officially surveyed districts do not represent all historical construction. Cheap, frequently replaced housing is particularly important for TCE and needs archaeological and local-inventory evidence alongside celebrated monuments.

**Ethnographic observations are not direct measurements of prehistory.** In D-PLACE’s Ethnographic Atlas component, approximately **69% of focal dates fall in 1900–1950**, and only about **3% predate 1800**. These observations can inform hypotheses about associations, but cannot be assigned uncritically to early farming or prehistoric foraging societies. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391)

**Stylistic categories can hide the process being modeled.** A later label may group buildings whose patrons and builders understood their choices differently. Record observable traits, phases and connections first; use style names as summaries.

**Hybridization does not establish consent.** Shared workmanship or mixed motifs can accompany migration and cooperation, but also conquest, appropriation or unequal patronage. The model should preserve those different causal histories even when the resulting geometry looks similar. [AAE Portal](https://aaeportal.com/publications/-18546/objects-of-translation-material-culture-and-medieval-hindu-muslim-encounter)

### Bottom line

The strongest foundation for TCE is **a project-based, networked system with durable material memory**:

**People encounter and evaluate exemplars; workshops retain and adapt knowledge; institutions fund and constrain projects; buildings preserve the consequences.**

With that structure, regional lag, conservative sacred forms, fashionable façades, hybrid districts, revival, interrupted monuments and divergent local traditions can emerge from the same rules—without scripted historical eras or automatic culture-wide architectural replacements.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92907-bb30-83e9-bf13-c6fbc03fca84)
