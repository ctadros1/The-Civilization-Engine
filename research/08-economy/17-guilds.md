# Guilds as economic, political, and knowledge institutions

## Main conclusion for TCE

**Model a guild as a changeable bundle of rights, services, obligations, and political relationships—not as a building that grants craftsmanship while penalizing innovation.**

Historical guilds could organize training, enforce agreements, support members, regulate products, and bargain with governments. Those same organizations could exclude competitors, subordinate workers, and preserve valuable privileges. The central disagreement is not whether these activities existed, but their relative importance and whether alternative institutions could have supplied the useful services at lower social cost. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.28.4.169)

For TCE, separate five things that games often merge:

**Technical competence ≠ completion of apprenticeship ≠ guild membership ≠ permission to operate independently ≠ ownership of a workshop.**

A capable outsider may lack permission to sell. A wealthy license-holder may employ skilled workers without possessing their expertise. An apprentice may leave early yet retain useful knowledge. These distinctions are essential to producing believable exclusion, mobility, innovation, and institutional change.

---

## 1. Mechanisms: rules a simulation can implement

The rules below are **proposed implementations**, informed by the cited historical mechanisms. They are not historically estimated equations.

### 1.1 Formation: cooperation and privilege can motivate the same association

Do not require a “medieval era,” a particular population, or a guildhall before association becomes possible.

Instead, let independent producers or merchants consider organizing when they repeatedly encounter shared problems: unreliable apprentices, uncertain product quality, insecure trading rights, expensive collective facilities, disputes, or competition they would prefer to restrict.

A candidate coalition forms when members expect:

\[
\text{service benefits}+\text{expected privileges}
>
\text{dues}+\text{coordination costs}+\text{obligations}.
\]

These are private incentives, not a social-welfare test. A profitable association may harm nonmembers.

Maintain separate states for **association exists**, **government recognizes it**, and **government enforces its exclusive rights**. This allows voluntary societies, privileged corporations, illegal combinations, and state-organized occupational bodies to emerge from related components.

Merchant associations also need a collective-security mechanism. Greif, Milgrom, and Weingast’s repeated-game explanation emphasizes organized merchants’ ability to make rulers respect trading rights; its subject is not simply a craft cartel restricting local entry. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/261953)

### 1.2 Membership: represent a ladder of rights, not a single status

Make membership rules predicates over socially defined characteristics: occupation, residence, origin, citizenship, kinship, religion, gender, training record, sponsor, fee payment, and reputation.

Then assign rights separately:

| Right | Why it needs its own field |
| --- | --- |
| Receive training | Training access need not confer future mastership. |
| Work for a member | Employees may be permitted without full membership. |
| Sell independently | The principal barrier may concern independent production, not employment. |
| Open a workshop or hire apprentices | These can require additional qualifications or capital. |
| Use a collective mark | Certification may be separable from occupational licensing. |
| Vote or hold office | Members need not possess equal political rights. |
| Inherit, lease, or transfer privileges | Rights can become assets rather than personal qualifications. |

Allow exemptions and alternative admission routes: a master’s child, an incoming specialist, a widow continuing a household business, a purchased admission, or a government-sponsored exemption.

European evidence rejects universal hereditary closure, but openness among admitted masters does not establish equal opportunity for the wider population. Formal guild participation was heavily gendered, with important local exceptions. [OUP Academic](https://academic.oup.com/jsh/article/54/2/421/5644454)

**Implementation consequence:** record applications, refusals, discouraged potential applicants, admissions, and actual employment separately. Counting only successful entrants will make restrictive institutions look more inclusive than they are.

### 1.3 Apprenticeship: a household, labor, and training contract

An apprenticeship should exchange several real resources over time: instruction, practice opportunities, food, lodging, materials, productive work, sometimes an upfront payment, and sometimes later compensation.

The master decides how much time to teach rather than produce. The apprentice decides whether to remain, seek another master, migrate, or work independently. Guarantors and relatives can supply money, credibility, or enforcement.

Historical contracts did not guarantee uninterrupted service. Wallis’s work on England emphasizes frequent early departures and the distinction between learning, completing an indenture, and later acquiring urban freedom. [LSE Research Online](https://eprints.lse.ac.uk/22515/1/2207Wallis.pdf)

A suitable learning rule is:

\[
\Delta s\_{ik}
=
\alpha\_k h\_{ij}\tau\_j\max(s\_{jk}-s\_{ik},0)
+\text{practice learning},
\]

where \(s\_{ik}\) is learner \(i\)’s proficiency in technique \(k\), \(h\_{ij}\) is relevant contact time, and \(\tau\_j\) is teaching effort or effectiveness. Choose \(\alpha\_k\) through calibration; it is not a historical constant.

Important consequences:

* Time served is not equivalent to skill acquired.
* Excess apprentices can dilute instruction unless other skilled workers teach.
* An apprentice’s departure preserves acquired knowledge.
* A master’s death can terminate a contract without erasing the learner’s progress.

Treat low-paid apprentice labor as an economic incentive for masters, not as evidence that all apprenticeships were purely educational or purely exploitative.

### 1.4 Quality control: separate actual quality, certification, and conformity

A useful quality system needs three variables:

**Actual performance:** durability, purity, fit, safety, or another product-specific attribute.

**Buyer beliefs:** confidence in an individual producer or collective mark.

**Regulatory conformity:** compliance with required materials, dimensions, processes, or approved techniques.

These can diverge. A technically superior product may violate an old process rule; a conforming product may still be poorly made.

Give inspections a labor cost, a detection capability, and a corruption risk. Complaints and failed products should update reputations. A dishonest member can damage the guild’s collective mark; members therefore have a reason to monitor one another.

Do not assume that a rule described as protecting quality actually improves it. The guild literature disputes how often such regulation served consumers rather than incumbents. [American Economic Association](https://www.aeaweb.org/articles?id=10.1257%2Fjep.28.4.169)

**TCE rule:** measure quality improvements through product outcomes, not through the presence of certification.

### 1.5 Monopoly: exclusivity matters only where it can be enforced

Specify exactly what an exclusive right covers: an occupation, product, process, trading location, customer category, or geographic jurisdiction.

Then simulate enforcement:

\[
E[\text{consequence}]
=
P(\text{detection})
P(\text{sanction upheld}\mid\text{detection})
\times \text{sanction}.
\]

Detection should depend on inspectors, complaints, visibility, and territory. Whether punishment is upheld should depend on courts, officials, exemptions, and political relationships.

An excluded producer can respond by becoming an employee, moving beyond the jurisdiction, selling through an intermediary, changing products, petitioning, concealing production, or abandoning the occupation.

This produces a more credible monopoly than automatically removing nonmembers from the economy.

**Keep economic outcomes endogenous.** Restricted entry may increase incumbent margins, but imports, substitute products, smuggling, demand contraction, and internal cheating can limit those gains. Higher master income also need not mean higher journeyman wages.

### 1.6 Political power: distinguish corporate representation from worker power

A guild’s political resources can include dues, loans, organized votes, office eligibility, administrative expertise, provisioning capacity, collective petitions, and the ability to interrupt economic activity.

But “guild government” need not mean government by ordinary artisans. Soly’s comparative textile research emphasizes conflicts between merchants and producers, and between larger and smaller masters. In Ulm, merchant and mercer interests supported rural competition rather than granting urban fustian weavers the monopoly they wanted. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/political-economy-of-european-craft-guilds-power-relations-and-economic-strategies-of-merchants-and-master-artisans-in-the-medieval-and-early-modern-textile-industries/CB6B4FD5B10DF57521C5B44F2BBF2A55)

Represent at least three conflicts:

**Guild versus outsiders:** access, prices, and privileges.

**Guild versus government:** taxes, service obligations, enforcement, and autonomy.

**Within the guild:** exporters versus local sellers, employers versus employees, wealthy officeholders versus poorer members.

A useful capture feedback loop is:

\[
\text{incumbent surplus}
\rightarrow \text{political resources}
\rightarrow \text{privileges}
\rightarrow \text{incumbent surplus}.
\]

It should face countervailing forces: consumer complaints, rival associations, fiscal emergencies, competing jurisdictions, or rulers seeking to weaken an entrenched coalition.

Also model the reverse relationship: governments may use associations to organize taxpayers and suppliers. Political dependence can run in either direction.

### 1.7 Knowledge: distinguish invention, adoption, and diffusion

These are different processes.

**Invention** creates or modifies a technique.

**Adoption** occurs when a workshop decides that technique is worth using.

**Diffusion** occurs when others learn enough to reproduce it.

Guilds can affect each process differently. An organization might support experimentation among members while blocking outsiders from learning its results. Alternatively, it might admit skilled migrants while prohibiting a labor-saving process that threatens incumbent assets.

For each technique, store both **information access** and **practical proficiency**. Hearing that a new firing method exists is not equivalent to mastering it.

Secrecy should alter contact opportunities, teaching permissions, documentation access, and sanctions for disclosure. It should not make knowledge magically invisible to observation or erase it when someone resigns.

The relevant historical hypothesis is that guild-supported training and mobility could spread transferable craft knowledge; Epstein made this a central part of the positive case for guilds. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/craft-guilds-apprenticeship-and-technological-change-in-preindustrial-europe/4B18A7808BACBFA40D76475FBCB665E0)

### 1.8 Welfare, religion, and identity are functional parts of the institution

Associations should be able to fund assistance, funerals, rituals, communal meals, meeting spaces, and other member services. These consume real resources but can strengthen trust and willingness to cooperate.

Richardson’s analysis of late-medieval English guilds argues that occupational and religious activities reinforced one another rather than constituting unrelated functions. [Sage Journals](https://journals.sagepub.com/doi/10.1177/1043463105051631)

For TCE, repeated meetings create information-sharing opportunities. Assistance makes membership valuable during illness or unemployment. Exclusion from the association can therefore impose social and economic losses beyond a fine.

Keep the treasury explicit. An epidemic, war, or trade collapse can increase claims while reducing contributions. Institutions should sometimes cut benefits, borrow, raise dues, or fail.

### 1.9 Innovation: what the Ogilvie–Epstein debate actually implies

| Interpretation | Main mechanism | What TCE should test |
| --- | --- | --- |
| **Epstein:** guilds could support technical progress | Training arrangements, skilled mobility, and institutional support could facilitate knowledge transmission and innovation. | Does the association expand effective learning beyond what households or kin groups would otherwise provide? |
| **Ogilvie:** guild privileges commonly imposed substantial costs | Exclusion and political bargains could preserve rents while restricting access and competition. | Could the same useful services be supplied without compulsory membership or exclusive rights? |
| **Institution-specific synthesis** | The services and restrictions were bundled differently across places and periods. | Which provisions cause the observed outcome, and who receives the gains? |

These are competing interpretations, not estimates of a universal positive or negative coefficient. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/craft-guilds-apprenticeship-and-technological-change-in-preindustrial-europe/4B18A7808BACBFA40D76475FBCB665E0)

**The counterfactual is decisive.** A guild can improve knowledge transmission relative to kin-only learning while performing worse than open apprenticeship supported by reliable courts.

Likewise, an association can give its members political representation while excluding most inhabitants. Ogilvie distinguishes participation within an organization, representation of the organization, and inclusion across society. Those are different dashboard metrics. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-institutional-economics/article/thinking-carefully-about-inclusiveness-evidence-from-european-guilds/6F8FDAADA887CF4297F20F9FC634F1D4)

Do not settle the debate by programming one side’s conclusion. Program the rival mechanisms and compare outcomes under matched conditions.

---

## 2. Parameters: historical anchors and explicit design assumptions

### 2.1 Empirical anchors

**Confidence concerns the stated observation, not its applicability everywhere.** “High” generally means a well-defined documented rule or sample statistic; “medium” indicates greater selection or measurement problems. None of these values is a global guild average.

| Variable and setting | Value and units | Interpretation and confidence |
| --- | --- | --- |
| English statutory apprenticeship requirement, 1563 | **7 years minimum** | **High for the legal prescription.** Not a universal duration of learning or a guarantee of service. Wallis. [LSE Research Online](https://eprints.lse.ac.uk/22515/1/2207Wallis.pdf) |
| Genoese apprenticeship contracts, 1451–1530 | **6.1 years mean**, across **7,363 contracts** with recorded lengths | **High within the selected records.** Contracted duration, not actual time served. Brioschi, 2026. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70120) |
| Occupational variation in that Genoese sample | **5.2 years for bakers; 6.6 for joiners** | **High for these occupational means.** These are not minimum and maximum individual contracts. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70120) |
| Continued residence with original master, London’s 1695 linked records | About **38% in the seventh year** | **Medium-to-low as a transferable target:** small, selected observations; residence is not identical to contract completion or qualification. Wallis. [LSE Research Online](https://eprints.lse.ac.uk/22515/1/2207Wallis.pdf) |
| Chinese apprenticeship prescriptions discussed by Moll-Murata | Often **3 years**; often **1 apprentice per workshop** | **Medium.** Examples of institutional rules, not measured universal practice. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/chinese-guilds-from-the-seventeenth-to-the-twentieth-centuries-an-overview/61A96EB7FF67CE0BE8073C36DDB049CD) |
| New masters who were masters’ children, selected European observations | **26% France; 27% Low Countries; 44% German sample**, or **36% excluding Wildberg** | **Medium.** Selected, unweighted observations—not national population estimates. Prak and coauthors. [OUP Academic](https://academic.oup.com/jsh/article/54/2/421/5644454) |
| Antwerp coopers’ admission charges | Outsiders paid **1.3–2.7 times** sons’ fees | **Medium; case-specific.** Useful for a relative kinship preference, not an absolute entry-cost calibration. [OUP Academic](https://academic.oup.com/jsh/article/54/2/421/5644454) |
| Kinship in Genoese apprenticeship contracts | **6.0%** related to master; **54.9%** related to guarantor | **High for recorded relationships; incomplete for all social ties.** Categories can overlap. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70120) |
| Late-Qing Beijing huiguan | Approximately **400 halls**, only **10–20%** commercial or craft-related | **Medium estimate.** A hall count is not a count of occupational guilds. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/chinese-guilds-from-the-seventeenth-to-the-twentieth-centuries-an-overview/61A96EB7FF67CE0BE8073C36DDB049CD) |
| Modern US occupational licensing, survey analyzed by Kleiner and Krueger in 2013 | **29% of employees licensed**; licensing associated with about **18% higher wages** | **Medium for the study’s estimates.** Not current 2026 prevalence, not a medieval parameter, and not a universal causal wage effect. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/669060) |

Several tempting parameters are **not defensibly identified as universal constants**: guild price markups, productivity effects, innovation penalties, apprenticeship completion rates, admission rejection rates, and probabilities of capturing government. Treat them as outcomes or locally calibrated quantities.

### 2.2 Proposed sensitivity ranges—not historical estimates

Use these to explore model behavior before selecting particular historical calibration cases.

| Design parameter | Initial experimental range | Implementation meaning |
| --- | --- | --- |
| Candidate founding coalition | **3–10 independent workshops** | Computational search threshold, not a historical minimum guild size. |
| Formal apprenticeship duration | **2–8 years** | Benchmark range around selected cases; allow no fixed term and craft-specific exceptions. |
| Apprentice cap | **1–4 per master**, or no legal cap | Separate the legal cap from physical teaching capacity. |
| Entry charge | **0–180 local unskilled wage-days** | Tests affordable versus severe financial barriers. Do not confuse this with total workshop startup capital. |
| Annual membership dues | **0–10 unskilled wage-days per member** | Must actually finance administration and services. |
| Deliberate teaching effort | **5–25% of a teacher’s available work time** | Experimental time allocation; teaching should reduce other output unless combined with productive work. |
| Inspection intensity | **0–0.25 inspections per workshop-month** | An aggregate target constrained by inspector capacity and travel. |
| Knowledge-access policy | **Household-only / members / selected outsiders / open** | Prefer explicit permissions to a vague “secrecy bonus.” |

All entries in this second table have **unestimated historical confidence**: they are proposed test settings. Sweep them and examine outcomes rather than quietly treating them as facts.

Use local wage-days to normalize institutional charges, but keep actual payments in the world’s money or goods. A fee is a transfer; the labor spent collecting, contesting, or avoiding it is a resource cost.

---

## 3. Variation across eras and regions

### 3.1 Across economic settings—not a mandatory institutional ladder

| Setting | Recommended representation | Main caution |
| --- | --- | --- |
| **Foraging societies** | Learning networks, ritual associations, reputation, reciprocal assistance, and control over specialized knowledge can exist without urban occupational corporations. | The evidence reviewed here does not justify calibrated prehistoric “craft guilds.” Do not project later institutions backward. |
| **Early farming and emerging specialization** | Allow household training, kin-based expertise, seasonal specialists, and production attached to powerful households or religious authorities; associations become an additional possibility. | Specialized production alone does not establish independent corporate organization. |
| **Pre-industrial towns and trade networks** | Combine occupational regulation, merchant coordination, apprenticeships, collective assets, identity, and political privileges in locally specific ways. | There was no universal European—or worldwide—guild constitution. |
| **Industrializing economies** | Permit firms, schools, courts, associations, and governments to take over particular functions; some organizations disappear, others transform or expand. | Do not schedule automatic abolition when a technology threshold is crossed. |
| **Modern economies** | Reuse components for professional associations, occupational licensing, certification, trade associations, and worker organizations. | A licensing board, an employers’ association, and a trade union represent different constituencies and powers. |

Modern licensing demonstrates that restricted occupational entry is not exclusively a pre-industrial phenomenon, but it does not make modern professions identical to historical guilds. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/669060)

### 3.2 Europe: substantial variation within the supposedly standard case

A useful European distinction is between associations dominated by merchants, independent master artisans, or competing factions within a trade. Political constitutions and production structures affected which group benefited from regulation.

Soly’s work is especially useful because it links guild policy to merchants’ control of markets and to differences among producers, rather than treating every master as having the same economic interest. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/political-economy-of-european-craft-guilds-power-relations-and-economic-strategies-of-merchants-and-master-artisans-in-the-medieval-and-early-modern-textile-industries/CB6B4FD5B10DF57521C5B44F2BBF2A55)

**TCE implication:** initialize differences through ownership, market access, and political rights—not a cultural trait such as “European guild conservatism.”

### 3.3 China: huiguan were not simply European craft guilds

Moll-Murata distinguishes **huiguan**, commonly organized around native-place identity, from **gongsuo**, commonly associated with occupational organization, while emphasizing overlap. Such bodies could supply lodging, mediation, ritual services, and commercial coordination. In some settings, regulation sought to bring practitioners into an association rather than simply prevent new entrants. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/chinese-guilds-from-the-seventeenth-to-the-twentieth-centuries-an-overview/61A96EB7FF67CE0BE8073C36DDB049CD)

Chinese associations expanded markedly in the nineteenth century, complicating any worldwide story in which guilds necessarily decline as commerce develops. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/chinese-guilds-from-the-seventeenth-to-the-twentieth-centuries-an-overview/61A96EB7FF67CE0BE8073C36DDB049CD)

**TCE implication:** allow origin-based networks to become commercial organizations. Membership can span several trades, and a migrant hall need not possess licensing powers.

### 3.4 South Asia: śreṇī, community, and occupational regulation

The **śreṇī** literature describes organizations with collective resources, internal rules, leadership, and recognized dealings with rulers. Khanna’s legal-economic reconstruction discusses their commercial and other collective functions. However, analogies to modern corporations require caution: corporate identity, perpetual existence, limited liability, and tradable ownership are separate institutional properties, not a package that can safely be assumed. [Semantic Scholar](https://pdfs.semanticscholar.org/4068/5945c93145922a51321f9ad4054e317cb212.pdf)

For later South Asia, Roy emphasizes the variety of formal guilds, artisan councils, community networks, and master-centered arrangements. Kinship and community could organize credit, trust, training, and access to resources without an institution resembling a European chartered corporation. Caste and guild could overlap but were not identical. [Cambridge University Press](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/D7569A6FADA78282B51C48B18212C7D0/S0020859008003623a.pdf/guild_in_modern_south_asia.pdf)

**TCE implication:** represent hereditary social membership separately from occupational association. Social sanctions may enforce business rules without a specialized guild court. Do not assign occupations or abilities biologically; model historically contingent access, expectations, and coercion.

**Evidence caution:** ancient śreṇī are better used here to motivate institutional possibilities than to supply precise quantitative defaults.

### 3.5 Islamicate societies: asnaf/esnaf and negotiated authority

Ottoman **esnaf** provide a well-documented example, not a template for every Islamic society. Occupational groups interacted with officials and courts over production, provisioning, obligations, and privileges.

Especially useful for simulation is **gedik**: rights associated with practicing a trade and its productive establishment. Yıldırım describes their development into inheritable, transferable, and pledgeable interests; outsiders could acquire rights through transactions or debt enforcement. These were not simply personal certificates of craft competence. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/ottoman-guilds-in-the-early-modern-era/DBB9453C3BE34AEE2D4F8D3C9E390BAA)

**TCE implication:** a license can be an asset owned separately from skill, labor, and real estate. This permits rent extraction, leasing, foreclosure, and conflict between working artisans and rights-holders.

### 3.6 Japan: patronage, associations, and membership rights

Nagata distinguishes medieval **za** arrangements from early-modern **kabunakama** and related stock societies. Patronage and membership rights mattered, but organizational forms and entry restrictions varied.

Her account of the 1841 abolition describes commercial disruption rather than an uncomplicated release of competitive efficiency. This is an important counterexample—not proof that abolition always raises prices. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/brotherhoods-and-stock-societies-guilds-in-premodern-japan/80232105C29B649F36B16ABE1E3F425D)

**TCE implication:** abolishing exclusion can also remove coordination services. The result depends on whether substitutes exist and on which provisions are abolished.

### 3.7 African analogues: broader associations of expertise and authority

Jansen’s research on Mande hunters’ associations in Mali shows organizations combining specialized knowledge, ritual identity, collective action, and civic functions. These are useful analogues for associational mechanisms, not evidence of a universal craft-guild form. Nor should contemporary hunters’ associations be treated as unchanged survivals of prehistoric foraging society. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/from-guild-to-rotary-hunters-associations-and-malis-search-for-a-civil-society/1A43533E9703E74F0B6F38AB1037622D)

**TCE implication:** the same institutional components should support expert associations outside urban manufacturing, including organizations whose authority rests on initiation, reputation, and public service rather than a trade charter.

---

## 4. Stylized facts a credible simulation should reproduce

These are **conditional validation targets**, not patterns that every generated society must exhibit.

| Pattern | Evidence or numerical anchor | What to measure in TCE |
| --- | --- | --- |
| **Guild membership is not universally hereditary.** | Comparative European records show substantial admission beyond masters’ children. [OUP Academic](https://academic.oup.com/jsh/article/54/2/421/5644454) | Kinship shares among entrants; origin diversity; rejected applicants; wealth and gender exclusions. |
| **Training outside the family can coexist with strong family advantages.** | In Genoa, kin links to masters were uncommon, but kin guarantors were widespread. The same study finds relationships mattered for subsequent advancement. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70120) | Sponsorship networks, financing, admission, and later independent establishment—not just who taught whom. |
| **Formal training duration does not determine actual trajectories.** | English evidence distinguishes residence, service, freedom, and migration; Genoese contracts varied across occupations. [LSE Research Online](https://eprints.lse.ac.uk/22515/1/2207Wallis.pdf) | Contract length, time served, skill, migration, employment, and licensing as separate series. |
| **Political representation does not guarantee broad economic inclusion.** | Corporate representation and society-wide participation are distinct. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-institutional-economics/article/thinking-carefully-about-inclusiveness-evidence-from-european-guilds/6F8FDAADA887CF4297F20F9FC634F1D4) | Council seats alongside prices, excluded workers, entry barriers, and household welfare. |
| **Guild politics includes conflict among producers.** | Merchant and artisan interests could diverge sharply in textile centers. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/political-economy-of-european-craft-guilds-power-relations-and-economic-strategies-of-merchants-and-master-artisans-in-the-medieval-and-early-modern-textile-industries/CB6B4FD5B10DF57521C5B44F2BBF2A55) | Voting coalitions by business size, customers, assets, and employment position. |
| **Occupational rights can become rent-bearing assets.** | Ottoman gedik transactions separated rights-holding from personal craftsmanship. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/ottoman-guilds-in-the-early-modern-era/DBB9453C3BE34AEE2D4F8D3C9E390BAA) | License prices, leases, concentration, foreclosure, and divergence between owners and skilled workers. |
| **Institutional removal can have transitional costs.** | Japan’s 1841 episode illustrates disruption when established coordination arrangements are dismantled. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/brotherhoods-and-stock-societies-guilds-in-premodern-japan/80232105C29B649F36B16ABE1E3F425D) | Short-run transaction failures and learning disruption separately from long-run entry and price changes. |

A further **mechanistic test**, rather than a settled universal historical fact, concerns innovation: a guild that broadens teaching contacts should sometimes accelerate diffusion, while an otherwise similar guild that blocks new producers or methods should sometimes retard it.

The simulation should be able to produce both outcomes without changing a hidden “good guild/bad guild” switch.

---

## 5. Modeling recommendation for TCE

### 5.1 Use composable institutions and distinct economic entities

| Entity/component | Essential state |
| --- | --- |
| **Person** | Skills, technique proficiencies, wealth, household, employment, contracts, memberships, social ties, eligibility attributes, political participation. |
| **Workshop** | Owner, assets, location, workforce, methods, inventory, costs, customers, reputation, required rights. |
| **Association** | Members, officers, treasury, collective property, services, internal rules, voting arrangements, external relationships. |
| **Charter or legal privilege** | Issuer, beneficiaries, occupational/product scope, geography, exclusivity, duties, exemptions, duration, enforcement authority. |
| **Apprenticeship contract** | Parties, guarantor, term, payments, maintenance obligations, teaching arrangements, exit and transfer conditions. |
| **Occupational right** | Holder, permitted activity, transferability, inheritance, lease, pledge, revocation conditions. |
| **Technique** | Required knowledge, materials, tools, practice, observable outputs, documentation, secrecy permissions. |
| **Government relationship** | Taxes, loans, services owed, representation, disputes, enforcement commitments, patronage. |

A person should be able to belong to several associations—for example, a craft body and a native-place society—without duplicating that person economically.

Do not make the guild itself own every member’s workshop unless its actual property rules say so.

### 5.2 Express rules as permissions, obligations, and sanctions

A compact rule representation can be:

`scope + subject + condition + permitted/required/prohibited action + sanction + authority`

This supports rules such as:

“Members may hire one apprentice.”

“Only approved workshops may use the city cloth mark.”

“A deceased member’s household may continue trading temporarily.”

“Members must contribute to an agreed provisioning obligation.”

“Exporting this technique requires permission.”

Crucially, **the existence of a rule does not establish compliance**. Store detected breaches and adjudicated outcomes separately.

### 5.3 Make collective decisions emerge from members

Avoid an omniscient institution that maximizes a single “guild welfare” score.

Let officers propose policies and eligible members support them according to expected effects on income, security, status, obligations, and relationships. Governance rules determine whose preferences count.

For a new technique, distinguish a master who owns expensive existing equipment, an exporter facing overseas competition, a worker whose skill loses value, and an apprentice who gains a new opportunity. They may reach different conclusions without any irrationality or universal cultural hostility to innovation.

Officers may also pursue their own interests. Leadership should be able to become entrenched through wealth, seniority, restricted eligibility, or control over institutional information.

### 5.4 Keep knowledge embodied and geographically connected

Use sparse technique-proficiency records per person. Transmission requires a plausible channel: teaching, joint work, observation, documents, inspection, migration, or collaboration.

A guild meeting can introduce people or demonstrate a procedure, but it should not instantly grant mastery to every member. Written instructions can improve access while leaving substantial practical learning necessary.

For architecture, attach construction methods and stylistic practices to trained builders, workshops, and commissioning institutions. A migrating specialist may introduce a roof system; a guild may standardize its execution; clients may reject it because materials or preferences differ.

This yields architectural continuity and change without a citywide “style unlocked” flag.

### 5.5 Schedule the expensive decisions, not every relationship every day

For 10k–50k people, a suitable unbenchmarked implementation strategy is:

| Cadence | Suggested work |
| --- | --- |
| **Daily or existing production tick** | Work, teaching time, consumption, production, actual encounters. |
| **Monthly or event-triggered** | Apprentice matching, admission applications, dues, assistance claims, inspection allocation. |
| **Annual or constitutional cycle** | Elections, budgets, charter negotiations, policy revisions. |
| **Immediate events** | Death, migration, default, expulsion, major dispute, workshop closure, government intervention. |

Index candidates by occupation, location, and relevant relationships. Avoid searching every person against every possible master. Cache jurisdictional permissions until the relevant law, charter, location, or personal status changes.

Use stable iteration and seeded choices for reproducibility. These are architectural recommendations, not measured performance guarantees.

Visible events should arise from the simulation: inspectors visiting workshops, apprentices practicing, members assembling, traders petitioning, or a household contesting exclusion.

### 5.6 Validate services and restrictions independently

Run matched-seed institutional experiments:

| Configuration | Purpose |
| --- | --- |
| Household/kin training only | Baseline with restricted personal learning networks. |
| Voluntary training and mutual-aid association | Isolate cooperative services. |
| Same association plus exclusive entry rights | Identify consequences of closure. |
| Open apprenticeship with reliable external enforcement | Test whether a guild is necessary for contractual support. |
| Separate public certification | Separate quality assurance from occupational monopoly. |

Track consumer prices, actual product failures, output per work-hour, training access, worker wages, master profits, migration, knowledge diffusion, and political concentration.

Do not judge success solely by incumbent wealth or aggregate output. A regime may improve product reliability while reducing access, or increase total production while concentrating the gains.

### 5.7 Existing models and games worth borrowing from

| Model or game | Useful element | Limitation |
| --- | --- | --- |
| **De la Croix, Doepke, and Mokyr, “Clans, Guilds, and Markets” (2018)** | Formalizes how apprenticeship institutions and person-to-person knowledge transmission affect growth. Useful for comparing restricted and broader learning networks. | An analytical growth model, not a validated individual-agent guild simulation. Its stylized institutional comparisons should not become fixed cultural traits. [Northwestern Faculty](https://faculty.wcas.northwestern.edu/mdo738/research/delaCroix_Doepke_Mokyr_QJE_2018.pdf) |
| **Greif, Milgrom, and Weingast (1994)** | Collective action and credible responses to rulers help explain merchant security. | A specific merchant-guild mechanism, not a complete model of craft regulation. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/abs/10.1086/261953) |
| **Dwarf Fortress** | Developer material connects occupational associations with petitions for physical meeting spaces. This makes institutions visible in daily life. | Borrow the link between organization, demands, and place—not a population threshold as a historical law. [Bay 12 Games](https://www.bay12games.com/dwarves/dev_2019.html) |
| **The Guild 3** | Connects household businesses, dynasties, offices, and political maneuvering. | Its designed gameplay and European scenario are not historical calibration or a demonstration of unscripted institutional emergence. [Steam Store](https://store.steampowered.com/app/311260/The_Guild_3/) |

---

## 6. Sources, datasets, and evidence limits

### Research base

The core interpretive contrast is **S. R. Epstein’s 1998 “Craft Guilds, Apprenticeship, and Technological Change in Preindustrial Europe”** against **Sheilagh Ogilvie’s 2014 “The Economics of Guilds”** and her broader 2019 book, *The European Guilds: An Economic Analysis*. Read these as competing explanatory programs, not interchangeable summaries. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-economic-history/article/craft-guilds-apprenticeship-and-technological-change-in-preindustrial-europe/4B18A7808BACBFA40D76475FBCB665E0)

For worldwide comparison, the **2008 International Review of Social History supplement on guilds** is particularly useful: the contributions by Moll-Murata, Roy, Yıldırım, Nagata, Soly, and Jansen support comparisons without assuming identical institutions behind different names. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/chinese-guilds-from-the-seventeenth-to-the-twentieth-centuries-an-overview/61A96EB7FF67CE0BE8073C36DDB049CD)

### Data sources for calibration

| Source | Available material | Best use and principal warning |
| --- | --- | --- |
| **Ogilvie’s European guild databases** | **12,051 qualitative observations**, **5,333 quantitative observations**, and **1,032 bibliography entries**. | Extract particular rules, practices, and settings. These are observations, not counts of distinct guilds; coverage is not representative. The posted license includes a noncommercial restriction, so check terms before incorporating data into a commercial distribution. [Sheilagh Ogilvie](https://sheilaghogilvie.com/guilds-book/guilds-databases/) |
| **Records of London’s Livery Companies Online—ROLLCO** | Searchable apprenticeship and freedom records, broadly covering **1400–1900**. | Link people, masters, origins, and admissions. Freedom records do not directly measure training completion. [London Roll](https://www.londonroll.org/) |
| **Prak and coauthors, “Access to the Trade” (2020)** | Comparative evidence covering over **100,000 masters in 27 towns** and **450,000 apprentices in 21 towns**. | Calibrate heterogeneous recruitment, not a single European closure rate. [OUP Academic](https://academic.oup.com/jsh/article/54/2/421/5644454) |
| **Brioschi’s Genoa study (2026)** | Contract-based analysis and supporting tables on duration, networks, and later trajectories. | Useful for relationships and timing; observed later mastership is an incomplete proxy, not a census of all successful careers. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ehr.70120) |
| **Chinese guild source collections discussed by Moll-Murata, including Peng Zeyi’s compilation** | Institutional documents and chronologies rather than a harmonized individual panel. | Compare rules and organizational types; do not treat documentary survival as institutional prevalence. [Cambridge University Press](https://www.cambridge.org/core/journals/international-review-of-social-history/article/chinese-guilds-from-the-seventeenth-to-the-twentieth-centuries-an-overview/61A96EB7FF67CE0BE8073C36DDB049CD) |

### The most consequential uncertainties

**Rules versus practice:** statutes are evidence of what someone sought to require, not proof of universal obedience.

**Members versus everyone else:** records of admitted members leave out rejected, discouraged, informal, and otherwise excluded producers.

**Association versus causation:** prosperous towns may sustain strong guilds; strong guilds may affect prosperity. Their coexistence alone cannot identify the direction.

**Surviving records versus representative populations:** administrative continuity, literacy, wealth, and later preservation shape the sample.

**Cross-cultural translation:** “guild” can conceal distinctions among occupational regulation, merchant coordination, religious association, native-place identity, and hereditary community. The regional studies above repeatedly require those distinctions rather than eliminating them. [Cambridge University Press](https://www.cambridge.org/core/services/aop-cambridge-core/content/view/D7569A6FADA78282B51C48B18212C7D0/S0020859008003623a.pdf/guild_in_modern_south_asia.pdf)

**Bottom line:** TCE should let a guild become a training network, an insurer, a cartel, a political coalition, a license-owning oligarchy, or several of these simultaneously. Its economic effects should follow from who can learn, work, compete, enforce rules, and exercise power—not from the institution’s name.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928a1-7cd8-83e9-9321-47dee9904589)
