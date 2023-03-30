use std::mem::MaybeUninit;

const SUROTTO_FREE: u32 = 0b00000000;
const SUROTTO_OCCUPIED: u32 = 0b10000000;

struct Surotto<T> {
    val: MaybeUninit<T>,
    version: u32, // (S) 1 bit occupied(1) / free(0) | (V) 7 bits verison | 0bSVVVVVVV
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
}

impl<T> SurottoMap<T> {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            next_free: 0,
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
        }
    }

    pub fn insert(&mut self, val: T) -> usize {
        if self.next_free == 0 {
            let pos = self.inner.len();
            self.inner.push(Surotto {
                val: MaybeUninit::new(val),
                version: 0 | SUROTTO_OCCUPIED,
                next_free: 0,
            });
            pos
        } else {
            todo!()
        }
    }
}
