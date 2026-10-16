//! Patron gifts (M7c slice BY, ADR-0026 §4): a sponsor whose law would fail gives food to
//! households a gift would bring round, the reason recorded, and their members' stances gain what
//! their household owes; nobody gives for a law that would pass, or where nothing is owed for
//! gifts; a patron who saw a client stand against them gives it nothing more; a world with patron
//! gifts saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::polity::{
    Gathering, IssueKind, Law, LawStatus, Outcome, PolicyKind, Stance, StanceRecord,
};
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

struct Scene {
    sim: Sim,
    polity: PermanentId,
    sponsor: PermanentId,
    giver: PermanentId,
}

/// A village of world 4 a day old; the eldest of its largest household, which holds `kg` more
/// grain, is to sponsor laws.
fn village(kg: f64) -> Scene {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Patrons".to_owned(),
            seed: 4,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [6; 16],
    )
    .expect("generates");
    sim.advance_minutes(DAY).expect("lives");
    let grain = sim.rules().catalog.crops[0].good;
    let pop = sim.people_mut_for_tests();
    let mut households: Vec<(usize, PermanentId)> = pop
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| (h.members.len(), h.id))
        .collect();
    households.sort_unstable_by(|a, b| b.cmp(a));
    let giver = households[0].1;
    let x = pop
        .households
        .iter_mut()
        .map(|(_, x)| x)
        .find(|x| x.id == giver)
        .expect("household");
    x.stores[grain] += kg;
    let sponsor = x.members[0];
    let polity = pop.polities[0].id;
    Scene {
        sim,
        polity,
        sponsor,
        giver,
    }
}

/// A store law of the sponsor's put to a gathering meeting tomorrow, and everyone told, every
/// household reckoning it at `others` points and the sponsor's at `mine`.
fn put(sc: &mut Scene, mine: f32, others: f32) -> PermanentId {
    put_with(sc, mine, others, &[])
}

/// As [`put`], each of `special`'s households reckoning it as given.
fn put_with(sc: &mut Scene, mine: f32, others: f32, special: &[(PermanentId, f32)]) -> PermanentId {
    let policy = sc
        .sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|p| p.kind == PolicyKind::CommonStore)
        .expect("in core") as u16;
    let id = sc.sim.allocate_id_for_tests();
    let now = sc.sim.now();
    let (sponsor, giver) = (sc.sponsor, sc.giver);
    let params = sc.sim.rules().people.clone();
    let pop = sc.sim.people_mut_for_tests();
    let mut stakes: Vec<(PermanentId, f32)> = pop
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| {
            let stake = special
                .iter()
                .find(|s| s.0 == h.id)
                .map_or(if h.id == giver { mine } else { others }, |s| s.1);
            (h.id, stake)
        })
        .collect();
    stakes.sort_unstable_by_key(|s| s.0);
    let meets = now.day_index() + 1;
    let p = &mut pop.polities[0];
    p.laws.push(Law {
        id,
        policy,
        kind: PolicyKind::CommonStore,
        levy_share: 0.1,
        holder: None,
        relief_days: 0.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::Proposed,
        sponsor,
        proposed: now,
        issue: IssueKind::FoodShort,
        meets_day: meets,
        decided: None,
        outcome: None,
        eligible: 0,
        stances: Vec::new(),
        known: vec![(sponsor, now.day_index())],
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
        target: None,
    });
    p.gathering = Some(Gathering {
        law: Some(id),
        day: meets,
        stakes,
        present: Vec::new(),
        cases: Vec::new(),
    });
    pop.hear_of_gatherings_called(now, &params);
    id
}

fn gives(sc: &mut Scene, law: PermanentId) {
    let polity = sc.polity;
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.patron_gives_for_tests(ctx, polity, law));
}

fn the_law(sim: &Sim, id: PermanentId) -> Law {
    sim.people().polities[0]
        .laws
        .iter()
        .find(|l| l.id == id)
        .cloned()
        .expect("kept")
}

fn food(sim: &Sim, h: PermanentId) -> f64 {
    let goods = &sim.rules().catalog.goods;
    let x = sim.people().household(h).expect("household");
    x.stores
        .iter()
        .zip(goods)
        .map(|(s, g)| s.max(0.0) * g.kcal_per_kg)
        .sum()
}

#[test]
fn a_sponsor_whose_law_would_fail_gives_food_to_households_it_brings_round() {
    let mut sc = village(3000.0);
    let law = put(&mut sc, 5.0, -0.3);
    let before: Vec<(PermanentId, f64)> = sc
        .sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, food(&sc.sim, h.id)))
        .collect();
    gives(&mut sc, law);
    let gifts = sc.sim.people().polities[0].patron_gifts.clone();
    assert!(!gifts.is_empty(), "the sponsor gave");
    let pp = sc.sim.rules().people.polity.clone();
    for g in &gifts {
        assert_eq!((g.patron, g.giver, g.law), (sc.sponsor, sc.giver, law));
        assert_ne!(g.client, sc.giver);
        assert!(
            g.days > 0.0 && f64::from(g.days) <= pp.patron_gift_days + 0.1,
            "{g:?}"
        );
        let was = before.iter().find(|b| b.0 == g.client).expect("was").1;
        assert!(food(&sc.sim, g.client) > was, "the client holds more food");
    }
    let was = before.iter().find(|b| b.0 == sc.giver).expect("was").1;
    assert!(food(&sc.sim, sc.giver) < was, "the patron holds less");
    // At the gathering, members of households given to stand with what they owe added.
    for _ in 0..4 * 48 {
        sc.sim.advance_minutes(30).expect("lives");
        if the_law(&sc.sim, law).decided.is_some() {
            break;
        }
    }
    let l = the_law(&sc.sim, law);
    assert!(l.decided.is_some(), "the gathering sat");
    let owed: Vec<&StanceRecord> = l.stances.iter().filter(|r| r.owed > 0.0).collect();
    assert!(!owed.is_empty(), "someone given to came: {:?}", l.stances);
    for r in &owed {
        let days: f32 = gifts
            .iter()
            .filter(|g| g.client == r.household)
            .map(|g| g.days)
            .sum();
        assert!(days > 0.0);
        assert!((r.owed - pp.w_owed as f32 * days).abs() < 1e-3, "{r:?}");
    }
    assert!(owed.iter().any(|r| r.stance == Stance::Support));
    // The government panel tells the gifts, and why those given to stood where they did.
    let payload = civ_sim::frames::government::government_response(&sc.sim);
    let info = civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&payload)
        .expect("a response")
        .body_as_government()
        .expect("the government");
    let line = info
        .polities()
        .expect("polities")
        .iter()
        .flat_map(|p| p.laws().expect("laws").iter())
        .find(|x| x.id() == law.get())
        .expect("the law's line");
    let conduct = line.conduct().expect("its conduct told");
    assert!(
        conduct.starts_with("before the gathering, ") && conduct.contains("gave"),
        "{conduct}"
    );
    let why = civ_agents::polity::stance_words(owed[0], pp.stance_margin, sc.sponsor);
    assert!(why.contains("owed the sponsor for gifts"), "{why}");
    // Nobody else owes anything.
    assert!(
        l.stances
            .iter()
            .filter(|r| !gifts.iter().any(|g| g.client == r.household))
            .all(|r| r.owed == 0.0)
    );
}

#[test]
fn nobody_gives_for_a_law_that_would_pass_or_where_nothing_is_owed_for_gifts() {
    let mut sc = village(3000.0);
    let law = put(&mut sc, 5.0, 1.0);
    gives(&mut sc, law);
    assert!(sc.sim.people().polities[0].patron_gifts.is_empty());
    let mut sc = village(3000.0);
    sc.sim
        .rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .polity
        .w_owed = 0.0;
    let law = put(&mut sc, 5.0, -0.3);
    gives(&mut sc, law);
    assert!(sc.sim.people().polities[0].patron_gifts.is_empty());
    // Nor a sponsor whose household the law is worth nothing to.
    let mut sc = village(3000.0);
    let law = put(&mut sc, 0.0, -0.3);
    gives(&mut sc, law);
    assert!(sc.sim.people().polities[0].patron_gifts.is_empty());
}

#[test]
fn a_patron_who_saw_a_client_stand_against_them_gives_it_nothing_more() {
    let mut sc = village(6000.0);
    let first = put(&mut sc, 5.0, -0.3);
    gives(&mut sc, first);
    let gifts = sc.sim.people().polities[0].patron_gifts.clone();
    let client = gifts.first().expect("a gift").client;
    // At the gathering on it, the patron came and a member of the client household opposed it.
    let (sponsor, giver, now) = (sc.sponsor, sc.giver, sc.sim.now());
    let member = sc.sim.people().household(client).expect("client").members[0];
    let stood = |person, household, stance| StanceRecord {
        person,
        household,
        stance,
        gain: 0.0,
        regard: 0.0,
        opinion: 0.0,
        values: 0.0,
        view: 0.0,
        owed: 0.0,
    };
    {
        let pop = sc.sim.people_mut_for_tests();
        let l = pop.polities[0].law_mut(first).expect("law");
        l.stances = vec![
            stood(sponsor, giver, Stance::Support),
            stood(member, client, Stance::Oppose),
        ];
        l.decided = Some(now);
        l.outcome = Some(Outcome::Failed);
        l.status = LawStatus::Rejected;
        pop.polities[0].gathering = None;
        // As if the patron had given to that household alone.
        pop.polities[0].patron_gifts.retain(|g| g.client == client);
    }
    // What it owes would carry it at -0.3; at -0.6 a gift would bring it round again.
    let second = put_with(&mut sc, 5.0, -0.3, &[(client, -0.6)]);
    gives(&mut sc, second);
    let later: Vec<_> = sc.sim.people().polities[0]
        .patron_gifts
        .iter()
        .filter(|g| g.law == second)
        .copied()
        .collect();
    assert!(!later.is_empty(), "the patron still gives to others");
    assert!(later.iter().all(|g| g.client != client), "{later:?}");
}

#[test]
fn a_world_with_patron_gifts_saves_and_loads_exactly() {
    let mut sc = village(3000.0);
    let law = put(&mut sc, 5.0, -0.3);
    gives(&mut sc, law);
    assert!(!sc.sim.people().polities[0].patron_gifts.is_empty());
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sc.sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "patron",
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
    same(&sc.sim, &loaded);
    // Past the gathering, its stances with what was owed.
    sc.sim.advance_minutes(2 * DAY).expect("lives");
    loaded.advance_minutes(2 * DAY).expect("lives");
    same(&sc.sim, &loaded);
    assert!(the_law(&loaded, law).decided.is_some());
}
