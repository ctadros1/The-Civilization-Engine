//! The refuse round (M11f, third part; ADR-0038 §4): a household keeping a cart posts a round to a
//! tip a polity keeps by law, calling at the homes that would join at its fee; on a run it empties
//! their heaps into the cart up to its payload and tips the load whole, every kilogram and every
//! disease load traced from heap to tip, each home paying its fee in food on the `Refuse` channel
//! with goods conserved; homes beyond the load wait for the next run. With no tip there is no
//! round, and a run whose tip is gone does not load. A home joins only where a law asks it to carry
//! its heap and the fee costs it less; under no law it does not. A polity may run a round by law,
//! paid from its store as common work. A world with a round saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::contagion::{How, Node};
use civ_agents::ledger::Channel;
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided, PolicyKind, Tip};
use civ_agents::population;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim, persist};
use civ_world::WATER_LAND;

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
    Sim::create_for_tests(
        &NewWorld {
            name: "Rounds".to_owned(),
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
    .expect("generates")
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "rounds").expect("saves");
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

/// The index of cholera in the catalog.
fn cholera(sim: &Sim) -> u16 {
    let i = sim
        .rules()
        .catalog
        .disease_index(CHOLERA)
        .expect("cholera is content");
    u16::try_from(i).expect("few diseases")
}

/// A law of template `policy` in force in the first polity, every adult of its settlement
/// knowing it, as the tips target's: its id.
fn a_law_of(sim: &mut Sim, policy: usize) -> PermanentId {
    let kind = sim.rules().catalog.policies[policy].kind;
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

/// A law of the first template of `kind` in force in the first polity.
fn a_law(sim: &mut Sim, kind: PolicyKind) -> PermanentId {
    let policy = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.kind == kind)
        .expect("the core content has the template");
    a_law_of(sim, policy)
}

/// The law carrying refuse to a tip passed in the first polity, its tip opened: the law and the
/// tip.
fn the_tip_law_passed(sim: &mut Sim) -> (PermanentId, Tip) {
    let law = a_law(sim, PolicyKind::RefuseToTip);
    sim.with_ctx_for_tests(|pop, ctx| pop.open_tip_for_tests(ctx, 0, law));
    let tip = sim.people().polities[0]
        .tips
        .iter()
        .find(|t| t.law == law)
        .cloned()
        .expect("a place qualifies in this valley, or this test shows nothing");
    (law, tip)
}

/// Households of the first polity's settlement with people, by id.
fn our_households(sim: &Sim) -> Vec<PermanentId> {
    let settlement = sim.people().polities[0].settlement;
    let mut out: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|(_, x)| x.id)
        .collect();
    out.sort_unstable();
    out
}

/// A handcart made and kept by household `owner`: its id.
fn a_handcart(sim: &mut Sim, owner: PermanentId) -> PermanentId {
    let vehicle = sim
        .rules()
        .catalog
        .vehicle_index("core:vehicle/handcart")
        .expect("a handcart");
    let (id, today) = (sim.allocate_id_for_tests(), sim.now().day_index());
    let at = sim
        .people()
        .household(owner)
        .map(|x| population::cell_of(sim.map(), x.home) as u32)
        .expect("the keeper");
    let land = sim.land_mut_for_tests();
    land.carts.list.push(civ_land::boats::Boat {
        id,
        vehicle: vehicle as u16,
        owner,
        at,
        made: today,
        wear: 0.0,
        labour_h: 0.0,
        work_h: 0.0,
        saves_h_year: 0.0,
        crew: (0, 0),
    });
    land.carts.changed();
    id
}

/// A cart way over every dry cell about the settlement's homes and `also`, a few cells beyond:
/// ground a cart can go on between any of them.
fn cart_ground(sim: &mut Sim, also: (f32, f32)) {
    use civ_land::ways::{CART, Keeper, Rank, Segment, Surface, WALK, Way};
    let map = sim.map().clone();
    let size = map.cell_size_m;
    let places: Vec<(f32, f32)> = our_households(sim)
        .iter()
        .filter_map(|&h| sim.people().household(h).map(|x| x.home))
        .chain([also])
        .collect();
    let (x0, x1, y0, y1) = places.iter().fold(
        (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
        |(a, b, c, d), &(x, y)| (a.min(x), b.max(x), c.min(y), d.max(y)),
    );
    let cell = |v: f32, n: u32| ((v / size).floor() as i64).clamp(0, i64::from(n) - 1);
    let pad = 3.0 * size;
    let (cx0, cx1) = (cell(x0 - pad, map.width), cell(x1 + pad, map.width));
    let (cy0, cy1) = (cell(y0 - pad, map.height), cell(y1 + pad, map.height));
    let width = i64::from(map.width);
    let cells: Vec<u32> = (cy0..=cy1)
        .flat_map(|y| (cx0..=cx1).map(move |x| (y * width + x) as u32))
        .filter(|&c| map.water[c as usize] == WATER_LAND)
        .collect();
    let (way, today) = (sim.allocate_id_for_tests(), sim.now().day_index());
    let land = sim.land_mut_for_tests();
    land.ways.add(Way {
        id: way,
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
        modes: WALK | CART,
        trips: 0.0,
        trips_day: today,
        rank: Rank::Lane,
        track: None,
    });
    land.ways.changed();
}

/// Household `h`'s heap now: its decaying share and its ash, kilograms.
fn heap(sim: &Sim, h: PermanentId) -> (f64, f64) {
    let params = &sim.rules().people.midden;
    sim.people().household(h).map_or((0.0, 0.0), |x| {
        (
            x.midden.at_time(sim.now(), x.members.len(), params),
            x.midden.ash_kg,
        )
    })
}

/// Sets household `h`'s heap to `kg` of decaying share and `ash_kg` of ash, as of now.
fn set_heap(sim: &mut Sim, h: PermanentId, kg: f64, ash_kg: f64) {
    let now = sim.now();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == h {
            x.midden.kg = kg;
            x.midden.ash_kg = ash_kg;
            x.midden.at = now;
        }
    }
}

/// Food moved by `channel` since the counters began, kcal.
fn food_kcal(sim: &Sim, channel: Channel) -> f64 {
    let goods = &sim.rules().catalog.goods;
    (0..goods.len())
        .map(|g| sim.people().transfers.get(channel, g) * goods[g].kcal_per_kg)
        .sum()
}

/// The first polity's refuse round kept by `operator`, if any.
fn round_of(sim: &Sim, operator: PermanentId) -> Option<civ_agents::polity::RefuseRound> {
    sim.people().polities[0]
        .rounds
        .iter()
        .find(|r| r.operator == operator)
        .cloned()
}

/// A world with the tip law passed, its first household keeping a handcart and a way for it about
/// the homes and the tip: the world, the keeper and the tip.
fn a_village_with_a_cart() -> (Sim, PermanentId, Tip) {
    let mut sim = world_with(5);
    sim.advance_minutes(2 * DAY).expect("advances");
    let (_, tip) = the_tip_law_passed(&mut sim);
    let keeper = our_households(&sim)[0];
    a_handcart(&mut sim, keeper);
    cart_ground(&mut sim, tip.place);
    (sim, keeper, tip)
}

#[test]
fn a_posted_round_empties_heaps_into_its_cart_and_tips_them_whole_for_a_fee() {
    let (mut sim, keeper, tip) = a_village_with_a_cart();
    let review = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_refuse_round_for_tests(ctx, keeper))
    };
    review(&mut sim);
    let round = round_of(&sim, keeper).expect("homes that carry under the law join a round");
    eprintln!(
        "{} homes join at {:.1} kcal a call, every {} days",
        round.homes.len(),
        round.fee_kcal,
        round.every_days
    );
    assert_eq!((round.tip, round.law, round.next), (tip.id, None, 0));
    assert_eq!(round.every_days, population::ROUND_EVERY_DAYS);
    assert!(round.fee_kcal > 0.0 && !round.homes.contains(&keeper));
    assert!(round.homes.len() >= 3, "{:?}", round.homes);
    let payload = sim.rules().catalog.vehicles[sim
        .rules()
        .catalog
        .vehicle_index("core:vehicle/handcart")
        .expect("a handcart")]
    .payload_kg;

    // Heaps of 60 kg and 2 kg of ash each: a handcart of 150 kg takes the first two whole and part
    // of the third; the rest wait.
    for &h in &round.homes {
        set_heap(&mut sim, h, 60.0, 2.0);
    }
    let d = cholera(&sim);
    let (first, third) = (round.homes[0], round.homes[2]);
    {
        let pop = sim.people_mut_for_tests();
        pop.contagion.add(Node::Midden(first), d, 500.0);
        pop.contagion.add(Node::Midden(third), d, 200.0);
    }
    // Their stores brought up to now, as a run does, so their heaps hold the ash of what they
    // burned.
    for &h in &round.homes {
        sim.with_ctx_for_tests(|pop, ctx| {
            pop.buildings_changed(ctx.now, ctx.land, ctx.catalog, ctx.params, h)
        });
    }
    let heaps: Vec<(PermanentId, (f64, f64))> =
        round.homes.iter().map(|&h| (h, heap(&sim, h))).collect();
    let tip_before = sim.people().polities[0].tips[0].clone();
    let held_before = sim.people().goods_held();
    let flows_before = sim.people().flows();
    let paid_before = food_kcal(&sim, Channel::Refuse);
    let ran = sim.with_ctx_for_tests(|pop, ctx| pop.refuse_run_for_tests(ctx, round.id));
    assert!(ran, "the round ran");
    let after = round_of(&sim, keeper).expect("still posted");
    let run = after.runs.last().expect("its run recorded").clone();
    assert_eq!(after.last_run, sim.now().day_index());
    eprintln!(
        "the run called at {} homes in {:.2} h: {:?}",
        run.calls.len(),
        run.hours,
        run.calls
    );
    assert_eq!(run.calls.len(), 3, "two whole heaps and part of a third");
    assert_eq!(
        run.calls.iter().map(|c| c.household).collect::<Vec<_>>(),
        round.homes[..3].to_vec(),
        "it calls in turn"
    );
    let loaded: f64 = run.calls.iter().map(|c| c.kg + c.ash_kg).sum();
    assert!((loaded - payload).abs() < 1e-6, "{loaded} kg of {payload}");
    assert_eq!(
        after.next, 2,
        "the part-emptied home is called at first next time"
    );
    // Every kilogram from the heaps lies at the tip.
    for c in &run.calls {
        let before = heaps.iter().find(|x| x.0 == c.household).expect("on it").1;
        let now = heap(&sim, c.household);
        assert!((before.0 - now.0 - c.kg).abs() < 1e-9, "{c:?}");
        assert!((before.1 - now.1 - c.ash_kg).abs() < 1e-9, "{c:?}");
    }
    let t = &sim.people().polities[0].tips[0];
    let kg: f64 = run.calls.iter().map(|c| c.kg).sum();
    let ash: f64 = run.calls.iter().map(|c| c.ash_kg).sum();
    assert!((t.kg - tip_before.kg - kg).abs() < 1e-9);
    assert!((t.ash_kg - tip_before.ash_kg - ash).abs() < 1e-9);
    for c in &run.calls {
        assert!(t.households.binary_search(&c.household).is_ok());
    }
    // Homes the load could not take keep their heaps.
    for &h in &round.homes[3..] {
        let before = heaps.iter().find(|x| x.0 == h).expect("on it").1;
        assert_eq!(heap(&sim, h), before, "{h} waits for the next run");
    }
    // The disease loads moved with the heaps, whole and part, recorded as carried.
    let c = &sim.people().contagion;
    assert_eq!(c.load(Node::Midden(first), d), 0.0);
    let share = run.calls[2].kg / heaps[2].1.0;
    assert!(share > 0.0 && share < 1.0);
    assert!((c.load(Node::Midden(third), d) - 200.0 * (1.0 - share)).abs() < 1e-6);
    assert!(
        (c.load(Node::Midden(third), d) + c.load(Node::Tip(t.id), d) - 700.0).abs() < 1e-6,
        "the load is conserved"
    );
    assert!(c.moves.iter().any(|m| m.how == How::Carried
        && m.from == Node::Midden(first)
        && m.to == Node::Tip(t.id)
        && m.amount == 500.0));
    // Each home called at paid its fee in food on the `Refuse` channel, goods conserved.
    let paid = food_kcal(&sim, Channel::Refuse) - paid_before;
    let fees = f64::from(round.fee_kcal) * run.calls.len() as f64;
    assert!(
        (paid - fees).abs() < 1e-3 * fees.max(1.0),
        "{paid} paid of {fees}"
    );
    let gaps = population::unaccounted(
        sim.rules().catalog.goods.len(),
        (&held_before, &flows_before),
        (&sim.people().goods_held(), &sim.people().flows()),
    );
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    for c in &run.calls {
        assert_eq!(
            sim.people().household(c.household).map(|x| x.midden.tipped),
            Some(sim.now().day_index()),
            "the law counts its heap carried"
        );
    }
    // The polity's page and the keeper's words name the round.
    let polity = sim.people().polities[0].clone();
    let words = civ_sim::frames::government::services_words(&sim, &polity);
    eprintln!("{words:?}");
    assert!(
        words.iter().any(|w| w.contains("'s refuse round calls at ")
            && w.contains(" every 7 days and carries to the tip ")
            && w.contains(" carried this year")),
        "{words:?}"
    );
    // Lived on, it runs again on its day by itself, starting where it stopped.
    sim.advance_minutes(8 * DAY).expect("advances");
    let lived = round_of(&sim, keeper).expect("still posted");
    eprintln!("{:?}", lived.runs.iter().map(|r| r.day).collect::<Vec<_>>());
    assert!(lived.runs.len() >= 2, "it ran again on its day");
    assert_eq!(
        lived.runs[1].calls.first().map(|c| c.household),
        Some(round.homes[2]),
        "the home part-emptied is called at first"
    );
    // And a world with a round saves and loads exactly.
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn no_tip_no_round_and_a_run_whose_tip_is_gone_does_not_load() {
    let mut sim = world_with(5);
    sim.advance_minutes(2 * DAY).expect("advances");
    let keeper = our_households(&sim)[0];
    a_handcart(&mut sim, keeper);
    let review = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_refuse_round_for_tests(ctx, keeper))
    };
    // No tip: nothing posted, and a day lived posts nothing either.
    review(&mut sim);
    assert!(round_of(&sim, keeper).is_none());
    // A tip opened by law: a round to it.
    let (law, tip) = the_tip_law_passed(&mut sim);
    cart_ground(&mut sim, tip.place);
    review(&mut sim);
    let round = round_of(&sim, keeper).expect("a round once a tip is kept");
    for &h in &round.homes {
        set_heap(&mut sim, h, 30.0, 1.0);
    }
    let heaps =
        |sim: &Sim| -> Vec<(f64, f64)> { round.homes.iter().map(|&h| heap(sim, h)).collect() };
    let run =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.refuse_run_for_tests(ctx, round.id));
    let set_status = |sim: &mut Sim, status: LawStatus| {
        for l in &mut sim.people_mut_for_tests().polities[0].laws {
            if l.id == law {
                l.status = status;
            }
        }
    };
    // The law that keeps the tip lapses: the run does not load.
    let before = heaps(&sim);
    set_status(&mut sim, LawStatus::Superseded);
    assert!(!run(&mut sim));
    assert_eq!(heaps(&sim), before);
    assert!(round_of(&sim, keeper).expect("posted").runs.is_empty());
    // In force again but the tip gone: still nothing loads.
    set_status(&mut sim, LawStatus::InForce);
    let tips = std::mem::take(&mut sim.people_mut_for_tests().polities[0].tips);
    assert!(!run(&mut sim));
    assert_eq!(heaps(&sim), before);
    // Weighed again with its tip gone, the round is withdrawn.
    review(&mut sim);
    assert!(round_of(&sim, keeper).is_none());
    // The tip back: it runs and loads.
    sim.people_mut_for_tests().polities[0].tips = tips;
    review(&mut sim);
    let round = round_of(&sim, keeper).expect("posted again");
    assert!(sim.with_ctx_for_tests(|pop, ctx| pop.refuse_run_for_tests(ctx, round.id)));
}

#[test]
fn a_home_joins_where_a_law_asks_more_carrying_than_the_fee_and_not_otherwise() {
    let mut sim = world_with(5);
    sim.advance_minutes(2 * DAY).expect("advances");
    let h = our_households(&sim)[1];
    let join = |sim: &mut Sim, fee: f64| {
        sim.with_ctx_for_tests(|pop, ctx| pop.would_join_round_for_tests(ctx, h, fee, 7))
    };
    let asked = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.carrying_asked_h_year_for_tests(ctx, h))
    };
    // Under no law it carries nothing and gains nothing: it joins at no fee at all.
    assert_eq!(asked(&mut sim), 0.0);
    assert!(!join(&mut sim, 0.0));
    assert!(!join(&mut sim, 1.0));
    // Under the tip law it is asked an hour every 30 days: it joins where the fees a year cost
    // it less than those hours at what its hour brings in food, and not where they cost more.
    the_tip_law_passed(&mut sim);
    let def = sim
        .rules()
        .catalog
        .policies
        .iter()
        .find_map(|d| d.tip)
        .expect("tip terms");
    let want = 365.0 / f64::from(def.every_days) * f64::from(def.carry_h);
    assert!((asked(&mut sim) - want).abs() < 1e-9);
    assert!(join(&mut sim, 0.0));
    assert!(join(&mut sim, 1.0));
    assert!(!join(&mut sim, 1e9));
    // The fee where it stops joining is its hours asked a year over the calls a year.
    let (mut lo, mut hi) = (1.0, 1e9);
    for _ in 0..80 {
        let mid = (lo + hi) / 2.0;
        if join(&mut sim, mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let hour_kcal = lo * (365.0 / 7.0) / want;
    eprintln!("joins below {lo:.1} kcal a call: its hour brings {hour_kcal:.0} kcal of food");
    assert!(hour_kcal > 0.0);
}

#[test]
fn a_polity_runs_a_round_by_law_paid_from_its_store() {
    let (mut sim, keeper, tip) = a_village_with_a_cart();
    let law = a_law(&mut sim, PolicyKind::CollectRefuse);
    sim.with_ctx_for_tests(|pop, ctx| pop.law_rounds_for_tests(ctx));
    let round = round_of(&sim, keeper).expect("the law's round, worked by the cart's keeper");
    let mut others: Vec<PermanentId> = our_households(&sim)
        .into_iter()
        .filter(|&h| h != keeper)
        .collect();
    others.sort_unstable();
    let mut homes = round.homes.clone();
    homes.sort_unstable();
    assert_eq!(
        (round.law, round.tip, round.fee_kcal),
        (Some(law), tip.id, 0.0)
    );
    assert_eq!(homes, others, "it calls at every other home");
    for &h in &round.homes {
        set_heap(&mut sim, h, 20.0, 1.0);
    }
    let run =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.refuse_run_for_tests(ctx, round.id));
    // Nothing to pay its hours with: it does not go.
    let before: Vec<(f64, f64)> = round.homes.iter().map(|&h| heap(&sim, h)).collect();
    assert!(!run(&mut sim));
    assert_eq!(
        round
            .homes
            .iter()
            .map(|&h| heap(&sim, h))
            .collect::<Vec<_>>(),
        before
    );
    // A law paying common work and coin in the store: it goes, the keeper paid from the store,
    // and no home pays a fee.
    let pay_work = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.id == "core:policy/pay_work")
        .expect("a pay template");
    a_law_of(&mut sim, pay_work);
    let coin = sim
        .rules()
        .catalog
        .good_index("core:good/coin")
        .expect("coin");
    let (polity, today) = (sim.people().polities[0].id, sim.now().day_index());
    assert!(
        sim.people_mut_for_tests()
            .strike_for_tests(coin, polity, polity, today, 100.0, 8.0, 1.0)
    );
    let coin_of = |sim: &Sim| {
        sim.people()
            .household(keeper)
            .map_or(0.0, |x| x.stores.get(coin).copied().unwrap_or(0.0))
    };
    let (store, held) = (sim.people().polities[0].stores[coin], coin_of(&sim));
    let fees = food_kcal(&sim, Channel::Refuse);
    assert!(run(&mut sim), "the round ran");
    let after = round_of(&sim, keeper).expect("still kept");
    let r = after.runs.last().expect("its run");
    assert!(!r.calls.is_empty() && r.hours > 0.0);
    let wage = coin_of(&sim) - held;
    eprintln!(
        "{} homes called at in {:.2} h; the keeper was paid {wage:.1} coin",
        r.calls.len(),
        r.hours
    );
    assert!(wage > 0.0, "paid from the store");
    assert!((store - sim.people().polities[0].stores[coin] - wage).abs() < 1e-9);
    assert_eq!(food_kcal(&sim, Channel::Refuse), fees, "no home paid a fee");
    let t = &sim.people().polities[0].tips[0];
    let loaded: f64 = r.calls.iter().map(|c| c.kg + c.ash_kg).sum();
    assert!((t.kg + t.ash_kg - tip.kg - tip.ash_kg - loaded).abs() < 1e-9);
    let polity = sim.people().polities[0].clone();
    let words = civ_sim::frames::government::services_words(&sim, &polity);
    eprintln!("{words:?}");
    assert!(
        words
            .iter()
            .any(|w| w.starts_with("The polity's refuse round, worked by ")
                && w.ends_with("paid from the store")),
        "{words:?}"
    );
    // The law ended, its round goes with it.
    for l in &mut sim.people_mut_for_tests().polities[0].laws {
        if l.id == law {
            l.status = LawStatus::Superseded;
        }
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.law_rounds_for_tests(ctx));
    assert!(round_of(&sim, keeper).is_none());
}
