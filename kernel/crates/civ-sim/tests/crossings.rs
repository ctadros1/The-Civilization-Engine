//! Crossings over water (M5c slice AW, step one): an open crossing's span is walked at its deck's
//! speed, so a river too big to wade is crossed; rot takes its members, and one that can no
//! longer carry its own weight, or someone stepping onto it, gives way, dropping whoever is on it
//! into the water; and no route crosses it once it has. Households build logs where their own
//! wades repay them (step two), and a village where all of them together would (step three).

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::bridge::{margin, size_members};
use civ_agents::fords::Wade;
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome, PolicyKind};
use civ_agents::{Cause, ChronicleKind, CrossingStep};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::crossings::{Collapse, Crossing, CrossingOwner, CrossingState};
use civ_sim::{NewWorld, Sim, persist};
use civ_world::nav::RouteResult;
use civ_world::{WATER_LAND, WATER_RIVER};

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
            name: "Crossing".to_owned(),
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

/// A place a log could cross: a cell of a river too big to wade, with dry land on either side
/// of it east and west or north and south, nearest the hearth. Returns the two banks, the water
/// cell and the river's width.
fn site(sim: &Sim) -> ([u32; 2], u32, f32) {
    let map = sim.map();
    let ford = sim.nav().params().ford_max_discharge_m3s;
    let hearth = sim.land().settlements[0].hearth_m;
    let w = map.width;
    let mut best: Option<(f32, [u32; 2], u32, f32)> = None;
    for reach in map
        .reaches
        .iter()
        .filter(|r| f64::from(r.discharge_m3s) > ford)
    {
        for &c in &reach.cells {
            if map.water[c as usize] != WATER_RIVER || c % w == 0 || c % w == w - 1 || c < w {
                continue;
            }
            let land = |x: u32| map.water.get(x as usize) == Some(&WATER_LAND);
            for banks in [[c - 1, c + 1], [c - w, c + w]] {
                if banks.iter().all(|&b| land(b)) {
                    let (x, y) = cell_centre(sim, c);
                    let d = (x - hearth.0).hypot(y - hearth.1);
                    if best.is_none_or(|b| d < b.0) {
                        best = Some((d, banks, c, reach.width_m));
                    }
                }
            }
        }
    }
    let (_, banks, cell, width) = best.expect("a river too big to wade, with banks");
    (banks, cell, width)
}

/// Lays a log crossing at the site, open since today, owned by the first household, with its
/// logs' rot at `loss`.
fn lay(sim: &mut Sim, loss: f32) -> (PermanentId, [u32; 2], u32) {
    let (banks, cell, width) = site(sim);
    let system = sim
        .rules()
        .catalog
        .bridge_index("core:bridge/log_beam")
        .expect("the core content has a log footbridge");
    let def = sim.rules().catalog.bridges[system].clone();
    let timber = sim.rules().catalog.goods[def.good]
        .timber
        .expect("timber strengths");
    let span = width.clamp(def.span_m.0 as f32, def.span_m.1 as f32);
    let d = size_members(&def, &timber, f64::from(span)).expect("a log spans it");
    let id = sim.allocate_id_for_tests();
    let household = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h.id)
        .min()
        .expect("a household");
    let now = sim.now();
    sim.land_mut_for_tests().crossings.list.push(Crossing {
        id,
        system: system as u16,
        banks,
        cells: vec![cell],
        span_m: span,
        members: def.members as u8,
        diameter_cm: d as f32,
        quality: 1.0,
        loss,
        owner: CrossingOwner::Household(household),
        labour_h: 0.0,
        skill_h: 0.0,
        begun: now,
        state: CrossingState::Open {
            since: now.day_index(),
        },
        shares: Vec::new(),
    });
    sim.land_mut_for_tests().crossings.changed();
    sim.lay_decks_for_tests();
    (id, banks, cell)
}

/// The route between two cells on the walking grid as it stands, with nobody's trails.
fn route(sim: &Sim, from: u32, to: u32) -> Option<(Vec<u32>, f32)> {
    match sim.nav().route(
        &sim.map().elevation,
        from as usize,
        to as usize,
        &|_| 0.0,
        2_000_000,
    ) {
        RouteResult::Found(r) => {
            let secs = r.seconds.last().copied().unwrap_or(0.0);
            Some((r.cells, secs))
        }
        _ => None,
    }
}

/// The chronicle's crossing entries.
fn crossing_entries(sim: &Sim) -> Vec<(f64, String)> {
    sim.people()
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::Crossing)
        .map(|e| (e.number, e.name.clone()))
        .collect()
}

/// The first person by id, put on bank `from`.
fn walker_at(sim: &mut Sim, from: (f32, f32)) -> PermanentId {
    let walker = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p.id)
        .min()
        .expect("someone");
    let p = sim
        .people_mut_for_tests()
        .people
        .iter_mut()
        .map(|(_, p)| p)
        .find(|p| p.id == walker)
        .expect("alive");
    p.trip = None;
    p.pos = from;
    walker
}

#[test]
fn an_open_crossing_is_walked_over_and_once_it_gives_way_no_route_crosses_it() {
    let mut sim = village(3);
    sim.advance_minutes(DAY).expect("lives");
    let (banks, cell, _) = site(&sim);
    // A river too big to wade: nobody stands in it, and the way round is long or none.
    assert!(!sim.nav().walkable(cell as usize));
    let before = route(&sim, banks[0], banks[1]);
    let (id, banks, cell) = lay(&mut sim, 0.0);
    assert!(sim.nav().walkable(cell as usize));
    let (cells, secs) = route(&sim, banks[0], banks[1]).expect("across the crossing");
    assert!(cells.contains(&cell));
    if let Some((_, around)) = before {
        assert!(secs < around, "across {secs} s, around {around} s");
    }

    // Someone on one bank walks to the other over it: new logs carry them.
    let (from, to) = (cell_centre(&sim, banks[0]), cell_centre(&sim, banks[1]));
    let walker = walker_at(&mut sim, from);
    assert!(sim.with_ctx_for_tests(|pop, ctx| pop.walk_for_tests(ctx, walker, to)));
    assert!(sim.land().crossings.list[0].open());

    // Rot past what the logs can bear of their own weight, a minute before midnight, when
    // nobody is setting off over it: at midnight it gives way.
    let to_midnight = DAY - sim.now().minutes().rem_euclid(DAY);
    sim.advance_minutes(to_midnight - 1).expect("lives");
    sim.land_mut_for_tests().crossings.list[0].loss = 0.95;
    sim.advance_minutes(2).expect("lives");
    let c = &sim.land().crossings.list[0];
    assert_eq!(c.id, id);
    assert!(
        matches!(
            c.state,
            CrossingState::Failed {
                why: Collapse::OwnWeight,
                ..
            }
        ),
        "{:?}",
        c.state
    );
    assert!(!sim.nav().walkable(cell as usize), "the river as it was");
    if let Some((cells, _)) = route(&sim, banks[0], banks[1]) {
        assert!(!cells.contains(&cell));
    }
    let entries = crossing_entries(&sim);
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0].0, f64::from(CrossingStep::Failed as u8));
    assert!(
        entries[0]
            .1
            .contains("could no longer carry their own weight"),
        "{}",
        entries[0].1
    );
    assert!(sim.land().crossings.problems().is_empty());
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn a_rotten_crossing_gives_way_under_the_one_who_steps_onto_it() {
    let mut sim = village(3);
    sim.advance_minutes(DAY).expect("lives");
    // Rot that leaves the logs able to carry themselves but not a walker as well.
    let (_, cell, width) = site(&sim);
    let system = sim
        .rules()
        .catalog
        .bridge_index("core:bridge/log_beam")
        .expect("a log footbridge");
    let def = sim.rules().catalog.bridges[system].clone();
    let timber = sim.rules().catalog.goods[def.good].timber.expect("timber");
    let span = f64::from(width.clamp(def.span_m.0 as f32, def.span_m.1 as f32));
    let d = size_members(&def, &timber, span).expect("a log spans it");
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if margin(&def, &timber, span, def.members, d, 1.0, mid, 1) < 1.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let loss = hi + 1e-6;
    assert!(margin(&def, &timber, span, def.members, d, 1.0, loss, 0) >= 1.0);
    let (_, banks, _) = lay(&mut sim, loss as f32);
    let (from, to) = (cell_centre(&sim, banks[0]), cell_centre(&sim, banks[1]));
    let walker = walker_at(&mut sim, from);
    let went = sim.with_ctx_for_tests(|pop, ctx| pop.walk_for_tests(ctx, walker, to));
    assert!(!went, "it gave way under them, and they did not cross");
    let c = &sim.land().crossings.list[0];
    assert!(
        matches!(
            c.state,
            CrossingState::Failed {
                why: Collapse::UnderWalker,
                ..
            }
        ),
        "{:?}",
        c.state
    );
    assert!(!sim.nav().walkable(cell as usize));
    let entries = crossing_entries(&sim);
    let name = sim.people().name_of(walker);
    assert!(
        entries[0]
            .1
            .contains(&format!("{name} fell into the water")),
        "{}",
        entries[0].1
    );
    // They died of the fall, or are where they stepped on.
    let died = sim
        .people()
        .records
        .get(&walker)
        .and_then(|r| r.died)
        .map(|(_, cause)| cause);
    match died {
        Some(cause) => {
            assert_eq!(cause, Cause::Fell);
            assert!(entries[0].1.contains("died"), "{}", entries[0].1);
        }
        None => {
            let p = sim.people().person(walker).expect("alive");
            assert_eq!(p.pos, from);
        }
    }
    saves_and_goes_on_alike(&mut sim, DAY);
}

/// A stream a log could cross as a household would weigh it: a cell of a river narrow enough to
/// wade whose channel is within the log footbridge's spans, with dry land either side of it,
/// nearest the hearth.
fn ford(sim: &Sim) -> u32 {
    let map = sim.map();
    let ford = sim.nav().params().ford_max_discharge_m3s;
    let system = sim
        .rules()
        .catalog
        .bridge_index("core:bridge/log_beam")
        .expect("a log footbridge");
    let span = sim.rules().catalog.bridges[system].span_m;
    let hearth = sim.land().settlements[0].hearth_m;
    let w = map.width;
    let land = |x: u32| map.water.get(x as usize) == Some(&WATER_LAND);
    let mut best: Option<(f32, u32)> = None;
    for reach in map.reaches.iter().filter(|r| {
        f64::from(r.discharge_m3s) <= ford && (span.0..=span.1).contains(&f64::from(r.width_m))
    }) {
        for &c in &reach.cells {
            if map.water[c as usize] != WATER_RIVER || c % w == 0 || c % w == w - 1 || c < w {
                continue;
            }
            if [[c - 1, c + 1], [c - w, c + w]]
                .iter()
                .any(|b| b.iter().all(|&x| land(x)))
            {
                let (x, y) = cell_centre(sim, c);
                let d = (x - hearth.0).hypot(y - hearth.1);
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, c));
                }
            }
        }
    }
    let (d, cell) = best.expect("a stream a log could cross");
    assert!(d < 1000.0, "the nearest is {d} m from the hearth");
    cell
}

/// Household `household` remembers its people wading `cell` `wades` times, as of today.
fn remember(sim: &mut Sim, household: PermanentId, cell: u32, wades: f32) {
    let day = sim.now().day_index();
    let list = sim
        .people_mut_for_tests()
        .fords
        .households
        .entry(household)
        .or_default();
    list.retain(|w| w.cell != cell);
    let at = list.partition_point(|w| w.cell < cell);
    list.insert(at, Wade { cell, wades, day });
}

#[test]
fn a_household_that_wades_a_stream_often_builds_a_log_over_it_and_walks_it_once_open() {
    // World 4's hearth has a stream a log could cross within a walk (in eight of the first
    // sixteen river valleys there is none at all).
    let mut sim = village(4);
    sim.advance_minutes(DAY).expect("lives");
    // What households remember wading is river.
    for list in sim.people().fords.households.values() {
        assert!(
            list.iter()
                .all(|w| sim.map().water[w.cell as usize] == WATER_RIVER)
        );
    }
    let cell = ford(&sim);
    assert!(sim.nav().walkable(cell as usize), "waded");
    let household = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| h.id)
        .min()
        .expect("a household");
    // Waded a few times: no log is worth its work.
    remember(&mut sim, household, cell, 5.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.review_crossing_for_tests(ctx, household));
    assert!(sim.land().crossings.list.is_empty());
    // Waded by its people every day, both ways: the walking a log saves over its life is more
    // than the work it takes, so the household begins one.
    remember(&mut sim, household, cell, 1500.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.review_crossing_for_tests(ctx, household));
    let c = sim.land().crossings.list.first().expect("begun").clone();
    assert_eq!(c.owner, CrossingOwner::Household(household));
    assert_eq!(c.cells, vec![cell]);
    assert!(matches!(c.state, CrossingState::Building { work_h } if work_h == 0.0));
    assert!((40.0..200.0).contains(&c.labour_h), "{}", c.labour_h);
    // A second review begins nothing more while it is being built.
    sim.with_ctx_for_tests(|pop, ctx| pop.review_crossing_for_tests(ctx, household));
    assert_eq!(sim.land().crossings.list.len(), 1);
    // Its people work on it until it opens.
    for _ in 0..120 {
        sim.advance_minutes(DAY).expect("lives");
        if sim.land().crossings.list[0].open() {
            break;
        }
    }
    let c = &sim.land().crossings.list[0];
    assert!(c.open(), "{:?}", c.state);
    assert!(c.quality > 0.0 && c.quality <= 1.0);
    assert!(c.skill_h > 0.0);
    assert_eq!(sim.nav().revision(), sim.land().crossings.revision());
    let entries = crossing_entries(&sim);
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0].0, f64::from(CrossingStep::Opened as u8));
    assert!(entries[0].1.contains("log footbridge"), "{}", entries[0].1);
    // Walked on its deck now: nobody wades it.
    let (banks, _) = (c.banks, c.cells.clone());
    let route_cells = route(&sim, banks[0], banks[1]).expect("across").0;
    assert!(route_cells.contains(&cell));
    assert!(sim.land().crossings.problems().is_empty());
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn a_crossing_the_gathering_asks_for_is_built_by_the_households_shares_of_its_work() {
    // World 4's stream, waded by every household of the village but one some 400 times a year:
    // about two hours of walking a year each, too little for any one household to repay a log's
    // work alone, but more than enough for all of them together.
    let mut sim = village(4);
    sim.advance_minutes(DAY).expect("lives");
    let cell = ford(&sim);
    let settlement = sim.land().settlements[0].id;
    let mut households: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| h.id)
        .collect();
    households.sort_unstable();
    assert!(households.len() >= 3, "{} households", households.len());
    let (dry, wading) = households.split_last().expect("households");
    for &h in wading {
        remember(&mut sim, h, cell, 300.0);
    }
    // No household begins one of its own.
    for &h in &households {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_crossing_for_tests(ctx, h));
    }
    assert!(sim.land().crossings.list.is_empty());
    // The settlement's people could build one there together: a log footbridge.
    let site = sim
        .with_ctx_for_tests(|pop, ctx| pop.public_sites_for_tests(ctx, settlement))
        .into_iter()
        .find(|s| s.cell == cell)
        .expect("a site at the ford");
    assert!((40.0..200.0).contains(&site.labour_h), "{}", site.labour_h);
    // A law to build it would bring each household that wades it more than the one that does
    // not, which would only give its share of the work. In food, what an hour of their work
    // brings, the walking is little either way.
    let gains =
        sim.with_ctx_for_tests(|pop, ctx| pop.crossing_gains_for_tests(ctx, settlement, cell));
    let gain = |h: PermanentId| gains.iter().find(|g| g.0 == h).expect("weighed").1;
    assert!(gain(*dry) < 0.0, "{gains:?}");
    assert!(wading.iter().all(|&h| gain(h) > gain(*dry)), "{gains:?}");
    // The gathering passed it: a law in force, known to every adult, its site recorded; the
    // polity begins the crossing.
    let polity = sim.people().polities[0].id;
    let adults: Vec<PermanentId> = sim
        .people()
        .people
        .iter()
        .filter(|(_, p)| p.age_years(sim.now()) >= 16.0)
        .map(|(_, p)| p.id)
        .collect();
    let policy = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.kind == PolicyKind::BuildCrossing)
        .expect("the template") as u16;
    let id = sim.allocate_id_for_tests();
    let (now, today) = (sim.now(), sim.now().day_index());
    {
        let pop = sim.people_mut_for_tests();
        let p = &mut pop.polities[0];
        p.laws.push(Law {
            id,
            policy,
            kind: PolicyKind::BuildCrossing,
            levy_share: 0.0,
            holder: None,
            relief_days: 0.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: adults[0],
            proposed: now,
            issue: IssueKind::Fords,
            meets_day: today,
            decided: Some(now),
            outcome: Some(Outcome::Passed),
            eligible: 0,
            stances: Vec::new(),
            known: adults.iter().map(|&a| (a, today)).collect(),
            compliance: Default::default(),
            watch: Default::default(),
            body: None,
            ends: None,
            agreement: None,
        });
        p.sites.push((id, site));
    }
    let words = sim.people().polities[0].words_of(
        &sim.people().polities[0]
            .laws
            .last()
            .expect("the law")
            .clone(),
        &sim.rules().catalog.policies,
        &|_| String::new(),
    );
    assert!(words.contains("each household doing its share"), "{words}");
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_public_crossing_for_tests(ctx, polity, site));
    let c = sim.land().crossings.list.first().expect("begun").clone();
    assert_eq!(c.owner, CrossingOwner::Polity(polity));
    assert_eq!(c.cells, vec![cell]);
    // Its people give their households' shares as they choose, until it opens; the household
    // that never wades it gives too, for the gathering's word.
    for _ in 0..120 {
        sim.advance_minutes(DAY).expect("lives");
        if sim.land().crossings.list[0].open() {
            break;
        }
    }
    let c = sim.land().crossings.list[0].clone();
    assert!(c.open(), "{:?}, shares {:?}", c.state, c.shares);
    assert!(c.shares.len() >= 2, "{:?}", c.shares);
    let given: f32 = c.shares.iter().map(|s| s.1).sum();
    assert!(
        (given - c.labour_h).abs() < 1e-2,
        "{given} of {}",
        c.labour_h
    );
    let entries = crossing_entries(&sim);
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert!(
        entries[0].1.contains("their gathering agreed to build"),
        "{}",
        entries[0].1
    );
    // Its deck is walked; no household begins one of its own there, nor beside it, where
    // walkers would take its deck: not even one that waded the next cell of the stream daily.
    assert!(
        route(&sim, c.banks[0], c.banks[1])
            .expect("across")
            .0
            .contains(&cell)
    );
    let w = sim.map().width;
    let beside = [cell - w, cell + w, cell - 1, cell + 1]
        .into_iter()
        .find(|&b| sim.map().water[b as usize] == WATER_RIVER && !c.banks.contains(&b));
    if let Some(b) = beside {
        remember(&mut sim, households[0], b, 1500.0);
    }
    for &h in &households {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_crossing_for_tests(ctx, h));
    }
    assert_eq!(sim.land().crossings.list.len(), 1);
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.public_sites_for_tests(ctx, settlement))
            .iter()
            .all(|s| s.cell != cell && Some(s.cell) != beside)
    );
    assert!(sim.land().crossings.problems().is_empty());
    saves_and_goes_on_alike(&mut sim, DAY);
}

/// The centre of a terrain cell, metres.
fn cell_centre(sim: &Sim, cell: u32) -> (f32, f32) {
    civ_agents::population::cell_centre(sim.map(), cell as usize)
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "cross").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(sim, &loaded);
    assert_eq!(sim.nav().revision(), loaded.nav().revision());
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}
