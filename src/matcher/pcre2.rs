//! General match step:
//! 1. compile the pattern string: `pattern = pcre2_compile_*`, if Null pattern create failed, exit.
//! 2. set up match data block to hold the matching result with `match_data = pcre2_match_data_create_from_pattern_*`.
//!    If you need to know the block size: `block_size = pcre2_get_match_data_size(match_data)`;
//! 3. run match: `rc = pcre2_match_*`, If failed(rc < 0), free the match data block and pattern, created in step 1 & 2.
//!    pcre2_match_data_free_*(match_data) & pcre2_code_free_*(pattern).
//!    case PCRE2_ERROR_NOMATCH(-1), and others if need to special handle.
//! 4. get the matched output pointer of string vector: `pcre2_get_ovector_pointer(match_data)`.
//!    you can get the ovector count: `count = pcre2_get_ovector_count_*(match_data)`
//!
//! 5. don't forget to free the `pattern` and `match data block`.

#![allow(non_camel_case_types)]
#![allow(dead_code)]

use pcre2_sys::*;
use std::fmt;
use std::ops::BitAnd;
use std::ops::BitXor;
use std::ptr;
use std::str;

use anyhow::{anyhow, bail, Result};
use libc::{c_int, size_t};

pub struct CompileContext(*mut pcre2_compile_context_8);

impl CompileContext {
    // panic when allocate failed
    pub fn new() -> Self {
        let ctx = unsafe { pcre2_compile_context_create_8(ptr::null_mut()) };
        assert!(!ctx.is_null(), "context allocate fail");
        Self(ctx)
    }

    fn as_mut_ptr(&mut self) -> *mut pcre2_compile_context_8 {
        self.0
    }
}

/// The following option bits can be passed only to pcre2_compile(). However,
/// they may affect compilation, JIT compilation, and/or interpretive execution.
/// The following tags indicate which:

/// C   alters what is compiled by pcre2_compile()
/// J   alters what is compiled by pcre2_jit_compile()
/// M   is inspected during pcre2_match() execution
/// D   is inspected during pcre2_dfa_match() execution
/// {
///     Default = 0x00000000,
///     ALLOW_EMPTY_CLASS = 0x00000001,   /* C       */
///     ALT_BSUX = 0x00000002,            /* C       */
///     AUTO_CALLOUT = 0x00000004,        /* C       */
///     CASELESS = 0x00000008,            /* C       */
///     DOLLAR_ENDONLY = 0x00000010,      /*   J M D */
///     DOTALL = 0x00000020,              /* C       */
///     DUPNAMES = 0x00000040,            /* C       */
///     EXTENDED = 0x00000080,            /* C       */
///     FIRSTLINE = 0x00000100,           /*   J M D */
///     MATCH_UNSET_BACKREF = 0x00000200, /* C J M   */
///     MULTILINE = 0x00000400,           /* C       */
///     NEVER_UCP = 0x00000800,           /* C       */
///     NEVER_UTF = 0x00001000,           /* C       */
///     NO_AUTO_CAPTURE = 0x00002000,     /* C       */
///     NO_AUTO_POSSESS = 0x00004000,     /* C       */
///     NO_DOTSTAR_ANCHOR = 0x00008000,   /* C       */
///     NO_START_OPTIMIZE = 0x00010000,   /*   J M D */
///     UCP = 0x00020000,                 /* C J M D */
///     UNGREEDY = 0x00040000,            /* C       */
///     UTF = 0x00080000,                 /* C J M D */
///     NEVER_BACKSLASH_C = 0x00100000,   /* C       */
///     ALT_CIRCUMFLEX = 0x00200000,      /*   J M D */
///     ALT_VERBNAMES = 0x00400000,       /* C       */
///     USE_OFFSET_LIMIT = 0x00800000,    /*   J M D */
///     EXTENDED_MORE = 0x01000000,       /* C       */
///     LITERAL = 0x02000000,             /* C       */
///     ENDANCHORED = 0x20000000,         /* C   M D */
///     NO_UTF_CHECK = 0x40000000,        /* C   M D */
///     ANCHORED = 0x80000000,            /* C   M D */
/// }

pub const OPTION_MASK: u32 = !0xe35efeef;

fn is_option_valid(option: u32) -> bool {
    option.bitand(OPTION_MASK).eq(&0)
}

pub struct Pattern {
    code: *mut pcre2_code_8,
}

impl Pattern {
    // SAFETY: option is specified
    pub fn new(pattern: &str, options: u32, mut ctx: CompileContext) -> Result<Self> {
        if !is_option_valid(options) {
            bail!("invalid compile option: {}", options);
        }
        let (mut error_code, mut error_offset) = (0, 0);
        let code = unsafe {
            pcre2_compile_8(
                pattern.as_ptr(),
                pattern.len(),
                options,
                &mut error_code,
                &mut error_offset,
                ctx.as_mut_ptr(),
            )
        };
        if code.is_null() {
            bail!("pattern compile error: {:?} {}", error_code, error_offset)
        }
        Ok(Self { code })
    }
}

mod ffi {
    use pcre2_sys::{pcre2_match_8, PCRE2_ERROR_NOMATCH, PCRE2_SIZE, PCRE2_SPTR8};

    pub fn pcre2_match() {
        unsafe {
            // let rc = pcre2_match_8(
            //     code.as_ptr(),
            //     subject.as_ptr(),
            //     subject.len(),
            //     start,
            //     options,
            //     self.as_mut_ptr(),
            //     self.match_context,
            // );
            let rc = 0;
            if rc == PCRE2_ERROR_NOMATCH {
                // no match
                todo!()
            } else if rc > 0 {
                // match successfully
                todo!()
            } else {
                // since we create match data always with
                // pcre2_match_data_create_from_pattern, so the
                // ovector should big enough
                assert!(rc != 0);
                // other error handle
                // Err(Error::matching(rc))
                todo!()
            }
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    mod context {
        use crate::matcher::CompileContext;

        #[test]
        fn test_context() {
            let ctx = CompileContext::new();
            assert!(!ctx.0.is_null());
        }
    }

    #[test]
    fn test_option_valid() {
        let re = is_option_valid(0x20000000);
        assert!(re);
        let re = is_option_valid(0x10000000);
        assert!(!re);
    }

    #[test]
    fn test_compile_option() {
        let ctx = CompileContext::new();
        let pattern = Pattern::new(r"*", 0x10000000, ctx);
        assert!(pattern.is_err());
        let ctx = CompileContext::new();
        let pattern = Pattern::new(r"(?<=\d{4})[^\d\s]{3,11}(?=\S)", 0x20000000, ctx);
        assert!(pattern.is_ok());
    }

}
