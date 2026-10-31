# ADR-0043: What is left of a source, and where people have looked

Status: Accepted (2026-10-31, by Claude under the owner's standing authority)
Date: 2026-10-31
Milestone: M11 (M11d's demo questions; plan §7, M11d's ninth and tenth parts)
Amends: M5a's memory of places (`KnownPatch`) for stocks that do not renew; ADR-0010 §1's
prospecting (M11d's third part; plan §9, "Design priors for prospecting")

## Context

Two faults of memory showed in the M11 demo world (plan §7, M11d's ninth and tenth parts):

1. **A stock that only falls is believed rich after it is emptied.** A household remembers a place
   by the mean return of every hour worked there (`KnownPatch`), the evidence fading over the
   resource's renewal time, ten years at most. Bog ore is a kilogram a unit and never regrows;
   Thorndale's wanter emptied its rich patch (0.6 units left) and still believed it gave 1.28
   units an hour, its value (1.06) second among its options. In a scene, a household scraped a
   patch empty and walked back to it for almost nothing on three days.
2. **Prospecting remembers nothing of where people have looked.** A household short of a dug
   material walks a loop over ground of the right slope, worth a flat one load (`PROSPECT_HOPE`),
   drawn afresh daily. Over the year after teaching, Thorndale's wanter walked 4,030 person-hours
   and Stonefield's 13,320 looking for copper that lies beyond their reach, and found nothing. The
   prior's own revisit condition is met.

Research 03-08 (written for this): a forager leaves a patch when its marginal return falls to
what elsewhere gives (Charnov's marginal value theorem), so what counts is the patch's return now;
how much old experience counts depends on how fast the world changes (McNamara and Houston 1987),
and for a stock that only falls, every earlier sight is of a richer place, so the last sight of
what was left is all worth keeping. A search finds a showing body with a probability that rises
with the ground swept (Koopman); for rare bodies, a walk's hope is about proportional to the fresh
ground it covers, the share not yet looked over when walks are drawn over the whole, and nil when
all is covered. Prospectors looked over ground, tried it, and moved on. Research 03-05 §1.3, 03-06
§1.2 and §5.3 and 08-01 §1 were read with it.

## Decision

### 1. A stock that does not renew is judged by what was left

For a land resource that does not renew (`Growth::Deposit`: stone, flint, copper ore and bog ore
in the core content), each visit to a place replaces what was remembered of it with the return
that as many hours more there would give a capable adult as the place was left (computed on the
land without taking anything, `Land::yield_around`), over the hours of that visit
(`KnownPatch::saw_left`). The memory never fades back to what land of its kind is expected to
give: a worked-out place stays worked out, until someone works it again. The belief that weighs
it against the walk is unchanged: what was seen against the expectation of land of its kind, the
expectation counting as one unit's worth of evidence. Settlements share it as before. Resources
that renew keep the mean of hours fading over their renewal time.

### 2. A settlement remembers the ground it has looked over

A settlement (a household of no settlement: its own) keeps the blocks of ground its prospectors'
walks have crossed, with the day each was first looked over. Blocks are squares of twice
`FIND_M` (100 m) on a side, by the map's cells. Only prospecting walks (a dig activity walked with
no deposit as its target) are counted: an errand's way is the same few trails, already passed
many times. Nothing fades: a showing body does not appear later, and one an earthwork uncovers is
found by the earthwork (ADR-0010 §1).

### 3. A prospecting walk's hope falls with the ground covered

A prospecting walk is worth `PROSPECT_HOPE` (unchanged, a logged nudge) times the share of the
reachable ground of the right kind (land within the digging activity's walk, on the slope the
land profile lays such bodies on) not yet looked over, measured by blocks. The day's draws keep
only places in blocks not yet looked over; the walk goes out to the first of them and across to
the farthest. With no such place drawn, or nothing of the right kind left unlooked, nobody
prospects for it. Word of mouth between settlements is not added: a find already becomes known by
walking past it, and barren ground is a settlement's own knowledge.

### 4. The boundary

The ground looked over is saved with the deposits section (appended within the unreleased saves
220); a save without it loads with nothing looked over. A `KnownPatch` saved before this change
keeps its meaning until the place is next worked. Nothing new rides the wire; no content changes.

## Consequences

- A household leaves a patch of ore when what is left no longer repays the walk, and does not walk
  back to it for nothing; it goes to fresh ground sooner. A household with nothing else in reach
  gets less ore from a patch than when it believed the patch richer, because it no longer works
  the patch to the last unit.
- Prospecting for a body that lies beyond reach ends once the reachable ground is covered: weeks of
  walking, not years. A body within reach is still found by walking past it.
- Stone and flint, also stocks that never renew, are remembered the same way; with stocks of
  thousands of units a visit changes little, so their places look as before.

## Alternatives considered

- **An exponentially weighted mean with a short memory.** With a stock that falls monotonically,
  any weight on older sights biases the belief upward; the last sight of what was left is the
  limit of such means and needs no learning rate.
- **The visit's own return.** A visit that empties a patch can have the highest return of all; it
  is what is left that the next visit meets.
- **Counting resources that renew slowly** (deadwood, about 14 years). Its stock is hundreds of
  times a trip's take and its depletion shows in its existing memory; left until a demo shows a
  household walking back to emptied deadwood.
- **Ground looked over on every walk.** Truer to how bodies are found, but it adds work to every
  walk and state to every world for a decision only prospecting reads.
- **A probability map of bodies** (geology-informed priors, Bayesian cells). Belongs with geology
  (ADR-0010's revisit); the flat hope over a share is the smallest rule that ends a search.
- **Per household memory of ground.** A village's prospectors would each search the same ground;
  the hearth shares it, as known places and deposits are shared.

## Revisit when

- Geology arrives (ADR-0010): bodies' likelihood by rock, and a prior per block.
- A demo shows settlements that never look again after their reach grows (new trails extend it;
  the share counts ground reachable today, so new ground is fresh).
- Organised prospecting (parties, paid prospectors, campaigns; 03-05 §5) is designed.
