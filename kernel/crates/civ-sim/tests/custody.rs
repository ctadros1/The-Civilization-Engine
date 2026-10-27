//! Custody (M10d slice EG, second part; ADR-0027 §4): a household with no roof of its own leaves
//! what keeps better under one with a neighbour who has room, keeps some days of its food in hand,
//! pays a keeper who is not close kin a share each month, and takes it all back once it has a
//! roof; the goods are the depositor's throughout and are conserved. Where someone of the
//! keeper's household can write, they write a receipt (third part), which the observer tells of,
//! as notes the depositor presents at the keeper's door to take the goods back (slice EH).

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ledger::Channel;
use civ_agents::person::KnowSource;
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

fn world(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Custody".to_owned(),
            seed,
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
    .expect("generates")
}

fn good(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect(id)
}

/// A world where household `d` has no roof of its own and 5 t of provisions, and `k`, of its
/// settlement and not its close kin, alone has a roof with room, on `d`'s review day.
fn scene() -> (Sim, PermanentId, PermanentId) {
    let mut sim = world(3);
    sim.advance_minutes(DAY).expect("lives");
    let (d, settlement) = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| !x.members.is_empty())
        .find_map(|x| x.settlement.map(|s| (x.id, s)))
        .expect("a household");
    let kin = sim.people().kin_households_for_tests(d);
    let k = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .find(|x| {
            x.id != d
                && !x.members.is_empty()
                && x.settlement == Some(settlement)
                && !kin.contains(&x.id)
        })
        .map(|x| x.id)
        .expect("a neighbour who is not close kin");
    let review = i64::from(sim.rules().people.market.review_days.max(1));
    while (d.get() as i64 + sim.now().day_index()).rem_euclid(review) != 0 {
        sim.advance_minutes(DAY).expect("lives");
    }
    let provisions = good(&sim, "core:good/provisions");
    let factor = sim.rules().people.household.raised_store_factor;
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.settlement != Some(settlement) {
            continue;
        }
        x.sheltered = x.id == k;
        x.keeping = civ_agents::person::Keeping {
            raised_kg: 0.0,
            roofed_kg: if x.id == k { 100_000.0 } else { 0.0 },
            raised_factor: factor,
        };
        if x.id == d {
            x.stores[provisions] = 5_000.0;
        }
    }
    (sim, d, k)
}

#[test]
fn a_household_with_no_roof_leaves_its_goods_with_a_neighbour_and_takes_them_back() {
    let (mut sim, d, k) = scene();
    let provisions = good(&sim, "core:good/provisions");
    // Its provisions as they are now, spoiled to this minute.
    let before = {
        let x = sim.people().household(d).expect("household");
        let rules = sim.rules();
        civ_agents::population::stores_now(x, sim.now(), &rules.people, &rules.catalog.goods)
            [provisions]
    };
    // What it keeps to its next harvest is at least its margin of days.
    let keep_back = {
        let x = sim.people().household(d).expect("household");
        let need = x.members.len() as f64 * sim.rules().people.household.daily_kcal_per_person;
        sim.rules().people.household.harvest_margin_days * need
    };
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    let (left, _) = sim.people().custody_for_tests(d);
    let (_, keeps) = sim.people().custody_for_tests(k);
    assert!(left > 100.0, "{left} kg left with a neighbour");
    assert!((left - keeps).abs() < 1e-9, "{left} against {keeps}");
    assert!(sim.people().transfers.get(Channel::Custody, provisions) > 100.0);
    // It kept in hand the food it keeps to its next harvest, and left what it could spare.
    let x = sim.people().household(d).expect("household");
    let food = civ_agents::person::stock_kcal(&x.stores, &sim.rules().catalog.goods);
    assert!(food >= keep_back, "{food} kcal in hand against {keep_back}");
    assert!(x.stores[provisions] < before - 100.0);
    // The goods are the same goods, held apart.
    let mine = |sim: &Sim| sim.people().household(d).expect("household").stores[provisions];
    let kept = sim.people().kept[0].stores[provisions];
    assert!(
        (mine(&sim) + kept - before).abs() < 1e-6,
        "{} + {kept} against {before}",
        mine(&sim)
    );
    // A month on, the keeper, not close kin, takes its share.
    let day = sim.now().day_index();
    sim.people_mut_for_tests().kept[0].fee_day = day - 30;
    // The keeper's provisions as they are now, spoiled to this minute.
    let held = |sim: &Sim| {
        let x = sim.people().household(k).expect("keeper");
        let rules = sim.rules();
        civ_agents::population::stores_now(x, sim.now(), &rules.people, &rules.catalog.goods)
            [provisions]
    };
    let keeper_before = held(&sim);
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    let fee = sim.rules().people.market.custody_fee_share;
    assert!(
        (held(&sim) - keeper_before - kept * fee).abs() < 1e-6,
        "{} against {}",
        held(&sim) - keeper_before,
        kept * fee
    );
    // Under a roof of its own with no room for them, it leaves them where they are.
    let factor = sim.rules().people.household.raised_store_factor;
    let roof = |sim: &mut Sim, room: f64| {
        for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
            if x.id == d {
                x.sheltered = true;
                x.keeping = civ_agents::person::Keeping {
                    raised_kg: 0.0,
                    roofed_kg: room,
                    raised_factor: factor,
                };
            }
        }
    };
    roof(&mut sim, 0.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert!(sim.people().custody_for_tests(d).0 > 100.0, "still kept");
    // With room for them all, it takes everything back.
    roof(&mut sim, 100_000.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert_eq!(sim.people().custody_for_tests(d), (0.0, 0.0));
    assert!(sim.people().kept.is_empty());
    assert!(
        (mine(&sim) - (before - kept * fee)).abs() < 1e-6,
        "{} against {}",
        mine(&sim),
        before - kept * fee
    );
}

/// Gives writing to `who` of household `k` and takes it from its other members, or from all of
/// them when `who` is none.
fn writers(sim: &mut Sim, k: PermanentId, who: Option<PermanentId>) {
    let writing = sim
        .rules()
        .catalog
        .techniques
        .iter()
        .position(|t| t.id == "core:technique/writing")
        .expect("writing");
    let members = sim.people().household(k).expect("keeper").members.clone();
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if !members.contains(&p.id) {
            continue;
        }
        p.knows.retain(|t| usize::from(t.technique) != writing);
        if Some(p.id) == who {
            p.come_to_know(writing, KnowSource::Observer, now);
        }
    }
}

#[test]
fn a_keeper_who_can_write_writes_a_receipt_and_the_observer_tells_of_it() {
    // Nobody of the keeper's household can write: no receipt.
    let (mut sim, d, k) = scene();
    writers(&mut sim, k, None);
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert!(sim.people().custody_for_tests(d).0 > 100.0);
    assert_eq!(sim.people().kept[0].receipt, None);
    let settlement = sim
        .people()
        .household(k)
        .and_then(|x| x.settlement)
        .expect("settlement");
    let words = civ_sim::frames::markets::custody_words(&sim, settlement);
    assert!(
        words.starts_with("1 household keeps goods for 1 other here, ")
            && words.ends_with("kg; 0 with a written receipt."),
        "{words}"
    );
    // Its last member can: they write one, held by the depositor, on the day the goods are left.
    let (mut sim, d, k) = scene();
    let writer = *sim
        .people()
        .household(k)
        .expect("keeper")
        .members
        .iter()
        .max()
        .expect("members");
    writers(&mut sim, k, Some(writer));
    let day = sim.now().day_index();
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert!(sim.people().custody_for_tests(d).0 > 100.0);
    assert_eq!(sim.people().kept[0].receipt, Some((writer, day)));
    let words = civ_sim::frames::markets::custody_words(&sim, settlement);
    assert!(words.ends_with("kg; 1 with a written receipt."), "{words}");
    let name = |h: PermanentId| {
        let first = sim.people().household(h).expect("household").members[0];
        sim.people().name_of(first)
    };
    let letters = civ_sim::frames::people::letters_words(&sim, writer);
    let receipt = format!(
        "rote a receipt for goods {}'s household left with {}'s household",
        name(d),
        name(k)
    );
    assert!(letters.contains(&receipt), "{letters}");
    assert!(letters.contains("household, as notes"), "{letters}");
    // It is only a record: once the goods are all taken back, it is gone with them.
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == d {
            x.sheltered = true;
            x.keeping.roofed_kg = 100_000.0;
        }
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert!(sim.people().kept.is_empty());
    assert_eq!(
        civ_sim::frames::markets::custody_words(&sim, settlement),
        ""
    );
}

#[test]
fn goods_kept_for_another_save_and_load_exactly() {
    let (mut sim, d, k) = scene();
    let writer = sim.people().household(k).expect("keeper").members[0];
    writers(&mut sim, k, Some(writer));
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert!(sim.people().custody_for_tests(d).0 > 100.0);
    assert!(sim.people().kept[0].receipt.is_some());
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "kept").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    // As saved: its counters are not (as a load in transit's are not).
    let saved_of = |sim: &Sim| {
        sim.people()
            .kept
            .iter()
            .map(|k| {
                (
                    k.id,
                    k.depositor,
                    k.keeper,
                    k.since,
                    k.fee_day,
                    k.stores.clone(),
                    k.receipt,
                    k.noted,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(saved_of(&loaded), saved_of(&sim));
    assert!(!sim.people().notes.issues.is_empty(), "notes written");
    assert_eq!(loaded.people().notes, sim.people().notes);
    let same = |x: &Sim, y: &Sim, when: &str| {
        let (x, y) = (persist::encode_sections(x), persist::encode_sections(y));
        assert_eq!(x.len(), y.len());
        for (p, q) in x.iter().zip(&y) {
            assert!(p.bytes == q.bytes, "section `{}` differs {when}", p.tag);
        }
    };
    same(&sim, &loaded, "on loading");
    // The shelter the scene set by hand is derived, not saved: both as the land has it.
    sim.with_ctx_for_tests(|pop, ctx| pop.derive_shelter(ctx.land, ctx.catalog, ctx.params));
    sim.advance_minutes(2 * DAY).expect("lives");
    loaded.advance_minutes(2 * DAY).expect("lives");
    same(&sim, &loaded, "two days on");
}

#[test]
fn a_keeper_who_can_write_writes_notes_the_depositor_presents_at_its_door() {
    let (mut sim, d, k) = scene();
    let provisions = good(&sim, "core:good/provisions");
    let note = good(&sim, "core:good/note");
    let writer = sim.people().household(k).expect("keeper").members[0];
    writers(&mut sim, k, Some(writer));
    let keeper_held = |sim: &Sim| {
        let x = sim.people().household(k).expect("keeper");
        let rules = sim.rules();
        civ_agents::population::stores_now(x, sim.now(), &rules.people, &rules.catalog.goods)
            [provisions]
    };
    let keeper_before = keeper_held(&sim);
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    // The keeper, not close kin, took a year's fee at once, and wrote notes for the rest, a
    // kilogram a piece, held by the depositor: its issue in provisions.
    let lot = sim.people().kept[0].clone();
    assert!(lot.noted);
    let fee = keeper_held(&sim) - keeper_before;
    let share = 12.0 * sim.rules().people.market.custody_fee_share;
    let left = lot.stores[provisions] + fee;
    assert!(
        (fee - left * share).abs() < 1e-6,
        "{fee} against {}",
        left * share
    );
    let notes = &sim.people().notes;
    let issue = notes.issue_of(k, provisions).expect("an issue").clone();
    assert_eq!(issue.kg, 1.0);
    assert!((issue.count - lot.stores[provisions]).abs() < 1e-6);
    assert!((notes.held(d, issue.id) - issue.count).abs() < 1e-6);
    let depositor = sim.people().household(d).expect("depositor");
    assert!((depositor.stores[note] - issue.count).abs() < 1e-6);
    // A month on, a lot backing notes pays no monthly fee.
    let day = sim.now().day_index();
    sim.people_mut_for_tests().kept[0].fee_day = day - 30;
    let keeper_before = keeper_held(&sim);
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert!((keeper_held(&sim) - keeper_before).abs() < 1e-6, "no fee");
    // Under a roof of its own with room, it presents its notes and is paid at the keeper's door:
    // the provisions come back on `Redeemed`, the pieces go to the keeper and out of being.
    let mine = |sim: &Sim| {
        let x = sim.people().household(d).expect("depositor");
        let rules = sim.rules();
        civ_agents::population::stores_now(x, sim.now(), &rules.people, &rules.catalog.goods)
    };
    let before = mine(&sim)[provisions];
    let backing = sim.people().kept[0].stores[provisions];
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == d {
            x.sheltered = true;
            x.keeping.roofed_kg = 100_000.0;
        }
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert!(sim.people().kept.is_empty(), "all taken back");
    assert!((mine(&sim)[provisions] - before - backing).abs() < 1e-6);
    assert!(mine(&sim)[note].abs() < 1e-6, "no pieces left");
    assert!((sim.people().transfers.get(Channel::Redeemed, provisions) - backing).abs() < 1e-6);
    assert!((sim.people().transfers.get(Channel::Redeemed, note) - issue.count).abs() < 1e-6);
    let after = sim.people().notes.issue(issue.id).expect("issue").clone();
    assert!(after.count.abs() < 1e-6, "retired: {}", after.count);
    assert!((after.presented - issue.count).abs() < 1e-6);
    assert_eq!(after.refused, 0.0);
    let keeper = sim.people().household(k).expect("keeper");
    assert!(keeper.stores[note].abs() < 1e-6, "retired, not held");
    assert!(sim.people().notes.purse(d).is_none() && sim.people().notes.purse(k).is_none());
}

#[test]
fn pieces_the_backing_cannot_pay_are_refused_and_stay_owed() {
    let (mut sim, d, k) = scene();
    let provisions = good(&sim, "core:good/provisions");
    let writer = sim.people().household(k).expect("keeper").members[0];
    writers(&mut sim, k, Some(writer));
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    let issue = sim
        .people()
        .notes
        .issue_of(k, provisions)
        .expect("an issue")
        .clone();
    // Half the backing lost by hand, the keeper holding no provisions of its own to pay with
    // (slice EI), and its roof gone: every piece is presented.
    let half = sim.people().kept[0].stores[provisions] / 2.0;
    sim.people_mut_for_tests().kept[0].stores[provisions] = half;
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == k {
            x.sheltered = false;
            x.stores[provisions] = 0.0;
        }
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert!(sim.people().kept.is_empty(), "the backing is paid out");
    let after = sim.people().notes.issue(issue.id).expect("issue").clone();
    assert!((after.presented - issue.count).abs() < 1e-6);
    assert!(
        (after.refused - (issue.count - half)).abs() < 1e-6,
        "{} refused against {}",
        after.refused,
        issue.count - half
    );
    assert_eq!(after.refused_day, sim.now().day_index());
    // The market's summary tells of it.
    let settlement = sim
        .people()
        .household(k)
        .and_then(|x| x.settlement)
        .expect("settlement");
    let words = civ_sim::frames::markets::notes_words(&sim, settlement);
    let out = issue.count - half;
    let told = format!(
        "household's notes in provisions: {out:.0} out against 0 kg kept; {:.0} presented, \
         {out:.0} refused.",
        issue.count
    );
    assert!(words.contains(&told), "{words}");
    // The refused pieces stay with their holder, still owed.
    assert!((after.count - (issue.count - half)).abs() < 1e-6);
    assert!((sim.people().notes.held(d, issue.id) - after.count).abs() < 1e-6);
}

/// A household of `k`'s settlement other than `d` and `k`, with all its food taken away.
fn hungry_neighbour(sim: &mut Sim, d: PermanentId, k: PermanentId) -> PermanentId {
    let settlement = sim
        .people()
        .household(k)
        .and_then(|x| x.settlement)
        .expect("settlement");
    let s = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .find(|x| {
            x.id != d && x.id != k && !x.members.is_empty() && x.settlement == Some(settlement)
        })
        .map(|x| x.id)
        .expect("a neighbour");
    let food: Vec<usize> = sim
        .rules()
        .catalog
        .goods
        .iter()
        .enumerate()
        .filter(|(_, g)| g.kcal_per_kg > 0.0)
        .map(|(i, _)| i)
        .collect();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == s {
            for &g in &food {
                x.stores[g] = 0.0;
            }
        }
    }
    s
}

#[test]
fn notes_are_worth_their_good_by_belief_and_a_neighbour_short_of_it_takes_them() {
    let (mut sim, d, k) = scene();
    let provisions = good(&sim, "core:good/provisions");
    let note = good(&sim, "core:good/note");
    let writer = sim.people().household(k).expect("keeper").members[0];
    writers(&mut sim, k, Some(writer));
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    let s = hungry_neighbour(&mut sim, d, k);
    let worth = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.pay_worth_for_tests(ctx, s, note, 0.0))
            .unwrap_or(0.0)
    };
    let own = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.pay_worth_for_tests(ctx, s, provisions, 0.0))
            .unwrap_or(0.0)
    };
    // A piece is worth about a kilogram of provisions to a neighbour, less a share of the walk.
    let (w, p) = (worth(&mut sim), own(&mut sim));
    assert!(w > 0.9 * p && w <= p, "{w} against {p}");
    // Short of food, it takes notes in payment for what it sells.
    let terms =
        sim.with_ctx_for_tests(|pop, ctx| pop.terms_for_tests(ctx, s, &[(provisions, 10.0)]));
    assert!(
        terms.iter().any(|o| usize::from(o.payment) == note),
        "{terms:?}"
    );
    // Half the pieces presented at the keeper's door refused: half the worth.
    {
        let c = &mut sim.people_mut_for_tests().notes.issues[0];
        c.presented = 100.0;
        c.refused = 50.0;
    }
    // The walk's share is the same, so half the provisions' worth less it.
    let half = worth(&mut sim);
    assert!(
        (half - (w - p / 2.0)).abs() < 1e-9,
        "{half} against {w} and {p}"
    );
    // Notes pass hand to hand, piece by piece of their issue.
    let issue = sim.people().notes.issues[0].id;
    assert!(sim.with_ctx_for_tests(|pop, ctx| pop.transfer_for_tests(ctx, d, s, note, 10.0)));
    assert!((sim.people().notes.held(s, issue) - 10.0).abs() < 1e-9);
}

#[test]
fn a_holder_short_of_food_presents_its_notes_and_on_hearing_refusals_all_of_them() {
    let (mut sim, d, k) = scene();
    let provisions = good(&sim, "core:good/provisions");
    let note = good(&sim, "core:good/note");
    let writer = sim.people().household(k).expect("keeper").members[0];
    writers(&mut sim, k, Some(writer));
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    let s = hungry_neighbour(&mut sim, d, k);
    let issue = sim.people().notes.issues[0].id;
    assert!(sim.with_ctx_for_tests(|pop, ctx| pop.transfer_for_tests(ctx, d, s, note, 10.0)));
    // Short of food, it presents what brings it back toward its keep: here all ten.
    sim.with_ctx_for_tests(|pop, ctx| pop.notes_review_for_tests(ctx, s));
    let x = sim.people().household(s).expect("holder");
    assert!(
        (x.stores[provisions] - 10.0).abs() < 1e-6,
        "{}",
        x.stores[provisions]
    );
    assert!(sim.people().notes.held(s, issue) < 1e-9);
    // Fed, it holds them; hearing the keeper refused some lately, it presents them all.
    assert!(sim.with_ctx_for_tests(|pop, ctx| pop.transfer_for_tests(ctx, d, s, note, 10.0)));
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == s {
            x.stores[provisions] = 5_000.0;
        }
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.notes_review_for_tests(ctx, s));
    assert!(
        (sim.people().notes.held(s, issue) - 10.0).abs() < 1e-9,
        "kept"
    );
    let today = sim.now().day_index();
    sim.people_mut_for_tests().notes.issues[0].refused_day = today - 1;
    sim.with_ctx_for_tests(|pop, ctx| pop.notes_review_for_tests(ctx, s));
    assert!(sim.people().notes.held(s, issue) < 1e-9, "all presented");
}

#[test]
fn a_keeper_with_notes_out_lends_in_them_within_its_coverage_and_pays_from_its_own() {
    let (mut sim, d, k) = scene();
    let provisions = good(&sim, "core:good/provisions");
    let note = good(&sim, "core:good/note");
    let writer = sim.people().household(k).expect("keeper").members[0];
    writers(&mut sim, k, Some(writer));
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    // The keeper holds no provisions of its own: what is behind its notes is what was left.
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == k {
            x.stores[provisions] = 0.0;
        }
    }
    let issue = sim
        .people()
        .notes
        .issue_of(k, provisions)
        .expect("an issue")
        .clone();
    let behind = sim.people().kept[0].stores[provisions];
    let coverage = sim.rules().people.market.note_coverage;
    let per_kg = sim.rules().catalog.goods[provisions].kcal_per_kg;
    let (lends_in, kcal) = sim
        .with_ctx_for_tests(|pop, ctx| pop.note_lending_for_tests(ctx, k))
        .expect("it may lend in its notes");
    assert_eq!(lends_in, issue.id);
    let kg = behind / coverage - issue.count;
    assert!(
        (kcal - kg * per_kg).abs() < 1e-3 * kg * per_kg,
        "{kcal} against {}",
        kg * per_kg
    );
    // It lends all it may to a hungry neighbour: pieces written on `Lent`, owed more.
    let s = hungry_neighbour(&mut sim, d, k);
    assert!(sim.with_ctx_for_tests(|pop, ctx| pop.lend_notes_for_tests(ctx, (k, s), issue.id, kg)));
    let after = sim.people().notes.issue(issue.id).expect("issue").clone();
    assert!((after.count - issue.count - kg).abs() < 1e-6);
    assert!((after.lent - kg).abs() < 1e-6);
    assert!((sim.people().notes.held(s, issue.id) - kg).abs() < 1e-6);
    assert!((sim.people().transfers.get(Channel::Lent, note) - kg).abs() < 1e-6);
    assert!(sim.people().household(k).expect("keeper").stores[note].abs() < 1e-9);
    // Its coverage reached, it may lend no more.
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.note_lending_for_tests(ctx, k))
            .is_none_or(|(_, kcal)| kcal < 1e-3)
    );
    // The borrower, short of food, presents them and is paid from what others left.
    sim.with_ctx_for_tests(|pop, ctx| pop.notes_review_for_tests(ctx, s));
    let paid = sim.people().household(s).expect("borrower").stores[provisions];
    assert!(paid > 1.0, "{paid}");
    assert!((sim.people().kept[0].stores[provisions] - (behind - paid)).abs() < 1e-6);
    // With nothing left behind them, the keeper pays from its own beyond what it keeps.
    sim.people_mut_for_tests().kept[0].stores[provisions] = 0.0;
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == k {
            x.stores[provisions] = 50_000.0;
        }
        if x.id == s {
            x.stores[provisions] = 0.0;
        }
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.notes_review_for_tests(ctx, s));
    let again = sim.people().household(s).expect("borrower").stores[provisions];
    assert!(again > 1.0, "{again}");
    assert!(
        sim.people().household(k).expect("keeper").stores[provisions] < 50_000.0 - again + 1e-6
    );
    let c = sim.people().notes.issue(issue.id).expect("issue").clone();
    assert_eq!(c.refused, 0.0);
}

#[test]
fn holders_run_on_hearing_of_a_default_or_seeing_others_present_and_a_refusing_keeper_suspends() {
    let (mut sim, d, k) = scene();
    let provisions = good(&sim, "core:good/provisions");
    let note = good(&sim, "core:good/note");
    let writer = sim.people().household(k).expect("keeper").members[0];
    writers(&mut sim, k, Some(writer));
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    let s = hungry_neighbour(&mut sim, d, k);
    let issue = sim.people().notes.issues[0].id;
    let today = sim.now().day_index();
    // The keeper holds no provisions of its own: what is behind its notes is what was left.
    let set = |sim: &mut Sim, who: PermanentId, kg: f64| {
        for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
            if x.id == who {
                x.stores[provisions] = kg;
            }
        }
    };
    set(&mut sim, k, 0.0);
    // Fed, a holder keeps its pieces.
    let give = |sim: &mut Sim| {
        assert!(sim.with_ctx_for_tests(|pop, ctx| pop.transfer_for_tests(ctx, d, s, note, 10.0)));
    };
    give(&mut sim);
    set(&mut sim, s, 5_000.0);
    let review =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.notes_review_for_tests(ctx, s));
    let held = |sim: &Sim| sim.people().notes.held(s, issue);
    review(&mut sim);
    assert!((held(&sim) - 10.0).abs() < 1e-9, "kept");
    // Seeing a tenth of the pieces out presented at the door this week, it presents them all.
    let count = sim.people().notes.issue(issue).expect("issue").count;
    {
        let c = sim
            .people_mut_for_tests()
            .notes
            .issue_mut(issue)
            .expect("issue");
        c.seen = 0.2 * count;
        c.seen_day = today;
    }
    review(&mut sim);
    assert!(held(&sim) < 1e-9, "presented on seeing others");
    // Hearing that a debtor of the keeper's loans in them defaulted, likewise.
    give(&mut sim);
    {
        let c = sim
            .people_mut_for_tests()
            .notes
            .issue_mut(issue)
            .expect("issue");
        c.seen = 0.0;
        c.seen_day = -1;
        c.defaulted_day = today - 1;
    }
    review(&mut sim);
    assert!(held(&sim) < 1e-9, "presented on hearing of a default");
    // With 5 kg behind its notes, the keeper pays 5 pieces, refuses the rest and suspends.
    give(&mut sim);
    sim.people_mut_for_tests().kept[0].stores[provisions] = 5.0;
    review(&mut sim);
    let c = sim.people().notes.issue(issue).expect("issue").clone();
    assert!((held(&sim) - 5.0).abs() < 1e-6, "{}", held(&sim));
    assert_eq!(c.suspended_until, today + 30);
    // Suspended, it refuses every piece while what is behind them does not cover all out.
    sim.people_mut_for_tests().kept[0].stores[provisions] = 4.0;
    let refused = c.refused;
    review(&mut sim);
    let c = sim.people().notes.issue(issue).expect("issue").clone();
    assert!((held(&sim) - 5.0).abs() < 1e-6, "refused while suspended");
    assert!((c.refused - refused - 5.0).abs() < 1e-6);
    assert!(
        (sim.people().kept[0].stores[provisions] - 4.0).abs() < 1e-9,
        "nothing paid"
    );
    // Covered again, it pays and lifts the suspension.
    sim.people_mut_for_tests().kept[0].stores[provisions] = c.count + 1.0;
    review(&mut sim);
    assert!(held(&sim) < 1e-9, "paid once covered");
    assert_eq!(
        sim.people()
            .notes
            .issue(issue)
            .expect("issue")
            .suspended_until,
        -1
    );
}

#[test]
fn once_its_keeper_lends_in_them_a_note_is_worth_what_is_seen_behind_it() {
    let (mut sim, d, k) = scene();
    let provisions = good(&sim, "core:good/provisions");
    let note = good(&sim, "core:good/note");
    let writer = sim.people().household(k).expect("keeper").members[0];
    writers(&mut sim, k, Some(writer));
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == k {
            x.stores[provisions] = 0.0;
        }
    }
    let s = hungry_neighbour(&mut sim, d, k);
    let worth = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.pay_worth_for_tests(ctx, s, note, 0.0))
            .unwrap_or(0.0)
    };
    let p = sim
        .with_ctx_for_tests(|pop, ctx| pop.pay_worth_for_tests(ctx, s, provisions, 0.0))
        .expect("provisions are worth something");
    let before = worth(&mut sim);
    let walk = p - before;
    // It lends in its notes as far as its coverage lets it, so that share of what they pay stays
    // behind them, and a piece is worth that share of the provisions, less the same walk.
    let issue = sim.people().notes.issues[0].clone();
    let behind = sim.people().kept[0].stores[provisions];
    let coverage = sim.rules().people.market.note_coverage;
    let kg = behind / coverage - issue.count;
    assert!(sim.with_ctx_for_tests(|pop, ctx| pop.lend_notes_for_tests(ctx, (k, s), issue.id, kg)));
    let after = worth(&mut sim);
    assert!(
        (after - (p * coverage - walk)).abs() < 1e-6,
        "{after} against {}",
        p * coverage - walk
    );
}
