//! Ideologies at work (M4c slice AG, step four, ADR-0016 §4; research 06-04 §1.4, §1.7, §6.1).
//! Each midnight anyone new takes their start: one with no parents recorded (a founder, a
//! newcomer) brings each ideology content names by its share of founders; a child takes up what a
//! parent holds by its chance of inheritance. At the hearth a holder now and then speaks of what
//! they hold, and a listener takes it up by their trust in the teller and how well it fits what
//! they hold dear (06-04 §1.4: exposure is not acceptance). A holder weighs every law with the
//! ideology's commitments beside their own values, and, when the problem it explains is before
//! the village, weighs the laws its program names among their moves; a law they propose so keeps
//! the creed it was proposed under.

use super::*;
use crate::ideology::{Holding, adopt_chance, fit};
use crate::norm::id_key;
use crate::polity::IssueKind;

/// Purpose tags for a founder's bringing an ideology, a child's taking up a parent's, a holder's
/// speaking of one, and a listener's taking it up.
pub const PURPOSE_IDEOLOGY_FOUNDER: u64 = 0x6964_656f_666f_756e; // "ideofoun"
pub const PURPOSE_IDEOLOGY_INHERIT: u64 = 0x6964_656f_696e_6865; // "ideoinhe"
pub const PURPOSE_IDEOLOGY_TELL: u64 = 0x6964_656f_7465_6c6c; // "ideotell"
pub const PURPOSE_IDEOLOGY_ADOPT: u64 = 0x6964_656f_6164_6f70; // "ideoadop"

impl Population {
    /// Midnight: holdings of those no longer here are let go, and anyone new takes their start.
    pub(super) fn take_ideology_starts(&mut self, ctx: &Ctx) {
        let index = &self.index;
        self.ideologies
            .held
            .retain(|h| index.contains_key(&h.holder));
        let seen = self.ideologies.seen;
        let mut new: Vec<PermanentId> = self
            .people
            .iter()
            .map(|(_, p)| p.id)
            .filter(|id| id.get() > seen)
            .collect();
        if new.is_empty() {
            return;
        }
        new.sort_unstable();
        let day = ctx.now.day_index();
        for &id in &new {
            let Some(p) = self.person(id) else {
                continue;
            };
            let parents: Vec<PermanentId> = [p.mother, p.father].into_iter().flatten().collect();
            for (k, def) in ctx.catalog.ideologies.iter().enumerate() {
                let k = k as u16;
                let key = id_key(&def.id);
                let from = if parents.is_empty() {
                    let u = Rng64::from_key(&[ctx.seed, PURPOSE_IDEOLOGY_FOUNDER, id.get(), key])
                        .next_f64();
                    (u < def.founders).then_some(None)
                } else {
                    parents
                        .iter()
                        .copied()
                        .filter(|&q| self.ideologies.holds(q, k))
                        .find(|&q| {
                            Rng64::from_key(&[
                                ctx.seed,
                                PURPOSE_IDEOLOGY_INHERIT,
                                id.get(),
                                q.get(),
                                key,
                            ])
                            .next_f64()
                                < def.inherit
                        })
                        .map(Some)
                };
                if let Some(from) = from {
                    self.ideologies.take_up(Holding {
                        holder: id,
                        ideology: k,
                        since: day,
                        from,
                    });
                }
            }
        }
        self.ideologies.seen = new.last().map_or(seen, |id| id.get().max(seen));
    }

    /// `a` and `b` keep company at the hearth: each may speak of an ideology they hold, by its
    /// chance, and the other, if they do not hold it, takes it up by their trust in the teller and
    /// how well it fits what they hold dear. Who says what is fixed first, so the order of the two
    /// changes nothing.
    pub(crate) fn share_ideologies(&mut self, ctx: &Ctx, a: PermanentId, b: PermanentId) {
        let ideas = &ctx.catalog.ideologies;
        if ideas.is_empty() {
            return;
        }
        let (day, minute) = (ctx.now.day_index(), ctx.now.minutes() as u64);
        let mut said: Vec<(PermanentId, PermanentId, u16)> = Vec::new();
        for (teller, listener) in [(a, b), (b, a)] {
            for h in self.ideologies.held_by(teller) {
                let Some(def) = ideas.get(usize::from(h.ideology)) else {
                    continue;
                };
                let key = [
                    ctx.seed,
                    PURPOSE_IDEOLOGY_TELL,
                    teller.get(),
                    listener.get(),
                    id_key(&def.id),
                    minute,
                ];
                if Rng64::from_key(&key).next_f64() < def.share {
                    said.push((listener, teller, h.ideology));
                }
            }
        }
        for (listener, teller, k) in said {
            self.ideologies.told += 1;
            if self.ideologies.holds(listener, k) {
                continue;
            }
            let def = &ideas[usize::from(k)];
            let trust = self
                .ties
                .regard(listener, teller, day, &ctx.params.ties)
                .clamp(0.0, 1.0);
            let fits = fit(&def.commitments, &|v| {
                self.values.get(listener, v).map_or(0.0, f64::from)
            });
            let key = [
                ctx.seed,
                PURPOSE_IDEOLOGY_ADOPT,
                listener.get(),
                teller.get(),
                id_key(&def.id),
                minute,
            ];
            if Rng64::from_key(&key).next_f64() < adopt_chance(def, trust, fits) {
                self.ideologies.take_up(Holding {
                    holder: listener,
                    ideology: k,
                    since: day,
                    from: Some(teller),
                });
                self.ideologies.taken += 1;
            }
        }
    }

    /// The points the ideologies `person` holds add to what they make of a law of template
    /// `policy`: each one's commitments, as if they were values they hold (06-04 §1.1).
    pub(super) fn creed_points(&self, ctx: &Ctx, person: PermanentId, policy: u16) -> f64 {
        let Some(def) = ctx.catalog.policies.get(usize::from(policy)) else {
            return 0.0;
        };
        let values = &ctx.catalog.values;
        self.ideologies
            .held_by(person)
            .iter()
            .filter_map(|h| ctx.catalog.ideologies.get(usize::from(h.ideology)))
            .map(|idea| {
                def.bears
                    .iter()
                    .map(|&(k, b)| {
                        let c = idea
                            .commitments
                            .iter()
                            .find(|c| c.0 == k)
                            .map_or(0.0, |c| f64::from(c.1));
                        let w = values.get(usize::from(k)).map_or(0.0, |v| v.weight);
                        w * c * f64::from(b)
                    })
                    .sum::<f64>()
            })
            .sum()
    }

    /// The ideology whose program names template `policy` that `person` holds, with the most
    /// points for proposing it, and those points.
    pub(super) fn creed_for(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        policy: u16,
    ) -> Option<(u16, f64)> {
        self.ideologies
            .held_by(person)
            .iter()
            .filter_map(|h| {
                let def = ctx.catalog.ideologies.get(usize::from(h.ideology))?;
                def.program
                    .contains(&policy)
                    .then_some((h.ideology, def.w_program))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
    }

    /// The templates the ideologies `person` holds propose for the problems before them
    /// (`issues`), with the issue each explains, in template order: an ideology shapes the answer
    /// to a problem when it is felt (06-04 §1.2), not in its absence.
    pub(super) fn programs_of(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        issues: &[IssueKind],
    ) -> Vec<(u16, IssueKind)> {
        let mut out: Vec<(u16, IssueKind)> = self
            .ideologies
            .held_by(person)
            .iter()
            .filter_map(|h| ctx.catalog.ideologies.get(usize::from(h.ideology)))
            .filter(|def| issues.contains(&def.explains))
            .flat_map(|def| def.program.iter().map(move |&k| (k, def.explains)))
            .collect();
        out.sort_by_key(|p| p.0);
        out.dedup_by_key(|p| p.0);
        out
    }
}
