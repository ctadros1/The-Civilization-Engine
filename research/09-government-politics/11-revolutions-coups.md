# Revolutions, coups, and regime change

## A simulation-ready report for The Civilization Engine

**Recommended design:** Treat regime change as three connected but distinct contests: **mobilizing opposition, controlling the organizations that sustain government, and bargaining over the successor order**. A population can be deeply dissatisfied without overthrowing its rulers; a small insider coalition can replace a ruler without changing the regime; and a successful revolution can lose its political gains during constitution-making or counter-revolution. These distinctions are central to the research on military coordination, authoritarian breakdown, and postrevolutionary survival. [Hopkins Press](https://www.press.jhu.edu/books/title/10989/seizing-power)

For TCE, the most important departure from conventional strategy games is therefore:

> **Do not make “unrest reaches 100” create a revolution, or make “revolution wins” install the victorious faction’s preferred government automatically. Model the organizations, defections, bargains, and enforcement that produce those outcomes.**

The numerical evidence below comes from explicitly bounded historical samples. It is substantially stronger for twentieth-century states than for early agrarian societies. **Historical measurements, observational associations, and proposed simulation settings are labeled separately.**

---

## 1. Mechanisms: implementable causal processes

### 1.1 First distinguish what actually changed

TCE should maintain separate identities for the **incumbent leadership, ruling regime, constitution, and territorial polity**. Research comparing stability measures finds that roughly **40–60% of apparent breakdowns in several commonly used leader-, government-, or polity-based measures occur while the underlying autocratic regime survives**. Colpus explicitly distinguishes coups intended to reshuffle leaders from coups intended to change regimes. [Sage Journals](https://journals.sagepub.com/doi/10.1177/2053168015626606)

| Event | What TCE should record | What must not be assumed |
| --- | --- | --- |
| Leadership replacement | Officeholders change | The governing coalition or rules changed |
| Insider coup | State insiders attempt an unconstitutional executive replacement | Mass participation, democracy, or constitutional replacement |
| Popular revolutionary challenge | Organized mass action challenges the existing political order | Success, a unified opposition, or predominantly armed action |
| Regime transformation | Fundamental rules of political access or the effective ruling coalition change | Territorial collapse or destruction of administration |
| Constitutional replacement | A new fundamental institutional package is adopted | A new regime, or effective implementation |
| State fragmentation or collapse | Central authority loses sustained administrative or territorial control | All successor communities stop governing themselves |

These are **overlapping event tags**, not mutually exclusive boxes. A popular uprising may provoke an insider coup, followed by a negotiated transition, followed by another coup.

### 1.2 Structural pressure creates vulnerability—not an automatic uprising

Goldstone’s comparative work connects early-modern breakdowns to interacting fiscal distress, demographic pressure, elite competition, and institutional inflexibility across European and Asian cases. Skocpol emphasizes the interaction of state organizations, international pressures, and class relations rather than treating revolution as a direct expression of popular anger. [Routledge](https://www.routledge.com/Revolution-and-Rebellion-in-the-Early-Modern-World-Population-Change-and/Goldstone/p/book/9781138222120?utm_source=chatgpt.com)

**TCE rule:** Calculate the consequences of crises through existing economic and institutional systems.

A harvest failure can reduce household food availability, taxable production, market demand, and government provisioning. War can simultaneously increase expenditure and destroy revenue sources. Whether these pressures destabilize government depends on reserves, borrowing, tax bargaining, redistribution, and which groups absorb the losses.

A useful accounting identity is:

\[
\text{arrears}\_{t+1}
=
\max\left(0,\text{arrears}\_{t}
+\text{obligations due}\_{t}
-\text{payments made}\_{t}\right)
\]

Maintain arrears **by recipient organization**. Unpaid soldiers, creditors, palace retainers, and local officials should not react identically.

**Military defeat should affect several variables, not apply a fixed revolution bonus:** surviving force strength, prestige, recruitment burdens, fiscal resources, territorial access, and confidence in commanders. A defeat can undermine the incumbent, but its political consequences depend on who remains organized and capable of replacing that incumbent. This organizational emphasis is consistent with comparative research finding military characteristics important to coup attempts and outcomes. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/0022002712445732?utm_source=chatgpt.com)

**Modeling qualification:** Severe deprivation should also reduce some agents’ ability to organize. A hungry person may flee, seek work, or depend more heavily on a patron rather than join a movement. This follows from modeling participation costs explicitly instead of assuming that greater hardship always increases effective opposition.

### 1.3 Grievances require organization, information, and feasible action

Kuran’s model explains why public loyalty can conceal private opposition: individuals weigh personal preferences against social pressures and anticipated consequences. Once visible opposition changes those expectations, participation can increase abruptly. The resulting cascade need not correspond to a sudden change in underlying material conditions. [Sites@Duke Express](https://sites.duke.edu/timurkuran/1989/04/01/article-sparks-and-prairie-fires-a-theory-of-unanticipated-political-revolution/)

**TCE rule:** Give each person separate values for:

* private evaluation of the regime;
* public behavior;
* organizational affiliation and trusted contacts;
* perceived probability of punishment and political success.

A compact decision rule is:

\[
U\_i(a)=
B\_i(a)
+\eta\_i\,S\_i(a)
+\kappa\_i\,O\_i(a)
-C\_i(a)
-\widehat p\_i(\text{punishment}\mid a)L\_i(a)
\]

Here, \(a\) can be obedience, petitioning, refusal, protest, flight, or joining an armed organization. \(B\_i\) includes material and ideological benefits; \(S\_i\) is perceived social support; \(O\_i\) represents organizational assistance; \(C\_i\) includes time, income, provisioning, and travel costs.

This is a **proposed TCE formulation**, not an estimated historical equation. Decisions can be satisficing or noisy rather than perfectly optimized.

Organizations should supply coordination, meeting opportunities, resources, and continuity. Their forms can vary: kin groups, villages, religious institutions, occupational associations, parties, or military units. NAVCO’s campaign-year data are particularly useful because they separately record participation, elite behavior, repression, external support, and alternative institutions. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/00223433221092938?utm_source=chatgpt.com)

**Do not use the “3.5% rule” as a victory threshold.** Chenoweth explicitly treats it as a descriptive tendency, not an invariant law; organization, momentum, leadership, and sustainability also matter. [Harvard Kennedy School](https://www.hks.harvard.edu/index.php/faculty-research/policy-topics/advocacy-social-movements/35-rule-understanding-what-makes-protest?utm_source=chatgpt.com)

### 1.4 Coup mechanics: willingness, coordination, and resistance are separate

Singh’s central finding is that military coups are often better understood as **coordination contests** than as straightforward battles or popularity contests. Officers respond to what they believe other officers will do; the organizational level from which a coup originates also matters. [Hopkins Press](https://www.press.jhu.edu/books/title/10989/seizing-power)

**TCE implementation:** Give each coercive organization—and important subunits—four possible alignments:

`support_incumbent | support_challenger | remain_neutral | fragment`

An alignment decision should depend on expected survival, corporate interests, personal loyalties, material support, legitimacy, and beliefs about other units. Ordinary soldiers need not automatically share their commander’s choice.

Separate:

\[
P(\text{attempt})
\quad\text{from}\quad
P(\text{success}\mid\text{attempt})
\]

This distinction prevents misleading mechanics. Providing the military with more resources may reduce dissatisfaction while increasing its ability to prevail if it nevertheless intervenes. Powell’s study explicitly investigates this willingness–capability distinction. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002712445732?utm_source=chatgpt.com)

Likewise, rival security organizations should not simply give a generic “coup protection” modifier. De Bruin finds that counterbalancing helps principally by creating forces with incentives to **resist** a coup, rather than merely obstructing coordination. [Sage Journals](https://journals.sagepub.com/doi/10.1177/0022002717692652?utm_source=chatgpt.com)

**Trade-off:** Removing powerful insiders may reduce their access to the center while pushing their networks toward rebellion. Roessler documents this coup–civil-war substitution in African personalist regimes. Consequently, a purge should move people, assets, and loyalties through TCE’s world—not delete their political relevance. [Cambridge University Press](https://www.cambridge.org/core/journals/world-politics/article/enemy-within-personal-rule-coups-and-civil-war-in-africa/0113FB019766DF1280A7F36E326EE696?utm_source=chatgpt.com)

### 1.5 Revolutionary victory is a change in effective authority

Mass participation matters partly because it can disrupt governance and induce defections among a regime’s supporters. It should not be represented as a simple headcount contest against the entire army. [Journal of Democracy](https://www.journalofdemocracy.org/articles/the-future-of-nonviolent-resistance-2/)

**TCE rule:** Track, by settlement and organization, who can actually obtain:

| Government function | Observable evidence |
| --- | --- |
| Revenue | Taxes, tribute, dues, or contributions delivered |
| Compliance | Orders followed, judgments accepted, regulations obeyed |
| Coercive support | Units obeying commands, remaining neutral, or refusing |
| Administration | Officials working, records available, supplies distributed |
| Territorial authority | Local bodies recognizing an authority and accepting its decisions |

A revolutionary situation can contain **multiple claimants exercising different functions simultaneously**. The ruler’s departure does not resolve this automatically.

Possible outcomes should include concessions, negotiated succession, an opposition government, military tutelage, prolonged dual authority, civil war, partition, or administrative disintegration.

For nonscripted worlds, international effects should work through ordinary mechanisms: foreign resources, sanctuary, trade, diplomatic recognition, and the expectations created by events elsewhere. Evidence that post-Cold-War coup outcomes differed with international incentives argues against making external reactions historically constant. [Cambridge University Press](https://www.cambridge.org/core/journals/british-journal-of-political-science/article/coups-and-democracy/505910A86167FE82C8D7019FF6A829AB?utm_source=chatgpt.com)

### 1.6 What winners build: constitutions as bargains constrained by organization

The victorious side is rarely a single actor with one coherent institutional preference. TCE should distinguish **agreement about removing the old regime** from agreement about property, religion, regional autonomy, political participation, and control of armed force.

Recent research covering **153 civil wars, 1944–2016**, finds that victorious revolutionary-socialist and secessionist rebels were more likely to promulgate new constitutions, whereas victorious incumbents were more likely to amend existing ones. Rebel ideology also predicted aspects of constitutional content. This supports modeling substantive preferences, not only material bargaining power. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/00104140251369319?utm_source=chatgpt.com)

A practical founding process is:

**Provisional authority → representation rules → institutional proposals → bargaining and ratification → implementation → revision or renewed conflict.**

The following are proposed TCE rules.

**Provisional authority.** Preserve surviving offices and essential administration unless actors dismantle them. A revolutionary committee, commander, council, or restored ruler can exercise temporary powers without having settled the permanent constitution.

**Representation.** Determine who participates through institutions that actually exist. Delegates might come from villages, military units, estates, religious bodies, parties, or elections. Representativeness and bargaining power are different variables.

**Proposal generation.** Factions propose packages assembled from known primitives, inherited arrangements, and available institutional innovations. They should not receive an omniscient menu of historically optimal constitutions.

**Bargaining.** Evaluate packages against each faction’s alternatives:

\[
U\_f(C)=
\alpha\_f\,\text{policy fit}
+\beta\_f\,\text{material position}
+\gamma\_f\,\text{future security}
-\delta\_f\,\text{implementation cost}
-\epsilon\_f\,\text{expected exclusion risk}
\]

The relevant comparison is not “Is this ideal?” but “Is this preferable to continued conflict, exit, submission, or another alliance?”

**Ratification.** Apply the agreed procedure, while retaining the possibility of coercion, boycotts, contested authority, and rejected drafts.

**Implementation.** A provision becomes effective only through appointments, budgets, records, adjudication, and obedience. Written rights and actual rights must be separate state variables.

Outgoing elites can also shape the successor arrangement. Albertus and Menaldo identify mechanisms including electoral rules, reserved appointments, legal immunities, territorial arrangements, and demanding amendment thresholds. A transition should therefore permit significant political change alongside retained elite privileges. [Political Science Department](https://www.polisci.washington.edu/research/publications/authoritarianism-and-elite-origins-democracy?utm_source=chatgpt.com)

**Minimum constitutional dimensions for TCE**

| Dimension | Examples of primitives |
| --- | --- |
| Political membership | Citizenship, franchise, eligibility, excluded categories |
| Selection and succession | Inheritance, election, appointment, rotation, council selection |
| Allocation of powers | Executive, assembly, courts, religious authorities, local bodies |
| Economic settlement | Tax authority, property protections, land redistribution, labor status |
| Coercive control | Appointment of commanders, militia obligations, civilian oversight |
| Constraints and adaptation | Vetoes, review, emergency powers, amendment and replacement procedures |

A constitution need not be a written document. In an early agrarian polity, the relevant outcome can be a publicly recognized settlement over offices, obligations, succession, and dispute resolution.

### 1.7 Counter-revolution, consolidation, and restoration

Counter-revolution requires its own **emergence and success processes**. Clarke’s research finds that revolutionary violence is associated with lower counter-revolutionary success, principally through the successor regime’s coercive organization—not a corresponding disappearance of counter-revolutionary challenges. This is observational evidence, not a recommendation that violence improves overall outcomes. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/revolutionary-violence-and-counterrevolution/F39A56769C951BA7EE35166F03C4A80D?utm_source=chatgpt.com)

**TCE rule:** Retain former elites’ surviving wealth, clients, legitimacy claims, organizational links, and foreign relationships. Let them choose between accommodation, withdrawal, participation, and attempted restoration.

Founding conflict can also reshape the victors. Levitsky and Way argue that surviving intense counter-revolutionary conflict can strengthen ruling-coalition cohesion, loyal coercive institutions, and the destruction of alternative power centers. But some revolutionary regimes are destroyed during precisely this process: **do not grant an immediate “revolutionary durability bonus.”** [Politics at U of T](https://www.politics.utoronto.ca/research-publications/faculty-publications/revolution-and-dictatorship-violent-origins-durable?utm_source=chatgpt.com)

Restoration should mean **a new settlement drawing on an older claim to authority**, not a save-state rollback. France’s 1814 Charter, for example, restored monarchy while protecting property, including property classified as national, and forbidding investigations of pre-restoration opinions and votes. [Wikisource](https://en.wikisource.org/wiki/French_Constitutional_Charter_of_1814?utm_source=chatgpt.com)

---

## 2. Parameters: empirical anchors and proposed settings

### 2.1 Evidence conventions

**H — High descriptive confidence:** a clear count or coding rule within the stated dataset.  
**M — Moderate:** an observational relationship or comparison with material identification or measurement limitations.  
**L — Low transferability:** extrapolation across very different institutions, scales, or eras.

A number can be **H within its source sample and L as a universal TCE parameter**.

### 2.2 Historical calibration targets

| Quantity | Value and units | Population, period, and source | Confidence and use |
| --- | --- | --- | --- |
| Coup attempts | **457 attempts**, including **227 successes** | Powell–Thyne’s original global sample, **1950–2010** | H descriptive; historical benchmark |
| Coup success conditional on an attempt | **49.7%** | Same sample | H descriptive; not a universal probability |
| Global frequency in that sample | **7.49 attempts/year**, **3.72 successes/year** | Calculated from those counts over 61 calendar years | H arithmetic; **not a polity-year hazard** |
| Operational coup-success threshold | Control for **at least 7 days** | Powell–Thyne coding | H definition; not a consolidation threshold |
| Revolutionary episodes | **345 episodes**, or **3.00 recorded episodes/global year** | Beissinger, **1900–2014** | H count within definition; includes unsuccessful episodes |
| Completed maximalist campaign success | Approximately **51% nonviolent; 26% violent** | Chenoweth’s comparison of **565 completed campaigns**, within **1900–2019** | H/M descriptive; selection and outcome coding matter |
| Recent-period campaign success in that comparison | **Below 34% nonviolent; 8% violent** | **2010–2019** | M; demonstrates nonstationarity |
| Average peak nonviolent participation | About **2.7% of population** in the 1990s; **1.3%** in the 2010s | Decade averages reported by Chenoweth | M; these are averages, not victory thresholds |
| Restorative challenge after revolution | **65/123 = 52.8%** experienced at least one challenge | Clarke’s successful revolutions, **1900–2015**, examining the first decade | H/M within coding |
| Successful restoration after revolution | **22/123 = 17.9%** were overturned; **22/98 = 22.4%** of discrete challenges succeeded | Same study; some regimes faced multiple challenges | H/M; denominators are not interchangeable |
| Rebellion after exclusion of former allies | Approximately **16 times** the likelihood while represented, during the first **3 years** after exclusion | Roessler’s African sample, former coconspirators and their coethnics | M association; L universal transportability |
| Constitutional inheritance in new democracies | **66%** inherited a constitution designed under dictatorship | Albertus–Menaldo’s coding, **1800–2006** | M; all new democracies, not just revolutions |
| Constitutional longevity | Only about half survive beyond **9 years** | Elkins–Ginsburg–Melton’s historical study | M; constitutional replacement is not identical to regime failure |

Sources for the coup measures: Powell and Thyne. [Jonathan Powell, Ph.D.](https://jonathanmpowell.com/wp-content/uploads/2022/11/powell-thyne-2011jpr-global-instances-of-coups.pdf)  
Revolutionary episodes: Beissinger. [Mark R. Beissinger](https://mbeissinger.scholar.princeton.edu/home)  
Campaign outcomes and participation: Chenoweth. [Journal of Democracy](https://www.journalofdemocracy.org/articles/the-future-of-nonviolent-resistance-2/)  
Counter-revolution: Clarke. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/revolutionary-violence-and-counterrevolution/F39A56769C951BA7EE35166F03C4A80D)  
Exclusion, inheritance, and constitutional longevity: Roessler; Albertus and Menaldo; Elkins, Ginsburg, and Melton. [Cambridge University Press](https://www.cambridge.org/core/journals/world-politics/article/enemy-within-personal-rule-coups-and-civil-war-in-africa/0113FB019766DF1280A7F36E326EE696?utm_source=chatgpt.com)

**Do not combine these datasets into one numerator.** Revolutionary episodes, maximalist campaigns, successful political revolutions, and coups are different observational units. For example, dividing Clarke’s 123 successful revolutions by Beissinger’s 345 episodes would require reconciling versions, dates, and inclusion rules first.

### 2.3 Regional coup variation

Powell–Thyne’s original Figure 1 reports the following regional distribution for **1950–2010**:

| Region, using the source’s categories | Attempts | Successful |
| --- | --- | --- |
| Africa | 169 | 51.5% |
| Americas | 145 | 48.3% |
| Asia | 59 | 55.9% |
| Middle East | 72 | 45.8% |
| Europe | 12 | 33.3% |

These are **event distributions and conditional success rates**, not regional annual risks. The denominators do not adjust for numbers of states, state-years, or institutional composition. [Jonathan Powell, Ph.D.](https://jonathanmpowell.com/wp-content/uploads/2022/11/powell-thyne-2011jpr-global-instances-of-coups.pdf)

For TCE, reproduce regional differences through differing institutions and histories—not intrinsic regional coefficients.

### 2.4 Estimating a usable base rate

For a matched institutional stratum, estimate:

\[
\widehat\lambda\_{\mathrm{attempt}}
=
\frac{\text{number of coup attempts}}
{\text{total at-risk polity-years}}
\]

Separately estimate the fraction of polity-years containing at least one attempt; this differs from event frequency when multiple attempts occur in a year.

Include peaceful observations. Dividing attempts only by the years of countries that experienced coups selects for instability.

If a residual stochastic hazard is necessary:

\[
P(\text{event in }\Delta t)=1-e^{-\lambda\Delta t}
\]

with \(\lambda\) and \(\Delta t\) expressed in matching units. **Do not roll a national coup probability separately for every person or every simulation tick.**

I have not reconstructed a harmonized country-year exposure panel here, so the global averages above are not presented as country-level hazards. The modern datasets also do not establish defensible universal annual rates for early farming communities.

### 2.5 Proposed engineering priors—not historical measurements

These are starting ranges for sensitivity experiments, **not claims about real-world frequencies or universal human psychology**.

| Setting | Initial range | Units | Purpose and confidence |
| --- | --- | --- | --- |
| Routine political reconsideration | 7–30 | simulation days | Reduce unnecessary per-frame decisions; engineering choice |
| Crisis reconsideration | 1 | simulation day | Allow rapid changes without instantaneous global coordination |
| Fast political-belief memory | 7–60 | days, half-life | Test how quickly agents revise expectations; empirically uncalibrated |
| Personal grievance memory | 0.5–5 | years, half-life | Test persistence; distinguish personal memories from institutionalized narratives |
| Arrears stress tests | 0–6 | scheduled pay periods | Explore recipient-specific reactions; **no automatic revolt threshold** |
| Founding-process scenarios | 3–36 | months | Test rapid settlements and prolonged bargaining; not a forced completion timer |
| Post-transition observation | 1, 5, 10 | years | Record immediate, medium-term, and consolidation outcomes |

Prefer measuring actual food shortages, arrears, organizational membership, and losses over adding numerous unexplained “revolution pressure” coefficients.

---

## 3. Variation across eras and regions

### 3.1 Use capabilities, not historical stages

The following are comparative patterns and modeling implications, **not a sequence every TCE society should follow**.

| Social setting | Relevant breakdown processes | Likely institutional changes to represent | Evidence limitations |
| --- | --- | --- | --- |
| **Foragers and other decentralized communities** | Withdrawal of cooperation, collective sanctions against domineering individuals, leadership replacement, group division | Changes in leadership expectations, coalition membership, residence, and decision customs | Boehm’s reverse-dominance model supplies ethnographic mechanisms, not prehistoric coup rates. Do not assume all foragers were politically identical. |
| **Early farming communities** | Disputes over stored surplus, labor obligations, access to land, and dependence on local leaders; departure where feasible | New councils, ritual or managerial offices, altered tribute obligations, settlement division | Archaeological evidence often cannot identify whether abandonment or destruction represents revolt, migration, conquest, or another process. Treat numerical rates as uncertain. |
| **Preindustrial states** | Fiscal breakdown, dynastic or elite rivalry, succession disputes, military pressures, rural resistance, provincial fragmentation | Replacement dynasties, altered elite bargains, tax settlements, local autonomy, rebuilt administrations | Comparative work includes Ottoman and Chinese crises as well as England and France; mechanisms are better supported than universal thresholds. |
| **Industrializing societies** | Larger occupational organizations, urban mobilization, mass recruitment, ideological programs, and national integration | Broader political membership, land and labor settlements, stronger central organizations, parties and military regimes | Urbanization changes opportunities for disruption and coordination; it does not guarantee democratic outcomes. |
| **Modern states** | Coups, mass campaigns, negotiated transitions, insider institutional changes, external sponsorship, and counter-revolution | New constitutions, amendments, competitive systems, military tutelage, party dominance, or personal rule | Richer datasets exist, but outcomes and campaign success vary by period and definition. |

The principal foundations are Boehm for decentralized sanctions; Scott’s interpretation of early-state dependence and evasion; Goldstone for comparative early-modern crises; and Beissinger for urbanization and revolution. Scott’s account is an influential interpretation, not an uncontested universal description of early states. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/204166?utm_source=chatgpt.com)

For preindustrial administration, Blanton and colleagues provide another useful mechanism: in comparatively collective systems, rulers’ violations of expected obligations can undermine taxpayer cooperation, services, and interconnected economic activity. Their limited comparative sample should motivate feedbacks, not a law that “good government collapses faster.” [Frontiers](https://www.frontiersin.org/journals/political-science/articles/10.3389/fpos.2020.568704/full?utm_source=chatgpt.com)

### 3.2 What different winners actually built

| Case | Historical institutional outcome | TCE implication |
| --- | --- | --- |
| **Japan, 1868 onward** | Restoration of imperial authority became part of a much larger transformation from decentralized warrior rule toward a centralized modern state | “Restoration” and major institutional innovation can coexist. Traditional legitimacy can support new institutions. |
| **Mexico, 1917** | The revolutionary constitution combined political institutions with major education, land, and labor provisions; Article 123 included an eight-hour workday and collective labor rights | A constitutional settlement should change the economy and social obligations, not just executive selection. |
| **Iran, 1979 settlement, revised 1989** | The constitutional order combined elected institutions with religious leadership and clerical supervision | Allow mixed institutional packages. Do not force every constitution onto one democracy–autocracy slider. |
| **South Africa, 1993–1996** | A negotiated transition used an interim constitution, an elected constitutional assembly, and judicial certification against 34 agreed principles | Distinguish binding prior bargains, drafting authority, and review. This is a negotiated-transition comparison, not an interchangeable example of an armed revolution. |

Sources: Jansen; the Library of Congress’s original-constitution exhibit; Iran’s constitutional text; and South Africa’s constitutional certification judgment. [Cambridge University Press](https://www.cambridge.org/core/books/abs/cambridge-history-of-japan/meiji-restoration/805521FF46C88C15E64576184DB63A58?utm_source=chatgpt.com)

Across these cases, the transferable lesson is **institutional recombination**. The labels “revolution,” “restoration,” and “democratization” do not uniquely determine the resulting constitution.

---

## 4. Stylized facts a correct simulation should reproduce

These should be **validation targets for appropriately matched scenarios**, not mandatory outcomes in every world.

**Uneventful periods coexist with concentrated crises.** The original coup data show clustering and changing frequency over time, rather than an even sequence of independent events. A simulation should generate persistent differences in vulnerability and occasional cascades. [Jonathan Powell, Ph.D.](https://jonathanmpowell.com/wp-content/uploads/2022/11/powell-thyne-2011jpr-global-instances-of-coups.pdf)

**Leadership instability exceeds regime instability.** Frequent officeholder replacement need not imply frequent reconstruction of fundamental institutions. Validate separate survival curves for leaders, regimes, constitutions, and states. [journals.sagepub.com](https://journals.sagepub.com/doi/10.1177/2053168015626606)

**Attempt and success have different determinants.** More capable armed organizations can be less willing to intervene yet more effective when they do. Counterweights can change resistance after an attempt without eliminating the underlying grievance. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/0022002712445732?utm_source=chatgpt.com)

**Public opposition can grow discontinuously.** Sudden participation changes should sometimes arise from revised beliefs about other people, not from a sudden global increase in hardship. [Sites@Duke Express](https://sites.duke.edu/timurkuran/1989/04/01/article-sparks-and-prairie-fires-a-theory-of-unanticipated-political-revolution/)

**Overthrow and durable transformation are different achievements.** Reproduce both successful removal and subsequent reversal. Clarke’s approximate **53% challenge / 18% overthrow** distinction is a useful matched-sample target. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/revolutionary-violence-and-counterrevolution/F39A56769C951BA7EE35166F03C4A80D)

**Popular campaigns should not require majority participation.** Validate participant shares, duration, organizational breadth, and defections separately. Do not impose a universal minimum participation percentage. [Harvard Kennedy School](https://www.hks.harvard.edu/index.php/faculty-research/policy-topics/advocacy-social-movements/35-rule-understanding-what-makes-protest?utm_source=chatgpt.com)

**Success rates change with historical conditions.** A model calibrated to the entire twentieth century should not automatically reproduce the 2010s. NAVCO’s updates explicitly document declining nonviolent campaign success and changes in campaign characteristics. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/00223433221092938?utm_source=chatgpt.com)

**Excluding allies can relocate danger rather than remove it.** Lost central access can coexist with retained provincial or community networks and increased rebellion risk. [Cambridge University Press](https://www.cambridge.org/core/journals/world-politics/article/enemy-within-personal-rule-coups-and-civil-war-in-africa/0113FB019766DF1280A7F36E326EE696?utm_source=chatgpt.com)

**Institutional inheritance is common.** The founding moment should normally begin with surviving organizations and previous rules—not an empty constitutional editor. [Cambridge University Press](https://www.cambridge.org/core/books/authoritarianism-and-the-elite-origins-of-democracy/introduction/43F25A6838D8B41DAFF09543043EFDC0?utm_source=chatgpt.com)

**Neither coups nor revolutions should automatically democratize.** Research on coups and democracy disagrees partly over periods, comparison groups, and identification. Competitive elections after a coup are also not the same outcome as consolidated democracy. [Cambridge University Press](https://www.cambridge.org/core/journals/british-journal-of-political-science/article/coups-and-democracy/505910A86167FE82C8D7019FF6A829AB?utm_source=chatgpt.com)

---

## 5. Modeling recommendation for TCE

### 5.1 Represent persons, organizations, and institutions separately

The following is a proposed implementation architecture.

| Layer | Essential state |
| --- | --- |
| **Person** | Material conditions, private preferences, public alignment, trust links, risk tolerance, memories, memberships |
| **Faction or movement** | Goals, organizers, membership, resources, internal cohesion, allies, territorial presence |
| **Coercive organization** | Command structure, personnel, resources, corporate interests, faction links, obedience and fragmentation |
| **Administrative organization** | Staff, records, fiscal obligations, service delivery, recognized authority |
| **Regime** | Effective ruling coalition, political-access rules, succession practices, relationships to coercive bodies |
| **Constitution** | Formal or customary primitives, adoption process, amendments, recognized validity, implementation status |
| **Political episode** | Participants, claims, onset, actions, defections, outcomes, causal links to subsequent episodes |

Use **stable IDs and an event ledger**. A sequence might be:

`FiscalShortfall → Arrears → EliteDefection → MassChallenge → LeadershipExit → FoundingBargain → CoalitionSplit`

This is an example causal record, not a required event chain. It lets the UI explain why a transition happened without retroactively inventing a single cause.

### 5.2 Keep political computation sparse and event-driven

For 10k–50k persons, the proposed priorities are:

**Routine life supplies political inputs.** Food, employment, taxes, conscription, disputes, and repression already occur in TCE. Political state should consume these events rather than simulate a second abstract economy.

**Most coordination happens through organizations and limited contacts.** Avoid comparing every agent with every other agent. Cache faction resources and membership aggregates; update them when people join, leave, pay, defect, or die.

**Increase decision frequency locally during crises.** A distant village should not instantly react to a capital event. Communication should use TCE’s travel and information systems.

**Preserve subunit autonomy.** Aggregating a military unit is efficient, but allow disagreement to trigger desertion, disobedience, or fragmentation. Otherwise one commander becomes an unrealistic switch controlling everyone.

**Separate political judgments from narrative labels.** First simulate changed authority and institutions; then classify the event for the UI as a coup, revolution, restoration, or transition.

### 5.3 What to simplify—and what not to simplify

You can simplify speechmaking, detailed constitutional prose, and every minor patronage relationship. Store a proposal as an institutional package with sponsors, expected effects, and supporting organizations.

Do **not** simplify away:

* the distinction between private preference and public action;
* the distinction between dissatisfaction and organizational capacity;
* the independence of coercive organizations;
* the persistence of defeated elites and surviving administration;
* the difference between constitutional adoption and implementation.

For the early-agrarian start, the most useful minimal system is probably **household grievances + local coalitions + officeholder legitimacy + organized force + negotiated obligations**. Mass parties and professional constitutional assemblies can emerge later when their organizational prerequisites exist, without being tied to a named era.

### 5.4 Existing models and games worth borrowing from

| Model or game | Useful component | Limitation for TCE |
| --- | --- | --- |
| **Kuran’s revolutionary-cascade model** | Hidden preferences and changing expectations | Does not itself supply fiscal administration or constitution-making |
| **Singh’s coup coordination framework** | Organizational loyalties and beliefs about other military actors | Needs integration with TCE’s economy, society, and successor institutions |
| **Epstein’s civil-violence model / NetLogo Rebellion** | A minimal grievance–risk participation model with local enforcement | Random movement, simplified hardship, and global legitimacy are unsuitable as TCE’s complete political model |
| **Victoria 3’s documented revolution design** | Connects political demands, social support, and opposition to particular laws | A revolution clock and authored events are interface/game-design devices, not empirical causal models |

The NetLogo implementation provides inspectable rules; Victoria 3’s developer diaries explain its design explicitly. The cited diaries describe particular versions, not an assertion about every current mechanic. [CCL](https://ccl.northwestern.edu/netlogo/models/Rebellion?utm_source=chatgpt.com)

For commercial development, note that the cited NetLogo model carries a **noncommercial license**; study the mechanism rather than assuming its implementation can be copied into TCE. [Modeling 2](https://modeling2.tech.northwestern.edu/browse/one_model/1472?utm_source=chatgpt.com)

### 5.5 Validation strategy

Run many independent worlds and compare **distributions**, not one dramatic historical replay.

Measure attempt frequency, conditional success, leadership versus regime turnover, mobilization size and duration, defections, territorial loss, constitutional inheritance, and post-transition survival.

Hold out regions or periods when fitting modern parameters. Handle ongoing campaigns and surviving regimes as censored observations, not failures or immortal successes. Evaluate whether multiple parameter combinations fit the same aggregate statistics; otherwise an apparently realistic coup rate can conceal unrealistic individual behavior.

Most importantly, do not scale modern campaigns mechanically into TCE’s population. A world of 50,000 actual people is not automatically a miniature twentieth-century nation-state. Match **organizational structure, participation opportunities, communication, and administrative reach**, not just percentages.

**Illustrative TCE outcome, not a historical claim:** A poor harvest leaves a ruler unable to provision retainers. Village representatives demand lower obligations; a religious institution offers reserves conditional on council oversight; some retainers support replacing the ruler. The resulting settlement might preserve hereditary leadership but transfer taxation to a council. The same harvest shock in another world could produce relief, emigration, a palace replacement, fragmentation, or no regime change at all.

---

## 6. Source base, contested claims, and evidence gaps

### Core datasets to build around

| Source | Best use in TCE research |
| --- | --- |
| **Powell and Thyne, “Global Instances of Coups”** | Coup-event definitions, attempts, outcomes, historical comparison |
| **Chin, Carter, and Wright, Colpus** | Independent coding comparison; military/nonmilitary actors; leader reshuffling versus regime change |
| **Beissinger, *The Revolutionary City*** | Revolutionary episodes, urbanization, mobilization, and changing revolutionary forms |
| **Chenoweth and Shay, NAVCO** | Campaign-year participation, repression, elite behavior, external support, and outcomes |
| **Geddes, Wright, and Frantz, autocratic regimes data** | Regime identities and transitions distinct from leader turnover |
| **Clarke, counter-revolution dataset** | Challenges and reversals after successful revolutions |
| **Comparative Constitutions Project** | Constitutional events and coded institutional content |

Use versioned snapshots. At retrieval, Powell’s site listed an **August 29, 2026** release, while CCP’s **Chronology v6.0** was updated through **2025**. The headline historical figures in this report remain tied to their original publication windows rather than silently mixing releases. [Jonathan Powell, Ph.D.](https://jonathanmpowell.com/coups/) Colpus and NAVCO also have distinct versions and coverage; NAVCO 2.1’s published description covers **389 campaigns, 1945–2013**. [John J. Chin](https://www.johnjchin.com/colpus)

### Claims that should remain uncertain in the simulation’s research documentation

**There is no established universal fiscal tipping point.** Debt ratios, tax burdens, or missed payments have different implications depending on who is owed, what alternatives exist, and how government is organized. Goldstone’s structural approach is useful precisely as an interaction model, not a single threshold. [Routledge](https://www.routledge.com/Revolution-and-Rebellion-in-the-Early-Modern-World-Population-Change-and/Goldstone/p/book/9781138222120?utm_source=chatgpt.com)

**Violent versus nonviolent outcomes are not randomized comparisons.** Campaigns differ in opponents, objectives, organization, and circumstances. Lower counter-revolutionary vulnerability among some violent revolutionary regimes does not establish that violence produces better overall political or human outcomes. NAVCO also finds substantially lower per-capita fatalities in nonviolent campaigns. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/revolutionary-violence-and-counterrevolution/F39A56769C951BA7EE35166F03C4A80D?utm_source=chatgpt.com)

**Constitutional text is an imperfect proxy for power.** Ideological provisions, rights guarantees, and institutional inheritance are observable, but their practical consequences require separate evidence and modeling. [Sage Journals](https://journals.sagepub.com/doi/abs/10.1177/00104140251369319?utm_source=chatgpt.com)

**Prehistoric and early-agrarian rates remain especially thin.** Ethnography, archaeology, and comparative history support possible mechanisms, but they do not justify assigning a precise annual “Neolithic revolution probability.” Uncertainty here should produce broad sensitivity tests—not a default of zero political change.

### Bottom line

For TCE, **a regime should fall when its opponents become sufficiently organized and its supporting organizations can no longer—or no longer wish to—reproduce its authority**. What follows should depend on who retains resources, who can cooperate, what institutions they regard as legitimate, and which promises they can enforce.

The winning faction should enter constitution-making with **advantages, constraints, allies, rivals, and an inherited state**—not with unconditional control of a blank world.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928d1-cc4c-83e9-a76f-b626bcceef78)
