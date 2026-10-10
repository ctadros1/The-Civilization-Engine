//! Factions (plan §7 M4c slice AH, steps one and two; ADR-0017 §2-3; research 04-10 §1.1–§1.5,
//! §3, §5.1–§5.4): someone who feels a grievance against the gathering keenly, and has heard that
//! others they trust hold one too, founds a faction over it; others who know its members join it
//! for their own reasons, which the record keeps; a member's household short of food is given
//! from its store; its organizer petitions the gathering, those who heard choose whether to come,
//! and the gathering decides what it asks as any law; all of it saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::ledger::Channel;
use civ_agents::polity::{Body, IssueKind, Law, LawStatus, Outcome, PolicyKind};
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
    let (sim, adults, _) = aggrieved_at(seed, false);
    (sim, adults)
}

/// As [`aggrieved_village`]; with `keeper`, in the spring of the second year, when households
/// are under roofs: a store in force and its keeper (the village's own, or else the last adult by
/// id), and the grievance held against the keeper over food the store did not have. Returns the
/// keeper's law with the keeper too.
fn aggrieved_at(
    seed: u64,
    keeper: bool,
) -> (Sim, Vec<PermanentId>, Option<(PermanentId, PermanentId)>) {
    let mut sim = world_with(content(), seed);
    let days = if keeper { 405 } else { 40 };
    sim.advance_minutes(days * DAY).expect("advances");
    let now = sim.now();
    let day = now.day_index();
    let params = sim.rules().people.clone();
    let policy_of = |sim: &Sim, kind: PolicyKind| {
        sim.rules()
            .catalog
            .policies
            .iter()
            .position(|p| p.kind == kind)
            .expect("in core") as u16
    };
    let (store, keep) = (
        policy_of(&sim, PolicyKind::CommonStore),
        policy_of(&sim, PolicyKind::KeepStore),
    );
    let next = sim.ids().peek_next() + 1_000_000;
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
    let mut office = pop.polities[0]
        .keeper()
        .map(|(holder, law)| (law.id, holder))
        .filter(|_| keeper);
    if keeper && office.is_none() {
        let holder = *adults.last().expect("adults");
        let law = |id: u64, policy: u16, kind: PolicyKind, holder: Option<PermanentId>| Law {
            id: PermanentId::from_raw(id).expect("non-zero"),
            policy,
            kind,
            levy_share: if kind == PolicyKind::CommonStore {
                0.05
            } else {
                0.0
            },
            holder,
            relief_days: 5.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: adults[0],
            proposed: now,
            issue: IssueKind::FoodShort,
            meets_day: day,
            decided: Some(now),
            outcome: Some(Outcome::Passed),
            eligible: 0,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Default::default(),
            watch: Default::default(),
            body: None,
            ends: None,
            agreement: None,
        };
        let laws = &mut pop.polities[0].laws;
        if !laws
            .iter()
            .any(|l| l.kind == PolicyKind::CommonStore && l.status == LawStatus::InForce)
        {
            laws.push(law(next, store, PolicyKind::CommonStore, None));
        }
        laws.push(law(next + 1, keep, PolicyKind::KeepStore, Some(holder)));
        office = Some((PermanentId::from_raw(next + 1).expect("non-zero"), holder));
    }
    let (against, issue, wrong, over) = match office {
        Some((law, _)) => (
            Blamed::Office(law),
            Grieved::Subsistence,
            Wrong::StoreEmpty,
            law,
        ),
        None => (
            Blamed::Body(polity),
            Grieved::Extraction,
            Wrong::LeanLevy,
            polity,
        ),
    };
    // The keeper does not blame their own keeping.
    let holders: Vec<PermanentId> = adults
        .iter()
        .copied()
        .filter(|&a| office.is_none_or(|(_, k)| k != a))
        .collect();
    for &a in &holders {
        pop.word.grievances.push(Grievance {
            holder: a,
            issue,
            blamed: against,
            law: over,
            harm_days: 10.0,
            unresolved_days: 10.0,
            activation: 1.0,
            raised: day,
            made: day,
            wrong,
        });
        let claim = pop.word.make(Claim {
            id: 0,
            kind: ClaimKind::Grievance,
            settlement,
            day,
            subject: Some(a),
            grievance: Some((against, issue)),
        });
        for &b in &holders {
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
    (sim, adults, office)
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

#[test]
fn a_faction_petitions_the_gathering_and_those_who_heard_choose_whether_to_come() {
    let (mut sim, _) = aggrieved_village(5);
    // A store in force taking a fifth of each harvest, from households that hold plenty: what it
    // takes, they expect never to need back.
    let grain = sim
        .rules()
        .catalog
        .good_index("core:good/grain")
        .expect("in core");
    let store = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|p| p.kind == PolicyKind::CommonStore)
        .expect("in core") as u16;
    let now = sim.now();
    let old = PermanentId::from_raw(sim.ids().peek_next() + 1_000_000).expect("non-zero");
    let pop = sim.people_mut_for_tests();
    let sponsor = pop.polities[0].laws.first().map_or_else(
        || pop.people.iter().map(|(_, p)| p.id).min().expect("people"),
        |l| l.sponsor,
    );
    pop.polities[0].laws.push(Law {
        id: old,
        policy: store,
        kind: PolicyKind::CommonStore,
        levy_share: 0.2,
        holder: None,
        relief_days: 5.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor,
        proposed: now,
        issue: IssueKind::FoodShort,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Outcome::Passed),
        eligible: 0,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
    });
    for (_, x) in pop.households.iter_mut() {
        x.stores.resize(grain + 1, 0.0);
        x.stores[grain] += 4000.0;
    }
    // A faction is founded, and at its organizer's next review they call a petition; it sits the
    // next evening and goes before the gathering.
    let mut called = None;
    for _ in 0..80 {
        sim.advance_minutes(DAY).expect("advances");
        if let Some(p) = sim.people().factions.petitions.first() {
            called = Some(p.clone());
            break;
        }
    }
    let called = called.expect("a petition was called");
    let pop = sim.people();
    assert_eq!(called.ends, old, "it asks to replace the store in force");
    assert!(
        (f64::from(called.levy_share) - 0.2).abs() > 1e-6,
        "at another share: {called:?}"
    );
    let f = pop.factions.get(called.faction).expect("its faction");
    assert_eq!(called.organizer, f.organizer);
    // Its members heard of it first, from the organizer.
    let claim = pop
        .word
        .petition(called.settlement, called.day)
        .expect("word of it");
    for m in pop.factions.members_of(f.id) {
        assert!(pop.word.has_heard(m.person, claim), "{m:?}");
    }
    let words = civ_sim::frames::word::claim_words(
        &sim,
        pop.word
            .claims
            .iter()
            .find(|c| c.id == claim)
            .expect("kept"),
    );
    assert!(words.contains("petitions the gathering"), "{words}");
    // Late on the evening it sits, who has heard of it (word of it is let go once it has sat).
    let late = called.day * DAY + 23 * 60;
    assert!(sim.now().minutes() <= late, "it has not sat yet");
    sim.advance_minutes(late - sim.now().minutes())
        .expect("advances");
    let pop = sim.people();
    let heard: Vec<PermanentId> = pop
        .people
        .iter()
        .map(|(_, p)| p.id)
        .filter(|&q| pop.word.has_heard(q, claim))
        .collect();
    // It sits, goes before the gathering, and is decided.
    for _ in 0..10 {
        sim.advance_minutes(DAY).expect("advances");
        if sim.people().factions.petitions[0].answered {
            break;
        }
    }
    let pop = sim.people();
    let p = &pop.factions.petitions[0];
    assert!(p.answered, "{p:?}");
    assert!(!p.came.is_empty(), "some came: {p:?}");
    // Only those who had heard of it came, each by their own choice.
    for &q in &p.came {
        assert!(heard.contains(&q), "{q} came unheard");
    }
    let law = p.law.expect("it went before the gathering");
    let polity = &pop.polities[0];
    let l = polity.laws.iter().find(|l| l.id == law).expect("kept");
    assert_eq!(l.issue, IssueKind::Petition);
    assert_eq!(l.sponsor, called.organizer);
    assert_eq!(l.ends, Some(old));
    let before = polity.laws.iter().find(|l| l.id == old).expect("kept");
    match l.outcome {
        Some(Outcome::Passed) => {
            assert_eq!(l.status, LawStatus::InForce);
            assert_eq!(
                before.status,
                LawStatus::Superseded,
                "the old levy gave way"
            );
        }
        other => {
            assert!(other.is_some(), "decided: {l:?}");
            assert_eq!(before.status, LawStatus::InForce);
            // Those who came hold its turning down against the gathering.
            for &q in &p.came {
                if pop.person(q).is_none() {
                    continue;
                }
                assert!(
                    pop.word
                        .grievances_of(q)
                        .any(|g| g.wrong == Wrong::Refused && g.law == law),
                    "{q} holds the refusal"
                );
            }
        }
    }
    // The government panel says what happened.
    let lines = civ_sim::frames::government::petition_words(&sim, polity.settlement);
    assert!(
        lines.iter().any(|w| w.contains("petitioned the gathering")),
        "{lines:?}"
    );
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn a_faction_against_the_keeper_petitions_for_another_and_the_gathering_weighs_both() {
    let (mut sim, _, office) = aggrieved_at(7, true);
    let (old, keeper) = office.expect("a keeper");
    let mut called = None;
    for _ in 0..80 {
        sim.advance_minutes(DAY).expect("advances");
        if let Some(p) = sim.people().factions.petitions.first() {
            called = Some(p.clone());
            break;
        }
    }
    let called = called.expect("a petition was called");
    let pop = sim.people();
    let f = pop.factions.get(called.faction).expect("its faction");
    assert_eq!(f.against, Blamed::Office(old));
    // It asks for another keeper in place of the one blamed.
    assert_eq!(called.ends, old);
    let nominee = called.nominee.expect("someone named");
    assert_ne!(nominee, keeper);
    let words = civ_sim::frames::word::petition_demand_words(&sim, &called);
    assert!(
        words.contains("as keeper of the common store in place of"),
        "{words}"
    );
    for _ in 0..12 {
        sim.advance_minutes(DAY).expect("advances");
        if sim.people().factions.petitions[0].answered {
            break;
        }
    }
    let pop = sim.people();
    let p = &pop.factions.petitions[0];
    assert!(p.answered && !p.came.is_empty(), "{p:?}");
    let law = p.law.expect("it went before the gathering");
    let polity = &pop.polities[0];
    let l = polity.laws.iter().find(|l| l.id == law).expect("kept");
    assert_eq!((l.holder, l.ends), (Some(nominee), Some(old)));
    // The keeper in office did not back being replaced.
    if let Some(r) = l.stances.iter().find(|r| r.person == keeper) {
        assert!(r.regard <= 0.0, "{r:?}");
        assert_ne!(r.stance, civ_agents::polity::Stance::Support, "{r:?}");
    }
    let before = polity.laws.iter().find(|l| l.id == old).expect("kept");
    if l.outcome == Some(Outcome::Passed) {
        assert_eq!(before.status, LawStatus::Superseded);
        assert_eq!(polity.keeper().map(|k| k.0), Some(nominee));
    } else {
        assert_eq!(before.status, LawStatus::InForce);
        assert_eq!(polity.keeper().map(|k| k.0), Some(keeper));
    }
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn where_no_petition_can_be_called_a_faction_keeps_back_the_levy_together() {
    let (mut sim, adults) = aggrieved_village(5);
    let store = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|p| p.kind == PolicyKind::CommonStore)
        .expect("in core") as u16;
    let now = sim.now();
    let law = PermanentId::from_raw(sim.ids().peek_next() + 1_000_000).expect("non-zero");
    let pop = sim.people_mut_for_tests();
    let sponsor = pop.people.iter().map(|(_, p)| p.id).min().expect("people");
    pop.polities[0].laws.push(Law {
        id: law,
        policy: store,
        kind: PolicyKind::CommonStore,
        levy_share: 0.2,
        holder: None,
        relief_days: 5.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor,
        proposed: now,
        issue: IssueKind::FoodShort,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Outcome::Passed),
        eligible: 0,
        stances: Vec::new(),
        known: adults.iter().map(|&a| (a, now.day_index())).collect(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
    });
    // Nobody here holds that what the gathering decides binds, nor believes others abide by it.
    for s in &mut pop.norms.states {
        s.endorse = 0.0;
        s.expect = 0.0;
    }
    // A faction is founded; it petitioned lately, so it cannot again.
    let mut faction = None;
    for _ in 0..40 {
        sim.advance_minutes(DAY).expect("advances");
        if let Some(f) = sim.people().factions.list.iter().find(|f| f.is_live()) {
            faction = Some((f.id, f.settlement, f.organizer));
            break;
        }
    }
    let (faction, settlement, organizer) = faction.expect("a faction");
    let now = sim.now();
    let id = PermanentId::from_raw(sim.ids().peek_next() + 2_000_000).expect("non-zero");
    sim.people_mut_for_tests()
        .factions
        .petitions
        .push(civ_agents::faction::Petition {
            id,
            faction,
            settlement,
            organizer,
            called: now,
            day: now.day_index(),
            policy: store,
            levy_share: 0.0,
            nominee: None,
            ends: law,
            came: Vec::new(),
            law: None,
            answered: true,
        });
    let mut called = None;
    for _ in 0..40 {
        sim.advance_minutes(DAY).expect("advances");
        if let Some(r) = sim.people().factions.refusals.first() {
            called = Some(r.clone());
            break;
        }
    }
    let called = called.expect("a refusal was called");
    assert_eq!((called.faction, called.law), (faction, law));
    let pop = sim.people();
    let claim = pop.word.refusal(called.id).expect("word of it");
    for m in pop.factions.members_of(faction) {
        assert!(pop.word.has_heard(m.person, claim), "{m:?}");
    }
    // At threshing, some of those who heard of it keep back their household's levy under it.
    let mut kept = false;
    for _ in 0..30 {
        sim.advance_minutes(10 * DAY).expect("advances");
        let pop = sim.people();
        if pop.factions.refusals[0].kept.len() >= 2 {
            kept = true;
            break;
        }
    }
    assert!(kept, "{:?}", sim.people().factions.refusals[0]);
    let pop = sim.people();
    let r = &pop.factions.refusals[0];
    let l = pop.polities[0]
        .laws
        .iter()
        .find(|l| l.id == law)
        .expect("kept");
    // Each keeping back is counted on the law; each who kept back once on the refusal.
    assert!(l.compliance.refused as usize >= r.kept.len());
    assert!(r.kept_kg > 0.0 && (l.compliance.refused_kg - r.kept_kg).abs() < 1e-6);
    assert!(l.compliance.withheld_kg >= l.compliance.refused_kg);
    let lines = civ_sim::frames::government::refusal_words(&sim, settlement);
    assert!(
        lines.iter().any(|w| w.contains("to keep back the levy of")),
        "{lines:?}"
    );
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

/// A village of [`aggrieved_village`] whose founding custom lets only its elders decide, where
/// nobody holds the norm that the gathering binds, and where a levy the elders passed went against
/// the households of the faction its grievance founds: no amendment one change from the custom
/// would have decided it otherwise, so none would serve them, and there is no levy in force to
/// petition over or keep back. Runs until its organizer calls a revolt; returns it, and the
/// elders' body.
fn revolt_called(seed: u64) -> (Sim, civ_agents::faction::Revolt, Body) {
    use civ_agents::polity::{Membership, Stance};
    let (mut sim, adults) = aggrieved_village(seed);
    let pop = sim.people_mut_for_tests();
    // Its founding custom lets only the elders of its households decide.
    let elders = Body {
        members: Membership::Elders,
        ..pop.polities[0].body
    };
    pop.polities[0].body = elders;
    pop.polities[0].versions[0].body = elders;
    // Nobody here holds that what the gathering decides binds, nor believes others abide by it.
    for s in &mut pop.norms.states {
        s.endorse = 0.0;
        s.expect = 0.0;
    }
    let mut faction = None;
    for _ in 0..40 {
        sim.advance_minutes(DAY).expect("advances");
        if let Some(f) = sim.people().factions.list.iter().find(|f| f.is_live()) {
            faction = Some((f.id, f.settlement, f.organizer));
            break;
        }
    }
    let (faction, settlement, organizer) = faction.expect("a faction");
    let store = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|p| p.kind == PolicyKind::CommonStore)
        .expect("in core") as u16;
    let now = sim.now();
    let day = now.day_index();
    let law = PermanentId::from_raw(sim.ids().peek_next() + 1_000_000).expect("non-zero");
    let came: Vec<PermanentId> = sim
        .people()
        .body_members(&sim.land().fields, 0, now, &sim.rules().people)
        .into_iter()
        .map(|m| m.0)
        .collect();
    let pop = sim.people_mut_for_tests();
    let household = |pop: &civ_agents::population::Population, p: PermanentId| {
        pop.person(p).expect("here").household
    };
    let mut theirs: Vec<PermanentId> = pop
        .factions
        .members_of(faction)
        .map(|m| household(pop, m.person))
        .collect();
    theirs.sort_unstable();
    theirs.dedup();
    // A levy the elders decided lately, all of them there: those of the faction's households
    // stood against it, two in three or more stood for it, and it passed. No body one change
    // from the custom would have decided it otherwise, so no amendment would serve them; a
    // gathering of all the adults, half of whom must come, would not have reached it. The store
    // it made has since given way, so there is no levy to petition over or keep back.
    let stood = |p: PermanentId, stance, gain| civ_agents::polity::StanceRecord {
        person: p,
        household: household(pop, p),
        stance,
        gain,
        regard: 0.0,
        opinion: 0.0,
        values: 0.0,
    };
    let (against, others): (Vec<PermanentId>, Vec<PermanentId>) = came
        .iter()
        .partition(|&&a| theirs.binary_search(&household(pop, a)).is_ok());
    let (o, s, n) = (against.len(), others.len(), adults.len());
    assert!(
        o >= 1 && 3 * s >= 2 * (s + o) && 2 * came.len() < n && 4 * came.len() >= n,
        "{o} against, {s} for, of {} elders and {n} adults",
        came.len()
    );
    let mut stances: Vec<_> = against
        .iter()
        .map(|&a| stood(a, Stance::Oppose, -5.0))
        .chain(others.iter().map(|&a| stood(a, Stance::Support, 0.0)))
        .collect();
    stances.sort_by_key(|r| r.person);
    pop.polities[0].laws.push(Law {
        id: law,
        policy: store,
        kind: PolicyKind::CommonStore,
        levy_share: 0.2,
        holder: None,
        relief_days: 5.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::Superseded,
        sponsor: others[0],
        proposed: now,
        issue: IssueKind::FoodShort,
        meets_day: day,
        decided: Some(now),
        outcome: Some(Outcome::Passed),
        eligible: came.len() as u32,
        stances,
        known: adults.iter().map(|&a| (a, day)).collect(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
    });
    assert!(
        !pop.polities[0]
            .laws
            .iter()
            .any(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::CommonStore),
        "no levy in force"
    );
    // At the organizer's next review, they call on everyone to stand with the faction.
    let mut called = None;
    for _ in 0..40 {
        sim.advance_minutes(DAY).expect("advances");
        if let Some(r) = sim.people().factions.revolts.first() {
            called = Some(r.clone());
            break;
        }
    }
    let called = called.expect("a revolt was called");
    assert_eq!(
        (called.faction, called.settlement, called.organizer),
        (faction, settlement, organizer)
    );
    assert_ne!(called.body, elders, "it would put another body in place");
    assert_eq!(called.body.members, Membership::Adults, "{:?}", called.body);
    let pop = sim.people();
    assert!(pop.factions.petitions.is_empty() && pop.factions.refusals.is_empty());
    let claim = pop
        .word
        .episode(ClaimKind::Revolt, called.id)
        .expect("word of it");
    for m in pop.factions.members_of(faction) {
        assert!(pop.word.has_heard(m.person, claim), "{m:?}");
    }
    let words = civ_sim::frames::word::claim_words(
        &sim,
        pop.word
            .claims
            .iter()
            .find(|c| c.id == claim)
            .expect("kept"),
    );
    assert!(
        words.contains("calls on everyone to stand with it"),
        "{words}"
    );
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == civ_agents::ChronicleKind::RevoltCalled),
        "the chronicle tells it"
    );
    (sim, called, elders)
}

#[test]
fn where_neither_petition_nor_refusal_can_be_a_faction_calls_a_revolt_which_holds() {
    use civ_agents::faction::{RevoltEnd, Side};
    let (mut sim, called, _) = revolt_called(5);
    let (settlement, organizer) = (called.settlement, called.organizer);
    let claim = sim
        .people()
        .word
        .episode(ClaimKind::Revolt, called.id)
        .expect("word of it");
    // Each day, those who heard of it take a side, until it holds.
    let mut sided = false;
    for _ in 0..(called.until - sim.now().day_index() + 2) {
        sim.advance_minutes(DAY).expect("advances");
        let r = &sim.people().factions.revolts[0];
        sided |= !r.sides.is_empty();
        if r.ended.is_some() {
            break;
        }
    }
    assert!(sided, "people took sides");
    let pop = sim.people();
    let r = &pop.factions.revolts[0];
    let (end, at) = r.ended.expect("it ended");
    assert_eq!(end, RevoltEnd::Held, "{r:?}");
    // Only those who had heard of it took a side; more stood with it than with the gathering.
    for &(p, _) in &r.sides {
        assert!(
            pop.word.has_heard(p, claim) || pop.person(p).is_none(),
            "{p}"
        );
    }
    assert!(r.count(Side::With) > r.count(Side::Gathering), "{r:?}");
    // Its body decides from now on: a version of the custom taken, not amended.
    let p = &pop.polities[0];
    assert_eq!(p.body, called.body, "its body decides now");
    let v = p.versions.last().expect("versions");
    assert_eq!((v.body, v.since, v.law), (called.body, at, None));
    assert_eq!(v.seized_by, Some(organizer));
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == civ_agents::ChronicleKind::CustomTaken),
        "the chronicle tells it"
    );
    let lines = civ_sim::frames::government::revolt_words(&sim, settlement);
    assert!(lines.iter().any(|w| w.contains("it held on")), "{lines:?}");
    let history = civ_agents::polity::custom_history_words(p, &|_| "Ada".to_owned());
    assert!(
        history
            .last()
            .is_some_and(|h| h.contains("not by its procedure")),
        "{history:?}"
    );
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn a_revolt_that_has_not_held_when_its_time_is_out_comes_to_nothing() {
    use civ_agents::faction::RevoltEnd;
    let (mut sim, called, elders) = revolt_called(5);
    // Its time is out tonight, before anyone could have stood with it a week.
    let today = sim.now().day_index();
    sim.people_mut_for_tests().factions.revolts[0].until = today;
    sim.advance_minutes(2 * DAY).expect("advances");
    let pop = sim.people();
    let r = &pop.factions.revolts[0];
    assert_eq!(r.ended.map(|e| e.0), Some(RevoltEnd::Failed), "{r:?}");
    assert_eq!(pop.polities[0].body, elders, "the custom stands");
    assert!(
        pop.polities[0]
            .versions
            .iter()
            .all(|v| v.seized_by.is_none())
    );
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == civ_agents::ChronicleKind::RevoltFailed),
        "the chronicle tells it"
    );
    let lines = civ_sim::frames::government::revolt_words(&sim, called.settlement);
    assert!(
        lines.iter().any(|w| w.contains("it came to nothing")),
        "{lines:?}"
    );
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn a_body_that_took_the_deciding_weighs_ending_the_laws_the_old_custom_made() {
    use civ_agents::faction::RevoltEnd;
    let (mut sim, called, _) = revolt_called(5);
    for _ in 0..(called.until - sim.now().day_index() + 2) {
        sim.advance_minutes(DAY).expect("advances");
        if sim.people().factions.revolts[0].ended.is_some() {
            break;
        }
    }
    let r = &sim.people().factions.revolts[0];
    assert_eq!(r.ended.map(|e| e.0), Some(RevoltEnd::Held), "{r:?}");
    // The old custom had made a curfew and a heavy levy, from households that hold plenty: what
    // the levy takes, they expect never to need back, and the curfew keeps their grown members
    // home where nothing is taken.
    let policy_of = |sim: &Sim, kind: PolicyKind| {
        sim.rules()
            .catalog
            .policies
            .iter()
            .position(|p| p.kind == kind)
            .expect("in core") as u16
    };
    let (store, curfew) = (
        policy_of(&sim, PolicyKind::CommonStore),
        policy_of(&sim, PolicyKind::Curfew),
    );
    let grain = sim
        .rules()
        .catalog
        .good_index("core:good/grain")
        .expect("in core");
    let next = sim.ids().peek_next() + 3_000_000;
    let pop = sim.people_mut_for_tests();
    let taken = pop.polities[0].versions.last().expect("versions").since;
    let before = civ_core::SimTime::from_minutes(called.called.minutes() - DAY);
    let sponsor = called.organizer;
    let law = |id: u64, policy: u16, kind: PolicyKind, levy_share: f32, hours: (u8, u8)| Law {
        id: PermanentId::from_raw(id).expect("non-zero"),
        policy,
        kind,
        levy_share,
        holder: None,
        relief_days: if kind == PolicyKind::CommonStore {
            5.0
        } else {
            0.0
        },
        sanction: Default::default(),
        hours,
        status: LawStatus::InForce,
        sponsor,
        proposed: before,
        issue: IssueKind::FoodShort,
        meets_day: before.day_index(),
        decided: Some(before),
        outcome: Some(Outcome::Passed),
        eligible: 0,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
    };
    let (levy, home) = (
        PermanentId::from_raw(next).expect("non-zero"),
        PermanentId::from_raw(next + 1).expect("non-zero"),
    );
    pop.polities[0]
        .laws
        .push(law(next, store, PolicyKind::CommonStore, 0.2, (0, 0)));
    pop.polities[0]
        .laws
        .push(law(next + 1, curfew, PolicyKind::Curfew, 0.0, (19, 7)));
    for (_, x) in pop.households.iter_mut() {
        x.stores.resize(grain + 1, 0.0);
        x.stores[grain] += 4000.0;
    }
    // Within the founding, the new body's members weigh ending each; some propose an end, and it
    // decides each as any law.
    let founding = i64::from(sim.rules().people.polity.founding_days);
    let mut put = Vec::new();
    for _ in 0..founding {
        sim.advance_minutes(DAY).expect("advances");
        put = sim.people().polities[0]
            .laws
            .iter()
            .filter(|l| l.issue == IssueKind::Founding && l.decided.is_some())
            .cloned()
            .collect();
        if put.len() >= 2 {
            break;
        }
    }
    assert!(
        !put.is_empty(),
        "an end to a law the old custom made was put"
    );
    let pop = sim.people();
    let p = &pop.polities[0];
    for l in &put {
        let ended = l.ends.expect("each names the law it would end");
        assert!(ended == levy || ended == home, "{l:?}");
        assert!(l.proposed >= taken, "proposed under the new custom");
        let old = p.laws.iter().find(|x| x.id == ended).expect("kept");
        let words = p.words_of(l, &sim.rules().catalog.policies, &|id| pop.name_of(id));
        if ended == home {
            assert_eq!(l.kind, PolicyKind::Repeal);
            assert_eq!(words, "an end to the curfew from 19:00 to 7:00");
        } else {
            assert_eq!((l.kind, l.levy_share), (PolicyKind::CommonStore, 0.0));
            assert!(
                words.starts_with("an end to the common store's levy"),
                "{words}"
            );
        }
        match l.outcome {
            Some(Outcome::Passed) => {
                assert_eq!(old.status, LawStatus::Superseded);
                let expect = if ended == home {
                    LawStatus::Carried
                } else {
                    LawStatus::InForce
                };
                assert_eq!(l.status, expect, "{l:?}");
            }
            _ => {
                assert_eq!(old.status, LawStatus::InForce);
                assert_eq!(l.status, LawStatus::Rejected);
            }
        }
        // Each was put once: what the new body leaves stands.
        assert_eq!(
            p.laws.iter().filter(|x| x.ends == Some(ended)).count(),
            1,
            "{ended}"
        );
    }
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn where_the_watch_is_several_one_of_them_may_take_the_deciding_for_it() {
    use civ_agents::faction::{RevoltEnd, Side};
    use civ_agents::polity::Membership;
    let (mut sim, adults) = aggrieved_village(5);
    let watch = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|p| p.kind == PolicyKind::KeepWatch)
        .expect("in core") as u16;
    let now = sim.now();
    let next = sim.ids().peek_next() + 4_000_000;
    let pop = sim.people_mut_for_tests();
    // Three keep the watch, each by a law of their own; nobody here holds that what the
    // gathering decides binds.
    let watchers: Vec<PermanentId> = adults.iter().copied().take(3).collect();
    for (k, &w) in watchers.iter().enumerate() {
        pop.polities[0].laws.push(Law {
            id: PermanentId::from_raw(next + k as u64).expect("non-zero"),
            policy: watch,
            kind: PolicyKind::KeepWatch,
            levy_share: 0.0,
            holder: Some(w),
            relief_days: 0.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: w,
            proposed: now,
            issue: IssueKind::Takings,
            meets_day: now.day_index(),
            decided: Some(now),
            outcome: Some(Outcome::Passed),
            eligible: 0,
            stances: Vec::new(),
            known: Vec::new(),
            compliance: Default::default(),
            watch: Default::default(),
            body: None,
            ends: None,
            agreement: None,
        });
    }
    for s in &mut pop.norms.states {
        s.endorse = 0.0;
        s.expect = 0.0;
    }
    let before = pop.polities[0].body;
    // At their monthly review, one of them calls on the others to take the deciding.
    let mut called = None;
    for _ in 0..40 {
        sim.advance_minutes(DAY).expect("advances");
        if let Some(c) = sim.people().factions.coups.first() {
            called = Some(c.clone());
            break;
        }
    }
    let called = called.expect("a coup was called");
    assert!(watchers.contains(&called.challenger), "{called:?}");
    assert_eq!(called.body.members, Membership::Watch);
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.kind == civ_agents::ChronicleKind::CoupCalled),
        "the chronicle tells it"
    );
    // Each day the watchers, and only they, take a side.
    for _ in 0..(called.until - sim.now().day_index() + 2) {
        sim.advance_minutes(DAY).expect("advances");
        let c = &sim.people().factions.coups[0];
        assert!(c.sides.iter().all(|s| watchers.contains(&s.0)), "{c:?}");
        if c.ended.is_some() {
            break;
        }
    }
    let pop = sim.people();
    let c = &pop.factions.coups[0];
    let (end, at) = c.ended.expect("it ended");
    let p = &pop.polities[0];
    let lines = civ_sim::frames::government::coup_words(&sim, called.settlement);
    // All three keep a grievance against the gathering and regard one another well: it holds a
    // week after the call.
    assert_eq!(end, RevoltEnd::Held, "{c:?}");
    match end {
        RevoltEnd::Held => {
            assert!(c.count(Side::With) > c.count(Side::Gathering), "{c:?}");
            assert_eq!(p.body, called.body, "the watch decides now");
            let v = p.versions.last().expect("versions");
            assert_eq!((v.since, v.seized_by), (at, Some(called.challenger)));
            // Its body is those who keep the watch.
            let members: Vec<PermanentId> = pop
                .body_members(&sim.land().fields, 0, sim.now(), &sim.rules().people)
                .into_iter()
                .map(|m| m.0)
                .collect();
            let mut keeping: Vec<PermanentId> = p.watchers().map(|w| w.0).collect();
            keeping.sort_unstable();
            assert_eq!(members, keeping);
            assert!(lines.iter().any(|w| w.contains("it held on")), "{lines:?}");
        }
        RevoltEnd::Failed => {
            assert_eq!(p.body, before, "the custom stands");
            assert!(
                lines.iter().any(|w| w.contains("it came to nothing")),
                "{lines:?}"
            );
        }
    }
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}
