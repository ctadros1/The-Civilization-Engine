# Leagues, confederations and unions: a simulation-ready report for TCE

## Executive conclusion

**Model a union as a continuing bargain over particular powers, revenues, and obligations—not as a diplomatic relationship that automatically becomes a single state.** Historical unions combined shared action with retained institutions in markedly different ways. Commercial cooperation could remain loosely organized for centuries; military cooperation could become imperial domination; a short-lived constitution could be replaced by a more effective union rather than end in separation. The Hanse, Delian League, and American constitutional transition illustrate these different trajectories. [Hansischer Geschichtsverein](https://www.hansischergeschichtsverein.de/file/wubs_hanse_hq.pdf)

For TCE, distinguish five arrangements:

| Arrangement | Recommended defining feature |
| --- | --- |
| **League** | Members coordinate specified activities, usually through their own governments. |
| **Confederation** | A continuing common organization exists, but constituent governments retain substantial control over implementation and resources. |
| **Personal union** | One person occupies multiple sovereign offices; the underlying institutions remain separate unless explicitly changed. |
| **Federation** | Both common and constituent governments possess constitutionally protected authority, potentially acting directly on individuals. |
| **Incorporating union or merger** | Formerly separate governments transfer their sovereign authority to a successor, while some local laws or institutions may survive. |

These are useful modeling distinctions, not universally agreed historical classifications. In particular, the Hanse was not simply a federation of sovereign cities, and the boundary between a confederation and federation is disputed in some cases. [Hansischer Geschichtsverein](https://www.hansischergeschichtsverein.de/file/wubs_hanse_hq.pdf)

**Track durability along separate dimensions:** organizational survival, continued member consent, retained autonomy, constitutional continuity, and delivery of promised services. A union held together by force is durable in one sense but no longer voluntarily maintained. Equally, replacing a confederal constitution with a federal one need not count as failure.

“Voluntary” also needs an explicit constituency: agreement among monarchs, provincial estates, town councils, or clan authorities is not necessarily consent among everyone they govern. The Swiss charter of 1291, for example, protected established authority and obligations to lawful overlords rather than announcing universal political equality. [Federal Council of Switzerland](https://www.admin.ch/en/federal-charter-1291)

---

## 1. Mechanisms: rules the simulation can implement

### 1.1 Common dangers and common benefits create a bargaining opportunity

The economic argument for union is a trade-off: larger political units can share the costs of defense and public services, but common policies may fit members’ preferences poorly. Alesina and Spolaore formalize this tension between economies of scale and heterogeneous preferences; their model also shows why access to external trade can make small independent states more viable. [OUP Academic](https://academic.oup.com/qje/article-abstract/112/4/1027/1911699)

**TCE rule:** generate proposals when cooperation offers benefits that an existing arrangement cannot provide cheaply. Relevant triggers include an approaching enemy, expensive defensive infrastructure, disputed trade access, recurrent inter-polity violence, or a succession opportunity.

Evaluate the proposal against the **best available alternative**, not against total isolation. A polity might prefer a defensive treaty, commercial agreement, or externally guaranteed neutrality to federation.

For an individual decision-maker \(a\), a useful formulation is:

\[
\Delta V\_a(C)=
\mathbb{E}\_a\!\left[
\sum\_{t=0}^{H\_a}\delta\_a^t
\left(u\_a(x\_t\mid C)-u\_a(x\_t\mid O\_a)\right)
\right]-K\_a(C)
\]

Here \(C\) is the proposed constitution, \(O\_a\) the perceived outside option, \(H\_a\) the planning horizon, and \(K\_a\) the transition cost. Utility should reflect security, livelihood, office, status, religious or legal protections, and political autonomy—not merely aggregate output.

This is a **proposed implementation**, not an empirically estimated historical equation.

### 1.2 Formation requires two bargains: between polities and within them

An agreement acceptable to two negotiators may be unacceptable to their respective governing coalitions. The 1603 union of the English and Scottish crowns did not enable James to merge their parliaments by personal decision; attempts at fuller union encountered institutional resistance. [Parliament UK News](https://www.parliament.uk/about/living-heritage/evolutionofparliament/legislativescrutiny/act-of-union-1707/overview/union-of-the-crowns/)

**TCE rule:** separate negotiation, signature, ratification, and implementation. Each negotiator receives a mandate from the relevant institutions. A proposal can be signed, rejected at home, amended, or implemented only partially.

Support should vary by position. Merchants may value access to a larger market while toll collectors lose revenue. A ruler may accept cooperation that preserves the dynasty but reject an elected common executive. Local judges may resist losing jurisdiction even when residents expect commercial benefits.

Do not require popular voting in every society. Use the polity’s actual decision procedure, while separately tracking excluded groups’ grievances and ability to obstruct compliance.

### 1.3 Bargains succeed by separating what must be shared from what can remain local

The 1707 Anglo-Scottish agreement combined a common parliament and commercial arrangements with continued Scottish legal institutions. Tanzania’s constitutional arrangement combines union authority with a separate Zanzibar government rather than establishing identical governments for both constituent territories. [Parliament UK News](https://www.parliament.uk/documents/heritage/articlesofunion.pdf)

**TCE rule:** negotiate powers by domain. For each domain, specify:

> Who decides? Who pays? Who implements? Who interprets disputes? Who can change the arrangement?

Permit asymmetric provisions. One member might retain a religious court, another a local succession system, while both accept common external defense.

Constitutional concessions, transitional exemptions, debt settlements, and office allocations can make a package acceptable without making every clause equally attractive to every participant.

### 1.4 Revenue collection determines whether common decisions become real

The Dutch experience provides an especially useful distinction between text and practice: planned uniform common excises were not implemented; provincial contribution quotas became the working arrangement. [UC San Diego Pages](https://pages.ucsd.edu/~bslantchev/courses/ps143a/readings/Boogman%20-%20The%20Union%20of%20Utrecht.pdf)

**TCE rule:** a common budget creates claims, not resources.

For a quota-funded organization:

\[
d\_i=B\_U\frac{w\_i}{\sum\_jw\_j}
\]

where \(B\_U\) is approved common expenditure and \(w\_i\) is the constitutionally agreed contribution weight. Actual payment must then pass through local authorization, collection, transport, and accounting.

Track separately:

* Assessed contributions.
* Deliveries actually received.
* Arrears caused by inability.
* Arrears caused by deliberate withholding.

A failed harvest and a calculated refusal to pay should not produce identical diplomatic responses. Grain, labor, ships, and troops must be transferred from existing stocks or activities; they cannot appear because a treaty promises them.

### 1.5 Military specialization can alter the constitutional balance

In the Delian system, obligations to provide ships or money coexisted with Athenian command. The suppression of Naxos demonstrates that an alliance of initially autonomous participants could become a relationship in which departure was forcibly prevented. The timing and interpretation of this transformation remain debated. [UC Press E-Books Collection](https://publishing.cdlib.org/ucpressebooks/public/book/athens-from-cleisthenes-to-pericles.html)

**TCE rule:** record ownership, command, recruitment, maintenance, and loyalty of common forces separately.

Paying a stronger partner to provide defense may be efficient immediately while reducing the payer’s future bargaining power. A common force can become genuinely federal, remain a collection of member contingents, or become the dominant member’s army in practice.

Do not equate all centralization with coercion. The important question is whether transfers of authority remain authorized and contestable under the agreed rules.

### 1.6 Institutions must restrain both member opportunism and central encroachment

Bednar’s account of robust federalism emphasizes interacting safeguards rather than a single constitutional guarantee. A federation must cope with constituent governments shirking or exploiting one another and with the common government exceeding its authority. [Cambridge University Press](https://www.cambridge.org/core/books/robust-federation/3D43B1FEA5C02B1FB07FB9442CAB0A83)

**TCE rule:** give constitutional enforcement multiple possible foundations: arbitration, judicial review, reciprocal sanctions, common assemblies, accounting inspections, leadership removal, and organized opposition.

Each safeguard requires an actor with information, incentives, and practical capacity. A court without accepted authority or an implementer cannot enforce an award merely because its constitution says it can.

Repeated violations should change expectations about future compliance. Conversely, successful dispute resolution should strengthen confidence in particular institutions—not create an undifferentiated friendship bonus.

### 1.7 Two-member unions face a special deadlock–domination problem

Comparative research on dyadic federations identifies representation, inclusion, and distribution as important, but the cases are too heterogeneous to establish that two-member unions must fail. [Dial.pr](https://research.dial.uclouvain.be/bitstreams/2207db05-247f-4090-8bd6-f43abdd392c6/download)

For two TCE polities, equal member voting can make every disagreement a veto. Population-weighted voting can give the larger member permanent control.

A workable authored package could therefore combine:

**Joint authorization for fundamental changes; delegated administration for routine business; protected local domains; and a specified deadlock procedure.**

For example, both members approve a five-year defensive mandate and its contribution formula. An accountable commander can then operate within that mandate without obtaining two fresh approvals for every supply purchase. Disputes over expanding the mandate return to both governments.

Rotating offices may improve inclusion, but rotation alone does not solve unequal resources or conflicting interests.

### 1.8 Unions survive through adaptation, not just loyalty

Senegambia illustrates the danger of divergent expectations: Senegal favored closer integration, while Gambian leaders guarded autonomy; changing perceptions of security reduced the attractiveness of the initial bargain. [AfricaBib](https://www.africabib.org/rec.php?RID=098253948)

**TCE rule:** reevaluate arrangements after meaningful shocks: a threat disappears, trade routes change, a ruler dies, contributions become unaffordable, or a member’s relative strength shifts.

Possible responses should include renegotiation, an opt-out, suspended participation, peaceful withdrawal, constitutional replacement, attempted secession, coercive subordination, and general dissolution.

Do not make collapse the automatic consequence of declining support. A union may narrow its mandate and survive.

---

## 2. Historical bargains and quantitative parameters

### 2.1 What was shared, what was retained, and what endured?

The cases below are **comparative examples, not a representative survival sample**.

| Case | Constitutional bargain | Durability and interpretation |
| --- | --- | --- |
| **Delian League, founded 478/7 BCE** | Common campaigning under Athenian leadership; contributions in ships or money; initially autonomous allies and common meetings. | Coercive subordination of members emerged during its history. Formal association therefore cannot be treated as continuous voluntary consent. The original scope of the bargain and chronology of imperialization are contested. [UC Press E-Books Collection](https://publishing.cdlib.org/ucpressebooks/public/book/athens-from-cleisthenes-to-pericles.html) |
| **Hanseatic network, approximately mid-12th to mid-17th centuries** | Merchants and towns coordinated privileges, commercial protection, diplomacy, and dispute management. Membership and participation were variable; local government remained separate. | Roughly **five centuries** of changing cooperation, without a clean foundation or dissolution date. Its endurance was not the survival of a single sovereign federal constitution. [Hansischer Geschichtsverein](https://www.hansischergeschichtsverein.de/file/wubs_hanse_hq.pdf) |
| **Lombard League, founded 1167** | Italian communes cooperated militarily and politically against imperial pressure while defending communal institutions and privileges. | The **1183 Peace of Constance** was a major settlement, not a simple conversion into a unified state. Subsequent activity and renewal make a single uninterrupted “league lifespan” misleading. [OUP Academic](https://academic.oup.com/british-academy-scholarship-online/book/20035) |
| **Swiss alliances and later federation** | The **1291** pact provided mutual aid, arbitration, and protection of established jurisdiction. It did not abolish local authority or obligations to lawful overlords. | A changing confederate order preceded the **1848 federal constitution**. Do not represent the intervening centuries as one unchanged constitution or infer that modern Switzerland was fully created in 1291. [Federal Council of Switzerland](https://www.admin.ch/en/federal-charter-1291) |
| **Haudenosaunee Confederacy** | A Great Council joined nations through a peace-making constitutional tradition, consensus procedures, titled offices, and clan-based nomination and accountability. National and community institutions remained important. | The original **Five Nations** were joined by the Tuscarora around **1722**. Institutions have substantial historical continuity, but the foundation date is disputed; an exact uninterrupted lifespan is not a defensible calibration target. [National Museum of the American Indian](https://americanindian.si.edu/sites/1/files/pdf/education/HaudenosauneeGuide.pdf) |
| **Dutch Republic / Union of Utrecht** | Common defense and external policy alongside provincial privileges, taxation, and strong local institutions. Implementation departed substantially from the treaty text. | **1579–1795: about 216 years** for the Utrecht-based political order. Its eventual replacement involved revolution and French intervention, not simply spontaneous dissolution from excessive decentralization. [UC San Diego Pages](https://pages.ucsd.edu/~bslantchev/courses/ps143a/readings/Boogman%20-%20The%20Union%20of%20Utrecht.pdf) |
| **English–Scottish personal union and 1707 union** | **1603:** a shared monarch, not a shared parliament. **1707:** a separately negotiated incorporating union with common parliamentary authority and important retained Scottish institutions. | Personal union and political merger were distinct transactions. Shared rulership did not automatically eliminate constitutional barriers. [Parliament UK News](https://www.parliament.uk/about/living-heritage/evolutionofparliament/legislativescrutiny/act-of-union-1707/overview/union-of-the-crowns/) |
| **United States: confederation to federation** | Under the Articles, states retained extensive authority and supplied common resources. The successor Constitution established stronger common legislative, fiscal, executive, and judicial institutions. | The Articles operated from **1781 to 1789**, approximately **eight years**. Their replacement should be coded as constitutional transformation, not the disappearance of the union. [National Archives](https://www.archives.gov/milestone-documents/articles-of-confederation) |
| **Senegambia, 1982–1989** | A security-driven arrangement between Senegal and The Gambia, with divergent ambitions concerning further political and economic integration. | Approximately **seven to eight years**, depending on the precise institutional endpoints used. Illustrates changing outside options and autonomy concerns in a two-member arrangement. [AfricaBib](https://www.africabib.org/rec.php?RID=098253948) |
| **Tanzania, founded 1964** | Union government plus a separate Zanzibar government. The union government also handles mainland non-union matters: the arrangement is structurally asymmetric. | The union reached its **58th anniversary in 2022**. It provides a counterexample to claims that two-member unions necessarily dissolve quickly. Longevity does not by itself establish consensus over every constitutional question. [Embassy of Tanzania in Japan](https://www.jp.tzembassy.go.tz/resources/view/the-united-republic-of-tanzania-58th-union-day-anniversary) |
| **United Arab Emirates, 1971 constitutional framework** | Federal institutions coexist with emirate authority. The constitutional voting rule gives a coalition requirement plus special protection to two major emirates. | A useful example of negotiated asymmetry among rulers rather than a population-based democratic federation. Retained emirate powers are explicit in the constitutional text. [Constitute Project](https://www.constituteproject.org/constitution/United_Arab_Emirates_2004) |

### 2.2 Evidence-based numerical anchors

**Confidence legend:** High means well-supported documentary value or institutional rule; Medium means a historical estimate, reported assessment, or scope-dependent count. Neither implies that a value is transferable to other societies.

| Parameter | Observed value and units | Source and confidence | Modeling interpretation |
| --- | --- | --- | --- |
| Founding participants in the Swiss 1291 pact | **3 communities** | Federal Charter; **High**. [Federal Council of Switzerland](https://www.admin.ch/en/federal-charter-1291) | A very small pact can establish mutual enforcement without creating a comprehensive central state. |
| Haudenosaunee titled council positions | **50 offices** | Onondaga Nation; **High** for the institutional account. [Onondaga Nation](https://www.onondaganation.org/government/) | Model offices and deliberative procedure, not simply 50 interchangeable votes. |
| Distribution of those offices | Mohawk **9**, Oneida **9**, Onondaga **14**, Cayuga **10**, Seneca **8** | Onondaga Nation; **High**. [Onondaga Nation](https://www.onondaganation.org/government/) | Unequal office counts can coexist with consensus and national equality of voice. |
| Hanseatic participation scale | About **70 larger towns**, plus **100–130 smaller towns** | Wubs-Mrozewicz; **Medium**, dependent on definition and period. [Hansischer Geschichtsverein](https://www.hansischergeschichtsverein.de/file/wubs_hanse_hq.pdf) | This is not a fixed simultaneous membership roll; store varying participation by activity. |
| Initial Delian tribute assessment | **460 talents**, reported initial assessment | Thucydides as examined by Fornara and Samons; **Medium** as fiscal evidence. [UC Press E-Books Collection](https://publishing.cdlib.org/ucpressebooks/public/book/athens-from-cleisthenes-to-pericles.html) | An assessed obligation is not verified revenue received. Do not convert directly into a universal output share. |
| Holland’s contribution quota | **More than 58%** of common expenditure from **1616** | Boogman; **High** for the reported quota. [UC San Diego Pages](https://pages.ucsd.edu/~bslantchev/courses/ps143a/readings/Boogman%20-%20The%20Union%20of%20Utrecht.pdf) | Denominator is common expenditure—not Dutch GDP or all public revenue. |
| Utrecht decision thresholds | **Unanimity** for war, truce, peace, and common financial burdens; majority for other decisions | Treaty provisions discussed by Boogman; **High** for formal rules. [UC San Diego Pages](https://pages.ucsd.edu/~bslantchev/courses/ps143a/readings/Boogman%20-%20The%20Union%20of%20Utrecht.pdf) | Thresholds belong to policy domains; actual practice may depart from the written rule. |
| U.S. confederal voting | **1 vote per state**; **9 of 13** for specified major decisions; **13 of 13** for amendments | Articles of Confederation; **High**. [National Archives](https://www.archives.gov/milestone-documents/articles-of-confederation) | Separate representation, ordinary decisions, exceptional decisions, and constitutional amendment. |
| U.S. federal amendment thresholds | Normally **⅔ of both houses** to propose; **¾ of states** to ratify, with alternative convention procedures | Constitution, Article V; **High**. [National Archives](https://www.archives.gov/founding-docs/constitution-transcript) | Entrenchment can differ from ordinary legislation and from founding ratification. |
| Anglo-Scottish initial “Equivalent” | **£398,085 10 shillings**, specified one-time sum | Articles of Union, Article XV; **High** as an agreed amount. [Parliament UK News](https://www.parliament.uk/documents/heritage/articlesofunion.pdf) | Model debt-related compensation and its recipients; not a universal merger price. |
| UAE substantive Supreme Council decisions | At least **5 of 7 votes**, including **Abu Dhabi and Dubai** | Constitution, Article 49, 2004 version; **High** for the rule. [Constitute Project](https://www.constituteproject.org/constitution/United_Arab_Emirates_2004) | Named-member assent can coexist with a numerical majority threshold. |

### 2.3 Numerical exploration where historical estimates are unavailable

The following are **engineering test settings**, not historically measured defaults.

| Variable | Suggested sensitivity sweep | Units / provenance |
| --- | --- | --- |
| Number of constituent polities | **2, 3, 5, 8, 16** | Polities; analyst-selected test cases. |
| Common contribution burden | **0%, 5%, 15%, 30%** of a member’s ordinary public revenue | Revenue share; unvalidated exploration, **not GDP share**. |
| Common control of deployable military resources | **0%, 25%, 50%, 75%, 100%** | Resource share; test ownership and command separately. |
| Decision rules | Simple majority, **⅔**, **¾**, unanimity, named-member assent | Authored institutional alternatives; inspired by documented rules, not estimated frequencies. |
| Maximum member size share | **50%, 65%, 80%, 95%** | Share of population or revenue; test each denominator independently. |
| Scheduled political review | Every **1, 3, or 12 simulated months**, plus event-triggered reviews | Simulation cadence; performance and behavioral sensitivity setting. |

There is **no defensible universal annual breakup probability** supplied by these examples. Avoid converting their average lifespan into one.

---

## 3. Variation across eras and regions

### Foraging societies: networks before territorial constitutions

Hill and colleagues studied **32 contemporary foraging societies**, covering **5,067 individuals**, with a mean experienced residential group size of **28.2 adults**. Their findings emphasize flexible residence and substantial interaction among unrelated adults. These are observations about social organization, not a dataset of sovereign confederations. [Arizona State University](https://asu.elsevierpure.com/en/publications/co-residence-patterns-in-hunter-gatherer-societies-show-unique-hu/)

**TCE implication:** begin with overlapping visiting, marriage, exchange, ritual, and dispute-settlement networks. A group can cooperate without permanently merging its population or territory. Do not treat contemporary foragers as an unchanged record of all prehistoric societies.

### Early farming: persistent councils are plausible, but rates are poorly evidenced

For TCE’s early agrarian start, storage, recurring land claims, settlement permanence, and repeated shared works provide plausible reasons to establish durable inter-community offices.

That is a **mechanistic modeling hypothesis**, not a measured rule that agriculture causes federation. An archaeological pattern of shared pottery or infrastructure should not automatically be interpreted as voluntary political union. For this stage, use broad exploratory settings and make the institutional history emerge from observed disputes, contributions, and decisions.

### Pre-industrial societies: several non-European institutional paths

The Haudenosaunee example makes clan relations, nomination authority, oral constitutional practice, and consensus central rather than peripheral. Its council procedure cannot be reproduced accurately by placing an ordinary majority-vote parliament beneath a different visual style. [Onondaga Nation](https://www.onondaganation.org/government/)

In South Asia, the *Mahāparinibbāna Sutta* associates Vajjian strength with frequent assemblies, concord, respect for established practices and elders, and other conditions. This is valuable evidence for a normative understanding of collective government, **not an econometric demonstration of seven causes of political survival or a complete federal constitution**. [Ancient Buddhist Texts](https://ancient-buddhist-texts.net/Texts-and-Translations/Mahaparinibbanasuttam/02-Prevent-Vajji-Decline.htm)

European commercial and territorial arrangements also differed: the Hanse’s overlapping participation is a poor template for a confederation expected to finance a common army continuously. [Hansischer Geschichtsverein](https://www.hansischergeschichtsverein.de/file/wubs_hanse_hq.pdf)

### Industrial-period federations: broader institutions, but not a new universal sequence

The United States and Switzerland illustrate constitutional systems in which common authority and constituent institutions were explicitly combined. Australia’s federation brought together **six self-governing colonies in 1901**, but those colonies existed within the British Empire; this was not a union of six fully independent international states. [National Archives](https://www.archives.gov/founding-docs/constitution-transcript)

**TCE implication:** administrative reach, communication, recordkeeping, and tax collection can make wider common responsibilities feasible. They should not unlock a mandatory “federal era.”

### Modern arrangements: persistent asymmetry

Tanzania and the UAE show why a modern union need not have identical constituent governments, equal practical power, or a democratic founding bargain. Constitutional provisions may preserve different governmental structures or special assent rights. [Constitute Project](https://www.constituteproject.org/constitution/Tanzania_2005)

Across all periods, represent language, religion, kinship, and identity through their institutional consequences—communication costs, protected practices, constituencies, and trusted relationships—not as an automatic diversity penalty.

---

## 4. Stylized facts and validation targets

These are patterns a plausible simulation should be able to generate, not historical outcomes every run must reproduce.

| Pattern | Historical anchor | Validation implication |
| --- | --- | --- |
| **A narrow organization can endure without becoming a state.** | The Hanse operated across roughly **five centuries** with changing participation. [Hansischer Geschichtsverein](https://www.hansischergeschichtsverein.de/file/wubs_hanse_hq.pdf) | Long survival must not automatically accumulate “integration points” until annexation occurs. |
| **Unequal contributions need not immediately destroy a union.** | Holland carried **over 58%** of common expenditure quotas. [UC San Diego Pages](https://pages.ucsd.edu/~bslantchev/courses/ps143a/readings/Boogman%20-%20The%20Union%20of%20Utrecht.pdf) | Fiscal asymmetry should interact with benefits, influence, and outside options—not trigger a fixed breakup threshold. |
| **Institutional replacement can be successful integration.** | The U.S. Articles operated for about **eight years** before constitutional replacement. [National Archives](https://www.archives.gov/milestone-documents/articles-of-confederation) | Distinguish dissolution from a successor constitution governing substantially the same association. |
| **Organizational survival can conceal lost consent.** | Delian allies could be forcibly subordinated. [UC Press E-Books Collection](https://publishing.cdlib.org/ucpressebooks/public/book/athens-from-cleisthenes-to-pericles.html) | Measure whether members still possess meaningful autonomy and whether attempted departure is suppressed. |
| **Two-member arrangements can have very different outcomes.** | Senegambia lasted under a decade; Tanzania reached **58 years** in 2022. [AfricaBib](https://www.africabib.org/rec.php?RID=098253948) | Member count alone must not determine survival. |
| **Office counts do not uniquely determine voting power.** | Haudenosaunee governance combines **50 titled offices** with structured consensus. [Onondaga Nation](https://www.onondaganation.org/government/) | The deliberative algorithm matters, not just seat arithmetic. |
| **Promises and implementation can diverge for long periods.** | Dutch common excises remained unimplemented while provincial quotas functioned. [UC San Diego Pages](https://pages.ucsd.edu/~bslantchev/courses/ps143a/readings/Boogman%20-%20The%20Union%20of%20Utrecht.pdf) | A constitution needs both a written rule and a record of actual practice. |

Additional **counterfactual tests** should alter one condition at a time: remove the common enemy; double one member’s tax base; interrupt courier access; replace a cooperative ruler; expose a concealed debt; or transfer the common army’s recruitment to one member.

Observe whether the result is renegotiation, paralysis, centralization, exit, or coercion. A model that produces only “stable” and “collapsed” misses much of the relevant behavior.

---

## 5. Recommended representation for TCE

### 5.1 Keep people, polities, offices, and unions as distinct entities

A union should not initially delete its constituent polity IDs. Represent it as an institutional relationship capable of possessing officers, assets, obligations, and domain-specific authority.

| Entity or record | Essential contents |
| --- | --- |
| **Union** | Members, constitution versions, common institutions, assets, treasury, diplomatic identity where applicable. |
| **Membership** | Entry agreement, representation, obligations, exemptions, arrears, exit provisions, actual participation. |
| **Competence clause** | Policy domain, scope, decision-maker, financing rule, implementer, enforcement and appeal. |
| **Office** | Selection procedure, term or succession, powers, accountability, current individual holder. |
| **Constitutional event** | Proposal, mandate, ratification, amendment, violation, adjudication, withdrawal, replacement. |
| **Domestic constituency** | Affected individuals and organizations, political influence, perceived benefits and losses, information. |

Permit overlapping memberships where mandates are compatible. A polity could participate in a commercial league and a defensive confederation. Conflicting obligations should create an actual legal or political problem, not be silently discarded.

### 5.2 Use a competence matrix, not one centralization value

For an initial implementation, eight domains are sufficient:

| Domain | Examples of separable provisions |
| --- | --- |
| Defense | Mutual assistance, common command, recruitment, fortification ownership. |
| External relations | Independent treaties, mandatory consultation, exclusive common diplomacy. |
| Revenue and debt | Contributions, direct taxation, borrowing, inherited debts, guarantees. |
| Trade and movement | Internal tolls, market access, migration, commercial privileges. |
| Money and standards | Accepted coinage, minting, weights and measures. |
| Justice | Inter-member arbitration, appellate jurisdiction, enforcement across borders. |
| Infrastructure | Roads, ports, canals, maintenance responsibilities. |
| Constitutional rights | Retained powers, representation, amendment, admission, withdrawal. |

Religion, education, and other domains can be added when those systems become politically consequential. A centralized defense system need not imply common private law or a common currency.

### 5.3 Make personal union an office-holding relationship

Implement a personal union as:

> One individual holds two or more sovereign offices whose jurisdictions and succession rules remain distinct.

On death, deposition, abdication, or succession reform, resolve each office independently. The same successor may inherit all of them, only some, or none.

Recognition by the relevant institutions remains important. A dynastic claim does not automatically grant practical control.

A subsequent merger requires a separate constitutional transaction—precisely the distinction illustrated by the English–Scottish case. [Parliament UK News](https://www.parliament.uk/about/living-heritage/evolutionofparliament/legislativescrutiny/act-of-union-1707/overview/union-of-the-crowns/)

### 5.4 Preserve individual politics without requiring universal constant deliberation

For individual agents, union policy should matter through concrete consequences: taxes, levies, employment, market access, litigation, loss of office, religious guarantees, or exposure to violence.

Do not calculate every resident’s response to every clause every day. Instead:

1. Calculate policy effects when a relevant proposal or outcome occurs.
2. Cache household, occupational, settlement, and organizational interests.
3. Let politically active individuals mobilize those interests through existing institutions.

A merchant whose trade improves need not instantly endorse every centralizing amendment. A councilor may personally benefit from federation while constituents lose. Agents should possess incomplete information about distant members and uncertain beliefs about promised gains.

### 5.5 Make enforcement and exit expensive processes, not buttons

Withdrawal needs a settlement of outstanding matters: debts, common assets, soldiers, forts, access rights, pending cases, and obligations to third parties.

Where the constitution forbids withdrawal, an exit attempt should invoke the existing political and conflict systems. It may fail, provoke negotiation, become a civil war, or succeed despite the formal rule.

Similarly, an incorporating merger should transfer authority and liabilities explicitly. It should not erase settlement identities, interpersonal relationships, local legal expectations, or opposition.

### 5.6 Maintain separate outcome indicators

Recommended diagnostics are:

**Mandate performance:** Are promised services delivered?

**Compliance:** Are authorized contributions and decisions implemented?

**Consent:** Which governing institutions and affected populations continue to support membership?

**Autonomy:** How much retained authority can members actually exercise?

**Distribution:** Who pays, who benefits, and who controls valuable offices and assets?

**Enforcement dependence:** How much of continued membership relies on threatened or actual coercion?

A convenient interface may summarize these, but the simulation should not reduce them to one hidden “cohesion” number.

### 5.7 Performance and fast-forward

Use event-driven constitutional politics. Restrict proposal generation to known, reachable, or strategically connected polities; do not enumerate every possible partition of the political map.

Budget settlement can occur seasonally or annually, while war, defaults, ruler deaths, and serious violations trigger immediate review. Couriers and assemblies can use the same travel and scheduling systems as other institutions.

For fast-forward, preserve ledgers, office succession, major shocks, and threshold-crossing disputes. Aggregate routine compliance and deliberation, not the consequences of constitutional change.

This is an architectural recommendation, **not a benchmark demonstrating a particular runtime for 50,000 agents**.

### 5.8 Existing models and games worth borrowing from

| Model or game | Useful idea | Important limitation for TCE |
| --- | --- | --- |
| **Alesina–Spolaore, “On the Number and Size of Nations”** | Endogenous political size from economies of scale, preference differences, and outside options. | A formal economic model, not a complete account of historical constitutional bargaining. [OUP Academic](https://academic.oup.com/qje/article-abstract/112/4/1027/1911699) |
| **Bednar, *The Robust Federation*** | Interacting safeguards against several forms of institutional opportunism. | Needs explicit actors, resources, information, and enforcement when translated into a simulation. [Cambridge University Press](https://www.cambridge.org/core/books/robust-federation/3D43B1FEA5C02B1FB07FB9442CAB0A83) |
| **Stellaris federation rework, 2019 design diary** | Configurable federation laws, fleet contributions, leadership succession, and different organizational purposes. | Cohesion and experience-based unlocks are game abstractions, not validated historical mechanisms. [Steam Store](https://store.steampowered.com/news/posts/?appids=281990&enddate=1573128061) |
| **Europa Universalis IV Native-federation design diary, 2020** | External threats influence federation cohesion and leadership dynamics. | A single cohesion quantity is too coarse to represent consent, fiscal viability, and domination separately. This is a reference to the published design, not a claim about every current mechanic. [Reddit](https://www.reddit.com/r/paradoxplaza/comments/ixmxr4/eu4_development_diary_22nd_of_september_2020/) |

---

## 6. Sources, datasets, and limits of the evidence

### Research foundations

The most useful theoretical starting points are **Alesina and Spolaore’s “On the Number and Size of Nations”** for the scale–preference trade-off; **Jenna Bednar’s *The Robust Federation*** for institutional safeguards; and **Alfred Stepan’s “Federalism and Democracy: Beyond the U.S. Model”** for avoiding the assumption that all federations arise through the same coming-together process. [OUP Academic](https://academic.oup.com/qje/article-abstract/112/4/1027/1911699)

For historical mechanisms, **Fornara and Samons, *Athens from Cleisthenes to Pericles*** explicitly examine disputed interpretations of Athenian imperialization; **Raccagni, *The Lombard League, 1167–1225*** treats the league in its medieval political setting; and **Wubs-Mrozewicz’s introduction to *The Hanse in Medieval and Early Modern Europe*** is particularly valuable for correcting assumptions about fixed membership and statehood. [UC Press E-Books Collection](https://publishing.cdlib.org/ucpressebooks/public/book/athens-from-cleisthenes-to-pericles.html)

For Indigenous constitutional practice, use **Onondaga Nation’s account of its own institutions** alongside the **Smithsonian National Museum of the American Indian’s Haudenosaunee guide**. They are preferable to reconstructing these institutions solely through European constitutional analogies. [Onondaga Nation](https://www.onondaganation.org/government/)

### Datasets suitable for calibration work

| Dataset | Verified coverage/version | Appropriate use | Main limitation |
| --- | --- | --- | --- |
| **Alliance Treaty Obligations and Provisions — ATOP** | Version **5.1**, **1815–2018** | Code commitment types, consultation, military cooperation, and treaty institutionalization. | Military alliances are not the same population as domestic federations. [ATOP](https://atopdata.org/) |
| **Correlates of War Formal Alliances** | Version **4.1**, **1816–2012** | Alliance formation, membership, duration, and commitment categories. | Alliance termination can reflect integration or other changes, not simply cooperative failure. [Correlates of War](https://correlatesofwar.org/data-sets/formal-alliances/) |
| **Comparative Constitutions Project** | Chronology version **6.0**, **1789–2025**; characteristics version **5.0** | Constitutional replacement, amendment, and formal institutional design. | Written constitutional provisions do not establish actual enforcement or popular consent. [Comparative Constitutions Project](https://comparativeconstitutionsproject.org/download-data/) |
| **A purpose-built TCE historical case register** | Recommended research product | Combine treaties, contribution records, institutional changes, exit attempts, and member-level histories. | Requires explicit coding judgments and source-quality flags. |

For that case register, record **constitution episodes and member episodes**, not only union names. Store uncertain foundation dates as intervals. Distinguish peaceful withdrawal, expulsion, constitutional replacement, conquest, and coercive capture. Treat organizations still operating at the end of observation as **right-censored**, not as having a completed lifespan.

### Claims that should remain flagged

**Founding dates and continuity.** Haudenosaunee origins are disputed in chronological scholarship; Swiss national founding narratives should not be confused with an unchanged institutional history beginning in 1291. [Academia](https://www.academia.edu/45522296/A_Sign_in_the_Sky_Dating_the_League_of_the_Haudenosaunee)

**Constitution versus operation.** The Dutch case is a warning against coding every treaty clause as an implemented institution. [UC San Diego Pages](https://pages.ucsd.edu/~bslantchev/courses/ps143a/readings/Boogman%20-%20The%20Union%20of%20Utrecht.pdf)

**Consent versus elite agreement.** A surviving treaty or constitutional text establishes an agreement and its claims, not the preferences of every subject.

**Selection bias.** Famous examples are not a random sample. Long-lived organizations, successful states, and well-documented literate elites are easier to study than abortive negotiations and short-lived local associations.

**Causal uncertainty.** Historical comparisons support mechanisms and plausible interactions much more strongly than universal coefficients. The dyadic-federation literature is useful for generating hypotheses, not for assigning a reliable breakup probability to every two-member union. [Dial.pr](https://research.dial.uclouvain.be/bitstreams/2207db05-247f-4090-8bd6-f43abdd392c6/download)

## Bottom line for TCE

The minimum convincing implementation is **a competence-based constitution, domestic ratification, a real resource ledger, explicit military control, and procedures for renegotiation and exit**.

Build those systems first. They allow the same authored primitives to produce a limited league that lasts for centuries, a federation that survives constitutional replacement, a dynastic union that separates at succession, or a defensive alliance that becomes an empire—without scripting which historical path any world must follow.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92952-06cc-83ea-9865-24b3fbb3b095)
