use std::{
    collections::TryReserveError,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

pub mod iterators;
use self::iterators::{IntoIter, Iter, IterMut, Keys, Values, ValuesMut};

mod key;
pub use self::key::*;

/// A datastructure where values can only be inserted, returning a typed key.
///
/// # Important
///
/// The key type must be unique to this and only this surotto.
/// This is required for safely getting values without `Option`s.
/// Associated surottos are still allowed tho, because they don't create any keys.
pub struct SimpleSurotto<K: SimpleKey, V> {
    inner: Vec<V>,
    phantom: PhantomData<K>,
}

impl<K: SimpleKey, V> SimpleSurotto<K, V> {
    /// Constructs a new, empty `SimpleSurotto<K, V>`.
    ///
    /// The surotto will not allocate until elements are inserted.
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Constructs a new, empty `SimpleSurotto<K, V>` with at least the specified capacity.
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

    /// Inserts a value into the surotto, returning its key.
    ///
    /// # Panics
    ///
    /// Panics when the key index is `usize::MAX`.
    pub fn insert(&mut self, value: V) -> K {
        let key = unsafe {
            // SAFETY: Caller assures that we are the surotto allowed to create keys.
            K::new(self.inner.len())
        };
        self.inner.push(value);
        key
    }

    /// Inserts a value from the closure with the key into the surotto,
    /// returning its key.
    pub fn insert_with<F>(&mut self, f: F) -> K
    where
        F: FnOnce(K) -> V,
    {
        let key = unsafe {
            // SAFETY: Caller assures that we are the surotto allowed to create keys.
            K::new(self.inner.len())
        };
        self.inner.push(f(key));
        key
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: K) -> &V {
        unsafe {
            // SAFETY: The caller assures that the keys are only from this surotto.
            //          Hence only we create valid keys and can't remove any value, it's safe.
            self.inner.get_unchecked(key.idx())
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: K) -> &mut V {
        unsafe {
            // SAFETY: The caller assures that the keys are only from this surotto.
            //          Hence only we create valid keys and can't remove any value, it's safe.
            self.inner.get_unchecked_mut(key.idx())
        }
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
    /// in the given `SimpleSurotto<K, V>`. The collection may reserve more space to
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
    /// be inserted in the given `SimpleSurotto<K, V>`. Unlike [`reserve`], this will not
    /// deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `reserve_exact`, capacity will be greater than or equal to
    /// `self.len() + additional`. Does nothing if the capacity is already
    /// sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore, capacity can not be relied upon to be precisely
    /// minimal. Prefer [`reserve`] if future insertions are expected.
    ///
    /// [`reserve`]: SimpleSurotto::reserve
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional)
    }

    /// Tries to reserve capacity for at least `additional` more elements to be inserted
    /// in the given `SimpleSurotto<K, V>`. The collection may reserve more space to speculatively avoid
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
    /// elements to be inserted in the given `SimpleSurotto<K, V>`. Unlike [`try_reserve`],
    /// this will not deliberately over-allocate to speculatively avoid frequent
    /// allocations. After calling `try_reserve_exact`, capacity will be greater
    /// than or equal to `self.len() + additional` if it returns `Ok(())`.
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore, capacity can not be relied upon to be precisely
    /// minimal. Prefer [`try_reserve`] if future insertions are expected.
    ///
    /// [`try_reserve`]: SimpleSurotto::try_reserve
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

    /// Maps the surotto over a function, retaining its keys
    pub fn map<F, T>(self, map: F) -> SimpleSurotto<K, T>
    where
        F: Fn(K, V) -> T,
    {
        SimpleSurotto {
            inner: self
                .inner
                .into_iter()
                .enumerate()
                .map(|(k, v)| {
                    map(
                        unsafe {
                            // SAFETY: the key is present in the map
                            K::new(k)
                        },
                        v,
                    )
                })
                .collect(),
            phantom: PhantomData,
        }
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

impl<K: SimpleKey, V> Default for SimpleSurotto<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: SimpleKey, V> Index<K> for SimpleSurotto<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.get(key)
    }
}

impl<K: SimpleKey, V> IndexMut<K> for SimpleSurotto<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.get_mut(key)
    }
}

impl<K: SimpleKey, V> IntoIterator for SimpleSurotto<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.inner.into_iter().enumerate(),
            phantom: PhantomData,
        }
    }
}
