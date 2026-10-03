# Social stratification and inequality: emergence, persistence, and mobility

## Executive conclusion

**TCE should model stratification as the interaction of unequal resources, enforceable rights, and intergenerational transmission—not as a population moving through a predetermined sequence of classes.**

A productive household does not automatically become a noble lineage. Its advantage becomes durable when it can retain assets, exclude others from valuable opportunities, transfer advantages to descendants, and obtain institutional protection. Conversely, sharing obligations, accessible land, migration, competitive office selection, and collective resistance can prevent temporary differences from becoming hereditary domination. Comparative archaeology and anthropology support multiple trajectories rather than an inevitable progression from farming to aristocracy. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12037039/)

For implementation, keep four dimensions distinct:

| Dimension | What it measures | Why it must remain separate |
| --- | --- | --- |
| **Economic resources** | Assets, consumption, income, land access, debts | A wealthy merchant may lack political privileges. |
| **Political power** | Command, office, jurisdiction, influence over decisions | An official may control resources they do not personally own. |
| **Social prestige** | Locally recognized honor, expertise, generosity, ancestry | Esteem need not entail coercive authority. |
| **Legal status** | Rights, restrictions, obligations, eligibility, dependency | An economically successful person may remain legally subordinated. |

These are proposed simulation dimensions, not four universal historical ranks. Classes can emerge from economic relationships; estates and caste-like groups require additional mechanisms of institutional or social closure.

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Defensible resources turn temporary advantages into durable assets

Land, livestock, fishing locations, irrigation infrastructure, storage facilities, and trade access differ in how easily people can monopolize them. The important question is not simply whether a resource is productive, but whether access can be restricted and whether control survives its current holder. Dow and Reed’s formal model demonstrates a route from initially equal communities to exclusionary property rights and divisions between insiders and outsiders. This is a mechanism hypothesis, not a universal historical sequence. [Simon Fraser University](https://www.sfu.ca/~gdow/download/jpe%202013.pdf)

**Implement:** Each resource should have productivity, access cost, storage characteristics, exclusion cost, and transferable rights. Control should yield income only when other agents recognize the claim or its holder can enforce it. A claim unsupported by either convention or force should be contestable.

Distinguish ownership from use: communal land can provide valuable livelihood rights without being privately saleable.

### 1.2 Land scarcity changes bargaining power—but population alone is insufficient

The 2025 GINI-project analysis of **1,267 archaeological sites** associates greater inequality with the transition from **labor-limited** production, where households can expand cultivation, to **land-limited** production, where access to suitable land constrains them. Agriculture itself is not the decisive switch. Land limitation can arise through population pressure, geography, improvements, or socially restricted access. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12037039/)

**Implement:** Calculate the accessible opportunities facing each household, not merely settlement population density.

A household’s outside option might be:

\[
O\_h=\max\{\text{open new land},\text{rent elsewhere},\text{wage work},
\text{join kin},\text{migrate}\}-\text{associated costs}.
\]

As outside options deteriorate, households may accept higher rents, stronger patrons, or restrictive contracts. Fertile unoccupied territory should constrain landlords’ bargaining power only when households can actually reach and use it.

### 1.3 Surplus matters partly because some production is easier to appropriate

Mayshar, Moav, and Pascali argue that cereals’ storability and harvest characteristics made them more appropriable than some roots and tubers, facilitating taxation and hierarchy. However, a **2026 published critique** finds their statistical conclusions sensitive to outliers and to which institutional transition is examined. Treat appropriability as one plausible mechanism, not a rule that “grain creates states.” [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086%2F718372)

**Implement:** Taxation and predation should depend on harvest visibility, storage location, transportability, concealment opportunities, and enforcement. Two crops with equal caloric output need not generate equal taxable surpluses.

Do not substitute a generic “agricultural surplus” bonus for actual goods and collection costs.

### 1.4 Useful leadership can become entrenched authority

Leadership can initially increase welfare by coordinating public works, defense, or production. Powers and Lehmann’s individual-based model shows how beneficial coordination can support voluntary leadership, while costly exit and demographic expansion can subsequently permit greater appropriation by leaders. Its result depends on its assumptions; it does not establish that historical hierarchy generally began consensually. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4132689/?utm_source=chatgpt.com)

**Implement:** Allow organizers to acquire followers through demonstrated benefits. Followers should evaluate benefits received, contributions demanded, alternatives, and trust.

Office becomes more entrenched when its holder controls appointment, records, armed personnel, or access to essential infrastructure. But management should not automatically create private ownership: a community can retain collective infrastructure while delegating its operation.

### 1.5 Inheritance preserves both assets and the consequences of past shocks

Comparative evidence from small-scale societies finds stronger intergenerational persistence where material wealth is important and transferable. A favorable harvest, livestock windfall, or acquisition of land can therefore affect descendants rather than disappearing with its beneficiary. Wealth transmission also includes training, connections, and advantageous marriages—not just probate transfers. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

**Implement:** Transmit specific assets and opportunities through actual events: inheritance, marriage transfers, apprenticeship, introductions, and office nominations.

Keep their mechanisms separate. Equal division among children may fragment an estate, but marriage consolidation, purchases from siblings, unequal fertility, and indebtedness can offset that fragmentation. Do not apply an additional hereditary “class persistence” multiplier after already simulating these transfers.

### 1.6 Social closure restricts who can convert effort into opportunity

Caste illustrates how descent and endogamy can organize access to marriage, support, and economic opportunities. Munshi emphasizes that caste networks can provide insurance and employment assistance while also restricting choices and sustaining unequal access. **Jati**, the relevant endogamous community in many settings, should not be collapsed into the broad ideological categories of **varna**. Occupation, wealth, and ritual standing do not map perfectly onto one another. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjel.20171307)

**Implement:** Opportunities should have eligibility rules and social gatekeepers. These can include lineage membership, initiation, marriage connections, credentials, citizenship, patron sponsorship, or religious affiliation.

Model both benefits and barriers. A closed group may support disadvantaged insiders while excluding better-qualified outsiders. Changing jobs should not automatically change inherited group identity.

### 1.7 Coercion can substitute for paying workers their outside-option wage

Domar’s classic hypothesis links labor coercion to the conjunction of abundant land, scarce labor, and political capacity to restrict workers’ alternatives. The critical qualification is political: labor scarcity can raise free workers’ bargaining power **or** induce powerful landholders to seek restrictions on movement. Scarcity alone does not select the outcome. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/abs/causes-of-slavery-or-serfdom-a-hypothesis/B6055D4D909C9D21B79BB42D2CD952C5?utm_source=chatgpt.com)

**Implement:** Enslavement, debt bondage, and attachment to an estate require explicit institutional permissions or unlawful coercive actions—not automatic consequences of poverty.

Track the source of a coercive relationship, its enforcer, permitted obligations, escape possibilities, inherited status rules, and release conditions. Include supervision costs, resistance, flight, and distorted incentives.

The abolition of Russian serfdom provides evidence that changing rights can alter production: Markevich and Zhuravskaya find improvements in agricultural productivity, industrial output, and nutrition after emancipation, with subsequent land institutions affecting outcomes. This argues against treating coercion as simply a cheap-labor production bonus. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20160144)

### 1.8 Egalitarianism requires mechanisms, not merely an absence of elites

Boehm’s comparative work describes collective sanctions against domineering individuals, including criticism, ridicule, ostracism, and removal. Its broad evolutionary interpretation is debated, but the institutional lesson is useful: relatively egalitarian arrangements can be actively maintained. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/204166?utm_source=chatgpt.com)

**Implement:** Give agents ways to coordinate refusal, demand sharing, remove leaders, defend common access, and leave exploitative arrangements. These actions need supporters, information, and costs.

An ambitious individual should sometimes fail to become an elite because others successfully constrain them—not because the simulation silently caps their wealth.

### 1.9 Institutions can interrupt—or redirect—accumulation

Taxes, inheritance rules, public education, land reform, and access to finance change the conversion of current advantage into future advantage. Modern distributional evidence also shows markedly different inequality outcomes among economies with advanced technology; technological development does not specify one distribution. [World Inequality Report 2022](https://wir2022.wid.world/executive-summary/)

**Implement:** Redistribution must transfer real goods or claims. Public services must affect actual opportunities. Institutional reforms should change relevant rights and incentives immediately, while previously accumulated assets, networks, and reputations persist until subsequent events alter them.

Likewise, do not script “plague reduces inequality” or “industrialization increases inequality.” Let deaths, inheritance, asset destruction, wages, migration, and political responses determine the result.

---

## 2. Status systems and variation across societies

### 2.1 Estates, castes, classes, slavery, and serfdom are different structures

For TCE, use the following operational distinctions. They are deliberately compositional: one society can contain several simultaneously.

| System | Core organizing relationship | Representation in TCE | What mobility means |
| --- | --- | --- | --- |
| **Ranked order** | Unequal recognized honor or precedence | Ritual privileges, titles, recognized ancestry, audience-specific prestige | Recognition, succession, achievement, or loss of standing |
| **Estate** | Corporate legal privileges and obligations | A group with its own eligibility, taxation, service, jurisdiction, or representation rules | Admission, ennoblement, entry into an institution, expulsion, or abolition |
| **Caste-like order** | Descent-based membership plus strong closure, especially marriage | Hereditary group identity, marriage rules, sanctions, opportunity networks | Economic mobility within groups; rarer boundary crossing or collective changes in standing |
| **Class structure** | Different positions in production and control of assets | Landlords, tenants, independent producers, employers, wage workers, administrators | Changes in ownership, occupation, dependence, or control |
| **Slavery** | Institutionally enforced claims over a person and their labor | A coercive relationship between agents, with locally specified powers | Manumission, escape, redemption, emancipation, or changes in coercive control |
| **Serfdom** | Restricted exit and inherited obligations to a lord or estate | Land-use rights bundled with labor, rent, jurisdictional, and movement restrictions | Release, negotiated conversion of obligations, flight, or institutional reform |

**Do not make these six values of one enum.** A person can be a tenant, a member of a privileged estate, and relatively poor. Another can have substantial economic responsibility while lacking freedom. Historical caste research and the slavery/serfdom literature particularly warn against reducing status to income. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjel.20171307)

### 2.2 Foragers: often low material inequality, but not an unstratified universal

Mobile foraging societies often have strong leveling practices. However, the forager category also encompasses substantially different settlement patterns and resource regimes. Storage and restricted access to concentrated resources can support greater differentiation; the absence of farming does not itself guarantee equality. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/204166?utm_source=chatgpt.com)

**TCE implication:** Start from the actual property and cooperation arrangements. Do not assign all foragers a low-inequality trait or make hierarchy impossible before agriculture.

### 2.3 Early farming: long periods of relative equality are possible

The global archaeological evidence does not support an immediate, universal inequality jump at domestication. Settlement growth, land limitation, and governance interact, and some farming communities maintained relatively limited housing disparities. The older Eurasia–Americas contrast associated with traction animals is informative but should not become an exclusion rule: inequality also developed where those animals were absent. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12037039/)

**TCE implication:** Cultivation unlocks assets and opportunities for differentiation; it does not unlock an obligatory hereditary ruling class.

### 2.4 Pastoral societies: movable wealth can still reproduce inequality

Livestock illustrates why “mobile economy” and “egalitarian economy” are not synonyms. In the comparative sample below, pastoral wealth and its intergenerational persistence are substantially higher than among foraging and horticultural populations. These are sample patterns, not parameters for every pastoral society. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

**TCE implication:** Reproducing herds, transfer rules, labor availability, grazing rights, and catastrophic losses should generate the distribution. A large herd must still require care and access to pasture.

### 2.5 Pre-industrial states: several routes to status coexisted

**China:** Wen, Wang, and Hout analyze **3,640 male Tang-dynasty epitaphs**. Aristocratic pedigree became less predictive of career attainment while examination success became more important; fathers’ positions still mattered, especially for non-passers. This is evidence about an elite-recorded population, not a national peasant-to-official mobility rate. [PubMed](https://pubmed.ncbi.nlm.nih.gov/38236732/?utm_source=chatgpt.com)

**South Asia:** Descent-based marriage and support networks could coexist with changing occupations and economic fortunes. A caste-like order should therefore permit wealthy members of subordinated groups and impoverished members of prestigious groups rather than forcing wealth and rank to coincide. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjel.20171307)

**Northern Andes:** Boada’s research at El Venado, Colombia, investigates changing hierarchy through household, settlement, and burial evidence. Its value for TCE is precisely that status must be reconstructed from several domains rather than inferred from a single palace or rich grave. [University of Pittsburgh](https://sites.pitt.edu/~ccapubs/books/m017.html)

**Europe and the Russian Empire:** Landholding, corporate privilege, office, and peasant obligations created overlapping hierarchies. Russian emancipation also shows that removing personal restrictions and reorganizing land rights are distinct reforms. [Cambridge University Press](https://www.cambridge.org/core/books/abs/capitalism-socialism-and-serfdom/causes-of-slavery-or-serfdom-a-hypothesis/34F14D3F2B6F8C08374BE90E71054479?utm_source=chatgpt.com)

### 2.6 Industrial and modern societies: structural change is not the same as equal opportunity

Industrialization changes the number and kinds of jobs. Many children can leave farming without parental background becoming less consequential within the new occupational structure. Long and Ferrie’s historical Britain–United States comparison finds changing occupational mobility, but published critiques show that results depend importantly on farming categories and changing occupational composition. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.103.4.1109)

Modern societies likewise differ greatly within as well as between countries. African educational mobility varies sharply across regions; Indian mobility differs across social groups; Brazilian income-rank persistence remains substantial. These are reasons to model local opportunity structures, not attach one mobility coefficient to a national culture. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.3982/ECTA17018?utm_source=chatgpt.com)

---

## 3. Quantitative parameters and calibration targets

### 3.1 First separate the measurements

A **Gini coefficient** summarizes inequality in a specified distribution. Housing-area Gini, income Gini, consumption Gini, and net-wealth Gini are different observables.

An **intergenerational elasticity**, or IGE, commonly estimates:

\[
\log y\_{\text{child}}=a+\beta\log y\_{\text{parent}}+\epsilon.
\]

A **rank–rank slope** instead relates positions in the two generations’ distributions. Neither is a probability that a child inherits a parent’s class. Absolute mobility—such as earning more real income than one’s parents—is another quantity again. These distinctions are central to the mobility literature. [OUP Academic](https://academic.oup.com/ej/article/128/612/F404/5089531)

The confidence labels below concern suitability as historical calibration evidence: **high** for the stated, well-measured population; **medium** for informative but restricted or model-dependent evidence; **low** for particularly uncertain reconstructions. They do not imply universal applicability.

### 3.2 Small-scale societies: inequality and persistence

The following are **importance-weighted averages across material, embodied, and relational wealth**, not ordinary monetary-wealth statistics. Values are estimates ± **one standard error**, not cross-society ranges. Source: Borgerhoff Mulder et al. (2009), 21 populations. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

| Economic grouping in the study | Weighted Gini, dimensionless | Weighted intergenerational transmission coefficient | Confidence for transfer to TCE |
| --- | --- | --- | --- |
| Hunter-gatherer | 0.25 ± 0.04 | 0.19 ± 0.05 | Medium; small, heterogeneous sample |
| Horticultural | 0.27 ± 0.03 | 0.18 ± 0.04 | Medium |
| Pastoral | 0.42 ± 0.05 | 0.43 ± 0.06 | Medium |
| Agricultural | 0.48 ± 0.04 | 0.36 ± 0.05 | Medium |

Use these as **joint outcome checks**, not as coefficients applied to newborn agents. They describe historical and contemporary small-scale populations, not direct measurements of prehistoric ancestors.

### 3.3 Pre-industrial income inequality

These examples use the explicitly identified **2007 social-table reconstruction**, Table 2, preceding the expanded 2011 study. Values shown are its Gini2 estimate where available and Gini1 otherwise. Social tables infer distributions from estimated group sizes and incomes; apparent precision should not be mistaken for measurement certainty. [Munich Personal RePEc Archive](https://mpra.ub.uni-muenchen.de/5388/1/MPRA_paper_5388.pdf)

| Society and date | Estimated income Gini, 0–1 | Confidence | Important limitation |
| --- | --- | --- | --- |
| Roman Empire, 14 CE | 0.394 | Low–medium | Reconstructed social groups and incomes |
| England and Wales, 1688 | 0.450 | Medium | Historical social-table assumptions |
| Mughal India, approximately 1750 | 0.489 | Low | Sensitive to reconstruction method |
| Nueva España, approximately 1790 | 0.635 | Low | Very coarse underlying class division |
| China, approximately 1880 | 0.245 | Low | Particularly coarse grouping; not evidence of equal rights |

The associated **inequality possibility frontier** asks how much inequality is feasible given average income and an assumed subsistence floor. In a large population, its limiting form is approximately:

\[
G\_{\max}\approx1-\frac{s}{\bar y}.
\]

Thus, a low-income society can have a moderate Gini while most people remain near subsistence. For TCE, always inspect consumption adequacy alongside inequality. The frontier is an accounting benchmark conditional on its floor, not a guarantee that everyone survives. [Munich Personal RePEc Archive](https://mpra.ub.uni-muenchen.de/5388/1/MPRA_paper_5388.pdf)

### 3.4 Modern distributional anchors

These are **2021 estimates reported in the World Inequality Report 2022**, not current-2026 figures. Regional distributions include both within-country and between-country variation. They are not direct targets for an isolated town. [World Inequality Report 2022](https://wir2022.wid.world/executive-summary/)

| Observable | Value | Unit | Confidence |
| --- | --- | --- | --- |
| Global top 10% income share | 52% | Share of income | Medium–high; harmonized estimates |
| Global bottom 50% income share | 8.5% | Share of income | Medium–high |
| Global top 10% wealth share | 76% | Share of wealth | Medium; wealth measurement is harder |
| Global bottom 50% wealth share | 2% | Share of wealth | Medium |
| European top 10% income share | Approximately 36% | Regional income share | Medium–high |
| Latin American top 10% income share | Approximately 55% | Regional income share | Medium–high |
| Middle East and North African top 10% income share | Approximately 58% | Regional income share | Medium |

### 3.5 Mobility: useful anchors, but not interchangeable rates

| Population and study | Estimate | Exact meaning | Confidence and caution |
| --- | --- | --- | --- |
| England and Wales, wealth at death, 1858–2012; Clark and Cummins | 0.70–0.75 | Persistence parameter in their underlying wealth-transmission model | Medium; rare-surname/probate selection and model interpretation matter. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ecoj.12165?utm_source=chatgpt.com) |
| United States, Vosters’ father–son sample | IGE 0.439, SE 0.075 | Association of fathers’ and sons’ income in 415 pairs | Medium for this sample; not a universal US estimate. [OUP Academic](https://academic.oup.com/ej/article/128/612/F404/5089531) |
| Brazil; Britto et al., 2024 author version | Rank–rank slope approximately 0.55 | Ten parental income percentiles predict approximately 5.5 child percentiles | Medium–high; informal income partly modeled. [Atlas Mobilidade Social](https://atlasmobilidadesocial.org.br/website/wp-content/uploads/2025/05/2024_04_IGM_BFPSW.pdf) |
| India, sons born 1980–1989; Asher et al. | 37.1–37.2 | Expected education percentile for sons of fathers in the bottom half | Medium–high; partial-identification bounds, **not percentages upwardly mobile**. [Charlie Rafkin](https://www.charlierafkin.com/docs/anr_mobility.pdf) |
| Same Indian cohort, selected groups | Approximately 28.9 for Muslims; 41.3 for Forward/Others | The same bottom-half educational-rank measure | Group gaps are contextual outcomes, not innate group parameters. [Charlie Rafkin](https://www.charlierafkin.com/docs/anr_mobility.pdf) |
| African census comparison; Alesina et al. | Approximately 4% in South Sudan to 80% in South Africa | Primary completion among ages 14–18 whose parents did not complete primary school | Medium–high for sampled censuses; co-residence and historical coverage matter. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.3982/ECTA17018?utm_source=chatgpt.com) |
| United States, 1940 versus 1980s birth cohorts; Chetty et al. | Approximately 90% → 50% | Fraction earning more real income than their parents, assessed around age 30 | High for the defined exercise; **absolute**, not rank, mobility. [Opportunity Insights](https://opportunityinsights.org/paper/the-fading-american-dream/) |

### 3.6 What surname studies establish—and what they do not

Clark and Cummins find persistent differences among families identified through rare surnames over five generations. This is important evidence against assuming that initial advantages disappear after one or two generations. Their estimated underlying persistence is **0.70–0.75**. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ecoj.12165?utm_source=chatgpt.com)

However, the stronger claim of a nearly universal underlying mobility law is contested. Vosters tests a proposed explanation for the discrepancy between conventional and surname-based estimates and does not find the predicted convergence toward very high persistence. Group averages can also preserve differences associated with geography, networks, or group boundaries without measuring an individual’s transition probability. [OUP Academic](https://academic.oup.com/ej/article/128/612/F404/5089531)

**Recommendation:** Measure both lineage persistence and individual mobility. Never encode surname prestige as an inherited ability score. Let surnames identify families whose assets, networks, location, and institutional access have actual histories.

### 3.7 Causal controls: proposed sensitivity tests, not historical estimates

Most historical evidence identifies outcomes more reliably than portable annual behavioral probabilities. The following are **engineering experiment settings**. They should be replaced or narrowed when calibrating a specific institutional configuration.

| Control | Suggested test settings | Unit / interpretation | Evidence status |
| --- | --- | --- | --- |
| Inheritance concentration | Equal division; half to principal heir; all to principal heir | Allocation of the estate remaining after obligations | Authored legal alternatives, not universal frequencies |
| Enforceability | 0.1, 0.5, 0.9 | Probability of enforcement per eligible, detected case | Sensitivity settings only |
| Sharing obligation | 0, 0.25, 0.50 | Fraction of an explicitly defined distributable surplus | Sensitivity settings only |
| Endogamy preference | Odds multipliers 1, 5, 20 | Within-group versus otherwise comparable match | Sensitivity settings; distinguish preference from prohibition |
| Patronage in office selection | None, mixed, dominant | Weight of sponsorship relative to other selection criteria | Institutional variants |
| Exit environment | Accessible frontier; costly frontier; no accessible alternative | Real alternative livelihood opportunities | Derive costs from the map and economy |

Do not apply enforcement probabilities when a claim is legally inapplicable. Do not apply sharing to the household’s survival reserve unless the institution actually permits that demand.

---

## 4. Stylized facts a credible simulation should reproduce

These are **conditional validation targets**, not outcomes every generated world must reach.

| Pattern | Empirical anchor | What to test in TCE |
| --- | --- | --- |
| **Similar subsistence technologies can support different distributions.** | Archaeological relationships depend on land limitation and institutions, not agriculture alone. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12037039/) | Hold technology constant; vary tenure, exit, and governance. Outcomes should differ materially. |
| **Inherited advantages can persist for many generations without making every descendant rich.** | Rare-surname wealth differences remain detectable across five generations. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ecoj.12165?utm_source=chatgpt.com) | Track lineage means alongside within-lineage dispersion and downward mobility. |
| **Opening one route to office need not erase family advantage everywhere.** | Tang examination success gained importance while parental position still mattered in other pathways. [PubMed](https://pubmed.ncbi.nlm.nih.gov/38236732/?utm_source=chatgpt.com) | Competitive selection should weaken some patronage effects without deleting inherited preparation and connections. |
| **Economic and educational mobility can vary sharply within a polity.** | Kenya’s regional primary-completion mobility ranged roughly from 5% to 85% in the African study. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/full/10.3982/ECTA17018?utm_source=chatgpt.com) | Access to schools, jobs, transport, and networks should generate geographic differences. |
| **Legal emancipation changes behavior, not just a label.** | Russian emancipation was associated with substantial economic improvements. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.20160144) | Ending coercive obligations should affect labor allocation, investment, migration, and production. |
| **Absolute mobility can change without a matching change in relative opportunity.** | US children’s probability of exceeding parental real income declined from approximately 90% to 50% across cohorts. [Opportunity Insights](https://opportunityinsights.org/paper/the-fading-american-dream/) | Record living-standard improvement separately from rank movement. |
| **Occupational change can exaggerate apparent social fluidity.** | Historical mobility comparisons are sensitive to farming and occupational composition. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Faer.103.5.2003) | Compare raw occupation switching with mobility conditional on changing job availability. |

Add negative tests. A successful merchant should not automatically become legally noble. An impoverished titleholder should not automatically lose a title. An agent’s emancipation should not conjure land or savings. A large house should not automatically grant command over its occupants.

---

## 5. Modeling recommendation for TCE

### 5.1 Represent relationships first; derive class labels afterward

A compact conceptual schema is sufficient:

```
Person
    household, parents, partners, group memberships
    skills, health, occupation
    offices, individual rights, obligations
    reputation by audience

Household
    members, dependants
    inventories, productive assets, liabilities
    property and use-right claims

CorporateGroup
    membership and descent rules
    marriage and admission rules
    collective assets, privileges, obligations
    sanctions and internal decision procedures

Office
    jurisdiction, powers, remuneration
    eligibility, selection, term, succession
    records, personnel, accountability

PropertyClaim
    resource, claimant, type of right
    transfer rules, competing claims
    recognized authority and enforcement

DependencyRelationship
    claimant, subordinated person
    institutional basis, demanded obligations
    restrictions, enforcement, exit and release conditions
```

This is an architectural recommendation, not a claim that historical societies used these categories.

**Households** are useful economic units, but rights and welfare must also exist at the **person** level. Otherwise, household averages conceal unequal access among spouses, children, servants, and dependent laborers.

An enslaved person must remain an individual agent. A society’s recognized ownership claim can appear in historical accounting, but it should be stored as a coercive relationship—not by replacing that person with an inventory item. For analytical comparisons, distinguish ordinary productive assets from capitalized coercive claims.

### 5.2 Use a causal sequence for class formation

An illustrative, unscripted trajectory might be:

A household accumulates grain and livestock. It lends seed to neighbors. Some borrowers lose harvests and transfer land-use rights under prevailing rules. The lender hires workers and sponsors guards. Its allies obtain offices. A coalition then secures privileges or hereditary eligibility, and related households begin preferring marriages within that coalition.

At each step, a different outcome must remain possible: repayment, debt cancellation, refusal to recognize transfers, successful exit, electoral defeat, loss of guards’ loyalty, or an anti-monopoly rule.

The eventual labels—landlord, tenant, official lineage, privileged estate—should describe those relationships. They should not be hidden prerequisites for acquiring them.

A **class-conscious political group** is an additional outcome. Shared economic position creates potential common interests, but organization and identity should still depend on communication, leadership, and existing networks.

### 5.3 Separate economic accounting from influence

For each household, maintain a balance:

\[
\Delta A\_h =
\text{net production and earnings}
+\text{net transfers}
-\text{consumption}
-\text{asset losses}.
\]

Resolve this by asset and good, with debts and claims recorded separately. Avoid counting a producer’s output and its sale revenue as two additions to wealth.

Political influence can then draw on actual resources: paid followers, reciprocal obligations, recognized office, access to information, and support from institutions. Do not grant a free influence bonus simply because an analytical wealth classifier calls someone “upper class.”

Similarly, prestige should depend on the audience. Generosity can increase standing among recipients while reducing material wealth; conspicuous consumption can impress one group and offend another.

### 5.4 Make status visible without turning it into a universal costume tier

Research on sumptuary regulation across Asian, Latin American, Ottoman, and European settings shows that clothing and expenditure were contested expressions of identity and authority. Regulation extended beyond garments to banquets, festivities, and funerals. This supports a system of locally meaningful signs, permissions, and enforcement—not a universal equation of expensive clothing with noble status. [Cambridge University Press](https://www.cambridge.org/core/books/the-right-to-dress/EC71EEED0F80C4B4060862D9E5378D94?utm_source=chatgpt.com)

| Visible domain | Generate from | Useful gameplay consequences |
| --- | --- | --- |
| **Dress and adornment** | Available materials, expense, occupation, group affiliation, permitted insignia | Recognition, imitation, challenges, confiscation, reputational effects |
| **Housing** | Household size, wealth, tenure, labor access, climate, defensibility | Larger compounds, guest spaces, workshops, barriers, servant quarters |
| **Burial** | Family resources, ritual rules, claimed identity, obligations to the deceased | Funeral expenditure, monuments, redistribution, disputes |
| **Titles and precedence** | Office, recognition, lineage rules, ceremonial institutions | Seating order, forms of address, access, claims to command |

Do not equate burial expenditure with the deceased’s personal wealth. Model who pays and which goods are removed from use, redistributed, or retained.

For Unreal rendering, cultural building blocks can supply the visual vocabulary while simulated institutions determine who uses which markers and whether others accept their use.

### 5.5 Reproduce the archaeological observation process

The GINI project treats housing data as a comparative proxy requiring contextual interpretation. Archaeological sampling, contemporaneity, household definition, and building function matter. A settlement’s house-size distribution is not automatically its distribution of personal wealth. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/global-dynamics-of-inequality-gini-project-analysing-archaeological-housing-data/CBA474CDCB265DED23906F0A193BF7B3)

TCE has a major advantage: it can observe both the underlying society and a simulated archaeological sample.

Produce two outputs:

**True state:** individual and household resources, rights, consumption, office access, and dependency.

**Observable remains:** sampled house areas, construction materials, surviving graves, stored objects, and settlement layout.

Then test whether archaeological-style measurements recover the intended relationships. This will catch errors such as a seemingly egalitarian housing distribution concealing a large subordinated population living inside elite compounds.

### 5.6 Updates and scaling

For 10k–50k people, I recommend:

**Event-driven updates** for births, marriage, inheritance, debt default, admission, expulsion, office succession, emancipation, and law changes.

**Seasonal or annual economic summaries** for wealth distributions, rents, tax burdens, and dependence on wage labor. Daily life still changes the underlying inventories and relationships.

**Sparse networks** for patronage, kinship, assistance, and reputation. Maintain local audiences rather than comparing everyone with everyone.

**Cohort-based mobility statistics** using parents’ and children’s outcomes at comparable life stages, preferably smoothed over several years. Store enough history to distinguish temporary poverty from durable position.

Compute annual distributional statistics with sorting or aggregated histograms; there is no need to recalculate society-wide Ginis every simulation tick.

### 5.7 What to simplify in the first implementation

Implement actual inheritance, asset access, offices, patronage, and a small set of legal-right bundles before attempting a full taxonomy of named estates or castes.

A strong first version could support open membership, hereditary membership, sponsored admission, and restricted exit. More elaborate ritual precedence and multiple overlapping jurisdictions can come later.

Keep health, skill formation, and opportunity transmission explicit enough that disadvantaged childhoods affect subsequent outcomes. But do not introduce inherited “social worth,” lineage talent, or caste ability to force persistence.

Most importantly, **treat measured mobility coefficients as validation outputs** when individual inheritance and opportunity mechanisms are already simulated. Applying both explicit transmission and an additional fitted persistence coefficient risks counting the same process twice.

### 5.8 Existing models worth borrowing from

**Sugarscape and NetLogo’s wealth-distribution implementations** demonstrate bottom-up inequality from heterogeneous agents and resource landscapes, with Lorenz curves and Gini diagnostics. They are useful testbeds for accumulation and distributional measurement, but a wealth distribution alone is not a model of estates, caste, or legal dependency. [CCL](https://ccl.northwestern.edu/netlogo/models/WealthDistribution?utm_source=chatgpt.com)

**Powers and Lehmann (2014)** provide a directly relevant individual-based model connecting coordination benefits, leadership appropriation, demographic change, and exit costs. Borrow the feedback structure, not its generation-level reproduction process or an assumption that all settlements must follow its trajectory. [PubMed Central (PMC)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4132689/?utm_source=chatgpt.com)

**Dow and Reed (2013)** offer a formal model of the emergence of exclusionary property and insider–outsider divisions. It is especially useful for testing whether TCE can generate inequality through changing resource access without preassigning economic classes. [Simon Fraser University](https://www.sfu.ca/~gdow/download/jpe%202013.pdf)

---

## 6. Sources, datasets, and evidence limits

### Priority datasets

| Resource | Best use for TCE | Main limitation |
| --- | --- | --- |
| **GINI project, tDAR**; including “GINI Database All Records 20240721” | Housing-area distributions, site comparisons, archaeological calibration | Proxy quality and site coverage vary; preserve dataset version. [tDAR](https://core.tdar.org/project/496853/the-global-dynamics-of-inequality-gini-project) |
| **El Venado comparative archaeological dataset** | Joint analysis of household, burial, and material differentiation in Colombia | A specific regional trajectory, not a global template. [Comparative Archaeology Database](https://www.cadb.pitt.edu/boada/index.html) |
| **Clark–Cummins supplementary data** | Multigenerational wealth persistence and surname-based diagnostics | Probate coverage and selected rare surnames. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ecoj.12165?utm_source=chatgpt.com) |
| **Asher–Novosad–Rafkin replication materials** | Educational mobility, social-group comparisons, coarse-rank measurement | Education is not equivalent to wealth or legal status. [ICPSR](https://www.icpsr.umich.edu/sites/aea/view/studies/184504/versions/V1.0) |
| **Opportunity Insights research and accompanying data** | Absolute mobility and clearly defined parent–child comparisons | Modern US setting and specific income definitions. [Opportunity Insights](https://opportunityinsights.org/paper/the-fading-american-dream/) |
| **World Inequality Database / World Inequality Report** | Income and wealth shares; comparative modern distributions | Harmonization and imputation; regional totals are not settlement distributions. [WID - World Inequality Database](https://wid.world/methodology/) |

### Core scholarly reading

For emergence, begin with **Borgerhoff Mulder et al. (2009), “Intergenerational Wealth Transmission and the Dynamics of Inequality in Small-Scale Societies”**, and **Bogaard et al. (2025), “Labor, land, and the global dynamics of economic inequality.”** Together they connect persistence to both resources and institutions. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

For alternative mechanisms, pair **Boehm (1993)** on leveling with **Dow and Reed (2013)** on exclusion and **Powers and Lehmann (2014)** on leadership and exit. These should be treated as competing or complementary mechanisms, not a single universal origin story. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/204166?utm_source=chatgpt.com)

For mobility, read **Clark and Cummins (2015)** alongside **Vosters (2018)**, then compare **Wen, Wang, and Hout (2024)**, **Asher, Novosad, and Rafkin (2024)**, and **Alesina et al. (2021)**. The contrast between their measures is as important as their numerical findings. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/abs/10.1111/ecoj.12165?utm_source=chatgpt.com)

### Where evidence is thinnest

**Prehistoric mobility rates are poorly identified.** Archaeological inequality, prestigious childhood burial, and lineage continuity can suggest inherited advantage, but they do not directly provide population-wide transition matrices. Housing research should therefore constrain plausible outcomes rather than supply precise annual promotion rates. [Cambridge University Press](https://www.cambridge.org/core/journals/antiquity/article/global-dynamics-of-inequality-gini-project-analysing-archaeological-housing-data/CBA474CDCB265DED23906F0A193BF7B3)

**Historical records select the visible.** Elite epitaphs, probate inventories, and occupationally linked records do not represent all people equally. Their conclusions must remain attached to their recorded populations. The Tang study’s elite sample and the English probate study are especially clear examples. [PubMed](https://pubmed.ncbi.nlm.nih.gov/38236732/?utm_source=chatgpt.com)

**Mechanism identification remains contested.** The cereal-appropriability debate and the surname-mobility debate show why TCE should permit multiple causal pathways instead of installing a single scholarly hypothesis as a law of history. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/740225?utm_source=chatgpt.com)

**Final design recommendation:** Make assets, opportunity access, offices, enforceable obligations, and intergenerational transfers the causes. Make class labels, inequality statistics, and visible status hierarchies the consequences. That architecture lets TCE generate persistent aristocracies, fluid commercial elites, closed descent groups, relatively equal farming communities, and institutional reversals without prescribing any of them in advance.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab9282f-6508-83ea-95d3-d9d53a13cc77)
