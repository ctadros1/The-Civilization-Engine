//! Property regimes (M3a slice K, ADR-0007): a world lives under the regime it was made under.
//! Under village tenure the settlement holds the ground its households break and gives it out by
//! need at its yearly review; under household tenure a household holds what it breaks, its
//! fields are divided among its heirs when it is no more, and ground nobody holds any more is
//! taken up by households short of land. Goods stay accounted for throughout.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::{Population, population};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::{FieldStage, Party};
use civ_sim::{NewWorld, Sim, SimError, persist};
use commons_persist::{SaveDir, SaveKind};

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

fn create(seed: u64, regime: &str) -> Result<Sim, SimError> {
    Sim::create(
        &NewWorld {
            name: "Tenure".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: regime.to_owned(),
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    )
}

/// A world of seed 3 under `regime` (empty: the default), lived until its households have
/// marked out at least `fields` fields.
fn with_fields(regime: &str, fields: usize) -> Sim {
    let mut sim = create(3, regime).expect("generates");
    for _ in 0..40 {
        if sim.land().fields.len() >= fields {
            break;
        }
        sim.advance_minutes(24 * 60).expect("advances");
    }
    assert!(
        sim.land().fields.len() >= fields,
        "only {} fields after 40 days",
        sim.land().fields.len()
    );
    sim
}

/// Living households with their member counts, by id.
fn households(sim: &Sim) -> Vec<(PermanentId, usize)> {
    let mut out: Vec<(PermanentId, usize)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, h.members.len()))
        .filter(|&(_, n)| n > 0)
        .collect();
    out.sort_unstable();
    out
}

/// The area a household of `members` working every field plans to crop, hectares: at the yield
/// those fields are expected to give (ADR-0012 §4).
fn need_ha(sim: &Sim, members: usize) -> f64 {
    let rules = sim.rules();
    let crop = &rules.catalog.crops[rules.people.farm.crop];
    let kcal = rules.catalog.goods[crop.good].kcal_per_kg;
    let day = sim.now().day_index();
    let expected = civ_agents::farm::expected_yield_kg_ha(sim.land().fields.iter(), crop, day);
    civ_agents::farm::need_area_ha(members, &rules.people, crop, kcal, expected)
}

fn worked_ha(sim: &Sim, household: PermanentId) -> f64 {
    sim.land()
        .fields
        .iter()
        .filter(|f| f.household == household)
        .map(|f| f.rect.area_ha())
        .sum()
}

/// Puts every field between crops (fallow, nothing done toward its next crop or waiting to be
/// threshed), as they are at a winter review.
fn between_crops(sim: &mut Sim) {
    for f in &mut sim.land_mut_for_tests().fields {
        f.stage = FieldStage::Fallow;
        f.work_h = 0.0;
        f.tended_h = 0.0;
        f.sheaves_kg = 0.0;
    }
}

fn held(pop: &Population) -> (Vec<f64>, civ_agents::person::Flows) {
    (pop.goods_held(), pop.flows())
}

/// Nothing is wrong with who holds and works the land.
fn sound_claims(sim: &Sim) {
    let problems = sim
        .people()
        .claims_problems(sim.land(), sim.regime(), sim.ids().peek_next());
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn broken_claims_are_found() {
    let mut sim = with_fields("core:regime/village", 4);
    sound_claims(&sim);
    let household = households(&sim)[0].0;
    let (regime, next, now) = (sim.regime().clone(), sim.ids().peek_next(), sim.now());
    let land = sim.land_mut_for_tests();
    land.fields[0].holder = Party::Household(household);
    land.fields[1].lease = Some(civ_land::Lease {
        since: now,
        until: now,
        holder_share: 0.25,
    });
    land.fields[2].holder = Party::Settlement(PermanentId::from_raw(next + 5).expect("an id"));
    let problems = sim.people().claims_problems(sim.land(), &regime, next);
    for words in [
        "is held by household",
        "which lets no land",
        "is let by no other household",
        "which is not in the world",
    ] {
        assert!(
            problems.iter().any(|p| p.contains(words)),
            "{words}: {problems:?}"
        );
    }
}

#[test]
fn a_world_keeps_the_regime_it_was_made_under() {
    let sim = create(3, "").expect("generates");
    assert_eq!(sim.regime().id, "core:regime/household", "the default");
    assert_eq!(sim.meta().regime_id, "core:regime/household");
    let sim = create(3, "core:regime/village").expect("generates");
    assert_eq!(sim.regime().id, "core:regime/village");
    let mut sim = sim;
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "tenure").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(
        loaded.regime().id,
        "core:regime/village",
        "kept in its saves"
    );
    assert!(matches!(
        create(3, "core:regime/feudal"),
        Err(SimError::UnknownRegime(_))
    ));
}

#[test]
fn village_fields_are_held_by_the_settlement_and_given_out_by_need() {
    let mut sim = with_fields("core:regime/village", 8);
    let settlement = sim.land().settlements[0].id;
    assert!(
        sim.land()
            .fields
            .iter()
            .all(|f| f.holder == Party::Settlement(settlement)),
        "the village holds the ground its households break"
    );
    // Before the review, the smallest household works every field, all of them between crops.
    let all = households(&sim);
    let (rich, members) = *all
        .iter()
        .min_by_key(|&&(id, n)| (n, id))
        .expect("households");
    let need = need_ha(&sim, members);
    let day = sim.now().day_index();
    sim.regime_mut_for_tests().review_day = ((day + 1).rem_euclid(365)) as u16;
    // Just before midnight, when nobody is in the fields.
    let to_midnight = 24 * 60 - sim.now().minute_of_day();
    sim.advance_minutes(to_midnight - 1).expect("advances");
    between_crops(&mut sim);
    for f in &mut sim.land_mut_for_tests().fields {
        f.household = rich;
    }
    let total: f64 = sim.land().fields.iter().map(|f| f.rect.area_ha()).sum();
    assert!(total > need + 0.25, "{total} ha against a need of {need}");
    let start = held(sim.people());
    sim.advance_minutes(2).expect("the review comes");
    // The village gave out what the household did not need, and left it what it does.
    let kept = worked_ha(&sim, rich);
    assert!(kept >= need - 1e-9, "kept {kept} ha of a need of {need}");
    assert!(
        kept < total - 1e-9,
        "nothing was given out: {kept} of {total} ha"
    );
    let others: Vec<PermanentId> = all
        .iter()
        .map(|&(id, _)| id)
        .filter(|&id| id != rich && worked_ha(&sim, id) > 0.0)
        .collect();
    assert!(!others.is_empty(), "no household short of land got any");
    assert!(
        sim.land()
            .fields
            .iter()
            .all(|f| f.holder == Party::Settlement(settlement)),
        "giving out its use does not change who holds it"
    );
    // Changing hands moves no goods.
    let end = held(sim.people());
    let goods = sim.rules().catalog.goods.len();
    let gaps = population::unaccounted(goods, (&start.0, &start.1), (&end.0, &end.1));
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    sound_claims(&sim);
}

#[test]
fn household_fields_are_divided_among_heirs_when_a_household_is_no_more() {
    let mut sim = with_fields("", 6);
    assert!(
        sim.land()
            .fields
            .iter()
            .all(|f| f.holder == Party::Household(f.household)),
        "a household holds the ground it breaks"
    );
    let all = households(&sim);
    assert!(all.len() >= 3, "{all:?}");
    let (a, b, c) = (all[0].0, all[1].0, all[2].0);
    // Household `a` holds and works four fields; `b` and `c` are its last member's children.
    let fields: Vec<PermanentId> = sim.land().fields.iter().take(4).map(|f| f.id).collect();
    for f in &mut sim.land_mut_for_tests().fields {
        if fields.contains(&f.id) {
            f.household = a;
            f.holder = Party::Household(a);
        }
    }
    let members_of = |sim: &Sim, h: PermanentId| -> Vec<PermanentId> {
        sim.people()
            .household(h)
            .expect("household")
            .members
            .clone()
    };
    // The youngest die first, so the household is never left to children alone (who would be
    // taken in by kin, its land with them) and empties with its eldest.
    let mut gone = members_of(&sim, a);
    let now = sim.now();
    gone.sort_by(|x, y| {
        let age = |id: &PermanentId| sim.people().person(*id).expect("alive").age_years(now);
        age(x).total_cmp(&age(y))
    });
    let last = *gone.last().expect("a member");
    let child_b = members_of(&sim, b)[0];
    let child_c = members_of(&sim, c)[0];
    {
        let pop = sim.people_mut_for_tests();
        for child in [child_b, child_c] {
            let r = pop.records.get_mut(&child).expect("a record");
            r.mother = Some(last);
        }
    }
    let start = held(sim.people());
    for &m in &gone {
        sim.die_for_tests(m);
    }
    assert!(
        sim.people().household(a).is_none(),
        "the household is no more"
    );
    let land = &sim.land().fields;
    let mut to_b = 0;
    let mut to_c = 0;
    for f in land.iter().filter(|f| fields.contains(&f.id)) {
        match f.holder {
            Party::Household(h) if h == b => to_b += 1,
            Party::Household(h) if h == c => to_c += 1,
            other => panic!("field {} went to {other:?}", f.id),
        }
        assert_eq!(Party::Household(f.household), f.holder, "the heir works it");
    }
    assert_eq!((to_b, to_c), (2, 2), "divided evenly between the two heirs");
    // Its goods went to one heir through the ledger; nothing was lost.
    let end = held(sim.people());
    let goods = sim.rules().catalog.goods.len();
    let gaps = population::unaccounted(goods, (&start.0, &start.1), (&end.0, &end.1));
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
}

#[test]
fn ground_nobody_holds_is_taken_up_by_households_short_of_land() {
    let mut sim = with_fields("", 4);
    let all = households(&sim);
    let a = all[0].0;
    let fields: Vec<PermanentId> = sim.land().fields.iter().take(2).map(|f| f.id).collect();
    between_crops(&mut sim);
    for f in &mut sim.land_mut_for_tests().fields {
        if fields.contains(&f.id) {
            f.household = a;
            f.holder = Party::Household(a);
        }
    }
    // Its people die with no kin outside it: its ground is held by no one.
    {
        let pop = sim.people_mut_for_tests();
        let members = pop.household(a).expect("household").members.clone();
        for r in pop.records.values_mut() {
            if !members.contains(&r.id) {
                r.mother = r.mother.filter(|m| !members.contains(m));
                r.father = r.father.filter(|m| !members.contains(m));
            }
        }
        for m in &members {
            let r = pop.records.get_mut(m).expect("a record");
            r.mother = None;
            r.father = None;
        }
    }
    for m in sim
        .people()
        .household(a)
        .expect("household")
        .members
        .clone()
    {
        sim.die_for_tests(m);
    }
    for f in sim.land().fields.iter().filter(|f| fields.contains(&f.id)) {
        let Party::Household(taker) = f.holder else {
            panic!("field {} is held by {:?}", f.id, f.holder);
        };
        assert_ne!(taker, a);
        assert!(
            sim.people().household(taker).is_some(),
            "taken up by the living"
        );
        assert_eq!(f.household, taker, "and worked by its new holder");
    }
}

/// Sets the review to the coming midnight and lives until just past it, with `setup` done just
/// before, when nobody is in the fields.
fn review_after(sim: &mut Sim, setup: impl FnOnce(&mut Sim)) {
    let day = sim.now().day_index();
    sim.regime_mut_for_tests().review_day = ((day + 1).rem_euclid(365)) as u16;
    let to_midnight = 24 * 60 - sim.now().minute_of_day();
    sim.advance_minutes(to_midnight - 1).expect("advances");
    setup(sim);
    sim.advance_minutes(2).expect("the review comes");
}

#[test]
fn a_household_lets_ground_it_can_spare_for_a_share_of_its_grain() {
    let mut sim = with_fields("", 8);
    let rules = sim.rules().clone();
    let crop = &rules.catalog.crops[rules.people.farm.crop];
    let lease = rules
        .catalog
        .regime("core:regime/household")
        .and_then(|r| r.lease);
    let share = lease.expect("household tenure allows leases").holder_share;
    let all = households(&sim);
    let (landlord, members) = *all
        .iter()
        .min_by_key(|&&(id, n)| (n, id))
        .expect("households");
    let need = need_ha(&sim, members);
    // One household holds every field, more than it needs; the others none.
    review_after(&mut sim, |sim| {
        between_crops(sim);
        for f in &mut sim.land_mut_for_tests().fields {
            f.household = landlord;
            f.holder = Party::Household(landlord);
            f.lease = None;
        }
    });
    let now = sim.now();
    let let_out: Vec<PermanentId> = sim
        .land()
        .fields
        .iter()
        .filter(|f| f.household != landlord)
        .map(|f| f.id)
        .collect();
    assert!(!let_out.is_empty(), "nothing was let");
    for f in sim.land().fields.iter().filter(|f| let_out.contains(&f.id)) {
        assert_eq!(
            f.holder,
            Party::Household(landlord),
            "letting keeps the holder"
        );
        let l = f.lease.expect("a lease");
        assert!((f64::from(l.holder_share) - share).abs() < 1e-6);
        assert!(
            l.until > now.plus_minutes(300 * 24 * 60),
            "a crop year's term"
        );
    }
    assert!(
        worked_ha(&sim, landlord) >= need - 1e-9,
        "it keeps what it needs"
    );
    // A let field's grain is threshed: its holder's share goes to it as rent.
    let field = let_out[0];
    let tenant = sim
        .land()
        .fields
        .iter()
        .find(|f| f.id == field)
        .expect("field")
        .household;
    let start = held(sim.people());
    let grain_of =
        |sim: &Sim, h: PermanentId| sim.people().household(h).expect("household").stores[crop.good];
    let before = grain_of(&sim, landlord);
    for f in &mut sim.land_mut_for_tests().fields {
        if f.id == field {
            f.stage = FieldStage::Reaped;
            f.sheaves_kg = 150.0;
        }
    }
    let threshed = |sim: &Sim| {
        sim.land()
            .fields
            .iter()
            .find(|f| f.id == field)
            .is_some_and(|f| f.sheaves_kg < 150.0)
    };
    for _ in 0..30 {
        if threshed(&sim) {
            break;
        }
        sim.advance_minutes(24 * 60).expect("advances");
    }
    assert!(threshed(&sim), "the tenant never threshed");
    let rent = sim
        .people()
        .transfers
        .get(civ_agents::Channel::Rent, crop.good)
        + sim
            .people()
            .transfers
            .get(civ_agents::Channel::Rent, crop.seed_good);
    let left = sim
        .land()
        .fields
        .iter()
        .find(|f| f.id == field)
        .map_or(0.0, |f| f64::from(f.sheaves_kg));
    let paid_for = 150.0 - left;
    assert!(
        (rent - paid_for * share).abs() < 1e-6 * paid_for.max(1.0),
        "rent {rent} kg for {paid_for} kg threshed"
    );
    assert!(
        grain_of(&sim, landlord) > before - 1e-9,
        "the landlord got its share"
    );
    assert_ne!(tenant, landlord);
    let end = held(sim.people());
    let goods = sim.rules().catalog.goods.len();
    let gaps = population::unaccounted(goods, (&start.0, &start.1), (&end.0, &end.1));
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    // Leases survive a save and load.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "leases").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().fields, sim.land().fields);
    // At the end of its term a lease is renewed while the holder can spare the field and the
    // tenant still needs it.
    let renewable: Vec<PermanentId> = let_out[1..].to_vec();
    review_after(&mut sim, |sim| {
        between_crops(sim);
        let now = sim.now();
        for f in &mut sim.land_mut_for_tests().fields {
            if let Some(l) = f.lease.as_mut() {
                l.until = now;
            }
        }
    });
    // (A tenant that broke enough ground of its own meanwhile gives its lease up.)
    let now = sim.now();
    let mut renewed = 0;
    for f in sim
        .land()
        .fields
        .iter()
        .filter(|f| renewable.contains(&f.id))
    {
        if f.household != landlord {
            let l = f.lease.expect("still let");
            assert!(l.until > now, "renewed for another term");
            renewed += 1;
        } else {
            assert_eq!(
                f.lease, None,
                "a lease given up leaves the field to its holder"
            );
        }
    }
    assert!(renewed > 0, "no lease was renewed");
}

#[test]
fn wealth_is_measured_each_year_and_kept_in_saves() {
    // Under household tenure the households hold the ground they broke.
    let sim = with_fields("", 8);
    let rules = sim.rules();
    let now = sim.people().wealth_by_settlement(
        &rules.catalog,
        &rules.people,
        &rules.land,
        sim.land(),
        sim.now(),
    );
    assert_eq!(now.len(), 1, "one settlement");
    let (spread, households) = &now[0];
    let held: f64 = households.iter().map(|h| h.held_ha).sum();
    let fields: f64 = sim.land().fields.iter().map(|f| f.rect.area_ha()).sum();
    assert!((held - fields).abs() < 1e-9, "{held} of {fields} ha held");
    assert!(spread.holding_none < 1.0);
    assert_eq!(spread.common_ha, 0.0);
    assert_eq!(spread.people as usize, sim.people().living());
    assert!(
        spread.goods_h_per_head > 0.0,
        "the band's provisions are worth something"
    );
    sound_claims(&sim);

    // Under village tenure the settlement holds it all; each year's end is recorded and saved.
    let mut sim = create(3, "core:regime/village").expect("generates");
    let new_year = civ_core::time::MINUTES_PER_YEAR;
    sim.advance_minutes(new_year - sim.now().minutes() + 60)
        .expect("advances");
    let years = sim.people().wealth_years.clone();
    assert!(!years.is_empty(), "the year's end was recorded");
    let first = years[0];
    assert_eq!(first.year, 1, "the year that ended");
    assert!(first.spread.people > 0 && first.spread.households > 0);
    assert_eq!(first.spread.holding_none, 1.0, "no household holds a field");
    assert_eq!(first.spread.gini_held, 0.0);
    assert!(first.spread.common_ha > 0.0, "the village holds its fields");
    assert!(first.spread.worked_ha_per_head > 0.0);
    sound_claims(&sim);
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "wealth").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().wealth_years, years, "kept in its saves");
}
