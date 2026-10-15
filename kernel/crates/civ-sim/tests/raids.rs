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
        mustered: Vec::new(),
        arrived: Vec::new(),
        home: Vec::new(),
        stage: BandStage::Called,
        since: now.minutes(),
        clash: None,
        taken: Vec::new(),
        taken_kcal: 0.0,
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
    for _ in 0..(3 * 24) {
        sim.advance_minutes(60).expect("advances");
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
