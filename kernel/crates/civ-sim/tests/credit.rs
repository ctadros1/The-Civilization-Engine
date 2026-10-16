//! Loans in goods (M7d slice CA, ADR-0027 §1-2): a household short of food before its harvest
//! borrows from one with food to spare, at an increment, due after its harvest; a lender that
//! believes the loan too likely lost refuses; at the due day the debtor repays, or, unpaid past
//! its days of grace, the loan is defaulted and the lender's elder notes it; a world with loans
//! saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::credit::{Claim, Standing};
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
    borrower: PermanentId,
    lender: PermanentId,
    grain: usize,
}

/// Household `h`'s grain, kilograms.
fn grain_of(sim: &Sim, h: PermanentId, grain: usize) -> f64 {
    sim.people().household(h).expect("household").stores[grain]
}

/// Sets household `h`'s food: every food good emptied, then `kg` of grain.
fn feed(sim: &mut Sim, h: PermanentId, kg: f64) {
    let food: Vec<bool> = sim
        .rules()
        .catalog
        .goods
        .iter()
        .map(|g| g.kcal_per_kg > 0.0)
        .collect();
    let grain = sim.rules().catalog.crops[0].good;
    let pop = sim.people_mut_for_tests();
    let x = pop
        .households
        .iter_mut()
        .map(|(_, x)| x)
        .find(|x| x.id == h)
        .expect("household");
    for (s, f) in x.stores.iter_mut().zip(&food) {
        if *f {
            *s = 0.0;
        }
    }
    x.stores[grain] = kg;
}

/// A village of world 4, lived until a household holds fields: that household, its food gone, is
/// to borrow; the largest other household, holding 3000 kg of grain, can lend; every other
/// household holds no more than it keeps.
fn village() -> Scene {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Lenders".to_owned(),
            seed: 4,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [8; 16],
    )
    .expect("generates");
    let grain = sim.rules().catalog.crops[0].good;
    let mut borrower = None;
    for _ in 0..60 {
        sim.advance_minutes(DAY).expect("lives");
        let fields = &sim.land().fields;
        borrower = sim
            .people()
            .households
            .iter()
            .map(|(_, h)| h)
            .filter(|h| !h.members.is_empty())
            .map(|h| h.id)
            .find(|&h| fields.iter().any(|f| f.holder.household() == Some(h)));
        if borrower.is_some() {
            break;
        }
    }
    let borrower = borrower.expect("a household holds fields");
    let mut others: Vec<(usize, PermanentId)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| !h.members.is_empty() && h.id != borrower)
        .map(|h| (h.members.len(), h.id))
        .collect();
    others.sort_unstable_by(|a, b| b.cmp(a));
    let lender = others[0].1;
    for &(_, h) in &others[1..] {
        feed(&mut sim, h, 0.0);
    }
    feed(&mut sim, borrower, 0.0);
    feed(&mut sim, lender, 3000.0);
    Scene {
        sim,
        borrower,
        lender,
        grain,
    }
}

fn weigh(sim: &mut Sim) {
    sim.with_ctx_for_tests(|pop, ctx| pop.weigh_borrowing_for_tests(ctx));
}

fn loans(sim: &Sim) -> Vec<Claim> {
    sim.people().order.credit.claims.clone()
}

#[test]
fn a_household_short_before_its_harvest_borrows_from_one_with_food_to_spare() {
    let mut sc = village();
    let before = (
        grain_of(&sc.sim, sc.borrower, sc.grain),
        grain_of(&sc.sim, sc.lender, sc.grain),
    );
    weigh(&mut sc.sim);
    let l = loans(&sc.sim);
    assert_eq!(l.len(), 1, "{l:?}");
    let c = &l[0];
    assert_eq!((c.debtor, c.creditor), (sc.borrower, sc.lender));
    assert_eq!(usize::from(c.good), sc.grain);
    assert_eq!(c.standing, Standing::Open);
    let cp = sc.sim.rules().people.credit.clone();
    assert!(c.lent > 0.0);
    assert!(
        f64::from(c.owed) >= f64::from(c.lent) * (1.0 + cp.customary_increment) - 1e-3,
        "{c:?}"
    );
    assert!(c.due > c.made, "due after the harvest");
    // At most its share of what the lender could spare.
    assert!(f64::from(c.lent) <= 3000.0 * cp.lend_share + 1e-3);
    let after = (
        grain_of(&sc.sim, sc.borrower, sc.grain),
        grain_of(&sc.sim, sc.lender, sc.grain),
    );
    assert!((after.0 - before.0 - f64::from(c.lent)).abs() < 1e-3);
    // The lender's grain also settles what spoiled since it was last counted.
    let drop = before.1 - after.1;
    assert!(
        drop >= f64::from(c.lent) - 1e-3 && drop <= f64::from(c.lent) + 0.01 * before.1,
        "{drop} for {}",
        c.lent
    );
    // One owing on an open loan asks for no other.
    weigh(&mut sc.sim);
    assert_eq!(loans(&sc.sim).len(), 1);
    assert!(sc.sim.people().order.credit.asked >= 1);
    // The wealth panel says so.
    let settlement = sc
        .sim
        .people()
        .household(sc.borrower)
        .and_then(|x| x.settlement)
        .expect("a settlement");
    let words = civ_sim::frames::wealth::loan_words(&sc.sim, settlement);
    assert_eq!(words.len(), 1);
    assert!(
        words[0].contains(" owes ") && words[0].contains("kg of"),
        "{words:?}"
    );
}

#[test]
fn a_lender_that_believes_the_loan_too_likely_lost_refuses() {
    let mut sc = village();
    sc.sim
        .rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .credit
        .p_max = 0.0;
    weigh(&mut sc.sim);
    assert!(loans(&sc.sim).is_empty());
    let credit = &sc.sim.people().order.credit;
    assert!(
        credit.refused >= 1 && credit.refused <= credit.asked,
        "{credit:?}"
    );
    // Nor does one with food lend to a household short of nothing.
    let mut sc = village();
    feed(&mut sc.sim, sc.borrower, 5000.0);
    weigh(&mut sc.sim);
    assert!(loans(&sc.sim).is_empty());
}

#[test]
fn a_debtor_repays_at_the_due_day_or_defaults_when_grace_runs_out() {
    let mut sc = village();
    weigh(&mut sc.sim);
    let c = loans(&sc.sim)[0].clone();
    // With the grain to pay, it pays in full.
    feed(&mut sc.sim, sc.borrower, f64::from(c.owed) + 10.0);
    let lender_before = grain_of(&sc.sim, sc.lender, sc.grain);
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.repay_loans_for_tests(ctx, c.due));
    let paid = loans(&sc.sim)[0].clone();
    assert_eq!(paid.standing, Standing::Repaid);
    assert!((paid.paid - paid.owed).abs() < 1e-3);
    let lender_after = grain_of(&sc.sim, sc.lender, sc.grain);
    assert!((lender_after - lender_before - f64::from(c.owed)).abs() < 1e-2);
    // Without it, the loan stays open through its days of grace, then is defaulted.
    let mut sc = village();
    weigh(&mut sc.sim);
    let c = loans(&sc.sim)[0].clone();
    feed(&mut sc.sim, sc.borrower, 0.0);
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.repay_loans_for_tests(ctx, c.due));
    assert_eq!(loans(&sc.sim)[0].standing, Standing::Open);
    let grace = i64::from(sc.sim.rules().people.credit.grace_days);
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.repay_loans_for_tests(ctx, c.due + grace + 1));
    assert_eq!(loans(&sc.sim)[0].standing, Standing::Defaulted);
}

#[test]
fn a_world_with_loans_saves_and_loads_exactly() {
    let mut sc = village();
    weigh(&mut sc.sim);
    assert_eq!(loans(&sc.sim).len(), 1);
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sc.sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "loans",
    )
    .expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loans(&loaded), loans(&sc.sim));
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(&sc.sim, &loaded);
    sc.sim.advance_minutes(3 * DAY).expect("lives");
    loaded.advance_minutes(3 * DAY).expect("lives");
    same(&sc.sim, &loaded);
}
