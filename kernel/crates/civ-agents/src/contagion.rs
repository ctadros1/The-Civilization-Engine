//! What people shed of a disease, where it lies and where it goes (M6a slice AZ, step two;
//! ADR-0021 §4; research 12-02 §5.2-§5.3, 03-02 §1.6). Loads are kept in households' heaps, in
//! wells, in river reaches (what passes them today), in tips (ADR-0038 §3), on sewage farms
//! (ADR-0044 §4) and in households' stored water, the last by where the water was drawn; a load
//! soaking through the ground is on its way to a well or a river until the day it arrives. Every
//! move is recorded. Nothing makes a load but someone shedding (12-02 §1.1), and no choice reads
//! one.

use std::collections::BTreeMap;

use civ_core::PermanentId;

/// Where a load lies or passes. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Node {
    /// Someone shedding it.
    Person(PermanentId),
    /// A household's heap.
    Midden(PermanentId),
    /// A well's water.
    Well(PermanentId),
    /// A river reach: what passes it today.
    Reach(u32),
    /// A household's stored water.
    Store(PermanentId),
    /// A tip, by its id (M11f, second part; ADR-0038 §3): what heaps carried there held.
    Tip(PermanentId),
    /// The refuse a household holds as a good, drawn from a tip to be burned (M11f, fourth part;
    /// ADR-0038 §5): what it held of a disease, which burning it destroys.
    Refuse(PermanentId),
    /// A sewer's settling tank at its outfall, by the sewer's id (ADR-0041 §4): what settled
    /// there with the sludge.
    Sewer(PermanentId),
    /// A field a sewer's flow is led onto as a farm, by the field's id (ADR-0044 §4): what the
    /// farm took, washing and soaking in from the field's place as a tip's does.
    Field(PermanentId),
    /// The ground at a map cell where someone shedding worked (ADR-0021 §4): at a pit or a
    /// deposit, a gathering block or a way under work, washing and soaking in from the cell's
    /// middle as a heap does.
    Ground(u32),
    /// A holder's stock of a food good (ADR-0045): what a shedder who made it passed to it, mixed
    /// through the stock as stored water's load is through its litres.
    Food { holder: PermanentId, good: u16 },
}

/// The bits of a food node's number that hold its holder's id; the good is above them (ADR-0045).
const FOOD_HOLDER_BITS: u32 = 48;

impl Node {
    /// Its code and number in saves.
    pub fn code(self) -> (u8, u64) {
        match self {
            Node::Person(p) => (0, p.get()),
            Node::Midden(h) => (1, h.get()),
            Node::Well(w) => (2, w.get()),
            Node::Reach(r) => (3, u64::from(r)),
            Node::Store(h) => (4, h.get()),
            Node::Tip(t) => (5, t.get()),
            Node::Refuse(h) => (6, h.get()),
            Node::Sewer(s) => (7, s.get()),
            Node::Field(f) => (8, f.get()),
            Node::Ground(c) => (9, u64::from(c)),
            Node::Food { holder, good } => {
                (10, holder.get() | (u64::from(good) << FOOD_HOLDER_BITS))
            }
        }
    }

    /// The node with code `code` and number `n`.
    pub fn from_code(code: u8, n: u64) -> Option<Node> {
        Some(match code {
            0 => Node::Person(PermanentId::from_raw(n)?),
            1 => Node::Midden(PermanentId::from_raw(n)?),
            2 => Node::Well(PermanentId::from_raw(n)?),
            3 => Node::Reach(u32::try_from(n).ok()?),
            4 => Node::Store(PermanentId::from_raw(n)?),
            5 => Node::Tip(PermanentId::from_raw(n)?),
            6 => Node::Refuse(PermanentId::from_raw(n)?),
            7 => Node::Sewer(PermanentId::from_raw(n)?),
            8 => Node::Field(PermanentId::from_raw(n)?),
            9 => Node::Ground(u32::try_from(n).ok()?),
            10 => Node::Food {
                holder: PermanentId::from_raw(n & ((1u64 << FOOD_HOLDER_BITS) - 1))?,
                good: u16::try_from(n >> FOOD_HOLDER_BITS).ok()?,
            },
            _ => return None,
        })
    }
}

/// How a load moved. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum How {
    /// Shed by someone into their household's heap.
    Shed = 0,
    /// Washed off a heap by the day's surplus into a wellhead or a river.
    Runoff = 1,
    /// Soaked from a heap into the ground, on its way to a well or a river.
    Seep = 2,
    /// Arrived through the ground.
    Arrived = 3,
    /// Drawn with water into a household's store.
    Drawn = 4,
    /// Carried with a heap to a tip (M11f, second part; ADR-0038 §3), or by a sewer to its
    /// outfall (ADR-0041 §2) and on to its farm (ADR-0044 §4).
    Carried = 5,
    /// Passed by someone shedding to food they made, or moved with that food from one holder to
    /// another (ADR-0045).
    Handled = 6,
}

impl How {
    pub fn code(self) -> u8 {
        self as u8
    }

    pub fn from_code(code: u8) -> Option<How> {
        [
            How::Shed,
            How::Runoff,
            How::Seep,
            How::Arrived,
            How::Drawn,
            How::Carried,
            How::Handled,
        ]
        .get(usize::from(code))
        .copied()
    }
}

/// A day's moves of a load along one way, recorded (12-02 §5.2: "This well received contamination
/// from Pit 184 after groundwater rose"; 05-03 §7.4: "well 17, contamination mixture during days
/// 120-123").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    pub day: i64,
    pub disease: u16,
    pub how: How,
    pub from: Node,
    pub to: Node,
    /// Doses.
    pub amount: f64,
}

/// A load soaking through the ground toward `to`, arriving on `arrives` (what decays on the way
/// is already taken off).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transit {
    pub arrives: i64,
    pub disease: u16,
    pub from: Node,
    pub to: Node,
    pub amount: f64,
}

/// How long moves are kept, days.
pub const MOVES_KEPT_DAYS: i64 = 400;

/// A load smaller than this many doses is gone, and a move of less is not recorded: drunk from a
/// well of a thousand litres, it would be a hazard of a few millionths.
pub const NEGLIGIBLE: f64 = 1e-3;

/// Every load and move. Saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contagion {
    /// Each heap's, well's and reach's load of each disease, doses.
    pub loads: BTreeMap<(Node, u16), f64>,
    /// Each household's stored water's load of each disease, by where the water came from.
    pub stores: BTreeMap<(PermanentId, u16), BTreeMap<Node, f64>>,
    /// Loads on their way through the ground, in the order sent.
    pub transit: Vec<Transit>,
    /// Moves of the last [`MOVES_KEPT_DAYS`] days, in the order made.
    pub moves: Vec<Move>,
    /// Each food node's kilograms as last reconciled with its holder's stock (ADR-0045): a stock
    /// found lower lost its load in proportion, one found higher was diluted by clean food.
    pub food_kg: BTreeMap<Node, f64>,
    /// What each person shedding passed to food today ahead of their shedding, by disease, doses:
    /// taken from the next midnight's shedding, so nothing is shed twice (ADR-0045).
    pub owed: BTreeMap<(PermanentId, u16), f64>,
    /// Today's doses eaten with food, by person and disease, and the food they came from: drawn
    /// at midnight with the day's other exposures (ADR-0045).
    pub eaten: Vec<(PermanentId, u16, f64, Node)>,
}

impl Contagion {
    /// Whether nothing lies anywhere or is on its way.
    pub fn is_empty(&self) -> bool {
        self.loads.is_empty()
            && self.stores.is_empty()
            && self.transit.is_empty()
            && self.owed.is_empty()
            && self.eaten.is_empty()
    }

    /// Whether any food carries a load (ADR-0045): while none does, nothing about food is done.
    pub fn food_loaded(&self) -> bool {
        !self.food_kg.is_empty()
    }

    /// Reconciles food node `node` with its holder's stock of `kg` now (ADR-0045): a stock lower
    /// than last seen took its load away with what left it, by its share; a higher one was
    /// diluted by clean food, its load unchanged. Nothing is ever made.
    pub fn food_reconcile(&mut self, node: Node, kg: f64) {
        let Some(&was) = self.food_kg.get(&node) else {
            return;
        };
        let kg = kg.max(0.0);
        if kg + 1e-12 < was {
            let keep = if was > 0.0 { kg / was } else { 0.0 };
            for ((n, _), l) in self.loads.iter_mut() {
                if *n == node {
                    *l *= keep;
                }
            }
            self.loads.retain(|(n, _), l| *n != node || *l > NEGLIGIBLE);
        }
        if self.loads.keys().any(|(n, _)| *n == node) {
            self.food_kg.insert(node, kg);
        } else {
            self.food_kg.remove(&node);
        }
    }

    /// The diseases food node `node` carries, with their loads.
    pub fn food_loads(&self, node: Node) -> Vec<(u16, f64)> {
        self.loads
            .iter()
            .filter(|((n, _), _)| *n == node)
            .map(|(&(_, d), &l)| (d, l))
            .collect()
    }

    /// Food node `node`'s stock is `kg` now after a hooked change that has already moved its
    /// load: kept as seen, or let go with its load.
    pub fn food_settled(&mut self, node: Node, kg: f64) {
        if self.loads.keys().any(|(n, _)| *n == node) && kg > 0.0 {
            self.food_kg.insert(node, kg);
        } else {
            self.loads.retain(|(n, _), _| *n != node);
            self.food_kg.remove(&node);
        }
    }

    /// Puts `amount` of `disease` on food node `node`, its holder's stock `kg` now.
    pub fn food_add(&mut self, node: Node, disease: u16, amount: f64, kg: f64) {
        if amount <= 0.0 || kg <= 0.0 {
            return;
        }
        self.food_reconcile(node, kg);
        self.add(node, disease, amount);
        self.food_kg.insert(node, kg);
    }

    /// The load of `disease` at `node`.
    pub fn load(&self, node: Node, disease: u16) -> f64 {
        self.loads.get(&(node, disease)).copied().unwrap_or(0.0)
    }

    /// Adds `amount` of `disease` at `node`.
    pub fn add(&mut self, node: Node, disease: u16, amount: f64) {
        if amount > 0.0 {
            *self.loads.entry((node, disease)).or_default() += amount;
        }
    }

    /// Takes `share` of the load of `disease` at `node`; returns what was taken.
    pub fn take(&mut self, node: Node, disease: u16, share: f64) -> f64 {
        let Some(l) = self.loads.get_mut(&(node, disease)) else {
            return 0.0;
        };
        let taken = *l * share.clamp(0.0, 1.0);
        *l -= taken;
        if *l <= NEGLIGIBLE {
            self.loads.remove(&(node, disease));
        }
        taken
    }

    /// Takes `amount` of `disease` at `node`, or all there is; returns what was taken.
    pub fn take_amount(&mut self, node: Node, disease: u16, amount: f64) -> f64 {
        let held = self.load(node, disease);
        if held <= 0.0 {
            return 0.0;
        }
        self.take(node, disease, amount / held)
    }

    /// The load of `disease` in `household`'s stored water, from wherever it came.
    pub fn store_load(&self, household: PermanentId, disease: u16) -> f64 {
        self.stores
            .get(&(household, disease))
            .map_or(0.0, |s| s.values().sum())
    }

    /// Adds `amount` of `disease`, drawn at `source`, to `household`'s stored water.
    pub fn store_add(&mut self, household: PermanentId, disease: u16, source: Node, amount: f64) {
        if amount > 0.0 {
            *self
                .stores
                .entry((household, disease))
                .or_default()
                .entry(source)
                .or_default() += amount;
        }
    }

    /// Takes `share` of `household`'s stored water's load of `disease`, from each source alike;
    /// returns what was taken from each, in source order.
    pub fn store_take(
        &mut self,
        household: PermanentId,
        disease: u16,
        share: f64,
    ) -> Vec<(Node, f64)> {
        let share = share.clamp(0.0, 1.0);
        let Some(s) = self.stores.get_mut(&(household, disease)) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (source, l) in s.iter_mut() {
            let taken = *l * share;
            *l -= taken;
            out.push((*source, taken));
        }
        s.retain(|_, l| *l > NEGLIGIBLE);
        if s.is_empty() {
            self.stores.remove(&(household, disease));
        }
        out
    }

    /// Lets `decay(disease)` of every load and store die, dropping what is too small to count.
    pub fn decay(&mut self, decay: impl Fn(u16) -> f64) {
        for ((_, d), l) in self.loads.iter_mut() {
            *l *= 1.0 - decay(*d).clamp(0.0, 1.0);
        }
        for ((_, d), s) in self.stores.iter_mut() {
            let keep = 1.0 - decay(*d).clamp(0.0, 1.0);
            for l in s.values_mut() {
                *l *= keep;
            }
            s.retain(|_, l| *l > NEGLIGIBLE);
        }
        self.loads.retain(|_, l| *l > NEGLIGIBLE);
        self.stores.retain(|_, s| !s.is_empty());
        // A food node whose load has all died keeps no kilograms.
        let loads = &self.loads;
        self.food_kg
            .retain(|n, _| loads.keys().any(|(m, _)| m == n));
    }

    /// Records a move, adding it to one already recorded that day along the same way.
    pub fn record(&mut self, m: Move) {
        if m.amount <= NEGLIGIBLE {
            return;
        }
        let same = self
            .moves
            .iter_mut()
            .rev()
            .take_while(|x| x.day == m.day)
            .find(|x| (x.how, x.from, x.to, x.disease) == (m.how, m.from, m.to, m.disease));
        match same {
            Some(x) => x.amount += m.amount,
            None => self.moves.push(m),
        }
    }

    /// Lets go of moves older than [`MOVES_KEPT_DAYS`] before `day`.
    pub fn prune_moves(&mut self, day: i64) {
        let from = day - MOVES_KEPT_DAYS;
        if self.moves.first().is_some_and(|m| m.day < from) {
            self.moves.retain(|m| m.day >= from);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn loads_add_take_and_decay_and_a_store_keeps_its_sources() {
        let mut c = Contagion::default();
        assert!(c.is_empty());
        c.add(Node::Well(id(5)), 0, 10.0);
        assert_eq!(c.load(Node::Well(id(5)), 0), 10.0);
        assert_eq!(c.take(Node::Well(id(5)), 0, 0.25), 2.5);
        assert_eq!(c.take_amount(Node::Well(id(5)), 0, 100.0), 7.5);
        assert!(c.is_empty());
        c.store_add(id(2), 0, Node::Well(id(5)), 3.0);
        c.store_add(id(2), 0, Node::Reach(1), 1.0);
        c.store_add(id(2), 1, Node::Reach(1), 9.0);
        c.store_add(id(3), 0, Node::Reach(1), 9.0);
        assert_eq!(c.store_load(id(2), 0), 4.0);
        let taken = c.store_take(id(2), 0, 0.5);
        assert_eq!(taken, vec![(Node::Well(id(5)), 1.5), (Node::Reach(1), 0.5)]);
        assert_eq!(c.store_load(id(2), 0), 2.0);
        assert_eq!(c.store_load(id(2), 1), 9.0);
        c.decay(|d| if d == 0 { 0.5 } else { 0.0 });
        assert_eq!(c.store_load(id(2), 0), 1.0);
        assert_eq!(c.store_load(id(3), 0), 4.5);
        assert_eq!(c.store_load(id(2), 1), 9.0);
        // A day's moves along one way are one record; another day's another.
        let m = Move {
            day: 3,
            disease: 0,
            how: How::Drawn,
            from: Node::Well(id(5)),
            to: Node::Store(id(2)),
            amount: 1.0,
        };
        c.record(m);
        c.record(Move {
            to: Node::Store(id(3)),
            ..m
        });
        c.record(m);
        c.record(Move {
            amount: NEGLIGIBLE / 2.0,
            ..m
        });
        c.record(Move { day: 4, ..m });
        assert_eq!(c.moves.len(), 3);
        assert_eq!(c.moves[0].amount, 2.0);
        for code in 0..9u8 {
            let n = Node::from_code(code, 7).expect("a node");
            assert_eq!(n.code(), (code, 7));
        }
    }
}
