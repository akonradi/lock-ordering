use lock_ordering::{relation::LockAfter, LockLevel, LockedAt, MutualExclusion, Unlocked};

struct FirstLock;
struct SecondLock;

// Either lock can be acquired without acquiring the other, but if the second
// lock *is* held, the first lock can't be acquired.
impl LockAfter<Unlocked> for FirstLock {}
impl LockAfter<Unlocked> for SecondLock {}
impl LockAfter<FirstLock> for SecondLock {}

impl LockLevel for FirstLock {
    type Method = MutualExclusion;
}
impl LockLevel for SecondLock {
    type Method = MutualExclusion;
}

fn main() {
    let first = std::sync::Mutex::new(1234);
    let second = std::sync::Mutex::new('b');

    let mut locked = LockedAt::new();

    // This is fine: the second lock can be acquired without holding the first.
    let (mut locked, mut second_guard) = locked.with_lock::<SecondLock, _>(&second).unwrap();
    *second_guard = 'c';

    // This is problematic: the first lock can't be acquired while the second is
    // held.
    let mut first_guard = locked.lock::<FirstLock, _>(&first);
}
