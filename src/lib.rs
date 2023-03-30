use std::mem::MaybeUninit;

const SUROTTO_FREE: u32 = 0b0;
const SUROTTO_OCCUPIED: u32 = 0b1 << 31;

#[derive(Debug)]
struct Surotto<T> {
    val: MaybeUninit<T>,
    version: u32, // (S) 1 bit occupied(1) / free(0) | (V) 31 bits verison, increments on free | 0bSVV....VVV
    next_free: usize, // 0 -> push | i -> occupied at i - 1
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
                version: 0 | SUROTTO_FREE,
                next_free: i + 2,
            });
        }
        inner.push(Surotto {
            val: MaybeUninit::uninit(),
            version: 0 | SUROTTO_FREE,
            next_free: 0,
        });
        Self {
            inner,
            next_free: 1,
            len: 0,
        }
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

    pub fn insert(&mut self, val: T) -> usize {
        if self.next_free == 0 {
            let pos = self.inner.len();
            self.inner.push(Surotto {
                val: MaybeUninit::new(val),
                version: 0 | SUROTTO_OCCUPIED,
                next_free: 0,
            });
            self.len += 1;
            pos
        } else {
            let pos = self.next_free - 1;
            let surotto = &mut self.inner[pos];
            debug_assert!(surotto.version & SUROTTO_OCCUPIED == 0);
            surotto.val.write(val);
            surotto.version |= SUROTTO_OCCUPIED;
            self.next_free = surotto.next_free;
            self.len += 1;
            pos
        }
    }

    pub fn get(&self, key: usize) -> Option<&T> {
        if let Some(surotto) = self.inner.get(key) {
            if surotto.version & SUROTTO_OCCUPIED != 0 {
                // SAFETY: the slot is occupied, data is held
                unsafe { Some(surotto.val.assume_init_ref()) }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        if let Some(surotto) = self.inner.get_mut(key) {
            if surotto.version & SUROTTO_OCCUPIED != 0 {
                // SAFETY: the slot is occupied, data is held
                unsafe { Some(surotto.val.assume_init_mut()) }
            } else {
                None
            }
        } else {
            None
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

        assert_eq!(map.get(0), None);
    }
}
