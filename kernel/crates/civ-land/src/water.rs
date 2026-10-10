//! Water under the ground, and rivers by the day (M6a slice AY; ADR-0021 §1–§2; research 03-02
//! §1.1, §1.3, §2.2, §5.1–5.2).
//!
//! What the reference soil cannot hold on a day (its surplus, [`crate::weather::DayWater`]) is
//! split: a share drains down to the shallow aquifer and the rest runs off at once. The aquifer
//! keeps one head per habitat patch (128 m at 8 m cells, inside 03-02 §5.1's 64–128 m) on 03-02
//! §1.3's storage equation: `Sy A dh/dt = R A + Σ C (h_j − h) + C_w (stage − h)`, with recharge,
//! exchange with the four neighbouring patches, and exchange with the river, lake or sea in a
//! patch. Where the head reaches the lowest ground of a patch, the water seeps out there. What
//! reaches the water and what runs off enter one runoff store for the world, which drains by a
//! recession; its outflow against the landscape's mean surplus is the day's flow factor, which
//! scales each reach's mean discharge (a simplification of 03-02 §1.1: one store and one weather
//! for the map). The heads and the store are saved; everything else is derived from the map,
//! the content and the world's seed. Each unit of ground (alluvium, weathered rock) draws its
//! conductivity and specific yield once for a world, within 03-02 §2.2's priors. Springs are
//! derived from the heads, never saved.
//!
//! Volumes are cubic metres, heads and elevations metres above the map's datum, time in days.

use civ_core::rng::Rng64;
use civ_world::{WATER_LAKE, WATER_LAND, WATER_OCEAN, WorldMap};

use crate::{HabitatRule, Patches};

/// Purpose tag for the aquifer's draws.
const PURPOSE_AQUIFER: u64 = 0x0061_7175_6966_6572; // "aquifer"

/// The most explicit sub-steps a day is split into; a world whose ground would need more is
/// stepped with this many, each as large as is stable for the fastest patch.
const MAX_SUBSTEPS: u32 = 2000;

/// Gauss–Seidel over-relaxation for settling the heads (projected SOR).
const SETTLE_OMEGA: f64 = 1.85;

/// Settling stops once no head moves more than this in a sweep, metres.
const SETTLE_TOLERANCE_M: f64 = 1e-5;

/// Settling stops after this many sweeps whatever is left.
const SETTLE_SWEEPS: u32 = 50_000;

/// One unit of ground the aquifer lies in, by the habitats over it (ADR-0021 §2).
#[derive(Clone, Debug, PartialEq)]
pub struct AquiferUnit {
    /// What it is, for the observer: "alluvial sand".
    pub name: String,
    /// Habitat ids it lies under; empty for every habitat no earlier unit names.
    pub habitats: Vec<String>,
    /// Least and most hydraulic conductivity, m/day: a world's is drawn between them, evenly in
    /// its logarithm (03-02 §2.2).
    pub k_m_day: [f64; 2],
    /// Least and most specific yield: a world's is drawn evenly between them (03-02 §2.2).
    pub specific_yield: [f64; 2],
    /// Its saturated thickness, metres: its transmissivity is its conductivity times this.
    pub thickness_m: f64,
}

/// How a landscape's water moves under the ground and down its rivers (ADR-0021 §1–§2).
#[derive(Clone, Debug, PartialEq)]
pub struct WaterParams {
    /// The share of the reference soil's surplus that drains down to the aquifer; the rest runs
    /// off at once (03-02 §1.1).
    pub recharge_share: f64,
    /// The runoff store's recession, days: each day it lets go `1 − e^(−1/k)` of what it holds.
    pub recession_days: f64,
    /// The least a spring flows to be a source, m³ a day.
    pub spring_min_m3_day: f64,
    /// Units of ground, first match wins; the last should name no habitats.
    pub units: Vec<AquiferUnit>,
}

impl Default for WaterParams {
    /// A landscape of one unit of ground, an alluvial sand under everything, for tests and
    /// profiles that give none.
    fn default() -> WaterParams {
        WaterParams {
            recharge_share: 0.5,
            recession_days: 3.0,
            spring_min_m3_day: 0.5,
            units: vec![AquiferUnit {
                name: "alluvial sand".to_owned(),
                habitats: Vec::new(),
                k_m_day: [1.0, 10.0],
                specific_yield: [0.15, 0.3],
                thickness_m: 10.0,
            }],
        }
    }
}

impl WaterParams {
    /// The unit of ground under habitat `class` of `habitats`: the first that names it, or the
    /// first that names none.
    pub fn unit_of(&self, habitats: &[HabitatRule], class: u8) -> usize {
        let id = habitats.get(usize::from(class)).map(|h| h.id.as_str());
        self.units
            .iter()
            .position(|u| id.is_some_and(|id| u.habitats.iter().any(|h| h == id)))
            .or_else(|| self.units.iter().position(|u| u.habitats.is_empty()))
            .unwrap_or(0)
    }
}

/// The aquifer as a world's ground makes it: derived from the map, the content and the seed,
/// never saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Aquifer {
    /// Patches west–east and north–south.
    pub cols: u32,
    pub rows: u32,
    /// Each unit's conductivity, m/day, and specific yield, as drawn for this world.
    pub unit_k_m_day: Vec<f64>,
    pub unit_sy: Vec<f64>,
    /// Each patch's unit of ground.
    pub unit: Vec<u8>,
    /// Each patch's land area, m²: recharge falls on it and its head stores water over it.
    pub land_m2: Vec<f64>,
    /// Each patch's specific yield times its land area, m² (what a metre of head holds).
    pub storage_m2: Vec<f64>,
    /// The lowest land cell of each patch, and its elevation: where the head reaches the ground
    /// first. `None` for a patch of water alone.
    pub seep: Vec<Option<(u32, f32)>>,
    /// The water level in each patch with water cells, metres: the mean of its river beds, lake
    /// levels and the sea level over its water cells.
    pub stage: Vec<Option<f64>>,
    /// Conductance to the patch east and the patch south, m²/day (0 at the map's edge).
    pub c_east: Vec<f64>,
    pub c_south: Vec<f64>,
    /// Conductance between a patch and its water, m²/day.
    pub c_water: Vec<f64>,
    /// Explicit sub-steps a day is lived in.
    pub substeps: u32,
    /// The map's land area, m².
    pub total_land_m2: f64,
}

impl Aquifer {
    /// Derives a world's aquifer: each unit's conductivity and specific yield drawn from `seed`,
    /// each patch's unit from its habitat, its lowest land, its water and the conductances
    /// between them.
    pub fn derive(
        map: &WorldMap,
        patches: &Patches,
        habitats: &[HabitatRule],
        params: &WaterParams,
        seed: u64,
    ) -> Aquifer {
        let (cols, rows) = (patches.cols as usize, patches.rows as usize);
        let n = cols * rows;
        let mut unit_k = Vec::with_capacity(params.units.len());
        let mut unit_sy = Vec::with_capacity(params.units.len());
        for (u, unit) in params.units.iter().enumerate() {
            let mut rng = Rng64::from_key(&[seed, PURPOSE_AQUIFER, u as u64]);
            let (lo, hi) = (unit.k_m_day[0].max(1e-12), unit.k_m_day[1].max(1e-12));
            let k = (lo.ln() + (hi.ln() - lo.ln()) * rng.next_f64()).exp();
            let (a, b) = (unit.specific_yield[0], unit.specific_yield[1]);
            let sy = (a + (b - a) * rng.next_f64()).max(1e-4);
            unit_k.push(k);
            unit_sy.push(sy);
        }
        let unit: Vec<u8> = (0..n)
            .map(|p| {
                let class = patches.class.get(p).copied().unwrap_or(0);
                params.unit_of(habitats, class).min(usize::from(u8::MAX)) as u8
            })
            .collect();
        let cell_m2 = f64::from(map.cell_size_m) * f64::from(map.cell_size_m);
        let mut land_cells = vec![0u32; n];
        let mut seep: Vec<Option<(u32, f32)>> = vec![None; n];
        let mut stage_sum = vec![0.0f64; n];
        let mut water_cells = vec![0u32; n];
        for i in 0..map.cell_count() {
            let p = patches.of_cell(i, map.width);
            if p >= n {
                continue;
            }
            let z = map.elevation[i];
            match map.water[i] {
                WATER_LAND => {
                    land_cells[p] += 1;
                    if seep[p].is_none_or(|(_, s)| z < s) {
                        seep[p] = Some((i as u32, z));
                    }
                }
                w => {
                    water_cells[p] += 1;
                    let level = match w {
                        WATER_OCEAN => f64::from(map.sea_level_m),
                        WATER_LAKE => map
                            .lake_id
                            .get(i)
                            .and_then(|&id| map.lakes.get((id as usize).checked_sub(1)?))
                            .map_or(f64::from(z), |l| f64::from(l.level_m)),
                        _ => f64::from(z),
                    };
                    stage_sum[p] += level;
                }
            }
        }
        let t: Vec<f64> = unit
            .iter()
            .map(|&u| {
                let u = usize::from(u);
                unit_k.get(u).copied().unwrap_or(0.0)
                    * params.units.get(u).map_or(0.0, |x| x.thickness_m.max(0.0))
            })
            .collect();
        // Between square patches the interface is as wide as the patches are apart, so a
        // conductance is a transmissivity: the harmonic mean of the two (03-02 §1.3).
        let between = |a: f64, b: f64| {
            if a + b > 0.0 {
                2.0 * a * b / (a + b)
            } else {
                0.0
            }
        };
        let mut c_east = vec![0.0; n];
        let mut c_south = vec![0.0; n];
        for y in 0..rows {
            for x in 0..cols {
                let p = y * cols + x;
                if x + 1 < cols {
                    c_east[p] = between(t[p], t[p + 1]);
                }
                if y + 1 < rows {
                    c_south[p] = between(t[p], t[p + cols]);
                }
            }
        }
        let patch_m = f64::from(patches.patch_cells) * f64::from(map.cell_size_m);
        let stage: Vec<Option<f64>> = (0..n)
            .map(|p| (water_cells[p] > 0).then(|| stage_sum[p] / f64::from(water_cells[p])))
            .collect();
        // Water crossing a patch lies about half a patch from its middle: a channel the patch's
        // length across exchanges with it as two of its own interfaces would (a design prior).
        let c_water: Vec<f64> = (0..n)
            .map(|p| {
                if water_cells[p] == 0 {
                    return 0.0;
                }
                let length = f64::from(water_cells[p]) * f64::from(map.cell_size_m);
                t[p] * (2.0 * length / patch_m.max(1e-9)).min(4.0)
            })
            .collect();
        let land_m2: Vec<f64> = land_cells.iter().map(|&c| f64::from(c) * cell_m2).collect();
        let storage_m2: Vec<f64> = (0..n)
            .map(|p| land_m2[p] * unit_sy.get(usize::from(unit[p])).copied().unwrap_or(0.1))
            .collect();
        // The largest stable sub-step of the explicit update, from the patch whose storage
        // empties fastest into its neighbours and its water.
        let mut fastest: f64 = 0.0;
        for y in 0..rows {
            for x in 0..cols {
                let p = y * cols + x;
                if storage_m2[p] <= 0.0 {
                    continue;
                }
                let mut c = c_east[p] + c_south[p] + c_water[p];
                if x > 0 {
                    c += c_east[p - 1];
                }
                if y > 0 {
                    c += c_south[p - cols];
                }
                fastest = fastest.max(c / storage_m2[p]);
            }
        }
        let substeps = ((fastest / 0.9).ceil() as u32).clamp(1, MAX_SUBSTEPS);
        Aquifer {
            cols: patches.cols,
            rows: patches.rows,
            unit_k_m_day: unit_k,
            unit_sy,
            unit,
            total_land_m2: land_m2.iter().sum(),
            land_m2,
            storage_m2,
            seep,
            stage,
            c_east,
            c_south,
            c_water,
            substeps,
        }
    }

    /// Number of patches.
    pub fn len(&self) -> usize {
        self.unit.len()
    }

    /// Whether there are no patches.
    pub fn is_empty(&self) -> bool {
        self.unit.is_empty()
    }

    /// Whether patch `p`'s head is fixed at its water's level: a patch of water alone.
    fn fixed(&self, p: usize) -> bool {
        self.storage_m2[p] <= 0.0
    }

    /// The flow into patch `p` from its neighbours and its water at heads `h`, m³/day, and the
    /// part of it from its water (negative when the patch gives water to the river).
    fn inflow(&self, h: &[f64], p: usize) -> (f64, f64) {
        let cols = self.cols as usize;
        let (x, y) = (p % cols, p / cols);
        let mut q = 0.0;
        if x + 1 < cols {
            q += self.c_east[p] * (h[p + 1] - h[p]);
        }
        if x > 0 {
            q += self.c_east[p - 1] * (h[p - 1] - h[p]);
        }
        if y + 1 < self.rows as usize {
            q += self.c_south[p] * (h[p + cols] - h[p]);
        }
        if y > 0 {
            q += self.c_south[p - cols] * (h[p - cols] - h[p]);
        }
        let from_water = self.stage[p].map_or(0.0, |s| self.c_water[p] * (s - h[p]));
        (q + from_water, from_water)
    }
}

/// What a day did to the world's water.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WaterDay {
    /// Recharge reaching the aquifer, m³.
    pub recharge_m3: f64,
    /// Groundwater reaching rivers, lakes and the sea, net of what they lost to it, m³.
    pub to_water_m3: f64,
    /// Groundwater seeping out where heads reached the ground, m³.
    pub seeped_m3: f64,
    /// What the runoff store let go, mm over the map's land.
    pub flow_mm: f64,
}

/// A spring: where the head stands at the lowest ground of a patch off the water, and what
/// flows to it (derived from the heads, never saved).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// Its patch.
    pub patch: u32,
    /// The terrain cell it rises at.
    pub cell: u32,
    /// What flows to it from the patches round it at today's heads, m³ a day.
    pub flow_m3_day: f64,
}

/// The world's water under the ground and in its runoff store.
#[derive(Clone, Debug, Default)]
pub struct Water {
    /// The aquifer as the ground makes it. Derived: rebuilt by [`Water::derive`].
    pub aquifer: Aquifer,
    /// The water table's head in each patch, metres. Saved.
    pub heads: Vec<f64>,
    /// Water in the runoff store, mm over the map's land, after today's outflow. Saved.
    pub runoff_mm: f64,
    /// Scratch for a sub-step's inflows. Neither saved nor compared.
    scratch: Vec<f64>,
}

impl PartialEq for Water {
    /// Equal when the aquifer, the heads and the store are; the scratch is not compared.
    fn eq(&self, other: &Water) -> bool {
        self.aquifer == other.aquifer
            && self.heads == other.heads
            && self.runoff_mm == other.runoff_mm
    }
}

impl Water {
    /// The aquifer of a world with no heads yet: call [`Water::settle`] to give it some.
    pub fn derive(
        map: &WorldMap,
        patches: &Patches,
        habitats: &[HabitatRule],
        params: &WaterParams,
        seed: u64,
    ) -> Water {
        Water {
            aquifer: Aquifer::derive(map, patches, habitats, params, seed),
            heads: Vec::new(),
            runoff_mm: 0.0,
            scratch: Vec::new(),
        }
    }

    /// Settles the heads to where a steady mean recharge, `mean_surplus_mm` of surplus a day,
    /// would hold them, and fills the runoff store to what that mean keeps in it. Used when a
    /// world is made and when a save from before groundwater loads (ADR-0021 §2): a world then
    /// lives on from the long-run mean. Returns the sweeps it took.
    pub fn settle(&mut self, params: &WaterParams, mean_surplus_mm: f64) -> u32 {
        let aq = &self.aquifer;
        let n = aq.len();
        let r = (mean_surplus_mm.max(0.0) * params.recharge_share.clamp(0.0, 1.0)) / 1000.0;
        // Start full, each head at its lowest ground (or its water), and let it drain.
        let mut h: Vec<f64> = (0..n)
            .map(|p| match (aq.seep[p], aq.stage[p]) {
                (_, Some(s)) if aq.fixed(p) => s,
                (Some((_, z)), _) => f64::from(z),
                (None, Some(s)) => s,
                (None, None) => 0.0,
            })
            .collect();
        let cols = aq.cols as usize;
        let mut sweeps = 0;
        while sweeps < SETTLE_SWEEPS {
            sweeps += 1;
            let mut moved: f64 = 0.0;
            for p in 0..n {
                if aq.fixed(p) {
                    continue;
                }
                let (x, y) = (p % cols, p / cols);
                let (mut c, mut ch) = (0.0, r * aq.land_m2[p]);
                if x + 1 < cols {
                    c += aq.c_east[p];
                    ch += aq.c_east[p] * h[p + 1];
                }
                if x > 0 {
                    c += aq.c_east[p - 1];
                    ch += aq.c_east[p - 1] * h[p - 1];
                }
                if y + 1 < aq.rows as usize {
                    c += aq.c_south[p];
                    ch += aq.c_south[p] * h[p + cols];
                }
                if y > 0 {
                    c += aq.c_south[p - cols];
                    ch += aq.c_south[p - cols] * h[p - cols];
                }
                if let Some(s) = aq.stage[p] {
                    c += aq.c_water[p];
                    ch += aq.c_water[p] * s;
                }
                let ground = aq.seep[p].map_or(f64::INFINITY, |(_, z)| f64::from(z));
                // Where nothing drains it, a patch fills to its lowest ground.
                let target = if c > 0.0 { ch / c } else { ground };
                let next = (h[p] + SETTLE_OMEGA * (target - h[p])).min(ground);
                moved = moved.max((next - h[p]).abs());
                h[p] = next;
            }
            if moved < SETTLE_TOLERANCE_M {
                break;
            }
        }
        self.heads = h;
        // In the long run the store lets go its mean input each day: S (e^(1/k) − 1) = mean.
        let k = params.recession_days.max(1e-3);
        self.runoff_mm = mean_surplus_mm.max(0.0) / ((1.0 / k).exp() - 1.0);
        sweeps
    }

    /// Lives a day with `surplus_mm` of surplus from the reference soil: recharge, flow between
    /// patches and to the water, seepage, then the runoff store. Every volume is accounted for:
    /// what the aquifer gains is recharge less what reached the water and what seeped out.
    pub fn day(&mut self, params: &WaterParams, surplus_mm: f64) -> WaterDay {
        let aq = &self.aquifer;
        let n = aq.len();
        let mut out = WaterDay::default();
        if self.heads.len() != n {
            return out;
        }
        let surplus = surplus_mm.max(0.0);
        let share = params.recharge_share.clamp(0.0, 1.0);
        let r = surplus * share / 1000.0;
        let steps = aq.substeps.max(1);
        let dt = 1.0 / f64::from(steps);
        self.scratch.resize(n, 0.0);
        for _ in 0..steps {
            // Every flow from the heads at the start of the sub-step, then every head moves.
            for p in 0..n {
                let (q, from_water) = aq.inflow(&self.heads, p);
                self.scratch[p] = if aq.fixed(p) {
                    // A patch of water alone takes in what its neighbours give it.
                    out.to_water_m3 += q * dt;
                    0.0
                } else {
                    out.to_water_m3 -= from_water * dt;
                    q + r * aq.land_m2[p]
                };
            }
            for p in 0..n {
                if aq.fixed(p) {
                    continue;
                }
                out.recharge_m3 += r * aq.land_m2[p] * dt;
                let mut head = self.heads[p] + self.scratch[p] * dt / aq.storage_m2[p];
                if let Some((_, z)) = aq.seep[p]
                    && head > f64::from(z)
                {
                    out.seeped_m3 += (head - f64::from(z)) * aq.storage_m2[p];
                    head = f64::from(z);
                }
                self.heads[p] = head;
            }
        }
        let land = aq.total_land_m2.max(1.0);
        let input = surplus * (1.0 - share) + (out.to_water_m3 + out.seeped_m3) / land * 1000.0;
        let k = params.recession_days.max(1e-3);
        let held = (self.runoff_mm + input).max(0.0);
        let keep = (-1.0 / k).exp();
        out.flow_mm = held * (1.0 - keep);
        self.runoff_mm = held * keep;
        out
    }

    /// What the runoff store let go today, mm over the map's land: derived from what it holds
    /// after letting it go.
    pub fn flow_mm(&self, params: &WaterParams) -> f64 {
        let k = params.recession_days.max(1e-3);
        self.runoff_mm * ((1.0 / k).exp() - 1.0)
    }

    /// Today's flow against the landscape's mean: each reach carries its mean discharge times
    /// this. 1 when the landscape has no mean surplus to measure against.
    pub fn flow_factor(&self, params: &WaterParams, mean_surplus_mm: f64) -> f64 {
        if mean_surplus_mm > 0.0 {
            self.flow_mm(params) / mean_surplus_mm
        } else {
            1.0
        }
    }

    /// The head under patch `p`, metres, if the aquifer has heads.
    pub fn head(&self, p: usize) -> Option<f64> {
        self.heads.get(p).copied()
    }

    /// The spring in patch `p` today, if it has one: a patch off the water whose head stands at
    /// its lowest ground, where what flows to it from the patches round it at today's heads is at
    /// least `spring_min_m3_day`.
    pub fn spring_at(&self, params: &WaterParams, p: usize) -> Option<Spring> {
        let aq = &self.aquifer;
        if self.heads.len() != aq.len() || p >= aq.len() {
            return None;
        }
        let (Some((cell, z)), None) = (aq.seep[p], aq.stage[p]) else {
            return None;
        };
        if self.heads[p] < f64::from(z) - 1e-6 {
            return None;
        }
        let cols = aq.cols as usize;
        let (x, y) = (p % cols, p / cols);
        let mut q = 0.0;
        let mut add = |c: f64, other: usize| {
            q += (c * (self.heads[other] - self.heads[p])).max(0.0);
        };
        if x + 1 < cols {
            add(aq.c_east[p], p + 1);
        }
        if x > 0 {
            add(aq.c_east[p - 1], p - 1);
        }
        if y + 1 < aq.rows as usize {
            add(aq.c_south[p], p + cols);
        }
        if y > 0 {
            add(aq.c_south[p - cols], p - cols);
        }
        (q >= params.spring_min_m3_day).then_some(Spring {
            patch: p as u32,
            cell,
            flow_m3_day: q,
        })
    }

    /// The springs today, in patch order ([`Water::spring_at`]).
    pub fn springs(&self, params: &WaterParams) -> Vec<Spring> {
        (0..self.aquifer.len())
            .filter_map(|p| self.spring_at(params, p))
            .collect()
    }

    /// What is wrong with saved water state for the aquifer, if anything.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if !self.heads.is_empty() && self.heads.len() != self.aquifer.len() {
            out.push(format!(
                "the water table has {} heads for {} patches",
                self.heads.len(),
                self.aquifer.len()
            ));
        }
        if self.heads.iter().any(|h| !h.is_finite()) {
            out.push("the water table has a head that is not a number".to_owned());
        }
        if !(self.runoff_mm.is_finite() && self.runoff_mm >= 0.0) {
            out.push("the runoff store holds an invalid amount".to_owned());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An aquifer of `cols` × `rows` patches of 128 m, every patch of one unit with conductance
    /// `c` to each neighbour and specific yield `sy`, its lowest ground at `ground`; patches in
    /// `water` hold a river at `stage` and exchange with it at `c`.
    fn aquifer(
        cols: u32,
        rows: u32,
        c: f64,
        sy: f64,
        ground: impl Fn(usize) -> f32,
        water: &[(usize, f64)],
    ) -> Aquifer {
        let n = (cols * rows) as usize;
        let area = 128.0 * 128.0;
        let mut aq = Aquifer {
            cols,
            rows,
            unit_k_m_day: vec![c / 10.0],
            unit_sy: vec![sy],
            unit: vec![0; n],
            land_m2: vec![area; n],
            storage_m2: vec![area * sy; n],
            seep: (0..n).map(|p| Some((p as u32, ground(p)))).collect(),
            stage: vec![None; n],
            c_east: vec![0.0; n],
            c_south: vec![0.0; n],
            c_water: vec![0.0; n],
            substeps: 1,
            total_land_m2: area * n as f64,
        };
        let (cols, rows) = (cols as usize, rows as usize);
        for y in 0..rows {
            for x in 0..cols {
                let p = y * cols + x;
                if x + 1 < cols {
                    aq.c_east[p] = c;
                }
                if y + 1 < rows {
                    aq.c_south[p] = c;
                }
            }
        }
        for &(p, s) in water {
            aq.stage[p] = Some(s);
            aq.c_water[p] = c;
        }
        let fastest = (0..n)
            .map(|p| {
                let (x, y) = (p % cols, p / cols);
                let mut k = aq.c_east[p] + aq.c_south[p] + aq.c_water[p];
                if x > 0 {
                    k += aq.c_east[p - 1];
                }
                if y > 0 {
                    k += aq.c_south[p - cols];
                }
                k / aq.storage_m2[p]
            })
            .fold(0.0, f64::max);
        aq.substeps = ((fastest / 0.9).ceil() as u32).clamp(1, MAX_SUBSTEPS);
        aq
    }

    fn params(share: f64) -> WaterParams {
        WaterParams {
            recharge_share: share,
            ..WaterParams::default()
        }
    }

    fn water(aq: Aquifer, heads: Vec<f64>) -> Water {
        Water {
            aquifer: aq,
            heads,
            runoff_mm: 0.0,
            scratch: Vec::new(),
        }
    }

    #[test]
    fn fifty_millimetres_of_recharge_raise_a_closed_aquifer_of_specific_yield_a_fifth_by_a_quarter_metre()
     {
        // 03-02 §4's storage-equation test: 50 mm of net recharge, Sy 0.20, nothing else.
        let aq = aquifer(1, 1, 0.0, 0.2, |_| 100.0, &[]);
        let mut w = water(aq, vec![10.0]);
        let day = w.day(&params(1.0), 50.0);
        assert!((w.heads[0] - 10.25).abs() < 1e-12, "{}", w.heads[0]);
        assert!((day.recharge_m3 - 0.05 * 128.0 * 128.0).abs() < 1e-9);
        assert_eq!(day.to_water_m3, 0.0);
        assert_eq!(day.seeped_m3, 0.0);
    }

    #[test]
    fn every_cubic_metre_is_accounted_for() {
        // A slope of 5 x 4 patches draining to a river along its foot, wet enough to seep at
        // the top: what the aquifer gains is what was recharged, less what reached the river
        // and what seeped out.
        let ground = |p: usize| 40.0 - 8.0 * (p / 5) as f32;
        let river: Vec<(usize, f64)> = (15..20).map(|p| (p, 15.0)).collect();
        let aq = aquifer(5, 4, 50.0, 0.1, ground, &river);
        let start: Vec<f64> = (0..20).map(|p| f64::from(ground(p)) - 1.0).collect();
        let mut w = water(aq, start);
        let p = params(0.6);
        let (mut recharge, mut out) = (0.0, 0.0);
        for d in 0..60 {
            let stored =
                |w: &Water| -> f64 { (0..20).map(|i| w.heads[i] * w.aquifer.storage_m2[i]).sum() };
            let before = stored(&w);
            let day = w.day(&p, if d % 7 == 0 { 40.0 } else { 0.5 });
            let gained = stored(&w) - before;
            assert!(
                (gained - (day.recharge_m3 - day.to_water_m3 - day.seeped_m3)).abs() < 1e-6,
                "day {d}: gained {gained}, accounted {day:?}"
            );
            recharge += day.recharge_m3;
            out += day.to_water_m3 + day.seeped_m3;
        }
        assert!(recharge > 0.0 && out > 0.0);
    }

    #[test]
    fn settled_heads_hold_under_the_mean_and_the_store_lets_go_the_mean() {
        let ground = |p: usize| 30.0 - 2.0 * (p % 6) as f32;
        let river: Vec<(usize, f64)> = (0..5).map(|y| (y * 6 + 5, 18.0)).collect();
        let aq = aquifer(6, 5, 20.0, 0.15, ground, &river);
        let mut w = water(aq, Vec::new());
        let p = params(0.5);
        let mean = 0.8;
        w.settle(&p, mean);
        assert!((w.flow_factor(&p, mean) - 1.0).abs() < 1e-12);
        let settled = w.heads.clone();
        for (i, h) in settled.iter().enumerate() {
            assert!(
                *h <= f64::from(ground(i)) + 1e-9,
                "patch {i} stands above its ground"
            );
        }
        for _ in 0..30 {
            w.day(&p, mean);
        }
        for (a, b) in settled.iter().zip(&w.heads) {
            assert!(
                (a - b).abs() < 1e-3,
                "settled {a}, after a month at the mean {b}"
            );
        }
        assert!((w.flow_factor(&p, mean) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn a_dry_spell_lowers_the_heads_and_the_flow_and_rain_raises_them() {
        let ground = |p: usize| 30.0 - 2.0 * (p % 6) as f32;
        let river: Vec<(usize, f64)> = (0..5).map(|y| (y * 6 + 5, 18.0)).collect();
        let aq = aquifer(6, 5, 20.0, 0.15, ground, &river);
        let mut w = water(aq, Vec::new());
        let p = params(0.5);
        w.settle(&p, 1.0);
        let wet = w.heads.clone();
        for _ in 0..90 {
            w.day(&p, 0.0);
        }
        let factor = w.flow_factor(&p, 1.0);
        assert!(
            factor < 0.5,
            "after three dry months the rivers run at {factor} of the mean"
        );
        assert!(factor > 0.0, "the ground still feeds them");
        assert!(w.heads.iter().zip(&wet).all(|(d, h)| d <= h));
        assert!(w.heads.iter().zip(&wet).any(|(d, h)| d < &(h - 0.1)));
        w.day(&p, 60.0);
        assert!(
            w.flow_factor(&p, 1.0) > 5.0,
            "a storm's runoff passes at once"
        );
    }

    #[test]
    fn a_spring_rises_where_the_head_meets_the_lowest_ground_off_the_water() {
        // Recharge on a high patch flows to a low one with nothing to drain it but seepage.
        let ground = |p: usize| if p == 0 { 50.0 } else { 20.0 };
        let aq = aquifer(2, 1, 100.0, 0.1, ground, &[]);
        let mut w = water(aq, Vec::new());
        let p = params(1.0);
        w.settle(&p, 1.0);
        assert!(
            (w.heads[1] - 20.0).abs() < 1e-9,
            "the low patch fills to its ground"
        );
        let springs = w.springs(&p);
        assert_eq!(springs.len(), 1);
        assert_eq!(springs[0].patch, 1);
        // What reaches it is the high patch's recharge: 1 mm a day over 128 m square.
        assert!(
            (springs[0].flow_m3_day - 16.384).abs() < 0.01,
            "{springs:?}"
        );
        // A spring below the least flow is no source.
        let p = WaterParams {
            spring_min_m3_day: 20.0,
            ..p
        };
        assert!(w.springs(&p).is_empty());
    }

    #[test]
    fn fast_ground_is_stepped_stably() {
        // Open gravel, 1,000 m/day over 10 m: the explicit step needs many sub-steps a day.
        let ground = |p: usize| 30.0 - (p % 8) as f32;
        let river: Vec<(usize, f64)> = (0..8).map(|y| (y * 8 + 7, 20.0)).collect();
        let aq = aquifer(8, 8, 10_000.0, 0.2, ground, &river);
        assert!(aq.substeps > 1);
        let mut w = water(aq, (0..64).map(|p| f64::from(ground(p))).collect());
        let p = params(0.5);
        for d in 0..200 {
            w.day(&p, if d % 10 == 0 { 30.0 } else { 0.0 });
            assert!(
                w.heads
                    .iter()
                    .all(|h| h.is_finite() && *h > 0.0 && *h < 31.0)
            );
        }
    }
}
