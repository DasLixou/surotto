use crate::{Surotto, SUROTTO_OCCUPIED};

pub struct Values<'s, T> {
    pub(crate) inner: std::slice::Iter<'s, Surotto<T>>,
}

impl<'s, T> Iterator for Values<'s, T> {
    type Item = &'s T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .find(|surotto| surotto.version & SUROTTO_OCCUPIED != 0)
            .map(|surotto| {
                // SAFETY: the slot is occupied
                unsafe { surotto.val.assume_init_ref() }
            })
    }
}
