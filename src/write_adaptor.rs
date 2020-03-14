use std::{ fmt, io };



/// A shim which translates an io::Write to a fmt::Write
/// 
/// If an error occurs with the inner io::Write,
/// it is stored in an optional member,
/// and all further calls to fmt::Write
/// will be short circuited until it is cleared
/// 
/// # Safety
/// + Does not call `flush`
/// + Adapting an adaptor would cause all sorts of problems
pub struct FMTAdaptor<'a>{
  io_writer: &'a mut dyn io::Write,
  /// If an io error occurs while using a FMTAdaptor,
  /// it will be stored here and block further usage until it is resolved
  pub error: Option<io::Error>
}

/// Allows a io::Write to be routed to an fmt::Write
/// 
/// See FMTAdaptor for more details
pub trait FMTAdaptable {
  /// Create a FMTAdaptor for an io::Write
  fn adapt_to_fmt (&mut self) -> FMTAdaptor;
}

impl<T> FMTAdaptable for T
where T: io::Write
{
  #[inline]
  fn adapt_to_fmt (&mut self) -> FMTAdaptor<'_> {
    FMTAdaptor { io_writer: self, error: None }
  }
}

impl<'a> fmt::Write for FMTAdaptor<'a> {
  #[inline]
  fn write_str (&mut self, s: &str) -> fmt::Result {
    if self.error.is_none() {
      match self.io_writer.write_all(s.as_bytes()) {
        Ok(()) => Ok(()),
        Err(e) => {
          self.error = Some(e);
          Err(fmt::Error)
        }
      }
    } else {
      Err(fmt::Error)
    }
  }
}



/// A shim which translates a fmt::Write to a io::Write
/// 
/// # Safety
/// + This will not return a valid length from the `io::Write::write` implementation
/// + `flush` does nothing
/// + Adapting an adaptor would cause all sorts of problems
pub struct IOAdaptor<'a>(&'a mut dyn fmt::Write);

/// Allows a fmt::Write to be routed to an io::Write
/// 
/// See IOAdaptor for more details
pub trait IOAdaptable {
  /// Create a IOAdaptor for a fmt::Write
  fn adapt_to_io (&mut self) -> IOAdaptor;
}

impl<T> IOAdaptable for T
where T: fmt::Write
{
  #[inline]
  fn adapt_to_io (&mut self) -> IOAdaptor<'_> {
    IOAdaptor(self)
  }
}

impl<'a> io::Write for IOAdaptor<'a> {
  #[inline]
  fn write (&mut self, buf: &[u8]) -> io::Result<usize> {
    match std::str::from_utf8(buf) {
      Ok(str) => match self.0.write_str(str) {
        Ok(()) => Ok(0),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e))
      },
      Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e))
    }
  }

  #[inline]
  fn flush (&mut self) -> io::Result<()> { Ok(()) }
}