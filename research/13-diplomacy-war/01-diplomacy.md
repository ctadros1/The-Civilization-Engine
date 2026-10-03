# Diplomacy and inter-polity relations for The Civilization Engine

## Executive recommendation

**Represent diplomacy as a network of people exchanging information and making institutionally authorized, conditional commitments—not as a single “relations” score.**

For TCE, the core diplomatic object should be an **agreement whose obligations become due under specified circumstances**. Individuals negotiate it; institutions authorize it; households, officials, soldiers, and treasuries carry it out. Other actors observe some portion of that performance and revise their expectations.

A workable minimum consists of four connected systems:

| System | Central question |
| --- | --- |
| Contact and communication | Who knows whom, what do they know, and when does information arrive? |
| Bargaining | Which packages are preferable to available alternatives? |
| Authorization | Who can bind which people, resources, offices, or territories? |
| Performance and enforcement | What was promised, was it delivered, and what happens next? |

The strongest quantitative evidence concerns comparatively recent interstate agreements. Earlier sources provide excellent examples of mechanisms and contractual provisions, but much weaker estimates of their population-wide frequencies. **Use historical treaty terms as authored examples, modern datasets as conditional calibration targets, and explicitly labeled design priors where measurements are absent.**

---

## 1. Mechanisms that TCE can implement

### 1.1 Contact is a process, not a map-reveal event

Distinguish **awareness**, **access**, and **recognition**. Knowing that another settlement exists does not imply knowing its ruler, possessing a safe route to it, or recognizing its authority.

The Amarna correspondence illustrates this distinction particularly well. A fourteenth-century BCE Assyrian communication presented itself as the opening of correspondence with Egypt, accompanied by gifts and a request that its messenger be allowed to visit. Other letters operated within established relationships between rulers described as brothers or between superiors and subordinates. Contact therefore conveyed political status as well as information. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-amarna-letters)

**Implementation rule:** traders, migrants, kin, travelers, captives, and officials generate reports of foreign communities. A diplomatic mission becomes possible when a sponsor has a destination, a plausible intermediary or route, and sufficient resources. Receiving the mission is a separate decision.

Keep several potentially inconsistent beliefs:

* “This settlement exists.”
* “This person speaks for it.”
* “This person can deliver the promised action.”
* “We accept this person’s claimed rank.”

The distinction permits de facto dealings without full recognition, and negotiation with a town, lineage, rebel coalition, or tributary whose status remains disputed.

### 1.2 Envoys need mandates, logistics, and discretion

Give each envoy an actual principal, instructions, permitted concessions, credentials, language competence, and resources. An envoy may be authorized to discuss everything but conclude nothing, or to conclude agreements within a narrow range.

Diplomatic communication also changes delegation. Nickles’s multinational study of telegraphy shows that faster communication helped centralize foreign ministries and reduce some forms of diplomatic autonomy, while introducing interception, garbling, expense, and faster-moving crises. Speed did not simply produce better decisions. [JSTOR](https://www.jstor.org/stable/j.ctv1q8tgc2?utm_source=chatgpt.com)

For TCE:

\[
T\_{\text{negotiation}}
=
T\_{\text{travel}}
+T\_{\text{waiting}}
+T\_{\text{discussion}}
+T\_{\text{referral}}
+T\_{\text{authorization}}.
\]

These components should emerge from the world. A messenger returning for instructions incurs another journey; a resident representative avoids that journey but still needs communication with the principal.

**Separate negotiating skill from magical persuasion.** Skill should improve information gathering, package construction, interpretation, and identification of acceptable concessions. It should not routinely make an institution accept a materially disastrous agreement.

### 1.3 Gifts, marriages, and hostages solve different problems

**Gifts.** Diplomatic gifts can open access, demonstrate resources, acknowledge standing, and sustain reciprocal expectations. Amarna letters explicitly discuss gifts alongside royal marriages, showing that these were substantive elements of relationships rather than ornamental additions. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-amarna-letters)

In TCE, gifts should transfer real goods to identifiable recipients. Their effects depend on recipient needs, customary expectations, public visibility, and the relationship being claimed. Repeated small gifts should not indefinitely overcome territorial conflict. A gift to a ruler personally may also have different consequences from a payment to a polity’s treasury.

**Marriages.** Represent a marriage through the people and property arrangements it creates: kinship, residence, dowry or other transfers where applicable, succession eligibility, children, correspondence, and divided loyalties. The diplomatic benefit then follows from those connections. The same marriage can create a useful intermediary today and a disputed succession claim later. This is a modeling recommendation, not a universal historical marriage-effect coefficient.

**Hostages.** Kosto’s historical treatment distinguishes medieval hostageship as a guarantee attached to agreements from indiscriminate kidnapping or ordinary ransom captivity. His analysis also emphasizes that physical control of people could create political relationships extending beyond the immediate guarantee. [OUP Academic](https://academic.oup.com/book/3170)

Make hostages living agents with residence, treatment, release conditions, education, relationships, and future political roles. Distinguish a person transferred immediately from someone pledged only upon default. Do not automatically execute a hostage after a breach: that removes a politically valuable person and should be a separate, consequential decision. The historical evidence reviewed here does not justify a universal execution probability.

### 1.4 Negotiation and ratification are different games

Putnam’s “two-level game” is directly applicable: leaders bargain externally while seeking formal or informal acceptance internally. An internationally attractive bargain can fail because it falls outside the set of agreements acceptable to the relevant domestic participants. Domestic constraints may also strengthen a negotiator’s bargaining position. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/abs/diplomacy-and-domestic-politics-the-logic-of-twolevel-games/B2E11FB757C4465C4097015BD421035F)

For each polity, define its **ratifiable set** through actual institutional rules:

\[
\operatorname{Ratifiable}\_i(z)
=
\operatorname{AuthorizationRule}\_i
\bigl(\text{positions of required decision-makers on package }z\bigr).
\]

A council’s approval, a ruler’s oath, constituent communities’ assent, and a legislature’s consent need not be interchangeable.

Keep separate states for **negotiated**, **approved internally**, **consent communicated**, **effective**, and **implemented**. Modern treaty law itself recognizes several ways to express consent and treats entry into force separately; ratification is not mandatory for every international agreement. That modern legal framework should not be projected unchanged onto earlier societies. [United Nations Legal Affairs](https://legal.un.org/ilc/texts/instruments/english/conventions/1_1_1969.pdf)

Also distinguish **authority to promise** from **capacity to perform**. A ruler may formally promise border security while lacking control over raiding households.

### 1.5 Trust should be contextual and observer-specific

Sartori models how reputational concerns can make diplomatic communication informative even without an immediate material cost to speaking. Crescenzi and colleagues find that histories of honoring alliances are associated with greater likelihood of entering subsequent alliances in their interstate sample. Neither result implies that every observer possesses the same information or that reputation overrides current interests. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/might-of-the-pen-a-reputational-theory-of-communication-in-international-disputes/9357C2D6B88D6913F72D378DF9C6366E)

For TCE, maintain separate expectations about:

| Expectation | Relevant evidence |
| --- | --- |
| Willingness to honor a particular obligation | Performance when that obligation actually became due |
| Capacity to perform | Resources, logistics, command over subjects |
| Truthfulness | Claims later checked against outcomes or other reports |
| Resolve | Costs accepted when an issue was contested |
| Institutional continuity | Whether successors preserve offices, commitments, and enforcement |

A cheap implementation is a set of weighted success/failure records by obligation domain. For an uncertain reliability parameter:

\[
E[r]=\frac{\alpha}{\alpha+\beta}.
\]

Update \(\alpha\) and \(\beta\) only after a relevant opportunity, weighted by observation quality and attribution. This is an engineering approximation, not an empirically established psychological law.

**Ten peaceful years without an attack requiring assistance are not ten successful alliance tests.** Nor should a missed grain shipment during a documented famine count identically to a deliberate diversion of grain to an enemy.

### 1.6 Maintenance requires performance, interpretation, and repair

The Song–Liao agreement associated with Chanyuan/Shanyuan combined annual transfers with border restraints, rules against sheltering fugitives, construction restrictions, and oaths. Its implementation involved financial administration and a specified transfer location, not merely ceremonial friendship. [Reed College](https://www.reed.edu/chinese/chin-hum/materials/shanyuan/shanyuan.html)

Translate maintenance into scheduled actions: payment collection, transport, receipt, inspections, meetings, oath renewals, dispute hearings, and return of pledged people.

For an apparent breach, evaluate three separate questions:

**Did the triggering event occur? Was the obligation violated? Who was responsible?**

Then permit explanation, compensation, replacement performance, suspension, renegotiation, or escalation. A frontier raid should not automatically prove that the neighboring government ordered it.

A useful **design inequality** for self-enforcement is:

\[
\text{gain from defection}
\le
\Pr(\text{detection})\,
E[\text{credible immediate sanction}\mid\text{detection}]
+
E[\text{lost future cooperation}].
\]

The sanction must itself be credible. A guarantor that would lose more than it gains by intervening cannot be treated as an automatic enforcement mechanism.

---

## 2. Fearon’s bargaining model: what to implement

### 2.1 Start with a benchmark that produces peace

Let two actors dispute a divisible benefit normalized to \(1\). Actor A receives share \(x\), B receives \(1-x\). Let \(p\) be A’s probability of winning a terminal war, and \(c\_A,c\_B>0\) their war costs in the same utility units.

With common knowledge, linear utility, and enforceable settlement:

\[
W\_A=p-c\_A,\qquad W\_B=1-p-c\_B.
\]

Both accept peace when:

\[
\boxed{p-c\_A\le x\le p+c\_B}
\]

with the interval intersected with \([0,1]\).

Before clipping, its width is \(c\_A+c\_B\). The fundamental puzzle is therefore not why actors have conflicting interests, but why they fail to secure mutually preferable settlements. Fearon identifies private information combined with incentives to misrepresent, and commitment problems, as central mechanisms. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/abs/rationalist-explanations-for-war/E3B716A4034C11ECF8CE8732BC2F80DD)

**TCE validation rule:** with complete information, positive war costs, freely divisible benefits, aligned decision-makers, and enforceable agreements, the benchmark solver should find peace.

### 2.2 Private information must affect bargaining behavior

Do not implement uncertainty as “add a random chance of war.” Give actors beliefs about their opponent’s strength, willingness to absorb costs, and acceptable terms. They then choose offers while anticipating acceptance or rejection.

A general offer evaluation is:

\[
Q\_i(z)=
P(\text{accept}\mid z,\mathcal I\_i)\,U\_i(z)
+
\bigl[1-P(\text{accept}\mid z,\mathcal I\_i)\bigr]V\_i^{\text{continuation}}.
\]

The continuation value may include another offer, delay, mobilization, retreat, or fighting.

**Illustrative unit test—not historical calibration.** In a one-off ultimatum game, suppose A has \(p=0.6\) and \(c\_A=0.1\). B’s cost is privately known: \(0.30\) with probability \(0.8\), or \(0.05\) with probability \(0.2\).

B’s minimum acceptable share is therefore either \(0.10\) or \(0.35\). Assuming acceptance at indifference:

| A’s offer to B | A’s expected payoff |
| --- | --- |
| Offer \(0.35\), accepted by either type | \(0.65\) |
| Offer \(0.10\), accepted only by the high-cost type; rejection causes war | \(0.8(0.90)+0.2(0.50)=0.82\) |

A prefers the risky offer despite a 20% probability of war. The uncertainty concerns B’s costs, not an arbitrary aggression roll. Allowing costless, unlimited renegotiation changes this game and must change the implementation.

### 2.3 Commitment problems survive accurate information

A settlement can be attractive now but impossible to maintain after a change in power. Disarming, abandoning a fortress, transferring a strategic route, or accepting another actor’s rapid growth may alter the future bargaining position. Powell’s analysis explains how such shifts can make costly conflict rational even without an informational misunderstanding. [Slantchev's Website](https://slantchev.ucsd.edu/courses/pdf/powell-io2006.pdf?utm_source=chatgpt.com)

For TCE, evaluate the **post-agreement state**, not just the immediate exchange. An apparently generous proposal may be rejected because it leaves the recipient vulnerable next year.

Possible contractual responses include staged transfers, reciprocal withdrawals, external guarantees, retained defenses, limited duration, and scheduled renegotiation. None should automatically work: each changes resources, information, or future incentives.

Treat indivisibility as a restriction on feasible packages. A city may be physically indivisible, while its revenues, access rights, custody, or associated compensation can still be negotiated. Conversely, political or religious constraints may genuinely prevent such substitutions.

### 2.4 Fighting and bargaining should run concurrently

Filson and Werner implement repeated proposals alongside fighting, with offer rejection and battlefield outcomes revealing information while losses change available resources. Slantchev similarly models simultaneous bargaining and fighting under asymmetric information. These are much closer to a useful war–diplomacy interface than “reach a war-score threshold, then unlock peace.” [Rochelle Terman](https://rochelleterman.com/ir/sites/default/files/Filsen%202002.pdf)

After a battle, failed mobilization, famine, ally’s intervention, or succession, recompute beliefs and continuation values. Do not guarantee that every additional battle brings peace closer: favorable outcomes can encourage greater demands, while political or commitment problems can persist despite learning.

---

## 3. Treaty clauses, durations, and quantitative parameters

### 3.1 Build clauses from obligations, not treaty labels

A “friendship treaty” can contain several different commitments. Conversely, two agreements both called alliances can require different behavior.

The following are **documented clause families and term examples, not estimates of their prevalence across all societies**.

| Clause family | Historical anchor or duration evidence | TCE representation |
| --- | --- | --- |
| Peace and nonaggression | The Peace of Nicias specified **50 years**, with dispute handling and mutual amendment provisions. [Livius](https://www.livius.org/sources/content/thucydides-historian/peace-of-nicias/) | Prohibited acts, covered actors and territory, effective date, expiration, dispute procedure |
| Conditional military assistance | The **1902 Anglo-Japanese alliance** distinguished neutrality in a qualifying war against one power from assistance when additional powers joined. [World Japan](https://worldjpn.net/documents/texts/pw/19020130.T1E.html) | Explicit trigger, qualifying opponent set, geographic scope, required contribution |
| Consultation rather than intervention | NATO’s founding treaty distinguishes consultation from its collective-defense provisions; assistance is not specified as one identical automatic military response by every member. [NATO](https://www.nato.int/en/about-us/official-texts-and-resources/official-texts/1949/04/04/the-north-atlantic-treaty) | Consultation event, response deadline, admissible forms of assistance |
| Transfers, subsidies, or tribute | Song–Liao terms specified **annual** transfers. [Reed College](https://www.reed.edu/chinese/chin-hum/materials/shanyuan/shanyuan.html) | Obligor, beneficiary, goods, quantity, recurrence, collection point, delivery and receipt |
| Borders and military construction | Song–Liao terms restrained cross-border incursions and new construction while allowing maintenance of existing structures. [Reed College](https://www.reed.edu/chinese/chin-hum/materials/shanyuan/shanyuan.html) | Spatially defined restrictions, permitted repairs, monitoring and attribution |
| Captive return and reconciliation | The **1701 Montreal ratification record** describes specific prisoner transfers, promises of further returns, and ceremonial commitments. [Open History Seminar](https://openhistoryseminar.com/canadianhistory/chapter/document-3-great-peace-of-montreal-1701/) | Identified people, responsible custodians, transfer schedule, disputed cases |
| Hostage or personal guarantee | Historical hostageship could attach to finite transactions or open-ended relationships. [OUP Academic](https://academic.oup.com/book/3170) | Person, custodian, underlying obligation, treatment, replacement and release conditions |
| Resource access, trade, and transit | These should be separately negotiable rights rather than consequences automatically implied by peace; the Nicias text, for example, specifically protected access to shared religious sites. [Livius](https://www.livius.org/sources/content/thucydides-historian/peace-of-nicias/) | Rights over routes, markets, land, water, or facilities; users; fees; seasons; exclusions |
| Renewal and withdrawal | The 1902 Anglo-Japanese agreement had a **5-year initial term**, **12-month notice**, and continuation during an ongoing war. [World Japan](https://worldjpn.net/documents/texts/pw/19020130.T1E.html) | Initial term, renewal rule, notice period, exceptions, pending-obligation handling |

**Store duration at both agreement and clause level.** A permanent boundary settlement can coexist with a temporary subsidy, recurring meetings, and a hostage released after a particular transfer.

Indefinite agreements also need exit rules. NATO’s founding text provides for review after **10 years** and withdrawal with **one year’s notice after 20 years**. These are distinct clocks, not a twenty-year expiration. [NATO](https://www.nato.int/en/about-us/official-texts-and-resources/official-texts/1949/04/04/the-north-atlantic-treaty)

Koremenos’s work cautions against interpreting longer duration as automatically greater commitment. Finite terms and renegotiation can make cooperation more feasible under uncertainty. **Expiration, replacement, and breach should therefore be different outcomes.** [Cambridge University Press](https://www.cambridge.org/core/journals/american-journal-of-international-law/article/duration-seriousness-of-commitment-an-empirical-and-theoretical-critique-of-nyarkos-treaties-vs-executive-agreements/9BC4EC9BCB4166FBD231EBB4B7008A93)

### 3.2 Quantitative observations worth retaining

**Confidence:** H = clearly documented provision or transparent coding; M = credible but context-bound behavioral evidence. Confidence in a historical observation is not confidence that it transfers unchanged to TCE.

| Quantity | Value and unit | Scope and source | Confidence and appropriate use |
| --- | --- | --- | --- |
| Opening diplomatic gift bundle | **1 chariot, 2 horses, 1 carved lapis-lazuli object** | Assyrian opening correspondence with Egypt in the Amarna material. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-amarna-letters) | **H for this example.** Demonstrates inventory-based gifts, not a standard embassy price |
| Annual interstate transfer | **100,000 taels of silver and 200,000 bolts of silk per year** | Song–Liao agreement text. [Reed College](https://www.reed.edu/chinese/chin-hum/materials/shanyuan/shanyuan.html) | **H for stated terms.** Imperial-scale example; do not paste these quantities into a small polity |
| Treaty oath renewal | **Every 1 year** | Nicias agreements; the alliance text linked reciprocal visits to festivals. [Livius](https://www.livius.org/sources/content/thucydides-historian/peace-of-nicias/) | **H for provision.** Supports recurrent diplomatic maintenance |
| Domestic treaty-consent threshold | **Two-thirds of senators present** | U.S. constitutional treaty procedure, as described by the Senate. [U.S. Senate](https://www.senate.gov/about/powers-procedures/treaties.htm) | **H for this institution.** Not a universal ratification threshold |
| Alliance commitments honored | Approximately **75% of relevant wartime commitments** | Leeds’s historical alliance-reliability research, focused on 1816–1944. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/abs/alliance-reliability-in-times-of-war-explaining-state-decisions-to-violate-treaties/EC9C08122EC24E9800F7AFE6000672E4) | **M, conditional sample.** Not a per-year survival rate |
| Alternative alliance-reliability estimates | **50% overall; 66% before 1945; 22% in 1945–2003** | Berkemeier and Fuhrmann’s revised and extended 1816–2003 analysis. [Sage Journals](https://journals.sagepub.com/doi/10.1177/2053168018779697) | **M, coding/sample dependent.** Evidence against one timeless compliance constant |

The disagreement in alliance estimates is important rather than inconvenient. “Honored” depends on what the agreement required, whether an obligation was activated, the observation unit, and the sample. **Do not turn these estimates into a 25% or 50% annual betrayal roll.**

### 3.3 Initial design priors—not historical estimates

The following values are proposed for prototyping and sensitivity analysis. They should remain visibly separate from historical calibration data.

| Parameter | Initial value or range | Units | Rationale and confidence |
| --- | --- | --- | --- |
| Unknown obligation reliability | \(\alpha=\beta=1\); test prior strengths of **1–8 total pseudo-observations** | Beta-distribution shape parameters | Neutral mean with uncertain confidence; **design prior** |
| Relevance half-life of old behavioral evidence | Test **2, 5, and 20** | Years | Contextual relevance may decay without deleting archives; **design prior** |
| War-cost test grid | **0.02–0.50** of disputed benefit’s value | Normalized utility | Solver tests only; production costs come from expected losses and political valuations; **design prior** |
| Routine relationship reassessment | **7–30** | Simulation days | Events interrupt the schedule; this is not message-delivery time; **engineering choice** |
| Candidate packages per bargaining decision | **8–32** | Packages | Bounded search over authored clause combinations; **engineering choice** |
| Opponent-belief scenarios | **16–64** | Samples per candidate | Cheap uncertainty approximation, using cached aggregate forecasts; **engineering choice** |
| Available duration proposals | **1 season; 1, 5, 10, 50 years; event-based; indefinite** | Calendar or event terms | Negotiation menu, not a uniform historical distribution; **content choice** |

No reviewed evidence supports universal constants for envoy mortality, negotiation rounds, gift expenditure as a share of income, hostage effectiveness, or trust decay. For these, preserve mechanisms and run sensitivity tests rather than presenting invented historical averages.

---

## 4. Variation across eras and regions

**Do not make these fixed technology eras.** They are configurations of political authority, communication, record-keeping, resources, and social organization that TCE can produce in different combinations.

| Context | Evidence and distinguishing features | Modeling consequence |
| --- | --- | --- |
| **Foragers: southern African Ju/’hoansi** | Wiessner’s research describes *hxaro* as a regional reciprocal network supporting relationships and access to resources beyond a single residential group. It is not an interstate treaty system. [eHRAF World Cultures](https://ehrafworldcultures.yale.edu/cultures/fx10/documents/020) | Begin with interpersonal and household ties, overlapping memberships, hospitality, and access expectations. Do not invent a sovereign foreign ministry for every camp |
| **Early farming communities** | The evidence reviewed does not establish a general distribution of treaty institutions or durations for prehistoric farming villages. Seasonal access, compensation, kin-mediated negotiation, and joint ceremonies are useful **reconstruction hypotheses**, not measured universal rules | Allow lineage, household, and village representatives with limited authority. Test alternative institutional arrangements rather than assigning one “Neolithic diplomacy” package |
| **Early literate states: Egypt and western Asia** | Amarna correspondence used written messages, diplomatic gifts, royal marriage, and differentiated status relationships among distant rulers. [The Metropolitan Museum of Art](https://www.metmuseum.org/essays/the-amarna-letters) | Add scribes, interpreters, archives, credentials, court access, and disputed rank. Writing preserves commitments but does not eliminate ambiguity |
| **Large agrarian states: Song and Liao** | Their agreement bundled security, transfers, border management, and an enduring oath. Research on the wider regional order also shows that arrangements between two major powers could fail to accommodate third-party ambitions. [Cambridge University Press](https://www.cambridge.org/core/journals/journal-of-chinese-history/article/fragility-of-peace-song-chinas-northwestern-frontier-and-erosion-of-the-chanyuan-paradigm-in-the-mideleventh-century/AF2F19A32A23ED0F304C3DE814851A3F) | Permit parity bargains as well as hierarchy. Bilateral stability must not imply regional stability |
| **Medieval Europe and neighboring regions** | Kosto documents hostageship as a varied guarantee institution operating alongside other political and legal practices, not merely a primitive substitute for them. [OUP Academic](https://academic.oup.com/book/3170) | Personal guarantees can coexist with sophisticated councils, accounts, and treaty texts. Preserve overlapping personal and institutional commitments |
| **West-central Africa: Kongo** | Thornton’s research identifies Kongo as an active participant shaping an alliance with the Netherlands and Dutch strategy in the Atlantic conflict, rather than a passive object of European diplomacy. [JSTOR](https://www.jstor.org/stable/43901848) | Give African and other non-European polities independent strategic objectives, initiative, diplomacy, and coalition choice |
| **Indigenous North America and New France** | The Montreal record describes emissaries sent to previously absent partners, collective assembly, prisoner exchanges, and commitments expressed through wampum and ceremonial speech. Its French framing must not be mistaken for a neutral account of every party’s authority claims. [Open History Seminar](https://openhistoryseminar.com/canadianhistory/chapter/document-3-great-peace-of-montreal-1701/) | Oral and material records can support sophisticated multilateral diplomacy. Track which communities actually assent and whether the parties interpret the agreement alike |
| **Industrializing interstate systems** | Telegraphy changed speed, supervision, secrecy, and crisis handling; the Anglo-Japanese treaty demonstrates precise conditional obligations and exit provisions. [JSTOR](https://www.jstor.org/stable/j.ctv1q8tgc2?utm_source=chatgpt.com) | Separate communication improvements from institutional changes. Faster instructions can reduce envoy discretion while shortening decision windows |
| **Modern institutional diplomacy** | Formal rules distinguish representatives’ authority, consent, entry into force, and termination. Specific treaties may create standing councils and continuing implementation bodies. [United Nations Legal Affairs](https://legal.un.org/ilc/texts/instruments/english/conventions/1_1_1969.pdf) | Add permanent organizations, depositaries, monitoring, legal interpretation, and domestic implementation processes without assuming universal compliance |

The design implication is that **literacy, sovereignty, centralization, and diplomatic sophistication must not be one scalar progression**. An unwritten agreement can have elaborate authorization and maintenance; a literate empire can still rely heavily on personal relations.

---

## 5. Stylized facts and validation targets

A correct simulation should reproduce the following patterns **under the conditions that generate them**, rather than hardcoding their aggregate frequency.

| Pattern | Observable test in TCE |
| --- | --- |
| **An alliance does not mean joining every war.** Historical agreements distinguish defense, neutrality, consultation, and other commitments. [ATOP](https://www.atopdata.org/) | Some nonparticipation is coded as fulfillment or “not obligated,” not betrayal |
| **Treaty longevity and reliability are different.** Fixed terms can support cooperation; replacement need not represent failure. [Cambridge University Press](https://www.cambridge.org/core/journals/american-journal-of-international-law/article/duration-seriousness-of-commitment-an-empirical-and-theoretical-critique-of-nyarkos-treaties-vs-executive-agreements/9BC4EC9BCB4166FBD231EBB4B7008A93) | Record scheduled expiry, renewal, replacement, withdrawal, and violation separately |
| **Political and power changes affect reliability.** Leeds finds alliance violations associated with changes affecting the original bargain. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/abs/alliance-reliability-in-times-of-war-explaining-state-decisions-to-violate-treaties/EC9C08122EC24E9800F7AFE6000672E4) | Breaches cluster around relevant shocks rather than arriving as memoryless annual events |
| **Reputation affects opportunities without determining them.** Honoring earlier agreements is associated with subsequent alliance formation. [Arizona State University](https://asu.elsevierpure.com/en/publications/reliability-reputation-and-alliance-formation/) | Known reliable actors receive more acceptable offers, controlling for interests and capacity; unknown events have no instantaneous global effect |
| **Domestic agreement can be the binding constraint.** External and internal bargaining interact. [Cambridge University Press](https://www.cambridge.org/core/journals/international-organization/article/abs/diplomacy-and-domestic-politics-the-logic-of-twolevel-games/B2E11FB757C4465C4097015BD421035F) | Some attractive packages fail authorization; compensation or altered terms can change the domestic coalition |
| **War can reveal information while changing the object of negotiation.** Wartime bargaining models incorporate both learning and resource loss. [Rochelle Terman](https://rochelleterman.com/ir/sites/default/files/Filsen%202002.pdf) | The same offer can be rejected before a battle and accepted afterward for an explainable reason |

Add three controlled engineering tests:

**Complete-information peace test:** disable commitment failures, indivisibility, political misalignment, and decision errors; no war should occur when a feasible mutually preferred settlement exists.

**Delayed-information test:** a remote commander can act on obsolete orders after a central agreement, without the kernel automatically classifying the incident as deliberate state betrayal.

**Succession test:** replacing a ruler preserves institutional obligations where the constitutional rules require it, but may change willingness, capacity, and counterpart beliefs.

For empirical comparison, retain the denominator: **commitment opportunities**, not simply wars or treaty-years. Multilateral treaty observations also share participants and events; do not treat every dyadic expansion as independent evidence.

---

## 6. Recommended implementation for TCE

### 6.1 State representation

Use distinct but linked entities:

| Entity | Minimum state |
| --- | --- |
| **Diplomatic relationship** | Known counterpart representatives; communication routes; recognition claims; issue-specific interests; observed history |
| **Mission** | Sponsor, principal, envoy and entourage, destination, credentials, mandate, cargo, instructions, message history |
| **Agreement** | Parties, represented constituencies, approving institutions, clauses, effective conditions, public/secret portions, succession and termination rules |
| **Obligation** | Responsible actor, beneficiary, trigger, action, quantity or standard, place, deadline, exceptions, evidence, remedies |
| **Diplomatic incident** | Alleged act, reports and witnesses, competing interpretations, responsible actors, demands and responses |

Represent a treaty as a **multilateral object**, not merely a collection of bilateral relation bonuses. Some provisions apply to all participants, others only to a subset.

The binding subject should be explicit: a person, dynasty, office, community, polity, or participating coalition. Changing a ruler must not silently rewrite that choice.

### 6.2 Event-driven negotiation and execution

On contact or a new issue, identify authorized negotiators and generate a bounded set of packages. Evaluate each package against resource constraints, expected external alternatives, domestic approval, and post-agreement enforcement.

Send offers through the communication system. On arrival, the receiving institution may accept, reject, counteroffer, seek instructions, or delay. After authorization and entry into force, create obligation events.

When an obligation becomes due, resolve **actual performance** through the relevant subsystem. Grain must be collected and transported; troops must assemble and travel; access rights must be honored by local gatekeepers. Record observations separately from ground truth.

This produces visible diplomatic life: arriving delegations, lodging and feasts, guarded wards, councils, tribute caravans, border meetings, and couriers carrying news of an agreement that not everyone yet knows exists.

### 6.3 Computational scope and simplifications

The 10k–50k population does not require 10k–50k diplomatic planners. Most diplomatic decisions belong to a much smaller set of active representatives and institutions.

Use a sparse contact graph and an event queue. Reconsider relationships after material events or on a slow schedule; do not evaluate every polity pair every tick.

At the proposed upper prototype setting, **32 packages × 64 belief samples = 2,048 scenario evaluations per bargaining decision**, before authorization checks. These must use cached aggregate military, economic, and political forecasts—not 2,048 rollouts of the full population simulation. This is a suggested workload, **not a measured performance result**.

For v1, simplify free-form negotiation into a typed clause grammar and bounded package search. Keep natural-language explanations downstream of decisions. There is no need for a language model in the authoritative Rust simulation loop.

Do **not** simplify away conditional triggers, message delay, ratification, successor continuity, or actual resource delivery. Those mechanisms produce much of the historically interesting behavior.

### 6.4 Existing models and games worth borrowing from

| Model or game | Useful contribution | What not to assume |
| --- | --- | --- |
| **Filson–Werner, 2002** | An explicit repeated bargaining-and-fighting model with private information and changing resources. [Rochelle Terman](https://rochelleterman.com/ir/sites/default/files/Filsen%202002.pdf) | Its assumptions and equilibrium results are not universal historical transition probabilities |
| **Slantchev, 2003** | Simultaneous offers and fighting; information from negotiation and battle. [Cambridge University Press](https://www.cambridge.org/core/journals/american-political-science-review/article/principle-of-convergence-in-wartime-negotiations/7F3E14B2106771D14F74E6BBFE20945F) | Learning alone does not cover every commitment or domestic-political problem |
| **Axelrod’s emergence model; Cederman’s state-system models** | Bottom-up political aggregation, territorial actors, and endogenous system structure. [Taylor & Francis](https://www.taylorfrancis.com/chapters/oa-edit/10.4324/9780203993699-9/model-emergence-new-political-actors-robert-axelrod) | These are not complete treaty-negotiation or ratification systems |
| **Crusader Kings III: Wards and Wardens design** | Hostages are represented as people residing at another court, with a status distinct from ordinary prisoners. [Steam Community](https://steamcommunity.com/ogg/1158310/announcements/detail/5475861744260026903) | Deterrence penalties are game rules, not estimated historical effects |
| **CICERO / Diplomacy** | Separates strategic planning from controlled dialogue and predicts others’ actions using board state and conversation history. [Meta AI](https://ai.meta.com/research/cicero/) | Performance in a seven-player board game does not validate historical diplomacy or century-scale institutions |

The closest fit is a **hybrid**: Fearon-style bargaining benchmarks, Putnam-style authorization, relational agents, and an obligation-execution system attached to the rest of TCE.

---

## 7. Sources, datasets, and limits of inference

### Datasets to use

| Resource | Coverage and useful content | Main limitation |
| --- | --- | --- |
| **Alliance Treaty Obligations and Provisions, ATOP 5.1** | Alliance content for **1815–2018**; treaty-specific documentation and coding decisions. Best starting point for obligation types and alliance design. [ATOP](https://www.atopdata.org/) | Military alliances among documented states, not all diplomacy or all historical societies |
| **Correlates of War Diplomatic Exchange, v2006.1** | Diplomatic representation at chargé d’affaires, minister, and ambassador levels, **1817–2005**. Useful for representation networks. [Correlates of War](https://correlatesofwar.org/data-sets/diplomatic-exchange/) | Representation is not trust, first contact, or effective influence; inspect observation spacing |
| **Continent of International Law, COIL** | **234 randomly selected agreements**, **1925–2004**, covering economic, environmental, human-rights, and security issues. Useful for treaty-design comparisons. [Cambridge University Press](https://www.cambridge.org/core/journals/american-journal-of-international-law/article/duration-seriousness-of-commitment-an-empirical-and-theoretical-critique-of-nyarkos-treaties-vs-executive-agreements/9BC4EC9BCB4166FBD231EBB4B7008A93) | Modern registered agreements; over two-thirds of this sample were concluded in 1970–1999 |
| **PA-X Peace Agreements Database** | Agreement texts and tools for studying peace processes and implementation. Useful for constructing and testing richer peace-clause grammars. [Peace Agreements](https://www.peaceagreements.org/) | A peace-process collection is not the population of all inter-polity agreements |
| **eHRAF and source-specific ethnographic studies** | Context for nonstate relationships, reciprocal exchange, representation, and conflict settlement. Wiessner’s *Hxaro* study is particularly relevant. [eHRAF World Cultures](https://ehrafworldcultures.yale.edu/cultures/fx10/documents/020) | Ethnographic cases must not be treated as unchanged survivals of prehistory |

The central theoretical readings are Fearon’s **“Rationalist Explanations for War” (1995)**, Putnam’s **“Diplomacy and Domestic Politics” (1988)**, Powell’s **“War as a Commitment Problem” (2006)**, and the wartime bargaining models above. For historical mechanisms, prioritize **Kosto’s *Hostages in the Middle Ages* (2012)**, the **Amarna correspondence**, and translated agreements read alongside scholarship on their political setting.

Three uncertainties should remain explicit in the implementation documentation.

**First, surviving agreements are selected evidence.** Written court archives and successful preservation overrepresent some institutions. A recorded obligation establishes what was promised more securely than what was routinely performed.

**Second, the empirical meaning of reliability is disputed.** The approximately 75% and 50% alliance estimates are not interchangeable measurements of one immutable parameter. They should motivate alternative calibrations and careful coding, not be averaged into a universal constant.

**Third, institutions do not work independently of power and interpretation.** Gifts, marriage, hostages, law, and reputation can help sustain cooperation, but their effects depend on who values them, who can enforce them, and what future alternatives remain.

**For TCE, the decisive design choice is to make promises executable and contestable.** A convincing diplomatic history should emerge because particular people had particular information, authority, interests, and obligations—and because fulfilling or abandoning those obligations changed the world.

---

[Original ChatGPT research conversation](https://chatgpt.com/g/g-p-6ab9247582ac8191ba7a44a8a3ea0556/c/6ab92949-ac44-83ea-98d8-722358c060eb)
