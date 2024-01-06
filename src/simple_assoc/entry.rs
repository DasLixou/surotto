use std::{hint::unreachable_unchecked, iter};

use crate::simple::SimpleKey;

use super::SimpleAssocSurotto;

/// A view into a single entry in a surotto, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`SimpleAssocSurotto`].
///
/// [`entry`]: SimpleAssocSurotto::entry
pub enum Entry<'a, K: SimpleKey, V> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V>),

    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V>),
}

pub struct OccupiedEntry<'a, K: SimpleKey, V> {
    pub(super) surotto: &'a mut SimpleAssocSurotto<K, V>,
    pub(super) key: K,
}

pub struct VacantEntry<'a, K: SimpleKey, V> {
    pub(super) surotto: &'a mut SimpleAssocSurotto<K, V>,
    pub(super) key: K,
}

impl<'a, K: SimpleKey, V> Entry<'a, K, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    pub fn or_insert(self, val: V) -> &'a mut V {
        self.or_insert_with(|| val)
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F>(self, f: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(f()),
        }
    }

    /// Returns this entry's key.
    pub fn key(&self) -> K {
        match self {
            Entry::Occupied(o) => o.key,
            Entry::Vacant(v) => v.key,
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the surotto.
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a, K: SimpleKey, V: Default> Entry<'a, K, V> {
    /// Ensures a value is in the entry by inserting the default value if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_default(self) -> &'a mut V {
        self.or_insert_with(Default::default)
    }
}

impl<'a, K: SimpleKey, V> OccupiedEntry<'a, K, V> {
    /// Returns this entry's key.
    pub fn key(&self) -> K {
        self.key
    }

    /// Take the ownership of the key and value from the surotto.
    pub fn remove_entry(self) -> (K, V) {
        (self.key, self.remove())
    }

    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        unsafe { self.surotto.get_unchecked(self.key) }
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the
    /// destruction of the `Entry` value, see [`into_mut`].
    ///
    /// [`into_mut`]: Self::into_mut
    pub fn get_mut(&mut self) -> &mut V {
        unsafe { self.surotto.get_unchecked_mut(self.key) }
    }

    /// Converts the `OccupiedEntry` into a mutable reference to the value in the entry
    /// with a lifetime bound to the surotto itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: Self::get_mut
    pub fn into_mut(self) -> &'a mut V {
        unsafe { self.surotto.get_unchecked_mut(self.key) }
    }

    /// Sets the value of the entry, and returns the entry's old value.
    pub fn insert(&mut self, value: V) -> V {
        let slot = unsafe { self.surotto.inner.get_unchecked_mut(self.key.idx()) };
        match slot.replace(value) {
            Some(val) => val,
            None => unsafe { unreachable_unchecked() },
        }
    }

    /// Takes the value out of the entry, and returns it.
    pub fn remove(self) -> V {
        let slot = unsafe { self.surotto.inner.get_unchecked_mut(self.key.idx()) };
        match std::mem::replace(slot, None) {
            Some(val) => val,
            None => unsafe { unreachable_unchecked() },
        }
    }
}

impl<'a, K: SimpleKey, V> VacantEntry<'a, K, V> {
    /// Returns this entry's key.
    pub fn key(&self) -> K {
        self.key
    }

    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V {
        let missing_slots = self.key.idx() - self.surotto.inner.len();
        self.surotto
            .inner
            .extend(iter::repeat_with(|| None).take(missing_slots));

        unsafe {
            self.surotto
                .inner
                .get_unchecked_mut(self.key.idx())
                .insert(value)
        }
    }
}
