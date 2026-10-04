//! Household workshops (slice J, ADR-0006 §5): a household that sets to work making a good to sell
//! sets up a workshop for it, an account with stores of its own. Its owners put in what the work
//! uses and draw out what it was paid; it posts terms for its stock on their review day, sells
//! through the ledger, keeps books, passes to the household its owners join, and closes when
//! nobody is left to keep it or it has sold nothing for months.

use civ_core::{Handle, PermanentId, SimTime};

use super::{Ctx, Population, fuel_per_day, stores_now};
use crate::firm::{BookKind, Entry, Exit, Firm};
use crate::history::ChronicleKind;
use crate::ledger::{Channel, Leg};
use crate::make;
use crate::params::{ActivityDef, interpolate};
use crate::person::{Flow, Household, Person};

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
        Some(id)
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
        }
        // The work is done with the owners' tools, at their home.
        self.wear_tools(ctx, owner, &recipe.tools, hours);
        if let (Some((k, s)), Some(p)) = (skill, self.people.get_mut(h)) {
            let next = s.practised(p.skill(k), hours);
            p.set_skill(k, next);
        }
    }

    /// Workshop `id` on its owners' review day: they draw out what it was paid, and any of its
    /// goods they now need themselves; it posts terms for its stock as they would for theirs;
    /// its stock is valued for the month; and once it has sold nothing for `idle_close_days` it is
    /// given up.
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
        let Some(f) = self.firm(id) else {
            return;
        };
        let mut legs = Vec::new();
        for (g, &kg) in f.stores.iter().enumerate() {
            if kg <= 1e-9 {
                continue;
            }
            let take = if f.lines.iter().any(|&l| usize::from(l) == g) {
                // What its owners lack of it themselves.
                (holding.keep.get(g).copied().unwrap_or(0.0)
                    - owner_stores.get(g).copied().unwrap_or(0.0))
                .clamp(0.0, kg)
            } else {
                kg
            };
            if take > 1e-9 {
                legs.push(Leg {
                    from: id,
                    to: owner,
                    good: g,
                    amount: take,
                });
            }
        }
        if !legs.is_empty() && self.transfer(now, params, goods, &legs, Channel::Owner) {
            let keep = params.firm.book_entries;
            if let Some(f) = self.firm_mut(id) {
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
        let costs = self.own_costs_of(ctx, &hh);
        let rows = match settlement {
            Some(s) => self.post_terms(ctx, s, &costs, &holding, &stock, &old),
            None => Vec::new(),
        };
        let worth: f64 = stock
            .iter()
            .map(|&(g, units)| units * costs.get(g).copied().flatten().unwrap_or(0.0))
            .sum();
        if let Some(f) = self.firm_mut(id) {
            f.offers = rows;
            f.books.month_mut(now).stock_h = worth as f32;
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
