use core::marker::PhantomData;

use crate::{
    lock::{MutexLock, RwLock},
    relation::LockAfter,
    LockLevel, Unlocked,
};

/// Indicator type for a mutual exclusion lock.
///
/// This can be used as the [`LockLevel::Method`] associated type for lock
/// levels whose data is accessed by enforcing [mutual exclusion].
///
/// [mutual exclusion]: https://en.wikipedia.org/wiki/Mutual_exclusion
pub struct MutualExclusion;

/// Indicator type for a shared-read/exclusive-write lock.
///
/// This can be used as the [`LockLevel::Method`] associated type for lock
/// levels whose data can be accessed either by [multiple simultaneous readers
/// or by a single
/// writer](https://en.wikipedia.org/wiki/Readers%E2%80%93writer_lock).
pub struct ReadWrite;

/// Empty type that enforces lock acquisition ordering.
///
/// This type wraps a lock level `L` representing the level of the "currently
/// held" lock, and provides methods for accessing state for other lock levels.
/// For a given `L`, the methods on `LockedAt<'_, L>` will allow accessing state
/// for a lock level `M` if [`M: LockAfter<L>`](crate::relation::LockAfter).
///
/// The `with_` methods on this type will (if they don't return an error),
/// produce two values: a new `LockedAt` instance and an accessor for locked
/// state.  Both values will exclusively borrow the original `LockedAt`
/// instance, preventing its use, until the new values go out of scope.
pub struct LockedAt<'a, L>(PhantomData<&'a mut L>);

impl LockedAt<'static, Unlocked> {
    /// Creates a new `LockedAt` without any locks held.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<L> LockedAt<'_, L> {
    /// Attempts to acquire a lock on `NewLock` state.
    ///
    /// Assuming `NewLock` is a lock level that can be acquired after `L`, this
    /// method provides access to state held in the [`MutexLock`] type T. If the
    /// lock acquisition fails, an error will be returned. Otherwise, this
    /// method returns a new `LockedAt` along with an accessor for the held
    /// state.
    ///
    /// If no further `LockedAt` calls need to be made after this one, consider
    /// using [`LockedAt::lock`] instead.
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

    /// Attempts to acquire a shared lock on `NewLock` state.
    ///
    /// Assuming `NewLock` is a lock level that can be acquired after `L`, this
    /// method provides access to state held in the [`ReadWrite`] type T. If the
    /// lock acquisition fails, an error will be returned. Otherwise, this
    /// method returns a new `LockedAt` along with a read-only accessor for the
    /// held state.
    ///
    /// If no further `LockedAt` calls need to be made after this one, consider
    /// using [`LockedAt::read_lock`] instead.
    pub fn with_read_lock<'a, NewLock: LockAfter<L> + LockLevel<Method = ReadWrite>, T: RwLock>(
        &'a mut self,
        t: &'a T,
    ) -> Result<(LockedAt<'a, NewLock>, T::ReadGuard<'a>), T::ReadError<'a>> {
        t.read().map(|guard| (LockedAt(PhantomData), guard))
    }

    /// Attempts to acquire an exclusive lock on `NewLock` state.
    ///
    /// Assuming `NewLock` is a lock level that can be acquired after `L`, this
    /// method provides access to state held in the [`ReadWrite`] type T. If the
    /// lock acquisition fails, an error will be returned. Otherwise, this
    /// method returns a new `LockedAt` along with a read/write accessor for the
    /// held state.
    ///
    /// If no further `LockedAt` calls need to be made after this one, consider
    /// using [`LockedAt::write_lock`] instead.
    pub fn with_write_lock<'a, NewLock: LockAfter<L> + LockLevel<Method = ReadWrite>, T: RwLock>(
        &'a mut self,
        t: &'a T,
    ) -> Result<(LockedAt<'a, NewLock>, T::WriteGuard<'a>), T::WriteError<'a>> {
        t.write().map(|guard| (LockedAt(PhantomData), guard))
    }
}

// Convenience methods for accessing leaf locks in the ordering tree.
impl<L> LockedAt<'_, L> {
    /// Provides access to a [MutexLock]'s state.
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

    /// Provides read access to a [RwLock]'s state.
    pub fn read_lock<'a, NewLock: LockAfter<L> + LockLevel<Method = ReadWrite> + 'a, T: RwLock>(
        &'a mut self,
        t: &'a T,
    ) -> Result<T::ReadGuard<'a>, T::ReadError<'a>> {
        self.with_read_lock::<NewLock, T>(t)
            .map(|(_locked, guard)| guard)
    }

    /// Provides read/write access to a [RwLock]'s state.
    pub fn write_lock<'a, NewLock: LockAfter<L> + LockLevel<Method = ReadWrite> + 'a, T: RwLock>(
        &'a mut self,
        t: &'a T,
    ) -> Result<T::WriteGuard<'a>, T::WriteError<'a>> {
        self.with_write_lock::<NewLock, T>(t)
            .map(|(_locked, guard)| guard)
    }
}
