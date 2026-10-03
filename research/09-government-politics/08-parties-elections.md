# Factions, parties, voting, and elections

## A simulation-ready research report for The Civilization Engine

**The central recommendation is to simulate factions before parties, political choices before ballots, and electoral rules independently of both.** A party should emerge when an organization finds it worthwhile to coordinate candidates, supporters, and officeholders—not when a society reaches a predefined technological era.

For TCE, distinguish four things:

**Social categories** describe overlapping interests and identities. **Factions** organize people to pursue influence. **Parties** coordinate electoral competition and political careers. **Governing coalitions** negotiate the exercise of authority. These should be separate entities: a faction can exist without elections, several factions can inhabit one party, and several parties can support one government.

The evidence supports many mechanisms, but not a universal set of voting coefficients. Accordingly, this report separates **observed historical quantities**, **conditional theoretical predictions**, and **proposed simulation parameters**.

---

# 1. Mechanisms: implementable causal processes

## 1.1 Social differences become cleavages through organization

Lipset and Rokkan’s classic account identifies four major sources of European party conflict: center–periphery, state–church, agriculture–industry, and owners–workers. Their broader contribution is not that every society must acquire these four divisions, but that historical conflicts become durable electoral alignments through organizations and institutional opportunities. A social difference does not automatically produce a party. [Janda](https://janda.org/c24/Readings/Lipset%26Rokkan/Lipset%26Rokkan.htm)

**TCE rule:** Generate political grievances from the simulation’s actual relationships: landholding, taxation, wages, religious authority, regional autonomy, military obligations, access to office, and legal status. A potential cleavage becomes politically active when people with related grievances encounter one another, recognize common interests, and obtain organizers capable of coordinating action.

A useful organizing condition is:

\[
\text{expected collective influence}
>
\text{organization costs}+\text{repression risk}+\text{internal disagreement}.
\]

This is an implementation proposal, not an estimated historical equation. Influence can mean changing a law, securing appointments, protecting a community, or defeating another faction—not necessarily winning an election.

**Group boundaries should be politically contingent.** Posner’s comparison of Chewas and Tumbukas in Zambia and Malawi shows that the same cultural distinction can become politically divisive in one national arena and comparatively unimportant in another. Relative group size and competitive incentives help explain the difference. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/political-salience-of-cultural-difference-why-chewas-and-tumbukas-are-allies-in-zambia-and-adversaries-in-malawi/039468BDC4AAB0FC0E9899F459EE2B7A)

Therefore, an agent should not have a fixed “ethnic voting probability.” Membership affects interests, trust, information, and potential allies; which membership matters depends on the contest. A merchant can simultaneously identify with a town, religious community, lineage, occupational association, and province.

## 1.2 Factions do not require parties—or popular elections

Boehm’s research on small-scale societies describes coalitions that constrain would-be dominant individuals. These are political organizations without being modern parties. In a very different setting, Levine documents Northern Song factional conflict organized around officials, appointments, policy, and competing conceptions of legitimate government. Neither case requires mass electoral competition. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/204166)

**TCE rule:** Allow factions to organize around any consequential decision arena: a village council, royal household, army command, religious body, guild, bureaucracy, legislature, or electorate.

A faction needs an agenda, supporters, organizers, resources, and relationships with officeholders. It does **not** need a public name, written manifesto, formal membership register, or electoral platform.

A faction becomes an electoral organization when coordinating nominations and supporters offers more influence than remaining an informal network. Repeated competition can then justify maintaining a recognizable identity, recruiting successors, collecting resources, and disciplining representatives.

Do not replace factions when parties appear. Preserve them inside parties. Internal factions should compete over nominations, leaders, policy commitments, and access to offices.

**Formation and dissolution should follow resources and incentives.** An organizer can found a party when expected gains exceed establishment costs; factions can merge when separate candidacies waste support; organizations can split when leadership or policy disputes outweigh coordination benefits. Party survival should depend on maintained supporters, resources, and influence—not an arbitrary expiration timer. Endogenous entry and exit are already demonstrated in the agent-based framework of Laver and Schilperoord. [RePub](https://repub.eur.nl/pub/60755)

## 1.3 Voting should combine policy, performance, identity, and relationships

Spatial voting treats citizens as preferring political alternatives closer to their own positions. Downs provides an early rational-choice foundation; later computational models show how parties can adapt through bounded behavioral rules rather than omniscient optimization. Retrospective voting adds judgments about previous performance, while group and organizational relationships provide additional reasons for supporting candidates. These are complementary mechanisms, not mutually exclusive voter types. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/257897?utm_source=chatgpt.com)

For TCE, use a mixed utility model. One workable specification is:

\[
U\_{ij}
=
-\beta\_i^S D\_{ij}
+\beta\_i^R R\_{ij}
+\beta\_i^G G\_{ij}
+\beta\_i^L L\_{ij}
+\beta\_i^Q Q\_{ij}
+\beta\_i^P P\_{ij},
\]

where voter \(i\) evaluates candidate or party \(j\), and

\[
D\_{ij}
=
\sum\_k w\_{ik}
\left(\frac{x\_{ik}-\hat z\_{ijk}}{2}\right)^2.
\]

Here:

| Term | Simulation meaning |
| --- | --- |
| \(x\_{ik}\) | Agent’s preferred position on issue \(k\), normalized to \([-1,1]\). |
| \(\hat z\_{ijk}\) | Agent’s **perception** of the candidate’s position—not the true platform. |
| \(w\_{ik}\) | Issue salience; nonnegative and summing to one for politically attentive agents. |
| \(R\_{ij}\) | Recent performance attributed to that political alternative. |
| \(G\_{ij}\) | Expected representation or protection of valued groups. |
| \(L\_{ij}\) | Personal loyalty and accumulated party attachment. |
| \(Q\_{ij}\) | Prior reputation for competence, integrity, or leadership. |
| \(P\_{ij}\) | Expected selective benefits minus risks associated with supporting that alternative. |

The \(\beta\) coefficients vary across agents. **These are proposed model components, not historically measured universal weights.**

Derive policy preferences partly from the economy: a land tax affects landlords and tenants differently; grain restrictions affect producers and consumers differently. But allow moral commitments, identities, and beliefs to override narrow material advantage. Do not translate occupation directly into a predetermined party vote.

For uncertain choice, a softmax is sufficient:

\[
\Pr(i\rightarrow j)
=
\frac{\exp(U\_{ij}/\tau\_i)}
{\sum\_{\ell}\exp(U\_{i\ell}/\tau\_i)}.
\]

Keep the utility scale fixed when calibrating \(\tau\); otherwise utility weights and choice noise are not separately identifiable. For ranked ballots, generate a consistent ranking from one set of candidate evaluations rather than drawing unrelated preferences at every elimination round.

## 1.4 Performance voting needs responsibility attribution and imperfect information

Duch and Stevenson’s comparative research finds that economic voting depends on political and informational conditions, including how voters assess competence and responsibility. Their evidence covers 165 election studies in 19 countries, rather than supporting a single context-free relationship between economic growth and incumbent votes. [Cambridge University Press](https://www.cambridge.org/core/books/the-economic-vote/57D49941B6465119EA9CA9D2D8518903)

**TCE rule:** Record experienced outcomes—food availability, taxes, employment, security, judicial treatment—and update evaluations of actors believed responsible.

Separate three quantities:

1. What happened.
2. Who actually controlled the relevant decision.
3. Who the agent believes deserves credit or blame.

Coalition partners, local officials, rulers, and legislatures can consequently receive different evaluations. Citizens may misattribute a regional drought to an incumbent, but they should not all do so identically.

Use faster decay for recent performance than for long-standing attachments. Reinforcing events can maintain loyalty; repeated disappointment can erode it. Avoid giving everyone perfect knowledge of national conditions or every party’s complete platform.

Clientelism also needs imperfect information. Stokes’s model, supported with evidence from Argentina, emphasizes political machines’ ability to infer behavior through networks. It does not justify giving brokers direct access to every secret ballot. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/perverse-accountability-a-formal-model-of-machine-politics-with-evidence-from-argentina/A2A7141AA5EC213E6C5FAFD86FBCC859)

In TCE, public attendance, association, and turnout may be observable; a secret vote should remain hidden except through explicitly modeled breaches or noisy inference.

## 1.5 Strategic voting and party coordination operate locally

Duverger-style effects combine a **mechanical effect**, whereby rules translate votes into seats, with a **coordination effect**, whereby voters and political entrepreneurs adjust to those rules. Cox’s analysis emphasizes the coordination problems facing both voters and elites. These pressures operate in particular electoral contests; they do not imply that plurality voting mechanically creates exactly two parties nationwide. [Gary Cox](https://gwcox.people.stanford.edu/making-votes-count-abstract)

**TCE rule:** Agents estimate viability within their actual district or selection body. Information can come from previous results, public endorsements, meetings, canvassing, and trusted contacts.

A bounded strategic heuristic is adequate:

> When the favorite appears unable to win, compare viable alternatives and switch only when the expected tactical benefit exceeds the value of expressing loyalty to the favorite.

Do not implement strategic choice as “candidate utility multiplied by probability of winning.” That can produce bandwagon behavior unrelated to whether a vote can affect the result.

At the organizational level, factions should negotiate withdrawals, common candidates, seat-sharing arrangements, and mergers. Geographically concentrated organizations can survive nationally even when they are small overall.

## 1.6 Participation is a separate decision from vote choice

The classic calculus-of-voting literature distinguishes the instrumental benefits of influencing an outcome from participation costs and other benefits of voting. A model relying only on the probability of casting the decisive vote will generally need additional mechanisms to explain substantial participation. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/abs/theory-of-the-calculus-of-voting/500608E51991E92AC96EB6860F1192CA?utm_source=chatgpt.com)

Use an explicit participation pipeline:

\[
\text{legally eligible}
\rightarrow
\text{administratively able to participate}
\rightarrow
\text{participates}
\rightarrow
\text{casts a valid ballot}.
\]

A proposed turnout model is:

\[
\Pr(\text{participate}\_i)
=
\sigma(
\text{duty}
+\text{habit}
+\text{mobilization}
+\text{perceived stakes}
+\text{expected nonparticipation penalty}
-\text{time and income costs}
-\text{danger}
-\text{administrative burden}
).
\]

Convert travel, queues, work conflicts, and fees into costs through the existing daily-life and household systems. Compulsory participation should matter through expected enforcement, not merely through a constitutional flag.

Participation incentives are not exclusively modern. Aristotle discusses arrangements involving attendance payments, fines, and property-linked participation, making institutional costs relevant even to ancient political bodies. [Internet Classics Archive](https://classics.mit.edu/Aristotle/politics.4.four.html)

Finally, effective participation depends on the interface. Fujiwara’s study of electronic voting in Brazil finds that voting technology reduced obstacles to valid participation among less-educated voters. TCE should therefore model ballot complexity, symbols, assistance, and administrative competence separately from formal suffrage. [Research Program in Political Economy](https://rppe.princeton.edu/publications/voting-technology-political-responsiveness-and-infant-health-evidence-brazil-0)

## 1.7 Electoral systems are exact algorithms, not party-count modifiers

Store the franchise, voting unit, nomination rules, district boundaries, district magnitude, ballot format, allocation method, tiers, thresholds, secrecy, and executive-selection procedure independently.

Here, **district magnitude \(M\) means seats elected per district**, not the number of residents.

| System | Counting rule and important distinction |
| --- | --- |
| Single-member plurality/FPTP | The candidate with the highest district total wins its seat. |
| Two-round election | Apply the specified first-round winning condition, finalist rule, and separate runoff. |
| Instant-runoff voting | Eliminate candidates and transfer ranked ballots. It is a single-winner system, not proportional representation. |
| List proportional representation | For d’Hondt, rank quotients \(V\_p/1,V\_p/2,V\_p/3,\ldots\); for standard Sainte-Laguë use divisors \(1,3,5,\ldots\). Allocate the highest \(M\) quotients. |
| Single transferable vote | Use a specified multi-seat quota and transfer procedure. A Droop quota is \(\lfloor V\_{\mathrm{valid}}/(M+1)\rfloor+1\). |
| Mixed-member systems | Distinguish compensatory allocation, as in MMP, from independently allocated parallel tiers. |

These distinctions follow the institutional taxonomy in International IDEA’s electoral-system handbook. Implement the chosen rule’s tie-breaking, exhausted ballots, rounding, thresholds, and transfer variant explicitly. [International IDEA](https://www.idea.int/publications/catalogue/electoral-system-design-new-international-idea-handbook)

**TCE recommendation:** Keep rule choice endogenous. Officeholders can propose rules that protect their position; excluded organizations can demand changes; constitutional constraints can prevent either. Never assume electoral rules are selected impartially.

Also distinguish actual ballots, counted totals, announced results, and acceptance of the outcome when modeling electoral misconduct. Manipulation should operate through concrete administrative powers and actions.

## 1.8 Franchise expansion is a political bargain, not a development milestone

Two influential explanations emphasize different mechanisms. Acemoglu and Robinson model suffrage concessions as a credible commitment under threats of unrest. Lizzeri and Persico show how some elites can favor expansion because it changes political competition toward broader public goods, even without a revolutionary threat. Neither explanation should become TCE’s sole law of democratization. [Voices](https://voices.uchicago.edu/jamesrobinson/2018/06/20/why-did-the-west-extend-the-franchise-growth-inequality-and-democracy-in-historical-perspective/)

**TCE rule:** At a franchise-reform proposal, included actors estimate effects on future control, taxation, public goods, and security. Excluded actors can organize, bargain, withhold cooperation, or challenge authority. Actors compare reform with repression, partial concessions, and maintaining the status quo.

Represent eligibility as predicates over age, residence, citizenship or subject status, property, sex, dependency, occupation, religion, or membership in a recognized body. Keep voting rights, candidate eligibility, and eligibility for particular offices separate.

Expansion must be reversible. It can broaden one dimension while preserving or introducing exclusion along another.

## 1.9 Coalitions bargain over policy, offices, and survival

Coalition formation is not simply “combine the ideologically nearest parties.” Electoral alliances, parliamentary support arrangements, cabinets, and non-electoral ruling coalitions are different relationships.

Gamson’s law describes an approximate relationship between a party’s contribution to a coalition’s legislative seats and its share of cabinet portfolios. Carroll and Cox connect such patterns to bargaining and pre-election coordination. The relevant denominator is **the coalition’s seats**, not all seats in the legislature. [eScholarship](https://escholarship.org/uc/item/085128ff)

**TCE rule:** A party evaluates a proposed coalition through office benefits, policy distance, supporter benefits, reputational costs, anticipated electoral consequences, and expected stability. It accepts only when the package beats its perceived outside option.

Apply the institution’s actual government-formation procedure: who proposes a government, whether investiture is required, what majority is necessary, and how confidence can be withdrawn. Research on European parliamentary governments shows that the particular investiture decision rule matters for minority-government formation. [Sage Journals](https://journals.sagepub.com/doi/10.1177/1354068819850447)

Allow minority cabinets with external support and oversized coalitions. A **minimal-winning** coalition loses its majority when any member leaves; it need not be the coalition with the fewest total seats.

For presidential or non-parliamentary systems, do not import cabinet-confidence logic automatically. Electoral support, legislative cooperation, and executive appointments require their own agreements.

---

# 2. Quantitative anchors and proposed parameters

## 2.1 Evidence-backed quantities and theoretical benchmarks

Confidence below refers to the stated finding **within its scope**. It does not imply equal confidence in transferring the number to another society.

| Quantity | Numerical anchor and units | Confidence and appropriate use |
| --- | --- | --- |
| Social-pressure mobilization | The strongest treatment in Gerber, Green, and Larimer’s Michigan field experiment increased turnout by **8.1 percentage points**. | **High internal causal confidence; low universal transferability.** A benchmark for a specific intervention among registered voters, not a generic bonus for every contact. [J-PAL](https://www.povertyactionlab.org/evaluation/social-pressure-and-voter-turnout-united-states) |
| Compulsory voting | Fowler estimates an Australian turnout increase of **24 percentage points** following compulsory voting. Labor vote and seat shares increased by **7–10 points**. | **Strong context-specific quasi-experimental evidence.** The partisan direction is not a universal consequence of higher turnout. [DOI](https://doi.org/10.1561/100.00012055) |
| Historical parliamentary turnout | IDEA’s historical global series reports approximately **76% in the 1980s**, declining to **66% in 2011–2015**, using registered voters as the denominator. | **Descriptive benchmark.** Country composition and coverage change; these are not current worldwide estimates. [International IDEA](https://www.idea.int/sites/default/files/publications/voter-turnout-trends-around-the-world.pdf) |
| India’s first national election | **173.2 million registered electors** and **45.67% turnout**, according to ECI’s historical table for the first election. | **High confidence as an official historical aggregate.** Preserve the source’s denominator and statistical conventions. [Election Commission of India](https://www.eci.gov.in/about-eci/) |
| Effective number of parties | \(N=1/\sum p\_j^2\), with \(p\_j\) expressed as fractions. Calculate separately for votes and seats. | **Exact descriptive index**, not a party-generation rule. It distinguishes many tiny parties from several consequential ones. |
| Seat-product benchmark | For suitable simple electoral systems, \(N\_s\approx(MS)^{1/6}\), where \(S\) is assembly size. With \(S=100\), \(M=1\) gives **2.15**; \(M=10\) gives **3.16**. | **Conditional aggregate benchmark**, not an exact prediction for one polity or its first election. The example values are calculated from the model. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0261379415001845) |
| Portfolio allocation | Baseline portfolio share \(\approx s\_j/\sum\_{\ell\in C}s\_\ell\), where \(C\) is the governing coalition. | **Empirical regularity, not a binding formula.** Portfolio importance, bargaining institutions, and internal party organization matter. [eScholarship](https://escholarship.org/uc/item/085128ff) |

Do not treat an observed percentage-point effect as a logistic coefficient. To reproduce an intervention, simulate its mechanism and compare the resulting average treatment effect with the study under comparable conditions.

## 2.2 Proposed initial calibration ranges

**Every numerical range in this table is a TCE design proposal, not an observed historical parameter.** They are starting points for sensitivity analysis.

| Parameter | Proposed starting range | Units | Source/status |
| --- | --- | --- | --- |
| Active policy dimensions | **4–8** | Dimensions per polity | TCE proposal; expand or replace dimensions as laws and conflicts change. |
| Political information sampling | **8–32** | Existing social contacts sampled per agent per month | TCE proposal; not a cap on the agent’s total relationships. |
| Recent-performance memory | **6–24** | Months of half-life | TCE proposal; vary with event severity and reinforcement. |
| Party-attachment memory | **1–5** | Election cycles of half-life without reinforcement | TCE proposal; test stronger persistence and intergenerational transmission separately. |
| Strategic-voting propensity | Sweep **0, 0.25, 0.5, 0.75, 1** | Fraction of agents using the strategic heuristic | Sensitivity grid, not a historical distribution. |
| Choice noise \(\tau\) | **0.1–1.0** | Utility units after fixing utility normalization | TCE proposal; jointly inspect aggregate volatility and individual persistence. |
| Background political update | **Monthly**, plus immediate consequential events | Simulation time | Engineering proposal; election participation still occurs on actual scheduled days. |
| Travel and participation cost | **Calculated, not assigned a universal constant** | Hours, lost income, danger, administrative effort | Derived from map distance, transport, queues, work, and institutions. |

For a decaying memory, use the time-step-independent retention factor \(2^{-\Delta t/h}\), where \(h\) is the half-life. This avoids accidentally changing political behavior when the simulation update frequency changes.

There is currently no defensible universal empirical range for “months required to form a faction,” “probability of voting with one’s class,” or “party loyalty inherited from parents.” Those require explicit assumptions and calibration.

---

# 3. Variation across eras and regions

The following are **historical configurations, not stages TCE must traverse**.

| Context | Relevant political organization | What TCE should preserve |
| --- | --- | --- |
| Small-scale foraging societies | Issue coalitions, personal influence, and collective resistance to domination appear in ethnographic research. | Model persuasion, coalition pressure, and exit without presuming formal ballots. Ethnographic cases are not direct measurements of prehistoric political behavior. [RCNi Company Limited](https://www.journals.uchicago.edu/doi/10.1086/204166) |
| Early agrarian societies | Councils and rulers can bargain over information, resources, and cooperation. Ahmed and Stasavage connect early joint governance to conditions affecting rulers’ ability to obtain revenue and information. | Make independent local power and administrative alternatives consequential. Treat the empirical associations as evidence for mechanisms, not a deterministic agriculture-to-democracy formula. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/origins-of-early-democracy/09CE2053657A9CF004DB2078B563EB65) |
| Ancient Mediterranean political bodies | Participation could depend on property and on payments, fines, and attendance rules. | Represent eligibility, physical attendance, compensation, and agenda control separately; “assembly” does not mean equal participation by all residents. [Internet Classics Archive](https://classics.mit.edu/Aristotle/politics.4.four.html) |
| Imperial China, especially Northern Song | Bureaucratic factions contested appointments, policy, and political legitimacy without mass electoral parties. | Permit durable ideological and patronage coalitions inside an appointed administration. [Institute of History and Philology](https://www1.ihp.sinica.edu.tw/en/Publications/AsiaMajor/640/Article/100) |
| Haudenosaunee institutions | Onondaga descriptions emphasize clan mothers’ roles in selecting and removing chiefs and consensus procedures among representative bodies. | Model nomination, accountability, and agreement among bodies rather than translating everything into individual majority ballots. These are Indigenous descriptions of traditional institutions, not proof of an unchanged prehistoric constitution. [Onondaga Nation](https://www.onondaganation.org/government/clan-mothers/) |
| Industrializing Europe | Territorial, religious, agrarian, and class conflicts became organized through historically specific institutions and associations. | Let expanding workplaces and organizations alter recruitment networks; do not automatically generate a socialist and conservative party at an industrial threshold. [Janda](https://janda.org/c24/Readings/Lipset%26Rokkan/Lipset%26Rokkan.htm) |
| Modern African and Latin American cases | Political identities vary with the competitive arena; networks can sustain clientelism; administrative technology can change effective enfranchisement. | Make identity salience, monitoring, and ballot accessibility conditional variables available in every region—not region-specific stereotypes. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/political-salience-of-cultural-difference-why-chewas-and-tumbukas-are-allies-in-zambia-and-adversaries-in-malawi/039468BDC4AAB0FC0E9899F459EE2B7A) |

## Franchise history supplies useful institutional tests

| Case | Historical change | Simulation lesson |
| --- | --- | --- |
| New Zealand, **1893** | Voting rights extended to women who were British subjects aged **21 or older**, including Māori women. | Eligibility can expand across sex while still retaining age and legal-status conditions. [NZ History](https://nzhistory.govt.nz/politics/womens-suffrage/brief-history) |
| United Kingdom, **1918–1928** | In 1918, virtually all men aged **21+** obtained the vote, while women generally faced an age threshold of **30** and qualifications. Equal voting terms at **21** followed in 1928. | Franchise changes can be partial and asymmetric rather than one binary democratization event. [Parliament UK News](https://www.parliament.uk/about/living-heritage/transformingsociety/electionsvoting/womenvote/overview/thevote/) |
| Japan, **1945–1946** | The postwar reform enfranchised men and women aged **20+**; women first participated in the national election of April 1946. | A major political rupture can rapidly change eligibility without generations of incremental expansion. [Library of Congress](https://www.loc.gov/item/2021668781/) |
| India, **1951–1952** | The first national election operated on an adult-franchise basis with an electorate exceeding **173 million**. | Broad suffrage should be an institutional possibility, not the reward for completing a prescribed sequence of restricted elections. [Election Commission of India](https://www.eci.gov.in/about-eci/) |
| Brazil, **1881–1985** | A literacy exclusion introduced in 1881 was reversed in 1985, when illiterate citizens regained voluntary voting rights. | Model contraction and restoration, and distinguish voluntary from compulsory participation. [Justiça Eleitoral](https://www.tse.jus.br/comunicacao/noticias/2016/Novembro/constituicao-de-1985-garantiu-o-direito-ao-voto-aos-eleitores-analfabetos) |
| South Africa, **1994** | The first nonracial national election included adult citizens aged **18+**. | Franchise boundaries can be central to negotiated regime transformation. [Government of South Africa](https://www.gov.za/issues/20years/milestones-20-years-freedom) |

The important generalization is **institutional diversity and reversibility**, not a universal chronology from property voting to universal suffrage.

---

# 4. Stylized facts and validation targets

A correct simulation should reproduce these patterns **under appropriate conditions and across ensembles**, not force every world to display them.

| Pattern | Observable validation target |
| --- | --- |
| Social categories and parties are not interchangeable. | The same distribution of identities can produce different alignments when territorial scale, coalition opportunities, or organizational networks change. Posner’s comparison is a particularly useful qualitative target. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/political-salience-of-cultural-difference-why-chewas-and-tumbukas-are-allies-in-zambia-and-adversaries-in-malawi/039468BDC4AAB0FC0E9899F459EE2B7A) |
| Electoral rules affect both representation and adaptation. | With ballots held fixed, changing the allocation rule changes seats. Across subsequent elections, nomination and strategic behavior should also respond. [Gary Cox](https://gwcox.people.stanford.edu/making-votes-count-abstract) |
| National party counts need not match local competition. | Under plurality, local competition can become concentrated while different parties remain competitive in different regions. Measure district-level and national fragmentation separately. [Cambridge University Press](https://www.cambridge.org/core/books/making-votes-count/42CD9425E1410457FFC5079EC851F32B) |
| Greater seat capacity generally permits more representation. | Under conditions appropriate to the seat-product model, typical effective party numbers should increase with district magnitude and assembly size, without being forced to the predicted value. [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0261379415001845) |
| Participation responds to social and institutional incentives. | Comparable mobilization and compulsory-voting scenarios should be capable of producing substantial turnout effects, including the **8.1-point** and **24-point** study benchmarks—not necessarily those effects in every setting. [J-PAL](https://www.povertyactionlab.org/evaluation/social-pressure-and-voter-turnout-united-states) |
| Economic accountability is conditional. | Performance shocks should affect support differently when responsibility is clear, divided, or poorly understood. A single universal economic-vote coefficient should fail validation. [Cambridge University Press](https://www.cambridge.org/core/books/the-economic-vote/57D49941B6465119EA9CA9D2D8518903) |
| Formal suffrage is not effective participation. | Broad legal eligibility can coexist with administrative exclusion, abstention, or invalid ballots. Improvements in ballot usability can change whose preferences enter the count. [Research Program in Political Economy](https://rppe.princeton.edu/publications/voting-technology-political-responsiveness-and-infant-health-evidence-brazil-0) |
| Government formation is not a largest-party appointment rule. | Permit majority, minority, and surplus arrangements where institutionally feasible; portfolio shares should often approximate coalition contributions without being mechanically identical. [Sage Journals](https://journals.sagepub.com/doi/10.1177/1354068819850447) |

## A deterministic electoral-system test

Consider a **synthetic**, not historical, electorate with 100 identical districts. In every district:

* A receives **42%**.
* B receives **33%**.
* C receives **25%**.

Under single-member plurality, the seat result is **100–0–0**.

Under a single nationwide, 100-seat proportional allocation without an excluding threshold, the result is **42–33–25**.

The effective number of vote-winning parties is approximately **2.88** in both cases. The effective number of seat-winning parties is **1.00** in the plurality case and approximately **2.88** in the proportional case.

In the proportional chamber, B and C can form a **58-seat majority excluding the largest party**. All three possible two-party coalitions are minimal-winning; only B+C has the smallest seat total.

This is an excellent unit test because no psychological assumptions are needed. A separate repeated-election test should then allow candidates, alliances, and voters to adapt.

## Measure more than party count

Record vote and seat effective-party numbers, district-level competition, geographic concentration, party entry and survival, electoral volatility, and government duration.

For disproportionality, a useful diagnostic is the Gallagher index:

\[
G=\sqrt{\frac12\sum\_j(s\_j-v\_j)^2},
\]

with vote and seat shares both expressed in percentage points.

For participation, report separate ratios using **eligible people**, **registered people**, and **voting-age population** as denominators. Also report invalid ballots. An election with high turnout among a narrow elite is not equivalent to broad political inclusion.

---

# 5. Recommended TCE representation and implementation

## 5.1 Use reusable agents and institutions, not an isolated election subsystem

A compact component design would be:

| Component | Essential state |
| --- | --- |
| `PersonPolitics` | Issue preferences and salience, overlapping memberships, perceived alternatives, loyalties, participation disposition, and dated political experiences. |
| `Faction` | Organizers, core and peripheral supporters, agenda, resources, relationships, and internal disagreements. |
| `Party` | Constituent factions, nomination procedures, candidates, platform commitments, reputation, finances, and discipline. |
| `ElectoralInstitution` | Franchise predicates, voting units, districts, ballot format, allocation rule, schedule, secrecy, and administration. |
| `Office` | Powers, selection method, term, constraints, and removal procedure. |
| `CoalitionAgreement` | Members, external supporters, policy commitments, office allocations, promises, and conditions for withdrawal. |

Reuse existing household, workplace, kinship, religious, neighborhood, and patronage relationships. Do not construct an independent all-to-all political network.

Issue dimensions should be projections of authored law and policy primitives. “Who controls irrigation?” and “Who may hold office?” can matter before a generalized economic left–right dimension exists. An issue can lose salience without deleting the organizations originally formed around it.

## 5.2 Separate slow politics from consequential events

A suitable update cycle is:

**Daily-life events → belief and grievance updates → organizing and recruitment → nominations and campaigning → eligibility and turnout → exact counting → office and coalition formation → policy implementation and accountability.**

Run background belief diffusion and organizational bookkeeping monthly. Trigger immediate updates for consequential events such as arrests, betrayals, leadership deaths, major policy changes, or election announcements.

Campaigning and voting should generate visible activities only where the institution actually requires them: meetings, travel, council attendance, public declarations, queues, or ballot casting. A court faction should not behave like a modern party canvassing household voters.

For performance, a complete evaluation of **50,000 agents × 8 alternatives × 6 issue dimensions** requires **2.4 million dimension comparisons**. Sampling 16 political contacts per agent produces **800,000 directed contact evaluations** per update. These are workload counts, not measured Rust performance claims.

Use deterministic random streams keyed to agent, election, and subsystem. Avoid redrawing an agent’s entire political personality every day or allowing save/load operations to change election outcomes through random-number ordering.

## 5.3 Keep coalition search bounded

For up to 12 parties, enumerating all subsets involves at most \(2^{12}=4,096\) combinations before institutional and compatibility filtering. For larger party systems, use a bounded search over plausible partners rather than exhaustive negotiation trees.

Evaluate policy feasibility and actual confidence support separately from cabinet membership. A party can support a government without receiving ministries.

Do not assign all portfolios equal importance unless deliberately simplifying. Control of taxation, armed force, courts, or food distribution can have very different value in a particular TCE polity.

## 5.4 Existing computational models worth adapting

| Model | Useful mechanism | What TCE must add or change |
| --- | --- | --- |
| **Laver (2005), “Policy and the Dynamics of Political Competition”** | Parties adapt through behavioral heuristics rather than global optimization. | Endogenous interests, social networks, factional organization, electoral administration, and non-electoral politics. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/policy-and-the-dynamics-of-political-competition/89669AF3A2D7CA5196C72272B4AEE3BC) |
| **Laver and Schilperoord (2007), endogenous-party spatial model** | Party entry and exit; dissatisfied citizens can become party leaders. | Real organization costs, multiple institutions, memberships, geographically distinct contests, and evolving preferences. [RePub](https://repub.eur.nl/pub/60755) |
| **Mustillo (2026), “Party Competition with Costly Voting”** | Integrates participation costs into spatial party competition rather than assuming full turnout. | Its fixed citizen positions and restricted party setup should not become TCE constraints. Its simulation results are model outputs, not empirical validation. [JASSS](https://www.jasss.org/29/3/4.html) |
| **CoMSES “Political Competition” replication** | Downloadable replication material for Laver-style competition, useful for comparing implementations. | Treat it as a reference implementation and testbed, not a complete civilization political system. [CoMSES Net](https://www.comses.net/codebases/d28ce2f9-1d29-4f06-a59b-ad37c7daf128/releases/1.1.0/) |

These models provide inspectable starting mechanisms. None covers the full chain from pre-electoral factions through franchise bargaining, individualized participation, and governing coalitions.

## 5.5 What to simplify—and what not to simplify

Simplify the number of active issues, the detail of campaign messages, the frequency of background political updates, and parties’ search for improved platforms. Bounded heuristics are preferable to assuming every actor solves a complete equilibrium model.

Do **not** simplify away district geography, franchise restrictions, nomination control, the difference between public and secret behavior, overlapping identities, or responsibility for actual governing powers. Those features are central to the phenomena TCE is intended to generate.

For calibration, use matched counterfactuals with the same population and initial preferences under alternative institutions. First isolate mechanical effects; then allow behavioral adaptation. A proposed initial ensemble of **30–100 random seeds per scenario** is reasonable for identifying gross instability, but precision requirements should determine the final sample size.

Fit several outcomes jointly. A model can match the national party count while producing implausible individual switching, no geographic structure, unrealistic turnout, and impossible coalitions.

---

# 6. Sources, datasets, and evidence limitations

The scholarly foundations cited above supply mechanisms and competing explanations. The following datasets are especially useful for calibration and validation.

| Source | Best use in TCE | Important limitation |
| --- | --- | --- |
| **International IDEA Voter Turnout Database** | National participation histories and denominator comparisons. | Registered-voter turnout and voting-age-population turnout measure different things. [International IDEA](https://www.idea.int/data-tools/data/voter-turnout-database) |
| **International IDEA Electoral System Design Database** | Electoral-system families, institutional configurations, and rule comparisons. | Implement country-specific details rather than relying only on a broad system label. [International IDEA](https://www.idea.int/data-tools/data/electoral-system-design) |
| **The Elections Archive, including CLEA resources** | Constituency-level returns, territorial competition, and geographic concentration; the archive also provides district-related resources. | Harmonize party identities, boundaries, and election units across time. [Elections Archive](https://electiondataarchive.org/) |
| **Comparative Study of Electoral Systems—CSES** | Individual-level vote choice, attitudes, participation, and institutional context. | Module coverage varies; inspect questionnaires and variable availability before pooling studies. [CSES](https://cses.org/data-download/) |
| **V-Dem and V-Party** | Franchise, institutional conditions, and party-level organizational characteristics. | Treat expert-coded measures and their uncertainty differently from directly counted ballots. [V-Dem](https://www.v-dem.net/data/v-party-dataset/) |
| **ParlGov** | Parties, elections, cabinets, coalition composition, and government sequences. | Its stated coverage is EU/OECD democracies, **1900–2023**, not a globally representative historical sample. [ParlGov](https://www.parlgov.org/) |
| **Afrobarometer** | Individual political attitudes and experiences beyond predominantly European electoral datasets. Round 9 covers **39 countries in 2021–2023**. | Survey responses and electoral returns are different observations; preserve country and survey context. [DataFirst](https://www.datafirst.uct.ac.za/dataportal/index.php/catalog/989) |
| **Ahmed–Stasavage replication data** | Cross-cultural institutional comparisons relevant to councils and early joint governance. | These observations cannot supply prehistoric individual voting rates or faction lifetimes. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/origins-of-early-democracy/09CE2053657A9CF004DB2078B563EB65) |

## Where the evidence is strong, contested, or thin

**Strongest foundations:** exact counting procedures; documented franchise changes; official election aggregates; and well-identified interventions within their original populations.

**Conditional rather than universal:** Duverger-style coordination, seat-product relationships, economic voting, portfolio proportionality, and particular mobilization effects. Each depends on institutions, information, organizational behavior, and the population being studied.

**Contested explanations:** why elites broaden suffrage; how much party alignment follows material interests rather than identity; how reliably political machines monitor behavior; and whether an apparent institutional effect is causal or reflects the circumstances in which that institution was selected. Competing mechanisms should remain available rather than being resolved by a single hard-coded coefficient.

**Thinnest evidence:** prehistoric faction structure, individual preference distributions in early agrarian societies, numerical rates of informal faction formation and dissolution, and cross-era estimates of political memory. Ethnography, institutional history, and formal theory constrain possibilities, but they do not justify precise universal values.

**Implementation priority:** build the exact institutional counting and office-selection layer first, then connect it to persistent factions, imperfect political information, and the lived economic interests of individual agents. That combination can generate distinctive political histories without scripting particular parties, electoral outcomes, or a predetermined path toward democracy.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab928c9-9570-83ea-975d-2403adb9a0fb)
