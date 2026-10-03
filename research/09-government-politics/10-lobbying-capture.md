# Lobbying, rent-seeking and institutional capture

## A simulation-ready report for The Civilization Engine

**The most useful model for TCE is a feedback loop, not a generic corruption score:**

**Economic resources → collective organization and political access → control of decisions and appointments → protected economic advantages → resources that reproduce political control.**

Capture becomes durable when an interest no longer needs to purchase each favorable decision separately: it controls eligibility for office, the appointment of regulators, access to markets, or the institutions that investigate misconduct. Formal constitutional reform need not break this loop if the same actors retain financial, organizational, or coercive power. [OUP Academic](https://academic.oup.com/oxrep/article-abstract/22/2/203/334718)

Three distinctions are essential:

| Concept | Meaning for TCE |
| --- | --- |
| **Lobbying** | Attempts to influence decisions, including legitimate petitions, expertise, collective representation, and advocacy. It is not necessarily corrupt or socially harmful. |
| **Rent-seeking** | Resources devoted to obtaining or defending privileged returns rather than creating additional value. A monopoly concession is a potential rent; the effort spent securing it is rent-seeking. |
| **Institutional capture** | Persistent influence that redirects an institution toward particular interests, potentially through entirely legal rules. Bribery is one possible mechanism, not a requirement. |

These distinctions follow the rent-seeking literature, research on informational lobbying, and broader treatments of regulatory capture. They also prevent a crucial modeling error: treating every regulation, political donation, or profitable enterprise as evidence of capture. [Cambridge University Press](https://www.cambridge.org/core/books/abs/perspectives-on-public-choice/rent-seeking/1A89F9D608D0AB0605BC57130C0085DB)

---

## 1. Mechanisms: implementable causal rules

### 1.1 Concentrated benefits make organization worthwhile

Olson’s central insight is about incentives to organize. People sharing an interest do not automatically act together. Organization requires time, contributions, monitoring, and sometimes selective benefits or compulsory membership. A small group with large individual stakes may overcome these costs more easily than a dispersed population with small individual stakes. This is a tendency, not a rule that small groups always win. [De Gruyter Brill](https://www.degruyterbrill.com/document/doi/10.4159/9780674041660/html)

**Implementation rule:** calculate prospective gains and organizational costs at both individual and group levels.

A hypothetical privilege yielding 1,000 grain-units annually might give ten mill owners 100 units each while costing 1,000 households one unit each. The mill owners have stronger individual incentives to attend meetings, hire an intermediary, and monitor agreements. Households may nevertheless organize when an existing religious association, neighborhood council, or political entrepreneur lowers their coordination costs.

Represent collective action through:

* Individual contributions, expected benefits, and free-riding.
* Existing organizations that can collect dues or sanction noncontributors.
* Internal disagreements: exporters, import-competing merchants, large masters, and small masters should not automatically share a position.

**Do not implement “wealthy class supports policy” without an organization or other means of coordinating action.**

### 1.2 Stigler’s mechanism: political authority supplies economically valuable restrictions

Stigler’s theory treats regulation as something organized industries may seek because the state can restrict entry, support prices, allocate subsidies, or disadvantage substitutes. Later work adds competing constituencies, information asymmetries, and conflicts between politicians and administrators. It does not establish that all regulation serves producers or that capture is inevitable. [JSTOR](https://www.jstor.org/stable/3003160)

**Implementation rule:** firms compare investments in production with investments in changing the rules governing production.

Potential targets should include more than tax rates:

| Target | Advantage sought | Necessary institutional capability |
| --- | --- | --- |
| Market entry | Exclusive trading rights, restricted guild admission, licensing barriers | Authority to recognize and enforce eligibility |
| Public purchasing | Favored contracts, inflated prices, tailored specifications | Procurement discretion |
| Finance | Preferential loans, guarantees, debt forgiveness | Control over lenders or treasury commitments |
| Taxation | Exemptions, favorable assessments, selective collection | Assessment and collection discretion |
| Property and labor | Favorable land grants, debt enforcement, labor restrictions | Courts, registries, coercive enforcement |
| Political participation | Restricted franchise, hereditary eligibility, controlled nominations | Power to amend membership and selection rules |

These are proposed TCE policy primitives. Their effects should arise through the corresponding economic and institutional systems, rather than through a universal “capture increases profits” modifier.

### 1.3 Money often buys access and effort before it buys agreement

Lobbyists can provide research, draft language, intelligence about other actors, and administrative labor. Hall and Deardorff describe lobbying as a subsidy to legislators who already share some of the lobby’s objectives. This differs from paying an opponent to reverse a preference. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/lobbying-as-legislative-subsidy/AE4B5D8AB9C2487BB78C2A51BB53E03F)

Experimental evidence also separates access from policy outcomes: disclosing that meeting participants were campaign donors increased access to senior congressional officials, but the experiment did not establish an equivalent effect on legislative votes. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ajps.12180)

**Implementation rule:** model at least three separate stages:

**Access → attention and information → decision.**

An interest can obtain a meeting but lose the vote. It can win the vote but fail to secure implementation. It can supply accurate information that improves a policy while simultaneously steering attention away from alternatives.

Give officials limited time. Better-connected organizations can occupy that time, but their proposals still encounter ideology, perceived public benefit, legal constraints, rival constituencies, and institutional procedure.

### 1.4 Different influence channels create different forms of dependence

| Channel | Implementable mechanism | Important limitation |
| --- | --- | --- |
| **Bribery** | An official weighs a private benefit against moral costs, detection, sanctions, and the consequences of betrayal. | A bribe does not automatically purchase a reliable or enforceable promise. |
| **Political funding** | A patron supplies campaign resources, coalition maintenance, retainers, transport, food, or communications. Dependence grows when substitutes are scarce. | Campaign finance is only one historically specific version of political funding. |
| **Office purchase** | A ruler exchanges appointment rights for immediate revenue. Buyers value future income, status, influence, and transferability. | The sale may be legal; subsequent extraction may or may not be. |
| **Marriage and kinship** | Marriage connects households, assets, information, succession claims, and reciprocal obligations. | Kin are not perfectly loyal; alliances can divide families or connect rival factions. |
| **Patronage and appointments** | An incumbent appoints dependents who owe their position or livelihood to the patron. | Dependence can weaken after tenure, wealth, or an alternative patron becomes available. |
| **Professional connections** | Former officials sell knowledge and access acquired in office; prospective employers may influence career incentives. | Expertise and access are distinct assets and can depreciate differently. |

Office-market research shows that prices reflected expected returns and that socially connected buyers could receive favorable terms. Florentine network research demonstrates why business and marriage connections should be modeled together rather than collapsed into wealth alone. Revolving-door evidence shows that an intermediary’s value can fall when a particular political connection disappears. [Cambridge University Press](https://www.cambridge.org/core/books/abs/venal-origins-of-development-in-spanish-america/eighteenthcentury-market-of-offices/9D9123754F8E9FD07AE91CA5BB26903B)

### 1.5 Capture becomes durable through control of reproduction

A favorable contract creates a temporary advantage. A rule that reserves future contracts for a hereditary category creates a self-reproducing advantage.

**Implementation rule:** allow successful coalitions to seek changes in:

**Eligibility:** ancestry, property, citizenship, profession, religion, sponsorship, or fees.

**Selection:** nomination, co-option, appointment, election, lot, office purchase, or inheritance.

**Oversight:** who appoints investigators, controls their budgets, receives complaints, and hears appeals.

**Economic reproduction:** inheritance, access to credit, apprenticeship, land ownership, and exclusive concessions.

The key vulnerability is often the institution supervising other institutions. In colonial Peru, Guardado finds that selling higher-level judicial oversight positions increased the value of subordinate offices and was associated with more spontaneous rebellion—consistent with weakened oversight increasing opportunities for extraction. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/717757)

A useful TCE implication is that buying a judge or appointment committee can be more valuable than bribing many frontline collectors.

### 1.6 Capture changes economic selection, not just the distribution of income

Protected firms may survive despite weak productivity, while outsiders cannot enter or expand. Political investment can become a substitute for innovation.

Research using Italian firms, politicians, and patents finds that market leaders were more politically connected but less innovative. Connections supported survival and growth in employment and revenue without corresponding productivity growth; aggregate losses in the authors’ model arose through weaker reallocation and growth. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.3982/ECTA18338)

**Implementation rule:** let privileges change actual constraints:

* Entry costs and eligibility.
* Credit availability and repayment discipline.
* Procurement demand.
* Exposure to competition.
* Expected returns to adopting or suppressing a technique.

Do not simply subtract a fixed percentage from discovery or productivity whenever capture rises.

**Accounting matters:** a bribe is a transfer from payer to recipient, not automatically an equal destruction of social output. Social losses arise from distorted production, exclusion, resources consumed in influence contests, concealment, enforcement, and defensive action. Nor should TCE assume that the entire value of a rent is always dissipated in competition for it. [Cambridge University Press](https://www.cambridge.org/core/books/abs/perspectives-on-public-choice/rent-seeking/1A89F9D608D0AB0605BC57130C0085DB)

### 1.7 Reversal requires changing incentives and the ability to enforce rules

Capture can weaken through outsider organization, alternative financing, changes in economic opportunities, independent investigation, or the destruction of legal privileges. But removing officeholders is different from removing the system that reproduces their advantages.

Acemoglu and Robinson’s model explains why an elite losing formal authority may compensate by investing more in informal influence or coercion. [National Bureau of Economic Research](https://www.nber.org/papers/w12108)

For TCE, reforms should modify specific mechanisms:

| Reform or disruption | Mechanism changed | Possible failure |
| --- | --- | --- |
| Independent audit and appeal | Raises the probability misconduct produces consequences | Investigators or courts remain dependent on the same coalition |
| Alternative taxation or lenders | Reduces government dependence on particular financiers | New financiers negotiate equivalent privileges |
| Open entry and procurement | Reduces the value of incumbent restrictions | Incumbents shift to credit, land, or enforcement barriers |
| Broader representation and association rights | Lowers outsiders’ organizational and political barriers | Elite control of nominations, information, or coercion persists |
| Succession, defeat, or revolution | Breaks particular connections and coalition agreements | A replacement coalition captures the same institutions |

Auditing can matter materially: Olken’s Indonesian experiment found a substantial reduction in missing project expenditure when audit probability increased. Conversely, historical evidence from French-imposed reforms in Germany suggests that removing multiple legal protections for old elites could improve later growth opportunities. Neither result establishes that transparency alone or any revolution automatically reverses capture. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/517935)

---

## 2. Parameters and quantitative calibration

### 2.1 Empirical anchors

**These are case-specific validation targets, not universal behavioral coefficients.** A treatment effect on access is not a coefficient for changing votes; a connected-firm profit share is not an estimate of aggregate welfare loss.

“High” below means relatively strong support for the stated result within the studied setting. “Medium” indicates historical reconstruction, descriptive measurement, or greater identification uncertainty. These are qualitative evidence assessments, not statistical confidence intervals.

| Quantity | Observed value and units | Setting and source | Confidence and proper use |
| --- | --- | --- | --- |
| **Time required for political closure** | Principal closure process **1297–1323: about 26 years** | Venice; Puga and Trefler, 2014. [DOI](https://doi.org/10.1093%2Fqje%2Fqju006) | **Medium–high historical.** Supports cumulative institutional change rather than an instantaneous oligarchy switch. |
| **Outsider participation in commercial contracts** | Commoner participation: **30/59 ≈ 51%** in 1241–1261; **22/81 ≈ 27%** in 1310–1323; **1/34 ≈ 3%** in 1324–1342, last figure calculated by combining table rows | Surviving Venetian *colleganza* contracts; Table I. [Diego Puga](https://diegopuga.org/papers/venice.pdf) | **Medium.** Small, selectively surviving archival samples—not population-wide trade shares. |
| **A financial barrier to political association** | Admission fee for men whose fathers were not members rose from **50 to 100 florins in 1350**; the latter was estimated at roughly **25 years of average shop rent** | Florence’s **Parte Guelfa**, not a generic craft-guild fee; Becker. [Cambridge University Press](https://www.cambridge.org/core/journals/comparative-studies-in-society-and-history/article/abs/some-aspects-of-oligarchical-dictatorial-and-popular-signorie-in-florence-12821382/F4E20568BD182AE91A50265DDED67B9F) | **Medium historical.** Useful example of a nominally purchasable route becoming practically exclusionary. |
| **Donor-status effect on access** | Senior policymakers became available **3–4 times as often** when donor status was revealed; experiment covered **191 congressional offices** | Kalla and Broockman, 2016. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ajps.12180) | **High for this access intervention.** Donor-status disclosure was randomized, not contribution amounts or votes. |
| **Loss of a political connection** | Former Senate staffers’ lobbying revenue fell **24%** when their former senator left office | Blanes i Vidal, Draca, and Fons-Rosen, 2012. [Benny](https://benny.aeaweb.org/articles?id=10.1257%2Faer.102.7.3731) | **Medium–high.** Calibrate office-specific relationship value, not a universal 24% influence penalty. |
| **Connected lending advantage** | Connected firms borrowed **45% more** and had **50% higher default rates**, in relative terms; preferential treatment was concentrated in government banks | Pakistan, 1996–2002; Khwaja and Mian, 2005. [OUP Academic](https://academic.oup.com/qje/article-abstract/120/4/1371/1926665) | **Medium–high within this lending setting.** “50% higher” is not 50 percentage points. |
| **Connected firms’ economic concentration** | Ben Ali-connected firms accounted for **less than 1% of wage employment** but **more than one-fifth of net corporate profits** in 2010 | Tunisia; Rijkers, Freund, and Nucifora’s research. [PIIE](https://www.piie.com/commentary/op-eds/tunisias-golden-age-crony-capitalism) | **Strong descriptive contrast; weaker causal attribution.** These profits are not all demonstrated rents. |
| **Audit intervention** | Announced audit probability rose from approximately **4% to 100%**; missing expenditure fell by about **8 percentage points** | More than 600 Indonesian village road projects; Olken, 2007. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/517935) | **High local causal evidence.** Missing expenditure is an accounting discrepancy measure, not a direct census of theft. |

There is no comparably defensible universal estimate for “percentage of surplus spent capturing government,” “annual probability of oligarchic closure,” or “loyalty created by marrying an official’s relative.” Those require explicit modeling assumptions and sensitivity testing.

### 2.2 Suggested initial TCE parameters

The following are **engineering priors proposed here**, not historical measurements. Their source is the model design below; confidence in their empirical calibration is **low**. The ranges are intended for sensitivity experiments.

| Parameter | Initial value | Sensitivity range | Units and interpretation |
| --- | --- | --- | --- |
| Political reconsideration interval | 1 | 1–6 | Months between ordinary organizational strategy updates; major events interrupt |
| Fixed coalition-organization cost | 30 | 5–300 | Person-days, reduced by existing organizations and trusted ties |
| Ordinary political spending budget | 2% | 0–15% | Share of positive annual discretionary cash flow; exceptional campaigns can draw reserves |
| Planning discount rate | 10% | 3–30% | Annual real rate; expectations about privilege survival are modeled separately |
| Discretionary relationship half-life | 5 | 1–20 | Years without reinforcing contact; kinship itself does not disappear |
| Official’s organized-interest meeting capacity | 4 | 1–20 | Meetings per month, alongside other duties and petitions |
| Audit probability | 5% | 0–50%; 100% intervention test | Per eligible discretionary transaction, subject to actual auditing capacity |
| Detection conditional on audit | 50% | 10–90% | Probability, varying with evidence, complexity, and investigator competence |

Do not independently randomize every number and expect historical realism. Fit combinations to mechanisms: for example, organization costs, meeting capacity, and relationship persistence jointly determine access concentration.

Use local wage-days, grain purchasing power, and enterprise surplus rather than transplanting modern dollar amounts.

For time conversion, an annual event probability \(p\_y\) becomes

\[
p\_{\Delta t}=1-(1-p\_y)^{\Delta t},
\]

where \(\Delta t\) is measured in years. This prevents a monthly implementation from accidentally multiplying annual opportunities for misconduct or reform.

---

## 3. Variation across eras and regions

### 3.1 Institutional conditions matter more than era labels

| Environment | Relevant pathways | Implication for TCE |
| --- | --- | --- |
| **Foragers** | Dependence can arise around resource access, obligations, or leadership, but mobility, sharing practices, and active resistance to domination can constrain accumulation. | Do not insert corporations, campaign finance, or bureaucratic capture into societies lacking those institutions. Model resource dependence and countervailing social sanctions. |
| **Early farming and mixed subsistence** | More defensible and transmissible wealth can support durable inequality and dependence, but farming alone does not determine hierarchy. | Make storage, land access, inheritance, household authority, and exit options causal variables—not “agriculture ⇒ oligarchy.” |
| **Pre-industrial towns and territorial states** | Guild privileges, merchant credit, tax farming, office markets, hereditary status, and overlapping household–government interests. | Offices and concessions can be assets; political organizations may also provide production, welfare, and defense. |
| **Industrial settings** | Larger enterprises and organized workers confront regulation, infrastructure allocation, finance, and changing political participation. | Model opposing organized interests and the displacement of old privileges by new economic opportunities. |
| **Modern bureaucratic settings** | Specialist lobbying, party funding, public procurement, administrative discretion, and revolving-door relationships. | Separate elected authorities, professional agencies, courts, public lenders, and information suppliers. |

The early-society distinctions draw on Woodburn and comparative work on the defensibility and transmission of wealth. The later distinctions are supported by historical office-market research, studies of institutional reform, and the regulatory-capture literature. These are configurations, not a compulsory sequence. [Stanford Poverty Center](https://inequality.stanford.edu/publications/media/details/egalitarian-societies)

### 3.2 Historical and regional cases

#### Venice: successful merchants restrict the routes that produced their success

Puga and Trefler describe an initial expansion of commercial opportunity followed by hereditary political closure and restrictions favoring nobles in lucrative maritime activity. The relevant mechanism is **commercial success → threatened incumbency → political closure → economic exclusion**. Treat the Serrata as cumulative coalition-building and rule changes, not a single decree causing immediate economic collapse. [DOI](https://doi.org/10.1093%2Fqje%2Fqju006)

#### Dutch regents: allocation of offices within a governing network

The Dutch case illustrates closure through inter-family agreements rather than simply purchasing individual votes. Early-eighteenth-century “Contracts of Correspondence” distributed offices among regent families, reducing conflict within the governing elite while reinforcing perceptions that office served private interests. [OUP Academic](https://academic.oup.com/british-academy-scholarship-online/book/51415/chapter/417816349)

**TCE implication:** an office-sharing agreement can stabilize government and exclude outsiders simultaneously. Low turnover or low factional violence is therefore not sufficient evidence of broad representation. Model a coalition’s internal rules for distributing positions, including compensation for members temporarily left out.

#### Florence and guild-run cities: corporate representation is not universal representation

Florentine guild politics differentiated wealthy merchant associations, lesser crafts, and workers lacking guild representation. The Ciompi upheaval of 1378 produced changes in participation, but an oligarchical restoration followed in 1382. [Cambridge University Press](https://www.cambridge.org/core/books/abs/criminal-justice-and-crime-in-late-renaissance-florence-15371609/bureaucratic-structure-of-the-otto-the-personnel-and-their-functions/62FF5B00E641AF4C552AC0F71C96EC49)

**TCE implication:** “guild government” must specify which guilds, who governs each guild, and who is excluded. A council representing masters may oppose patricians while still denying political standing to wage workers.

Florentine elite-network research also suggests that power can come from connecting otherwise separated social networks—not merely from having the most money or marriages. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/230190)

#### Qing China: hereditary commercial privilege intertwined with imperial finance

Jia Feng’s archival study of Ji’an’s salt monopoly describes hereditary permit rights, merchant operators, elite households, and imperial claims on revenues. Privilege holders and operating merchants were not always the same actors. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-the-royal-asiatic-society/article/from-local-jurisdiction-to-imperial-domain-the-making-of-jians-salt-monopoly-under-the-imperial-household-department-16441795/4AF25F6063DD716E6F05C82CC22D7E57)

**TCE implication:** separate **ownership of a concession**, **operation of the enterprise**, and **obligations to the treasury or patron**. An apparently wealthy merchant may depend on an elite landlord of commercial rights. This local case should not become a uniform rule for all Chinese commerce.

#### Ottoman Empire: fiscal delegation can sustain, not merely undermine, central rule

Salzmann’s analysis of eighteenth-century Ottoman fiscal arrangements challenges a simple account in which tax farming inevitably produces state decline. Fiscal rights connected central officials, financiers, and provincial actors; the state relied on relationships crossing the supposed boundary between public and private. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0032329293021004003)

**TCE implication:** selling or delegating revenue collection can solve an immediate financing problem while creating durable bargaining power. Whether it strengthens administration or entrenches extraction depends on contracts, oversight, competition, and the ruler’s alternatives.

#### Colonial Peru: capture of oversight raises the value of lower offices

Selling supervisory positions could increase the expected returns available to subordinate officials. This is capture spreading **down an institutional hierarchy**, not simply corruption spreading between peers. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/717757)

**TCE implication:** monitor dependence should affect multiple subordinate transactions. A compromised appeal court can change the incentives of an entire administrative district.

#### Sierra Leone: captured civil society can remain visibly active

Acemoglu, Reed, and Robinson find worse development outcomes where chiefdoms had fewer historically recognized ruling families and therefore less competition for chieftaincy. Yet these places could also exhibit greater expressed respect for chiefs and stronger measured social capital. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/674988)

**TCE implication:** association membership, public attendance, and expressed loyalty cannot be treated as automatic measures of political independence. Organizations may coordinate resistance—or reinforce dependence.

#### Pakistan and Tunisia: modern capture can be institution- or sector-specific

Pakistan’s government-bank/private-bank contrast shows why capture should attach to particular institutions rather than the whole economy. Tunisia’s connected-firm evidence similarly points toward advantages concentrated in regulated sectors, while acknowledging that connected firms also selected into profitable activities. [OUP Academic](https://academic.oup.com/qje/article-abstract/120/4/1371/1926665)

**TCE implication:** a polity can contain competitive industries, captured lenders, and highly restricted concessions at the same time.

---

## 4. Stylized facts and validation tests

A correct simulation should reproduce a **family of outcomes under corresponding conditions**, not force every world to resemble Venice or a modern patronage state.

| Pattern to reproduce | Suggested test |
| --- | --- |
| **Organization matters independently of aggregate wealth.** Concentrated interests can organize more effectively, but collective action is not automatic. [JSTOR](https://www.jstor.org/stable/j.ctvjsf3ts) | Hold total stakeholder wealth constant; vary concentration, existing associations, and contribution enforcement. |
| **Access and policy success are different outcomes.** [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.1111/ajps.12180) | Reproduce an access intervention without hard-coding any change in votes. Observe whether subsequent information or bargaining creates one. |
| **Connections have person- and office-specific value.** [Benny](https://benny.aeaweb.org/articles?id=10.1257%2Faer.102.7.3731) | Remove a connected officeholder. Intermediaries dependent on that connection should lose more than diversified intermediaries. |
| **Favoritism is concentrated in controllable institutions.** [OUP Academic](https://academic.oup.com/qje/article-abstract/120/4/1371/1926665) | Give public and independent lenders identical borrowers but different appointment and oversight structures. |
| **Political protection can increase firm survival without increasing productivity.** [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.3982/ECTA18338) | Compare connected and unconnected firms conditional on initial productivity; inspect entry, exit, credit, and innovation. |
| **Changing formal representation need not immediately change economic privileges.** [National Bureau of Economic Research](https://www.nber.org/papers/w12108) | Expand the franchise while leaving finance, appointment networks, and coercive resources unchanged; compare with broader reform. |
| **Public participation can coexist with elite control.** [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/674988) | Distinguish voluntary organization, dependence on patrons, and the ability to oppose leaders. |
| **Reform benefits can be delayed and complementary to new opportunities.** [MIT Open Scholarship](https://dspace.mit.edu/entities/publication/3f430882-a892-4b39-ac6b-3f1337ae556e) | Remove barriers before and after a technological or trade opportunity; compare entry and growth over subsequent decades. |

The numerical results in Section 2 provide corresponding empirical moments. Use them in appropriately configured scenarios, with distributions across seeds rather than a single exact match.

### Recommended diagnostic outputs

These are proposed simulation measurements:

**Political closure:** share of appointments going to incumbent families; number of eligible outsiders; entry into the governing coalition; appointment concentration.

**Economic privilege:** excess credit or procurement conditional on observable qualifications; exemptions; barriers facing entrants; concession ownership.

**Institutional independence:** concentration of appointment and funding relationships; whether complaints against patrons proceed; enforcement differences between otherwise comparable actors.

**Reproduction and reversal:** persistence of family advantages across deaths, bankruptcies, elections, and constitutional changes.

**Distributional and productive effects:** household consumption, enterprise productivity, innovation adoption, prices, entry, and resources spent influencing or defending rules.

A single aggregate “capture index” can be a UI summary, but retain the underlying dimensions. A guild-controlled licensing board and a financier-controlled treasury are not interchangeable states.

---

## 5. Modeling recommendation for TCE

### 5.1 Represent decision rights, dependencies, and transactions

The following architecture is a proposed design rather than a claim that one existing model has validated the complete system.

| Entity | Minimal relevant state |
| --- | --- |
| **Person** | Assets, occupation, skills, public-service norms, political beliefs, perceived risks, memberships, relationships, offices, and remembered obligations |
| **Household or lineage** | Shared assets, inheritance rules, kinship, marriage links, dependents, and internal allocation |
| **Organization** | Treasury, members, dues, internal leadership, policy interests, productive assets, and ability to mobilize members |
| **Office or governing body** | Jurisdiction, powers, eligibility, selection method, tenure, agenda control, vetoes, appointment rights, and oversight |
| **Privilege or rule** | Beneficiaries, eligibility conditions, economic effect, obligations, transferability, inheritance, duration, and revocation procedure |
| **Influence interaction** | Participants, requested action, resources transferred or promised, information supplied, secrecy, evidence, and resulting obligations |

Organizations should be able to perform multiple functions. A guild may simultaneously train apprentices, certify goods, insure members, collect dues, field armed supporters, and lobby against entry. These functions must not collapse into a single “guild strength” variable.

### 5.2 Make political investment compete with productive investment

For organization \(g\), define the expected value of policy \(p\), conditional on initially securing it:

\[
V\_{g,p}
=
\sum\_{h=1}^{H}
\frac{
q\_{p,h}\,
\mathbb{E}[\pi\_g(p,h)-\pi\_g(p\_0,h)]
}{
(1+r\_g)^h
}.
\]

Here:

* \(p\_0\) is the current policy.
* \(q\_{p,h}\) is the probability the privilege still operates at horizon \(h\).
* \(\pi\_g\) is income or profit under the corresponding policy.
* \(r\_g\) is the organization’s discount rate.

An influence action is attractive when

\[
\Delta P\_{\text{success}}\;V\_{g,p}
>
C\_{\text{cash}}
+
C\_{\text{time}}
+
\mathbb{E}[C\_{\text{sanctions}}].
\]

This need not be solved optimally. Agents can estimate returns from recent experience, imitate successful peers, evaluate a small set of actions, and overestimate their influence.

Crucially, the organization must also consider a workshop expansion, a new trading route, debt repayment, or improved equipment. Otherwise political spending has no meaningful opportunity cost.

For office purchase, use the same logic: expected legal income, private advantages, and status determine willingness to pay, constrained by available finance and the risks of removal or confiscation.

### 5.3 Keep access separate from official preferences

A possible meeting-allocation score is

\[
z\_{g,j}
=
a\log(1+s\_{g,j}/s\_0)
+bT\_{g,j}
+cI\_{g,j}
+dR\_{g,j},
\]

where \(s\) is politically relevant support known to official \(j\), \(T\) is relationship strength, \(I\) is expected informational usefulness, and \(R\) represents constituency relevance.

Allocate scarce attention among requests using a bounded choice rule. Include ordinary petitions and a “no meeting” alternative.

The official’s eventual policy utility should be separate:

\[
U\_j(p)=
w\_W\widehat{W}\_j(p)
+w\_C\widehat{C}\_j(p)
+w\_H\widehat{H}\_j(p)
+w\_NN\_j(p)
+w\_KK\_j(p)
-\widehat{S}\_j(p).
\]

These terms represent perceived public benefit, constituency benefit, household benefit, normative agreement, career consequences, and expected sanctions. Put them on a common utility scale.

This allows sincere public-interest officials, factional loyalists, careerists, and corrupt officials to coexist. It also allows an official to support a beneficial policy for self-interested reasons.

**Respect information boundaries:** the simulation ledger may know a bribe occurred, but other agents should learn through observation, records, disclosure, investigation, or gossip.

### 5.4 Model the complete accountability chain

For misconduct, a useful decomposition is

\[
P(\text{sanction})
=
P(\text{audit})
P(\text{detection}\mid\text{audit})
P(\text{proceeding}\mid\text{detection})
P(\text{execution}\mid\text{proceeding}).
\]

These are conditional probabilities, not an assumption of independent stages.

Competent investigators can detect misconduct while a captured prosecutor prevents proceedings. An independent court may rule against an elite while dependent guards refuse to act.

Audits should also consume actual administrative capacity. A polity cannot investigate every transaction merely because a slider says “100% audit.”

### 5.5 Make closure an ordinary institutional action

Agents should be able to propose:

> Require sponsorship by two existing members.

> Exempt current license holders from new requirements.

> Reserve specified offices for members of recognized households.

> Give the council authority to nominate its successors.

> Delegate certification to the producers’ association.

Each proposal travels through TCE’s existing lawmaking procedure. Its effects follow from changed eligibility, appointments, and enforcement.

This produces closure without an authored “oligarchy event.” It also permits partial reversals: sponsorship might disappear while inherited concessions remain.

### 5.6 Scale political detail to 10,000–50,000 people

Use **sparse individual relationships plus explicit organization membership**, not all-to-all influence calculations.

Ordinary citizens need not reconsider institutional capture daily. They can update political participation after salient events: a denied license, tax demand, price shock, patron’s request, public scandal, or contested appointment.

Organizations can reconsider strategy monthly. Succession, elections, appointments, marriages, and major policy changes should be event-driven. Calculate prospective policy effects for affected sectors and households rather than recomputing the entire economy for every possible law.

Represent important intermediaries and decision-makers individually. Aggregate routine member contributions where doing so preserves distributional differences.

Visible meetings, gifts, petitions, ceremonies, and investigations can be generated from these causal events for UE5. Their occurrence should reflect simulation decisions, not be merely decorative animations.

### 5.7 Existing models and games worth borrowing from

| Model or game | Useful component | What not to import uncritically |
| --- | --- | --- |
| **Grossman–Helpman, “Protection for Sale”** | Competing organized interests and a government balancing political contributions with welfare | A specialized trade-policy model is not a complete theory of every polity. [National Bureau of Economic Research](https://www.nber.org/papers/w4149) |
| **Acemoglu–Robinson, “Persistence of Power, Elites, and Institutions”** | Substitution between formal authority and informal influence | Its equilibrium persistence results depend on assumptions; they are not an inevitability. [National Bureau of Economic Research](https://www.nber.org/papers/w12108) |
| **Hammond, “Endogenous Transition Dynamics in Corruption”** | Heterogeneous agents and endogenous transitions between corruption regimes | Emergent behavior in an ABM is not by itself empirical validation. [Brookings](https://www.brookings.edu/articles/endogenous-transition-dynamics-in-corruption-an-agent-based-computer-model/) |
| **Akcigit–Baslandze–Lotti, “Connecting to Power”** | Political connections interacting with firm survival, innovation, and reallocation | Its estimates and institutional setting are not automatically portable across history. [Wiley Online Library](https://onlinelibrary.wiley.com/doi/10.3982/ECTA18338) |
| **Victoria 3’s documented political design** | Readable links among wealth, population groups, legal participation, and political strength | Fixed interest-group templates and historical content are less suitable for TCE’s open-ended institutional emergence. [Paradox Interactive](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-45-elections) |
| **The Guild 3** | Visible connections among businesses, dynasties, bribery, guilds, and town offices | Treat it as an interaction-design reference, not evidence about historical frequencies or economic effects. [Steam Store](https://store.steampowered.com/app/311260/The_Guild_3/) |

### An example that should emerge without scripting

A town needs funds for defense. Merchants offer a loan in exchange for exclusive market rights. The concession initially improves provisioning, but its beneficiaries later require incumbent sponsorship for new traders. Protected profits support marriages, education, and appointments connecting their households to the council. An outsider introduces a cheaper technique but cannot obtain a license.

What happens next should depend on the world: consumers might organize, a rival financier might support reform, the ruler might revoke the concession, incumbents might adopt the technique themselves, or the outsider might migrate. No single outcome needs to be prescribed.

---

## 6. Sources, datasets, and limits of the evidence

### 6.1 Priority scholarly foundations

For the theoretical core, use **Stigler’s “The Theory of Economic Regulation,” Olson’s *The Logic of Collective Action*, and Dal Bó’s “Regulatory Capture: A Review.”** Together they distinguish economic stakes, organization, and institutional mechanisms. [JSTOR](https://www.jstor.org/stable/3003160)

For micro-level political processes, prioritize **Hall and Deardorff on legislative subsidies**, **Kalla and Broockman on access**, and **Blanes i Vidal, Draca, and Fons-Rosen on political connections**. Their value is in distinguishing mechanisms that an aggregate influence score would hide. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/lobbying-as-legislative-subsidy/AE4B5D8AB9C2487BB78C2A51BB53E03F)

For historical institutional reproduction, the most directly applicable cases here are **Puga and Trefler’s Venice study**, **Padgett and Ansell’s Florentine network analysis**, **Salzmann’s Ottoman fiscal study**, and **Guardado’s research on office markets and oversight**. [DOI](https://doi.org/10.1093%2Fqje%2Fqju006)

### 6.2 Datasets and replication material

| Resource | Useful TCE application | Main caution |
| --- | --- | --- |
| **Puga–Trefler Venice replication data** | Council membership, commercial participation, and noble marriage networks | Elite-focused records with gaps and selective survival. [Diego Puga](https://diegopuga.org/data/venice/) |
| **V-Dem, version 16, March 2026** | Comparative institutional configurations and long-run political change | Expert-coded indices are not literal event probabilities; do not compare absolute scores across releases without checking methodology. [V-Dem](https://v-dem.net/data/the-v-dem-dataset/) |
| **World Bank Enterprise Surveys** | Firm experiences of administrative bribery and business constraints | Bribery incidence measures requests across six specified public transactions, not comprehensive state capture. [DataBank](https://databank.worldbank.org/metadataglossary/world-development-indicators/series/IC.FRM.BRIB.ZS) |
| **FEC campaign-finance data and LDA filings** | Observable funding, intermediaries, clients, and reported lobbying activity | Reported finance and lobbying are not direct measures of purchased policy or hidden corruption. [FEC.gov](https://www.fec.gov/data/) |
| **D-PLACE** | Cross-cultural comparison of social organization in ecological and linguistic context | Cross-cultural observations are not a universal chronological ladder. [D-PLACE](https://d-place.org/) |
| **Published experimental and firm-study replication packages** | Separate tests of access, monitoring, connections, and firm dynamics | Reproducing a study requires matching its outcome definition and institutional setting. [Benny](https://benny.aeaweb.org/articles?id=10.1257%2Faer.102.7.3731) |

### 6.3 Contested claims and thin evidence

**Guilds are a genuine empirical and interpretive dispute.** Ogilvie emphasizes privileges, exclusion, and political rent extraction. Epstein and Prak emphasize circumstances in which guild arrangements supported skills, coordination, and innovation. TCE should represent both exclusionary powers and potentially useful services, then calculate their consequences rather than assign all guilds a positive or negative productivity modifier. [OUP Academic](https://academic.oup.com/princeton-scholarship-online/book/23122/chapter-abstract/183980067)

**Profitability is not proof of capture.** Political connections may cause advantages, but successful firms also have more resources to cultivate connections, and elites may select already-profitable sectors. The Tunisia research explicitly confronts this problem. Prefer changes around appointments, exits, close political contests, and randomized interventions when identifying particular causal links. [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0304387816300608)

**Prehistoric capture rates are largely unidentifiable.** Evidence for inherited wealth, dependency, or stratification does not reveal a numerical probability that a leader accepted a gift or favored relatives. Ethnographic comparisons help identify mechanisms and countermechanisms, but should not be treated as direct observations of all early societies. [PubMed](https://pubmed.ncbi.nlm.nih.gov/27519458/)

**Reversal is not synonymous with replacement.** A new governing family, broader electorate, or public anticorruption campaign can leave the underlying distribution of practical power intact. Conversely, reforms that change entry, oversight, and financing can matter even without replacing every incumbent. [National Bureau of Economic Research](https://www.nber.org/papers/w12108)

**Bottom line for TCE:** implement scarce access, organizations with real budgets, office-specific powers, economic privileges, and inheritable relationships. Let capture be the durable pattern those interactions produce. The most informative question is not “How corrupt is this polity?” but **“Who can obtain which decisions, through what dependencies, and what allows that advantage to survive opposition and succession?”**

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928cf-1c7c-83ea-8e85-35a577b6f059)
