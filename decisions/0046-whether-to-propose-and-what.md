# ADR-0046: Whether to propose, and what

Status: Proposed (2026-11-18), for the owner. Not accepted: it changes how every sponsor in every
world chooses, and the notables' gate's tolerances were calibrated on the present choice.
Date: 2026-11-18
Milestone: M4c's deliberation (ADR-0013 §5), found by the seventh coverage pass's diagnosis
Amends (if accepted): ADR-0013 §5 (the rule-based deliberator's choice), ADR-0039 §2 (the rule
the model's fallback draws on)

## Context

The rule-based deliberator (ADR-0013 §5, `RuleDeliberator::choose`) scores each move a sponsor
may make. The score is the chance it passes, times what they forecast for their household and
those who regard them, less `propose_cost` (0.5). It then draws one move from a softmax at
`temperature` 0.5 over **every move and one "propose nothing"** scored 0.

So the chance of proposing anything grows with **how many moves are on offer**, whatever each is
worth. Two templates with identical forecasts count twice. This is the familiar logit
duplicate-alternatives problem (the red bus and the blue bus).

**How it was found.** Fixture 1, stream 5, every adult deliberating, replayed with each sponsor's
moves logged (plan §9, "Why 'an evening at the hearth given once' broke fixture 1"):

- **Day 661.** Marek's nine moves score between −0.83 and +0.46 against nothing's 0.
  - "Nothing" draws only 0.116 (0.140 with the forum fix).
  - A forum draws 0.381 (0.253), although every forum scores below nothing (best −0.175, with
    the fix −0.474).
  - `name_a_hearer` and `zestfully_name_a_hearer_for_a_year` forecast exactly alike. The
    forecast cannot see a term, so each has its own share of the draw.
- **Days 1025 to 1095.** The forum has lapsed and cases wait. Each week a sponsor weighs six
  forum templates, all below nothing (Marek's best −0.48 to −0.51; a lot about −0.01).
  - "Nothing" still draws only 0.34 to 0.37 for Marek and 0.21 to 0.22 for the others.
  - 9 of 13 weekly chances became proposals, each before a council that could not sit.
- **Where the scores come from.** The sponsors' own gains there are about 0.002, so every score
  is about −`propose_cost`. It is the count of options, not their worth, that proposes.

**The engine's own precedent.** The daily choice of activities settled this question on
2026-10-03 (plan §9, "Choosing among acceptable options"). Its softmax samples only the options
worth more than doing nothing (research 01-09 §4.3 applies softmax to acceptable alternatives),
because sampling every option let unfavourable choices win draws: a hungry village gathered
7–12 t of firewood nobody needed. The deliberator never had that rule.

**Research.** 09-05 §1.2: a sponsor weighs expected success and what proposing costs. Nothing in
09-05 or 01-09 suggests that the number of drafts a sponsor could write raises the chance they
write one.

## Options

Each option was built on scratch and run through the notables' gate (40 runs, the eight
aggregates: every adult deliberating against the tier, as a difference of the tier's mean). The
base is 06d020e, with a council's quorum counted against those seated (plan §9). B was also run
on 677ddcb, before that fix (the last row).

| Gate | Fixture 1 proposals | Fixture 1 enacted | Fixture 1 offices | Fixture 1 moved | Fixture 2 proposals | Fixture 2 enacted | Fixture 2 offices | Fixture 2 moved |
|---|---|---|---|---|---|---|---|---|
| Tolerance | 105 % | 55 % | 85 % | 95 % | 105 % | 55 % | 85 % | 95 % |
| A, base (06d020e) | +51.9 % | +42.3 % | +20.0 % | +38.6 % | +52.7 % | +42.1 % | +25.0 % | −9.5 % |
| B, two stages | +66.7 % | +45.8 % | +0.0 % | **+96.9 %** | +24.3 % | +14.8 % | +50.0 % | −20.5 % |
| C, acceptable moves only | −6.5 % | −6.5 % | −70.0 % | −48.8 % | −10.0 % | +0.0 % | −10.0 % | −5.3 % |
| D, twins merged | +51.9 % | +42.3 % | +20.0 % | +38.6 % | +61.6 % | +40.4 % | +25.0 % | −0.3 % |
| B on 677ddcb (base +85.7 % / +103.9 % proposals) | **+112.5 %** | +45.8 % | +20.0 % | **+95.7 %** | +36.8 % | +6.9 % | +36.4 % | +5.4 % |

Bold marks an aggregate out of its tolerance.

**What each does to the tier's own politics** (mean a run over the ten runs with the tier;
proposals then enacted):

| | Fixture 1 | Fixture 2 |
|---|---|---|
| A, base | 2.7, 2.6 | 7.4, 5.7 |
| B | 2.4, 2.4 | 7.4, 6.1 |
| C | 3.1, 3.1 | 3.0, 2.7 |
| D | 2.7, 2.6 | 7.3, 5.7 |

### A. Keep the present choice

Everything stays calibrated. The option-count dependence stays. Every template added to
content raises how often its kind is proposed. Cases waiting before a body that cannot decide
bring a proposal most weeks.

### B. Two stages: whether, then what

Propose with the chance the **best** move has against nothing, by the same softmax and
temperature. Then choose among the moves as before.

- Duplicates no longer raise the chance of proposing.
- It does not stop a move worth less than nothing being proposed. It only stops the count of
  such moves from mattering.
- The gate fails: fixture 1's moved goes to +96.9 % against 95 %, as the tier side's proposals
  fall (2.7 to 2.4 a run) more than every adult's. Fixture 2's proposals gap narrows from +52.7 %
  to +24.3 %.
- On 677ddcb it failed twice (fixture 1 proposals +112.5 %, moved +95.7 %). With the forum fix
  on top there, stream 5 fell from 16 proposals to 9.

### C. Acceptable moves only

Sample only the moves that score above doing nothing, as the daily choice does. With none above
nothing, propose nothing. The temperature stays 0.5; it is not taken from the spread, so no
weight changes.

- A move worth less than not proposing is never proposed, and duplicates only share a draw that
  is already decided to happen.
- The gate passes, and the tier and every adult come together: every difference lies between
  −10 % and 0 % except fixture 1's offices (−70.0 %, 0.9 against 0.2 a run) and moved (−48.8 %).
  That is what the gate checks (ADR-0014 §4: the tier is an approximation of every adult
  deliberating weekly, checked by comparing these aggregates). It also explains the present
  gap: under A, every extra deliberation is another chance to propose a move worth less than
  nothing, so every adult proposes far more than the tier (+52.7 % in fixture 2).
- **But politics shrink.** Fixture 2's tier proposes 3.0 a run where it proposed 7.4, and enacts
  2.7 where it enacted 5.7. In fixture 1 every proposal passes (3.1 and 3.1). With
  `propose_cost` at 0.5, only moves clearly worth it are proposed, so contested proposals and
  defeats nearly vanish. A world where nothing is turned down is not plausible: research 09-05 §4
  asks that "proposals that never reach enactment" be part of what is measured.
- The calibrated tolerances under C (fixture 1's proposals 123.2 %, fixture 2's 113.5 %) exceed
  the present 105 %. The present tolerances do not fit it.

### D. Merge twins

Before the draw, drop a move whose template kind, nominee and score equal an earlier move's.

- It treats only the duplicate symptom: six different forum templates below nothing are still
  proposed most weeks.
- The gate passes and is nearly neutral: fixture 1 unchanged, fixture 2's proposals +52.7 % to
  +61.6 % and moved −9.5 % to −0.3 %. The tier's own politics are unchanged (7.3 and 5.7 a run).
- It needs no recalibration.

## Recalibration any of B to D needs

- The notables' gate's tolerances (plan §9, of 2026-10-08 and since) were set at three standard
  errors of the present choice's runs. Under a new choice they must be recalibrated
  (`civ-host notables --calibrate`, ten runs with the tier from each fixture) and the
  tolerances rewritten, as the plan's calibration entries did.
- `propose_cost` and `temperature` were tuned against the present choice. Under B or C, a
  proposal happens only when something is worth more than doing nothing. A `propose_cost` of 0.5
  then rules out most moves, and it would want re-tuning against the smoke worlds' politics
  (whether villages still amend, seat councils and name hearers at plausible rates; ADR-0024's
  context: no village of about 50 amended its custom at 92–100 % present).
- A homelab long run of both fixtures and the five long-run worlds, before and after, to read
  proposals, enactments and offices over decades rather than three years.

## Recommendation

**C, after re-tuning, with D as an optional first step.**

- **C is the grounded choice.** It is the rule the engine already uses for daily activities
  (research 01-09 §4.3, plan §9 of 2026-10-03), it is the only option that removes the
  option-count dependence and not just its duplicate form, and it brings the tier and every adult
  together. It cannot be accepted as measured, though: with `propose_cost` and `temperature`
  tuned for the present choice it removes most contested proposals. It should be accepted only
  with a re-tuning pass in the same slice:
  - lower `propose_cost`, reading the smoke worlds and both fixtures until proposals, defeats and
    enactments are plausible (some proposals never reaching enactment, 09-05 §4);
  - then recalibrate the gate's tolerances;
  - then a homelab long run.
- **B is not recommended.** It fails the gate, and it still proposes moves worth less than
  nothing.
- **D can land on its own now if the owner wants the twin symptom gone before that work.** It is
  near-neutral and needs no recalibration, but it leaves the real dependence in place. It is
  subsumed by C.

## Consequences

- **If accepted:** the deliberator samples as the engine's other choices do. The number of
  templates in content stops being a lever on how often people propose. The gate is recalibrated
  once, and the tuning values are re-read against the smoke worlds.
- **If rejected:** recorded as a known property of the deliberator. Content authors should know
  that twin templates and long template lists raise proposal rates.

## Alternatives considered

- **A nested logit over kinds** (a nest per policy kind, its inclusive value against nothing).
  It is the textbook answer to duplicates, but it needs a nest parameter, a new weight, and
  templates of one kind are not the only duplicates.
- **Counting a failed sitting in the sponsor's expected success** (the diagnosis's F3). It is
  real but inert here, where gains are about 0.002 and the score is about −`propose_cost`.

## Revisit when

- A content pack adds many templates of one kind.
- A model deliberator (ADR-0039) is in use: its choices come from the same move list and are
  not drawn from this softmax.
