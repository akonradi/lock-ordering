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
/// This should be implemented on marker types that correspond to locked state in `T`.
pub trait MutexLockedState<T> {
    type LockLevel: MutexLockLevel;
    /// Returns a reference to the corresponding mutex in `T`.
    fn mutex(t: &T) -> &<Self::LockLevel as MutexLockLevel>::Mutex;
}

/// Implementing types correspond to [`MutexLock`] state in `T`.
///
/// This should be implemented on marker types that correspond to read/write-locked state in `T`.
pub trait RwLockedState<T> {
    type LockLevel: RwLockLevel;
    /// Returns a reference to the corresponding mutex in `T`.
    fn rw_lock(t: &T) -> &<Self::LockLevel as RwLockLevel>::RwLock;
}

#[cfg(feature = "async")]
pub trait AsyncMutexLockedState<T> {
    type LockLevel: AsyncMutexLockLevel;
    fn mutex(t: &T) -> &<Self::LockLevel as AsyncMutexLockLevel>::Mutex;
}

#[cfg(feature = "async")]
pub trait AsyncRwLockedState<T> {
    type LockLevel: AsyncRwLockLevel;
    fn rw_lock(t: &T) -> &<Self::LockLevel as AsyncRwLockLevel>::RwLock;
}
