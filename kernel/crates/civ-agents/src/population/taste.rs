//! Taste moving toward admired buildings (M3b slice R; research 11-02 §1.1, §2.2): once a year,
//! each household meets the buildings its settlement finished in the year past, and its taste in
//! building moves a little toward each, the more the more admired it is.

use super::*;

use crate::params::Taste;
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
    /// Once a year, as wealth is recorded: each household's taste moves toward each building of
    /// its settlement finished in the year past, other than its own, oldest first. It moves
    /// `alpha` of the way for the most admired, and less for others in proportion. A building is
    /// admired from 1 to `prestige_most` times, for its patron and its builders equally (11-02
    /// §1.1: prestige is not wealth alone): its owner's standing in goods among the settlement's
    /// households, and how well it was built among the year's new buildings there, each ranked
    /// from the least to the most. A building that has given way, or any part of it, is admired
    /// by nobody. Each new building is met once, so familiarity saturates. The household
    /// remembers the building that moved its taste most, which its next building follows.
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
        let most = rules.prestige_most.max(1.0);
        let units = rules.tradition_spread.traits();
        for (_, h) in self.households.iter_mut() {
            let Some(settlement) = h.settlement else {
                continue;
            };
            let mut moved_most: Option<(f64, PermanentId)> = None;
            for m in met
                .iter()
                .filter(|m| m.settlement == settlement && m.owner != h.id)
            {
                let rank = (standing.get(&m.owner).copied().unwrap_or(0.0)
                    + craft.get(&m.id).copied().unwrap_or(0.0))
                    / 2.0;
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
    }
}
