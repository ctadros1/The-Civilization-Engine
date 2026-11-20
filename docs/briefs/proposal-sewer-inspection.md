# Proposal (status: **Proposed**, not decided): looking over a sewer

Backlog C's "Not yet" list (plan §7, M11f) leaves open "inspection (12-08 §5.6: faults are seen
only as they show)". This brief proposes inspection on an existing pattern in the code. Nothing is
implemented and nothing is decided.

## What exists now (claude/claude-development 48cd61b4)

- **Sewers** (ADR-0044 §5; `population/sewers.rs`): each route cell wears monthly. The loss is kept
  in `Sewer::loss`, and a cell worn through collapses. The polity learns of a fault only as it
  shows, as a collapse or as a home backing up for lack of room it once had. The review then weighs
  relining or clearing and asks the work. A worn but standing cell is never seen, so it is never
  relined before it fails.
- **Precedent: banks** (M6b slice BG; `population/levees.rs` `look_over_banks`, called from
  `population/floods.rs`): banks that a law keeps are "looked over when a high water goes down and
  at each year's start (research 11-12 §5.5: event-driven maintenance plus periodic inspection)".
  A bank that wants mending has the work asked of households in equal shares, and this is told.
- **Research:** 12-08 §1.4 says to give recurring upkeep "its own workers, accounts, inspections".
  §5.6 suggests "one scheduled inspection per major maintenance season, plus inspections after
  damaging events", labelled a **design assumption**, and an information experiment comparing
  timely inspection with information delayed by a maintenance cycle. USACE's annual routine
  inspection (12-08 §2) is a modern benchmark, not a universal interval.

## Proposed design

1. **When:** at each year's start, and after a storm or flood episode that reached the sewer's
   cells. These are the bank precedent's two occasions, and §5.6's design assumption.
2. **Who and what it costs:** inspection is asked as sewer work is asked, an equal share of each
   household under the law that laid the sewer, paid where a law pays for common work. Its hours
   are walking the route at the population's walking pace, plus `inspect_h_m` hours a metre. That
   is a content prior in the sewer's content definition, documented as a design assumption with no
   research figure. No inspection is done unless the hours are given; an unworked inspection
   leaves the polity knowing what it knew.
3. **What it reveals:** each cell's current soundness. Today the review sees only collapsed cells.
   After an inspection it also sees the cells it was told about, with their wear as of that
   inspection. Between inspections that knowledge goes stale, which is what makes §5.6's "delayed
   information" emerge without a delay setting.
4. **What the review does with it:** for a known worn cell, it weighs relining now against
   waiting. Waiting costs the hours the homes above it would lose while backed up, which is
   `sewer_spared_h` for those homes, over the expected days from collapse until it is relined
   again. Relining now costs laying's hours and materials a metre, exactly as relining a collapse
   costs. The expected days until collapse come from the cell's own monthly wear so far. No new
   threshold: the review already compares these quantities for collapses.
5. **Told and shown:** "the sewer was looked over: 3 cells worn past half" is entered in the
   chronicle only when something is found. The sewer's page shows the date it was last looked
   over and the cells known worn.

## Acceptance examples (tests to write)

- **No law, no inspection:** a sewer whose law has ended is never looked over, as now.
- **Year's start:** a sewer under its law has inspection hours asked on day 0 of the year, and none
  on other days unless a storm reached it.
- **Reveals wear:** a cell at `loss = 0.6`, unseen before, is known at 0.6 after the inspection
  hours are given. If its loss then rises to 0.7, it is still known at 0.6 until the next
  inspection.
- **Pre-emptive relining:** a known worn cell with 4 homes above it, each spared 20 h a year, and a
  short expected time to collapse, has relining asked. With 1 home spared 1 h a year it does not.
- **Unworked inspection:** if nobody gives the hours, nothing is learned and nothing is asked
  beyond today's rules.
- **No sewer, no change:** worlds without sewers are unchanged apart from length-only effects.
  Saves round-trip exactly, including the known wear and the last date looked over.

## Bounded implementation plan

Four commits:
1. `inspect_h_m` in the sewer definition (strict TOML, range, content API bump), and the known
   wear fields saved on `Sewer`.
2. The occasions and the asking, following `look_over_banks`.
3. The review weighing known worn cells.
4. The chronicle and observer words.

Estimate: one to two sessions. It touches `polity.rs` (SewerDef, Sewer), `population/sewers.rs`,
`population/floods.rs` (the event hook), `civ-content` (one field) and `persist/agents.rs`.

## Uncertainties for the owner

- The `inspect_h_m` prior: there is no research figure. A first value might be half of
  clearing's per-metre hours, stated as an assumption. The owner may prefer a fixed walk-only
  inspection with no per-metre work.
- Whether expected time to collapse should use the cell's own observed wear rate (proposed) or
  the content's `life_years`.
- Whether a health officer or other office should do inspections instead of households in equal
  shares. 12-08 §1.4 argues for "its own workers", but the code's office for upkeep does not exist
  yet. The proposal keeps the equal-share pattern the sewer already uses.
