# How leaders and elites emerge: a simulation-ready report for TCE

## Executive recommendation

**Do not make “elite” an intrinsic character class or the automatic result of a high wealth-and-charisma score.** Model three distinct developments:

**Personal influence → organized following → durable control of positions and resources.**

Each transition should be conditional and reversible. Someone can become an influential organizer without acquiring coercive authority; a hereditary officeholder can command resources without being admired; and an elite family can survive the replacement of its individual leaders. These distinctions are central to the anthropological and political-economic evidence. [UR Scholarship Repository](https://scholarship.richmond.edu/jepson-faculty-publications/213/)

For TCE, use the following operational definitions:

| Concept | What the simulation should measure |
| --- | --- |
| **Prestige** | Voluntarily conferred respect, attention, and willingness to learn from someone. |
| **Dominance** | Capacity to obtain compliance through credible threats or imposed costs. |
| **Leadership** | Influence over a particular collective decision or coordinated activity. |
| **Office** | A socially recognized position with powers, duties, selection rules, and a jurisdiction. |
| **Elite membership** | Durable privileged access to consequential resources, decisions, or recruitment channels. |
| **Notability** | A presentation-layer judgment that someone is consequential enough to display—not a source of power itself. |

The central modeling distinction is between **being followed**, **being obeyed**, and **controlling who gets to lead next**.

---

## 1. Mechanisms: implementable causal rules

### 1.1 Leadership begins with problems that benefit from coordination

Collective work creates opportunities for organizers: arranging labor, resolving disputes, choosing a route, distributing tasks, or negotiating with another group. Leadership becomes attractive when its contribution exceeds its costs, but those costs can include both compensation and the danger of empowering an exploiter. Hooper, Kaplan, and Boone formalize this trade-off in a public-goods model; hierarchy is one possible solution, not an inevitable outcome of increasing population. [University of New Mexico](https://www.unm.edu/~phooper/HooperKaplanBoone2010_Leadership.pdf)

**TCE rule:** Generate leadership opportunities from actual tasks. Agents propose plans; others evaluate the proposer’s relevant competence, reliability, relationships, and proposed division of benefits.

Keep authority initially task-specific. A successful irrigation coordinator should not automatically control marriages, military decisions, and religious practice.

Crucially, distinguish **leadership effectiveness from surplus appropriation**. In experiments among Tsimane communities in Bolivia, leaders helped coordinate cooperation but did not take larger shares of the proceeds. Leadership characteristics also worked differently depending on whether participants had previously cooperated. [scholarship.richmond.edu](https://scholarship.richmond.edu/jepson-faculty-publications/213/?utm_source=chatgpt.com)

**Implementation consequence:** Reward successful coordination first with task completion, trust, and future invitations. Material privileges require an additional bargaining process or institutional rule.

### 1.2 Prestige and dominance are separate routes, not opposite ends of one scale

Prestige theory links freely granted deference to perceived useful knowledge and skill. Experimental work also distinguishes prestige from dominance: both can predict influence, although admiration and intimidation have different social consequences. These routes can coexist within one person. [Joe Henrich](https://henrich.fas.harvard.edu/publications/evolution-prestige-freely-conferred-deference-mechanism-enhancing-benefits)

**TCE rule:** Store admiration and fear separately, by audience and domain.

A renowned healer may receive voluntary deference. A guard commander may receive obedience because refusal is dangerous. A successful defender may acquire both.

Do not infer competence from status without error. Observers should sometimes mistake confidence, inherited symbols, ceremonial success, or connections for ability. Update reputations through observed outcomes, but let attribution be noisy: followers may credit the leader for favorable weather or excuse failure as sabotage.

**Important distinction:** A frightened subject can comply while disliking the ruler and preparing to defect. Compliance must not automatically increase loyalty.

### 1.3 Egalitarianism requires mechanisms that restrain would-be rulers

Boehm’s “reverse dominance” account emphasizes collective resistance to domineering individuals. Woodburn’s work connects egalitarian arrangements to features such as immediate access to subsistence, mobility, and limited dependence on another person’s productive assets. Egalitarian societies need not lack influential individuals; they can prevent influence from becoming entrenched command. [JSTOR](https://www.jstor.org/stable/2743665)

**TCE rule:** Give agents responses to overreach: refusal, ridicule, demands for sharing, collective criticism, withdrawal of cooperation, replacement, or departure. More severe sanctions should require stronger justification and coalition support.

Whether these responses work depends on:

* Accessible alternatives to the leader’s land, food, employment, or protection.
* The ability of dissatisfied people to communicate and coordinate.
* Norms defining legitimate leadership and unacceptable appropriation.

These are not merely “low hierarchy” modifiers. They are actions with costs and consequences.

A village with independent household stores and accessible land should constrain an aspiring monopolist differently from a settlement whose population depends on one guarded granary.

### 1.4 Big men build personal followings through costly relationships

Sahlins’s classic contrast distinguishes the **big man**, whose position depends heavily on personal achievement and cultivated support, from the **chief**, whose recognized position is embedded in an enduring political structure. These are ideal types drawn from Melanesian and Polynesian comparisons, not universal stages or exhaustive descriptions of either region. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/poor-man-rich-man-bigman-chief-political-types-in-melanesia-and-polynesia/E392602EF37440C9B50A3FDD10AE2A68)

**TCE rule:** Let ambitious individuals invest in hosting, gifts, dispute settlement, protection, introductions, and collective projects. Such investments can recruit followers, but also create obligations.

The budget constraint matters:

\[
\text{gifts}+\text{hosting}+\text{retainer costs}+\text{promised assistance}
\leq \text{available resources and credit}.
\]

A successful organizer can become overextended. Missed assistance, unfair distributions, or excessive demands can dissolve a following.

Hayden’s aggrandizer framework makes competitive feasting a possible route to inequality. Treat this as a hypothesis about particular settings—not a rule that feasts necessarily create debt or hereditary elites. [Cambridge University Press](https://www.cambridge.org/core/books/power-of-feasts/domesticating-plants-and-animals-for-feasts/9FC3ADF0362BBCF050B9BB810CB61D4E)

**Implementation consequence:** A feast can express solidarity, repay obligations, recruit labor, compete for prestige, or establish dependency. Its consequences should depend on the local institution and participants’ interpretations.

### 1.5 Patronage converts resources into conditional support

Scott’s Southeast Asian research describes patron–client relations as asymmetric but reciprocal: patrons provide access, protection, assistance, or insurance; clients provide labor, services, political support, and deference. Patronage is therefore neither ordinary friendship nor simply naked coercion. [University of Texas at Austin](https://www.la.utexas.edu/users/chenry/pmena/coursemats/2009/Scott-1972-clientelism.pdf)

**TCE rule:** Represent patronage as a directed, multiplex relationship containing:

`benefits supplied`, `benefits promised`, `obligations`, `dependency`, `trust`, and `alternatives`.

A patron’s leverage should increase when assistance is valuable and substitutes are scarce. It should weaken when clients gain independent income, alternative protection, competing patrons, or collective bargaining power.

Avoid the equation “gift received = permanent loyalty.” Clients can accept aid opportunistically, divide their commitments, renegotiate, or defect.

Allow **brokers** to emerge between groups. A person who connects farmers to a merchant, villagers to a court, or recruits to a commander may have considerable influence without being the richest actor. Brokerage should come from actual control over introductions, information, or access—not a generic network-centrality bonus.

### 1.6 Durable elites require advantages that can be preserved and transmitted

Comparative small-scale-society research finds stronger intergenerational persistence where important wealth consists of material assets such as land and livestock. This is a statistical association involving institutions and family circumstances, not a biological explanation of elite status. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

**TCE rule:** Track the properties of assets that make political conversion possible:

| Property | Political implication to model |
| --- | --- |
| Storable | Surplus can finance followers between harvests or during emergencies. |
| Excludable | An owner can deny access or impose conditions. |
| Transferable | Assets can purchase support, repay debts, or endow successors. |
| Defensible | Control can be maintained at an affordable enforcement cost. |
| Difficult to replace | Dependents have fewer credible alternatives. |
| Inheritable | Advantages survive the original accumulator. |

Control of production alone is insufficient: someone must recognize or enforce the claimant’s rights.

Mayshar, Moav, and Pascali emphasize agricultural **appropriability**, especially cereals, rather than productivity alone. However, a replication reports sensitivity to specification, coding, sample restrictions, and outliers. Use appropriability as a mechanism to test, not “grain automatically creates states.” [RCNi Company Limited](https://www.journals.uchicago.edu/doi/full/10.1086%2F718372)

### 1.7 Officeholding can create the conditions for future elite recruitment

Power can reproduce itself through appointment rights, controlled training, privileged information, marriage connections, property allocation, and restrictions on eligibility. Smith’s analysis of Aztec inequality emphasizes exploitation and the restriction of access to opportunities as mechanisms sustaining durable class differences. [Arizona State University](https://asu.elsevierpure.com/en/publications/durable-inequality-in-aztec-society/)

**TCE rule:** Officeholders can attempt to alter institutions in their favor, but proposals require recognition and enforcement. Examples include reserving an office for certain lineages, appointing loyal collectors, restricting access to military training, or converting temporary resource management into hereditary control.

Do not make these changes automatic. Other officeholders, clients, assemblies, or neighboring communities may resist.

The causal direction also runs from office to family advantage. Dal Bó, Dal Bó, and Snyder find that longer congressional tenure in the United States increases the likelihood that relatives subsequently enter Congress—not merely that politically talented families repeatedly win independently. [OUP Academic](https://academic.oup.com/restud/article-abstract/76/1/115/1574319)

### 1.8 Entrenchment depends on competition, not only public approval

In Sierra Leone, Acemoglu, Reed, and Robinson examine competition among ruling families eligible for paramount chieftaincy. Less competition is associated with worse development outcomes, while some measures of civic participation and respect for chiefs are higher. Organized civil society can therefore be captured rather than necessarily constraining elites. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/674988)

**TCE rule:** Separate:

**popular esteem**, **participation in organizations**, **freedom to challenge**, and **effective accountability**.

A crowded ceremony or large association membership should not prove that leadership is voluntary. Attendance may reflect dependence or pressure.

External intervention must also be able to change elite recruitment. Conquerors or colonial administrations can recognize particular families, create offices, or strengthen formerly limited leaders. Elite formation is not always an entirely internal process. [MIT Open Scholarship](https://dspace.mit.edu/handle/1721.1/96712)

### 1.9 Succession is a reassembly of support, not a complete transfer of a person

For implementation, distinguish four things a successor might receive:

**Property; eligibility; organizational control; personal relationships.**

The first three may transfer under institutional rules. The fourth should be reconsidered by the people involved.

A capable predecessor can leave an incompetent heir with substantial inherited power. Conversely, a successor can lose personal clients while retaining an enforceable office.

Rotation also need not mean frequent elections. In Haudenosaunee governance, chiefs can hold lifetime positions while clan mothers retain authority to remove them. Thus long tenure and accountability can coexist. [National Museum of the American Indian](https://americanindian.si.edu/sites/1/files/pdf/education/HaudenosauneeGuide.pdf)

**TCE rule:** On death, retirement, removal, or term expiration, transfer recognized claims, then trigger a succession process: endorsement, bargaining, rival candidacies, client recommitment, and possible disputes.

---

## 2. Parameters and quantitative calibration

### Evidence classification

**High confidence** below means well-supported for the particular measurement or institutional rule—not universally applicable. **Moderate confidence** indicates useful observational evidence with substantial sampling or interpretation limits. **Design prior** means a proposed simulation setting, not an empirical estimate.

### 2.1 How large are elites?

**There is no defensible universal “elite percentage.”** Hereditary estates, active officeholders, wealthy households, and influential intermediaries are different populations.

| Society and observation | Value | Denominator and interpretation | Confidence and source |
| --- | --- | --- | --- |
| Yacapixtla, central Mexico, early colonial census | **1.0%** | Noble persons / total population | Moderate; Smith & Hicks 2016 |
| Huitzillan, same source context | **1.4%** | Same | Moderate |
| Quauhchichinollan, same source context | **1.8%** | Same | Moderate |
| Basin of Mexico, broader reconstruction | **About 2%** | Noble population, not active rulers | Moderate–low |
| Some Huexotzinco communities | **4–14%** | Reported noble population; discrepancy with other regions remains unresolved | Moderate–low |
| Japan, 1876 | **About 5%** | Former samurai population, **including family members**, not just administrators or military leaders | Moderate–high |
| Haudenosaunee Grand Council | **50 positions** | Institutional seats, **not a population percentage**; excludes other consequential actors such as clan mothers | High for institutional description |

The Mexican estimates come from Smith and Hicks’s *Inequality and Social Class in Aztec Society*, which explicitly describes the evidence as fragmentary. The Japanese estimate is documented by Basco and Tang; the council structure is described by the Smithsonian’s National Museum of the American Indian. [Academia](https://www.academia.edu/26633074/_Inequality_and_Social_Class_in_Aztec_Society_Smith_and_Hicks_2016_)

These figures should calibrate **specific institutional configurations**, not supply a global quota. In the Aztec case, many nobles did not themselves hold powerful positions, while some merchants and other commoners possessed wealth and influence. [Academia](https://www.academia.edu/26633074/_Inequality_and_Social_Class_in_Aztec_Society_Smith_and_Hicks_2016_)

For a hypothetical 10,000-person TCE polity, a 2% hereditary estate would contain **200 people including dependents**. It would not imply 200 active political leaders.

### 2.2 Intergenerational persistence and inequality

Borgerhoff Mulder and colleagues compare **21 populations**. Their weighted measures combine material, relational, and embodied wealth; the following values are **mean ± standard error**, not universal ranges. Both measures are dimensionless. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

| Economic system in the sample | Intergenerational association, β | Weighted wealth Gini | Confidence for extrapolation |
| --- | --- | --- | --- |
| Hunter-gatherer | **0.19 ± 0.05** | **0.25 ± 0.04** | Moderate |
| Horticultural | **0.18 ± 0.04** | **0.27 ± 0.03** | Moderate |
| Pastoral | **0.43 ± 0.06** | **0.42 ± 0.05** | Moderate |
| Agricultural | **0.36 ± 0.05** | **0.48 ± 0.04** | Moderate |

Material-wealth persistence is particularly strong in the pastoral and agricultural samples: **β ≈ 0.67 and 0.55**, respectively. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

**How to use this:** Compare these associations with simulated parent–adult-child outcomes. **β is not the fraction of property inherited.** It incorporates multiple transmission processes. Do not impose β mechanically on offspring wealth while also simulating inheritance, schooling, marriage, and patronage; that would risk counting the same persistence twice. These Ginis are also not directly interchangeable with modern income Ginis. [Max Planck Evolutionary Anthropology](https://www.eva.mpg.de/documents/AAAS/Borgerhoff-Mulder_Intergenerational_Science_2009_2189437.pdf)

### 2.3 Leadership, networks, and rotation

| Quantity | Empirical value | Appropriate use | Confidence and source |
| --- | --- | --- | --- |
| Prestige–dominance correlation in one group-task study | **r = 0.01** | Supports keeping the dimensions separate | High within study; limited external generalization |
| Dominance / prestige correlations with perceived influence | **r = 0.68 / 0.57** | Validation of perceived rank, not causal coefficients | Same |
| Dominance / prestige correlations with behavioral influence | **r = 0.17 / 0.17** | Demonstrates that perceived status and demonstrated influence differ | Same; Cheng et al. 2013, **177 analyzed participants** |
| Cooperative-network mean degree across studies discussed in Tsimane research | **3.4–13.1 nominated ties/person** | Scale of selected cooperation relationships—not total acquaintances or all political supporters | Moderate |
| Oromo Gadaa leadership rotation | **8 years** | A historically grounded institutional configuration, not a generic leader tenure | High for the named system |
| Haudenosaunee chiefly tenure | **Lifetime, subject to removal** | Separate tenure duration from recallability | High for institutional description |

Sources: Cheng and colleagues; von Rueden and colleagues; Borana University’s account of Gadaa; Smithsonian institutional description. [Joe Henrich](https://henrich.fas.harvard.edu/sites/g/files/omnuum5811/files/henrich/files/cheng_et_al_2013.pdf)

The eight-year Tsimane longitudinal study is especially useful for dynamics: cooperative relationships and status influenced one another, but status effects were smaller than important kinship and reciprocity effects. It did not find a simple universal process of high-status individuals exclusively choosing one another. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/vonruedenetal2019.pdf)

### 2.4 Proposed TCE starting priors—not research estimates

The literature does **not** identify universal prestige-decay rates, gift-to-loyalty conversion factors, or annual probabilities of elite replacement. Use explicit sensitivity ranges instead.

| Implementation parameter | Initial experimental range | Units | Status |
| --- | --- | --- | --- |
| Routine reconsideration of political commitments | **7–30** | Simulated days | Design prior |
| Candidates actively considered in an ordinary local decision | **4–12** | Persons, plus an outside option | Design prior |
| Event learning gain for an uncertain reputation | **0.02–0.20** | Fractional update per informative event | Design prior |
| Unreinforced reputation half-life | **0.5–5** | Simulated years | Design prior; vary by domain |
| Personal-obligation memory horizon | **1–10** | Simulated years | Design prior; legal debts use actual rules |
| Average explicitly maintained relationship records | **16–64** | Directed records/person | Engineering prior, not a social-network law |
| Succession reconsideration | **On every relevant succession** | Event trigger | Architectural recommendation |

Do not apply these ranges to everything. A broken appointment and a remembered rescue should have different persistence. An inherited title should not evaporate because a reputation timer expires.

---

## 3. Variation across societies, regions, and economic settings

The useful distinctions are **asset structure, recruitment rules, outside options, and organizational scale**, not a single progression from “primitive” to “advanced.”

| Setting | Characteristic route or constraint | Implication for TCE |
| --- | --- | --- |
| **Immediate-return foragers**, including African ethnographic cases | Influence can coexist with sharing demands, mobility, and resistance to command. | Skilled organizers can remain influential without acquiring hereditary rights. [JSTOR](https://www.jstor.org/stable/2801707) |
| **Delayed-return foraging settings** | Investment in productive assets and future returns changes dependence; “forager” is not synonymous with equal power. | Model assets and access rules rather than attaching egalitarianism to the subsistence label. [JSTOR](https://www.jstor.org/stable/2801707) |
| **Early farming communities** | Storage, productive investments, feasting, and enforceable claims offer possible routes to differentiation; no single route is established as universal. | Public granaries and private granaries should produce different political opportunities despite identical crop technology. [Cambridge University Press](https://www.cambridge.org/core/books/power-of-feasts/domesticating-plants-and-animals-for-feasts/9FC3ADF0362BBCF050B9BB810CB61D4E) |
| **Melanesian big-man configurations** | Personal achievement and cultivated followings are central. | Support requires continuing investment; the death of an organizer need not produce orderly hereditary succession. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/poor-man-rich-man-bigman-chief-political-types-in-melanesia-and-polynesia/E392602EF37440C9B50A3FDD10AE2A68) |
| **Polynesian chiefly configurations** | Recognized rank and office structure precede particular occupants. | Separate personal reputation from powers attached to position. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/poor-man-rich-man-bigman-chief-political-types-in-melanesia-and-polynesia/E392602EF37440C9B50A3FDD10AE2A68) |
| **Pre-industrial central Mexico** | Hereditary noble status existed alongside economically and politically consequential non-nobles. | Legal rank, wealth, and actual power should overlap imperfectly. [Academia](https://www.academia.edu/26633074/_Inequality_and_Social_Class_in_Aztec_Society_Smith_and_Hicks_2016_) |
| **Oromo Gadaa; Haudenosaunee institutions** | Rotation, generational organization, selection, and recall provide alternatives to unconstrained hereditary rule. | Sophisticated checks on leadership do not require industrialization. Track influential selectors as well as officeholders. [Bahir Dar University](https://bru.edu.et/the-gadaa-system/) |
| **Rural Southeast Asia under commercialization and colonial change** | Patronage relationships changed as markets and administrative structures altered bargaining positions and obligations. | New markets can undermine some patrons while strengthening others; modernization need not simply abolish patronage. [Duke University Press](https://read.dukeupress.edu/journal-of-asian-studies/article/32/1/5/330131/The-Erosion-of-Patron-Client-Bonds-and-Social) |
| **Industrial transformation: Meiji Japan** | The conversion of samurai stipends into bonds illustrates how old privileges could become financial assets. | Institutional abolition can change the form of advantage rather than erase it. [Diposit Digital UB](https://diposit.ub.edu/bitstreams/353f3fc4-0bba-437d-a429-b127e6ec6ca9/download) |
| **Modern electoral institutions: United States** | Holding office can generate subsequent advantages for relatives. | Competitive selection and dynastic persistence are compatible outcomes. [OUP Academic](https://academic.oup.com/restud/article-abstract/76/1/115/1574319) |

Gender, age, descent, and household position must enter through **opportunities and institutions**, not hardcoded natural leadership ability. A system that only searches for male officeholders would miss the consequential selection and removal powers of Haudenosaunee clan mothers. [National Museum of the American Indian](https://americanindian.si.edu/sites/1/files/pdf/education/HaudenosauneeGuide.pdf)

---

## 4. Stylized facts a correct simulation should reproduce

These are validation targets for distributions of outcomes—not requirements that every settlement exhibit every pattern.

| Target pattern | Observable simulation test |
| --- | --- |
| **Leadership without enrichment** | Some effective coordinators retain ordinary material shares; leadership and extraction are not identical. This matches the Tsimane experimental counterexample. [scholarship.richmond.edu](https://scholarship.richmond.edu/jepson-faculty-publications/213/?utm_source=chatgpt.com) |
| **Several routes to influence** | Admired experts and feared enforcers can both affect decisions, with different loyalty and approval profiles. [Joe Henrich](https://henrich.fas.harvard.edu/sites/g/files/omnuum5811/files/henrich/files/cheng_et_al_2013.pdf) |
| **Wealth, rank, and office overlap imperfectly** | Produce wealthy non-officeholders, low-wealth respected leaders, and titled people with limited practical influence. The Aztec evidence is a useful calibration case. [Academia](https://www.academia.edu/26633074/_Inequality_and_Social_Class_in_Aztec_Society_Smith_and_Hicks_2016_) |
| **Status and cooperation coevolve** | Useful relationships can raise standing, and standing can attract cooperation, without every network becoming an exclusive elite club. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/vonruedenetal2019.pdf) |
| **Accountability is not tenure length** | A lifetime office can remain removable; a frequently replaced officeholder can belong to an enduring ruling family. [National Museum of the American Indian](https://americanindian.si.edu/sites/1/files/pdf/education/HaudenosauneeGuide.pdf) |
| **Institutional reform need not eliminate old advantages** | Formerly privileged households may convert claims into capital and remain influential after offices or estates disappear. [Diposit Digital UB](https://diposit.ub.edu/bitstreams/353f3fc4-0bba-437d-a429-b127e6ec6ca9/download) |
| **Public participation is not necessarily constraint on rulers** | High participation can coexist with restricted competition and elite-controlled organizations. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/674988) |

### Measure turnover at four levels

TCE should report separate measures for **persons, households or lineages, factions, and institutional arrangements**.

For example, define annual personal turnover as:

\[
T\_{\text{person}}
=
\frac{\text{new people entering the tracked positions during the year}}
{\text{positions at the start of the year}}.
\]

Report newly created positions separately so institutional expansion is not confused with replacement. Also distinguish deaths, voluntary retirement, scheduled rotation, dismissal, electoral defeat, expulsion, and violent removal.

To measure concentration without imposing a fixed “top 1%,” use a proposed diagnostic:

\[
N\_{\mathrm{effective},d}
=
\frac{\left(\sum\_i I\_{i,d}\right)^2}
{\sum\_i I\_{i,d}^{\,2}},
\]

where \(I\_{i,d}\) is nonnegative influence over decision domain \(d\). Equal influence gives a large effective count; concentration gives a small one. Treat a zero-influence domain as undefined, not zero elites.

Calculate this separately for military mobilization, resource allocation, dispute resolution, appointments, and other consequential decisions.

---

## 5. Recommended agent and institution model

### 5.1 Minimal state

Use existing TCE skills, personality, wealth, kinship, and memory systems rather than duplicating them inside a separate “politics” module.

| Entity | Additional state needed |
| --- | --- |
| **Person** | Domain-specific reputation; willingness to lead; recognized positions; current commitments; known candidates and alternatives |
| **Relationship** | Trust, admiration, fear, obligations, dependency, recent assistance, conflict, and confidence in remembered information |
| **Household or lineage** | Shared property, succession claims, marriage connections, recognized membership, and collective commitments |
| **Organization** | Members, leaders, resources, communication channels, internal selection rules, and decision procedure |
| **Office** | Jurisdiction, powers, duties, revenue rights, eligibility, selection, tenure, removal, and succession |
| **Institutional rule** | Who recognizes it, who benefits, who enforces it, and how it may be challenged or amended |

A person may simultaneously belong to a lineage, work crew, religious association, militia, and trading partnership. Commitments can conflict.

### 5.2 Following should emerge from locally perceived alternatives

For voluntary support, a useful **proposed** decision score is:

\[
U\_{i\rightarrow j,d}
=
\widehat B\_{ij,d}
+\widehat A\_{ij}
+\widehat L\_{ij,d}
-\widehat X\_{ij,d}
-\widehat K\_{ij,d}.
\]

Here:

* \(\widehat B\): expected collective benefits associated with the leader.
* \(\widehat A\): expected personal assistance or access.
* \(\widehat L\): value placed on legitimate procedure, identity, or obligations.
* \(\widehat X\): expected extraction.
* \(\widehat K\): participation costs and risks.

Convert these components into a consistent utility scale. Choose among known candidates, existing arrangements, independent action, and exit using a noisy rule such as softmax—not perfect maximization.

The hats matter: agents act on estimates, rumors, and selective experience, not global ground truth.

**Compute coercive compliance separately**, by comparing the perceived consequences of obeying, resisting, hiding, negotiating, and leaving. This preserves the difference between a loyal supporter and someone obeying under threat.

### 5.3 Mobilizable support must respect capacity and conflicting commitments

A starting estimate of usable power is:

\[
C\_{j,d}
=
C^{\mathrm{direct}}\_{j,d}
+
\sum\_i p\_{i\rightarrow j,d}\,a\_{i,d},
\]

where \(a\_{i,d}\) is the capacity agent \(i\) can contribute and \(p\) is their estimated probability of honoring the commitment.

But this is only an estimate. When action occurs, resolve commitments once. A farmer cannot provide the same labor day to two patrons, and a soldier cannot fight simultaneously for two rival coalitions.

Support also needs food, equipment, transport, communication, and organization. Wealth does not instantly become an army; followers do not instantly become effective force.

### 5.4 Create offices from recurrent problems and political bargains

Do not unlock “chiefdom” at a population threshold.

Instead, let agents propose institutional changes when existing arrangements repeatedly fail or when they see an advantage in changing them. The authored components might include a seasonal coordinator, dispute mediator, storekeeper, tax collector, council seat, militia captain, or ritual custodian.

A proposed office needs a bundle of rules:

**Who selects it? What may it do? How is it supported? Who can remove it? What happens when it becomes vacant?**

Competing bundles should be possible. A recurring storekeeping problem could produce:

* A rotating custodian accountable to households.
* A permanent official appointed by a council.
* A hereditary steward.
* A private warehouse owner with dependent clients.

These are alternative simulation outcomes, not four compulsory stages.

Increasing workload can motivate delegation, but delegation should introduce its own problems: monitoring costs, information loss, favoritism, and local power bases.

### 5.5 Make succession operationally explicit

At succession:

1. Transfer property and recognized institutional claims under the applicable rules.
2. Identify eligible candidates and the people or organizations entitled to decide.
3. Reevaluate endorsements, appointments, and personal commitments.
4. Resolve competing claims through the existing bargaining and enforcement systems.

This supports several different outcomes: orderly inheritance, appointment of a more capable relative, adoption into a house, council selection, split followings, institutional reform, or conflict.

Do not copy the predecessor’s full reputation into the successor. At most, inherited association supplies a prior expectation that subsequent behavior can confirm or contradict.

### 5.6 Computational implementation for 10k–50k agents

Use sparse relationships and event-driven updates.

At **50,000 agents × 32 directed relationship records**, the graph contains **1.6 million records**. At an illustrative **32–64 bytes per record**, raw edge payload is approximately **51–102 MB**, before indexing, allocator overhead, and event histories. This is a storage calculation, not a performance benchmark.

Avoid uniform hard degree caps. Ordinary agents can have small active neighborhoods; unusually connected leaders and institutions need variable-sized collections or aggregated memberships.

Recommended scheduling:

| Timescale | Political work |
| --- | --- |
| On meaningful interaction | Update assistance, obligations, reputation evidence, and grievances |
| Weekly or monthly | Reconsider a limited set of commitments and alternatives |
| On collective task | Select or recognize coordinators and resolve contributions |
| On vacancy, death, removal, or term end | Run succession or recruitment |
| On institutional proposal | Evaluate recognition, opposition, and enforceability |

Do not recompute global network centrality every day. Candidate discovery can use local ties, organizational membership, public reputation, and introductions.

### 5.7 Notables should be a derived view

Select displayed notables from actual consequential activity: office powers, resources controlled, mobilizable support, brokerage, and domain-specific influence.

Maintain local and polity-wide views. A village mediator can matter locally without entering the realm-wide list.

Use display hysteresis to avoid constant churn, but **never grant powers because the interface selected someone as notable**. Include rivals, dissidents, and influential selectors—not just incumbents.

### 5.8 Existing models worth adapting

| Model | What to borrow | What it does not solve |
| --- | --- | --- |
| **Hooper, Kaplan & Boone 2010, “A theory of leadership in human cooperative groups”** | The trade-off between coordination benefits, monitoring costs, and a leader’s compensation | Not a complete model of hereditary estates, recruitment, or state formation. [University of New Mexico](https://www.unm.edu/~phooper/HooperKaplanBoone2010_Leadership.pdf) |
| **Gavrilets, Auerbach & van Vugt 2016, “Convergence to consensus in heterogeneous groups and the emergence of informal leadership”** | Heterogeneous influence and emergent informal leadership during collective decisions | Consensus influence does not by itself establish property rights or dynasties. [PubMed](https://pubmed.ncbi.nlm.nih.gov/27412692/) |
| **von Rueden et al. 2019, cooperation–status network analysis** | Jointly updating relationships and status rather than treating either as fixed | A statistical model of particular observed networks, not a universal behavioral law. [Gurven Lab](https://gurven.anth.ucsb.edu/sites/secure.lsit.ucsb.edu.anth.d7_gurven/files/sitefiles/papers/vonruedenetal2019.pdf) |

For calibration, run matched worlds with inheritance, exit, patron budgets, eligibility restrictions, and collective resistance alternately enabled or disabled. Those ablations will reveal whether elite persistence comes from the intended mechanisms or an accidental positive-feedback loop.

---

## 6. Sources, datasets, and limits of the evidence

### Most useful datasets

| Resource | Use for TCE | Main limitation |
| --- | --- | --- |
| **D-PLACE**, introduced by Kirby et al. 2016 | Cross-cultural combinations of political organization, stratification, inheritance, subsistence, and environment | Ethnographic observations are situated in particular places and periods; societies are not statistically independent. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391) |
| **Seshat Global History Databank** | Historical institutional configurations and trajectories; use publication-specific replication datasets for reproducible calibration | Coding uncertainty and temporal aggregation; not an individual-level elite census. The official data page currently distinguishes replication datasets, snapshots, and live API data. [Seshat Databank](https://seshatdatabank.info/data) |
| **Archigos** | Leader spells and modes of entering and leaving national leadership | Measures apex leaders, not the full elite population or informal local leadership. [Ksgleditsch](https://ksgleditsch.com/archigos.html) |
| **Tsimane cooperation/status data and supplementary materials** | Longitudinal network and status validation | Specific communities and predominantly men’s measured networks. [Figshare](https://rs.figshare.com/articles/journal_contribution/Supplementary_Material_for_The_dynamics_of_men_s_cooperation_and_social_status_in_a_small-scale_society_/8953295) |
| **Garfield, Syme & Hagen’s cross-cultural leadership study** | Compare leadership dimensions across **59 mostly nonindustrial populations** | Ethnographic reporting and coding are uneven; not a representative census of all leaders. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S109051382030091X) |

### What is relatively secure

The strongest basis for TCE is the **separation of mechanisms**: coordination, prestige, coercion, patronage, institutional authority, and family transmission. Cross-cultural leadership research supports treating leadership as variable across domains and social arrangements rather than as one fixed human role. [Springer](https://link.springer.com/article/10.1007/s12110-019-09338-4)

### What remains thin or contested

**Universal numerical conversion rates are missing.** The sources assembled here do not establish a general percentage of wealth converted into followers, a universal reputation half-life, or a background annual hazard of elite replacement. Those should remain explicit calibration parameters.

**Origins are harder to establish than maintenance.** Feasting and agricultural appropriability offer plausible mechanisms, but evidence that a mechanism can sustain hierarchy is not proof that it caused every historical hierarchy to originate. The appropriability debate illustrates the importance of testing alternative specifications and explanations. [Cambridge University Press](https://www.cambridge.org/core/books/power-of-feasts/domesticating-plants-and-animals-for-feasts/9FC3ADF0362BBCF050B9BB810CB61D4E)

**Measurement changes the apparent elite.** Estate membership, officeholding, household wealth, burial treatment, and actual influence cannot be substituted for one another without justification. The Aztec evidence makes this particularly clear. [Academia](https://www.academia.edu/26633074/_Inequality_and_Social_Class_in_Aztec_Society_Smith_and_Hicks_2016_)

**Ethnography is not a snapshot of prehistory.** Dates, contact histories, regional relationships, and institutional change must remain attached to comparative observations; this is one reason resources such as D-PLACE preserve geographic, linguistic, and temporal context. [PLOS](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0158391)

### Bottom line for TCE

The most productive minimal implementation is:

**People solve problems, cultivate relationships, make commitments, and sometimes impose costs. Institutions determine which advantages become exclusive, enforceable, and inheritable.**

That architecture permits a successful organizer to remain an accountable village notable in one world, become a patron with dependent clients in another, and found a durable ruling house in a third—without prescribing any of those outcomes in advance.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92814-f9b4-83ea-9dee-303ef8ea5b19)
