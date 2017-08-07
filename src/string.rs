//! Functionality for handling strings when working with LLVM.

use std::fmt;
use std::fmt::{Display, Debug};
use std::ffi::{CStr, OsStr};
use std::mem::transmute;
use std::ops::Deref;

use super::*;

/// Representation of the data of a C-style (null terminated) string.
// TODO: make this an unsized type. A slice doesn't work, because &Str needs to
// have the same size as *const i8 so that conversions are a simple `transmute`.
// TODO: Replace this with std::ffi::CStr when CStr no longer requries upfront
// length calculations, which will also probably be when it's possible to make
// this an unsized type.
#[allow(dead_code)]
pub struct Str {
    data: i8,
}

impl Str {
    /// 0-cost cast to an &llvm::Str from a pointer to a C-style string that
    /// must originate from LLVM.
    pub unsafe fn from_ptr<'a>(ptr: *const i8) -> &'a Str {
        transmute(ptr)
    }

    pub fn as_ptr(&self) -> *const i8 {
        unsafe { transmute(self) }
    }

    /// Creates a string slice pointing to the data of this llvm::String, not
    /// including the null-terminator. This performs a length calculation, so
    /// this conversion isn't free.
    fn as_str<'a>(&'a self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(CStr::from_ptr(self.as_ptr()).to_bytes()) }
    }
}

impl Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<Str> for Str {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<str> for Str {
    fn as_ref<'a>(&'a self) -> &'a str {
        unsafe {
            CStr::from_ptr(transmute(self)).to_str().expect(
                "LLVM string contained invalid UTF-8 somehow.",
            )
        }
    }
}

impl AsRef<OsStr> for Str {
    fn as_ref(&self) -> &OsStr {
        <Str as AsRef<str>>::as_ref(self).as_ref()
    }
}

/// Wrapper for owned strings received from LLVM.
///
/// The LLVM C API sometimes returns strings that need to be `free`d, but manual
/// memory management is not idiomatic in Rust, so we wrap them with this.
pub struct String {
    ptr: *mut i8,
}

/*impl String {
    /// Creates a string slice pointing to the data of this llvm::String, not
    /// including the null-terminator. This performs a length calculation, so
    /// this conversion isn't free.
    fn as_str<'a>(&'a self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(CStr::from_ptr(self.ptr).to_bytes()) }
    }
}*/

impl String {
    /// 0-cost cast to an llvm::String from a pointer to an owned string that
    /// must originate from LLVM.
    pub(crate) fn from_mut(ptr: *mut i8) -> String {
        String { ptr }
    }
}

impl Display for String {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Str as Display>::fmt(self.as_ref(), f)
    }
}

impl Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Str as Debug>::fmt(self.as_ref(), f)
    }
}

impl AsRef<Str> for String {
    fn as_ref(&self) -> &Str {
        unsafe { Str::from_ptr(self.ptr) }
    }
}

impl Deref for String {
    type Target = Str;

    fn deref<'a>(&'a self) -> &'a Self::Target {
        unsafe { Str::from_ptr(self.ptr) }
    }
}

impl Drop for String {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeMessage(self.ptr);
        }
    }
}

// Cast from `CString`s to `&llvm::Str`.
impl AsRef<Str> for std::ffi::CString {
    fn as_ref(&self) -> &Str {
        unsafe { Str::from_ptr(self.as_ptr()) }
    }
}

/// Turn non-null terminated string literal into null-terminated
/// `&'static llvm::Str`. Note that this won't work in static variables, but it
/// does work with the `lazy_static` crate.
///
/// Passing no argument creates an empty string, and is equivalent to
/// `llvm_str!("")`.
///
/// # Example
///
/// ```rust
/// #[macro_use]extern crate llvm;
/// # fn main() {
/// # let mut context = llvm::Context::new();
/// let mut my_module = context.create_module_with_name(llvm_str!("my module"));
/// # }
/// ```
// TODO: when stmt_expr_attributes (rust issue #15701) is finished, uncomment
// the `#[allow(unused_unsafe)]` below
#[macro_export]
macro_rules! llvm_str {
    ($s:expr) => {
        //#[allow(unused_unsafe)]
        unsafe { llvm::Str::from_ptr(concat!($s, "\0").as_ptr() as *mut i8) }
    };
    () => {
        //#[allow(unused_unsafe)]
        unsafe { llvm::Str::from_ptr(&mut 0i8 as *mut i8) }
    }
}
