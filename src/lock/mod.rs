//! Traits that describe how locked data is accessed.

pub use mutex::MutexLock;
pub use rwlock::RwLock;
#[cfg(feature = "async")]
pub use {mutex::AsyncMutexLock, rwlock::AsyncRwLock};

use crate::{LockLevel, MutualExclusion, ReadWrite};

mod mutex;
mod rwlock;

/// Connects a [`LockLevel`] with a [`MutexLock`] implementation.
pub trait MutexLockLevel: LockLevel<Method = MutualExclusion> {
    type Mutex: MutexLock;
}

/// Connects a [`LockLevel`] with a [`RwLock`] implementation.
pub trait RwLockLevel: LockLevel<Method = ReadWrite> {
    type RwLock: RwLock;
}

/// Connects a [`LockLevel`] with a [`MutexLock`] implementation.
#[cfg(feature = "async")]
pub trait AsyncMutexLockLevel: LockLevel<Method = MutualExclusion> {
    type Mutex: AsyncMutexLock;
}

/// Connects a [`LockLevel`] with a [`RwLock`] implementation.
#[cfg(feature = "async")]
pub trait AsyncRwLockLevel: LockLevel<Method = ReadWrite> {
    type RwLock: AsyncRwLock;
}
