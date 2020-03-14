/// Provides an iterator reduction method similar to Iterator::fold,
/// but the accumulator is initialized by taking the first element of the iterator
pub trait Reduce: Iterator {
  /// Standard iterator reduction where the accumulator
  /// is initialized with the first value of the iterator
  /// before the callback is called
  fn reduce<F> (self, f: F) -> Self::Item
  where F: Fn(Self::Item, Self::Item) -> Self::Item;
}

impl<T> Reduce for T
where T: Iterator
{
  #[inline]
  fn reduce<F> (mut self, f: F) -> Self::Item
  where F: Fn(Self::Item, Self::Item) -> Self::Item
  {
    let mut acc = self.next().unwrap();

    for e in self {
      acc = f(acc, e);
    }

    acc
  }
}