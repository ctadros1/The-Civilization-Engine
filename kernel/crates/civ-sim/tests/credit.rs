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

#[test]
fn a_keeper_whose_notes_are_out_lends_in_them_and_keeps_its_grain() {
    let mut sc = village();
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
