//! Counting travel as it happens (M8b slice CR, first part; see [`crate::travel`]): each trip
//! as it sets off, each wait at a bottleneck, and at midnight each resident's day.

use super::*;
use crate::travel::{Mode, Occupation};

/// The age below which someone's travel is the young's, years (a design prior, the age activities
/// begin to ask hard work at).
const YOUNG_YEARS: f64 = 14.0;
/// Within this of their household's home, metres, a walk sets off from home (a home's plot is a
/// few metres a side, and its door faces the hearth).
const FROM_HOME_M: f32 = 12.0;

impl Population {
    /// The person `h` has just set off on a walk: one that leaves their household's home for
    /// somewhere beyond it is a trip from home to that place (M8c slice CS, first part).
    pub(super) fn count_going(&mut self, ctx: &Ctx, h: Handle<Person>) {
        let Some(p) = self.people.get(h) else {
            return;
        };
        let Some(home) = self.household(p.household).map(|x| x.home) else {
            return;
        };
        let Some((from, to)) = p
            .trip
            .as_ref()
            .and_then(|t| Some((*t.points.first()?, *t.points.last()?)))
        else {
            return;
        };
        let near = |a: (f32, f32)| (a.0 - home.0).hypot(a.1 - home.1) <= FROM_HOME_M;
        if !near(from) || near(to) {
            return;
        }
        let place = crate::travel::going_place(cell_of(ctx.map, to) as u32, ctx.map.width);
        let who = p.id;
        self.travel.went(who, place);
    }

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
        let residents: Vec<(PermanentId, PermanentId, PermanentId, Occupation)> = self
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
                Some((p.id, p.household, settlement, occupation))
            })
            .collect();
        let half = ctx.params.places.use_half_life_days;
        self.travel
            .close_day(year, yesterday.day_index(), half, residents);
        // The day's trips from home are held, so where a household builds weighs the same all
        // the next day (M8c slice CS).
        self.travel.hold_goings(yesterday.day_index(), half);
        // A household gone takes what it held of its days with it.
        let hh = &self.hh_index;
        self.travel.households.retain(|h, _| hh.contains_key(h));
        // Someone gone takes where they went with them.
        let people = &self.index;
        self.travel.goings.retain(|p, _| people.contains_key(p));
    }
}
