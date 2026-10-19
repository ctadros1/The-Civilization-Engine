//! Buildings weighed against what they carry, each day (ADR-0009 §5; M3b slice P): the goods in
//! their lofts and on their floors, people on floors off the ground, their roofs' covering, the
//! month's storm on its day and the snow lying on their roofs (ADR-0012 §5). A group shows when its margin runs low or it sags, and
//! gives way below a margin of 1: a loft or a floor drops what it holds, a broken tie beam brings
//! its lofts down, rafters that break bring the roof in, and posts that give way leave a ruin.
//! Those inside may die. Nothing here decides what anyone stores, builds or mends.

use super::*;
use crate::history::Cause;
use crate::person::Keeping;
use crate::structure::{self, G, Loads};
use civ_grammar::{Group, GroupKind, Level, storage};
use civ_land::{BuildingState, Climatology, GroupState, Weather, WeatherParams};

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

/// Snow on a roof that gave way heavy enough to be told of, pascals on its plan: about 10 cm of
/// new snow (a tuning value).
pub const SNOW_TOLD_PA: f64 = 100.0;

/// The month's storm on the day of `now`, pascals on a roof's plan: [`Climatology::storm`]'s
/// load on its day, and none on the others (ADR-0012 §5). Every roof under the world's weather
/// meets the same storm, and a roof mended after it does not meet it again.
pub fn storm_pa(climate: &Climatology, params: &WeatherParams, now: SimTime) -> f64 {
    let d = now.date();
    let (day, pa) = climate.storm(params, d.year, usize::from(d.month.saturating_sub(1)));
    if d.day == day { pa } else { 0.0 }
}

/// The snow lying on the roof of building `e` at height `z_m`, pascals on its plan: its water's
/// weight (research 11-05 §2.4: p = ρgd from the snow lying), the share of what lies on the ground
/// there that a roof of its pitch keeps (ADR-0012 §5).
pub fn snow_on_roof_pa(weather: &Weather, params: &WeatherParams, z_m: f64, e: &Expansion) -> f64 {
    let pitch = structure::roof_cos(e).clamp(0.0, 1.0).acos().to_degrees();
    weather.snow_at(z_m).max(0.0) * G * params.roof_snow_share_at(pitch)
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
        let storm = storm_pa(&ctx.land.climatology, &ctx.land_params.weather, ctx.now);
        for household in owners {
            self.check_household_at(ctx, household, storm, 0, true);
        }
    }

    /// Performs the daily building check under controlled flood exposure (tests).
    #[doc(hidden)]
    pub fn flood_loads_day_for_tests(&mut self, ctx: &mut Ctx) {
        self.check_buildings(ctx);
    }

    /// Rechecks loads under a specified physical storm pulse (tests).
    #[doc(hidden)]
    pub fn storm_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        pressure_pa: f64,
        observer: u32,
    ) {
        self.check_household_origin(ctx, household, pressure_pa, observer);
    }

    /// What household `household` keeps under the roof of building `bi` now, kilograms of each
    /// good by kind of room: its stores settled to now and shared among its roofs as the daily
    /// check shares them.
    pub(super) fn kept_by(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        bi: usize,
    ) -> [Vec<f64>; 3] {
        let (now, catalog, params) = (ctx.now, ctx.catalog, ctx.params);
        let goods = &catalog.goods;
        let hd = self.hh_index.get(&household).copied();
        if let Some(hd) = hd
            && let Some(x) = self.households.get_mut(hd)
        {
            let members = x.members.len().max(1);
            x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
            x.stores.resize(goods.len(), 0.0);
        }
        let mut room_all = [0.0; 3];
        for b in ctx
            .land
            .buildings
            .iter()
            .filter(|b| b.household == household)
        {
            for (a, v) in room_all.iter_mut().zip(room_of(b, catalog)) {
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
        let room = ctx
            .land
            .buildings
            .get(bi)
            .map_or([0.0; 3], |b| room_of(b, catalog));
        kept_in(&fill, room_all, room)
    }

    /// Weighs household `household`'s buildings against what they carry now, with the month's
    /// storm `storm` pascals on their roofs' plan today.
    pub(super) fn check_household(&mut self, ctx: &mut Ctx, household: PermanentId, storm: f64) {
        self.check_household_origin(ctx, household, storm, 0);
    }

    pub(super) fn check_household_origin(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        storm: f64,
        observer: u32,
    ) {
        self.check_household_at(ctx, household, storm, observer, false);
    }

    fn check_household_at(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        storm: f64,
        observer: u32,
        daily_flood: bool,
    ) {
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
        let mut changed = false;
        for (&i, &room) in mine.iter().zip(&rooms) {
            let kept = kept_in(&fill, room_all, room);
            let b = &ctx.land.buildings[i];
            let cell = cell_of(ctx.map, build::centre_m(&b.spec));
            let flood_depth = ctx.land.flood_depths.get(cell).copied().unwrap_or(0.0);
            if daily_flood
                && flood_depth > 0.0
                && let Some(hd) = hd
                && let Some(x) = self.households.get_mut(hd)
            {
                for (g, &kg_floor) in kept[storage::FLOOR].iter().enumerate() {
                    let def = &goods[g];
                    if kg_floor > 0.0 && (def.half_life_days > 0.0 || def.kcal_per_kg > 0.0) {
                        let spoil_rate = (f64::from(flood_depth) * 0.25).clamp(0.02, 0.15);
                        let lost =
                            (kg_floor * spoil_rate).min(x.stores.get(g).copied().unwrap_or(0.0));
                        if lost > 0.0 {
                            x.stores[g] -= lost;
                            x.flows.add(crate::person::Flow::Spoiled, g, lost);
                            // The household sees its own stores spoil where the water stands
                            // (M6b slice BG).
                            x.remember_flood_loss(crate::person::FloodLoss {
                                day: now.day_index(),
                                cell: cell as u32,
                                depth_cm: (flood_depth * 100.0).min(f32::from(u16::MAX)) as u16,
                                kcal: (lost * def.kcal_per_kg) as f32,
                                hours: 0.0,
                            });
                            let reach = ctx.land.floodplain.cell_reach.get(cell).copied();
                            if let Some(hw) =
                                self.floods.high_waters.iter_mut().find(|h| {
                                    h.ended.is_none() && reach == Some(u32::from(h.reach))
                                })
                            {
                                hw.spoiled_kg += lost as f32;
                                if let Some(ep) = hw.episode.and_then(|id| {
                                    self.floods.flood_episodes.iter_mut().find(|e| e.id == id)
                                }) {
                                    ep.total_spoiled_kg += lost as f32;
                                }
                            }
                        }
                    }
                }
            }
            if let Some((fell, snow)) = self.weigh(ctx, i, &kept, storm, observer) {
                changed = true;
                if !fell.is_empty() {
                    self.give_way(ctx, i, &kept, &fell, storm, snow);
                }
            }
        }
        if changed {
            self.buildings_changed(now, ctx.land, catalog, params, household);
        }
    }

    /// Weighs building `i`, which holds `kept` of its household's goods, under the month's storm
    /// `storm` pascals and the snow lying on its roof: each group's state follows its wear and its
    /// margin. What gave way today, the gravest first, and the snow on its roof, pascals on its
    /// plan; `None` if nothing did.
    fn weigh(
        &mut self,
        ctx: &mut Ctx,
        i: usize,
        kept: &[Vec<f64>; 3],
        storm: f64,
        observer: u32,
    ) -> Option<(Vec<Fell>, f64)> {
        let catalog = ctx.catalog;
        let b = &ctx.land.buildings[i];
        if b.condition.is_empty() || !b.standing() {
            return None;
        }
        let def = catalog
            .building_index(&b.spec.program)
            .and_then(|d| catalog.buildings.get(d))?;
        let e = self.expansion_of(b, def)?;
        let (x, y) = build::centre_m(&b.spec);
        let z = civ_land::height_at(ctx.map, x, y);
        let snow = snow_on_roof_pa(&ctx.land.weather, &ctx.land_params.weather, z, &e);
        let loads = loads_on(b, &e, kept);
        let loads = Loads {
            peak_pa: if loads.covering_pa > 0.0 {
                storm + snow
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
        let mut torn = false;
        for c in b.condition.iter_mut() {
            if c.state == GroupState::Failed {
                continue;
            }
            let Some(g) = e.groups.iter().find(|g| g.id == c.group) else {
                continue;
            };
            let wear = condition::wear_state(c, &def.upkeep);
            let load = if g.kind == GroupKind::MassWall {
                let dummy = structure::Timber {
                    bending_pa: 0.0,
                    compression_pa: 0.0,
                    stiffness_pa: 0.0,
                    creep: 0.0,
                    sustained: 1.0,
                };
                structure::margin(g, c, &e, &loads, &dummy).state()
            } else if g.kind == GroupKind::Covering {
                let day = ctx.now.day_index();
                let checked = ctx.land.storm_checks.iter().any(|s| {
                    s.building == b.id
                        && s.group == c.group
                        && s.day == day
                        && s.observer == observer
                });
                // The natural storm is evaluated at the daily check; a covering installed later cannot
                // meet it retrospectively. Repair does not give the same pulse another draw.
                let after_natural = observer == 0 && c.installed.minutes() > day * 1440;
                if storm > 0.0 && !checked && !after_natural {
                    ctx.land.storm_checks.push(civ_land::StormCheck {
                        building: b.id,
                        group: c.group,
                        day,
                        observer,
                    });
                    let is_tile = g
                        .material
                        .and_then(|slot| def.materials.get(usize::from(slot)))
                        .and_then(|&good| catalog.goods.get(good))
                        .is_some_and(|good| good.id.contains("tile"));
                    let capacity = if is_tile {
                        1500.0 + 1000.0 * (1.0 - f64::from(c.loss))
                    } else {
                        600.0 + 600.0 * f64::from(c.quality) * (1.0 - f64::from(c.loss))
                    };
                    if storm > capacity {
                        c.loss = (c.loss + 0.4).min(1.0);
                        let torn_state = condition::wear_state(c, &def.upkeep);
                        torn = true;
                        torn_state
                    } else {
                        GroupState::Sound
                    }
                } else {
                    GroupState::Sound
                }
            } else {
                timber(g).map_or(GroupState::Sound, |t| {
                    structure::margin(g, c, &e, &loads, &t).state()
                })
            };
            c.state = condition::worse(wear, load);
            if c.state == GroupState::Failed && g.kind != GroupKind::Covering {
                fell.push(Fell {
                    kind: g.kind,
                    quality: c.quality,
                    loss: c.loss,
                });
            }
        }
        if torn {
            let wind_speed_ms = (2.0 * storm / 1.225).sqrt() as f32;
            let damage_id = (self.floods.storm_damages.len() + 1) as u32;
            let origin = crate::floods::Origin::recorded(
                (observer > 0).then_some(observer).into_iter().collect(),
            );
            let ep_id = if let Some(ep) = self
                .floods
                .storm_episodes
                .iter_mut()
                .find(|e| e.ended.is_none() && e.began.day_index() == ctx.now.day_index())
            {
                ep.damages.push(damage_id);
                ep.origin.join(&origin);
                if wind_speed_ms > ep.peak_wind_speed_ms {
                    ep.peak_wind_speed_ms = wind_speed_ms;
                }
                ep.id
            } else {
                let new_id = (self.floods.storm_episodes.len() + 1) as u32;
                self.floods
                    .storm_episodes
                    .push(crate::floods::StormEpisode {
                        origin: origin.clone(),
                        id: new_id,
                        settlement: self.household(b.household).and_then(|x| x.settlement),
                        began: ctx.now,
                        ended: None,
                        peak_wind_speed_ms: wind_speed_ms,
                        damages: vec![damage_id],
                    });
                new_id
            };
            self.floods.storm_damages.push(crate::floods::StormDamage {
                origin,
                id: damage_id,
                building: b.id,
                household: b.household,
                settlement: self.household(b.household).and_then(|x| x.settlement),
                at: (x, y),
                time: ctx.now,
                wind_speed_ms,
                uplift_pa: storm as f32,
                episode: Some(ep_id),
            });
        }
        if fell.is_empty() && !torn {
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
        Some((fell, snow))
    }

    /// What follows when parts of building `i`, which held `kept` of its household's goods, gave
    /// way (`fell`, the gravest first) under the month's storm `storm` and the snow `snow` on its
    /// roof, pascals on its plan: the goods on what fell are spilled and part lost, those inside
    /// may die, and the chronicle tells of it.
    fn give_way(
        &mut self,
        ctx: &mut Ctx,
        i: usize,
        kept: &[Vec<f64>; 3],
        fell: &[Fell],
        storm: f64,
        snow: f64,
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
            let (ground, upper) = (area(Level::Ground), upper_storeys_m2(&e));
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
            Some(GroupKind::Posts | GroupKind::MassWall) => DIES_IN_RUIN,
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
            Some(GroupKind::MassWall) => "fell: its walls gave way",
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
        } else if storm > 0.0
            && storm >= snow
            && storm >= 2.0 * ctx.land_params.weather.storm_median_pa
        {
            words.push_str(" in a storm");
        } else if snow >= SNOW_TOLD_PA && snow > storm {
            words.push_str(" under snow");
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
        // And each household of it hears it told and remembers it (M6b slice BH).
        if let Some(b) = ctx.land.buildings.get(i) {
            let seen = crate::person::GaveWay {
                day: now.day_index(),
                building: id,
                own: false,
                wall_kind: crate::rules::wall_kind(&b.spec, &def.rules),
                hours: Self::fire_cost_h(ctx, b) as f32,
            };
            for (_, x) in self.households.iter_mut() {
                if settlement.is_some() && x.settlement == settlement {
                    x.remember_gave_way(crate::person::GaveWay {
                        own: x.id == household,
                        ..seen
                    });
                }
            }
        }
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
/// Floor of every storey above the first, square metres.
fn upper_storeys_m2(e: &Expansion) -> f64 {
    e.spaces
        .iter()
        .filter(|s| s.level.above_ground())
        .map(|s| s.area_m2)
        .sum()
}

fn gravity(kind: GroupKind) -> u8 {
    match kind {
        GroupKind::Posts | GroupKind::MassWall => 0,
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
    let (ground, upper) = (area(Level::Ground), upper_storeys_m2(e));
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
