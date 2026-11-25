//! Help with mending (research 11-04 §1.6: households "recruit relatives or neighbors", and
//! reciprocal help makes an obligation, not free labour; 11-01: communal work parties). A
//! household with a building to mend and nobody who may do the work, by a build activity's ages
//! or the building's technique, asks for help: its nearest kin household first, then a
//! neighbour its people hold a tie with, the best regarded first. The one asked must have
//! someone who may do the work, and gives it as it gives food: unless it believes the asker took
//! from it or from those it regards ([`Population::refuses`]). Its people mend the building
//! through the same work as their own, with their household's materials, and once it is
//! mended the asker's household owes them the hours (`helped_mend`, `gave_mend`). A household
//! with no home, standing or begun, and nobody of an age to build asks the same way for help with
//! a first home: the one asked begins the asker's home, where and as the asker would, on the
//! asker's ground, and builds it with its own materials while it is unfinished, the asker owing
//! it each stage's work. Who asks whom is worked out afresh each day and not saved: it reads
//! everyone's age as at the day's start, and is worked out again once a household's members
//! change, so a world loaded that day finds the same (ADR-0011 §4, §8 25's first item).

use super::*;
use crate::ties::Act;

impl Population {
    /// Whether person `p` may do building work to `program` at `at`: of an age a build activity
    /// admits, and knowing its technique if it needs one.
    fn may_build(ctx: &Ctx, p: &Person, program: &str, at: SimTime) -> bool {
        let catalog = ctx.catalog;
        Self::of_building_age(ctx, p, at)
            && catalog
                .building_index(program)
                .and_then(|i| catalog.buildings.get(i))
                .is_some_and(|d| d.technique.is_none_or(|t| p.knows(t)))
    }

    /// Whether person `p` is of an age a build activity admits at `at`.
    fn of_building_age(ctx: &Ctx, p: &Person, at: SimTime) -> bool {
        let age = p.age_years(at);
        ctx.catalog.activities.iter().any(|a| {
            a.behavior == Behavior::Build && age >= a.min_age_years && age <= a.max_age_years
        })
    }

    /// Whether someone of `members` may do building work to `program` now
    /// ([`Population::may_build`]).
    pub(super) fn someone_may_build(
        &self,
        ctx: &Ctx,
        members: &[PermanentId],
        program: &str,
    ) -> bool {
        self.someone_may_build_at(ctx, members, program, ctx.now)
    }

    /// Whether someone of `members` may do building work to `program` at `at`.
    fn someone_may_build_at(
        &self,
        ctx: &Ctx,
        members: &[PermanentId],
        program: &str,
        at: SimTime,
    ) -> bool {
        members
            .iter()
            .filter_map(|m| self.person(*m))
            .any(|p| Self::may_build(ctx, p, program, at))
    }

    /// Whether someone of `members` may do building work on `b` now ([`Population::may_build`]).
    pub(super) fn someone_may_mend(
        &self,
        ctx: &Ctx,
        members: &[PermanentId],
        b: &Building,
    ) -> bool {
        self.someone_may_build_at(ctx, members, &b.spec.program, ctx.now)
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
    /// and does not refuse it. So does a household with no home standing or begun, or with one
    /// begun that nobody of it may work on, and nobody of an age to build: the one it asks must
    /// have someone of that age, and someone who may work on the home begun. Ages are read as at
    /// the day's start, so that whenever in the day the asks are worked out, in this world or in
    /// one loaded that day, they are the same; a change of a household's members has them worked
    /// out again ([`Population::members_changed`]).
    fn mend_asks_today(&mut self, ctx: &Ctx) {
        let day = ctx.now.day_index();
        if self.mend_asks.0 == Some(day) {
            return;
        }
        let at = SimTime::from_minutes(day * MINUTES_PER_DAY);
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
                .filter(|b| !self.someone_may_build_at(ctx, &x.members, &b.spec.program, at))
                .collect();
            // A first home: none of its own standing or begun that someone of it may work on,
            // and nobody of it of an age to build one.
            let unhoused = !x
                .members
                .iter()
                .filter_map(|m| self.person(*m))
                .any(|p| Self::of_building_age(ctx, p, at))
                && !ctx.land.buildings.iter().any(|b| {
                    b.household == id
                        && catalog.is_dwelling(&b.spec.program)
                        && b.finished()
                        && b.standing()
                });
            let begun: Vec<&Building> = if unhoused {
                ctx.land
                    .buildings
                    .iter()
                    .filter(|b| {
                        b.household == id && !b.finished() && catalog.is_dwelling(&b.spec.program)
                    })
                    .collect()
            } else {
                Vec::new()
            };
            if wanting.is_empty() && !unhoused {
                continue;
            }
            let nothing_begun = begun.is_empty();
            let wanting: Vec<&Building> = wanting.into_iter().chain(begun).collect();
            let Some(elder) = self.elder_of(id, at, ctx.params) else {
                continue;
            };
            let candidates = self.mend_candidates(ctx, x, at);
            let mut helper = candidates
                .iter()
                .copied()
                .find(|&h| self.may_be_asked(ctx, h, &wanting, unhoused, elder, at));
            // With no home begun, the one asked must also be able to build the home it would
            // begin (§8 25, second; as [`Population::first_home_plan`] requires of it), or the
            // same one is asked every day and builds nothing. Found only once someone may be
            // asked, where that one's own plan would look for it.
            if unhoused
                && nothing_begun
                && helper.is_some()
                && let Some(site) = self.first_home_site(ctx, id)
            {
                let program = site.spec.program;
                helper = candidates.iter().copied().find(|&h| {
                    self.may_be_asked(ctx, h, &wanting, unhoused, elder, at)
                        && self.household(h).is_some_and(|hx| {
                            self.someone_may_build_at(ctx, &hx.members, &program, at)
                        })
                });
            }
            if let Some(h) = helper {
                asks.entry(h).or_default().push(id);
            }
        }
        self.mend_asks = (Some(day), asks);
    }

    /// Whether household `h` may be asked to do the building work `wanting` for a household whose
    /// elder is `elder`: someone of it may work on each, someone of it is of an age to build if
    /// the one asking is `unhoused`, and it does not refuse that elder; ages as at `at`.
    fn may_be_asked(
        &self,
        ctx: &Ctx,
        h: PermanentId,
        wanting: &[&Building],
        unhoused: bool,
        elder: PermanentId,
        at: SimTime,
    ) -> bool {
        self.household(h).is_some_and(|hx| {
            wanting
                .iter()
                .all(|b| self.someone_may_build_at(ctx, &hx.members, &b.spec.program, at))
                && (!unhoused
                    || hx
                        .members
                        .iter()
                        .filter_map(|m| self.person(*m))
                        .any(|p| Self::of_building_age(ctx, p, at)))
        }) && !self.refuses(ctx, h, elder)
    }

    /// Whom household `x` would ask to mend, in order: its people's close kin's households of
    /// its settlement, the nearest first, then the households of its settlement whose elders its
    /// people hold a tie with, the best regarded first (ties: the lower id). Elders by their ages
    /// at `at`.
    fn mend_candidates(&self, ctx: &Ctx, x: &Household, at: SimTime) -> Vec<PermanentId> {
        let params = ctx.params;
        let day = ctx.now.day_index();
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
                if self.elder_of(h, at, params) != Some(t.to) {
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
