# M4 design brief: governance primitives

**Scope.** This brief covers notables, the rule-based `Deliberator`, constitution primitives v0, succession, the law pipeline (plan §5.2, stages 1–7) and label inference. I assume sibling briefs own factions, coups, reputation, ideology, how news spreads, the god tools, crime, corruption and the policy library. Where this brief needs something from them, it says what.

**Evidence.** The reports agree on the architecture far more than on numbers:

- Institutions are positions, bodies, powers and procedures held by real people (09-02 §1.1, 09-01 §6.1).
- Offices arise from proposals that answer recurring problems (09-01 §6.2, 04-11 §5.4, 06-09 §1.2).
- Compliance is each agent's decision (09-02 §3.2, 09-04 §5.4).
- Labels are summaries worked out afterwards (09-01 §6.5, 09-15 executive recommendation).

They give no rates for how often offices form (09-01 §8: they "do not justify universal annual probabilities"), no elite share (04-11 §2.1), and thin evidence on early farming villages (09-03 §3.1, 09-04 §3, 09-05 §3.1). Nearly every number below is a design prior or a tuning value.

**What exists to build on:**

- People: five traits, but no values, grievances or event memory.
- Decisions: scored considerations, softmax and a receipt (`decide.rs`).
- The ledger: gift, share, wage and rent channels.
- Settlements: they hold land but no goods (ADR-0007), and village fields allocate land by need automatically.
- Keyed draws (ADR-0011 §3).

## 1. Mechanisms

### 1.1 Standing, following and notables

The plan and the research pull apart here:

- Plan §4.2 makes notables a cognition tier: the top ~1 % by an influence score, capped at 500.
- 04-11 §5.7 calls notability a presentation-layer judgment: "never grant powers because the interface selected someone as notable."

They reconcile if notability decides only who gets the costlier deliberation, and every power comes from an office, a seat in a body or relationships. At 1–2k agents, 1 % is 10–20 people. At today's ~65 it is less than one, so the rule needs a floor (tuning).

- **Esteem: what governance needs from the reputation model.** The factions brief owns that model, and there should be one model, not two. Governance needs four things from it:
  - Esteem per domain: provision, craft and counsel.
  - Fear, kept separate (04-11 §1.2; prestige and dominance correlated at r = 0.01, 04-11 §2.3).
  - Updates from events that already exist: a gift received in need, a kill shared, wages paid, a field let (dependency, 04-11 §1.5), a building admired, and kin.
  - Updates from new events: a proposal that passed and delivered, and an office duty done or neglected. Attribution is noisy (04-11 §1.2).
- **Following.** When a choice needs an endorsement, the person uses the existing softmax over the candidates they know plus an outside option. The score is U = B̂ + Â + L̂ − X̂ − K̂ (04-11 §5.2).
- **Influence (derived monthly, not saved).** It sums followers' weight, the powers of offices held, rank in goods and land, adult kin and founder status. Notables are:
  - the top people by influence, with display hysteresis;
  - holders of selection or removal powers;
  - each incumbent's strongest rival (04-11 §5.7).
- **Resisting overreach.** Refusing, withdrawing esteem and leaving are costly actions (04-11 §1.3), not a hierarchy modifier.

### 1.2 The rule-based Deliberator

Plan §4.2 names an HTN planner. The simplest thing that gives the reports' behaviour is a one-level scored choice, shaped like `decide.rs`:

- List the authored moves that this notable's powers and the current issues allow.
- Score each move on:
  - the notable's goals: their household's food outlook, their followers' outlook, their standing, and values later;
  - predicted support: the stage-2 function (§1.5), run only over people the notable knows (followers, kin, fellow members), since people act on estimates (04-11 §5.2).
- Choose by softmax with a draw keyed by (seed, purpose, person, day), and keep a receipt.

HTN waits for plans of several steps (M7).

- **The trait.** `Deliberator::choose(&NotableView, &[MoveOption]) -> (Choice, Receipt)`. The view is a copied struct of facts, so an `LlmDeliberator` can be plugged in later.
- **Moves v0.** Propose a policy, an amendment, or an office or body from templates (mediator, rotating overseer, storekeeper, collector, watch captain, headman: 09-01 §6.2); nominate; call a gathering; designate an heir; resign. Faction and coup moves, and the whisper tool's move (scored like any other, plan §2), come from the factions brief.
- **Issues** (09-05 §1.1). Authored kinds with detectors: a food shortfall against the outlook, thefts reported, fields short, a vacant office.
  - An issue only makes moves available.
  - A move's score comes from its forecast effect on the notable's goals. An authored issue→policy weight would make "a levy follows a famine" plot.
- **Cadence.** A routine review every 30 days, triggers on events, and crisis review every 1–7 days (09-03 §2.6).

### 1.3 Constitution primitives v0

These are 09-02 §1.1's families, cut to what one settlement needs:

- **Polity.** It has an id separate from the settlement even while the two are one to one (09-14 §5.1). M5's splinter founding needs this.
- **Body.** It has:
  - a membership rule: adults, household heads, holders of an office, or seats filled by a procedure;
  - a decision rule with 09-05 §1.4's fields: quorum, denominator, aggregation by person or household, threshold, ties, visibility;
  - an advisory or consenting role (09-05 §1.3);
  - a calendar and an agenda holder.

  A body has no mind; it acts through its members (09-02 §1.1).
- **Office.** It has a content title, seats, eligibility rules, a selection procedure, removal rules, powers and pay. Its tenure is stored as conditions, not only as days (09-02 §8.6).
- **Power.** A power is (holder, action, target, domain, conditions, delegability) (09-02 §1.2). Actions v0: propose, set the agenda, enact, levy, allocate, command, sanction (no courts), appoint or remove, and amend.
- **Selection.** Selection is a composition of steps, not an enum (09-02 §1.3, 09-03 §1.2):
  1. eligibility;
  2. candidates;
  3. selection by descent rank, a vote, appointment, lot (a keyed draw), acclamation or seizure;
  4. optional confirmation;
  5. installation.

  Heredity varies by its priority rule: primogeniture, seniority or designation.
- **The founding custom.** 09-01 reads "starting without government" as having no standing offices or compulsory treasury, not as having no rules.
  - v0 is one content-authored custom for every world: a gathering of adults that decides by acclamation, where anyone may propose and the same gathering amends.
  - The demo's "same start, five seeds" needs this. Divergence must come from dynamics.

**Rules for every institution:**

- **Acts take time** (09-01 §6.3): attending a gathering at the hearth, collecting, patrolling.
  - Attendance follows 09-02 §3.1's funnel: eligible ⊇ aware ⊇ present. So a gathering at harvest time is thin by mechanism.
  - Residents, eligible people, attendees and officeholders are counted separately (09-14 §1.5).
- **Failures stay failures.** An empty candidate pool, a failed quorum or a tie takes an explicit failure path and is flagged, never repaired (09-02 §10).
- **Institutions hold goods.** A levy and office pay need institutions in the ledger. Stock, records, claims and deliveries stay separate (08-11 §5.2), and office assets stay apart from the holder's household (08-11 §1.3).

### 1.4 Succession

- **The episode.** A vacancy opens an episode with separate timestamps for departure, selection, accession, recognition and consolidation (09-03 §1.1).
- **Claims, not a ruler** (09-02 §3.6). The rule produces eligible claimants, at most 8 (4–16, 09-03 §2.6). Eligibility is cached and invalidated on births and deaths (09-03 §5.6).
- **Recognition.** Selectors or followers recognize a claimant by softmax over the claimants plus "none". The score weighs procedural validity, entitlement, esteem and expected benefit: 09-03 §5.2's five-part legitimacy vector and its separate support utility, cut to four terms.
- **Three maps.** Legal title, recognition and operational control stay separate (09-03 §5.4). "Contested" means two claimants each hold more than a tuning share of recognition. Escalation belongs to the coup brief.
- **A minor heir.** A guardian chosen by rule holds the powers until a majority age (tested at 12–21, 09-03 §2.6). This is 09-02 §10's first test.
- **What passes.** Property, eligibility and control pass; followings are reconsidered, never copied (04-11 §1.9, §5.5). Relatives are allies too, so there is no instability term per relative (09-03 §1.4).
- **Rates.** 09-03's figures describe European monarchies and must not become rolls (§2.3): deposition per reign of 16/8 %, 49/21 % and 57/43 % under primogeniture, election and agnatic seniority (§2.2), and civil-war onset rising from 2.7 % to 6.3 %/14.9 % in a year a ruler dies (§2.3). A world sees perhaps 2–4 successions in 40 years, so test the mechanics (§4.1–4.7), not the rates.

### 1.5 The law pipeline (plan stages 1–7)

A law is a versioned object with its full history saved, which the demo needs ("one law's full history is inspectable"). 09-05's finer states map onto the plan's seven stages:

1. **Proposal.** A sponsor with the power to propose (under the founding custom, anyone) files a typed policy with parameters, or an amendment, against an issue.
   - Admission to the agenda is a separate power, so a proposal can wait undecided forever (non-decision, 09-05 §1.2).
2. **Support.** Each eligible member takes a stance: support, tolerate, abstain or object.
   - The stance comes from the forecast effect on their household (cached per situation, 09-05 §5.3), esteem for the sponsor, the authority's legitimacy, and values later.
   - Factions aggregate the stances.
   - The sponsor may offer three parameter levels (3–8, 09-05 §2.3).
3. **Decision.** The route the constitution sets for the domain (09-05 §5.1): a decree by the power's holder, or a vote of the members present.
4. **Promulgation.** Authorized, in force, communicated and implemented are separate states (09-05 §1.7).
   - Attendees know at once. The information brief spreads awareness to everyone else.
   - The engine's knowledge of a law is nobody's knowledge.
5. **Enforcement.** Capacity comes from officers' actual hours on duty and their skill (09-02 §3.3), with a backlog (09-01 §1.7).
   - The plan's "officers × competence × (1 − corruption)" should be the result of that, not a scalar (09-02 §5.2: prefer derivation).
   - Corruption is officers' choices (crime brief).
6. **Compliance.** It is decided when a law bears on a choice (09-04 §5.4), as an added consideration: expected sanction p_D·p_E·S (09-02 §3.2), plus legitimacy and norm terms, minus the cost of complying.
   - Detection and enforcement are separate probabilities (06-09 §1.4).
   - Receipts read "the curfew: −x points".
   - Being unable to comply is recorded apart from refusing (09-01 §6.4).
7. **Effects and feedback.** Goods move on new ledger channels. Experienced events update each person's assessment of the authority.
   - The update is x′ = x + η·c·a·(y − x), with η(Δt) = 1 − 2^(−Δt/h), and no drift to neutral without evidence (09-04 §5.3).
   - Grievance goes to the factions brief.

**The minimum of legitimacy.** Each person holds assessments of 4–8 authorities (09-04 §5.6): the gathering, the head office, its holder and the watch.

- Each has 09-04 §5.1's six dimensions; the sacred one stays unused until religion exists.
- They combine as L = σ(b + gΣwx) (§5.2).
- This costs about 192 KB at 2k people.

### 1.6 Label inference

The label comes from a pure classifier (09-15 §6.7) over two inputs:

- **The de jure graph:** who selects, removes, commands, levies, judges and amends.
- **De facto evidence over a time window:**
  - binding adverse decisions obeyed, or "unobserved" when there were none (§6.5);
  - seizures of power;
  - whether authority outlived its holder;
  - the effective number of decision-makers by domain (04-11 §4);
  - legal and group participation shares (09-15 §6.3).

It composes 09-15 §2's terms (council community, big-man leadership, chiefdom, monarchy, republic, oligarchy), with "why" sentences rendered by the kernel.

- The rules are explicit, not learned (§6.8).
- Nothing in the simulation may read the label (09-01 §6.5: "the label must never grant the capabilities used to classify it"). Enforce this through module dependencies.
- Polity, regime episode, government and officeholder each have their own id (09-15 §6.6).

### Parameters

| Parameter | Start (range) | Source and status |
|---|---|---|
| Routine political review | 30 d | Reports differ: 7–30 (09-01 §3.3), 7/30/90 (09-02 §5.2), 30–90 (09-14 §2.2, 09-03 §2.6) |
| Notables | ~1 %, plus floor and hysteresis | plan §4.2; the floor is tuning (04-11 §2.1: no universal share) |
| Small council | 5–15 (3–30) | 09-05 §2.3 (09-14 §2.2's 6–24 / 20–200 are urban) |
| Rotating term | 3–12 months | 09-14 §2.2; design |
| Legitimacy half-times | 1 y / 3 y / 20 y (performance, procedure, custom) | 09-04 §2.3; ranges 0.25–2, 1–5, 10–40 |
| η_e; g; β_L | 0.15; 4; 2 | 09-04 §2.3 |
| Detection; enforceability | 0.30 (0.05–0.80); 0.70 (0.20–1.00) | 06-09 §2.2; derive from patrols where possible |
| Levy share | 0–30 % sweep | 09-01 §3.3; Gavrilets' 10–30 % is "not a universal harvest tax" (§3.2) |
| Classifier refresh; window; persistence | 30 d; 24 mo; 6 mo | 09-15 §4B |
| Population gate for any label | none | 09-15 §4B, 09-01 §3.1 |

## 2. What to defer

| Defer | To | Why |
|---|---|---|
| Party elections; courts, precedent, appeals; HTN planning | M7 | Plan §7. The sanction power has no appeal until then |
| `LlmDeliberator` | M12 | Plan §8, open question 2 |
| Several polities, charters, ruler–town bargaining (09-14 §1.2–1.4), delegation over distance (09-01 §1.7) | M5–M6 | One settlement in M4 |
| Commons associations (06-09), municipal boards (09-14 §1.7) | M6 | They come with irrigation earthworks and services, and reuse M4's pipeline |
| Drafting capacity (09-05 §1.6) | Writing | M4 queues only meeting time |
| Clerical rule | Religion primitives | Plan §8, open question 4 |
| Dataset adapters (09-15 §6.7) | After v1 | Worlds are below Polity's 500k threshold (09-15 §5) |

## 3. Pitfalls, and how to show it works

- **Plot.** Watch for population thresholds (09-01 §3.1), issue→policy weights, and template weights tuned until a chief appears. Time-box, then log a `NUDGE:`.
- **A label that restates the configuration.**
  - Keep a table-driven test of 09-15 §7's synthetic cases. For example, a doubled population keeps its label, and privileged households selecting officials is an oligarchy, not a democracy.
  - Every demo world starts from one custom, so different labels must come from history.
  - The dashboard row "≥2 of 5 labelled differently" goes live.
- **Labels or notability granting power.** Enforce by module dependency, and test that disabling the classifier leaves every digest unchanged.
- **Runaway concentration.** Office → wealth → influence → office is a loop. Run 04-11 §5.8's ablations and 09-04 §2.3's ablation with no legitimacy effect, as smoke checks.
- **Instability.** Watch for constitutions that flip back and forth (use label persistence), for every death becoming a crisis (09-03 §4.1), and for double counting. Resolve commitments once (04-11 §5.3), and add no governance factor on top of patrols (06-09 §2.2).
- **Leaving dominates.** Villages facing a short year leave today (§9 NUDGE: "Revisit with M4's councils"). Exit utility should count what would be lost (09-01 §1.2).
- **Scale.** "A 500-seat council is not automatically meaningful in a polity of 700 people" (09-02 §9.4); likewise the 1 % notables rule.
- **Performance.** Politics is cheap; the day loop is the cost. At 44 ms a day for ~65 people, linear growth (not measured) gives ~1 s a day at 1,500, about 4 h a world for the 40-year demo. Political events must be identical in both modes (ADR-0011 §1, §3).
- **Tests.**
  - 09-02 §10's edge cases: an empty pool, a tie, no quorum, a minor heir, rival claimants.
  - Gate A, and an exact save → load → save.
  - 09-04 §4's test: settlements with equal compliance but different legitimacy diverge when enforcement is removed.
- **In the observer.** A government panel (the label, why, and who selects, removes and commands); a law's timeline (sponsor, stances with reasons, vote, awareness, enforcement, evasion, goods moved); notables with why each is notable; chronicle lines (09-01 §6.7).

## 4. Choices that are expensive to reverse

- **ADR candidate 1: institutions in the kernel, saves and boundary.** It covers:
  - polity, body, office and procedure saved as values that point to content vocabulary;
  - what a save does when a referenced kind is gone. Saves refuse rather than repair (ADR-0002); a fall back to a default, as regimes have, would rewrite a constitution;
  - versioned laws with their histories;
  - succession episodes and legitimacy;
  - whether the label's evidence window is saved, so the label can be rebuilt on load.

  New wire tables are append-only under ADR-0001.
- **ADR candidate 2 (or an amendment to ADR-0006/0007): institutions as holders of goods in the ledger.** The rule that every good is accounted for has to cover them. Allocation by need may become a held power.
- **Content API.** Office and body templates, selection steps, issue kinds and possibly label rules. Saves will bind to their ids.
- **Not ADRs** (record them in §9): the `Deliberator` trait, scored moves instead of HTN, and parameter values.

## 5. Open questions for the designer

1. **The founding custom.** One gathering for every world, or a choice per world like regimes?
2. **Allocation by need.** Should it become a power the gathering holds, so an office can capture it? That reopens ADR-0007, but it is a strong route to oligarchy.
3. **Notables.** Percentile, threshold, or top-k with a floor? Is notability purely a compute tier?
4. **Scored moves instead of HTN.** Acceptable as a §9 amendment to plan §4.2?
5. **Label rules.** Content predicates or code? What is the label when nothing has ever been decided? 09-15 has no term for it.
6. **Values and ideology.** These do not exist yet. Which brief adds them, and when?
7. **Attendance.** Must a vote need people physically at the hearth in v0?
8. **Legitimacy.** Owned here or by the factions brief? 09-04's six dimensions per authority and 09-03's five per claimant overlap but do not match.
9. **Scale.** Can 1–2k agents meet the demo's budget without the samplers ADR-0011 §4 defers?
