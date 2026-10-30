//! Refuse carried to a tip (M11f, second part; ADR-0038 §3): passing the law opens one tip at
//! least its distance from every open well, spring, stream and home, or none where no place
//! qualifies, and then nobody carries; under it a household carries its heap there, and the heap's
//! decaying share, its ash and its load of a disease arrive whole; what lies at a tip washes and
//! seeps toward the water by the same links as a heap at the same place, and none of it reaches a
//! well above it or beyond reach; a heap grown large beside another's home raises `heaps_near`;
//! without the law nobody tips; and a world with a tip saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::contagion::{How, Node};
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided, PolicyKind, Tip, TipDef};
use civ_agents::population::{WellWeighed, cell_of};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::wells::WellState;
use civ_sim::{NewWorld, Sim, persist};
use civ_world::{WATER_LAND, WATER_RIVER};

const PRESET: &str = "core:worldgen/river_valley";
const CHOLERA: &str = "core:disease/cholera";
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

/// A world of `seed` whose band has just settled, as the crime target's.
fn world_with(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Tips".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

/// A village of world `seed` with a fixed identity, as the contagion target's.
fn village(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Wellfield".to_owned(),
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

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "tips").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
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

/// The content's terms for a tip.
fn tip_def(sim: &Sim) -> TipDef {
    sim.rules()
        .catalog
        .policies
        .iter()
        .find_map(|d| d.tip)
        .expect("the core content carries refuse to a tip")
}

/// The index of cholera in the catalog.
fn cholera(sim: &Sim) -> u16 {
    let i = sim
        .rules()
        .catalog
        .disease_index(CHOLERA)
        .expect("cholera is content");
    u16::try_from(i).expect("few diseases")
}

/// A law of `kind` in force in the first polity, every adult of its settlement knowing it, as the
/// crime target's: its id.
fn a_law(sim: &mut Sim, kind: PolicyKind) -> PermanentId {
    let policies = &sim.rules().catalog.policies;
    let policy = policies
        .iter()
        .position(|d| d.kind == kind)
        .expect("the core content has the template");
    let id = sim.allocate_id_for_tests();
    let (now, grown) = (sim.now(), sim.rules().people.family.independent_age);
    let pop = sim.people_mut_for_tests();
    let settlement = pop.polities[0].settlement;
    let mut adults: Vec<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(settlement))
        .flat_map(|(_, h)| h.members.clone())
        .filter(|&m| pop.person(m).is_some_and(|p| p.age_years(now) >= grown))
        .collect();
    adults.sort_unstable();
    let mut law = Law {
        id,
        policy: policy as u16,
        kind,
        levy_share: 0.0,
        holder: None,
        relief_days: 0.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor: adults[0],
        proposed: now,
        issue: IssueKind::HeapsNear,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Decided::Passed),
        eligible: adults.len() as u32,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
        target: None,
    };
    for &a in &adults {
        law.learn(a, now.day_index());
    }
    pop.polities[0].laws.push(law);
    id
}

/// The refuse law passed in the first polity, its tip opened as passing it opens one: the law and
/// the tip, if one opened.
fn the_law_passed(sim: &mut Sim) -> (PermanentId, Option<Tip>) {
    let law = a_law(sim, PolicyKind::RefuseToTip);
    sim.with_ctx_for_tests(|pop, ctx| pop.open_tip_for_tests(ctx, 0, law));
    let tip = sim.people().polities[0]
        .tips
        .iter()
        .find(|t| t.law == law)
        .cloned();
    (law, tip)
}

/// Households of the first polity's settlement with people, by id.
fn our_households(sim: &Sim) -> Vec<PermanentId> {
    let settlement = sim.people().polities[0].settlement;
    let mut out: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|(_, x)| x.id)
        .collect();
    out.sort_unstable();
    out
}

/// The nearest open well, spring, stream cell and home of anyone to `at`, metres, each.
fn clearances(sim: &Sim, at: (f32, f32)) -> [f64; 4] {
    let (ax, ay) = (f64::from(at.0), f64::from(at.1));
    let dist = |p: (f32, f32)| (f64::from(p.0) - ax).hypot(f64::from(p.1) - ay);
    let map = sim.map();
    let size = f64::from(map.cell_size_m);
    let centre = |cell: usize| {
        let (x, y) = (cell % map.width as usize, cell / map.width as usize);
        (
            ((x as f64 + 0.5) * size) as f32,
            ((y as f64 + 0.5) * size) as f32,
        )
    };
    let wells = sim
        .land()
        .wells
        .list
        .iter()
        .filter(|w| w.is_open())
        .map(|w| dist(w.rect.centre_m()))
        .fold(f64::INFINITY, f64::min);
    let springs = sim
        .land()
        .water
        .aquifer
        .seep
        .iter()
        .flatten()
        .map(|&(c, _)| dist(centre(c as usize)))
        .fold(f64::INFINITY, f64::min);
    let streams = (0..map.water.len())
        .filter(|&c| map.water[c] == WATER_RIVER)
        .map(|c| dist(centre(c)))
        .fold(f64::INFINITY, f64::min);
    let homes = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| !x.members.is_empty())
        .map(|(_, x)| dist(x.home))
        .fold(f64::INFINITY, f64::min);
    [wells, springs, streams, homes]
}

#[test]
fn under_the_law_households_carry_their_heaps_to_a_tip_clear_of_the_water_and_the_homes() {
    let mut sim = world_with(5);
    sim.advance_minutes(2 * DAY).expect("advances");
    let def = tip_def(&sim);
    let (law, tip) = the_law_passed(&mut sim);
    let tip = tip.expect("a place qualifies in this valley, or this test shows nothing");
    // The tip stands more than the law's distance from every source and every home, on land.
    let [wells, springs, streams, homes] = clearances(&sim, tip.place);
    let from = f64::from(def.from_m);
    eprintln!(
        "the tip lies {wells:.0} m from a well, {springs:.0} m from a spring, {streams:.0} m from \
         a stream and {homes:.0} m from a home"
    );
    for (what, d) in [
        ("well", wells),
        ("spring", springs),
        ("stream", streams),
        ("home", homes),
    ] {
        assert!(d >= from, "the tip lies {d:.1} m from a {what}");
    }
    assert_eq!(sim.map().water[cell_of(sim.map(), tip.place)], WATER_LAND);
    assert_eq!(
        (tip.kg, tip.ash_kg, tip.opened),
        (0.0, 0.0, sim.now().day_index())
    );
    assert_eq!(
        sim.with_ctx_for_tests(|pop, ctx| pop.tip_site_for_tests(ctx, pop.polities[0].settlement)),
        Some(tip.place),
        "the place is chosen once, as the content's terms find it"
    );

    // One household's heap, carried there by hand: its decaying share, its ash and its load of a
    // disease arrive whole, and the heap is empty.
    let households = our_households(&sim);
    let h = households[0];
    let d = cholera(&sim);
    let (now, params) = (sim.now(), sim.rules().people.midden.clone());
    let carrier = {
        let pop = sim.people_mut_for_tests();
        pop.contagion.add(Node::Midden(h), d, 500.0);
        let (_, x) = pop
            .households
            .iter_mut()
            .find(|(_, x)| x.id == h)
            .expect("the household");
        x.midden.ash_kg += 3.25;
        x.members[0]
    };
    let x = sim.people().household(h).expect("there");
    let (kg, ash) = (
        x.midden.at_time(now, x.members.len(), &params),
        x.midden.ash_kg,
    );
    assert!(kg > 0.0 && ash > 0.0, "{kg} kg, {ash} kg of ash");
    sim.with_ctx_for_tests(|pop, ctx| pop.carry_midden_for_tests(ctx, carrier, tip.place));
    let x = sim.people().household(h).expect("there");
    assert_eq!((x.midden.kg, x.midden.ash_kg), (0.0, 0.0));
    assert_eq!(x.midden.tipped, now.day_index());
    let t = &sim.people().polities[0].tips[0];
    assert_eq!(
        (t.kg, t.ash_kg),
        (kg, ash),
        "what left the heap lies at the tip"
    );
    assert_eq!(t.households, vec![h]);
    let c = &sim.people().contagion;
    assert_eq!(c.load(Node::Midden(h), d), 0.0);
    assert_eq!(c.load(Node::Tip(t.id), d), 500.0);
    assert!(c.moves.iter().any(|m| m.how == How::Carried
        && m.from == Node::Midden(h)
        && m.to == Node::Tip(t.id)
        && m.amount == 500.0));
    // Carried from anywhere but the tip, nothing moves.
    let other = households[1];
    let (before, member) = {
        let x = sim.people().household(other).expect("there");
        (x.midden, x.members[0])
    };
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.carry_midden_for_tests(ctx, member, (tip.place.0 + 500.0, tip.place.1))
    });
    assert_eq!(sim.people().household(other).expect("there").midden, before);

    // Lived on, households carry their own: each that carried is on the tip's roll, and the tip
    // holds more than the one heap carried by hand. A law clearing middens in force beside it
    // asks nothing of them: the tip's supersedes it.
    let middens = a_law(&mut sim, PolicyKind::ClearMiddens);
    sim.advance_minutes(6 * DAY).expect("advances");
    let m = sim.people().polities[0]
        .laws
        .iter()
        .find(|l| l.id == middens)
        .expect("the law");
    assert_eq!(m.compliance.broken, 0, "the tip's law asks instead");
    assert!(
        households.iter().all(|&x| sim
            .people()
            .household(x)
            .is_some_and(|x| x.midden.carried < 0)),
        "nobody carried a heap from the water instead of to the tip"
    );
    let tipped: Vec<PermanentId> = households
        .iter()
        .copied()
        .filter(|&x| {
            x != h
                && sim
                    .people()
                    .household(x)
                    .is_some_and(|x| x.midden.tipped >= 0)
        })
        .collect();
    let t = &sim.people().polities[0].tips[0];
    let l = sim.people().polities[0]
        .laws
        .iter()
        .find(|l| l.id == law)
        .expect("the law");
    eprintln!(
        "{} of {} households carried their heaps; the tip holds {:.1} kg, {:.2} of it ash; {} \
         household-days broke it",
        tipped.len() + 1,
        households.len(),
        t.kg + t.ash_kg,
        t.ash_kg,
        l.compliance.broken
    );
    assert!(
        !tipped.is_empty(),
        "a household carried its heap to the tip"
    );
    for x in &tipped {
        assert!(
            t.households.binary_search(x).is_ok(),
            "{x} is on the tip's roll"
        );
    }
    assert!(t.kg > kg && t.ash_kg >= ash);
    // The services words say where the tip is and what lies in it.
    let polity = sim.people().polities[0].clone();
    let words = civ_sim::frames::government::services_words(&sim, &polity);
    eprintln!("{words:?}");
    assert!(
        words
            .iter()
            .any(|w| w.starts_with("The tip ") && w.contains(" of it ash, from ")),
        "{words:?}"
    );
    assert!(
        words
            .iter()
            .any(|w| w.starts_with("Refuse to the tip: broken "))
    );
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn where_no_place_qualifies_no_tip_opens_and_nobody_carries() {
    let mut sim = world_with(5);
    sim.advance_minutes(2 * DAY).expect("advances");
    // A distance no ground in the valley clears.
    for d in &mut sim.rules_mut_for_tests().expect("rules").catalog.policies {
        if let Some(t) = d.tip.as_mut() {
            t.from_m = 1000.0;
        }
    }
    let (law, tip) = the_law_passed(&mut sim);
    assert!(tip.is_none(), "no tip opened: {tip:?}");
    assert!(sim.people().polities.iter().all(|p| p.tips.is_empty()));
    sim.advance_minutes(6 * DAY).expect("advances");
    assert!(
        sim.people()
            .households
            .iter()
            .all(|(_, x)| x.midden.tipped < 0),
        "nobody carried"
    );
    assert!(
        !sim.people()
            .contagion
            .moves
            .iter()
            .any(|m| m.how == How::Carried)
    );
    let l = sim.people().polities[0]
        .laws
        .iter()
        .find(|l| l.id == law)
        .expect("the law");
    assert_eq!(l.compliance.broken, 0, "a law with no tip asks nothing");
}

#[test]
fn without_the_law_nobody_tips() {
    let mut sim = world_with(5);
    sim.advance_minutes(10 * DAY).expect("advances");
    let pop = sim.people();
    assert!(pop.polities.iter().all(|p| p.tips.is_empty()));
    assert!(pop.households.iter().all(|(_, x)| x.midden.tipped < 0));
    assert!(
        !pop.contagion
            .loads
            .keys()
            .any(|(n, _)| matches!(n, Node::Tip(_)))
    );
}

#[test]
fn a_heap_grown_large_beside_another_home_raises_heaps_near() {
    let mut sim = world_with(5);
    sim.advance_minutes(2 * DAY).expect("advances");
    let def = tip_def(&sim);
    let households = our_households(&sim);
    // A household with another's home within reach of its own.
    let near = |sim: &Sim, h: PermanentId| {
        let home = sim.people().household(h).expect("there").home;
        sim.people().households.iter().any(|(_, o)| {
            o.id != h
                && !o.members.is_empty()
                && f64::from(o.home.0 - home.0).hypot(f64::from(o.home.1 - home.1))
                    <= f64::from(def.heap_near_m)
        })
    };
    let h = households
        .iter()
        .copied()
        .find(|&h| near(&sim, h))
        .expect("homes stand close, or this test shows nothing");
    let raised = |sim: &mut Sim| {
        let list = households.clone();
        sim.with_ctx_for_tests(|pop, ctx| pop.heaps_near_for_tests(ctx, &list))
    };
    assert!(
        !raised(&mut sim).contains(&h),
        "a young heap raises nothing"
    );
    // Its ash alone past the content's size counts.
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == h {
            x.midden.ash_kg = f64::from(def.heap_kg) + 1.0;
        }
    }
    assert!(raised(&mut sim).contains(&h));
}

/// Lives on to `minute` past the next midnight.
fn to_morning(sim: &mut Sim, minute: i64) {
    let rest = DAY - sim.now().minutes().rem_euclid(DAY) + minute;
    sim.advance_minutes(rest).expect("advances");
}

/// A village of world 6 with one household's well dug and open, as the contagion target's: the
/// well, at about eight in the morning.
fn an_open_well() -> (Sim, PermanentId) {
    let mut sim = village(6);
    let well = 'found: {
        for _ in 0..12 {
            to_morning(&mut sim, 8 * 60);
            if let Some(w) = sim
                .land()
                .wells
                .list
                .iter()
                .find(|w| w.state == WellState::Digging)
            {
                break 'found w.id;
            }
            let all: Vec<PermanentId> = sim
                .people()
                .households
                .iter()
                .filter(|(_, x)| !x.members.is_empty())
                .map(|(_, x)| x.id)
                .collect();
            for h in all {
                let weighed = sim.with_ctx_for_tests(|pop, ctx| pop.review_well_for_tests(ctx, h));
                if let WellWeighed::Begun { .. } = weighed {
                    break 'found sim.land().wells.list.last().expect("begun").id;
                }
            }
            sim.advance_minutes(10 * DAY).expect("lives");
        }
        panic!("no household began a well");
    };
    let w = *sim.land().wells.get(well).expect("the well");
    let p = sim.land().patches.of_cell(w.cell as usize, sim.map().width);
    let floor = f64::from(w.ground_m) - f64::from(w.target_m);
    sim.land_mut_for_tests().water.heads[p] = floor + 2.0;
    let def = &sim.rules().catalog.wells[w.system];
    let h_per_m = def.h_per_m(sim.rules().people.digging.h_per_m3);
    let hours = f64::from(w.target_m - w.depth_m) * h_per_m + 0.5;
    sim.with_ctx_for_tests(|pop, ctx| pop.well_work_for_tests(ctx, well, hours));
    assert!(sim.land().wells.get(well).expect("the well").is_open());
    (sim, well)
}

/// A tip of the first polity at `place` holding `doses` of `disease`: its id.
fn a_tip_at(sim: &mut Sim, place: (f32, f32), disease: u16, doses: f64) -> PermanentId {
    let id = sim.allocate_id_for_tests();
    let day = sim.now().day_index();
    let pop = sim.people_mut_for_tests();
    pop.polities[0].tips.push(Tip {
        id,
        law: id,
        opened: day,
        place,
        kg: 100.0,
        ash_kg: 10.0,
        households: Vec::new(),
    });
    pop.contagion.add(Node::Tip(id), disease, doses);
    id
}

#[test]
fn a_tip_washes_and_seeps_to_the_well_below_it_as_a_heap_would_and_nowhere_else() {
    let (mut sim, well) = an_open_well();
    let d = cholera(&sim);
    let params = sim.rules().land.contamination.clone();
    let reach = params.runoff_radius_m.min(params.link_radius_m);
    let farthest = params.runoff_radius_m.max(params.link_radius_m);
    let w = *sim.land().wells.get(well).expect("the well");
    let (wx, wy) = w.rect.centre_m();
    let ground = f64::from(w.ground_m);
    // A household whose heap, at home, lies within reach of the well and no lower.
    let above = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| !x.members.is_empty() && x.midden.carried < 0)
        .filter_map(|x| {
            let dist = f64::from(wx - x.home.0)
                .hypot(f64::from(wy - x.home.1))
                .max(1.0);
            let drop = f64::from(sim.map().elevation[cell_of(sim.map(), x.home)]) - ground;
            (dist <= reach && drop >= 0.0).then_some((x.id, x.home, dist))
        })
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .expect("a heap within reach above the well");
    let (h, home, _) = above;
    // Land cells within reach of the well but below its mouth, and beyond every link's reach.
    let map = sim.map();
    let size = f64::from(map.cell_size_m);
    let cells: Vec<((f32, f32), f64, f64)> = (0..map.water.len())
        .filter(|&c| map.water[c] == WATER_LAND)
        .map(|c| {
            let (x, y) = (c % map.width as usize, c / map.width as usize);
            let at = (
                ((x as f64 + 0.5) * size) as f32,
                ((y as f64 + 0.5) * size) as f32,
            );
            let dist = f64::from(wx - at.0).hypot(f64::from(wy - at.1));
            (at, dist, f64::from(map.elevation[c]) - ground)
        })
        .collect();
    let below = cells
        .iter()
        .filter(|c| c.1 <= reach && c.2 < -0.5)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|c| c.0);
    let beyond = cells
        .iter()
        .filter(|c| c.1 > farthest + size && c.1 < farthest + 4.0 * size && c.2 >= 0.0)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|c| c.0)
        .expect("land above the well beyond reach");
    // A load on the heap at home and as much on a tip at the same place, and on tips below the
    // well and beyond reach of it, on a wet day.
    sim.people_mut_for_tests()
        .contagion
        .add(Node::Midden(h), d, 1000.0);
    let beside = a_tip_at(&mut sim, home, d, 1000.0);
    let low = below.map(|at| a_tip_at(&mut sim, at, d, 1000.0));
    let far = a_tip_at(&mut sim, beyond, d, 1000.0);
    sim.land_mut_for_tests().water.last_surplus_mm = 20.0;
    let day = sim.now().day_index();
    sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
    let c = &sim.people().contagion;
    let today: Vec<_> = c.moves.iter().filter(|m| m.day == day).collect();
    // What leaves the tip goes where the heap's goes, in the same amounts.
    let from = |n: Node| {
        let mut v: Vec<(u8, Node, f64)> = today
            .iter()
            .filter(|m| m.from == n)
            .map(|m| (m.how.code(), m.to, m.amount))
            .collect();
        v.sort_by_key(|a| (a.0, a.1));
        v
    };
    let (heap, tip) = (from(Node::Midden(h)), from(Node::Tip(beside)));
    assert!(
        heap.iter()
            .any(|m| m.1 == Node::Well(well) && m.0 == How::Runoff.code()),
        "the heap washes into the well below it"
    );
    assert_eq!(heap.len(), tip.len(), "{heap:?} against {tip:?}");
    for (a, b) in heap.iter().zip(&tip) {
        assert_eq!((a.0, a.1), (b.0, b.1));
        assert!((a.2 - b.2).abs() < 1e-9, "{a:?} against {b:?}");
    }
    let held = |n: Node| c.load(n, d);
    assert!((held(Node::Midden(h)) - held(Node::Tip(beside))).abs() < 1e-9);
    let transit = |n: Node| {
        let mut v: Vec<(i64, Node, f64)> = c
            .transit
            .iter()
            .filter(|t| t.from == n)
            .map(|t| (t.arrives, t.to, t.amount))
            .collect();
        v.sort_by_key(|a| (a.0, a.1));
        v
    };
    assert_eq!(
        transit(Node::Midden(h)).len(),
        transit(Node::Tip(beside)).len()
    );
    // None reaches the well from below it or beyond reach.
    let into_well = |n: Node| {
        today
            .iter()
            .any(|m| m.from == n && m.to == Node::Well(well))
            || c.transit
                .iter()
                .any(|t| t.from == n && t.to == Node::Well(well))
    };
    if let Some(low) = low {
        assert!(!into_well(Node::Tip(low)), "nothing climbs to the well");
    } else {
        eprintln!("no land within reach lies below the well's mouth here");
    }
    assert!(
        !into_well(Node::Tip(far)),
        "nothing reaches the well from beyond reach"
    );
    saves_and_goes_on_alike(&mut sim, DAY);
}
