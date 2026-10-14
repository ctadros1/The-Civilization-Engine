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
            && [self.a, self.b]
                .iter()
                .all(|p| p[0] >= 0.0 && p[1] >= 0.0 && p[0] <= width && p[1] <= height)
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
                sign * volume * f64::from(state.cut_m3) / total,
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
            volume * f64::from(work.cut_m3) / total,
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
        let before = ground.clone();
        assert_eq!(
            place_from_borrow(&bank, &mut works, &map, &mut ground, 1.0),
            Ok(0.0)
        );
        assert_eq!(ground, before);
        assert!(place_from_borrow(&bank, &mut works, &map, &mut ground, f64::NAN).is_err());
    }
}
