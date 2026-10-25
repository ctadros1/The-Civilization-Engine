//! Streets lit at night (M10b slice DY, fourth part, third step; ADR-0034 §9; brief §2): where
//! something was taken from a household after dark and nobody saw who took it, and someone of the
//! settlement knows how to make a light, anyone may propose that the polity light its streets, as a
//! watch is proposed. Passed, it names a lamplighter and sets a street lamp on a public way beside
//! each home; lit streets let those about see after dark as a lit home does.

use super::*;
use crate::crime::Outcome;
use crate::decide::WatchFacts;
use civ_land::gas::StreetLamp;
use civ_land::ways::Keeper;

impl Population {
    /// The households of `households` something was taken from after dark within the year, with
    /// nobody seeing who took it, and whose people found it missing.
    pub(super) fn dark_taken(&self, ctx: &Ctx, households: &[PermanentId]) -> Vec<PermanentId> {
        let since = ctx.now.day_index() - DAYS_PER_YEAR;
        let mut out: Vec<PermanentId> = self
            .order
            .amounts
            .iter()
            .rev()
            .take_while(|a| a.day >= since)
            .filter(|a| a.lost && households.contains(&a.household))
            .filter(|a| {
                self.order.incident(a.incident).is_some_and(|i| {
                    i.outcome == Outcome::Taken
                        && i.noticed
                        && i.seen_by.is_empty()
                        && dark_at(ctx.params, i.at)
                })
            })
            .map(|a| a.household)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Whether someone of `households` knows how to make a light (a recipe making a lamp whose
    /// technique one of them knows, or that asks none).
    pub(super) fn knows_a_light(&self, ctx: &Ctx, households: &[PermanentId]) -> bool {
        let goods = &ctx.catalog.goods;
        ctx.catalog.recipes.iter().any(|r| {
            r.outputs.iter().any(|&(o, _)| {
                goods
                    .get(o)
                    .is_some_and(|d| d.light.is_some_and(|l| !l.piped))
            }) && households.iter().any(|&h| {
                self.household(h).is_some_and(|x| {
                    r.technique
                        .is_none_or(|t| self.household_knows(&x.members, t))
                })
            })
        })
    }

    /// Where polity `pi` would set its street lamps: for each home of its settlement, in the
    /// order of its households' ids, the cell of a public way (kept by nobody or by the polity)
    /// nearest beside it, within a cell each way; each cell once, and none where one stands.
    pub(super) fn street_lamp_sites(&self, ctx: &Ctx, pi: usize) -> Vec<u32> {
        let polity = &self.polities[pi];
        let settlement = polity.settlement;
        let w = i64::from(ctx.map.width.max(1));
        let public: Vec<u32> = {
            let mut cells: Vec<u32> = ctx
                .land
                .ways
                .list
                .iter()
                .flat_map(|way| way.segments.iter())
                .filter(|s| {
                    matches!(s.keeper, Keeper::Nobody) || s.keeper == Keeper::Polity(polity.id)
                })
                .flat_map(|s| s.cells.iter().copied())
                .collect();
            cells.sort_unstable();
            cells.dedup();
            cells
        };
        let mut homes: Vec<(PermanentId, u32)> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
            .map(|(_, x)| (x.id, cell_of(ctx.map, x.home) as u32))
            .collect();
        homes.sort_unstable();
        let mut out = Vec::new();
        for (_, home) in homes {
            let (hx, hy) = (i64::from(home) % w, i64::from(home) / w);
            let nearest = public
                .iter()
                .copied()
                .filter(|&c| {
                    (i64::from(c) % w - hx).abs() <= 1 && (i64::from(c) / w - hy).abs() <= 1
                })
                .min_by_key(|&c| {
                    (
                        (i64::from(c) % w - hx).abs() + (i64::from(c) / w - hy).abs(),
                        c,
                    )
                });
            if let Some(c) = nearest
                && !out.contains(&c)
                && !ctx.land.gas.lamps.iter().any(|l| l.cell == c)
            {
                out.push(c);
            }
        }
        out
    }

    /// Whether the settlement of polity `pi`, its households `households`, has the issue
    /// `dark_takings`: something taken from one of them after dark unseen, someone of them knowing
    /// how to make a light, and somewhere to set a lamp.
    pub(super) fn dark_takings(&self, ctx: &Ctx, pi: usize, households: &[PermanentId]) -> bool {
        !self.dark_taken(ctx, households).is_empty()
            && self.knows_a_light(ctx, households)
            && !self.street_lamp_sites(ctx, pi).is_empty()
    }

    /// A law to light the streets passed in polity `pi` sets its lamps at once, unlit.
    pub(super) fn begin_street_lamps(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        let polity = self.polities[pi].id;
        for cell in self.street_lamp_sites(ctx, pi) {
            ctx.land.gas.lamps.push(StreetLamp {
                cell,
                polity,
                law,
                lit_until: -1,
            });
        }
    }

    /// The round of its street lamps the lamplighter `person` of household `hh` could walk now
    /// (M10b slice DY, fourth part, third step's second half): after dark, while lamps of the law
    /// naming them stand unlit, and a night's fuel for one of them lies in the polity's store or
    /// their household's. Each unlit lamp is a stop, in the order set; the round is worth
    /// `w_light` points.
    pub(super) fn light_facts(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        hh: &Household,
        dark: bool,
    ) -> Option<WatchFacts> {
        if !dark || ctx.land.gas.lamps.is_empty() {
            return None;
        }
        let pi = self.polity_of(hh.settlement?)?;
        let polity = &self.polities[pi];
        let (holder, law) = polity.lamplighter()?;
        if holder != person {
            return None;
        }
        let now = ctx.now.minutes();
        let (fuel, kg_h) = street_fuel(&ctx.catalog.goods)?;
        let night_kg = kg_h * (dawn_after(ctx.params, ctx.now) - now) as f64 / 60.0;
        let held = polity.stores.get(fuel).copied().unwrap_or(0.0)
            + hh.stores.get(fuel).copied().unwrap_or(0.0);
        if held + 1e-9 < night_kg {
            return None;
        }
        let stops: Vec<(f32, f32)> = ctx
            .land
            .gas
            .lamps
            .iter()
            .filter(|l| l.polity == polity.id && l.law == law.id && l.lit_until <= now)
            .map(|l| cell_centre(ctx.map, l.cell as usize))
            .collect();
        if stops.is_empty() {
            return None;
        }
        Some(WatchFacts {
            stops,
            points: ctx.params.crime.w_light,
        })
    }

    /// The lamplighter `who`, standing at `pos`, lights the unlit lamp of their polity's law
    /// there until dawn (M10b slice DY, fourth part): its fuel for the night at a lamp's rate,
    /// from the polity's store, the rest from their own household's; a lamp whose night's fuel
    /// is not there stays dark. The world's first lamp lit is remembered, as a lamp lit at home
    /// is.
    pub(super) fn light_lamp(&mut self, ctx: &mut Ctx, who: PermanentId, pos: (f32, f32)) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(household) = self.person(who).map(|p| p.household) else {
            return;
        };
        let Some(pi) = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return;
        };
        let Some((polity, law)) = self.polities[pi]
            .lamplighter()
            .filter(|(h, _)| *h == who)
            .map(|(_, l)| (self.polities[pi].id, l.id))
        else {
            return;
        };
        let Some((fuel, kg_h)) = street_fuel(goods) else {
            return;
        };
        let cell = cell_of(ctx.map, pos) as u32;
        let Some(i) = ctx.land.gas.lamps.iter().position(|l| {
            l.cell == cell && l.polity == polity && l.law == law && l.lit_until <= now.minutes()
        }) else {
            return;
        };
        let until = dawn_after(params, now);
        let need = kg_h * (until - now.minutes()) as f64 / 60.0;
        // The polity's store first, then the lamplighter's household's.
        let sheltered = self.polity_sheltered(pi);
        let p = &mut self.polities[pi];
        p.settle_stores(now, goods, sheltered);
        p.stores.resize(goods.len(), 0.0);
        let from_polity = p.stores[fuel].max(0.0).min(need);
        let held = match self.household_mut(household) {
            Some(x) => {
                let members = x.members.len().max(1);
                x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
                x.stores.resize(goods.len(), 0.0);
                x.stores[fuel].max(0.0)
            }
            None => 0.0,
        };
        if from_polity + held + 1e-9 < need {
            return;
        }
        let from_home = need - from_polity;
        let p = &mut self.polities[pi];
        p.stores[fuel] -= from_polity;
        p.flows.add(Flow::Used, fuel, from_polity);
        if from_home > 0.0
            && let Some(x) = self.household_mut(household)
        {
            x.stores[fuel] = (x.stores[fuel] - from_home).max(0.0);
            x.flows.add(Flow::Used, fuel, from_home);
        }
        ctx.land.gas.lamps[i].lit_until = until;
        if self.order.lamp_first_lit.is_none() {
            self.order.lamp_first_lit = Some(now);
        }
    }

    /// The round of the street lamps `person` of household `household` could walk now: for
    /// tests.
    #[doc(hidden)]
    pub fn light_facts_for_tests(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        household: PermanentId,
    ) -> Option<WatchFacts> {
        let hh = self.household(household)?;
        self.light_facts(ctx, person, hh, dark_at(ctx.params, ctx.now))
    }

    /// The lamplighter `who` standing at `pos` lights the lamp there: for tests.
    #[doc(hidden)]
    pub fn light_lamp_for_tests(&mut self, ctx: &mut Ctx, who: PermanentId, pos: (f32, f32)) {
        self.light_lamp(ctx, who, pos);
    }

    /// Whether polity `pi`'s settlement has the issue `dark_takings` now: for tests.
    #[doc(hidden)]
    pub fn dark_takings_for_tests(&self, ctx: &Ctx, pi: usize) -> bool {
        let settlement = self.polities[pi].settlement;
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement))
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        self.dark_takings(ctx, pi, &households)
    }

    /// Where polity `pi` would set its street lamps: for tests.
    #[doc(hidden)]
    pub fn street_lamp_sites_for_tests(&self, ctx: &Ctx, pi: usize) -> Vec<u32> {
        self.street_lamp_sites(ctx, pi)
    }

    /// Sets polity `pi`'s street lamps as a law `law` to light its streets passed would: for
    /// tests.
    #[doc(hidden)]
    pub fn begin_street_lamps_for_tests(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        self.begin_street_lamps(ctx, pi, law);
    }
}

/// The fuel a street lamp burns and its kilograms an hour (M10b slice DY, fourth part): those of
/// the first light the catalog lists that a household fills (the fat lamp in the core).
fn street_fuel(goods: &[crate::params::GoodDef]) -> Option<(usize, f64)> {
    goods.iter().find_map(|d| {
        let l = d.light.filter(|l| !l.piped && l.kg_per_hour > 0.0)?;
        Some((l.fuel?, l.kg_per_hour))
    })
}

/// The minute of the next sunrise after `now`, at the profile's latitude: today's while it is
/// before today's, else tomorrow's.
fn dawn_after(params: &PeopleParams, now: SimTime) -> i64 {
    let day = now.day_index();
    let today = needs::daylight(params.latitude_deg, civ_land::day_of_year(day)).0;
    let minute = now.minutes().rem_euclid(MINUTES_PER_DAY);
    if minute < today {
        day * MINUTES_PER_DAY + today
    } else {
        let tomorrow = needs::daylight(params.latitude_deg, civ_land::day_of_year(day + 1)).0;
        (day + 1) * MINUTES_PER_DAY + tomorrow
    }
}
