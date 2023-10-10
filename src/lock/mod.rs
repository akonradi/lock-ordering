//! Traits that describe how locked data is accessed.

pub use mutex::MutexLock;
pub use rwlock::RwLock;

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
