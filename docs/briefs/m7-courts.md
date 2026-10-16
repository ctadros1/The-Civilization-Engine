# M7 design brief: courts, precedent and judicial review

**Scope.** Plan §7's M7 ("Politics deepens") asks for "**Courts and precedent**, and judicial review as a check", and its demo includes "a court strikes down a curfew". Plan §5.2 lists "judicial review (M7)" among the checks and says "Courts and precedent come in M7". This brief covers, for polities of 100–3,000 people:

- courts as bodies or offices: who judges, and how they are chosen;
- a case's procedure beyond today's gathering hearing it in one sitting;
- precedent: what a precedent record is, and how past findings bind or guide later ones;
- judicial review: a court striking down a law, on what grounds, under what higher rule, at whose suit, and how a village could come to have such a check at all;
- how rulings are carried out, and obeyed or not;
- customary law and enacted law.

Sibling briefs own elections and parties, corruption and patronage, lobbying, and credit. This one needs from them only that a forum's members are ordinary people with kin, regard and stakes, as gathering members are now. Every weight below is a **tuning value** or a **design prior** unless a citation gives it.

**What exists to build on** (checked in the code):

- **Cases** (`civ-agents/src/crime.rs`, ADR-0015 §4). A `Case` holds the incident as a join key it never reads, the law it is brought under, the accuser's household and the member who brought it, the accused and their household, the food lost, its `leads` (distinct witnesses: several people repeating one are one lead), its stage, when it was heard, the eligible count and each stance (`CaseStance`: stance, stake, belief, regard). `CaseStage` is `Open`, `Found`, `NotFound`, `Unheard` or `Lapsed`, append only.
- **Bringing a case** (`population/cases.rs`). Only under a law against taking (or raiding) that the bringer knows: "knowing of a forum is one of the barriers" (09-07 §4.1). `report_points` is 09-07 §4.1's U = p̂(success)·V − C, with p̂ = 1 − 0.5^accounts, a tuning form. A watcher may bring what they saw (`WatchRecord::cases`).
- **Hearing** (`put_cases`, `hear_case`). Cases go on the next gathering, or one is called `notice_days` on (1 in the core content). Each body member present stands by what they believe (`belief_points`), their household's stake in days of food, and their regard for the bringer against the accused; `Body::decide` gives `Found`, `NotFound` or `Unheard`. A finding imposes the law's fixed bundle as `Restitution`, `Compensation` and `Fine` obligations due in `due_days` 30, and exile if the law says so. Those present come to believe the finding. A gathering may find against an outsider, but "its finding binds nobody there" (M6c slice BI). A broken *no raids* clause is heard as a case (README).
- **After a hearing.** `grieve_case` (`population/word.rs`) gives grievances against the body (`Blamed::Body`) for `FoundAgainst`, `NotFound` and `Unheard`. `Owed` has four kinds; ADR-0015 §5 also names labour-days, which `Owed` lacks. `Standing` keeps `Refused` (unwilling) apart from `Defaulted` (unable).
- **Carrying out a finding** (`population/force.rs`). A watcher may go to collect a refused finding, at `w_collect` plus their own norm points; each adult answers `Yielded`, `Barred` or `Struck`, and every blow is a `Harm` in an `Encounter`.
- **The custom** (`polity.rs`). `Body` is a membership (`Adults`, `Elders`, `Landholders`, `Watch`), a `quorum_share` and a `PassRule` (`MoreForThanAgainst`, `TwoThirds`). `versions` keeps every `CustomVersion` with the law that made it or who seized it; `body_at(t)` gives the body that decided at a time.
- **Laws.** `LawStatus` is `Proposed`, `InForce`, `Rejected`, `Lapsed`, `Superseded` or `Carried`. There are 19 `PolicyKind`s. `KeepWatch` names a holder and lapses when they die or leave: an office in its first form. `AmendBody` changes one gate at a time and is open only to someone a gathering overruled or who holds a grievance against the body; `decisions_seen` and `body_gain` judge a rule by how it would have decided the decisions seen under the present custom (a guard added after "two camps toggle[d] the custom week by week"). `Repeal` ends a named law.
- **The curfew.** A `Curfew` sets hours; keeping it is each person's choice, weighed in `curfew_now` by their norm points, where they stood on it and their regard for its sponsor, never below nothing. It has **no sanction**, and "the watch neither stops nor reports those out at night" (plan §9, slice AD). It counts breaches by those who knew of it and those who did not.
- **Norms** (`norm.rs`, ADR-0016 §4). One norm kind is built, `AbideByLaws` ("what the gathering decides binds"). `norm_points` feeds the curfew, the levy, collection, bands, wells and enclosures.
- **What is missing.** No office hears cases: ADR-0015 §4 allows a law to give the power to an office, but none does. No precedent, no review, no higher rule than the body itself, no office pay, no per-authority legitimacy score (regard and grievances against the body stand in for it), and no writing in the content: every law is oral, known person by person (`Law::known`).
- **How much there is to hear.** In the ten-year smoke, three coast villages saw about one taking a decade and broke their curfews 720, 1,655 and 1,218 times; the fifty-year dashboard worlds saw no attempt to take at all (plan §9, slice AD). M6b left "the watch seeing a noncompliant roof and bringing a case" waiting for courts (plan §9, M6b design).

## Where the reports agree

Ten points recur across 09-07, 09-06, 09-02, 09-05, 09-04, 06-05 and 06-09. I treat them as invariants.

1. **Justice is a branching dispute process.** "Model justice as a branching process for handling grievances—not as 'crime → arrest → punishment'" (09-07, opening). People may ignore, negotiate, retaliate or seek a forum, and "negotiation can recur at several stages" (09-07 §1.1). Institutional responses combine restitution, mediation, exclusion and more "according to local rules" (04-09 §5.6).
2. **Disputes outnumber adjudications.** 49 % of people surveyed had a legal problem in two years; 29 % sought advice and 17 % went to a third party, courts and others together (09-07 §2.2). Access is a chain of barriers: recognition, standing, knowing a forum, affordability, safety, proof, an enforceable remedy (09-07 §4.1).
3. **Who decides, how evidence counts and how a verdict is aggregated are separate** (09-07 §1.3). Adjudication "is a choice of decision procedure, not a technology tier" (09-07 §1.3). A larger panel does not "automatically supply independent information" (ibid.).
4. **Procedure and truth are separate outputs.** "Procedural acceptability and factual correctness must remain separate outputs" (09-07 §1.4). Record the outcome and the process apart: a loser can come to accept the court (09-04 §1.2).
5. **Precedent is several dimensions, and does not converge.** Written coverage, authority hierarchy, precedent authority, interpretive discretion, record reach and revision procedure are separate values; avoid "a `CommonLaw | CivilLaw` switch" (09-07 §1.5). 465 appellate decisions on one rule developed divergently (09-07 §1.5); "do not hard-code inevitable improvement through precedent" (09-07 §6.5).
6. **A check is a relationship, not a bonus.** Veto, accountability and separation are compositions of powers and procedures; a court with review authority but no evidence, enforcers or independence "has a different practical effect" (09-02 §1.4). "An independent court can be powerless; an effective court can be politically dependent" (09-07 §4.3).
7. **Review is several primitives.** Rehearing, appeal, sentence confirmation, pardon, review of administrative conduct and constitutional review differ; specify "who can trigger review, what it can examine, whether it suspends enforcement, and what remedies the reviewer can order" (09-07 §4.4).
8. **Rulings need people to carry them out.** "Require an actual payer, collector, guard, escort, community coalition" (09-07 §1.1). Detection, decision and execution are stages that each consume work and can end for a recorded reason (09-06 §1.3, §5.5). "Cannot enforce" and "will not enforce" differ (09-06 §1.1). The chain of rules ends where "an official or group must decide whether to carry out a sanction" (09-02 §10).
9. **Courts emerge from demand and supporters, never dates.** "Repeated unresolved conflicts create demand for dependable arbitration", but "demand does not guarantee creation" (09-07 §6.3). Institutional change is political action: amendment, reinterpretation, exception, noncompliance or replacement (09-02 §3.7).
10. **Custom and enacted law coexist.** "An oral rule is not automatically informal" (09-02 §6). A customary prescription can "accumulate recognition through repeated application, dispute settlement, and endorsement" (09-05 §3.1). Writing should enable records and authoritative versions, "not automatically turn all norms into a comprehensive statutory code" (09-05 §3.1).

**What models and games teach to avoid:**

- Eco's "automatic rule enforcement is precisely what TCE should replace with knowledge, enforcement agents, discretion, and evasion" (09-07 §6.5; 09-05 §5.6).
- Dwarf Fortress's justice "is not a validated reconstruction of historical courts" (09-07 §6.5).
- A "judicial independence +20" modifier (09-02 §1.4); V-Dem's ordinal scores read as probabilities (09-07 §2.2, §4.3).
- Ordeals as "supernatural access to the kernel's truth state" (09-07 §1.4).
- "Permanently loyal police" carried over from the Epstein model (09-06 §5.6).

**Where they differ, or differ from the plan:**

- **When precedent begins.** 09-07's opening says "customary adjudication and remembered precedents should exist from TCE's beginning"; the plan put precedent in M7. ADR-0015 §4 kept every gathering's decision whole, so the findings since M4 can serve as precedent now (open question 1).
- **What a court is.** Plan §5.2 lists "judge" among powers and judicial review among checks, but no forum. 09-07 §6.1 asks for a forum entity with jurisdiction, access rules, personnel, selection, calendar, budget, authority sources, evidence rules, decision procedure and review links. Most of these are a `Body` and a law already; the rest are new.
- **The higher rule.** Constitutional review needs something to review against. TCE's only higher rule is the body, and failures there are never repaired, so no passed law breaches it. 09-02 §9.1's record carries `precedence_and_exceptions`, and 09-07 §1.5 an "authority hierarchy". Something new must hold limits on laws (§1.4). This is the expensive part (§3).
- **The demo names an outcome.** "A court strikes down a curfew" assumes a court, a higher rule and a challenge. 09-07 §6.3 says demand does not guarantee creation. A village may never build review. §3 treats this as a plot risk, as the M6 war brief treated "a siege ends in annexation".
- **Who brings a curfew to court.** Today a curfew has no sanction and the watch reports nobody (plan §9, slice AD), so no case of breach can arise. Review must come either by a challenge from someone bound (abstract) or by first giving curfews a sanction (concrete) (open question 5).
- **Written law.** The assignment says "written law"; the content has no writing. Enacted (passed) and customary (held) law are the distinction that exists. 09-05 §1.7's Icelandic law-speaking shows public oral transmission can be "an institutional responsibility".
- **Scale.** Europe's median of 17.6 professional judges per 100,000 (09-07 §2.2) gives about 0.5 for 3,000 people (my arithmetic); 09-07 §6.4 calls such figures "a derived illustration of scale—not a recommended staffing ratio" and says village adjudicators "should usually be agents with other occupations".
- **Volume.** 09-07 §6.3's demand needs repeated unresolved conflicts. Lived worlds give about one taking a decade, or none (plan §9, slice AD). Curfew breaches are in the thousands. Courts may have almost nothing to hear unless curfews, building rules or land become cases.

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| A forum | A law naming it: its seat rule (a holder, a panel by lot, or a membership), its `Body`-style decision rule, the laws it hears, who may bring, what counts as an account, its precedent authority, its sitting calendar, and any power of review | A law the gathering passes; selection by its steps |
| A case, extended | Appended stages (settled, withdrawn, struck down), the forum, the accused's answer, the precedents each member recalled, a departure from a recalled finding, a rehearing link, hours spent | People bringing, answering, settling and sitting |
| A limit | A content template and its parameters, the law that adopted it, its own pass rule for change | An amendment by the custom's procedure |
| A challenge | A case whose subject is a law: the challenger, the law, the limit, stances, outcome; the law's appended status `StruckDown` naming the case | A person with standing |
| Remembered findings | Claims (ADR-0016 §3) held by those present and those told | Hearings and talk |

**Derived, not saved:** each case's fact tags, a forum's backlog and times to closure, lines of findings, independence measures, and every rate. **Size:** cases are sparse; a forum is one law; a limit is one law.

### 1.1 Who hears cases

**Reports.**

- Decision-makers can be a mediator, elders, a ruler, a magistrate, a judge, a lay panel or a jury; appointment, inheritance, election, rotation and lot are separate selection primitives (09-07 §1.3). Athens used lot and large panels with restricted eligibility (ibid.).
- For a panel, keep members' beliefs and relationships, and aggregate by an authored rule (09-07 §1.3).
- The Althing separated the gathering, the law council and the lawspeaker (09-05 §3.4).
- Commons that last provide "a local hearing or mediation process whose travel, delay, and participation costs are much lower than prolonged retaliation" (06-09 §1.1, principle 6).
- Individuals or coalitions supply adjudication "when authority, compensation, prestige, or political control makes it attractive" (09-07 §6.3).

**My proposal.**

- **The gathering stays the default forum** (ADR-0015 §4). Nothing creates a court.
- **A law may name a forum.** An appended `PolicyKind` (call it *hear cases*) names the laws whose cases it hears, and its seat:
  - one holder, nominated as `KeepWatch` nominates and lapsing as it does;
  - a panel of k drawn by lot (a keyed draw) from the body's members for each sitting (09-07 §1.3);
  - or a membership the custom already knows, such as the elders.
- **A panel decides by a `Body` value.** Its quorum and `PassRule` are the forum's own. One holder decides alone. No new aggregation code is needed.
- **The issue that opens it** (appended): *cases waiting*, when a case a household brought was `Unheard`, or waited past a span, or a member holds an `Unheard` or `NotFound` grievance. It opens the move and weighs toward none (ADR-0013 §5).
- **What a household forecasts.** The cases it expects (from its known takings), heard by people it regards against the gathering's members, and the hours its adults would no longer sit. Illustration (my arithmetic, with two hours a sitting as an assumption): 23 adults at a hearing spend 46 person-hours; 09-07 §2.3's prior of 0.25–3 adjudicator-days is 2–24 hours at eight hours a day.
- **No pay and no fees in v0.** A hearer is unpaid, as the watch is (plan §9, M4b design). This keeps 09-07 §4.3's financial dependence empty until office pay exists.

### 1.2 A case's procedure

**Reports.**

- Incident → attribution → ignore, avoid, negotiate, retaliate or seek a forum → settlement or adjudication → remedy → review, enforcement, evasion or default → changed relationships (09-07 §1.1).
- A forum has "a small set of admissible evidence categories and thresholds", and adjudicators interpret them "with imperfect knowledge, credibility judgments, and possible pressure" (09-07 §1.4).
- Track pending cases as B′ = B + A − H − S − W, with adjudication, settlement and withdrawal kept apart; a forum sitting every s days adds an average wait of s/2; record "time to closure and whether the claim was resolved" (09-07 §4.2).
- Design priors (09-07 §2.3, low confidence): mediation 0.25–2 facilitator-days; routine adjudication 0.25–3 adjudicator-days; sittings 7–90 days apart; rehearing 0.5–2 times the first effort.
- Among the Enga, third parties "commonly helped with compensation and reintegration, while direct third-party punishment was rare" (06-05 §1.6).
- People may answer an obligation with "negotiation, appeal, bribery, relocation, and resistance" as well as compliance; "disagreement over liability" is its own case (09-06 §1.5).

**My proposal.**

- **Standing is the forum's value.** Who may bring: the household wronged (today), the watch (today), or any member. Bringing stays `report_points`, with the forum's wait and the bringer's hours added to C.
- **Sittings.** A forum sits on the calendar its law sets, or when a case is brought (09-07 §2.3's 7–90 days is for intermittent forums; a village hearer can sit the next day). Waiting then emerges (09-07 §4.2).
- **Settling before a hearing.** Once a case is brought, the accused's household may offer what the finding would cost it, and the accuser may accept, using the existing demand and comply points. The case closes as *settled*. A bringer may *withdraw*. Both stages are appended to `CaseStage`.
- **The accused answers.** Deny, admit, or plead that the law does not bind (§1.5). An admission counts as an account.
- **Accounts.** The forum's law says whether being told counts or only having seen (09-07 §1.4). Leads stay deduplicated by origin. No oaths or ordeals: they need beliefs the kernel lacks (09-07 §1.4).
- **The finding.** Each forum member stands as gathering members do now (belief, stake, regard), plus the precedent term of §1.3. The bundle stays the law's (prescribed = imposed) in v0 (open question 8).
- **Rehearing.** If the forum's law grants it, a party found against may ask the gathering to rehear; whether obligations wait meanwhile is the law's switch (09-07 §4.4).
- **The record** keeps time to closure, whether it was resolved, and the hours each sitter spent (09-07 §4.2; 09-06 §1.4).

### 1.3 Precedent

**Reports.**

- A judgment record holds "material fact tags, interpreted rule, decision, authority, jurisdiction, date, later treatment" (09-07 §6.1).
- Retrieval "can retrieve a few cases sharing offense tags, relationship categories, and jurisdiction. Oral transmission can preserve a small, selective set of remembered decisions; writing expands persistence and reach" (09-07 §6.4).
- Precedent authority may be "binding, persuasive, or merely informative", apart from interpretive discretion and record reach (09-07 §1.5). Refinement differs from overruling, and overruling "can produce persistent instability" (ibid.).
- Gossip must keep provenance, so ten repetitions of one report are not ten reports (06-05 §1.5).

**My proposal.**

- **A precedent is a saved case.** ADR-0015 §4 already keeps every decision whole. Its fact tags are derived on load from what the case knows, never from the incident: the law's kind; whether the accused is of the settlement; kin or not; how many accounts; admitted or denied; and, for a challenge, the limit and the law.
- **Memory is people's.** Those present hold the finding as a claim (ADR-0016 §3) with its source; the forum's sitters hold theirs; others hear it as other news. It fades as claims fade and dies with its holders. A world without writing keeps a short, selective case law (09-07 §6.4).
- **Recall.** When a sitter takes a stance, they recall up to k remembered findings with the most matching tags, newest first. k of 3–8 is a design prior (09-05 §2.3's bound for alternatives weighed, borrowed).
- **Authority is the forum's value,** adopted and changed by amendment: *informative* (nothing beyond what the finding made people believe, as today), *persuasive* (points toward the recalled findings' direction, by share of tags matched) or *binding* (a large weight that stake and regard can still outweigh: 09-07 §1.5's interpretive discretion, and 09-02 §10's "keep interpretive decisions attributable to actors").
- **Later treatment.** A finding against the direction of a recalled one with the same tags records "departed from case N". Departures are the record of 09-07 §1.5's overruling.
- **Custom from findings.** A line of findings (say, hungry takers let off) becomes what people expect of the forum. This is 09-05 §3.1's custom by "repeated application, dispute settlement, and endorsement", and needs no record beyond the cases and the claims.

### 1.4 Custom, enacted law and higher rules

**Reports.**

- Keep three representations: the authoritative rule, people's beliefs and expectations, and observed behaviour (09-02 §6). "Customs, interpretation, precedent, selective enforcement, and ambiguous language matter" (09-02 §10).
- ADICO lacks constitutive rules: those that make "a valid judgment" or an office (09-02 §2.1). 09-02 §9.1's rule record carries `precedence_and_exceptions`.
- An "authority hierarchy" decides "which sources override others: enactments, religious texts, custom, ruler's orders, decisions" (09-07 §1.5).
- "Custom constrains rulers as well as authorizing them" (09-04 §1.2).
- A legal change separately moves legal knowledge, expected enforcement, legitimacy and beliefs about others' approval (06-05 §1.9).

**My proposal.**

- **No "written law" until writing exists.** Enacted law is what a body passed. Customary law is the custom's body and the norms and expectations people hold. Writing, when it comes, should change record reach and persistence (09-07 §6.4), not authority.
- **A higher rule is a limit.** An appended kind adopted only by the custom's own amendment procedure. It names what laws may not do, as a predicate over a law's own fields, so whether a law breaches it can be read from the law, which its people know. Content authors limit templates as vocabulary, for example (each to be checked against what laws carry):
  - no law keeps a household from its own fields in daylight (a curfew's `hours`);
  - no law sends anyone from the valley but on a finding (a `Sanction`'s `exile`);
  - no levy above a share (`levy_share`);
  - no law binds households its body does not admit (a `Body`'s membership).
- **Entrenchment.** A limit carries its own pass rule for change, such as two-thirds (an existing `PassRule`). A later simple majority can still pass a law that breaches it. The limit matters only if someone challenges and a forum with review finds against the law. Until then a limit is words, as a claim is "words until outsiders heed it" (`ClaimPlace`).
- **Who proposes a limit.** The `AmendBody` route: someone a gathering overruled, or who holds a grievance against the body. They judge a limit as a body is judged now, by how it would have decided the decisions they saw (`body_gain`). It weighs toward none.
- **The custom as a ground.** A law decided by a body other than the one in force, or in a founding window after a seizure (ADR-0017 §5), can be challenged on procedure. Ordinary failures never pass (ADR-0013 §2), so this ground is rare by construction.

### 1.5 Judicial review

**Reports.**

- Review needs a trigger, a scope, a suspension rule and remedies (09-07 §4.4). It "should therefore not unlock only when a polity adopts democratic or separation-of-powers institutions": Qing review ran inside an imperial hierarchy (09-07 §4.4, §3.2).
- Legal vetoes, political vetoes and delay powers differ (09-05 §1.5). "Decree authority, representative deliberation, popular ratification, and constitutional review can coexist" (09-05 §3.10).
- Political change can modify "who occupies a gate, not only by abolishing it" (09-05 §3.9).
- Independence is a vector: appointment, removal, finances, social ties, decisional and enforcement autonomy (09-07 §4.3).
- Users need a route "to propose, contest, or amend rules" (06-09 §1.1, principle 3).

**My proposal.**

- **Review is a power a law gives a forum** (appended), never a property of having a forum. It names who may challenge, which limits it hears, that a challenge does not suspend the law in v0, and its remedy: the law no longer binds.
- **Standing.** A member bound by the law who knows it, whose household bears its cost: for a curfew, a grown member it keeps home in its hours (`curfew_now` already finds them). Within a case, the accused may plead the same.
- **Choosing to challenge** is 09-07 §4.1's U = p̂(success)·p̂(enforcement)·V − C, with V the household's forecast gain if the law ended (the gain a `Repeal` move uses), p̂(success) from the limit's predicate as they read it and the findings they remember, and C the hours of a hearing and the regard they expect to lose with the law's supporters (06-05 §1.6: power and dependence decide which sanctions are feasible). It keeps a receipt.
- **Hearing a challenge.** Each sitter weighs:
  - the reading: a clear breach, clearly none, or a margin either side (where the curfew's hours lie near daylight's edge), so sitters can differ;
  - their household's stake in the law;
  - regard for the challenger against the law's sponsor;
  - recalled findings;
  - what they expect if they strike it: the forum's removal rule, and how those who chose them stood on the law (09-07 §4.3, appointment and removal dependence).
- **The remedy** appends `LawStatus::StruckDown` with the case. The law's history shows it whole. Nothing before it is undone: breaches stay recorded.
- **Answers are moves.** The gathering may pass the law again (and face a new challenge), change the limit (at its entrenched rule), remove the forum or its review power, or replace its holder. Each is open to those the ruling overruled, through the existing *overruled* route. None is weighted.
- **How a village comes to have review:** only by its own laws, a forum, a limit and a review power, each passed by its custom's procedure and weighed by forecast. Most worlds may never pass all three. That is a result, not a gap (09-07 §6.3).

### 1.6 Rulings carried out, and obeyed or not

**Reports.**

- "Illustrative stage probabilities of 0.5, 0.8, 0.5, 0.5, 0.5 imply an effective sanction probability of only 5%" (09-06 §1.3).
- Unwillingness, inability, ignorance and disagreement over liability must "generate different behavior under assistance, reminders, appeals, threats, and seizures" (09-06 §1.5).
- Legitimacy, moral agreement, descriptive norms and expected reactions enter choices separately (09-06 §1.7). Lawful differentiation, triage, favouritism and coercive limits are four kinds of selective treatment (09-06 §1.8).
- A court ruling raised perceived social support without changing personal attitudes in Tankard and Paluck's sample (06-05 §1.9).
- Executions per capital sentence at the Old Bailey fell from 68.5 % in the 1750s to 10.9 % in the 1810s (09-07 §2.1): prescribed and completed differ.

**My proposal.**

- **Findings are carried out as now.** Obligations, refusals and a watcher's choice to collect (`force.rs`). With a forum, the watcher collects the forum's finding by the same choice. "Will not" and "cannot" stay recorded apart.
- **A ruling is known person by person.** Those present learn it; others are told, as laws spread (`Law::known`). Someone who knows the curfew was struck stops weighing it; someone who does not still keeps it. The engine's knowledge of a ruling is nobody's (09-05 §1.7).
- **The norm question.** `AbideByLaws` is "what the gathering decides binds". A forum is not the gathering. I suggest an appended norm kind, *abide by findings*, drawn and learned as the first is, so people can trust the gathering and not its hearer, or the reverse (09-04, opening: people can "accept a court while distrusting tax collectors") (open question 7).
- **Defiance is recorded, not blocked.** A struck law passed again, a hearer removed after a ruling, a finding nobody collects: each is an event in the record. The label classifier's evidence of "binding adverse decisions obeyed" (m4-governance brief §1.6) can count rulings, and stays a label nothing reads.
- **Grievances.** A finding against a member's view adds grievances as `grieve_case` does now, blamed on the forum (an appended `Blamed` target) rather than the body, so 09-04's per-authority separation shows in what people hold.

## 2. What to defer, and why

| Defer | To | Why |
|---|---|---|
| Office pay, court fees | With office pay or credit (M7's credit brief) | Nothing pays officers now; fees need a treasury flow (09-07 §4.3, financial dependence) |
| Written records, a code | Writing | No writing content; writing changes reach, not authority (09-05 §3.1; 09-07 §6.4) |
| Confinement, corporal punishment, execution | Not planned | Plan §9 rules out the last two; custody needs food, guards and a place (09-07 §3.1, §6.2) |
| Collective liability (a kin group pays) | With kin groups beyond the household | 09-07 §1.2's *diya* needs a paying group the kernel lacks |
| Oaths and ordeals | Religion | They need beliefs in supernatural judgment (09-07 §1.4) |
| Pardon and commutation | When a polity has an office that could hold it | A distinct review primitive (09-07 §4.4) |
| A standing mediator office | After forums are measured | Settlement between parties (§1.2) comes first; 06-05 §1.6's third parties can be added as an office later |
| Courts above courts | Polities of several settlements (ADR-0023 §9) at scale | Appeal from a held settlement to the seat is plausible but needs jurisdiction rules (open question 10) |
| Land, tenure and debt claims | The credit brief, or a later slice | The m4-crime brief §1.6 deferred tenure disputes to M7's forums; this brief makes forums able to hear any law's cases |
| A courts god tool | Never | It would choose a ruling |

## 3. Plot guards, risks and validation

**Plot.** The demo names its outcome. A curfew struck down must come from a minority overruled on it, a limit someone proposed and a gathering entrenched, a forum someone named, and a challenger who chose to bring it. No date, era or size may create a forum, a limit or a challenge, and no issue weighs toward one (ADR-0013 §5). Content must not author a limit aimed at curfews alone; the daylight limit must bind any law with hours. A scaffold world may set conditions, as `theft_world` empties stores (plan §9, slice AD), never decisions. After a time-boxed week without a strike-down, nudge a prior (the weight a household puts on hours a curfew keeps it home, or the cost of proposing an amendment), never an event, and log a `NUDGE:` (plan §1, rule 3).

**Nothing to hear.** Lived worlds see about one taking a decade, or none (plan §9, slice AD). Forums may never form, precedent may never build, and the *cases waiting* issue may never open. Count the funnel before tuning: takings known, cases brought, unheard, settled, found; members overruled; limits proposed and passed; challenges weighed and brought. A move never invoked may be "an unreachable action" (the M6 war brief §3). Curfew breaches are plentiful (720–1,655 a decade in the smoke's villages), which is why standing to challenge a curfew comes from bearing it, not from a case of breach.

**Toggling.** Ruling, re-enactment and new challenge could loop, as body amendments once toggled week by week (the guard in `decisions_seen`). Measure it first. Entrenchment and the cost of each challenge should damp it; add no rule unless the loop is seen.

**Leaking truth.** A forum knows what was brought (ADR-0015 §1). A limit reads a law's fields, which its people know; it never reads an incident. Test that every recalled precedent traces to a claim its sitter holds.

**Compute.** Justice runs event by event (09-07 §6.4): on a filing, a sitting, a settlement, a challenge. Recall is bounded by k. A world with no forum law must give the same digests as the build before. Add cases by stage and rulings to Gate B (ADR-0011 §5), with wide tolerances.

**Stylized facts and tests** (plausibility checks, never targets):

- **Disputes outnumber hearings.** Known takings, demands, settlements, cases and findings counted apart; the 17 % third-party share (09-07 §2.2, §5) is a direction, not a target.
- **Wait is calendar plus queue.** A forum sitting every 30 days adds about 15 days of wait before congestion (09-07 §4.2's s/2, my arithmetic).
- **Precedent need not converge.** Two polities from one start keep different lines of findings on like tags (09-07 §5).
- **An independent forum can be powerless.** Hold the forum fixed and remove anyone willing to collect; rulings stay findings, unpaid (09-07 §4.3).
- **Procedure and outcome apart.** A party who loses can still hold no grievance against the forum (09-04 §1.2).
- **Prescribed, imposed, completed.** Counted apart for every finding (09-07 §5; 09-06 §5.7 adds successful appeals and uncollectible arrears).
- **Informal is not benign.** Settlements can rise with coerced terms; in Liberia, mediation training in 86 of 246 towns brought more land-dispute resolution, less violence and more extrajudicial punishment (09-07 §3.2).
- **Paired runs changing one feature** (09-07 §6.5): sittings more often; persuasive against binding precedent; review with and without anyone to act on it; a gathering hearing cases against a hearer.

**Observer.**

- The Cases panel shows each case's forum, the accused's answer, the precedents each sitter recalled, any departure, and how it closed, with truth, the forum's knowledge and beliefs in their columns (ADR-0015 §7).
- A law's history shows challenges and rulings. The Government panel lists forums, limits and review powers, each with the law that made it.
- The chronicle tells rulings plainly: who challenged what, under which limit, who stood where.

**Choices that are expensive to reverse.** One ADR for forums, precedent and review: a forum as a law with a `Body`-style rule and a seat rule; appended case stages and `StruckDown`; limits as constitutive content with predicates over law fields and their own pass rule; precedent derived from saved cases plus remembered claims; review as a power. It amends ADR-0013 §2 (powers gain *judge* and *review*), ADR-0015 §4 (its "no courts" ends) and ADR-0016 §3 (a claim kind for findings). Elections, corruption and credit will want theirs, so one is the budget here (plan §1).

## 4. Suggested slices

1. **A forum by law:** the *hear cases* policy (holder, panel by lot, or a membership), the *cases waiting* issue, sittings, settled and withdrawn closures, and time to closure; the gathering stays the default.
2. **Precedent:** fact tags derived from saved cases, findings held as claims, recall by sitters, authority as the forum's value, and departures recorded.
3. **Limits and review:** limit templates adopted by amendment with entrenchment, the review power, challenges by those with standing, `StruckDown`, and the gathering's answers as moves.
4. **Rulings carried out:** rulings spreading person by person, the *abide by findings* norm, grievances against a forum, and the dashboard's justice counts.
5. **The demo:** a court strikes down a curfew in a lived or scaffolded world, time-boxed, with any `NUDGE:` logged.

## 5. Open questions for the designer

1. **Old findings as precedent.** Do the gathering's findings since M4 count as precedent, as 09-07's opening wants, or only a forum's from M7?
2. **Seats.** One holder, a panel by lot, a membership, or all three in v0?
3. **Limits.** Content predicates over law fields, or code? Which templates for v0, and does each map onto fields laws already carry?
4. **Review.** A separate power passed by its own law, as proposed, or named inside the limit that it enforces?
5. **Standing.** Abstract challenges by anyone bound, as proposed, or only pleas within a case? The second needs curfew breaches to become cases.
6. **Curfew sanction.** Should breaking a curfew carry a bundle, and the watch report those it sees out, in M7? It would multiply cases (720–1,655 breaches a decade in the smoke).
7. **The norm.** Does a forum's finding engage `AbideByLaws`, or a new *abide by findings* norm?
8. **Discretion.** May a forum lower a law's bundle (09-07 §1.1, "legally authorized discretion"), or is the bundle fixed?
9. **Rehearing.** Built in v0? Do obligations wait while it is pending?
10. **Several settlements.** One forum at the seat, or one per settlement, with appeal to the seat?
11. **The demo's nudge.** Which prior, if any, may be nudged after the time box?
12. **Land and debt.** Are tenure and credit disputes this milestone's cases or the credit brief's?
