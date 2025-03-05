//! Tests of the public API

use lock_ordering::relation::LockAfter;
use lock_ordering::{impl_transitive_lock_order, Unlocked};

/// Make sure we can write `L: LockedBefore<OurLock>` as a bound on methods.
#[cfg(feature = "std")]
#[test]
fn lock_before_as_bound() {
    use std::ops::DerefMut;

    use lock_ordering::lock::MutexLockLevel;
    use lock_ordering::relation::LockBefore;
    use lock_ordering::{LockLevel, LockedAt, MutualExclusion};

    enum Inner {}

    impl LockLevel for Inner {
        type Method = MutualExclusion;
    }
    impl MutexLockLevel for Inner {
        type Mutex = std::sync::Mutex<()>;
    }

    impl LockAfter<Unlocked> for Inner {}
    #[derive(Default)]
    struct State {
        inner: std::sync::Mutex<()>,
    }

    #[allow(unused)]
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

#[test]
fn transitive_lock_relations() {
    enum First {}
    enum Second {}

    impl LockAfter<Unlocked> for First {}
    impl LockAfter<First> for Second {}
    impl_transitive_lock_order!(First => Second);

    static_assertions::assert_impl_all!(Second: LockAfter<Unlocked>);
}
