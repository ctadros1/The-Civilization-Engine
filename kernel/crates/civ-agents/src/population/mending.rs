//! Help with mending (research 11-04 §1.6: households "recruit relatives or neighbors", and
//! reciprocal help makes an obligation, not free labour; 11-01: communal work parties). A
//! household with a building to mend and nobody who may do the work, by a build activity's ages
//! or the building's technique, asks for help: its nearest kin household first, then a
//! neighbour its people hold a tie with, the best regarded first. The one asked must have
//! someone who may do the work, and gives it as it gives food: unless it believes the asker took
//! from it or from those it regards ([`Population::refuses`]). Its people mend the building
//! through the same work as their own, with their household's materials, and once it is
//! mended the asker's household owes them the hours (`helped_mend`, `gave_mend`). Who asks
//! whom is worked out afresh each day and not saved.

use super::*;
use crate::ties::Act;

impl Population {
    /// Whether person `p` may do building work on `b` now: of an age a build activity admits,
    /// and knowing `b`'s technique if it needs one.
    pub(super) fn may_mend(&self, ctx: &Ctx, p: &Person, b: &Building) -> bool {
        let catalog = ctx.catalog;
        let age = p.age_years(ctx.now);
        let of_age = catalog.activities.iter().any(|a| {
            a.behavior == Behavior::Build && age >= a.min_age_years && age <= a.max_age_years
        });
        of_age
            && catalog
                .building_index(&b.spec.program)
                .and_then(|i| catalog.buildings.get(i))
                .is_some_and(|d| d.technique.is_none_or(|t| p.knows(t)))
    }

    /// Whether someone of `members` may do building work on `b` now ([`Population::may_mend`]).
    pub(super) fn someone_may_mend(
        &self,
        ctx: &Ctx,
        members: &[PermanentId],
        b: &Building,
    ) -> bool {
        members
            .iter()
            .filter_map(|m| self.person(*m))
            .any(|p| self.may_mend(ctx, p, b))
    }

    /// The households asking household `helper` today to mend a building of theirs, worked out
    /// once a day ([`Population::mend_asks_today`]).
    pub(super) fn asking_to_mend(&mut self, ctx: &Ctx, helper: PermanentId) -> Vec<PermanentId> {
        self.mend_asks_today(ctx);
        self.mend_asks.1.get(&helper).cloned().unwrap_or_default()
    }

    /// Whether household `helper` mends household `owner`'s buildings today.
    pub(super) fn mends_for(&mut self, ctx: &Ctx, helper: PermanentId, owner: PermanentId) -> bool {
        self.asking_to_mend(ctx, helper).contains(&owner)
    }

    /// Works out, once a day, who asks whom to mend: each household with a finished building
    /// standing whose wear shows, or with a repair begun, and nobody who may do the work, asks
    /// the first of its candidates ([`Population::mend_candidates`]) that has someone who may
    /// and does not refuse it.
    fn mend_asks_today(&mut self, ctx: &Ctx) {
        let day = ctx.now.day_index();
        if self.mend_asks.0 == Some(day) {
            return;
        }
        self.refresh_kin();
        let catalog = ctx.catalog;
        let mut asks: FastMap<PermanentId, Vec<PermanentId>> = FastMap::default();
        let mut ids: Vec<PermanentId> = self.households.iter().map(|(_, x)| x.id).collect();
        ids.sort_unstable();
        for id in ids {
            let Some(x) = self.household(id) else {
                continue;
            };
            if x.members.is_empty() {
                continue;
            }
            let wanting: Vec<&Building> = ctx
                .land
                .buildings
                .iter()
                .filter(|b| b.household == id && b.finished() && b.standing())
                .filter(|b| {
                    catalog
                        .building_index(&b.spec.program)
                        .and_then(|d| catalog.buildings.get(d))
                        .is_some_and(|d| {
                            b.repair.is_some() || condition::worst(b, &d.upkeep).is_some()
                        })
                })
                .filter(|b| !self.someone_may_mend(ctx, &x.members, b))
                .collect();
            if wanting.is_empty() {
                continue;
            }
            let Some(elder) = self.elder_of(id, ctx.now, ctx.params) else {
                continue;
            };
            let helper = self.mend_candidates(ctx, x).into_iter().find(|&h| {
                self.household(h).is_some_and(|hx| {
                    wanting
                        .iter()
                        .all(|b| self.someone_may_mend(ctx, &hx.members, b))
                }) && !self.refuses(ctx, h, elder)
            });
            if let Some(h) = helper {
                asks.entry(h).or_default().push(id);
            }
        }
        self.mend_asks = (Some(day), asks);
    }

    /// Whom household `x` would ask to mend, in order: its people's close kin's households of
    /// its settlement, the nearest first, then the households of its settlement whose elders its
    /// people hold a tie with, the best regarded first (ties: the lower id).
    fn mend_candidates(&self, ctx: &Ctx, x: &Household) -> Vec<PermanentId> {
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        let near = |h: PermanentId| {
            self.household(h).is_some_and(|o| {
                !o.members.is_empty() && o.settlement.is_some() && o.settlement == x.settlement
            })
        };
        let dist = |h: PermanentId| {
            self.household(h).map_or(f64::INFINITY, |o| {
                f64::from(o.home.0 - x.home.0).hypot(f64::from(o.home.1 - x.home.1))
            })
        };
        let mut kin: Vec<PermanentId> = self
            .kin_households(x)
            .into_iter()
            .filter(|&h| near(h))
            .collect();
        kin.sort_by(|&a, &b| dist(a).total_cmp(&dist(b)).then(a.cmp(&b)));
        let mut tied: Vec<(f64, PermanentId)> = Vec::new();
        for &m in &x.members {
            for t in self.ties.of(m) {
                let Some(h) = self.person(t.to).map(|p| p.household) else {
                    continue;
                };
                if h == x.id || !near(h) || kin.contains(&h) {
                    continue;
                }
                if self.elder_of(h, now, params) != Some(t.to) {
                    continue;
                }
                let regard = t.regard_at(day, &params.ties);
                match tied.iter_mut().find(|(_, o)| *o == h) {
                    Some(e) => e.0 = e.0.max(regard),
                    None => tied.push((regard, h)),
                }
            }
        }
        tied.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        kin.extend(tied.into_iter().map(|(_, h)| h));
        kin
    }

    /// Household `owner`'s building `building` was mended by `who` of household `helper`, which
    /// it asked: `owner`'s elder sees `helper`'s give the help, and owes it `hours` of work.
    pub(super) fn mended_for(
        &mut self,
        ctx: &Ctx,
        owner: PermanentId,
        helper: PermanentId,
        hours: f64,
    ) {
        self.note_between(
            ctx,
            None,
            owner,
            helper,
            Act::HelpedMend,
            Act::GaveMend,
            hours,
        );
    }
}
