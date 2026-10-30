# ADR-0037: Frames of steel and concrete

Status: Accepted (2026-10-30, by Claude under the owner's standing authority)
Date: 2026-10-30
Milestone: M11
Amends: ADR-0031 §1 (a frame version 4 beside frozen versions 1 to 3), ADR-0033 (cement, steel
and rolling as recipes at the works), ADR-0022 §5 (what a full burn does to a metal member)

## Context

Plan §7's M11 lists "modern architecture primitives". ADR-0036 left them for their research, now
written: 11-14 (modern construction), digested with 12-10 in `docs/briefs/m11-building-waste.md`.
The report finds that frames change which limit governs height (walls eating floor under masonry;
stiffness, fire, foundations and lifts under a frame, 11-14 §1.1–§1.5), that each material fails
its own way (§1.2), that the materials come from long powered chains (§1.7), and that people built
tall where floor was dear and someone could fill the floors (§1.10). TCE prices ground in hours
(ADR-0030 §1) and lets height be a choice in those hours (ADR-0031 §6), but households build only
their own homes and nobody lets a storey.

## Decision

### 1. Parts

M11e builds this ADR in three parts: the materials (§2), frame version 4 with its checks (§3–§4),
and fire on metal with the observer (§5). Lifts, wind and letting storeys are deferred (§7).

### 2. Materials as recipes on the works' chain

- **Portland cement.** Goods `cement_clinker` and `portland_cement`; a technique `portland_cement`
  requiring `lime_burning` and `firing`; a recipe burning limestone and clay to clinker in a kiln
  as a batch by elapsed days (about 1.7 kg of raw material a kilogram of clinker, 11-14 §2.2), and
  one grinding clinker to cement with power at a site (ADR-0033 §5). Kiln fuel is any fuel its
  recipe names; figures beyond 11-14 §2.2's are design priors.
- **Steel.** A technique `steelmaking` requiring `blast_furnace`; a converter (a fixed tool good);
  a recipe turning `pig_iron` into `steel` as a batch, leaving slag (ADR-0038 §2).
- **Rolling.** A technique `rolling` requiring `steelmaking`; a powered recipe making
  `steel_section` and `reinforcing_bar` from `steel` at a mill (a fixed tool good).
- **Concrete is a building stage, not a good:** cement, sand or gravel and water are consumed where
  a member is cast; formwork is a tool worn per use. No glass is made yet.
- Each technique is tried in its recipe's work (ADR-0008); nobody starts knowing one; the observer
  may introduce them as any technique.

### 3. Frame version 4, beside frozen versions 1 to 3

- **Version 4 extends version 3's spec and assembly;** versions 1 to 3 are frozen, their goldens
  unchanged. `MAX_STOREYS` is 12 in version 4 (content narrows per program); 4 in version 3.
- **Two post systems are appended:** `skeleton` (steel columns per storey, steel girders and beams,
  floors on them) and `concrete` (columns, beams and slabs cast per storey, bar where tension is).
  Each is refused by name unless the builder knows its technique: `skeleton_frame` (requires
  `rolling` and `jointed_frame`) or `reinforced_concrete` (requires `portland_cement`,
  `rolling` and `jointed_frame`).
- **The enclosure is carried by the frame storey by storey.** A frame system's walls (timber,
  wattle or mass infill) are one storey's, bear on that storey's beams and never carry the walls
  above, so they keep one thickness however tall the building (11-14 §1.1, §1.3). Version 3's mass
  walls that carry the walls above are unchanged.
- **The stair stays required from three storeys;** its flights take floor as in version 3.
- **No caps:** a program's content gives ranges; the checks decide what stands.

### 4. Each material's own checks

- **Steel members** use `structure.rs`'s bending and buckling with a yield strength and stiffness
  from content (11-14 §2.1), not timber's.
- **Concrete** is checked in compression with its strength by its age since cast: its 28-day
  strength times a content curve of days (0.74 at 7 days, 11-14 §2.1). A member loaded before it
  has strength enough fails its daily check: curing is elapsed time in the build.
- **Reinforced members** carry bending by bar in tension and concrete in compression (a design
  prior of the simplest section rule; no report gives TCE one).
- Margins and states (`STRAIN_SHOWS`, failure below 1) are as now.

### 5. Fire on metal

- **Each steel group has a protection in minutes** (0 bare; encasement in content, 11-14 §1.6).
- **During a building's full burn** (ADR-0022 §4–§5), once its protection is spent, a bare steel
  member's strength falls along a content curve (about 0.6 of yield at 550 °C, 0.4 at 600 °C,
  11-14 §2.1); a member below its demand fails as any group does. Concrete's cover is its
  protection.
- **A rule template `ProtectedFrame { minutes }`** is appended beside `MostStoreys`, proposed only
  by someone who knows a frame technique and saw a building burn (ADR-0031 §5's path).

### 6. The boundary

Frame version 4's parameters, the new goods, techniques and recipes, and each group's protection
are content and saved specs: the save schema, wire and content API bump; no new saved section.

### 7. Deferred, by the time box

- **Lifts and wind:** both matter only where people build tall. A lift (a powered fixed tool whose
  shafts take floor) and a wind case on the lateral mode wait for the first tall frame in a lived
  world.
- **Letting storeys:** the reason anyone builds tall (11-14 §5.6). It is a residence change (a
  household living in another's building, on the `Rent` channel) and waits for its own design.
- **Glass, curtain walls, welding, prestressing, piles and settlement** (11-14 §5.8).

## Consequences

- A world where nobody knows the new techniques lives exactly as before.
- With nobody letting floors, a frame wins only where its cost in hours beats timber or masonry for
  the builder's own use: most likely a wide or fire-prone works. Most worlds will build none; that
  is a correct outcome (11-14 §5.6), not a reason to nudge.
- Steel and cement need the whole M10 chain (ore, coke, a furnace, power at a site), so a frame
  shows how far a world's works have come.

## Alternatives considered

- **A new `tower` grammar family** (11-14 §5.2). Cleaner for very tall buildings, but it duplicates
  v3's assembly, stages, checks and kit; a frozen version is how v3 was added.
- **Raising v3's storey cap.** It would change what v3 buildings are; versions are frozen.
- **A modern-era height allowance.** Forbidden: no eras, and height is a choice in hours.

## Amendment (2026-10-30): narrowed as built

Decided by Claude under the owner's standing authority, by the time box (plan §9):

- **§3, the skeleton only.** Frame version 4 adds post system 2, a steel skeleton: storeyed posts
  (allowed on one storey too) whose posts, plates, tie and cross beams and joists take a fifth
  material slot, `frame`, named by the program with its `frame_kg_per_m3`; walls are infill
  (wall kind 0). The reinforced-concrete frame is deferred with mass infill, and `MAX_STOREYS`
  stays 4 in version 4 (taller needs a new `Level` and wire change, and nobody builds tall until
  letting storeys exists, §7).
- **§4, steel smeared over its member's envelope.** A steel member is drawn as the envelope of the
  timber member it replaces, with an I-section's properties spread over it (a design prior:
  bending 68 MPa, compression 33.5, stiffness 61 GPa, no creep, from 235 MPa and 210 GPa at about
  0.14 of the envelope's area and 0.29 of its section modulus), so `structure.rs`'s checks apply
  unchanged. Concrete's age curve waits with the concrete frame. A skeleton member's own weight in
  the checks still uses the program's timber density, and its labour timber's: both simplifications.
- **The designer** offers the skeleton where its builders know `skeleton_frame` and the program
  names it, and takes it only when it takes fewer hours and fewer kilograms than the timber design
  and more of neither, as it reaches for no version 3 system but ties. With the core content steel
  weighs more than the timber it replaces for the same labour, so the designer never takes it; a
  skeleton is built only where a design is given one (the observer, or a later designer that
  sizes for spans timber cannot carry). Recorded, not nudged.
- **§5, fire on bare steel,** is the next part.

## Revisit when

- Letting storeys is designed: lifts and wind follow.
- A lived world builds a frame of more than four storeys.
- Glass is made: windows and curtain walls become a choice.
