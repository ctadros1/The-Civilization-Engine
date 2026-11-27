//! Bounded hiring, the first candidate (owner's decisions of 2026-11-26; brief
//! `docs/briefs/bounded-hire.md`; research 08-10 §5.3, §5.6 and 08-04, "Search and matching"): a
//! household seeking field work weighs the offers of a bounded number of distinct farm employers
//! at a decision, not every one within reach. Its attention budget (`firm.hire_search`, 8) is
//! spent first on contacts (its people's close kin's households and those holding anyone its
//! people hold a tie with, in any settlement), at most `firm.hire_contacts` (4) of them, then on
//! local exploration of the employers whose fields or home lie within reach (the seen ground).
//! Both are provisional design settings (`NUDGE:`), not historical constants.
//!
//! Who is weighed is a fresh draw keyed on the world's seed, this purpose, the seeking household
//! and the day (`Rng64::from_key`), without replacement, over lists read from saved state alone
//! (ties, kin, fields, households, the walking grid), so a save lives on as if never saved
//! (ADR-0011 §1, §3) and no employer is weighed twice in a search. Nothing promises when a given
//! employer is first weighed: each search reaches it by chance.
//!
//! Reachability is checked before a slot is spent: an employer enters the seen ground only if one
//! of its fields' centres or its home lies in the travel field it is searched from (another
//! settlement's within the work's walk), so unreachable fields take no slot. An offer whose place
//! proves out of reach once worked out is passed over without spending one; an employer with no
//! work, or unable to pay, spends one. At most twice the budget of offers is worked out at a
//! decision, retries included.
//!
//! The seen ground is kept per travel field ([`HirePool`]): derived, never saved, rebuilt when
//! any input it reads changes ([`PoolStamp`]), and in debug builds checked at every read against
//! one built afresh. Workshop hiring, wages and payment are unchanged.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use civ_core::{FastMap, PermanentId, Rng64};
use civ_world::nav::TravelField;

use super::reapers::ReapOffer;
use super::{Ctx, Population, cell_centre, cell_of};
use crate::params::Behavior;
use crate::person::Household;

/// The purpose of the keyed draw of whom a seeker of field work weighs (ADR-0011 §3).
pub const PURPOSE_HIRE: u64 = 0x6869_7265_7365_656b; // "hireseek"

/// What a kept [`HirePool`] was built on: rebuilt when any of it differs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PoolStamp {
    /// The travel field's origin cell and its revision (`HomeField`'s `cell` and `rev`: the
    /// land's route revision with the walking grid's travel revision, which crossings, worn
    /// ground, a flood's depths and a closed ford move).
    cell: usize,
    rev: u64,
    /// The settlement whose households are the seeker's own.
    settlement: PermanentId,
    /// How many fields there are (they are only appended in play, and their ground is fixed).
    fields: usize,
    /// Changes of who works a field ([`Population::fields_moved`]).
    fields_moved: u64,
    /// Changes of households' homes, settlements and existence (`homes_moved`).
    homes_moved: u64,
    /// The day.
    day: i64,
}

/// The employers a seeker on one travel field may explore: every household with a field whose
/// centre, or a home, lies in the field (another settlement's within the work's walk), by id,
/// with whether it is of another settlement. Derived, never saved.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct HirePool {
    pub employers: Vec<(PermanentId, bool)>,
    /// Reachability lookups the build took (report only).
    pub lookups: u64,
}

/// The kept pools, by the key of the travel field they were built for.
pub(super) type HirePools = std::sync::Mutex<FastMap<PermanentId, (PoolStamp, Arc<HirePool>)>>;

/// Counts of the search's work, for the record (the brief's §4.2b and §6): they change nothing
/// and are never saved.
#[derive(Debug, Default)]
pub struct HireCounts {
    decisions: AtomicU64,
    offers: AtomicU64,
    slots: AtomicU64,
    passed_over: AtomicU64,
    capped: AtomicU64,
    most_offers: AtomicU64,
    contacts_read: AtomicU64,
    contacts_eligible: AtomicU64,
    contacts_weighed: AtomicU64,
    offer_lookups: AtomicU64,
    pool_builds: AtomicU64,
    pool_lookups: AtomicU64,
    pool_reads: AtomicU64,
    pool_size: AtomicU64,
}

/// A copy of [`HireCounts`] as plain numbers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HireTally {
    /// Searches of field work made (one a household view's paid work).
    pub decisions: u64,
    /// Offers worked out (`reap_offer_with`), retries included.
    pub offers: u64,
    /// Slots of the budget spent: employers weighed, with work or none.
    pub slots: u64,
    /// Offers passed over without a slot, their place out of reach.
    pub passed_over: u64,
    /// Searches that stopped at twice the budget.
    pub capped: u64,
    /// The most offers any one search worked out.
    pub most_offers: u64,
    /// Kin and tie entries read for contacts.
    pub contacts_read: u64,
    /// Contacts among them that were reachable employers.
    pub contacts_eligible: u64,
    /// Contacts weighed.
    pub contacts_weighed: u64,
    /// Reachability lookups at searches (one an offer worked out).
    pub offer_lookups: u64,
    /// Pools built, and the reachability lookups they took.
    pub pool_builds: u64,
    pub pool_lookups: u64,
    /// Pools read as kept.
    pub pool_reads: u64,
    /// Employers summed over the pools searched (over `decisions`, the mean pool).
    pub pool_size: u64,
}

impl HireCounts {
    fn add(c: &AtomicU64, n: u64) {
        c.fetch_add(n, Ordering::Relaxed);
    }

    /// The counts so far.
    pub fn tally(&self) -> HireTally {
        let r = |c: &AtomicU64| c.load(Ordering::Relaxed);
        HireTally {
            decisions: r(&self.decisions),
            offers: r(&self.offers),
            slots: r(&self.slots),
            passed_over: r(&self.passed_over),
            capped: r(&self.capped),
            most_offers: r(&self.most_offers),
            contacts_read: r(&self.contacts_read),
            contacts_eligible: r(&self.contacts_eligible),
            contacts_weighed: r(&self.contacts_weighed),
            offer_lookups: r(&self.offer_lookups),
            pool_builds: r(&self.pool_builds),
            pool_lookups: r(&self.pool_lookups),
            pool_reads: r(&self.pool_reads),
            pool_size: r(&self.pool_size),
        }
    }
}

/// A draw without replacement from `0..n`, one at a time (a Fisher-Yates shuffle kept sparse:
/// only the places it has swapped are held, so a draw of `k` costs about `k`, not `n`).
struct Draw {
    n: usize,
    i: usize,
    moved: Vec<(usize, usize)>,
}

impl Draw {
    fn new(n: usize) -> Draw {
        Draw {
            n,
            i: 0,
            moved: Vec::new(),
        }
    }

    fn at(&self, k: usize) -> usize {
        self.moved
            .iter()
            .rev()
            .find(|&&(at, _)| at == k)
            .map_or(k, |&(_, v)| v)
    }

    fn next(&mut self, rng: &mut Rng64) -> Option<usize> {
        if self.i >= self.n {
            return None;
        }
        let j = self.i + rng.below((self.n - self.i) as u64) as usize;
        let (vj, vi) = (self.at(j), self.at(self.i));
        self.moved.push((j, vi));
        self.i += 1;
        Some(vj)
    }
}

/// An offer found in a search: the employer, its offer, the walk to its place, minutes.
pub(super) type Weighed = (PermanentId, ReapOffer, f64);

/// What weighing an employer came to in a search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// No work it would hire, or none it can pay for: a slot spent.
    NoWork,
    /// Work within reach: a slot spent, the offer weighed with the rest.
    Found,
    /// Work whose place proved out of reach: passed over, no slot spent.
    PassedOver,
}

/// A search's result: the offers found, every employer weighed in order with what it came to,
/// and how many of the first were contacts.
pub(super) struct Search {
    pub found: Vec<Weighed>,
    pub weighed: Vec<(PermanentId, Outcome)>,
    pub from_contacts: usize,
}

impl Population {
    /// The employers a seeker of settlement `settlement` searching from travel field `key`
    /// (origin cell `cell`, field `reach`) may explore, kept while what it reads stands
    /// ([`PoolStamp`]), else built afresh; in debug builds every kept read is checked against one
    /// built afresh.
    pub(super) fn hire_pool(
        &self,
        ctx: &Ctx,
        key: PermanentId,
        settlement: PermanentId,
        (cell, rev, reach): (usize, u64, &TravelField),
        (walk_min, walk_m): (f64, f64),
    ) -> Arc<HirePool> {
        let stamp = PoolStamp {
            cell,
            rev,
            settlement,
            fields: ctx.land.fields.len(),
            fields_moved: self.field_moves,
            homes_moved: self.homes_moved,
            day: ctx.now.day_index(),
        };
        let origin = cell_centre(ctx.map, cell);
        let kept = {
            // A poisoned lock still holds only pools, each as true as its stamp says.
            let pools = self
                .hire_pools
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            pools
                .get(&key)
                .filter(|(at, _)| *at == stamp)
                .map(|(_, p)| Arc::clone(p))
        };
        if let Some(pool) = kept {
            debug_assert_eq!(
                pool.employers,
                self.build_hire_pool(ctx, settlement, reach, origin, (walk_min, walk_m))
                    .employers,
                "what a kept pool of employers reads changed without its stamp"
            );
            HireCounts::add(&self.hire_counts.pool_reads, 1);
            return pool;
        }
        let pool =
            Arc::new(self.build_hire_pool(ctx, settlement, reach, origin, (walk_min, walk_m)));
        HireCounts::add(&self.hire_counts.pool_builds, 1);
        HireCounts::add(&self.hire_counts.pool_lookups, pool.lookups);
        let mut pools = self
            .hire_pools
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Pools of other days are let go, so a household's own cut-off field leaves none behind.
        pools.retain(|_, (at, _)| at.day == stamp.day);
        pools.insert(key, (stamp, Arc::clone(&pool)));
        pool
    }

    /// [`Self::hire_pool`] built afresh: the households of `settlement` with a field or home in
    /// `reach`, and those of other settlements with a field whose centre lies within `walk_m` of
    /// `origin` in a straight line (the walk at the fastest pace walking allows) and a field or
    /// home within `walk_min` minutes' walk.
    fn build_hire_pool(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        reach: &TravelField,
        origin: (f32, f32),
        (walk_min, walk_m): (f64, f64),
    ) -> HirePool {
        let land: &civ_land::Land = ctx.land;
        let mut lookups = 0u64;
        let mut walk = |at: (f32, f32)| {
            lookups += 1;
            reach
                .seconds_to(cell_of(ctx.map, at))
                .map(|s| f64::from(s) / 60.0)
        };
        let by_household: Option<FastMap<PermanentId, Vec<u32>>> = (!self.fields_all_indexed(land))
            .then(|| {
                let mut by: FastMap<PermanentId, Vec<u32>> = FastMap::default();
                for (i, f) in land.fields.iter().enumerate() {
                    by.entry(f.household).or_default().push(i as u32);
                }
                by
            });
        let fields_of = |e: PermanentId| -> &[u32] {
            match &by_household {
                Some(by) => by.get(&e).map_or(&[][..], Vec::as_slice),
                None => self.indexed_fields(land, e),
            }
        };
        let mut employers: Vec<(PermanentId, bool)> = Vec::new();
        for x in self.households_in(settlement) {
            let fields = fields_of(x.id);
            if fields.is_empty() {
                continue;
            }
            let reached = fields
                .iter()
                .any(|&fi| walk(land.fields[fi as usize].rect.centre_m()).is_some())
                || walk(x.home).is_some();
            if reached {
                employers.push((x.id, false));
            }
        }
        let mut near: Vec<PermanentId> = self
            .fields_near(land, origin, walk_m)
            .into_iter()
            .map(|i| land.fields[i].household)
            .filter(|&e| {
                self.household(e)
                    .is_some_and(|x| x.settlement.is_some() && x.settlement != Some(settlement))
            })
            .collect();
        near.sort_unstable();
        near.dedup();
        for e in near {
            let within = |w: Option<f64>| w.is_some_and(|w| w <= walk_min);
            let reached = fields_of(e)
                .iter()
                .any(|&fi| within(walk(land.fields[fi as usize].rect.centre_m())))
                || self.household(e).is_some_and(|x| within(walk(x.home)));
            if reached {
                employers.push((e, true));
            }
        }
        employers.sort_unstable();
        HirePool { employers, lookups }
    }

    /// The households of household `hh`'s contacts: its people's close kin's, and those holding
    /// anyone its people hold a tie with, in any settlement, by id, its own left out; with the
    /// entries read.
    fn hire_contacts(&self, hh: &Household) -> (Vec<PermanentId>, u64) {
        let mut out = self.kin_households(hh);
        let mut read = out.len() as u64;
        for &m in &hh.members {
            for t in self.ties.of(m) {
                read += 1;
                if let Some(p) = self.person(t.to)
                    && p.household != hh.id
                {
                    out.push(p.household);
                }
            }
        }
        out.sort_unstable();
        out.dedup();
        (out, read)
    }

    /// The employers of `pool` among household `hh`'s contacts ([`Self::hire_contacts`]), by
    /// id, with the entries read: each contact marked where the pool holds it, not all gathered,
    /// sorted and then looked up (an exact cut, ADR-0028 §1: the pool is sorted by id, so its
    /// order is theirs).
    fn contacts_in_pool(&self, hh: &Household, pool: &HirePool) -> (Vec<(PermanentId, bool)>, u64) {
        let find = |e: PermanentId| pool.employers.binary_search_by_key(&e, |&(id, _)| id).ok();
        let mut marked = vec![false; pool.employers.len()];
        let kin = self.kin_households(hh);
        let mut read = kin.len() as u64;
        for i in kin.into_iter().filter_map(find) {
            marked[i] = true;
        }
        for &m in &hh.members {
            for t in self.ties.of(m) {
                read += 1;
                if let Some(p) = self.person(t.to)
                    && p.household != hh.id
                    && let Some(i) = find(p.household)
                {
                    marked[i] = true;
                }
            }
        }
        let out: Vec<(PermanentId, bool)> = marked
            .iter()
            .zip(&pool.employers)
            .filter(|&(&m, _)| m)
            .map(|(_, &e)| e)
            .collect();
        debug_assert!({
            let (all, all_read) = self.hire_contacts(hh);
            let slow: Vec<(PermanentId, bool)> = all
                .into_iter()
                .filter_map(|e| find(e).map(|i| pool.employers[i]))
                .collect();
            all_read == read && slow == out
        });
        (out, read)
    }

    /// The farm offers household `hh` of `settlement` weighs at a decision, searching from
    /// travel field `field_key` (see the module's account), its draw keyed on day `key_day` (the
    /// day's own, but for tests): `None` without a travel field.
    pub(super) fn field_search(
        &self,
        ctx: &Ctx,
        hh: &Household,
        settlement: PermanentId,
        field_key: PermanentId,
        key_day: i64,
    ) -> Option<Search> {
        let home = self.homes.get(&field_key)?;
        let reach = &home.reach;
        let walk_min = ctx
            .catalog
            .activities
            .iter()
            .filter(|a| a.behavior == Behavior::Hire)
            .map(|a| f64::from(a.max_walk_minutes))
            .fold(0.0, f64::max);
        let walk_m = Self::hire_scan_m(ctx, walk_min);
        let pool = self.hire_pool(
            ctx,
            field_key,
            settlement,
            (home.cell, home.rev, reach),
            (walk_min, walk_m),
        );
        // Each employer's fields from the index of who works what, while it holds every field,
        // else from one pass over them; and the hands already out for hire gathered once, when
        // first wanted (§8 24, seventh: an exact cut, ADR-0028 §1).
        let by_household: Option<FastMap<PermanentId, Vec<u32>>> =
            (!self.fields_all_indexed(ctx.land)).then(|| {
                let mut by: FastMap<PermanentId, Vec<u32>> = FastMap::default();
                for (i, f) in ctx.land.fields.iter().enumerate() {
                    by.entry(f.household).or_default().push(i as u32);
                }
                by
            });
        let hired = std::cell::OnceCell::new();
        let set_out = |ids: &[PermanentId]| {
            Self::hours_on(hired.get_or_init(|| self.hired_set_out(ctx)), ids)
        };
        let offer = |e: PermanentId| {
            let fields = match &by_household {
                Some(by) => by.get(&e).map_or(&[][..], Vec::as_slice),
                None => self.indexed_fields(ctx.land, e),
            };
            self.reap_offer_with(ctx, e, fields, &set_out)
        };
        Some(self.search_field_work(ctx, hh, &pool, reach, walk_min, key_day, &offer))
    }

    /// The farm offers household `hh` weighs at a decision (see the module's account): at most
    /// `firm.hire_search` distinct employers spend a slot, contacts first (at most
    /// `firm.hire_contacts`), then the seen ground of `pool`, each drawn without replacement by a
    /// draw keyed on the seed, the household and day `key_day`; offers whose place `reach` does
    /// not reach (another settlement's beyond `walk_min`) are passed over without a slot, and no
    /// more than twice the budget of offers is worked out. `offer` works out employer `e`'s offer.
    #[allow(clippy::too_many_arguments)]
    fn search_field_work(
        &self,
        ctx: &Ctx,
        hh: &Household,
        pool: &HirePool,
        reach: &TravelField,
        walk_min: f64,
        key_day: i64,
        offer: &dyn Fn(PermanentId) -> Option<ReapOffer>,
    ) -> Search {
        let firm = &ctx.params.firm;
        let budget = firm.hire_search as usize;
        let contact_slots = (firm.hire_contacts as usize).min(budget);
        let cap = 2 * budget;
        let day = key_day as u64;
        let (contacts, read) = self.contacts_in_pool(hh, pool);
        let counts = &self.hire_counts;
        HireCounts::add(&counts.decisions, 1);
        HireCounts::add(&counts.contacts_read, read);
        HireCounts::add(&counts.contacts_eligible, contacts.len() as u64);
        HireCounts::add(&counts.pool_size, pool.employers.len() as u64);
        let (mut slots, mut passed, mut lookups) = (0usize, 0u64, 0u64);
        let mut weighed: Vec<(PermanentId, Outcome)> = Vec::new();
        let mut found: Vec<Weighed> = Vec::new();
        let mut weigh = |(e, other): (PermanentId, bool),
                         slots: &mut usize,
                         weighed: &mut Vec<(PermanentId, Outcome)>| {
            let Some(o) = offer(e) else {
                // No work, or none it can pay for: weighed, a slot spent.
                *slots += 1;
                weighed.push((e, Outcome::NoWork));
                return;
            };
            lookups += 1;
            let walk = reach
                .seconds_to(cell_of(ctx.map, o.at))
                .map(|s| f64::from(s) / 60.0);
            match walk {
                // Another settlement's only within the work's walk, lest a far one hide a near
                // one.
                Some(w) if !other || w <= walk_min => {
                    *slots += 1;
                    weighed.push((e, Outcome::Found));
                    found.push((e, o, w));
                }
                _ => {
                    passed += 1;
                    weighed.push((e, Outcome::PassedOver));
                }
            }
        };
        let mut rng = Rng64::from_key(&[ctx.seed, PURPOSE_HIRE, hh.id.get(), day, 0]);
        let mut draw = Draw::new(contacts.len());
        while slots < contact_slots && weighed.len() < cap {
            let Some(i) = draw.next(&mut rng) else {
                break;
            };
            weigh(contacts[i], &mut slots, &mut weighed);
        }
        let from_contacts = weighed.len();
        let mut rng = Rng64::from_key(&[ctx.seed, PURPOSE_HIRE, hh.id.get(), day, 1]);
        let mut draw = Draw::new(pool.employers.len());
        while slots < budget && weighed.len() < cap {
            let Some(i) = draw.next(&mut rng) else {
                break;
            };
            let e = pool.employers[i];
            if e.0 == hh.id || weighed.iter().any(|&(w, _)| w == e.0) {
                continue;
            }
            weigh(e, &mut slots, &mut weighed);
        }
        let offers = weighed.len();
        HireCounts::add(&counts.contacts_weighed, from_contacts as u64);
        HireCounts::add(&counts.offers, offers as u64);
        HireCounts::add(&counts.slots, slots as u64);
        HireCounts::add(&counts.passed_over, passed);
        HireCounts::add(&counts.offer_lookups, lookups);
        if offers >= cap {
            HireCounts::add(&counts.capped, 1);
        }
        counts
            .most_offers
            .fetch_max(offers as u64, Ordering::Relaxed);
        Search {
            found,
            weighed,
            from_contacts,
        }
    }

    /// Brings up to date what a decision of household `household` reads before it weighs paid
    /// work: its travel field, who works what, who is out for hire and its people's kin. Gives
    /// the household, its settlement and its travel field's key.
    fn ready_for_search(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Option<(Household, PermanentId, PermanentId)> {
        let hh = self.household(household)?.clone();
        let settlement = hh.settlement?;
        let (key, _) = self.walking_field_of(ctx, (hh.id, hh.settlement, hh.home));
        self.index_fields(ctx.land);
        self.index_hired(ctx);
        self.refresh_kin();
        Some((hh, settlement, key))
    }

    /// The farm employers household `household` weighs at a decision now, in the order weighed,
    /// with what each came to, its draw keyed on day `key_day` in place of today's, and how many
    /// of the first were its contacts: for tests (a controlled draw).
    #[doc(hidden)]
    pub fn hire_weighed_for_tests(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        key_day: i64,
    ) -> Option<(Vec<(PermanentId, Outcome)>, usize)> {
        let (hh, settlement, key) = self.ready_for_search(ctx, household)?;
        self.field_search(ctx, &hh, settlement, key, key_day)
            .map(|s| (s.weighed, s.from_contacts))
    }

    /// The employers household `household`'s search may explore now (its travel field's kept
    /// pool), by id, with whether each is of another settlement: for tests.
    #[doc(hidden)]
    pub fn hire_pool_for_tests(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Vec<(PermanentId, bool)> {
        let Some((_, settlement, key)) = self.ready_for_search(ctx, household) else {
            return Vec::new();
        };
        let Some(home) = self.homes.get(&key) else {
            return Vec::new();
        };
        let walk_min = ctx
            .catalog
            .activities
            .iter()
            .filter(|a| a.behavior == Behavior::Hire)
            .map(|a| f64::from(a.max_walk_minutes))
            .fold(0.0, f64::max);
        let walk_m = Self::hire_scan_m(ctx, walk_min);
        self.hire_pool(
            ctx,
            key,
            settlement,
            (home.cell, home.rev, &home.reach),
            (walk_min, walk_m),
        )
        .employers
        .clone()
    }

    /// Household `household`'s contacts that are employers within reach of its search, by id:
    /// for tests.
    #[doc(hidden)]
    pub fn hire_contacts_for_tests(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Vec<PermanentId> {
        let pool = self.hire_pool_for_tests(ctx, household);
        let Some(hh) = self.household(household).cloned() else {
            return Vec::new();
        };
        self.hire_contacts(&hh)
            .0
            .into_iter()
            .filter(|e| pool.binary_search_by_key(e, |&(id, _)| id).is_ok())
            .collect()
    }

    /// Household `household`'s home moves to `at`, as a home built or a move makes it (the
    /// change counted, `homes_moved`): for tests.
    #[doc(hidden)]
    pub fn move_home_for_tests(&mut self, household: PermanentId, at: (f32, f32)) {
        if let Some(&hd) = self.hh_index.get(&household)
            && let Some(x) = self.households.get_mut(hd)
        {
            x.home = at;
            self.homes_moved += 1;
        }
    }

    /// The counts of the search for field work so far (report only, never saved).
    pub fn hire_tally(&self) -> HireTally {
        self.hire_counts.tally()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_draw_gives_every_index_once_in_a_keyed_order() {
        for n in [0usize, 1, 2, 7, 40] {
            let mut rng = Rng64::from_key(&[1, PURPOSE_HIRE, 9, n as u64]);
            let mut d = Draw::new(n);
            let mut got: Vec<usize> = std::iter::from_fn(|| d.next(&mut rng)).collect();
            let mut again = Draw::new(n);
            let mut rng2 = Rng64::from_key(&[1, PURPOSE_HIRE, 9, n as u64]);
            let same: Vec<usize> = std::iter::from_fn(|| again.next(&mut rng2)).collect();
            assert_eq!(got, same, "the same key draws the same order");
            got.sort_unstable();
            assert_eq!(got, (0..n).collect::<Vec<_>>(), "each index once");
        }
    }

    #[test]
    fn a_draw_of_a_few_from_many_is_near_uniform() {
        // Over many keys, the first three of 20 drawn: each index about 3/20 of the time.
        let mut seen = [0u32; 20];
        let keys = 20_000u64;
        for k in 0..keys {
            let mut rng = Rng64::from_key(&[7, PURPOSE_HIRE, k]);
            let mut d = Draw::new(20);
            for _ in 0..3 {
                seen[d.next(&mut rng).expect("drawn")] += 1;
            }
        }
        let expect = keys as f64 * 3.0 / 20.0;
        for (i, &s) in seen.iter().enumerate() {
            assert!(
                (f64::from(s) - expect).abs() < 0.06 * expect,
                "index {i} drawn {s} times of {expect}"
            );
        }
    }
}
