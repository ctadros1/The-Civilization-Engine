# Proposal (status: **Proposed**, not decided): a sewer's mending backlog, and abandoning it

Backlog C's "Not yet" list (plan §7, M11f) leaves open two lines: "backlogs of mending a store
cannot pay (12-08 §5.4)" and "a polity choosing to abandon a sewer rather than mend it while its
law stands". Research 12-08 §5.4 treats them as one mechanism, so this brief does too. Nothing is
implemented and nothing is decided.

## What exists now (claude/claude-development 48cd61b4)

- **The review** (`population/sewers.rs`, `sewer_reviews`; ADR-0044 §5): while the law that laid
  a sewer is in force, the polity weighs each fault it has seen (a collapse, or a home backing up
  for want of room). It compares the hours spared over the sewer's remaining life against relining
  or clearing work and materials. Where the repair is worth it, the review asks the work as laying
  was asked: an equal share for each household, given as they choose, and paid where a law pays
  for common work.
- **What is missing:** nothing records work asked but not given, or asked and unpaid because the
  store is empty. A fault that is not worth mending is simply not asked, every review. The only way
  out is ending the law, after which nobody mends.

## What the research says (12-08)

- §5.4: keep a queue of required work, `backlog(t+1) = max(0, backlog(t) + new − completed)`,
  prioritised by the institution's behaviour, not an optimal scheduler. "Lost revenue → postponed
  work → poorer service → … → larger backlog" is "a simulation mechanism to test, not an inevitable
  trajectory". Recovery must stay possible. "Abandonment should sometimes be rational… demolition
  and material reuse should compete with preservation."
- §4.3: routine care does not substitute for renewal. §4.4: expenditure can produce no service.

## Proposed design

1. **A backlog per sewer, in hours.** It is saved on `Sewer`, as an append. Each review adds the
   hours it asks, and each hour of sewer work given subtracts one (`sewer_work` already credits the
   hours). The backlog never goes negative. Faults the review judged not worth mending are **not**
   added: the backlog is asked work left undone, not every defect.
2. **Unpaid is visible, not penalised.** Where a law pays for common work and the store cannot pay
   an hour given, the hour is still given, as now, and the shortfall is recorded beside the backlog
   (`unpaid_h`). The existing household choice to give work already weighs pay. Nothing new
   forces or discourages it.
3. **Abandonment is weighed at the review, as laying was.** When a sewer's backlog exceeds the
   hours its remaining life would spare (`sewer_spared_h` × life left, the review's own quantity),
   the review proposes to its gathering an amendment ending that sewer's upkeep. This is a
   `PolicyKind` appended on `lay_sewer`'s pattern, voted as laws are, so households decide. If it
   passes, the sewer is kept as a record and no more work is asked. Homes stay connected while
   cells stand and heap again as cells fall, through the existing rule. A later law may take it up
   again. The research's "material reuse" (salvaging walls) is not part of this brief.
4. **The observer:** the sewer's page shows its backlog hours, unpaid hours, and "abandoned by law
   on day N" where that applies.

No tuning value is added: the trigger compares two quantities the review already computes.

## Acceptance examples (tests to write)

- **Backlog arithmetic:** a review asking 40 h, then 25 h given, leaves 15 h. A further ask of
  10 h makes 25 h. Giving 30 h makes 0, not −5.
- **Not worth it is not backlog:** a fault the review declines to ask adds nothing.
- **Unpaid recorded:** with pay for common work in force and an empty store, 10 h given gives
  `unpaid_h = 10`, and the work counts against the backlog.
- **Abandonment proposed only when the backlog exceeds what is left to spare:** a sewer with 300 h
  of backlog and 200 h left to spare has the amendment proposed. At 100 h of backlog it does not.
  Once the amendment passes, later reviews ask nothing for that sewer.
- **Recovery:** a new `lay_sewer` law naming the same sewer resumes upkeep, and the backlog is kept.
- **No sewer, no change:** worlds without sewers are unchanged apart from length-only digest
  effects. Saves round-trip exactly.

## Bounded implementation plan

Four commits:
1. Backlog and unpaid fields, saved, with arithmetic tests.
2. Crediting from `sewer_work` and the store's pay.
3. The amendment kind, its proposal at the review, and the gathering's vote, with tests.
4. Observer words.

Estimate: one to two sessions. It touches `polity.rs` (Sewer, PolicyKind), `population/sewers.rs`,
`population/polity.rs` (proposal), `persist/agents.rs` and a sewer test file.

## Uncertainties for the owner

- Whether abandonment should be a separate amendment (proposed here) or simply the existing repeal
  of the sewer's law. Repeal would also stop wear-mending of sewers the polity still values.
- Whether the backlog should decay when the fault it was asked for is mended by other means. The
  proposal keeps hours as asked, per 12-08's queue.
- Whether priorities between several sewers' backlogs need a rule now. The proposal leaves the
  existing review order, which is the institution's behaviour.
