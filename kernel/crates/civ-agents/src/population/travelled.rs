//! Counting travel as it happens (M8b slice CR, first part; see [`crate::travel`]): each trip
//! as it sets off, each wait at a bottleneck, and at midnight each resident's day.

use super::*;
use crate::travel::{Mode, Occupation};

/// The age below which someone's travel is the young's, years (a design prior, the age activities
/// begin to ask hard work at).
const YOUNG_YEARS: f64 = 14.0;

impl Population {
    /// The person `h` set off by `mode` on a trip of `minutes`, for the activity they
    /// are about: counted in their settlement's year, if they have one.
    pub(super) fn count_trip(&mut self, ctx: &Ctx, h: Handle<Person>, mode: Mode, minutes: f32) {
        let Some(p) = self.people.get(h) else {
            return;
        };
        let Some(settlement) = self.household(p.household).and_then(|x| x.settlement) else {
            return;
        };
        let purpose = ctx
            .catalog
            .activities
            .get(usize::from(p.act.def))
            .and_then(|a| Behavior::ALL.iter().position(|&b| b == a.behavior))
            .map_or(u8::MAX, |i| i as u8);
        let (year, who) = (ctx.now.date().year, p.id);
        self.travel
            .trip(year, settlement, who, purpose, mode, minutes);
    }

    /// Someone of `household`, `person`, waited `minutes` at a bottleneck: `beyond` the trip
    /// they were counted on when it lengthened it.
    pub(super) fn count_wait(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        person: PermanentId,
        minutes: f32,
        beyond: bool,
    ) {
        let Some(settlement) = self.household(household).and_then(|x| x.settlement) else {
            return;
        };
        self.travel
            .waited(ctx.now.date().year, settlement, person, minutes, beyond);
    }

    /// Midnight: the day just over is counted for everyone living in a settlement, by what their
    /// household lives by.
    pub(super) fn close_travel_day(&mut self, ctx: &Ctx) {
        let yesterday = ctx.now.plus_minutes(-1);
        let year = yesterday.date().year;
        let mut workshops: Vec<PermanentId> = self
            .firms
            .iter()
            .filter(|f| f.closed.is_none())
            .map(|f| f.owner)
            .collect();
        workshops.sort_unstable();
        workshops.dedup();
        let mut farms: Vec<PermanentId> = ctx.land.fields.iter().map(|f| f.household).collect();
        farms.sort_unstable();
        farms.dedup();
        let residents: Vec<(PermanentId, PermanentId, Occupation)> = self
            .people
            .iter()
            .filter_map(|(_, p)| {
                let settlement = self.household(p.household)?.settlement?;
                let occupation = if p.age_years(yesterday) < YOUNG_YEARS {
                    Occupation::Young
                } else if workshops.binary_search(&p.household).is_ok() {
                    Occupation::Workshop
                } else if farms.binary_search(&p.household).is_ok() {
                    Occupation::Fields
                } else {
                    Occupation::Other
                };
                Some((p.id, settlement, occupation))
            })
            .collect();
        self.travel.close_day(year, residents);
    }
}
