use std::sync::{Mutex, RwLock};

use lock_ordering::{
    lock::{MutexLockLevel, RwLockLevel}, relation::LockAfter, LockLevel, LockedAt, MutualExclusion, ReadWrite,
    Unlocked,
};

#[derive(Default)]
struct HoldsSiblingLocks {
    a: Mutex<usize>,
    b: Mutex<bool>,
}

#[derive(Default)]
struct HoldsNestedLocks {
    c: RwLock<Nested>,
}

#[derive(Default)]
struct Nested {
    d: Mutex<u8>,
}

/// Marker type for [`HoldsSiblingLocks::a`].
struct LockA;
/// Marker type for [`HoldsSiblingLocks::b`].
struct LockB;
/// Marker type for [`HoldsNestedLocks::c`].
struct LockC;
/// Marker type for [`Nested::d`].
struct LockD;

impl LockAfter<Unlocked> for LockA {}
impl LockAfter<LockA> for LockB {}
impl LockAfter<LockB> for LockC {}
impl LockAfter<LockC> for LockD {}

impl LockLevel for LockA {
    type Method = MutualExclusion;
}
impl MutexLockLevel for LockA {
    type Mutex = Mutex<usize>;
}

impl LockLevel for LockB {
    type Method = MutualExclusion;
}
impl MutexLockLevel for LockB {
    type Mutex = Mutex<bool>;
}

impl LockLevel for LockC {
    type Method = ReadWrite;
}
impl RwLockLevel for LockC {
    type RwLock = RwLock<Nested>;
}

impl LockLevel for LockD {
    type Method = MutualExclusion;
}
impl MutexLockLevel for LockD {
    type Mutex = Mutex<u8>;
}

#[derive(Default)]
struct State {
    sibling: HoldsSiblingLocks,
    nested: HoldsNestedLocks,
}

const MAX_THREADS: usize = 16;

fn main() {
    let state = State::default();

    std::thread::scope(|scope| {
        for _ in 0..MAX_THREADS {
            scope.spawn(|| {
                let mut locked = LockedAt::new();

                let (mut locked, mut a_guard) =
                    locked.with_lock::<LockA>(&state.sibling.a).unwrap();

                let (mut locked, mut b_guard) =
                    locked.with_lock::<LockB>(&state.sibling.b).unwrap();

                let (mut locked, c_guard) =
                    locked.with_read_lock::<LockC>(&state.nested.c).unwrap();

                let mut d_guard = locked.lock::<LockD>(&c_guard.d).unwrap();

                // Perform some work with the locked state.
                *d_guard = d_guard.wrapping_add(*a_guard as u8);
                *b_guard = !*b_guard;
                *a_guard += 1;
            });
        }
    });

    assert_eq!(
        *LockedAt::new()
            .lock::<LockA>(&state.sibling.a)
            .expect("wasn't poisoned"),
        MAX_THREADS
    );
}
