//! The store's keeper (M7c slice BY, ADR-0026 §4): one who keeps a common store may draw from it
//! for their own household, weighed against their objection to taking what is not theirs, and
//! never above its filter; what they draw moves on its own channel and is counted on the store's
//! law; a low store's keeper passes over one they regard too little, who holds it against the
//! office, and answers one they favour; a world with a diversion saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome, PolicyKind};
use civ_agents::ties::Act;
use civ_agents::word::{Blamed, Wrong};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim, persist};

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

/// A village of world 4 a day old, whose polity keeps a common store of `kg` grain, with relief
/// of five days an ask, kept by the eldest adult of its largest household, everyone knowing both
/// laws. Returns the world, the polity, the keeper, the store's law and grain's good index.
fn village(kg: f64) -> (Sim, PermanentId, PermanentId, PermanentId, usize) {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Granary".to_owned(),
            seed: 4,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [5; 16],
    )
    .expect("generates");
    sim.advance_minutes(DAY).expect("lives");
    let policies = sim.rules().catalog.policies.clone();
    let of = |kind: PolicyKind| {
        policies
            .iter()
            .position(|p| p.kind == kind)
            .expect("in core") as u16
    };
    let grain = sim.rules().catalog.crops[0].good;
    let goods = sim.rules().catalog.goods.len();
    let ids = [sim.allocate_id_for_tests(), sim.allocate_id_for_tests()];
    let now = sim.now();
    let pop = sim.people_mut_for_tests();
    let mut households: Vec<(usize, PermanentId)> = pop
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| (h.members.len(), h.id))
        .collect();
    households.sort_unstable_by(|a, b| b.cmp(a));
    let keeper = pop.household(households[0].1).expect("household").members[0];
    let everyone: Vec<PermanentId> = pop.people.iter().map(|(_, p)| p.id).collect();
    let law = |id: PermanentId, kind: PolicyKind, holder: Option<PermanentId>| {
        let mut l = Law {
            id,
            policy: of(kind),
            kind,
            levy_share: 0.0,
            holder,
            relief_days: if kind == PolicyKind::CommonStore {
                5.0
            } else {
                0.0
            },
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: keeper,
            proposed: now,
            issue: IssueKind::FoodShort,
            meets_day: now.day_index(),
            decided: Some(now),
            outcome: Some(Outcome::Passed),
            eligible: 0,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Default::default(),
            watch: Default::default(),
            body: None,
            ends: None,
            agreement: None,
            target: None,
        };
        for &m in &everyone {
            l.learn(m, now.day_index());
        }
        l
    };
    let p = &mut pop.polities[0];
    let polity = p.id;
    p.laws.push(law(ids[0], PolicyKind::CommonStore, None));
    p.laws
        .push(law(ids[1], PolicyKind::KeepStore, Some(keeper)));
    p.stores.resize(goods, 0.0);
    p.stores[grain] = kg;
    p.stores_at = now;
    (sim, polity, keeper, ids[0], grain)
}

/// Person `p`'s objection to taking, set.
fn object(sim: &mut Sim, p: PermanentId, objection: f32) {
    let pop = sim.people_mut_for_tests();
    let person = pop
        .people
        .iter_mut()
        .map(|(_, x)| x)
        .find(|x| x.id == p)
        .expect("person");
    person.objection = objection;
    person.risk_seen = 0.0;
}

/// The household of person `p`, its stores emptied of food.
fn empty_home(sim: &mut Sim, p: PermanentId) -> PermanentId {
    let goods: Vec<bool> = sim
        .rules()
        .catalog
        .goods
        .iter()
        .map(|g| g.kcal_per_kg > 0.0)
        .collect();
    let pop = sim.people_mut_for_tests();
    let h = pop.person(p).expect("person").household;
    let x = pop
        .households
        .iter_mut()
        .map(|(_, x)| x)
        .find(|x| x.id == h)
        .expect("household");
    for (s, food) in x.stores.iter_mut().zip(&goods) {
        if *food {
            *s = 0.0;
        }
    }
    h
}

fn store_law(sim: &Sim, id: PermanentId) -> Law {
    sim.people().polities[0]
        .laws
        .iter()
        .find(|l| l.id == id)
        .cloned()
        .expect("kept")
}

/// Law `id`'s conduct in the government panel's words.
fn conduct_of(sim: &Sim, id: PermanentId) -> String {
    let payload = civ_sim::frames::government::government_response(sim);
    let info = civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&payload)
        .expect("a response")
        .body_as_government()
        .expect("the government");
    info.polities()
        .expect("polities")
        .iter()
        .flat_map(|p| p.laws().expect("laws").iter())
        .find(|x| x.id() == id.get())
        .and_then(|x| x.conduct())
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn a_keeper_with_no_objection_draws_from_the_store_and_one_who_objects_never_does() {
    let (mut sim, polity, keeper, law, grain) = village(2000.0);
    object(&mut sim, keeper, 0.0);
    let home = empty_home(&mut sim, keeper);
    let before = sim.people().household(home).expect("home").stores[grain];
    let drawn = sim.with_ctx_for_tests(|pop, ctx| pop.keeper_weighs_drawing_for_tests(ctx, polity));
    assert!(drawn > 0.0, "drew {drawn} kg");
    let c = store_law(&sim, law).compliance;
    assert_eq!(c.diverted, 1);
    assert!((c.diverted_kg - drawn).abs() < 1e-6);
    // At most the share of what it held.
    let share = sim.rules().people.crime.divert_share;
    assert!(drawn <= share * 2000.0 + 1e-6);
    assert!(sim.people().polities[0].stores[grain] < 2000.0);
    let after = sim.people().household(home).expect("home").stores[grain];
    assert!((after - before - drawn).abs() < 1e-6, "{before} -> {after}");
    let told = conduct_of(&sim, law);
    assert!(
        told.starts_with("its keeper drew") && told.ends_with("1 time"),
        "{told}"
    );
    // Above the objection's filter, never.
    let (mut sim, polity, keeper, law, _) = village(2000.0);
    let filter = sim.rules().people.crime.objection_filter as f32;
    object(&mut sim, keeper, filter);
    empty_home(&mut sim, keeper);
    for _ in 0..20 {
        sim.with_ctx_for_tests(|pop, ctx| pop.keeper_weighs_drawing_for_tests(ctx, polity));
    }
    assert_eq!(store_law(&sim, law).compliance.diverted, 0);
}

#[test]
fn a_low_store_s_keeper_passes_over_one_they_regard_too_little_and_answers_one_they_favour() {
    // Well below the reserve: a few days' food for the village.
    let (mut sim, polity, keeper, law, _) = village(30.0);
    sim.rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .crime
        .favour_floor = 0.3;
    let keeper_home = sim.people().person(keeper).expect("keeper").household;
    let others: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| h.id != keeper_home && !h.members.is_empty())
        .map(|(_, h)| h.members[0])
        .collect();
    let (stranger, friend) = (others[0], others[1]);
    let stranger_home = empty_home(&mut sim, stranger);
    let friend_home = empty_home(&mut sim, friend);
    // The keeper keeps company with the friend's household's elder.
    let day = sim.now().day_index();
    let tp = sim.rules().people.ties.clone();
    let elder = sim.people().household(friend_home).expect("home").members[0];
    let pop = sim.people_mut_for_tests();
    for _ in 0..40 {
        pop.ties
            .record(keeper, elder, Act::Hearth, 4.0, 0.0, day, &tp);
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.give_relief_for_tests(ctx, polity, stranger_home));
    let c = store_law(&sim, law).compliance;
    assert_eq!(c.passed_over, 1, "{c:?}");
    assert_eq!(c.relieved, 0);
    let held = sim
        .people()
        .word
        .grievances_of(stranger)
        .any(|g| g.wrong == Wrong::PassedOver && matches!(g.blamed, Blamed::Office(_)));
    assert!(held, "the one passed over holds it against the office");
    sim.with_ctx_for_tests(|pop, ctx| pop.give_relief_for_tests(ctx, polity, friend_home));
    let c = store_law(&sim, law).compliance;
    assert_eq!((c.favoured, c.relieved), (1, 1), "{c:?}");
}

#[test]
fn a_world_with_a_diversion_saves_and_loads_exactly() {
    let (mut sim, polity, keeper, _, _) = village(2000.0);
    object(&mut sim, keeper, 0.0);
    empty_home(&mut sim, keeper);
    sim.with_ctx_for_tests(|pop, ctx| pop.keeper_weighs_drawing_for_tests(ctx, polity));
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "keeper",
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
    sim.advance_minutes(2 * DAY).expect("lives");
    loaded.advance_minutes(2 * DAY).expect("lives");
    same(&sim, &loaded);
}
