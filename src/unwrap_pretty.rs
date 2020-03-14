use std::{
  fmt::Display,
  any::type_name,
};

/// Convenience trait to unwrap and expect Results where the Err implements Display
/// and can be printed prettier than with Err formatted with Debug
pub trait UnwrapPretty {
  /// The type of the inner value yielded by unwrap_pretty and expect_pretty
  type Result;

  /// Unwrap a container type and if there is an error, print it with fmt::Display
  fn unwrap_pretty (self) -> Self::Result;

  /// Unwrap a container type and if there is an error, print it and a message using fmt::Display
  fn expect_pretty (self, msg: &str) -> Self::Result;
}

impl<R, E> UnwrapPretty for Result<R, E>
where E: Display
{
  type Result = R;

  fn unwrap_pretty (self) -> Self::Result {
    match self {
      Ok(r) => r,
      Err(e) => panic!("Failed to unwrap {}:\n{}", type_name::<Self>(), e)
    }
  }

  fn expect_pretty (self, msg: &str) -> Self::Result {
    match self {
      Ok(r) => r,
      Err(e) => panic!("{}:\n{}", msg, e)
    }
  }
}