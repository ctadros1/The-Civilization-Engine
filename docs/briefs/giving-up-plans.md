# Brief: where a household that gives up goes

Status: **proposed, for the owner's review before any build** (2026-11-24). Plan §8 question 24,
item 1. It builds out an aim the M5 settlements brief set (`m5-settlements.md`: "Founding is one
plan beside staying, joining a known settlement and leaving the map") for households that give up,
where today a fixed order decides.

The owner's direction (2026-11-24): households that give up should weigh viable alternatives
rather than vanish; relocation to an established settlement, founding where it is realistically
feasible, and leaving the map when no viable internal alternative exists, each arising from the
household's circumstances, the land, what it knows, what it can reach and carry, and its prospects
of surviving, not from fixed probabilities or a fixed order of outcomes.

## 1. What happens now, and why it falls short

Each day a household out of food, worn down and with no crop coming weighs leaving: it goes with a
chance set by how far its food, what others could spare and any common store fall short of the
wait to its next harvest, against what its fields should bring (`leave_chance`, `life.rs`
`departures`). Where it goes is then a fixed order:

1. the known lived-in settlement whose *pull* is highest, if any pull is above nothing
   (`refuge`, `moving.rs`): `w_kin·kin + w_ties·sat(known) + w_fed·(food seen − fed at home)`,
   a place last seen no better fed than home excluded;
2. failing that, where others giving up the same night just went (chain migration);
3. failing that, off the map.

Departure diagnostic run 112 (world 2 alone, 9cf0512) shows the cost: all nine households that left
Thornbank knew Sedgestead was lived in, but only by word or a sight, with no food impression, no
kin and nobody known there. Its pull was exactly 0.00, so they left the map. The other 11
households that gave up, leaving new settlements, went home to kin, as designed. Founding is never
weighed by a household that gives up, and "off the map" is a default rather than a choice.

A second sample on current code (homelab run 124: the same diagnostic on verification 38ff89f,
after founding's vacant fields, help with a first home and the exact hire cut) shows the same
pattern, earlier in the world's life. In the 16th year (days 5985 to 6011), 12 households gave up,
all of them in Thornbank, and all 12 left the map. The first 3 knew no other place. The other 9 had
learned of Sedgestead by sight or word on day 5991. They knew it was lived in, but nobody had seen
its food, and none had kin or anyone known there. It looked better than home, where the fed share
was 0.04 to 0.44, yet its pull was 0.00, so it was refused as a refuge. Thornbank kept 3 or 4
people from then on, while Sedgestead fed about 80 for the 35 years after (its median food days
276 to 729). So it had room for at least some of them. Under this design the 3 who knew nowhere
would still leave the map; the 9 would weigh Sedgestead.

## 2. What the research asks (05-06, migration; 10-01, site selection)

- **Compare feasible plans, "stay" among them** (05-06 §5.1: consider → compare feasible plans →
  prepare → travel). A plan's value is the change it brings in expected consumption, safety and
  support, less switching costs: `V = β_c ΔE[consumption] + β_s Δsafety + β_k Δsupport − β_m
  costs`. The travel itself belongs in the budget, not charged again as a bare distance penalty.
- **Famine migration answers expected access to food**, not hunger alone, and moves in stages
  through a nearby refuge (05-06 §1.4). Starvation can also make travel impossible.
- **Willingness differs from ability** (05-06 §1.2): severe deprivation raises the wish to leave
  while lowering the means. A plan must be *feasible* before it is valued.
- **Networks help but are not needed** (05-06 §1.2): kin lower a move's costs and uncertainty, but
  migration without relatives happens (London apprentices 1600–1749).
- **Emergencies screen for survival first** (05-06 §5.1): under immediate danger, first ask whether
  a plan plausibly gets the household through, then compare the rest.
- **No world truth** (05-06 §5.1; ADR-0018): use remembered or reported conditions; a place nobody
  has seen or heard of cannot be chosen.
- **Destination feedback, not caps** (05-06 §5.3): arrivals meet the actual land, housing and
  people there; shortage shows as crowding, hunger or onward movement, never a population cap.
- **Founding is a project with resources** (05-06 §5.2; 10-01 §1.5): a founding party needs a site
  it knows, seed, and food to its first harvest; it can fail and return.

## 3. Proposed design

### 3.1 One comparison, at the moment of giving up

The trigger stays as it is: `leave_chance` decides *that* the household can no longer stay on its
current footing, a hazard over the day as 05-06 §5.1 recommends. What changes is *where*: instead
of the fixed order, the household compares every plan it can carry out, by one value, and takes the
best (the same bounded choice, with its temperature, that `decide::choose` makes, so that near-equal plans
split between households rather than all following one).

### 3.2 The plans

For each, first a **feasibility screen**, then a **value**. Every term uses the household's own
knowledge (members' food impressions, kin and acquaintances, places known and how) and quantities
the world already keeps.

1. **Stay and endure.** Always feasible. Its value is what staying offers: food, others' spare and
   the common store against the wait to harvest (the `reach` that set off the hazard), plus the
   stake in its fields. A household too weak to reach anywhere stays, which is what the research
   expects of the starving.
2. **Go to a known lived-in settlement.**
   - *Feasible* if the walk there (`walk_hours`) can be made on the food it carries plus what its
     members' reserves allow before collapse. Distance therefore limits refuge through the
     household's own means, not through a fixed radius.
   - *Value*: expected food access there against at home, support (kin, then those known),
     safety (raids heard of),
     land or work to be had there as the household knows it, less the move's real costs: the
     journey's food, and the home and fields abandoned (stake).
   - *The key change*, for a place known only by word: expected food there is not taken as "no
     better than home". It is the household's **prior for a lived-in place**: the mean of what its
     members have seen of the places they know, falling back to the content's prior if they have
     seen none. A place is lived in because it mostly feeds its people, so a household starving at
     home expects better of a living village it has never seen, but less than of one it has seen
     fed, and nothing of one it has seen hungry.
   - *A stranger is fed there as a neighbour is*: giving is need-based (research 08-11 §5.4).
     Any household with food to spare gives to one that asks (`give_food`), refusing only one it
     believes stole from it or from those it regards (`refuses`). Regard only makes an asker walk
     further to someone it knows (`best_giver`, within the content's `ties.ask_known_min` minutes). So the support term needs no
     separate stranger's penalty: a newcomer without ties loses only kin's and acquaintances'
     pull, and its prospects there are the place's food, which the prior estimates.
3. **Found a settlement.**
   - *Feasible* only with a site the household knows (`founding_sites`), seed for the site's
     need, and food, carried or to be had, to its first harvest; or by joining a coalition already
     gathering that it is asked into.
   - *Value*: the existing founding terms (`found_parts`): the site's cropland against home's,
     kin and ties going against staying, and the cost of founding.
   - A household that gives up for hunger will seldom pass the screen. That is the research's
     expectation, and it emerges from what the household holds rather than from a rule.
4. **Leave the map.**
   - *Feasible* if the map's edge can be reached on its food and reserves, judged by the same
     walk test as a settlement.
   - *Value*: what the rest of the world offers a stranger: the same lived-in prior, with no kin or
     acquaintances, less the walk to the edge and the loss of everything held here.
   - So it wins only when no reachable internal plan is worth more: when every known place is
     known hungry, out of reach or unknown, or the household knows of nowhere. That is the owner's
     "when viable internal alternatives do not exist", arrived at by comparison rather than placed
     last by rule.

Chain migration is kept as it is. Those who just left for a place tell those they know, so the
second household has a fresher impression and kin or ties there; it is the same comparison, with
better information.

### 3.3 What stays the same

- The giving-up hazard and its inputs.
- `move_worth` for ordinary moving, which is untouched, though it shares the new prior helper.
- Relocation, which already puts a household in the destination's regime, where the land review
  gives it fields.
- The off-map account (ADR-0018 §3: people conserved, the reservoir explicit).
- Saves: no new saved state. Plans are worked out when a household gives up and are not kept. A
  persistent journey state (05-06 §5.3) is left for a later slice.

## 4. Implementation dependencies

What exists:

| Piece | Where | Use |
|---|---|---|
| Known places and food impressions | the household's `Places` (`places.rs`), `KnownPlace.food` | knowledge, the prior's sample |
| Kin and acquaintances by place | `draws` (`moving.rs`) | support term |
| Walk time between settlements (hearth to hearth in a straight line, at the off-trail pace) | `walk_hours` (`moving.rs`) | feasibility, cost |
| Founding sites and their worth | `founding_sites`, `found_parts` (`founding.rs`) | the founding plan |
| Raids heard of | `raids_believed` | safety term |
| Relocation, leaving the map | `relocate`, `leave` | carrying out the choice |
| Bounded choice with temperature | `decide::choose` | taking a plan |

What is new:

- A walk estimate from home to the map's nearest edge, measured as `walk_hours` measures a walk to
  a settlement: a straight line at the off-trail pace. It is cheap, so nothing needs caching.
  *A known limit, shared with moving today:* a straight line ignores water that no walk crosses
  and the slopes on the way, so it reads some walks as shorter than they are, and feasibility is
  optimistic there. Tracing the walk over the navigation grid would fix that for every move, not
  only these; it is left for a later slice unless the scenes or the departure diagnostic show
  households setting out for places they cannot reach.
- The feasibility test: food carried and reserves against the walk's days.
- The lived-in prior.
- One `giving_up_plan` function replacing `refuge`'s fixed order.
- New content parameters, each documented as tuning and logged as a NUDGE:
  - the prior's fallback (the fed share expected of a lived-in place seen by nobody);
  - the reserve days a worn household can walk on.

No wire or schema change. The observer's chronicle already tells "moved to" and "gave up and
left". A word for "set out to found" would come with the founding plan.

**ADR:** an amendment to ADR-0018 §3 and §5 ("a household that gives up compares feasible
plans"), proposed with the first slice, for the owner's acceptance.

## 5. Validation

Scenes, in the style of `civ-sim/tests/settlements.rs`; each must fail without the change:

1. A household giving up that knows a lived-in village within reach only by word goes there, not
   off the map (the run 112 case).
2. A household whose only known places are known hungry, or out of reach on its food, leaves the
   map.
3. A household too weak to make any walk stays.
4. A household with seed, food to a harvest and a known site founds when no village draws it more,
   and without the seed it does not.
5. Kin still draw first where they would before: the existing refuge and chain-migration tests
   unchanged.
6. Destination feedback: a village receiving many such households feeds them, or its own
   shortage shows (hunger, onward movement) with no cap. People are conserved throughout
   (`residence_problems`, `problems` empty).

Lived worlds:

- The departure diagnostic (run 112's world-2-alone set-up, DEPART lines) on the change. The
  expected result is that Thornbank's leavers go to Sedgestead or found, and few leave the map. Then
  read Sedgestead's fate: does it carry them, or does crowding push some on?
- A long run: smoke, Gate B, the notables' gate, and the dashboard's Moves, Population and
  Population accounting rows. A rise in internal moves and a fall in off-map departures are
  expected and to be reported as a behaviour change, with acceptance criteria unchanged.
- Stylized facts from 05-06 §4, checked from the moves recorded: distance decay (most refuges
  near), corridors that follow earlier moves, and no single settlement absorbing every leaver.
- Time-boxed, plausible-not-proven: the priors are tuning, logged as NUDGE entries with the scene
  figures that set them.

## 6. Slices

1. The comparison with stay, known settlement and off-map plans, the prior and the feasibility
   test; scenes 1–3, 5 and 6; the departure diagnostic; ADR-0018 amendment proposed.
2. The founding plan for households that give up; scene 4.
3. (Later, if wanted) a journey state for long refuges (05-06 §5.3), a save change, with its own
   ADR.

## 7. For the owner

- Accept the design, or change it, before slice 1 is built.
- Two choices inside it:
  - the bounded choice (near-equal plans split) against taking the best outright;
  - whether "stay and endure" should be able to win once the hazard has fired, or only when no
    plan is feasible.

  I would take the bounded choice, and let staying win only when nothing is feasible, since the
  hazard has already weighed staying.
