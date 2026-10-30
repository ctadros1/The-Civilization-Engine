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
| E2006 | A reference names something that is not defined (a people profile's name list, provisions good, crop or home program, an activity's land resource, recipe or tool, a resource's, crop's or building program's good, a recipe's good, tool or skill, a good's `reserve_for`, a technique named by work, a technique's prerequisites, domain, `tried_in` or `needs`, a founders' technique) |
| E3001 | A value is out of its allowed range |
| E3002 | Unknown or missing `kind` |
| E3003 | Not exactly one world-generation preset has `default = true` |
| E3004 | Not exactly one people profile, or not exactly one land profile |
| E3005 | A technique gates no work: no recipe, activity, building program, bridge, vehicle or well system names it, nor does a technique that gates work need it |
| E3006 | Techniques' prerequisites form a cycle |
| E3007 | A recipe can never be worked: an input or tool comes only from recipes that need it (research 07-03 §6's bootstrap test) |

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
| | `water_use_by_walk_min` | Optional (content API 70): what a person uses a day by the one-way walk to the water, pairs `[minutes, litres]` with minutes of 0 or more, rising, and litres from 1 to 100, interpolated between and held beyond the ends. A household's use follows the walk to the water it last fetched; left out, every household uses `water_l_per_person_day`. |
| | `food_target_days`, `carry_kg`, `daily_kcal_per_person` | The food store people aim for, the load one person carries home, average need. |
| | `harvest_margin_days` | The food a household wants in store beyond the first grain of its next harvest: stores short of that much make wild food worth going out for (`decision.w_lean`). |
| | `raised_store_factor` | How many times as long goods keep on a raised store's floor as elsewhere under a roof (1 or more). |
| | `fuel_kg_per_person_day`, `fuel_target_days` | Firewood burned per person per day by month (January first), and the store people aim for; another fuel stands for firewood by its heat (`[fuel]` on goods). |
| | `short_food_days`, `recovered_food_days` | The chronicle notes a settlement's shortage below the first and its end above the second. |
| | `leave_at_depletion`, `leave_per_day`, `leave_unless_ripe_within_days` | A household with less than a day's food, whose members have drawn on average this share of their reserve, and with no crop of its own ripening within these days or reaped and waiting, weighs leaving, with at most this chance a day. |
| | `leave_w_gap`, `leave_w_stake`, `leave_stay` | How it weighs it (M4a slice Z; content API 32): points toward going for the whole of the wait to its next harvest that its food, its share of what other households could spare and its share of a common store it knows it may ask would not cover (0 to 100); toward staying for a whole year's food its fields should bring, which leaving gives up (0 to 100); and toward staying before either (−100 to 100). The day's chance is `leave_per_day` times the logistic of going's points less staying's. |
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
| | `peak_work_hours_per_day` | Content API 24: field work a capable adult gives at a peak on a day the ground can be worked (from `work_hours_per_day` to 16). A household plans its spring work on the days the ground can usually be worked at these hours, never on more than every day's ordinary work: the days the weather takes are made up while enough are left (ADR-0012 §5). |
| | `field_m`, `max_walk_minutes`, `site_candidates` | New fields are squares of `field_m`, within this walk of the settlement; how many places are looked at when marking one out. |
| `build` | `programs` | What a household may build (building ids), among what someone in it knows how to build: its home, the dwelling that covers its members and goods for the fewest hours, and stores beside it. At least one must be a dwelling its founders know how to build (content API 12; stores since API 13). |
| | `store_horizon_days` | The days over which a household reckons what a store would save: a store is worth building when what the goods its roofs have no room for would lose in the open over these days, less what they would lose in it, is worth more hours of its work than the store takes (content API 13). |
| | `home_work_places` | People who can work at a craft at once in a home, beside living there: a firm that has had more working for it at once lately builds a workshop with places for them all (content API 14). |
| | `quality_spread` | `[novice, master]`: how unevenly the parts of a building are made. Each part's quality is one less the spread at its builders' average building skill times the size of a normal draw (ADR-0009 §6; content API 15). `0 ≤ master ≤ novice < 1`. |
| | `[build.caution]` | How builders answer what their settlement has seen of a technique's buildings (ADR-0009 §6; content API 17): `half_life_years` (0.5 to 100), over which failures and the building-years they stood fade by half; `most` (1 to 10), the most times its usual strength a frame building's joists and posts are made; `half_rate` (above 0), the failures a building-year at which caution is half way to its most; `death_weight` (0 to 100), how many failures more each death in one counts as. |
| | `[build.levelling]` | How a household levels the plot of a building on sloping ground (ADR-0010 §2; content API 19): `from_m` and `most_m` (0 < from_m ≤ most_m ≤ 20), metres the ground may drop across a plot before it is cut and filled to one level, and beyond which it is not built on; `h_per_m3` (above 0), hours to dig a cubic metre of earth as it lay in the ground, carry it across the plot and spread and tamp it as fill; `side_run` (0.5 to 10), metres across a platform's sides run for each metre up or down. |
| `digging` | `h_per_m3`, `pit_side_m` | How people dig at a deposit (ADR-0010 §2; content API 20): hours a capable adult takes to dig a cubic metre of earth as it lay in the ground and lift it out (above 0), and the side of the square pit, metres (1 to 20), whose spoil is heaped on a square of the same side beside it. |
| `midden` | `kg_per_person_day`, `n_kg_per_person_year`, `half_life_days`, `load_kg`, `spread_h_per_t` | Each household's midden beside its home and carrying it to the fields (M3c slice V; content API 29): kilograms a member adds to the heap a day (0 to 10) and the nitrogen a member's share holds a year (0 to 20; the core pack's must stay below the nitrogen in the grain a person eats in a year, which a test checks: a midden returns part of what harvests took, never more); days the heap takes to lose half of itself and its nitrogen to the air and the rain (0 to 36,500; 0 keeps it all); kilograms carried in one load (1 to 200); and hours a capable adult takes to dig a tonne out of the heap and spread it (0 to 100), besides carrying it a load at a time there and back. |
| `ties` | `room`, `companions`, `prior`, `evidence_half_life_days`, `familiarity_half_life_days`, `warmth_half_life_days`, `fear_half_life_days`, `help_half_life_days`, `hold_help_h`, `salience_per_evidence`, `new_share`, `ask_known_min`, `acts` | What people remember of one another (M4a slice Y, ADR-0014; content API 30): the most ties a person keeps (1 to 1,000); how many of those at the hearth a session's company touches (0 to 100); the good and bad evidence a stranger starts from in each domain (0.01 to 100); the half-lives, in days, at which evidence fades back to that prior and familiarity, warmth, fear and the balance of help fade toward nothing (1 to 36,500); the hours of help owed or owing above which a tie is let go only when no other can be (0 to 10,000); the salience a unit of evidence adds beside familiarity and warmth (0 to 10); the share of a session's company drawn from those not yet known (0 to 1); and the minutes further a household short of food walks to ask someone its members regard fully than a stranger, among those who could give it as much (0 to 1,440). `acts` says, for every act the engine records (`gift_received`, `gift_given`, `wages_paid`, `work_seen`, `rent_paid`, `land_lent`, `traded`, `learned_from`, `hearth`; from content API 35 also `took_from_us`, `took_from_others`, `restored` and `refused_restitution`: coming to believe the other took food from one's own household or another's, and the other's household giving back, or refusing to give back, what was taken from one's own), the share of the gap to 1 that each act, or each hour for `learned_from` and `hearth`, closes in familiarity and warmth (0 to 1), and the good and bad evidence it adds (0 to 100) in its `domain` (`provision`, `craft`, `word`, `counsel`, or `none`). Every act must be listed once; an act the engine does not record is an error. |
| `standing` | `candidates`, `notable_share`, `notable_floor`, `notable_keep` | Standing and notables (M4a slice Y, ADR-0014 §3-4; content API 30), worked out on the first of each month: how many people each adult counts among those they regard most (1 to 100); the share of a settlement's adults who are notables (0 to 1), at least `notable_floor` of them (0 to 1,000); and how far down the ranking a notable may fall and stay one, in multiples of their number (1 to 10). Being a notable grants nothing: it only decides who considers the settlement's affairs weekly, from slice Z. |
| `polity` | `review_days`, `notice_days`, `gathering_minutes`, `quorum_share` | The polity and its founding custom (M4a slice Z, ADR-0013; content API 31): days between a polity's routine reviews, when its notables, the elders of households whose food will not last, and whoever came to a gathering or learnt a law since the last review weigh proposing (1 to 365); days from a proposal to the gathering that decides it (1 to 30); minutes a gathering sits from the start of the evening (15 to 600); and the share of the settlement's adults who must come for it to decide (0 to 1). Every world starts from this one custom: the adults at the hearth decide by acclamation, more for than against, a tie failing. |
| | `w_gain`, `w_regard`, `stance_margin` | Stances (0 to 100 each): points per unit of what a law is forecast to bring a member's household (the change in the expected log of its year's food above subsistence, over an ordinary and a lean year), points for full regard for the law's sponsor, and the points either way within which a member abstains. |
| | `attend_base`, `w_attend` | Attending: the points the custom itself is worth (−100 to 100), and points per point a member's household and regard for the sponsor have at stake (0 to 100). |
| | `wet_attend` | Points a gathering held in the open is worth the less on a wet day, no hall of the polity's standing (0 to 100; content API 154, M8d slice DC). |
| | `w_followers`, `propose_cost`, `temperature` | Proposing (ADR-0013 §5): the weight of those who regard a sponsor against the sponsor's own household (0 to 10), the points a proposal costs (0 to 100), and the temperature of the choice among moves and none (0.01 to 100). A move is worth its forecast by the share of those the sponsor knows who would back it. |
| | `vote_memory_days` | Days a member remembers how a gathering they came to stood on a proposal (0 to 3650): while they do, they expect no more support for the same proposal, at the same level or naming the same holder, than it got there (content API 34; research 09-05 §1.2: a sponsor weighs expected success). |
| | `prior_lean`, `prior_years`, `lean_harvest`, `subsistence_share` | Forecasts: the lean years believed in before any are seen, out of `prior_years` (lean years seen are those the settlement's food ran short in); a lean year's harvest as a share of an ordinary one (0 to 1); and the share of a year's food below which a household cannot live (0 to 1): food there is worth the most, and a household that would fall below it by paying a levy cannot pay. |
| | `comply_base`, `w_stance` | Paying a levy (ADR-0013 §3): the points for paying before its cost and apart from the norms the payer holds (−100 to 100; 0 in the core pack since content API 42, when the norm `core:norm/gathering_binds` took over its former 1.5), and per unit of the stance the payer took (1 for, −1 against, 0 otherwise; 0 to 100), against what paying costs their household. The chance of paying is logistic in the points. |
| `crime` | `objection_mean`, `objection_sd`, `objection_heritability`, `objection_filter` | Taking (M4b slice AA, ADR-0015; content API 35). Each person's objection to taking what is not theirs, 0–1: the log-odds of a founder's is drawn from a normal of this mean (−20 to 20) and spread (0 to 20), and a child's is pulled toward its parents' with this heritability (0 to 1; research 04-09 §1.1, §5.1). At or above `objection_filter` (0 to 1) taking is not weighed at all: a moral filter first (04-09 §5.3). |
| | `w_objection`, `w_seen`, `w_risk_trait`, `w_regard` | Points against taking (0 to 1,000; `w_risk_trait` 0 to 10): for a full objection; for a certainty of being seen, scaled by `exp(-w_risk_trait × risk)` for the person's risk-taking; and for full regard for the elder of the household taken from. The gain is asking's: the food the household needs, by the load a taker can carry. |
| | `risk_prior`, `risk_alpha`, `risk_alpha_told` | The chance of being seen a taker runs, as each person believes it (0 to 1 each): before anything is seen or heard, and the share of the gap their own attempt closes toward what happened (seen or turned back 1, unseen 0) and a taking they hear of closes toward 1 (04-09 §5.4: p̂ ← (1 − α)p̂ + αs). A loss found that nobody saw moves it toward 0. |
| | `sight_m`, `dark_sight_m`, `lit_m`, `notice_chance`, `retry_hours`, `guardian_age`, `wake_chance` | Guardianship, settled on arrival (12-04 §1.4): metres within which someone awake sees a taker at a store (0 to 10,000); after sunset once the world has lit a lamp, `dark_sight_m` instead (content API 167; 0 to `sight_m`, absent the same), unless the store lies within `lit_m` of a home where a lamp burns for work (0 to 100, absent 0); the chance the taker notices each person of `guardian_age` or more about, and turns back (0 to 1); the hours, on average, someone who turned back or fled waits before weighing taking again (0 to 87,600; the wait is drawn between half and one and a half times this); the age from which someone at home or about stops a taker (0 to 130; younger children see, and do not stop); and the chance a grown sleeper at home wakes (0 to 1). A member of that age home and awake turns the taker back. |
| | `remember_days`, `refuse_regard` | Days a person keeps what they believe of a taking (0 to 36,500), and the regard (0 to 100) for the elder of a household taken from at which a household refuses the asks of the taker it believes took from it. |
| | `demand_base`, `w_demand_loss`, `w_forgive`, `due_days` | A household that learns who took from it chooses to demand the food back or let it go (09-07 §4.1): points before anything is weighed (−100 to 100), per day of its food taken (0 to 100) and against, for full regard for the taker (0 to 100); the chance of demanding is logistic in the points. A demand falls due after `due_days` (1 to 3,650). |
| | `comply_base`, `w_comply_known`, `w_comply_regard`, `w_comply_cost`, `keep_days` | The taker's household answers a demand once: points before anything is weighed (−100 to 100), for the share of its settlement's households that believe the taker took and for full regard for the household owed (0 to 100 each), and against, per day of its own food paying would cost (0 to 100). One that means to pay gives food beyond `keep_days` of its need (0 to 3,650) day by day; what it cannot pay when due is an arrear. |
| | `report_cost`, `w_case_belief`, `w_comply_found`, `exile_days` | Cases (M4b slice AB, ADR-0015 §4–§5; content API 36). Under a law against taking its chooser knows of, a household taken from may bring a case: points for what it would recover (the food and the bundle's compensation, at `w_demand_loss` a day) times the chance it believes the gathering would find (1 − 0.5^accounts, from its distinct witnesses; research 12-04 §1.5), less `report_cost` (0 to 100; 09-07 §4.1) and `w_forgive` for regard; one whose demand was refused or unpaid when due may bring one after. At the hearing each who came weighs `w_case_belief` points (0 to 100) toward a finding if they believe the accused took, otherwise the accounts told there either side of an even chance; the days of food their household stands to gain or lose; and the polity's `w_regard` for the one who brought it less that for the accused. What a finding imposes is answered once, whole, with `w_comply_found` points toward paying (−100 to 100). A bundle that exiles weighs on a taker's household, and stands at stake for it at a hearing, as `exile_days` of its food (0 to 3,650). |
| | `watch_guard`, `w_watch`, `rounds_per_night`, `round_stops`, `w_watch_report`, `ask_days` | The watch (M4b slice AC, ADR-0015 §6; content API 37). A household weighs a law naming a watch by `watch_guard` (0 to 1) of all it found missing in the past year back to it and as much of what its own members took from it. The one named walks rounds at night: a round is worth `w_watch` points (0 to 100) for the night's first and less for each walked since, up to `rounds_per_night` (1 to 24; research 12-04 §2.1 gives one account of three), each standing watch at `round_stops` homes (1 to 1,000) in a set order. A watcher who sees a taking chooses once: saying nothing is 0 points; bringing it before the gathering, or telling those taken from, is `w_watch_report` (−100 to 100) and `w_forgive` for regard for the household taken from less for regard for the taker; under a law against taking, asking the taker's household for `ask_days` of its food (0 to 365; at most what it can spare) is worth `w_demand_loss` a day of the watcher's household's food, less `w_objection` for their objection (never weighed above its filter) and `w_seen` for the chance they believe they run of being found out (research 09-09 §1.3). The household asked pays with `w_comply_known` points against `w_comply_cost` a day of its food. |
| | `curfew_guard`, `curfew_cost_days` | The curfew (M4b slice AD; content API 38). A household weighs a curfew by `curfew_guard` (0 to 1) of all it found missing in the past year back to it and as much of what its own members took from it, less `curfew_cost_days` (0 to 365) of its food a year for each hour the curfew runs and each grown member it keeps at home. |
| `word` | `share_home`, `share_urgent`, `share_routine`, `news_days` | Word of mouth (M4c slice AE, ADR-0016 §3; content API 39). A gathering is heard of by contact, never broadcast: its sponsor, or whoever brought its case, knows first; at midnight each member of a household tells each other member with `share_home` (0 to 1), and companions at the hearth tell each other with `share_urgent` (0 to 1; research 09-16 §2.2 gives 0.4 to 0.9 for urgent news). Only those who heard may come. A grievance held keenly enough is told to a companion with `share_routine` (0 to 1; 09-16 §2.2: 0.05 to 0.25), and stays news `news_days` (1 to 3,650) without being told again. |
| | `max_grievances`, `half_life_days`, `full_harm_days`, `reminder`, `tell_floor`, `remind_days` | Grievances (ADR-0016 §2; research 04-06 §1.7, §5.3). A person holds at most `max_grievances` (1 to 64), the least keenly felt giving way. Its activation fades by `half_life_days` (four values above 0 and at most 3,650: subsistence, extraction, treatment, a collective claim; 04-06 §2.2 gives 7 to 90 days for severe events) and is raised only by a reminder. A harm of `full_harm_days` (0.01 to 3,650) of the household's food is felt fully, less in proportion; hearing it told raises it by `reminder` (0 to 1); below `tell_floor` (0 to 1) it is not told; a harm that goes on (a store still empty) raises it again only after `remind_days` (1 to 365). All tuning values. |
| `opinion` | `share`, `eta`, `epsilon`, `youth_until`, `youth_factor` | Talk at the hearth (M4c slice AG, ADR-0016 §4; content API 41): a companion says where they stand on a question with `share` (0 to 1) times its salience, and a listener moves `eta` (0 to 1) of the way toward it at full salience and trust (their regard for the teller), less the further apart they are, over `epsilon` (0.01 to 10), and `youth_factor` (0 to 10) times as far below the age `youth_until` (0 to 120). Research 06-04 §3.2's design priors: 0.5 exposures a person a week, η 0.03, ε 0.25, susceptibility doubled between 12 and 30. No negative influence. |
| | `anchor_half_life_days`, `anchor_points`, `w_position`, `salience_live`, `salience_idle` | Each adult's anchor on a question is what their household's lot makes of it, worked out on the first of each month: a household forecasting `anchor_points` (0.01 to 100) points of gain leans 0.73 for it. Their position is pulled toward it with a half-life of `anchor_half_life_days` (1 to 36,500; 06-04 §3.2: 2 years). How far talk has moved someone from their household's lot adds `w_position` (0 to 100) points to their stance at a gathering. Salience is `salience_live` (0 to 1) while a law on the question is in force or before the gathering, else `salience_idle`. Tuning values. |
| `style` | `alpha`, `prestige_most`, `innovation`, `seen_most`, `sight_m` | How households' taste in building moves (ADR-0009 §7; content API 22): the share of the way a household's taste moves toward the most admired building its settlement finished in the year past (0 to 1); how many times the least admired the most admired weighs (1 to 10), for its owner's standing in goods and how well it was built, equally; and the chance a building has one trait new to its builders, within what its program allows (0 to 1). Content API 57 (M5b slice AR): how many buildings of other settlements, finished in the year past, a person keeps in mind until their household's next review (0 to 64; 0, nobody notes any), and how far from where they stand, at a hearth they visit or a door they buy at, they see them (0 to 2,000 m). |
| | `tradition`, `tradition_spread`, `personal_spread` | Each a table of `pitch_deg`, `eave_m` and `overhang_m`: the way of building founding bands' are drawn around (pitch 0 to 80°, eaves 0.5 to 6 m, overhang 0 to 3 m), the standard deviation of a band's from it, and of each household's from its band's (each from 0 to 20°, 1 m and 1 m). A building is built to its household's taste held to what its program allows. |
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
| `[weather]` | How the weather is drawn (ADR-0012 §1; content API 23): one daily series for the whole map, from the world's seed and its landscape, the same at every speed. Monthly values are twelve figures, January first. Rain: `wet_days` (0 to 0.95), the share of days with at least `wet_day_mm` (0.1 to 10) of rain or snow water; `rain_share` (summing to 1), each month's share of the preset's `precipitation_mm_per_yr`; `persistence` (0 to 0.95), how much likelier rain is after a wet day than after a dry one; `gamma_shape` (0.2 to 5), the shape of a wet day's amount above the threshold. Temperature: `mean_c`, each month's mean at `normals_at_m` metres (interpolated between mid-months); `day_sd_c` and `day_persistence` (0 to 0.99), the day's anomaly about it and its lag-one correlation; `day_range_c`, the day's range from lowest to highest; `wet_day_range` (0.1 to 1), a wet day's range against the month's; `wet_day_cooling_c`, how much cooler a wet day is than a dry one, each month's mean kept; `lapse_c_per_km` (0 to 12), the fall with height. The slow anomaly, a monthly AR(1) that carries wet and dry spells across seasons: `slow_months` (1 to 600), its persistence; `slow_amount` (0 to 1) and `slow_wet_days` (0 to 0.5), how it scales wet days' amounts (mean kept) and their share; `slow_warmth_c`, its effect on each month's temperature. Snow: falls below `snow_below_c`, melts `melt_mm_per_c` mm of water a degree-day, and lies in bands of 100 m. Soil water (FAO-56): `soil_water_mm` (10 to 500) held between field capacity and wilting, `easy_water_share` (0.1 to 0.9) drawn unstressed, and `cover_kc` (0.1 to 2), the wild cover's use against the reference evapotranspiration (Hargreaves, from temperature and the people profile's latitude). Workable days (ADR-0012 §5; content API 24): `wet_ground_mm` (0.5 to 100), a day's rain or snow, mm of water, that makes the ground too wet to work that day, and `frozen_below_c` (−10 to 5), the day's mean below which it is frozen; snow lying keeps people off it too. Breaking, preparing and sowing ground wait for a day it can be worked (weeding, reaping and threshing go on), and households plan their spring work on the share of the sowing window's days the landscape usually allows, worked at the people profile's `peak_work_hours_per_day`. Roof loads (ADR-0012 §5; content API 25): `storm_median_kpa` (0 to 50) and `storm_spread` (0 to 3), the month's storm, one draw a month for every roof under the world's weather on one day of the month, log-normal in kilopascals on a roof's plan (its median and the spread of its logarithm); `roof_snow_share` (0 to 2), the share of the snow lying on the ground at a building's height that a roof keeps when pitched at `roof_snow_full_deg` or less, falling in proportion to none at `roof_snow_shed_deg` (0 to 90, above `roof_snow_full_deg`). Snow on a roof presses its water's weight. |
| `[soil]` | How the soils hold nitrogen (ADR-0012 §3; content API 27), kilograms a hectare of native ground of average richness (a field's ground scales them): `slow_n_kg_ha` (0 to 50,000), organic nitrogen in humus, and `fast_n_kg_ha` (0 to 5,000), in roots, stubble and fresh organic matter; `slow_turnover` (0 to 0.5) and `fast_turnover` (0 to 1, at least `slow_turnover`), the share of each pool that turns to mineral nitrogen in a year; `free_n_kg_ha` (0 to 200), mineral nitrogen the air and free-living microbes add a year; and `uptake_share` (0 to 1), the share of a year's mineral nitrogen a crop can take up. Each field's pools turn once a year, on the first of January: what they release and what the air adds is the year's supply, the rest of last year's being lost, and a harvest is the least of what its season and that supply allow. A field that gave no harvest the year before grew its wild cover, which brings each pool back toward native ground's at the pool's own rate. `regrown_years` (content API 28; 0 to 100, 0 never): whole years broken ground can lie unsown before it has grown over and must be broken again (not cleared again). `manure_fast_share` (content API 29; 0 to 1): the share of dung's nitrogen that enters the fast pool, the rest the slow one, each pool's share of it turning at once so the year's crop draws on `manure_fast_share × fast_turnover + (1 − manure_fast_share) × slow_turnover` of it wherever in the year it is spread. |
| `[banks]` | Conditional overtopping erosion (content API 87; optional). `erosion_per_head_day` (0 to 10, default 0.02) multiplies actual overflow head in metres to the power 1.5 to give metres of crest loss in a day. This is a design prior, not a historical breach frequency. Dry and sub-crest conditions do not erode a bank. Removed soil remains in an explicit compacted-equivalent washed-sediment stock; deposition and remobilization are not modelled yet. Piping, quality, settlement changes and repair work remain planned. |
| `[groundwater]` | Water under the ground and the rivers by the day (ADR-0021 §1–§2; content API 69; optional). What the reference soil cannot hold on a day is its surplus: `recharge_share` (0 to 1) of it drains down to a shallow aquifer, the rest runs off at once. The aquifer keeps a head for each habitat patch and exchanges water with the four patches beside it and with the river, lake or sea in a patch; where a head reaches the lowest ground of its patch the water seeps out there. What reaches the water and what runs off pass through one runoff store for the world, which lets go `1 − e^(−1/recession_days)` of what it holds a day (`recession_days` 0.1 to 365); its outflow against the landscape's long-run mean surplus scales every reach's mean discharge. A patch off the water whose head stands at its lowest ground is a spring there once what flows to it from the patches round it reaches `spring_min_m3_day` (0 to 10,000). `[[groundwater.unit]]` (1 to 16, first match wins): a unit of ground under the habitats it names (`habitats`, habitat ids; the last unit names none and lies under the rest), with a `name`, `k_m_day` (`[least, most]` within 1e-9 to 10,000) and `specific_yield` (`[least, most]` within 0.0001 to 0.5), between which each world draws its conductivity (evenly in its logarithm) and specific yield once, and `thickness_m` (0.1 to 500), its saturated thickness. Left out, one unit of alluvial sand (1 to 10 m a day, 0.15 to 0.3, 10 m) lies under everything, half the surplus recharges and the store's recession is three days. |
| `[contamination]` | How what people shed moves through the ground and water (ADR-0021 §4; content API 73; optional). Each day a heap beside a home loses `wash_per_mm` (0 to 1) of its load for each mm of the day's surplus, at most `wash_max` (0 to 1), to the open wellheads and river banks within `runoff_radius_m` (0 to 1,000) whose ground lies no higher, split by nearness (none there, it soaks away); `seep_per_day` (0 to 1) of it soaks in, and `capture_share` (0 to 1) of that reaches the wells and rivers within `link_radius_m` (0 to 1,000) and no higher, split by nearness, after the ground's travel time `d·n / (K·i)` (research 03-02 §1.6; `K` and `n` its unit's conductivity and specific yield, `i` the slope of the ground, at least `min_gradient`, 0 to 1), less what dies on the way. Each person drinks `drink_l_per_day` (0 to 30) of their household's water: what carries a dose. Left out, the core's values hold (0.01, 0.5, 30 m, 0.02, 30 m, 0.1, 0.005, 3 L); all are tuning values. |
| `[paths]` | `wear_per_walk`, `wear_half_life_days`: a walk across an 8 m cell wears away this share of what is left unworn, and unused wear halves in this many days. `trail_at`, `trail_until`: a cell becomes trail at the first wear and stays trail until it fades below the second. |
| `[[habitat]]` | `id`, `name`, `arable`, and optional `min_water_fraction`, `max_median_hand_m`, `max_mean_slope`. The first habitat whose conditions a patch meets is its habitat; the last must have none. Arable ground can carry fields; optional `clear_h_per_ha` is the work to clear it (woodland) before it is first broken. `wetness` (zero or more; 1 is average ground) scales how fast posts set in it rot at their foot (content API 15). |
| `[[deposit]]` | Where a kind of deposit lies and how large its bodies are, laid down once from the world's seed (ADR-0010 §1; content API 18; optional): `good` it yields; `slope` and `hand_m`, `[least, most]` mean slope (0 to 10) and height above the nearest channel in metres of the ground it lies under; `per_km2` (0 to 1000), bodies expected on each square kilometre of land that qualifies; `radius_m` (0.5 to 1000), `top_m` (0 to 100, the cover over it) and `thickness_m` (0.05 to 100), `[least, most]` metres; `quality` (0 to 1); `exposed_share` (0 to 1), the share of covered bodies that still show; `density_kg_m3` (100 to 10,000), kilograms of the good in a cubic metre of a body; optional `dig_h_per_m3` (0 to 1,000; content API 21), hours a capable adult takes to break a cubic metre of a body out of the ground, or 0 (the default) to dig it as earth at the people profile's `[digging]` rate, its cover always dug as earth; optional `working` (1 to 24 characters; content API 21), what a working of it is called, "pit" by default ("quarry"). |
| `[[resource]]` | `id`, `name`; `good` (the good a harvest yields) and `unit_kg` (its kilograms per unit of stock: 1 for stocks in kilograms, a carcass's meat for stocks in animals); `discrete` (harvests are whole units drawn from the expected count); `in_water` (lives in a patch's water, not its land); `range_patches` (a trip works a block of `2·range + 1` patches a side); `max_rate_per_hour` and `half_rate_stock_per_ha` (gathering slows as the stock falls). Then exactly one growth table. |
| `[resource.plant]` | `production_per_ha_yr` (one figure per habitat, in habitat order), `loss_per_day`, `season` (twelve monthly weights): a seasonal production each day, a share of the standing stock lost each day. A new world's stock, and what people expect of land they have not worked, is the yearly cycle this settles into: what stands lags what grows by about the time it lasts (last summer's reeds still stand in March). Optional `follows_water` (content API 23; false by default): production follows the soil water under the wild cover, smoothed over about a month, against what the month usually has, so a dry summer gives less and a wet one more, and an average year what the figures say (ADR-0012 §5). |
| `[resource.animal]` | `capacity_per_ha` (per habitat), `growth_per_year`, `spread_per_month`: logistic growth toward the habitat's capacity, and a monthly spread between neighbouring patches toward an even share of capacity. |
| `[resource.deposit]` | `stock_per_ha` (per habitat): a stock laid down once, that never grows back (stone, flint; M3a). |

Stocks change daily; nothing respawns. What people learn about a place fades over the time its
resource takes to renew: `1 / loss_per_day` days for plants, `365 / growth_per_year` for animals,
ten years for deposits.

### `good`

Something people carry home and keep.

| Field | Meaning |
|---|---|
| `purpose` | `food`, `fuel`, `material` (built with, or made into something), `tool` (M3a) or `store` (a vessel food is kept in; M3b, content API 20). |
| `kcal_per_kg` | Food energy; 0 for anything else. |
| `half_life_days` | Days for half a stored amount to spoil; 0 keeps. Fuel must keep. |
| `sheltered_half_life_days` | The same in a household's store under its own roof; 0 when a roof makes no difference. Never shorter than `half_life_days`; a good that keeps needs none. |
| `eaten` | `raw` (as it is), `cooked` (needs a fire: not eaten while the household has no firewood; cooking adds no energy, research 05-02 §1.1) or `never` (a recipe must make it food first, as grain is ground or pounded). Anything but food is `never`. |
| `shared` | When brought home it is shared among every household of the settlement, by members. |
| `reserve_for` | The food it is kept back from, like seed from grain, or `""`: a recipe that needs that food takes this one only in real hunger, and never the seed to sow the ground already cropped. |
| `[tool]` | For a tool only: `life_h`, the hours of use a standard tool lasts (stores count tools in standard tools, so 2.4 sickles are two and what is left of a third, ADR-0006); `per_adult`, how many a household wants for each member of working age (rounded up; 0 for none); `fixed`, it stays where it was made and is never carried off: founders and families the observer sends bring none of it (an oven). |
| `[timber]` | For a material built with as timber only (content API 16, ADR-0009 §5): `bending_mpa`, `compression_mpa` and `stiffness_gpa`, its strength in bending and in compression along the grain and its stiffness; `creep`, how much further it bends under load carried for years than at first (φ); `sustained`, the share of its strength it keeps under such load (above 0, at most 1); optional `burnt_keeps` (0 to 1, content API 208, ADR-0037 §5), the share of its strength a bare member keeps once its building has burnt out, for a material that softens rather than chars (absent, its members char). Members of a material without it carry nothing in the checks. |
| `[store]` | For a store only (content API 20, ADR-0010 §2): `keeps_kg` (above 0, at most 1000), the kilograms of food one keeps as a raised floor keeps them. A store stands under a roof in the room it keeps: a household counts each as turning that many kilograms of the room in its lofts and on its floors into raised room, and makes more while food lies in its lofts and on its floors that would keep better in one. Stores add no room, and food in the open gains nothing from them. |

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
| `kc`, `kc_days`, `ky` | Its water (ADR-0012 §2; content API 23): crop coefficients at the start, in mid-season and at ripeness (0.05 to 2; its need is the reference evapotranspiration times the coefficient, flat, rising, flat and falling by FAO-56's curve); the days of its initial, development, mid-season and late stages, summing to `grow_days`; and the share of its yield lost for each share of its need unmet (0 to 3, FAO's `Ky`). |
| `grain_n_kg_per_kg`, `straw_n_kg_per_kg`, `crop_n_kg_per_kg` | Its nitrogen (ADR-0012 §3; content API 27), each 0 to 0.2: in a kilogram of its grain and of its straw, and what the whole crop takes up for each kilogram of grain it grows, which must cover the grain's and the straw carried home's. A harvest's grain and straw take their nitrogen from the field; the rest of what the crop took up returns to the soil's fast pool. |

What a harvest brings depends on the field's ground (the richness of its patches), the water its
crop had through the season, when sowing finished, how much of its tending was done, and how long
it stood ripe. Each growing field keeps a root-zone water balance from the day it is sown to the
day it ripens; its harvest scales by `1 − ky × (1 − water got ÷ water needed)`, divided by that
figure's long-run mean for the landscape, so a season of average water gives `yield_kg_per_ha`.
The soil can hold it back (content API 27): a harvest is the least of what the season allows and
what the year's supply of mineral nitrogen lets the crop grow, `supply × uptake_share ÷
crop_n_kg_per_kg` a hectare (research 03-04 §5.2), and each field remembers its last eight
harvests and which of the two held each back.

### `building`

A building program: what it is for, what it is made of, the work and material each part takes, and
the dimensions people build it to. The grammar that expands a design into its parts is code
(`civ-grammar`): `hut`, the round hut of M1, frozen at version 1, and `frame`, rectangular
post-framed buildings in bays (M3b slice O, ADR-0009). Who builds what, where and when is decided
by people at run time: households build their homes, and stores beside them, to the people
profile's `programs`. Content API 11 added `use` and the frame grammar, API 12 the frame programs'
`[design]`, API 13 stores among the programs households build, API 14 workshops, API 15 the
building skill and `[upkeep]`.

| Table | Fields | Meaning |
|---|---|---|
| (top) | `grammar` | The grammar that expands it: `hut` or `frame`. |
| | `use` | What its buildings are for: `dwelling`, `store`, `work` or, from content API 154, `hall` (a polity's gatherings meet in it), and from 155 `tower` (a polity's watch looks out from it); only a polity raises a hall or a tower (the use of the plot it stands on). |
| | `eave_cm`, `pitch_deg` | The wall height (a frame's: each storey's) and roof pitch people build to, within the rules' ranges. |
| | `roof_by_day` | The day of the year (from 0) a household wants to be under its roof by; the pressure to build grows as it nears. |
| | `technique` | The technique building it needs (ADR-0008). |
| | `skill` | The skill building and mending it use and train (a skill id, or `""`): its builders' average level sets how evenly its parts are made (ADR-0009 §6). |
| `upkeep` | `covering`, `posts`, `infill`, `under_leak` | How its parts wear (ADR-0009 §4), each `[share lost a year, share at which it shows]`: the roof's covering in the weather; posts at their foot, times the wetness of the habitat it stands in; walls' infill at their foot; and roofed timber (beams, joists, rafters, a raised floor) times the share of the roof over it that leaks. Shares lost a year are zero or more, thresholds above 0 and at most 1. A part is lost entirely at a loss of 1: a covering is gone, posts give way (a ruin). |
| `materials` | a hut's `timber`, `wattle`, `thatch`; a frame's `timber`, `wattle`, `covering`, `boards`, and optionally `frame` (content API 207) | The good each material slot is made of (good ids, each a `material`); `frame` is a steel skeleton's posts, beams and joists (ADR-0037 §3). |
| `rules` (a hut's) | `radius_cm`, `eave_cm`, `pitch_deg` | Allowed ranges, `[least, most]`. |
| | `floor_base_m2`, `floor_m2_per_sleeper` | The floor a household needs whatever its size and per resident; the hut's radius follows, rounded up to whole decimetres. |
| | `store_kg_per_m2` | Goods its floor holds besides living on it, kilograms a square metre. |
| | `post_spacing_cm`, `post_diameter_cm`, `posthole_depth_cm`, `wall_thickness_cm`, `roof_overhang_cm`, `thatch_thickness_cm`, `hearth_cm` | Dimensions. The plot a household claims is the square its roof covers. |
| | `groundwork_h_per_m2`, `posthole_h`, `post_h`, `rafter_h`, `wattle_h_per_m2`, `daub_h_per_m2`, `thatch_h_per_m2`, `finish_h_per_m2` | Person-hours of a capable adult for each part: foundation (groundwork and postholes), frame (posts and rafters), walls (wattle and daub), roof (thatch), finish (floor and hearth). All positive. |
| | `post_kg`, `rafter_kg`, `wattle_kg_per_m2`, `thatch_kg_per_m2` | Material in each part. |

A frame program's rules are a `[frame]` table instead:

| Fields | Meaning |
|---|---|
| `bays`, `bay_cm`, `width_cm`, `storeys`, `eave_cm`, `pitch_deg`, `joist_cm`, `post_cm`, `overhang_cm`, `floor_raise_cm`, `wall_cm` | Allowed ranges, `[least, most]`: bays (up to 8), a bay's length, the width between the long walls, storeys (one or two), each storey's height, roof pitch, the diameters of joists and posts, the roof's overhang, a raised floor's height (`[0, 0]` for none) and the walls' thickness. |
| `lofts` | Whether bays may be floored as lofts (a floor at the wall heads inside the roof, holding goods only). |
| `ground`, `upper` | What the first storey and an upper storey are for: `living`, `store` or `work`. A hearth goes on the lowest living storey. |
| `posthole_depth_cm`, `beam_cm`, `rafter_cm`, `rafter_spacing_cm`, `joist_spacing_cm`, `decking_cm`, `thatch_thickness_cm`, `hearth_cm`, `door_cm` | Dimensions: plates, rails, beams and the ridge are `beam_cm` poles. |
| `floor_base_m2`, `floor_m2_per_sleeper`, `work_m2_per_worker` | Living floor a household needs whatever its size and per resident; floor a place to work takes. |
| `store_kg_per_m2`, `loft_kg_per_m2`, `living_kg_per_m2` | Goods a store's floor, a loft and living or working floor hold, kilograms a square metre. |
| `timber_kg_per_m3` | The timber's density: posts, beams, rafters, joists and boards are costed from their sizes. |
| `frame_kg_per_m3` | A steel skeleton's members' weight over their envelope (content API 207; 0, the default, for none): its posts, beams and joists are costed from their sizes at it. |
| `groundwork_h_per_m2`, `posthole_h`, `post_h`, `beam_h_per_m`, `rafter_h`, `joist_h_per_m`, `wattle_h_per_m2`, `daub_h_per_m3`, `thatch_h_per_m2`, `finish_h_per_m2`, `decking_h_per_m2`, `ladder_h` | Person-hours of a capable adult for each part. All positive. |
| `wattle_kg_per_m2`, `thatch_kg_per_m2`, `ladder_kg` | Material in each part besides the timber its sizes give. |
| `mass_wall_cm` | A mass wall's allowed thickness from frame version 3, `[least, most]` (before it, mass walls are held to `wall_cm`). |
| `[frame.masonry]` `rubble`, `mud_brick`, `fired_brick` | Each kind of mass wall: `[hours to lay a cubic metre, kilograms a cubic metre, strength in compression in pascals]`, all positive. |

A bay and the width between the long walls must each be longer than the thickest post allows
(`bay_cm`'s and `width_cm`'s least above `post_cm`'s most). The grammar refuses some buildings
whose every size lies within these ranges: a raised floor whose ladder would stand beyond the
roof's edge (half the wall's thickness plus 0.364 of the raise must not exceed the overhang, the
ladder rising at 70°), and a building of more than 32 component groups (posts, beams, walls,
floors and the roof, many of them one to a bay: two storeys of eight bays, each lofted, are too
many).

A frame program also has a `[design]` table: the sizes people build it to (`bay_cm`, `width_cm`,
`post_cm`, `wall_cm`, `overhang_cm`, `joist_cm`, and `floor_raise_cm`, 0 unless set), each within
its `[frame]` range. What varies between its buildings is how many bays, storeys and lofts they
have. At least one of those must be a building the grammar allows.

The core pack has the hut and three frame programs: a longhouse (a dwelling, with lofts and up to
two storeys), a granary (a store raised on posts) and a workshop (a working floor with a store
loft). They need jointed timber framing. Households may build the hut or the longhouse as their
home, a granary beside it, and a workshop for a firm with more at work at once than a home holds.

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
| `behavior` | One of `sleep`, `eat`, `fetch_water`, `gather`, `socialize`, `rest`, `play`, `farm`, `ask`, `build`, `make` (M3a), `trade` (M3a), `hire` (M3a), `try` (M3b), `dig` (M3b), `attend` (M4a: going to the gathering called at the hearth, while it sits, for the settlement's adults), `take` (M4b: going to another household's home to take food from its store, when short; ADR-0015 §2), `watch` (M4b: the watch's rounds at night, for whoever holds the office), `petition` (M4c: going to the hearth on the evening a faction's petition sits, for an adult who has heard of it; ADR-0017 §3), `visit` (M5a: going to the hearth of another settlement the household knows and home the same day; ADR-0018 §2), `fetch` (M5b: going to a seller's door in another settlement by a price report the household holds, to buy at the terms it posts there now, for itself or to sell at home, and home the same day; ADR-0019 §2, §6), `carry` (M5c: taking up goods a payment owes where the paying store set them aside, walking them to the other settlement's hearth and home the same day, for the one who is to carry them; ADR-0020 §7), `bridge` (M5c: walking to the bank where the household is building a crossing, working on it and walking home, only while it is building one, or while their polity is building one under a law they know of), `well` (M6a: digging and lining the well the household is making beside its home, or relining it, only while one is owed work; ADR-0021 §3), `tend` (M6a: sitting at home with members of the household who are ill there and still owed a day's care, its hours shared among them; ADR-0021 §5), `douse`, `strip`, `salvage` (M6b fire tasks; see the fire primitives below). |
| `resource` | For `gather` only: the land resource gathered (hunting, fishing, collecting firewood and cutting building materials are gathering too). A trip works until its load is full or its time runs out. |
| `task` | For `farm` only: the field work, one of `prepare`, `sow`, `tend`, `reap`, `thresh`. |
| `recipe` | For `make` only: the recipe worked, at home. |
| `digs` | For `dig` only: the good dug (a good id; content API 20). People dig it at a pit on a deposit of it that their settlement knows, while their household needs it, and carry it home; the land profile must lay down deposits of it. |
| `tools` | Tools the work needs and wears (good ids; `[]` for none). Without a free one in the household the work is left out, and the tool counts as one work waits on. A `make` activity's tools are its recipe's, so it lists none. |
| `rate` | Work done in an hour against the task's authored rates: 1 with the tools they assume, less by hand (reaping without a sickle, cutting rods without an axe). Field work at a lower rate is left out while the same task can be done at a higher one, and gathering at a lower rate while the household holds the tool for a higher one. |
| `name`, `doing` | "Gather plants"; "gathering wild plants" (what the inspector says). |
| `par` | Physical activity ratio of the work (1–10). |
| `min_age_years`, `max_age_years` | Who does it. |
| `min_minutes`, `max_minutes` | How long the work lasts (sleep and meals take their length from the people profile). Rest, play and the hearth last `min_minutes`, a session; in Accelerated mode's leisure blocks they last a run of sessions, at most `max_minutes` (ADR-0011 §4; Detailed mode never uses it). |
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

### `regime`

A property regime (slice K, ADR-0007): who holds the land a household breaks, who works it, how
its use is given, what becomes of it when a household is no more, and whether it may be let. A
world is made under one and keeps it for its life; exactly one regime is the default
(`default = true`). Which ground anyone breaks, works or lets, and when, is decided at run time.
Names such as "village" or "household" are content, not states the engine switches between.

| Table | Field | Meaning |
|---|---|---|
| `land` | `holder` | `breaker`: the household that breaks ground holds it. `settlement`: the settlement it lies by holds it. |
| | `use` | `holder`: a holder works its own ground or lets it, and ground whose holder is no more is taken up by a household short of land, which then holds it. `need`: the settlement gives each household fields to work by how many it feeds, never leaving a household short of its own need. Needs `holder = "settlement"`; `holder` needs `breaker`. |
| | `review` | `[month, day]` of the yearly review, when fields change hands between crops (a review also follows when a household forms or ends). |
| `succession` | `holdings` | What becomes of a household's fields when it is no more: `heir` (all to one heir), `divided` (shared among its heirs' households, whole fields as near equal in area as they can be) or `settlement` (back to the settlement). Heirs are the households of its last member's nearest living kin, those in its own settlement first. |
| | `union_share` | A new couple's household takes a share of its families' fields, nearest its home first, as it takes a share of their stores. |
| `lease` | `allowed` | Whether a holder may let ground it does not need. Needs `use = "holder"`. |
| | `holder_share` | The holder's share of the grain threshed from a let field, 0 to below 1, paid through the ledger (`rent`). |
| | `term_years` | Crop years a lease runs before it is renewed (while the holder can spare the field and the tenant still needs it) or ends. |

The core pack has two: `core:regime/household` (the default: households hold what they break,
let it for a quarter of the grain, take a share at a union and divide it among their heirs) and
`core:regime/village` (the village holds the ground and gives it out by need; fields go back to
it). Content API 8 added the kind.

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
- `activity`: `behavior = "trade"` (slice I): a person walks to the household whose posted terms
  save their own household the most hours of its work, the walk included, and the deal settles
  at its door through the ledger. Which seller, what and what is paid are decided at run time.
- `people`: a `[market]` table (slice I, ADR-0006 §4):

  | Field | Meaning |
  |---|---|
  | `review_days` | Days between a household's reviews of what it can spare and the terms it posts (each on its own day). |
  | `margin` | What a seller asks over its own cost of a good, a share of that cost. |
  | `max_change` | The largest change of an ask in one review, a share of it. |
  | `stock_response` | Content API 26 (0 to 3): how strongly a seller's ask for food answers what it holds beyond its needs. Its anchor, its cost and `margin`, is multiplied by `exp(-stock_response × s)`, `s` the years of its own need it can spare, at most one (ADR-0006 §4's stock term; research 08-04 §1.2, 08-05 §1.5), so asks answer the harvest. |
  | `memory_days` | Half-life of what a settlement's market remembers: sales, payments, demand nobody met, trades. |
  | `money_share`, `money_min_trades` | A good is the settlement's money once it settles at least this share of the payments' worth over at least this many remembered trades. Nothing names a money good. |
  | `accept_want` | A seller takes a good in payment when it wants at least this much more of it (1 when short of it), or when the good is the settlement's money. |
  | `recent_trades` | Trades a market keeps in its list of the latest. |

  New decision reasons: *others want it* (making a tool to sell), *it costs less from a
  neighbour* and *nobody nearby offers it*.
- `activity`: `behavior = "hire"` (slice J): a person works a session for a household's
  workshop at its posted wage, making its goods at its owners' home with their tools, and is
  paid for the time through the ledger. Which workshop, and whether the pay is worth the walk
  and the effort, are decided at run time; `par` is the effort it costs, as for other work.
- `people`: a `[firm]` table (slice J, ADR-0006 §5), every value a tuning value:

  | Field | Meaning |
  |---|---|
  | `idle_close_days` | A workshop that sells nothing for this long is given up by its owners (a voluntary exit). |
  | `book_entries` | Entries a workshop's books keep in full; its monthly statements are kept for its life. |
  | `wage_share` | The share of what an hour's work adds for a workshop, at middling skill, that it first offers as a wage. |
  | `wage_review_days`, `wage_max_change` | How often a workshop reviews its wage, and the most it raises it at a review while work goes untaken (research 08-10 §5.6: 14–30 days, 1–5 %). |
  | `max_hire_hours` | The most hours of work a workshop hires between two of its weekly reviews. |

  New decision reasons: *what the work is paid* and *nobody nearby is hiring*.

## Kinds (M3b)

### `technique`

A practical capability people know, learn and can lose (slice M, ADR-0008). It is phrased as
what a competent person can do, and it gates work: only someone who knows it does the work that
names it, or someone working beside a person who does. Who knows what, who learns it from whom,
and when it is lost are decided at run time. Nothing in content says when a technique appears.

| Field | Meaning |
|---|---|
| `name`, `can` | Its name, and what a competent person can do, completing "a competent person can …". |
| `domain` | The skill whose practice it is (a skill id), or `""`. Knowing is the gate; skill is how well. |
| `requires` | Its prerequisites as alternative routes, each a list of technique ids: `[["a", "b"], ["c"]]` is a and b, or c. `[]` for none. Someone finds it only if they know one route wholly. |
| `tried_in`, `needs`, `e50_h` | For discovery (slice N): the activities whose practice counts toward finding it (a share of their hours, `knowledge.experiment_share`), the goods a household must hold to find or try toward it, and the qualified hours at which half of those trying have found it. |
| `[answers]` `spoilage` | The household problem it answers, which draws people to try toward it: goods whose loss to spoiling it would prevent (good ids; `[]` for none). |
| `learn_h` | Hours of work beside someone who knows it that teach it. |
| `upbringing` | Children learn it at home: a child not yet grown, in a household where someone knows it, learns it on reaching the youngest age of the work it gates, or in the year before growing up if the work comes later. |

Recipes, activities and building programs name the technique they need with `technique` (a
technique id, or `""` for none). An activity that works a recipe needs the recipe's technique.

The people profile's `[knowledge]` table (every value a tuning value):

| Field | Meaning |
|---|---|
| `founders` | `[{ technique, share }, …]`: the share of founders old enough for each technique's work who know it. A craft not learnt at home is known only from adulthood. A band always brings at least one knower of each technique with a share above zero. Families the observer sends draw the same way. |
| `max_learners` | Learners one person teaches at once (research 07-02 §2.3: one to three). |
| `w_learn` | Utility points for working beside someone to learn what they know. |
| `experiment_share` | The share of routine work's hours that counts as experiment toward what its practice can find (research 07-01 §2.3: 1 %). |
| `aware_try_factor` | How many times its hours trying counts for someone already aware of the technique. |
| `w_try` | Utility points for trying at a problem at home, times the share of the household's food the problem would cost. |
| `try_gap_days` | Least days between one person's sessions of trying. |
| `watch_m` | How far, metres (0 to 1,000; 0, never), someone at another settlement's hearth or a seller's door there sees its people at work well enough to come to know of a technique the work needs (content API 58, M5b slice AR). Awareness only. |

The core pack has thirteen techniques. Ten are the founders' repertoire of today's work, known by
every founder and learnt in upbringing (research 06-08 §1.1, §3): growing emmer, grinding at a
quern, pounding grain, baking flatbread, knapping sickle blades, shaping stone, shaping wood,
building roundhouses, making pottery (slice Q: forming clay into storage pots and firing them in
an open fire, research 07-05 §3.1) and baking in an oven (slice Q: a domed clay oven built beside
the home, 11-13 §1.2). Three are not known at first. **Drying and smoking** (slice N) answers meat
and fish spoiling and is found from hunting, fishing and trying. **Grinding at a rotary quern**
(slice N) needs shaping stone and wood, and its E50 of 100,000 hours makes it, in practice, the
observer's introduction. **Jointed timber framing** (slice O) builds the frame programs; no work
tries toward it yet, so only the observer can introduce it. Content API 9 added the kind and API
10 its `[answers]`.

A `try` activity (`behavior = "try"`, slice N) spends spare hours at home trying toward the
technique whose problem would cost the household the most food, among those the person could
find. It names no technique: what it works toward is chosen when it is done. The core pack's
`core:activity/try` is an hour, from age 12.

## Changes in M3c

Content API 23 (slice U, ADR-0012) brings weather. The land profile's `climate_cv` and
`climate_autocorrelation` are gone: the yearly climate factor they drew is replaced by the
profile's `[weather]`, a daily series. A `[resource.plant]` may follow the soil water
(`follows_water`). A crop states its water use (`kc`, `kc_days`, `ky`). The core pack's wild plant
food follows the soil water, and fallen wood, poles and reeds keep their authored growth whatever
the weather.

Content API 24 (slice U) adds the days the ground cannot be worked: the weather profile's
`wet_ground_mm` and `frozen_below_c`, and the people profile's `peak_work_hours_per_day`, the
longer hours that make up lost days. The core pack's valley keeps people from turning the soil
on days with 5 mm of rain or more, with snow lying, or with a mean below freezing, and its early
farmers work 10 hours at a peak against 6 ordinarily (research 04-02 §2.4).

Content API 25 (slice U) takes roof loads from the weather: the weather profile's
`storm_median_kpa`, `storm_spread`, `roof_snow_share`, `roof_snow_full_deg` and
`roof_snow_shed_deg` replace the land profile's `[peak_load]`, which is retired (a profile that
still has it is refused as unknown). The core pack keeps the stand-in's storm figures (a median of
0.25 kPa, one month in twenty above about 0.7) and lets a roof keep 0.8 of the snow lying when
pitched at 30° or less, none from 60°: a hut's 45-55° thatch keeps 0.13 to 0.4 of it.

Content API 27 (slice V) brings soils: the land profile's `[soil]` and each crop's
`grain_n_kg_per_kg`, `straw_n_kg_per_kg` and `crop_n_kg_per_kg`. The core pack's valley holds
2,600 kg of nitrogen a hectare in humus, turning over at 0.02 a year, and 100 kg in the fast pool
at 0.3 (RothC's base rates, research 03-04 §2.5), with 15 kg a year from the air and a crop taking
up 0.6 of the supply: native ground of average richness supplies 97 kg a year, enough for 1,660
kg of emmer a hectare. Emmer's grain carries 0.020 kg of nitrogen a kilogram and its straw 0.006
(03-04 §2.2), and the whole crop takes up 0.035 for each kilogram of grain.

Content API 28 (slice V) adds `[soil] regrown_years`: broken ground left unsown that many whole
years has grown over and is broken again before it is cropped. The core pack's valley takes 3 (a
tuning value).

Content API 29 (slice V) brings middens and manuring: the people profile's `[midden]`, the land
profile's `[soil] manure_fast_share` and the field task `manure`. The core pack's households heap
0.5 kg a person a day holding 1 kg of nitrogen a person a year (research 12-02 §2.3: 0.42 kg in
faeces and 3.56 in urine, most of which soaks away), which loses half of itself in a year, and
carry it in 25 kg loads; dung's nitrogen goes 0.3 to the fast pool, so a tenth of it reaches the
year's crop (03-04 §2.3: 5-15 %). `core:technique/manuring`, which founders bring (07-04 §4.4:
manuring is documented in Neolithic Europe by the sixth millennium BCE), gates
`core:activity/manure_field`.

Content API 30 (M4a slice Y) brings ties between people: the people profile's `[ties]`. The core
pack keeps 48 ties a person (research 04-06 §2.2: 32-128; 04-11 §2.4: 16-64), touches three of
those at the hearth each session (04-04 §1.5: a gathering is not a clique), draws 15 % of that
company from people not yet known (04-04 §2.2's 5-20 % of encounters by introduction), and fades
evidence in half a year (04-06 §2.2: 30-365 days), familiarity in half a year (04-04 §2.2: 90-365),
warmth in two (1-5 years) and help owed in three (04-11 §2.4: 1-10). A gift writes provision,
wages, rent and land lent write word, work seen and learning write craft, a trade a little word,
and company at the hearth familiarity and warmth at 0.01 an hour (04-04 §2.2: 0.005-0.02). The
people profile's `[standing]` comes with it. A household short of food walks up to 30 minutes further to ask someone its
members regard fully (esteem and warmth together) than a stranger. Each adult counts eight people
among those they regard most (04-11 §2.4: 4-12 candidates); a settlement's notables are its 1 %
most counted (plan §4.2), at least three, keeping their place while ranked within 1.5 times their
number (04-11 §5.7: hysteresis). All are tuning values.

The core pack's `core:activity/cut_rods` (no API change) cuts thin rods by hand or with a flint
flake at a fifth of an axe's rate. It is left out while the household holds an axe, even one in
another member's hands: it is how a household whose axe has worn out gets the wood for a new
haft.

## Kinds (M4a)

### `policy`

A policy template (M4a slice Z, ADR-0013 §3): what a law of this kind does, the issues whose
presence makes proposing it a move, and the levels a sponsor may put forward. An issue only makes
a move available and weighs toward no policy; whether anyone proposes a law, at which level, and
whether a gathering passes it, are people's choices, each scored by what they forecast for their
household and those who regard them. A save binds to the template's id: a save whose laws name a
template the content no longer has is refused.

| Field | Meaning |
|---|---|
| `does` | What the kernel does under it: `common_store`, a share of each household's threshed grain paid into a store the polity keeps at the hearth, and food from it for a household short of food that asks; `keep_store`, a named adult keeps the common store under their own roof and gives from it at their home (the office's first form, below); `against_taking` (content API 36), whoever the gathering finds took from another household's store gives back what was taken, with what the bundle adds (below); `keep_watch` (content API 37), a named adult keeps watch over the households' stores at night (below); `curfew` (content API 38), nobody may be away from home in the hours it sets, but the watch at its rounds and those at a gathering (below); or `amend_body` (content API 40), the custom itself changes: who belongs to the deciding body, how many must come and how it decides (below). Content API 49 adds `repeal`: a law that ends the one it names, with nothing in its place; passed, it is carried rather than in force, and the law it names is superseded. Content API 59 adds `claim_place`: the polity claims the places its people saw people of another settlement working within the year, unclaimed, each named in the law when it is proposed; its own people's use is unchanged (ADR-0020 §5). Content API 67 adds `build_crossing`: the polity builds a crossing at the site its sponsor names, each household asked an equal share of the work (below). Content API 78 adds `close_source`: nobody may draw at the source of water its sponsor holds sickens, kept by each person's choice (below). Content API 79 adds `dig_well`: the polity digs a well beside the hearth, each household asked an equal share of the work (below). |
| `answers` | The issues whose presence makes proposing it a move: `food_short` (the settlement's food ran short within the year, or a household's food will not last until its next harvest); `store_unkept` (a common store is in force and holds food, and no one keeps it); `takings` (a household of the settlement found food taken from its store within the year). At least one. Content API 40 adds `overruled`: a gathering the person came to within memory decided against where they stood, or they hold a grievance against the gathering; it opens moves to them alone. Content API 49 adds `founding`: within `founding_days` of the custom being taken from the gathering, while laws the old custom made are unweighed; it opens to every member of the new body an end to each of them. Content API 59 adds `outsiders`: a household of the settlement saw people of another settlement working, within the year, a place its own people work and the polity does not claim. Content API 67 adds `fords`: a household of the settlement wades a stream where a crossing of a system its people know could stand, within the work's walk of the hearth, and none stands. Content API 78 adds `sickness` (someone of a household of the settlement lay sick at home lately, as its members saw) and `source_suspected` (someone holds, from their own tally, that a source of water the polity does not close sickens; it opens moves to them alone). Content API 79 adds `water_far` (a household of the settlement walks far for water, as it last walked it, so far that its people use less water for it, and a well dug together beside the hearth would save the settlement's households more walking over its lining's life than its work). |
| `levy_shares` | `common_store` only: the shares of threshed grain a sponsor may propose (1 to 8 of them, each above 0 and below 1). A `keep_store` or `keep_watch` template leaves it out. |
| `relief_days` | `common_store` only: the most food one ask brings, in days of the asking household's need (above 0, at most 365). A `keep_store` template leaves it out. |
| `hours` | `curfew` only (content API 38): the hours a sponsor may propose, 1 to 8 pairs `[from, to]` of hours of the day (0 to 23, different), the curfew running from the first to the second, past midnight when the second is the smaller. Any other template leaves it out. |
| `members`, `quorum_shares`, `pass` | `amend_body` only (content API 40): who may belong to the body (1 to 4 of `adults`, `elders` (each household's eldest of an age to keep one), `landholders` (the adults of households holding a field), `watch` (content API 50: those who keep the watch by a law in force)), the shares of members who must come (1 to 8, each 0 to 1), and how it decides (1 or 2 of `more_for`, more for than against, and `two_thirds`, two of every three who take a side). They combine into bodies; a sponsor weighs those one change away from the custom they live under, and only if the new body would keep them in it. Any other template leaves them out. |
| `[bears]` | Optional (content API 43): how a law of the template bears on each value (`"core:value/security" = 1.0`), from -1 (against it) to 1 (for it); each must name a value that exists. What a law does to what someone holds dear weighs, beside their household's lot, in their anchor on its question, their stance at its gathering and how they weigh proposing it. An authoring judgement, not a measurement. |
| `question` | Optional (content API 41): the question people take positions on, in words to follow "on" ("whether to keep a common store"). Each adult then holds a position on it, anchored in what their household's lot makes of the template's law in force, or else its middle level, and moved by talk at the hearth (the `[opinion]` table). Left out, nobody holds one. |
| `bundles` | `against_taking` only: the sanctions a sponsor may propose, 1 to 8 tables of `compensation_days` (food to the household taken from beyond what was taken) and `fine_days` (food to the polity's common store), each 0 to 365 days of the taker's household's food, and `exile` (the one found is sent from the valley; default false). Research 09-07 §1.2 keeps restitution, compensation and a fine apart; §6.2: a sanction is a bundle, never one severity number. |

The core pack has `core:policy/common_store`, at a twentieth, a tenth or a fifth (research 09-01
§3.3 gives 0-30 % of the harvest as an uncalibrated starting range; §3.2 warns a 10-30 % tribute
"is not a universal harvest tax"), at most five days of food an ask (a tuning value; 09-17 §1.5: a
public store combines storage with allocation), `core:policy/keep_store`, the storekeeper, and
`core:policy/against_taking`, with four bundles: restitution alone; three days' compensation;
three days' compensation and three days' fine; and that with exile (tuning values, reckoned in
days of household food as 09-07 §2.3 advises).

A law against taking is weighed by each household from what it knows: what it would recover of
what it lost in the past year to takers one of its members knows of, with the bundle's
compensation, less what its own members took, with compensation, the fine and (for exile)
`exile_days` of its food, at the chance its grown members believe a taker runs of being seen.
Households taken from since the last review may propose it. Cases under it are heard by the
gathering, as laws are decided; a finding imposes the bundle as obligations on the accused's
household, and exile sends the accused from the valley, recorded as a leaving.

A `keep_watch` law names its holder as a `keep_store` law does: its sponsor nominates the adult
they regard most (themselves at full regard). A household weighs it by `watch_guard` of all it
found missing in the past year, whoever took it, back to it, and as much of what its own members
took from it. The holder walks rounds at night as a chosen activity (`behavior = "watch"`,
`core:activity/keep_watch`), and the law lapses when they die or leave.

A `curfew` law sets its hours from the template's (the core pack's `core:policy/curfew` offers
21:00 to 5:00 and 23:00 to 4:00, tuning values). A household weighs it by `curfew_guard` of all it
found missing in the past year back to it and as much of what its own members took from it, as it
would a watch, less `curfew_cost_days` of its food a year for each hour the curfew runs and each
grown member it keeps at home. Under a curfew in force, someone who knows of it weighs keeping it
against any option that would take them off their home's plot (20 m) in its hours: the people
profile's `[polity]` `comply_base`, the norms they hold that the gathering binds (the `norm` kind),
`w_stance` for where they stood on it and `w_regard` for their regard for its sponsor, as a levy is
weighed (research 09-06 §1.5), never below nothing. The watch
at its rounds and those at a gathering are exempt; breaking it carries no sanction in v0, and the
law counts the times it was broken by those who knew of it and by those who did not. Closing a
place (a grove, a fishing ground) is not a kind yet: a household could not forecast it without a
record of what it gathers.

A `keep_store` law names its holder. Its sponsor nominates the sheltered adult they regard most
(themselves counted at full regard), each weighs what a year's spoilage saved under a roof is worth
to their household, and the gathering decides as for any law. While the holder lives in the
settlement the store spoils as a roofed store and relief is asked at their home; when they die or
leave the law lapses (the chronicle says so), the store is unkept again and the issue is open for
a new proposal: succession is a new law, not a rule written in advance (research 09-03 §3.1, §5.1).

Content API 105 (M7a slice BN, ADR-0024 §2) brings an office's tenure. A `keep_store` or
`keep_watch` template may carry a `[tenure]` table: `term_days` (1 to 3650) the office is held
from the day the body passes it, `renewable` (default true) whether its holder may be named or
drawn for the next term, and `fill`, `named` (the default: the sponsor nominates, as before) or
`lot`. Filled by lot, a law names nobody; once it passes, a lot keyed by the law draws its holder
from those who stood for it at the gathering (a design prior: wanting the office kept is taken as
willingness to keep it), less the last holder where the office is not renewable and anyone
keeping the watch already. With nobody to draw, the law lapses. At its term's end the office
lapses, as at a death, and the issue opens again. The core adds
`core:policy/your_turn_keeping_the_store` (half a year, by lot, not renewable) and
`core:policy/your_year_keeping_the_store` (a year, named, renewable).

Content API 106 (M7a slice BO, ADR-0024 §1, §3, §6) brings councils and their elections. An
`amend_body` template may add three council levels, given together: `seats` (3 to 31 each),
`seat_methods` (`election` or `lot`) and `seat_terms` (30 to 3650 days each); its bodies are then
councils, one for each combination, whose `members` name their electorates (`adults`, `elders` or
`landholders`). A council is one change from a gathering of its own electorate, so a sponsor
weighs it as any other amendment: each household judges it by how the council the sponsor expects
would have decided what its members saw decided (a design prior: by election, the n of the
electorate the sponsor knows who stand highest in counsel would sit; by lot, the electorate's seen
stances in proportion). Under a council the body is those who sit; its quorum counts its seats,
filled or empty; only they propose and come to its sittings. An election is called the midnight
after the custom becomes a council and again as the seats' term nears its end; word of it goes
round as word of a gathering does; each of the electorate who hears of it chooses once whether to
stand; when names close, a field no larger than the seats sits unopposed, a lot draws among the
names, or the polling day comes, when each of the electorate who heard of it may walk to the hearth
(the `core:activity/poll` activity, behaviour `poll`) and name up to n of those who stood, ranked
by one draw at a fixed scale over regard, their own household and standing in counsel; the block
vote counts the names, a keyed lot settling a tie for the last seats. Nobody standing or nobody
polling fails, recorded, and the seats stay empty until the next election. The people profile's
`[elections]` table holds the days names are open, the minutes polling takes, the points of
standing (`w_sit`, `w_standing`, `sit_cost`, `stand_temperature`), a voter's weights (`w_regard`,
`w_household`, `w_counsel`, `w_self`, `vote_scale`) and the points of coming to poll (`poll_base`,
`w_poll_stake`). The core adds `core:policy/seat_a_council` (councils of 5 or 9 for 360 days, by
election or by lot, quorum half, more for than against).

Content API 139 (M8b slice CR, second part) brings the people profile's `[moving] w_walk`
(required; 0 to 100 points): what moving is worth for the minutes a day a household's members
travel, as it holds them, less what a resident of the place weighed has travelled a day, as a
share of a 16-hour waking day. The core sets it at 4, as `w_fed`.

Content API 138 (M8b slice CQ, third part) brings the policy kind `build_way` and the issue
`carts_wait`: where carts of the settlement's households wait at stretches of trail one cart wide,
and a passing place cleared together at a stretch's middle would spare them more waiting over its
life (the land profile's `passing_life_years`) than its work, the polity may clear it, each
household asked an equal share; it levies nothing. The core adds
`core:policy/yours_to_clear_a_passing_place`; it asks no question, so it enters no monthly
opinion, and its id sorts after every earlier template's, so their indices, which key opinion's
draws, are unchanged.

Content API 137 (M8b slice CQ, second part) brings the activity behaviour `way`: walk to the passing
place the household is making on its carts' way, clear and level ground there, and walk home, the
work's own labour (the land profile's `[way_works]`: `passing_width_m`, the width it is cleared to;
`clear_h_m2`, hours a square metre; `passing_life_years`, the years a household reckons it lasts;
optional, with defaults of 2.0, 0.34 and 10). The core adds
`core:activity/work_with_wooden_spade_on_passing_place` (from 14, one to four hours a session).

Content API 136 (M8b slice CP, third part) brings the activity behaviour `cart`: work at home on a
cart the household is making at its door, until its vehicle's `labour_h` is done, no more than its
`crew` at once in a day; the cart's `material_kg` of its `good` comes out of the store as it is
begun. The core adds `core:technique/wheel_and_axle` (research 07-08 L05, L11: needs jointed
framing and the shaping of wood, tried in building and boat work with timber, E50 50,000 hours, a
far technique), which `core:vehicle/handcart` now needs, and
`core:activity/work_with_wood_on_handcart` (from 14, two to four hours a session; tuning values).
A technique that no activity, building, well or treatment gates but a vehicle's making is learnt
at home from the age of that work.

Content API 135 (M8b slice CP) appends the vehicle mode `cart`: wheels pulled by a person. A
cart names `tare_kg` (its own mass), `way_resistance` and `trail_resistance` (rolling resistance
on a way that serves carts and on a worn trail; research 10-07 §1.2 illustrates 0.02), `pull_n`
and `power_w` (its puller's most and sustained), and `brake_grade` (the steepest grade down they
can hold it back on). A boat's `draft_m`, `paddle_m_s` and `slack_share` are a boat's only: each
mode refuses the other's fields. The core's `core:vehicle/handcart` (M8b slice CP, second part)
carries 150 kg, weighs 40 kg and is 0.9 m across, pulled at 200 N and 75 W, its resistances 0.02
on a way and 0.05 on a worn trail, braked down 15 % at most (design priors where the reports give
ranges); it loads at a boat's 10 kg a minute. Its makers need wheels on an axle (the third part).

Content API 134 (M8b slice CO, third part) brings the activity behaviour `boat`: work on a boat
the household is making, on the bank where it will be drawn up, until its vehicle's `labour_h` is
done, no more than its `crew` at once in a day. The core adds `core:activity/work_with_wood_on_dugout`
(heavy work with an axe, as work on a crossing is; its sessions and effort are tuning values as
there). The trunk is felled where it is worked, so nothing is taken from the store.

Content API 133 (M8b slice CO, second part) brings a vehicle's `handling_kg_min` (kilograms a
person loads or unloads a minute; ADR-0029 §5: handling is people's work, 12-05 §2.3's 5-20) and
`slack_share` (the share of a reach's mean velocity it meets going upstream along the banks,
ADR-0029 §5 as amended). The core's dugout handles 10 kg a minute and meets 0.4 of the mean
upstream: design priors, marked in its file.

Content API 132 (M8b slice CO, ADR-0029 §2) brings vehicles: a new content kind (`kind =
"vehicle"`, files under `vehicle/`), a made thing that carries more than a person can. It names
its `mode` (`boat`, the only one yet), the `technique` and `skill` its makers need and practise
and a `tool` they work with (a good with `[tool]`; each may be empty), the `good` it is made
from and `material_kg` of it, `labour_h` and the `crew` who may work on one at once, its
`payload_kg`, the `draft_m` it needs loaded, its paddlers' `paddle_m_s` through still water, its
beam `width_m` and its `life_years`. Using one needs no technique; a technique a vehicle names
gates work (E3005). A boat made of one is a record of its own, not a good in a store: it belongs
to a household and stands at the bank where it was left. A reach carries it where its stage at
the day's discharge is above its draft, at its paddlers' speed plus the reach's mean velocity
downstream and less it upstream. The core's `core:vehicle/dugout` (research 07-08 W02) carries
400 kg at a 0.3 m draft and 1.2 m/s, is made with an axe by `core:technique/wood_shaping` in 200
hours from a 1,300 kg trunk, and lasts 12 years: design priors inside 07-08 §2.1's ranges, each
marked in its file.

Content API 131 (M7d slice CE4c, ADR-0027 §6) brings foreign coin. A coin's `[money]` may give a
`foreign_discount` (below 1) and `foreign_assay_h`: one who cannot assay pays with another polity's
pieces first and takes them by weight, the metal in the lightest it holds at what the metal is
worth to it, less the discount and the hours of weighing a piece, unless what pieces bought it says
more. The core's coin: a tenth off, 0.02 hours. `[money]` hours and the discount are now checked.

Content API 130 (M7d slices CE4a-b, ADR-0027 §6) brings assay and debasement. A coin good's
`[money]` may name the `metal` it is struck of and the technique (`assay`) whose knowers can tell
how much metal a piece holds: a household with such a member values coin by the metal in the
pieces it would pay with, at its own cost of the metal (or the levy's floor, if higher), and pays
with the pieces holding least metal first; others count pieces alike and pay with the oldest. A
second `mint` template may strike other terms: passed while a mint is in force, it takes that
mint's place. The core's coin is struck of copper and assayed by those who know smelting; the core
adds `core:policy/mint_light` (6 g pieces).

Content API 129 (M7d slice CE3b, ADR-0027 §6) brings a levy in coin. A `common_store` policy may
have a `[medium]` table: `good` (a money) and `per_kg` (units of it that pay for a kilogram of the
grain owed). A household that holds some pays its levy in it when what a unit would otherwise bring
it is worth less than the grain it pays for; and under such a law in force, which its people know,
a unit is worth to its holder at least the grain it pays for, less as it holds more than the levy
could take of it. The core's common store takes coin at two pieces a kilogram.

Content API 128 (M7d slice CE3a, ADR-0027 §6) brings the polity's first spending. A policy may do
`pay_work`, with a `[pay]` table: `good` (a money) and `per_hour` (units of it the store pays for
each hour a household gives to the polity's crossing, well, flood bank or palisade, as far as the
store holds it); it answers the issue `coin_idle` (the store holds money while a law asks for
common work). The core adds `core:policy/pay_work` (8 pieces of coin an hour).

Content API 127 (M7d slice CE2b, ADR-0027 §6) brings the mint. A policy may do `mint`, with a
`[mint]` table: `recipe` (whose first input is a metal paid by weight and first output a coin),
`mass_g` and `fineness` (each piece of the issues struck under it) and `fee` (the share of the
pieces struck that the polity's store keeps, below 1); it answers the issue `weighed` (households
paid or were paid in weighed metal lately) and names a keeper who knows the recipe's technique.
The people profile's `[market]` adds `strike_points` (what striking is worth to the keeper as the
duty of their office, 12). The core adds `core:policy/mint` (8 g pieces of pure copper, a 3 % fee),
`core:recipe/strike_coin` and `core:activity/strike_coin` (2 hours and 5 kg of firewood a
kilogram), and counts a piece of coin in about a second (`verify_h = 0.0003`).

Content API 126 (M7d slice CE, ADR-0027 §6) brings coin. A good's `[money]` table may say `coin =
true`: it is counted in pieces of issues a mint struck, each issue (issuer, day, grams a piece,
fineness) kept apart in its holders' purses, payments moving the oldest issue first. The core adds
`core:good/coin` (0.005 hours to count a piece, a second from API 127). Nothing strikes coin yet.

Content API 125 (M7d slice CD, ADR-0027 §5) brings money's worth. A good may have a `[money]`
table, `verify_h` (hours a holder spends weighing and testing a unit): it is paid by weight, and a
holder values a unit at the greater of its own cost and what a unit bought in its purchases its
markets remember, at its own costs and faded by the market's `memory_days`, less `verify_h`;
beyond `money_days` of its spending in it (the people profile's `[market]`, 14), what a unit buys is
worth less to it, by balance ÷ held. The core gives copper `[money]` (0.1 hours a kilogram).

Content API 124 (M7d slice CE, ADR-0027 §6) brings metal. A tool's `[tool]` table may name
`serves`, another tool (one that serves for none) whose work it does as well: work that needs
that tool may be done with this one, the longest-lived at hand is worn first, and a household
needing that tool makes whichever of the ways it can work costs fewer of its own hours for each
hour of use; a material such a way needs that the household makes itself is made for it. The
core adds copper: `core:good/copper_ore` (picked from upland surface patches, resource
`copper_ore`, and dug from rare bodies at deposits), `core:good/copper`, `core:good/copper_axe`
(serving for `core:good/axe`, 600 hours), the technique `core:technique/copper_smelting`
(practised as pottery) (found by those who fire kilns with ore and fuel at home;
founders do not bring it), the recipes `smelt_copper` (20 kg of ore and 40 kg of firewood a
kilogram) and `cast_copper_axe`, and the activities `gather_ore`, `dig_ore`, `smelt_copper` and
`cast_copper_axe`.

Content API 123 (M7d slice CC, ADR-0027 §4) brings lending from the store. A policy may `does =
"lend_from_store"` with a `[lending]` table: `increment` (what is owed on a loan beyond what was
lent, a share of it) and `share` (the most of the food the store holds it lends at once). The core
adds `core:policy/lend_from_store` (a tenth more, a quarter of the store), answering `food_short`.

Content API 122 (M7d slice CB, ADR-0027 §3) brings *debts heard*. A policy may `does =
"debts_heard"`, answering the appended issue `debts_unpaid` (a loan a household made was refused
or left unpaid, and nothing was taken for it), with a `[debts]` table: `remedy` (`time`, the loan
open again and due `extend_days` after the finding, or `imposed`, what is still owed imposed as a
finding's obligation) and `share` (what households expect a finding to move, for their
forecasts). The core adds `core:policy/debts_heard` (imposed) and
`core:policy/debts_heard_time` (180 days more); one of them in force replaces the other.

Content API 121 (M7d slice CB, ADR-0027 §3) brings wage arrears: the `[credit]` table adds
`wage_days`, the days a workshop has to pay wages it could not pay before it fails.

Content API 120 (M7d slice CB, ADR-0027 §3) brings pledges and answers at the due day. The
`[credit]` table adds `field_recovery` (the share of a debt a lender believes the use of a field
pledged recovers), `guarantor_regard` (the least regard a household's elder holds for a
borrower's to stand its guarantor), `keep_days` (days of its food a debtor keeps back when it
pays), `extension_days` and `max_extensions` (how far, and how often, a creditor puts the due day
off), `extend_regard` (the least regard a creditor's elder holds for the debtor's to grant time),
and `pay_base`, `w_pay_regard` and `w_pay_cost` (points toward paying at the due day over
refusing: a base, regard for the creditor, and the share of the debtor's food the debt would
take). `p_max` is now the believed loss, after what a pledge would recover.

Content API 119 (M7d slice CA, ADR-0027 §1-2) brings loans in goods. The people profile adds a
`[credit]` table: `review_days` (how often a household asks itself whether its food will last to
its harvest), `least_days` (the least days of its food it borrows), `customary_increment` (the
least increment a lender asks, a share of what it lends), `max_increment` (the most a borrower
accepts), `p_prior` (the chance a lender believes a loan to a stranger is not repaid), `p_max`
(above which it refuses), `keep_return` (what keeping the food at home earns a lender a year),
`lend_share` (the most of what it can spare a lender lends), `max_asked` (lenders asked at one
review) and `grace_days` (days after the due day before a loan is defaulted). The `[ties]` acts
gain `lent`, `repaid` and `defaulted`.

Content API 118 (M7c slice BZ, ADR-0026 §5) brings closure. An amendment's `members` may name
`organized` (the adults of households in an organization by stake the polity recognizes) and
`privileged` (those of households in one it grants a privilege in force); while the polity
recognizes, or grants, none, such a body has nobody, and its gatherings fail for want of a
quorum, recorded. The core adds `core:policy/close_the_custom`, answering `overruled`.

Content API 117 (M7c slice BY, ADR-0026 §4) brings corruption and patronage. The people
profile's `[crime]` table adds `divert_share` (the most of a common store's food its keeper draws
for their own household at one review), `favour_reserve_days` (below this many days of the
settlement's food in the store, its keeper answers asks by their regard) and `favour_floor` (the
regard below which they pass one over), `collector_sight` (the chance a duty's collector sees a
purchase brought home) and `collector_ask_share` (the share of what is owed they ask to assess
nothing). A duty's `[duty]` table may say `collector = true`: its law names a collector. The
`[polity]` table adds `w_owed` (the points a stance gains for each day of its household's food a
patron's gifts brought it), `owed_memory_days` and `patron_gift_days` (the most one gift is). The
core adds `core:policy/duty_on_tools_collected`.

Content API 116 (M7c slice BX, ADR-0026 §3) brings privileges. A policy may `does = "privilege"`
with a `[privilege]` table, `grant` (`licence`: the sole right to offer goods of its `purpose` at
doors in the settlement; `staple`: the sole right to sell there such goods fetched from another
settlement) and `purpose` (`food`, `fuel`, `material`, `tool` or `store`), and the `bundles` a
finding of a breach imposes (as a law against taking's, its compensation going to the household
that brought the case). It names a recognized organization when proposed. A forum's `hears` may
name `privilege`. The core adds `core:policy/licence_tools` and `core:policy/staple_tools`, and the
elders and the panel hear privileges.

Content API 115 (M7c slice BW, ADR-0026 §2) brings organizations by stake and their recognition.
The people profile's `[faction]` table adds `organize_days` (the cost of organizing, in days of the
founder's household's food), `w_stake` (the points a duty worth that much a year brings, and what
organizing itself costs), `stake_admit` (`open`: anyone who sells the goods; `sponsor`: one a member
speaks for), `stake_decides` (`organizer` or `members`: who decides an application) and
`stake_dues_share` (the share of each threshing a member gives, at least 0 and below 1). A policy
may `does = "recognize"`, which takes nothing of its own: it names an organization by stake its
proposer's household belongs to, and while it is in force that organization's members' households
owe none of the polity's duties on goods of what it sells. The core adds
`core:policy/recognize_organization`, answering `fetched_elsewhere`.

Content API 114 (M7c slice BV, ADR-0026 §1) brings trade laws. A policy may `does = "duty"`, its
rates its `levy_shares` (above 0 and below 1) and a `[duty]` table: `purpose` (`food`, `fuel`,
`material`, `tool` or `store`: the goods it is laid on) and `exempt_sellers` (households that offer
such goods at home are spared it); or `does = "embargo"`, which takes nothing of its own: it names
the settlement its people bought most from when proposed. Both answer the new issue
`fetched_elsewhere`. The core adds `core:policy/duty_on_tools` and `core:policy/embargo`.

Content API 113 (M7b slice BU, ADR-0025 §5) brings the norm that findings bind. A norm may
`does = "abide_by_findings"`: its points weigh toward meeting what a forum's finding imposes, as
`abide_by_laws` weighs toward meeting the gathering's, and its `[expect] share` and `tell_days`
say how often companions at the hearth tell whether their household met what forums imposed on it,
and for how long that is worth telling. The core adds `core:norm/findings_bind`.

Content API 112 (M7b slice BT, ADR-0025 §3-4) brings limits and review. A policy may `does =
"limit"` with a `[limit]` table: `predicate` (`daylight_work`, with `minutes`: the most of the
longest day's light a curfew may cover, 1 to 720; or `levy_share`, with `share`: the most of a
harvest a law may levy, above 0 and below 1) and `change` (`more_for` or `two_thirds`: the rule
its own repeal is decided by). It answers the `overruled` issue and is weighed only as an amendment
is. A `[forum]` table may add `review = true`: that forum hears challenges of laws in force under
limits in force, brought by those a law costs. The people profile's `[crime]` table adds
`w_reading`. The core adds `core:policy/limit_daylight_work` and `core:policy/limit_levy`, and
gives review to the elders and to the hearer bound by findings.

Content API 111 (M7b slice BS, ADR-0025 §2) brings precedent. A `[forum]` table may add
`authority`: `informative` (the default: sitters follow no finding but as they believe it),
`persuasive` or `binding`. The people profile's `[crime]` table adds `recall_k` (3 to 8: how
many remembered findings a sitter recalls, those whose fact tags match the case's most, newest
first), `w_persuasive` and `w_binding` (the points a recalled finding moves a sitter toward its
direction, times the share of tags matched, under each authority). Every finding is remembered by
those who heard it as a claim and told as news. The core makes the elders persuasive and adds
`core:policy/hearer_by_precedent` (a hearer bound by findings).

Content API 110 (M7b slice BR, ADR-0025 §1) brings forums. A policy may `does = "hear_cases"`
with a `[forum]` table: `seat` (`holder`, one the body names, as an office; `panel`, `panel` of
3 to 15 drawn by a keyed lot from the body's members for each sitting; or `elders`, the eldest
adult of each household), `hears` (the kinds of law whose cases it hears: `against_taking`,
`against_raiding`), and for a panel or the elders `quorum_share` (default 0.5) and `pass`
(default `more_for`). It answers the `cases_waiting` issue. A case brought under a law it hears is
heard by it the midnight after, nobody of the two households sitting; before then it may be
settled or withdrawn. The core adds `core:policy/name_a_hearer`, `core:policy/hear_by_panel` and
`core:policy/hear_by_elders`.

Content API 109 (M7a slice BQ, ADR-0024 §6) prices the evenings a council saves. The people
profile's `[elections]` table adds `w_evening`: the points an evening at the gathering costs one
who comes. When a household judges a council by the decisions its members saw, it also counts,
for each, its people who came and would not sit under the council the sponsor expects, at
`w_evening` each (the same points a proposal and a stance weigh, over `polity.w_gain`).

Content API 108 (M7a slice BQ, ADR-0024 §1) lets a council's seats be shared among a polity's
settlements. An `amend_body` template with council levels may add `seat_districts`: `at_large`
(the whole polity elects every seat, as before) and `by_settlement` (each settlement it holds is a
district electing its share). A council by settlement is offered only to a polity that holds more
than one settlement. When its election is called, the seats are shared out by the size of each
settlement's electorate then, by the largest remainder (a tie to the seat's settlement first);
each who stands stands in the district they live in, for its seats; each voter polls at their own
hearth for their own district's seats, as many as it has; each district is counted apart, and one
whose field is no larger than its seats sits unopposed. `core:policy/seat_a_council` offers both.

Content API 107 (M7a slice BP, ADR-0024 §3-5) brings campaigns, positions and slates to
elections. The people profile's `[elections]` table adds `positions_told` (0 to 3: how many
questions one who stands says where they stand on, those that matter most to them), `w_distance`
(a voter's utility against the mean distance from where a candidate said they stand, on the
questions the voter holds a position on, once they have heard of the candidacy), `w_record` (for
how the candidate's seen stances lined up with what the decided laws meant to the voter's
household, −1 to 1), `w_slate` (for a candidate the voter's own faction backs) and `w_campaign`
(points one who stands finds in an evening at the hearth while their election runs).

Content API 31 (M4a slice Z) brings the polity: the `policy` kind, the people profile's
`[polity]` and `behavior = "attend"` (`core:activity/attend_gathering`, adults only). The core
pack reviews weekly (ADR-0014 §4; research 09-01 §3.3: 7-30 days), calls the gathering the next
evening for two hours, needs a quarter of the adults (no figure in the reports), believes in one
lean year in four before it has seen any, takes a lean harvest as half an ordinary one and half a
year's food as the edge of subsistence. All are tuning values.

Content API 32 (M4a slice Z) weighs leaving: the people profile's `[household]` gains
`leave_w_gap`, `leave_w_stake` and `leave_stay`. In the core pack a household with nothing to wait
on goes at nearly `leave_per_day`, and one that others or the store could carry to its harvest at
about a tenth of it (tuning values).

Content API 33 (M4a slice Z) brings the storekeeper: `does = "keep_store"`, the `store_unkept`
issue and `core:policy/keep_store`. `levy_shares` and `relief_days` became optional and are checked
per kind: required for `common_store`, refused for `keep_store`.

Content API 34 (M4a slice Z) adds `vote_memory_days` to the people profile's `[polity]`: a year in
the core pack (a tuning value). Before it, a sponsor who had watched a fifth turned down 1–23
could propose it again the next week; in one run a village voted 27 times on a fifth or a tenth
before it passed a twentieth.

Content API 35 (M4b slice AA) brings taking: the people profile's `[crime]`, `behavior = "take"`
(`core:activity/take_food`, from 12) and the tie acts `took_from_us`, `took_from_others`,
`restored` and `refused_restitution`. In the core pack about one founder in six falls below the
moral filter. Research 04-09 §5.4 finds no established figures for any of these weights: all are
tuning values, to be tested.

Content API 36 (M4b slice AB) brings cases: the policy kind `against_taking` with its `bundles`,
the issue `takings`, `core:policy/against_taking`, and the `[crime]` keys `report_cost`,
`w_case_belief`, `w_comply_found` and `exile_days`, all tuning values.

Content API 37 (M4b slice AC) brings the watch: the policy kind `keep_watch`
(`core:policy/keep_watch`), the behaviour `watch` (`core:activity/keep_watch`, its stand the
activity's `min_minutes`), and the `[crime]` keys `watch_guard`, `w_watch`, `rounds_per_night`,
`round_stops`, `w_watch_report` and `ask_days`, all tuning values.

Content API 38 (M4b slice AD) brings the curfew: the policy kind `curfew` with its `hours`
(`core:policy/curfew`), and the `[crime]` keys `curfew_guard` and `curfew_cost_days`, tuning
values (the research gives no figure for what a curfew costs or stops).

Content API 45 (M4c slice AH) brings factions: the people profile's `[faction]` table (how often
each adult reviews where they belong, the weights of a grievance, of regard for an organizer, of
those one knows belonging and of dues, the thresholds, the founding cost and how long a founder
whose faction ended waits before founding another, the dues share, the
reserve past which a store asks no more dues and how much one ask of it may bring), all design
priors (research 04-10 and 09-12 give
no founding size or joining rate).

Content API 46 (M4c slice AH, step two) brings petitions: the behaviour `petition`
(`core:activity/petition`, adults who heard of one, the evening it sits) and the `[faction]` keys
`petition_members` (the fewest members before an organizer weighs calling one), `petition_days`
(days between one faction's petitions), `petition_cost` (what calling one costs the organizer,
points), `w_member` and `w_expect` (the points belonging and the share of those one knows who
belong add to coming), `free_ride_share` (the share of people for whom more others coming makes
their own coming matter less) and `refused_days` (the wrong a petition turned down is to each who
came, in days of a household's food), all design priors (research 04-10 §1.4, §5.3 give the terms
and no values).

Content API 47 (M4c slice AH, step three) brings refusals of a levy: the `[faction]` keys
`refusal_cost` (what calling on members to keep back a levy costs an organizer, points, beside
what the norms they hold weigh) and `refusal_days` (how long the call stands), design priors.

Content API 48 (M4c slice AI, step one) brings revolts: the `[faction]` keys `revolt_cost` (what
calling on everyone to stand with a faction's program costs an organizer, points, beside what the
norms they hold weigh), `revolt_days` (how long the call stands), `hold_days` (how long every
officeholder, and more adults than stand with the gathering, must stand with it before it holds)
and `w_exclusion` (what the new body leaving one out weighs against their standing with it,
points), design priors (`hold_days` is research 09-11 §2.2's seven days).

Content API 79 (M6a slice BA, step four) brings a well dug together by law. A `policy` may `does
= "dig_well"`: the polity digs a well beside the hearth, a metre below where its people expect
water, of the system its people know whose walking saved over its lining's life most exceeds its
work; once passed it is begun, the polity's, each household is asked an equal share of its work
(given as each person chooses, weighing the gathering's word), and once it meets water the
polity's people may draw at it. The issue `water_far` may be answered: a household of the settlement walks far for water, as it
last walked it (so far that its people use less water for the walk: from the first point of the
people profile's `water_use_by_walk`), and such a well would save the settlement's households
more walking over its lining's life than its work. The core adds `core:policy/works_well`.

Content API 78 (M6a slice BA, step four) brings closing a source by law. A `policy` may `does =
"close_source"`: nobody may draw water at the source its sponsor names, one they hold sickens;
keeping it is each person's choice, as a curfew's is, and someone with nowhere else they know to
draw draws there still. The issues `sickness` (someone of a household of the settlement lay sick
at home lately, as its members saw) and `source_suspected` (someone holds that a source the polity
does not close sickens; it opens moves to them alone, as `overruled` does) may be answered. The
people profile's `[suspicion]` gains what a household believes a source's sickness costs it:
`case_days` (0 to 365: days of work a case among its people costs, abed and tended),
`case_death_share` (0 to 1: the share of cases it believes die) and `w_death` (0 to 100,000: days
of work a death is counted as); the core's 8, 0.2 and 1,000 are design priors (research 05-03 §3.2:
untreated clinical cholera can kill about half, prompt rehydration fewer than one in a hundred,
neither the share of everyone who takes it). The core adds `core:policy/works_source_closed`.

Content API 77 (M6a slice BA, step three) brings answers to suspicion. A `well` kind gains
`cover_h` (0 to 1,000: hours to make one a lid and curb, owed by a household that suspects its
own well) and `covered_runoff` (0 to 1: the share of the runoff an open mouth would take that a
covered one takes); the core's timber-lined well takes 8 and 0.1, design priors. The people
profile's `[moving]` gains `w_sickness` (points for the share of the settlement's households a
household's members know had sickness lately, weighed against staying; the core's 3 is a design
prior set as `w_grievance`).

Content API 76 (M6a slice BA, step two) brings suspicion of a source. The people profile gains
a `[suspicion]` table: `min_households` (1 to 1,000: households known to draw at a source, at
least, before its share counts), `min_sick` (1 to 1,000: of those, households that had sickness
lately, at least), `ratio` (1 to 100: how many times the share among the rest it must be),
`draws_min_days` (0 to 365: faded days of drawing from which a household draws at a source),
`known_min` (0 to 1: a tie's familiarity from which someone knows a household's usual source) and
`review_days` (1 to 365: days between tallies while someone holds news of sickness or a
suspicion). The core's 3, 2, 3, 2, 0.3 and 7 are design priors; research 12-02 §4's 8.5-fold
contrast between water companies orients `ratio`.

Content API 75 (M6a slice BA, step one) brings sickness seen. The people profile's `[word]`
gains `share_sickness` (0 to 1: the chance someone tells a companion at the hearth of a household's
sickness they heard of; research 09-16 §2.2's 0.4-0.9 for urgent, personally relevant news, the
core taking 0.4), `sickness_days` (1 to 365: how long a household's sickness stays news after it
was first seen) and `sickness_spell_days` (1 to 365: how long after it was first seen someone abed
there again is new sickness, seen and told afresh); the core's 30 and 14 are design priors.

Content API 74 (M6a slice AZ, step three) brings care. A disease gives `care_h_per_day` (0 to
24: the hours of a carer's time a day that count as tending someone ill of it) and `care_rr` (0
to 1: what a severe day with that much care does to the chance of dying that day, as a relative
risk; research 05-05 §2.2's prior for organized supportive care is 0.95), and may give
`[[treatment]]` tables, each a `technique` (content id) and an `rr` (0 to 1): when a carer that day
knew it, the least such `rr` holds instead. A technique named only as a treatment gates no work
but is allowed (E3005 counts treatments). The people profile's `[decision]` gains `w_tend`
(points for tending a member of the household severely ill and still owed a day's care, half for
one ill but not severely). The core adds the activity `core:activity/work_tending_the_sick`
(behaviour `tend`: at home, its hours shared among the ill there) and the far technique
`core:technique/fluid_replacement` (found only by tending, at 100,000 hours for half; learnt at
home), which cholera names as a treatment at 0.05; dysentery has none (05-03 §1.4: rehydration
chiefly changes cholera survival).

Content API 73 (M6a slice AZ, step two) moves what people shed: a disease gives what one shedding
person adds to their household's heap a day, in doses (`shed_ill_per_day` while ill and
`shed_silent_per_day` otherwise, 0 to 1e12; a dose drunk is a hazard of one, research 12-02
§5.4's exponential with its rate taken into the dose) and the share of a load that dies a day
outside a person (`decay_per_day`, 0 to 1); a disease that does not pass by water sheds nothing
into the heap. A land profile may give a `[contamination]` section (above). The core's numbers
are design priors and tuning values, each marked in its file.

Content API 72 (M6a slice AZ, step one) brings diseases: a new content kind (`kind = "disease"`,
files under `disease/`), with `routes` (the ways it passes: `water`, `household`), its own clocks
as `[mean, sd]` days of a lognormal (research 05-03 §7.3): `incubation_days` (infection to
symptoms), `shed_lead_days` (how long before symptoms shedding starts; 0 with them), `shed_days`
(how long shedding lasts from its start), `shed_after_days` (how long it goes on after illness
ends) and `ill_days`; `symptomatic` (the share of infections with symptoms), `severe_by_age`
(`[[age, share], ...]`, the share of the ill severely so, interpolated), `severe_death_per_day`
(the chance a day of dying while severely ill: a hazard by stage, never a fate drawn at
infection, 05-03 §7.6), `immunity_years` (`[least, most]`, drawn evenly) and `household_hazard`
(what each shedding member puts on each other member of a household a day; exposure by every
route is summed and drawn once a day, 05-03 §7.2). Nothing about a disease is a rate of cases or
deaths. The core's `core:disease/cholera` and `core:disease/bacillary_dysentery` take their clocks
from 05-03 §2.1; their shares, death hazards and household hazards are design priors and tuning
values, each marked in its file. No world has a disease until the observer brings one (ADR-0021
§8).

Content API 71 (M6a slice AY, step three) brings wells: a new content kind, well systems (`kind =
"well"`, files under `well/`), with `technique` and `skill` (as a bridge system's; the skill is
practised by the work and its lining's quality drawn from it, ADR-0009 §6), `dig_radius_m` (the
radius dug: the earth taken out of each metre is the circle of it, at the people profile's
`digging.h_per_m3`), `radius_m` (inside the lining, the water column's; no more than the radius
dug), `lining_h_per_m` (hours to line a metre), `water_m` (how far below where they expect water
its diggers mean to go), `max_depth_m` (the deepest they go before giving it up), `influence_m`
(03-02 §1.4's radius of influence, over which the column refills from the ground), `lift_min_per_m`
(minutes to haul a load up a metre to the ground), `loss_per_year` (the share of its lining rot
takes a year on ground of wetness 1; the lining gives way, and the shaft falls in, once rot has
taken what its quality left) and `crew` (how many may begin a session of work on one in a day).
The core's `core:well/timber_lined` is dug 1.5 m across and lined 1.2 m inside (research 12-01
§2.2), its lining at 16 hours a metre, at most 10 m deep (03-02 §2.3), and its work is
`core:activity/work_on_well` (behaviour `well`; heavy work with an axe, as digging clay is). The
technique `core:technique/well_digging`, which founders bring (12-01 §3.1: timber-lined wells in
the early Neolithic), is learned at the age of the well work. A technique a bridge or well system
names gates work (E3005). What a well yields is never authored: its water is the column standing
in its shaft, refilled from the water table round it, and what is drawn leaves the water table.

Content API 70 (M6a slice AY, step two) brings water people choose: the `[household]` key
`water_use_by_walk_min` (above). The core's people use 20 L a day while the walk is a quarter of
an hour or less each way, falling to 10 L at half an hour (research 12-01 §2.1: up to 20 L at 5-30
minutes, often under 5 L beyond; its prior for carried water is 10-30 L). Springs are sources:
people fetch from the nearest that flows and has a load left today when it is nearer than the
river or lake.

Content API 69 (M6a slice AY, step one) brings water under the ground: the land profile's
optional `[groundwater]` section and its `[[groundwater.unit]]` tables (above), from which each
world derives its aquifer, settles its water table where the landscape's mean recharge holds it,
and lives its heads and its rivers' flow day by day. The core's valley has alluvial sand under
its water, wet margins and floodplain, silt under the lower-slope woodland and weathered rock
under the rest, each within research 03-02 §2.2's priors.

Content API 68 (M5c slice AW, step three) brings the room to work on a crossing: the bridge key
`crew` (1 to 100), at most how many begin a session of work on one crossing in a day; anyone else
who would work on it that day is told it is full (research 11-11 §1.3: progress is limited by the
usable work front). The core's log footbridge takes 5, the crew research 11-07 §2.3 states for a
simple log footbridge's one to six working days.

Content API 67 (M5c slice AW, step three) brings crossings built together: the policy kind
`build_crossing` and the issue `fords` (a household of the settlement wades a stream where a
crossing of a system its people know could stand and none does), with the core pack's
`core:policy/works_crossing`. Its sponsor names a site their own household wades (one of the
three it wades most); once passed, the polity begins the crossing there, its own, and each
household is asked an equal share of its work, which its people give as they choose, weighing the
gathering's word as a levy's payment is weighed. A household weighs the law by the walking a year
it would save its own people, less its share of the work over the crossing's life, each hour worth
the food an hour of its own work brings (research 11-07 §2.3: foregone production), beside what
the template bears on (`[bears]`). Nothing is authored about where or whether a village builds
one, and a polity builds one at a time.

Content API 66 (M5c slice AW, step two) brings households building crossings: the bridge key
`skill` (the skill its builders practise, from which its members' quality is drawn when it opens,
as a building's groups are, ADR-0009 §6; empty for a middling builder's) and the behaviour
`bridge`, whose activity is the work on a crossing the household is building, its labour the
crossing's. The core's log footbridge trains `core:skill/building`, and its work is
`core:activity/work_on_crossing` (heavy work, an axe, as building a hut is). Nothing is authored
about when a household builds one: it weighs the walking its own recorded wades of a stream would
save over the crossing's life against the work it takes.

Content API 65 (M5c slice AW, step one) brings bridge systems, a new content kind (`kind =
"bridge"`, files under `bridge/`): `technique` (the one its builders need, or empty), `good` (its
members' good, which must have `[timber]` strengths) and `density_kg_m3`; `span_min_m` and
`span_max_m` (the clear spans it may be proposed for; research 11-07 §2.1's envelopes);
`members` and `diameter_min_cm`/`diameter_max_cm` (how many lie side by side and what they may be
cut to); `bearing_m` (how far each end rests on a bank); `deck_factor` (walking speed on it, a
share of dry ground's); `labour_h_per_m` (per metre of member; 11-07 §2.3's guardrails);
`loss_per_year` (the share of the members' section rot takes a year; 11-07 §2.4); `margin` (what
builders size the members to, over their own weight and one walker); and `fall_kills` (the
chance someone on it dies when it gives way). The core's first is `core:bridge/log_beam`, a log
footbridge. What any one bridge carries is worked out from its own members, never authored.

Content API 64 (M5c slice AV, step two) brings performance seen: the `[relations]` key
`performance` (0 to 10; 1.0, as much as a trespass seen, a design prior; research 13-01 §1.5:
trust moves only after a relevant opportunity). When a payment an agreement owes is handed over in
full, or missed, each of the receiving polity's people who knows its own law deciding the
agreement takes that much evidence that the paying polity keeps its word, or does not; a gift
handed over is as much that it helps.

Content API 63 (M5c slice AV, step one) brings goods for leave, and payments carried (ADR-0020
§7): the behaviour `carry` and its activity `core:activity/walk_owed_goods` (one way at most
`max_walk_minutes`, 120, as a visit; daylight only, and only for the one named to carry a payment
set aside), and the `[relations]` keys `gifts_kg` and `transfers_kg` (at most 8 amounts each, 1 to
100,000 kg; 100 and 400 for a gift and 100 for a transfer, design priors: research 13-02 §2.3
gives a ceremonial relation no default share), `transfer_days` (1 to 3,650; 365), `deliver_days`
(1 to 365; 30) and `carry_points` (0 to 100; 12, enough to outweigh the longest walk allowed, at
`decision.w_walk_hour`, by about a morning's useful work), all design priors. `packages` becomes
16, still inside research 13-01 §3.3's 8–32, so the goods packages fit beside leave's. A polity
whose common store is in force may give, for leave to the places the other claims, a gift once or
a transfer every `transfer_days` of the good its store holds most, into the other's store when it
keeps one; a payment falls due when the agreement comes into force, the store sets aside what it
holds of it each midnight, its keeper (or else the one who agreed to the terms) may walk it to the
other's hearth, and one not handed over within `deliver_days` is missed, its cause kept.

Content API 62 (M5c slice AU, step two) brings agreements between polities (ADR-0020 §6): the
policy kind `agreement` (`core:policy/word_given`, answering the issue `claimed_from_us`; it
asks no question and bears reciprocity +0.5 and security −0.5, authoring judgements), and the
`[relations]` keys `packages` (1 to 32; 8, inside research 13-01 §3.3's 8–32 and 09-05 §2.3's
3–8 per round), `answer_days` (1 to 3,650; 120, a design prior) and `terms_days` (at least one
term of at most 36,500 days, 0 for one that runs until a law ends it; 365 and 1,825, from
ADR-0020's menu of a year and five years). Someone
whose household heard that another polity claims places it works, and who may propose at home,
meets the one of that settlement they know best who may propose there; the two weigh leave to
use the places either side claims, for each term, up to `packages` of them, each by their own
household's forecast and the support they predict at home, and agree on the one both expect to
pass and to be worth sponsoring, or part with none. Each sponsors it at home as a law of the
template; it is in force once both gatherings have passed it and each side has heard of the
other's decision from someone of the other settlement. A side not put to its gathering within
`answer_days`, or not hearing within `answer_days` of the later decision, fails it. Leave in force
lifts `claimed_worth` and the trespass for those who know their own polity's law deciding it.
The issue `terms_sought` names why the other side's negotiator proposed it; it answers no
settlement's issue.

Content API 61 (M5c slice AU, step one) brings claims that bind outsiders' choices (ADR-0020
§2, §5): the `[relations]` keys `share_claims` (0 to 1; 0.15, inside research 09-16 §2.2's 0.05–0.25
for routine news) and `claimed_worth` (0 to 1; 0.5, a design prior). Someone who knows a claim
their polity's law in force makes tells a companion from another settlement at the hearth with
`share_claims`, and so does someone whose household heard of another polity's claim; the
listener's household then holds every place the law claims. Word of a claim crosses no other
way. A household that heard of another polity's claim on a place weighs it, when choosing where
to gather or dig, at `claimed_worth` of what it would yield: it goes there only when nothing
else is half as good, and is held to trespass if seen there. A place its own polity claims too it
holds as its own and weighs whole, and since M5c slice AX a household some of whose people are
going hungry weighs no claim at all. A claim heard of is let go once no law in force makes it.

Content API 60 (M5c slice AT, step two) brings views of other polities (ADR-0020 §3): the
people profile's `[relations]` table, with `prior` (0.01 to 100; 1 in the core pack, ADR-0020 §3's
α = β = 1), `half_life_days` (1 to 36,500; 1,826, ADR-0020's five years), `seen_trespass` and
`heard_trespass` (0 to 100 each; 1 and 0.5, design priors). A person holds a view of a polity they
saw or heard of: evidence for and against in three domains (keeps its word, harms us, helps us),
each starting at the prior and fading back to it. Someone who worked a place their polity claims
and knows the claim, and saw people of another settlement work it the same day, holds a
grievance against each of those households (its harm the food that household got there, in days
of their own household's need) and adds `seen_trespass` to their view that the other polity harms
theirs; one told of it at the hearth for the first time adds `heard_trespass`. A view is read by
nothing yet but the observer, and a grievance against outsiders is no reason to leave home.

Content API 59 (M5c slice AT, step one) brings claims on wild ground (ADR-0020 §5): the policy
kind `claim_place`, the issue `outsiders`, the core pack's `core:policy/wild_ground`, the
`[places]` key `use_half_life_days` (1 to 3,650) and the `[polity]` key `claim_keeps` (0 to 1).
Each household keeps the places its people gather from or dig at: the days they worked each and
the food they got there, and, for each other settlement whose people worked it on the same day,
the days it saw them there, all fading by the half-life (180 days in the core pack, a design
prior). Outsiders seen within the year at a place the polity does not claim make the issue; a
claim's sponsor names every such place, and a household weighs it by what it believes outsiders
take a year at the places it works (the food it got at each, times their share of the days
there) times `claim_keeps` (0.5, a design prior: the share a claim would keep for it). Only food
places count in that forecast; a deposit's clay is named but weighs nothing yet. A claim in force
changes nobody's use of a place: what outsiders' use of a claimed place means comes with slice
AT's second step. The template asks no question, so it enters no monthly opinion, and its id
sorts after every earlier template's, so their indices, which key opinion's draws, are
unchanged.

Content API 58 (M5b slice AR, step two) brings the `[knowledge]` key `watch_m` (0 to 1,000 m;
0, never): how far from where they stand, at another settlement's hearth or a seller's door
there, someone sees its people at work well enough to come to know of a technique the work
needs (a design prior). Seeing gives awareness only, never knowing (research 07-02 §1.2); a
good bought there that only one technique makes shows it too.

Content API 57 (M5b slice AR) brings buildings seen in other settlements: the `[style]` keys
`seen_most` (the most a person keeps in mind until their household's next taste review, newest
first; inside research 11-02 §5.5's 5–20 salient exemplars a person, which it calls a prototype
storage choice rather than an empirical figure) and `sight_m` (how far from where they stand
at another settlement's hearth, or at a door they go to buy at, they see its buildings finished in
the year past), both design priors. At the review the household meets them beside its own
settlement's, each once, a stranger's building admired for how well it was built and for the
esteem its people hold its owner's in. `seen_most = 0` turns it off, and a world of one
settlement lives as before.

Content API 56 (M5b slice AP) brings buying from another settlement by report: the people
profile's `[reports]` table, with `half_life_days` (days over which how much a household believes
a price report halves), `max_age_days` (days after which one is let go) and `share_told` (the
chance someone tells a companion at the hearth of an offer elsewhere their household holds a newer
report of; research 09-16 §2.2's 0.05–0.25 for routine news), all design priors; and the activity
behaviour `fetch`, whose `max_walk_minutes` is the farthest one-way walk to a seller's door and
which is reached in daylight. A report is held per market, good and payment; someone who keeps
company at another settlement's hearth tells of what their household and its workshops offer.
Since M5b slice AQ (no new fields), `fetch` also carries an errand: a trip to buy a good
elsewhere to sell at home, planned at the household's weekly review from its reports and its
market's demand, and weighed as making to sell is (ADR-0019 §6). Content without `fetch` has no
replacement anchor and no errands, and lives as before.

Content API 55 (M5a slice AO) brings founding a settlement of one's own: the people profile's
`[founding]` table, with `cost` (points against breaking every field and building where there is
no hearth, store or neighbour yet, besides `[moving]`'s `cost`), `yield_share` (the share of the
believed yield a site is forecast at: research 10-01 §2.3's 10th–30th percentile), `walk_hours`
(the farthest walk from home a site may lie), `candidates` (sites weighed at a review, of those a
member has walked; 10-01 §2.3: 16–64), `buffer_months` (months of food beyond the first harvest a
coalition must hold to go; 10-01 §2.3: 1–3) and `work_h_per_day` (hours a day an adult breaks new
ground, for whether the first crop can be sown this year), all design priors.

Content API 54 (M5a slice AN, step one) brings moving between settlements: the people profile's
`[moving]` table, with `w_kin` (points for each close kin of a member living at the place, less
each living at home outside the household), `w_ties` (at most this for those its members know
there, less the same at home), `w_fed` (for the share of the place's people a member last saw not
going hungry, less the share at home now), `w_grievance` (for the most keenly felt grievance a
member holds), `w_stake` (against the harvest the household's fields here should bring over a
year's need), `cost` (against the work of a new home and new ground before the first harvest) and
`reviews` (how many of the household's yearly reviews running a place must win before it moves
there; research 10-01 §2.3 gives 1–3), all design priors (05-06 §5.1 gives the terms and no
values).

Content API 53 (M5a slice AM, step two) brings visits: the behaviour `visit` (going to the
hearth of another settlement the household knows, keeping company there and walking home the same
day; chosen only in daylight that lasts the walk there), the core pack's `core:activity/visit`
(its `max_walk_minutes`, 120, is a day's range there and back), and the `[places]` keys `w_kin`
(points a visit is worth for each parent, child, brother or sister living there), `w_ties` (at
most this for those the visitor knows there), `w_seek` (for an unpartnered adult who looked for a
partner at home within `seek_days` and found nobody) and `revisit_days` (the days over which the
wish to go again grows back after a member of the household was there), all design priors
(research 04-08 §1.1: partners come from those one meets, neighbouring settlements among them;
04-04 §3.1: visitors, marriage partners and kin are the ties that bridge villages; no study gives
values).

Content API 52 (M5a slice AM, step one) brings known places: the people profile's `[places]`
table, with `sight_m` (metres from another settlement's hearth within which a walk passes in
sight of its homes, so the walker's household knows it) and `share_told` (the chance someone at
the hearth tells a companion of a place their household knows and the companion's does not;
someone from another settlement always says where they are from), both design priors (research
13-01 §1.1 says awareness of other places comes by traders, migrants, kin and travellers, 09-16
§2.2 gives 0.05–0.25 for passing on routine news, and nothing gives a distance at which a village
is seen).

Content API 51 (M4c slice AI, step four) brings force: the `[crime]` keys `w_collect` (points
toward going to take what a refused finding owed, for one who keeps the watch, beside the norms
they hold), `w_harm` (points against taking it by force for each at the household who resisted),
`strike_threshold` (the range each person's reluctance to strike is drawn from, points),
`hurt_days` (the range of days a blow keeps the one struck from work) and `kill_share` (the chance
a blow kills), all design priors; and the tie act `struck`.

Content API 50 (M4c slice AI, step three) brings coups: the `[faction]` key `coup_cost` (what
calling on the others who keep the watch to take the deciding for it costs a watcher, points,
beside what the norms they hold weigh), a design prior; and the body membership `watch` (those who
keep the watch by a law in force), which a coup makes and an amendment template may name.

Content API 49 (M4c slice AI, step two) brings founding: the policy kind `repeal` (a law that ends
the one it names, carried rather than in force), the issue `founding` (present for
`founding_days` after the custom is taken from the gathering, while laws the old custom made are
unweighed), the core pack's `core:policy/repeal`, and the `[polity]` key `founding_days`, a design
prior (research 09-11 §1.6 gives founding processes of 3 to 36 months, none forced to finish).

Content API 44 (M4c slice AG) brings ideologies: the kind `ideology` (below) and the core pack's
`core:ideology/common_provision`, `order_kept` and `own_say`.

Content API 43 (M4c slice AG) brings values: the kind `value` (below), the core pack's
`core:value/security`, `autonomy` and `reciprocity`, and the policy templates' optional `[bears]`
(`common_store`, `against_taking`, `keep_watch` and `curfew` bear on them).

Content API 42 (M4c slice AG) brings norms: the kind `norm` (below) and the core pack's
`core:norm/gathering_binds`; the people profile's `[polity]` `comply_base` drops from 1.5 to 0, as
the norm now carries what it stood for.

Content API 41 (M4c slice AG) brings opinion: a policy template's optional `question`
(`core:policy/common_store`, `against_taking`, `keep_watch` and `curfew` ask one) and the people
profile's `[opinion]` table, design priors from research 06-04 §3.2.

Content API 40 (M4c slice AF) brings amending the custom: the policy kind `amend_body` with its
`members`, `quorum_shares` and `pass` (`core:policy/amend_custom`: adults, elders or landholders,
a quarter or a half, more for or two-thirds; tuning values), and the issue `overruled`.

Content API 39 (M4c slice AE) brings word of mouth and grievances: the people profile's `[word]`
table (`share_home`, `share_urgent`, `share_routine`, `news_days`, `max_grievances`,
`half_life_days`, `full_harm_days`, `reminder`, `tell_floor` and `remind_days`), tuning values
with the ranges research 09-16 and 04-06 give where they give one.

## Kinds (M4c)

### `norm`

A prescription people hold each their own way (ADR-0016 §4; research 06-05). Each person holds an
endorsement of it, drawn by a key and pulled toward their parents', a belief of how many
households abide by it, and a threshold on that belief; together they add points for doing what it
asks. What anyone believes of others is moved only by what companions tell of their own
households' acts, never by the settlement's true rate. Every number is a design prior (06-05
§2.2), not an estimate.

| Key | Meaning |
| --- | --- |
| `id`, `name`, `description` | As for every kind (`pack:norm/name`). |
| `statement` | What it says, in words the inspector shows after "That": "what the gathering decides binds everyone". |
| `does` | What the kernel does with it: `abide_by_laws` (its points weigh in paying a levy and keeping a curfew the gathering passed, and companions tell whether their household paid its last levy; a household that could not pay is not counted). |
| `[endorse]` `mean`, `sd`, `heritability` | A founder's endorsement in log-odds (−10 to 10) and its spread (0 to 10); a child takes on `heritability` (0 to 1) of its parents' mean, as the objection to taking is drawn. |
| `[expect]` `prior`, `learn_rate`, `share`, `tell_days` | What a founder believes of others before anyone has told them anything (0 to 1; someone new takes their household's); the share of the gap one account closes (0 to 1; 06-05 §2.2: 0.02–0.30); the chance a companion at the hearth tells of their household's last act, a session (0 to 1); and the days an act is worth telling (1 to 3,650). |
| `[threshold]` `low`, `high`, `width` | Each person's threshold on what they believe others do is drawn evenly from `low` to `high` (0 to 1; 06-05 §2.2: 0.2–0.9); `width` (0.001 to 1) smooths the step. |
| `[weights]` `endorse`, `expect` | Points for doing what it asks, for a full endorsement and for a belief well above one's threshold (0 to 100 each). |

The core pack's `core:norm/gathering_binds` is set so a founding village starts near the flat 1.5
points it replaced: an endorsement of about 0.6 on average, and most founders' thresholds below
the 0.85 they believe at first. Not built: the normative expectation (what others think one ought
to do), sanctions for breaking a norm, and norms that do anything but `abide_by_laws`.

### `value`

A slow axis on which people weigh what a law does beyond their household's own food (ADR-0016 §4;
research 06-04 §1.1: values such as security, autonomy and reciprocity). Each person holds each
value somewhere between -1 (less than most) and 1 (more than most), drawn by a key and pulled
toward their parents'; it does not change after (06-04 §3.2's twenty-year drift is not built).
Policy templates say in `[bears]` how a law of their kind bears on each.

| Key | Meaning |
| --- | --- |
| `id`, `name`, `description` | As for every kind (`pack:value/name`); `name` follows "cares for": "safety from want and harm". |
| `high`, `low` | Holding it more, and less, than most, in words the inspector shows: "holds safety from want and harm dear". |
| `mean`, `sd`, `heritability` | A founder's value is `tanh(mean + sd · z)` (mean -5 to 5, sd 0 to 5); a child takes on `heritability` (0 to 1) of its parents' mean. A mean of 0 tilts nobody one way. Design priors. |
| `weight` | Points a law that bears fully on it adds for one who holds it fully (0 to 100). A tuning value. |

The core pack names three, each with mean 0, spread 0.8, heritability 0.5 and weight 1. Not
built: values that move with experience, values seen by others (a sponsor counts on others by
their households' lots alone), and values weighing in anything but laws.

### `ideology`

A content record of a political idea (ADR-0016 §4; research 06-04 §6.1: problem explanation,
moral commitments, institutional proposals, legitimacy narrative). A holder weighs every law with
its commitments beside their own values, and, when the problem it explains is before the
village, weighs the laws of its program among their moves; a law they so propose keeps the creed
in its record. It travels only
with people.

| Key | Meaning |
| --- | --- |
| `id`, `name`, `description` | As for every kind (`pack:ideology/name`); `name` follows "holds to": "common provision". |
| `explains` | The problem it explains: an issue (`food_short`, `store_unkept`, `takings`, `overruled`). Only while it is before the village do holders weigh its program, and a law of the program is proposed as answering it. |
| `legitimacy` | Its legitimacy story, in words the inspector shows. |
| `program` | Policy template ids it proposes (each must exist; amendments of the custom are not weighed from a program yet). |
| `[commitments]` | Value id to -1 .. 1: how it tilts what its holders hold dear when they weigh a law (each value must exist). |
| `[spread]` `founders` | The share (0 to 1) of founders, and of newcomers with no parents recorded, who bring it. |
| `[spread]` `share`, `adopt` | The chance (0 to 1) a holder speaks of it to a companion at the hearth, a session; and the chance one who hears it takes it up when they trust the teller fully and hold dear just what it is committed to (scaled by trust, their regard for the teller, and by fit: none unless it fits what they hold dear at all, all of it for a perfect fit; 06-04 §1.4). |
| `[spread]` `inherit` | The chance (0 to 1) a child takes up what a parent holds. |
| `[spread]` `w_program` | Points (0 to 100) it adds, for a holder, to proposing a law of its program. |

The core pack names three, each brought by a tenth of the founders: `common_provision` (food
short; for safety and giving back; a common store and its keeper), `order_kept` (takings; a law
against taking and a watch) and `own_say` (overruled; for a household's say; no program). Not
built: giving an ideology up, new ones made by people (06-04 §4.2's recombination), organizations
that teach one, and the god tool that introduces one (slice AJ).

## M6b fire primitives (kernel content API 86)

The people profile's strict `[fire]` table supplies the physical and decision priors. It does
not supply fire events or outcomes. Core rates and durations are design priors unless the
content's comments cite a measurement; the planned ignition sweep remains separate.

| Fields | Meaning |
| --- | --- |
| `reach_m`, `flux_kw`, `covering_flux_share`, `wind_stretch_per_ms` | The footprint gap considered for heat, heat at 1 m and 4 m, the covering-stage share, and the wind's effect on effective gap. |
| `exposure_minutes`, `catch_per_minute`, `extinction_moisture`, `reference_moisture` | Accumulated heat, catching hazard and fuel moisture. |
| `involve_minutes`, `involved_minutes`, `covering_minutes` | Building-fire stages and covering consumption. |
| `char_mm_first_hour`, `char_exponent`, `infill_loss` | Loss of timber section and wall infill during burning. |
| `hearth_lit_per_hour`, `hearth_banked_per_hour`, `worked_per_hour`, `lamp_lit_per_hour` | Escapes per hour of the fires actually in use, scaled by dryness; `lamp_lit_per_hour` (content API 166, default 0, 0 to 0.1) is a lamp lit at home for work after dark, counted only as far as `worked_per_hour`. |
| `beside_m`, `grow_minutes`, `wake_minutes`, `sight_m`, `earshot_m` | Finding and controlling an escape, growth and waking intervals, sight and alarms. |
| `water_stays`, `wet_drains_per_minute` | Delivered water retained on roofs and its subsequent loss; a carrier must obtain and deliver each load. |
| `w_save`, `kin_share`, `neighbour_share` | The actor's stakes in saving a threatened roof. |
| `pull_crew`, `pull_h_per_m2`, `pull_cost` | Minimum simultaneous crew (2–32), shared hours per m² (0.001–100) and lost-worth choice cost (0–1). Core uses 2, 0.03 and 0.8, design priors. |
| `strip_m2_per_minute`, `strip_cost`, `salvage_share` | Covering removed per work-minute, the cost of stripping and the stake in carrying goods out. |

The fire-response activity behaviors are `douse`, `strip`, `salvage` and `pull`. Their ages, tools,
effort and session lengths use the ordinary activity fields. Dousing requires a known water
trip or a stored load; lack of that trip does not suppress stripping or salvage. Stripping
requires ownership or leave on site; salvage concerns the actor's own household's goods.
Pulling requires tools, an empty threatened building and ownership or an owner awake on site;
crew work is shared and loses all parts when finished. Its core activity uses axes, ages 14–65
and one-minute work intervals. Controlled crews are tested; spontaneous crew formation under
the default costs remains uncalibrated.
The good's optional strict `[fire]` table defines its burning share and catching threshold;
its absence does not make an authored material a universal fireproof building.

## Planned kinds

More techniques (each only with the work behind it), more building programs and grammars, style
primitives, offices beyond the storekeeper and further policies, service capability ladders, all
as the milestones in the plan introduce them (§5, §7).

Content API 140 (M8c slice CS, second part) brings the people profile's `[build] site_years`
(required, positive): the years over which a household about to build a home reckons a plot's
walking, its members' walks a year from there to where they go times these, beside the levelling
the plot needs once. The core sets it at 5.

Content API 141 (M8c slice CT, second part) brings the policy kind `plan` (ADR-0030 §2) with its
`[plan]` table: an orthogonal grid's block sides along and across (`block_m`), a parcel's
frontage and depth (`parcel_m`, the depth at most half a block's width), the streets' width
(`street_m`), the extents in blocks a sponsor chooses among, smallest first (`extents`, 1 to 8 of
1 to 16 blocks each way), whether the origin block is kept as a square (`square`), the hours
staking out a parcel takes (`stake_hours`), and the technique a sponsor must know (`needs`). It
brings the issues `no_ground`, `way_built_over` and `new_ground`, the core's
`core:technique/laying_out` (laying out by cord and stake, which one founder in ten brings) and
its template `core:policy/yours_to_plan_the_ground`. A technique a law's sponsor must know counts as gating work (E3005).

Content API 142 (M8c slice CU, second part) brings a plan's `allot`, how its keeper gives its
parcels: `need` (the default: a household about to build a home, the free parcel nearest the
hearth), `lot` (a free parcel drawn by lot), `offer` (the parcel nearest the hearth to a household
that pays the store the `[plan.fee]`'s `amount` of its `good`, once) or `standing` (the parcel
nearest the hearth to a household whose eldest the keeper regards). A fee is for `offer` only, and
`offer` needs one. The core's plan gives by need.

Content API 143 (M8c slice CU, third part) brings a plan's `[plan.rent]`, for `allot = "offer"`:
the food (a good with `kcal_per_kg`) and `amount` a household owes the store each year it holds a
parcel given by offer, on the day it was given, answered as a fine is. An offer names a fee, a
rent or both.

Content API 144 (M8c slice CU, fourth part) brings a plan's `phases` (1 to 8, 1 when unnamed) and
`open_at` (above 0 and at most 1, 0.75 when unnamed): its parcels open in phases outward from where
it is laid, each once that share of the parcels of those before it is given.

Content API 145 (M8c slice CU, fifth part) brings a plan's `fields`: `around` (the default: its
layout bends round fields), `taken` (laid over them, each field under its ground reserved without
consent once it is in force) or `paid` (laid over them, the store offering each field's holder the
`[plan.pay]`'s `per_ha` of its `good` a hectare, which the holder takes or refuses). A pay is for
`paid` only, and `paid` needs one.

Content API 146 (M8c slice CV, first part) lets a plan template name a sanction bundle in
`bundles`: at most one, without exile, its days within 0 to 365. A plan in force carries it as its
law's sanction, and a forum's `hears` may name `plan` to hear cases of building on reserved ground.

Content API 147 (M8c slice CV, second part) lets a plan's bundle take down what is found standing
on its reserved ground (`remove = true`) or let it stand for `yearly_days` (0 to 365) of the
household's food a year to the common store, not both. Only a plan's bundle has either.

Content API 148 (M8c slice CW, first part) appends two `building_rule` templates, each with
`from_m` (above 0 and at most 50): `fire_from_homes`, a new home whose household works fire (holds
a fixed tool a fuelled recipe needs) stands `from_m` from other households' homes; and
`midden_from_wells`, a new home, where its midden lies, stands `from_m` from every open well. The
core authors them as `works_wrought_worked_fires` (10 m, answering `burned`) and
`works_wrought_wells` (30 m, answering `sickness`).

Content API 149 (M8c slice CW, second part) brings a plan's `[[plan.districts]]`: each names a
`phase` of the plan's (1 to its `phases`, once) and the `uses` allowed on that phase's parcels, of
`dwelling`, `store` and `work`. A phase named in none allows every use.

Content API 150 (M8d slice CY, ninth part) moves mass walls' figures from code into each frame
program (ADR-0031 §1): `frame.mass_wall_cm`, a mass wall's own thickness range from frame version
3, and `[frame.masonry]`, for rubble, mud brick and fired brick the hours to lay a cubic metre,
its weight and its strength in compression, which the structure check reads.

Content API 151 (M8d slice CZ, second part) brings a building program's `[systems]`: for each of
frame version 3's systems, the technique that opens it, `trussed_roof`, `storeyed_posts` and
`wall_ties` (each a technique id, or omitted for none). A technique a program names there gates
work (E3005). The core adds `trussed_roof`, `storeyed_frame` and `tied_masonry`, none known to
founders, each tried toward in building huts; `jointed_frame` is now tried toward there too.

Content API 152 (M8d slice DB) brings two building-rule templates (ADR-0031 §5): `most_storeys`,
a dimension rule that a new building whose walls are of one of `walls` rise at most `storeys` (1 to
3); and `tied_walls`, a technique rule that new mass walls above a storey be tied at each floor,
which a policy gates by `needs`. The core authors them as `yours_to_stay_low` (rubble and mud brick
a storey high) and `yours_to_tie_the_walls` (needing `tied_masonry`), each answering `gave_way`.

Content API 153 (M8d slice DC, second part) brings the policy kind `raise_building`, whose
`[raise]` table names the building `program` a polity raises in common; it answers the issue
`store_spoils` (the common store spoiling unsheltered, as measured a year), and a store program
raised so shelters the common store once it stands. Work on it is the behaviour `raise`. The core
authors `yourselves_raise_a_storehouse` (a granary) and the activity
`work_with_wooden_tools_on_raising`.

Content API 154 (M8d slice DC, third part) brings the building use `hall` and the people
profile's `polity.wet_attend`, the points a gathering held in the open is worth the less on a wet
day. A gathering rained off for want of a quorum is measured, and the issue `rained_off` opens
while it is; a hall raised under `raise_building` answers it, and the polity's gatherings then
meet under its roof. The core authors the program `hall` and the template `yourselves_roof_a_hall`.

Content API 155 (M8d slice DC, third part) brings the building use `tower`. A fire nobody saw for
ten minutes after its covering caught, not caught from a neighbour already burning, is measured by
the work its building takes to put right, and the issue `fires_unseen` opens while it is and a
watch keeps watch; a tower raised under `raise_building` answers it. A watch round then begins at
the tower, and whoever stands at it sees a burning roof 500 m off. The core authors the program
`watch_tower` and the template `yourselves_watch_from_a_tower`.

Content API 156 (M8d slice DC, fourth part) brings four ties acts (`ties.acts` must list them):
`share_raised` (the other's household gave its share of the work on a building raised together),
`raised_well` (someone of the other's household worked on one that stands finished),
`raised_as_proposed` (one the other proposed was finished as proposed) and `proposal_stalled` (one
the other proposed stalled). Those who use a building raised together, every adult of its polity,
see them when it is finished or stalls.

Content API 157 (M8d slice DD) gives a taste a front: `style.tradition`, `tradition_spread` and
`personal_spread` may name `openings` (the share of a frame's walls left open), `ornament_h_per_m2`
(hours of craft a square metre of its front), `finish` (0 bare, 1 daubed, 2 lime-washed) and
`symmetry` (0 none to 1 a front set out about its middle), each 0 where not named. Only a frame
of version 3 has a front, a store no openings; a lime wash takes someone who knows lime burning.

Content API 158 (M10a slice DP) gives a fuel its heat: a good of purpose `fuel` must carry a
strict `[fuel]` table with `mj_per_kg` (1–60; firewood 15.6, charcoal 29.5, coal 25.8 by research
07-06 §2.3) and `smoke` (0–10, against firewood's 1). A household's hearth need stays in
kilograms of the first fuel in the catalogue (firewood); any other fuel burns by its heat, once
the fuels before it are gone. Which fuel a recipe takes is the recipe's, not the table's.

Content API 159 (M10a slice DP) gives a recipe `elapsed_days` (0–60, default 0): a batch that
runs unattended once started. A session of it takes the inputs and starts a batch; the outputs
come when someone of the household opens it on a later session, once its days are past; while
one of that recipe is under way the household does not start another. Charcoal is the first:
`good/charcoal` (a material, 29.5 MJ a kilogram), `technique/charcoal_burning` (found from firing),
`recipe/burn_charcoal` (5 kg of wood a kilogram, ten days) and `activity/burn_charcoal`.

Content API 160 (M10a slice DR) gives a fixed tool a `[power]` table and a recipe
`kwh_per_unit` (0–100, default 0). `[power]` names its `source` (`water`, a wheel at an
installation, or `fuel`, an engine), `rated_kw` (above 0, at most 1000) and `efficiency` (above 0,
at most 1; research 07-06 §2.2: simple wheels 20–40 %, overshot 50–70 %); only a fixed tool may
carry one. A recipe's `kwh_per_unit` is the power a unit takes from a powered station; a session
makes no more than the station's power over its hours allows. An installation's power is worked
out each day from its river reach's discharge, P = 9.81 Q H η, and is never saved.

Content API 161 (M10a slice DR, fourth part) adds the behavior `mill`: carry a load of the
household's grain to a working water mill of another household of its settlement, grind it there
by the way the mill's wheel turns, and carry the flour home, the mill's owner keeping a toll of it
(a sixteenth, a tuning value). Its activity names no recipe; the way is the mill's. The first is
`activity/carry_grain_to_mill`.

Content API 162 (M10a slice DS) adds three optional fields. A deposit rule's `keyed_ids` (default
false) gives its bodies ids of their own, numbered by the rule's place among the land profile's
rules, rather than the world's next ones, so adding the rule moves no other id in any world; a
fuel's `[fuel] technique` names the technique a household must know to keep that fuel for its
hearth (the hearth's measure stays the first fuel that asks none); and a technique's
`needs_known` lists goods its finder's settlement must know a deposit of, with some left. The
first of each is coal: `good/coal`, the coal rule of `land/temperate_valley`,
`activity/dig_coal` and `technique/coal_burning`.

Content API 163 (M10a slice DS, second part) lets a deposit rule give `k_m_day` (0–100, default
0), the hydraulic conductivity of the body and the ground it lies in: a pit dug below the water
table takes in water toward the table's head at the rate a wide well of its area would (research
03-02 §1.4), and what stands in it is lifted out by hand before digging below it goes on. A rule
giving none keeps its pits dry. The people profile's `[digging]` gains `bail_w` (a person's
sustained power, watts; default 75), `bail_efficiency` (the share of it that lifts water, above 0
and at most 1; default 0.5) and `influence_m` (a pit's radius of influence, metres; default 50).
The core's coal rule gives 0.3 m/day; its other bodies none.

M10a slice DS, third part, adds no field: a fixed tool whose `[power]` source is `fuel` is an
engine, and one set at a pit pumps its water each day while its owner holds it whole and fuel,
burning the owner's fuels in the catalogue's order by their heat at the engine's efficiency. The
first is `good/atmospheric_engine` (4 kW at 1 %, 40,000 hours). Its fourth part adds no field
either: a household wants an engine only for a pit it has chosen to set one at, weighing the hours
it bails there against the engine and its fuel; `technique/atmospheric_engine`,
`recipe/build_atmospheric_engine` and `activity/build_atmospheric_engine` are the first way of
making one.

Content API 164 (M10a slice DT) lets a tool's `[tool]` table name its `crew` (1-20, default 1):
the fewest hands at it together for it to run. Only a fixed tool asks more than one; a session
of a recipe that needs it makes nothing without its crew, and a household works such a recipe
only at a works. The people profile's `[firm]` gains `shift_h` (1-16, default 10): the hours of a
works' shift from its bell at sunrise, cut at sunset. The first machine is `good/tilt_hammer`
(crew 2), with `technique/water_hammer`, `recipe/build_tilt_hammer` and
`recipe/forge_bar_by_hammer` and their activities.

M10a slice DT's fourth part adds no field: coke and the blast furnace are content made of what is
already there. `good/coke` (28.2 MJ a kilogram, a furnace's material as charcoal is) is baked from
coal in a four-day batch by `recipe/coke_coal` under `technique/coking` (charcoal burning and
burning coal; found burning charcoal with coal at home). `good/blast_furnace` (crew 4, 40,000
hours) is built by `recipe/build_blast_furnace` under `technique/blast_furnace` (the bloomery and
the water hammer; found smelting iron); blown by a works' wheel it smelts `good/pig_iron` from
ore on charcoal (`recipe/smelt_pig_iron`) or, under `technique/coke_smelting`, on coke
(`recipe/smelt_pig_iron_with_coke`), and the hammer's two fine the pig to bar
(`recipe/fine_pig_iron`). Each has its activity; founders bring none of the three techniques.

Content API 165 (M10b slice DU) lets a tool carry a `[light]` table: `fuel`, the good it burns (by
content id), and `kg_per_hour`, how much of it an hour lit (above 0 and at most 1). Only a tool
gives light. The first lamp is `good/fat_lamp`, a clay dish burning `good/tallow` (8 g an hour),
with `technique/lamps` (found drying meat, with meat at home; founders do not bring it),
`recipe/render_tallow`, `recipe/make_fat_lamp` and their activities. Nobody wants a lamp for
its own sake (`per_worker = 0`); what light changes comes in DU's later parts.

Content API 166 (DU's second part) adds `[fire] lamp_lit_per_hour` to a people profile: the
escapes an hour of each person working at home by a lamp after dark (core 2.0e-6, as a lit
hearth). A household holding a whole lamp and fuel for it may make at home after dark, for no
longer than its fuel lasts; work away from home stays to daylight.

Content API 167 (DU's third part) adds `[crime] dark_sight_m` and `lit_m`. Once a lamp has been
lit for work anywhere in the world, a taking after sunset is seen only within `dark_sight_m`
(core 10 m) unless it is within `lit_m` (core 3 m) of a home where a lamp burns for work. Until
then the dark shortens nobody's sight.

Content API 168 (M10b slice DV) appends the vehicle mode `rail`: a wagon on track. A wagon names
`rail_resistance` (its rolling resistance on its track) and `gauge_m` (the track it runs on, metres)
beside a cart's `tare_kg`, `pull_n`, `power_w` and `brake_grade`, and refuses a boat's fields and a
cart's road resistances; a cart and a boat refuse a wagon's. The core adds `core:vehicle/wagon` (2 t
aboard a 0.5 t wagon at 0.005 on timber rails of a metre's gauge, pushed at 200 N and 75 W, held
back down 1 %) and `core:technique/wooden_railway`, which needs wheels on an axle and is tried in
digging coal and ore; design priors marked in their files.

Content API 169 (DV's second part) adds a land profile's `[track]`: what laying track takes. It
names the `good` its sleepers and rails are made of and `good_kg_m` of it a metre, `lay_h_m` (hours
to hew and lay a metre), `earth_h_m3` (hours to dig or place a cubic metre in grading), `shoulder_m`
(the bed's width beyond the gauge each side) and `search_grade` (the steepest ground between cells a
line is laid over before its bed is graded). The core lays timber at 50 kg and 1.5 hours a metre,
grades at 2.3 hours a cubic metre, on a bed 0.5 m wider than the gauge each side, over ground no
steeper than 5 %. A profile that leaves it out lays no track.

From DV's third part the core's `[track] search_grade` is 0.4: a line is cut across ground up to 40
% between cells, its grading's earth weighed against going round, and goes round only what is
steeper. The field's meaning is unchanged; content API 169 still.

Content API 170 (M10b slice DW) lets a rail vehicle name an `[engine]`: the `fuel` it burns (a good
with `[fuel]`), the `efficiency` with which it turns that fuel's heat into work (0.001-0.5) and its
`top_speed_m_s` (0.1-50). Its `pull_n` and `power_w` are then the engine's: the most its driven
wheels hold before they slip, and its power. Only a rail vehicle may have one; a rail vehicle
without one is a wagon pushed by people. The core adds `core:vehicle/locomotive` (5 t, 7.5 kW,
9,810 N, 2.2 m/s, burning coal at 3 %) and `core:technique/locomotive`, which needs the atmospheric
engine, wooden railways and forging and is tried in digging coal; design priors marked in their
files.

Content API 171 (M10b slice DX, third part) brings the policy kind `build_line` and the issue
`loads_carried`. The issue opens when two or more of a settlement's households dig one pit and a
line laid together from it to beside the hearth would spare them, over a wagon's life, more than
its laying and timber, once someone of the settlement knows how to make a wagon. Under
`build_line` the polity marks the line out from the pit to the hearth, and each household is asked
an equal share of laying it. The core adds `core:policy/yourselves_work_a_line_to_the_pit`.

Content API 172 (M10b slice DY) lets a fixed tool carry a `[holder]` table: `good`, the good it
keeps apart (by content id), and `kg`, how many kilograms of it it holds full (above 0 and at most
100,000); and a `[light]` may be `piped` (default false), its fuel coming by a main rather than
from its household's stores, so it is no lamp a household fills. The core adds `good/coal_gas` (40
MJ a kilogram, a material, not a hearth's fuel), `good/gas_retort` and `good/gas_holder` (fixed;
the holder keeps 150 kg of coal gas), `good/gas_burner` (piped, 0.071 kg an hour), the technique
`technique/gas_making` (needs coking and lamps, found coking coal with coal at home; founders do
not bring it), `recipe/make_gas` (a kilogram of coal into 0.14 kg of gas and 0.65 kg of coke at the
retort), `recipe/build_gas_retort`, `recipe/build_gas_holder`, `recipe/make_gas_burner` and their
activities. Nobody wants any of them for its own sake (`per_worker = 0`); what gas does comes in
DY's later parts.

Content API 173 (M10b slice DY, third part) adds a land profile's `[gas_main]`: what laying a gas
main takes. `good` is the good its pipe is made of, `good_kg_m` how many kilograms a metre takes,
`lay_h_m` the hours to dig, lay, joint and fill in a metre, and `capacity_kg_h` the most gas a main
of its bore carries an hour (each above 0). A profile that leaves it out lays no main. The core
lays cast-iron mains of 25 kg and 3 hours a metre, carrying 6 kg an hour.

Content API 174 (M10b slice DY, fourth part) adds `life_years` to `[gas_main]`: the years a main
laid serves (above 0 and at most 1,000), over which a gasworks reckons whether the homes along a
main it might lay would repay it. The core's serves 50 years.

Content API 175 (M10b slice DY, fourth part) adds the policy kind `light_streets` (an office, its
holder the lamplighter) and the issue `dark_takings` it answers, and a people profile's
`crime.lit_guard`: the share of takings after dark a household believes those about would see by
lit streets, what it weighs such a law by (0 to 1; the core's 0.3). The core adds
`policy/yourselves_work_the_street_lamps`.

Content API 176 (M10b slice DY, fourth part) adds the behaviour `light` (the lamplighter's round of
the street lamps, after dark, a stand of the activity's least minutes at each) and a people
profile's `crime.w_light`, the points the round is worth to the lamplighter (0 to 100; the core's
6, as a watch's round). The core adds `activity/light_lamps`.

Content API 181 (M10c slice EB, third part) adds the policy kind `keep_a_school` (an office, its
holder the schoolteacher) and the issue `untaught` it answers: a household of the polity could not
hold the fee to send a child to learn a technique it lacked within the year, and someone grown of
the settlement knows it. The schoolteacher takes the polity's children to learn what they know
without a fee, and where a law pays common work the store pays their household for the hours they
teach them. The core adds `policy/yourselves_work_with_a_teacher`.

Content API 182 (M10c slice EC, first part) adds the behaviours `write` (writing down at home a
technique the writer alone of their settlement knows and no record there keeps) and `read`
(reading a record the household holds of a technique the reader does not know), with their
points `decision.w_write` and `decision.w_read` (at least 0; the core's 3), a people profile's
`knowledge.record_h` (the hours of writing a record takes, above 0 and at most 1,000; the core's
20), and a technique's `codified` (how many times its `learn_h` learning it from a record alone
takes: 0, the default, for heard of there but never learnt, or 1 to 100). A technique that another
technique gating work requires now gates work itself (E3005). The core adds the techniques
`reckoning`, `reading` and `writing`, the activities `write_a_record` and `read_a_record`, and
makes `laying_out` codified at 2.

Content API 183 (M10c slice EC, third part) adds the policy kind `licence` and the issue
`unknowing_care` it answers (someone of a household of the polity died within the year in the care
of a carer it paid who knew no treatment of their disease, and someone grown of the settlement
knows one). A `licence` template names in `[licence] technique` what those paid to tend the sick
must know. The core adds `policy/yourselves_work_with_carers_who_know`, asking fluid replacement.

Content API 184 (M10c slice ED, first part) adds the disease route `company` (in company at the
hearth with someone shedding it) and a disease's `company_hazard`, the hazard an hour in company
with one shedding it puts on each met (0 to 10; default 0). The core adds `disease/smallpox`,
passing at home and in company, which no world has until the observer brings it.

Content API 185 (M10c slice ED, second part) adds a disease's `[inoculation]`: the `technique`
that gives it on purpose (by content id), `deaths` (the share of a natural course's severe share an
inoculated course has, 0 to 1) and `believed_take` (the chance a household believes its people
would take it where it is near, 0 to 1; it calls an inoculator only where `deaths` is below it).
The technique gates work (E3005). The core gives smallpox variolation at 0.127 (Jurin's 2.1/16.6)
and 0.5, and adds `technique/variolation`.

Content API 186 (M10c slice ED, third part) adds a disease's `[vaccination]`: the `technique` that
gives it and the `good` it is given from (by content id), `dose_kg` (a person's dose), `efficacy`
(the share of the disease's hazards it turns away while whole, 0 to 1), `full_years` (least and
most years it stays whole, drawn evenly) and `waning_years` (the years over which what it turns
away then falls by a factor of e). A household makes the good only for calls to vaccinate that
stand at its door. The core gives smallpox `technique/vaccination` (requiring variolation) with
`good/vaccine_matter` at 0.001 kg, 0.95, [3, 5] and 10, and adds the recipe and activity
`make_vaccine_matter`.

Content API 187 (M10c slice EE, first part) adds `does = "keep_apart"` and its `[keep_apart]`
table: `days_after` (the days after the last of a household is well that it is still kept apart,
1 to 365) and `cost_share` (the share of a day's work each grown member loses while their home is
kept apart, as a household weighs it, 0 to 1), and the issue `contagion` (someone of a household
of the polity died within the year of a sickness passing at home or in company). The core adds
`policy/yourselves_work_without_meeting_the_sick` at 14 days and 0.25.

Content API 188 (M10c slice EE, second part) adds an office template's `holder_needs` (a
technique, by content id, its holder must know: the nominee and those a lot draws among), and
`does = "keep_register"` with its `[register]` table: `avoided_share` (the share of a household's
deaths of disease within the year it believes a register would have spared, 0 to 1). The core adds
`policy/yourselves_write_down_the_sick`, answering `sickness`, asking `technique/writing`, at 0.1.

Content API 189 (M10c slice EE, second part) adds the behaviour `rounds`: the round of the one a
law names to keep the register of the sick, in daylight, once a day, a stand of the activity's
`min_minutes` at each home of the watch's set round. The core adds `activity/walk_the_rounds` at 10
minutes.

Content API 190 (M10c slice EE, third part) adds `does = "clear_middens"` and its `[middens]`
table: `distance_m` (how far from every open well and stream a household near one carries its heap,
1 to 1000), `keep_days` (how long a heap carried lies there, 1 to 365), `carry_h` (the hours a
carrying takes, as a household weighs it) and `avoided_share` (the share of its deaths of disease
within the year it believes cleaner water would have spared, 0 to 1); and the behaviour `midden`.
The core adds `policy/yourselves_yonder_with_middens` (30 m, 2 days, 0.5 h, 0.2), answering
`source_suspected`, and `activity/carry_out_the_midden` (20 minutes).

Content API 191 (M10c slice EE, fourth part) adds the building use `sickhouse` and the issue
`untended` (severe cases of the polity's households lay ill with nobody tending them, measured, and
no sickhouse raised). The core adds `building/sickhouse` and `policy/yourselves_zone_a_sickhouse`,
a raising together answering it.

Content API 201 (M11a, first part; ADR-0036 §2) adds the `[power]` source `shaft`: a fixed tool
turned by the shaft of a wheel or an engine set beside it, delivering `efficiency` of that shaft's
work up to `rated_kw`. The first is `good/dynamo` (10 kW at 80 %, design priors), made by
`recipe/build_dynamo` under `technique/dynamo`, which no work tries toward, so only the observer can
bring it. Nothing reads a dynamo yet.

Content API 202 (M11a, third part; ADR-0036 §3) adds the land profile's optional `[wire]`: what
laying a line of current takes and how it carries. `good` (must exist) and `good_kg_m` are the
conductor a metre of line takes out and back, `lay_h_m` the hours of work to set poles and string a
metre, `rating_kw` the most a line passes, `voltage_v` the world's one direct-current voltage,
`ohm_per_m` a metre's resistance out and back (losses are I²R at that voltage), and `life_years`
how long a line serves; each above 0 and in range. The core's: copper, 0.62 kg and half an hour a
metre, 6 kW, 110 V, 0.001 ohms, forty years (design priors). A profile without it lays no line.

Content API 207 (M11e, second part; ADR-0037 §3) brings frame version 4's steel skeleton. A frame
program's `[systems]` may name `skeleton`, the technique that opens it; its `[materials]` may name
`frame`, a fifth slot holding the good a skeleton's posts, plates, tie and cross beams and joists
are made of; and its `[frame]` may give `frame_kg_per_m3`, what a cubic metre of those members
weighs over their envelope (zero or more, 0 by default). A program naming `skeleton` needs both. A
skeleton member is checked by its good's `[timber]` strengths, spread over the envelope of the
round member it replaces. The core adds `technique/skeleton_frame` (requiring `rolling` and
`jointed_frame`, tried toward in building huts, a far technique nobody starts knowing), gives
`good/steel_section` a `[timber]` table (an I-section of 235 MPa and 210 GPa smeared over its
envelope: 68 MPa in bending, 33.5 in compression, 61 GPa, no creep), and offers the skeleton in the
workshop and the hall at 1119 kg/m³ (design priors).

Content API 208 (M11e, third part; ADR-0037 §5) adds `[timber]`'s optional `burnt_keeps`: the share
of its strength a bare member of the good keeps once its building has burnt out. Such members do not
char; when the fire burns out each keeps that share of what it carried, and the daily check decides
whether it still stands. The core gives `good/steel_section` 0.4 (research 11-14 §1.6, §2.1: about
0.6 of yield at 550 °C, 0.4 at 600 °C). Protection in minutes and the `ProtectedFrame` rule are
deferred.

Content API 209 (M11f, first part; ADR-0038 §2) adds `[fuel]`'s optional `ash_share` (0–0.5,
default 0): kilograms of ash a kilogram of the good leaves burned. What a household's hearth burns
puts its ash on the household's heap, an inert share that never decays, dungs no field and carries
nothing to the water. What a recipe takes of a fuel, by unit or by session, leaves its ash in the
maker's store as `good/ash`, less the ash the fuels it gives keep (coke keeps its coal's, charcoal
its wood's). The core gives firewood 0.005 and coal 0.11 (research 12-10 §2.1), charcoal 0.03 and
coke 0.10 (design priors), and adds the residues `good/ash`, `good/slag`, `good/coal_tar` and
`good/ammoniacal_liquor`, all materials that keep. `recipe/make_gas` gives 0.05 kg of tar and
0.13 kg of liquor a kilogram of coal besides its gas and coke (12-10 §5.7); the bloomery leaves
6 kg of slag a kilogram of bloom, the blast furnace 0.6 a kilogram of pig on charcoal or coke, the
finery 0.35 a kilogram of bar and the converter 0.15 a kilogram of steel (design priors; 12-10 §5.7
for the furnace). Nobody wants a residue unless a recipe takes one.
