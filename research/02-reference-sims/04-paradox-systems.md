# Grand-strategy models: what TCE should borrow—and what it should not

**The strongest combination for TCE is Victoria 3’s connection between material conditions and political power, Crusader Kings 3’s persistent people and obligations, and Europa Universalis’s legible institutional choices.** The missing ingredient is a general mechanism through which agents create, contest, administer, and revise those institutions.

My recommendation is to make **institutions executable arrangements among people**, not bundles of national modifiers. A law should determine who may act, who owes what, who decides disputes, and how decisions become effective. Its economic and political consequences should then follow through the ordinary simulation.

This report distinguishes original designs from subsequent revisions. In particular, Victoria 3’s original trade system and law-enactment system are not adequate descriptions of all later versions: trade was redesigned for 1.9, and 1.12 added law amendments. CK3’s schemes were substantially redesigned for 1.13. The examples below are dated mechanisms and documented failures, not claims that every historical problem remains unresolved in September 2026. [Steam Community](https://steamcommunity.com/ogg/529340/announcements/detail/524212840420081890)

## 1. How the models work

### 1.1 Victoria 3: an economy that changes the constituency for political change

#### Pops are cohorts, not persistent individual citizens

Victoria 3’s population model aggregates people into *pops*. Economic and demographic changes alter these groups, rather than simulating each represented person’s daily activities. Importantly, a pop is **not politically unanimous**: its members can support different interest groups, or remain politically inactive. Profession, wealth, literacy, and other characteristics influence those distributions. [Steam Store](https://store.steampowered.com/news/posts/?enddate=1625155596&feed=steam_community_announcements)

The standard-of-living model provides a particularly useful abstraction. Wealth determines a consumption basket; income relative to its cost pushes wealth upward or downward over time. Wealth is therefore not simply a bank balance. Needs are expressed in categories with substitute goods: heating, for example, can be satisfied by several fuels. Cultural preferences affect substitution. Richer people acquire additional needs, and goods serving both ordinary and luxury consumption connect rich households’ demand to poorer households’ expenses. Standard of living also affects demographic and political outcomes. The original documented scale was **1–99**. [Steam Community](https://steamcommunity.com/games/529340/announcements/detail/2965045886379003737)

**Design payoff:** economic development changes people’s lives and, through those lives, the political environment. A policy does not need an independently scripted “creates discontent” event when it already changes food affordability, employment, or distribution.

**Limit for TCE:** a cohort’s average wealth cannot replace individual inventories, household obligations, hunger, or travel. Adopt the needs taxonomy and substitution logic, not the loss of personal continuity.

#### Production, employment, and ownership are different relationships

The 1.7 ownership redesign is especially instructive. Ownership became associated with individual **building levels**, allowing mixed ownership within a building. Owners can be workers, private investors, or the state. Financial Districts and Manor Houses hold ownership interests and receive dividends from establishments elsewhere; foreign investment extends this separation across countries. State construction and subsequent privatization become distinct operations. [Paradox Plaza Admin Forum](https://admin-forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-110-building-ownership-foreign-investment.1647879/page-6)

This produces a valuable separation:

> Where production occurs ≠ where owners reside ≠ where the resulting political influence is concentrated.

For TCE, the equivalent is not necessarily nineteenth-century corporate finance. A mill could be worked by one household, owned by a temple, maintained through village obligations, and subject to a noble’s toll rights. Those should be distinct records.

#### “Market clearing” is chiefly price formation, not physical inventory matching

Victoria 3’s market design uses buy and sell orders as aggregate economic flows. Do not interpret the market interface as evidence that every demanded unit has been matched to a particular existing item and transferred between inventories. The original developer discussion explicitly rejected military-goods stockpiles in favor of production, trade access, and financial preparation. [Reddit](https://www.reddit.com/r/victoria3/comments/oylfn3/victoria_3_dev_diary_9_national_markets/)

A useful numerical illustration is a **2022 community reconstruction**, checked against developer footage but explicitly identified by its author as not developer-confirmed:

\[
P=P\_0\left[1+0.75\,\operatorname{clamp}
\left(\frac{B-S}{\min(B,S)},-1,1\right)\right],
\qquad B,S>0
\]

Here \(B\) and \(S\) are buy and sell orders. This gives a price range of **25%–175% of base price**. Under that reconstruction, 120 buy orders against 100 sell orders produce 115% of base price; 200 against 100 reach the upper bound. These are historical model figures, not a verification of the current executable’s exact implementation. [Reddit](https://www.reddit.com/r/victoria3/comments/y7nzrb/economic_efficiency_goods_price_formula_pt_2/)

The architectural distinction matters more than the formula: **a price signal can represent scarcity without constituting a conserved-goods allocation algorithm**.

For TCE, especially an agrarian TCE, this shortcut is dangerous. Grain carried over from harvest, food trapped behind a destroyed bridge, seed reserved for planting, and spoilage in storage must remain quantities that exist—or do not exist.

#### Local prices exposed weaknesses in the original trade design

Victoria 3’s developers described the old, manually managed trade routes as unreliable, fiddly, and increasingly ill-suited to the local-pricing system introduced in 1.5. The 1.9 redesign introduced a **World Market above national markets**. Trade Centers automatically adjust imports and exports for profitability, using trade capacity; they transact between world and local markets. Merchant Marine became a trade input, separating commercial shipping from the earlier convoy arrangement. Government-directed strategic trade remained a different concern from autonomous commercial activity. [Reddit](https://www.reddit.com/r/victoria3/comments/1jl5yhi/victoria_3_dev_diary_143_trade_rework_the_world/)

This is a general integration lesson: **adding spatial detail to one subsystem can invalidate another subsystem’s decision model**. A trade AI optimizing an old national price is not repaired merely by drawing local prices in the interface.

#### Interest groups translate social position into political power

Interest groups aggregate support into political organizations with authored ideologies and leaders. Wealth, status, and voting arrangements determine political strength; **clout is a group’s share of total political strength**, not its share of population. Approval is separate from clout, and the combination affects political behavior and group bonuses or penalties. Leader ideology can alter a group’s position. [Steam Store](https://store.steampowered.com/news/posts/?enddate=1625155596&feed=steam_community_announcements)

Political parties provide another aggregation layer: interest groups can organize into parties within electoral politics. Government composition and legitimacy therefore operate through organizations and electoral arrangements, not through a simulated parliament of individually deliberating citizens. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-46-political-parties)

The productive feedback loop is:

**Employment and ownership → income and status → political strength → laws → employment and ownership.**

For TCE, this is more useful than assigning every farmer an immutable “farmer ideology.” Preserve material interests, but let actual organizations form around combinations of occupation, locality, kinship, religion, grievance, and leadership.

#### Laws are authored configurations, with increasing room for amendments

Victoria 3’s basic law model selects among authored alternatives within law groups. The 1.3 enactment redesign made passage a staged process involving success, advancement, debate, and stalling, rather than a single favorable roll. Interest-group support and opposition influence that process; political movements and government petitions create additional pressure. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-80-law-enactment-and-revolution-clock-in-13)

Institutions such as education and policing are administrative systems enabled and shaped by laws, with bureaucracy acting as an implementation resource. This is more substantial than a cost-free policy toggle, but still much more aggregated than simulating teachers, classrooms, patrols, and casework. [Reddit](https://www.reddit.com/r/victoria3/comments/ou0b0n/victoria_3_dev_diary_8_institutions/)

**The important modern qualification is amendments.** The December 2025 1.12 patch documents amendments that alter existing laws: examples include restrictions on electoral manipulation, historical limits on government reform, and country-specific acceptance exceptions. Amendments can also participate in unlocking or disallowing production methods and decrees. Thus, “Victoria 3 laws are only mutually exclusive enum values” is now too crude. The better description is **authored base laws plus authored exceptions and modifications**. [SteamDB](https://steamdb.info/patchnotes/21114983/)

That is a meaningful step toward composability—but not yet a general grammar in which agents invent a new office, assign its jurisdiction, negotiate its funding, and establish its appeal procedure.

### 1.2 Crusader Kings 3: politics as persistent relationships and claims

#### The economic model serves the personal political model

CK3’s distinctive detailed entities are characters, titles, dynasties, and relationships—not a population of individually employed households. In the traditional feudal model, an important economic mechanism is the **liege–vassal contract**: tax and levy obligations can vary independently, alongside special rights and protections. Negotiation, hooks, opinion, and tyranny constrain changes. [Strategie Zone](https://www.strategie-zone.de/forum/viewtopic.php?t=24473)

For TCE, this suggests modeling economic extraction as a relationship. “The government receives 10% tax” conceals the important questions: who owes it, under which tenure, to which office, collected by whom, and with what exemptions?

This description concerns CK3’s established feudal framework; it should not be generalized unchanged to every government type added later.

#### Succession changes control without erasing the people who lost

CK3 distinguishes gender eligibility, realm succession, and title-specific succession rules. The resulting inheritance is therefore a composition of rules rather than a single universal “next ruler” function. [Crusader Kings III Wiki](https://ck3.paradoxwikis.com/Succession_laws)

Partition divides holdings among eligible heirs. Confederate Partition can create additional titles during succession, whereas ordinary Partition does not create those extra titles. A title-specific elective arrangement can change who receives that title, producing interactions with the realm’s broader inheritance rules. Community succession guides consequently focus on title structure and heir eligibility as much as on the displayed primary heir. [Reddit](https://www.reddit.com/r/CrusaderKings/comments/16ccavt/guide_how_to_handle_succession_with_confederate/)

**What works:** succession redistributes real control among persistent people. A sibling who did not inherit the primary title does not disappear; family relationships and rival claims can continue to matter.

**What needs adaptation:** players can manipulate title arrangements and heir eligibility to optimize inheritance. For TCE, such manipulation should be a possible political strategy, but it should incur the relevant relationship, legitimacy, enforcement, and opportunity costs—not merely satisfy a convenient technical condition.

#### Factions are organized threats, not simply low happiness

In CK3’s familiar vassal politics, factions have concrete objectives: reducing authority, obtaining independence, or installing a claimant; dissolution factions add another form of attack on the existing realm. Their danger depends on collective strength and accumulating discontent, not merely on the number of unhappy characters. The documented standard strength threshold is **80% of the liege’s strength**, above which discontent grows, subject to the relevant rules and modifiers. [Crusader Kings III Wiki](https://ck3.paradoxwikis.com/index.php?mobileaction=toggle_view_desktop&redirect=no&title=Factions)

The transferable principle is excellent: **grievance, organization, and capacity are separate variables**.

The legibility problem is equally important. The original *Factions Explained* mod exposed why individual characters belonged to factions. Its author later removed that explanatory feature because equivalent functionality had entered the base game. This is evidence of a genuine interface need: players require reasons, not just an apparently inexplicable coalition. [GitHub](https://github.com/amtep/ck3-factions-explained)

#### Schemes turn relationships into capabilities—but underwent a major redesign

The 2024 scheme redesign addressed an old pattern in which a scheme was either highly reliable or unattractive, while skill tended to improve speed, success, and secrecy together. The revised system separates preparation from execution: phases accumulate advantages, agents occupy roles, and the player chooses when to attempt execution. Different agents can improve different dimensions; recruitment can involve money, hooks, or other concessions. Exposure accumulates through breaches. [Steam Store](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1727356181&feed=steam_community_announcements)

The developers also reported unsuccessful prototypes. Randomized agent-selection events created excessive micromanagement, and extending the elaborate system to simple schemes was not worthwhile. Basic schemes therefore retained simpler treatment. [Steam Store](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1727356181&feed=steam_community_announcements)

For TCE, the useful model is not “every conspiracy needs an elaborate minigame.” It is **an action requiring access, collaborators, preparation, and exposure to detection**. A servant with access to a storeroom should enable a different plan from a distant supporter with no physical opportunity.

#### Character AI is selective, not continuous deliberation by everyone

CK3’s performance documentation describes many authored AI actions, evaluated at different frequencies according to character importance and action type. Candidate filtering is central: the engine must decide whom to consider before performing expensive evaluations. This is not evidence of every character continuously solving a general long-horizon plan. [store.steampowered.com](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1761307139&feed=steam_community_announcements)

TCE should copy that computational selectivity while preserving the causal importance of ordinary people. A farmer need not reevaluate constitutional reform every hour; the farmer still must remain hungry, indebted, related to someone, and capable of joining a petition.

### 1.3 Europa Universalis IV: state capacity and institutional choices

EU4 principally treats the **state as the enduring decision-making entity**. Rulers change without necessarily changing the player’s objectives or institutional continuity. Province development summarizes economic and manpower capacity, while administrative, diplomatic, and military power provide abstract constraints on development and state action. Historian Bret Devereaux’s firsthand analysis emphasizes the corresponding omission: households and non-state organizations have much less independent agency than states do. [A Collection of Unmitigated Pedantry](https://acoup.blog/2021/04/30/collections-teaching-paradox-europa-univeralis-iv-part-i-state-of-play/)

Its trade system is a different abstraction from Victoria 3’s markets. Trade value moves along a predefined directional network, and control of favorable collection points can produce large revenues. Devereaux identifies how fixed downstream geography advantages particular trade positions, and how a human player can exploit the network more effectively than AI states. This is strategic geography, not a general model of goods flowing toward changing local demand. [A Collection of Unmitigated Pedantry](https://acoup.blog/2021/05/14/collections-teaching-paradox-europa-universalis-iv-part-iii-europa-provincalis/)

EU4’s institutional vocabulary—government configurations, ideas, policies, and estate arrangements—is useful as an **authoring and explanation vocabulary**. The central limitation for TCE is the locus of agency: these are predominantly configurations of state capabilities, not the output of a society of individually simulated legislators, collectors, judges, creditors, and constituents. [A Collection of Unmitigated Pedantry](https://acoup.blog/2021/04/30/collections-teaching-paradox-europa-univeralis-iv-part-i-state-of-play/)

A particularly instructive balance failure was **Concentrate Development**. The developers explicitly called the original feature unbalanced and immersion-breaking despite working as designed. The 1.32 redesign converted removed development into a monarch-point value, then used that value to purchase development at the capital’s costs, with applicable losses. It also addressed double concentration through ownership followed by release as a vassal, and increased subjects’ liberty desire. [Steam Store](https://store.steampowered.com/news/posts/?appids=236850%2C231740%2C210908%2C202270%2C225420%2C210904%2C210905%2C226660%2C203770%2C204080%2C218130%2C227760%2C202130%2C48700%2C25800%2C42990%2C22130%2C230940%2C73170%2C42910%2C48720%2C200370%2C22100%2C25890%252&enddate=1632829306)

**Lesson:** moving the same abstract “development point” between locations ignored differences in marginal cost. For TCE, moving people, capital, or productive capacity must preserve the associated costs and losses rather than transfer a generic bonus.

### 1.4 Europa Universalis V: a materially closer comparison

EU5 should not be conflated with EU4. Its published design introduces a much more population-centered economy and greater logistical detail. [Paradox Interactive](https://www.paradoxinteractive.com/games/europa-universalis-v/about)

In Devereaux’s firsthand launch-era examination, buildings require appropriate workers; population needs connect to dissatisfaction and political conditions; and **control** distinguishes nominal territory from the resources a government can actually extract. Control varies with accessibility from the capital, making locally effective vassals potentially more useful than ineffective direct rule. Estate privileges provide benefits as well as political constraints, creating reasons for societies to remain in locally stable arrangements. [A Collection of Unmitigated Pedantry](https://acoup.blog/2025/10/31/miscellania-europa-universalis-v-confirmed-first-impressions/)

His observed trade-automation limitation is particularly relevant: profitable trade in an intermediate good may look unattractive when the major return occurs farther down a production chain. The player could intervene, but the example exposes the difference between **a trader’s immediate margin and a production network’s total value**. This is a launch-era observation, not proof of the current AI’s behavior. [A Collection of Unmitigated Pedantry](https://acoup.blog/2025/10/31/miscellania-europa-universalis-v-confirmed-first-impressions/)

For TCE, EU5’s control distinction is worth adopting. Its chronological ages and historical setup are not necessary.

## 2. What worked, what failed, and why

The strongest successes are the connections between systems: material change redistributes political influence; inheritance redistributes control among persistent relatives; distance limits effective government. The most instructive failures occur where an abstraction, incentive, or AI evaluation breaks one of those connections.

| Documented case | Specific problem or revision | Implication for TCE |
| --- | --- | --- |
| **Victoria 3, post-release plans, November 2022** | Developers identified AI failure to develop resources such as oil and rubber, alongside late-economy problems. Autonomous private construction was a planned improvement. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/victoria-3-dev-diary-64-Post-Release-Plans) | A resource is not economically available merely because a deposit exists. Someone must recognize the opportunity, finance it, obtain access, and build capacity. |
| **Victoria 3, law enactment, 1.3** | The original system produced implausible lucky passes, frustrating adverse-event spirals, and cancellation/restart incentives. The redesign introduced staged passage, setbacks, and stronger commitment costs. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-80-law-enactment-and-revolution-clock-in-13) | Uncertainty should represent uncertain actors or events. Repeated arbitrary rolls should not substitute for political procedure. |
| **Victoria 3, government membership exploit** | Bringing a group into government could neutralize its revolutionary pressure without delivering the policy it wanted. Government petitions were introduced to attach consequences to demands. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-80-law-enactment-and-revolution-clock-in-13) | Joining a coalition must not erase a grievance. Store promises, deadlines, counterparties, and fulfillment separately from membership. |
| **Victoria 3, local prices and trade** | Increased spatial detail made the old route-management model less effective, motivating the 1.9 rework. [Reddit](https://www.reddit.com/r/victoria3/comments/1jl5yhi/victoria_3_dev_diary_143_trade_rework_the_world/) | Whenever representation changes, audit every AI that reads it. “More realistic data” can make existing behavior worse. |
| **CK3, schemes, 1.13** | Highly correlated success dimensions encouraged near-certain schemes; more elaborate agent handling also risked excessive micromanagement. [Steam Store](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1727356181&feed=steam_community_announcements) | Separate access, speed, secrecy, and execution risk, but reserve expensive planning for consequential actions. |
| **CK3, 1.16 AI correction** | The published changelog, quoted in its discussion, acknowledged that AI had not been checking whether it could feudalize holdings. Players had interpreted the resulting behavior as strategic reluctance. [Reddit](https://www.reddit.com/r/CrusaderKings/comments/1k55olf/pc_dev_diary_170_changelog_achievements/) | Test action availability and invocation, not just utility scores. A plausible-looking preference can conceal an unreachable action. |
| **EU4, Concentrate Development, 1.32** | A feature could function correctly yet yield implausible concentration and exploitable transfers. The fix restored cost-sensitive conversion and political consequences. [Steam Store](https://store.steampowered.com/news/posts/?appids=236850%2C231740%2C210908%2C202270%2C225420%2C210904%2C210905%2C226660%2C203770%2C204080%2C218130%2C227760%2C202130%2C48700%2C25800%2C42990%2C22130%2C230940%2C73170%2C42910%2C48720%2C200370%2C22100%2C25890%252&enddate=1632829306) | Correct code does not guarantee a coherent model. Test repeated composition of legal actions. |

Community work is most useful when it identifies an observable mechanism. For example, the author of a Victoria 3 MAPI overhaul criticized the relationship between infrastructure and market integration, then changed the relevant parameters and infrastructure effects. That is a concrete alternative model to inspect—not independent proof that every vanilla outcome was a bug. [Nexus Mods](https://www.nexusmods.com/victoria3/mods/20)

A recurring distinction is worth preserving:

**Implementation bug:** the AI never invokes a valid action.  
**Incentive failure:** the AI rationally chooses behavior the designer dislikes.  
**Representation failure:** the available variables cannot express the relevant constraint.  
**Legibility failure:** sensible behavior occurs, but the observer cannot discover why.

These require different repairs.

## 3. Numbers and performance evidence

The most useful published engineering figures here come from **CK3’s October 2025 performance diary**, rather than from population totals advertised for the games.

| Figure | What it actually establishes |
| --- | --- |
| Approximately **30–40% more playable land and living characters** in the expansion under discussion | Content expansion increased both spatial and character workloads. |
| Tests over **150–250 simulated years**, beginning in 1066 | The team measured aging worlds, not only fresh starts. |
| Approximately **550 AI action types** | Action scheduling and eligibility checks are substantial systems in their own right. |
| Processing roughly **1/30 of eligible characters per day** for a relevant periodic workload | Monthly work can be amortized instead of synchronized into a large spike. |
| Filtering a succession candidate set from roughly **4,000 to 100**, with a reported **3× improvement** during optimization | Narrowing candidates can outweigh adding parallelism; this is a subsystem improvement, not a whole-game speedup. |

These figures come from one developer report and should not be treated as independent benchmarks. Its succession investigation also found a bad cache interaction: algorithmic filtering alone did not eliminate the remaining cost. [store.steampowered.com](https://store.steampowered.com/news/posts/?appids=1158310&enddate=1761307139&feed=steam_community_announcements)

Victoria 3’s 1.3 legislation supplies another concrete design figure: **three successful stages** to pass a law and **three setbacks** to terminate the attempt. That creates a more structured progression than one decisive roll, but the numbers themselves do not establish political realism. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-80-law-enactment-and-revolution-clock-in-13)

For TCE, the immediately relevant scaling calculation is your own: 50,000 agents considering every other agent means **2,499,950,000 directed pairs**. Restricting each agent to 20 relevant contacts produces **1,000,000 links**. This is arithmetic, not a measured performance claim, but it explains why bounded social neighborhoods and institutional candidate indexes are essential.

I did not find a credible apples-to-apples benchmark showing that these games’ workloads establish feasibility for **50,000 persistent people with visible daily life alongside UE5 rendering**. Cohort population totals and politically significant character counts are not that benchmark.

## 4. A concrete institutional design for TCE

The following is a proposed architecture, not a claim about Paradox’s internal implementation.

### 4.1 Separate institutions, organizations, offices, and laws

Use distinct entities:

| Entity | Purpose |
| --- | --- |
| **Organization** | Persistent membership, assets, treasury, relationships, and collective objectives: household, village, temple, guild, council. |
| **Office** | A position with eligibility, selection, term, jurisdiction, powers, duties, and succession. |
| **Rule** | A permission, prohibition, obligation, eligibility condition, or decision procedure. |
| **Commitment** | A promise or agreement with parties, conditions, deadlines, fulfillment, and breach consequences. |
| **Case or proceeding** | A particular application of rules: election, inheritance, tax assessment, appeal, petition, trial. |

This avoids confusing a government with its current officeholders or a political promise with an already enacted law.

The research literature provides a useful vocabulary. Institutional Grammar treats institutional statements as combinations of defined syntactic components; North’s formulation also explicitly includes informal constraints alongside formal rules. Neither is a ready-made game engine, but both are better foundations than treating “institution” as synonymous with “national bonus.” [arXiv](https://arxiv.org/abs/2008.08937)

### 4.2 Compose mechanisms, not just modifiers

An executable rule should answer:

**Who is covered? Under what conditions? Who may or must do what? Using which resources? Who recognizes, monitors, or enforces the result?**

A village’s grain-reserve arrangement could be authored from these parts:

```
Jurisdiction: village member households

Contribution:
  Each household owes a share of its recorded harvest.
  Seed grain is exempt.

Custody:
  Grain belongs to the village organization.
  A selected keeper controls access to the storehouse.

Distribution:
  Eligible households may request an emergency allocation.
  Allocations cannot exceed available inventory.

Accountability:
  Witnesses inspect the reserve after harvest.
  Proven diversion creates a restitution claim.

Office:
  The keeper serves a fixed term.
  The assembly may remove the keeper under a specified procedure.
```

The contribution share, seed exemption, eligibility test, selection method, and removal procedure are composable choices. The same primitives can produce a household reserve, temple granary, merchant warehouse association, or coercive lordly store.

Crucially, **the grain remains in the economy**. Collection requires transport and labor. The keeper can be absent, incompetent, dishonest, or threatened. A declared entitlement cannot feed somebody when the store is empty.

### 4.3 Give agents bounded ways to propose change

Do not ask every notable to search the entire space of constitutions. Generate proposals from experienced problems and existing examples.

A useful sequence is:

**Observed problem → candidate amendment → coalition building → bargaining → authorized decision → implementation → dispute and revision.**

For example, repeated theft may generate proposals for a second keeper, joint access, more frequent inspection, or harsher penalties. These are small changes with intelligible consequences.

Each proposer should evaluate alternatives using limited beliefs: expected household income, food security, autonomy, status, kinship interests, ideology, and commitments. An agent need not know the globally optimal arrangement. Different actors should disagree about both values and predicted consequences.

Equally, not every institution needs a legislature. A custom may gain force through repeated compliance and recognized precedent; an assembly may ratify it later. A ruler’s decree, village consensus, contract amendment, inheritance, and council vote should use different procedures—not different labels on one universal progress bar.

### 4.4 Keep four political quantities separate

**Legal validity:** was the decision made under recognized rules?  
**Implementation capacity:** are people, resources, information, and reach sufficient?  
**Compliance:** do affected agents actually obey?  
**Legitimacy:** do they regard the arrangement as acceptable or rightful?

These can diverge.

A lawful tax may be uncollectable in a distant hamlet. An unauthorized emergency granary opening may enjoy broad approval. A feared ruler may obtain obedience without legitimacy. A respected council may lack guards or reliable records.

This also prevents the familiar shortcut in which passing an education law instantly produces education. The rule should authorize or obligate provision; buildings, teachers, materials, household choices, and travel determine delivery.

### 4.5 Make ownership and succession transactional

Treat death, resignation, deposition, and organizational dissolution as explicit transitions.

An inheritance procedure should resolve assets, offices, claims, debts, dependents, and continuing commitments. It should distinguish:

* A personal promise from an obligation attached to an office.
* Private property from an organization’s property.
* A legally recognized successor from the person who physically controls the assets.

Do not transfer everything associated with the old ruler to the new one merely because both occupy the same character slot. Conversely, do not erase institutional obligations when the signatory dies.

This is where CK3’s persistent claimants and Victoria 3’s separation of ownership from workplaces become particularly valuable design precedents.

### 4.6 Preserve economic conservation where daily life depends on it

For food, fuel, tools, seed, and construction materials, use explicit quantities and transfers. Aggregate order submission and settlement for speed, but maintain the authoritative ownership and inventory consequences.

Price formation and allocation should be separate operations. A price can change before all desired transactions occur. Unfilled demand must remain unfilled demand—not consumption.

Allow several allocation mechanisms to share that underlying inventory system: household sharing, customary dues, rationing, reciprocal obligations, barter, market exchange, and appropriation. Early agrarian society should not require a universal cash market before its economy functions.

The same distinction applies to finance. Record claims and liabilities explicitly when they exist; do not confuse “money is conserved” with “the model is realistic.” Credit issuance, repayment, default, taxation, and currency creation need defined counterparties and accounting rules.

### 4.7 Build explanations into the execution path

Every important political action should retain a compact reason record:

> “Supported joint custody because the previous keeper’s shortage harmed this household; expected contribution cost is unchanged; cousin supports the proposal; existing oath does not prohibit it.”

The record should reference actual evaluated facts and rules. It should not be a plausible story invented afterward.

Likewise, a failed enactment should explain whether it lacked a quorum, lost a supporter, violated a higher rule, missed a deadline, or passed but could not be implemented. “Failed a 42% roll” can be appropriate for uncertain evidence or an individual decision; it is a poor universal explanation for institutional failure.

### 4.8 Schedule politics by relevance and events

Use cheap daily updates for changing needs and exposure, less frequent reviews for ordinary commitments, and event-driven work for deaths, vacancies, breaches, elections, crises, and proposals.

Maintain indexes for eligible officeholders, affected households, local creditors, witnesses, and potential coalition partners. Avoid repeatedly scanning the world to rediscover these sets.

For the Rust kernel, a useful pattern is immutable read views for parallel evaluation, followed by validated commits for resource transfers, office appointments, and rule changes. A proposal that was valid when evaluated must be rechecked when committed if its dependencies changed.

UE5 should display the consequences of this authoritative state. A distant person can receive cheaper rendering and less frequent deliberation without losing their identity, obligations, or economic existence.

### 4.9 Test institutional continuity, not predetermined historical outcomes

A small fixed set of diagnostic worlds should test whether obligations survive succession correctly, inventories balance, invalid appointments are rejected, and agents can actually invoke every supported institutional action.

Long-running checks should look for unexplained asset creation, permanently unreachable decisions, accumulating dead references, proposal deadlocks, and runaway combinations of legal bonuses.

Do **not** require a world to industrialize, democratize, centralize, or collapse on schedule. TCE’s success condition is that observed changes have traceable causes and remain mechanically coherent—not that they converge on a preferred historical sequence.

## 5. Sources worth reading first

**Victoria 3 mechanics and revisions:** [Standard of Living, DD13](https://steamcommunity.com/games/529340/announcements/detail/2965045886379003737?utm_source=chatgpt.com); [law enactment and revolution revision, DD80](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-80-law-enactment-and-revolution-clock-in-13?utm_source=chatgpt.com); [ownership and foreign investment, DD110](https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-110-building-ownership-foreign-investment.1647879/); [trade redesign, DD143](https://steamcommunity.com/ogg/529340/announcements/detail/524212840420081890?utm_source=chatgpt.com); and [1.12 release notes, mirrored from the developer announcement](https://steamdb.info/patchnotes/21114983/?utm_source=chatgpt.com). These provide the most directly reusable economy–politics mechanisms.

**CK3 engineering and political systems:** [Schemes and Stories, DD157](https://forum.paradoxplaza.com/forum/developer-diary/dev-diary-157-schemes-stories.1703863/), its [official video presentation](https://www.youtube.com/watch?v=rNqV5uc8VqI&utm_source=chatgpt.com), and [Performance and Optimization, DD187](https://forum.paradoxplaza.com/forum/developer-diary/dev-diary-187-performance-optimization.1861437/). Use the [succession-law wiki](https://ck3.paradoxwikis.com/Succession_laws?utm_source=chatgpt.com) as a mechanics index and [Factions Explained’s repository](https://github.com/amtep/ck3-factions-explained?utm_source=chatgpt.com) as a concrete example of explanatory tooling.

**Europa Universalis and institutional theory:** [EU4’s 1.32 development-concentration redesign](https://store.steampowered.com/news/app/236850/view/2887360777094419215?utm_source=chatgpt.com); Devereaux’s [EU4 analysis](https://acoup.blog/2021/04/30/collections-teaching-paradox-europa-univeralis-iv-part-i-state-of-play/?utm_source=chatgpt.com) and [EU5 firsthand examination](https://acoup.blog/2025/10/31/miscellania-europa-universalis-v-confirmed-first-impressions/?utm_source=chatgpt.com); [Institutional Grammar 2.0](https://arxiv.org/abs/2008.08937?utm_source=chatgpt.com); and Ostrom’s [Nobel lecture on polycentric governance](https://www.nobelprize.org/prizes/economic-sciences/2009/ostrom/lecture/?utm_source=chatgpt.com).

**Bottom line:** TCE should not implement “Victoria 3, but every pop is a person.” It should implement persistent people whose work, property, relationships, and commitments create pressures for institutions—and institutions that alter those same concrete relationships. That closes the causal loop these games approach from different directions, while avoiding their dependence on a player acting as the coordinating intelligence of society.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab927d9-4cb0-83ea-a3b3-7a98c7d591c6)
