//! Flood banks a polity's people build together (M6b slice BG; ADR-0022 §11; research 11-12,
//! 12-08). A household remembers what floods cost it, at the places it saw the water; where a
//! straight bank beside the river would have kept that water out of those places, anyone may
//! propose that the polity build it, and each household is asked an equal share of the work.
//! Its people dig the earth from a borrow pit beside it, carry it a basket at a time, and spread
//! and tamp it: each charged apart, nothing placed that was not dug. Nobody keeps it up unless a
//! law asks (ADR-0022 §11).

use super::*;
use crate::decide::LeveeFacts;
use crate::polity::{BankSite, LawStatus, PolicyKind};
use civ_land::RectCm;
use civ_land::banks::{self, BankSegment};
use civ_land::earth::{self, EarthKind, Earthwork};

/// Where along the line from the river to a place it keeps a bank's crest is laid, as shares of
/// the way (design priors: from beside the river, where ground is often free, to near the place).
const ALONG: [f64; 7] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.75];
/// How far a bank's crest runs either side of its middle, in cells: long enough, at the most,
/// for its ends to reach ground the water does not (design priors).
const HALF_CELLS: [f64; 8] = [1.5, 2.5, 4.0, 6.0, 9.0, 13.0, 18.0, 25.0];
/// The most places a settlement's remembered losses are laid out from, the costliest first.
const MOST_PLACES: usize = 6;
/// Metres between a bank's foot and its borrow pit.
const PIT_GAP_M: f64 = 2.0;

/// A flood bank a polity's people could build together, and what it would save each of the
/// settlement's households a year, hours, as they reckon it from what floods cost them.
#[derive(Clone, Debug)]
pub(crate) struct PublicBank {
    pub(crate) site: BankSite,
    /// The years it would keep water out, over which its work is reckoned.
    pub(crate) life: f64,
    /// The hours a year it would save each household that remembers a loss it would have kept
    /// out, by household id.
    pub(crate) saves: Vec<(PermanentId, f64)>,
    /// How many households its work would be asked of.
    pub(crate) households: usize,
}

impl PublicBank {
    /// The share of its work asked of each household, hours.
    pub(crate) fn share_h(&self) -> f64 {
        self.site.labour_h / self.households.max(1) as f64
    }

    /// The hours a year it would save household `h`.
    pub(crate) fn saves_of(&self, h: PermanentId) -> f64 {
        self.saves
            .binary_search_by_key(&h, |s| s.0)
            .map_or(0.0, |i| self.saves[i].1)
    }

    /// The hours it would save all the households over its life, less its work.
    fn pooled_net_h(&self) -> f64 {
        self.saves.iter().map(|s| s.1).sum::<f64>() * self.life - self.site.labour_h
    }

    /// The cell at its middle: the site a move to build it names.
    pub(crate) fn cell(&self, map: &WorldMap) -> u32 {
        bank_cell(map, &self.site)
    }
}

/// The cell at the middle of bank site `site`.
pub(crate) fn bank_cell(map: &WorldMap, site: &BankSite) -> u32 {
    let mid = (
        ((site.a[0] + site.b[0]) / 2.0) as f32,
        ((site.a[1] + site.b[1]) / 2.0) as f32,
    );
    cell_of(map, mid) as u32
}

/// A place a household remembers a flood cost it: the cell, the water's surface there as height
/// above the nearest drainage (HAND metres), and the hours a year the loss is worth to it.
#[derive(Clone, Copy, Debug)]
struct Place {
    household: PermanentId,
    cell: u32,
    stage: f64,
    year_h: f64,
}

/// The segment a site lays out, before any earth: its work and borrow ids are placeholders.
fn segment_of(site: &BankSite, work: PermanentId, borrow: PermanentId) -> BankSegment {
    BankSegment {
        work,
        borrow,
        a: site.a,
        b: site.b,
        height_m: site.height_m,
        crest_width_m: site.crest_width_m,
        side_run: site.side_run,
        compacted_per_bank: site.compacted_per_bank,
        settlement_m: 0.0,
        erosion_m: 0.0,
        breached: false,
        sediment_m3: 0.0,
        piping: 0.0,
    }
}

/// A square of side `side` metres centred at `at` metres.
fn square(at: [f64; 2], side: f64) -> RectCm {
    let half = side / 2.0;
    RectCm {
        x: ((at[0] - half) * 100.0).floor() as i32,
        y: ((at[1] - half) * 100.0).floor() as i32,
        w: (side * 100.0).ceil() as i32,
        h: (side * 100.0).ceil() as i32,
    }
}

impl Population {
    /// The places households `households` remember a flood cost them, with what each loss is
    /// worth a year to its household: the food lost at what an hour of its own work brings
    /// (research 11-07 §2.3), and the work lost, over the years its people remember.
    fn flood_places(&self, ctx: &Ctx, households: &[PermanentId]) -> Vec<Place> {
        let years = ctx.params.levees.memory_years.max(1e-6);
        let hands = &ctx.land.floodplain.cell_hand;
        let mut out: Vec<Place> = Vec::new();
        for &h in households {
            let Some(x) = self.household(h) else {
                continue;
            };
            if x.flood_losses.is_empty() {
                continue;
            }
            let kcal = self.hour_kcal(ctx, x).max(1e-6);
            for l in &x.flood_losses {
                let Some(&hand) = hands.get(l.cell as usize) else {
                    continue;
                };
                let hours = f64::from(l.kcal) / kcal + f64::from(l.hours);
                let stage = f64::from(hand) + f64::from(l.depth_cm) / 100.0;
                match out
                    .iter_mut()
                    .find(|p| p.household == h && p.cell == l.cell)
                {
                    Some(p) => {
                        p.stage = p.stage.max(stage);
                        p.year_h += hours / years;
                    }
                    None => out.push(Place {
                        household: h,
                        cell: l.cell,
                        stage,
                        year_h: hours / years,
                    }),
                }
            }
        }
        out
    }

    /// Where settlement `settlement`'s people could build a flood bank together (M6b slice BG;
    /// ADR-0022 §11), and what it would save each household: a straight bank beside the river
    /// laid out to stand above the highest water they remember at the places floods cost them,
    /// with its borrow pit beside it, on ground free to dig, whose saving over its life (the
    /// losses each household remembers at the places it would keep dry, at what an hour of its
    /// own work brings) most exceeds its work. A place is kept dry when the river's water,
    /// rising as high as they remember it there, finds no way to it but over the bank's crest
    /// (the reach's own floodplain, past every bank standing). None when no such bank would
    /// repay its work, the polity is building one already, or content has no policy to build
    /// one. Households that remember no loss gain nothing from it, and are asked their share.
    pub(crate) fn public_bank(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        households: &[PermanentId],
    ) -> Option<PublicBank> {
        if !ctx
            .catalog
            .policies
            .iter()
            .any(|d| d.kind == PolicyKind::BuildBank)
        {
            return None;
        }
        let places = self.flood_places(ctx, households);
        if places.is_empty() {
            return None;
        }
        let pi = self.polity_of(settlement)?;
        let polity = &self.polities[pi];
        // One bank at a time, as one crossing.
        let building = polity.banks.iter().any(|(_, s)| {
            s.work.is_some_and(|w| {
                ctx.land
                    .earthworks
                    .iter()
                    .any(|e| e.id == w && e.kind == EarthKind::Bank && e.done < 1.0)
            })
        });
        if building {
            return None;
        }
        let lp = &ctx.params.levees;
        let map = ctx.map;
        let fp = &ctx.land.floodplain;
        let cell_m = f64::from(map.cell_size_m);
        let standing: Vec<_> = ctx
            .land
            .banks
            .iter()
            .flat_map(|b| b.barriers(map, &ctx.land.earthworks, &fp.cell_hand))
            .collect();
        let changes =
            banks::ground_changes(&ctx.land.banks, &ctx.land.earthworks, map, &ctx.land.ground);
        let speed = interpolate(&ctx.params.walk_speed_by_age, 30.0).max(0.05) * 60.0;
        let placeholder = PermanentId::from_raw(1).expect("id");
        let centre = |c: u32| {
            let (x, y) = cell_centre(map, c as usize);
            [f64::from(x), f64::from(y)]
        };
        // The places, by reach.
        let mut reaches: BTreeMap<u32, Vec<Place>> = BTreeMap::new();
        for p in &places {
            let Some(&r) = fp.cell_reach.get(p.cell as usize) else {
                continue;
            };
            if r != u32::MAX {
                reaches.entry(r).or_default().push(*p);
            }
        }
        let mut best: Option<PublicBank> = None;
        for (reach, group) in reaches {
            let Some(river) = map.reaches.iter().find(|r| r.id == reach) else {
                continue;
            };
            if river.cells.is_empty() {
                continue;
            }
            let crest = group
                .iter()
                .map(|p| p.stage)
                .fold(f64::NEG_INFINITY, f64::max)
                + lp.freeboard_m;
            // The costliest places first.
            let mut cells: Vec<(u32, f64)> = Vec::new();
            for p in &group {
                match cells.iter_mut().find(|c| c.0 == p.cell) {
                    Some(c) => c.1 += p.year_h,
                    None => cells.push((p.cell, p.year_h)),
                }
            }
            cells.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            cells.truncate(MOST_PLACES);
            for &(target, _) in &cells {
                let t = centre(target);
                let Some(r) = river.cells.iter().map(|&c| centre(c)).min_by(|a, b| {
                    let da = (a[0] - t[0]).hypot(a[1] - t[1]);
                    let db = (b[0] - t[0]).hypot(b[1] - t[1]);
                    da.total_cmp(&db)
                }) else {
                    continue;
                };
                let dist = (t[0] - r[0]).hypot(t[1] - r[1]);
                if dist < 2.0 * cell_m {
                    continue;
                }
                let dir = [(t[0] - r[0]) / dist, (t[1] - r[1]) / dist];
                let along_river = [-dir[1], dir[0]];
                for f in ALONG {
                    let mid = [r[0] + dir[0] * dist * f, r[1] + dir[1] * dist * f];
                    for half_cells in HALF_CELLS {
                        let half = half_cells * cell_m;
                        let a = [
                            mid[0] - along_river[0] * half,
                            mid[1] - along_river[1] * half,
                        ];
                        let b = [
                            mid[0] + along_river[0] * half,
                            mid[1] + along_river[1] * half,
                        ];
                        let Some(p) = self.bank_candidate(
                            ctx,
                            (reach, &group, crest),
                            (a, b, mid, dir),
                            (&standing, &changes),
                            (speed, placeholder),
                            households.len(),
                        ) else {
                            continue;
                        };
                        if best
                            .as_ref()
                            .is_none_or(|b| p.pooled_net_h() > b.pooled_net_h())
                        {
                            best = Some(p);
                        }
                    }
                }
            }
        }
        best.filter(|p| p.pooled_net_h() > 0.0)
    }

    /// A bank from `a` to `b` against reach `reach`'s water, laid out to stand at `crest` (HAND
    /// metres) at its lowest ground, with its borrow pit on the river's side of it (`dir` points
    /// from the river to the places) or else the other: what it would cost and keep dry. None
    /// where it would stand too high, leave the map, take ground not free to dig, or keep
    /// nothing dry.
    #[allow(clippy::type_complexity)]
    fn bank_candidate(
        &self,
        ctx: &Ctx,
        (reach, group, crest): (u32, &[Place], f64),
        (a, b, mid, dir): ([f64; 2], [f64; 2], [f64; 2], [f64; 2]),
        (standing, changes): (&[civ_world::hydraulics::FloodBarrier], &[(u32, f32)]),
        (speed, placeholder): (f64, PermanentId),
        households: usize,
    ) -> Option<PublicBank> {
        let lp = &ctx.params.levees;
        let map = ctx.map;
        let fp = &ctx.land.floodplain;
        let mut site = BankSite {
            reach,
            a,
            b,
            height_m: 1.0,
            crest_width_m: lp.crest_width_m,
            side_run: lp.side_run,
            compacted_per_bank: lp.compacted_per_bank,
            envelope: RectCm {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
            pit: RectCm {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
            labour_h: 0.0,
            work: None,
        };
        // Its height: the crest over the lowest ground it crosses.
        let base = segment_of(&site, placeholder, placeholder)
            .barriers_at(map, &fp.cell_hand, 0.0)
            .iter()
            .map(|e| e.crest_hand_m)
            .fold(f64::INFINITY, f64::min);
        if !base.is_finite() {
            return None;
        }
        let height = (crest - base).max(lp.min_height_m);
        if height > lp.max_height_m {
            return None;
        }
        site.height_m = height;
        let seg = segment_of(&site, placeholder, placeholder);
        if !seg.valid(map) {
            return None;
        }
        site.envelope = seg.envelope();
        if !crate::build::plot_clear(ctx.land, map, ctx.nav, &site.envelope) {
            return None;
        }
        // Its borrow pit, dug as deep as people dig one, beside its foot.
        let side = (seg.borrow_m3() / lp.pit_depth_m.max(1e-6)).sqrt().max(2.0);
        let reach_m = site.crest_width_m / 2.0 + site.side_run * height;
        let off = reach_m + PIT_GAP_M + side / 2.0;
        site.pit = [-1.0, 1.0]
            .into_iter()
            .map(|sign| {
                square(
                    [mid[0] + sign * dir[0] * off, mid[1] + sign * dir[1] * off],
                    side,
                )
            })
            .find(|pit| {
                !pit.near(&site.envelope, 0)
                    && crate::build::plot_clear(ctx.land, map, ctx.nav, pit)
            })?;
        // What it would keep dry: the places the water, as high as they remember it, would
        // reach only over its crest.
        let mut barriers = standing.to_vec();
        barriers.extend(seg.barriers_at(map, &fp.cell_hand, height));
        let entry = fp.entry_stages(map, reach, &barriers, changes);
        let mut saves: Vec<(PermanentId, f64)> = Vec::new();
        for p in group {
            if entry
                .get(p.cell as usize)
                .is_some_and(|&e| e > p.stage + 1e-6)
            {
                match saves.iter_mut().find(|s| s.0 == p.household) {
                    Some(s) => s.1 += p.year_h,
                    None => saves.push((p.household, p.year_h)),
                }
            }
        }
        if saves.is_empty() {
            return None;
        }
        saves.sort_unstable_by_key(|s| s.0);
        let pit = site.pit.centre_m();
        let haul = (f64::from(pit.0) - mid[0]).hypot(f64::from(pit.1) - mid[1]);
        site.labour_h = seg.borrow_m3() * lp.hours_per_m3(ctx.params.digging.h_per_m3, haul, speed);
        Some(PublicBank {
            site,
            life: lp.life_years,
            saves,
            households,
        })
    }

    /// Polity `pi` begins the bank its law `law` names (M6b slice BG), unless the ground there
    /// is no longer free: its earthwork and borrow pit are laid out, with no earth in them yet.
    pub(super) fn begin_public_bank(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        let polity = self.polities[pi].id;
        let Some(site) = self.polities[pi].bank_site_of(law) else {
            return;
        };
        if site.work.is_some()
            || !crate::build::plot_clear(ctx.land, ctx.map, ctx.nav, &site.envelope)
            || !crate::build::plot_clear(ctx.land, ctx.map, ctx.nav, &site.pit)
        {
            return;
        }
        let (work, borrow) = (ctx.ids.allocate(), ctx.ids.allocate());
        let seg = segment_of(&site, work, borrow);
        if !seg.valid(ctx.map) {
            return;
        }
        let make = |id, kind, rect| Earthwork {
            id,
            kind,
            rect,
            level_cm: 0,
            side_run_cm: (site.side_run * 100.0).round() as u16,
            plot: None,
            deposit: None,
            heap: None,
            household: polity,
            cut_m3: 0.0,
            done: 0.0,
            version: earth::EARTH_VERSION,
            begun: ctx.now,
        };
        ctx.land
            .earthworks
            .push(make(work, EarthKind::Bank, site.envelope));
        ctx.land
            .earthworks
            .push(make(borrow, EarthKind::Pit, site.pit));
        ctx.land.banks.push(seg);
        if let Some(s) = self.polities[pi].banks.iter_mut().find(|s| s.0 == law) {
            s.1.work = Some(work);
        }
    }

    /// Polity `pi` begins the bank its law `law` names, as its law passed would: for tests.
    #[doc(hidden)]
    pub fn begin_public_bank_for_tests(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        self.begin_public_bank(ctx, pi, law);
    }

    /// The bank person `who` may work on now, as their decision sees it: for tests.
    #[doc(hidden)]
    pub fn levee_facts_for_tests(&self, ctx: &Ctx, who: PermanentId) -> Option<LeveeFacts> {
        let x = self.household(self.person(who)?.household)?;
        let key = x.settlement.unwrap_or(x.id);
        self.levee_facts(ctx, x, who, key, false)
    }

    /// `hours` of a capable adult's work by `who` on bank `bank`: for tests.
    #[doc(hidden)]
    pub fn levee_work_by_for_tests(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        bank: PermanentId,
        hours: f64,
    ) {
        let (Some(&h), Some(household)) =
            (self.index.get(&who), self.person(who).map(|p| p.household))
        else {
            return;
        };
        self.levee_work(ctx, (h, who), household, bank, hours);
    }

    /// Where settlement `settlement`'s people could build a flood bank together now, and what it
    /// would save each household a year, hours: for tests.
    #[doc(hidden)]
    pub fn public_bank_for_tests(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
    ) -> Option<(BankSite, Vec<(PermanentId, f64)>)> {
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        self.public_bank(ctx, settlement, &households)
            .map(|p| (p.site, p.saves))
    }

    /// The bank household `x`'s people may work on, if any (M6b slice BG): one their polity is
    /// building under a law in force `person` knows of; where its work is done, the walk there,
    /// the work left, what each hour of it is worth to the household (the hours a year it
    /// reckoned the bank would save it, for each hour of the bank's work), and, while the
    /// household has not given the share asked of it, what keeping to the gathering's word is
    /// worth to `person`, as for a crossing built together. Full when as many work on it now as
    /// its length gives room for (research 11-12 §1.2: a narrow front holds few).
    pub(super) fn levee_facts(
        &self,
        ctx: &Ctx,
        x: &Household,
        person: PermanentId,
        field_key: PermanentId,
        dark: bool,
    ) -> Option<LeveeFacts> {
        if dark || ctx.land.banks.is_empty() {
            return None;
        }
        let pi = self.polity_of(x.settlement?)?;
        let polity = &self.polities[pi];
        let (law, site, work) = polity.banks.iter().find_map(|(l, s)| {
            let w = s.work?;
            let law = polity.laws.iter().find(|k| {
                k.id == *l
                    && k.status == LawStatus::InForce
                    && k.kind == PolicyKind::BuildBank
                    && k.knows(person)
            })?;
            let e = ctx.land.earthworks.iter().find(|e| e.id == w)?;
            (e.done < 1.0).then_some((law, *s, e))
        })?;
        let households = self
            .households
            .iter()
            .filter(|(_, y)| y.settlement == x.settlement && !y.members.is_empty())
            .count()
            .max(1);
        let share = site.labour_h / households as f64;
        let mut duty = 0.0;
        if f64::from(polity.bank_given_by(law.id, x.id)) + 1e-6 < share {
            let stance = law
                .stances
                .iter()
                .find(|r| r.person == person)
                .map_or(0.0, |r| r.stance.sign());
            let regard =
                self.ties
                    .regard(person, law.sponsor, ctx.now.day_index(), &ctx.params.ties);
            let norm = self.norm_points(ctx, person);
            duty = crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity)
                .max(0.0);
        }
        let at = (
            ((site.a[0] + site.b[0]) / 2.0) as f32,
            ((site.a[1] + site.b[1]) / 2.0) as f32,
        );
        // The walk there over the ground as the household walks it from home, as to a
        // crossing's bank.
        let walk_s = self
            .homes
            .get(&field_key)?
            .reach
            .seconds_to(cell_of(ctx.map, at))?;
        let room = (site.length_m() / ctx.params.levees.front_m.max(1e-6))
            .floor()
            .max(1.0) as usize;
        let at_work = self
            .people
            .iter()
            .filter(|(_, q)| q.act.target == Target::Bank(work.id))
            .count();
        Some(LeveeFacts {
            bank: work.id,
            at,
            walk_min: f64::from(walk_s) / 60.0,
            left_h: site.labour_h * f64::from(1.0 - work.done).max(0.0),
            saves_per_hour: f64::from(polity.bank_saves_of(law.id, x.id)) / site.labour_h.max(1e-6),
            duty,
            full: at_work >= room,
        })
    }

    /// `hours` of a capable adult's work by `who` (handle and id) of `household` on bank `bank`
    /// (M6b slice BG): as much earth as those hours dig from its borrow pit, carry the straight way
    /// from the pit, which lies beside it, to the bank's middle a basket at a time at their own walking pace, and spread
    /// and tamp, each charged apart (research 11-12 §1.2), is placed; the hours that placed it
    /// are the household's given. Earth moves only when the work is done: none is left half-way.
    /// Once all its earth is in, the bank is finished.
    pub(super) fn levee_work(
        &mut self,
        ctx: &mut Ctx,
        (h, who): (Handle<Person>, PermanentId),
        household: PermanentId,
        bank: PermanentId,
        hours: f64,
    ) {
        let Some(pi) = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return;
        };
        let Some((law, site)) = self.polities[pi]
            .banks
            .iter()
            .find(|(_, s)| s.work == Some(bank))
            .copied()
        else {
            return;
        };
        let Some(seg) = ctx.land.banks.iter().find(|b| b.work == bank).copied() else {
            return;
        };
        let age = self.people.get(h).map_or(30.0, |p| p.age_years(ctx.now));
        let speed = interpolate(&ctx.params.walk_speed_by_age, age).max(0.05) * 60.0;
        let pit = site.pit.centre_m();
        let mid = [(site.a[0] + site.b[0]) / 2.0, (site.a[1] + site.b[1]) / 2.0];
        let haul = (f64::from(pit.0) - mid[0]).hypot(f64::from(pit.1) - mid[1]);
        let per_m3 = ctx
            .params
            .levees
            .hours_per_m3(ctx.params.digging.h_per_m3, haul, speed);
        let Ok(dug) = banks::place_from_borrow(
            &seg,
            &mut ctx.land.earthworks,
            ctx.map,
            &mut ctx.land.ground,
            hours.max(0.0) / per_m3.max(1e-9),
        ) else {
            return;
        };
        if dug <= 0.0 {
            return;
        }
        self.polities[pi].give_to_bank(law, household, (dug * per_m3) as f32);
        let done = ctx
            .land
            .earthworks
            .iter()
            .find(|e| e.id == bank)
            .map_or(0.0, |e| e.done);
        if done < 1.0 {
            return;
        }
        let settlement = self.polities[pi].settlement;
        let place = ctx
            .land
            .settlements
            .iter()
            .find(|t| t.id == settlement)
            .map_or_else(|| "the village".to_owned(), |t| t.name.clone());
        let sentence = format!(
            "{place}'s people finished the bank against floodwater their gathering agreed to \
             build, {:.0} m long and {:.1} m high, {} tamping the last of its earth.",
            site.length_m(),
            site.height_m,
            self.name_of(who)
        );
        self.chronicle_push(
            ctx.now,
            ChronicleKind::Flood,
            vec![who],
            Some(settlement),
            Some((mid[0] as f32, mid[1] as f32)),
            f64::from(crate::history::FloodStep::BankBuilt as u8),
            sentence,
        );
    }
}
