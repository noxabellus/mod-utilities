use core::hint::unreachable_unchecked;

use std::fmt::Debug;

/// Allows unsafely unwrapping a container value without checking if it is valid
pub trait UnwrapUnchecked {
  /// The type of the inner value of the container yielded by unwrap_unchecked
  type Result;

  /// Unwrap a container value and return its result type.
  /// 
  /// # Safety
  /// If debug asserts are not enabled,
  /// no check is performed for the validity of the value,
  /// and it is up to the callee to ensure the result value is available.
  unsafe fn unwrap_unchecked (self) -> Self::Result;
}


impl<T> UnwrapUnchecked for Option<T> {
  type Result = T;

  unsafe fn unwrap_unchecked (self) -> Self::Result {
    if cfg!(debug_assertions) {
      self.unwrap()
    } else if let Some(v) = self {
      v
    } else {
      unreachable_unchecked()
    }
  }
}

impl<R, E: Debug> UnwrapUnchecked for Result<R, E> {
  type Result = R;

  unsafe fn unwrap_unchecked (self) -> Self::Result {
    if cfg!(debug_assertions) {
      self.unwrap()
    } else if let Ok(v) = self {
      v
    } else {
      unreachable_unchecked()
    }
  }
}