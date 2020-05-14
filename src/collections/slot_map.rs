//! SlotMap and support structures

use std::{
  hash::Hash,
  ops::{
    Deref,
    Index,
    IndexMut,
  },
  slice::{
    Iter as SliceIter,
    IterMut as SliceIterMut,
  },
  vec::IntoIter as VecIntoIter,
  marker::PhantomData,
};

use crate::POD;


/// The interior data type contained by SlotMap Keys
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct KeyData {
  idx: u32,
  gen: u32,
}

/// The data type used by SlotMaps to map from Keys to values
#[derive(Debug, Clone)]
pub struct Slot {
  idx: u32,
  gen: u32,
}

/// A classification trait for types which can be used as a Key in a SlotMap
/// 
/// Use the `make_key_type!` macro to generate types which implements this trait
pub trait Key: From<KeyData>
             + Into<KeyData>
             + Deref<Target = KeyData>
             + POD
             + Hash
{ }

impl<T> Key for T
where T: From<KeyData>
       + Into<KeyData> 
       + Deref<Target = KeyData>
       + POD
       + Hash
{ }


/// A wrapper macro to generate data and implementation for a unique Key type for use with SlotMaps
/// 
/// This allows type safety when using multiple SlotMaps,
/// meaning a Key created for one SlotMap will not work for another, being rejected at compile time
#[macro_export]
macro_rules! make_key_type {
  ($(#[$meta:meta])* $vis: vis struct $name: ident ; $($rest: tt)*) => {
    $(#[$meta])*
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    $vis struct $name($crate::collections::slot_map::KeyData);

    impl $name {
      /// An uninitialized null value for a SlotMap key (the equivalent of Key::default(), but const)
      pub const NULL: Self = Self($crate::collections::slot_map::KeyData { idx: 0, gen: 0 });
    }

    impl From<$crate::collections::slot_map::KeyData> for $name {
      fn from (data: $crate::collections::slot_map::KeyData) -> Self {
        Self(data)
      }
    }

    impl From<&$crate::collections::slot_map::KeyData> for $name {
      fn from (data: &$crate::collections::slot_map::KeyData) -> Self {
        Self(*data)
      }
    }

    impl Into<$crate::collections::slot_map::KeyData> for $name {
      fn into (self) -> $crate::collections::slot_map::KeyData {
        self.0
      }
    }

    impl std::ops::Deref for $name {
      type Target = $crate::collections::slot_map::KeyData;

      fn deref (&self) -> &Self::Target {
        &self.0
      }
    }

    $crate::make_key_type!($($rest)*);
  };

  () => {};
}

make_key_type! {
  /// A standard Key for use with SlotMaps
  pub struct DefaultKey;
}




#[derive(Debug, Clone)]
struct FreeList {
  head: u32,
  tail: u32
}


/// A Vec with an always up-to-date indirection layer
/// 
/// SlotMaps allow a single-jump association between an index and a value,
/// that is always valid as long as the value exists
/// 
/// This is implemented via use of generation counting
/// and a secondary vector of mappings which never shrinks
/// 
/// This SlotMap type is implemented with a value-dense vec:
/// Order is not preserved during value removal,
/// but values are always tightly-packed for optimal data locality and iteration speed
#[derive(Debug, Clone)]
pub struct SlotMap<K: Key, V> {
  keys: Vec<K>,
  values: Vec<V>,
  slots: Vec<Slot>,

  freelist: Option<FreeList>,
}

impl<K: Key, V> Default for SlotMap<K, V> {
  #[inline] fn default () -> Self { Self::new() }
}

impl<K: Key, V> SlotMap<K, V> {
  const DEFAULT_CAPACITY: usize = 256;
  

  /// Create a new SlotMap and initialize its Vecs with a given capacity
  #[inline]
  pub fn with_capacity (cap: usize) -> Self {
    Self {
      keys: Vec::with_capacity(cap),
      values: Vec::with_capacity(cap),
      slots: Vec::with_capacity(cap),

      freelist: None,
    }
  }

  /// Create a new SlotMap and initialize its Vecs with SlotMap::DEFAULT_CAPACITY
  #[inline]
  pub fn new () -> Self {
    Self::with_capacity(Self::DEFAULT_CAPACITY)
  }


  /// Determine if a SlotMap (still) has a value associated with a given Key
  #[inline]
  pub fn contains_key (&self, key: K) -> bool {
    if let Some(slot) = self.slots.get(key.idx as usize) {
      slot.gen == key.gen
    } else {
      false
    }
  }

  
  /// Find the Key associated with a given value in a SlotMap,
  /// if it exists (and implements PartialEq)
  pub fn find_key (&self, value: &V) -> Option<K>
  where V: PartialEq
  {
    for (idx, own_value) in self.values.iter().enumerate() {
      if value == own_value {
        return Some(unsafe { *self.keys.get_unchecked(idx as usize) })
      }
    }

    None
  }


  /// Get an immutable slice of the keys of a SlotMap
  #[inline]
  pub fn keys (&self) -> &[K] {
    self.keys.as_slice()
  }

  /// Get a mutable slice of the values of a SlotMap
  #[inline]
  pub fn values (&self) -> &[V] {
    self.values.as_slice()
  }

  /// Get a mutable iterator over the values of a SlotMap
  #[inline]
  pub fn values_mut (&mut self) -> &mut [V] {
    self.values.as_mut_slice()
  }


  /// Get an immutable iterator over the values in a SlotMap
  #[inline]
  pub fn iter (&self) -> SliceIter<V> {
    self.values.iter()
  }

  /// Get a mutable iterator over the values in a SlotMap
  #[inline]
  pub fn iter_mut (&mut self) -> SliceIterMut<V> {
    self.values.iter_mut()
  }


  /// Get an iterator over the Keys in a SlotMap, in value order
  #[inline]
  pub fn key_iter (&self) -> SliceIter<K> {
    self.keys.iter()
  }

  /// Get an immutable iterator over the (Key, value) pairs in a SlotMap, in value order
  #[inline]
  pub fn pair_iter (&self) -> PairIter<K, V> {
    PairIter::new(self)
  }

  /// Get a (value) mutable iterator over the (Key, value) pairs in a SlotMap, in value order
  #[inline]
  pub fn pair_iter_mut (&mut self) -> PairIterMut<K, V> {
    PairIterMut::new(self)
  }


  /// Get an immutable reference to a value associated with a given Key in a SlotMap,
  /// if it (still) exists
  /// 
  /// This bounds checks the slot index in the Key,
  /// and then validates the generation count in the resulting slot
  #[inline]
  pub fn get (&self, key: K) -> Option<&V> {
    let slot = self.slots.get(key.idx as usize)?;

    if slot.gen == key.gen {
      Some(unsafe { self.values.get_unchecked(slot.idx as usize) })
    } else {
      None
    }
  }

  /// Get a mutable reference to a value associated with a given Key in a SlotMap,
  /// if it (still) exists
  /// 
  /// This bounds checks the slot index in the Key,
  /// and then validates the generation count in the resulting slot
  #[inline]
  pub fn get_mut (&mut self, key: K) -> Option<&mut V> {
    let slot = self.slots.get(key.idx as usize)?;

    if slot.gen == key.gen {
      Some(unsafe { self.values.get_unchecked_mut(slot.idx as usize) })
    } else {
      None
    }
  }

  /// Unsafely get an immutable reference to a value associated with a given Key in a SlotMap,
  /// by assuming it still exists
  /// 
  /// # Safety
  /// This does **not** bounds check the slot index in the Key,
  /// and also does **not** validate the generation count in the resulting slot
  #[inline]
  pub unsafe fn get_unchecked (&self, key: K) -> &V {
    self.values.get_unchecked(self.slots.get_unchecked(key.idx as usize).idx as usize)
  }

  /// Unsafely get a mutable reference to a value associated with a given Key in a SlotMap,
  /// by assuming it still exists
  /// 
  /// # Safety
  /// This does **not** bounds check the slot index in the Key,
  /// and also does **not** validate the generation count in the resulting slot
  #[inline]
  pub unsafe fn get_unchecked_mut (&mut self, key: K) -> &mut V {
    self.values.get_unchecked_mut(self.slots.get_unchecked(key.idx as usize).idx as usize)
  }

  
  /// Get the number of values in a SlotMap
  #[inline]
  pub fn len (&self) -> usize {
    self.values.len()
  }

  /// Determine if a SlotMap contains any values
  #[inline]
  pub fn is_empty (&self) -> bool {
    self.values.is_empty()
  }


  fn acquire_slot (&mut self, value_idx: u32) -> KeyData {
    let slot_idx;
    let slot;

    if let Some(freelist) = self.freelist.as_mut() {
      slot_idx = freelist.head;
      slot = unsafe { self.slots.get_unchecked_mut(freelist.head as usize) };
      
      if freelist.tail != slot_idx {
        freelist.head = slot.idx;
      } else {
        self.freelist = None;
      }
    } else {
      slot_idx = self.slots.len() as u32;
      
      self.slots.push(Slot { idx: 0, gen: 0 });

      slot = unsafe { self.slots.get_unchecked_mut(slot_idx as usize) };
    }
    
    slot.idx = value_idx;
    
    KeyData {
      idx: slot_idx,
      gen: slot.gen
    }
  }


  fn free_slot (&mut self, free_idx: u32) {
    let free_slot = unsafe { self.slots.get_unchecked_mut(free_idx as usize) };
    free_slot.gen += 1;

    if let Some(freelist) = self.freelist.as_mut() {
      let old_tail = unsafe { self.slots.get_unchecked_mut(freelist.tail as usize) };

      old_tail.idx = free_idx;
      
      freelist.tail = free_idx;
    } else {
      self.freelist = Some(FreeList {
        head: free_idx,
        tail: free_idx,
      });
    }
  }


  /// Add a value to a SlotMap and get a Key to retrieve it later
  #[inline]
  pub fn insert (&mut self, value: V) -> K {
    let key = self.acquire_slot(self.len() as u32).into();
  
    self.values.push(value);
    self.keys.push(key);

    key
  }

  /// Add a value to a SlotMap,
  /// using a closure that receives the Key
  /// that will be used to retrieve the value later
  /// 
  /// Also returns the Key associated with the value returned by the closure
  #[inline]
  pub fn insert_with_key<F: FnOnce(K) -> V> (&mut self, f: F) -> K {
    let key = self.acquire_slot(self.len() as u32).into();
    let value = f(key);

    self.values.push(value);
    self.keys.push(key);

    key
  }
  

  /// Remove the value associated with a given Key in a SlotMap,
  /// if it (still) exists
  /// 
  /// Returns the value removed, if one was found
  #[inline]
  pub fn remove (&mut self, key: K) -> Option<V> {
    let slot_idx = key.idx;

    if let Some(slot) = self.slots.get(slot_idx as usize) {
      if slot.gen == key.gen {
        let value_idx = slot.idx as usize;

        self.keys.swap_remove(value_idx);
        let value = self.values.swap_remove(value_idx);
        
        if let Some(key) = self.keys.get(value_idx) {
          unsafe { self.slots.get_unchecked_mut(key.idx as usize) }.idx = value_idx as u32;
        }

        self.free_slot(slot_idx);

        return Some(value)
      }
    }

    None
  }
}

impl<K: Key, V> Index<K> for SlotMap<K, V> {
  type Output = V;

  fn index (&self, key: K) -> &Self::Output {
    self.get(key).expect("Attempted SlotMap[] access to invalid key")
  }
}

impl<K: Key, V> IndexMut<K> for SlotMap<K, V> {
  fn index_mut (&mut self, key: K) -> &mut Self::Output {
    self.get_mut(key).expect("Attempted SlotMap[] access to invalid key")
  }
}

impl<K: Key, V> IntoIterator for SlotMap<K, V> {
  type Item = V;
  type IntoIter = VecIntoIter<V>;

  fn into_iter(self) -> Self::IntoIter {
    self.values.into_iter()
  }
}

/// An iterator over (Key, Value) for a SlotMap
pub struct PairIter<'a, K: Key + 'a, V: 'a> {
  len: usize,
  idx: usize,

  keys: *const K,
  values: *const V,

  k_phantom: PhantomData<&'a K>,
  v_phantom: PhantomData<&'a V>,
}

impl<'a, K: Key + 'a, V: 'a> PairIter<'a, K, V> {
  /// Create a new PairIter for a SlotMap
  #[inline]
  pub fn new (map: &'a SlotMap<K, V>) -> PairIter<'a, K, V> {
    Self {
      len: map.len(),
      idx: 0,

      keys: map.keys.as_ptr(),
      values: map.values.as_ptr(),

      k_phantom: PhantomData,
      v_phantom: PhantomData,
    }
  }
}

/// An iterator over (Key, mut Value) for a SlotMap
pub struct PairIterMut<'a, K: Key + 'a, V: 'a> {
  len: usize,
  idx: usize,

  keys: *const K,
  values: *mut V,

  k_phantom: PhantomData<&'a K>,
  v_phantom: PhantomData<&'a mut V>,
}

impl<'a, K: Key, V: 'a> PairIterMut<'a, K, V> {
  /// Create a new PairIterMut for a SlotMap
  #[inline]
  pub fn new (map: &'a mut SlotMap<K, V>) -> PairIterMut<'a, K, V> {
    Self {
      len: map.len(),
      idx: 0,

      keys: map.keys.as_ptr(),
      values: map.values.as_mut_ptr(),

      k_phantom: PhantomData,
      v_phantom: PhantomData,
    }
  }
}

impl<'a, K: Key, V: 'a> Iterator for PairIter<'a, K, V> {
  type Item = (&'a K, &'a V);

  fn next (&mut self) -> Option<Self::Item> {
    if self.idx < self.len {
      let value_idx = self.idx;
      self.idx += 1;

      unsafe {
        let key = &*self.keys.add(value_idx);
        let value = &*self.values.add(value_idx);

        Some((key, value))
      }
    } else {
      None
    }
  }
}

impl<'a, K: Key + 'a, V: 'a> Iterator for PairIterMut<'a, K, V> {
  type Item = (&'a K, &'a mut V);

  fn next (&mut self) -> Option<Self::Item> {
    if self.idx < self.len {
      let value_idx = self.idx;
      self.idx += 1;

      unsafe {
        let key = &*self.keys.add(value_idx);
        let value = &mut *self.values.add(value_idx);

        Some((key, value))
      }
    } else {
      None
    }
  }
}


#[cfg(test)]
mod tests {
  #[test]
  fn check_slot_map () {
    let mut sm: super::SlotMap<super::DefaultKey, usize> = super::SlotMap::new();
  
    let k0 = sm.insert(3200);
    let k1 = sm.insert(6400);

    assert_eq!(*sm.get(k0).expect("Failed to get k0"), 3200);
    assert_eq!(*sm.get(k1).expect("Failed to get k1"), 6400);
    assert_eq!(sm.get(super::KeyData { idx: 3, gen: 0 }.into()), None);

    assert_eq!(sm.remove(k0).expect("Failed to remove k0"), 3200);

    let k2 = sm.insert(12800);
    let k3 = sm.insert(25600);

    assert_eq!(sm.find_key(&12800), Some(k2));

    let expect_keys = vec![k1, k2, k3];

    for (i, (k, v)) in sm.pair_iter_mut().enumerate() {
      assert_eq!(&expect_keys[i], k);

      let ov = *v;
      *v += 1;

      println!("{} | {:?} : {} + 1 = {}", i, k, ov, v);
    }

    let expect = vec![6401usize, 12801usize, 25601usize];

    for (i, (k, v)) in sm.pair_iter().enumerate() {
      assert_eq!(&expect[i], v);

      println!("{} | {:?} : {}", i, k, v);
    }
  }
}
