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
