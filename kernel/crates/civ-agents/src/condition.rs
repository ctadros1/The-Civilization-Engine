//! The condition of buildings (ADR-0009 §4, §6; M3b slice P): each component group's quality,
//! drawn once from the skill of those who built it when the stage that puts it in place is
//! finished, and its loss, which grows month by month with the exposure of its kind. A group
//! shows once enough is lost and fails once all of it is: a covering then leaks, the goods under
//! it keeping as in the open in proportion; posts that fail leave a ruin. Upkeep renews a share of
//! the worst group showing, with that share of the group's labour and materials, and changes only
//! that group (research 11-08: a new roof protects a wall without restoring what it lost).
//!
//! Nothing here decides who builds or mends what, or when.

use civ_core::{PermanentId, Rng64, SimTime};
use civ_grammar::{Expansion, Group, GroupKind, Stage, StageNeeds};
use civ_land::{Building, BuildingState, GroupCondition, GroupState, Repair};

use crate::params::{Upkeep, Wear};

/// Purpose tag of quality draws.
const PURPOSE_QUALITY: u64 = 0x7175_616c_6974_7931; // "quality1"

/// The building skill assumed where nobody's is known: work saved before buildings had a
/// condition, or a program that names no skill (a tuning value).
pub const MIDDLING_SKILL: f64 = 0.5;

/// The least quality a group is drawn at: a draw beyond it is a member that would not have been
/// used (a tuning value).
pub const MIN_QUALITY: f32 = 0.1;

/// The quality of group `group` of building `building`, made by builders of average building
/// skill `skill` (0 a novice, 1 a master): one less the spread at that skill (`spread`, a
/// novice's and a master's, interpolated) times the size of a normal draw. Skill buys fit and
/// consistency, an average nearer what sound members of the sizes carry and fewer bad ones,
/// never stronger wood (research 11-05 §1.3, 11-08 §1.2). Keyed by the world's seed, the building
/// and the group, so a draw is made once and never changes (11-05 §1.6).
pub fn draw_quality(
    seed: u64,
    building: PermanentId,
    group: u32,
    skill: f64,
    spread: [f64; 2],
) -> f32 {
    let s = skill.clamp(0.0, 1.0);
    let sigma = spread[0] + (spread[1] - spread[0]) * s;
    let mut rng = Rng64::from_key(&[seed, PURPOSE_QUALITY, building.get(), u64::from(group)]);
    // Box–Muller: the size of a standard normal draw.
    let u1 = rng.next_f64().max(f64::MIN_POSITIVE);
    let u2 = rng.next_f64();
    let z = ((-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()).abs();
    ((1.0 - sigma * z) as f32).clamp(MIN_QUALITY, 1.0)
}

/// The quality of group `group` of building `building` rebuilt whole at `at` after it gave way,
/// by a builder of building skill `skill`: new members, so a new draw ([`draw_quality`]), keyed
/// also by when, so each rebuilding is its own draw and the first build's draw is untouched.
pub fn draw_rebuilt_quality(
    seed: u64,
    building: PermanentId,
    group: u32,
    at: SimTime,
    skill: f64,
    spread: [f64; 2],
) -> f32 {
    let s = skill.clamp(0.0, 1.0);
    let sigma = spread[0] + (spread[1] - spread[0]) * s;
    let mut rng = Rng64::from_key(&[
        seed,
        PURPOSE_QUALITY,
        building.get(),
        u64::from(group),
        at.minutes() as u64,
    ]);
    let u1 = rng.next_f64().max(f64::MIN_POSITIVE);
    let u2 = rng.next_f64();
    let z = ((-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()).abs();
    ((1.0 - sigma * z) as f32).clamp(MIN_QUALITY, 1.0)
}

/// Puts in place the groups of `e` that stage `stage` builds, on building `b`, made at average
/// building skill `skill`, at `now` (ADR-0009 §4). A group already in place is left as it is.
pub fn install(
    b: &mut Building,
    e: &Expansion,
    stage: Stage,
    skill: f64,
    seed: u64,
    spread: [f64; 2],
    now: SimTime,
) {
    for g in e.groups.iter().filter(|g| g.stage == stage) {
        if b.group(g.id).is_some() {
            continue;
        }
        b.condition.push(GroupCondition {
            group: g.id,
            quality: draw_quality(seed, b.id, g.id, skill, spread),
            loss: 0.0,
            installed: now,
            repaired: now,
            state: GroupState::Sound,
        });
    }
}

/// Every group of `e` that the first `stages` stages build, in place on building `b` as if built
/// at `skill`: for a building that predates condition (a save from before slice P), sound as of
/// `now` (ADR-0009 §8).
pub fn install_through(
    b: &mut Building,
    e: &Expansion,
    stages: usize,
    skill: f64,
    seed: u64,
    spread: [f64; 2],
    now: SimTime,
) {
    for s in Stage::ALL.iter().take(stages) {
        install(b, e, *s, skill, seed, spread, now);
    }
}

/// Brings building `b`'s condition into line with its expansion `e` at `now` (on loading): a
/// group its expansion no longer has is dropped, with any repair of it, and every group of the
/// stages it has finished that has none is put in place sound, as if built at middling skill
/// ([`MIDDLING_SKILL`]), so a loaded world shows no false wave of decay (ADR-0009 §8). Whether
/// anything changed.
pub fn reconcile(
    b: &mut Building,
    e: &Expansion,
    seed: u64,
    spread: [f64; 2],
    now: SimTime,
) -> bool {
    let done = usize::from(b.stage).min(Stage::ALL.len());
    let before = b.condition.len();
    b.condition.retain(|c| {
        e.groups
            .iter()
            .any(|g| g.id == c.group && g.stage.index() < done)
    });
    let dropped = b.condition.len() != before;
    install_through(b, e, done, MIDDLING_SKILL, seed, spread, now);
    if b.repair.is_some_and(|r| b.group(r.group).is_none()) {
        b.repair = None;
    }
    let changed = dropped || b.condition.len() != before;
    if changed {
        settle_state(b);
    }
    changed
}

/// Brings the condition of every building in `buildings` into line with its program's
/// expansion in `catalog` ([`reconcile`]); a building whose program has left the content keeps
/// what it has.
pub fn reconcile_all(
    buildings: &mut [Building],
    catalog: &crate::params::Catalog,
    seed: u64,
    spread: [f64; 2],
    now: SimTime,
) {
    for b in buildings {
        let Some(def) = catalog
            .building_index(&b.spec.program)
            .and_then(|i| catalog.buildings.get(i))
        else {
            continue;
        };
        if let Ok(e) = civ_grammar::expand(&b.spec, &def.rules) {
            reconcile(b, &e, seed, spread, now);
        }
    }
}

/// How a group of kind `kind` wears, by `upkeep`: `None` for a kind that does not (a floor on the
/// ground).
pub fn wear_of(kind: GroupKind, upkeep: &Upkeep) -> Option<Wear> {
    match kind {
        GroupKind::Posts => Some(upkeep.posts),
        GroupKind::Covering => Some(upkeep.covering),
        GroupKind::Infill => Some(upkeep.infill),
        GroupKind::TieBeams
        | GroupKind::LoftJoists
        | GroupKind::FloorJoists
        | GroupKind::RaisedFloor
        | GroupKind::RoofFrame => Some(upkeep.under_leak),
        GroupKind::Floor => None,
    }
}

/// How near a loss must come to a threshold to have reached it: losses are kept to single
/// precision, and a month's steps summed there fall a hair short of the figure they add up to.
const REACHED: f64 = 1e-5;

/// The state a group of wear `w` is in at loss `loss`.
fn state_at(loss: f32, w: Wear) -> GroupState {
    let loss = f64::from(loss) + REACHED;
    if loss >= 1.0 {
        GroupState::Failed
    } else if loss >= w.shows_at {
        GroupState::Symptom
    } else {
        GroupState::Sound
    }
}

/// How group `c` stands by its wear alone, under `upkeep` (failed once it has failed by load
/// too, ADR-0009 §5).
pub fn wear_state(c: &GroupCondition, upkeep: &Upkeep) -> GroupState {
    if c.state == GroupState::Failed {
        return GroupState::Failed;
    }
    GroupKind::of_group(c.group)
        .and_then(|k| wear_of(k, upkeep))
        .map_or(GroupState::Sound, |w| state_at(c.loss, w))
}

/// How far a group of wear `w` has gone beyond the share at which it shows, over what was left to
/// lose then: 0 until it shows, 1 once all is lost.
fn beyond(loss: f32, w: Wear) -> f64 {
    ((f64::from(loss) - w.shows_at) / (1.0 - w.shows_at).max(1e-9)).clamp(0.0, 1.0)
}

/// How much of building `b`'s roof leaks, 0 to 1: each covering group's loss beyond the share
/// at which it leaks, over what is left to lose, averaged over its coverings (a frame's two
/// slopes each cover half its roof). A failed roof frame leaks fully; a ruin has no roof.
pub fn leak(b: &Building, upkeep: &Upkeep) -> f64 {
    if b.state == BuildingState::Ruin {
        return 1.0;
    }
    let roof_down = b.condition.iter().any(|c| {
        c.state == GroupState::Failed && GroupKind::of_group(c.group) == Some(GroupKind::RoofFrame)
    });
    if roof_down {
        return 1.0;
    }
    let (sum, n) = b
        .condition
        .iter()
        .filter(|c| GroupKind::of_group(c.group) == Some(GroupKind::Covering))
        .fold((0.0, 0usize), |(sum, n), c| {
            (sum + beyond(c.loss, upkeep.covering), n + 1)
        });
    if n == 0 { 0.0 } else { sum / n as f64 }
}

/// A month of wear on building `b` by `upkeep`, its posts set in ground of wetness `wetness`
/// (ADR-0009 §4): posts lose at their foot, coverings in the weather, infill at the wall's foot,
/// and roofed timber only as much as the roof over it leaks. Its groups' states and its own
/// follow. Whether any group's state, the building's or how far its roof leaks changed.
pub fn wear_month(b: &mut Building, upkeep: &Upkeep, wetness: f64) -> bool {
    if b.condition.is_empty() || b.state == BuildingState::Ruin {
        return false;
    }
    let before = (leak(b, upkeep), b.state, states(b));
    let leaking = before.0;
    for c in &mut b.condition {
        let Some(kind) = GroupKind::of_group(c.group) else {
            continue;
        };
        let Some(w) = wear_of(kind, upkeep) else {
            continue;
        };
        if c.state == GroupState::Failed {
            continue;
        }
        let factor = match kind {
            GroupKind::Posts => wetness.max(0.0),
            GroupKind::Covering | GroupKind::Infill => 1.0,
            _ => leaking,
        };
        let lost = w.per_year / 12.0 * factor;
        c.loss = (c.loss + lost as f32).min(1.0);
        // Its wear may make it worse than its load did, never better.
        c.state = worse(c.state, state_at(c.loss, w));
    }
    settle_state(b);
    let after = (leak(b, upkeep), b.state, states(b));
    after != before
}

/// The worse of two states.
pub fn worse(a: GroupState, b: GroupState) -> GroupState {
    let rank = |s: GroupState| match s {
        GroupState::Sound => 0,
        GroupState::Symptom => 1,
        GroupState::Failed => 2,
    };
    if rank(b) > rank(a) { b } else { a }
}

fn states(b: &Building) -> Vec<GroupState> {
    b.condition.iter().map(|c| c.state).collect()
}

/// A building's own state from its groups': a ruin once its posts fail, damaged once anything
/// else has failed, otherwise standing.
pub fn settle_state(b: &mut Building) {
    let failed = |k: GroupKind| {
        b.condition
            .iter()
            .any(|c| c.state == GroupState::Failed && GroupKind::of_group(c.group) == Some(k))
    };
    b.state = if failed(GroupKind::Posts) {
        BuildingState::Ruin
    } else if b.condition.iter().any(|c| c.state == GroupState::Failed) {
        BuildingState::Damaged
    } else {
        BuildingState::Standing
    };
}

/// The group upkeep would mend first on building `b`, and how far it has gone ([`beyond`]; 1 for
/// a failed group): of those its wear shows on or that have failed, the one gone furthest (ties:
/// the lower group id). `None` when nothing does, or for a ruin, which is not mended.
pub fn worst(b: &Building, upkeep: &Upkeep) -> Option<(u32, f64)> {
    if b.state == BuildingState::Ruin {
        return None;
    }
    let mut best: Option<(f64, u32)> = None;
    for c in &b.condition {
        // What its load alone shows (a sag, a lean) is not mended: only what is worn or broken.
        if wear_state(c, upkeep) == GroupState::Sound {
            continue;
        }
        let Some(w) = GroupKind::of_group(c.group).and_then(|k| wear_of(k, upkeep)) else {
            continue;
        };
        let over = if c.state == GroupState::Failed {
            1.0
        } else {
            beyond(c.loss, w)
        };
        let better = best
            .is_none_or(|(o, id)| over > o + 1e-12 || ((over - o).abs() <= 1e-12 && c.group < id));
        if better {
            best = Some((over, c.group));
        }
    }
    best.map(|(over, id)| (id, over))
}

/// The repair building `b` would have under way: the one begun, or else renewing all that is
/// lost of the worst group showing ([`worst`]), or all of it once it has failed. `None` when
/// nothing shows, or for a ruin.
pub fn repair_of(b: &Building, upkeep: &Upkeep) -> Option<Repair> {
    if !b.standing() {
        return None;
    }
    if let Some(r) = b.repair {
        return Some(r);
    }
    let (group, _) = worst(b, upkeep)?;
    let c = b.group(group)?;
    Some(Repair {
        group,
        share: if c.state == GroupState::Failed {
            1.0
        } else {
            c.loss
        },
        work_h: 0.0,
    })
}

/// Whether a group's members are timber, costed by their volume (otherwise by the area it covers).
fn timber(g: &Group) -> bool {
    !matches!(
        g.kind,
        GroupKind::Covering | GroupKind::Infill | GroupKind::Floor
    )
}

/// A group's size, to share out its stage's labour and materials: the volume of its members for
/// timber, the area it covers otherwise.
fn size(g: &Group) -> f64 {
    if timber(g) {
        let [w, d] = g.section_cm.map(f64::from);
        f64::from(g.count) * w.max(0.0) * d.max(0.0) * f64::from(g.length_cm).max(0.0)
    } else {
        g.area_m2.max(0.0)
    }
}

/// What renewing `share` of group `group` of expansion `e` needs: that share of the group's
/// part of its stage's labour and materials, the group's part being its size over that of every
/// group of like measure its stage puts in place. `None` if `e` has no such group.
pub fn mend_needs(e: &Expansion, group: u32, share: f64) -> Option<StageNeeds> {
    let g = e.groups.iter().find(|g| g.id == group)?;
    let stage = e.stages.get(g.stage.index())?;
    let total: f64 = e
        .groups
        .iter()
        .filter(|o| o.stage == g.stage && timber(o) == timber(g))
        .map(size)
        .sum();
    let part = if total > 0.0 { size(g) / total } else { 0.0 };
    let f = part * share.clamp(0.0, 1.0);
    Some(StageNeeds {
        stage: g.stage,
        labour_h: stage.labour_h * f,
        materials_kg: stage.materials_kg.iter().map(|kg| kg * f).collect(),
    })
}

/// Group `group` of building `b` is mended at `now`: `share` of its section or covering is
/// renewed, its state and the building's follow, and the repair under way is done. A worn group
/// keeps its quality, the new members fitted among the old; a group rebuilt whole after it gave
/// way takes `rebuilt`, the quality of its new members ([`draw_rebuilt_quality`]).
pub fn mend(
    b: &mut Building,
    group: u32,
    share: f32,
    upkeep: &Upkeep,
    now: SimTime,
    rebuilt: Option<f32>,
) {
    b.repair = None;
    let Some(c) = b.condition.iter_mut().find(|c| c.group == group) else {
        return;
    };
    if let Some(q) = rebuilt {
        c.quality = q;
    }
    c.loss = (c.loss - share).max(0.0);
    c.repaired = now;
    // Renewed, it stands by its wear until its load is next weighed.
    c.state = GroupKind::of_group(group)
        .and_then(|k| wear_of(k, upkeep))
        .map_or(GroupState::Sound, |w| state_at(c.loss, w));
    settle_state(b);
}

/// The room building `b` gives its household's goods, `room` kilograms by kind as its design
/// gives them, as its condition leaves it: none in a ruin, and of the rest what the roof keeps
/// dry; a loft whose joists failed holds nothing.
pub fn room_left(b: &Building, upkeep: &Upkeep, e: &Expansion, room: [f64; 3]) -> [f64; 3] {
    use civ_grammar::storage;
    if b.state == BuildingState::Ruin {
        return [0.0; 3];
    }
    let dry = 1.0 - leak(b, upkeep);
    let lofts: Vec<&Group> = e
        .groups
        .iter()
        .filter(|g| g.kind == GroupKind::LoftJoists)
        .collect();
    let fallen = lofts
        .iter()
        .filter(|g| b.group(g.id).is_some_and(|c| c.state == GroupState::Failed))
        .count();
    let mut out = room.map(|r| r * dry);
    if !lofts.is_empty() {
        out[storage::LOFT] *= 1.0 - fallen as f64 / lofts.len() as f64;
    }
    // A raised floor or an upper storey's floor that gave way holds nothing on its bays.
    let share_down = |kind: GroupKind| {
        let decks: Vec<&Group> = e.groups.iter().filter(|g| g.kind == kind).collect();
        let down = decks
            .iter()
            .filter(|g| b.group(g.id).is_some_and(|c| c.state == GroupState::Failed))
            .count();
        if decks.is_empty() {
            0.0
        } else {
            down as f64 / decks.len() as f64
        }
    };
    out[storage::RAISED] *= 1.0 - share_down(GroupKind::RaisedFloor);
    let upper: f64 = e
        .spaces
        .iter()
        .filter(|s| s.level == civ_grammar::Level::Upper)
        .map(|s| s.area_m2)
        .sum();
    let floors: f64 = e
        .spaces
        .iter()
        .filter(|s| {
            matches!(
                s.level,
                civ_grammar::Level::Ground | civ_grammar::Level::Upper
            )
        })
        .map(|s| s.area_m2)
        .sum();
    if upper > 0.0 && floors > 0.0 {
        out[storage::FLOOR] *= 1.0 - share_down(GroupKind::FloorJoists) * upper / floors;
    }
    out
}

/// Words for what building `b` shows, worst first: "the thatch leaks; rot at the posts' foot".
/// Empty when nothing shows.
pub fn symptoms(b: &Building, upkeep: &Upkeep) -> String {
    if b.state == BuildingState::Ruin {
        return "a ruin: its posts gave way".to_owned();
    }
    let mut shown: Vec<(f64, &'static str)> = Vec::new();
    for c in &b.condition {
        let Some(kind) = GroupKind::of_group(c.group) else {
            continue;
        };
        let Some(w) = wear_of(kind, upkeep) else {
            continue;
        };
        // What its load alone shows: it sags, or leans.
        let loaded = c.state == GroupState::Symptom && state_at(c.loss, w) == GroupState::Sound;
        let words = match (kind, c.state) {
            (_, GroupState::Sound) => continue,
            (GroupKind::LoftJoists, _) if loaded => "the loft sags",
            (GroupKind::RoofFrame, _) if loaded => "the roof sags",
            (GroupKind::Posts, _) if loaded => "the posts lean",
            (GroupKind::FloorJoists | GroupKind::RaisedFloor, _) if loaded => "the floor sags",
            (_, _) if loaded => "a beam sags",
            (GroupKind::RaisedFloor | GroupKind::FloorJoists, GroupState::Failed) => {
                "a floor gave way"
            }
            (GroupKind::TieBeams, GroupState::Failed) => "a beam broke",
            (GroupKind::Covering, GroupState::Failed) => "the thatch is gone",
            (GroupKind::Covering, _) => "the thatch leaks",
            (GroupKind::Posts, GroupState::Failed) => "the posts gave way",
            (GroupKind::Posts, _) => "rot at the posts' foot",
            (GroupKind::Infill, GroupState::Failed) => "the walls are open",
            (GroupKind::Infill, _) => "the daub is worn",
            (GroupKind::RoofFrame, GroupState::Failed) => "the roof fell in",
            (GroupKind::LoftJoists, GroupState::Failed) => "a loft fell",
            (_, GroupState::Failed) => "timber gave way",
            (_, _) => "rot in the roofed timber",
        };
        let over = if loaded { 0.0 } else { beyond(c.loss, w) };
        match shown.iter_mut().find(|(_, s)| *s == words) {
            Some(seen) => seen.0 = seen.0.max(over),
            None => shown.push((over, words)),
        }
    }
    shown.sort_by(|a, b| b.0.total_cmp(&a.0));
    shown
        .into_iter()
        .map(|(_, s)| s)
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_grammar::{BuildingSpec, Footprint, HUT_VERSION, PARAMS, hut_params};

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("nonzero")
    }

    fn upkeep() -> Upkeep {
        Upkeep {
            covering: Wear {
                per_year: 0.06,
                shows_at: 0.15,
            },
            posts: Wear {
                per_year: 0.02,
                shows_at: 0.3,
            },
            infill: Wear {
                per_year: 0.05,
                shows_at: 0.2,
            },
            under_leak: Wear {
                per_year: 0.1,
                shows_at: 0.3,
            },
        }
    }

    fn hut() -> (Building, Expansion) {
        let rules = crate::build::tests::hut_rules();
        let mut params = [0; PARAMS];
        params[hut_params::EAVE_CM] = 180;
        params[hut_params::PITCH_CENTIDEG] = 4500;
        let spec = BuildingSpec {
            program: "core:building/hut".into(),
            version: HUT_VERSION,
            footprint: Footprint::Round {
                x: 10_000,
                y: 10_000,
                radius: 300,
            },
            storeys: 1,
            params,
            materials: vec!["t".into(), "t".into(), "h".into()],
            style_seed: 0,
        };
        let e = civ_grammar::expand(&spec, &civ_grammar::ProgramRules::Hut(rules)).expect("a hut");
        let mut b = Building::new(id(5), id(2), id(3), spec, SimTime::ZERO);
        b.stage = Stage::ALL.len() as u8;
        install_through(
            &mut b,
            &e,
            Stage::ALL.len(),
            0.5,
            7,
            [0.3, 0.1],
            SimTime::ZERO,
        );
        (b, e)
    }

    fn group_of(e: &Expansion, kind: GroupKind) -> u32 {
        e.groups
            .iter()
            .find(|g| g.kind == kind)
            .map(|g| g.id)
            .expect("a group of the kind")
    }

    #[test]
    fn skill_buys_consistency_not_strength() {
        let draws = |skill: f64| -> Vec<f32> {
            (0..2000)
                .map(|g| draw_quality(11, id(3), g, skill, [0.3, 0.1]))
                .collect()
        };
        let (novice, master) = (draws(0.0), draws(1.0));
        let mean = |v: &[f32]| v.iter().map(|&q| f64::from(q)).sum::<f64>() / v.len() as f64;
        let worst = |v: &[f32]| v.iter().copied().fold(1.0f32, f32::min);
        assert!(
            novice
                .iter()
                .chain(&master)
                .all(|&q| (MIN_QUALITY..=1.0).contains(&q))
        );
        // A half-normal's mean is 0.8 of its sigma: about 0.76 for a novice, 0.92 for a master.
        assert!((mean(&novice) - 0.76).abs() < 0.02, "{}", mean(&novice));
        assert!((mean(&master) - 0.92).abs() < 0.01, "{}", mean(&master));
        assert!(worst(&master) > 0.55 && worst(&novice) < 0.2);
        // A draw is made once: the same group of the same building draws the same.
        assert_eq!(
            draw_quality(11, id(3), 4, 0.5, [0.3, 0.1]),
            draw_quality(11, id(3), 4, 0.5, [0.3, 0.1])
        );
    }

    #[test]
    fn a_part_rebuilt_after_it_gave_way_is_a_new_draw_and_a_worn_one_keeps_its_quality() {
        let spread = [0.3, 0.1];
        let first = draw_quality(7, id(9), 3, 0.2, spread);
        let at = SimTime::from_minutes(1_000);
        let again = draw_rebuilt_quality(7, id(9), 3, at, 0.2, spread);
        assert_eq!(again, draw_rebuilt_quality(7, id(9), 3, at, 0.2, spread));
        assert_ne!(again, first, "a new draw");
        let later = SimTime::from_minutes(2_000);
        assert_ne!(again, draw_rebuilt_quality(7, id(9), 3, later, 0.2, spread));
        // Over many rebuildings a novice's parts are as uneven as their first ones.
        let mean = (0..2_000)
            .map(|t| {
                f64::from(draw_rebuilt_quality(
                    7,
                    id(9),
                    3,
                    SimTime::from_minutes(t),
                    0.0,
                    spread,
                ))
            })
            .sum::<f64>()
            / 2_000.0;
        assert!(
            (mean - (1.0 - 0.3 * (2.0 / std::f64::consts::PI).sqrt())).abs() < 0.02,
            "{mean}"
        );
        // Mending sets a rebuilt part's quality and leaves a worn one's.
        let (mut b, e) = hut();
        let u = upkeep();
        let covering = group_of(&e, GroupKind::Covering);
        let q = b.group(covering).expect("in place").quality;
        mend(&mut b, covering, 0.1, &u, SimTime::from_minutes(1), None);
        assert_eq!(b.group(covering).expect("in place").quality, q);
        mend(
            &mut b,
            covering,
            1.0,
            &u,
            SimTime::from_minutes(2),
            Some(0.42),
        );
        assert_eq!(b.group(covering).expect("in place").quality, 0.42);
    }

    #[test]
    fn a_finished_hut_has_its_five_groups_in_place_and_sound() {
        let (b, e) = hut();
        assert_eq!(e.groups.len(), 5);
        assert_eq!(b.condition.len(), e.groups.len());
        assert!(
            b.condition
                .iter()
                .all(|c| c.state == GroupState::Sound && c.loss == 0.0)
        );
        assert_eq!(leak(&b, &upkeep()), 0.0);
        assert_eq!(worst(&b, &upkeep()), None);
        assert_eq!(symptoms(&b, &upkeep()), "");
    }

    #[test]
    fn thatch_leaks_after_about_two_and_a_half_years_and_rots_the_roof_beneath_untended() {
        let (mut b, e) = hut();
        let u = upkeep();
        let covering = group_of(&e, GroupKind::Covering);
        let mut months = 0;
        while b.group(covering).expect("in place").state == GroupState::Sound {
            wear_month(&mut b, &u, 1.0);
            months += 1;
        }
        // 0.15 at 0.06 a year: 30 months (11-08 §2.4: significant mending every 2–3 years).
        assert_eq!(months, 30);
        assert!(leak(&b, &u) < 1e-6, "just showing, barely leaking");
        assert_eq!(worst(&b, &u).map(|(g, _)| g), Some(covering));
        assert!(symptoms(&b, &u).contains("the thatch leaks"));
        // Untended for years more, the leak grows and the rafters beneath it begin to rot.
        for _ in 0..60 {
            wear_month(&mut b, &u, 1.0);
        }
        let l = leak(&b, &u);
        assert!(l > 0.2 && l < 0.6, "{l}");
        let rafters = group_of(&e, GroupKind::RoofFrame);
        assert!(b.group(rafters).expect("in place").loss > 0.0);
        // The room under it shrinks with the leak.
        let room = room_left(&b, &u, &e, [0.0, 0.0, 3000.0]);
        assert!((room[2] - 3000.0 * (1.0 - l)).abs() < 1e-6);
        // Mending renews the thatch: dry again, and the rafters keep what they lost.
        let lost = b.group(covering).expect("in place").loss;
        mend(&mut b, covering, lost, &u, SimTime::from_minutes(1), None);
        assert_eq!(leak(&b, &u), 0.0);
        assert!(
            b.group(rafters).expect("in place").loss > 0.0,
            "upkeep changes only what it touches"
        );
    }

    #[test]
    fn posts_rot_faster_in_wet_ground_and_a_ruin_shelters_nothing() {
        let (mut wet, e) = hut();
        let (mut dry, _) = hut();
        let u = upkeep();
        let posts = group_of(&e, GroupKind::Posts);
        for _ in 0..12 * 10 {
            wear_month(&mut wet, &u, 2.0);
            wear_month(&mut dry, &u, 0.7);
        }
        let (w, d) = (
            wet.group(posts).expect("posts").loss,
            dry.group(posts).expect("posts").loss,
        );
        assert!((w - 0.4).abs() < 1e-3 && (d - 0.14).abs() < 1e-3, "{w} {d}");
        assert_eq!(wet.group(posts).expect("posts").state, GroupState::Symptom);
        assert!(symptoms(&wet, &u).contains("rot at the posts' foot"));
        // Untended, wet posts give way in their twenty-fifth year: a ruin.
        for _ in 0..12 * 15 {
            wear_month(&mut wet, &u, 2.0);
        }
        assert_eq!(wet.state, BuildingState::Ruin);
        assert_eq!(room_left(&wet, &u, &e, [0.0, 0.0, 3000.0]), [0.0; 3]);
        assert_eq!(worst(&wet, &u), None, "a ruin is not mended");
    }

    #[test]
    fn mending_a_group_takes_its_share_of_its_stage() {
        let (_, e) = hut();
        let covering = group_of(&e, GroupKind::Covering);
        let roof = &e.stages[Stage::Roof.index()];
        let half = mend_needs(&e, covering, 0.5).expect("needs");
        assert!((half.labour_h - roof.labour_h / 2.0).abs() < 1e-9);
        for (a, b) in half.materials_kg.iter().zip(&roof.materials_kg) {
            assert!((a - b / 2.0).abs() < 1e-9);
        }
        // Posts are a share of the frame's timber, with the rafters.
        let posts = group_of(&e, GroupKind::Posts);
        let frame = &e.stages[Stage::Frame.index()];
        let all = mend_needs(&e, posts, 1.0).expect("needs");
        assert!(all.labour_h > 0.0 && all.labour_h < frame.labour_h);
    }
}
