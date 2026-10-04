# ADR-0009: Building components: grammar v2, condition and trust

Status: Accepted
Date: 2026-10-04
Milestone: M3b
Amended: 2026-10-04, to match what was built. Slice O's first step: §2's parameters as frame
version 1 names them, §3's group kinds and storage, §8's schema 15 and wire 1.14. Second step:
§1's repertoire and choice, §5's room under a roof. Slice P: §4's condition and upkeep, §5's
loads and failure (with the storm's day), §6's skill and caution, §8's schemas 17 and 18.

## Context

M1 has one building program, the round hut (ADR-0004 §2), and a household builds one home.
A finished building never changes: it has no condition, nothing wears it, and nothing can fail.
M3b needs grammar v2 (houses, workshops and storehouses of one or two storeys) and structural
rules v0: storeys against wall material, builder skill, foundations on a slope, age and upkeep,
partial failures and collapses in the chronicle, and technique trust per culture (plan §5.1,
§7). ADR-0004 asks to be revisited "when a second building program arrives". Saved worlds will
keep what was built, how it has aged and what failed, and Unreal will key its building kit and
weathering on the same parts.

Reports read: 11-01 (vernacular typology), 11-03 (shape grammars), 11-04 (construction),
11-05 (structural engineering history), 11-06 (failures), 11-07 (bridges), 11-08 (materials),
11-09 (codes), 11-13 (interiors), 10-06 (housing typologies), 14-06 (weathering), with 08-02
and 08-11 on storage. Their main points:

- **Plans:** circular and rectangular dwellings are both common (42.9 % and 48.6 % of coded
  societies, 11-01 §5.1), with no obligatory transition between them (11-01 §4.2). Timber
  houses grow by bays at constant width; roundhouses grow by another structure (10-06 §1.2,
  §3.1). Early-farming longhouses reach 20–25 × 7 m (10-06 §2.1).
- **Floor demand** is a0 + bN + production + status, with a0 = 6 m² and b = 4.8 m² a resident
  as the central prior (10-06 §2.3). Wealth buys reception and storage space and more buildings,
  not only sleeping room (10-06 §1.2; 11-13 §1.3).
- **Vertical organisation** is its own dimension (one floor, loft, storeys; 10-06 §5.2), and a
  storage platform is not a storey (11-01 §2.4). Priors: structural bays 2–6 m, storeys
  1.8–3.5 m (11-01 §3.4); 1–2 storeys in earth and 1–3 in timber (11-05 §2.2).
- **Structural systems, not material caps:** check bending, deflection and connections for
  beams, eccentricity and slenderness for earth walls. Doubling a span gives four times the
  bending stress and sixteen times the deflection. A metre of stored wheat is about 7.6 kPa,
  several times a dwelling floor's 1.5–2 kPa, so a change of use is more dangerous than
  occupancy (11-05 §1.2, §2.4, §4). Builders design with an estimated reserve, which differs
  from the physical reserve and from serviceability (11-05 §5.1).
- **A coarse structural graph, not a health bar:** 8–32 component groups, each with capacity,
  demand, quality and deterioration. Defects are sampled once and kept, never rerolled
  (11-06 §5.1; 11-05 §1.6).
- **Deterioration comes from exposure, not calendar age:** wet timber decays (decay is
  possible above about 30 % moisture, not below about 20 %, 11-06 §2.2), leaking roofs expose
  protected members, and earth erodes when wet (11-05 §1.6; 11-06 §1.2; 11-08 §2.4). Losing a
  tenth of a member's diameter leaves 73 % of its bending capacity (11-07 §5.5).
- **Failures:** record vulnerability, trigger and propagation separately (11-06 §1.1). Damage is
  local, with conditional cascades. Cracks are not collapse, and a distressed building can stay
  occupied (11-05 §4; 11-06 §1.3). Casualties come from the people actually present
  (11-06 §5.3). There is no defensible failure rate for premodern housing (11-06 §2.1, §2.4).
- **Upkeep** is continuing production, not rejuvenation. Rethatching, replastering and
  replacing a post change different variables, and act only when labour and materials arrive
  (11-04 §4; 11-06 §5.7; 14-06 §4.1).
- **Learning from failure** concerns specific combinations of span, material and load
  (11-06 §5.6). Builders respond with stronger supports or reduced ambition (11-06 §3.4). Memory
  of a disaster fades with a half-life of 2–20 years, a low-confidence prior (11-09 §2.3).
- **Storage:** maintained dry stores lose 2–8 % of grain in 6–12 months, poor ones 10–20 %
  (08-02 §7.1). Early farmers built raised, ventilated granaries (08-11 §2.1).

## Decision

### 1. Grammars and programs

- The **hut** grammar stays frozen at version 1. Its expansion, golden hash and saved specs are
  unchanged.
- A second grammar, **frame** (version 1), builds rectilinear post-framed buildings:
  - a length of equal bays at a constant width, with a pitched roof;
  - one or two storeys;
  - a loft (a floor at tie-beam height inside the roof, holding goods only) over chosen bays;
  - optionally a floor raised on posts.
- A program names its grammar, its use (house, storehouse or workshop), the use of each level,
  its material slots, its rules and the technique it needs (ADR-0008). The people profile lists
  the programs its households may build.
- `expand` dispatches on the program's grammar and the spec's version, and refuses unknown
  versions. Expansions stay pure and golden-hashed per grammar version.
- As built (slice O's second step), a household's first home is the shape, among the programs
  it may build, that covers its members and its goods for the fewest hours; its means then buy
  the largest shape of that program they pay for. A program's frame buildings vary in bays,
  storeys and lofts, at sizes the program gives.

### 2. Specs

- `Footprint` gains `Rect { x, y, length, width, angle }`, in centimetres on a 25 cm quantum,
  with the angle in turn units (ADR-0004 §1).
- `params` grows from eight integers to sixteen. A hut v1 spec uses its first three, as before.
  Frame v1 names them: storey height, pitch, bays, door (side and bay), loft bays, joist
  section, eave overhang, raised-floor height, post section, wall kind, wall thickness, footing
  kind, and four style traits (§7).
  - As built, version 1 knows one wall kind (wattle and daub) and one footing (posts set in the
    ground), and has no style traits yet, so those parameters must be 0. A building without
    joists has a joist section of 0.
  - The door's side is one of the two long walls or the two ends, and an end's door has bay 0.
  - Lengths and widths lie on the 25 cm quantum, and the length divides into whole bays.
- A spec stays what its builder decided. It never changes; an alteration would make a new spec.
- Condition is building state (§4), not design. This supersedes ADR-0004 §2's "condition" in
  the spec.

### 3. What an expansion yields

Besides parts, outline, door and stage needs (the five stages and their saved indexes keep
their meanings), an expansion yields:

- **Spaces:** each level's bays, net area and use.
- **Derived facts:** net floor by use, sleeping places, storage capacity in kilograms by kind
  (raised store, loft, living floor) and places to work.
- **Component groups** with stable semantic ids built from level, bay, element and index, so
  that a later bay renumbers nothing (11-03 §4.8). Each group gives:
  - its kind: posts of a wall line, tie beams, loft or floor joists, roof frame, covering,
    infill, mass wall or floor;
  - its members' count, section, span or height, spacing and tributary area;
  - its material slot and the technique it needs.
- An expansion has 8–32 groups (11-06 §5.1). For the frozen hut, a separate pure function
  derives five groups (posts, roof frame, covering, infill, floor) from its v1 expansion,
  without touching its golden hash.
- As built (slice O's first step):
  - A frame's groups are each long wall's posts, a raised floor's middle posts, the tie beams at
    the wall heads and the cross beams under each floor, each lofted or floored bay's joists, each
    bay's roof frame, each slope's covering, each wall's infill (an end's with its gable) and a
    floor on the ground.
  - A group's material slot is given; the technique each needs is the program's until slice O
    names features' techniques.
  - Storage comes in three kinds, best first: on a raised store's floor, in a loft, and on other
    floors (a store room's at its own rate, living and working floor at a lower one).
  - Part ids are semantic too: level, bay or frame line, part kind and index.

### 4. Condition

- For each group, a building keeps:
  - its quality `q`, drawn once when its stage is finished, from the skill of those who built it
    (§6);
  - its loss `x`: the share of effective section lost, or of a covering worn;
  - a lean, for posts;
  - when it was installed and last repaired;
  - its state: sound, showing a symptom, or failed.
- For the whole building, it keeps a state (standing, damaged or ruin) and who built each stage.
- Capacity, demand and margins are derived from the expansion, the condition and the loads
  whenever they are needed, and never saved.
- Loss grows with exposure, at authored rates (tuning values), updated monthly:
  - posts with ground contact and site wetness;
  - covered members only under a leaking covering;
  - coverings with weather;
  - earth walls at the foot.
- Upkeep is work like building, aimed at the worst visible symptom. It renews a share of a
  group with that share of the group's labour and materials from the grammar, and changes only
  that group.
- As built (slice P's first step): each group's quality, loss, state, and when it was put in
  place and last mended; the building's state; no lean yet (it comes with the checks that read
  it), and no builders' names: the stage under way keeps its builders' skill hours instead.
  - Rates and thresholds are each program's `[upkeep]` table, per kind of group: coverings,
    posts (times the wetness of the building's habitat), infill, and roofed timber (times how
    much the roof over it leaks). Floors on the ground do not wear.
  - A roof leaks by how far its coverings have gone beyond the share at which they leak,
    averaged over them; fully once a covering or the roof frame has failed. Its room for goods
    shrinks by that share (§5's capacities, as its condition leaves them), and a failed loft's
    joists drop that loft's room.
  - Posts that fail leave a ruin, which shelters nothing and is taken down when its household's
    next building of the same use has its roof on; any other failure leaves it damaged.
  - Upkeep comes after the building under way and before anything new, for the group gone
    furthest beyond the share at which it shows. It renews all that group has lost, at the
    group's part of its stage's labour and materials (by member volume for timber, by area for
    coverings and infill), and is deferred while other work presses.

### 5. Loads, checks and failure

- **Loads:**
  - Roofed storage is a capacity. A household's storable goods fill its best roofed storage
    first (raised store, then loft, then living floor). What is left lies in the open and keeps
    as unsheltered goods do.
  - The allocation is derived at each daily review and whenever a roof goes on or a group
    fails, and never saved.
  - As built (slice O's second step), the goods that lose the most a day in the open per
    kilogram fill the room first, and the shares are worked out from the stores whenever they
    are settled; a loft's and a raised floor's room counts once the building is finished.
  - A loft's load is the kilograms allocated to it over its area. A floor's is its people plus
    its goods.
- **Margins:**
  - Each group has a margin R = capacity / demand for its mode: bending and deflection with the
    beam relations of 11-05 §5.2, buckling for posts, height over thickness for mass walls.
  - Capacity scales with (1 − x)^3 in bending and (1 − x)^4 in buckling.
  - Sustained loads add creep to deflection (11-05 §2.4).
- **Checks:**
  - Groups are checked monthly, and at once whenever their load changes.
  - Randomness comes only from the persistent `q`, from one shared peak load per settlement per
    month and from what people actually do. The peak load stands in for M3c's weather and will
    be replaced by it, never added to it.
- **Symptoms and failure:**
  - A group shows a symptom (sag, lean, leak, worn infill) below an authored margin, and fails
    below 1.
  - A failed loft drops its goods (a share spoiled); the house stays roofed.
  - A failed roof frame unroofs the building.
  - Failed posts or a failed load-bearing wall make it a ruin.
  - After any failure, the groups that carry the failed group's load are checked once more.
- **Occupants:** the people inside at that minute are at risk, with a new cause of death,
  *collapse*.
- **Chronicle:** a partial failure or collapse gives vulnerability, trigger and propagation in
  words, and links the building.
- As built (slice P's second step):
  - Margins are checked each day for every standing building. The month's peak is drawn once a
    month for each settlement with the day it comes (1 to 28), and weighs on roofs that day
    only, so each storm is evaluated once and a roof mended after it does not meet it again (at
    first it weighed every day of its month, which the step's smoke showed wrong). There is no
    separate check when loads change.
  - Deflection is serviceability: a sag past L/180 (11-05 §2.4) only shows, and a failure
    needs a strength margin below 1. The sag is checked on lofts, floors and beams only; a
    roof frame shows only its strain, as thatched rafters bow under their covering without
    harm (at first every hut's roof showed a sag). A sag or a lean is not mended by upkeep, which rebuilds
    a failed part whole.
  - Posts carry their share, by tributary area, of the whole building's weight, and fail by
    buckling (Euler's load halved for imperfection) or crushing. Mass walls and footings wait
    for the grammars that build them.
  - A failed loft spills its goods, 30 % lost; a failed tie beam at the wall heads brings its
    lofts down; failed rafters open the roof; failed posts leave a ruin. People inside die
    with a chance by what fell (tuning values; the plan's §9 NUDGE).
  - The chronicle entry names the building's household's eldest and those killed, and says
    in words what gave way, under what and why. It gives the place, not a link to the
    building, which may be taken down.

### 6. Builders and trust

- **Builder skill:** building stages train a new skill, building. The hours-weighted skill of a
  stage's workers sets the distribution its groups' `q` is drawn from: skill buys fit and
  consistency, not stronger wood (11-05 §1.3).
  - As built (slice P's first step): `q = 1 − σ·|z|`, z a normal draw keyed by the world, the
    building and the group, never below 0.1, with σ from the people profile's
    `quality_spread`, a novice's 0.30 (11-06 §2.2's dispersion) to a master's 0.10 (tuning).
    Mending trains the skill too, and skill does not change how fast anyone builds.
- **Trust:** each settlement keeps, for each building technique, two recency-weighted sums.
  Until cultures exist, the settlement stands in for its culture. The sums are:
  - failures seen, weighted by severity and deaths;
  - component-years in use.
- **Caution:** their ratio sets a caution factor. New designs size members to a margin times
  that caution, over the builder's estimate of capacity (with the household's planned loads),
  not the true capacity. So after a failure, joists are deeper, posts thicker and lofts fewer.
- **Recovery:** failures fade with an authored half-life inside 11-09 §2.3's 2–20 years while
  survival accumulates, so margins come back without a script.
- As built (slice P's third step):
  - The sums are failures (one a building that gave way, each death in it counting
    `death_weight` more) and building-years rather than component-years, the unit of
    11-06 §2.4's plausible band (1e-5 to 1e-3 a building-year). Both fade by half over
    `half_life_years` and are counted against at least 10 building-years.
  - Caution is `1 + (most − 1)·rate/(rate + half_rate)`, the people profile's
    `[build.caution]` (content API 17). It sizes members directly rather than through the
    builder's estimate of capacity: a frame design's joist diameter times caution's cube root
    and post diameter times its fourth root, rounded and kept within the program's ranges, so
    a joist or a post is about `caution` times as strong. Fewer lofts wait: a household still
    chooses its shape by the usual design's cost and pays for the stronger one as it builds.
  - Huts are built as before, their grammar frozen. A household of no settlement remembers
    nothing.

### 7. Other uses, plots and style

- A building has a plot of its own, with a use: dwelling, store or work.
- As built (slice O's third step), a household builds one thing at a time, from the programs
  the people profile lists: what it has under way and someone in it can work on, else its first
  home, else a store beside its home when the goods its roofs have no room for would lose more
  in the open over a horizon than the store costs in hours and its means pay for all of it, else
  a workshop, else a larger home: the first of these it can pay for and find ground for. A
  store never moves its household's home.
- A workshop building may name a firm, and is then where the firm works, hires and keeps its
  stores (ADR-0006 §5).
  - As built, a household builds one for each firm of its that has lately had more people at
    work for it at once than a home holds (a tuning value), the busiest first, and never for a
    firm alone; one busier than any workshop has places for gets the largest. The firm works
    and hires there once its roof is on; its stock is still kept and sold at its owners' home.
    When a firm closes, its building goes to the household's oldest open firm without one, by
    the next day; a firm that outgrows its workshop stays in it (no extensions yet).
- Heirs take all of a household's buildings (ADR-0007 §1, widened from the home).
- Style traits are realised decisions kept in the spec's parameters. A household's taste and the
  provenance of each trait are behaviour state, saved but cheap to change.

### 8. Saves and boundary

- The `builds` section gains the Rect footprint, sixteen parameters, each building's groups'
  condition, its state and builders, plot uses and the firm link. Each settlement gains its
  trust sums. The schema becomes 13 or later in M3b.
  - As built, schema 15 (slice O's first step) adds the Rect footprint, sixteen parameters and
    plot uses for stores and workshops, and schema 16 (its third step) the firm link. Schema 17
    (slice P's first step) adds each group's condition, the building's state, the stage's
    builders' skill hours and the repair under way; saves of schema 16 and earlier load as the
    next bullet says. Builders' names and lean come with the slices that use them.
- Schema-12 saves load with every hut v1 and every building sound as of loading, with qualities
  drawn as if built at middling skill, so a loaded world shows no false wave of decay.
- The wire's `BuildingInfo` gains, appended:
  - the footprint's kind and size, storeys, bays and lofts;
  - each group's condition and state;
  - the building's state, and its symptoms in words;
  - floor and storage by use.
  - As built, wire 1.14 appends the grammar, the program's use, the size and direction, storeys,
    bays and loft bays, the gabled roof's corners and ridge, the apex height, floor by use,
    storage by kind and places to work. Wire 1.17 (slice P's first step) appends the state,
    the symptoms in words, how much of the roof leaks, each group's condition and the upkeep
    under way in words.
- New chronicle kinds and the cause *collapse* are appended.

## Consequences

- Houses can differ in plan, storeys and storage, and wealth has more to buy than radius
  (§9 NUDGE: house size by wealth is silent).
- What a household stores, and where, now matters: stores beyond roofed capacity spoil faster,
  a loft can be overloaded, and a leaking roof spoils goods.
- Buildings age by where they stand and how they are kept, so rebuilding has a cause.
- A loaded loft can sag and fail, and builders over-build for a while after a failure, from
  mechanisms rather than scripts.
- Forbidden:
  - building-wide health bars;
  - failure probabilities per building-year;
  - strength rerolled each check;
  - decay by calendar age alone;
  - a structural outcome decided by rendering physics (plan §5.1);
  - changing a saved spec in place.

## Alternatives considered

- **Extend the hut grammar with rectangular plans.** It would change hut v1's golden output or
  mix two construction systems in one grammar. A frozen hut and a separate frame grammar keep
  saved huts exact and each grammar small.
- **A margin per building, decaying with age** (plan §5.1's first sketch). It cannot tell a
  sagging loft from rotten posts, nor repairs from rebuilding. The research rejects calendar
  decay and health bars (11-06 §5.1, §5.7).
- **Located stores, each good placed on a floor and saved.** It is more precise, but needs saved
  inventories per space and the decisions to move them. A derived allocation by capacity gives
  loft loads with no new saved state.
- **Trust kept by each person.** It is closer to 11-06 §5.6, but nothing yet passes memories
  between people. The settlement stands in for the culture until cultures exist (M5).

## Revisit when

- Extensions, alterations or renovations are needed (per-bay progress; semantic ids leave room).
- Weather (M3c) replaces the peak-load stand-in, adds snow and wet seasons, and shares failures
  across a storm.
- Unreal's building kit (M2's S1 spike) fixes its module vocabulary.
- Builders are hired and their reputation matters, or codes arrive (M4): liability and
  inspection.
- Masonry, vaults or bridges arrive: new grammars and check modes.
