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

/// The adults (16 and over) living in settlement `s`, in id order, with their sex.
fn adults_of(sim: &Sim, s: civ_core::PermanentId) -> Vec<(civ_core::PermanentId, civ_agents::Sex)> {
    let pop = sim.people();
    let mut out: Vec<_> = pop
        .people
        .iter()
        .filter(|(_, p)| p.age_years(sim.now()) >= 16.0)
        .filter(|(_, p)| pop.household(p.household).and_then(|h| h.settlement) == Some(s))
        .map(|(_, p)| (p.id, p.sex))
        .collect();
    out.sort_by_key(|&(id, _)| id);
    out
}

/// Visits people of `from` made to the hearth of `to`, all years.
fn visits(sim: &Sim, from: civ_core::PermanentId, to: civ_core::PermanentId) -> u32 {
    sim.people()
        .contacts
        .years
        .iter()
        .filter(|((_, f, t), _)| *f == from && *t == to)
        .map(|(_, c)| c.visits)
        .sum()
}

#[test]
fn kin_living_in_another_settlement_are_visited_and_each_visit_is_counted() {
    // Two settlements that know where each other camped. A woman of the first is made the mother
    // of an adult of the second, as if they had married away: the visit is worth her company to
    // him and his to her, against the walk there and back (M5a slice AM).
    let mut sim = world_knowing(3, 30, &[30], true).expect("generates");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let mother = adults_of(&sim, a)
        .into_iter()
        .find(|&(_, sex)| sex == civ_agents::Sex::Female)
        .map(|(id, _)| id)
        .expect("a woman in the first settlement");
    let son = adults_of(&sim, b)[0].0;
    sim.people_mut_for_tests()
        .records
        .get_mut(&son)
        .expect("on record")
        .mother = Some(mother);
    sim.advance_minutes(60 * 24 * 60).expect("lives");

    let (there, back) = (visits(&sim, a, b), visits(&sim, b, a));
    assert!(there + back > 0, "kin visit: {there} there, {back} back");
    let pop = sim.people();
    // A visitor's household knows the place it went to, and what its member saw of its food.
    let visited: Vec<_> = pop
        .known_places
        .known
        .values()
        .flatten()
        .filter(|k| k.food.is_some())
        .collect();
    assert!(!visited.is_empty(), "a visitor saw how the place was fed");
    // A visit is presence, never residence: everyone still lives where they were founded.
    for r in pop.records.values() {
        assert_eq!(r.residence.len(), 1, "{:?}", r.residence);
    }
    // Laws are told only among those they bind: nobody knows a law of another polity.
    for polity in &pop.polities {
        for law in &polity.laws {
            for &(p, _) in &law.known {
                let home = pop
                    .person(p)
                    .and_then(|q| pop.household(q.household))
                    .and_then(|h| h.settlement);
                assert!(
                    home.is_none() || home == Some(polity.settlement),
                    "{p} knows a law of another polity"
                );
            }
        }
    }
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());

    // The visits are saved and shown.
    let words = civ_sim::frames::people::contacts_words(&sim, b);
    assert!(words.contains("visit"), "{words}");
}

#[test]
fn someone_who_found_no_partner_at_home_goes_to_look_elsewhere() {
    // Every unpartnered adult of the first settlement looked for a partner at home today and found
    // nobody: the hope of meeting someone is a reason to go to the other settlement's hearth.
    let mut sim = world_knowing(3, 30, &[30], true).expect("generates");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let today = sim.now().day_index();
    let seekers: Vec<_> = adults_of(&sim, a)
        .into_iter()
        .filter(|&(id, _)| sim.people().person(id).is_some_and(|p| p.partner.is_none()))
        .map(|(id, _)| id)
        .collect();
    assert!(!seekers.is_empty());
    for &id in &seekers {
        sim.people_mut_for_tests().unmatched.insert(id, today);
    }
    sim.advance_minutes(30 * 24 * 60).expect("lives");
    assert!(visits(&sim, a, b) > 0, "nobody went to look");
}

#[test]
fn a_tie_across_settlements_can_become_a_marriage_counted_and_known_as_kin() {
    // Two bands that know where each other camped. Every unpartnered adult of each holds warm ties
    // with those of the other sex in the other, as if they had met at visits, and all look for a
    // partner about daily. Those of the first are brothers and sisters, so none may marry another
    // at home: they find partners across (M5a slice AM; research 04-08 §1.1).
    let mut sim = world_knowing(3, 50, &[50], true).expect("generates");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let now = sim.now();
    let seeking = |sim: &Sim, s| -> Vec<(civ_core::PermanentId, civ_agents::Sex)> {
        let family = &sim.rules().people.family;
        adults_of(sim, s)
            .into_iter()
            .filter(|&(id, sex)| {
                sim.people()
                    .person(id)
                    .is_some_and(|p| p.partner.is_none() && family.seeks_at(sex, p.age_years(now)))
            })
            .collect()
    };
    let (here, there) = (seeking(&sim, a), seeking(&sim, b));
    assert!(!here.is_empty() && !there.is_empty());
    let ties = sim.rules().people.ties.clone();
    let mother = adults_of(&sim, a)
        .into_iter()
        .find(|&(id, sex)| sex == civ_agents::Sex::Female && here.iter().all(|h| h.0 != id))
        .map(|(id, _)| id)
        .expect("an older woman");
    {
        let day = now.day_index();
        let pop = sim.people_mut_for_tests();
        for &(x, _) in &here {
            pop.records.get_mut(&x).expect("on record").mother = Some(mother);
        }
        for &(x, sx) in &here {
            for &(y, sy) in &there {
                if sx == sy {
                    continue;
                }
                for _ in 0..10 {
                    pop.ties
                        .record(x, y, civ_agents::ties::Act::Hearth, 1.0, 0.0, day, &ties);
                    pop.ties
                        .record(y, x, civ_agents::ties::Act::Hearth, 1.0, 0.0, day, &ties);
                }
            }
        }
    }
    sim.rules_mut_for_tests()
        .expect("the rules are held once")
        .people
        .family
        .seek_per_month = [1.0, 1.0];
    sim.advance_minutes(30 * 24 * 60).expect("lives");

    let pop = sim.people();
    let marriages: u32 = pop.contacts.years.values().map(|c| c.marriages).sum();
    assert!(marriages > 0, "no marriage across");
    // Each who married into the other settlement lives there now, by marriage, and the couple's
    // household knows where they came from as where kin live.
    let mut moved = 0;
    for r in pop.records.values() {
        let [.., before, last] = r.residence.as_slice() else {
            continue;
        };
        if last.why != ResidenceWhy::Married || before.settlement == last.settlement {
            continue;
        }
        moved += 1;
        let Some(p) = pop.person(r.id) else {
            continue;
        };
        let kin = pop
            .known_places
            .of(p.household)
            .iter()
            .find(|k| Some(k.settlement) == before.settlement)
            .expect("the couple's household knows where the mover came from");
        assert!(matches!(
            kin.how,
            civ_agents::places::PlaceHow::Kin | civ_agents::places::PlaceHow::Founded
        ));
    }
    assert_eq!(moved, marriages, "every marriage across moved someone");
    // The chronicle says where they settled.
    let named = pop
        .chronicle
        .iter()
        .filter(|e| e.kind == civ_agents::ChronicleKind::Paired && !e.name.is_empty())
        .count();
    assert_eq!(named as u32, marriages);
    assert!(
        pop.residence_problems().is_empty(),
        "{:?}",
        pop.residence_problems()
    );
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
    let words = civ_sim::frames::people::contacts_words(&sim, a)
        + &civ_sim::frames::people::contacts_words(&sim, b);
    assert!(words.contains("marriage"), "{words}");
}

/// Makes `n` adults of settlement `s` children of `parent`, as if they had moved there.
fn kin_in(sim: &mut Sim, parent: civ_core::PermanentId, s: civ_core::PermanentId, n: usize) {
    let female = sim
        .people()
        .person(parent)
        .is_some_and(|p| p.sex == civ_agents::Sex::Female);
    let children: Vec<_> = adults_of(sim, s).into_iter().take(n).map(|x| x.0).collect();
    let pop = sim.people_mut_for_tests();
    for c in children {
        let r = pop.records.get_mut(&c).expect("on record");
        if female {
            r.mother = Some(parent);
        } else {
            r.father = Some(parent);
        }
    }
}

/// A household of settlement `s` with an adult and others, and that adult.
fn household_of(
    sim: &Sim,
    s: civ_core::PermanentId,
) -> (civ_core::PermanentId, civ_core::PermanentId) {
    let pop = sim.people();
    let mut found: Vec<_> = pop
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(s) && x.members.len() >= 3)
        .map(|(_, x)| (x.id, x.members[0]))
        .collect();
    found.sort();
    found[0]
}

#[test]
fn a_household_drawn_by_kin_elsewhere_moves_there_after_two_reviews() {
    // Five adults of the second settlement are children of a member of a household of the first:
    // once two of its reviews running find it worth more to live near them, it moves (M5a slice
    // AN, ADR-0018 §5).
    let mut sim = world_knowing(3, 50, &[50], true).expect("generates");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (household, parent) = household_of(&sim, a);
    kin_in(&mut sim, parent, b, 5);
    let members = sim
        .people()
        .household(household)
        .expect("lives")
        .members
        .clone();
    sim.people_mut_for_tests().review_due.insert(household);
    sim.advance_minutes(24 * 60).expect("lives");
    assert!(
        sim.people().household(household).is_some(),
        "one review is not enough"
    );
    assert_eq!(
        sim.people()
            .leanings
            .get(&household)
            .map(|l| (l.settlement, l.reviews)),
        Some((b, 1))
    );
    sim.people_mut_for_tests().review_due.insert(household);
    sim.advance_minutes(24 * 60).expect("lives");

    let pop = sim.people();
    assert!(
        pop.household(household).is_none(),
        "the household it was is no more"
    );
    let moved: Vec<_> = members.iter().filter_map(|&m| pop.person(m)).collect();
    assert!(!moved.is_empty());
    let new = moved[0].household;
    for p in &moved {
        assert_eq!(p.household, new, "they moved together");
        let last = pop.records[&p.id].residence.last().expect("a stay");
        assert_eq!((last.settlement, last.why), (Some(b), ResidenceWhy::Moved));
    }
    assert_eq!(pop.household(new).and_then(|x| x.settlement), Some(b));
    let lived = pop.known_places.of(new).iter().find(|k| k.settlement == a);
    assert_eq!(
        lived.map(|k| k.how),
        Some(civ_agents::places::PlaceHow::Lived)
    );
    let people: u32 = pop.contacts.years.values().map(|c| c.moved).sum();
    assert_eq!(people as usize, members.len());
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == civ_agents::ChronicleKind::Moved && e.settlement == Some(b))
    );
    assert!(
        pop.residence_problems().is_empty(),
        "{:?}",
        pop.residence_problems()
    );
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
    let words = civ_sim::frames::people::contacts_words(&sim, b);
    assert!(words.contains("moved here from"), "{words}");
}

#[test]
fn a_household_out_of_food_goes_where_its_kin_are_and_its_kin_follow() {
    // Just before midnight the whole first settlement is out of food and worn down, and gives up
    // at the first chance. The household with kin in the second goes there, and the households of
    // the band, kin of those who just went, follow them, one after another (research 05-06 §1.2:
    // chain migration); none is drawn beyond the map while kin are a walk away (M5a slice AN,
    // ADR-0018 §3, §5).
    let mut sim = world_knowing(3, 50, &[50], true).expect("generates");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (household, parent) = household_of(&sim, a);
    kin_in(&mut sim, parent, b, 5);
    let to_midnight = 24 * 60 - sim.now().minutes().rem_euclid(24 * 60) - 10;
    sim.advance_minutes(to_midnight).expect("lives");
    let now = sim.now();
    let members = sim
        .people()
        .household(household)
        .expect("lives")
        .members
        .clone();
    sim.rules_mut_for_tests()
        .expect("the rules are held once")
        .people
        .household
        .leave_per_day = 1.0;
    {
        let pop = sim.people_mut_for_tests();
        let mut starving = Vec::new();
        for (_, x) in pop.households.iter_mut() {
            if x.settlement == Some(a) {
                x.stores.iter_mut().for_each(|kg| *kg = 0.0);
                x.stores_at = now;
                starving.extend(x.members.iter().copied());
            }
        }
        for (_, p) in pop.people.iter_mut() {
            if starving.contains(&p.id) {
                p.energy_kcal = -1.0e7;
                p.burn_kcal_min = 0.0;
                p.needs_at = now;
            }
        }
    }
    sim.advance_minutes(20).expect("lives");

    let pop = sim.people();
    let p = pop.person(members[0]).expect("still in the world");
    assert_eq!(
        pop.household(p.household).and_then(|x| x.settlement),
        Some(b),
        "they went to their kin"
    );
    assert!(pop.records[&members[0]].left.is_none());
    let moved: Vec<_> = pop
        .chronicle
        .iter()
        .filter(|e| e.kind == civ_agents::ChronicleKind::Moved)
        .collect();
    assert!(moved.len() > 1, "kin followed");
    assert!(moved.iter().all(|e| e.settlement == Some(b)));
    let left = pop.records.values().filter(|r| r.left.is_some()).count();
    assert_eq!(left, 0, "nobody went beyond the map");
    assert!(
        pop.residence_problems().is_empty(),
        "{:?}",
        pop.residence_problems()
    );
}

#[test]
fn an_exile_goes_where_kin_live_and_one_drawn_nowhere_leaves_the_map() {
    // Two settlements that know where each other camped. One adult of the first has kin in the
    // second; another has none there. Exiled, the first lives on near their kin; the second goes
    // beyond the map, as exiles did before (M5a slice AN; ADR-0015 §5, ADR-0018 §3).
    let mut sim = world_knowing(3, 50, &[50], true).expect("generates");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (_, parent) = household_of(&sim, a);
    kin_in(&mut sim, parent, b, 3);
    let kinless = {
        let pop = sim.people();
        let parents_kin = pop.records[&parent].mother;
        adults_of(&sim, a)
            .into_iter()
            .map(|x| x.0)
            .find(|&x| {
                x != parent
                    && pop.records[&x].mother != parents_kin
                    && pop.records[&x].mother != Some(parent)
                    && pop.records[&x].father != Some(parent)
            })
            .expect("someone with no kin there")
    };
    sim.exile_for_tests(parent);
    sim.exile_for_tests(kinless);
    sim.advance_minutes(60).expect("goes on");

    let pop = sim.people();
    let p = pop.person(parent).expect("still in the world");
    let home = pop
        .household(p.household)
        .expect("a household of their own");
    assert_eq!((home.settlement, home.members.len()), (Some(b), 1));
    let last = pop.records[&parent].residence.last().expect("a stay");
    assert_eq!((last.settlement, last.why), (Some(b), ResidenceWhy::Exiled));
    assert!(pop.records[&parent].left.is_none());
    assert!(
        pop.person(kinless).is_none(),
        "drawn nowhere, they left the map"
    );
    assert!(pop.records[&kinless].left.is_some());
    let moved: u32 = pop.contacts.years.values().map(|c| c.moved).sum();
    assert_eq!(moved, 1);
    assert!(
        pop.residence_problems().is_empty(),
        "{:?}",
        pop.residence_problems()
    );
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    use civ_sim::persist;
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "wave").expect("saves");
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

#[test]
fn a_migration_wave_comes_over_its_days_as_kin_of_one_band_knowing_what_lies_near() {
    use civ_agents::influence::InfluenceKind;
    let mut sim = world(3, 30, &[30]).expect("generates");
    sim.advance_minutes(10 * 24 * 60).expect("lives");
    let start = sim.now();
    let far = dry_ground_apart(&sim, 1200.0).expect("dry ground far from both camps");
    assert!(
        sim.send_wave(far, 4, 3, 6).is_err(),
        "fewer than five households"
    );
    assert!(sim.send_wave(far, 10, 8, 6).is_err(), "more than a week");
    assert!(
        sim.send_wave(far, 10, 3, 13).is_err(),
        "more than a year's food"
    );
    let (record, came) = sim.send_wave(far, 10, 3, 6).expect("a wave comes");
    // Ten households over three days: four at once, the rest at the next two midnights.
    assert_eq!(came.len(), 4);
    assert!(came[0].founded, "far from both camps it makes its own");
    assert!(
        came[1..]
            .iter()
            .all(|s| !s.founded && s.settlement == came[0].settlement)
    );
    let camp = came[0].settlement;
    assert_eq!(
        sim.land()
            .settlements
            .iter()
            .find(|s| s.id == camp)
            .map(|s| s.founding),
        Some(Founding::Wave)
    );
    let pop = sim.people();
    let i = pop
        .influences
        .list
        .iter()
        .find(|i| i.id == record)
        .expect("one record");
    assert_eq!((i.kind, i.subject), (InfluenceKind::Wave, 10));
    assert_eq!(i.target, came[0].people[0]);
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.name.contains("the first of a wave of 10 households")),
        "the chronicle tells it once"
    );
    let wave = pop.influences.wave(record).expect("its wave").clone();
    assert_eq!((wave.arrived, wave.came), (4, 4));
    assert!(!wave.done());
    // Each household carries six months' food for its people, and seed besides.
    let people = &sim.rules().people;
    let good = people.band.provisions_good;
    let kcal = sim.rules().catalog.goods[good].kcal_per_kg;
    for s in &came {
        let h = pop.household(s.household).expect("its household");
        let n = h.members.len() as f64;
        let food = n * people.household.daily_kcal_per_person * 182.0 / kcal;
        let held = h.stores[good];
        assert!(
            held >= food - 1e-6 && held <= food + n * people.band.seed_kg_per_person + 1e-6,
            "{held} kg against {food} kg of food"
        );
    }
    // In each run of three households one of each couple is a child of the same parents, who
    // stayed behind: brothers and sisters.
    let child_of = |s: &civ_agents::Spawned,
                    (m, f): (civ_core::PermanentId, civ_core::PermanentId)| {
        s.people.iter().any(|q| {
            let r = &pop.records[q];
            r.mother == Some(m) && r.father == Some(f)
        })
    };
    assert!(came[..3].iter().all(|s| child_of(s, wave.parents[0])));
    assert!(child_of(&came[3], wave.parents[1]));
    assert!(!child_of(&came[3], wave.parents[0]));
    // They know the settlements near where they were sent, and those they passed in sight of.
    let mut told = 0;
    for s in &sim.land().settlements {
        if s.id == camp {
            continue;
        }
        let near = (s.hearth_m.0 - far.0).hypot(s.hearth_m.1 - far.1) <= civ_agents::WAVE_KNOWN_M;
        let seen = civ_agents::places::distance_to_walk(&[wave.edge, far], s.hearth_m)
            <= people.places.sight_m;
        for h in &came {
            let known = pop
                .known_places
                .of(h.household)
                .iter()
                .any(|k| k.settlement == s.id);
            assert_eq!(known, near || seen, "{}", s.name);
        }
        told += usize::from(near && !seen);
    }
    assert!(
        told > 0,
        "this world has settlements near enough to be told of"
    );
    // Saved with households still to come, a world goes on alike: they come all the same.
    sim.advance_minutes(24 * 60).expect("lives");
    let w = sim.people().influences.wave(record).expect("its wave");
    assert_eq!(w.arrived, 7, "three more at the first midnight");
    saves_and_goes_on_alike(&mut sim, 2 * 24 * 60);
    let pop = sim.people();
    let w = pop.influences.wave(record).expect("its wave");
    assert!(w.done());
    assert_eq!(w.came, 10, "all found room");
    // Everyone it brought came from off the map to the camp; the accounts balance.
    let now = sim.now().plus_minutes(1);
    let a = pop.accounts(camp, start, now);
    assert!(a.balance(), "{a:?}");
    assert_eq!(a.arrivals.get(&None).copied(), Some(w.people.len() as u32));
    assert!(
        w.people
            .iter()
            .all(|p| pop.records[p].residence[0].why == ResidenceWhy::Arrived)
    );
    assert!(
        pop.problems(sim.ids().peek_next(), sim.rules().catalog.activities.len())
            .is_empty()
    );
}

#[test]
fn a_household_gathers_its_kin_and_founds_a_settlement_when_they_hold_enough_to_go() {
    // One village. A household's member has adult children in four other households of it, and
    // its people have walked land two field walks out. With founding made worth something to it
    // alone (moving's cost below nothing, founding's nothing, the walk free), it gathers; its
    // kin's households
    // join only once they would hold food and seed enough to go with it, and when its plan has
    // won two reviews they go if they hold enough, and else wait (M5a slice AO; research 05-06
    // §1.3, 10-01 §2.3).
    use civ_agents::places::{CoalitionFate, Lacking};
    let mut sim = world(3, 50, &[]).expect("generates");
    sim.advance_minutes(30 * 24 * 60).expect("lives");
    let content_rules = {
        let p = &sim.rules().people;
        (p.moving.cost, p.founding.cost, p.decision.w_walk_hour)
    };
    {
        let rules = sim.rules_mut_for_tests().expect("rules of its own");
        rules.people.moving.cost = -1.0;
        rules.people.founding.cost = 0.0;
        rules.people.decision.w_walk_hour = 0.0;
    }
    let a = sim.land().settlements[0].id;
    let hearth = sim.land().settlements[0].hearth_m;
    let (organizer, parent) = household_of(&sim, a);
    let female = sim
        .people()
        .person(parent)
        .is_some_and(|p| p.sex == civ_agents::Sex::Female);
    let children: Vec<_> = adults_of(&sim, a)
        .into_iter()
        .map(|x| x.0)
        .filter(|&q| sim.people().person(q).map(|p| p.household) != Some(organizer))
        .take(4)
        .collect();
    // Land walked two field walks out: the centre of a patch on dry, gentle ground.
    let reach = field_reach_m(&sim) as f32;
    let patches = &sim.land().patches;
    let site = (0..patches.class.len())
        .find(|&p| {
            let c = patches.centre_m(p);
            let d = (c.0 - hearth.0).hypot(c.1 - hearth.1);
            let cell = civ_agents::population::cell_of(sim.map(), c);
            d > 2.0 * reach + 100.0
                && d < 2.0 * reach + 1500.0
                && sim.map().water[cell] == civ_world::WATER_LAND
                && sim.nav().walkable(cell)
                && f64::from(civ_world::terrain::slope_at(sim.map(), cell))
                    <= sim.rules().people.band.site_max_slope
        })
        .expect("dry, gentle land two field walks out");
    let day = sim.now().day_index();
    {
        let pop = sim.people_mut_for_tests();
        for &c in &children {
            let r = pop.records.get_mut(&c).expect("on record");
            if female {
                r.mother = Some(parent);
            } else {
                r.father = Some(parent);
            }
        }
        pop.forget_derived();
        for (_, x) in pop.households.iter_mut() {
            if x.id == organizer {
                x.known.push(civ_agents::person::KnownPatch {
                    resource: 0,
                    patch: site as u32,
                    rate: 1.0,
                    hours: 1.0,
                    seen_day: day,
                });
            }
        }
        pop.review_due.insert(organizer);
    }
    sim.advance_minutes(24 * 60).expect("lives");
    let c = sim
        .people()
        .coalitions
        .last()
        .cloned()
        .expect("it began to gather");
    assert_eq!((c.organizer, c.from, c.reviews), (organizer, a, 1));
    assert_eq!(c.fate, CoalitionFate::Gathering);
    let kin_households: Vec<_> = children
        .iter()
        .filter_map(|&q| sim.people().person(q).map(|p| p.household))
        .collect();
    // With what a band brought sown or eaten, they would hold too little to go: nobody who would
    // leave it short is asked, and it gathers alone.
    assert_eq!(c.members, vec![organizer], "{c:?}");
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.kind == civ_agents::ChronicleKind::Coalition && e.number == 0.0)
    );
    let words = civ_sim::frames::people::coalition_words(&sim, a);
    assert!(
        words.len() == 1 && words[0].contains("to found a settlement") && words[0].contains("km"),
        "{words:?}"
    );
    // Its second winning review: in the growing season, with what a band brought sown or eaten,
    // they hold too little seed or food to go yet.
    sim.people_mut_for_tests().review_due.insert(organizer);
    sim.advance_minutes(24 * 60).expect("lives");
    let c = sim
        .people()
        .coalitions
        .last()
        .cloned()
        .expect("still on record");
    assert_eq!(c.fate, CoalitionFate::Gathering);
    assert!(matches!(c.lacking, Lacking::Food | Lacking::Seed), "{c:?}");
    assert!(sim.people().chronicle.iter().any(|e| {
        e.kind == civ_agents::ChronicleKind::Coalition && (e.number == 1.0 || e.number == 2.0)
    }));
    // With grain enough laid by in its own and its kin's households, to eat and to sow, the
    // next review takes its kin on and sends them.
    let ours = sim
        .people()
        .household(organizer)
        .expect("lives")
        .members
        .clone();
    {
        let rules = sim.rules().clone();
        let crop = &rules.catalog.crops[rules.people.farm.crop];
        let pop = sim.people_mut_for_tests();
        for (_, x) in pop.households.iter_mut() {
            if x.id == organizer || kin_households.contains(&x.id) {
                for (good, kg) in [(crop.good, 2_000.0), (crop.seed_good, 200.0)] {
                    x.stores[good] += kg;
                    x.flows.add(civ_agents::person::Flow::Brought, good, kg);
                }
            }
        }
        pop.review_due.insert(organizer);
    }
    let kin_people: Vec<_> = kin_households
        .iter()
        .flat_map(|&h| sim.people().household(h).expect("lives").members.clone())
        .collect();
    sim.advance_minutes(24 * 60).expect("lives");
    let pop = sim.people();
    let c = pop.coalitions.last().cloned().expect("on record");
    assert_eq!(c.fate, CoalitionFate::Founded, "{c:?}");
    assert!(c.members.len() > 1, "kin's households join: {c:?}");
    let new = c.settlement.expect("its settlement");
    let s = sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id == new)
        .expect("written");
    assert_eq!(
        (s.parent, s.founding),
        (Some(a), civ_land::Founding::Coalition)
    );
    let went = |q: &civ_core::PermanentId| {
        pop.records[q]
            .residence
            .last()
            .is_some_and(|x| (x.settlement, x.why) == (Some(new), ResidenceWhy::Founded))
    };
    let going = pop.records.keys().filter(|q| went(q)).count();
    assert_eq!(c.people as usize, going);
    assert!(ours.iter().all(went));
    assert!(kin_people.iter().any(went));
    // Its polity lives under the body its founders knew, with no law of the parent's.
    let parent_body = pop
        .polities
        .iter()
        .find(|p| p.settlement == a)
        .expect("the parent's polity")
        .body;
    let daughter = pop
        .polities
        .iter()
        .find(|p| p.settlement == new)
        .expect("its own polity");
    assert_eq!(daughter.body, parent_body);
    assert!(daughter.laws.is_empty());
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == civ_agents::ChronicleKind::Coalition
                && e.number == 4.0
                && e.settlement == Some(new))
    );
    let now = sim.now().plus_minutes(1);
    for s in &sim.land().settlements {
        let acc = pop.accounts(s.id, SimTime::ZERO, now);
        assert!(acc.balance(), "{}: {acc:?}", s.name);
    }
    assert!(
        pop.problems(sim.ids().peek_next(), sim.rules().catalog.activities.len())
            .is_empty()
    );
    let words = civ_sim::frames::people::coalition_words(&sim, a);
    assert!(
        words
            .iter()
            .any(|w| w.contains(&format!("founded {}", s.name))),
        "{words:?}"
    );
    // Saved and loaded with a settlement founded, a world goes on alike (under the content's
    // rules, which a save keeps).
    {
        let rules = sim.rules_mut_for_tests().expect("rules of its own");
        let p = &mut rules.people;
        (p.moving.cost, p.founding.cost, p.decision.w_walk_hour) = content_rules;
    }
    saves_and_goes_on_alike(&mut sim, 24 * 60);
}

#[test]
fn a_refused_petition_has_its_organizer_weigh_leaving_together_with_its_faction() {
    // When the gathering turns down a faction's petition, its organizer's household weighs
    // leaving that midnight, and founds with its members (M5a slice AO, "leave together"; 10-01
    // §1.5): a household tied to it only by belonging to its faction is among those who would go,
    // while one tied to it by nothing is not. Founding is made worth something to every household
    // here (moving's cost set below nothing, the walk free), and the organizing household and the
    // one in its faction are made to forget whom they regard just before midnight (recruiting goes
    // on through those who join), so only kin and the faction decide who is asked.
    use civ_agents::faction::{Faction, Member, Petition};
    use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome, PolicyKind};
    use civ_agents::word::Blamed;
    use civ_core::PermanentId;
    let mut sim = world(3, 50, &[]).expect("generates");
    sim.advance_minutes(24 * 60).expect("lives");
    {
        let rules = sim.rules_mut_for_tests().expect("rules of its own");
        rules.people.moving.cost = -1.0;
        rules.people.founding.cost = 0.0;
        rules.people.decision.w_walk_hour = 0.0;
    }
    // To a minute before midnight, when the reviews are made.
    let to_midnight = 24 * 60 - sim.now().minute_of_day();
    sim.advance_minutes(to_midnight - 1).expect("lives");
    let a = sim.land().settlements[0].id;
    let hearth = sim.land().settlements[0].hearth_m;
    let (organizer, leader) = household_of(&sim, a);
    let day = sim.now().day_index();
    // Households with no close kin among the organizer's (parent, child or sibling).
    let pop = sim.people();
    let ours = pop.household(organizer).expect("lives").members.clone();
    let parents = |q: PermanentId| {
        pop.records
            .get(&q)
            .map_or([None, None], |r| [r.mother, r.father])
    };
    let kin = |p: PermanentId, q: PermanentId| {
        let (pp, pq) = (parents(p), parents(q));
        pp.contains(&Some(q))
            || pq.contains(&Some(p))
            || pp.iter().flatten().any(|x| pq.contains(&Some(*x)))
    };
    let unrelated = |them: &[PermanentId], to: &[PermanentId]| {
        them.iter().all(|&g| to.iter().all(|&m| !kin(g, m)))
    };
    let mut apart: Vec<_> = pop
        .households
        .iter()
        .filter(|(_, x)| x.id != organizer && x.settlement == Some(a) && !x.members.is_empty())
        .filter(|(_, x)| unrelated(&ours, &x.members))
        .map(|(_, x)| (x.id, x.members.clone()))
        .collect();
    apart.sort();
    let (joined, joined_members) = apart.first().cloned().expect("a household apart");
    let (other, _) = apart
        .iter()
        .skip(1)
        .find(|(_, m)| unrelated(&joined_members, m))
        .cloned()
        .expect("another household apart from both");
    let polity = pop.polities[0].id;
    // Dry, gentle land two field walks out, which the organizing household knows of.
    let reach = field_reach_m(&sim) as f32;
    let patches = &sim.land().patches;
    let site = (0..patches.class.len())
        .find(|&p| {
            let c = patches.centre_m(p);
            let d = (c.0 - hearth.0).hypot(c.1 - hearth.1);
            let cell = civ_agents::population::cell_of(sim.map(), c);
            d > 2.0 * reach + 100.0
                && d < 2.0 * reach + 1500.0
                && sim.map().water[cell] == civ_world::WATER_LAND
                && sim.nav().walkable(cell)
                && f64::from(civ_world::terrain::slope_at(sim.map(), cell))
                    <= sim.rules().people.band.site_max_slope
        })
        .expect("dry, gentle land two field walks out");
    // The organizer's member leads a faction against the gathering; one of `joined` belongs.
    let (grain, seed) = {
        let rules = sim.rules();
        let crop = &rules.catalog.crops[rules.people.farm.crop];
        (crop.good, crop.seed_good)
    };
    let store = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|p| p.kind == PolicyKind::CommonStore)
        .expect("in core") as u16;
    let now = sim.now();
    let (fid, law, asked) = (
        sim.allocate_id_for_tests(),
        sim.allocate_id_for_tests(),
        sim.allocate_id_for_tests(),
    );
    let pop = sim.people_mut_for_tests();
    pop.factions
        .list
        .push(Faction::found(fid, a, Blamed::Body(polity), leader, now));
    for p in [leader, joined_members[0]] {
        pop.factions.join(Member {
            person: p,
            faction: fid,
            since: day,
            why: Default::default(),
        });
    }
    for m in ours.iter().chain(&joined_members) {
        pop.ties.forget_holder(*m);
    }
    for (_, x) in pop.households.iter_mut() {
        if x.id == organizer {
            x.known.push(civ_agents::person::KnownPatch {
                resource: 0,
                patch: site as u32,
                rate: 1.0,
                hours: 1.0,
                seen_day: day,
            });
        }
        // Each holds grain enough to eat and to sow, so none would leave a coalition short.
        for (good, kg) in [(grain, 2_000.0), (seed, 200.0)] {
            x.stores[good] += kg;
            x.flows.add(civ_agents::person::Flow::Brought, good, kg);
        }
    }
    // Yesterday the faction petitioned the gathering, which has since turned it down.
    pop.polities[0].laws.push(Law {
        id: law,
        policy: store,
        kind: PolicyKind::CommonStore,
        levy_share: 0.0,
        holder: None,
        relief_days: 5.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::Rejected,
        sponsor: leader,
        proposed: now,
        issue: IssueKind::Petition,
        meets_day: day,
        decided: Some(now),
        outcome: Some(Outcome::Failed),
        eligible: 0,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
    });
    pop.factions.petitions.push(Petition {
        id: asked,
        faction: fid,
        settlement: a,
        organizer: leader,
        called: now,
        day: day - 1,
        policy: store,
        levy_share: 0.0,
        nominee: None,
        ends: law,
        came: vec![leader, joined_members[0]],
        law: Some(law),
        answered: false,
    });
    assert!(!pop.review_due.contains(&organizer));
    sim.advance_minutes(2).expect("lives");
    assert!(sim.people().factions.petitions[0].answered);
    let c = sim
        .people()
        .coalitions
        .iter()
        .find(|c| c.organizer == organizer)
        .cloned()
        .expect("it began to gather");
    assert!(c.members.contains(&joined), "{c:?}");
    assert!(!c.members.contains(&other), "{c:?}");
}

#[test]
fn a_household_going_with_anothers_coalition_gathers_none_of_its_own() {
    // Households counted among those going with a coalition go with it (M5a slice AO): at their
    // own reviews they weigh no plan of their own to found, so no second coalition of the same
    // households gathers beside the first. Founding is made worth something to every household
    // here, and every household knows the same site.
    let mut sim = world(3, 50, &[]).expect("generates");
    sim.advance_minutes(24 * 60).expect("lives");
    {
        let rules = sim.rules_mut_for_tests().expect("rules of its own");
        rules.people.moving.cost = -1.0;
        rules.people.founding.cost = 0.0;
        rules.people.decision.w_walk_hour = 0.0;
    }
    let to_midnight = 24 * 60 - sim.now().minute_of_day();
    sim.advance_minutes(to_midnight - 1).expect("lives");
    let a = sim.land().settlements[0].id;
    let hearth = sim.land().settlements[0].hearth_m;
    let (organizer, _) = household_of(&sim, a);
    let day = sim.now().day_index();
    let reach = field_reach_m(&sim) as f32;
    let patches = &sim.land().patches;
    let site = (0..patches.class.len())
        .find(|&p| {
            let c = patches.centre_m(p);
            let d = (c.0 - hearth.0).hypot(c.1 - hearth.1);
            let cell = civ_agents::population::cell_of(sim.map(), c);
            d > 2.0 * reach + 100.0
                && d < 2.0 * reach + 1500.0
                && sim.map().water[cell] == civ_world::WATER_LAND
                && sim.nav().walkable(cell)
                && f64::from(civ_world::terrain::slope_at(sim.map(), cell))
                    <= sim.rules().people.band.site_max_slope
        })
        .expect("dry, gentle land two field walks out");
    let (grain, seed) = {
        let rules = sim.rules();
        let crop = &rules.catalog.crops[rules.people.farm.crop];
        (crop.good, crop.seed_good)
    };
    let pop = sim.people_mut_for_tests();
    for (_, x) in pop.households.iter_mut() {
        x.known.push(civ_agents::person::KnownPatch {
            resource: 0,
            patch: site as u32,
            rate: 1.0,
            hours: 1.0,
            seen_day: day,
        });
        // Each holds grain enough to eat and to sow, so none would leave a coalition short.
        for (good, kg) in [(grain, 2_000.0), (seed, 200.0)] {
            x.stores[good] += kg;
            x.flows.add(civ_agents::person::Flow::Brought, good, kg);
        }
    }
    pop.review_due.insert(organizer);
    sim.advance_minutes(2).expect("lives");
    let first = sim
        .people()
        .coalitions
        .iter()
        .find(|c| c.organizer == organizer)
        .cloned()
        .expect("it began to gather");
    assert!(first.members.len() > 2, "{first:?}");
    // The next midnight, every other household going with it reviews.
    sim.advance_minutes(24 * 60 - 2).expect("lives");
    let pop = sim.people_mut_for_tests();
    for &h in first.members.iter().skip(1) {
        pop.review_due.insert(h);
    }
    sim.advance_minutes(2).expect("lives");
    let coalitions = &sim.people().coalitions;
    assert_eq!(coalitions.len(), 1, "{coalitions:?}");
}
