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

use crate::decide;
use crate::history::{ChronicleKind, Origin, PersonRecord};
use crate::needs::Sex;
use crate::params::PeopleParams;
use crate::person::{Activity, Household, Load, Person, Target, Traits};
use crate::population::{Ctx, Population, cell_centre, cell_of, prior_rate};

/// Purpose tag for founding draws.
pub const PURPOSE_BAND: u64 = 0x6261_6e64_3030_3031; // "band0001"
/// Longest walk from the hearth to a home, seconds.
const HOME_REACH_SECONDS: f32 = 300.0;
/// Farthest a home moves from where its family wanted it, cells.
const HOME_SEARCH_CELLS: i64 = 8;

/// Where a family makes its home: the wanted point if it is on dry ground reachable from the
/// hearth, otherwise the centre of the nearest such cell within a few cells.
fn home_site(
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
        if d.unit() < params.mortality.survival(age) {
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
        // Wild food within the radius, in years of the band's needs.
        let mut food = 0.0;
        for p in 0..patches.len() {
            let c = patches.centre_m(p);
            if (c.0 - at.0).hypot(c.1 - at.1) > radius {
                continue;
            }
            for r in 0..ctx.land_params.resources.len() {
                food +=
                    prior_rate(ctx.land_params, ctx.land, r, p) * f64::from(patches.richness[p]);
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
        let food_years = food * 8.0 * 365.0 / year_need.max(1.0);
        let mut score = band.site_w_food * (1.0 + food_years).ln()
            - band.site_w_water_per_100m * water_m / 100.0
            - band.site_w_slope_per_pct * f64::from(slopes[cell]) * 100.0;
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
        pop.insert_household(Household {
            id: hh_id,
            members: ids.clone(),
            home,
            settlement: Some(settlement),
            food_kcal: members
                * params.household.daily_kcal_per_person
                * params.band.provisions_days,
            water_l: members
                * params.household.water_l_per_person_day
                * params.household.water_target_days,
            water_at: now,
            known: Vec::new(),
        });
        households.push(hh_id);
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
            });
            people.push(id);
        }
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
        BandParams, DecisionParams, EnergyParams, HouseholdParams, NameParams, Siler, SleepParams,
        SocialParams,
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
                carry_food_kcal: 12000.0,
                daily_kcal_per_person: 2100.0,
            },
            decision: DecisionParams {
                temperature_sd_fraction: 0.35,
                min_temperature: 0.4,
                w_hunger: 10.0,
                w_sleep: 14.0,
                w_social: 5.0,
                w_food: 10.0,
                w_work: 2.5,
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
                elder_chance: 0.3,
                young_adult_chance: 0.35,
                birth_spacing_months: 36.0,
                site_max_slope: 0.06,
                site_w_food: 3.0,
                site_w_water_per_100m: 1.0,
                site_w_slope_per_pct: 0.5,
                site_w_flood: 3.0,
                site_flood_hand_m: 1.5,
            },
            mortality: Siler {
                a: 0.351,
                b: 0.895,
                c: 0.011,
                d: 6.70e-6,
                e: 0.125,
            },
            names: NameParams::default(),
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
