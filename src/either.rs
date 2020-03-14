/// A wrapper for a value that is either one type or another
/// 
/// Similar to Result, but without the semantic connotations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Either<A, B> {
  A(A),
  B(B),
}

impl<A, B> Either<A,B> {
  /// Determine if an Either is the A variant
  pub fn is_a (&self) -> bool {
    match self {
      Self::A(_) => true,
      _ => false
    }
  }
  
  /// Determine if an Either is the B variant
  pub fn is_b (&self) -> bool {
    match self {
      Self::B(_) => true,
      _ => false
    }
  }


  /// Convert an Either<A, B> to an Option<A>
  pub fn into_a (self) -> Option<A> {
    match self {
      Self::A(a) => Some(a),
      _ => None
    }
  }

  /// Convert an Either<A, B> to an Option<B>
  pub fn into_b (self) -> Option<B> {
    match self {
      Self::B(b) => Some(b),
      _ => None
    }
  }
}

/// Allows converting a value into some side of an Either
pub trait IntoEither: Sized {
  /// Convert a value into an Either::A
  fn into_a<B> (self) -> Either<Self, B> {
    Either::A(self)
  }

  /// Convert a value into an Either::B
  fn into_b<A> (self) -> Either<A, Self> {
    Either::B(self)
  }
}

impl<T> IntoEither for T { }