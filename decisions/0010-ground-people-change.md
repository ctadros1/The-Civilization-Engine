# ADR-0010: The ground people change: deposits and earthworks

Status: Accepted
Date: 2026-10-04
Milestone: M3b

## Context

The world's ground is generated once from its seed and never changes: elevation is a fixed bed
of 8 m cells, saved whole, and walking, water and the map all read it. Stone and flint lie as a
stock per hectare by habitat, a stand-in for geology (§9, slice H). M3b needs earthworks for
building (plot levelling, quarries and clay pits), deposits, and the god tool *place a deposit*
(plan §5.1, §7). M3c will add ditches and terraces on the same footing. Saved worlds will keep
what people dug and where the clay and stone lie, and Unreal's terrain will be rebuilt from the
same record (M2's S1 spike).

Reports read: 11-12 (earthworks), 03-05 (resource geology), 03-01 (terrain generation), 14-01
(runtime terrain), with 11-04 and 11-08 on digging and quarrying. Their main points:

- **Conserve soil mass, not apparent volume.** Bank, loose and compacted volumes differ
  (priors 1.20 and 0.90 of bank). A cut makes a stockpile and a depression, and no material
  appears without a visible pit or quarry (11-12 §1.1, §2.4).
- **Work is a sequence of operations,** with rates by ground. Non-metal digging is a prior of
  1.0 m³ of bank per eight-hour day (0.5–2.0), and spreading and compacting 3 m³ (11-12 §1.2,
  §2.4). Levelling a plot on a uniform slope cuts about A·s·W/8, and buildings often adapt to
  slopes instead (11-12 §2.5B).
- **Represent narrow works below the grid,** and update the bed in chunks (11-12 §5.1, §5.5).
  The physical bed is not the routing surface; after generation, change it only locally, with
  deliberate drainage invalidation (03-01 §3.1, §5.5).
- **Checkpoint terrain, not an endless replay.** Save materialised edit deltas with tile
  revisions, and never apply an additive edit twice (14-01 §4.1, §4.2, §4.4).
- **Geology is fixed; usefulness is conditional.** A deposit becomes a resource through
  exposure, discovery, access and processing. Knowledge of a deposit is separate from the
  deposit (03-05 §1.3, §5.1). Clayey ground is not pottery or brick clay, flint lies in
  particular horizons, and stone never renews (03-05 §1.1, §4.3).

## Decision

### 1. Deposits are bodies

- A **deposit** is a body with a permanent id. It has:
  - the good it yields;
  - a shape in integer centimetres;
  - a depth to its top (overburden) and a thickness;
  - a quality;
  - whether it shows at the surface;
  - an inventory in kilograms, which never renews: what is taken plus what is left is always
    the initial amount.
- The land profile's `[[deposit]]` entries place bodies once, when the land is created, from the
  seed and from rules on the generated terrain: habitat, slope, height above drainage and
  distance to a channel. As with habitats, placement is deterministic per seed (ADR-0004 §3).
  Geology generated before the surface (03-05 §1.2) is deferred.
- Slice H's loose stone and flint per patch stay as they are, beside the bodies.
- **Knowledge of a deposit** belongs to a settlement, with its finder and date:
  - an exposed body is found when a member's route passes near it;
  - a buried one is found when an earthwork cuts into it.
- **The god tool** `PlaceDeposit { at, kind, size, exposed }` creates a body on dry land and
  records it in the chronicle.
- As built (slice Q's first step): each `[[deposit]]` rule gives a slope range and a height-above-
  drainage range (distance to a channel waits), a count per qualifying square kilometre, and
  ranges for radius, cover, thickness and quality, a share of covered bodies that still show, and
  a density. Bodies are discs in plan with a centre and radius in centimetres. A new world places
  them after its founding band arrives, so the founding draws do not move; a save from before
  them gains, on loading, the bodies a new world of its seed has, with ids of its own.
- As built: "passes near" is within 50 m of a body's edge along the route of a walk, checked when
  the walk ends (a tuning value). The god tool takes a radius and whether the body shows, and
  takes the rest of the body from the land profile's rule for its good. Both are chronicled.
- As built (slice Q's third step): a platform finds a buried body when the deepest cut on its
  plot, times the share done, reaches the body's cover, and some of its plot lies over the body.
  This is a simplification: the deepest cut need not lie over the body. A pit is dug only on a
  body the settlement already knows, so it finds nothing new.

### 2. Earthworks are records

- An **earthwork** is a record in the land state, like a building. Its kind is platform, pit or
  spoil heap; kinds are append-only, and M3c adds ditches and terraces. It holds:
  - a shape in integer centimetres;
  - its target level or floor, decided when it is designed, and its side slope;
  - links to its plot, deposit and household;
  - planned and done volume (bank cubic metres);
  - its rules version.
- A pure, versioned expansion turns (record, progress, base heights) into a height patch,
  volumes and the needs of each stage, as building grammars do. The kernel runs it at 8 m and
  Unreal at its finer resolution. Golden hashes pin each version.
- A **platform** levels a plot by cut and fill.
- A **pit** deepens as material is taken; overburden and rejects go to a spoil heap beside it.
- Earth balances: what is cut equals what is filled, heaped and taken as goods, in kilograms.
- As built (slice Q's second step): platforms only, for plots whose ground drops more than the
  people profile's threshold. Households seek level ground first, so the weighing is that rule,
  not hours against hours. A platform's level balances its cut and fill by volume as the earth
  lay in the ground; bulking and compaction (11-12 §2.4) wait for spoil heaps and goods. Its
  earth costs hours on the building's first stage and is done with its first hours. The
  expansion samples every half metre; a golden hash pins version 1. Pits and spoil heaps follow.
- As built (slice Q's third step): clay pits. People dig at a body of the good their settlement
  knows, while their household needs it. A pit is a square of the people profile's
  `pit_side_m` (3 m) on the body, at the free spot nearest the settlement's hearth. Its heap is a
  square of the same size beside it, toward the hearth first. Each links to its deposit, and the
  pit links to its heap. A pit deepens by what each session digs, at `h_per_m3` (8 h a cubic
  metre: 11-12 §2.4's non-metal prior, dig and lift only). The cover and the share of the body
  unfit for use (one less its quality) go on the heap, and the rest is carried home, a load at a
  time. A pit dug through its body is done, and the next is begun on the body. Volumes balance as
  the earth lay in the ground, so bulking is still not kept: what is dug equals what is heaped
  plus what is carried, and the body's density turns volume into kilograms. Both change the
  ground evenly over their squares, which are smaller than a cell, as cell-mean deltas.
  Quarries for stone and flint follow.

### 3. The bed is the generated bed plus a delta layer

- The generated bed is never rewritten.
- Earthworks write a sparse layer of materialised, volume-conserving cell-mean deltas, in
  64 × 64-cell tiles with revisions, as worn ground does (ADR-0004 §4). The bed in use is base
  plus delta, rebuilt on load without replaying anything.
- A test checks that the records reproduce the tiles.
- Building earthworks are smaller than a cell, so foundation fit and the map read the records,
  not the bed's slope.
- In M3b, routing and water stay as generated (03-01 §3.1): earthworks are refused on or beside
  water. M3c brings drainage invalidation with its ditches.
- As built (slice Q's second step): cell-mean deltas in metres, in 64 × 64-cell tiles whose
  revision changes with each change. A platform advanced from one share to the next adds only the
  difference, and a test expands the records afresh and compares them with the tiles. Where two
  platforms' sides overlap, their changes add, each designed on the generated bed. A plot is
  levelled only where every cell its platform's sides could reach is dry land; otherwise it is
  passed over. Only the tests read the bed plus its deltas so far; the elevation raster gains
  them with the boundary's next step.
- As built (slice Q's third step): a pit or heap needs dry land and no other earthwork, plot or
  field on its square, and plots and fields keep their usual gaps from pits and heaps. Every earthwork's change to the ground is reproduced from its record alone:
  a platform as far as it is done, and a pit or heap as the earth it holds or lacks.

### 4. Saves and boundary

- Saves gain an `earth` section (records and delta tiles) and a `deposits` section (bodies and
  each settlement's knowledge of them). Saves without deposits place them on first load, from
  the seed, so older worlds gain the same deposits a new world of the same seed would have.
- The wire gains:
  - an earthworks revision in the snapshot;
  - a query for the earthworks, in words, and the changed tiles;
  - the deposits a settlement knows;
  - the `PlaceDeposit` command.
- The elevation raster serves base plus delta. All of it is appended.
- As built (slice Q's second step): wire 1.20 adds the earthworks revision, a query that lists
  each earthwork in words with every changed tile and its revision, and elevation served as the
  bed plus its deltas. The deposits and `PlaceDeposit` came with 1.19. The query of a
  settlement's deposits is the deposits query's `known_by`.
- As built (slice Q's third step): saves schema 21 add each earthwork's links to its deposit and
  heap. Wire 1.21 appends each earthwork's deposit, and kinds 1 (pit) and 2 (spoil heap), which
  the observer draws dark and pale.

## Consequences

- Clay, building stone and flint have places, quantities and finders, and pits and quarries
  visibly grow where they are worked.
- Levelling a plot costs real hours, so households weigh it against building on the slope
  (ADR-0009's lean).
- A world remembers how its ground was changed, and Unreal can rebuild it from the same
  records.
- Forbidden:
  - material without a source;
  - rewriting the generated bed;
  - replaying shovel events to rebuild ground;
  - an additive edit applied twice;
  - drainage changed silently by an earthwork.

## Alternatives considered

- **Edit the saved elevation in place.** It is simplest, but the bed becomes mutable, the
  generated surface is lost, and the raster and the records become two sources of truth.
- **Keep only the records and derive the bed on load.** Saves are smaller, but a change to the
  rasteriser would silently change saved ground unless every version were kept forever: the
  replay 14-01 §4.2 advises against.
- **Deposits as a per-cell layer.** It cannot hold a body's inventory, quality and finder
  together, and bodies are what 03-05 recommends.

## Revisit when

- M3c's ditches and terraces change drainage: invalidate routing locally.
- Geology arrives: generate rock bodies before the final surface (03-05 §1.2), and add ores.
- Unreal's terrain spike (S1) fixes its edit contract.
- Claims and law govern who may dig where (ADR-0007; M4).
