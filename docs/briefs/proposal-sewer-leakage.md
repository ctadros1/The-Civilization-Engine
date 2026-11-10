# Proposal (status: **Proposed**, not decided): leakage through worn sewer walls

Backlog C's "Not yet" list (plan §7, M11f) leaves open "leakage through worn walls (a contagion
node at the worn cell and a rate the research lacks)". This brief proposes a design that needs no
invented rate. Nothing is implemented and nothing is decided.

## What exists now (claude/claude-development 48cd61b4)

- **Wear** (ADR-0044 §5; `population/sewers.rs` module doc): every month each route cell of a sewer
  wears `1 / life_years` of its soundness, times its ground's wetness and the month's rain, as a
  well's lining does. The per-cell loss is kept in `Sewer::loss`. A cell worn through collapses.
  Homes entering at or above it heap again until it is relined.
- **Loads** (`contagion.rs`): what homes served by a sewer shed goes with their share to the
  outfall (`sewer_loads`) and its settling tank (`Node::Sewer`), losing nothing on the way. Heaps
  already soak into the ground toward wells and reaches as `Transit` loads, using the
  source-to-receptor connections of research 12-02 §5.3, with travel time and attenuation.
- **Research:** 12-02 §1.5 lists leakage among what a sewer's conveyance must represent
  ("transfer it to the outlet unless leakage, overflow, settling, or treatment intervenes"). The
  report gives **no exfiltration rate** for a worn conveyance. 12-02 §5.3 gives the shape of
  ground transport (directional connections, travel time, attenuation). Its §2 figures (horizontal
  migration from under 1.5 m to about 25 m; 30 m pit setbacks) are about pits, not sewers.

## Proposed design

1. **No new rate: leakage scales with wear already measured.** A load passing a route cell leaves
   a share equal to that cell's lost soundness (`loss[i]`, 0 when sound, 1 when worn through) times
   the share of the day's flow a sound sewer of that age would carry anyway, which is 1. In words: a
   wall a quarter worn lets a quarter of what passes it leak. This is a **design prior**, stated as
   such: it bounds leakage between none (a new sewer) and all (a collapse, where the existing rule
   already heaps the flow). It introduces no free parameter. The brief does not claim the
   research supports linearity.
2. **The leaked load enters the existing ground path.** It becomes a `Transit` from a new node
   `Node::SewerCell(sewer, cell)` (an appended save code), placed at the cell's centre. Wells and
   reaches are found through the same source-to-receptor connections heaps use, so travel time,
   attenuation and decay are the heap pathway's own, unchanged.
3. **What a household can know:** nothing new. Wells already record contamination moves (12-02 §5.2's
   "this well received contamination from Pit 184"). A move from a sewer cell is recorded the same
   way and named in the observer's words: "seeped from the sewer's worn wall 40 m above its
   outfall". Relining (already built) restores the cell's soundness and stops the leak. No new
   decision, no inspection: 12-08 §5.6, "faults are seen only as they show", still holds.
4. **Mass:** only disease loads leak. The deposit and sludge mass accounting is untouched, as the
   contagion path already separates loads from mass.

## Acceptance examples (tests to write)

- **New sewer, no leak:** with every cell sound, the outfall receives exactly what was shed.
  Existing `sewer_loads` tests are unchanged.
- **Quarter-worn cell:** one cell with `loss = 0.25`, ten doses passing it, and a well within the
  heap pathway's reach. 2.5 doses leave at that cell as a `Transit`, the outfall receives 7.5, and
  the well later receives what the existing attenuation lets through. Assert the conservation sum
  with nothing else changed.
- **Cells below the leak:** cells downstream of a worn cell carry only what is left. Two
  quarter-worn cells in series leave 10 × 0.75 × 0.75 at the outfall.
- **Relined cell:** after relining, no further leak at that cell.
- **No sewer, no change:** worlds without sewers have identical save digests apart from the
  appended node code, and identical bench report lines.
- **Saves:** a `Transit` from `Node::SewerCell` saves and loads exactly.

## Bounded implementation plan

Three commits:
1. `Node::SewerCell` appended, with its save code and round-trip test.
2. Leakage in `sewer_loads`, walking the route from the entry cells to the outfall, with the
   conservation tests above.
3. The well move's words in the observer.

Estimate: one working session. It touches `contagion.rs`, `population/contagion.rs`,
`population/sewers.rs` (the route walk only) and `persist/agents.rs` (node code). No content or
wire field is needed unless the observer words go on the wire, which today they do through
existing text fields.

## Uncertainties for the owner

- **Linearity.** Whether a share equal to lost soundness is acceptable as a stated prior, or the
  owner prefers leakage to wait for a sourced rate. If it waits, this stays unbuilt.
- Whether stormwater overflow (12-02 §1.5's "overflow"), when the sewer has no room, should be
  designed at the same time. This brief leaves it out.
- Whether connections entering below a worn cell should be counted from their own entry cell. The
  proposal does: each home's load only passes the cells between its entry and the outfall.
