use core::marker::PhantomData;

use crate::relation::LockBefore;
use crate::{
    lock::{MutexLock, MutexLockLevel, RwLock, RwLockLevel},
    Unlocked,
};

#[cfg(feature = "async")]
use crate::lock::{AsyncMutexLock, AsyncMutexLockLevel, AsyncRwLock, AsyncRwLockLevel};

#[cfg(doc)]
use crate::LockLevel;

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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<L> LockedAt<'_, L> {
    /// Attempts to acquire a lock on `NewLock` state.
    ///
    /// Assuming `NewLock` is a lock level that can be acquired after `L`, this
    /// method provides access to state held in the [`MutexLock`] type
    /// `NewLock::Mutex`. If the lock acquisition fails, an error will be
    /// returned. Otherwise, this method returns a new `LockedAt` along with an
    /// accessor for the held state.
    ///
    /// If no further `LockedAt` calls need to be made after this one, consider
    /// using [`LockedAt::lock`] instead.
    #[allow(clippy::type_complexity)]
    pub fn with_lock<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::Mutex,
    ) -> Result<
        (
            LockedAt<'a, NewLock>,
            <NewLock::Mutex as MutexLock>::Guard<'a>,
        ),
        <NewLock::Mutex as MutexLock>::Error<'a>,
    >
    where
        NewLock: MutexLockLevel,
        L: LockBefore<NewLock>,
    {
        t.lock().map(|guard| (LockedAt(PhantomData), guard))
    }

    /// Attempts to acquire a shared lock on `NewLock` state.
    ///
    /// Assuming `NewLock` is a lock level that can be acquired after `L`, this
    /// method provides access to state held in the [`ReadWrite`] type
    /// `NewLock::RwLock`. If the lock acquisition fails, an error will be
    /// returned. Otherwise, this method returns a new `LockedAt` along with a
    /// read-only accessor for the held state.
    ///
    /// If no further `LockedAt` calls need to be made after this one, consider
    /// using [`LockedAt::read_lock`] instead.
    #[allow(clippy::type_complexity)]
    pub fn with_read_lock<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::RwLock,
    ) -> Result<
        (
            LockedAt<'a, NewLock>,
            <NewLock::RwLock as RwLock>::ReadGuard<'a>,
        ),
        <NewLock::RwLock as RwLock>::ReadError<'a>,
    >
    where
        NewLock: RwLockLevel,
        L: LockBefore<NewLock>,
    {
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
    #[allow(clippy::type_complexity)]
    pub fn with_write_lock<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::RwLock,
    ) -> Result<
        (
            LockedAt<'a, NewLock>,
            <NewLock::RwLock as RwLock>::WriteGuard<'a>,
        ),
        <NewLock::RwLock as RwLock>::WriteError<'a>,
    >
    where
        NewLock: RwLockLevel,
        L: LockBefore<NewLock>,
    {
        t.write().map(|guard| (LockedAt(PhantomData), guard))
    }
}

// Convenience methods for accessing leaf locks in the ordering tree.
impl<L> LockedAt<'_, L> {
    /// Provides access to a [MutexLock]'s state.
    pub fn lock<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::Mutex,
    ) -> Result<<NewLock::Mutex as MutexLock>::Guard<'a>, <NewLock::Mutex as MutexLock>::Error<'a>>
    where
        NewLock: 'a + MutexLockLevel,
        L: LockBefore<NewLock>,
    {
        self.with_lock::<NewLock>(t).map(|(_locked, guard)| guard)
    }

    /// Provides read access to a [RwLock]'s state.
    pub fn read_lock<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::RwLock,
    ) -> Result<
        <NewLock::RwLock as RwLock>::ReadGuard<'a>,
        <NewLock::RwLock as RwLock>::ReadError<'a>,
    >
    where
        NewLock: RwLockLevel + 'a,
        L: LockBefore<NewLock>,
    {
        self.with_read_lock::<NewLock>(t)
            .map(|(_locked, guard)| guard)
    }

    /// Provides read/write access to a [RwLock]'s state.
    pub fn write_lock<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::RwLock,
    ) -> Result<
        <NewLock::RwLock as RwLock>::WriteGuard<'a>,
        <NewLock::RwLock as RwLock>::WriteError<'a>,
    >
    where
        NewLock: RwLockLevel + 'a,
        L: LockBefore<NewLock>,
    {
        self.with_write_lock::<NewLock>(t)
            .map(|(_locked, guard)| guard)
    }
}

#[cfg(feature = "async")]
impl<L> LockedAt<'_, L> {
    /// Asynchronously acquires a lock on `NewLock` state.
    ///
    /// Assuming `NewLock` is a lock level that can be acquired after `L`, this
    /// method provides access to state held in the [`AsyncMutexLock`] type
    /// `NewLock::Mutex`, yielding the current task until the lock can be
    /// acquired. Once the state is locked, returns a guard for accessing it
    /// and a new `LockedAt` instance that can be used to acquire additional
    /// locks.
    ///
    /// If no further `LockedAt` calls need to be made after this one, consider
    /// using [`LockedAt::wait_lock`] instead.
    pub async fn wait_for_lock<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::Mutex,
    ) -> (
        LockedAt<'a, NewLock>,
        <NewLock::Mutex as AsyncMutexLock>::Guard<'a>,
    )
    where
        NewLock: AsyncMutexLockLevel + 'a,
        L: LockBefore<NewLock>,
    {
        let guard = t.lock().await;
        (LockedAt(PhantomData), guard)
    }

    /// Asynchronously acquires a shared lock on `NewLock` state.
    ///
    /// Assuming `NewLock` is a lock level that can be acquired after `L`, this
    /// method provides access to state held in the [`ReadWrite`] type T.
    /// This method will yield the current task until the lock can be acquired.
    /// Once the state is locked, this method returns a guard for accessing it
    /// and a new `LockedAt` instance that can be used to acquire additional
    /// locks.
    ///
    /// If no further `LockedAt` calls need to be made after this one, consider
    /// using [`LockedAt::wait_read`] instead.
    pub async fn wait_for_read<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::RwLock,
    ) -> (
        LockedAt<'a, NewLock>,
        <NewLock::RwLock as AsyncRwLock>::ReadGuard<'a>,
    )
    where
        NewLock: AsyncRwLockLevel + 'a,
        L: LockBefore<NewLock>,
    {
        let guard = t.read().await;
        (LockedAt(PhantomData), guard)
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
    pub async fn wait_for_write<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::RwLock,
    ) -> (
        LockedAt<'a, NewLock>,
        <NewLock::RwLock as AsyncRwLock>::WriteGuard<'a>,
    )
    where
        NewLock: AsyncRwLockLevel + 'a,
        L: LockBefore<NewLock>,
    {
        let guard = t.write().await;
        (LockedAt(PhantomData), guard)
    }
}

// Convenience methods for accessing leaf locks in the ordering tree.
#[cfg(feature = "async")]
impl<L> LockedAt<'_, L> {
    /// Asynchronously provides access to an [AsyncMutexLock]'s state.
    pub async fn wait_lock<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::Mutex,
    ) -> <NewLock::Mutex as AsyncMutexLock>::Guard<'a>
    where
        NewLock: 'a + AsyncMutexLockLevel,
        L: LockBefore<NewLock>,
    {
        let (_locked, guard) = self.wait_for_lock::<NewLock>(t).await;
        guard
    }

    /// Asynchronously provides read access to an [AsyncRwLock]'s state.
    pub async fn wait_read<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::RwLock,
    ) -> <NewLock::RwLock as AsyncRwLock>::ReadGuard<'a>
    where
        NewLock: AsyncRwLockLevel + 'a,
        L: LockBefore<NewLock>,
    {
        let (_locked, guard) = self.wait_for_read::<NewLock>(t).await;
        guard
    }

    /// Asynchronously provides read/write access to an [AsyncRwLock]'s state.
    pub async fn wait_write<'a, NewLock>(
        &'a mut self,
        t: &'a NewLock::RwLock,
    ) -> <NewLock::RwLock as AsyncRwLock>::WriteGuard<'a>
    where
        NewLock: AsyncRwLockLevel + 'a,
        L: LockBefore<NewLock>,
    {
        let (_locked, guard) = self.wait_for_write::<NewLock>(t).await;
        guard
    }
}
