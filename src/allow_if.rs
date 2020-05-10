/// A trait that allows discarding Option values if some condition is or is not met
pub trait AllowIf: Sized {
  /// The type of value given by AllowIf calls
  type Accepted;

  /// Discard Option value if some condition is not met
  fn allow_if<F: FnOnce(&Self::Accepted) -> bool> (self, f: F) -> Option<Self::Accepted>;
  
  /// Discard Option value if some condition is not met
  fn allow_if_mut<F: FnOnce(&mut Self::Accepted) -> bool> (self, f: F) -> Option<Self::Accepted>;
  
  /// Discard Option value if some condition is met
  #[inline]
  fn allow_if_not<F: FnOnce(&Self::Accepted) -> bool> (self, f: F) -> Option<Self::Accepted> {
    self.allow_if(|x| !f(x))
  }
  
  /// Discard Option value if some condition is met
  #[inline]
  fn allow_if_not_mut<F: FnOnce(&mut Self::Accepted) -> bool> (self, f: F) -> Option<Self::Accepted> {
    self.allow_if_mut(|x| !f(x))
  }
}

impl<T> AllowIf for Option<T> {
  type Accepted = T;

  #[inline]
  fn allow_if<F: FnOnce(&Self::Accepted) -> bool> (self, f: F) -> Option<T> {
    match self {
      Some(x) => if f(&x) { Some(x) } else { None },
      None => None
    }
  }

  #[inline]
  fn allow_if_mut<F: FnOnce(&mut Self::Accepted) -> bool> (self, f: F) -> Option<T> {
    match self {
      Some(mut x) => if f(&mut x) { Some(x) } else { None },
      None => None
    }
  }
}