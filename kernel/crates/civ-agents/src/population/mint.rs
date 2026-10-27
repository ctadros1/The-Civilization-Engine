//! The mint (M7d slice CE2b, ADR-0027 §6; research 08-06 §1.5-1.6). A polity's law may name one
//! who strikes its coin. Each midnight, households of the polity bring the metal they hold beyond
//! what they keep, with the fuel to strike it, when the pieces they would get back, less the
//! law's fee, are worth more to them than the metal and fuel they give: bullion is given up only
//! when minted money's advantages exceed its costs. The metal waits in the polity's store, counted
//! as theirs. The minter strikes it as their office asks, working the mint's recipe at home: the
//! pieces of the day's issue go to those who brought the metal, oldest first, less the fee, which
//! the polity's store keeps in coin.
//!
//! A polity spends what its store holds of money first on its common works (M7d slice CE3a):
//! under a law of pay for common work, each hour a household gives to the polity's crossing,
//! well, bank or palisade is paid from the store, at the law's rate, as far as the store holds the
//! money the law names.

use super::*;
use crate::coin::Brought;
use crate::polity::{LawStatus, MintDef, PolicyKind};

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
        self.chronicle_struck(ctx, pi, def, kg / mk);
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

    /// The chronicle's words when polity `pi`'s mint strikes `pieces` under `def` and its last
    /// issue, if any, was of other terms: its first coin, or a lighter, heavier or baser one.
    fn chronicle_struck(&mut self, ctx: &Ctx, pi: usize, def: &MintDef, pieces: f64) {
        let polity = self.polities[pi].id;
        let last = self
            .order
            .coinage
            .cohorts
            .iter()
            .rev()
            .find(|c| c.issuer == polity)
            .map(|c| (c.mass_g, c.fineness));
        if last == Some((def.mass_g, def.fineness)) {
            return;
        }
        let settlement = self.polities[pi].settlement;
        let (place, name) = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map_or((None, String::new()), |s| {
                (Some(s.hearth_m), s.name.clone())
            });
        let keeper = self.polities[pi].minter().map(|(h, _)| h);
        let who = keeper.map_or("The mint".to_owned(), |h| {
            format!("{}, keeper of the mint,", self.name_of(h))
        });
        let metal = ctx
            .catalog
            .recipes
            .get(def.recipe)
            .and_then(|r| r.inputs.first())
            .and_then(|&(g, _)| ctx.catalog.goods.get(g))
            .map_or("metal".to_owned(), |g| g.name.to_lowercase());
        let terms = format!(
            "{:.0} g pieces of {metal}{}",
            def.mass_g,
            if def.fineness < 1.0 {
                format!(", {:.0} % fine", f64::from(def.fineness) * 100.0)
            } else {
                String::new()
            }
        );
        let words = match last {
            None => format!(
                "{who} struck the first coin at {name}: {pieces:.0} {terms}, a {:.0} % fee kept.",
                f64::from(def.fee) * 100.0
            ),
            Some((mass, fine)) => format!(
                "{who} struck coin of new terms at {name}: {pieces:.0} {terms}, where the last \
                 were {mass:.0} g{}.",
                if fine < 1.0 {
                    format!(", {:.0} % fine", f64::from(fine) * 100.0)
                } else {
                    String::new()
                }
            ),
        };
        self.chronicle_push(
            ctx.now,
            ChronicleKind::CoinStruck,
            keeper.into_iter().collect(),
            Some(settlement),
            place,
            pieces,
            words,
        );
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
    /// The money a law of pay for common work in force in polity `pi` pays, and its rate: the
    /// oldest law's.
    pub(super) fn work_pay_of(&self, ctx: &Ctx, pi: usize) -> Option<crate::polity::PayDef> {
        self.work_pays_of(ctx, pi).next()
    }

    /// What each law of pay for common work in force in polity `pi` pays, oldest law first (M10d
    /// slice EJ: hours one cannot pay for, the next pays).
    pub(super) fn work_pays_of<'a>(
        &'a self,
        ctx: &'a Ctx,
        pi: usize,
    ) -> impl Iterator<Item = crate::polity::PayDef> + 'a {
        self.polities.get(pi).into_iter().flat_map(move |p| {
            p.laws.iter().filter_map(move |l| {
                if l.status != LawStatus::InForce || l.kind != PolicyKind::PayWork {
                    return None;
                }
                ctx.catalog.policies.get(usize::from(l.policy))?.pay
            })
        })
    }

    /// The terms of polity `pi`'s notes under a law of notes issued in force, if any (M10d slice
    /// EJ).
    pub(super) fn store_notes_of(&self, ctx: &Ctx, pi: usize) -> Option<crate::polity::NotesDef> {
        self.polities.get(pi)?.laws.iter().find_map(|l| {
            if l.status != LawStatus::InForce || l.kind != PolicyKind::IssueNotes {
                return None;
            }
            ctx.catalog.policies.get(usize::from(l.policy))?.notes
        })
    }

    /// Polity `pi`'s own notes as a medium of what is owed its store, under a law of notes issued
    /// in force that has its store take them (M10d slice EJ, second part): only pieces of its own
    /// issue, at the law's rate.
    pub(super) fn notes_dues_medium(
        &self,
        ctx: &Ctx,
        pi: usize,
    ) -> Option<crate::polity::MediumDef> {
        let def = self.store_notes_of(ctx, pi)?;
        let good = ctx
            .catalog
            .goods
            .iter()
            .position(|g| g.money.as_ref().is_some_and(|m| m.note))?;
        (def.dues_per_kg > 0.0).then_some(crate::polity::MediumDef {
            good,
            per_kg: def.dues_per_kg,
        })
    }

    /// Polity `pi`'s issue under terms `def` (M10d slice EJ): the redemption good, and what a
    /// piece pays of it net of the store's fee; for notes nobody redeems, the note good and
    /// nothing.
    pub(super) fn store_terms(def: &crate::polity::NotesDef, note: usize) -> (usize, f32) {
        match def.redeem {
            Some((good, kg)) => (good, kg * (1.0 - def.fee)),
            None => (note, 0.0),
        }
    }

    /// Pieces of its own notes polity `pi`'s store may still write under its law of notes
    /// issued (M10d slice EJ): its ceiling less the pieces out of the store, if someone of its
    /// keeper's household can write; none otherwise.
    pub(super) fn store_may_write(&self, ctx: &Ctx, pi: usize, note: usize) -> f64 {
        let Some(def) = self.store_notes_of(ctx, pi) else {
            return 0.0;
        };
        let polity = &self.polities[pi];
        let writes = polity
            .keeper()
            .and_then(|(k, _)| self.person(k))
            .map(|p| p.household)
            .and_then(|h| self.receipt_writer(ctx, h))
            .is_some();
        if !writes {
            return 0.0;
        }
        let (good, kg) = Self::store_terms(&def, note);
        let out = self
            .notes
            .issues
            .iter()
            .find(|c| c.issuer == polity.id && usize::from(c.good) == good && c.kg == kg)
            .map_or(0.0, |c| {
                (c.count - self.notes.held(polity.id, c.id)).max(0.0)
            });
        (f64::from(def.ceiling) - out).max(0.0)
    }

    /// Pays for the hours households gave to polities' common works since the last call, under
    /// each polity's law of pay for common work, as far as its store holds the money; hours
    /// unpaid for are forgotten.
    pub(super) fn pay_for_work_given(&mut self, ctx: &Ctx) {
        if self.work_given.is_empty() {
            return;
        }
        let given = std::mem::take(&mut self.work_given);
        for (polity, household, hours) in given {
            self.pay_for_work(ctx, polity, household, hours);
        }
    }

    /// Household `household` gave `hours` to a common work of polity `polity`: the store pays it
    /// each law's rate for them, as far as it holds the money, oldest law first, the hours one
    /// cannot pay going to the next (M10d slice EJ). Paying in notes under a law of notes issued,
    /// it writes what it lacks of them first, within the law's ceiling.
    pub(super) fn pay_for_work(
        &mut self,
        ctx: &Ctx,
        polity: PermanentId,
        household: PermanentId,
        hours: f64,
    ) {
        if hours.is_nan() || hours <= 0.0 || self.household(household).is_none() {
            return;
        }
        let Some(pi) = self.polities.iter().position(|p| p.id == polity) else {
            return;
        };
        let goods = &ctx.catalog.goods;
        let note = goods
            .iter()
            .position(|g| g.money.as_ref().is_some_and(|m| m.note));
        let pays: Vec<crate::polity::PayDef> = self.work_pays_of(ctx, pi).collect();
        let mut hours = hours;
        for pay in pays {
            if hours <= 1e-9 || pay.per_hour <= 0.0 {
                break;
            }
            let per_hour = f64::from(pay.per_hour);
            let mut held = self.polities[pi]
                .stores
                .get(pay.good)
                .copied()
                .unwrap_or(0.0);
            if Some(pay.good) == note {
                let wanted = per_hour * hours - held;
                let write = wanted.min(self.store_may_write(ctx, pi, pay.good));
                if write > 1e-9
                    && let Some(def) = self.store_notes_of(ctx, pi)
                {
                    let terms = Self::store_terms(&def, pay.good);
                    if self
                        .put_written(pay.good, polity, polity, terms, write)
                        .is_some()
                    {
                        held += write;
                    }
                }
            }
            let amount = (per_hour * hours).min(held);
            if amount <= 1e-9 {
                continue;
            }
            let leg = Leg {
                from: polity,
                to: household,
                good: pay.good,
                amount,
            };
            if self.transfer(ctx.now, ctx.params, goods, &[leg], Channel::Wage) {
                hours -= amount / per_hour;
            }
        }
    }

    /// For tests: household `household` gave `hours` to a common work of polity `polity`, paid
    /// for at once.
    #[doc(hidden)]
    pub fn work_given_for_tests(
        &mut self,
        ctx: &Ctx,
        polity: PermanentId,
        household: PermanentId,
        hours: f64,
    ) {
        self.work_given.push((polity, household, hours));
        self.pay_for_work_given(ctx);
    }
    /// What each of `households` would be paid a year under each template of pay for common
    /// work, kcal, by template index then household (M7d slice CE3a): its even share of the
    /// money polity `pi`'s store holds, at what a unit is worth to it in hours, at what its hours
    /// bring in food. None while the store holds none.
    pub(super) fn pay_forecasts(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
    ) -> Vec<(u16, Vec<(PermanentId, f64)>)> {
        let goods = &ctx.catalog.goods;
        let mut out = Vec::new();
        for (k, d) in ctx.catalog.policies.iter().enumerate() {
            let Some(pay) = d.pay else {
                continue;
            };
            // Notes the store may still write count as held, worth what they would pay (M10d
            // slice EJ).
            let issued = self.store_notes_of(ctx, pi).filter(|_| {
                goods
                    .get(pay.good)
                    .and_then(|g| g.money.as_ref())
                    .is_some_and(|m| m.note)
            });
            let held = self.polities[pi]
                .stores
                .get(pay.good)
                .copied()
                .unwrap_or(0.0)
                + if issued.is_some() {
                    self.store_may_write(ctx, pi, pay.good)
                } else {
                    0.0
                };
            if held <= 1e-9 || households.is_empty() {
                continue;
            }
            let each = held / households.len() as f64;
            let kcal: Vec<(PermanentId, f64)> = households
                .iter()
                .filter_map(|&h| {
                    let hh = self.household(h)?;
                    let stores = stores_now(hh, ctx.now, ctx.params, goods);
                    let costs = self.own_costs_of(ctx, hh);
                    let paid = self
                        .pay_worth(
                            ctx,
                            hh,
                            pay.good,
                            costs.get(pay.good).copied().flatten(),
                            stores.get(pay.good).copied().unwrap_or(0.0) + each,
                            &costs,
                        )
                        .unwrap_or(0.0);
                    let worth = issued
                        .and_then(|def| self.store_piece_worth(ctx, hh, pi, &def, &costs))
                        .map_or(paid, |w| w.max(paid));
                    Some((h, each * worth * self.hour_kcal(ctx, hh)))
                })
                .collect();
            out.push((k as u16, kcal));
        }
        out
    }
    /// Whether polity `pi`'s store could not pay an hour of common work under any law of pay for
    /// common work in force, for want of what it pays in, counting the notes it may still write
    /// (M10d slice EJ); false with no such law.
    pub(super) fn work_unpaid(&self, ctx: &Ctx, pi: usize) -> bool {
        let note = ctx
            .catalog
            .goods
            .iter()
            .position(|g| g.money.as_ref().is_some_and(|m| m.note));
        let mut pays = self.work_pays_of(ctx, pi).peekable();
        pays.peek().is_some()
            && pays.all(|pay| {
                let held = self.polities[pi]
                    .stores
                    .get(pay.good)
                    .copied()
                    .unwrap_or(0.0);
                let may = if Some(pay.good) == note {
                    self.store_may_write(ctx, pi, pay.good)
                } else {
                    0.0
                };
                held + may < f64::from(pay.per_hour)
            })
    }

    /// Whether polity `polity`'s store could not pay an hour of common work under its laws of pay
    /// ([`Self::work_unpaid`]): for tests.
    #[doc(hidden)]
    pub fn work_unpaid_for_tests(&self, ctx: &Ctx, polity: PermanentId) -> bool {
        self.polities
            .iter()
            .position(|p| p.id == polity)
            .is_some_and(|pi| self.work_unpaid(ctx, pi))
    }

    /// What a piece of polity `pi`'s notes under terms `def` would be worth to household `hh` at
    /// its own costs, in hours, before any is written (M10d slice EJ): what it pays, at its cost
    /// of the good, less its share of a walk to the hearth and back.
    pub(super) fn store_piece_worth(
        &self,
        ctx: &Ctx,
        hh: &Household,
        pi: usize,
        def: &crate::polity::NotesDef,
        costs: &[Option<f64>],
    ) -> Option<f64> {
        let note = ctx
            .catalog
            .goods
            .iter()
            .position(|g| g.money.as_ref().is_some_and(|m| m.note))?;
        let (good, kg) = Self::store_terms(def, note);
        let issue = crate::notes::Issue {
            id: 0,
            issuer: self.polities.get(pi)?.id,
            good: u16::try_from(good).ok()?,
            kg,
            count: 0.0,
            presented: 0.0,
            refused: 0.0,
            refused_day: -1,
            lent: 0.0,
            defaulted_day: -1,
            seen: 0.0,
            seen_day: -1,
            suspended_until: -1,
        };
        // Its own, once written, as households believe it pays (M10d slice EH).
        let issue = self
            .notes
            .issues
            .iter()
            .find(|c| c.issuer == issue.issuer && c.good == issue.good && c.kg == kg)
            .unwrap_or(&issue);
        self.piece_worth(ctx, hh, issue, costs)
    }

    /// What each household would be paid in the store's notes under each template of notes
    /// issued, kcal, by template index and then household id (M10d slice EJ): its share of the
    /// ceiling, at what a piece would pay it, at what its hours bring in food; none where no
    /// template of pay for common work pays in notes, as the store issues only to spend.
    pub(super) fn store_notes_forecasts(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
    ) -> Vec<(u16, Vec<(PermanentId, f64)>)> {
        let goods = &ctx.catalog.goods;
        let spends = ctx.catalog.policies.iter().any(|d| {
            d.pay.is_some_and(|p| {
                goods
                    .get(p.good)
                    .and_then(|g| g.money.as_ref())
                    .is_some_and(|m| m.note)
            })
        });
        if !spends || households.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for (k, d) in ctx.catalog.policies.iter().enumerate() {
            let Some(def) = d.notes else {
                continue;
            };
            let each = f64::from(def.ceiling) / households.len() as f64;
            let kcal: Vec<(PermanentId, f64)> = households
                .iter()
                .filter_map(|&h| {
                    let hh = self.household(h)?;
                    let costs = self.own_costs_of(ctx, hh);
                    let worth = self.store_piece_worth(ctx, hh, pi, &def, &costs)?;
                    Some((h, each * worth * self.hour_kcal(ctx, hh)))
                })
                .collect();
            out.push((k as u16, kcal));
        }
        out
    }

    /// What a piece of the coin a mint template strikes is worth to each of `households` that
    /// counts pieces rather than assaying them, hours, by household id (M7d slice CE4b): what
    /// pieces bought it, as it remembers; 0 for one that can assay.
    pub(super) fn count_piece_worths(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
        mints: &[(u16, f64, f64, f64)],
    ) -> Vec<(PermanentId, f64)> {
        let goods = &ctx.catalog.goods;
        let Some(coin) = mints.iter().find_map(|&(k, ..)| {
            let m = ctx.catalog.policies.get(usize::from(k))?.mint?;
            ctx.catalog
                .recipes
                .get(m.recipe)?
                .outputs
                .first()
                .map(|o| o.0)
        }) else {
            return Vec::new();
        };
        let Some(money) = goods.get(coin).and_then(|g| g.money.as_ref()) else {
            return Vec::new();
        };
        let mut out: Vec<(PermanentId, f64)> = households
            .iter()
            .filter_map(|&h| {
                let hh = self.household(h)?;
                if self.assays(h, money.assay) {
                    return Some((h, 0.0));
                }
                let held = hh.stores.get(coin).copied().unwrap_or(0.0);
                let w = self.pay_worth_raw(ctx, hh, coin, None, held).unwrap_or(0.0);
                Some((h, w))
            })
            .collect();
        out.sort_unstable_by_key(|x| x.0);
        out
    }
}
