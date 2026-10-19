# ADR-0031: Tall buildings, public buildings and shaking

Status: Accepted
Date: 2026-10-19
Milestone: M8d
Amends: ADR-0009 §1–3 (frame grammar version 3 beside frozen versions 1 and 2), ADR-0009 §5 (a
lateral mode for every group, used only when something shakes), ADR-0009 §6 (trust kept by every
technique a building used), ADR-0004 §2 (a building a polity owns), ADR-0022 §13–14 (an appended
influence for shaking, its records and their origin), ADR-0013 (an appended law kind that raises a
building)

## Context

Plan §7's M8 lists:

- "Grammar v3: 3–4 storeys, monuments and landmark templates, style drift fully live";
- "Earthquakes act through structural margins; taller buildings and longer spans need the
  matching techniques";
- an "Earthquake god tool (damages buildings; terrain unchanged)";
- and a demo, "a 30k-person planned capital next to an organic port town".

The building brief (`docs/briefs/m8-building.md`) reads 11-05, 11-06, 03-07, 11-10, 11-03, 11-02,
11-09 and 11-13. It finds eight points they share:

- systems, not material caps;
- vulnerability, trigger and propagation kept apart, with one evaluation per event;
- damage is not collapse;
- no disaster calendar;
- a monument is an ordinary project with an extraordinary meaning;
- feasibility before taste, and the realised design is what is kept;
- rules change new work and are fought over;
- style moves at commissions, and the stock lags.

What the kernel has (checked in the code):

- **The frame grammar is at version 2 and full.** `FRAME_VERSION` 2, `MAX_STOREYS` 2,
  `MAX_BAYS` 8, `MAX_GROUPS` 32, and all sixteen `PARAMS` used. The four style slots must be 0. A
  mass wall is one group over every storey.
- **Margins are for gravity only.** The storm reaches roofs and coverings; nothing pushes a
  building sideways.
- **Failure and its deaths go through `give_way`,** with the existing shares by what fell.
- **Caution is kept per program technique,** so a fallen rubble wall teaches about jointed
  framing. Fire trust is kept by covering and wall kind.
- **The hazard tools carry their origin** (`InfluenceKind` 0–10, `Storm` = 10).
- **A building belongs to a household,** and perhaps a firm. A polity owns no building.
- **Laws ask equal shares of work and may pay for it.**
- **No village builds a frame** (plan §9's NUDGE, 2026-10-05), so nobody has ever built a second
  storey.
- **M8c gave land a cost** a household reckons in hours: walks, fees, ground rent and expected
  sanction. Plots may be given on a plan's parcels, which are fixed in size.
- **One settlement of 10,000 lives a day at Max in about 40 s** on the development container
  (slice CX).
- **Boundary:** saves schema 161, wire 1.89, content API 149.

## Decision

### 1. Frame version 3, beside frozen versions 1 and 2

- **A third frame version is added; versions 1 and 2 keep their goldens.** Version 3 has:
  - `MAX_STOREYS` 4;
  - `MAX_BAYS` 12, which content narrows per program;
  - `PARAMS` 32 for version 3 only. A spec's parameter count follows its version, so a saved
    version 1 or 2 spec is read as it was.
- **The new parameters,** in order:
  - roof system (pole rafters or a truss);
  - post system (through posts or storeyed posts);
  - wall ties (none or at each floor);
  - each storey's mass-wall thickness (four slots);
  - each storey's opening share (four slots);
  - a stair;
  - the four style traits (§7).

  Unused slots are 0, and the grammar refuses non-zero values where its version gives them no
  meaning. Every refusal names the parameter.
- **Mass walls are grouped per storey,** one group per wall per storey, each with its own
  thickness. A wall may thin upward, and each storey's height over thickness is its own. A
  group's material carries its wall kind (the `spacing` field no longer does, in version 3).
  Assembly strengths and unit weights move from code constants into content. A program with mass
  walls gives its own thickness range, apart from the daub range.
- **Openings** are a share of each storey's wall, set on the bay rhythm. An opening costs no wall,
  and it shortens the wall's bearing and lateral length. `PartKind::Window` and `PartKind::Stair`
  are appended. A building of three or more storeys needs a stair, which takes floor area from
  each storey.
- **The realised spec is saved, never re-derived from a seed** (ADR-0009 §2; 11-03 §4.8).

### 2. Techniques open systems; the checks decide how far they go

- **Three content techniques,** each entering with its footprint (plan §5.6):
  - `trussed_roof`: principal rafters, a tie and purlins at each frame line;
  - `storeyed_frame`: posts that stand on the storey below;
  - `tied_masonry`: floors and roof framed into the walls at each storey.

  The first two require `jointed_frame`. `tied_masonry` requires a masonry technique and
  `jointed_frame`. Founders know none of them. Each is learnt by trying, beside a knower or by
  contact (ADR-0008).
- **No caps.** A technique lets the grammar express a system; the margins decide whether a
  chosen building stands. A program's content gives the ranges a spec may choose from.
- **Gravity checks grow with the grammar:**
  - a mass wall carries its own weight and the walls above it;
  - each storey's floor bears on that storey's walls, with live load by the floor's use;
  - a storeyed post buckles over one storey;
  - a tied wall is restrained at each floor;
  - a truss's members and joints are checked, with joint capacity as a design prior.
- **Caution thickens mass walls** within the program's range, as it now deepens timber sections.

### 3. A lateral mode for every group, used only when something shakes

- **Every group gains a lateral check of `V = C_s·W`** (11-05 §5.4), used only during an event:
  - a mass wall out of plane holds while `C_s < t/h` over its restrained height (11-05 §5.3);
  - a mass wall in plane slides when its storey shear exceeds `μN + cA`, with `c` lost once
    cracked (11-05 §2.5);
  - posts act as cantilevers in bending against the storey shear, and a storeyed frame's joints
    are checked as well.
- **Nothing lateral is evaluated daily.**
- **No vulnerability class is read by behaviour.** Whether light frames and heavy masonry order
  themselves as EMS-98 does (03-07 §2.2 A) is a test, not a table.

### 4. Shaking: the observer's tool, one evaluation, kept apart by origin

- **`Request::Shake { strength, settlement }`** puts a lateral share of weight (0.02–0.6) over the
  map. The map's few kilometres lie inside one event's footprint, so v0 has no epicentre and no
  falloff. Terrain is unchanged.
  - `InfluenceKind::Earthquake` is appended as code 11, and its effects carry `Origin` with the
    observer.
  - The observer's panel speaks in intensity words, mapped from the strength. Behaviour never
    reads the words.
- **One evaluation per event.** Each building's response is `C_s = strength · exp(0.30·z)`, with
  `z` drawn once, keyed by `[seed, event, building]` (11-06 §2.2's dispersion as a starting
  reference). Quality already carries workmanship, so `z` stands for response and site only.
  - Every standing building, finished or under construction, is checked once in all its lateral
    modes at the event's minute.
  - Failures go through `give_way`, with the trigger named.
  - The same event is never rerolled (11-06 §5.2; 03-07 §5.3).
- **Damage short of collapse.** A mass wall whose lateral margin falls below `STRAIN_SHOWS`
  cracks: an authored loss, and its cohesion gone. Upkeep mends it as it mends worn daub. A
  second shake meets the cracked stock.
- **People inside** face the existing shares by what fell, so a night shake kills more than a noon
  one without a rule. A household whose home falls goes to kin or to the hearth, as a flooded one
  does. Trapped people and rescue wait.
- **Records.** Each event keeps:
  - its day, minute, strength and origin;
  - each building's response draw and outcome (stood, cracked or fell);
  - an incident per settlement and an episode linking them.

  The chronicle names what fell by wall kind and storeys.
- **People cannot tell the observer's shaking from a natural one.** Statistics, including plan
  §4.7's structural failures row, leave it out by origin (ADR-0022 §14). Shaking gets a row of
  its own on the dashboard, which counts interventions apart.
- **No natural earthquakes in v1.** Recurrence belongs to a fault system (03-07 §2.1), and the
  world has none. Aftershocks, ground classes, landslides, liquefaction and fire after shaking
  wait with it.

### 5. What a failure teaches, and the rules that may follow

- **Trust is kept by every technique a building used:** its program's, its wall kind's and each
  version 3 system's. A fallen rubble wall raises caution for rubble walling, which thickens the
  next rubble walls. Quiet years count as survival, so caution falls again (11-06 §5.6's warning,
  measured and not corrected).
- **Two building-rule templates are appended:**
  - `MostStoreys { storeys, kinds }`, a dimension rule;
  - `TiedWalls`, a technique rule, proposed only by someone who knows `tied_masonry`.

  They open under `IssueKind::GaveWay` as the existing templates do. A household that would
  build tall forecasts the floor a height rule takes from it; one that lost kin forecasts the
  losses avoided. Neither is authored to win.

### 6. Building upward, and buildings a polity owns

- **Height is a household's choice in hours.** A taller home wins when its extra structure costs
  less than the ground and walking it saves: plot fees, ground rent, expected sanction and walks,
  as ADR-0030 §1 reckons them. A plan's fixed parcel makes floor area dearer to spread. Nothing
  pushes toward height. If nobody builds tall in the demo's time box, the nudge is a land or
  technique-spread prior, logged, never a storey count.
- **A building may be owned by a polity** (`Building.polity`). Its upkeep falls to nobody unless a
  law asks it, as with banks (ADR-0022 §11).
- **`PolicyKind::RaiseBuilding` is appended,** shaped like `BuildCrossing`: a program, a site (on a
  plan's public site where one is in force) and a design.
  - Passed, it begins. Each household is asked an equal share of the work, given by choice.
  - `PayWork` may pay for its hours, and a patron may give a share in goods or hire hands,
    recorded as gifts with their reason.
  - Crews are capped by work fronts.
  - Proposal, start, first use and completion are kept as four dates; a stall stays visible.
- **Public programs answer measured losses only:**
  - `storehouse`, for the common store's spoilage under a keeper's roof;
  - `hall`, for gatherings held in the open, measured first;
  - `tower`, for fires noticed late.

  Without a measured loss, nobody proposes one. Those who use the building gain `Provision`
  evidence of its payers and `Counsel` evidence of its sponsor if it was built as proposed, or
  the reverse if it stalled. Builders gain `Craft` evidence. Nothing gives a completion bonus.
- **Organizations own no buildings in M8;** only polities do.

### 7. Style across seven traits

- **Version 3's four style traits** are:
  - opening share;
  - ornament (hours of craft a square metre of front);
  - finish (bare, daubed or lime-washed);
  - symmetry.

  Each costs real hours or goods, and openings weaken walls.
- **Taste moves over seven traits** as it now does over three. Ornament and finish move only
  toward exemplars the household could build.
- **A building is seen from `sight_m` scaled by its height** over the eaves of its neighbours, so a
  hall or a tower is an exemplar from farther away.
- **The settlement panel's three clocks** cover all seven traits.

### 8. The M8 demo and the gate

- **The demo is two settlements founded large,** an organic port by the river and an inland
  settlement whose gathering may pass a plan, at the largest size that lives its span within the
  time box. Plan §7's 30,000 is attempted only if the bench says it fits; otherwise the size
  reached is recorded.
- **The demo measures, within the time box:**
  - who builds a frame, a second storey and a third, and why others did not;
  - which public buildings were proposed, raised or stalled;
  - what fell when the observer shook both settlements alike;
  - which rules followed.
- **Nudges are logged** as `NUDGE:` entries.
- **The scale gate is measured with M8b–M8d in and amended to the figure reached** (ADR-0028 §1).

## Consequences

- **Easier:**
  - buildings of up to four storeys, and halls and towers, can be expressed, checked and saved;
  - an earthquake is a test of the stock as built, with its record and its origin;
  - lessons and rules after a disaster come from what people saw fall;
  - land's cost has a structural answer to weigh against spreading out.
- **Harder:**
  - two spec layouts by version;
  - per-storey wall groups;
  - a lateral mode on every group;
  - an event record with a draw per building;
  - trust kept under more keys;
  - a polity as an owner;
  - another public-work law.
- **Forbidden:**
  - material caps on span or height;
  - a vulnerability class read by behaviour;
  - rerolling one event;
  - natural earthquakes without a fault system;
  - a completion bonus;
  - a storey count pushed by a nudge.

## Alternatives considered

- **Fragility curves by archetype (11-06 §5.2; 03-07 §2.2 A).** Lost: TCE's margins are mechanical
  (ADR-0009 §5). A lognormal curve by class would author the ordering this design means to test.
- **Raising caps by technique (plan §5.6's wording).** Lost to 11-05 §2.1, which rejects caps.
  Techniques open systems instead.
- **An epicentre with falloff.** Lost: no report here gives an attenuation relation for a map this
  size.
- **Monuments for commemoration or worship.** Lost for now: no content gives them a purpose
  (11-10 §1.2).

## Revisit when

- worldgen gains a tectonic source (natural earthquakes, aftershocks);
- a source gives ground amplification by terrain;
- content gives commemoration, burial or worship a mechanism;
- the demo shows no frame is ever built after M8c's land costs, which would make height a
  question of what makes frames worth building rather than of the grammar.
