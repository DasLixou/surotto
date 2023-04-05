use core::iter;

use crate::{Key, Surotto, SUROTTO_OCCUPIED};

pub struct Keys<'s, T> {
    pub(crate) inner: iter::Enumerate<core::slice::Iter<'s, Surotto<T>>>,
}

impl<'s, T> Iterator for Keys<'s, T> {
    type Item = Key;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .find(|(_, surotto)| surotto.version & SUROTTO_OCCUPIED != 0)
            .map(|(index, surotto)| {
                let version = surotto.version;
                Key { index, version }
            })
    }
}
