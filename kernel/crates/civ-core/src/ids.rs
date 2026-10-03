//! Identity.
//!
//! Two different things are called "an id" in a long-running simulation (research 01-02 §1.2):
//!
//! - A **live handle** ([`Handle`]) points at a row of a [`GenTable`]. Rows are reused after
//!   removal, so a handle carries the row's generation; a handle to a removed row never resolves,
//!   even after the row holds someone else.
//! - A **permanent id** ([`PermanentId`]) names an entity forever, in genealogies, histories and
//!   saves. It is never reused. Its allocator is world state and is saved.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A generation-checked reference to a row of a [`GenTable<T>`].
pub struct Handle<T> {
    index: u32,
    generation: u32,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    /// A handle from its parts. Only meaningful for the table that issued them.
    pub const fn from_raw_parts(index: u32, generation: u32) -> Self {
        Handle {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    /// The row index.
    pub const fn index(self) -> u32 {
        self.index
    }

    /// The row generation at the time the handle was issued.
    pub const fn generation(self) -> u32 {
        self.generation
    }

    /// Packs the handle into 64 bits (generation high, index low), for wire formats.
    pub const fn to_bits(self) -> u64 {
        ((self.generation as u64) << 32) | self.index as u64
    }

    /// Unpacks [`Handle::to_bits`].
    pub const fn from_bits(bits: u64) -> Self {
        Handle::from_raw_parts(bits as u32, (bits >> 32) as u32)
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state);
    }
}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.index, self.generation).cmp(&(other.index, other.generation))
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}v{}", self.index, self.generation)
    }
}

#[derive(Clone)]
struct Slot<T> {
    generation: u32,
    value: Option<T>,
}

/// A table of rows addressed by generation-checked handles.
///
/// Removing a row bumps its generation and frees it for reuse, so old handles stop resolving. A
/// row whose generation would overflow is retired instead of reused, so a handle can never come
/// back to life by wrap-around.
#[derive(Clone)]
pub struct GenTable<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
    len: usize,
    retired: usize,
}

impl<T> Default for GenTable<T> {
    fn default() -> Self {
        GenTable::new()
    }
}

impl<T: fmt::Debug> fmt::Debug for GenTable<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> GenTable<T> {
    /// An empty table.
    pub fn new() -> Self {
        GenTable {
            slots: Vec::new(),
            free: Vec::new(),
            len: 0,
            retired: 0,
        }
    }

    /// Number of live rows.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the table has no live rows.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Number of rows retired because their generation was exhausted.
    pub fn retired(&self) -> usize {
        self.retired
    }

    /// Inserts a row and returns its handle.
    ///
    /// # Panics
    /// If the table would exceed `u32::MAX` rows.
    pub fn insert(&mut self, value: T) -> Handle<T> {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            debug_assert!(slot.value.is_none(), "free list held a live row");
            slot.value = Some(value);
            self.len += 1;
            return Handle::from_raw_parts(index, slot.generation);
        }
        let index = u32::try_from(self.slots.len()).expect("table exceeds u32::MAX rows");
        self.slots.push(Slot {
            generation: 1,
            value: Some(value),
        });
        self.len += 1;
        Handle::from_raw_parts(index, 1)
    }

    /// Removes the row `handle` points at. Returns `None` for a stale or foreign handle.
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let slot = self.slots.get_mut(handle.index as usize)?;
        if slot.generation != handle.generation {
            return None;
        }
        let value = slot.value.take()?;
        self.len -= 1;
        match slot.generation.checked_add(1) {
            Some(next) => {
                slot.generation = next;
                self.free.push(handle.index);
            }
            None => self.retired += 1,
        }
        Some(value)
    }

    /// The row `handle` points at, if it is still the same row.
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let slot = self.slots.get(handle.index as usize)?;
        if slot.generation == handle.generation {
            slot.value.as_ref()
        } else {
            None
        }
    }

    /// Mutable access to the row `handle` points at, if it is still the same row.
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.index as usize)?;
        if slot.generation == handle.generation {
            slot.value.as_mut()
        } else {
            None
        }
    }

    /// Whether `handle` still resolves.
    pub fn contains(&self, handle: Handle<T>) -> bool {
        self.get(handle).is_some()
    }

    /// Live rows in row order.
    pub fn iter(&self) -> impl Iterator<Item = (Handle<T>, &T)> {
        self.slots.iter().enumerate().filter_map(|(i, s)| {
            s.value
                .as_ref()
                .map(|v| (Handle::from_raw_parts(i as u32, s.generation), v))
        })
    }

    /// Live rows in row order, mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Handle<T>, &mut T)> {
        self.slots.iter_mut().enumerate().filter_map(|(i, s)| {
            let generation = s.generation;
            s.value
                .as_mut()
                .map(|v| (Handle::from_raw_parts(i as u32, generation), v))
        })
    }

    /// Handles of the live rows in row order.
    pub fn handles(&self) -> impl Iterator<Item = Handle<T>> + '_ {
        self.iter().map(|(h, _)| h)
    }
}

/// A world-scoped identity that is never reused. Zero is never a valid id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PermanentId(u64);

impl PermanentId {
    /// An id from its saved value; `None` for zero.
    pub const fn from_raw(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(PermanentId(value))
        }
    }

    /// The saved value.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for PermanentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Issues [`PermanentId`]s in increasing order. Its counter is world state: save it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdAllocator {
    next: u64,
}

impl Default for IdAllocator {
    fn default() -> Self {
        IdAllocator::new()
    }
}

impl IdAllocator {
    /// An allocator whose first id is 1.
    pub const fn new() -> Self {
        IdAllocator { next: 1 }
    }

    /// An allocator restored from a saved counter. Zero is treated as a fresh allocator.
    pub const fn from_next(next: u64) -> Self {
        IdAllocator {
            next: if next == 0 { 1 } else { next },
        }
    }

    /// The value the next id will have; this is what gets saved.
    pub const fn peek_next(&self) -> u64 {
        self.next
    }

    /// Issues the next id.
    ///
    /// # Panics
    /// After 2^64 − 1 ids, which no world will reach.
    pub fn allocate(&mut self) -> PermanentId {
        let id = PermanentId(self.next);
        self.next = self.next.checked_add(1).expect("permanent ids exhausted");
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashMap;

    #[test]
    fn stale_handles_never_resolve_after_reuse() {
        let mut table = GenTable::new();
        let a = table.insert("ana");
        assert_eq!(table.remove(a), Some("ana"));
        let b = table.insert("bo");
        assert_eq!(a.index(), b.index(), "the row is reused");
        assert_ne!(a, b);
        assert_eq!(table.get(a), None);
        assert_eq!(table.remove(a), None);
        assert_eq!(table.get(b), Some(&"bo"));
    }

    #[test]
    fn exhausted_generations_retire_the_row() {
        let mut table = GenTable::new();
        let h = table.insert(1);
        // Fast-forward the row to its last generation.
        table.slots[h.index() as usize].generation = u32::MAX;
        let last = Handle::from_raw_parts(h.index(), u32::MAX);
        assert_eq!(table.remove(last), Some(1));
        assert_eq!(table.retired(), 1);
        let next = table.insert(2);
        assert_ne!(next.index(), h.index(), "a retired row is never reused");
    }

    #[test]
    fn handle_bits_round_trip() {
        let h: Handle<()> = Handle::from_raw_parts(0xdead_beef, 7);
        assert_eq!(Handle::<()>::from_bits(h.to_bits()), h);
    }

    #[test]
    fn permanent_ids_increase_and_skip_zero() {
        let mut ids = IdAllocator::new();
        assert_eq!(ids.allocate().get(), 1);
        assert_eq!(ids.allocate().get(), 2);
        assert_eq!(ids.peek_next(), 3);
        assert_eq!(IdAllocator::from_next(0).peek_next(), 1);
        assert_eq!(PermanentId::from_raw(0), None);
    }

    #[derive(Clone, Debug)]
    enum Op {
        Insert(u32),
        Remove(usize),
        RemoveStale(usize),
        Get(usize),
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            3 => any::<u32>().prop_map(Op::Insert),
            2 => any::<usize>().prop_map(Op::Remove),
            1 => any::<usize>().prop_map(Op::RemoveStale),
            2 => any::<usize>().prop_map(Op::Get),
        ]
    }

    proptest! {
        /// Model-based check against a HashMap: live handles resolve to exactly their value,
        /// removed handles never resolve again, and the length always agrees.
        #[test]
        fn gen_table_matches_a_model(ops in proptest::collection::vec(op(), 1..200)) {
            let mut table = GenTable::new();
            let mut model: HashMap<Handle<u32>, u32> = HashMap::new();
            let mut live: Vec<Handle<u32>> = Vec::new();
            let mut dead: Vec<Handle<u32>> = Vec::new();
            for op in ops {
                match op {
                    Op::Insert(v) => {
                        let h = table.insert(v);
                        prop_assert!(!model.contains_key(&h));
                        model.insert(h, v);
                        live.push(h);
                    }
                    Op::Remove(i) if !live.is_empty() => {
                        let h = live.swap_remove(i % live.len());
                        prop_assert_eq!(table.remove(h), model.remove(&h));
                        dead.push(h);
                    }
                    Op::RemoveStale(i) if !dead.is_empty() => {
                        let h = dead[i % dead.len()];
                        prop_assert_eq!(table.remove(h), None);
                    }
                    Op::Get(i) if !dead.is_empty() => {
                        prop_assert_eq!(table.get(dead[i % dead.len()]), None);
                    }
                    _ => {}
                }
                prop_assert_eq!(table.len(), model.len());
                for (h, v) in &model {
                    prop_assert_eq!(table.get(*h), Some(v));
                }
            }
            prop_assert_eq!(table.iter().count(), model.len());
        }
    }
}
