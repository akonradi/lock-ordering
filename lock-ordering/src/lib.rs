#![allow(stable_features)]
#![cfg_attr(not(feature = "std"), no_std)]

//! # Lock ordering enforcement at compile time
//!
//! This library contains types and traits to ensure that locks that are held at
//! the same time are acquired in the correct order. This lets code authors
//! verify, at compile time, that their code is free of deadlock opportunities.
//!
//! The way this works is by using traits in the [relation] module to define
//! orderings between marker types that represent different lock-levels. The core
//! logic lives in the [`LockedAt`] type; it uses trait bounds to ensure that
//! any acquisition of locks respects these orderings.
//!
//! # How it works
//!
//! This crate combines two ideas:
//! 1. acquiring and holding locks can be represented as mutable borrowing; and
//! 2. a correct lock ordering can be modeled as a directed graph of pairwise
//!    before/after relationships.
//!
//! ## Representing locking using mutable state
//!
//! Within any call tree, a (non-reentrant) lock cannot be acquired more than
//! once at a time. Doing otherwise would lead to deadlock.
//!
//! To visualize this, look at the following code:
//! ```
//! let mutex = std::sync::Mutex::new(false);
//!
//! {
//!     // in a function somewhere
//!     let guard = mutex.lock().unwrap();
//!     // do something with the value
//!     drop(guard);
//! }
//! ```
//!
//! For every call tree we assign each lock some mutable state and pair every
//! acquisition of a lock with a mutable borrow of that state. What that looks
//! like here:
//! ```
//! # let mutex = std::sync::Mutex::new(false);
//! {
//!     let mut state = ();
//!
//!     let (guard, borrow) = (mutex.lock().unwrap(), &mut state);
//!     // do something
//!     drop((guard, borrow));
//! }
//! ```
//!
//! Now we can take advantage of Rust's enforcement of exclusivity for mutable
//! borrows. This lets us prevent at compile time what would otherwise be a run
//! time deadlock!
//! ```compile_fail
//! # let mutex = std::sync::Mutex::new(false);
//! {
//!   let mut state = ();
//!
//!   let (guard, borrow) = (mutex.lock().unwrap(), &mut state);
//!   {
//!     // This fails to compile because `state` is already mutably borrowed.
//!     let (second_guard, second_borrow) = (mutex.lock().unwrap(), &mut state);
//!     drop((second_guard, second_borrow));
//!   }
//!   drop((guard, borrow));
//! }
//! ```
//!
//! This only works so long as we make sure that the borrows of our mutable
//! marker state last as long as the actual borrows of the locked state. We can
//! enforce that in the type system by tying the two lifetimes together.
//! ```
//! fn lock_with_marker_state<'a, T>(
//!     mutex: &'a std::sync::Mutex<T>,
//!     borrow: &'a mut (),
//! ) -> (std::sync::MutexGuard<'a, T>, &'a mut ()) {
//! # unimplemented!()
//! }
//! ```
//!
//! ## Modeling lock ordering as a directed graph of orderings
//!
//! In order for a program to follow a consistent lock ordering, it must be
//! ensured that for any two locks held at the same time, they are acquired in
//! the same order in every possible tree of function calls. One way to achieve
//! this is to assign every lock an ordered "level". Then we ensure that at each
//! point in the code we only acquire locks with a higher level than the highest
//! currently held.
//!
//! Since we're using Rust, we can use marker types to represent levels and
//! traits to represent the ordering between them. That might look something like this:
//! ```
//! trait LockedBefore<L> {}
//!
//! struct Unlocked;
//! struct FirstLevel;
//! struct SecondLevel;
//!
//! impl LockedBefore<FirstLevel> for Unlocked {}
//! impl LockedBefore<SecondLevel> for Unlocked {}
//! impl LockedBefore<SecondLevel> for FirstLevel {}
//! ```
//!
//! That's not enough on its own, but we can define functions whose trait bounds
//! enforce our lock ordering:
//! ```
//! # trait LockedBefore<L> {}
//! struct HeldLockLevel<L>(std::marker::PhantomData<L>);
//!
//! fn acquire_lock<'a, CurrentLevel, L, T>(
//!     lock: &'a (std::sync::Mutex<T>, std::marker::PhantomData<L>),
//!     current_level: &'a mut HeldLockLevel<CurrentLevel>,
//! ) -> (std::sync::MutexGuard<'a, T>, &'a mut HeldLockLevel<L>)
//! where
//!     CurrentLevel: LockedBefore<L>,
//! {
//!     // ...
//! # unimplemented!()
//! }
//! ```
//!
//! These can prevent us from acquiring locks out of order. With the following
//! shared state:
//! ```
//! # struct FirstLevel;
//! # struct SecondLevel;
//! let first_mutex = (
//!     std::sync::Mutex::new(true),
//!     std::marker::PhantomData::<FirstLevel>,
//! );
//! let second_mutex = (
//!     std::sync::Mutex::new('a'),
//!     std::marker::PhantomData::<SecondLevel>,
//! );
//! ```
//! This works:
//! ```no_run
//! # trait LockedBefore<L> {}
//! # struct Unlocked;
//! # struct FirstLevel;
//! # struct SecondLevel;
//! # impl LockedBefore<FirstLevel> for Unlocked {}
//! # impl LockedBefore<SecondLevel> for Unlocked {}
//! # impl LockedBefore<SecondLevel> for FirstLevel {}
//! # struct HeldLockLevel<L>(std::marker::PhantomData<L>);
//! # fn acquire_lock<'a, CurrentLevel, L, T>(
//! #     lock: &'a (std::sync::Mutex<T>, std::marker::PhantomData<L>),
//! #     current_level: &'a mut HeldLockLevel<CurrentLevel>,
//! # ) -> (std::sync::MutexGuard<'a, T>, &'a mut HeldLockLevel<L>)
//! # where
//! #     CurrentLevel: LockedBefore<L>,
//! # {
//! # unimplemented!()
//! # }
//! # let first_mutex = (
//! #     std::sync::Mutex::new(true),
//! #     std::marker::PhantomData::<FirstLevel>,
//! # );
//! # let second_mutex = (
//! #     std::sync::Mutex::new('a'),
//! #     std::marker::PhantomData::<SecondLevel>,
//! # );
//! let mut current_level = HeldLockLevel::<Unlocked>(std::marker::PhantomData);
//!
//! let (mut first_guard, mut current_level) = acquire_lock(&first_mutex, &mut current_level);
//! let (mut second_guard, mut current_level) = acquire_lock(&second_mutex, &mut current_level);
//! ```
//! Attempting to access locks out of order fails:
//! ```compile_fail
//! # trait LockedBefore<L> {}
//! # struct Unlocked;
//! # struct FirstLevel;
//! # struct SecondLevel;
//! # impl LockedBefore<FirstLevel> for Unlocked {}
//! # impl LockedBefore<SecondLevel> for Unlocked {}
//! # impl LockedBefore<SecondLevel> for FirstLevel {}
//! # struct HeldLockLevel<L>(std::marker::PhantomData<L>);
//! # fn acquire_lock<'a, CurrentLevel, L, T>(
//! #     lock: &'a (std::sync::Mutex<T>, std::marker::PhantomData<L>),
//! #     current_level: &'a mut HeldLockLevel<CurrentLevel>,
//! # ) -> (std::sync::MutexGuard<'a, T>, &'a mut HeldLockLevel<L>)
//! # where
//! #     CurrentLevel: LockedBefore<L>,
//! # {
//! # unimplemented!()
//! # }
//! # let first_mutex = (
//! #     std::sync::Mutex::new(true),
//! #     std::marker::PhantomData::<FirstLevel>,
//! # );
//! # let second_mutex = (
//! #     std::sync::Mutex::new('a'),
//! #     std::marker::PhantomData::<SecondLevel>,
//! # );
//! let mut current_level = HeldLockLevel::<Unlocked>(std::marker::PhantomData);
//!
//! let (mut second_guard, mut current_level) = acquire_lock(&second_mutex, &mut current_level);
//! // This will fail to compile because SecondLevel does not implement LockedBefore<FirstLevel>
//! let (mut first_guard, mut current_level) = acquire_lock(&first_mutex, &mut current_level);
//! ```
//!
//! It's worth noting that this only works so long as we ensure our
//! `LockedBefore` trait implementations are consistent. There's no
//! language-level feature that prevents us from introducing a cycle in our
//! trait implementations and defining an invalid lock ordering!
//!
//! # How to use this crate
//!
//! You'll need to define a marker type for each "level" in your lock ordering
//! hierarchy, then implement [`LockLevel`] for each of them. Then carefully
//! implement [`relation::LockBefore`] to express the order in which pairs of
//! locks can be acquired. The [`relation::impl_transitive_lock_order`] macro
//! can help with this by providing transitive implementations of `LockBefore`.
//!
//! Next, you'll need to specify the acquisition method for each lock by
//! implementing one of [`crate::lock::MutexLock`], [`crate::lock::RwLock`], or
//! their `async` counterparts.
//!
//! Lastly, ensure that every call tree in your code constructs at most one
//! `LockedAt`. "Root" functions that are only called externally can construct
//! one using [`LockedAt::new`], but any non-root function that acquires a lock
//! should take a `&mut LockedAt<'_, L>` as an argument. Make sure you consider
//! the full call graph! A function invoked as a callback, for example, is not a
//! root function, and so should not construct a `LockedAt` unless the callback
//! invocation code doesn't acquire any locks.
//!
//! See the examples for more details.

pub mod lock;
mod lockedat;
pub mod relation;

pub use lockedat::{LockedAt, MutualExclusion, ReadWrite};

/// The least-restrictive lock level, when no locks are held.
pub struct Unlocked;

/// Marker for a type that indicates a level in the locking hierarchy.
pub trait LockLevel {
    /// How the lock can be acquired.
    ///
    /// This should be either [`MutualExclusion`] or [`ReadWrite`].
    type Method;
}
