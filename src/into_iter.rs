use std::mem;

use crate::{Key, Surotto, SUROTTO_OCCUPIED};

pub struct IntoIter<T> {
    pub(crate) inner: std::iter::Enumerate<std::vec::IntoIter<Surotto<T>>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = (Key, T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .find(|(_, surotto)| surotto.version & SUROTTO_OCCUPIED != 0)
            .map(|(index, surotto)| {
                let version = surotto.version;
                // SAFETY: the slot is occupied and will be leaked after
                let val = unsafe { surotto.val.assume_init_read() };
                mem::forget(surotto);
                (Key { index, version }, val)
            })
    }
}
