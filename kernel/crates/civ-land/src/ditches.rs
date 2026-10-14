//! Irrigation ditches (M6b slice BG; ADR-0022 §11; the M6 fire brief §1.11): a straight channel
//! below the grid from an intake beside a river reach to the field it waters, dug as an
//! earthwork. Its capacity is Manning's at its gradient (research 11-12 §1.6: a canal is a
//! hydraulic connection, not merely a trench), less what silt has filled and weeds slow
//! (11-12 §2.3, §4.4). While its intake is open it takes what the field's root zone lacks, no more
//! than it can carry or the reach can spare; what seeps from it on the way recharges the ground
//! water, and what the field does not hold returns to the river below the intake (research 03-02
//! §1.5: every loss keeps its destination). The water it takes is debited from its reach and every
//! reach below, once.

use crate::earth::{EarthKind, Earthwork};
use civ_core::PermanentId;

/// Authored ditch hydraulics (content API 91). Every value is a design prior unless cited.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DitchParams {
    /// Manning's n of a clean and of a densely weeded earthen ditch, s/m^(1/3) (research 11-12
    /// §2.3: 0.025 for an average unlined canal, 0.040 badly kept).
    pub n_clean: f64,
    pub n_weedy: f64,
    /// The share of its reach's flow an intake may take.
    pub max_share: f64,
    /// The share of what it takes that reaches the field; the rest seeps into the ground
    /// (research 03-02 §1.5: conveyance and application losses are distinct).
    pub conveyance: f64,
    /// The share of what reaches the field that its root zone keeps; the rest runs back to the
    /// river (research 03-02 §2.1: surface application about 60 %).
    pub application: f64,
    /// The least fall of its bed along its length, metres per metre (research 11-12 §2.3: small
    /// earthen canals about 0.05-0.2 %).
    pub min_gradient: f64,
}

impl Default for DitchParams {
    fn default() -> Self {
        Self {
            n_clean: 0.025,
            n_weedy: 0.040,
            max_share: 0.5,
            conveyance: 0.8,
            application: 0.6,
            min_gradient: 0.0005,
        }
    }
}

/// One straight ditch: its earthwork, the reach its intake takes from, the field it waters, its
/// line from intake to field in metres from the map's corner, its section, the fall of its bed,
/// its state, whether its intake is open, and the water it has moved, m³.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DitchSegment {
    pub work: PermanentId,
    pub reach: u32,
    pub field: PermanentId,
    /// The intake end and the field end.
    pub a: [f64; 2],
    pub b: [f64; 2],
    /// Bottom width and depth, metres, and side slope, horizontal per vertical (research 11-12
    /// §2.3: 1.5:1 in ordinary earth).
    pub bottom_m: f64,
    pub depth_m: f64,
    pub side_run: f64,
    /// How far its bed falls from intake to field end, metres.
    pub drop_m: f64,
    /// How weedy it is, 0 clean to 1 densely weeded.
    pub weeds: f64,
    /// Silt on its bed, metres.
    pub silt_m: f64,
    /// Whether its intake is open: its people's choice.
    pub open: bool,
    /// Water it has taken from the river, kept in the field's root zone, lost to the ground on
    /// the way, and returned to the river from the field, m³. Saved.
    pub taken_m3: f64,
    pub kept_m3: f64,
    pub seeped_m3: f64,
    pub returned_m3: f64,
}

impl DitchSegment {
    pub fn length_m(&self) -> f64 {
        (self.a[0] - self.b[0]).hypot(self.a[1] - self.b[1])
    }

    /// The earth its channel takes out, cubic metres in place (research 11-12 §1.5:
    /// V = L d (b + z d)).
    pub fn cut_m3(&self) -> f64 {
        self.length_m() * self.depth_m * (self.bottom_m + self.side_run * self.depth_m)
    }

    /// Its Manning's n, by how weedy it is.
    pub fn roughness(&self, params: &DitchParams) -> f64 {
        params.n_clean + self.weeds.clamp(0.0, 1.0) * (params.n_weedy - params.n_clean)
    }

    /// What it can carry running full above its silt, m³/s: Manning's Q = A R^(2/3) S^(1/2) / n
    /// (research 11-12 §1.6), its bed's fall as the energy gradient. Nothing below the least
    /// gradient: a ditch that does not fall does not run.
    pub fn capacity_m3s(&self, params: &DitchParams) -> f64 {
        let length = self.length_m();
        let d = (self.depth_m - self.silt_m).max(0.0);
        if length <= 0.0 || d <= 0.0 {
            return 0.0;
        }
        let s = self.drop_m / length;
        if s < params.min_gradient {
            return 0.0;
        }
        let area = d * (self.bottom_m + self.side_run * d);
        let wetted = self.bottom_m + 2.0 * d * (1.0 + self.side_run * self.side_run).sqrt();
        let r = area / wetted.max(1e-9);
        area * r.powf(2.0 / 3.0) * s.sqrt() / self.roughness(params).max(1e-6)
    }

    /// Whether its record is whole: finite, of positive section and length, its state in range,
    /// and every cubic metre it took kept, seeped or returned.
    pub fn valid(&self) -> bool {
        let finite = [
            self.a[0],
            self.a[1],
            self.b[0],
            self.b[1],
            self.bottom_m,
            self.depth_m,
            self.side_run,
            self.drop_m,
            self.weeds,
            self.silt_m,
            self.taken_m3,
            self.kept_m3,
            self.seeped_m3,
            self.returned_m3,
        ]
        .iter()
        .all(|v| v.is_finite());
        finite
            && self.length_m() > 0.0
            && self.bottom_m > 0.0
            && self.depth_m > 0.0
            && self.side_run >= 0.0
            && self.drop_m >= 0.0
            && (0.0..=1.0).contains(&self.weeds)
            && (0.0..=self.depth_m).contains(&self.silt_m)
            && [
                self.taken_m3,
                self.kept_m3,
                self.seeped_m3,
                self.returned_m3,
            ]
            .iter()
            .all(|v| *v >= 0.0)
            && (self.kept_m3 + self.seeped_m3 + self.returned_m3 - self.taken_m3).abs()
                <= 1e-6 * self.taken_m3.max(1.0)
    }
}

/// One day's water through a ditch: what it took, kept, lost to the ground and returned, m³.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DitchDay {
    pub taken_m3: f64,
    pub kept_m3: f64,
    pub seeped_m3: f64,
    pub returned_m3: f64,
}

/// One day's water through an open ditch whose earthwork `work` is finished (M6b slice BG):
/// `lacks_m3` is what the field's root zone lacks to be full, `spare_m3s` what its reach still
/// carries today. It takes what, after its losses, would fill the root zone, no more than it can
/// carry or the reach can spare; the field keeps `application` of what reaches it, and the rest
/// returns to the river. Nothing while it is closed, unfinished or the field lacks nothing.
pub fn ditch_day(
    ditch: &DitchSegment,
    work: &Earthwork,
    params: &DitchParams,
    lacks_m3: f64,
    spare_m3s: f64,
) -> DitchDay {
    if !ditch.open || work.kind != EarthKind::Ditch || work.done < 1.0 || lacks_m3 <= 0.0 {
        return DitchDay::default();
    }
    let delivered_share = (params.conveyance * params.application).max(1e-9);
    let can = (ditch.capacity_m3s(params) * 86_400.0)
        .min(params.max_share.clamp(0.0, 1.0) * spare_m3s.max(0.0) * 86_400.0);
    let taken = (lacks_m3 / delivered_share).min(can).max(0.0);
    let reached = taken * params.conveyance.clamp(0.0, 1.0);
    let kept = reached * params.application.clamp(0.0, 1.0);
    DitchDay {
        taken_m3: taken,
        kept_m3: kept,
        seeped_m3: taken - reached,
        returned_m3: reached - kept,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ditch() -> DitchSegment {
        DitchSegment {
            work: PermanentId::from_raw(1).expect("id"),
            reach: 0,
            field: PermanentId::from_raw(2).expect("id"),
            a: [0.0, 0.0],
            b: [100.0, 0.0],
            bottom_m: 0.3,
            depth_m: 0.4,
            side_run: 1.5,
            drop_m: 0.1,
            weeds: 0.0,
            silt_m: 0.0,
            open: true,
            taken_m3: 0.0,
            kept_m3: 0.0,
            seeped_m3: 0.0,
            returned_m3: 0.0,
        }
    }

    #[test]
    fn manning_capacity_falls_with_weeds_silt_and_a_flat_bed() {
        let p = DitchParams::default();
        let d = ditch();
        // 100 m at 0.1 % of a 0.3 m bottom, 0.4 m deep, 1.5:1: A = 0.36 m², P ≈ 1.742 m.
        assert!((d.cut_m3() - 100.0 * 0.4 * (0.3 + 0.6)).abs() < 1e-9);
        let area: f64 = 0.4 * (0.3 + 1.5 * 0.4);
        let wetted = 0.3 + 2.0 * 0.4 * (1.0f64 + 2.25).sqrt();
        let q = area * (area / wetted).powf(2.0 / 3.0) * 0.001f64.sqrt() / 0.025;
        assert!((d.capacity_m3s(&p) - q).abs() < 1e-12);
        // Research 11-12 §4.4: from n 0.025 to 0.040 a canal carries 62.5 % as much.
        let weedy = DitchSegment { weeds: 1.0, ..d };
        assert!((weedy.capacity_m3s(&p) / d.capacity_m3s(&p) - 0.625).abs() < 1e-9);
        let silted = DitchSegment { silt_m: 0.2, ..d };
        assert!(silted.capacity_m3s(&p) < d.capacity_m3s(&p));
        let flat = DitchSegment { drop_m: 0.01, ..d };
        assert_eq!(flat.capacity_m3s(&p), 0.0);
    }

    #[test]
    fn a_day_takes_what_the_field_lacks_within_capacity_and_share_and_keeps_every_loss() {
        let p = DitchParams::default();
        let d = ditch();
        let work = Earthwork {
            id: d.work,
            kind: EarthKind::Ditch,
            rect: crate::RectCm {
                x: 0,
                y: 0,
                w: 10_000,
                h: 100,
            },
            level_cm: 0,
            side_run_cm: 150,
            plot: None,
            deposit: None,
            heap: None,
            household: d.field,
            cut_m3: d.cut_m3() as f32,
            done: 1.0,
            version: crate::earth::EARTH_VERSION,
            begun: civ_core::SimTime::ZERO,
        };
        // A hectare short by 20 mm lacks 200 m³: it takes 200 / (0.8 × 0.6) ≈ 417 m³.
        let day = ditch_day(&d, &work, &p, 200.0, 10.0);
        assert!((day.taken_m3 - 200.0 / 0.48).abs() < 1e-6);
        assert!((day.kept_m3 - 200.0).abs() < 1e-6);
        assert!(
            (day.kept_m3 + day.seeped_m3 + day.returned_m3 - day.taken_m3).abs() < 1e-9,
            "every loss kept"
        );
        // A small river spares only its share.
        let small = ditch_day(&d, &work, &p, 200.0, 0.001);
        assert!((small.taken_m3 - 0.5 * 0.001 * 86_400.0).abs() < 1e-6);
        // Closed, unfinished or nothing lacking: nothing.
        let closed = DitchSegment { open: false, ..d };
        assert_eq!(
            ditch_day(&closed, &work, &p, 200.0, 10.0),
            DitchDay::default()
        );
        let unfinished = Earthwork { done: 0.5, ..work };
        assert_eq!(
            ditch_day(&d, &unfinished, &p, 200.0, 10.0),
            DitchDay::default()
        );
        assert_eq!(ditch_day(&d, &work, &p, 0.0, 10.0), DitchDay::default());
    }
}
