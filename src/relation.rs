//! Traits for describing the relationships between lock orders.

/// Indicates that `Self` is locked before `Other`.
///
/// Indicates that a lock corresponding to the lock level `Other` can be
/// acquired while a lock corresponding to the implementing type `Self` is
/// held.
///
/// This is similar to [`LockAfter`] but with inverted `Self` and `Other` types.
/// Like [`From`] and [`Into`], it allows writing `where` bounds more naturally.
/// This trait is blanket-implemented in terms of `LockAfter`.
pub trait LockBefore<Other> {}

/// Indicates that `Self` is locked after `Other`.
///
/// Indicates that a lock corresponding to the lock level for `Self` can be
/// acquired while a lock corresponding to the other lock level type `Other` is
/// held.
///
/// The trait bound `B: LockAfter<A>` indicates that, while a lock with level
/// `A` is held, a lock with level `B` can be acquired. The trait [`LockBefore`]
/// is blanket-implemented in terms of this trait, so `B: LockAfter<A>` implies
/// `A: LockBefore<B>`.

pub trait LockAfter<Other> {}

impl<Before, After> LockBefore<After> for Before where After: LockAfter<Before> {}
