# Proposal (status: **Proposed**, not decided): wear while a sewer is still being laid

Backlog C's "Not yet" list (plan §7, M11f) leaves open "wear while a sewer is still being laid".
This brief sets out what would be needed and recommends a narrow first step. Nothing is
implemented and nothing is decided.

## What exists now (claude/claude-development 48cd61b4)

- `sewers_month` (`population/sewers.rs`) wears only sewers that are `whole()`: each route cell
  loses `rain × wetness / life_years / 12` of its soundness a month (ADR-0044 §5).
- Laying (`sewer_work`) advances a single `laid_m`, in metres, up to `length_m`. The record does
  **not** say which cells the laid metres are. The sewer carries nothing until it is whole.
- The review weighs and asks mending only for whole sewers in force (`s.whole() && in_force`).

## Why this is underspecified

To wear a part-laid sewer the kernel must know which cells are laid, and that depends on the
direction of laying, which the code and research do not state. Research 12-02 §1.5 gives a sewer's
processes (capacity, slope, blockage, leakage, maintenance, outlet), but not a construction order.
I make no historical claim about digging from the outfall or from the head.

## Options

A. **Do nothing new (recommended for now).** A part-laid sewer carries no flow, so its walls hold
   nothing back and a collapse would change no service. Its only effect would be a cell that is
   already worn on the day the sewer becomes whole. In the dashboard's worlds sewers take weeks to
   months to lay, against a life of decades, so the error is a fraction of a percent of life.
   Close the backlog line with this reasoning, recorded in §9.

B. **Wear the laid length, by a stated order.** Add a content field `lay_from` (`head` or
   `outfall`) to the sewer definition, defaulting to `head`, which is the order the route is held
   in. Cells are laid while the cumulative metres up to them (`cell_metres`) are within `laid_m`,
   and `sewers_month` wears those cells. A collapse before the sewer is whole sets
   `broken`. Since the review ignores sewers that are not whole, add a rule that laying work on a
   collapsed laid cell relines it first, at relining's hours. This is a save-visible behaviour
   change for any world laying a sewer, a content API bump, and two new tests.

## Acceptance examples (if B is chosen)

- A sewer half laid from its head, through one wet month: cells within the laid half lose
  `rain × wetness / life / 12`, and the rest lose nothing.
- A laid cell reaching `loss = 1` before the sewer is whole sets `broken`. The next laying hours
  reline it before any new metre is laid.
- Once whole, wear continues as now, and the review takes over mending.
- A world without sewers is unchanged. Saves round-trip exactly.

## Recommendation and uncertainty

Recommend A: a recorded "not built, and why" in plan §9, with no code. The owner may prefer B if
long laying times (a slow polity with few hands) turn out to matter in lived worlds. That would
show as sewers becoming whole with cells already near collapse, which the sewer's observer page
would reveal. Neither option touches thresholds or tuning.
