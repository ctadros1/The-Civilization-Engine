//! Relations between polities (ADR-0020; plan §7, M5c): the places people work and the outsiders
//! they see there, the issue that makes for their polity, and a claim on wild ground by law.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::polity::{IssueKind, LawStatus, PolicyKind};
use civ_agents::uses::{Place, Worked};
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

#[test]
fn outsiders_seen_at_a_place_make_an_issue_and_a_gathering_decides_a_claim_on_it() {
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
            });
        }
    }
    // Within a few weeks one village's gathering has decided a claim, its issue the outsiders.
    let mut decided = None;
    for _ in 0..40 {
        sim.advance_minutes(DAY).expect("lives");
        decided = sim.people().polities.iter().find_map(|p| {
            p.laws
                .iter()
                .find(|l| l.kind == PolicyKind::ClaimPlace && l.outcome.is_some())
                .map(|l| (p.id, l.clone()))
        });
        if decided.is_some() {
            break;
        }
    }
    let (polity_id, law) = decided.expect("a gathering decided a claim");
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
    } else {
        assert!(polity.claims_now().is_empty());
    }
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
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
