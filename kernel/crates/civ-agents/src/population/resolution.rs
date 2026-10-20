//! A resolution per settlement (M9a slice DF, ADR-0032 §1, §3): which settlements are
//! household-resolved is a run setting, never saved. Their people are parked: an activity with no
//! steps standing at [`PARKED_STEP`], so nothing is scheduled for them, which a save keeps as it is. Parking and taking up
//! the day again happen at a midnight, and change nobody, no holding and no good.

use super::*;

/// The physical activity level of a household-resolved day, the same day's need [`Population`]'s
/// meals reckon with (a design prior until slice DG resolves the day's work).
const NEEDS_DAY_PAR: f64 = 1.6;

/// The step a parked activity stands at: past the end of no steps, which nothing else sets (an
/// activity restarted on load, or a newborn's or founder's first, has no steps and stands at 0).
pub const PARKED_STEP: u8 = u8::MAX;

/// What a switch of resolution must leave as it found it (ADR-0032 §3): who lives, in which
/// household, which households live where with whom, and who holds every field, building and
/// workshop. Goods are checked by the books ([`unaccounted`]).
#[derive(Clone, Debug, PartialEq)]
pub struct SwitchInvariants {
    /// Every living person and their household, by person.
    pub people: Vec<(PermanentId, PermanentId)>,
    /// Every household, its settlement and its members, by household.
    pub households: Vec<(PermanentId, Option<PermanentId>, Vec<PermanentId>)>,
    /// Every field, the household working it and its holder.
    pub fields: Vec<(PermanentId, PermanentId, Party)>,
    /// Every building and its household.
    pub buildings: Vec<(PermanentId, PermanentId)>,
    /// Every workshop and its owner.
    pub firms: Vec<(PermanentId, PermanentId)>,
}

impl Population {
    /// The settlements now household-resolved.
    pub fn household_resolved(&self) -> &BTreeSet<PermanentId> {
        &self.resolved
    }

    /// Sets the settlements household-resolved; the caller reconciles at a midnight
    /// ([`Self::reconcile_resolutions`]).
    pub fn set_household_resolved(&mut self, settlements: BTreeSet<PermanentId>) {
        self.resolved = settlements;
    }

    /// Whether person `h` lives in a household-resolved settlement.
    pub(super) fn resolved_person(&self, h: Handle<Person>) -> bool {
        if self.resolved.is_empty() {
            return false;
        }
        self.people
            .get(h)
            .and_then(|p| self.household(p.household))
            .and_then(|x| x.settlement)
            .is_some_and(|s| self.resolved.contains(&s))
    }

    /// Whether person `h` is parked: their activity has no steps and stands at
    /// [`PARKED_STEP`], so nothing is scheduled for them.
    fn parked(&self, h: Handle<Person>) -> bool {
        self.people
            .get(h)
            .is_some_and(|p| p.act.steps.is_empty() && p.act.step == PARKED_STEP)
    }

    /// Parks person `h` (ADR-0032 §3): what they carry is banked, they stand at home, and their
    /// activity is replaced by one of no steps, so any event of the old one does nothing. Parking
    /// someone already parked changes nothing.
    pub(super) fn park(&mut self, ctx: &mut Ctx, h: Handle<Person>) {
        if self.parked(h) {
            return;
        }
        let laden = self
            .people
            .get(h)
            .is_some_and(|p| p.carrying.kg > 0.0 || p.carrying.water_l > 0.0);
        if laden {
            self.deposit(ctx, h);
        }
        let Some(home) = self
            .people
            .get(h)
            .and_then(|p| self.household(p.household))
            .map(|x| x.home)
        else {
            return;
        };
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        settle(p, ctx.now, ctx.params);
        p.trip = None;
        p.pos = home;
        p.act = Activity {
            def: p.act.def,
            target: Target::None,
            steps: Vec::new(),
            step: PARKED_STEP,
            started: ctx.now,
            step_started: ctx.now,
            step_ends: ctx.now,
            version: p.act.version.wrapping_add(1),
        };
    }

    /// Brings everyone's execution in line with the resolutions in force (ADR-0032 §3): those
    /// of household-resolved settlements are parked ([`Self::park_resolved`]), then anyone parked
    /// elsewhere takes up their day ([`Self::resume_unresolved`]).
    pub fn reconcile_resolutions(&mut self, ctx: &mut Ctx) {
        self.park_resolved(ctx);
        self.resume_unresolved(ctx);
    }

    /// Parks everyone of a household-resolved settlement not yet parked, in person-table order.
    /// It changes nobody, no holding and no good the books do not account for.
    pub fn park_resolved(&mut self, ctx: &mut Ctx) {
        if self.resolved.is_empty() {
            return;
        }
        let handles: Vec<Handle<Person>> = self.people.iter().map(|(h, _)| h).collect();
        for h in handles {
            if self.resolved_person(h) {
                self.park(ctx, h);
            }
        }
    }

    /// Anyone parked outside a household-resolved settlement takes up their day with a first
    /// decision, as founders do, in person-table order.
    pub fn resume_unresolved(&mut self, ctx: &mut Ctx) {
        let handles: Vec<Handle<Person>> = self
            .people
            .iter()
            .filter(|(_, p)| p.act.steps.is_empty() && p.act.step == PARKED_STEP)
            .map(|(h, _)| h)
            .collect();
        for h in handles {
            if !self.resolved_person(h) {
                self.decide(ctx, h, 0);
            }
        }
    }

    /// The day of each household-resolved settlement, lived at the midnight that begins it. In
    /// this first form (slice DF) it is the needs alone: each parked member's needs are brought
    /// up to now at a day's mean activity, they eat what they need from their household's
    /// stores as a meal at home does, they wake rested, and the household's water is what a day
    /// uses. Slice DG resolves the day's work.
    pub fn live_household_days(&mut self, ctx: &mut Ctx) {
        if self.resolved.is_empty() {
            return;
        }
        let (now, params) = (ctx.now, ctx.params);
        let settlements: Vec<PermanentId> = self.resolved.iter().copied().collect();
        for s in settlements {
            let households: Vec<PermanentId> = self.households_in(s).map(|x| x.id).collect();
            for id in households {
                let Some(&hh) = self.hh_index.get(&id) else {
                    continue;
                };
                let Some(members) = self.households.get(hh).map(|x| x.members.clone()) else {
                    continue;
                };
                if let Some(x) = self.households.get_mut(hh) {
                    let litres_day = x.members.len().max(1) as f64 * x.water_use(&params.household);
                    x.settle_water(now, litres_day);
                    x.water_l = x.water_l.max(litres_day);
                }
                for m in members {
                    let Some(&h) = self.index.get(&m) else {
                        continue;
                    };
                    if !self.parked(h) {
                        continue;
                    }
                    if let Some(p) = self.people.get_mut(h) {
                        settle(p, now, params);
                        p.asleep = false;
                        p.sleep_pressure = 0.0;
                        p.burn_kcal_min = burn_rate(p, now, params, NEEDS_DAY_PAR);
                    }
                    self.eat(ctx, h, id);
                }
            }
        }
    }

    /// What a switch of resolution must leave unchanged (ADR-0032 §3).
    pub fn switch_invariants(&self, land: &Land) -> SwitchInvariants {
        let mut people: Vec<_> = self
            .people
            .iter()
            .map(|(_, p)| (p.id, p.household))
            .collect();
        people.sort_unstable();
        let mut households: Vec<_> = self
            .households
            .iter()
            .map(|(_, x)| {
                let mut m = x.members.clone();
                m.sort_unstable();
                (x.id, x.settlement, m)
            })
            .collect();
        households.sort_unstable_by_key(|x| x.0);
        SwitchInvariants {
            people,
            households,
            fields: land
                .fields
                .iter()
                .map(|f| (f.id, f.household, f.holder))
                .collect(),
            buildings: land.buildings.iter().map(|b| (b.id, b.household)).collect(),
            firms: self.firms.iter().map(|f| (f.id, f.owner)).collect(),
        }
    }

    /// Anyone whose execution does not match their settlement's resolution: someone of a
    /// household-resolved settlement who is not parked, or someone parked elsewhere. Empty after
    /// every midnight (ADR-0032 §3).
    pub fn resolution_problems(&self) -> Vec<String> {
        self.people
            .iter()
            .filter_map(|(h, p)| match (self.resolved_person(h), self.parked(h)) {
                (true, false) => Some(format!(
                    "{} lives in a household-resolved settlement but is not parked",
                    p.id.get()
                )),
                (false, true) => Some(format!(
                    "{} is parked outside a household-resolved settlement",
                    p.id.get()
                )),
                _ => None,
            })
            .collect()
    }
}
