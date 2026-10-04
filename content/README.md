# Content

Authored primitives: the engine's **vocabulary**, never its plot (plan §1). Content says what *can*
exist and how it behaves; which of it a world uses, and when, is up to the simulation and the
people in it.

## Layout

```
content/
  <pack>/                 one directory per pack; `core` is an ordinary pack
    pack.toml             identity and compatibility
    <kind>/<name>.toml    one definition per file; the path must match the id
```

Every definition has `kind` and `id`. Ids are `pack:kind/name` with lowercase `[a-z0-9_]` segments,
for example `core:worldgen/river_valley`. Unknown fields are errors, and so are missing ones: a
misspelling must never silently become a default.

## Validating

```sh
cd kernel
cargo run -p civ-host -- content validate          # human-readable
cargo run -p civ-host -- content validate --json   # for tools and agents
```

CI runs the validator. Diagnostics have stable codes:

| Code | Meaning |
|---|---|
| E1001 | Not valid TOML, or a field is unknown, missing or mistyped (reported with line and column) |
| E1002 | A pack directory has no `pack.toml` |
| E1003 | The pack needs a content schema or kernel content API this build lacks |
| E1004 | A file could not be read |
| E2001 | An id is not `pack:kind/name` (or a pack id does not match its directory) |
| E2002 | An id names a different pack from the one it is in |
| E2003 | An id's kind segment does not match the file's `kind` |
| E2004 | The file's path does not match its id |
| E2005 | Two definitions share an id |
| E2006 | A reference names something that is not defined (a people profile's name list, provisions good, crop or home program, an activity's land resource, recipe or tool, a resource's, crop's or building program's good, a recipe's good, tool or skill, a good's `reserve_for`) |
| E3001 | A value is out of its allowed range |
| E3002 | Unknown or missing `kind` |
| E3003 | Not exactly one world-generation preset has `default = true` |
| E3004 | Not exactly one people profile, or not exactly one land profile |

**Fingerprints.** Each pack gets an *artifact* fingerprint, a BLAKE3 hash of its files' bytes. The
whole set also gets a *semantic* fingerprint, a BLAKE3 hash of the effective compiled values. Saves
record the semantic one, so reformatting a file, reordering keys, writing `3` for `3.0` or adding a
comment does not mark saves as "content changed"; changing a value does.

A world saved with other content still loads and runs by the content loaded now. Saves refer to
activities, habitats, land resources, goods, crops and skills by id: a person whose activity is gone decides
again, a habitat that is gone becomes the last (catch-all) habitat, a good that is gone is dropped
from stores and loads, a field whose crop is gone is dropped, and a new resource starts at
equilibrium. So does a resource whose `good` or `unit_kg` changed, since its saved stocks may count
something else.

## Kinds (M0)

### `worldgen_preset`

A kind of landscape the new-world dialog offers. The seed decides where its mountains, rivers and
coasts actually go. Exactly one preset is the default. Every parameter must be present.

| Table | Field | Unit | Meaning |
|---|---|---|---|
| `region` | `context_factor` | × | Size of the simulated watershed relative to the map (1–4). Rivers can enter from it. |
| | `outlet_edges` | count | Context edges at base level, 1 or 2 adjacent; the seed picks which. |
| | `base_level_m` | m | Elevation where water leaves the region; negative puts the outlet under the sea. |
| | `initial_relief_m` | m | Low-frequency relief of the starting surface. |
| | `regional_slope` | m/m | Starting tilt toward the base-level edges. |
| | `trunk_inflow_km2` | km² | Catchment of a large river entering from beyond the region (0 = none). |
| | `window_offset` | 0–1 | Map position in the watershed: 0 centred, 1 against the base-level edge. |
| `uplift` | `uplift_mm_per_yr` | mm/yr | Peak rock uplift; builds mountain ranges. |
| | `lowland_uplift_fraction` | 0–1 | Uplift away from ranges, as a fraction of the peak. |
| | `range_scale_km` | km | Spacing of mountain ranges. |
| | `basin_strength` | 0–1 | Subsiding pockets that can become basins (0 = none). |
| `erosion` | `erodibility` | m^(1−2m)/yr | Stream-power K (drainage area in m²). |
| | `erodibility_variation` | 0–0.9 | Spatial variation of K: rock of differing resistance. |
| | `area_exponent` | – | Stream-power m (n = 1). |
| | `channel_initiation_km2` | km² | Below this drainage area, ground is hillslope only (0 = off). |
| | `diffusivity_m2_per_yr` | m²/yr | Hillslope soil creep. |
| | `talus_slope` | m/m | Steepest stable loose slope. |
| | `endorheic_depth_m` | m | Deeper basins keep their water during evolution (0 = all overflow; arid presets). |
| | `coarse_iterations` | count | Landscape-evolution steps on the coarse grid. |
| | `coarse_dt_years` | yr | Length of each coarse step. |
| `refinement` | `fine_iterations` | [count; 3] | Erosion passes at 4×, 2× and 1× the simulation cell. |
| | `fine_dt_years` | yr | Length of each fine step. |
| | `detail_amplitude` | × relief | Added roughness, scaled to local relief. |
| | `routing_jitter` | 0–0.9 | Perturbation that keeps rivers from running in straight 45° lines. |
| | `floodplain_min_area_km2` | km² | Rivers draining this much get a flat valley floor. |
| | `floodplain_width_factor` | × channel width | Floodplain half-width (0 = no floodplains). |
| `water` | `sea_level_m` | m | Sea level. |
| | `precipitation_mm_per_yr` | mm/yr | Mean annual precipitation. |
| | `evapotranspiration_mm_per_yr` | mm/yr | Evapotranspiration from land; runoff = precipitation − this. |
| | `lake_evaporation_mm_per_yr` | mm/yr | Evaporation from open water; decides whether a basin's lake overflows. |
| | `channel_area_km2` | km² | Drainage area where a river reach begins. |
| | `min_lake_depth_m` | m | Shallower depressions are filled as terrain artifacts. |
| | `min_lake_area_m2` | m² | Smaller depressions are filled as terrain artifacts. |
| | `channel_width_coefficient` | m/(m³/s)^0.5 | `a` in channel width `w = a·Q^0.5`. |

**Shipped presets:** `river_valley` (default) and `ria_coast`. Their values are tuning starting
points, checked against figures `civ-host` prints (relief, share of gentle land, discharge). They
are not calibrations. An arid closed-basin preset is planned for M3, once climate exists.

## Kinds (M1)

M1 adds seven kinds (kernel content API 6). Every field is required, and every number in the
shipped files carries its research source or says it is a tuning starting point.

### `people`

How a world's people live. Exactly one profile. `names` is the id of a name list.

| Table | Fields | Meaning |
|---|---|---|
| (top) | `latitude_deg` | Latitude for day length (−66 to 66). |
| | `walk_speed_by_age`, `capacity_by_age` | `[[age, factor], …]`, ascending: walking speed and work efficiency by age. |
| `walking` | `top_speed_kmh`, `slope_sensitivity`, `best_slope_offset` | Tobler's hiking function: speed peaks at `top_speed_kmh` on a slope of −`best_slope_offset`. |
| | `offtrail_factor`, `wading_factor`, `ford_max_discharge_m3s`, `max_slope` | Speed off worn trails; speed wading; the largest river that can be waded; the steepest walkable slope. |
| `energy` | `bmr_band_starts`, `bmr_male`, `bmr_female` | Schofield basal metabolism: `[[kcal/kg/day, kcal/day], …]` per age band. |
| | `mass_by_age` | `[[age, male kg, female kg], …]`. |
| | `walk_par`, `idle_par` | Physical activity ratios while walking and between activities. |
| | `satiety_hours`, `hunger_ramp_hours`, `deficit_unit_kcal`, `max_surplus_kcal`, `meal_minutes` | How long a meal keeps someone full, how hunger rises after, how an energy deficit adds to hunger, the largest banked surplus, a meal's length. |
| | `reserve_kcal_per_kg` | What the body can draw on in a shortage, per kilogram of body mass: the floor of the energy balance. The share of it drawn is what hunger's hazards and its effect on conception follow. |
| | `eat_reserve_at_deficit` | Food kept back (seed) is eaten only once a person has drawn this share of that reserve, and never the seed to sow again the ground the household already crops. |
| `sleep` | `tau_awake_h`, `tau_asleep_h`, `wake_pressure` | Two-process sleep pressure: rise awake, fall asleep, the level a sleeper wakes at. Sleep is not an option below it. |
| | `min_hours`, `max_hours`, `nap_min_minutes`, `nap_max_minutes` | A night's sleep and a daytime nap. |
| | `day_factor`, `bedtime_after_sunset_hours` | Weight of sleep pressure by day, and when the evening's sleep gate opens. |
| `social` | `tau_h`, `quality_per_companion`, `household_quality` | Relatedness eases toward the quality of present company. |
| `household` | `water_l_per_person_day`, `carry_water_l`, `water_target_days` | Water use, what one trip carries, the store people aim for. |
| | `food_target_days`, `carry_kg`, `daily_kcal_per_person` | The food store people aim for, the load one person carries home, average need. |
| | `harvest_margin_days` | The food a household wants in store beyond the first grain of its next harvest: stores short of that much make wild food worth going out for (`decision.w_lean`). |
| | `fuel_kg_per_person_day`, `fuel_target_days` | Firewood burned per person per day by month (January first), and the store people aim for. |
| | `short_food_days`, `recovered_food_days` | The chronicle notes a settlement's shortage below the first and its end above the second. |
| | `leave_at_depletion`, `leave_per_day`, `leave_unless_ripe_within_days` | A household with less than a day's food, whose members have drawn on average this share of their reserve, and with no crop of its own ripening within these days or reaped and waiting, gives up and leaves with this chance a day. |
| `decision` | `temperature_sd_fraction`, `min_temperature` | Softmax temperature: a fraction of the spread of the acceptable options' scores, with a floor. Only options worth more than doing nothing (a positive total) are sampled, unless none is. |
| | `w_*`, `trip_half_worth_days` | Points per unit of each consideration (hunger, sleep, company, food, firewood and water shortage, useful work, walking, effort, darkness, rest, play, field work's harvest, shelter, stores that will not last to the next harvest (`w_lean`), and a deadline's pressure); a trip bringing `trip_half_worth_days` of the household's need is worth half a very large haul. Beyond any shortage, more of a good is worth less the more of it is in store. `w_shelter` is what a session of building, or a load of what the household still needs for its roof, is worth. |
| `band` | `default_size`, `min_size`, `max_size`, `min_families` | The founding band the new-world dialog offers. `min_size` is at least twice `min_families`. |
| | `camp_candidates`, `site_radius_m`, `site_max_slope`, `site_w_*`, `site_flood_hand_m` | How the band scores sampled camp sites: wild food within the radius, land it could crop within a field walk (ground to be cleared counts for less), distance to fresh water, slope, flood risk. |
| | `provisions_days`, `provisions_good`, `seed_kg_per_person` | The food each family brings, in days of its needs, and the good (a food) it is carried as; the seed it brings for the crop it grows. |
| | `elder_chance`, `young_adult_chance`, `birth_spacing_months` | Who is in the families. |
| `farm` | `crop` | The crop households grow (a crop id). |
| | `grain_share`, `plan_yield_share` | The share of a year's food a household plans to grow, and the cautious yield it plans on (a share of the crop's average). |
| | `loss_share` | The share of the grain grown that is lost before it is eaten (in store, at the quern, as bread and flour go off): a household grows its food over one less this. |
| | `grain_target_days` | Grain held at which another harvest is worth half as much. |
| | `work_hours_per_day` | Field work a capable adult gives a day, for planning what the household can prepare and sow in a season. |
| | `field_m`, `max_walk_minutes`, `site_candidates` | New fields are squares of `field_m`, within this walk of the settlement; how many places are looked at when marking one out. |
| `build` | `home_program` | The building program households build their homes to (a building id). |
| `mortality` | `a`, `b`, `c`, `d`, `e` | Siler hazard `A·e^(−Bx) + C + D·e^(Ex)` per year: the founders' ages, and everyone's daily risk of dying. |
| | `hunger_ratio_at_half`, `hunger_ratio_max` | Hunger multiplies that hazard: by the first at half the reserve drawn, as `ratio^(4d²)` of the share `d` drawn, up to the second. |
| | `exhaustion_per_day`, `exhaustion_power` | A body at the end of its reserve dies with this chance a day, scaled by the share drawn to this power. |
| | `maternal_death_per_birth` | A mother's chance of dying in childbirth. |
| `fertility` | `conception_per_month`, `age_factor`, `fecundity_sd`, `hunger_halving` | A woman living with her partner conceives with this chance a month at her most fecund ages, times `[[age, factor], …]`, a lasting lognormal factor of her own (spread of its log) and a hunger factor that halves for each `hunger_halving` of the reserve drawn. |
| | `pregnancy_days`, `pregnancy_sd_days`, `loss_by_age`, `loss_days` | A pregnancy's length; the chance it is lost by the mother's age (`[[age, chance], …]`), and when after conception a loss comes (`[earliest, latest]` days). |
| | `recovery_months`, `recovery_sd_months`, `recovery_min_months`, `loss_recovery_months`, `weaned_recovery_months` | How long after a birth (or a loss) a mother cannot conceive, and how soon she can after her nursing child dies. |
| | `boys_per_100_girls`, `pregnancy_kcal_day` | The sex ratio at birth; what a pregnancy costs its mother a day, by trimester. |
| `family` | `seek_min_age_female`, `seek_min_age_male`, `seek_max_age_female`, `seek_max_age_male`, `seek_per_month_female`, `seek_per_month_male` | Between these ages, an unpartnered woman or man looks for a partner with this chance a month. |
| | `age_gap_years`, `preferred_gap_years`, `w_gap_per_year` | How much older a man may be than his partner (`[least, most]`, negative for younger), the gap people look for, and the points a candidate loses for each year away from it. |
| | `kin_exclusion_generations` | No couple shares an ancestor within this many generations, or has one partner descend from the other within them. |
| | `residence` | Where a new couple lives: `new_household`, `his_household` or `her_household`. Someone who keeps a household with no other adult is joined there whatever the rule. |
| | `independent_age`, `trait_heritability` | From this age someone can keep a household; how much of a child's personality regresses on the mean of its parents'. |

### `land`

How land is classified and what grows wild. Exactly one profile.

| Field | Meaning |
|---|---|
| `patch_cells` | Patch side in terrain cells (16 = 128 m at 8 m cells). |
| `channel_area_km2` | Drainage area that counts as a stream when measuring height above the nearest stream. |
| `richness_min`, `richness_max`, `richness_feature_m` | Patch-to-patch variation of productivity and its spatial scale. |
| `climate_cv`, `climate_autocorrelation` | Year-to-year variation of production. |
| `[paths]` | `wear_per_walk`, `wear_half_life_days`: a walk across an 8 m cell wears away this share of what is left unworn, and unused wear halves in this many days. `trail_at`, `trail_until`: a cell becomes trail at the first wear and stays trail until it fades below the second. |
| `[[habitat]]` | `id`, `name`, `arable`, and optional `min_water_fraction`, `max_median_hand_m`, `max_mean_slope`. The first habitat whose conditions a patch meets is its habitat; the last must have none. Arable ground can carry fields; optional `clear_h_per_ha` is the work to clear it (woodland) before it is first broken. |
| `[[resource]]` | `id`, `name`; `good` (the good a harvest yields) and `unit_kg` (its kilograms per unit of stock: 1 for stocks in kilograms, a carcass's meat for stocks in animals); `discrete` (harvests are whole units drawn from the expected count); `in_water` (lives in a patch's water, not its land); `range_patches` (a trip works a block of `2·range + 1` patches a side); `max_rate_per_hour` and `half_rate_stock_per_ha` (gathering slows as the stock falls). Then exactly one growth table. |
| `[resource.plant]` | `production_per_ha_yr` (one figure per habitat, in habitat order), `loss_per_day`, `season` (twelve monthly weights): a seasonal production each day, a share of the standing stock lost each day. A new world's stock, and what people expect of land they have not worked, is the yearly cycle this settles into: what stands lags what grows by about the time it lasts (last summer's reeds still stand in March). |
| `[resource.animal]` | `capacity_per_ha` (per habitat), `growth_per_year`, `spread_per_month`: logistic growth toward the habitat's capacity, and a monthly spread between neighbouring patches toward an even share of capacity. |
| `[resource.deposit]` | `stock_per_ha` (per habitat): a stock laid down once, that never grows back (stone, flint; M3a). |

Stocks change daily; nothing respawns. What people learn about a place fades over the time its
resource takes to renew: `1 / loss_per_day` days for plants, `365 / growth_per_year` for animals,
ten years for deposits.

### `good`

Something people carry home and keep.

| Field | Meaning |
|---|---|
| `purpose` | `food`, `fuel`, `material` (built with, or made into something) or `tool` (M3a). |
| `kcal_per_kg` | Food energy; 0 for anything else. |
| `half_life_days` | Days for half a stored amount to spoil; 0 keeps. Fuel must keep. |
| `sheltered_half_life_days` | The same in a household's store under its own roof; 0 when a roof makes no difference. Never shorter than `half_life_days`; a good that keeps needs none. |
| `eaten` | `raw` (as it is), `cooked` (needs a fire: not eaten while the household has no firewood; cooking adds no energy, research 05-02 §1.1) or `never` (a recipe must make it food first, as grain is ground or pounded). Anything but food is `never`. |
| `shared` | When brought home it is shared among every household of the settlement, by members. |
| `reserve_for` | The food it is kept back from, like seed from grain, or `""`: a recipe that needs that food takes this one only in real hunger, and never the seed to sow the ground already cropped. |
| `[tool]` | For a tool only: `life_h`, the hours of use a standard tool lasts (stores count tools in standard tools, so 2.4 sickles are two and what is left of a third, ADR-0006); `per_adult`, how many a household wants for each member of working age (rounded up; 0 for none); `fixed`, it stays where it was made and is never carried off. |

People eat the most perishable food that can be eaten first. A material or tool is never eaten,
kept back or shared: a household brings it for what it builds and makes, and only as much as it
still needs. A tool is worn by the hours of the work that needs it, and gone when worn out.

### `crop`

A plant people sow, tend and reap by hand. Which fields are sown, by whom and when, is decided by
people at run time.

| Field | Meaning |
|---|---|
| `good`, `seed_good` | The good the harvest yields and the good its seed is kept as (both good ids). |
| `seed_kg_per_ha`, `yield_kg_per_ha` | Seed sown, and clean grain from average ground in an average year with timely work. |
| `prepare_from_day`, `sow_from_day`, `sow_until_day` | The calendar (days of the year from 0): ground can be prepared from the first, sown between the other two. |
| `late_sowing_loss_per_day` | Share of the yield lost for each day sowing finishes after `sow_from_day`. |
| `grow_days` | Days from sowing to ripeness; the crop must ripen within the year it is sown. |
| `standing_loss_per_day` | Share of the ripe crop lost for each day it stands unreaped; it is all gone in the end. |
| `untended_loss` | Share of the yield weeds take from a crop nobody tends. |
| `break_h_per_ha`, `prepare_h_per_ha`, `sow_h_per_ha`, `tend_h_per_ha`, `reap_h_per_ha` | Person-hours of a capable adult per hectare: breaking new ground, preparing cropped ground again, sowing, tending over the season, reaping and carrying home. |
| `thresh_h_per_kg` | Person-hours to thresh and clean a kilogram of grain. |
| `straw_good`, `straw_kg_per_kg` | Optional, both or neither: the good threshing leaves as straw (thatch, for the shipped crop), and kilograms of it per kilogram of grain. |

What a harvest brings depends on the field's ground (the richness of its patches), the year's
weather, when sowing finished, how much of its tending was done, and how long it stood ripe.

### `building`

A building program: what it is made of, the work and material each part takes, and the dimensions
people build it to. The grammar that expands a design into its parts is code (`civ-grammar`;
`grammar = "hut"` is the only one in M1). Who builds what, where and when is decided by people at
run time.

| Table | Fields | Meaning |
|---|---|---|
| (top) | `grammar` | The grammar that expands it: `hut`. |
| | `eave_cm`, `pitch_deg` | The wall height and roof pitch people build to, within the rules' ranges. |
| | `roof_by_day` | The day of the year (from 0) a household wants to be under its roof by; the pressure to build grows as it nears. |
| `materials` | `timber`, `wattle`, `thatch` | The good each material slot is made of (good ids, each a `material`). |
| `rules` | `radius_cm`, `eave_cm`, `pitch_deg` | Allowed ranges, `[least, most]`. |
| | `floor_base_m2`, `floor_m2_per_sleeper` | The floor a household needs whatever its size and per resident; the hut's radius follows, rounded up to whole decimetres. |
| | `post_spacing_cm`, `post_diameter_cm`, `posthole_depth_cm`, `wall_thickness_cm`, `roof_overhang_cm`, `thatch_thickness_cm`, `hearth_cm` | Dimensions. The plot a household claims is the square its roof covers. |
| | `groundwork_h_per_m2`, `posthole_h`, `post_h`, `rafter_h`, `wattle_h_per_m2`, `daub_h_per_m2`, `thatch_h_per_m2`, `finish_h_per_m2` | Person-hours of a capable adult for each part: foundation (groundwork and postholes), frame (posts and rafters), walls (wattle and daub), roof (thatch), finish (floor and hearth). All positive. |
| | `post_kg`, `rafter_kg`, `wattle_kg_per_m2`, `thatch_kg_per_m2` | Material in each part. |

A stage uses its materials in proportion to its work and waits when they run out; the last half
kilogram of a material is made up from scraps. Under its roof, a household keeps its stores at
their sheltered half-lives.

### `names`

`male`, `female`: given names; `place_first`, `place_second`: parts joined into place names. The
shipped list is invented and implies no real culture.

### `activity`

What people can do: an engine behavior with its numbers. Which activity anyone does, and where, is
decided by people at run time.

| Field | Meaning |
|---|---|
| `behavior` | One of `sleep`, `eat`, `fetch_water`, `gather`, `socialize`, `rest`, `play`, `farm`, `ask`, `build`, `make` (M3a). |
| `resource` | For `gather` only: the land resource gathered (hunting, fishing, collecting firewood and cutting building materials are gathering too). A trip works until its load is full or its time runs out. |
| `task` | For `farm` only: the field work, one of `prepare`, `sow`, `tend`, `reap`, `thresh`. |
| `recipe` | For `make` only: the recipe worked, at home. |
| `tools` | Tools the work needs and wears (good ids; `[]` for none). Without a free one in the household the work is left out, and the tool counts as one work waits on. A `make` activity's tools are its recipe's, so it lists none. |
| `rate` | Work done in an hour against the task's authored rates: 1 with the tools they assume, less by hand (reaping without a sickle). Field work at a lower rate is left out while the same task can be done at a higher one. |
| `name`, `doing` | "Gather plants"; "gathering wild plants" (what the inspector says). |
| `par` | Physical activity ratio of the work (1–10). |
| `min_age_years`, `max_age_years` | Who does it. |
| `min_minutes`, `max_minutes` | How long the work lasts (sleep and meals take their length from the people profile). |
| `daylight_only`, `max_walk_minutes` | Only in daylight; the longest one-way walk people make for it. |

## Kinds (M3a)

### `recipe`

What goes in, what comes out, the work and the tools (ADR-0006 §2). Working one is a `make`
activity. Who works a recipe, and when, is decided by people at run time.

| Field | Meaning |
|---|---|
| `inputs`, `outputs` | `[{ good, amount }, …]` per unit made, in each good's unit (kilograms, or standard tools). A tool made counts its maker's quality: a skilled maker's lasts longer. |
| `session_inputs` | `[{ good, amount }, …]` per session, whatever its size (the fuel to heat an oven). |
| `unit_h`, `session_h` | Labour per unit and per session, hours of a capable adult of middling skill. A session makes what its time, its inputs and its `max_units` allow. |
| `max_units` | Most units one session makes; 0 for no limit. |
| `tools` | Tools it needs and wears (good ids). |
| `skill` | The skill it uses and trains (a skill id), or `""`. |

People make food ready (grind, bake, pound) when what is ready to eat, or a step from it, falls
below the days the people profile keeps of it, and only as much as brings it back: what is made
ahead of need spoils. They make a tool when the household holds fewer than it wants, and sooner
when work waits on it.

### `skill`

A domain people get better at with practice (ADR-0006 §2).

| Field | Meaning |
|---|---|
| `t80_h` | Hours of practice that close 80 % of the gap to mastery: s′ = s + (1 − s)(1 − e^(−kE)), k = ln 5 / T80 (research 06-08 §5.2). Levels run 0–1. |
| `speed` | `[[level, factor], …]`, ascending: how fast the work goes. |
| `quality` | `[[level, factor], …]`: the life of the tools made. |
| `founder_level` | `[from, to]`: a grown founder's level is drawn evenly between them; younger founders bring a part of it, growing from age 10. |

### Changes to M1 kinds

- `good`: `purpose = "tool"` and its `[tool]` table; `eaten` replaces `cooked`; `reserve_for`
  replaces `reserve` (content API 7).
- `activity`: `behavior = "make"` with its `recipe`; `tools` and `rate` on every activity.
- `land`: a resource can grow as a `[resource.deposit]`, with `stock_per_ha` (one figure per
  habitat): a stock laid down once that never grows back (stone, flint). Geology replaces it in
  M3b.
- `people`: `household.ready_food_days` and `household.processed_food_days` (the days of food ready
  to eat, and of each food a step from ready, a household keeps in hand);
  `household.harvest_margin_days` and `decision.w_lean` (stores that will not see a household
  through to its next harvest make wild food worth going out for); `farm.loss_share` (the grain
  lost before it is eaten, which a household grows over); `decision.w_tools` (what a session
  making a tool the household lacks, or a load of what it is made of, is worth).

## Planned kinds

Technologies (each only with its content footprint), more building programs with authored rule
graphs (grammar v2) and style primitives, property regimes, offices and policies, service
capability ladders, all as the milestones in the plan introduce them (§5, §7).
