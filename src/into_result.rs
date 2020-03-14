/// Allows the creation of a Result with a custom Err from a container type
pub trait IntoResult {
  /// The inner value type of the container, which becomes the Ok variant of Result when `into_result` is called
  type Value;

  /// Convert a container to a Result with a custom Err
  fn into_result<E, F> (self, if_none: F) -> Result<Self::Value, E>
  where F: FnOnce() -> E;
}


impl<T> IntoResult for Option<T> {
  type Value = T;

  fn into_result<E, F> (self, if_none: F) -> Result<Self::Value, E>
  where F: FnOnce() -> E
  {
    if let Some(value) = self {
      Ok(value)
    } else {
      Err(if_none())
    }
  }
}