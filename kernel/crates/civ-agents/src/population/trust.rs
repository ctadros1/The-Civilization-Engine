//! What each settlement has seen of its buildings, and how cautiously it builds (ADR-0009 §6):
//! every building that gives way is remembered against its technique in its household's
//! settlement, and so is every month a building of it stands. The caution that follows sizes the
//! next frame buildings' members ([`crate::build::design_cautious`]).

use super::*;
use crate::caution::{CautionParams, Trust};
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

    /// A month passes: each settlement remembers a month of standing for every building of each
    /// technique its households have standing (not a ruin, and with parts in place).
    pub(super) fn remember_standing(&mut self, ctx: &Ctx) {
        let catalog = ctx.catalog;
        let mut standing: BTreeMap<(PermanentId, usize), u32> = BTreeMap::new();
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
        }
        let p = ctx.params.build.caution;
        for ((s, t), n) in standing {
            self.trust_mut(s, t, ctx.now, p.half_life_years).years += f64::from(n) / 12.0;
        }
    }
}
