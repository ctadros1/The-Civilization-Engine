//! The observer's interventions at work (M4c slice AJ, ADR-0016 §5; research 15-05 §4.2, §4.5).
//! A whisper places a true claim the settlement's word already holds in one person's hearing, as
//! if heard: what they do with it is what anyone who heard it would do, and it is let go as news
//! is. An ideology is heard of by one person from no one they know, and weighed as one heard at
//! the hearth is, by how well it fits what they hold dear (06-04 §1.4: exposure is not
//! acceptance); with no teller to doubt, only the fit gates it. The draw is keyed to the person and
//! the ideology alone, so telling them again changes nothing, and they weigh it again at their
//! reviews while it is fresh, so only a change in them can change the answer (15-05 §4.5: new
//! context can matter).

use super::*;
use crate::ideology::{Holding, adopt_chance, fit};
use crate::influence::{DAYS_RANGE, InfluenceKind, Luck, SHARE_RANGE};
use crate::norm::id_key;

/// Purpose tag for taking up an ideology heard of from the observer, keyed by person and ideology.
pub const PURPOSE_IDEOLOGY_HEARD: u64 = 0x6964_656f_6865_6172; // "ideohear"

/// What a god tool did, for its reply.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reached {
    /// The intervention's number.
    pub id: u32,
    /// Whether it repeated an earlier use, which it refreshed and did not add to.
    pub repeat: bool,
    /// For an ideology: whether they hold it now.
    pub holds: bool,
    /// For an ideology: how well it fits what they hold dear, and the chance it carried.
    pub fit: f64,
    pub chance: f64,
}

impl Population {
    /// The adult `person`'s given name and settlement, or why a god tool cannot reach them.
    fn reachable(&self, ctx: &Ctx, person: PermanentId) -> Result<(String, PermanentId), String> {
        let p = self
            .person(person)
            .ok_or_else(|| format!("{person} is not alive here"))?;
        if p.age_years(ctx.now) < ctx.params.family.independent_age {
            return Err(format!("{} is a child", p.given));
        }
        let settlement = self
            .household(p.household)
            .and_then(|x| x.settlement)
            .ok_or_else(|| format!("{} lives in no settlement", p.given))?;
        Ok((p.given.clone(), settlement))
    }

    /// The observer whispers claim `claim` to `person` (ADR-0016 §5): a true claim their
    /// settlement's word holds, which they have not heard, placed in their hearing as if heard,
    /// from no one. `words` is the claim in words, for the chronicle. A repeat refreshes the
    /// hearing and the record, and adds nothing.
    pub fn whisper(
        &mut self,
        ctx: &mut Ctx,
        person: PermanentId,
        claim: u32,
        words: &str,
    ) -> Result<Reached, String> {
        let (given, settlement) = self.reachable(ctx, person)?;
        let (now, day) = (ctx.now, ctx.now.day_index());
        let c = *self
            .word
            .claim(claim)
            .ok_or_else(|| format!("there is no news {claim} to whisper"))?;
        if c.settlement != settlement {
            return Err(format!(
                "{given} would not have heard of that where they live"
            ));
        }
        if let Some(k) = self.influences.find(InfluenceKind::Whisper, person, claim) {
            let i = &mut self.influences.list[k];
            i.last = now;
            i.uses += 1;
            let id = i.id;
            self.word.hear(person, claim, day, None, None);
            return Ok(Reached {
                id,
                repeat: true,
                holds: false,
                fit: 0.0,
                chance: 0.0,
            });
        }
        if self.word.has_heard(person, claim) {
            return Err(format!("{given} has heard that already"));
        }
        self.word.hear(person, claim, day, None, None);
        let id = self
            .influences
            .add(now, InfluenceKind::Whisper, person, claim);
        self.chronicle_push(
            now,
            ChronicleKind::Influence,
            vec![person],
            Some(settlement),
            None,
            f64::from(InfluenceKind::Whisper.code()),
            format!(" that {words}."),
        );
        Ok(Reached {
            id,
            repeat: false,
            holds: false,
            fit: 0.0,
            chance: 0.0,
        })
    }

    /// The observer tells `person` of ideology `k` (ADR-0016 §5, as `IntroduceTechnique {
    /// aware_only }` makes a technique known): they weigh it at once, and again at their reviews
    /// while it is fresh. A repeat refreshes the record and draws nothing new.
    pub fn tell_of_ideology(
        &mut self,
        ctx: &mut Ctx,
        person: PermanentId,
        k: u16,
    ) -> Result<Reached, String> {
        let (given, settlement) = self.reachable(ctx, person)?;
        let now = ctx.now;
        let def = ctx
            .catalog
            .ideologies
            .get(usize::from(k))
            .ok_or_else(|| format!("there is no ideology {k}"))?;
        let subject = u32::from(k);
        if let Some(i) = self
            .influences
            .find(InfluenceKind::Ideology, person, subject)
        {
            let r = &mut self.influences.list[i];
            r.last = now;
            r.uses += 1;
            let id = r.id;
            let (fit, chance) = self.ideology_chance(ctx.catalog, person, k);
            return Ok(Reached {
                id,
                repeat: true,
                holds: self.ideologies.holds(person, k),
                fit,
                chance,
            });
        }
        if self.ideologies.holds(person, k) {
            return Err(format!(
                "{given} holds to {} already",
                def.name.to_lowercase()
            ));
        }
        let id = self
            .influences
            .add(now, InfluenceKind::Ideology, person, subject);
        self.chronicle_push(
            now,
            ChronicleKind::Influence,
            vec![person],
            Some(settlement),
            None,
            f64::from(InfluenceKind::Ideology.code()),
            format!(" of {}.", def.name.to_lowercase()),
        );
        let (fit, chance) = self.weigh_heard_ideology(ctx, self.influences.list.len() - 1);
        Ok(Reached {
            id,
            repeat: false,
            holds: self.ideologies.holds(person, k),
            fit,
            chance,
        })
    }

    /// How well ideology `k` fits what `person` holds dear, and the chance one heard of from no
    /// one carries for them.
    pub fn ideology_chance(&self, catalog: &Catalog, person: PermanentId, k: u16) -> (f64, f64) {
        let Some(def) = catalog.ideologies.get(usize::from(k)) else {
            return (0.0, 0.0);
        };
        let fits = fit(&def.commitments, &|v| {
            self.values.get(person, v).map_or(0.0, f64::from)
        });
        (fits, adopt_chance(def, 1.0, fits))
    }

    /// The one told of an ideology in record `i` weighs it: they take it up if the draw keyed to
    /// them and the ideology falls under the chance it carries for them now.
    fn weigh_heard_ideology(&mut self, ctx: &Ctx, i: usize) -> (f64, f64) {
        let r = self.influences.list[i];
        let Ok(k) = u16::try_from(r.subject) else {
            return (0.0, 0.0);
        };
        let Some(def) = ctx.catalog.ideologies.get(usize::from(k)) else {
            return (0.0, 0.0);
        };
        let (fits, chance) = self.ideology_chance(ctx.catalog, r.target, k);
        self.influences.list[i].weighed += 1;
        if self.ideologies.holds(r.target, k) {
            return (fits, chance);
        }
        let u = Rng64::from_key(&[
            ctx.seed,
            PURPOSE_IDEOLOGY_HEARD,
            r.target.get(),
            id_key(&def.id),
        ])
        .next_f64();
        if u < chance {
            let day = ctx.now.day_index();
            self.ideologies.take_up(Holding {
                holder: r.target,
                ideology: k,
                since: day,
                from: None,
            });
            self.influences.list[i].taken = Some(day);
        }
        (fits, chance)
    }

    /// At their review, `person` weighs again each ideology the observer told them of that they
    /// have not taken up, while it is fresh (heard within the content's `news_days`).
    pub(super) fn weigh_heard_ideologies(&mut self, ctx: &Ctx, person: PermanentId) {
        let day = ctx.now.day_index();
        let fresh = i64::from(ctx.params.word.news_days);
        let due: Vec<usize> = self
            .influences
            .list
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.target == person
                    && r.kind == InfluenceKind::Ideology
                    && r.taken.is_none()
                    && day - r.last.day_index() <= fresh
            })
            .map(|(i, _)| i)
            .collect();
        for i in due {
            self.weigh_heard_ideology(ctx, i);
        }
    }

    /// The observer blesses `person`, or with `curse` curses them, for `days` days by `share`
    /// (M4c slice AJ, ADR-0016 §5): their own draws for illness or accident and for finding
    /// things out are moved their way, or against them, as [`Luck`] says. Never a decision's
    /// draw, a vote, a tie, detection or punishment. One in force of the same kind is refreshed,
    /// its period running from today if that is later, and never stacks; one of the other kind
    /// ends today.
    pub fn bless(
        &mut self,
        ctx: &mut Ctx,
        person: PermanentId,
        curse: bool,
        days: u32,
        share: f32,
    ) -> Result<Reached, String> {
        let p = self
            .person(person)
            .ok_or_else(|| format!("{person} is not alive here"))?;
        let settlement = self.household(p.household).and_then(|x| x.settlement);
        if !(DAYS_RANGE[0]..=DAYS_RANGE[1]).contains(&days) {
            return Err(format!(
                "a blessing lasts {} to {} days",
                DAYS_RANGE[0], DAYS_RANGE[1]
            ));
        }
        if !(SHARE_RANGE[0]..=SHARE_RANGE[1]).contains(&share) {
            return Err(format!(
                "a blessing moves a draw by {} to {} of its way",
                SHARE_RANGE[0], SHARE_RANGE[1]
            ));
        }
        let (now, day) = (ctx.now, ctx.now.day_index());
        let until = day + i64::from(days);
        if let Some(l) = self.influences.luck(person, day)
            && l.bless == curse
        {
            self.influences.list[l.index].until = day;
        }
        if let Some(l) = self.influences.luck(person, day) {
            let r = &mut self.influences.list[l.index];
            r.until = r.until.max(until);
            r.share = share;
            r.last = now;
            r.uses += 1;
            return Ok(Reached {
                id: r.id,
                repeat: true,
                holds: false,
                fit: 0.0,
                chance: 0.0,
            });
        }
        let kind = if curse {
            InfluenceKind::Curse
        } else {
            InfluenceKind::Bless
        };
        let id = self.influences.add(now, kind, person, 0);
        if let Some(r) = self.influences.list.last_mut() {
            r.until = until;
            r.share = share;
        }
        let luck = Luck {
            index: 0,
            bless: !curse,
            share: f64::from(share),
        };
        self.chronicle_push(
            now,
            ChronicleKind::Influence,
            vec![person],
            settlement,
            None,
            f64::from(kind.code()),
            format!(
                " for {days} days: of their own chances, that of illness or accident is {:.2} of \
                 what it was, and that of finding things out {:.2}.",
                luck.harm(),
                luck.fortune()
            ),
        );
        Ok(Reached {
            id,
            repeat: false,
            holds: false,
            fit: 0.0,
            chance: 0.0,
        })
    }

    /// A blessing or a curse turned one of `person`'s draws: a death by illness or accident
    /// (`technique` none) or a find of `technique`. Its record counts it and the chronicle tells
    /// it as one recorded influence.
    pub(super) fn note_turned(
        &mut self,
        ctx: &Ctx,
        person: PermanentId,
        luck: Luck,
        technique: Option<usize>,
    ) {
        let Some(r) = self.influences.list.get_mut(luck.index) else {
            return;
        };
        let id = r.id;
        match technique {
            None => r.deaths += 1,
            Some(_) => r.finds += 1,
        }
        let what = match (technique, luck.bless) {
            (None, true) => " was spared a death by illness or accident: the observer's blessing \
                              turned the draw."
                .to_owned(),
            (None, false) => {
                " died of illness or accident by a draw the observer's curse turned.".to_owned()
            }
            (Some(t), bless) => {
                let name = ctx
                    .catalog
                    .techniques
                    .get(t)
                    .map_or_else(|| "something".to_owned(), |d| d.name.to_lowercase());
                if bless {
                    format!(" found {name} by a draw the observer's blessing turned.")
                } else {
                    format!(" did not find {name}: the observer's curse turned the draw.")
                }
            }
        };
        let settlement = self
            .person(person)
            .and_then(|p| self.household(p.household))
            .and_then(|x| x.settlement);
        self.chronicle_push(
            ctx.now,
            ChronicleKind::InfluenceTurned,
            vec![person],
            settlement,
            None,
            f64::from(id),
            what,
        );
    }
}
