//! Sewers and treatment (ADR-0041 §1-§4): a sewer's route must fall all the way to the river, or
//! it is refused; laid by law a metre at a time from its households' walls and binder, it carries
//! once whole; homes beside it connect, and each day their heaps' new decaying share and what their
//! people shed go whole to the outfall's reach, recorded as carried, in the order connected while
//! its capacity lasts; a settling tank its keeper holds keeps the content's share as sludge, with
//! that share of each disease; without the law nothing changes; and a world with a sewer saves and
//! loads exactly. A sewage farm (ADR-0044 §4): a field beside the route below the lowest connection,
//! whose last harvest its soil held back, takes what leaves the tank up to its area's limit, into
//! its soil as dung and its loads to its own node, washing to a well beside it; mass and loads are
//! conserved through tank, farm and reach; its crop rises only where the soil held it back.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::contagion::{How, Node};
use civ_agents::polity::{
    IssueKind, Law, LawStatus, Outcome as Decided, PolicyKind, SewerCause, SewerDef, WaySite,
};
use civ_content::ContentRegistry;
use civ_core::{PermanentId, SimTime};
use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
use civ_land::wells::{Well, WellState};
use civ_land::{Allowance, Field, FieldSoil, FieldStage, HarvestRecord, Limit, Party, RectCm};
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

/// A world of `seed` whose band has just settled, as the tips target's.
fn world_with(seed: u64) -> Sim {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Sewers".to_owned(),
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
    .expect("generates");
    sim.advance_minutes(2 * DAY).expect("advances");
    sim
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "sewers").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(sim, &loaded);
    let sewers = |s: &Sim| {
        s.people()
            .polities
            .iter()
            .map(|p| p.sewers.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(sewers(&loaded), sewers(sim));
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}

/// The content's terms for a sewer.
fn sewer_def(sim: &Sim) -> SewerDef {
    sim.rules()
        .catalog
        .policies
        .iter()
        .find_map(|d| d.sewer.clone())
        .expect("the core content lays sewers")
}

fn index(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect("a core good")
}

fn cholera(sim: &Sim) -> u16 {
    let i = sim
        .rules()
        .catalog
        .disease_index(CHOLERA)
        .expect("cholera is content");
    u16::try_from(i).expect("few diseases")
}

fn set(sim: &mut Sim, household: PermanentId, g: usize, kg: f64) {
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == household {
            x.stores[g] = kg;
        }
    }
}

/// A law of `kind` in force in the first polity, every adult of its settlement knowing it, as the
/// tips target's: its id.
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

/// The eight steps, axis first.
const STEPS: [(i64, i64); 8] = [
    (1, 0),
    (0, 1),
    (-1, 0),
    (0, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
    (1, -1),
];

/// A straight line of `n` land cells ending beside a river cell, from its head to the river cell
/// (kept last), that `ok` accepts, none of its land cells beside a river cell but the last: the
/// first found.
fn a_line(sim: &Sim, n: i64, ok: impl Fn(&[u32]) -> bool) -> Option<Vec<u32>> {
    let map = sim.map();
    let (w, h) = (i64::from(map.width), i64::from(map.height));
    let at = |x: i64, y: i64| (x >= 0 && y >= 0 && x < w && y < h).then(|| (y * w + x) as usize);
    let river_beside = |c: usize| {
        let (x, y) = (c as i64 % w, c as i64 / w);
        STEPS
            .iter()
            .any(|&(dx, dy)| at(x + dx, y + dy).is_some_and(|n| map.water[n] == WATER_RIVER))
    };
    for r in (0..map.water.len()).filter(|&c| map.water[c] == WATER_RIVER) {
        let (rx, ry) = (r as i64 % w, r as i64 / w);
        'dirs: for &(dx, dy) in &STEPS {
            // From the head outward n cells, back to the river.
            let mut cells = Vec::new();
            for k in (1..=n).rev() {
                let Some(c) = at(rx + dx * k, ry + dy * k) else {
                    continue 'dirs;
                };
                if map.water[c] != WATER_LAND || (k > 1 && river_beside(c)) {
                    continue 'dirs;
                }
                cells.push(c as u32);
            }
            cells.push(r as u32);
            if ok(&cells) {
                return Some(cells);
            }
        }
    }
    None
}

/// Whether `cells` fall at each step by at least `fall` of its length.
fn falls(sim: &Sim, cells: &[u32], fall: f64) -> bool {
    let map = sim.map();
    let w = i64::from(map.width);
    cells.windows(2).all(|p| {
        let (dx, dy) = (
            i64::from(p[1]) % w - i64::from(p[0]) % w,
            i64::from(p[1]) / w - i64::from(p[0]) / w,
        );
        let len = f64::from(map.cell_size_m) * ((dx * dx + dy * dy) as f64).sqrt();
        f64::from(map.elevation[p[1] as usize])
            <= f64::from(map.elevation[p[0] as usize]) - fall * len - 1e-6
    })
}

/// A way kept by nobody over `cells`.
fn a_way(sim: &mut Sim, cells: Vec<u32>) {
    let id = sim.allocate_id_for_tests();
    let today = sim.now().day_index();
    sim.land_mut_for_tests().ways.add(Way {
        id,
        begun: today,
        segments: vec![Segment {
            cells,
            corridor_m: 4.0,
            built_m: 4.0,
            usable_m: 4.0,
            surface: Surface::Cleared,
            keeper: Keeper::Nobody,
            kept: today,
        }],
        modes: WALK,
        trips: 0.0,
        trips_day: today,
        rank: Rank::Lane,
        track: None,
    });
}

/// Households of the first polity's settlement with people and a building, by id.
fn our_households(sim: &Sim) -> Vec<PermanentId> {
    let settlement = sim.people().polities[0].settlement;
    let mut out: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .filter(|(_, x)| sim.land().buildings.iter().any(|b| b.household == x.id))
        .map(|(_, x)| x.id)
        .collect();
    out.sort_unstable();
    out
}

/// The centre of cell `c`, metres.
fn centre(sim: &Sim, c: u32) -> (f32, f32) {
    let map = sim.map();
    let size = map.cell_size_m;
    (
        ((c % map.width) as f32 + 0.5) * size,
        ((c / map.width) as f32 + 0.5) * size,
    )
}

/// A world with a falling line from a head to the river, a way along it, the first `homes` of
/// its households moved onto its land cells (from the head), and the law laying a sewer passed and
/// its sewer marked out from the head: the sim, the line, the households moved, and the sewer.
fn a_sewer_marked_out(homes: usize) -> (Sim, Vec<u32>, Vec<PermanentId>, PermanentId) {
    let mut sim = world_with(5);
    let fall = f64::from(sewer_def(&sim).fall);
    let line = a_line(&sim, 6, |c| falls(&sim, c, fall))
        .expect("a line of land falling to the river in this valley, or this test shows nothing");
    a_way(&mut sim, line[1..line.len() - 1].to_vec());
    let ours = our_households(&sim);
    assert!(ours.len() >= homes, "{} households", ours.len());
    let moved: Vec<PermanentId> = ours[..homes].to_vec();
    let places: Vec<(f32, f32)> = (0..homes).map(|i| centre(&sim, line[i])).collect();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if let Some(i) = moved.iter().position(|&h| h == x.id) {
            x.home = places[i];
        }
    }
    let law = a_law(&mut sim, PolicyKind::LaySewer);
    sim.people_mut_for_tests().polities[0].ways.push((
        law,
        WaySite {
            cell: line[0],
            labour_h: 1.0,
            work: None,
        },
    ));
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_public_sewer_for_tests(ctx, 0, law));
    let sewer = sim.people().polities[0]
        .way_site_of(law)
        .and_then(|s| s.work)
        .expect("the sewer marked out");
    (sim, line, moved, sewer)
}

/// The sewer of the first polity, laid whole by household `h` from walls and binder given it.
fn lay_whole(sim: &mut Sim, h: PermanentId, sewer: PermanentId) {
    let def = sewer_def(sim);
    let len = f64::from(sim.people().polities[0].sewers[0].length_m);
    let (brick, lime) = (
        index(sim, "core:good/brick"),
        index(sim, "core:good/quicklime"),
    );
    set(sim, h, brick, len * f64::from(def.walls_kg_m) + 1.0);
    set(sim, h, lime, len * f64::from(def.binder_kg_m) + 1.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_work_for_tests(ctx, h, sewer, 1.0e9));
    assert!(sim.people().polities[0].sewers[0].whole());
}

#[test]
fn a_route_that_would_rise_is_refused() {
    let mut sim = world_with(5);
    let fall = f64::from(sewer_def(&sim).fall);
    let law = a_law(&mut sim, PolicyKind::LaySewer);
    let begin = |sim: &mut Sim, from: u32| {
        sim.with_ctx_for_tests(|pop, ctx| pop.begin_sewer(ctx, 0, law, from))
    };
    // A line to the river whose first step rises: no pumping, so no sewer.
    let rising = a_line(&sim, 4, |c| {
        let z = |i: usize| sim.map().elevation[c[i] as usize];
        z(1) > z(0) + 0.01
    })
    .expect("a line to the river whose first step climbs");
    a_way(&mut sim, rising[1..rising.len() - 1].to_vec());
    assert_eq!(
        begin(&mut sim, rising[0]),
        None,
        "a rising route is refused"
    );
    assert!(sim.people().polities[0].sewers.is_empty());
    // A line falling all the way is taken, once a way runs along it.
    let falling = a_line(&sim, 6, |c| falls(&sim, c, fall)).expect("a line falling to the river");
    assert_eq!(begin(&mut sim, falling[0]), None, "no way, no route");
    a_way(&mut sim, falling[1..falling.len() - 1].to_vec());
    let id = begin(&mut sim, falling[0]).expect("a falling route is taken");
    let s = sim.people().polities[0]
        .sewers
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .expect("the sewer");
    assert_eq!(s.cells.first(), Some(&falling[0]));
    assert_eq!(
        sim.map().water[*s.cells.last().expect("an outfall") as usize],
        WATER_RIVER
    );
    assert!(falls(&sim, &s.cells, fall), "it falls all the way");
    assert!(s.length_m > 0.0 && s.laid_m == 0.0 && !s.whole());
}

#[test]
fn a_sewer_laid_by_law_carries_its_homes_share_and_loads_to_the_river_exactly() {
    let (mut sim, line, homes, sewer) = a_sewer_marked_out(3);
    let def = sewer_def(&sim);
    let d = cholera(&sim);
    let s = sim.people().polities[0].sewers[0].clone();
    assert_eq!(s.id, sewer);
    assert_eq!(s.cells, line, "it runs along the way to the river");
    // Laid in part, it carries nothing and nobody connects.
    let (brick, lime) = (
        index(&sim, "core:good/brick"),
        index(&sim, "core:good/quicklime"),
    );
    let half = f64::from(s.length_m) / 2.0;
    set(&mut sim, homes[0], brick, half * f64::from(def.walls_kg_m));
    set(&mut sim, homes[0], lime, 1.0e6);
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_work_for_tests(ctx, homes[0], sewer, 1.0e9));
    let laid = sim.people().polities[0].sewers[0].laid_m;
    assert!((f64::from(laid) - half).abs() < 1e-2, "{laid} of {half}");
    let x = sim.people().household(homes[0]).expect("home");
    assert!(x.stores[brick] < 1e-6, "its walls used");
    assert!(
        sim.people().polities[0].way_given_by(s.law, homes[0]) > 0.0,
        "its hours given to the law"
    );
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_day_for_tests(ctx, &[]));
    assert!(sim.people().polities[0].sewers[0].connections.is_empty());
    // Laid whole, the homes beside it connect (free, with no law asking them to carry) and each
    // gives its day's decaying share; what its people shed today goes with it to the reach.
    lay_whole(&mut sim, homes[1], sewer);
    let heap = |sim: &Sim, h: PermanentId| {
        let x = sim.people().household(h).expect("home");
        x.midden
            .at_time(sim.now(), x.members.len(), &sim.rules().people.midden)
    };
    let before: Vec<f64> = homes.iter().map(|&h| heap(&sim, h)).collect();
    let shed: Vec<(PermanentId, u16, f64)> = homes
        .iter()
        .enumerate()
        .map(|(i, &h)| (h, d, 1000.0 * (i + 1) as f64))
        .collect();
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_day_for_tests(ctx, &shed));
    let s = sim.people().polities[0].sewers[0].clone();
    let joined: Vec<PermanentId> = s.connections.iter().map(|c| c.household).collect();
    for h in &homes {
        assert!(joined.contains(h), "{h:?} beside it connected");
    }
    let day = sim.now().day_index();
    let params = sim.rules().people.midden.clone();
    let mut entered = 0.0;
    for (i, &h) in homes.iter().enumerate() {
        let c = s
            .connections
            .iter()
            .find(|c| c.household == h)
            .expect("connected");
        let members = sim.people().household(h).expect("home").members.len();
        assert_eq!((c.day, c.cause), (day, SewerCause::Served));
        let gain = params.after(0.0, members, 1.0);
        assert!((c.kg - gain.min(before[i])).abs() < 1e-9, "{} {gain}", c.kg);
        // Its heap lost exactly what went in.
        assert!(
            (heap(&sim, h) - (before[i] - c.kg)).abs() < 1e-9,
            "{} {}",
            heap(&sim, h),
            before[i]
        );
        entered += c.kg;
    }
    assert!((s.entered_kg - entered).abs() < 1e-9 && s.settled_kg == 0.0);
    // Every dose shed went whole to the outfall's reach, recorded as carried.
    let total: f64 = shed.iter().map(|s| s.2).sum();
    let reach = sim.people().contagion.load(Node::Reach(s.reach), d);
    assert!((reach - total).abs() < 1e-6, "{reach} of {total}");
    for &(h, _, amount) in &shed {
        assert_eq!(sim.people().contagion.load(Node::Midden(h), d), 0.0);
        let carried: f64 = sim
            .people()
            .contagion
            .moves
            .iter()
            .filter(|m| {
                m.how == How::Carried
                    && m.from == Node::Midden(h)
                    && m.to == Node::Reach(s.reach)
                    && m.day == day
            })
            .map(|m| m.amount)
            .sum();
        assert!((carried - amount).abs() < 1e-9, "{carried} {amount}");
    }
    // The observer tells it in words.
    let words = civ_sim::frames::government::services_words(&sim, &sim.people().polities[0]);
    assert!(
        words.iter().any(|w| w.starts_with("A sewer of ")
            && w.contains("3 homes joined")
            && w.contains("a day to the river")),
        "{words:?}"
    );
    // Days on, the street's heaps stay light while an unconnected one grows.
    let other = our_households(&sim)
        .into_iter()
        .find(|h| !homes.contains(h))
        .expect("a home not beside it");
    let (light0, grows0) = (heap(&sim, homes[0]), heap(&sim, other));
    // Its people drink a heavy load and fall sick; what they shed goes down the sewer to the river
    // below the outfall, day by day, not into their heap.
    let now = sim.now();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == homes[0] {
            x.water_l = 200.0;
            x.water_at = now;
        }
    }
    sim.people_mut_for_tests()
        .contagion
        .store_add(homes[0], d, Node::Reach(s.reach), 1.0e6);
    let from = sim.now().day_index();
    sim.advance_minutes(10 * DAY).expect("lives");
    assert!(
        heap(&sim, homes[0]) <= light0 + 1e-9,
        "a sewered heap stays light"
    );
    assert!(heap(&sim, other) > grows0, "an unsewered heap grows");
    let sum = |sim: &Sim, how: How, from_node: Node, to: Node| -> f64 {
        sim.people()
            .contagion
            .moves
            .iter()
            .filter(|m| m.how == how && m.from == from_node && m.to == to && m.day > from)
            .map(|m| m.amount)
            .sum()
    };
    let carried = sum(
        &sim,
        How::Carried,
        Node::Midden(homes[0]),
        Node::Reach(s.reach),
    );
    let shed: f64 = sim
        .people()
        .contagion
        .moves
        .iter()
        .filter(|m| m.how == How::Shed && m.to == Node::Midden(homes[0]) && m.day > from)
        .map(|m| m.amount)
        .sum();
    eprintln!("shed {shed:.0} doses into the heap, {carried:.0} carried to the river");
    assert!(shed > 0.0, "its people shed");
    assert!(
        (carried - shed).abs() < 1e-6 * shed,
        "all of it went to the river"
    );
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn a_sewer_serves_its_connections_in_order_while_its_capacity_lasts() {
    let (mut sim, _, homes, sewer) = a_sewer_marked_out(3);
    lay_whole(&mut sim, homes[0], sewer);
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_day_for_tests(ctx, &[]));
    let s = sim.people().polities[0].sewers[0].clone();
    assert_eq!(s.connections.len(), 3);
    let first = s.connections[0].kg;
    assert!(first > 0.0);
    // A capacity of just the first's share: the first is served, the rest wait, heaping as before.
    sim.advance_minutes(DAY).expect("lives");
    let d = cholera(&sim);
    let order: Vec<PermanentId> = s.connections.iter().map(|c| c.household).collect();
    let heap = |sim: &Sim, h: PermanentId| {
        let x = sim.people().household(h).expect("home");
        x.midden
            .at_time(sim.now(), x.members.len(), &sim.rules().people.midden)
    };
    let before: Vec<f64> = order.iter().map(|&h| heap(&sim, h)).collect();
    sim.people_mut_for_tests().polities[0].sewers[0].capacity_kg_day = (first * 1.01) as f32;
    let shed = [(order[1], d, 500.0)];
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_day_for_tests(ctx, &shed));
    let s = sim.people().polities[0].sewers[0].clone();
    let causes: Vec<SewerCause> = s.connections.iter().map(|c| c.cause).collect();
    assert_eq!(
        causes,
        [
            SewerCause::Served,
            SewerCause::Overloaded,
            SewerCause::Overloaded
        ]
    );
    assert!((s.entered_kg - s.connections[0].kg).abs() < 1e-9);
    for i in 1..3 {
        assert!(
            (heap(&sim, order[i]) - before[i]).abs() < 1e-9,
            "unserved heaps keep theirs"
        );
    }
    // An unserved home's shedding stays in its heap.
    assert_eq!(
        sim.people().contagion.load(Node::Midden(order[1]), d),
        500.0
    );
    // Broken, it serves nobody.
    sim.people_mut_for_tests().polities[0].sewers[0].broken = true;
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_day_for_tests(ctx, &[]));
    let s = &sim.people().polities[0].sewers[0];
    assert!(s.connections.iter().all(|c| c.cause == SewerCause::Broken));
    assert_eq!(s.entered_kg, 0.0);
}

#[test]
fn a_settling_tank_keeps_its_share_as_sludge_with_that_share_of_each_disease() {
    let (mut sim, _, homes, sewer) = a_sewer_marked_out(2);
    let def = sewer_def(&sim);
    lay_whole(&mut sim, homes[0], sewer);
    let (tank, sludge) = (
        index(&sim, "core:good/settling_tank"),
        index(&sim, "core:good/sludge"),
    );
    let goods = sim.rules().catalog.goods.len();
    {
        let p = &mut sim.people_mut_for_tests().polities[0];
        p.stores.resize(goods, 0.0);
        p.stores[tank] = 1.0;
    }
    let d = cholera(&sim);
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_day_for_tests(ctx, &[(homes[1], d, 800.0)]));
    let s = sim.people().polities[0].sewers[0].clone();
    let share = f64::from(def.settle_share);
    assert!((share - 0.5).abs() < 1e-9, "the prior");
    assert!(s.entered_kg > 0.0);
    assert!((s.settled_kg - share * s.entered_kg).abs() < 1e-9);
    let held = sim.people().polities[0].stores[sludge];
    assert!(
        (held - s.settled_kg).abs() < 1e-9,
        "the sludge is the store's: {held}"
    );
    let c = &sim.people().contagion;
    assert!((c.load(Node::Sewer(sewer), d) - 400.0).abs() < 1e-9);
    assert!((c.load(Node::Reach(s.reach), d) - 400.0).abs() < 1e-9);
    let words = civ_sim::frames::government::services_words(&sim, &sim.people().polities[0]);
    assert!(
        words.iter().any(|w| w.contains("settled as sludge")),
        "{words:?}"
    );
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn without_the_law_nothing_changes() {
    let mut a = world_with(5);
    // A day of sewers where none is laid changes nothing.
    let before = persist::encode_sections(&a);
    a.with_ctx_for_tests(|pop, ctx| pop.sewer_day_for_tests(ctx, &[]));
    let after = persist::encode_sections(&a);
    assert_eq!(before.len(), after.len());
    for (x, y) in before.iter().zip(&after) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
    a.advance_minutes(5 * DAY).expect("lives");
    assert!(a.people().polities.iter().all(|p| p.sewers.is_empty()));
    assert!(
        a.people()
            .contagion
            .moves
            .iter()
            .all(|m| !matches!(m.to, Node::Sewer(_)))
    );
}

#[test]
fn where_laws_ask_homes_to_carry_their_heaps_a_sewer_beside_them_is_reckoned_to_repay_its_work() {
    let (mut sim, line, homes, _) = a_sewer_marked_out(3);
    // The marked-out sewer's law gone, none is being laid; a law carrying refuse to a tip asks
    // each household to carry its heap every 30 days.
    {
        let p = &mut sim.people_mut_for_tests().polities[0];
        p.sewers.clear();
        p.ways.clear();
        p.laws.retain(|l| l.kind != PolicyKind::LaySewer);
    }
    let settlement = sim.people().polities[0].settlement;
    let public = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.public_sewer_for_tests(ctx, settlement))
    };
    assert_eq!(
        public(&mut sim),
        None,
        "nothing asks them to carry, no death: no sewer"
    );
    let tip = a_law(&mut sim, PolicyKind::RefuseToTip);
    sim.with_ctx_for_tests(|pop, ctx| pop.open_tip_for_tests(ctx, 0, tip));
    if sim.people().polities[0].tips.is_empty() {
        eprintln!("no tip opens in this valley; nothing to weigh");
        return;
    }
    // Without walls or binder at hand or a price for them, it is not reckoned.
    let def = sewer_def(&sim);
    let (brick, lime) = (
        index(&sim, "core:good/brick"),
        index(&sim, "core:good/quicklime"),
    );
    for &h in &homes {
        set(&mut sim, h, brick, 100.0 * f64::from(def.walls_kg_m));
        set(&mut sim, h, lime, 100.0 * f64::from(def.binder_kg_m));
    }
    let (head, labour_h, saves) = public(&mut sim).expect("a sewer beside the homes repays itself");
    assert!(
        line[..line.len() - 1].contains(&head),
        "from a home on the falling line, the shortest that serves them all: {head}"
    );
    assert!(labour_h > 0.0);
    for h in &homes {
        assert!(
            saves.iter().any(|s| s.0 == *h && s.1 > 0.0),
            "{h:?} would be spared its carrying: {saves:?}"
        );
    }
}

/// The index of the last route cell any home connected to the first polity's first sewer lies
/// beside.
fn lowest_connection(sim: &Sim) -> usize {
    let s = &sim.people().polities[0].sewers[0];
    let w = i64::from(sim.map().width);
    let xy = |c: u32| (i64::from(c) % w, i64::from(c) / w);
    s.connections
        .iter()
        .filter_map(|c| {
            let home = civ_agents::population::cell_of(
                sim.map(),
                sim.people().household(c.household)?.home,
            ) as u32;
            let (hx, hy) = xy(home);
            s.cells.iter().rposition(|&r| {
                let (rx, ry) = xy(r);
                (rx - hx).abs() <= 1 && (ry - hy).abs() <= 1
            })
        })
        .max()
        .expect("a connection")
}

/// A land cell beside route cell `line[k]`, off the route.
fn beside_cell(sim: &Sim, line: &[u32], k: usize) -> Option<u32> {
    let map = sim.map();
    let (w, h) = (i64::from(map.width), i64::from(map.height));
    let (x, y) = (i64::from(line[k]) % w, i64::from(line[k]) / w);
    STEPS.iter().find_map(|&(dx, dy)| {
        let (nx, ny) = (x + dx, y + dy);
        let c = (nx >= 0 && ny >= 0 && nx < w && ny < h).then(|| (ny * w + nx) as u32)?;
        (map.water[c as usize] == WATER_LAND && !line.contains(&c)).then_some(c)
    })
}

/// The north-west corner of cell `c`, centimetres.
fn corner_cm(sim: &Sim, c: u32) -> (i32, i32) {
    let map = sim.map();
    let size = (map.cell_size_m * 100.0) as i32;
    ((c % map.width) as i32 * size, (c / map.width) as i32 * size)
}

/// This calendar year.
fn this_year(sim: &Sim) -> i32 {
    i32::try_from(sim.now().date().year).expect("a year")
}

/// A field of household `h`, `side_m` square at the north-west corner of cell `c`, sown in time
/// and fully tended, its soil thin (10 kg of nitrogen a hectare to give this year), and its last
/// harvest, of `year`, held back by `limit`: its id.
fn a_field(
    sim: &mut Sim,
    h: PermanentId,
    c: u32,
    side_m: f32,
    limit: Limit,
    year: i32,
) -> PermanentId {
    let id = sim.allocate_id_for_tests();
    let (x, y) = corner_cm(sim, c);
    let crop = &sim.rules().catalog.crops[0];
    let sown_day = i64::from(crop.sow_from_day);
    let side = (side_m * 100.0) as i32;
    sim.land_mut_for_tests().fields.push(Field {
        id,
        household: h,
        holder: Party::Household(h),
        lease: None,
        rect: RectCm {
            x,
            y,
            w: side,
            h: side,
        },
        crop: 0,
        stage: FieldStage::Reaped,
        stage_since: SimTime::ZERO,
        work_h: 0.0,
        tended_h: 1.0e6,
        ground: 1.0,
        clear_h_per_ha: 0.0,
        broken: true,
        sown_day,
        sheaves_kg: 0.0,
        harvests: 1,
        water_mm: 0.0,
        need_mm: 0.0,
        got_mm: 0.0,
        soil: FieldSoil {
            fast_n: 50.0,
            slow_n: 1000.0,
            supply_n: 10.0,
            record: vec![HarvestRecord {
                year,
                kg_per_ha: 300.0,
                limit,
            }],
        },
        last_flood_depth_cm: 0,
        submerged_days: 0,
        reserved: None,
    });
    id
}

fn field(sim: &Sim, id: PermanentId) -> Field {
    sim.land()
        .fields
        .iter()
        .find(|f| f.id == id)
        .expect("the field")
        .clone()
}

/// A sewer laid whole beside two homes at its head, its homes connected, a settling tank at its
/// outfall if `tank`, and a field of the first home's household `side_m` square on a land cell
/// beside its route below its lowest connection, its last harvest this year's held back by
/// `limit`: the sim, the route, the homes, the sewer, the field and its cell.
fn a_sewer_with_a_field(
    tank: bool,
    side_m: f32,
    limit: Limit,
) -> (
    Sim,
    Vec<u32>,
    Vec<PermanentId>,
    PermanentId,
    PermanentId,
    u32,
) {
    let (mut sim, line, homes, sewer) = a_sewer_marked_out(2);
    lay_whole(&mut sim, homes[0], sewer);
    if tank {
        let tank = index(&sim, "core:good/settling_tank");
        let goods = sim.rules().catalog.goods.len();
        let p = &mut sim.people_mut_for_tests().polities[0];
        p.stores.resize(goods, 0.0);
        p.stores[tank] = 1.0;
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_day_for_tests(ctx, &[]));
    assert!(
        sim.people().polities[0].sewers[0].farm.is_none(),
        "no field yet"
    );
    let lowest = lowest_connection(&sim);
    let (k, cell) = (lowest + 1..line.len())
        .find_map(|k| Some((k, beside_cell(&sim, &line, k)?)))
        .expect("land beside the route below its lowest connection, or this test shows nothing");
    eprintln!(
        "lowest connection at {lowest} of {}; the field beside {k}",
        line.len()
    );
    let year = this_year(&sim);
    let f = a_field(&mut sim, homes[0], cell, side_m, limit, year);
    sim.advance_minutes(DAY).expect("lives");
    (sim, line, homes, sewer, f, cell)
}

#[test]
fn a_sewage_farm_takes_what_leaves_the_tank_up_to_its_area_and_mass_and_loads_are_conserved() {
    // A field of 10 m by 10 m, a hundredth of a hectare: one person's discharge a day.
    let (mut sim, _, homes, sewer, f, _) = a_sewer_with_a_field(true, 10.0, Limit::Soil);
    let def = sewer_def(&sim);
    assert_eq!(def.farm_people_ha, 100.0, "the prior");
    let d = cholera(&sim);
    let midden = sim.rules().people.midden.clone();
    let soil_params = sim.rules().land.soil.clone();
    let before = field(&sim, f);
    let ha = before.area_ha();
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_day_for_tests(ctx, &[(homes[1], d, 900.0)]));
    let s = sim.people().polities[0].sewers[0].clone();
    assert_eq!(s.farm, Some(f), "the field's household takes the flow");
    let rest = s.entered_kg - s.settled_kg;
    let limit = f64::from(def.farm_people_ha) * ha * midden.kg_per_person_day;
    eprintln!(
        "{:.2} kg in, {:.2} settled, {:.2} to the farm (its limit {limit:.2}), {:.2} to the river",
        s.entered_kg,
        s.settled_kg,
        s.farm_kg,
        rest - s.farm_kg
    );
    assert!(
        (s.settled_kg - 0.5 * s.entered_kg).abs() < 1e-9,
        "settling first"
    );
    assert!(
        rest > limit,
        "more than the farm takes, or this test shows nothing"
    );
    assert!(
        (s.farm_kg - limit).abs() < 1e-9,
        "the farm takes its area's limit"
    );
    assert!(
        (limit - 0.5).abs() < 1e-9,
        "100 people a hectare at 0.5 kg each: {limit}"
    );
    // Its nitrogen entered the soil's pools as dung at the heap's nitrogen a kilogram.
    let mut expected = before.soil.clone();
    expected.manured(&soil_params, s.farm_kg * midden.n_per_kg() / ha);
    let after = field(&sim, f).soil;
    assert_eq!(after, expected);
    assert!(after.fast_n > before.soil.fast_n && after.slow_n > before.soil.slow_n);
    assert!(after.supply_n > before.soil.supply_n);
    // Every dose is conserved: half at the tank, the farm's share of the rest at the field, the
    // rest to the river, each recorded as carried.
    let c = &sim.people().contagion;
    let (tank, farm, reach) = (
        c.load(Node::Sewer(sewer), d),
        c.load(Node::Field(f), d),
        c.load(Node::Reach(s.reach), d),
    );
    eprintln!("{tank:.1} doses settled, {farm:.1} on the farm, {reach:.1} to the river");
    assert!((tank - 450.0).abs() < 1e-9);
    assert!((farm - 450.0 * s.farm_kg / rest).abs() < 1e-9);
    assert!((tank + farm + reach - 900.0).abs() < 1e-9, "conserved");
    assert!(reach < 450.0, "the reach below takes less");
    let day = sim.now().day_index();
    let carried: f64 = c
        .moves
        .iter()
        .filter(|m| {
            m.how == How::Carried
                && m.from == Node::Midden(homes[1])
                && m.to == Node::Field(f)
                && m.day == day
        })
        .map(|m| m.amount)
        .sum();
    assert!((carried - farm).abs() < 1e-9, "{carried} {farm}");
    // The observer names the farm and what it took.
    let words = civ_sim::frames::government::services_words(&sim, &sim.people().polities[0]);
    assert!(
        words.iter().any(|w| w.contains("settled as sludge")
            && w.contains("as a farm")
            && w.contains("spread on a field of")),
        "{words:?}"
    );
    // A hectare takes all that leaves the tank, and the river nothing of it.
    for x in sim.land_mut_for_tests().fields.iter_mut() {
        if x.id == f {
            (x.rect.w, x.rect.h) = (10_000, 10_000);
        }
    }
    sim.advance_minutes(DAY).expect("lives");
    let s = sim.people().polities[0].sewers[0].clone();
    assert_eq!(s.farm, Some(f));
    assert!(s.entered_kg > 0.0 && s.entered_kg - s.settled_kg < 50.0);
    assert!(
        (s.farm_kg - (s.entered_kg - s.settled_kg)).abs() < 1e-9,
        "{} of {}",
        s.farm_kg,
        s.entered_kg - s.settled_kg
    );
    let words = civ_sim::frames::government::services_words(&sim, &sim.people().polities[0]);
    assert!(
        words.iter().any(|w| w.contains("0 kg a day to the river")),
        "{words:?}"
    );
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn a_farm_raises_a_soil_limited_crop_and_not_a_season_limited_one() {
    let (mut sim, _, _, _, f, _) = a_sewer_with_a_field(false, 20.0, Limit::Soil);
    let crop = sim.rules().catalog.crops[0].clone();
    let soil_params = sim.rules().land.soil.clone();
    // What the field would give at its ripe day in a season whose water allows `water`.
    let grain = |sim: &Sim, water: f64| {
        let x = field(sim, f);
        let allow = Allowance {
            water,
            soil_kg_per_ha: x.soil.allows_kg_ha(&soil_params, &crop),
        };
        x.harvest_kg(&crop, x.ripe_day(&crop), allow)
    };
    let (wet, dry) = (grain(&sim, 1.0), grain(&sim, 0.05));
    assert_eq!(
        wet.2,
        Limit::Soil,
        "an average season: the soil holds it back"
    );
    assert_eq!(
        dry.2,
        Limit::Season,
        "a dry season: the season holds it back"
    );
    let farmed: f64 = (0..5)
        .map(|_| {
            sim.advance_minutes(DAY).expect("lives");
            sim.people().polities[0].sewers[0].farm_kg
        })
        .sum();
    assert!(farmed > 0.0, "the farm took the flow");
    let (wet2, dry2) = (grain(&sim, 1.0), grain(&sim, 0.05));
    eprintln!(
        "{farmed:.1} kg farmed: {:.2} kg of grain in an average season, then {:.2}; {:.2} in a \
         dry one, then {:.2}",
        wet.1, wet2.1, dry.1, dry2.1
    );
    assert!(wet2.1 > wet.1, "the soil-limited crop rises");
    assert_eq!(dry2, dry, "the season-limited crop does not");
}

#[test]
fn the_fields_household_takes_the_flow_only_after_its_soil_held_a_harvest_back() {
    let (mut sim, _, _, _, f, _) = a_sewer_with_a_field(false, 20.0, Limit::Season);
    let soil = field(&sim, f).soil;
    let s = sim.people().polities[0].sewers[0].clone();
    assert!(s.entered_kg > 0.0);
    assert_eq!(
        (s.farm, s.farm_kg),
        (None, 0.0),
        "the season held it back: no farm"
    );
    assert_eq!(field(&sim, f).soil, soil, "its soil untouched");
    let set_record = |sim: &mut Sim, record: Vec<HarvestRecord>| {
        for x in sim.land_mut_for_tests().fields.iter_mut() {
            if x.id == f {
                x.soil.record = record.clone();
            }
        }
    };
    let year = this_year(&sim);
    let soil_held = |year: i32| HarvestRecord {
        year,
        kg_per_ha: 300.0,
        limit: Limit::Soil,
    };
    // Held back by the soil, but three years since: the field is not farmed now.
    set_record(&mut sim, vec![soil_held(year - 3)]);
    sim.advance_minutes(DAY).expect("lives");
    assert_eq!(
        sim.people().polities[0].sewers[0].farm,
        None,
        "an old harvest"
    );
    // This year's: the flow is led onto it.
    set_record(&mut sim, vec![soil_held(year)]);
    sim.advance_minutes(DAY).expect("lives");
    let s = sim.people().polities[0].sewers[0].clone();
    assert_eq!(s.farm, Some(f));
    assert!(s.farm_kg > 0.0);
    // A harvest after it the season held back, and the household stops.
    set_record(
        &mut sim,
        vec![
            soil_held(year),
            HarvestRecord {
                year,
                kg_per_ha: 900.0,
                limit: Limit::Season,
            },
        ],
    );
    sim.advance_minutes(DAY).expect("lives");
    let s = sim.people().polities[0].sewers[0].clone();
    assert_eq!(
        (s.farm, s.farm_kg),
        (None, 0.0),
        "the soil no longer holds it back"
    );
}

#[test]
fn a_well_beside_a_farm_takes_its_load() {
    let (mut sim, _, homes, _, f, cell) = a_sewer_with_a_field(false, 10.0, Limit::Soil);
    let d = cholera(&sim);
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_day_for_tests(ctx, &[(homes[1], d, 10_000.0)]));
    let on_farm = sim.people().contagion.load(Node::Field(f), d);
    assert!(on_farm > 0.0, "the farm holds its share");
    // An open well a few metres from the field, its mouth no higher than the field's ground.
    let id = sim.allocate_id_for_tests();
    let (x, y) = corner_cm(&sim, cell);
    let ground = sim.map().elevation[cell as usize];
    let now = sim.now();
    sim.land_mut_for_tests().wells.list.push(Well {
        id,
        system: 0,
        cell,
        rect: RectCm {
            x: x + 1200,
            y,
            w: 200,
            h: 200,
        },
        household: homes[0],
        polity: None,
        ground_m: ground,
        depth_m: 6.0,
        target_m: 6.0,
        work_h: 100.0,
        skill_h: 0.0,
        quality: 0.8,
        loss: 0.0,
        mend_h: 0.0,
        cover_h: 0.0,
        covered: false,
        worth: 0.0,
        level_m: f64::from(ground) - 3.0,
        level_at: now,
        drawn_l: 0.0,
        begun: now,
        state: WellState::Open { since: now },
        crew: (0, 0),
    });
    // A wet day: the farm washes into the well beside it, as a tip there would.
    sim.land_mut_for_tests().water.last_surplus_mm = 20.0;
    let day = sim.now().day_index();
    sim.with_ctx_for_tests(|pop, ctx| pop.contagion_day_for_tests(ctx));
    let c = &sim.people().contagion;
    let washed: f64 = c
        .moves
        .iter()
        .filter(|m| {
            m.day == day
                && m.how == How::Runoff
                && m.from == Node::Field(f)
                && m.to == Node::Well(id)
        })
        .map(|m| m.amount)
        .sum();
    eprintln!("{on_farm:.0} doses on the farm; {washed:.1} washed into the well beside it");
    assert!(washed > 0.0, "the well beside the farm takes load");
    assert!(c.load(Node::Well(id), d) > 0.0);
}

#[test]
fn with_no_field_below_the_last_connection_there_is_no_farm() {
    let (mut sim, line, homes, _, _, _) = a_sewer_with_a_field(false, 20.0, Limit::Soil);
    assert!(
        sim.people().polities[0].sewers[0].farm.is_some(),
        "below the lowest connection, a farm"
    );
    // A connected home moves beside the outfall: no field lies below it.
    let last = centre(&sim, line[line.len() - 2]);
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == homes[1] {
            x.home = last;
        }
    }
    let d = cholera(&sim);
    sim.with_ctx_for_tests(|pop, ctx| pop.sewer_day_for_tests(ctx, &[(homes[1], d, 500.0)]));
    let s = sim.people().polities[0].sewers[0].clone();
    assert_eq!((s.farm, s.farm_kg), (None, 0.0));
    let reach = sim.people().contagion.load(Node::Reach(s.reach), d);
    assert!(
        (reach - 500.0).abs() < 1e-9,
        "all of it to the river: {reach}"
    );
}
