//! How households come to know other settlements (ADR-0018 §4; M5a slice AM): by walking within
//! sight of them (in `start_walk`), by being told at the hearth, by founding alongside them, and
//! by going there. A visit is presence, never residence (ADR-0018 §2): the visitor keeps their
//! household and settlement, keeps company at the other hearth like anyone there, and walks home
//! the same day.

use super::*;
use crate::decide::VisitOption;
use crate::places::PlaceHow;
use civ_core::time::DAYS_PER_YEAR;

/// Purpose tag for the draw of whether a place is told at the hearth.
pub const PURPOSE_PLACE: u64 = 0x706c_6163_6530_3031; // "place001"
/// Purpose tag for the draw of whether a price report is told at the hearth (M5b slice AP).
pub const PURPOSE_REPORT: u64 = 0x7072_6963_6530_3031; // "price001"

impl Population {
    /// `a` and `b` keep company at the hearth: each learns where the other is from, if it is
    /// another settlement, and each tells the other of the places their household knows that
    /// the other's does not, each by the content's chance (ADR-0018 §4).
    pub(crate) fn share_places(&mut self, ctx: &Ctx, a: PermanentId, b: PermanentId) {
        let home_of = |p: PermanentId| {
            let q = self.person(p)?;
            Some((q.household, self.household(q.household)?.settlement))
        };
        let (Some(ha), Some(hb)) = (home_of(a), home_of(b)) else {
            return;
        };
        let mut learnt: Vec<(PermanentId, PermanentId, PermanentId)> = Vec::new();
        for ((teller, (th, ts)), (listener, (lh, ls))) in [((a, ha), (b, hb)), ((b, hb), (a, ha))] {
            // Someone from elsewhere says where they are from.
            if let Some(s) = ts
                && Some(s) != ls
                && !self.known_places.knows(lh, s)
            {
                learnt.push((lh, s, teller));
            }
            for k in self.known_places.of(th) {
                if Some(k.settlement) == ls || self.known_places.knows(lh, k.settlement) {
                    continue;
                }
                let key = [
                    ctx.seed,
                    PURPOSE_PLACE,
                    teller.get(),
                    listener.get(),
                    k.settlement.get(),
                    ctx.now.minutes() as u64,
                ];
                if Rng64::from_key(&key).next_f64() < ctx.params.places.share_told {
                    learnt.push((lh, k.settlement, teller));
                }
            }
        }
        let day = ctx.now.day_index();
        for (household, settlement, from) in learnt {
            self.known_places
                .learn(household, settlement, day, PlaceHow::Told, Some(from));
        }
    }

    /// `a` and `b` keep company at the hearth (M5b slice AP, ADR-0019 §1): someone from another
    /// settlement says what their household and its workshops offer, and the listener's household
    /// holds it as a report told by them; and each tells the other, with the content's chance, of
    /// an offer elsewhere their household holds a newer report of than the other's does, dated
    /// as it was seen (telling never makes a price fresher). Nobody hears of offers in their own
    /// settlement this way: those they know as they stand.
    pub(crate) fn share_reports(&mut self, ctx: &Ctx, a: PermanentId, b: PermanentId) {
        use crate::reports::{PriceReport, ReportHow};
        let home_of = |p: PermanentId| {
            let q = self.person(p)?;
            Some((q.household, self.household(q.household)?.settlement?))
        };
        let (Some(ha), Some(hb)) = (home_of(a), home_of(b)) else {
            return;
        };
        let day = ctx.now.day_index();
        let mut learnt: Vec<(PermanentId, PriceReport)> = Vec::new();
        for ((teller, (th, ts)), (listener, (lh, ls))) in [((a, ha), (b, hb)), ((b, hb), (a, ha))] {
            if ts != ls {
                // What their own household and its workshops offer.
                let own = self
                    .household(th)
                    .map(|x| (x.id, false, x.offers.as_slice()))
                    .into_iter()
                    .chain(
                        self.firms
                            .iter()
                            .filter(|f| f.owner == th && f.is_open())
                            .map(|f| (f.id, true, f.offers.as_slice())),
                    );
                for (seller, firm, offers) in own {
                    for o in offers {
                        learnt.push((
                            lh,
                            PriceReport {
                                market: ts,
                                seller,
                                firm,
                                good: o.good,
                                payment: o.payment,
                                price: o.price,
                                ask_h: o.ask_h,
                                units: o.units,
                                day,
                                how: ReportHow::Told,
                                from: Some(teller),
                            },
                        ));
                    }
                }
            }
            for r in self.reports.of(th) {
                if r.market == ls {
                    continue;
                }
                let held = self.reports.of(lh);
                let theirs = held
                    .binary_search_by_key(&r.key(), PriceReport::key)
                    .ok()
                    .map(|i| &held[i]);
                if theirs.is_some_and(|x| x.day >= r.day) {
                    continue;
                }
                // Keyed by market and good, not payment: a good's terms are told together.
                let key = [
                    ctx.seed,
                    PURPOSE_REPORT,
                    teller.get(),
                    listener.get(),
                    r.market.get(),
                    u64::from(r.good),
                    ctx.now.minutes() as u64,
                ];
                if Rng64::from_key(&key).next_f64() < ctx.params.reports.share_told {
                    learnt.push((
                        lh,
                        PriceReport {
                            how: ReportHow::Told,
                            from: Some(teller),
                            ..*r
                        },
                    ));
                }
            }
        }
        for (household, report) in learnt {
            self.reports.note(household, report);
        }
    }

    /// The founding groups `founded` came to the valley together and know where each other
    /// camped (the new-world choice of ADR-0018 §6).
    pub fn know_each_other(&mut self, founded: &[crate::found::Founded], day: i64) {
        for f in founded {
            for g in founded.iter().filter(|g| g.settlement != f.settlement) {
                for &h in &f.households {
                    self.known_places
                        .learn(h, g.settlement, day, PlaceHow::Founded, None);
                }
            }
        }
    }

    /// A walk through `points` by a member of `household` passes within sight of the homes of
    /// any other settlement still lived in: the household knows it (ADR-0018 §4).
    pub(crate) fn see_places(&mut self, ctx: &Ctx, household: PermanentId, points: &[(f32, f32)]) {
        if ctx.land.settlements.len() < 2 {
            return;
        }
        let home = self.household(household).and_then(|x| x.settlement);
        let day = ctx.now.day_index();
        for s in &ctx.land.settlements {
            if Some(s.id) != home
                && s.abandoned.is_none()
                && crate::places::distance_to_walk(points, s.hearth_m) <= ctx.params.places.sight_m
            {
                self.known_places
                    .learn(household, s.id, day, PlaceHow::Seen, None);
            }
        }
    }

    /// The other settlements `person` of `household` could visit, from their settlement's
    /// travel field `field_key`: those their household knows that are lived in and lie within
    /// `max_walk_min` of their hearth, each with what those they would see there are worth to
    /// them (kin, those they know, the hope of meeting someone). Empty in the dark.
    pub(crate) fn visit_options(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        household: PermanentId,
        field_key: PermanentId,
        max_walk_min: f64,
        dark: bool,
    ) -> Vec<VisitOption> {
        let known = self.known_places.of(household);
        if dark || known.is_empty() {
            return Vec::new();
        }
        let Some(home) = self.household(household).and_then(|x| x.settlement) else {
            return Vec::new();
        };
        let Some(reach) = self.homes.get(&field_key).map(|f| &f.reach) else {
            return Vec::new();
        };
        let mut out: Vec<(VisitOption, Option<i64>)> = Vec::new();
        for k in known {
            let Some(s) = ctx
                .land
                .settlements
                .iter()
                .find(|x| x.id == k.settlement && x.abandoned.is_none())
            else {
                continue;
            };
            let Some(secs) = reach.seconds_to(cell_of(ctx.map, s.hearth_m)) else {
                continue;
            };
            let walk_min = f64::from(secs) / 60.0;
            if walk_min <= max_walk_min {
                out.push((
                    VisitOption {
                        settlement: s.id,
                        hearth: s.hearth_m,
                        walk_min,
                        points: 0.0,
                    },
                    k.visited,
                ));
            }
        }
        if out.is_empty() {
            return Vec::new();
        }
        let pp = &ctx.params.places;
        let day = ctx.now.day_index();
        let seeking = self.seeks_elsewhere(ctx, person);
        let kin = self.close_kin(person);
        let lives_in = |q: PermanentId| {
            self.person(q)
                .and_then(|p| self.household(p.household))
                .and_then(|x| x.settlement)
        };
        let mut known_in: Vec<(PermanentId, f64)> = Vec::new();
        for t in self.ties.of(person) {
            let Some(s) = lives_in(t.to).filter(|&s| s != home) else {
                continue;
            };
            let k = t.known_at(day, &ctx.params.ties);
            match known_in.iter_mut().find(|(x, _)| *x == s) {
                Some(e) => e.1 += k,
                None => known_in.push((s, k)),
            }
        }
        out.into_iter()
            .map(|(mut v, visited)| {
                let kin_there = kin
                    .iter()
                    .filter(|&&q| lives_in(q) == Some(v.settlement))
                    .count();
                let known = known_in
                    .iter()
                    .find(|(s, _)| *s == v.settlement)
                    .map_or(0.0, |e| e.1);
                let since = visited.map(|d| day - d);
                v.points = pp.company_points(kin_there, known, seeking, since);
                v
            })
            .collect()
    }

    /// Whether `person` is an unpartnered adult who looked for a partner at home within the
    /// content's days and found nobody (research 04-08 §1.1).
    pub(crate) fn seeks_elsewhere(&self, ctx: &Ctx, person: PermanentId) -> bool {
        let Some(p) = self.person(person) else {
            return false;
        };
        p.partner.is_none()
            && ctx.params.family.seeks_at(p.sex, p.age_years(ctx.now))
            && self
                .unmatched
                .get(&person)
                .is_some_and(|&d| ctx.now.day_index() - d <= ctx.params.places.seek_days)
    }

    /// The living parents, children, brothers and sisters of `person`, from the records of
    /// everyone who has lived (children through the index [`Self::refresh_kin`] keeps).
    pub(crate) fn close_kin(&self, person: PermanentId) -> Vec<PermanentId> {
        let Some(r) = self.records.get(&person) else {
            return Vec::new();
        };
        let children = |q: PermanentId| self.children.get(&q).map_or(&[][..], Vec::as_slice);
        let mut out: Vec<PermanentId> = Vec::new();
        for parent in [r.mother, r.father].into_iter().flatten() {
            out.push(parent);
            out.extend(children(parent).iter().copied().filter(|&c| c != person));
        }
        out.extend_from_slice(children(person));
        out.sort_unstable();
        out.dedup();
        out.retain(|&q| self.person(q).is_some());
        out
    }

    /// Has the index of everyone's children rebuilt when next wanted: for when a record's parents
    /// change after it was written.
    pub fn forget_kin(&mut self) {
        self.children_indexed = usize::MAX;
    }

    /// Brings the index of everyone's children up to date with the records (derived; rebuilt
    /// whenever the records have grown).
    pub(crate) fn refresh_kin(&mut self) {
        if self.children_indexed == self.records.len() {
            return;
        }
        self.children.clear();
        for (&id, r) in &self.records {
            for parent in [r.mother, r.father].into_iter().flatten() {
                self.children.entry(parent).or_default().push(id);
            }
        }
        self.children_indexed = self.records.len();
    }

    /// `me` came to the hearth of `settlement`, where `present` are, for `minutes` (M5a slice
    /// AM): their household knows it as of today, and what they saw of its food (the share of
    /// those of it there who were not going hungry, if any were), and the visit is counted.
    pub(crate) fn visited(
        &mut self,
        ctx: &Ctx,
        me: PermanentId,
        settlement: PermanentId,
        present: &[PermanentId],
        minutes: u32,
    ) {
        let Some((household, home)) = self
            .person(me)
            .and_then(|p| Some((p.household, self.household(p.household)?.settlement?)))
        else {
            return;
        };
        if home == settlement {
            return;
        }
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        let (mut met, mut fed) = (0u32, 0u32);
        for q in present.iter().filter_map(|&q| self.person(q)) {
            if self.household(q.household).and_then(|x| x.settlement) == Some(settlement) {
                met += 1;
                if !may_eat_reserve(q, now, params) {
                    fed += 1;
                }
            }
        }
        self.known_places
            .learn(household, settlement, day, PlaceHow::Visited, None);
        self.known_places.went(
            household,
            settlement,
            day,
            (met > 0).then(|| fed as f32 / met as f32),
        );
        self.contacts
            .visit(day.div_euclid(DAYS_PER_YEAR), home, settlement, minutes);
        // What they see of its new buildings from its hearth (M5b slice AR).
        if let Some(at) = ctx
            .land
            .settlements
            .iter()
            .find(|x| x.id == settlement)
            .map(|x| x.hearth_m)
        {
            self.note_sights(ctx, me, settlement, at);
            // And the work its people are doing about it.
            self.watch_work(ctx, me, settlement, at);
        }
    }
}
