//! The mint (M7d slice CE2b, ADR-0027 §6; research 08-06 §1.5-1.6). A polity's law may name one
//! who strikes its coin. Each midnight, households of the polity bring the metal they hold beyond
//! what they keep, with the fuel to strike it, when the pieces they would get back, less the
//! law's fee, are worth more to them than the metal and fuel they give: bullion is given up only
//! when minted money's advantages exceed its costs. The metal waits in the polity's store, counted
//! as theirs. The minter strikes it as their office asks, working the mint's recipe at home: the
//! pieces of the day's issue go to those who brought the metal, oldest first, less the fee, which
//! the polity's store keeps in coin.

use super::*;
use crate::coin::Brought;
use crate::polity::MintDef;

/// Least metal a household brings at once, kilograms.
const LEAST_BROUGHT_KG: f64 = 0.05;

impl Population {
    /// Polity `pi`'s mint: the one its law in force names, while they live, and the law's terms.
    pub(super) fn mint_of_polity(&self, ctx: &Ctx, pi: usize) -> Option<(PermanentId, MintDef)> {
        let (minter, law) = self.polities.get(pi)?.minter()?;
        let def = ctx.catalog.policies.get(usize::from(law.policy))?.mint?;
        self.person(minter)?;
        Some((minter, def))
    }

    /// The polity whose mint `person` keeps, and its terms.
    pub(super) fn mint_kept_by(&self, ctx: &Ctx, person: PermanentId) -> Option<(usize, MintDef)> {
        (0..self.polities.len()).find_map(|pi| {
            let (minter, def) = self.mint_of_polity(ctx, pi)?;
            (minter == person).then_some((pi, def))
        })
    }

    /// Metal brought to polity `polity`'s mint and not yet struck, kilograms.
    pub(super) fn brought_to(&self, polity: PermanentId) -> f64 {
        self.order
            .coinage
            .brought
            .iter()
            .filter(|b| b.polity == polity)
            .map(|b| b.metal_kg)
            .sum()
    }

    /// What `person`'s mint could strike now: its polity, its terms, and the kilograms of metal
    /// brought and waiting there, as far as the polity's store holds the fuel to strike them.
    pub(super) fn mint_work(
        &self,
        ctx: &Ctx,
        person: PermanentId,
    ) -> Option<(usize, MintDef, f64)> {
        let (pi, def) = self.mint_kept_by(ctx, person)?;
        let r = ctx.catalog.recipes.get(def.recipe)?;
        let &(metal, per) = r.inputs.first()?;
        let stores = &self.polities[pi].stores;
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        let mut kg = self.brought_to(self.polities[pi].id).min(held(metal));
        for &(g, amount) in r.inputs.iter().skip(1) {
            if amount > 0.0 {
                kg = kg.min(held(g) * per / amount);
            }
        }
        (kg > 1e-6).then_some((pi, def, kg))
    }

    /// Kilograms of metal paid by weight that each of `households` paid or was paid a year, as
    /// its markets remember them, faded as money's worth is (M7d slice CD): what a mint would
    /// save it weighing. In id order.
    pub(super) fn weighed_kg_year(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
    ) -> Vec<(PermanentId, f64)> {
        let goods = &ctx.catalog.goods;
        let mp = &ctx.params.market;
        let today = ctx.now.day_index();
        let mut out: Vec<(PermanentId, f64)> = households.iter().map(|&h| (h, 0.0)).collect();
        out.sort_unstable_by_key(|x| x.0);
        out.dedup_by_key(|x| x.0);
        let weighed = |g: u16| {
            goods
                .get(usize::from(g))
                .and_then(|d| d.money.as_ref())
                .is_some_and(|m| !m.coin)
        };
        let memory = mp.memory_days.max(1e-6);
        for t in self.markets.iter().flat_map(|m| m.recent.iter()) {
            if !weighed(t.payment) || t.paid <= 0.0 {
                continue;
            }
            let age = (today - t.at.day_index()).max(0) as f64;
            let kg = 0.5f64.powf(age / memory) * f64::from(t.paid) * std::f64::consts::LN_2
                / memory
                * 365.0;
            for h in [t.buyer, t.seller] {
                if let Ok(i) = out.binary_search_by_key(&h, |x| x.0) {
                    out[i].1 += kg;
                }
            }
        }
        out
    }

    /// Whether a household of `households` paid or was paid in weighed metal within its market's
    /// memory (the issue a mint answers).
    pub(super) fn weighed_lately(&self, ctx: &Ctx, households: &[PermanentId]) -> bool {
        self.weighed_kg_year(ctx, households)
            .iter()
            .any(|&(_, kg)| kg > 0.0)
    }

    /// The hours of weighing a kilogram of metal paid as coin of a mint with terms `def` saves
    /// over paying it by weight: the metal's hours a kilogram less the coin's hours a piece for
    /// the pieces in a kilogram.
    pub(super) fn weighing_saved_h(ctx: &Ctx, def: &MintDef) -> f64 {
        let goods = &ctx.catalog.goods;
        let Some(r) = ctx.catalog.recipes.get(def.recipe) else {
            return 0.0;
        };
        let verify = |g: Option<usize>| {
            g.and_then(|g| goods.get(g))
                .and_then(|d| d.money.as_ref())
                .map_or(0.0, |m| m.verify_h)
        };
        let metal = verify(r.inputs.first().map(|i| i.0));
        let coin = verify(r.outputs.first().map(|o| o.0));
        let mk = def.metal_kg();
        if mk <= 0.0 {
            return 0.0;
        }
        (metal - coin / mk).max(0.0)
    }

    /// The day's minting: each household of a polity with a mint weighs bringing it metal.
    pub(super) fn bring_to_mints(&mut self, ctx: &Ctx) {
        for pi in 0..self.polities.len() {
            let Some((_, def)) = self.mint_of_polity(ctx, pi) else {
                continue;
            };
            let polity = self.polities[pi].id;
            let mut households: Vec<PermanentId> = self
                .members_of_polity(pi, ctx.now, ctx.params)
                .into_iter()
                .map(|m| m.1)
                .collect();
            households.sort_unstable();
            households.dedup();
            for h in households {
                self.bring_metal(ctx, polity, h, &def);
            }
        }
    }

    /// For tests: the day's bringing of metal to mints.
    #[doc(hidden)]
    pub fn bring_to_mints_for_tests(&mut self, ctx: &Ctx) {
        self.bring_to_mints(ctx);
    }

    /// Household `h` of polity `polity` weighs bringing its mint, with terms `def`, the metal it
    /// holds beyond what it keeps, as far as it holds the fuel to strike it beyond what it keeps:
    /// it brings it when the pieces it would get back, less the fee, are worth more to it than
    /// the metal and fuel ([`Population::pay_worth`]; a piece is worth to it at least the metal in
    /// it, at its own cost of the metal).
    fn bring_metal(&mut self, ctx: &Ctx, polity: PermanentId, h: PermanentId, def: &MintDef) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(r) = ctx.catalog.recipes.get(def.recipe) else {
            return;
        };
        let (Some(&(metal, per)), Some(&(coin, _))) = (r.inputs.first(), r.outputs.first()) else {
            return;
        };
        let mk = def.metal_kg();
        if per <= 0.0 || mk <= 0.0 {
            return;
        }
        let Some(hh) = self.household(h) else {
            return;
        };
        let stores = stores_now(hh, now, params, goods);
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0).max(0.0);
        if held(metal) < LEAST_BROUGHT_KG {
            return;
        }
        let holding = self.holding_of(ctx, hh, &stores);
        let spare = |g: usize| (held(g) - holding.keep.get(g).copied().unwrap_or(0.0)).max(0.0);
        let mut kg = spare(metal);
        for &(g, amount) in r.inputs.iter().skip(1) {
            if amount > 0.0 {
                kg = kg.min(spare(g) * per / amount);
            }
        }
        if kg < LEAST_BROUGHT_KG {
            return;
        }
        let pieces = kg / mk * (1.0 - f64::from(def.fee));
        let costs = self.own_costs_of(ctx, hh);
        let own_metal = costs.get(metal).copied().flatten();
        let metal_worth = self
            .pay_worth(ctx, hh, metal, own_metal, held(metal), &costs)
            .unwrap_or(0.0);
        let piece_worth = self
            .pay_worth(
                ctx,
                hh,
                coin,
                own_metal.map(|c| c * mk),
                held(coin) + pieces,
                &costs,
            )
            .unwrap_or(0.0);
        let fuel: f64 = r
            .inputs
            .iter()
            .skip(1)
            .map(|&(g, amount)| costs.get(g).copied().flatten().unwrap_or(0.0) * amount / per * kg)
            .sum();
        if pieces * piece_worth <= kg * metal_worth + fuel {
            return;
        }
        let legs: Vec<Leg> = r
            .inputs
            .iter()
            .map(|&(g, amount)| Leg {
                from: h,
                to: polity,
                good: g,
                amount: amount / per * kg,
            })
            .collect();
        if self.transfer(now, params, goods, &legs, Channel::Mint) {
            self.order.coinage.brought.push(Brought {
                polity,
                household: h,
                metal_kg: kg,
            });
        }
    }

    /// `units` of the mint's recipe struck by the minter of polity `pi`, with terms `def`: the
    /// metal and fuel taken from the polity's store, and the pieces of the day's issue given to
    /// those who brought the metal, oldest first, less the law's fee, which the store keeps. The
    /// pieces of a household gone since it brought its metal are the store's.
    pub(super) fn strike(&mut self, ctx: &Ctx, pi: usize, def: &MintDef, units: f64) {
        let goods = &ctx.catalog.goods;
        let Some(r) = ctx.catalog.recipes.get(def.recipe) else {
            return;
        };
        let (Some(&(metal, per)), Some(&(coin, _))) = (r.inputs.first(), r.outputs.first()) else {
            return;
        };
        let mk = def.metal_kg();
        if per <= 0.0 || mk <= 0.0 || units.is_nan() || units <= 0.0 {
            return;
        }
        let polity = self.polities[pi].id;
        let waiting = self.brought_to(polity);
        let sheltered = self.polity_sheltered(pi);
        let p = &mut self.polities[pi];
        p.settle_stores(ctx.now, goods, sheltered);
        p.stores.resize(goods.len(), 0.0);
        let mut kg = (units * per).min(p.stores[metal]).min(waiting);
        for &(g, amount) in r.inputs.iter().skip(1) {
            if amount > 0.0 {
                kg = kg.min(p.stores[g] * per / amount);
            }
        }
        if kg <= 1e-9 {
            return;
        }
        for &(g, amount) in &r.inputs {
            p.stores[g] = (p.stores[g] - amount / per * kg).max(0.0);
        }
        // Whose metal was struck, oldest first.
        let mut left = kg;
        let mut owed: Vec<(PermanentId, f64)> = Vec::new();
        for b in self
            .order
            .coinage
            .brought
            .iter_mut()
            .filter(|b| b.polity == polity)
        {
            if left <= 1e-12 {
                break;
            }
            let take = b.metal_kg.min(left);
            b.metal_kg -= take;
            left -= take;
            owed.push((b.household, take));
        }
        self.order.coinage.brought.retain(|b| b.metal_kg > 1e-9);
        let day = ctx.now.day_index();
        let fee = f64::from(def.fee);
        let mut kept = 0.0;
        for (h, take) in owed {
            let pieces = take / mk;
            kept += pieces * fee;
            let theirs = pieces * (1.0 - fee);
            if !self.put_struck(coin, polity, h, day, theirs, def.mass_g, def.fineness) {
                kept += theirs;
            }
        }
        if kept > 0.0 {
            self.put_struck(coin, polity, polity, day, kept, def.mass_g, def.fineness);
        }
    }

    /// For tests: the minter of polity `polity` strikes `units` of its mint's recipe.
    #[doc(hidden)]
    pub fn strike_mint_for_tests(&mut self, ctx: &Ctx, polity: PermanentId, units: f64) -> bool {
        let Some(pi) = self.polities.iter().position(|p| p.id == polity) else {
            return false;
        };
        let Some((_, def)) = self.mint_of_polity(ctx, pi) else {
            return false;
        };
        self.strike(ctx, pi, &def, units);
        true
    }
}
