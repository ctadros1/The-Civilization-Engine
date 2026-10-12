# ADR-0015: Incidents, cases and obligations

Status: Accepted
Date: 2026-10-08
Milestone: M4b
Amended: 2026-10-10, M6 design: incidents of every kind, episodes and typed links (§8).

## Context

M4a gave each settlement a polity, laws with whole histories and a common store. Nothing can yet
go wrong between people:

- Nobody takes what is not theirs.
- A household that runs short asks, and is given to or refused, but no one can wrong it.
- A law's only obligation is a levy, paid or kept back at threshing.

M4b brings theft, what is seen of it, the responses of victims and of the polity, a watch,
punishments and corruption v0 (plan §7). Its design needs one decision expensive to reverse: how
what happened, what an office knows and what each person believes are kept and saved, and how a
sanction becomes something someone owes.

Reports read: 04-09 (criminology), 12-04 (policing), 09-06 (enforcement and compliance), 09-07
(justice and courts), 09-09 (corruption), 08-13 (taxation), 08-18 (the informal economy) and
06-10 (dark content). The M4 crime brief (`docs/briefs/m4-crime.md`) digests them. Their main
points:

- **Crime is encounter-driven, not a daily roll** (04-09, opening): opportunities come from daily
  life, need awareness and access (§5.2), and pass a moral filter before a noisy choice, or a
  noisy chooser "may eventually make almost everyone offend" (§5.3).
- **Four things stay separate** (04-09, opening): an action occurred; someone considered it
  wrongful; an authority learned about it; an institution classified and resolved it.
- **Truth and knowledge are separate layers** (12-04 §5.2, "the highest-value architectural
  decision"): an accusation can enter without an offence, an offence can stay unknown, and a
  case can close without finding the offender.
- **Knowledge of misconduct travels along relationships** (12-04 §1.2); several witnesses
  repeating one rumour are not several observations (12-04 §1.5).
- **Perceived and actual enforcement differ** (04-09 §5.4); certainty of being caught matters more
  than severity (04-09 §2.2).
- **Restitution, compensation and a fine are different transfers** (09-07 §1.2), and punishment
  is a bundle, never one severity number (09-07 §6.2). An award creates no money: an obligation
  has debtors, beneficiaries and default rules (09-07 §1.2).
- **Corruption is actions by people with discretion or entrusted goods**, never a rate (09-09
  §1.2; 12-04 §1.7).
- **Dark content is told plainly**: who did what to whom, under which institution (06-10 §4).

## Decision

### 1. Three layers, kept apart

- **Incidents are the kernel's truth.** An incident records its kind (an attempt or a completed
  taking), the actor, the target household, the goods, the time and place, and who saw it. Only
  the kernel reads incidents; no choice reads one.
- **Beliefs are what people know.** A person learns of an incident by seeing it, by being told
  by someone who knew, or by the polity's decision. Each belief records its source. Several
  people repeating one teller are one source.
- **Cases are what a polity knows.** A case opens when someone reports to the body or office that
  hears cases. It holds the allegation, its leads with their sources, its stage, the hours spent
  on it and a closure reason. It knows only what reached it.
- Choices read beliefs and cases, never incidents. A test checks that everything a case holds
  traces to a witness, a report or a decision.

### 2. Taking is a choice

- **`Take` is a behaviour** (appended), scored like `Ask` with the same considerations: the food
  the household needs, the walk, the person's objection to taking, what they believe they would
  lose if seen and punished, and what they would lose with those they regard.
- **A moral filter comes first.** Above a threshold objection, `Take` is not a candidate, and the
  receipt says so.
- **Targets are found as `Ask` finds a giver:** the stores of households the person knows, within
  reach. Whether anyone is home and awake, or in sight, is settled on arrival: the taker turns
  back or is seen.
- **Goods move on a `take` channel** and are never created. Every good stays accounted for.
- **Each person carries an objection to taking and perceived risks.** The objection is drawn at
  birth, pulled toward the parents', and moved by what the person lives. The risks are updated
  from what the person sees and is told, never from the kernel's rates.

### 3. Responses are choices too

- A victim notices a loss at the next use of the store. It knows who took only if a member saw,
  or someone who saw tells it.
- It chooses among letting it go, demanding restitution, telling those it is tied to, and
  reporting to whoever hears cases. Each choice has a receipt.
- A household that believes someone took from it, or from those it regards, refuses their asks.
  Exclusion is credible only when enough households believe it.

### 4. Cases are decided by the polity's own procedure

- **No courts** (plan §7; 09-07 §1.1: settlement and adjudication are steps of one dispute
  process): no forum, evidence rules, panels, precedent or review. The
  body or office that a law gives the power to hear cases decides summarily from its case.
- **Under the founding custom the gathering hears cases.** A case is put to it as a law is: those
  who come take a stance from what they believe and their regard for each party, and the body's
  rule decides. A law may give the power to an office instead.
- **The sanction comes from a law**, a policy template's bundle (§5) at levels people propose. A
  world without such a law has only victims' own responses.
- The decision is recorded like a law's, whole: who came, where each stood and why. A wrong
  decision is a possible outcome, recorded as such in the truth layer and never repaired.

### 5. Every sanction is an obligation

- An **obligation** has a kind, an amount, a debtor, a beneficiary, a due date, its progress and
  a default rule.
- The v0 kinds are restitution (to the victim), compensation (to the victim), a fine (to the
  polity's store), labour-days (worked for the polity) and exile (leaving, recorded as a
  migration, not a disappearance).
- Prescribed, imposed and completed stay apart: a law prescribes a bundle, a decision imposes it,
  and the debtor pays, works, leaves or defaults.
- Payment moves on its own channel. An award never creates goods: a debtor without them owes an
  arrear, and the default rule says what follows.
- No corporal punishment or execution in v0 (plan §9).

### 6. The watch and corruption are people's choices

- **A watch is an office** named by a law, as the storekeeper is. Its holders walk rounds as an
  activity they choose; on a round they guard and see only where they are. Enforcement capacity
  is the watch's hours, with a backlog of cases, and a reported measure, never an input.
- **Corruption v0** is a choice of someone with discretion: an officer who identifies a taker
  may take a payment, or look away. A payment moves on a `bribe` channel. It is exposed only as
  any taking is, by those who saw. No person carries a "corrupt" flag (09-09 §5.1).

### 7. The boundary

- **Saves** gain sections for incidents, cases, obligations and each person's beliefs, objection
  and perceived risks. The schema version is bumped.
- **The ledger** gains channels for taking, restitution, compensation, fines, labour and bribes,
  appended.
- **The wire** keeps truth and knowledge in separate tables. A panel shows them side by side,
  labelled, and cannot mix them by accident: what happened, what the polity knows, and what
  people believe.
- **The chronicle** tells incidents and decisions plainly: who did what to whom, and under which
  law (06-10 §4). Minor takings are summarised; decisions and exiles are told one by one.

### 8. Incidents of every kind, episodes and typed links (M6)

M6 adds illness, fire, high water, storm damage and raids (plan §7). The M6 observer brief
(`docs/briefs/m6-observer.md`) reads 12-07, 15-01, 15-02, 15-04 and 15-05 for them.

- **Every kind keeps §1's layers.** An incident of any kind is the kernel's truth: what
  happened, where, to whom, and its losses. What people know is beliefs and claims with their
  sources (ADR-0016 §3), and what a polity knows and did is its cases, issues and laws. A kind
  that has no layer yet (no polity hears of fires) shows the layer as empty, never as filled
  from the truth.
- **An incident is a chain of stamped steps** (12-07 §1.1): it began, was noticed (by whom), was
  told (to whom), help came (who, from where, when), useful work began, and it ended, with its
  losses. Each step is recorded when it happens; a step never reached stays not reached, and one
  known only to the day keeps a day, never an invented hour. Arrival alone never decides
  success.
- **Episodes.** An outbreak, a conflagration, a flood or a siege is an episode record: a kind, a
  span, a footprint, the shared cause recorded once (a river's peak, a storm's gusts) and an
  ending. Its incidents are `part_of` it. When an episode of a kind opens and closes is that
  kind's rule, set by the ADR that brings it (ADR-0021 for outbreaks).
- **Typed links, recorded when they happen** (15-02 §1.1): `caused_by` (a case to the source or
  person it was caught from, a fire to its ignition or a burning neighbour, a failure to the
  storm or flood, anything to the observer's `Influence` that started it), `part_of` (an
  incident to its episode) and `responded_to` (a proposal to the incidents its sponsor's issue
  names, from the issue's own record). A link points at a record that exists and is never
  inferred from time: where only time joins two things, the text says "after" (15-01 §3.5).
- **Each kind keeps its own store.** Takings stay as §1–§7 built them. One query presents every
  kind in one shape (a row, its layers, its steps, its links), so one panel can list them.
- **Aggregates and coverage are derived.** Rates, counts by kind, response times (with "none
  came" kept as an outcome, 12-07 §1.8) and coverage are computed from the records and the
  walking ground, rebuilt on load and never saved. Each states its scope, its period, its
  denominator and how many of its incidents the observer started.
- **No choice reads an incident, an episode, a link, an aggregate or a coverage figure.** It is
  tested as labels are: a world lived with every query called daily matches one lived without.
- **The boundary.** Saves keep incidents, episodes and links, appended with the first kind that
  needs them (M6a's illness). The snapshot carries only a revision that changes when an incident
  or episode opens, moves on or ends; rows, pages and episodes are queries paged by a cursor, so
  an outbreak of hundreds of cases never fills a frame. The chronicle tells an episode when it
  begins and when it ends, with counts; every death is told and resolves to an incident and a
  `Cause`; small incidents dealt with at once are summarised, as minor takings are (§7).

## Consequences

- Theft, reporting, cases and punishments arise from people's choices; crime rates are outputs.
- The observer can show the gap between what happened and what is known.
- Saves bind to new content ids (offence policies, sanction kinds).
- **Forbidden:**
  - a crime rate as an input, or a controller that pulls toward one;
  - any choice reading an incident;
  - an office knowing what nobody told it;
  - a sanction that creates goods;
  - repairing a wrong decision.

## Alternatives considered

- **A daily crime roll per person.** Rejected by 04-09's opening: crime is encounter-driven.
- **One "crime" scalar per settlement.** It cannot tell recorded crime from crime (12-04 §4.B).
- **Courts now.** Plan §7 defers forums, evidence rules and appeal to M7; cases are shaped so
  that a forum can later sit between "identified" and "sanctioned".
- **Severity as one number.** 09-07 §6.2: a bundle of distinct transfers.

## Revisit when

- Courts arrive (M7): forums, evidence rules, precedent and appeal.
- Violence is modelled (injury first; 04-09 §5.5).
- Several polities exist (M5): exile to a neighbour, jurisdiction over distance.
- Story patterns beyond episodes are wanted (15-02 §1.4), or `enabled_by` is needed to explain a
  chain the stored links cannot.
