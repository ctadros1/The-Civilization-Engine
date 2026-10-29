//! The telegraph (M11c; ADR-0036 §8): an office is a household that holds a whole telegraph
//! instrument and someone who can write and knows every technique of a recipe making it, its
//! operator. On its day of the year an office weighs a wire to an office of another settlement,
//! as a station weighs a line: what the errands its settlement's households ran to that market in
//! a year would pay it in fees, had each asked first by wire, against stringing the wire along
//! ways with leave and its conductor. It begins the best that repays over a wire's life and strings
//! it as its way work. A wire carries messages only once whole.
//!
//! What is sent (the third part): a household about to run an errand to another settlement's
//! market, where an office of its own settlement is joined by a whole wire to an office of the
//! market's, asks by wire what is offered there before it walks, where the chance its trip finds
//! nothing to buy (as its settlement's trips there have gone) times the walk there and back is
//! worth more to it than the fee. The telegram waits in each operator's queue, sent in the order
//! handed in while the office is kept; the far operator answers with what the market's sellers
//! offer as they stand, and the answer is walked to the household, which holds it as price
//! reports told by that operator and lets its errand go if nothing it meant to fetch is offered.
//! It waits on the answer before it sets out, two days at most.

use civ_core::time::DAYS_PER_YEAR;
use civ_core::{PermanentId, Rng64, SimTime};
use civ_land::wire::Telegraph;

use super::*;
use crate::decide::WayFacts;
use crate::ledger::Channel;
use crate::reports::{Errand, PriceReport, ReportHow, Telegram, TelegramStage};

/// What a message handed in at an office costs, in days of one person's food (a design prior, as
/// the meter's price is: some 125 kcal at 2,500 a day, an hour or so of a labourer's food).
pub const TELEGRAM_FEE_FOOD_DAYS: f64 = 0.05;

/// The longest a household waits on the answer to its telegram before it sets out on its errand
/// regardless, minutes.
pub const TELEGRAM_WAIT_MIN: i64 = 2 * MINUTES_PER_DAY;

/// The minutes of the day an office is kept, from and until (a design prior: eight to eight).
const OFFICE_OPENS_MIN: i64 = 8 * 60;
const OFFICE_CLOSES_MIN: i64 = 20 * 60;

/// Keyed-randomness purpose of an office's day of the year for weighing a wire.
const PURPOSE_TELEGRAPH_REVIEW: u64 = 0x7465_6c65_6772_6170; // "telegrap"

/// What an office reckons of a wire to another office at its review (M11c): the far office, the
/// cells and length, metres, what it would repay a year and what stringing it takes, hours.
#[derive(Clone, Debug, PartialEq)]
pub struct TelegraphReckoning {
    pub to: PermanentId,
    pub cells: Vec<u32>,
    pub length_m: f64,
    pub worth_h_year: f64,
    pub cost_h: f64,
}

/// A message's fee, kcal of food.
pub(super) fn fee_kcal(ctx: &Ctx) -> f64 {
    TELEGRAM_FEE_FOOD_DAYS * ctx.params.household.daily_kcal_per_person
}

impl Population {
    /// The telegraph instruments of the catalogue, by good index, with the messages an hour each
    /// sends.
    pub(super) fn telegraph_goods(ctx: &Ctx) -> Vec<(usize, f64)> {
        ctx.catalog
            .goods
            .iter()
            .enumerate()
            .filter_map(|(g, d)| Some((g, d.telegraph?.messages_h)))
            .collect()
    }

    /// Household `office`'s operator and the messages an hour its office sends, if it is an
    /// office (M11c): it holds a whole telegraph instrument, and someone grown of it (the first by
    /// id) can write and knows every technique of a recipe making that instrument. The best of
    /// its instruments where it holds several kinds.
    pub fn telegraph_office(&self, ctx: &Ctx, office: PermanentId) -> Option<(PermanentId, f64)> {
        let x = self.household(office)?;
        let writing = Self::letter_technique(ctx.catalog, Behavior::Write)?;
        Self::telegraph_goods(ctx)
            .into_iter()
            .filter(|&(g, _)| x.stores.get(g).copied().unwrap_or(0.0) >= 1.0 - 1e-9)
            .filter_map(|(g, rate)| {
                let needs: Vec<usize> = ctx
                    .catalog
                    .recipes
                    .iter()
                    .filter(|r| r.outputs.iter().any(|&(o, _)| o == g))
                    .filter_map(|r| r.technique)
                    .collect();
                let operator = x.members.iter().copied().filter(|&m| {
                    self.person(m).is_some_and(|p| {
                        p.age_years(ctx.now) >= ctx.params.family.independent_age
                            && p.knows(writing)
                            && needs.iter().all(|&t| p.knows(t))
                    })
                });
                Some((operator.min()?, rate))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// Each midnight, the offices whose day of the year it is weigh a wire to another office
    /// (M11c). Nothing where the land lays no wire or the catalogue has no instrument.
    pub(super) fn telegraph_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        if ctx.land_params.wire.is_none() || Self::telegraph_goods(ctx).is_empty() {
            return;
        }
        let due: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x.id)
            .filter(|&id| {
                Rng64::from_key(&[ctx.seed, PURPOSE_TELEGRAPH_REVIEW, id.get()]).next_u64()
                    % DAYS_PER_YEAR as u64
                    == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .filter(|&id| self.telegraph_office(ctx, id).is_some())
            .collect();
        for office in due {
            self.review_telegraph(ctx, office);
        }
    }

    /// The errands people of settlement `from` ran to the market of settlement `to` in a year,
    /// bought or not: the more of this year's and last year's counts.
    pub(super) fn errands_between(&self, ctx: &Ctx, from: PermanentId, to: PermanentId) -> u32 {
        let year = ctx.now.day_index().div_euclid(DAYS_PER_YEAR);
        [year - 1, year]
            .iter()
            .map(|&y| {
                self.contacts
                    .years
                    .get(&(y, from, to))
                    .map_or(0, |c| c.bought + c.missed.iter().sum::<u32>())
            })
            .max()
            .unwrap_or(0)
    }

    /// What wires from office `office` to an office of another settlement would repay it, the best
    /// first (M11c): to each such office it has no wire with, routed along ways with leave from
    /// its home to the other's, worth a year the fee in its own hours of food for each errand its
    /// settlement's people ran to that market in a year, as though each had asked by wire first;
    /// costing a metre's stringing and conductor at its own cost (or in hand) each metre. None
    /// while one of its wires is being strung.
    pub(super) fn telegraph_reckonings(
        &self,
        ctx: &Ctx,
        office: PermanentId,
    ) -> Vec<TelegraphReckoning> {
        let mut out = Vec::new();
        let Some(params) = ctx.land_params.wire else {
            return out;
        };
        let Some(x) = self.household(office) else {
            return out;
        };
        let Some(home) = x.settlement else {
            return out;
        };
        let wires = &ctx.land.wires;
        if self.telegraph_office(ctx, office).is_none()
            || wires.telegraphs.iter().any(|t| t.a == office && !t.whole())
        {
            return out;
        }
        let costs = self.own_costs_of(ctx, x);
        let Some(per_kcal) = super::wires::hours_per_kcal(&ctx.catalog.goods, &costs) else {
            return out;
        };
        let held = x.stores.get(params.good).copied().unwrap_or(0.0);
        let conductor_h = costs.get(params.good).copied().flatten();
        let from = cell_of(ctx.map, x.home) as u32;
        let fars: Vec<(PermanentId, PermanentId, u32)> = self
            .households
            .iter()
            .map(|(_, y)| y)
            .filter(|y| y.id != office && y.settlement.is_some_and(|s| s != home))
            .filter(|y| !wires.telegraphs.iter().any(|t| t.joins(office, y.id)))
            .filter(|y| self.telegraph_office(ctx, y.id).is_some())
            .filter_map(|y| Some((y.id, y.settlement?, cell_of(ctx.map, y.home) as u32)))
            .collect();
        for (to, market, cell) in fars {
            let errands = self.errands_between(ctx, home, market);
            if errands == 0 {
                continue;
            }
            let Some((cells, length_m)) = self
                .wire_route(ctx, office, from, cell)
                .filter(|r| r.1 >= 1e-3)
            else {
                continue;
            };
            let kg = length_m * params.good_kg_m;
            let conductor = match conductor_h {
                Some(h) => kg * h,
                None if held + 1e-9 >= kg => 0.0,
                None => continue,
            };
            out.push(TelegraphReckoning {
                to,
                cells,
                length_m,
                worth_h_year: f64::from(errands) * fee_kcal(ctx) * per_kcal,
                cost_h: length_m * params.lay_h_m + conductor,
            });
        }
        let net = |r: &TelegraphReckoning| r.worth_h_year * params.life_years - r.cost_h;
        out.sort_by(|a, b| net(b).total_cmp(&net(a)).then(a.to.cmp(&b.to)));
        out
    }

    /// What wires from office `office` would repay it, the best first: for tests.
    #[doc(hidden)]
    pub fn telegraph_reckonings_for_tests(
        &self,
        ctx: &Ctx,
        office: PermanentId,
    ) -> Vec<TelegraphReckoning> {
        self.telegraph_reckonings(ctx, office)
    }

    /// Office `office` weighs a wire (M11c): it begins the one that repays it most where that
    /// repays more over a wire's life than stringing it takes, marked out with what it reckoned.
    fn review_telegraph(&mut self, ctx: &mut Ctx, office: PermanentId) {
        let Some(params) = ctx.land_params.wire else {
            return;
        };
        let Some(best) = self.telegraph_reckonings(ctx, office).into_iter().next() else {
            return;
        };
        if best.worth_h_year * params.life_years <= best.cost_h {
            return;
        }
        if let Some(t) = self.begin_telegraph(ctx, office, best.to)
            && let Some(w) = ctx.land.wires.telegraphs.iter_mut().find(|w| w.id == t)
        {
            w.worth_h = best.worth_h_year;
        }
    }

    /// Office `office` weighs a wire now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_telegraph_for_tests(&mut self, ctx: &mut Ctx, office: PermanentId) {
        self.review_telegraph(ctx, office);
    }

    /// Begins a telegraph wire for household `a` from its home to household `b`'s, routed as a
    /// line of `a`'s is ([`Population::begin_line`]), marked out at its length with nothing yet
    /// strung. Returns its id; `None` where the land lays no wire or no route keeps to such ways.
    pub fn begin_telegraph(
        &self,
        ctx: &mut Ctx,
        a: PermanentId,
        b: PermanentId,
    ) -> Option<PermanentId> {
        ctx.land_params.wire?;
        let from = cell_of(ctx.map, self.household(a)?.home) as u32;
        let to = cell_of(ctx.map, self.household(b)?.home) as u32;
        let (cells, length_m) = self.wire_route(ctx, a, from, to)?;
        let id = ctx.ids.allocate();
        ctx.land.wires.telegraphs.push(Telegraph {
            id,
            a,
            b,
            cells,
            length_m,
            laid_m: 0.0,
            broken: false,
            worth_h: 0.0,
        });
        Some(id)
    }

    /// The telegraph wire household `x` is stringing, as way work for its people (M11c, as a
    /// line's): where its work stands, the walk there, the hours still to string, and each hour's
    /// worth, the hours a year it was reckoned to repay over a wire's life spread over its
    /// stringing. `None` without one, or without a metre's conductor in hand.
    pub(super) fn telegraph_facts(&self, ctx: &Ctx, x: &Household) -> Option<WayFacts> {
        let params = ctx.land_params.wire?;
        let t = ctx
            .land
            .wires
            .telegraphs
            .iter()
            .find(|t| t.a == x.id && !t.whole() && !t.broken)?;
        if x.stores.get(params.good).copied().unwrap_or(0.0) < params.good_kg_m {
            return None;
        }
        let n = t.cells.len();
        let share = t.laid_m / t.length_m.max(1e-3);
        let cell = *t
            .cells
            .get(((share * n as f64) as usize).min(n.checked_sub(1)?))?;
        let at = cell_centre(ctx.map, cell as usize);
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let labour_h = t.length_m * params.lay_h_m;
        Some(WayFacts {
            way: t.id,
            at,
            walk_min: f64::from((at.0 - x.home.0).hypot(at.1 - x.home.1)) / pace / 60.0,
            left_h: (t.length_m - t.laid_m).max(0.0) * params.lay_h_m,
            saves_per_hour: t.worth_h * params.life_years / labour_h.max(1e-6),
            duty: 0.0,
            full: false,
        })
    }

    /// `hours` of work by someone of `household` on its telegraph wire `wire`: a metre strung each
    /// `lay_h_m` hours, each taking its conductor from the household's store, no more than the
    /// conductor held allows or the wire has left.
    pub(super) fn telegraph_work(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        wire: PermanentId,
        hours: f64,
    ) {
        let Some(params) = ctx.land_params.wire else {
            return;
        };
        let Some(t) = ctx
            .land
            .wires
            .telegraphs
            .iter_mut()
            .find(|t| t.id == wire && t.a == household && !t.whole())
        else {
            return;
        };
        let Some(x) = self.household_mut(household) else {
            return;
        };
        let held = x.stores.get(params.good).copied().unwrap_or(0.0).max(0.0);
        let metres = (hours.max(0.0) / params.lay_h_m.max(1e-9))
            .min(held / params.good_kg_m.max(1e-9))
            .min((t.length_m - t.laid_m).max(0.0));
        t.laid_m = (t.laid_m + metres).min(t.length_m);
        if t.length_m - t.laid_m < 1e-3 {
            t.laid_m = t.length_m;
        }
        let used = (metres * params.good_kg_m).min(held);
        if used > 0.0 {
            x.stores[params.good] = held - used;
            x.flows.add(Flow::Used, params.good, used);
        }
    }

    /// `hours` of work on a telegraph wire: for tests.
    #[doc(hidden)]
    pub fn telegraph_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        wire: PermanentId,
        hours: f64,
    ) {
        self.telegraph_work(ctx, household, wire, hours);
    }
}

impl Population {
    /// Whether offices `x` and `y` are joined by a whole, unbroken telegraph wire.
    fn wired(ctx: &Ctx, x: PermanentId, y: PermanentId) -> bool {
        ctx.land
            .wires
            .telegraphs
            .iter()
            .any(|t| t.joins(x, y) && t.whole() && !t.broken)
    }

    /// The offices household `hh` would ask through after the market of settlement `market`
    /// (M11c): an office of its own settlement joined by a whole wire to an office of the
    /// market's; the nearest such office to its home, and of its wires the first by id.
    pub(super) fn telegraph_route(
        &self,
        ctx: &Ctx,
        hh: &Household,
        market: PermanentId,
    ) -> Option<(PermanentId, PermanentId)> {
        let home = hh.settlement?;
        let settlement_of = |h: PermanentId| self.household(h).and_then(|x| x.settlement);
        let mut routes: Vec<(f32, PermanentId, PermanentId)> = ctx
            .land
            .wires
            .telegraphs
            .iter()
            .filter(|t| t.whole() && !t.broken)
            .flat_map(|t| [(t.a, t.b), (t.b, t.a)])
            .filter(|&(n, f)| {
                settlement_of(n) == Some(home)
                    && settlement_of(f) == Some(market)
                    && self.telegraph_office(ctx, n).is_some()
                    && self.telegraph_office(ctx, f).is_some()
            })
            .filter_map(|(n, f)| {
                let at = self.household(n)?.home;
                Some(((at.0 - hh.home.0).hypot(at.1 - hh.home.1), n, f))
            })
            .collect();
        routes.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        routes.first().map(|&(_, n, f)| (n, f))
    }

    /// The telegram household `hh`, with its `stores` and the `costs` it puts on goods, about to
    /// run errand `e`, would hand in, if it would ask by wire first (M11c): one it has not
    /// already asked by, that holds the fee in food, where the chance its trip finds nothing to
    /// buy (its settlement's trips to that market this year and last that bought nothing, of all
    /// of them, each kind counted from one) times the walk there and back, hours, is worth more
    /// than the fee in its own hours of food.
    pub(super) fn telegraph_ask(
        &self,
        ctx: &Ctx,
        hh: &Household,
        e: &Errand,
        stores: &[f64],
        costs: &[Option<f64>],
    ) -> Option<Telegram> {
        if self.reports.telegrams.contains_key(&hh.id) || hh.settlement == Some(e.market) {
            return None;
        }
        let route = self.telegraph_route(ctx, hh, e.market)?;
        let goods = &ctx.catalog.goods;
        let per_kcal = super::wires::hours_per_kcal(goods, costs)?;
        let food: f64 = goods
            .iter()
            .enumerate()
            .filter(|(_, d)| d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 && !d.kept_back())
            .map(|(g, d)| stores.get(g).copied().unwrap_or(0.0).max(0.0) * d.kcal_per_kg)
            .sum();
        if food < fee_kcal(ctx) {
            return None;
        }
        let year = ctx.now.day_index().div_euclid(DAYS_PER_YEAR);
        let (mut bought, mut missed) = (0u32, 0u32);
        for y in [year - 1, year] {
            if let Some(c) = self.contacts.years.get(&(y, hh.settlement?, e.market)) {
                bought += c.bought;
                missed += c.missed.iter().sum::<u32>();
            }
        }
        let miss = f64::from(missed + 1) / f64::from(bought + missed + 2);
        let at = self.seller_home(e.seller, e.firm, e.market)?;
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let trip_h = 2.0 * f64::from((at.0 - hh.home.0).hypot(at.1 - hh.home.1)) / pace / 3600.0;
        (miss * trip_h > fee_kcal(ctx) * per_kcal).then(|| Telegram {
            near: route.0,
            far: route.1,
            market: e.market,
            handed: ctx.now.minutes(),
            due: -1,
            stage: TelegramStage::Handed,
            answer: Vec::new(),
        })
    }

    /// Household `household` weighs asking by wire about the errand it means to run, as at its
    /// review, and hands the telegram in if it would: for tests.
    #[doc(hidden)]
    pub fn hand_in_telegram_for_tests(&mut self, ctx: &Ctx, household: PermanentId) -> bool {
        let Some(x) = self.household(household) else {
            return false;
        };
        let Some(e) = self.reports.errands.get(&household).copied() else {
            return false;
        };
        let stores = stores_now(x, ctx.now, ctx.params, &ctx.catalog.goods);
        let costs = self.own_costs_of(ctx, x);
        let Some(t) = self.telegraph_ask(ctx, x, &e, &stores, &costs) else {
            return false;
        };
        self.reports.telegrams.insert(household, t);
        true
    }

    /// Whether household `household` waits on a telegram before it sets out on its errand: for
    /// tests.
    #[doc(hidden)]
    pub fn waits_on_telegram_for_tests(&self, ctx: &Ctx, household: PermanentId) -> bool {
        self.reports
            .errands
            .get(&household)
            .is_some_and(|e| self.waits_on_telegram(ctx, household, e.market))
    }

    /// Whether household `household` waits on the answer to a telegram after the market of
    /// settlement `market` before it sets out there (two days at most from handing it in).
    pub(super) fn waits_on_telegram(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        market: PermanentId,
    ) -> bool {
        self.reports
            .telegrams
            .get(&household)
            .is_some_and(|t| t.market == market && ctx.now.minutes() - t.handed < TELEGRAM_WAIT_MIN)
    }

    /// The minute office `office`'s operator sends its next message at or after minute `after`:
    /// one every sixty over its messages an hour minutes after the last in its queue, within the
    /// hours the office is kept. `None` where it is no office.
    fn office_slot(&self, ctx: &Ctx, office: PermanentId, after: i64) -> Option<i64> {
        let (_, rate) = self.telegraph_office(ctx, office)?;
        let gap = (60.0 / rate.max(1e-6)).ceil().max(1.0) as i64;
        let last = self
            .reports
            .telegrams
            .values()
            .filter(|t| match t.stage {
                TelegramStage::Asked => t.near == office,
                TelegramStage::Answering => t.far == office,
                _ => false,
            })
            .map(|t| t.due)
            .max()
            .unwrap_or(i64::MIN);
        let t = after.max(last) + gap;
        let start = t - t.rem_euclid(MINUTES_PER_DAY);
        Some(match t - start {
            m if m < OFFICE_OPENS_MIN + gap => start + OFFICE_OPENS_MIN + gap,
            m if m > OFFICE_CLOSES_MIN => start + MINUTES_PER_DAY + OFFICE_OPENS_MIN + gap,
            _ => t,
        })
    }

    /// The telegrams handed in at today's reviews (M11c): each pays its office the fee in food
    /// and joins its operator's queue. One whose office is no more is not sent.
    pub(super) fn telegrams_handed(&mut self, ctx: &mut Ctx) {
        let handed: Vec<PermanentId> = self
            .reports
            .telegrams
            .iter()
            .filter(|(_, t)| t.stage == TelegramStage::Handed)
            .map(|(&h, _)| h)
            .collect();
        let now = ctx.now.minutes();
        for sender in handed {
            let Some(near) = self.reports.telegrams.get(&sender).map(|t| t.near) else {
                continue;
            };
            let Some(due) = self.office_slot(ctx, near, now) else {
                self.reports.telegrams.remove(&sender);
                continue;
            };
            if sender != near {
                self.pay_food_by(ctx, sender, near, fee_kcal(ctx), Channel::Telegraph);
            }
            if let Some(t) = self.reports.telegrams.get_mut(&sender) {
                t.stage = TelegramStage::Asked;
                t.due = due;
            }
            ctx.schedule
                .push((SimTime::from_minutes(due), AgentEvent::Telegraph));
        }
    }

    /// The telegrams queued at today's reviews: for tests.
    #[doc(hidden)]
    pub fn telegrams_handed_for_tests(&mut self, ctx: &mut Ctx) {
        self.telegrams_handed(ctx);
    }

    /// What the sellers of settlement `market` offer as they stand, as reports told on `day` by
    /// `teller`: every household's offers and every open workshop's.
    fn market_offers(
        &self,
        market: PermanentId,
        day: i64,
        teller: PermanentId,
    ) -> Vec<PriceReport> {
        let sellers = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| x.settlement == Some(market))
            .map(|x| (x.id, false, x.offers.as_slice()))
            .chain(
                self.firms
                    .iter()
                    .filter(|f| f.is_open() && f.settlement == Some(market))
                    .map(|f| (f.id, true, f.offers.as_slice())),
            );
        let mut out = Vec::new();
        for (seller, firm, offers) in sellers {
            for o in offers {
                out.push(PriceReport {
                    market,
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
                });
            }
        }
        out
    }

    /// Each telegram whose step is due now takes it (M11c): one asked is sent and joins the far
    /// operator's queue; one come there is answered with what its market's sellers offer as they
    /// stand, and walked from the near office to the household that asked; one walked there is
    /// delivered. One whose offices or wire are gone is lost, the fee kept.
    pub(super) fn telegrams_due(&mut self, ctx: &mut Ctx) {
        let now = ctx.now.minutes();
        let due: Vec<PermanentId> = self
            .reports
            .telegrams
            .iter()
            .filter(|(_, t)| t.stage != TelegramStage::Handed && (0..=now).contains(&t.due))
            .map(|(&h, _)| h)
            .collect();
        for sender in due {
            let Some(t) = self.reports.telegrams.get(&sender).cloned() else {
                continue;
            };
            if t.stage == TelegramStage::Walking {
                self.reports.telegrams.remove(&sender);
                self.deliver_telegram(sender, &t);
                continue;
            }
            let far_operator = self.telegraph_office(ctx, t.far).map(|o| o.0);
            let up = self.telegraph_office(ctx, t.near).is_some()
                && far_operator.is_some()
                && Self::wired(ctx, t.near, t.far);
            let next = match (up, t.stage) {
                (true, TelegramStage::Asked) => self
                    .office_slot(ctx, t.far, now)
                    .map(|d| (TelegramStage::Answering, d, Vec::new())),
                (true, _) => {
                    let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
                    let walk = match (self.household(t.near), self.household(sender)) {
                        (Some(n), Some(x)) => {
                            f64::from((n.home.0 - x.home.0).hypot(n.home.1 - x.home.1))
                                / pace
                                / 60.0
                        }
                        _ => 0.0,
                    };
                    let answer = far_operator
                        .map(|o| self.market_offers(t.market, ctx.now.day_index(), o))
                        .unwrap_or_default();
                    Some((
                        TelegramStage::Walking,
                        now + (walk.ceil() as i64).max(1),
                        answer,
                    ))
                }
                (false, _) => None,
            };
            let Some((stage, due, answer)) = next else {
                self.reports.telegrams.remove(&sender);
                continue;
            };
            if let Some(x) = self.reports.telegrams.get_mut(&sender) {
                x.stage = stage;
                x.due = due;
                if stage == TelegramStage::Walking {
                    x.answer = answer;
                }
            }
            ctx.schedule
                .push((SimTime::from_minutes(due), AgentEvent::Telegraph));
        }
    }

    /// The telegrams whose step is due now: for tests.
    #[doc(hidden)]
    pub fn telegrams_due_for_tests(&mut self, ctx: &mut Ctx) {
        self.telegrams_due(ctx);
    }

    /// Household `sender` receives the answer to telegram `t`: it holds each offer as a report
    /// told by the far operator, and lets go an errand to that market where nothing it meant to
    /// fetch from its seller, in what it meant to pay, is offered.
    fn deliver_telegram(&mut self, sender: PermanentId, t: &Telegram) {
        if self.household(sender).is_none() {
            return;
        }
        for r in &t.answer {
            self.reports.note(sender, *r);
        }
        let gone = self.reports.errands.get(&sender).is_some_and(|e| {
            e.market == t.market
                && !t.answer.iter().any(|r| {
                    r.seller == e.seller
                        && r.firm == e.firm
                        && r.good == e.good
                        && r.payment == e.payment
                        && r.units > 0.0
                })
        });
        if gone {
            self.reports.errands.remove(&sender);
        }
    }
}
