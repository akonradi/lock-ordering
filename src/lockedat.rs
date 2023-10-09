use core::marker::PhantomData;

use crate::{
    lock::{MutexLock, RwLock},
    relation::LockAfter,
    LockLevel, Unlocked,
};

pub struct MutualExclusion;
pub struct ReadWrite;

/// Empty type that enforces lock acquisition ordering.
///
/// This type wraps a lock level `L` representing the level of the "currently
/// held" lock. When a new lock is acquired via one of the inherent methods on
/// this type, the `LockedAt` instance used to acquire the lock remains borrowed
/// until the lock, and any locks acquired after, are released.
pub struct LockedAt<'a, L>(PhantomData<&'a mut L>);

impl LockedAt<'static, Unlocked> {
    /// Creates a new `LockedAt` without any locks held.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<L> LockedAt<'_, L> {
    pub fn with_lock<
        'a,
        NewLock: LockAfter<L> + LockLevel<Method = MutualExclusion>,
        T: MutexLock,
    >(
        &'a mut self,
        t: &'a T,
    ) -> Result<(LockedAt<'a, NewLock>, T::Guard<'a>), T::Error<'a>> {
        t.lock().map(|guard| (LockedAt(PhantomData), guard))
    }

    pub fn with_read_lock<'a, NewLock: LockAfter<L> + LockLevel<Method = ReadWrite>, T: RwLock>(
        &'a mut self,
        t: &'a T,
    ) -> Result<(LockedAt<'a, NewLock>, T::ReadGuard<'a>), T::ReadError<'a>> {
        t.read().map(|guard| (LockedAt(PhantomData), guard))
    }

    pub fn with_write_lock<'a, NewLock: LockAfter<L> + LockLevel<Method = ReadWrite>, T: RwLock>(
        &'a mut self,
        t: &'a T,
    ) -> Result<(LockedAt<'a, NewLock>, T::WriteGuard<'a>), T::WriteError<'a>> {
        t.write().map(|guard| (LockedAt(PhantomData), guard))
    }
}

// Convenience methods for accessing leaf locks in the ordering tree.
impl<L> LockedAt<'_, L> {
    pub fn lock<
        'a,
        NewLock: LockAfter<L> + 'a + LockLevel<Method = MutualExclusion>,
        T: MutexLock,
    >(
        &'a mut self,
        t: &'a T,
    ) -> Result<T::Guard<'a>, T::Error<'a>> {
        self.with_lock::<NewLock, T>(t)
            .map(|(_locked, guard)| guard)
    }

    pub fn read_lock<'a, NewLock: LockAfter<L> + LockLevel<Method = ReadWrite> + 'a, T: RwLock>(
        &'a mut self,
        t: &'a T,
    ) -> Result<T::ReadGuard<'a>, T::ReadError<'a>> {
        self.with_read_lock::<NewLock, T>(t)
            .map(|(_locked, guard)| guard)
    }

    pub fn write_lock<'a, NewLock: LockAfter<L> + LockLevel<Method = ReadWrite> + 'a, T: RwLock>(
        &'a mut self,
        t: &'a T,
    ) -> Result<T::WriteGuard<'a>, T::WriteError<'a>> {
        self.with_write_lock::<NewLock, T>(t)
            .map(|(_locked, guard)| guard)
    }
}
