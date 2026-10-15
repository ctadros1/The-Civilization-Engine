//! Force between polities (ADR-0023; plan §7, M6c). Slice BI: a household short of food may take
//! from a home of another settlement its people have seen, and only from one they have seen; one
//! who knows who took from their settlement's stores holds that the takers' polity harms theirs,
//! and their household holds the issue `raided`; a stranger from elsewhere is not named; and the
//! incident keeps the taker's settlement through a save.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::crime::{Belief, Incident, KnownAmount, Outcome, Source};
use civ_agents::params::GoodUse;
use civ_agents::places::PlaceHow;
use civ_agents::views::Domain;
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

/// Two villages of `band`, founded apart and unknown to each other, with a fixed identity.
fn two_villages(seed: u64, band: u32) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Neighbours".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: band,
            neighbours: vec![band],
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

/// The households of settlement `s` with people, in id order.
fn households_of(sim: &Sim, s: PermanentId) -> Vec<PermanentId> {
    let mut out: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(s) && !h.members.is_empty())
        .map(|(_, h)| h.id)
        .collect();
    out.sort_unstable();
    out
}

#[test]
fn a_household_short_of_food_takes_only_from_homes_of_another_settlement_it_has_seen() {
    let mut sim = two_villages(3, 30);
    sim.advance_minutes(DAY).expect("advances");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (ours, theirs) = (households_of(&sim, a), households_of(&sim, b));
    let (taker, held) = (ours[0], theirs[0]);
    let rules = sim.rules().clone();
    let goods = &rules.catalog.goods;
    let provisions = goods
        .iter()
        .position(|g| g.id == "core:good/provisions")
        .expect("provisions");
    let day = sim.now().day_index();
    {
        let pop = sim.people_mut_for_tests();
        let at = pop.household(taker).expect("the taker's household").home;
        // A household of the other village lives a short walk from the taker's home: the M5
        // worlds' villages are kilometres apart, beyond what anyone walks to take.
        for (_, h) in pop.households.iter_mut() {
            if h.id == held {
                h.home = (at.0 + 40.0, at.1);
            }
            let ours = h.settlement == Some(a);
            for (kg, g) in h.stores.iter_mut().zip(goods) {
                if g.purpose == GoodUse::Food && (ours || h.id == held) {
                    *kg = 0.0;
                }
            }
            if h.id == held {
                let need = h.members.len() as f64 * rules.people.household.daily_kcal_per_person;
                h.stores[provisions] = 30.0 * need / goods[provisions].kcal_per_kg;
            }
        }
        // Only the taker's household has seen the other village's homes.
        pop.known_places.learn(taker, b, day, PlaceHow::Seen, None);
        let members = pop.household(taker).expect("household").members.clone();
        for (_, p) in pop.people.iter_mut() {
            if members.contains(&p.id) {
                p.objection = 0.0;
            }
        }
    }
    let mut found: Option<Incident> = None;
    for _ in 0..30 {
        sim.advance_minutes(DAY).expect("advances");
        found = sim
            .people()
            .order
            .incidents
            .iter()
            .find(|i| i.across.is_some())
            .cloned();
        if found.is_some() {
            break;
        }
    }
    let i = found.expect("someone of the hungry household went to the store it had seen");
    assert_eq!(i.actor_household, taker);
    assert_eq!(i.target, held);
    assert_eq!(
        i.across,
        Some(a),
        "the incident names the taker's settlement"
    );
    // Nobody else of the village went there: they had never seen it.
    assert!(
        sim.people()
            .order
            .incidents
            .iter()
            .filter(|x| x.target == held)
            .all(|x| x.actor_household == taker)
    );
    // Those of the other village who saw it name the taker only if they know them.
    let tp = &rules.people.ties;
    let pop = sim.people();
    for &w in &i.seen_by {
        let Some(belief) = pop.order.belief(w, i.id) else {
            continue;
        };
        let theirs = pop
            .person(w)
            .and_then(|p| pop.household(p.household))
            .and_then(|x| x.settlement)
            == Some(b);
        if theirs
            && pop
                .ties
                .tie(w, i.actor, sim.now().day_index(), tp)
                .is_none()
        {
            assert_eq!(belief.taker, None, "a stranger from elsewhere is not named");
        }
    }
}

#[test]
fn knowing_who_of_another_village_took_holds_its_polity_harms_ours_and_raises_raided() {
    let mut sim = two_villages(3, 30);
    sim.advance_minutes(DAY).expect("advances");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (ours, theirs) = (households_of(&sim, a), households_of(&sim, b));
    let (taker_hh, held) = (ours[0], theirs[0]);
    let now = sim.now();
    let day = now.day_index();
    let pop = sim.people();
    let taker = pop.household(taker_hh).expect("household").members[0];
    let holder = pop.household(held).expect("household").members[0];
    let polity_a = pop
        .polities
        .iter()
        .find(|p| p.settlement == a)
        .expect("a polity")
        .id;
    let rp = sim.rules().people.relations.clone();
    let before = sim
        .people()
        .polity_views
        .of(holder, polity_a, day, &rp)
        .map_or(rp.prior as f32, |v| v.yes[Domain::HarmsUs.index()]);
    assert!(
        !sim.people().raided_within_for_tests(held, day - 365),
        "nothing taken yet"
    );
    let id = {
        let order = &mut sim.people_mut_for_tests().order;
        let id = order.incidents.last().map_or(1, |i| i.id + 1);
        order.incidents.push(Incident {
            id,
            at: now,
            actor: taker,
            actor_household: taker_hh,
            target: held,
            place: (0.0, 0.0),
            outcome: Outcome::Taken,
            goods: Vec::new(),
            kcal: 5_000.0,
            seen_by: Vec::new(),
            noticed: true,
            food_days: f32::NAN,
            richer: f32::NAN,
            across: Some(a),
        });
        order.amounts.push(KnownAmount {
            household: held,
            incident: id,
            kcal: 5_000.0,
            lost: true,
            day,
        });
        id
    };
    // Their household found the loss and knows nobody for it: not yet raided, as they know.
    assert!(!sim.people().raided_within_for_tests(held, day - 365));
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.come_to_believe_for_tests(
            ctx,
            Belief {
                holder,
                incident: id,
                taker: Some(taker),
                source: Source::Told,
                from: None,
                origin: None,
                day,
            },
        );
    });
    assert!(
        sim.people().raided_within_for_tests(held, day - 365),
        "a household that knows who of another village took from it holds `raided`"
    );
    let after = sim
        .people()
        .polity_views
        .of(holder, polity_a, day, &rp)
        .expect("a view of the takers' polity")
        .yes[Domain::HarmsUs.index()];
    assert!(
        (f64::from(after - before) - rp.heard_taking).abs() < 1e-4,
        "told of it: `heard_taking` toward harms us ({before} -> {after})"
    );
    // The incident keeps the taker's settlement through a save.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "raided",
    )
    .expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let i = loaded
        .people()
        .order
        .incident(id)
        .expect("the incident is kept");
    assert_eq!(i.across, Some(a));
    assert!(loaded.people().raided_within_for_tests(held, day - 365));
}

/// A band of `n` adults of settlement `a`, called by the first, to the first household of `b`,
/// mustering at `muster_hour` the next day: set down as if called (the calling is a choice weighed
/// elsewhere; this shows what follows from it).
fn a_band(sim: &mut Sim, a: PermanentId, b: PermanentId, n: usize, muster_hour: i64) -> u32 {
    use civ_agents::war::{Band, BandPurpose, BandStage};
    let now = sim.now();
    let adult = sim.rules().people.bands.min_age;
    let theirs = households_of(sim, b)[0];
    let pop = sim.people_mut_for_tests();
    let place = pop.household(theirs).expect("household").home;
    let mut adults: Vec<PermanentId> = pop
        .people
        .iter()
        .filter(|(_, p)| p.age_years(now) >= adult && p.age_years(now) <= 50.0)
        .filter(|(_, p)| pop.household(p.household).and_then(|x| x.settlement) == Some(a))
        .map(|(_, p)| p.id)
        .collect();
    adults.sort_unstable();
    adults.truncate(n);
    let id = pop.order.bands.last().map_or(1, |x| x.id + 1);
    pop.order.bands.push(Band {
        id,
        organizer: adults[0],
        settlement: a,
        purpose: BandPurpose::Take,
        target: theirs,
        place,
        called: now.minutes(),
        muster: (now.day_index() + 1) * DAY + muster_hour * 60,
        members: adults,
        asked: Vec::new(),
        mustered: Vec::new(),
        arrived: Vec::new(),
        home: Vec::new(),
        stage: BandStage::Called,
        since: now.minutes(),
        clash: None,
        incident: None,
        law: None,
        taken: Vec::new(),
        taken_kcal: 0.0,
        approach: None,
    });
    id
}

#[test]
fn a_band_musters_walks_to_the_household_meets_whoever_stands_and_comes_home() {
    use civ_agents::war::BandStage;
    let mut sim = two_villages(3, 30);
    sim.advance_minutes(DAY).expect("advances");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let id = a_band(&mut sim, a, b, 5, 10);
    // The observer sees it called (wire 1.74), and who said they would go.
    let lines = civ_sim::frames::people::band_lines(&sim);
    let (_, going, words) = lines.iter().find(|l| l.0 == id).expect("it is shown");
    assert_eq!(going.len(), 5);
    assert!(words.contains("has called 5 of"), "{words}");
    let mut seen_out = false;
    for _ in 0..(3 * 24) {
        sim.advance_minutes(60).expect("advances");
        if let Some((_, _, w)) = civ_sim::frames::people::band_lines(&sim)
            .into_iter()
            .find(|l| l.0 == id)
            && w.contains(" leads ")
        {
            seen_out = true;
        }
        let band = sim
            .people()
            .order
            .bands
            .iter()
            .find(|x| x.id == id)
            .expect("band");
        if band.stage.over() {
            break;
        }
    }
    assert!(seen_out, "the observer saw it out");
    assert!(
        civ_sim::frames::people::band_lines(&sim)
            .iter()
            .all(|l| l.0 != id),
        "a band home is no longer shown"
    );
    let pop = sim.people();
    let band = pop.order.bands.iter().find(|x| x.id == id).expect("band");
    assert!(
        band.stage == BandStage::Home,
        "the band went and came home: {:?}, {} of {} came to the muster",
        band.stage,
        band.mustered.len(),
        band.members.len()
    );
    assert!(band.mustered.len() >= 2, "muster is who comes");
    assert!(band.mustered.iter().all(|m| band.members.contains(m)));
    let clash = pop
        .order
        .clashes
        .iter()
        .find(|c| Some(c.id) == band.clash)
        .expect("it met what it met");
    // Everyone of the band in it came at the muster and arrived; at most two blows a pair a
    // minute; an incident across the boundary records it.
    for f in clash
        .fighters
        .iter()
        .filter(|f| f.side == civ_agents::war::Side::Band)
    {
        assert!(band.arrived.contains(&f.person));
    }
    assert!(clash.blows.len() <= 2 * clash.frontage as usize * usize::from(clash.minutes.max(1)));
    let incident = pop
        .order
        .incidents
        .iter()
        .rev()
        .find(|i| i.target == band.target)
        .expect("the taking or the try is an incident");
    assert_eq!(incident.across, Some(a));
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == civ_agents::history::ChronicleKind::Raid)
    );
    // What the band took is what the incident records.
    assert!((incident.kcal - band.taken_kcal).abs() < 1.0);
    if let Some(e) = pop
        .chronicle
        .iter()
        .rev()
        .find(|e| e.kind == civ_agents::history::ChronicleKind::Raid)
    {
        eprintln!(
            "{} of {} came to the muster, {} arrived; {} minutes, {:?}: {}",
            band.mustered.len(),
            band.members.len(),
            band.arrived.len(),
            clash.minutes,
            clash.ended,
            e.name
        );
    }
}

#[test]
fn a_band_out_saves_and_loads_and_goes_on_alike() {
    use civ_agents::war::BandStage;
    let mut sim = two_villages(3, 30);
    sim.advance_minutes(DAY).expect("advances");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let id = a_band(&mut sim, a, b, 5, 10);
    for _ in 0..48 {
        sim.advance_minutes(30).expect("advances");
        let stage = sim
            .people()
            .order
            .bands
            .iter()
            .find(|x| x.id == id)
            .map(|x| x.stage);
        if stage != Some(BandStage::Called) {
            break;
        }
    }
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "band").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let same = |x: &Sim, y: &Sim| {
        let (x, y) = (persist::encode_sections(x), persist::encode_sections(y));
        assert_eq!(x.len(), y.len());
        for (p, q) in x.iter().zip(&y) {
            assert!(p.bytes == q.bytes, "section `{}` differs", p.tag);
        }
    };
    same(&sim, &loaded);
    sim.advance_minutes(DAY).expect("advances");
    loaded.advance_minutes(DAY).expect("advances");
    same(&sim, &loaded);
}

#[test]
fn who_stayed_home_loses_regard_those_about_see_the_band_and_all_there_know_of_the_raid() {
    // Step two (ADR-0023 §3, §8, §10): when the band sets off, each who went thinks less of one
    // asked who said no; anyone of another settlement awake within sight of it walking hears that
    // a band was seen; everyone in the encounter hears of the raid, which names it; and a save
    // keeps both claims.
    use civ_agents::war::BandStage;
    use civ_agents::word::ClaimKind;
    let mut sim = two_villages(3, 30);
    sim.advance_minutes(DAY).expect("advances");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let id = a_band(&mut sim, a, b, 5, 10);
    let now = sim.now();
    let adult = sim.rules().people.bands.min_age;
    let tp = sim.rules().people.ties.clone();
    let word = civ_agents::ties::Domain::Word.index();
    let (members, shirker) = {
        let pop = sim.people_mut_for_tests();
        let band = pop.order.bands.iter().find(|x| x.id == id).expect("band");
        let members = band.members.clone();
        let mut others: Vec<PermanentId> = pop
            .people
            .iter()
            .filter(|(_, p)| p.age_years(now) >= adult && p.age_years(now) <= 50.0)
            .filter(|(_, p)| pop.household(p.household).and_then(|x| x.settlement) == Some(a))
            .map(|(_, p)| p.id)
            .filter(|p| !members.contains(p))
            .collect();
        others.sort_unstable();
        let shirker = others[0];
        pop.order
            .bands
            .iter_mut()
            .find(|x| x.id == id)
            .expect("band")
            .asked = vec![shirker];
        (members, shirker)
    };
    let bad_before: Vec<f64> = members
        .iter()
        .map(|&m| {
            sim.people()
                .ties
                .tie(m, shirker, now.day_index(), &tp)
                .map_or(tp.prior, |t| f64::from(t.bad[word]))
        })
        .collect();
    let mut out_seen = false;
    for _ in 0..(3 * 24 * 4) {
        sim.advance_minutes(15).expect("advances");
        let band = sim
            .people()
            .order
            .bands
            .iter()
            .find(|x| x.id == id)
            .expect("band");
        if band.stage == BandStage::Out && !out_seen {
            out_seen = true;
            let day = sim.now().day_index();
            assert!(!band.mustered.is_empty());
            for (k, m) in members.iter().enumerate() {
                if !band.mustered.contains(m) {
                    continue;
                }
                let bad = sim
                    .people()
                    .ties
                    .tie(*m, shirker, day, &tp)
                    .map(|t| f64::from(t.bad[word]))
                    .expect("they hold a view of the one who stayed home");
                assert!(
                    bad > bad_before[k] + 0.3,
                    "staying home weighs against them: {} then {bad}",
                    bad_before[k]
                );
            }
        }
        if band.stage.over() {
            break;
        }
    }
    assert!(out_seen, "the band set off");
    let pop = sim.people();
    let band = pop.order.bands.iter().find(|x| x.id == id).expect("band");
    assert_eq!(band.stage, BandStage::Home);
    // Seen walking, by people of the other settlement only.
    let seen = pop
        .word
        .claims
        .iter()
        .find(|c| c.kind == ClaimKind::BandSeen && c.event == Some(id))
        .expect("someone about saw the band");
    assert_eq!(seen.settlement, a, "a sighting is of the band's people");
    let seers: Vec<PermanentId> = pop
        .word
        .heard
        .iter()
        .filter(|h| h.claim == seen.id && h.from.is_none())
        .map(|h| h.holder)
        .collect();
    assert!(!seers.is_empty());
    for s in &seers {
        let home = pop
            .person(*s)
            .and_then(|p| pop.household(p.household))
            .and_then(|x| x.settlement);
        assert_ne!(home, Some(a), "its own people do not report it as seen");
    }
    // The raid names the encounter, and everyone in it who lived heard of it there.
    let clash = band.clash.expect("it met what it met");
    let raid = pop
        .word
        .claims
        .iter()
        .find(|c| c.kind == ClaimKind::Raid && c.event == Some(clash))
        .expect("the raid is told");
    assert_eq!(raid.settlement, b);
    assert_eq!(raid.subject, Some(band.target));
    let x = pop
        .order
        .clashes
        .iter()
        .find(|x| x.id == clash)
        .expect("encounter");
    for f in x.fighters.iter().filter(|f| !f.killed) {
        if pop.person(f.person).is_some() {
            assert!(
                pop.word
                    .heard_by(f.person)
                    .iter()
                    .any(|h| h.claim == raid.id),
                "{:?} was there",
                f.person
            );
        }
    }
    let words = civ_sim::frames::word::claim_words(&sim, raid);
    assert!(words.contains("came in a band to"), "{words}");
    let words = civ_sim::frames::word::claim_words(&sim, seen);
    assert!(words.contains("was seen walking"), "{words}");
    // A save keeps both.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "raid").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let before: Vec<_> = sim.people().word.claims.clone();
    assert_eq!(loaded.people().word.claims, before);
    let b2 = loaded
        .people()
        .order
        .bands
        .iter()
        .find(|x| x.id == id)
        .expect("band");
    assert_eq!(b2.asked, vec![shirker]);
}

/// Puts a law of `kind` in force in settlement `s`'s polity, known to all its adults, naming
/// `holder` and against `target`: its id.
fn a_raid_law(
    sim: &mut Sim,
    s: PermanentId,
    kind: civ_agents::polity::PolicyKind,
    holder: Option<PermanentId>,
    target: Option<PermanentId>,
) -> PermanentId {
    use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided};
    let policies = &sim.rules().catalog.policies;
    let policy = policies
        .iter()
        .position(|d| d.kind == kind)
        .expect("the content has the policy");
    let sanction = policies[policy].bundles.last().copied().unwrap_or_default();
    let id = sim.allocate_id_for_tests();
    let (now, grown) = (sim.now(), sim.rules().people.family.independent_age);
    let pop = sim.people_mut_for_tests();
    let pi = pop
        .polities
        .iter()
        .position(|p| p.settlement == s)
        .expect("its polity");
    let mut adults: Vec<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(s))
        .flat_map(|(_, h)| h.members.clone())
        .filter(|&m| pop.person(m).is_some_and(|p| p.age_years(now) >= grown))
        .collect();
    adults.sort_unstable();
    let mut law = Law {
        id,
        policy: policy as u16,
        kind,
        levy_share: 0.0,
        holder,
        relief_days: 0.0,
        sanction,
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor: adults[0],
        proposed: now,
        issue: IssueKind::Raided,
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
        target,
    };
    for &a in &adults {
        law.learn(a, now.day_index());
    }
    pop.polities[pi].laws.push(law);
    id
}

#[test]
fn a_band_a_law_sends_asks_each_household_and_the_law_lapses_when_it_is_home() {
    // Step three (ADR-0023 §5): the midnight after a law to send a band is in force, it is
    // called, led by the one the law names, against a household of the settlement it names that
    // the leader's people have seen; each other household sends its most willing grown member or
    // keeps its share back; once the band is home the law has run its course.
    use civ_agents::polity::{LawStatus, PolicyKind};
    use civ_agents::war::BandStage;
    let mut sim = two_villages(12, 30);
    sim.advance_minutes(DAY).expect("advances");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let now = sim.now();
    let adult = sim.rules().people.bands.min_age;
    let leader = {
        let pop = sim.people();
        let mut grown: Vec<PermanentId> = pop
            .people
            .iter()
            .filter(|(_, p)| p.age_years(now) >= adult && p.age_years(now) <= 50.0)
            .filter(|(_, p)| pop.household(p.household).and_then(|x| x.settlement) == Some(a))
            .map(|(_, p)| p.id)
            .collect();
        grown.sort_unstable();
        grown[0]
    };
    {
        let day = now.day_index();
        let pop = sim.people_mut_for_tests();
        let hh = pop.person(leader).expect("lives").household;
        pop.known_places.learn(hh, b, day, PlaceHow::Seen, None);
    }
    let law = a_raid_law(&mut sim, a, PolicyKind::SendBand, Some(leader), Some(b));
    let to_midnight = DAY - sim.now().minutes().rem_euclid(DAY) + 1;
    sim.advance_minutes(to_midnight).expect("advances");
    let (id, households) = {
        let pop = sim.people();
        let band = pop
            .order
            .bands
            .iter()
            .find(|x| x.law == Some(law))
            .expect("the law called a band");
        assert_eq!(band.organizer, leader);
        assert_eq!(
            pop.household(band.target).and_then(|x| x.settlement),
            Some(b),
            "against the settlement the law names"
        );
        // One from each household at most, asked or gone.
        let mut seen: Vec<PermanentId> = band
            .members
            .iter()
            .chain(&band.asked)
            .filter_map(|&m| pop.person(m).map(|p| p.household))
            .collect();
        let n = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            seen.len(),
            n,
            "each household sends one or keeps its share back"
        );
        (band.id, households_of(&sim, a).len())
    };
    assert!(households >= 2);
    for _ in 0..(3 * 24) {
        sim.advance_minutes(60).expect("advances");
        let over = sim
            .people()
            .order
            .bands
            .iter()
            .find(|x| x.id == id)
            .is_some_and(|x| x.stage.over());
        if over {
            break;
        }
    }
    sim.advance_minutes(DAY).expect("advances");
    let pop = sim.people();
    let band = pop.order.bands.iter().find(|x| x.id == id).expect("band");
    assert!(band.stage.over(), "{:?}", band.stage);
    assert_ne!(band.stage, BandStage::Called);
    let l = pop
        .polities
        .iter()
        .flat_map(|p| &p.laws)
        .find(|l| l.id == law)
        .expect("the law");
    assert_eq!(l.status, LawStatus::Lapsed, "the law has run its course");
    assert_eq!(
        pop.order
            .bands
            .iter()
            .filter(|x| x.law == Some(law))
            .count(),
        1,
        "it sends one band"
    );
}

#[test]
fn under_a_law_against_raiding_a_raid_comes_before_the_raiders_own_gathering() {
    // Step three (ADR-0023 §5): where a law against raiding is in force and known, one who heard
    // of a raid their own people made, and did not go, may bring it before their own gathering
    // for the household raided; the case is heard where the raiders live, against the one who
    // led it.
    use civ_agents::polity::PolicyKind;
    let mut sim = two_villages(3, 30);
    sim.advance_minutes(DAY).expect("advances");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let law = a_raid_law(&mut sim, a, PolicyKind::AgainstRaiding, None, None);
    let id = a_band(&mut sim, a, b, 5, 10);
    let mut clash_day = None;
    for _ in 0..(3 * 24 * 4) {
        sim.advance_minutes(15).expect("advances");
        let band = sim
            .people()
            .order
            .bands
            .iter()
            .find(|x| x.id == id)
            .expect("band")
            .clone();
        if let (None, Some(c)) = (clash_day, band.clash) {
            // Everyone at home hears of it at once, so that someone who did not go knows.
            let day = sim.now().day_index();
            clash_day = Some(day);
            let pop = sim.people_mut_for_tests();
            let claim = pop
                .word
                .claims
                .iter()
                .find(|x| x.kind == civ_agents::word::ClaimKind::Raid && x.event == Some(c))
                .map(|x| x.id)
                .expect("the raid is told");
            let at_home: Vec<PermanentId> = pop
                .people
                .iter()
                .filter(|(_, p)| pop.household(p.household).and_then(|x| x.settlement) == Some(a))
                .map(|(_, p)| p.id)
                .collect();
            for p in at_home {
                pop.word.hear(p, claim, day, None, None);
            }
        }
        if clash_day.is_some_and(|d| sim.now().day_index() > d + 1) {
            break;
        }
    }
    let pop = sim.people();
    let band = pop.order.bands.iter().find(|x| x.id == id).expect("band");
    assert!(
        band.taken_kcal > 0.0,
        "the band took something to answer for"
    );
    let incident = band.incident.expect("the raid is an incident");
    let case = pop
        .order
        .cases
        .iter()
        .find(|c| c.incident == incident)
        .expect("someone brought the raid before their own gathering");
    assert_eq!(case.settlement, a, "heard where the raiders live");
    assert_eq!(case.law, law);
    assert_eq!(case.accused, band.organizer);
    assert_eq!(case.accuser, band.target);
    assert!(
        !band.mustered.contains(&case.by),
        "brought by one who did not go"
    );
}
