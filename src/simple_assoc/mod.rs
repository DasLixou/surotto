use std::{collections::TryReserveError, hint::unreachable_unchecked, iter, marker::PhantomData};

use crate::simple::SimpleKey;

/// A datastructure where values can be associated with a key from a [`SimpleSurotto`].
///
/// [`SimpleSurotto`]: crate::simple::SimpleSurotto
pub struct SimpleAssocSurotto<K: SimpleKey, V> {
    inner: Vec<Option<V>>,
    phantom: PhantomData<K>,
}

impl<K: SimpleKey, V> Default for SimpleAssocSurotto<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: SimpleKey, V> SimpleAssocSurotto<K, V> {
    /// Constructs a new, empty `SimpleAssocSurotto<K, V>`.
    ///
    /// The surotto will not allocate until elements are inserted.
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Constructs a new, empty `SimpleAssocSurotto<K, V>` with at least the specified capacity.
    ///
    /// The surotto will be able to hold at least `capacity` elements without
    /// reallocating. This method is allowed to allocate for more elements than
    /// `capacity`. If `capacity` is 0, it will not allocate.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            phantom: PhantomData,
        }
    }

    /// Inserts a key-value pair into the surotto.
    ///
    /// If the surotto did not have this key present, [`None`] is returned.
    ///
    /// If the surotto did have this key present, the value is updated, and the old
    /// value is returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let key = key.idx();

        let missing_slots = key - self.inner.len();
        self.inner
            .extend(iter::repeat_with(|| None).take(missing_slots));

        unsafe {
            // SAFETY: we just enlarged the bounds to make the slot at key in length.
            self.inner.get_unchecked_mut(key).replace(value)
        }
    }

    /// Removes a value from the surotto, returning the value at the
    /// key if the key was previously in the surotto.
    pub fn remove(&mut self, key: K) -> Option<V> {
        if let Some(slot) = self.inner.get_mut(key.idx()) {
            slot.take()
        } else {
            None
        }
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: K) -> Option<&V> {
        match self.inner.get(key.idx()) {
            Some(inner) => inner.as_ref(),
            None => None,
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        match self.inner.get_mut(key.idx()) {
            Some(inner) => inner.as_mut(),
            None => None,
        }
    }

    /// Returns a reference to an element without checking on the key or bounds
    pub unsafe fn get_unchecked(&self, key: K) -> &V {
        self.inner
            .get_unchecked(key.idx())
            .as_ref()
            .unwrap_unchecked()
    }

    /// Returns a mutable reference to an element without checking on the key or bounds
    pub unsafe fn get_unchecked_mut(&mut self, key: K) -> &mut V {
        self.inner
            .get_unchecked_mut(key.idx())
            .as_mut()
            .unwrap_unchecked()
    }

    /// Gets the given key's corresponding entry in the surotto for in-place manipulation.
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        if self.contains_key(key) {
            Entry::Occupied(OccupiedEntry { surotto: self, key })
        } else {
            Entry::Vacant(VacantEntry { surotto: self, key })
        }
    }

    /// Returns true if the map contains a value for the specified key.
    pub fn contains_key(&self, key: K) -> bool {
        self.inner
            .get(key.idx())
            .map(|o| o.is_some())
            .unwrap_or(false)
    }

    /// Returns true if the surotto contains no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of elements in the surotto, also referred to
    /// as its 'length'.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the total number of elements the surotto can hold without
    /// reallocating.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `SimpleAssocSurotto<K, V>`. The collection may reserve more space to
    /// speculatively avoid frequent reallocations. After calling `reserve`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    /// Reserves the minimum capacity for at least `additional` more elements to
    /// be inserted in the given `SimpleAssocSurotto<K, V>`. Unlike [`reserve`], this will not
    /// deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `reserve_exact`, capacity will be greater than or equal to
    /// `self.len() + additional`. Does nothing if the capacity is already
    /// sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore, capacity can not be relied upon to be precisely
    /// minimal. Prefer [`reserve`] if future insertions are expected.
    ///
    /// [`reserve`]: SimpleAssocSurotto::reserve
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional)
    }

    /// Tries to reserve capacity for at least `additional` more elements to be inserted
    /// in the given `SimpleAssocSurotto<K, V>`. The collection may reserve more space to speculatively avoid
    /// frequent reallocations. After calling `try_reserve`, capacity will be
    /// greater than or equal to `self.len() + additional` if it returns
    /// `Ok(())`. Does nothing if capacity is already sufficient. This method
    /// preserves the contents even if an error occurs.
    ///
    /// # Errors
    ///
    /// If the capacity overflows, or the allocator reports a failure, then an error
    /// is returned.
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve(additional)
    }

    /// Tries to reserve the minimum capacity for at least `additional`
    /// elements to be inserted in the given `SimpleAssocSurotto<K, V>`. Unlike [`try_reserve`],
    /// this will not deliberately over-allocate to speculatively avoid frequent
    /// allocations. After calling `try_reserve_exact`, capacity will be greater
    /// than or equal to `self.len() + additional` if it returns `Ok(())`.
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore, capacity can not be relied upon to be precisely
    /// minimal. Prefer [`try_reserve`] if future insertions are expected.
    ///
    /// [`try_reserve`]: SimpleAssocSurotto::try_reserve
    ///
    /// # Errors
    ///
    /// If the capacity overflows, or the allocator reports a failure, then an error
    /// is returned.
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.inner.try_reserve_exact(additional)
    }

    /// Shrinks the capacity of the surotto as much as possible.
    ///
    /// It will drop down as close as possible to the length but the allocator
    /// may still inform the vector that there is space for a few more elements.
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }

    /// Shrinks the capacity of the surotto with a lower bound.
    ///
    /// The capacity will remain at least as large as both the length
    /// and the supplied value.
    ///
    /// If the current capacity is less than the lower limit, this is a no-op.
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity)
    }
}

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
    surotto: &'a mut SimpleAssocSurotto<K, V>,
    key: K,
}

pub struct VacantEntry<'a, K: SimpleKey, V> {
    surotto: &'a mut SimpleAssocSurotto<K, V>,
    key: K,
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
