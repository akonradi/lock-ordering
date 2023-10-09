pub trait LockBefore<Other> {}

pub trait LockAfter<Other> {}

impl<Before, After> LockBefore<After> for Before where After: LockAfter<Before> {}
