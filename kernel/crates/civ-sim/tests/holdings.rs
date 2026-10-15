//! A polity over several settlements (M6c slice BM, ADR-0023 §9): one polity takes another in;
//! the other's deciding closes and its record is kept; the settlement's residents become members
//! of the polity that took it; its store becomes that polity's; its agreements end; and all of it
//! saves and loads.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ChronicleKind;
use civ_agents::agreements::{Agreement, AgreementState, Clause};
use civ_agents::polity::{HeldHow, LawStatus};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
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

/// Two villages of `band`, founded apart and unknown to each other, with a fixed identity, a day
/// old.
fn two_villages(seed: u64, band: u32) -> Sim {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Holdings".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: band,
            neighbours: vec![band],
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [9; 16],
    )
    .expect("generates");
    sim.advance_minutes(DAY).expect("advances");
    sim
}

/// The index of the polity seated at settlement `s`.
fn seated(sim: &Sim, s: PermanentId) -> usize {
    sim.people()
        .polities
        .iter()
        .position(|p| p.settlement == s)
        .expect("its polity")
}

/// Saves `sim`, loads it, and checks every section is the same, and that the two go on alike for
/// `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "held").expect("saves");
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

#[test]
fn a_polity_taken_in_decides_no_more_and_its_people_are_the_takers_members() {
    let mut sim = two_villages(4, 30);
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (pi, qi) = (seated(&sim, a), seated(&sim, b));
    let (taker, taken) = (sim.people().polities[pi].id, sim.people().polities[qi].id);
    let grain = sim.rules().catalog.crops[0].good;
    let goods = sim.rules().catalog.goods.len();
    // The village to be taken in keeps a store, and has an agreement with the other.
    let agreement = sim.allocate_id_for_tests();
    let now = sim.now();
    {
        let pop = sim.people_mut_for_tests();
        let q = &mut pop.polities[qi];
        q.stores.resize(goods, 0.0);
        q.stores[grain] = 300.0;
        q.stores_at = now;
        let negotiators = [pop.polities[pi].id, pop.polities[qi].id];
        pop.agreements.list.push(Agreement {
            id: agreement,
            polities: [taker, taken],
            negotiators,
            clauses: vec![Clause::NoRaids { from: 0 }],
            term_days: 365,
            made: now.day_index(),
            laws: [None, None],
            passed: [None, None],
            heard: [None, None],
            state: AgreementState::Offered,
        });
    }
    let (fields, params) = (sim.land().fields.clone(), sim.rules().people.clone());
    let before = sim.people().body_members(&fields, pi, now, &params).len();
    let theirs = sim.people().body_members(&fields, qi, now, &params).len();
    assert!(before > 0 && theirs > 0);
    let held_before = sim.people().polities[pi]
        .stores
        .get(grain)
        .copied()
        .unwrap_or(0.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.take_in_for_tests(ctx, pi, qi, HeldHow::Annexed));

    let pop = sim.people();
    let (p, q) = (&pop.polities[pi], &pop.polities[qi]);
    // The one taken in decides no more, and its record names who took it.
    let end = q.ended.expect("its deciding closed");
    assert_eq!((end.by, end.how), (taker, HeldHow::Annexed));
    assert!(q.gathering.is_none());
    assert!(q.laws.iter().all(|l| l.status != LawStatus::InForce));
    assert!(!q.holds(Some(b)));
    // The taker holds both, and the far village's adults are its members.
    assert!(p.holds(Some(a)) && p.holds(Some(b)));
    assert_eq!(p.settlements(), vec![a, b]);
    assert_eq!(p.held[0].from, taken);
    assert_eq!(
        pop.body_members(&fields, pi, now, &params).len(),
        before + theirs,
        "membership is residence in any of its settlements"
    );
    // Its store is the taker's now.
    assert!(
        (p.stores[grain] - held_before - 300.0).abs() < 1.0,
        "{}",
        p.stores[grain]
    );
    assert!(q.stores.iter().all(|&kg| kg == 0.0));
    // Its agreements ended.
    let x = pop.agreements.get(agreement).expect("kept");
    assert!(matches!(x.state, AgreementState::Ended { by: Some(1), .. }));
    // The chronicle tells it.
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::PolityJoined && e.settlement == Some(b)),
        "the taking in is chronicled"
    );
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());

    // Days on, no polity is founded for the settlement taken in, and only the taker reviews.
    let laws_before = sim.people().polities[qi].laws.len();
    sim.advance_minutes(20 * DAY).expect("advances");
    let pop = sim.people();
    assert_eq!(
        pop.polities.len(),
        2,
        "no new polity for the settlement taken in"
    );
    assert_eq!(
        pop.polities[qi].laws.len(),
        laws_before,
        "it proposes nothing"
    );
    assert!(pop.polities[qi].gathering.is_none());
    assert!(pop.polities[pi].holds(Some(b)));
    saves_and_goes_on_alike(&mut sim, DAY);
}
