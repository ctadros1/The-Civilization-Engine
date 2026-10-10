//! Writing ties (ADR-0014 §2): the acts people see each other do, recorded where they happen.
//! Standing is worked out here from the ties once a month.

use super::*;
use crate::ties::Act;

/// Purpose tag for the draw of a session's company at the hearth.
pub const PURPOSE_HEARTH: u64 = 0x6865_6172_7468_3031; // "hearth01"

impl Population {
    /// The member who stands for household `household` in what passes between households (a
    /// gift, wages, rent, a trade): its eldest of an age to keep a household, or else its
    /// eldest.
    pub(crate) fn elder_of(
        &self,
        household: PermanentId,
        now: SimTime,
        params: &PeopleParams,
    ) -> Option<PermanentId> {
        let x = self.household(household)?;
        let grown = x.members.iter().copied().find(|&m| {
            self.person(m)
                .is_some_and(|p| p.age_years(now) >= params.family.independent_age)
        });
        grown.or_else(|| {
            x.members
                .iter()
                .copied()
                .find(|&m| self.person(m).is_some())
        })
    }

    /// Records `units` of `act` by `to`, seen by `holder` now, with `help_h` hours of help
    /// received (negative: given).
    pub(crate) fn note_tie(
        &mut self,
        ctx: &Ctx,
        holder: PermanentId,
        to: PermanentId,
        act: Act,
        units: f64,
        help_h: f64,
    ) {
        self.ties.record(
            holder,
            to,
            act,
            units,
            help_h,
            ctx.now.day_index(),
            &ctx.params.ties,
        );
    }

    /// Records an act between households `a` and `b`: `who` (or else the member who stands for
    /// `a`, [`Population::elder_of`]) sees `b`'s elder do `seen_by_a`, and `b`'s elder sees them
    /// do `seen_by_b`; `help_h` hours of help pass from `b` to `a` (negative: from `a` to `b`).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn note_between(
        &mut self,
        ctx: &Ctx,
        who: Option<PermanentId>,
        a: PermanentId,
        b: PermanentId,
        seen_by_a: Act,
        seen_by_b: Act,
        help_h: f64,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let Some(mine) = who.or_else(|| self.elder_of(a, now, params)) else {
            return;
        };
        let Some(theirs) = self.elder_of(b, now, params) else {
            return;
        };
        self.note_tie(ctx, mine, theirs, seen_by_a, 1.0, help_h);
        self.note_tie(ctx, theirs, mine, seen_by_b, 1.0, -help_h);
    }

    /// Those keeping company at `hearth` now other than `me`, in id order. Each settlement's
    /// hearth is its own (the target names it), and company there is whoever is present
    /// (ADR-0018 §2). Only those who set to it since the last look, or were there then, are
    /// looked at: nobody comes to be there without setting to it (M5a slice AL).
    pub(crate) fn hearth_company(
        &mut self,
        ctx: &Ctx,
        me: PermanentId,
        hearth: Target,
    ) -> Vec<PermanentId> {
        let Target::Hearth(s) = hearth else {
            return Vec::new();
        };
        let present = |q: &Person| {
            q.trip.is_none()
                && matches!(
                    q.act.steps.get(q.act.step as usize),
                    Some(Step::Work { .. })
                )
                && ctx
                    .catalog
                    .activities
                    .get(q.act.def as usize)
                    .is_some_and(|a| is_company(a.behavior))
        };
        let at = self.at_hearth.get_or_insert_with(|| {
            let mut at: FastMap<PermanentId, AtHearth> = FastMap::default();
            for (h, q) in self.people.iter() {
                if let Target::Hearth(t) = q.act.target
                    && present(q)
                {
                    at.entry(t).or_default().push((q.id, h));
                }
            }
            for there in at.values_mut() {
                there.sort_unstable_by_key(|&(q, _)| q);
            }
            at
        });
        let Some(there) = at.get_mut(&s) else {
            return Vec::new();
        };
        let people = &self.people;
        there.retain(|&(_, h)| {
            people
                .get(h)
                .is_some_and(|q| q.act.target == hearth && present(q))
        });
        there.iter().map(|&(q, _)| q).filter(|&q| q != me).collect()
    }

    /// `me` keeps company at the hearth for `minutes` with `present` there: a few of them, drawn
    /// with known faces the likelier, become ties or warmer ones, each for a session's hours
    /// (ADR-0014 §2). Nobody's own household is company at the hearth: they keep company at
    /// home. A block of `sessions` sessions (Accelerated mode's, ADR-0011 §4) draws for each.
    pub(crate) fn keep_company(
        &mut self,
        ctx: &Ctx,
        me: PermanentId,
        present: &[PermanentId],
        minutes: u32,
        sessions: u32,
    ) {
        let tp = &ctx.params.ties;
        let Some(household) = self.person(me).map(|p| p.household) else {
            return;
        };
        let day = ctx.now.day_index();
        let others: Vec<PermanentId> = present
            .iter()
            .copied()
            .filter(|&q| self.person(q).is_some_and(|p| p.household != household))
            .collect();
        let sessions = sessions.max(1);
        let want = (tp.companions * sessions as usize).min(others.len());
        if want == 0 || minutes == 0 {
            return;
        }
        // Each companion is someone new with the content's share, if anyone new is there, and
        // otherwise someone known, the better known the likelier (research 04-04 §1.5: people
        // keep up the ties that matter to them). The draw is keyed by person and minute, so it
        // touches no other draw.
        let mut rng =
            Rng64::from_key(&[ctx.seed, PURPOSE_HEARTH, me.get(), ctx.now.minutes() as u64]);
        let (mut known, mut new): (Vec<(f64, PermanentId)>, Vec<PermanentId>) =
            (Vec::new(), Vec::new());
        for &q in &others {
            match self.ties.known(me, q, day, tp) {
                k if k > 0.0 => known.push((k, q)),
                _ => new.push(q),
            }
        }
        let hours = f64::from(minutes) / 60.0 / f64::from(sessions);
        let home_of = |p: &Population, q: PermanentId| {
            p.person(q)
                .and_then(|x| p.household(x.household))
                .and_then(|x| x.settlement)
        };
        let my_home = home_of(self, me);
        for _ in 0..want {
            let pick_new = !new.is_empty() && (known.is_empty() || rng.next_f64() < tp.new_share);
            let q = if pick_new {
                new.swap_remove((rng.next_u64() % new.len() as u64) as usize)
            } else {
                let total: f64 = known.iter().map(|k| k.0).sum();
                let mut at = rng.next_f64() * total;
                let mut k = known.len() - 1;
                for (i, (w, _)) in known.iter().enumerate() {
                    if at < *w {
                        k = i;
                        break;
                    }
                    at -= w;
                }
                known.remove(k).1
            };
            self.note_tie(ctx, me, q, Act::Hearth, hours, 0.0);
            // Word of the laws in force goes round at the hearth among those they bind
            // (ADR-0013 §3, stage 4); with someone from elsewhere, it is talk of another
            // settlement's laws, which nothing here carries yet (M5c).
            let neighbours = my_home.is_some() && home_of(self, q) == my_home;
            if neighbours {
                self.share_laws(me, q, day);
            }
            // So does word of who took from whom (ADR-0015 §1).
            self.share_takings(ctx, me, q);
            // And of gatherings called (M4c slice AE, ADR-0016 §3).
            self.share_word(ctx, me, q);
            // And where they stand on the questions of the day (M4c slice AG, ADR-0016 §4).
            self.share_opinion(ctx, me, q);
            // And whether their household paid its last levy (M4c slice AG, ADR-0016 §4): what
            // households of their own settlement do.
            if neighbours {
                self.share_norms(ctx, me, q);
            }
            // And of the ideologies they hold (M4c slice AG, ADR-0016 §4).
            self.share_ideologies(ctx, me, q);
            // And of the other settlements they know (M5a slice AM, ADR-0018 §4).
            self.share_places(ctx, me, q);
            // And of what is offered elsewhere (M5b slice AP, ADR-0019 §1).
            self.share_reports(ctx, me, q);
            // And of the places a polity's law claims (M5c slice AU, ADR-0020 §2).
            self.share_claims(ctx, me, q);
            // And of what a gathering decided on an agreement between their polities (M5c slice
            // AU, ADR-0020 §6).
            self.share_agreement_word(ctx, me, q);
        }
    }

    /// Works out every settlement's standing as of `now` from its adults' ties (ADR-0014 §3),
    /// the notables before keeping their place within the keep factor (§4).
    pub fn derive_standing(&mut self, now: SimTime, params: &PeopleParams) {
        let adults: Vec<(PermanentId, PermanentId)> = self
            .people
            .iter()
            .filter(|(_, p)| p.age_years(now) >= params.family.independent_age)
            .filter_map(|(_, p)| Some((self.household(p.household)?.settlement?, p.id)))
            .collect();
        self.standing = crate::standing::derive(
            &self.ties,
            &adults,
            now.day_index(),
            &params.ties,
            &params.standing,
            &self.standing,
        );
    }

    /// Lets go of every tie to someone no longer here (dead or gone), and of the ties those
    /// people held.
    pub(crate) fn prune_ties(&mut self) {
        let here: std::collections::HashSet<PermanentId> =
            self.people.iter().map(|(_, p)| p.id).collect();
        for holder in self.ties.holders() {
            if !here.contains(&holder) {
                self.ties.forget_holder(holder);
            }
        }
        self.ties.prune(|to| here.contains(&to));
        // And of failed searches for a partner by those no longer here (M5a slice AM).
        self.unmatched.retain(|id, _| here.contains(id));
        // And of the buildings they saw elsewhere (M5b slice AR).
        self.seen_away.retain(|id, _| here.contains(id));
    }
}
