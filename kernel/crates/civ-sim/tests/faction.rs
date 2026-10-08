//! Factions (plan §7 M4c slice AH, step one; ADR-0017 §2; research 04-10 §1.1–§1.5, §5.1–§5.4):
//! someone who feels a grievance against the gathering keenly, and has heard that others they
//! trust hold one too, founds a faction over it; others who know its members join it for their
//! own reasons, which the record keeps; a member's household short of food is given from its
//! store; all of it saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::ledger::Channel;
use civ_agents::ties::Tie;
use civ_agents::word::{Blamed, Claim, ClaimKind, Grievance, Grieved, Wrong};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
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
            name: "Factions".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 512,
            band_size: 0,
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
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "factions").expect("saves");
    let mut loaded = persist::load(&saved.path, content).expect("loads");
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        let differ: Vec<String> = a
            .iter()
            .zip(&b)
            .filter(|(x, y)| x.bytes != y.bytes)
            .map(|(x, _)| x.tag.to_string())
            .collect();
        assert!(differ.is_empty(), "sections differ: {differ:?}");
    };
    same(sim, &loaded);
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}

/// A village whose adults all hold a keen grievance against the gathering over a lean year's
/// levy, have each heard that every other holds one, and know and regard one another well.
fn aggrieved_village(seed: u64) -> (Sim, Vec<PermanentId>) {
    let mut sim = world_with(content(), seed);
    sim.advance_minutes(40 * DAY).expect("advances");
    let now = sim.now();
    let day = now.day_index();
    let params = sim.rules().people.clone();
    let pop = sim.people_mut_for_tests();
    let polity = pop.polities[0].id;
    let settlement = pop.polities[0].settlement;
    let mut adults: Vec<PermanentId> = pop
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) >= params.family.independent_age)
        .map(|p| p.id)
        .collect();
    adults.sort_unstable();
    assert!(adults.len() >= 4, "a band of several adults");
    let against = Blamed::Body(polity);
    for &a in &adults {
        pop.word.grievances.push(Grievance {
            holder: a,
            issue: Grieved::Extraction,
            blamed: against,
            law: polity,
            harm_days: 10.0,
            unresolved_days: 10.0,
            activation: 1.0,
            raised: day,
            made: day,
            wrong: Wrong::LeanLevy,
        });
        let claim = pop.word.make(Claim {
            id: 0,
            kind: ClaimKind::Grievance,
            settlement,
            day,
            subject: Some(a),
            grievance: Some((against, Grieved::Extraction)),
        });
        for &b in &adults {
            pop.word.hear(b, claim, day, (a != b).then_some(a), Some(a));
        }
        let ties: Vec<Tie> = adults
            .iter()
            .filter(|&&b| b != a)
            .map(|&b| Tie {
                familiarity: 1.0,
                warmth: 1.0,
                ..Tie::new(b, day, &params.ties)
            })
            .collect();
        pop.ties.restore(a, ties);
    }
    (sim, adults)
}

#[test]
fn a_shared_grievance_founds_a_faction_and_those_who_know_its_members_join_for_their_reasons() {
    let (mut sim, adults) = aggrieved_village(5);
    // Everyone reviews once in the month.
    sim.advance_minutes(31 * DAY).expect("advances");
    let pop = sim.people();
    let polity = pop.polities[0].id;
    let live: Vec<_> = pop.factions.list.iter().filter(|f| f.is_live()).collect();
    assert_eq!(live.len(), 1, "one faction, the rest join it: {live:?}");
    let f = live[0];
    assert_eq!(f.against, Blamed::Body(polity));
    assert!(adults.contains(&f.founder));
    let members: Vec<_> = pop.factions.members_of(f.id).collect();
    assert!(members.len() >= 2, "others joined: {members:?}");
    for m in &members {
        // Each joined for reasons of their own, and the record says so.
        assert!(m.why.worth() >= f64::from(m.why.threshold) - 0.5, "{m:?}");
        assert!(m.why.grievance > 0.0, "{m:?}");
        let words =
            civ_agents::faction::why_words(&m.why, "the gathering", "", m.person == f.organizer);
        assert!(
            words.contains("holding a grievance against the gathering"),
            "{words}"
        );
    }
    // Some held the grievance and did not join: belief is not membership (04-10 §5.1).
    assert!(pop.factions.joined >= members.len() as u64);
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn a_members_household_short_of_food_is_given_from_its_store() {
    let (mut sim, _) = aggrieved_village(6);
    sim.advance_minutes(31 * DAY).expect("advances");
    let grain = sim
        .rules()
        .catalog
        .good_index("core:good/grain")
        .expect("in core");
    let pop = sim.people_mut_for_tests();
    let (fid, member) = {
        let f = pop
            .factions
            .list
            .iter()
            .find(|f| f.is_live())
            .expect("a faction was founded");
        let m = pop
            .factions
            .members_of(f.id)
            .find(|m| m.person != f.organizer)
            .or_else(|| pop.factions.members_of(f.id).next())
            .expect("members")
            .person;
        (f.id, m)
    };
    let hh = pop.person(member).expect("here").household;
    // The faction's store holds grain; nobody else has any food to give.
    let fi = pop
        .factions
        .list
        .iter()
        .position(|f| f.id == fid)
        .expect("kept");
    pop.factions.list[fi].stores.resize(grain + 1, 0.0);
    pop.factions.list[fi].stores[grain] = 200.0;
    for (_, x) in pop.households.iter_mut() {
        for kg in &mut x.stores {
            *kg = 0.0;
        }
    }
    let given_before = pop.transfers.get(Channel::Gift, grain);
    sim.advance_minutes(2 * DAY).expect("advances");
    let pop = sim.people();
    let f = pop.factions.get(fid).expect("kept");
    assert!(
        f.stores.get(grain).copied().unwrap_or(0.0) < 200.0,
        "the store gave: {:?}",
        f.stores
    );
    assert!(pop.transfers.get(Channel::Gift, grain) > given_before);
    // Its member's household holds food again.
    let held = pop
        .household(hh)
        .map_or(0.0, |x| x.stores.get(grain).copied().unwrap_or(0.0));
    assert!(held > 0.0, "the member's household was given grain");
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}
