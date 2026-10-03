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
| E3001 | A value is out of its allowed range |
| E3002 | Unknown or missing `kind` |
| E3003 | Not exactly one world-generation preset has `default = true` |

**Fingerprints.** Each pack gets an *artifact* fingerprint, a BLAKE3 hash of its files' bytes. The
whole set also gets a *semantic* fingerprint, a BLAKE3 hash of the effective compiled values. Saves
record the semantic one, so reformatting a file or adding a comment does not mark saves as
"content changed"; changing a value does.

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

## Planned kinds

Goods and recipes, technologies (each only with its content footprint), building programs and
style primitives, offices and policies, service capability ladders, all as the milestones in the
plan introduce them (§5, §7).
