//! Bottlenecks for walkers (M8b slice CN; ADR-0029 §4): a village inside a whole palisade passes
//! its open gates first come; at their own pace (a walker every 0.7 s through each way's files of
//! a 2.5 m gate) a village of 30 waits a fraction of a second a passage and never a minute, and
//! made slow enough to queue, its walkers wait at
//! the approach, their trips are re-timed as new revisions, their households remember the waits,
//! and everyone still arrives. It all saves and goes on alike.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::traffic::Facility;
use civ_content::ContentRegistry;
use civ_land::enclosures::{Enclosure, ring};
use civ_schema::{flatbuffers, wire};
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
    Sim::create_for_tests(
        &NewWorld {
            name: "Gates".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

/// A whole palisade 80 m round the first settlement's hearth with the content's gates, put in
/// the world and laid on the walking grid.
fn enclose(sim: &mut Sim) -> Enclosure {
    let s = &sim.land().settlements[0];
    let (settlement, hearth) = (s.id, s.hearth_m);
    let ep = sim.rules().people.enclosures.clone();
    let sectors = ring(
        hearth,
        80.0,
        ep.sector_m as f32,
        ep.gates as usize,
        0.0,
        ep.palisade_h_per_m as f32,
        sim.map().cell_size_m,
        sim.map().width,
        sim.map().height,
    );
    let id = sim.allocate_id_for_tests();
    let mut e = Enclosure {
        id,
        owner: settlement,
        law: settlement,
        settlement,
        centre: hearth,
        radius_m: 80.0,
        sectors,
        work_h: 0.0,
        shares: Vec::new(),
        crew: (0, 0),
        begun: sim.now(),
        finished: Some(sim.now().day_index()),
        upkeep_h: 0.0,
    };
    e.work_h = e.labour_h();
    let land = sim.land_mut_for_tests();
    land.enclosures.list = vec![e.clone()];
    land.enclosures.changed();
    sim.lay_decks_for_tests();
    e
}

/// Passages and minutes waited at every facility, as remembered today.
fn tallied(sim: &Sim) -> (f32, f32) {
    let day = sim.now().day_index();
    sim.people()
        .traffic
        .tallies
        .iter()
        .map(|t| t.faded(day).totals())
        .fold((0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1))
}

/// Saves `sim`, loads it, and checks every section is the same and that the two go on alike for
/// `minutes`, with the same test headway (which is not saved) on both.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "gates").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    loaded.people_mut_for_tests().traffic.test_headway_s = sim.people().traffic.test_headway_s;
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(sim, &loaded);
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}

#[test]
fn a_village_passes_its_gates_without_waiting_and_made_slow_its_walkers_queue_and_arrive() {
    let mut sim = village(1);
    let e = enclose(&mut sim);
    assert!(
        e.sectors.iter().any(|s| s.gate.is_some()),
        "the line has gates"
    );
    // At the gates' own pace.
    sim.advance_minutes(3 * DAY).expect("lives");
    let (passed, waited) = tallied(&sim);
    assert!(
        passed > 20.0,
        "people went out and in by the gates: {passed}"
    );
    assert!(
        sim.people()
            .traffic
            .tallies
            .iter()
            .all(|t| matches!(t.facility, Facility::Gate { enclosure, .. } if enclosure == e.id)),
        "only the gates are facilities here"
    );
    // Those who set off together step through one behind another, a fraction of a second each;
    // nobody waits a minute, so no trip is re-timed and no household holds a wait.
    assert!(
        waited * 60.0 / passed < 1.0,
        "a village of 30 hardly waits at a 2.5 m gate: {waited} minutes over {passed} passages"
    );
    assert!(sim.people().traffic.waits.is_empty());

    // Made slow: a walker every five minutes each way.
    let living = sim.people().living();
    sim.people_mut_for_tests().traffic.test_headway_s = Some(300.0);
    let mut revised = 0;
    for _ in 0..(2 * 24) {
        sim.advance_minutes(60).expect("lives");
        revised += sim
            .people()
            .people
            .iter()
            .filter(|(_, p)| p.trip.as_ref().is_some_and(|t| t.rev > 0))
            .count();
    }
    let (_, waited) = tallied(&sim);
    assert!(
        waited > 30.0,
        "walkers waited at the gates: {waited} minutes"
    );
    assert!(revised > 0, "waiting walkers' trips were re-timed");
    let day = sim.now().day_index();
    let half = sim.rules().people.places.use_half_life_days;
    let traffic = &sim.people().traffic;
    let held: f32 = traffic
        .waits
        .keys()
        .flat_map(|&h| traffic.held(h, day, half))
        .map(|w| w.minutes)
        .sum();
    assert!(held > 30.0, "households remember their waits: {held}");
    // Nobody vanished in the queue, and nobody is stuck in it.
    assert_eq!(sim.people().living(), living);
    let now = sim.now();
    assert!(
        sim.people()
            .people
            .iter()
            .all(|(_, p)| p.act.step_ends >= now),
        "every walk still ends"
    );
    assert!(sim.people().traffic.problems().is_empty());
    // Waits of a quarter of an hour or more are told as incidents.
    let long = &sim.people().traffic.long_waits;
    assert!(!long.is_empty(), "some waited a quarter of an hour");
    assert!(long.iter().all(|w| w.minutes >= 15.0));
    let payload = civ_sim::frames::incidents::incidents_response(
        &sim,
        None,
        civ_sim::frames::incidents::WAIT,
        None,
        0,
    );
    let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
    let rows = response
        .body_as_incidents()
        .and_then(|i| i.rows())
        .expect("rows");
    assert!(!rows.is_empty());
    let words = rows.get(0).happened().unwrap_or_default().to_owned();
    assert!(
        words.starts_with("waited ") && words.contains("a gate of"),
        "{words}"
    );
    // The Traffic lens: the gates, busiest first, with their passages and waits by hour.
    let payload = civ_sim::frames::traffic::lens_response(&sim, None);
    let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
    let lens = response.body_as_coverage().expect("coverage");
    assert_eq!(lens.lens(), civ_sim::frames::traffic::TRAFFIC);
    let places = lens.places().expect("places");
    assert!(!places.is_empty());
    let first = places.get(0);
    assert_eq!(first.hour_passages().map_or(0, |v| v.len()), 24);
    assert!(first.passages() > 0.0 && first.waited_min() > 0.0);
    assert!(
        places
            .iter()
            .zip(places.iter().skip(1))
            .all(|(a, b)| a.passages() >= b.passages())
    );
    let summary = lens.summary().expect("summary").get(0);
    assert!(summary.contains("crossings and gates passed"), "{summary}");
    saves_and_goes_on_alike(&mut sim, DAY / 2);
}

#[test]
fn a_bands_days_of_travel_are_counted_by_occupation_and_purpose() {
    // M8b slice CR, first part: each resident's day counted at midnight in its band, the trips
    // they set off on counted by purpose and mode.
    use civ_agents::travel::{BANDS, Mode, Occupation};
    let mut sim = village(3);
    let start = sim.now().minutes();
    let to_midnight = 24 * 60 - start.rem_euclid(24 * 60);
    sim.advance_minutes(to_midnight + 3 * 24 * 60)
        .expect("lives");
    let settlement = sim.land().settlements[0].id;
    let travel = &sim.people().travel;
    let years: Vec<_> = travel
        .years
        .iter()
        .filter(|((_, s), _)| *s == settlement)
        .map(|(_, y)| y)
        .collect();
    assert!(!years.is_empty());
    let days: u32 = years.iter().flat_map(|y| y.days.iter().flatten()).sum();
    // Four midnights passed: the first day's, partly lived, and three whole days; each counts
    // everyone living in the settlement then, about thirty people.
    assert!((4 * 20..=4 * 40).contains(&days), "{days} person-days");
    let walked: u32 = years
        .iter()
        .map(|y| y.modes[Mode::Walk as usize].trips)
        .sum();
    assert!(walked > 30, "{walked} trips on foot");
    assert!(years.iter().any(|y| !y.purposes.is_empty()));
    // Someone of working age travelled a fair part of a day, and the young are counted apart.
    let adults: u32 = years
        .iter()
        .flat_map(|y| y.days[Occupation::Other as usize][1..BANDS].iter())
        .chain(
            years
                .iter()
                .flat_map(|y| y.days[Occupation::Fields as usize][1..BANDS].iter()),
        )
        .sum();
    assert!(adults > 0);
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
}

#[test]
fn where_people_go_from_home_is_held_by_place_and_finds_their_fields_and_the_hearth() {
    // M8c slice CS, first part: each walk out of a home's door is a trip to the place it goes,
    // held by the walker, fading; nothing reads it yet. In a village's first days its people go
    // from home to their own fields and to the hearth.
    let mut sim = village(3);
    sim.advance_minutes(5 * DAY).expect("lives");
    let (w, cs) = (sim.map().width, sim.map().cell_size_m);
    let hearth = sim.land().settlements[0].hearth_m;
    let people = sim.people();
    let travel = &people.travel;
    assert!(
        travel.goings.len() >= 15,
        "{} hold goings",
        travel.goings.len()
    );
    let near_rect = |c: (f32, f32), r: &civ_land::RectCm| {
        let (x, y) = ((c.0 * 100.0) as i32, (c.1 * 100.0) as i32);
        let dx = (r.x - x).max(x - (r.x + r.w)).max(0);
        let dy = (r.y - y).max(y - (r.y + r.h)).max(0);
        (dx as f32).hypot(dy as f32) <= 1600.0
    };
    let mut to_hearth = 0;
    let mut farmers = std::collections::BTreeSet::new();
    let mut to_fields = std::collections::BTreeSet::new();
    for (p, g) in &travel.goings {
        let person = people.person(*p).expect("a living walker");
        let household = person.household;
        let fields: Vec<_> = sim
            .land()
            .fields
            .iter()
            .filter(|f| f.household == household)
            .map(|f| f.rect)
            .collect();
        if !fields.is_empty() {
            farmers.insert(household);
        }
        let centres: Vec<_> =
            g.to.iter()
                .map(|&(place, _)| civ_agents::travel::going_centre(place, w, cs))
                .collect();
        to_hearth += usize::from(
            centres
                .iter()
                .any(|c| (c.0 - hearth.0).hypot(c.1 - hearth.1) <= 16.0),
        );
        if centres
            .iter()
            .any(|&c| fields.iter().any(|r| near_rect(c, r)))
        {
            to_fields.insert(household);
        }
    }
    assert!(
        2 * to_hearth >= travel.goings.len(),
        "{to_hearth} of {} went to the hearth",
        travel.goings.len()
    );
    assert!(!farmers.is_empty());
    assert_eq!(
        to_fields, farmers,
        "every farming household went to its fields"
    );
    assert!(people.problems(u64::MAX, usize::MAX).is_empty());
    saves_and_goes_on_alike(&mut sim, DAY / 2);
}

#[test]
fn a_household_builds_its_home_where_its_people_s_walks_cost_least_and_keeps_why() {
    // M8c slice CS, second part (ADR-0030 §1): a household about to build a home weighs plots
    // about its old point and toward where its people go, in hours of walking over the content's
    // years and levelling, and takes the cheapest; the plot keeps the receipt. Its people having
    // gone nowhere, it seeks about its old point alone, as before.
    let mut sim = village(3);
    sim.advance_minutes(2 * DAY).expect("lives");
    let mut farmers: Vec<_> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.members.len() >= 2)
        .filter_map(|(_, x)| {
            let f = sim.land().fields.iter().find(|f| f.household == x.id)?;
            let c = (
                (f.rect.x + f.rect.w / 2) as f32 / 100.0,
                (f.rect.y + f.rect.h / 2) as f32 / 100.0,
            );
            Some((x.id, x.home, c, x.members.clone()))
        })
        .collect();
    farmers.sort_by_key(|f| f.0);
    let (household, home, field, members) = farmers.first().cloned().expect("a farming household");
    let d = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).hypot(a.1 - b.1);
    assert!(d(home, field) > 100.0, "its field lies off from home");
    for m in &members {
        sim.people_mut_for_tests().travel.goings.remove(m);
    }
    let site = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.home_site_for_tests(ctx, household))
            .expect("clear ground")
    };
    let (before, receipt) = site(&mut sim);
    let receipt = receipt.expect("a receipt");
    assert_eq!(receipt.weighed, 1);
    assert_eq!(receipt.hours, receipt.next_hours);
    assert!(
        d(before, home) < 40.0,
        "about its old point: {before:?} {home:?}"
    );
    // Its people go to their field daily from home.
    let (w, day) = (sim.map().width, sim.now().minutes() / DAY);
    let cs = sim.map().cell_size_m;
    let cell = (field.1 / cs) as u32 * w + (field.0 / cs) as u32;
    let place = civ_agents::travel::going_place(cell, w);
    let half = sim.rules().people.places.use_half_life_days;
    for m in &members {
        for _ in 0..30 {
            sim.people_mut_for_tests().travel.went(*m, place);
        }
    }
    sim.people_mut_for_tests().travel.hold_goings(day, half);
    let (after, receipt) = site(&mut sim);
    let receipt = receipt.expect("a receipt");
    assert!(receipt.weighed > 1, "{receipt:?}");
    assert!(receipt.next_hours > receipt.hours, "{receipt:?}");
    assert!(
        d(after, field) + 50.0 < d(before, field),
        "nearer its field: {after:?} against {before:?}, the field at {field:?}"
    );
}

#[test]
fn a_plan_is_laid_three_to_eight_ways_over_clear_ground_kept_as_laid_and_versioned() {
    // M8c slice CT, second part (ADR-0030 §2): a sponsor lays the plan's grid at the hearth along
    // both axes and at the built edge to each side, over the ground as it is, and each household
    // weighs each layout; the one put forward is kept with the law as it was laid, a later plan
    // replacing it as its next version; both, and what the polity saw of its ground, survive a
    // save.
    use civ_agents::polity::{GroundSeen, IssueKind, LawStatus, Outcome, PolicyKind};
    let mut sim = village(3);
    sim.advance_minutes(2 * DAY).expect("lives");
    let settlement = sim.land().settlements[0].id;
    let hearth = sim.land().settlements[0].hearth_m;
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    let plans = sim.with_ctx_for_tests(|pop, ctx| pop.plans_for_tests(ctx, settlement));
    assert!((3..=8).contains(&plans.len()), "{}", plans.len());
    let at_hearth = plans.iter().filter(|p| p.0.origin == hearth).count();
    assert_eq!(at_hearth, 2, "both axes at the hearth");
    let households = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .count();
    for (layout, gains) in &plans {
        assert!(!layout.parcels.is_empty());
        // Over clear ground only: no parcel, street or square on a plot or a field.
        let claims: Vec<_> = sim
            .land()
            .plots
            .iter()
            .map(|p| p.rect)
            .chain(sim.land().fields.iter().map(|f| f.rect))
            .collect();
        let laid = layout
            .parcels
            .iter()
            .map(|p| p.rect)
            .chain(layout.streets.iter().copied())
            .chain(layout.sites.iter().map(|s| s.rect));
        for r in laid {
            assert!(claims.iter().all(|c| !c.near(&r, 0)), "{r:?} on a claim");
        }
        // Every household weighs it and owes the same share of the staking.
        assert_eq!(gains.len(), households);
        let owed = gains[0].2;
        assert!(owed > 0.0);
        assert!(
            gains
                .iter()
                .all(|g| (g.2 - owed).abs() < 1e-9 && g.1 >= 0.0)
        );
    }
    // What the polity saw of its ground raises the plan's issues.
    let day = sim.now().minutes() / DAY;
    let household = sim
        .people()
        .households
        .iter()
        .find(|(_, x)| x.settlement == Some(settlement))
        .map(|(_, x)| x.id)
        .expect("a household");
    sim.people_mut_for_tests().polities[pi].see_ground(GroundSeen {
        day,
        settlement,
        household,
        issue: IssueKind::NoGround,
    });
    // Someone puts the first forward: the law keeps the layout as laid, the polity's first plan.
    let sponsor = sim
        .people()
        .household(household)
        .and_then(|x| x.members.first().copied())
        .expect("a sponsor");
    let first = sim
        .with_ctx_for_tests(|pop, ctx| {
            pop.propose_plan_for_tests(ctx, settlement, sponsor, 0, IssueKind::NoGround)
        })
        .expect("a plan proposed");
    let polity = &sim.people().polities[pi];
    let laid = polity.plan_of(first).expect("its plan kept");
    assert_eq!(laid.version, 1);
    assert_eq!(laid.layout, plans[0].0);
    let law = polity.laws.iter().find(|l| l.id == first).expect("the law");
    assert_eq!(
        (law.kind, law.issue),
        (PolicyKind::Plan, IssueKind::NoGround)
    );
    let words = polity.words_of(law, &sim.rules().catalog.policies, &|_| String::new());
    assert!(words.contains("version 1"), "{words}");
    // Passed, it is in force; a later plan replaces it as the second version.
    {
        let pop = sim.people_mut_for_tests();
        let l = pop.polities[pi].law_mut(first).expect("the law");
        l.status = LawStatus::InForce;
        l.outcome = Some(Outcome::Passed);
        pop.polities[pi].gathering = None;
    }
    assert_eq!(sim.people().polities[pi].plans_in_force().count(), 1);
    let second = sim
        .with_ctx_for_tests(|pop, ctx| {
            pop.propose_plan_for_tests(ctx, settlement, sponsor, 1, IssueKind::NoGround)
        })
        .expect("a second plan proposed");
    let polity = &sim.people().polities[pi];
    assert_eq!(polity.plan_of(second).map(|p| p.version), Some(2));
    assert_eq!(
        polity
            .laws
            .iter()
            .find(|l| l.id == second)
            .and_then(|l| l.ends),
        Some(first)
    );
    assert!(!polity.ground_seen.is_empty());
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn a_plan_in_force_keeps_its_keepers_off_its_streets_and_records_what_is_built_on_them() {
    // M8c slice CT, third part (ADR-0030 §2): once a plan is in force and known, a household
    // whose eldest keeps it lays out its next home clear of the plan's streets and square; a
    // building begun on that ground is an encroachment, counted against the law and recorded,
    // and the record survives a save.
    use civ_agents::polity::{IssueKind, LawStatus, Outcome};
    use civ_core::PermanentId;
    let mut sim = village(3);
    sim.advance_minutes(2 * DAY).expect("lives");
    let settlement = sim.land().settlements[0].id;
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    let homes: Vec<(PermanentId, PermanentId)> = sim
        .land()
        .buildings
        .iter()
        .filter(|b| b.standing())
        .filter(|b| {
            sim.people()
                .household(b.household)
                .is_some_and(|x| x.settlement == Some(settlement) && !x.members.is_empty())
        })
        .map(|b| (b.id, b.household))
        .collect();
    let (_, household) = *homes.first().expect("a building");
    let sponsor = sim
        .people()
        .household(household)
        .and_then(|x| x.members.first().copied())
        .expect("a sponsor");
    let law = sim
        .with_ctx_for_tests(|pop, ctx| {
            pop.propose_plan_for_tests(ctx, settlement, sponsor, 0, IssueKind::NoGround)
        })
        .expect("a plan proposed");
    // Where each would lay out its next home without it.
    let plot_of = |sim: &mut Sim, h: PermanentId| {
        let spec = sim
            .with_ctx_for_tests(|pop, ctx| pop.first_home_for_tests(ctx, h))
            .expect("a home laid out");
        let catalog = &sim.rules().catalog;
        let def = catalog
            .building_index(&spec.program)
            .map(|i| &catalog.buildings[i])
            .expect("its program");
        civ_agents::build::plot_rect(&spec, def)
    };
    let unplanned: Vec<_> = homes
        .iter()
        .map(|h| (h.1, plot_of(&mut sim, h.1)))
        .collect();
    // Before it is in force nobody keeps it.
    assert!(homes.iter().all(|h| {
        sim.with_ctx_for_tests(|pop, ctx| pop.plans_kept_for_tests(ctx, h.1))
            .is_empty()
    }));
    let now = sim.now();
    {
        let pop = sim.people_mut_for_tests();
        let l = pop.polities[pi].law_mut(law).expect("the law");
        l.status = LawStatus::InForce;
        l.outcome = Some(Outcome::Passed);
        l.decided = Some(now);
        pop.polities[pi].gathering = None;
    }
    let everyone: Vec<PermanentId> = sim.people().people.iter().map(|(_, p)| p.id).collect();
    for p in everyone {
        sim.people_mut_for_tests().polities[pi]
            .law_mut(law)
            .expect("the law")
            .learn(p, 0);
    }
    // A keeper's next home stands clear of what the plan reserves: here a street laid where it
    // would have built without it.
    let keeper = homes
        .iter()
        .map(|h| h.1)
        .find(|&h| {
            !sim.with_ctx_for_tests(|pop, ctx| pop.plans_kept_for_tests(ctx, h))
                .is_empty()
        })
        .expect("someone keeps it");
    let before = unplanned
        .iter()
        .find(|u| u.0 == keeper)
        .map(|u| u.1)
        .expect("where it would build");
    {
        let polity = &mut sim.people_mut_for_tests().polities[pi];
        let laid = polity
            .plans
            .iter_mut()
            .find(|p| p.0 == law)
            .expect("its plan");
        laid.1.layout.streets.push(before);
    }
    let layout = sim.people().polities[pi]
        .plan_of(law)
        .expect("its plan")
        .layout
        .clone();
    assert_eq!(
        sim.with_ctx_for_tests(|pop, ctx| pop.plans_kept_for_tests(ctx, keeper)),
        vec![layout.clone()]
    );
    let rect = plot_of(&mut sim, keeper);
    assert_ne!(rect, before);
    assert!(!layout.reserves(&rect, 0), "{rect:?} on reserved ground");
    // A building begun where the plan lays a street is an encroachment: counted against the
    // law, by one who knew of it, and recorded.
    let street = *layout.streets.first().expect("a street");
    let (building, owner) = homes[0];
    {
        let land = sim.land_mut_for_tests();
        let plot = land
            .buildings
            .iter()
            .find(|b| b.id == building)
            .map(|b| b.plot)
            .expect("its plot");
        let p = land
            .plots
            .iter_mut()
            .find(|p| p.id == plot)
            .expect("the plot");
        p.rect.x = street.x;
        p.rect.y = street.y;
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.count_rule_keeping_for_tests(ctx, owner, building));
    let polity = &sim.people().polities[pi];
    let c = polity
        .laws
        .iter()
        .find(|l| l.id == law)
        .expect("the law")
        .compliance;
    assert_eq!((c.broken, c.broken_unaware, c.complied), (1, 0, 0), "{c:?}");
    let e = polity.encroachments.last().copied().expect("recorded");
    assert_eq!(
        (e.law, e.household, e.building, e.day),
        (law, owner, building, now.day_index())
    );
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn a_plan_s_keeper_gives_a_household_short_of_a_home_the_parcel_nearest_the_hearth() {
    // M8c slice CU, first part (ADR-0030 §4): the one who puts a plan forward keeps it; while
    // they live there, a household about to build a home is offered the free parcel nearest the
    // hearth and weighs it as any plot, in hours; one that builds there holds the parcel by need,
    // the keeper's staking is counted, and the next household is offered the next parcel. With
    // its keeper gone the plan stays in force and gives nothing.
    use civ_agents::polity::{IssueKind, LawStatus, Outcome};
    use civ_core::PermanentId;
    let mut sim = village(3);
    sim.advance_minutes(2 * DAY).expect("lives");
    let settlement = sim.land().settlements[0].id;
    let hearth = sim.land().settlements[0].hearth_m;
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    let homes: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|(_, x)| x.id)
        .collect();
    let sponsor = sim
        .people()
        .household(homes[0])
        .map(|x| x.members[0])
        .expect("a sponsor");
    let law = sim
        .with_ctx_for_tests(|pop, ctx| {
            pop.propose_plan_for_tests(ctx, settlement, sponsor, 0, IssueKind::NoGround)
        })
        .expect("a plan put forward");
    // Put forward, it gives nothing; its sponsor would keep it.
    assert_eq!(
        sim.people().polities[pi]
            .laws
            .iter()
            .find(|l| l.id == law)
            .and_then(|l| l.holder),
        Some(sponsor)
    );
    let household = homes[1];
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.parcel_offer_for_tests(ctx, household))
            .is_none()
    );
    let now = sim.now();
    {
        let pop = sim.people_mut_for_tests();
        let l = pop.polities[pi].law_mut(law).expect("the law");
        l.status = LawStatus::InForce;
        l.outcome = Some(Outcome::Passed);
        l.decided = Some(now);
        pop.polities[pi].gathering = None;
    }
    // In force, the free parcel nearest the hearth is offered.
    let offered = sim
        .with_ctx_for_tests(|pop, ctx| pop.parcel_offer_for_tests(ctx, household))
        .expect("a parcel offered");
    assert_eq!(
        (offered.law, offered.rule, offered.fee),
        (law, civ_land::AllotRule::Need, None)
    );
    let (parcel, ground) = (offered.parcel, offered.ground);
    let layout = sim.people().polities[pi]
        .plan_of(law)
        .expect("its plan")
        .layout
        .clone();
    let far = |r: &civ_land::RectCm| {
        let (x, y) = r.centre_m();
        (x - hearth.0).hypot(y - hearth.1)
    };
    assert_eq!(layout.parcels[usize::from(parcel)].rect, ground);
    assert!(
        layout
            .parcels
            .iter()
            .all(|q| far(&q.rect) >= far(&ground) - 1e-3)
    );
    // A household whose old point is on the parcel, whose people have gone nowhere yet and
    // which finds no other clear ground near (claims all round the parcel, as a crowded village
    // would have), builds its home on it.
    let (reach, gap) = (300_000, 300);
    let (x0, y0) = (ground.x - gap, ground.y - gap);
    let (w0, h0) = (ground.w + 2 * gap, ground.h + 2 * gap);
    let round = [
        (x0 - reach, y0 - reach, reach, h0 + 2 * reach),
        (x0 + w0, y0 - reach, reach, h0 + 2 * reach),
        (x0, y0 - reach, w0, reach),
        (x0, y0 + h0, w0, reach),
    ];
    let mut claims = Vec::new();
    for (x, y, w, h) in round {
        let id = sim.allocate_id_for_tests();
        claims.push(id);
        let since = sim.now();
        sim.land_mut_for_tests().plots.push(civ_land::Plot {
            id,
            household: homes[0],
            rect: civ_land::RectCm { x, y, w, h },
            use_: civ_land::PlotUse::Store,
            since,
            receipt: None,
            allotted: None,
        });
    }
    if let Some((_, x)) = sim
        .people_mut_for_tests()
        .households
        .iter_mut()
        .find(|(_, x)| x.id == household)
    {
        x.home = ground.centre_m();
    }
    let today = sim.now().day_index();
    let members = sim
        .people()
        .household(household)
        .expect("it")
        .members
        .clone();
    for m in members {
        sim.people_mut_for_tests().travel.goings.insert(
            m,
            civ_agents::travel::Goings {
                day: today,
                to: Vec::new(),
            },
        );
    }
    let (spec, given) = sim
        .with_ctx_for_tests(|pop, ctx| pop.first_home_parcel_for_tests(ctx, household))
        .expect("a home laid out");
    assert_eq!(
        given,
        Some(civ_land::Allotment {
            law,
            parcel,
            rule: civ_land::AllotRule::Need,
        })
    );
    let catalog = &sim.rules().catalog;
    let def = catalog
        .building_index(&spec.program)
        .map(|i| &catalog.buildings[i])
        .expect("its program");
    let rect = civ_agents::build::plot_rect(&spec, def);
    let (cx, cy) = (rect.x + rect.w / 2, rect.y + rect.h / 2);
    assert!(
        cx >= ground.x && cy >= ground.y && cx < ground.x + ground.w && cy < ground.y + ground.h,
        "{rect:?} on {ground:?}"
    );
    // Begun there, the plot holds the parcel and the keeper's staking is counted; the next
    // household is offered another.
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_home_for_tests(ctx, household))
        .expect("begun");
    let plot = sim
        .land()
        .plots
        .iter()
        .find(|p| p.household == household && p.allotted.is_some())
        .expect("its plot");
    assert_eq!(plot.allotted.map(|a| a.parcel), Some(parcel));
    // The building's line says where it stands, and how it was given.
    let begun = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.plot == plot.id)
        .map(|b| b.id)
        .expect("its building");
    let bytes = civ_sim::frames::buildings::buildings_response(&sim);
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let site = response
        .body_as_buildings()
        .and_then(|b| b.buildings())
        .and_then(|l| l.iter().find(|b| b.id() == begun.get()))
        .and_then(|b| b.site())
        .unwrap_or_default()
        .to_owned();
    assert!(
        site.ends_with(&format!(
            "on parcel {} of the plan, given by need",
            parcel + 1
        )),
        "{site}"
    );
    let staked = sim.people().polities[pi].plan_of(law).expect("it").staked_h;
    assert!(staked > 0.0);
    sim.land_mut_for_tests()
        .plots
        .retain(|p| !claims.contains(&p.id));
    let next = sim
        .with_ctx_for_tests(|pop, ctx| pop.parcel_offer_for_tests(ctx, homes[2]))
        .expect("another parcel");
    assert_ne!(next.parcel, parcel);
    // Saved and loaded alike, the parcel given and the staking with it.
    saves_and_goes_on_alike(&mut sim, DAY);
    // Its keeper gone, the plan stands, its office vacant, and gives nothing.
    let gone = sim.allocate_id_for_tests();
    sim.people_mut_for_tests().polities[pi]
        .law_mut(law)
        .expect("the law")
        .holder = Some(gone);
    sim.advance_minutes(DAY).expect("lives");
    let l = sim.people().polities[pi]
        .laws
        .iter()
        .find(|l| l.id == law)
        .expect("the law");
    assert_eq!((l.status, l.holder), (LawStatus::InForce, None));
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.parcel_offer_for_tests(ctx, homes[2]))
            .is_none()
    );
}

/// Claims all round `ground`, a gap of `gap_cm` from it, of household `owner`: a crowded
/// village's, leaving the ground the only clear ground near; their ids.
fn claims_round(
    sim: &mut Sim,
    ground: civ_land::RectCm,
    owner: civ_core::PermanentId,
) -> Vec<civ_core::PermanentId> {
    let (reach, gap) = (300_000, 300);
    let (x0, y0) = (ground.x - gap, ground.y - gap);
    let (w0, h0) = (ground.w + 2 * gap, ground.h + 2 * gap);
    let round = [
        (x0 - reach, y0 - reach, reach, h0 + 2 * reach),
        (x0 + w0, y0 - reach, reach, h0 + 2 * reach),
        (x0, y0 - reach, w0, reach),
        (x0, y0 + h0, w0, reach),
    ];
    let mut ids = Vec::new();
    for (x, y, w, h) in round {
        let id = sim.allocate_id_for_tests();
        ids.push(id);
        let since = sim.now();
        sim.land_mut_for_tests().plots.push(civ_land::Plot {
            id,
            household: owner,
            rect: civ_land::RectCm { x, y, w, h },
            use_: civ_land::PlotUse::Store,
            since,
            receipt: None,
            allotted: None,
        });
    }
    ids
}

#[test]
fn a_plan_s_parcels_go_by_lot_for_a_fee_or_by_its_keeper_s_regard() {
    // M8c slice CU, second part (ADR-0030 §4): a template names how its keeper gives parcels.
    // By lot, the parcel offered is drawn, the same draw today for the same household; for a
    // fee, only to a household holding it, which pays the store as it begins its home there;
    // by the keeper's choice, only to a household whose eldest the keeper regards.
    use civ_agents::polity::{IssueKind, LawStatus, Outcome, PolicyKind};
    use civ_core::PermanentId;
    use civ_land::AllotRule;
    let mut sim = village(3);
    sim.advance_minutes(2 * DAY).expect("lives");
    let settlement = sim.land().settlements[0].id;
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    let plan_policy = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.kind == PolicyKind::Plan)
        .expect("the plan template");
    let grain = sim
        .rules()
        .catalog
        .good_index("core:good/grain")
        .expect("grain");
    let set_terms =
        |sim: &mut Sim, allot: AllotRule, fee: Option<(usize, f32)>, rent: Option<(usize, f32)>| {
            let rules = sim.rules_mut_for_tests().expect("rules of its own");
            let plan = rules.catalog.policies[plan_policy]
                .plan
                .as_mut()
                .expect("a plan");
            plan.allot = allot;
            plan.fee = fee;
            plan.rent = rent;
        };
    let set_rule = |sim: &mut Sim, allot: AllotRule, fee: Option<(usize, f32)>| {
        set_terms(sim, allot, fee, None);
    };
    let homes: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|(_, x)| x.id)
        .collect();
    let keeper = sim
        .people()
        .household(homes[0])
        .map(|x| x.members[0])
        .expect("a sponsor");
    let law = sim
        .with_ctx_for_tests(|pop, ctx| {
            pop.propose_plan_for_tests(ctx, settlement, keeper, 0, IssueKind::NoGround)
        })
        .expect("a plan put forward");
    let now = sim.now();
    {
        let pop = sim.people_mut_for_tests();
        let l = pop.polities[pi].law_mut(law).expect("the law");
        l.status = LawStatus::InForce;
        l.outcome = Some(Outcome::Passed);
        l.decided = Some(now);
        pop.polities[pi].gathering = None;
    }
    let offer = |sim: &mut Sim, h: PermanentId| {
        sim.with_ctx_for_tests(|pop, ctx| pop.parcel_offer_for_tests(ctx, h))
    };
    // By lot: a draw, the same today for the same household.
    set_rule(&mut sim, AllotRule::Lot, None);
    let drawn = offer(&mut sim, homes[1]).expect("a parcel drawn");
    assert_eq!(drawn.rule, AllotRule::Lot);
    assert_eq!(offer(&mut sim, homes[1]), Some(drawn));
    // By the keeper's choice: only to a household whose eldest the keeper regards.
    set_rule(&mut sim, AllotRule::Standing, None);
    let (now, grown) = (sim.now(), sim.rules().people.family.independent_age);
    let mut regarded = 0;
    for &h in &homes[1..] {
        let members = sim.people().household(h).expect("it").members.clone();
        let eldest = members
            .iter()
            .copied()
            .find(|&m| {
                sim.people()
                    .person(m)
                    .is_some_and(|p| p.age_years(now) >= grown)
            })
            .or_else(|| members.first().copied())
            .expect("someone");
        let regard =
            sim.people()
                .ties
                .regard(keeper, eldest, now.day_index(), &sim.rules().people.ties);
        let given = offer(&mut sim, h);
        assert_eq!(given.is_some(), regard > 0.0, "{h:?} regarded {regard}");
        regarded += usize::from(regard > 0.0);
        if let Some(o) = given {
            assert_eq!(o.rule, AllotRule::Standing);
        }
    }
    assert!(
        regarded < homes.len() - 1,
        "someone the keeper does not regard"
    );
    // For a fee: only to a household holding it.
    set_rule(&mut sim, AllotRule::Offer, Some((grain, 2.0)));
    let household = homes[1];
    let held = |sim: &Sim, h: PermanentId| sim.people().household(h).expect("it").stores[grain];
    let set_held = |sim: &mut Sim, h: PermanentId, kg: f64| {
        if let Some((_, x)) = sim
            .people_mut_for_tests()
            .households
            .iter_mut()
            .find(|(_, x)| x.id == h)
        {
            x.stores[grain] = kg;
        }
    };
    set_held(&mut sim, household, 1.0);
    assert!(offer(&mut sim, household).is_none(), "short of the fee");
    set_held(&mut sim, household, 30.0);
    let o = offer(&mut sim, household).expect("a parcel for a fee");
    assert_eq!((o.rule, o.fee), (AllotRule::Offer, Some((grain, 2.0))));
    let polity = &sim.people().polities[pi];
    let words = polity.words_of(
        polity.laws.iter().find(|l| l.id == law).expect("the law"),
        &sim.rules().catalog.policies,
        &|_| String::new(),
    );
    assert!(words.contains("parcels given for a fee"), "{words}");
    // It builds there, paying the store as it begins.
    let store = sim.people().polities[pi].id;
    let in_store = |sim: &Sim| {
        sim.people().polities[pi]
            .stores
            .get(grain)
            .copied()
            .unwrap_or(0.0)
    };
    let before = in_store(&sim);
    if let Some((_, x)) = sim
        .people_mut_for_tests()
        .households
        .iter_mut()
        .find(|(_, x)| x.id == household)
    {
        x.home = o.ground.centre_m();
    }
    let members = sim
        .people()
        .household(household)
        .expect("it")
        .members
        .clone();
    let today = sim.now().day_index();
    for m in members {
        sim.people_mut_for_tests().travel.goings.insert(
            m,
            civ_agents::travel::Goings {
                day: today,
                to: Vec::new(),
            },
        );
    }
    let claims = claims_round(&mut sim, o.ground, homes[0]);
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_home_for_tests(ctx, household))
        .expect("begun");
    let plot = sim
        .land()
        .plots
        .iter()
        .find(|p| p.household == household && p.allotted.is_some())
        .expect("its plot");
    assert_eq!(
        plot.allotted.map(|a| (a.parcel, a.rule)),
        Some((o.parcel, AllotRule::Offer))
    );
    assert!(
        (held(&sim, household) - 28.0).abs() < 0.05,
        "{}",
        held(&sim, household)
    );
    assert!(
        (in_store(&sim) - before - 2.0).abs() < 1e-6,
        "{}",
        in_store(&sim)
    );
    assert!(store.get() > 0);
    sim.land_mut_for_tests()
        .plots
        .retain(|p| !claims.contains(&p.id));
    // For a ground rent alone, a household holding nothing of it is offered a parcel too
    // (M8c slice CU, third part).
    set_terms(&mut sim, AllotRule::Offer, None, Some((grain, 3.0)));
    set_held(&mut sim, homes[2], 0.0);
    let o = offer(&mut sim, homes[2]).expect("a parcel for a rent");
    assert_eq!((o.fee, o.rent), (None, Some((grain, 3.0))));
    let polity = &sim.people().polities[pi];
    let words = polity.words_of(
        polity.laws.iter().find(|l| l.id == law).expect("the law"),
        &sim.rules().catalog.policies,
        &|_| String::new(),
    );
    assert!(
        words.contains("parcels given for a yearly ground rent"),
        "{words}"
    );
    // A year after its plot was given, the household holding it owes the store a year's rent,
    // answered as a fine is.
    let kcal_per_kg = sim.rules().catalog.goods[grain].kcal_per_kg;
    let tomorrow = sim.now().day_index() + 1;
    if let Some(p) = sim
        .land_mut_for_tests()
        .plots
        .iter_mut()
        .find(|p| p.household == household && p.allotted.is_some())
    {
        p.since = civ_core::SimTime::from_minutes((tomorrow - 365) * 24 * 60);
    }
    set_held(&mut sim, household, 300.0);
    let rents = |sim: &Sim| -> Vec<civ_agents::crime::Obligation> {
        sim.people()
            .order
            .obligations
            .iter()
            .filter(|o| o.kind == civ_agents::crime::Owed::GroundRent)
            .copied()
            .collect()
    };
    assert!(rents(&sim).is_empty());
    let before = in_store(&sim);
    sim.advance_minutes(DAY).expect("lives");
    let owed = rents(&sim);
    assert_eq!(owed.len(), 1, "{owed:?}");
    let rent = owed[0];
    assert_eq!(
        (rent.debtor, rent.beneficiary, rent.case),
        (household, store, None)
    );
    assert!(
        (f64::from(rent.kcal) - 3.0 * kcal_per_kg).abs() < 1.0,
        "{rent:?}"
    );
    sim.advance_minutes(3 * DAY).expect("lives");
    let rent = rents(&sim)[0];
    assert_eq!(rent.standing, civ_agents::crime::Standing::Met, "{rent:?}");
    assert!(in_store(&sim) > before, "{} {before}", in_store(&sim));
    // Only once a year.
    assert_eq!(rents(&sim).len(), 1);
    saves_and_goes_on_alike(&mut sim, DAY);
}
