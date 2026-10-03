# Labor regimes and wages: a simulation-ready report for TCE

## Executive recommendation

**Represent labor organization as a bundle of enforceable rights, obligations, and payment rules—not as a single “labor regime” assigned to a civilization.** A person can cultivate household land, owe seasonal public labor, hire out during harvest, and train a child in a craft. A cooperative can pay wages; an enslaved person can receive money without acquiring freedom; an unpaid household producer can retain substantial autonomy. Global labor-history classifications therefore distinguish work relationships rather than treating all productive activity as employment. [KNAW](https://pure.knaw.nl/ws/files/1825045/JGL2016.pdf)

For TCE, the central causal chain should be:

> **Resources and household needs → available alternatives → bargaining and coercive power → work obligations and compensation → production, distribution, and institutional change.**

The most important distinction is between **how much someone receives**, **what work they must perform**, and **whether they can refuse or leave**. These should remain separate throughout the simulation.

---

## 1. Mechanisms: rules a simulation can implement

### 1.1 The principal arrangements

These are useful authoring templates, not mutually exclusive population classes.

| Arrangement | How work and compensation are organized | Implementable representation |
| --- | --- | --- |
| **Household production and reciprocal labor** | Household members produce for consumption or sale; assistance can create reciprocal obligations rather than immediate payment. Much work never enters a wage market. | Household production plans, access to assets, consumption-sharing rules, care responsibilities, and reciprocal claims between households. [KNAW](https://pure.knaw.nl/ws/files/1825045/JGL2016.pdf) |
| **Wage labor** | Payment can be by time, task, or output, in cash or goods. Employment can still involve restrictions on quitting: nineteenth-century British wage workers faced criminal sanctions for breach of contract. | A contract specifies remuneration, work, duration, payment dates, termination rights, and enforcement—not simply a wage number. [National Bureau of Economic Research](https://www.nber.org/papers/w17051) |
| **Apprenticeship** | Productive work is exchanged for training, maintenance, and sometimes wages; fees may flow from the apprentice’s household to the master. Formal terms and actual service can differ substantially. | A training-and-employment contract with mentor time, skill acquisition, board, fees, productive assignments, and early-exit possibilities. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/apprenticeship-and-training-in-premodern-england/9F138B4E456FE2AF7D2DC4584C942E71) |
| **Sharecropping** | A cultivator receives a specified share of output. Land, seed, animals, tools, and other inputs may be supplied by different parties. | Separate output shares from input-cost shares, fixed dues, debt claims, tenure duration, and eviction rights. [LSE Personal Pages](https://personal.lse.ac.uk/ghatak/tenancy.pdf) |
| **Corvée or labor levy** | An authority claims labor from specified people or communities. Obligations may be rotational, commutable into money, or accompanied by provisioning or pay. | A public or seigneurial claim on eligible people, with quotas, scheduling windows, substitutes, exemptions, and enforcement. [Melissa Dell](https://dell.scholars.harvard.edu/sites/g/files/omnuum7696/files/dell/files/ecta8121_0.pdf) |
| **Serfdom** | Dependent cultivators combine access to holdings with dues and personal restrictions. The particular combination of labor services, payments, inheritance, and mobility restrictions varies. | Link claims to a person, household, holding, or jurisdiction; do not assume every serf owes the same weekly work schedule. [Sheilagh Ogilvie](https://sheilaghogilvie.com/wp-content/uploads/publications/Ogilvie-Carus-2014-CESifo-WP-4861.pdf) |
| **Slavery** | Institutions recognize extensive claims over persons and their labor; transferability, inheritance of status, manumission, and permitted economic activity vary. Slavery can operate inside commercial economies. | Preserve the enslaved person as an agent. Model legal claims, constrained movement, provisioning, appropriation, resistance, family relationships, and possible status changes. [Cambridge University Press](https://www.cambridge.org/core/books/abs/slavery-in-the-late-roman-world-ad-275425/introduction/2B81412E65F5CD7007547BF90BB115DB?utm_source=chatgpt.com) |
| **Worker cooperatives** | Workers possess membership rights over governance and residual income. Members may receive wages and distributions; nonmember employees can have different rights. | Separate employment from membership, voting rights, capital contributions, retained earnings, and surplus allocation. [King Center on Global Development](https://kingcenter.stanford.edu/publications/working-paper/wages-employment-and-capital-capitalist-and-worker-owned-firms) |

A finite indenture, a debt obligation, and hereditary enslavement should likewise be different combinations of rules. **Do not infer permanent bondage merely from debt, or free labor merely from a nominal contract.**

### 1.2 Outside options govern bargaining—but scarcity has two possible effects

A worker’s meaningful alternative is not the highest wage anywhere in the world. It is the best option they can actually access after accounting for travel, information, land rights, family responsibilities, moving costs, and enforcement.

Research on coercion makes an important conditional prediction: better worker alternatives can weaken coercion, while an increase in the value of extracted labor can strengthen employers’ incentives to impose it. Consequently, “labor becomes scarce” does not determine the institutional outcome by itself. [nber.org](https://www.nber.org/papers/w15581?utm_source=chatgpt.com)

**TCE rule:** evaluate two separate responses to scarcity.

* With accessible land, competing employers, and protected exit, vacancies should tend to raise compensation.
* With concentrated ownership, restricted escape, and strong enforcement, elites may instead seek tighter obligations or movement controls.

The second response should require political action and resources. It must not appear as an automatic economy-wide modifier. Historical institutions often served distributional interests rather than maximizing total output. [Sheilagh Ogilvie](https://sheilaghogilvie.com/wp-content/uploads/publications/Ogilvie-Carus-2014-CESifo-WP-4861.pdf)

### 1.3 Households allocate labor across several activities

An individual’s employment decision belongs inside a household production problem. Food cultivation, gathering fuel, childcare, processing food, maintaining tools, and paid work compete for time. Observed poor households frequently combine several occupations rather than specialize completely. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.21.1.141)

**TCE rule:** maintain a household task calendar. A wage offer is attractive only relative to the value and urgency of displaced activities.

This produces useful emergent behavior:

* A farmer accepts construction work in a slack season but refuses the same offer during harvest.
* A household with food reserves can reject an unfavorable contract.
* An illness can remove both the sick person’s labor and a caregiver’s time.
* A household can remain employed yet become poorer because it loses subsistence production.

These are consequences of the proposed time-and-resource accounting, not separate scripted events.

### 1.4 Contract form allocates risk, incentives, and supervision

For a simple sharecropping contract:

\[
I\_T=\alpha\,pY-C\_T-D
\]

where \(I\_T\) is tenant net income, \(\alpha\) the tenant’s output share, \(pY\) harvest value, \(C\_T\) tenant-paid inputs, and \(D\) additional dues.

A fixed-rent tenant retains the marginal harvest but bears substantial output risk. A share tenant shares that risk while retaining only part of additional output. A wage worker receives more predictable contractual compensation, although employment and payment themselves may be uncertain. Contract theory explains the coexistence of these arrangements through risk, monitoring, wealth constraints, and incomplete markets. [LSE Personal Pages](https://personal.lse.ac.uk/ghatak/tenancy.pdf)

**TCE rule:** derive outcomes from those underlying constraints. Do not give sharecropping a universal “−20% productivity” penalty.

Security also matters. When better performance can trigger higher extraction or eviction, investment and effort may be discouraged. Tenancy reform in West Bengal provides evidence that changes in bargaining power and security can affect agricultural performance. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/pdf/10.1086%2F338744?utm_source=chatgpt.com)

### 1.5 Apprenticeship is production plus investment

Historical apprenticeship was not simply schooling followed by a skill unlock. Apprentices contributed productive labor while learning, and many left before completing their formal terms. English evidence also challenges the assumption that training costs were always recovered only at the very end. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/apprenticeship-and-training-in-premodern-england/9F138B4E456FE2AF7D2DC4584C942E71)

**TCE rule:** update skill through suitable tasks, supervision, and accumulated experience. Deduct teaching time from the mentor’s current production.

A useful contract contains:

\[
\text{current pay}+\text{maintenance}+\text{training}
\quad\leftrightarrow\quad
\text{work}+\text{fees}+\text{service commitment}.
\]

Allow completion, transfer to another master, early departure, exploitation without much training, and successful training followed by independent business formation. Access restrictions should affect who obtains training—not impose inherent ability differences between social groups.

### 1.6 Coercion must have material and human consequences

Coercive institutions should alter the feasible choices of agents, not remove their decision-making. Enforcement requires an organization capable of detecting, pursuing, adjudicating, or punishing noncompliance. Its effectiveness depends partly on workers’ escape opportunities and the surrounding political order. [DOI](https://doi.org/10.3982/ECTA8963)

**TCE rule:** separate the claimant’s financial return from the worker’s welfare and the economy’s physical productivity.

Model provisioning, supervision, interrupted family life, restricted movement, fatigue, injury, flight, and resistance through existing systems. A profitable coercive enterprise need not be socially productive: profitability can reflect appropriation and externalized costs.

Do not assign an intrinsic productivity coefficient to “slave,” “serf,” or “free worker.” Let task suitability, knowledge, nutrition, effort, supervision, and incentives produce the difference.

### 1.7 Transitions change particular rights, not an entire era

| Pressure or reform | Possible transition | Necessary qualification |
| --- | --- | --- |
| Stronger worker alternatives | Labor services become harder to enforce; compensation rises; obligations may be commuted. | Scarcity alone does not guarantee this outcome. Political and property institutions matter. [Sheilagh Ogilvie](https://sheilaghogilvie.com/wp-content/uploads/publications/Ogilvie-Carus-2014-CESifo-WP-4861.pdf) |
| Fiscal and administrative restructuring | Authorities substitute money payments and hired personnel for some hereditary service obligations. | Paid service can remain unfree. Ming–Qing China supplies examples of overlapping paid, hereditary, and bonded arrangements. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/tributary-labour-relations-in-china-during-the-mingqing-transition-seventeenth-to-eighteenth-centuries/7AC034138FC2F5591D10313C256F2E53) |
| Emancipation | Personal restrictions end; incentives and mobility can change. | Subsequent land arrangements can amplify or limit gains. Research on Russia’s 1861 emancipation distinguishes emancipation from later communal land arrangements. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20160144) |
| Tenancy reform | Crop shares, eviction threats, and investment incentives change. | A legal entitlement must become enforceable to alter behavior. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/pdf/10.1086%2F338744?utm_source=chatgpt.com) |
| Worker organization | Members acquire control over hiring, compensation, and surplus. | Worker-owned firms may prioritize employment stability over stable wages. [King Center on Global Development](https://kingcenter.stanford.edu/publications/working-paper/wages-employment-and-capital-capitalist-and-worker-owned-firms) |

For TCE, a reform should therefore be an operation such as `remove_exit_permission_requirement`, `make_status_nonhereditary`, or `convert_labor_due_to_cash`, with a jurisdiction, affected population, enforcement capacity, and political opposition.

---

## 2. Parameters: quantitative evidence and calibration limits

### 2.1 Measure real wages without confusing them with household welfare

An influential Allen-style comparison uses:

\[
WR=\frac{250w}{3\times1.05\times B},
\]

where \(w\) is the male daily wage and \(B\) an adult’s annual basic consumption basket. The convention assumes **250 paid days**, **three adult-equivalent consumption baskets**, and a **5% housing addition**. A ratio of one means that hypothetical annual earnings purchase the specified family budget. [Nuffield College](https://www.nuff.ox.ac.uk/Users/Allen/fiveauthor%20chines.pdf)

**This is a comparative accounting convention, not a demographic or employment law.** Annual-contract evidence can produce different historical income trajectories from daily wages multiplied by assumed working days. [History Oxford](https://www.history.ox.ac.uk/publication/1100544/crossref)

For TCE, record separately:

1. Hourly or daily compensation in purchasing-power terms.
2. Actual paid days and annual earnings.
3. Household consumption, including own production and transfers.
4. Working time, health, and freedom of exit.

A welfare ratio below one does not establish starvation: the household may have other earners, land, stored food, credit, or transfers. Conversely, a high male daily wage does not establish secure year-round household income. Historical wage and household-income research makes precisely this distinction important. [History Oxford](https://www.history.ox.ac.uk/publication/1100544/crossref)

### 2.2 Illustrative real-wage benchmarks

**Confidence:** **H** = well-documented within the stated definition; **M** = reconstructed or sample-sensitive; **L** = approximate or thin evidence. None is a universal regional constant.

| Population and period | Approximate benchmark | Units and interpretation | Confidence / source |
| --- | --- | --- | --- |
| London urban unskilled men, roughly 1740–1800 | **3–4.5** | Family basic-basket welfare ratio; approximate readings of the published trajectory, not a period mean. | **M–L**; Allen et al. 2011. [Nuffield College](https://www.nuff.ox.ac.uk/Users/Allen/fiveauthor%20chines.pdf) |
| Beijing urban unskilled men, roughly 1740–1800 | **1.1–1.6** | Same study’s comparative framework; approximate chart range. | **M–L**; Allen et al. 2011. [Nuffield College](https://www.nuff.ox.ac.uk/Users/Allen/fiveauthor%20chines.pdf) |
| Northern Indian wage evidence | About **1.5 in the late seventeenth century**, falling **below 1 by the mid-eighteenth** | Reconstructed basic-basket purchasing power; regional coverage and basket choices matter. | **M**; de Zwart and Lucassen 2020. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.12996) |
| Selected British-colonial West versus East African cities, 1900–1940 | Unweighted mean **2.7 versus 1.4** | Author working-paper estimates using **312 annual workdays**; not population-weighted regional means and not directly interchangeable with 250-day estimates. | **M**; Frankema and van Waijenburg. [Ewout Frankema](https://ewoutfrankema.com/wp-content/uploads/2016/05/CGEH.WP.No_.24.2012-1.pdf) |

These are useful **scenario anchors**, not targets that every TCE world should converge toward. Before comparing datasets, harmonize workdays, household equivalence, basket contents, housing, and treatment of payments in kind.

### 2.3 Contract, compensation, and work-allocation parameters

| Parameter | Historical value or range | Correct scope and simulation use | Confidence / source |
| --- | --- | --- | --- |
| English apprenticeship term | **7 years** | Formal statutory benchmark; actual completion and service length varied. Use as a contract template, not a universal learning time. | **H** for the formal rule; **M** for practice. Wallis. [Cambridge University Press](https://www.cambridge.org/core/books/abs/apprenticeship-in-early-modern-europe/apprenticeship-in-england/16E7737A34B47561364BE9C4D032EFFF) |
| English construction skill ratio | Approximately **2.0** before the Black Death; around **1.5** afterward for extended periods | Skilled daily pay divided by unskilled pay. A fall from a 100% to a 50% premium does **not** mean the wage ratio halved. | **M**; Federico, Nuvolari, and Vasta 2020. [New York University Abu Dhabi](https://nyuad.nyu.edu/content/dam/nyuad/academics/divisions/social-science/working-papers/2020/0051.pdf) |
| Chinese construction skill ratio, 1769–1795 | **1.83** in Beijing; **1.36** for the population-weighted national norms | Calculated from official wage schedules. These are not verified realized market wages for every project. | **M–L**; Allen et al. 2011. [Nuffield College](https://www.nuff.ox.ac.uk/Users/Allen/fiveauthor%20chines.pdf) |
| Skilled/unskilled urban construction pay in sampled British African colonies | Mostly **2–4×** | Craft and location-specific gaps, not an inherent productivity ratio. | **M**; Frankema and van Waijenburg. [Ewout Frankema](https://ewoutfrankema.com/wp-content/uploads/2016/05/CGEH.WP.No_.24.2012-1.pdf) |
| Rural versus urban unskilled African compensation | Approximately **0.5–1.0×** | Rural cash-plus-kind remuneration relative to urban unskilled wages in the assembled evidence. | **M**; same study. [Ewout Frankema](https://ewoutfrankema.com/wp-content/uploads/2016/05/CGEH.WP.No_.24.2012-1.pdf) |
| Female/male unskilled urban wage ratio, Bengal | Around **0.4** in the mid-eighteenth century; around **0.8** in the 1810s–1820s | Based on only **192 female observations**; occupation composition and institutions complicate interpretation. Never translate into biological productivity coefficients. | **M–L**; de Zwart and Lucassen. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.12996) |
| Colonial Andean mining mita recruitment | **1/7 of assessed adult men** in designated communities, in rotation | A recruitment rule for the affected population, not one-seventh of everybody’s time or of the total population. The institution operated **1573–1812**. | **H** for the prescribed system; implementation varied. Dell 2010. [Melissa Dell](https://dell.scholars.harvard.edu/sites/g/files/omnuum7696/files/dell/files/ecta8121_0.pdf) |
| Italian worker-cooperative wage differential, 1982–1994 sample | About **14% lower average wages** | Accompanied by more variable wages and more stable employment. A sample outcome, not a cooperative wage penalty. | **M**; Pencavel, Pistaferri, and Schivardi 2006. [Sage Journals](https://journals.sagepub.com/doi/10.1177/001979390606000102) |
| Agta mothers’ direct childcare, Philippines field observations | **30.1%** of observed daylight with youngest child under two, versus **15.9%** with youngest aged two to ten | A local time-allocation observation; excludes any claim about a universal prehistoric workday. | **M** locally, **L** for broad transfer. Dyble et al. 2019. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/10091712/3/Dyble_Agta%20time%20budgets%20MS.pdf) |

**Parameters that should remain scenario-specific:** serf labor-days owed, corvée days per year, apprenticeship fees, slave provisioning, probability of escape, punishment severity, monitoring cost, and the production effect of coercion. The sources assembled here do not justify globally applicable numerical defaults for them.

For these, author the actual institution first—eligibility, calendar, dues, exemptions, enforcement—and label initial numerical settings as **calibration choices**, not historical estimates.

### 2.4 Accounting rules that prevent misleading results

TCE should value cash and usable goods separately before aggregating them. Board supplied by an employer is a resource transfer; it must not also appear as independently purchased household food. A harvest share is gross revenue until input costs and dues are deducted.

Likewise, distinguish wages **promised**, **accrued**, and **paid**. Your historical observer can then generate both contractual wage series and actual realized earnings.

These are recommended bookkeeping invariants. They are especially important when comparing annual service, intermittent day labor, and mixed cash-and-kind contracts—the categories that historical income reconstruction often struggles to reconcile. [History Oxford](https://www.history.ox.ac.uk/publication/1100544/crossref)

---

## 3. Variation across eras and regions

The following periods organize evidence; they should **not** become mandatory TCE development stages.

| Setting | Important variation | Implication for TCE |
| --- | --- | --- |
| **Foraging societies** | Foragers were not uniformly egalitarian. Pacific Northwest societies included hereditary hierarchy and slavery, while nearby societies rejected such arrangements. Contemporary Agta research also finds different time allocations with greater agricultural engagement. [AnthroSource](https://anthrosource.onlinelibrary.wiley.com/doi/10.1111/aman.12969) | Agriculture is not a prerequisite for hierarchy or slavery. Storage, property institutions, social norms, and political relationships need independent representation. Do not use one universal “forager workweek.” |
| **Early farming and administrative states** | Ur III Mesopotamian records include detailed accounts of workers and labor-days. Such administrative categories do not, by themselves, establish that every recorded worker was a free wage earner or an enslaved person. [CDLI](https://cdli.earth/artifacts/101241) | Introduce labor accounting, provisioning, and obligations without requiring coinage or a modern employment category. Keep uncertain historical status classifications explicit. |
| **Ancient Mediterranean** | Slavery remained economically significant in late antiquity, including agriculture and textile production. Its persistence does not fit a simple sequence in which commercial development immediately replaces slavery. [Cambridge University Press](https://www.cambridge.org/core/books/abs/slavery-in-the-late-roman-world-ad-275425/introduction/2B81412E65F5CD7007547BF90BB115DB?utm_source=chatgpt.com) | Commercialization can coexist with coercion. Market access and labor freedom require separate variables. |
| **Medieval and early-modern Europe** | Serfdom, household farming, urban crafts, apprenticeship, and wage work overlapped. Differences in political organization help explain divergent institutional trajectories rather than a uniform continent-wide transition. [Sheilagh Ogilvie](https://sheilaghogilvie.com/wp-content/uploads/publications/Ogilvie-Carus-2014-CESifo-WP-4861.pdf) | Let neighboring jurisdictions maintain different rights and enforcement arrangements despite similar technologies. |
| **Ming–Qing China** | Household agriculture coexisted with hereditary service, bondservants, wage workers, and privileged but unfree service groups. Long-term hired workers’ legally inferior status gradually eased after 1735. Bondage was not always attachment to land. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/tributary-labour-relations-in-china-during-the-mingqing-transition-seventeenth-to-eighteenth-centuries/7AC034138FC2F5591D10313C256F2E53) | Permit claims over households or persons independently of parcels. Pay, privilege, and freedom can move in different directions. |
| **South Asia** | Northern Indian wage series reveal substantial temporal variation; evidence also documents seasonal and long-distance labor recruitment. Estimates from northern regions should not be generalized automatically to southern India or all household producers. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.12996) | Build regional labor markets linked by routes and recruitment networks, alongside local household production and status restrictions. |
| **Africa and the Atlantic world** | African colonial wage evidence varies with access to land, migration, taxation, and labor recruitment. The transatlantic slave trade forcibly relocated millions: approximately **12.5 million embarked** and **10.7 million survived the crossing** over the sixteenth–nineteenth centuries. These figures exclude many deaths outside the crossing itself. [Ewout Frankema](https://ewoutfrankema.com/wp-content/uploads/2016/05/CGEH.WP.No_.24.2012-1.pdf) | Model displacement, capture, transport, and receiving labor systems as connected processes. Do not reduce Africa to a single wage level or labor institution. |
| **Colonial Andes** | The mining mita imposed territorially defined recruitment and encouraged avoidance, substitution, and migration. It should not be treated as identical to every precolonial Andean labor obligation. [Melissa Dell](https://dell.scholars.harvard.edu/sites/g/files/omnuum7696/files/dell/files/ecta8121_0.pdf) | Eligibility attaches to communities and jurisdictions; people can respond by moving, bargaining, or paying for substitutes. |
| **Industrial societies** | Factory employment did not immediately imply free contract. British criminal enforcement of worker contract breach persisted until **1875**; Russia emancipated serfs in **1861**, with outcomes shaped by subsequent land arrangements. [National Bureau of Economic Research](https://www.nber.org/papers/w17051) | Industrial technology must not automatically remove coercion or equalize bargaining power. |
| **Modern societies** | Formal wage employment coexists with household production, informal work, cooperatives, and coercion. The ILO’s **2018** statistical picture estimated **61.2%** of world employment was informal. Its **2021-reference-year** forced-labor estimate was **27.6 million people**, including **3.9 million** in state-imposed forced labor. These are dated snapshots, not 2026 estimates. [International Labour Organization](https://www.ilo.org/publications/women-and-men-informal-economy-statistical-picture-third-edition) | Informality, wage payment, and coercion are separate dimensions. Modernization does not collapse all labor into one salaried category. |

A particularly useful historical contrast is therefore not “primitive versus advanced,” but **the same productive task performed under different property, recruitment, compensation, and exit rules**.

---

## 4. Stylized facts a credible simulation should reproduce

These are conditional validation patterns, not requirements that every generated history follow the same trajectory.

### 4.1 Households can have several livelihoods simultaneously

In a survey discussed by Banerjee and Duflo, households in **27 West Bengal villages** had a median of **three working members engaged in seven occupations**. The transferable fact is livelihood diversification, not a universal target of seven occupations. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.21.1.141)

**Test:** households with seasonal farming and uncertain cash employment should maintain multiple productive relationships. A schema permitting only one permanent occupation per person will systematically miss this behavior.

### 4.2 Daily wages and annual living standards can diverge

Historical annual-income estimates can differ materially from conventional daily-wage reconstructions. [History Oxford](https://www.history.ox.ac.uk/publication/1100544/crossref)

**Test:** an economy must permit rising day rates alongside falling annual earnings. For example, a **10%** increase in daily pay combined with a decline from **250 to 150 paid days** produces:

\[
1.10\times150/250=0.66,
\]

or **34% lower annual wage income**. This is an illustrative calculation, not a historical estimate.

It should also permit food-price inflation to reduce real wages while contractual money wages remain unchanged.

### 4.3 Skill premiums vary rather than converging to a universal multiplier

The English and African construction benchmarks above span markedly different skill ratios. Cross-country historical research likewise finds substantial variation in skill premia. [Cambridge University Press](https://www.cambridge.org/core/journals/european-review-of-economic-history/article/skill-premium-and-the-great-divergence/2683C3D108F87DAF8941BB14374402DA)

**Test:** scarcity of trained workers, access to apprenticeship, occupational exclusion, and demand composition should influence skill premiums. Doubling physical skill should not mechanically double pay.

### 4.4 Labor scarcity has institution-dependent consequences

The coercion literature does not support an unconditional rule that fewer workers always produce greater freedom. Changes in worker alternatives and the profitability of extraction pull in different directions. [nber.org](https://www.nber.org/papers/w15581?utm_source=chatgpt.com)

**Test:** run the same population shock under two institutional configurations. Protected exit and competing employers should produce a different response from concentrated ownership and effective movement restrictions.

### 4.5 Productivity gains need not pass immediately into workers’ consumption

A reconstruction discussed by Crafts implies approximately **38% growth in output per worker** versus **21% growth in real consumption earnings** in Britain between **1780 and 1840**. However, alternative earnings series give substantially different results. The famous “Engels’ pause” is therefore not a single uncontested numerical target. [OUP Academic](https://academic.oup.com/oep/article/74/1/1/6145336)

**Test:** productivity gains can initially accrue to owners, lower prices, capital accumulation, or rents rather than wages. The eventual distribution should depend on hiring competition, ownership, bargaining, and policy—not a fixed labor share.

### 4.6 Institutional reforms can have persistent effects

Dell’s border-based study of Peru’s mining mita estimates approximately **25% lower household consumption** and **six percentage points more childhood stunting** in historically affected areas. These are long-run, geographically specific effects involving subsequent institutions and infrastructure—not immediate penalties to apply whenever forced labor exists. [DOI](https://doi.org/10.2139/ssrn.1596425)

Similarly, Banerjee, Gertler, and Ghatak attribute around **28% of subsequent agricultural productivity growth** in their analysis to West Bengal’s tenancy reform. That is a contribution to observed growth, **not** a universal 28% productivity-level increase. [Paul Gertler](https://paulgertler.com/research/?utm_source=chatgpt.com)

**Test:** changing rights should alter investment, migration, wealth distribution, and political power over time. Removing an obligation should not erase its accumulated consequences.

### 4.7 Cooperatives can absorb shocks through income rather than employment

Evidence from Italian and Uruguayan worker cooperatives identifies different wage and employment adjustment patterns from conventional firms. [King Center on Global Development](https://kingcenter.stanford.edu/publications/working-paper/wages-employment-and-capital-capitalist-and-worker-owned-firms)

**Test:** cooperative members facing a revenue decline can vote to reduce distributions or hours before dismissing members. This should emerge from their objective and governance rules, not from guaranteed superior survival.

### 4.8 Unpaid care is an economically significant constraint

The ILO’s 2018 care-work report estimated that women performed **76.2% of unpaid care hours globally**. This describes a gendered allocation of work, not an inherent difference in productive capacity. [International Labour Organization](https://www.ilo.org/resource/news/ilo-women-do-4-times-more-unpaid-care-work-men-asia-and-pacific)

**Test:** changes in childcare provision, household composition, and care-sharing norms should alter market participation and training opportunities without changing underlying ability.

---

## 5. Recommended representation for individual agents and institutions

### 5.1 Store relationships, not a single labor-status enum

A compact design could use the following entities.

| Entity | Essential state |
| --- | --- |
| **Person** | Skills, health, available time, location, social connections, individual assets, legal status, permissions, and active obligations. |
| **Household** | Members, dependents, food stocks, productive assets, care requirements, income pooling, internal allocation rules, and seasonal work plan. |
| **Work contract or obligation** | Parties, tasks, duration, calendar, payment formula, input provision, claim priority, termination conditions, and enforcement jurisdiction. |
| **Enterprise or worksite** | Production tasks, material constraints, vacancies, supervision capacity, liquidity, owners, and member-governance rules where applicable. |
| **Institution** | Eligibility rules, permitted contracts, labor dues, mobility restrictions, adjudication, enforcement resources, and reform procedures. |

A display label such as “tenant,” “apprentice,” or “bonded worker” can be derived from these relationships. It should not determine all behavior.

**Keep ownership of assets, control of work, and control of persons separate.** This allows, for example, a tenant to own tools, an apprentice to retain savings, or a cooperative to employ nonmembers.

### 5.2 Give every person one conserved daily time budget

A daily scheduler should reconcile household tasks, contractual employment, levy obligations, care, travel, rest, and illness.

The key invariant is:

\[
\sum \text{scheduled activity time}
\leq
\text{available time}.
\]

An authority’s claim does not create extra labor-hours. Conflicting demands must produce something observable: default, negotiation, displacement of household production, exhaustion, substitution, or enforcement.

For visible daily life, this gives TCE meaningful scenes without additional narrative scripting: harvest crews arriving late because of a levy, apprentices dividing time between errands and instruction, or caregivers turning down distant jobs.

### 5.3 Use bounded local matching

For ordinary employment, use a posted-offer process:

1. A worksite creates vacancies justified by tasks, expected sales, and its ability to pay.
2. Workers inspect a small set of reachable offers through local and social networks.
3. They compare expected take-home resources, reliability, travel, training, working conditions, and exit costs.
4. Employers choose among applicants using skill and institutionally permitted criteria.
5. Unfilled vacancies and hiring losses inform later wage revisions.

Existing EURACE-family models provide a useful precedent: workers search subsets of offers, use reservation wages, and differ in skills; firms adjust offers in response to recruitment difficulties. [Springer](https://link.springer.com/article/10.1007/s43253-021-00052-5)

For constrained workers, the same evaluation can inform resistance or attempts to leave, but it must not override legal and physical restrictions automatically.

### 5.4 Separate enforcement from compliance

A useful sequence is:

> Obligation becomes due → agent complies, negotiates, substitutes, or defaults → claimant detects default → institution decides whether and how to act.

This permits weak states, selective enforcement, patronage, evasion, and administrative overload.

Track coercive claims in financial and legal records without turning people into interchangeable inventory. Capture, sale, relocation, manumission, and inherited status should update the same continuing person and family relationships.

### 5.5 Let institutional change come from political actors

TCE notables and organized groups should propose specific changes when their interests or norms shift.

Workers may seek exit rights or lower dues; employers may support wider recruitment but resist bargaining rights; rulers may prefer predictable cash revenue to difficult labor collection. These motives can conflict even when all groups would benefit from some increase in total production.

The implementation recommendation is to connect proposed reforms to **expected group effects**, organizational capacity, legitimacy, and enforcement—not to an institution-wide “efficiency score.”

### 5.6 Initial engineering settings

The following are **proposed starting settings**, not historical estimates.

| Setting | Initial range | Purpose |
| --- | --- | --- |
| Ordinary offer search | **3–8 candidate offers per search event** | Bound computation while preserving imperfect information. |
| Routine job-search review | **Weekly**, with event-triggered search after job loss | Avoid continuous reconsideration by every worker. |
| Employer wage review | Every **14–30 days**, with faster review for urgent seasonal tasks | Provide persistence without preventing harvest bidding. |
| Ordinary wage adjustment | Approximately **1–5% per review**, configurable | Start with bounded adjustments; allow exceptional renegotiation after major shocks. |
| Contract and levy processing | Event-driven at due dates | Avoid scanning every obligation every tick. |
| Skill updates | Daily accumulated experience; periodic summaries | Connect visible work to learning without expensive planning. |

At **50,000 people**, inspecting five offers per person in one complete search round gives **250,000 candidate evaluations**. That is an arithmetic workload estimate, not a performance benchmark. Search only relevant workers and index vacancies by location, task, and eligibility.

The historical parameter tables should constrain plausible outcomes; these engineering settings determine how cheaply the simulation approaches them.

### 5.7 Existing models and games worth borrowing from

| Model or game | Useful mechanism | What TCE must add or change |
| --- | --- | --- |
| **EURACE@Unibi and related models** | Heterogeneous skills, reservation wages, sampled job search, vacancies, and employer wage adjustments. [Springer](https://link.springer.com/article/10.1007/s43253-021-00052-5) | Household subsistence production, seasonal obligations, restricted mobility, noncash contracts, and overlapping legal statuses. |
| **Labor-market extensions of Keynes-meets-Schumpeter** | Links between wage setting, bargaining arrangements, hiring/firing, unemployment, and aggregate demand. [OUP Academic](https://academic.oup.com/ser/article/16/4/687/4739737) | Treat policy results as model-dependent. Do not transplant a modern institutional package into early agrarian worlds. |
| **Victoria 3’s documented employment systems** | A game-scale precedent for connecting building wages to recruitment, profitability, and labor competition. Its population groups provide an aggregation strategy. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-78-update-1-2-changelog) | TCE needs persistent individuals, actual time allocation, household portfolios, and authored rights rather than relying on population-group categories. |

**The most valuable simplification is bounded decision-making—not deleting household work or collapsing legal relationships.**

---

## 6. Sources, datasets, and uncertainty

### 6.1 Recommended data foundations

| Source | What it supplies | Main caution |
| --- | --- | --- |
| **Global Price and Income History, UC Davis** | Downloadable historical wages, prices, and related series across Europe, Asia, Africa, and the Americas. Useful for constructing local baskets and occupational pay comparisons. [GPIH](https://gpih.ucdavis.edu/Datafilelist.htm) | Preserve units, coverage, occupation, payment period, and cash/kind conventions. A column called “wage” is not automatically comparable across files. |
| **IISH Global Collaboratory on the History of Labour Relations** | Global labor-relationship classifications and historical snapshots, including household and nonwage work. [IISH](https://iisg.amsterdam/en/research/projects/global-collaboratory-on-the-history-of-labour-relations-1500-2000) | Snapshot categories should inform validation, not force each TCE person into one exclusive lifelong class. |
| **De Zwart–Lucassen northern Indian wage data** | Occupational and regional wage observations underlying the 2020 study; associated data are identified by **10.3886/E118365V1**. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.12996) | Uneven geography, sparse early coverage, and limited female observations. |
| **SlaveVoyages** | Voyages, forced migration, and demographic evidence for the transatlantic and intra-American slave trades. [Slave Voyages](https://www.slavevoyages.org/?utm_source=chatgpt.com) | A source for transport and demographic processes—not a general dataset of workplace productivity or living conditions. |
| **Cuneiform Digital Library Initiative** | Primary administrative records useful for early labor accounting, provisioning, and institutional obligations. [CDLI](https://cdli.earth/artifacts/101241) | Translation and classification require care; administrative totals are not automatically individual annual work schedules. |
| **ILO statistical reports** | Modern employment status, informality, care work, and forced-labor estimates. [International Labour Organization](https://www.ilo.org/publications/women-and-men-informal-economy-statistical-picture-third-edition) | Use dated reference populations and definitions. Informal employment and forced labor are not interchangeable categories. |

### 6.2 The strongest scholarly starting points

For **comparative wages**, start with Allen et al. (2011), de Zwart and Lucassen (2020), and Frankema and van Waijenburg’s African reconstruction. For **annual-income measurement**, add Humphries and Weisdorf’s *Unreal Wages?* rather than relying solely on daily rates. [Nuffield College](https://www.nuff.ox.ac.uk/Users/Allen/fiveauthor%20chines.pdf)

For **institutional mechanisms**, Acemoglu and Wolitzky’s *The Economics of Labor Coercion*, Wallis’s apprenticeship research, Banerjee–Gertler–Ghatak on tenancy reform, and Markevich–Zhuravskaya on Russian emancipation are particularly relevant. [DOI](https://doi.org/10.3982/ECTA8963)

For **regional institutional diversity**, Moll-Murata’s research on Chinese labor relations and Harper’s work on late-Roman slavery prevent a Europe-centered progression from becoming the implicit model. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/tributary-labour-relations-in-china-during-the-mingqing-transition-seventeenth-to-eighteenth-centuries/7AC034138FC2F5591D10313C256F2E53)

### 6.3 Claims to flag rather than hard-code

**Legal rules are not observed compliance.** Official wage schedules, apprenticeship terms, and levy quotas establish what was prescribed. They do not establish how often employers paid, apprentices completed, or communities supplied the required labor. The distinction is visible in both apprenticeship and mita research. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/apprenticeship-and-training-in-premodern-england/9F138B4E456FE2AF7D2DC4584C942E71)

**The Great Divergence is sensitive to region and measurement.** A northern Indian urban wage reconstruction is not a measurement of all Indian living standards. Basket construction and selected occupations affect comparisons. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.12996)

**Industrial wage growth remains contested.** Different earnings series can substantially alter the apparent gap between productivity and worker gains. Do not calibrate to a single famous “Engels’ pause” number. [OUP Academic](https://academic.oup.com/oep/article/74/1/1/6145336)

**Causal effect sizes are less portable than mechanisms.** A robustness study of Naidu and Yuchtman’s British contract-enforcement analysis reproduced baseline results but found sensitivity to specification. The existence of criminal enforcement is firmer evidence than a universal estimate of its wage effect. [Research Portal](https://research.hhs.se/esploro/outputs/workingPaper/Robustness-Report-on-Coercive-Contract-Enforcement/991001583999306056)

**Prehistoric work-hours and universal coercion-efficiency estimates are particularly weak calibration foundations.** Local ethnography is valuable, but its measured activities, daylight windows, ecology, and household composition must accompany the numbers. [UCL Discovery](https://discovery.ucl.ac.uk/id/eprint/10091712/3/Dyble_Agta%20time%20budgets%20MS.pdf)

### Bottom line for TCE

Build **one household economy, one local labor-matching system, and one composable rights-and-obligations system**. Derive named labor regimes from their combinations.

That architecture can generate a farmer who hires harvest workers but owes road labor, an apprentice whose training becomes a path to independence, a commercially successful yet coercive estate, or a cooperative that preserves employment by reducing distributions. More importantly, it lets transitions change individual lives through actual work, resources, movement, and political power—rather than through an era label or a productivity bonus.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9288d-cc90-83e9-8a7b-d3570db04eab)
