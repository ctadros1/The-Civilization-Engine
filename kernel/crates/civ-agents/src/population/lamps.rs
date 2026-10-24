//! Streets lit at night (M10b slice DY, fourth part, third step; ADR-0034 §9; brief §2): where
//! something was taken from a household after dark and nobody saw who took it, and someone of the
//! settlement knows how to make a light, anyone may propose that the polity light its streets, as a
//! watch is proposed. Passed, it names a lamplighter and sets a street lamp on a public way beside
//! each home; lit streets let those about see after dark as a lit home does.

use super::*;
use crate::crime::Outcome;
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
