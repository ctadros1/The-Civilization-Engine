//! Custody at midnight (M10d slice EG, second part; ADR-0027 §4; ADR-0035 §9; brief m10-money
//! §3): the probe of slice EG found households under their own roof never hold more than it
//! keeps, but those still waiting on one keep everything in the open. At its weekly review such a
//! household leaves the food that keeps better under a roof it can spare beyond what it keeps to
//! its next harvest (as it would offer it for sale; not the seed it keeps back to sow), with
//! a neighbour of its settlement whose roofs have room for it after its own goods and what it
//! keeps for others: close kin first, who ask nothing, else the nearest, who take a share each
//! month. It does when what lying in the open would lose it in a month, at its own costs, is more
//! than the fee and the walks there and back (weighed, not walked, as bringing metal to a mint
//! is). Once it has a roof of its own it takes back as much as that roof has room for, the rest
//! still kept; when its food in hand runs below nine tenths of what it keeps to its harvest, the
//! food that brings it back there; and everything when the keeper has no longer room for it.
//! What a household gone left is the keeper's. Each is applied at midnight in household id order.
//!
//! Where someone of the keeper's household can write (the first by id), they write a receipt when
//! the goods are left (third part; brief m10-money §3), as notes (slice EH): the keeper takes a
//! year's fee at once (none from close kin) and writes its issue in each good for the rest, a
//! kilogram a piece, held by the depositor. The goods then back the keeper's notes, whoever holds
//! them: the depositor takes them back by presenting its pieces at the keeper's door, and when the
//! keeper can no longer keep them every piece is presented. Presentations are paid at midnight
//! issue by issue in an order drawn for the issue and the day, never household id order, from the
//! goods backing them; pieces they cannot pay are refused and stay owed.

use super::*;
use crate::custody::Kept;

/// Days between a keeper's fees.
pub(super) const FEE_DAYS: i64 = 30;
/// Below this share of the food it keeps to its next harvest, a household takes back what brings
/// it there (a design prior, so that it does not go back and forth each day).
pub(super) const RETAKE_SHARE: f64 = 0.9;
/// Months of a keeper's fee taken at once from goods whose receipt is written as notes, which
/// pay no monthly fee after (M10d slice EH; a design prior, so that fees do not draw the goods
/// backing notes below them).
const FEE_PREPAID_MONTHS: f64 = 12.0;
/// Days a keeper that refused presentations of its notes pays none, unless what is behind them
/// again covers every piece out (M10d slice EI; a design prior).
const SUSPEND_DAYS: i64 = 30;
/// Kilograms of its good a piece of a keeper's notes is redeemed for (a design prior).
const NOTE_KG: f32 = 1.0;
/// Metres a second one walks on an errand to a neighbour (a design prior, as the walk is weighed,
/// not walked).
pub(super) const WALK_M_S: f64 = 1.2;

impl Population {
    /// What household `x` with `stores` has room for under its roofs beyond its own goods that
    /// keep better there and what it keeps for others, kilograms: none without a roof.
    pub(super) fn custody_room(&self, ctx: &Ctx, x: &Household, stores: &[f64]) -> f64 {
        if !x.sheltered {
            return 0.0;
        }
        let goods = &ctx.catalog.goods;
        let keeping = x.keeping.with_stores(stores, goods);
        let room = keeping.raised_kg.max(0.0) + keeping.roofed_kg.max(0.0);
        let own: f64 = goods
            .iter()
            .zip(stores)
            .filter(|(d, _)| d.sheltered_half_life_days > 0.0)
            .map(|(_, kg)| kg.max(0.0))
            .sum();
        let others: f64 = self
            .kept
            .iter()
            .filter(|k| k.keeper == x.id)
            .map(Kept::kg)
            .sum();
        if room.is_finite() {
            room - own - others
        } else {
            f64::INFINITY
        }
    }

    /// The midnight's custody (see the module documentation).
    pub(super) fn custody_day(&mut self, ctx: &mut Ctx, day: i64) {
        let (now, params) = (ctx.now, ctx.params);
        let goods = &ctx.catalog.goods;
        let mp = &params.market;
        if !self.kept.is_empty() {
            for k in &mut self.kept {
                k.settle_stores(now, goods);
            }
            // What goes back, to whom, how much of each good, and the fees, decided first; for
            // goods backing notes, the pieces presented, and the keepers whose notes are all
            // called in (M10d slice EH).
            let mut back: Vec<(PermanentId, PermanentId, Vec<f64>)> = Vec::new();
            let mut fees: Vec<(PermanentId, PermanentId, Vec<f64>)> = Vec::new();
            let mut presented: Vec<(u32, PermanentId, f64)> = Vec::new();
            let mut called_in: Vec<PermanentId> = Vec::new();
            for k in &self.kept {
                let depositor = self
                    .household(k.depositor)
                    .filter(|x| !x.members.is_empty());
                let keeper = self.household(k.keeper).filter(|x| !x.members.is_empty());
                if k.noted {
                    let Some(keeper) = keeper else {
                        called_in.push(k.keeper);
                        continue;
                    };
                    let keeper_stores = stores_now(keeper, now, params, goods);
                    if !keeper.sheltered || self.custody_room(ctx, keeper, &keeper_stores) < -1e-6 {
                        called_in.push(k.keeper);
                        continue;
                    }
                    // Its depositor takes back what it would by presenting the pieces it holds.
                    if let Some(d) = depositor
                        && let Some(take) = self.take_back(ctx, d, k)
                    {
                        for (g, kg) in take.into_iter().enumerate() {
                            let Some(issue) = self.notes.issue_of(k.keeper, g) else {
                                continue;
                            };
                            let pieces = (kg / f64::from(issue.kg).max(1e-9))
                                .min(self.notes.held(d.id, issue.id));
                            if pieces > 1e-9 {
                                presented.push((issue.id, d.id, pieces));
                            }
                        }
                    }
                    continue;
                }
                match (depositor, keeper) {
                    // A household gone left what it kept with another to its keeper.
                    (None, Some(_)) => back.push((k.id, k.keeper, k.stores.clone())),
                    (None, None) => {}
                    (Some(_), None) => back.push((k.id, k.depositor, k.stores.clone())),
                    (Some(d), Some(keeper)) => {
                        let keeper_stores = stores_now(keeper, now, params, goods);
                        let crowded = self.custody_room(ctx, keeper, &keeper_stores) < -1e-6;
                        if crowded || !keeper.sheltered {
                            back.push((k.id, k.depositor, k.stores.clone()));
                            continue;
                        }
                        if let Some(take) = self.take_back(ctx, d, k) {
                            back.push((k.id, k.depositor, take));
                        } else if day - k.fee_day >= FEE_DAYS
                            && self.kin_households(d).binary_search(&keeper.id).is_err()
                        {
                            let fee: Vec<f64> =
                                k.stores.iter().map(|x| x * mp.custody_fee_share).collect();
                            fees.push((k.id, k.keeper, fee));
                        }
                    }
                }
            }
            for (id, keeper, fee) in fees {
                let legs: Vec<Leg> = fee
                    .iter()
                    .enumerate()
                    .filter(|&(_, &kg)| kg > 1e-9)
                    .map(|(good, &amount)| Leg {
                        from: id,
                        to: keeper,
                        good,
                        amount,
                    })
                    .collect();
                if !legs.is_empty() {
                    self.transfer(now, params, goods, &legs, Channel::Custody);
                }
                if let Some(k) = self.kept.iter_mut().find(|k| k.id == id) {
                    k.fee_day = day;
                }
            }
            for (id, to, take) in back {
                let legs: Vec<Leg> = take
                    .iter()
                    .enumerate()
                    .filter(|&(_, &kg)| kg > 1e-9)
                    .map(|(good, &amount)| Leg {
                        from: id,
                        to,
                        good,
                        amount,
                    })
                    .collect();
                if !legs.is_empty() {
                    self.transfer(now, params, goods, &legs, Channel::Custody);
                }
            }
            // Holders of notes whose review it is present them when short or on hearing refusals
            // (M10d slice EH, second part).
            presented.extend(self.notes_review(ctx, day));
            // Every piece of a keeper's notes is presented when its goods can no longer be kept.
            called_in.sort_unstable();
            called_in.dedup();
            for keeper in called_in {
                for issue in self.notes.issues.iter().filter(|c| c.issuer == keeper) {
                    for p in &self.notes.purses {
                        let n = self.notes.held(p.holder, issue.id);
                        if p.holder != keeper && n > 1e-9 {
                            presented.push((issue.id, p.holder, n));
                        }
                    }
                }
            }
            self.redeem(ctx, day, presented);
            self.settle_unbacked(ctx);
            let (empty, full): (Vec<Kept>, Vec<Kept>) = std::mem::take(&mut self.kept)
                .into_iter()
                .partition(Kept::is_empty);
            for k in &empty {
                self.kept_flows.absorb(&k.flows);
            }
            self.kept = full;
        }
        // Households with no roof of their own whose day it is weigh leaving goods with another.
        let review_days = i64::from(mp.review_days.max(1));
        let leaving: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| {
                x.settlement.is_some()
                    && !x.members.is_empty()
                    && !x.sheltered
                    && (x.id.get() as i64 + day).rem_euclid(review_days) == 0
                    && !self.kept.iter().any(|k| k.depositor == x.id)
            })
            .map(|x| x.id)
            .collect();
        for h in leaving {
            let Some((keeper, legs)) = self.custody_choice(ctx, h) else {
                continue;
            };
            let id = ctx.ids.allocate();
            self.kept
                .push(Kept::new(id, (h, keeper), now, ctx.catalog.goods.len()));
            let legs: Vec<Leg> = legs
                .into_iter()
                .map(|(good, amount)| Leg {
                    from: h,
                    to: id,
                    good,
                    amount,
                })
                .collect();
            if !self.transfer(now, params, &ctx.catalog.goods, &legs, Channel::Custody) {
                self.kept.pop();
                continue;
            }
            let writer = self.receipt_writer(ctx, keeper);
            if let Some(k) = self.kept.last_mut() {
                k.receipt = writer.map(|w| (w, day));
            }
            if writer.is_some() {
                self.write_notes(ctx, id, day);
            }
        }
    }

    /// What household `d` takes back of lot `k` it left, by good, if anything (see the module
    /// documentation): under a roof of its own, as much as its room holds; short of the food it
    /// keeps to its next harvest, what brings it back there.
    fn take_back(&self, ctx: &Ctx, d: &Household, k: &Kept) -> Option<Vec<f64>> {
        let (now, params) = (ctx.now, ctx.params);
        let goods = &ctx.catalog.goods;
        let stores = stores_now(d, now, params, goods);
        let food = stock_kcal(&stores, goods);
        let target = self.holding_of(ctx, d, &stores).food_keep;
        let room_share = if d.sheltered {
            let room = self.custody_room(ctx, d, &stores).max(0.0);
            (room / k.kg().max(1e-9)).min(1.0)
        } else {
            0.0
        };
        let kept_food = stock_kcal(&k.stores, goods);
        let food_share = if food < RETAKE_SHARE * target {
            ((target - food) / kept_food.max(1e-9)).min(1.0)
        } else {
            0.0
        };
        (room_share > 0.0 || food_share > 0.0).then(|| {
            k.stores
                .iter()
                .zip(goods)
                .map(|(&kg, g)| {
                    let food = g.purpose == GoodUse::Food && g.kcal_per_kg > 0.0;
                    let share = if food {
                        room_share.max(food_share)
                    } else {
                        room_share
                    };
                    kg * share
                })
                .collect()
        })
    }

    /// The receipt for lot `id`, just left, written as notes (M10d slice EH): its keeper takes a
    /// year's fee from it at once unless close kin of its depositor, and writes its issue in each
    /// good for the rest, a kilogram a piece, into the depositor's purse; the lot backs them from
    /// then on and pays no monthly fee. Nothing where the content has no note good.
    fn write_notes(&mut self, ctx: &Ctx, id: PermanentId, day: i64) {
        let (now, params) = (ctx.now, ctx.params);
        let goods = &ctx.catalog.goods;
        let Some(note) = goods
            .iter()
            .position(|g| g.money.as_ref().is_some_and(|m| m.note))
        else {
            return;
        };
        let Some(k) = self.kept.iter().find(|k| k.id == id) else {
            return;
        };
        let (depositor, keeper) = (k.depositor, k.keeper);
        let kin = self
            .household(depositor)
            .is_some_and(|d| self.kin_households(d).binary_search(&keeper).is_ok());
        if !kin {
            let share = (FEE_PREPAID_MONTHS * params.market.custody_fee_share).min(1.0);
            let legs: Vec<Leg> = k
                .stores
                .iter()
                .enumerate()
                .filter(|&(_, &kg)| kg * share > 1e-9)
                .map(|(good, &kg)| Leg {
                    from: id,
                    to: keeper,
                    good,
                    amount: kg * share,
                })
                .collect();
            if !legs.is_empty() {
                self.transfer(now, params, goods, &legs, Channel::Custody);
            }
        }
        let Some(k) = self.kept.iter_mut().find(|k| k.id == id) else {
            return;
        };
        k.noted = true;
        k.fee_day = day;
        let lots: Vec<(usize, f64)> = k
            .stores
            .iter()
            .enumerate()
            .filter(|&(_, &kg)| kg > 1e-9)
            .map(|(g, &kg)| (g, kg))
            .collect();
        for (g, kg) in lots {
            let pieces = kg / f64::from(NOTE_KG);
            self.put_written(note, keeper, depositor, (g, NOTE_KG), pieces);
        }
    }

    /// Pays the pieces `presented` (issue, holder, pieces) at their issuers' doors (M10d slice EH;
    /// brief m10-money §9, §10): issue by issue, in an order drawn for the issue and the day, never
    /// household id order, each from the goods its issuer keeps backing it, oldest left first,
    /// the pieces moving to the issuer and retired and the good to the holder on `Redeemed`.
    /// Pieces the backing can no longer pay are refused, staying with their holder, and counted.
    /// A keeper living that refused any suspends paying them for `SUSPEND_DAYS`, refusing every
    /// piece presented, unless what is behind them again covers every piece out (slice EI).
    pub(super) fn redeem(
        &mut self,
        ctx: &Ctx,
        day: i64,
        mut presented: Vec<(u32, PermanentId, f64)>,
    ) {
        if presented.is_empty() {
            return;
        }
        let (now, params) = (ctx.now, ctx.params);
        let goods = &ctx.catalog.goods;
        let Some(note) = goods
            .iter()
            .position(|g| g.money.as_ref().is_some_and(|m| m.note))
        else {
            return;
        };
        presented.sort_by_key(|&(issue, holder, _)| (issue, holder));
        let mut order: Vec<(u32, PermanentId, f64)> = Vec::with_capacity(presented.len());
        for chunk in presented.chunk_by(|a, b| a.0 == b.0) {
            let mut chunk = chunk.to_vec();
            let mut rng = Rng64::from_key(&[0x4e4f_5445, u64::from(chunk[0].0), day as u64]);
            for i in (1..chunk.len()).rev() {
                let j = rng.below(i as u64 + 1) as usize;
                chunk.swap(i, j);
            }
            order.extend(chunk);
        }
        for (issue, holder, pieces) in order {
            let Some(c) = self.notes.issue(issue) else {
                continue;
            };
            let (issuer, good, kg) = (c.issuer, usize::from(c.good), f64::from(c.kg));
            let pieces = pieces.min(self.notes.held(holder, issue));
            if pieces <= 1e-9 || kg <= 0.0 {
                continue;
            }
            // A polity's store pays its own notes from its stores (M10d slice EJ).
            let store = self.polities.iter().position(|p| p.id == issuer);
            let alive = store.is_some()
                || self
                    .household(issuer)
                    .is_some_and(|x| !x.members.is_empty());
            // Suspended, it pays none, unless what is behind them covers every piece out.
            if alive && c.suspended_until > day {
                let covered = self.note_behind(ctx, c) + 1e-6 >= c.count * kg;
                if !covered {
                    self.notes.note_presented(issue, pieces, pieces, day);
                    self.refused_to_debt(ctx, day, (issue, holder), pieces);
                    continue;
                }
                if let Some(c) = self.notes.issue_mut(issue) {
                    c.suspended_until = -1;
                }
            }
            // From its lots backing notes, oldest left first.
            let mut want = pieces * kg;
            let mut legs: Vec<Leg> = Vec::new();
            for k in self.kept.iter().filter(|k| k.keeper == issuer && k.noted) {
                if want <= 1e-9 {
                    break;
                }
                let has = k.stores.get(good).copied().unwrap_or(0.0).max(0.0);
                let take = has.min(want);
                if take > 1e-9 {
                    legs.push(Leg {
                        from: k.id,
                        to: holder,
                        good,
                        amount: take,
                    });
                    want -= take;
                }
            }
            // Then from the keeper's own beyond what it keeps (M10d slice EI), or from the store.
            if want > 1e-9 {
                let own = match store {
                    Some(pi) => self.polities[pi]
                        .stores
                        .get(good)
                        .copied()
                        .unwrap_or(0.0)
                        .max(0.0),
                    None => self.keeper_spare(ctx, issuer, good),
                }
                .min(want);
                if own > 1e-9 {
                    legs.push(Leg {
                        from: issuer,
                        to: holder,
                        good,
                        amount: own,
                    });
                    want -= own;
                }
            }
            let paid = (pieces - want.max(0.0) / kg).max(0.0);
            let at = if alive && paid > 1e-9 {
                legs.push(Leg {
                    from: holder,
                    to: issuer,
                    good: note,
                    amount: paid,
                });
                issuer
            } else {
                holder
            };
            let mut done = 0.0;
            if paid > 1e-9 && self.transfer(now, params, goods, &legs, Channel::Redeemed) {
                if at == issuer {
                    // The pieces went to their issuer outside the purses; it holds them now.
                    self.notes.give_issue(holder, issuer, issue, paid);
                }
                if self.retire_written((now, params, goods), at, (note, issue), paid) {
                    done = paid;
                }
            }
            let refused = (pieces - done).max(0.0);
            self.notes.note_presented(issue, pieces, refused, day);
            if alive && refused > 1e-9 {
                self.refused_to_debt(ctx, day, (issue, holder), refused);
            }
            // Having refused, a keeper living suspends; a store does not of its own accord.
            if alive
                && store.is_none()
                && refused > 1e-9
                && let Some(c) = self.notes.issue_mut(issue)
            {
                c.suspended_until = day + SUSPEND_DAYS;
            }
        }
    }

    /// Under a law of bearer heard in force where its keeper lives (M10d slice EI, third part;
    /// brief m10-money §4), `pieces` of issue `issue` refused to `holder` stand as what the keeper
    /// owes it: they are retired from its purse and booked in the credit book as a loan of their
    /// good refused today, which the holder may bring before the gathering as a debt. Without the
    /// law they stay with the holder, owed and unheard.
    fn refused_to_debt(
        &mut self,
        ctx: &Ctx,
        day: i64,
        (issue, holder): (u32, PermanentId),
        pieces: f64,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let goods = &ctx.catalog.goods;
        let Some(c) = self.notes.issue(issue) else {
            return;
        };
        let (issuer, good, kg) = (c.issuer, c.good, f64::from(c.kg));
        let heard = self
            .household(issuer)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
            .is_some_and(|pi| {
                self.polities[pi].laws.iter().any(|l| {
                    l.kind == crate::polity::PolicyKind::BearerHeard
                        && l.status == crate::polity::LawStatus::InForce
                })
            });
        let Some(note) = goods
            .iter()
            .position(|g| g.money.as_ref().is_some_and(|m| m.note))
        else {
            return;
        };
        if !heard || !self.retire_written((now, params, goods), holder, (note, issue), pieces) {
            return;
        }
        let witnesses = [
            self.elder_of(holder, now, params).unwrap_or(holder),
            self.elder_of(issuer, now, params).unwrap_or(issuer),
        ];
        let credit = &mut self.order.credit;
        credit.next += 1;
        let id = credit.next;
        credit.claims.push(crate::credit::Claim {
            id,
            creditor: holder,
            debtor: issuer,
            good,
            lent: (pieces * kg) as f32,
            owed: (pieces * kg) as f32,
            paid: 0.0,
            made: day,
            due: day,
            standing: crate::credit::Standing::Defaulted,
            witnesses,
            pledge: None,
            answer: None,
            asks: 0,
            extensions: 0,
            taken: None,
            by_guarantor: 0.0,
            recourse: None,
        });
        if let Err(at) = self.notes.dishonoured.binary_search(&id) {
            self.notes.dishonoured.insert(at, id);
        }
    }

    /// Goods backing a keeper's notes that no piece outstanding claims any more (pieces taken out
    /// of the world, or rounding) are its keeper's, or with no keeper the depositor's (M10d slice
    /// EH).
    fn settle_unbacked(&mut self, ctx: &Ctx) {
        let (now, params) = (ctx.now, ctx.params);
        let goods = &ctx.catalog.goods;
        let mut moves: Vec<(PermanentId, PermanentId, usize, f64)> = Vec::new();
        for k in self.kept.iter().filter(|k| k.noted) {
            for (g, &kg) in k.stores.iter().enumerate() {
                if kg <= 1e-9 {
                    continue;
                }
                let owed = self
                    .notes
                    .issue_of(k.keeper, g)
                    .map_or(0.0, |c| c.count * f64::from(c.kg));
                let backing: f64 = self
                    .kept
                    .iter()
                    .filter(|o| o.keeper == k.keeper && o.noted)
                    .map(|o| o.stores.get(g).copied().unwrap_or(0.0).max(0.0))
                    .sum();
                if owed > 1e-6 || backing <= 1e-9 {
                    continue;
                }
                let to = [k.keeper, k.depositor]
                    .into_iter()
                    .find(|&h| self.household(h).is_some_and(|x| !x.members.is_empty()));
                if let Some(to) = to {
                    moves.push((k.id, to, g, kg));
                }
            }
        }
        for (from, to, good, amount) in moves {
            let legs = [Leg {
                from,
                to,
                good,
                amount,
            }];
            self.transfer(now, params, goods, &legs, Channel::Custody);
        }
    }

    /// The first person by id of household `keeper` who can write, if any.
    pub(super) fn receipt_writer(&self, ctx: &Ctx, keeper: PermanentId) -> Option<PermanentId> {
        let writing = Self::letter_technique(ctx.catalog, Behavior::Write)?;
        self.household(keeper)?
            .members
            .iter()
            .copied()
            .filter(|&m| self.person(m).is_some_and(|p| p.knows(writing)))
            .min()
    }

    /// The keeper household `h` would leave goods with, and what it would leave (good,
    /// kilograms), if leaving them is worth more to it than it costs (see the module
    /// documentation).
    fn custody_choice(
        &self,
        ctx: &Ctx,
        h: PermanentId,
    ) -> Option<(PermanentId, Vec<(usize, f64)>)> {
        let (now, params) = (ctx.now, ctx.params);
        let goods = &ctx.catalog.goods;
        let mp = &params.market;
        let x = self.household(h)?;
        let settlement = x.settlement?;
        let stores = stores_now(x, now, params, goods);
        // Of its food, what it can spare beyond what it keeps to its next harvest (as it would
        // offer it for sale), from what keeps longest.
        let holding = self.holding_of(ctx, x, &stores);
        let leave: Vec<(usize, f64)> = goods
            .iter()
            .enumerate()
            // Food only: not seed kept back for sowing, nor what it works or builds with.
            .filter(|(_, d)| {
                d.sheltered_half_life_days > 0.0
                    && d.purpose == GoodUse::Food
                    && d.kcal_per_kg > 0.0
                    && !d.kept_back()
            })
            .filter_map(|(g, _)| {
                let held = stores.get(g).copied().unwrap_or(0.0).max(0.0);
                let kg = held - holding.keep.get(g).copied().unwrap_or(held);
                (kg > 1e-6).then_some((g, kg))
            })
            .collect();
        let total: f64 = leave.iter().map(|l| l.1).sum();
        if total < 1.0 {
            return None;
        }
        // The keeper: close kin with room first, else the nearest with room.
        let kin_of = self.kin_households(x);
        let (keeper, kin, metres) = self
            .households_in(settlement)
            .filter(|k| k.id != h && !k.members.is_empty() && k.sheltered)
            .filter(|k| {
                let ks = stores_now(k, now, params, goods);
                self.custody_room(ctx, k, &ks) >= total
            })
            .map(|k| {
                let d = f64::from(k.home.0 - x.home.0).hypot(f64::from(k.home.1 - x.home.1));
                (k.id, kin_of.binary_search(&k.id).is_ok(), d)
            })
            .min_by(|a, b| b.1.cmp(&a.1).then(a.2.total_cmp(&b.2)).then(a.0.cmp(&b.0)))?;
        // What lying in the open would lose it in a month, against the fee and the walks.
        let costs = self.own_costs_of(ctx, x);
        let lost = |half: f64| {
            if half > 0.0 {
                1.0 - 0.5f64.powf(FEE_DAYS as f64 / half)
            } else {
                0.0
            }
        };
        let mut gain = 0.0;
        let mut worth = 0.0;
        for &(g, kg) in &leave {
            let d = &goods[g];
            let c = costs.get(g).copied().flatten().unwrap_or(0.0);
            gain += kg * c * (lost(d.half_life_days) - lost(d.sheltered_half_life_days)).max(0.0);
            worth += kg * c;
        }
        let fee = if kin {
            0.0
        } else {
            mp.custody_fee_share * worth
        };
        // There and back to leave them, and again to take them back.
        let walk_h = 4.0 * metres / WALK_M_S / 3600.0;
        (gain > fee + walk_h).then_some((keeper, leave))
    }

    /// What household `household` has left with others, and what it keeps for others,
    /// kilograms: for tests.
    #[doc(hidden)]
    pub fn custody_for_tests(&self, household: PermanentId) -> (f64, f64) {
        let left = self
            .kept
            .iter()
            .filter(|k| k.depositor == household)
            .map(Kept::kg)
            .sum();
        let keeps = self
            .kept
            .iter()
            .filter(|k| k.keeper == household)
            .map(Kept::kg)
            .sum();
        (left, keeps)
    }

    /// The households of household `household`'s people's close kin: for tests.
    #[doc(hidden)]
    pub fn kin_households_for_tests(&self, household: PermanentId) -> Vec<PermanentId> {
        self.household(household)
            .map_or_else(Vec::new, |x| self.kin_households(x))
    }

    /// The midnight's custody now, as at midnight: for tests.
    #[doc(hidden)]
    pub fn custody_day_for_tests(&mut self, ctx: &mut Ctx) {
        let day = ctx.now.day_index();
        self.custody_day(ctx, day);
    }
}
