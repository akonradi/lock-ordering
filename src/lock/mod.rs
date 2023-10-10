//! Traits that describe how locked data is accessed.

pub use mutex::MutexLock;
pub use rwlock::RwLock;

mod mutex;
mod rwlock;
