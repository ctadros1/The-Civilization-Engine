# Classifying economic systems for The Civilization Engine

## Executive recommendation

**TCE should classify economic systems as combinations of institutions, not as mutually exclusive historical stages.** Record who controls productive resources, how production and distribution are coordinated, how people obtain livelihoods, and who can appropriate their output. Infer labels from those observations afterward.

The major scholarly typologies answer different questions. Polanyi classifies forms of economic integration; Marxian approaches emphasize relations of production; Kornai analyzes mutually reinforcing institutional configurations; varieties-of-capitalism research distinguishes coordination arrangements within capitalist economies. They should therefore inform different classifier layers rather than compete for one universal taxonomy. [Taylor & Francis](https://www.taylorfrancis.com/chapters/edit/10.4324/9780429494338-2/economy-instituted-process-karl-polanyi)

A useful TCE description would be:

> **Seigneurial agrarian economy**, with market-oriented towns, communal grazing rights, and substantial household production.

That conveys more than “feudalism: 78%.” It also permits institutions to change independently.

**The principal evidence limitation:** the sources below do not establish universal numerical boundaries between capitalism, feudalism, socialism, or command economies. Historical observations can calibrate individual indicators. The thresholds that translate those indicators into UI labels must remain explicit, testable design choices.

---

## 1. Typologies and the mechanisms behind them

### 1.1 What the major typologies actually classify

| Framework | Central question | Observable features for TCE | Important limitation |
| --- | --- | --- | --- |
| **Polanyi: forms of integration** | How are economic activities connected across households and groups? | Reciprocal obligations, centralized collection and redistribution, exchange through price-making markets | A transfer’s institutional context matters. A gift, a tax, or a market transaction alone does not establish the dominant integration of an economy. |
| **Marxian modes of production** | Who controls the conditions of production, and how is surplus appropriated? | Producer access to land and tools, dependence on employers or landlords, enforceable labor obligations, residual claims, class reproduction | Different Marxian traditions disagree about boundaries and historical sequences. A labor form is not automatically an entire mode of production. |
| **Kornai’s system paradigm** | Which institutional arrangements reinforce one another into a recognizable system? | Ownership, political authority, bureaucratic versus market coordination, managerial appointment, investment control, budget constraints | His classical socialist configuration is not a definition of every possible socialist institution or mixed economy. |
| **Varieties of capitalism** | How do capitalist firms coordinate with workers, financiers, suppliers, and one another? | Collective bargaining, skill formation, finance and corporate governance, interfirm relations, employee relations | Primarily a comparison within capitalism, not a taxonomy suitable for every historical society. |

These distinctions come from the foundational works rather than a common measurement standard. [Taylor & Francis](https://www.taylorfrancis.com/chapters/edit/10.4324/9780429494338-2/economy-instituted-process-karl-polanyi)

#### Polanyi: classify integration, not merely transactions

Polanyi’s 1957 formulation distinguishes **reciprocity, redistribution, and exchange**, associated with patterned social relationships, collection around a center, and price-making markets. **Householding—production for a group’s own use—also deserves a TCE indicator**, but it should not be presented as an identical fourth category in the 1957 formulation: it is particularly prominent in *The Great Transformation* of 1944. [Taylor & Francis](https://www.taylorfrancis.com/chapters/edit/10.4324/9780429494338-2/economy-instituted-process-karl-polanyi)

For implementation, distinguish the protocol governing a transfer from its physical form. Grain might be given because of kinship obligations, collected as tribute, allocated as a ration, or sold. Identical goods can participate in different institutions.

Reciprocity need not mean immediate equal-value repayment; redistribution need not mean egalitarian redistribution. Neither should be represented simply as “trade without money.”

#### Marxian approaches: classify production relations

For Marx, commodity exchange alone is insufficient to identify capitalist production. His analysis distinguishes selling products from selling labor-power and emphasizes the separation of workers from independent access to means of production. That makes **livelihood dependence, control of production, and appropriation of the resulting surplus** more useful indicators than the mere existence of merchants or currency. [Marxists Internet Archive](https://www.marxists.org/archive/marx/works/1867-c1/ch06.htm)

For TCE, separate:

* **Producer position:** independent producer, tenant, employee, cooperative member, or person subject to coercive control.
* **Appropriation:** household retention, rent, obligatory service, taxation, interest, or enterprise residual income.
* **Reproduction:** whether property, dependency, and authority persist through inheritance, marriage, debt, or enforceable status.

“Feudalism” needs special caution. Political vassalage, land tenure, manorial organization, and dependent peasant production are related concepts, not interchangeable observations. Reynolds challenges conventional interpretations of medieval European fiefs and vassalage; the Sharma–Mukhia debate illustrates disagreement over applying feudalism to Indian history. TCE should expose the underlying relations and make the broad label optional. [OUP Academic](https://academic.oup.com/book/47420)

#### Kornai: distinguish defining institutions from predicted symptoms

Kornai’s classical socialist configuration links concentrated political authority, state dominance, bureaucratic coordination, plan bargaining, and characteristic growth and shortage dynamics. These are a proposed causal configuration, not interchangeable diagnostic signs. A shortage caused by drought does not establish a command economy. [OUP Academic](https://academic.oup.com/book/4729/chapter/146973120)

His **soft budget constraint** is especially reusable: an organization expects another actor to cover losses or prevent failure, and this expectation changes its behavior beforehand. A single subsidy or rescue is not enough to establish that expectation. Kornai, Maskin, and Roland explicitly discuss soft budget constraints beyond socialist economies. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2F002205103771799999)

Also retain distinctions within public ownership. Kornai’s discussion of self-management separates worker selection of managers and worker claims on residual income from appointment and control from above. [OUP Academic](https://academic.oup.com/book/4729/chapter/146977356)

#### Varieties of capitalism: classify coordination within capitalism

Hall and Soskice distinguish **liberal market economies**, relying more heavily on competitive markets and arm’s-length contracting, from **coordinated market economies**, relying more on negotiated and relational coordination. Their five spheres are industrial relations, training, corporate governance, interfirm relations, and employee relations. These are ideal types, not an exhaustive binary division. [UMass Courses](https://courses.umass.edu/marta/managmnt394g-marta/Varieties%20of%20CapitalismIntrodoction.pdf)

Do not force every modern economy into that pair. Schneider’s **hierarchical market economy** formulation for Latin America emphasizes diversified business groups, multinational companies, low skills, and atomistic labor relations. It provides an example of extending the dimensions rather than treating European and North American cases as universal templates. [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/ab1e3268-d9d9-4825-acd3-42d241440250)

### 1.2 Causal mechanisms expressed as simulation rules

The following are **proposed implementations of source-grounded mechanisms**, not empirically estimated universal equations.

| Mechanism | Rule TCE could implement | Observable consequences |
| --- | --- | --- |
| **Reciprocal provisioning** | Households transfer goods according to needs, available buffers, recognized obligations, and relationship history. Transfers update trust and future claims. | Persistent sharing networks; uneven reciprocity over time; dependence on social membership rather than purchasing power alone. Ethnographic research supports treating sharing and property arrangements as variable rather than uniformly egalitarian. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/) |
| **Collective resource governance** | A governing group can specify access, extraction, maintenance, monitoring, and sanctions for a resource. Compliance depends on actual enforcement and incentives. | Commons with effective exclusion and management, rather than automatically depleted open-access resources. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/PropertyRightsRegimesNaturalResources.SchlagerOstrom1992.pdf) |
| **Property–production complementarity** | Investment becomes more attractive when its benefits are defensible; a production technique can also change which rights are valuable to defend. | Institutional and technological changes reinforce one another. Bowles and Choi model this for farming and possession-based property, but it remains a particular explanatory model rather than a universal origin story. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/701789?mobileUi=0) |
| **Dependent production and extraction** | Institutions attach obligations to land, persons, offices, or contracts. Households comply, evade, renegotiate, migrate, or resist according to alternatives and expected consequences. | Rent, labor service, taxes, and coercion can emerge in different combinations. Do not collapse all obligatory payments into feudal rent. |
| **Markets and organizations** | Agents use exchange when search, transport, trust, and contracting costs permit. Organizations substitute internal direction where it is advantageous, while incurring managerial costs. | Markets coexist with firms and other hierarchies; organization-internal planning does not by itself make the whole economy a command economy. [OUP Academic](https://academic.oup.com/book/51875/chapter/420653388) |
| **Administrative planning** | Authorities issue binding production, input, delivery, and investment decisions using incomplete reports and limited resources. Units bargain over targets and respond to incentives. | Bottlenecks, queues, priority allocation, and reporting distortions may arise from the mechanism rather than an imposed “planning inefficiency” penalty. [OUP Academic](https://academic.oup.com/book/4729/chapter/146973120) |
| **Soft budgets** | Organizations learn the probability and conditions of rescue. Expected rescue changes investment, borrowing, and responses to losses; rescuers also face costs and political incentives. | Recurrent dependence on support may occur under several ownership regimes. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2F002205103771799999) |
| **Institutional transition** | Coalitions change specific rights, obligations, or decision procedures; enforcement and organizational adaptation occur unevenly. Old and new arrangements can coexist. | Persistent hybrids, partial reforms, and reversals. China’s dual-track reform is an important example of retaining planned obligations while permitting market activity at the margin. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/262113) |

**Do not make the inferred label cause these mechanisms.** “Feudalism” should summarize dependencies and rights already operating, not activate a package of dependencies and rights.

---

## 2. Observable indicators and quantitative parameters

### 2.1 Measurement specifications

The ranges below are **measurement domains**, not claims about typical historical values.

Record legal title, effective control, residual-income rights, and enforcement separately. Schlager and Ostrom’s distinction among access, withdrawal, management, exclusion, and alienation provides a useful foundation; TCE should add appointment, inheritance, and income claims where needed. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/PropertyRightsRegimesNaturalResources.SchlagerOstrom1992.pdf)

| Indicator | Operational definition and unit | Domain | Evidence basis and confidence |
| --- | --- | --- | --- |
| **Productive-resource control** | Share of land area, sector-specific capacity, or reference-value productive assets controlled by each holder type | 0–100% within each denominator | Rights-bundle approach; **high conceptual confidence**, historical measurement often weak. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/PropertyRightsRegimesNaturalResources.SchlagerOstrom1992.pdf) |
| **Rights distribution** | Holder of each right over each asset; record joint or conditional rights explicitly | Categorical or fractional | Do not assume title implies every right; **high conceptual confidence**. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/PropertyRightsRegimesNaturalResources.SchlagerOstrom1992.pdf) |
| **Household provisioning mix** | Shares of final household provisioning obtained through own production, reciprocity, redistribution, and market acquisition | 0–100% per channel | Polanyi-inspired operationalization; **medium**, because practical coding boundaries require conventions. [Taylor & Francis](https://www.taylorfrancis.com/chapters/edit/10.4324/9780429494338-2/economy-instituted-process-karl-polanyi) |
| **Administrative production control** | Share of production governed by binding cross-enterprise output or input assignments | 0–100% | Kornai-inspired; **medium** operational confidence. Ownership alone is not the measure. [OUP Academic](https://academic.oup.com/book/4729/chapter/146973120) |
| **Investment authority** | Share of new productive capacity whose creation and location are ultimately selected by households, firms, communities, or public authorities | 0–100% | **Proposed TCE indicator**; distinguish decision authority from financing source. |
| **Work relationship** | Person-hours classified by independence, organizational authority, payment form, and membership | 0–100% within each dimension | ICSE-18 distinguishes authority and economic risk; extend beyond employment to own-use and unpaid work. **High for the distinction, medium for historical mapping.** [UNSD](https://unstats.un.org/unsd/classifications/Family/Detail/2098) |
| **Coercive labor control** | Person-hours under specified exit restrictions, compulsory service, debt bondage, or ownership-like control of persons | 0–100%; accompanying legal/enforcement flags | **Proposed multidimensional coding**. Payment and coercion must remain separate variables. |
| **Obligatory appropriation** | Actual rent, tax, service, and other compulsory transfers relative to positive household or unit output | Percentage; labor days/year | **Case-specific measurement**. Claims or payments financed from stocks can exceed current output; do not mechanically cap every ratio at 100%. |
| **Market dependence** | Household need for purchased necessities and production-unit need for sales revenue to continue operating | Shares; days of independent provisioning | Marxian-inspired operationalization; **medium conceptual, low historical calibration confidence**. [Marxists Internet Archive](https://www.marxists.org/archive/marx/works/1867-c1/ch06.htm) |
| **Coordination institutions** | Bargaining coverage, training arrangements, finance relationships, interfirm agreements, and employee participation | Percentages and categorical variables | VoC and labor-relations research; **medium to high for modern measurement**, limited ancient applicability. [OUP Academic](https://academic.oup.com/book/301/chapter/134893294) |
| **Budget softness** | Expected rescue conditional on financial distress; separately record actual rescue frequency | Probability 0–1; events/year | Expectation is theoretically central but often latent; **medium conceptual, low direct historical measurement confidence**. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2F002205103771799999) |

Holder types should include households, outside investors, worker collectives, kin or community organizations, religious institutions, guild-like corporations, and public bodies. **Foreign location is a separate attribute**, not an ownership category competing with “private” or “state.”

### 2.2 Denominators that prevent false classifications

**Do not weight ownership by the number of organizations.** Hundreds of household workshops can coexist with a single organization controlling most productive capacity. Report land, labor, capacity, and output-weighted measures separately.

Where prices do not exist, use physical sector measures or a documented reference-value system. Revalue structural indicators consistently: a grain-price spike should not, by itself, appear to transfer ownership toward agriculture.

**Separate stocks from flows.** Existing asset control differs from control over new investment. A government might own inherited enterprises while private actors determine most new capacity—or the reverse.

**Separate stages of allocation.** Grain collected as tax, stored centrally, rationed to a worker, and subsequently exchanged must not count as four units of final provisioning. Maintain different accounts for production decisions, intermediate transfers, and final household acquisition.

**Do not force work into one permanent status per person.** Someone can cultivate a household plot, perform compulsory service, and work for wages in the same year. Payment method, dependence, tenure, and coercion are overlapping dimensions.

### 2.3 Empirical anchors for calibration and testing

These observations constrain the classifier; they are **not cutoffs defining systems**.

| Observation | Quantitative value and scope | What it tests | Confidence and qualification |
| --- | --- | --- | --- |
| Global informal employment | **61.2%** of employment in **2016**; **50.5%** excluding agriculture | A large nonformal economy can coexist with markets and wage employment | Official harmonized estimates; **medium–high within their statistical framework**, not a current estimate. [International Labour Organization](https://www.ilo.org/sites/default/files/2024-04/Women_men_informal_economy_statistical_picture.pdf) |
| Regional informal employment | **85.8% Africa; 68.6% Arab States; 68.2% Asia–Pacific; 40.0% Americas; 25.1% Europe–Central Asia**, 2016 | “Modern economy” must not imply uniformly formal employment | Same source and reference year; substantial within-region variation. [International Labour Organization](https://www.ilo.org/sites/default/files/2024-04/Women_men_informal_economy_statistical_picture.pdf) |
| Informality among employees | **39.7%** of employees globally were informally employed, 2016 | Employee status must not automatically imply formal employment | **Medium–high** for the reported aggregate. [International Labour Organization](https://www.ilo.org/sites/default/files/2024-04/Women_men_informal_economy_statistical_picture.pdf) |
| Netherlands labor coordination | Union density **15.4%** in 2019, versus adjusted bargaining coverage **75.6%** | Union membership is not interchangeable with bargaining coverage | **High** for recorded indicators; denominators differ: employees versus employees entitled to bargain. [OECD](https://www.oecd.org/content/dam/oecd/en/publications/reports/2024/10/main-indicators-and-characteristics-of-collective-bargaining-2021-country-notes_4b5553c9/netherlands_9d87118c/ddce81b8-en.pdf) |
| United States slavery | **3,953,760 enslaved people** out of **31,443,321 residents** in 1860: approximately **12.6%**, calculated | A nationally minority institution can remain structurally important; national averaging must not erase it | **High** as reported census counts, subject to historical enumeration limits; population share is not labor-hour or output share. [Census.gov](https://www.census.gov/about/history/stories/monthly/2014/november-2014.html) |
| Asset inequality in a small-scale maritime economy | Lamalera boat-share ownership had an estimated **Gini of 0.47** in Smith and colleagues’ study | Sharing and collective activity do not imply equal ownership of every asset | **Medium**; local, asset-specific estimate, not an income Gini or complete wealth measure. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/) |

The most important calibration lesson is **non-equivalence**: ownership, employment status, informality, coordination, and inequality cannot substitute for one another.

---

## 3. Historical and regional variation

Treat these as comparative cases and boundary conditions, not a progression the simulation must follow.

| Setting | Relevant observed variation | Implication for TCE |
| --- | --- | --- |
| **Foraging and mixed small-scale economies** | Research spanning societies in Africa, South America, Indonesia, and the Torres Strait distinguishes embodied, relational, and material wealth. Sharing can coexist with individually held tools or unequal productive assets; sedentary resource-rich settings differ from highly mobile ones. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC2999363/) | Do not assign “primitive communism” from subsistence technology. Generate resource rights, mobility, sharing obligations, and status separately. |
| **Early farming in western Eurasia** | Bogaard, Fochesato, and Bowles analyze **90 archaeological site phases** and find greater inequality in land-limited than labor-limited farming settings. Their measures draw on houses, storage, and grave goods, not direct household-income records. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/farminginequality-nexus-new-insights-from-ancient-western-eurasia/8EFE3B8F5AFA07450F87E4E9B553A43E) | Farming should not automatically create landlords or a state. Land scarcity, productive assets, inheritance, and control institutions should mediate outcomes. |
| **Aztec Mesoamerica** | Intensive production, specialization, merchants, markets, commodity money, and tribute coexisted. This is a strong counterexample to classifying a preindustrial tributary polity as simply “nonmarket.” [Cambridge University Press](https://www.cambridge.org/core/elements/aztec-economy/9AF63462E22863211A226A3BA89B4B73) | Permit **tributary–market hybrids** and measure urban and rural allocation separately. |
| **Inka Andes** | D’Altroy and Earle distinguish staple finance from wealth finance and document large-scale state storage and mobilization in the Upper Mantaro Valley. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/pdf/10.1086/203249?download=true) | Model collection, storage, provisioning, and political obligations directly. Large public granaries do not justify importing a complete Soviet-style enterprise model. |
| **Medieval Europe and precolonial India** | Historians disagree about how far “feudalism” identifies comparable institutions across places. European legal terminology and Indian land grants or intermediaries do not establish identical production relations. [OUP Academic](https://academic.oup.com/book/47420) | Prefer “lordly jurisdiction,” “hereditary dependency,” “land-based dues,” and “intermediated revenue collection” as first-order observations. |
| **Industrial-era United States** | The 1860 population still included nearly four million enslaved people. An industrial-era date therefore cannot substitute for observing labor institutions. [Census.gov](https://www.census.gov/about/history/stories/monthly/2014/november-2014.html) | Technology, trade, and labor freedom must be independent state variables. |
| **Twentieth-century state-socialist reform** | Kornai distinguishes bureaucratic appointment from worker self-management, including Yugoslavia. Lau, Qian, and Roland analyze China’s coexistence of planned obligations and market transactions during dual-track reform. [OUP Academic](https://academic.oup.com/book/4729/chapter/146977356) | “Public,” “planned,” and “worker-controlled” need separate indicators. Transitional rules can operate on different portions of the same unit’s output. |
| **Modern capitalist and mixed economies** | VoC research differentiates coordination institutions; Schneider identifies a distinct Latin American hierarchical configuration. Xu’s historical account of Chinese reform emphasizes centralized personnel authority alongside decentralized regional economic responsibilities. [OUP Academic](https://academic.oup.com/book/301/chapter/134893294) | Avoid a Europe-centered liberal/coordinated binary. Record corporate hierarchy, public influence, regional discretion, and coordination separately. |

A settlement’s relationship to external authorities also matters. TCE should be able to describe **a market town inside a tributary empire** or **a worker-managed enterprise inside a politically centralized state** without forcing the local and polity classifications to match.

---

## 4. Stylized facts a correct simulation should reproduce

### Markets coexist with nonmarket institutions

The Aztec case demonstrates that markets and tribute can be substantial parts of the same economy. The relevant test is not whether TCE produces perfectly pure types, but whether different allocation mechanisms can persist together without being treated as temporary errors. [Cambridge University Press](https://www.cambridge.org/core/elements/aztec-economy/9AF63462E22863211A226A3BA89B4B73)

**Simulation test:** introducing a market must not automatically abolish household production, communal resource rights, or obligatory transfers.

### Collective ownership and open access are different

A group can hold exclusion and management rights even when no individual can sell the entire resource. Schlager and Ostrom explicitly separate bundles of rights rather than equating all nonprivate resources with unrestricted access. [ISU Sites](https://faculty.sites.iastate.edu/tesfatsi/archive/tesfatsi/PropertyRightsRegimesNaturalResources.SchlagerOstrom1992.pdf)

**Simulation test:** communal pasture with enforceable access rules should behave differently from pasture that nobody can effectively exclude others from using.

### The same visible outcome can have different causes

Shortages may be associated with the institutional mechanisms Kornai analyzes, but his explanation concerns a configuration of authority, incentives, and coordination—not a rule that any shortage identifies socialism. [OUP Academic](https://academic.oup.com/book/4729/chapter/146973120)

**Simulation test:** drought, siege, transport failure, and administrative misallocation can all create queues without producing the same system label.

### Institutional dimensions are only partially correlated

Informal employment can occur among employees, and bargaining coverage can greatly exceed union membership. These measured gaps show why correlated indicators should not be treated as synonyms. [International Labour Organization](https://www.ilo.org/sites/default/files/2024-04/Women_men_informal_economy_statistical_picture.pdf)

**Simulation test:** changing formal registration should not automatically change who directs work; changing union membership should not mechanically determine how many workers an agreement covers.

### Hybrids can be stable; transition requires temporal evidence

Dual-track reform demonstrates simultaneous operation of old obligations and new market opportunities. The associated model’s favorable welfare result depends on conditions including enforcement of the old track; it is not proof that every partial reform benefits everyone. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/262113)

**Simulation test:** a mixed configuration persisting for fifty years should be labeled a stable hybrid. Reserve “transitioning” for observed directional changes in rights, allocation, and organizational behavior.

---

## 5. Recommended classifier and agent representation

### 5.1 Use several classification layers

I recommend four outputs:

**Structural profile.** Publish the measured ownership, control, allocation, labor, and appropriation indicators.

**Integration descriptors.** Identify householding, reciprocity, redistribution, and market integration without requiring one winner.

**Production-system descriptors.** Apply cautious labels such as seigneurial agrarian, capitalist market, cooperative market, or administratively coordinated.

**Modifiers.** Add coercive labor, informality, concentrated ownership, soft budgets, strong collective bargaining, regional decentralization, and external dependence.

Political regime and technological capability should appear alongside this classification, not be silently inferred from it.

### 5.2 Candidate label prototypes

These are **proposed TCE operational definitions**, not claims that scholarship has agreed on exact boundaries.

| Label | Essential evidence | Evidence that is insufficient by itself |
| --- | --- | --- |
| **Household-provisioning economy** | Households retain a major role in organizing production and meeting needs directly | Low income, farming, or absence of coins |
| **Reciprocity-centered economy** | Durable relationship-based obligations organize a substantial part of provisioning | Occasional gifts or charity |
| **Tributary–redistributive economy** | Enforced transfers to political or religious centers materially organize production or provisioning | Any taxation whatsoever |
| **Seigneurial/feudal agrarian economy** | Land-linked appropriation combined with dependent tenure or personal obligations and consequential lordly jurisdiction | Private land ownership, inequality, or voluntary rent alone |
| **Market-oriented household-producer economy** | Households or small producer units retain productive control while selling substantial output | The existence of markets alone does not distinguish it from capitalism |
| **Capitalist market economy** | Investor or proprietor control, residual claims, market-dependent enterprise reproduction, and consequential dependence on hired labor | Merchants, currency, or private household property alone |
| **Administratively coordinated / command economy** | Binding cross-enterprise authority substantially determines production, inputs, and investment | Public ownership, high taxes, or internal management within ordinary firms |
| **Cooperative market economy** | Workers or members effectively govern production and residual income while units substantially coordinate through markets | A cooperative legal title without effective member control |
| **State-led market economy** | Public control or directed investment is substantial, but production and exchange remain materially decentralized through markets | A few public enterprises or isolated industrial subsidies |

Treat **slavery and other coercive labor systems as both explicit presence flags and scale-sensitive structural descriptors**. A majority threshold must never make them disappear. An economy can have a commercially important slave-based sector without most residents being enslaved.

Likewise, assess strategic control separately from bulk shares. Control over a small number of grain stores, credit institutions, ports, or transport links may confer influence disproportionate to their asset count.

### 5.3 Prefer fuzzy rules over a forced one-class model

Begin with an explainable rule system. Historical labels are too contested to justify treating a small hand-labeled dataset as objective ground truth.

For a continuous feature, use a transparent membership function:

\[
h(x;a,b)=
\operatorname{clip}\left(\frac{x-a}{b-a},0,1\right).
\]

For label \(k\), calculate a weighted match:

\[
S\_k =
G\_k\,
\frac{\sum\_{j\in\text{observed}} w\_{kj}m\_{kj}(x\_j)}
{\sum\_{j\in\text{observed}} w\_{kj}}.
\]

Here, \(m\_{kj}\) measures compatibility with the label, and \(G\_k\) represents essential conditions. For example, absence of lordly jurisdiction should block the proposed seigneurial prototype even if land ownership is unequal.

**Unknown is not false.** Missing an essential observation should produce “insufficient evidence,” not a zero score. Report evidence coverage separately.

**Scores are resemblance scores, not probabilities.** A score of 0.8 does not mean an 80% historical probability of capitalism.

### 5.4 Initial numerical settings

All values in this table are **engineering defaults proposed here, with low empirical calibration confidence**. They should be varied during testing rather than presented as historical findings.

| Setting | Suggested starting value | Sensitivity range | Purpose |
| --- | --- | --- | --- |
| Flow-accounting window | 1 agricultural/calendar year | 1–3 years | Avoid classifying harvest-season trade as the permanent structure |
| Structural smoothing horizon | 5 years | 3–10 years | Reduce label oscillation |
| Fuzzy “substantial/dominant” ramp | Membership rises from 0 at 40% to 1 at 70% | Shift endpoints by ±10 percentage points | Soft defaults for appropriate share indicators—not universal gates |
| Label entry / exit scores | 0.70 / 0.50 | Test alternatives | Hysteresis |
| Minimum weighted evidence coverage | 0.80 | 0.70–0.95 | Encourage abstention when evidence is incomplete |
| Confirmation persistence | 3 annual reviews | 2–5 | Distinguish durable structure from noise |
| Close-match margin within one family | 0.10 | 0.05–0.20 | Return multiple plausible labels instead of an arbitrary winner |

Do not apply slow smoothing blindly to revolutions or conquest. Show an immediate **provisional institutional change**, then confirm the structural effect as the new rules are enforced.

#### Worked example: public ownership without full command

Consider a **synthetic** economy with:

* 60% of productive capacity under public control;
* 80% of output allocated through decentralized markets;
* 60% of new investment selected administratively.

The proposed description is:

> **State-led market economy with administratively directed investment.**

Public ownership alone does not warrant “command economy.” Conversely, direct administrative assignment can be extensive even where legal ownership remains mixed.

The UI should explain the classification with measured statements:

> “Public bodies control most productive capacity, but enterprises usually sell output through markets.”

### 5.5 Represent institutions through rights, obligations, and decisions

A compact entity model is sufficient:

| Entity | Essential state |
| --- | --- |
| **Person** | Household membership, work relationships, skills, legal status, obligations, available exit options |
| **Household** | Members, consumption requirements, buffers, productive assets, reciprocal relationships |
| **Production unit** | Inputs, outputs, decision procedures, workers, manager, residual claimants |
| **Asset / resource** | Physical characteristics and separate holders of access, extraction, management, exclusion, transfer, and income rights |
| **Institution** | Membership, offices, rules, decision procedure, budget, enforcement capacity |
| **Contract / obligation** | Parties, required action or transfer, conditions, duration, sanctions, inheritance or transfer rules |
| **Economic event** | Who decided, who produced, who transferred, who received, quantity, mechanism, and enforcing institution |

Crawford and Ostrom’s institutional grammar is useful inspiration: express rules through actors, permitted or required actions, conditions, and consequences. This allows a temple, guild, cooperative, landlord, and ministry to use the same underlying machinery without behaving identically. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/grammar-of-institutions/7D37CD3BC5ED2D9FD57D2EE292958F47)

For 10k–50k people, maintain **incremental counters from economic events** rather than repeatedly scanning complete histories. Update individual activity at the simulation’s normal cadence, financial and contractual summaries less frequently, and structural classifications annually or after major institutional events.

Classification then costs approximately \(O(KD)\) per aggregate for \(K\) prototypes and \(D\) features. That is an algorithmic estimate, not a performance benchmark. Settlement and sector summaries can feed polity summaries without all-to-all agent comparisons.

Unreal should display the resulting snapshots; building appearance should not determine institutional type.

### 5.6 Existing models and games worth borrowing from

| Model or game | Reusable component | Limitation for TCE |
| --- | --- | --- |
| **Bowles–Choi institutional coevolution model** | Interaction between production choices and property institutions | A focused explanation of agricultural origins, not a complete societal simulator. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/701789?mobileUi=0) |
| **Eurace@Unibi** | Households, firms, finance, labor and goods markets, spatial structure, asynchronous decisions, consistent accounting | Designed around a modern market economy; institutions would need generalization. [Yildizoglu](https://yildizoglu.fr/macroabm1/docs/vanderHoog-Eurace.pdf) |
| **Sugarscape / *Growing Artificial Societies*** | A precedent for investigating macro-level patterns generated by interacting agents | Useful methodological precedent, not a validated classifier of historical economic systems. [MIT Press](https://mitpress.mit.edu/9780262550260/growing-artificial-societies/) |
| **Victoria 3’s documented 1.7 ownership redesign** | Separation of production buildings from their owners; mixed ownership and allocation of dividends | Borrow the separation of ownership from production, not an assumption that its predefined laws and categories constitute historical ground truth. This refers to the documented 2024 design, not a claim about the latest version. [Reddit](https://www.reddit.com/r/victoria3/comments/1bpxpax/victoria_3_dev_diary_110_building_ownership/) |

### 5.7 Validation priorities

Build **paired counterfactual tests**: change one institution while holding the others constant.

A nationalization decree without effective control should change legal-title indicators before operational-control indicators. A welfare expansion should not automatically turn a market economy into a command economy. A private factory’s internal production schedule should not classify the surrounding society as centrally planned. Paid compulsory labor should not become free employment merely because money changes hands.

For historical validation, use expert-reviewed case descriptions with multiple acceptable labels. Hold out entire regions or historical settings during tuning. Measure whether explanations remain correct when thresholds shift—not merely whether the classifier reproduces one annotator’s preferred terminology.

---

## 6. Sources, datasets, and limits of the evidence

### 6.1 Datasets useful for calibration

| Resource | Best use in TCE | Important limitation |
| --- | --- | --- |
| **D-PLACE / Ethnographic Atlas and associated datasets** | Comparative coding of subsistence, property, inheritance, social organization, and related institutions | Observations have specific dates and locations, often from the nineteenth and twentieth centuries. They are not direct observations of prehistoric societies; cultural and linguistic relatedness also complicates independence assumptions. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) |
| **Seshat Global History Databank** | Historical political organization, administrative complexity, and institutional context | The influential 2018 analysis covered **414 societies in 30 regions over 10,000 years**, but this is not a labeled training set of economic modes. Code uncertainty rather than treating absent evidence as institutional absence. [PubMed](https://pubmed.ncbi.nlm.nih.gov/29269395/) |
| **ILOSTAT and ICSE-18** | Employment relationships, authority, risk, and informality | Employment categories do not by themselves cover all own-use production, unpaid activity, or coercive relationships. [UNSD](https://unstats.un.org/unsd/classifications/Family/Detail/2098) |
| **OECD/AIAS ICTWSS** | Bargaining coverage, unionization, bargaining levels, and coordination institutions | Mainly useful for the modern labor-relations layer, not for assigning an all-era economic-system label. [Statistiques Luxembourg](https://statistiques.public.lu/dam-assets/fr/actualites/semeco/collective-bargaining-in-a-changing-world-of-work.pdf?utm_source=chatgpt.com) |
| **World Bank, *The Business of the State* database/report** | Modern state participation and corporate ownership networks | The 2023 report covers about **76,000 companies in 91 countries**, using **more than 10% government ownership** as an inclusion criterion. That cutoff is not a universal definition of effective state control. [World Bank](https://www.worldbank.org/en/publication/business-of-the-state) |
| **Historical censuses, estate records, contracts, and institutional accounts** | Case-specific labor status, dues, title, and allocation rules | Different sources observe different parts of the system. Legal obligations, recorded claims, and actual extraction need separate treatment. |

### 6.2 Scholarly foundations and contested claims

The core reading set is Polanyi’s *The Great Transformation* and “The Economy as Instituted Process”; Marx’s 1859 preface and *Capital*’s analysis of labor-power; Kornai’s *The Socialist System* and work on soft budgets; Hall and Soskice’s *Varieties of Capitalism*; and Schlager and Ostrom on property-rights bundles. Together they support a multidimensional representation, but they do not supply a single shared classifier. [INCT/PPED](https://www.inctpped.org/spiderweb/pdf_4/Great_Transformation.pdf)

Three cautions should remain visible in TCE’s research documentation.

**Historical labels are contested.** “Feudalism,” “socialism,” “state capitalism,” and Marxian modes encode different theoretical commitments. Marx’s own list of modes is not evidence that every region followed the same sequence. Avoid geographic labels such as “Asiatic mode” as automatic consequences of location or irrigation. [Marxists Internet Archive](https://www.marxists.org/archive/marx/works/1859/critique-pol-economy/preface.htm)

**Proxy measurements are not interchangeable.** Archaeological house-size inequality is not income inequality. State shareholding is not necessarily operational control. Labor status in a census is not a complete measure of working time or coercion. Keep source definitions attached to every calibration target. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/farminginequality-nexus-new-insights-from-ancient-western-eurasia/8EFE3B8F5AFA07450F87E4E9B553A43E)

**Causal explanations are not settled universal laws.** Property–farming coevolution, institutional complementarities, and the conditions under which partial reforms succeed are useful models to implement and test—not historical scripts to enforce. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086/701789?mobileUi=0)

### Bottom line

For TCE, the strongest architecture is:

**Individual actions and enforceable rules → rights and obligation networks → measured structural features → qualified, multi-layer labels.**

Author the rights, contracts, decision procedures, and enforcement mechanisms. Let households and institutions produce the economy. Then let the UI describe what exists—including configurations that no familiar historical label captures cleanly.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928ad-2ee8-83e9-880d-ad6daf83e03a)
