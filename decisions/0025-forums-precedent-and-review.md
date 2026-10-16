# ADR-0025: Forums, precedent and review

Status: Accepted
Date: 2026-10-16
Milestone: M7b

## Context

Through M7a, every case is heard by the polity's one body at a gathering (ADR-0015 §4): those
present stand by what they believe, their household's stake and their regard for the bringer
against the accused, and `Body::decide` finds or not. A finding imposes the law's fixed bundle as
obligations, and a watcher may go to collect a refused one (`force.rs`). Every case is kept whole.
There is no forum other than the body, no precedent beyond what a finding made people believe, no
rule above the body, and no way to end a law except a later law. A curfew has no sanction and the
watch reports nobody (plan §9, slice AD). Councils, terms, lot and elections exist (ADR-0024), so
a seat and a method of filling it are vocabulary already.

Plan §7's M7 asks for "courts and precedent, and judicial review as a check", and its demo has
"a court strikes down a curfew". The courts brief (`docs/briefs/m7-courts.md`) reads 09-07,
09-06, 09-02, 09-05, 09-04, 06-05 and 06-09. Its invariants:

- justice is a branching dispute process, and disputes outnumber adjudications (09-07 §1.1,
  §2.2);
- who decides, how evidence counts and how a verdict is aggregated are separate (09-07 §1.3);
- procedure and truth are separate outputs (09-07 §1.4);
- precedent is several values (authority, reach, revision), and does not converge (09-07 §1.5);
- a check is a relationship: review needs a trigger, a scope, a suspension rule and a remedy
  (09-07 §4.4), and someone to carry rulings out (09-07 §1.1, 09-06 §1.3);
- courts come from demand and supporters, never dates (09-07 §6.3).

Lived worlds see about one taking a decade (plan §9, slice AD), so forums may have little to
hear. Curfew breaches are plentiful, which is why standing to challenge a law comes from bearing
it, not from a case of breach.

## Decision

### 1. A forum is a law

- An appended `PolicyKind`, *hear cases* (`HearCases`), names a forum: the laws whose cases it
  hears (by kind), its seat and its rule. The gathering stays the default forum; nothing creates
  a court.
- Its seat is one of three, each content's to offer: one holder, named and lapsing as an office
  is, with a tenure as ADR-0024 §2 gives offices; a panel of k drawn by a keyed lot from the
  body's members for each sitting (`count::draw`); or a membership the custom knows (the elders).
- A panel decides by its own quorum and `PassRule` (a `Body` value); one holder decides alone. No
  new aggregation.
- It sits the evening after a case is brought to it, at the seat's hearth. Its hearers are unpaid,
  as the watch is.
- The issue *cases waiting* opens the move: a case a household brought went `Unheard`, or a member
  holds a grievance against the body over a case. It weighs toward none (ADR-0013 §5).
- `CaseStage` gains `Settled` and `Withdrawn` (append only): the accused's household may offer
  what a finding would cost it before the sitting, and the accuser may accept, by the existing
  demand and comply points; a bringer may withdraw.

### 2. Precedent is the saved case and the remembered finding

- Every case since M4 is a precedent; the gathering's findings count (09-07's opening: remembered
  precedents from the beginning). Its fact tags are derived when needed from what the case holds,
  never from the incident: the law's kind, whether the accused is of the settlement, kin or not,
  the accounts, admitted or denied, and for a challenge the limit and the law.
- Memory is people's: those present hold the finding as a claim (an appended `ClaimKind`, finding)
  with its source; others hear it as news; it fades and dies with its holders.
- When a sitter takes a stance, they recall up to `k` remembered findings with the most matching
  tags, newest first (a design prior, 3 to 8).
- Authority is the forum's value, adopted and changed by amendment: *informative* (nothing beyond
  belief), *persuasive* (points toward the recalled findings' direction by share of tags matched)
  or *binding* (a large weight that stake and regard can still outweigh).
- A finding against a recalled one with the same tags records "departed from case N". Nothing
  forces convergence.

### 3. A limit is a law above laws

- An appended kind, *limit* (`Limit`), adopted only through the custom's amendment route (open to
  the overruled and the aggrieved, judged by how it would have decided what they saw). It carries
  its own `PassRule` for its own change (entrenchment).
- What a limit forbids is a predicate over a law's own fields, from a closed set the kernel knows
  and content parameterizes: v0 has *no law keeps a household from its own work in daylight* (a
  curfew's hours against the day's light) and *no levy above a share* (`levy_share`). Each binds
  any law with those fields; content may not author one aimed at curfews alone.
- A law that breaches a limit still passes. The limit matters only when someone challenges and a
  forum with review finds against the law.

### 4. Review is a power a law gives a forum

- A *hear cases* law may grant review. It names that anyone bound by a law and bearing its cost
  may challenge it under a limit in force; a challenge does not suspend the law; the remedy is
  that the law no longer binds (`LawStatus::StruckDown`, appended, naming the case).
- Standing comes from bearing the law (for a curfew, a grown member it keeps home in its hours),
  never from a case of breach. Curfews keep no sanction in M7b.
- Choosing to challenge is 09-07 §4.1's U = p̂(success)·V − C: V the household's forecast gain if
  the law ended (as a repeal is weighed), p̂ from the predicate as they read it and the findings
  they remember, C the hours and the regard they expect to lose with the law's supporters.
- Sitters weigh the reading (a clear breach, clearly none, or a margin), their household's stake,
  regard for the challenger against the law's sponsor, and recalled findings.
- Answers are moves already open to the overruled: pass the law again (and face a challenge),
  change the limit at its entrenched rule, repeal the forum or its review. None is weighted.

### 5. Rulings are carried out and known as anything else

- Findings are carried out as now (obligations, refusals, collection). A ruling is known person by
  person: who knows a law was struck stops weighing it; who does not still keeps it.
- An appended norm kind, *abide by findings*, drawn and learned as the first is, so people can
  trust the gathering and not its hearer, or the reverse (09-04).
- Grievances over a finding blame the forum (an appended `Blamed` target), not the body.
- Defiance is recorded, never blocked.

### 6. Not built in M7b

Office pay and fees; writing and codes; oaths and ordeals; rehearing and appeal; pardon; a
standing mediator; a forum per settlement (one forum at the seat); collective liability; land
and debt disputes (M7d); a courts god tool (never).

## Consequences

- Easier: M7c's captured assembly can be reviewed; offices with tenure and lot are reused.
- Harder: cases gain stages and claims; a sitter recalls findings at each stance (bounded by k).
- A world with no *hear cases* law gives the same digests as before (tested).
- Forums, limits and challenges may never arise in lived worlds. That is a result, and the demo
  shows what its world reached, nudging only priors after a time box (`NUDGE:`), never an event.

## Alternatives considered

- **A court as a second body beside the gathering.** Lost: ADR-0013 §1's one body stands, and a
  forum is a law naming who hears what, as ADR-0024 made a council a membership.
- **Review as a property of any forum.** Lost: 09-07 §4.4 asks for a trigger, scope and remedy
  given by someone; a forum without review is common and must be possible.
- **Limits written as content predicates in a small language.** Lost for v0: a closed set the
  kernel reads keeps them checkable against the fields laws carry; content parameterizes them.
- **A curfew sanction so that breaches become cases.** Lost: it would multiply cases by
  hundreds a decade and decide the demo's route; standing from bearing reaches the same check.

## Revisit when

- A forum per settlement or appeal is wanted (polities of several settlements at scale).
- Writing arrives (record reach and persistence change, not authority).
- Office pay or fees exist (financial dependence of hearers).
