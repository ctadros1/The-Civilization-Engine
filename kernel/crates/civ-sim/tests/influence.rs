//! The observer's interventions (plan §7 M4c slice AJ, ADR-0016 §5; research 15-05 §4.2, §4.5):
//! a whisper places a true claim in one person's hearing and does nothing else; an ideology told
//! of is weighed by how well it fits what they hold dear, by a draw a repeat never makes again;
//! each use is one record, logged as one recorded influence, and all of it saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::ChronicleKind;
use civ_agents::influence::InfluenceKind;
use civ_agents::word::{Blamed, Claim, ClaimKind, Grieved};
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
            name: "Influences".to_owned(),
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
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "influence").expect("saves");
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

/// The adults of the first settlement, in id order, with it.
fn adults(sim: &Sim) -> (PermanentId, Vec<PermanentId>) {
    let pop = sim.people();
    let now = sim.now();
    let grown = sim.rules().people.family.independent_age;
    let settlement = pop.polities[0].settlement;
    let mut ids: Vec<PermanentId> = pop
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) >= grown)
        .filter(|p| pop.household(p.household).and_then(|x| x.settlement) == Some(settlement))
        .map(|p| p.id)
        .collect();
    ids.sort_unstable();
    (settlement, ids)
}

#[test]
fn a_whisper_is_heard_as_news_is_and_a_repeat_adds_nothing() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(DAY + 60).expect("advances");
    let (settlement, adults) = adults(&sim);
    let (holder, target) = (adults[0], adults[1]);
    let body = sim.people().polities[0].id;
    // A grievance `holder` holds and has told: a true claim of their settlement's word.
    let day = sim.now().day_index();
    let pop = sim.people_mut_for_tests();
    let claim = pop.word.make(Claim {
        id: 0,
        kind: ClaimKind::Grievance,
        settlement,
        day,
        subject: Some(holder),
        grievance: Some((Blamed::Body(body), Grieved::Extraction)),
    });
    pop.word.hear(holder, claim, day, None, Some(holder));
    let before = {
        let pop = sim.people();
        (
            pop.word.grievances.clone(),
            pop.factions.list.len(),
            pop.polities[0].laws.len(),
            pop.ideologies.held.clone(),
        )
    };
    let entries = |sim: &Sim| {
        sim.people()
            .chronicle
            .iter()
            .filter(|e| e.kind == ChronicleKind::Influence)
            .count()
    };
    let reached = sim.whisper(target, claim).expect("whispers");
    assert!(!reached.repeat);
    {
        let pop = sim.people();
        let heard = pop
            .word
            .heard_by(target)
            .iter()
            .find(|h| h.claim == claim)
            .expect("heard");
        assert_eq!((heard.from, heard.origin), (None, None), "from no one");
        assert_eq!(pop.influences.list.len(), 1);
        let r = pop.influences.list[0];
        assert_eq!(
            (r.kind, r.target, r.subject, r.uses),
            (InfluenceKind::Whisper, target, claim, 1)
        );
        let e = pop.chronicle.last().expect("logged");
        assert_eq!(
            (e.kind, e.people.as_slice()),
            (ChronicleKind::Influence, &[target][..])
        );
        assert!(
            e.name.contains("holds a grievance against the gathering"),
            "{}",
            e.name
        );
    }
    // A hundred whispers more are one: the record is refreshed, never added to.
    for _ in 0..100 {
        assert!(sim.whisper(target, claim).expect("whispers").repeat);
    }
    assert_eq!(sim.people().influences.list.len(), 1);
    assert_eq!(sim.people().influences.list[0].uses, 101);
    assert_eq!(entries(&sim), 1);
    // Who has heard it already, or is a child, cannot be whispered it.
    assert!(sim.whisper(holder, claim).is_err());
    let now = sim.now();
    let grown = sim.rules().people.family.independent_age;
    let child = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .find(|p| p.age_years(now) < grown)
        .map(|p| p.id);
    if let Some(child) = child {
        assert!(sim.whisper(child, claim).is_err());
    }
    // It wrote nothing but the hearing: no grievance, faction, law or holding.
    let pop = sim.people();
    assert_eq!(pop.word.grievances, before.0);
    assert_eq!(pop.factions.list.len(), before.1);
    assert_eq!(pop.polities[0].laws.len(), before.2);
    assert_eq!(pop.ideologies.held, before.3);
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn an_ideology_told_of_is_weighed_by_fit_and_telling_again_draws_nothing_new() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(DAY + 60).expect("advances");
    let k = sim
        .rules()
        .catalog
        .ideologies
        .iter()
        .position(|d| d.id == "core:ideology/common_provision")
        .expect("in core") as u16;
    let id = sim.rules().catalog.ideologies[usize::from(k)].id.clone();
    // It is taken up readily where it fits at all: by hand, so the draw cannot decide it.
    let rules = sim.rules_mut_for_tests().expect("not yet shared");
    rules.catalog.ideologies[usize::from(k)].adopt = 5.0;
    let commitments = rules.catalog.ideologies[usize::from(k)].commitments.clone();
    let (_, adults) = adults(&sim);
    let free: Vec<PermanentId> = adults
        .iter()
        .copied()
        .filter(|&p| !sim.people().ideologies.holds(p, k))
        .collect();
    let (against, with) = (free[0], free[1]);
    let set = |sim: &mut Sim, p: PermanentId, sign: f32| {
        let pop = sim.people_mut_for_tests();
        for &(value, c) in &commitments {
            pop.values.insert(civ_agents::values::Held {
                holder: p,
                value,
                v: sign * c.signum(),
            });
        }
    };
    set(&mut sim, against, -1.0);
    set(&mut sim, with, 1.0);
    // One whom it does not fit hears of it and does not take it up, however often told.
    let r = sim.tell_of_ideology(against, &id).expect("tells");
    assert!(!r.repeat && !r.holds && r.chance == 0.0, "{r:?}");
    for _ in 0..100 {
        let again = sim.tell_of_ideology(against, &id).expect("tells");
        assert!(again.repeat && !again.holds);
    }
    // One whom it fits takes it up, from no one.
    let r = sim.tell_of_ideology(with, &id).expect("tells");
    assert!(r.holds && r.fit > 0.9, "{r:?}");
    let holder = {
        let pop = sim.people();
        let held = pop
            .ideologies
            .held_by(with)
            .iter()
            .find(|h| h.ideology == k)
            .expect("holds it");
        assert_eq!(held.from, None);
        assert_eq!(pop.influences.list.len(), 2, "one record each");
        let first = pop.influences.list[0];
        assert_eq!((first.uses, first.weighed, first.taken), (101, 1, None));
        assert!(pop.influences.list[1].taken.is_some());
        // Telling one who holds it already, from anyone, is refused.
        let holder = pop
            .ideologies
            .held
            .iter()
            .find(|h| h.ideology == k && h.holder != with)
            .map(|h| h.holder);
        holder.filter(|&h| adults.contains(&h))
    };
    if let Some(h) = holder {
        assert!(sim.tell_of_ideology(h, &id).is_err());
    }
    // A change in them can change the answer: once it fits, they take it up at their next review
    // while it is fresh, by the same draw.
    set(&mut sim, against, 1.0);
    sim.advance_minutes(31 * DAY).expect("advances");
    let pop = sim.people();
    let first = pop.influences.list[0];
    assert!(first.weighed >= 2, "{first:?}");
    assert!(first.taken.is_some(), "{first:?}");
    assert!(pop.ideologies.holds(against, k));
    let rules = sim.rules_mut_for_tests().expect("not yet shared");
    rules.catalog.ideologies[usize::from(k)].adopt = 0.3;
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn an_agitator_is_one_newcomer_holding_their_ideology_and_knowing_nobody() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(DAY + 60).expect("advances");
    let (hearth, village) = {
        let s = &sim.land().settlements[0];
        (s.hearth_m, s.name.clone())
    };
    let unions = sim.people().unions.len();
    let k = sim
        .rules()
        .catalog
        .ideologies
        .iter()
        .position(|d| d.id == "core:ideology/order_kept")
        .expect("in core") as u16;
    let (sent, record) = sim
        .send_agitator(hearth, "core:ideology/order_kept")
        .expect("sends");
    assert!(!sent.founded, "placed at the hearth, they join it");
    assert_eq!(sent.name, village);
    assert_eq!(sent.people.len(), 1, "one adult, alone");
    let id = sent.people[0];
    {
        let pop = sim.people();
        let p = pop.person(id).expect("here");
        assert!(p.age_years(sim.now()) >= 20.0);
        assert_eq!(p.partner, None);
        assert_eq!(pop.unions.len(), unions, "no couple came");
        assert_eq!(
            pop.household(p.household).map(|x| x.members.clone()),
            Some(vec![id]),
            "a household of their own"
        );
        let held = pop
            .ideologies
            .held_by(id)
            .iter()
            .find(|h| h.ideology == k)
            .expect("holds it");
        assert_eq!(held.from, None);
        assert!(pop.ties.of(id).is_empty(), "they know nobody");
        let r = pop.influences.list.last().expect("a record");
        assert_eq!(
            (r.id, r.kind, r.target, r.subject),
            (record, InfluenceKind::Agitator, id, u32::from(k))
        );
        assert!(r.taken.is_some());
        let e = pop.chronicle.last().expect("logged");
        assert_eq!(e.kind, ChronicleKind::Influence);
        let name = sim.rules().catalog.ideologies[usize::from(k)]
            .name
            .to_lowercase();
        assert!(
            e.name == format!(", who holds to {name}, to {village}."),
            "{}",
            e.name
        );
        assert!(
            !pop.chronicle
                .iter()
                .rev()
                .take(3)
                .any(|e| e.kind == ChronicleKind::FamilyArrived),
            "no family arrived"
        );
    }
    // Into the water, nobody can be sent.
    assert!(
        sim.send_agitator((-5.0, -5.0), "core:ideology/order_kept")
            .is_err()
    );
    assert!(sim.send_agitator(hearth, "core:ideology/none").is_err());
    sim.advance_minutes(DAY).expect("advances");
    assert!(sim.people().person(id).is_some(), "they live among them");
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn a_blessing_spares_and_a_curse_brings_what_their_own_draws_would_not_have() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(DAY + 60).expect("advances");
    // Illness or accident comes often: by hand, so a few days show what the draws do.
    let rules = sim.rules_mut_for_tests().expect("not yet shared");
    let c = rules.people.mortality.siler.c;
    rules.people.mortality.siler.c = 80.0;
    let (_, adults) = adults(&sim);
    let half = adults.len() / 2;
    assert!(half >= 5, "{} adults", adults.len());
    for &p in &adults[..half] {
        assert!(!sim.bless(p, false, 30, 0.5).expect("blesses").repeat);
    }
    for &p in &adults[half..] {
        assert!(!sim.bless(p, true, 30, 0.5).expect("curses").repeat);
    }
    // Out of range, or a second blessing: refused, or refreshed and never stacked.
    assert!(sim.bless(adults[0], false, 30, 0.9).is_err());
    assert!(sim.bless(adults[0], false, 0, 0.25).is_err());
    assert!(
        sim.bless(adults[0], false, 60, 0.5)
            .expect("refreshes")
            .repeat
    );
    assert_eq!(sim.people().influences.list.len(), adults.len());
    sim.advance_minutes(10 * DAY).expect("advances");
    {
        let pop = sim.people();
        let (mut spared, mut brought) = (0, 0);
        for r in &pop.influences.list {
            match r.kind {
                InfluenceKind::Bless => spared += r.deaths,
                InfluenceKind::Curse => brought += r.deaths,
                _ => {}
            }
        }
        assert!(
            spared > 0 && brought > 0,
            "{spared} spared, {brought} brought"
        );
        let told = pop
            .chronicle
            .iter()
            .filter(|e| e.kind == ChronicleKind::InfluenceTurned)
            .count() as u32;
        let finds: u32 = pop.influences.list.iter().map(|r| r.finds).sum();
        assert_eq!(told, spared + brought + finds, "every turned draw is told");
        // Every death a curse brought was a death of the one cursed.
        for e in pop
            .chronicle
            .iter()
            .filter(|e| e.kind == ChronicleKind::InfluenceTurned && e.name.contains("curse"))
        {
            if e.name.contains("died") {
                assert!(pop.person(e.people[0]).is_none(), "they died");
            }
        }
    }
    // A blessing given to one cursed ends the curse that day.
    let cursed = adults[half..]
        .iter()
        .copied()
        .find(|&p| sim.people().person(p).is_some());
    if let Some(p) = cursed {
        sim.bless(p, false, 5, 0.25).expect("blesses");
        let day = sim.now().day_index();
        let pop = sim.people();
        let luck = pop.influences.luck(p, day).expect("blessed now");
        assert!(luck.bless);
        assert!(
            pop.influences
                .list
                .iter()
                .filter(|r| r.target == p && r.kind == InfluenceKind::Curse)
                .all(|r| r.until == day)
        );
    }
    let rules = sim.rules_mut_for_tests().expect("not yet shared");
    rules.people.mortality.siler.c = c;
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}
