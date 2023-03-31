pub mod into_iter;
pub mod iter;
pub mod iter_mut;
pub mod keys;
pub mod values;
pub mod values_mut;

use std::{
    mem::{self, MaybeUninit},
    ops::{Index, IndexMut},
};

use into_iter::IntoIter;
use iter::Iter;
use iter_mut::IterMut;
use keys::Keys;
use values::Values;
use values_mut::ValuesMut;

const SUROTTO_FREE: u32 = 0b0;
const SUROTTO_OCCUPIED: u32 = 0b1 << 31;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub(crate) index: usize,
    pub(crate) version: u32,
}

#[derive(Debug)]
pub(crate) struct Surotto<T> {
    pub(crate) val: MaybeUninit<T>,
    pub(crate) version: u32, // (S) 1 bit occupied(1) / free(0) | (V) 31 bits verison, increments on free | 0bSVV....VVV
    pub(crate) next_free: usize, // 0 -> push | i -> occupied at i - 1
}

impl<T> Drop for Surotto<T> {
    fn drop(&mut self) {
        if self.version & SUROTTO_OCCUPIED != 0 {
            // SAFETY: the slot is occupied, data is held
            unsafe { self.val.assume_init_drop() }
        }
    }
}

pub struct SurottoMap<T> {
    inner: Vec<Surotto<T>>,
    next_free: usize, // 0 -> push | i -> occupied at i - 1
    len: usize,
}

impl<T> SurottoMap<T> {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            next_free: 0,
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut inner = Vec::with_capacity(capacity);
        for i in 0..capacity - 1 {
            inner.push(Surotto {
                val: MaybeUninit::uninit(),
                version: SUROTTO_FREE,
                next_free: i + 2,
            });
        }
        inner.push(Surotto {
            val: MaybeUninit::uninit(),
            version: SUROTTO_FREE,
            next_free: 0,
        });
        Self {
            inner,
            next_free: 1,
            len: 0,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    pub fn big_len(&self) -> usize {
        self.inner.len()
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Returns `true` if the key is linked to an occupied slot with a correct version
    pub fn validate_key(&self, key: Key) -> bool {
        if let Some(surotto) = self.inner.get(key.index) {
            surotto.version | SUROTTO_OCCUPIED == key.version
                && surotto.version & SUROTTO_OCCUPIED != 0
        } else {
            false
        }
    }

    pub fn insert(&mut self, val: T) -> Key {
        if self.next_free == 0 {
            let pos = self.inner.len();
            self.inner.push(Surotto {
                val: MaybeUninit::new(val),
                version: SUROTTO_OCCUPIED,
                next_free: 0,
            });
            self.len += 1;
            Key {
                index: pos,
                version: SUROTTO_OCCUPIED,
            }
        } else {
            let pos = self.next_free - 1;
            let surotto = &mut self.inner[pos];
            debug_assert!(surotto.version & SUROTTO_OCCUPIED == 0);
            surotto.val.write(val);
            surotto.version |= SUROTTO_OCCUPIED;
            self.next_free = surotto.next_free;
            self.len += 1;
            Key {
                index: pos,
                version: surotto.version,
            }
        }
    }

    pub fn get(&self, key: Key) -> Option<&T> {
        if self.validate_key(key) {
            // SAFETY: we checked if it is a valid key: contained and occupied with correct version
            let surotto = unsafe { self.inner.get_unchecked(key.index) };
            unsafe { Some(surotto.val.assume_init_ref()) }
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self, key: Key) -> &T {
        // SAFETY: user promised
        let surotto = unsafe { self.inner.get_unchecked(key.index) };
        unsafe { surotto.val.assume_init_ref() }
    }

    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        if self.validate_key(key) {
            // SAFETY: we checked if it is a valid key: contained and occupied with correct version
            let surotto = unsafe { self.inner.get_unchecked_mut(key.index) };
            unsafe { Some(surotto.val.assume_init_mut()) }
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked_mut(&mut self, key: Key) -> &mut T {
        // SAFETY: user promised
        let surotto = unsafe { self.inner.get_unchecked_mut(key.index) };
        unsafe { surotto.val.assume_init_mut() }
    }

    pub fn get_disjoint<const N: usize>(&mut self, keys: [Key; N]) -> Option<[&T; N]> {
        // SAFETY: When return Some with this array, everything will be valid - code from unstable MaybeUninit::uninit_array()
        let mut refs = unsafe { MaybeUninit::<[MaybeUninit<&T>; N]>::uninit().assume_init() };
        let mut undo_idx = N;
        for (i, key) in keys.iter().enumerate() {
            if !self.validate_key(*key) {
                undo_idx = i;
                break;
            }

            // SAFETY: we checked if it is a valid key: contained and occupied with correct version
            let surotto = unsafe { self.inner.get_unchecked_mut(i) };
            surotto.version &= !SUROTTO_OCCUPIED;
            // SAFETY: the slot is occupied, data is held
            unsafe { refs[i].write(&*surotto.val.as_ptr()) };
        }
        for k in &keys[..undo_idx] {
            unsafe {
                self.inner.get_unchecked_mut(k.index).version |= SUROTTO_OCCUPIED;
            }
        }
        // modified code from unstable MaybeUninit::array_assume_init(refs)
        if undo_idx == N {
            // SAFETY:
            // * The caller guarantees that all elements of the array are initialized
            // * `MaybeUninit<T>` and T are guaranteed to have the same layout
            // * `MaybeUninit` does not drop, so there are no double-frees
            // And thus the conversion is safe
            let ret = unsafe {
                //intrinsics::assert_inhabited::<[&T; N]>();
                (&refs as *const _ as *const [&T; N]).read()
            };

            // FIXME: required to avoid `~const Destruct` bound
            mem::forget(refs);
            Some(ret)
        } else {
            None
        }
    }

    pub fn get_disjoint_mut<const N: usize>(&mut self, keys: [Key; N]) -> Option<[&mut T; N]> {
        // SAFETY: When return Some with this array, everything will be valid - code from unstable MaybeUninit::uninit_array()
        let mut refs = unsafe { MaybeUninit::<[MaybeUninit<&mut T>; N]>::uninit().assume_init() };
        let mut undo_idx = N;
        for (i, key) in keys.iter().enumerate() {
            if !self.validate_key(*key) {
                undo_idx = i;
                break;
            }

            // SAFETY: we checked if it is a valid key: contained and occupied with correct version
            let surotto = unsafe { self.inner.get_unchecked_mut(i) };
            surotto.version &= !SUROTTO_OCCUPIED;
            // SAFETY: the slot is occupied, data is held
            unsafe { refs[i].write(&mut *surotto.val.as_mut_ptr()) };
        }
        for k in &keys[..undo_idx] {
            unsafe {
                self.inner.get_unchecked_mut(k.index).version |= SUROTTO_OCCUPIED;
            }
        }
        // modified code from unstable MaybeUninit::array_assume_init(refs)
        if undo_idx == N {
            // SAFETY:
            // * The caller guarantees that all elements of the array are initialized
            // * `MaybeUninit<T>` and T are guaranteed to have the same layout
            // * `MaybeUninit` does not drop, so there are no double-frees
            // And thus the conversion is safe
            let ret = unsafe {
                //intrinsics::assert_inhabited::<[&mut T; N]>();
                (&refs as *const _ as *const [&mut T; N]).read()
            };

            // FIXME: required to avoid `~const Destruct` bound
            mem::forget(refs);
            Some(ret)
        } else {
            None
        }
    }

    pub fn remove(&mut self, key: Key) -> Option<T> {
        if self.validate_key(key) {
            // SAFETY: we checked if it is a valid key: contained and occupied with correct version
            let surotto = unsafe { self.inner.get_unchecked_mut(key.index) };
            // SAFETY: we will mark it as free or overwrite later, no double free
            let val = unsafe { surotto.val.assume_init_read() };
            surotto.version = (surotto.version + 1) & !SUROTTO_OCCUPIED;
            surotto.next_free = self.next_free;
            self.next_free = key.index + 1;
            self.len -= 1;
            Some(val)
        } else {
            None
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.inner.iter().enumerate(),
        }
    }
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            inner: self.inner.iter_mut().enumerate(),
        }
    }
    #[inline]
    pub fn keys(&self) -> Keys<'_, T> {
        Keys {
            inner: self.inner.iter().enumerate(),
        }
    }
    #[inline]
    pub fn values(&self) -> Values<'_, T> {
        Values {
            inner: self.inner.iter(),
        }
    }
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, T> {
        ValuesMut {
            inner: self.inner.iter_mut(),
        }
    }
}

impl<T> Index<Key> for SurottoMap<T> {
    type Output = T;

    #[inline]
    fn index(&self, key: Key) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<T> IndexMut<Key> for SurottoMap<T> {
    #[inline]
    fn index_mut(&mut self, key: Key) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}

impl<T> IntoIterator for SurottoMap<T> {
    type Item = (Key, T);
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.inner.into_iter().enumerate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_insert() {
        let mut map: SurottoMap<String> = SurottoMap::new();

        let pos1 = map.insert(String::from("Hello"));
        let pos2 = map.insert(String::from("World"));

        assert_eq!(map.len(), 2);
        assert_eq!(map.big_len(), 2);
        assert_eq!(map.get(pos1), Some(&String::from("Hello")));
        assert_eq!(map.get(pos2), Some(&String::from("World")));
    }

    #[test]
    fn test_do_insert() {
        let mut map: SurottoMap<String> = SurottoMap::with_capacity(2);

        let pos1 = map.insert(String::from("Hello"));
        let pos2 = map.insert(String::from("World"));

        assert_eq!(map.len(), 2);
        assert_eq!(map.big_len(), 2);
        assert_eq!(map.get(pos1), Some(&String::from("Hello")));
        assert_eq!(map.get(pos2), Some(&String::from("World")));
    }

    #[test]
    fn test_get_mut() {
        let mut map: SurottoMap<String> = SurottoMap::new();

        let pos1 = map.insert(String::from("Hello"));
        let pos2 = map.insert(String::from("World"));

        if let Some(val) = map.get_mut(pos1) {
            *val = String::from("Goodbye");
        }

        assert_eq!(map.len(), 2);
        assert_eq!(map.get(pos1), Some(&String::from("Goodbye")));
        assert_eq!(map.get(pos2), Some(&String::from("World")));
    }

    #[test]
    fn test_get_out_of_bounds() {
        let map: SurottoMap<String> = SurottoMap::new();

        assert_eq!(
            map.get(Key {
                index: 50,
                version: 0
            }),
            None
        );
    }

    #[test]
    fn test_remove() {
        let mut map: SurottoMap<String> = SurottoMap::new();

        let _pos1 = map.insert(String::from("Hello"));
        let pos2 = map.insert(String::from("World"));
        let _pos3 = map.insert(String::from("Surotto"));

        assert_eq!(
            map.remove(Key {
                index: 50,
                version: 0
            }),
            None
        );

        assert_eq!(map.remove(pos2), Some(String::from("World")));
        assert_eq!(map.get(pos2), None);
        assert_eq!(map.len(), 2);
        assert_eq!(map.big_len(), 3);
        assert_eq!(map.remove(pos2), None);

        let repos2 = map.insert(String::from("Second World"));

        assert_eq!(pos2.index, repos2.index);
        assert_ne!(pos2.version, repos2.version);
        assert_eq!(map.get(repos2), Some(&String::from("Second World")));
        assert_eq!(map.len(), 3);
        assert_eq!(map.big_len(), 3);
    }

    #[test]
    fn test_old_versioned_key() {
        let mut map: SurottoMap<String> = SurottoMap::new();

        let _pos1 = map.insert(String::from("Hello"));
        let pos2 = map.insert(String::from("World"));
        let _pos3 = map.insert(String::from("Surotto"));

        assert_eq!(map.get(pos2), Some(&String::from("World")));
        assert_eq!(map.remove(pos2), Some(String::from("World")));
        assert_eq!(map.get(pos2), None);

        let repos2 = map.insert(String::from("World"));
        assert_eq!(pos2.index, repos2.index);
        assert_ne!(pos2.version, repos2.version);

        assert_eq!(map.get(pos2), None);
        assert_eq!(map.get(repos2), Some(&String::from("World")));
    }

    #[test]
    fn test_into_iter() {
        let mut map: SurottoMap<String> = SurottoMap::new();

        let pos1 = map.insert(String::from("Hello"));
        let pos2 = map.insert(String::from("World"));
        let pos3 = map.insert(String::from("Surotto"));

        assert_eq!(
            map.into_iter().collect::<Vec<_>>().as_slice(),
            &[
                (pos1, String::from("Hello")),
                (pos2, String::from("World")),
                (pos3, String::from("Surotto"))
            ]
        );
    }
}
