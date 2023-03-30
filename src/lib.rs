use std::mem::MaybeUninit;

const SUROTTO_FREE: u32 = 0b00000000;
const SUROTTO_OCCUPIED: u32 = 0b10000000;

struct Surotto<T> {
    val: MaybeUninit<T>,
    version: u32, // (S) 1 bit occupied(1) / free(0) | (V) 7 bits verison, increments on free | 0bSVVVVVVV
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
            let pos = self.next_free;
            let surotto = &mut self.inner[pos];
            debug_assert!(surotto.version & SUROTTO_OCCUPIED == 0);
            surotto.val.write(val);
            surotto.version |= SUROTTO_OCCUPIED;
            self.next_free = surotto.next_free;
            self.len += 1;
            pos
        }
    }
}
