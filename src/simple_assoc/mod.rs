use std::{collections::TryReserveError, iter, marker::PhantomData};

use crate::simple::SimpleKey;

use self::{
    entry::{Entry, OccupiedEntry, VacantEntry},
    iterators::{Iter, IterMut, Keys, Values, ValuesMut},
};

pub mod entry;
pub mod iterators;

/// A datastructure where values can be associated with a key from a [`SimpleSurotto`].
///
/// [`SimpleSurotto`]: crate::simple::SimpleSurotto
pub struct SimpleAssocSurotto<K: SimpleKey, V> {
    inner: Vec<Option<V>>,
    phantom: PhantomData<K>,
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

    /// An iterator visiting all key-value pairs.
    /// The iterator element type is `(K, &'a V)`.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            inner: self.inner.iter().enumerate(),
            phantom: PhantomData,
        }
    }

    /// An iterator visiting all key-value pairs,
    /// with mutable references to the values.
    /// The iterator element type is `(K, &'a mut V)`.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            inner: self.inner.iter_mut().enumerate(),
            phantom: PhantomData,
        }
    }

    /// An iterator visiting all keys.
    /// The iterator element type is `K`.
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys { inner: self.iter() }
    }

    /// An iterator visiting all values.
    /// The iterator element type is `&'a V`.
    pub fn values(&self) -> Values<'_, K, V> {
        Values { inner: self.iter() }
    }

    /// An iterator visiting all values mutably.
    /// The iterator element type is `&'a mut V`.
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            inner: self.iter_mut(),
        }
    }
}

impl<K: SimpleKey, V> Default for SimpleAssocSurotto<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
