//! BiMap and support structures

pub use std::{
  hash::{
    Hash,
    Hasher,
  },
  collections::hash_map::DefaultHasher,
  mem::replace,
  slice::{
    Iter as SliceIter,
    IterMut as SliceIterMut,
  },
  ops::{
    Index,
    IndexMut,
  },
  marker::PhantomData,
  iter::FromIterator,
  vec::IntoIter as VecIntoIter,
};


/// An associative array of keys to values
/// 
/// Allows bi-directional lookup,
/// using hashing for both keys and values
/// 
/// Both the Key and Value types must implement PartialEq, and Hash
#[derive(Debug, Clone)]
pub struct BiMap<K: PartialEq + Hash, V: PartialEq + Hash> {
  keys: Vec<K>,
  values: Vec<V>,
  key_hashes: Vec<u64>,
  value_hashes: Vec<u64>,
}

impl<K: PartialEq + Hash, V: PartialEq + Hash> BiMap<K, V> {
  const DEFAULT_CAPACITY: usize = 256;

  /// Used by all Dictionaries of a given type to generate key_hashes from keys
  #[inline]
  pub fn hash_key<EqK: Hash + ?Sized> (key: &EqK) -> u64
  where K: PartialEq<EqK>
  {
    let mut hasher = DefaultHasher::new();

    key.hash(&mut hasher);

    hasher.finish()
  }

  /// Used by all Dictionaries of a given type to generate value_hashes from values
  #[inline]
  pub fn hash_value<EqV: Hash + ?Sized> (value: &EqV) -> u64
  where V: PartialEq<EqV>
  {
    let mut hasher = DefaultHasher::new();

    value.hash(&mut hasher);

    hasher.finish()
  }

  /// Create a BiMap and pre-allocate its Vecs with a specified capacity
  #[inline]
  pub fn with_capacity (cap: usize) -> Self {
    Self {
      keys: Vec::with_capacity(cap),
      values: Vec::with_capacity(cap),
      key_hashes: Vec::with_capacity(cap),
      value_hashes: Vec::with_capacity(cap),
    }
  }

  /// Create a BiMap and pre-allocate its Vecs with the BiMap::DEFAULT_CAPACITY
  #[inline]
  pub fn new () -> Self {
    Self::with_capacity(Self::DEFAULT_CAPACITY)
  }


  #[inline]
  fn index_of_hashed_key<EqK: Hash + ?Sized> (&self, hash: u64, key: &EqK) -> Option<usize>
  where K: PartialEq<EqK>
  {
    for (idx, own_hash) in self.key_hashes.iter().enumerate() {
      if *own_hash == hash {
        let own_key = unsafe { self.keys.get_unchecked(idx) };

        if own_key == key {
          return Some(idx)
        }
      }
    }

    None
  }

  /// Find the vec index of a key if it exists in a BiMap
  pub fn index_of_key<EqK: Hash + ?Sized> (&self, key: &EqK) -> Option<usize>
  where K: PartialEq<EqK>
  {
    self.index_of_hashed_key(Self::hash_key(key), key)
  }
  

  #[inline]
  fn index_of_hashed_value<EqV: Hash + ?Sized> (&self, hash: u64, value: &EqV) -> Option<usize>
  where V: PartialEq<EqV>
  {
    for (idx, own_hash) in self.value_hashes.iter().enumerate() {
      if *own_hash == hash {
        let own_value = unsafe { self.values.get_unchecked(idx) };

        if own_value == value {
          return Some(idx)
        }
      }
    }

    None
  }

  /// Find the vec index of a value if it exists in a BiMap
  pub fn index_of_value<EqV: Hash + ?Sized> (&self, value: &EqV) -> Option<usize>
  where V: PartialEq<EqV>
  {
    self.index_of_hashed_value(Self::hash_value(value), value)
  }


  /// Determine if a BiMap contains a given key
  #[inline]
  pub fn contains_key<EqK: Hash + ?Sized> (&self, key: &EqK) -> bool
  where K: PartialEq<EqK>
  {
    self.index_of_key(key).is_some()
  }

  /// Determine if a BiMap contains a given value
  #[inline]
  pub fn contains_value<EqV: Hash + ?Sized> (&self, value: &EqV) -> bool
  where V: PartialEq<EqV>
  {
    self.index_of_value(value).is_some()
  }


  /// Determine if a BiMap potentially contains a given key
  /// 
  /// This works by comparing key_hashes only, and may yield false positives,
  /// but will never yield a false negative
  pub fn maybe_contains_key<EqK: Hash + ?Sized> (&self, key: &EqK) -> bool
  where K: PartialEq<EqK>
  {
    let hash = Self::hash_key(key);

    for own_hash in self.key_hashes.iter() {
      if *own_hash == hash {
        return true
      }
    }

    false
  }

  /// Determine if a BiMap potentially contains a given value
  /// 
  /// This works by comparing value_hashes only, and may yield false positives,
  /// but will never yield a false negative
  pub fn maybe_contains_value<EqV: Hash + ?Sized> (&self, value: &EqV) -> bool
  where V: PartialEq<EqV>
  {
    let hash = Self::hash_value(value);

    for own_hash in self.value_hashes.iter() {
      if *own_hash == hash {
        return true
      }
    }

    false
  }


  /// Get the number of (key, value) pairs in a BiMap
  #[inline]
  pub fn len (&self) -> usize {
    self.values.len()
  }

  /// Determine if a BiMap contains any (key, value) pairs
  #[inline]
  pub fn is_empty (&self) -> bool {
    self.values.is_empty()
  }


  /// Get an immutable reference to a value associated with a given key in a BiMap,
  /// if it contains a pair with a matching key
  #[inline]
  pub fn find_value<EqK: Hash + ?Sized> (&self, key: &EqK) -> Option<&V>
  where K: PartialEq<EqK>
  {
    if let Some(idx) = self.index_of_key(key) {
      Some(unsafe { self.values.get_unchecked(idx) })
    } else {
      None
    }
  }

  /// Get a mutable reference to a value associated with a given key in a BiMap,
  /// if it contains a pair with a matching key
  #[inline]
  pub fn find_value_mut<EqK: Hash + ?Sized> (&mut self, key: &EqK) -> Option<&mut V>
  where K: PartialEq<EqK>
  {
    if let Some(idx) = self.index_of_key(key) {
      Some(unsafe { self.values.get_unchecked_mut(idx) })
    } else {
      None
    }
  }


  /// Get an immutable reference to a key associated with a given value in a BiMap,
  /// if it contains a pair with a matching value
  #[inline]
  pub fn find_key<EqV: Hash + ?Sized> (&self, value: &EqV) -> Option<&K>
  where V: PartialEq<EqV>
  {
    if let Some(idx) = self.index_of_value(value) {
      Some(unsafe { self.keys.get_unchecked(idx) })
    } else {
      None
    }
  }

  /// Get a mutable reference to a key associated with a given value in a BiMap,
  /// if it contains a pair with a matching value
  #[inline]
  pub fn find_key_mut<EqV: Hash + ?Sized> (&mut self, value: &EqV) -> Option<&mut K>
  where V: PartialEq<EqV>
  {
    if let Some(idx) = self.index_of_value(value) {
      Some(unsafe { self.keys.get_unchecked_mut(idx) })
    } else {
      None
    }
  }

  
  /// Get an immutable references to a (key, value) pair in a BiMap by index
  /// 
  /// # Safety
  /// Does not range check the index
  /// 
  /// Note that the BiMap type does not necessarily preserve its order,
  /// so index-based referencing is temporaly unstable
  #[inline]
  pub unsafe fn get_pair_unchecked (&self, idx: usize) -> (&K, &V) {
    (self.keys.get_unchecked(idx), self.values.get_unchecked(idx))
  }

  /// Get a mutable references to a (key, value) pair in a BiMap by index
  /// 
  /// # Safety
  /// Does not range check the index
  /// 
  /// Note that the BiMap type does not necessarily preserve its order,
  /// so index-based referencing is temporaly unstable
  #[inline]
  pub unsafe fn get_pair_unchecked_mut (&mut self, idx: usize) -> (&mut K, &mut V) {
    (self.keys.get_unchecked_mut(idx), self.values.get_unchecked_mut(idx))
  }


  /// Get an immutable references to a (key, value) pair in a BiMap by index
  /// 
  /// A range check is performed on the index
  /// 
  /// Note that the BiMap type does not necessarily preserve its order,
  /// so index-based referencing is temporaly unstable
  #[inline]
  pub fn get_pair (&self, idx: usize) -> Option<(&K, &V)> {
    if idx < self.len() {
      Some(unsafe { self.get_pair_unchecked(idx) })
    } else {
      None
    }
  }

  /// Get a mutable references to a (key, value) pair in a BiMap by index
  /// 
  /// A range check is performed on the index
  /// 
  /// Note that the BiMap type does not necessarily preserve its order,
  /// so index-based referencing is temporaly unstable
  #[inline]
  pub fn get_pair_mut (&mut self, idx: usize) -> Option<(&mut K, &mut V)> {
    if idx < self.len() {
      Some(unsafe { self.get_pair_unchecked_mut(idx) })
    } else {
      None
    }
  }


  /// Insert a (key, value) pair at a key location, overwriting the existing value if one is found
  /// 
  /// Returns the existing value if one is already bound to the key
  /// (The opposite of `insert_unique`)
  #[inline]
  pub fn insert_at_key (&mut self, key: K, value: V) -> Option<V> {
    let key_hash = Self::hash_key(&key);
    let value_hash = Self::hash_value(&value);

    for (idx, own_hash) in self.key_hashes.iter().enumerate() {
      if *own_hash == key_hash {
        let own_key = unsafe { self.keys.get_unchecked(idx) };

        if own_key == &key {
          replace(unsafe { self.value_hashes.get_unchecked_mut(idx) }, value_hash);
          return Some(replace(unsafe { self.values.get_unchecked_mut(idx) }, value))
        }
      }
    }

    self.keys.push(key);
    self.values.push(value);
    self.key_hashes.push(key_hash);
    self.value_hashes.push(value_hash);

    None
  }

  /// Insert a (key, value) pair at a value location, overwriting the existing key if one is found
  /// 
  /// Returns the existing key if one is already bound to the value
  /// (The opposite of `insert_unique`)
  #[inline]
  pub fn insert_at_value (&mut self, value: V, key: K) -> Option<K> {
    let key_hash = Self::hash_key(&key);
    let value_hash = Self::hash_value(&value);

    for (idx, own_hash) in self.value_hashes.iter().enumerate() {
      if *own_hash == value_hash {
        let own_value = unsafe { self.values.get_unchecked(idx) };

        if own_value == &value {
          replace(unsafe { self.key_hashes.get_unchecked_mut(idx) }, key_hash);
          return Some(replace(unsafe { self.keys.get_unchecked_mut(idx) }, key))
        }
      }
    }

    self.keys.push(key);
    self.values.push(value);
    self.key_hashes.push(key_hash);
    self.value_hashes.push(value_hash);

    None
  }

  /// Insert a value at the given key in a BiMap if the key does not already exist
  /// 
  /// Returns the given (key, value) pair and does nothing if an existing key is found
  /// (The opposite of `insert`)
  #[inline]
  pub fn insert_unique_key (&mut self, key: K, value: V) -> Option<(K, V)> {
    let key_hash = Self::hash_key(&key);
    let value_hash = Self::hash_value(&value);

    if self.index_of_hashed_key(key_hash, &key).is_some() { return Some((key, value)) }

    self.keys.push(key);
    self.values.push(value);
    self.key_hashes.push(key_hash);
    self.value_hashes.push(value_hash);

    None
  }

  /// Insert a value at the given key in a BiMap if the value does not already exist
  /// 
  /// Returns the given (key, value) pair and does nothing if an existing value is found
  /// (The opposite of `insert`)
  #[inline]
  pub fn insert_unique_value (&mut self, key: K, value: V) -> Option<(K, V)> {
    let key_hash = Self::hash_key(&key);
    let value_hash = Self::hash_value(&value);

    if self.index_of_hashed_value(value_hash, &value).is_some() { return Some((key, value)) }

    self.keys.push(key);
    self.values.push(value);
    self.key_hashes.push(key_hash);
    self.value_hashes.push(value_hash);

    None
  }

  
  /// Removes a (key, value) pair at the given index in a BiMap if it is in range
  /// 
  /// Returns the pair if one is found
  /// 
  /// Does not preserve order
  #[inline]
  pub fn remove_by_index (&mut self, idx: usize) -> Option<(K, V)> {
    if idx < self.len() {
      self.key_hashes.swap_remove(idx);
      self.value_hashes.swap_remove(idx);

      Some((self.keys.swap_remove(idx), self.values.swap_remove(idx)))
    } else {
      None
    }
  }

  /// Removes a (key, value) pair matching the given key in a BiMap if one exists
  /// 
  /// Returns the pair if one is found
  /// 
  /// Does not preserve order
  #[inline]
  pub fn remove_by_key<EqK: Hash + ?Sized> (&mut self, key: &EqK) -> Option<(K, V)>
  where K: PartialEq<EqK>
  {
    self.index_of_key(key).and_then(|idx| self.remove_by_index(idx))
  }
  
  /// Removes the first (key, value) pair matching the given value in a BiMap if one exists
  /// 
  /// Returns the pair if one is found
  /// 
  /// Does not preserve order
  #[inline]
  pub fn remove_by_value<EqV: Hash + ?Sized> (&mut self, value: &EqV) -> Option<(K, V)>
  where V: PartialEq<EqV>
  {
    self.index_of_value(value).and_then(|idx| self.remove_by_index(idx))
  }

  /// Remove a (key, value) pair from a BiMap if there are any
  /// 
  /// Returns the pair if one exists
  /// 
  /// Preserves order, removing the last pair of the BiMap
  #[inline]
  pub fn pop (&mut self) -> Option<(K, V)> {
    if !self.is_empty() {
      self.key_hashes.pop();
      Some((self.keys.pop().unwrap(), self.values.pop().unwrap()))
    } else {
      None
    }
  }


  /// Get an immutable slice of the keys of a BiMap
  #[inline]
  pub fn keys (&self) -> &[K] {
    self.keys.as_slice()
  }

  /// Get a mutable iterator over the keys of a BiMap
  #[inline]
  pub fn keys_mut (&mut self) -> &mut [K] {
    self.keys.as_mut_slice()
  }

  /// Get a mutable slice of the values of a BiMap
  #[inline]
  pub fn values (&self) -> &[V] {
    self.values.as_slice()
  }

  /// Get a mutable iterator over the values of a BiMap
  #[inline]
  pub fn values_mut (&mut self) -> &mut [V] {
    self.values.as_mut_slice()
  }

  /// Get an immutable iterator over the keys of a BiMap
  #[inline]
  pub fn key_iter (&self) -> SliceIter<K> {
    self.keys.iter()
  }


  /// Get a mutable iterator over the keys of a BiMap
  #[inline]
  pub fn key_iter_mut (&mut self) -> SliceIterMut<K> {
    self.keys.iter_mut()
  }


  /// Get an immutable iterator over the values of a BiMap
  #[inline]
  pub fn value_iter (&self) -> SliceIter<V> {
    self.values.iter()
  }

  /// Get a mutable iterator over the values of a BiMap
  #[inline]
  pub fn value_iter_mut (&mut self) -> SliceIterMut<V> {
    self.values.iter_mut()
  }


  /// Get an immutable iterator over the (key, value) pairs of a BiMap
  #[inline]
  pub fn iter (&self) -> PairIter<K, V> {
    PairIter::new(self)
  }

  /// Get a mutable iterator over the (key, value) pairs of a BiMap
  #[inline]
  pub fn iter_mut (&mut self) -> PairIterMut<K, V> {
    PairIterMut::new(self)
  }


  /// Move the (key, value) pairs of another BiMap into a BiMap
  /// 
  /// Uses `insert_unique` to move values, thereby discarding values from the other BiMap,
  /// if they share a key with an existing entry
  /// 
  /// Consumes the other BiMap
  /// 
  /// Use `merge_discard_to_vec` to retain the discarded values
  pub fn merge_discard (&mut self, other: Self) {
    for (key, value) in other {
      self.insert_unique_key(key, value);
    }
  }

  /// Move the (key, value) pairs of another BiMap into a BiMap
  /// 
  /// Uses `insert_unique` to move values, thereby discarding values from the other BiMap,
  /// if they share a key with an existing entry
  /// 
  /// Consumes the other BiMap and retains discarded values in a Vec
  /// 
  /// Use `merge_discard` to drop discard values immediately
  pub fn merge_discard_to_vec (&mut self, other: Self) -> Vec<(K, V)> {
    let mut discard = Vec::new();

    for (key, value) in other {
      if let Some(value) = self.insert_unique_key(key, value) {
        discard.push(value);
      }
    }

    discard
  }

  /// Move the (key, value) pairs of another BiMap into a BiMap
  /// 
  /// Uses `insert` to move values, therby overwriting values from the BiMap,
  /// if they share a key with an entry from the other BiMap
  /// 
  /// Consumes the other BiMap
  pub fn merge_overwrite (&mut self, other: Self) {
    for (key, value) in other {
      self.insert_at_key(key, value);
    }
  }
}



impl<EqK: Hash + ?Sized, K: PartialEq + Hash, V: PartialEq + Hash> Index<&EqK> for BiMap<K, V>
where K: PartialEq<EqK>
{
  type Output = V;

  fn index (&self, key: &EqK) -> &Self::Output {
    self.find_value(key).expect("Attempted BiMap[] access to invalid key")
  }
}

impl<EqK: Hash + ?Sized, K: PartialEq + Hash, V: PartialEq + Hash> IndexMut<&EqK> for BiMap<K, V>
where K: PartialEq<EqK>
{
  fn index_mut (&mut self, key: &EqK) -> &mut Self::Output {
    self.find_value_mut(key).expect("Attempted BiMap[] access to invalid key")
  }
}


/// An iterator over (Key, Value) for a BiMap
pub struct PairIter<'a, K: PartialEq + Hash + 'a, V: PartialEq + Hash + 'a> {
  keys: *const K,
  values: *const V,

  idx: usize,
  len: usize,

  k_phantom: PhantomData<&'a K>,
  v_phantom: PhantomData<&'a V>,
}

impl<'a, K: PartialEq + Hash + 'a, V: PartialEq + Hash + 'a> PairIter<'a, K, V> {
  /// Create a new PairIter for a BiMap
  #[inline]
  pub fn new (map: &'a BiMap<K, V>) -> Self {
    Self {
      keys: map.keys.as_ptr(),
      values: map.values.as_ptr(),

      idx: 0,
      len: map.len(),

      k_phantom: PhantomData,
      v_phantom: PhantomData,
    }
  }
}

impl<'a, K: PartialEq + Hash + 'a, V: PartialEq + Hash + 'a> Iterator for PairIter<'a, K, V> {
  type Item = (&'a K, &'a V);

  fn next (&mut self) -> Option<Self::Item> {
    if self.idx < self.len {
      let pair_idx = self.idx;
      self.idx += 1;

      Some(unsafe { (&*self.keys.add(pair_idx), &*self.values.add(pair_idx)) })
    } else {
      None
    }
  }
}

/// An iterator over (mut Key, mut Value) for a BiMap
pub struct PairIterMut<'a, K: PartialEq + Hash + 'a, V: PartialEq + Hash + 'a> {
  keys: *mut K,
  values: *mut V,

  idx: usize,
  len: usize,

  k_phantom: PhantomData<&'a mut K>,
  v_phantom: PhantomData<&'a mut V>,
}

impl<'a, K: PartialEq + Hash + 'a, V: PartialEq + Hash + 'a> PairIterMut<'a, K, V> {
  /// Create a new PairIterMut for a BiMap
  #[inline]
  pub fn new (map: &'a mut BiMap<K, V>) -> Self {
    Self {
      keys: map.keys.as_mut_ptr(),
      values: map.values.as_mut_ptr(),

      idx: 0,
      len: map.len(),

      k_phantom: PhantomData,
      v_phantom: PhantomData,
    }
  }
}

impl<'a, K: PartialEq + Hash + 'a, V: PartialEq + Hash + 'a> Iterator for PairIterMut<'a, K, V> {
  type Item = (&'a mut K, &'a mut V);

  fn next (&mut self) -> Option<Self::Item> {
    if self.idx < self.len {
      let pair_idx = self.idx;
      self.idx += 1;

      Some(unsafe { (&mut *self.keys.add(pair_idx), &mut *self.values.add(pair_idx)) })
    } else {
      None
    }
  }
}

/// A by-value consuming iterator for a BiMap
pub struct IntoIter<K: PartialEq + Hash, V: PartialEq + Hash> {
  keys: VecIntoIter<K>,
  values: VecIntoIter<V>,
}

impl<K: PartialEq + Hash, V: PartialEq + Hash> Iterator for IntoIter<K, V> {
  type Item = (K, V);

  fn next (&mut self) -> Option<Self::Item> {
    if let Some(key) = self.keys.next() {
      Some((key, self.values.next().unwrap()))
    } else {
      None
    }
  }
}

impl<K: PartialEq + Hash, V: PartialEq + Hash> IntoIterator for BiMap<K, V> {
  type Item = (K, V);
  type IntoIter = IntoIter<K, V>;

  fn into_iter (self) -> Self::IntoIter {
    Self::IntoIter {
      keys: self.keys.into_iter(),
      values: self.values.into_iter()
    }
  }
}


impl<K: PartialEq + Hash, V: PartialEq + Hash> FromIterator<(K, V)> for BiMap<K, V> {
  fn from_iter<I: IntoIterator<Item=(K, V)>> (iter: I) -> Self {
    let mut map = Self::new();

    for (key, value) in iter {
      map.insert_at_key(key, value);
    }

    map
  }
}