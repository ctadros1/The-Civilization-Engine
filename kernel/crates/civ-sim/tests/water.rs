//! Water people choose (M6a slice AY, step two; ADR-0021 §1): a household's people fetch from
//! the nearest spring that flows and has a load left today, if it is nearer than the river or
//! lake, and from the river or lake otherwise; a spring gives what flows to it in a day; every
//! draw is logged as a use of the source; and what a household uses a day follows the walk to
//! the water it last fetched.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::uses::Place;
use civ_content::ContentRegistry;
use civ_sim::{NewWorld, Sim, persist};

const PRESET: &str = "core:worldgen/river_valley";
const DAY: i64 = 24 * 60;

fn content() -> &'static ContentRegistry {
    static CONTENT: OnceLock<ContentRegistry> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("content/ is above the crate");
        civ_content::load(&root)
            .registry
            .expect("the core content loads")
    })
}

/// A village of world `seed`, with a fixed identity so it lives the same life every run.
fn village(seed: u64) -> Sim {
    village_of(seed, [7; 16])
}

/// [`village`] of identity `identity`.
fn village_of(seed: u64, identity: [u8; 16]) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Spring".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        identity,
    )
    .expect("generates")
}

#[test]
fn a_stranded_load_is_delivered_before_another_task_and_survives_a_save() {
    use civ_agents::params::Behavior;
    use civ_agents::person::{Step, Target};
    use commons_persist::{SaveDir, SaveKind};

    let mut sim = village(2);
    let now = sim.now();
    let rest = sim
        .rules()
        .catalog
        .activities
        .iter()
        .position(|a| a.behavior == Behavior::Rest)
        .expect("rest") as u16;
    let (person, household, due) = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| (p.id, p.household, p.act.step_ends))
        .next()
        .expect("person");
    let home = sim.people().household(household).expect("household").home;
    {
        let hh = sim
            .people_mut_for_tests()
            .households
            .iter_mut()
            .find(|(_, h)| h.id == household)
            .map(|(_, h)| h)
            .expect("household");
        hh.water_l = 1000.0;
        hh.water_at = now;
    }
    // The state left by a failed delivery: a load remains, but the activity has no deposit.
    // Keep the existing scheduled completion, then let the ordinary decision machinery resume.
    {
        let p = sim
            .people_mut_for_tests()
            .people
            .iter_mut()
            .find(|(_, p)| p.id == person)
            .map(|(_, p)| p)
            .expect("person");
        p.pos = home;
        p.trip = None;
        p.carrying.water_l = 15.0;
        p.act.def = rest;
        p.act.target = Target::None;
        p.act.steps = vec![Step::Work { minutes: 1 }];
        p.act.step = 0;
        p.act.started = now;
        p.act.step_started = now;
    }
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "stranded water").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let minutes = (due.minutes() - now.minutes()).max(0) + 5;
    for s in [&mut sim, &mut loaded] {
        s.advance_minutes(minutes).expect("advances");
        assert_eq!(
            s.people().person(person).expect("person").carrying.water_l,
            0.0
        );
        let hh = s.people().household(household).expect("household");
        let used =
            hh.members.len() as f64 * hh.water_use(&s.rules().people.household) * minutes as f64
                / DAY as f64;
        assert!(hh.water_l >= 1015.0 - used - 1e-9, "water {}", hh.water_l);
        assert!(hh.water_l <= 1015.0);
    }
    let a = persist::encode_sections(&sim);
    let b = persist::encode_sections(&loaded);
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x.bytes, y.bytes, "section {}", x.tag);
    }
}

/// Lives on to a minute past the next midnight: the heads are lived at midnight, so what a test
/// sets them to holds until the one after.
fn to_morning(sim: &mut Sim) {
    let rest = DAY - sim.now().minutes().rem_euclid(DAY) + 1;
    sim.advance_minutes(rest).expect("advances");
}

/// The patch off the water whose lowest ground is nearest the hearth, and that cell.
fn spring_site(sim: &Sim) -> (usize, u32) {
    let (map, land) = (sim.map(), sim.land());
    let hearth = land.settlements[0].hearth_m;
    let aq = &land.water.aquifer;
    (0..aq.len())
        .filter(|&p| aq.stage[p].is_none())
        .filter_map(|p| aq.seep[p].map(|(cell, _)| (p, cell)))
        .min_by(|a, b| {
            let d = |c: u32| {
                let (x, y) = civ_agents::population::cell_centre(map, c as usize);
                (x - hearth.0).hypot(y - hearth.1)
            };
            d(a.1).total_cmp(&d(b.1)).then(a.0.cmp(&b.0))
        })
        .expect("a patch off the water")
}

/// Sets the heads round patch `p` so a spring rises there giving `m3` a day, or, with `None`, so
/// its head stands a metre below its lowest ground and nothing rises: its head at its lowest
/// ground, every neighbour's no higher, and one neighbour's high enough to give it `m3`.
fn set_spring(sim: &mut Sim, p: usize, m3: Option<f64>) {
    let water = &mut sim.land_mut_for_tests().water;
    let aq = &water.aquifer;
    let cols = aq.cols as usize;
    let z = f64::from(aq.seep[p].expect("lowest ground").1);
    let (x, y) = (p % cols, p / cols);
    let mut round = Vec::new();
    if x + 1 < cols {
        round.push((p + 1, aq.c_east[p]));
    }
    if x > 0 {
        round.push((p - 1, aq.c_east[p - 1]));
    }
    if y + 1 < aq.rows as usize {
        round.push((p + cols, aq.c_south[p]));
    }
    if y > 0 {
        round.push((p - cols, aq.c_south[p - cols]));
    }
    let (q, c) = round
        .iter()
        .copied()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("a neighbour");
    assert!(c > 0.0, "the ground passes water");
    for &(n, _) in &round {
        water.heads[n] = water.heads[n].min(z);
    }
    match m3 {
        Some(m3) => {
            water.heads[p] = z;
            water.heads[q] = z + m3 / c;
        }
        None => water.heads[p] = z - 1.0,
    }
}

/// Loads of water drawn today, logged at the source `cell` and at any other.
fn draws(sim: &Sim, cell: u32) -> (usize, usize) {
    let today = &sim.people().uses.today;
    let at = |w: &&civ_agents::uses::Worked| w.place == Place::Source(cell);
    let here = today.iter().filter(at).count();
    let elsewhere = today
        .iter()
        .filter(|w| matches!(w.place, Place::Source(c) if c != cell))
        .count();
    (here, elsewhere)
}

/// A spring nearer than the river gives what flows to it in a day and no more, and then people go
/// to the river. The village wants more water today than the spring gives: its households' water
/// is set so what they fetch to end the day with their days of it is more than the spring flows.
/// The scene once left the households' water as their days had left it, and whether anyone went
/// to the river after the spring was drawn came down to whether one ran short that evening (§9,
/// "A home site is planned afresh at every call"). A world saved as the day begins goes on as the
/// unsaved one does.
#[test]
fn a_spring_nearer_than_the_river_gives_its_days_water_and_then_people_go_to_the_river() {
    a_spring_drawn_then_the_river(village(6));
}

/// [`a_spring_nearer_than_the_river_gives_its_days_water_and_then_people_go_to_the_river`] in
/// `sim` (of any identity, [`village_of`]).
fn a_spring_drawn_then_the_river(mut sim: Sim) {
    to_morning(&mut sim);
    let (p, cell) = spring_site(&sim);
    // The least a spring flows to be a source: 500 L, a little over 33 loads of 15 L.
    let flow = sim.rules().land.water.spring_min_m3_day;
    set_spring(&mut sim, p, Some(flow));
    let day = sim.now().day_index();
    let spring = sim
        .land()
        .water
        .spring_at(&sim.rules().land.water, p)
        .expect("a spring rises");
    assert!((spring.flow_m3_day - flow).abs() < 1e-9);
    // The households' water stands as though each had last fetched at another hour: the k-th of
    // n holds k / n of the days of water it keeps, so they come for more one after another
    // through the day rather than all at once.
    let params = sim.rules().people.household.clone();
    let load = params.carry_water_l;
    let now = sim.now();
    let n = sim.people().households.len() as f64;
    let mut wanted = 0.0;
    for (k, (_, h)) in sim.people_mut_for_tests().households.iter_mut().enumerate() {
        let day_l = h.members.len() as f64 * h.water_use(&params);
        let keep = day_l * params.water_target_days;
        h.water_l = keep * k as f64 / n;
        h.water_at = now;
        // What it fetches today to end the day with its days of water: what it lacks of them
        // now and the day's use.
        wanted += keep - h.water_l + day_l;
    }
    // What the scene stands on: the spring is where every household would fetch from, nearer
    // than the river, and the water they fetch today is more than the spring gives with five
    // loads over.
    let homes: Vec<_> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| h.id)
        .collect();
    assert!(!homes.is_empty());
    for &h in &homes {
        let at = sim.with_ctx_for_tests(|pop, ctx| pop.water_source_for_tests(ctx, h, true));
        assert_eq!(
            at.map(|o| o.cell),
            Some(cell),
            "household {h} fetches from the spring"
        );
    }
    assert!(
        wanted > flow * 1000.0 + 5.0 * load,
        "{wanted} L wanted, the spring giving {flow} m³"
    );
    let mut loaded = saves_alike(&mut sim);
    for s in [&mut sim, &mut loaded] {
        s.advance_minutes(DAY - 60).expect("advances");
    }
    for sim in [&sim, &loaded] {
        let (here, elsewhere) = draws(sim, cell);
        let drawn = sim.people().spring_draws.drawn(day, p as u32);
        assert!(here >= 30, "only {here} loads were drawn at the spring");
        assert!((drawn - here as f64 * load).abs() < 1e-6);
        // Nobody sets off for it once a load is not left, though some on their way may find
        // less.
        assert!(
            drawn <= flow * 1000.0 + 5.0 * load,
            "{drawn} L drawn of {flow} m³"
        );
        assert!(
            elsewhere > 0,
            "once the spring was drawn nobody went to the river"
        );
        // What households use follows the walk to the water they fetched: the profile's 20 L,
        // the spring and the river both being near.
        let households = &sim.people().households;
        assert!(households.iter().any(|(_, h)| h.water_use_l == 20.0));
        assert!(
            households
                .iter()
                .all(|(_, h)| h.water_use_l == 0.0 || h.water_use_l == 20.0)
        );
    }
    let (a, b) = (
        persist::encode_sections(&sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
    // Where they would fetch now is not the spring, drawn for the day.
    for &h in &homes {
        let at = sim.with_ctx_for_tests(|pop, ctx| pop.water_source_for_tests(ctx, h, true));
        assert!(
            at.is_some_and(|o| o.cell != cell),
            "household {h} would go to the spring still"
        );
    }
    // The next day the spring gives again.
    to_morning(&mut sim);
    assert_eq!(sim.people().spring_draws.drawn(day + 1, p as u32), 0.0);
}

/// A spring counts the loads promised to those on their way as well as those drawn (§9, "A spring
/// counts the loads promised as well as drawn"): with every household's water spent at once, the
/// village of [`a_spring_nearer_than_the_river_gives_its_days_water_and_then_people_go_to_the_river`]
/// sets out together at first light, and the spring still gives no more than flows to it, the rest
/// going to the river. Whether a load is left was once looked for when one set off, against what
/// had been drawn and not what those already on the way would draw: this scene drew 46 loads,
/// 690 L of 500 L. A world saved in the rush, with people on their way, goes on as the unsaved one
/// does.
#[test]
fn a_spring_gives_no_more_than_it_flows_when_everyone_sets_out_for_it_at_once() {
    a_spring_rushed(village(6));
}

/// [`a_spring_gives_no_more_than_it_flows_when_everyone_sets_out_for_it_at_once`] in `sim`.
fn a_spring_rushed(mut sim: Sim) {
    to_morning(&mut sim);
    let (p, cell) = spring_site(&sim);
    let flow = sim.rules().land.water.spring_min_m3_day;
    set_spring(&mut sim, p, Some(flow));
    let day = sim.now().day_index();
    let start = sim.now().minutes();
    let params = sim.rules().people.household.clone();
    let load = params.carry_water_l;
    let now = sim.now();
    // Every household's water spent at once: what each fetches today to end the day with its
    // days of water is all of them and the day's use.
    let mut wanted = 0.0;
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        h.water_l = 0.0;
        h.water_at = now;
        let day_l = h.members.len() as f64 * h.water_use(&params);
        wanted += day_l * params.water_target_days + day_l;
    }
    // Every household fetches from the spring.
    let homes: Vec<_> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| h.id)
        .collect();
    assert!(
        wanted > flow * 1000.0 + 5.0 * load,
        "{wanted} L wanted, the spring giving {flow} m³"
    );
    for &h in &homes {
        let at = sim.with_ctx_for_tests(|pop, ctx| pop.water_source_for_tests(ctx, h, true));
        assert_eq!(
            at.map(|o| o.cell),
            Some(cell),
            "household {h} fetches from the spring"
        );
    }
    let on_the_way = |sim: &Sim| on_the_way(sim, cell);
    // Into the rush: loads drawn and more on the way. At no minute is more drawn and promised
    // than flows.
    let mut most = 0;
    loop {
        sim.advance_minutes(1).expect("advances");
        assert!(
            sim.now().minutes() - start < DAY - 60,
            "no rush to the spring"
        );
        let drawn = sim.people().spring_draws.drawn(day, p as u32);
        let promised = on_the_way(&sim);
        most = most.max(promised);
        assert!(
            drawn + promised as f64 * load <= flow * 1000.0 + 1e-6,
            "{drawn} L drawn and {promised} loads promised of {flow} m³"
        );
        if drawn > 0.0 && promised >= 3 {
            break;
        }
    }
    let mut loaded = saves_alike(&mut sim);
    assert_eq!(on_the_way(&loaded), on_the_way(&sim));
    let rest = DAY - 60 - (sim.now().minutes() - start);
    for s in [&mut sim, &mut loaded] {
        for _ in 0..rest {
            s.advance_minutes(1).expect("advances");
            let drawn = s.people().spring_draws.drawn(day, p as u32);
            let promised = on_the_way(s);
            most = most.max(promised);
            assert!(
                drawn + promised as f64 * load <= flow * 1000.0 + 1e-6,
                "{drawn} L drawn and {promised} loads promised of {flow} m³"
            );
        }
    }
    assert!(most >= 3, "at most {most} on the way to the spring at once");
    for sim in [&sim, &loaded] {
        let (here, elsewhere) = draws(sim, cell);
        let drawn = sim.people().spring_draws.drawn(day, p as u32);
        assert!((drawn - here as f64 * load).abs() < 1e-6);
        assert!(here >= 30, "only {here} loads were drawn at the spring");
        assert!(drawn <= flow * 1000.0, "{drawn} L drawn of {flow} m³");
        assert!(
            elsewhere > 0,
            "once the spring was drawn nobody went to the river"
        );
        assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
        assert!(sim.people().uses.problems().is_empty());
    }
    let (a, b) = (
        persist::encode_sections(&sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
    // The next day the spring gives again.
    to_morning(&mut sim);
    assert_eq!(sim.people().spring_draws.drawn(day + 1, p as u32), 0.0);
}

/// A household-resolved settlement's day (ADR-0032 §2) asks of each of its trips for water whether
/// the spring has a load left, as one setting off in a Detailed day does (§9, "A spring counts the
/// loads promised as well as drawn"). A band of 60 in the world of the spring scenes fetches from
/// the spring that flows there, 614 L a day, nearer than the river; household-resolved, its
/// households' water spent the evening before, it lives its day at midnight drawing no more than
/// flows to the spring, the rest from the river. The day once repeated each household's first
/// trip of the morning until its water was made up, so the household whose trips the spring ran
/// out in drew past it. A world saved the evening before goes on, set household-resolved again
/// (a resolution is never saved), as the unsaved one does through two household-resolved days.
#[test]
fn a_household_resolved_day_draws_no_more_than_a_spring_flows() {
    a_spring_resolved([7; 16]);
}

/// [`a_household_resolved_day_draws_no_more_than_a_spring_flows`] in the world of identity
/// `identity`.
fn a_spring_resolved(identity: [u8; 16]) {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Spring".to_owned(),
            seed: 6,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 60,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        identity,
    )
    .expect("generates");
    to_morning(&mut sim);
    // The spring every household fetches from.
    let homes: Vec<_> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| h.id)
        .collect();
    let at = |sim: &mut Sim, h| {
        sim.with_ctx_for_tests(|pop, ctx| pop.water_source_for_tests(ctx, h, true))
            .map(|o| o.cell)
    };
    let cell = at(&mut sim, homes[0]).expect("a source of water");
    let aq = &sim.land().water.aquifer;
    let p = (0..aq.len())
        .find(|&p| aq.stage[p].is_none() && aq.seep[p].is_some_and(|(c, _)| c == cell))
        .expect("the households fetch from a spring");
    for &h in &homes {
        assert_eq!(
            at(&mut sim, h),
            Some(cell),
            "household {h} fetches from the spring"
        );
    }
    let settlement = sim.land().settlements[0].id;
    sim.set_household_resolved([settlement]);
    // To the last minutes of the evening, when nobody fetches water: every household's water
    // spent, so the day it lives at midnight is a day of fetching.
    sim.advance_minutes(DAY - 2).expect("advances");
    let params = sim.rules().people.household.clone();
    let load = params.carry_water_l;
    let now = sim.now();
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        h.water_l = 0.0;
        h.water_at = now;
    }
    let mut loaded = saves_alike(&mut sim);
    loaded.set_household_resolved([settlement]);
    for day in 0..2 {
        for s in [&mut sim, &mut loaded] {
            to_morning(s);
            assert!(s.people().resolution_problems().is_empty());
            assert_eq!(
                s.household_resolved().iter().collect::<Vec<_>>(),
                [&settlement]
            );
            // The day is lived: what flows to the spring today and what was drawn there.
            let today = s.now().day_index();
            let flow = s
                .land()
                .water
                .spring_at(&s.rules().land.water, p)
                .map_or(0.0, |x| x.flow_m3_day * 1000.0);
            let wanted: f64 = s
                .people()
                .households
                .iter()
                .map(|(_, h)| h.members.len() as f64 * h.water_use(&params))
                .sum();
            assert!(wanted > flow + 5.0 * load, "{wanted} L wanted of {flow} L");
            let drawn = s.people().spring_draws.drawn(today, p as u32);
            let (here, elsewhere) = draws(s, cell);
            assert!((drawn - here as f64 * load).abs() < 1e-6);
            // A load short of its flow at most is left at the spring, and the rest of the water
            // the village wants comes from the river.
            assert!(
                drawn <= flow && drawn > flow - load,
                "day {day}: {drawn} L drawn of {flow} L"
            );
            assert!(elsewhere > 0, "day {day}: nobody went to the river");
            assert!(s.people().problems(u64::MAX, usize::MAX).is_empty());
            assert!(s.people().uses.problems().is_empty());
        }
        let (a, b) = (
            persist::encode_sections(&sim),
            persist::encode_sections(&loaded),
        );
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "day {day}: section `{}` differs", x.tag);
        }
    }
}

/// How many are on their way to draw a load at `cell`, their draw not yet done: on a trip for
/// water aimed at it before its work step ends, or for a fire before its filling (its first work
/// step) ends. Read from people's activities, as the engine's own count is.
fn on_the_way(sim: &Sim, cell: u32) -> usize {
    use civ_agents::params::Behavior;
    use civ_agents::person::{Step, Target};
    let activities = &sim.rules().catalog.activities;
    sim.people()
        .people
        .iter()
        .filter(|(_, x)| {
            let work_ahead = |until: usize| {
                x.act.steps.iter().enumerate().any(|(i, s)| {
                    i >= x.act.step as usize && i < until && matches!(s, Step::Work { .. })
                })
            };
            match (
                activities.get(x.act.def as usize).map(|a| a.behavior),
                x.act.target,
            ) {
                (Some(Behavior::FetchWater), Target::Water(c)) => {
                    c == cell && work_ahead(usize::MAX)
                }
                (Some(Behavior::Douse), Target::Douse { from, .. }) => {
                    from == cell && work_ahead(2)
                }
                _ => false,
            }
        })
        .count()
}

/// Saves `sim` and loads it: every section of the loaded world is the same.
fn saves_alike(sim: &mut Sim) -> Sim {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "spring").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let (a, b) = (
        persist::encode_sections(sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
    loaded
}

#[test]
fn a_spring_that_does_not_flow_is_left_for_the_river() {
    let mut sim = village(6);
    to_morning(&mut sim);
    let (p, cell) = spring_site(&sim);
    set_spring(&mut sim, p, None);
    assert!(
        sim.land()
            .water
            .spring_at(&sim.rules().land.water, p)
            .is_none()
    );
    sim.advance_minutes(DAY - 60).expect("advances");
    let (here, elsewhere) = draws(&sim, cell);
    assert_eq!(here, 0);
    assert!(elsewhere > 0, "nobody fetched water");
}

#[test]
fn draws_at_a_spring_save_load_and_go_on_alike() {
    let mut sim = village(6);
    to_morning(&mut sim);
    let (p, _) = spring_site(&sim);
    let flow = sim.rules().land.water.spring_min_m3_day;
    set_spring(&mut sim, p, Some(flow));
    sim.advance_minutes(10 * 60).expect("advances");
    let day = sim.now().day_index();
    assert!(sim.people().spring_draws.drawn(day, p as u32) > 0.0);
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "spring",
    )
    .expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(&sim, &loaded);
    sim.advance_minutes(DAY).expect("advances");
    loaded.advance_minutes(DAY).expect("advances");
    same(&sim, &loaded);
}
