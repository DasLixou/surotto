use crate::{Surotto, SUROTTO_OCCUPIED};

pub struct ValuesMut<'s, T> {
    pub(crate) inner: core::slice::IterMut<'s, Surotto<T>>,
}

impl<'s, T> Iterator for ValuesMut<'s, T> {
    type Item = &'s mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .find(|surotto| surotto.version & SUROTTO_OCCUPIED != 0)
            .map(|surotto| {
                // SAFETY: the slot is occupied
                unsafe { surotto.val.assume_init_mut() }
            })
    }
}
