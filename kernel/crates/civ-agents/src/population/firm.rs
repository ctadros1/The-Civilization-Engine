//! Household workshops (slice J, ADR-0006 §5): a household that sets to work making a good to sell
//! sets up a workshop for it, an account with stores of its own. Its owners put in what the work
//! uses and draw out what it was paid; it posts terms for its stock on their review day, sells
//! through the ledger, keeps books, passes to the household its owners join, and closes when
//! nobody is left to keep it or it has sold nothing for months.

use civ_core::{Handle, PermanentId, SimTime};
use civ_land::PlotUse;

use civ_world::nav::TravelField;

use super::{Ctx, Population, cell_of, fuel_per_day, stores_now};
use crate::decide::{JobOption, TradeWorth};
use crate::firm::{BookKind, Entry, Exit, Firm, WageOffer};
use crate::history::ChronicleKind;
use crate::ledger::{Channel, Leg};
use crate::make;
use crate::params::{ActivityDef, Behavior, GoodUse, interpolate};
use crate::person::{Flow, Household, Person, Step, Target, stock_kcal};

impl Population {
    /// A firm, open or closed.
    pub fn firm(&self, id: PermanentId) -> Option<&Firm> {
        self.firms.iter().find(|f| f.id == id)
    }

    fn firm_mut(&mut self, id: PermanentId) -> Option<&mut Firm> {
        self.firms.iter_mut().find(|f| f.id == id)
    }

    /// The open workshop of household `owner` that makes `good`.
    pub fn workshop_of(&self, owner: PermanentId, good: usize) -> Option<PermanentId> {
        self.firms
            .iter()
            .find(|f| {
                f.is_open() && f.owner == owner && f.lines.iter().any(|&g| usize::from(g) == good)
            })
            .map(|f| f.id)
    }

    /// Household `owner` sets up a workshop to make `good`, founded by `founder`, who is setting
    /// to work making it to sell. Its existing one if it has one.
    pub(super) fn found_firm(
        &mut self,
        ctx: &mut Ctx,
        owner: PermanentId,
        founder: PermanentId,
        good: usize,
    ) -> Option<PermanentId> {
        if let Some(id) = self.workshop_of(owner, good) {
            return Some(id);
        }
        let (settlement, home) = self.household(owner).map(|x| (x.settlement, x.home))?;
        let goods = &ctx.catalog.goods;
        let what = goods.get(good)?.name.to_lowercase();
        let id = ctx.ids.allocate();
        self.firms.push(Firm::new(
            id,
            owner,
            founder,
            settlement,
            good as u16,
            goods.len(),
            ctx.now,
        ));
        self.chronicle_push_firm(
            ctx.now,
            ChronicleKind::WorkshopOpened,
            vec![founder],
            settlement,
            Some(home),
            0.0,
            what,
            id,
        );
        // A workshop building of its household whose firm closed, or that has none, goes to the
        // new one if no older firm of the household lacks one (ADR-0009 §7).
        self.hand_on_workshops(ctx.land, ctx.catalog);
        Some(id)
    }

    /// Every workshop building no open firm works in ([`Population::free_workshop`]) goes to an
    /// open firm of its household that has no building of its own, the oldest first (ADR-0009 §7:
    /// a closed firm's building stays, and the household's next firm works in it). Done when a
    /// firm is founded and each day, so a firm that closes passes its building on by the next.
    pub fn hand_on_workshops(&self, land: &mut civ_land::Land, catalog: &crate::params::Catalog) {
        for i in 0..land.buildings.len() {
            if !self.free_workshop(catalog, &land.buildings[i]) {
                continue;
            }
            let owner = land.buildings[i].household;
            let taker = self
                .firms
                .iter()
                .filter(|f| {
                    f.is_open()
                        && f.owner == owner
                        && !land
                            .buildings
                            .iter()
                            .any(|b| b.firm == Some(f.id) && b.standing())
                })
                .map(|f| f.id)
                .min();
            if taker.is_some() {
                land.buildings[i].firm = taker;
            }
        }
    }

    /// Whether building `b` is a standing workshop no open firm works in: one whose firm closed,
    /// or that never had one.
    pub fn free_workshop(&self, catalog: &crate::params::Catalog, b: &civ_land::Building) -> bool {
        catalog.use_of(&b.spec.program) == Some(PlotUse::Work)
            && b.standing()
            && b.firm
                .is_none_or(|f| self.firm(f).is_none_or(|f| !f.is_open()))
    }

    /// Where firm `id` works: the middle of its workshop building, once that has its roof on;
    /// `None` while it works in its owners' home.
    pub fn firm_site(
        &self,
        land: &civ_land::Land,
        catalog: &crate::params::Catalog,
        id: PermanentId,
    ) -> Option<(f32, f32)> {
        self.firm(id)?;
        land.buildings
            .iter()
            .find(|b| {
                b.firm == Some(id)
                    && b.roofed()
                    && catalog.use_of(&b.spec.program) == Some(PlotUse::Work)
            })
            .map(|b| crate::build::centre_m(&b.spec))
    }

    /// How many are working for firm `id` now, owners and hired hands: those whose work is for it
    /// and under way.
    fn working_for(&self, ctx: &Ctx, id: PermanentId) -> usize {
        self.people
            .iter()
            .filter(|(_, q)| {
                q.act.target == Target::Firm(id)
                    && matches!(
                        q.act.steps.get(usize::from(q.act.step)),
                        Some(Step::Work { .. })
                    )
                    && ctx
                        .catalog
                        .activities
                        .get(usize::from(q.act.def))
                        .is_some_and(|a| matches!(a.behavior, Behavior::Make | Behavior::Hire))
            })
            .count()
    }

    /// Brings a firm's stores up to now, under its owners' roof when they have one.
    fn settle_firm(&mut self, ctx: &Ctx, id: PermanentId) {
        let sheltered = self
            .firm(id)
            .and_then(|f| self.household(f.owner))
            .is_some_and(|x| x.sheltered);
        if let Some(f) = self.firm_mut(id) {
            f.settle_stores(ctx.now, &ctx.catalog.goods, sheltered);
        }
    }

    /// What a household's goods are worth to it, hours of its own work a unit (0 where it has no
    /// way to get a good: such a good is booked at no cost).
    fn worth_to(&self, ctx: &Ctx, household: PermanentId) -> Vec<f64> {
        match self.household(household).cloned() {
            Some(hh) => self
                .own_costs_of(ctx, &hh)
                .into_iter()
                .map(|c| c.unwrap_or(0.0))
                .collect(),
            None => vec![0.0; ctx.catalog.goods.len()],
        }
    }

    /// `minutes` of person `h`'s work at a session of make activity `def` for workshop `id`:
    /// its owners put in what the work uses (the `owner` channel), what is made is the
    /// workshop's, the owners' tools are worn, and the person's skill is practised. The books
    /// record it all, the inputs at what they cost the owners.
    pub(super) fn firm_make(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        def: &ActivityDef,
        minutes: u32,
        id: PermanentId,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let Some(recipe) = def.recipe.and_then(|r| ctx.catalog.recipes.get(r)) else {
            return;
        };
        let Some(p) = self.people.get(h) else {
            return;
        };
        let Some(owner) = self.firm(id).filter(|f| f.is_open()).map(|f| f.owner) else {
            return;
        };
        let hired = p.household != owner;
        // Who works for it at once, this person among them (still at the work).
        let at_once = self.working_for(ctx, id);
        let skill = recipe
            .skill
            .and_then(|k| ctx.catalog.skills.get(k).map(|s| (k, s)));
        let level = skill.map_or(0.0, |(k, _)| p.skill(k));
        let speed = skill.map_or(1.0, |(_, s)| interpolate(&s.speed, level))
            * interpolate(&params.capacity_by_age, p.age_years(now))
            * def.rate;
        let quality = skill.map_or(1.0, |(_, s)| interpolate(&s.quality, level));
        let hours = f64::from(minutes) / 60.0;
        let units = make::units_in(recipe, f64::from(minutes), speed);
        // What the recipe takes from the owners' stores for what can be made in the time (never
        // the seed: a workshop's work is not hunger's), and what it gives.
        let goods = &ctx.catalog.goods;
        let Some(hd) = self.hh_index.get(&owner).copied() else {
            return;
        };
        let Some(x) = self.households.get_mut(hd) else {
            return;
        };
        let members = x.members.len().max(1);
        x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        x.stores.resize(goods.len(), 0.0);
        let before = x.stores.clone();
        let mut after = before.clone();
        make::apply(recipe, &mut after, goods, units, quality, false, None);
        let taken: Vec<(usize, f64)> = before
            .iter()
            .zip(&after)
            .enumerate()
            .filter(|(_, (b, a))| *b > *a)
            .map(|(g, (b, a))| (g, b - a))
            .collect();
        let made: Vec<(usize, f64)> = before
            .iter()
            .zip(&after)
            .enumerate()
            .filter(|(_, (b, a))| *a > *b)
            .map(|(g, (b, a))| (g, a - b))
            .collect();
        let legs: Vec<Leg> = taken
            .iter()
            .map(|&(good, amount)| Leg {
                from: owner,
                to: id,
                good,
                amount,
            })
            .collect();
        let put_in = legs.is_empty() || self.transfer(now, params, goods, &legs, Channel::Owner);
        let worth = self.worth_to(ctx, owner);
        let keep = params.firm.book_entries;
        if let Some(f) = self.firm_mut(id) {
            if put_in && !made.is_empty() {
                for &(g, amount) in &taken {
                    f.stores[g] -= amount;
                    f.flows.add(Flow::Used, g, amount);
                    let entry = |kind| Entry {
                        at: now,
                        kind,
                        good: g as u16,
                        amount: amount as f32,
                        other: Some(owner),
                    };
                    f.books.record(entry(BookKind::PutIn), 0.0, keep);
                    f.books
                        .record(entry(BookKind::Used), amount * worth[g], keep);
                }
                for &(g, amount) in &made {
                    f.stores[g] += amount;
                    f.flows.add(Flow::Made, g, amount);
                    f.books.record(
                        Entry {
                            at: now,
                            kind: BookKind::Made,
                            good: g as u16,
                            amount: amount as f32,
                            other: None,
                        },
                        0.0,
                        keep,
                    );
                }
            }
            f.books.worked(now, hours, hired);
            f.saw_at_once(u8::try_from(at_once).unwrap_or(u8::MAX), now.day_index());
        }
        // The work is done with the owners' tools.
        self.wear_tools(ctx, owner, &recipe.tools, hours);
        if let (Some((k, s)), Some(p)) = (skill, self.people.get_mut(h)) {
            let next = s.practised(p.skill(k), hours);
            p.set_skill(k, next);
        }
    }

    /// Workshop `id` on its owners' review day: it plans the work it would hire and its wage;
    /// its owners draw out what it was paid, beyond what it keeps to pay wages, and any of its
    /// goods they now need themselves, and put in what the wages will need; it posts terms for
    /// its stock as they would for theirs; its stock is valued for the month; and once it has
    /// sold nothing for `idle_close_days` it is given up.
    pub(super) fn review_firm(&mut self, ctx: &Ctx, id: PermanentId) {
        let (now, params) = (ctx.now, ctx.params);
        let Some(f) = self.firm(id).filter(|f| f.is_open()) else {
            return;
        };
        let (owner, settlement) = (f.owner, f.settlement);
        let since = f.last_sale.unwrap_or(f.founded);
        let Some(hh) = self.household(owner).cloned() else {
            self.close_firm(ctx, id, Exit::OwnerGone);
            return;
        };
        if (now.minutes() - since.minutes()) as f64 / 1440.0 > params.firm.idle_close_days {
            self.close_firm(ctx, id, Exit::Idle);
            return;
        }
        self.settle_firm(ctx, id);
        let goods = &ctx.catalog.goods;
        let owner_stores = stores_now(&hh, now, params, goods);
        let holding = self.holding_of(ctx, &hh, &owner_stores);
        let costs = self.own_costs_of(ctx, &hh);
        let Some(f) = self.firm(id) else {
            return;
        };
        let wage = self.plan_wage(ctx, f, &costs, &holding, &owner_stores);
        // What the wages until the next review need of the good they are paid in.
        let reserve = wage.map_or((usize::MAX, 0.0), |w| {
            (
                usize::from(w.pay),
                f64::from(w.per_hour) * f64::from(w.hours),
            )
        });
        let mut draws = Vec::new();
        let mut put_in = Vec::new();
        for (g, &kg) in f.stores.iter().enumerate() {
            let take = if f.lines.iter().any(|&l| usize::from(l) == g) {
                // What its owners lack of it themselves.
                (holding.keep.get(g).copied().unwrap_or(0.0)
                    - owner_stores.get(g).copied().unwrap_or(0.0))
                .clamp(0.0, kg.max(0.0))
            } else if g == reserve.0 {
                (kg - reserve.1).max(0.0)
            } else {
                kg.max(0.0)
            };
            if take > 1e-9 {
                draws.push(Leg {
                    from: id,
                    to: owner,
                    good: g,
                    amount: take,
                });
            }
        }
        if reserve.1 > 0.0 {
            let short = reserve.1 - f.stores.get(reserve.0).copied().unwrap_or(0.0);
            if short > 1e-9 {
                put_in.push(Leg {
                    from: owner,
                    to: id,
                    good: reserve.0,
                    amount: short,
                });
            }
        }
        let keep = params.firm.book_entries;
        for (legs, kind) in [(draws, BookKind::Drawn), (put_in, BookKind::PutIn)] {
            if !legs.is_empty()
                && self.transfer(now, params, goods, &legs, Channel::Owner)
                && let Some(f) = self.firm_mut(id)
            {
                for l in &legs {
                    f.books.record(
                        Entry {
                            at: now,
                            kind,
                            good: l.good as u16,
                            amount: l.amount as f32,
                            other: Some(owner),
                        },
                        0.0,
                        keep,
                    );
                }
            }
        }
        // Terms for its stock, at its owners' costs, in what they want.
        let Some(f) = self.firm(id) else {
            return;
        };
        let stock: Vec<(usize, f64)> = f
            .lines
            .iter()
            .map(|&g| usize::from(g))
            .filter_map(|g| {
                let held = f.stores.get(g).copied().unwrap_or(0.0);
                let least = if goods.get(g).is_some_and(|d| d.tool.is_some()) {
                    super::market::MIN_OFFER_TOOL
                } else {
                    super::market::MIN_OFFER_KG
                };
                (held >= least).then_some((g, held))
            })
            .collect();
        let old = f.offers.clone();
        let rows = match settlement {
            Some(s) => self.post_terms(ctx, s, &costs, &costs, &holding, &stock, &old),
            None => Vec::new(),
        };
        let worth: f64 = stock
            .iter()
            .map(|&(g, units)| units * costs.get(g).copied().flatten().unwrap_or(0.0))
            .sum();
        // Its wage stays on record when it hires no one, so work under way is paid at it.
        let wage = wage.or_else(|| {
            self.firm(id).and_then(|f| f.wage).map(|w| WageOffer {
                hours: 0.0,
                taken: 0.0,
                ..w
            })
        });
        if let Some(f) = self.firm_mut(id) {
            f.offers = rows;
            f.wage = wage;
            f.books.month_mut(now).stock_h = worth as f32;
        }
    }

    /// The work workshop `f` would hire until its next review, and its wage (ADR-0006 §5;
    /// research 08-10 §5.3): what buyers want of its line beyond its stock (wants nobody met,
    /// and a review's share of what sold lately), in hours of its recipe, no more than
    /// `max_hire_hours`. The wage starts at `wage_share` of what an hour's work adds at middling
    /// skill (its terms less the inputs, in hours of its owners' work) and rises by
    /// `wage_max_change` at each wage review while hours go untaken (08-10 §5.6), never above
    /// all of it: hired work always pays the workshop.
    /// It is paid in the settlement's money, or else in what neighbours most accept of what it or
    /// its owners can spare (never a tool), for as many hours as that covers.
    fn plan_wage(
        &self,
        ctx: &Ctx,
        f: &Firm,
        costs: &[Option<f64>],
        holding: &super::market::Holding,
        owner_stores: &[f64],
    ) -> Option<WageOffer> {
        let (params, catalog) = (ctx.params, ctx.catalog);
        let fp = &params.firm;
        let line = usize::from(*f.lines.first()?);
        let (activity, recipe) = catalog.activities.iter().enumerate().find_map(|(i, a)| {
            let r = catalog.recipes.get(a.recipe?)?;
            (a.behavior == Behavior::Make && r.outputs.first().is_some_and(|&(g, _)| g == line))
                .then_some((i, r))
        })?;
        let market = self.market(f.settlement?)?;
        let review = f64::from(params.market.review_days.max(1));
        let share = 1.0 - 0.5f64.powf(review / params.market.memory_days.max(1e-6));
        let wanted = market.unmet.get(line).copied().unwrap_or(0.0)
            + market.sold.get(line).copied().unwrap_or(0.0) * share
            - f.stores.get(line).copied().unwrap_or(0.0);
        if wanted <= 0.0 || recipe.unit_h <= 0.0 {
            return None;
        }
        let ask = costs.get(line).copied().flatten()? * (1.0 + params.market.margin);
        let inputs: f64 = recipe
            .inputs
            .iter()
            .map(|&(g, a)| a * costs.get(g).copied().flatten().unwrap_or(0.0))
            .sum();
        let adds = (ask - inputs) / recipe.unit_h;
        if adds <= 0.0 {
            return None;
        }
        let day = ctx.now.day_index();
        let (mut hour_h, mut reviewed) = (adds * fp.wage_share, day);
        if let Some(w) = f.wage {
            hour_h = f64::from(w.hour_h);
            reviewed = w.reviewed;
            if day - w.reviewed >= i64::from(fp.wage_review_days) {
                if w.open_hours() > 0.5 {
                    hour_h *= 1.0 + fp.wage_max_change;
                }
                reviewed = day;
            }
        }
        let hour_h = hour_h.min(adds);
        let money = self.money_of(f.settlement, &params.market);
        let accepted = market.acceptance();
        let spare = |g: usize| {
            f.stores.get(g).copied().unwrap_or(0.0).max(0.0)
                + (owner_stores.get(g).copied().unwrap_or(0.0)
                    - holding.keep.get(g).copied().unwrap_or(0.0))
                .max(0.0)
        };
        let (pay, cost, have) = catalog
            .goods
            .iter()
            .enumerate()
            .filter(|(g, d)| d.tool.is_none() && !d.kept_back() && *g != line)
            .filter_map(|(g, _)| {
                let cost = costs.get(g).copied().flatten().filter(|&c| c > 0.0)?;
                let have = spare(g);
                (have * cost >= hour_h).then_some((g, cost, have))
            })
            .max_by(|a, b| {
                (money == Some(a.0))
                    .cmp(&(money == Some(b.0)))
                    .then(
                        accepted
                            .get(a.0)
                            .copied()
                            .unwrap_or(0.0)
                            .total_cmp(&accepted.get(b.0).copied().unwrap_or(0.0)),
                    )
                    .then((a.2 * a.1).total_cmp(&(b.2 * b.1)))
                    .then(b.0.cmp(&a.0))
            })?;
        let per_hour = hour_h / cost;
        let hours = (wanted * recipe.unit_h)
            .min(fp.max_hire_hours)
            .min(have / per_hour);
        (hours >= 1.0).then_some(WageOffer {
            activity: activity as u16,
            pay: pay as u16,
            per_hour: per_hour as f32,
            hour_h: hour_h as f32,
            hours: hours as f32,
            taken: 0.0,
            reviewed,
        })
    }

    /// The paid work a person of household `hh` could go and do: of the workshops of its
    /// settlement hiring (not its own), the one whose pay is worth most to the household, the
    /// work wanted, the walk (`reach`, from home) and what the pay brings it (food it is short
    /// of, or goods at what they are worth to it). `session_min` is the longest session.
    pub(crate) fn best_job(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        reach: Option<&TravelField>,
        session_min: f64,
    ) -> Option<JobOption> {
        let settlement = hh.settlement?;
        let reach = reach?;
        let hiring: Vec<(&Firm, WageOffer)> = self
            .firms
            .iter()
            .filter(|f| f.is_open() && f.owner != hh.id && f.settlement == Some(settlement))
            .filter_map(|f| f.wage.filter(|w| w.open_hours() >= 0.5).map(|w| (f, w)))
            .collect();
        if hiring.is_empty() {
            return None;
        }
        let (catalog, goods) = (ctx.catalog, &ctx.catalog.goods);
        let costs = self.own_costs_of(ctx, hh);
        let holding = self.holding_of(ctx, hh, stores);
        let food = stock_kcal(stores, goods);
        let food_keep: f64 = goods
            .iter()
            .enumerate()
            .filter(|(_, d)| d.purpose == GoodUse::Food && !d.kept_back())
            .map(|(g, d)| holding.keep[g].max(0.0) * d.kcal_per_kg)
            .sum::<f64>()
            .max(food);
        let short = food_keep > food;
        let mut best: Option<(f64, JobOption)> = None;
        for (f, w) in hiring {
            let Some(owner) = self.household(f.owner) else {
                continue;
            };
            // The work is done with the workshop's owners' tools.
            let Some(recipe) = catalog
                .activities
                .get(usize::from(w.activity))
                .and_then(|a| a.recipe)
                .and_then(|r| catalog.recipes.get(r))
            else {
                continue;
            };
            if recipe.tools.iter().any(|&t| {
                let held = owner.stores.get(t).copied().unwrap_or(0.0);
                make::in_use(&catalog.goods, t, held) < crate::decide::MIN_TOOL
            }) {
                continue;
            }
            // At its workshop once it has one, else at its owners' home.
            let at = self
                .firm_site(ctx.land, ctx.catalog, f.id)
                .unwrap_or(owner.home);
            let Some(walk) = reach
                .seconds_to(cell_of(ctx.map, at))
                .map(|s| f64::from(s) / 60.0)
            else {
                continue;
            };
            let minutes = (w.open_hours() * 60.0).min(session_min);
            let units = f64::from(w.per_hour) * minutes / 60.0;
            let p = usize::from(w.pay);
            let Some(d) = goods.get(p) else {
                continue;
            };
            let want = holding.want.get(p).copied().unwrap_or(0.0);
            let hours = units * costs.get(p).copied().flatten().unwrap_or(0.0) * want;
            let worth = if d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 && short {
                TradeWorth::Food {
                    kcal: units * d.kcal_per_kg,
                }
            } else {
                TradeWorth::Goods { hours }
            };
            let better = best
                .as_ref()
                .is_none_or(|(h, b)| hours > *h + 1e-9 || (hours >= *h - 1e-9 && f.id < b.firm));
            if better {
                best = Some((
                    hours,
                    JobOption {
                        firm: f.id,
                        walk_min: walk,
                        at,
                        minutes,
                        worth,
                    },
                ));
            }
        }
        best.map(|(_, j)| j)
    }

    /// Someone takes `minutes` of the work workshop `id` is hiring for.
    pub(super) fn take_work(&mut self, id: PermanentId, minutes: f64) {
        if let Some(w) = self.firm_mut(id).and_then(|f| f.wage.as_mut()) {
            w.taken += (minutes / 60.0) as f32;
        }
    }

    /// Person `h` has worked `minutes` for workshop `id`: the work is done as its owners' would
    /// be (see [`Population::firm_make`]), and it pays the person's household its wage for the
    /// time through the ledger (the `wage` channel). What it lacks of the pay its owners put in,
    /// for they answer for what it owes; if they cannot, it pays what it can, and fails.
    pub(super) fn hired_work(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        id: PermanentId,
        minutes: u32,
    ) {
        let Some(w) = self.firm(id).filter(|f| f.is_open()).and_then(|f| f.wage) else {
            return;
        };
        let Some(def) = ctx.catalog.activities.get(usize::from(w.activity)).cloned() else {
            return;
        };
        let Some((who, worker)) = self.people.get(h).map(|p| (p.id, p.household)) else {
            return;
        };
        self.firm_make(ctx, h, &def, minutes, id);
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(owner) = self.firm(id).filter(|f| f.is_open()).map(|f| f.owner) else {
            return;
        };
        let pay = usize::from(w.pay);
        let owed = f64::from(w.per_hour) * f64::from(minutes) / 60.0;
        self.settle_firm(ctx, id);
        let held = |pop: &Population| {
            pop.firm(id)
                .and_then(|f| f.stores.get(pay))
                .copied()
                .unwrap_or(0.0)
                .max(0.0)
        };
        let keep = params.firm.book_entries;
        let short = owed - held(self);
        if short > 1e-9 {
            let can = self
                .household(owner)
                .map(|x| stores_now(x, now, params, goods))
                .and_then(|s| s.get(pay).copied())
                .unwrap_or(0.0)
                .max(0.0)
                .min(short);
            let leg = [Leg {
                from: owner,
                to: id,
                good: pay,
                amount: can,
            }];
            if can > 1e-9
                && self.transfer(now, params, goods, &leg, Channel::Owner)
                && let Some(f) = self.firm_mut(id)
            {
                f.books.record(
                    Entry {
                        at: now,
                        kind: BookKind::PutIn,
                        good: pay as u16,
                        amount: can as f32,
                        other: Some(owner),
                    },
                    0.0,
                    keep,
                );
            }
        }
        let paid = owed.min(held(self));
        let leg = [Leg {
            from: id,
            to: worker,
            good: pay,
            amount: paid,
        }];
        if paid > 1e-9 && self.transfer(now, params, goods, &leg, Channel::Wage) {
            // The worker saw the wages paid; the owners saw the work done (ADR-0014 §2).
            if owner != worker {
                let (wages, work) = (crate::ties::Act::WagesPaid, crate::ties::Act::WorkSeen);
                self.note_between(ctx, Some(who), worker, owner, wages, work, 0.0);
            }
            let worth = self.worth_to(ctx, owner).get(pay).copied().unwrap_or(0.0) * paid;
            if let Some(f) = self.firm_mut(id) {
                f.books.record(
                    Entry {
                        at: now,
                        kind: BookKind::Wages,
                        good: pay as u16,
                        amount: paid as f32,
                        other: Some(worker),
                    },
                    worth,
                    keep,
                );
            }
        }
        if paid + 1e-6 < owed {
            self.close_firm(ctx, id, Exit::Failure);
        }
    }

    /// Closes workshop `id` for `why`: what it holds goes to its owners, or, with nobody left to
    /// keep it, is left behind.
    pub(super) fn close_firm(&mut self, ctx: &Ctx, id: PermanentId, why: Exit) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(f) = self.firm(id).filter(|f| f.is_open()) else {
            return;
        };
        let (owner, founder, settlement, line) = (f.owner, f.founder, f.settlement, f.lines[0]);
        self.settle_firm(ctx, id);
        let keep = params.firm.book_entries;
        if self.household(owner).is_some() {
            let legs: Vec<Leg> = self
                .firm(id)
                .map(|f| {
                    f.stores
                        .iter()
                        .enumerate()
                        .filter(|&(_, &kg)| kg > 1e-12)
                        .map(|(g, &kg)| Leg {
                            from: id,
                            to: owner,
                            good: g,
                            amount: kg,
                        })
                        .collect()
                })
                .unwrap_or_default();
            if !legs.is_empty()
                && self.transfer(now, params, goods, &legs, Channel::Owner)
                && let Some(f) = self.firm_mut(id)
            {
                for l in &legs {
                    f.books.record(
                        Entry {
                            at: now,
                            kind: BookKind::Drawn,
                            good: l.good as u16,
                            amount: l.amount as f32,
                            other: Some(owner),
                        },
                        0.0,
                        keep,
                    );
                }
            }
        }
        let place = self.household(owner).map(|x| x.home);
        if let Some(f) = self.firm_mut(id) {
            // What nobody is left to keep is left behind.
            for (g, kg) in f.stores.iter_mut().enumerate() {
                if *kg > 0.0 {
                    f.flows.add(Flow::Departed, g, *kg);
                    f.books.record(
                        Entry {
                            at: now,
                            kind: BookKind::Lost,
                            good: g as u16,
                            amount: *kg as f32,
                            other: None,
                        },
                        0.0,
                        keep,
                    );
                }
                *kg = 0.0;
            }
            f.offers.clear();
            f.closed = Some((now, why));
        }
        let what = goods
            .get(usize::from(line))
            .map_or_else(String::new, |d| d.name.to_lowercase());
        self.chronicle_push_firm(
            now,
            ChronicleKind::WorkshopClosed,
            vec![founder],
            settlement,
            place,
            f64::from(why as u8),
            what,
            id,
        );
    }

    /// The workshops of household `from` pass to `to`, the household its people joined, or,
    /// with no one to take them, close.
    pub(super) fn pass_firms(&mut self, ctx: &Ctx, from: PermanentId, to: Option<PermanentId>) {
        let ids: Vec<PermanentId> = self
            .firms
            .iter()
            .filter(|f| f.is_open() && f.owner == from)
            .map(|f| f.id)
            .collect();
        for id in ids {
            match to {
                Some(to) => {
                    let now = ctx.now;
                    if let Some(f) = self.firm_mut(id) {
                        f.owner = to;
                        f.owner_since = now;
                    }
                }
                None => self.close_firm(ctx, id, Exit::OwnerGone),
            }
        }
    }

    /// What household `hh` holds of `good` in its open workshops.
    pub(crate) fn workshop_stock(&self, hh: &Household, good: usize) -> f64 {
        self.firms
            .iter()
            .filter(|f| f.is_open() && f.owner == hh.id)
            .map(|f| f.stores.get(good).copied().unwrap_or(0.0).max(0.0))
            .sum()
    }

    /// Firm `id` sold something at `now`.
    pub(super) fn note_sale(&mut self, id: PermanentId, now: SimTime) {
        if let Some(f) = self.firm_mut(id) {
            f.last_sale = Some(now);
        }
    }
}
