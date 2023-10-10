pub trait RwLock {
    type ReadError<'a>
    where
        Self: 'a;
    type WriteError<'a>
    where
        Self: 'a;

    type ReadGuard<'a>
    where
        Self: 'a;
    type WriteGuard<'a>
    where
        Self: 'a;

    fn read(&self) -> Result<Self::ReadGuard<'_>, Self::ReadError<'_>>;
    fn write(&self) -> Result<Self::WriteGuard<'_>, Self::WriteError<'_>>;
}

#[cfg(feature = "std")]
mod std {
    use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

    impl<T: ?Sized> super::RwLock for RwLock<T> {
        type ReadError<'a> = PoisonError<RwLockReadGuard<'a, T>> where Self: 'a ;
        type WriteError<'a> = PoisonError<RwLockWriteGuard<'a, T>> where Self: 'a;

        type ReadGuard<'a> = RwLockReadGuard<'a, T> where Self: 'a ;
        type WriteGuard<'a> = RwLockWriteGuard<'a, T> where Self: 'a;

        fn read(&self) -> Result<Self::ReadGuard<'_>, Self::ReadError<'_>> {
            RwLock::read(self)
        }

        fn write(&self) -> Result<Self::WriteGuard<'_>, Self::WriteError<'_>> {
            RwLock::write(self)
        }
    }
}
