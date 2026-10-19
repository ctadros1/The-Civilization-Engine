//! Plans as laws (M8c slice CT, second part; ADR-0030 §2). What a polity sees of its
//! settlements' ground raises a plan's issues: a household that found no clear ground for the home
//! it meant to build, a building begun on a worn trail, a home built beyond the settlement's built
//! edge. A sponsor who knows how to lay ground out by cord and stake lays the template's grid at
//! the hearth along both axes and at the built edge to each side along it, at the least extent
//! whose parcels would house twice the households it remembers wanting ground, and each household
//! weighs each layout in hours: the walking a parcel would save it, over a home's years, against
//! the ground it would take now if it means to build, less its share of staking the parcels out.
//! The layout put forward is saved with the law as realised; the law builds nothing.

use super::*;
use crate::polity::{IssueKind, LawStatus, PolicyKind};
use civ_land::plans::{Axis, Grid, Layout};

/// Days between two plans one polity weighs (ADR-0030 §2: at most every 60 days; 10-04 §3.4 gives
/// 30-90).
pub(super) const PLAN_EVERY_DAYS: i64 = 60;

/// A plan a sponsor could put forward (M8c slice CT, second part): its template, its code (the
/// origin's cell twice over and its axis: [`plan_code`]), the grid laid, its layout over the ground
/// as it is, and what each household reckons it would bring it a year: the hours of walking it
/// would save, and its share of staking the parcels out, by household.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct PublicPlan {
    pub(super) policy: u16,
    pub(super) code: u32,
    pub(super) grid: Grid,
    pub(super) layout: Layout,
    pub(super) gains: PlanGains,
}

impl PublicPlan {
    /// What it would bring household `h` a year, hours: walking saved, and staking owed.
    pub(super) fn gain_of(&self, h: PermanentId) -> (f64, f64) {
        self.gains
            .binary_search_by_key(&h, |g| g.0)
            .map_or((0.0, 0.0), |i| (self.gains[i].1, self.gains[i].2))
    }
}

/// A plan's code in a move: its origin's cell twice over, and its axis.
pub(super) fn plan_code(map: &WorldMap, origin: (f32, f32), axis: Axis) -> u32 {
    (cell_of(map, origin) as u32) * 2 + u32::from(axis.code())
}

/// The parcels grid `g` lays where no ground is refused.
fn parcels_of(g: &Grid) -> usize {
    let (nl, nw) = (usize::from(g.blocks.0), usize::from(g.blocks.1));
    let per_row = if g.parcel_m.0 > 0.0 {
        (g.block_m.0 / g.parcel_m.0).floor().max(0.0) as usize
    } else {
        0
    };
    let blocks = (nl * nw).saturating_sub(usize::from(g.square));
    blocks * 2 * per_row
}

/// A grid's extent, metres: along its axis and across.
fn extent_m(g: &Grid) -> (f32, f32) {
    let (nl, nw) = (f32::from(g.blocks.0), f32::from(g.blocks.1));
    (
        nl * g.block_m.0 + (nl + 1.0) * g.street_m,
        nw * g.block_m.1 + (nw + 1.0) * g.street_m,
    )
}

/// What a plan would bring each household a year, hours: (household, walking saved, staking
/// owed), by household.
pub type PlanGains = Vec<(PermanentId, f64, f64)>;

/// What a household would weigh a plan by: who, where its people go from home (centre, trips a
/// year), and, if it means to build, the hours the plot it would take now would cost it and the
/// plot's ground (`None` inside when it found no ground it would take).
type Weighing = (
    PermanentId,
    Vec<((f32, f32), f64)>,
    Option<Option<(f64, civ_land::RectCm)>>,
);

/// A parcel a plan's keeper would give a household (M8c slice CU): the plan's law, the parcel's
/// index in its layout and its ground, the rule it is given by, and the fee the household pays
/// the store for it (the good, by index in the catalog's goods, and the amount), if one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParcelOffer {
    pub law: PermanentId,
    pub parcel: u16,
    pub ground: civ_land::RectCm,
    pub rule: civ_land::AllotRule,
    pub fee: Option<(usize, f64)>,
    /// For a parcel given for a ground rent (M8c slice CU, third part): the food and amount owed
    /// the store each year.
    pub rent: Option<(usize, f64)>,
}

impl Population {
    /// The polity holding settlement `settlement` saw `issue` of household `household`'s building
    /// (M8c slice CT, second part): kept within memory.
    pub(super) fn see_ground(
        &mut self,
        ctx: &Ctx,
        settlement: Option<PermanentId>,
        household: PermanentId,
        issue: IssueKind,
    ) {
        let Some(settlement) = settlement else {
            return;
        };
        let Some(pi) = self.polity_of(settlement) else {
            return;
        };
        let day = ctx.now.day_index();
        let remember = day - i64::from(ctx.params.crime.remember_days);
        let polity = &mut self.polities[pi];
        polity.ground_seen.retain(|g| g.day >= remember);
        polity.see_ground(crate::polity::GroundSeen {
            day,
            settlement,
            household,
            issue,
        });
    }

    /// Household `household` takes the ground `rect` for a building, a home if `dwelling` (M8c
    /// slice CT, second part): its polity sees a worn trail built over, and a home beyond the
    /// settlement's built edge (farther from the hearth than every plot its households hold).
    pub(super) fn ground_taken(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        rect: &civ_land::RectCm,
        dwelling: bool,
    ) {
        let Some(settlement) = self.household(household).and_then(|x| x.settlement) else {
            return;
        };
        let cell_cm = ((f64::from(ctx.map.cell_size_m) * 100.0).round() as i32).max(1);
        let w = ctx.map.width as i32;
        let (x0, y0) = (rect.x / cell_cm, rect.y / cell_cm);
        let (x1, y1) = (
            (rect.x + rect.w - 1) / cell_cm,
            (rect.y + rect.h - 1) / cell_cm,
        );
        let on_trail = (y0.max(0)..=y1).any(|y| {
            (x0.max(0)..=x1.min(w - 1)).any(|x| ctx.land.wear.is_trail((y * w + x) as usize))
        });
        if on_trail {
            self.see_ground(ctx, Some(settlement), household, IssueKind::WayBuiltOver);
        }
        if !dwelling {
            return;
        }
        let Some(hearth) = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map(|s| s.hearth_m)
        else {
            return;
        };
        let from_hearth = |r: &civ_land::RectCm| {
            let c = r.centre_m();
            (c.0 - hearth.0).hypot(c.1 - hearth.1)
        };
        let edge = ctx
            .land
            .plots
            .iter()
            .filter(|p| {
                self.household(p.household)
                    .is_some_and(|x| x.settlement == Some(settlement))
            })
            .map(|p| from_hearth(&p.rect))
            .fold(None, |m: Option<f32>, d| Some(m.map_or(d, |m| m.max(d))));
        if edge.is_some_and(|e| from_hearth(rect) > e) {
            self.see_ground(ctx, Some(settlement), household, IssueKind::NewGround);
        }
    }

    /// What polity `pi` saw of settlement `settlement`'s ground within memory, and since the
    /// plan in force was decided if one is (a new version answers what that plan did not): the
    /// plan's issues it raises, in code order, and the households it was of, in id order.
    pub(super) fn ground_issues(
        &self,
        ctx: &Ctx,
        pi: usize,
        settlement: PermanentId,
    ) -> (Vec<IssueKind>, Vec<PermanentId>) {
        let decided = self.polities[pi]
            .laws
            .iter()
            .filter(|l| l.kind == PolicyKind::Plan && l.status == LawStatus::InForce)
            .filter_map(|l| l.decided.map(|t| t.day_index() + 1))
            .max();
        let remember = (ctx.now.day_index() - i64::from(ctx.params.crime.remember_days))
            .max(decided.unwrap_or(i64::MIN));
        let seen = self.polities[pi]
            .ground_seen
            .iter()
            .filter(|g| g.settlement == settlement && g.day >= remember);
        let mut issues: Vec<IssueKind> = seen.clone().map(|g| g.issue).collect();
        issues.sort_unstable_by_key(|i| IssueKind::ALL.iter().position(|k| k == i));
        issues.dedup();
        let mut households: Vec<PermanentId> = seen.map(|g| g.household).collect();
        households.sort_unstable();
        households.dedup();
        (issues, households)
    }

    /// The plans a sponsor of polity `pi` could put forward under template `policy` for
    /// settlement `settlement`, whose households are `households` (in id order): its grid laid at
    /// the hearth along each axis, and at the built edge east, west, south and north along it,
    /// at the first of the template's extents whose parcels would house twice the households it
    /// remembers wanting ground and those without a home (or the largest), each over the ground
    /// as it is and weighed by each household. None for a template that is no plan or a
    /// settlement without a hearth; a layout with no parcel is left out.
    pub(super) fn public_plans(
        &self,
        ctx: &Ctx,
        pi: usize,
        settlement: PermanentId,
        households: &[PermanentId],
        policy: u16,
    ) -> Vec<PublicPlan> {
        let Some(def) = ctx
            .catalog
            .policies
            .get(usize::from(policy))
            .and_then(|d| d.plan.as_ref())
        else {
            return Vec::new();
        };
        let Some(hearth) = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map(|s| s.hearth_m)
        else {
            return Vec::new();
        };
        let catalog = ctx.catalog;
        // Who means to build: a household without a finished home standing.
        let homed = |h: PermanentId| {
            ctx.land.buildings.iter().any(|b| {
                b.household == h
                    && b.finished()
                    && b.standing()
                    && catalog.is_dwelling(&b.spec.program)
            })
        };
        let unhoused: Vec<PermanentId> =
            households.iter().copied().filter(|&h| !homed(h)).collect();
        // Sized by the households it remembers wanting ground and those without a home now.
        let (_, wanting) = self.ground_issues(ctx, pi, settlement);
        let mut want: Vec<PermanentId> = wanting
            .into_iter()
            .chain(unhoused.iter().copied())
            .collect();
        want.sort_unstable();
        want.dedup();
        let need = (2 * want.len()).max(1);
        let with_blocks = |blocks: (u8, u8)| Grid { blocks, ..def.grid };
        let Some(grid) = def
            .extents
            .iter()
            .map(|&b| with_blocks(b))
            .find(|g| parcels_of(g) >= need)
            .or_else(|| def.extents.last().map(|&b| with_blocks(b)))
        else {
            return Vec::new();
        };
        // The built edge: the farthest of its households' plots from the hearth.
        let edge_m = ctx
            .land
            .plots
            .iter()
            .filter(|p| households.binary_search(&p.household).is_ok())
            .map(|p| {
                let c = p.rect.centre_m();
                (c.0 - hearth.0).hypot(c.1 - hearth.1)
            })
            .fold(0.0f32, f32::max);
        // The ground it may lay on: clear of water, claims, works and wells, and not too steep to
        // level by hand.
        let lev = &ctx.params.build.levelling;
        let bed = |x: f64, y: f64| civ_land::earth::bed_height(ctx.map, (x, y));
        let ground = |r: &civ_land::RectCm| {
            crate::build::plot_clear(ctx.land, ctx.map, ctx.nav, r)
                && civ_land::earth::drop_across(r, &bed) <= lev.most_m
        };
        let mut origins: Vec<((f32, f32), Axis)> = Axis::ALL.iter().map(|&a| (hearth, a)).collect();
        let (_, across) = extent_m(&grid);
        for (dx, dy) in [(1.0f32, 0.0f32), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
            // Along the edge, its width outward: east and west of the hearth a grid runs north and
            // south; north and south of it, east and west.
            let axis = if dx != 0.0 {
                Axis::NorthSouth
            } else {
                Axis::EastWest
            };
            let r = edge_m + across / 2.0;
            origins.push(((hearth.0 + dx * r, hearth.1 + dy * r), axis));
        }
        // What each household would weigh it by.
        let speed = ctx.nav.params().top_speed_ms().max(0.1);
        let years = ctx.params.build.site_years.max(1e-9);
        let reach_h = ctx.params.farm.max_walk_minutes / 60.0;
        let weighing: Vec<Weighing> = households
            .iter()
            .filter_map(|&h| {
                let hh = self.household(h)?;
                let places = self.goings_of(ctx, hh);
                let now = unhoused.binary_search(&h).is_ok().then(|| {
                    self.new_building(ctx, hh, Some(hearth), settlement, 24.0, None)
                        .ok()
                        .and_then(|site| {
                            let program = catalog
                                .building_index(&site.spec.program)
                                .and_then(|i| catalog.buildings.get(i))?;
                            let at = crate::build::centre_m(&site.spec);
                            let hours = site.receipt.map_or_else(
                                || years * siting::walk_hours_year(at, &places, speed),
                                |r| f64::from(r.hours),
                            );
                            Some((hours, crate::build::plot_rect(&site.spec, program)))
                        })
                });
                Some((h, places, now))
            })
            .collect();
        let mut out: Vec<PublicPlan> = Vec::new();
        for (origin, axis) in origins {
            let code = plan_code(ctx.map, origin, axis);
            if out.iter().any(|p| p.code == code) {
                continue;
            }
            let layout = civ_land::plans::lay_grid(&grid, origin, axis, &ground);
            if layout.parcels.is_empty() {
                continue;
            }
            // Each household's equal share of staking the parcels out, spread over a home's
            // years.
            let stake = f64::from(def.stake_h) * layout.parcels.len() as f64
                / households.len().max(1) as f64
                / years;
            let gains = weighing
                .iter()
                .map(|(h, places, now)| {
                    let saved = now.as_ref().map_or(0.0, |now| {
                        // The best parcel for it, over a home's years.
                        let parcel_h = layout
                            .parcels
                            .iter()
                            .map(|p| {
                                years * siting::walk_hours_year(p.rect.centre_m(), places, speed)
                            })
                            .fold(f64::INFINITY, f64::min);
                        match now {
                            // Against where it would build now: what a parcel saves it, or, if
                            // the plan keeps that ground open, what it would cost it.
                            Some((now_h, rect)) => {
                                if layout.reserves(rect, 0) {
                                    now_h - parcel_h
                                } else {
                                    (now_h - parcel_h).max(0.0)
                                }
                            }
                            // No ground it would take: every walk as from the edge of its reach
                            // (one a day for each member, while nobody has gone anywhere yet).
                            None => {
                                let trips: f64 = places.iter().map(|p| p.1).sum();
                                let trips = if trips > 0.0 {
                                    trips
                                } else {
                                    self.household(*h).map_or(1, |x| x.members.len().max(1)) as f64
                                        * 365.0
                                };
                                (years * trips * 2.0 * reach_h - parcel_h).max(0.0)
                            }
                        }
                    });
                    (*h, saved / years, stake)
                })
                .collect();
            out.push(PublicPlan {
                policy,
                code,
                grid,
                layout,
                gains,
            });
        }
        out
    }

    /// The parcel a plan in force where household `hh` lives would give it now, to build its home
    /// on (M8c slice CU; ADR-0030 §4), by the plan's rule: by need, the free parcel nearest the
    /// hearth; by lot, a free parcel drawn by a keyed lot; for a fee, the nearest, to a household
    /// holding the fee (one for a ground rent alone asks nothing held); by the keeper's choice, the nearest, to a household whose eldest the keeper
    /// regards. None while no plan in force there has a keeper living there, or every parcel is
    /// given or no longer clear.
    pub(super) fn parcel_offer(&self, ctx: &Ctx, hh: &Household) -> Option<ParcelOffer> {
        let settlement = hh.settlement?;
        let pi = self.polity_of(settlement)?;
        let polity = &self.polities[pi];
        if polity.plans.is_empty() {
            return None;
        }
        let hearth = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)?
            .hearth_m;
        let day = ctx.now.day_index();
        for (law, laid) in polity.plans_in_force() {
            let Some(l) = polity.laws.iter().find(|l| l.id == law) else {
                continue;
            };
            let Some(keeper) = l.holder.filter(|&k| {
                self.person(k)
                    .and_then(|p| self.household(p.household))
                    .is_some_and(|x| x.settlement == Some(settlement))
            }) else {
                continue;
            };
            let Some(def) = ctx
                .catalog
                .policies
                .get(usize::from(l.policy))
                .and_then(|d| d.plan.as_ref())
            else {
                continue;
            };
            let rule = def.allot;
            let fee = def.fee.map(|(g, a)| (g, f64::from(a)));
            let rent = def.rent.map(|(g, a)| (g, f64::from(a)));
            match rule {
                civ_land::AllotRule::Offer => {
                    if fee.is_none() && rent.is_none() {
                        continue;
                    }
                    if let Some((good, amount)) = fee
                        && hh.stores.get(good).copied().unwrap_or(0.0) < amount
                    {
                        continue;
                    }
                }
                civ_land::AllotRule::Standing => {
                    let regarded = self
                        .elder_of(hh.id, ctx.now, ctx.params)
                        .is_some_and(|e| self.ties.regard(keeper, e, day, &ctx.params.ties) > 0.0);
                    if !regarded {
                        continue;
                    }
                }
                civ_land::AllotRule::Need | civ_land::AllotRule::Lot => {}
            }
            let given: Vec<u16> = ctx
                .land
                .plots
                .iter()
                .filter_map(|p| p.allotted.filter(|a| a.law == law).map(|a| a.parcel))
                .collect();
            let far = |r: &civ_land::RectCm| {
                let (x, y) = r.centre_m();
                (x - hearth.0).hypot(y - hearth.1)
            };
            let mut free: Vec<(usize, &civ_land::plans::Parcel)> = laid
                .layout
                .parcels
                .iter()
                .enumerate()
                .filter(|(i, _)| !given.contains(&(*i as u16)))
                .filter(|(_, q)| build::plot_clear(ctx.land, ctx.map, ctx.nav, &q.rect))
                .collect();
            if free.is_empty() {
                continue;
            }
            let chosen = if rule == civ_land::AllotRule::Lot {
                let key =
                    Rng64::from_key(&[ctx.seed, law.get(), hh.id.get(), day as u64]).next_u64();
                free[(key % free.len() as u64) as usize]
            } else {
                free.sort_by(|a, b| {
                    far(&a.1.rect)
                        .total_cmp(&far(&b.1.rect))
                        .then(a.0.cmp(&b.0))
                });
                free[0]
            };
            return Some(ParcelOffer {
                law,
                parcel: chosen.0 as u16,
                ground: chosen.1.rect,
                rule,
                fee: fee.filter(|_| rule == civ_land::AllotRule::Offer),
                rent: rent.filter(|_| rule == civ_land::AllotRule::Offer),
            });
        }
        None
    }

    /// Plan law `law`'s keeper has staked a parcel it gave (M8c slice CU, first part): its hours,
    /// by the template's staking, are added to the plan's.
    pub(super) fn stake_parcel(
        &mut self,
        ctx: &Ctx,
        settlement: Option<PermanentId>,
        law: PermanentId,
    ) {
        let Some(pi) = settlement.and_then(|s| self.polity_of(s)) else {
            return;
        };
        let polity = &mut self.polities[pi];
        let hours = polity
            .laws
            .iter()
            .find(|l| l.id == law)
            .and_then(|l| ctx.catalog.policies.get(usize::from(l.policy)))
            .and_then(|d| d.plan.as_ref())
            .map_or(0.0, |p| p.stake_h);
        if let Some(p) = polity.plans.iter_mut().find(|p| p.0 == law) {
            p.1.staked_h += hours;
        }
    }

    /// Whether polity `pi` has a plan before its gathering, or put one forward within
    /// [`PLAN_EVERY_DAYS`] of day `day`.
    pub(super) fn plan_lately(&self, pi: usize, day: i64) -> bool {
        self.polities[pi].laws.iter().any(|l| {
            l.kind == PolicyKind::Plan
                && (l.status == LawStatus::Proposed
                    || day - l.proposed.day_index() < PLAN_EVERY_DAYS)
        })
    }

    /// The plans the first plan template would lay for settlement `settlement` today, each with
    /// what it brings each household (hours a year saved, owed): for tests.
    #[doc(hidden)]
    pub fn plans_for_tests(&self, ctx: &Ctx, settlement: PermanentId) -> Vec<(Layout, PlanGains)> {
        let Some(pi) = self.polity_of(settlement) else {
            return Vec::new();
        };
        let Some(policy) = ctx
            .catalog
            .policies
            .iter()
            .position(|d| d.kind == PolicyKind::Plan)
        else {
            return Vec::new();
        };
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        self.public_plans(ctx, pi, settlement, &households, policy as u16)
            .into_iter()
            .map(|p| (p.layout, p.gains))
            .collect()
    }

    /// Ground rent falls due (M8c slice CU, third part; ADR-0030 §4): each year to the day after
    /// a household was given a parcel for an offer under a plan whose template asks a ground rent, it owes
    /// the polity's store a year's rent, answered as a fine is. Owed while the household holds
    /// the plot and the polity whose law gave it stands.
    pub(super) fn levy_ground_rent(&mut self, ctx: &Ctx) {
        let day = ctx.now.day_index();
        let year = civ_core::time::DAYS_PER_YEAR;
        let due_days = i64::from(ctx.params.crime.due_days);
        for plot in &ctx.land.plots {
            let Some(given) = plot
                .allotted
                .filter(|a| a.rule == civ_land::AllotRule::Offer)
            else {
                continue;
            };
            let since = plot.since.day_index();
            if day <= since || (day - since) % year != 0 {
                continue;
            }
            let Some((polity, law)) = self.polities.iter().find_map(|p| {
                p.laws
                    .iter()
                    .find(|l| l.id == given.law)
                    .map(|l| (p.id, l.policy))
            }) else {
                continue;
            };
            let Some((good, amount)) = ctx
                .catalog
                .policies
                .get(usize::from(law))
                .and_then(|d| d.plan.as_ref())
                .and_then(|d| d.rent)
            else {
                continue;
            };
            if self.household(plot.household).is_none() {
                continue;
            }
            let kcal =
                f64::from(amount) * ctx.catalog.goods.get(good).map_or(0.0, |g| g.kcal_per_kg);
            if kcal <= 0.0 {
                continue;
            }
            let id = self.order.obligations.last().map_or(1, |o| o.id + 1);
            self.order.obligations.push(crate::crime::Obligation {
                id,
                incident: 0,
                kind: crate::crime::Owed::GroundRent,
                case: None,
                debtor: plot.household,
                beneficiary: polity,
                kcal: kcal as f32,
                paid_kcal: 0.0,
                made: day,
                due: day + due_days,
                standing: crate::crime::Standing::Open,
                answer: None,
            });
        }
    }
}

impl Population {
    /// `sponsor` puts forward the `i`th plan the first plan template would lay for settlement
    /// `settlement` today, answering `issue`: the law's id, if there is such a plan; for tests.
    #[doc(hidden)]
    pub fn propose_plan_for_tests(
        &mut self,
        ctx: &mut Ctx,
        settlement: PermanentId,
        sponsor: PermanentId,
        i: usize,
        issue: IssueKind,
    ) -> Option<PermanentId> {
        let pi = self.polity_of(settlement)?;
        let policy = ctx
            .catalog
            .policies
            .iter()
            .position(|d| d.kind == PolicyKind::Plan)? as u16;
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        let code = self
            .public_plans(ctx, pi, settlement, &households, policy)
            .get(i)?
            .code;
        let m = crate::polity::MoveOption {
            policy,
            levy_share: 0.0,
            issue,
            nominee: Some(sponsor),
            sanction: Default::default(),
            hours: (0, 0),
            body: None,
            ends: None,
            site: Some(code),
            target: None,
            own_gain: 0.0,
            followers_gain: 0.0,
            support: 0.5,
        };
        self.propose(ctx, pi, sponsor, m, Vec::new(), None);
        self.polities[pi].laws.last().map(|l| l.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grid_s_parcels_and_extent_are_counted_as_it_would_lay_them() {
        let g = Grid {
            block_m: (80.0, 40.0),
            parcel_m: (10.0, 20.0),
            street_m: 6.0,
            blocks: (3, 3),
            square: true,
        };
        let laid = civ_land::plans::lay_grid(&g, (500.0, 500.0), Axis::EastWest, &|_| true);
        assert_eq!(parcels_of(&g), laid.parcels.len());
        assert_eq!(extent_m(&g), (264.0, 144.0));
        assert_eq!(
            parcels_of(&Grid {
                blocks: (1, 1),
                ..g
            }),
            0,
            "a square alone"
        );
    }
}
