//! A collection of data structures, traits, and functions for use in mod engine / mod language

#![warn(missing_docs)]
#![warn(clippy::all)]

#![allow(incomplete_features)]
#![feature(const_generics)]

mod take_first;
mod discard;
mod try_extended;

mod pod;
pub use pod::POD;

pub mod collections;

mod unwrap_pretty;
pub use unwrap_pretty::UnwrapPretty;

mod unwrap_unchecked;
pub use unwrap_unchecked::UnwrapUnchecked;

mod allow_if;
pub use allow_if::AllowIf;

mod unref;
pub use unref::Unref;

pub mod temp;

pub mod wrapped_array;
pub use wrapped_array::WrappedArray;

mod reduce;
pub use reduce::Reduce;

mod write_indent;
pub use write_indent::write_indent;

mod into_result;
pub use into_result::IntoResult;

mod count_digits;
pub use count_digits::count_digits;

mod padding;
pub use padding::padding;

mod write_adaptor;
pub use write_adaptor::*;

mod un_escape_str;
pub use un_escape_str::*;

mod either;
pub use either::*;