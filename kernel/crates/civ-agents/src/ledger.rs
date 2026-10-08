//! The ledger (ADR-0006 §3): every movement of goods between holders (households, firms and
//! polities, ADR-0013 §4) is one operation that moves exact quantities and carries its channel. A
//! transfer that cannot be covered does not happen, so stores are never driven below zero, and
//! over all holders goods are conserved: a transfer counts as given by one holder and received by
//! the other (see `Population::transfer`).

use civ_core::{PermanentId, SimTime};

/// Why goods moved between holders (ADR-0006 §3; research 08-06 §1.1, 08-11 §1.1). Codes are
/// part of saves and the boundary: never renumber.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Channel {
    /// Need-based help, with no debt kept (research 08-11 §1.1).
    Gift = 1,
    /// A kill shared out among a settlement's households.
    Share = 2,
    /// A new household's share of what its people's households held.
    Allocation = 3,
    /// What a household that is no more leaves to the household its people join.
    Inherit = 4,
    /// Goods for goods.
    Barter = 5,
    /// Goods for the settlement's commodity money.
    Sale = 6,
    /// A firm's owners put goods in or draw them out (slice J).
    Owner = 7,
    /// Wages a firm pays for work (slice J).
    Wage = 8,
    /// A tenant's share of a let field's grain to its holder (slice K, ADR-0007 §3).
    Rent = 9,
    /// A household's share of its threshed grain into its polity's common store (ADR-0013 §4).
    Levy = 10,
    /// Food from a polity's common store to a household that asked (ADR-0013 §4).
    Relief = 11,
    /// Food taken from a household's store without its leave (M4b slice AA, ADR-0015 §2).
    Take = 12,
    /// Food given back to a household that was taken from, as it demanded or a gathering found
    /// (ADR-0015 §5).
    Restitution = 13,
    /// Food to a household taken from beyond what was taken, as a gathering's finding imposed
    /// (M4b slice AB; research 09-07 §1.2).
    Compensation = 14,
    /// Food to the polity's common store, as a gathering's finding imposed (M4b slice AB).
    Fine = 15,
}

impl Channel {
    /// Every channel, in code order.
    pub const ALL: [Channel; 15] = [
        Channel::Gift,
        Channel::Share,
        Channel::Allocation,
        Channel::Inherit,
        Channel::Barter,
        Channel::Sale,
        Channel::Owner,
        Channel::Wage,
        Channel::Rent,
        Channel::Levy,
        Channel::Relief,
        Channel::Take,
        Channel::Restitution,
        Channel::Compensation,
        Channel::Fine,
    ];

    /// The channel with this code.
    pub fn from_code(code: u8) -> Option<Channel> {
        Channel::ALL.into_iter().find(|c| *c as u8 == code)
    }

    /// Plain-English label.
    pub fn label(self) -> &'static str {
        match self {
            Channel::Gift => "gift",
            Channel::Share => "share",
            Channel::Allocation => "household share",
            Channel::Inherit => "inheritance",
            Channel::Barter => "barter",
            Channel::Sale => "sale",
            Channel::Owner => "owner",
            Channel::Wage => "wage",
            Channel::Rent => "rent",
            Channel::Levy => "levy",
            Channel::Relief => "relief",
            Channel::Take => "taking",
            Channel::Restitution => "restitution",
            Channel::Compensation => "compensation",
            Channel::Fine => "fine",
        }
    }
}

/// One leg of a transfer: `amount` of good `good` (in its unit) from household `from` to
/// household `to`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Leg {
    /// The household that gives.
    pub from: PermanentId,
    /// The household that receives.
    pub to: PermanentId,
    /// The good, by index in the catalog's goods.
    pub good: usize,
    /// How much, in the good's unit.
    pub amount: f64,
}

/// A trade, as a settlement's market remembers it: `units` of `good` went from `seller` to
/// `buyer` for `paid` of `payment`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trade {
    /// When.
    pub at: SimTime,
    /// The household that sold.
    pub seller: PermanentId,
    /// The household that bought.
    pub buyer: PermanentId,
    /// What was sold, by index in the catalog's goods.
    pub good: u16,
    /// How much of it, in its unit.
    pub units: f32,
    /// What it was paid in.
    pub payment: u16,
    /// How much of that.
    pub paid: f32,
    /// Barter, or a sale for the settlement's money.
    pub channel: Channel,
}

/// Amounts of each good moved between households, by channel, since the counters began:
/// counters for reports, not world state, never saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Transfers {
    by: Vec<Vec<f64>>,
}

impl Transfers {
    /// Counts `amount` of good `good` moved by `channel`.
    pub fn add(&mut self, channel: Channel, good: usize, amount: f64) {
        if amount == 0.0 || !amount.is_finite() {
            return;
        }
        let c = channel as usize;
        if self.by.len() <= c {
            self.by.resize(c + 1, Vec::new());
        }
        let row = &mut self.by[c];
        if row.len() <= good {
            row.resize(good + 1, 0.0);
        }
        row[good] += amount;
    }

    /// The amount of good `good` moved by `channel`.
    pub fn get(&self, channel: Channel, good: usize) -> f64 {
        self.by
            .get(channel as usize)
            .and_then(|row| row.get(good))
            .copied()
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channels_keep_their_codes() {
        for c in Channel::ALL {
            assert_eq!(Channel::from_code(c as u8), Some(c));
        }
        assert_eq!(Channel::from_code(0), None);
        assert_eq!(Channel::Sale as u8, 6);
    }

    #[test]
    fn moved_goods_are_counted_by_channel() {
        let mut m = Transfers::default();
        m.add(Channel::Gift, 2, 1.5);
        m.add(Channel::Gift, 2, 0.5);
        m.add(Channel::Barter, 0, 3.0);
        assert_eq!(m.get(Channel::Gift, 2), 2.0);
        assert_eq!(m.get(Channel::Barter, 0), 3.0);
        assert_eq!(m.get(Channel::Sale, 0), 0.0);
        assert_eq!(m.get(Channel::Gift, 9), 0.0);
    }
}
