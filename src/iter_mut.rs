use crate::{Key, Surotto, SUROTTO_OCCUPIED};

pub struct IterMut<'s, T> {
    pub(crate) inner: std::iter::Enumerate<std::slice::IterMut<'s, Surotto<T>>>,
}

impl<'s, T> Iterator for IterMut<'s, T> {
    type Item = (Key, &'s mut T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .find(|(_, surotto)| surotto.version & SUROTTO_OCCUPIED != 0)
            .map(|(index, surotto)| {
                let version = surotto.version;
                // SAFETY: the slot is occupied
                let val = unsafe { surotto.val.assume_init_mut() };
                (Key { index, version }, val)
            })
    }
}