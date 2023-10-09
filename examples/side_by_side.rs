use std::sync::Mutex;

use lock_ordering::{relation::LockAfter, LockLevel, LockedAt, MutualExclusion, Unlocked};

#[derive(Default)]
struct HoldsLocks {
    a: Mutex<usize>,
    b: Mutex<bool>,
}

/// Marker type for [`HoldsLocks::a`].
struct LockA;
/// Marker type for [`HoldsLocks::b`].
struct LockB;

impl LockAfter<Unlocked> for LockA {}
impl LockLevel for LockA {
    type Method = MutualExclusion;
}

impl LockAfter<LockA> for LockB {}
impl LockLevel for LockB {
    type Method = MutualExclusion;
}

const MAX_THREADS: usize = 16;

fn main() {
    let holds = HoldsLocks::default();

    std::thread::scope(|scope| {
        for _ in 0..MAX_THREADS {
            scope.spawn(|| {
                let mut locked = LockedAt::new();

                let (mut locked, mut a_guard) = locked
                    .with_lock::<LockA, _>(&holds.a)
                    .expect("not poisoned");
                let mut b_guard = locked.lock::<LockB, _>(&holds.b).expect("not poisoned");

                *a_guard += *b_guard as usize;
                *b_guard = !*b_guard;
            });
        }
    });

    assert_eq!(
        *LockedAt::new()
            .lock::<LockA, _>(&holds.a)
            .expect("wasn't poisoned"),
        MAX_THREADS / 2
    );
}
