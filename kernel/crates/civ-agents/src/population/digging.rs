//! Digging (ADR-0010 §2; M3b slice Q). At a deposit, a pit or quarry on the body, at its edge
//! nearest the settlement's hearth, deepens as people dig there: its cover is dug as earth and the
//! body at its land profile rule's rate. The cover, and what of the body is unfit for use, go on a
//! spoil heap beside it; the rest is carried home. What is taken of a deposit plus what is left is
//! always what it began with, and the ground loses only what is carried away. A platform that cuts
//! into a buried body finds it. Beside a building, a daub pit gives the earth its walls are daubed
//! with, as they go up and as they are mended; all of that earth goes into the walls.

use civ_land::deposits::{Body, Deposit};
use civ_land::earth::{self, EarthKind, Earthwork};
use civ_land::{LandParams, RectCm};

use super::*;

/// The gap between a pit and its spoil heap, or a daub pit and its plot, metres (a tuning value).
const HEAP_GAP_M: f64 = 0.5;
/// How deep a daub pit is sized to go with its building's daub, metres (a tuning value).
const DAUB_PIT_DEPTH_M: f64 = 1.0;
/// The farthest a daub pit lies from its plot's edge, metres (a tuning value).
const DAUB_PIT_REACH_M: f64 = 12.0;

/// Kilograms of a body in a cubic metre of it: its mass over its volume.
pub(crate) fn density(b: &Body) -> f64 {
    let r = f64::from(b.radius_cm) / 100.0;
    let volume = std::f64::consts::PI * r * r * f64::from(b.thickness_cm) / 100.0;
    if volume > 0.0 {
        b.initial_kg / volume
    } else {
        0.0
    }
}

/// A body's cover and thickness, metres.
fn layers(b: &Body) -> (f64, f64) {
    (
        f64::from(b.top_cm.max(0)) / 100.0,
        f64::from(b.thickness_cm.max(0)) / 100.0,
    )
}

/// Hours a capable adult takes to dig a cubic metre of body `b`'s cover and of the body itself:
/// the cover as earth, at `earth_h`, and the body at the land profile's rate for its good, or as
/// earth when the profile gives none.
pub(crate) fn rates(land: &LandParams, b: &Body, earth_h: f64) -> (f64, f64) {
    let body = land
        .deposits
        .iter()
        .find(|r| r.good == usize::from(b.good))
        .map_or(0.0, |r| r.dig_h_per_m3);
    (earth_h, if body > 0.0 { body } else { earth_h })
}

/// Kilograms of a body's good fit for use that an hour of a capable adult's digging brings, at
/// `rates` hours a cubic metre of its cover and of the body ([`rates`]), its cover spread over
/// the body under it.
pub(crate) fn dig_kg_per_hour(b: &Body, (cover_h, body_h): (f64, f64)) -> f64 {
    let (cover, thick) = layers(b);
    let hours = cover * cover_h + thick * body_h;
    if cover_h <= 0.0 || body_h <= 0.0 || hours <= 0.0 {
        return 0.0;
    }
    density(b) * f64::from(b.quality) * thick / hours
}

/// The pit on deposit `d` not yet dug through its body, by index in the earthworks.
fn open_pit(land: &Land, d: &Deposit) -> Option<usize> {
    let (cover, thick) = layers(&d.body);
    land.earthworks.iter().position(|w| {
        w.kind == EarthKind::Pit && w.deposit == Some(d.id) && w.depth_m() < cover + thick - 1e-3
    })
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

/// Whether a pit or heap could go on `rect`: dry land and clear of every earthwork, plot and
/// field.
fn free(land: &Land, map: &WorldMap, rect: &RectCm) -> bool {
    earth::clear_of_water(map, rect, 0.0, 0.0)
        && !land.earthworks.iter().any(|w| w.rect.near(rect, 0))
        && !land.plots.iter().any(|p| p.rect.near(rect, 0))
        && !land.fields.iter().any(|f| f.rect.near(rect, 0))
}

/// Where a new pit on deposit `d` would go, and its heap: the free square of side `side` on the
/// body nearest `toward` (the settlement's hearth), its heap beside it, toward `toward` first.
/// `None` when there is no such place.
fn new_pit_site(
    land: &Land,
    map: &WorldMap,
    d: &Deposit,
    toward: (f64, f64),
    side: f64,
) -> Option<(RectCm, RectCm)> {
    let centre = (d.body.at_cm.0 as f64 / 100.0, d.body.at_cm.1 as f64 / 100.0);
    let inner = (f64::from(d.body.radius_cm) / 100.0 - side / 2.0).max(0.0);
    let n = (inner / side).ceil() as i64;
    let mut spots: Vec<(f64, f64)> = Vec::new();
    for j in -n..=n {
        for i in -n..=n {
            let at = (centre.0 + i as f64 * side, centre.1 + j as f64 * side);
            if (at.0 - centre.0).hypot(at.1 - centre.1) <= inner + 1e-9 {
                spots.push(at);
            }
        }
    }
    let far = |p: &(f64, f64)| (p.0 - toward.0).hypot(p.1 - toward.1);
    spots.sort_by(|a, b| far(a).total_cmp(&far(b)));
    for at in spots {
        let pit = square(at, side);
        if !free(land, map, &pit) {
            continue;
        }
        // The heap beside it: on the side toward `toward`, else round the other sides.
        let (dx, dy) = (toward.0 - at.0, toward.1 - at.1);
        let step = side + HEAP_GAP_M;
        let first = if dx.abs() >= dy.abs() {
            (dx.signum(), 0.0)
        } else {
            (0.0, dy.signum())
        };
        let first = if first == (0.0, 0.0) {
            (1.0, 0.0)
        } else {
            first
        };
        let sides = [
            first,
            (-first.1, first.0),
            (first.1, -first.0),
            (-first.0, -first.1),
        ];
        for (sx, sy) in sides {
            let heap = square((at.0 + sx * step, at.1 + sy * step), side);
            if free(land, map, &heap) {
                return Some((pit, heap));
            }
        }
    }
    None
}

/// Where someone would dig deposit `d`: its open pit's middle, else a new pit's (nearest
/// `toward`). `None` when there is nowhere to dig it.
pub(super) fn dig_place(
    land: &Land,
    map: &WorldMap,
    d: &Deposit,
    toward: (f64, f64),
    side: f64,
) -> Option<(f32, f32)> {
    match open_pit(land, d) {
        Some(p) => Some(land.earthworks[p].rect.centre_m()),
        None => new_pit_site(land, map, d, toward, side).map(|(pit, _)| pit.centre_m()),
    }
}

/// Someone of household `household`, its settlement's hearth at `toward`, digs `effort_h` hours
/// of a capable adult's work at deposit `deposit`, carrying `carry_kg` at most: at its open pit,
/// or a new one with its heap, down through the cover and the body until the load is full or the
/// work is spent. Returns the kilograms of the good carried home: 0 when the digging reached no
/// body yet, or there is nowhere to dig.
pub(super) fn dig_at(
    ctx: &mut Ctx,
    household: PermanentId,
    toward: (f64, f64),
    deposit: PermanentId,
    effort_h: f64,
    carry_kg: f64,
) -> f64 {
    let rules = ctx.params.digging;
    let Some(di) = ctx.land.deposits.iter().position(|d| d.id == deposit) else {
        return 0.0;
    };
    let d = ctx.land.deposits[di];
    let (cover, thick) = layers(&d.body);
    let (density, quality) = (density(&d.body), f64::from(d.body.quality));
    let (cover_h, body_h) = rates(ctx.land_params, &d.body, rules.h_per_m3);
    if d.left_kg() <= 0.0 || density <= 0.0 || cover_h <= 0.0 || body_h <= 0.0 || effort_h <= 0.0 {
        return 0.0;
    }
    let (pit_i, heap_i) = match open_pit(ctx.land, &d) {
        Some(p) => {
            let heap = ctx.land.earthworks[p].heap;
            match heap.and_then(|h| ctx.land.earthworks.iter().position(|w| w.id == h)) {
                Some(h) => (p, h),
                None => return 0.0,
            }
        }
        None => {
            let Some((pit_rect, heap_rect)) =
                new_pit_site(ctx.land, ctx.map, &d, toward, rules.pit_side_m)
            else {
                return 0.0;
            };
            let (pit_id, heap_id) = (ctx.ids.allocate(), ctx.ids.allocate());
            let map = ctx.map;
            let ground = |r: &RectCm| {
                let (x, y) = r.centre_m();
                (earth::bed_height(map, (f64::from(x), f64::from(y))) * 100.0).round() as i32
            };
            let now = ctx.now;
            let work = |id, kind, rect: RectCm, heap| Earthwork {
                id,
                kind,
                rect,
                level_cm: ground(&rect),
                // Unused: a pit's walls and a heap's sides are below the cell size.
                side_run_cm: 100,
                plot: None,
                deposit: Some(d.id),
                heap,
                household,
                cut_m3: 0.0,
                done: 1.0,
                version: earth::EARTH_VERSION,
                begun: now,
            };
            let pit = work(pit_id, EarthKind::Pit, pit_rect, Some(heap_id));
            let heap = work(heap_id, EarthKind::Spoil, heap_rect, None);
            ctx.land.earthworks.push(pit);
            ctx.land.earthworks.push(heap);
            (ctx.land.earthworks.len() - 2, ctx.land.earthworks.len() - 1)
        }
    };
    let mut pit = ctx.land.earthworks[pit_i];
    let mut heap = ctx.land.earthworks[heap_i];
    let area = f64::from(pit.rect.w) * f64::from(pit.rect.h) / 10_000.0;
    if area <= 0.0 {
        return 0.0;
    }
    // How far down it is, what is left of the cover and the body under it, and what a full load
    // needs of the body.
    let depth = pit.depth_m();
    let cover_left = (cover - depth).max(0.0) * area;
    let body_left = (cover + thick - depth.max(cover)).max(0.0) * area;
    let load = carry_kg.min(d.left_kg() * quality);
    let body_for_load = load / (density * quality).max(1e-9);
    // The cover first, as earth, then the body at its own rate, until the work is spent or the
    // load is full.
    let cover_m3 = cover_left.min(effort_h / cover_h);
    let body_m3 = ((effort_h - cover_m3 * cover_h) / body_h)
        .max(0.0)
        .min(body_for_load.min(body_left));
    let m3 = cover_m3 + body_m3;
    if m3 <= 0.0 {
        return 0.0;
    }
    // Of the body dug, what is fit for use.
    let body_kg = (body_m3 * density).min(d.left_kg());
    let kg = body_kg * quality;
    earth::dig(
        &mut pit,
        &mut heap,
        ctx.map,
        &mut ctx.land.ground,
        m3,
        kg / density,
    );
    ctx.land.earthworks[pit_i] = pit;
    ctx.land.earthworks[heap_i] = heap;
    ctx.land.deposits[di].taken_kg += body_kg;
    kg
}

/// Where the daub pit beside plot `plot` would go: a free square of side `side` as near the
/// plot's edge as it can be, behind the plot (away from `toward`, its settlement's hearth) before
/// beside or in front of it. `None` when no free square lies within [`DAUB_PIT_REACH_M`] of it.
fn daub_pit_site(
    land: &Land,
    map: &WorldMap,
    plot: &RectCm,
    toward: (f64, f64),
    side: f64,
) -> Option<RectCm> {
    let (cx, cy) = plot.centre_m();
    let (cx, cy) = (f64::from(cx), f64::from(cy));
    let (hw, hh) = (f64::from(plot.w) / 200.0, f64::from(plot.h) / 200.0);
    let step = side / 2.0;
    let n = ((hw.max(hh) + side + DAUB_PIT_REACH_M) / step).ceil() as i64;
    let (fx, fy) = (toward.0 - cx, toward.1 - cy);
    let reach = fx.hypot(fy);
    // Its gap from the plot, a half metre at a time; how far it faces the hearth (-1 behind the
    // plot, 1 in front); and where it is.
    let mut spots: Vec<(i64, f64, (f64, f64))> = Vec::new();
    for j in -n..=n {
        for i in -n..=n {
            let at = (cx + i as f64 * step, cy + j as f64 * step);
            let (dx, dy) = (at.0 - cx, at.1 - cy);
            let gap = (dx.abs() - hw - side / 2.0)
                .max(0.0)
                .hypot((dy.abs() - hh - side / 2.0).max(0.0));
            if !(HEAP_GAP_M - 1e-9..=DAUB_PIT_REACH_M).contains(&gap) {
                continue;
            }
            let facing = if reach > 0.0 {
                (dx * fx + dy * fy) / (dx.hypot(dy).max(1e-9) * reach)
            } else {
                0.0
            };
            spots.push(((gap / HEAP_GAP_M).floor() as i64, facing, at));
        }
    }
    spots.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    spots
        .into_iter()
        .map(|(_, _, at)| square(at, side))
        .find(|r| free(land, map, r))
}

/// Digs `m3` of earth for the daub of the building on plot `plot` from the plot's daub pit: begun
/// when first needed, beside the plot and behind it from `toward` (its settlement's hearth), and
/// sized so that the building's daub, `daub_m3` in all, would take it about a metre down. The earth
/// goes into the walls, so the ground loses all of it. Nothing when there is no free ground beside
/// the plot, and the walls are daubed all the same.
pub(super) fn dig_daub(
    ctx: &mut Ctx,
    plot: PermanentId,
    toward: (f64, f64),
    daub_m3: f64,
    m3: f64,
) {
    if m3 <= 0.0 {
        return;
    }
    let Some((rect, household)) = ctx
        .land
        .plots
        .iter()
        .find(|p| p.id == plot)
        .map(|p| (p.rect, p.household))
    else {
        return;
    };
    let at = match ctx
        .land
        .earthworks
        .iter()
        .position(|w| w.kind == EarthKind::Pit && w.plot == Some(plot))
    {
        Some(i) => i,
        None => {
            let side = (daub_m3.max(0.0) / DAUB_PIT_DEPTH_M)
                .sqrt()
                .max(ctx.params.digging.pit_side_m);
            let side = (side * 2.0).ceil() / 2.0;
            let Some(pit) = daub_pit_site(ctx.land, ctx.map, &rect, toward, side) else {
                return;
            };
            let (x, y) = pit.centre_m();
            let level = earth::bed_height(ctx.map, (f64::from(x), f64::from(y)));
            ctx.land.earthworks.push(Earthwork {
                id: ctx.ids.allocate(),
                kind: EarthKind::Pit,
                rect: pit,
                level_cm: (level * 100.0).round() as i32,
                // Unused: a pit's walls are below the cell size.
                side_run_cm: 100,
                plot: Some(plot),
                deposit: None,
                heap: None,
                household,
                cut_m3: 0.0,
                done: 1.0,
                version: earth::EARTH_VERSION,
                begun: ctx.now,
            });
            ctx.land.earthworks.len() - 1
        }
    };
    let mut pit = ctx.land.earthworks[at];
    earth::dig_out(&mut pit, ctx.map, &mut ctx.land.ground, m3);
    ctx.land.earthworks[at] = pit;
}

impl Population {
    /// Where household `household`'s settlement has its hearth, metres, or its home when it
    /// belongs to none. `None` for a household that is no more.
    pub(super) fn hearth_toward(&self, land: &Land, household: PermanentId) -> Option<(f64, f64)> {
        let h = self.household(household)?;
        let at = h
            .settlement
            .and_then(|s| land.settlements.iter().find(|x| x.id == s))
            .map_or(h.home, |s| s.hearth_m);
        Some((f64::from(at.0), f64::from(at.1)))
    }

    /// Platform `work`, as far as it is done, cut into the ground of household `household`'s plot:
    /// its settlement finds every buried deposit under the platform whose cover the cut has gone
    /// through, `who` its finder, and the chronicle says so (ADR-0010 §1).
    pub(super) fn find_by_cutting(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        work: &Earthwork,
    ) {
        let Some(settlement) = self.household(household).and_then(|h| h.settlement) else {
            return;
        };
        if work.kind != EarthKind::Platform || work.done <= 0.0 {
            return;
        }
        let map = ctx.map;
        let bed = |x: f64, y: f64| earth::bed_height(map, (x, y));
        let cut =
            (earth::highest(&work.rect, &bed) - work.level_m()).max(0.0) * f64::from(work.done);
        let (rx0, ry0) = (
            f64::from(work.rect.x) / 100.0,
            f64::from(work.rect.y) / 100.0,
        );
        let (rx1, ry1) = (
            rx0 + f64::from(work.rect.w) / 100.0,
            ry0 + f64::from(work.rect.h) / 100.0,
        );
        let found: Vec<(PermanentId, u16, (f64, f64))> = ctx
            .land
            .deposits
            .iter()
            .filter(|d| !d.body.exposed && d.left_kg() > 0.0)
            .filter(|d| cut >= f64::from(d.body.top_cm) / 100.0)
            .filter_map(|d| {
                let at = (d.body.at_cm.0 as f64 / 100.0, d.body.at_cm.1 as f64 / 100.0);
                let near = (at.0.clamp(rx0, rx1), at.1.clamp(ry0, ry1));
                let reach = f64::from(d.body.radius_cm) / 100.0;
                ((at.0 - near.0).hypot(at.1 - near.1) <= reach).then_some((d.id, d.body.good, at))
            })
            .filter(|&(id, _, _)| !self.knows_deposit(settlement, id))
            .collect();
        for (deposit, good, at) in found {
            self.deposits_known.push(deposits::DepositKnown {
                settlement,
                deposit,
                finder: who,
                at: ctx.now,
            });
            let what = ctx
                .catalog
                .goods
                .get(usize::from(good))
                .map_or_else(|| "a deposit".to_owned(), |g| g.name.to_lowercase());
            self.chronicle_push(
                ctx.now,
                ChronicleKind::DepositFound,
                vec![who],
                Some(settlement),
                Some((at.0 as f32, at.1 as f32)),
                0.0,
                format!("{what} while levelling a plot"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A body of 10 m radius at 2,500 kg a cubic metre, half of it fit for use.
    fn body(top_cm: i32, thickness_cm: i32) -> Body {
        let volume = std::f64::consts::PI * 10.0 * 10.0 * f64::from(thickness_cm) / 100.0;
        Body {
            good: 0,
            at_cm: (0, 0),
            radius_cm: 1_000,
            top_cm,
            thickness_cm,
            quality: 0.5,
            exposed: top_cm == 0,
            initial_kg: volume * 2_500.0,
        }
    }

    #[test]
    fn a_body_slower_to_break_out_than_earth_brings_less_an_hour_and_its_cover_is_dug_as_earth() {
        // Showing: an hour at 12 hours a cubic metre brings a twelfth of a cubic metre's good.
        let bare = body(0, 200);
        assert!((dig_kg_per_hour(&bare, (8.0, 12.0)) - 2_500.0 * 0.5 / 12.0).abs() < 1e-6);
        assert!((dig_kg_per_hour(&bare, (8.0, 8.0)) - 2_500.0 * 0.5 / 8.0).abs() < 1e-6);
        // Under a metre of cover over 2 m of it: each square metre of pit is 8 hours of earth and
        // 24 of the body for 2 m³ of it.
        let covered = body(100, 200);
        let hours = 1.0 * 8.0 + 2.0 * 12.0;
        assert!(
            (dig_kg_per_hour(&covered, (8.0, 12.0)) - 2_500.0 * 0.5 * 2.0 / hours).abs() < 1e-6
        );
        // With no rate, nothing.
        assert_eq!(dig_kg_per_hour(&bare, (8.0, 0.0)), 0.0);
        assert_eq!(dig_kg_per_hour(&covered, (0.0, 12.0)), 0.0);
    }
}
