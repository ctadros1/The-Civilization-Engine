//! Buildings weighed against what they carry, each day (ADR-0009 §5; M3b slice P): the goods in
//! their lofts and on their floors, people on floors off the ground, their roofs' covering and the
//! heaviest wind and snow of the month. A group shows when its margin runs low or it sags, and
//! gives way below a margin of 1: a loft or a floor drops what it holds, a broken tie beam brings
//! its lofts down, rafters that break bring the roof in, and posts that give way leave a ruin.
//! Those inside may die. Nothing here decides what anyone stores, builds or mends.

use super::*;
use crate::history::Cause;
use crate::person::Keeping;
use crate::structure::{self, G, Loads};
use civ_grammar::{Group, GroupKind, Level, storage};
use civ_land::{BuildingState, GroupState};

/// Keyed-randomness purpose of a settlement's heaviest wind and snow in a month.
const PURPOSE_PEAK: u64 = 0x7065_616b_6c6f_6164; // "peakload"

/// Keyed-randomness purpose of who dies when a building gives way.
const PURPOSE_COLLAPSE: u64 = 0x636f_6c6c_6170_7365; // "collapse"

/// People and what they carry on a floor off the ground, pascals: research 11-05 §2.4's occupied
/// dwelling floor (1.5–2 kPa), a proposed scenario rather than a historical figure.
pub const LIVE_PA: f64 = 1500.0;

/// The share of the goods on a loft or floor that gives way that is lost, spilled and spoiled in
/// the wreckage (a tuning value).
pub const SPILLED: f64 = 0.3;

/// The chance that someone inside dies when the posts give way and the building falls, when its
/// roof falls in, and when a loft or a floor gives way: tuning values, as the research gives no
/// defensible share (11-06 §5.3: those on, below and beside the failed part are at risk).
pub const DIES_IN_RUIN: f64 = 0.3;
/// See [`DIES_IN_RUIN`].
pub const DIES_UNDER_ROOF: f64 = 0.15;
/// See [`DIES_IN_RUIN`].
pub const DIES_UNDER_FLOOR: f64 = 0.05;

/// The heaviest wind and snow on the roofs of settlement (or lone household) `key` in the month
/// of `now`: the day of the month it comes (1 to 28) and its load, pascals on their plan. One
/// draw a month, the same for every roof there (ADR-0009 §5: a shared event, evaluated once and
/// never rerolled).
pub fn month_peak(
    seed: u64,
    key: PermanentId,
    now: SimTime,
    peak: &civ_land::PeakLoad,
) -> (u8, f64) {
    let d = now.date();
    let month = (d.year * 12 + i64::from(d.month)) as u64;
    let mut rng = Rng64::from_key(&[seed, PURPOSE_PEAK, key.get(), month]);
    let u1 = rng.next_f64().max(f64::MIN_POSITIVE);
    let u2 = rng.next_f64();
    let z = (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos();
    let day = 1 + (rng.next_f64() * 28.0).floor().clamp(0.0, 27.0) as u8;
    (
        day,
        peak.median_pa.max(0.0) * (peak.spread.max(0.0) * z).exp(),
    )
}

/// The month's heaviest wind and snow on the roofs of settlement (or lone household) `key` on
/// the day of `now`, pascals on their plan: [`month_peak`]'s load on its day, and none on the
/// others. A roof mended after a storm does not meet the same storm again.
pub fn peak_pa(seed: u64, key: PermanentId, now: SimTime, peak: &civ_land::PeakLoad) -> f64 {
    let (day, pa) = month_peak(seed, key, now, peak);
    if now.date().day == day { pa } else { 0.0 }
}

/// A mass in words: "850 kg", "1.2 t".
fn mass(kg: f64) -> String {
    if kg < 1_000.0 {
        format!("{:.0} kg", kg.max(0.0))
    } else {
        format!("{:.1} t", kg / 1_000.0)
    }
}

/// What a household keeps in one building now, kilograms of each good by kind of room
/// ([`storage`]): each kind of room's goods shared among its buildings by the room each gives.
fn kept_in(fill: &[Vec<f64>; 3], room_all: [f64; 3], room: [f64; 3]) -> [Vec<f64>; 3] {
    let mut out: [Vec<f64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for kind in 0..storage::KINDS {
        let share = if room_all[kind] > 0.0 && room[kind].is_finite() {
            (room[kind] / room_all[kind]).clamp(0.0, 1.0)
        } else {
            0.0
        };
        out[kind] = fill[kind].iter().map(|kg| kg * share).collect();
    }
    out
}

/// A group of a building that gave way today, and why it was weak.
struct Fell {
    kind: GroupKind,
    quality: f32,
    loss: f32,
}

impl Population {
    /// Weighs every standing building with groups in place against what it carries now
    /// (ADR-0009 §5). Each household's stores are settled first, and its shelter derived again
    /// when anything of its gave way.
    pub(super) fn check_buildings(&mut self, ctx: &mut Ctx) {
        let mut owners: Vec<PermanentId> = ctx
            .land
            .buildings
            .iter()
            .filter(|b| !b.condition.is_empty() && b.standing())
            .map(|b| b.household)
            .collect();
        owners.sort_unstable();
        owners.dedup();
        for household in owners {
            self.check_household(ctx, household);
        }
    }

    /// Weighs household `household`'s buildings against what they carry now.
    fn check_household(&mut self, ctx: &mut Ctx, household: PermanentId) {
        let (now, catalog, params) = (ctx.now, ctx.catalog, ctx.params);
        let goods = &catalog.goods;
        // Its goods as they lie now, by kind of room, across its roofs.
        let hd = self.hh_index.get(&household).copied();
        if let Some(hd) = hd
            && let Some(x) = self.households.get_mut(hd)
        {
            let members = x.members.len().max(1);
            x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
            x.stores.resize(goods.len(), 0.0);
        }
        let mine: Vec<usize> = ctx
            .land
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.household == household)
            .map(|(i, _)| i)
            .collect();
        let rooms: Vec<[f64; 3]> = mine
            .iter()
            .map(|&i| room_of(&ctx.land.buildings[i], catalog))
            .collect();
        let mut room_all = [0.0; 3];
        for r in &rooms {
            for (a, v) in room_all.iter_mut().zip(r) {
                if v.is_finite() {
                    *a += v;
                }
            }
        }
        let stores = hd
            .and_then(|hd| self.households.get(hd))
            .map(|x| x.stores.clone())
            .unwrap_or_default();
        let fill = Keeping::fill(&stores, goods, room_all);
        let settlement = hd
            .and_then(|hd| self.households.get(hd))
            .and_then(|x| x.settlement)
            .unwrap_or(household);
        let peak = peak_pa(ctx.seed, settlement, now, &ctx.land_params.peak_load);
        let mut changed = false;
        for (&i, &room) in mine.iter().zip(&rooms) {
            let kept = kept_in(&fill, room_all, room);
            if let Some(fell) = self.weigh(ctx, i, &kept, peak) {
                changed = true;
                self.give_way(ctx, i, &kept, &fell, peak);
            }
        }
        if changed {
            self.buildings_changed(now, ctx.land, catalog, params, household);
        }
    }

    /// Weighs building `i`, which holds `kept` of its household's goods, under the month's peak
    /// `peak` pascals: each group's state follows its wear and its margin. What gave way today,
    /// the gravest first; `None` if nothing did.
    fn weigh(
        &mut self,
        ctx: &mut Ctx,
        i: usize,
        kept: &[Vec<f64>; 3],
        peak: f64,
    ) -> Option<Vec<Fell>> {
        let catalog = ctx.catalog;
        let b = &ctx.land.buildings[i];
        if b.condition.is_empty() || !b.standing() {
            return None;
        }
        let def = catalog
            .building_index(&b.spec.program)
            .and_then(|d| catalog.buildings.get(d))?;
        let e = self.expansion_of(b, def)?;
        let plot = ctx.land.plots.iter().find(|p| p.id == b.plot)?;
        let cx = (plot.rect.x as f32 / 100.0 / ctx.map.cell_size_m).max(0.0) as usize;
        let cy = (plot.rect.y as f32 / 100.0 / ctx.map.cell_size_m).max(0.0) as usize;
        let cx = cx.min(ctx.map.width as usize - 1);
        let cy = cy.min(ctx.map.height as usize - 1);
        let z = ctx.map.elevation[cy * (ctx.map.width as usize) + cx];
        let snow_mm = ctx.land.weather.snow_at(z as f64);
        let snow_pa = snow_mm * G;
        let loads = loads_on(b, &e, kept);
        let loads = Loads {
            peak_pa: if loads.covering_pa > 0.0 {
                peak + snow_pa
            } else {
                0.0
            },
            ..loads
        };
        let timber = |g: &Group| {
            g.material
                .and_then(|slot| def.materials.get(usize::from(slot)))
                .and_then(|&good| catalog.goods.get(good))
                .and_then(|good| good.timber)
        };
        let b = &mut ctx.land.buildings[i];
        let mut fell: Vec<Fell> = Vec::new();
        for c in b.condition.iter_mut() {
            if c.state == GroupState::Failed {
                continue;
            }
            let Some(g) = e.groups.iter().find(|g| g.id == c.group) else {
                continue;
            };
            let wear = condition::wear_state(c, &def.upkeep);
            let load = timber(g).map_or(GroupState::Sound, |t| {
                structure::margin(g, c, &e, &loads, &t).state()
            });
            c.state = condition::worse(wear, load);
            if c.state == GroupState::Failed {
                fell.push(Fell {
                    kind: g.kind,
                    quality: c.quality,
                    loss: c.loss,
                });
            }
        }
        if fell.is_empty() {
            return None;
        }
        // A tie beam at the wall heads that breaks brings down the lofts it carries.
        let beams_down = b.condition.iter().any(|c| {
            c.state == GroupState::Failed
                && GroupKind::of_group(c.group) == Some(GroupKind::TieBeams)
                && (c.group >> 24) & 0xff == u32::from(Level::Roof.code())
        });
        if beams_down {
            for c in b.condition.iter_mut() {
                if GroupKind::of_group(c.group) == Some(GroupKind::LoftJoists)
                    && c.state != GroupState::Failed
                {
                    c.state = GroupState::Failed;
                    fell.push(Fell {
                        kind: GroupKind::LoftJoists,
                        quality: c.quality,
                        loss: c.loss,
                    });
                }
            }
        }
        condition::settle_state(b);
        fell.sort_by_key(|f| gravity(f.kind));
        Some(fell)
    }

    /// What follows when parts of building `i`, which held `kept` of its household's goods, gave
    /// way (`fell`, the gravest first) under the month's peak `peak`: the goods on what fell are
    /// spilled and part lost, those inside may die, and the chronicle tells of it.
    fn give_way(
        &mut self,
        ctx: &mut Ctx,
        i: usize,
        kept: &[Vec<f64>; 3],
        fell: &[Fell],
        peak: f64,
    ) {
        let (now, catalog) = (ctx.now, ctx.catalog);
        let goods = &catalog.goods;
        let b = &ctx.land.buildings[i];
        let (id, household) = (b.id, b.household);
        let Some(def) = catalog
            .building_index(&b.spec.program)
            .and_then(|d| catalog.buildings.get(d))
        else {
            return;
        };
        let Some(e) = self.expansion_of(b, def) else {
            return;
        };
        let (at, technique) = (build::centre_m(&b.spec), def.technique);
        let reach_m = match b.spec.footprint {
            civ_grammar::Footprint::Round { radius, .. } => f64::from(radius) / 100.0,
            civ_grammar::Footprint::Rect { length, width, .. } => {
                (f64::from(length) / 2.0).hypot(f64::from(width) / 2.0) / 100.0
            }
        };
        // Goods on the lofts and floors that fell: each deck's share of its kind.
        let decks = |kind: GroupKind| e.groups.iter().filter(|g| g.kind == kind).count().max(1);
        let upper_share = {
            let area = |level: Level| -> f64 {
                e.spaces
                    .iter()
                    .filter(|s| s.level == level)
                    .map(|s| s.area_m2)
                    .sum()
            };
            let (ground, upper) = (area(Level::Ground), area(Level::Upper));
            if ground + upper > 0.0 {
                upper / (ground + upper)
            } else {
                0.0
            }
        };
        let mut spilled = vec![0.0; goods.len()];
        for f in fell {
            let (kind, share) = match f.kind {
                GroupKind::LoftJoists => (storage::LOFT, 1.0 / decks(GroupKind::LoftJoists) as f64),
                GroupKind::RaisedFloor => {
                    (storage::RAISED, 1.0 / decks(GroupKind::RaisedFloor) as f64)
                }
                GroupKind::FloorJoists => (
                    storage::FLOOR,
                    upper_share / decks(GroupKind::FloorJoists) as f64,
                ),
                _ => continue,
            };
            for (s, kg) in spilled.iter_mut().zip(&kept[kind]) {
                *s += kg * share;
            }
        }
        let fallen_kg: f64 = spilled.iter().sum();
        let mostly = spilled
            .iter()
            .enumerate()
            .filter(|(_, kg)| **kg > 0.0)
            .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)))
            .and_then(|(g, _)| goods.get(g))
            .map(|g| g.name.to_lowercase());
        if let Some(x) = self
            .hh_index
            .get(&household)
            .and_then(|&hd| self.households.get_mut(hd))
        {
            for (g, kg) in spilled.iter().enumerate() {
                let held = x.stores.get(g).copied().unwrap_or(0.0).max(0.0);
                let lost = (kg * SPILLED).min(held);
                if lost > 0.0 {
                    x.stores[g] -= lost;
                    x.flows.add(Flow::Spoiled, g, lost);
                }
            }
        }
        // Those inside: everyone not on their way somewhere, within the building's reach.
        let gravest = fell.first().map(|f| f.kind);
        let risk = match gravest {
            Some(GroupKind::Posts) => DIES_IN_RUIN,
            Some(GroupKind::RoofFrame) => DIES_UNDER_ROOF,
            _ => DIES_UNDER_FLOOR,
        };
        let t = now.minutes() as f64;
        let inside: Vec<PermanentId> = self
            .people
            .iter()
            .filter(|(_, p)| p.trip.is_none())
            .filter(|(_, p)| {
                let (x, y) = p.position_at(t);
                f64::from(x - at.0).hypot(f64::from(y - at.1)) <= reach_m
            })
            .map(|(_, p)| p.id)
            .collect();
        let day = now.day_index() as u64;
        let dead: Vec<PermanentId> = inside
            .into_iter()
            .filter(|who| {
                let key = [ctx.seed, PURPOSE_COLLAPSE, id.get(), who.get(), day];
                Rng64::from_key(&key).next_f64() < risk
            })
            .collect();
        // The chronicle: what gave way, under what, and why it was weak.
        let what = match gravest {
            Some(GroupKind::Posts) => "fell: its posts gave way",
            Some(GroupKind::RoofFrame) => "lost its roof: the rafters broke",
            Some(GroupKind::TieBeams) => "lost its lofts: a tie beam broke",
            Some(GroupKind::LoftJoists) => "lost its loft: the joists broke",
            Some(GroupKind::RaisedFloor) => "lost its raised floor: the joists broke",
            Some(GroupKind::FloorJoists) => "lost its upper floor: the joists broke",
            _ => "gave way",
        };
        let mut words = format!("{} {what}", def.name.to_lowercase());
        if fallen_kg >= 1.0 {
            words.push_str(&format!(" under {}", mass(fallen_kg)));
            if let Some(m) = mostly {
                words.push_str(&format!(" of {m}"));
            }
        } else if peak > 0.0 && peak >= 2.0 * ctx.land_params.peak_load.median_pa {
            words.push_str(" in a storm");
        }
        if let Some(f) = fell.first() {
            match (f.quality < 0.6, f.loss >= 0.25) {
                (true, true) => words.push_str("; they were poorly made and had rotted"),
                (true, false) => words.push_str("; they were poorly made"),
                (false, true) => words.push_str("; they had rotted"),
                (false, false) => {}
            }
        }
        let owner = self
            .household(household)
            .and_then(|x| x.members.first().copied());
        let settlement = self.household(household).and_then(|x| x.settlement);
        let mut people: Vec<PermanentId> = owner.into_iter().collect();
        people.extend(dead.iter().copied());
        self.chronicle_push(
            now,
            ChronicleKind::BuildingFailed,
            people,
            settlement,
            Some(at),
            dead.len() as f64,
            words,
        );
        // The settlement remembers it against the technique (ADR-0009 §6).
        let caution = &ctx.params.build.caution;
        self.remember_failure(settlement, technique, dead.len(), now, caution);
        for who in dead {
            self.die(ctx, who, Cause::Collapse);
        }
        // Nobody mends a ruin.
        if let Some(b) = ctx.land.buildings.get_mut(i)
            && b.state == BuildingState::Ruin
        {
            b.repair = None;
        }
    }
}

/// How grave it is that a group of kind `kind` gave way, the gravest first.
fn gravity(kind: GroupKind) -> u8 {
    match kind {
        GroupKind::Posts => 0,
        GroupKind::RoofFrame => 1,
        GroupKind::TieBeams => 2,
        GroupKind::FloorJoists => 3,
        GroupKind::LoftJoists => 4,
        GroupKind::RaisedFloor => 5,
        _ => 6,
    }
}

/// What presses on building `b` (of expansion `e`) holding `kept` of its household's goods, the
/// month's peak aside: its covering's weight once it is on, the goods on its lofts and floors,
/// and people on floors off the ground.
fn loads_on(b: &Building, e: &Expansion, kept: &[Vec<f64>; 3]) -> Loads {
    let area = |level: Level| -> f64 {
        e.spaces
            .iter()
            .filter(|s| s.level == level)
            .map(|s| s.area_m2)
            .sum()
    };
    // The covering's own weight, once it is on: its stage's materials over its slopes.
    let coverings: Vec<&Group> = e
        .groups
        .iter()
        .filter(|g| g.kind == GroupKind::Covering && b.group(g.id).is_some())
        .collect();
    let slope: f64 = coverings.iter().map(|g| g.area_m2.max(0.0)).sum();
    let mut slots: Vec<u8> = coverings.iter().filter_map(|g| g.material).collect();
    slots.sort_unstable();
    slots.dedup();
    let roof_kg: f64 = e
        .stages
        .get(civ_grammar::Stage::Roof.index())
        .map_or(0.0, |s| {
            slots
                .iter()
                .filter_map(|&m| s.materials_kg.get(usize::from(m)))
                .sum()
        });
    let covering_pa = if slope > 0.0 {
        roof_kg * G / slope
    } else {
        0.0
    };
    let kg = |kind: usize| -> f64 { kept[kind].iter().map(|v| v.max(0.0)).sum() };
    let per = |kg: f64, m2: f64| if m2 > 0.0 { kg * G / m2 } else { 0.0 };
    let raised = e.groups.iter().any(|g| g.kind == GroupKind::RaisedFloor);
    let (ground, upper) = (area(Level::Ground), area(Level::Upper));
    let upper_kg = if ground + upper > 0.0 {
        kg(storage::FLOOR) * upper / (ground + upper)
    } else {
        0.0
    };
    Loads {
        covering_pa,
        peak_pa: 0.0,
        loft_pa: per(kg(storage::LOFT), area(Level::Loft)),
        raised_pa: if raised {
            per(kg(storage::RAISED), ground)
        } else {
            0.0
        },
        upper_pa: per(upper_kg, upper),
        live_pa: LIVE_PA,
    }
}
