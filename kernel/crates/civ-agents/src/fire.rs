//! Fire in a building and between buildings (M6b slice BC; ADR-0022 §4, §5): the arithmetic of a
//! building's fire and of the heat it puts on its neighbours, apart from who lives where or what
//! anyone does about it.
//!
//! - **A building burns by stages.** Its covering catches, the building is fully involved a few
//!   minutes later, and it burns out after a while more. These are authored durations, design
//!   priors: research 12-03 gives no curve for a hut. Its covering is consumed by the time it has
//!   burned `covering_minutes`.
//! - **Heat reaches neighbours by the gap between them.** A fully involved building puts
//!   `flux_kw[0]` on a surface a metre from it, falling with the gap as 12-03 §2.2's
//!   measurements fall: 36 kW/m² at 1 m to 5 kW/m² at 4 m. A covering alight puts a share of it.
//!   The wind stretches the gap upwind and shortens it downwind. Firebrands are folded into that
//!   stretch, as one route.
//! - **Exposure accumulates.** A threatened building keeps a store that moves toward the heat on
//!   it over `exposure_minutes` and falls as it cools. Its covering catches by a hazard on how far
//!   the store stands above the covering's threshold (12-03 §1.4: "accumulated exposure, not
//!   distance alone"). Dryness scales the threshold: wet fuel needs more heat, and fuel at
//!   `extinction_moisture` does not catch at all.
//! - **Timber chars.** While a building is fully involved, its exposed members char from every
//!   face, `char_mm_first_hour` in the first hour on a curve that slows as the char deepens. Char
//!   is loss of section (11-08 §2.5: about 0.6 mm/min over the first hour; a 200 mm member exposed
//!   on four faces keeps a 128 mm core).
//!
//! - **Fires start where fire is used** (slice BD; ADR-0022 §3). A household's hearth, lit while
//!   any of its people is awake at home and banked otherwise, and a fire worked at home (baking,
//!   firing pots) each carry a hazard per hour, scaled by dryness. An ignition with someone awake
//!   in or beside the building is put out at once and only counted; otherwise its covering
//!   catches.
//!
//! Nothing here decides what anyone does.

use civ_core::{PermanentId, SimTime};

/// The heat that boils off a litre of water, kJ.
pub const LATENT_KJ_PER_L: f64 = 2257.0;

/// How fire behaves (the people profile's `[fire]`; from content API 81). Every figure is a design
/// prior unless its comment cites a measurement.
#[derive(Clone, Debug, PartialEq)]
pub struct FireParams {
    /// Buildings within this gap of a burning one are weighed for its heat, metres.
    pub reach_m: f64,
    /// The heat a fully involved building puts on a surface 1 m and 4 m from it, kW/m² (research
    /// 12-03 §2.2: 36 and 5 in the studied configuration).
    pub flux_kw: [f64; 2],
    /// The share of that heat a building whose covering is alight puts out before it is fully
    /// involved.
    pub covering_flux_share: f64,
    /// How the wind stretches a gap: downwind of a burning building a gap counts as
    /// `gap · exp(−this · wind · cos θ)`, upwind as longer, per m/s of wind.
    pub wind_stretch_per_ms: f64,
    /// Minutes over which a building's exposure moves most of the way to the heat on it.
    pub exposure_minutes: f64,
    /// The hazard, per minute, with which a covering catches for each multiple of its threshold
    /// its exposure stands above it.
    pub catch_per_minute: f64,
    /// The water, as a share of fuel's dry mass, at which exposed fuel no longer catches (fire
    /// behaviour's moisture of extinction).
    pub extinction_moisture: f64,
    /// The water, as a share of fuel's dry mass, at which a good's `catch_kw_m2` is its threshold:
    /// drier fuel catches at less heat, wetter at more.
    pub reference_moisture: f64,
    /// Minutes from a building's covering catching to its being fully involved.
    pub involve_minutes: f64,
    /// Minutes a building stays fully involved before it is burnt out.
    pub involved_minutes: f64,
    /// Minutes in which a burning covering is consumed.
    pub covering_minutes: f64,
    /// Char on a member's exposed faces after an hour fully involved, mm (11-08 §2.5).
    pub char_mm_first_hour: f64,
    /// How char deepens with time: `char_mm_first_hour · (t / 60)^this`, slowing as the char
    /// insulates the wood beneath (11-08 §2.5: faster at first, about 0.6 mm/min over the hour).
    pub char_exponent: f64,
    /// The share of a wall's infill a full burn takes: wattle burns and daub stays.
    pub infill_loss: f64,
    /// Ignitions per hour of a household's hearth while any of its people is awake at home, at
    /// `reference_moisture` (slice BD; ADR-0022 §3).
    pub hearth_lit_per_hour: f64,
    /// Ignitions per hour of a banked hearth: embers kept while nobody of the household is
    /// awake at home.
    pub hearth_banked_per_hour: f64,
    /// Ignitions per hour of each fire worked at home: someone baking or firing pots.
    pub worked_per_hour: f64,
    /// How near the building, metres, someone awake must be to put an ignition out at once.
    pub beside_m: f64,
    /// The least and most minutes an escape left alone grows inside before it reaches the
    /// covering (slice BD; 12-03 §1.2: an ignition is locally controlled or growing before the
    /// building is involved). Drawn evenly between them.
    pub grow_minutes: [f64; 2],
    /// The least and most minutes after an escape before someone asleep in the building wakes to
    /// it (12-03 §2.5: 5 to 30 or more for the sleeping). Drawn evenly between them, for each
    /// sleeper and escape.
    pub wake_minutes: [f64; 2],
    /// How far, metres, someone awake sees a building burning (ADR-0022 §6).
    pub sight_m: f64,
    /// How far, metres, a shout of fire is heard (ADR-0022 §6).
    pub earshot_m: f64,
    /// The share of the water thrown on a roof that stays on it, the rest running off or
    /// missing (slice BD, step three; 12-03 §1.7's η).
    pub water_stays: f64,
    /// The share of the water on a roof that runs off or dries each minute.
    pub wet_drains_per_minute: f64,
    /// Points: saving their household's own building from a fire, were it sure to catch
    /// otherwise (slice BD, step three; ADR-0022 §6).
    pub w_save: f64,
    /// What saving a building of their close kin's household is worth, as a share of their own.
    pub kin_share: f64,
    /// What saving a neighbour's building is worth, as a share of their own.
    pub neighbour_share: f64,
    /// Square metres of a roof's covering one person strips a minute (slice BD, step four).
    pub strip_m2_per_minute: f64,
    /// What stripping a roof costs its household, as a share of what losing it to fire would:
    /// the covering is gone either way, the rest of the building is not.
    pub strip_cost: f64,
    /// What the goods a household keeps under a roof are worth to it, as a share of the
    /// building's worth (`w_save`), when a load of them is carried out of a fire's way.
    pub salvage_share: f64,
}

/// How far a building's fire has gone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    /// Its covering is alight.
    Covering,
    /// It is fully involved: covering, timber and what lies under its roof are burning.
    Involved,
    /// It has burnt out.
    Out,
}

impl FireParams {
    /// The core content's values (`content/core/people/early_farmers.toml`), for tests.
    pub fn core() -> FireParams {
        FireParams {
            reach_m: 10.0,
            flux_kw: [36.0, 5.0],
            covering_flux_share: 0.5,
            wind_stretch_per_ms: 0.1,
            exposure_minutes: 1.0,
            catch_per_minute: 1.0,
            extinction_moisture: 0.35,
            reference_moisture: 0.12,
            involve_minutes: 1.0,
            involved_minutes: 30.0,
            covering_minutes: 15.0,
            char_mm_first_hour: 36.0,
            char_exponent: 0.813,
            infill_loss: 0.5,
            hearth_lit_per_hour: 2.0e-6,
            hearth_banked_per_hour: 1.0e-6,
            worked_per_hour: 4.0e-6,
            beside_m: 3.0,
            grow_minutes: [2.0, 15.0],
            wake_minutes: [5.0, 30.0],
            sight_m: 150.0,
            earshot_m: 60.0,
            water_stays: 0.5,
            wet_drains_per_minute: 0.03,
            w_save: 20.0,
            kin_share: 0.5,
            neighbour_share: 0.25,
            strip_m2_per_minute: 1.0,
            strip_cost: 0.5,
            salvage_share: 0.5,
        }
    }

    /// Litres of water the heat `kw_m2` on a roof of `area_m2` boils off in `minutes`: a wet
    /// covering cannot pass the boiling point until it is dry, so the heat goes into its water
    /// first (2,257 kJ a litre).
    pub fn boiled_l(&self, kw_m2: f64, area_m2: f64, minutes: f64) -> f64 {
        (kw_m2.max(0.0) * area_m2.max(0.0) * minutes.max(0.0) * 60.0 / LATENT_KJ_PER_L).max(0.0)
    }

    /// How dryness scales an ignition hazard: 1 at `reference_moisture`, more for drier fuel,
    /// falling to nothing at `extinction_moisture` (12-03 §1.1, §1.5).
    pub fn ignition_scale(&self, moisture: f64) -> f64 {
        let x = self.extinction_moisture;
        if x.is_nan() || x <= 0.0 {
            return 0.0;
        }
        let reference = self.reference_moisture.clamp(0.0, x * 0.99);
        ((1.0 - moisture.max(0.0) / x) / (1.0 - reference / x)).max(0.0)
    }

    /// The stage of a fire `minutes` after its covering caught.
    pub fn stage_at(&self, minutes: f64) -> Stage {
        if minutes < self.involve_minutes {
            Stage::Covering
        } else if minutes < self.involve_minutes + self.involved_minutes {
            Stage::Involved
        } else {
            Stage::Out
        }
    }

    /// The share of its full heat a fire puts out at stage `stage`.
    pub fn heat_share(&self, stage: Stage) -> f64 {
        match stage {
            Stage::Covering => self.covering_flux_share.clamp(0.0, 1.0),
            Stage::Involved => 1.0,
            Stage::Out => 0.0,
        }
    }

    /// The heat a fully involved building puts on a surface `gap_m` from it across still air,
    /// kW/m²: falling exponentially through the two measured points, and never more than at a
    /// gap of nothing.
    pub fn flux_at(&self, gap_m: f64) -> f64 {
        let [near, far] = self.flux_kw;
        if !(near > 0.0 && far > 0.0) {
            return 0.0;
        }
        let k = (near / far).ln() / 3.0;
        near * (-k * (gap_m.max(0.0) - 1.0)).exp()
    }

    /// The gap a neighbour `gap_m` away counts as in the wind: `toward` is the cosine between the
    /// wind's direction and the direction from the burning building to the neighbour (1
    /// downwind, −1 upwind).
    pub fn wind_gap(&self, gap_m: f64, wind_ms: f64, toward: f64) -> f64 {
        gap_m.max(0.0)
            * (-self.wind_stretch_per_ms * wind_ms.max(0.0) * toward.clamp(-1.0, 1.0)).exp()
    }

    /// The heat a fire at stage `stage` puts on a neighbour `gap_m` away, kW/m².
    pub fn heat_on(&self, stage: Stage, gap_m: f64, wind_ms: f64, toward: f64) -> f64 {
        if gap_m > self.reach_m {
            return 0.0;
        }
        self.heat_share(stage) * self.flux_at(self.wind_gap(gap_m, wind_ms, toward))
    }

    /// An exposure store `store` after `minutes` more of `heat` kW/m².
    pub fn expose(&self, store: f64, heat: f64, minutes: f64) -> f64 {
        let keep = (-minutes / self.exposure_minutes.max(1e-6)).exp();
        store * keep + heat * (1.0 - keep)
    }

    /// The exposure at which a covering of threshold `catch_kw` (at `reference_moisture`) catches
    /// when exposed fuel holds `moisture` of its dry mass; `None` when it is too wet to catch.
    pub fn threshold(&self, catch_kw: f64, moisture: f64) -> Option<f64> {
        let x = self.extinction_moisture;
        if !(catch_kw > 0.0 && moisture < x) {
            return None;
        }
        let reference = self.reference_moisture.clamp(0.0, x * 0.99);
        Some(catch_kw * (1.0 - reference / x) / (1.0 - moisture.max(0.0) / x))
    }

    /// The chance a covering whose exposure is `store` against its threshold `threshold` catches
    /// within `minutes`.
    pub fn catch_chance(&self, store: f64, threshold: f64, minutes: f64) -> f64 {
        if !(threshold > 0.0 && store > threshold) {
            return 0.0;
        }
        let hazard = self.catch_per_minute * (store / threshold - 1.0);
        1.0 - (-hazard * minutes.max(0.0)).exp()
    }

    /// The char on an exposed face after `minutes` fully involved, mm.
    pub fn char_mm(&self, minutes: f64) -> f64 {
        if minutes <= 0.0 {
            return 0.0;
        }
        self.char_mm_first_hour * (minutes / 60.0).powf(self.char_exponent)
    }

    /// The share of its covering a fire has consumed `minutes` after it caught.
    pub fn covering_burnt(&self, minutes: f64) -> f64 {
        (minutes / self.covering_minutes.max(1e-6)).clamp(0.0, 1.0)
    }
}

/// The loss of section of a member whose smaller dimension is `size_mm` and which had already
/// lost `loss` of it, after `char_mm` more char on every face: what is left shrinks from both
/// sides (as `GroupCondition::loss` counts it, a share of the dimension).
pub fn charred_loss(loss: f32, size_mm: f64, char_mm: f64) -> f32 {
    if size_mm.is_nan() || size_mm <= 0.0 {
        return loss;
    }
    (f64::from(loss) + 2.0 * char_mm.max(0.0) / size_mm).clamp(0.0, 1.0) as f32
}

/// A building's outline as fire sees it: the edge of its roof, metres from the map's north-west
/// corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outline {
    /// A round roof: its centre and radius.
    Round {
        /// Centre east, metres.
        x: f64,
        /// Centre south, metres.
        y: f64,
        /// Radius, metres.
        r: f64,
    },
    /// A rectangular roof, as its box: west, north, east, south.
    Box {
        /// West edge, metres.
        x0: f64,
        /// North edge, metres.
        y0: f64,
        /// East edge, metres.
        x1: f64,
        /// South edge, metres.
        y1: f64,
    },
}

impl Outline {
    /// The area of roof it covers in plan, m².
    pub fn area_m2(&self) -> f64 {
        match *self {
            Outline::Round { r, .. } => std::f64::consts::PI * r * r,
            Outline::Box { x0, y0, x1, y1 } => ((x1 - x0) * (y1 - y0)).abs(),
        }
    }

    /// Its centre, metres.
    pub fn centre(&self) -> (f64, f64) {
        match *self {
            Outline::Round { x, y, .. } => (x, y),
            Outline::Box { x0, y0, x1, y1 } => ((x0 + x1) / 2.0, (y0 + y1) / 2.0),
        }
    }

    /// The point of the outline nearest `p`, or `p` itself when it lies within.
    fn nearest(&self, p: (f64, f64)) -> (f64, f64) {
        match *self {
            Outline::Round { x, y, r } => {
                let (dx, dy) = (p.0 - x, p.1 - y);
                let d = dx.hypot(dy);
                if d <= r || d == 0.0 {
                    p
                } else {
                    (x + dx * r / d, y + dy * r / d)
                }
            }
            Outline::Box { x0, y0, x1, y1 } => (p.0.clamp(x0, x1), p.1.clamp(y0, y1)),
        }
    }

    /// The gap between this outline and `other`, metres: 0 where they touch or overlap.
    pub fn gap(&self, other: &Outline) -> f64 {
        match (*self, *other) {
            (
                Outline::Round { x, y, r },
                Outline::Round {
                    x: x2,
                    y: y2,
                    r: r2,
                },
            ) => ((x2 - x).hypot(y2 - y) - r - r2).max(0.0),
            (
                Outline::Box { x0, y0, x1, y1 },
                Outline::Box {
                    x0: a0,
                    y0: b0,
                    x1: a1,
                    y1: b1,
                },
            ) => {
                let dx = (a0 - x1).max(x0 - a1).max(0.0);
                let dy = (b0 - y1).max(y0 - b1).max(0.0);
                dx.hypot(dy)
            }
            (Outline::Round { x, y, r }, b @ Outline::Box { .. })
            | (b @ Outline::Box { .. }, Outline::Round { x, y, r }) => {
                let q = b.nearest((x, y));
                ((q.0 - x).hypot(q.1 - y) - r).max(0.0)
            }
        }
    }

    /// The cosine between the direction from this outline's centre to `other`'s and the
    /// direction the wind blows toward, `(east, south)` as a unit vector: 1 when `other` lies
    /// straight downwind.
    pub fn toward(&self, other: &Outline, wind: (f64, f64)) -> f64 {
        let (a, b) = (self.centre(), other.centre());
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let d = dx.hypot(dy);
        if d == 0.0 {
            0.0
        } else {
            (dx * wind.0 + dy * wind.1) / d
        }
    }
}

/// How a building's fire began: the kernel's truth, recorded when it happens (ADR-0015 §8:
/// `caused_by`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kindled {
    /// The observer set its covering alight (an influence record, ADR-0022 §14).
    Observer {
        /// The influence's id.
        influence: u32,
    },
    /// A burning neighbour's heat caught its covering.
    Neighbour {
        /// The fire that exposed it most.
        fire: u32,
        /// The gap between them, metres.
        gap_m: f32,
        /// The day's wind, m/s.
        wind_ms: f32,
        /// The cosine between the wind and the way from the burning building to this one (1
        /// downwind).
        toward: f32,
        /// The water exposed fuel held, as a share of its dry mass.
        moisture: f32,
    },
    /// A fire in use escaped its keeping with nobody awake there to put it out (slice BD).
    Ignition {
        /// The ignition's id.
        ignition: u32,
    },
}

/// The fire in use an ignition came from (ADR-0022 §3). Numeric in saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hearth {
    /// The household's hearth, lit while someone was awake at home.
    Lit = 0,
    /// The household's hearth, banked: embers kept while nobody of it was awake at home.
    Banked = 1,
    /// A fire worked at home: baking or firing pots.
    Worked = 2,
}

impl Hearth {
    /// The kind of fire for its code in a save.
    pub fn from_code(code: u8) -> Option<Hearth> {
        match code {
            0 => Some(Hearth::Lit),
            1 => Some(Hearth::Banked),
            2 => Some(Hearth::Worked),
            _ => None,
        }
    }
}

/// An unintended ignition of a fire in use (12-03 §1.2's first counter): put out at once by
/// someone awake there, or the start of a building's fire. The kernel's truth. Saved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ignition {
    /// Its id, from 1, in the order they happened.
    pub id: u32,
    /// When.
    pub at: SimTime,
    /// The building whose fire in use escaped.
    pub building: PermanentId,
    /// Its household.
    pub household: PermanentId,
    /// The settlement it stood in, if any.
    pub settlement: Option<PermanentId>,
    /// The fire in use it came from.
    pub hearth: Hearth,
    /// The water exposed fuel held that day, as a share of its dry mass.
    pub moisture: f32,
    /// Minutes it grows inside, left alone, before it reaches the covering (0 for one put out
    /// at once).
    pub grows_min: f32,
    /// When it was put out, reached the covering, or went out; none while it grows.
    pub ended: Option<SimTime>,
    /// Who put it out, at once or while it grew, if anyone did.
    pub put_out_by: Option<PermanentId>,
    /// The building's fire it became, if nobody did and its covering could catch (neither: it
    /// went out with nothing to catch).
    pub fire: Option<u32>,
}

impl Ignition {
    /// Whether it still grows inside, neither put out nor yet at the covering.
    pub fn growing(&self) -> bool {
        self.ended.is_none()
    }
}

/// How someone came to know of a building's fire (ADR-0022 §6). Numeric in saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Noticed {
    /// They were awake inside when its covering caught.
    Inside = 0,
    /// They were asleep inside and it woke them when it was fully involved around them.
    Woke = 1,
    /// They saw it burning.
    Saw = 2,
    /// They heard the shout of fire.
    Heard = 3,
}

impl Noticed {
    /// How someone came to know, for its code in a save.
    pub fn from_code(code: u8) -> Option<Noticed> {
        match code {
            0 => Some(Noticed::Inside),
            1 => Some(Noticed::Woke),
            2 => Some(Noticed::Saw),
            3 => Some(Noticed::Heard),
            _ => None,
        }
    }
}

/// Someone came to know of a fire (ADR-0022 §7): a record of a person, not of the fire. Each
/// fire they are inside or see is one; hearing of a fire is one for each conflagration (or lone
/// fire) they had not known of. Saved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Knew {
    /// The fire: the one they saw or were in, or the one whose shout they heard.
    pub fire: u32,
    /// Who.
    pub person: PermanentId,
    /// When.
    pub at: SimTime,
    /// How.
    pub how: Noticed,
    /// Whether they have shouted of it: each shouts once for each fire they come to know of
    /// first, the minute after.
    pub shouted: bool,
}

/// A load of water someone threw on a roof a fire threatened (ADR-0022 §6): the kernel's truth.
/// Saved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thrown {
    /// The fire it was thrown against: the first of the conflagration burning then, or the lone
    /// fire.
    pub fire: u32,
    /// The building it was thrown on.
    pub building: PermanentId,
    /// Who threw it.
    pub person: PermanentId,
    /// When.
    pub at: SimTime,
    /// Litres thrown.
    pub litres: f32,
    /// Where it was drawn: a source's cell, or [`crate::person::FROM_STORE`].
    pub from: u32,
}

/// A building's fire: the kernel's record of it, from its covering catching to its burning out
/// (ADR-0022 §13: an incident). Saved.
#[derive(Clone, Debug, PartialEq)]
pub struct Fire {
    /// Its id, from 1, in the order fires began.
    pub id: u32,
    /// The building that burned.
    pub building: PermanentId,
    /// The household it belonged to when the fire began.
    pub household: PermanentId,
    /// The settlement it stood in, if any.
    pub settlement: Option<PermanentId>,
    /// Where it stood: its centre, metres.
    pub at: (f32, f32),
    /// When its covering caught.
    pub began: SimTime,
    /// How it began.
    pub kindled: Kindled,
    /// The conflagration it is part of, if it reached another building or was reached from one.
    pub episode: Option<u32>,
    /// When it burnt out.
    pub ended: Option<SimTime>,
    /// Kilograms of goods it burned.
    pub goods_kg: f32,
    /// Who died in it.
    pub deaths: Vec<PermanentId>,
}

impl Fire {
    /// Minutes since its covering caught, at `now`.
    pub fn minutes_at(&self, now: SimTime) -> f64 {
        (now.minutes() - self.began.minutes()) as f64
    }

    /// Whether it still burns.
    pub fn burning(&self) -> bool {
        self.ended.is_none()
    }
}

/// A conflagration: fires that spread from one building to others (ADR-0022 §13: an episode). It
/// opens when a fire reaches a second building, and closes when the last of its fires is out.
#[derive(Clone, Debug, PartialEq)]
pub struct Conflagration {
    /// Its id, from 1.
    pub id: u32,
    /// The settlement it began in, if any.
    pub settlement: Option<PermanentId>,
    /// The fire it began with.
    pub first: u32,
    /// When that fire began.
    pub began: SimTime,
    /// When its last fire burnt out.
    pub ended: Option<SimTime>,
}

/// A building the spread weighed (ADR-0022 §13: "why not"): the most heat its exposure reached and
/// what its covering needed then. Kept per conflagration or lone fire, by its first fire's id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weighed {
    /// The first fire of the conflagration, or the lone fire, that weighed it.
    pub fire: u32,
    /// The building.
    pub building: PermanentId,
    /// The highest its exposure stood, kW/m².
    pub peak_kw: f32,
    /// Its covering's threshold when it stood highest, kW/m² (infinite when too wet, or for a
    /// covering that does not catch).
    pub threshold_kw: f32,
    /// Whether it caught.
    pub caught: bool,
}

/// Every fire of a world, the conflagrations they made, and the exposure of buildings near those
/// burning now. Saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fires {
    /// Every fire, oldest first.
    pub list: Vec<Fire>,
    /// Every conflagration, oldest first.
    pub episodes: Vec<Conflagration>,
    /// The exposure, kW/m², of each building weighed near a fire burning now, by building id.
    pub exposure: Vec<(PermanentId, f64)>,
    /// What the spread weighed.
    pub weighed: Vec<Weighed>,
    /// Every unintended ignition of a fire in use, oldest first (slice BD).
    pub ignitions: Vec<Ignition>,
    /// Who came to know of each fire, and how, oldest first (slice BD).
    pub knew: Vec<Knew>,
    /// The water standing on roofs near fires burning now, litres, by building id (slice BD,
    /// step three).
    pub wet: Vec<(PermanentId, f64)>,
    /// Every load of water thrown at a fire, oldest first (slice BD, step three).
    pub thrown: Vec<Thrown>,
    /// Every stretch of covering torn from a roof a fire threatened, oldest first (slice BD,
    /// step four).
    pub stripped: Vec<Stripped>,
    /// Every load of goods carried out from under a roof a fire threatened, oldest first.
    pub salvaged: Vec<Salvaged>,
}

/// Covering someone tore from a roof a fire threatened (ADR-0022 §6): the kernel's truth. Saved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stripped {
    /// The fire it was done against: the first of the conflagration burning then.
    pub fire: u32,
    /// The building whose roof it was.
    pub building: PermanentId,
    /// Who.
    pub person: PermanentId,
    /// When.
    pub at: SimTime,
    /// Square metres torn off.
    pub m2: f32,
}

/// Goods someone carried out from under a roof a fire threatened (ADR-0022 §5-§6: goods carried
/// out survive): the kernel's truth. They stay their household's; the building's fire, if it
/// comes, does not reach them. Saved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Salvaged {
    /// The fire it was done against: the first of the conflagration burning then.
    pub fire: u32,
    /// The building they were carried out of.
    pub building: PermanentId,
    /// Who.
    pub person: PermanentId,
    /// When.
    pub at: SimTime,
    /// The good, by index in the catalog's goods.
    pub good: u16,
    /// Kilograms.
    pub kg: f32,
}

impl Fires {
    /// Whether any fire burns.
    pub fn any_burning(&self) -> bool {
        self.list.iter().any(Fire::burning)
    }

    /// Whether anything needs a minute's stepping: a fire burning, or an escape growing.
    pub fn active(&self) -> bool {
        self.any_burning() || self.ignitions.iter().rev().any(Ignition::growing)
    }

    /// The fire burning in `building`, if any.
    pub fn burning_in(&self, building: PermanentId) -> Option<&Fire> {
        self.list
            .iter()
            .rev()
            .find(|f| f.building == building && f.burning())
    }

    /// The fire numbered `id`.
    pub fn get(&self, id: u32) -> Option<&Fire> {
        id.checked_sub(1)
            .and_then(|i| self.list.get(i as usize))
            .filter(|f| f.id == id)
    }

    /// The first fire of the conflagration `fire` is part of, or `fire` itself when it is part
    /// of none.
    pub fn root(&self, fire: u32) -> u32 {
        self.get(fire)
            .and_then(|f| f.episode)
            .and_then(|e| self.episodes.get(e as usize - 1))
            .map_or(fire, |e| e.first)
    }

    /// What is wrong with the record, if anything.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (i, f) in self.list.iter().enumerate() {
            if f.id as usize != i + 1 {
                out.push(format!("fire {} is numbered out of order", f.id));
            }
            if f.ended.is_some_and(|e| e < f.began) {
                out.push(format!("fire {} ended before it began", f.id));
            }
            if f.episode
                .is_some_and(|e| e == 0 || e as usize > self.episodes.len())
            {
                out.push(format!("fire {} is part of an unknown conflagration", f.id));
            }
            if !(f.goods_kg.is_finite() && f.goods_kg >= 0.0) {
                out.push(format!("fire {} burned an invalid amount of goods", f.id));
            }
        }
        for (i, e) in self.episodes.iter().enumerate() {
            if e.id as usize != i + 1 || e.first == 0 || e.first as usize > self.list.len() {
                out.push(format!("conflagration {} is malformed", e.id));
            }
        }
        for (i, g) in self.ignitions.iter().enumerate() {
            if g.id as usize != i + 1 {
                out.push(format!("ignition {} is numbered out of order", g.id));
            }
            if g.put_out_by.is_some() && g.fire.is_some() {
                out.push(format!("ignition {} was both put out and a fire", g.id));
            }
            if g.fire
                .is_some_and(|f| f == 0 || f as usize > self.list.len())
            {
                out.push(format!("ignition {} became an unknown fire", g.id));
            }
            if g.growing() && (g.put_out_by.is_some() || g.fire.is_some()) {
                out.push(format!("ignition {} grows but has ended", g.id));
            }
            if !g.grows_min.is_finite() || g.grows_min < 0.0 {
                out.push(format!("ignition {} grows for an invalid time", g.id));
            }
        }
        if self.wet.iter().any(|&(_, l)| !(l.is_finite() && l >= 0.0)) {
            out.push("a roof holds an invalid amount of water".to_owned());
        }
        for t in &self.thrown {
            if t.fire == 0 || t.fire as usize > self.list.len() {
                out.push(format!(
                    "water was thrown against an unknown fire {}",
                    t.fire
                ));
            }
            if !(t.litres.is_finite() && t.litres > 0.0) {
                out.push(format!(
                    "a load thrown against fire {} held no water",
                    t.fire
                ));
            }
        }
        let known = |f: u32| f != 0 && f as usize <= self.list.len();
        if self
            .stripped
            .iter()
            .any(|s| !known(s.fire) || !(s.m2.is_finite() && s.m2 > 0.0))
        {
            out.push("a stripped roof is malformed".to_owned());
        }
        if self
            .salvaged
            .iter()
            .any(|s| !known(s.fire) || !(s.kg.is_finite() && s.kg > 0.0))
        {
            out.push("goods carried out of a fire's way are malformed".to_owned());
        }
        for (i, k) in self.knew.iter().enumerate() {
            if k.fire == 0 || k.fire as usize > self.list.len() {
                out.push(format!("someone knew of an unknown fire {}", k.fire));
            }
            if i > 0 && self.knew[i - 1].at > k.at {
                out.push("who knew of fires is out of order".to_owned());
            }
        }
        if self
            .exposure
            .iter()
            .any(|(_, s)| !(s.is_finite() && *s >= 0.0))
        {
            out.push("a building's exposure is invalid".to_owned());
        }
        out
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn params() -> FireParams {
        FireParams::core()
    }

    #[test]
    fn heat_falls_with_the_gap_as_measured_and_the_wind_carries_it_downwind() {
        let p = params();
        assert!((p.flux_at(1.0) - 36.0).abs() < 1e-9);
        assert!((p.flux_at(4.0) - 5.0).abs() < 1e-9);
        assert!(p.flux_at(2.0) < 36.0 && p.flux_at(2.0) > 5.0);
        // Nothing beyond the reach, less at a covering alight than fully involved.
        assert_eq!(p.heat_on(Stage::Involved, 11.0, 0.0, 1.0), 0.0);
        assert_eq!(p.heat_on(Stage::Out, 1.0, 0.0, 1.0), 0.0);
        assert!(
            p.heat_on(Stage::Covering, 2.0, 0.0, 0.0) < p.heat_on(Stage::Involved, 2.0, 0.0, 0.0)
        );
        // Downwind more, upwind less, across the wind as still air.
        let still = p.heat_on(Stage::Involved, 2.0, 0.0, 1.0);
        assert!(p.heat_on(Stage::Involved, 2.0, 5.0, 1.0) > still);
        assert!(p.heat_on(Stage::Involved, 2.0, 5.0, -1.0) < still);
        assert!((p.heat_on(Stage::Involved, 2.0, 5.0, 0.0) - still).abs() < 1e-9);
    }

    #[test]
    fn exposure_builds_and_cools_and_wet_thatch_needs_more_heat_or_will_not_catch() {
        let p = params();
        let mut s = 0.0;
        s = p.expose(s, 20.0, 0.5);
        assert!(s > 5.0 && s < 12.0, "{s}");
        for _ in 0..3 {
            s = p.expose(s, 20.0, 1.0);
        }
        assert!(s > 18.0 && s < 20.0, "{s}");
        let cooled = p.expose(s, 0.0, 1.0);
        assert!(cooled < s / 2.0, "{cooled}");
        // At the reference dryness the threshold is the covering's own; drier needs less, wetter
        // more.
        let r = p.reference_moisture;
        assert!((p.threshold(10.0, r).expect("catches") - 10.0).abs() < 1e-9);
        assert!(p.threshold(10.0, 0.08).expect("catches") < 10.0);
        assert!(p.threshold(10.0, 0.25).expect("catches") > 15.0);
        assert_eq!(p.threshold(10.0, 0.4), None);
        assert_eq!(p.threshold(0.0, 0.1), None);
        // Dryness scales an ignition hazard the other way: 1 at the reference, more for drier
        // fuel, none at the moisture of extinction.
        assert!((p.ignition_scale(r) - 1.0).abs() < 1e-12);
        assert!(p.ignition_scale(0.06) > 1.0 && p.ignition_scale(0.25) < 0.5);
        assert_eq!(p.ignition_scale(0.4), 0.0);
        // A wet roof's water takes the heat first: 10 kW/m² on 30 m² boils about 8 L a minute.
        assert!((p.boiled_l(10.0, 30.0, 1.0) - 7.976).abs() < 0.01);
        assert_eq!(p.boiled_l(-1.0, 30.0, 1.0), 0.0);
        let round = Outline::Round {
            x: 0.0,
            y: 0.0,
            r: 2.0,
        };
        assert!((round.area_m2() - std::f64::consts::PI * 4.0).abs() < 1e-9);
        // Below the threshold it never catches; above it, the more so the higher it stands.
        assert_eq!(p.catch_chance(9.0, 10.0, 1.0), 0.0);
        let a = p.catch_chance(15.0, 10.0, 1.0);
        let b = p.catch_chance(30.0, 10.0, 1.0);
        assert!(a > 0.0 && b > a && b < 1.0, "{a} {b}");
    }

    #[test]
    fn a_member_exposed_an_hour_on_four_faces_keeps_its_core() {
        // 11-08 §2.5: a 200 mm member at about 0.6 mm/min for an hour keeps a 128 mm core, 41 %
        // of its area and 26 % of its section modulus.
        let p = params();
        let char = p.char_mm(60.0);
        assert!((char - 36.0).abs() < 1e-9);
        let loss = charred_loss(0.0, 200.0, char);
        let left = 1.0 - f64::from(loss);
        assert!((left * 200.0 - 128.0).abs() < 0.01, "{}", left * 200.0);
        assert!((left * left - 0.41).abs() < 0.01);
        assert!((left.powi(3) - 0.26).abs() < 0.01);
        // Faster at first: the first ten minutes char more than a sixth of the hour's.
        assert!(p.char_mm(10.0) > char / 6.0);
        assert!(p.char_mm(10.0) / 10.0 > 0.6 && p.char_mm(10.0) / 10.0 < 1.0);
        // A thin member is lost.
        assert_eq!(charred_loss(0.2, 40.0, 25.0), 1.0);
    }

    #[test]
    fn a_fire_passes_through_its_stages() {
        let p = params();
        assert_eq!(p.stage_at(0.0), Stage::Covering);
        assert_eq!(p.stage_at(1.0), Stage::Involved);
        assert_eq!(p.stage_at(30.9), Stage::Involved);
        assert_eq!(p.stage_at(31.0), Stage::Out);
        assert_eq!(p.covering_burnt(7.5), 0.5);
        assert_eq!(p.covering_burnt(20.0), 1.0);
    }

    #[test]
    fn gaps_are_between_roof_edges_and_the_wind_is_read_from_centre_to_centre() {
        let a = Outline::Round {
            x: 0.0,
            y: 0.0,
            r: 3.0,
        };
        let b = Outline::Round {
            x: 7.0,
            y: 0.0,
            r: 3.0,
        };
        assert!((a.gap(&b) - 1.0).abs() < 1e-9);
        let c = Outline::Box {
            x0: 4.0,
            y0: -1.0,
            x1: 8.0,
            y1: 1.0,
        };
        assert!((a.gap(&c) - 1.0).abs() < 1e-9);
        assert!((c.gap(&a) - 1.0).abs() < 1e-9);
        let d = Outline::Box {
            x0: 9.0,
            y0: 3.0,
            x1: 12.0,
            y1: 6.0,
        };
        assert!((c.gap(&d) - (1.0f64).hypot(2.0)).abs() < 1e-9);
        let e = Outline::Box {
            x0: 5.0,
            y0: 0.0,
            x1: 6.0,
            y1: 4.0,
        };
        assert_eq!(c.gap(&e), 0.0);
        // The wind toward the east finds b straight downwind of a.
        assert!((a.toward(&b, (1.0, 0.0)) - 1.0).abs() < 1e-9);
        assert!((b.toward(&a, (1.0, 0.0)) + 1.0).abs() < 1e-9);
        assert!(a.toward(&b, (0.0, 1.0)).abs() < 1e-9);
    }

    /// Lives a block of huts by the model alone, `rows` deep along the wind and `cols` across
    /// it, 6 m roofs `gap_m` apart, `dt` minutes a step, the upwind row lit at once, with each
    /// covering's catch drawn from `draw` (a uniform keyed by hut and step start, in minutes):
    /// the minute each hut caught, if it did.
    fn live_grid(
        rows: usize,
        cols: usize,
        gap_m: f64,
        wind_ms: f64,
        moisture: f64,
        dt: f64,
        draw: &dyn Fn(usize, f64) -> f64,
    ) -> Vec<Option<f64>> {
        let p = params();
        let step = 6.0 + gap_m;
        let outlines: Vec<_> = (0..rows * cols)
            .map(|i| Outline::Round {
                x: (i / cols) as f64 * step,
                y: (i % cols) as f64 * step,
                r: 3.0,
            })
            .collect();
        let huts = outlines.len();
        let wind = (1.0, 0.0);
        let mut caught: Vec<Option<f64>> = vec![None; huts];
        for c in caught.iter_mut().take(cols) {
            *c = Some(0.0);
        }
        let mut store = vec![0.0; huts];
        let mut t = 0.0;
        while t < 120.0 {
            let lit: Vec<(usize, f64)> = (0..huts)
                .filter_map(|i| caught[i].map(|c| (i, c)))
                .collect();
            for j in 0..huts {
                if caught[j].is_some() {
                    continue;
                }
                let heat: f64 = lit
                    .iter()
                    .map(|&(i, c)| {
                        let o = &outlines[i];
                        p.heat_on(
                            p.stage_at(t - c),
                            o.gap(&outlines[j]),
                            wind_ms,
                            o.toward(&outlines[j], wind),
                        )
                    })
                    .sum();
                store[j] = p.expose(store[j], heat, dt);
                if let Some(th) = p.threshold(10.0, moisture)
                    && draw(j, t) < p.catch_chance(store[j], th, dt)
                {
                    caught[j] = Some(t + dt);
                }
            }
            t += dt;
        }
        caught
    }

    /// A uniform in [0, 1) keyed by hut and minute.
    fn uniform(seed: u64) -> impl Fn(usize, f64) -> f64 {
        move |hut, t| civ_core::Rng64::from_key(&[seed, hut as u64, (t * 1000.0) as u64]).next_f64()
    }

    #[test]
    fn twenty_huts_a_metre_or_two_apart_in_wind_all_burn_within_minutes() {
        // 12-03 §2.2: twenty dwellings 1-2 m apart in 4.2-6.9 m/s of wind, four lit together,
        // were all involved within about five minutes; a dwelling passed 1,000 °C within about a
        // minute and downwind neighbours could catch within another. Five rows of four, the
        // upwind row lit: every hut catches, the last within ten minutes on average (within twice
        // the measured figure: the model's minute steps and authored stages are no fit to one
        // experiment), and in a calm the same block burns slower.
        let last = |wind: f64, seeds: std::ops::Range<u64>| -> f64 {
            let n = seeds.end - seeds.start;
            seeds
                .map(|s| {
                    let caught = live_grid(5, 4, 1.5, wind, 0.12, 1.0, &uniform(s));
                    if wind > 0.0 {
                        assert!(caught.iter().all(Option::is_some), "seed {s}: {caught:?}");
                    }
                    caught
                        .iter()
                        .map(|c| c.unwrap_or(120.0))
                        .fold(0.0, f64::max)
                })
                .sum::<f64>()
                / n as f64
        };
        let windy = last(5.5, 0..20);
        assert!(
            windy <= 10.0,
            "the last hut caught {windy:.1} minutes in, on average"
        );
        let calm = last(0.0, 0..20);
        assert!(calm > windy, "calm {calm:.1}, windy {windy:.1}");
    }

    #[test]
    fn wider_gaps_wet_thatch_or_still_air_lose_fewer_huts_and_no_gap_is_a_guarantee() {
        let lost = |gap: f64, wind: f64, moisture: f64| -> f64 {
            (0..40)
                .map(|s| {
                    live_grid(5, 4, gap, wind, moisture, 1.0, &uniform(s))
                        .iter()
                        .filter(|c| c.is_some())
                        .count() as f64
                })
                .sum::<f64>()
                / 40.0
        };
        let close = lost(1.5, 5.5, 0.12);
        let wide = lost(6.0, 5.5, 0.12);
        assert!(close > 19.0, "{close}");
        assert!(wide < close, "wide {wide}, close {close}");
        // Huts a metre and a half apart burn whether their thatch is dry or damp; three metres
        // apart, damp thatch holds better than dry.
        let (dry, wet) = (lost(3.0, 5.5, 0.12), lost(3.0, 5.5, 0.3));
        assert!(wet < dry, "wet {wet}, dry {dry}");
        // Even 4 m apart, downwind in a strong wind, some huts beyond the first row still catch.
        let four = lost(4.0, 9.0, 0.10);
        assert!(four > 4.0, "{four}");
    }

    #[test]
    fn half_minute_steps_give_the_same_spread_as_minutes() {
        // 12-03 §2.5 proposes seconds for an active fire and asks that the step's convergence be
        // checked: the kernel steps by the minute, and a half-minute step must not tell a
        // different story.
        let spread = |dt: f64, gap: f64| -> (f64, f64) {
            let runs: Vec<Vec<Option<f64>>> = (0..60)
                .map(|s| live_grid(5, 4, gap, 3.0, 0.14, dt, &uniform(s)))
                .collect();
            let n = runs.len() as f64;
            let lost = runs
                .iter()
                .map(|r| r.iter().filter(|c| c.is_some()).count() as f64)
                .sum::<f64>()
                / n;
            let last = runs
                .iter()
                .map(|r| r.iter().flatten().fold(0.0f64, |a, &b| a.max(b)))
                .sum::<f64>()
                / n;
            (lost, last)
        };
        for gap in [1.5, 3.0] {
            let (a, ta) = spread(1.0, gap);
            let (b, tb) = spread(0.5, gap);
            assert!(
                (a - b).abs() <= 0.15 * a.max(1.0),
                "gap {gap}: {a} against {b}"
            );
            assert!(
                (ta - tb).abs() <= 0.2 * ta.max(1.0),
                "gap {gap}: {ta} against {tb}"
            );
        }
    }
}
