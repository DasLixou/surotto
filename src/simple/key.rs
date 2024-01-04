pub unsafe trait SimpleKey: Sized + Clone + Copy {
    unsafe fn new(idx: usize) -> Self;
    fn idx(self) -> usize;
}

#[macro_export]
macro_rules! simple_key {
    ($vis:vis struct $name:ident;) => {
        #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
        #[repr(transparent)]
        $vis struct $name(usize);

        unsafe impl $crate::simple::SimpleKey for $name {
            unsafe fn new(idx: usize) -> Self {
                Self(idx)
            }

            fn idx(self) -> usize {
                self.0
            }
        }
    };
}
