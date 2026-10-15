//! Enclosures (M6c slice BK; ADR-0023 §6; research 11-11): a closed line of sectors around a
//! settlement's hearth, raised by a polity's law and laid on the walking grid. Each sector is a
//! stretch of palisade 10-25 m long over the cells under it; one or more are gates. Work is done
//! on the line as a whole and the sectors stand in order as it is done (11-11 §1.3: incomplete
//! works give partial benefit, never the whole). A standing sector's cells are crossed only by
//! climbing, slowly; a gate's cells are the ground while it is open and the sector while it is
//! shut. Posts rot at the foot by wetness: a sector whose posts have rotted through is a gap.
//! Every change that alters walking bumps [`Enclosures::revision`], which the walking grid and
//! every cached route are kept against (ADR-0004 amended), as for crossings.

use civ_core::{PermanentId, SimTime};

/// A gate in a sector (ADR-0023 §6): open or shut by its keeper's choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gate {
    pub open: bool,
}

/// One stretch of an enclosure's line.
#[derive(Clone, Debug, PartialEq)]
pub struct Sector {
    /// Its ends, metres.
    pub a: (f32, f32),
    pub b: (f32, f32),
    /// The cells under it, in order from `a`.
    pub cells: Vec<u32>,
    /// The work it takes, hours.
    pub labour_h: f32,
    /// Its gate, if it is one.
    pub gate: Option<Gate>,
    /// The share of its posts' section lost to rot at the foot (0-1); at 1 it is a gap.
    pub loss: f32,
}

impl Sector {
    /// Its length, metres.
    pub fn length_m(&self) -> f32 {
        (self.b.0 - self.a.0).hypot(self.b.1 - self.a.1)
    }

    /// Whether its posts have rotted through.
    pub fn gap(&self) -> bool {
        self.loss >= 1.0
    }
}

/// An enclosure: its owner, its line and the work done on it.
#[derive(Clone, Debug, PartialEq)]
pub struct Enclosure {
    pub id: PermanentId,
    /// The polity whose law raised it, and the law.
    pub owner: PermanentId,
    pub law: PermanentId,
    /// The settlement it encloses, and the point it is drawn round.
    pub settlement: PermanentId,
    pub centre: (f32, f32),
    pub radius_m: f32,
    /// Its sectors, round the line in order.
    pub sectors: Vec<Sector>,
    /// Hours of work done on it.
    pub work_h: f32,
    /// The hours each household's people worked on it, by household id.
    pub shares: Vec<(PermanentId, f32)>,
    /// The day its latest work began, and how many sessions of work began on it that day.
    pub crew: (i64, u8),
    pub begun: SimTime,
    /// The day its last sector stood, if it has.
    pub finished: Option<i64>,
}

impl Enclosure {
    /// The work its whole line takes, hours.
    pub fn labour_h(&self) -> f32 {
        self.sectors.iter().map(|s| s.labour_h).sum()
    }

    /// How many of its sectors stand, in order round the line, by the work done.
    pub fn standing(&self) -> usize {
        let mut done = self.work_h;
        let mut n = 0;
        for s in &self.sectors {
            if done + 1e-3 < s.labour_h {
                break;
            }
            done -= s.labour_h;
            n += 1;
        }
        n
    }

    /// Whether sector `i` stands as a barrier now: built, not rotted through, and not an open gate.
    pub fn bars(&self, i: usize) -> bool {
        let Some(s) = self.sectors.get(i) else {
            return false;
        };
        i < self.standing() && !s.gap() && s.gate.is_none_or(|g| !g.open)
    }

    /// Whether its whole line stands.
    pub fn whole(&self) -> bool {
        self.standing() == self.sectors.len()
    }

    /// The hours household `household`'s people worked on it.
    pub fn worked_by(&self, household: PermanentId) -> f32 {
        self.shares
            .binary_search_by_key(&household, |s| s.0)
            .map_or(0.0, |i| self.shares[i].1)
    }

    /// `hours` more of household `household`'s people's work on it.
    pub fn add_share(&mut self, household: PermanentId, hours: f32) {
        match self.shares.binary_search_by_key(&household, |s| s.0) {
            Ok(i) => self.shares[i].1 += hours,
            Err(i) => self.shares.insert(i, (household, hours)),
        }
    }

    /// Whether point `p` lies inside its line (the circle it is drawn round).
    pub fn inside(&self, p: (f32, f32)) -> bool {
        (p.0 - self.centre.0).hypot(p.1 - self.centre.1) < self.radius_m
    }
}

/// Every enclosure in the world, in the order begun, and the revision of their barriers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Enclosures {
    pub list: Vec<Enclosure>,
    revision: u32,
}

impl Enclosures {
    /// Enclosures `list` at revision `revision` (a loaded world's).
    pub fn with_revision(list: Vec<Enclosure>, revision: u32) -> Enclosures {
        Enclosures { list, revision }
    }

    /// Changes whenever a sector stands or becomes a gap, or a gate opens or shuts: routes
    /// planned before are not to be trusted. 0 while there has never been one.
    pub fn revision(&self) -> u32 {
        self.revision
    }

    /// Notes that what bars walking changed.
    pub fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1).max(1);
    }

    /// The cells that bar walking, each crossed at `climb` of the ground's speed: what the
    /// walking grid lays over the ground.
    pub fn barriers(&self, climb: f32) -> Vec<(u32, f32)> {
        let mut out = Vec::new();
        for e in &self.list {
            for (i, s) in e.sectors.iter().enumerate() {
                if e.bars(i) {
                    out.extend(s.cells.iter().map(|&c| (c, climb)));
                }
            }
        }
        out
    }

    /// The enclosure numbered `id`, by position.
    pub fn position(&self, id: PermanentId) -> Option<usize> {
        self.list.iter().position(|e| e.id == id)
    }

    /// What is wrong with the record, for the invariant checks.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.list.windows(2).any(|w| w[0].id >= w[1].id) {
            out.push("enclosures are out of order or repeated".to_owned());
        }
        for e in &self.list {
            if e.sectors.len() < 3 {
                out.push(format!("enclosure {} has fewer than three sectors", e.id));
            }
            if !(e.work_h.is_finite() && e.work_h >= 0.0) {
                out.push(format!("enclosure {} has its work malformed", e.id));
            }
            if !(e.radius_m.is_finite() && e.radius_m > 0.0) {
                out.push(format!("enclosure {} has no radius", e.id));
            }
            for s in &e.sectors {
                if s.cells.is_empty()
                    || !(s.labour_h.is_finite() && s.labour_h > 0.0)
                    || !(0.0..=1.0).contains(&s.loss)
                {
                    out.push(format!("enclosure {} has a malformed sector", e.id));
                    break;
                }
            }
            if !e.shares.windows(2).all(|w| w[0].0 < w[1].0) {
                out.push(format!("enclosure {} has its shares out of order", e.id));
            }
        }
        out
    }
}

/// The sectors of a ring of radius `radius_m` round `centre`, each at most `sector_m` long, with
/// `gates` gates spread round it, the first facing `toward` (a bearing, radians, east 0, south
/// positive): each sector's cells on a `width` × `height` map of `cell_m` cells, joined edge to
/// edge so that nobody steps through it corner-wise, and its work `labour_h_per_m` an hour for
/// each metre. A sector off the map has no cells and is left out.
#[allow(clippy::too_many_arguments)]
pub fn ring(
    centre: (f32, f32),
    radius_m: f32,
    sector_m: f32,
    gates: usize,
    toward: f32,
    labour_h_per_m: f32,
    cell_m: f32,
    width: u32,
    height: u32,
) -> Vec<Sector> {
    let circumference = std::f32::consts::TAU * radius_m;
    let n = ((circumference / sector_m.max(1.0)).ceil() as usize).max(6);
    let step = std::f32::consts::TAU / n as f32;
    // Start half a sector before `toward`, so the first sector faces it.
    let start = toward - step / 2.0;
    let point = |k: usize| {
        let t = start + step * k as f32;
        (centre.0 + radius_m * t.cos(), centre.1 + radius_m * t.sin())
    };
    let gates = gates.min(n);
    let gate_at: Vec<usize> = (0..gates).map(|g| g * n / gates.max(1)).collect();
    let mut out = Vec::with_capacity(n);
    for k in 0..n {
        let (a, b) = (point(k), point(k + 1));
        let cells = crate::paths::cells_along_by(&[a, b], cell_m, width, height, |first, _| first);
        if cells.is_empty() {
            continue;
        }
        let length = (b.0 - a.0).hypot(b.1 - a.1);
        out.push(Sector {
            a,
            b,
            cells,
            labour_h: (length * labour_h_per_m).max(0.1),
            gate: gate_at.contains(&k).then_some(Gate { open: true }),
            loss: 0.0,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    fn enclosure(sectors: Vec<Sector>) -> Enclosure {
        Enclosure {
            id: id(9),
            owner: id(1),
            law: id(2),
            settlement: id(3),
            centre: (500.0, 500.0),
            radius_m: 60.0,
            sectors,
            work_h: 0.0,
            shares: Vec::new(),
            crew: (0, 0),
            begun: SimTime::from_minutes(0),
            finished: None,
        }
    }

    #[test]
    fn a_ring_closes_round_its_centre_with_its_gates_and_sectors_stand_in_order() {
        let sectors = ring((500.0, 500.0), 60.0, 20.0, 2, 0.0, 2.0, 10.0, 100, 100);
        let n = sectors.len();
        // 2π·60 m in sectors of at most 20 m.
        assert_eq!(n, 19);
        assert!(sectors.iter().all(|s| s.length_m() <= 20.0 + 1e-3));
        assert_eq!(sectors.iter().filter(|s| s.gate.is_some()).count(), 2);
        // The first faces east, toward the bearing given.
        assert!(sectors[0].gate.is_some());
        assert!(sectors[0].a.0 > 555.0 && sectors[0].b.0 > 555.0);
        // Joined end to end, closed.
        for k in 0..n {
            let (b, a) = (sectors[k].b, sectors[(k + 1) % n].a);
            assert!((b.0 - a.0).abs() < 1e-2 && (b.1 - a.1).abs() < 1e-2);
        }
        // Each sector's cells step edge to edge.
        for s in &sectors {
            for w in s.cells.windows(2) {
                let (x0, y0, x1, y1) = (w[0] % 100, w[0] / 100, w[1] % 100, w[1] / 100);
                assert!(x0.abs_diff(x1) + y0.abs_diff(y1) <= 1, "{w:?}");
            }
        }
        let mut e = enclosure(sectors);
        let whole = e.labour_h();
        assert!((whole - 2.0 * e.sectors.iter().map(Sector::length_m).sum::<f32>()).abs() < 0.1);
        assert_eq!(e.standing(), 0);
        e.work_h = e.sectors[0].labour_h + e.sectors[1].labour_h * 0.5;
        assert_eq!(e.standing(), 1, "a half-built sector does not stand");
        // An open gate does not bar; shut, it does.
        assert!(!e.bars(0));
        e.sectors[0].gate = Some(Gate { open: false });
        assert!(e.bars(0));
        e.work_h = whole;
        assert!(e.whole());
        assert!(e.bars(1) && e.bars(5));
        e.sectors[5].loss = 1.0;
        assert!(!e.bars(5), "a rotted sector is a gap");
        assert!(e.inside((510.0, 495.0)) && !e.inside((600.0, 500.0)));
        let mut all = Enclosures::default();
        all.list.push(e);
        let barred = all.barriers(0.1);
        assert!(barred.iter().all(|b| b.1 == 0.1));
        assert!(!barred.is_empty());
        assert!(all.problems().is_empty(), "{:?}", all.problems());
        all.changed();
        assert_eq!(all.revision(), 1);
    }
}
