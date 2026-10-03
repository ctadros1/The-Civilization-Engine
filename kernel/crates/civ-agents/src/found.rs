//! A founding band arrives (plan §2 core loop; research 05-01 §4.5, 10-01 §1.1).
//!
//! The band is at least [`crate::params::BandParams::min_families`] unrelated families whose ages,
//! couples and children come from the life table, so the first generation is not a crowd of
//! newly paired adults (05-01 §5.4). They choose where to camp by scoring sampled sites on the
//! wild food around them, the walk to fresh water, slope and flood risk, and sampling one with a
//! softmax; the site is theirs to choose, not the engine's.

use civ_core::{PermanentId, Rng64, SimTime};
use civ_world::nav::TravelField;
use civ_world::{WATER_LAKE, WATER_LAND, WATER_RIVER, terrain};

use crate::demography;
use crate::history::{ChronicleKind, Origin, PersonRecord, Union};
use crate::needs::Sex;
use crate::params::{GoodUse, PeopleParams, step_at};
use crate::person::{Activity, Household, Load, Person, Repro, Target, Traits};
use crate::population::{Ctx, Population, cell_centre, cell_of};
use crate::{decide, farm};

/// Purpose tag for founding draws.
pub const PURPOSE_BAND: u64 = 0x6261_6e64_3030_3031; // "band0001"
/// Longest walk from the hearth to a home, seconds.
pub(crate) const HOME_REACH_SECONDS: f32 = 300.0;
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

struct Draws(Rng64);

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

/// Scores candidate camp sites and picks one. Returns the chosen cell.
fn choose_site(ctx: &Ctx, params: &PeopleParams, d: &mut Draws, size: u32) -> Option<usize> {
    let map = ctx.map;
    let band = &params.band;
    let slopes = terrain::slopes(map);
    let hand =
        terrain::height_above_drainage(map, (ctx.land_params.channel_area_km2 * 1.0e6) as f32);
    let n = map.cell_count();
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
        farm::need_area_ha(size as usize, params, c, kcal)
    });
    let break_h = crop.map_or(1.0, |c| c.break_h_per_ha);
    let field_reach_m = (params.nav.tobler_ms(0.0)
        * params.nav.offtrail_factor
        * params.farm.max_walk_minutes
        * 60.0) as f32;
    let arable: Vec<f64> = patches
        .class
        .iter()
        .map(|&c| match ctx.land_params.habitats.get(usize::from(c)) {
            Some(h) if h.arable => break_h / (break_h + h.clear_h_per_ha).max(1e-6),
            _ => 0.0,
        })
        .collect();
    let mut sites = Vec::new();
    let mut attempts = 0u32;
    while sites.len() < band.camp_candidates as usize && attempts < 50 * band.camp_candidates {
        attempts += 1;
        let cell = (d.0.next_u64() % n as u64) as usize;
        if map.water[cell] != WATER_LAND
            || !ctx.nav.walkable(cell)
            || f64::from(slopes[cell]) > band.site_max_slope
        {
            continue;
        }
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
        let mut score = band.site_w_food * (1.0 + food_years).ln()
            - band.site_w_water_per_100m * water_m / 100.0
            - band.site_w_slope_per_pct * f64::from(slopes[cell]) * 100.0;
        if need_ha > 0.0 {
            score += band.site_w_arable * (1.0 + arable_ha / need_ha).ln();
        }
        if f64::from(hand[cell]) < band.site_flood_hand_m {
            score -= band.site_w_flood;
        }
        sites.push((cell, score as f32));
    }
    if sites.is_empty() {
        return None;
    }
    let totals: Vec<f32> = sites.iter().map(|s| s.1).collect();
    let (i, _, _) = decide::choose(&totals, &params.decision, d.unit());
    Some(sites[i].0)
}

/// Brings a founding band of `size` people into the world at a site they choose, founds their
/// settlement, records the chronicle, and has everyone decide what to do first.
pub fn found_band(pop: &mut Population, ctx: &mut Ctx, size: u32) -> Result<Founded, String> {
    let params = ctx.params;
    let now = ctx.now;
    let mut d = Draws(Rng64::from_key(&[
        ctx.seed,
        PURPOSE_BAND,
        now.minutes() as u64,
    ]));
    let cell = choose_site(ctx, params, &mut d, size)
        .ok_or_else(|| "there is no dry, gentle ground to camp on".to_owned())?;
    let hearth = cell_centre(ctx.map, cell);
    let name = match (
        d.pick(&params.names.place_first).cloned(),
        d.pick(&params.names.place_second).cloned(),
    ) {
        (Some(a), Some(b)) => format!("{a}{b}"),
        (Some(a), None) => a,
        _ => "the camp".to_owned(),
    };
    let settlement = ctx.ids.allocate();
    ctx.land.settlements.push(civ_land::Settlement {
        id: settlement,
        name: name.clone(),
        founded: now,
        hearth_m: hearth,
        food_short: false,
        harvest_kg: 0.0,
    });

    let families = plan_band(params, &mut d, size);
    let mut people = Vec::new();
    let mut households = Vec::new();
    let mut used_names: Vec<String> = Vec::new();
    // Homes go on dry ground a short walk from the hearth: never in a river or across one.
    let near = ctx
        .nav
        .travel_field(&ctx.map.elevation, cell, HOME_REACH_SECONDS, &|_| 0.0);
    let count = families.len().max(1);
    for (fi, family) in families.iter().enumerate() {
        let angle = std::f64::consts::TAU * fi as f64 / count as f64 + d.range(-0.2, 0.2);
        let radius = d.range(12.0, 24.0);
        let wanted = (
            hearth.0 + (radius * angle.cos()) as f32,
            hearth.1 + (radius * angle.sin()) as f32,
        );
        let home = home_site(ctx.map, &near, wanted).unwrap_or(hearth);
        let hh_id = ctx.ids.allocate();
        let ids: Vec<PermanentId> = family.iter().map(|_| ctx.ids.allocate()).collect();
        let members = family.len() as f64;
        // The band carries its provisions in; each family holds its own share.
        let mut stores = vec![0.0; ctx.catalog.goods.len()];
        if let Some(good) = ctx.catalog.goods.get(params.band.provisions_good)
            && good.kcal_per_kg > 0.0
        {
            stores[params.band.provisions_good] =
                members * params.household.daily_kcal_per_person * params.band.provisions_days
                    / good.kcal_per_kg;
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
            known: Vec::new(),
            sheltered: false,
        });
        households.push(hh_id);
        let couple = founding_couple(params, family, now, &mut d);
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
                    origin: Origin::Founder,
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
                partner: match mi {
                    0 => Some(ids[1]),
                    1 => Some(ids[0]),
                    _ => None,
                },
                repro: if mi == 0 {
                    couple.repro(&ids)
                } else {
                    Repro::Open
                },
                fecundity: match m.sex {
                    Sex::Female => demography::fecundity(&params.fertility, &mut d.0),
                    Sex::Male => 1.0,
                },
                nursing: if mi == 0 {
                    couple.nursing.map(|i| ids[i])
                } else {
                    None
                },
            });
            people.push(id);
        }
        pop.unions.push(Union {
            woman: ids[0],
            man: ids[1],
            since: couple.since,
            ended: None,
        });
    }
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
    Ok(Founded {
        settlement,
        name,
        hearth,
        people,
        households,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{
        BandParams, DecisionParams, EnergyParams, FarmParams, HouseholdParams, NameParams,
        SleepParams, SocialParams,
    };
    use civ_world::nav::NavParams;

    fn params() -> PeopleParams {
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
            },
            decision: DecisionParams {
                temperature_sd_fraction: 0.35,
                min_temperature: 0.4,
                w_hunger: 10.0,
                max_hunger_drive: 2.0,
                w_sleep: 14.0,
                w_social: 5.0,
                w_food: 10.0,
                w_work: 2.5,
                w_fuel: 6.0,
                w_farm: 8.0,
                w_deadline: 6.0,
                w_shelter: 4.0,
                trip_half_worth_days: 0.25,
                w_water: 8.0,
                w_walk_hour: 2.0,
                w_effort: 0.8,
                w_dark: 6.0,
                w_rest: 1.0,
                w_play: 3.0,
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
            names: NameParams::default(),
            farm: FarmParams {
                crop: 0,
                grain_share: 0.75,
                plan_yield_share: 0.8,
                grain_target_days: 400.0,
                work_hours_per_day: 6.0,
                field_m: 50.0,
                max_walk_minutes: 30.0,
                site_candidates: 24,
            },
            home_program: 0,
        }
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
