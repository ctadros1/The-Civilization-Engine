//! Wells in the day and underground (M6a slice AY, step three; ADR-0021 §1, §3; research 03-02
//! §1.4, 12-01 §2.2-§3.1). Once a year a household weighs a well beside its home as it weighs a
//! log footbridge: the walking its fetching would save over the well's life against the hours to
//! dig and line it to the depth it expects water at, from what its people know (the water they
//! saw stand in the settlement's wells and the depth of those given up dry, else their height
//! above the water they draw from). The shaft is dug and lined a metre at a time; at the depth
//! its diggers meant it meets water if the true water table stands above its floor, or it is dug
//! a metre deeper, or, at the deepest its system goes, given up. An open well is drawn from by
//! its household and its kin's households, and by any household short of water; what is drawn is
//! taken from the water table at the day's end. Its lining rots by the wetness of its ground and
//! the month, and its household weighs relining it once rot has taken half of what its quality
//! left; when rot has taken it all, the shaft falls in.

use super::*;
use crate::decide::WellFacts;
use crate::history::WellStep;
use civ_land::RectCm;
use civ_land::buildings::PlotUse;
use civ_land::wells::{Well, WellState};

/// Keyed-randomness purpose of the day of the year a household weighs a well.
const PURPOSE_WELL_REVIEW: u64 = 0x7765_6c6c_7265_7677; // "wellrevw"
/// The share of what its quality left that rot takes before a household weighs relining its
/// well (a design prior: halfway to falling in).
const RELINE_AT: f32 = 0.5;
/// The farthest a well is sunk from the edge of its household's home plot, metres (a design
/// prior: beside the home).
const WELL_REACH_M: f64 = 8.0;
/// A household with less water at home than this many days' use is short of it, and may draw at
/// any well it knows (ADR-0021 §1: need over another's word; a design prior).
const SHORT_DAYS: f64 = 0.5;
/// The longest life a household reckons a lining for, years.
const LONGEST_LIFE_YEARS: f64 = 50.0;
/// Cells a household's people search for the walk from home to the water they draw from now.
const SOURCE_ROUTE_BUDGET: usize = 200_000;
/// The side of the ground at the hearth a polity's well is sunk beside, metres (M6a slice BA,
/// step four; a design prior: the gathering place, as a household's is sunk beside its home).
const HEARTH_SPOT_M: f64 = 4.0;

/// A well a polity's people could dig together beside the hearth (M6a slice BA, step four;
/// ADR-0021 §7), and what it would be worth to each of the settlement's households: the walking
/// a year it would save them, from the walk to the water each last fetched, as they walked it.
#[derive(Clone, Debug)]
pub(crate) struct PublicWell {
    /// What it would dig.
    pub(crate) site: crate::polity::WellSite,
    /// The years its lining would last, at the quality the settlement's people would lay it on
    /// average.
    pub(crate) life: f64,
    /// The hours of walking a year it would save each household, by household id.
    pub(crate) saves: Vec<(PermanentId, f64)>,
    /// How many households its work would be asked of.
    pub(crate) households: usize,
}

impl PublicWell {
    /// The share of its work asked of each household, hours.
    pub(crate) fn share_h(&self) -> f64 {
        f64::from(self.site.labour_h) / self.households.max(1) as f64
    }

    /// The hours of walking a year it would save household `h`.
    pub(crate) fn saves_of(&self, h: PermanentId) -> f64 {
        self.saves
            .binary_search_by_key(&h, |s| s.0)
            .map_or(0.0, |i| self.saves[i].1)
    }

    /// The hours of walking it would save all the households over its life, less its work.
    fn pooled_net_h(&self) -> f64 {
        self.saves.iter().map(|s| s.1).sum::<f64>() * self.life - f64::from(self.site.labour_h)
    }
}

/// A square of side `side` metres centred at `at` metres.
fn square(at: (f64, f64), side: f64) -> RectCm {
    let half = side / 2.0;
    RectCm {
        x: ((at.0 - half) * 100.0).round() as i32,
        y: ((at.1 - half) * 100.0).round() as i32,
        w: (side * 100.0).round() as i32,
        h: (side * 100.0).round() as i32,
    }
}

/// Where a well beside the home plot `plot` would go: a square of side `side` that could be
/// claimed as ground ([`crate::build::plot_clear`]), as near the plot as can be and in front of it
/// (toward `toward`, its settlement's hearth) before beside or behind it, within
/// [`WELL_REACH_M`]. `None` when there is no such ground.
fn well_site(ctx: &Ctx, plot: &RectCm, toward: (f64, f64), side: f64) -> Option<RectCm> {
    let (cx, cy) = plot.centre_m();
    let (cx, cy) = (f64::from(cx), f64::from(cy));
    let (hw, hh) = (f64::from(plot.w) / 200.0, f64::from(plot.h) / 200.0);
    let step = side / 2.0;
    let n = ((hw.max(hh) + side + WELL_REACH_M) / step).ceil() as i64;
    let (fx, fy) = (toward.0 - cx, toward.1 - cy);
    let reach = fx.hypot(fy);
    let mut spots: Vec<(i64, f64, (f64, f64))> = Vec::new();
    for j in -n..=n {
        for i in -n..=n {
            let at = (cx + i as f64 * step, cy + j as f64 * step);
            let (dx, dy) = (at.0 - cx, at.1 - cy);
            let gap = (dx.abs() - hw - side / 2.0)
                .max(0.0)
                .hypot((dy.abs() - hh - side / 2.0).max(0.0));
            if gap > WELL_REACH_M {
                continue;
            }
            let facing = if reach > 0.0 {
                (dx * fx + dy * fy) / (dx.hypot(dy).max(1e-9) * reach)
            } else {
                0.0
            };
            spots.push(((gap / step).floor() as i64, -facing, at));
        }
    }
    spots.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    spots
        .into_iter()
        .map(|(_, _, at)| square(at, side))
        .find(|r| crate::build::plot_clear(ctx.land, ctx.map, ctx.nav, r))
}

/// The water surface at the bank cell `cell` people draw from: the lowest of the river or lake
/// cells beside it, else its own ground.
fn water_beside(map: &WorldMap, cell: u32) -> f64 {
    let (w, h) = (map.width as i64, map.height as i64);
    let (x, y) = (i64::from(cell) % w, i64::from(cell) / w);
    let mut z = f64::from(map.elevation[cell as usize]);
    for &(dx, dy) in civ_world::grid::D8.iter() {
        let (nx, ny) = (x + i64::from(dx), y + i64::from(dy));
        if nx < 0 || ny < 0 || nx >= w || ny >= h {
            continue;
        }
        let n = (ny * w + nx) as usize;
        if matches!(map.water[n], WATER_RIVER | WATER_LAKE) {
            z = z.min(f64::from(map.elevation[n]));
        }
    }
    z
}

/// The habitat wetness of the ground at `cell`.
fn wetness_at(ctx: &Ctx, cell: u32) -> f64 {
    let patches = &ctx.land.patches;
    patches
        .class
        .get(patches.of_cell(cell as usize, ctx.map.width))
        .and_then(|&c| ctx.land_params.habitats.get(usize::from(c)))
        .map_or(1.0, |h| h.wetness)
}

/// The water table's head under `cell`, and the rate a column of well system `def` there refills
/// toward it a day.
fn ground_water(ctx: &Ctx, def: &crate::params::WellDef, cell: u32) -> (f64, f64) {
    ctx.land
        .ground_water(cell as usize, ctx.map.width, def.radius_m, def.influence_m)
}

/// What a household's weighing of a well came to (M6a slice AY, step three).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WellWeighed {
    /// Nothing to weigh: no such household, nobody in it, or no well system or work on one in
    /// content.
    Nothing,
    /// Work on one of its wells is owed already.
    Busy,
    /// Its open well's lining is sound enough to leave.
    Sound,
    /// It has no home plot to dig beside.
    NoHome,
    /// It knows of no water it draws from now.
    NoSource,
    /// Its people know no well system's technique.
    Unknown,
    /// No ground beside its home is free to dig.
    NoSite,
    /// It expects water deeper than any system it knows is dug.
    TooDeep { target_m: f64 },
    /// The walking it would save over the lining's life is less than the work.
    NotWorth { saved_h: f64, labour_h: f64 },
    /// It begins relining its well.
    Reline { saved_h: f64, labour_h: f64 },
    /// It begins a well.
    Begun { saved_h: f64, labour_h: f64 },
}

/// The water a household draws from now, other than one well: the walk to it from home and the
/// lift there, minutes, and the level its water stands at, metres.
struct Source {
    walk_min: f64,
    lift_min: f64,
    water_z: f64,
}

impl Source {
    /// Minutes a load takes there beyond filling the vessels: the walk there and back, and the
    /// lift.
    fn trip_min(&self) -> f64 {
        2.0 * self.walk_min + self.lift_min
    }
}

/// Minutes to haul a load up to the ground at a well of system `def` whose water stands
/// `depth_m` below it.
fn lift_min(def: &crate::params::WellDef, depth_m: f64) -> f64 {
    def.lift_min_per_m * depth_m.max(0.0)
}

impl Population {
    /// The activity of work on a well, if content has it and a well system.
    fn well_kit<'a>(ctx: &'a Ctx) -> Option<&'a crate::params::ActivityDef> {
        if ctx.catalog.wells.is_empty() {
            return None;
        }
        ctx.catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Well)
    }

    /// Seconds to walk from `from` to `to` in a straight line, if it can be walked.
    pub(super) fn well_walk_s(ctx: &Ctx, from: (f32, f32), to: (f32, f32)) -> Option<f64> {
        let wear = &ctx.land.wear;
        ctx.nav
            .segment_seconds(&ctx.map.elevation, from, to, &|c| wear.factor(c))
            .map(f64::from)
    }

    /// The households of `x`'s people's close kin, `x`'s own left out.
    fn kin_households(&self, x: &Household) -> Vec<PermanentId> {
        let mut out: Vec<PermanentId> = x
            .members
            .iter()
            .flat_map(|&m| self.close_kin(m))
            .filter_map(|k| self.person(k).map(|p| p.household))
            .filter(|&h| h != x.id)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Which wells household `x`'s people may draw at by right (ADR-0021 §1, rights v0): their
    /// own, their polity's and their kin's households'. Anyone short of water may draw at any;
    /// the Water lens keeps that access apart from this permission (M6a slice BB, step three).
    pub fn by_right(&self, x: &Household) -> impl Fn(&Well) -> bool + use<> {
        let ours = self.ours(x);
        let kin = self.kin_households(x);
        move |w: &Well| ours(w) || kin.binary_search(&w.household).is_ok()
    }

    /// Whether well `w` is household `x`'s own or its polity's (M6a slice BA, step four): the
    /// wells its people draw at by right.
    fn ours(&self, x: &Household) -> impl Fn(&Well) -> bool + use<> {
        let (own, polity) = (x.id, self.polity_id_of(x.settlement));
        move |w: &Well| w.household == own || (w.polity.is_some() && w.polity == polity)
    }

    /// The id of settlement `settlement`'s polity, if it has one.
    fn polity_id_of(&self, settlement: Option<PermanentId>) -> Option<PermanentId> {
        settlement
            .and_then(|s| self.polity_of(s))
            .map(|pi| self.polities[pi].id)
    }

    /// Whether well `w` is the polity's of settlement `settlement`.
    fn of_polity(&self, w: &Well, settlement: Option<PermanentId>) -> bool {
        w.polity.is_some() && w.polity == self.polity_id_of(settlement)
    }

    /// The open well people of household `x` may draw at that holds a load now and takes least
    /// time there and back with its lift (ADR-0021 §1, rights v0): their own, their kin's
    /// households', or, while `short` of water, any. Wells whose round trip would take longer
    /// than `bound_min` minutes cannot be chosen, so others' wells farther than that at the
    /// fastest walk are passed over before anyone's kin is looked up.
    pub(super) fn well_option<'a>(
        &self,
        ctx: &'a Ctx,
        x: &Household,
        short: bool,
        bound_min: f64,
        shunned: &dyn Fn(u32) -> bool,
    ) -> Option<WaterOption> {
        let wells = &ctx.land.wells.list;
        if !wells.iter().any(Well::is_open) {
            return None;
        }
        let ours = self.ours(x);
        let carry = ctx.params.household.carry_water_l;
        // (minutes a load takes there, walk, lift, well)
        let mut best: Option<(f64, f64, f64, &Well)> = None;
        let pick = |best: Option<(f64, f64, f64, &'a Well)>, w: &'a Well| match Self::well_trip(
            ctx, x.home, w, carry,
        ) {
            Some((trip, walk, lift))
                if best.is_none_or(|(b, _, _, bw)| trip < b || (trip == b && w.id < bw.id)) =>
            {
                Some((trip, walk, lift, w))
            }
            _ => best,
        };
        for w in wells
            .iter()
            .filter(|w| w.is_open() && ours(w) && !shunned(w.cell))
        {
            best = pick(best, w);
        }
        let bound = best.map_or(bound_min, |b| b.0.min(bound_min));
        let metres_a_minute = ctx.nav.params().top_speed_ms() * 60.0;
        let near: Vec<&Well> = wells
            .iter()
            .filter(|w| w.is_open() && !ours(w) && !shunned(w.cell))
            .filter(|w| {
                let at = w.rect.centre_m();
                let metres = f64::from((at.0 - x.home.0).hypot(at.1 - x.home.1));
                2.0 * metres / metres_a_minute.max(1e-9) <= bound + 1e-6
            })
            .collect();
        if !near.is_empty() {
            let kin = if short {
                Vec::new()
            } else {
                self.kin_households(x)
            };
            for w in near {
                if short || kin.binary_search(&w.household).is_ok() {
                    best = pick(best, w);
                }
            }
        }
        best.map(|(_, walk_min, lift_min, w)| WaterOption {
            cell: w.cell,
            walk_min,
            at: w.rect.centre_m(),
            well: Some(w.id),
            lift_min,
        })
    }

    /// The minutes a load takes from `home` at well `w` there and back with its lift, the walk
    /// and the lift, if it holds a load of `carry` litres now and can be walked to.
    fn well_trip(ctx: &Ctx, home: (f32, f32), w: &Well, carry: f64) -> Option<(f64, f64, f64)> {
        let def = ctx.catalog.wells.get(w.system)?;
        let (head, rate) = ground_water(ctx, def, w.cell);
        if w.litres(ctx.now, head, rate, def.area_m2()) < carry {
            return None;
        }
        let walk = Self::well_walk_s(ctx, home, w.rect.centre_m())? / 60.0;
        let lift = lift_min(
            def,
            f64::from(w.ground_m) - w.level_now(ctx.now, head, rate),
        );
        Some((2.0 * walk + lift, walk, lift))
    }

    /// Whether household `x`, holding `water_l` litres and using `water_day` a day, is short of
    /// water: it may then draw at any well it knows.
    pub(super) fn short_of_water(water_l: f64, water_day: f64) -> bool {
        water_l < SHORT_DAYS * water_day
    }

    /// Someone of `household` draws a load at well `well` now: what its column holds of a load,
    /// litres, logged as a use of it as a source (ADR-0021 §1).
    pub(super) fn drew_from_well(
        &mut self,
        ctx: &mut Ctx,
        well: PermanentId,
        household: PermanentId,
        who: PermanentId,
        now: SimTime,
    ) -> f64 {
        let carry = ctx.params.household.carry_water_l;
        let Some(i) = ctx.land.wells.list.iter().position(|w| w.id == well) else {
            return 0.0;
        };
        let w = ctx.land.wells.list[i];
        if !w.is_open() {
            return 0.0;
        }
        let Some(def) = ctx.catalog.wells.get(w.system) else {
            return 0.0;
        };
        let (head, rate) = ground_water(ctx, def, w.cell);
        let area = def.area_m2();
        let w = &mut ctx.land.wells.list[i];
        let held = w.litres(now, head, rate, area);
        let got = held.min(carry);
        if got > 0.0 {
            w.draw(now, head, rate, area, got);
        }
        let cell = w.cell;
        if got > 0.0 {
            self.drew_load_from_well(well, household, now.day_index(), got / held);
        }
        self.water_draws.wells += 1;
        self.water_draws.short += u64::from(got < carry);
        self.water_draws.litres += got;
        self.log_work(crate::uses::Place::Source(cell), household, who, now, 0.0);
        got
    }

    /// The wells' midnight: each open well's water is brought up to now, and what was drawn from
    /// it since is taken from the water table of its patch (ADR-0021 §2: wells draw on the
    /// aquifer, never on a supply of their own). A patch of open water is held at its water's
    /// level, which gives what is drawn there.
    pub(super) fn wells_day(&mut self, ctx: &mut Ctx) {
        let now = ctx.now;
        // A household whose people suspect its own well makes it a lid and curb (M6a slice BA,
        // ADR-0021 §7): the hours are owed until done.
        if !self.word.suspicions.is_empty() {
            for i in 0..ctx.land.wells.list.len() {
                let w = ctx.land.wells.list[i];
                if !w.is_open() || w.covered || w.cover_h > 0.0 || w.mend_h > 0.0 {
                    continue;
                }
                let Some(def) = ctx.catalog.wells.get(w.system) else {
                    continue;
                };
                let suspect = self.household(w.household).is_some_and(|x| {
                    self.suspected_sources(x)
                        .contains(&crate::uses::Place::Source(w.cell))
                });
                if suspect && def.cover_h > 0.0 {
                    ctx.land.wells.list[i].cover_h = def.cover_h as f32;
                }
            }
        }
        for i in 0..ctx.land.wells.list.len() {
            let w = ctx.land.wells.list[i];
            if !w.is_open() {
                continue;
            }
            let Some(def) = ctx.catalog.wells.get(w.system) else {
                continue;
            };
            let (head, rate) = ground_water(ctx, def, w.cell);
            let drawn = w.drawn_l;
            let well = &mut ctx.land.wells.list[i];
            well.settle(now, head, rate);
            well.drawn_l = 0.0;
            if drawn <= 0.0 {
                continue;
            }
            let p = ctx.land.patches.of_cell(w.cell as usize, ctx.map.width);
            let water = &mut ctx.land.water;
            let fixed = water.aquifer.stage.get(p).is_some_and(Option::is_some);
            let storage = water.aquifer.storage_m2.get(p).copied().unwrap_or(0.0);
            if !fixed && storage > 0.0 {
                water.heads[p] -= drawn / 1000.0 / storage;
            }
        }
    }

    /// A month of rot on every open well's lining, as wet as its ground and the month just lived
    /// were (as for posts' feet, ADR-0012 §5): one whose lining rot has taken all its quality
    /// left falls in.
    pub(super) fn wells_month(&mut self, ctx: &mut Ctx) {
        let rain = ctx
            .land
            .weather
            .last_month_wetness(&ctx.land.climatology)
            .unwrap_or(1.0)
            .clamp(condition::WET_WEAR.0, condition::WET_WEAR.1);
        for i in 0..ctx.land.wells.list.len() {
            let w = ctx.land.wells.list[i];
            if !w.is_open() {
                continue;
            }
            let Some(def) = ctx.catalog.wells.get(w.system) else {
                continue;
            };
            let lost = def.loss_per_year / 12.0 * wetness_at(ctx, w.cell) * rain;
            let well = &mut ctx.land.wells.list[i];
            well.loss = (well.loss + lost as f32).min(1.0);
            if well.loss + 1e-6 < well.quality {
                continue;
            }
            well.state = WellState::FellIn { on: ctx.now };
            well.mend_h = 0.0;
            let name = def.name.to_lowercase();
            let whose = match w.polity {
                Some(p) => self
                    .polities
                    .iter()
                    .find(|q| q.id == p)
                    .map(|q| format!("{}'s", self.village_name(ctx, Some(q.settlement)))),
                None => self
                    .elder_of(w.household, ctx.now, ctx.params)
                    .map(|e| format!("{}'s household's", self.name_of(e))),
            };
            let sentence = match whose {
                Some(whose) => {
                    format!("{whose} {name} fell in: its lining had rotted through.")
                }
                None => format!("A {name} fell in: its lining had rotted through."),
            };
            let settlement = match w.polity {
                Some(p) => self
                    .polities
                    .iter()
                    .find(|q| q.id == p)
                    .map(|q| q.settlement),
                None => self.household(w.household).and_then(|x| x.settlement),
            };
            self.chronicle_push(
                ctx.now,
                ChronicleKind::Well,
                Vec::new(),
                settlement,
                Some(w.rect.centre_m()),
                f64::from(WellStep::FellIn as u8),
                sentence,
            );
        }
    }

    /// Households whose day of the year it is weigh a well (M6a slice AY, step three).
    pub(super) fn well_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        if Self::well_kit(ctx).is_none() {
            return;
        }
        let due: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x.id)
            .filter(|h| {
                Rng64::from_key(&[ctx.seed, PURPOSE_WELL_REVIEW, h.get()]).next_u64()
                    % DAYS_PER_YEAR as u64
                    == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .collect();
        for h in due {
            self.review_well(ctx, h);
        }
    }

    /// Household `household` weighs a well now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_well_for_tests(&mut self, ctx: &mut Ctx, household: PermanentId) -> WellWeighed {
        self.review_well(ctx, household)
    }

    /// How skilled at building household `x`'s people old enough for work on a well are, on
    /// average: the skill well system `def` practises, else a middling builder's.
    fn well_skill(&self, ctx: &Ctx, x: &Household, def: &crate::params::WellDef) -> f64 {
        let Some(k) = def.skill else {
            return condition::MIDDLING_SKILL;
        };
        let min_age = Self::well_kit(ctx).map_or(0.0, |a| a.min_age_years);
        let levels: Vec<f64> = x
            .members
            .iter()
            .filter_map(|&m| self.person(m))
            .filter(|p| p.age_years(ctx.now) >= min_age)
            .map(|p| p.skill(k))
            .collect();
        if levels.is_empty() {
            condition::MIDDLING_SKILL
        } else {
            levels.iter().sum::<f64>() / levels.len() as f64
        }
    }

    /// Where household `x`'s people draw water now, other than well `except`: the nearer of the
    /// wells they may draw at and the spring or bank their settlement's people go to, with the
    /// walk there from home (along the ground) and the level its water stands at.
    fn other_source(
        &self,
        ctx: &Ctx,
        x: &Household,
        except: Option<PermanentId>,
    ) -> Option<Source> {
        let kin = self.kin_households(x);
        let ours = self.ours(x);
        let carry = ctx.params.household.carry_water_l;
        let mut best: Option<Source> = None;
        for w in ctx.land.wells.list.iter().filter(|w| w.is_open()) {
            if Some(w.id) == except || !(ours(w) || kin.binary_search(&w.household).is_ok()) {
                continue;
            }
            let Some(def) = ctx.catalog.wells.get(w.system) else {
                continue;
            };
            let (head, rate) = ground_water(ctx, def, w.cell);
            if w.litres(ctx.now, head, rate, def.area_m2()) < carry {
                continue;
            }
            let Some(s) = Self::well_walk_s(ctx, x.home, w.rect.centre_m()) else {
                continue;
            };
            let level = w.level_now(ctx.now, head, rate);
            let source = Source {
                walk_min: s / 60.0,
                lift_min: lift_min(def, f64::from(w.ground_m) - level),
                water_z: level,
            };
            if best
                .as_ref()
                .is_none_or(|b| source.trip_min() < b.trip_min())
            {
                best = Some(source);
            }
        }
        let key = x
            .settlement
            .filter(|s| self.homes.contains_key(s))
            .unwrap_or(x.id);
        let Some((cell, water_z)) = self.spring_or_bank_level(ctx, key) else {
            return best;
        };
        let wear = &ctx.land.wear;
        let from = cell_of(ctx.map, x.home);
        let walk_min = match ctx.nav.route_bounded(
            &ctx.map.elevation,
            from,
            cell as usize,
            &|c| wear.factor(c),
            wear.max_factor(),
            SOURCE_ROUTE_BUDGET,
        ) {
            RouteResult::Found(r) => f64::from(r.seconds.last().copied().unwrap_or(0.0)) / 60.0,
            RouteResult::Unreachable | RouteResult::BudgetExhausted => return best,
        };
        let source = Source {
            walk_min,
            lift_min: 0.0,
            water_z,
        };
        if best
            .as_ref()
            .is_none_or(|b| source.trip_min() < b.trip_min())
        {
            best = Some(source);
        }
        best
    }

    /// Settlement `settlement`'s name, or "the village".
    fn village_name(&self, ctx: &Ctx, settlement: Option<PermanentId>) -> String {
        settlement
            .and_then(|s| ctx.land.settlements.iter().find(|t| t.id == s))
            .map_or_else(|| "the village".to_owned(), |t| t.name.clone())
    }

    /// The spring or edge of the water the people of the home field `key` go to, and the level its
    /// water stands at, metres.
    fn spring_or_bank_level(&self, ctx: &Ctx, key: PermanentId) -> Option<(u32, f64)> {
        let field = self.homes.get(&key)?;
        let o = self.water_source(ctx, Some(field), None, None)?;
        let spring = field.springs.iter().any(|&(_, _, c)| c == o.cell);
        let z = if spring {
            f64::from(ctx.map.elevation[o.cell as usize])
        } else {
            water_beside(ctx.map, o.cell)
        };
        Some((o.cell, z))
    }

    /// The level household `x` expects water at under `site`, from what its people know: the
    /// water standing in the nearest well of its settlement they could have seen dug, or, where
    /// that well was given up dry, deeper than its floor; with none, the water they draw from now.
    fn expected_water(&self, ctx: &Ctx, x: &Household, site: (f32, f32), source_z: f64) -> f64 {
        let nearest = ctx
            .land
            .wells
            .list
            .iter()
            .filter(|w| matches!(w.state, WellState::Open { .. } | WellState::GivenUp { .. }))
            .filter(|w| {
                w.household == x.id
                    || self.of_polity(w, x.settlement)
                    || self
                        .household(w.household)
                        .is_some_and(|y| y.settlement.is_some() && y.settlement == x.settlement)
            })
            .min_by(|a, b| {
                let d = |w: &&Well| {
                    let c = w.rect.centre_m();
                    (c.0 - site.0).hypot(c.1 - site.1)
                };
                d(a).total_cmp(&d(b)).then(a.id.cmp(&b.id))
            });
        let Some(w) = nearest else {
            return source_z;
        };
        let Some(def) = ctx.catalog.wells.get(w.system) else {
            return source_z;
        };
        match w.state {
            WellState::GivenUp { .. } => source_z.min(w.floor_m() - def.water_m),
            _ => {
                let (head, rate) = ground_water(ctx, def, w.cell);
                w.level_now(ctx.now, head, rate)
            }
        }
    }

    /// Household `household` weighs a well (M6a slice AY, step three; ADR-0021 §3), as it weighs
    /// a crossing: what one would save is the walk from home to the water it draws from now and
    /// back, and the lift there, less the walk to the well and its lift, for every load its people
    /// use, over the years the lining would last at the quality its people would lay it; what it would cost is the hours to dig and line it a metre below
    /// where it expects the water. Of the systems its people know the technique of, it begins
    /// the well whose saving most exceeds its work, if any does, beside its home. A household
    /// with a well whose lining rot has taken half of what its quality left weighs relining it
    /// the same way. One well's work at a time. Each session of the work then weighs the walking
    /// a year of the well would save for each hour of it (`Well::worth`): the near return, set
    /// against what else the day asks of them, as a household weighs a store over a few years
    /// (a design prior; counting the lining's whole life into every session made digging
    /// outweigh hunger).
    fn review_well(&mut self, ctx: &mut Ctx, household: PermanentId) -> WellWeighed {
        let Some(x) = self.household(household) else {
            return WellWeighed::Nothing;
        };
        if x.members.is_empty() || Self::well_kit(ctx).is_none() {
            return WellWeighed::Nothing;
        }
        let mine: Vec<&Well> = ctx
            .land
            .wells
            .list
            .iter()
            .filter(|w| w.household == household)
            .collect();
        if mine.iter().any(|w| w.is_worked()) {
            return WellWeighed::Busy;
        }
        let params = ctx.params;
        let carry = params.household.carry_water_l;
        let members = x.members.len() as f64;
        let loads_a_year =
            members * x.water_use(&params.household) / carry.max(1e-6) * DAYS_PER_YEAR as f64;
        let spread = params.build.quality_spread;
        // Relining the household's open well, once rot has taken half of what its lining had.
        if let Some(w) = mine.iter().find(|w| w.is_open()) {
            if w.loss < RELINE_AT * w.quality {
                return WellWeighed::Sound;
            }
            let Some(def) = ctx.catalog.wells.get(w.system) else {
                return WellWeighed::Nothing;
            };
            let Some(source) = self.other_source(ctx, x, Some(w.id)) else {
                return WellWeighed::NoSource;
            };
            let Some(walk_s) = Self::well_walk_s(ctx, x.home, w.rect.centre_m()) else {
                return WellWeighed::NoSite;
            };
            let (head, rate) = ground_water(ctx, def, w.cell);
            let lift = lift_min(
                def,
                f64::from(w.ground_m) - w.level_now(ctx.now, head, rate),
            );
            let trip = 2.0 * walk_s / 60.0 + lift;
            let quality = condition::expected_quality(self.well_skill(ctx, x, def), spread);
            let rot = def.loss_per_year * wetness_at(ctx, w.cell);
            let life = if rot > 0.0 {
                (quality / rot).min(LONGEST_LIFE_YEARS)
            } else {
                LONGEST_LIFE_YEARS
            };
            let year_h = loads_a_year * (source.trip_min() - trip).max(0.0) / 60.0;
            let saved_h = year_h * life;
            let labour_h = f64::from(w.depth_m) * def.lining_h_per_m;
            if labour_h <= 0.0 || saved_h <= labour_h {
                return WellWeighed::NotWorth { saved_h, labour_h };
            }
            let id = w.id;
            if let Some(w) = ctx.land.wells.get_mut(id) {
                w.mend_h = labour_h as f32;
                w.skill_h = 0.0;
                w.worth = (year_h / labour_h) as f32;
            }
            return WellWeighed::Reline { saved_h, labour_h };
        }
        // A new well beside its home.
        let Some(plot) = ctx
            .land
            .plots
            .iter()
            .filter(|p| p.household == household && p.use_ == PlotUse::Dwelling)
            .min_by(|a, b| {
                let d = |p: &&civ_land::buildings::Plot| {
                    let c = p.rect.centre_m();
                    (c.0 - x.home.0).hypot(c.1 - x.home.1)
                };
                d(a).total_cmp(&d(b)).then(a.id.cmp(&b.id))
            })
            .map(|p| p.rect)
        else {
            return WellWeighed::NoHome;
        };
        let toward = x
            .settlement
            .and_then(|s| ctx.land.settlements.iter().find(|t| t.id == s))
            .map_or(x.home, |t| t.hearth_m);
        let toward = (f64::from(toward.0), f64::from(toward.1));
        let Some(source) = self.other_source(ctx, x, None) else {
            return WellWeighed::NoSource;
        };
        let knows = |t: Option<usize>| {
            t.is_none_or(|t| {
                x.members
                    .iter()
                    .filter_map(|&m| self.person(m))
                    .any(|p| p.knows(t))
            })
        };
        let dig_h_per_m3 = params.digging.h_per_m3;
        // (system, site, target depth, labour, saving over its life, saving a year)
        let mut best: Option<(usize, RectCm, f64, f64, f64, f64)> = None;
        let mut why = WellWeighed::Unknown;
        for (system, def) in ctx.catalog.wells.iter().enumerate() {
            if !knows(def.technique) {
                continue;
            }
            let Some(rect) = well_site(ctx, &plot, toward, 2.0 * def.dig_radius_m) else {
                if why == WellWeighed::Unknown {
                    why = WellWeighed::NoSite;
                }
                continue;
            };
            let at = rect.centre_m();
            let cell = cell_of(ctx.map, at) as u32;
            let ground = f64::from(ctx.map.elevation[cell as usize]);
            let expect = self.expected_water(ctx, x, at, source.water_z);
            let target = (ground - expect).max(0.0) + def.water_m;
            if target > def.max_depth_m {
                if matches!(why, WellWeighed::Unknown | WellWeighed::NoSite) {
                    why = WellWeighed::TooDeep { target_m: target };
                }
                continue;
            }
            let Some(walk_s) = Self::well_walk_s(ctx, x.home, at) else {
                continue;
            };
            let trip = 2.0 * walk_s / 60.0 + lift_min(def, ground - expect);
            let quality = condition::expected_quality(self.well_skill(ctx, x, def), spread);
            let rot = def.loss_per_year * wetness_at(ctx, cell);
            let life = if rot > 0.0 {
                (quality / rot).min(LONGEST_LIFE_YEARS)
            } else {
                LONGEST_LIFE_YEARS
            };
            let year_h = loads_a_year * (source.trip_min() - trip).max(0.0) / 60.0;
            let saved_h = year_h * life;
            let labour = target * def.h_per_m(dig_h_per_m3);
            if saved_h > labour {
                if best.is_none_or(|b| saved_h - labour > b.4 - b.3) {
                    best = Some((system, rect, target, labour, saved_h, year_h));
                }
            } else if !matches!(why, WellWeighed::NotWorth { saved_h: s, labour_h: l } if s - l >= saved_h - labour)
            {
                why = WellWeighed::NotWorth {
                    saved_h,
                    labour_h: labour,
                };
            }
        }
        let Some((system, rect, target, labour, saved_h, year_h)) = best else {
            return why;
        };
        let cell = cell_of(ctx.map, rect.centre_m()) as u32;
        let ground = ctx.map.elevation[cell as usize];
        let id = ctx.ids.allocate();
        ctx.land.wells.list.push(Well {
            id,
            system,
            cell,
            rect,
            household,
            polity: None,
            ground_m: ground,
            depth_m: 0.0,
            target_m: target as f32,
            work_h: 0.0,
            skill_h: 0.0,
            quality: 0.0,
            loss: 0.0,
            mend_h: 0.0,
            cover_h: 0.0,
            covered: false,
            worth: (year_h / labour.max(1e-6)) as f32,
            level_m: f64::from(ground),
            level_at: ctx.now,
            drawn_l: 0.0,
            begun: ctx.now,
            state: WellState::Digging,
            crew: (0, 0),
        });
        WellWeighed::Begun {
            saved_h,
            labour_h: labour,
        }
    }

    /// Where settlement `settlement`'s people could dig a well together (M6a slice BA, step four;
    /// ADR-0021 §7), once one of its households walks far for water (so far, as it last walked
    /// it, that its people use less water for the walk: research 12-01 §2.1), weighed as a
    /// household weighs its own beside its home: beside the hearth,
    /// a metre below where they expect water (from the settlement's wells, else the water they
    /// draw from now), of the system its people know whose walking saved over its lining's life
    /// most exceeds its work. Each of households `households` would save, every load its people
    /// use a year, the walk to the water it last fetched there and back, as they walked it, less
    /// the walk to the well and back and its lift. None when no such well would repay its work,
    /// a polity's well is being dug already, or content has no policy to dig one.
    pub(crate) fn public_well(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        households: &[PermanentId],
    ) -> Option<PublicWell> {
        if !ctx
            .catalog
            .policies
            .iter()
            .any(|d| d.kind == crate::polity::PolicyKind::DigWell)
        {
            return None;
        }
        Self::well_kit(ctx)?;
        let params = ctx.params;
        let carry = params.household.carry_water_l.max(1e-6);
        let homes: Vec<&Household> = households
            .iter()
            .filter_map(|&h| self.household(h))
            .filter(|x| !x.members.is_empty())
            .collect();
        // Some household walks far: so far that its people use less water for it.
        let far = params.household.far_walk_min()?;
        if !homes.iter().any(|x| f64::from(x.water_walk_min) >= far) {
            return None;
        }
        let loads = |x: &Household| {
            x.members.len() as f64 * x.water_use(&params.household) / carry * DAYS_PER_YEAR as f64
        };
        // Nothing to save anyone: no household walks far enough to repay the least of any well.
        let most_h: f64 = homes
            .iter()
            .map(|x| loads(x) * 2.0 * f64::from(x.water_walk_min) / 60.0)
            .sum::<f64>()
            * LONGEST_LIFE_YEARS;
        let dig_h_per_m3 = params.digging.h_per_m3;
        let least_h = ctx
            .catalog
            .wells
            .iter()
            .map(|d| d.water_m * d.h_per_m(dig_h_per_m3))
            .fold(f64::INFINITY, f64::min);
        if most_h <= least_h {
            return None;
        }
        let polity = self.polity_id_of(Some(settlement))?;
        if ctx
            .land
            .wells
            .list
            .iter()
            .any(|w| w.polity == Some(polity) && w.state == WellState::Digging)
        {
            return None;
        }
        let hearth = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)?
            .hearth_m;
        let hearth = (f64::from(hearth.0), f64::from(hearth.1));
        let spot = square(hearth, HEARTH_SPOT_M);
        let source_z = self.spring_or_bank_level(ctx, settlement).map(|l| l.1)?;
        let first = homes.first()?;
        let knows = |t: Option<usize>| {
            t.is_none_or(|t| {
                homes
                    .iter()
                    .flat_map(|x| x.members.iter())
                    .filter_map(|&m| self.person(m))
                    .any(|p| p.knows(t))
            })
        };
        let spread = params.build.quality_spread;
        let mut best: Option<PublicWell> = None;
        for (system, def) in ctx.catalog.wells.iter().enumerate() {
            if !knows(def.technique) {
                continue;
            }
            let Some(rect) = well_site(ctx, &spot, hearth, 2.0 * def.dig_radius_m) else {
                continue;
            };
            let at = rect.centre_m();
            let cell = cell_of(ctx.map, at) as u32;
            let ground = f64::from(ctx.map.elevation[cell as usize]);
            let expect = self.expected_water(ctx, first, at, source_z);
            let target = (ground - expect).max(0.0) + def.water_m;
            if target > def.max_depth_m {
                continue;
            }
            let lift = lift_min(def, ground - expect);
            let saves: Vec<(PermanentId, f64)> = homes
                .iter()
                .map(|x| {
                    let trip = Self::well_walk_s(ctx, x.home, at)
                        .map_or(f64::INFINITY, |s| 2.0 * s / 60.0 + lift);
                    let now = 2.0 * f64::from(x.water_walk_min);
                    (x.id, loads(x) * (now - trip).max(0.0) / 60.0)
                })
                .collect();
            let skill = homes
                .iter()
                .map(|x| self.well_skill(ctx, x, def))
                .sum::<f64>()
                / homes.len().max(1) as f64;
            let quality = condition::expected_quality(skill, spread);
            let rot = def.loss_per_year * wetness_at(ctx, cell);
            let life = if rot > 0.0 {
                (quality / rot).min(LONGEST_LIFE_YEARS)
            } else {
                LONGEST_LIFE_YEARS
            };
            let mut saves = saves;
            saves.sort_unstable_by_key(|s| s.0);
            let p = PublicWell {
                site: crate::polity::WellSite {
                    cell,
                    rect,
                    system: system as u16,
                    target_m: target as f32,
                    labour_h: (target * def.h_per_m(dig_h_per_m3)) as f32,
                },
                life,
                saves,
                households: homes.len(),
            };
            if best
                .as_ref()
                .is_none_or(|b| p.pooled_net_h() > b.pooled_net_h())
            {
                best = Some(p);
            }
        }
        best.filter(|p| p.pooled_net_h() > 0.0)
    }

    /// Polity `polity` begins the well its law names at `site` (M6a slice BA, step four), unless
    /// the ground there is no longer free.
    pub(super) fn begin_public_well(
        &mut self,
        ctx: &mut Ctx,
        polity: PermanentId,
        site: crate::polity::WellSite,
    ) {
        if ctx.catalog.wells.get(usize::from(site.system)).is_none()
            || !crate::build::plot_clear(ctx.land, ctx.map, ctx.nav, &site.rect)
        {
            return;
        }
        let ground = ctx.map.elevation[site.cell as usize];
        let id = ctx.ids.allocate();
        ctx.land.wells.list.push(Well {
            id,
            system: usize::from(site.system),
            cell: site.cell,
            rect: site.rect,
            household: polity,
            polity: Some(polity),
            ground_m: ground,
            depth_m: 0.0,
            target_m: site.target_m,
            work_h: 0.0,
            skill_h: 0.0,
            quality: 0.0,
            loss: 0.0,
            mend_h: 0.0,
            cover_h: 0.0,
            covered: false,
            worth: 0.0,
            level_m: f64::from(ground),
            level_at: ctx.now,
            drawn_l: 0.0,
            begun: ctx.now,
            state: WellState::Digging,
            crew: (0, 0),
        });
    }

    /// Polity `polity` begins a well at `site`, as its law passed would: for tests.
    #[doc(hidden)]
    pub fn begin_public_well_for_tests(
        &mut self,
        ctx: &mut Ctx,
        polity: PermanentId,
        site: crate::polity::WellSite,
    ) {
        self.begin_public_well(ctx, polity, site);
    }

    /// Where settlement `settlement`'s people could dig a well together now, the walking a year
    /// it would save each household and its lining's life: for tests.
    #[doc(hidden)]
    #[allow(clippy::type_complexity)]
    pub fn public_well_for_tests(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
    ) -> Option<(crate::polity::WellSite, Vec<(PermanentId, f64)>, f64)> {
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        self.public_well(ctx, settlement, &households)
            .map(|p| (p.site, p.saves, p.life))
    }

    /// The well household `x`'s people may work on, if any: their household's own, else one
    /// their polity is digging under a law in force they know of (M6a slice BA, step four);
    /// where its work is done, the walk there, the work left, what each hour of it is worth,
    /// and, while the household has not given the share of a polity's well asked of it, what
    /// keeping to the gathering's word is worth to `person`, as for a crossing built together (M6a
    /// slice AY, step three).
    pub(super) fn well_facts(
        &self,
        ctx: &Ctx,
        x: &Household,
        person: PermanentId,
        dark: bool,
    ) -> Option<WellFacts> {
        if dark || ctx.land.wells.list.is_empty() {
            return None;
        }
        let mut duty = 0.0;
        let mut worth = None;
        let w = match ctx
            .land
            .wells
            .list
            .iter()
            .find(|w| w.household == x.id && w.is_worked())
        {
            Some(w) => w,
            None => {
                let pi = self.polity_of(x.settlement?)?;
                let polity = &self.polities[pi];
                let (w, law) = ctx
                    .land
                    .wells
                    .list
                    .iter()
                    .filter(|w| w.polity == Some(polity.id) && w.state == WellState::Digging)
                    .find_map(|w| {
                        let law = polity.laws.iter().find(|l| {
                            l.status == crate::polity::LawStatus::InForce
                                && l.kind == crate::polity::PolicyKind::DigWell
                                && l.knows(person)
                                && polity.well_site_of(l.id).is_some_and(|s| s.cell == w.cell)
                        })?;
                        Some((w, law))
                    })?;
                let households = self
                    .households
                    .iter()
                    .filter(|(_, y)| y.settlement == x.settlement && !y.members.is_empty())
                    .count()
                    .max(1);
                let labour_h = polity
                    .well_site_of(law.id)
                    .map_or(1.0, |s| f64::from(s.labour_h).max(1.0));
                let share = labour_h / households as f64;
                if f64::from(ctx.land.wells.given(w.id, x.id)) + 1e-6 < share {
                    let stance = law
                        .stances
                        .iter()
                        .find(|r| r.person == person)
                        .map_or(0.0, |r| r.stance.sign());
                    let regard = self.ties.regard(
                        person,
                        law.sponsor,
                        ctx.now.day_index(),
                        &ctx.params.ties,
                    );
                    let norm = self.norm_points(ctx, person);
                    duty =
                        crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity)
                            .max(0.0);
                }
                // Each hour of it is worth the walking a year it would save this household.
                let def = ctx.catalog.wells.get(w.system)?;
                let lift = lift_min(def, f64::from(w.target_m) - def.water_m);
                let trip = Self::well_walk_s(ctx, x.home, w.rect.centre_m())
                    .map_or(f64::INFINITY, |s| 2.0 * s / 60.0 + lift);
                let carry = ctx.params.household.carry_water_l.max(1e-6);
                let loads = x.members.len() as f64 * x.water_use(&ctx.params.household) / carry
                    * DAYS_PER_YEAR as f64;
                let year_h = loads * (2.0 * f64::from(x.water_walk_min) - trip).max(0.0) / 60.0;
                worth = Some(year_h / labour_h);
                w
            }
        };
        let def = ctx.catalog.wells.get(w.system)?;
        let left_h = if w.state == WellState::Digging {
            f64::from(w.target_m - w.depth_m).max(0.0) * def.h_per_m(ctx.params.digging.h_per_m3)
        } else if w.mend_h > 0.0 {
            f64::from(w.mend_h)
        } else {
            f64::from(w.cover_h)
        };
        let at = w.rect.centre_m();
        let walk_s = Self::well_walk_s(ctx, x.home, at)?;
        Some(WellFacts {
            well: w.id,
            at,
            walk_min: walk_s / 60.0,
            left_h,
            saves_per_hour: worth.unwrap_or(f64::from(w.worth)),
            duty,
            full: w.crew.0 == ctx.now.day_index() && u32::from(w.crew.1) >= def.crew,
        })
    }

    /// `hours` of a capable adult's work on well `well` by the first of its household's people:
    /// for tests.
    #[doc(hidden)]
    pub fn well_work_for_tests(&mut self, ctx: &mut Ctx, well: PermanentId, hours: f64) {
        let Some(household) = ctx.land.wells.get(well).map(|w| w.household) else {
            return;
        };
        let Some(who) = self
            .household(household)
            .and_then(|x| x.members.first().copied())
        else {
            return;
        };
        self.well_work_by_for_tests(ctx, who, well, hours);
    }

    /// `hours` of a capable adult's work on well `well` by `who`, for their household: for tests.
    #[doc(hidden)]
    pub fn well_work_by_for_tests(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        well: PermanentId,
        hours: f64,
    ) {
        let (Some(&h), Some(household)) =
            (self.index.get(&who), self.person(who).map(|p| p.household))
        else {
            return;
        };
        self.well_work(ctx, (h, who), household, well, hours);
    }

    /// Where household `household`'s people would fetch water now, as short of it or not: for
    /// tests.
    #[doc(hidden)]
    pub fn water_source_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        short: bool,
    ) -> Option<WaterOption> {
        let x = self.household(household)?;
        let key = x
            .settlement
            .filter(|s| self.homes.contains_key(s))
            .unwrap_or(x.id);
        self.water_source(ctx, self.homes.get(&key), Some((x, short)), None)
    }

    /// Where `person` would fetch water now for their household, as short of it or not, with
    /// the closings they keep (M6a slice BA, step four): for tests.
    #[doc(hidden)]
    pub fn water_source_by_for_tests(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        short: bool,
    ) -> Option<WaterOption> {
        let x = self.household(self.person(person)?.household)?;
        let key = x
            .settlement
            .filter(|s| self.homes.contains_key(s))
            .unwrap_or(x.id);
        self.water_source(ctx, self.homes.get(&key), Some((x, short)), Some(person))
    }

    /// Someone sets off to work on well `well`: one of its crew today.
    pub(super) fn joined_well_crew(ctx: &mut Ctx, well: PermanentId) {
        let day = ctx.now.day_index();
        if let Some(w) = ctx.land.wells.get_mut(well) {
            if w.crew.0 != day {
                w.crew = (day, 0);
            }
            w.crew.1 = w.crew.1.saturating_add(1);
        }
    }

    /// `hours` of a capable adult's work by `who` (handle and id) of `household` on its well
    /// `well`: being dug, it goes down as far as those hours dig and line, weighted by their
    /// building skill toward its lining's quality, and they practise the skill. At the depth its
    /// diggers meant, it meets water if the water table stands above its floor and opens, its
    /// lining's quality drawn from how skilled its diggers were on average (ADR-0009 §6); else it
    /// is dug a metre deeper, or, at the deepest its system goes, given up. Being relined, the
    /// hours go to the relining, and once it is done the lining is new.
    pub(super) fn well_work(
        &mut self,
        ctx: &mut Ctx,
        (h, who): (Handle<Person>, PermanentId),
        household: PermanentId,
        well: PermanentId,
        hours: f64,
    ) {
        // Their household's own, or their polity's (M6a slice BA, step four).
        let polity = self.polity_id_of(self.household(household).and_then(|x| x.settlement));
        let Some(i) = ctx.land.wells.list.iter().position(|w| {
            w.id == well
                && (w.household == household || (w.polity.is_some() && w.polity == polity))
                && w.is_worked()
        }) else {
            return;
        };
        let Some(def) = ctx.catalog.wells.get(ctx.land.wells.list[i].system) else {
            return;
        };
        let skill = def
            .skill
            .and_then(|k| ctx.catalog.skills.get(k).map(|s| (k, s)));
        let level = match skill {
            Some((k, _)) => self.people.get(h).map_or(0.0, |p| p.skill(k)),
            None => condition::MIDDLING_SKILL,
        };
        let (now, seed, spread) = (ctx.now, ctx.seed, ctx.params.build.quality_spread);
        let per_m = def.h_per_m(ctx.params.digging.h_per_m3);
        let w = ctx.land.wells.list[i];
        // Owed a lid and curb with no relining owed (M6a slice BA).
        let covering = w.state != WellState::Digging && w.mend_h <= 0.0;
        let done = if w.state == WellState::Digging {
            let left_m = f64::from(w.target_m - w.depth_m).max(0.0);
            let m = (hours / per_m.max(1e-9)).clamp(0.0, left_m);
            m * per_m
        } else if covering {
            hours.clamp(0.0, f64::from(w.cover_h))
        } else {
            hours.clamp(0.0, f64::from(w.mend_h))
        };
        if let (Some((k, s)), Some(p)) = (skill, self.people.get_mut(h)) {
            p.set_skill(k, s.practised(p.skill(k), done));
        }
        // What each household gave of a polity's well is kept (research 09-06: labour asked and
        // labour given apart).
        if ctx.land.wells.list[i].polity.is_some() {
            ctx.land.wells.add_given(well, household, done as f32);
        }
        let w = &mut ctx.land.wells.list[i];
        w.work_h += done as f32;
        if covering {
            w.cover_h = (f64::from(w.cover_h) - done).max(0.0) as f32;
            if w.cover_h > 1e-4 {
                return;
            }
            w.cover_h = 0.0;
            w.covered = true;
            let place = Some(w.rect.centre_m());
            let sentence = format!(
                "{}'s household made their {} a lid and curb.",
                self.name_of(who),
                def.name.to_lowercase()
            );
            let settlement = self.household(household).and_then(|x| x.settlement);
            self.chronicle_push(
                now,
                ChronicleKind::Well,
                vec![who],
                settlement,
                place,
                f64::from(WellStep::Covered as u8),
                sentence,
            );
            return;
        }
        w.skill_h += (done * level) as f32;
        if w.state != WellState::Digging {
            // Relining.
            w.mend_h = (f64::from(w.mend_h) - done).max(0.0) as f32;
            if w.mend_h > 1e-4 {
                return;
            }
            let lining = f64::from(w.depth_m) * def.lining_h_per_m;
            let avg = if lining > 0.0 {
                (f64::from(w.skill_h) / lining).clamp(0.0, 1.0)
            } else {
                level
            };
            w.mend_h = 0.0;
            w.loss = 0.0;
            w.quality = condition::draw_rebuilt_quality(seed, w.id, 0, now, avg, spread);
            return;
        }
        w.depth_m = (f64::from(w.depth_m) + done / per_m.max(1e-9)) as f32;
        if w.depth_m + 1e-4 < w.target_m {
            return;
        }
        w.depth_m = w.target_m;
        let w = ctx.land.wells.list[i];
        let (head, _) = ground_water(ctx, def, w.cell);
        let name = def.name.to_lowercase();
        let depth = w.depth_m;
        let settlement = self.household(household).and_then(|x| x.settlement);
        let place = Some(w.rect.centre_m());
        if head > w.floor_m() {
            // It met water: lined at the quality its diggers' skill gives, and open.
            let dug = f64::from(w.work_h);
            let avg = if dug > 0.0 {
                (f64::from(w.skill_h) / dug).clamp(0.0, 1.0)
            } else {
                level
            };
            let well = &mut ctx.land.wells.list[i];
            well.quality = condition::draw_quality(seed, well.id, 0, avg, spread);
            well.level_m = head.min(f64::from(well.ground_m));
            well.level_at = now;
            well.state = WellState::Open { since: now };
            well.skill_h = 0.0;
            let sentence = match w.polity {
                Some(_) => format!(
                    "{}'s people finished the {name} their gathering agreed to dig beside the \
                     hearth, {} digging the last of it: {depth:.1} m deep, it met water.",
                    self.village_name(ctx, settlement),
                    self.name_of(who)
                ),
                None => format!(
                    "{}'s household dug a {name} {depth:.1} m deep beside their home, and it met \
                     water.",
                    self.name_of(who)
                ),
            };
            self.chronicle_push(
                now,
                ChronicleKind::Well,
                vec![who],
                settlement,
                place,
                f64::from(WellStep::Opened as u8),
                sentence,
            );
        } else if f64::from(depth) + 1.0 <= def.max_depth_m + 1e-6 {
            // Dry: a metre deeper.
            ctx.land.wells.list[i].target_m = depth + 1.0;
        } else {
            ctx.land.wells.list[i].state = WellState::GivenUp { on: now };
            let sentence = match w.polity {
                Some(_) => format!(
                    "{}'s people gave up the {name} their gathering agreed to dig beside the \
                     hearth: {depth:.1} m down, it had met no water.",
                    self.village_name(ctx, settlement)
                ),
                None => format!(
                    "{}'s household gave up the {name} it was digging beside its home: \
                     {depth:.1} m down, it had met no water.",
                    self.name_of(who)
                ),
            };
            self.chronicle_push(
                now,
                ChronicleKind::Well,
                vec![who],
                settlement,
                place,
                f64::from(WellStep::GivenUp as u8),
                sentence,
            );
        }
    }
}
