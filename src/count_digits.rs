/// Determine how many digits it takes to represent a number `n` in the given `base`
pub fn count_digits (mut n: usize, base: usize) -> usize {
  let mut d = 0;

  loop {
    d += 1;
    n /= base;
    
    if n == 0 { break d }
  }
}