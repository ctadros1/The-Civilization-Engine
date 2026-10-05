# ADR-0012: Weather and the soil

Status: Accepted
Date: 2026-10-05
Milestone: M3c

## Context

How weather and soil work today:

- **Weather is one number a year.** Each year a climate factor is drawn from
  `[seed, climate, year]` as an AR(1) with correlation 0.3 and spread 0.25. It scales every
  harvest and the wild plants. There is no landscape in its key, so both landscapes of a seed
  live through the same years in every run, and their lean years line up with every village
  failure recorded so far (§9, 2026-10-05).
- **Storms are a stand-in.** They are one log-normal roof load per settlement and month.
- **Wear and fuel use follow fixed tables.**
- **Fields keep no soil.** A harvest is 900 kg/ha × the ground's richness × the year's factor ×
  sowing, weeding and standing losses. Planning assumes an average year, and nothing is ever
  rested on purpose.

M3c brings weather and seasons, soils and fertility, and "land that remembers how it was used".
Its demo is a dry year and a wet year in one village.

Reports read: 03-03 (climate and weather), 03-04 (soils and fertility), 03-02 (hydrology), 08-02
(agrarian production), 07-04 (agricultural technology), 11-12 (earthworks) and 14-09 (seasons on
screen). Their main points:

- **One coherent regional series.** Dice per farm destroy the shared shocks that make a bad year
  bad (03-03 §1.5).
- **Daily rain:** occurrence follows a first-order Markov chain, and wet-day amounts a gamma
  distribution. Temperature is an autocorrelated anomaly around monthly normals. A slow monthly
  anomaly carries droughts across seasons without drifting the mean. Persistence lies within
  0.1–0.5 and gamma shapes within 0.5–2 (03-03 §1.3, §1.6, §2.2). Temperature falls about
  6.5 °C per km of height (§2.1).
- **Climate does not advance with technology** (§3.1).
- **Agents and institutions see weather as it happens, never future draws** (§5.2). Both speeds
  must see the same weather (§5.4).
- **Crop water use:** crop coefficients by stage (03-02 §2.1), and yield answers relative
  evapotranspiration through a response factor (08-02 §7.2). The year-to-year spread of yields
  lies within 0.20–0.35 (08-02 §7.3).
- **A nutrient budget:**
  - Organic matter in fast and slow pools, decaying at years-to-decades rates (03-04 §1.2,
    §2.1, §2.5).
  - Grain and straw carry nitrogen away (§2.2).
  - Manure releases little in its first year (§2.3).
  - Fallow recovers over 10–15 years (§2.5).
  - Water and nutrients limit yield together, not twice over (§5.2).
  - Farmers judge the soil by what it gave them (§5.4).
- **Terraces and drainage help only where their constraint binds** (03-04 §2.8). Ditches are
  lines with gradients and upkeep (11-12 §5.1).

## Decision

### 1. Weather is a daily series per world

- One series serves the whole map; temperature is adjusted for each place's elevation.
- Each day is drawn from `[seed, weather, landscape, day]`.
  - Every speed sees the same weather (ADR-0011 §3).
  - Each landscape of a seed has its own history.
- The generator:
  - wet days by month, from a two-state chain with persistence;
  - gamma rain on wet days, scaled to the landscape's annual total and its monthly shares;
  - a daily temperature anomaly around monthly normals;
  - a slow monthly anomaly that shifts rain and summer warmth;
  - snow that falls when the day's mean is near freezing, lies by elevation band and melts by
    degree-days.
- Parameters live in the land profile's `[weather]`, each marked as a research or tuning value.
- Saved: what tomorrow depends on (yesterday wet, the anomalies, the snow) and each month's
  record.

### 2. A field's water sets its harvest

- From sowing, each field keeps a root-zone water balance. Reference evapotranspiration comes
  from temperature (Hargreaves); the crop's need follows coefficients by stage.
- Its harvest scales by `1 − Ky × (1 − actual ÷ needed)`, divided by that factor's long-run
  mean. So an average year still gives the content's yield, and variance is not counted twice.
  - The long-run mean is computed when a world is made or loaded. It is derived, never saved.
- The yearly climate factor and its parameters are retired.

### 3. The soil remembers

- Each field keeps two pools of organic nitrogen, a fast and a slow one.
- They turn once a year on a fixed day, with no random draws:
  - decay releases nitrogen;
  - grain and straw carried home take it away;
  - roots and stubble return some;
  - middens and manure add it, mostly to the pools rather than at once.
- A harvest is the least of what water allows and what the soil's nitrogen supplies.
- A field left unsown recovers toward its native stock.
- Each field keeps the record of its last harvests, with what limited each.
- Parameters live in the land profile's `[soil]`.

### 4. Who sees what

- People plan from what they can see: the season so far, and a field's own past harvests. The
  hidden stocks and future draws are never used.
- Households decide which fields to crop or rest, when to break ground (outside the sowing
  season too), and whether to manure, as other work is decided (ADR-0006, ADR-0008).

### 5. What the weather drives first

- Harvests (§2).
- Wild plants, by the month's soil water.
- Workable days: rain, snow on the ground or frost stop field work.
- Wear: the month's rain against its normal wears coverings, infill and posts.
- Roof loads: the month's storm day from the weather, with snow on roofs from the snowpack.
  These replace the monthly peak-load stand-in, and its tuned failure band is checked again.

### 6. Saves and boundary

(Renumbered on 2026-10-05: slice S took saves schema 23 and wire 1.23, plan §9.)

- **Saves schema 24** holds the weather's state and records, each field's water, soil pools
  and record of harvests, and resting fields.
- **Older saves** load as follows:
  - the weather starts with the old year's deviate as its slow anomaly, soils at field
    capacity and no snow;
  - each field's pools are rebuilt by replaying its harvests from native ground;
  - fields growing at the time are unstressed so far.
- **Wire 1.24** appends:
  - the day's weather and the season on the clock and snapshot;
  - each field's water and soil in words;
  - a weather query for a panel of months and years.
- The **content API** gains `[weather]` and `[soil]`.

### 7. Not in M3c

- **Ditches and terraces move to M6,** beside the levees:
  - their value needs soil water and erosion, which M3c begins but does not finish;
  - the presets' rain exceeds emmer's need;
  - a ditch is a line with a gradient and upkeep, unlike the rectangles of ADR-0010.
- ADR-0010's "Revisit when" item on M3c's ditches moves with them.

## Consequences

- Each landscape has its own weather. Lean years still come, but their frequency and length now
  follow the climate the research gives, not a single draw per seed.
- Fields that are cropped every year decline, rested fields recover, and a field's record says
  why each harvest was what it was.
- Villages may fail in other years, or more often: never tune the weather to save them. Answers
  belong in what people do with what they can see (§4).
- Forbidden:
  - weather keyed by speed or by call;
  - nitrogen created from nothing;
  - planning from hidden stocks or future draws;
  - a flat bonus for terraces or ditches.

## Alternatives considered

- **Keep the yearly factor and key it by landscape.** That is the cheapest fix, but one number
  cannot carry workable days, storms, snow, wild food or a dry May.
- **Weather per cell or per field.** The maps are 6–33 km across; one regional series with an
  elevation lapse fits 03-03 §1.5. Spatial fields come with regions (M5).
- **A fuller soil:** phosphorus, potassium, erosion and salt. These are deferred until crops,
  livestock and irrigation give them consequences.

## Revisit when

- Livestock arrives: dung, folding and fodder.
- A legume or a second crop arrives: rotations.
- M6's water: rivers by season, floods, wells, ditches and terraces.
- M5's regions: spatial weather.
