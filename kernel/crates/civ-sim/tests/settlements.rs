//! Several settlements (ADR-0018; plan §7, M5a slice AK): founding groups placed together at
//! setup, each a band of its own; the residence histories and the accounts derived from them.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::{Origin, ResidenceWhy};
use civ_content::ContentRegistry;
use civ_core::SimTime;
use civ_land::Founding;
use civ_sim::{NewWorld, Sim, SimError};

const PRESET: &str = "core:worldgen/river_valley";

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

/// World `seed` with a fixed identity, so it lives the same life every run (its identity orders
/// the events of an instant).
fn world(seed: u64, band: u32, neighbours: &[u32]) -> Result<Sim, SimError> {
    world_knowing(seed, band, neighbours, false)
}

/// [`world`], its founding groups knowing where each other camped if `known`.
fn world_knowing(seed: u64, band: u32, neighbours: &[u32], known: bool) -> Result<Sim, SimError> {
    Sim::create_for_tests(
        &NewWorld {
            name: "Neighbours".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: band,
            neighbours: neighbours.to_vec(),
            neighbours_known: known,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
}

/// A dry, walkable point at least `metres` from every hearth, if there is one.
fn dry_ground_apart(sim: &Sim, metres: f32) -> Option<(f32, f32)> {
    let (w, h) = sim.map().extent_m();
    (1..32)
        .flat_map(|i| (1..32).map(move |j| (i, j)))
        .find_map(|(i, j)| {
            let p = (w as f32 * i as f32 / 32.0, h as f32 * j as f32 / 32.0);
            let cell = civ_agents::population::cell_of(sim.map(), p);
            let dry = sim.map().water[cell] == civ_world::WATER_LAND && sim.nav().walkable(cell);
            let apart = sim
                .land()
                .settlements
                .iter()
                .all(|s| (s.hearth_m.0 - p.0).hypot(s.hearth_m.1 - p.1) >= metres);
            (dry && apart).then_some(p)
        })
}

/// How far a field may lie from its hearth, metres, by the content.
fn field_reach_m(sim: &Sim) -> f64 {
    let p = &sim.rules().people;
    p.nav.tobler_ms(0.0) * p.nav.offtrail_factor * p.farm.max_walk_minutes * 60.0
}

#[test]
fn founding_groups_settle_apart_each_a_band_of_its_own() {
    let sim = world(3, 50, &[30]).expect("generates");
    assert_eq!(sim.founding_problem(), None);
    let land = sim.land();
    assert_eq!(land.settlements.len(), 2, "two groups, two settlements");
    let (a, b) = (&land.settlements[0], &land.settlements[1]);
    let apart = f64::from((a.hearth_m.0 - b.hearth_m.0).hypot(a.hearth_m.1 - b.hearth_m.1));
    assert!(
        apart >= 2.0 * field_reach_m(&sim) - 1.0,
        "{apart:.0} m apart: their fields' reaches overlap"
    );
    for s in [a, b] {
        assert_eq!(s.founding, Founding::Setup);
        assert_eq!(s.parent, None);
        assert_eq!(s.abandoned, None);
    }
    let pop = sim.people();
    assert_eq!(pop.residents(a.id), 50);
    assert_eq!(pop.residents(b.id), 30);
    for r in pop.records.values() {
        assert_eq!(r.origin, Origin::Founder);
        assert_eq!(r.residence.len(), 1, "one stay each");
        assert_eq!(r.residence[0].why, ResidenceWhy::Founder);
    }
    // Each band builds as its own people do: the two traditions are drawn apart.
    let mean_pitch = |s| {
        let hs: Vec<f64> = pop
            .households
            .iter()
            .filter(|(_, h)| h.settlement == Some(s))
            .map(|(_, h)| f64::from(h.taste.pitch_centideg))
            .collect();
        hs.iter().sum::<f64>() / hs.len().max(1) as f64
    };
    assert!((mean_pitch(a.id) - mean_pitch(b.id)).abs() > 1e-3);
    assert!(pop.residence_problems().is_empty());
}

#[test]
fn the_order_founding_groups_are_listed_in_does_not_choose_their_land() {
    let one = world(3, 50, &[30]).expect("generates");
    let other = world(3, 30, &[50]).expect("generates");
    let hearth_of = |sim: &Sim, size: u32| {
        sim.land()
            .settlements
            .iter()
            .find(|s| sim.people().residents(s.id) == size)
            .map(|s| s.hearth_m)
            .expect("the group settled")
    };
    assert_eq!(hearth_of(&one, 50), hearth_of(&other, 50));
    assert_eq!(hearth_of(&one, 30), hearth_of(&other, 30));
}

#[test]
fn a_world_takes_at_most_three_founding_groups() {
    assert!(matches!(
        world(3, 0, &[0, 0, 0]),
        Err(SimError::TooManyGroups(4))
    ));
}

#[test]
fn every_settlement_s_accounts_balance_and_agree_with_who_lives_there() {
    let mut sim = world(3, 50, &[30]).expect("generates");
    let start = sim.now();
    sim.advance_minutes(150 * 24 * 60).expect("lives");
    // A family the observer sends far from both camps founds a third settlement: arrivals from
    // off the map.
    let far = dry_ground_apart(&sim, 1200.0).expect("dry ground far from both camps");
    let sent = Some(sim.spawn_family(far).expect("a family arrives"));
    sim.advance_minutes(30 * 24 * 60).expect("lives");
    let now = sim.now().plus_minutes(1);
    let pop = sim.people();
    assert!(
        pop.problems(sim.ids().peek_next(), sim.rules().catalog.activities.len())
            .is_empty()
    );
    let mut born = 0;
    for s in &sim.land().settlements {
        let a = pop.accounts(s.id, start, now);
        assert!(a.balance(), "{}: {a:?}", s.name);
        assert_eq!(a.end, pop.residents(s.id), "{}: {a:?}", s.name);
        assert_eq!(
            a.departures.keys().filter(|d| d.is_some()).count(),
            0,
            "nobody moves between settlements yet"
        );
        assert!(
            a.arrivals.keys().all(Option::is_none),
            "everyone came from off the map"
        );
        born += a.births;
    }
    let records_born = pop
        .records
        .values()
        .filter(|r| r.origin == Origin::Born)
        .count() as u32;
    assert_eq!(born, records_born, "every birth counted once");
    if let Some(sent) = sent {
        assert!(sent.founded, "far from both camps it makes its own");
        let camp = sim
            .land()
            .settlements
            .iter()
            .find(|s| s.id == sent.settlement)
            .expect("its settlement");
        assert_eq!(camp.founding, Founding::Sent);
        let s = pop
            .accounts(sent.settlement, SimTime::ZERO, now)
            .arrivals
            .get(&None)
            .copied();
        assert!(s.unwrap_or(0) >= sent.people.len() as u32);
        assert!(
            sent.people
                .iter()
                .all(|p| pop.records[p].residence[0].why == ResidenceWhy::Arrived)
        );
    }
}

#[test]
fn nothing_passes_between_settlements_that_have_no_contact() {
    use civ_agents::person::KnowSource;
    use civ_core::PermanentId;
    let mut sim = world(3, 50, &[30]).expect("generates");
    sim.advance_minutes(2 * 365 * 24 * 60).expect("lives");
    let pop = sim.people();
    // Where each person lived: nobody moves between settlements yet, so their first stay.
    let home = |p: PermanentId| {
        pop.records
            .get(&p)
            .and_then(|r| r.residence.first())
            .and_then(|s| s.settlement)
    };
    let mut across = Vec::new();
    for h in pop.ties.holders() {
        for t in pop.ties.of(h) {
            if home(h) != home(t.to) {
                across.push(format!("{h} holds a tie to {} ({:?})", t.to, t.reason));
            }
        }
    }
    for u in &pop.unions {
        if home(u.woman) != home(u.man) {
            across.push(format!("{} and {} married", u.woman, u.man));
        }
    }
    for h in &pop.word.heard {
        if let Some(f) = h.from.filter(|&f| home(f) != home(h.holder)) {
            across.push(format!("{} heard claim {} from {f}", h.holder, h.claim));
        }
        if let Some(c) = pop.word.claims.iter().find(|c| c.id == h.claim)
            && Some(c.settlement) != home(h.holder)
        {
            across.push(format!(
                "{} heard of another settlement's claim {}",
                h.holder, c.id
            ));
        }
    }
    for x in &pop.ideologies.held {
        if let Some(f) = x.from.filter(|&f| home(f) != home(x.holder)) {
            across.push(format!("{} took up an ideology from {f}", x.holder));
        }
    }
    for (_, p) in pop.people.iter() {
        for k in &p.knows {
            if let KnowSource::Upbringing(q) | KnowSource::Taught(q) = k.source
                && home(q) != home(p.id)
            {
                across.push(format!("{} learnt a technique from {q}", p.id));
            }
        }
    }
    for i in &pop.order.incidents {
        if let Some(target) = pop.household(i.target).and_then(|x| x.settlement)
            && home(i.actor) != Some(target)
        {
            across.push(format!(
                "{} went to take from household {}",
                i.actor, i.target
            ));
        }
    }
    for m in &pop.factions.members {
        if let Some(f) = pop.factions.get(m.faction)
            && home(m.person) != Some(f.settlement)
        {
            across.push(format!(
                "{} joined a faction of another settlement",
                m.person
            ));
        }
    }
    let land = sim.land();
    for (_, h) in pop.households.iter() {
        let Some(b) = h
            .admired
            .and_then(|b| land.buildings.iter().find(|x| x.id == b))
        else {
            continue;
        };
        if pop.household(b.household).and_then(|x| x.settlement) != h.settlement {
            across.push(format!("household {} admired a building elsewhere", h.id));
        }
    }
    assert!(across.is_empty(), "{} crossings: {across:#?}", across.len());
    // Within each settlement people did meet and come to know one another, so the test can see a
    // crossing if one happens. (Word passes only when there is news to tell, a gathering called
    // or a grievance held, which two years may not bring.)
    for s in &land.settlements {
        let ties = pop
            .ties
            .holders()
            .into_iter()
            .filter(|&h| home(h) == Some(s.id))
            .count();
        assert!(ties > 0, "{}: nobody holds a tie", s.name);
    }
}

#[test]
fn founding_groups_that_know_each_other_know_where_each_camped_and_no_more() {
    use civ_agents::places::PlaceHow;
    let strangers = world(3, 50, &[30]).expect("generates");
    assert!(
        strangers
            .people()
            .known_places
            .known
            .values()
            .all(Vec::is_empty),
        "strangers know no place but their own"
    );
    let sim = world_knowing(3, 50, &[30], true).expect("generates");
    let pop = sim.people();
    let land = sim.land();
    for (_, h) in pop.households.iter() {
        let known = pop.known_places.of(h.id);
        assert_eq!(known.len(), 1, "household {} knows the other camp", h.id);
        assert_ne!(Some(known[0].settlement), h.settlement, "never its own");
        assert!(land.settlements.iter().any(|s| s.id == known[0].settlement));
        assert_eq!(known[0].how, PlaceHow::Founded);
    }
    assert!(
        pop.problems(sim.ids().peek_next(), sim.rules().catalog.activities.len())
            .is_empty()
    );
}

#[test]
fn a_walk_in_sight_of_another_settlement_makes_it_known_and_word_of_it_goes_round() {
    use civ_agents::places::PlaceHow;
    let mut sim = world(3, 50, &[30]).expect("generates");
    // Bring the second camp's hearth to 900 m east of the first's, so that some of the first's
    // walks pass within sight of it and others do not.
    let first = sim.land().settlements[0].clone();
    let near = (first.hearth_m.0 + 900.0, first.hearth_m.1);
    sim.land_mut_for_tests().settlements[1].hearth_m = near;
    let other = sim.land().settlements[1].id;
    sim.advance_minutes(40 * 24 * 60).expect("lives");
    let pop = sim.people();
    let ours: Vec<_> = pop
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(first.id))
        .map(|(_, h)| h.id)
        .collect();
    let how = |how: PlaceHow| {
        ours.iter()
            .filter(|&&h| {
                pop.known_places
                    .of(h)
                    .iter()
                    .any(|k| k.settlement == other && k.how == how)
            })
            .count()
    };
    let (seen, told) = (how(PlaceHow::Seen), how(PlaceHow::Told));
    assert!(seen > 0, "nobody walked within sight of it");
    assert!(told > 0, "nobody was told of it ({seen} saw it)");
    for &h in &ours {
        for k in pop.known_places.of(h) {
            if k.how == PlaceHow::Told {
                let teller = k.from.expect("someone told them");
                assert!(pop.records.contains_key(&teller));
            }
        }
    }
}
