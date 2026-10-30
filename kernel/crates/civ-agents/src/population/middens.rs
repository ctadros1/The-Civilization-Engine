//! Middens cleared from the water (M10c slice EE, third part; ADR-0035 §7; brief m10-services
//! §2.4; research 05-05 §1.7): where someone suspects a source of water, anyone may propose that
//! a household whose home lies within some metres of an open well or a stream carries its midden
//! away, to a place as far from every source. It is kept by choice: one who knows the law weighs
//! carrying the heap out by what keeping it weighs with them, and a heap carried within the law's
//! days lies at the place it was carried to, washing and seeping toward the water from there; one
//! not carried lies at home, as before. A household near the water that did not carry its heap
//! within the days breaks the law; nobody brings it before anyone here (ADR-0025: cases are
//! brought). A household weighs the law by a share of its deaths of disease within the year it
//! believes the cleaner water would have spared, against the hours its people would carry.

use super::*;
use crate::decide::WatchFacts;
use crate::polity::{LawStatus, MiddensDef, PolicyKind};
use crate::sickness::Outcome;

impl Population {
    /// The terms of the first template that clears middens, if the content has one.
    fn middens_def(ctx: &Ctx) -> Option<MiddensDef> {
        ctx.catalog.policies.iter().find_map(|d| d.middens)
    }

    /// The nearest open well or stream cell within `radius_m` of `at`, if any: its place.
    pub(super) fn nearest_source(
        ctx: &Ctx,
        at: (f32, f32),
        radius_m: f64,
    ) -> Option<((f32, f32), f64)> {
        let (ax, ay) = (f64::from(at.0), f64::from(at.1));
        let mut best: Option<((f32, f32), f64)> = None;
        let mut take = |p: (f32, f32), d: f64| {
            if d <= radius_m && best.is_none_or(|b| d < b.1) {
                best = Some((p, d));
            }
        };
        for w in ctx.land.wells.list.iter().filter(|w| w.is_open()) {
            let (wx, wy) = w.rect.centre_m();
            take((wx, wy), (f64::from(wx) - ax).hypot(f64::from(wy) - ay));
        }
        let map = ctx.map;
        let size = f64::from(map.cell_size_m);
        let (cx, cy) = ((ax / size).floor() as i64, (ay / size).floor() as i64);
        let span = (radius_m / size).ceil() as i64;
        let (w, h) = (i64::from(map.width), i64::from(map.height));
        for y in (cy - span).max(0)..=(cy + span).min(h - 1) {
            for x in (cx - span).max(0)..=(cx + span).min(w - 1) {
                if map.water[(y * w + x) as usize] != WATER_RIVER {
                    continue;
                }
                let (px, py) = ((x as f64 + 0.5) * size, (y as f64 + 0.5) * size);
                take((px as f32, py as f32), (px - ax).hypot(py - ay));
            }
        }
        best
    }

    /// Where household home `home` would carry its heap under terms `def`: a place on land at
    /// least `distance_m` from every source, along the line from the nearest source through the
    /// home or a few turns from it; none where the home is that far already, or no such place is
    /// found.
    fn midden_site(ctx: &Ctx, home: (f32, f32), def: &MiddensDef) -> Option<(f32, f32)> {
        let distance = f64::from(def.distance_m);
        let (source, d) = Self::nearest_source(ctx, home, distance)?;
        let (hx, hy) = (f64::from(home.0), f64::from(home.1));
        let (dx, dy) = if d > 1e-6 {
            (
                (hx - f64::from(source.0)) / d,
                (hy - f64::from(source.1)) / d,
            )
        } else {
            (1.0, 0.0)
        };
        let map = ctx.map;
        let (w, h) = (f64::from(map.width), f64::from(map.height));
        let size = f64::from(map.cell_size_m);
        for turn in [0.0_f64, 0.5, -0.5, 1.0, -1.0, 1.5, -1.5] {
            let (s, c) = (turn * std::f64::consts::FRAC_PI_4 * 2.0).sin_cos();
            let (ux, uy) = (dx * c - dy * s, dx * s + dy * c);
            let step = distance - d.min(distance) + size;
            let (x, y) = (hx + ux * step, hy + uy * step);
            if x < 0.0 || y < 0.0 || x >= w * size || y >= h * size {
                continue;
            }
            let at = (x as f32, y as f32);
            if map.water[cell_of(map, at)] != WATER_LAND {
                continue;
            }
            if Self::nearest_source(ctx, at, distance).is_none() {
                return Some(at);
            }
        }
        None
    }

    /// The law clearing middens where household `hh` lives and its index, if one is in force.
    fn middens_law(&self, ctx: &Ctx, hh: &Household) -> Option<(usize, usize, MiddensDef)> {
        let pi = self.polity_of(hh.settlement?)?;
        let li = self.polities[pi]
            .laws
            .iter()
            .position(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::ClearMiddens)?;
        let def = ctx
            .catalog
            .policies
            .get(usize::from(self.polities[pi].laws[li].policy))
            .and_then(|d| d.middens)?;
        Some((pi, li, def))
    }

    /// Whether household `x`'s heap was carried away within the days of terms `def` as of
    /// `day`.
    fn midden_carried(x: &Household, def: &MiddensDef, day: i64) -> bool {
        x.midden.carried >= 0 && day - x.midden.carried <= i64::from(def.keep_days)
    }

    /// Where household `x`'s heap lies on `day`: where it was carried, within the content's
    /// days, or at home.
    pub(super) fn midden_place(&self, ctx: &Ctx, x: &Household, day: i64) -> (f32, f32) {
        match Self::middens_def(ctx) {
            Some(def) if Self::midden_carried(x, &def, day) => x.midden.site,
            _ => x.home,
        }
    }

    /// The heap person `person` of household `hh` could carry away now (M10c slice EE, third
    /// part): where a law clearing middens is in force where they live and they know it, in
    /// daylight, their home lies near the water and the heap was not carried within the law's
    /// days, the place to carry it to and what keeping the law weighs with them.
    pub(super) fn midden_facts(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        hh: &Household,
        dark: bool,
    ) -> Option<WatchFacts> {
        // Under a law carrying refuse to an open tip, that law asks instead (M11f, second part;
        // ADR-0038 §3).
        if let Some(tip) = self.tip_facts(ctx, person, hh, dark) {
            return tip;
        }
        if dark {
            return None;
        }
        let (pi, li, def) = self.middens_law(ctx, hh)?;
        let day = ctx.now.day_index();
        let kg = hh
            .midden
            .at_time(ctx.now, hh.members.len(), &ctx.params.midden);
        if Self::midden_carried(hh, &def, day) || kg <= 0.0 {
            return None;
        }
        let law = &self.polities[pi].laws[li];
        if !law.knows(person) {
            return None;
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
            return None;
        }
        let site = Self::midden_site(ctx, hh.home, &def)?;
        Some(WatchFacts {
            stops: vec![site],
            points,
        })
    }

    /// Person `person` carried their household's heap to `pos` (M10c slice EE, third part): to
    /// the tip, under a law carrying refuse to an open one (M11f, second part; ADR-0038 §3).
    pub(super) fn carry_midden(&mut self, ctx: &Ctx, person: PermanentId, pos: (f32, f32)) {
        let day = ctx.now.day_index();
        let Some(h) = self.person(person).map(|p| p.household) else {
            return;
        };
        if self.carry_to_tip(ctx, h, pos) {
            return;
        }
        if let Some(x) = self.household_mut(h) {
            x.midden.carried = day;
            x.midden.site = pos;
        }
        self.changed(h);
    }

    /// The day's breaches of each law clearing middens (M10c slice EE, third part): a household
    /// of its settlement whose home lies near the water and whose heap was not carried within its
    /// days breaks it, unless a law carrying refuse to an open tip asks of it instead (ADR-0038
    /// §3).
    pub(super) fn middens_day(&mut self, ctx: &Ctx, day: i64) {
        let laws: Vec<(usize, usize, PermanentId, MiddensDef)> = self
            .polities
            .iter()
            .enumerate()
            .flat_map(|(pi, p)| {
                p.laws
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| {
                        l.status == LawStatus::InForce && l.kind == PolicyKind::ClearMiddens
                    })
                    .map(move |(li, l)| (pi, li, p.settlement, l.policy))
            })
            .filter_map(|(pi, li, s, policy)| {
                Some((
                    pi,
                    li,
                    s,
                    ctx.catalog.policies.get(usize::from(policy))?.middens?,
                ))
            })
            .collect();
        for (pi, li, settlement, def) in laws {
            let distance = f64::from(def.distance_m);
            let broken = self
                .households
                .iter()
                .map(|(_, x)| x)
                .filter(|x| x.settlement == Some(settlement) && !x.members.is_empty())
                .filter(|x| !Self::midden_carried(x, &def, day))
                .filter(|x| Self::nearest_source(ctx, x.home, distance).is_some())
                .filter(|x| self.tip_law(ctx, x).is_none())
                .count() as u32;
            if let Some(law) = self.polities[pi].laws.get_mut(li) {
                law.compliance.broken += broken;
            }
        }
    }

    /// What clearing middens would bring each of `households` a year and cost it, kcal (M10c
    /// slice EE, third part): its people's deaths of disease within the year, at the template's
    /// share it believes cleaner water would have spared, each weighed as a licence's death
    /// avoided; and, where its home lies near the water, the template's hours a carrying every
    /// `keep_days` days of the year, at what an hour of its work brings in food.
    pub(super) fn middens_forecasts(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
    ) -> Vec<(PermanentId, f64, f64)> {
        let Some(def) = Self::middens_def(ctx) else {
            return Vec::new();
        };
        let params = ctx.params;
        let day = ctx.now.day_index();
        let since = day - DAYS_PER_YEAR;
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
                let near = Self::nearest_source(ctx, x.home, f64::from(def.distance_m)).is_some();
                if died == 0 && !near {
                    return None;
                }
                let kcal = self.hour_kcal(ctx, x);
                let gain = died as f64
                    * f64::from(def.avoided_share)
                    * params.suspicion.w_death
                    * day_h
                    * kcal;
                let cost = if near {
                    DAYS_PER_YEAR as f64 / f64::from(def.keep_days.max(1))
                        * f64::from(def.carry_h)
                        * kcal
                } else {
                    0.0
                };
                Some((h, gain, cost))
            })
            .collect()
    }

    /// Where household `household` would carry its heap under the content's terms, and where
    /// it lies now: for tests.
    #[doc(hidden)]
    pub fn midden_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> (Option<(f32, f32)>, (f32, f32)) {
        let Some(x) = self.household(household) else {
            return (None, (0.0, 0.0));
        };
        let site = Self::middens_def(ctx).and_then(|d| Self::midden_site(ctx, x.home, &d));
        (site, self.midden_place(ctx, x, ctx.now.day_index()))
    }
}
