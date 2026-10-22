//! A resolution per settlement (M9a slice DF, ADR-0032 §1, §3): a settlement switched to
//! household resolution has its people parked at a midnight and lives its day then; the switch
//! changes nobody, no holding and no good the books do not account for; a save of such a world
//! lives on as if never saved once the setting is made again; and a settlement brought back takes
//! up its day.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use civ_agents::person::Flow;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_core::time::MINUTES_PER_DAY;
use civ_sim::persist::{self, SECTION_META};
use civ_sim::{NewWorld, Sim};
use commons_persist::{SaveDir, SaveKind, SectionData};

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

/// Two founding groups on a river valley with a fixed identity, lived ten days to a midnight.
fn two_settlements() -> Sim {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Resolutions".to_owned(),
            seed: 3,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: 0,
            neighbours: vec![0],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates");
    assert_eq!(sim.land().settlements.len(), 2);
    sim.advance_minutes(10 * MINUTES_PER_DAY).expect("advances");
    sim.advance_to_midnight().expect("advances");
    sim
}

/// The second settlement's id.
fn second(sim: &Sim) -> PermanentId {
    sim.land().settlements[1].id
}

/// The people living in settlement `s`.
fn residents(sim: &Sim, s: PermanentId) -> Vec<PermanentId> {
    let people = sim.people();
    people
        .people
        .iter()
        .filter(|(_, p)| {
            people
                .household(p.household)
                .and_then(|x| x.settlement)
                .is_some_and(|x| x == s)
        })
        .map(|(_, p)| p.id)
        .collect()
}

fn save(sim: &mut Sim, dir: &Path, label: &str) -> PathBuf {
    let saves = SaveDir::create(dir, civ_schema::SAVE_EXTENSION).expect("save dir");
    persist::save(sim, &saves, SaveKind::Manual, label)
        .expect("saves")
        .path
}

fn load(path: &Path) -> Sim {
    persist::load(path, content()).expect("loads")
}

fn differing(a: &[SectionData], b: &[SectionData]) -> Vec<String> {
    if a.len() != b.len() {
        return vec![format!("{} sections against {}", a.len(), b.len())];
    }
    a.iter()
        .zip(b)
        .filter(|(x, y)| x.tag != SECTION_META && (x.tag != y.tag || x.bytes != y.bytes))
        .map(|(x, _)| format!("{} {}", x.tag, x.index))
        .collect()
}

fn assert_same(what: &str, a: &Sim, b: &Sim) {
    let d = differing(&persist::encode_sections(a), &persist::encode_sections(b));
    assert!(d.is_empty(), "{what}: sections differ: {d:?}");
}

#[test]
fn a_switch_undone_before_it_takes_effect_changes_nothing() {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let mut sim = two_settlements();
    let start = save(&mut sim, dir.path(), "start");
    let mut straight = load(&start);
    straight
        .advance_minutes(6 * MINUTES_PER_DAY)
        .expect("advances");
    // Set and undone at a midnight, then held and undone between midnights.
    let mut undone = load(&start);
    let s = second(&undone);
    undone.set_household_resolved([s]);
    undone.set_household_resolved([]);
    undone
        .advance_minutes(MINUTES_PER_DAY / 2)
        .expect("advances");
    undone.set_household_resolved([s]);
    undone.set_household_resolved([]);
    undone
        .advance_minutes(6 * MINUTES_PER_DAY - MINUTES_PER_DAY / 2)
        .expect("advances");
    assert!(undone.household_resolved().is_empty());
    assert_same("a switch undone", &straight, &undone);
}

#[test]
fn a_household_resolved_settlement_is_parked_and_lives_its_needs_at_midnight() {
    let mut sim = two_settlements();
    let s = second(&sim);
    let first = sim.land().settlements[0].id;
    // Held between midnights: nothing changes until the next one.
    sim.advance_minutes(MINUTES_PER_DAY / 3).expect("advances");
    sim.set_household_resolved([s]);
    assert!(sim.household_resolved().is_empty(), "held for midnight");
    sim.advance_to_midnight().expect("advances");
    assert_eq!(sim.household_resolved(), &BTreeSet::from([s]));
    let eaten = |sim: &Sim| {
        sim.people()
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(s))
            .map(|(_, x)| {
                (0..x.stores.len())
                    .map(|g| x.flows.get(Flow::Eaten, g).abs())
                    .sum::<f64>()
            })
            .sum::<f64>()
    };
    let before = eaten(&sim);
    sim.advance_minutes(3 * MINUTES_PER_DAY + 600)
        .expect("advances");
    assert!(sim.people().resolution_problems().is_empty());
    let parked = residents(&sim, s);
    assert!(!parked.is_empty());
    for id in &parked {
        let p = sim.people().person(*id).expect("lives");
        assert!(p.act.steps.is_empty(), "{} is parked", p.given);
        let home = sim.people().household(p.household).expect("household").home;
        assert_eq!(p.pos, home, "{} stands at home", p.given);
        assert!(p.trip.is_none());
    }
    assert!(
        eaten(&sim) > before,
        "they eat from their stores at midnight"
    );
    // The other settlement lives through the day as ever.
    assert!(residents(&sim, first).iter().all(|id| {
        !sim.people()
            .person(*id)
            .expect("lives")
            .act
            .steps
            .is_empty()
    }));
}

#[test]
fn parking_changes_nobody_no_holding_and_no_good_beyond_the_books() {
    let mut sim = two_settlements();
    // Mid-morning, when people are out with loads.
    sim.advance_minutes(10 * 60).expect("advances");
    let s = second(&sim);
    let before = (
        sim.people().switch_invariants(sim.land()),
        sim.people().goods_held(),
        sim.people().flows(),
    );
    sim.with_ctx_for_tests(|people, ctx| {
        people.set_household_resolved(BTreeSet::from([s]));
        people.park_resolved(ctx);
    });
    let after = (
        sim.people().switch_invariants(sim.land()),
        sim.people().goods_held(),
        sim.people().flows(),
    );
    assert_eq!(before.0, after.0, "nobody and no holding changed");
    let gaps = civ_agents::population::unaccounted(
        before.1.len().max(after.1.len()),
        (&before.1, &before.2),
        (&after.1, &after.2),
    );
    assert!(gaps.is_empty(), "the books balance: {gaps:?}");
    assert!(
        residents(&sim, s)
            .iter()
            .all(|id| sim.people().person(*id).expect("lives").carrying.kg == 0.0),
        "what they carried is banked"
    );
}

#[test]
fn a_save_of_a_household_resolved_world_lives_on_as_if_never_saved() {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let mut sim = two_settlements();
    let s = second(&sim);
    sim.set_household_resolved([s]);
    sim.advance_minutes(3 * MINUTES_PER_DAY).expect("advances");
    let mid = save(&mut sim, dir.path(), "resolved");
    sim.advance_minutes(5 * MINUTES_PER_DAY).expect("advances");
    let mut resumed = load(&mid);
    assert!(resumed.household_resolved().is_empty(), "never saved");
    resumed.set_household_resolved([s]);
    resumed
        .advance_minutes(5 * MINUTES_PER_DAY)
        .expect("advances");
    assert_same("saved household-resolved and set again", &sim, &resumed);
}

#[test]
fn a_loaded_world_not_set_again_takes_up_its_day_at_once() {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let mut sim = two_settlements();
    let s = second(&sim);
    sim.set_household_resolved([s]);
    sim.advance_minutes(2 * MINUTES_PER_DAY).expect("advances");
    let mid = save(&mut sim, dir.path(), "resolved");
    let mut loaded = load(&mid);
    loaded.advance_minutes(1).expect("advances");
    assert!(loaded.people().resolution_problems().is_empty());
    assert!(
        residents(&loaded, s).iter().all(|id| !loaded
            .people()
            .person(*id)
            .expect("lives")
            .act
            .steps
            .is_empty()),
        "everyone decided"
    );
}

#[test]
fn a_settlement_brought_back_takes_up_its_day() {
    let mut sim = two_settlements();
    let s = second(&sim);
    sim.set_household_resolved([s]);
    sim.advance_minutes(3 * MINUTES_PER_DAY).expect("advances");
    let at_home: Vec<_> = residents(&sim, s)
        .iter()
        .map(|id| sim.people().person(*id).expect("lives").pos)
        .collect();
    sim.set_household_resolved([]);
    sim.advance_minutes(MINUTES_PER_DAY + 13 * 60)
        .expect("advances");
    assert!(sim.household_resolved().is_empty());
    assert!(sim.people().resolution_problems().is_empty());
    let now: Vec<_> = residents(&sim, s)
        .iter()
        .map(|id| sim.people().person(*id).expect("lives").pos)
        .collect();
    assert_ne!(at_home, now, "they are about their day again");
}

#[test]
fn switching_back_and_forth_keeps_everyone_and_their_holdings() {
    // The kernel checks every switch itself with debug checks on (the suite's profile); here a
    // settlement is switched every few days and its people are all still there, each household
    // where it was.
    let mut sim = two_settlements();
    let s = second(&sim);
    let people_before = residents(&sim, s).len();
    for i in 0..6 {
        let set: Vec<_> = if i % 2 == 0 { vec![s] } else { Vec::new() };
        sim.set_household_resolved(set);
        sim.advance_minutes(2 * MINUTES_PER_DAY).expect("advances");
        assert!(sim.people().resolution_problems().is_empty(), "switch {i}");
    }
    let people_after = residents(&sim, s).len();
    assert!(
        people_after + 2 >= people_before,
        "nobody lost to a switch: {people_before} then {people_after}"
    );
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
}

/// Hours of work done on the fields of settlement `s`'s households, and how many they hold.
fn field_work(sim: &Sim, s: PermanentId) -> (f64, usize) {
    let people = sim.people();
    let ours = |h: PermanentId| {
        people
            .household(h)
            .and_then(|x| x.settlement)
            .is_some_and(|x| x == s)
    };
    let fields: Vec<_> = sim
        .land()
        .fields
        .iter()
        .filter(|f| ours(f.household))
        .collect();
    (
        fields
            .iter()
            .map(|f| f64::from(f.work_h) + f64::from(f.tended_h))
            .sum(),
        fields.len(),
    )
}

/// Ties held by residents of settlement `s`.
fn ties_of(sim: &Sim, s: PermanentId) -> usize {
    residents(sim, s)
        .iter()
        .map(|id| sim.people().ties.of(*id).len())
        .sum()
}

#[test]
fn a_household_resolved_settlement_works_and_keeps_company_through_its_days() {
    // Slice DG (ADR-0032 §2): each household's day is scored once at midnight and its members'
    // hours shared among what they would choose, each lived through its own effects.
    let mut sim = two_settlements();
    let s = second(&sim);
    sim.set_household_resolved([s]);
    sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    let (work_before, fields_before) = field_work(&sim, s);
    let ties_before = ties_of(&sim, s);
    let held_before = (sim.people().goods_held(), sim.people().flows());
    sim.advance_minutes(20 * MINUTES_PER_DAY).expect("advances");
    let (work_after, fields_after) = field_work(&sim, s);
    assert!(
        work_after > work_before || fields_after > fields_before,
        "their fields are worked: {work_before:.1} h on {fields_before} fields, then \
         {work_after:.1} h on {fields_after}"
    );
    assert!(
        ties_of(&sim, s) > ties_before,
        "they keep company at the hearth and come to know one another"
    );
    assert!(sim.people().resolution_problems().is_empty());
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
    let gaps = civ_agents::population::unaccounted(
        held_before.0.len().max(sim.people().goods_held().len()),
        (&held_before.0, &held_before.1),
        (&sim.people().goods_held(), &sim.people().flows()),
    );
    assert!(gaps.is_empty(), "the books balance: {gaps:?}");
    // Everyone is fed: nobody of the settlement died of hunger in three weeks.
    assert!(residents(&sim, s).len() * 10 >= 9 * residents(&two_settlements(), s).len());
}

#[test]
fn household_resolved_days_are_the_same_however_the_time_is_cut() {
    let mut sim = two_settlements();
    let s = second(&sim);
    sim.set_household_resolved([s]);
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let start = save(&mut sim, dir.path(), "start");
    let mut whole = load(&start);
    whole.set_household_resolved([s]);
    whole
        .advance_minutes(8 * MINUTES_PER_DAY)
        .expect("advances");
    let mut cut = load(&start);
    cut.set_household_resolved([s]);
    let (total, mut done) = (8 * MINUTES_PER_DAY, 0);
    while done < total {
        let step = 433.min(total - done);
        cut.advance_minutes(step).expect("advances");
        done += step;
    }
    assert_same("eight days cut every 433 minutes", &whole, &cut);
}

#[test]
fn a_burning_settlement_is_held_detailed_until_its_fire_is_out() {
    // ADR-0032 §1: nobody fights a fire in a household-resolved settlement, so one where a fire
    // burns lives Detailed until it is out.
    let mut sim = two_settlements();
    // Lived on until a home is roofed; the settlement it stands in is the one resolved.
    let mut found = None;
    for _ in 0..60 {
        found = sim.land().buildings.iter().find(|b| b.roofed()).map(|b| {
            let s = sim
                .people()
                .household(b.household)
                .and_then(|x| x.settlement)
                .expect("a settlement");
            (b.id, s)
        });
        if found.is_some() {
            break;
        }
        sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    }
    let (building, s) = found.expect("a home is roofed within two months");
    sim.set_household_resolved([s]);
    sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    // Set alight ten minutes before midnight: it still burns when the day turns.
    sim.advance_minutes(MINUTES_PER_DAY - 10).expect("advances");
    sim.set_fire(building).expect("set alight");
    sim.advance_to_midnight().expect("advances");
    assert!(
        sim.people().fires.list.iter().any(|f| f.burning()),
        "it still burns"
    );
    assert!(
        sim.people().held_detailed().contains(&s),
        "held while it burns"
    );
    assert!(
        residents(&sim, s).iter().all(|id| !sim
            .people()
            .person(*id)
            .expect("lives")
            .act
            .steps
            .is_empty()),
        "its people are about their day, and may fight it"
    );
    // Fires burn out within hours; by the midnight after, it is household-resolved again.
    sim.advance_minutes(2 * MINUTES_PER_DAY).expect("advances");
    assert!(!sim.people().fires.list.iter().any(|f| f.burning()));
    assert!(sim.people().held_detailed().is_empty());
    assert!(sim.people().resolution_problems().is_empty());
}

#[test]
fn nothing_not_taken_is_ever_lived_while_household_resolved() {
    // ADR-0032 §2: visits, fetches, carrying dues, bands and fighting fires are never taken while
    // household-resolved; work, care and the rest are.
    let mut sim = two_settlements();
    let both: Vec<_> = sim.land().settlements.iter().map(|x| x.id).collect();
    sim.set_household_resolved(both);
    sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    let before = sim.people().time_use.work;
    sim.advance_minutes(15 * MINUTES_PER_DAY).expect("advances");
    let after = sim.people().time_use.work;
    let minutes = |b: civ_agents::params::Behavior| {
        let i = civ_agents::params::Behavior::ALL
            .iter()
            .position(|&x| x == b)
            .expect("listed");
        after[i] - before[i]
    };
    for b in civ_agents::population::NOT_TAKEN {
        assert_eq!(minutes(b), 0.0, "{b:?} was lived");
    }
    assert!(
        minutes(civ_agents::params::Behavior::Farm) > 0.0
            || minutes(civ_agents::params::Behavior::Gather) > 0.0
    );
    assert!(sim.people().resolution_problems().is_empty());
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
}

/// Five founding groups of 30 on a river valley with a fixed identity, at a midnight.
fn five_settlements() -> Sim {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Focus".to_owned(),
            seed: 3,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: vec![30; 4],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates");
    assert_eq!(
        sim.land().settlements.len(),
        5,
        "{:?}",
        sim.founding_problem()
    );
    sim.advance_to_midnight().expect("advances");
    sim
}

#[test]
fn under_the_focus_settlements_away_from_it_are_household_resolved_and_the_focus_held_its_days() {
    // M9b slice DK (ADR-0032 §6): off, as every world begins, nothing is household-resolved.
    let mut sim = five_settlements();
    let ids: Vec<PermanentId> = sim.land().settlements.iter().map(|s| s.id).collect();
    sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    assert!(sim.household_resolved().is_empty());
    // On, with the first in focus and the third pinned: the rest from the next midnight.
    sim.set_resolution_by_focus(true);
    sim.set_focus(Some(ids[0]));
    sim.pin_detailed(ids[2], true);
    sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    let away: std::collections::BTreeSet<_> = [ids[1], ids[3], ids[4]].into();
    assert_eq!(sim.household_resolved(), &away);
    // The observer turns to the second: Detailed from the next midnight, the first held.
    sim.set_focus(Some(ids[1]));
    sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    assert_eq!(sim.household_resolved(), &[ids[3], ids[4]].into());
    sim.advance_minutes(29 * MINUTES_PER_DAY).expect("advances");
    assert!(
        sim.household_resolved().contains(&ids[0]),
        "let go after its days"
    );
    assert!(!sim.household_resolved().contains(&ids[1]));
    let problems = sim
        .people()
        .problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    assert!(problems.is_empty(), "{problems:?}");
    // Off: every settlement Detailed from the next midnight.
    sim.set_resolution_by_focus(false);
    sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    assert!(sim.household_resolved().is_empty());
}

#[test]
fn a_household_resolved_persons_day_is_shown_block_by_block_by_the_clock() {
    // M9b slice DK (ADR-0032 §6): someone of a household-resolved settlement is shown in the
    // block of their day the clock is in, drawn where it is done; someone Detailed is in none.
    let mut sim = two_settlements();
    let s = second(&sim);
    let first = sim.land().settlements[0].id;
    sim.set_household_resolved([s]);
    sim.advance_to_midnight().expect("advances");
    assert_eq!(sim.household_resolved(), &BTreeSet::from([s]));
    let parked = residents(&sim, s);
    let detailed = residents(&sim, first);
    assert!(!parked.is_empty() && !detailed.is_empty());
    let home = |sim: &Sim, id: PermanentId| {
        let people = sim.people();
        let p = people.person(id).expect("lives");
        people.household(p.household).expect("household").home
    };
    let (mut shown, mut away, mut asleep_at_three) = (0usize, 0usize, 0usize);
    for step in 0..48 {
        if step > 0 {
            sim.advance_minutes(30).expect("advances");
        }
        let now = sim.now();
        let minute = now.minutes() - now.day_index() * MINUTES_PER_DAY;
        for &id in &detailed {
            assert!(
                sim.people().day_block(id, now).is_none(),
                "{id:?} is Detailed"
            );
        }
        for &id in &parked {
            let Some(b) = sim.people().day_block(id, now) else {
                continue;
            };
            shown += 1;
            assert!(i64::from(b.from) <= minute && minute < i64::from(b.until));
            if minute == 180 && b.asleep {
                asleep_at_three += 1;
            }
            if b.asleep {
                assert_eq!(b.at, home(&sim, id), "the night is slept at home");
            } else if b.at != home(&sim, id) {
                away += 1;
            }
        }
    }
    assert!(shown > 0, "blocks are shown");
    assert!(away > 0, "some blocks are done away from home");
    assert!(asleep_at_three > 0, "the night is shown asleep");
    // The next day is a new day's blocks; the day before's are gone.
    let before = sim.now();
    sim.advance_to_midnight().expect("advances");
    assert!(
        parked
            .iter()
            .all(|&id| sim.people().day_block(id, before).is_none())
    );
    // Brought back, they are Detailed and in no block.
    sim.set_household_resolved([]);
    sim.advance_to_midnight().expect("advances");
    sim.advance_minutes(600).expect("advances");
    assert!(
        parked
            .iter()
            .all(|&id| sim.people().day_block(id, sim.now()).is_none())
    );
}

#[test]
fn under_the_focus_the_settlement_of_the_person_followed_is_held_detailed() {
    // M9b slice DK (ADR-0032 §6): following someone holds their settlement as the focus is.
    let mut sim = five_settlements();
    let ids: Vec<PermanentId> = sim.land().settlements.iter().map(|s| s.id).collect();
    let someone = residents(&sim, ids[3])[0];
    sim.set_resolution_by_focus(true);
    sim.set_focus(Some(ids[0]));
    sim.set_follow(Some(someone));
    sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    assert_eq!(sim.household_resolved(), &[ids[1], ids[2], ids[4]].into());
}

#[test]
fn a_skip_lives_every_settlement_by_household_at_max_and_puts_everything_back() {
    // M9c slice DL (ADR-0032 §7): even two settlements, fewer than the focus would resolve.
    let mut sim = two_settlements();
    let all: BTreeSet<PermanentId> = sim.land().settlements.iter().map(|s| s.id).collect();
    let speed = sim.speed();
    let restore = sim.begin_skip();
    assert_eq!(sim.mode(), civ_sim::Mode::Accelerated, "at a midnight: now");
    sim.advance_minutes(3 * MINUTES_PER_DAY).expect("advances");
    assert_eq!(sim.household_resolved(), &all);
    sim.end_skip(restore);
    assert_eq!(sim.speed(), speed);
    assert!(sim.focus_policy().is_none());
    sim.advance_to_midnight().expect("advances");
    assert!(sim.household_resolved().is_empty(), "Detailed again");
    assert_eq!(sim.mode(), civ_sim::Mode::Detailed);
    let problems = sim
        .people()
        .problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn a_skip_puts_back_the_observers_focus_as_it_was() {
    let mut sim = five_settlements();
    let ids: Vec<PermanentId> = sim.land().settlements.iter().map(|s| s.id).collect();
    sim.set_resolution_by_focus(true);
    sim.set_focus(Some(ids[0]));
    sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    let restore = sim.begin_skip();
    sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    assert_eq!(sim.household_resolved().len(), ids.len(), "every one");
    sim.end_skip(restore);
    assert_eq!(sim.focus_policy().and_then(|p| p.focus), Some(ids[0]));
    sim.advance_minutes(MINUTES_PER_DAY).expect("advances");
    assert!(
        !sim.household_resolved().contains(&ids[0]),
        "the focus is Detailed again"
    );
}

#[test]
fn a_skip_stops_at_the_first_entry_of_a_kind_marked() {
    use civ_sim::skip::{mark_of, stop};
    let sim = two_settlements();
    // The settlements' foundings are in the chronicle from the start.
    let first = sim
        .skip_stop(0, stop::SETTLEMENTS)
        .expect("a settlement founded");
    assert_eq!(mark_of(first.kind), stop::SETTLEMENTS);
    let earliest = sim
        .people()
        .chronicle
        .iter()
        .find(|e| mark_of(e.kind) & stop::SETTLEMENTS != 0)
        .expect("one");
    assert_eq!(first.seq, earliest.seq, "the first after the head given");
    assert!(!sim.chronicle_words(first).is_empty());
    assert!(sim.skip_stop(sim.chronicle_head(), stop::ALL).is_none());
    assert!(
        sim.skip_stop(0, 0).is_none(),
        "nothing marked stops nothing"
    );
}
