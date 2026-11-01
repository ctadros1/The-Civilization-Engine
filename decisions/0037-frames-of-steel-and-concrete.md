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

## Amendment (2026-11-01): the reinforced-concrete frame, as built

Decided by Claude under the owner's standing authority (plan §9):

- **§3, frame version 5.** Version 5 is version 4 with post system 3, a reinforced-concrete frame,
  and two more material slots, `concrete` (5) and `bar` (6); versions 1 to 3 stay frozen and
  version 4 expands within it as before, its two new slots needing nothing. A concrete frame is
  laid out as a skeleton is (storeyed posts, one storey too, up to twelve) and its posts, plates,
  tie and cross beams and joists are cast in their envelopes: its frame stage takes their volume
  at the program's `concrete_kg_per_m3` and `bar_kg_per_m3`, and labour at `concrete_h_per_m3`
  in place of the posts', beams' and joists' (its ridge, trusses and rafters stay timber). Walls
  are infill. The program's `[systems] concrete` names the technique, `reinforced_concrete`
  (portland cement, rolling and the jointed frame).
- **Concrete as a good:** a dry mix of portland cement and broken stone (about 1 to 5.5 by weight,
  a design prior), batched by those who know portland cement and kept dry as cement keeps; it is
  cast with water on site, which is not weighed.
- **§4, concrete smeared over its member's envelope,** as steel is: a cast member 1.6 times the
  envelope's diameter (a design prior; a concrete column is thicker than the oak it stands for)
  spread over it, read by the concrete good's `[timber]` (20 MPa concrete, about 1.5 % bar of 250
  MPa, cracked stiffness half of 30 GPa, creep φ 2, 0.85 under sustained load) and weighed at
  6,150 kg a cubic metre of envelope (2,400 over 2.56 of its area) in the own-weight check, where
  a skeleton's stays timber's. The program's figures follow the same scale (5,760 kg of mix, 307
  of bar and 64 hours a cubic metre of envelope: 2,250, 120 and 25 a cubic metre cast).
- **The designer** takes a concrete frame on the skeleton's terms: known, offered, and cheaper or
  the only design so tall. With the core content it is dearer than timber in kilograms, so it is
  taken only above four storeys, where its designed six-storey hall stands its checks.
- **§4's concrete by its age, as built (its second part):** a cast group carries, in the daily
  check, `days / (4 + 0.85 days)` of its full strength in bending and crushing from the day its
  forms are stripped (7 days; 0.70 then, all of it by about 27), counted from the day it was put in
  place (each group's `installed`, already saved); before, its forms carry it. Curing is elapsed
  time in the build: a concrete frame's walls are begun only once its forms may be stripped, 7
  days after its frame stage ended (the building's `stage_since`, already saved). The curve and
  the stripping age are the concrete good's (content API 217). Its stiffness is held at full, and
  the shaking and wind checks read the cured strength (simplifications: both matter for a frame's
  first weeks only). No save or wire change.
- **Mass infill, as built (its third part):** in version 5 a steel skeleton or a concrete frame
  may have mass walls (rubble, mud brick or fired brick, on their stone footing) as infill between
  its posts: its posts and frame stand as with wattle, and each storey's wall is a group of its
  own that bears only itself, held at each floor (as tied walls are); its storey's beams carry
  it and the frame's posts take its whole weight with what else they carry (a simplification:
  shared in proportion to what each storey's posts carry, not storey by storey). Wall ties are
  refused in a frame; version 4's skeleton still refuses mass walls. The designer offers the
  concrete frame to builders who would lay mass walls too. No content, save or wire change.
- **§5's protection, as built (its fourth part):** a member keeps all its strength through a burn
  whose full involvement is no longer than its protection: its own cover (concrete's 60 minutes,
  a `[timber] protection_min`) and, for a version 5 skeleton cased in the program's concrete
  (parameter 28, `ENCASEMENT`), the program's `encasement_min` (90 minutes, design priors within
  research 11-14 §2.5's 30-120); otherwise its material's `burnt_keeps`. The casing is its frame
  stage's concrete and hours. A fire's full involvement is 30 minutes, so cover and casing both
  outlast it. The casing's own weight is not added in the checks (a simplification), and the
  designer cases nothing yet: a skeleton is cased only where a design is given one.
- **§5's `ProtectedFrame` rule, as built (its fifth part):** a building rule template
  `protected_frame { minutes }` (content API 219; templates are content, so no save or wire
  change was needed after all): a new steel skeleton must be cased for at least `minutes`. It
  binds a skeleton begun once in force and is kept by one cased in a program that cases that
  long; its keepers case their next skeleton (written as version 5). A household weighs it by the
  bare skeletons it heard gave way or saw burn against the hours casing its own next home would
  take. The core's `zealously_zone_cased_steel` answers `burned`, asks an hour, and needs the
  steel skeleton to propose. **ADR-0037's deferrals are done**, but for those kept by the time
  box: glass, curtain walls, welding, prestressing, piles and settlement (§7), the casing's
  weight in the checks, the young concrete and infill in the shaking and wind checks.

## Revisit when

- Letting storeys is designed: lifts and wind follow.
- A lived world builds a frame of more than four storeys.
- Glass is made: windows and curtain walls become a choice.
