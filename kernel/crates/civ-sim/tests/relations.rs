//! Relations between polities (ADR-0020; plan §7, M5c): the places people work and the outsiders
//! they see there, the issue that makes for their polity, and a claim on wild ground by law.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::polity::{IssueKind, Law, LawStatus, PolicyKind};
use civ_agents::uses::{Place, Worked};
use civ_agents::views::{Domain, ViewAct};
use civ_agents::word::{Blamed, Wrong};
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

/// Two settlements of world `seed`, with a fixed identity so it lives the same life every run.
fn two_villages(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Neighbours".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: vec![30],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

/// The households living in settlement `s`, in id order.
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

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "uses").expect("saves");
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

/// A fishing place both villages of world 5 worked for sixty days, and the first claim a
/// gathering decided after it: the world, the claiming polity's id, the law and the place.
fn a_claim_decided() -> (Sim, PermanentId, Law, Place) {
    let mut sim = two_villages(5);
    sim.advance_minutes(DAY).expect("lives");
    let settlements: Vec<PermanentId> = sim.land().settlements.iter().map(|s| s.id).collect();
    assert_eq!(settlements.len(), 2);
    let (home, away) = (settlements[0], settlements[1]);
    let (ours, theirs) = (households_of(&sim, home), households_of(&sim, away));
    assert!(!ours.is_empty() && !theirs.is_empty());
    // For the past sixty days people of both villages fished one stretch of water; the work is
    // logged as though done, and folded in at the next midnight.
    let place = Place::Patch(4_242);
    let today = sim.now().day_index();
    let pop = sim.people_mut_for_tests();
    for day in today - 60..today {
        for (&h, s) in ours
            .iter()
            .map(|h| (h, home))
            .chain(theirs.iter().map(|h| (h, away)))
        {
            pop.uses.worked(Worked {
                place,
                day,
                household: h,
                settlement: s,
                kcal: 2_500.0,
                person: None,
            });
        }
    }
    // Within a few weeks one village's gathering has decided a claim, its issue the outsiders.
    for _ in 0..40 {
        sim.advance_minutes(DAY).expect("lives");
        let decided = sim.people().polities.iter().find_map(|p| {
            p.laws
                .iter()
                .find(|l| l.kind == PolicyKind::ClaimPlace && l.outcome.is_some())
                .map(|l| (p.id, l.clone()))
        });
        if let Some((polity, law)) = decided {
            return (sim, polity, law, place);
        }
    }
    panic!("no gathering decided a claim");
}

#[test]
fn outsiders_seen_at_a_place_make_an_issue_and_a_gathering_decides_a_claim_on_it() {
    let (mut sim, polity_id, law, place) = a_claim_decided();
    assert_eq!(law.issue, IssueKind::Outsiders);
    let pop = sim.people();
    let polity = pop
        .polities
        .iter()
        .find(|p| p.id == polity_id)
        .expect("its polity");
    // The law names the place, whatever its gathering decided.
    assert!(
        polity
            .claimed
            .iter()
            .any(|&(l, p)| l == law.id && p == place),
        "{:?}",
        polity.claimed
    );
    // Its people saw outsiders there and fished it themselves.
    let mine = households_of(&sim, polity.settlement);
    assert!(mine.iter().any(|&h| {
        pop.uses.households.get(&h).is_some_and(|list| {
            list.iter()
                .any(|u| u.place == place && !u.outsiders.is_empty())
        })
    }));
    if law.status == LawStatus::InForce {
        assert_eq!(polity.claims_now(), vec![place]);
        assert_eq!(polity.claim_on(place), Some(law.id));
    } else {
        assert!(polity.claims_now().is_empty());
    }
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
    // All of it saves and loads exactly, and goes on alike.
    saves_and_goes_on_alike(&mut sim, 3 * DAY);
}

#[test]
fn people_who_see_outsiders_work_a_claimed_place_hold_it_against_them_and_think_worse_of_their_polity()
 {
    let (mut sim, polity_id, law, place) = a_claim_decided();
    assert_eq!(
        law.status,
        LawStatus::InForce,
        "world 5's gathering passed it"
    );
    let (ours, theirs_polity) = {
        let pop = sim.people();
        let ours = pop
            .polities
            .iter()
            .find(|p| p.id == polity_id)
            .expect("ours");
        let other = pop
            .polities
            .iter()
            .find(|p| p.id != polity_id)
            .expect("theirs");
        (ours.settlement, (other.id, other.settlement))
    };
    let (their_polity, theirs) = theirs_polity;
    let pop = sim.people();
    // One of ours who knows the claim, and one of theirs, of households of their own.
    let adult = |s: PermanentId, knows: bool| {
        households_of(&sim, s).into_iter().find_map(|h| {
            let x = pop.household(h)?;
            x.members
                .iter()
                .copied()
                .find(|&m| {
                    pop.person(m)
                        .is_some_and(|p| p.age_years(sim.now()) >= 16.0)
                        && (!knows || law.knows(m))
                })
                .map(|m| (h, m))
        })
    };
    let (our_household, witness) = adult(ours, true).expect("one of ours knows the claim");
    let (their_household, outsider) = adult(theirs, false).expect("an adult of theirs");
    assert!(pop.word.grievances_of(witness).next().is_none());
    // Today both fish the claimed place; at midnight it is folded in.
    let fish_today = |sim: &mut Sim| {
        let today = sim.now().day_index();
        let pop = sim.people_mut_for_tests();
        for (household, settlement, person) in [
            (our_household, ours, witness),
            (their_household, theirs, outsider),
        ] {
            pop.uses.worked(Worked {
                place,
                day: today,
                household,
                settlement,
                kcal: 3_000.0,
                person: Some(person),
            });
        }
    };
    fish_today(&mut sim);
    sim.advance_minutes(DAY).expect("lives");
    let pop = sim.people();
    let rp = &sim.rules().people.relations;
    // The witness holds it against the outsiders' household, under the claim.
    let g = pop
        .word
        .grievances_of(witness)
        .find(|g| g.wrong == Wrong::Trespass)
        .expect("a grievance for the trespass");
    assert_eq!(g.blamed, Blamed::Household(their_household));
    assert_eq!(g.law, law.id);
    assert!(g.harm_days > 0.0);
    // And takes it that the other polity harms theirs; nothing else of their view moved.
    let v = pop
        .polity_views
        .of(witness, their_polity, sim.now().day_index(), rp)
        .expect("a view of the other polity");
    assert!(v.lean(Domain::HarmsUs) > 0.5, "{v:?}");
    assert!((v.lean(Domain::HelpsUs) - 0.5).abs() < 1e-9);
    assert!((v.lean(Domain::KeepsWord) - 0.5).abs() < 1e-9);
    assert_eq!(v.reason.map(|r| r.act), Some(ViewAct::SawTrespass));
    // The outsider is bound by nothing; they hold it against the witness's household only if
    // their own polity claims the place too, under its own claim.
    let their_claim = pop
        .polities
        .iter()
        .find(|p| p.id == their_polity)
        .and_then(|p| p.claim_on(place));
    for g in pop
        .word
        .grievances_of(outsider)
        .filter(|g| g.wrong == Wrong::Trespass)
    {
        assert_eq!(Some(g.law), their_claim);
        assert_eq!(g.blamed, Blamed::Household(our_household));
    }
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
    // A day's fishing is a small harm, not worth telling; days of it on end are, and word of it
    // goes round the hearth: within a month someone who was not there has heard.
    let mut heard = false;
    for day in 0..30 {
        if day < 15 {
            fish_today(&mut sim);
        }
        sim.advance_minutes(DAY).expect("lives");
        heard = sim.people().polity_views.held.iter().any(|(&p, list)| {
            p != witness
                && list.iter().any(|v| {
                    v.polity == their_polity
                        && v.reason.is_some_and(|r| r.act == ViewAct::HeardTrespass)
                })
        });
        if heard {
            break;
        }
    }
    assert!(heard, "nobody heard of the trespass");
    // All of it saves and loads exactly, and goes on alike.
    saves_and_goes_on_alike(&mut sim, 3 * DAY);
}

#[test]
fn a_village_alone_holds_its_places_but_sees_no_outsiders() {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Alone".to_owned(),
            seed: 5,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates");
    sim.advance_minutes(20 * DAY).expect("lives");
    let pop = sim.people();
    assert!(!pop.uses.households.is_empty(), "its people work places");
    assert!(
        pop.uses
            .households
            .values()
            .flatten()
            .all(|u| u.outsiders.is_empty())
    );
    assert!(pop.polities.iter().all(|p| p.claimed.is_empty()));
}
