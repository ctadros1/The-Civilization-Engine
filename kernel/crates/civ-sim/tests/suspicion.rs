//! Suspicion of a source of water (M6a slice BA, step two; ADR-0021 §6; research 12-02 §4): each
//! person tallies their own records of who draws where and which households had sickness; one
//! whose known households at a well fell sick, against few elsewhere, holds that it sickens and
//! says so with the counts, which word carries; with the news gone the suspicion is let go; and
//! suspicions, their claims and the households seen drawing water save and load exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::uses::{Place, PlaceUse, SeenAt};
use civ_agents::word::{Claim, ClaimKind, Counts};
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

/// A village of about sixty, with a fixed identity so it lives the same life every run.
fn village(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Wellmere".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: 60,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [11; 16],
    )
    .expect("generates")
}

fn to_morning(sim: &mut Sim, minute: i64) {
    let rest = DAY - sim.now().minutes().rem_euclid(DAY) + minute;
    sim.advance_minutes(rest).expect("advances");
}

fn use_of(place: Place, day: i64, seen: &[PermanentId]) -> PlaceUse {
    PlaceUse {
        place,
        days: 10.0,
        kcal: 0.0,
        outsiders: Vec::new(),
        seen: seen
            .iter()
            .map(|&household| SeenAt {
                household,
                days: 10.0,
                last: day,
            })
            .collect(),
        day,
    }
}

#[test]
fn someone_suspects_the_well_whose_households_fell_sick_and_word_of_it_goes_round() {
    let mut sim = village(3);
    to_morning(&mut sim, 9 * 60);
    let today = sim.now().day_index();
    let well = Place::Source(7);
    let river = Place::Source(9);
    // An adult of one household, four households seen at their well, six known through ties
    // drawing at the river; three at the well had sickness, none at the river.
    let (person, home, at_well, at_river) = {
        let pop = sim.people();
        let grown = sim.rules().people.family.independent_age;
        let mut households: Vec<PermanentId> = pop.households.iter().map(|(_, x)| x.id).collect();
        households.sort_unstable();
        assert!(households.len() >= 11, "{} households", households.len());
        let home = households[0];
        let person = pop
            .household(home)
            .expect("a household")
            .members
            .iter()
            .copied()
            .find(|&m| {
                pop.person(m)
                    .is_some_and(|p| p.age_years(sim.now()) >= grown)
            })
            .expect("an adult");
        (
            person,
            home,
            households[1..5].to_vec(),
            households[5..11].to_vec(),
        )
    };
    let settlement = sim
        .people()
        .household(home)
        .and_then(|x| x.settlement)
        .expect("settled");
    {
        let ties = sim.rules().people.ties.clone();
        let gift = civ_agents::ties::Act::from_name("gift_received").expect("an act");
        let pop = sim.people_mut_for_tests();
        pop.uses.households.clear();
        pop.uses.today.clear();
        pop.uses
            .households
            .insert(home, vec![use_of(well, today, &at_well)]);
        for &h in &at_river {
            pop.uses
                .households
                .insert(h, vec![use_of(river, today, &[])]);
            let member = pop.household(h).expect("a household").members[0];
            pop.ties
                .record(person, member, gift, 5.0, 0.0, today, &ties);
        }
        for &h in &at_well[..3] {
            let c = pop.word.make(Claim {
                id: 0,
                kind: ClaimKind::Sickness,
                settlement,
                day: today,
                subject: Some(h),
                grievance: None,
                suspected: None,
            });
            pop.word.hear(person, c, today, None, None);
        }
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.suspicion_day_for_tests(ctx));
    let pop = sim.people();
    let counts = Counts {
        sick_at: 3,
        at: 5,
        sick_elsewhere: 0,
        elsewhere: 6,
    };
    let held = pop.word.suspicions_of(person);
    assert_eq!(held.len(), 1, "{held:?}");
    assert_eq!(
        (held[0].source, held[0].since, held[0].counts),
        (well, today, counts)
    );
    // Nobody else holds one: nobody else heard of the sickness.
    assert_eq!(pop.word.suspicions.len(), 1);
    // They say so, with the counts, at first hand.
    let claim = *pop
        .word
        .claims
        .iter()
        .find(|c| c.kind == ClaimKind::Suspicion)
        .expect("said");
    assert_eq!(
        (claim.subject, claim.suspected),
        (Some(person), Some((well, counts)))
    );
    let h = pop
        .word
        .heard_by(person)
        .iter()
        .find(|h| h.claim == claim.id)
        .expect("theirs");
    assert_eq!((h.from, h.origin), (None, Some(person)));
    let words = civ_sim::frames::word::claim_words(&sim, &claim);
    assert!(
        words.contains("holds that the water at ")
            && words.ends_with(
                "sickens: of 5 households they know that draw there, 3 had sickness lately, \
                 against 0 of 6 that draw elsewhere"
            ),
        "{words}"
    );
    // It saves and loads exactly, with the households seen drawing water.
    reloaded(&mut sim);
    // Word of it goes round.
    for _ in 0..6 {
        to_morning(&mut sim, 0);
    }
    let told = sim
        .people()
        .word
        .heard
        .iter()
        .filter(|h| h.claim == claim.id && h.holder != person)
        .count();
    assert!(told > 0, "word of the suspicion went round");
    // With no news of sickness left, their next tally lets it go.
    {
        let rules = sim.rules_mut_for_tests().expect("the rules are not shared");
        rules.people.suspicion.review_days = 1;
    }
    {
        let pop = sim.people_mut_for_tests();
        let sick: Vec<u32> = pop
            .word
            .claims
            .iter()
            .filter(|c| c.kind == ClaimKind::Sickness)
            .map(|c| c.id)
            .collect();
        pop.word
            .heard
            .retain(|h| !(h.holder == person && sick.contains(&h.claim)));
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.suspicion_day_for_tests(ctx));
    assert!(sim.people().word.suspicions_of(person).is_empty());
}

#[test]
fn a_world_without_sickness_holds_no_suspicion_and_households_know_who_draws_beside_them() {
    let mut sim = village(3);
    to_morning(&mut sim, 0);
    sim.advance_minutes(10 * DAY).expect("lives");
    let pop = sim.people();
    assert!(pop.word.suspicions.is_empty());
    assert!(
        !pop.word
            .claims
            .iter()
            .any(|c| c.kind == ClaimKind::Suspicion)
    );
    // Households drawing at the same edge of the water the same day saw each other there.
    let seen = pop
        .uses
        .households
        .values()
        .flatten()
        .filter(|u| matches!(u.place, Place::Source(_)))
        .map(|u| u.seen.len())
        .sum::<usize>();
    assert!(seen > 0, "nobody saw anyone drawing water beside them");
    assert!(pop.uses.problems().is_empty());
}

/// A save of `sim` loaded again, checked to be the same section for section.
fn reloaded(sim: &mut Sim) -> Sim {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "suspect").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let (a, b) = (
        persist::encode_sections(sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
    assert_eq!(sim.people().word, loaded.people().word);
    assert_eq!(sim.people().uses, loaded.people().uses);
    loaded
}
