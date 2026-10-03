# Gender, age, and the division of labor

## A simulation-ready research report for The Civilization Engine

**Recommendation: make occupations emerge from task requirements, access to resources and training, household care obligations, bargaining power, and socially enforced expectations. Do not assign occupations directly from gender or age.**

The evidence supports neither a universal “men produce, women care” template nor an assumption that gendered specialization is absent. Divisions of labor are widespread, but their content, rigidity, and consequences vary. Crucially, **equal participation in an activity can coexist with strong specialization within it**: the Ethnographic Atlas explicitly distinguishes equal agricultural participation with different tasks from equal participation without task differentiation. [d-place.org](https://d-place.org/parameters/EA054)

For TCE, the useful causal structure is:

**Task demands and technology → feasible opportunities → household and individual decisions → accumulated skills and observed behavior → expectations, sanctions, and institutions → future opportunities.**

This creates persistence without making roles permanent. It also allows apparently similar settlements to develop different divisions of labor.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Count all production, not just employment

A person may produce food, process grain, collect fuel, maintain clothing, supervise children, and assist older relatives without holding a recognized occupation. Modern time-use research treats these activities separately from employment; excluding them changes the apparent division of labor substantially. Historical occupational labels are therefore an inadequate template for an agent’s daily production. [International Labour Organization](https://www.ilo.org/publications/unpaid-care-work-and-labour-market-analysis-time-use-data-based-latest)

**Implementable rule:** every necessary household service must consume actual time, goods, infrastructure, or another person’s labor. “Homemaker,” “dependent,” and “unemployed” must not mean economically inactive.

Record three distinct quantities:

| Quantity | What it measures |
| --- | --- |
| Participation | Whether someone performs an activity during the observation period |
| Labor input | Hours, effort, or effective task time contributed |
| Output and control | What is produced, who receives it, and who decides how it is used |

A woman contributing half the agricultural hours need not control half the land or harvest. Likewise, a child can contribute work while remaining a net consumer.

### 1.2 Allocate tasks, not indivisible occupations

Classic cross-cultural research identifies patterned combinations of activities rather than a single universal division. Making or acquiring a product can also be associated with processing and using it, producing linked task bundles. [HRAF](https://hraf.yale.edu/ehc/documents/280)

**Implementable rule:** decompose “farmer,” “hunter,” and “craftsperson” into tasks with different requirements.

For farming, distinguish clearing, animal handling, ploughing, planting, weeding, harvesting, transport, threshing, storage, and food preparation. For hunting, distinguish tracking, pursuit, trapping, driving animals, killing, carrying, butchery, and preservation.

Give each task properties such as strength demand, endurance demand, skill complexity, danger, distance from home, interruptibility, seasonality, equipment requirements, and opportunities for assistance. Evaluate individuals against those properties—not against an occupational gender tag.

### 1.3 Care obligations create constraints, but caregivers are substitutable

A task’s compatibility with caring for a dependent matters independently of its physical difficulty. Proximity to home, the ability to stop suddenly, and the availability of other caregivers can alter who performs it. Among Tsimane households, task delegation responds to harvest demands, household composition, and parental absence; children sometimes perform tasks that are strongly gendered among adults. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/stieglitzetal2013ca.pdf)

**Implementable rule:** dependents generate time-specific care requirements. Households can meet these through parents, siblings, grandparents, other kin, neighbors, servants, or institutions.

Represent nursing and reproductive recovery separately from general childcare. The former depend on individual physiological state; the latter need not be restricted by gender.

Do not make care disappear when its usual provider takes another job. It must be transferred, purchased, combined with compatible work, or left inadequately supplied. Conversely, shared care should sometimes make activities possible that would otherwise be inaccessible.

### 1.4 Skills and expectations reinforce one another

Specialization can become self-reinforcing: people train for roles they expect to occupy, and their resulting comparative advantage makes those roles more attractive. Hadfield’s coordination model demonstrates how gendered specialization can arise through coordinated human-capital investment rather than requiring large initial productivity differences. It is a theoretical mechanism, not a calibrated universal explanation. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0167268199000530)

**Implementable rule:** practice improves task-specific competence; access to practice depends on mentors, tools, invitations, household assignments, and anticipated future opportunities.

This produces a feedback loop:

> Expected role → training opportunity → observed competence → employer and family expectations → expected role.

Removing a prohibition should therefore not instantly erase an occupational imbalance. Previously excluded agents may still lack skills, equipment, contacts, or confidence in receiving fair treatment.

### 1.5 Household allocation involves power, not just efficiency

Household work cannot safely be modeled as a benevolent planner maximizing one shared utility. Members may differ over consumption, leisure, schooling, prestige, and control of income. In Tsimane research, delegation can support cooperation and learning but also create parent–child conflicts over present production versus future development. Historical research on China likewise shows hierarchies among women within households, not simply one undifferentiated female position. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/stieglitzetal2013ca.pdf)

**Implementable rule:** distinguish the person doing work, the person directing it, and the person controlling its returns.

Bargaining power can depend on property, alternative employment, kin support, institutional office, age rank, and the credible ability to refuse or leave. Where coercion exists, model commands, resistance, punishment, and escape opportunities explicitly. Do not present compelled labor as an unconstrained preference.

This also permits an older household head to direct labor while doing relatively little physical work.

### 1.6 Technology changes task bundles; it does not automatically produce equality

Boserup’s plough hypothesis proposes that agricultural technology helped shape gendered work and subsequent norms. Alesina, Giuliano, and Nunn find that ancestral plough use predicts less egalitarian gender attitudes and lower female participation in several public economic domains, including among descendants of immigrants raised in a common destination country. That supports cultural persistence, but does not supply a universal effect of introducing one plough into a village. [OUP Academic](https://academic.oup.com/qje/article/128/2/469/1943509)

**Implementable rule:** a new technology changes particular task requirements, returns, schedules, and ownership structures.

A plough can change land preparation while leaving weeding, harvesting, processing, and care largely intact. A mill can remove household processing work but concentrate income in whoever owns it. A factory can reduce some strength requirements while imposing rigid attendance incompatible with existing care arrangements.

Technology can also relax constraints: Dinkelman’s South African electrification study found a substantial increase in women’s employment following rollout. Treat such findings as evidence for a mechanism, not as a universal employment bonus attached to electricity. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.101.7.3078)

### 1.7 Children work through a mixture of learning, contribution, and compulsion

Children’s participation is not adequately represented by either “no work until adulthood” or “small adults with reduced productivity.” In research with Yucatec Maya children aged 7–11, children described observing, initiating help, and learning through participation; belonging and responsibility mattered alongside adult instruction. [Karger Publishers](https://karger.com/hde/article/65/4/191/820948/How-Yucatec-Maya-Children-Learn-to-Help-at-Home)

**Implementable rule:** distinguish exploratory participation, supervised learning, routine contribution, and compulsory labor. Each has different immediate output, learning, fatigue, risk, and autonomy consequences.

Poverty can increase pressure to work, but assets can also increase demand for children’s labor. Bhalotra and Heady found that the association between land ownership and child farm labor persisted for girls in rural Ghana and Pakistan after conditioning on other factors. Thus, child work must respond to both household need and available productive opportunities. [World Bank](https://documents1.worldbank.org/curated/en/343821468179964730/pdf/774040JRN020030IC00Child0Farm0Labor.pdf)

Schooling should compete with some work while complementing other future work. In Bangladesh, garment-sector expansion was associated with increased schooling among younger girls and increased employment among older girls, alongside delayed marriage and childbirth. [IZA](https://www.iza.org/publications/dp/8483)

### 1.8 Older adults change their task portfolios rather than simply becoming inactive

Physical performance, practical skill, recognized expertise, and social authority have different age profiles. Among Tsimane adults, older people’s skill portfolios became more concentrated in difficult, lower-strength activities; recognized expertise in music and storytelling was especially late-peaking. These are not equivalent to direct measurements of hourly productivity. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/schniteretal2015.pdf)

**Implementable rule:** age affects several capacities separately. Allow declining endurance alongside retained craft skill, ecological knowledge, teaching ability, or administrative authority.

Older people can substitute toward processing, repair, supervision, care, trade, teaching, and decision-making when opportunities exist. Others may become substantially dependent because of illness or disability. Neither outcome should be automatic at a single birthday.

Institutional retirement should be a separate mechanism involving income security, eligibility, employment rules, and preferences—not the biological termination of productive capacity.

### 1.9 Perceived norms can differ from actual preferences

In a Saudi Arabian experiment, men frequently underestimated other men’s support for women working outside the home. Correcting these beliefs changed job-search-related behavior. This demonstrates why behavior, private approval, and perceived approval must be separate state variables. [University of Chicago Home](https://home.uchicago.edu/bursztyn/Misperceived_Norms_2020_3_6.pdf)

**Implementable rule:** an agent can personally approve of a role while avoiding it because they expect condemnation. Other agents can make the same mistake, sustaining an apparently unanimous convention.

Visible counterexamples, private conversations, public endorsements, and institutional announcements should update different beliefs. An observed female craft worker is evidence that the activity occurs; it is not necessarily evidence that everyone approves.

### 1.10 Norms can change within lives, not only through cohort replacement

A randomized school intervention in India shifted adolescents’ gender attitudes, with effects still present two years after the intervention ended. Adult expectations also changed in the Saudi experiment. Norms are therefore neither instantaneous reflections of incentives nor immutable childhood programming. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20201112)

**Implementable rule:** allow both gradual socialization and event-driven revision. Migration, teacher influence, successful role violations, economic shocks, and political organization can alter beliefs.

Behavior may change before approval, or approval before opportunity. Preserve these possibilities rather than updating everything together.

---

## 2. Quantitative parameters and calibration targets

### How to interpret the numbers

Most available estimates are **observed outcomes**, not coefficients that can be inserted directly into an agent’s decision function. A country’s female agricultural labor share does not identify the strength of its gender norm.

Confidence below refers to the stated measurement within its studied setting. **Transferability to an early agrarian simulation is a separate judgment.**

### 2.1 Empirical anchors

| Measure | Quantitative finding | Appropriate use in TCE | Confidence and limitations |
| --- | --- | --- | --- |
| Agricultural gender specialization | The displayed EA054 data contain **72 male-only**, **170 male-biased**, **92 differentiated-but-equal**, **141 equal-without-marked-differentiation**, **228 female-biased**, and **32 female-only** cases. | Validate the existence of multiple agricultural regimes, including equality with task segregation. | **High confidence in coded counts; moderate in underlying classification.** Not population-weighted frequencies or measured hour shares. [d-place.org](https://d-place.org/parameters/EA054) |
| Hunting performance over age | Approximately **23,000 hunting records**, **more than 1,800 individuals**, **40 locations**; average peak around **30–35 years**. | Anchor a hunting-specific maturation curve with substantial individual variation and an extended adult plateau. | **Moderate–high.** Applies to observed hunters and modeled hunting performance, not all work. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/default/files/sitefiles/papers/kosteretal2020.pdf) |
| Acquisition of complex subsistence and social skills | Tsimane median reported acquisition ages across categories were approximately **13–16 years for females** and **14–17 for males**. | Separate first participation, acquisition, proficiency, and recognized expertise. | **Moderate.** Retrospective reports; category-specific and culturally structured. Not a minimum age for helping. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/schniteretal2015.pdf) |
| Late-life recognized expertise | Tsimane music/storytelling expertise peaked over late-age ranges beginning around **65 for women** and **66 for men**. | Make cultural and teaching roles potentially valuable well beyond peak physical performance. | **Moderate for reputational expertise; low as a direct productivity coefficient.** [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/schniteretal2015.pdf) |
| Adult production time in two Amazonian populations | Machiguenga/Piro estimates place peak male production near **5 hours/day** and female production near **6 hours/day**. | A local time-allocation benchmark, not a universal workday. | **Moderate.** Estimates derive from a **12-hour observation day** and omit overnight foraging trips and wage labor. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/GurvenKaplan2006.pdf) |
| Women’s crop-production labor share | Six-country African study: average approximately **40%**; **24% Niger**, **29% Ethiopia**, **37% Nigeria**, and slightly above **50%** in Malawi, Tanzania, and Uganda. | Validate regional and household variation under different farming systems and opportunities. | **Moderate–high for surveyed crop labor.** Not all agricultural work, food output, or control of production. [PubMed](https://pubmed.ncbi.nlm.nih.gov/28413246/) |
| Unpaid care division | ILO’s 2018 estimates attributed **76.2%** of unpaid care hours to women, approximately **3.2 times** men’s contribution. | A modern aggregate target demonstrating the importance of nonmarket work. | **Moderate–high aggregate evidence**, with heterogeneous surveys and substantial country variation. Not a timeless ratio. [The United Nations in Indonesia](https://indonesia.un.org/en/250835-international-day-care-and-support-29-october) |
| Child labor, global, 2024 | Approximately **137.6 million**, or **7.8% of children aged 5–17**; hazardous work approximately **54.0 million**, or **3.1%**. | Validate a modern institutional scenario using the report’s definitions. | **High-quality official estimates**, subject to survey and modeling uncertainty. Do not equate child labor with every useful activity performed by children. [International Labour Organization](https://www.ilo.org/sites/default/files/2025-06/2024%20Global%20Estimates%20of%20Child%20Labour%20Report.pdf) |
| Geographic variation in child labor, 2024 | ILO regional prevalence: **21.5% sub-Saharan Africa**, **3.1% Asia–Pacific**, **5.5% Latin America/Caribbean**, **3.3% Europe/Central Asia**. | Demonstrate that one global child-work probability is unsuitable. | **High-quality aggregate estimates.** Regional categories and definitions must remain consistent. [International Labour Organization](https://www.ilo.org/sites/default/files/2025-06/2024%20Global%20Estimates%20of%20Child%20Labour%20Report.pdf) |
| Sectoral concentration of child labor | Approximately **61% agriculture**, **27% services**, **13% industry** in 2024; rounded shares. | A modern model should not place most child labor in factories. | **High-quality aggregate evidence.** These are shares among children classified as in child labor. [International Labour Organization](https://www.ilo.org/resource/other/2024-global-estimates-child-labour-figures) |
| Electrification and women’s employment | South African study estimates approximately **+9–9.5 percentage points over five years**. | Test whether reduced domestic constraints and changed local opportunities can meaningfully affect participation. | **Moderate–high local causal evidence.** Not a portable global elasticity. [Energia](https://energia.org/assets/2015/09/dinkelman_electricity_0810.pdf) |
| Correcting perceived gender norms | Saudi experiment: job-matching signup increased approximately **9 percentage points**, from a **23%** baseline. | Test rapid changes in action following credible information about peer approval. | **High internal validity for the intervention.** Signup is not employment. [J-PAL](https://www.povertyactionlab.org/evaluation/effects-misperceptions-social-norms-female-labor-force-participation-saudi-arabia) |
| Deliberate attitude change | Indian school intervention: **0.18 standard-deviation** shift toward gender equality after a two-year program; effects persisted **two years afterward**. | Validate within-cohort attitude change and persistence. | **High internal validity for attitudes**; weaker evidence for some behavioral outcomes. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20201112) |
| Older-age survival in subsistence populations | Gurven and Kaplan report an average adult modal death age around **72**, with a **68–78-year** range. | Ensure substantial scope for older adult roles despite high childhood mortality. | **Moderate.** This is an adult mortality mode, not life expectancy at birth or a retirement age. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/GurvenKaplan2007pdr.pdf) |

### 2.2 Parameters that should remain explicit design assumptions

The literature does **not** identify a universal norm-learning rate, penalty for role violation, or utility weight for tradition. These must be fitted jointly to outcomes and varied in sensitivity tests.

The following are proposed initial test settings, **not empirical human constants**:

| Model parameter | Suggested exploration range | Units and implementation meaning |
| --- | --- | --- |
| Update weight for a credible observation | **0.02–0.20** | Fraction of the gap closed between a current descriptive belief and new evidence |
| Update weight for personal endorsement | **0.001–0.05** | Fraction of the gap closed per salient, trusted evaluative event; keep separate from observation |
| Choice-noise scale | **0.1–1.0** | Relative to one standard deviation of normalized candidate-task utility |
| Occupational switching cost | **0–2** | Normalized utility units; apply to changing a sustained role, not every task transition |
| Probability a recognized violation is sanctioned | **0–1** | Conditional probability determined by audience, institution, and enforcement resources |
| Modest material sanctions | **0–10** | Days of household subsistence-equivalent value; severe coercion should be modeled as separate events |
| Care compatibility | **0–1** | Task-specific ability to provide effective supervision while working; calibrate separately by dependent needs |

**Do not infer these independently from aggregate gender shares.** Strong sanctions with weak skill differences can produce the same observed allocation as weak sanctions with highly unequal training access. Experiments and policy changes help distinguish mechanisms better than a single cross-section.

---

## 3. Variation across societies, regions, and economic systems

The categories below are comparison cases, not stages every TCE world should pass through.

| Setting | Evidence and variation | What the simulation should permit |
| --- | --- | --- |
| **Foraging societies across regions** | Gendered specialization is common, but women’s hunting is documented. The dispute concerns its prevalence, frequency, and implications—not whether it ever occurs. [University of Utah Faculty Profiles](https://profiles.faculty.utah.edu/u0839608/publications) | Flexible participation, task-specific specialization, joint production, and occasional or sustained cross-role work |
| **Australian Western Desert, Martu** | Research on women’s hunting emphasizes differences in resource reliability, risk, and social organization rather than a simple hunter/nonhunter distinction. [PubMed](https://pubmed.ncbi.nlm.nih.gov/19230267/) | Different prey, methods, group compositions, and return variance within the same nominal occupation |
| **Early farming, Central Europe** | Skeletal evidence indicates exceptionally intensive female upper-limb loading during the first millennia of farming. This contradicts equating domestic or processing work with light work; skeletal loading does not directly reveal hours or output. [Science](https://www.science.org/doi/10.1126/sciadv.aao3893) | Heavy processing and repetitive manual labor outside male-coded field tasks |
| **Mixed farming and household production, Amazonia and Mesoamerica** | Tsimane delegation changes seasonally and with household composition; Maya children’s accounts emphasize learning through contribution. These are contemporary or recent observations, not direct reconstructions of ancient farming. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/stieglitzetal2013ca.pdf) | Seasonal recruitment, mixed-age work groups, sibling assistance, and gradual responsibility |
| **African smallholder farming** | The six-country crop-labor estimates span approximately one-quarter to over one-half female labor input. “African agriculture is women’s work” is too coarse even before considering tasks, livestock, processing, or ownership. [PubMed](https://pubmed.ncbi.nlm.nih.gov/28413246/) | Strong variation between neighboring production systems and households—not continent-wide role rules |
| **Late imperial China** | Bray’s research links domestic architecture, textile production, commercialization, and household hierarchy. Women’s household location did not imply absence from productive, financial, educational, or ritual responsibilities. [University of California Press](https://www.ucpress.edu/books/technology-and-gender/paper) | Home-based market production, household managerial roles, and differences between wives, daughters, servants, and senior women |
| **Industrializing Britain** | Humphries’ historical research finds increased child participation and younger entry into work during the classic industrialization period, with family vulnerability central to the explanation. Industrialization did not immediately eliminate child labor. [JSTOR](https://www.jstor.org/stable/pdf/42921562.pdf) | Factories that initially recruit children and young adults, followed by changes in schooling, family income, regulation, and political support |
| **Manufacturing-led change, Bangladesh** | New garment opportunities changed both work and education incentives for girls, with different responses by age. [IZA](https://www.iza.org/publications/dp/8483) | Schooling and employment increasing together across different age groups |
| **Modern societies with persistent care specialization** | The Child Penalty Atlas documents parenthood-related employment differences across **134 countries**, using harmonized event-study-style estimates and differing underlying data structures. [OUP Academic](https://academic.oup.com/restud/article/92/5/3174/7840285) | Parenthood shocks interacting with childcare, job schedules, policy, and prior specialization rather than one universal maternal penalty |

### The U-shaped participation hypothesis is useful—but not a historical script

Goldin’s classic account describes circumstances in which women’s measured labor-force participation initially falls as production moves out of family enterprises, then rises with education and new employment opportunities. It does **not** mean women initially stop doing all work. [SSRN](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=233692)

Later cross-country work by Gaddis and Klasen finds little support for treating that U-shape as a general development law. The sectoral composition of growth, measurement choices, and institutions matter. [Springer](https://link.springer.com/article/10.1007/s00148-013-0488-2)

**TCE implication:** do not tie female participation to an “era,” GDP level, or technology score. Let it follow from the actual jobs, care arrangements, access rules, and returns available.

---

## 4. Stylized facts a successful simulation should reproduce

These are validation conditions, not quotas to impose on every generated society.

| Stylized fact | Operational test |
| --- | --- |
| **Specialization can exist without exclusion.** | A settlement can show balanced total agricultural labor while men and women concentrate on different component tasks, matching the distinction in EA054. [d-place.org](https://d-place.org/parameters/EA054) |
| **Regional labor shares vary substantially.** | Under appropriately different farming conditions, models can generate female crop-labor shares resembling the observed **24% to over 50%** range without changing a global female-productivity multiplier. [PubMed](https://pubmed.ncbi.nlm.nih.gov/28413246/) |
| **Child contribution responds to household demand and season.** | Harvest pressure and parental absence change assignments; children sometimes cross adult role boundaries. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/stieglitzetal2013ca.pdf) |
| **Children are not either fully dependent or fully productive.** | Track work contribution and net resource consumption separately. Allow useful chores alongside substantial dependence and learning. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/stieglitzetal2013ca.pdf) |
| **Different abilities peak at different ages.** | Hunting performance can peak around **30–35**, while some forms of recognized expertise peak much later. No single productivity curve should govern both. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/default/files/sitefiles/papers/kosteretal2020.pdf) |
| **Work and approval need not change together.** | A credible information intervention can alter participation-related behavior without immediately changing tools, physical capacity, or household composition. [University of Chicago Home](https://home.uchicago.edu/bursztyn/Misperceived_Norms_2020_3_6.pdf) |
| **Technological change can have ambiguous gender effects.** | Removing processing work can free time, transfer income to equipment owners, or both; changing opportunities should not mechanically equalize every task. The historical plough association and electrification evidence illustrate different pathways. [OUP Academic](https://academic.oup.com/qje/article/128/2/469/1943509) |
| **Child labor is not principally a factory phenomenon.** | In a comparable modern scenario, agriculture should be capable of accounting for roughly **three-fifths** of child labor. [International Labour Organization](https://www.ilo.org/resource/other/2024-global-estimates-child-labour-figures) |
| **Norm change can persist within a cohort.** | Simulated attitude interventions should be capable of effects lasting several years without requiring everyone to be replaced by a new generation. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20201112) |

Also run **engineering counterfactuals** that have no prescribed historical target:

**Label-swap test.** Holding physiology, skills, resources, relationships, and institutions fixed, changing a gender label must not silently change technical productivity.

**Constraint-removal test.** Remove care shortages, training exclusion, and sanctions separately. Each should have distinguishable effects.

**Accounting test.** New market employment must not create extra hours in the day or erase previously necessary household work.

**Path-dependence test.** Identical initial ecologies with different network and learning histories should sometimes converge on different conventions. They should not always become different, nor always converge.

---

## 5. Recommended individual-agent and institutional model

### 5.1 Minimum state representation

The following is a proposed architecture rather than a claim that people consciously calculate these variables.

| Layer | Recommended state |
| --- | --- |
| Individual | Continuous age; health; strength/endurance/mobility; reproductive state; task skills; fatigue; preferences; personal norm endorsements; beliefs about others; current commitments |
| Household | Dependents and care requirements; food and wealth; productive assets; ownership and control; resource-sharing arrangements; bargaining relationships |
| Task | Location; season; deadline; inputs; equipment; technical demands; interruptibility; danger; learning opportunity; team requirements |
| Social environment | Reference groups; observed participation; perceived approval; prestige; reputations; mentor and employer networks |
| Institution | Eligibility rules; property and inheritance rules; schooling obligations; labor contracts; sanctions; enforcement resources; political supporters and opponents |

Keep physiological attributes separate from gender categories. Historical datasets usually use binary classifications; that measurement limitation need not determine the engine’s ontology.

### 5.2 Technical productivity

A useful starting structure is:

\[
q\_{ij}
=
A\_j
\,s\_{ij}
\,h\_i
\,F(\mathbf c\_i,\mathbf d\_j)
\,t\_{ij},
\]

where:

* \(A\_j\) is technology and equipment effectiveness;
* \(s\_{ij}\) is task-specific skill;
* \(h\_i\) is current health/fatigue effectiveness;
* \(F\) matches individual capacities \(\mathbf c\_i\) to task demands \(\mathbf d\_j\);
* \(t\_{ij}\) is effective time.

**Gender is not an independent multiplier in this production function.** Any physical differences enter through measured capacities; social differences enter through access, training, assignments, and sanctions.

Use bottlenecks where appropriate. A task requiring controlled lifting or uninterrupted attention should not be represented as a smooth bonus from arbitrary extra strength. Tools, assistants, and alternative methods can change those requirements.

### 5.3 Task choice and assignment

For a feasible candidate task:

\[
U\_{ij}
=
V\_i(\text{return and its distribution})
+
L\_{ij}
+
R\_{ij}
+
D\_{ij}
-
E\_{ij}
-
C\_{ij}
-
S\_{ij}
-
K\_{ij}.
\]

Here \(L\) is learning value, \(R\) reputation or status, \(D\) duty or personal endorsement, \(E\) effort and risk, \(C\) care and other opportunity costs, \(S\) expected sanctions, and \(K\) switching costs.

The return term must reflect **who receives and controls the proceeds**, not just total household production.

A bounded-choice implementation can use:

\[
P(j\mid i)=
\frac{\exp(U\_{ij}/\tau\_i)}
{\sum\_{k\in\mathcal J\_i}\exp(U\_{ik}/\tau\_i)}.
\]

But use this only where the agent has meaningful discretion. Parents, employers, household heads, and coercive institutions can issue assignments; recipients then comply, negotiate, evade, or resist according to their available options.

**Separate technical infeasibility from prohibition.** Lacking a required tool may prevent a task. A guild rule against someone practicing a craft should create a legal and social risk, not make the underlying action physically impossible.

### 5.4 Care as a real scheduling system

Represent care as required coverage during particular windows, with demands varying by dependent condition.

For each window, calculate effective care supplied by available caregivers and compatible concurrent activities. Primary activities occupy clock time; secondary supervision occupies attention and may reduce other output.

For example, an agent might grind grain while supervising a nearby child, but not simultaneously provide equally effective distant-field labor. A school or communal nursery creates coverage only when staffed and supplied.

This prevents a major simulation error: making adults’ productive hours independent of the existence of children.

### 5.5 Skills, teaching, and age

Maintain distinct measures of:

**Exposure → basic competence → proficiency → recognized expertise.**

Use actual practice and teaching opportunities to update competence. Age should influence maturation and physical capacities, but should not automatically grant occupational mastery.

This distinction is supported by the gap between acquiring skills during adolescence, later proficiency, and still-later social recognition in the Tsimane evidence. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/schniteretal2015.pdf)

Permit children to contribute to low-demand substeps before they can perform an entire production chain. Permit older specialists to teach or supervise workers whose physical capacities exceed their own.

Avoid a universal “child productivity = 0.5 adult productivity” rule: it would give the same relative competence in carrying water, storytelling, herding, arithmetic, and metalworking.

### 5.6 Norms: maintain four separate quantities

For each relevant role–task combination, represent:

| Quantity | Example |
| --- | --- |
| Descriptive belief | “Most people in this role do this task.” |
| Personal endorsement | “I think this allocation is appropriate.” |
| Perceived approval | “People important to me approve of it.” |
| Institutional rule | “The guild permits, requires, or prohibits it.” |

A simple descriptive update is:

\[
B\_{t+1}=B\_t+\alpha w\_{\text{obs}}(y-B\_t),
\]

where observation weight depends on credibility, relevance, visibility, and familiarity.

**Do not automatically convert a majority practice into moral approval.** Personal endorsement can instead respond to experienced benefits and harms, teaching, prestigious examples, identity, and conflict. Approval beliefs respond to expressed opinions and observed sanctions.

Sanctioning should also have costs. An employer may dislike a role violation yet hire a competent worker during a shortage. A guild may enforce exclusion because incumbents benefit. A household may excuse behavior in an emergency without endorsing it generally.

### 5.7 How change emerges without scripted eras

A plausible endogenous sequence is:

A labor shortage makes a previously unusual assignment worthwhile. Someone gains experience. Their performance becomes visible. Employers or households revise expectations. Some observers revise approval; others resist. A coalition changes access rules. The next cohort receives different training opportunities.

A different sequence can reverse the change: demand collapses, owners recapture opportunities, sanctions return, and the newly trained group is displaced.

**Illustrative TCE scenario:** a harvest crisis leads households to recruit adolescents and older adults into selected tasks. An older woman’s successful organization raises her reputation. She trains apprentices and acquires control of storage. Her influence may change local expectations—but that outcome is contingent on ownership, kin support, and institutional resistance, not guaranteed by the crisis.

### 5.8 Computational simplification for 10,000–50,000 agents

These are engineering choices:

Use approximately **12–20 task families** for norm and skill aggregation, while rendering more detailed activities. Cache local opportunities and shared beliefs at household, neighborhood, occupation, and settlement levels. Give individuals sparse deviations and memories rather than a complete belief matrix about everyone.

Evaluate perhaps **6–12 locally relevant task alternatives** during planning, not every job in the world. At 50,000 agents and eight candidates, that is **400,000 candidate evaluations per planning pass**—a workload estimate, not a performance benchmark.

Update schedules daily and when events invalidate them. Update skills from performed work. Update beliefs from sampled social encounters. Reconsider institutional rules through political events and periodic deliberation, rather than every simulation tick.

### 5.9 Existing models and games: what to borrow

| Model or game | Useful element | What not to copy uncritically |
| --- | --- | --- |
| **Hadfield, 1999: coordination model of the sexual division of labor** | Expectations and investment can sustain specialization without large initial talent differences. | An equilibrium mechanism is not a complete account of coercion, care, or cultural change. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0167268199000530) |
| **Basu and Van, 1998: economics of child labor** | A formal framework linking adult earnings, household decisions, and child labor, including multiple possible labor-market outcomes. | Do not assume poverty alone explains all child work; asset-based labor demand and power also matter. [JSTOR](https://www.jstor.org/stable/116842) |
| **Gurven and colleagues: bioeconomic household models** | Age-specific production, dependency, household cooperation, and the division of labor belong in the same system. | Empirical role patterns from one population should not become universal eligibility rules. [Springer](https://link.springer.com/article/10.1007/s12110-009-9062-8) |
| **RimWorld: Biotech** | Childhood learning competes with work; development changes future capabilities. | Its authored growth moments at **7, 10, and 13** are game design, not anthropological calibration. [Ludeon Studios](https://ludeon.com/blog/2022/10/biotech-preview-3-reproduction-children-genetic-modification-release-date/) |
| **Victoria 3** | Laws, interest groups, and workforce participation are connected. | Its documented aggregate workforce-ratio effects are too coarse for TCE’s individual daily labor and endogenous task norms. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-45-elections) |

No example above provides the whole required system. The most useful combination is household production and dependency accounting, task-specific learning, socially conditioned choice, and endogenous institutional access.

---

## 6. Sources, datasets, and limits of inference

### Recommended calibration stack

| Source family | Best use | Main caution |
| --- | --- | --- |
| **Ethnographic Atlas and D-PLACE; Murdock–Provost research** | Task specialization, agricultural systems, household organization, and cross-cultural comparisons | Codes are not time diaries. Societies share ancestry and contact histories, so observations are not independent. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) |
| **Detailed ethnographic production studies** | Age-specific task participation, returns, learning, delegation, and seasonality | Intensive local observation gives strong mechanisms but limited geographic coverage. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/GurvenKaplan2006.pdf) |
| **LSMS-type household and agricultural surveys** | Plot-level labor input, household composition, assets, and regional variation | Respondent recall, seasonality, and the boundary of agricultural work matter. [World Bank](https://documents.worldbank.org/en/publication/documents-reports/documentdetail/979671468189858347/how-much-of-the-labor-in-african-agriculture-is-provided-by-women) |
| **ILO time-use compilations and UNICEF/ILO child-work data** | Modern work accounting and age/gender comparisons | Paid work, own-use production, household chores, and hazardous child labor are different statistical categories. [International Labour Organization](https://www.ilo.org/publications/unpaid-care-work-and-labour-market-analysis-time-use-data-based-latest) |
| **Child Penalty Atlas** | Parenthood-associated employment trajectories across modern countries | Understand its country-specific data and pseudo-event-study methodology before interpreting estimates as uniform causal effects. [Child Penalty Atlas](https://childpenaltyatlas.org/methodology) |
| **Experiments and quasi-experiments** | Distinguishing opportunity, belief, and attitude mechanisms | Local treatment effects should validate responses, not become universal constants. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.101.7.3078) |

### Claims that need explicit caution

**“Women did not hunt.”** False as a universal statement. But Anderson and colleagues’ reported **79% of 63 societies** with documented female hunting is not the proportion of women who hunted, the share of hunting time, or the share of meat supplied. A subsequent specialist critique identified sample-selection and coding problems. Use well-documented local cases and maintain both participation and intensity measures. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0287101)

**“The plough caused patriarchy.”** Too strong. The historical association and evidence of persistence are important, but broad institutions, inheritance, ecology, and political organization co-vary. For TCE, plough adoption should alter opportunities and bargaining conditions; a patriarchal outcome must still arise through modeled processes. [OUP Academic](https://academic.oup.com/qje/article/128/2/469/1943509)

**“Ethnographic societies are windows directly into prehistory.”** They are not. In the 2016 D-PLACE description, **69% of Ethnographic Atlas focal observations were from 1900–1950**, with only **3% before 1800**. Use them to identify possible mechanisms and configurations, not as unmodified Paleolithic or Neolithic snapshots. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391)

**“People in subsistence societies rarely lived long enough to contribute as elders.”** Low life expectancy at birth does not imply the absence of older adults. The adult mortality evidence is incompatible with eliminating productive social roles at forty or fifty. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/GurvenKaplan2007pdr.pdf)

**“The literature supplies universal labor-share targets.”** It does not. Evidence is particularly thin for ancient task hours, the numerical strength of sanctions, rates of norm revision, and the separate causal contributions of capacity, exclusion, and preference.

**Bottom line for TCE:** build the accounting of care and household production first, then task-level capabilities and learning, then access and bargaining, then beliefs and enforcement. Let the resulting gender and age distributions become observable properties of a society—not instructions imposed on its citizens.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92818-07e0-83e9-bd2d-80b1e4f0a38a)
