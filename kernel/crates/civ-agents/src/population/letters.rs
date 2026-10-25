//! Letters and records (M10c slice EC; ADR-0035 §6; brief m10-services §3.3; research 06-08
//! §1.5, 07-09 §1.10 and §5.1): reckoning, reading and writing are techniques like any other,
//! which founders do not bring. Someone who can write, and who alone of their settlement knows a
//! technique no record it holds keeps, may write it down at home; once the hours of a record are
//! written, their household holds it. Someone who can read, whose household holds a record of a
//! technique they do not know, may read it at home: they hear of the technique there, and where it
//! can be learnt from the page, they learn toward it at a multiple of the hours beside a knower.

use super::*;
use crate::knowledge::Record;
use crate::person::KnowSource;

impl Population {
    /// The technique the work of behaviour `b` needs, if any: writing for writing a record,
    /// reading for reading one.
    fn letter_technique(catalog: &Catalog, b: Behavior) -> Option<usize> {
        catalog
            .activities
            .iter()
            .find(|a| a.behavior == b)
            .and_then(|a| a.technique)
    }

    /// Whether someone of settlement `s` other than `who` knows technique `t`.
    fn another_knows(&self, s: PermanentId, t: usize, who: PermanentId) -> bool {
        self.households
            .iter()
            .filter(|(_, x)| x.settlement == Some(s))
            .flat_map(|(_, x)| x.members.iter())
            .any(|&m| m != who && self.person(m).is_some_and(|q| q.knows(t)))
    }

    /// Whether a household of settlement `s` holds a record of technique `t`.
    fn record_held_in(&self, s: PermanentId, t: usize) -> bool {
        self.writings.iter().any(|r| {
            usize::from(r.technique) == t
                && self
                    .household(r.holder)
                    .is_some_and(|x| x.settlement == Some(s))
        })
    }

    /// The technique person `who` would write down (M10c slice EC): one they know, the one they
    /// are writing first, that nobody else of their settlement knows and no record held there
    /// keeps; none unless they know writing.
    pub(super) fn write_facts(&self, ctx: &Ctx, who: PermanentId) -> Option<u16> {
        let writing = Self::letter_technique(ctx.catalog, Behavior::Write)?;
        let p = self.person(who)?;
        if !p.knows(writing) {
            return None;
        }
        let s = self.household(p.household)?.settlement?;
        let begun = self
            .written
            .iter()
            .filter(|w| w.person == who)
            .map(|w| w.technique);
        let known = p.knows.iter().filter(|k| k.known).map(|k| k.technique);
        begun.chain(known).find(|&t| {
            let t = usize::from(t);
            p.knows(t) && !self.another_knows(s, t, who) && !self.record_held_in(s, t)
        })
    }

    /// Person `who` wrote technique `t` for `minutes` (M10c slice EC): once the hours of a record
    /// are written, their household holds it.
    pub(super) fn finish_write(&mut self, ctx: &mut Ctx, who: PermanentId, t: u16, minutes: f64) {
        crate::knowledge::tally(&mut self.written, who, t, minutes as f32);
        let done = crate::knowledge::tallied(&self.written, who, t);
        if f64::from(done) + 1e-6 < ctx.params.knowledge.record_h * 60.0 {
            return;
        }
        self.written
            .retain(|w| !(w.person == who && w.technique == t));
        let Some(holder) = self.person(who).map(|p| p.household) else {
            return;
        };
        self.writings.push(Record {
            id: ctx.ids.allocate(),
            technique: t,
            holder,
            maker: who,
            made: ctx.now.day_index(),
        });
    }

    /// The technique person `who` would read of (M10c slice EC): the first record their household
    /// holds of a technique they do not know, unless they have heard of it already and it cannot
    /// be learnt from the page; none unless they know reading.
    pub(super) fn read_facts(&self, ctx: &Ctx, who: PermanentId) -> Option<u16> {
        if self.writings.is_empty() {
            return None;
        }
        let reading = Self::letter_technique(ctx.catalog, Behavior::Read)?;
        let p = self.person(who)?;
        if !p.knows(reading) {
            return None;
        }
        self.writings
            .iter()
            .filter(|r| r.holder == p.household)
            .map(|r| r.technique)
            .find(|&t| {
                let i = usize::from(t);
                !p.knows(i)
                    && (p.know(i).is_none()
                        || ctx
                            .catalog
                            .techniques
                            .get(i)
                            .is_some_and(|d| d.codified > 0.0))
            })
    }

    /// Person `who` read a record of technique `t` for `minutes` (M10c slice EC): they hear of it
    /// from its writer, and where it can be learnt from the page, learn toward it at `codified`
    /// times the hours beside a knower.
    pub(super) fn finish_read(&mut self, ctx: &Ctx, who: PermanentId, t: u16, minutes: f64) {
        let Some(p) = self.person(who) else {
            return;
        };
        let Some(maker) = self
            .writings
            .iter()
            .find(|r| r.holder == p.household && r.technique == t)
            .map(|r| r.maker)
        else {
            return;
        };
        let Some(def) = ctx.catalog.techniques.get(usize::from(t)) else {
            return;
        };
        let (codified, learn_h, now) = (def.codified, def.learn_h, ctx.now);
        let source = KnowSource::Read(maker);
        let i = usize::from(t);
        let learnt = self.person_mut_by_id(who).is_some_and(|p| {
            if codified > 0.0 {
                p.learn_from(i, minutes / 60.0 / codified, source, learn_h, now)
            } else {
                p.hear_of(i, source, now);
                false
            }
        });
        if learnt {
            self.on_known(ctx, who, i, source);
        }
    }

    /// Person `who` writes technique `t` for `minutes` now: for tests.
    #[doc(hidden)]
    pub fn finish_write_for_tests(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        t: u16,
        minutes: f64,
    ) {
        self.finish_write(ctx, who, t, minutes);
    }

    /// What person `who` would write and read now: for tests.
    #[doc(hidden)]
    pub fn letters_for_tests(&self, ctx: &Ctx, who: PermanentId) -> (Option<u16>, Option<u16>) {
        (self.write_facts(ctx, who), self.read_facts(ctx, who))
    }
}
