/// Get a str filled with a given number of spaces, up to 256
pub fn padding (n: u8) -> &'static str {
  const SPACES: [u8; 256] = [b' '; 256];

  unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(SPACES.as_ptr(), n as _)) }
}