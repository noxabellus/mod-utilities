use std::fmt::{
  Write,
  Result,
};

/// Write `level` copies of `indent` into `w`
pub fn write_indent (level: u32, indent: &str, w: &mut dyn Write) -> Result {
  for _ in 0..level { write!(w, "{}", indent)?; }
  Ok(())
}