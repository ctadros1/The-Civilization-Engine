//! The telegraph (M11c; ADR-0036 §8): offices, the wire between them, and what is sent.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim, persist};

fn content() -> &'static ContentRegistry {
    static C: OnceLock<ContentRegistry> = OnceLock::new();
    C.get_or_init(|| {
        civ_content::load(
            &civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("root"),
        )
        .registry
        .expect("content")
    })
}

/// Two settlements founded apart.
fn world() -> Sim {
    let sim = Sim::create_for_tests(
        &NewWorld {
            name: "Telegraph".to_owned(),
            seed: 3,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: 0,
            neighbours: vec![0],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates");
    assert_eq!(sim.land().settlements.len(), 2);
    sim
}

fn good(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect("a good")
}

fn teach(sim: &mut Sim, household: PermanentId, id: &str) {
    let t = sim
        .rules()
        .catalog
        .techniques
        .iter()
        .position(|t| t.id == id)
        .expect("a technique");
    let learn_h = sim.rules().catalog.techniques[t].learn_h;
    let members = sim
        .people()
        .household(household)
        .expect("household")
        .members
        .clone();
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if members.contains(&p.id) {
            p.learn(t, learn_h, members[0], learn_h, now);
        }
    }
}

fn set(sim: &mut Sim, household: PermanentId, g: usize, kg: f64) {
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == household {
            x.stores[g] = kg;
        }
    }
}

fn straight(width: u32, a: u32, b: u32) -> Vec<u32> {
    let w = i64::from(width);
    let (mut x, mut y) = (i64::from(a) % w, i64::from(a) / w);
    let (bx, by) = (i64::from(b) % w, i64::from(b) / w);
    let mut cells = vec![a];
    while (x, y) != (bx, by) {
        x += (bx - x).signum();
        y += (by - y).signum();
        cells.push((y * w + x) as u32);
    }
    cells
}

/// A household of each settlement and their settlements and home cells.
fn pair(sim: &Sim) -> [(PermanentId, PermanentId, u32); 2] {
    let map = sim.map().clone();
    let first = |s: PermanentId| {
        sim.people()
            .households
            .iter()
            .map(|(_, h)| h)
            .filter(|h| h.settlement == Some(s) && !h.members.is_empty())
            .map(|h| {
                (
                    h.id,
                    s,
                    civ_agents::population::cell_of(&map, h.home) as u32,
                )
            })
            .min_by_key(|h| h.0)
            .expect("a household")
    };
    let s = &sim.land().settlements;
    [first(s[0].id), first(s[1].id)]
}

/// Makes `household` an office: an instrument, and its people write and know the code.
fn office(sim: &mut Sim, household: PermanentId) {
    let instrument = good(sim, "core:good/telegraph_instrument");
    set(sim, household, instrument, 1.0);
    teach(sim, household, "core:technique/writing");
    teach(sim, household, "core:technique/telegraphy");
}

#[test]
fn an_office_strings_a_wire_to_another_where_its_errands_there_would_repay_it() {
    use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
    let mut sim = world();
    let [(a, sa, from), (b, sb, to)] = pair(&sim);
    let is_office =
        |sim: &mut Sim, h| sim.with_ctx_for_tests(|pop, ctx| pop.telegraph_office(ctx, h));
    assert_eq!(is_office(&mut sim, a), None, "nobody knows the telegraph");
    let instrument = good(&sim, "core:good/telegraph_instrument");
    set(&mut sim, a, instrument, 1.0);
    teach(&mut sim, a, "core:technique/telegraphy");
    assert_eq!(is_office(&mut sim, a), None, "nobody of it can write");
    teach(&mut sim, a, "core:technique/writing");
    let (operator, rate) = is_office(&mut sim, a).expect("an office");
    assert_eq!(rate, 6.0);
    assert!(
        sim.people()
            .household(a)
            .expect("a")
            .members
            .contains(&operator)
    );
    office(&mut sim, b);
    let reckon = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.telegraph_reckonings_for_tests(ctx, a))
    };
    // Nobody of its settlement has run an errand to the other's market: nothing to repay it.
    assert!(reckon(&mut sim).is_empty());
    let year = sim
        .now()
        .day_index()
        .div_euclid(civ_core::time::DAYS_PER_YEAR);
    sim.people_mut_for_tests()
        .contacts
        .years
        .entry((year, sa, sb))
        .or_default()
        .bought = 5_000;
    // No way between the offices: no route.
    assert!(reckon(&mut sim).is_empty());
    let cells = straight(sim.map().width, from, to);
    let way = sim.allocate_id_for_tests();
    let today = sim.now().day_index();
    sim.land_mut_for_tests().ways.add(Way {
        id: way,
        begun: today,
        segments: vec![Segment {
            cells: cells[1..cells.len() - 1].to_vec(),
            corridor_m: 4.0,
            built_m: 4.0,
            usable_m: 4.0,
            surface: Surface::Cleared,
            keeper: Keeper::Nobody,
            kept: today,
        }],
        modes: WALK,
        trips: 0.0,
        trips_day: today,
        rank: Rank::Lane,
        track: None,
    });
    let params = sim.rules().land.wire.expect("the core lays wire");
    set(&mut sim, a, params.good, 1e6);
    let r = reckon(&mut sim);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].to, b);
    assert_eq!(
        (r[0].cells.first(), r[0].cells.last()),
        (Some(&from), Some(&to))
    );
    assert!(r[0].worth_h_year > 0.0);
    assert!(
        (r[0].cost_h - r[0].length_m * params.lay_h_m).abs() < 1e-6,
        "copper in hand costs nothing more: {}",
        r[0].cost_h
    );
    let worth = r[0].worth_h_year;
    assert!(worth * params.life_years > r[0].cost_h, "it repays");
    // Half the errands, half the worth: a fee each.
    sim.people_mut_for_tests()
        .contacts
        .years
        .get_mut(&(year, sa, sb))
        .expect("contacts")
        .bought = 2_500;
    assert!((reckon(&mut sim)[0].worth_h_year * 2.0 - worth).abs() < 1e-6 * worth.max(1.0));
    // It begins it, marked out with what it reckoned, and strings it a metre each lay_h_m hours
    // of work from its copper; only whole does it carry.
    sim.with_ctx_for_tests(|pop, ctx| pop.review_telegraph_for_tests(ctx, a));
    let t = sim.land().wires.telegraphs[0].clone();
    assert_eq!((t.a, t.b, t.laid_m), (a, b, 0.0));
    assert!(t.worth_h > 0.0 && !t.whole());
    // Already joined to it: nothing more to weigh there.
    assert!(reckon(&mut sim).is_empty());
    sim.with_ctx_for_tests(|pop, ctx| pop.telegraph_work_for_tests(ctx, a, t.id, 10.0));
    let laid = sim.land().wires.telegraphs[0].laid_m;
    assert!((laid - 10.0 / params.lay_h_m).abs() < 1e-9, "{laid}");
    // The far office strings none of it.
    sim.with_ctx_for_tests(|pop, ctx| pop.telegraph_work_for_tests(ctx, b, t.id, 10.0));
    assert_eq!(sim.land().wires.telegraphs[0].laid_m, laid);
    sim.with_ctx_for_tests(|pop, ctx| pop.telegraph_work_for_tests(ctx, a, t.id, 1e9));
    assert!(sim.land().wires.telegraphs[0].whole());
    // Saved and loaded exactly.
    let before = sim.land().wires.clone();
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "telegraph",
    )
    .expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().wires, before);
    let (x, y) = (
        persist::encode_sections(&sim),
        persist::encode_sections(&loaded),
    );
    for (p, q) in x.iter().zip(&y) {
        assert!(p.bytes == q.bytes, "section `{}` differs", p.tag);
    }
}

/// The second household of settlement `s` (with people), other than `not`.
fn another(sim: &Sim, s: PermanentId, not: PermanentId) -> PermanentId {
    sim.people()
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| h.settlement == Some(s) && !h.members.is_empty() && h.id != not)
        .map(|h| h.id)
        .min()
        .expect("another household")
}

#[test]
fn a_household_asks_by_wire_what_is_offered_before_it_walks_and_acts_on_the_answer() {
    use civ_agents::ledger::Channel;
    use civ_agents::market::Offer;
    use civ_agents::reports::{Errand, ReportHow, TelegramStage};
    let mut sim = world();
    let [(a, sa, _), (b, sb, _)] = pair(&sim);
    office(&mut sim, a);
    office(&mut sim, b);
    let (h, y) = (another(&sim, sa, a), another(&sim, sb, b));
    let food = sim
        .rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.purpose == civ_agents::params::GoodUse::Food && g.kcal_per_kg > 0.0)
        .expect("a food");
    let offer = Offer {
        good: food as u16,
        payment: food as u16,
        price: 1.0,
        units: 40.0,
        ask_h: 0.5,
    };
    let errand = Errand {
        market: sb,
        seller: y,
        firm: false,
        good: food as u16,
        payment: food as u16,
        units: 10.0,
        home_h: 1.0,
        share: 0.5,
        day: sim.now().day_index(),
        by_boat: None,
        by_cart: None,
        by_ride: None,
    };
    let setup = |sim: &mut Sim| {
        for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
            if x.id == y {
                x.offers = vec![offer];
            }
        }
        sim.people_mut_for_tests().reports.errands.insert(h, errand);
    };
    setup(&mut sim);
    let year = sim
        .now()
        .day_index()
        .div_euclid(civ_core::time::DAYS_PER_YEAR);
    sim.people_mut_for_tests()
        .contacts
        .years
        .entry((year, sa, sb))
        .or_default()
        .missed[0] = 50;
    let ask =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.hand_in_telegram_for_tests(ctx, h));
    // No wire between the offices: nothing to ask by.
    assert!(!ask(&mut sim));
    let wire = sim.allocate_id_for_tests();
    sim.land_mut_for_tests()
        .wires
        .telegraphs
        .push(civ_land::wire::Telegraph {
            id: wire,
            a,
            b,
            cells: vec![0, 1],
            length_m: 100.0,
            laid_m: 50.0,
            broken: false,
            worth_h: 1.0,
        });
    assert!(!ask(&mut sim), "not while it is being strung");
    sim.land_mut_for_tests().wires.telegraphs[0].laid_m = 100.0;
    // With trips there going for nothing, it asks, and only once.
    assert!(ask(&mut sim));
    assert!(!ask(&mut sim));
    let waits =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.waits_on_telegram_for_tests(ctx, h));
    assert!(waits(&mut sim), "it waits on the answer before it sets out");
    let t = sim.people().reports.telegrams[&h].clone();
    assert_eq!(
        (t.near, t.far, t.market, t.stage),
        (a, b, sb, TelegramStage::Handed)
    );
    // Handed in, it pays the office a fee in food and waits in its operator's queue: sent within
    // the hours the office is kept.
    let paid = |sim: &Sim| -> f64 {
        let goods = &sim.rules().catalog.goods;
        (0..goods.len())
            .map(|g| sim.people().transfers.get(Channel::Telegraph, g) * goods[g].kcal_per_kg)
            .sum()
    };
    sim.with_ctx_for_tests(|pop, ctx| pop.telegrams_handed_for_tests(ctx));
    let fee = civ_agents::population::TELEGRAM_FEE_FOOD_DAYS
        * sim.rules().people.household.daily_kcal_per_person;
    assert!(
        (paid(&sim) - fee).abs() < 1e-6 * fee,
        "{} of {fee}",
        paid(&sim)
    );
    let t = sim.people().reports.telegrams[&h].clone();
    assert_eq!(t.stage, TelegramStage::Asked);
    let of_day = t.due.rem_euclid(1440);
    assert!(t.due > sim.now().minutes() && (8 * 60..=20 * 60 + 10).contains(&of_day));
    // Sent at its minute, it waits in the far office's queue.
    sim.advance_minutes(t.due - sim.now().minutes() + 1)
        .expect("lives");
    let t2 = sim.people().reports.telegrams[&h].clone();
    assert_eq!(t2.stage, TelegramStage::Answering);
    assert!(t2.due > t.due);
    // Answered at its minute with what the market's sellers offer as they stand, told by the far
    // operator, and walked home; with it the word the operator heard of a fire there.
    setup(&mut sim);
    let far_operator = sim
        .with_ctx_for_tests(|pop, ctx| pop.telegraph_office(ctx, b))
        .expect("an office")
        .0;
    let today = sim.now().day_index();
    let fire = {
        let word = &mut sim.people_mut_for_tests().word;
        let c = word.make(civ_agents::word::Claim {
            id: 0,
            kind: civ_agents::word::ClaimKind::Fire,
            settlement: sb,
            day: today,
            subject: None,
            grievance: None,
            suspected: None,
            fire: None,
            event: None,
        });
        word.hear(far_operator, c, today, None, None);
        c
    };
    sim.advance_minutes(t2.due - sim.now().minutes() + 1)
        .expect("lives");
    let t3 = sim.people().reports.telegrams[&h].clone();
    assert_eq!(t3.stage, TelegramStage::Walking);
    let operator = sim
        .with_ctx_for_tests(|pop, ctx| pop.telegraph_office(ctx, b))
        .expect("an office")
        .0;
    let told = t3
        .answer
        .iter()
        .find(|r| r.seller == y)
        .copied()
        .expect("the seller's offer");
    assert_eq!(
        (told.market, told.good, told.units, told.how, told.from),
        (sb, food as u16, 40.0, ReportHow::Told, Some(operator))
    );
    assert_eq!(told.day, t2.due.div_euclid(1440));
    // Delivered, it holds it as a report, and its errand stands: the seller still offers it.
    setup(&mut sim);
    sim.advance_minutes(t3.due - sim.now().minutes() + 1)
        .expect("lives");
    assert!(!sim.people().reports.telegrams.contains_key(&h));
    assert!(
        sim.people()
            .reports
            .of(h)
            .iter()
            .any(|r| r.market == sb && r.how == ReportHow::Told && r.from == Some(operator))
    );
    assert!(sim.people().reports.errands.contains_key(&h));
    // Each of its people heard of the fire, told by the far operator.
    let members = sim.people().household(h).expect("h").members.clone();
    assert!(!members.is_empty());
    for m in members {
        let heard = sim
            .people()
            .word
            .heard_by(m)
            .iter()
            .find(|x| x.claim == fire)
            .copied()
            .expect("heard of the fire");
        assert_eq!(heard.from, Some(far_operator));
    }
    assert!(!waits(&mut sim));
    // Asked again where the seller has nothing left: the answer lets the errand go.
    sim.people_mut_for_tests().reports.errands.insert(h, errand);
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == y {
            x.offers.clear();
        }
    }
    let now = sim.now().minutes();
    let blank = |stage| civ_agents::reports::Telegram {
        near: a,
        far: b,
        market: sb,
        handed: now,
        due: now,
        stage,
        answer: Vec::new(),
        claims: Vec::new(),
        teller: None,
    };
    sim.people_mut_for_tests()
        .reports
        .telegrams
        .insert(h, blank(TelegramStage::Walking));
    sim.with_ctx_for_tests(|pop, ctx| pop.telegrams_due_for_tests(ctx));
    assert!(!sim.people().reports.telegrams.contains_key(&h));
    assert!(!sim.people().reports.errands.contains_key(&h));
    // A telegram whose wire is down is lost.
    sim.land_mut_for_tests().wires.telegraphs[0].broken = true;
    sim.people_mut_for_tests()
        .reports
        .telegrams
        .insert(h, blank(TelegramStage::Asked));
    sim.with_ctx_for_tests(|pop, ctx| pop.telegrams_due_for_tests(ctx));
    assert!(!sim.people().reports.telegrams.contains_key(&h));
}

#[test]
fn a_telegram_waiting_on_its_answer_saves_and_loads_exactly() {
    use civ_agents::reports::{PriceReport, ReportHow, Telegram, TelegramStage};
    let mut sim = world();
    let [(a, _, _), (b, sb, _)] = pair(&sim);
    let h = another(&sim, sim.land().settlements[0].id, a);
    sim.people_mut_for_tests().reports.telegrams.insert(
        h,
        Telegram {
            near: a,
            far: b,
            market: sb,
            handed: 120,
            due: 610,
            stage: TelegramStage::Walking,
            answer: vec![PriceReport {
                market: sb,
                seller: b,
                firm: false,
                good: 3,
                payment: 4,
                price: 1.5,
                ask_h: 0.25,
                units: 12.0,
                day: 0,
                how: ReportHow::Told,
                from: Some(b),
            }],
            claims: vec![7, 9],
            teller: Some(b),
        },
    );
    let before = sim.people().reports.clone();
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "telegrams",
    )
    .expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().reports, before);
    let (x, y) = (
        persist::encode_sections(&sim),
        persist::encode_sections(&loaded),
    );
    for (p, q) in x.iter().zip(&y) {
        assert!(p.bytes == q.bytes, "section `{}` differs", p.tag);
    }
}

#[test]
fn the_observer_shows_offices_their_wires_and_who_waits_on_an_answer() {
    use civ_agents::reports::{Errand, Telegram, TelegramStage};
    let mut sim = world();
    // Ten days for homes to stand.
    sim.advance_minutes(10 * 24 * 60).expect("lives");
    let [(_, sa, _), (b, sb, _)] = pair(&sim);
    let a = sim
        .land()
        .buildings
        .iter()
        .map(|x| x.household)
        .filter(|&h| sim.people().household(h).and_then(|x| x.settlement) == Some(sa))
        .min()
        .expect("a household with a building");
    office(&mut sim, a);
    office(&mut sim, b);
    let wire = sim.allocate_id_for_tests();
    sim.land_mut_for_tests()
        .wires
        .telegraphs
        .push(civ_land::wire::Telegraph {
            id: wire,
            a,
            b,
            cells: vec![0, 1, 2],
            length_m: 2400.0,
            laid_m: 960.0,
            broken: false,
            worth_h: 1.0,
        });
    let seen = |sim: &Sim| -> Vec<(bool, f32, String)> {
        let bytes = civ_sim::frames::ways::ways_response(sim);
        let response =
            civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
        response
            .body_as_ways()
            .expect("ways")
            .currents()
            .expect("lines")
            .iter()
            .map(|l| {
                (
                    l.telegraph(),
                    l.laid(),
                    l.words().unwrap_or_default().to_owned(),
                )
            })
            .collect()
    };
    let rev = civ_sim::frames::ways::ways_rev(&sim);
    let s = seen(&sim);
    assert_eq!(s.len(), 1);
    assert!(s[0].0 && (s[0].1 - 0.4).abs() < 1e-6, "{s:?}");
    assert!(
        s[0].2.starts_with("Telegraph wire of 2400 m from ") && s[0].2.ends_with(", 40% strung."),
        "{}",
        s[0].2
    );
    sim.land_mut_for_tests().wires.telegraphs[0].laid_m = 2400.0;
    assert_ne!(
        civ_sim::frames::ways::ways_rev(&sim),
        rev,
        "strung, the layer changes"
    );
    // A household of the first settlement waits on an answer about its errand there.
    let h = another(&sim, sa, a);
    let errand = Errand {
        market: sb,
        seller: b,
        firm: false,
        good: 0,
        payment: 0,
        units: 1.0,
        home_h: 1.0,
        share: 0.5,
        day: sim.now().day_index(),
        by_boat: None,
        by_cart: None,
        by_ride: None,
    };
    sim.people_mut_for_tests().reports.errands.insert(h, errand);
    let now = sim.now().minutes();
    sim.people_mut_for_tests().reports.telegrams.insert(
        h,
        Telegram {
            near: a,
            far: b,
            market: sb,
            handed: now,
            due: now + 600,
            stage: TelegramStage::Asked,
            answer: Vec::new(),
            claims: Vec::new(),
            teller: None,
        },
    );
    assert!(seen(&sim)[0].2.ends_with(", strung; 1 telegram on it now."));
    let words = civ_sim::frames::markets::errand_words(&sim, h);
    assert!(
        words.contains("; waiting on an answer by telegraph, asked at "),
        "{words}"
    );
    // The office's home says it is one, who keeps it and what waits in its queue.
    let home = sim
        .land()
        .buildings
        .iter()
        .find(|x| x.household == a)
        .map(|x| x.id)
        .expect("its first building");
    let bytes = civ_sim::frames::buildings::buildings_response(&sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
    let words = response
        .body_as_buildings()
        .expect("buildings")
        .buildings()
        .expect("a list")
        .iter()
        .find(|x| x.id() == home.get())
        .expect("its home")
        .wire()
        .unwrap_or_default()
        .to_owned();
    assert!(
        words.contains("Telegraph office kept by ")
            && words.contains("; 1 wire, 1 telegram in its queue"),
        "{words}"
    );
}

#[test]
fn a_household_that_knows_the_telegraph_and_writes_wants_an_instrument_where_another_settlement_knows_it()
 {
    let mut sim = world();
    let [(a, _, _), (b, _, _)] = pair(&sim);
    let instrument = good(&sim, "core:good/telegraph_instrument");
    let keep =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.keep_for_tests(ctx, a)[instrument]);
    assert_eq!(keep(&mut sim), 0.0, "nobody knows the telegraph");
    teach(&mut sim, a, "core:technique/telegraphy");
    teach(&mut sim, a, "core:technique/writing");
    assert_eq!(keep(&mut sim), 0.0, "nobody elsewhere to wire to");
    teach(&mut sim, b, "core:technique/telegraphy");
    assert!(keep(&mut sim) >= 1.0, "one, to be an office");
}

/// A tool's materials are kept as many steps back as the household knows the way (M11d, from the
/// demo): an office to be short of the instrument's iron keeps the bloom it is forged from, and
/// the bog ore and charcoal the bloom is smelted with, once it knows how to burn charcoal; until
/// then the bloomery is no way it can work and no ore is kept.
#[test]
fn an_office_to_be_short_of_iron_keeps_bog_ore_toward_its_instrument_once_it_knows_the_whole_way() {
    let mut sim = world();
    let [(a, _, _), (b, _, _)] = pair(&sim);
    let [instrument, copper, timber, iron, bloom, ore, charcoal] = [
        "core:good/telegraph_instrument",
        "core:good/copper",
        "core:good/timber",
        "core:good/iron",
        "core:good/iron_bloom",
        "core:good/iron_ore",
        "core:good/charcoal",
    ]
    .map(|id| good(&sim, id));
    for t in [
        "core:technique/telegraphy",
        "core:technique/writing",
        "core:technique/bloomery",
        "core:technique/forging",
    ] {
        teach(&mut sim, a, t);
    }
    teach(&mut sim, b, "core:technique/telegraphy");
    for (g, kg) in [
        (copper, 10.0),
        (timber, 4.0),
        (iron, 0.0),
        (bloom, 0.0),
        (ore, 0.0),
        (charcoal, 0.0),
    ] {
        set(&mut sim, a, g, kg);
    }
    let keep = |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.keep_for_tests(ctx, a));
    let k = keep(&mut sim);
    assert!(k[instrument] >= 1.0, "wanted");
    assert!(k[iron] >= 8.0, "its iron is kept: {}", k[iron]);
    assert_eq!(
        k[ore], 0.0,
        "no charcoal to be had: the bloomery is no way yet"
    );
    teach(&mut sim, a, "core:technique/charcoal_burning");
    let k = keep(&mut sim);
    // Eight kilograms of bar from ten of bloom, and that from eighty of ore and its charcoal.
    assert!(k[bloom] >= 10.0 - 1e-6, "the bloom: {}", k[bloom]);
    assert!(k[ore] >= 80.0 - 1e-6, "the ore: {}", k[ore]);
    assert!(k[charcoal] > 0.0, "the charcoal: {}", k[charcoal]);
}

/// Prospecting (M11d): an office to be short of its instrument's copper, whose settlement knows no
/// body of copper ore, sends someone to look over the kind of ground it lies in, and finds an
/// outcrop where they look; the same household without the want does not go, and in the same days
/// nobody's other walks pass near it.
#[test]
fn an_office_to_be_short_of_copper_looks_for_it_and_finds_an_outcrop_nobody_passed() {
    let found = |want: bool| -> bool {
        let mut sim = world();
        let [(a, sa, _), (b, _, _)] = pair(&sim);
        let ore = good(&sim, "core:good/copper_ore");
        let max_walk = sim
            .rules()
            .catalog
            .activities
            .iter()
            .find(|x| x.id == "core:activity/dig_ore")
            .map(|x| f64::from(x.max_walk_minutes))
            .expect("digging ore");
        // An outcrop where the household would look on one of the coming days, twenty minutes to
        // half an hour out, beyond the ground its daily round crosses: the same place whether or
        // not it wants copper.
        let today = sim.now().day_index();
        let at = (today + 1..today + 30)
            .find_map(|day| {
                sim.with_ctx_for_tests(|pop, ctx| {
                    pop.prospect_points_for_tests(ctx, a, ore, max_walk, day)
                })
                .filter(|p| (20.0..30.0).contains(&p.walk_min))
                .map(|p| p.at)
            })
            .expect("copper ground within half an hour of home");
        let body = sim
            .place_deposit(at, "core:good/copper_ore", 10.0, true)
            .expect("placed");
        if want {
            for t in [
                "core:technique/telegraphy",
                "core:technique/writing",
                "core:technique/copper_smelting",
            ] {
                teach(&mut sim, a, t);
            }
            teach(&mut sim, b, "core:technique/telegraphy");
            for (g, kg) in [("core:good/iron", 8.0), ("core:good/timber", 4.0)] {
                let g = good(&sim, g);
                set(&mut sim, a, g, kg);
            }
        }
        for _ in 0..30 {
            sim.advance_minutes(24 * 60).expect("advances");
            if sim.people().knows_deposit(sa, body) {
                return true;
            }
        }
        false
    };
    assert!(found(true), "it looked and found the outcrop");
    assert!(!found(false), "nobody passed near it without looking");
}

/// A household that wants an instrument and holds what it is made of builds it (M11d, fourth
/// part): until the recipe had a work that made it, an instrument wanted was never built.
#[test]
fn an_office_to_be_holding_its_copper_iron_and_timber_builds_its_instrument() {
    let mut sim = world();
    let [(a, _, _), (b, _, _)] = pair(&sim);
    let [instrument, copper, timber, iron] = [
        "core:good/telegraph_instrument",
        "core:good/copper",
        "core:good/timber",
        "core:good/iron",
    ]
    .map(|id| good(&sim, id));
    teach(&mut sim, a, "core:technique/telegraphy");
    teach(&mut sim, a, "core:technique/writing");
    teach(&mut sim, b, "core:technique/telegraphy");
    for (g, kg) in [(copper, 20.0), (timber, 10.0), (iron, 16.0)] {
        set(&mut sim, a, g, kg);
    }
    let held = |sim: &Sim, g: usize| sim.people().household(a).expect("household").stores[g];
    for _ in 0..10 {
        sim.advance_minutes(24 * 60).expect("advances");
    }
    assert!(
        held(&sim, instrument) > 0.0,
        "building it: {} of an instrument",
        held(&sim, instrument)
    );
    assert!(held(&sim, iron) < 16.0, "its iron went into it");
}

/// A material made toward a tool is worth a session whatever its length, and its effort grows
/// with its hours, so a household works the shortest session (M11d, fourth part): one holding the
/// bloom and charcoal for a whole day at the hearth forges its instrument's bar, where before it
/// never did (the session it weighed was a whole day of heavy work, never worth its effort).
#[test]
fn a_household_with_bloom_and_charcoal_enough_for_a_long_day_forges_its_bar() {
    let mut sim = world();
    let [(a, _, _), (b, _, _)] = pair(&sim);
    let [instrument, timber, iron, bloom, charcoal] = [
        "core:good/telegraph_instrument",
        "core:good/timber",
        "core:good/iron",
        "core:good/iron_bloom",
        "core:good/charcoal",
    ]
    .map(|id| good(&sim, id));
    // Forging alone: the bloom it holds is all the bloom it has.
    for t in [
        "core:technique/telegraphy",
        "core:technique/writing",
        "core:technique/forging",
    ] {
        teach(&mut sim, a, t);
    }
    teach(&mut sim, b, "core:technique/telegraphy");
    for (g, kg) in [
        (timber, 10.0),
        (iron, 0.0),
        (bloom, 10.0),
        (charcoal, 100.0),
    ] {
        set(&mut sim, a, g, kg);
    }
    let k = sim.with_ctx_for_tests(|pop, ctx| pop.keep_for_tests(ctx, a));
    assert!(
        k[instrument] >= 1.0 && k[iron] >= 8.0,
        "it wants the instrument's iron"
    );
    for _ in 0..20 {
        sim.advance_minutes(24 * 60).expect("advances");
    }
    let left = sim.people().household(a).expect("household").stores[bloom];
    assert!(
        left <= 5.0,
        "it forged bar from its bloom: {left:.1} kg of 10 left"
    );
}

/// Wood toward a tool's charcoal (M11d, sixth part): a household that wants an instrument, holds
/// its copper, timber and the ore for its iron, and knows the whole way from wood to bar gathers
/// wood beyond its hearth's for charcoal and burns it in mounds the size its shortfall calls for,
/// so the bloom and bar follow and the iron goes toward the instrument. Before, it burned only the
/// hearth's spare wood, a few kilograms a mound, and in 120 days held at most 15 kg of charcoal and
/// 0.3 kg of bar.
#[test]
fn a_household_short_of_charcoal_for_its_instruments_iron_gathers_wood_and_burns_it() {
    let mut sim = world();
    let [(a, _, _), (b, _, _)] = pair(&sim);
    let [instrument, copper, timber, iron, ore, charcoal] = [
        "core:good/telegraph_instrument",
        "core:good/copper",
        "core:good/timber",
        "core:good/iron",
        "core:good/iron_ore",
        "core:good/charcoal",
    ]
    .map(|id| good(&sim, id));
    for t in [
        "core:technique/telegraphy",
        "core:technique/writing",
        "core:technique/charcoal_burning",
        "core:technique/bloomery",
        "core:technique/forging",
    ] {
        teach(&mut sim, a, t);
    }
    teach(&mut sim, b, "core:technique/telegraphy");
    for (g, kg) in [
        (copper, 10.0),
        (timber, 4.0),
        (iron, 0.0),
        (ore, 80.0),
        (charcoal, 0.0),
    ] {
        set(&mut sim, a, g, kg);
    }
    let (mut most_charcoal, mut most_iron) = (0.0f64, 0.0f64);
    for _ in 0..60 {
        sim.advance_minutes(24 * 60).expect("advances");
        let s = &sim.people().household(a).expect("household").stores;
        most_charcoal = most_charcoal.max(s[charcoal]);
        // Its bar, and the bar built into the instrument so far.
        most_iron = most_iron.max(s[iron] + 8.0 * s[instrument]);
    }
    assert!(
        most_charcoal >= 40.0,
        "it burned charcoal: {most_charcoal:.1} kg at most"
    );
    assert!(
        most_iron >= 4.0,
        "half the instrument's iron: {most_iron:.1} kg at most"
    );
}
