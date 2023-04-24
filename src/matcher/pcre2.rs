#![allow(non_camel_case_types)]
#![allow(dead_code)]

use pcre2_sys::*;
use std::fmt;
use std::ptr;
use std::str;

use libc::{c_int, size_t};

pub struct Regex {
    code: *mut pcre2_code_8,
    match_data: *mut pcre2_match_data_8,
    ovector: *mut size_t,
}

unsafe impl Send for Regex {}

impl Drop for Regex {
    fn drop(&mut self) {
        unsafe {
            // Release memory used for the match
            pcre2_match_data_free_8(self.match_data);
            // data and the compiled pattern.
            pcre2_code_free_8(self.code);
        }
    }
}

pub struct Error {
    code: c_int,
    offset: size_t,
}

impl Regex {
    pub fn new(pattern: &str) -> Result<Regex, Error> {
        let mut error_code: c_int = 0;
        let mut error_offset: size_t = 0;
        // compile the regular expression pattern, and handle any errors that are detected.
        let code = unsafe {
            pcre2_compile_8(
                pattern.as_ptr(),
                pattern.len(),
                // PCRE2 can get significantly faster in some cases depending
                // on the permutation of these options (in particular, dropping
                // UCP). We should endeavor to have a separate "ASCII compatible"
                // benchmark.
                PCRE2_UCP | PCRE2_UTF,
                &mut error_code,
                &mut error_offset,
                ptr::null_mut(),
            )
        };
        if code.is_null() {
            return Err(Error {
                code: error_code,
                offset: error_offset,
            });
        }
        // let err = unsafe { pcre2_jit_compile_8(code, PCRE2_JIT_COMPLETE) };
        // if err < 0 {
        //     panic!("pcre2_jit_compile_8 failed with error: {:?}", err);
        // }
        // do a pattern match against the subject string. This does just ONE match.
        let match_data = unsafe { pcre2_match_data_create_from_pattern_8(code, ptr::null_mut()) };
        if match_data.is_null() {
            panic!("could not allocate match_data");
        }
        // Match succeeded. Get a pointer to the output vector, where string offsets are stored.
        let ovector = unsafe { pcre2_get_ovector_pointer_8(match_data) };
        if ovector.is_null() {
            panic!("could not get ovector");
        }
        Ok(Regex {
            code,
            match_data,
            ovector,
        })
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.find_at(text, 0).is_some()
    }

    pub fn find_iter<'r, 't>(&'r self, text: &'t str) -> FindMatches<'r, 't> {
        FindMatches {
            re: self,
            text,
            last_match_end: 0,
        }
    }

    fn find_at(&self, text: &str, start: usize) -> Option<(usize, usize)> {
        // The man pages for PCRE2 say that pcre2_jit_match is the fastest
        // way to execute a JIT match because it skips sanity checks. We also
        // explicitly disable the UTF-8 validity check, but it's probably not
        // necessary.
        let err = unsafe {
            pcre2_match_8(
                self.code,          // the compiled pattern
                text.as_ptr(),      // the subject string
                text.len(),         // the length of the subject
                start,              // the start offset in the subject
                PCRE2_NO_UTF_CHECK, // options
                self.match_data,    // block for storing the result
                ptr::null_mut(),    // use the default match context
            )
        };
        if err == PCRE2_ERROR_NOMATCH {
            None
        } else if err < 0 {
            panic!("unknown error code: {err:?}");
        } else {
            Some(unsafe { (*self.ovector, *self.ovector.offset(1)) })
        }
    }
}

pub struct FindMatches<'r, 't> {
    re: &'r Regex,
    text: &'t str,
    last_match_end: usize,
}

impl<'r, 't> Iterator for FindMatches<'r, 't> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        match self.re.find_at(self.text, self.last_match_end) {
            None => None,
            Some((s, e)) => {
                self.last_match_end = e;
                Some((s, e))
            }
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const BUF_LEN: size_t = 256;
        let mut buf = [0; BUF_LEN];
        let len = unsafe { pcre2_get_error_message_8(self.code, buf.as_mut_ptr(), BUF_LEN) };
        if len < 0 {
            write!(
                f,
                "Unknown PCRE error. (code: {:?}, offset: {:?})",
                self.code, self.offset
            )
        } else {
            let msg = str::from_utf8(&buf[..len as usize]).unwrap();
            write!(f, "error at {:?}: {}", self.offset, msg)
        }
    }
}