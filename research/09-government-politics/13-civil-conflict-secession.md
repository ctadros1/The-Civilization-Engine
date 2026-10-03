# Rebellion, civil war, and secession: a simulation-ready report for TCE

## Executive recommendation

**Model rebellion as an attempt to build a rival political organization—not as the automatic conversion of unhappiness into soldiers.** Separate five processes: reasons to resist, recruitment and coordination, organizational survival, competition for authority, and bargaining over an ending. Research on insurgency, ethnic mobilization, and conflict termination supports treating these as distinct problems rather than estimating one universal “rebellion probability.” [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/ethnicity-insurgency-and-civil-war/B1D5D0E7C782483C5D7E102A61AD6605)

For TCE, the central causal chain should be:

> **A contested demand + a coordinating coalition + resources and protection + an opening in state control → a potentially sustainable rebellion.**

This is a modeling synthesis, not a claim that every rebellion passes through identical stages. A governor’s defection can create an army immediately because the organization already exists; a dispossessed village movement must assemble one. Both may seek concessions rather than independence.

Keep **military defeat, cessation of violence, and political resolution separate**. UCDP’s termination categories distinguish agreements, ceasefires, victories, low activity, and disappearance of an actor. Several end an observed conflict episode without settling its underlying dispute. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/monadterm/UCDPConflictTerminationDataset_v4_2024_Codebook.pdf)

The strongest quantitative evidence concerns nineteenth- to twenty-first-century states, especially the period after 1945. Use it to test mechanisms and modern-like configurations—not to impose modern country-level rates on early farming settlements.

---

# 1. Mechanisms: rules the simulation can implement

## 1.1 Resolve “greed versus grievance” into different causal channels

Collier and Hoeffler’s influential results favored variables associated with the **opportunity to sustain rebellion** over several country-level grievance indicators. This did not establish that rebels were personally motivated by greed. Economic variables can represent financing, recruitment costs, state weakness, or grievances imperfectly measured elsewhere. Their headline primary-commodity relationship was subsequently challenged for sensitivity to sample construction and missing data. [OUP Academic](https://academic.oup.com/oep/article-abstract/56/4/563/2361902)

Fearon and Laitin emphasized conditions favoring insurgency: weak state capabilities, political instability, difficult terrain, and poverty. They found that ethnic or religious diversity alone did not explain civil-war onset after accounting for other factors. This is not equivalent to saying that politically structured ethnic grievances are irrelevant. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/ethnicity-insurgency-and-civil-war/B1D5D0E7C782483C5D7E102A61AD6605)

Research using politically relevant groups rather than whole-country diversity measures finds stronger relationships between rebellion and **exclusion from power, recent loss of access, mobilization capacity, and previous conflict**. TCE should therefore represent *who is disadvantaged by which institution*, not assign a rebellion modifier to cultural diversity itself. [Cambridge University Press](https://www.cambridge.org/core/journals/world-politics/article/abs/why-do-ethnic-groups-rebel-new-data-and-analysis/4F69AEFBD88FAADE63207FF7D1F69449)

### Implementable mechanism table

The rules below are proposed translations of the evidence, not estimated behavioral equations.

| Mechanism | Rule for TCE | Important qualification |
| --- | --- | --- |
| **Grievance and attribution** | Record concrete harms—confiscation, discriminatory exclusion, unpaid obligations, coercive labor, violence—and whom the person holds responsible. Increase support for movements offering a relevant remedy. | Material hardship without a blamed actor or plausible alternative can produce withdrawal, migration, or accommodation instead. Political exclusion is more informative than generic diversity. [Cambridge University Press](https://www.cambridge.org/core/journals/world-politics/article/abs/why-do-ethnic-groups-rebel-new-data-and-analysis/4F69AEFBD88FAADE63207FF7D1F69449) |
| **Coordination and organizational survival** | A prospective organizer must obtain commitments through existing relationships and institutions. Distinguish sympathizers, providers, recruits, and reliable members. | Study failed organizations too. Lewis’s Uganda research finds that factors explaining initial formation differ from those explaining survival and escalation. [Sage Journals](https://journals.sagepub.com/doi/citedby/10.1177/0010414016672235) |
| **Opportunity costs versus appropriable revenue** | Lower civilian earnings can make participation less costly; concentrated resources can finance contenders or make territorial control more valuable. Calculate these channels separately. | In Colombia, coffee and oil price changes affected conflict through different mechanisms. “More resources” should not have one universal sign. [Universidad del Rosario](https://pure.urosario.edu.co/en/publications/commodity-price-shocks-and-civil-conflict-evidence-from-colombia-2/) |
| **Uneven state reach** | Calculate deployable force, response time, reliable local information, and official loyalty for each district. A large distant army is not equivalent to effective local authority. | Remoteness and border access are associated with longer conflicts, but effects depend on the capabilities of both sides. [Sage Journals](https://journals.sagepub.com/doi/pdf/10.1177/0022002709336457) |
| **Elite and military defection** | Allow officeholders and commanders to transfer their existing followers, stores, and administrative relationships into a rival organization. | This is a different onset pathway from assembling an insurgency among civilians. Tang military organization illustrates how regional commands could become durable competing power centers. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/reach-of-the-military-tang/ECFE9C64AE6E73EC4319B165A606F88C) |
| **Control, information, and civilian choices** | Let residents comply with, inform, evade, or support different authorities according to their local security and relationships. Public compliance need not indicate sincere loyalty. | Kalyvas models selective violence as jointly produced by armed organizations and civilians supplying information, including for personal reasons. [Cambridge University Press](https://www.cambridge.org/core/books/abs/logic-of-violence-in-civil-war/theory-of-selective-violence/F536CE68812EFC89D673084F26D8DC36) |
| **Repression with competing effects** | Violence can increase fear, destroy organizational capacity, create grievances, and change allegiance simultaneously. Track these channels independently. | Repression does not invariably backfire. Lyall’s Chechnya study found reduced subsequent attacks in its particular comparison; this does not establish lasting legitimacy or general effectiveness. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002708330881) |
| **Faction fragmentation** | Allow organizations to split over leadership, territory, resources, ideology, or settlement terms. Count factions that can actually obstruct an agreement. | More consequential veto players can prolong conflict through incompatible demands, information problems, and incentives to hold out. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/pdf/10.1111/j.1540-5907.2006.00221.x) |
| **Credible commitment** | Reject settlements that appear attractive today but expose a faction to unacceptable vulnerability after demobilization. Evaluate implementation and future enforcement. | A central problem in settlement is whether parties can safely comply, not merely whether they can identify a mutually beneficial bargain. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/critical-barrier-to-civil-war-settlement/AF2E36B866EC5E658266D01C5B00B42F) |

### Two particularly important consequences

**First, grievances and resources interact.** A subsistence crisis may increase willingness to resist while simultaneously making it harder to feed supporters. Conversely, a prosperous district may have relatively content residents but powerful elites with the means and incentives to seek greater autonomy. Implement the underlying household and organizational budgets rather than adding independent “poverty” and “wealth” rebellion bonuses.

**Second, ethnicity should be a relationship and political boundary, not a behavioral essence.** People can share language while fighting over succession, class, religion, land, or patronage; a movement can also unite several cultural groups. Cederman’s emergent-actor modeling is useful precisely because it does not assume that states and nations are fixed, identical units. [International Conflict Research](https://icr.ethz.ch/publications/emergent/)

## 1.2 When does a movement seek secession?

Give each organization an explicit, revisable objective:

| Objective | What success requires in the simulation |
| --- | --- |
| **Policy concession** | Change the disputed tax, labor obligation, land arrangement, religious restriction, or similar rule. |
| **Replacement of officials** | Remove specified officeholders without necessarily replacing the political order. |
| **Central-government takeover** | Acquire enough central institutions and allegiance to exercise government. |
| **Regional autonomy** | Obtain enforceable control over specified domains, such as taxation, courts, appointments, or local armed forces. |
| **Independence** | Establish a separate authority that can sustain itself against the parent’s competing claim. |

Do not make independence the default endpoint of regional anger.

For a proposed secession, evaluate five conditions separately:

**A politically coordinated constituency.** A regional majority is not automatically an organization. Existing councils, provincial administrations, religious institutions, or elite networks can make collective action and subsequent government more feasible.

**A recognizable territorial unit.** Administrative boundaries can supply a focal point for both mobilization and settlement. Griffiths finds that movements aligned with particular administrative categories have different prospects from movements demanding new boundaries; governments can sometimes release an entire category without establishing the same precedent for their core territories. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/abs/between-dissolution-and-blood-how-administrative-lines-and-categories-shape-secessionist-outcomes/0809B62A86516C3C83C3B1DDD04AB601)

**A viable post-separation arrangement.** Assess food access, revenues, administrative personnel, trade dependence, and security. These are proposed TCE accounting requirements, not a fixed historical minimum population or income.

**A parent government willing or unable to retain the region.** Its calculation should include revenue, strategic value, domestic prestige, feared precedents, and the cost of continued rule.

**An external political environment.** Foreign intervention, abandonment, expulsion, and collapse at the center feature prominently in Roeder’s study of successful independence campaigns. Secessionist military effort alone is not an adequate model of success. [OUP Academic](https://academic.oup.com/cornell-scholarship-online/book/30740/chapter-abstract/261985469)

Represent **de facto independence, parent-state acceptance, and outside recognition as separate variables**. For early empires, add intermediate arrangements: tribute without direct administration, nominal allegiance with local rule, or contested obligations. Do not require every successful breakaway to resemble a modern internationally recognized state.

## 1.3 How wars end

Use an outcome structure richer than `winner = government/rebels`.

| Ending | Necessary simulation change | What may remain unresolved |
| --- | --- | --- |
| **Government victory** | The rival organization loses the practical ability to pursue its armed challenge. | Local grievances, clandestine networks, displaced populations, and future challengers. |
| **Rebel victory at the center** | Central institutions and enough supporting organizations transfer to the victorious coalition. | Rival factions, constitutional disagreement, provincial obedience. |
| **Negotiated reform or autonomy** | Specific institutional concessions and security arrangements are implemented. | Compliance, enforcement, and the distribution of authority. |
| **Secession or partition** | Authority over a territory separates, with a viable government or enduring rival jurisdiction. | Borders, minorities, recognition, and future reconquest. |
| **Ceasefire or low activity** | Organized violence falls or stops. | Armed capacity and the political dispute may survive. |
| **Fragmentation or disappearance** | A faction dissolves, merges, becomes inactive, or changes organizational identity. | Other factions may continue fighting over the same issue. |

The distinction between agreement, ceasefire, victory, low activity, and actor termination follows the logic of UCDP’s coding system; the institutional consequences above are proposed TCE implementations. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/monadterm/UCDPConflictTerminationDataset_v4_2024_Codebook.pdf)

**War weariness is not a sufficient termination rule.** Even exhausted parties may fear what happens after surrendering their bargaining power. Peace agreements therefore need an implementation process: changes in control, phased compliance, credible protection, and consequences for violations. Walter’s commitment argument is more useful here than a universal “exhaustion reaches 100” threshold. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/critical-barrier-to-civil-war-settlement/AF2E36B866EC5E658266D01C5B00B42F)

Outside guarantees should work only through actual capabilities and incentives. Fortna’s research associates peacekeeping with more durable peace, but the appropriate abstraction for TCE is a mechanism—monitoring, reassurance, enforcement—not a magical peacekeeping bonus. [JSTOR](https://www.jstor.org/stable/j.ctt7sv7j)

---

# 2. Parameters: evidence, units, and calibration limits

## 2.1 Empirical benchmarks

**These are observational benchmarks or study estimates, not ready-made engine constants.** Confidence below distinguishes confidence in the reported quantity from confidence that it transfers to TCE.

| Quantity | Value and units | Population, period, and source | Confidence and appropriate use |
| --- | --- | --- | --- |
| **UCDP armed-conflict threshold** | **≥25 battle-related deaths per calendar year**, with the required actors and political incompatibility | UCDP/PRIO coding rules. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/ucdpprio/ucdp-prio-acd-261.pdf) | **High as a definition.** Use for comparable reporting, not to trigger rebellion. |
| **UCDP war-intensity threshold** | **≥1,000 battle-related deaths per calendar year** | UCDP/PRIO intensity classification. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/ucdpprio/ucdp-prio-acd-261.pdf) | **High as a definition; poor direct scale fit** for a 10k–50k-person world. |
| **Overall civil-war duration** | Estimated **median 7.1 years; mean 11.1 years** | Fearon 2004: 128 wars during 1945–1999; Weibull estimates accounting for ongoing cases. [ResearchGate](https://www.researchgate.net/publication/2533194_Why_Do_Some_Civil_Wars_Last_So_Much_Longer_than_Others) | **Moderate historical benchmark; low direct premodern transfer.** Test a long-tailed distribution. |
| **Different pathways, different durations** | Estimated median **2.1 years** for wars arising from coups/popular revolutions versus **9.0 years** for other wars | Fearon 2004. These are conflicts meeting the study’s civil-war criteria—not all coup attempts. [ResearchGate](https://www.researchgate.net/publication/2533194_Why_do_Some_Civil_Wars_Last_so_Much_Longer_than_Others) | **Moderate**, observational and classification-sensitive. Do not assign all wars one termination hazard. |
| **“Sons-of-the-soil” conflict duration** | Estimated median **23.2 years**, versus **5.8 years** for other wars | Fearon 2004: conflicts involving local populations, land, and state-supported settlement. [ResearchGate](https://www.researchgate.net/publication/2533194_Why_do_Some_Civil_Wars_Last_so_Much_Longer_than_Others) | **Low–moderate** because of a small, censored subgroup. Treat as evidence for persistent mechanisms, not a 23-year timer. |
| **Secession campaign success** | **26/171 = 15.2%** achieved independence | Roeder: campaigns reaching the agenda of Western great powers, **1945–2016**, including nonviolent campaigns. [OUP Academic](https://academic.oup.com/cornell-scholarship-online/book/30740/chapter-abstract/261985469) | **Moderate for this selected denominator.** Not an annual probability or an armed-war success rate. |
| **Negotiated civil-war endings in an older sample** | Approximately **20%**, compared with **55%** of interstate wars | Walter’s **1940–1990** comparison. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/critical-barrier-to-civil-war-settlement/AF2E36B866EC5E658266D01C5B00B42F) | **Moderate historical benchmark.** Not a timeless or current global proportion. |
| **Peacekeeping and recurrence** | Hazard ratio approximately **0.32** in one post-Cold-War model: about **68% lower instantaneous recurrence hazard** | Fortna 2004, post-Cold-War analysis. [Columbia University](https://www.columbia.edu/~vpf4/ISQ%20offprint.PDF) | **Moderate association; uncertain causal transfer.** Not a 68-percentage-point reduction in relapse probability. |
| **Multiple veto players and long wars** | **Four versus two veto players:** approximately **4×** the probability of a conflict lasting 13 years in the reported comparison | Cunningham, *Barriers to Peace in Civil War*. [Cambridge University Press](https://www.cambridge.org/core/books/abs/barriers-to-peace-in-civil-war/effects-of-veto-players-on-conflict-severity-genocide-and-the-duration-of-peace/4BCB88ED6B4489289F2616BC6F9E0B9F) | **Moderate–low as a portable magnitude.** Not four times the duration or a per-faction coefficient. |

### Three measurement rules

**Preserve the denominator.** An independence campaign, an armed organization, a state–rebel dyad, a conflict episode, and a country-year are different observations. A movement may contain several organizations and experience several episodes before achieving—or abandoning—its objective.

**Account for unfinished wars.** Ongoing conflicts have incomplete durations. Fitting a distribution only to completed wars systematically favors short cases. The duration estimates above should not be confused with the age of wars still underway. [ResearchGate](https://www.researchgate.net/publication/2533194_Why_Do_Some_Civil_Wars_Last_So_Much_Longer_than_Others)

**Do not shrink modern death thresholds proportionally and call the result validated.** For TCE’s internal classification, use organized reciprocal violence, rival authority, persistence, and mobilization. Export death-threshold classifications separately. A political community of 2,000 people can experience a consequential rebellion without reaching a modern international dataset’s inclusion threshold.

There is **no well-supported universal annual rebellion rate for an early agrarian district** in the evidence reviewed here. It is better to leave that as an emergent outcome than invent a seemingly empirical probability.

## 2.2 Proposed engineering priors

These values are **design starting points, not historical estimates**. Their confidence is low until sensitivity testing and scenario calibration establish that they do not determine the results artificially.

| Parameter | Starting value | Sensitivity range | Units and purpose |
| --- | --- | --- | --- |
| Routine political reassessment interval | 14 | 7–30 | Days; major events can trigger immediate reassessment. |
| Acute grievance-memory half-life | 24 | 6–60 | Months; decay of a specific recent harm, not inherited identity. |
| Trust-recovery half-life after betrayal | 5 | 2–15 | Years; keep separate from grievance decay. |
| Salient social contacts evaluated per decision | 16 | 8–32 | Contacts; an attention/computation budget, **not total network size**. |
| Organizational food-buffer stress tests | 90 | 30–180 | Days of actual consumption covered by accessible stocks; supplies must exist in the economy. |
| Persistence before reporting stable local control | 90 | 30–180 | Days; reporting hysteresis, not an independence prerequisite. |
| Observation window for a “quiet episode” label | 12 | 6–24 | Months; does not erase unresolved demands or surviving organizations. |

Do not introduce a free-standing “rebel manpower percentage.” Membership should come from actual individuals and institutions; combat capacity must be constrained by equipment, food, training, health, competing work, and allegiance.

---

# 3. Variation across eras and regions

## 3.1 Use institutional capabilities, not era unlocks

| Social configuration | Conflict processes to represent | Principal modeling caution |
| --- | --- | --- |
| **Foragers and small mobile communities** | Group fission, departure, collective resistance to overbearing individuals, and contests over leadership or resources. | “Civil war” may be an inappropriate label where there is no durable central authority. Boehm’s reverse-dominance framework is a useful hypothesis, not evidence that all foragers were uniformly egalitarian or peaceful. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/204166) |
| **Early farming communities and emerging states** | Resistance to extraction, flight from officials, local coalitions, succession disputes, and contests over stored resources and labor. | Distinguish farming from statehood. Some communities can leave; others are constrained by land, infrastructure, neighbors, or dependence on centralized institutions. Treat these as explicit TCE conditions rather than fixed historical stages. |
| **Pre-industrial states and empires** | Provincial defection, dynastic competition, religious mobilization, resistance to labor and tribute, and negotiated layers of autonomy. | Authority can be divided among courts, towns, temples, nobles, military commanders, and village institutions. “Who owns the province?” is often too simple. Tang and Kongo provide concrete examples. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/reach-of-the-military-tang/ECFE9C64AE6E73EC4319B165A606F88C) |
| **Industrializing and mass-mobilizing societies** | Larger political constituencies, stronger administrative penetration, rapid communication, and organizational competition over both central and regional government. | As a TCE design, let transport, communications, schooling, taxation, and military organization change coordination and control separately. Increased destructive power need not produce better local information; Lyall and Wilson advance this argument for counterinsurgency, though its generality is contested. [DOI](https://doi.org/10.1017/S0020818309090031) |
| **Modern territorial states** | Professional armed organizations, internationalized support, legal-administrative territorial claims, formal peace processes, and recognition politics. | This is where the strongest comparable quantitative evidence exists. Its institutions must be present before transferring its estimates. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/) |

### Geography is conditional, not destiny

Scott’s account of mainland Southeast Asian highlands emphasizes mobility, dispersion, and adaptation as ways of evading state extraction. That suggests an important TCE alternative to rebellion: **make oneself harder to govern**. [Yale University Press](https://yalebooks.yale.edu/book/9780300169171/the-art-of-not-being-governed/)

But archaeology and historical evidence from Cambodia, Java, and the Philippines also show highland settings sustaining substantial state connections, agriculture, and political organization. Mountains should affect travel, production, information, and settlement—not automatically generate statelessness or insurgency. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-global-history/article/do-mountains-kill-states-exploring-the-diversity-of-southeast-asian-highland-communities/25BAB08DB456293630979EEF83CADE0C)

## 3.2 Premodern cases as mechanism tests

These are illustrative cases, not a representative sample from which to calculate global rates.

| Case | Historical pattern | Useful TCE test |
| --- | --- | --- |
| **An Lushan rebellion, Tang China, 755–763** | During the rebellion, military provinces expanded from ten frontier commands to more than forty, many in the interior. Regional military organization became a lasting institutional problem. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/reach-of-the-military-tang/ECFE9C64AE6E73EC4319B165A606F88C) | A dynasty can survive an uprising while losing centralized control. Raising forces to survive today can create tomorrow’s autonomous power centers. |
| **Zanj rebellion, southern Iraq, 869–883** | A prolonged uprising connected to harsh labor conditions and the agricultural landscape sustained a major challenge to Abbasid authority before suppression. Recent archaeological work revisits the material setting. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/landscape-of-the-zanj-rebellion-dating-the-remains-of-a-largescale-agricultural-system-in-southern-iraq/8EFD7A767C88F526B45330E4AC4D309C) | Exploitation alone is insufficient: test the interaction of concentrated labor, organizing opportunities, resources, geography, and state response. |
| **Kingdom of Kongo, civil-war era after 1665** | Fragmentation changed political opportunities, including the public exercise of territorial power by elite women; rival centers and religious mobilization were part of the subsequent struggle over reunification. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-african-history/article/abs/elite-women-in-the-kingdom-of-kongo-historical-perspectives-on-womens-political-power/45D114266D66DF3BFF35B23892C608CB) | Do not force every war into two stable ethnic camps. Offices, kinship, factional membership, and the eligibility of political leaders can change during conflict. |
| **Pueblo revolt, 1680; Spanish reconquest beginning in 1692** | An effective anti-colonial coalition expelled Spanish authority, but the unity enabling the revolt did not remain intact indefinitely. Barrett emphasizes interacting demographic, environmental, and colonial pressures. [University of New Mexico Press](https://www.unmpress.com/9780826324139/conquest-and-catastrophe/) | Initial liberation and durable independence are different outcomes. A victorious coalition can fragment, and an expelled ruler can return. |

For ancient and premodern settings, evidence is particularly thin on **failed conspiracies, unobtrusive resistance, ordinary peaceful years, and reliable casualty totals**. Use well-supported institutional sequences more confidently than global frequency estimates.

---

# 4. Stylized facts a correct simulation should reproduce

These should be ensemble-level tendencies, not scripted outcomes in every world.

| Target pattern | What to measure in TCE |
| --- | --- |
| **Formation is not survival.** Explanations based only on organizations that reached sustained fighting miss early failures. [Sage Journals](https://journals.sagepub.com/doi/citedby/10.1177/0010414016672235) | Separately count attempted organization, durable organization, territorial challenge, and sustained war. |
| **War duration is highly heterogeneous.** The historical benchmarks above include both relatively short central struggles and very prolonged peripheral conflicts. [ResearchGate](https://www.researchgate.net/publication/2533194_Why_Do_Some_Civil_Wars_Last_So_Much_Longer_than_Others) | Survival curves and duration quantiles by mechanism, with unfinished wars retained—not just mean duration. |
| **Political relationships matter more than diversity alone.** Access to power, exclusion, and changes in status are central group-level predictors. [Cambridge University Press](https://www.cambridge.org/core/journals/world-politics/article/abs/why-do-ethnic-groups-rebel-new-data-and-analysis/4F69AEFBD88FAADE63207FF7D1F69449) | Hold cultural composition constant while changing institutional inclusion. Conflict risk should respond without rewriting identities. |
| **Economic shocks can have opposite effects.** Labor-income and appropriable-revenue channels differ. [Universidad del Rosario](https://pure.urosario.edu.co/en/publications/commodity-price-shocks-and-civil-conflict-evidence-from-colombia-2/) | Compare a wage shock with a concentrated-rent windfall; do not require the same sign of response. |
| **Independence is not the usual outcome of every campaign.** Roeder’s selected sample records 26 successes among 171 campaigns. [OUP Academic](https://academic.oup.com/cornell-scholarship-online/book/30740/chapter-abstract/261985469) | Track concessions, autonomy, inactivity, defeat, and independence separately. Do not target 15.2% outside comparable conditions. |
| **A quiet year need not settle the dispute.** Conflict datasets explicitly distinguish low activity from political agreements and victories. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/monadterm/UCDPConflictTerminationDataset_v4_2024_Codebook.pdf) | Preserve surviving organizations, disputed jurisdiction, displaced people, and unfulfilled promises after violence declines. |
| **Ending a rebellion need not restore the previous constitution.** Tang military decentralization illustrates lasting institutional change during a nominally surviving regime. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/reach-of-the-military-tang/ECFE9C64AE6E73EC4319B165A606F88C) | Measure postwar tax control, appointments, military command, and regional autonomy—not only the ruler’s identity. |

An additional design check is **conservation of people and resources**. Long conflicts must reproduce their support through actual food production, financing, recruitment, migration, and generational replacement. They should not persist because an invisible rebel pool replenishes itself.

---

# 5. Modeling recommendation for a 10k–50k-agent Rust simulation

## 5.1 Minimum useful entities

Use existing TCE people, households, institutions, economic stocks, and military organizations wherever possible.

| Entity | Essential state |
| --- | --- |
| **Person** | Household obligations; resources; salient harms; institutional trust; multiple affiliations; relationships; perceived risks; current commitments. |
| **Household** | Food security, dependents, labor availability, displacement options, members’ political commitments. |
| **Political organization** | Demands, leaders, membership, supporters, internal factions, obligations, resources, legitimacy claims, negotiating authority. |
| **District** | Population and production, transport connections, disputed claims, administrative presence, tax compliance, information access, armed presence. |
| **Polity** | Appointment and succession rules, revenue arrangements, coercive institutions, legitimacy claims, agreements with subordinate authorities. |
| **Conflict** | Participating organizations, disputed issues, episodes, casualties, displacement, current control, proposed settlements, implementation status. |

An organization can exist before its armed wing. A military unit can change allegiance without changing personnel. A political movement can survive the loss of an army.

## 5.2 Individual decisions: several alternatives, imperfect information

Use the same bounded-rationality machinery as the rest of TCE. At a political decision point, a person might consider continued compliance, negotiation, evasion, migration, material support, organizational membership, or armed participation.

A proposed utility structure is:

\[
U\_i(a)=
\text{expected material outcome}
+\text{security}
+\text{social commitments}
+\text{perceived justice}
-\text{opportunity cost}
-\text{expected sanctions}.
\]

A noisy choice rule can then be:

\[
P\_i(a)=
\frac{\exp\!\left(U\_i(a)/\tau\_i\right)}
{\sum\_{b\in A\_i}\exp\!\left(U\_i(b)/\tau\_i\right)},
\]

where \(A\_i\) includes only actions the person can actually undertake, and \(\tau\_i\) governs decision noise.

This is a proposed implementation, not a civil-war equation estimated from the cited studies. Keep its utility units compatible with TCE’s existing decisions. **Do not count deprivation twice**—once through lost material utility and again through an unrelated universal poverty bonus.

Give people beliefs based on local experience, trusted contacts, and communications. They should not know the true national probability of rebel victory.

A simple event-memory component could be:

\[
g\_i(t+\Delta t)=
\operatorname{clip}\_{[0,1]}
\left[
g\_i(t)\,2^{-\Delta t/h\_i}
+\text{attributed harm}
-\text{credible redress}
\right].
\]

Keep perceived injustice, fear, trust, and attachment to a political identity separate. One scalar cannot represent a person who fears the government, distrusts the rebels, and still wants autonomy.

## 5.3 Organization and logistics: no spontaneous armies

A movement’s commitments must be financed from real resources:

\[
\text{Food runway in days}
=
\frac{\text{accessible food}}
{\text{daily consumption of supported people}}.
\]

“Accessible” matters: nominally friendly stocks may be unreachable or withheld.

Track ordinary expenditures and obligations—food, equipment, compensation, transport, administration, and support for affected households—through the normal economy. Members can desert, return to farming, change factions, or become unavailable through illness and injury.

Let organizational strategy respond to shortages, but do not make depletion an automatic surrender. Leaders and households can choose among compromise, reduced activity, dissolution, migration, or continued resistance at greater cost.

## 5.4 Territory: represent capabilities, not only ownership

For each district, distinguish:

**Claimed jurisdiction:** who says it belongs to them.

**Administrative activity:** who appoints officials, adjudicates disputes, or provides services.

**Extraction:** who actually receives taxes, tribute, or supplies.

**Coercive access:** who can operate there and how consistently.

**Population cooperation:** who receives voluntary assistance, concealment, or information.

These quantities need not identify the same authority. A district may pay two contenders while trusting neither. Border polygons can be a presentation layer over this richer state.

For secession, create a new polity when an organization establishes sufficiently persistent governing institutions and separate external relations under the simulation’s rules. Keep the parent’s acceptance and other polities’ recognition independent of that creation event.

## 5.5 Peace as an executable institutional contract

A settlement should contain concrete terms rather than a generic “peace accepted” flag:

| Contract component | Example of an implementable term |
| --- | --- |
| Jurisdiction | A regional council receives appointment authority in specified districts. |
| Revenue | A defined share or category of taxation remains local. |
| Security | Named forces change status under a staged, observable process. |
| Legal protection | Specified past acts receive amnesty, subject to the polity’s actual law. |
| Participation | Excluded organizations receive defined access to offices or deliberation. |
| Enforcement | A capable institution or outside polity monitors and responds to violations. |

Evaluate agreements using each consequential faction’s expected future security, not only its current payoff. A rebel leader should be able to prefer imperfect autonomy with credible protection over generous promises that can be revoked immediately.

Implementation failures should produce specific consequences: reduced trust, renewed demands, defections, or renewed fighting. Avoid resetting the entire population’s grievances when a treaty is signed.

## 5.6 Scheduling and simplification

Use **daily** updates for consumption, movement, combat consequences, displacement, and local incidents. Use **weekly or monthly** reassessment for routine political commitments, organizational budgets, and bargaining, with immediate event-triggered updates for major defections or institutional changes.

A sparse relationship graph and cached district-level conditions are preferable to checking every person against every faction every day. At 50,000 people, 16 salient outgoing relationships each would involve about 800,000 relationship entries; that is an architectural illustration, not a performance benchmark.

Simplify combat detail before simplifying allegiance, provisioning, territorial administration, or settlement credibility. For this research question, those political-economic processes explain more than tactical battlefield realism.

When using stochastic event hazards, preserve time units:

\[
P(\text{event in }\Delta t)=1-e^{-\lambda\Delta t}.
\]

Do not apply a yearly probability every month. Also, do not allow subdivision of one district into ten reporting districts to multiply its underlying rebellion risk tenfold. Attach organization-formation opportunities to people, networks, and institutions rather than arbitrary map partitions.

## 5.7 Existing models worth borrowing from

| Model or framework | Borrow | Do not assume it already solves |
| --- | --- | --- |
| **Epstein, “Modeling civil violence” (2002)** | Local information, heterogeneous grievance and risk, and abrupt collective activation. | Persistent rebel institutions, war economies, secession, or constitutional change. Epstein explicitly notes that the model does not represent an existing political order and its overthrow. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC128592/) |
| **Cederman, *Emergent Actors in World Politics* (1997)** | States and nations as changing, emergent actors; formation and dissolution of political units. | Individually grounded household provisioning, local daily life, or empirically calibrated behavior for every era. [International Conflict Research](https://icr.ethz.ch/publications/emergent/) |
| **Kalyvas, *The Logic of Violence in Civil War* (2006)** | Interaction between fragmented authority, information, civilian choices, and armed organizations. | A universal model of onset or an equation saying all violence is highest in the same type of territory. [Cambridge University Press](https://www.cambridge.org/core/books/abs/logic-of-violence-in-civil-war/theory-of-selective-violence/F536CE68812EFC89D673084F26D8DC36) |

The best TCE approach is a synthesis: **Epstein-like local activation, emergent political organizations, institution-specific territorial control, real economic constraints, and commitment-sensitive bargaining.**

---

# 6. Source stack, validation strategy, and evidence limits

## 6.1 Recommended datasets

| Source | Best use | Main limitation |
| --- | --- | --- |
| **UCDP/PRIO Armed Conflict Dataset 26.1**, covering **1946–2025**, plus dyadic data | Comparable modern conflict incidence, actors, incompatibilities, and intensity. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/) | Omits much subthreshold and unsuccessful organization; not a premodern rebellion census. |
| **UCDP Conflict Termination Dataset, v4-2024** | Endings, episodes, and distinctions between inactivity and resolution. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/monadterm/UCDPConflictTerminationDataset_v4_2024_Codebook.pdf) | Pin versions and reconcile identifiers before combining with newer conflict releases. |
| **UCDP External Support Dataset**, **1975–2017** | Donors, recipients, and types of support. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/) | Support is selective and endogenous; correlations do not automatically identify effects. |
| **UCDP Peace Agreement Dataset 22.2**, **1975–2021**, and Conflict Issues data | Settlement terms and demands beyond a government-versus-territory binary. [Uppsala Conflict Data Program](https://ucdp.uu.se/downloads/) | Signed documents do not establish effective implementation. |
| **Ethnic Power Relations and GeoEPR** | Political inclusion, exclusion, group geography, and relationships to state power. [International Conflict Research](https://icr.ethz.ch/data/epr/beta.html) | Politically relevant group categories are not immutable individual identities. |
| **Ryan Griffiths’s secession datasets** | Historical movement-level comparison; version 5 extends across **1816–2017**. [Ryan D. Griffiths](https://www.ryan-griffiths.com/data) | Definitions and coverage differ from armed-conflict and Roeder campaign samples. |

The principal scholarly foundations are Collier–Hoeffler and Fearon–Laitin for onset; Cederman–Wimmer–Min and Lewis for political exclusion and organization; Dube–Vargas for distinct economic channels; Fearon and Cunningham for duration; Walter and Fortna for settlement credibility and durability; and Griffiths and Roeder for secession.

## 6.2 What is contested or thin

**Causal interpretation of national regressions remains limited.** Income, weak institutions, violence, and resource dependence can influence each other. Fearon’s critique of the primary-commodity result is a strong warning against converting an attractive cross-country curve into a universal engine rule. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002705277544)

**Short-run suppression is not long-run political success.** A decrease in attacks does not by itself demonstrate legitimacy, durable peace, or resolved grievances. The measured outcome and observation window must remain explicit. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002708330881)

**Settlement comparisons are affected by selection.** Peacekeepers, negotiated agreements, and military victories occur in different kinds of wars. Their observed associations should guide mechanisms and counterfactual tests rather than become unconditional bonuses. [Columbia University](https://www.columbia.edu/~vpf4/ISQ%20offprint.PDF)

**Premodern evidence supports mechanisms more securely than universal rates.** Historical cases are indispensable for representing provincial military power, colonial expulsion, religious mobilization, and layered authority. They provide a much weaker basis for a single cross-era probability of rebellion or secession.

For validation, run ensembles that vary one structural feature at a time: political exclusion with culture held constant; productive earnings versus concentrated rents; state reach versus nominal troop totals; intact versus fragmented rebel organizations; and credible versus unenforceable settlements. Report both aggregate outcomes and the individual/institutional pathways that produced them.

**Bottom line:** the minimum convincing rebellion system is not an unrest meter plus combat. It is **coalition formation, defections, provisioning, competing local authority, explicit political demands, and enforceable—or unenforceable—peace**. With those components, TCE can generate failed uprisings, negotiated autonomy, dynastic wars, prolonged insurgencies, successful secessions, and reconquests without scripting their historical sequence.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928d7-3bb4-83e9-9937-35e39a0da5d0)
