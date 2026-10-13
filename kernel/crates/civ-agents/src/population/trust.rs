//! What each settlement has seen of its buildings, and how cautiously it builds (ADR-0009 §6):
//! every building that gives way is remembered against its technique in its household's
//! settlement, and so is every month a building of it stands. The caution that follows sizes the
//! next frame buildings' members ([`crate::build::design_cautious`]).

use super::*;
use crate::caution::{CautionParams, FireTrust, Trust};
use civ_land::BuildingState;
use std::collections::BTreeMap;

impl Population {
    /// How many times their usual strength builders of settlement `settlement` make the members
    /// of a building of technique `technique` at `now`: 1 for a household of no settlement, a
    /// program of no technique, and a technique whose buildings the settlement has seen nothing
    /// of.
    pub fn caution(
        &self,
        settlement: Option<PermanentId>,
        technique: Option<usize>,
        now: SimTime,
        p: &CautionParams,
    ) -> f64 {
        let (Some(s), Some(t)) = (settlement, technique) else {
            return 1.0;
        };
        self.trust
            .iter()
            .find(|x| x.settlement == s && usize::from(x.technique) == t)
            .map_or(1.0, |x| x.caution(now, p))
    }

    /// What settlement `settlement` has seen of technique `technique`'s buildings, brought up to
    /// `now` (a new record when it has seen nothing yet).
    fn trust_mut(
        &mut self,
        settlement: PermanentId,
        technique: usize,
        now: SimTime,
        half_life_years: f64,
    ) -> &mut Trust {
        let i = match self
            .trust
            .iter()
            .position(|x| x.settlement == settlement && usize::from(x.technique) == technique)
        {
            Some(i) => i,
            None => {
                let t = u16::try_from(technique).unwrap_or(u16::MAX);
                self.trust.push(Trust::new(settlement, t, now));
                self.trust.len() - 1
            }
        };
        let t = &mut self.trust[i];
        t.fade(now, half_life_years);
        t
    }

    /// A building of technique `technique` in settlement `settlement` gave way at `now`, killing
    /// `deaths`: the settlement remembers one failure, and each death as
    /// [`CautionParams::death_weight`] more.
    pub(super) fn remember_failure(
        &mut self,
        settlement: Option<PermanentId>,
        technique: Option<usize>,
        deaths: usize,
        now: SimTime,
        p: &CautionParams,
    ) {
        let (Some(s), Some(t)) = (settlement, technique) else {
            return;
        };
        self.trust_mut(s, t, now, p.half_life_years).failures +=
            1.0 + deaths as f64 * p.death_weight;
    }

    /// What settlement `settlement` has seen of buildings lost to fire of `tile` covering and
    /// `wall_kind`, brought up to `now`.
    fn fire_trust_mut(
        &mut self,
        settlement: PermanentId,
        tile: bool,
        wall_kind: u8,
        now: SimTime,
        half_life_years: f64,
    ) -> &mut FireTrust {
        let i =
            match self.fire_trust.iter().position(|x| {
                x.settlement == settlement && x.tile == tile && x.wall_kind == wall_kind
            }) {
                Some(i) => i,
                None => {
                    self.fire_trust
                        .push(FireTrust::new(settlement, tile, wall_kind, now));
                    self.fire_trust.len() - 1
                }
            };
        let t = &mut self.fire_trust[i];
        t.fade(now, half_life_years);
        t
    }

    /// Remembers a building lost to fire in settlement `settlement` (ADR-0022 §7).
    pub(super) fn remember_fire_loss(
        &mut self,
        settlement: Option<PermanentId>,
        tile: bool,
        wall_kind: u8,
        now: SimTime,
        p: &CautionParams,
    ) {
        let Some(s) = settlement else {
            return;
        };
        self.fire_trust_mut(s, tile, wall_kind, now, p.half_life_years)
            .losses += 1.0;
    }

    /// How cautiously builders of settlement `settlement` view thatch covering against fire at `now`:
    /// 1.0 when nothing has burned, rising with the thatch loss rate.
    pub fn fire_caution_thatch(
        &self,
        settlement: Option<PermanentId>,
        now: SimTime,
        p: &CautionParams,
    ) -> f64 {
        let Some(s) = settlement else {
            return 1.0;
        };
        let mut total_losses = 0.0;
        let mut total_years = 0.0;
        for t in self
            .fire_trust
            .iter()
            .filter(|x| x.settlement == s && !x.tile)
        {
            let (l, y) = t.faded(now, p.half_life_years);
            total_losses += l;
            total_years += y;
        }
        let rate = total_losses / total_years.max(crate::caution::LEAST_YEARS);
        1.0 + (p.most - 1.0).max(0.0) * rate / (rate + p.half_rate.max(1e-12))
    }

    /// How cautiously builders of settlement `settlement` view wattle and daub walls (kind 0)
    /// against fire at `now`: 1.0 when nothing has burned, rising with the wattle wall loss rate.
    pub fn fire_caution_wattle(
        &self,
        settlement: Option<PermanentId>,
        now: SimTime,
        p: &CautionParams,
    ) -> f64 {
        let Some(s) = settlement else {
            return 1.0;
        };
        let mut total_losses = 0.0;
        let mut total_years = 0.0;
        for t in self
            .fire_trust
            .iter()
            .filter(|x| x.settlement == s && x.wall_kind == 0)
        {
            let (l, y) = t.faded(now, p.half_life_years);
            total_losses += l;
            total_years += y;
        }
        let rate = total_losses / total_years.max(crate::caution::LEAST_YEARS);
        1.0 + (p.most - 1.0).max(0.0) * rate / (rate + p.half_rate.max(1e-12))
    }

    /// A month passes: each settlement remembers a month of standing for every building of each
    /// technique its households have standing (not a ruin, and with parts in place), and a month
    /// of standing for each covering and wall kind against fire.
    pub(super) fn remember_standing(&mut self, ctx: &Ctx) {
        let catalog = ctx.catalog;
        let mut standing: BTreeMap<(PermanentId, usize), u32> = BTreeMap::new();
        let mut fire_standing: BTreeMap<(PermanentId, bool, u8), u32> = BTreeMap::new();
        for b in &ctx.land.buildings {
            if b.state == BuildingState::Ruin || b.condition.is_empty() {
                continue;
            }
            let technique = catalog
                .building_index(&b.spec.program)
                .and_then(|i| catalog.buildings[i].technique);
            let settlement = self.household(b.household).and_then(|x| x.settlement);
            if let (Some(s), Some(t)) = (settlement, technique) {
                *standing.entry((s, t)).or_default() += 1;
            }
            if let Some(s) = settlement {
                let tile = b
                    .spec
                    .materials
                    .get(usize::from(civ_grammar::frame::frame_materials::COVERING))
                    .is_some_and(|m| m.contains("tile"));
                let wall_kind = b.spec.params[civ_grammar::frame::frame_params::WALL_KIND] as u8;
                *fire_standing.entry((s, tile, wall_kind)).or_default() += 1;
            }
        }
        let p = ctx.params.build.caution;
        for ((s, t), n) in standing {
            self.trust_mut(s, t, ctx.now, p.half_life_years).years += f64::from(n) / 12.0;
        }
        for ((s, tile, wall_kind), n) in fire_standing {
            self.fire_trust_mut(s, tile, wall_kind, ctx.now, p.half_life_years)
                .years += f64::from(n) / 12.0;
        }
    }
}
