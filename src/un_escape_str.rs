/// Unescape special character sequences into their literal equivalent
/// 
/// For example `\n` becomes a real new line character
/// 
/// Expects utf escapes to be in the format `\uXXXX` where `X` are hex digits
/// 
/// This version creates a new String, use `unescape_str_into` to use an existing String
#[inline]
pub fn unescape_str (source: &str) -> String {
  let mut result = String::new();
  unescape_str_into(source, &mut result);
  result
}


/// Unescape special character sequences into their literal equivalent
/// 
/// For example `\n` becomes a real new line character
/// 
/// Expects utf escapes to be in the format `\uXXXX` where `X` are hex digits
/// 
/// This version copies onto the end of an existing String, use `unescape_str` to use a new String
/// 
/// Note that if the last char of the String is an unaccompanied backslash `\`,
/// this is considered an invalid escape sequence and it is simply discarded
pub fn unescape_str_into (source: &str, dest: &mut String) {
  dest.reserve(source.len());

  let mut chars = source.chars();

  while let Some(ch) = chars.next() {
    dest.push(
      if ch != '\\' {
        ch
      } else {
        match chars.next() {
          Some('u') => {
            let value = chars.by_ref().take(4).fold(0, |acc, c| acc * 16 + c.to_digit(16).unwrap());
            std::char::from_u32(value).unwrap()
          }
          Some('b') => '\x08',
          Some('f') => '\x0c',
          Some('n') => '\n',
          Some('r') => '\r',
          Some('t') => '\t',

          Some(ch) => ch,

          None => return
        }
      }
    )
  }
}


/// Unescape special character sequences into their serialization-safe equivalent
/// 
/// For example `\n` becomes two characters, `\` followed by `n`
/// 
/// Utf escapes to be in the format `\uXXXX` where `X` are hex digits
/// 
/// This version creates a new String, use `escape_str_into` to use an existing String
#[inline]
pub fn escape_str (source: &str) -> String {
  let mut result = String::new();
  escape_str_into(source, &mut result);
  result
}

/// Unescape special character sequences into their serialization-safe equivalent
/// 
/// For example `\n` becomes two characters, `\` followed by `n`
/// 
/// Utf escapes to be in the format `\uXXXX` where `X` are hex digits
/// 
/// This version copies onto the end of an existing String, use `escape_str` to use a new String
pub fn escape_str_into (source: &str, dest: &mut String) {
  dest.reserve(source.len());

  for ch in source.chars() {
    match ch {
      '\\' => dest.push_str("\\\\"),
      '\x08' => dest.push_str("\\b"),
      '\x0c' => dest.push_str("\\f"),
      '\'' => dest.push_str("\\'"),
      '"' => dest.push_str("\\\""),
      '\n' => dest.push_str("\\n"),
      '\r' => dest.push_str("\\r"),
      '\t' => dest.push_str("\\t"),
      '\x7f' ..= std::char::MAX => {
        let mut esc = *b"\\u0000";

        for hex_digit_idx in (0..4).rev() {
          let digit = (((ch as u32) >> (hex_digit_idx * 4)) & 0xf) as u8;
          esc[5 - hex_digit_idx] = if digit < 10 { b'0' + digit } else { b'a' + digit - 10 }
        }

        dest.push_str(unsafe { std::str::from_utf8_unchecked(&esc) });
      },
      _ => dest.push(ch)
    }
  }
}


#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn unescape_ok () {
    let result = unescape_str(r#"\\\"\u2764"#);
    let expected = "\\\"\u{2764}";
    println!("Got unescaped string: `{}`", result);
    println!("Expected: `{}`", expected);
    assert_eq!(expected, result);
  }

  #[test]
  fn escape_ok () {
    let result = escape_str("\\\"\u{2764}");
    let expected = r#"\\\"\u2764"#;
    println!("Got escaped string: `{}`", result);
    println!("Expected: `{}`", expected);
    assert_eq!(expected, result);
  }
}