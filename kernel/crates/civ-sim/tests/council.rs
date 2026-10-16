//! Councils and their elections (M7a slice BO, ADR-0024 §1, §3): a custom taken to a council of
//! seats calls its election at the next midnight; word of it goes round; those of the electorate
//! who hear of it choose whether to stand; a contested field is polled at the hearth and counted
//! by the block vote; those chosen sit, and only they are the body; the term's end calls the next
//! election; the label reads the selection chain; and all of it saves and loads exactly.

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
    while sim.now().day_index() <= called + stand_days + 2 {
        sim.advance_minutes(DAY).expect("advances");
    }
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
