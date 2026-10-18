//! Plans laid over the ground (M8c slice CT, first part; ADR-0030 §2). A plan's template is a
//! composition of operations; M8's first is an orthogonal subdivision: a grid of blocks with
//! streets between and round them, each block cut into two rows of parcels back to back, and the
//! block at the origin kept, if the template says so, as a square. A pure function lays a
//! template over the ground as it is: a parcel, a stretch of street or a square on ground the
//! caller's check refuses (water, too steep, a claim the layout bends round) is left out and
//! counted. What it returns is the realised layout, saved as a law's ground once passed (ADR-0030
//! §2: "the realised layout is authoritative state"), so it does not change when the ground does.
//!
//! Plots are axis-aligned rectangles, so a grid runs east and west or north and south: its
//! orientation is which way its blocks' long sides run.

use crate::RectCm;

/// Which way a grid's blocks' long sides run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    EastWest,
    NorthSouth,
}

impl Axis {
    /// Both, in code order.
    pub const ALL: [Axis; 2] = [Axis::EastWest, Axis::NorthSouth];

    /// Its code in saves.
    pub fn code(self) -> u8 {
        match self {
            Axis::EastWest => 0,
            Axis::NorthSouth => 1,
        }
    }

    /// The axis saved as `code`.
    pub fn from_code(code: u8) -> Option<Axis> {
        match code {
            0 => Some(Axis::EastWest),
            1 => Some(Axis::NorthSouth),
            _ => None,
        }
    }

    /// In words.
    pub fn name(self) -> &'static str {
        match self {
            Axis::EastWest => "east and west",
            Axis::NorthSouth => "north and south",
        }
    }
}

/// An orthogonal subdivision (ADR-0030 §2), in metres, as a template's menu sets it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    /// A block's sides: along its long side and across it.
    pub block_m: (f32, f32),
    /// A parcel's frontage on the block's long side, and its depth (at most half the block's
    /// width: two rows back to back).
    pub parcel_m: (f32, f32),
    /// The width of every street.
    pub street_m: f32,
    /// Blocks along the axis and across it.
    pub blocks: (u8, u8),
    /// Whether the block at the origin is kept as a square.
    pub square: bool,
}

/// What a public site is kept for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SiteUse {
    /// A square: ground kept open for gathering.
    Square,
}

impl SiteUse {
    /// Its code in saves.
    pub fn code(self) -> u8 {
        match self {
            SiteUse::Square => 0,
        }
    }

    /// The use saved as `code`.
    pub fn from_code(code: u8) -> Option<SiteUse> {
        match code {
            0 => Some(SiteUse::Square),
            _ => None,
        }
    }
}

/// A public site a plan keeps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Site {
    pub rect: RectCm,
    pub use_: SiteUse,
}

/// A parcel a plan lays out, and the block it is in (counting along the axis first, from the
/// north-west).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parcel {
    pub rect: RectCm,
    pub block: u16,
}

/// A plan as laid over the ground: its origin, metres, and its axis; its streets (in stretches of
/// a block's side and the crossings between them), public sites and parcels, as realised; and how
/// many of each the ground refused.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    pub origin: (f32, f32),
    pub axis: Option<Axis>,
    pub streets: Vec<RectCm>,
    pub sites: Vec<Site>,
    pub parcels: Vec<Parcel>,
    pub refused: Refused,
}

/// What the ground refused of a layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Refused {
    pub streets: u32,
    pub sites: u32,
    pub parcels: u32,
}

impl Layout {
    /// The ground it reserves (streets and sites), square metres.
    pub fn reserved_m2(&self) -> f64 {
        self.streets
            .iter()
            .chain(self.sites.iter().map(|s| &s.rect))
            .map(|r| f64::from(r.w) * f64::from(r.h) / 1e4)
            .sum()
    }

    /// Whether `rect` overlaps ground it reserves, or comes within `gap_cm` of it.
    pub fn reserves(&self, rect: &RectCm, gap_cm: i32) -> bool {
        self.streets
            .iter()
            .chain(self.sites.iter().map(|s| &s.rect))
            .any(|r| r.near(rect, gap_cm))
    }
}

fn cm(m: f32) -> i32 {
    (f64::from(m) * 100.0).round() as i32
}

/// Grid `g` laid with its origin block centred at `origin`, metres, its blocks' long sides
/// running along `axis`; `ground` says whether a rectangle may be laid where it is. A grid with no
/// blocks or a non-positive size lays nothing.
pub fn lay_grid(
    g: &Grid,
    origin: (f32, f32),
    axis: Axis,
    ground: &dyn Fn(&RectCm) -> bool,
) -> Layout {
    let mut out = Layout {
        origin,
        axis: Some(axis),
        ..Layout::default()
    };
    let (nl, nw) = (i32::from(g.blocks.0), i32::from(g.blocks.1));
    let (bl, bw, s) = (cm(g.block_m.0), cm(g.block_m.1), cm(g.street_m));
    let (pf, pd) = (cm(g.parcel_m.0), cm(g.parcel_m.1).min(bw / 2));
    if nl <= 0 || nw <= 0 || bl <= 0 || bw <= 0 || s < 0 || pf <= 0 || pd <= 0 {
        return out;
    }
    // In the grid's own frame: u along the axis, v across it, from the grid's north-west corner;
    // the origin block's centre at the origin.
    let (ou, ov) = (nl / 2, nw / 2);
    let u0 = -(s + ou * (bl + s) + bl / 2);
    let v0 = -(s + ov * (bw + s) + bw / 2);
    let (cx, cy) = (cm(origin.0), cm(origin.1));
    // A rectangle of the grid's frame on the map.
    let place = |u: i32, v: i32, du: i32, dv: i32| match axis {
        Axis::EastWest => RectCm {
            x: cx + u0 + u,
            y: cy + v0 + v,
            w: du,
            h: dv,
        },
        Axis::NorthSouth => RectCm {
            x: cx + v0 + v,
            y: cy + u0 + u,
            w: dv,
            h: du,
        },
    };
    let keep = |r: RectCm, refused: &mut u32, into: &mut Vec<RectCm>| {
        if ground(&r) {
            into.push(r);
        } else {
            *refused += 1;
        }
    };
    // Streets: along each line between and round the blocks, a stretch a block long and a
    // crossing where lines meet.
    let at_u = |i: i32| i * (bl + s);
    let at_v = |j: i32| j * (bw + s);
    for j in 0..=nw {
        for i in 0..=nl {
            keep(
                place(at_u(i), at_v(j), s, s),
                &mut out.refused.streets,
                &mut out.streets,
            );
            if i < nl {
                keep(
                    place(at_u(i) + s, at_v(j), bl, s),
                    &mut out.refused.streets,
                    &mut out.streets,
                );
            }
            if j < nw {
                keep(
                    place(at_u(i), at_v(j) + s, s, bw),
                    &mut out.refused.streets,
                    &mut out.streets,
                );
            }
        }
    }
    // Blocks: the origin's a square if the template keeps one, the others cut into parcels, two
    // rows back to back along the long side, as many as the frontage fits.
    let per_row = bl / pf;
    for j in 0..nw {
        for i in 0..nl {
            let (u, v) = (at_u(i) + s, at_v(j) + s);
            let block = (j * nl + i) as u16;
            if g.square && i == ou && j == ov {
                let rect = place(u, v, bl, bw);
                if ground(&rect) {
                    out.sites.push(Site {
                        rect,
                        use_: SiteUse::Square,
                    });
                } else {
                    out.refused.sites += 1;
                }
                continue;
            }
            // The rows' parcels centred along the block, any slack left at both ends.
            let slack = (bl - per_row * pf) / 2;
            for row in [0, bw - pd] {
                for k in 0..per_row {
                    let rect = place(u + slack + k * pf, v + row, pf, pd);
                    if ground(&rect) {
                        out.parcels.push(Parcel { rect, block });
                    } else {
                        out.refused.parcels += 1;
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> Grid {
        // 10-04 §3.4's neutral fixture: 40 by 80 m blocks, 6 m streets.
        Grid {
            block_m: (80.0, 40.0),
            parcel_m: (10.0, 20.0),
            street_m: 6.0,
            blocks: (3, 3),
            square: true,
        }
    }

    fn area(rs: &[RectCm]) -> f64 {
        rs.iter()
            .map(|r| f64::from(r.w) * f64::from(r.h) / 1e4)
            .sum()
    }

    #[test]
    fn a_grid_lays_streets_round_its_blocks_a_square_at_the_origin_and_parcels_back_to_back() {
        let l = lay_grid(&grid(), (1000.0, 1000.0), Axis::EastWest, &|_| true);
        // Eight blocks of two rows of eight 10 m parcels, the ninth a square.
        assert_eq!(l.parcels.len(), 8 * 16);
        assert_eq!(l.sites.len(), 1);
        let square = l.sites[0].rect;
        assert_eq!(square.centre_m(), (1000.0, 1000.0));
        assert_eq!((square.w, square.h), (8000, 4000));
        // The streets take what the blocks leave of the whole: 3 by 3 blocks of 80 by 40 with
        // 6 m streets between and round (four each way), 264 by 144 m.
        let whole = 264.0 * 144.0;
        assert!((area(&l.streets) - (whole - 9.0 * 80.0 * 40.0)).abs() < 1e-6);
        // Inside, 10-04 §5.6's share for 40 by 80 m blocks and 6 m streets is 19.1%: the border
        // streets make a small grid's share higher.
        let share = area(&l.streets) / whole;
        assert!((0.19..0.30).contains(&share), "{share}");
        // Nothing overlaps: parcels, streets and the square tile without touching.
        let all: Vec<RectCm> = l
            .streets
            .iter()
            .copied()
            .chain(l.sites.iter().map(|s| s.rect))
            .chain(l.parcels.iter().map(|p| p.rect))
            .collect();
        for (a, x) in all.iter().enumerate() {
            for y in &all[a + 1..] {
                assert!(!x.near(y, 0), "{x:?} and {y:?}");
            }
        }
        assert_eq!(l.refused, Refused::default());
        assert!(l.reserves(&square, 0));
        assert!(!l.reserves(&l.parcels[0].rect, 0));
    }

    #[test]
    fn a_grid_running_north_and_south_is_the_same_grid_turned_and_refused_ground_is_left_out() {
        let ns = lay_grid(&grid(), (0.0, 0.0), Axis::NorthSouth, &|_| true);
        let ew = lay_grid(&grid(), (0.0, 0.0), Axis::EastWest, &|_| true);
        assert_eq!(ns.parcels.len(), ew.parcels.len());
        let p = ns.parcels[0].rect;
        assert_eq!((p.w, p.h), (2000, 1000), "deep across, frontage along");
        // Ground east of x = 50 m refused: what lies on it is left out and counted.
        let west = lay_grid(&grid(), (0.0, 0.0), Axis::EastWest, &|r| r.x + r.w <= 5000);
        assert!(west.parcels.len() < ew.parcels.len());
        assert_eq!(
            west.parcels.len() as u32 + west.refused.parcels,
            ew.parcels.len() as u32
        );
        assert!(west.parcels.iter().all(|p| p.rect.x + p.rect.w <= 5000));
        assert!(west.refused.streets > 0);
        // A grid of nothing lays nothing.
        let none = Grid {
            blocks: (0, 3),
            ..grid()
        };
        assert!(
            lay_grid(&none, (0.0, 0.0), Axis::EastWest, &|_| true)
                .parcels
                .is_empty()
        );
    }
}
