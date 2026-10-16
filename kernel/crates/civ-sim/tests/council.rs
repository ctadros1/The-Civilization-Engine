//! Councils and their elections (M7a slice BO, ADR-0024 §1, §3): a custom taken to a council of
//! seats calls its election at the next midnight; word of it goes round; those of the electorate
//! who hear of it choose whether to stand; a contested field is polled at the hearth and counted
//! by the block vote; those chosen sit, and only they are the body; the term's end calls the next
//! election; the label reads the selection chain; and all of it saves and loads exactly.
//!
//! M7a slice BP (ADR-0024 §3-5): each who stands says where they stand on the questions that
//! matter most to them, and word of it goes round as a claim; a faction backs those of its members
//! who stood, as a slate; and both save and load.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::ChronicleKind;
use civ_agents::polity::{Body, CustomVersion, Membership, PassRule};
use civ_agents::seats::{ElectionStage, ElectionWhy, SeatMethod, Seats};
use civ_content::ContentRegistry;
use civ_sim::persist;
use civ_sim::{NewWorld, Sim};

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

fn world_with(content: &ContentRegistry, seed: u64) -> Sim {
    Sim::create(
        &NewWorld {
            name: "Council".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates")
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, content: &ContentRegistry, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "council").expect("saves");
    let mut loaded = persist::load(&saved.path, content).expect("loads");
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
fn a_council_is_elected_sits_as_the_body_and_is_elected_again_as_its_term_ends() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(DAY).expect("advances");
    // Standing costs nothing here, so about half of those who hear of it stand: the field is
    // contested and polled (a test's setting; the content's own cost is a design prior).
    {
        let rules = sim.rules_mut_for_tests().expect("rules of its own");
        rules.people.elections.sit_cost = 0.0;
        rules.people.elections.w_standing = 0.0;
    }
    let seats = Seats {
        n: 5,
        electorate: Membership::Adults,
        method: SeatMethod::Election,
        term_days: 40,
        by_settlement: false,
    };
    let body = Body {
        members: Membership::Seats,
        quorum_share: 0.5,
        pass: PassRule::MoreForThanAgainst,
        seats: Some(seats),
    };
    let now = sim.now();
    let adult = sim.rules().people.family.independent_age;
    let settlement = sim.people().polities[0].settlement;
    let mut adults: Vec<civ_core::PermanentId> = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) >= adult)
        .filter(|p| {
            sim.people()
                .household(p.household)
                .is_some_and(|x| x.settlement == Some(settlement))
        })
        .map(|p| p.id)
        .collect();
    adults.sort_unstable();
    assert!(
        adults.len() > 10,
        "a village to elect from ({})",
        adults.len()
    );
    // Each adult holds a position on two questions, spread from against to for (set down as if
    // talked over: opinion is weighed elsewhere), so that those who stand have somewhere to say
    // they stand (M7a slice BP).
    let questions: Vec<u16> = sim
        .rules()
        .catalog
        .policies
        .iter()
        .enumerate()
        .filter(|(_, p)| p.question.is_some())
        .map(|(i, _)| u16::try_from(i).expect("few policies"))
        .take(2)
        .collect();
    assert_eq!(questions.len(), 2, "content asks questions");
    {
        let pop = sim.people_mut_for_tests();
        for (k, &a) in adults.iter().enumerate() {
            for (j, &q) in questions.iter().enumerate() {
                pop.opinion.insert(civ_agents::opinion::Position {
                    holder: a,
                    policy: q,
                    x: ((k + j) % 5) as f32 / 4.0,
                    anchor: 0.5,
                    salience: 0.5,
                    since: now.day_index(),
                    heard: 0,
                });
            }
        }
    }
    // A faction of every other adult, so that some of its members stand (set down as if founded:
    // founding is a choice weighed elsewhere).
    let faction = sim.allocate_id_for_tests();
    let faction_members: Vec<civ_core::PermanentId> = adults.iter().step_by(2).copied().collect();
    {
        let pop = sim.people_mut_for_tests();
        let against = civ_agents::word::Blamed::Body(pop.polities[0].id);
        pop.factions.list.push(civ_agents::faction::Faction::found(
            faction,
            settlement,
            against,
            faction_members[0],
            now,
        ));
        for &m in &faction_members {
            pop.factions.join(civ_agents::faction::Member {
                person: m,
                faction,
                since: now.day_index(),
                why: civ_agents::faction::Why::default(),
            });
        }
    }
    // The custom is taken to a council, by one who will tell of the election first (set down as
    // if taken: the taking is a choice weighed elsewhere).
    {
        let polity = &mut sim.people_mut_for_tests().polities[0];
        polity.body = body;
        polity.versions.push(CustomVersion {
            body,
            since: now,
            law: None,
            seized_by: Some(adults[0]),
        });
    }
    sim.advance_minutes(DAY).expect("advances");
    let called = {
        let polity = &sim.people().polities[0];
        let e = polity.elections.last().expect("an election is called");
        assert_eq!(e.why, ElectionWhy::Amendment);
        assert_eq!(e.stage, ElectionStage::Standing);
        assert_eq!(e.seats, seats);
        assert!(polity.seated.is_empty(), "nobody sits before the count");
        e.called
    };
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|x| x.kind == ChronicleKind::ElectionCalled)
    );
    // While nobody sits, the council has no members.
    let fields = sim.land().fields.clone();
    let params = sim.rules().people.clone();
    assert!(
        sim.people()
            .body_members(&fields, 0, sim.now(), &params)
            .is_empty()
    );
    // Mid-election, a save goes on alike.
    sim.advance_minutes(3 * DAY).expect("advances");
    saves_and_goes_on_alike(&mut sim, content(), DAY);
    // Names close, the polling day comes, the count is made.
    let stand_days = i64::from(sim.rules().people.elections.stand_days);
    let mut word_checked = false;
    while sim.now().day_index() <= called + stand_days + 2 {
        sim.advance_minutes(DAY).expect("advances");
        // While the polling is yet to come, word of each who stood, and where they stand, goes
        // round from them (M7a slice BP); it is no longer news once the polling day is past.
        let e = sim.people().polities[0]
            .elections
            .last()
            .expect("the election");
        if e.stage == ElectionStage::Polling && !word_checked {
            word_checked = true;
            let word = &sim.people().word;
            let mut heard_of_someone = 0;
            for c in &e.candidates {
                let claim = word
                    .candidacy(settlement, e.poll_day, c.person)
                    .expect("word of each who stood");
                assert!(word.has_heard(c.person, claim), "they know they stand");
                heard_of_someone += adults
                    .iter()
                    .filter(|&&a| a != c.person && word.has_heard(a, claim))
                    .count();
            }
            assert!(heard_of_someone > 0, "word of who stands goes round");
        }
    }
    assert!(word_checked, "the polling was awaited");
    let polity = &sim.people().polities[0];
    let e = polity.elections.last().expect("the election");
    assert_eq!(e.stage, ElectionStage::Installed, "{:?}", e.stage);
    let standing = e.standing();
    assert!(
        standing.len() > usize::from(seats.n),
        "with standing free, the field is contested ({} stood)",
        standing.len()
    );
    assert!(!e.ballots.is_empty(), "some came to poll");
    for (voter, named) in &e.ballots {
        assert!(
            adults.binary_search(voter).is_ok(),
            "only the electorate polls"
        );
        assert!(named.len() <= usize::from(seats.n));
        assert!(named.iter().all(|c| standing.binary_search(c).is_ok()));
    }
    let counted: u32 = e.totals.iter().map(|t| t.1).sum();
    let named: usize = e.ballots.iter().map(|b| b.1.len()).sum();
    assert_eq!(
        counted as usize, named,
        "the block vote counts every name once"
    );
    assert_eq!(e.winners.len(), usize::from(seats.n));
    // The winners are those named most.
    let votes_of = |c: civ_core::PermanentId| e.totals.iter().find(|t| t.0 == c).map_or(0, |t| t.1);
    let last_in = e.winners.iter().map(|&w| votes_of(w)).min().unwrap_or(0);
    assert!(
        e.totals
            .iter()
            .filter(|t| !e.winners.contains(&t.0))
            .all(|t| t.1 <= last_in)
    );
    let mut sitting: Vec<_> = polity.seated.iter().map(|s| s.holder).collect();
    sitting.sort_unstable();
    let mut winners = e.winners.clone();
    winners.sort_unstable();
    assert_eq!(sitting, winners);
    let until = e.until;
    assert!(polity.seated.iter().all(|s| s.until == until));
    // Each who stood told where they stand on up to three questions they hold a position on.
    let told = usize::from(sim.rules().people.elections.positions_told);
    for c in &e.candidates {
        assert!(c.positions.len() <= told);
        let mut qs: Vec<u16> = c.positions.iter().map(|x| x.0).collect();
        qs.dedup();
        assert_eq!(qs.len(), c.positions.len(), "one position a question");
        assert!(c.positions.iter().all(|x| (0.0..=1.0).contains(&x.1)));
    }
    assert!(
        e.candidates.iter().any(|c| !c.positions.is_empty()),
        "some who stood hold positions to tell"
    );
    // The faction backs those of its members who stood, as many as there are seats.
    for (f, slate) in &e.endorsements {
        assert_eq!(*f, faction, "the only faction here");
        assert!(slate.len() <= usize::from(seats.n));
        assert!(slate.iter().all(|p| standing.binary_search(p).is_ok()));
    }
    assert!(
        !e.endorsements.is_empty(),
        "with half the electorate in it, the faction has members who stood"
    );
    // The council's words name the slate.
    let words = civ_agents::seats::council_words(
        polity.body.seats,
        &polity.seated,
        &polity.elections,
        &|id| sim.people().name_of(id),
        &|_| "its faction".to_owned(),
        &|_| "here".to_owned(),
        &|d| d.to_string(),
    );
    assert!(
        words.iter().any(|w| w.starts_with("its faction backs ")),
        "{words:?}"
    );
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|x| x.kind == ChronicleKind::ElectionHeld)
    );
    // The council is the body now: those who sit, and nobody else.
    let members: Vec<_> = sim
        .people()
        .body_members(&fields, 0, sim.now(), &params)
        .into_iter()
        .map(|m| m.0)
        .collect();
    assert_eq!(members, sitting);
    // The label reads the chain from electorate to council.
    let label = civ_sim::labels::label_of(&sim, &sim.people().polities[0]);
    assert_eq!(label.name, "Republic", "{label:?}");
    // A change of franchise alone is forecast by counting the open ballots again with only the
    // new electorate's (M7a slice BQ, research 09-08 §1.8).
    let elders = Seats {
        electorate: Membership::Elders,
        ..seats
    };
    let (forecast, electorate) = sim
        .with_ctx_for_tests(|pop, ctx| pop.forecast_sitters_for_tests(ctx, 0, &elders, adults[0]));
    let e = sim.people().polities[0]
        .elections
        .last()
        .expect("the election");
    let theirs: Vec<Vec<civ_core::PermanentId>> = e
        .ballots
        .iter()
        .filter(|b| electorate.binary_search(&b.0).is_ok())
        .map(|b| b.1.clone())
        .collect();
    if !theirs.is_empty() {
        let mut named: Vec<_> = theirs.iter().flatten().copied().collect();
        named.sort_unstable();
        named.dedup();
        assert!(forecast.len() <= usize::from(seats.n));
        assert!(
            forecast.iter().all(|f| named.binary_search(f).is_ok()),
            "only those the elders named"
        );
    }
    saves_and_goes_on_alike(&mut sim, content(), DAY);
    // As the term ends, the next election is called, and is done by the time the seats end.
    while sim.now().day_index() < until + 2 {
        sim.advance_minutes(DAY).expect("advances");
    }
    let polity = &sim.people().polities[0];
    let next = polity.elections.last().expect("another election");
    assert_eq!(next.why, ElectionWhy::TermEnding);
    assert!(next.id > 1);
    assert!(
        !next.stage.open(),
        "called before the term ended, it is done by then ({:?})",
        next.stage
    );
}

/// M7a slice BQ (ADR-0024 §1): a polity of two settlements whose council's seats are shared
/// among them: each settlement's share follows the size of its electorate; each who stands
/// stands where they live; each voter names only those of their own district, as many as its
/// seats; each district is counted apart; and it saves and loads alike.
#[test]
fn seats_shared_among_settlements_are_elected_district_by_district() {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Districts".to_owned(),
            seed: 4,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: vec![30],
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [9; 16],
    )
    .expect("generates");
    sim.advance_minutes(DAY).expect("advances");
    {
        let rules = sim.rules_mut_for_tests().expect("rules of its own");
        rules.people.elections.sit_cost = 0.0;
        rules.people.elections.w_standing = 0.0;
    }
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == a)
        .expect("its polity");
    let qi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == b)
        .expect("its polity");
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.take_in_for_tests(ctx, pi, qi, civ_agents::polity::HeldHow::Annexed)
    });
    let seats = Seats {
        n: 5,
        electorate: Membership::Adults,
        method: SeatMethod::Election,
        term_days: 40,
        by_settlement: true,
    };
    let body = Body {
        members: Membership::Seats,
        quorum_share: 0.5,
        pass: PassRule::MoreForThanAgainst,
        seats: Some(seats),
    };
    let now = sim.now();
    let first = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p.id)
        .min()
        .expect("people");
    {
        let polity = &mut sim.people_mut_for_tests().polities[pi];
        polity.body = body;
        polity.versions.push(CustomVersion {
            body,
            since: now,
            law: None,
            seized_by: Some(first),
        });
    }
    sim.advance_minutes(DAY).expect("advances");
    let (districts, called) = {
        let e = sim.people().polities[pi]
            .elections
            .last()
            .expect("an election is called");
        (e.districts.clone(), e.called)
    };
    // Each settlement a district, the seats shared out among them, the seat's first.
    assert_eq!(
        districts.iter().map(|d| d.0).collect::<Vec<_>>(),
        vec![a, b]
    );
    assert_eq!(districts.iter().map(|d| u32::from(d.1)).sum::<u32>(), 5);
    assert!(districts.iter().all(|d| d.1 > 0), "{districts:?}");
    sim.advance_minutes(3 * DAY).expect("advances");
    saves_and_goes_on_alike(&mut sim, content(), DAY);
    let stand_days = i64::from(sim.rules().people.elections.stand_days);
    while sim.now().day_index() <= called + stand_days + 2 {
        sim.advance_minutes(DAY).expect("advances");
    }
    let pop = sim.people();
    let e = pop.polities[pi].elections.last().expect("the election");
    assert_eq!(e.stage, ElectionStage::Installed, "{:?}", e.stage);
    let lives = |p: civ_core::PermanentId| {
        pop.person(p)
            .and_then(|x| pop.household(x.household))
            .and_then(|h| h.settlement)
    };
    // Each stood where they lived when they stood (none have moved in a few days).
    for c in &e.candidates {
        assert!(c.district == Some(a) || c.district == Some(b));
        assert_eq!(c.district, lives(c.person));
    }
    // Each voter named only those of their own district, at most its seats.
    for (voter, named) in &e.ballots {
        let d = lives(*voter);
        let k = e.seats_in(d);
        assert!(named.len() <= usize::from(k));
        assert!(named.iter().all(|c| {
            e.candidates
                .iter()
                .any(|x| x.person == *c && x.district == d)
        }));
    }
    // Each district seats no more than its share, and only those who stood there.
    for &(d, k) in &e.districts {
        let won = e
            .winners
            .iter()
            .filter(|w| {
                e.candidates
                    .iter()
                    .any(|c| c.person == **w && c.district == Some(d))
            })
            .count();
        assert!(won <= usize::from(k), "{won} won in a district of {k}");
    }
    assert!(!e.winners.is_empty());
    assert_eq!(pop.polities[pi].seated.len(), e.winners.len());
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

/// M7a slice BQ (ADR-0024 §5, research 09-08 §1.9): three factions with members standing for a
/// council of five, whose organizers know every member. The largest alone would take every seat
/// if each voted its own slate; the two smaller, together larger, meet before names close and
/// agree one common slate shared by following, which both back; the largest is left alone. The
/// chronicle tells the alliance, and it saves and loads alike.
#[test]
fn two_factions_that_lose_apart_agree_one_slate_before_names_close() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(DAY).expect("advances");
    {
        let rules = sim.rules_mut_for_tests().expect("rules of its own");
        rules.people.elections.sit_cost = 0.0;
        rules.people.elections.w_standing = 0.0;
    }
    let seats = Seats {
        n: 5,
        electorate: Membership::Adults,
        method: SeatMethod::Election,
        term_days: 40,
        by_settlement: false,
    };
    let body = Body {
        members: Membership::Seats,
        quorum_share: 0.5,
        pass: PassRule::MoreForThanAgainst,
        seats: Some(seats),
    };
    let now = sim.now();
    let adult = sim.rules().people.family.independent_age;
    let settlement = sim.people().polities[0].settlement;
    let mut adults: Vec<civ_core::PermanentId> = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) >= adult)
        .filter(|p| {
            sim.people()
                .household(p.household)
                .is_some_and(|x| x.settlement == Some(settlement))
        })
        .map(|p| p.id)
        .collect();
    adults.sort_unstable();
    let n = adults.len();
    assert!(n >= 20, "a village to share ({n})");
    // Followings of about 30 %, 25 % and 40 %: the last the largest, the first two larger
    // together.
    let (ka, kb) = (n * 30 / 100, n * 25 / 100);
    let kc = n * 40 / 100;
    let groups = [
        adults[..ka].to_vec(),
        adults[ka..ka + kb].to_vec(),
        adults[ka + kb..ka + kb + kc].to_vec(),
    ];
    let ids: Vec<civ_core::PermanentId> = (0..3).map(|_| sim.allocate_id_for_tests()).collect();
    let tp = sim.rules().people.ties.clone();
    {
        let pop = sim.people_mut_for_tests();
        let against = civ_agents::word::Blamed::Body(pop.polities[0].id);
        for (g, &f) in groups.iter().zip(&ids) {
            pop.factions.list.push(civ_agents::faction::Faction::found(
                f, settlement, against, g[0], now,
            ));
            for &m in g {
                pop.factions.join(civ_agents::faction::Member {
                    person: m,
                    faction: f,
                    since: now.day_index(),
                    why: civ_agents::faction::Why::default(),
                });
            }
        }
        // The organizers know each other and every member (set down as if met at the hearth).
        for g in &groups {
            for h in &groups {
                for &m in h {
                    pop.ties.record(
                        g[0],
                        m,
                        civ_agents::ties::Act::Hearth,
                        1.0,
                        0.0,
                        now.day_index(),
                        &tp,
                    );
                }
            }
        }
        let polity = &mut pop.polities[0];
        polity.body = body;
        polity.versions.push(CustomVersion {
            body,
            since: now,
            law: None,
            seized_by: Some(adults[0]),
        });
    }
    sim.advance_minutes(DAY).expect("advances");
    let called = sim.people().polities[0]
        .elections
        .last()
        .expect("an election is called")
        .called;
    let stand_days = i64::from(sim.rules().people.elections.stand_days);
    while sim.now().day_index() <= called + stand_days + 1 {
        sim.advance_minutes(DAY).expect("advances");
    }
    let pop = sim.people();
    let e = pop.polities[0].elections.last().expect("the election");
    assert!(
        !e.stage.open() || e.stage == ElectionStage::Polling,
        "{:?}",
        e.stage
    );
    let slate_of = |f| {
        e.endorsements
            .iter()
            .find(|s| s.0 == f)
            .map(|s| s.1.clone())
    };
    let stood = |g: &[civ_core::PermanentId]| g.iter().filter(|p| e.stood(**p)).count();
    let (sa, sb, sc) = (stood(&groups[0]), stood(&groups[1]), stood(&groups[2]));
    eprintln!("stood: {sa}, {sb}, {sc}; alliances: {:?}", e.alliances);
    // The largest alone takes the seats when its slate fills them; then the two smaller gain
    // by joining.
    if sc >= 5 && sa > 0 && sb > 0 {
        assert_eq!(e.alliances.len(), 1, "{:?}", e.alliances);
        let a = &e.alliances[0];
        assert_eq!(a.factions, [ids[0], ids[1]]);
        assert_eq!(a.package, civ_agents::seats::Package::Common);
        let common = slate_of(ids[0]).expect("a slate");
        assert_eq!(
            slate_of(ids[1]),
            Some(common.clone()),
            "both back one slate"
        );
        assert!(common.len() <= 5);
        assert!(common.iter().any(|p| groups[0].contains(p)));
        assert!(common.iter().any(|p| groups[1].contains(p)));
        assert!(
            slate_of(ids[2]).is_some_and(|s| s.iter().all(|p| groups[2].contains(p))),
            "the largest keeps its own"
        );
        assert!(
            pop.chronicle
                .iter()
                .any(|x| x.kind == ChronicleKind::AllianceMade)
        );
    }
    assert!(e.alliances.iter().all(|a| a.factions[0] < a.factions[1]));
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn a_council_too_few_sat_for_is_called_again() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(DAY).expect("advances");
    // Nobody stands at first (a test's setting).
    sim.rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .elections
        .sit_cost = 100.0;
    let seats = Seats {
        n: 5,
        electorate: Membership::Adults,
        method: SeatMethod::Election,
        term_days: 360,
        by_settlement: false,
    };
    let body = Body {
        members: Membership::Seats,
        quorum_share: 0.5,
        pass: PassRule::MoreForThanAgainst,
        seats: Some(seats),
    };
    let now = sim.now();
    let first = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p.id)
        .min()
        .expect("people");
    {
        let polity = &mut sim.people_mut_for_tests().polities[0];
        polity.body = body;
        polity.versions.push(CustomVersion {
            body,
            since: now,
            law: None,
            seized_by: Some(first),
        });
    }
    let stand_days = i64::from(sim.rules().people.elections.stand_days);
    sim.advance_minutes((stand_days + 4) * DAY)
        .expect("advances");
    {
        let polity = &sim.people().polities[0];
        let e = polity.elections.first().expect("an election");
        assert_ne!(e.stage, ElectionStage::Standing, "{:?}", e.stage);
        assert!(
            (polity.seated.len() as u32) < body.quorum(5),
            "too few sat: {:?}",
            polity.seated
        );
    }
    // Standing costs nothing now; the seats are called again within a stand's length of the last
    // election's close, and filled.
    sim.rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .elections
        .sit_cost = 0.0;
    sim.advance_minutes((3 * stand_days + 6) * DAY)
        .expect("advances");
    let polity = &sim.people().polities[0];
    assert!(
        polity
            .elections
            .iter()
            .any(|e| e.why == ElectionWhy::TooFewSat),
        "{:?}",
        polity
            .elections
            .iter()
            .map(|e| (e.why, e.stage))
            .collect::<Vec<_>>()
    );
    assert!(
        (polity.seated.len() as u32) >= body.quorum(5),
        "enough sit: {:?}",
        polity.seated
    );
}
