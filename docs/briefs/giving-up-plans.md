# Brief: where a household that gives up goes

Status: **proposed, revised for the owner's second review** (2026-11-25, after the owner's review
of that day, sections 3, 4 and the migration parts of 5, and corrected for the owner's second
message that day, points 2 to 4: the move charged, staying eligible while it passes the survival
screen, exits known only by knowledge; first version 2026-11-24). The owner approved the design
*direction* only; nothing here is approved for building. Plan §8 question 24,
item 1, building out the M5 settlements brief's aim ("Founding is one plan beside staying, joining
a known settlement and leaving the map") for households that give up.

The owner's direction (2026-11-24): households that give up should weigh viable alternatives:
relocation to an established settlement, founding where realistically feasible, and leaving the map
when no viable internal alternative exists, each arising from the household's circumstances, the
land, what it knows, what it can reach and carry, and its prospects of surviving, not from fixed
probabilities or a fixed order of outcomes.

## 1. What happens now (checked against the code, 771a064d)

**The trigger.** Each midnight (`life.rs` `live_day` calls `departures`), a household whose food is
below a day's need, whose members' mean depletion is at least `leave_at_depletion`, and with no crop
of its own ripe within `leave_unless_ripe_within_days` or reaped, draws against `leave_chance`
(`life.rs`): `leave_per_day` times the logistic of `leave_w_gap·gap − leave_w_stake·stake −
leave_stay`, where `gap = 1 − reach/wait` sets its own food, its share of what its settlement's
households could spare (`village_food`) and of a common store whose law a member knows, against
the days to its next harvest. When the draw falls under the chance, the household **is committed to
leave**: no outcome of what follows keeps it at home.

**Where it goes** is a fixed order (`moving.rs` `refuge`, then the `stranded` loop in
`departures`): (1) the known lived-in settlement with the highest pull `w_kin·kin +
w_ties·sat(known) + w_fed·(food seen − fed share at home)`, if above zero, a place last seen no
better fed than home excluded and one with no food impression counted as **no different from
home** (`k.food.map_or(0.0, ..)`), so with no kin or ties its pull is exactly zero and it is
refused; (2) failing that, asking again once others giving up the same night have moved (chain
migration: they are now kin or ties living there, via `draws`); (3) failing that, off the map
(`life.rs` `leave`).

**What a move is.** `relocate` (`relocate_seeing`, gave-up form) is instantaneous for the
household: a new household record at the destination's hearth holds the old one's whole store at
once, membership and residence change that minute, its old ground is handed over, and its made
boats and carts go too when `boating.rs` `vehicles_move` finds a way for them (then also placed at
once). Members' bodies are not moved: each keeps its activity and reaches the new home by its next
ordinary walks, routed on the walking grid (in a household-resolved settlement, `resolution.rs`
`park` places them at home). Nothing checks before the move that such a walk exists.

**What leaving the map is.** `leave` removes the household's people that minute, closes their
records (`ResidenceWhy::LeftMap`), books its goods as `Flow::Departed` and hands over its land.
Nobody walks anywhere, and there is no off-map value, exit or connection: it is the default.

**Distance.** `moving.rs` `walk_hours` is the straight line between hearths divided by
`NavParams::top_speed_ms` (Tobler's top speed, 6 km/h in the core content, the fastest any step is
walked). Its doc comment says "at the pace people walk off trails", and the first version of this
brief repeated that; the code uses the top speed, so an off-trail walk on the flat (5 km/h times
`offtrail_factor` 0.6) reads as half as long as it is, before any slope or detour. It ignores
water and floods. Founding's site screen (`founding.rs` `founding_sites`) uses the same line.

**The baseline.** Departure diagnostic run 112 (world 2 alone, 9cf0512): all nine households that
left Thornbank knew Sedgestead was lived in, by word or sight only, with no food impression, kin or
anyone known there; its pull was 0.00, so they left the map. Homelab run 124 (the same diagnostic
on verification 38ff89f): in the 16th year (days 5985–6011) 12 households gave up, all in
Thornbank, and all 12 left the map. Three knew no other place. Nine had learned of Sedgestead by
sight or word on day 5991, with no food impression, kin or acquaintance there, while the fed share
at home was 0.04–0.44. Thornbank kept 3 or 4 people afterwards. (The first version went on to
argue from Sedgestead's later food stocks that it "had room" for them. That inference is withdrawn:
later stocks say nothing about what it could have spared newcomers then, or whether the walk could
be made.) These runs are earlier evidence, not re-run for this revision.

## 2. What the research asks

- **Consider, compare feasible plans, prepare, travel; "stay" is a plan** (05-06 §5.1), valued by
  the change in expected consumption, safety and support less switching costs, normalized; travel
  spending belongs in the budget. **Emergencies screen for survival first** (§5.1).
- **Willingness differs from ability** (05-06 §1.2): deprivation raises the wish to leave and
  lowers the means; dependents constrain departure. Starvation can make travel impossible, and
  escape is not survival (05-06 §1.4; 05-02 §1.5).
- **Famine migration follows expected access to food**, in stages through nearby refuge (05-06
  §1.4). **Networks help but are not needed** (05-06 §1.2).
- **No world truth** (05-06 §5.1; ADR-0018 §4): remembered or reported conditions only.
- **Journeys, not straight lines** (09-16 §1.2: routes, departures, capacity, weather; 07-08 §1.4:
  journey speed is not underway speed; §1.8: migrants carry people, dependents and possessions).
  A persistent journey in which consumption and illness go on (05-06 §5.3), and destination
  feedback rather than caps.
- **Founding is a project** that can fail and return (05-06 §1.3, §5.2; 10-01 §1.5).
- **Eras as capabilities** (05-06 §3; 07-08 §3; 09-16 §1.2: every service has an access policy,
  schedule, capacity, price and reliability).

## 3. The design in slices

Slice 1 is emergency reconsidering with routes and a charged one-day move (§3.1–3.5); slice 2 a
persistent journey for walks longer than a day; slice 3 founding by those who give up; slice 4
ordinary migration with transport-aware off-map plans (§3.6–3.7).

### 3.1 Slice 1: the hazard triggers reconsidering

The hazard and its inputs stay as they are, but a draw under `leave_chance` now means **the
household reconsiders tonight**, not that it leaves. Staying is one of the plans and stays
competitive whenever it passes the survival screen (§3.3, step 3), so it can win both when its
prospects are better and when they are a little worse but home holds the household. A household
that stays remains under the hazard, so it reconsiders at most at the hazard's own rate (at most
`leave_per_day`, 0.1, a day); there is no second daily roll (05-06 §5.1). A household whose only
feasible plan is staying **stays trapped**: nothing about staying feeds it, and it may starve.
Because the hazard was calibrated as a rate of leaving and is now a rate of reconsidering, fewer
leave per firing. That is reported as a behaviour change; the hazard is not retuned to make up for
it.

### 3.2 Slice 1: the plans and their physical screen

Plans: **stay**; **go to each known, lived-in settlement** (from the household's `Places`);
**leave by an exit the household knows** (§3.5). For every plan but staying, a physical screen
first; a plan that fails any test is not a plan tonight, and the reason is recorded:

- **A route exists today.** One bounded Dijkstra search from home (`nav.rs`
  `NavGrid::travel_field`) on today's grid, with its crossings, enclosures, fords at today's flow
  and today's flood depths (deeper than 0.5 m impassable, `with_decks_and_flood`; the owner's
  2026-11-20 call keeps it daily), gives the walking time to every known hearth and known exit
  within a walking day; one A* search (`route_with`, within the existing cell budget) each for any
  known place or exit beyond it. Lakes, sea and unfordable rivers bar a route; a detour is a longer
  time. A search that runs out of budget is counted as *reachability unknown*, treated below as a
  plan beyond the bound. This replaces `walk_hours` for this purpose only; ordinary moving keeps it
  until slice 4.
- **It can carry what it must**: members who cannot walk it (under `walking_age_years`, or too
  depleted or sick) by body mass (`needs::mass_kg`), and the walk's food and water, against
  `carry_kg` per able member not pulling a vehicle, plus the payload of vehicles a way would bring
  (`vehicles_move`'s test), each needing an able member to pull it.
- **It has the provisions**: food it holds and can carry covers every member's need for the whole
  walk at its burn (walkers and carriers at the move's effort, those carried at rest); water is
  carried or the route passes river or lake cells each day.
- **Its walkers can make it**: no walker's depletion (`life.rs` `depleted`) passes
  `collapse_depletion` by the walk's end; the acutely sick count as carried.

A plan that passes is **feasible** whatever its length. Whether slice 1 can carry it out is a
separate mark: *within the bound* when the routed walk at the slowest walker's pace fits the
daylight left from first light (at most `walk_hours_day`), *beyond the bound* otherwise (§3.4).
The mark never removes a plan from the comparison. Staying is always feasible. Viability (§3.3)
is another question again: a feasible plan may not be believed to get the household through.

### 3.3 Slice 1: beliefs, viability and choice

**Beliefs about food.** A new helper, `food_belief(place, day)`, gives the household's belief about
the fed share at a known place as a mean and an uncertainty, from an explicit prior and the dated
observation it holds: a prior of mean `prior_fed` and weight `prior_weight` (in observations;
deliberately weak); the observation `o = KnownPlace.food` weighted `w =
0.5^(age/fed_half_life_days)`, times `told_share` if it came by word; mean `μ =
(prior_weight·prior_fed + w·o)/(prior_weight + w)`, and the uncertainty of a Beta of that weight.

So a place known only by word is neither zero food nor as good as home: it is the prior, with wide
uncertainty; an old impression drifts back to the prior; a fresh one dominates. The helper reads
only what the household holds, never the destination's stores, numbers or later stocks.

What exists and what does not: `KnownPlace` keeps the last food impression a member *saw* (a visit,
`Places::went`, or leaving, `saw_on_leaving`) but no date of that impression (`visited` is the date
of the last visit, and leaving sets none), and impressions never travel by word today (a place told
of has `food: None`). Slice 1 therefore adds the impression's date and how it came (seen or told),
and lets a teller at the hearth pass their household's impression with its original date when
they tell of the place (the existing `share_told` telling). That is a save change (§5).

**Food access is not the fed share.** A newcomer eats from what it carries and from what others
there will spare it; residents who are fed may have nothing to spare (`spare_food_kcal`: only food
above twice the days a household keeps), and no crop of its own comes before its first harvest
there. Expected coverage of its need over a horizon `H` (the days to the first harvest it could
bring in there, as `founding.rs` `lacking` reckons sowing and growing, capped at a year):

`c_j = min(1, s_j + (1 − s_j)·μ_j·g_j)`, with `s_j` = (food it can carry − the walk's food) / `(N·H)`,

where `N` is the members' daily need and `g_j`, a newcomer's access to others' spare, is
`stranger_access` with no kin or ties, rising toward 1 with them (`draws`; chain migration enters
here). For staying, `c_stay = min(1, reach/wait)`, the quantity the hazard used (`1 − gap`), read
from home as the household sees it now.

**Viability** (defined apart from feasibility): a plan is *believed viable* when its coverage at a
pessimistic reading of the belief (`μ` at the `viable_quantile` of the Beta, as founding forecasts
yield at a low share of belief, 10-01 §2.3) reaches `viable_coverage`. Staying is judged the same
way on `c_stay`. Viability decides one thing only: whether exits may be weighed (step 2).

**Score.** Every term is in shares of the horizon's need, so the scores are normalized and **zero is
staying** by construction:

`V_j = (E[c_j] − c_stay) + w_support·(support_j − support_home) + w_safety·(safety_j − safety_home)
− w_stake·stake`, with `V_stay = 0`.

**The walk's food is charged once, in coverage.** `s_j` counts only the food left after the walk,
so the walk already lowers `E[c_j]`; the first version also subtracted `walk_food/(N·H)` from the
score, charging it twice, and that term is removed. Coverage keeps it because that is the walk's
real effect on a household in danger: food eaten on the road is food the horizon no longer has. A
household whose coverage is capped at 1 then pays nothing in the score for the walk's food, which
is right for survival; the walk's hours are not scored in slice 1 (a day's work lost is small
beside the horizon, and lived in fact, §3.4). Support is `sat(kin + known)` as `draws` gives it,
safety the raids believed (`raids_believed`), and the stake what its fields here should bring (as
the hazard reckons it). Moving's point weights are not reused: they are in different units.

**Eligibility, then choice.**

1. *Physical screen* (§3.2): the feasible set, staying always in it, plans within and beyond the
   bound alike.
2. *Off-map rule*: exits join the set only if no internal plan (staying, or a feasible
   settlement within or beyond the bound, or one whose reachability is unknown) is believed viable.
   A reachable destination the slice cannot yet walk to still exists, and still blocks off-map.
3. *Survival screen*, on that set: it drops every plan whose pessimistic coverage is more than
   `survival_margin` below the best one's. Staying is treated exactly as any other plan: it stays
   eligible whenever it passes, believed viable or not, and is dropped only when it is
   survival-dominated. The best plan always passes, so the set is never empty.
4. *Choice*: among the eligible, `P_j = exp(V_j/T)/Σ exp(V_k/T)` with `T = plan_temperature`, a
   fixed value in those same units, drawn with a keyed draw (household, day, a new purpose;
   ADR-0011 §3).
5. *Execution*: a plan within the bound is carried out (§3.4, §3.5). A plan drawn beyond the
   bound is not: the household stays tonight, still under the hazard, counted as *held by the
   bound* with the plan it drew, until slice 2 exists.

Eligibility is a rule applied before the draw, so no randomness can select an impossible or
survival-dominated plan, and off-map cannot be drawn while any internal plan is believed viable.
Chain migration stays: households reconsider in id order, and one whose draw took it off the map is
held until the night's other choices are made, then weighed once more, as the `stranded` loop does
now, since kin who just moved may make a place viable.

**`decide::choose` is not reused, and stays unchanged.** It samples only options with a positive
total (all of them when none is), at a temperature of `temperature_sd_fraction` (0.35) times the
standard deviation of the sampled totals, floored at `min_temperature` (0.4). Here that is wrong
three ways: staying, at exactly zero, would be excluded whenever any move scores above it; when
nothing scores above zero, every plan, the worst included, would be sampled; and a spread-based
temperature makes a 0.01 gap between two plans as decisive as a 1.0 gap. A sibling in `decide.rs`
takes the eligible set and a fixed temperature, sharing only the softmax arithmetic.

**Ordinary moving is untouched.** `move_worth` keeps treating no impression as no difference from
home, and `refuge` is replaced only on the giving-up path. Were `move_worth` to use
`food_belief`, households at a hungry home would be drawn toward places known only by word
(`w_fed` is 4 points against a move `cost` of 2), and old good impressions would weigh less: a
large change to ordinary moves. So the helper is separate and used for giving up only, until slice
4 measures that change on its own.

### 3.4 Slice 1: the move, charged and lived within a day

A route check moves nobody and spends nothing, so slice 1 charges and lives the move itself, within
one day. Recommended over a persistent journey for slice 1 (§7, choice 1), because the walk ends
before midnight and so needs no new saved state, and because on today's maps nearly every walk fits:
a map is at most 2,048 cells of 8 m (16.4 km a side, `civ-sim` `MAP_SIZES`, `CELL_SIZE_M`), and
the M5a demo's settlements stood 3.2–5.4 km apart. What is charged, and how:

- **The time.** The decision is midnight's, as now (ADR-0032 §4). At their first decision after it,
  every member takes one activity, *moving house* (new content: a walk step along the route, then
  a deposit), and walks the routed path at the slowest walker's pace, setting off at first light.
  It is an ordinary activity and trip (`person.rs` `Activity`, `Trip`, saved as now); the bound
  makes it end before dark, so no walk crosses midnight. The day's work is not done; that is the
  cost of the hours.
- **Dependents and carriers.** Those who cannot walk are carried on the same trip and timings as
  their carriers. Carriers burn at the activity's effort (`par`, a design prior above walking's
  `walk_par`, as a loaded walk is), the carried at rest, through the existing energy accounting, so
  depletion rises as the walk makes it rise.
- **Provisions.** The walk's food is not deducted at midnight: the walkers' bodies spend it as they
  walk (their deficit), and they eat it from the store at the new hearth, so it is spent exactly
  once. Water likewise.
- **Possessions.** What it carries is chosen by its worth in hours of the household's work per
  kilogram (`value.rs`), after the dependents, the walk's food and water, and food for the horizon,
  within the capacity of §3.2. That goes with it. **What exceeds capacity is left**: in slice 1
  lost to the household and booked as a new flow, *abandoned* (flows are counters, never saved), so
  the goods ledger shows it; the alternative, leaving it at the old home in the household's name to
  be fetched, needs a holder and a save change (§7, choice 2). Vehicles go only with a member to
  pull them.
- **What is still not lived.** The record, residence and carried goods are at the new hearth from
  the midnight of the move, some hours before the carriers arrive (the goods are not a consignment
  on the road). A walk cut short (a crossing gives way, water rises) re-routes as any walk does, and
  failing a route its walkers wait and decide again, as now. In a household-resolved settlement the
  move is a block of the household day, as a fetch is (ADR-0032 §2, §4).

Consequences, stated: households leave tools, seed and stores behind when they cannot carry them (as
distressed migrants abandon or sell bulky property, 07-08 §1.8; 05-06 §1.4), so they arrive poorer
than today's relocation leaves them; a household of small children and few adults may be unable to
go at all; the moving day is lost to work; and destinations beyond a day's light, though they count
in the comparison, cannot be reached until slice 2, the household staying instead (§3.3, step 5).

### 3.5 Slice 1: exits, and leaving the map

ADR-0018 §3's meaning stays: off the map is a departure account, not a destination, a place or a
stock; nobody comes back from it, and nothing draws from it.

- **Exits are actual connections a household knows.** The navigation grid can find every edge; that
  is not knowledge. A household knows an exit only through what it holds: an edge its members
  walked; the edge or way it, or kin, came in by; word of a route told at a hearth; and, in slice 4,
  a road, line or posted run it knows leaves the map. Today only the first can be read from saved
  state, roughly: `Household.known`'s dated `KnownPatch` records of worked ground, where a patch
  reaches the map's edge. The rest needs a saved known-exits record per household (the edge cell or
  way, how known: walked, came in by, told; first and last day), told on at the hearth as places are
  (`share_told`), with arrivals (waves, sent families) recording the edge they came by: a save
  change (§5; §7, choice 3). Each known exit is screened as any plan is (§3.2) on today's grid: an
  edge across an unfordable river or a flooded valley is not an exit tonight, and the nearest edge
  in a straight line has no standing. Founders placed at setup came in by no recorded way, so their
  descendants know an edge only by walking or by word.
- **What lies beyond is uncertain**: an outside belief with mean `outside_fed` and a low weight
  `outside_weight` (wider than a lived-in place's), no kin or ties, stranger access. It is never
  presumed promising: in the emergency it is weighed only when no internal plan is believed viable
  (§3.3, step 2).
- **Leaving** is lived as a move is (§3.4): the household walks to the exit carrying what it can,
  leaves the rest, and is booked off the map (`leave`) when its last member reaches the edge. An
  exit beyond the bound is held by the bound, as a place is.
- **No known exit**: with no viable internal plan and no known feasible exit, a household chooses
  among staying and its feasible moves that pass the survival screen; with no feasible move, it
  stays, trapped. Run 124's three households that knew no other place would, unless one of them knew
  an edge, stay at Thornbank under this design and may starve there; that is reported as a behaviour
  change, not tuned away. Exploring for an unknown way out is not a plan in slice 1.

### 3.6 Slices 2 and 3

**Slice 2, a persistent journey for walks longer than a day (when slice 1's count of plans held by
the bound shows it matters, or slice 4 needs it).** What exists: saved trips with routes and timings
(`person.rs` `Trip`); walk, paddle, haul and ride steps; posted rides on trains and road runs
(`trains.rs`, `transit.rs`); goods in transit (`consignments.rs` `Consignment`, ADR-0029 §3, tied to
a boat); and the war band, with saved, dated stages (`war.rs` `BandStage`: out, camped by a shut
line, holding a place where its people sleep away), supply carried within `carry_kg` and movement at
the slowest member's pace on the walking grid (ADR-0023 §3). What does not: a household on the road
with a dated record, legs ending at a night's camp, or a remaining-time clock across days. A minimal
journey would be a saved record on the band's pattern (origin, destination or exit, route, stage and
dates, goods as a consignment on foot), day legs walked as trips, nights in camp, residence
unchanged until arrival (ADR-0018 §2, §3: people in transit stay counted), off-map booked at the
exit. Not proposed for building now.

**Slice 3, founding by those who give up.** Through the existing coalition machinery
(`founding.rs` `founding_plan`, `gather`, `lacking`, `found_settlement`), as a plan in §3.3's
comparison. Preconditions, each a screen: a site the household's members walked
(`founding_sites`, with its route found as in §3.2, not the straight line); **seed reserved apart
from food** (`lacking` already subtracts the seed for the needed ground before counting food);
**seasonal timing** (`lacking` already asks whether the ground can be broken before sowing ends);
**tools**, **water** within reach of the site, and **shelter** (none of which `founding_sites` or
`lacking` checks today); **labour** for breaking ground and building; and **credible support to
the first dependable food**: food held to the first harvest and `buffer_months` beyond, or support
believed from kin at a parent within reach. Failure and return: today coalitions have only three
fates (`places.rs` `CoalitionFate`: gathering, founded, dissolved) where ADR-0018 §5 names six
stages; slice 3 adds a failed fate and lets the founders' households reconsider, which may take
them home. A household that gives up for hunger will seldom pass these screens; that should emerge,
not be ruled.

### 3.7 Slice 4: ordinary migration and transport-aware off-map plans

Separate from the emergency. The yearly or event review (`review_residence`) gains an off-map plan
that may compete **even when staying or an internal move is viable**, reflecting actual
connections (exits on foot, by water for a household with a boat, by ways or lines reaching the
edge, with posted runs, fares, in food today, schedules and reliability where they exist: ADR-0034
§5, ADR-0036 §7), the household's means (fares and provisions it can pay and carry, dependents,
health), access constraints (who may ride, what a line carries) and information (word, letters or
the telegraph where they exist, ADR-0035, ADR-0036 §8, sharpening or widening the outside belief).

Only connections the household knows of count (§3.5). Differences between eras come only from those
capabilities and that knowledge, never a "modern" bonus: where no known way leads out, a household
may weigh exploring for one, at the cost of days, food and an uncertain outcome; a known line or
posted run with a timetable and fare, or word by letter or telegraph, cuts that uncertainty and
cost. A ferry, canal or posted road can make leaving practical in an old world, and a modern
household without fare, health or a known connection stays trapped. No exit connection to the
outside exists in content or worldgen today; how one comes to exist without scripting a plot (a
setup choice like ADR-0018 §6's, or derived from ways people build to the edge) is an owner decision
for that slice, as is whether ordinary moving adopts `food_belief`.

## 4. Implementation dependencies (slice 1)

| Piece | Where (verified) | Use |
|---|---|---|
| Trigger | `life.rs` `departures`, `leave_chance` | unchanged |
| Known places, food impression | `places.rs` `Places`, `KnownPlace` | beliefs (gains a date and how) |
| Kin, ties by place | `moving.rs` `draws` | support, access |
| Raids believed | `moving.rs` `raids_believed` | safety |
| Walking grid, today | `nav.rs` `NavGrid::with_decks_and_flood`, `travel_field`; `civ-sim` `refresh_nav` | route screen |
| Carrying, vehicles | `HouseholdParams::carry_kg`, `boating.rs` `vehicles_move` | capacity |
| What to carry first | `value.rs` (worth in hours of the household's work) | possessions |
| The walk itself | `person.rs` `Activity`, `Step::Walk`, `Trip` | the lived move |
| Worked ground | `person.rs` `KnownPatch` (`Household.known`) | walked edges |
| Body and walking | `life.rs` `depleted`, `needs::mass_kg`, `EnergyParams::walk_par` | health, dependents, burn |
| Moving, leaving | `moving.rs` `relocate`; `life.rs` `leave` | carrying out the choice |
| Softmax | `decide.rs` (new sibling of `choose`) | the draw |

New: `food_belief`; the screen; the known-exit set; the plan function replacing `refuge`'s order
on this path; the *moving house* activity (content) and the first-decision rule that starts it
(read from the residence history's entry of last midnight, saved already); the *abandoned* flow;
content parameters, each a design prior logged as a NUDGE with the scene figures that set it:
`prior_fed`, `prior_weight`, `fed_half_life_days`, `told_share`, `stranger_access`,
`viable_quantile`, `viable_coverage`, `survival_margin`, `plan_temperature`, `walk_hours_day`,
`walking_age_years`, `collapse_depletion`, `outside_fed`, `outside_weight`, `w_support`, `w_safety`,
`w_stake`, and the moving activity's `par` for carriers and the carried (a content API bump,
documented in `content/README.md`). The route search runs only for
households whose hazard fired, one travel field each bounded to a walking day, on the wear
factors the route caches use (`population.rs` `sync_routes`); its cost is to be measured, not
assumed.

## 5. Save-format and ADR changes, for approval before any build

Slice 1:

- **Save, places:** known places gain the food impression's day and how it came (seen or told):
  a section version bump with an explicit migration (old impressions dated by `visited`, else
  `last`, as seen).
- **Save, known exits** (if choice 3 takes the record): a per-household list of exits known (edge
  cell or way, how: walked, came in by, told; first and last day), appended to the places section
  or a section of its own, absent while empty; a version bump, old saves loading with none.
- **Not saved:** plans (worked out at midnight, not kept); the screen's routes; the move itself,
  which is an ordinary activity and trip in the people section as now, started from the residence
  history and ended before midnight. The *abandoned* flow is a counter. A goods holder at the old
  home is needed only if choice 2 takes it.
- **Wire:** none required. A chronicle word for a household that reconsidered and stayed, or
  could not leave, is optional and not proposed for slice 1.
- **ADR:** an amendment to ADR-0018 §3 (exits as known, screened connections; off-map eligible in
  the emergency only when no internal plan, within or beyond the bound, is believed viable) and §5
  (a household that gives up reconsiders among screened plans, staying among them; the charged
  one-day move and what it leaves behind).

Slice 2 (if built): a new journeys section in `tce_save.fbs` (`tools/gen-schema.sh`, flatc
25.12.19), one record a household on the road: the household, origin settlement, destination
(settlement or exit), route waypoints, the leg reached, stage (on the road, camped, arrived,
turned back, stranded) with its dates, and tonight's camp point; and the consignments section's
boat made optional, so its goods travel as a consignment carried on foot. Members' activities and
trips stay as saved now. ADR-0018's "people stay away overnight" revisit, ADR-0029 §3, ADR-0032 §1
(Detailed held, or a journey block in the household day, while a household is on the road) and
ADR-0011 §2 (legs across midnight): probably one ADR.

Slice 3: append-only codes for a failed fate and for what a coalition lacked (tools, water,
season); an ADR-0018 §5 amendment. Slice 4: an ADR on exit connections and outside expectations
(ADR-0018's "the map gains an outside world" revisit), with no outside stock.

## 6. Acceptance checklist

Scenes in a new `kernel/crates/civ-sim/tests/giving_up.rs`, in the style of `settlements.rs`;
each fails without its rule and ends asserting population and goods accounting
(`residence_problems`, the settlements' accounts, the goods ledger with `Flow::Departed`). A drawn
choice is checked with a fixed keyed draw or by probabilities over many draws, never one
destination under every draw. Each item is tied to its commit and evidence when built.

| Requirement | Scene, run or metric | Slice |
|---|---|---|
| Word-only knowledge is the prior, not zero | `a_village_known_only_by_word_is_weighed_at_the_prior_not_refused` (the run 124 case) | 1 |
| Stale knowledge fades to the prior | `an_old_good_impression_counts_less_than_a_fresh_one` | 1 |
| Staying competes and can win | `a_household_whose_staying_looks_better_mostly_stays` (probability check) | 1 |
| Staying not believed viable stays eligible | `staying_not_believed_viable_is_still_drawn_while_it_passes_the_survival_screen`; and dropped when survival-dominated | 1 |
| Immobility is not survival | `a_household_that_cannot_carry_its_young_stays_and_goes_hungry` | 1 |
| Impassable routes | `a_village_beyond_an_unfordable_river_is_not_a_plan` | 1 |
| Current flood restrictions | `a_ford_flooded_today_closes_the_plan_and_opens_when_the_water_falls` | 1 |
| Detours | `a_detour_makes_the_nearer_village_the_longer_walk` | 1 |
| Exits | `off_the_map_only_when_no_known_place_is_believed_viable` | 1 |
| A place beyond the bound blocks off-map | `a_viable_village_beyond_a_day_keeps_the_household_from_the_edge_and_it_stays` (counted as held by the bound) | 1 |
| No exit | `with_no_known_exit_and_no_viable_place_a_household_stays` | 1 |
| Exits only by knowledge | `an_edge_nobody_walked_or_heard_of_is_not_an_exit`; `an_exit_told_at_the_hearth_becomes_known` | 1 |
| Nearest edge not presumed | `an_edge_across_water_is_not_an_exit` | 1 |
| Insufficient provisions | `too_little_food_for_the_walk_rules_the_plan_out` | 1 |
| The move is charged | `a_moving_household_walks_the_day_carrying_its_young_and_arrives_depleted`; `what_it_cannot_carry_is_left_and_booked` (goods ledger) | 1 |
| Walk food counted once | unit test: a plan's score falls with the walk's food only through coverage | 1 |
| Leaving is lived | `a_household_leaving_walks_to_the_exit_and_is_booked_there` | 1 |
| Eligibility before randomness | unit tests in `civ-agents`: non-viable and survival-dominated plans never drawn over 10,000 draws; drawn shares match `P_j` | 1 |
| Kin and chain migration | the existing refuge and chain-migration tests in `settlements.rs`, re-read for changed behaviour, not kept as trajectories | 1 |
| Detailed saves at ordinary hours | `a_world_saved_while_a_household_walks_to_its_refuge_goes_on_alike` (mid-move), and one mid-walk to an exit | 1 |
| Saves at the event | `a_world_saved_at_the_midnight_a_household_reconsidered_goes_on_alike` | 1 |
| Accelerated midnight saves, mode changes | Gate A's Detailed/Accelerated/Detailed and 30×1 = 1×30 over a giving-up scene; a household-resolved switch at that midnight | 1 |
| Mid-journey saves | `a_world_saved_on_the_second_day_of_a_journey_goes_on_alike` | 2 |
| Failed founding | `a_hungry_coalition_without_seed_does_not_go`; `founders_whose_first_harvest_fails_may_go_home` | 3 |
| Transport-aware off-map | scenes per connection kind (foot, boat, posted run), and a household without the fare trapped | 4 |
| Lived worlds | the departure diagnostic (run 112's set-up) and a long run (smoke, Gate B, the notables' gate, the dashboard's Moves, Population and accounting rows) | 1 |

Metrics reported, never targeted: per reconsidering, the plans weighed, each screen's verdict and
reason, viability, the probabilities and the choice; households trapped; plans held by the bound;
goods abandoned by kind;
internal moves against off-map departures; what became of arrivals a year on (alive, still there,
moved on, dead); distance decay and corridors from the moves recorded (05-06 §4). For an
intentional change the result is the changed behaviour, reported, with consistent continuation
from the same saved state; old trajectories are not demanded.

**No tuning toward an outcome.** Not saving Thornbank, not fewer departures, not keeping any
destination from dominating. The priors are set from scenes, logged as NUDGE entries, and left
there whatever the lived worlds show; a lived-world result that looks wrong goes back to the owner,
not into the numbers.

## 7. Remaining choices for the owner

1. **Slice 1's move: the charged one-day move (§3.4) or a persistent journey now (§3.6).** I
   recommend the one-day move: it charges time, dependents, depletion and possessions with existing
   activities and trips and no new saved state, and on maps of at most 16.4 km it covers nearly
   every walk. The cost: carried goods reach the new store some hours before their carriers, and a
   viable place beyond a day's light holds the household at home (counted) until slice 2.
2. **What exceeds carrying capacity:** lost to the household and booked as abandoned (recommended
   for slice 1: no save change, and it does not overstate what a distressed household brings), or
   left at the old home in its name to be fetched later, more faithful within a day's walk but
   needing a goods holder at a home, a save change, and fetch trips.
3. **Exit knowledge in slice 1:** a saved known-exits record (walked, came in by, told; recommended,
   since word of routes is part of the owner's request), or only edges derived from worked ground
   (`KnownPatch`), with no save change but more households trapped. Either way no household knows an
   edge merely because a route to it exists.
4. **Word-of-food reports in slice 1** (recommended: reports are part of the owner's request), or
   sight only, leaving the prior to carry word-only places (run 124's nine households held no
   impression of Sedgestead) and avoiding the telling change.
5. **The prior's centre:** a content value (recommended), or the mean of the household's other
   impressions, which lowers expectations in a regional famine but correlates the beliefs.
6. **Households that split** (sending some members, leaving dependents with kin; 05-06 §1.2):
   recommended out of these slices; the cost is that care constraints only ever block a move.

Settled by the owner (2026-11-25) and written in: staying stays eligible whenever it passes the
survival screen (§3.3, step 3); exits are known only through what households know (§3.5).

## 8. Not approved by this review

- Implementation of any slice; new or changed parameters; the ADR amendments and save changes of
  §5; performance-limit amendments; promotion.
- Separately deferred, untouched by this brief: the older price-convergence persistence proposal,
  the notables under-proposal question, sowing-wage timing, and the other ADR and sewer decisions.
- The open validation blockers named in the review (combined determinism candidate 74e6519d's
  failures in chain 192, benchmark 131, long run 132) are outside this brief.
