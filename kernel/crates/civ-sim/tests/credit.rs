//! Loans in goods (M7d slices CA–CB, ADR-0027 §1-3): a household short of food before its
//! harvest borrows from one with food to spare, at an increment, due after its harvest; a lender
//! that believes the loan too likely lost refuses, or lends against a field's use or with a
//! guarantor; at the due day the debtor answers, repays, asks for time or refuses, or, unpaid past
//! its days of grace, the loan is defaulted and the pledge taken; a world with loans saves and
//! loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::credit::{Claim, Pledge, Standing};
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

/// A village of world 4, lived until a household holds a sown field: that household, its food gone, is
/// to borrow; the largest other household, holding 3000 kg of grain and every other household's
/// fields, can lend; every other household holds no food and no field.
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
    for _ in 0..120 {
        sim.advance_minutes(DAY).expect("lives");
        let fields = &sim.land().fields;
        borrower = sim
            .people()
            .households
            .iter()
            .map(|(_, h)| h)
            .filter(|h| !h.members.is_empty())
            .map(|h| h.id)
            .find(|&h| {
                fields.iter().any(|f| {
                    f.holder.household() == Some(h) && f.stage == civ_land::FieldStage::Sown
                })
            });
        if borrower.is_some() {
            break;
        }
    }
    let borrower = borrower.expect("a household holds a sown field");
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
    // The borrower alone holds fields: any other household's pass to the lender, who has food
    // to spare and no need to borrow, so no other short household has a harvest to borrow
    // against however many broke ground on the same day.
    for &(_, h) in &others[1..] {
        sim.with_ctx_for_tests(|pop, ctx| pop.move_fields_for_tests(ctx.land, h, lender));
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

/// The loan household `debtor` took first, and its place in the book.
fn loan_of(sim: &Sim, debtor: PermanentId) -> (usize, Claim) {
    let claims = &sim.people().order.credit.claims;
    let i = claims
        .iter()
        .position(|c| c.debtor == debtor && c.recourse.is_none())
        .unwrap_or_else(|| panic!("a loan to {debtor:?}: {claims:?}"));
    (i, claims[i].clone())
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

/// The scene's credit priors, to set for a test.
fn credit(sim: &mut Sim) -> &mut civ_agents::credit::CreditParams {
    &mut sim
        .rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .credit
}

/// Loans due by `day` are answered and paid now.
fn repay(sim: &mut Sim, day: i64) {
    sim.with_ctx_for_tests(|pop, ctx| pop.repay_loans_for_tests(ctx, day));
}

#[test]
fn a_debtor_repays_at_the_due_day_or_defaults_when_grace_runs_out() {
    let mut sc = village();
    // A debtor that means to pay (M7d slice CB: it answers first).
    credit(&mut sc.sim).pay_base = 50.0;
    weigh(&mut sc.sim);
    let c = loans(&sc.sim)[0].clone();
    // With the grain to pay and its days of food kept besides, it pays in full.
    feed(&mut sc.sim, sc.borrower, f64::from(c.owed) + 200.0);
    let lender_before = grain_of(&sc.sim, sc.lender, sc.grain);
    repay(&mut sc.sim, c.due);
    let paid = loans(&sc.sim)[0].clone();
    assert_eq!(paid.standing, Standing::Repaid, "{paid:?}");
    assert_eq!(paid.answer.map(|a| a.0), Some(true));
    assert!((paid.paid - paid.owed).abs() < 1e-3);
    let lender_after = grain_of(&sc.sim, sc.lender, sc.grain);
    assert!((lender_after - lender_before - f64::from(c.owed)).abs() < 1e-2);
    // Without it, and with no time granted, the loan stays open through its days of grace, then
    // is defaulted.
    let mut sc = village();
    let cp = credit(&mut sc.sim);
    cp.pay_base = 50.0;
    cp.max_extensions = 0;
    weigh(&mut sc.sim);
    let c = loans(&sc.sim)[0].clone();
    feed(&mut sc.sim, sc.borrower, 0.0);
    repay(&mut sc.sim, c.due);
    assert_eq!(loans(&sc.sim)[0].standing, Standing::Open);
    let grace = i64::from(sc.sim.rules().people.credit.grace_days);
    repay(&mut sc.sim, c.due + grace + 1);
    assert_eq!(loans(&sc.sim)[0].standing, Standing::Defaulted);
}

#[test]
fn a_debtor_short_at_the_due_day_asks_for_time_and_one_unwilling_refuses() {
    // The creditor's elder regards the debtor's enough: the due day moves on.
    let mut sc = village();
    let cp = credit(&mut sc.sim);
    cp.pay_base = 50.0;
    cp.extend_regard = -1.0;
    weigh(&mut sc.sim);
    let c = loans(&sc.sim)[0].clone();
    feed(&mut sc.sim, sc.borrower, 0.0);
    repay(&mut sc.sim, c.due);
    let later = loans(&sc.sim)[0].clone();
    let days = i64::from(sc.sim.rules().people.credit.extension_days);
    assert_eq!((later.asks, later.extensions), (1, 1), "{later:?}");
    assert_eq!(later.due, c.due + days);
    assert_eq!(later.standing, Standing::Open);
    // It does not regard it enough: asked, refused, and the days of grace run from the first day.
    let mut sc = village();
    let cp = credit(&mut sc.sim);
    cp.pay_base = 50.0;
    cp.extend_regard = 2.0;
    weigh(&mut sc.sim);
    let c = loans(&sc.sim)[0].clone();
    feed(&mut sc.sim, sc.borrower, 0.0);
    repay(&mut sc.sim, c.due);
    let later = loans(&sc.sim)[0].clone();
    assert_eq!((later.asks, later.extensions, later.due), (1, 0, c.due));
    // One unwilling refuses at the due day, whatever it holds, and the lender's household notes
    // that the borrower's word was not kept.
    let mut sc = village();
    credit(&mut sc.sim).pay_base = -50.0;
    weigh(&mut sc.sim);
    let c = loans(&sc.sim)[0].clone();
    feed(&mut sc.sim, sc.borrower, f64::from(c.owed) + 200.0);
    repay(&mut sc.sim, c.due);
    let refused = loans(&sc.sim)[0].clone();
    assert_eq!(refused.standing, Standing::Refused);
    assert_eq!(refused.answer.map(|a| a.0), Some(false));
    assert_eq!(refused.paid, 0.0);
    let words = civ_sim::frames::wealth::loan_words(
        &sc.sim,
        sc.sim
            .people()
            .household(sc.borrower)
            .and_then(|x| x.settlement)
            .expect("a settlement"),
    );
    assert!(words[0].contains("refused to repay"), "{words:?}");
}

#[test]
fn a_lender_that_would_refuse_lends_with_a_guarantor_who_pays_when_the_debtor_does_not() {
    let mut sc = village();
    let cp = credit(&mut sc.sim);
    // Bare, and against a field's use, the loss believed is too much; one who will stand
    // guarantor brings it within reach.
    cp.p_max = 0.05;
    cp.field_recovery = 0.0;
    cp.guarantor_regard = -1.0;
    cp.pay_base = 50.0;
    cp.max_extensions = 0;
    // The guarantor: another household, with grain to spare.
    let guarantor = sc
        .sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| !h.members.is_empty() && h.id != sc.borrower && h.id != sc.lender)
        .map(|h| h.id)
        .min()
        .expect("a third household");
    feed(&mut sc.sim, guarantor, 3000.0);
    // The lender's household trusts the guarantor's, as neighbours long at one hearth do (the
    // guarantor's worth to it is its regard and what it can spare, ADR-0027 §3), whatever ties
    // the village's first weeks happened to make.
    let (tp, today) = (sc.sim.rules().people.ties.clone(), sc.sim.now().day_index());
    let members = |sim: &Sim, h: PermanentId| {
        sim.people()
            .household(h)
            .map(|x| x.members.clone())
            .unwrap_or_default()
    };
    let (from, to) = (members(&sc.sim, sc.lender), members(&sc.sim, guarantor));
    let pop = sc.sim.people_mut_for_tests();
    for &x in &from {
        for &y in &to {
            pop.ties
                .record(x, y, civ_agents::ties::Act::Hearth, 400.0, 0.0, today, &tp);
        }
    }
    weigh(&mut sc.sim);
    let (i, c) = loan_of(&sc.sim, sc.borrower);
    let Some(Pledge::Guarantor(g)) = c.pledge else {
        panic!("lent with a guarantor: {c:?}");
    };
    assert!(sc.sim.people().order.credit.guarantees(g));
    // Unpaid when grace runs out: the guarantor pays what it can spare of the good, and the
    // debtor owes it that.
    feed(&mut sc.sim, sc.borrower, 0.0);
    let grace = i64::from(sc.sim.rules().people.credit.grace_days);
    let g_before = grain_of(&sc.sim, g, sc.grain);
    repay(&mut sc.sim, c.due + grace + 1);
    let l = loans(&sc.sim);
    let failed = &l[i];
    assert_eq!(failed.standing, Standing::Defaulted);
    assert!(failed.by_guarantor > 0.0, "{failed:?}");
    assert!((failed.paid - failed.by_guarantor).abs() < 1e-3);
    let g_after = grain_of(&sc.sim, g, sc.grain);
    assert!(g_before - g_after >= f64::from(failed.by_guarantor) - 1e-3);
    let recourse = l
        .iter()
        .find(|r| r.recourse == Some(c.id))
        .expect("the guarantor's claim");
    assert_eq!(
        (recourse.creditor, recourse.debtor, recourse.recourse),
        (g, sc.borrower, Some(c.id))
    );
    assert!((recourse.owed - failed.by_guarantor).abs() < 1e-3);
    assert_eq!(recourse.standing, Standing::Open);
}

#[test]
fn a_field_pledged_on_a_failed_loan_is_worked_by_the_creditor_until_the_debt_is_paid() {
    let mut sc = village();
    // The pledge is the matter, not the courts: no law of debts heard is ever proposed here, so a
    // case's time to pay does not reopen the loan before its field lies between crops (cases are
    // tested below, `under_debts_heard_*`).
    for d in &mut sc
        .sim
        .rules_mut_for_tests()
        .expect("rules of its own")
        .catalog
        .policies
    {
        d.answers
            .retain(|&i| i != civ_agents::polity::IssueKind::DebtsUnpaid);
    }
    let cp = credit(&mut sc.sim);
    cp.p_max = 0.05;
    cp.field_recovery = 0.99;
    cp.pay_base = 50.0;
    cp.max_extensions = 0;
    weigh(&mut sc.sim);
    let (i, c) = loan_of(&sc.sim, sc.borrower);
    let Some(Pledge::Field(field)) = c.pledge else {
        panic!("lent against a field: {c:?}");
    };
    assert!(sc.sim.people().order.credit.pledged(field));
    let worked_by = |sim: &Sim| {
        let f = sim
            .land()
            .fields
            .iter()
            .find(|f| f.id == field)
            .expect("the field");
        (f.household, f.lease.is_some())
    };
    assert_eq!(worked_by(&sc.sim), (sc.borrower, false));
    // Unpaid when grace runs out: from the day it lies between crops the creditor works it.
    feed(&mut sc.sim, sc.borrower, 0.0);
    let grace = i64::from(sc.sim.rules().people.credit.grace_days);
    repay(&mut sc.sim, c.due + grace + 1);
    assert_eq!(loans(&sc.sim)[i].standing, Standing::Defaulted);
    let mut days = 0;
    while worked_by(&sc.sim).0 != sc.lender && days < 400 {
        repay(&mut sc.sim, c.due + grace + 1);
        sc.sim.advance_minutes(DAY).expect("lives");
        days += 1;
    }
    assert_eq!(worked_by(&sc.sim), (sc.lender, true), "after {days} days");
    assert!(loans(&sc.sim)[i].taken.is_some());
    // Word of the field taken goes about (backlog D's "Not yet": a pledged field taken told).
    let word: Vec<_> = sc
        .sim
        .people()
        .word
        .claims
        .iter()
        .filter(|c| c.kind == civ_agents::word::ClaimKind::PledgeTaken)
        .collect();
    assert_eq!(word.len(), 1, "told once");
    assert_eq!(word[0].event, Some(c.id));
    // Once its crop has paid the debt, it goes back.
    {
        let pop = sc.sim.people_mut_for_tests();
        let claim = &mut pop.order.credit.claims[i];
        claim.paid = claim.owed;
    }
    let mut days = 0;
    while worked_by(&sc.sim).0 != sc.borrower && days < 400 {
        sc.sim.advance_minutes(DAY).expect("lives");
        days += 1;
    }
    assert_eq!(
        worked_by(&sc.sim),
        (sc.borrower, false),
        "after {days} days"
    );
}

#[test]
fn a_world_with_loans_saves_and_loads_exactly() {
    let mut sc = village();
    weigh(&mut sc.sim);
    assert_eq!(loans(&sc.sim).len(), 1);
    // With a pledge, an answer and remedies (M7d slice CB).
    {
        let pop = sc.sim.people_mut_for_tests();
        let c = &mut pop.order.credit.claims[0];
        c.pledge = Some(Pledge::Guarantor(c.creditor));
        c.answer = Some((true, 1.25));
        c.asks = 1;
        c.extensions = 1;
        c.taken = Some(c.made + 3);
        c.by_guarantor = 2.5;
        c.recourse = Some(7);
    }
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

#[test]
fn a_workshop_that_cannot_pay_a_wage_owes_it_and_fails_only_when_it_is_still_unpaid() {
    // M7d slice CB (ADR-0027 §3): what a workshop could not pay, it owes the worker's household;
    // it hires nobody meanwhile; its owners pay what they can spare when it falls due, and if it is
    // unpaid then, the workshop fails.
    let open = |sim: &Sim, id: PermanentId| {
        sim.people()
            .firms
            .iter()
            .find(|f| f.id == id)
            .is_some_and(|f| f.is_open())
    };
    let wage_days = |sim: &Sim| i64::from(sim.rules().people.credit.wage_days);
    let set_up = |owner_grain: f64| {
        let mut sc = village();
        let sickle = sc
            .sim
            .rules()
            .catalog
            .goods
            .iter()
            .position(|g| g.id == "core:good/sickle")
            .expect("sickle");
        feed(&mut sc.sim, sc.lender, owner_grain);
        let founder = sc.sim.people().household(sc.lender).expect("owner").members[0];
        let (owner, worker, grain) = (sc.lender, sc.borrower, sc.grain);
        let firm = sc
            .sim
            .with_ctx_for_tests(|pop, ctx| pop.found_firm_for_tests(ctx, owner, founder, sickle))
            .expect("a workshop");
        sc.sim.with_ctx_for_tests(|pop, ctx| {
            pop.owe_wages_for_tests(ctx, firm, worker, grain, 6.0);
            pop.owe_wages_for_tests(ctx, firm, worker, grain, 4.0);
        });
        (sc, firm)
    };
    // Its owners have grain to spare: they pay what the workshop owes when it falls due.
    let (mut sc, firm) = set_up(3000.0);
    let owed: Vec<Claim> = loans(&sc.sim)
        .into_iter()
        .filter(|c| c.debtor == firm)
        .collect();
    assert_eq!(
        owed.len(),
        1,
        "wages owed one household are one claim: {owed:?}"
    );
    assert!((owed[0].owed - 10.0).abs() < 1e-4);
    assert_eq!(owed[0].creditor, sc.borrower);
    assert!(sc.sim.people().order.credit.owes(firm));
    let due = owed[0].made + wage_days(&sc.sim);
    assert_eq!(owed[0].due, due);
    let worker_before = grain_of(&sc.sim, sc.borrower, sc.grain);
    repay(&mut sc.sim, due);
    let paid = loans(&sc.sim)
        .into_iter()
        .find(|c| c.debtor == firm)
        .expect("the wages");
    assert_eq!(paid.standing, Standing::Repaid, "{paid:?}");
    assert!((grain_of(&sc.sim, sc.borrower, sc.grain) - worker_before - 10.0).abs() < 1e-3);
    assert!(open(&sc.sim, firm), "a workshop that paid stays open");
    // Its owners have nothing: unpaid when due, the workshop fails.
    let (mut sc, firm) = set_up(0.0);
    let due = sc.sim.now().day_index() + wage_days(&sc.sim);
    repay(&mut sc.sim, due - 1);
    assert!(open(&sc.sim, firm), "not yet due");
    repay(&mut sc.sim, due);
    let unpaid = loans(&sc.sim)
        .into_iter()
        .find(|c| c.debtor == firm)
        .expect("the wages");
    assert_eq!(unpaid.standing, Standing::Defaulted);
    assert!(!open(&sc.sim, firm), "a workshop unpaid when due fails");
    let settlement = sc
        .sim
        .people()
        .household(sc.borrower)
        .and_then(|x| x.settlement)
        .expect("a settlement");
    let words = civ_sim::frames::wealth::loan_words(&sc.sim, settlement);
    assert!(
        words
            .iter()
            .any(|w| w.contains("wages unpaid since") && w.contains("failed")),
        "{words:?}"
    );
}

/// The law of debts heard of template `template` in force at household `h`'s polity since today,
/// known to everyone there.
fn debts_heard(sim: &mut Sim, h: PermanentId, template: &str) -> PermanentId {
    law_in_force(sim, h, template, civ_agents::polity::IssueKind::DebtsUnpaid)
}

/// The law of template `template`, answering `issue`, in force at household `h`'s polity since
/// today, known to everyone there.
fn law_in_force(
    sim: &mut Sim,
    h: PermanentId,
    template: &str,
    issue: civ_agents::polity::IssueKind,
) -> PermanentId {
    use civ_agents::polity::{Law, LawStatus, Outcome as Decided};
    let policies = &sim.rules().catalog.policies;
    let policy = policies
        .iter()
        .position(|d| d.id == template)
        .expect("the core content has the template");
    let kind = policies[policy].kind;
    let id = sim.allocate_id_for_tests();
    let now = sim.now();
    let pop = sim.people_mut_for_tests();
    let at = pop
        .household(h)
        .and_then(|x| x.settlement)
        .expect("settled");
    let members: Vec<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(at))
        .flat_map(|(_, x)| x.members.clone())
        .collect();
    let pi = pop
        .polities
        .iter()
        .position(|p| p.settlement == at)
        .expect("a polity");
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
        sponsor: members[0],
        proposed: now,
        issue,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Decided::Passed),
        eligible: members.len() as u32,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
        target: None,
    };
    for &m in &members {
        law.learn(m, now.day_index());
    }
    pop.polities[pi].laws.push(law);
    id
}

/// A village whose borrower took a loan and left it unpaid when its days of grace ran out.
fn defaulted() -> (Scene, u32) {
    let mut sc = village();
    let cp = credit(&mut sc.sim);
    cp.pay_base = 50.0;
    cp.max_extensions = 0;
    weigh(&mut sc.sim);
    let (i, c) = loan_of(&sc.sim, sc.borrower);
    feed(&mut sc.sim, sc.borrower, 0.0);
    let grace = i64::from(sc.sim.rules().people.credit.grace_days);
    repay(&mut sc.sim, c.due + grace + 1);
    assert_eq!(loans(&sc.sim)[i].standing, Standing::Defaulted);
    (sc, c.id)
}

#[test]
fn under_debts_heard_a_creditor_brings_a_loan_left_unpaid_and_it_is_heard() {
    // M7d slice CB (ADR-0027 §3): without the law nobody brings it; with it in force, the
    // lender's household brings it before the gathering, which hears it.
    let (mut sc, loan) = defaulted();
    let pi = |sim: &Sim, h: PermanentId| {
        let at = sim.people().household(h).and_then(|x| x.settlement);
        sim.people()
            .polities
            .iter()
            .position(|p| p.settlement == at.expect("settled"))
            .expect("a polity")
    };
    let p = pi(&sc.sim, sc.lender);
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.weigh_debts_for_tests(ctx, p));
    let case = |sim: &Sim| {
        sim.people()
            .order
            .cases
            .iter()
            .find(|c| c.loan == Some(loan))
            .cloned()
    };
    assert!(case(&sc.sim).is_none(), "no law, no case");
    let law = debts_heard(&mut sc.sim, sc.lender, "core:policy/debts_heard");
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.weigh_debts_for_tests(ctx, p));
    let c = case(&sc.sim).expect("the lender's household brought it");
    assert_eq!(
        (c.law, c.accuser, c.accused_household, c.incident),
        (law, sc.lender, sc.borrower, 0)
    );
    // Brought once.
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.weigh_debts_for_tests(ctx, p));
    let brought = sc
        .sim
        .people()
        .order
        .cases
        .iter()
        .filter(|c| c.loan == Some(loan))
        .count();
    assert_eq!(brought, 1);
    assert!(
        sc.sim
            .people()
            .chronicle
            .iter()
            .any(|e| e.name.contains("for a loan it did not repay")),
        "the chronicle says it was brought"
    );
    // A world with it saves and loads exactly.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sc.sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "debts",
    )
    .expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(case(&loaded), case(&sc.sim));
    // The gathering hears it.
    for _ in 0..60 {
        if case(&sc.sim).is_some_and(|c| c.stage != civ_agents::crime::CaseStage::Open) {
            break;
        }
        sc.sim.advance_minutes(DAY).expect("advances");
    }
    let c = case(&sc.sim).expect("kept");
    assert_ne!(
        c.stage,
        civ_agents::crime::CaseStage::Open,
        "heard within two months"
    );
    if c.stage == civ_agents::crime::CaseStage::Found {
        assert!(
            sc.sim.people().order.obligations.iter().any(|o| {
                o.case == Some(c.id)
                    && o.kind == civ_agents::crime::Owed::Debt
                    && o.incident == loan
                    && o.debtor == sc.borrower
                    && o.beneficiary == sc.lender
            }),
            "a finding imposes what is owed"
        );
    }
}

#[test]
fn a_finding_on_a_debt_gives_time_or_imposes_what_is_owed() {
    use civ_agents::polity::{DebtRemedy, DebtsDef};
    // Time: the loan is open again, unanswered, due later.
    let (mut sc, loan) = defaulted();
    let today = sc.sim.now().day_index();
    let time = DebtsDef {
        remedy: DebtRemedy::Time,
        share: 0.5,
        extend_days: 180,
    };
    let words = sc
        .sim
        .with_ctx_for_tests(|pop, ctx| pop.debt_found_for_tests(ctx, loan, 1, time));
    assert!(words.contains("given until"), "{words}");
    let c = loans(&sc.sim)
        .into_iter()
        .find(|c| c.id == loan)
        .expect("kept");
    assert_eq!(
        (c.standing, c.answer, c.due),
        (Standing::Open, None, today + 180)
    );
    // Imposed: what is still owed is owed as a finding's obligation, and what is paid on it is
    // paid on the loan.
    let (mut sc, loan) = defaulted();
    let imposed = DebtsDef {
        remedy: DebtRemedy::Imposed,
        share: 1.0,
        extend_days: 0,
    };
    let left = loans(&sc.sim)
        .into_iter()
        .find(|c| c.id == loan)
        .expect("kept")
        .left();
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.debt_found_for_tests(ctx, loan, 1, imposed));
    let ob = sc
        .sim
        .people()
        .order
        .obligations
        .iter()
        .find(|o| o.kind == civ_agents::crime::Owed::Debt)
        .copied()
        .expect("an obligation");
    assert_eq!(
        (ob.incident, ob.debtor, ob.beneficiary),
        (loan, sc.borrower, sc.lender)
    );
    let per_kg = sc.sim.rules().catalog.goods[sc.grain].kcal_per_kg;
    assert!(
        (f64::from(ob.kcal) - left * per_kg).abs() < 1.0,
        "{ob:?} {left}"
    );
    // The debtor, given grain and meaning to pay, pays on it day by day.
    feed(&mut sc.sim, sc.borrower, 3000.0);
    for _ in 0..10 {
        sc.sim.advance_minutes(DAY).expect("advances");
    }
    let after = loans(&sc.sim)
        .into_iter()
        .find(|c| c.id == loan)
        .expect("kept")
        .left();
    let ob_after = sc
        .sim
        .people()
        .order
        .obligations
        .iter()
        .find(|o| o.id == ob.id)
        .copied()
        .expect("kept");
    if ob_after.paid_kcal > 0.0 {
        assert!(after < left, "paid on the loan: {after} of {left}");
    }
}

#[test]
fn under_lending_from_the_store_a_short_household_borrows_from_it_and_repays_into_it() {
    // M7d slice CC (ADR-0027 §4): the store first, at the law's terms; repaid into the store.
    let mut sc = village();
    credit(&mut sc.sim).pay_base = 50.0;
    let at = sc
        .sim
        .people()
        .household(sc.borrower)
        .and_then(|x| x.settlement)
        .expect("settled");
    let pi = sc
        .sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == at)
        .expect("a polity");
    let store = sc.sim.people().polities[pi].id;
    let goods = sc.sim.rules().catalog.goods.len();
    {
        let store = &mut sc.sim.people_mut_for_tests().polities[pi].stores;
        store.resize(goods, 0.0);
        store[sc.grain] = 2000.0;
    }
    law_in_force(
        &mut sc.sim,
        sc.borrower,
        "core:policy/lend_from_store",
        civ_agents::polity::IssueKind::FoodShort,
    );
    let def = sc
        .sim
        .rules()
        .catalog
        .policies
        .iter()
        .find(|d| d.id == "core:policy/lend_from_store")
        .and_then(|d| d.lending)
        .expect("terms");
    weigh(&mut sc.sim);
    let (_, c) = loan_of(&sc.sim, sc.borrower);
    assert_eq!(c.creditor, store, "the store lent: {c:?}");
    assert_eq!(usize::from(c.good), sc.grain);
    assert!(f64::from(c.lent) <= 2000.0 * f64::from(def.share) + 1e-3);
    assert!(
        (f64::from(c.owed) - f64::from(c.lent) * (1.0 + f64::from(def.increment))).abs() < 1e-2,
        "{c:?}"
    );
    let held = |sim: &Sim| sim.people().polities[pi].stores[sc.grain];
    let after_lending = held(&sc.sim);
    assert!(
        after_lending < 2000.0 - f64::from(c.lent) + 1.0,
        "{after_lending}"
    );
    // Repaid into the store.
    feed(&mut sc.sim, sc.borrower, f64::from(c.owed) + 200.0);
    repay(&mut sc.sim, c.due);
    let (_, paid) = loan_of(&sc.sim, sc.borrower);
    assert_eq!(paid.standing, Standing::Repaid, "{paid:?}");
    assert!(held(&sc.sim) > after_lending + f64::from(c.owed) - 1.0);
    let words = civ_sim::frames::wealth::loan_words(&sc.sim, at);
    assert!(
        words.iter().any(|w| w.contains("the common store")),
        "{words:?}"
    );
}

/// The village's lender made a keeper: no grain of its own to spare, 3000 kg left with it by
/// another household whose notes for it are out. The issue's number.
fn keeper_of_notes(sc: &mut Scene) -> u32 {
    let note = sc
        .sim
        .rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/note")
        .expect("notes");
    // The lender holds no grain to spare but keeps 3000 kg left with it by another household,
    // whose notes for it are out.
    feed(&mut sc.sim, sc.lender, 0.0);
    let depositor = sc
        .sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .find(|h| !h.members.is_empty() && h.id != sc.lender && h.id != sc.borrower)
        .map(|h| h.id)
        .expect("another household");
    let (now, goods) = (sc.sim.now(), sc.sim.rules().catalog.goods.len());
    let lot = PermanentId::from_raw(u64::MAX - 7).expect("an id");
    let pop = sc.sim.people_mut_for_tests();
    let mut kept = civ_agents::custody::Kept::new(lot, (depositor, sc.lender), now, goods);
    kept.stores[sc.grain] = 3000.0;
    kept.noted = true;
    pop.kept.push(kept);
    let issue = pop
        .notes
        .write(sc.lender, (sc.grain, 1.0), 3000.0, depositor);
    for (_, x) in pop.households.iter_mut() {
        if x.id == depositor {
            x.stores[note] += 3000.0;
        }
    }
    issue
}

#[test]
fn a_keeper_whose_notes_are_out_lends_in_them_and_keeps_its_grain() {
    let mut sc = village();
    let issue = keeper_of_notes(&mut sc);
    weigh(&mut sc.sim);
    let (_, c) = loan_of(&sc.sim, sc.borrower);
    assert_eq!(c.creditor, sc.lender);
    // Owed in the grain its notes pay; lent in its notes, written for the borrower.
    assert_eq!(usize::from(c.good), sc.grain);
    let lent = f64::from(c.lent);
    assert!(lent > 1.0);
    let notes = &sc.sim.people().notes;
    assert!((notes.held(sc.borrower, issue) - lent).abs() < 1e-3);
    let i = notes.issue(issue).expect("issue");
    assert!((i.lent - lent).abs() < 1e-3 && (i.count - 3000.0 - lent).abs() < 1e-3);
    assert!(
        grain_of(&sc.sim, sc.lender, sc.grain).abs() < 1e-9,
        "no grain lent"
    );
    // Within what its coverage lets it write.
    let coverage = sc.sim.rules().people.market.note_coverage;
    assert!(lent <= 3000.0 / coverage - 3000.0 + 1e-3);
}

#[test]
fn a_debtor_of_a_loan_in_notes_defaulting_is_heard_of_on_the_issue() {
    let mut sc = village();
    let issue = keeper_of_notes(&mut sc);
    let cp = credit(&mut sc.sim);
    cp.pay_base = 50.0;
    cp.max_extensions = 0;
    weigh(&mut sc.sim);
    let (_, c) = loan_of(&sc.sim, sc.borrower);
    assert_eq!(c.creditor, sc.lender);
    let issue_of = |sim: &Sim| sim.people().notes.issue(issue).expect("issue").clone();
    assert_eq!(issue_of(&sc.sim).defaulted_day, -1);
    feed(&mut sc.sim, sc.borrower, 0.0);
    let grace = i64::from(sc.sim.rules().people.credit.grace_days);
    repay(&mut sc.sim, c.due);
    repay(&mut sc.sim, c.due + grace + 1);
    assert_eq!(
        loan_of(&sc.sim, sc.borrower).1.standing,
        Standing::Defaulted
    );
    assert_eq!(issue_of(&sc.sim).defaulted_day, c.due + grace + 1);
}

/// The village's keeper of notes (`keeper_of_notes`) with only 1000 kg left behind its 3000
/// pieces, and their holder presenting every one at its door, as on hearing of a default: what
/// it refused, and the holder.
fn refuse_notes(sc: &mut Scene, issue: u32) -> (f64, PermanentId) {
    let holder = sc.sim.people().notes.purses[0].holder;
    let today = sc.sim.now().day_index();
    let pop = sc.sim.people_mut_for_tests();
    pop.kept[0].stores[sc.grain] = 1000.0;
    pop.notes.issue_mut(issue).expect("issue").defaulted_day = today;
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.notes_review_for_tests(ctx, holder));
    let c = sc.sim.people().notes.issue(issue).expect("issue").clone();
    (c.refused, holder)
}

#[test]
fn under_bearer_heard_notes_refused_become_a_debt_the_gathering_hears() {
    // Without the law, the refused pieces stay with their holder, owed and unheard.
    let mut sc = village();
    let issue = keeper_of_notes(&mut sc);
    let (refused, holder) = refuse_notes(&mut sc, issue);
    assert!((refused - 2000.0).abs() < 1e-6, "{refused}");
    assert!((sc.sim.people().notes.held(holder, issue) - 2000.0).abs() < 1e-6);
    assert!(sc.sim.people().notes.dishonoured.is_empty());
    // With it in force where the keeper lives, they are retired and booked as what the keeper
    // owes their holder, in grain, refused today.
    let mut sc = village();
    let issue = keeper_of_notes(&mut sc);
    let law = law_in_force(
        &mut sc.sim,
        sc.lender,
        "core:policy/zealously_hear_the_bearer",
        civ_agents::polity::IssueKind::NotesRefused,
    );
    let (refused, holder) = refuse_notes(&mut sc, issue);
    assert!((refused - 2000.0).abs() < 1e-6, "{refused}");
    assert!(sc.sim.people().notes.held(holder, issue) < 1e-9, "retired");
    assert!(sc.sim.people().notes.issue(issue).expect("issue").count < 1e-6);
    let claim = sc
        .sim
        .people()
        .order
        .credit
        .claims
        .iter()
        .find(|c| c.creditor == holder && c.debtor == sc.lender)
        .cloned()
        .expect("a debt booked");
    assert_eq!(usize::from(claim.good), sc.grain);
    assert!((f64::from(claim.owed) - 2000.0).abs() < 1e-3);
    assert_eq!(claim.standing, Standing::Defaulted);
    assert!(sc.sim.people().notes.is_dishonoured(claim.id));
    // The holder's household brings it before the gathering under that law, and the chronicle
    // says so; debts heard alone would not.
    let at = sc
        .sim
        .people()
        .household(sc.lender)
        .and_then(|x| x.settlement)
        .expect("settled");
    let p = sc
        .sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == at)
        .expect("a polity");
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.weigh_debts_for_tests(ctx, p));
    let case = sc
        .sim
        .people()
        .order
        .cases
        .iter()
        .find(|c| c.loan == Some(claim.id))
        .cloned()
        .expect("the holder's household brought it");
    assert_eq!(
        (case.law, case.accuser, case.accused_household),
        (law, holder, sc.lender)
    );
    assert!(
        sc.sim
            .people()
            .chronicle
            .iter()
            .any(|e| e.name.contains("for notes it refused at its door")),
        "the chronicle says it was brought"
    );
    // A finding imposes what is still owed.
    let def = sc
        .sim
        .rules()
        .catalog
        .policies
        .iter()
        .find(|d| d.id == "core:policy/zealously_hear_the_bearer")
        .and_then(|d| d.debts)
        .expect("its remedy");
    let words = sc
        .sim
        .with_ctx_for_tests(|pop, ctx| pop.debt_found_for_tests(ctx, claim.id, case.id, def));
    assert_eq!(words, ", which the finding imposes");
    assert!(
        sc.sim
            .people()
            .order
            .obligations
            .iter()
            .any(|o| o.debtor == sc.lender && o.beneficiary == holder && o.kcal > 0.0),
        "an obligation imposed"
    );
    // A world with it saves and loads exactly.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sc.sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "bearer",
    )
    .expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().notes, sc.sim.people().notes);
}

#[test]
fn word_of_a_default_goes_about_and_those_who_hear_it_think_the_worse_of_the_debtor() {
    // Backlog D (research 08-07 §5.3): a loan failed is word told by contact, not known
    // everywhere at once. The creditor's people hold it at first hand; others hear it at the
    // hearth and think the worse of the debtor's elder in their word; the debtor's own household
    // neither thinks the worse of its elder nor tells it.
    use civ_agents::ties::{Act, Domain};
    use civ_agents::word::ClaimKind;
    let (mut sc, loan) = defaulted();
    let day = sc.sim.now().day_index();
    let members = |sim: &Sim, h: PermanentId| -> Vec<PermanentId> {
        let mut m = sim
            .people()
            .household(h)
            .expect("household")
            .members
            .clone();
        m.sort_unstable();
        m
    };
    let word = &sc.sim.people().word;
    let said: Vec<_> = word
        .claims
        .iter()
        .filter(|c| c.kind == ClaimKind::Defaulted)
        .copied()
        .collect();
    assert_eq!(said.len(), 1, "{said:?}");
    let claim = said[0];
    assert_eq!(claim.event, Some(loan));
    let elder = claim.subject.expect("names the debtor's elder");
    assert!(members(&sc.sim, sc.borrower).contains(&elder));
    assert_eq!(
        Some(claim.settlement),
        sc.sim
            .people()
            .household(sc.borrower)
            .and_then(|x| x.settlement)
    );
    // The creditor's people hold it at first hand; nobody else has heard it.
    let lenders = members(&sc.sim, sc.lender);
    for &m in &lenders {
        assert!(word.has_heard(m, claim.id));
    }
    let teller = lenders[0];
    let others: Vec<PermanentId> = sc
        .sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| h.id != sc.borrower && h.id != sc.lender && !h.members.is_empty())
        .flat_map(|h| h.members.iter().copied())
        .collect();
    assert!(others.len() >= 2, "two people of other households");
    let (x, y) = (others[0], others[1]);
    for &p in others.iter().chain(&members(&sc.sim, sc.borrower)) {
        assert!(!word.has_heard(p, claim.id));
    }
    let words = civ_sim::frames::word::claim_words(&sc.sim, &claim);
    assert!(
        words.contains("did not repay what")
            && words.contains("kg of")
            && words.contains(" lent it"),
        "{words}"
    );
    let tp = sc.sim.rules().people.ties.clone();
    let regard = |sim: &Sim, p: PermanentId| sim.people().ties.regard(p, elder, day, &tp);
    let bad_word = |sim: &Sim, p: PermanentId| {
        sim.people()
            .ties
            .tie(p, elder, day, &tp)
            .map_or(tp.prior as f32, |t| t.bad[Domain::Word.index()])
    };
    let (regard_before, bad_before) = (regard(&sc.sim, x), bad_word(&sc.sim, x));
    // At the hearth, where every routine word is told, the creditor's people tell it.
    sc.sim
        .rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .word
        .share_routine = 1.0;
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.share_word_for_tests(ctx, teller, x));
    let word = &sc.sim.people().word;
    assert!(word.has_heard(x, claim.id));
    let tie = sc
        .sim
        .people()
        .ties
        .tie(x, elder, day, &tp)
        .expect("a tie now");
    assert_eq!(tie.reason.map(|r| r.act), Some(Act::DefaultedOthers));
    let heard_bad = f64::from(tie.bad[Domain::Word.index()] - bad_before);
    assert!(
        (heard_bad - tp.acts[Act::DefaultedOthers as usize].bad).abs() < 1e-4,
        "{heard_bad}"
    );
    assert!(regard(&sc.sim, x) < regard_before, "{regard_before}");
    // Heard again, it teaches nothing more.
    sc.sim.with_ctx_for_tests(|pop, ctx| {
        pop.share_word_for_tests(ctx, lenders[lenders.len() - 1], x)
    });
    let again = sc.sim.people().ties.tie(x, elder, day, &tp).expect("tie");
    assert_eq!(again.bad, tie.bad);
    // The debtor's own people, told it, think no worse of their elder and do not tell it on.
    let kin: Vec<PermanentId> = members(&sc.sim, sc.borrower)
        .into_iter()
        .filter(|&m| m != elder)
        .collect();
    {
        let k = *kin
            .first()
            .expect("the debtor's household has another member");
        let kin_before = sc.sim.people().ties.tie(k, elder, day, &tp);
        sc.sim
            .with_ctx_for_tests(|pop, ctx| pop.share_word_for_tests(ctx, x, k));
        assert!(sc.sim.people().word.has_heard(k, claim.id));
        assert_eq!(sc.sim.people().ties.tie(k, elder, day, &tp), kin_before);
        sc.sim
            .with_ctx_for_tests(|pop, ctx| pop.share_word_for_tests(ctx, k, y));
        assert!(!sc.sim.people().word.has_heard(y, claim.id));
    }
    // Nor does the elder think the worse of themselves.
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.share_word_for_tests(ctx, x, elder));
    assert!(sc.sim.people().word.has_heard(elder, claim.id));
    assert!(sc.sim.people().ties.tie(elder, elder, day, &tp).is_none());
    // One of another household, told it, tells it on.
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.share_word_for_tests(ctx, x, y));
    assert!(sc.sim.people().word.has_heard(y, claim.id));
    assert_eq!(
        sc.sim
            .people()
            .ties
            .tie(y, elder, day, &tp)
            .and_then(|t| t.reason)
            .map(|r| r.act),
        Some(Act::DefaultedOthers)
    );
    // It saves and loads exactly.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sc.sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "default word",
    )
    .expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let (a, b) = (
        persist::encode_sections(&sc.sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (s, t) in a.iter().zip(&b) {
        assert!(s.bytes == t.bytes, "section `{}` differs", s.tag);
    }
    // It is news for `news_days` from the day it failed, then let go and told no more.
    let wp = sc.sim.rules().people.word.clone();
    let news = i64::from(wp.news_days);
    let gone_on = |today: i64| {
        let mut w = sc.sim.people().word.clone();
        w.prune(today, news, i64::from(wp.sickness_days), 10_000);
        !w.claims
            .iter()
            .any(|c| c.kind == ClaimKind::Defaulted && c.event == Some(loan))
    };
    assert!(!gone_on(claim.day + news));
    assert!(gone_on(claim.day + news + 1));
}

#[test]
fn word_of_wages_a_workshop_left_unpaid_goes_about_too() {
    // Backlog D: a workshop unpaid when its wages fall due is word as a loan failed is, naming
    // its owners' elder; the worker's household holds it at first hand.
    use civ_agents::word::ClaimKind;
    let mut sc = village();
    let sickle = sc
        .sim
        .rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/sickle")
        .expect("sickle");
    feed(&mut sc.sim, sc.lender, 0.0);
    let founder = sc.sim.people().household(sc.lender).expect("owner").members[0];
    let (owner, worker, grain) = (sc.lender, sc.borrower, sc.grain);
    let firm = sc
        .sim
        .with_ctx_for_tests(|pop, ctx| pop.found_firm_for_tests(ctx, owner, founder, sickle))
        .expect("a workshop");
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.owe_wages_for_tests(ctx, firm, worker, grain, 6.0));
    let due = sc.sim.now().day_index() + i64::from(sc.sim.rules().people.credit.wage_days);
    repay(&mut sc.sim, due - 1);
    assert!(
        !sc.sim
            .people()
            .word
            .claims
            .iter()
            .any(|c| c.kind == ClaimKind::Defaulted),
        "nothing to tell before it falls due"
    );
    repay(&mut sc.sim, due);
    let wages = loans(&sc.sim)
        .into_iter()
        .find(|c| c.debtor == firm)
        .expect("the wages");
    assert_eq!(wages.standing, Standing::Defaulted);
    let said: Vec<_> = sc
        .sim
        .people()
        .word
        .claims
        .iter()
        .filter(|c| c.kind == ClaimKind::Defaulted)
        .copied()
        .collect();
    assert_eq!(said.len(), 1, "{said:?}");
    assert_eq!(said[0].event, Some(wages.id));
    let elder = said[0].subject.expect("names the owners' elder");
    let owners = sc
        .sim
        .people()
        .household(owner)
        .expect("owners")
        .members
        .clone();
    assert!(owners.contains(&elder));
    for &m in &sc.sim.people().household(worker).expect("worker").members {
        assert!(sc.sim.people().word.has_heard(m, said[0].id));
    }
    let words = civ_sim::frames::word::claim_words(&sc.sim, &said[0]);
    assert!(
        words.contains("did not pay") && words.contains("the wages it owed (6 kg of"),
        "{words}"
    );
}

/// Household `h`'s members, in order.
fn members_of(sim: &Sim, h: PermanentId) -> Vec<PermanentId> {
    let mut m = sim
        .people()
        .household(h)
        .expect("household")
        .members
        .clone();
    m.sort_unstable();
    m
}

/// The members of every household but those of `not`, household by household.
fn people_but(sim: &Sim, not: &[PermanentId]) -> Vec<PermanentId> {
    sim.people()
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| !not.contains(&h.id) && !h.members.is_empty())
        .flat_map(|h| h.members.iter().copied())
        .collect()
}

/// The claims of kind `kind` the word book holds.
fn claims_of(sim: &Sim, kind: civ_agents::word::ClaimKind) -> Vec<civ_agents::word::Claim> {
    sim.people()
        .word
        .claims
        .iter()
        .filter(|c| c.kind == kind)
        .copied()
        .collect()
}

/// `teller` and `listener` keep company at the hearth, where every routine word is told.
fn hearth(sim: &mut Sim, teller: PermanentId, listener: PermanentId) {
    sim.rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .word
        .share_routine = 1.0;
    sim.with_ctx_for_tests(|pop, ctx| pop.share_word_for_tests(ctx, teller, listener));
}

/// Every save section of `sim` is the same after a save and a load.
fn saves_exactly(sim: &mut Sim) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "debt word").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let (a, b) = (
        persist::encode_sections(sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (s, t) in a.iter().zip(&b) {
        assert!(s.bytes == t.bytes, "section `{}` differs", s.tag);
    }
}

#[test]
fn word_of_a_loan_repaid_goes_about_and_those_who_hear_it_think_the_better_of_the_debtor() {
    // Backlog D (research 08-07 §5.3): a loan repaid is word told by contact, as a default is.
    // The creditor's people hold it at first hand; others hear it at the hearth and think the
    // better of the debtor's elder in their word; the debtor's own household neither learns from
    // it nor tells it.
    use civ_agents::ties::{Act, Domain};
    use civ_agents::word::ClaimKind;
    let mut sc = village();
    credit(&mut sc.sim).pay_base = 50.0;
    weigh(&mut sc.sim);
    let (i, c) = loan_of(&sc.sim, sc.borrower);
    feed(&mut sc.sim, sc.borrower, f64::from(c.owed) + 200.0);
    repay(&mut sc.sim, c.due - 1);
    assert!(
        claims_of(&sc.sim, ClaimKind::Repaid).is_empty(),
        "nothing to tell before it is repaid"
    );
    repay(&mut sc.sim, c.due);
    assert_eq!(loans(&sc.sim)[i].standing, Standing::Repaid);
    assert!(claims_of(&sc.sim, ClaimKind::Defaulted).is_empty());
    let said = claims_of(&sc.sim, ClaimKind::Repaid);
    assert_eq!(said.len(), 1, "{said:?}");
    let claim = said[0];
    assert_eq!((claim.event, claim.day), (Some(c.id), c.due));
    let elder = claim.subject.expect("names the debtor's elder");
    assert!(members_of(&sc.sim, sc.borrower).contains(&elder));
    assert_eq!(
        Some(claim.settlement),
        sc.sim
            .people()
            .household(sc.borrower)
            .and_then(|x| x.settlement)
    );
    // The creditor's people hold it at first hand; nobody else has heard it.
    let lenders = members_of(&sc.sim, sc.lender);
    let word = &sc.sim.people().word;
    for &m in &lenders {
        assert!(word.has_heard(m, claim.id));
    }
    let others = people_but(&sc.sim, &[sc.borrower, sc.lender]);
    assert!(others.len() >= 2, "two people of other households");
    let (x, y) = (others[0], others[1]);
    for &p in others.iter().chain(&members_of(&sc.sim, sc.borrower)) {
        assert!(!word.has_heard(p, claim.id));
    }
    let words = civ_sim::frames::word::claim_words(&sc.sim, &claim);
    assert!(
        words.contains("repaid what") && words.contains(" lent it (") && words.contains("kg of"),
        "{words}"
    );
    // Told at the hearth by one of the creditor's people, one of a third household thinks the
    // better of the elder, by exactly the act's good evidence in `word`.
    let day = sc.sim.now().day_index();
    let tp = sc.sim.rules().people.ties.clone();
    let regard = |sim: &Sim, p: PermanentId| sim.people().ties.regard(p, elder, day, &tp);
    let good_word = |sim: &Sim, p: PermanentId| {
        sim.people()
            .ties
            .tie(p, elder, day, &tp)
            .map_or(tp.prior as f32, |t| t.good[Domain::Word.index()])
    };
    let (regard_before, good_before) = (regard(&sc.sim, x), good_word(&sc.sim, x));
    hearth(&mut sc.sim, lenders[0], x);
    assert!(sc.sim.people().word.has_heard(x, claim.id));
    let tie = sc
        .sim
        .people()
        .ties
        .tie(x, elder, day, &tp)
        .expect("a tie now");
    assert_eq!(tie.reason.map(|r| r.act), Some(Act::RepaidOthers));
    let heard_good = f64::from(tie.good[Domain::Word.index()] - good_before);
    assert!(
        (heard_good - tp.acts[Act::RepaidOthers as usize].good).abs() < 1e-4,
        "{heard_good}"
    );
    assert!(regard(&sc.sim, x) > regard_before, "{regard_before}");
    assert_eq!(
        civ_agents::ties::Reason {
            times: 1,
            ..tie.reason.expect("a reason")
        }
        .words()
        .split(", last in")
        .next(),
        Some("paid another household what they owed it, as they heard")
    );
    // Heard again, it teaches nothing more.
    hearth(&mut sc.sim, lenders[lenders.len() - 1], x);
    let again = sc.sim.people().ties.tie(x, elder, day, &tp).expect("tie");
    assert_eq!(again.good, tie.good);
    // The debtor's own people, told it, learn nothing of their elder from it and do not tell it.
    let kin: Vec<PermanentId> = members_of(&sc.sim, sc.borrower)
        .into_iter()
        .filter(|&m| m != elder)
        .collect();
    let k = *kin
        .first()
        .expect("the debtor's household has another member");
    let kin_before = sc.sim.people().ties.tie(k, elder, day, &tp);
    hearth(&mut sc.sim, x, k);
    assert!(sc.sim.people().word.has_heard(k, claim.id));
    assert_eq!(sc.sim.people().ties.tie(k, elder, day, &tp), kin_before);
    hearth(&mut sc.sim, k, y);
    assert!(!sc.sim.people().word.has_heard(y, claim.id));
    // One of another household, told it, tells it on.
    hearth(&mut sc.sim, x, y);
    assert!(sc.sim.people().word.has_heard(y, claim.id));
    assert_eq!(
        sc.sim
            .people()
            .ties
            .tie(y, elder, day, &tp)
            .and_then(|t| t.reason)
            .map(|r| r.act),
        Some(Act::RepaidOthers)
    );
    saves_exactly(&mut sc.sim);
    // It is news for `news_days` from the day it was repaid, then let go.
    let wp = sc.sim.rules().people.word.clone();
    let news = i64::from(wp.news_days);
    let gone_on = |today: i64| {
        let mut w = sc.sim.people().word.clone();
        w.prune(today, news, i64::from(wp.sickness_days), 10_000);
        !w.claims
            .iter()
            .any(|x| x.kind == ClaimKind::Repaid && x.event == Some(c.id))
    };
    assert!(!gone_on(claim.day + news));
    assert!(gone_on(claim.day + news + 1));
}

#[test]
fn word_of_wages_a_workshop_paid_when_due_goes_about_too() {
    // Backlog D: wages a workshop owed and paid when they fell due are word as a loan repaid is,
    // naming its owners' elder; the worker's household holds it at first hand.
    use civ_agents::word::ClaimKind;
    let mut sc = village();
    let sickle = sc
        .sim
        .rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/sickle")
        .expect("sickle");
    let founder = sc.sim.people().household(sc.lender).expect("owner").members[0];
    let (owner, worker, grain) = (sc.lender, sc.borrower, sc.grain);
    let firm = sc
        .sim
        .with_ctx_for_tests(|pop, ctx| pop.found_firm_for_tests(ctx, owner, founder, sickle))
        .expect("a workshop");
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.owe_wages_for_tests(ctx, firm, worker, grain, 6.0));
    let due = sc.sim.now().day_index() + i64::from(sc.sim.rules().people.credit.wage_days);
    repay(&mut sc.sim, due - 1);
    assert!(claims_of(&sc.sim, ClaimKind::Repaid).is_empty());
    repay(&mut sc.sim, due);
    let wages = loans(&sc.sim)
        .into_iter()
        .find(|c| c.debtor == firm)
        .expect("the wages");
    assert_eq!(wages.standing, Standing::Repaid);
    let said = claims_of(&sc.sim, ClaimKind::Repaid);
    assert_eq!(said.len(), 1, "{said:?}");
    assert_eq!(said[0].event, Some(wages.id));
    let elder = said[0].subject.expect("names the owners' elder");
    assert!(members_of(&sc.sim, owner).contains(&elder));
    for &m in &members_of(&sc.sim, worker) {
        assert!(sc.sim.people().word.has_heard(m, said[0].id));
    }
    for &m in &members_of(&sc.sim, owner) {
        assert!(!sc.sim.people().word.has_heard(m, said[0].id));
    }
    let words = civ_sim::frames::word::claim_words(&sc.sim, &said[0]);
    assert!(
        words.contains(" paid ") && words.contains("the wages it owed (6 kg of"),
        "{words}"
    );
}

#[test]
fn word_of_a_guarantor_called_on_goes_about_and_so_does_the_debtor_repaying_it() {
    // Backlog D: a guarantor called on that pays is word as a loan repaid is, naming its elder,
    // held first by the lender's people; one who hears it thinks the better of that elder. What
    // the debtor then repays the guarantor is told as a loan repaid, held first by the
    // guarantor's people.
    use civ_agents::ties::Act;
    use civ_agents::word::ClaimKind;
    let mut sc = village();
    let cp = credit(&mut sc.sim);
    cp.p_max = 0.05;
    cp.field_recovery = 0.0;
    cp.guarantor_regard = -1.0;
    cp.pay_base = 50.0;
    cp.max_extensions = 0;
    let guarantor = sc
        .sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| !h.members.is_empty() && h.id != sc.borrower && h.id != sc.lender)
        .map(|h| h.id)
        .min()
        .expect("a third household");
    feed(&mut sc.sim, guarantor, 3000.0);
    let (tp, today) = (sc.sim.rules().people.ties.clone(), sc.sim.now().day_index());
    let (from, to) = (
        members_of(&sc.sim, sc.lender),
        members_of(&sc.sim, guarantor),
    );
    let pop = sc.sim.people_mut_for_tests();
    for &x in &from {
        for &y in &to {
            pop.ties.record(x, y, Act::Hearth, 400.0, 0.0, today, &tp);
        }
    }
    weigh(&mut sc.sim);
    let (i, c) = loan_of(&sc.sim, sc.borrower);
    let Some(Pledge::Guarantor(g)) = c.pledge else {
        panic!("lent with a guarantor: {c:?}");
    };
    feed(&mut sc.sim, sc.borrower, 0.0);
    let grace = i64::from(sc.sim.rules().people.credit.grace_days);
    let failed_on = c.due + grace + 1;
    repay(&mut sc.sim, failed_on);
    assert!(loans(&sc.sim)[i].by_guarantor > 0.0);
    // The default is told, and so is the guarantor's paying.
    assert_eq!(claims_of(&sc.sim, ClaimKind::Defaulted).len(), 1);
    let said = claims_of(&sc.sim, ClaimKind::GuarantorPaid);
    assert_eq!(said.len(), 1, "{said:?}");
    let claim = said[0];
    assert_eq!((claim.event, claim.day), (Some(c.id), failed_on));
    let elder = claim.subject.expect("names the guarantor's elder");
    assert!(members_of(&sc.sim, g).contains(&elder));
    assert_eq!(
        Some(claim.settlement),
        sc.sim.people().household(g).and_then(|x| x.settlement)
    );
    // (Whichever household lent: the guarantor's elder vouched to it.)
    let lenders = members_of(&sc.sim, c.creditor);
    for &m in &lenders {
        assert!(sc.sim.people().word.has_heard(m, claim.id));
    }
    for &m in &members_of(&sc.sim, g) {
        assert!(!sc.sim.people().word.has_heard(m, claim.id));
    }
    let words = civ_sim::frames::word::claim_words(&sc.sim, &claim);
    assert!(
        words.contains(", called on as guarantor, paid ") && words.contains(" owed it, on "),
        "{words}"
    );
    // One of a fourth household, told it, thinks the better of the guarantor's elder.
    let others = people_but(&sc.sim, &[sc.borrower, sc.lender, c.creditor, g]);
    let x = *others.first().expect("one of a fourth household");
    let day = sc.sim.now().day_index();
    hearth(&mut sc.sim, lenders[0], x);
    assert!(sc.sim.people().word.has_heard(x, claim.id));
    assert_eq!(
        sc.sim
            .people()
            .ties
            .tie(x, elder, day, &tp)
            .and_then(|t| t.reason)
            .map(|r| r.act),
        Some(Act::RepaidOthers)
    );
    // The guarantor's own people do not tell it.
    let y = others.get(1).copied().expect("another of other households");
    let own = members_of(&sc.sim, g)[0];
    sc.sim.with_ctx_for_tests(|pop, ctx| {
        pop.share_word_for_tests(ctx, lenders[0], own);
        pop.share_word_for_tests(ctx, own, y)
    });
    assert!(sc.sim.people().word.has_heard(own, claim.id));
    assert!(!sc.sim.people().word.has_heard(y, claim.id));
    saves_exactly(&mut sc.sim);
    // The debtor repays the guarantor: told as a loan repaid, held first by the guarantor's people.
    let recourse = loans(&sc.sim)
        .into_iter()
        .find(|r| r.recourse == Some(c.id))
        .expect("the guarantor's claim");
    feed(&mut sc.sim, sc.borrower, f64::from(recourse.owed) + 200.0);
    repay(&mut sc.sim, recourse.due);
    let back = loans(&sc.sim)
        .into_iter()
        .find(|r| r.id == recourse.id)
        .expect("the guarantor's claim");
    assert_eq!(back.standing, Standing::Repaid, "{back:?}");
    let said = claims_of(&sc.sim, ClaimKind::Repaid);
    assert_eq!(said.len(), 1, "{said:?}");
    assert_eq!(said[0].event, Some(recourse.id));
    assert!(members_of(&sc.sim, sc.borrower).contains(&said[0].subject.expect("an elder")));
    for &m in &members_of(&sc.sim, g) {
        assert!(sc.sim.people().word.has_heard(m, said[0].id));
    }
    let words = civ_sim::frames::word::claim_words(&sc.sim, &said[0]);
    assert!(
        words.contains(" what it had paid for it as guarantor ("),
        "{words}"
    );
}

#[test]
fn a_default_given_time_by_a_finding_and_then_repaid_is_told_as_repaid() {
    // Backlog D: a loan a finding under debts heard gives time is open again; repaid in full by
    // its new due day, it is told as any loan repaid, beside the word of its default.
    use civ_agents::polity::{DebtRemedy, DebtsDef};
    use civ_agents::word::ClaimKind;
    let (mut sc, loan) = defaulted();
    let time = DebtsDef {
        remedy: DebtRemedy::Time,
        share: 0.5,
        extend_days: 180,
    };
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.debt_found_for_tests(ctx, loan, 1, time));
    let c = loans(&sc.sim)
        .into_iter()
        .find(|c| c.id == loan)
        .expect("kept");
    feed(&mut sc.sim, sc.borrower, c.left() + 200.0);
    repay(&mut sc.sim, c.due);
    let c = loans(&sc.sim)
        .into_iter()
        .find(|c| c.id == loan)
        .expect("kept");
    assert_eq!(c.standing, Standing::Repaid, "{c:?}");
    let failed = claims_of(&sc.sim, ClaimKind::Defaulted);
    let repaid = claims_of(&sc.sim, ClaimKind::Repaid);
    assert_eq!((failed.len(), repaid.len()), (1, 1), "{repaid:?}");
    assert_eq!((failed[0].event, repaid[0].event), (Some(loan), Some(loan)));
    assert_eq!(failed[0].subject, repaid[0].subject);
    for &m in &members_of(&sc.sim, sc.lender) {
        assert!(sc.sim.people().word.has_heard(m, repaid[0].id));
    }
}

/// A village whose borrower took a loan from its polity's common store, under a law of lending
/// from the store, with one of a third household keeping the store when `kept`: the scene, the
/// loan, and the keeper.
fn lent_by_the_store(kept: bool) -> (Scene, Claim, Option<PermanentId>) {
    let mut sc = village();
    let cp = credit(&mut sc.sim);
    cp.pay_base = 50.0;
    cp.max_extensions = 0;
    let at = sc
        .sim
        .people()
        .household(sc.borrower)
        .and_then(|x| x.settlement)
        .expect("settled");
    let pi = sc
        .sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == at)
        .expect("a polity");
    let goods = sc.sim.rules().catalog.goods.len();
    {
        let store = &mut sc.sim.people_mut_for_tests().polities[pi].stores;
        store.resize(goods, 0.0);
        store[sc.grain] = 2000.0;
    }
    law_in_force(
        &mut sc.sim,
        sc.borrower,
        "core:policy/lend_from_store",
        civ_agents::polity::IssueKind::FoodShort,
    );
    let keeper = kept.then(|| {
        let keeper = people_but(&sc.sim, &[sc.borrower, sc.lender])[0];
        let law = law_in_force(
            &mut sc.sim,
            sc.borrower,
            "core:policy/keep_store",
            civ_agents::polity::IssueKind::StoreUnkept,
        );
        let p = &mut sc.sim.people_mut_for_tests().polities[pi];
        p.laws
            .iter_mut()
            .find(|l| l.id == law)
            .expect("the law")
            .holder = Some(keeper);
        assert_eq!(p.keeper().map(|k| k.0), Some(keeper));
        keeper
    });
    weigh(&mut sc.sim);
    let (_, c) = loan_of(&sc.sim, sc.borrower);
    assert_eq!(
        c.creditor,
        sc.sim.people().polities[pi].id,
        "the store lent: {c:?}"
    );
    (sc, c, keeper)
}

#[test]
fn word_of_a_default_to_the_common_store_is_the_keepers_and_goes_about() {
    // Backlog D (research 08-07 §5.3; ADR-0027 §4: the keeper carries the store's loans): a loan
    // from the common store left unpaid is word told as one between households is. Its keeper
    // holds it at first hand, nobody else; those told it think the worse of the debtor's elder by
    // an act of the store's own; the debtor's household neither learns from it nor tells it.
    use civ_agents::ties::{Act, Domain, Reason};
    use civ_agents::word::ClaimKind;
    let (mut sc, c, keeper) = lent_by_the_store(true);
    let keeper = keeper.expect("kept");
    let day = sc.sim.now().day_index();
    let tp = sc.sim.rules().people.ties.clone();
    let bad_of = |sim: &Sim, p: PermanentId, q: PermanentId| {
        sim.people()
            .ties
            .tie(p, q, day, &tp)
            .map_or(tp.prior as f32, |t| t.bad[Domain::Word.index()])
    };
    let keeper_before: Vec<(PermanentId, f32)> = members_of(&sc.sim, sc.borrower)
        .into_iter()
        .map(|m| (m, bad_of(&sc.sim, keeper, m)))
        .collect();
    feed(&mut sc.sim, sc.borrower, 0.0);
    let grace = i64::from(sc.sim.rules().people.credit.grace_days);
    repay(&mut sc.sim, c.due + grace + 1);
    let (_, failed) = loan_of(&sc.sim, sc.borrower);
    assert_eq!(failed.standing, Standing::Defaulted);
    let said = claims_of(&sc.sim, ClaimKind::Defaulted);
    assert_eq!(said.len(), 1, "{said:?}");
    let claim = said[0];
    assert_eq!(claim.event, Some(c.id));
    let elder = claim.subject.expect("names the debtor's elder");
    let keeper_before = keeper_before
        .iter()
        .find(|m| m.0 == elder)
        .expect("the elder is of the debtor's household")
        .1;
    let bad_word = |sim: &Sim, p: PermanentId| bad_of(sim, p, elder);
    assert_eq!(
        Some(claim.settlement),
        sc.sim
            .people()
            .household(sc.borrower)
            .and_then(|x| x.settlement)
    );
    // The keeper holds it at first hand, and thinks the worse of the elder for it; nobody else
    // has heard it, not even the keeper's household or the store's other lenders.
    let word = &sc.sim.people().word;
    assert!(word.has_heard(keeper, claim.id));
    for p in people_but(&sc.sim, &[]) {
        assert_eq!(word.has_heard(p, claim.id), p == keeper, "{p:?}");
    }
    let tie = sc
        .sim
        .people()
        .ties
        .tie(keeper, elder, day, &tp)
        .expect("the keeper's tie");
    assert_eq!(tie.reason.map(|r| r.act), Some(Act::DefaultedStore));
    assert!(
        (f64::from(tie.bad[Domain::Word.index()] - keeper_before)
            - tp.acts[Act::DefaultedStore as usize].bad)
            .abs()
            < 1e-4
    );
    let words = civ_sim::frames::word::claim_words(&sc.sim, &claim);
    assert!(
        words.contains("did not repay what the common store of ")
            && words.contains(" lent it (")
            && words.contains("kg of"),
        "{words}"
    );
    // Told at the hearth by the keeper, one of the lender's household thinks the worse of the
    // elder, by exactly the act's bad evidence in `word`, and the reason says so.
    let x = members_of(&sc.sim, sc.lender)[0];
    let before = bad_word(&sc.sim, x);
    hearth(&mut sc.sim, keeper, x);
    assert!(sc.sim.people().word.has_heard(x, claim.id));
    let tie = sc
        .sim
        .people()
        .ties
        .tie(x, elder, day, &tp)
        .expect("a tie now");
    assert_eq!(tie.reason.map(|r| r.act), Some(Act::DefaultedStore));
    assert!(
        (f64::from(tie.bad[Domain::Word.index()] - before)
            - tp.acts[Act::DefaultedStore as usize].bad)
            .abs()
            < 1e-4
    );
    assert_eq!(
        Reason {
            times: 1,
            ..tie.reason.expect("a reason")
        }
        .words()
        .split(", last in")
        .next(),
        Some("did not repay what the common store lent")
    );
    // Heard again, nothing more.
    hearth(&mut sc.sim, keeper, x);
    let again = sc.sim.people().ties.tie(x, elder, day, &tp).expect("tie");
    assert_eq!(again.bad, tie.bad);
    // The debtor's own people, told it, think no worse of their elder and do not tell it on.
    let k = members_of(&sc.sim, sc.borrower)
        .into_iter()
        .find(|&m| m != elder)
        .expect("the debtor's household has another member");
    let y = members_of(&sc.sim, sc.lender)
        .into_iter()
        .find(|&m| m != x)
        .expect("the lender's household has another member");
    let kin_before = sc.sim.people().ties.tie(k, elder, day, &tp);
    hearth(&mut sc.sim, x, k);
    assert!(sc.sim.people().word.has_heard(k, claim.id));
    assert_eq!(sc.sim.people().ties.tie(k, elder, day, &tp), kin_before);
    hearth(&mut sc.sim, k, y);
    assert!(!sc.sim.people().word.has_heard(y, claim.id));
    saves_exactly(&mut sc.sim);
}

#[test]
fn word_of_a_loan_repaid_into_the_common_store_goes_about_too() {
    // Backlog D: a loan from the store repaid in full is told as one between households is, held
    // at first hand by the store's keeper; those told think the better of the debtor's elder.
    use civ_agents::ties::{Act, Domain};
    use civ_agents::word::ClaimKind;
    let (mut sc, c, keeper) = lent_by_the_store(true);
    let keeper = keeper.expect("kept");
    feed(&mut sc.sim, sc.borrower, f64::from(c.owed) + 200.0);
    repay(&mut sc.sim, c.due - 1);
    assert!(claims_of(&sc.sim, ClaimKind::Repaid).is_empty());
    repay(&mut sc.sim, c.due);
    let (_, paid) = loan_of(&sc.sim, sc.borrower);
    assert_eq!(paid.standing, Standing::Repaid);
    assert!(claims_of(&sc.sim, ClaimKind::Defaulted).is_empty());
    let said = claims_of(&sc.sim, ClaimKind::Repaid);
    assert_eq!(said.len(), 1, "{said:?}");
    let claim = said[0];
    assert_eq!((claim.event, claim.day), (Some(c.id), c.due));
    let elder = claim.subject.expect("names the debtor's elder");
    assert!(members_of(&sc.sim, sc.borrower).contains(&elder));
    for p in people_but(&sc.sim, &[]) {
        assert_eq!(
            sc.sim.people().word.has_heard(p, claim.id),
            p == keeper,
            "{p:?}"
        );
    }
    let day = sc.sim.now().day_index();
    let tp = sc.sim.rules().people.ties.clone();
    assert_eq!(
        sc.sim
            .people()
            .ties
            .tie(keeper, elder, day, &tp)
            .and_then(|t| t.reason)
            .map(|r| r.act),
        Some(Act::RepaidStore)
    );
    let words = civ_sim::frames::word::claim_words(&sc.sim, &claim);
    assert!(
        words.contains("repaid what the common store of ")
            && words.contains(" lent it (")
            && words.contains("kg of"),
        "{words}"
    );
    let x = members_of(&sc.sim, sc.lender)[0];
    let good_word = |sim: &Sim| {
        sim.people()
            .ties
            .tie(x, elder, day, &tp)
            .map_or(tp.prior as f32, |t| t.good[Domain::Word.index()])
    };
    let before = good_word(&sc.sim);
    hearth(&mut sc.sim, keeper, x);
    let tie = sc
        .sim
        .people()
        .ties
        .tie(x, elder, day, &tp)
        .expect("a tie now");
    assert_eq!(tie.reason.map(|r| r.act), Some(Act::RepaidStore));
    assert!(
        (f64::from(good_word(&sc.sim) - before) - tp.acts[Act::RepaidStore as usize].good).abs()
            < 1e-4
    );
    saves_exactly(&mut sc.sim);
}

#[test]
fn a_loan_of_a_store_nobody_keeps_has_nobody_to_tell_it() {
    // Backlog D: with no keeper, nobody saw what came into the store and what did not; a loan
    // from it, failed or repaid, is not told.
    use civ_agents::word::ClaimKind;
    let (mut sc, c, keeper) = lent_by_the_store(false);
    assert!(keeper.is_none());
    feed(&mut sc.sim, sc.borrower, 0.0);
    let grace = i64::from(sc.sim.rules().people.credit.grace_days);
    repay(&mut sc.sim, c.due + grace + 1);
    assert_eq!(
        loan_of(&sc.sim, sc.borrower).1.standing,
        Standing::Defaulted
    );
    assert!(claims_of(&sc.sim, ClaimKind::Defaulted).is_empty());
    let (mut sc, c, _) = lent_by_the_store(false);
    feed(&mut sc.sim, sc.borrower, f64::from(c.owed) + 200.0);
    repay(&mut sc.sim, c.due);
    assert_eq!(loan_of(&sc.sim, sc.borrower).1.standing, Standing::Repaid);
    assert!(claims_of(&sc.sim, ClaimKind::Repaid).is_empty());
}
