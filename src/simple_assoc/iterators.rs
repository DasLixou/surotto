use std::{iter, marker::PhantomData};

use crate::simple::SimpleKey;

pub struct Iter<'a, K: SimpleKey, V> {
    pub(super) inner: iter::Enumerate<core::slice::Iter<'a, Option<V>>>,
    pub(super) phantom: PhantomData<K>,
}

impl<'a, K: SimpleKey, V> Iterator for Iter<'a, K, V> {
    type Item = (K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find_map(|(i, opt)| {
            opt.as_ref().map(|val| {
                (
                    unsafe {
                        // SAFETY: The iterator only returns elements which are present and
                        //          elements can't be removed from the `SimpleSurotto`, thus the creation of the key is safe here.
                        K::new(i)
                    },
                    val,
                )
            })
        })
    }
}

pub struct IterMut<'a, K: SimpleKey, V> {
    pub(super) inner: iter::Enumerate<core::slice::IterMut<'a, Option<V>>>,
    pub(super) phantom: PhantomData<K>,
}

impl<'a, K: SimpleKey, V> Iterator for IterMut<'a, K, V> {
    type Item = (K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find_map(|(i, opt)| {
            opt.as_mut().map(|val| {
                (
                    unsafe {
                        // SAFETY: The iterator only returns elements which are present and
                        //          elements can't be removed from the `SimpleSurotto`, thus the creation of the key is safe here.
                        K::new(i)
                    },
                    val,
                )
            })
        })
    }
}

pub struct Keys<'a, K: SimpleKey, V> {
    pub(super) inner: Iter<'a, K, V>,
}

impl<'a, K: SimpleKey, V> Iterator for Keys<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, _)| key)
    }
}

pub struct Values<'a, K: SimpleKey, V> {
    pub(super) inner: Iter<'a, K, V>,
}

impl<'a, K: SimpleKey, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, val)| val)
    }
}

pub struct ValuesMut<'a, K: SimpleKey, V> {
    pub(super) inner: IterMut<'a, K, V>,
}

impl<'a, K: SimpleKey, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, val)| val)
    }
}
