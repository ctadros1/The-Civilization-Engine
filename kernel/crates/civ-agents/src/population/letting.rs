//! Letting rooms at midnight (ADR-0040). Lets on a building no longer roofed, or whose landlord
//! or lodger is gone, end first. Rent falls due a month at a time: the lodger pays it on the
//! `Rent` channel from its food, the let's good first and the same food energy in its other foods
//! for what that does not cover; one that cannot pay is in arrears, and a second unpaid month
//! ends the let (§3). At a let's review it ends when the lodger has a roof of its own or the
//! landlord's members need the floor. Then households whose review day it is: one with no roof of
//! its own and none nearly on (its roof's stage under way) weighs up to `candidates` homes of its
//! settlement within its reach with spare floor for its members, nearest first, against staying
//! as it is: a month's rent at the landlord's asking in hours by its `hour_kcal`, and the change
//! in its walks a month (ADR-0030 §1's walks), against what its goods would keep under the room's
//! share of the roof beyond the open over a month and the custody fee it pays (ADR-0035). It
//! offers on the cheapest that pays; its landlord accepts when it still has the room, else turns
//! it away and asks a step more (§2). A household with spare floor and no lodger while some of its
//! settlement are roofless asks a step less. Each is applied in household id order.

use super::*;
use crate::letting::{ARREARS_MONTHS, Let, LetBegun, RECENT_DAYS, RENT_DAYS};

/// A household's home it could let floor in (ADR-0040 §1).
#[derive(Clone, Copy, Debug)]
pub(super) struct LetHome {
    pub building: PermanentId,
    /// Its sleeping (living) floor, m².
    pub living_m2: f64,
    /// What its program gives a sleeper, and a household whatever its size, m².
    pub per_sleeper_m2: f64,
    pub base_m2: f64,
    /// Its centre, metres.
    pub centre: (f32, f32),
    /// The storey a let in it lies on now, 0 the first (ADR-0042 §1): the highest with floor to
    /// spare.
    pub let_storey: u8,
}

/// A home with spare floor as a day's seekers see it: its landlord, the landlord's settlement, the
/// home and the floor spare in it, m².
type Spare = (PermanentId, PermanentId, LetHome, f64);

/// The sleeping floor program `def` gives a household and each of its sleepers, m².
pub(super) fn sleeping(def: &crate::params::BuildingDef) -> (f64, f64) {
    match &def.rules {
        civ_grammar::ProgramRules::Hut(h) => (h.floor_base_m2, h.floor_m2_per_sleeper),
        civ_grammar::ProgramRules::Frame(f) => (f.floor_base_m2, f.floor_m2_per_sleeper),
    }
}

impl Population {
    /// Whether household `h` lodges under another's roof (ADR-0040 §4).
    pub(crate) fn lodges(&self, h: PermanentId) -> bool {
        self.letting.lodging(h).is_some()
    }

    /// The home household `x` could let floor in: of its roofed, finished dwellings `homes`, the
    /// one with the most sleeping floor (ADR-0040 §1). `None` for a lodger, which lets nothing.
    pub(super) fn let_home(
        &self,
        land: &Land,
        catalog: &Catalog,
        x: &Household,
        homes: &[&Building],
    ) -> Option<LetHome> {
        if self.lodges(x.id) {
            return None;
        }
        let mut best: Option<(LetHome, Vec<f64>)> = None;
        for &b in homes {
            let Some(def) = catalog
                .building_index(&b.spec.program)
                .and_then(|i| catalog.buildings.get(i))
            else {
                continue;
            };
            let Some((living, by_storey)) = self.living_floor(land, catalog, b, def) else {
                continue;
            };
            let (base, per) = sleeping(def);
            if best.as_ref().is_none_or(|h| living > h.0.living_m2) {
                best = Some((
                    LetHome {
                        building: b.id,
                        living_m2: living,
                        per_sleeper_m2: per,
                        base_m2: base,
                        centre: build::centre_m(&b.spec),
                        let_storey: 0,
                    },
                    by_storey,
                ));
            }
        }
        best.map(|(home, by_storey)| self.with_let_storey(x, home, &by_storey))
    }

    /// Building `b`'s sleeping floor under program `def`, m², in all and by storey: less, on
    /// every storey, what a lift's shaft standing in it takes (ADR-0042 §2).
    fn living_floor(
        &self,
        land: &Land,
        catalog: &Catalog,
        b: &Building,
        def: &crate::params::BuildingDef,
    ) -> Option<(f64, Vec<f64>)> {
        let of = |e: &civ_grammar::Expansion| {
            (
                e.floor_by_use[civ_grammar::SpaceUse::Living.index()],
                crate::climb::living_by_storey(e),
            )
        };
        let (living, mut by_storey) = match self.expansions.get(&b.id) {
            Some(e) => of(e),
            None => of(&civ_grammar::expand(&b.spec, &def.rules).ok()?),
        };
        let Some(lift) = self.lift_in(land, &catalog.goods, b) else {
            return Some((living, by_storey));
        };
        for a in &mut by_storey {
            *a = (*a - lift.shaft_m2).max(0.0);
        }
        Some((by_storey.iter().sum(), by_storey))
    }

    /// `home` of household `x`, with the storey a let in it would lie on: the highest with floor
    /// to spare once its own members' floor is taken from the ground up and what it lets from the
    /// top down (ADR-0042 §1).
    fn with_let_storey(&self, x: &Household, mut home: LetHome, by_storey: &[f64]) -> LetHome {
        let own = home.base_m2 + home.per_sleeper_m2 * x.members.len() as f64;
        let let_m2: f64 = self
            .letting
            .letting(x.id)
            .filter(|l| l.building == home.building)
            .map(|l| l.floor_m2)
            .sum();
        home.let_storey = crate::climb::let_storey(by_storey, own, let_m2).min(255) as u8;
        home
    }

    /// The floor household `x` has spare in `home`: its sleeping floor beyond what its own
    /// members need, less what it lets there to households other than `except` (ADR-0040 §1).
    pub(super) fn spare_m2(
        &self,
        x: &Household,
        home: &LetHome,
        except: Option<PermanentId>,
    ) -> f64 {
        let own = home.base_m2 + home.per_sleeper_m2 * x.members.len() as f64;
        let let_m2: f64 = self
            .letting
            .letting(x.id)
            .filter(|l| l.building == home.building && Some(l.lodger) != except)
            .map(|l| l.floor_m2)
            .sum();
        home.living_m2 - own - let_m2
    }

    /// Brings households `ids`' stores up to `now` under the shelter they had, then derives their
    /// shelter again with the lets as they now stand.
    fn reshelter(
        &mut self,
        now: SimTime,
        (land, catalog, params): (&Land, &Catalog, &PeopleParams),
        ids: &[PermanentId],
    ) {
        for &h in ids {
            let Some(&hd) = self.hh_index.get(&h) else {
                continue;
            };
            let shelter = shelter_with(&self.letting.lets, land, catalog, params, h);
            if let Some(x) = self.households.get_mut(hd) {
                let members = x.members.len().max(1);
                x.settle_stores(now, &catalog.goods, &|d| fuel_per_day(params, members, d));
                (x.sheltered, x.keeping) = shelter;
            }
        }
    }

    /// Ends let `i` (ADR-0040 §3): its lodger goes home, to a roofed dwelling of its own if it
    /// has one, else where it was or is building; both households' goods are kept as their roofs
    /// now allow.
    fn end_let(&mut self, now: SimTime, rules: (&Land, &Catalog, &PeopleParams), i: usize) {
        let (land, catalog, _) = rules;
        // Their goods are kept as they were until now, then as their roofs now allow.
        let l = self.letting.lets[i].clone();
        self.letting.lets.remove(i);
        let own = land
            .buildings
            .iter()
            .filter(|b| {
                b.household == l.lodger && b.roofed() && catalog.is_dwelling(&b.spec.program)
            })
            .map(|b| build::centre_m(&b.spec))
            .next();
        if let Some(x) = self
            .hh_index
            .get(&l.lodger)
            .and_then(|&hd| self.households.get_mut(hd))
        {
            x.home = own.unwrap_or(l.own_home);
            self.homes_moved += 1;
        }
        self.reshelter(now, rules, &[l.lodger, l.landlord]);
    }

    /// Ends every let whose building is gone, unroofed or no longer its landlord's, or whose
    /// landlord or lodger is gone (ADR-0040 §3: at once).
    pub(crate) fn lets_standing(
        &mut self,
        now: SimTime,
        land: &Land,
        catalog: &Catalog,
        params: &PeopleParams,
    ) {
        let mut i = self.letting.lets.len();
        while i > 0 {
            i -= 1;
            let l = &self.letting.lets[i];
            let living = |h: PermanentId| self.household(h).is_some_and(|x| !x.members.is_empty());
            let stands = land
                .buildings
                .iter()
                .any(|b| b.id == l.building && b.household == l.landlord && b.roofed());
            if !(stands && living(l.landlord) && living(l.lodger)) {
                self.end_let(now, (land, catalog, params), i);
            }
        }
    }

    /// The midnight's letting (see the module documentation).
    pub(super) fn letting_day(&mut self, ctx: &mut Ctx, day: i64) {
        self.letting_with(ctx, day, None);
    }

    /// [`Population::letting_day`], with households `seek` reviewing today, when given, in place
    /// of those whose day it is.
    fn letting_with(&mut self, ctx: &mut Ctx, day: i64, seek: Option<&[PermanentId]>) {
        let (now, params) = (ctx.now, ctx.params);
        let rules = (&*ctx.land, ctx.catalog, params);
        self.letting.begun.retain(|b| day - b.day < RECENT_DAYS);
        // A household gone asks nothing.
        if !self.letting.asking.is_empty() {
            let (index, households) = (&self.hh_index, &self.households);
            self.letting.asking.retain(|h, _| {
                index
                    .get(h)
                    .and_then(|&hd| households.get(hd))
                    .is_some_and(|x| !x.members.is_empty())
            });
        }
        if !self.letting.lets.is_empty() {
            self.lets_standing(now, rules.0, rules.1, rules.2);
            self.rent_due(ctx, day);
            self.let_reviews(ctx, day);
        }
        // Who seeks a room today, and the settlements where some are roofless.
        let review_days = i64::from(params.market.review_days.max(1));
        let mut roofless: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| !x.members.is_empty() && !x.sheltered)
            .filter_map(|x| x.settlement)
            .collect();
        if roofless.is_empty() {
            return;
        }
        roofless.sort_unstable();
        roofless.dedup();
        let today = |x: &Household| match seek {
            Some(seek) => seek.contains(&x.id),
            None => (x.id.get() as i64 + day).rem_euclid(review_days) == 0,
        };
        let seekers: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| {
                x.settlement.is_some()
                    && !x.members.is_empty()
                    && !x.sheltered
                    && today(x)
                    && !self.lodges(x.id)
                    && !self.roof_nearly_on(ctx.land, ctx.catalog, x.id)
            })
            .map(|x| x.id)
            .collect();
        // The homes with spare floor where someone seeks a room today, or whose household
        // reviews its asking rent today, as they stand at the day's start.
        let mut sought: Vec<PermanentId> = seekers
            .iter()
            .filter_map(|&h| self.household(h).and_then(|x| x.settlement))
            .collect();
        sought.sort_unstable();
        sought.dedup();
        let mut dwellings: BTreeMap<PermanentId, Vec<&Building>> = BTreeMap::new();
        for b in ctx
            .land
            .buildings
            .iter()
            .filter(|b| b.finished() && b.roofed() && ctx.catalog.is_dwelling(&b.spec.program))
        {
            dwellings.entry(b.household).or_default().push(b);
        }
        let mut spare: Vec<Spare> = Vec::new();
        let mut vacant: Vec<PermanentId> = Vec::new();
        for (_, x) in self.households.iter() {
            let Some(s) = x.settlement.filter(|s| roofless.binary_search(s).is_ok()) else {
                continue;
            };
            if x.members.is_empty() || !(today(x) || sought.binary_search(&s).is_ok()) {
                continue;
            }
            let Some(homes) = dwellings.get(&x.id) else {
                continue;
            };
            let Some(home) = self.let_home(ctx.land, ctx.catalog, x, homes) else {
                continue;
            };
            let m2 = self.spare_m2(x, &home, None);
            if m2 + 1e-9 >= home.per_sleeper_m2 {
                spare.push((x.id, s, home, m2));
                if today(x) && self.letting.letting(x.id).next().is_none() {
                    vacant.push(x.id);
                }
            }
        }
        spare.sort_by_key(|s| s.0);
        let step = params.letting.step.max(0.0);
        let prior = params.letting.asking_rent_food_days_per_person_month;
        // Vacancy: a step less (ADR-0040 §2).
        vacant.sort_unstable();
        for h in vacant {
            let a = self.letting.asking_of(h, &params.letting);
            self.set_asking(h, (a - step).max(0.0), prior);
        }
        let mut seekers = seekers;
        seekers.sort_unstable();
        for h in seekers {
            let Some((landlord, home, rent_kcal, good)) = self.room_choice(ctx, h, &spare) else {
                continue;
            };
            let (Some(x), Some(lord)) = (self.household(h), self.household(landlord)) else {
                continue;
            };
            let need = home.per_sleeper_m2 * x.members.len() as f64;
            // Turned away for want of room: the landlord asks a step more.
            if self.spare_m2(lord, &home, None) + 1e-9 < need {
                let a = self.letting.asking_of(landlord, &params.letting);
                self.set_asking(landlord, a + step, prior);
                continue;
            }
            self.begin_let(ctx, (h, landlord), home, (good, rent_kcal), day);
        }
    }

    /// Sets household `h`'s asking rent to `a`, forgetting it where it is the prior again.
    fn set_asking(&mut self, h: PermanentId, a: f64, prior: f64) {
        if (a - prior).abs() <= 1e-9 {
            self.letting.asking.remove(&h);
        } else {
            self.letting.asking.insert(h, a);
        }
    }

    /// Whether household `h`'s own roof is nearly on: a dwelling of its own has its roof's
    /// stage under way (ADR-0040 §2: not within days of finishing one).
    fn roof_nearly_on(&self, land: &Land, catalog: &Catalog, h: PermanentId) -> bool {
        land.buildings.iter().any(|b| {
            b.household == h
                && b.standing()
                && b.stage() == Some(Stage::Roof)
                && catalog.is_dwelling(&b.spec.program)
        })
    }

    /// The food household `x` pays rent from: what it can eat that is not seed kept back, in
    /// catalog order with `first` first, each with its kilograms now.
    fn rent_foods(goods: &[GoodDef], stores: &[f64], first: usize) -> Vec<(usize, f64)> {
        let ok = |g: usize| goods.get(g).is_some_and(|d| d.edible() && !d.kept_back());
        std::iter::once(first)
            .chain((0..goods.len()).filter(|&g| g != first))
            .filter(|&g| ok(g))
            .map(|g| (g, stores.get(g).copied().unwrap_or(0.0).max(0.0)))
            .collect()
    }

    /// What household `x` pays of `kcal` from its food, kilograms by good (the let's `good`
    /// first), or `None` when its food does not cover it.
    fn rent_legs(
        goods: &[GoodDef],
        stores: &[f64],
        good: usize,
        kcal: f64,
    ) -> Option<Vec<(usize, f64)>> {
        let mut left = kcal;
        let mut out = Vec::new();
        for (g, kg) in Self::rent_foods(goods, stores, good) {
            if left <= 1e-9 {
                break;
            }
            let per = goods[g].kcal_per_kg;
            let take = (left / per).min(kg);
            if take > 1e-9 {
                out.push((g, take));
                left -= take * per;
            }
        }
        (left <= 1e-6 * kcal.max(1.0)).then_some(out)
    }

    /// Pays a month's rent of let `i` from its lodger to its landlord on `Rent`, if its food
    /// covers it.
    fn pay_rent(&mut self, ctx: &Ctx, i: usize) -> bool {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let l = &self.letting.lets[i];
        let (lodger, landlord, good) = (l.lodger, l.landlord, l.good);
        let kcal = l.rent_kg * goods.get(good).map_or(0.0, |d| d.kcal_per_kg);
        if kcal <= 0.0 {
            return true;
        }
        let Some(x) = self.household(lodger) else {
            return false;
        };
        let stores = stores_now(x, now, params, goods);
        let Some(legs) = Self::rent_legs(goods, &stores, good, kcal) else {
            return false;
        };
        let legs: Vec<Leg> = legs
            .into_iter()
            .map(|(good, amount)| Leg {
                from: lodger,
                to: landlord,
                good,
                amount,
            })
            .collect();
        self.transfer(now, params, goods, &legs, Channel::Rent)
    }

    /// Rent falling due today (ADR-0040 §3).
    fn rent_due(&mut self, ctx: &Ctx, day: i64) {
        let rules = (&*ctx.land, ctx.catalog, ctx.params);
        let mut i = self.letting.lets.len();
        while i > 0 {
            i -= 1;
            if day < self.letting.lets[i].due {
                continue;
            }
            let paid = self.pay_rent(ctx, i);
            let l = &mut self.letting.lets[i];
            l.due += RENT_DAYS;
            if !paid {
                l.unpaid = l.unpaid.saturating_add(1);
                if l.unpaid >= ARREARS_MONTHS {
                    self.end_let(ctx.now, rules, i);
                }
            }
        }
    }

    /// Lets whose review it is (ADR-0040 §3): one ends when its lodger has a roof of its own or
    /// its landlord's members need the floor.
    fn let_reviews(&mut self, ctx: &Ctx, day: i64) {
        let rules = (&*ctx.land, ctx.catalog, ctx.params);
        let review_days = i64::from(ctx.params.market.review_days.max(1));
        let mut i = self.letting.lets.len();
        while i > 0 {
            i -= 1;
            let l = &self.letting.lets[i];
            if day < l.next_review {
                continue;
            }
            let own_roof = shelter_of(rules.0, rules.1, rules.2, l.lodger).0;
            let needed = self.household(l.landlord).is_none_or(|lord| {
                self.let_home_of(rules.0, rules.1, lord, l.building)
                    .is_none_or(|home| {
                        self.spare_m2(lord, &home, Some(l.lodger)) + 1e-9 < l.floor_m2
                    })
            });
            if own_roof || needed {
                self.end_let(ctx.now, rules, i);
            } else {
                self.letting.lets[i].next_review = day + review_days;
            }
        }
    }

    /// Household `x`'s building `building` as a home it lets in, if it is a roofed, finished
    /// dwelling of its.
    fn let_home_of(
        &self,
        land: &Land,
        catalog: &Catalog,
        x: &Household,
        building: PermanentId,
    ) -> Option<LetHome> {
        let b = land
            .buildings
            .iter()
            .find(|b| b.id == building && b.household == x.id && b.finished() && b.roofed())?;
        let def = catalog
            .building_index(&b.spec.program)
            .and_then(|i| catalog.buildings.get(i))?;
        let (living, by_storey) = self.living_floor(land, catalog, b, def)?;
        let (base, per) = sleeping(def);
        let home = LetHome {
            building: b.id,
            living_m2: living,
            per_sleeper_m2: per,
            base_m2: base,
            centre: build::centre_m(&b.spec),
            let_storey: 0,
        };
        Some(self.with_let_storey(x, home, &by_storey))
    }

    /// The room household `h` would take, if any (ADR-0040 §2): of up to `candidates` homes of
    /// its settlement in `spare` within its reach and with spare floor for its members, nearest
    /// first, the one where a month's rent at its landlord's asking and the change in its walks
    /// cost it least below what being roofless costs it a month, in hours. Its landlord, the
    /// home, the rent (kcal a month) and the good it is reckoned in.
    fn room_choice(
        &self,
        ctx: &Ctx,
        h: PermanentId,
        spare: &[Spare],
    ) -> Option<(PermanentId, LetHome, f64, usize)> {
        let (now, params) = (ctx.now, ctx.params);
        let goods = &ctx.catalog.goods;
        let x = self.household(h)?;
        let settlement = x.settlement?;
        let n = x.members.len();
        let reach = self.homes.get(&settlement).map(|f| &f.reach);
        let max_s = params.farm.max_walk_minutes * 60.0;
        let mut near: Vec<(f64, &Spare)> = spare
            .iter()
            .filter(|c| c.0 != h && c.1 == settlement)
            .filter(|c| c.3 + 1e-9 >= c.2.per_sleeper_m2 * n as f64)
            .filter(|c| {
                reach.is_none_or(|r| {
                    r.seconds_to(cell_of(ctx.map, c.2.centre))
                        .is_some_and(|s| f64::from(s) <= max_s)
                })
            })
            .map(|c| {
                let d =
                    f64::from(c.2.centre.0 - x.home.0).hypot(f64::from(c.2.centre.1 - x.home.1));
                (d, c)
            })
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.0.cmp(&b.1.0)));
        near.truncate(params.letting.candidates.max(1) as usize);
        if near.is_empty() {
            return None;
        }
        let hour_kcal = self.hour_kcal(ctx, x);
        if hour_kcal <= 0.0 {
            return None;
        }
        let stores = stores_now(x, now, params, goods);
        // The rent is reckoned in the food it holds most energy in.
        let good = goods
            .iter()
            .enumerate()
            .filter(|(_, d)| d.edible() && !d.kept_back())
            .max_by(|a, b| {
                let kcal = |(g, d): &(usize, &GoodDef)| {
                    stores.get(*g).copied().unwrap_or(0.0).max(0.0) * d.kcal_per_kg
                };
                kcal(a).total_cmp(&kcal(b)).then(b.0.cmp(&a.0))
            })
            .map(|(g, _)| g)?;
        let costs: Vec<f64> = self
            .own_costs_of(ctx, x)
            .into_iter()
            .map(|c| c.unwrap_or(0.0))
            .collect();
        // The custody fee it pays a month now (ADR-0035), saved while it lodges.
        let fee_h: f64 = self
            .kept
            .iter()
            .filter(|k| k.depositor == h && !k.noted)
            .filter(|k| self.kin_households(x).binary_search(&k.keeper).is_err())
            .map(|k| {
                params.market.custody_fee_share
                    * k.stores
                        .iter()
                        .zip(&costs)
                        .map(|(kg, c)| kg.max(0.0) * c)
                        .sum::<f64>()
            })
            .sum();
        let places = self.goings_of(ctx, x);
        let speed = ctx.nav.params().top_speed_ms().max(0.1);
        let walks_now = siting::walk_hours_year(x.home, &places, speed);
        // Its trips out and back a year, each climbing to the room's storey (ADR-0042 §1).
        let trips_year: f64 = places.iter().map(|p| p.1).sum();
        let daily = params.household.daily_kcal_per_person * n as f64;
        let mut best: Option<(f64, PermanentId, LetHome, f64)> = None;
        for (_, &(landlord, _, home, _)) in near {
            let Some(b) = ctx.land.buildings.iter().find(|b| b.id == home.building) else {
                continue;
            };
            let rent_kcal = self.letting.asking_of(landlord, &params.letting) * daily;
            let rent_h = rent_kcal / hour_kcal;
            let walk_h = (siting::walk_hours_year(home.centre, &places, speed) - walks_now) / 12.0;
            let climb_h = self.room_climb_h_year(ctx, b, &home, n, trips_year) / 12.0;
            // What its goods would keep under the room's share of the roof over a month.
            let share = (home.per_sleeper_m2 * n as f64 / build::floor_m2(&b.spec).max(1e-9))
                .clamp(0.0, 1.0);
            let room = room_of(b, ctx.catalog).map(|r| r * share);
            let kept_h = build::store_saves_h(
                &stores,
                goods,
                &costs,
                room,
                params.household.raised_store_factor,
                RENT_DAYS as f64,
            );
            let net = rent_h + walk_h + climb_h - kept_h - fee_h;
            if net < 0.0 && best.as_ref().is_none_or(|b| net < b.0) {
                best = Some((net, landlord, home, rent_kcal));
            }
        }
        best.map(|(_, landlord, home, rent_kcal)| (landlord, home, rent_kcal, good))
    }

    /// Hours a year a lodging household of `n` members making `trips_year` trips would climb to
    /// a room in `home` (building `b`), on the storey its floor lies on, by the stair or its
    /// building's powered lift (ADR-0042 §1–§2).
    fn room_climb_h_year(
        &self,
        ctx: &Ctx,
        b: &Building,
        home: &LetHome,
        n: usize,
        trips_year: f64,
    ) -> f64 {
        let t = usize::from(home.let_storey);
        if t == 0 {
            return 0.0;
        }
        crate::climb::climb_h_year(&vec![t; n], trips_year, |t| self.trip_s_in(ctx, b, t))
    }

    /// Household `h` takes floor in `landlord`'s `home` for `rent_kcal` a month reckoned in
    /// `good`, paying the first month now (ADR-0040 §3), and moves its home there (§4).
    fn begin_let(
        &mut self,
        ctx: &Ctx,
        (h, landlord): (PermanentId, PermanentId),
        home: LetHome,
        (good, rent_kcal): (usize, f64),
        day: i64,
    ) {
        let rules = (&*ctx.land, ctx.catalog, ctx.params);
        let goods = &ctx.catalog.goods;
        let Some(x) = self.household(h) else {
            return;
        };
        let floor_m2 = home.per_sleeper_m2 * x.members.len() as f64;
        let per_kg = goods.get(good).map_or(0.0, |d| d.kcal_per_kg);
        if per_kg <= 0.0 || floor_m2 <= 0.0 {
            return;
        }
        let own_home = x.home;
        let review_days = i64::from(ctx.params.market.review_days.max(1));
        let settlement = self.household(landlord).and_then(|l| l.settlement);
        self.letting.lets.push(Let {
            landlord,
            lodger: h,
            building: home.building,
            floor_m2,
            good,
            rent_kg: rent_kcal / per_kg,
            begun: day,
            due: day + RENT_DAYS,
            next_review: day + review_days,
            unpaid: 0,
            own_home,
        });
        let i = self.letting.lets.len() - 1;
        // The first month is paid as it begins; a household that cannot pay it lodges nowhere.
        if !self.pay_rent(ctx, i) {
            self.letting.lets.pop();
            return;
        }
        if let Some(s) = settlement {
            self.letting.begun.push(LetBegun {
                day,
                settlement: s,
                kcal_per_m2_month: rent_kcal / floor_m2,
            });
        }
        // Its goods kept as they were until now, then under the landlord's roof; its home there.
        if let Some(x) = self
            .hh_index
            .get(&h)
            .and_then(|&hd| self.households.get_mut(hd))
        {
            x.home = home.centre;
            self.homes_moved += 1;
        }
        self.reshelter(ctx.now, rules, &[h, landlord]);
    }

    /// The yearly hours household `hh` would count a home's `spare_m2` of floor as letting for
    /// (ADR-0040 §5): its settlement's mean rent of lets begun in the last year, none if none,
    /// times the share of its households that are roofless, by its `hour_kcal`.
    pub(super) fn rent_credit_h_year(&self, ctx: &Ctx, hh: &Household, spare_m2: f64) -> f64 {
        let Some(s) = hh.settlement else {
            return 0.0;
        };
        let rent = self.letting.recent_rent(s, ctx.now.day_index());
        if rent.is_none() || spare_m2 <= 0.0 {
            return 0.0;
        }
        let (mut all, mut roofless) = (0usize, 0usize);
        for x in self.households_in(s).filter(|x| !x.members.is_empty()) {
            all += 1;
            if !x.sheltered {
                roofless += 1;
            }
        }
        let share = if all > 0 {
            roofless as f64 / all as f64
        } else {
            0.0
        };
        crate::letting::rent_credit_h_year(rent, spare_m2, share, self.hour_kcal(ctx, hh))
    }

    /// The midnight's letting now, as at midnight, with households `seek` reviewing (seeking a
    /// room, or asking less for one) whatever their day: for tests.
    #[doc(hidden)]
    pub fn letting_day_for_tests(&mut self, ctx: &mut Ctx, seek: &[PermanentId]) {
        let day = ctx.now.day_index();
        self.letting_with(ctx, day, Some(seek));
    }

    /// Building `building`'s sleeping floor, in all and by storey, less a lift's shaft: for tests.
    #[doc(hidden)]
    pub fn living_floor_for_tests(
        &self,
        ctx: &Ctx,
        building: PermanentId,
    ) -> Option<(f64, Vec<f64>)> {
        let b = ctx.land.buildings.iter().find(|b| b.id == building)?;
        let def = ctx
            .catalog
            .building_index(&b.spec.program)
            .and_then(|i| ctx.catalog.buildings.get(i))?;
        self.living_floor(ctx.land, ctx.catalog, b, def)
    }

    /// The hours a year household `household` would count `spare_m2` of a home's floor as
    /// letting for: for tests.
    #[doc(hidden)]
    pub fn rent_credit_for_tests(&self, ctx: &Ctx, household: PermanentId, spare_m2: f64) -> f64 {
        self.household(household)
            .map_or(0.0, |x| self.rent_credit_h_year(ctx, x, spare_m2))
    }
}
