//! Crossings in the day and underfoot (M5c slice AW; research 11-07 §1.3, §2.4, §6.2; the
//! settlements brief §1.7). Each midnight rot takes its share of every open crossing's members,
//! and one that can no longer carry its own weight gives way with nobody on it. Whoever sets off
//! on a walk over an open crossing steps onto it: one that cannot carry them gives way under them
//! (11-07 §6.2: loads are checked as agents step on). A crossing that gives way drops whoever is
//! on it into the water, who may die; everyone whose walk would have crossed it stops where they
//! are and decides again, as no route may cross a fallen bridge (01-08 §7).

use super::*;
use crate::bridge::crossing_margin;
use crate::history::{Cause, CrossingStep};
use civ_land::crossings::{Collapse, CrossingState};
use civ_land::paths::cells_along;

/// Keyed-randomness purpose of who dies when a crossing gives way under them.
const PURPOSE_FALL: u64 = 0x6272_6964_6765_6661; // "bridgefa"

impl Population {
    /// The crossings' midnight: rot takes its daily share of each open crossing's members, and
    /// one that can no longer carry its own weight gives way.
    pub(super) fn crossings_day(&mut self, ctx: &mut Ctx) {
        for i in 0..ctx.land.crossings.list.len() {
            let c = &mut ctx.land.crossings.list[i];
            if !c.open() {
                continue;
            }
            let Some(def) = ctx.catalog.bridges.get(usize::from(c.system)) else {
                continue;
            };
            c.loss = (c.loss + (def.loss_per_year / 365.0) as f32).min(1.0);
            let Some(timber) = ctx.catalog.goods.get(def.good).and_then(|g| g.timber) else {
                continue;
            };
            if crossing_margin(c, def, &timber, 0) < 1.0 {
                self.crossing_gives_way(ctx, i, Collapse::OwnWeight, None);
            }
        }
    }

    /// `who` sets off along `points`: each open crossing on the way is stepped onto. One that
    /// cannot carry them gives way under them, and they do not go (`false`).
    pub(super) fn step_on(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        points: &[(f32, f32)],
    ) -> bool {
        let map = ctx.map;
        let mut crossed: Vec<usize> = cells_along(points, map.cell_size_m, map.width, map.height)
            .into_iter()
            .filter_map(|c| ctx.land.crossings.open_at(c))
            .collect();
        crossed.sort_unstable();
        crossed.dedup();
        for i in crossed {
            let c = &ctx.land.crossings.list[i];
            let Some(def) = ctx.catalog.bridges.get(usize::from(c.system)) else {
                continue;
            };
            let Some(timber) = ctx.catalog.goods.get(def.good).and_then(|g| g.timber) else {
                continue;
            };
            if crossing_margin(c, def, &timber, 1) < 1.0 {
                self.crossing_gives_way(ctx, i, Collapse::UnderWalker, Some(who));
                return false;
            }
        }
        true
    }

    /// Crossing `i` gives way for `why`, under `under` if someone stepped onto it: whoever is on
    /// it falls, and may die; whoever would have walked over it stops where they are.
    fn crossing_gives_way(
        &mut self,
        ctx: &mut Ctx,
        i: usize,
        why: Collapse,
        under: Option<PermanentId>,
    ) {
        let (now, day) = (ctx.now, ctx.now.day_index());
        let crossing = &mut ctx.land.crossings.list[i];
        crossing.state = CrossingState::Failed { day, why };
        let (id, cells, banks, system) = (
            crossing.id,
            crossing.cells.clone(),
            crossing.banks,
            crossing.system,
        );
        ctx.land.crossings.changed();
        let kills = ctx
            .catalog
            .bridges
            .get(usize::from(system))
            .map_or(0.0, |d| d.fall_kills);
        let map = ctx.map;
        let t = now.minutes() as f64;
        // Who is on it now, and whose walk would cross it.
        let mut on: Vec<(Handle<Person>, PermanentId)> = Vec::new();
        let mut stop: Vec<(Handle<Person>, (f32, f32))> = Vec::new();
        for (h, p) in self.people.iter() {
            let Some(trip) = &p.trip else {
                continue;
            };
            let at = trip.position_at(t);
            if cells.contains(&(cell_of(map, at) as u32)) {
                on.push((h, p.id));
                continue;
            }
            // What is left of the walk, from where they are.
            let since = (t - trip.depart.minutes() as f64) as f32;
            let mut rest = vec![at];
            rest.extend(
                trip.points
                    .iter()
                    .zip(&trip.minutes)
                    .filter(|&(_, &m)| m > since)
                    .map(|(&pt, _)| pt),
            );
            if rest.len() > 1
                && cells_along(&rest, map.cell_size_m, map.width, map.height)
                    .iter()
                    .any(|c| cells.contains(c))
            {
                stop.push((h, at));
            }
        }
        if let Some(who) = under
            && let Some(&h) = self.index.get(&who)
        {
            on.push((h, who));
        }
        // Those who fell: some die; the rest scramble out onto the nearer bank.
        let mut dead = Vec::new();
        let mut fell = Vec::new();
        for &(h, who) in &on {
            fell.push(who);
            let key = [ctx.seed, PURPOSE_FALL, id.get(), who.get(), day as u64];
            if Rng64::from_key(&key).next_f64() < kills {
                dead.push(who);
                continue;
            }
            let Some(p) = self.people.get_mut(h) else {
                continue;
            };
            if p.trip.is_some() {
                let at = p.trip.as_ref().map_or(p.pos, |trip| trip.position_at(t));
                let near = |b: u32| {
                    let c = cell_centre(map, b as usize);
                    (c.0 - at.0).powi(2) + (c.1 - at.1).powi(2)
                };
                let bank = if near(banks[0]) <= near(banks[1]) {
                    banks[0]
                } else {
                    banks[1]
                };
                p.pos = cell_centre(map, bank as usize);
                self.wait(ctx, h, 1);
            }
        }
        for (h, at) in stop {
            if let Some(p) = self.people.get_mut(h) {
                p.pos = at;
                self.wait(ctx, h, 1);
            }
        }
        // The chronicle, then the dead.
        let name = ctx
            .catalog
            .bridges
            .get(usize::from(system))
            .map_or_else(|| "crossing".to_owned(), |d| d.name.to_lowercase());
        let cause = match why {
            Collapse::OwnWeight => "its rotten members could no longer carry their own weight",
            Collapse::UnderWalker => "its rotten members broke under someone stepping onto it",
        };
        let mut sentence = format!("A {name} gave way: {cause}.");
        if !fell.is_empty() {
            let names: Vec<String> = fell.iter().map(|&p| self.name_of(p)).collect();
            sentence += &format!(
                " {} fell into the water{}.",
                names.join(" and "),
                if dead.is_empty() {
                    String::new()
                } else {
                    let d: Vec<String> = dead.iter().map(|&p| self.name_of(p)).collect();
                    format!("; {} died", d.join(" and "))
                }
            );
        }
        let middle = cells[cells.len() / 2] as usize;
        self.chronicle_push(
            now,
            ChronicleKind::Crossing,
            fell,
            None,
            Some(cell_centre(map, middle)),
            f64::from(CrossingStep::Failed as u8),
            sentence,
        );
        for who in dead {
            self.die(ctx, who, Cause::Fell);
        }
    }
}
