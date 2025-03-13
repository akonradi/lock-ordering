#[cfg(feature = "async")]
use lock_ordering::lock::{AsyncMutexLock, AsyncMutexLockLevel, AsyncRwLock, AsyncRwLockLevel};
use lock_ordering::lock::{MutexLock, MutexLockLevel, RwLock, RwLockLevel};
use lock_ordering::relation::LockBefore;
use lock_ordering::{LockedAt, Unlocked};

#[cfg(feature = "async")]
use crate::state::{AsyncMutexLockedState, AsyncRwLockedState};
use crate::state::{MutexLockedState, RwLockedState};

pub mod state;

/// Allows safe access to locked values held in some state.
///
/// This type wraps a value of `&T` and provides access to locked state held in
/// `T` while enforcing correct lock ordering.
pub struct Locked<'l, T, L> {
    state: &'l T,
    locked: LockedAt<'l, L>,
}

impl<'l, T> Locked<'l, T, Unlocked> {
    /// Creates a new `Locked` assuming no locks are held.
    pub fn new(state: &'l T) -> Self {
        Self {
            state,
            locked: LockedAt::new(),
        }
    }
}

impl<'l, T, L> Locked<'l, T, L> {
    /// Scopes the current lock level lower in the ordering tree.
    ///
    /// Acts as if the current lock level is `NewLock` without actually
    /// acquiring any locks. This is notionally equivalent to calling
    /// [`Locked::with_lock`] and then `drop`ping the returned guard, but without
    /// performing any locking operations.
    pub fn skip_locking<'a, NewLock>(&'a mut self) -> Locked<'a, T, NewLock>
    where
        L: LockBefore<NewLock>,
    {
        let Self { state, locked } = self;
        Locked {
            state,
            locked: locked.skip_locking(),
        }
    }
}

impl<'l, T, L> Locked<'l, T, L> {
    /// Moves to a new lock level without actually locking

    /// Attempts to acquire a lock on `NewLock` state in `T`.
    ///
    /// Acquires access to the state indicated by the marker type `NewLock`. If
    /// the lock acquisition fails, an error will be returned. Otherwise, this
    /// method returns a new `Locked` along with an accessor for the held state.
    ///
    /// If no further locking calls need to be made after this one, consider
    /// using [`Locked::lock`] instead.
    pub fn with_lock<'a, NewLock>(
        &'a mut self,
    ) -> Result<
        (
            Locked<'a, T, NewLock::LockLevel>,
            <<NewLock::LockLevel as MutexLockLevel>::Mutex as MutexLock>::Guard<'a>,
        ),
        <<NewLock::LockLevel as MutexLockLevel>::Mutex as MutexLock>::Error<'a>,
    >
    where
        NewLock: MutexLockedState<T>,
        NewLock::LockLevel: MutexLockLevel,
        L: LockBefore<NewLock::LockLevel>,
    {
        let Self { state, locked } = self;
        let mutex = NewLock::mutex(state);
        locked.with_lock(mutex).map(|(locked, guard)| {
            (
                Locked {
                    state: *state,
                    locked,
                },
                guard,
            )
        })
    }

    /// Attempts to acquire a shared lock on `NewLock` state in `T`.
    ///
    /// Provides access to state indicated by the marker type `NewLock`.  If the
    /// lock acquisition fails, an error will be returned. Otherwise, this
    /// method returns a new `Locked` along with a read-only accessor for the
    /// held state.
    ///
    /// If no further locking calls need to be made after this one, consider
    /// using [`Locked::read_lock`] instead.
    #[allow(clippy::type_complexity)]
    pub fn with_read_lock<'a, NewLock>(
        &'a mut self,
    ) -> Result<
        (
            Locked<'a, T, NewLock::LockLevel>,
            <<NewLock::LockLevel as RwLockLevel>::RwLock as RwLock>::ReadGuard<'a>,
        ),
        <<NewLock::LockLevel as RwLockLevel>::RwLock as RwLock>::ReadError<'a>,
    >
    where
        NewLock: RwLockedState<T>,
        NewLock::LockLevel: RwLockLevel,
        L: LockBefore<NewLock::LockLevel>,
    {
        let Self { state, locked } = self;
        let rw_lock = NewLock::rw_lock(state);
        locked.with_read_lock(rw_lock).map(|(locked, guard)| {
            (
                Locked {
                    locked,
                    state: *state,
                },
                guard,
            )
        })
    }

    /// Attempts to acquire an exclusive lock on `NewLock` state in `T`.
    ///
    /// Provides access to state indicated by the marker type `NewLock`. If the
    /// lock acquisition fails, an error will be returned. Otherwise, this
    /// method returns a new `Locked` along with a read/write accessor for the
    /// held state.
    ///
    /// If no further `Locked` calls need to be made after this one, consider
    /// using [`Locked::write_lock`] instead.
    #[allow(clippy::type_complexity)]
    pub fn with_write_lock<'a, NewLock>(
        &'a mut self,
    ) -> Result<
        (
            Locked<'a, T, NewLock::LockLevel>,
            <<NewLock::LockLevel as RwLockLevel>::RwLock as RwLock>::WriteGuard<'a>,
        ),
        <<NewLock::LockLevel as RwLockLevel>::RwLock as RwLock>::WriteError<'a>,
    >
    where
        NewLock: RwLockedState<T>,
        NewLock::LockLevel: RwLockLevel,
        L: LockBefore<NewLock::LockLevel>,
    {
        let Self { state, locked } = self;
        let rw_lock = NewLock::rw_lock(state);
        locked.with_write_lock(rw_lock).map(|(locked, guard)| {
            (
                Locked {
                    locked,
                    state: *state,
                },
                guard,
            )
        })
    }
}

/// Convenience wrappers.
impl<'l, T, L> Locked<'l, T, L> {
    /// Provides access to state in `T` indicated by `NewLock`.
    ///
    /// Convenience wrapper for [`Locked::with_lock`] for when no further locks
    /// need to be acquired after `NewLock`.
    pub fn lock<'a, NewLock>(
        &'a mut self,
    ) -> Result<
        <<NewLock::LockLevel as MutexLockLevel>::Mutex as MutexLock>::Guard<'a>,
        <<NewLock::LockLevel as MutexLockLevel>::Mutex as MutexLock>::Error<'a>,
    >
    where
        NewLock: MutexLockedState<T>,
        NewLock::LockLevel: MutexLockLevel + 'a,
        L: LockBefore<NewLock::LockLevel>,
    {
        self.with_lock::<NewLock>().map(|(_locked, guard)| guard)
    }

    /// Provides read-only access to state in `T` indicated by `NewLock`.
    ///
    /// Convenience wrapper for [`Locked::with_read_lock`] for when no further locks
    /// need to be acquired after `NewLock`.
    pub fn read_lock<'a, NewLock>(
        &'a mut self,
    ) -> Result<
        <<NewLock::LockLevel as RwLockLevel>::RwLock as RwLock>::ReadGuard<'a>,
        <<NewLock::LockLevel as RwLockLevel>::RwLock as RwLock>::ReadError<'a>,
    >
    where
        NewLock: RwLockedState<T>,
        NewLock::LockLevel: RwLockLevel + 'a,
        L: LockBefore<NewLock::LockLevel>,
    {
        self.with_read_lock::<NewLock>()
            .map(|(_locked, guard)| guard)
    }

    /// Provides write access to state in `T` indicated by `NewLock`.
    ///
    /// Convenience wrapper for [`Locked::with_read_lock`] for when no further locks
    /// need to be acquired after `NewLock`.
    pub fn write_lock<'a, NewLock>(
        &'a mut self,
    ) -> Result<
        <<NewLock::LockLevel as RwLockLevel>::RwLock as RwLock>::WriteGuard<'a>,
        <<NewLock::LockLevel as RwLockLevel>::RwLock as RwLock>::WriteError<'a>,
    >
    where
        NewLock: RwLockedState<T>,
        NewLock::LockLevel: RwLockLevel + 'a,
        L: LockBefore<NewLock::LockLevel>,
    {
        self.with_write_lock::<NewLock>()
            .map(|(_locked, guard)| guard)
    }
}

#[cfg(feature = "async")]
impl<'l, T, L> Locked<'l, T, L> {
    /// Asynchronously acquires a lock on `NewLock` state in `T`.
    ///
    /// Provides access to state held in `T` indicated by the marker type
    /// `NewLock`, yielding the current task until the lock can be acquired.
    /// Once the state is locked, returns a guard for accessing it and a new
    /// `Locked` instance that can be used to acquire additional locks.
    ///
    /// If no further `Locked` calls need to be made after this one, consider
    /// using [`Locked::wait_lock`] instead.
    pub async fn wait_for_lock<'a, NewLock>(
        &'a mut self,
    ) -> (
        Locked<'a, T, NewLock::LockLevel>,
        <<NewLock::LockLevel as AsyncMutexLockLevel>::Mutex as AsyncMutexLock>::Guard<'a>,
    )
    where
        NewLock: AsyncMutexLockedState<T>,
        NewLock::LockLevel: AsyncMutexLockLevel,
        L: LockBefore<NewLock::LockLevel>,
    {
        let Self { locked, state } = self;
        let mutex = NewLock::mutex(state);
        let (locked, guard) = locked.wait_for_lock(mutex).await;
        (
            Locked {
                locked,
                state: *state,
            },
            guard,
        )
    }

    /// Asynchronously acquires a shared lock on `NewLock` state in `T`.
    ///
    /// Provides access to state held in `T` indicated by marker type `T`. This
    /// method will yield the current task until the lock can be acquired.  Once
    /// the state is locked, this method returns a guard for accessing it and a
    /// new `Locked` instance that can be used to acquire additional locks.
    ///
    /// If no further `Locked` calls need to be made after this one, consider
    /// using [`Locked::wait_read`] instead.
    pub async fn wait_for_read<'a, NewLock>(
        &'a mut self,
    ) -> (
        Locked<'a, T, NewLock::LockLevel>,
        <<NewLock::LockLevel as AsyncRwLockLevel>::RwLock as AsyncRwLock>::ReadGuard<'a>,
    )
    where
        NewLock: AsyncRwLockedState<T>,
        NewLock::LockLevel: AsyncRwLockLevel,
        L: LockBefore<NewLock::LockLevel>,
    {
        let Self { locked, state } = self;
        let mutex = NewLock::rw_lock(state);
        let (locked, guard) = locked.wait_for_read(mutex).await;
        (
            Locked {
                locked,
                state: *state,
            },
            guard,
        )
    }

    /// Attempts to acquire an exclusive lock on `NewLock` state in `T`.
    ///
    /// Provides access to state held in `T` indicated by the marker type
    /// `NewLock``. If the lock acquisition fails, an error will be returned.
    /// Otherwise, this method returns a new `Locked` along with a read/write
    /// accessor for the held state.
    ///
    /// If no further `Locked` calls need to be made after this one, consider
    /// using [`Locked::write_lock`] instead.
    pub async fn wait_for_write<'a, NewLock>(
        &'a mut self,
    ) -> (
        Locked<'a, T, NewLock::LockLevel>,
        <<NewLock::LockLevel as AsyncRwLockLevel>::RwLock as AsyncRwLock>::WriteGuard<'a>,
    )
    where
        NewLock: AsyncRwLockedState<T>,
        NewLock::LockLevel: AsyncRwLockLevel,
        L: LockBefore<NewLock::LockLevel>,
    {
        let Self { locked, state } = self;
        let mutex = NewLock::rw_lock(state);
        let (locked, guard) = locked.wait_for_write(mutex).await;
        (
            Locked {
                locked,
                state: *state,
            },
            guard,
        )
    }
}

// Convenience methods for accessing leaf locks in the ordering tree.
#[cfg(feature = "async")]
impl<'l, T, L> Locked<'l, T, L> {
    /// Asynchronously provides access to an [AsyncMutexLock]'s state.
    pub async fn wait_lock<'a, NewLock>(
        &'a mut self,
    ) -> <<NewLock::LockLevel as AsyncMutexLockLevel>::Mutex as AsyncMutexLock>::Guard<'a>
    where
        NewLock: AsyncMutexLockedState<T>,
        NewLock::LockLevel: AsyncMutexLockLevel + 'a,
        L: LockBefore<NewLock::LockLevel>,
    {
        let (_locked, guard) = self.wait_for_lock::<NewLock>().await;
        guard
    }

    /// Asynchronously provides read access to an [AsyncRwLock]'s state.
    pub async fn wait_read<'a, NewLock>(
        &'a mut self,
    ) -> <<NewLock::LockLevel as AsyncRwLockLevel>::RwLock as AsyncRwLock>::ReadGuard<'a>
    where
        NewLock: AsyncRwLockedState<T>,
        NewLock::LockLevel: AsyncRwLockLevel + 'a,
        L: LockBefore<NewLock::LockLevel>,
    {
        let (_locked, guard) = self.wait_for_read::<NewLock>().await;
        guard
    }

    /// Asynchronously provides read/write access to an [AsyncRwLock]'s state.
    pub async fn wait_write<'a, NewLock>(
        &'a mut self,
    ) -> <<NewLock::LockLevel as AsyncRwLockLevel>::RwLock as AsyncRwLock>::WriteGuard<'a>
    where
        NewLock: AsyncRwLockedState<T>,
        NewLock::LockLevel: AsyncRwLockLevel + 'a,
        L: LockBefore<NewLock::LockLevel>,
    {
        let (_locked, guard) = self.wait_for_write::<NewLock>().await;
        guard
    }
}
