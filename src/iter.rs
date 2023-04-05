use core::iter;

use crate::{Key, Surotto, SUROTTO_OCCUPIED};

pub struct Iter<'s, T> {
    pub(crate) inner: iter::Enumerate<core::slice::Iter<'s, Surotto<T>>>,
}

impl<'s, T> Iterator for Iter<'s, T> {
    type Item = (Key, &'s T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .find(|(_, surotto)| surotto.version & SUROTTO_OCCUPIED != 0)
            .map(|(index, surotto)| {
                let version = surotto.version;
                // SAFETY: the slot is occupied
                let val = unsafe { surotto.val.assume_init_ref() };
                (Key { index, version }, val)
            })
    }
}
