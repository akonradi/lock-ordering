#![allow(stable_features)]
#![cfg_attr(not(feature = "std"), no_std)]

//! # Lock ordering enforcement at compile time
//!
//! This library contains types and traits to ensure that locks that are held at
//! the same time are acquired in the correct order. This lets code authors
//! verify, at compile time, that their code is free of deadlock opportunities.
//!
//! The way this works is by using traits in the [relation] crate to define
//! orderings between marker types that represent different lock-levels. The core
//! logic lives in the [`LockedAt`] type; it uses trait bounds to ensure that
//! any acquisition of locks respects these orderings.

pub mod lock;
mod lockedat;
pub mod relation;

pub use lockedat::{LockedAt, MutualExclusion, ReadWrite};

/// The least-restrictive lock level, when no locks are held.
pub struct Unlocked;

/// Marker for a type that indicates a level in the locking hierarchy.
pub trait LockLevel {
    type Method;
}

#[cfg(test)]
mod tests {
    #[test]
    fn compile_fail() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/fail/*.rs");
    }
}
