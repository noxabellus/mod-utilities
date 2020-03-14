/// Discards all syntax passed to it
#[macro_export]
macro_rules! discard {
  ($($any: tt)*) => { };
}

/// Discards the first comma separated syntax passed to it
#[macro_export]
macro_rules! discard_first {
  ($first: tt, $($any: tt)*) => { $($any)* };
}