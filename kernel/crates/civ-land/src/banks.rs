//! Sub-grid bank segments (ADR-0022 §11; research 11-12 §1.5).
//! The earthwork owns construction and its borrow pit; a segment owns the hydraulic crest.

use civ_core::PermanentId;
use civ_world::{WorldMap, hydraulics::FloodBarrier};

use crate::earth::{EarthKind, Earthwork};

/// One straight segment of a bank, linked to an earthwork. Metres from the map corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BankSegment {
    pub work: PermanentId,
    pub borrow: PermanentId,
    pub a: [f64; 2],
    pub b: [f64; 2],
    /// Finished height above the local ground, before settlement and erosion.
    pub height_m: f64,
    pub crest_width_m: f64,
    pub side_run: f64,
    /// Compacted volume per cubic metre excavated in place. Stored with the design.
    pub compacted_per_bank: f64,
    pub settlement_m: f64,
    pub erosion_m: f64,
    pub breached: bool,
    /// Soil washed off the bank, in compacted-equivalent cubic metres. Saved.
    pub sediment_m3: f64,
    /// Seepage damage held against it so far, 0 to 1: at 1 water has piped through and the
    /// bank has given way (M6b slice BG; research 11-12 §4.2). It persists until repaired.
    pub piping: f64,
}

impl BankSegment {
    pub fn length_m(&self) -> f64 {
        (self.a[0] - self.b[0]).hypot(self.a[1] - self.b[1])
    }

    /// Compacted fill required by the trapezoid; never just its digging budget.
    pub fn fill_m3(&self) -> f64 {
        self.length_m()
            * (self.height_m * self.crest_width_m + self.side_run * self.height_m.powi(2))
    }

    /// A rising trapezoid has area C*h + z*h²; a share of its volume is not a share
    /// of its finished height.
    pub fn built_height_m(&self, done: f32) -> f64 {
        let area = (self.height_m * self.crest_width_m + self.side_run * self.height_m.powi(2))
            * f64::from(done);
        ((self.crest_width_m.powi(2) + 4.0 * self.side_run * area).sqrt() - self.crest_width_m)
            / (2.0 * self.side_run)
    }

    pub fn borrow_m3(&self) -> f64 {
        self.fill_m3() / self.compacted_per_bank
    }

    pub fn valid(&self, map: &WorldMap) -> bool {
        let width = f64::from(map.width) * f64::from(map.cell_size_m);
        let height = f64::from(map.height) * f64::from(map.cell_size_m);
        let finite = self
            .a
            .into_iter()
            .chain(self.b)
            .chain([
                self.height_m,
                self.crest_width_m,
                self.side_run,
                self.compacted_per_bank,
                self.settlement_m,
                self.erosion_m,
                self.sediment_m3,
                self.piping,
            ])
            .all(f64::is_finite);
        let length = self.length_m();
        let reach = self.crest_width_m / 2.0 + self.side_run * self.height_m;
        let normal = [
            (self.b[1] - self.a[1]) / length,
            (self.a[0] - self.b[0]) / length,
        ];
        let footprint_inside = [self.a, self.b].iter().all(|p| {
            [-1.0, 1.0].iter().all(|sign| {
                let x = p[0] + sign * normal[0] * reach;
                let y = p[1] + sign * normal[1] * reach;
                x >= 0.0 && y >= 0.0 && x <= width && y <= height
            })
        });
        finite
            && footprint_inside
            && self.length_m() > 0.0
            && self.height_m > 0.0
            && self.crest_width_m > 0.0
            && self.side_run > 0.0
            && self.compacted_per_bank > 0.0
            && self.compacted_per_bank <= 1.0
            && self.settlement_m >= 0.0
            && self.erosion_m >= 0.0
            && self.sediment_m3 >= 0.0
            && (0.0..=1.0).contains(&self.piping)
            && [self.a, self.b]
                .iter()
                .all(|p| p[0] >= 0.0 && p[1] >= 0.0 && p[0] <= width && p[1] <= height)
    }

    /// The least axis-aligned rectangle holding the bank's full trapezoid, rounded out to
    /// whole centimetres: the ground a bank's earthwork reserves.
    pub fn envelope(&self) -> crate::RectCm {
        let length = self.length_m();
        let normal = [
            (self.b[1] - self.a[1]) / length,
            (self.a[0] - self.b[0]) / length,
        ];
        let reach = self.crest_width_m / 2.0 + self.side_run * self.height_m;
        let corners: Vec<[f64; 2]> = [self.a, self.b]
            .iter()
            .flat_map(|p| {
                [-1.0, 1.0].map(|sign| {
                    [
                        p[0] + sign * normal[0] * reach,
                        p[1] + sign * normal[1] * reach,
                    ]
                })
            })
            .collect();
        let x0 = corners.iter().map(|c| c[0]).fold(f64::INFINITY, f64::min);
        let y0 = corners.iter().map(|c| c[1]).fold(f64::INFINITY, f64::min);
        let x1 = corners
            .iter()
            .map(|c| c[0])
            .fold(f64::NEG_INFINITY, f64::max);
        let y1 = corners
            .iter()
            .map(|c| c[1])
            .fold(f64::NEG_INFINITY, f64::max);
        let x = (x0 * 100.0).floor() as i32;
        let y = (y0 * 100.0).floor() as i32;
        crate::RectCm {
            x,
            y,
            w: ((x1 * 100.0).ceil() as i32 - x).max(1),
            h: ((y1 * 100.0).ceil() as i32 - y).max(1),
        }
    }

    /// The reserved envelope must contain the bank's full trapezoid, not just its line.
    pub fn inside_envelope(&self, rect: crate::RectCm) -> bool {
        let length = self.length_m();
        let normal = [
            (self.b[1] - self.a[1]) / length,
            (self.a[0] - self.b[0]) / length,
        ];
        let reach = self.crest_width_m / 2.0 + self.side_run * self.height_m;
        let x0 = f64::from(rect.x) / 100.0;
        let y0 = f64::from(rect.y) / 100.0;
        let x1 = x0 + f64::from(rect.w) / 100.0;
        let y1 = y0 + f64::from(rect.h) / 100.0;
        [self.a, self.b].iter().all(|p| {
            [-1.0, 1.0].iter().all(|sign| {
                let x = p[0] + sign * normal[0] * reach;
                let y = p[1] + sign * normal[1] * reach;
                x >= x0 && x <= x1 && y >= y0 && y <= y1
            })
        })
    }

    /// Edges crossed by the segment. The barrier retains its full sub-grid height, unlike
    /// cell-average ground shading. Unbuilt, eroded and breached segments block nothing.
    pub fn barriers(
        &self,
        map: &WorldMap,
        works: &[Earthwork],
        hands: &[f32],
    ) -> Vec<FloodBarrier> {
        let Some(work) = works
            .iter()
            .find(|w| w.id == self.work && w.kind == EarthKind::Bank)
        else {
            return Vec::new();
        };
        let height = (self.built_height_m(work.done) - self.settlement_m - self.erosion_m).max(0.0);
        if self.breached || height <= 0.0 {
            return Vec::new();
        }
        self.barriers_at(map, hands, height)
    }

    /// The edges the segment would block if it stood `height` metres above its ground: a
    /// design's crest before any earth is placed (M6b slice BG: choosing where to build).
    pub fn barriers_at(&self, map: &WorldMap, hands: &[f32], height: f64) -> Vec<FloodBarrier> {
        let cell = f64::from(map.cell_size_m);
        let min_x = (self.a[0].min(self.b[0]) / cell).floor().max(0.0) as u32;
        let min_y = (self.a[1].min(self.b[1]) / cell).floor().max(0.0) as u32;
        let max_x = (self.a[0].max(self.b[0]) / cell)
            .ceil()
            .min(f64::from(map.width.saturating_sub(1))) as u32;
        let max_y = (self.a[1].max(self.b[1]) / cell)
            .ceil()
            .min(f64::from(map.height.saturating_sub(1))) as u32;
        let mut out = Vec::new();
        for y in min_y.saturating_sub(1)..=max_y {
            for x in min_x.saturating_sub(1)..=max_x {
                let a = y * map.width + x;
                let centre = [(f64::from(x) + 0.5) * cell, (f64::from(y) + 0.5) * cell];
                for b in [
                    (x + 1 < map.width).then(|| a + 1),
                    (y + 1 < map.height).then(|| a + map.width),
                ]
                .into_iter()
                .flatten()
                {
                    let next = [
                        (f64::from(b % map.width) + 0.5) * cell,
                        (f64::from(b / map.width) + 0.5) * cell,
                    ];
                    if crosses(self.a, self.b, centre, next) {
                        out.push(FloodBarrier {
                            a,
                            b,
                            crest_hand_m: f64::from(hands[a as usize].max(hands[b as usize]))
                                + height,
                        });
                    }
                }
            }
        }
        out
    }
}

/// Ground differences under a bank's reserved envelope and its borrow pit. The geometry
/// and pit stay in the hydraulic account after a breach; cells shared by envelopes count once.
pub fn ground_changes(
    banks: &[BankSegment],
    works: &[Earthwork],
    map: &WorldMap,
    ground: &crate::earth::GroundDelta,
) -> Vec<(u32, f32)> {
    let mut cells = std::collections::BTreeSet::new();
    let cell = f64::from(map.cell_size_m);
    for bank in banks {
        if !bank.valid(map) {
            continue;
        }
        for id in [bank.work, bank.borrow] {
            let Some(work) = works.iter().find(|w| w.id == id) else {
                continue;
            };
            let r = work.rect;
            let x0 = (f64::from(r.x) / 100.0 / cell)
                .floor()
                .max(0.0)
                .min(f64::from(map.width)) as u32;
            let y0 = (f64::from(r.y) / 100.0 / cell)
                .floor()
                .max(0.0)
                .min(f64::from(map.height)) as u32;
            let x1 = ((f64::from(r.x) + f64::from(r.w)) / 100.0 / cell)
                .ceil()
                .max(0.0)
                .min(f64::from(map.width)) as u32;
            let y1 = ((f64::from(r.y) + f64::from(r.h)) / 100.0 / cell)
                .ceil()
                .max(0.0)
                .min(f64::from(map.height)) as u32;
            for y in y0..y1 {
                for x in x0..x1 {
                    cells.insert(y * map.width + x);
                }
            }
        }
    }
    cells
        .into_iter()
        .filter_map(|c| {
            let change = ground.at(c as usize);
            (change != 0.0).then_some((c, change))
        })
        .collect()
}

/// Authored damage priors, not annual failure probabilities (research 11-12 §4.2: separate
/// the load from the resistance; damage persists across events unless repaired).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BankParams {
    /// Crest metres eroded per day per metre of overflow head to the power 1.5.
    pub erosion_per_head_day: f64,
    /// Seepage damage a day per metre of water held against a bank below its crest: it gives
    /// way by piping once its damage reaches 1.
    pub piping_per_head_day: f64,
    /// The share of its built height placed fill settles by as it consolidates, and the time
    /// constant of that settling, days: design priors (research 11-12 names crest settlement
    /// among a levee's defects but gives no figure).
    pub settle_share: f64,
    pub settle_days: f64,
}

impl Default for BankParams {
    fn default() -> Self {
        Self {
            erosion_per_head_day: 0.02,
            piping_per_head_day: 0.01,
            settle_share: 0.05,
            settle_days: 365.0,
        }
    }
}

/// One day's settling of a bank's placed fill (M6b slice BG): its crest sinks toward
/// `settle_share` of its built height, most of the way within `settle_days`. Settling only
/// deepens; fill placed later settles in its turn. Mass is unchanged: the fill only packs down.
pub fn settle_day(bank: &mut BankSegment, work: &Earthwork, params: BankParams) -> bool {
    if work.done <= 0.0 || params.settle_share <= 0.0 || params.settle_days <= 0.0 {
        return false;
    }
    let target = params.settle_share * bank.built_height_m(work.done);
    if target <= bank.settlement_m {
        return false;
    }
    bank.settlement_m += (target - bank.settlement_m) * (1.0 - (-1.0 / params.settle_days).exp());
    true
}

/// One day's seepage through a bank (M6b slice BG; research 11-12 §4.2): water standing against
/// one side of it, below its crest, higher than the other side's water or ground, adds damage by
/// that head; at a damage of 1 water pipes through and the bank gives way before its crest is
/// reached. Uses the elapsed day's depths, as overtopping does.
pub fn pipe_day(
    bank: &mut BankSegment,
    work: &Earthwork,
    map: &WorldMap,
    floodplain: &civ_world::hydraulics::WorldFloodplain,
    depths: &[f32],
    params: BankParams,
    ground: &crate::earth::GroundDelta,
) -> bool {
    if bank.breached || work.done <= 0.0 || params.piping_per_head_day <= 0.0 {
        return false;
    }
    let hands = &floodplain.cell_hand;
    let surface = |c: u32| -> Option<(f64, bool)> {
        let d = f64::from(*depths.get(c as usize)?);
        let ground = f64::from(*hands.get(c as usize)?) + f64::from(ground.at(c as usize));
        Some((ground + d.max(0.0), d > 0.0))
    };
    let head = bank
        .barriers(map, std::slice::from_ref(work), hands)
        .iter()
        .filter_map(|edge| {
            let (a, b) = (surface(edge.a)?, surface(edge.b)?);
            let (high, low) = if a.0 >= b.0 { (a, b) } else { (b, a) };
            high.1
                .then(|| (high.0.min(edge.crest_hand_m) - low.0).max(0.0))
        })
        .fold(0.0_f64, f64::max);
    if head <= 0.0 {
        return false;
    }
    bank.piping = (bank.piping + params.piping_per_head_day * head).min(1.0);
    if bank.piping >= 1.0 {
        bank.breached = true;
    }
    true
}

/// Apply one day's actual overtopping. No head, no wear; soil removed from the bank
/// enters its explicit washed-sediment account rather than disappearing from construction.
pub fn overtop_day(
    bank: &mut BankSegment,
    work: &Earthwork,
    map: &WorldMap,
    floodplain: &civ_world::hydraulics::WorldFloodplain,
    depths: &[f32],
    params: BankParams,
    ground: &mut crate::earth::GroundDelta,
) -> bool {
    if bank.breached || work.done <= 0.0 || params.erosion_per_head_day <= 0.0 {
        return false;
    }
    let hands = &floodplain.cell_hand;
    let head = bank
        .barriers(map, std::slice::from_ref(work), hands)
        .iter()
        .map(|edge| {
            [edge.a, edge.b]
                .into_iter()
                .filter_map(|c| {
                    let d = *depths.get(c as usize)?;
                    (d > 0.0).then(|| {
                        f64::from(hands[c as usize])
                            + f64::from(ground.at(c as usize))
                            + f64::from(d)
                            - edge.crest_hand_m
                    })
                })
                .fold(0.0_f64, f64::max)
        })
        .fold(0.0_f64, f64::max);
    if head <= 0.0 {
        return false;
    }
    let height = bank.built_height_m(work.done);
    let before = (height - bank.erosion_m).max(0.0);
    let loss =
        (params.erosion_per_head_day * head.powf(1.5)).min((before - bank.settlement_m).max(0.0));
    if loss <= 0.0 {
        return false;
    }
    let after = (before - loss).max(0.0);
    let washed = bank.length_m()
        * (bank.crest_width_m * loss + bank.side_run * (before.powi(2) - after.powi(2)));
    let remaining = (f64::from(work.cut_m3) - bank.sediment_m3).max(0.0);
    let washed = washed.min(remaining);
    let samples = bank_samples(bank, height, map);
    let total: f64 = samples.iter().map(|(_, v)| v).sum();
    if total <= 0.0 {
        return false;
    }
    for ((x, y), volume) in samples {
        ground.add(
            (
                f64::from(x) * f64::from(map.cell_size_m) + 0.5,
                f64::from(y) * f64::from(map.cell_size_m) + 0.5,
            ),
            -volume * washed / total,
        );
    }
    bank.erosion_m += loss;
    bank.sediment_m3 += washed;
    bank.breached = after <= bank.settlement_m + 1e-9;
    true
}

fn crosses(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let cross = |x: [f64; 2], y: [f64; 2]| x[0] * y[1] - x[1] * y[0];
    let ab = [b[0] - a[0], b[1] - a[1]];
    let cd = [d[0] - c[0], d[1] - c[1]];
    let ac = [c[0] - a[0], c[1] - a[1]];
    let denominator = cross(ab, cd);
    if denominator.abs() < 1e-12 {
        return false;
    }
    let t = cross(ac, cd) / denominator;
    let u = cross(ac, ab) / denominator;
    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

/// Place excavated earth in a bank. The caller supplies completed earth-moving work, as
/// `earth::dig_out` does for a pit; this physical operation does not award labour or choose it.
/// Returns the bank volume actually moved, capped by the unbuilt design.
pub fn place_from_borrow(
    bank: &BankSegment,
    works: &mut [Earthwork],
    map: &WorldMap,
    ground: &mut crate::earth::GroundDelta,
    bank_m3: f64,
) -> Result<f64, &'static str> {
    if !bank.valid(map) || !bank_m3.is_finite() || bank_m3 < 0.0 {
        return Err("invalid bank design or earth quantity");
    }
    let at = works
        .iter()
        .position(|w| w.id == bank.work && w.kind == EarthKind::Bank)
        .ok_or("bank earthwork missing")?;
    let pit = works
        .iter()
        .position(|w| w.id == bank.borrow && w.kind == EarthKind::Pit && w.deposit.is_none())
        .ok_or("bank borrow source missing")?;
    let before = works[at];
    let source = works[pit];
    let source_inside = source.rect.x >= 0
        && source.rect.y >= 0
        && source.rect.w > 0
        && source.rect.h > 0
        && f64::from(source.rect.x) + f64::from(source.rect.w)
            <= f64::from(map.width) * f64::from(map.cell_size_m) * 100.0
        && f64::from(source.rect.y) + f64::from(source.rect.h)
            <= f64::from(map.height) * f64::from(map.cell_size_m) * 100.0;
    if !bank.inside_envelope(before.rect)
        || !source_inside
        || !source.cut_m3.is_finite()
        || source.cut_m3 < 0.0
        || !before.cut_m3.is_finite()
        || before.cut_m3 < 0.0
        || !(0.0..=1.0).contains(&before.done)
    {
        return Err("invalid bank construction or borrow source");
    }

    let remaining = (bank.fill_m3() - f64::from(before.cut_m3)).max(0.0) / bank.compacted_per_bank;
    let moved = bank_m3.min(remaining);
    if moved <= 0.0 {
        return Ok(0.0);
    }
    let after_fill = (f64::from(before.cut_m3) + moved * bank.compacted_per_bank) as f32;
    let after_done = (f64::from(after_fill) / bank.fill_m3()).min(1.0) as f32;
    let delta = f64::from(after_fill) - f64::from(before.cut_m3);
    let after = Earthwork {
        cut_m3: after_fill,
        done: after_done,
        ..before
    };
    for (state, sign) in [(before, -1.0), (after, 1.0)] {
        let samples = bank_samples(bank, bank.built_height_m(state.done), map);
        let total: f64 = samples.iter().map(|(_, v)| v).sum();
        if total <= 0.0 && state.cut_m3 > 0.0 {
            return Err("bank footprint has no fill storage");
        }
        for ((x, y), volume) in samples {
            ground.add(
                (
                    f64::from(x) * f64::from(map.cell_size_m) + 0.5,
                    f64::from(y) * f64::from(map.cell_size_m) + 0.5,
                ),
                sign * volume * (f64::from(state.cut_m3) - bank.sediment_m3).max(0.0) / total,
            );
        }
    }
    let dug = delta / bank.compacted_per_bank;
    crate::earth::dig_out(&mut works[pit], map, ground, dug);
    works[at].cut_m3 = after_fill;
    works[at].done = after_done;
    Ok(dug)
}

/// Rebuild a bank's cell-average ground from its linear design and placed volume.
/// Borrow pits are replayed separately through `earth::replay`.
pub fn replay(
    bank: &BankSegment,
    work: &Earthwork,
    map: &WorldMap,
    ground: &mut crate::earth::GroundDelta,
) {
    let samples = bank_samples(bank, bank.built_height_m(work.done), map);
    let total: f64 = samples.iter().map(|(_, v)| v).sum();
    if total <= 0.0 {
        return;
    }
    for ((x, y), volume) in samples {
        ground.add(
            (
                f64::from(x) * f64::from(map.cell_size_m) + 0.5,
                f64::from(y) * f64::from(map.cell_size_m) + 0.5,
            ),
            volume * (f64::from(work.cut_m3) - bank.sediment_m3).max(0.0) / total,
        );
    }
}

fn bank_samples(bank: &BankSegment, height: f64, map: &WorldMap) -> Vec<((u32, u32), f64)> {
    let reach = bank.crest_width_m / 2.0 + bank.side_run * height;
    let step = crate::earth::SAMPLE_M;
    let (x0, x1) = (
        (bank.a[0].min(bank.b[0]) - reach).max(0.0),
        (bank.a[0].max(bank.b[0]) + reach).min(f64::from(map.width) * f64::from(map.cell_size_m)),
    );
    let (y0, y1) = (
        (bank.a[1].min(bank.b[1]) - reach).max(0.0),
        (bank.a[1].max(bank.b[1]) + reach).min(f64::from(map.height) * f64::from(map.cell_size_m)),
    );
    let dx = bank.b[0] - bank.a[0];
    let dy = bank.b[1] - bank.a[1];
    let length2 = dx * dx + dy * dy;
    let mut out = Vec::new();
    for iy in 0..((y1 - y0) / step).ceil() as u32 {
        for ix in 0..((x1 - x0) / step).ceil() as u32 {
            let x = x0 + (f64::from(ix) + 0.5) * step;
            let y = y0 + (f64::from(iy) + 0.5) * step;
            let along = ((x - bank.a[0]) * dx + (y - bank.a[1]) * dy) / length2;
            if !(0.0..=1.0).contains(&along) || x >= x1 || y >= y1 {
                continue;
            }
            let distance = ((x - bank.a[0]) * dy - (y - bank.a[1]) * dx).abs() / length2.sqrt();
            let fill =
                (height - (distance - bank.crest_width_m / 2.0).max(0.0) / bank.side_run).max(0.0);
            if fill > 0.0 {
                out.push((
                    (
                        (x / f64::from(map.cell_size_m)) as u32,
                        (y / f64::from(map.cell_size_m)) as u32,
                    ),
                    fill * step * step,
                ));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RectCm,
        earth::{EARTH_VERSION, GroundDelta},
    };
    use civ_core::SimTime;

    #[test]
    fn construction_moves_borrow_earth_and_raises_a_trapezoid_without_free_fill() {
        let map = crate::tests::map();
        let id = |n| PermanentId::from_raw(n).expect("id");
        let bank = BankSegment {
            work: id(1),
            borrow: id(2),
            a: [24.0, 16.0],
            b: [24.0, 40.0],
            height_m: 1.5,
            crest_width_m: 1.0,
            side_run: 2.0,
            compacted_per_bank: 0.9,
            settlement_m: 0.0,
            erosion_m: 0.0,
            breached: false,
            sediment_m3: 0.0,
            piping: 0.0,
        };
        let work = |n, kind, rect| Earthwork {
            id: id(n),
            kind,
            rect,
            level_cm: 0,
            side_run_cm: 200,
            plot: None,
            deposit: None,
            heap: None,
            household: id(3),
            cut_m3: 0.0,
            done: 0.0,
            version: EARTH_VERSION,
            begun: SimTime::ZERO,
        };
        let mut works = [
            work(
                1,
                EarthKind::Bank,
                RectCm {
                    x: 1700,
                    y: 1600,
                    w: 1400,
                    h: 2400,
                },
            ),
            work(
                2,
                EarthKind::Pit,
                RectCm {
                    x: 3400,
                    y: 1600,
                    w: 1000,
                    h: 2400,
                },
            ),
        ];
        let mut ground = GroundDelta::new(map.width, map.height, map.cell_size_m);
        assert!((bank.fill_m3() - 144.0).abs() < 1e-9);
        assert!((bank.borrow_m3() - 160.0).abs() < 1e-9);
        assert_eq!(
            place_from_borrow(&bank, &mut works, &map, &mut ground, 80.0),
            Ok(80.0)
        );
        assert!((f64::from(works[0].done) - 0.5).abs() < 1e-6);
        assert!(bank.built_height_m(works[0].done) > 0.75);
        assert_eq!(
            place_from_borrow(&bank, &mut works, &map, &mut ground, 1000.0),
            Ok(80.0)
        );
        assert_eq!(works[0].done, 1.0);
        assert!((works[1].cut_m3 - 160.0).abs() < 1e-4);
        // Apparent ground volume shrinks under compaction; dry soil mass is conserved.
        assert!((ground.net_m3() + 16.0).abs() < 0.001);
        let changes = ground_changes(&[bank], &works, &map, &ground);
        let area = f64::from(map.cell_size_m).powi(2);
        assert!(
            (changes
                .iter()
                .map(|(_, h)| f64::from(*h) * area)
                .sum::<f64>()
                - ground.net_m3())
            .abs()
                < 0.001
        );
        assert!(changes.iter().any(|(_, h)| *h < 0.0));
        assert!(changes.iter().any(|(_, h)| *h > 0.0));
        let breached = BankSegment {
            breached: true,
            ..bank
        };
        assert_eq!(changes, ground_changes(&[breached], &works, &map, &ground));
        // Controlled overflow: duration increases damage; neither dry nor sub-crest water wears it.
        let mut eroding = bank;
        let floodplain = civ_world::hydraulics::WorldFloodplain {
            cell_hand: vec![0.0; map.cell_count()],
            ..Default::default()
        };
        let hands = &floodplain.cell_hand;
        let dry = vec![0.0; map.cell_count()];
        let prior = BankParams {
            erosion_per_head_day: 0.02,
            piping_per_head_day: 0.0,
            ..BankParams::default()
        };
        assert!(!overtop_day(
            &mut eroding,
            &works[0],
            &map,
            &floodplain,
            &dry,
            prior,
            &mut ground
        ));
        let low: Vec<_> = (0..map.cell_count())
            .map(|c| (0.5 - ground.at(c)).max(0.0))
            .collect();
        assert!(!overtop_day(
            &mut eroding,
            &works[0],
            &map,
            &floodplain,
            &low,
            prior,
            &mut ground
        ));
        let overflow: Vec<_> = (0..map.cell_count())
            .map(|c| (2.0 - ground.at(c)).max(0.0))
            .collect();
        assert!(overtop_day(
            &mut eroding,
            &works[0],
            &map,
            &floodplain,
            &overflow,
            prior,
            &mut ground
        ));
        let first = eroding.erosion_m;
        let first_soil = eroding.sediment_m3;
        assert!(first > 0.0 && first_soil > 0.0 && !eroding.breached);
        assert!((ground.net_m3() + eroding.sediment_m3 + 16.0).abs() < 0.001);
        let overflow: Vec<_> = (0..map.cell_count())
            .map(|c| (2.0 - ground.at(c)).max(0.0))
            .collect();
        assert!(overtop_day(
            &mut eroding,
            &works[0],
            &map,
            &floodplain,
            &overflow,
            prior,
            &mut ground
        ));
        assert!(eroding.erosion_m > first && eroding.sediment_m3 > first_soil);
        let mut rebuilt = GroundDelta::new(map.width, map.height, map.cell_size_m);
        crate::earth::replay(&works[1], &map, &mut rebuilt);
        replay(&eroding, &works[0], &map, &mut rebuilt);
        for c in 0..map.cell_count() {
            assert!((rebuilt.at(c) - ground.at(c)).abs() < 1e-4);
        }
        let huge: Vec<_> = (0..map.cell_count())
            .map(|c| (100.0 - ground.at(c)).max(0.0))
            .collect();
        assert!(overtop_day(
            &mut eroding,
            &works[0],
            &map,
            &floodplain,
            &huge,
            prior,
            &mut ground
        ));
        assert!(eroding.breached);
        assert!(eroding.barriers(&map, &works, hands).is_empty());
        assert!((ground.net_m3() + eroding.sediment_m3 + 16.0).abs() < 0.001);

        let before = ground.clone();
        assert_eq!(
            place_from_borrow(&bank, &mut works, &map, &mut ground, 1.0),
            Ok(0.0)
        );
        assert_eq!(ground, before);
        assert!(place_from_borrow(&bank, &mut works, &map, &mut ground, f64::NAN).is_err());

        // Seepage: water held below the crest on one side adds damage by its head; none without
        // a head; at a damage of 1 the bank gives way though its crest was never reached.
        let mut seeping = bank;
        works[0].done = 1.0;
        let level = GroundDelta::new(map.width, map.height, map.cell_size_m);
        let piping = BankParams {
            piping_per_head_day: 0.25,
            ..BankParams::default()
        };
        assert!(!pipe_day(
            &mut seeping,
            &works[0],
            &map,
            &floodplain,
            &dry,
            piping,
            &level
        ));
        let edges = seeping.barriers(&map, &works[..1], hands);
        assert!(!edges.is_empty());
        let mut one_side = vec![0.0_f32; map.cell_count()];
        for e in &edges {
            one_side[e.a.min(e.b) as usize] = 0.8;
        }
        assert!(pipe_day(
            &mut seeping,
            &works[0],
            &map,
            &floodplain,
            &one_side,
            piping,
            &level
        ));
        let first = seeping.piping;
        assert!(first > 0.0 && first < 1.0 && !seeping.breached);
        // The same water on both sides holds no head.
        let mut level_water = seeping;
        let both = vec![0.8_f32; map.cell_count()];
        assert!(!pipe_day(
            &mut level_water,
            &works[0],
            &map,
            &floodplain,
            &both,
            piping,
            &level
        ));
        assert_eq!(level_water.piping, first);
        for _ in 0..20 {
            pipe_day(
                &mut seeping,
                &works[0],
                &map,
                &floodplain,
                &one_side,
                piping,
                &level,
            );
        }
        assert_eq!(seeping.piping, 1.0);
        assert!(seeping.breached);
        assert!(seeping.barriers(&map, &works[..1], hands).is_empty());

        // Settling: the crest sinks toward its share of the built height and never rises.
        let mut settling = bank;
        let params = BankParams::default();
        let mut last = 0.0;
        for _ in 0..(3 * 365) {
            assert!(settle_day(&mut settling, &works[0], params));
            assert!(settling.settlement_m > last);
            last = settling.settlement_m;
        }
        let target = params.settle_share * bank.built_height_m(1.0);
        assert!(last < target && last > 0.9 * target);
        let unbuilt = Earthwork {
            done: 0.0,
            ..works[0]
        };
        assert!(!settle_day(&mut settling, &unbuilt, params));
    }
}
