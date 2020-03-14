use std::{
  fmt::Debug,
  // hash::Hash,
};

/// Convenient shortcut trait for guarding on types that implement all the standard deriveable traits
pub trait POD: Debug
             + Clone
             + Copy
             + PartialEq
            //  + Eq
             + PartialOrd
            //  + Ord
            //  + Hash
             + Default
{ }

impl<T> POD for T
where T: Debug
       + Clone
       + Copy
       + PartialEq
      //  + Eq
       + PartialOrd
      //  + Ord
      //  + Hash
       + Default
{ }