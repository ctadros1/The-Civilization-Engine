//! Wealth measures (ADR-0007 §4): what each household has, as several separate quantities, and how
//! each spreads over a settlement's people (research 08-14 §1.1: no one universal "inequality"
//! statistic). Derived, recorded once a year, and never an input to behaviour.
//!
//! - **Land held and land worked**, hectares. A field a settlement holds counts as worked by the
//!   household it gives it to and held by none of them (08-14 §1.1: keep institutional ownership
//!   separate from household access).
//! - **Goods**, in hours of work: what is in a household's store and in the stores of the
//!   workshops it owns, each good valued at its settlement's price, the median of what it costs
//!   the settlement's households in their own work, so households can be compared.
//! - **House floor area**, square metres under its homes' roofs: the archaeological proxy.
//! - **Roofed floor and room for goods** beside it (ADR-0009 §7): the floor under every roof it
//!   has, homes, stores and workshops, and the kilograms of goods those roofs have room for.
//!
//! Inequality is person-weighted, per head: a household is a consumption-sharing unit, and each of
//! its members counts at the household's measure over its members (08-14 §1.1, §5.6). Floor area
//! is compared house by house, as archaeologists measure it (a proxy of its own, 08-14 §1.1).

use std::collections::{BTreeMap, BTreeSet};

use civ_core::{PermanentId, SimTime};
use civ_land::{Land, LandParams, Party};

use crate::params::{Catalog, PeopleParams};
use crate::population::{Population, stores_now};

/// One household's measures.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseholdWealth {
    /// The household.
    pub household: PermanentId,
    /// Its settlement.
    pub settlement: PermanentId,
    /// Its members.
    pub members: u32,
    /// Land it holds, whoever works it, hectares.
    pub held_ha: f64,
    /// Land it works, whoever holds it, hectares.
    pub worked_ha: f64,
    /// Land it holds and has let to other households, hectares.
    pub let_ha: f64,
    /// Land it works on lease from other households, hectares.
    pub rented_ha: f64,
    /// Its goods, hours of work at its settlement's prices.
    pub goods_h: f64,
    /// Floor area of its homes, under their roofs, square metres.
    pub floor_m2: f64,
    /// Floor under all its roofs, homes, stores and workshops, square metres.
    pub roofed_m2: f64,
    /// Room for goods under all its roofs, kilograms ([`crate::population::room_of`]).
    pub storage_kg: f64,
}

/// How a settlement's measures spread over its people at one moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spread {
    /// The settlement.
    pub settlement: PermanentId,
    /// Households measured.
    pub households: u32,
    /// Their people.
    pub people: u32,
    /// Gini of goods per head, person-weighted.
    pub gini_goods: f64,
    /// Gini of land held per head, person-weighted (0 when no household holds any).
    pub gini_held: f64,
    /// Gini of land worked per head, person-weighted.
    pub gini_worked: f64,
    /// Gini of floor area, house by house: the households with a roof of their own.
    pub gini_floor: f64,
    /// The share of all goods that the richest tenth of people (by goods per head) have.
    pub top_tenth_goods: f64,
    /// The share of households that hold no land.
    pub holding_none: f64,
    /// The share of households that work none.
    pub working_none: f64,
    /// Goods per head, hours of work.
    pub goods_h_per_head: f64,
    /// Land worked per head, hectares.
    pub worked_ha_per_head: f64,
    /// Floor area per house, square metres.
    pub floor_m2_per_house: f64,
    /// Land the settlement itself holds, hectares.
    pub common_ha: f64,
    /// Floor under all their roofs, homes, stores and workshops, per household with a roof,
    /// square metres.
    pub roofed_m2_per_house: f64,
    /// Room for goods under all their roofs, per household with a roof, kilograms.
    pub storage_kg_per_house: f64,
}

/// A settlement's measures at the end of a year.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WealthYear {
    /// The year that ended.
    pub year: i64,
    /// The measures.
    pub spread: Spread,
}

/// Observations to weigh, `(value, weight)`, without empty weights, smallest value first.
fn sorted(obs: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut v: Vec<(f64, f64)> = obs
        .iter()
        .filter(|o| o.1 > 0.0)
        .map(|&(x, w)| (x.max(0.0), w))
        .collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v
}

/// The Gini coefficient of `obs`, each `(value, weight)`: for a person-weighted Gini, a
/// household's measure per head and its people, so that each person counts once at it (research
/// 08-14 §5.6, its weighted form). 0 when there is nothing to share or nobody to share it.
pub fn gini(obs: &[(f64, f64)]) -> f64 {
    let v = sorted(obs);
    let weight: f64 = v.iter().map(|o| o.1).sum();
    let total: f64 = v.iter().map(|o| o.0 * o.1).sum();
    if weight <= 0.0 || total <= 0.0 {
        return 0.0;
    }
    // One less twice the area under the Lorenz curve, a trapezoid per observation: exact for
    // people at equal shares within a household.
    let (mut cum, mut area) = (0.0, 0.0);
    for (x, w) in v {
        let before = cum;
        cum += x * w;
        area += w / weight * (before + cum) / total;
    }
    (1.0 - area).clamp(0.0, 1.0)
}

/// The share of the total of `obs`, each `(value per head, people)`, that the richest `fraction`
/// of people have; a household on the line counts in part. 0 when there is nothing.
pub fn top_share(obs: &[(f64, f64)], fraction: f64) -> f64 {
    let mut v = sorted(obs);
    v.reverse();
    let people: f64 = v.iter().map(|o| o.1).sum();
    let total: f64 = v.iter().map(|o| o.0 * o.1).sum();
    if people <= 0.0 || total <= 0.0 {
        return 0.0;
    }
    let mut left = people * fraction.clamp(0.0, 1.0);
    let mut top = 0.0;
    for (x, w) in v {
        if left <= 0.0 {
            break;
        }
        let take = w.min(left);
        top += x * take;
        left -= take;
    }
    (top / total).clamp(0.0, 1.0)
}

/// Each of `goods` goods' price at a settlement: the median of what it costs the settlement's
/// households in their own work (`costs[household][good]`, hours per unit, where known), and 0
/// where none of them knows.
pub fn median_prices(costs: &[Vec<Option<f64>>], goods: usize) -> Vec<f64> {
    (0..goods)
        .map(|g| {
            let mut known: Vec<f64> = costs
                .iter()
                .filter_map(|c| c.get(g).copied().flatten())
                .filter(|x| x.is_finite() && *x >= 0.0)
                .collect();
            known.sort_by(f64::total_cmp);
            let n = known.len();
            match n {
                0 => 0.0,
                _ if n % 2 == 1 => known[n / 2],
                _ => (known[n / 2 - 1] + known[n / 2]) / 2.0,
            }
        })
        .collect()
}

/// How the measures of `households`, the households of `settlement`, spread over its people;
/// `common_ha` is the land the settlement itself holds.
pub fn spread(settlement: PermanentId, common_ha: f64, households: &[HouseholdWealth]) -> Spread {
    let people: u32 = households.iter().map(|h| h.members).sum();
    let per_head = |f: fn(&HouseholdWealth) -> f64| -> Vec<(f64, f64)> {
        households
            .iter()
            .filter(|h| h.members > 0)
            .map(|h| (f(h) / f64::from(h.members), f64::from(h.members)))
            .collect()
    };
    let goods = per_head(|h| h.goods_h);
    let houses: Vec<(f64, f64)> = households
        .iter()
        .filter(|h| h.floor_m2 > 0.0)
        .map(|h| (h.floor_m2, 1.0))
        .collect();
    let roofed = households.iter().filter(|h| h.roofed_m2 > 0.0).count() as f64;
    let total = |f: fn(&HouseholdWealth) -> f64| households.iter().map(f).fold(0.0, |a, b| a + b);
    let share = |n: usize| {
        if households.is_empty() {
            0.0
        } else {
            n as f64 / households.len() as f64
        }
    };
    let per = |x: f64, n: f64| if n > 0.0 { x / n } else { 0.0 };
    Spread {
        settlement,
        households: households.len() as u32,
        people,
        gini_goods: gini(&goods),
        gini_held: gini(&per_head(|h| h.held_ha)),
        gini_worked: gini(&per_head(|h| h.worked_ha)),
        gini_floor: gini(&houses),
        top_tenth_goods: top_share(&goods, 0.1),
        holding_none: share(households.iter().filter(|h| h.held_ha <= 0.0).count()),
        working_none: share(households.iter().filter(|h| h.worked_ha <= 0.0).count()),
        goods_h_per_head: per(total(|h| h.goods_h), f64::from(people)),
        worked_ha_per_head: per(total(|h| h.worked_ha), f64::from(people)),
        floor_m2_per_house: per(total(|h| h.floor_m2), houses.len() as f64),
        common_ha: common_ha.max(0.0),
        roofed_m2_per_house: per(total(|h| h.roofed_m2), roofed),
        storage_kg_per_house: per(total(|h| h.storage_kg), roofed),
    }
}

impl Population {
    /// The measures of every household with members and a settlement at `now`, in id order
    /// (ADR-0007 §4).
    pub fn wealth(
        &self,
        catalog: &Catalog,
        params: &PeopleParams,
        land_params: &LandParams,
        land: &Land,
        now: SimTime,
    ) -> Vec<HouseholdWealth> {
        let goods = &catalog.goods;
        let mut living: Vec<(&crate::Household, PermanentId)> = self
            .households
            .iter()
            .map(|(_, h)| h)
            .filter(|h| !h.members.is_empty())
            .filter_map(|h| h.settlement.map(|s| (h, s)))
            .collect();
        living.sort_by_key(|(h, _)| h.id);
        // Each settlement's prices.
        let mut costs: BTreeMap<PermanentId, Vec<Vec<Option<f64>>>> = BTreeMap::new();
        for &(h, s) in &living {
            costs.entry(s).or_default().push(self.own_costs_with(
                catalog,
                params,
                land_params,
                h,
                now,
            ));
        }
        let prices: BTreeMap<PermanentId, Vec<f64>> = costs
            .into_iter()
            .map(|(s, c)| (s, median_prices(&c, goods.len())))
            .collect();
        living
            .into_iter()
            .map(|(h, settlement)| {
                let mut w = HouseholdWealth {
                    household: h.id,
                    settlement,
                    members: h.members.len() as u32,
                    held_ha: 0.0,
                    worked_ha: 0.0,
                    let_ha: 0.0,
                    rented_ha: 0.0,
                    goods_h: 0.0,
                    floor_m2: 0.0,
                    roofed_m2: 0.0,
                    storage_kg: 0.0,
                };
                for f in &land.fields {
                    let ha = f.area_ha();
                    let held = f.holder == Party::Household(h.id);
                    let worked = f.household == h.id;
                    if held {
                        w.held_ha += ha;
                    }
                    if worked {
                        w.worked_ha += ha;
                    }
                    if f.lease.is_some() && held && !worked {
                        w.let_ha += ha;
                    }
                    if f.lease.is_some() && worked && !held {
                        w.rented_ha += ha;
                    }
                }
                let price = prices.get(&settlement).map_or(&[][..], Vec::as_slice);
                let value = |stores: &[f64]| {
                    stores
                        .iter()
                        .zip(price)
                        .fold(0.0, |a, (kg, p)| a + kg.max(0.0) * p)
                };
                w.goods_h = value(&stores_now(h, now, params, goods));
                for firm in self.firms.iter().filter(|f| f.is_open() && f.owner == h.id) {
                    w.goods_h += value(&firm.stores_at_time(now, goods, h.sheltered));
                }
                for b in land
                    .buildings
                    .iter()
                    .filter(|b| b.household == h.id && b.roofed())
                {
                    if let Some(def) = catalog
                        .building_index(&b.spec.program)
                        .and_then(|i| catalog.buildings.get(i))
                        && let Ok(e) = civ_grammar::expand(&b.spec, &def.rules)
                    {
                        if def.use_ == civ_land::PlotUse::Dwelling {
                            w.floor_m2 += e.floor_area_m2;
                        }
                        w.roofed_m2 += e.floor_area_m2;
                        w.storage_kg += crate::population::room_of(b, catalog).iter().sum::<f64>();
                    }
                }
                w
            })
            .collect()
    }

    /// Each settlement's measures at `now`, settlements in id order: how they spread, and its
    /// households' own.
    pub fn wealth_by_settlement(
        &self,
        catalog: &Catalog,
        params: &PeopleParams,
        land_params: &LandParams,
        land: &Land,
        now: SimTime,
    ) -> Vec<(Spread, Vec<HouseholdWealth>)> {
        let all = self.wealth(catalog, params, land_params, land, now);
        let settlements: BTreeSet<PermanentId> = all.iter().map(|w| w.settlement).collect();
        settlements
            .into_iter()
            .map(|s| {
                let households: Vec<HouseholdWealth> =
                    all.iter().filter(|w| w.settlement == s).cloned().collect();
                let common = land
                    .fields
                    .iter()
                    .filter(|f| f.holder == Party::Settlement(s))
                    .fold(0.0, |a, f| a + f.area_ha());
                (spread(s, common, &households), households)
            })
            .collect()
    }

    /// Records each settlement's measures for `year`, the year that ended at `now`.
    pub fn record_wealth(
        &mut self,
        catalog: &Catalog,
        params: &PeopleParams,
        land_params: &LandParams,
        land: &Land,
        now: SimTime,
        year: i64,
    ) {
        let spreads = self.wealth_by_settlement(catalog, params, land_params, land, now);
        self.wealth_years.extend(
            spreads
                .into_iter()
                .map(|(spread, _)| WealthYear { year, spread }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("nonzero")
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn gini_of_equals_is_zero_and_of_one_holding_all_is_its_share_of_the_rest() {
        assert!(close(gini(&[(3.0, 1.0); 5]), 0.0));
        assert!(close(gini(&[]), 0.0), "nobody");
        assert!(close(gini(&[(0.0, 2.0), (0.0, 3.0)]), 0.0), "nothing");
        // One of four has everything: (n - 1) / n.
        let g = gini(&[(0.0, 1.0), (0.0, 1.0), (8.0, 1.0), (0.0, 1.0)]);
        assert!(close(g, 0.75), "{g}");
    }

    #[test]
    fn a_household_weighs_as_its_people_and_scale_changes_nothing() {
        // A household of three at 2 a head is three people at 2.
        let weighted = gini(&[(2.0, 3.0), (5.0, 1.0), (1.0, 2.0)]);
        let people = gini(&[
            (2.0, 1.0),
            (2.0, 1.0),
            (2.0, 1.0),
            (5.0, 1.0),
            (1.0, 1.0),
            (1.0, 1.0),
        ]);
        assert!(close(weighted, people), "{weighted} vs {people}");
        let scaled = gini(&[(20.0, 3.0), (50.0, 1.0), (10.0, 2.0)]);
        assert!(close(weighted, scaled));
        // The textbook formula: G = 2 Σ i x(i) / (N Σ x) − (N + 1) / N over [1 1 2 2 2 5].
        let n = 6.0;
        let sum = 13.0;
        let ranked = 1.0 + 2.0 + 2.0 * 3.0 + 2.0 * 4.0 + 2.0 * 5.0 + 5.0 * 6.0;
        assert!(close(weighted, 2.0 * ranked / (n * sum) - (n + 1.0) / n));
    }

    #[test]
    fn the_top_tenth_counts_a_household_on_the_line_in_part() {
        // One of ten people at 10, nine at 1.
        let mut obs = vec![(1.0, 1.0); 9];
        obs.push((10.0, 1.0));
        assert!(close(top_share(&obs, 0.1), 10.0 / 19.0));
        // Two households of ten, at 3 and at 1 a head: the top two people are both at 3.
        let t = top_share(&[(1.0, 10.0), (3.0, 10.0)], 0.1);
        assert!(close(t, 6.0 / 40.0), "{t}");
        assert!(close(top_share(&[(0.0, 4.0)], 0.1), 0.0));
    }

    #[test]
    fn prices_are_the_median_of_known_costs() {
        let costs = vec![
            vec![Some(1.0), None, Some(5.0)],
            vec![Some(3.0), None, None],
            vec![Some(2.0), None, Some(5.0)],
        ];
        assert_eq!(median_prices(&costs, 3), vec![2.0, 0.0, 5.0]);
        let even = vec![vec![Some(1.0)], vec![Some(4.0)]];
        assert_eq!(median_prices(&even, 1), vec![2.5]);
    }

    #[test]
    fn a_spread_counts_households_people_and_the_landless() {
        let hh = |n: u64, members: u32, held: f64, worked: f64, goods: f64, floor: f64| {
            HouseholdWealth {
                household: id(n),
                settlement: id(1),
                members,
                held_ha: held,
                worked_ha: worked,
                let_ha: 0.0,
                rented_ha: 0.0,
                goods_h: goods,
                floor_m2: floor,
                roofed_m2: floor,
                storage_kg: floor * 100.0,
            }
        };
        let mut all = [
            hh(2, 4, 1.0, 1.0, 400.0, 30.0),
            hh(3, 2, 0.0, 0.5, 100.0, 0.0),
            hh(4, 2, 0.0, 0.0, 0.0, 10.0),
        ];
        // The first also has a granary of 15 m², raised, room for 7.5 t.
        all[0].roofed_m2 += 15.0;
        all[0].storage_kg += 7_500.0;
        let s = spread(id(1), 0.5, &all);
        assert_eq!((s.households, s.people), (3, 8));
        assert!(close(s.holding_none, 2.0 / 3.0));
        assert!(close(s.working_none, 1.0 / 3.0));
        assert!(close(s.goods_h_per_head, 500.0 / 8.0));
        assert!(close(s.worked_ha_per_head, 1.5 / 8.0));
        assert!(close(s.floor_m2_per_house, 20.0), "two houses");
        assert!(close(s.common_ha, 0.5));
        // Beside the houses: all the floor under their roofs, and room for goods.
        assert!(
            close(s.roofed_m2_per_house, 27.5),
            "{}",
            s.roofed_m2_per_house
        );
        assert!(close(
            s.storage_kg_per_house,
            (3_000.0 + 7_500.0 + 1_000.0) / 2.0
        ));
        // Land held: four people at 0.25 a head, four at none.
        assert!(close(s.gini_held, 0.5), "{}", s.gini_held);
        // Floor area house by house, [10, 30]: 2·(10 + 60)/(2·40) − 3/2.
        assert!(close(s.gini_floor, 0.25), "{}", s.gini_floor);
        assert!(s.gini_goods > s.gini_worked);
    }
}
