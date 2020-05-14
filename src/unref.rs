/// Convenience trait to extract a copy of a value referenced in an Option or Result
pub trait Unref {
  /// The inner value type of the container yielded by unref
  type Target;

  /// Extract a copy of a value referenced in an Option or Result 
  fn unref (self) -> Self::Target;
}

impl<T> Unref for Option<&T>
where T: Copy
{
  type Target = Option<T>;

  fn unref (self) -> Self::Target {
    match self {
      Some(inner) => Some(*inner),
      None => None
    }
  }
}

impl<T> Unref for Option<&mut T>
where T: Copy
{
  type Target = Option<T>;

  fn unref (self) -> Self::Target {
    match self {
      Some(inner) => Some(*inner),
      None => None
    }
  }
}

impl<R, E> Unref for Result<&R, E>
where R: Copy
{
  type Target = Result<R, E>;

  fn unref (self) -> Self::Target {
    match self {
      Ok(res) => Ok(*res),
      Err(err) => Err(err)
    }
  }
}

impl<R, E> Unref for Result<&mut R, E>
where R: Copy
{
  type Target = Result<R, E>;

  fn unref (self) -> Self::Target {
    match self {
      Ok(res) => Ok(*res),
      Err(err) => Err(err)
    }
  }
}