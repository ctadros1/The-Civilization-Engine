//! Markets (slice I, ADR-0006 §4): each household reviews, on its own day, what it can spare and
//! the terms it posts; a buyer looks for the offer in its settlement that costs it the fewest
//! hours of its own work, the walk included; and the trade settles through the ledger when the
//! buyer arrives.

use std::collections::BTreeMap;

use civ_core::PermanentId;
use civ_core::time::{DAYS_PER_YEAR, SimTime};
use civ_land::LandParams;
use civ_world::nav::TravelField;

use super::{Ctx, Population, cell_of, fuel_per_day, stores_now};
use crate::condition;
use crate::decide::{TradeOption, TradeWorth};
use crate::farm;
use crate::firm::{BookKind, Entry};
use crate::ledger::{Channel, Leg, Trade};
use crate::make;
use crate::market::{Market, Offer};
use crate::params::{Catalog, GoodDef, GoodUse, MarketParams, PeopleParams, RecipeDef};
use crate::person::{Household, Person, stock_kcal};
use crate::reports::{Errand, Missed, PriceReport, ReportHow};
use crate::value;

/// Food a household keeps beyond what sees it to its next harvest before it offers any, a share
/// of that (a tuning value).
const FOOD_KEEP_MARGIN: f64 = 0.25;
/// Least of a tool worth offering, standard tools.
pub(super) const MIN_OFFER_TOOL: f64 = 0.5;
/// Least of any other good worth offering, kilograms.
pub(super) const MIN_OFFER_KG: f64 = 5.0;
/// Most goods a seller accepts in payment.
const MAX_PAYMENTS: usize = 4;

/// The farthest a landing may be from a seller's door for an errand by boat, metres (M8b slice
/// CO): its load is carried between them a hand load at a time, which the errand prices, so this
/// only bounds the search (about three quarters of an hour's walk).
pub(crate) const LANDING_WALK_M: f32 = 3_000.0;

/// A way by boat to a seller's door, as an errand prices it.
struct BoatWay {
    from: u32,
    to: u32,
    /// Hours to the boat and back, by water there and back, and from the landing to the door and
    /// back; and the walk one way between home and the boat, and between the landing and the door.
    travel_h: f64,
    home_walk_h: f64,
    far_walk_h: f64,
    payload_kg: f64,
    handling_kg_min: f64,
}
/// A way by cart to a seller's door and back, as an errand prices it: the hours there and back,
/// and the cart's load and handling.
struct CartWay {
    travel_h: f64,
    payload_kg: f64,
    handling_kg_min: f64,
}
/// Least a food stays good for to be offered, days: food that keeps (grain), not food made to
/// be eaten soon (bread).
const OFFER_FOOD_HALF_LIFE_DAYS: f64 = 365.0;

/// What a household keeps of each good, and how much more of each it wants, 0–1: 1 when it is
/// short of it, less as it holds more than it keeps, 0 for what it has no use for; and the food
/// it can spare beyond what it keeps, in years of its own need.
#[derive(Clone, Debug, Default)]
pub(crate) struct Holding {
    pub keep: Vec<f64>,
    pub want: Vec<f64>,
    pub spare_years: f64,
}

/// A vehicle a household could make that would keep it `gain_h` hours beyond the walk's on the
/// best errand it weighed (M8b slice CO, third part; slice CP, third part): its catalog index and
/// the cell it would be made at (a boat's bank, a cart's door).
struct VehicleWish {
    vehicle: usize,
    at: u32,
    gain_h: f64,
}

/// A seller a buyer could go to: its id, where it is, its offers, whether it is a workshop, how
/// far it is (none: by the buyer's walk), and how much the buyer believes its offers still stand
/// (1 for offers seen as they are; a report's weight for another settlement's, ADR-0019 §1).
struct Seller<'a> {
    id: PermanentId,
    at: (f32, f32),
    offers: std::borrow::Cow<'a, [Offer]>,
    firm: bool,
    walk_min: Option<f64>,
    weight: f64,
}

/// A purchase as it stands: from whom, what, how much, and what it is paid with.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Deal {
    pub seller: PermanentId,
    /// The seller is a workshop.
    pub firm: bool,
    pub at: (f32, f32),
    pub walk_min: f64,
    pub good: usize,
    pub units: f64,
    pub payment: usize,
    pub paid: f64,
    /// What the seller's terms say a unit is worth to it, hours.
    pub ask_h: f64,
    /// Hours of its own work the buyer saves, the walk included.
    pub saving_h: f64,
    pub worth: TradeWorth,
}

/// The ask after a review (research 08-04 §1.2: a cost anchor with a restrained scarcity
/// response): half the way toward the anchor, nudged up by demand nobody met and down when
/// nothing sold, never more than `max_change` either way.
fn reviewed_ask(prev: f64, anchor: f64, unmet: bool, unsold: bool, max_change: f64) -> f64 {
    let mut step = 0.5 * (anchor / prev.max(1e-9)).ln();
    if unmet {
        step += max_change / 2.0;
    }
    if unsold {
        step -= max_change / 2.0;
    }
    prev * step.clamp(-max_change, max_change).exp()
}

/// A seller's anchor for a good, its cost and margin `anchor`, when it can spare `spare_years`
/// of its own need of it: lower the more it can spare, by `exp(-response × s)` with `s` at most
/// one year (research 08-04 §1.2: modest inventory feedback; 08-05 §1.5: a seasonal producer's
/// stock against its trajectory to the next harvest, here measured in years of need so that a
/// store drawn down as planned does not move it).
fn stock_anchor(anchor: f64, spare_years: f64, response: f64) -> f64 {
    anchor * (-response.max(0.0) * spare_years.clamp(0.0, 1.0)).exp()
}

/// Whether a household offers what it holds of good `d` beyond what it keeps: a tool that can be
/// carried away, food not kept back for sowing, materials and stores (pots); never firewood.
pub(crate) fn can_offer(d: &GoodDef) -> bool {
    match d.purpose {
        GoodUse::Tool => !d.tool.as_ref().is_some_and(|t| t.fixed),
        GoodUse::Food => !d.kept_back(),
        GoodUse::Material | GoodUse::Store => true,
        GoodUse::Fuel => false,
    }
}

impl Population {
    /// The market of `settlement`, once anything has been offered or wanted there.
    pub fn market(&self, settlement: PermanentId) -> Option<&Market> {
        self.markets.iter().find(|m| m.settlement == settlement)
    }

    /// The money of `settlement`: the good most of its payments settle in, if one does (ADR-0006
    /// §4).
    pub fn money_of(
        &self,
        settlement: Option<PermanentId>,
        params: &MarketParams,
    ) -> Option<usize> {
        self.market(settlement?)?
            .money(params.money_share, params.money_min_trades)
    }

    fn market_mut(
        &mut self,
        settlement: PermanentId,
        goods: usize,
        day: i64,
        memory_days: f64,
    ) -> &mut Market {
        let i = match self.markets.iter().position(|m| m.settlement == settlement) {
            Some(i) => i,
            None => {
                self.markets.push(Market::new(settlement, goods, day));
                self.markets.len() - 1
            }
        };
        let m = &mut self.markets[i];
        m.age_to(day, memory_days);
        m
    }

    /// Whether household `hh`, holding `stores`, can work recipe `r` as a way to a tool (M7d
    /// slice CE): someone in it knows its technique, and each input it is short of comes from a
    /// place it knows (a patch it has worked, a deposit its settlement knows) or is made, by a
    /// recipe it knows, of inputs it holds or knows where to find. A way it believes cheap but
    /// cannot get the makings of is no way.
    pub(crate) fn can_work_way(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        r: &crate::params::RecipeDef,
    ) -> bool {
        let catalog = ctx.catalog;
        let knows = |t: usize| self.household_knows(&hh.members, t);
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0);
        let found = |g: usize| {
            hh.known.iter().any(|k| {
                ctx.land_params
                    .resources
                    .get(usize::from(k.resource))
                    .is_some_and(|res| res.good == g)
            }) || hh.settlement.is_some_and(|s| {
                ctx.land
                    .deposits
                    .iter()
                    .any(|d| usize::from(d.body.good) == g && self.knows_deposit(s, d.id))
            })
        };
        let gets = |g: usize, amount: f64| held(g) >= amount - 1e-9 || found(g);
        r.technique.is_none_or(&knows)
            && r.inputs.iter().all(|&(g, amount)| {
                gets(g, amount)
                    || catalog.recipes.iter().any(|m| {
                        m.technique.is_none_or(&knows)
                            && m.outputs.iter().any(|&(o, a)| o == g && a > 0.0)
                            && m.inputs.iter().all(|&(i, a)| gets(i, a))
                    })
            })
    }

    /// What a unit of good `p` is worth to household `hh`, holding `held` of it, as a payment,
    /// hours of its own work: its own cost (`own`); or, for a good paid by weight as money (M7d
    /// slice CD, ADR-0027 §5), the greater of that and what a unit bought in its purchases its
    /// markets remember, at its own costs (`costs`) of what it bought and faded by the markets'
    /// memory, less the hours of verifying it. Beyond its desired balance, `money_days` of its
    /// spending in it, what a unit buys is worth less to it, as stores beyond a target are
    /// (target ÷ held). The quantity of money reaches prices only by this belief.
    ///
    /// A money its polity's common store takes for its levy is worth to it at least the grain a
    /// unit pays for, as far as the levy could take it (M7d slice CE3b; research 08-06 §1.4: an
    /// authority makes a medium more useful by accepting it for obligations it enforces).
    pub(crate) fn pay_worth(
        &self,
        ctx: &Ctx,
        hh: &Household,
        p: usize,
        own: Option<f64>,
        held: f64,
        costs: &[Option<f64>],
    ) -> Option<f64> {
        let raw = self.pay_worth_raw(ctx, hh, p, own, held, costs);
        let Some(money) = ctx.catalog.goods.get(p).and_then(|g| g.money.as_ref()) else {
            return raw;
        };
        // One who can assay a coin values it by the metal in the pieces it would pay with, at its
        // own cost of the metal, not by what it bought (M7d slice CE4a, ADR-0027 §6).
        let raw = if money.coin && self.assays(hh.id, money.assay) {
            self.assayed_worth(hh, money, costs)
                .map(|v| v - money.verify_h)
                .filter(|&v| v > 0.0)
        } else if money.coin {
            // One who cannot assay takes another polity's pieces, which it pays with first, by
            // weight at the coin's discount, less weighing them, unless what they bought says
            // more (M7d slice CE4c).
            match self.foreign_worth(ctx, hh, money, costs) {
                Some(f) => Some(raw.map_or(f, |r| r.max(f))),
                None => raw,
            }
        } else {
            raw
        };
        let floor = self
            .levy_floor(ctx, hh, p, held, costs)
            .map(|f| f - money.verify_h)
            .filter(|&f| f > 0.0);
        match (raw, floor) {
            (Some(r), Some(f)) => Some(r.max(f)),
            (r, f) => r.or(f),
        }
    }

    /// What a piece of another polity's issue is worth by weight to household `hh`, which cannot
    /// assay it, in hours, when it holds such pieces (M7d slice CE4c): the metal in the lightest it
    /// holds, at what the metal is worth to it (its own cost, or what metal bought it), less the
    /// coin's foreign discount and the hours of weighing it.
    pub(crate) fn foreign_worth(
        &self,
        ctx: &Ctx,
        hh: &Household,
        money: &crate::params::MoneyDef,
        costs: &[Option<f64>],
    ) -> Option<f64> {
        let coinage = &self.order.coinage;
        let home = self.home_issuer(hh.id);
        let kg = coinage
            .purse(hh.id)?
            .counts
            .iter()
            .filter_map(|&(i, _)| coinage.cohort(i))
            .filter(|c| Some(c.issuer) != home)
            .map(|c| c.metal_kg())
            .reduce(f64::min)?;
        let metal = money.metal?;
        let held = hh.stores.get(metal).copied().unwrap_or(0.0);
        let metal_h = self.pay_worth_raw(
            ctx,
            hh,
            metal,
            costs.get(metal).copied().flatten(),
            held,
            costs,
        )?;
        let w = kg * metal_h * (1.0 - money.foreign_discount) - money.foreign_assay_h;
        (w > 0.0).then_some(w)
    }

    /// What a piece of coin `money` is worth to household `hh`, which can assay it, in hours: the
    /// metal in the piece it would pay with (of the issues its purse holds, the one holding least;
    /// with none, the newest struck), at its own cost of the metal (M7d slice CE4a).
    pub(crate) fn assayed_worth(
        &self,
        hh: &Household,
        money: &crate::params::MoneyDef,
        costs: &[Option<f64>],
    ) -> Option<f64> {
        let metal_h = costs.get(money.metal?).copied().flatten()?;
        let coinage = &self.order.coinage;
        let kg = match coinage.purse(hh.id) {
            Some(p) => p
                .counts
                .iter()
                .filter_map(|&(i, _)| coinage.cohort(i).map(|c| c.metal_kg()))
                .reduce(f64::min)?,
            None => coinage.cohorts.last()?.metal_kg(),
        };
        Some(kg * metal_h)
    }

    /// What a unit of money `p` pays for, in hours, under a common store in force where `hh`
    /// lives whose levy its people know may be paid in it: the grain it pays for, at the
    /// household's own cost of grain, less as it holds more than the levy could take of it (its
    /// share of the grain the household holds) (M7d slice CE3b).
    pub(crate) fn levy_floor(
        &self,
        ctx: &Ctx,
        hh: &Household,
        p: usize,
        held: f64,
        costs: &[Option<f64>],
    ) -> Option<f64> {
        let pi = self.polity_of(hh.settlement?)?;
        let grain = ctx.catalog.crops.get(ctx.params.farm.crop)?.good;
        let grain_h = costs.get(grain).copied().flatten()?;
        let grain_held = hh.stores.get(grain).copied().unwrap_or(0.0).max(0.0);
        self.polities[pi]
            .laws
            .iter()
            .filter(|l| {
                l.status == crate::polity::LawStatus::InForce
                    && l.kind == crate::polity::PolicyKind::CommonStore
            })
            .filter(|l| l.levy_share > 0.0 && hh.members.iter().any(|&m| l.knows(m)))
            .filter_map(|l| {
                let m = ctx.catalog.policies.get(usize::from(l.policy))?.medium?;
                if m.good != p || m.per_kg <= 0.0 {
                    return None;
                }
                let per_kg = f64::from(m.per_kg);
                let usable = f64::from(l.levy_share) * grain_held * per_kg;
                let scale = if held > usable {
                    usable / held.max(1e-9)
                } else {
                    1.0
                };
                Some(grain_h / per_kg * scale)
            })
            .reduce(f64::max)
    }

    /// [`Population::pay_worth`] without what a levy taking the money adds: what it bought.
    pub(crate) fn pay_worth_raw(
        &self,
        ctx: &Ctx,
        hh: &Household,
        p: usize,
        own: Option<f64>,
        held: f64,
        costs: &[Option<f64>],
    ) -> Option<f64> {
        let Some(money) = ctx.catalog.goods.get(p).and_then(|g| g.money.as_ref()) else {
            return own;
        };
        let mp = &ctx.params.market;
        let today = ctx.now.day_index();
        let (mut bought_h, mut paid) = (0.0, 0.0);
        for t in self
            .markets
            .iter()
            .flat_map(|m| m.recent.iter())
            .filter(|t| t.buyer == hh.id && usize::from(t.payment) == p && t.paid > 0.0)
        {
            let Some(c) = costs.get(usize::from(t.good)).copied().flatten() else {
                continue;
            };
            let age = (today - t.at.day_index()).max(0) as f64;
            let w = 0.5f64.powf(age / mp.memory_days.max(1e-6));
            bought_h += w * c * f64::from(t.units);
            paid += w * f64::from(t.paid);
        }
        // Units spent a day: the faded payments over the memory's mean age.
        let per_day = paid * std::f64::consts::LN_2 / mp.memory_days.max(1e-6);
        let desired = mp.money_days * per_day;
        let scale = if held > desired {
            desired / held.max(1e-9)
        } else {
            1.0
        };
        let bought = (paid > 1e-9).then(|| bought_h / paid * scale);
        let worth = match (own, bought) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
        worth.map(|v| v - money.verify_h).filter(|&v| v > 0.0)
    }

    /// What a unit of good `p` is worth as a payment to household `household` holding `held` of
    /// it ([`Population::pay_worth`]): for tests and probes.
    #[doc(hidden)]
    pub fn pay_worth_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        p: usize,
        held: f64,
    ) -> Option<f64> {
        let hh = self.household(household)?;
        let costs = self.own_costs_of(ctx, hh);
        self.pay_worth(ctx, hh, p, costs.get(p).copied().flatten(), held, &costs)
    }

    /// `costs` with every good paid by weight as money valued as a payment by household `hh`
    /// holding `stores` ([`Population::pay_worth`]); other goods as they are.
    pub(crate) fn pay_costs(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        costs: Vec<Option<f64>>,
    ) -> Vec<Option<f64>> {
        let goods = &ctx.catalog.goods;
        if goods.iter().all(|g| g.money.is_none()) {
            return costs;
        }
        (0..costs.len())
            .map(|p| {
                if goods.get(p).is_some_and(|g| g.money.is_some()) {
                    let held = stores.get(p).copied().unwrap_or(0.0).max(0.0);
                    self.pay_worth(ctx, hh, p, costs[p], held, &costs)
                } else {
                    costs[p]
                }
            })
            .collect()
    }

    /// The recipe household `household` would make tool `t` by, and its own costs of every good
    /// (M7d slice CE): for tests and probes.
    #[doc(hidden)]
    pub fn tool_way_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        t: usize,
    ) -> (Option<String>, Vec<Option<f64>>) {
        let Some(hh) = self.household(household) else {
            return (None, Vec::new());
        };
        let stores = stores_now(hh, ctx.now, ctx.params, &ctx.catalog.goods);
        let costs = self.own_costs_of(ctx, hh);
        let can_work = |r: &crate::params::RecipeDef| self.can_work_way(ctx, hh, &stores, r);
        let way = make::way_to_tool(ctx.catalog, t, &can_work, &|| costs.clone())
            .and_then(|r| ctx.catalog.recipes.get(r))
            .map(|r| r.id.clone());
        (way, costs)
    }

    /// What a household's goods cost it, hours of its own work per unit, at its best member's
    /// skills and from what it knows of the land.
    pub(crate) fn own_costs_of(&self, ctx: &Ctx, hh: &Household) -> Vec<Option<f64>> {
        self.own_costs_with(
            ctx.catalog,
            ctx.params,
            ctx.land_params,
            ctx.land,
            hh,
            ctx.now,
        )
    }

    /// [`Population::own_costs_of`], from the rules alone (for measures read outside a step).
    pub(crate) fn own_costs_with(
        &self,
        catalog: &Catalog,
        params: &PeopleParams,
        land_params: &LandParams,
        land: &civ_land::Land,
        hh: &Household,
        now: SimTime,
    ) -> Vec<Option<f64>> {
        let members: Vec<&Person> = hh.members.iter().filter_map(|m| self.person(*m)).collect();
        // A way that needs power is no way for a household without a working installation of its
        // own (M10a slice DR): it cannot grind by a wheel it has no headrace for.
        let powered = std::cell::OnceCell::new();
        let powered = || {
            *powered.get_or_init(|| {
                land.installations
                    .iter()
                    .enumerate()
                    .any(|(i, x)| x.owner == hh.id && land.installation_ready(i))
            })
        };
        // Each recipe at the skill of the best of the members who know it (ADR-0008: knowing is
        // the gate, skill is how well), or of them all if it needs no technique.
        let level = |r: &RecipeDef| -> Option<f64> {
            if r.kwh_per_unit > 0.0 && !powered() {
                return None;
            }
            let skill = |p: &&Person| r.skill.map_or(0.0, |k| p.skill(k));
            match r.technique {
                None => Some(members.iter().map(skill).fold(0.0, f64::max)),
                Some(t) => members
                    .iter()
                    .filter(|p| p.knows(t))
                    .map(skill)
                    .reduce(f64::max),
            }
        };
        let n = catalog.goods.len();
        let gathered = value::gathered_costs(&land_params.resources, &hh.known, n);
        // What it grows is carried in its store from the harvest, under its roof if it has one.
        let half_life = |g: usize| {
            catalog.goods.get(g).map_or(0.0, |d| {
                if hh.sheltered && d.sheltered_half_life_days > 0.0 {
                    d.sheltered_half_life_days
                } else {
                    d.half_life_days
                }
            })
        };
        let grown = value::grown_costs(
            &catalog.crops,
            n,
            params.farm.plan_yield_share,
            now.day_index(),
            &half_life,
        );
        let mut costs = value::own_costs(catalog, &level, &gathered, &grown);
        value::food_by_energy(&catalog.goods, &mut costs);
        costs
    }

    /// What a household keeps of each good and how much more of each it wants, from its
    /// `stores`: food to see it to its next harvest and its margin (pooled by energy), the tools
    /// its members' work needs and a spare, what the tools it lacks are made of, its firewood,
    /// and what it builds with while it has no finished home or builds anything.
    pub(crate) fn holding_of(&self, ctx: &Ctx, hh: &Household, stores: &[f64]) -> Holding {
        let (params, catalog) = (ctx.params, ctx.catalog);
        let goods = &catalog.goods;
        let n = goods.len();
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        let mut keep = vec![0.0; n];
        let mut want = vec![0.0; n];
        let members = hh.members.len().max(1);
        let kcal_day = members as f64 * params.household.daily_kcal_per_person;
        let outlook = catalog.crops.get(params.farm.crop).map_or(0.0, |c| {
            farm::days_to_harvest(c, self.fields_of(ctx.land, hh.id), ctx.now.day_index())
                + params.household.harvest_margin_days
        });
        let food_keep = outlook * kcal_day * (1.0 + FOOD_KEEP_MARGIN);
        let food = stock_kcal(stores, goods);
        let want_food = if food < food_keep || food <= 0.0 {
            1.0
        } else {
            food_keep / food
        };
        let mut spare_kcal = (food - food_keep).max(0.0);
        let spare_years = spare_kcal / (DAYS_PER_YEAR as f64 * kcal_day).max(1e-9);
        for (g, d) in goods.iter().enumerate() {
            if d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 && !d.kept_back() {
                want[g] = want_food;
                // The food it can spare comes out of what keeps longest.
                let keeps =
                    d.half_life_days <= 0.0 || d.half_life_days >= OFFER_FOOD_HALF_LIFE_DAYS;
                let spare = if keeps {
                    (spare_kcal / d.kcal_per_kg).min(held(g))
                } else {
                    0.0
                };
                spare_kcal -= spare * d.kcal_per_kg;
                keep[g] = held(g) - spare;
            }
        }
        let ages: Vec<f64> = hh
            .members
            .iter()
            .filter_map(|m| self.person(*m))
            .map(|p| p.age_years(ctx.now))
            .collect();
        let knows = |t: usize| self.household_knows(&hh.members, t);
        let mut wants = make::tool_wants_for(catalog, &ages, params.family.independent_age, &knows);
        super::cap_water_wants(ctx, hh.id, &mut wants);
        let can_work = |r: &crate::params::RecipeDef| self.can_work_way(ctx, hh, stores, r);
        let costs_once = std::cell::OnceCell::new();
        let costs = || {
            costs_once
                .get_or_init(|| self.own_costs_of(ctx, hh))
                .clone()
        };
        for (t, &w) in wants.iter().enumerate() {
            if w <= 0.0 {
                continue;
            }
            keep[t] = w + make::SPARE_TOOL;
            // The tools that serve for it are kept as it is (M7d slice CE).
            for s in make::serving(goods, t).skip(1) {
                keep[s] = keep[t];
            }
            let held_all: f64 = make::serving(goods, t).map(held).sum();
            if make::tool_need(w, held_all) > 0.0
                && let Some(r) = make::way_to_tool(catalog, t, &can_work, &costs)
                    .and_then(|r| catalog.recipes.get(r))
            {
                for &(g, amount) in &r.inputs {
                    if let Some(k) = keep.get_mut(g) {
                        *k += amount;
                    }
                    // What it makes that material of, as far as it is short of it.
                    let short = (amount - held(g)).max(0.0);
                    let made = catalog.recipes.iter().find_map(|m| {
                        m.outputs
                            .iter()
                            .find(|&&(o, a)| o == g && a > 0.0)
                            .filter(|_| short > 0.0 && can_work(m))
                            .map(|&(_, a)| (m, a))
                    });
                    if let Some((m, per_unit)) = made {
                        for &(i, a) in &m.inputs {
                            if let Some(k) = keep.get_mut(i) {
                                *k += a * short / per_unit;
                            }
                        }
                    }
                }
            }
        }
        // Until its home is finished, and while it builds or mends anything else it can work
        // on, what it builds with stays for that.
        let mine = || ctx.land.buildings.iter().filter(|b| b.household == hh.id);
        let mending = |b: &civ_land::Building| {
            catalog
                .building_index(&b.spec.program)
                .is_some_and(|i| condition::repair_of(b, &catalog.buildings[i].upkeep).is_some())
        };
        let housed = mine()
            .any(|b| b.finished() && b.standing() && ctx.catalog.is_dwelling(&b.spec.program))
            && mine()
                .all(|b| (b.finished() && !mending(b)) || !self.can_build(catalog, &hh.members, b));
        let fuel_day = fuel_per_day(params, members, ctx.now.day_index());
        for (g, d) in goods.iter().enumerate() {
            if d.purpose == GoodUse::Fuel {
                // The household's need is in kilograms of the first fuel; another by its heat.
                keep[g] = params.household.fuel_target_days * fuel_day
                    / crate::params::fuel_ratio(goods, g);
            }
            // What it builds with stays while it is not housed; a money is not built with.
            if d.purpose == GoodUse::Material && !housed && d.money.is_none() {
                keep[g] = keep[g].max(held(g));
            }
            if d.purpose != GoodUse::Food || d.kept_back() {
                want[g] = if keep[g] <= 0.0 {
                    0.0
                } else if held(g) < keep[g] {
                    1.0
                } else {
                    keep[g] / held(g)
                };
            }
        }
        Holding {
            keep,
            want,
            spare_years,
        }
    }

    /// Each household whose day it is reviews what it offers and on what terms (ADR-0006 §4):
    /// it offers what it holds beyond what it keeps, at its own cost and margin moved toward
    /// what the market shows, in each good it wants or its settlement's money; and the tools it
    /// needs and finds nobody offering are recorded as demand. Its workshops are reviewed the
    /// same day.
    pub(super) fn review_offers(&mut self, ctx: &Ctx, day: i64) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let mp = &params.market;
        let review_days = i64::from(mp.review_days.max(1));
        let mut ids: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| {
                x.settlement.is_some()
                    && !x.members.is_empty()
                    && (x.id.get() as i64 + day).rem_euclid(review_days) == 0
            })
            .map(|x| x.id)
            .collect();
        ids.sort_unstable();
        for id in ids {
            let Some(hh) = self.household(id) else {
                continue;
            };
            let Some(settlement) = hh.settlement else {
                continue;
            };
            let stores = stores_now(hh, now, params, goods);
            let own = self.own_costs_of(ctx, hh);
            // A money paid by weight is worth what it buys (M7d slice CD).
            let costs = self.pay_costs(ctx, hh, &stores, own);
            let holding = self.holding_of(ctx, hh, &stores);
            let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
            // What it can spare.
            let mut offered: Vec<(usize, f64)> = Vec::new();
            for (g, d) in goods.iter().enumerate() {
                let spare = held(g) - holding.keep.get(g).copied().unwrap_or(0.0);
                let least = if d.tool.is_some() {
                    MIN_OFFER_TOOL
                } else {
                    MIN_OFFER_KG
                };
                if can_offer(d) && spare >= least {
                    offered.push((g, spare));
                }
            }
            let old = hh.offers.clone();
            // Asks may anchor on what replacing a good from another settlement would cost
            // (M5b slice AQ, ADR-0019 §5).
            let anchors: Option<Vec<Option<f64>>> = (!self.reports.of(id).is_empty()).then(|| {
                costs
                    .iter()
                    .enumerate()
                    .map(|(g, &own)| {
                        if !offered.iter().any(|&(o, _)| o == g) {
                            return own;
                        }
                        match (own, self.replacement_cost(ctx, hh, g, own, &costs)) {
                            (Some(own), Some(r)) => Some(own.min(r)),
                            (own, r) => own.or(r),
                        }
                    })
                    .collect()
            });
            let rows = self.post_terms(
                ctx,
                settlement,
                &costs,
                anchors.as_deref().unwrap_or(&costs),
                &holding,
                &offered,
                &old,
            );
            // The tools it needs and nobody in its settlement offers: demand on record, as much
            // as it needs, weighted so that a want recorded at every review adds up, as the
            // market forgets, to the want itself (a level, not a count of reviews).
            let offered_here: Vec<usize> = self
                .households
                .iter()
                .map(|(_, x)| x)
                .filter(|x| x.settlement == Some(settlement) && x.id != id)
                .flat_map(|x| x.offers.iter().map(|o| usize::from(o.good)))
                .chain(
                    self.firms
                        .iter()
                        .filter(|f| {
                            f.is_open() && f.settlement == Some(settlement) && f.owner != id
                        })
                        .flat_map(|f| f.offers.iter().map(|o| usize::from(o.good))),
                )
                .collect();
            let mut wanted: Vec<(usize, f64, f64)> = Vec::new();
            for (t, d) in goods.iter().enumerate() {
                // Demand is for the work: recorded for a tool, not for those that serve for it,
                // and not while any of them is offered (M7d slice CE).
                if d.tool.as_ref().is_none_or(|x| x.serves.is_some())
                    || make::serving(goods, t).any(|g| offered_here.contains(&g))
                {
                    continue;
                }
                let wants =
                    (holding.keep.get(t).copied().unwrap_or(0.0) - make::SPARE_TOOL).max(0.0);
                let need = make::tool_need(wants, make::serving(goods, t).map(held).sum());
                if need > 0.0
                    && let Some(c) = costs.get(t).copied().flatten()
                {
                    wanted.push((t, need, c));
                }
            }
            // What it might fetch from elsewhere to sell at home (M5b slice AQ, ADR-0019 §6).
            let (errand, wish) = self.plan_errand(ctx, hh, &stores, &costs, &holding, &offered);
            // A boat or cart it could make that would save it more over its life than making
            // it takes (M8b slice CO, third part; slice CP, third part): an errand a review, the
            // hours it would keep beyond the walk's on the best of them.
            for w in wish {
                let saves_h_year = w.gain_h * DAYS_PER_YEAR as f64 / review_days as f64;
                if ctx
                    .catalog
                    .vehicles
                    .get(w.vehicle)
                    .is_some_and(|v| saves_h_year * v.life_years > v.labour_h)
                {
                    self.boat_wishes
                        .push((id, w.vehicle, w.at, saves_h_year as f32));
                }
            }
            // A privilege that binds it, kept, withdraws the offers it covers, and one who keeps
            // a staple fetches no such goods to sell (M7c slice BX).
            let (rows, staples) = self.keep_privileges(ctx, id, rows);
            let errand = errand.filter(|e| {
                goods
                    .get(usize::from(e.good))
                    .is_none_or(|d| !staples.contains(&d.purpose))
            });
            if let Some(x) = self.household_mut_by_id(id) {
                x.offers = rows;
            }
            match errand {
                Some(e) => {
                    self.reports.errands.insert(id, e);
                }
                None => {
                    self.reports.errands.remove(&id);
                }
            }
            let weight = 1.0 - 0.5f64.powf(review_days as f64 / mp.memory_days.max(1e-6));
            let m = self.market_mut(settlement, goods.len(), day, mp.memory_days);
            for (t, need, worth_h) in wanted {
                m.record_unmet(t, need * weight, worth_h * need * weight);
            }
        }
        // Workshops are reviewed on their owners' day.
        let firms: Vec<PermanentId> = self
            .firms
            .iter()
            .filter(|f| f.is_open() && (f.owner.get() as i64 + day).rem_euclid(review_days) == 0)
            .map(|f| f.id)
            .collect();
        for id in firms {
            self.review_firm(ctx, id);
        }
    }

    /// The terms a seller posts for what it offers (`offered`: (good, units)), at its own
    /// `costs` and margin moved toward what the market shows (its asks in `old`), in the goods
    /// it wants (`holding`) or its settlement's money: those it is shortest of first, then what
    /// its neighbours already pay in, what keeps longest and what is worth most for its weight
    /// (research 08-06 §1.3: acceptance follows familiarity, durability and portability).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn post_terms(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        costs: &[Option<f64>],
        anchors: &[Option<f64>],
        holding: &Holding,
        offered: &[(usize, f64)],
        old: &[Offer],
    ) -> Vec<Offer> {
        let (params, goods) = (ctx.params, &ctx.catalog.goods);
        let mp = &params.market;
        let money = self.money_of(Some(settlement), mp);
        // What it takes in payment: what it wants, or its settlement's money, each valued at
        // its own cost.
        let mut accepts: Vec<(usize, f64)> = goods
            .iter()
            .enumerate()
            .filter(|(_, d)| !d.kept_back() && !d.tool.as_ref().is_some_and(|t| t.fixed))
            .filter_map(|(p, _)| {
                let want = if money == Some(p) {
                    1.0
                } else {
                    holding.want.get(p).copied().unwrap_or(0.0)
                };
                let cost = costs.get(p).copied().flatten()?;
                (want >= mp.accept_want && cost > 0.0).then_some((p, cost * want))
            })
            .collect();
        let want_of = |p: usize| {
            if money == Some(p) {
                1.0
            } else {
                holding.want.get(p).copied().unwrap_or(0.0)
            }
        };
        let accepted: Vec<f64> = self
            .market(settlement)
            .map(|m| m.acceptance())
            .unwrap_or_default();
        let keeps = |p: usize| match goods[p].half_life_days {
            h if h > 0.0 => h,
            _ => f64::INFINITY,
        };
        accepts.sort_by(|a, b| {
            want_of(b.0)
                .total_cmp(&want_of(a.0))
                .then(
                    accepted
                        .get(b.0)
                        .copied()
                        .unwrap_or(0.0)
                        .total_cmp(&accepted.get(a.0).copied().unwrap_or(0.0)),
                )
                .then(keeps(b.0).total_cmp(&keeps(a.0)))
                .then(b.1.total_cmp(&a.1))
                .then(a.0.cmp(&b.0))
        });
        let (unmet, sold): (Vec<f64>, Vec<f64>) = match self.market(settlement) {
            Some(m) => (m.unmet.clone(), m.sold.clone()),
            None => (Vec::new(), Vec::new()),
        };
        let mut rows = Vec::new();
        for &(g, units) in offered {
            // Food is asked for less the more of it the seller can spare (ADR-0006 §4's stock
            // term), so asks answer the harvest.
            let stock = if goods[g].purpose == GoodUse::Food {
                holding.spare_years
            } else {
                0.0
            };
            let anchor = anchors
                .get(g)
                .copied()
                .flatten()
                .map(|c| stock_anchor(c * (1.0 + mp.margin), stock, mp.stock_response));
            let prev = old
                .iter()
                .find(|o| usize::from(o.good) == g)
                .map(|o| f64::from(o.ask_h));
            let ask = match (prev, anchor) {
                (Some(prev), Some(anchor)) => reviewed_ask(
                    prev,
                    anchor,
                    unmet.get(g).copied().unwrap_or(0.0) > 0.5,
                    sold.get(g).copied().unwrap_or(0.0) < 0.1,
                    mp.max_change,
                ),
                (None, Some(anchor)) => anchor,
                (Some(prev), None) => prev,
                (None, None) => continue,
            };
            // Terms in the goods it takes, as far as one person could carry what a unit costs
            // (research 08-06 §1.3: a payment must be portable).
            let mut posted = 0;
            for &(p, value_h) in &accepts {
                if p == g || posted == MAX_PAYMENTS {
                    continue;
                }
                let price = ask / value_h;
                let unit = if goods[g].tool.is_some() {
                    1.0
                } else {
                    MIN_OFFER_KG
                };
                if goods[p].tool.is_none() && price * unit > params.household.carry_kg {
                    continue;
                }
                // A tool is paid whole or worn, not in splinters: at least half a tool.
                if goods[p].tool.is_some() && price * unit < MIN_OFFER_TOOL {
                    continue;
                }
                rows.push(Offer {
                    good: g as u16,
                    payment: p as u16,
                    price: price as f32,
                    units: units as f32,
                    ask_h: ask as f32,
                });
                posted += 1;
            }
        }
        rows
    }

    /// The best purchase household `hh` can make with its `stores` as they stand: the offer in
    /// its settlement that saves it the most hours of its own work over getting the good itself,
    /// the walk there and back (`walk_min(seller)`, minutes one way) included, for a tool it
    /// needs or food it is short of. `only` limits the search to one seller (a buyer at the
    /// seller's door), in whichever settlement it lives (M5b slice AP: a buyer from elsewhere
    /// who came by a report).
    pub(crate) fn find_deal(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        walk_min: &dyn Fn((f32, f32)) -> Option<f64>,
        only: Option<PermanentId>,
    ) -> Option<Deal> {
        let settlement = hh.settlement?;
        let here = |s: Option<PermanentId>| only.is_some() || s == Some(settlement);
        // Its neighbours' offers, and those of the workshops among them (at their owners'
        // homes), not its own: the one seller's at its door, else its settlement's.
        let one = only.and_then(|s| self.household(s));
        let near = only
            .is_none()
            .then(|| self.households_in(settlement))
            .into_iter()
            .flatten();
        let sellers: Vec<Seller> = one
            .into_iter()
            .chain(near)
            .filter(|x| x.id != hh.id && !x.offers.is_empty())
            .map(|x| Seller {
                id: x.id,
                at: x.home,
                offers: std::borrow::Cow::Borrowed(x.offers.as_slice()),
                firm: false,
                walk_min: None,
                weight: 1.0,
            })
            .chain(
                self.firms
                    .iter()
                    .filter(|f| {
                        f.is_open()
                            && f.owner != hh.id
                            && here(f.settlement)
                            && !f.offers.is_empty()
                            && only.is_none_or(|s| s == f.id)
                    })
                    .filter_map(|f| {
                        let at = self.household(f.owner)?.home;
                        Some(Seller {
                            id: f.id,
                            at,
                            offers: std::borrow::Cow::Borrowed(f.offers.as_slice()),
                            firm: true,
                            walk_min: None,
                            weight: 1.0,
                        })
                    }),
            )
            .collect();
        self.best_deal(ctx, hh, stores, sellers, walk_min)
    }

    /// The best of `sellers`' offers for household `hh` with its `stores`, as
    /// [`Population::find_deal`] weighs them: what a buyer saves, believed by each seller's
    /// weight, less the walk there and back.
    fn best_deal(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        sellers: Vec<Seller>,
        walk_min: &dyn Fn((f32, f32)) -> Option<f64>,
    ) -> Option<Deal> {
        let (params, goods) = (ctx.params, &ctx.catalog.goods);
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        if sellers.is_empty() {
            return None;
        }
        // Its own costs, worked out only once an offer is worth pricing (most are not).
        let own_costs = std::cell::OnceCell::new();
        let costs = || own_costs.get_or_init(|| self.own_costs_of(ctx, hh));
        let holding = self.holding_of(ctx, hh, stores);
        // Food it is short of: what it keeps less what it has, by energy.
        let (short_kcal, food_keep) = food_short(goods, &holding.keep, stores);
        let lean = if food_keep > 0.0 {
            short_kcal / food_keep
        } else {
            0.0
        };
        let mut best: Option<Deal> = None;
        for s in sellers {
            let (seller, at, firm) = (s.id, s.at, s.firm);
            let Some(walk) = s.walk_min.or_else(|| walk_min(at)) else {
                continue;
            };
            for o in s.offers.iter() {
                let (g, p) = (usize::from(o.good), usize::from(o.payment));
                let Some(d) = goods.get(g) else {
                    continue;
                };
                // What it would buy: a tool it needs (up to one standard tool, worn or not), or
                // food it is short of (what one person carries, no more than it is short).
                let (units, worth, own_h) = if d.tool.is_some() {
                    let wanted = (holding.keep[g] - make::SPARE_TOOL).max(0.0);
                    let all = make::serving(goods, make::served(goods, g)).map(held).sum();
                    let need = make::tool_need(wanted, all);
                    let units = f64::from(o.units).min(1.0);
                    if need <= 0.0 || units < MIN_OFFER_TOOL - 1e-6 {
                        continue;
                    }
                    let own = costs()[g].unwrap_or(f64::INFINITY) * units;
                    (units, TradeWorth::Tool { need }, own)
                } else if d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 {
                    if short_kcal <= 0.0 {
                        continue;
                    }
                    let units = (short_kcal / d.kcal_per_kg)
                        .min(params.household.carry_kg)
                        .min(f64::from(o.units));
                    if units < 1.0 {
                        continue;
                    }
                    // Short of food before the harvest, growing more is no help now: food is
                    // worth more the shorter the household is.
                    let own = costs()[g].unwrap_or(f64::INFINITY) * (1.0 + lean) * units;
                    let kcal = units * d.kcal_per_kg;
                    (units, TradeWorth::Food { kcal }, own)
                } else {
                    continue;
                };
                // Food bought is limited to a payment one person can carry back.
                let units = if goods[p].tool.is_none() && d.tool.is_none() && o.price > 0.0 {
                    units.min(params.household.carry_kg / f64::from(o.price))
                } else {
                    units
                };
                let paid = f64::from(o.price) * units;
                // It pays out of what it holds beyond what it keeps, valued at its own cost; a
                // tool whole or worn, not in splinters.
                let spare = held(p) - holding.keep.get(p).copied().unwrap_or(0.0);
                if !(paid > 0.0 && paid <= spare + 1e-9)
                    || (goods[p].tool.is_some() && paid < MIN_OFFER_TOOL - 1e-9)
                {
                    continue;
                }
                let own_pay = costs().get(p).copied().flatten();
                let Some(pay_cost) = self.pay_worth(ctx, hh, p, own_pay, held(p), costs()) else {
                    continue;
                };
                let pay_h = paid * pay_cost * holding.want.get(p).copied().unwrap_or(0.0);
                // What it saves is believed as far as the seller's terms are (ADR-0019 §1); the
                // walk is certain.
                let saving = s.weight * (own_h - pay_h) - 2.0 * walk / 60.0;
                if saving.is_nan() || saving <= 0.0 {
                    continue;
                }
                // What it cannot get itself saves all a buyer would give: rank those by price.
                let rank = if saving.is_finite() {
                    saving
                } else {
                    1e12 - pay_h
                };
                let better = best.is_none_or(|b| {
                    let b_rank = if b.saving_h.is_finite() {
                        b.saving_h
                    } else {
                        1e12 - b.paid
                    };
                    rank > b_rank + 1e-9
                        || ((rank - b_rank).abs() <= 1e-9
                            && (seller, g, p) < (b.seller, b.good, b.payment))
                });
                if better {
                    best = Some(Deal {
                        seller,
                        firm,
                        at,
                        walk_min: walk,
                        good: g,
                        units,
                        payment: p,
                        paid,
                        ask_h: f64::from(o.ask_h),
                        saving_h: saving,
                        worth,
                    });
                }
            }
        }
        best
    }

    /// The purchase a person of household `hh` could go and make, for the decision: see
    /// [`Population::find_deal`]. `reach` is the walking field from the household's home.
    pub(crate) fn best_purchase(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        reach: Option<&TravelField>,
    ) -> Option<TradeOption> {
        let reach = reach?;
        let walk = |at: (f32, f32)| {
            reach
                .seconds_to(cell_of(ctx.map, at))
                .map(|s| f64::from(s) / 60.0)
        };
        let d = self.find_deal(ctx, hh, stores, &walk, None)?;
        Some(TradeOption {
            seller: d.seller,
            firm: d.firm,
            walk_min: d.walk_min,
            at: d.at,
            good: d.good,
            worth: d.worth,
            by_boat: None,
            by_cart: None,
        })
    }

    /// A buyer of household `buyer` at the door of `seller`: the best deal it can make there now
    /// settles through the ledger, as barter or, paid in the money of the seller's market, a
    /// sale; the seller's offer shrinks by what it sold, and the seller's market remembers the
    /// trade (ADR-0019 §4: tallied where it settles). A buyer from another settlement, who came by
    /// a report (M5b slice AP), sees the seller's offers as they stand, and a trip that buys
    /// nothing is counted with why.
    pub(super) fn settle_trade(
        &mut self,
        ctx: &mut Ctx,
        buyer: PermanentId,
        seller: PermanentId,
        who: PermanentId,
    ) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(hh) = self.household(buyer) else {
            return;
        };
        let Some(settlement) = hh.settlement else {
            return;
        };
        // The market the seller belongs to.
        let market = if let Some(f) = self.firm(seller) {
            f.settlement
        } else {
            self.household(seller).and_then(|x| x.settlement)
        };
        let Some(market) = market else {
            return;
        };
        let away = market != settlement;
        let stores = stores_now(hh, now, params, goods);
        let mut deal = self.find_deal(ctx, hh, &stores, &|_| Some(0.0), Some(seller));
        // Nothing for its own need: one on an errand buys to sell at home (M5b slice AQ,
        // ADR-0019 §6), or says why it could not.
        let mut missed = None;
        // An errand by boat or cart carries what it buys to sell in a consignment (M8b slices
        // CO and CP), in the boat or cart.
        let vehicle = self
            .reports
            .errands
            .get(&buyer)
            .filter(|e| e.seller == seller)
            .and_then(|e| e.by_boat.map(|b| b.boat).or(e.by_cart.map(|c| c.cart)));
        let mut for_resale = false;
        if away && deal.is_none() && !self.stop_trade_between {
            match self.resale_at(ctx, hh, &stores, seller) {
                Some(Ok(d)) => {
                    deal = Some(d);
                    for_resale = true;
                }
                Some(Err(why)) => missed = Some(why),
                None => {}
            }
        }
        let year = now.day_index().div_euclid(DAYS_PER_YEAR);
        // The walk there, by when the buyer set off; the walk home is as long (ADR-0019 §7).
        let month = crate::market::month_of(now);
        let walk_h = self.person(who).map_or(0.0, |p| {
            2.0 * (now.minutes() - p.act.started.minutes()).max(0) as f64 / 60.0
        });
        if away {
            self.convergence.trip(month, settlement, market, walk_h);
            // What they see of its new buildings from the seller's door (M5b slice AR).
            let door = match self.firm(seller) {
                Some(f) => self.household(f.owner).map(|x| x.home),
                None => self.household(seller).map(|x| x.home),
            };
            if let Some(at) = door {
                self.note_sights(ctx, who, market, at);
                // And the work its people are doing about it.
                self.watch_work(ctx, who, market, at);
            }
            // An errand to this door is run, whatever came of it.
            if self
                .reports
                .errands
                .get(&buyer)
                .is_some_and(|e| e.seller == seller)
            {
                self.reports.errands.remove(&buyer);
            }
            // The demo's twin: purchases between settlements are stopped (ADR-0019 §8), here
            // for one who set out before they were.
            if self.stop_trade_between {
                return;
            }
        }
        let Some(d) = deal else {
            if away {
                let why =
                    missed.unwrap_or_else(|| self.missed_why(ctx, buyer, market, seller, &stores));
                self.contacts.missed(year, settlement, market, why);
                self.note_seen(buyer, market, seller, now.day_index());
            }
            return;
        };
        // Bought to sell at home on an errand by boat or cart: into a load in transit, carried by
        // `who` in it (ADR-0029 §3); otherwise into the buyer's store at the door (ADR-0019 §3).
        let consigned = match vehicle.filter(|_| away && for_resale) {
            Some(vehicle) => {
                let id = ctx.ids.allocate();
                let c = crate::consignments::Consignment::new(
                    id,
                    (buyer, who, vehicle),
                    (market, settlement),
                    now,
                    goods.len(),
                );
                self.consignments.push(c);
                Some(id)
            }
            None => None,
        };
        let legs = [
            Leg {
                from: seller,
                to: consigned.unwrap_or(buyer),
                good: d.good,
                amount: d.units,
            },
            Leg {
                from: buyer,
                to: seller,
                good: d.payment,
                amount: d.paid,
            },
        ];
        let channel = if self.money_of(Some(market), &params.market) == Some(d.payment) {
            Channel::Sale
        } else {
            Channel::Barter
        };
        if !self.transfer(now, params, goods, &legs, channel) {
            if consigned.is_some() {
                self.consignments.pop();
            }
            // One who came from elsewhere bought nothing and says why (ADR-0019 §2): the seller
            // had less than its terms said, or the buyer less to pay with; and holds what it
            // saw, so that it does not come again for what is not there.
            if away {
                let stock = |id: PermanentId, g: usize| {
                    self.firm(id)
                        .map(|f| f.stores.get(g).copied().unwrap_or(0.0))
                        .or_else(|| {
                            self.household(id)
                                .map(|x| x.stores.get(g).copied().unwrap_or(0.0))
                        })
                        .unwrap_or(0.0)
                };
                let why = if stock(seller, d.good) < d.units {
                    Missed::SoldOut
                } else {
                    Missed::Payment
                };
                self.contacts.missed(year, settlement, market, why);
                self.note_seen(buyer, market, seller, now.day_index());
                if why == Missed::SoldOut {
                    let gone: Vec<(u16, u16)> = self
                        .reports
                        .of(buyer)
                        .iter()
                        .filter(|r| r.market == market && r.seller == seller)
                        .filter(|r| usize::from(r.good) == d.good)
                        .map(|r| (r.good, r.payment))
                        .collect();
                    for key in gone {
                        self.reports
                            .sold_out(buyer, market, seller, key, now.day_index());
                    }
                }
            }
            return;
        }
        // Buyer and seller each saw the other keep to the terms (ADR-0014 §2).
        let seller_household = if d.firm {
            self.firm(seller).map(|f| f.owner)
        } else {
            Some(seller)
        };
        if let Some(sh) = seller_household.filter(|&sh| sh != buyer) {
            let traded = crate::ties::Act::Traded;
            self.note_between(ctx, Some(who), buyer, sh, traded, traded, 0.0);
        }
        // What one from elsewhere bought may show a technique it was made with (M5b slice AR).
        if away {
            self.see_made_with(ctx, who, market, d.good);
        }
        let shrink = |offers: &mut Vec<Offer>| {
            for o in offers.iter_mut().filter(|o| usize::from(o.good) == d.good) {
                o.units = (f64::from(o.units) - d.units).max(0.0) as f32;
            }
            offers.retain(|o| o.units > 1e-6);
        };
        if d.firm {
            // A workshop's books keep the sale and what it was paid, at what that is worth to
            // its owners.
            let owner = self.firm(seller).map(|f| f.owner);
            let worth = owner
                .and_then(|o| self.household(o).cloned())
                .map_or(0.0, |x| {
                    self.own_costs_of(ctx, &x)
                        .get(d.payment)
                        .copied()
                        .flatten()
                        .unwrap_or(0.0)
                        * d.paid
                });
            let keep = params.firm.book_entries;
            if let Some(f) = self.firms.iter_mut().find(|f| f.id == seller) {
                shrink(&mut f.offers);
                let entry = |kind, good: usize, amount: f64| Entry {
                    at: now,
                    kind,
                    good: good as u16,
                    amount: amount as f32,
                    other: Some(buyer),
                };
                f.books
                    .record(entry(BookKind::Sold, d.good, d.units), 0.0, keep);
                f.books
                    .record(entry(BookKind::Paid, d.payment, d.paid), worth, keep);
            }
            self.note_sale(seller, now);
        } else if let Some(x) = self.household_mut_by_id(seller) {
            shrink(&mut x.offers);
        }
        let trade = Trade {
            at: now,
            seller,
            buyer,
            good: d.good as u16,
            units: d.units as f32,
            payment: d.payment as u16,
            paid: d.paid as f32,
            channel,
            from: away.then_some(settlement),
        };
        let mp = &params.market;
        let m = self.market_mut(market, goods.len(), now.day_index(), mp.memory_days);
        m.record_trade(trade, d.ask_h * d.units, mp.recent_trades);
        if away {
            self.contacts.bought(year, settlement, market);
            self.convergence
                .carry(month, settlement, market, d.good as u16, d.units);
            self.note_seen(buyer, market, seller, now.day_index());
            // An embargo on these doors, broken (M7c slice BV); and the duties owed on what was
            // bought.
            if let Some((pi, li, _)) = self.embargo_on(ctx, who, buyer, market) {
                let knew = self.polities[pi].laws[li].knows(who);
                let c = &mut self.polities[pi].laws[li].compliance;
                if knew {
                    c.broken += 1;
                } else {
                    c.broken_unaware += 1;
                }
            }
            self.pay_duties(
                ctx,
                who,
                buyer,
                market,
                d.good,
                d.units,
                (d.payment, d.paid),
            );
        }
    }

    /// What household `hh` believes replacing a unit of good `g` would cost it, hours of its own
    /// work, by the price reports it holds of other settlements' markets (M5b slice AQ, ADR-0019
    /// §5; research 08-05 §1.5: a reference price incorporates procurement and carrying): the
    /// reported price, valued at its own cost of the payment good (`costs`), and the walk there
    /// and back and the trading spread over a load, at its walking pace off the trails. Each
    /// report is believed by its weight, what it does not believe falling back on its own cost
    /// (`own`), and the best is taken. None without a way to fetch it (content with no `fetch`
    /// activity, or every seller beyond its walk) or a report of the good.
    fn replacement_cost(
        &self,
        ctx: &Ctx,
        hh: &Household,
        g: usize,
        own: Option<f64>,
        costs: &[Option<f64>],
    ) -> Option<f64> {
        let fetch = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == crate::params::Behavior::Fetch)?;
        let home = hh.settlement?;
        let (params, goods) = (ctx.params, &ctx.catalog.goods);
        let pace = params.nav.tobler_ms(0.0) * params.nav.offtrail_factor;
        if pace <= 0.0 {
            return None;
        }
        let trade_h = f64::from(fetch.min_minutes) / 60.0;
        let (day, half) = (ctx.now.day_index(), params.reports.half_life_days);
        let tool = goods.get(g).is_some_and(|d| d.tool.is_some());
        let mut best: Option<f64> = None;
        for r in self
            .reports
            .of(hh.id)
            .iter()
            .filter(|r| usize::from(r.good) == g && r.market != home && r.units > 0.0)
        {
            let Some(pay) = costs.get(usize::from(r.payment)).copied().flatten() else {
                continue;
            };
            let at = if r.firm {
                self.firm(r.seller)
                    .and_then(|f| self.household(f.owner))
                    .map(|x| x.home)
            } else {
                self.household(r.seller).map(|x| x.home)
            };
            let Some(at) = at else {
                continue;
            };
            let walk_h = f64::from((at.0 - hh.home.0).hypot(at.1 - hh.home.1)) / pace / 3600.0;
            if walk_h * 60.0 > f64::from(fetch.max_walk_minutes) {
                continue;
            }
            let units = f64::from(r.units);
            let load = if tool {
                units
            } else {
                units.min(params.household.carry_kg)
            };
            if load <= 0.0 {
                continue;
            }
            // A duty its polity lays on what it would bring home from there (M7c slice BV).
            let duty = self.duty_rate(ctx, hh, g, r.market);
            let cost = f64::from(r.price) * pay * (1.0 + duty) + (2.0 * walk_h + trade_h) / load;
            let w = r.weight(day, half);
            let believed = match own {
                Some(o) => o - w * (o - cost).max(0.0),
                None => cost,
            };
            best = Some(best.map_or(believed, |b: f64| b.min(believed)));
        }
        best
    }

    /// On a month's first day, what the month before left on record per pair of settlements
    /// lived in (M5b slice AQ, ADR-0019 §7): for each good offered in both, the median of its
    /// sellers' asks in each, hours a unit, and what a unit fetched there that month.
    pub(super) fn record_convergence(&mut self, ctx: &Ctx) {
        let now = ctx.now;
        if now.date().day != 1 {
            return;
        }
        let Some(month) = crate::market::month_of(now).checked_sub(1) else {
            return;
        };
        let lived: Vec<PermanentId> = ctx
            .land
            .settlements
            .iter()
            .filter(|s| self.residents(s.id) > 0)
            .map(|s| s.id)
            .collect();
        if lived.len() < 2 {
            return;
        }
        // Each seller's ask of each good it offers, by settlement and good.
        let mut asks: BTreeMap<(PermanentId, u16), Vec<f64>> = BTreeMap::new();
        let mut note = |s: PermanentId, offers: &[Offer]| {
            let mut seen: Vec<u16> = Vec::new();
            for o in offers {
                if !seen.contains(&o.good) {
                    seen.push(o.good);
                    asks.entry((s, o.good))
                        .or_default()
                        .push(f64::from(o.ask_h));
                }
            }
        };
        for (_, h) in self.households.iter() {
            if let Some(s) = h.settlement.filter(|_| !h.members.is_empty()) {
                note(s, &h.offers);
            }
        }
        for f in self.firms.iter().filter(|f| f.is_open()) {
            if let Some(s) = f.settlement {
                note(s, &f.offers);
            }
        }
        let median: BTreeMap<(PermanentId, u16), f64> = asks
            .into_iter()
            .map(|(k, mut v)| (k, crate::convergence::median(&mut v)))
            .collect();
        let paid = |s: PermanentId, g: u16| -> f32 {
            self.market(s)
                .and_then(|m| {
                    m.history
                        .iter()
                        .find(|h| h.month == month && h.good == g && h.units > 0.0)
                })
                .map_or(-1.0, |h| h.paid_h / h.units)
        };
        let mut found = Vec::new();
        for (i, &a) in lived.iter().enumerate() {
            for &b in &lived[i + 1..] {
                let (a, b) = (a.min(b), a.max(b));
                let gaps: Vec<crate::convergence::GoodGap> = median
                    .iter()
                    .filter(|((s, _), _)| *s == a)
                    .filter_map(|(&(_, g), &ask_a)| {
                        let ask_b = *median.get(&(b, g))?;
                        Some(crate::convergence::GoodGap {
                            good: g,
                            ask_h: [ask_a as f32, ask_b as f32],
                            paid_h: [paid(a, g), paid(b, g)],
                        })
                    })
                    .filter(|g| g.ask_h.iter().all(|x| x.is_finite() && *x > 0.0))
                    .collect();
                if !gaps.is_empty() {
                    found.push(((month, a, b), gaps));
                }
            }
        }
        self.convergence.gaps.extend(found);
    }

    /// Household `household` saw at the door of `seller`, of `market`, on `day` what it offers
    /// now: a report of each offer, and that none is left of what it no longer offers (ADR-0019
    /// §1–§2).
    fn note_seen(
        &mut self,
        household: PermanentId,
        market: PermanentId,
        seller: PermanentId,
        day: i64,
    ) {
        let (offers, firm) = if let Some(f) = self.firm(seller) {
            (f.offers.clone(), true)
        } else if let Some(x) = self.household(seller) {
            (x.offers.clone(), false)
        } else {
            return;
        };
        let stale: Vec<(u16, u16)> = self
            .reports
            .of(household)
            .iter()
            .filter(|r| r.market == market && r.seller == seller)
            .map(|r| (r.good, r.payment))
            .filter(|&(g, p)| !offers.iter().any(|o| o.good == g && o.payment == p))
            .collect();
        for key in stale {
            self.reports.sold_out(household, market, seller, key, day);
        }
        for o in offers {
            self.reports.note(
                household,
                PriceReport {
                    market,
                    seller,
                    firm,
                    good: o.good,
                    payment: o.payment,
                    price: o.price,
                    ask_h: o.ask_h,
                    units: o.units,
                    day,
                    how: ReportHow::Seen,
                    from: None,
                },
            );
        }
    }

    /// Why a buyer of household `buyer` who came to `seller`, of `market`, by its reports could
    /// buy nothing, judged by the goods it holds reports of that the household still wants, as
    /// [`Population::find_deal`] wants them (a tool it needs, food it is short of): none of them
    /// now; none of them offered any more, or a tool by less than a whole one; offered only for
    /// what it has none of to spare; or on terms that no longer serve it.
    fn missed_why(
        &self,
        ctx: &Ctx,
        buyer: PermanentId,
        market: PermanentId,
        seller: PermanentId,
        stores: &[f64],
    ) -> Missed {
        let goods = &ctx.catalog.goods;
        let offers = if let Some(f) = self.firm(seller) {
            f.offers.as_slice()
        } else {
            self.household(seller)
                .map_or(&[][..], |x| x.offers.as_slice())
        };
        let Some(hh) = self.household(buyer) else {
            return Missed::NoLonger;
        };
        let holding = self.holding_of(ctx, hh, stores);
        let (short_kcal, _) = food_short(goods, &holding.keep, stores);
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        let wanted = |g: usize| {
            goods.get(g).is_some_and(|d| {
                if d.tool.is_some() {
                    let keep = (holding.keep[g] - make::SPARE_TOOL).max(0.0);
                    make::tool_need(keep, held(g)) > 0.0
                } else {
                    d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 && short_kcal > 0.0
                }
            })
        };
        let reported: Vec<u16> = self
            .reports
            .of(buyer)
            .iter()
            .filter(|r| r.market == market && r.seller == seller && wanted(usize::from(r.good)))
            .map(|r| r.good)
            .collect();
        if reported.is_empty() {
            return Missed::NoLonger;
        }
        let still: Vec<&Offer> = offers
            .iter()
            .filter(|o| {
                let g = usize::from(o.good);
                let least = if goods[g].tool.is_some() {
                    MIN_OFFER_TOOL
                } else {
                    1.0
                };
                reported.contains(&o.good) && f64::from(o.units) + 1e-6 >= least
            })
            .collect();
        if still.is_empty() {
            return Missed::SoldOut;
        }
        let can_pay = still.iter().any(|o| {
            let p = usize::from(o.payment);
            let spare = held(p) - holding.keep.get(p).copied().unwrap_or(0.0);
            spare > 0.0 && spare + 1e-9 >= f64::from(o.price) * f64::from(o.units).min(1.0)
        });
        if can_pay {
            Missed::Terms
        } else {
            Missed::Payment
        }
    }

    /// The best purchase a person of household `hh` could go and make in another settlement it
    /// holds a report of (M5b slice AP, ADR-0019 §2): the reported offers weighed as
    /// [`Population::find_deal`] weighs those at home, each believed by its report's weight, the
    /// walk from the household's hearth (`reach`) there and back included. Sellers who no longer
    /// live where they were reported to are not sought.
    pub(crate) fn best_fetch(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        reach: Option<&TravelField>,
    ) -> Option<TradeOption> {
        let reach = reach?;
        let home = hh.settlement?;
        let reports = self.reports.of(hh.id);
        if reports.is_empty() {
            return None;
        }
        let (day, rp) = (ctx.now.day_index(), &ctx.params.reports);
        let mut sellers: Vec<Seller> = Vec::new();
        for r in reports.iter().filter(|r| r.market != home && r.units > 0.0) {
            let Some(at) = self.seller_home(r.seller, r.firm, r.market) else {
                continue;
            };
            let Some(secs) = reach.seconds_to(cell_of(ctx.map, at)) else {
                continue;
            };
            sellers.push(Seller {
                id: r.seller,
                at,
                offers: std::borrow::Cow::Owned(vec![Offer {
                    good: r.good,
                    payment: r.payment,
                    price: r.price,
                    units: r.units,
                    ask_h: r.ask_h,
                }]),
                firm: r.firm,
                walk_min: Some(f64::from(secs) / 60.0),
                weight: r.weight(day, rp.half_life_days),
            });
        }
        let d = self.best_deal(ctx, hh, stores, sellers, &|_| None)?;
        Some(TradeOption {
            seller: d.seller,
            firm: d.firm,
            walk_min: d.walk_min,
            at: d.at,
            good: d.good,
            worth: d.worth,
            by_boat: None,
            by_cart: None,
        })
    }

    /// Where `seller`, a household or workshop (`firm`) reported in `market`'s, now sells: its
    /// home, or its owners'. None if it no longer lives there, or the workshop closed.
    fn seller_home(
        &self,
        seller: PermanentId,
        firm: bool,
        market: PermanentId,
    ) -> Option<(f32, f32)> {
        if firm {
            self.firm(seller)
                .filter(|f| f.is_open() && f.settlement == Some(market))
                .and_then(|f| self.household(f.owner))
                .map(|x| x.home)
        } else {
            self.household(seller)
                .filter(|x| x.settlement == Some(market))
                .map(|x| x.home)
        }
    }

    /// The errand household `hh` would run, weighed at its review with its `stores`, own `costs`
    /// and `holding`, and what it now `offered` (M5b slice AQ, ADR-0019 §6; research 08-12 §1.6:
    /// a trader weighs depth as well as price): of the goods it holds reports of in other
    /// settlements' markets, and is not short of itself, those its own market wants (buyers found
    /// none, or one sold lately) for more than a unit would cost it, as making to sell is weighed
    /// ([`Population::for_sale`]). A unit's cost there is the reported price in what it pays
    /// with, valued at its own cost, with the walk there and back (from the distance at its pace
    /// off the trails, a belief) and the trading spread over the units, and its usual margin.
    /// The units are the least of a load, the units reported, those it can pay for out of what
    /// it can spare (and carry) and those it expects to sell at home over a review: its market's
    /// sales over one and the demand nobody met, less what is on offer there already, its own
    /// included. The best is the one it keeps the largest share of, the walk paid for; the trip
    /// is worth that share without the walk, which the decision weighs as on any trip. None
    /// without a `fetch` activity or a report.
    fn plan_errand(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        costs: &[Option<f64>],
        holding: &Holding,
        offered: &[(usize, f64)],
    ) -> (Option<Errand>, Vec<VehicleWish>) {
        let reports = self.reports.of(hh.id);
        if reports.is_empty() {
            return (None, Vec::new());
        }
        let Some(fetch) = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == crate::params::Behavior::Fetch)
        else {
            return (None, Vec::new());
        };
        let Some((home, market)) = hh.settlement.and_then(|s| Some((s, self.market(s)?))) else {
            return (None, Vec::new());
        };
        let (params, goods, day) = (ctx.params, &ctx.catalog.goods, ctx.now.day_index());
        let mp = &params.market;
        let pace = params.nav.tobler_ms(0.0) * params.nav.offtrail_factor;
        if pace <= 0.0 {
            return (None, Vec::new());
        }
        let trade_h = f64::from(fetch.min_minutes) / 60.0;
        let memory = mp.memory_days.max(1e-6);
        let fade = 0.5f64.powf((day - market.day).max(0) as f64 / memory);
        let over_review = 1.0 - 0.5f64.powf(f64::from(mp.review_days.max(1)) / memory);
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        let keep = |g: usize| holding.keep.get(g).copied().unwrap_or(0.0);
        let carry = params.household.carry_kg;
        // What is on offer at home of each good, worked out once a report is worth it.
        let on_offer = std::cell::OnceCell::new();
        let on_offer = || {
            on_offer.get_or_init(|| {
                let mut units = vec![0.0; goods.len()];
                for &(g, u) in offered {
                    units[g] += u;
                }
                let mut add = |offers: &[Offer]| {
                    let mut seen: Vec<(u16, f32)> = Vec::new();
                    for o in offers {
                        match seen.iter_mut().find(|(g, _)| *g == o.good) {
                            Some(x) => x.1 = x.1.max(o.units),
                            None => seen.push((o.good, o.units)),
                        }
                    }
                    for (g, u) in seen {
                        if let Some(x) = units.get_mut(usize::from(g)) {
                            *x += f64::from(u);
                        }
                    }
                };
                for x in self.households_in(home) {
                    if x.id != hh.id {
                        add(&x.offers);
                    }
                }
                for f in self.firms.iter() {
                    if f.is_open() && f.settlement == Some(home) {
                        add(&f.offers);
                    }
                }
                units
            })
        };
        let (mut best, mut best_net): (Option<Errand>, Option<f64>) = (None, None);
        // A boat it has none of but could make (M8b slice CO, third part), and the most hours
        // one would keep it beyond the walk on any errand.
        let could = self.boat_it_could_make(ctx, hh, stores);
        let mut gain_h = 0.0f64;
        // And a cart it could make at its door (M8b slice CP, third part), likewise.
        let could_cart = self.cart_it_could_make(ctx, hh, stores);
        let mut cart_gain_h = 0.0f64;
        for r in reports.iter().filter(|r| r.market != home && r.units > 0.0) {
            let (g, p) = (usize::from(r.good), usize::from(r.payment));
            let (Some(d), Some(pd)) = (goods.get(g), goods.get(p)) else {
                continue;
            };
            // Something it can sell, and is not short of itself (that it buys for itself).
            if !can_offer(d) || held(g) < keep(g) {
                continue;
            }
            let Some(home_h) = market.unmet_worth(g).or_else(|| market.price_h(g)) else {
                continue;
            };
            let Some(pay_cost) = costs.get(p).copied().flatten() else {
                continue;
            };
            let price = f64::from(r.price);
            if !(price > 0.0 && home_h > 0.0) {
                continue;
            }
            let Some(at) = self.seller_home(r.seller, r.firm, r.market) else {
                continue;
            };
            let walk_h = f64::from((at.0 - hh.home.0).hypot(at.1 - hh.home.1)) / pace / 3600.0;
            let sold = market.sold.get(g).copied().unwrap_or(0.0);
            let unmet = market.unmet.get(g).copied().unwrap_or(0.0);
            let depth = (sold * over_review + unmet) * fade - on_offer()[g];
            let tool = d.tool.is_some();
            let spare = held(p) - keep(p);
            let least = if tool { MIN_OFFER_TOOL } else { MIN_OFFER_KG };
            let pay_h = price * pay_cost * holding.want.get(p).copied().unwrap_or(0.0);
            // The units a load of `load` kilograms brings, what it keeps of each (the net) and
            // what the trip is worth (the share), with `travel_h` hours of travel and
            // `per_unit_h` hours handling each unit.
            let priced = |load: f64, travel_h: f64, per_unit_h: f64| {
                let mut units = f64::from(r.units).min(depth).min(spare / price);
                if !tool {
                    units = units.min(load);
                }
                if pd.tool.is_none() {
                    units = units.min(load / price);
                }
                if units.is_nan()
                    || units < least
                    || (pd.tool.is_some() && units * price < MIN_OFFER_TOOL)
                {
                    return None;
                }
                // Worth the walk: what it keeps of a unit when the walk is paid for too.
                let cost = (pay_h + per_unit_h + (travel_h + trade_h) / units) * (1.0 + mp.margin);
                let net = 1.0 - cost / home_h;
                if net.is_nan() || net <= 0.0 {
                    return None;
                }
                // What the trip is worth, as making to sell is; the walk is weighed as on any
                // trip (as a purchase for itself is, ADR-0019 §2).
                let share = (1.0
                    - (pay_h + per_unit_h + trade_h / units) * (1.0 + mp.margin) / home_h)
                    * units.min(1.0);
                Some((units, net, share))
            };
            let on_foot = (walk_h * 60.0 <= f64::from(fetch.max_walk_minutes))
                .then(|| priced(carry, 2.0 * walk_h, 0.0))
                .flatten()
                .map(|o| (o, None, None));
            // By the household's boat (M8b slice CO, ADR-0029 §3): hours against hours, the
            // whole chain (to the boat, the water both ways, from the landing to the door,
            // handling at each end) against the walk, a boat's load against a hand load.
            // Each kilogram is loaded and unloaded, and carried a hand load at a time between
            // the door and the boat at each end, there and back.
            let afloat = |w: &BoatWay| {
                let handle_h = 1.0 / (w.handling_kg_min * 60.0);
                let load_h = handle_h + 2.0 * w.far_walk_h / carry;
                let unload_h = handle_h + 2.0 * w.home_walk_h / carry;
                (w.travel_h * 30.0 <= f64::from(fetch.max_walk_minutes))
                    .then(|| priced(w.payload_kg, w.travel_h, load_h + unload_h))
                    .flatten()
                    .map(|o| (o, load_h, unload_h))
            };
            let by_boat = (!tool)
                .then(|| self.boat_way(ctx, hh, at, pace))
                .flatten()
                .and_then(|(boat, w)| {
                    afloat(&w).map(|((units, net, share), load_h, unload_h)| {
                        let way = crate::reports::ByBoat {
                            boat,
                            from: w.from,
                            to: w.to,
                            travel_min: (w.travel_h * 60.0) as f32,
                            load_min: (units * load_h * 60.0) as f32,
                            unload_min: (units * unload_h * 60.0) as f32,
                        };
                        ((units, net, share), Some(way), None)
                    })
                });
            // By the household's cart (M8b slice CP, ADR-0029 §3): the cart pulled empty to the
            // door and loaded back, pushed between each door and the ground it goes on, and
            // each kilogram loaded at the seller's door and unloaded at home.
            let wheeled = |w: &CartWay| {
                let handle_h = 1.0 / (w.handling_kg_min * 60.0);
                (w.travel_h * 30.0 <= f64::from(fetch.max_walk_minutes))
                    .then(|| priced(w.payload_kg, w.travel_h, 2.0 * handle_h))
                    .flatten()
                    .map(|o| (o, handle_h))
            };
            let by_cart = (!tool)
                .then(|| self.cart_way(ctx, hh, at))
                .flatten()
                .and_then(|(cart, w)| {
                    wheeled(&w).map(|((units, net, share), handle_h)| {
                        let way = crate::reports::ByCart {
                            cart,
                            travel_min: (w.travel_h * 60.0) as f32,
                            load_min: (units * handle_h * 60.0) as f32,
                            unload_min: (units * handle_h * 60.0) as f32,
                        };
                        ((units, net, share), None, Some(way))
                    })
                });
            // By a boat it could make: the hours it would keep beyond what the walk keeps.
            if let Some(c) = could.as_ref().filter(|_| !tool)
                && let Some(((units, net, _), _, _)) = ctx
                    .catalog
                    .vehicles
                    .get(c.vehicle)
                    .and_then(|v| self.way_by_water(ctx, hh, v, c.bank, at, pace))
                    .and_then(|w| afloat(&w))
            {
                let walked = on_foot.map_or(0.0, |((u, n, _), _, _)| u * n);
                gain_h = gain_h.max((units * net - walked) * home_h);
            }
            // By a cart it could make: likewise.
            if let Some(v) = could_cart.filter(|_| !tool)
                && let Some(((units, net, _), _)) = ctx
                    .catalog
                    .vehicles
                    .get(v)
                    .and_then(|v| self.way_by_wheels(ctx, hh, v, at))
                    .and_then(|w| wheeled(&w))
            {
                let walked = on_foot.map_or(0.0, |((u, n, _), _, _)| u * n);
                cart_gain_h = cart_gain_h.max((units * net - walked) * home_h);
            }
            // The way it keeps the largest share by; the walk, then the boat, on a tie.
            let chosen = [on_foot, by_boat, by_cart]
                .into_iter()
                .flatten()
                .reduce(|a, b| if b.0.1 > a.0.1 + 1e-9 { b } else { a });
            let Some(((units, net, share), way, cart_way)) = chosen else {
                continue;
            };
            if best_net.is_none_or(|b| net > b + 1e-9) {
                best_net = Some(net);
                best = Some(Errand {
                    market: r.market,
                    seller: r.seller,
                    firm: r.firm,
                    good: r.good,
                    payment: r.payment,
                    units: units as f32,
                    home_h: home_h as f32,
                    share: share as f32,
                    day,
                    by_boat: way,
                    by_cart: cart_way,
                });
            }
        }
        let door = cell_of(ctx.map, hh.home) as u32;
        let wishes = could
            .filter(|_| gain_h > 0.0)
            .map(|c| VehicleWish {
                vehicle: c.vehicle,
                at: c.bank,
                gain_h,
            })
            .into_iter()
            .chain(
                could_cart
                    .filter(|_| cart_gain_h > 0.0)
                    .map(|vehicle| VehicleWish {
                        vehicle,
                        at: door,
                        gain_h: cart_gain_h,
                    }),
            )
            .collect();
        (best, wishes)
    }

    /// The way by household `hh`'s boat to the bank nearest a seller's door at `at`, open both
    /// ways by water today (M8b slice CO): None without a boat, a bank within
    /// [`LANDING_WALK_M`] of the door, or a way there and back. Walks are at `pace`.
    fn boat_way(
        &self,
        ctx: &Ctx,
        hh: &Household,
        at: (f32, f32),
        pace: f64,
    ) -> Option<(PermanentId, BoatWay)> {
        let b = ctx.land.boats.made_of(hh.id).next()?;
        let v = ctx.catalog.vehicles.get(usize::from(b.vehicle))?;
        self.way_by_water(ctx, hh, v, b.at, at, pace)
            .map(|w| (b.id, w))
    }

    /// The way by household `hh`'s cart, standing at its door, to a seller's door at `at` and
    /// back (M8b slice CP): pulled empty there and with a full load back, by today's quickest
    /// ways for its wheels; None without a cart at the door, or a way either way.
    fn cart_way(
        &self,
        ctx: &Ctx,
        hh: &Household,
        at: (f32, f32),
    ) -> Option<(PermanentId, CartWay)> {
        let door = cell_of(ctx.map, hh.home) as u32;
        let c = ctx.land.carts.made_of(hh.id).find(|c| c.at == door)?;
        let v = ctx.catalog.vehicles.get(usize::from(c.vehicle))?;
        self.way_by_wheels(ctx, hh, v, at).map(|w| (c.id, w))
    }

    /// The way by a cart of vehicle `v` standing at household `hh`'s door, as
    /// [`Population::cart_way`] finds it.
    fn way_by_wheels(
        &self,
        ctx: &Ctx,
        hh: &Household,
        v: &crate::params::VehicleDef,
        at: (f32, f32),
    ) -> Option<CartWay> {
        if v.mode != crate::params::VehicleMode::Cart {
            return None;
        }
        let walk = ctx.params.nav.tobler_ms(0.0);
        let (there, there_legs) = ctx
            .land
            .cart_way(ctx.map, &v.wheels(0.0, walk), hh.home, at)?;
        let (back, back_legs) =
            ctx.land
                .cart_way(ctx.map, &v.wheels(v.payload_kg, walk), at, hh.home)?;
        let travel_h = (there.seconds + there_legs + back.seconds + back_legs) / 3600.0;
        Some(CartWay {
            travel_h,
            payload_kg: v.payload_kg,
            handling_kg_min: v.handling_kg_min,
        })
    }

    /// The way by a boat of vehicle `v` drawn up at bank `from` to the bank nearest a seller's
    /// door at `at`, as [`Population::boat_way`] finds it.
    fn way_by_water(
        &self,
        ctx: &Ctx,
        hh: &Household,
        v: &crate::params::VehicleDef,
        from: u32,
        at: (f32, f32),
        pace: f64,
    ) -> Option<BoatWay> {
        let hull = v.hull();
        let rivers = &ctx.land.rivers;
        let (hyd, q) = (&ctx.land.floodplain.reaches, &ctx.land.reach_discharges);
        // Where the water carries a boat today, nearest the door.
        let to = rivers.nearest_bank(at, LANDING_WALK_M, |c| {
            c != from && rivers.floats(ctx.map, c, hyd, q, hull)
        })?;
        let home_cells = rivers.boarding(ctx.map, from);
        let far_cells = rivers.boarding(ctx.map, to);
        let there = rivers.route(hyd, q, hull, &home_cells, &far_cells)?;
        let back = rivers.route(hyd, q, hull, &far_cells, &home_cells)?;
        let walk_h =
            |a: (f32, f32), z: (f32, f32)| f64::from((a.0 - z.0).hypot(a.1 - z.1)) / pace / 3600.0;
        let (home_walk_h, far_walk_h) = (
            walk_h(hh.home, rivers.centre(from)),
            walk_h(rivers.centre(to), at),
        );
        let travel_h = 2.0 * (home_walk_h + far_walk_h) + (there.seconds + back.seconds) / 3600.0;
        Some(BoatWay {
            from,
            to,
            travel_h,
            home_walk_h,
            far_walk_h,
            payload_kg: v.payload_kg,
            handling_kg_min: v.handling_kg_min,
        })
    }

    /// The trip on household `hh`'s errand, for the decision (M5b slice AQ, ADR-0019 §6): planned
    /// at its last review, and to a seller who still sells where it was reported to, the walk
    /// from the household's hearth (`reach`).
    pub(crate) fn errand_trip(
        &self,
        ctx: &Ctx,
        hh: &Household,
        reach: Option<&TravelField>,
    ) -> Option<TradeOption> {
        let e = self.reports.errands.get(&hh.id)?;
        let reach = reach?;
        let review_days = i64::from(ctx.params.market.review_days.max(1));
        if hh.settlement == Some(e.market) || ctx.now.day_index() - e.day >= review_days {
            return None;
        }
        let at = self.seller_home(e.seller, e.firm, e.market)?;
        // By boat (M8b slice CO): only while the boat is still drawn up where it was planned
        // from; its travel as planned, half of it each way.
        let (walk_min, by_boat) = match e.by_boat {
            Some(b) => {
                ctx.land
                    .boats
                    .made_of(hh.id)
                    .find(|x| x.id == b.boat && x.at == b.from)?;
                let rivers = &ctx.land.rivers;
                let legs = crate::decide::BoatLegs {
                    from: b.from,
                    to: b.to,
                    from_at: rivers.centre(b.from),
                    to_at: rivers.centre(b.to),
                    load_min: b.load_min.ceil().max(1.0) as u32,
                    unload_min: b.unload_min.ceil().max(1.0) as u32,
                };
                (f64::from(b.travel_min) / 2.0, Some(legs))
            }
            None => (
                f64::from(reach.seconds_to(cell_of(ctx.map, at))?) / 60.0,
                None,
            ),
        };
        // By cart (M8b slice CP): only while the cart still stands at the door; its travel as
        // planned, half of it each way.
        let (walk_min, by_cart) = match e.by_cart {
            Some(c) => {
                let door = cell_of(ctx.map, hh.home) as u32;
                ctx.land
                    .carts
                    .made_of(hh.id)
                    .find(|x| x.id == c.cart && x.at == door)?;
                let legs = crate::decide::CartLegs {
                    load_min: c.load_min.ceil().max(1.0) as u32,
                    unload_min: c.unload_min.ceil().max(1.0) as u32,
                };
                (f64::from(c.travel_min) / 2.0, Some(legs))
            }
            None => (walk_min, None),
        };
        Some(TradeOption {
            seller: e.seller,
            firm: e.firm,
            walk_min,
            at,
            good: usize::from(e.good),
            worth: TradeWorth::Sale {
                share: f64::from(e.share),
            },
            by_boat,
            by_cart,
        })
    }

    /// What a buyer of household `hh`, with its `stores`, on its errand to the door of `seller`,
    /// buys there to sell at home (M5b slice AQ, ADR-0019 §6): the good it came for on the terms
    /// that cost it least, as much as it meant to, the seller offers, it can pay out of what it
    /// can spare and one person carries, so long as a unit still costs it less, with its usual
    /// margin, than it expects one to fetch at home (the walk is behind it). Or why it bought
    /// nothing. None if it is on no errand to this seller.
    fn resale_at(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        seller: PermanentId,
    ) -> Option<Result<Deal, Missed>> {
        let e = *self
            .reports
            .errands
            .get(&hh.id)
            .filter(|e| e.seller == seller)?;
        let (params, goods) = (ctx.params, &ctx.catalog.goods);
        let g = usize::from(e.good);
        let d = goods.get(g)?;
        let (offers, at) = if e.firm {
            let f = self.firm(seller)?;
            (f.offers.as_slice(), self.household(f.owner)?.home)
        } else {
            let x = self.household(seller)?;
            (x.offers.as_slice(), x.home)
        };
        let tool = d.tool.is_some();
        let least = if tool { MIN_OFFER_TOOL } else { MIN_OFFER_KG };
        // A boat's or cart's load on an errand by boat or cart (M8b slices CO and CP), a hand
        // load otherwise.
        let vehicle = e
            .by_boat
            .and_then(|b| ctx.land.boats.list.iter().find(|x| x.id == b.boat))
            .or_else(|| {
                e.by_cart
                    .and_then(|c| ctx.land.carts.list.iter().find(|x| x.id == c.cart))
            });
        let carry = vehicle
            .and_then(|b| ctx.catalog.vehicles.get(usize::from(b.vehicle)))
            .map_or(params.household.carry_kg, |v| v.payload_kg);
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        let mut found = false;
        let mut payable = false;
        let mut best: Option<Deal> = None;
        let mut known: Option<(Vec<Option<f64>>, Holding)> = None;
        for o in offers
            .iter()
            .filter(|o| o.good == e.good && f64::from(o.units) + 1e-6 >= least)
        {
            found = true;
            let p = usize::from(o.payment);
            let (Some(pd), price) = (goods.get(p), f64::from(o.price)) else {
                continue;
            };
            if price.is_nan() || price <= 0.0 {
                continue;
            }
            let (costs, holding) = known.get_or_insert_with(|| {
                (self.own_costs_of(ctx, hh), self.holding_of(ctx, hh, stores))
            });
            let spare = held(p) - holding.keep.get(p).copied().unwrap_or(0.0);
            let mut units = f64::from(e.units)
                .min(f64::from(o.units))
                .min(spare / price);
            if !tool {
                units = units.min(carry);
            }
            if pd.tool.is_none() {
                units = units.min(carry / price);
            }
            if units.is_nan()
                || units < least
                || (pd.tool.is_some() && units * price < MIN_OFFER_TOOL)
            {
                continue;
            }
            payable = true;
            let Some(pay_cost) = costs.get(p).copied().flatten() else {
                continue;
            };
            let pay_h = price * pay_cost * holding.want.get(p).copied().unwrap_or(0.0);
            if pay_h * (1.0 + params.market.margin) >= f64::from(e.home_h) {
                continue;
            }
            let gain = units * (f64::from(e.home_h) - pay_h);
            if best.is_none_or(|b| gain > b.saving_h + 1e-9) {
                best = Some(Deal {
                    seller,
                    firm: e.firm,
                    at,
                    walk_min: 0.0,
                    good: g,
                    units,
                    payment: p,
                    paid: price * units,
                    ask_h: f64::from(o.ask_h),
                    saving_h: gain,
                    worth: TradeWorth::Sale {
                        share: f64::from(e.share),
                    },
                });
            }
        }
        Some(match best {
            Some(d) => Ok(d),
            None if !found => Err(Missed::SoldOut),
            None if payable => Err(Missed::Terms),
            None => Err(Missed::Payment),
        })
    }

    /// For each tool household `hh` does not need itself: how much it is worth making to sell, 0–1
    /// (research 08-04 §1.2: produce for funded demand, not for sales already made). Others in
    /// its settlement want it: they found nobody offering it, or it sold lately and nobody offers
    /// any now; and what they would give for one (what those who found none would have given,
    /// or what one fetched lately) is more than the household's own cost and margin. It is
    /// worth the share of that the household keeps, times how much of it is wanted (up to one
    /// tool). Nothing while the household, or its workshop, still holds one to sell.
    pub(crate) fn for_sale(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
        tool_need: &[f64],
    ) -> Vec<f64> {
        let goods = &ctx.catalog.goods;
        let mut out = vec![0.0; goods.len()];
        let Some(settlement) = hh.settlement else {
            return out;
        };
        let Some(market) = self.market(settlement) else {
            return out;
        };
        // What is wanted of each tool: wants nobody met, and what sold lately when none is on
        // offer now.
        let on_offer = |t: usize| {
            self.households_in(settlement)
                .flat_map(|x| x.offers.iter())
                .chain(
                    self.firms
                        .iter()
                        .filter(|f| f.is_open() && f.settlement == Some(settlement))
                        .flat_map(|f| f.offers.iter()),
                )
                .any(|o| usize::from(o.good) == t)
        };
        // A tool that serves for another answers the demand for that one (M7d slice CE).
        let wanted: Vec<(usize, f64, f64)> = goods
            .iter()
            .enumerate()
            .filter(|(t, d)| {
                d.tool.is_some()
                    && tool_need
                        .get(make::served(goods, *t))
                        .copied()
                        .unwrap_or(0.0)
                        <= 0.0
            })
            .filter_map(|(t, _)| {
                let u = make::served(goods, t);
                let unmet = market.unmet.get(u).copied().unwrap_or(0.0);
                let sold = market.sold.get(t).copied().unwrap_or(0.0);
                let (units, worth) = match market.unmet_worth(u) {
                    Some(worth) => (unmet, worth),
                    None if sold > 0.1 && !on_offer(t) => (sold, market.price_h(t)?),
                    None => return None,
                };
                Some((t, units, worth))
            })
            .collect();
        if wanted.is_empty() {
            return out;
        }
        let costs = self.own_costs_of(ctx, hh);
        let holding = self.holding_of(ctx, hh, stores);
        let margin = ctx.params.market.margin;
        for (t, units, worth) in wanted {
            let Some(cost) = costs[t] else {
                continue;
            };
            // What it holds beyond its keep, its workshop's stock included.
            let spare = stores.get(t).copied().unwrap_or(0.0) - holding.keep[t]
                + self.workshop_stock(hh, t);
            if spare >= 1.0 {
                continue;
            }
            let share = 1.0 - cost * (1.0 + margin) / worth.max(1e-9);
            if share > 0.0 {
                out[t] = share * units.min(1.0);
            }
        }
        out
    }

    pub(super) fn household_mut_by_id(&mut self, id: PermanentId) -> Option<&mut Household> {
        let hd = *self.hh_index.get(&id)?;
        self.households.get_mut(hd)
    }
}

/// Food a household with `stores` is short of, kcal, and what it keeps of food, kcal: what it
/// keeps (`keep`, by good) less what it has, by energy.
fn food_short(goods: &[GoodDef], keep: &[f64], stores: &[f64]) -> (f64, f64) {
    let food = stock_kcal(stores, goods);
    let food_keep: f64 = goods
        .iter()
        .enumerate()
        .filter(|(_, d)| d.purpose == GoodUse::Food && !d.kept_back())
        .map(|(g, d)| keep[g].max(0.0) * d.kcal_per_kg)
        .sum::<f64>()
        .max(food);
    ((food_keep - food).max(0.0), food_keep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ask_moves_toward_its_anchor_by_at_most_the_largest_change() {
        // Far below the anchor: up by the cap.
        let up = reviewed_ask(1.0, 2.0, false, false, 0.05);
        assert!((up - 0.05f64.exp()).abs() < 1e-12);
        // Close to it: half the way, in logs.
        let near = reviewed_ask(1.0, 1.04, false, false, 0.05);
        assert!((near - 1.04f64.sqrt()).abs() < 1e-12);
        // At the anchor, demand nobody met raises it and an unsold offer lowers it.
        assert!(reviewed_ask(1.0, 1.0, true, false, 0.05) > 1.0);
        assert!(reviewed_ask(1.0, 1.0, false, true, 0.05) < 1.0);
        assert_eq!(reviewed_ask(1.0, 1.0, true, true, 0.05), 1.0);
    }

    #[test]
    fn a_seller_asks_less_for_food_the_more_of_it_it_can_spare() {
        // Nothing to spare beyond its needs: its cost and margin.
        assert_eq!(stock_anchor(2.0, 0.0, 0.5), 2.0);
        // Half a year's need to spare, and a whole year's: lower, and lower still...
        let half = stock_anchor(2.0, 0.5, 0.5);
        let year = stock_anchor(2.0, 1.0, 0.5);
        assert!((half - 2.0 * (-0.25f64).exp()).abs() < 1e-12);
        assert!((year - 2.0 * (-0.5f64).exp()).abs() < 1e-12);
        // ...but no lower for more than a year's.
        assert_eq!(stock_anchor(2.0, 3.0, 0.5), year);
        assert_eq!(stock_anchor(2.0, -1.0, 0.5), 2.0);
        // With no response, the stock does not move it.
        assert_eq!(stock_anchor(2.0, 1.0, 0.0), 2.0);
    }
}
