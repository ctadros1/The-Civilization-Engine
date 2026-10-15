//! Palisades a polity's people raise together round their hearth (M6c slice BK, step two;
//! ADR-0023 §6; research 11-11). A household that believes people of other settlements took
//! from its stores may propose that the polity raise a line round the hearth, taking in the
//! homes within a reach; each household weighs what it believes the line keeps out of its home
//! against its share of the work, spread over the posts' life, and the walk round to a gate.
//! Homes outside the line pay without its cover. Passed, the line is laid out and each household
//! is asked an equal share of its work; its people set its posts a stretch at a time, and each
//! stretch stands, and bars walking, once its work is done (11-11 §1.3: incomplete works give
//! partial benefit).

use super::*;
use crate::decide::EnclosureFacts;
use crate::enclose::{detour_h_a_year, ring_radius};
use crate::polity::{EnclosureSite, LawStatus, PolicyKind};
use civ_land::enclosures::{Enclosure, ring};

/// A palisade a polity's people could raise together round the hearth, and what each of the
/// settlement's households believes it would bring it.
#[derive(Clone, Debug)]
pub(crate) struct PublicEnclosure {
    /// What it would raise.
    pub(crate) site: EnclosureSite,
    /// The years a household reckons its posts stand.
    pub(crate) life: f64,
    /// For each household, by id: the food a year it believes the line would keep from people of
    /// other settlements, kcal, and the hours a year its people would walk round to a gate (both
    /// nothing for a home outside the line).
    pub(crate) keeps: Vec<(PermanentId, f64, f64)>,
    /// How many households its work would be asked of.
    pub(crate) households: usize,
}

impl PublicEnclosure {
    /// The share of its work asked of each household, hours.
    pub(crate) fn share_h(&self) -> f64 {
        f64::from(self.site.labour_h) / self.households.max(1) as f64
    }

    /// What household `h` believes it would keep a year, kcal, and walk round, hours.
    pub(crate) fn keeps_of(&self, h: PermanentId) -> (f64, f64) {
        self.keeps
            .binary_search_by_key(&h, |k| k.0)
            .map_or((0.0, 0.0), |i| (self.keeps[i].1, self.keeps[i].2))
    }
}

impl Population {
    /// The palisade settlement `settlement`'s people could raise together round their hearth
    /// now, and what each of households `households` believes it would bring it (M6c slice BK):
    /// a ring taking in the homes within `enclosures.reach_m` of the hearth, cut into sectors with
    /// `enclosures.gates` gates. A household whose home is inside believes it keeps out
    /// `enclosures.cover` of what people of other settlements took from it in the last year, as
    /// its members know, and walks round to a gate whenever its people cross the line. None when
    /// content has no policy to raise one, the settlement has a line already, no home is within
    /// reach, or nobody believes outsiders took anything.
    pub(crate) fn public_enclosure(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        households: &[PermanentId],
    ) -> Option<PublicEnclosure> {
        if !ctx
            .catalog
            .policies
            .iter()
            .any(|d| d.kind == PolicyKind::Enclose)
        {
            return None;
        }
        if ctx
            .land
            .enclosures
            .list
            .iter()
            .any(|e| e.settlement == settlement)
        {
            return None;
        }
        let ep = &ctx.params.enclosures;
        let centre = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)?
            .hearth_m;
        let homes: Vec<(&Household, f64)> = households
            .iter()
            .filter_map(|&h| self.household(h))
            .filter(|x| !x.members.is_empty())
            .map(|x| {
                let d = f64::from(x.home.0 - centre.0).hypot(f64::from(x.home.1 - centre.1));
                (x, d)
            })
            .collect();
        let distances: Vec<f64> = homes.iter().map(|h| h.1).collect();
        let radius = ring_radius(ep, &distances)?;
        let speed = interpolate(&ctx.params.walk_speed_by_age, 30.0).max(0.05);
        let detour = detour_h_a_year(ep, radius, ep.gates, speed);
        let mut keeps: Vec<(PermanentId, f64, f64)> = homes
            .iter()
            .map(|&(x, d)| {
                if d < radius {
                    let outside = self.takings_known(x.id, ctx.now, ctx.params).outside_kcal;
                    (x.id, ep.cover * outside, detour)
                } else {
                    (x.id, 0.0, 0.0)
                }
            })
            .collect();
        if !keeps.iter().any(|k| k.1 > 0.0) {
            return None;
        }
        keeps.sort_unstable_by_key(|k| k.0);
        let inside = homes.iter().filter(|h| h.1 < radius).count();
        Some(PublicEnclosure {
            site: EnclosureSite {
                centre,
                radius_m: radius as f32,
                gates: ep.gates.min(u32::from(u8::MAX)) as u8,
                inside: inside.min(usize::from(u16::MAX)) as u16,
                labour_h: (std::f64::consts::TAU * radius * ep.palisade_h_per_m) as f32,
                work: None,
            },
            life: ep.life_years,
            keeps,
            households: homes.len(),
        })
    }

    /// Polity `pi` begins the palisade law `law` names (M6c slice BK), unless the settlement has
    /// one already or its ring would not lie on the map: the line laid out round its centre in
    /// sectors, its first gate facing east, owned by the polity, no work yet done.
    pub(super) fn begin_public_enclosure(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        let Some(site) = self.polities[pi].enclosure_site_of(law) else {
            return;
        };
        let (polity, settlement) = (self.polities[pi].id, self.polities[pi].settlement);
        if site.work.is_some()
            || ctx
                .land
                .enclosures
                .list
                .iter()
                .any(|e| e.settlement == settlement)
        {
            return;
        }
        let ep = &ctx.params.enclosures;
        let sectors = ring(
            site.centre,
            site.radius_m,
            ep.sector_m as f32,
            usize::from(site.gates),
            0.0,
            ep.palisade_h_per_m as f32,
            ctx.map.cell_size_m,
            ctx.map.width,
            ctx.map.height,
        );
        if sectors.len() < 3 {
            return;
        }
        let id = ctx.ids.allocate();
        ctx.land.enclosures.list.push(Enclosure {
            id,
            owner: polity,
            law,
            settlement,
            centre: site.centre,
            radius_m: site.radius_m,
            sectors,
            work_h: 0.0,
            shares: Vec::new(),
            crew: (0, 0),
            begun: ctx.now,
            finished: None,
        });
        if let Some(s) = self.polities[pi].enclosures.iter_mut().find(|s| s.0 == law) {
            s.1.work = Some(id);
        }
    }

    /// Polity `pi` begins the palisade law `law` names, as its passing would: for tests.
    #[doc(hidden)]
    pub fn begin_public_enclosure_for_tests(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        self.begin_public_enclosure(ctx, pi, law);
    }

    /// The palisade household `x`'s people may work on, if any (M6c slice BK): one their polity
    /// is raising under a law in force `person` knows of, not yet whole; where its work is done
    /// (the middle of the first stretch still to stand that they can walk to), the walk there, the work left, what
    /// each hour of it is worth to the household (the hours a year it reckoned the line would
    /// keep it, for each hour of the line's work), and, while the household has not given the
    /// share asked of it, what keeping to the gathering's word is worth to `person`, as for a
    /// bank built together. Full when as many work on it now as the line still to stand gives
    /// room for.
    pub(super) fn enclosure_facts(
        &self,
        ctx: &Ctx,
        x: &Household,
        person: PermanentId,
        field_key: PermanentId,
        dark: bool,
    ) -> Option<EnclosureFacts> {
        if dark || ctx.land.enclosures.list.is_empty() {
            return None;
        }
        let settlement = x.settlement?;
        let pi = self.polity_of(settlement)?;
        let polity = &self.polities[pi];
        let e = ctx
            .land
            .enclosures
            .list
            .iter()
            .find(|e| e.settlement == settlement && e.owner == polity.id && !e.whole())?;
        let law = polity.laws.iter().find(|l| {
            l.id == e.law
                && l.status == LawStatus::InForce
                && l.kind == PolicyKind::Enclose
                && l.knows(person)
        })?;
        let labour = f64::from(e.labour_h());
        let left_h = (labour - f64::from(e.work_h)).max(0.0);
        // The first stretch still to stand that they can walk to (one across water is worked
        // from the nearest the work can be reached; its posts are set in order all the same).
        let reach = &self.homes.get(&field_key)?.reach;
        let (at, walk_s) = e.sectors.iter().skip(e.standing()).find_map(|s| {
            let at = ((s.a.0 + s.b.0) / 2.0, (s.a.1 + s.b.1) / 2.0);
            Some((at, reach.seconds_to(cell_of(ctx.map, at))?))
        })?;
        let households = self
            .households
            .iter()
            .filter(|(_, y)| y.settlement == Some(settlement) && !y.members.is_empty())
            .count()
            .max(1);
        let share = labour / households as f64;
        let mut duty = 0.0;
        if f64::from(e.worked_by(x.id)) + 1e-6 < share {
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
        let unbuilt_m: f64 = e
            .sectors
            .iter()
            .skip(e.standing())
            .map(|s| f64::from(s.length_m()))
            .sum();
        let room = (unbuilt_m / ctx.params.enclosures.front_m.max(1e-6))
            .floor()
            .max(1.0) as usize;
        let at_work = self
            .people
            .iter()
            .filter(|(_, q)| q.act.target == Target::Enclosure(e.id))
            .count();
        Some(EnclosureFacts {
            enclosure: e.id,
            at,
            walk_min: f64::from(walk_s) / 60.0,
            left_h,
            saves_per_hour: f64::from(polity.enclosure_saves_of(law.id, x.id)) / labour.max(1e-6),
            duty,
            full: at_work >= room,
        })
    }

    /// The palisade person `who` may work on, as their decision sees it: for tests.
    #[doc(hidden)]
    pub fn enclosure_facts_for_tests(&self, ctx: &Ctx, who: PermanentId) -> Option<EnclosureFacts> {
        let p = self.person(who)?;
        let x = self.household(p.household)?;
        let key = x
            .settlement
            .filter(|s| self.homes.contains_key(s))
            .unwrap_or(x.id);
        self.enclosure_facts(ctx, x, who, key, false)
    }

    /// `hours` of a capable adult's work by `who` of `household` on palisade `enclosure` (M6c
    /// slice BK): the work done on its line, and the household's given. Each stretch stands once
    /// the work its posts take is done, in order round the line, and bars walking from then;
    /// once the last stands the line is whole, and that is told.
    pub(super) fn enclosure_work(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        enclosure: PermanentId,
        hours: f64,
    ) {
        let Some(i) = ctx
            .land
            .enclosures
            .position(enclosure)
            .filter(|&i| !ctx.land.enclosures.list[i].whole())
        else {
            return;
        };
        let e = &mut ctx.land.enclosures.list[i];
        let before = e.standing();
        let left = (e.labour_h() - e.work_h).max(0.0);
        let done = (hours as f32).min(left);
        if done <= 0.0 {
            return;
        }
        e.work_h += done;
        e.add_share(household, done);
        let standing = e.standing();
        let whole = e.whole();
        if whole {
            e.finished = Some(ctx.now.day_index());
        }
        let (settlement, centre, length, gates) = (
            e.settlement,
            e.centre,
            e.sectors.iter().map(|s| s.length_m()).sum::<f32>(),
            e.sectors.iter().filter(|s| s.gate.is_some()).count(),
        );
        if standing != before {
            ctx.land.enclosures.changed();
        }
        if whole {
            let place = ctx
                .land
                .settlements
                .iter()
                .find(|s| s.id == settlement)
                .map_or_else(|| "the village".to_owned(), |s| s.name.clone());
            self.chronicle_push(
                ctx.now,
                ChronicleKind::Enclosure,
                vec![who],
                Some(settlement),
                Some(centre),
                f64::from(length),
                format!(
                    "{place}'s people set the last posts of the palisade round their hearth: \
                     {length:.0} m with {gates} gate{}.",
                    if gates == 1 { "" } else { "s" }
                ),
            );
        }
    }

    /// `hours` of a capable adult's work by `who`, for their household, on palisade `enclosure`:
    /// for tests.
    #[doc(hidden)]
    pub fn enclosure_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        enclosure: PermanentId,
        hours: f64,
    ) {
        let Some(household) = self.person(who).map(|p| p.household) else {
            return;
        };
        self.enclosure_work(ctx, who, household, enclosure, hours);
    }

    /// Where settlement `settlement`'s people could raise a palisade together now, and what each
    /// household believes it would keep a year and walk round: for tests.
    #[doc(hidden)]
    #[allow(clippy::type_complexity)]
    pub fn public_enclosure_for_tests(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
    ) -> Option<(EnclosureSite, Vec<(PermanentId, f64, f64)>)> {
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        self.public_enclosure(ctx, settlement, &households)
            .map(|p| (p.site, p.keeps))
    }
}
