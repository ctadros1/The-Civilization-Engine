//! Taste moving toward admired buildings (M3b slice R; research 11-02 §1.1, §2.2): once a year,
//! each household meets the buildings its settlement finished in the year past, and its taste in
//! building moves a little toward each, the more the more admired it is. Since M5b slice AR it
//! meets too the new buildings of other settlements its people saw there.

use super::*;

use crate::params::{StyleParams, Taste};
use crate::style;

/// Each of `items` (a score and an id) ranked from 0, the lowest, to 1, the highest, ties by id;
/// a lone item ranks 1.
fn ranks(mut items: Vec<(f64, PermanentId)>) -> Vec<(PermanentId, f64)> {
    items.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let n = items.len();
    items
        .iter()
        .enumerate()
        .map(|(i, &(_, id))| {
            let rank = if n > 1 {
                i as f64 / (n - 1) as f64
            } else {
                1.0
            };
            (id, rank)
        })
        .collect()
}

/// A building of the year past that stands whole, as its settlement's households meet it.
struct Met {
    settlement: PermanentId,
    owner: PermanentId,
    id: PermanentId,
    traits: Taste,
    /// How well it was built: its parts' quality, on average.
    built: f64,
}

impl Population {
    /// For a world saved before settlements' founding ways were kept (schema 60 and earlier): a
    /// settlement a founding band made at setup has its band's way, drawn again from its key; any
    /// other its households' taste as it is now, on average (an approximation), or none.
    pub fn derive_founding_ways(&mut self, land: &Land, rules: &StyleParams, seed: u64) {
        for s in &land.settlements {
            let way = if s.founding == civ_land::Founding::Setup {
                Some(style::band_way(rules, seed, s.id))
            } else {
                style::mean_taste(
                    self.households
                        .iter()
                        .filter(|(_, h)| h.settlement == Some(s.id) && !h.members.is_empty())
                        .map(|(_, h)| &h.taste),
                )
            };
            if let Some(way) = way {
                self.founding_ways.insert(s.id, way);
            }
        }
    }

    /// The settlement building `b` stands in: its household's, or once that household is gone,
    /// the one whose hearth is nearest (M5b slice AR).
    pub fn building_settlement(&self, land: &Land, b: &Building) -> Option<PermanentId> {
        self.household(b.household)
            .and_then(|h| h.settlement)
            .or_else(|| {
                let c = crate::build::centre_m(&b.spec);
                land.settlements
                    .iter()
                    .min_by(|x, y| {
                        let d = |p: (f32, f32)| (p.0 - c.0).hypot(p.1 - c.1);
                        d(x.hearth_m)
                            .total_cmp(&d(y.hearth_m))
                            .then(x.id.cmp(&y.id))
                    })
                    .map(|x| x.id)
            })
    }

    /// The first building in another settlement than `b`'s along `b`'s chain of followed
    /// buildings (the building it followed, the one that followed, and so on), if the chain
    /// crosses (M5b slice AR; the M5 diffusion brief §1.7).
    pub fn crossing_of(&self, land: &Land, b: &Building) -> Option<PermanentId> {
        let home = self.building_settlement(land, b);
        let mut at = b.style_from;
        // A chain is short; the bound keeps a cycle from running on.
        for _ in 0..64 {
            let x = land.buildings.iter().find(|x| Some(x.id) == at)?;
            if self.building_settlement(land, x) != home {
                return Some(x.id);
            }
            at = x.style_from;
        }
        None
    }

    /// Settlement `s`'s style on the three clocks (M5b slice AR; [`style::Clocks`]).
    pub fn style_clocks(
        &self,
        catalog: &Catalog,
        land: &Land,
        now: SimTime,
        s: PermanentId,
    ) -> style::Clocks {
        let since = now.minutes() - DAYS_PER_YEAR * MINUTES_PER_DAY;
        let taste = style::Spread::of(
            self.households
                .iter()
                .filter(|(_, h)| h.settlement == Some(s) && !h.members.is_empty())
                .map(|(_, h)| h.taste),
        );
        let mut new = Vec::new();
        let mut stock = Vec::new();
        let mut after = 0;
        for b in &land.buildings {
            if !(b.finished() && b.state == civ_land::BuildingState::Standing) {
                continue;
            }
            if self.building_settlement(land, b) != Some(s) {
                continue;
            }
            let Some(def) = catalog
                .building_index(&b.spec.program)
                .and_then(|i| catalog.buildings.get(i))
            else {
                continue;
            };
            let t = style::traits_of(&b.spec, def);
            stock.push(t);
            if b.stage_since.minutes() >= since {
                new.push(t);
                if self.crossing_of(land, b).is_some() {
                    after += 1;
                }
            }
        }
        style::Clocks {
            way: self.founding_ways.get(&s).copied(),
            taste,
            new: style::Spread::of(new),
            stock: style::Spread::of(stock),
            new_after_elsewhere: after,
        }
    }

    /// `me` stands at `at` in `settlement`, not their own, on a visit or a trip to buy (M5b slice
    /// AR; research 11-02 §1.1: exposure through travel and observed buildings): they see its
    /// buildings finished in the year past that stand whole within the content's `sight_m`, and
    /// keep up to `seen_most` of them in mind, newest first, a building seen again only once.
    pub(crate) fn note_sights(
        &mut self,
        ctx: &Ctx,
        me: PermanentId,
        settlement: PermanentId,
        at: (f32, f32),
    ) {
        let rules = &ctx.params.style;
        if rules.seen_most == 0 || rules.alpha <= 0.0 {
            return;
        }
        let home = self
            .person(me)
            .and_then(|p| self.household(p.household))
            .and_then(|h| h.settlement);
        if home.is_none() || home == Some(settlement) {
            return;
        }
        let since = ctx.now.minutes() - DAYS_PER_YEAR * MINUTES_PER_DAY;
        let reach = rules.sight_m.max(0.0) as f32;
        let mut seen: Vec<(i64, PermanentId)> = ctx
            .land
            .buildings
            .iter()
            .filter(|b| {
                b.finished()
                    && b.state == civ_land::BuildingState::Standing
                    && b.stage_since.minutes() >= since
                    && self.household(b.household).and_then(|h| h.settlement) == Some(settlement)
            })
            .filter(|b| {
                let c = crate::build::centre_m(&b.spec);
                (c.0 - at.0).hypot(c.1 - at.1) <= reach
            })
            .map(|b| (b.stage_since.minutes(), b.id))
            .collect();
        if seen.is_empty() {
            return;
        }
        // Oldest first, so that the newest ends at the front.
        seen.sort_unstable();
        let list = self.seen_away.entry(me).or_default();
        for (_, id) in seen {
            list.retain(|&x| x != id);
            list.insert(0, id);
        }
        list.truncate(rules.seen_most);
    }

    /// Once a year, as wealth is recorded: each household's taste moves toward each building of
    /// its settlement finished in the year past, other than its own, oldest first. It moves
    /// `alpha` of the way for the most admired, and less for others in proportion. A building is
    /// admired from 1 to `prestige_most` times, for its patron and its builders equally (11-02
    /// §1.1: prestige is not wealth alone): its owner's standing in goods among the settlement's
    /// households, and how well it was built among the year's new buildings there, each ranked
    /// from the least to the most. A building that has given way, or any part of it, is admired
    /// by nobody. Each new building is met once, so familiarity saturates. The household
    /// remembers the building that moved its taste most, which its next building follows.
    ///
    /// It meets too, in the same order, the new buildings of other settlements its people saw
    /// there since its last review (M5b slice AR; [`Population::note_sights`]), each once. A
    /// stranger's goods are unknown to it: such a building is admired for how well it was built
    /// among its own settlement's year and for what the household thinks of its owner, the
    /// highest esteem any of its people holds for any of the owner's, in every domain together
    /// (ADR-0014 §3: a tie's evidence above the prior, so a stranger's is nothing), taken as
    /// `e / (e + 1)`. The buildings seen are then let go.
    pub fn review_tastes(
        &mut self,
        catalog: &Catalog,
        params: &PeopleParams,
        land_params: &LandParams,
        land: &Land,
        now: SimTime,
    ) {
        let rules = &params.style;
        if rules.alpha <= 0.0 {
            return;
        }
        let since = SimTime::from_minutes(now.minutes() - DAYS_PER_YEAR * MINUTES_PER_DAY);
        // Each household's standing in goods among its settlement's.
        let mut goods: BTreeMap<PermanentId, Vec<(f64, PermanentId)>> = BTreeMap::new();
        for w in self.wealth(catalog, params, land_params, land, now) {
            goods
                .entry(w.settlement)
                .or_default()
                .push((w.goods_h, w.household));
        }
        let standing: HashMap<PermanentId, f64> = goods.into_values().flat_map(ranks).collect();
        // The year's new buildings that stand whole.
        let mut met: Vec<Met> = Vec::new();
        for b in &land.buildings {
            let sound = b.finished()
                && b.state == civ_land::BuildingState::Standing
                && b.stage_since >= since
                && !b
                    .condition
                    .iter()
                    .any(|c| c.state == civ_land::GroupState::Failed);
            if !sound {
                continue;
            }
            let Some(def) = catalog
                .building_index(&b.spec.program)
                .and_then(|i| catalog.buildings.get(i))
            else {
                continue;
            };
            let Some(settlement) = self.household(b.household).and_then(|h| h.settlement) else {
                continue;
            };
            let built = if b.condition.is_empty() {
                1.0
            } else {
                b.condition
                    .iter()
                    .map(|c| f64::from(c.quality))
                    .sum::<f64>()
                    / b.condition.len() as f64
            };
            met.push(Met {
                settlement,
                owner: b.household,
                id: b.id,
                traits: style::traits_of(&b.spec, def),
                built,
            });
        }
        // How well each was built among its settlement's.
        let mut built: BTreeMap<PermanentId, Vec<(f64, PermanentId)>> = BTreeMap::new();
        for m in &met {
            built.entry(m.settlement).or_default().push((m.built, m.id));
        }
        let craft: HashMap<PermanentId, f64> = built.into_values().flat_map(ranks).collect();
        met.sort_by_key(|m| m.id);
        // What each household's people saw elsewhere, among the year's new buildings, and what
        // it thinks of their owners (M5b slice AR).
        let mut away: HashMap<PermanentId, Vec<(usize, f64)>> = HashMap::new();
        if !self.seen_away.is_empty() {
            let day = now.day_index();
            let index: HashMap<PermanentId, usize> =
                met.iter().enumerate().map(|(i, m)| (m.id, i)).collect();
            let members: HashMap<PermanentId, &[PermanentId]> = self
                .households
                .iter()
                .map(|(_, x)| (x.id, x.members.as_slice()))
                .collect();
            let (ties, seen_away) = (&self.ties, &self.seen_away);
            for (_, h) in self.households.iter() {
                let Some(settlement) = h.settlement else {
                    continue;
                };
                let mut seen: Vec<usize> = h
                    .members
                    .iter()
                    .filter_map(|m| seen_away.get(m))
                    .flatten()
                    .filter_map(|b| index.get(b).copied())
                    .filter(|&i| met[i].settlement != settlement && met[i].owner != h.id)
                    .collect();
                seen.sort_unstable();
                seen.dedup();
                let list = seen
                    .into_iter()
                    .map(|i| {
                        let owners = members.get(&met[i].owner).copied().unwrap_or(&[]);
                        let esteem = h
                            .members
                            .iter()
                            .flat_map(|&m| owners.iter().map(move |&o| (m, o)))
                            .filter_map(|(m, o)| ties.tie(m, o, day, &params.ties))
                            .map(|t| {
                                crate::ties::Domain::ALL
                                    .iter()
                                    .map(|&d| t.esteem(d, &params.ties))
                                    .sum::<f64>()
                            })
                            .fold(0.0, f64::max);
                        (i, esteem / (esteem + 1.0))
                    })
                    .collect::<Vec<_>>();
                if !list.is_empty() {
                    away.insert(h.id, list);
                }
            }
        }
        let most = rules.prestige_most.max(1.0);
        let units = rules.tradition_spread.traits();
        for (_, h) in self.households.iter_mut() {
            let Some(settlement) = h.settlement else {
                continue;
            };
            let mut moved_most: Option<(f64, PermanentId)> = None;
            // Its own settlement's year and what its people saw elsewhere, oldest first: each
            // with the half of its admiration that is its patron's.
            let mut meet: Vec<(usize, f64)> = met
                .iter()
                .enumerate()
                .filter(|(_, m)| m.settlement == settlement && m.owner != h.id)
                .map(|(i, m)| (i, standing.get(&m.owner).copied().unwrap_or(0.0)))
                .collect();
            if let Some(list) = away.get(&h.id) {
                meet.extend(list.iter().copied());
                meet.sort_by_key(|&(i, _)| i);
            }
            for (m, patron) in meet.into_iter().map(|(i, p)| (&met[i], p)) {
                let rank = (patron + craft.get(&m.id).copied().unwrap_or(0.0)) / 2.0;
                let admired = style::prestige(rules, rank);
                let before = h.taste;
                h.taste = style::moved(&h.taste, &m.traits, rules.alpha * admired / most);
                // How far it moved, each trait in its band's spread.
                let shift: f64 = (0..3)
                    .filter(|&k| units[k] > 0.0)
                    .map(|k| {
                        f64::from((h.taste.traits()[k] - before.traits()[k]) / units[k]).powi(2)
                    })
                    .sum::<f64>()
                    .sqrt();
                if moved_most.is_none_or(|(most_shift, _)| shift > most_shift) {
                    moved_most = Some((shift, m.id));
                }
            }
            if let Some((_, id)) = moved_most {
                h.admired = Some(id);
            }
        }
        // What was seen elsewhere has been met.
        self.seen_away.clear();
    }
}
