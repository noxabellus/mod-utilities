//! WrappedArray and support structures

use std::{
  mem::MaybeUninit,
  ops::{
    Index,
    IndexMut,
    Deref,
    DerefMut,
  },
  iter::FromIterator,
  ptr::write,
};

/// A const generic wrapper type for doing more by-value operations on Arrays
pub struct WrappedArray<T, const N: usize>(pub [T; N]);

impl<T, const N: usize> WrappedArray<T, N> {
  /// Get a zero-initialized array
  /// # Safety
  /// This is only safe if your array elements are valid when their bytes are all zero
  #[inline] pub unsafe fn zeroed () -> Self { Self(MaybeUninit::zeroed().assume_init()) }
  
  /// Get a pointer to the first element of a WrappedArray
  #[inline] pub fn as_ptr (&self) -> *const T { self.0.as_ptr() }
  
  /// Get a mutable pointer to the first element of a WrappedArray
  #[inline] pub fn as_mut_ptr (&mut self) -> *mut T { self.0.as_mut_ptr() }
}


impl<T, const N: usize> AsRef<[T]> for WrappedArray<T, N> {
  #[inline] fn as_ref (&self) -> &[T] { self.0.as_ref() }
}

impl<T, const N: usize> AsMut<[T]> for WrappedArray<T, N> {
  #[inline] fn as_mut (&mut self) -> &mut [T] { self.0.as_mut() }
}

impl<T, const N: usize> Deref for WrappedArray<T, N> {
  type Target = [T; N];
  #[inline] fn deref (&self) -> &Self::Target { &self.0 }
}

impl<T, const N: usize> DerefMut for WrappedArray<T, N> {
  #[inline] fn deref_mut (&mut self) -> &mut Self::Target { &mut self.0 }
}


impl<T, const N: usize> From<[T; N]> for WrappedArray<T, N> {
  #[inline] fn from (arr: [T; N]) -> Self { Self(arr) }
}

impl<T, const N: usize> From<WrappedArray<T, N>> for [T; N] {
  #[inline] fn from (arr: WrappedArray<T, N>) -> Self { arr.0 }
}

impl<T, const N: usize> Index<usize> for WrappedArray<T, N> {
  type Output = T;
  #[inline] fn index (&self, idx: usize) -> &Self::Output { &self.0[idx] }
}

impl<T, const N: usize> IndexMut<usize> for WrappedArray<T, N> {
  #[inline] fn index_mut (&mut self, idx: usize) -> &mut Self::Output { &mut self.0[idx] }
}

/// A by-value consuming iterator for a WrappedArray
/// 
/// This is only valid where T: Copy,
/// because the values must be copied out of the WrappedArray
pub struct IntoIter<T: Copy, const N: usize> {
  arr: [T; N],
  idx: usize,
}

impl<T: Copy, const N: usize> Iterator for IntoIter<T, N> {
  type Item = T;

  #[inline]
  fn next (&mut self) -> Option<Self::Item> {
    if self.idx < N {
      let el = self.arr[self.idx];

      self.idx += 1;

      Some(el)
    } else {
      None
    }
  }
}

impl<T: Copy, const N: usize> IntoIterator for WrappedArray<T, N> {
  type Item = T;
  type IntoIter = IntoIter<T, N>;

  #[inline] fn into_iter (self) -> Self::IntoIter { IntoIter { arr: self.into(), idx: 0 } }
}

impl<T, const N: usize> FromIterator<T> for WrappedArray<T, N> {
  #[inline]
  fn from_iter<I: IntoIterator<Item = T>> (iter: I) -> Self {
    let mut res = MaybeUninit::uninit();
    let ptr = res.as_mut_ptr() as *mut T;
    
    let mut i = 0usize;

    for e in iter {
      assert!(i < N, "WrappedArray FromIterator overflow");
      unsafe { write(ptr.add(i), e) };
      i += 1;
    }

    assert!(i == N, "WrappedArray FromIterator underflow");

    unsafe { res.assume_init() }
  }
}
