//! Tests of the public API

use std::ops::DerefMut;

use lock_ordering::lock::MutexLockLevel;
use lock_ordering::relation::{LockAfter, LockBefore};
use lock_ordering::{LockLevel, LockedAt, MutualExclusion, Unlocked};

enum Inner {}

impl LockLevel for Inner {
    type Method = MutualExclusion;
}
impl MutexLockLevel for Inner {
    type Mutex = std::sync::Mutex<()>;
}

impl LockAfter<Unlocked> for Inner {}

/// Make sure we can write `L: LockedBefore<OurLock>` as a bound on methods.
#[test]
#[allow(unused)]
fn lock_before_as_bound() {
    #[derive(Default)]
    struct State {
        inner: std::sync::Mutex<()>,
    };

    impl State {
        fn lock<'s, L: LockBefore<Inner>>(
            &'s self,
            locked: &'s mut LockedAt<'_, L>,
        ) -> (LockedAt<'s, Inner>, impl DerefMut<Target = ()> + 's) {
            let (locked, inner) = locked.with_lock(&self.inner).unwrap();
            (locked, inner)
        }
    }
}
