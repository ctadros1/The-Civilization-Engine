//! Suspicion of a source of water, tallied by each person (M6a slice BA, step two; ADR-0021 §6).
//! Who draws where is what a person's own records say: their household's draws, the households it
//! saw drawing at the same source that day, and the usual source of the households of those they
//! know well enough. Who had sickness is the word of it they hold. Each adult who holds news of a
//! household's sickness, or a suspicion, tallies on first hearing of sickness and every
//! `review_days` after (on their household's day): a source the tally finds suspect is held, and
//! the first time it is, said as a claim with its counts, which word carries as it carries
//! sickness; one the tally no longer finds suspect is let go. Nothing here reads where anyone took
//! sickness or what a source holds.

use super::*;
use crate::uses::Place;
use crate::word::{Claim, ClaimKind, Counts};

impl Population {
    /// The day's tallies (see the module documentation).
    pub(super) fn suspicion_day(&mut self, ctx: &Ctx) {
        let sp = &ctx.params.suspicion;
        let day = ctx.now.day_index();
        let review = i64::from(sp.review_days.max(1));
        // Those who hold news of a household's sickness, and whether they first heard it since the
        // last tally; and those who hold a suspicion.
        let mut due: BTreeMap<PermanentId, bool> = BTreeMap::new();
        for h in &self.word.heard {
            if self
                .word
                .claim(h.claim)
                .is_some_and(|c| c.kind == ClaimKind::Sickness)
            {
                *due.entry(h.holder).or_default() |= h.first >= day - 1;
            }
        }
        for s in &self.word.suspicions {
            due.entry(s.holder).or_default();
        }
        if due.is_empty() {
            return;
        }
        let grown = ctx.params.family.independent_age;
        let mut found: Vec<(PermanentId, Vec<(Place, Counts)>)> = Vec::new();
        for (&person, &fresh) in &due {
            let Some(p) = self.person(person) else {
                continue;
            };
            let weekday = (p.household.get() as i64).rem_euclid(review) == day.rem_euclid(review);
            if !(fresh || weekday) || p.age_years(ctx.now) < grown {
                continue;
            }
            let draws = self.known_draws(ctx, person, p.household, day);
            let sick: BTreeSet<PermanentId> = self
                .word
                .heard_by(person)
                .iter()
                .filter_map(|h| self.word.claim(h.claim))
                .filter(|c| c.kind == ClaimKind::Sickness)
                .filter_map(|c| c.subject)
                .collect();
            found.push((person, crate::suspicion::tally(&draws, &sick, sp)));
        }
        for (person, suspect) in found {
            let held: Vec<Place> = self
                .word
                .suspicions_of(person)
                .iter()
                .map(|s| s.source)
                .collect();
            for &source in held
                .iter()
                .filter(|s| !suspect.iter().any(|(t, _)| t == *s))
            {
                self.word.unsuspect(person, source);
            }
            let settlement = self
                .person(person)
                .and_then(|p| self.household(p.household))
                .and_then(|x| x.settlement);
            for (source, counts) in suspect {
                if self.word.suspect(person, source, counts, day)
                    && let Some(settlement) = settlement
                {
                    let claim = self.word.make(Claim {
                        id: 0,
                        kind: ClaimKind::Suspicion,
                        settlement,
                        day,
                        subject: Some(person),
                        grievance: None,
                        suspected: Some((source, counts)),
                    });
                    self.word.hear(person, claim, day, None, Some(person));
                }
            }
        }
    }

    /// The day's tallies now, whatever else the day does: for tests.
    #[doc(hidden)]
    pub fn suspicion_day_for_tests(&mut self, ctx: &Ctx) {
        self.suspicion_day(ctx);
    }

    /// The households `person` (of `household`) knows to draw at each source, as of `day`: their
    /// own household where it has drawn for the content's days, the households it saw drawing
    /// there as often, and the households of those they know well enough, at their usual source.
    pub(crate) fn known_draws(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        household: PermanentId,
        day: i64,
    ) -> BTreeMap<Place, BTreeSet<PermanentId>> {
        let sp = &ctx.params.suspicion;
        let half = ctx.params.places.use_half_life_days;
        let mut draws: BTreeMap<Place, BTreeSet<PermanentId>> = BTreeMap::new();
        for place in self.sources_of(household) {
            let Some(u) = self.uses.held(household, place, day, half) else {
                continue;
            };
            if f64::from(u.days) >= sp.draws_min_days {
                draws.entry(place).or_default().insert(household);
            }
            for x in u
                .seen
                .iter()
                .filter(|x| f64::from(x.days) >= sp.draws_min_days)
            {
                draws.entry(place).or_default().insert(x.household);
            }
        }
        for t in self.ties.of(person) {
            if t.known_at(day, &ctx.params.ties) < sp.known_min {
                continue;
            }
            let Some(theirs) = self.person(t.to).map(|q| q.household) else {
                continue;
            };
            if theirs == household {
                continue;
            }
            if let Some(source) = self.usual_source(theirs, day, half, sp.draws_min_days) {
                draws.entry(source).or_default().insert(theirs);
            }
        }
        draws
    }

    /// The sources of water `household` has drawn at, in place order.
    fn sources_of(&self, household: PermanentId) -> Vec<Place> {
        self.uses
            .households
            .get(&household)
            .into_iter()
            .flatten()
            .map(|u| u.place)
            .filter(|p| matches!(p, Place::Source(_)))
            .collect()
    }

    /// Where `household` draws most, as of `day`, if it has drawn there for `min_days`.
    fn usual_source(
        &self,
        household: PermanentId,
        day: i64,
        half: f64,
        min_days: f64,
    ) -> Option<Place> {
        self.sources_of(household)
            .into_iter()
            .filter_map(|place| {
                let u = self.uses.held(household, place, day, half)?;
                (f64::from(u.days) >= min_days).then_some((u.days, place))
            })
            .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)))
            .map(|(_, place)| place)
    }
}
