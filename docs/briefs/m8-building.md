# M8 design brief: tall buildings, monuments and earthquakes

**Scope.** Plan §7's M8 ("The growing city (v1)") lists "Grammar v3: 3–4 storeys, monuments and landmark templates, style drift fully live", "Earthquakes act through structural margins; taller buildings and longer spans need the matching techniques" and "Earthquake god tool (damages buildings; terrain unchanged)". Plan §5.1 adds that "Monument templates use the same grammars at larger scale, with rarer rules: domes, towers, colonnades, stairs", that demand comes "from span, storeys and load ... plus events: floods, storms, fire damage, earthquakes", and that a household chooses "how tall (land value, tech)". Plan §4.2 gives notables the move "commission a building or monument, patronize a style". Plan §4.3 sets M8 at 10–50k agents in 2–3 (up to 5) settlements, and the demo is "a 30k-person planned capital next to an organic port town". This brief covers:

- grammar v3: buildings of three and four storeys, wider roofs, openings and the style traits the grammar has reserved since ADR-0009 §2;
- the techniques that let a building be taller or wider, as structural systems rather than caps;
- a lateral mode for structural margins, and the earthquake that acts through it, as the observer's tool;
- what people learn and what laws they may make after shaking;
- monuments and public buildings: who decides, who pays, who works, and what the engine can say about why;
- style drift across the whole repertoire, and interiors only as far as uses per storey.

I assume sibling M8 briefs own the planning institution, zoning and land value; roads, traffic and boats; road grading; crowd LOD; and the scale gate. This brief needs one thing from planning: land that costs something to spread over (§1.7). Without it nobody has a reason to build upward.

**What exists to build on** (checked in the code):

- **The frame grammar is at version 2 and full.** `civ-grammar/src/frame.rs`: `FRAME_VERSION = 2`, `MAX_BAYS = 8`, `MAX_STOREYS = 2` ("research 11-05 §2.2: one to three in timber; two here"), `MAX_GROUPS = 32`. `PARAMS = 16` (`lib.rs`), and `frame_params` uses all sixteen: `EAVE_CM` to `FOOTING`, then four `STYLE` traits that the grammar refuses unless 0 ("frame version 1 and 2 have no style traits"). `WALL_KIND` 0–3 (wattle, rubble, mud brick, fired brick), `FOOTING` 0–1, and "mass wall kind ... requires a stone footing". A post "runs through every storey to the plates"; the width is constant and tie beams span it. `PartKind` has `Ladder` but no stair and no window. The hut stays frozen at `HUT_VERSION = 1`.
- **Mass walls are one height for all storeys.** In `groups()`, a `GroupKind::MassWall` (code 10) takes `section: [l.l, l.wt]` and `length: storeys_h`, so a two-storey wall is checked as one unrestrained height. Its wall kind rides in the group's `spacing` field. A frame's mass wall takes the program's daub thickness (`d.wall_cm`; the longhouse's `wall_cm = [10, 25]`, design 15 cm), and `design_cautious` (`build.rs`) never thickens it.
- **Margins are gravity only.** `civ-agents/src/structure.rs` (`margin`, `Loads`, `Timber`, `POST_IMPERFECTION` 0.5, `STRAIN_SHOWS` 1.5): bending for beams, joists and rafters; Euler buckling or crushing for posts; for a mass wall, crushing at 1.5, 1.0 or 3.0 MPa (code constants) times a slenderness factor of 1 up to H/t 6 falling to 0.2. `weight_on_posts` leaves out every `MassWall`'s own weight. The storm reaches roofs as `peak_pa` on plan and as uplift on coverings (`floods.rs`); nothing pushes a building sideways. At the longhouse's 15 cm, a one-storey 2 m rubble wall has H/t 13.3 and a factor of 0.39 (my arithmetic from the code).
- **Failure and death.** `population/loads.rs` weighs every standing building daily (`check_buildings`); `check_household_origin(ctx, household, storm, observer)` reweighs under a given pulse. Giving way kills people inside at `DIES_IN_RUIN` 0.3, `DIES_UNDER_ROOF` 0.15, `DIES_UNDER_FLOOR` 0.05 (plan §9's NUDGE), cause `Cause::Collapse`, chronicle `ChronicleKind::BuildingFailed`. A failed mass wall leaves a ruin (plan §9, slice BE).
- **Caution is per program technique.** `caution.rs` (`Trust`, `CautionParams`; `[build.caution]` `half_life_years = 8.0`, `most = 2.0`). `population/trust.rs` counts a failure against `catalog.buildings[i].technique`, the program's (`core:technique/jointed_frame` for a longhouse), so a fallen rubble wall teaches about jointed framing. `FireTrust` is kept by covering and wall kind, and mass walls are chosen only through it (`FireDesign`, `caution_wattle > 1.0` and a known masonry technique).
- **Style v0 and contact.** `style.rs`: three traits (pitch, eaves height, overhang), `alpha = 0.1`, `prestige_most = 3.0`, `innovation = 0.01`, `seen_most = 8`, `sight_m = 200.0` (`early_farmers.toml`); founding ways (`PlacesSave.ways`), the three clocks (`style_clocks`) and the neighbours row (plan §9, slice AR).
- **Building rules.** `PolicyKind::BuildingRule` with `RuleTemplate::{RoofsUnburnt, WallsOfKinds, LeastGap}` (`polity.rs`), issues `IssueKind::Burned` and `IssueKind::GaveWay`, `Household.gave_way` memory for `rules.memory_years` (10). Rules bind buildings begun under them; the M6b demo found that a village that mends rather than builds is never bound (plan §9).
- **Hazard god tools.** `Request::{SetFire, RaiseRiver, SendStorm, Drought, Plague}` (`civ-host/src/protocol.rs`), each an `InfluenceKind` (codes 0–10, `Storm = 10`) whose effects carry `floods::Origin { known, observers }` (`natural()` when no observer touched it). `Population::send_storm` sets today's wind, runs `check_household_origin` for every household and chronicles `ChronicleKind::Influence`. There is no fault, tectonic or ground-class state in `civ-world`; `terrain::height_above_drainage` and the floodplain are the only ground facts.
- **Public works and owners.** `PolicyKind::{BuildCrossing, DigWell, BuildBank, KeepBank, Enclose}` ask each household an equal share of the work, given by choice; `PolicyKind::PayWork` pays coin per hour from the store (M7d); `keep_store.toml` keeps the common store "under their own roof". Every `civ_land::Building` has a `household`; a polity owns no building. Patron gifts exist (`population/patronage.rs`), and ties keep evidence in `Domain::{Provision, Craft, Word, Counsel}`.
- **Content.** Programs `hut`, `longhouse` (`storeys = [1, 2]`, `width_cm = [400, 700]`), `granary`, `workshop`. Founders bring `rubble_walling` and `mud_brick` but not `jointed_frame`; plan §9 logs "NUDGE: no village builds a frame". No religion, ritual, burial or commemoration exists in content; the values are autonomy, reciprocity and security.
- **Boundary.** Save schema 139, wire 1.83, content API 131 (plan §9, CE5).

## Where the reports agree

Eight points recur across 11-05, 11-06, 03-07, 11-10, 11-03, 11-02, 11-09 and 11-13. I treat them as invariants.

1. **Systems, not material caps.** TCE "should model structural systems, not assign each material a maximum span or height" (11-05, executive conclusion); "Do not enforce the table's upper values as hard caps" (11-05 §2.1). Earthquake damage depends on "construction ... more than era" (03-07 §2.2 A), and "good performance should be conditional, not a material hierarchy" (11-06 §4; *dhajji dewari*, 11-06 §3.8).
2. **Vulnerability, trigger and propagation apart; one event, one evaluation.** Record all three (11-06 §1.1). "Do not repeatedly reroll the same event" and "do not double-count vulnerability" (11-06 §5.2); a persistent exposure "must not become a fresh independent damage roll" (03-07 §5.3). Quality is drawn once, at building (03-07 §2.2 A; 11-05 §1.6).
3. **Damage is not collapse.** Physical damage, habitability, repair and collapse are separate (03-07 §1.1; 11-06 §5.2). "Cracks do not imply immediate collapse" (11-05 §4), and Nepal's "destroyed" is not physical collapse (11-06 §2.1). Damage is retained for aftershocks (11-06 §1.2).
4. **No disaster calendar.** No table gives earthquakes "every *x* years" (03-07, executive recommendation); recurrence belongs to a fault system (03-07 §2.1), and the ancient collapse baseline is "unknown" (11-06 §2.1). A god's earthquake runs the same machinery, with an origin flag (03-07 §1.3).
5. **A monument is an ordinary project with extraordinary meaning.** It is "not a special building that automatically produces prestige" (11-10, executive conclusion). It needs mobilizable, delivered resources (11-10 §1.5), comes in campaigns and phases (11-10 §1.7; 11-06 §3.4), is limited by its scarcest rate (11-10 §5.3), and must not "silently create an off-map workforce" (11-10 §2.5).
6. **Feasibility before taste, and the realised design is what is kept.** "A strong aesthetic preference cannot create an unavailable material or unsupported span" (11-03 §4.6); taste innovation is not technical innovation (11-02 §5.4). Save the realised decisions and grammar version, as "a seed alone is insufficient" (11-03 §4.8). A grammar does not make a building sound (11-03 §1.5; 11-05 §5.6).
7. **Rules change new work and are fought over.** Disasters open politics (11-09 §1.2); inquiries attribute causes that may be wrong, so "a collapse caused by undermined foundations might produce a height restriction" (11-09 §1.4); Lima's elite resisted height limits after 1746 (11-06 §3.6). Every rule needs an activation mode (11-09 §1.7).
8. **Style moves at commissions, and the stock lags.** Exposure, implementation and built stock are three clocks (11-02, opening); preferences change new buildings and renovations, never the existing city (11-03 §4.6; 11-02 §1.1). At 1 % replacement, universal adoption reaches about 39 % of the stock in fifty years (11-02 §2.3).

**What games and models teach to avoid:** Pharaoh's predetermined monument objectives and Foundation's player-directed patronage (11-10 §5.8); Dwarf Fortress cave-ins as a safety model (11-06 §5.8) and Medieval Engineers as validated engineering (11-05 §5.6); a gravity-equilibrium solution read as seismic safety (11-05 §5.6, RhinoVAULT); modern OpenQuake inventories as ancient values, and an aggregate loss curve as an agent-level collapse model (03-07 §5.5); "higher level" meaning "more modern" (11-03 §3, Cities: Skylines II); instant culture-wide visual assignment (11-02 §5.6, Crusader Kings III); laws that enforce themselves (11-09 §5.6, Eco); unconstrained WFC as the whole-building planner (11-03 §1.4).

**Where they differ, or differ from the plan:**

- **Techniques as gates.** Plan §7 says taller buildings and longer spans "need the matching techniques", and plan §5.6 has each technique raise "the structural capacity or maximum span". 11-05 §2.1 and its executive conclusion reject caps. They agree if a technique opens a *system* the grammar can express (a truss, storeyed posts, walls tied at each floor) and the checks decide how far it goes (§1.2).
- **How many storeys.** Plan §7 asks 3–4. 11-05 §2.2 offers 1–2 for earth, 1–3 for timber and 1–5 for masonry as "low-confidence content-generation priors", and Shibam's mud brick reaches seven storeys. London's house classes were 2, 3 or 4 storeys (11-09 §2.1). I read "3–4" as the grammar's reach, not a limit by material.
- **EMS-98's words.** 11-06 §2.3 reads "few" as 0–20 %, "many" 10–60 % and "most" 50–100 %; 03-07 §2.2 A reads them as about 1–15 %, 15–55 % and 55–100 %. Both call them imprecise bands; I use them only as shape checks (§3).
- **Mechanics or fragility.** 11-05 §5.4 proposes a lateral surrogate `V = C_s·W`, with `C_s` "not simply equal [to] peak ground acceleration"; 11-06 §5.2 and 03-07 §2.2 A propose lognormal fragility curves by archetype. TCE's margins are mechanical (ADR-0009 §5). I propose the surrogate with one keyed response draw per building per event, so fragility emerges and is measured, not authored.
- **Natural earthquakes.** Plan §7 names only the god tool. 03-07's "Recommended first implementation" includes "regional earthquake events with construction-class damage", but its §3.2 asks for "tectonic source systems, not arbitrary dangerous mountains", and TCE's worldgen has none. I defer natural ones (§2).
- **Why monuments are built.** 11-10 §1.2's motive vector starts with sacred obligation, burial and ancestral commemoration, and plan §5.1 lists shrine and temple. TCE has no religion or burial, so those motives have nothing to be about. Only services, standing and esteem have mechanisms today (§1.8).
- **Domes.** Plan §5.1 names domes; 11-05 §5.4 asks for "authored structural families with offline validation" for groin vaults and domes. I defer them (§2).
- **Memory against rarity.** Caution's 8-year half-life sits in 11-09 §2.3's 2–20 years; 03-07 §5.4 gives 10–30 years (D). Earthquakes are rarer than either, so a lesson will often be forgotten before the next one. I treat that as plausible and measure it, rather than tuning memory to the hazard.

## 1. Mechanisms

| Mechanism | Saved state | Driven by |
|---|---|---|
| Grammar v3 | A frame version 3 spec with more parameters: per-storey wall thickness, roof system, post system, ties, openings, stair, four style traits | A household's or a law's design within known techniques |
| Systems as techniques | Content techniques with their content footprint; nothing new per person (ADR-0008 knowledge) | Finding, contact, the knowledge tool |
| Gravity for tall and wide | Nothing new: margins stay derived | Own weight of walls, restraint per storey, purlins and trusses |
| Lateral margins | Nothing new outside an event | `V = C_s·W` on each group's mode |
| An earthquake | An event record: day and minute, strength, origin, response draws keyed, per-building outcome; loss on cracked groups | The observer's tool |
| Learning and rules after shaking | Trust per technique used (more keys, same record); appended rule templates | `gave_way` memories; forecasts |
| Building upward | Nothing new: shapes and plots as now | Land that costs (planning brief), household size, means |
| Public buildings and monuments | A building owned by a polity; a project with sponsor, payers, workers, phases and four dates | A law; a patron's share; equal shares; pay for work |
| Style fully live | The four style traits in specs; taste over seven traits | Commissions, exemplars seen, prominence, materials |
| Uses per storey | Program content: each storey's use; a let storey (open question) | The household's needs |

**Derived, not saved:** every margin, lateral or gravity; a building's vulnerability class; the settlement's damage counts by intensity; monumentality's attributes (11-10 §1.1); style clocks; what any household forecasts.

**Size (my arithmetic).** 30,000 people at five a household is 6,000 households; at about 1.5 buildings each, some 9,000 buildings. Sixteen more 4-byte parameters each is about 0.6 MB. An earthquake's record with one outcome byte and one response draw per building is under 0.1 MB. The lateral check runs once per event over the stock, not daily.

### 1.1 Grammar v3: storeys, walls by storey, openings

**Reports.**

- Overall timber height "is not unsupported column length. Repeated structural levels and connections matter" (11-05 §2.2, Fogong pagoda, 67.31 m). Represent the load path to the ground (11-05 §1.1) and give joints their own capacity (11-05 §1.3).
- Earth walls: "thick, predominantly compressed mass"; check eccentric compression and out-of-plane instability (11-05 §1.2). H/t below about 4–6 is fairly stable, 9–12 much poorer, and is "not a universal seismic pass/fail rule" (11-05 §2.5).
- Openings and connections matter to vulnerability (03-07 §2.2 A; 11-06 §1.2: "enlarging an opening changes the load path").
- Storey heights in London's classes were 2.59–3.20 m (11-09 §2.1); 11-03 §4.4 proposes floor-height families of 2.5 and 3 m and a 0.25 m quantum.
- Give every rule a termination contract and a diagnostic on failure (11-03 §4.2); expand tests before the rule language (11-03 §5.2).

**My proposal.**

- **Frame version 3** beside frozen versions 1 and 2 (their goldens unchanged). `MAX_STOREYS` 4, `MAX_BAYS` 12 for public programs (content narrows per program), `MAX_GROUPS` kept at 32 for houses and raised for public programs only if a test shows the need (11-06 §5.1: 8–32, "increasing only for large or unusual structures").
- **Mass walls per storey.** One `MassWall` group per wall per storey, each with its own thickness, so a wall may taper upward and each storey's H/t is its own. The wall kind moves out of `spacing` into the group's material, and assembly strengths move from code into content (rubble 1.0–2.0, dressed stone 2.6–3.8, brick in lime 2.6–4.3 MPa, with unit weights 19–21 and 18 kN/m³: 11-05 §2.3). Mass-wall programs get a thickness range of their own, not the daub range.
- **Openings.** A share of each storey's wall left open (doors, windows, shuttered or not), on the bay rhythm. It costs no wall, reduces bearing and lateral length, and is a style trait (§1.9). New part kinds `Window` and `Stair`, appended.
- **Stairs.** A building of three or more storeys needs a stair, which takes floor area from its storeys (my proposal; no report sizes one).
- **Spec.** `PARAMS` grows from 16 to 32 for version 3 (ADR item): roof system, post system, ties, per-storey wall thickness (4), opening share per storey (4), stair, and the four style traits.

### 1.2 Techniques that open systems

**Reports.**

- Span envelopes by system: pole beams 2–5 m routine and 5–8 specialist; timber trusses 6–12 and 12–20; masonry arches 2–8 and 8–25 (11-05 §2.1, low to medium-low). An added intermediate support "can greatly reduce beam demands" (11-05 §4).
- Timber trusses fail by member tension and compression, buckling, joint slip and splitting (11-05 §1.2). Westminster Hall needed "a specialized system and strengthened masonry supports—not simply longer ordinary rafters" (11-05 §2.2).
- Doubling a span at fixed section gives 4× bending stress and 16× deflection (11-05 §4).
- A recipe is "most trustworthy near the conditions under which it was learned" (11-05 §3.1); a builder knows examples and conditional rules, "rather than an omniscient structural-margin number" (11-05 §3.1, §5.5).

**My proposal.** Three content techniques, each entering with its footprint (plan §5.6's rule):

- **`trussed_roof`**: principal rafters, a tie and purlins at each frame line, so common rafters span between purlins. It opens frame widths beyond what pole rafters carry. With pole rafters, a 12 m width at 45° gives rafters about 8.5 m long (my arithmetic), past 11-05 §2.1's specialist pole range, so the existing bending check refuses them; the truss shortens them. Joint slip is a joint capacity (11-05 §1.3), a design prior per joint.
- **`storeyed_frame`**: each storey's posts stand on the frame below instead of running through. A post's buckling length becomes one storey; Euler's load goes as 1/L², so a through-post of 8 m is 16 times weaker in buckling than a 2 m storey post of the same section (my arithmetic), and needs a tree that long.
- **`tied_masonry`**: floors and roof framed into the walls at each storey (bond timbers, through-joists), so a wall's restrained height is one storey. Without it, a mass wall is checked over its full height, as version 2 does now.
- **What they require.** `trussed_roof` and `storeyed_frame` require `jointed_frame`; `tied_masonry` requires a masonry technique and `jointed_frame`. Each is `tried_in` building or mending frames (ADR-0008), learnt beside a knower, carried by contact. Founders know none.
- **No caps.** A program's content gives ranges a spec may choose; the checks, not the technique, decide whether a chosen building stands.

### 1.3 Gravity checks for tall and wide buildings

**Reports.** "The building stood yesterday" does not validate a changed building (11-06 §1.2). More masonry "can improve overturning resistance while worsening foundation demand" (11-05 §1.5). Crowded assembly floors are a 4–5 kPa stress case (11-05 §2.4). One metre of wheat adds about 7.6 kPa (11-05 §4).

**My proposal.**

- **Walls carry themselves.** A mass wall's demand includes the walls above it. A 4-storey rubble wall 8 m high presses about 0.15 MPa at its foot from its own weight at 19 kN/m³ (my arithmetic), 8–15 % of rubble's 1.0–2.0 MPa before any floor.
- **Each storey's floor bears on its storey's walls**, and its live load uses the floor's use: 1.5 kPa for living (as `LIVE_PA`), 4–5 kPa for a hall's upper floor (11-05 §2.4).
- **Footings.** A stone footing's bearing against the ground stays as v2 builds it; settlement waits (11-05 §1.5 asks for it; no ground classes exist).
- **Caution thickens walls.** `design_cautious` sizes mass-wall thickness by caution itself (crushing and overturning capacity both go as t), within the program's range.

### 1.4 Lateral margins

**Reports.**

- "A lateral-force surrogate such as `V = C_s·W` can drive an initial mechanism screen", with `C_s` the building's effective response, calibrated against building types "using connection quality, geometry, roof mass, and damage state" (11-05 §5.4).
- A no-tension interface "cannot transmit the required compression-only resultant" at |e| ≥ t/2; sliding needs |T| ≤ μN + cA (11-05 §5.3). Intact air-lime rubble joints fitted μ = 0.558, c = 0.0815 MPa; for cracked joints c = 0 and μ 0.4–0.7 as priors (11-05 §2.5).
- Vulnerability comes from "mass, connections, geometry, ductility, and ground conditions" (11-06 §1.2). EMS-98 puts rubble and adobe in class A, unreinforced brick B, massive dressed stone C, timber D, each "with variation" (03-07 §2.2 A).
- Japan's 1924 provision was 0.1 × building weight, "**not** a collapse acceleration threshold" (11-06 §2.2).

**My proposal.** Each group gains a lateral mode, used only when something shakes:

- **Mass walls, out of plane.** A wall rocking about its foot under its own weight and a lateral share `C` of it has e = C·h/2, so 11-05 §5.3's |e| ≥ t/2 gives `C_crit = t/h` over its restrained height (my arithmetic). A 50 cm rubble wall on a 2 m storey holds to 0.25; the same wall untied over two storeys to 0.125; the longhouse's 15 cm wall to 0.075, below the 0.1 that Japan designed for (11-06 §2.2, as an illustration only). Getty's H/t bands then read as `C` of about 0.17–0.25 (stable) and 0.08–0.11 (poor), which agrees in shape with 11-05 §2.5.
- **Mass walls, in plane.** Sliding at the foot and at each storey: the storey shear against μN + cA, with c lost once cracked (11-05 §2.5).
- **Timber frames.** Posts set in the ground act as cantilevers against the storey shear, in bending with the existing strength, quality and loss; storeyed frames also check their joints. A three-bay longhouse weighs about 4.3 t in thatch, timber and boards (plan §9); at `C` = 0.3 its eight 15 cm posts each carry about 1.6 kN against roughly 9.4 kN at a 2 m height in green oak (my arithmetic), a margin near 6 before joints and before the daub's inertia, which the posts also hold and for which these reports give no density. In the rubble version the walls themselves, about 7.5 m³ or 14 t at 19 kN/m³ (my arithmetic), hold their own inertia with no frame behind them and topple at `t/h`. Whether light frames and heavy masonry then order themselves as EMS-98 does (03-07 §2.2 A) is a test (§3), not a class table in the engine.
- **Coverings.** Tile and thatch add mass on the frame; their own shedding waits (no source).
- **Margins and states as now.** Below 1 a group fails; below `STRAIN_SHOWS` it shows (cracks, lean).

### 1.5 The earthquake as the observer's tool

**Reports.**

- "Magnitude is not local shaking intensity"; aftershocks strike already weakened structures; slope failure, fire and tsunami "require their own physical conditions" (03-07 §1.1).
- GEM's building-to-building log dispersion of 0.30 is a starting reference, "not ... universal premodern-building parameters" (11-06 §2.2).
- Use the real occupancy; "there is no defensible universal 'percentage killed in a collapse'" (11-06 §5.3). Brittle failures give little warning (11-06 §5.3). Mortality rises steeply: 0.019 % of the exposed at MMI VII against 4 % at IX, in one fitted model (03-07 §2.4). Collapse and death are not interchangeable (11-06 §4: deaths in about 4 % of bridge collapses).
- "Do not automatically spawn an epidemic after a disaster" (03-07 §1.2). Recovery envelopes (1–8 weeks to shelter, 3–36 months to rebuild housing) are design priors to check against (03-07 §2.5, D).

**My proposal.**

- **The tool.** `Request::Shake { strength, at }`: a lateral share of weight (0.02–0.6) at the settlement the observer names, the same over the map for v0, since a map of a few kilometres lies inside one event's footprint (my reading; no attenuation relation is given in these reports). Terrain is unchanged. An appended `InfluenceKind::Earthquake` (code 11) and every effect carry `Origin` with the observer's id.
- **One evaluation.** Each building's response is `C_s = strength × exp(0.30·z)`, z drawn once keyed by `[seed, event, building]`. Quality already carries workmanship, so the draw stands for response and site only (11-06 §5.2's safeguard). Every standing building, finished or under construction, is checked once in all its lateral modes at the event's minute; failures go through the existing `give_way` path with the trigger named "the shaking".
- **Damage short of collapse.** A mass wall whose lateral margin falls below `STRAIN_SHOWS` takes an authored loss (cracks) and loses cohesion; upkeep mends it as it mends worn daub. A second shake, from the observer, meets the cracked stock (11-06 §1.2; 03-07 §5.2). Aftershock sequences wait (no rate is given).
- **People.** Whoever is inside at that minute faces the existing shares by what fell, so a night shake kills more than a noon one without a rule. Everyone feels it; households whose home is a ruin go to kin or the hearth, as flooded households do. Trapped people and rescue wait (§2).
- **The record.** An incident per settlement (buildings shaken, cracked, fallen, deaths, by wall kind and storeys) and an episode linking them, kept apart from natural counts by origin; the chronicle says "The observer shook the land; 14 of Westford's 60 buildings fell, 11 of them stone".
- **People cannot tell** a god's earthquake from a natural one; statistics leave it out by origin (ADR-0022 §14), including plan §4.7's failures row.

### 1.6 What people learn and what laws follow

**Reports.**

- Lessons should be about "specific combinations of span, material, connection, foundation, and loading"; public distrust generalizes further; "survival under light loads should not be treated as proof of resistance to an unexperienced earthquake" (11-06 §5.6).
- A rule needs a property or a technique (11-09 §5.2), including "Maximum storeys; height by street class". Istanbul in 1849 allowed 10.5 m in timber and 15 m in masonry (11-09 §2.1); Rome's 70 feet was a jurisdictional rule (11-06 §2.2). Visible excess height is easier to detect than poor joints (11-09 §1.6).

**My proposal.**

- **Trust per technique used.** A building's failure is remembered against every technique it was built by (program, wall kind, and any v3 system), and its years count for each. `Trust` keeps its key; a settlement holds more records. A fallen rubble wall then raises caution for rubble walling, which thickens the next rubble walls (§1.3).
- **Quiet years teach the wrong lesson.** Years without shaking count as survival, so caution falls and walls thin again. I would not correct this; it is 11-06 §5.6's warning, measured.
- **Memory and issue.** Every household hears of buildings that gave way as now (`Household.gave_way`), with the trigger kept, so `IssueKind::GaveWay` opens.
- **Two appended templates.** `MostStoreys { storeys, kinds }` (height by wall kind, a dimension rule) and `TiedWalls` (new mass walls tied at each floor, a technique rule, proposed only by someone who knows `tied_masonry`). `WallsOfKinds` already lets a village forbid rubble or require timber.
- **Forecasts carry the conflict.** A household that would build tall forecasts the floor a height rule takes from its next building; one that lost kin to a wall forecasts the losses avoided. Whether a village bans height, ties walls, returns to timber or does nothing is its gathering's (11-06 §3.6; 11-09 §1.5). Fire caution pushes toward masonry and shaking may push back; neither is authored to win.

### 1.7 Why anyone builds upward

**Reports.** Plan §5.1: "how tall (land value, tech)". 11-02 §2.3: growth shortens turnover (with 0.5 % replacement, 2 % growth brings half the stock to a new trait in 28 years rather than 139). London's classes tied storeys to street class (11-09 §2.1).

**What the code implies.** A household builds the cheapest home that covers its members and goods among programs it knows, then the largest its means buy (ADR-0009 §1). Plots keep `PLOT_GAP_CM` 100 cm and are free. With free land and no frame known, nobody builds a second storey, let alone a fourth (plan §9's NUDGE).

**My proposal.**

- **Costs of spreading.** A plot's cost comes from the planning brief (price, rent or a zone's limit) and from walking time to where the household works and trades, which the trip model already knows. A taller building then wins when its extra structure costs less than the land and walking it saves.
- **Shared roofs.** A storey let to another household (rent in goods, on the `Rent` channel) would give a landlord a reason to add storeys and a newcomer somewhere to live (11-13 §1.4, §5.2). It needs a residence change (open question 7).
- **No push toward height.** If nobody builds tall in the demo's time box, the nudge is a land or cost prior, logged, never a storey count.

### 1.8 Public buildings and monuments

**Reports.**

- Keep relative investment, prominence, craft distinction, symbolic association, accessibility and historical depth apart, and "do not collapse them into one number until a particular agent evaluates them" (11-10 §1.1). "Perceived power ≠ perceived competence ≠ moral legitimacy ≠ affection" (11-10 §1.3); store sponsor, payer, labour provider, designer and beneficiary separately (ibid.).
- Projects can come from councils, clans and coalitions without a state (11-10 §1.4). Financing (gifts and pledges, a patron's income, taxes, compulsory labour, endowments, credit) determines control and vulnerability, and construction finance differs from operating finance (11-10 §1.6).
- Track proposal, commencement, first use and completion (11-10 §1.7). Maintenance can be the institution's purpose (11-10 §1.8).
- 10,000 people with 5,000 giving 20 days a year supply 100,000 person-days; a million-day project needs ten years (11-10 §2.5). Earth excavation 2.6 m³ and transport 0.384 m³ a person-day at 500 m; about 87 % of the labour is carrying (11-10 §2.3). At St Paul's, 12 % of labourers gave over 60 % of labour days (11-10 §4). Prior: 0–30 person-days a participant a year (11-10 §5.6).
- Overoptimistic patrons should sometimes stall (11-10 §5.2); public reactions divide and change (11-10 §4); diminishing returns on influence (11-10 §5.4).
- Institutions purchase and schedule shared interiors (11-13 §1.8); assembly needs a gathering area and access hierarchy, not a scaled domestic room (11-13 §3.3).

**My proposal.**

- **A building a polity owns.** `Building` gains an owner that may be a polity (ADR item). Its upkeep is nobody's unless a law asks, as for banks (ADR-0022 §11).
- **Programs with a use the engine can state.** `storehouse` (the common store's room, so the keeper's roof stops limiting it), `hall` (where the gathering sits and, with an upper floor, where a forum hears cases) and `tower` (height for the watch to see from). Each answers an issue whose loss is measured: the store's spoilage; gatherings rained off or crowded past the hearth (open question 9); fires noticed late. Without such a loss, nobody proposes one.
- **The law.** An appended `PolicyKind::RaiseBuilding` naming a program, a site and a design (shaped like `BuildCrossing`): passed, it begins; each household is asked an equal share of the work, given by choice; `PayWork`, extended to these works, may pay for the hours; crews are capped by work fronts (11-10 §5.3). A larger hall than need is a sponsor's design choice that the gathering votes on, with the hours in its words.
- **Patrons.** A sponsor may pay a share in goods or hire hands, recorded as gifts with their reason, as patron gifts are (M7c). Those who use the building gain `Provision` evidence for the payer and `Counsel` evidence for the sponsor if it was built as proposed, or the reverse if it stalled (11-10 §5.4; §4's ambition outrunning budgets). Builders gain `Craft` evidence. Nothing gives a completion bonus.
- **Campaigns.** Four dates kept (11-10 §1.7); a stage stopped for want of hands or goods stays visible (plan §5.1). Enlargement is a new project.
- **Scale check (my arithmetic).** A 30,000-person capital with 15,000 adults giving 20 days a year supplies about 300,000 person-days, so a hall of 20,000 person-days takes a month of the year's goodwill, and a mound of 3 million person-days would take ten years of every adult's 20 days.
- **Deferred.** Shrines, temples, tombs and commemorative monuments, until content gives a reason they could serve (§2).

### 1.9 Style drift, fully live

**Reports.**

- Style is a distribution of combinations across layers: construction, spatial program, visible form, surface and symbolism, execution quality (11-02 §5.1; 11-03 §4.6). Seeing is not knowing how to build (11-02 §1.1); a collapse may weaken a design's appeal (11-02 §5.3).
- Prestigious projects "create unusually visible exemplars and specialist employment" (11-02 §1.2); prominence is height and visibility (11-10 §1.1).
- Priors: α 0.02–0.20, prestige 1–3×, innovation 0.1–3 % a commission, ornament reconsidered every 5–20 years (11-02 §2.2). Several tempos coexist in one settlement (11-02 §3). Correlate variation by settlement, building, episode and part (11-03 §4.5).

**My proposal.**

- **Four traits more,** in v3's style parameters: opening share (window rhythm), ornament (carved or painted members, hours of craft a square metre of front), finish (bare, daubed or lime-washed, using quicklime), and symmetry of the door and openings. Each costs real hours or goods, and openings weaken walls, so taste meets structure without a special case (as eaves height does now).
- **Taste over seven traits,** moving as now by α × admiration ÷ `prestige_most`, each building met once. Ornament and finish only move toward exemplars a household could build: one it lacks the craft for is admired but not copied (11-02 §1.1).
- **Prominence.** A building is seen from `sight_m` scaled by its height over the eaves of buildings near it, so a hall or tower is an exemplar from further away. Public buildings are admired for their sponsor's esteem, as buildings elsewhere are (plan §9, slice AR).
- **Materials as constraints.** Tile, brick and lime enter taste only where the household can get them; climate acts through loads (snow on low pitches) as now.
- **Ideology.** Plan §5.1's "austerity values lower ornament" has no value to hang on (open question 8).
- **Clocks.** The settlement panel's three clocks cover all seven traits; the neighbours row counts them.

### 1.10 Uses per storey

**Reports.** Space is allocated to activities, not fixed functions (11-13 §1.1); production and retail compete with domestic space, as in the Horsham shop with its rear working space (11-13 §1.5); Titchfield's market hall combined selling, an upper meeting chamber and a lock-up (11-13 §1.8). "Do not impose an abrupt occupancy cutoff" (11-13 §5.1).

**My proposal.** A program names each storey's use (work below, living above, as `ground_use` and `upper_use` do for two). A workshop-house program follows Horsham. The room graph, fixtures and reservations of 11-13 §5.1 wait for UE interiors; TCE keeps spaces by level and use.

## 2. What to defer, and why

| Defer | To | Why |
|---|---|---|
| Natural earthquakes | When worldgen has a tectonic source | 03-07 §2.1, §3.2: recurrence attaches to a fault system, not a landscape picked as dangerous |
| Aftershock sequences | With natural earthquakes | 03-07 §2.1 asks for clustering; no rate is given; the observer may shake again |
| Landslides, liquefaction, tsunami, fire after shaking | Later | 03-07 §1.1: each needs its own physical conditions; terrain is unchanged by plan |
| Ground classes and site response | Open question 5 | 11-06 §1.2 names ground conditions; no report gives a factor, and TCE has only HAND |
| Trapped people and rescue | With injury | 11-06 §5.3: rescue depends on voids, labour, tools and secondary collapse |
| Vaults, domes, arches in buildings | Post-v1 | 11-05 §5.3–5.4: a thrust-line check or authored families validated offline |
| Shrines, temples, tombs, memorials | When content gives them a purpose | 11-10 §1.2's sacred and commemorative motives have no mechanism in TCE |
| Interior room graphs and fixtures | With UE interiors | 11-13 §5.1; the kernel needs only uses per storey |
| Inspection and enforcement of building rules | With the planning institution | 11-09 §1.6: enforcement is a sequence of costly actions |
| An earthquake that changes terrain | Never in v1 | Plan §7: "terrain unchanged"; §1 forbids terrain god tools |

## 3. Guards: risks and validation

**Plot.** No date, population or wealth opens a storey, a technique or a monument; no earthquake comes without the observer; no hall is proposed unless a measured loss stands behind it; no completion bonus; no material class table read by behaviour. If nobody builds a frame or a second storey in the demo's time box, nudge a land-cost or technique-spread prior, never a building, and log a `NUDGE:` (plan §1, rule 3).

**Nobody builds tall.** This is the largest risk. The M3b demo found no village builds a frame, founders do not know framing, and I found no later §9 entry reporting a lived world that built one. Storeys three and four need framing, a new technique and a reason (§1.7). Measure before tuning: households who know each technique, designs considered and why rejected, the cost per square metre of floor by storeys.

**Shaking calibrated in circles.** EMS-98 intensity is partly assigned from damage, so "validating a damage model against damage-derived intensity can be circular" (11-06 §2.3). Use the bands for shape only: ordering by wall kind and storeys, and nonlinearity, never to fit a level.

**Every stone village falls, or none does.** With 15 cm mass walls, rubble villages would fall at modest shaking (§1.4). Fix the wall thickness range from content before any calibration, never the shaking.

**Stylized facts, as tests:**

- **Doubling a span** at fixed section gives 4× bending stress and 16× deflection; an intermediate support (a purlin) relieves it (11-05 §4).
- **Order by construction, not era** (03-07 §2.2 A; 11-06 §4): sweep strength from 0.05 to 0.6 over one village built four ways (wattle frame, rubble, mud brick, fired brick in lime, tied and untied); masonry should cross from "few" to "many" falling well before timber, with spread inside each kind.
- **Nonlinear damage and deaths** (03-07 §4, §2.4): damage and deaths per exposed person rise much faster than strength through thresholds; deaths are not proportional to buildings fallen (11-06 §4).
- **Counterfactual settlements** (03-07 §4): the same event over identical villages that differ only in ties, wall thickness or storeys explains every difference through those parts.
- **Cracks are not collapse** (11-05 §4): a second shake of the same strength fells more of a cracked, unmended village than of a mended one.
- **Rules lag** (11-09 §4; 11-02 §2.3): after a height rule, old tall buildings stand and new cohorts differ; at 1 % replacement 61 % of the old stock remains after 50 years.
- **Monuments are rare and lumpy** (11-10 §3, §4): count new starts per 100 settlement-years by function; stalled projects exist; a small core of workers gives most hours.
- **Style tempos differ** (11-02 §3, §4): public buildings and houses drift at different speeds; the neighbours row stays green or amber, never red.

**Leaking truth.** No forecast reads a margin, a response draw or the kernel's cause; households know what they saw fall and what it was built of (11-06 §5.4).

**Compute.** Lateral checks run only on an event, once per building. Gravity checks keep their daily cadence; v3 adds at most four wall groups per building. The yearly style review meets each new building once per household: at 6,000 households and some 300 new buildings a year, about two million meetings (my arithmetic), to be measured against the scale gate.

**Saves.** Save → load → save exact in the middle of a public project and after a shake with cracked walls; an old save's v1 and v2 buildings expand to their goldens; a god's earthquake keeps its origin.

**Observer.** The inspector shows each storey's walls, ties, openings and roof system in words, and why it stands ("rubble walls 50 cm, tied at each floor"). An earthquake incident panel lists what fell, cracked and stood, by kind and storeys, and who died where; the map shades damage. A public project panel shows sponsor, payers, workers, phases and the four dates. The chronicle speaks plainly: "The shaking brought down nine stone houses at Westford and none of its timber ones."

## 4. Proposed slices

1. **Grammar v3 vocabulary, no change in behaviour.** Frame version 3, `PARAMS` 32, per-storey mass walls, openings, stairs, truss and storeyed-post expansions, content strengths, goldens for v3 and unchanged hashes for v1 and v2, the save and wire boundary. *Tests:* goldens; refusal diagnostics; an old save loads exactly. *Observer:* the inspector's words per storey.
2. **Tall and wide under gravity.** Walls' own weight, restraint by ties, purlins, joints, live load by use; `trussed_roof`, `storeyed_frame`, `tied_masonry` as content; caution thickens mass walls; trust against every technique used. *Tests:* 11-05 §4's span facts; a 4-storey untied rubble design refused by its margins and the tied one standing; ten-year smoke with the failure band unchanged. *Observer:* margins in words; the knowledge panel lists the new techniques.
3. **Lateral margins and the earthquake tool.** Lateral modes, `Request::Shake`, `InfluenceKind::Earthquake`, one keyed evaluation, cracks as loss, deaths by occupancy, incidents and episodes with origin, the preview (only certain effects, ADR-0022 §14). *Tests:* the four-way sweep, counterfactual villages, cracked versus mended, save mid-aftermath, origin kept out of the failures row. *Observer:* the earthquake panel and damage layer.
4. **After the shaking.** `MostStoreys` and `TiedWalls` templates, forecasts of floor lost and losses avoided, attribution from what fell. *Tests:* a shaken village proposes a rule only where someone knows the remedy; a household planning height opposes. *Observer:* the law's words name the fallen buildings it answers.
5. **Public buildings.** Polity owners, `RaiseBuilding`, storehouse, hall and tower programs, equal shares and pay for work, patron shares, ties evidence, four dates, stalls. *Tests:* no proposal without a measured loss; a stalled hall costs its sponsor `Counsel` evidence; ledger conservation over a project. *Observer:* the project panel.
6. **Style fully live.** Four more traits, prominence-scaled sighting, copy-only-what-you-can-build, clocks over seven traits. *Tests:* a world of one settlement drifts as before in the three old traits; a tower is seen from further; the neighbours row stays green or amber. *Observer:* style words in the inspector and settlement panel.
7. **The M8 building demo, time-boxed.** In the demo world, measure who builds tall, what public buildings arise, then shake the port town and the capital alike and record what falls and what laws follow. Nudges logged.

## 5. Open questions for the designer

1. **Techniques.** Are `trussed_roof`, `storeyed_frame` and `tied_masonry` the right three, and do founders of any M8 world bring them, or framing itself?
2. **Strength's unit.** Is a lateral share of weight the right observer-facing strength, or should the tool speak in an intensity word with a mapping kept out of behaviour?
3. **Footprint.** One strength over the map, or an epicentre with a falloff, which no report here gives?
4. **Natural earthquakes.** Defer them to a worldgen tectonic source, or allow a landscape-authored source flagged as a design prior?
5. **Ground.** Should low ground by HAND shake harder, with a flagged factor, or not at all until a source gives one?
6. **Death shares.** Do the existing collapse shares serve shaking, or does a wall falling outward need its own debris footprint (11-06 §5.3)?
7. **Shared buildings.** May a household let a storey to another, changing residence rules, in M8?
8. **Austerity.** Should a value or ideology bear on ornament, as plan §5.1 suggests, and which?
9. **The hall's reason.** Does a gathering suffer from rain or crowding today, so a hall answers a measured loss, or must that be built first?
10. **Polity owners.** Do organizations (M7c) own buildings too, or only polities?
11. **Fire after shaking.** Should a building that falls on a burning hearth ignite, as a flagged prior, or wait?
12. **Dashboard.** Does shaking get its own row, apart from plan §4.7's structural failures, and is that row computed without interventions?

## 6. What an ADR would need to decide

Plan §1 allows about three ADRs a milestone, and planning and transport will want theirs. I suggest one, "Tall buildings, public buildings and shaking", amending ADR-0009 and ADR-0022. It would decide:

- **Frame version 3 and the spec:** `PARAMS` from 16 to 32, the meaning of each new parameter, per-storey mass-wall groups and their ids, and that v1 and v2 stay frozen (ADR-0009 §1–3; 11-03 §4.8).
- **The lateral mode:** `V = C_s·W` per group by mode, evaluated once per event with one keyed response per building, cracks as loss, never rerolled (ADR-0009 §5; 11-05 §5.4; 11-06 §5.2).
- **The earthquake record:** an appended `InfluenceKind`, the event and its per-building outcomes, incidents and episodes with `Origin`, and the cause and chronicle words (ADR-0022 §13–14).
- **Owners:** a building owned by a polity (and perhaps an organization), its upkeep by law only.
- **Trust by technique used:** what a failure is remembered against (ADR-0009 §6).
- **Whether natural earthquakes exist in v1:** I would say no, until a tectonic source does.

Policy, issue, template, part, group and influence kinds stay append-only.
