pub use mutex::MutexLock;
pub use rwlock::RwLock;

mod mutex {

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
}

mod rwlock {
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
}
