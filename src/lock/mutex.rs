pub trait MutexLock {
    type Error<'a>
    where
        Self: 'a;
    type Guard<'a>
    where
        Self: 'a;

    fn lock(&self) -> Result<Self::Guard<'_>, Self::Error<'_>>;
}

#[cfg(feature = "std")]
mod std {
    use std::sync::{Mutex, MutexGuard, PoisonError};

    use super::MutexLock;

    impl<T: ?Sized> MutexLock for Mutex<T> {
        type Guard<'a> = MutexGuard<'a, T> where Self: 'a;
        type Error<'a> = PoisonError<MutexGuard<'a, T>>
    where
        Self: 'a;

        fn lock(&self) -> Result<Self::Guard<'_>, Self::Error<'_>> {
            Mutex::lock(self)
        }
    }
}
