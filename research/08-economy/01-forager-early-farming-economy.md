# Food economies before money: a simulation-ready report for TCE

## Central conclusion

**TCE should begin with people producing within households, sharing through several overlapping relationships, and holding rights over particular resources—not with either isolated barter traders or a settlement-wide communal inventory.** Food-sharing research among the Agta and Mbendjele BaYaka identifies households nested within small sharing clusters and wider camps. Australian research shows that sharing can be actively demanded through recognized relationships rather than offered as a gift or negotiated as a trade. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0960982216305644)

The transition to farming should change this system gradually. **Cultivation, domestication, sedentism, storage, private ownership, and political hierarchy are separate variables.** Archaeological granaries preceded fully domesticated cereals in southwest Asia, while pottery was used by Japanese hunter-gatherers long before farming became their economic foundation. Neither storage nor pottery should require an “agricultural era.” [DOI](https://doi.org/10.1073%2Fpnas.0812764106)

Throughout this report, distinguish:

* **Observed evidence:** ethnographic measurements and archaeological finds.
* **Reconstructions:** estimates derived from experiments, analogies, settlement remains, or demographic models.
* **Proposed parameters:** useful development settings, explicitly not historical measurements.

The rules below are implementation proposals informed by the evidence, not claims that every society followed one economic logic.

---

# 1. Mechanisms: rules the simulation can implement

## 1.1 Production draws on seasonal resources and complete labor chains

Represent wild resources as spatially distributed stocks with regeneration, seasonality, accessibility, and depletion. Gathering from a productive patch today should affect tomorrow’s opportunities. Hunting, fishing, collecting shellfish, digging roots, and gathering nuts need different production functions.

For hunting, a useful starting model is:

\[
P(\text{successful acquisition})
=1-\exp(-\lambda\_{p,t}s\_i h)
\]

Here, \(\lambda\_{p,t}\) is a patch- and season-specific acquisition rate per search-hour, \(s\_i\) represents relevant skill and equipment, and \(h\) is search effort. Success produces a variable batch, constrained by actual prey stocks and transport capacity. Failure consumes time and energy without producing a compensating fraction of an animal.

Gathering can usually use a more continuous harvest function, although seasonal fruits, honey, and concentrated resources can also generate large, irregular returns.

**Keep three productivity measures separate:**

| Measure | Definition | What it determines |
| --- | --- | --- |
| Labor productivity | Edible kcal per complete-chain person-hour | Whether people have time for other activities |
| Land productivity | Edible kcal per hectare per year | How many people a territory can support |
| Energetic efficiency | Food energy acquired divided by energy expended | Physiological and, later, fuel/draft requirements |

An improvement can raise one while lowering another. These distinctions are central to research comparing foraging and cultivation. [DOI](https://doi.org/10.1126/science.abf0130?utm_source=chatgpt.com)

For implementation, count search, travel, harvesting, carrying, processing, tool maintenance, and required fuel or water. **Do not apply a published whole-chain return rate and then charge the same processing labor again.**

Also distinguish food acquired, food eaten during work, food carried home, and food lost. A Hadza field study documents substantial consumption during forays; food arriving at camp is therefore not a complete measure of production or individual intake. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S1090513816000118)

## 1.2 Households allocate labor, but people remain individual agents

Use the household as a practical budgeting unit, not as an indivisible worker or necessarily a nuclear family. Forager co-residence research finds flexible residence arrangements and substantial co-residence among people who are not close biological relatives. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21393537/)

A household should estimate its food requirements, available workers, inventories, obligations, and near-term opportunities. Individuals then accept, modify, or reject work according to health, skill, relationships, preferences, and social norms.

Include nonfood tasks from the beginning: carrying water, collecting fuel, maintaining shelter, repairing tools, preparing containers, childcare, and processing food. Otherwise the simulation creates an unrealistically large pool of “idle” labor.

Recommended rules:

* Productivity depends on task-specific experience, physical capacity, tools, and local knowledge.
* Children and older people can contribute to suitable tasks without becoming interchangeable adult workers.
* Care responsibilities constrain where someone can work and for how long.
* Labor parties can cross household boundaries.
* Gendered task expectations should be cultural rules that can change, not immutable biological job permissions.

Maintain both **clock time** and **attention requirements**. Some supervision can overlap with other work; a person cannot simultaneously perform two strenuous, location-exclusive jobs.

## 1.3 Sharing is a set of institutions, not one generosity coefficient

Demand sharing means that someone can request resources through a socially recognized relationship. Compliance may express obligation, membership, or pressure to be generous. It need not create a precisely repayable debt. Such practices can persist in economies that also contain wages and government payments. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/abs/10.1525/aa.1993.95.4.02a00050)

Implement several transfer protocols:

| Protocol | Trigger | Allocation logic | Relationship consequence |
| --- | --- | --- | --- |
| Household provisioning | Meal preparation or household shortage | Dependents, needs, household priorities | Usually no explicit debt |
| Demand sharing | A recognized person requests food or goods | Need, relationship, visibility, applicable norms | Refusal may damage relations or invite sanctions |
| Reciprocal assistance | A partner experiences a shortfall | Past assistance and expected future interaction | Creates a social expectation, not necessarily an exact account |
| Collective distribution | A shared acquisition or communal event | Participation, membership, customary claims | Reinforces or contests group membership and authority |
| Exchange | Parties agree to transfer goods or labor | Negotiated terms | May settle an obligation immediately or create credit |

These protocols can coexist. A person may retain a tool, share a large food acquisition broadly, lend a container, and owe labor to a relative.

**Separate ownership from the ability to refuse a claim.** An inventory can belong to a household while other people have legitimate claims on part of it.

For performance and realism, resolve requests locally: household first, close sharing partners next, wider networks when appropriate. Do not automatically equalize everyone’s stocks. Allow concealment, refusal, favoritism, negotiation, and withdrawal from relationships.

### Why sharing reduces risk—and where it fails

For an illustrative equal-sharing group of \(n\) producers, each with production variance \(\sigma^2\) and pairwise correlation \(\rho\):

\[
\operatorname{Var}(\text{production per member after pooling})
=\sigma^2\left[\rho+\frac{1-\rho}{n}\right].
\]

With eight producers and independent outcomes, variance falls to **12.5%** of individual variance. With correlation \(0.8\), it remains **82.5%**.

These are mathematical illustrations, not measured prehistoric parameters. They imply an important rule: **sharing can buffer one hunter’s bad day much more effectively than a drought affecting everyone.** Experimental virtual-foraging research also finds that higher production risk can encourage reciprocal exchange, although that is laboratory evidence rather than direct observation of ancient institutions. [Chapman University Digital Commons](https://digitalcommons.chapman.edu/esi_pubs/19/)

## 1.4 Storage exchanges present costs for future security

Storage should be an activity with material requirements and risks, not an inventory checkbox.

Each food lot needs a commodity, quantity, location, owner or rights-holder, processing state, age, and deterioration conditions. Facilities modify moisture exposure, pests, access, and catastrophic risks.

Preservation recipes require labor and inputs. Drying or smoking may reduce transport weight and extend usability, but does not manufacture additional food energy. Containers, drying space, fuel, and suitable weather can become bottlenecks.

Use explicit stock accounting:

\[
\Delta S\_g
=H\_g+T^{in}\_g-C\_g-T^{out}\_g-U\_g-L\_g
\]

where \(H\) is new harvest, \(T\) transfers, \(C\) consumption, \(U\) use in processing or planting, and \(L\) loss. Processing inputs generate separate output lots; transfers cancel when aggregated across the whole world.

**Seed deserves its own inventory state.** Edibility and germination viability are different properties. Seed eaten during a shortage may prevent immediate hunger while reducing future planted area.

Do not treat all reserve strategies alike. A grain bin, fish-drying rack, unharvested root patch, and live herd have different labor costs, access constraints, and risks.

The raised, ventilated granaries found at Dhra’ demonstrate deliberate protection of stored food before fully domesticated cereals. Their location and architecture do not, by themselves, prove a particular ownership regime. [DOI](https://doi.org/10.1073%2Fpnas.0812764106)

## 1.5 Farming adds scheduling commitments, not an automatic productivity bonus

Represent cultivation as a seasonal task chain:

**Prepare land → plant → weed/tend/guard → harvest → dry/process → store → prepare meals.**

The appropriate chain varies by crop and technique; not every system requires full-field hoeing or the same processing sequence.

Yield should depend on the area successfully prepared and planted, seed or planting material, soil, moisture, crop traits, weeds, and the timing of labor. Missing a planting or harvest window can matter more than total annual labor availability.

Crucially:

\[
\text{Seed requirement}
=\text{planned planted area}\times\text{seed rate}.
\]

Do not define seed as a constant percentage of the realized harvest. A poor harvest makes a fixed sowing requirement more burdensome.

The productivity debate does not justify either “farmers always work harder” or “farming always improves hourly returns.” Bowles’s reconstruction argues that early cereal cultivation need not outperform foraging once indirect costs are included. Contemporary horticultural comparisons and within-population studies answer different questions, with different crops, tools, and labor definitions. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3064343/)

**Decision rule:** compare another unit of cultivation with the *next-best activity it displaces*, not with average foraging returns. Cultivation can expand because accessible wild resources are crowded, because land yields are higher, or because predictable harvests fit household needs—even without improving leisure.

Domestication should remain distinct from cultivation: heritable crop changes emerge through repeated propagation and selection, rather than appearing immediately when someone first plants a seed. [DOI](https://doi.org/10.1073%2Fpnas.1308937110)

## 1.6 Goods circulate without requiring money—or universal barter

Give households production recipes for tools, containers, shelter components, clothing, cordage, and food processing. These compete for the same time and materials as subsistence.

Permit goods and labor to move through gifts, assistance, loans, marriage-related obligations, visiting relationships, collective work, and negotiated exchange. Record the **kind of obligation**, not merely an equivalent monetary value.

Part-time specialization can arise when an individual becomes particularly proficient and receives food or assistance from others. Full-time specialization requires dependable support, whether supplied through kinship, patronage, redistribution, credit, or markets.

Archaeological distributions of obsidian demonstrate connections between settlements. They do not uniquely identify the social mechanism behind every transfer. Agent-based work on early Near Eastern obsidian exchange shows how network structure can help explain spatial distributions, but a successful model fit is not proof that all exchanges were barter transactions. [MDPI](https://www.mdpi.com/2079-8954/4/2/18)

## 1.7 Settlement is a recurring decision, not an era transition

Evaluate staying, relocating, splitting, or joining another group using expected resource access, travel costs, water, stored reserves, fields, infrastructure, conflict, and relationships.

A fixed investment makes moving costly; a depleted local catchment makes staying costly. Both effects should operate simultaneously.

Allow seasonal aggregation and dispersal. Do not impose a universal camp or village population cap. The ethnographic distribution of residential group sizes is broad, and archaeological settlement estimates depend strongly on which buildings were occupied simultaneously. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/?utm_source=chatgpt.com)

Storage can support collective insurance or concentrated power. The proposition that easily appropriated cereals favor hierarchy is an influential hypothesis, not a rule that “grain creates states.” Keep storage ownership, enforcement, tribute, and authority as separately evolving institutions. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/718372)

---

# 2. Quantitative parameters and calibration ranges

**Confidence below concerns transfer into TCE, not merely whether a number was printed accurately.** “Medium” usually means a useful contextual anchor; “low” means strong dependence on reconstruction or local circumstances. Reported ranges are not probability distributions.

## 2.1 Ethnographic production and social organization

| Parameter | Quantitative evidence | Scope and source | Confidence / implementation use |
| --- | --- | --- | --- |
| Gross food return rate | **729 kcal/hour** for hunter-gatherers; **2,162 kcal/hour** for horticulturalists, group means | Kraft et al. (2021); database contains 14 forager and 22 horticultural populations, with availability varying by metric. [DOI](https://doi.org/10.1126/science.abf0130?utm_source=chatgpt.com) | **Medium** as comparative anchors; **low** as early-Neolithic recipe values |
| Recorded subsistence time | Means **4.5 hours/day** for foragers and **4.1 hours/day** for horticulturalists | Same cross-cultural compilation. [DOI](https://doi.org/10.1126/science.abf0130?utm_source=chatgpt.com) | **Medium**; not equivalent to total work plus care |
| Residential group size | Mean **41.4**, median **27**, range **13–250 people**; 263 cases | Compilation reported by Smith et al. (2010), drawing largely on Marlowe (2005). [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/?utm_source=chatgpt.com) | **Medium**; calibration distribution, not a fixed band size |
| Population density | Mean **0.30**, median **0.12**, range **<0.01–11 people/km²**; 312 cases | Same ethnographic compilation. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/?utm_source=chatgpt.com) | **Medium–low** across settings; ecology and territorial definitions matter |
| Residential mobility | Mean **7.4**, median **5.5**, range **0–58 moves/year**; 312 cases | Same compilation. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/?utm_source=chatgpt.com) | **Medium** for variation; no universal monthly moving schedule |
| Intermediate sharing cluster | Approximately **3–4 households** | Agta and Mbendjele BaYaka food-sharing networks; Dyble et al. (2016). [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0960982216305644) | **Medium** for these cases; useful nested-network template |

The return-rate difference in Kraft et al. is substantial in the means but statistically borderline in that comparison, **\(P=0.052\)**. It should not become an automatic threefold “farming technology” multiplier. Nor should separate cross-population means be multiplied together and treated as an observed household budget. [DOI](https://doi.org/10.1126/science.abf0130?utm_source=chatgpt.com)

## 2.2 Farming, storage, and settlement reconstructions

| Parameter | Value or range | Source and measurement basis | Confidence / caution |
| --- | --- | --- | --- |
| Hulled-wheat crop yield | Approximately **700–1,200 kg/ha/year** | Shukurov et al. (2015), Cucuteni–Trypillia reconstruction; not net food after seed and losses. [arXiv](https://arxiv.org/pdf/1505.05121) | **Low–medium**; regional model, not universal early-farming yield |
| Seed rates for relevant ancient wheat analogs | Approximately **67–100 kg/ha** | Cultivar/environment analogs assembled in that study. [arXiv](https://arxiv.org/pdf/1505.05121) | **Medium** for analogs; **low** for direct prehistoric transfer |
| Hand soil preparation | **10–20 m²/person-hour**; equivalent to **500–1,000 person-hours/ha** | Digging-stick/stone-hoe estimates compiled by Shukurov et al. [arXiv](https://arxiv.org/pdf/1505.05121) | **Low–medium**; soil and technique dependent |
| Flint-sickle reaping | **30–40 m²/person-hour**; equivalent to **250–333 person-hours/ha** | Particular low-cut harvesting experiments in the same synthesis. [arXiv](https://arxiv.org/pdf/1505.05121) | **Medium** for the specified method; excludes subsequent processing |
| Cereal storage loss | Approximately **10% of stored cereals** used as a reconstruction assumption | Bowles (2011), based on modern analogy rather than a prehistoric annual measurement. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3064343/) | **Low**; useful sensitivity anchor only |
| Small agricultural settlement feasibility | **50–300 people**, approximately **2–10 ha built area** | Shukurov et al.’s modeled economic configuration. [ePrints](https://eprints.ncl.ac.uk/235466) | **Low–medium**; neither a global distribution nor a maximum |
| Çatalhöyük East population | A 2024 reconstruction estimates **600–800 residents** in an average year of the **6700–6500 BCE** phase, versus earlier estimates in the thousands | Kuijt and Marciniak (2024), incorporating building histories and contemporaneity. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416524000047?utm_source=chatgpt.com) | **Low–medium; contested**. Preserve competing reconstructions |
| Early agricultural regional density | Taraco Peninsula phase indices imply approximately **7.1, 17.9, and 35.8 people/km²** across Early, Middle, and Late Chiripa, collectively **1500–250 BCE** | Calculated from Bandy’s population indices of 693, 1,752, and 3,507 over a **98 km²** survey. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/497665) | **Low–medium**; phase indices are not simultaneous censuses |
| Fertility change associated with Neolithic transitions | Approximately **+2 births per woman** in a prominent reconstruction | Bocquet-Appel (2011), inferred from demographic evidence. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21798934/) | **Low–medium**; not an instantaneous farming bonus |

**Never compare built-up settlement density directly with territorial population density.** A compact village can occupy little space while depending on a much larger agricultural, grazing, woodland, and gathering catchment.

## 2.3 Proposed development settings—not historical estimates

These are starting points for sensitivity testing. Replace them with region- and resource-specific calibrations as content matures.

| Parameter | Suggested starting value / test range | Units and purpose | Status |
| --- | --- | --- | --- |
| Adult-equivalent food requirement | Start **2,500**; test **2,000–3,200** | kcal/adult-equivalent/day; individual requirements must vary | **Proposed**, not a universal physiological value |
| Routine dry-stock deterioration | Test **5%, 10%, 20%** | Fraction lost per year under specified storage conditions | **Proposed**; catastrophic losses modeled separately |
| Correlation of food-production shocks | Test **0, 0.4, 0.8** | Dimensionless; local/idiosyncratic through widespread shocks | **Proposed scenario grid** |
| Immediate shortage assessment | **1–3 days** | Forecast horizon for requesting assistance or changing daily work | **Proposed** |
| Seasonal reserve objective | Until next reliable food opportunity, plus **0–90 days** | Days of household requirements | **Proposed**, institution- and ecology-dependent |
| Critical field-work window | Test **15–45 days** | Available days for a particular seasonal operation | **Proposed**; replace with local crop/weather calendars |
| Conditional hunt failure | Test **20–80%** for a defined expedition type | Failed expeditions / attempted expeditions | **Proposed stress test**, not an ethnographic range |

For constant deterioration, convert annual loss \(L\) into a daily retained fraction:

\[
r\_{\mathrm{day}}=(1-L)^{1/365}.
\]

This is preferable to arbitrarily choosing a daily spoilage percentage that accidentally destroys most annual reserves.

### Worked household budget

Consider a **hypothetical five-adult-equivalent household** requiring 2,500 kcal per adult-equivalent daily, with cereals supplying 60% of calories and containing an assumed 3,400 kcal/kg.

Annual cereal requirement:

\[
G=\frac{5\times2{,}500\times365\times0.60}{3{,}400}
\approx805\text{ kg}.
\]

Assume a clean-grain harvest of 1,000 kg/ha, seed reservation of 100 kg/ha, and 10% subsequent loss from the consumption stock:

\[
G\_{\mathrm{available}}=(1{,}000-100)\times0.9
=810\text{ kg/ha}.
\]

The household needs approximately **0.99 harvested hectares**. At a yield of 500 kg/ha, the requirement becomes **2.24 hectares**. Two fallow years per cropped year would triple rotational land requirements, before adding other land uses.

Now impose a labor deadline: two workers, 30 days, eight hours daily, and 15 m²/person-hour can prepare only:

\[
2\times30\times8\times15=7{,}200\text{ m²}=0.72\text{ ha}.
\]

**The annual food budget appears feasible, but the seasonal labor budget is not.** The household needs assistance, a different technique, a longer suitable window, another food source, or less dependence on cereals. The remaining 40% of its diet still requires resources and labor.

---

# 3. Variation across regions and economic systems

## 3.1 Regional pathways that should remain possible

| Region or tradition | Relevant evidence | Consequence for TCE |
| --- | --- | --- |
| **Southwest Asia** | Dhra’ granaries date to approximately **9350–9225 BCE**, before fully domesticated cereals. [DOI](https://doi.org/10.1073%2Fpnas.0812764106) | Allow cultivation and large seasonal stores without requiring completed domestication |
| **East Asia** | Rice and millet followed different domestication pathways. Japanese pottery containing evidence of aquatic-food processing dates to approximately **15,000–11,800 calibrated years BP**. [DOI](https://doi.org/10.1073%2Fpnas.1308937110) | Separate wetland and dryland cultivation; allow pottery among foragers and fishers |
| **African Sahel and connections to South Asia** | Research distinguishes sorghum’s northeastern African history from pearl millet’s western Sahelian history, with different relationships among cultivation, pastoralism, and settlement. Both later dispersed beyond their original regions. [Springer](https://link.springer.com/article/10.1007/s10437-018-9314-2) | Do not require a wheat-and-goat package; support mixed herding, gathering, fishing, and cultivation |
| **New Guinea highlands** | Kuk Swamp provides evidence for banana cultivation and mounded cultivation around **6950–6440 calibrated years BP**, with later ditched drainage. [Deakin University Research Repository](https://dro.deakin.edu.au/articles/journal_contribution/Origins_of_agriculture_at_Kuk_Swamp_in_the_highlands_of_New_Guinea/21023134) | Vegetatively propagated crops and drainage investments need their own recipes, planting material, and harvest schedules |
| **Mesoamerica, eastern North America, and tropical South America** | Crop domestication and agricultural dependence were not simultaneous. Indigenous eastern North American seed crops, for example, preceded their later dietary dominance; tropical American cultivation also developed through prolonged mixed economies. [DOI](https://doi.org/10.1073%2Fpnas.1308937110) | Permit millennia of mixed subsistence in historical validation; no instant conversion from “forager” to “farmer” |
| **North American Pacific drainage systems** | Bridge River, on the interior British Columbia plateau, combines fishing, storage, village aggregation, and changing inequality. Storage capacity did not map straightforwardly onto elite rank. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416513000597) | Productive fisheries can support settled communities; stored wealth and hierarchy must not be mechanically identical |
| **Indigenous Australia** | Demand-sharing institutions can remain important within contemporary domestic economies containing monetary income. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/taja.12036) | Introducing money must not delete kin obligations, requests, or shared consumption |

“Calibrated years BP” uses **1950** as its reference year, not the present simulation date.

## 3.2 What changes across the requested eras?

These are possible economic configurations, not stages that every TCE society must pass through.

| Configuration | Historical pattern | Modeling implication |
| --- | --- | --- |
| **Foraging economies** | Substantial variation in mobility, fishing reliance, settlement size, and population density appears in cross-cultural data. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/?utm_source=chatgpt.com) | Calibrate ecology and institutions independently; “forager” should not mean uniformly nomadic or without reserves |
| **Early farming economies** | Cultivation, domestication, and dependence on crops can develop at different speeds. [DOI](https://doi.org/10.1073%2Fpnas.1308937110) | Add field commitments, seasonal bottlenecks, planting material, and household claims while retaining wild-food production |
| **Preindustrial agrarian economies** | Production can connect complementary specialists. In Burkina Faso, archaeological isotope evidence supports long-term links between cultivation and livestock-manure management. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0305440319300640) | Model exchanges of labor, manure, animal services, and food; surplus extraction must operate through actual rights and enforcement |
| **Industrial economies** | U.S. farm employment fell from approximately **41% of the workforce in 1900 to 2% in 2000**, a dated example of structural transformation. [whitehouse.gov](https://obamawhitehouse.archives.gov/node/11491/) | Mechanization moves production beyond household muscle; retain upstream fuel, machinery, transport, and maintenance requirements |
| **Modern economies** | Agricultural specialization and integration into wider markets coexist with household provisioning and nonmarket obligations. [Economic Research Service](https://www.ers.usda.gov/publications/44198) | Add firms, wages, purchased food, and large distribution systems without replacing individual needs, household cooking, or social transfers |

The industrial employment figures concern **farm employment**, not everyone working in food processing, transport, retail, or input production.

---

# 4. Stylized facts and validation targets

A correct simulation should reproduce the following patterns **under the conditions that generate them**, not force all of them into every world.

| Pattern to reproduce | Validation target |
| --- | --- |
| **Small groups with wide variation** | An ethnographically comparable forager ensemble should accommodate a median near **27 residents**, but also substantially smaller and larger groups—not repeatedly snap to one preferred size. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/?utm_source=chatgpt.com) |
| **Nested sharing rather than universal pooling** | Configure cases with household provisioning, **3–4-household clusters**, and wider camp transfers; verify that different foods and relationships generate different transfer networks. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0960982216305644) |
| **Production and consumption are different distributions** | Under effective sharing, unlucky producers can continue eating; successful producers need not consume everything they acquire. Run separate production, transfer, and intake diagnostics |
| **Pooling is weaker against common shocks** | The same sharing institution should handle isolated failures better than region-wide shortages, as the covariance calculation predicts |
| **Annual sufficiency does not guarantee seasonal sufficiency** | A household may have enough expected annual output but fail at sowing, harvest, processing, or the preharvest reserve gap |
| **Farming does not guarantee more leisure** | Reproduce both high-return horticultural configurations and transitions where agricultural involvement reduces leisure, as observed among the Agta. [Nature](https://www.nature.com/articles/s41562-019-0614-6) |
| **More food can support more dependents rather than greater comfort** | Permit increased population and fertility without requiring improved per-person consumption; compare cautiously with demographic-transition reconstructions. [PubMed](https://pubmed.ncbi.nlm.nih.gov/21798934/) |
| **Economic transitions can be incomplete or reversed** | Sedentary fishers, pottery-using foragers, crop-growing mobile groups, abandoned fields, and renewed reliance on wild foods must remain representable |
| **Settlement growth can slow or reverse** | Do not impose uninterrupted exponential growth. Archaeological settlement analyses in Mexico and the Titicaca region identify growth followed by substantial slowing. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/497665) |

The last two are particularly important for an endless simulation: resource management and institutions should generate persistence, transformation, fission, and abandonment without scripted historical events.

---

# 5. Recommended representation in TCE

## 5.1 Core entities

| Entity | Minimum economically relevant state |
| --- | --- |
| **Person** | Age, health, needs, skills, current task, carried goods, relationships, remembered experiences |
| **Household** | Members, shared budget, meal arrangements, expected obligations, planning preferences |
| **Production party** | Participants, task, equipment contributions, allocation rules |
| **Patch or field** | Location, resource stocks, regeneration or crop stage, access rights, expected productivity |
| **Food or goods lot** | Commodity, quantity, processing state, physical location, ownership, deterioration, seed viability where relevant |
| **Facility** | Capacity, access, processing services, maintenance, storage protection |
| **Institution or norm set** | Eligibility to claim, distribution priorities, tenure, sanctions, inheritance or succession |
| **Relationship edge** | Kinship, affiliation, trust, assistance history, outstanding obligations of specified kinds |

**A lot’s physical location, legal or customary owner, and eligible claimants must be separate fields.** This supports household bins in communal buildings, individually owned tools used by others, and food subject to sharing claims.

## 5.2 Daily execution loop

Use a two-stage **intent and resolution** system:

1. People and households assess needs, deadlines, and remembered opportunities.
2. Agents propose work, travel, assistance requests, and transfers.
3. The kernel reserves contested resources and resolves access.
4. Work creates or transforms physical lots.
5. Goods are transported, processed, shared, consumed, or stored.
6. Physiology, relationships, equipment, and ecological stocks update.

Reservations prevent multiple agents from independently harvesting the same resource. Transfers require access and transport; belonging to one settlement does not make distant food instantly available.

Agents should use imperfect expectations. A household may overestimate a hunting area, underestimate spoilage, or trust a partner who later refuses help. Internal comparisons expressed in calories or expected utility are planning devices—not evidence that agents possess money.

## 5.3 What to simplify—and what not to simplify

**Keep explicit:** food energy, at least a basic protein or diet-composition constraint, seasonal task deadlines, transport, seed, storage losses, access rights, common shocks, and individual consumption.

**Simplify initially:** exact bargaining dialogue, every plant genotype, comprehensive nutrient chemistry, individual grains, and continuous optimization over every possible partner.

For 10k–50k agents, use batched ecological and stock updates, sparse relationship networks, local spatial queries, and cached expectations. Individualize consequential jobs and transfers rather than every biological process. Daily household planning can coexist with shorter-resolution movement and visible tasks.

Keep the Rust kernel authoritative: UE5 should render work and inventory movement, not independently complete recipes or create resources.

### Spatial scale is a substantive constraint

At an **assumed 0.1 person/km²**, a population of 10,000 requires about **100,000 km²**; 50,000 requires about **500,000 km²**. This is arithmetic illustrating scale, not a universal forager density.

Therefore, TCE needs some combination of small initial bands, a large strategic world, offscreen catchments, or explicit geographic compression. A visually compact map cannot silently support tens of thousands of low-density foragers while claiming realistic local resource budgets.

## 5.4 Existing models and games worth borrowing from

| Precedent | Relevant contribution | What not to copy uncritically |
| --- | --- | --- |
| **Village Ecodynamics Project / Village models** | Household agents, spatial agricultural constraints, food and other resource requirements, and experiments with specialization and exchange in the ancestral Pueblo region. [JASSS](https://www.jasss.org/16/4/4.html) | Household aggregation hides individual care and labor variation; a modeled barter regime is one institutional possibility |
| **Ortega et al. (2016), early Near Eastern obsidian-exchange ABM** | Connects settlement networks and exchange rules to archaeological distance-distribution patterns. [MDPI](https://www.mdpi.com/2079-8954/4/2/18) | A distributional fit does not uniquely establish the historical exchange mechanism |
| **Dawn of Man** | Visible acquisition and processing chains, hunting products, seasonal resources, and preparation for winter. [Madruga Works](https://www.madrugaworks.com/dawnofman/) | Its authored age progression is unsuitable as TCE’s causal structure |
| **Clanfolk** | Household-scale chores, seasonal preparation, food deterioration, fuel, and livestock feeding. [Hooded Horse](https://wiki.hoodedhorse.com/Clanfolk/Food_Production) | Use its interactions as design references, not its balance values as historical evidence |

The strongest combination is **Village-style resource accounting, individual visible work, and explicitly authored but dynamically competing sharing and tenure institutions**.

---

# 6. Source priorities, datasets, and remaining uncertainty

## 6.1 Sources to use as the calibration backbone

| Research source | Best use in TCE | Important limitation |
| --- | --- | --- |
| **Kraft et al. (2021), “The energetics of uniquely human subsistence strategies,” Science** | Comparative return rates, activity budgets, and supplementary subsistence data. [DOI](https://doi.org/10.1126/science.abf0130?utm_source=chatgpt.com) | Contemporary cases; heterogeneous measurement boundaries |
| **Peterson (1993), “Demand Sharing,” American Anthropologist; Dyble et al. (2016), Current Biology** | Sharing protocols, relationship-based claims, and multilevel networks. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/abs/10.1525/aa.1993.95.4.02a00050) | Local institutions should not become universal behavioral laws |
| **Smith et al. (2010), “Wealth Transmission and Inequality among Hunter-Gatherers,” Current Anthropology** | Comparative group size, density, mobility, and distinctions among forms of wealth. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/?utm_source=chatgpt.com) | Compiled ethnographic samples are not independent random samples of prehistory |
| **Bowles (2011), PNAS; Shukurov et al. (2015), Human Biology** | Auditable labor, seed, yield, and storage assumptions. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC3064343/) | Reconstructions are sensitive to techniques, processing assumptions, and analog selection |
| **Bandy (2005), Current Anthropology; Kuijt and Marciniak (2024), Journal of Anthropological Archaeology** | Settlement-demography tests and explicit uncertainty over contemporaneous occupation. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/497665) | Archaeological phase populations are not direct censuses |
| **D-PLACE’s Binford dataset** | Cross-cultural ecological, mobility, subsistence, and organizational comparisons across **339 hunter-gatherer groups**. [GitHub](https://github.com/D-PLACE/dplace-dataset-binford) | Distinguish observed inputs from modeled estimates; inspect variable definitions and missingness |
| **Ortega et al. (2016), Systems, supplementary site data** | Archaeological exchange distributions for testing network models. [MDPI](https://www.mdpi.com/2079-8954/4/2/18) | Source distributions constrain outcomes more strongly than the motives behind exchange |

D-PLACE’s Binford repository lists a **CC BY-NC 4.0** license; public accessibility should not be treated as permission to redistribute the dataset inside a commercial game. Some Agta individual-level data also require researcher access and community approval rather than unrestricted reuse. [GitHub](https://github.com/D-PLACE/dplace-dataset-binford)

## 6.2 Claims that should remain explicitly uncertain

**“Foragers worked only a few hours.”** Activity categories differ. Food acquisition, processing, domestic work, supervision, and leisure cannot be collapsed into one number without checking the original observation protocol.

**“Farming was either a mistake or an obvious labor-saving invention.”** Early-cereal reconstructions, contemporary horticultural comparisons, and observational transitions do not measure the same counterfactual. TCE should reproduce the trade-offs rather than settle them by assigning one universal multiplier.

**“A large granary proves communal ownership—or an elite.”** Architecture constrains possible practices but rarely identifies the full distribution regime. At Bridge River, storage and inequality do not support such a simple equation. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416513000597)

**“Excavated houses equal simultaneous residents.”** Building replacement, abandonment, reuse, and uncertain occupation lengths can change population estimates dramatically. The revised Çatalhöyük estimates are a particularly clear warning. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0278416524000047?utm_source=chatgpt.com)

**“Ethnographic foragers are unchanged prehistoric populations.”** D-PLACE observations come from particular historical contexts, many relatively recent. Use them as evidence for feasible behaviors and ecological relationships, not as a single preserved ancestral economy. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4938595/)

## Implementation priority

Build **seasonal resources, complete labor chains, household provisioning, physical storage, and local sharing claims** first. Add cultivation as another production system competing for the same people, time, land, and obligations.

That foundation allows money, markets, firms, tribute, specialization, and cities to emerge later without rewriting the economy—and allows worlds in which some of them never emerge at all.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9286e-abf0-83e9-ab7f-3f64ef7667fe)
