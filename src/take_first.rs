/// Helper macro that discards any content after the first `expr,` passed to it
#[macro_export]
macro_rules! take_first {
  ($(,)? $a: expr $(,)?) => { $a };
  ($(,)? $a: expr, $($rest: tt)*) => { $a };
}