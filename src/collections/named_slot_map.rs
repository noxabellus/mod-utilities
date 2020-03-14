//! NamedSlotMap and support structures

use std::{
  mem::{
    replace,
    transmute,
  },
  ops::{
    Index,
    IndexMut,
  },
  slice::{
    Iter as SliceIter,
    IterMut as SliceIterMut,
  },
};

use crate::Unref;

use super::{
  bimap::{
    BiMap,
    PairIter as BiMapPairIter,
    PairIterMut as BiMapPairIterMut,
  },
  slot_map::{
    SlotMap,
    Key,
    PairIter as SlotMapPairIter,
    PairIterMut as SlotMapPairIterMut,
  },
};


/// A combination of a SlotMap and a BiMap,
/// supplies (slow-ish) named lookup
/// and (fast) generational indexed key lookup
/// for values of any type
pub struct NamedSlotMap<K: Key, V> {
  slot_map: SlotMap<K, V>,
  id_bindings: BiMap<K, String>
}

impl<K: Key, V> Default for NamedSlotMap<K, V> {
  #[inline] fn default () -> Self { Self::new() }
}

impl<K: Key, V> NamedSlotMap<K, V> {
  const DEFAULT_CAPACITY: usize = 256;


  /// Create a new NamedSlotMap
  /// and initialize its SlotMap and BiMap
  /// with a given capacity
  #[inline]
  pub fn with_capacity (cap: usize) -> Self {
    Self {
      slot_map: SlotMap::with_capacity(cap),
      id_bindings: BiMap::with_capacity(cap),
    }
  }

  /// Create a new NamedSlotMap
  /// and initialize its SlotMap and BiMap
  /// with NamedSlotMap::DEFAULT_CAPACITY
  #[inline]
  pub fn new () -> Self {
    Self::with_capacity(Self::DEFAULT_CAPACITY)
  }

  
  /// Get the number of values in a NamedSlotMap
  #[inline]
  pub fn len (&self) -> usize {
    self.slot_map.len()
  }
  
  /// Determine if a NamedSlotMap contains any values
  #[inline]
  pub fn is_empty (&self) -> bool {
    self.slot_map.is_empty()
  }



  /// Determine if an ID is bound to any value in a NamedSlotMap
  #[inline]
  pub fn contains_id (&self, id: &str) -> bool {
    self.id_bindings.contains_value(id)
  }

  /// Determine if a NamedSlotMap potentially contains a given ID
  /// 
  /// This works by comparing hashes only,
  /// and may yield false positives,
  /// but will never yield a false negative
  #[inline]
  pub fn maybe_contains_id (&self, id: &str) -> bool {
    self.id_bindings.maybe_contains_value(id)
  }

  /// Determine if a NamedSlotMap (still) has a value associated with a given Key
  #[inline]
  pub fn contains_key (&self, key: K) -> bool {
    self.slot_map.contains_key(key)
  }


  /// Find the Key for a value
  /// associated with a given ID in a NamedSlotMap,
  /// if one exists
  #[inline]
  pub fn find_key (&self, id: &str) -> Option<K> {
    self.id_bindings.find_key(id).unref()
  }

  /// Find the Key associated with a given value in a SlotMap,
  /// if it exists (and implements PartialEq)
  #[inline]
  pub fn find_key_by_value (&self, value: &V) -> Option<K>
  where V: PartialEq
  {
    self.slot_map.find_key(value)
  }

  /// Get an immutable reference to the ID for a value
  /// associated with a given Key in a NamedSlotMap,
  /// if one exists
  #[inline]
  pub fn find_id (&self, key: K) -> Option<&String> {
    self.id_bindings.find_value(&key)
  }

  /// Get a mutable reference to the ID for a value
  /// associated with a given Key in a NamedSlotMap,
  /// if one exists
  #[inline]
  pub fn find_id_mut (&mut self, key: K) -> Option<&mut String> {
    self.id_bindings.find_value_mut(&key)
  }
  

  /// Get an immutable reference to a value
  /// associated with a given Key in a NamedSlotMap,
  /// if it (still) exists
  #[inline]
  pub fn get (&self, key: K) -> Option<&V> {
    self.slot_map.get(key)
  }

  /// Get a mutable reference to a value
  /// associated with a given Key in a NamedSlotMap,
  /// if it (still) exists
  #[inline]
  pub fn get_mut (&mut self, key: K) -> Option<&mut V> {
    self.slot_map.get_mut(key)
  }


  /// Unsafely get an immutable reference to a value
  /// associated with a given Key in a NamedSlotMap,
  /// by assuming it still exists
  /// 
  /// # Safety
  /// This does **not** bounds check the slot index in the Key,
  /// and also does **not** validate the generation count in the resulting slot
  #[inline]
  pub unsafe fn get_unchecked (&self, key: K) -> &V {
    self.slot_map.get_unchecked(key)
  }
  
  /// Unsafely get a mutable reference to a value associated with a given Key in a SlotMap,
  /// by assuming it still exists
  /// 
  /// # Safety
  /// This does **not** bounds check the slot index in the Key,
  /// and also does **not** validate the generation count in the resulting slot
  #[inline]
  pub unsafe fn get_unchecked_mut (&mut self, key: K) -> &mut V {
    self.slot_map.get_unchecked_mut(key)
  }


  /// Store a value in a NamedSlotMap with a given ID, and get a Key to retrieve it later
  /// 
  /// If a value is already registered with the given ID, it is replaced and returned
  pub fn insert (&mut self, id: String, value: V) -> (K, Option<V>) {
    if let Some(existing_key) = self.id_bindings.find_key(&id).unref() {
      let existing_value = unsafe { self.slot_map.get_unchecked_mut(existing_key) };

      (existing_key, Some(replace(existing_value, value)))
    } else {
      let new_key = self.slot_map.insert(value);

      self.id_bindings.insert_at_value(id, new_key);

      (new_key, None)
    }
  }

  /// Store a value in a NamedSlotMap with a given ID, and get a Key in a Result::Ok to retrieve it later
  /// 
  /// If a value is already registered with the given ID, does nothing and returns the value in a Result::Err
  pub fn insert_unique (&mut self, id: String, value: V) -> Result<K, V> {
    if self.id_bindings.contains_value(&id) {
      Err(value)
    } else {
      let new_key = self.slot_map.insert(value);

      self.id_bindings.insert_at_value(id, new_key);

      Ok(new_key)
    }
  }

  /// Add a value to a SlotMap,
  /// using a closure that receives the Key
  /// that will be used to retrieve the value later
  /// 
  /// If a value is already registered with the given ID, it is replaced and returned
  /// 
  /// Also returns the Key associated with the value returned by the closure
  pub fn insert_with_key<F: FnOnce(K) -> V> (&mut self, id: String, f: F) -> (K, Option<V>) {
    if let Some(existing_key) = self.id_bindings.find_key(&id).unref() {
      let existing_value = unsafe { self.slot_map.get_unchecked_mut(existing_key) };

      (existing_key, Some(replace(existing_value, f(existing_key))))
    } else {
      let new_key = self.slot_map.insert_with_key(f);

      self.id_bindings.insert_at_value(id, new_key);

      (new_key, None)
    }
  }

  /// Add a value to a SlotMap,
  /// using a closure that receives the Key
  /// that will be used to retrieve the value later
  /// 
  /// If a value is already registered with the given ID,
  /// the closure is not called and this method does nothing
  /// 
  /// If no value is already register,
  /// this method returns the Key associated
  /// with the value returned by the closure
  pub fn insert_unique_with_key<F: FnOnce(K) -> V> (&mut self, id: String, f: F) -> Option<K> {
    if self.id_bindings.contains_value(&id) {
      None
    } else {
      let new_key = self.slot_map.insert_with_key(f);

      self.id_bindings.insert_at_value(id, new_key);

      Some(new_key)
    }
  }
  

  /// Remove the value associated with a Key in a NamedSlotMap,
  /// if it (still) exists
  /// 
  /// Returns the removed value and its ID if it exists
  pub fn remove (&mut self, key: K) -> Option<(String, V)> {
    if let Some(value) = self.slot_map.remove(key) {
      let (_, id) = self.id_bindings.remove_by_key(&key).unwrap();

      Some((id, value))
    } else {
      None
    }
  }


  /// Get an immutable slice of the IDs of a NamedSlotMap
  #[inline]
  pub fn ids (&self) -> &[String] {
    self.id_bindings.values()
  }

  /// Get a mutable iterator over the IDs of a NamedSlotMap
  #[inline]
  pub fn ids_mut (&mut self) -> &mut [String] {
    self.id_bindings.values_mut()
  }

  /// Get an immutable slice of the keys of a NamedSlotMap
  #[inline]
  pub fn keys (&self) -> &[K] {
    self.slot_map.keys()
  }

  /// Get an immutable slice of the values of a NamedSlotMap
  #[inline]
  pub fn values (&self) -> &[V] {
    self.slot_map.values()
  }

  /// Get a mutable slice of the values of a NamedSlotMap
  #[inline]
  pub fn values_mut (&mut self) -> &mut [V] {
    self.slot_map.values_mut()
  }
  

  /// Get an immutable iterator over the keys of a NamedSlotMap
  #[inline]
  pub fn key_iter (&self) -> SliceIter<K> {
    self.slot_map.key_iter()
  }

  /// Get an immutable iterator over the values of a NamedSlotMap
  #[inline]
  pub fn value_iter (&self) -> SliceIter<V> {
    self.slot_map.iter()
  }

  /// Get a mutable iterator over the values of a NamedSlotMap
  #[inline]
  pub fn value_iter_mut (&mut self) -> SliceIterMut<V> {
    self.slot_map.iter_mut()
  }

  /// Get an immutable iterator over the (ID, Key) pairs of a NamedSlotMap
  #[inline]
  pub fn id_key_iter (&self) -> BiMapPairIter<K, String> {
    self.id_bindings.iter()
  }

  /// Get an immutable iterator over the (Key, value) pairs of a NamedSlotMap
  #[inline]
  pub fn key_value_iter (&self) -> SlotMapPairIter<K, V> {
    self.slot_map.pair_iter()
  }

  /// Get a (value) mutable iterator over the (Key, value) pairs of a NamedSlotMap
  #[inline]
  pub fn key_value_iter_mut (&mut self) -> SlotMapPairIterMut<K, V> {
    self.slot_map.pair_iter_mut()
  }

  /// Get an immutable iterator over the (ID, value) pairs of a NamedSlotMap
  #[inline]
  pub fn id_value_iter (&self) -> IDIter<K, V> {
    IDIter::new(self)
  }

  /// Get a mutable iterator over the (ID, value) pairs of a NamedSlotMap
  #[inline]
  pub fn id_value_iter_mut (&mut self) -> IDIterMut<K, V> {
    IDIterMut::new(self)
  }

  /// Get an immutable iterator over the (ID, Key, value) tris of a NamedSlotMap
  #[inline]
  pub fn tri_iter (&self) -> TriIter<K, V> {
    TriIter::new(self)
  }

  /// Get a (ID and value) mutable iterator over the (ID, Key, value) tris of a NamedSlotMap
  #[inline]
  pub fn tri_iter_mut (&mut self) -> TriIterMut<K, V> {
    TriIterMut::new(self)
  }
}


impl<K: Key, V> Index<K> for NamedSlotMap<K, V> {
  type Output = V;

  #[inline]
  fn index (&self, key: K) -> &Self::Output {
    self.get(key).expect("Attempted NamedSlotMap[Key] access to invalid key")
  }
}

impl<K: Key, V> IndexMut<K> for NamedSlotMap<K, V> {
  #[inline]
  fn index_mut (&mut self, key: K) -> &mut Self::Output {
    self.get_mut(key).expect("Attempted NamedSlotMap[Key] access to invalid key")
  }
}


impl<K: Key, V> Index<&str> for NamedSlotMap<K, V> {
  type Output = V;

  #[inline]
  fn index (&self, id: &str) -> &Self::Output {
    self.get(self.find_key(id).expect("Attempted NamedSlotMap[ID] access to invalid ID")).unwrap()
  }
}

impl<K: Key, V> IndexMut<&str> for NamedSlotMap<K, V> {
  #[inline]
  fn index_mut (&mut self, id: &str) -> &mut Self::Output {
    self.get_mut(self.find_key(id).expect("Attempted NamedSlotMap[ID] access to invalid ID")).unwrap()
  }
}


/// An iterator of (ID, Value) for a NamedSlotMap
pub struct IDIter<'a, K: Key, V> {
  slot_map: &'a SlotMap<K, V>,
  inner: BiMapPairIter<'a, K, String>,
}

impl<'a, K: Key, V> IDIter<'a, K, V> {
  /// Create a new IDIter for a NamedSlotMap
  #[inline]
  pub fn new (map: &'a NamedSlotMap<K, V>) -> Self {
    Self {
      slot_map: &map.slot_map,
      inner: map.id_key_iter()
    }
  } 
}

impl<'a, K: Key, V> Iterator for IDIter<'a, K, V> {
  type Item = (&'a String, &'a V);

  fn next (&mut self) -> Option<Self::Item> {
    if let Some((key, id)) = self.inner.next() {
      let value = unsafe { self.slot_map.get_unchecked(*key) };

      Some((id, value))
    } else {
      None
    }
  }
}

/// An iterator of (mut ID, mut Value) for a NamedSlotMap
pub struct IDIterMut<'a, K: Key, V> {
  slot_map: &'a mut SlotMap<K, V>,
  inner: BiMapPairIterMut<'a, K, String>,
}

impl<'a, K: Key, V> IDIterMut<'a, K, V> {
  /// Create a new IDIterMut for a NamedSlotMap
  #[inline]
  pub fn new (map: &'a mut NamedSlotMap<K, V>) -> Self {
    Self {
      slot_map: &mut map.slot_map,
      inner: map.id_bindings.iter_mut()
    }
  } 
}

impl<'a, K: Key, V> Iterator for IDIterMut<'a, K, V> {
  type Item = (&'a mut String, &'a mut V);

  fn next (&mut self) -> Option<Self::Item> {
    if let Some((key, id)) = self.inner.next() {
      // Transmute to rejigger the lifetime, this is safe because we are lifetime-bound elsewhere,
      // but doesnt work by default because of the anon lifetime on self here
      let value = unsafe { transmute(self.slot_map.get_unchecked_mut(*key)) };

      Some((id, value))
    } else {
      None
    }
  }
}

/// An iterator over (ID, Key, Value) for a NamedSlotMap
pub struct TriIter<'a, K: Key, V> {
  slot_map: &'a SlotMap<K, V>,
  inner: BiMapPairIter<'a, K, String>,
}

impl<'a, K: Key, V> TriIter<'a, K, V> {
  /// Create a new TriIter for a NamedSlotMap
  #[inline]
  pub fn new (map: &'a NamedSlotMap<K, V>) -> Self {
    Self {
      slot_map: &map.slot_map,
      inner: map.id_key_iter()
    }
  }
}

impl<'a, K: Key, V> Iterator for TriIter<'a, K, V> {
  type Item = (&'a String, &'a K, &'a V);

  fn next (&mut self) -> Option<Self::Item> {
    if let Some((key, id)) = self.inner.next() {
      let value = unsafe { self.slot_map.get_unchecked(*key) };

      Some((id, key, value))
    } else {
      None
    }
  }
}


/// An iterator over (mut ID, Key, mut Value) for a NamedSlotMap
pub struct TriIterMut<'a, K: Key, V> {
  slot_map: &'a mut SlotMap<K, V>,
  inner: BiMapPairIterMut<'a, K, String>,
}

impl<'a, K: Key, V> TriIterMut<'a, K, V> {
  /// Create a new TriIterMut for a NamedSlotMap
  #[inline]
  pub fn new (map: &'a mut NamedSlotMap<K, V>) -> Self {
    Self {
      slot_map: &mut map.slot_map,
      inner: map.id_bindings.iter_mut()
    }
  }
}

impl<'a, K: Key, V> Iterator for TriIterMut<'a, K, V> {
  type Item = (&'a mut String, &'a K, &'a mut V);

  fn next (&mut self) -> Option<Self::Item> {
    if let Some((key, id)) = self.inner.next() {
      // Transmute to rejigger the lifetime, this is safe because we are lifetime-bound elsewhere,
      // but doesnt work by default because of the anon lifetime on self here
      let value = unsafe { transmute(self.slot_map.get_unchecked_mut(*key)) };

      Some((id, key, value))
    } else {
      None
    }
  }
}