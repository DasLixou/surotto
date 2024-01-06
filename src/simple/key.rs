pub type SimpleKeyData = nonmax::NonMaxUsize;

pub unsafe trait SimpleKey: Sized + Clone + Copy {
    /// Creates a new key from usize
    ///
    /// # Panics
    ///
    /// When idx is `usize::MAX`.
    ///
    /// # Safety
    ///
    /// The corresponding entry must be present in the surotto
    unsafe fn new(idx: usize) -> Self;
    fn idx(self) -> usize;
}

#[macro_export]
macro_rules! simple_key {
    ($vis:vis struct $name:ident;) => {
        #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
        #[repr(transparent)]
        $vis struct $name($crate::simple::SimpleKeyData);

        unsafe impl $crate::simple::SimpleKey for $name {
            unsafe fn new(idx: usize) -> Self {
                Self($crate::simple::SimpleKeyData::new(idx).unwrap())
            }

            fn idx(self) -> usize {
                self.0.get()
            }
        }
    };
}
