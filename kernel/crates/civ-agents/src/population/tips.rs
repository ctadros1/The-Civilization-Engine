//! Refuse carried to a tip (M11f, second part; ADR-0038 §3; brief m11-building-waste §2.4;
//! research 12-10 §1.4, §1.9, §5.4, §5.9): where someone suspects a source of water, or a
//! household's heap has grown large beside another's home, anyone may propose that the polity open
//! a tip and its households carry their heaps there. Passing it names one place, once: clear
//! ground at least the law's distance from every open well, spring and stream and from every
//! home, the nearest such to the settlement's homes; with none, no tip opens and nobody carries.
//! A household's member carries its whole heap there, at most every so many days, walked and timed
//! as carrying a midden from the water is; on arrival its decaying share and its ash join the
//! tip's, and what it held of a disease lies at the tip's contagion node, washing and seeping to
//! the water from the tip's place by the same links as a heap's. It is kept by choice and breaks
//! only as cases (ADR-0025), as clearing middens does, which it supersedes for a household once a
//! tip is open.

use super::*;
use crate::contagion::{How, Move, Node};
use crate::decide::WatchFacts;
use crate::polity::{LawStatus, PolicyKind, Tip, TipDef};
use crate::sickness::Outcome;

/// How far from a tip's place, metres beyond a cell, a carrier may stand and tip there.
const AT_TIP_CELLS: f64 = 2.0;

impl Population {
    /// The terms of the first template that carries refuse to a tip, if the content has one.
    fn tip_def(ctx: &Ctx) -> Option<TipDef> {
        ctx.catalog.policies.iter().find_map(|d| d.tip)
    }

    /// Where a tip is: its place, if any polity keeps one of id `tip`.
    pub(super) fn tip_place(&self, tip: PermanentId) -> Option<(f32, f32)> {
        self.polities
            .iter()
            .flat_map(|p| p.tips.iter())
            .find(|t| t.id == tip)
            .map(|t| t.place)
    }

    /// The settlement of the polity that keeps tip `tip`.
    pub(super) fn tip_settlement(&self, tip: PermanentId) -> Option<PermanentId> {
        self.polities
            .iter()
            .find(|p| p.tips.iter().any(|t| t.id == tip))
            .map(|p| p.settlement)
    }

    /// Whether a spring rises within `radius_m` of `at`.
    fn spring_near(ctx: &Ctx, at: (f32, f32), radius_m: f64) -> bool {
        let (ax, ay) = (f64::from(at.0), f64::from(at.1));
        ctx.land
            .water
            .aquifer
            .seep
            .iter()
            .flatten()
            .any(|&(cell, _)| {
                let (sx, sy) = cell_centre(ctx.map, cell as usize);
                (f64::from(sx) - ax).hypot(f64::from(sy) - ay) <= radius_m
            })
    }

    /// Where a law of terms `def` of the polity of `settlement` would open its tip (ADR-0038 §3):
    /// of the cells about the settlement's homes on clear dry ground more than `from_m` from every
    /// open well, spring and stream and from every home, and from every open well as far as a
    /// rule keeping middens from wells in force there asks, the one nearest its homes (least
    /// summed distance), as a cell's centre; none where the settlement has no homes or no cell
    /// qualifies.
    pub(super) fn tip_site(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        def: &TipDef,
    ) -> Option<(f32, f32)> {
        let from = f64::from(def.from_m);
        // The siting rules the polity keeps (ADR-0038 §3: `midden_from_wells`).
        let wells_from = self
            .rules_in_force(ctx, Some(settlement))
            .into_iter()
            .filter_map(|(_, t)| match t {
                crate::polity::RuleTemplate::MiddenFromWells { from_m } => Some(f64::from(from_m)),
                _ => None,
            })
            .fold(from, f64::max);
        let wells: Vec<(f64, f64)> = ctx
            .land
            .wells
            .list
            .iter()
            .filter(|w| w.is_open())
            .map(|w| {
                let (x, y) = w.rect.centre_m();
                (f64::from(x), f64::from(y))
            })
            .collect();
        let homes: Vec<(f64, f64)> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| !x.members.is_empty())
            .map(|x| (f64::from(x.home.0), f64::from(x.home.1)))
            .collect();
        let ours: Vec<(f64, f64)> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| x.settlement == Some(settlement) && !x.members.is_empty())
            .map(|x| (f64::from(x.home.0), f64::from(x.home.1)))
            .collect();
        if ours.is_empty() {
            return None;
        }
        let map = ctx.map;
        let size = f64::from(map.cell_size_m);
        // About the settlement's homes, far enough out to clear them and the water near them.
        let pad = 4.0 * from + size;
        let (x0, x1, y0, y1) = ours.iter().fold(
            (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
            |(a, b, c, d), &(x, y)| (a.min(x), b.max(x), c.min(y), d.max(y)),
        );
        let cell = |v: f64, n: u32| ((v / size).floor() as i64).clamp(0, i64::from(n) - 1);
        let (cx0, cx1) = (cell(x0 - pad, map.width), cell(x1 + pad, map.width));
        let (cy0, cy1) = (cell(y0 - pad, map.height), cell(y1 + pad, map.height));
        let cm = |m: f64| (m * 100.0).round() as i32;
        let half = cm(size / 2.0).max(1);
        let mut best: Option<((f32, f32), f64)> = None;
        for y in cy0..=cy1 {
            for x in cx0..=cx1 {
                let i = (y * i64::from(map.width) + x) as usize;
                if map.water[i] != WATER_LAND {
                    continue;
                }
                let (px, py) = ((x as f64 + 0.5) * size, (y as f64 + 0.5) * size);
                let sum: f64 = ours.iter().map(|&(hx, hy)| (hx - px).hypot(hy - py)).sum();
                if best.is_some_and(|b| sum >= b.1) {
                    continue;
                }
                if homes
                    .iter()
                    .any(|&(hx, hy)| (hx - px).hypot(hy - py) <= from)
                    || wells
                        .iter()
                        .any(|&(wx, wy)| (wx - px).hypot(wy - py) <= wells_from)
                {
                    continue;
                }
                let at = (px as f32, py as f32);
                if Self::nearest_source(ctx, at, from).is_some() || Self::spring_near(ctx, at, from)
                {
                    continue;
                }
                let (cx, cy) = (cm(px), cm(py));
                let rect = civ_land::RectCm {
                    x: cx - half,
                    y: cy - half,
                    w: 2 * half,
                    h: 2 * half,
                };
                if !crate::build::plot_clear(ctx.land, map, ctx.nav, &rect) {
                    continue;
                }
                best = Some((at, sum));
            }
        }
        best.map(|b| b.0)
    }

    /// Opens the tip of law `law` of polity `pi`, just passed (ADR-0038 §3), where
    /// [`Population::tip_site`] finds one; none where no place qualifies.
    pub(super) fn open_tip(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        let Some(def) = self.polities[pi]
            .laws
            .iter()
            .find(|l| l.id == law)
            .and_then(|l| ctx.catalog.policies.get(usize::from(l.policy)))
            .and_then(|d| d.tip)
        else {
            return;
        };
        let settlement = self.polities[pi].settlement;
        let Some(place) = self.tip_site(ctx, settlement, &def) else {
            return;
        };
        let id = ctx.ids.allocate();
        self.polities[pi].tips.push(Tip {
            id,
            law,
            opened: ctx.now.day_index(),
            place,
            kg: 0.0,
            ash_kg: 0.0,
            households: Vec::new(),
        });
    }

    /// The law carrying refuse to a tip where household `hh` lives, if one is in force and its tip
    /// is open: the polity's index, the law's, the tip's and the terms.
    pub(super) fn tip_law(
        &self,
        ctx: &Ctx,
        hh: &Household,
    ) -> Option<(usize, usize, usize, TipDef)> {
        let pi = self.polity_of(hh.settlement?)?;
        let polity = &self.polities[pi];
        polity
            .laws
            .iter()
            .enumerate()
            .filter(|(_, l)| l.status == LawStatus::InForce && l.kind == PolicyKind::RefuseToTip)
            .find_map(|(li, l)| {
                let def = ctx.catalog.policies.get(usize::from(l.policy))?.tip?;
                let ti = polity.tips.iter().position(|t| t.law == l.id)?;
                Some((pi, li, ti, def))
            })
    }

    /// Whether household `x` carried its heap to a tip within terms `def`'s days as of `day`.
    fn tipped_lately(x: &Household, def: &TipDef, day: i64) -> bool {
        x.midden.tipped >= 0 && day - x.midden.tipped < i64::from(def.every_days.max(1))
    }

    /// The heap person `person` of household `hh` could carry to the tip now, where a law carrying
    /// refuse to one is in force and its tip open: in daylight, when the heap holds anything, was
    /// not tipped within the law's days and they know the law, the tip and what keeping the law
    /// weighs with them. `None` inside means the law applies and asks nothing now.
    pub(super) fn tip_facts(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        hh: &Household,
        dark: bool,
    ) -> Option<Option<WatchFacts>> {
        let (pi, li, ti, def) = self.tip_law(ctx, hh)?;
        let day = ctx.now.day_index();
        let kg = hh
            .midden
            .at_time(ctx.now, hh.members.len(), &ctx.params.midden)
            + hh.midden.ash_kg;
        let law = &self.polities[pi].laws[li];
        if dark || kg <= 0.0 || Self::tipped_lately(hh, &def, day) || !law.knows(person) {
            return Some(None);
        }
        let stance = law
            .stances
            .iter()
            .find(|r| r.person == person)
            .map_or(0.0, |r| r.stance.sign());
        let regard = self.ties.regard(person, law.sponsor, day, &ctx.params.ties);
        let norm = self.norm_points(ctx, person);
        let points =
            crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity).max(0.0);
        if points <= 0.0 {
            return Some(None);
        }
        Some(Some(WatchFacts {
            stops: vec![self.polities[pi].tips[ti].place],
            points,
        }))
    }

    /// Person `person` of household `household`, standing at `pos`, carried its heap to the tip of
    /// the law where it lives (ADR-0038 §3), if they stand at it: the heap's decaying share and its
    /// ash join the tip's, and what it held of each disease moves to the tip's node, recorded.
    /// Returns whether a law carrying refuse to a tip applies to the household.
    pub(super) fn carry_to_tip(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        pos: (f32, f32),
    ) -> bool {
        let Some((pi, _, ti, _)) = self.household(household).and_then(|x| self.tip_law(ctx, x))
        else {
            return false;
        };
        let place = self.polities[pi].tips[ti].place;
        let reach = AT_TIP_CELLS * f64::from(ctx.map.cell_size_m);
        if f64::from(pos.0 - place.0).hypot(f64::from(pos.1 - place.1)) > reach {
            return true;
        }
        self.tip_heap(ctx, household, pi, ti);
        true
    }

    /// Empties household `household`'s heap into tip `ti` of polity `pi` now.
    fn tip_heap(&mut self, ctx: &Ctx, household: PermanentId, pi: usize, ti: usize) {
        self.tip_share(ctx, household, pi, ti, 1.0);
    }

    /// Moves share `share` (0-1) of household `household`'s heap, its decaying share and its ash,
    /// into tip `ti` of polity `pi` now, with that share of what it holds of each disease,
    /// recorded as carried (ADR-0038 §3, and a refuse round's call, §4); the heap counts as
    /// tipped today. Returns the kilograms moved of its decaying share and of its ash.
    pub(super) fn tip_share(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        pi: usize,
        ti: usize,
        share: f64,
    ) -> (f64, f64) {
        let day = ctx.now.day_index();
        let share = share.clamp(0.0, 1.0);
        let Some(x) = self.household_mut(household) else {
            return (0.0, 0.0);
        };
        let members = x.members.len();
        x.midden.settle(ctx.now, members, &ctx.params.midden);
        let (kg, ash_kg) = if share >= 1.0 {
            (x.midden.kg, x.midden.ash_kg)
        } else {
            (x.midden.kg * share, x.midden.ash_kg * share)
        };
        if share >= 1.0 {
            x.midden.kg = 0.0;
            x.midden.ash_kg = 0.0;
        } else {
            x.midden.kg -= kg;
            x.midden.ash_kg -= ash_kg;
        }
        x.midden.tipped = day;
        let tip = &mut self.polities[pi].tips[ti];
        tip.kg += kg;
        tip.ash_kg += ash_kg;
        if let Err(i) = tip.households.binary_search(&household) {
            tip.households.insert(i, household);
        }
        let to = Node::Tip(tip.id);
        let from = Node::Midden(household);
        let diseases: Vec<u16> = self
            .contagion
            .loads
            .keys()
            .filter(|(n, _)| *n == from)
            .map(|&(_, d)| d)
            .collect();
        for disease in diseases {
            let amount = self.contagion.take(from, disease, share);
            self.contagion.add(to, disease, amount);
            self.contagion.record(Move {
                day,
                disease,
                how: How::Carried,
                from,
                to,
                amount,
            });
        }
        self.changed(household);
        (kg, ash_kg)
    }

    /// The day's breaches of each law carrying refuse to a tip (ADR-0038 §3): a household of its
    /// settlement whose heap holds anything and was not carried to the open tip within the law's
    /// days, counted from the tip's opening, breaks it.
    pub(super) fn tips_day(&mut self, ctx: &Ctx, day: i64) {
        let laws: Vec<(usize, usize, PermanentId, TipDef, i64)> = self
            .polities
            .iter()
            .enumerate()
            .flat_map(|(pi, p)| {
                p.laws
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| {
                        l.status == LawStatus::InForce && l.kind == PolicyKind::RefuseToTip
                    })
                    .filter_map(move |(li, l)| {
                        let opened = p.tips.iter().find(|t| t.law == l.id)?.opened;
                        Some((pi, li, p.settlement, l.policy, opened))
                    })
            })
            .filter_map(|(pi, li, s, policy, opened)| {
                let def = ctx.catalog.policies.get(usize::from(policy))?.tip?;
                Some((pi, li, s, def, opened))
            })
            .collect();
        for (pi, li, settlement, def, opened) in laws {
            if day - opened < i64::from(def.every_days.max(1)) {
                continue;
            }
            let broken = self
                .households
                .iter()
                .map(|(_, x)| x)
                .filter(|x| x.settlement == Some(settlement) && !x.members.is_empty())
                .filter(|x| !Self::tipped_lately(x, &def, day))
                .filter(|x| {
                    x.midden
                        .at_time(ctx.now, x.members.len(), &ctx.params.midden)
                        + x.midden.ash_kg
                        > 0.0
                })
                .count() as u32;
            if let Some(law) = self.polities[pi].laws.get_mut(li) {
                law.compliance.broken += broken;
            }
        }
    }

    /// Those of `households` whose heap, its decaying share and its ash, holds more than the
    /// content's `heap_kg` within its `heap_near_m` of another household's home (the issue
    /// `heaps_near`, ADR-0038 §3), by id; none without a template carrying refuse to a tip.
    pub(super) fn heaps_near(&self, ctx: &Ctx, households: &[PermanentId]) -> Vec<PermanentId> {
        let Some(def) = Self::tip_def(ctx) else {
            return Vec::new();
        };
        let day = ctx.now.day_index();
        let near = f64::from(def.heap_near_m);
        households
            .iter()
            .copied()
            .filter(|&h| {
                let Some(x) = self.household(h) else {
                    return false;
                };
                let kg = x
                    .midden
                    .at_time(ctx.now, x.members.len(), &ctx.params.midden)
                    + x.midden.ash_kg;
                if kg <= f64::from(def.heap_kg) {
                    return false;
                }
                let at = self.midden_place(ctx, x, day);
                self.households.iter().map(|(_, o)| o).any(|o| {
                    o.id != h
                        && !o.members.is_empty()
                        && f64::from(o.home.0 - at.0).hypot(f64::from(o.home.1 - at.1)) <= near
                })
            })
            .collect()
    }

    /// What carrying refuse to a tip would bring each of `households` a year and cost it, kcal
    /// (ADR-0038 §3), as clearing middens is weighed: its people's deaths of disease within the
    /// year, at the template's share it believes a cleaner place would have spared, each weighed
    /// as a licence's death avoided; and the template's hours a carrying every `every_days` days
    /// of the year, at what an hour of its work brings in food.
    pub(super) fn tips_forecasts(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
    ) -> Vec<(PermanentId, f64, f64)> {
        let Some(def) = Self::tip_def(ctx) else {
            return Vec::new();
        };
        let params = ctx.params;
        let since = ctx.now.day_index() - DAYS_PER_YEAR;
        let day_h = params.farm.work_hours_per_day;
        let mut deaths: Vec<(PermanentId, usize)> = Vec::new();
        for e in self
            .sickness
            .episodes()
            .iter()
            .rev()
            .take_while(|e| e.infected >= since - 120)
        {
            let (Some((d, Outcome::Died)), Some(h)) = (e.ended, e.household) else {
                continue;
            };
            if d <= since || !households.contains(&h) {
                continue;
            }
            match deaths.iter_mut().find(|x| x.0 == h) {
                Some(x) => x.1 += 1,
                None => deaths.push((h, 1)),
            }
        }
        households
            .iter()
            .filter_map(|&h| {
                let x = self.household(h)?;
                let died = deaths.iter().find(|d| d.0 == h).map_or(0, |d| d.1);
                let kcal = self.hour_kcal(ctx, x);
                let gain = died as f64
                    * f64::from(def.avoided_share)
                    * params.suspicion.w_death
                    * day_h
                    * kcal;
                let cost = DAYS_PER_YEAR as f64 / f64::from(def.every_days.max(1))
                    * f64::from(def.carry_h)
                    * kcal;
                Some((h, gain, cost))
            })
            .collect()
    }

    /// Opens the tip of law `law` of polity `pi` as passing it would: for tests.
    #[doc(hidden)]
    pub fn open_tip_for_tests(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        self.open_tip(ctx, pi, law);
    }

    /// Where a law of the content's terms would open the tip of settlement `settlement`: for
    /// tests.
    #[doc(hidden)]
    pub fn tip_site_for_tests(&self, ctx: &Ctx, settlement: PermanentId) -> Option<(f32, f32)> {
        Self::tip_def(ctx).and_then(|d| self.tip_site(ctx, settlement, &d))
    }

    /// Person `person` carries their household's heap to `pos`, as arriving there with it does:
    /// for tests.
    #[doc(hidden)]
    pub fn carry_midden_for_tests(&mut self, ctx: &Ctx, person: PermanentId, pos: (f32, f32)) {
        self.carry_midden(ctx, person, pos);
    }

    /// The households of `households` with the issue `heaps_near` now: for tests.
    #[doc(hidden)]
    pub fn heaps_near_for_tests(&self, ctx: &Ctx, households: &[PermanentId]) -> Vec<PermanentId> {
        self.heaps_near(ctx, households)
    }
}
