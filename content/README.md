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
| E2006 | A reference names something that is not defined (a people profile's name list, an activity's land resource) |
| E3001 | A value is out of its allowed range |
| E3002 | Unknown or missing `kind` |
| E3003 | Not exactly one world-generation preset has `default = true` |
| E3004 | Not exactly one people profile, or not exactly one land profile |

**Fingerprints.** Each pack gets an *artifact* fingerprint, a BLAKE3 hash of its files' bytes. The
whole set also gets a *semantic* fingerprint, a BLAKE3 hash of the effective compiled values. Saves
record the semantic one, so reformatting a file, reordering keys, writing `3` for `3.0` or adding a
comment does not mark saves as "content changed"; changing a value does.

A world saved with other content still loads and runs by the content loaded now. Saves refer to
activities, habitats and land resources by id: a person whose activity is gone decides again, a
habitat that is gone becomes the last (catch-all) habitat, and a new resource starts at equilibrium.

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

M1 adds four kinds (kernel content API 2). Every field is required, and every number in the
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
| `sleep` | `tau_awake_h`, `tau_asleep_h`, `wake_pressure` | Two-process sleep pressure: rise awake, fall asleep, the level a sleeper wakes at. Sleep is not an option below it. |
| | `min_hours`, `max_hours`, `nap_min_minutes`, `nap_max_minutes` | A night's sleep and a daytime nap. |
| | `day_factor`, `bedtime_after_sunset_hours` | Weight of sleep pressure by day, and when the evening's sleep gate opens. |
| `social` | `tau_h`, `quality_per_companion`, `household_quality` | Relatedness eases toward the quality of present company. |
| `household` | `water_l_per_person_day`, `carry_water_l`, `water_target_days` | Water use, what one trip carries, the store people aim for. |
| | `food_target_days`, `carry_food_kcal`, `daily_kcal_per_person` | The food store people aim for, what one person carries home, average need. |
| `decision` | `temperature_sd_fraction`, `min_temperature` | Softmax temperature: a fraction of the spread of the options' scores, with a floor. |
| | `w_*`, `trip_half_worth_days` | Points per unit of each consideration (hunger, sleep, company, food and water shortage, useful work, walking, effort, darkness, rest, play); a gathering trip bringing `trip_half_worth_days` of household food is worth half a very large haul. |
| `band` | `default_size`, `min_size`, `max_size`, `min_families` | The founding band the new-world dialog offers. `min_size` is at least twice `min_families`. |
| | `camp_candidates`, `site_radius_m`, `site_max_slope`, `site_w_*`, `site_flood_hand_m` | How the band scores sampled camp sites: wild food within the radius, distance to fresh water, slope, flood risk. |
| | `provisions_days`, `elder_chance`, `young_adult_chance`, `birth_spacing_months` | What families bring and who is in them. |
| `mortality` | `a`, `b`, `c`, `d`, `e` | Siler hazard `A·e^(−Bx) + C + D·e^(Ex)` per year (ages of founders now; deaths later in M1). |

### `land`

How land is classified and what grows wild. Exactly one profile.

| Field | Meaning |
|---|---|
| `patch_cells` | Patch side in terrain cells (16 = 128 m at 8 m cells). |
| `channel_area_km2` | Drainage area that counts as a stream when measuring height above the nearest stream. |
| `richness_min`, `richness_max`, `richness_feature_m` | Patch-to-patch variation of productivity and its spatial scale. |
| `climate_cv`, `climate_autocorrelation` | Year-to-year variation of production. |
| `[[habitat]]` | `id`, `name`, `arable`, and optional `min_water_fraction`, `max_median_hand_m`, `max_mean_slope`. The first habitat whose conditions a patch meets is its habitat; the last must have none. |
| `[[resource]]` | `id`, `name`, `unit`; `production_per_ha_yr` (one figure per habitat, in habitat order); `loss_per_day`; `season` (twelve monthly weights); `max_rate_per_hour` and `half_rate_stock_per_ha` (gathering slows as the stock falls). Stocks grow and waste daily; nothing respawns. |

### `names`

`male`, `female`: given names; `place_first`, `place_second`: parts joined into place names. The
shipped list is invented and implies no real culture.

### `activity`

What people can do: an engine behavior with its numbers. Which activity anyone does, and where, is
decided by people at run time.

| Field | Meaning |
|---|---|
| `behavior` | One of `sleep`, `eat`, `fetch_water`, `gather`, `socialize`, `rest`, `play`. |
| `resource` | For `gather` only: the land resource gathered. |
| `name`, `doing` | "Gather plants"; "gathering wild plants" (what the inspector says). |
| `par` | Physical activity ratio of the work (1–10). |
| `min_age_years`, `max_age_years` | Who does it. |
| `min_minutes`, `max_minutes` | How long the work lasts (sleep and meals take their length from the people profile). |
| `daylight_only`, `max_walk_minutes` | Only in daylight; the longest one-way walk people make for it. |

## Planned kinds

Goods and recipes, technologies (each only with its content footprint), building programs and
style primitives, offices and policies, service capability ladders, all as the milestones in the
plan introduce them (§5, §7).
