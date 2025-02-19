/// Locking implementation for [crate::ReadWrite].
///
/// Describes how to acquire access to the state for a [crate::LockLevel]
/// implementation with [Method](crate::LockLevel::Method) = `ReadWrite`.
/// The error and RAII guard types are implementation-defined.
pub trait RwLock {
    /// Error that could be produced when acquiring read access.
    ///
    /// For implementations where acquiring a lock is an infallible operation,
    /// the error type [`core::convert::Infallible`] can be used.
    type ReadError<'a>
    where
        Self: 'a;

    /// Error that could be produced when acquiring write access.
    ///
    /// For implementations where acquiring a lock is an infallible operation,
    /// the error type [`core::convert::Infallible`] can be used.
    type WriteError<'a>
    where
        Self: 'a;

    /// [RAII guard] for shared access to data protected by the lock.
    ///
    /// [RAII guard]: https://doc.rust-lang.org/rust-by-example/scope/raii.html
    type ReadGuard<'a>
    where
        Self: 'a;

    /// [RAII guard] for exclusive access to data protected by the lock.
    ///
    /// [RAII guard]: https://doc.rust-lang.org/rust-by-example/scope/raii.html
    type WriteGuard<'a>
    where
        Self: 'a;

    /// Attempts to acquire shared access to data.
    ///
    /// Returns an RAII guard that provides shared (read) access to the data, or
    /// an error on failure.
    fn read(&self) -> Result<Self::ReadGuard<'_>, Self::ReadError<'_>>;

    /// Attempts to acquire exclusive access to data.
    ///
    /// Returns an RAII guard that provides exclusive (read/write) access to the
    /// data, or an error on failure.
    fn write(&self) -> Result<Self::WriteGuard<'_>, Self::WriteError<'_>>;
}

#[cfg(feature = "std")]
mod std {
    //! Implementation of [`RwLock`] for [`std::sync::RwLock`].
    //!
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

/// Async locking implementation for [crate::ReadWrite].
///
/// Describes how to acquire access to the state for a [crate::LockLevel]
/// implementation with [Method](crate::LockLevel::Method) = `ReadWrite`.
/// The error and RAII guard types are implementation-defined.
#[cfg(feature = "async")]
pub trait AsyncRwLock {
    /// [RAII guard] for shared access to data protected by the lock.
    ///
    /// [RAII guard]: https://doc.rust-lang.org/rust-by-example/scope/raii.html
    type ReadGuard<'a>
    where
        Self: 'a;

    /// [RAII guard] for exclusive access to data protected by the lock.
    ///
    /// [RAII guard]: https://doc.rust-lang.org/rust-by-example/scope/raii.html
    type WriteGuard<'a>
    where
        Self: 'a;

    /// Acquires shared access to data.
    ///
    /// Locks the data in `self` for shared (read) access, yielding the current
    /// task until the lock has been acquired.
    fn read(&self) -> impl core::future::Future<Output = Self::ReadGuard<'_>>;

    /// Acquires exclusive access to data.
    ///
    /// Locks the data in `self` for exclusive (read/write) access, yielding the
    /// current task until the lock has been acquired.
    fn write(&self) -> impl core::future::Future<Output = Self::WriteGuard<'_>>;
}

#[cfg(feature = "tokio")]
mod tokio {
    use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

    impl<T: ?Sized> super::AsyncRwLock for RwLock<T> {
        type ReadGuard<'a> = RwLockReadGuard<'a, T> where Self: 'a ;

        type WriteGuard<'a> = RwLockWriteGuard<'a, T> where Self: 'a;

        async fn read(&self) -> Self::ReadGuard<'_> {
            RwLock::read(self).await
        }

        async fn write(&self) -> Self::WriteGuard<'_> {
            RwLock::write(self).await
        }
    }
}
