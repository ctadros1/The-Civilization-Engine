use super::*;
use crate::history::{ChronicleKind, FloodStep};

impl Population {
    /// Midnight processing for floods, high waters, and storm episodes (M6b slice BF).
    pub(super) fn floods_day(&mut self, ctx: &mut Ctx) {
        let (now, day) = (ctx.now, ctx.now.day_index());

        // 1. High-water monitoring along river reaches
        let num_reaches = ctx.map.reaches.len();
        for r in 0..num_reaches {
            let q = ctx.land.reach_discharges.get(r).copied().unwrap_or(0.0);
            let q_bf = ctx
                .land
                .floodplain
                .reaches
                .get(r)
                .map(|rh| rh.bankfull_discharge_m3s)
                .unwrap_or(0.0);
            let is_high = q_bf >= 2.0 && f64::from(q) > q_bf;

            let active_idx = self
                .floods
                .high_waters
                .iter()
                .position(|hw| hw.reach == r as u16 && hw.ended.is_none());

            if is_high {
                if let Some(idx) = active_idx {
                    let hw = &mut self.floods.high_waters[idx];
                    if q > hw.peak_discharge {
                        hw.peak_discharge = q;
                    }
                } else {
                    // New high-water incident
                    let reach = &ctx.map.reaches[r];
                    let at = if let Some(&c) = reach.cells.first() {
                        cell_centre(ctx.map, c as usize)
                    } else {
                        (0.0, 0.0)
                    };
                    let settlement = ctx
                        .land
                        .settlements
                        .iter()
                        .find(|s| {
                            let (sx, sy) = s.hearth_m;
                            (sx - at.0).hypot(sy - at.1) < 500.0
                        })
                        .map(|s| s.id);

                    let hw_id = (self.floods.high_waters.len() + 1) as u32;

                    // Find or create active flood episode
                    let ep_id = if let Some(ep) = self.floods.current_flood_episode_mut() {
                        ep.high_waters.push(hw_id);
                        if q > ep.peak_discharge {
                            ep.peak_discharge = q;
                        }
                        ep.id
                    } else {
                        let new_id = (self.floods.flood_episodes.len() + 1) as u32;
                        self.floods
                            .flood_episodes
                            .push(crate::floods::FloodEpisode {
                                id: new_id,
                                settlement,
                                first_reach: r as u16,
                                began: now,
                                ended: None,
                                peak_discharge: q,
                                high_waters: vec![hw_id],
                                total_spoiled_kg: 0.0,
                                total_buildings: 0,
                                total_fields_lost: 0,
                            });
                        new_id
                    };

                    self.floods.high_waters.push(crate::floods::HighWater {
                        id: hw_id,
                        reach: r as u16,
                        settlement,
                        at,
                        began: now,
                        ended: None,
                        peak_discharge: q,
                        peak_stage: 0.0,
                        episode: Some(ep_id),
                        buildings_flooded: 0,
                        fields_flooded: 0,
                        spoiled_kg: 0.0,
                        scoured_crossings: 0,
                    });

                    self.chronicle_push(
                        now,
                        ChronicleKind::Flood,
                        Vec::new(),
                        settlement,
                        Some(at),
                        f64::from(FloodStep::Inundated as u8),
                        format!("The river rose above its banks: flow reached {q:.1} m³/s."),
                    );
                }
            } else if let Some(idx) = active_idx {
                // High water receded
                let hw = &mut self.floods.high_waters[idx];
                hw.ended = Some(now);
                let at = hw.at;
                let settlement = hw.settlement;
                let ep_id = hw.episode;

                self.chronicle_push(
                    now,
                    ChronicleKind::Flood,
                    Vec::new(),
                    settlement,
                    Some(at),
                    f64::from(FloodStep::Receded as u8),
                    "Floodwaters receded.".to_owned(),
                );

                // If all high waters for this episode ended, close episode
                if let Some(ep_id) = ep_id
                    && let Some(ep) = self
                        .floods
                        .flood_episodes
                        .iter_mut()
                        .find(|e| e.id == ep_id)
                {
                    let any_open = ep.high_waters.iter().any(|&h_id| {
                        self.floods
                            .high_waters
                            .iter()
                            .any(|h| h.id == h_id && h.ended.is_none())
                    });
                    if !any_open {
                        ep.ended = Some(now);
                    }
                }
            }
        }

        if let Some(ep) = self.floods.current_flood_episode_mut() {
            let flooded_bld = ctx.land.buildings.iter().filter(|b| b.days_wet > 0).count() as u32;
            ep.total_buildings = ep.total_buildings.max(flooded_bld);
            let lost_fields = ctx
                .land
                .fields
                .iter()
                .filter(|f| f.submerged_days >= 5)
                .count() as u32;
            ep.total_fields_lost = ep.total_fields_lost.max(lost_fields);
        }
        for hw in self
            .floods
            .high_waters
            .iter_mut()
            .filter(|h| h.ended.is_none())
        {
            let flooded_bld = ctx.land.buildings.iter().filter(|b| b.days_wet > 0).count() as u32;
            hw.buildings_flooded = hw.buildings_flooded.max(flooded_bld);
            let fld_flooded = ctx
                .land
                .fields
                .iter()
                .filter(|f| f.submerged_days > 0)
                .count() as u32;
            hw.fields_flooded = hw.fields_flooded.max(fld_flooded);
        }

        // 2. Household flood memory
        for (_, hh) in self.households.iter_mut() {
            let home_cell = cell_of(ctx.map, hh.home);
            let home_cm = ctx
                .land
                .flood_depths
                .get(home_cell)
                .map(|&d| (d * 100.0) as u16)
                .unwrap_or(0);
            let field_cm = ctx
                .land
                .fields
                .iter()
                .filter(|f| f.household == hh.id)
                .map(|f| f.last_flood_depth_cm)
                .max()
                .unwrap_or(0);
            if home_cm > 0 || field_cm > 0 {
                hh.flood_memory.remember(home_cm, field_cm, day);
            }
        }

        // 3. Storm episode close if no storm today
        let storm = loads::storm_pa(&ctx.land.climatology, &ctx.land_params.weather, now);
        if storm == 0.0
            && let Some(ep) = self.floods.current_storm_episode_mut()
        {
            ep.ended = Some(now);
        }
    }

    /// The observer raises river reach `reach` to `discharge_m3s` for `days` days (god tool, M6b slice BF; ADR-0022 §13).
    pub fn raise_river(
        &mut self,
        ctx: &mut Ctx,
        reach: usize,
        discharge_m3s: f32,
        days: u32,
    ) -> Result<super::influence::Reached, String> {
        let reach_obj = ctx
            .map
            .reaches
            .get(reach)
            .ok_or_else(|| format!("no reach {reach}"))?;
        let at = reach_obj
            .cells
            .first()
            .map_or((0.0, 0.0), |&c| cell_centre(ctx.map, c as usize));
        let settlement = ctx
            .land
            .settlements
            .iter()
            .find(|s| {
                let (sx, sy) = s.hearth_m;
                (sx - at.0).hypot(sy - at.1) < 500.0
            })
            .map(|s| s.id);
        let id = (self.floods.high_waters.len() + 1) as u32;
        let inf_id = self.influences.add(
            ctx.now,
            crate::influence::InfluenceKind::Flood,
            settlement.unwrap_or_else(|| PermanentId::from_raw(1).expect("id")),
            id,
        );
        if let Some(inf) = self.influences.list.iter_mut().find(|i| i.id == inf_id) {
            inf.until = ctx.now.day_index() + i64::from(days);
            inf.subject = id;
        }
        self.chronicle_push(
            ctx.now,
            ChronicleKind::Influence,
            Vec::new(),
            settlement,
            Some(at),
            f64::from(crate::influence::InfluenceKind::Flood.code()),
            format!(
                "The observer raised reach {reach} to {discharge_m3s:.1} m³/s for {days} days."
            ),
        );
        self.floods_day(ctx);
        Ok(super::influence::Reached {
            id: inf_id,
            repeat: false,
            holds: false,
            fit: 0.0,
            chance: 0.0,
        })
    }

    /// The observer sends a gale with `wind_speed_ms` from `direction_deg` (god tool, M6b slice BF; ADR-0022 §13).
    pub fn send_storm(
        &mut self,
        ctx: &mut Ctx,
        wind_speed_ms: f32,
        direction_deg: f32,
    ) -> Result<super::influence::Reached, String> {
        let settlement = ctx.land.settlements.first().map(|s| s.id);
        let id = (self.floods.storm_episodes.len() + 1) as u32;
        let inf_id = self.influences.add(
            ctx.now,
            crate::influence::InfluenceKind::Storm,
            settlement.unwrap_or_else(|| PermanentId::from_raw(1).expect("id")),
            id,
        );
        let storm_pa = 0.5 * 1.225 * f64::from(wind_speed_ms).powi(2);
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
            self.check_household(ctx, household, storm_pa);
        }
        self.chronicle_push(
            ctx.now,
            ChronicleKind::Influence,
            Vec::new(),
            settlement,
            None,
            f64::from(crate::influence::InfluenceKind::Storm.code()),
            format!("The observer sent a gale: wind reached {wind_speed_ms:.1} m/s from {direction_deg:.0}°."),
        );
        Ok(super::influence::Reached {
            id: inf_id,
            repeat: false,
            holds: false,
            fit: 0.0,
            chance: 0.0,
        })
    }
}
