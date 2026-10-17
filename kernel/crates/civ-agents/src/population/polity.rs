//! The polity at work (ADR-0013): each settlement's polity founded under the custom every world
//! shares, the weekly review in which those who may propose weigh their moves, the gathering at
//! the hearth that decides, the levy at threshing and relief from the store, and word of a law
//! going round. Every choice here is a person's, scored by what they forecast for their household
//! and those who regard them; no issue weighs toward any policy.

use super::*;
use crate::decide::GatheringFacts;
use crate::polity::{
    Belief, Body, CustomVersion, Deliberator, Gathering, IssueKind, Law, LawStatus, Membership,
    MoveOption, Outcome, Outlook, PolicyKind, Polity, RuleDeliberator, Stance, StanceRecord,
};
use crate::word::Blamed;

/// A decision someone saw made at a gathering (M4c slice AF): each stance taken, by person and
/// household, with what the household stood to gain in points, and whether it passed.
pub(super) struct Seen {
    pub(super) stances: Vec<(PermanentId, PermanentId, Stance, f64)>,
    pub(super) passed: bool,
}

/// What a settlement's households expect (ADR-0013 §5), for forecasting what a move would bring
/// each: built once per review or month.
pub(super) struct Forecasts {
    /// Each household's outlook, by household id.
    pub(super) outlooks: Vec<(PermanentId, Outlook)>,
    /// What the settlement's people believe of their years.
    belief: Belief,
    /// What a roof over the store would keep each household a year, kcal.
    kept: f64,
    /// What each household knows of takings in the last year, by household id.
    takings: Vec<(PermanentId, crate::crime::TakingsKnown)>,
    /// Grown members of each household, whom a curfew keeps at home.
    grown: BTreeMap<PermanentId, u32>,
    /// Those who keep the watch now, with their laws (M4c slice AI: a watch of several).
    watchers: Vec<(PermanentId, PermanentId)>,
    /// What each household believes a claim on the places outsiders were seen at would keep it
    /// a year, kcal (M5c slice AT), by household id.
    claim: Vec<(PermanentId, f64)>,
    /// Where the settlement's people could build a crossing together, in cell order (M5c slice
    /// AW, step three).
    pub(super) sites: Vec<super::crossings::PublicSite>,
    /// What an hour of each household's work brings it in food, kcal, by household id: its
    /// cheapest food at its own cost in hours (M3a). The walking a crossing saves and the work
    /// it asks are worth that (research 11-07 §2.3: communal labour is priced at the production
    /// it forgoes). Kept only while there is a site.
    hour_kcal: Vec<(PermanentId, f64)>,
    /// What closing each source of water someone of the settlement suspects would bring each
    /// household (M6a slice BA, step four), by the source's cell, then by household id: the food
    /// its sickness believed avoided is worth, and the food its longer walks would cost, kcal.
    closures: Vec<(u32, Vec<ClosureForecast>)>,
    /// Where the settlement's people could dig a well together, if anywhere (M6a slice BA, step
    /// four).
    pub(super) well: Option<super::wells::PublicWell>,
    /// Where the settlement's people could build a flood bank together, if anywhere (M6b slice
    /// BG).
    pub(super) bank: Option<super::levees::PublicBank>,
    /// A bank the polity built that wants mending and that no law keeps, if any (M6b slice BG).
    pub(super) worn: Option<super::levees::WornBank>,
    /// The palisade the settlement's people could raise round their hearth together, if any
    /// (M6c slice BK).
    pub(super) enclosure: Option<super::enclosures::PublicEnclosure>,
    /// What each rule for building would bring each household (M6b slice BH), by template index
    /// and then household id: the food a year the fire loss it believes averted could bring, and
    /// the food a year what its next building would cost more could, kcal.
    rules: super::rules::RuleForecasts,
    /// A load of food someone hopes to carry home from a band, kcal (M6c slice BJ): a load
    /// of the densest food at `bands.food_prior` of it.
    band_load: f64,
    /// Each household's share of the store of the polity a band of its law holds, kcal, as its
    /// people know it there (M6c slice BM): what taking it in would bring each once.
    annex_share: f64,
    /// The cases the gathering or a forum heard within memory (M7b slices BR, BS): what one
    /// weighing a forum judges it by.
    forum_cases: Vec<ForumCase>,
    /// What a duty or an embargo would bring each household (M7c slice BV).
    trade: TradeForecast,
    /// What each household is owed on loans refused or left unpaid and not yet heard, and what
    /// it owes on such loans, kcal (M7d slice CB), by household id.
    debts: Vec<(PermanentId, f64, f64)>,
    /// The food the common store holds, kcal, and the households it serves (M7d slice CC):
    /// what lending from it could lend each.
    store_food: (f64, usize),
    /// Kilograms of metal paid by weight each household paid or was paid a year (M7d slice
    /// CE2b), by household id: the weighing a mint would save it.
    weighed: Vec<(PermanentId, f64)>,
    /// For each mint template, by index: the hours of weighing a kilogram paid in its coin saves,
    /// and the hours striking a kilogram takes its keeper.
    mints: Vec<(u16, f64, f64)>,
    /// The settlement's adults and their households, by person id: whose household a keeper
    /// named would work for the office.
    adults: Vec<(PermanentId, PermanentId)>,
    /// What each household would be paid a year under a law of pay for common work, kcal, by
    /// template index and then household id (M7d slice CE3a): its share of the money the store
    /// holds, at what a unit is worth to it in hours, at what its hours bring in food.
    pay: Vec<(u16, Vec<(PermanentId, f64)>)>,
}

/// What a household weighs a trade law by (M7c slice BV, ADR-0026 §1): its settlement's
/// purchases at other settlements' doors within a year (the other markets' records), what a unit
/// of each good bought is worth to each household, kcal, and which households offer goods of each
/// purpose at home.
#[derive(Default)]
pub(super) struct TradeForecast {
    away: Vec<super::duties::BoughtAway>,
    /// (household, good, kcal a unit), in household then good order.
    unit: Vec<(PermanentId, usize, f64)>,
    /// (household, purpose) for each purpose a household offers goods of at home.
    sellers: Vec<(PermanentId, GoodUse)>,
    /// Each good's purpose, by index in the catalog's goods.
    purpose: Vec<GoodUse>,
    households: usize,
    /// The polity's duties in force (M7c slice BW): each one's purpose, rate, and whether it
    /// spares those who offer such goods at home.
    duties: Vec<(GoodUse, f64, bool)>,
    /// The organizations by stake with members among the polity's households (M7c slice BW):
    /// each one's id, purpose, those households in order, and whether a law in force of the
    /// polity recognizes it.
    orgs: Vec<(PermanentId, GoodUse, Vec<PermanentId>, bool)>,
    /// Sales at the settlement's own doors within a year (M7c slice BX): (the selling household,
    /// a workshop's owner for a workshop's, the buying household, good, units).
    home: Vec<(PermanentId, PermanentId, usize, f64)>,
}

impl TradeForecast {
    fn unit(&self, h: PermanentId, good: usize) -> f64 {
        self.unit
            .binary_search_by_key(&(h, good), |u| (u.0, u.1))
            .map_or(0.0, |i| self.unit[i].2)
    }

    fn sells(&self, h: PermanentId, purpose: GoodUse) -> bool {
        self.sellers.contains(&(h, purpose))
    }

    fn sellers_of(&self, purpose: GoodUse) -> usize {
        self.sellers.iter().filter(|s| s.1 == purpose).count()
    }

    /// Whether household `h` owes no duty on goods of `purpose`: one that spares sellers and it
    /// offers them at home, or an organization of that purpose its people belong to is recognized
    /// (M7c slice BW).
    fn spared(&self, h: PermanentId, purpose: GoodUse, exempt: bool) -> bool {
        (exempt && self.sells(h, purpose))
            || self
                .orgs
                .iter()
                .any(|o| o.3 && o.1 == purpose && o.2.binary_search(&h).is_ok())
    }

    /// What a duty of `rate` on goods of `purpose` would bring household `h` a year, kcal: what
    /// its own purchases would owe (none when it is spared as one that offers such goods at
    /// home), and, as one that offers them, its share of the buyers it expects back (others'
    /// purchases at the rate, a design prior: the dearer by the duty, the more come home) at
    /// its margin, and its share of what the store would take.
    pub(super) fn duty(
        &self,
        h: PermanentId,
        rate: f64,
        purpose: GoodUse,
        exempt: bool,
        margin: f64,
    ) -> (f64, f64) {
        let of = |b: &&super::duties::BoughtAway| self.purpose.get(b.good) == Some(&purpose);
        let sells = self.sells(h, purpose);
        let owes: f64 = if self.spared(h, purpose, exempt) {
            0.0
        } else {
            self.away
                .iter()
                .filter(of)
                .filter(|b| b.household == h)
                .map(|b| b.units * rate * self.unit(h, b.good))
                .sum()
        };
        let back: f64 = if sells {
            self.away
                .iter()
                .filter(of)
                .filter(|b| b.household != h)
                .map(|b| b.units * rate * self.unit(h, b.good) * margin)
                .sum::<f64>()
                / self.sellers_of(purpose).max(1) as f64
        } else {
            0.0
        };
        let taken: f64 = self
            .away
            .iter()
            .filter(of)
            .filter(|b| !self.spared(b.household, purpose, exempt))
            .map(|b| b.units * rate * self.unit(h, b.good))
            .sum::<f64>()
            / self.households.max(1) as f64;
        (back + taken, owes)
    }

    /// What an embargo on `market`'s doors would bring household `h` a year, kcal: the saving its
    /// own purchases there made (at the market's margin, a design prior: what a buyer saves going
    /// elsewhere is what a seller at home would take over its cost), and, as one that offers goods
    /// of a purpose bought there, its share of the buyers who would come home.
    fn embargo(&self, h: PermanentId, market: PermanentId, margin: f64) -> (f64, f64) {
        let there = || self.away.iter().filter(move |b| b.market == market);
        let lost: f64 = there()
            .filter(|b| b.household == h)
            .map(|b| b.units * self.unit(h, b.good) * margin)
            .sum();
        let back: f64 = there()
            .filter(|b| b.household != h)
            .filter_map(|b| {
                let p = *self.purpose.get(b.good)?;
                self.sells(h, p).then(|| {
                    b.units * self.unit(h, b.good) * margin / self.sellers_of(p).max(1) as f64
                })
            })
            .sum();
        (back, lost)
    }

    /// What a privilege `def` for organization `org` would bring household `h` a year, kcal
    /// (M7c slice BX), and what it would cost it, at the market's margin (a design prior, as for
    /// an embargo). A licence: an organization's member that offers such goods gets its share of
    /// the sales at home others made; another that sells them loses its own sales, and a buyer
    /// pays the margin on what it bought from others (fewer sellers, dearer). A staple: a member
    /// that offers them gets its share of what others who offer them fetched from elsewhere;
    /// those others lose the margin on it.
    fn privilege(
        &self,
        h: PermanentId,
        org: PermanentId,
        def: crate::polity::PrivilegeDef,
        margin: f64,
    ) -> (f64, f64) {
        let Some((_, _, members, _)) = self.orgs.iter().find(|o| o.0 == org) else {
            return (0.0, 0.0);
        };
        let member = |x: PermanentId| members.binary_search(&x).is_ok();
        let of = |g: usize| self.purpose.get(g) == Some(&def.purpose);
        let member_sellers = members
            .iter()
            .filter(|&&m| self.sells(m, def.purpose))
            .count()
            .max(1) as f64;
        let shares = member(h) && self.sells(h, def.purpose);
        let (mut gain, mut cost) = (0.0, 0.0);
        match def.grant {
            crate::polity::Grant::Licence => {
                for &(seller, buyer, g, units) in
                    self.home.iter().filter(|s| of(s.2) && !member(s.0))
                {
                    let v = units * self.unit(h, g) * margin;
                    if shares {
                        gain += v / member_sellers;
                    }
                    if seller == h || buyer == h {
                        cost += v;
                    }
                }
            }
            crate::polity::Grant::Staple => {
                for b in self.away.iter().filter(|b| {
                    of(b.good) && !member(b.household) && self.sells(b.household, def.purpose)
                }) {
                    let v = b.units * self.unit(h, b.good) * margin;
                    if shares {
                        gain += v / member_sellers;
                    }
                    if b.household == h {
                        cost += v;
                    }
                }
            }
        }
        (gain, cost)
    }

    /// What recognizing organization `org` would bring household `h` a year, kcal (M7c slice
    /// BW): the duties in force its own purchases of the organization's goods would no longer
    /// owe, as a member's household; and what it costs, its share of what the store would no
    /// longer take from members' households.
    fn recognize(&self, h: PermanentId, org: PermanentId) -> (f64, f64) {
        let Some((_, purpose, members, _)) = self.orgs.iter().find(|o| o.0 == org) else {
            return (0.0, 0.0);
        };
        let (mut saved, mut lost) = (0.0, 0.0);
        for &(_, rate, exempt) in self.duties.iter().filter(|d| d.0 == *purpose) {
            for b in self.away.iter().filter(|b| {
                self.purpose.get(b.good) == Some(purpose)
                    && members.binary_search(&b.household).is_ok()
                    && !self.spared(b.household, *purpose, exempt)
            }) {
                let v = b.units * rate * self.unit(h, b.good);
                if b.household == h {
                    saved += v;
                }
                lost += v / self.households.max(1) as f64;
            }
        }
        (saved, lost)
    }
}

/// A case the gathering heard within memory, as one weighing a forum sees it (M7b slice BR,
/// ADR-0025 §1): the household that brought it and the accused's, what it said it lost, kcal,
/// whether it was found, and each stance taken (person, household, stance).
struct ForumCase {
    /// Heard at the gathering (else by a forum).
    gathered: bool,
    accuser: PermanentId,
    accused: PermanentId,
    kcal: f64,
    found: bool,
    stances: Vec<(PermanentId, PermanentId, Stance)>,
}

/// What closing a source would bring a household (M6a slice BA, step four): the household, the
/// food the sickness it believes avoided is worth and the food its longer walks would cost, kcal.
type ClosureForecast = (PermanentId, f64, f64);

impl Forecasts {
    /// What move `m` would bring household `h`, gain units: a store at a levy share, a store
    /// kept, a law against taking with its bundle, a watch, or a curfew (an amendment is
    /// forecast from what was seen, apart).
    pub(super) fn gain(
        &self,
        m: &MoveOption,
        h: PermanentId,
        policies: &[crate::polity::PolicyDef],
        params: &PeopleParams,
    ) -> f64 {
        let pp = &params.polity;
        let Ok(i) = self.outlooks.binary_search_by_key(&h, |o| o.0) else {
            return 0.0;
        };
        let o = &self.outlooks[i].1;
        let known = || {
            self.takings
                .binary_search_by_key(&h, |t| t.0)
                .map_or_else(|_| Default::default(), |k| self.takings[k].1)
        };
        match policies.get(usize::from(m.policy)).map(|d| d.kind) {
            Some(PolicyKind::KeepStore) => crate::polity::kept_gain(o, self.kept, pp),
            Some(PolicyKind::AgainstTaking) => {
                let (recover, owe) =
                    known().under(&m.sanction, o.year_need / 365.0, params.crime.exile_days);
                crate::polity::against_gain(o, recover, owe, pp)
            }
            // A watcher guards what the others leave unguarded (M4c slice AI): one more beside
            // `n` others adds `g(1 - g)^n` of the guard `g` one alone gives. The others are those
            // who keep it now but the one it names and the one whose law it would end.
            Some(PolicyKind::KeepWatch) => {
                let g = params.crime.watch_guard.clamp(0.0, 1.0);
                let others = self
                    .watchers
                    .iter()
                    .filter(|w| Some(w.0) != m.nominee && Some(w.1) != m.ends)
                    .count();
                let (recover, owe) = known().watched(g * (1.0 - g).powi(others as i32));
                crate::polity::against_gain(o, recover, owe, pp)
            }
            // A curfew (M4b slice AD): as a watch, at its own share, less what keeping its grown
            // members at home in its hours costs it.
            Some(PolicyKind::Curfew) => {
                let (recover, owe) = known().watched(params.crime.curfew_guard);
                let home = params.crime.curfew_cost_days
                    * f64::from(crate::polity::hours_long(m.hours))
                    * f64::from(self.grown.get(&h).copied().unwrap_or(0))
                    * o.year_need
                    / 365.0;
                crate::polity::against_gain(o, recover, owe + home, pp)
            }
            // A claim (M5c slice AT): what the household believes outsiders take at the places
            // it names that the claim would keep for it; it costs nothing to make.
            Some(PolicyKind::ClaimPlace) => {
                let keep = self
                    .claim
                    .binary_search_by_key(&h, |c| c.0)
                    .map_or(0.0, |k| self.claim[k].1);
                crate::polity::against_gain(o, keep, 0.0, pp)
            }
            // A crossing built together (M5c slice AW, step three): the food a year the hours of
            // walking it would save the household could bring, less what its share of the work,
            // spread over the crossing's life, could (research 11-07 §2.3: foregone production).
            Some(PolicyKind::BuildCrossing) => {
                let Some(p) = m.site.and_then(|cell| {
                    self.sites
                        .binary_search_by_key(&cell, |p| p.site.cell)
                        .ok()
                        .map(|i| &self.sites[i])
                }) else {
                    return 0.0;
                };
                let kcal = self
                    .hour_kcal
                    .binary_search_by_key(&h, |k| k.0)
                    .map_or(0.0, |i| self.hour_kcal[i].1);
                let saved = p.wades_of(h) * p.hours_a_wade * kcal;
                let owed = p.share_h() / p.life.max(1e-9) * kcal;
                crate::polity::kept_gain(o, saved, pp) - crate::polity::levy_cost(o, owed, pp)
            }
            // A well dug together (M6a slice BA, step four), as a crossing: the food a year the
            // hours of walking it would save the household could bring, less what its share of
            // the work, spread over the lining's life, could.
            Some(PolicyKind::DigWell) => {
                let Some(p) = self.well.as_ref().filter(|p| m.site == Some(p.site.cell)) else {
                    return 0.0;
                };
                let kcal = self
                    .hour_kcal
                    .binary_search_by_key(&h, |k| k.0)
                    .map_or(0.0, |i| self.hour_kcal[i].1);
                let saved = p.saves_of(h) * kcal;
                let owed = p.share_h() / p.life.max(1e-9) * kcal;
                crate::polity::kept_gain(o, saved, pp) - crate::polity::levy_cost(o, owed, pp)
            }
            // A flood bank built together (M6b slice BG), as a crossing: the food a year the
            // losses it remembers at the places the bank would keep dry could bring, less what
            // its share of the work, spread over the bank's life, could. A household that
            // remembers no such loss gains nothing and pays its share (research 12-08 §1.2).
            Some(PolicyKind::BuildBank) => {
                let Some(p) = self.bank.as_ref() else {
                    return 0.0;
                };
                let kcal = self
                    .hour_kcal
                    .binary_search_by_key(&h, |k| k.0)
                    .map_or(0.0, |i| self.hour_kcal[i].1);
                let saved = p.saves_of(h) * kcal;
                let owed = p.share_h() / p.life.max(1e-9) * kcal;
                crate::polity::kept_gain(o, saved, pp) - crate::polity::levy_cost(o, owed, pp)
            }
            // A bank kept (M6b slice BG), as a bank built: the food a year the losses the
            // household reckoned the bank keeps out could bring, less what its share of mending
            // it now, spread over a bank's life, could (design prior: a mended bank is reckoned
            // as good as new). A household that reckoned it saved nothing pays its share.
            Some(PolicyKind::KeepBank) => {
                let Some(p) = self.worn.as_ref().filter(|p| m.site == Some(p.cell_at)) else {
                    return 0.0;
                };
                let kcal = self
                    .hour_kcal
                    .binary_search_by_key(&h, |k| k.0)
                    .map_or(0.0, |i| self.hour_kcal[i].1);
                let saved = p.saves_of(h) * kcal;
                let owed = p.share_h() / p.life.max(1e-9) * kcal;
                crate::polity::kept_gain(o, saved, pp) - crate::polity::levy_cost(o, owed, pp)
            }
            // A source closed (M6a slice BA, step four): what the sickness the household believes
            // it brings would cost, less what walking further would (ADR-0021 §7: today's
            // units), both over the days its people's tallies speak of.
            Some(PolicyKind::CloseSource) => {
                let Some((avoided, walked)) = m.site.and_then(|cell| {
                    let i = self.closures.binary_search_by_key(&cell, |c| c.0).ok()?;
                    let of = &self.closures[i].1;
                    let k = of.binary_search_by_key(&h, |x| x.0).ok()?;
                    Some((of[k].1, of[k].2))
                }) else {
                    return 0.0;
                };
                crate::polity::kept_gain(o, avoided, pp) - crate::polity::levy_cost(o, walked, pp)
            }
            // A rule for building (M6b slice BH): the fire loss the household believes it
            // averts, less what its next building would cost more under it.
            Some(PolicyKind::BuildingRule) => {
                let Some((averted, owed)) = self
                    .rules
                    .iter()
                    .find(|r| r.0 == m.policy)
                    .and_then(|r| r.1.iter().find(|x| x.0 == h))
                    .map(|x| (x.1, x.2))
                else {
                    return 0.0;
                };
                crate::polity::kept_gain(o, averted, pp) - crate::polity::levy_cost(o, owed, pp)
            }
            // A band sent by law (M6c slice BJ): what the household lost to outsiders, to take
            // back, and a load if it has someone grown to send, less the days of its food the
            // one it sends is away and at risk (`bands.service_days`).
            Some(PolicyKind::SendBand) => {
                let k = known();
                let sends = self.grown.get(&h).copied().unwrap_or(0) > 0;
                let load = if sends { self.band_load } else { 0.0 };
                let away = if sends {
                    params.bands.service_days * o.year_need / 365.0
                } else {
                    0.0
                };
                crate::polity::against_gain(o, k.outside_kcal + load, away, pp)
            }
            // A law against raiding (M6c slice BJ): the raids on it the household believes its
            // own people's raids bring (`bands.retaliation_share` of what outsiders took from
            // it), less what its own members took from other settlements, which it would answer
            // for.
            Some(PolicyKind::AgainstRaiding) => {
                let k = known();
                crate::polity::against_gain(
                    o,
                    params.bands.retaliation_share * k.outside_kcal,
                    k.took_outside_kcal,
                    pp,
                )
            }
            // A palisade raised together (M6c slice BK): the food a year the household believes
            // the line keeps from people of other settlements while its home is inside, less what
            // its share of the work, spread over the posts' life, and its people's walk round to
            // a gate could bring. A home outside the line pays its share without its cover.
            // Taking a held settlement in (M6c slice BM): each household's share of the held
            // polity's store, once, as a gift to the store is weighed.
            Some(PolicyKind::Annex) => crate::polity::kept_gain(o, self.annex_share, pp),
            Some(PolicyKind::Enclose) => {
                let Some(p) = self.enclosure.as_ref() else {
                    return 0.0;
                };
                let kcal = self
                    .hour_kcal
                    .binary_search_by_key(&h, |k| k.0)
                    .map_or(0.0, |i| self.hour_kcal[i].1);
                let (kept, detour) = p.keeps_of(h);
                let owed = (p.share_h() / p.life.max(1e-9) + detour) * kcal;
                crate::polity::kept_gain(o, kept, pp) - crate::polity::levy_cost(o, owed, pp)
            }
            // A forum (M7b slice BR): the household's own cases the gathering heard within
            // memory, heard again as the forum would hear them (a holder as they stood there, or
            // at even odds if they did not come; a panel or the elders as those who came stood),
            // the difference in what it got back or paid, a year; and the evenings its people
            // spent hearing cases, which they would keep.
            Some(PolicyKind::HearCases) => {
                let Some(def) = policies
                    .get(usize::from(m.policy))
                    .and_then(|d| d.forum.as_ref())
                else {
                    return 0.0;
                };
                let (mut net, mut evenings) = (0.0, 0usize);
                for c in &self.forum_cases {
                    if c.gathered {
                        evenings += c.stances.iter().filter(|r| r.1 == h).count();
                    }
                    let side = if c.accuser == h {
                        1.0
                    } else if c.accused == h {
                        -1.0
                    } else {
                        continue;
                    };
                    let n = |s: Stance| c.stances.iter().filter(|r| r.2 == s).count() as f64;
                    let (support, oppose) = (n(Stance::Support), n(Stance::Oppose));
                    let chance = match def.seat {
                        crate::forum::ForumSeat::Holder => {
                            match m.nominee.and_then(|p| c.stances.iter().find(|r| r.0 == p)) {
                                Some((_, _, Stance::Support)) => 1.0,
                                Some((_, _, Stance::Oppose)) => 0.0,
                                _ => 0.5,
                            }
                        }
                        _ if support + oppose > 0.0 => support / (support + oppose),
                        _ => 0.5,
                    };
                    net += side * (chance - f64::from(u8::from(c.found))) * c.kcal;
                }
                net *= 365.0 / f64::from(params.crime.remember_days.max(1));
                let kept = evenings as f64 * params.elections.w_evening / pp.w_gain.max(1e-9);
                kept + if net >= 0.0 {
                    crate::polity::kept_gain(o, net, pp)
                } else {
                    -crate::polity::levy_cost(o, -net, pp)
                }
            }
            // A duty or an embargo (M7c slice BV): what it brings back less what it costs, a year.
            Some(PolicyKind::Duty) => {
                let Some(d) = policies.get(usize::from(m.policy)).and_then(|d| d.duty) else {
                    return 0.0;
                };
                let (gain, cost) = self.trade.duty(
                    h,
                    m.levy_share,
                    d.purpose,
                    d.exempt_sellers,
                    params.market.margin,
                );
                crate::polity::kept_gain(o, gain, pp) - crate::polity::levy_cost(o, cost, pp)
            }
            Some(PolicyKind::Embargo) => {
                let Some(at) = m.target else {
                    return 0.0;
                };
                let (gain, cost) = self.trade.embargo(h, at, params.market.margin);
                crate::polity::kept_gain(o, gain, pp) - crate::polity::levy_cost(o, cost, pp)
            }
            Some(PolicyKind::Recognize) => {
                let Some(org) = m.target else {
                    return 0.0;
                };
                let (gain, cost) = self.trade.recognize(h, org);
                crate::polity::kept_gain(o, gain, pp) - crate::polity::levy_cost(o, cost, pp)
            }
            // Debts heard (M7d slice CB): what it is owed and owes on failed loans, at the share
            // a finding is expected to move.
            Some(PolicyKind::DebtsHeard) => {
                let Some(def) = policies.get(usize::from(m.policy)).and_then(|d| d.debts) else {
                    return 0.0;
                };
                let share = f64::from(def.share);
                let (to, from) = self
                    .debts
                    .binary_search_by_key(&h, |d| d.0)
                    .map_or((0.0, 0.0), |i| (self.debts[i].1, self.debts[i].2));
                crate::polity::against_gain(o, to * share, from * share, pp)
            }
            // A mint (M7d slice CE2b): the hours of weighing its coin would save the household on
            // the metal it pays and is paid a year, worth what its hours bring in food; less, for
            // the household of the keeper it names, the hours of striking as much metal as
            // changes hands in the settlement a year.
            Some(PolicyKind::Mint) => {
                let Some(&(_, saved_h, strike_h)) = self.mints.iter().find(|x| x.0 == m.policy)
                else {
                    return 0.0;
                };
                let kcal = self
                    .hour_kcal
                    .binary_search_by_key(&h, |k| k.0)
                    .map_or(0.0, |i| self.hour_kcal[i].1);
                let kg = self
                    .weighed
                    .binary_search_by_key(&h, |k| k.0)
                    .map_or(0.0, |i| self.weighed[i].1);
                let keeper = m
                    .nominee
                    .and_then(|n| self.adults.binary_search_by_key(&n, |a| a.0).ok())
                    .map(|i| self.adults[i].1);
                let work = if keeper == Some(h) {
                    self.weighed.iter().map(|w| w.1).sum::<f64>() / 2.0 * strike_h * kcal
                } else {
                    0.0
                };
                crate::polity::kept_gain(o, kg * saved_h * kcal, pp)
                    - crate::polity::levy_cost(o, work, pp)
            }
            // Pay for common work (M7d slice CE3a): the household's share of the money the store
            // holds, as it values it.
            Some(PolicyKind::PayWork) => {
                let kcal = self
                    .pay
                    .iter()
                    .find(|p| p.0 == m.policy)
                    .and_then(|p| p.1.iter().find(|x| x.0 == h))
                    .map_or(0.0, |x| x.1);
                crate::polity::kept_gain(o, kcal, pp)
            }
            // Lending from the store (M7d slice CC): what its short year would borrow, up to its
            // share of what the store lends, less the increment it would owe.
            Some(PolicyKind::LendFromStore) => {
                let Some(def) = policies.get(usize::from(m.policy)).and_then(|d| d.lending) else {
                    return 0.0;
                };
                let (food, n) = self.store_food;
                let each = food * f64::from(def.share) / n.max(1) as f64;
                let short = (o.year_need - o.held - o.harvest).max(0.0);
                let borrow = short.min(each);
                crate::polity::against_gain(o, borrow, borrow * f64::from(def.increment), pp)
            }
            Some(PolicyKind::Privilege) => {
                let (Some(org), Some(def)) = (
                    m.target,
                    policies
                        .get(usize::from(m.policy))
                        .and_then(|d| d.privilege),
                ) else {
                    return 0.0;
                };
                let (gain, cost) = self.trade.privilege(h, org, def, params.market.margin);
                crate::polity::kept_gain(o, gain, pp) - crate::polity::levy_cost(o, cost, pp)
            }
            // An amendment's gain is judged from decisions seen, and an end to a law from that
            // law, kept (M4c slices AF, AI): neither forecasts anything of its own.
            Some(PolicyKind::AmendBody | PolicyKind::Repeal) => 0.0,
            _ => crate::polity::store_gain(o, &self.belief, m.levy_share, pp),
        }
    }
}

/// Purpose tag for a deliberator's draw among moves.
pub const PURPOSE_DELIBERATE: u64 = 0x6465_6c69_6265_7231; // "deliber1"
/// Purpose tag for the draw of whether someone pays a levy.
pub const PURPOSE_LEVY: u64 = 0x6c65_7679_7061_7931; // "levypay1"

/// The shortest sitting worth walking to: a gathering about to rise is not.
const LEAST_SITTING_MIN: i64 = 15;
/// Days within which people of another settlement seen at a place one's household works make an
/// issue for one's polity (M5c slice AT): a year, as for takings.
const OUTSIDERS_DAYS: i64 = 365;
/// The sites one weighing a crossing built together weighs: those their household wades most
/// (M5c slice AW, step three; research 09-05 §3.3: a few packages per decision, an engineering
/// choice).
const PUBLIC_SITES_WEIGHED: usize = 3;

impl Population {
    /// The index of the polity, still deciding, that holds settlement `settlement`: its seat, or
    /// one it took in (M6c slice BM).
    pub(crate) fn polity_of(&self, settlement: PermanentId) -> Option<usize> {
        self.polities.iter().position(|p| p.holds(Some(settlement)))
    }

    /// The polities' midnight (ADR-0013): every settlement without a polity gets one under the
    /// founding custom; yesterday's gathering decides; those at home hear of the laws their
    /// households know; and a polity due its review weighs its issues.
    pub(super) fn polity_day(&mut self, ctx: &mut Ctx) {
        for i in 0..ctx.land.settlements.len() {
            let s = &ctx.land.settlements[i];
            if self.polity_of(s.id).is_none() {
                let (settlement, founded) = (s.id, s.founded);
                let id = ctx.ids.allocate();
                self.polities
                    .push(Polity::found(id, settlement, founded, &ctx.params.polity));
            }
        }
        let day = ctx.now.day_index();
        // What people worked the day before, and who else worked it (M5c slice AT): folded in
        // before anyone weighs it; households no more and places long unworked let go.
        let half = ctx.params.places.use_half_life_days;
        let meetings = self.uses.fold(half);
        let index = &self.hh_index;
        self.uses
            .prune(day, half, OUTSIDERS_DAYS, |h| index.contains_key(&h));
        // Those who met outsiders at a place their polity claims hold it against them, and views
        // of polities long unfed, or of those gone, are let go (M5c slice AT, step two).
        self.trespass(ctx, &meetings);
        let mut views = std::mem::take(&mut self.polity_views);
        views.prune(day, &ctx.params.relations, |p| self.person(p).is_some());
        self.polity_views = views;
        // Claims heard of that no law in force makes now are let go (M5c slice AU).
        if !self.claims_heard.households.is_empty() {
            self.forget_ended_claims();
        }
        self.lapse_keepers(ctx);
        // Councils' elections, called, stood for, polled, counted and seated (M7a slice BO).
        self.elections_day(ctx);
        for pi in 0..self.polities.len() {
            if self.polities[pi]
                .gathering
                .as_ref()
                .is_some_and(|g| g.day < day)
            {
                self.decide_gathering(ctx, pi);
            }
        }
        // Forums hear the cases brought to them before today, those not settled or withdrawn
        // first (M7b slice BR).
        if self
            .order
            .cases
            .iter()
            .any(|c| c.forum.is_some() && c.stage == crate::crime::CaseStage::Open)
        {
            self.settle_cases(ctx);
            self.forums_day(ctx);
        }
        // Agreements between polities: a gathering's decision on one, a side not yet put to its
        // gathering, and one coming into force, failing or ending (M5c slice AU).
        self.agreements_day(ctx);
        // And the payments they owe, set aside and carried (M5c slice AV).
        if !self.agreements.list.is_empty() {
            self.dues_day(ctx);
        }
        // Cases still waiting go before the next gathering (M4b slice AB).
        if self
            .order
            .cases
            .iter()
            .any(|c| c.stage == crate::crime::CaseStage::Open)
        {
            for pi in 0..self.polities.len() {
                self.put_cases(
                    &ctx.catalog.policies,
                    pi,
                    day,
                    ctx.params.polity.notice_days,
                );
            }
        }
        self.tell_households(day);
        // Anyone new takes what they hold of each value and norm, and of the ideologies a founder
        // brings or a parent holds (M4c slice AG).
        self.take_new_holdings(ctx);
        self.take_ideology_starts(ctx);
        // Households tell their members of gatherings ahead, and those short of food who find
        // the common store empty hold it against it (M4c slice AE).
        self.word_day(ctx);
        // Those who hold news of sickness tally the sources they know (M6a slice BA).
        self.suspicion_day(ctx);
        self.grieve_empty_stores(ctx);
        // Petitions that have sat go before the gathering, and those decided are answered; then
        // those whose day it is review where they belong (M4c slice AH).
        self.petitions_day(ctx);
        self.revolts_day(ctx);
        self.coups_day(ctx);
        self.factions_day(ctx);
        for pi in 0..self.polities.len() {
            let p = &self.polities[pi];
            // One taken in decides no more (M6c slice BM).
            if p.ended.is_none()
                && day - p.reviewed >= i64::from(ctx.params.polity.review_days.max(1))
            {
                self.polities[pi].reviewed = day;
                self.review(ctx, pi);
                // Whether anyone bound by a law a limit forbids challenges it (M7b slice BT).
                self.weigh_challenges(ctx, pi);
                // Whether a member of an organization brings a breach of its privilege (M7c
                // slice BX).
                self.weigh_breaches(ctx, pi);
                // Whether a creditor brings a loan refused or left unpaid (M7d slice CB).
                self.weigh_debts(ctx, pi);
                // Whether the store's keeper draws from it for their own household (M7c slice
                // BY).
                self.keeper_weighs_drawing(ctx, pi);
                self.forget_patron_gifts(ctx, pi);
                // Terms with another polity are sought when nothing was put to the gathering
                // (M5c slice AU).
                self.seek_terms(ctx, pi);
            }
        }
    }

    /// Who may not be named or drawn for office template `policy` of polity `pi` on `day` (M7a
    /// slice BN, ADR-0024 §2): the holder of its last term, once that term has ended, where the
    /// template does not let them serve again.
    fn term_barred(
        &self,
        pi: usize,
        policy: u16,
        day: i64,
        def: &crate::polity::PolicyDef,
    ) -> Option<PermanentId> {
        if def.tenure.is_none_or(|t| t.renewable) {
            return None;
        }
        let p = &self.polities[pi];
        let (term, holder) = p
            .laws
            .iter()
            .rev()
            .filter(|l| l.policy == policy)
            .find_map(|l| Some((p.term_of(l.id)?, l.holder?)))?;
        (term.until <= day).then_some(holder)
    }

    /// Office law `law_id` of polity `pi` passed, and its template holds it for a term (M7a slice
    /// BN, ADR-0024 §2): its term runs from today. Filled by lot, those who stood for it at the
    /// gathering are those who put their names in (a design prior: wanting the office kept is
    /// taken as willingness to keep it), less any its last term bars and any keeping the watch
    /// already; a lot keyed by the law draws one. With nobody to draw, the law lapses and the
    /// chronicle says so.
    fn fill_term(&mut self, ctx: &mut Ctx, pi: usize, law_id: PermanentId) {
        let day = ctx.now.day_index();
        let Some(law) = self.polities[pi].laws.iter().find(|l| l.id == law_id) else {
            return;
        };
        let def = &ctx.catalog.policies[usize::from(law.policy)];
        let Some(tenure) = def.tenure else {
            return;
        };
        let mut pool = 0;
        if tenure.by_lot {
            let barred = self.term_barred(pi, law.policy, day, def);
            let watching: Vec<PermanentId> = self.polities[pi].watchers().map(|w| w.0).collect();
            let names: Vec<PermanentId> = law
                .stances
                .iter()
                .filter(|r| r.stance == crate::polity::Stance::Support)
                .map(|r| r.person)
                .filter(|&p| Some(p) != barred && !watching.contains(&p))
                .filter(|&p| self.person(p).is_some())
                .collect();
            pool = names.len() as u32;
            let key = Rng64::from_key(&[ctx.seed, law_id.get()]).next_u64();
            let drawn = crate::count::draw(&names, 1, key);
            let settlement = self.polities[pi].settlement;
            let (place, name) = ctx
                .land
                .settlements
                .iter()
                .find(|s| s.id == settlement)
                .map_or((None, String::new()), |s| {
                    (Some(s.hearth_m), s.name.clone())
                });
            let office = def.name.to_lowercase();
            match drawn.first() {
                Some(&h) => {
                    if let Some(l) = self.polities[pi].law_mut(law_id) {
                        l.holder = Some(h);
                    }
                    let words = format!(
                        "The lot fell to {}, of {pool} who put their names in, for the office of \
                         {office} at {name}.",
                        self.name_of(h)
                    );
                    self.chronicle_push(
                        ctx.now,
                        ChronicleKind::LotDrawn,
                        vec![h],
                        Some(settlement),
                        place,
                        f64::from(pool),
                        words,
                    );
                }
                None => {
                    if let Some(l) = self.polities[pi].law_mut(law_id) {
                        l.status = LawStatus::Lapsed;
                    }
                    let words = format!(
                        "Nobody put their name in for the office of {office} at {name}: the lot \
                         drew no one."
                    );
                    self.chronicle_push(
                        ctx.now,
                        ChronicleKind::LawLapsed,
                        Vec::new(),
                        Some(settlement),
                        place,
                        0.0,
                        words,
                    );
                    return;
                }
            }
        }
        let until = day + i64::from(tenure.term_days);
        self.polities[pi].set_term(law_id, crate::polity::Term { until, pool });
    }

    /// For tests: office law `law_id` of polity `pi`, passed, has its term filled.
    pub fn fill_term_for_tests(&mut self, ctx: &mut Ctx, pi: usize, law_id: PermanentId) {
        self.fill_term(ctx, pi, law_id);
    }

    /// A law that names someone lapses when they die or leave the settlement: the store they
    /// kept lies unkept again, or the watch they kept is unkept, which is a vacancy the next
    /// review may answer (succession, ADR-0013 §2). Nothing fills it by itself.
    fn lapse_keepers(&mut self, ctx: &mut Ctx) {
        for pi in 0..self.polities.len() {
            let settlement = self.polities[pi].settlement;
            let offices: Vec<(PermanentId, PermanentId, PolicyKind)> = self.polities[pi]
                .laws
                .iter()
                .filter(|l| l.status == LawStatus::InForce)
                .filter_map(|l| Some((l.holder?, l.id, l.kind)))
                .collect();
            for (holder, law, kind) in offices {
                self.lapse_office(ctx, pi, settlement, holder, law, kind);
            }
        }
    }

    /// Law `law` of polity `pi` names `holder` to an office of kind `kind`: it lapses if they no
    /// longer live in the settlement.
    fn lapse_office(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        settlement: PermanentId,
        holder: PermanentId,
        law: PermanentId,
        kind: PolicyKind,
    ) {
        {
            let here = self
                .person(holder)
                .and_then(|p| self.household(p.household))
                .is_some_and(|x| self.polities[pi].holds(x.settlement));
            // An office held for a term lapses at its end (M7a slice BN).
            let term_ended = self.polities[pi]
                .term_of(law)
                .is_some_and(|t| ctx.now.day_index() >= t.until);
            if here && !term_ended {
                return;
            }
            if kind == PolicyKind::KeepStore {
                let sheltered = self.polity_sheltered(pi);
                self.polities[pi].settle_stores(ctx.now, &ctx.catalog.goods, sheltered);
            }
            if let Some(l) = self.polities[pi].law_mut(law) {
                l.status = LawStatus::Lapsed;
            }
            let why = match self.records.get(&holder) {
                _ if here && term_ended => "their term ended",
                Some(r) if r.died.is_some() => "they died",
                Some(r) if r.left.is_some() => "they left the valley",
                _ => "they no longer live there",
            };
            let (place, name) = ctx
                .land
                .settlements
                .iter()
                .find(|s| s.id == settlement)
                .map_or((None, String::new()), |s| {
                    (Some(s.hearth_m), s.name.clone())
                });
            let office = match kind {
                PolicyKind::KeepWatch => "keeps watch",
                PolicyKind::HearCases => "hears cases",
                PolicyKind::Mint => "strikes coin",
                _ => "keeps the common store",
            };
            let words = format!(
                "{} no longer {office} at {name}: {why}.",
                self.name_of(holder)
            );
            self.chronicle_push(
                ctx.now,
                ChronicleKind::LawLapsed,
                vec![holder],
                Some(settlement),
                place,
                0.0,
                words,
            );
        }
    }

    /// The adults living in any settlement polity `pi` holds, with their households, in id order:
    /// its members (ADR-0023 §9: membership is residence in any of them).
    pub(super) fn members_of_polity(
        &self,
        pi: usize,
        now: SimTime,
        params: &PeopleParams,
    ) -> Vec<(PermanentId, PermanentId)> {
        let polity = &self.polities[pi];
        let mut out: Vec<(PermanentId, PermanentId)> = self
            .people
            .iter()
            .filter(|(_, p)| p.age_years(now) >= params.family.independent_age)
            .filter(|(_, p)| {
                self.household(p.household)
                    .is_some_and(|x| polity.holds(x.settlement))
            })
            .map(|(_, p)| (p.id, p.household))
            .collect();
        out.sort_unstable();
        out
    }

    /// The adults living in settlement `settlement`, with their households, in id order: the
    /// gathering's members where a polity holds one settlement (ADR-0013 §1: membership is
    /// residence).
    pub(super) fn members_of(
        &self,
        settlement: PermanentId,
        now: SimTime,
        params: &PeopleParams,
    ) -> Vec<(PermanentId, PermanentId)> {
        let mut out: Vec<(PermanentId, PermanentId)> = self
            .people
            .iter()
            .filter(|(_, p)| p.age_years(now) >= params.family.independent_age)
            .filter(|(_, p)| {
                self.household(p.household)
                    .is_some_and(|x| x.settlement == Some(settlement))
            })
            .map(|(_, p)| (p.id, p.household))
            .collect();
        out.sort_unstable();
        out
    }

    /// Of `adults` (adults and their households, in id order), those a body of membership `m`
    /// admits (M4c slice AF): all of them; each household's elder; or the adults of households
    /// holding a field among `fields`.
    pub(super) fn admitted(
        &self,
        m: Membership,
        adults: &[(PermanentId, PermanentId)],
        fields: &[civ_land::fields::Field],
        now: SimTime,
        params: &PeopleParams,
    ) -> Vec<(PermanentId, PermanentId)> {
        match m {
            Membership::Adults => adults.to_vec(),
            Membership::Elders => adults
                .iter()
                .copied()
                .filter(|&(p, h)| self.elder_of(h, now, params) == Some(p))
                .collect(),
            Membership::Landholders => {
                let mut holding: Vec<PermanentId> =
                    fields.iter().filter_map(|f| f.holder.household()).collect();
                holding.sort_unstable();
                holding.dedup();
                adults
                    .iter()
                    .copied()
                    .filter(|(_, h)| holding.binary_search(h).is_ok())
                    .collect()
            }
            // Those of them who keep a watch (M4c slice AI): a watcher keeps it where they live.
            Membership::Watch => adults
                .iter()
                .copied()
                .filter(|&(p, _)| self.keeps_a_watch(p))
                .collect(),
            // Those of them who hold a council seat (M7a slice BO): one sits where they live.
            Membership::Seats => self.sitting(adults),
            // Members of organizations it recognizes, or grants a privilege (M7c slice BZ).
            Membership::Organized | Membership::Privileged => adults
                .iter()
                .copied()
                .filter(|&(_, h)| self.closed_member(m, h))
                .collect(),
        }
    }

    /// The members of polity `pi`'s body now, with their households, in id order (ADR-0013 §2;
    /// M4c slice AF): the adults it admits under its custom.
    pub fn body_members(
        &self,
        fields: &[civ_land::fields::Field],
        pi: usize,
        now: SimTime,
        params: &PeopleParams,
    ) -> Vec<(PermanentId, PermanentId)> {
        let adults = self.members_of_polity(pi, now, params);
        self.admitted(self.polities[pi].body.members, &adults, fields, now, params)
    }

    /// Whether adult `person` of household `hh` belongs to polity `pi`'s body (M4c slice AF).
    fn admits(
        &self,
        fields: &[civ_land::fields::Field],
        pi: usize,
        person: PermanentId,
        hh: PermanentId,
        now: SimTime,
        params: &PeopleParams,
    ) -> bool {
        match self.polities[pi].body.members {
            Membership::Adults => true,
            Membership::Elders => self.elder_of(hh, now, params) == Some(person),
            Membership::Landholders => fields.iter().any(|f| f.holder.household() == Some(hh)),
            Membership::Watch => self.keeps_a_watch(person),
            Membership::Seats => self.sits(person),
            m @ (Membership::Organized | Membership::Privileged) => self.closed_member(m, hh),
        }
    }

    /// Whether `person` keeps a watch by a law in force (M4c slice AI).
    pub(super) fn keeps_a_watch(&self, person: PermanentId) -> bool {
        self.polities
            .iter()
            .any(|p| p.watchers().any(|(h, _)| h == person))
    }

    /// What household `household` expects of its coming year: the food it holds, an ordinary
    /// harvest of its fields as their records say (less the seed), and a year's need.
    pub(super) fn outlook(&self, ctx: &Ctx, household: PermanentId) -> Option<Outlook> {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let x = self.household(household)?;
        let need_day = x.members.len() as f64 * params.household.daily_kcal_per_person;
        let held = stock_kcal(&stores_now(x, now, params, goods), goods);
        let harvest = ctx.catalog.crops.get(params.farm.crop).map_or(0.0, |crop| {
            let fields = || self.fields_of(ctx.land, household);
            let area: f64 = fields().map(Field::area_ha).sum();
            if area <= 0.0 {
                return 0.0;
            }
            let per_ha = farm::expected_yield_kg_ha(fields(), crop, now.day_index());
            let kcal = goods.get(crop.good).map_or(0.0, |g| g.kcal_per_kg);
            (per_ha - crop.seed_kg_per_ha).max(0.0) * area * kcal
        });
        Some(Outlook {
            held,
            harvest,
            year_need: need_day * 365.0,
        })
    }

    /// What settlement `settlement`'s people believe of their years: lean years are those its
    /// food ran short in (the chronicle's), over the years since it was founded, with the prior;
    /// its households' ordinary harvest is the mean of `outlooks`.
    fn belief(&self, ctx: &Ctx, settlement: PermanentId, outlooks: &[Outlook]) -> Belief {
        let mut lean: Vec<i32> = self
            .chronicle
            .iter()
            .filter(|e| e.kind == ChronicleKind::FoodRanShort && e.settlement == Some(settlement))
            .map(|e| civ_land::calendar_year(e.at.day_index()))
            .collect();
        lean.dedup();
        let founded = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map_or(ctx.now, |s| s.founded);
        let years = (ctx.now.day_index() - founded.day_index()).max(0) as f64 / 365.0;
        let mean_harvest = if outlooks.is_empty() {
            0.0
        } else {
            outlooks.iter().map(|o| o.harvest).sum::<f64>() / outlooks.len() as f64
        };
        Belief {
            p_lean: crate::polity::p_lean(lean.len() as f64, years, &ctx.params.polity),
            mean_harvest,
        }
    }

    /// Whether household `household`'s food will not see it through to its next harvest: the
    /// shortfall against the outlook that reaches its people (ADR-0013 §5).
    fn pressed(&self, ctx: &Ctx, household: PermanentId) -> bool {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(x) = self.household(household) else {
            return false;
        };
        let need = x.members.len() as f64 * params.household.daily_kcal_per_person;
        let days = stock_kcal(&stores_now(x, now, params, goods), goods) / need.max(1.0);
        let to_harvest = ctx.catalog.crops.get(params.farm.crop).map_or(0.0, |crop| {
            farm::days_to_harvest(crop, self.fields_of(ctx.land, household), now.day_index())
        });
        days < to_harvest || days < params.household.short_food_days
    }

    /// Whether settlement `settlement`'s food is short now or ran short within the last year: a
    /// lean year, as its people have seen it (the chronicle's, ADR-0013 §5).
    pub(super) fn lean_lately(&self, ctx: &Ctx, settlement: PermanentId) -> bool {
        let short_now = ctx
            .land
            .settlements
            .iter()
            .any(|s| s.id == settlement && s.food_short);
        let since = ctx.now.day_index() - 365;
        short_now
            || self.chronicle.iter().rev().any(|e| {
                e.kind == ChronicleKind::FoodRanShort
                    && e.settlement == Some(settlement)
                    && e.at.day_index() >= since
            })
    }

    /// The issues settlement `settlement` faces now (research 09-05 §1.1): its food ran short
    /// within the last year, or a household's food will not last until its next harvest
    /// (`pressed`); its common store holds food nobody keeps under a roof (`unkept`); a household
    /// found food taken from its store within the last year (`takings`); and its people saw
    /// people of another settlement working places theirs work, unclaimed (`outsiders`, M5c
    /// slice AT); a household's people saw their own sick lately (`sickness`), and someone holds
    /// that a source it does not close sickens (`suspected`), and its households walk far enough
    /// for water that a well dug together would repay its work (`water_far`, M6a slice BA, step
    /// four).
    #[allow(clippy::too_many_arguments)]
    fn issues(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        pressed: bool,
        unkept: bool,
        takings: bool,
        founding: bool,
        outsiders: bool,
        fords: bool,
        sickness: bool,
        suspected: bool,
        water_far: bool,
        flooded: bool,
        bank_worn: bool,
        burned: bool,
        gave_way: bool,
        raided: bool,
    ) -> Vec<IssueKind> {
        let mut out = Vec::new();
        if self.lean_lately(ctx, settlement) || pressed {
            out.push(IssueKind::FoodShort);
        }
        if unkept {
            out.push(IssueKind::StoreUnkept);
        }
        if takings {
            out.push(IssueKind::Takings);
        }
        if founding {
            out.push(IssueKind::Founding);
        }
        if outsiders {
            out.push(IssueKind::Outsiders);
        }
        if fords {
            out.push(IssueKind::Fords);
        }
        if sickness {
            out.push(IssueKind::Sickness);
        }
        if suspected {
            out.push(IssueKind::SourceSuspected);
        }
        if water_far {
            out.push(IssueKind::WaterFar);
        }
        if flooded {
            out.push(IssueKind::Flooded);
        }
        if bank_worn {
            out.push(IssueKind::BankWorn);
        }
        if burned {
            out.push(IssueKind::Burned);
        }
        if gave_way {
            out.push(IssueKind::GaveWay);
        }
        if raided {
            out.push(IssueKind::Raided);
        }
        out
    }

    /// Whether household `household`'s people saw one of their own lie sick lately (M6a slice
    /// BA): a member holds the news of its own sickness.
    fn sick_at_home(&self, household: PermanentId) -> bool {
        let Some(x) = self.household(household) else {
            return false;
        };
        x.members.iter().any(|&m| {
            self.word.heard_by(m).iter().any(|h| {
                self.word.claim(h.claim).is_some_and(|c| {
                    c.kind == crate::word::ClaimKind::Sickness && c.subject == Some(household)
                })
            })
        })
    }

    /// The adults among `adults` who hold that a source of water sickens which polity `pi` does
    /// not close now, and the cells of those sources, each in order (M6a slice BA, step four).
    fn suspecting(
        &self,
        pi: usize,
        adults: &[(PermanentId, PermanentId)],
    ) -> (Vec<PermanentId>, Vec<u32>) {
        if self.word.suspicions.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let closed = self.polities[pi].closed_now();
        let (mut who, mut cells) = (Vec::new(), Vec::new());
        for &(a, _) in adults {
            for s in self.word.suspicions_of(a) {
                if let crate::uses::Place::Source(cell) = s.source
                    && closed.binary_search_by_key(&s.source, |c| c.0).is_err()
                {
                    who.push(a);
                    cells.push(cell);
                }
            }
        }
        who.sort_unstable();
        who.dedup();
        cells.sort_unstable();
        cells.dedup();
        (who, cells)
    }

    /// When polity `pi`'s custom was taken from the gathering, while its new body is founding
    /// (M4c slice AI, step two): within `founding_days` of a version seized rather than amended.
    pub(super) fn founding_since(&self, ctx: &Ctx, pi: usize) -> Option<SimTime> {
        let v = self.polities[pi].versions.last()?;
        v.seized_by?;
        let days = ctx.now.day_index() - v.since.day_index();
        (days < i64::from(ctx.params.polity.founding_days)).then_some(v.since)
    }

    /// The laws of polity `pi` the custom that was taken on `since` made and its new body has not
    /// yet weighed (M4c slice AI, step two): in force, decided before then, and neither an
    /// amendment nor a store that levies nothing already; none put to the new body to end it.
    fn inherited(&self, pi: usize, since: SimTime) -> Vec<&Law> {
        let laws = &self.polities[pi].laws;
        laws.iter()
            .filter(|l| l.status == LawStatus::InForce && l.decided.is_none_or(|t| t < since))
            .filter(|l| !matches!(l.kind, PolicyKind::AmendBody | PolicyKind::Repeal))
            .filter(|l| l.kind != PolicyKind::CommonStore || l.levy_share > 0.0)
            .filter(|l| {
                !laws
                    .iter()
                    .any(|x| x.ends == Some(l.id) && x.issue == IssueKind::Founding)
            })
            .collect()
    }

    /// Food polity `pi`'s store would keep over a year under a roof rather than in the open, kcal.
    fn roof_saves(&self, ctx: &Ctx, pi: usize) -> f64 {
        let goods = &ctx.catalog.goods;
        let left = |half: f64| {
            if half > 0.0 {
                0.5f64.powf(365.0 / half)
            } else {
                1.0
            }
        };
        self.polities[pi]
            .stores
            .iter()
            .zip(goods)
            .filter(|(_, g)| g.purpose == GoodUse::Food && !g.kept_back())
            .map(|(kg, g)| {
                let roofed = if g.sheltered_half_life_days > 0.0 {
                    g.sheltered_half_life_days
                } else {
                    g.half_life_days
                };
                kg.max(0.0) * g.kcal_per_kg * (left(roofed) - left(g.half_life_days)).max(0.0)
            })
            .sum()
    }

    /// Whether polity `pi`'s store is kept under a roof now: its keeper's household is under one.
    pub(crate) fn polity_sheltered(&self, pi: usize) -> bool {
        self.polities[pi]
            .keeper()
            .and_then(|(k, _)| self.person(k))
            .and_then(|p| self.household(p.household))
            .is_some_and(|x| x.sheltered)
    }

    /// What the households `households` of polity `pi`'s settlement expect, to forecast what a
    /// move would bring each (ADR-0013 §5): their outlooks, the settlement's belief about its
    /// years, what a roof over the store would keep each, what each knows of takings in the last
    /// year (M4b slice AB), and how many grown members each has among `adults`.
    pub(super) fn forecasts(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
        adults: &[(PermanentId, PermanentId)],
    ) -> Forecasts {
        let settlement = self.polities[pi].settlement;
        let sites = self.public_sites(ctx, settlement, households);
        let well = self.public_well(ctx, settlement, households);
        let bank = self.public_bank(ctx, settlement, households);
        let mut fc = self.forecasts_with(ctx, pi, households, adults, sites, well, bank);
        self.with_enclosure(ctx, &mut fc, settlement, households);
        fc
    }

    /// Adds to forecasts `fc` the palisade settlement `settlement`'s people could raise round
    /// their hearth (M6c slice BK), with what an hour of each household's work brings it if it
    /// was not reckoned already.
    fn with_enclosure(
        &self,
        ctx: &Ctx,
        fc: &mut Forecasts,
        settlement: PermanentId,
        households: &[PermanentId],
    ) {
        fc.enclosure = self.public_enclosure(ctx, settlement, households);
        if fc.enclosure.is_some() && fc.hour_kcal.is_empty() {
            fc.hour_kcal = households
                .iter()
                .filter_map(|&h| Some((h, self.hour_kcal(ctx, self.household(h)?))))
                .collect();
        }
    }

    /// [`Population::forecasts`], with the crossing sites already found.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn forecasts_with(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
        adults: &[(PermanentId, PermanentId)],
        sites: Vec<super::crossings::PublicSite>,
        well: Option<super::wells::PublicWell>,
        bank: Option<super::levees::PublicBank>,
    ) -> Forecasts {
        let outlooks: Vec<(PermanentId, Outlook)> = households
            .iter()
            .filter_map(|&h| Some((h, self.outlook(ctx, h)?)))
            .collect();
        let just: Vec<Outlook> = outlooks.iter().map(|o| o.1).collect();
        let belief = self.belief(ctx, self.polities[pi].settlement, &just);
        let kept = self.roof_saves(ctx, pi) / outlooks.len().max(1) as f64;
        let takings = households
            .iter()
            .map(|&h| (h, self.takings_known(h, ctx.now, ctx.params)))
            .collect();
        let mut grown: BTreeMap<PermanentId, u32> = BTreeMap::new();
        for &(_, h) in adults {
            *grown.entry(h).or_default() += 1;
        }
        let watchers = self.polities[pi]
            .watchers()
            .map(|(h, l)| (h, l.id))
            .collect();
        let places = self.unclaimed_outsider_places(ctx, pi, households);
        let claim = households
            .iter()
            .map(|&h| {
                let take = self.outsiders_take(ctx, h, &places);
                (h, take * ctx.params.polity.claim_keeps)
            })
            .collect();
        let closures = self.closure_forecasts(ctx, pi, households, adults);
        // A mint's forecast (M7d slice CE2b): what each template would save weighing, and what
        // striking would ask of its keeper.
        let mints: Vec<(u16, f64, f64)> = ctx
            .catalog
            .policies
            .iter()
            .enumerate()
            .filter_map(|(k, d)| {
                let m = d.mint?;
                let r = ctx.catalog.recipes.get(m.recipe)?;
                let per = r.inputs.first()?.1;
                let strike_h = if per > 0.0 { r.unit_h / per } else { 0.0 };
                Some((k as u16, Self::weighing_saved_h(ctx, &m), strike_h))
            })
            .collect();
        let weighed = if mints.is_empty() {
            Vec::new()
        } else {
            self.weighed_kg_year(ctx, households)
        };
        let weighing = weighed.iter().any(|w| w.1 > 0.0);
        let worn = self.worn_bank(ctx, pi, households.len());
        let rules = self.rule_forecasts(ctx, pi, households);
        Forecasts {
            outlooks,
            belief,
            kept,
            takings,
            grown,
            watchers,
            claim,
            closures,
            hour_kcal: if sites.is_empty()
                && well.is_none()
                && bank.is_none()
                && worn.is_none()
                && !weighing
            {
                Vec::new()
            } else {
                households
                    .iter()
                    .filter_map(|&h| Some((h, self.hour_kcal(ctx, self.household(h)?))))
                    .collect()
            },
            sites,
            well,
            bank,
            worn,
            enclosure: None,
            rules,
            band_load: {
                let density = ctx
                    .catalog
                    .goods
                    .iter()
                    .filter(|g| g.purpose == GoodUse::Food && !g.kept_back())
                    .map(|g| g.kcal_per_kg)
                    .fold(0.0, f64::max);
                ctx.params.household.carry_kg * density * ctx.params.bands.food_prior
            },
            annex_share: self.held_by(pi).and_then(|(_, qi)| qi).map_or(0.0, |qi| {
                let kcal: f64 = self.polities[qi]
                    .stores
                    .iter()
                    .zip(&ctx.catalog.goods)
                    .map(|(kg, g)| kg * g.kcal_per_kg)
                    .sum();
                let theirs = self
                    .households
                    .iter()
                    .filter(|(_, x)| self.polities[qi].holds(x.settlement) && !x.members.is_empty())
                    .count();
                kcal / (households.len() + theirs).max(1) as f64
            }),
            forum_cases: self.forum_cases(ctx, pi),
            trade: self.trade_forecast(ctx, pi, households),
            debts: self.debt_stakes(ctx, households),
            store_food: (
                stock_kcal(&self.polities[pi].stores, &ctx.catalog.goods),
                households.len(),
            ),
            weighed,
            mints,
            pay: self.pay_forecasts(ctx, pi, households),
            adults: {
                let mut a = adults.to_vec();
                a.sort_unstable();
                a
            },
        }
    }

    /// The cases polity `pi`'s gathering or forum heard within memory (M7b slices BR, BS), when
    /// its content offers a forum.
    fn forum_cases(&self, ctx: &Ctx, pi: usize) -> Vec<ForumCase> {
        if !ctx
            .catalog
            .policies
            .iter()
            .any(|d| d.kind == PolicyKind::HearCases)
        {
            return Vec::new();
        }
        let since = ctx.now.day_index() - i64::from(ctx.params.crime.remember_days);
        let polity = &self.polities[pi];
        self.order
            .cases
            .iter()
            .filter(|c| polity.holds(Some(c.settlement)))
            .filter(|c| c.heard.is_some_and(|t| t.day_index() >= since))
            .filter(|c| {
                matches!(
                    c.stage,
                    crate::crime::CaseStage::Found
                        | crate::crime::CaseStage::NotFound
                        | crate::crime::CaseStage::Unheard
                )
            })
            .map(|c| ForumCase {
                gathered: c.forum.is_none(),
                accuser: c.accuser,
                accused: c.accused_household,
                kcal: f64::from(c.kcal),
                found: c.stage == crate::crime::CaseStage::Found,
                stances: c
                    .stances
                    .iter()
                    .map(|r| (r.person, r.household, r.stance))
                    .collect(),
            })
            .collect()
    }

    /// What closing each source of water an adult among `adults` holds sickens, and polity `pi`
    /// does not close now, would bring each of households `households` (M6a slice BA, step four;
    /// ADR-0021 §7), by source cell and then household, over the days a tally speaks of (the
    /// days sickness stays news). Only a household whose people would draw there, their
    /// suspicions aside, and that knows somewhere else to draw, is touched: its people walk to
    /// that other source for every load, each hour worth the food an hour of its work brings
    /// (research 11-07 §2.3, as for a crossing); against that, its people's own tally of the
    /// source (how much likelier a household drawing there was to have sickness lately than one
    /// drawing elsewhere) times what a case costs it, `case_days` and its believed share of a
    /// death at `w_death` days of work, each day of work its hours at that worth. Who suspects
    /// nothing believes nothing avoided.
    fn closure_forecasts(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
        adults: &[(PermanentId, PermanentId)],
    ) -> Vec<(u32, Vec<ClosureForecast>)> {
        let (_, cells) = self.suspecting(pi, adults);
        if cells.is_empty() {
            return Vec::new();
        }
        let params = ctx.params;
        let sp = &params.suspicion;
        let window = f64::from(params.word.sickness_days.max(1));
        let carry = params.household.carry_water_l.max(1e-9);
        let case_days = sp.case_days + sp.case_death_share * sp.w_death;
        let trip = |o: &WaterOption| 2.0 * o.walk_min + o.lift_min;
        // Where each household would draw, its people's suspicions aside.
        let draws: Vec<(PermanentId, &Household, Option<&HomeField>, WaterOption)> = households
            .iter()
            .filter_map(|&h| {
                let x = self.household(h)?;
                let key = x
                    .settlement
                    .filter(|s| self.homes.contains_key(s))
                    .unwrap_or(x.id);
                let home = self.homes.get(&key);
                let o = self.water_source_among(ctx, home, Some((x, false)), &|_| false)?;
                Some((h, x, home, o))
            })
            .collect();
        cells
            .into_iter()
            .map(|cell| {
                let source = crate::uses::Place::Source(cell);
                let of = draws
                    .iter()
                    .filter(|d| d.3.cell == cell)
                    .filter_map(|&(h, x, home, o)| {
                        let other =
                            self.water_source_among(ctx, home, Some((x, false)), &|c| c == cell)?;
                        let kcal = self.hour_kcal(ctx, x);
                        let loads = x.members.len() as f64 * x.water_use(&params.household) / carry;
                        let walked = (trip(&other) - trip(&o)).max(0.0) / 60.0 * loads * window;
                        let believed = x
                            .members
                            .iter()
                            .flat_map(|&m| self.word.suspicions_of(m))
                            .filter(|s| s.source == source)
                            .map(|s| {
                                let c = s.counts;
                                let at = f64::from(c.sick_at) / f64::from(c.at.max(1));
                                let rest =
                                    f64::from(c.sick_elsewhere) / f64::from(c.elsewhere.max(1));
                                (at - rest).max(0.0)
                            })
                            .fold(0.0, f64::max);
                        let avoided = believed * case_days * params.farm.work_hours_per_day * kcal;
                        Some((h, avoided, walked * kcal))
                    })
                    .collect();
                (cell, of)
            })
            .collect()
    }

    /// A closing of a source of water (M6a slice BA, step four), in words that name the source
    /// as its people know it: "the closing of Wren's household's well: nobody may draw there".
    /// None for any other law, or one whose source is not recorded.
    pub(super) fn closing_words(&self, ctx: &Ctx, pi: usize, law: &Law) -> Option<String> {
        if law.kind != PolicyKind::CloseSource {
            return None;
        }
        let source = self.polities[pi].closure_of(law.id)?;
        Some(format!(
            "the closing of {}: nobody may draw there",
            self.source_words(ctx, source)
        ))
    }

    /// The sources of water closings in force where household `hh` lives name that `person`
    /// keeps (M6a slice BA, step four): those they know of, as a curfew's keeping is weighed
    /// (research 09-06 §1.5: the norms they hold that the gathering binds, where they stood on it
    /// and their regard for its sponsor), when that weighs above nothing. In place order.
    pub(crate) fn closures_kept(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        hh: &Household,
    ) -> Vec<crate::uses::Place> {
        let Some(pi) = hh.settlement.and_then(|s| self.polity_of(s)) else {
            return Vec::new();
        };
        let polity = &self.polities[pi];
        if polity.closed.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for (place, law) in polity.closed_now() {
            let Some(l) = polity.laws.iter().find(|l| l.id == law) else {
                continue;
            };
            if !l.knows(person) {
                continue;
            }
            let stance = l
                .stances
                .iter()
                .find(|r| r.person == person)
                .map_or(0.0, |r| r.stance.sign());
            let regard = self
                .ties
                .regard(person, l.sponsor, ctx.now.day_index(), &ctx.params.ties);
            let norm = self.norm_points(ctx, person);
            if crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity) > 0.0 {
                out.push(place);
            }
        }
        out
    }

    /// `person` of household `household` drew water at `cell`: if a closing in force where they
    /// live names it, they broke it, knowing of it or not (M6a slice BA, step four).
    pub(super) fn drew_at(&mut self, person: PermanentId, household: PermanentId, cell: u32) {
        let Some(pi) = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return;
        };
        if self.polities[pi].closed.is_empty() {
            return;
        }
        let source = crate::uses::Place::Source(cell);
        let Some(law) = self.polities[pi]
            .closed_now()
            .into_iter()
            .find(|c| c.0 == source)
            .map(|c| c.1)
        else {
            return;
        };
        if let Some(l) = self.polities[pi].law_mut(law) {
            if l.knows(person) {
                l.compliance.broken += 1;
            } else {
                l.compliance.broken_unaware += 1;
            }
        }
    }

    /// Settlement `settlement`'s polity reviews its issues now, as on its review day: for tests.
    #[doc(hidden)]
    pub fn review_for_tests(&mut self, ctx: &mut Ctx, settlement: PermanentId) {
        if let Some(pi) = self.polity_of(settlement) {
            self.polities[pi].reviewed = ctx.now.day_index();
            self.review(ctx, pi);
        }
    }

    /// What a law closing the source at `cell` would bring each household of settlement
    /// `settlement`, points, as its members would weigh it at the gathering: for tests.
    #[doc(hidden)]
    pub fn closure_gains_for_tests(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        cell: u32,
    ) -> Vec<(PermanentId, f64)> {
        self.site_gains_for_tests(ctx, settlement, PolicyKind::CloseSource, cell)
    }

    /// What a law of kind `kind` naming the site at `cell` (a source to close, a well to dig)
    /// would bring each household of settlement `settlement`, points, as its members would weigh
    /// it at the gathering: for tests.
    #[doc(hidden)]
    pub fn site_gains_for_tests(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        kind: PolicyKind,
        cell: u32,
    ) -> Vec<(PermanentId, f64)> {
        let Some(pi) = self.polity_of(settlement) else {
            return Vec::new();
        };
        let Some(k) = ctx.catalog.policies.iter().position(|d| d.kind == kind) else {
            return Vec::new();
        };
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        let adults = self.members_of(settlement, ctx.now, ctx.params);
        let fc = self.forecasts(ctx, pi, &households, &adults);
        let m = MoveOption {
            policy: k as u16,
            levy_share: 0.0,
            issue: IssueKind::SourceSuspected,
            nominee: None,
            sanction: Default::default(),
            hours: (0, 0),
            body: None,
            ends: None,
            site: Some(cell),
            target: None,
            own_gain: 0.0,
            followers_gain: 0.0,
            support: 0.5,
        };
        let pp = &ctx.params.polity;
        households
            .iter()
            .map(|&h| {
                (
                    h,
                    pp.w_gain * fc.gain(&m, h, &ctx.catalog.policies, ctx.params),
                )
            })
            .collect()
    }

    /// `person` of household `household` drew water at `cell` (see [`Population::drew_at`]): for
    /// tests.
    #[doc(hidden)]
    pub fn drew_at_for_tests(&mut self, person: PermanentId, household: PermanentId, cell: u32) {
        self.drew_at(person, household, cell);
    }

    /// What households `households` of polity `pi` weigh a trade law by (M7c slice BV): built
    /// only when the content has one and its people bought elsewhere within a year.
    pub(super) fn trade_forecast(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
    ) -> TradeForecast {
        let has = ctx.catalog.policies.iter().any(|d| {
            matches!(
                d.kind,
                PolicyKind::Duty
                    | PolicyKind::Embargo
                    | PolicyKind::Recognize
                    | PolicyKind::Privilege
            )
        });
        if !has {
            return TradeForecast::default();
        }
        let away = self.bought_away(
            self.polities[pi].settlement,
            ctx.now.day_index() - DAYS_PER_YEAR,
        );
        // Sales at home, of goods some privilege in the content is over (M7c slice BX).
        let privileged: Vec<GoodUse> = ctx
            .catalog
            .policies
            .iter()
            .filter_map(|d| d.privilege.map(|g| g.purpose))
            .collect();
        let since = ctx.now.day_index() - DAYS_PER_YEAR;
        let home: Vec<(PermanentId, PermanentId, usize, f64)> = self
            .market(self.polities[pi].settlement)
            .map(|m| {
                m.recent
                    .iter()
                    .filter(|t| t.at.day_index() >= since)
                    .filter(|t| {
                        ctx.catalog
                            .goods
                            .get(usize::from(t.good))
                            .is_some_and(|g| privileged.contains(&g.purpose))
                    })
                    .map(|t| {
                        let seller = self.firm(t.seller).map_or(t.seller, |f| f.owner);
                        (seller, t.buyer, usize::from(t.good), f64::from(t.units))
                    })
                    .collect()
            })
            .unwrap_or_default();
        if away.is_empty() && home.is_empty() {
            return TradeForecast::default();
        }
        let mut goods: Vec<usize> = away
            .iter()
            .map(|b| b.good)
            .chain(home.iter().map(|s| s.2))
            .collect();
        goods.sort_unstable();
        goods.dedup();
        let mut unit = Vec::new();
        let mut sellers = Vec::new();
        let mut hs = households.to_vec();
        hs.sort_unstable();
        for &h in &hs {
            let Some(x) = self.household(h) else {
                continue;
            };
            let costs = self.own_costs_of(ctx, x);
            let hour = self.hour_kcal(ctx, x);
            for &g in &goods {
                // What a unit is worth to it: what making one would cost it, or, for a good it
                // cannot make, its own ask for one if it offers it, or else what was paid for one
                // at the other doors, at its own cost of the payment (M7c slice BW).
                let own = costs.get(g).copied().flatten().or_else(|| {
                    x.offers
                        .iter()
                        .filter(|o| usize::from(o.good) == g && o.ask_h > 0.0)
                        .map(|o| f64::from(o.ask_h))
                        .reduce(f64::min)
                });
                let paid = || {
                    let (mut cost, mut units) = (0.0, 0.0);
                    for b in away.iter().filter(|b| b.good == g) {
                        let pay = costs.get(b.payment).copied().flatten()?;
                        cost += b.paid * pay;
                        units += b.units;
                    }
                    (units > 0.0).then(|| cost / units)
                };
                if let Some(c) = own.or_else(paid) {
                    unit.push((h, g, c * hour));
                }
            }
            for p in GoodUse::ALL {
                if self.offers_at_home(ctx, h, p) {
                    sellers.push((h, p));
                }
            }
        }
        let duties = self.polities[pi]
            .laws
            .iter()
            .filter(|l| l.kind == PolicyKind::Duty && l.status == LawStatus::InForce)
            .filter_map(|l| {
                let d = ctx.catalog.policies.get(usize::from(l.policy))?.duty?;
                Some((d.purpose, f64::from(l.levy_share), d.exempt_sellers))
            })
            .collect();
        let mut orgs: Vec<(PermanentId, GoodUse, Vec<PermanentId>, bool)> = Vec::new();
        for &h in &hs {
            let Some(x) = self.household(h) else {
                continue;
            };
            for &p in &x.members {
                let Some(f) = self
                    .factions
                    .membership(p)
                    .and_then(|m| self.factions.get(m.faction))
                    .filter(|f| f.ended.is_none())
                else {
                    continue;
                };
                let Some(stake) = f.stake else {
                    continue;
                };
                match orgs.iter_mut().find(|o| o.0 == f.id) {
                    Some(o) => {
                        if o.2.last() != Some(&h) {
                            o.2.push(h);
                        }
                    }
                    None => orgs.push((f.id, stake.purpose, vec![h], self.recognizes(pi, f.id))),
                }
            }
        }
        orgs.sort_unstable_by_key(|o| o.0);
        TradeForecast {
            away,
            unit,
            sellers,
            purpose: ctx.catalog.goods.iter().map(|g| g.purpose).collect(),
            households: hs.len(),
            duties,
            orgs,
            home,
        }
    }

    /// What an hour of household `hh`'s work brings it in food, kcal: of the foods it can get,
    /// the most energy for an hour at its own cost (M3a, ADR-0006).
    pub(super) fn hour_kcal(&self, ctx: &Ctx, hh: &Household) -> f64 {
        self.own_costs_of(ctx, hh)
            .iter()
            .zip(&ctx.catalog.goods)
            .filter_map(|(c, g)| {
                let hours = (*c)?;
                (g.kcal_per_kg > 0.0 && hours > 0.0).then(|| g.kcal_per_kg / hours)
            })
            .fold(0.0, f64::max)
    }

    /// The places households `households` of polity `pi`'s settlement saw people of another
    /// settlement working within the last year, and that the polity does not claim (M5c slice
    /// AT), in place order: what a claim proposed now would name.
    pub(super) fn unclaimed_outsider_places(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
    ) -> Vec<crate::uses::Place> {
        let since = ctx.now.day_index() - OUTSIDERS_DAYS;
        let claimed = self.polities[pi].claims_now();
        let mut out: Vec<crate::uses::Place> = households
            .iter()
            .flat_map(|&h| self.uses.outsiders_seen(h, since))
            .map(|(u, _)| u.place)
            .filter(|p| p.claimable() && claimed.binary_search(p).is_err())
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// What household `household` believes people of other settlements take a year at `places`,
    /// kcal (M5c slice AT): of the food it got at each, as much again as their share of the days
    /// at it, outsiders' days over its own and theirs. A household that works none of them
    /// believes they take nothing of its own.
    pub(super) fn outsiders_take(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        places: &[crate::uses::Place],
    ) -> f64 {
        let half = ctx.params.places.use_half_life_days;
        let day = ctx.now.day_index();
        places
            .iter()
            .filter_map(|&p| self.uses.held(household, p, day, half))
            .map(|u| {
                let theirs: f64 = u.outsiders.iter().map(|o| f64::from(o.days)).sum();
                let share = theirs / (f64::from(u.days) + theirs).max(1e-9);
                crate::uses::per_year(f64::from(u.kcal), half) * share
            })
            .sum()
    }

    /// What polity `pi`'s gatherings decided within memory under its present custom (M4c slice
    /// AF): each law and case, with the stances taken there (open acclamation: those who came saw
    /// them). Only decisions the present custom made count: a change of custom is judged by how
    /// the custom it would replace has decided, so a newly amended custom is not undone on the
    /// strength of what an older one did (that let two camps toggle the custom week by week).
    pub(super) fn decisions_seen(&self, ctx: &Ctx, pi: usize) -> Vec<Seen> {
        let pp = &ctx.params.polity;
        let day = ctx.now.day_index();
        let memory = i64::from(pp.vote_memory_days);
        let polity = &self.polities[pi];
        let custom_since = polity.versions.last().map_or(polity.founded, |v| v.since);
        let recent = |t: Option<SimTime>| {
            t.is_some_and(|t| day - t.day_index() <= memory && t >= custom_since)
        };
        // Amendments themselves are not among them: a rule is judged by the laws and cases it
        // decides, not by other changes of rule (which would let the custom feed on itself).
        let mut decisions: Vec<Seen> = polity
            .laws
            .iter()
            .filter(|l| recent(l.decided) && !l.stances.is_empty())
            .filter(|l| l.kind != PolicyKind::AmendBody)
            .map(|l| Seen {
                stances: l
                    .stances
                    .iter()
                    .map(|r| (r.person, r.household, r.stance, f64::from(r.gain)))
                    .collect(),
                passed: l.outcome == Some(Outcome::Passed),
            })
            .collect();
        decisions.extend(
            self.order
                .cases
                .iter()
                .filter(|c| recent(c.heard) && !c.stances.is_empty())
                .filter(|c| polity.laws.iter().any(|l| l.id == c.law))
                .map(|c| Seen {
                    stances: c
                        .stances
                        .iter()
                        .map(|r| (r.person, r.household, r.stance, f64::from(r.stake)))
                        .collect(),
                    passed: c.stage == crate::crime::CaseStage::Found,
                }),
        );
        decisions
    }

    /// What body `b` would have brought households `households` from `decisions`, summed (research
    /// 09-02 §3.7, 09-05 §1.2: a rule is judged by how it would have decided what was seen): each
    /// decision one of them was at that `b`, admitting those it admits now, would have turned the
    /// other way, at that household's stake in it (M4c slice AI: a faction's program).
    pub(super) fn body_gain(
        &self,
        ctx: &Ctx,
        pi: usize,
        b: &Body,
        sponsor: PermanentId,
        households: &[PermanentId],
        decisions: &[Seen],
    ) -> f64 {
        let (now, params) = (ctx.now, ctx.params);
        let pp = &params.polity;
        let settlement = self.polities[pi].settlement;
        let adults = self.members_of(settlement, now, params);
        let admitted: Vec<PermanentId> = self
            .admitted(b.members, &adults, &ctx.land.fields, now, params)
            .into_iter()
            .map(|a| a.0)
            .collect();
        // A council is judged by those its sponsor expects would sit (M7a slice BO).
        let sitters = b.seats.map(|seats| {
            let electorate: Vec<PermanentId> = self
                .admitted(seats.electorate, &adults, &ctx.land.fields, now, params)
                .into_iter()
                .map(|a| a.0)
                .collect();
            (
                seats,
                self.forecast_sitters(ctx, pi, &seats, &electorate, sponsor),
            )
        });
        decisions
            .iter()
            .map(|s| {
                if let Some((seats, sit)) = &sitters {
                    let stances: Vec<(PermanentId, Stance)> =
                        s.stances.iter().map(|r| (r.0, r.2)).collect();
                    // The evenings it would have saved them (ADR-0024 §6, M7a slice BQ): each of
                    // their people who came who would not sit.
                    let saved = s
                        .stances
                        .iter()
                        .filter(|r| households.contains(&r.1) && sit.binary_search(&r.0).is_err())
                        .count() as f64
                        * params.elections.w_evening
                        / pp.w_gain.max(1e-9);
                    let now_passes = super::elections::council_would_pass(b, seats, sit, &stances);
                    if now_passes == s.passed {
                        return saved;
                    }
                    let sign = if now_passes { 1.0 } else { -1.0 };
                    return saved
                        + households
                            .iter()
                            .filter_map(|&h| s.stances.iter().find(|r| r.1 == h))
                            .map(|r| sign * r.3 / pp.w_gain.max(1e-9))
                            .sum::<f64>();
                }
                let (mut present, mut support, mut oppose) = (0, 0, 0);
                for &(p, _, stance, _) in &s.stances {
                    if admitted.binary_search(&p).is_err() {
                        continue;
                    }
                    present += 1;
                    match stance {
                        Stance::Support => support += 1,
                        Stance::Oppose => oppose += 1,
                        Stance::Abstain => {}
                    }
                }
                let now_passes =
                    b.decide(admitted.len() as u32, present, support, oppose) == Outcome::Passed;
                if now_passes == s.passed {
                    return 0.0;
                }
                let sign = if now_passes { 1.0 } else { -1.0 };
                households
                    .iter()
                    .filter_map(|&h| s.stances.iter().find(|r| r.1 == h))
                    .map(|r| sign * r.3 / pp.w_gain.max(1e-9))
                    .sum::<f64>()
            })
            .sum()
    }

    /// A polity's routine review (ADR-0013 §5): with nothing before the gathering and an issue
    /// present, its notables and the elders of households whose food will not last to their
    /// next harvest each weigh proposing what answers it, at each level the template allows,
    /// against proposing nothing. The first to propose puts it to a gathering called for a few
    /// days on.
    fn review(&mut self, ctx: &mut Ctx, pi: usize) {
        let (now, params) = (ctx.now, ctx.params);
        let pp = &params.polity;
        let polity = &self.polities[pi];
        if polity.agenda().is_some() || polity.gathering.is_some() {
            return;
        }
        let settlement = polity.settlement;
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        let pressed: Vec<PermanentId> = households
            .iter()
            .copied()
            .filter(|&h| self.pressed(ctx, h))
            .collect();
        let polity = &self.polities[pi];
        let unkept = polity
            .in_force(&ctx.catalog.policies, PolicyKind::CommonStore)
            .next()
            .is_some()
            && polity.keeper().is_none()
            && stock_kcal(&polity.stores, &ctx.catalog.goods) > 0.0;
        // Households of the settlement that found food taken from their stores in the last year,
        // and since the last review (what they know: M4b slice AB).
        let day = now.day_index();
        let review_since = day - i64::from(pp.review_days.max(1));
        let robbed = |since: i64| -> Vec<PermanentId> {
            households
                .iter()
                .copied()
                .filter(|&h| self.order.lost_since(h, since))
                .collect()
        };
        let robbed_lately = robbed(review_since);
        // A body that took the deciding weighs the laws the old custom made (M4c slice AI).
        let founding = self
            .founding_since(ctx, pi)
            .filter(|&since| !self.inherited(pi, since).is_empty());
        // Households that saw outsiders at places they work, unclaimed (M5c slice AT).
        let outsider_places = self.unclaimed_outsider_places(ctx, pi, &households);
        let saw_outsiders: Vec<PermanentId> = if outsider_places.is_empty() {
            Vec::new()
        } else {
            let since = day - OUTSIDERS_DAYS;
            households
                .iter()
                .copied()
                .filter(|&h| {
                    self.uses
                        .outsiders_seen(h, since)
                        .any(|(u, _)| outsider_places.binary_search(&u.place).is_ok())
                })
                .collect()
        };
        // Where its people could build a crossing together (M5c slice AW, step three).
        let sites = self.public_sites(ctx, settlement, &households);
        // The settlement's adults; households whose people saw their own sick lately, and those
        // who hold that a source sickens (M6a slice BA, step four).
        let adults = self.members_of(settlement, now, params);
        let sick_homes: Vec<PermanentId> = households
            .iter()
            .copied()
            .filter(|&h| self.sick_at_home(h))
            .collect();
        let (suspecting, _) = self.suspecting(pi, &adults);
        // Where its people could dig a well together.
        let well = self.public_well(ctx, settlement, &households);
        // Where they could build a flood bank together, and the households a flood cost within
        // memory (M6b slice BG).
        let bank = self.public_bank(ctx, settlement, &households);
        let flooded_homes: Vec<PermanentId> = households
            .iter()
            .copied()
            .filter(|&h| {
                self.household(h)
                    .is_some_and(|x| !x.flood_losses.is_empty())
            })
            .collect();
        // And households that lost a building to fire or saw one burn within memory (M6b slice
        // BH).
        let burned_homes = self.burned_homes(&households);
        let gave_way_homes = self.gave_way_homes(&households);
        let mut issues = self.issues(
            ctx,
            settlement,
            !pressed.is_empty(),
            unkept,
            !robbed(day - 365).is_empty(),
            founding.is_some(),
            !saw_outsiders.is_empty(),
            !sites.is_empty(),
            !sick_homes.is_empty(),
            !suspecting.is_empty(),
            well.is_some(),
            bank.is_some(),
            self.worn_bank(ctx, pi, households.len()).is_some(),
            !burned_homes.is_empty(),
            !gave_way_homes.is_empty(),
            households
                .iter()
                .any(|&h| self.raided_within(h, day - i64::from(params.crime.remember_days))),
        );
        // A band its law sent holds another settlement (M6c slice BM).
        if self.held_by(pi).is_some() {
            issues.push(IssueKind::Holding);
        }
        // A case one of its households brought went unheard, or was not found, within memory
        // (M7b slice BR, ADR-0025 §1).
        let remember = day - i64::from(params.crime.remember_days);
        if self.order.cases.iter().any(|c| {
            c.settlement == settlement
                && c.opened >= remember
                && matches!(
                    c.stage,
                    crate::crime::CaseStage::Unheard | crate::crime::CaseStage::NotFound
                )
        }) || self.polities[pi]
            .forum(&ctx.catalog.policies)
            .is_some_and(|(f, _)| {
                self.word
                    .grievances
                    .iter()
                    .any(|g| g.blamed == Blamed::Forum(f.id) && g.unresolved_days > 0.0)
            })
        {
            issues.push(IssueKind::CasesWaiting);
        }
        // Its households offer at home what its people fetched elsewhere lately (M7c slice BV).
        if self.fetched_elsewhere(ctx, pi, &households) {
            issues.push(IssueKind::FetchedElsewhere);
        }
        // Its store holds money while a law of its asks for common work (M7d slice CE3a).
        let money_held = self.polities[pi].stores.iter().enumerate().any(|(g, &kg)| {
            kg > 1e-9 && ctx.catalog.goods.get(g).is_some_and(|d| d.money.is_some())
        });
        let common_work = self.polities[pi].laws.iter().any(|l| {
            l.status == LawStatus::InForce
                && matches!(
                    l.kind,
                    PolicyKind::BuildCrossing
                        | PolicyKind::DigWell
                        | PolicyKind::BuildBank
                        | PolicyKind::KeepBank
                        | PolicyKind::Enclose
                )
        });
        if money_held && common_work {
            issues.push(IssueKind::CoinIdle);
        }
        // A household of it paid or was paid in weighed metal lately (M7d slice CE2b).
        if self.weighed_lately(ctx, &households) {
            issues.push(IssueKind::Weighed);
        }
        // A loan one of its households made was refused or left unpaid, and nothing was taken
        // for it nor a case brought (M7d slice CB).
        if households
            .iter()
            .any(|&h| !self.unheard_debts(h).is_empty())
        {
            issues.push(IssueKind::DebtsUnpaid);
        }
        let polity = &self.polities[pi];
        // A watch may have one more while takings continue (M4c slice AI: a watch of several):
        // food taken from a household here since the newest watcher was named.
        let newest_watch = polity
            .watchers()
            .filter_map(|(_, l)| l.decided)
            .map(|t| t.day_index())
            .max();
        let taken_since = newest_watch.is_some_and(|since| {
            households
                .iter()
                .any(|&h| self.order.lost_since(h, since + 1))
        });
        // A polity builds one crossing at a time, as a household does (M5c slice AW).
        let building_crossing = ctx.land.crossings.list.iter().any(|c| {
            c.owner == civ_land::crossings::CrossingOwner::Polity(polity.id)
                && matches!(c.state, civ_land::crossings::CrossingState::Building { .. })
        });
        // Templates that answer a present issue and are not already in force here.
        let open: Vec<(u16, IssueKind)> = ctx
            .catalog
            .policies
            .iter()
            .enumerate()
            .filter(|(k, d)| {
                // Another crossing, at a site no crossing stands at, once none is being built
                // (M5c slice AW).
                if d.kind == PolicyKind::BuildCrossing {
                    return !building_crossing;
                }
                // Another closing, of a source none in force closes, and another well once none
                // is being dug (M6a slice BA).
                if matches!(d.kind, PolicyKind::CloseSource | PolicyKind::DigWell) {
                    return true;
                }
                // Another recognition, of an organization none in force recognizes (M7c slice
                // BW), and another privilege, for one none in force of its template names (M7c
                // slice BX).
                if matches!(d.kind, PolicyKind::Recognize | PolicyKind::Privilege) {
                    return true;
                }
                // Another bank once none is being built (M6b slice BG: `public_bank` holds to
                // one at a time).
                if d.kind == PolicyKind::BuildBank {
                    return true;
                }
                // A keeping, of a bank no law in force keeps (M6b slice BG: `worn_bank`).
                if d.kind == PolicyKind::KeepBank {
                    return true;
                }
                // A palisade, while the settlement has none (M6c slice BK: `public_enclosure`).
                if d.kind == PolicyKind::Enclose {
                    return true;
                }
                // One forum at a time (M7b slice BR): another may replace the one in force (M7b
                // slice BS: its seat or its authority changed by a law).
                if d.kind == PolicyKind::HearCases {
                    return !polity.laws.iter().any(|l| {
                        l.kind == PolicyKind::HearCases
                            && l.status == LawStatus::InForce
                            && usize::from(l.policy) == *k
                    });
                }
                (d.kind == PolicyKind::KeepWatch && taken_since)
                    // Another claim, for places the claims in force do not name (M5c slice AT).
                    || (d.kind == PolicyKind::ClaimPlace && !saw_outsiders.is_empty())
                    || !polity
                        .laws
                        .iter()
                        .any(|l| usize::from(l.policy) == *k && l.status == LawStatus::InForce)
            })
            .filter_map(|(k, d)| {
                let issue = d.answers.iter().copied().find(|a| issues.contains(a))?;
                Some((k as u16, issue))
            })
            .collect();
        // An amendment of the custom answers no settlement's issue: it is open to those a
        // gathering overruled (M4c slice AF), so a content that has one always reviews.
        let amendable = ctx
            .catalog
            .policies
            .iter()
            .any(|d| d.kind == PolicyKind::AmendBody && !d.bodies.is_empty());
        // Holders of an ideology weigh its program when the problem it explains is before the
        // village (M4c slice AG).
        let programs = ctx
            .catalog
            .ideologies
            .iter()
            .any(|d| !d.program.is_empty() && issues.contains(&d.explains))
            && !self.ideologies.held.is_empty();
        if open.is_empty() && !amendable && !programs {
            return;
        }
        // Those of the settlement's adults its body admits (M4c slice AF): only members propose
        // and stand at its gatherings; anyone may hold an office it creates.
        let members = self.admitted(
            self.polities[pi].body.members,
            &adults,
            &ctx.land.fields,
            now,
            params,
        );
        // What each household expects, to forecast what a move would bring it.
        let mut fc = self.forecasts_with(ctx, pi, &households, &adults, sites, well, bank);
        self.with_enclosure(ctx, &mut fc, settlement, &households);
        let outlooks = &fc.outlooks;
        // Who could keep the store: adults whose household is under a roof.
        let roofed: Vec<PermanentId> = adults
            .iter()
            .filter(|(_, h)| self.household(*h).is_some_and(|x| x.sheltered))
            .map(|(p, _)| *p)
            .collect();
        // Who weighs moves (ADR-0014 §4): the notables, a compute tier; and those something
        // reached since the last review: the elders of households whose food will not last, and
        // whoever came to a gathering or learnt a law. With the tier off, every adult; and every
        // member of a body that took the deciding while it is founding (M4c slice AI; research
        // 09-11 §1.6: representation through the body that exists).
        let mut deliberators: Vec<PermanentId> =
            if self.every_adult_deliberates || founding.is_some() {
                members.iter().map(|m| m.0).collect()
            } else {
                let since = now.day_index() - i64::from(pp.review_days.max(1));
                let reached = self.polities[pi].laws.iter().flat_map(|l| {
                    let came = l
                        .decided
                        .filter(|t| t.day_index() > since)
                        .map(|_| l.stances.iter().map(|r| r.person))
                        .into_iter()
                        .flatten();
                    let learnt = l.known.iter().filter(move |k| k.1 > since).map(|k| k.0);
                    came.chain(learnt)
                });
                self.standing
                    .in_settlement(settlement)
                    .filter(|r| r.notable)
                    .map(|r| r.person)
                    .chain(
                        pressed
                            .iter()
                            .chain(&robbed_lately)
                            .chain(&saw_outsiders)
                            .chain(&sick_homes)
                            // And households a flood cost within memory (M6b slice BG), and
                            // those that knew a fire (M6b slice BH).
                            .chain(&flooded_homes)
                            .chain(&burned_homes)
                            .chain(&gave_way_homes)
                            .filter_map(|&h| self.elder_of(h, now, params)),
                    )
                    // And whoever holds that a source sickens (M6a slice BA).
                    .chain(suspecting.iter().copied())
                    .chain(reached)
                    // And whoever holds an ideology whose program answers a problem before the
                    // village (M4c slice AG).
                    .chain(
                        self.ideologies
                            .held
                            .iter()
                            .filter(|h| {
                                ctx.catalog
                                    .ideologies
                                    .get(usize::from(h.ideology))
                                    .is_some_and(|d| {
                                        !d.program.is_empty() && issues.contains(&d.explains)
                                    })
                            })
                            .map(|h| h.holder),
                    )
                    .collect()
            };
        deliberators.sort_unstable();
        deliberators.dedup();
        deliberators.retain(|d| members.binary_search_by_key(d, |m| m.0).is_ok());
        let tp = &params.ties;
        // What its gatherings decided within memory (M4c slice AF): each law and case, with the
        // stances taken there (open acclamation: those who came saw them), and who each kind of
        // body would admit now.
        // Only decisions the present custom made count: a change of custom is judged by how the
        // custom it would replace has decided, so a newly amended custom is not undone on the
        // strength of what an older one did (that let two camps toggle the custom week by week).
        let decisions = self.decisions_seen(ctx, pi);
        let polity = &self.polities[pi];
        let (current, polity_id) = (polity.body, polity.id);
        let admitted_by: Vec<Vec<PermanentId>> = Membership::ALL
            .iter()
            .map(|&m| {
                self.admitted(m, &adults, &ctx.land.fields, now, params)
                    .into_iter()
                    .map(|a| a.0)
                    .collect()
            })
            .collect();
        // Whether body `b` would have passed decision `s`: those of the stances it admits now,
        // by its rule (research 09-05 §1.4); a council by the stances of `sitters`, those its
        // sponsor expects would sit (M7a slice BO, ADR-0024 §6).
        let would_pass = |s: &Seen, b: &Body, sitters: Option<&[PermanentId]>| {
            if let (Some(seats), Some(sit)) = (b.seats, sitters) {
                let stances: Vec<(PermanentId, Stance)> =
                    s.stances.iter().map(|r| (r.0, r.2)).collect();
                return super::elections::council_would_pass(b, &seats, sit, &stances);
            }
            let admitted = &admitted_by[usize::from(b.members.code())];
            let (mut present, mut support, mut oppose) = (0, 0, 0);
            for &(p, _, stance, _) in &s.stances {
                if admitted.binary_search(&p).is_err() {
                    continue;
                }
                present += 1;
                match stance {
                    Stance::Support => support += 1,
                    Stance::Oppose => oppose += 1,
                    Stance::Abstain => {}
                }
            }
            b.decide(admitted.len() as u32, present, support, oppose) == Outcome::Passed
        };
        // What body `b` would bring household `h`, from decisions `seen` (research 09-02 §3.7,
        // 09-05 §1.2: a rule is judged by how it would have decided what was seen): each one it
        // would have turned the other way, at the household's stake in it.
        let amend_gain =
            |b: &Body, sitters: Option<&[PermanentId]>, h: PermanentId, seen: &[usize]| -> f64 {
                seen.iter()
                    .map(|&i| {
                        let s = &decisions[i];
                        // Under a council, the evenings it would have saved the household: each
                        // of its people who came who would not sit (ADR-0024 §6, M7a slice BQ).
                        let saved = sitters.map_or(0.0, |sit| {
                            s.stances
                                .iter()
                                .filter(|r| r.1 == h && sit.binary_search(&r.0).is_err())
                                .count() as f64
                                * params.elections.w_evening
                                / pp.w_gain.max(1e-9)
                        });
                        let now_passes = would_pass(s, b, sitters);
                        if now_passes == s.passed {
                            return saved;
                        }
                        let stake = s.stances.iter().find(|r| r.1 == h).map_or(0.0, |r| r.3);
                        let sign = if now_passes { 1.0 } else { -1.0 };
                        saved + sign * stake / pp.w_gain.max(1e-9)
                    })
                    .sum()
            };
        // The decisions those `seen` saw.
        let seen_by = |seen: &dyn Fn(PermanentId, PermanentId) -> bool| -> Vec<usize> {
            decisions
                .iter()
                .enumerate()
                .filter(|(_, s)| s.stances.iter().any(|r| seen(r.0, r.1)))
                .map(|(i, _)| i)
                .collect()
        };
        let deliberator = RuleDeliberator { params: pp };
        let policies = &ctx.catalog.policies;
        let gain_of = |m: &MoveOption, h: PermanentId| fc.gain(m, h, policies, params);
        // While it is founding, the laws the old custom made, each as the move it would be were
        // it proposed now (M4c slice AI): what ending one brings a household is what keeping it
        // would bring, lost.
        let kept: Vec<(PermanentId, MoveOption)> = founding.map_or_else(Vec::new, |since| {
            self.inherited(pi, since)
                .into_iter()
                .map(|l| {
                    let m = MoveOption {
                        policy: l.policy,
                        levy_share: f64::from(l.levy_share),
                        issue: l.issue,
                        nominee: l.holder,
                        sanction: l.sanction,
                        hours: l.hours,
                        body: None,
                        ends: None,
                        target: l.target,
                        // A closing names its source, and a well its site (M6a slice BA).
                        site: match self.polities[pi].closure_of(l.id) {
                            Some(crate::uses::Place::Source(cell)) => Some(cell),
                            _ => self.polities[pi]
                                .well_site_of(l.id)
                                .map(|s| s.cell)
                                .or_else(|| {
                                    self.polities[pi]
                                        .bank_site_of(l.id)
                                        .or_else(|| {
                                            self.polities[pi].kept_bank_of(l.id).map(|k| k.1)
                                        })
                                        .map(|s| super::levees::bank_cell(ctx.map, &s))
                                }),
                        },
                        own_gain: 0.0,
                        followers_gain: 0.0,
                        support: 0.5,
                    };
                    (l.id, m)
                })
                .collect()
        });
        let ended = |m: &MoveOption| m.ends.and_then(|e| kept.iter().find(|k| k.0 == e));
        // Those who keep the watch now (M4c slice AI).
        let watching: Vec<PermanentId> = self.polities[pi].watchers().map(|w| w.0).collect();
        let held = |m: &MoveOption, h: PermanentId| ended(m).map_or(0.0, |k| gain_of(&k.1, h));
        // What each limit a template offers, not in force, would bring each household (M7b slice
        // BT, ADR-0025 §3): half what ending the laws in force it forbids would, for the chance a
        // challenge of each would be found (nothing for a law it is better off under).
        let latitude = params.latitude_deg;
        let limit_gains: Vec<(u16, Vec<(PermanentId, f64)>)> = policies
            .iter()
            .enumerate()
            .filter_map(|(k, def)| {
                let lim = def
                    .limit
                    .as_ref()
                    .filter(|_| def.kind == PolicyKind::Limit)?;
                let in_force = self.polities[pi]
                    .laws
                    .iter()
                    .any(|l| usize::from(l.policy) == k && l.status == LawStatus::InForce);
                if in_force {
                    return None;
                }
                let breached: Vec<MoveOption> = self.polities[pi]
                    .laws
                    .iter()
                    .filter(|l| l.status == LawStatus::InForce)
                    .filter(|l| {
                        lim.reading(l.kind, l.hours, f64::from(l.levy_share), latitude)
                            .is_some_and(|r| r > 0.0)
                    })
                    .map(|l| MoveOption {
                        policy: l.policy,
                        levy_share: f64::from(l.levy_share),
                        issue: l.issue,
                        nominee: l.holder,
                        sanction: l.sanction,
                        hours: l.hours,
                        body: None,
                        ends: None,
                        target: l.target,
                        site: None,
                        own_gain: 0.0,
                        followers_gain: 0.0,
                        support: 0.5,
                    })
                    .collect();
                if breached.is_empty() {
                    return None;
                }
                let gains = outlooks
                    .iter()
                    .map(|(h, _)| {
                        let g: f64 = breached.iter().map(|m| (-gain_of(m, *h)).max(0.0)).sum();
                        (*h, 0.5 * g)
                    })
                    .collect();
                Some((k as u16, gains))
            })
            .collect();
        let limit_gain = |m: &MoveOption, h: PermanentId| {
            limit_gains
                .iter()
                .find(|l| l.0 == m.policy)
                .and_then(|l| l.1.iter().find(|g| g.0 == h))
                .map(|g| g.1)
        };
        for d in deliberators {
            let Some(own) = members
                .binary_search_by_key(&d, |m| m.0)
                .ok()
                .map(|i| members[i].1)
            else {
                continue;
            };
            // The moves open to them: each level of a store, and a keeper of their choosing:
            // themselves if their roof would do, or whoever under a roof they regard most.
            let mut moves = Vec::new();
            // With the laws the ideologies they hold propose for a problem before the village, at
            // the issue each explains (M4c slice AG).
            let mut wants = open.clone();
            for (k, issue) in self.programs_of(ctx, d, &issues) {
                let in_force = self.polities[pi]
                    .laws
                    .iter()
                    .any(|l| l.policy == k && l.status == LawStatus::InForce);
                if !in_force && !wants.iter().any(|w| w.0 == k) {
                    wants.push((k, issue));
                }
            }
            for &(k, issue) in &wants {
                let def = &ctx.catalog.policies[usize::from(k)];
                let blank = MoveOption {
                    policy: k,
                    levy_share: 0.0,
                    issue,
                    nominee: None,
                    sanction: Default::default(),
                    hours: (0, 0),
                    body: None,
                    ends: None,
                    site: None,
                    target: None,
                    own_gain: 0.0,
                    followers_gain: 0.0,
                    support: 0.5,
                };
                match def.kind {
                    PolicyKind::CommonStore => {
                        for &share in &def.levy_shares {
                            moves.push(MoveOption {
                                levy_share: share,
                                ..blank
                            });
                        }
                    }
                    PolicyKind::AgainstTaking | PolicyKind::AgainstRaiding => {
                        for &sanction in &def.bundles {
                            moves.push(MoveOption { sanction, ..blank });
                        }
                    }
                    // A band against the settlement whose people raided their household lately,
                    // as its members know it, led by the one who proposes it (M6c slice BJ).
                    // Not while their polity's word against raiding that settlement is in force,
                    // as they know it (M6c slice BL), nor against a settlement their own polity
                    // holds: once taken in, its people are members (M6c slice BM). Only one of
                    // an age to go with a band proposes leading one: a law naming a leader who
                    // cannot go would lapse the midnight it passed (the M6c demo's probe: every
                    // such law of its world, 17, named a proposer of 67 to 69).
                    PolicyKind::SendBand => {
                        let since = day - i64::from(params.crime.remember_days);
                        let bp = &params.bands;
                        let can_lead = self
                            .person(d)
                            .map(|p| p.age_years(ctx.now))
                            .is_some_and(|a| bp.can_go(a));
                        if let Some(at) = self
                            .raiders_known(own, since)
                            .filter(|_| can_lead)
                            .filter(|&at| !self.polities[pi].holds(Some(at)))
                            .filter(|&at| self.word_against_raiding(d, at).is_none())
                        {
                            moves.push(MoveOption {
                                nominee: Some(d),
                                target: Some(at),
                                ..blank
                            });
                        }
                    }
                    // The settlement a band its law sent holds (M6c slice BM).
                    PolicyKind::Annex => {
                        if let Some((at, _)) = self.held_by(pi) {
                            moves.push(MoveOption {
                                target: Some(at),
                                ..blank
                            });
                        }
                    }
                    PolicyKind::Curfew => {
                        for &hours in &def.hours {
                            moves.push(MoveOption { hours, ..blank });
                        }
                    }
                    // Lending from the store (M7d slice CC): one law, its terms the template's.
                    PolicyKind::LendFromStore => moves.push(blank),
                    // Pay for common work (M7d slice CE3a): one law, its rate the template's.
                    PolicyKind::PayWork => moves.push(blank),
                    // Debts heard (M7d slice CB): one law, its remedy the template's; it
                    // replaces a law of debts heard of another template in force.
                    PolicyKind::DebtsHeard => {
                        let ends = self.polities[pi]
                            .laws
                            .iter()
                            .find(|l| {
                                l.kind == PolicyKind::DebtsHeard && l.status == LawStatus::InForce
                            })
                            .map(|l| l.id);
                        moves.push(MoveOption { ends, ..blank });
                    }
                    // A duty at each rate the template offers (M7c slice BV), naming as its
                    // collector, where the template has one, the adult the deliberator regards
                    // most, themselves included (M7c slice BY: favour in naming, as built).
                    PolicyKind::Duty => {
                        let collector = def.duty.is_some_and(|u| u.collector).then(|| {
                            adults
                                .iter()
                                .map(|a| a.0)
                                .map(|c| {
                                    let r = if c == d {
                                        1.0
                                    } else {
                                        self.ties.regard(d, c, day, tp).clamp(0.0, 1.0)
                                    };
                                    (r, c)
                                })
                                .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)))
                                .map(|x| x.1)
                        });
                        for &share in &def.levy_shares {
                            moves.push(MoveOption {
                                levy_share: share,
                                nominee: collector.flatten(),
                                ..blank
                            });
                        }
                    }
                    // An embargo on the settlement its people bought most from within a year, of
                    // those whose markets the deliberator's household holds a report of, and not
                    // one the polity holds (M7c slice BV).
                    PolicyKind::Embargo => {
                        let settlement = self.polities[pi].settlement;
                        let reports = self.reports.of(own);
                        let mut by_market: Vec<(PermanentId, f64)> = Vec::new();
                        for b in self.bought_away(settlement, day - DAYS_PER_YEAR) {
                            if self.polities[pi].holds(Some(b.market))
                                || !reports.iter().any(|r| r.market == b.market)
                            {
                                continue;
                            }
                            match by_market.iter_mut().find(|m| m.0 == b.market) {
                                Some(m) => m.1 += b.units,
                                None => by_market.push((b.market, b.units)),
                            }
                        }
                        if let Some((at, _)) = by_market
                            .into_iter()
                            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
                        {
                            moves.push(MoveOption {
                                target: Some(at),
                                ..blank
                            });
                        }
                    }
                    // Recognition of each organization by stake someone of the deliberator's
                    // household belongs to that no law in force of the polity recognizes (M7c
                    // slice BW).
                    PolicyKind::Recognize => {
                        for org in self.stake_organizations_of(own) {
                            if !self.recognizes(pi, org) {
                                moves.push(MoveOption {
                                    target: Some(org),
                                    ..blank
                                });
                            }
                        }
                    }
                    // A privilege, with each bundle a finding of a breach may impose, for each
                    // organization by stake of the template's purpose someone of the
                    // deliberator's household belongs to, that the polity recognizes and that no
                    // law in force of the template names (M7c slice BX).
                    PolicyKind::Privilege => {
                        let Some(g) = def.privilege else {
                            continue;
                        };
                        for org in self.stake_organizations_of(own) {
                            let fits = self
                                .factions
                                .get(org)
                                .and_then(|f| f.stake)
                                .is_some_and(|s| s.purpose == g.purpose);
                            let held = self.polities[pi].laws.iter().any(|l| {
                                l.policy == k
                                    && l.status == LawStatus::InForce
                                    && l.target == Some(org)
                            });
                            if fits && !held && self.recognizes(pi, org) {
                                for &sanction in &def.bundles {
                                    moves.push(MoveOption {
                                        target: Some(org),
                                        sanction,
                                        ..blank
                                    });
                                }
                            }
                        }
                    }
                    // An office filled by lot names nobody: the lot draws among those who put
                    // their names in once it passes (M7a slice BN).
                    PolicyKind::KeepStore | PolicyKind::KeepWatch | PolicyKind::Mint
                        if def.tenure.is_some_and(|t| t.by_lot) =>
                    {
                        moves.push(blank);
                    }
                    // A mint, its keeper whoever of the settlement who can work its recipe they
                    // regard most, themselves first (M7d slice CE2b).
                    PolicyKind::Mint => {
                        let technique = def
                            .mint
                            .and_then(|m| ctx.catalog.recipes.get(m.recipe))
                            .and_then(|r| r.technique);
                        let barred = self.term_barred(pi, k, day, def);
                        let pick = adults
                            .iter()
                            .map(|&(c, _)| c)
                            .filter(|&c| Some(c) != barred)
                            .filter(|&c| {
                                self.person(c)
                                    .is_some_and(|p| technique.is_none_or(|t| p.knows(t)))
                            })
                            .map(|c| {
                                let r = if c == d {
                                    1.0
                                } else {
                                    self.ties.regard(d, c, day, tp).clamp(0.0, 1.0)
                                };
                                (r, c)
                            })
                            .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
                        if let Some((_, c)) = pick {
                            moves.push(MoveOption {
                                nominee: Some(c),
                                ..blank
                            });
                        }
                    }
                    PolicyKind::KeepStore => {
                        let barred = self.term_barred(pi, k, day, def);
                        let pick = roofed
                            .iter()
                            .filter(|&&c| Some(c) != barred)
                            .map(|&c| {
                                let r = if c == d {
                                    1.0
                                } else {
                                    self.ties.regard(d, c, day, tp).clamp(0.0, 1.0)
                                };
                                (r, c)
                            })
                            .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
                        if let Some((_, c)) = pick {
                            moves.push(MoveOption {
                                nominee: Some(c),
                                ..blank
                            });
                        }
                    }
                    // An amendment answers no settlement's issue: it is weighed below.
                    PolicyKind::AmendBody => {}
                    // A claim names the places where the settlement's people saw outsiders at
                    // work (M5c slice AT): one move, its places fixed when it is proposed.
                    PolicyKind::ClaimPlace => moves.push(blank),
                    // A crossing at one of the sites their own household wades, the three it
                    // wades most (M5c slice AW, step three).
                    PolicyKind::BuildCrossing => {
                        let mut mine: Vec<(f64, u32)> = fc
                            .sites
                            .iter()
                            .map(|p| (p.wades_of(own), p.site.cell))
                            .filter(|w| w.0 > 0.0)
                            .collect();
                        mine.sort_unstable_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
                        for &(_, cell) in mine.iter().take(PUBLIC_SITES_WEIGHED) {
                            moves.push(MoveOption {
                                site: Some(cell),
                                target: None,
                                ..blank
                            });
                        }
                    }
                    // The closing of each source they hold sickens that none in force closes
                    // (M6a slice BA, step four).
                    PolicyKind::CloseSource => {
                        let closed = self.polities[pi].closed_now();
                        for s in self.word.suspicions_of(d) {
                            if let crate::uses::Place::Source(cell) = s.source
                                && closed.binary_search_by_key(&s.source, |c| c.0).is_err()
                            {
                                moves.push(MoveOption {
                                    site: Some(cell),
                                    target: None,
                                    ..blank
                                });
                            }
                        }
                    }
                    // A well beside the hearth, where the settlement's people could dig one
                    // together (M6a slice BA, step four).
                    PolicyKind::DigWell => {
                        if let Some(p) = &fc.well {
                            moves.push(MoveOption {
                                site: Some(p.site.cell),
                                target: None,
                                ..blank
                            });
                        }
                    }
                    // A flood bank, where the settlement's people could build one together (M6b
                    // slice BG), named by the cell at its middle.
                    PolicyKind::BuildBank => {
                        if let Some(p) = &fc.bank {
                            moves.push(MoveOption {
                                site: Some(p.cell(ctx.map)),
                                target: None,
                                ..blank
                            });
                        }
                    }
                    // A palisade round the hearth (M6c slice BK), named by the hearth's cell.
                    PolicyKind::Enclose => {
                        if let Some(p) = &fc.enclosure {
                            moves.push(MoveOption {
                                site: Some(cell_of(ctx.map, p.site.centre) as u32),
                                target: None,
                                ..blank
                            });
                        }
                    }
                    // Keeping a bank the polity built that wants mending (M6b slice BG), named
                    // by the cell at its middle.
                    PolicyKind::KeepBank => {
                        if let Some(p) = &fc.worn {
                            moves.push(MoveOption {
                                site: Some(p.cell_at),
                                target: None,
                                ..blank
                            });
                        }
                    }
                    // A rule for building, put forward only by someone who knows the technique
                    // it needs (M6b slice BH; research 11-09 §1.4: remedies come from known
                    // techniques).
                    PolicyKind::BuildingRule => {
                        let knows = def.rule.as_ref().is_some_and(|r| {
                            r.needs
                                .is_none_or(|t| self.person(d).is_some_and(|p| p.knows(t)))
                        });
                        if knows {
                            moves.push(blank);
                        }
                    }
                    // Seeking terms with another polity is weighed apart, after these (M5c
                    // slice AU): it is a meeting before it is a law.
                    PolicyKind::Agreement => {}
                    // A limit is weighed only as an amendment is (M7b slice BT).
                    PolicyKind::Limit => {}
                    // A forum (M7b slice BR): one holder the sponsor regards most, themselves at
                    // full regard; a panel or the elders name nobody.
                    PolicyKind::HearCases => {
                        // In place of the forum in force, if one is (M7b slice BS).
                        let blank = MoveOption {
                            ends: self.polities[pi]
                                .laws
                                .iter()
                                .find(|l| {
                                    l.kind == PolicyKind::HearCases
                                        && l.status == LawStatus::InForce
                                })
                                .map(|l| l.id),
                            ..blank
                        };
                        let holder = def
                            .forum
                            .as_ref()
                            .is_some_and(|f| f.seat == crate::forum::ForumSeat::Holder);
                        if holder {
                            let pick = adults
                                .iter()
                                .map(|&(c, _)| {
                                    let r = if c == d {
                                        1.0
                                    } else {
                                        self.ties.regard(d, c, day, tp).clamp(0.0, 1.0)
                                    };
                                    (r, c)
                                })
                                .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
                            if let Some((_, c)) = pick {
                                moves.push(MoveOption {
                                    nominee: Some(c),
                                    ..blank
                                });
                            }
                        } else {
                            moves.push(blank);
                        }
                    }
                    // While founding, an end to each law the old custom made (M4c slice AI): a
                    // store's levy ends as a petition would end it, the store giving what it
                    // holds; any other law is repealed.
                    PolicyKind::Repeal => {
                        for (e, k) in &kept {
                            let store = policies
                                .get(usize::from(k.policy))
                                .is_some_and(|d| d.kind == PolicyKind::CommonStore);
                            moves.push(if store {
                                MoveOption {
                                    policy: k.policy,
                                    ends: Some(*e),
                                    ..blank
                                }
                            } else {
                                MoveOption {
                                    ends: Some(*e),
                                    ..blank
                                }
                            });
                        }
                    }
                    // A watch: the adult they regard most, themselves at full regard (M4b slice
                    // AC), of those who do not keep it already (M4c slice AI).
                    PolicyKind::KeepWatch => {
                        let barred = self.term_barred(pi, k, day, def);
                        let pick = adults
                            .iter()
                            .filter(|&&(c, _)| !watching.contains(&c) && Some(c) != barred)
                            .map(|&(c, _)| {
                                let r = if c == d {
                                    1.0
                                } else {
                                    self.ties.regard(d, c, day, tp).clamp(0.0, 1.0)
                                };
                                (r, c)
                            })
                            .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
                        if let Some((_, c)) = pick {
                            moves.push(MoveOption {
                                nominee: Some(c),
                                ..blank
                            });
                        }
                    }
                }
            }
            // An amendment of the custom (M4c slice AF): open to one whom a gathering they came
            // to within memory overruled, or who holds a grievance against the gathering; each
            // body a template offers that differs from the custom and would keep them in it.
            let seen_d = seen_by(&|p, _| p == d);
            // Those the sponsor expects would sit in each council they weigh (M7a slice BO).
            let mut sat: Vec<(Body, Vec<PermanentId>)> = Vec::new();
            let overruled = seen_d.iter().any(|&i| {
                let s = &decisions[i];
                s.stances.iter().any(|r| {
                    r.0 == d
                        && match r.2 {
                            Stance::Support => !s.passed,
                            Stance::Oppose => s.passed,
                            Stance::Abstain => false,
                        }
                })
            }) || self
                .word
                .grievances_of(d)
                .any(|g| g.blamed == Blamed::Body(polity_id) && g.unresolved_days > 0.0);
            if overruled {
                // A limit, as an amendment is (M7b slice BT, ADR-0025 §3): one forbidding a law in
                // force.
                for (k, _) in &limit_gains {
                    moves.push(MoveOption {
                        policy: *k,
                        levy_share: 0.0,
                        issue: IssueKind::Overruled,
                        nominee: None,
                        sanction: Default::default(),
                        hours: (0, 0),
                        body: None,
                        ends: None,
                        site: None,
                        target: None,
                        own_gain: 0.0,
                        followers_gain: 0.0,
                        support: 0.5,
                    });
                }
                for (k, def) in ctx.catalog.policies.iter().enumerate() {
                    if def.kind != PolicyKind::AmendBody {
                        continue;
                    }
                    for &b in &def.bodies {
                        // Seats shared among settlements only where the polity holds several
                        // (M7a slice BQ).
                        if b.seats.is_some_and(|s| s.by_settlement)
                            && self.polities[pi].held.is_empty()
                        {
                            continue;
                        }
                        // A council keeps them in it if they may stand for it (M7a slice
                        // BO): its electorate.
                        let gate = b.seats.map_or(b.members, |s| s.electorate);
                        let keeps = admitted_by[usize::from(gate.code())]
                            .binary_search(&d)
                            .is_ok();
                        // One change at a time (research 09-05 §2.3: a sponsor weighs a few
                        // alternatives; §3.9: reform changes a gate within the procedure).
                        if b.changes_from(&current) == 1 && keeps {
                            if let Some(seats) = b.seats {
                                let electorate = &admitted_by[usize::from(seats.electorate.code())];
                                sat.push((
                                    b,
                                    self.forecast_sitters(ctx, pi, &seats, electorate, d),
                                ));
                            }
                            moves.push(MoveOption {
                                policy: k as u16,
                                levy_share: 0.0,
                                issue: IssueKind::Overruled,
                                nominee: None,
                                sanction: Default::default(),
                                hours: (0, 0),
                                body: Some(b),
                                ends: None,
                                site: None,
                                target: None,
                                own_gain: 0.0,
                                followers_gain: 0.0,
                                support: 0.5,
                            });
                        }
                    }
                }
            }
            let sitters_of = |b: &Body| sat.iter().find(|x| x.0 == *b).map(|x| x.1.as_slice());
            let gain_for = |m: &MoveOption, h: PermanentId| match (m.body, m.ends) {
                (Some(b), _) => amend_gain(&b, sitters_of(&b), h, &seen_d),
                (None, Some(_)) => -held(m, h),
                (None, None) => limit_gain(m, h).unwrap_or_else(|| gain_of(m, h)),
            };
            for m in &mut moves {
                // Those who regard them, by how much; and those they know, by where they would
                // stand with them as its sponsor (or with the one it names).
                let face = m.nominee.unwrap_or(d);
                let (mut weighed, mut weight, mut support, mut oppose) = (0.0, 0.0, 0, 0);
                for &(a, h) in &members {
                    if a == d {
                        continue;
                    }
                    let gain = gain_for(m, h);
                    let regard = self.ties.regard(a, d, day, tp).clamp(0.0, 1.0);
                    if regard > 0.0 {
                        weighed += regard * gain;
                        weight += regard;
                    }
                    if self.ties.known(d, a, day, tp) > 0.0 {
                        let to_face = if face == a {
                            1.0
                        } else {
                            self.ties.regard(a, face, day, tp)
                        };
                        match crate::polity::stance(pp.w_gain * gain, to_face, pp).0 {
                            Stance::Support => support += 1,
                            Stance::Oppose => oppose += 1,
                            Stance::Abstain => {}
                        }
                    }
                }
                // What it does to what they hold dear weighs beside their household's lot (M4c
                // slice AG); nobody sees another's, so those they count on are judged by their
                // households' lots alone.
                // And the ideology they hold that proposes it, if one does (M4c slice AG). An
                // end to a law weighs what that law does to them, turned about (M4c slice AI).
                let (policy, sign) = ended(m).map_or((m.policy, 1.0), |k| (k.1.policy, -1.0));
                let creed = self.creed_for(ctx, d, policy).map_or(0.0, |c| c.1);
                m.own_gain = gain_for(m, own)
                    + sign * (self.value_points(ctx, d, policy) + creed) / pp.w_gain.max(1e-9);
                m.followers_gain = if weight > 0.0 { weighed / weight } else { 0.0 };
                m.support = if support + oppose > 0 {
                    f64::from(support) / f64::from(support + oppose)
                } else {
                    0.5
                };
                // What they saw (research 09-05 §1.2: a sponsor weighs expected success): one who
                // came to a gathering that decided the same proposal, within memory, expects no
                // more support for it than it got there.
                let seen = self.polities[pi]
                    .laws
                    .iter()
                    .rev()
                    .filter(|l| {
                        l.policy == m.policy
                            && l.holder == m.nominee
                            && l.sanction == m.sanction
                            && l.hours == m.hours
                            && l.body == m.body
                    })
                    .filter(|l| (f64::from(l.levy_share) - m.levy_share).abs() < 1e-4)
                    .filter(|l| m.ends.is_none() || l.ends == m.ends)
                    .filter(|l| {
                        m.site.is_none()
                            || self.polities[pi].site_of(l.id).map(|s| s.cell) == m.site
                            || self.polities[pi].closure_of(l.id)
                                == m.site.map(crate::uses::Place::Source)
                            || self.polities[pi].well_site_of(l.id).map(|s| s.cell) == m.site
                            || self.polities[pi]
                                .bank_site_of(l.id)
                                .or_else(|| self.polities[pi].kept_bank_of(l.id).map(|k| k.1))
                                .map(|s| super::levees::bank_cell(ctx.map, &s))
                                == m.site
                    })
                    .filter(|l| {
                        l.decided
                            .is_some_and(|t| day - t.day_index() <= i64::from(pp.vote_memory_days))
                    })
                    .find(|l| l.stances.iter().any(|r| r.person == d));
                if let Some(l) = seen {
                    let (_, s, o) = l.counts();
                    if s + o > 0 {
                        m.support = m.support.min(f64::from(s) / f64::from(s + o));
                    }
                }
            }
            // An amendment is weighed only by one it would have served (research 09-02 §3.7:
            // institutional change is proposed for an expected benefit): with nothing to gain,
            // changing the custom is no move of theirs. So is a further watcher (M4c slice AI).
            let further = |m: &MoveOption| {
                policies
                    .get(usize::from(m.policy))
                    .is_some_and(|d| d.kind == PolicyKind::KeepWatch)
                    && !watching.is_empty()
            };
            // So is a limit (M7b slice BT).
            let limit = |m: &MoveOption| limit_gains.iter().any(|l| l.0 == m.policy);
            moves.retain(|m| (m.body.is_none() && !further(m) && !limit(m)) || m.own_gain > 0.0);
            let u =
                Rng64::from_key(&[ctx.seed, PURPOSE_DELIBERATE, d.get(), day as u64]).next_f64();
            let choice = deliberator.choose(&moves, u);
            if let Some(m) = choice.chosen.map(|i| moves[i]) {
                let stakes: Vec<(PermanentId, f32)> = outlooks
                    .iter()
                    .map(|(h, _)| {
                        // Each household judges an amendment by the decisions its own members
                        // saw (M4c slice AF).
                        let gain = match (m.body, m.ends) {
                            (Some(b), _) => {
                                amend_gain(&b, sitters_of(&b), *h, &seen_by(&|_, hh| hh == *h))
                            }
                            (None, Some(_)) => -held(&m, *h),
                            (None, None) => limit_gain(&m, *h).unwrap_or_else(|| gain_of(&m, *h)),
                        };
                        (*h, (pp.w_gain * gain) as f32)
                    })
                    .collect();
                let creed = self.creed_for(ctx, d, m.policy).map(|c| c.0);
                self.propose(ctx, pi, d, m, stakes, creed);
                return;
            }
        }
    }

    /// `sponsor` puts move `m` to the gathering (ADR-0013 §3, stage 1): the law is proposed, the
    /// sponsor knows it, and a gathering is called for `notice_days` on.
    pub(super) fn propose(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        sponsor: PermanentId,
        m: MoveOption,
        stakes: Vec<(PermanentId, f32)>,
        creed: Option<u16>,
    ) {
        let (now, pp) = (ctx.now, &ctx.params.polity);
        let day = now.day_index();
        let def = &ctx.catalog.policies[usize::from(m.policy)];
        let id = ctx.ids.allocate();
        let meets = day + i64::from(pp.notice_days.max(1));
        let law = Law {
            id,
            policy: m.policy,
            kind: def.kind,
            levy_share: m.levy_share as f32,
            holder: m.nominee,
            relief_days: def.relief_days as f32,
            sanction: m.sanction,
            hours: m.hours,
            status: LawStatus::Proposed,
            sponsor,
            proposed: now,
            issue: m.issue,
            meets_day: meets,
            decided: None,
            outcome: None,
            eligible: 0,
            stances: Vec::new(),
            known: vec![(sponsor, day)],
            compliance: Default::default(),
            watch: Default::default(),
            body: m.body,
            ends: m.ends,
            agreement: None,
            target: m.target,
        };
        // A crossing names its site, as the settlement's people would build it now (M5c slice
        // AW, step three).
        if def.kind == PolicyKind::BuildCrossing
            && let Some(cell) = m.site
        {
            let settlement = self.polities[pi].settlement;
            let mut households: Vec<PermanentId> = self
                .households
                .iter()
                .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
                .map(|(_, x)| x.id)
                .collect();
            households.sort_unstable();
            if let Some(p) = self
                .public_sites(ctx, settlement, &households)
                .into_iter()
                .find(|p| p.site.cell == cell)
            {
                self.polities[pi].sites.push((id, p.site));
            }
        }
        // A well names its site, as the settlement's people would dig it now (M6a slice BA, step
        // four).
        if def.kind == PolicyKind::DigWell
            && let Some(cell) = m.site
        {
            let settlement = self.polities[pi].settlement;
            let mut households: Vec<PermanentId> = self
                .households
                .iter()
                .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
                .map(|(_, x)| x.id)
                .collect();
            households.sort_unstable();
            if let Some(p) = self
                .public_well(ctx, settlement, &households)
                .filter(|p| p.site.cell == cell)
            {
                self.polities[pi].wells.push((id, p.site));
            }
        }
        // A bank names its site, as the settlement's people would build it now, and keeps what
        // each household reckoned it would save it (M6b slice BG).
        if def.kind == PolicyKind::BuildBank
            && let Some(cell) = m.site
        {
            let settlement = self.polities[pi].settlement;
            let mut households: Vec<PermanentId> = self
                .households
                .iter()
                .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
                .map(|(_, x)| x.id)
                .collect();
            households.sort_unstable();
            if let Some(p) = self
                .public_bank(ctx, settlement, &households)
                .filter(|p| p.cell(ctx.map) == cell)
            {
                let polity = &mut self.polities[pi];
                polity.banks.push((id, p.site));
                for &(h, saves) in &p.saves {
                    polity.bank_saves.push((id, h, saves as f32));
                }
                polity.bank_saves.sort_unstable_by_key(|s| (s.0, s.1));
            }
        }
        // A palisade names its line, as the settlement's people would raise it now, and keeps
        // the hours a year each household reckoned it would keep it: the food it believes kept,
        // at what an hour of its work brings it, less its walk round to a gate (M6c slice BK).
        if def.kind == PolicyKind::Enclose && m.site.is_some() {
            let settlement = self.polities[pi].settlement;
            let mut households: Vec<PermanentId> = self
                .households
                .iter()
                .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
                .map(|(_, x)| x.id)
                .collect();
            households.sort_unstable();
            if let Some(p) = self.public_enclosure(ctx, settlement, &households) {
                let saves: Vec<(PermanentId, f32)> = p
                    .keeps
                    .iter()
                    .map(|&(h, kept, detour)| {
                        let kcal = self
                            .household(h)
                            .map_or(0.0, |x| self.hour_kcal(ctx, x))
                            .max(1e-9);
                        (h, (kept / kcal - detour).max(0.0) as f32)
                    })
                    .collect();
                let polity = &mut self.polities[pi];
                polity.enclosures.push((id, p.site));
                for (h, s) in saves {
                    polity.enclosure_saves.push((id, h, s));
                }
                polity.enclosure_saves.sort_unstable_by_key(|s| (s.0, s.1));
            }
        }
        // A keeping names the bank it keeps, one that wants mending and no law keeps (M6b slice
        // BG).
        if def.kind == PolicyKind::KeepBank
            && let Some(cell) = m.site
        {
            let households = self
                .households
                .iter()
                .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
                .count();
            if let Some(p) = self
                .worn_bank(ctx, pi, households)
                .filter(|p| p.cell_at == cell)
            {
                self.polities[pi].bank_keeps.push((id, p.built));
            }
        }
        // A closing names the source its sponsor suspects (M6a slice BA, step four).
        if def.kind == PolicyKind::CloseSource
            && let Some(cell) = m.site
        {
            self.polities[pi]
                .closed
                .push((id, crate::uses::Place::Source(cell)));
        }
        // A claim names the places the settlement's people saw outsiders at, unclaimed (M5c
        // slice AT): named before it is told, so its words count them.
        if def.kind == PolicyKind::ClaimPlace {
            let mut households: Vec<PermanentId> = self
                .households
                .iter()
                .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
                .map(|(_, x)| x.id)
                .collect();
            households.sort_unstable();
            let places = self.unclaimed_outsider_places(ctx, pi, &households);
            self.polities[pi]
                .claimed
                .extend(places.into_iter().map(|p| (id, p)));
        }
        // "Mira proposed themselves as keeper of the common store", not "Mira as keeper"; and
        // the creed they proposed it under, if any (M4c slice AG).
        let held = creed
            .and_then(|k| ctx.catalog.ideologies.get(usize::from(k)))
            .map(|d| format!(", as one who holds to {}", d.name))
            .unwrap_or_default();
        let what = self.closing_words(ctx, pi, &law).unwrap_or_else(|| {
            self.polities[pi].words_of(&law, &ctx.catalog.policies, &|id| {
                if id == sponsor {
                    "themselves".to_owned()
                } else {
                    self.name_of(id)
                }
            })
        });
        let words = format!("{what}, because {}{held}", m.issue.words());
        if let Some(k) = creed {
            self.ideologies.note_creed(id, k);
        }
        let polity = &mut self.polities[pi];
        polity.laws.push(law);
        polity.gathering = Some(Gathering {
            law: Some(id),
            day: meets,
            stakes,
            present: Vec::new(),
            cases: Vec::new(),
        });
        let settlement = polity.settlement;
        // The sponsor knows it is called; everyone else hears by word (M4c slice AE).
        self.call_word(settlement, meets, Some(id), &[sponsor], day);
        // Its sponsor may give before it meets (M7c slice BY).
        self.patron_gives(ctx, pi, id);
        let place = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map(|s| s.hearth_m);
        self.chronicle_push(
            now,
            ChronicleKind::LawProposed,
            vec![sponsor],
            Some(settlement),
            place,
            m.levy_share,
            words,
        );
    }

    /// The gathering called at polity `pi` has sat: it decides the law before it, if any, and
    /// then hears its cases in the order they were brought (M4b slice AB).
    fn decide_gathering(&mut self, ctx: &mut Ctx, pi: usize) {
        let Some(g) = self.polities[pi].gathering.take() else {
            return;
        };
        if let Some(law) = g.law {
            self.decide_law(ctx, pi, &g, law);
        }
        for &case in &g.cases {
            self.hear_case(ctx, pi, &g.present, None, case);
        }
    }

    /// The gathering called at polity `pi` has sat on law `law_id` (ADR-0013 §3, stages 2-4):
    /// each member who came takes a stance from what their household made of the law and their
    /// regard for its sponsor, the body decides by its rule, and if it passed, those who came know
    /// it. A thin gathering or a tie is recorded as what it was.
    fn decide_law(&mut self, ctx: &mut Ctx, pi: usize, g: &Gathering, law_id: PermanentId) {
        let (now, params) = (ctx.now, ctx.params);
        let pp = &params.polity;
        let settlement = self.polities[pi].settlement;
        // Its body's members now (M4c slice AF: the custom says who they are).
        let members = self.body_members(&ctx.land.fields, pi, now, params);
        let Some(law) = self.polities[pi].laws.iter().find(|l| l.id == law_id) else {
            return;
        };
        let (sponsor, policy, kind) = (law.sponsor, law.policy, law.kind);
        // An end to a law, a repeal or a store's levy at none (M4c slices AH-AI), weighs what
        // that law does to people, turned about.
        let (policy, sign) = law
            .ends
            .filter(|_| {
                kind == PolicyKind::Repeal
                    || (kind == PolicyKind::CommonStore && law.levy_share <= 0.0)
            })
            .and_then(|e| self.polities[pi].laws.iter().find(|l| l.id == e))
            .map_or((policy, 1.0), |l| (l.policy, -1.0));
        let w_position = params.opinion.w_position;
        // Those who came weigh their regard for the one the law names, or else for its sponsor;
        // for one named in place of an office's holder, less their regard for that holder, which
        // a grievance against the office lowers (M4c slice AH; research 04-10 §1.1: blame).
        let face = law.holder.unwrap_or(sponsor);
        let replaced = law
            .ends
            .and_then(|e| self.polities[pi].laws.iter().find(|l| l.id == e))
            .filter(|l| l.status == LawStatus::InForce)
            .and_then(|l| Some((l.holder?, l.id)));
        let day = now.day_index();
        // An agreement is weighed with what each believes of the other polity, as regard for its
        // sponsor is (M5c slice AU, ADR-0020 §3).
        let other = law
            .agreement
            .and_then(|a| self.agreements.of_law(law_id).filter(|x| x.0.id == a))
            .map(|(a, s)| a.polities[1 - usize::from(s)]);
        // The store spoils under the roof it had until a new keeper takes it.
        let sheltered = self.polity_sheltered(pi);
        self.polities[pi].settle_stores(now, &ctx.catalog.goods, sheltered);
        let stances: Vec<StanceRecord> = g
            .present
            .iter()
            .filter_map(|&p| {
                let i = members.binary_search_by_key(&p, |m| m.0).ok()?;
                let household = members[i].1;
                let gain = g.stake(household);
                let regard = if p == face {
                    1.0
                } else {
                    self.ties.regard(p, face, day, &params.ties)
                };
                // How far talk at the hearth has moved them from their household's lot (M4c
                // slice AG).
                let talk = sign * self.opinion_points(p, policy, w_position);
                // And what it does to what they hold dear (M4c slice AG).
                let values = sign * self.value_points(ctx, p, policy);
                let view = other.map_or(0.0, |o| {
                    let rp = &params.relations;
                    let warmth = self
                        .polity_views
                        .of(p, o, day, rp)
                        .map_or(0.0, |v| v.warmth());
                    pp.w_regard * warmth
                });
                let held = replaced.map_or(0.0, |(o, office)| {
                    if p == o {
                        1.0
                    } else {
                        self.ties.regard(p, o, day, &params.ties).clamp(0.0, 1.0)
                            - self.keenness(ctx, p, Blamed::Office(office))
                    }
                });
                // And what their household owes the sponsor for gifts (M7c slice BY).
                let owed = self.owed_points(ctx, pi, household, [sponsor, face]);
                let (stance, regard_points) = if p == sponsor {
                    (Stance::Support, 0.0)
                } else {
                    crate::polity::stance_between(
                        gain + values + talk + view + owed,
                        regard,
                        held,
                        pp,
                    )
                };
                Some(StanceRecord {
                    person: p,
                    household,
                    stance,
                    gain: gain as f32,
                    regard: regard_points as f32,
                    opinion: talk as f32,
                    values: values as f32,
                    view: view as f32,
                    owed: owed as f32,
                })
            })
            .collect();
        let polity = &mut self.polities[pi];
        // A change of a limit is decided by the limit's own rule (M7b slice BT, ADR-0025 §3:
        // entrenchment).
        let entrenched = polity
            .laws
            .iter()
            .find(|l| l.id == law_id)
            .and_then(|l| l.ends)
            .and_then(|e| polity.laws.iter().find(|l| l.id == e))
            .and_then(|l| ctx.catalog.policies.get(usize::from(l.policy))?.limit)
            .map(|lim| lim.change);
        let body = Body {
            pass: entrenched.unwrap_or(polity.body.pass),
            ..polity.body
        };
        // A further watcher's rounds start past those of the watchers before them (M4c slice AI).
        let watching = polity.watchers().count() as u32;
        let Some(law) = polity.law_mut(law_id) else {
            return;
        };
        law.stances = stances;
        let (present, support, oppose) = law.counts();
        // A council counts its seats, filled or empty (M7a slice BO, ADR-0024 §1).
        let eligible = body.eligible(members.len());
        let outcome = body.decide(eligible, present, support, oppose);
        law.eligible = eligible;
        law.decided = Some(now);
        law.outcome = Some(outcome);
        law.status = if outcome == Outcome::Passed {
            for p in &g.present {
                law.learn(*p, day);
            }
            if kind == PolicyKind::KeepWatch {
                law.watch.next = watching.wrapping_mul(ctx.params.crime.round_stops.max(1));
            }
            // A repeal asks and gives nothing: it ends the law it names (M4c slice AI).
            if kind == PolicyKind::Repeal {
                LawStatus::Carried
            } else {
                LawStatus::InForce
            }
        } else {
            LawStatus::Rejected
        };
        // An office held for a term is filled, by lot where its template says (M7a slice BN).
        if outcome == Outcome::Passed
            && matches!(
                kind,
                PolicyKind::KeepStore | PolicyKind::KeepWatch | PolicyKind::Mint
            )
        {
            self.fill_term(ctx, pi, law_id);
        }
        // A crossing passed is begun at once, the polity's (M5c slice AW, step three).
        if outcome == Outcome::Passed
            && kind == PolicyKind::BuildCrossing
            && let Some(site) = self.polities[pi].site_of(law_id)
        {
            let polity = self.polities[pi].id;
            self.begin_public_crossing(ctx, polity, site);
        }
        // A well passed is begun at once, the polity's (M6a slice BA, step four).
        if outcome == Outcome::Passed
            && kind == PolicyKind::DigWell
            && let Some(site) = self.polities[pi].well_site_of(law_id)
        {
            let polity = self.polities[pi].id;
            self.begin_public_well(ctx, polity, site);
        }
        // A bank passed is begun at once, the polity's (M6b slice BG).
        if outcome == Outcome::Passed && kind == PolicyKind::BuildBank {
            self.begin_public_bank(ctx, pi, law_id);
        }
        // A palisade passed is laid out at once, the polity's (M6c slice BK).
        if outcome == Outcome::Passed && kind == PolicyKind::Enclose {
            self.begin_public_enclosure(ctx, pi, law_id);
        }
        // A settlement held is taken in at once (M6c slice BM).
        if outcome == Outcome::Passed
            && kind == PolicyKind::Annex
            && let Some(at) = self.polities[pi]
                .laws
                .iter()
                .find(|l| l.id == law_id)
                .and_then(|l| l.target)
        {
            self.annex_passed(ctx, pi, law_id, at);
        }
        // A keeping passed looks its bank over at once, and asks its mending (M6b slice BG).
        if outcome == Outcome::Passed
            && kind == PolicyKind::KeepBank
            && let Some((built, _)) = self.polities[pi].kept_bank_of(law_id)
        {
            self.ask_bank_mending(ctx, pi, built);
        }
        // A law that replaces one in force supersedes it (M4c slice AH).
        if outcome == Outcome::Passed
            && let Some(old) = self.polities[pi]
                .laws
                .iter()
                .find(|l| l.id == law_id)
                .and_then(|l| l.ends)
            && let Some(o) = self.polities[pi].law_mut(old)
            && o.status == LawStatus::InForce
        {
            o.status = LawStatus::Superseded;
        }
        let law = &self.polities[pi].laws[self.polities[pi]
            .laws
            .iter()
            .position(|l| l.id == law_id)
            .unwrap_or(0)];
        let what = self.closing_words(ctx, pi, law).unwrap_or_else(|| {
            self.polities[pi].words_of(law, &ctx.catalog.policies, &|id| self.name_of(id))
        });
        let (place, name) = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map_or((None, String::new()), |s| {
                (Some(s.hearth_m), s.name.clone())
            });
        let who = body.members.noun();
        let tally = format!("{support} for, {oppose} against; {present} of {eligible} {who} came");
        let words = match outcome {
            Outcome::Passed => format!("The gathering at {name} agreed to {what}: {tally}."),
            Outcome::Failed => format!("The gathering at {name} turned down {what}: {tally}."),
            Outcome::Tied => format!(
                "The gathering at {name} was evenly split on {what}, so it failed: {tally}."
            ),
            Outcome::NoQuorum => format!(
                "Too few came to the gathering at {name} to decide on {what}: {present} of \
                 {eligible} {who}, where {} were needed.",
                body.quorum(eligible)
            ),
        };
        self.chronicle_push(
            now,
            ChronicleKind::LawDecided,
            vec![sponsor],
            Some(settlement),
            place,
            f64::from(outcome as u8),
            words,
        );
        // An amendment passed changes the custom by its own procedure (M4c slice AF, ADR-0017
        // §1): a new version, the one it replaces superseded, told as an amendment.
        let amended = self.polities[pi]
            .laws
            .iter()
            .find(|l| l.id == law_id)
            .filter(|l| l.outcome == Some(Outcome::Passed) && l.kind == PolicyKind::AmendBody)
            .and_then(|l| l.body);
        if let Some(new) = amended {
            let polity = &mut self.polities[pi];
            for l in &mut polity.laws {
                if l.kind == PolicyKind::AmendBody
                    && l.status == LawStatus::InForce
                    && l.id != law_id
                {
                    l.status = LawStatus::Superseded;
                }
            }
            polity.body = new;
            polity.versions.push(CustomVersion {
                body: new,
                since: now,
                law: Some(law_id),
                seized_by: None,
            });
            let words = format!(
                "The custom at {name} changed by its own procedure, on {}'s proposal: from now \
                 on, {}.",
                self.name_of(sponsor),
                new.clause()
            );
            self.chronicle_push(
                now,
                ChronicleKind::CustomAmended,
                vec![sponsor],
                Some(settlement),
                place,
                f64::from(self.polities[pi].versions.len() as u32),
                words,
            );
        }
    }

    /// Those at home hear of the laws in force their households know (ADR-0013 §3, stage 4):
    /// nobody keeps a law from their own household.
    fn tell_households(&mut self, day: i64) {
        for pi in 0..self.polities.len() {
            if !self.polities[pi]
                .laws
                .iter()
                .any(|l| l.status == LawStatus::InForce)
            {
                continue;
            }
            let mut homes: Vec<(PermanentId, Vec<PermanentId>)> = self
                .households
                .iter()
                .filter(|(_, x)| self.polities[pi].holds(x.settlement) && x.members.len() > 1)
                .map(|(_, x)| (x.id, x.members.clone()))
                .collect();
            homes.sort_unstable_by_key(|h| h.0);
            for law in &mut self.polities[pi].laws {
                if law.status != LawStatus::InForce {
                    continue;
                }
                for (_, members) in &homes {
                    if members.iter().any(|&m| law.knows(m)) {
                        for &m in members {
                            law.learn(m, day);
                        }
                    }
                }
            }
        }
    }

    /// `a` and `b` kept company at the hearth: each tells the other the laws in force there they
    /// know (ADR-0013 §3, stage 4).
    pub(crate) fn share_laws(&mut self, a: PermanentId, b: PermanentId, day: i64) {
        let Some(settlement) = self
            .person(a)
            .and_then(|p| self.household(p.household))
            .and_then(|x| x.settlement)
        else {
            return;
        };
        let Some(pi) = self.polity_of(settlement) else {
            return;
        };
        for law in &mut self.polities[pi].laws {
            if law.status != LawStatus::InForce {
                continue;
            }
            match (law.knows(a), law.knows(b)) {
                (true, false) => {
                    law.learn(b, day);
                }
                (false, true) => {
                    law.learn(a, day);
                }
                _ => {}
            }
        }
    }

    /// What attending the gathering sitting now in the settlement of household `hh` is worth to
    /// `person`, aged `age`, at `minute` of the day, with the evening from `evening_start`: what
    /// their household made of the law and their regard for its sponsor, over the custom's own
    /// pull. `None` when no gathering of theirs sits, they are not a member, or they are there.
    pub(crate) fn gathering_facts(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        age: f64,
        hh: &Household,
        minute: i64,
        evening_start: i64,
    ) -> Option<GatheringFacts> {
        let params = ctx.params;
        let pp = &params.polity;
        if age < params.family.independent_age {
            return None;
        }
        let pi = self.polity_of(hh.settlement?)?;
        let polity = &self.polities[pi];
        let g = polity.gathering.as_ref()?;
        let end = evening_start + i64::from(pp.gathering_minutes);
        if g.day != ctx.now.day_index()
            || minute < evening_start
            || minute + LEAST_SITTING_MIN > end
            || g.present.binary_search(&person).is_ok()
            // Only those who heard it is called come (M4c slice AE, ADR-0016 §3).
            || !self.heard_of_gathering(person, polity.settlement, g.day)
            // And only the body's members (M4c slice AF).
            || !self.admits(&ctx.land.fields, pi, person, hh.id, ctx.now, params)
        {
            return None;
        }
        let day = ctx.now.day_index();
        let regard_for = |q: PermanentId| {
            if q == person {
                1.0
            } else {
                self.ties
                    .regard(person, q, day, &params.ties)
                    .clamp(0.0, 1.0)
            }
        };
        // What the law means to their household, and their regard for its sponsor; and for each
        // case, their regard for whichever party they regard more (their own household's case
        // counts as full regard).
        let law_stake = g
            .law
            .and_then(|id| polity.laws.iter().find(|l| l.id == id))
            .map_or(0.0, |l| {
                g.stake(hh.id).abs() + pp.w_regard * regard_for(l.sponsor)
            });
        let case_stake = g
            .cases
            .iter()
            .filter_map(|&c| self.order.case(c))
            .map(|c| {
                let party = hh.id == c.accuser || hh.id == c.accused_household;
                let r = if party {
                    1.0
                } else {
                    regard_for(c.by).max(regard_for(c.accused))
                };
                pp.w_regard * r
            })
            .fold(0.0, f64::max);
        let stake = law_stake.max(case_stake);
        Some(GatheringFacts {
            points: pp.attend_base + pp.w_attend * stake,
            minutes: (end - minute) as f64,
        })
    }

    /// A curfew in force where household `hh` lives and running at minute `minute` of the day
    /// (M4b slice AD): its polity and law, by index, and, if `person` knows of it, the points
    /// keeping it weighs with them (research 09-06 §1.5: the norms they hold that the gathering
    /// binds, where they stood on it and their regard for its sponsor, as a levy's; never below
    /// 0).
    pub(crate) fn curfew_now(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        hh: &Household,
        minute: i64,
    ) -> Option<(usize, usize, Option<f64>)> {
        let pi = self.polity_of(hh.settlement?)?;
        let laws = &self.polities[pi].laws;
        let li = laws.iter().position(|l| {
            (l.status == LawStatus::InForce || self.keeps_struck(ctx, person, l))
                && l.kind == PolicyKind::Curfew
                && crate::polity::within_hours(l.hours, minute)
        })?;
        let law = &laws[li];
        let points = law.knows(person).then(|| {
            let stance = law
                .stances
                .iter()
                .find(|r| r.person == person)
                .map_or(0.0, |r| r.stance.sign());
            let regard =
                self.ties
                    .regard(person, law.sponsor, ctx.now.day_index(), &ctx.params.ties);
            let norm = self.norm_points(ctx, person);
            crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity).max(0.0)
        });
        Some((pi, li, points))
    }

    /// Whether `person` still keeps law `l`, struck down, as if it bound them (M7b slice BU,
    /// ADR-0025 §5: a ruling is known person by person): they knew the law, have not heard the
    /// finding that struck it, and it was struck within memory.
    pub(super) fn keeps_struck(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        l: &crate::polity::Law,
    ) -> bool {
        self.still_keeps(
            person,
            l,
            ctx.now.day_index(),
            ctx.params.crime.remember_days,
        )
    }

    /// As [`Population::keeps_struck`], on day `today`, a ruling remembered `remember_days`.
    pub fn still_keeps(
        &self,
        person: PermanentId,
        l: &crate::polity::Law,
        today: i64,
        remember_days: u32,
    ) -> bool {
        if l.status != LawStatus::StruckDown || !l.knows(person) {
            return false;
        }
        let since = today - i64::from(remember_days);
        let Some(ruling) = self.order.cases.iter().rev().find(|c| {
            c.law == l.id && c.challenge.is_some() && c.stage == crate::crime::CaseStage::Found
        }) else {
            return false;
        };
        if ruling.heard.is_none_or(|t| t.day_index() < since) {
            return false;
        }
        !self.word.heard_by(person).iter().any(|h| {
            self.word.claim(h.claim).is_some_and(|c| {
                c.kind == crate::word::ClaimKind::Finding && c.event == Some(ruling.id)
            })
        })
    }

    /// `person` sits down at the gathering of their settlement.
    pub(crate) fn attend(&mut self, ctx: &Ctx, person: PermanentId) {
        let Some(settlement) = self
            .person(person)
            .and_then(|p| self.household(p.household))
            .and_then(|x| x.settlement)
        else {
            return;
        };
        let Some(pi) = self.polity_of(settlement) else {
            return;
        };
        let day = ctx.now.day_index();
        if let Some(g) = self.polities[pi]
            .gathering
            .as_mut()
            .filter(|g| g.day == day)
            && let Err(at) = g.present.binary_search(&person)
        {
            g.present.insert(at, person);
        }
    }

    /// `who` of household `household` threshed `grain_kg` of grain (good `grain`): under each
    /// common store in force where they live, they pay its share into the store, or cannot, or
    /// keep it back, or do not know they owe it (ADR-0013 §3, stages 6-7). Paying is their
    /// choice: the custom that the gathering binds, where they stood on it and their regard for
    /// its sponsor, against what it costs their household.
    pub(crate) fn levy(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        household: PermanentId,
        grain_kg: f64,
        grain: usize,
    ) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let pp = &params.polity;
        let Some(pi) = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return;
        };
        let laws: Vec<usize> = (0..self.polities[pi].laws.len())
            .filter(|&li| {
                let l = &self.polities[pi].laws[li];
                (l.status == LawStatus::InForce || self.keeps_struck(ctx, who, l))
                    && ctx
                        .catalog
                        .policies
                        .get(usize::from(l.policy))
                        .is_some_and(|d| d.kind == PolicyKind::CommonStore)
            })
            .collect();
        let kcal_per_kg = goods.get(grain).map_or(0.0, |g| g.kcal_per_kg);
        let day = now.day_index();
        for li in laws {
            let law = &self.polities[pi].laws[li];
            let owed = grain_kg * f64::from(law.levy_share);
            if owed <= 1e-9 {
                continue;
            }
            let (store, sponsor) = (self.polities[pi].id, law.sponsor);
            if !law.knows(who) {
                self.polities[pi].laws[li].compliance.unaware += 1;
                continue;
            }
            let stance = law
                .stances
                .iter()
                .find(|r| r.person == who)
                .map_or(0.0, |r| r.stance.sign());
            let Some(outlook) = self.outlook(ctx, household) else {
                continue;
            };
            let kcal = owed * kcal_per_kg;
            if crate::polity::cannot_pay(outlook.held, kcal, outlook.year_need, pp) {
                let c = &mut self.polities[pi].laws[li].compliance;
                c.could_not += 1;
                c.withheld_kg += owed;
                continue;
            }
            let regard = self.ties.regard(who, sponsor, day, &params.ties);
            let cost = pp.w_gain * crate::polity::levy_cost(&outlook, kcal, pp);
            // What the norms they hold add (M4c slice AG): someone new holds theirs now.
            self.ensure_norm_state(ctx, who);
            let norm = self.norm_points(ctx, who);
            // A faction's refusal of this levy they heard of weighs toward keeping it back,
            // openly (M4c slice AH, step three).
            let (law, settlement) = (self.polities[pi].laws[li].id, self.polities[pi].settlement);
            let refusal = self.refusal_points(ctx, who, settlement, law);
            let points = crate::polity::comply_points(norm, stance, regard, cost, pp)
                - refusal.map_or(0.0, |r| r.1);
            let chance = crate::polity::comply_chance(points);
            let key = [ctx.seed, PURPOSE_LEVY, who.get(), now.minutes() as u64];
            if Rng64::from_key(&key).next_f64() >= chance {
                let c = &mut self.polities[pi].laws[li].compliance;
                match refusal {
                    Some(_) => {
                        c.refused += 1;
                        c.refused_kg += owed;
                    }
                    None => c.evaded += 1,
                }
                c.withheld_kg += owed;
                if let Some((ri, _)) = refusal {
                    self.keep_back_in_refusal(ri, who, owed);
                }
                // What the household did is theirs to tell (M4c slice AG).
                self.norms.note_levy(household, day, false);
                continue;
            }
            let held = self
                .household(household)
                .and_then(|x| x.stores.get(grain))
                .copied()
                .unwrap_or(0.0)
                .max(0.0);
            let paid = owed.min(held);
            let leg = Leg {
                from: household,
                to: store,
                good: grain,
                amount: paid,
            };
            if paid > 1e-9 && self.transfer(now, params, goods, &[leg], Channel::Levy) {
                let c = &mut self.polities[pi].laws[li].compliance;
                c.complied += 1;
                c.levied_kg += paid;
                self.norms.note_levy(household, day, true);
                // A levy taken in a lean year that leaves the household short of a year's food is
                // held against the gathering (M4c slice AE); in an ordinary year it is the
                // custom, however it weighs (research 04-10 §1.1).
                let paid_kcal = paid * kcal_per_kg;
                let short = (outlook.year_need - (outlook.held - paid_kcal)).min(paid_kcal);
                let settlement = self.polities[pi].settlement;
                if short > 0.0 && self.lean_lately(ctx, settlement) {
                    let law = self.polities[pi].laws[li].id;
                    self.grieve_levy(ctx, pi, law, household, short);
                }
            }
        }
    }

    /// The common store of `hh`'s settlement as somewhere to ask for food (ADR-0013 §4), when a
    /// common store is in force there that one of its members knows, and it holds food: as much
    /// as the law allows an ask, up to `want` kcal, at its keeper's home or else at the hearth
    /// `hearth`, walked to as `reach` says.
    pub(crate) fn relief_option(
        &self,
        ctx: &Ctx,
        hh: &Household,
        kcal_day: f64,
        want: f64,
        reach: &TravelField,
        hearth: (f32, f32),
    ) -> Option<GiverOption> {
        let polity = &self.polities[self.polity_of(hh.settlement?)?];
        let at = polity
            .keeper()
            .and_then(|(k, _)| self.person(k))
            .and_then(|p| self.household(p.household))
            .map_or(hearth, |x| x.home);
        let walk_min = f64::from(reach.seconds_to(cell_of(ctx.map, at))?) / 60.0;
        let law = polity
            .in_force(&ctx.catalog.policies, PolicyKind::CommonStore)
            .find(|l| hh.members.iter().any(|&m| l.knows(m)))?;
        let held = stock_kcal(&polity.stores, &ctx.catalog.goods);
        let kcal = held.min(want).min(f64::from(law.relief_days) * kcal_day);
        (kcal > 0.0).then_some(GiverOption {
            household: polity.id,
            walk_min,
            at,
            kcal,
        })
    }

    /// `asker` of household `to` asked polity `pi`'s store for food (ADR-0013 §4): it gives what
    /// the law allows an ask, up to what brings the household to the days of food it tries to
    /// keep and what one person carries, the most perishable first. An ask the store cannot
    /// answer is recorded.
    pub(crate) fn give_relief(&mut self, ctx: &Ctx, pi: usize, to: PermanentId) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(li) = self.polities[pi].laws.iter().position(|l| {
            l.status == LawStatus::InForce
                && ctx
                    .catalog
                    .policies
                    .get(usize::from(l.policy))
                    .is_some_and(|d| d.kind == PolicyKind::CommonStore)
        }) else {
            return;
        };
        let Some(t) = self.household(to) else {
            return;
        };
        let need = t.members.len() as f64 * params.household.daily_kcal_per_person;
        let held = stock_kcal(&stores_now(t, now, params, goods), goods);
        // Relief by favour (M7c slice BY): a low store's keeper passes over one they regard too
        // little, who holds it against them.
        match self.keeper_favours(ctx, pi, to) {
            Some(true) => {
                self.polities[pi].laws[li].compliance.passed_over += 1;
                self.grieve_passed_over(ctx, pi, li, to);
                return;
            }
            Some(false) => self.polities[pi].laws[li].compliance.favoured += 1,
            None => {}
        }
        let law = &self.polities[pi].laws[li];
        let mut want = (f64::from(law.relief_days) * need)
            .min(params.household.food_target_days * need - held)
            .max(0.0);
        if want <= 0.0 {
            return;
        }
        let sheltered = self.polity_sheltered(pi);
        self.polities[pi].settle_stores(now, goods, sheltered);
        let store = self.polities[pi].id;
        let stores = self.polities[pi].stores.clone();
        let mut order: Vec<usize> = (0..goods.len())
            .filter(|&i| {
                goods[i].purpose == GoodUse::Food
                    && goods[i].kcal_per_kg > 0.0
                    && !goods[i].kept_back()
            })
            .collect();
        let keeps = |i: usize| match goods[i].half_life_days {
            h if h > 0.0 => h,
            _ => f64::INFINITY,
        };
        order.sort_by(|&a, &b| keeps(a).total_cmp(&keeps(b)).then(a.cmp(&b)));
        let mut carry = params.household.carry_kg;
        let mut legs = Vec::new();
        for i in order {
            if want <= 0.0 || carry <= 0.0 {
                break;
            }
            let kg = (want / goods[i].kcal_per_kg)
                .min(stores.get(i).copied().unwrap_or(0.0).max(0.0))
                .min(carry);
            if kg <= 1e-9 {
                continue;
            }
            legs.push(Leg {
                from: store,
                to,
                good: i,
                amount: kg,
            });
            carry -= kg;
            want -= kg * goods[i].kcal_per_kg;
        }
        let kg: f64 = legs.iter().map(|l| l.amount).sum();
        let kcal: f64 = legs
            .iter()
            .map(|l| l.amount * goods[l.good].kcal_per_kg)
            .sum();
        let given = !legs.is_empty() && self.transfer(now, params, goods, &legs, Channel::Relief);
        let store_law = self.polities[pi].laws[li].id;
        let c = &mut self.polities[pi].laws[li].compliance;
        if given {
            c.relieved += 1;
            c.relief_kg += kg;
            // What it gave redresses what was held against the store (M4c slice AE).
            self.redress_store(to, store_law, kcal / need.max(1.0));
        } else {
            c.unanswered += 1;
        }
    }
}

/// How near home a place is still on the household's own plot, metres: what a curfew allows (a
/// tuning value, about the reach of a home's own buildings).
pub(crate) const HOME_PLOT_M: f32 = 20.0;

/// Whether doing what `steps` say, from `pos`, takes someone off the plot of their home `home`:
/// a walk anywhere farther, or work where they stand while they are already farther. A walk home
/// is never away.
pub(crate) fn away_from_home(steps: &[Step], pos: (f32, f32), home: (f32, f32)) -> bool {
    let off = |at: (f32, f32)| (at.0 - home.0).hypot(at.1 - home.1) > HOME_PLOT_M;
    let mut walked = false;
    for s in steps {
        if let Step::Walk { to } = s {
            walked = true;
            if off(*to) {
                return true;
            }
        }
    }
    !walked && off(pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn going_out_is_away_and_walking_home_is_not() {
        let home = (100.0, 100.0);
        let walk = |to| Step::Walk { to };
        let work = Step::Work { minutes: 30 };
        // From home to the hearth 40 m off, and back: away.
        assert!(away_from_home(
            &[walk((140.0, 100.0)), work, walk(home)],
            home,
            home
        ));
        // Home from the hearth, to sleep: never away.
        assert!(!away_from_home(&[walk(home), work], (140.0, 100.0), home));
        // Work where they stand: away only if they stand off the plot.
        assert!(away_from_home(&[work], (140.0, 100.0), home));
        assert!(!away_from_home(&[work], (110.0, 100.0), home));
        // A store beside the home is on its plot.
        assert!(!away_from_home(&[walk((115.0, 105.0)), work], home, home));
    }

    #[test]
    fn a_trade_law_brings_sellers_back_buyers_and_costs_the_buyers_their_purchases() {
        use super::super::duties::BoughtAway;
        let id = |n: u64| PermanentId::from_raw(n).expect("nonzero");
        let (buyer, seller, other, there) = (id(1), id(2), id(3), id(9));
        // Two sickles (good 0, a tool) bought by the buyer's household at `there`, one by
        // another's; a unit is worth 1,000 kcal to each; the seller offers tools at home.
        let t = TradeForecast {
            away: vec![
                BoughtAway {
                    household: buyer,
                    market: there,
                    good: 0,
                    units: 2.0,
                    payment: 1,
                    paid: 0.0,
                },
                BoughtAway {
                    household: other,
                    market: there,
                    good: 0,
                    units: 1.0,
                    payment: 1,
                    paid: 0.0,
                },
            ],
            unit: vec![(buyer, 0, 1000.0), (seller, 0, 1000.0), (other, 0, 1000.0)],
            sellers: vec![(seller, GoodUse::Tool)],
            purpose: vec![GoodUse::Tool],
            households: 3,
            duties: Vec::new(),
            orgs: Vec::new(),
            home: Vec::new(),
        };
        // A duty of a quarter: the buyer owes 500 kcal and gets its share of what the store takes
        // (750 / 3); the seller gets back a quarter of the three bought, at its margin of a fifth
        // (150), and its share of the store.
        let (gain, cost) = t.duty(buyer, 0.25, GoodUse::Tool, false, 0.2);
        assert!((cost - 500.0).abs() < 1e-9 && (gain - 250.0).abs() < 1e-9);
        let (gain, cost) = t.duty(seller, 0.25, GoodUse::Tool, false, 0.2);
        assert!(cost == 0.0 && (gain - 400.0).abs() < 1e-9, "{gain}");
        // Spared as one that offers tools, the seller would owe nothing on its own purchases.
        assert_eq!(t.duty(seller, 0.25, GoodUse::Tool, true, 0.2).1, 0.0);
        // On food, nothing.
        assert_eq!(t.duty(buyer, 0.25, GoodUse::Food, false, 0.2), (0.0, 0.0));
        // An embargo on `there` costs the buyer the saving on its two (at the margin); the seller
        // gets the margin on all three back.
        let (gain, cost) = t.embargo(buyer, there, 0.2);
        assert!((cost - 400.0).abs() < 1e-9 && gain == 0.0);
        let (gain, cost) = t.embargo(seller, there, 0.2);
        assert!(cost == 0.0 && (gain - 600.0).abs() < 1e-9);
        // Elsewhere, nothing.
        assert_eq!(t.embargo(buyer, id(8), 0.2), (0.0, 0.0));
        // With a duty of a quarter in force on tools, recognizing an organization the buyer's
        // household belongs to saves it the 500 kcal it owes, and costs each household its share
        // of what the store no longer takes (500 / 3).
        let org = id(20);
        let mut t = t;
        t.duties = vec![(GoodUse::Tool, 0.25, false)];
        t.orgs = vec![(org, GoodUse::Tool, vec![buyer], false)];
        let (gain, cost) = t.recognize(buyer, org);
        assert!((gain - 500.0).abs() < 1e-9 && (cost - 500.0 / 3.0).abs() < 1e-9);
        let (gain, cost) = t.recognize(seller, org);
        assert!(gain == 0.0 && (cost - 500.0 / 3.0).abs() < 1e-9);
        // Once recognized, the buyer's household owes the duty nothing.
        t.orgs[0].3 = true;
        assert_eq!(t.duty(buyer, 0.25, GoodUse::Tool, false, 0.2).1, 0.0);
        // A licence for an organization the seller belongs to: at home the other household sold
        // 4 sickles to the buyer, which the seller gets the margin on (800), and which costs the
        // other household its sales and the buyer what it bought (800 each). A staple: the
        // seller gets the margin on what the other, which also offers tools, fetched (one
        // sickle: 200), which costs that other as much.
        use crate::polity::{Grant, PrivilegeDef};
        t.orgs = vec![(org, GoodUse::Tool, vec![seller], true)];
        t.home = vec![(other, buyer, 0, 4.0)];
        let licence = PrivilegeDef {
            grant: Grant::Licence,
            purpose: GoodUse::Tool,
        };
        assert_eq!(t.privilege(seller, org, licence, 0.2), (800.0, 0.0));
        assert_eq!(t.privilege(other, org, licence, 0.2), (0.0, 800.0));
        assert_eq!(t.privilege(buyer, org, licence, 0.2), (0.0, 800.0));
        t.sellers.push((other, GoodUse::Tool));
        let staple = PrivilegeDef {
            grant: Grant::Staple,
            ..licence
        };
        assert_eq!(t.privilege(seller, org, staple, 0.2), (200.0, 0.0));
        assert_eq!(t.privilege(other, org, staple, 0.2), (0.0, 200.0));
        assert_eq!(t.privilege(buyer, org, staple, 0.2), (0.0, 0.0));
    }
}
