/// Locking implementation for [crate::MutualExclusion].
///
/// Describes how to acquire access to the state for a [crate::LockLevel]
/// implementation with [Method](crate::LockLevel::Method) = `MutualExclusion`.
/// The error and RAII guard types are implementation-defined.
pub trait MutexLock {
    /// Error that could be produced when acquiring the lock.
    ///
    /// For implementations where acquiring a lock is an infallible operation,
    /// the error type [`core::convert::Infallible`] can be used.
    type Error<'a>
    where
        Self: 'a;

    /// [RAII guard] for accessing data protected by the lock.
    ///
    /// [RAII guard]: https://doc.rust-lang.org/rust-by-example/scope/raii.html
    type Guard<'a>
    where
        Self: 'a;

    /// Attempts to acquire exclusive access to data.
    ///
    /// Returns an RAII guard that provides access to the data, or an error on
    /// failure.
    fn lock(&self) -> Result<Self::Guard<'_>, Self::Error<'_>>;
}

#[cfg(feature = "std")]
mod std {
    //! Implementation of [`MutexLock`] for [`std::sync::Mutex`].

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
