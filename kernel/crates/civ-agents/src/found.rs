//! A founding band arrives (plan §2 core loop; research 05-01 §4.5, 10-01 §1.1).
//!
//! The band is at least [`crate::params::BandParams::min_families`] unrelated families whose ages,
//! couples and children come from the life table, so the first generation is not a crowd of
//! newly paired adults (05-01 §5.4). They choose where to camp by scoring sampled sites on the
//! wild food around them, the walk to fresh water, slope and flood risk, and sampling one with a
//! softmax; the site is theirs to choose, not the engine's.

use civ_core::time::DAYS_PER_YEAR;
use civ_core::{PermanentId, Rng64, SimTime};
use civ_world::nav::TravelField;
use civ_world::{WATER_LAKE, WATER_LAND, WATER_RIVER, terrain};

use crate::demography;
use crate::history::{ChronicleKind, Origin, PersonRecord, Union};
use crate::influence::{
    InfluenceKind, WAVE_DAYS, WAVE_HOUSEHOLDS, WAVE_MONTHS, WAVE_SIBLINGS, Wave,
};
use crate::needs::Sex;
use crate::params::{GoodUse, PeopleParams, step_at};
use crate::person::{Activity, Household, Load, Person, Repro, Target, Traits};
use crate::places::{PlaceHow, distance_to_walk};
use crate::population::{Ctx, Population, cell_centre, cell_of};
use crate::{decide, farm};

/// Purpose tag for founding draws.
pub const PURPOSE_BAND: u64 = 0x6261_6e64_3030_3031; // "band0001"
/// Purpose tag for the draws that make a family the observer sends (god tool).
pub const PURPOSE_SPAWN: u64 = 0x7370_6177_6e30_3031; // "spawn001"
/// Keyed-randomness purpose of what a founder knows (ADR-0008 §1): its own stream, so the other
/// founding draws are as they were.
pub const PURPOSE_KNOW: u64 = 0x6b6e_6f77_3030_3031; // "know0001"
/// Keys the draw of a find at the end of a session (ADR-0008 §3).
pub const PURPOSE_FIND: u64 = 0x6669_6e64_3030_3031; // "find0001"
/// Keyed-randomness purpose of the skills a founder brings: its own stream, so a skill added to
/// the content changes none of the other founding draws.
pub const PURPOSE_SKILLS: u64 = 0x736b_696c_6c73_3031; // "skills01"
/// A family the observer sends within this of a settlement's hearth, metres, joins it; farther
/// away it makes camp where it was placed.
pub const SPAWN_JOIN_M: f32 = 600.0;
/// Longest walk from the hearth to a home, seconds.
pub(crate) const HOME_REACH_SECONDS: f32 = 300.0;
/// Families in a band of the default size: a larger band's homes spread out from the hearth by
/// the square root of how many more families it has.
const CAMP_FAMILIES: f64 = 8.0;

/// Farthest a home moves from where its family wanted it, cells.
const HOME_SEARCH_CELLS: i64 = 8;

/// Where a family makes its home: the wanted point if it is on dry ground reachable from the
/// hearth, otherwise the centre of the nearest such cell within a few cells.
pub(crate) fn home_site(
    map: &civ_world::WorldMap,
    near: &TravelField,
    wanted: (f32, f32),
) -> Option<(f32, f32)> {
    let ok = |c: usize| map.water[c] == WATER_LAND && near.seconds_to(c).is_some();
    let at = cell_of(map, wanted);
    if ok(at) {
        return Some(wanted);
    }
    let (w, h) = (i64::from(map.width), i64::from(map.height));
    let (x, y) = (at as i64 % w, at as i64 / w);
    let mut best: Option<(i64, usize)> = None;
    for dy in -HOME_SEARCH_CELLS..=HOME_SEARCH_CELLS {
        for dx in -HOME_SEARCH_CELLS..=HOME_SEARCH_CELLS {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            let c = (ny * w + nx) as usize;
            let d2 = dx * dx + dy * dy;
            if ok(c) && best.is_none_or(|(b, bc)| d2 < b || (d2 == b && c < bc)) {
                best = Some((d2, c));
            }
        }
    }
    best.map(|(_, c)| cell_centre(map, c))
}

/// What a founding produced.
#[derive(Clone, Debug, PartialEq)]
pub struct Founded {
    /// The settlement.
    pub settlement: PermanentId,
    /// Its name.
    pub name: String,
    /// Its hearth, metres.
    pub hearth: (f32, f32),
    /// The people, oldest first per family.
    pub people: Vec<PermanentId>,
    /// The households.
    pub households: Vec<PermanentId>,
}

pub(crate) struct Draws(pub(crate) Rng64);

impl Draws {
    fn unit(&mut self) -> f64 {
        self.0.next_f64()
    }

    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }

    fn normal(&mut self) -> f64 {
        let u1 = 1.0 - self.unit();
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }

    fn pick<'a>(&mut self, list: &'a [String]) -> Option<&'a String> {
        if list.is_empty() {
            None
        } else {
            list.get((self.0.next_u64() % list.len() as u64) as usize)
        }
    }
}

/// One planned member of a founding family.
#[derive(Clone, Copy, Debug)]
struct Member {
    sex: Sex,
    age: f64,
    /// Index of mother and father within the family, if they are in it.
    mother: Option<usize>,
    father: Option<usize>,
}

fn plan_family(params: &PeopleParams, d: &mut Draws) -> Vec<Member> {
    let band = &params.band;
    let mother_age = d.range(18.0, 40.0);
    let father_age = (mother_age + 3.0 + 4.0 * d.normal()).clamp(17.0, 60.0);
    let mut family = vec![
        Member {
            sex: Sex::Female,
            age: mother_age,
            mother: None,
            father: None,
        },
        Member {
            sex: Sex::Male,
            age: father_age,
            mother: None,
            father: None,
        },
    ];
    let spacing = band.birth_spacing_months / 12.0;
    let mut t = 19.0 + 1.5 * d.normal();
    while t < mother_age - 0.75 {
        let age = mother_age - t;
        if d.unit() < params.mortality.siler.survival(age) {
            family.push(Member {
                sex: if d.unit() < 0.5 {
                    Sex::Female
                } else {
                    Sex::Male
                },
                age,
                mother: Some(0),
                father: Some(1),
            });
        }
        t += spacing * d.range(0.7, 1.3);
    }
    // The father is at least 16 years older than the eldest child.
    let eldest = family[2..].iter().map(|m| m.age).fold(0.0f64, f64::max);
    family[1].age = family[1].age.max(eldest + 16.0);
    if d.unit() < band.elder_chance {
        let elder = Member {
            sex: if d.unit() < 0.5 {
                Sex::Female
            } else {
                Sex::Male
            },
            age: d.range(52.0, 68.0),
            mother: None,
            father: None,
        };
        let i = family.len();
        family.push(elder);
        // The elder is a parent of one of the couple.
        let child = if d.unit() < 0.5 { 0 } else { 1 };
        if family[i].age - family[child].age >= 16.0 {
            if elder.sex == Sex::Female {
                family[child].mother = Some(i);
            } else {
                family[child].father = Some(i);
            }
        }
    }
    if d.unit() < band.young_adult_chance {
        family.push(Member {
            sex: if d.unit() < 0.5 {
                Sex::Female
            } else {
                Sex::Male
            },
            age: d.range(15.0, 22.0),
            mother: None,
            father: None,
        });
    }
    family
}

/// The couple at the head of a founding family: how long they have been partners, and where the
/// mother is in the reproductive cycle (research 05-01 §5.4: a mixture of pregnant, nursing and
/// other states, never a crowd of newly paired adults who all conceive at once).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Couple {
    since: SimTime,
    /// Nursing her youngest child (its index in the family) until `until`.
    nursing: Option<usize>,
    until: Option<SimTime>,
    /// Pregnant since, until, and whether it ends in a loss.
    pregnant: Option<(SimTime, SimTime, bool)>,
}

impl Couple {
    /// The mother's state; `ids` are the family's permanent ids (the father is the second).
    fn repro(&self, ids: &[PermanentId]) -> Repro {
        match (self.pregnant, self.until) {
            (Some((conceived, due, loss)), _) => Repro::Pregnant {
                conceived,
                due,
                father: ids.get(1).copied(),
                loss,
            },
            (None, Some(until)) => Repro::Recovering { until },
            (None, None) => Repro::Open,
        }
    }
}

fn founding_couple(
    params: &PeopleParams,
    family: &[Member],
    now: SimTime,
    d: &mut Draws,
) -> Couple {
    let f = &params.fertility;
    let year = civ_core::time::MINUTES_PER_YEAR as f64;
    let ago = |years: f64| now.plus_minutes(-(years * year) as i64);
    let mother_age = family[0].age;
    let youngest = (2..family.len())
        .filter(|&i| family[i].mother == Some(0))
        .min_by(|&a, &b| family[a].age.total_cmp(&family[b].age));
    let eldest = (2..family.len())
        .filter(|&i| family[i].mother == Some(0))
        .map(|i| family[i].age)
        .fold(0.0f64, f64::max);
    let together = if eldest > 0.0 {
        (eldest + 1.0).min(mother_age - 15.0)
    } else {
        d.range(0.2, (mother_age - 16.0).max(0.3))
    };
    let mut couple = Couple {
        since: ago(together.max(0.1)),
        nursing: None,
        until: None,
        pregnant: None,
    };
    let recovery_years = demography::recovery_days(f, &mut d.0) / 365.0;
    let open_since = match youngest {
        Some(i) if family[i].age < recovery_years => {
            couple.nursing = Some(i);
            couple.until = Some(ago(family[i].age - recovery_years));
            return couple;
        }
        Some(i) => family[i].age - recovery_years,
        None => together,
    };
    // Open for `open_since` years: she is pregnant about as often as an open woman's cycle of
    // waiting and gestation leaves her so (nine months against a mean wait of 1/q).
    let q = f.conception_per_month * step_at(&f.age_factor, mother_age);
    if q <= 0.0 || d.unit() >= 9.0 * q / (1.0 + 9.0 * q) {
        return couple;
    }
    let gestation_years = f.pregnancy_days / 365.0;
    let along = d.unit() * gestation_years.min(open_since.max(0.0));
    let conceived = ago(along);
    let (due, loss) = demography::pregnancy_course(f, mother_age - along, conceived, &mut d.0);
    if due > now {
        couple.pregnant = Some((conceived, due, loss));
    }
    couple
}

/// Removes member `i` of a family, keeping the other members' parent links right.
fn remove_member(family: &mut Vec<Member>, i: usize) {
    family.remove(i);
    let shift = |p: Option<usize>| match p {
        Some(j) if j == i => None,
        Some(j) if j > i => Some(j - 1),
        other => other,
    };
    for m in family.iter_mut() {
        m.mother = shift(m.mother);
        m.father = shift(m.father);
    }
}

fn plan_band(params: &PeopleParams, d: &mut Draws, size: u32) -> Vec<Vec<Member>> {
    let size = size as usize;
    let min_families = params.band.min_families as usize;
    let mut families: Vec<Vec<Member>> = Vec::new();
    let mut total = 0usize;
    while total < size || families.len() < min_families {
        let f = plan_family(params, d);
        total += f.len();
        families.push(f);
        if families.len() > 4 * size {
            break;
        }
    }
    // Trim to the exact size: the youngest children of the latest families first, then their
    // other extra members, and whole families only when every family is a bare couple.
    while total > size {
        let child = families.iter().enumerate().rev().find_map(|(fi, f)| {
            (2..f.len())
                .filter(|&i| f[i].mother == Some(0))
                .min_by(|&a, &b| f[a].age.total_cmp(&f[b].age))
                .map(|i| (fi, i))
        });
        let extra = families
            .iter()
            .rposition(|f| f.len() > 2)
            .map(|fi| (fi, families[fi].len() - 1));
        match child.or(extra) {
            Some((fi, i)) => {
                remove_member(&mut families[fi], i);
                total -= 1;
            }
            None if families.len() > min_families => {
                total -= families.pop().map_or(0, |f| f.len());
            }
            None => break,
        }
    }
    // Dropping a couple can leave the band one short: an unattached young adult joins.
    let mut k = 0usize;
    while total < size && !families.is_empty() {
        let at = k % families.len();
        families[at].push(Member {
            sex: if d.unit() < 0.5 {
                Sex::Female
            } else {
                Sex::Male
            },
            age: d.range(15.0, 22.0),
            mother: None,
            father: None,
        });
        total += 1;
        k += 1;
    }
    families
}

/// The ground a site is judged on, worked out once for all the candidates.
struct SiteGround {
    slopes: Vec<f32>,
    hand: Vec<f32>,
}

impl SiteGround {
    fn of(ctx: &Ctx) -> SiteGround {
        SiteGround {
            slopes: terrain::slopes(ctx.map),
            hand: terrain::height_above_drainage(
                ctx.map,
                (ctx.land_params.channel_area_km2 * 1.0e6) as f32,
            ),
        }
    }
}

/// How far a field may lie from the hearth, metres: the content's longest walk to a field, off
/// trail on the flat.
pub(crate) fn field_reach_m(params: &PeopleParams) -> f32 {
    (params.nav.tobler_ms(0.0) * params.nav.offtrail_factor * params.farm.max_walk_minutes * 60.0)
        as f32
}

/// Up to `want` candidate camp sites: dry, walkable, gentle cells drawn at random.
fn site_pool(
    ctx: &Ctx,
    params: &PeopleParams,
    ground: &SiteGround,
    d: &mut Draws,
    want: usize,
) -> Vec<usize> {
    let map = ctx.map;
    let n = map.cell_count();
    let mut cells = Vec::new();
    let mut attempts = 0usize;
    while cells.len() < want && attempts < 50 * want {
        attempts += 1;
        let cell = (d.0.next_u64() % n as u64) as usize;
        if map.water[cell] != WATER_LAND
            || !ctx.nav.walkable(cell)
            || f64::from(ground.slopes[cell]) > params.band.site_max_slope
        {
            continue;
        }
        cells.push(cell);
    }
    cells
}

/// How much of each land patch could be cropped, 0–1: its arable ground, less by the work of
/// clearing it before it is broken (research 10-01 §1.1).
fn cropland_weights(ctx: &Ctx, params: &PeopleParams) -> Vec<f64> {
    let break_h = ctx
        .catalog
        .crops
        .get(params.farm.crop)
        .map_or(1.0, |c| c.break_h_per_ha);
    ctx.land
        .patches
        .class
        .iter()
        .map(|&c| match ctx.land_params.habitats.get(usize::from(c)) {
            Some(h) if h.arable => break_h / (break_h + h.clear_h_per_ha).max(1e-6),
            _ => 0.0,
        })
        .collect()
}

/// Hectares that could be cropped within a field walk of `at`, as [`site_scores`] counts them.
pub(crate) fn cropland_ha(ctx: &Ctx, params: &PeopleParams, at: (f32, f32)) -> f64 {
    let patches = &ctx.land.patches;
    let reach = field_reach_m(params);
    cropland_weights(ctx, params)
        .iter()
        .enumerate()
        .filter(|&(p, &w)| {
            let c = patches.centre_m(p);
            w > 0.0 && (c.0 - at.0).hypot(c.1 - at.1) <= reach
        })
        .map(|(p, &w)| {
            let dry = 1.0 - f64::from(patches.water.get(p).copied().unwrap_or(0.0));
            patches.area_ha() * dry * w
        })
        .sum()
}

/// A cell's slope and height above drainage, measured on their own (no map-wide rasters).
pub(crate) fn ground_at(ctx: &Ctx, cell: usize) -> (f32, f32) {
    (
        terrain::slope_at(ctx.map, cell),
        terrain::hand_at(
            ctx.map,
            cell,
            (ctx.land_params.channel_area_km2 * 1.0e6) as f32,
        ),
    )
}

/// Each of `cells` scored as a camp for a band of `size`: wild food near, cropland within a field
/// walk, water, slope and floods. `ground` gives a cell's slope and its height above drainage.
pub(crate) fn site_scores(
    ctx: &Ctx,
    params: &PeopleParams,
    ground: &dyn Fn(usize) -> (f32, f32),
    cells: &[usize],
    size: u32,
) -> Vec<f32> {
    let map = ctx.map;
    let band = &params.band;
    let (w, h) = (map.width as i64, map.height as i64);
    let year_need = f64::from(size) * params.household.daily_kcal_per_person * 365.0;
    let patches = &ctx.land.patches;
    let radius = band.site_radius_m as f32;
    let today = ctx.now.day_index();
    // Resources that yield food, with the food energy of one unit of their stock and what of them
    // stands today.
    let food_resources: Vec<(usize, f64, f64)> = ctx
        .land_params
        .resources
        .iter()
        .enumerate()
        .filter_map(|(r, res)| {
            let good = ctx.catalog.goods.get(res.good)?;
            (good.purpose == GoodUse::Food && good.kcal_per_kg > 0.0).then(|| {
                (
                    r,
                    res.unit_kg * good.kcal_per_kg,
                    ctx.land.standing(ctx.land_params, r, today),
                )
            })
        })
        .collect();
    // Land that can be cropped within a field walk, against the area the band means to crop
    // (research 10-01 §1.1: farmers choose a livelihood catchment before a residential core).
    // Ground that must first be cleared counts for less, by the work of breaking it.
    let crop = ctx.catalog.crops.get(params.farm.crop);
    let need_ha = crop.map_or(0.0, |c| {
        let kcal = ctx.catalog.goods.get(c.good).map_or(0.0, |g| g.kcal_per_kg);
        farm::need_area_ha(size as usize, params, c, kcal, c.yield_kg_per_ha)
    });
    let field_reach_m = field_reach_m(params);
    let arable = cropland_weights(ctx, params);
    let mut scores = Vec::with_capacity(cells.len());
    for &cell in cells {
        let at = cell_centre(map, cell);
        // Wild food within the radius: what a person-hour of work would bring in each patch, in
        // kcal, summed over the food resources.
        let mut food = 0.0;
        let mut arable_ha = 0.0;
        for (p, &cropland) in arable.iter().enumerate() {
            let c = patches.centre_m(p);
            let distance = (c.0 - at.0).hypot(c.1 - at.1);
            if distance <= field_reach_m && cropland > 0.0 {
                let dry = 1.0 - f64::from(patches.water.get(p).copied().unwrap_or(0.0));
                arable_ha += patches.area_ha() * dry * cropland;
            }
            if distance > radius {
                continue;
            }
            for &(r, kcal_per_unit, standing) in &food_resources {
                food += ctx.land.typical_rate_at(ctx.land_params, r, p, standing) * kcal_per_unit;
            }
        }
        // Distance to fresh water, searched in rings up to 512 m.
        let (x, y) = ((cell as i64) % w, (cell as i64) / w);
        let mut water_m = 600.0f64;
        'rings: for r in 0..=64i64 {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    if matches!(map.water[(ny * w + nx) as usize], WATER_RIVER | WATER_LAKE) {
                        water_m = ((dx * dx + dy * dy) as f64).sqrt() * f64::from(map.cell_size_m);
                        break 'rings;
                    }
                }
            }
        }
        // Eight hours a day of one person's work for a year, against the band's yearly needs.
        let food_years = food * 8.0 * 365.0 / year_need.max(1.0);
        let (slope, hand) = ground(cell);
        let mut score = band.site_w_food * (1.0 + food_years).ln()
            - band.site_w_water_per_100m * water_m / 100.0
            - band.site_w_slope_per_pct * f64::from(slope) * 100.0;
        if need_ha > 0.0 {
            score += band.site_w_arable * (1.0 + arable_ha / need_ha).ln();
        }
        if f64::from(hand) < band.site_flood_hand_m {
            score -= band.site_w_flood;
        }
        scores.push(score as f32);
    }
    scores
}

/// Camp sites for founding groups of `sizes` people, chosen together (ADR-0018 §6): one pool of
/// candidates, each scored for each group, and one draw over the summed scores of the
/// assignments whose fields' reaches do not overlap, so the order the groups are listed in
/// cannot decide who gets the best land. Groups are taken largest first (groups of a size are
/// alike). Returns each group's cell, in the order given, or `None` when no assignment fits.
fn choose_sites(
    ctx: &Ctx,
    params: &PeopleParams,
    d: &mut Draws,
    sizes: &[u32],
) -> Option<Vec<usize>> {
    let ground = SiteGround::of(ctx);
    let groups = sizes.len().max(1);
    let pool = site_pool(
        ctx,
        params,
        &ground,
        d,
        params.band.camp_candidates as usize * groups,
    );
    if pool.is_empty() || sizes.is_empty() {
        return None;
    }
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by_key(|&g| std::cmp::Reverse(sizes[g]));
    let scores: Vec<Vec<f32>> = order
        .iter()
        .map(|&g| {
            let at = |c: usize| (ground.slopes[c], ground.hand[c]);
            site_scores(ctx, params, &at, &pool, sizes[g])
        })
        .collect();
    let apart = 2.0 * field_reach_m(params);
    let at: Vec<(f32, f32)> = pool.iter().map(|&c| cell_centre(ctx.map, c)).collect();
    let fits = |a: usize, b: usize| a != b && (at[a].0 - at[b].0).hypot(at[a].1 - at[b].1) >= apart;
    let n = pool.len();
    let mut totals: Vec<f32> = Vec::new();
    let mut choices: Vec<[u16; 3]> = Vec::new();
    match scores.len() {
        1 => {
            totals = scores[0].clone();
            choices = (0..n).map(|a| [a as u16, 0, 0]).collect();
        }
        2 => {
            for a in 0..n {
                for b in (0..n).filter(|&b| fits(a, b)) {
                    totals.push(scores[0][a] + scores[1][b]);
                    choices.push([a as u16, b as u16, 0]);
                }
            }
        }
        _ => {
            for a in 0..n {
                for b in (0..n).filter(|&b| fits(a, b)) {
                    for c in (0..n).filter(|&c| fits(a, c) && fits(b, c)) {
                        totals.push(scores[0][a] + scores[1][b] + scores[2][c]);
                        choices.push([a as u16, b as u16, c as u16]);
                    }
                }
            }
        }
    }
    if totals.is_empty() {
        return None;
    }
    let (i, _, _) = decide::choose(&totals, &params.decision, d.unit());
    let mut cells = vec![0; sizes.len()];
    for (k, &g) in order.iter().enumerate().take(3) {
        cells[g] = pool[usize::from(choices[i][k])];
    }
    Some(cells)
}

/// Adds a planned family to the world: a household of `settlement` living at `home`, carrying in
/// `provisions_days` days of food and seed as the founding band does. Returns the household and
/// its people, in the family's order (the mother, the father, then the rest).
#[allow(clippy::too_many_arguments)]
fn add_family(
    pop: &mut Population,
    ctx: &mut Ctx,
    family: &[Member],
    settlement: PermanentId,
    band: Option<PermanentId>,
    home: (f32, f32),
    origin: Origin,
    provisions_days: f64,
    d: &mut Draws,
    used_names: &mut Vec<String>,
) -> (PermanentId, Vec<PermanentId>) {
    let params = ctx.params;
    let now = ctx.now;
    let mut people = Vec::new();
    let hh_id = ctx.ids.allocate();
    let ids: Vec<PermanentId> = family.iter().map(|_| ctx.ids.allocate()).collect();
    let members = family.len() as f64;
    // The band carries its provisions in; each family holds its own share.
    let mut stores = vec![0.0; ctx.catalog.goods.len()];
    if let Some(good) = ctx.catalog.goods.get(params.band.provisions_good)
        && good.kcal_per_kg > 0.0
    {
        stores[params.band.provisions_good] =
            members * params.household.daily_kcal_per_person * provisions_days / good.kcal_per_kg;
    }
    // And seed for the crop they know.
    if let Some(seed) = ctx
        .catalog
        .crops
        .get(params.farm.crop)
        .and_then(|c| stores.get_mut(c.seed_good))
    {
        *seed += members * params.band.seed_kg_per_person;
    }
    // And the tools their work needs (ADR-0006 §1), but for those that stay where they are made
    // (an oven), which they build where they settle.
    let ages: Vec<f64> = family.iter().map(|m| m.age).collect();
    let founders_know = |t: usize| {
        params
            .knowledge
            .founders
            .iter()
            .any(|&(x, s)| x == t && s > 0.0)
    };
    let wants = crate::make::tool_wants_for(
        ctx.catalog,
        &ages,
        params.family.independent_age,
        &founders_know,
    );
    for ((kg, want), good) in stores.iter_mut().zip(wants).zip(&ctx.catalog.goods) {
        if !good.tool.as_ref().is_some_and(|t| t.fixed) {
            *kg += want;
        }
    }
    let mut flows = crate::person::Flows::default();
    for (g, kg) in stores.iter().enumerate() {
        flows.add(crate::person::Flow::Brought, g, *kg);
    }
    pop.insert_household(Household {
        id: hh_id,
        members: ids.clone(),
        home,
        settlement: Some(settlement),
        stores,
        stores_at: now,
        water_l: members
            * params.household.water_l_per_person_day
            * params.household.water_target_days,
        water_at: now,
        water_use_l: 0.0,
        known: Vec::new(),
        sheltered: false,
        keeping: crate::person::Keeping::default(),
        flows,
        offers: Vec::new(),
        // Its band's way of building, or its own when it comes alone (M3b slice R).
        taste: crate::style::founding_taste(&params.style, ctx.seed, band.unwrap_or(hh_id), hh_id),
        admired: None,
        midden: crate::person::Midden::begun(now),
    });
    // One who comes alone has no partner; a family's first two are its couple.
    let couple = (family.len() >= 2).then(|| founding_couple(params, family, now, d));
    for (mi, m) in family.iter().enumerate() {
        let list = if m.sex == Sex::Male {
            &params.names.male
        } else {
            &params.names.female
        };
        let mut given = String::new();
        for _ in 0..12 {
            if let Some(n) = d.pick(list)
                && !used_names.contains(n)
            {
                given = n.clone();
                break;
            }
        }
        if given.is_empty() {
            given = d
                .pick(list)
                .cloned()
                .unwrap_or_else(|| "Unnamed".to_owned());
        }
        used_names.push(given.clone());
        let born = SimTime::from_minutes(
            now.minutes() - (m.age * civ_core::time::MINUTES_PER_YEAR as f64) as i64,
        );
        let mother = m.mother.map(|i| ids[i]);
        let father = m.father.map(|i| ids[i]);
        let id = ids[mi];
        pop.records.insert(
            id,
            PersonRecord {
                id,
                given: given.clone(),
                sex: m.sex,
                born,
                died: None,
                left: None,
                mother,
                father,
                origin,
                residence: vec![crate::history::Stay {
                    settlement: Some(settlement),
                    since: now,
                    why: match origin {
                        Origin::Founder => crate::history::ResidenceWhy::Founder,
                        Origin::Born => crate::history::ResidenceWhy::Born,
                        Origin::Spawned => crate::history::ResidenceWhy::Arrived,
                    },
                }],
            },
        );
        let traits = Traits {
            openness: d.normal() as f32,
            conscientiousness: d.normal() as f32,
            extraversion: d.normal() as f32,
            agreeableness: d.normal() as f32,
            neuroticism: d.normal() as f32,
            risk: d.normal() as f32,
        };
        pop.insert_person(Person {
            id,
            given,
            sex: m.sex,
            born,
            mother,
            father,
            household: hh_id,
            traits,
            pos: home,
            energy_kcal: 0.0,
            satiety_until: now.plus_minutes(-120),
            sleep_pressure: params.sleep.wake_pressure as f32,
            relatedness: 0.6,
            needs_at: now,
            burn_kcal_min: 1.0,
            asleep: false,
            company: params.social.household_quality as f32,
            act: Activity {
                def: 0,
                target: Target::None,
                steps: Vec::new(),
                step: 0,
                started: now,
                step_started: now,
                step_ends: now,
                version: 0,
            },
            trip: None,
            carrying: Load::default(),
            draws: 0,
            receipts: Default::default(),
            partner: match (&couple, mi) {
                (Some(_), 0) => Some(ids[1]),
                (Some(_), 1) => Some(ids[0]),
                _ => None,
            },
            repro: match (&couple, mi) {
                (Some(c), 0) => c.repro(&ids),
                _ => Repro::Open,
            },
            fecundity: match m.sex {
                Sex::Female => demography::fecundity(&params.fertility, &mut d.0),
                Sex::Male => 1.0,
            },
            nursing: match (&couple, mi) {
                (Some(c), 0) => c.nursing.map(|i| ids[i]),
                _ => None,
            },
            skills: founder_skills(
                &ctx.catalog.skills,
                m.age,
                params.family.independent_age,
                &mut Rng64::from_key(&[ctx.seed, PURPOSE_SKILLS, id.get()]),
            ),
            knows: crate::knowledge::founder_knowledge(
                ctx.catalog,
                &params.knowledge,
                params.family.independent_age,
                m.age,
                &mut Rng64::from_key(&[ctx.seed, PURPOSE_KNOW, id.get()]),
                now,
            ),
            tried: None,
            // Founders' objections come from the content's distribution, each by its own keyed
            // draw (M4b slice AA): none of their parents lived here.
            objection: crate::crime::draw_objection(
                None,
                None,
                &params.crime,
                &mut demography::life_rng(ctx.seed, id, 0, demography::Draw::Objection),
            ),
            risk_seen: params.crime.risk_prior as f32,
            guarded: None,
        });
        people.push(id);
    }
    if let Some(c) = couple {
        pop.unions.push(Union {
            woman: ids[0],
            man: ids[1],
            since: c.since,
            ended: None,
        });
    }
    (hh_id, people)
}

/// The skills a founder of `age` brings: grown people (from `grown_at`) a level drawn for each
/// skill from its founders' range, younger ones a part of it growing from age 10 (ADR-0006 §2).
pub fn founder_skills(
    skills: &[crate::params::SkillDef],
    age: f64,
    grown_at: f64,
    rng: &mut Rng64,
) -> Vec<(u16, f32)> {
    let share = ((age - 10.0) / (grown_at - 10.0).max(1.0)).clamp(0.0, 1.0);
    skills
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let [lo, hi] = s.founder_level;
            let level = (lo + (hi - lo) * rng.next_f64()) * share;
            (level > 0.0).then_some((i as u16, level as f32))
        })
        .collect()
}

/// Most founding groups a world is made with (ADR-0018 §6).
pub const MAX_FOUNDING_GROUPS: usize = 3;

/// Brings a founding band of `size` people into the world at a site they choose, founds their
/// settlement, records the chronicle, and has everyone decide what to do first.
pub fn found_band(pop: &mut Population, ctx: &mut Ctx, size: u32) -> Result<Founded, String> {
    found_bands(pop, ctx, &[size]).map(|mut all| all.remove(0))
}

/// Brings founding groups of `sizes` people (at most [`MAX_FOUNDING_GROUPS`]) into the world,
/// each a band of its own, at sites chosen together so that none counts on another's fields
/// (ADR-0018 §6), and founds a settlement for each as [`found_band`] does. Fails when there is no
/// ground to camp on, or no room for the groups apart.
pub fn found_bands(
    pop: &mut Population,
    ctx: &mut Ctx,
    sizes: &[u32],
) -> Result<Vec<Founded>, String> {
    let sizes = &sizes[..sizes.len().min(MAX_FOUNDING_GROUPS)];
    let mut d = Draws(Rng64::from_key(&[
        ctx.seed,
        PURPOSE_BAND,
        ctx.now.minutes() as u64,
    ]));
    let cells = choose_sites(ctx, ctx.params, &mut d, sizes).ok_or_else(|| match sizes.len() {
        0 | 1 => "there is no dry, gentle ground to camp on".to_owned(),
        n => format!("there is no room for {n} settlements whose fields lie apart"),
    })?;
    Ok(sizes
        .iter()
        .zip(cells)
        .map(|(&size, cell)| settle_band(pop, ctx, &mut d, cell, size))
        .collect())
}

/// Draws to name a new settlement until the name is none of `settlements`' on record, abandoned
/// ones too: a draw that names no other place is the one a world has always drawn. After
/// [`NAME_TRIES`] draws that all name one, the last is told apart by a number.
pub(crate) fn place_name(
    params: &PeopleParams,
    d: &mut Draws,
    settlements: &[civ_land::Settlement],
) -> String {
    let taken = |n: &str| settlements.iter().any(|s| s.name == n);
    let mut name = String::new();
    for _ in 0..NAME_TRIES {
        name = match (
            d.pick(&params.names.place_first).cloned(),
            d.pick(&params.names.place_second).cloned(),
        ) {
            (Some(a), Some(b)) => format!("{a}{b}"),
            (Some(a), None) => a,
            _ => "the camp".to_owned(),
        };
        if !taken(&name) {
            return name;
        }
    }
    (2..)
        .map(|k| format!("{name} {k}"))
        .find(|n| !taken(n))
        .unwrap_or(name)
}

/// Draws of a name a new settlement makes before telling a taken one apart by a number.
const NAME_TRIES: usize = 16;

/// A founding band of `size` people camps at `cell`: their settlement is founded and named,
/// their families placed around its hearth, the chronicle told, and everyone decides what to do
/// first.
fn settle_band(
    pop: &mut Population,
    ctx: &mut Ctx,
    d: &mut Draws,
    cell: usize,
    size: u32,
) -> Founded {
    let params = ctx.params;
    let now = ctx.now;
    let hearth = cell_centre(ctx.map, cell);
    let name = place_name(params, d, &ctx.land.settlements);
    let settlement = ctx.ids.allocate();
    ctx.land.settlements.push(civ_land::Settlement {
        id: settlement,
        name: name.clone(),
        founded: now,
        hearth_m: hearth,
        food_short: false,
        harvest_kg: 0.0,
        parent: None,
        founding: civ_land::Founding::Setup,
        abandoned: None,
    });

    let families = plan_band(params, d, size);
    let mut people = Vec::new();
    let mut households = Vec::new();
    let mut used_names: Vec<String> = Vec::new();
    // Homes go on dry ground a short walk from the hearth: never in a river or across one.
    let near = ctx
        .nav
        .travel_field(&ctx.map.elevation, cell, HOME_REACH_SECONDS, &|_| 0.0);
    let count = families.len().max(1);
    // A band of more families spreads out in proportion, keeping its camp as dense as a band of
    // the default size (a tuning value).
    let spread = (count as f64 / CAMP_FAMILIES).sqrt().max(1.0);
    for (fi, family) in families.iter().enumerate() {
        let angle = std::f64::consts::TAU * fi as f64 / count as f64 + d.range(-0.2, 0.2);
        let radius = d.range(12.0, 24.0) * spread;
        let wanted = (
            hearth.0 + (radius * angle.cos()) as f32,
            hearth.1 + (radius * angle.sin()) as f32,
        );
        let home = home_site(ctx.map, &near, wanted).unwrap_or(hearth);
        let (hh_id, ids) = add_family(
            pop,
            ctx,
            family,
            settlement,
            Some(settlement),
            home,
            Origin::Founder,
            params.band.provisions_days,
            d,
            &mut used_names,
        );
        households.push(hh_id);
        people.extend(ids);
    }
    // The band brings at least one knower of each technique its people know (ADR-0008 §1), and
    // the settlement's record begins with what they brought.
    pop.ensure_knowers(ctx.catalog, params, now, &people);
    pop.note_arrivals(ctx.catalog, now, settlement, &people);
    // Its way of building as it was founded (M5b slice AR): the band's.
    pop.founding_ways.insert(
        settlement,
        crate::style::band_way(&params.style, ctx.seed, settlement),
    );
    pop.chronicle_push(
        now,
        ChronicleKind::BandArrived,
        people.clone(),
        Some(settlement),
        Some(hearth),
        people.len() as f64,
        String::new(),
    );
    pop.chronicle_push(
        now,
        ChronicleKind::SettlementFounded,
        Vec::new(),
        Some(settlement),
        Some(hearth),
        0.0,
        name.clone(),
    );
    for &id in &people {
        pop.begin(ctx, id);
    }
    Founded {
        settlement,
        name,
        hearth,
        people,
        households,
    }
}

/// Most families the observer sends at once.
pub const MAX_SPAWN_FAMILIES: u32 = 20;

/// Metres between the homes of families sent together, about (a tuning value: a camp about as
/// dense as a founding band's).
const SPAWN_SPACING_M: f64 = 15.0;

/// The observer's god tool for several families at once (chain migration: people follow those
/// who went before, research 05-06 §1.2). `count` families (at most [`MAX_SPAWN_FAMILIES`]) arrive
/// together: the first where the observer placed it, as [`spawn_family`] places one, and the
/// others around it on dry ground, each made, provisioned and chronicled the same way. All of them
/// join the settlement the first joins or founds. Fails only when not even the first can be
/// placed; fewer than `count` come when the ground around is too wet or steep for the rest.
pub fn spawn_families(
    pop: &mut Population,
    ctx: &mut Ctx,
    at: (f32, f32),
    count: u32,
) -> Result<Vec<Spawned>, String> {
    let count = count.clamp(1, MAX_SPAWN_FAMILIES) as usize;
    let first = spawn_one(pop, ctx, at, 0, None, None, Sent::Family)?;
    let joining = Some((first.settlement, first.name.clone()));
    let band = Some(first.household);
    let mut out = vec![first];
    // The others on a spiral around the first, one home to each point; a point that is not dry
    // land people can walk on is passed over.
    let golden = std::f64::consts::PI * (3.0 - 5f64.sqrt());
    for k in 1..count * 4 {
        if out.len() >= count {
            break;
        }
        let (r, angle) = (SPAWN_SPACING_M * (k as f64).sqrt(), golden * k as f64);
        let p = (
            at.0 + (r * angle.cos()) as f32,
            at.1 + (r * angle.sin()) as f32,
        );
        if let Ok(spawned) = spawn_one(pop, ctx, p, k as u64, joining.clone(), band, Sent::Family) {
            out.push(spawned);
        }
    }
    Ok(out)
}

/// What sending a family produced.
#[derive(Clone, Debug, PartialEq)]
pub struct Spawned {
    /// The settlement it joined or founded.
    pub settlement: PermanentId,
    /// Its name.
    pub name: String,
    /// Whether the family made camp alone, founding the settlement.
    pub founded: bool,
    /// The household.
    pub household: PermanentId,
    /// The people: the mother, the father, then the rest.
    pub people: Vec<PermanentId>,
}

/// The observer's god tool (plan §2, M1): a family arrives where the observer placed it. Within
/// [`SPAWN_JOIN_M`] of a settlement people live in, it joins that settlement; elsewhere it makes
/// camp on the spot and founds one. It never chooses its own site. It is made and provisioned like
/// a founding family, and the chronicle names it as the observer's doing.
pub fn spawn_family(
    pop: &mut Population,
    ctx: &mut Ctx,
    at: (f32, f32),
) -> Result<Spawned, String> {
    spawn_one(pop, ctx, at, 0, None, None, Sent::Family)
}

/// Whom the observer sends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sent {
    /// A family, as a founding family is made.
    Family,
    /// One adult alone (M4c slice AJ's agitator).
    Lone,
    /// A household of a migration wave (M5a slice AN), carrying `days` days of food, one of its
    /// couple a child of `parents` when given (the woman when `woman`, else the man, or the other
    /// when that one already has parents in the family).
    Wave {
        days: u32,
        parents: Option<(PermanentId, PermanentId)>,
        woman: bool,
    },
}

/// The observer's god tool (M4c slice AJ, ADR-0016 §5): one adult newcomer, holding ideology
/// `ideology` (an index in the catalog's ideologies), arrives where the observer placed them, as
/// a family is sent, in a household of their own. They are an ordinary person, bound by every law,
/// and start with a stranger's ties: they know nobody. Returns what arrived and the
/// intervention's number.
pub fn send_agitator(
    pop: &mut Population,
    ctx: &mut Ctx,
    at: (f32, f32),
    ideology: u16,
) -> Result<(Spawned, u32), String> {
    let name = ctx
        .catalog
        .ideologies
        .get(usize::from(ideology))
        .map(|d| d.name.to_lowercase())
        .ok_or_else(|| format!("there is no ideology {ideology}"))?;
    let sent = spawn_one(pop, ctx, at, 0, None, None, Sent::Lone)?;
    let Some(&id) = sent.people.first() else {
        return Err("nobody arrived".to_owned());
    };
    let (now, day) = (ctx.now, ctx.now.day_index());
    pop.ideologies.take_up(crate::ideology::Holding {
        holder: id,
        ideology,
        since: day,
        from: None,
    });
    let record = pop.influences.add(
        now,
        crate::influence::InfluenceKind::Agitator,
        id,
        u32::from(ideology),
    );
    if let Some(r) = pop.influences.list.last_mut() {
        r.taken = Some(day);
    }
    pop.chronicle_push(
        now,
        ChronicleKind::Influence,
        vec![id],
        Some(sent.settlement),
        pop.person(id).map(|p| p.pos),
        f64::from(crate::influence::InfluenceKind::Agitator.code()),
        format!(", who holds to {name}, to {}.", sent.name),
    );
    Ok((sent, record))
}

/// How far from where a wave is sent its households know the settlements, told of them on the way
/// in (M5a slice AN; a design prior: an hour and a quarter's walk, beyond the 3 km setup keeps
/// founding groups apart).
pub const WAVE_KNOWN_M: f32 = 5_000.0;

/// The observer's god tool (M5a slice AN, ADR-0016 §5, ADR-0018 §3): a migration wave of
/// `households` households of one band (within [`WAVE_HOUSEHOLDS`]) comes to `at` from the map's
/// edge nearest it over `days` days (within [`WAVE_DAYS`]): those due today at once, the rest at
/// the midnights after ([`wave_arrivals`]). Each household is made as a sent family is, carrying
/// `months` months of food (within [`WAVE_MONTHS`]) with seed and tools; all build in the way of
/// the first household's band; in each run of [`WAVE_SIBLINGS`] households one of each couple is a
/// brother or sister of the others; and each knows the settlements it passed within sight of on
/// its way in from the edge and those within [`WAVE_KNOWN_M`] of `at`. They join the settlement
/// there or found one, as a sent family does; nothing chooses where they go next. Returns the
/// intervention's number and the households that came at once.
pub fn send_wave(
    pop: &mut Population,
    ctx: &mut Ctx,
    at: (f32, f32),
    households: u32,
    days: u32,
    months: u32,
) -> Result<(u32, Vec<Spawned>), String> {
    let within = |v: u32, [lo, hi]: [u32; 2]| (lo..=hi).contains(&v);
    if !within(households, WAVE_HOUSEHOLDS) {
        return Err(format!(
            "a wave brings {} to {} households (asked {households})",
            WAVE_HOUSEHOLDS[0], WAVE_HOUSEHOLDS[1]
        ));
    }
    if !within(days, WAVE_DAYS) {
        return Err(format!(
            "a wave comes over {} to {} days (asked {days})",
            WAVE_DAYS[0], WAVE_DAYS[1]
        ));
    }
    if !within(months, WAVE_MONTHS) {
        return Err(format!(
            "a wave carries {} to {} months of food (asked {months})",
            WAVE_MONTHS[0], WAVE_MONTHS[1]
        ));
    }
    let (w, h) = ctx.map.extent_m();
    let (x, y) = (f64::from(at.0), f64::from(at.1));
    if !(x >= 0.0 && y >= 0.0 && x < w && y < h) {
        return Err("a wave can only be sent to a point on the map".to_owned());
    }
    let cell = cell_of(ctx.map, at);
    if ctx.map.water[cell] != WATER_LAND || !ctx.nav.walkable(cell) {
        return Err("a wave can only be sent to dry land people can walk on".to_owned());
    }
    // The edge nearest where it was sent, and which edge that is.
    let (w, h) = (w as f32, h as f32);
    let (edge, side) = [
        ((0.0, at.1), at.0, "west"),
        ((w, at.1), w - at.0, "east"),
        ((at.0, 0.0), at.1, "north"),
        ((at.0, h), h - at.1, "south"),
    ]
    .into_iter()
    .min_by(|a, b| a.1.total_cmp(&b.1))
    .map(|(p, _, side)| (p, side))
    .unwrap_or(((0.0, at.1), "west"));
    let runs = households.div_ceil(WAVE_SIBLINGS);
    let parents = (0..runs)
        .map(|_| (ctx.ids.allocate(), ctx.ids.allocate()))
        .collect();
    let day = ctx.now.day_index();
    // Its number, known before anyone comes, keys its households' draws apart from those of any
    // other family sent to the same point at the same minute.
    let record = pop.influences.list.last().map_or(0, |i| i.id) + 1;
    let mut wave = Wave {
        record,
        at,
        edge,
        day,
        households,
        days,
        provisions_days: (i64::from(months) * DAYS_PER_YEAR / 12) as u32,
        arrived: 0,
        came: 0,
        next: 0,
        settlement: None,
        band: None,
        parents,
        people: Vec::new(),
    };
    let came = arrive(pop, ctx, &mut wave, day);
    let (Some(first), Some(sent)) = (wave.people.first().copied(), came.first()) else {
        return Err("no household of the wave found room to camp there".to_owned());
    };
    let now = ctx.now;
    let added = pop
        .influences
        .add(now, InfluenceKind::Wave, first, households);
    debug_assert_eq!(added, record);
    pop.chronicle_push(
        now,
        ChronicleKind::Influence,
        vec![first],
        Some(sent.settlement),
        Some(at),
        f64::from(InfluenceKind::Wave.code()),
        format!(
            "'s household, the first of a wave of {households} households from the {side}, to {}.",
            sent.name
        ),
    );
    pop.influences.waves.push(wave);
    Ok((record, came))
}

/// The households of the waves whose day has come arrive (at midnight; M5a slice AN).
pub(crate) fn wave_arrivals(pop: &mut Population, ctx: &mut Ctx) {
    let day = ctx.now.day_index();
    for i in 0..pop.influences.waves.len() {
        let w = &pop.influences.waves[i];
        if w.done() || w.due(w.arrived) > day {
            continue;
        }
        let mut wave = w.clone();
        arrive(pop, ctx, &mut wave, day);
        pop.influences.waves[i] = wave;
    }
}

/// Each household of `wave` due by `day` comes: at the next point of the spiral around where it
/// was sent that takes a home (as sent families are placed), joining the settlement its first
/// household joined or founded while anyone lives there. One that finds no room within four
/// points a household goes on and never comes.
fn arrive(pop: &mut Population, ctx: &mut Ctx, wave: &mut Wave, day: i64) -> Vec<Spawned> {
    let golden = std::f64::consts::PI * (3.0 - 5f64.sqrt());
    let mut out = Vec::new();
    while !wave.done() && wave.due(wave.arrived) <= day {
        let joining = wave.settlement.and_then(|id| {
            ctx.land
                .settlements
                .iter()
                .find(|s| s.id == id && s.abandoned.is_none())
                .map(|s| (s.id, s.name.clone()))
        });
        let sent = Sent::Wave {
            days: wave.provisions_days,
            parents: wave
                .parents
                .get((wave.arrived / WAVE_SIBLINGS) as usize)
                .copied(),
            woman: wave.arrived.is_multiple_of(2),
        };
        let mut placed = None;
        while placed.is_none() && wave.next < wave.households * 4 {
            let k = wave.next;
            wave.next += 1;
            let (r, angle) = (SPAWN_SPACING_M * f64::from(k).sqrt(), golden * f64::from(k));
            let p = (
                wave.at.0 + (r * angle.cos()) as f32,
                wave.at.1 + (r * angle.sin()) as f32,
            );
            let index = (u64::from(wave.record) << 32) | u64::from(k);
            placed = spawn_one(pop, ctx, p, index, joining.clone(), wave.band, sent).ok();
        }
        wave.arrived += 1;
        let Some(spawned) = placed else {
            continue;
        };
        wave.came += 1;
        if wave.band.is_none() {
            wave.band = Some(spawned.household);
        }
        if joining.is_none() {
            wave.settlement = Some(spawned.settlement);
        }
        wave_knows(pop, ctx, wave, spawned.household, spawned.settlement);
        wave.people.extend_from_slice(&spawned.people);
        out.push(spawned);
    }
    out
}

/// Household `household` of `wave`, living in `home`, knows the settlements lived in that it
/// passed within sight of on its way in from the edge, and has been told of those within
/// [`WAVE_KNOWN_M`] of where it was sent (ADR-0018 §4).
fn wave_knows(
    pop: &mut Population,
    ctx: &Ctx,
    wave: &Wave,
    household: PermanentId,
    home: PermanentId,
) {
    let day = ctx.now.day_index();
    let way = [wave.edge, wave.at];
    for s in &ctx.land.settlements {
        if s.id == home || s.abandoned.is_some() {
            continue;
        }
        let how = if distance_to_walk(&way, s.hearth_m) <= ctx.params.places.sight_m {
            PlaceHow::Seen
        } else if (s.hearth_m.0 - wave.at.0).hypot(s.hearth_m.1 - wave.at.1) <= WAVE_KNOWN_M {
            PlaceHow::Told
        } else {
            continue;
        };
        pop.known_places.learn(household, s.id, day, how, None);
    }
}

/// One family sent to `at`: the `index`th of those sent together (its own draws), joining
/// `joining` when given or else as [`spawn_family`] decides, and building as the first of them,
/// household `band`, does when given, or in its own way.
fn spawn_one(
    pop: &mut Population,
    ctx: &mut Ctx,
    at: (f32, f32),
    index: u64,
    joining: Option<(PermanentId, String)>,
    band: Option<PermanentId>,
    sent: Sent,
) -> Result<Spawned, String> {
    let params = ctx.params;
    let now = ctx.now;
    let (w, h) = ctx.map.extent_m();
    let (x, y) = (f64::from(at.0), f64::from(at.1));
    if !(x >= 0.0 && y >= 0.0 && x < w && y < h) {
        return Err("a family can only be placed on the map".to_owned());
    }
    let cell = cell_of(ctx.map, at);
    if ctx.map.water[cell] != WATER_LAND || !ctx.nav.walkable(cell) {
        return Err("a family can only be placed on dry land people can walk on".to_owned());
    }
    // The first family's key is the one a lone family has always drawn from.
    let mut d = Draws(if index == 0 {
        Rng64::from_key(&[ctx.seed, PURPOSE_SPAWN, now.minutes() as u64, cell as u64])
    } else {
        Rng64::from_key(&[
            ctx.seed,
            PURPOSE_SPAWN,
            now.minutes() as u64,
            cell as u64,
            index,
        ])
    });
    let distance = |p: (f32, f32)| ((p.0 - at.0).powi(2) + (p.1 - at.1).powi(2)).sqrt();
    let near = joining.or_else(|| {
        let lived_in: Vec<PermanentId> = pop
            .households
            .iter()
            .filter(|(_, x)| !x.members.is_empty())
            .filter_map(|(_, x)| x.settlement)
            .collect();
        ctx.land
            .settlements
            .iter()
            .filter(|s| lived_in.contains(&s.id) && distance(s.hearth_m) <= SPAWN_JOIN_M)
            .min_by(|a, b| distance(a.hearth_m).total_cmp(&distance(b.hearth_m)))
            .map(|s| (s.id, s.name.clone()))
    });
    let wear = &ctx.land.wear;
    let reach = ctx
        .nav
        .travel_field(&ctx.map.elevation, cell, HOME_REACH_SECONDS, &|c| {
            wear.factor(c)
        });
    let home = home_site(ctx.map, &reach, at).unwrap_or_else(|| cell_centre(ctx.map, cell));
    let (settlement, name, founded) = match near {
        Some((id, name)) => (id, name, false),
        None => {
            let name = place_name(params, &mut d, &ctx.land.settlements);
            let id = ctx.ids.allocate();
            ctx.land.settlements.push(civ_land::Settlement {
                id,
                name: name.clone(),
                founded: now,
                hearth_m: home,
                food_short: false,
                harvest_kg: 0.0,
                parent: None,
                founding: if matches!(sent, Sent::Wave { .. }) {
                    civ_land::Founding::Wave
                } else {
                    civ_land::Founding::Sent
                },
                abandoned: None,
            });
            (id, name, true)
        }
    };
    let family = match sent {
        Sent::Family | Sent::Wave { .. } => plan_family(params, &mut d),
        Sent::Lone => vec![Member {
            sex: if d.unit() < 0.5 {
                Sex::Female
            } else {
                Sex::Male
            },
            age: d.range(20.0, 40.0),
            mother: None,
            father: None,
        }],
    };
    let mut used_names: Vec<String> = pop.people.iter().map(|(_, p)| p.given.clone()).collect();
    let (household, people) = add_family(
        pop,
        ctx,
        &family,
        settlement,
        band,
        home,
        Origin::Spawned,
        match sent {
            Sent::Wave { days, .. } => f64::from(days),
            Sent::Family | Sent::Lone => params.band.provisions_days,
        },
        &mut d,
        &mut used_names,
    );
    // A settlement this family founds builds as it does (M5b slice AR).
    if founded && let Some(t) = pop.household(household).map(|x| x.taste) {
        pop.founding_ways.insert(settlement, t);
    }
    // A wave's households come as brothers and sisters: one of the couple is a child of the
    // parents its run of households shares.
    if let Sent::Wave {
        parents: Some((mother, father)),
        woman,
        ..
    } = sent
    {
        let couple: Vec<usize> = (0..family.len().min(2)).collect();
        let first = if woman { 0 } else { 1 };
        let pick = [first, 1 - first].into_iter().find(|&i| {
            couple.contains(&i) && family[i].mother.is_none() && family[i].father.is_none()
        });
        if let Some(i) = pick {
            let id = people[i];
            if let Some(r) = pop.records.get_mut(&id) {
                r.mother = Some(mother);
                r.father = Some(father);
            }
            if let Some(p) = pop.person_mut_by_id(id) {
                p.mother = Some(mother);
                p.father = Some(father);
            }
            pop.forget_kin();
        }
    }
    // What the family brings that its settlement did not know is noted (ADR-0008 §2).
    pop.note_arrivals(ctx.catalog, now, settlement, &people);
    // One sent alone is told by whoever sent them.
    if sent != Sent::Lone {
        pop.chronicle_push(
            now,
            ChronicleKind::FamilyArrived,
            people.clone(),
            Some(settlement),
            Some(home),
            people.len() as f64,
            String::new(),
        );
    }
    if founded {
        pop.chronicle_push(
            now,
            ChronicleKind::SettlementFounded,
            Vec::new(),
            Some(settlement),
            Some(home),
            0.0,
            name.clone(),
        );
    }
    for &id in &people {
        pop.begin(ctx, id);
    }
    Ok(Spawned {
        settlement,
        name,
        founded,
        household,
        people,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::params::{
        BandParams, DecisionParams, EnergyParams, FarmParams, HouseholdParams, NameParams,
        SleepParams, SocialParams,
    };
    use civ_world::nav::NavParams;

    /// A full set of people parameters for tests.
    pub(crate) fn params() -> PeopleParams {
        PeopleParams {
            nav: NavParams {
                top_speed_kmh: 6.0,
                slope_sensitivity: 3.5,
                best_slope_offset: 0.05,
                offtrail_factor: 0.6,
                wading_factor: 0.25,
                ford_max_discharge_m3s: 3.0,
                max_slope: 1.0,
            },
            walk_speed_by_age: vec![(0.0, 1.0)],
            capacity_by_age: vec![(0.0, 1.0)],
            latitude_deg: 48.0,
            energy: EnergyParams {
                bmr_male: vec![(15.0, 690.0)],
                bmr_female: vec![(14.8, 487.0)],
                bmr_band_starts: vec![0.0],
                mass_by_age: vec![(0.0, 60.0, 52.0)],
                walk_par: 3.2,
                idle_par: 1.4,
                satiety_hours: 5.0,
                hunger_ramp_hours: 3.0,
                deficit_unit_kcal: 3000.0,
                max_surplus_kcal: 2000.0,
                reserve_kcal_per_kg: 1000.0,
                eat_reserve_at_deficit: 0.25,
                meal_minutes: 25,
            },
            sleep: SleepParams {
                tau_awake_h: 18.2,
                tau_asleep_h: 4.2,
                wake_pressure: 0.12,
                min_hours: 4.0,
                max_hours: 10.5,
                nap_min_minutes: 20.0,
                nap_max_minutes: 90.0,
                day_factor: 0.15,
                bedtime_after_sunset_hours: 3.3,
            },
            social: SocialParams {
                tau_h: 36.0,
                quality_per_companion: 0.2,
                household_quality: 0.45,
            },
            household: HouseholdParams {
                water_l_per_person_day: 20.0,
                carry_water_l: 15.0,
                water_use_by_walk: Vec::new(),
                water_target_days: 1.5,
                food_target_days: 5.0,
                carry_kg: 20.0,
                fuel_kg_per_person_day: [1.5; 12],
                fuel_target_days: 3.0,
                short_food_days: 2.0,
                recovered_food_days: 5.0,
                daily_kcal_per_person: 2100.0,
                leave_at_depletion: 0.3,
                leave_per_day: 0.1,
                leave_unless_ripe_within_days: 30.0,
                leave_w_gap: 6.0,
                leave_w_stake: 3.0,
                leave_stay: 2.0,
                ready_food_days: 2.0,
                harvest_margin_days: 30.0,
                raised_store_factor: 2.0,
                processed_food_days: 5.0,
            },
            decision: DecisionParams {
                temperature_sd_fraction: 0.35,
                min_temperature: 0.4,
                w_hunger: 10.0,
                max_hunger_drive: 2.0,
                w_sleep: 14.0,
                w_social: 5.0,
                w_food: 10.0,
                w_lean: 5.0,
                w_work: 2.5,
                w_fuel: 6.0,
                w_farm: 8.0,
                w_deadline: 6.0,
                w_shelter: 4.0,
                w_tools: 4.0,
                trip_half_worth_days: 0.25,
                w_water: 8.0,
                w_walk_hour: 2.0,
                w_effort: 0.8,
                w_dark: 6.0,
                w_rest: 1.0,
                w_play: 3.0,
                w_tend: 6.0,
            },
            band: BandParams {
                default_size: 40,
                min_size: 20,
                max_size: 60,
                min_families: 6,
                camp_candidates: 48,
                site_radius_m: 2000.0,
                provisions_days: 30.0,
                provisions_good: 0,
                seed_kg_per_person: 22.0,
                elder_chance: 0.3,
                young_adult_chance: 0.35,
                birth_spacing_months: 36.0,
                site_max_slope: 0.06,
                site_w_food: 3.0,
                site_w_arable: 3.0,
                site_w_water_per_100m: 1.0,
                site_w_slope_per_pct: 0.5,
                site_w_flood: 3.0,
                site_flood_hand_m: 1.5,
            },
            mortality: crate::demography::tests::mortality(),
            fertility: crate::demography::tests::fertility(),
            family: crate::demography::tests::family(),
            market: crate::params::MarketParams {
                review_days: 7,
                margin: 0.25,
                max_change: 0.05,
                stock_response: 0.5,
                memory_days: 30.0,
                money_share: 0.5,
                money_min_trades: 10.0,
                accept_want: 0.5,
                recent_trades: 64,
            },
            firm: crate::params::FirmParams {
                idle_close_days: 180.0,
                book_entries: 64,
                wage_share: 0.5,
                wage_review_days: 14,
                wage_max_change: 0.05,
                max_hire_hours: 21.0,
            },
            knowledge: crate::params::KnowledgeParams {
                founders: Vec::new(),
                max_learners: 2,
                w_learn: 2.0,
                experiment_share: 0.01,
                aware_try_factor: 3.0,
                w_try: 2.0,
                try_gap_days: 7.0,
                watch_m: 100.0,
            },
            digging: crate::params::Digging {
                h_per_m3: 8.0,
                pit_side_m: 3.0,
            },
            style: crate::params::StyleParams {
                alpha: 0.1,
                prestige_most: 3.0,
                innovation: 0.01,
                tradition_mean: crate::params::Taste {
                    pitch_centideg: 4_500.0,
                    eave_cm: 180.0,
                    overhang_cm: 50.0,
                },
                tradition_spread: crate::params::Taste {
                    pitch_centideg: 300.0,
                    eave_cm: 10.0,
                    overhang_cm: 8.0,
                },
                personal_spread: crate::params::Taste {
                    pitch_centideg: 100.0,
                    eave_cm: 3.0,
                    overhang_cm: 3.0,
                },
                seen_most: 8,
                sight_m: 200.0,
            },
            names: NameParams::default(),
            midden: crate::params::MiddenParams {
                kg_per_person_day: 0.5,
                n_kg_per_person_year: 1.0,
                half_life_days: 365.0,
                load_kg: 25.0,
                spread_h_per_t: 2.0,
            },
            ties: crate::ties::TieParams::core(),
            standing: crate::standing::StandingParams::default(),
            polity: crate::polity::PolityParams::core(),
            crime: crate::crime::CrimeParams::core(),
            word: crate::word::WordParams::core(),
            opinion: crate::opinion::OpinionParams::core(),
            faction: crate::faction::FactionParams::core(),
            places: crate::places::PlacesParams::core(),
            moving: crate::places::MovingParams::core(),
            founding: crate::places::FoundingParams::core(),
            reports: crate::reports::ReportParams::core(),
            relations: crate::views::RelationsParams::core(),
            suspicion: crate::suspicion::SuspicionParams::core(),
            farm: FarmParams {
                crop: 0,
                grain_share: 0.75,
                plan_yield_share: 0.8,
                loss_share: 0.0,
                grain_target_days: 400.0,
                work_hours_per_day: 6.0,
                peak_work_hours_per_day: 10.0,
                field_m: 50.0,
                max_walk_minutes: 30.0,
                site_candidates: 24,
            },
            build: crate::params::BuildParams {
                programs: vec![0],
                store_horizon_days: 1095.0,
                home_work_places: 2,
                quality_spread: [0.3, 0.1],
                levelling: crate::params::Levelling {
                    from_m: 0.4,
                    most_m: 2.0,
                    h_per_m3: 11.0,
                    side_run: 1.5,
                },
                caution: crate::caution::CautionParams {
                    half_life_years: 8.0,
                    most: 2.0,
                    half_rate: 0.01,
                    death_weight: 2.0,
                },
            },
        }
    }

    #[test]
    fn a_new_settlement_is_never_named_as_one_on_record() {
        let mut p = params();
        p.names.place_first = vec!["Ash".to_owned(), "Elm".to_owned()];
        p.names.place_second = vec!["ford".to_owned()];
        let place = |name: &str, id: u64| civ_land::Settlement {
            id: PermanentId::from_raw(id).expect("an id"),
            name: name.to_owned(),
            founded: SimTime::ZERO,
            hearth_m: (0.0, 0.0),
            food_short: false,
            harvest_kg: 0.0,
            parent: None,
            founding: civ_land::Founding::Setup,
            abandoned: Some(SimTime::ZERO),
        };
        let mut d = Draws(Rng64::from_key(&[1]));
        let first = place_name(&p, &mut d, &[]);
        assert!(first == "Ashford" || first == "Elmford", "{first}");
        // With one of the two taken, abandoned or not, the other is drawn.
        let other = if first == "Ashford" {
            "Elmford"
        } else {
            "Ashford"
        };
        for seed in 0..20 {
            let mut d = Draws(Rng64::from_key(&[seed]));
            assert_eq!(place_name(&p, &mut d, &[place(&first, 1)]), other);
        }
        // With both taken, a number tells the new one apart.
        let both = [
            place("Ashford", 1),
            place("Elmford", 2),
            place("Elmford 2", 3),
        ];
        let named = place_name(&p, &mut Draws(Rng64::from_key(&[3])), &both);
        assert!(named == "Ashford 2" || named == "Elmford 3", "{named}");
    }

    #[test]
    fn bands_have_the_asked_size_and_every_parent_is_in_the_family() {
        let p = params();
        for seed in 0..400u64 {
            for size in [p.band.min_size, 33, p.band.default_size, p.band.max_size] {
                let mut d = Draws(Rng64::from_key(&[seed, PURPOSE_BAND, u64::from(size)]));
                let families = plan_band(&p, &mut d, size);
                let total: usize = families.iter().map(Vec::len).sum();
                assert_eq!(total, size as usize, "seed {seed}, size {size}");
                for family in &families {
                    assert!(family.len() >= 2, "every family keeps its couple");
                    for m in family {
                        for parent in [m.mother, m.father].into_iter().flatten() {
                            assert!(parent < family.len(), "seed {seed}: a parent left the band");
                            assert!(family[parent].age >= m.age + 12.0, "parents are older");
                        }
                    }
                }
            }
        }
    }
}
