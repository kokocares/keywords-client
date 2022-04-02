#![deny(clippy::all)]

use libc::c_int;
use napi::bindgen_prelude::*;
use std::ptr;

use std::ffi::CString;
use std::os::raw::c_char;

#[link(name = "koko_keywords")]
extern "C" {
  fn c_koko_keywords_match(
    input: *const c_char,
    filter: *const c_char,
    version: *const c_char,
  ) -> c_int;
}

#[macro_use]
extern crate napi_derive;

#[napi(object)]
pub struct MatchOptions {
  pub filter: Option<String>,
  pub version: Option<String>,
}

impl Default for MatchOptions {
  fn default() -> Self {
    MatchOptions {
      filter: None,
      version: None,
    }
  }
}

#[allow(dead_code)]
#[napi(js_name = "match")]
fn keywords_match(input: String, options: Option<MatchOptions>) -> Result<bool> {
  let c_input = CString::new(input).expect("cstring::new failed");

  let options = options.unwrap_or_default();

  let filter = options.filter.unwrap_or("".to_string());
  let c_filter = CString::new(filter).expect("cstring::new failed");

  let version = options.version.unwrap_or("".to_string());
  let is_empty = version.is_empty();
  let c_version = CString::new(version).expect("cstring::new failed");
  let c_version_ptr = if is_empty {
    ptr::null()
  } else {
    c_version.as_ptr()
  };

  let rc = unsafe { c_koko_keywords_match(c_input.as_ptr(), c_filter.as_ptr(), c_version_ptr) };
  match rc {
    -1 => Err(Error::new(
      napi::Status::GenericFailure,
      "KOKO_KEYWORDS_AUTH must be set before importing the library".to_string(),
    )),
    -2 => Err(Error::new(
      napi::Status::GenericFailure,
    "Invalid credentials. Please confirm you are using valid credentials, contact us at api.kokocares.org if you need assistance.".to_string(),
    )),
    -3 => Err(Error::new(
      napi::Status::GenericFailure,
    "Unable to refresh cache. Please try again or contact us at api.kokocares.org if this issue persists.".to_string(),
    )),
    -4 => Err(Error::new(
      napi::Status::GenericFailure,
      "Unable to parse response from API. Please contact us at api.kokocares.org if this issue persists.".to_string(),
    )),
    -5 => Err(Error::new(
      napi::Status::GenericFailure,
      "Invalid url. Please ensure the url used is valid.".to_string(),
    )),
    _ => Ok(rc != 0),
  }
}
