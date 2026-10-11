# ADR-0022: Fire, floods and storms: hazards from what people do and what the weather brings, damage on parts, rules people propose

Status: Accepted
Date: 2026-10-11
Milestone: M6b

## Context

Through M6a, fire is part of daily life but never a hazard. Households burn firewood every day
(`population::fuel_per_day`), bake at the hearth or in a clay oven (a fixed tool), and fire pots
in an open fire. Nothing they burn can escape. Buildings keep each component group's quality,
loss, state and dates (`civ_land::buildings::GroupCondition`, ADR-0009 §4). A daily check fails
the groups that loads overcome and kills by the collapse shares (`Cause::Collapse`). Caution
remembers failures per settlement and technique, fading by half over 8 years. Plots keep only
`PLOT_GAP_CM` = 100 cm between them, so eaves can stand a metre apart. The hut grammar is frozen
with thatch at 30 kg/m². The frame grammar reserves `WALL_KIND` and `FOOTING` but knows only
wattle and daub on earthfast posts.

The weather (ADR-0012) gives daily rain, temperature, snow and a reference soil bucket, and one
storm a month: a day and a gust load, the same for every roof. There is no daily wind, no wind
direction and no fuel dryness. M6a gave the world one runoff store that scales each reach's mean
discharge to the day's flow (ADR-0021 §1). Walking still fords on the mean, nothing floods, and
earthworks are platforms, pits and spoil, refused beside water (ADR-0010 §3).

M6b's demo is "a great fire leads to a proposed masonry code, its stances' reasons on record, and
any house built in stone under it keeps the law in its provenance" (plan §7). The fire brief
(`docs/briefs/m6-fire.md`) digests 12-03, 12-07, 11-09, 11-06, 11-08, 03-07, 03-02, 11-12 and
12-08. The observer brief (`docs/briefs/m6-observer.md`) adds what the observer needs. Both
briefs' invariants are:

- there is no disaster calendar;
- stages and counts are kept apart;
- damage falls on components;
- people do the work, bounded by the scarcest input;
- there is no material ladder;
- a disaster opens politics but does not pass laws;
- knowledge travels;
- works are kept or they fail.

12-03, 11-09 and 11-08 were read in full for this ADR.

This is expensive to reverse because it adds:

- new weather state: dryness and wind;
- fire records that can be saved mid-fire;
- flood memory;
- new earthwork kinds, a new grammar version and a group kind;
- appended causes, issues, policies, influences, incident kinds and ledger channels;
- a save schema bump.

## Decision

### 1. No hazard calendar

- **Fires start where people use fire.** They start from hearths, ovens and firings, by the
  hour, or from the observer's tool.
- **Floods come from the runoff store and routing.** "Do not trigger a 'century flood' timer"
  (03-02 §1.2).
- **Storms are ADR-0012 §5's monthly storm,** now with a direction.
- **Counts are outputs.** Ignitions, fires that escaped, buildings lost, conflagrations, floods
  and their losses all emerge from the world.
- **Nothing reads them to steer.** Nothing reads a count, the time since the last hazard or the
  world's prosperity to make the next one likelier (02-07 §4 E).
- **Fix the cause, never the outcome.** A village that burns too often is answered through
  response, spacing and materials, never by holding spread below the evidence (the brief's §3).

### 2. The weather gains dryness and a daily wind (amends ADR-0012)

- **Dryness.** One fuel-dryness value per world approaches an equilibrium set by the day's
  temperature and rain, over a time constant of days (a design prior). It is saved as one
  number with the weather. It scales ignition and how readily a covering catches (12-03 §1.5).
- **A daily wind.** Each day has a speed and direction. It is drawn as an AR(1) about a
  prevailing direction that the land profile names.
- **Its own key.** The wind is drawn from its own key, so every rain, temperature and storm draw
  stays as it is (ADR-0011 §3).
- **Storm days.** On the month's storm day, the wind is the storm's. The storm gains a direction
  from its own key.
- **The observer sees both.** The clock and the weather panel show dryness and wind, so a reader
  can see why a fire ran east.

### 3. Fires start where fire is used; most die at once

- **A hazard per fire-hour.** Each fire in use carries a content rate per hour:
  - a hearth while it burns (lit while its people are home and awake, banked at night);
  - an oven while it bakes;
  - a pottery or kiln firing for its hours.

  Dryness scales the rate (12-03 §1.1).
- **The hearth's rate.** At average dryness, the hearth alone gives about 10 ignitions per 1,000
  household-years. This is an order-of-magnitude reference inside 12-03 §2.5's 1–100 prior,
  explored on a log scale, never a target.
- **Keyed draws.** Draws are keyed by building and hour, so the same day lived at any speed
  starts the same fires.
- **Most ignitions die at once.** An ignition with someone awake in or beside the building is
  put out at once. Only the kernel's count records it (12-03 §1.2's first counter).
- **Otherwise it grows.** An ignition with nobody awake there catches the covering.
- **Not in M6b:** lightning (no source gives a rate) and arson (it belongs with force and war,
  M6c).

### 4. A building burns as parts; fire spreads by exposure

- **Stages.** A building's fire passes from its covering alight, to fully involved, to burnt
  out. It consumes its covering, its timber and the combustible goods under it at authored
  rates.
- **Stepping.** While anything burns, it is stepped by one-minute events in every mode. ADR-0011
  §2 processes the same events in Accelerated mode, and a shout is a consequential boundary that
  ends a leisure block. A test at half a minute checks that results converge. Nothing is stepped
  while nothing burns (12-03 §5.5).
- **Exposure.**
  - Every building within a reach of a burning one (10 m, a tuning value) keeps an exposure
    store. It rises with heat and falls as the building cools or is wetted (12-03 §1.4).
  - Heat falls with the gap between footprints, as 12-03 §2.2's 36 kW/m² at 1 m and 5 kW/m² at
    4 m do.
  - The wind stretches the reach downwind. Firebrands are folded into that stretch for thatch,
    as one route.
  - A covering catches by a hazard on its exposure above a threshold by kind: thatch low, tile
    only at its eaves and openings, a mass wall never. Dryness scales the threshold.
- **The exposure graph is derived.** It is rebuilt only when buildings change (12-03 §5.1).
- **R_fire is a diagnostic, never a rule** (12-03 §4). There is "no universally safe
  three-meter firebreak" (§1.6).

### 5. What fire does falls on parts, goods and people; nothing is restored

- **Coverings.** A burning covering is lost, and the roof opens.
- **Timber.**
  - While a building is fully involved, its exposed timber chars at 11-08 §2.5's rate: about
    0.8 mm/min at first, about 0.6 mm/min later.
  - Char is loss of section in `GroupCondition`, and the daily load check fails what it
    weakens, by ADR-0009 §5's rules.
  - Wattle burns and daub stays.
  - A mass wall cracks after a full burn (an authored loss: brick and stone "crack and lose
    strength", 11-08 §2.5).
- **Goods.** Goods under a burning roof burn at authored shares by good, on an appended ledger
  channel. Goods carried out survive.
- **People.**
  - Those asleep inside and never woken die of an appended cause, `Burned`.
  - Those under a roof or posts that give way die by the existing collapse shares.
  - Burned area is not a proxy for deaths (12-03 §4).
- **Nothing is restored.** Putting a fire out stops further burning and restores nothing burned
  (12-03 §5.4). A second hazard acts on what the first left (03-07 §5.2).

### 6. People notice, shout and choose; water is carried, never conjured

- **Who notices:**
  - those awake inside notice at once;
  - sleepers wake after a draw in 12-03 §2.5's 5–30 minutes;
  - anyone within sight notices;
  - the watch sees only what its rounds pass.
- **The shout.** Whoever notices shouts. Those within earshot hear, their activity ends, and
  they decide again, as walkers do when a crossing gives way (12-07 §1.3: notification is
  physical).
- **Choices, scored as any.** Each person who knows chooses one task at a time: wake and lead
  out their household, carry water, strip a threatened roof, carry goods out, pull down a
  building, or keep away (12-03 §5.3).
  - Kin, ties, their own property's exposure and the danger weigh as in other choices.
  - Nobody is ordered, and there is no brigade office in M6b (12-07 §4).
- **Water is conserved** (12-03 §1.7, §5.4).
  - Carriers draw first on household stores, then walk to the nearest source with 5–10 L
    vessels.
  - Water applied cools exposure stores. Above an authored rate, it puts out a burning
    covering.
- **Pulling down** costs the parts it pulls down, even when it works (12-03 §1.6). A person may
  pull down:
  - their own household's building;
  - another household's with leave from one of its people who is there.

  No law grants the power in M6b.

### 7. What people know of a fire is their own

- **Records.** A fire is the kernel's truth (ADR-0015 §1). Seeing it, being told of it and
  coming to it are records of people. Residents may know only that a building "was found
  burning" (12-03 §1.2). Its cause is known only to someone who saw it start.
- **Fire caution.** Each settlement remembers buildings lost to fire by covering and wall kind,
  against building-years, fading as caution does (ADR-0009). A household's design weighs the
  loss it believes likely, so tile and stone can spread with no law at all.
- **Reading the truth.** No forecast reads the kernel's cause of a fire, a river's stage beyond
  what was seen, or future weather (03-03 §5.2).

### 8. Rivers rise and fall; water stands where routing puts it (amends ADR-0021 §1)

- **Stage and extent.**
  - Each reach's stage follows Manning's relation on a hydraulic-geometry channel, from the
    day's flow (ADR-0021 §1).
  - Water above bankfull fills that reach's floodplain compartment through a storage curve. The
    curve is precomputed from its cells' height above drainage and derived, never saved. Extent
    is therefore set by volume and connection, never by a radius (03-02 §1.2).
  - Steps are daily. Flash floods wait.
- **Fords follow the day's flow.** Walking moves from the mean discharge to the day's flow, so
  fords open and close. Cells under water deeper than wading are not walkable that day.
- **Effects** (03-07 §1.1):
  - Water above a building's floor spoils the goods on that floor. Raised stores and lofts keep
    theirs.
  - It wets daub and earth walls and soaks post feet, through the existing wetness wear.
  - A field under water for days loses a share of its standing crop (a design prior: no report
    gives emmer's).
  - A crossing over a reach in flood is checked for scour by a margin its bridge system authors
    (11-06 §1.2; ADR-0009 §9 left scour here).
- **People leave water.** A household whose floor is wet goes to kin or the hearth until the
  water falls. There is no drowning in M6b: deep cells cannot be walked into.
- **Memory.** Each household remembers the deepest water at its home and fields, fading as
  caution does. Saves keep each building's and field's last flood: its peak depth and days wet.

### 9. Storms gain a direction and tear coverings

- **The world's storm.** ADR-0012 §5's monthly storm stays the world's storm. It gains a
  direction, and its wind drives spread on its day.
- **Uplift.** A covering held by its quality is torn off above an authored load (03-07 §2.2 C:
  q = ½ρV²; 11-05 §5.4). The roof then leaks, and goods spoil by the existing rules. Roof loss
  increases rain damage (03-07 §1.1).
- **One storm, one evaluation.** It is never rerolled (11-06 §5.2).

### 10. Masonry, brick and tile are assemblies, not a ladder (amends ADR-0009)

- **Content, not code, for the vocabulary** (11-08 §1.1):
  - goods: mud brick, brick, roof tile and quicklime;
  - a kiln that stays where built, as the oven does;
  - techniques for walling in stone, moulding and laying mud brick, firing brick and tile, and
    burning lime.

  They are learnt and found under ADR-0008.
- **What founders bring.** Founders bring rubble walling and mud brick: stone construction and
  sun-dried earthen units are documented in Neolithic communities (07-05 §3.1). Kiln firing and
  lime burning come by finding, contact or the knowledge tool.
- **Batches.**
  - A firing is a batch with fuel per firing, a share broken and its days (11-08 §2.2).
  - Green bricks dry as elapsed time. A wet day spoils a share (a design prior).
- **Grammar.**
  - Frame version 2 gives `WALL_KIND` 1–3 (rubble, mud brick, fired brick) as load-bearing
    walls on a stone `FOOTING`, and a covering kind (thatch or tile).
  - An appended `GroupKind::MassWall` is checked by height over thickness and by crushing.
    Assemblies are rubble at 1.0–2.0 MPa and solid brick in lime at 2.6–4.3 MPa, never unit
    strengths (11-05 §2.3; 11-08 §2.1).
  - It wears at its foot by wetness, as daub does, and weakens while wet (11-08 §5.2).
  - A tile covering at 65 kg/m² is carried by the rafters under the existing checks
    (11-08 §4).
  - The hut stays frozen.
- **Cost emerges from the chain** (11-08 §1.4): digging, moulding, fuel, firing, carrying and
  laying. No material is better in every respect. "Noncombustible walls do not make a building
  fireproof" (11-06 §1.2; 11-08 §2.5).

### 11. Banks and ditches are lines; drainage changes openly (amends ADR-0010)

- **Earth kinds.** `Bank` and `Ditch` are appended as lines of segments below the grid
  (11-12 §5.1).
  - A bank segment keeps its crest, settlement, erosion and breach state.
  - A ditch keeps its cross-section, gradient and sediment.
- **Water and banks.** Water reaches cells behind a bank only over its crest or through a
  breach. Overtopping erodes a segment by depth and duration. Piping grows with days of head
  against its quality.
- **Works are kept or they fail.** Nobody mends a polity's bank unless a law asks (12-08 §1.4).
  A neglected bank is a finding, not a bug.
- **Intakes and drainage.** ADR-0010 §3's refusal beside water is relaxed for a ditch's intake.
  Each ditch records the water it takes from its reach, so drainage is never "changed silently
  by an earthwork".
- **Irrigation ditches only.** Drains wait until fields can waterlog, and terraces until slopes
  are farmed (ADR-0012 §7 holds).

### 12. Building rules are laws people propose; they bind new work

- **Issues** (appended):
  - `burned`: a household lost a building to fire, or its members saw one burn, within memory;
  - `gave_way`: they saw or heard of a building giving way;
  - `flooded`: their home, store or field stood in water.

  Each opens moves and weighs toward none (ADR-0013 §5).
- **A policy kind, `BuildingRule`** (appended). Its templates are content. Each names either a
  property or a technique (11-09 §5.2). M6b's templates:
  - new roofs within a set distance of another building must not burn;
  - new walls must be of named kinds;
  - new buildings must keep a least gap.
- **When a rule binds.** It binds buildings begun from the day it takes effect. "Never rebuild
  the entire city when a law passes" (11-09 §1.7), so old stock stays, and a patchwork is legal
  (11-09 §4).
- **A policy kind, `BuildBank`** (appended), shaped like `BuildCrossing`. Its work is asked in
  equal shares.
- **Who proposes.** Someone who holds the issue and knows a technique the rule needs proposes it
  when their household's forecast gains (11-09 §1.4: remedies come from known techniques).
  - The forecast weighs the loss it believes avoided against what its next building would cost
    under the rule.
  - A household that builds nothing soon pays nothing yet.
  - Attribution comes from what people saw. A fire nobody saw start is blamed on where it was
    first seen burning.
- **Compliance is chosen.** It weighs the norm that the gathering binds, with no sanction in
  M6b, as for `Curfew`. A rejected stringent rule followed by an affordable partial one is an
  outcome to welcome (11-09 §1.5).
- **Provenance.** A building's provenance names the rule it was built under, derived from its
  start day and the law's history. Otherwise it names the caution and taste that moved its
  builders.

### 13. Incidents, episodes and counts (ADR-0015 §8)

- **Incident kinds** (appended):
  - a building's fire, from ignition to out;
  - high water at a building or field;
  - storm damage to a building.
- **Counts kept apart** (12-03 §1.2):
  - Ignitions put out at once are counts per settlement and month, never incidents.
  - Fires that escaped, buildings lost and conflagrations are counted apart.
- **Episodes.** ADR-0015 §8 leaves when an episode opens and closes to the ADR that brings its
  kind:
  - a **conflagration** opens when a fire reaches a second building, and closes when the last
    of its fires is out;
  - a **flood** opens when a reach stands above bankfull over a building or field, and closes
    when it falls below;
  - a **storm** episode is the storm's day, when it damaged anything.
- **Typed links, recorded when they happen.**
  - `caused_by` links a fire to its fire in use (a hearth, an oven, a firing), or to the burning
    neighbour that exposed it (with the gap, wind and dryness at the time), or to the
    observer's `Influence`.
  - It links a failure or torn covering to the storm or flood.
  - `part_of` links an incident to its episode.
- **Why not.** A building weighed by the spread keeps its peak exposure. A building never
  weighed reads "not weighed", never a reason found afterwards (15-03 §1.2).
- **No choice reads any of these.** This is tested as for ADR-0015 §8.

### 14. The observer's fire, flood and storm

- **Fire:** a building's covering alight now.
- **Flood:** a reach's runoff raised by a stated amount for stated days. Routing decides what
  floods.
- **Storm:** a storm today of a chosen strength and direction. The world's own storm series is
  unchanged.
- **Each runs the ordinary machinery** (03-07 §1.3). Each is an `Influence` of an appended kind,
  and everything it starts carries that origin.
- **People cannot tell.** They remember a god's fire or flood as any other.
- **Statistics leave them out by origin:** return levels, the dashboard and Gate B.
- **The preview** states only the certain effect and what the kernel knows of the situation (who
  is home, today's wind, the nearest water). It never forecasts losses.

### 15. The boundary

- **Saves** gain:
  - dryness and the wind's state;
  - every fire burning (stage, fuel left, exposure stores, who is fighting it), so save → load →
    save is exact mid-fire;
  - fire, high-water and storm incidents and their episodes, and the monthly counts;
  - flood memory;
  - banks and ditches, mass walls and tile coverings.

  The schema is bumped. A save from before loads with no fires, no floods, dryness at its
  equilibrium and the wind drawn.
- **Content** gains:
  - fire kinds and their rates;
  - each covering's and wall's fire properties;
  - the masonry goods, the kiln and its techniques;
  - the wind profile;
  - the `BuildingRule` templates.
- **The wire** shows:
  - burning and burnt buildings;
  - wind and dryness;
  - flood water as a kernel raster of depth by day, sparse cells sent when they change (the
    observer brief's question 9);
  - the *Help at a fire* lens;
  - fire, high-water and storm incidents and their pages;
  - the three tools.
- **Gate B** (ADR-0011 §5) gains fire and flood aggregates with wide tolerances.

## Consequences

- Easier:
  - a great fire is explained house by house, each step with its gap, wind and dryness, beside
    who came and with how much water;
  - a masonry code comes from who lost what and who knows stone, and its houses keep the law in
    their provenance;
  - M6c's sieges find fire, water and walls already physical.
- Harder:
  - fires add minute stepping while they burn, and an exposure graph;
  - floods add a daily routing step per reach above bankfull;
  - Gate B must cover rare, heavy-tailed events, so their tolerances stay wide;
  - calibration must go in 12-03 §5.7's order: ignition and reporting, one building, pairs,
    task times and water, and only then settlement losses.
- Forbidden:
  - a fire, flood or storm drawn from a calendar, a target rate, prosperity or the time since
    the last;
  - a single "fire health" per building;
  - water applied that was never carried;
  - putting a fire out that restores what burned;
  - a law that rebuilds old stock, enforces itself, or passes because a disaster happened;
  - any choice reading an incident, an episode, the kernel's cause of a fire, or future weather.

## Alternatives considered

- **A fire risk per building per year.** There is no defensible rate (12-03 §2.1), and it would
  be a calendar. Fires in use carry the hazard instead.
- **A fire-protection ladder** (plan §5.4: none, bucket brigade, watch, engines, hydrants).
  12-03 §3.4 says this is "not a mandatory technological sequence", and 12-07 §4 favours
  combinable institutions. M6b has households and neighbours; offices come when towns need
  them.
- **Spread by distance alone, or a fixed firebreak.** Exposure must accumulate (12-03 §1.4), and
  no gap is a guarantee (§1.6).
- **Floods within a radius of the river.** Extent comes from volume, connection and storage
  (03-02 §1.2).
- **Codes only from a planning institution** (plan §5.1, M8). M6b needs a rule without one.
  Polities already pass laws by their custom, and 11-09 §3 starts early farming villages from
  households, shared obligations and customs.
- **A material ladder** (wattle, then brick, then stone). 11-08's executive conclusion: chains
  and assemblies.

## Revisit when

- Towns need brigades, pumps, engines or fire watches as offices (M7–M8; 12-03 §3.4).
- A source gives a lightning rate, or war brings arson (M6c).
- Courts and inspectors can enforce a rule (M7; 11-09 §1.6), or planning institutions arrive
  (M8).
- Fields can waterlog or slopes are farmed (drains, terraces).
- Earthquakes arrive (M8): masonry's vulnerability is their cost (03-07 §2.2 A).
- Firebrands need a route of their own, or vegetation burns (wildfire).
