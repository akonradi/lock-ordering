use lock_ordering::lock::MutexLockLevel;
use lock_ordering::lock::RwLockLevel;

#[cfg(all(doc, feature = "async"))]
use lock_ordering::lock::{AsyncMutexLock, AsyncRwLock};
#[cfg(feature = "async")]
use lock_ordering::lock::{AsyncMutexLockLevel, AsyncRwLockLevel};
#[cfg(doc)]
use lock_ordering::lock::{MutexLock, RwLock};

/// Implementing types correspond to [`MutexLock`] state in `T`.
///
/// This should be implemented on marker types that correspond to locked state
/// in `T`.
pub trait MutexLockedState<T> {
    /// The lock level associated with the locked state in `T`.
    type LockLevel: MutexLockLevel;

    /// Returns a reference to the corresponding mutex in `T`.
    fn mutex(t: &T) -> &<Self::LockLevel as MutexLockLevel>::Mutex;
}

/// Implementing types correspond to [`RwLock`] state in `T`.
///
/// This should be implemented on marker types that correspond to
/// read/write-locked state in `T`.
pub trait RwLockedState<T> {
    /// The lock level associated with the read/write-locked state in `T`.
    type LockLevel: RwLockLevel;

    /// Returns a reference to the corresponding read/write lock in `T`.
    fn rw_lock(t: &T) -> &<Self::LockLevel as RwLockLevel>::RwLock;
}

/// Implementing types correspond to [`AsyncMutexLock`] state in `T`.
///
/// This should be implemented on marker types that correspond to
/// asynchronously accessed locked state in `T`.
#[cfg(feature = "async")]
pub trait AsyncMutexLockedState<T> {
    /// The lock level associated with the async locked state in `T`.
    type LockLevel: AsyncMutexLockLevel;

    /// Returns a reference to the corresponding async mutex in `T`.
    fn mutex(t: &T) -> &<Self::LockLevel as AsyncMutexLockLevel>::Mutex;
}

/// Implementing types correspond to [`AsyncRwLock`] state in `T`.
///
/// This should be implemented on marker types that correspond to
/// asynchronously accessed read/write-locked state in `T`.
#[cfg(feature = "async")]
pub trait AsyncRwLockedState<T> {
    /// The lock level associated with the async read/write-locked state in `T`.
    type LockLevel: AsyncRwLockLevel;

    /// Returns a reference to the corresponding async read/write lock in `T`.
    fn rw_lock(t: &T) -> &<Self::LockLevel as AsyncRwLockLevel>::RwLock;
}
