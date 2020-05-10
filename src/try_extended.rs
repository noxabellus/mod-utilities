/// Extract the value of an Option::Some or return from the current context
/// 
/// Works similar to the try operator, but is usable in functions that return ()
#[macro_export]
macro_rules! some {
  ($val: expr) => { if let Some(v) = $val { v } else { return } };
  ($val: expr ; $ret: expr) => { if let Some(v) = $val { v } else { return $ret } };
}

/// Extract the value of a Result::Ok or return from the current context
/// 
/// Works similar to the try operator, but is usable in functions that return ()
#[macro_export]
macro_rules! ok {
  ($val: expr) => { if let Ok(v) = $val { v } else { return } };
  ($val: expr ; $ret: expr) => { if let Ok(v) = $val { v } else { return $ret } };
}

/// Extract the value of a Result::Err or return from the current context
/// 
/// Works similar to the try operator, but is usable in functions that return ()
#[macro_export]
macro_rules! err {
  ($val: expr) => { if let Err(v) = $val { v } else { return } };
  ($val: expr ; $ret: expr) => { if let Err(v) = $val { v } else { return $ret } };
}

/// A block with fallthrough capability
#[macro_export]
macro_rules! breakable_block {
  ($($tt: tt)*) => { #[allow(unreachable_code)] loop {
    { $($tt)* }
    unreachable!("breakable_block not broken");
  } }
}