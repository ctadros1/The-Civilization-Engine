# ADR-0004: Building specs, land state and paths

Status: Accepted
Date: 2026-10-03
Milestone: M1

## Context

M1's people forage, cut wood, clear fields, build huts and wear paths into the ground (plan §7).
That makes four kinds of world state that will be saved for years and read by Unreal from M2:

1. **Buildings.** The plan's pipeline stores only a small `BuildingSpec` and derives geometry
   from it with `civ-grammar`, the same crate Unreal will call over FFI (plan §3.4).
2. **Land.** Wild food, game, fish and wood grow back and are used up; fields are cleared, sown,
   harvested and left fallow.
3. **Plots**: who has claimed which ground, for which use.
4. **Paths**: wear left by walking, which slowly becomes the first road network (plan §5.1).

Reports read: 11-03 (shape grammars), 01-12 (computational geometry), 10-03 (organic urban form),
10-09 (procedural city generation), 03-06 (ecology and foraging), 08-01 (forager and early-farming
economy), 10-01 (site selection), 03-02 (hydrology); 10-06 and 11-04 for hut sizes and labour.

Their main points:

- **Store realised decisions, not seeds.** A footprint chosen by its builder must survive grammar
  changes; re-deriving it from a seed after an upgrade silently rewrites history (11-03 §4.8,
  01-12 §3).
- **Expansion must be a pure function**, tested with golden hashes, and every part needs a
  construction stage (11-03 §4.1, §4.8, §5.2).
- **Integer coordinates** for anything with an exact boundary; floating point for drawing only
  (01-12 §4.1).
- **Habitat depends on content thresholds and seeded noise**, so rebuilding it on load could
  change which stocks a saved world has; store it (03-06 §5.1, 10-01 §5.5).
- **Wild stocks need one growth model with no respawn timers** (03-06 §1.1–1.3).
- **Path wear is path-dependent**: whether a trail survives depends on its history (hysteresis),
  so its state must be saved, while road polylines can be derived (10-03 §1.1, §2.2).

## Decision

### 1. Coordinates

- World positions on the boundary stay `f32` metres from the map's north-west corner, x east and
  y south (ADR-0001's `Vec2`).
- **Footprints and plots are integer centimetres** (`i32`), angles in 1/65,536 of a turn.

### 2. `BuildingSpec` and `civ-grammar`

- A spec holds what was decided: grammar id (content id), grammar version, footprint (`Round`
  centre and radius, or `Rect` centre, size and angle), storeys, up to eight integer parameters
  (eave height, roof pitch, door direction for a hut), material ids, a style seed, condition,
  and construction progress per stage. **The footprint is authoritative**; the grammar never
  moves it.
- `civ_grammar::expand(spec, rules) -> Expansion` is **pure**: no clock, world access or shared
  randomness. Any draw it makes is keyed by `(style seed, part path)`. The expansion lists
  **parts** (id, kind, construction stage, position, size, material slot), the outline, the
  door, the labour and material each stage needs, and derived facts (floor area, sleeping places).
- Stages are `foundation`, `frame`, `walls`, `roof`, `finish` (plan §3.4).
- Golden-hash tests pin expansions. A change to what a rule produces **bumps the grammar
  version**; existing buildings keep their spec and are expanded with the rules of their version.
- M1 has one program, the hut. Its rules are Rust code with every dimension taken from content;
  authored rule graphs arrive with grammar v2 (M3), when there is more than one program to share
  them.

### 3. Land

- **Habitat patches**: a grid of 128 m patches (16 × 16 terrain cells) with a habitat class and a
  richness value, computed when a world is created, and on the first load of an M0 save, from
  terrain, water, slope and height above the nearest stream, using content thresholds. **Saved.**
- **Stocks** (wild plant food, game, fish, standing and dead wood) live per patch and grow
  logistically with daily steps and a yearly climate factor. **Saved.** Nothing respawns.
- **Fields** are entities: a rectangle in centimetres, the household that works it, a crop, a
  stage, fertility, and the dates each stage started. **Saved.**
- **Plots** are claims: rectangle, use (dwelling, field), claimant, status (provisional, held,
  abandoned), dates. A plot outlives its building. **Saved.**

### 4. Paths

- **Wear** is a saturating value per 8 m terrain cell, kept in sparse 64 × 64-cell tiles, raised by
  completed trips along their route and decaying in closed form when read. Cells cross into and
  out of the **trail** state with hysteresis. **Saved** as `u16` wear plus a trail bit.
- Trail polylines and the junctions between them are **derived** on load and each month from the
  trail cells. Walking cost falls with wear, so used paths attract more use.

### 5. Settlement

- A **settlement** record holds its name (from a naming event), founding date and hearth
  position. Labels such as "camp", "hamlet" or "village" are inferred from what exists (plan §1),
  never stored as state.

### 6. Saves

New sections: `land` (patch classes, richness and stocks), `fields`, `plots`, `builds` (specs and
construction progress), `wear` (tiles), `settle` (settlements). M0 saves load with land computed on
first load and every other section empty.

## Consequences

- A saved hut looks the same after any later grammar change, and Unreal can expand a spec with
  the same code the kernel uses for its facts.
- Land state costs about 0.3 MB per 16 km world plus a few kilobytes per village; wear tiles exist
  only where people walk.
- Content changes to habitat thresholds affect new worlds only.
- Forbidden from now on: geometry stored as authoritative state; seeds as the only record of a
  realised design; respawn timers; grid-wide sweeps over idle wear cells.

## Alternatives considered

- **A TOML rule language for the hut now.** It is the plan's direction, but with one program in M1
  it would be designed against a single example; it waits for grammar v2.
- **Habitat derived on every load.** Smaller saves, but a content change would silently remap the
  stocks of saved worlds.
- **Wear on a 1 m raster.** Truer path widths, but sixty-four times the cells and a second routing
  resolution; M1 draws trails as smoothed polylines instead, and the raster can be refined later.
- **An ID-stable road graph now.** Needed once roads are built and maintained (M3 onwards); in M1
  trails are only worn, so derived polylines suffice.

## Revisit when

- A second building program arrives (grammar v2), or Unreal needs part data the expansion lacks.
- People build or maintain roads deliberately, and road edges need identities of their own.
- Wear tiles or land stocks exceed 10 MB in a save.
