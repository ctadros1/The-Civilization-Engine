//! Splinter founding (M5a slice AO; ADR-0018 §1, §5; research 05-06 §1.3, 10-01 §1.5, §2.3, §5.2).
//! At its yearly review a household weighs founding a settlement with others beside staying and
//! joining a settlement it knows. Founding is a plan, not a spawn: the household whose plan wins
//! organizes a coalition of its kin's households and those its members regard, each going only if
//! founding with them beats its own best plan; it goes when the plan has won the reviews moving
//! needs, and only if the coalition holds food to its first harvest and months beyond, and seed
//! (10-01 §2.3). Sites are those a member has walked, two field walks from every settlement lived
//! in, forecast at a low share of the believed yield. The new settlement founds its polity under
//! a copy of the body its founders lived under (ADR-0018 §1); nothing else of the parent's polity
//! goes with them.

use super::*;
use crate::places::{Coalition, CoalitionFate, Lacking, PlaceHow};
use crate::{CoalitionStep, ResidenceWhy};
use civ_core::time::DAYS_PER_YEAR;

/// Purpose tag for the sites a household weighs at a review.
pub const PURPOSE_FOUND: u64 = 0x666f_756e_6430_3031; // "found001"

/// Passes over the households that might join a coalition: each pass may bring in kin of those
/// who joined in the one before.
const RECRUIT_PASSES: usize = 3;

/// The ground a plan weighs: the site, the hectares that could be cropped within a field walk of
/// it, and those still vacant within a field walk of home.
#[derive(Clone, Copy)]
struct Ground {
    site: (f32, f32),
    site_ha: f64,
    vacant: f64,
}

/// What founding is worth to a household and what it is made of: the share of the coalition's
/// need the site would bring and the share staying would (each at most 1), its close kin among
/// the households going and staying at home, how well its members know those going and those
/// staying, its most keenly felt grievance, the walk there in hours, and the worth in moving's
/// points.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoundParts {
    pub there: f64,
    pub here: f64,
    pub kin: usize,
    pub kin_home: usize,
    pub known: f64,
    pub known_home: f64,
    pub grievance: f64,
    pub walk_h: f64,
    pub worth: f64,
}

/// A plan to found: where, what it is worth to the household that weighs it, and who would go
/// (that household first) with how many people.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Plan {
    pub site: (f32, f32),
    pub worth: f64,
    pub members: Vec<PermanentId>,
    pub people: u32,
}

impl Population {
    /// The coalition `household` organizes that is still gathering, if any.
    pub(super) fn gathering_of(&self, household: PermanentId) -> Option<usize> {
        self.coalitions
            .iter()
            .position(|c| c.organizer == household && c.fate == CoalitionFate::Gathering)
    }

    /// Whether `household` is counted among those going with a coalition still gathering that a
    /// household other than `organizer` organizes (an organizer is first among its own): it goes
    /// with that one, and neither organizes another nor is asked to one (M5a slice AO).
    pub(super) fn going_with_another(
        &self,
        household: PermanentId,
        organizer: PermanentId,
    ) -> bool {
        self.coalitions.iter().any(|c| {
            c.fate == CoalitionFate::Gathering
                && c.organizer != organizer
                && c.members.contains(&household)
        })
    }

    /// `household`'s best plan to found, if it has a site to weigh: at the site its people know
    /// that would suit them best, the households that would go with it and what that is worth to
    /// it. `best_move` is what its best other plan is worth (at least staying's 0); kin who might
    /// join are only sought when they could carry the plan past it.
    pub(super) fn founding_plan(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        best_move: f64,
    ) -> Option<Plan> {
        let x = self.household(household)?;
        let home = x.settlement?;
        let hearth = ctx.land.settlements.iter().find(|s| s.id == home)?.hearth_m;
        let mut cells = self.founding_sites(ctx, household, hearth);
        if cells.is_empty() {
            return None;
        }
        // The site that would suit its people best, as a founding band chooses one.
        let size = x.members.len() as u32;
        let scores = crate::found::site_scores(
            ctx,
            ctx.params,
            &|c| crate::found::ground_at(ctx, c),
            &cells,
            size,
        );
        let best = (0..cells.len()).max_by(|&a, &b| {
            scores[a]
                .total_cmp(&scores[b])
                .then(cells[b].cmp(&cells[a]))
        })?;
        let site = cell_centre(ctx.map, cells.swap_remove(best));
        let ground = Ground {
            site,
            site_ha: crate::found::cropland_ha(ctx, ctx.params, site),
            vacant: self.vacant_cropland(ctx, hearth),
        };
        let alone = vec![household];
        // Every close kin coming could not carry it past its best plan: no coalition is sought.
        // Going alone leaves every kin and everyone known at home; were they all to come, kin and
        // ties would weigh as much for it as they weigh against it alone.
        let mp = &ctx.params.moving;
        let most = self
            .found_parts(ctx, household, &ground, &alone, x.members.len() as u32)
            .map_or(f64::NEG_INFINITY, |p| {
                p.worth
                    + 2.0 * mp.w_kin * p.kin_home as f64
                    + 2.0 * mp.w_ties * p.known_home / (1.0 + p.known_home)
            });
        if most <= best_move.max(0.0) {
            return None;
        }
        let (members, people) = self.recruit(ctx, household, home, &ground);
        let worth = self.found_worth(ctx, household, &ground, &members, people);
        Some(Plan {
            site,
            worth,
            members,
            people,
        })
    }

    /// [`Self::founding_plan`] for tests: who would go with `household`, their people and what
    /// founding is worth to it, beside a best other plan worth `best_move`.
    #[doc(hidden)]
    pub fn founding_plan_for_tests(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        best_move: f64,
    ) -> Option<(Vec<PermanentId>, u32, f64)> {
        self.refresh_kin();
        self.founding_plan(ctx, household, best_move)
            .map(|p| (p.members, p.people, p.worth))
    }

    /// For tests: the cropland within a field walk of `household`'s home, hectares, what of it is
    /// vacant, and what its people need at the believed yield.
    #[doc(hidden)]
    pub fn founding_land_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Option<(f64, f64, f64)> {
        let x = self.household(household)?;
        let home = x.settlement?;
        let hearth = ctx.land.settlements.iter().find(|s| s.id == home)?.hearth_m;
        Some((
            crate::found::cropland_ha(ctx, ctx.params, hearth),
            self.vacant_cropland(ctx, hearth),
            Self::crop_need_ha(ctx, x.members.len() as u32, 1.0),
        ))
    }

    /// The cells `household` might found at: centres of the patches its people have walked, on
    /// dry, gentle ground people can walk, two field walks from every settlement lived in (so no
    /// field counts for two) and within the content's walk of home; at most the content's number
    /// of them, drawn by a key of the household and the day. The site of a coalition it already
    /// organizes is always among them while it still qualifies.
    fn founding_sites(&self, ctx: &Ctx, household: PermanentId, hearth: (f32, f32)) -> Vec<usize> {
        let Some(x) = self.household(household) else {
            return Vec::new();
        };
        let (fp, band) = (&ctx.params.founding, &ctx.params.band);
        let apart = 2.0 * crate::found::field_reach_m(ctx.params);
        let speed = ctx.nav.params().top_speed_ms().max(0.1);
        let lived: Vec<(f32, f32)> = ctx
            .land
            .settlements
            .iter()
            .filter(|s| s.abandoned.is_none())
            .map(|s| s.hearth_m)
            .collect();
        let qualifies = |cell: usize| {
            let at = cell_centre(ctx.map, cell);
            ctx.map.water[cell] == civ_world::WATER_LAND
                && ctx.nav.walkable(cell)
                && lived
                    .iter()
                    .all(|h| (h.0 - at.0).hypot(h.1 - at.1) >= apart)
                && f64::from((at.0 - hearth.0).hypot(at.1 - hearth.1)) / speed / 3600.0
                    <= fp.walk_hours
                && f64::from(civ_world::terrain::slope_at(ctx.map, cell)) <= band.site_max_slope
        };
        let mut cells: Vec<usize> = x
            .known
            .iter()
            .map(|k| cell_of(ctx.map, ctx.land.patches.centre_m(k.patch as usize)))
            .collect();
        cells.sort_unstable();
        cells.dedup();
        cells.retain(|&c| qualifies(c));
        let day = ctx.now.day_index();
        let key = |c: usize| {
            Rng64::from_key(&[
                ctx.seed,
                PURPOSE_FOUND,
                household.get(),
                day as u64,
                c as u64,
            ])
            .next_u64()
        };
        cells.sort_by_cached_key(|&c| key(c));
        cells.truncate(fp.candidates as usize);
        if let Some(ci) = self.gathering_of(household) {
            let c = cell_of(ctx.map, self.coalitions[ci].site);
            if qualifies(c) && !cells.contains(&c) {
                cells.push(c);
            }
        }
        cells
    }

    /// Hectares that could still be cropped within a field walk of `hearth`: its cropland less
    /// the fields there now, whoever's (none below nothing).
    fn vacant_cropland(&self, ctx: &Ctx, hearth: (f32, f32)) -> f64 {
        let reach = crate::found::field_reach_m(ctx.params);
        let cropped: f64 = ctx
            .land
            .fields
            .iter()
            .filter(|f| {
                let c = f.rect.centre_m();
                (c.0 - hearth.0).hypot(c.1 - hearth.1) <= reach
            })
            .map(civ_land::Field::area_ha)
            .sum();
        (crate::found::cropland_ha(ctx, ctx.params, hearth) - cropped).max(0.0)
    }

    /// Hectares of the content's crop `people` people need, at `share` of its believed yield.
    fn crop_need_ha(ctx: &Ctx, people: u32, share: f64) -> f64 {
        ctx.catalog
            .crops
            .get(ctx.params.farm.crop)
            .map_or(0.0, |c| {
                let kcal = ctx.catalog.goods.get(c.good).map_or(0.0, |g| g.kcal_per_kg);
                crate::farm::need_area_ha(
                    people as usize,
                    ctx.params,
                    c,
                    kcal,
                    c.yield_kg_per_ha * share,
                )
            })
    }

    /// What founding on `ground` with the households `going` (`people` people in all) is worth to
    /// `household`, points (ADR-0018 §5; the people profile's `[founding]`): the share of their
    /// yearly need the site's cropland would bring at a low share of the believed yield, against
    /// the share its own fields and home's vacant cropland would; its most keenly felt grievance;
    /// kin and those known as moving weighs them, its close kin among the households going less
    /// those staying at home and how well its members know those going against those staying;
    /// against moving's cost and founding's besides, and the walk there once.
    fn found_worth(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        ground: &Ground,
        going: &[PermanentId],
        people: u32,
    ) -> f64 {
        self.found_parts(ctx, household, ground, going, people)
            .map_or(f64::NEG_INFINITY, |p| p.worth)
    }

    /// [`Self::found_worth`] and what it is made of.
    fn found_parts(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        ground: &Ground,
        going: &[PermanentId],
        people: u32,
    ) -> Option<FoundParts> {
        let (site, vacant) = (ground.site, ground.vacant);
        let x = self.household(household)?;
        let (mp, fp) = (&ctx.params.moving, &ctx.params.founding);
        let need_there = Self::crop_need_ha(ctx, people.max(1), fp.yield_share);
        let there = if need_there > 0.0 {
            (ground.site_ha / need_there).min(1.0)
        } else {
            1.0
        };
        let stake = self
            .outlook(ctx, household)
            .map_or(0.0, |o| (o.harvest / o.year_need.max(1.0)).clamp(0.0, 1.0));
        let need_here = Self::crop_need_ha(ctx, x.members.len().max(1) as u32, 1.0);
        let here = if need_here > 0.0 {
            (stake + vacant / need_here).min(1.0)
        } else {
            1.0
        };
        // Kin and those known as moving weighs them (ADR-0018 §5, `move_worth`): close kin of a
        // member among the households going less those staying at home, outside the household;
        // how well its members know those going against those staying.
        let home = x.settlement;
        let side = |q: PermanentId| -> Option<bool> {
            let h = self.person(q)?.household;
            if h == household {
                return None;
            }
            if going.contains(&h) {
                Some(true)
            } else {
                let at = self.household(h)?.settlement;
                (at.is_some() && at == home).then_some(false)
            }
        };
        let (mut kin, mut kin_home) = (0usize, 0usize);
        let (mut known, mut known_home) = (0.0, 0.0);
        let day = ctx.now.day_index();
        for &m in &x.members {
            for q in self.close_kin(m) {
                match side(q) {
                    Some(true) => kin += 1,
                    Some(false) => kin_home += 1,
                    None => {}
                }
            }
            for t in self.ties.of(m) {
                let k = t.known_at(day, &ctx.params.ties);
                match side(t.to) {
                    Some(true) => known += k,
                    Some(false) => known_home += k,
                    None => {}
                }
            }
        }
        let walk_h = x.settlement.map_or(0.0, |home| {
            let from = ctx
                .land
                .settlements
                .iter()
                .find(|s| s.id == home)
                .map_or(site, |s| s.hearth_m);
            f64::from((from.0 - site.0).hypot(from.1 - site.1))
                / ctx.nav.params().top_speed_ms().max(0.1)
                / 3600.0
        });
        let grievance = self.grievance_of(ctx, household);
        let sat = |s: f64| s / (1.0 + s);
        let worth = mp.w_stake * (there - here)
            + mp.w_grievance * grievance
            + mp.w_kin * (kin as f64 - kin_home as f64)
            + mp.w_ties * (sat(known) - sat(known_home))
            - mp.cost
            - fp.cost
            - ctx.params.decision.w_walk_hour * walk_h;
        Some(FoundParts {
            there,
            here,
            kin,
            kin_home,
            known,
            known_home,
            grievance,
            walk_h,
            worth,
        })
    }

    /// For tests: what founding at `site` with the households `going` is worth to `household`,
    /// and what it is made of, with `freed_ha` hectares counted vacant at home besides
    /// ([`Self::recruit`]).
    #[doc(hidden)]
    pub fn found_parts_for_tests(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        site: (f32, f32),
        going: &[PermanentId],
        freed_ha: f64,
    ) -> Option<FoundParts> {
        self.refresh_kin();
        let home = self.household(household)?.settlement?;
        let hearth = ctx.land.settlements.iter().find(|s| s.id == home)?.hearth_m;
        let ground = Ground {
            site,
            site_ha: crate::found::cropland_ha(ctx, ctx.params, site),
            vacant: self.vacant_cropland(ctx, hearth) + freed_ha,
        };
        let people = going
            .iter()
            .chain(std::iter::once(&household))
            .filter_map(|&h| self.household(h))
            .map(|x| x.members.len() as u32)
            .sum();
        self.found_parts(ctx, household, &ground, going, people)
    }

    /// The households of `home` that would go with `organizer` to found on `ground`, the organizer
    /// first, and their people: in passes, each household tied to those already going (a close
    /// kin of theirs, or someone a member of theirs regards) goes if founding with them beats its
    /// own best plan. A household asked weighs staying with the fields of those already going
    /// within a field walk of home counted as vacant: their going frees them (ADR-0018 §1, §5), so
    /// a crowded village splinters until those staying would have the land they need, and does not
    /// go whole for a crowding its own going would end.
    fn recruit(
        &self,
        ctx: &Ctx,
        organizer: PermanentId,
        home: PermanentId,
        ground: &Ground,
    ) -> (Vec<PermanentId>, u32) {
        let day = ctx.now.day_index();
        let mut candidates: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| {
                x.settlement == Some(home)
                    && !x.members.is_empty()
                    && x.id != organizer
                    && !self.going_with_another(x.id, organizer)
            })
            .map(|(_, x)| x.id)
            .collect();
        candidates.sort_unstable();
        let mut going = vec![organizer];
        let mut people = self
            .household(organizer)
            .map_or(0, |x| x.members.len() as u32);
        let mut gone_people: Vec<PermanentId> = self
            .household(organizer)
            .map_or_else(Vec::new, |x| x.members.clone());
        // The hectares the households going work within a field walk of home: theirs to free.
        let hearth = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == home)
            .map(|s| s.hearth_m);
        let reach = crate::found::field_reach_m(ctx.params);
        let frees = |h: PermanentId| -> f64 {
            hearth.map_or(0.0, |at| {
                self.fields_of(ctx.land, h)
                    .filter(|f| {
                        let c = f.rect.centre_m();
                        (c.0 - at.0).hypot(c.1 - at.1) <= reach
                    })
                    .map(civ_land::Field::area_ha)
                    .sum()
            })
        };
        let mut freed = frees(organizer);
        // The factions a member of the organizing household organizes: their members leave
        // together with it (M5a slice AO; a faction's organizer's "leave together").
        let led: Vec<PermanentId> = gone_people
            .iter()
            .filter_map(|&m| self.factions.membership(m).map(|x| x.faction))
            .filter(|&f| {
                self.factions
                    .get(f)
                    .is_some_and(|x| gone_people.contains(&x.organizer))
            })
            .collect();
        // How a household is tied to those going: close kin of theirs (0), in a faction one of
        // them organizes (1), or regarded by one of them (2); none if it is not.
        let tie = |x: &crate::person::Household, gone: &[PermanentId]| -> Option<u8> {
            let kin = |m: PermanentId| self.close_kin(m).iter().any(|q| gone.contains(q));
            let faction = |m: PermanentId| {
                self.factions
                    .membership(m)
                    .is_some_and(|ms| led.contains(&ms.faction))
            };
            let regarded = |m: PermanentId| {
                gone.iter()
                    .any(|&g| self.ties.regard(g, m, day, &ctx.params.ties) > 0.0)
            };
            if x.members.iter().any(|&m| kin(m)) {
                Some(0)
            } else if x.members.iter().any(|&m| faction(m)) {
                Some(1)
            } else if x.members.iter().any(|&m| regarded(m)) {
                Some(2)
            } else {
                None
            }
        };
        // Diagnostic (claude/w2-recruit-diag2): who is asked and why each goes or stays.
        let diag = std::env::var_os("TCE_RECRUIT_DIAG").is_some();
        let mut tally = [0u32; 3];
        let (mut asked_n, mut was_only, mut stake_sum) = (0u32, 0u32, 0.0f64);
        let (mut net_sum, mut net_pos, mut ties_sum, mut griev_sum) = (0i64, 0u32, 0.0f64, 0.0f64);
        let (mut declined_kin, mut declined_net_neg) = (0u32, 0u32);
        for _ in 0..RECRUIT_PASSES {
            let mut joined = false;
            // The closest tied are asked first, so who goes is not decided by the order
            // households were made in where not all can (10-01 §5.5).
            let mut asked: Vec<(u8, PermanentId)> = candidates
                .iter()
                .filter(|c| !going.contains(c))
                .filter_map(|&c| {
                    let x = self.household(c)?;
                    tie(x, &gone_people).map(|t| (t, c))
                })
                .collect();
            asked.sort_unstable();
            for (t, c) in asked {
                let Some(x) = self.household(c) else {
                    continue;
                };
                let mut with = going.clone();
                with.push(c);
                let n = people + x.members.len() as u32;
                let staying = Ground {
                    vacant: ground.vacant + freed,
                    ..*ground
                };
                let worth = self.found_worth(ctx, c, &staying, &with, n);
                let own = self
                    .move_worth(ctx, c)
                    .into_iter()
                    .map(|(_, v)| v)
                    .fold(0.0, f64::max);
                // It goes if founding with them is worth more to it than its own best plan, and
                // only if they would still hold food and seed enough to go with it (10-01 §2.3):
                // a household that would leave them short is not asked.
                let enough = self.lacking(ctx, &with) == Lacking::Nothing;
                let parts = if diag {
                    self.found_parts(ctx, c, &staying, &with, n)
                } else {
                    None
                };
                if diag {
                    asked_n += 1;
                    let was = self.found_worth(ctx, c, ground, &with, n);
                    if was > own && worth <= own && enough {
                        was_only += 1;
                    }
                    if !(worth > own && enough)
                        && let Some(p) = parts
                    {
                        declined_kin += u32::from(p.kin > 0);
                        declined_net_neg += u32::from(p.kin > 0 && p.kin < p.kin_home);
                    }
                }
                if worth > own && enough {
                    if diag {
                        tally[usize::from(t.min(2))] += 1;
                        if let Some(p) = parts {
                            let net = p.kin as i64 - p.kin_home as i64;
                            net_sum += net;
                            net_pos += u32::from(net > 0);
                            let sat = |v: f64| v / (1.0 + v);
                            ties_sum +=
                                ctx.params.moving.w_ties * (sat(p.known) - sat(p.known_home));
                            griev_sum += p.grievance;
                        }
                        stake_sum += self
                            .outlook(ctx, c)
                            .map_or(0.0, |o| (o.harvest / o.year_need.max(1.0)).clamp(0.0, 1.0));
                    }
                    going.push(c);
                    people = n;
                    gone_people.extend_from_slice(&x.members);
                    freed += frees(c);
                    joined = true;
                }
            }
            if !joined {
                break;
            }
        }
        if diag {
            let joined = going.len() as u32 - 1;
            let need: f64 = candidates
                .iter()
                .filter_map(|&c| self.household(c))
                .map(|x| Self::crop_need_ha(ctx, x.members.len() as u32, 1.0))
                .sum();
            let mean = |v: f64| {
                if joined > 0 {
                    v / f64::from(joined)
                } else {
                    0.0
                }
            };
            eprintln!(
                "RECRUIT day {} home {} org {} households {} asked {asked_n} joined {joined} \
                 (kin {}, faction {}, regard {}) people {people} joiners: net_kin_mean {:.2} \
                 net_kin_pos {net_pos} ties_mean {:.2} grievance_mean {:.2} stake_mean {:.2} | \
                 declined with kin going {declined_kin} (fewer going than staying {declined_net_neg}) \
                 | vacant {:.1} freed {:.1} need_rest {:.1} site_ha {:.1} declined_only_for_freed {was_only}",
                ctx.now.day_index(),
                home.get(),
                organizer.get(),
                candidates.len() + 1,
                tally[0],
                tally[1],
                tally[2],
                mean(net_sum as f64),
                mean(ties_sum),
                mean(griev_sum),
                mean(stake_sum),
                ground.vacant,
                freed,
                need,
                ground.site_ha,
            );
        }
        (going, people)
    }

    /// The organizing household `household`'s plan has won this review: its coalition gathers, or
    /// gathers on; when the plan has won the reviews moving needs, it goes if it holds enough
    /// (10-01 §2.3), and otherwise waits for the next review.
    pub(super) fn gather(&mut self, ctx: &mut Ctx, household: PermanentId, plan: Plan, day: i64) {
        let Some(home) = self.household(household).and_then(|x| x.settlement) else {
            return;
        };
        let cell = cell_of(ctx.map, plan.site);
        let same = self
            .gathering_of(household)
            .filter(|&ci| cell_of(ctx.map, self.coalitions[ci].site) == cell);
        let ci = match same {
            Some(ci) => {
                let c = &mut self.coalitions[ci];
                c.reviews += 1;
                c.members = plan.members;
                c.people = plan.people;
                ci
            }
            None => {
                self.give_up(ctx, household, day);
                let id = self.coalitions.last().map_or(0, |c| c.id) + 1;
                let named = self
                    .household(household)
                    .and_then(|x| x.members.first().copied());
                self.coalitions.push(Coalition {
                    id,
                    organizer: household,
                    named,
                    from: home,
                    site: plan.site,
                    formed: day,
                    reviews: 1,
                    members: plan.members,
                    people: plan.people,
                    lacking: Lacking::Nothing,
                    fate: CoalitionFate::Gathering,
                    ended: None,
                    settlement: None,
                });
                let words = self.site_words(ctx, home, plan.site);
                self.coalition_chronicle(
                    ctx,
                    household,
                    home,
                    plan.site,
                    CoalitionStep::Began,
                    words,
                );
                self.coalitions.len() - 1
            }
        };
        if std::env::var_os("TCE_RECRUIT_DIAG").is_some() {
            let c = &self.coalitions[ci];
            eprintln!(
                "GATHER day {} home {} org {} reviews {} households {} people {}",
                ctx.now.day_index(),
                home.get(),
                household.get(),
                c.reviews,
                c.members.len(),
                c.people
            );
        }
        if self.coalitions[ci].reviews < ctx.params.moving.reviews.max(1) {
            return;
        }
        let members = self.coalitions[ci].members.clone();
        let lacking = self.lacking(ctx, &members);
        self.coalitions[ci].lacking = lacking;
        match lacking {
            Lacking::Nothing => self.found_settlement(ctx, ci),
            Lacking::Food | Lacking::Seed => {
                let step = if lacking == Lacking::Food {
                    CoalitionStep::LackedFood
                } else {
                    CoalitionStep::LackedSeed
                };
                let site = self.coalitions[ci].site;
                self.coalition_chronicle(ctx, household, home, site, step, String::new());
            }
        }
    }

    /// The coalition `household` organizes, if gathering, is given up.
    pub(super) fn give_up(&mut self, ctx: &mut Ctx, household: PermanentId, day: i64) {
        let Some(ci) = self.gathering_of(household) else {
            return;
        };
        let (from, site) = (self.coalitions[ci].from, self.coalitions[ci].site);
        let c = &mut self.coalitions[ci];
        c.fate = CoalitionFate::Dissolved;
        c.ended = Some(day);
        self.coalition_chronicle(
            ctx,
            household,
            from,
            site,
            CoalitionStep::GaveUp,
            String::new(),
        );
    }

    /// What the households `members` lack to go now (10-01 §2.3): food for everyone to their first
    /// harvest and the content's months beyond, after seed for the ground they need; or that seed.
    /// The first harvest is this year's if their adults can break that ground before sowing ends.
    fn lacking(&self, ctx: &Ctx, members: &[PermanentId]) -> Lacking {
        let (params, now) = (ctx.params, ctx.now);
        let Some(crop) = ctx.catalog.crops.get(params.farm.crop) else {
            return Lacking::Nothing;
        };
        let goods = &ctx.catalog.goods;
        let (mut people, mut adults) = (0u32, 0u32);
        let mut held = vec![0.0; goods.len()];
        for &h in members {
            let Some(x) = self.household(h) else {
                continue;
            };
            for (g, kg) in stores_now(x, now, params, goods).into_iter().enumerate() {
                held[g] += kg;
            }
            for p in x.members.iter().filter_map(|&m| self.person(m)) {
                people += 1;
                adults += u32::from(p.age_years(now) >= params.family.independent_age);
            }
        }
        let need_ha = Self::crop_need_ha(ctx, people, 1.0);
        let seed_kg = need_ha * crop.seed_kg_per_ha;
        if held.get(crop.seed_good).copied().unwrap_or(0.0) < seed_kg {
            return Lacking::Seed;
        }
        let doy = now.day_index().rem_euclid(DAYS_PER_YEAR);
        let (sow_from, sow_until) = (i64::from(crop.sow_from_day), i64::from(crop.sow_until_day));
        let break_h = need_ha * crop.break_h_per_ha;
        let work = f64::from(adults) * params.founding.work_h_per_day;
        let this_year = doy <= sow_until && work * (sow_until - doy) as f64 >= break_h;
        let to_harvest = if this_year {
            sow_from.max(doy) + i64::from(crop.grow_days) - doy
        } else {
            DAYS_PER_YEAR - doy + sow_from + i64::from(crop.grow_days)
        };
        let days = to_harvest as f64 + params.founding.buffer_months * DAYS_PER_YEAR as f64 / 12.0;
        let need_kcal = f64::from(people) * params.household.daily_kcal_per_person * days;
        let food_kcal: f64 = goods
            .iter()
            .enumerate()
            .filter(|(_, g)| g.purpose == crate::params::GoodUse::Food && g.kcal_per_kg > 0.0)
            .map(|(i, g)| {
                let kg = if i == crop.seed_good {
                    (held[i] - seed_kg).max(0.0)
                } else {
                    held[i]
                };
                kg * g.kcal_per_kg
            })
            .sum();
        if food_kcal < need_kcal {
            Lacking::Food
        } else {
            Lacking::Nothing
        }
    }

    /// Coalition `ci` goes: its settlement is written, named and given a polity under a copy of
    /// the body its founders lived under, and each of its households moves there, as a household
    /// moving does; those left at home with kin among them know where they went.
    fn found_settlement(&mut self, ctx: &mut Ctx, ci: usize) {
        let now = ctx.now;
        let day = now.day_index();
        let c = self.coalitions[ci].clone();
        let id = ctx.ids.allocate();
        let mut d = crate::found::Draws(Rng64::from_key(&[ctx.seed, PURPOSE_FOUND, id.get()]));
        let name = crate::found::place_name(ctx.params, &mut d, &ctx.land.settlements);
        ctx.land.settlements.push(civ_land::Settlement {
            id,
            name,
            founded: now,
            hearth_m: c.site,
            food_short: false,
            harvest_kg: 0.0,
            parent: Some(c.from),
            founding: civ_land::Founding::Coalition,
            abandoned: None,
        });
        let mut polity =
            crate::polity::Polity::found(ctx.ids.allocate(), id, now, &ctx.params.polity);
        if let Some(pi) = self.polity_of(c.from) {
            let body = self.polities[pi].body;
            polity.body = body;
            for v in &mut polity.versions {
                v.body = body;
            }
        }
        self.polities.push(polity);
        // Its way of building as founded (M5b slice AR): its households' taste, on average.
        let tastes: Vec<crate::params::Taste> = c
            .members
            .iter()
            .filter_map(|&h| self.household(h))
            .filter(|x| x.settlement == Some(c.from))
            .map(|x| x.taste)
            .collect();
        if let Some(way) = crate::style::mean_taste(&tastes) {
            self.founding_ways.insert(id, way);
        }
        let first = self
            .household(c.organizer)
            .and_then(|x| x.members.first().copied());
        let mut people = Vec::new();
        let mut households = 0u32;
        for &h in &c.members {
            let Some(x) = self.household(h) else {
                continue;
            };
            if x.settlement != Some(c.from) {
                continue;
            }
            people.extend_from_slice(&x.members);
            households += 1;
            self.relocate_as(ctx, h, id, ResidenceWhy::Founded);
        }
        // Those at home with kin among them know where they went.
        let mut told: Vec<PermanentId> = Vec::new();
        for &p in &people {
            for q in self.close_kin(p) {
                if let Some(h) = self.person(q).map(|x| x.household)
                    && self
                        .household(h)
                        .is_some_and(|x| x.settlement == Some(c.from))
                    && !told.contains(&h)
                {
                    told.push(h);
                }
            }
        }
        for h in told {
            self.known_places.learn(h, id, day, PlaceHow::Kin, None);
        }
        let from_name = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == c.from)
            .map_or_else(String::new, |s| s.name.clone());
        let others = match households.saturating_sub(1) {
            0 => String::new(),
            1 => " and one more household".to_owned(),
            n => format!(" and {n} more households"),
        };
        let words = format!(
            "{others} ({} people in all) left {from_name} and founded ",
            people.len()
        );
        self.chronicle_push(
            now,
            ChronicleKind::Coalition,
            first.into_iter().collect(),
            Some(id),
            Some(c.site),
            f64::from(CoalitionStep::Founded as u8),
            words,
        );
        if std::env::var_os("TCE_RECRUIT_DIAG").is_some() {
            let left = self
                .households
                .iter()
                .filter(|(_, x)| x.settlement == Some(c.from) && !x.members.is_empty())
                .count();
            eprintln!(
                "FOUNDED day {} from {} as {} households {households} people {} households_left {left}",
                now.day_index(),
                c.from.get(),
                id.get(),
                people.len()
            );
        }
        let k = &mut self.coalitions[ci];
        k.fate = CoalitionFate::Founded;
        k.ended = Some(day);
        k.settlement = Some(id);
        k.people = people.len() as u32;
        k.lacking = Lacking::Nothing;
    }

    /// " 3.1 km north of ": how far the site is from home and which way.
    fn site_words(&self, ctx: &Ctx, home: PermanentId, site: (f32, f32)) -> String {
        let Some(h) = ctx.land.settlements.iter().find(|s| s.id == home) else {
            return " near ".to_owned();
        };
        let (dx, dy) = (site.0 - h.hearth_m.0, site.1 - h.hearth_m.1);
        let km = dx.hypot(dy) / 1000.0;
        // North is up the map: y grows southwards.
        let way = match ((dy.atan2(dx).to_degrees() + 360.0 + 22.5) % 360.0 / 45.0) as u32 {
            0 => "east",
            1 => "south-east",
            2 => "south",
            3 => "south-west",
            4 => "west",
            5 => "north-west",
            6 => "north",
            _ => "north-east",
        };
        format!(" {km:.1} km {way} of ")
    }

    /// A chronicle entry for a coalition's `step`, named for the organizing `household`'s eldest.
    fn coalition_chronicle(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        settlement: PermanentId,
        site: (f32, f32),
        step: CoalitionStep,
        words: String,
    ) {
        let first = self
            .household(household)
            .and_then(|x| x.members.first().copied());
        self.chronicle_push(
            ctx.now,
            ChronicleKind::Coalition,
            first.into_iter().collect(),
            Some(settlement),
            Some(site),
            f64::from(step as u8),
            words,
        );
    }
}
