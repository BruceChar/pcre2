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
use std::ops::BitAnd;
use std::ptr;
use std::slice;
use std::str;

use anyhow::{anyhow, bail, Result};

use super::Match;

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

impl Drop for CompileContext {
    fn drop(&mut self) {
        unsafe { pcre2_compile_context_free_8(self.0) }
    }
}

/// The following option bits can be passed only to pcre2_compile(). However,
/// they may affect compilation, JIT compilation, and/or interpretive execution.
/// The following tags indicate which:
/// C   alters what is compiled by pcre2_compile()
/// J   alters what is compiled by pcre2_jit_compile()
/// M   is inspected during pcre2_match() execution
/// D   is inspected during pcre2_dfa_match() execution
/// pub enum CompileOptions {
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

#[derive(Debug)]
pub struct Pattern {
    code: *mut pcre2_code_8,
}

impl Default for Pattern {
    fn default() -> Self {
        Self {
            code: ptr::null_mut(),
        }
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        unsafe { pcre2_code_free_8(self.code) }
    }
}

impl Pattern {
    pub fn new(pattern: &str) -> Result<Self> {
        // the default fast options
        Pattern::new_with(pattern, PCRE2_UCP | PCRE2_UTF, CompileContext::new())
    }

    // SAFETY: option is specified
    pub fn new_with(pattern: &str, options: u32, mut ctx: CompileContext) -> Result<Self> {
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

    pub fn as_ptr(&self) -> *const pcre2_code_8 {
        self.code
    }
}

pub struct MatchData {
    data: *mut pcre2_match_data_8,
    ovector_ptr: *const usize,
    ovector_cnt: u32,
}

impl Drop for MatchData {
    fn drop(&mut self) {
        unsafe {
            pcre2_match_data_free_8(self.data);
        }
    }
}

impl MatchData {
    pub fn new(pattern: &Pattern) -> Self {
        let data =
            unsafe { pcre2_match_data_create_from_pattern_8(pattern.as_ptr(), ptr::null_mut()) };
        assert!(!data.is_null(), "failed to allocate match data block");
        let ovector_ptr = unsafe { pcre2_get_ovector_pointer_8(data) };

        assert!(!ovector_ptr.is_null(), "null ovector pointer");
        let ovector_cnt = unsafe { pcre2_get_ovector_count_8(data) };
        MatchData {
            data,
            ovector_ptr,
            ovector_cnt,
        }
    }

    pub fn as_mut_ptr(&self) -> *mut pcre2_match_data_8 {
        self.data
    }

    pub fn ovector(&self) -> &[usize] {
        // SAFETY: Both our ovector pointer and count are derived directly from
        // the creation of a valid match data block.
        unsafe { slice::from_raw_parts(self.ovector_ptr, self.ovector_cnt as usize * 2) }
    }
}


pub struct Matches<'p, 's> {
    re: &'p PCRE2,
    data: &'p MatchData,
    subject: &'s [u8],
    last_end: usize,
    last_match: Option<usize>,
}

impl<'r, 's> Iterator for Matches<'r, 's> {
    type Item = Result<Match<'s>>;

    fn next(&mut self) -> Option<Result<Match<'s>>> {
        if self.last_end > self.subject.len() {
            return None;
        }
        let res = self.re.find_at(
            self.subject,
            self.last_end,
        );
        let m = match res {
            Err(err) => {
                if err.to_string().eq("No match items") {
                    return None
                }
                return Some(Err(err))
            },
            Ok(m) => m,
        };
        if m.start() == m.end() {
            // This is an empty match. To ensure we make progress, start
            // the next search at the smallest possible starting position
            // of the next match following this one.
            self.last_end = m.end() + 1;
            // Don't accept empty matches immediately following a match.
            // Just move on to the next match.
            if Some(m.end()) == self.last_match {
                return self.next();
            }
        } else {
            self.last_end = m.end();
        }
        self.last_match = Some(m.end());
        Some(Ok(m))
    }
}

pub struct PCRE2 {
    /// compile options
    options: u32,
    /// origin pattern string
    origin: String,
    /// compiled pcre2 pattern
    pattern: Pattern,
    /// match data used by pcre2 during matching
    data: MatchData,
}

impl PCRE2 {
    pub fn new(pattern: &str) -> Result<Self> {
        PCRE2Builder::new().build(pattern)
    }

    /// default NO_UTF_CHECK match mod
    /// and just do one match, match all see [`find_iter`]
    pub fn find_at<'s>(&self, subject: &'s [u8], start: usize) -> Result<Match<'s>> {
        self.find_at_with_options(subject, start, PCRE2_NO_UTF_CHECK)
    }

    pub fn find_at_with_options<'s>(
        &self,
        subject: &'s [u8],
        start: usize,
        options: u32,
    ) -> Result<Match<'s>> {
        let rc = unsafe {
            pcre2_match_8(
                self.pattern.as_ptr(),
                subject.as_ptr(),
                subject.len(),
                start,
                options,
                self.data.as_mut_ptr(),
                ptr::null_mut(),
            )
        };
        if rc == PCRE2_ERROR_NOMATCH {
            // no match
            return Err(anyhow!("No match items"));
        } else if rc > 0 {
            // match successfully
            let ovector = self.data.ovector();
            let (start, end) = (ovector[0], ovector[1]);
            Ok(Match {
                subject: &subject[start..end],
                start,
                end,
            })
        } else {
            // since we create match data always with
            // pcre2_match_data_create_from_pattern, so the
            // ovector should big enough
            assert!(rc != 0);
            // other error handle
            return Err(anyhow!("find error"))
        }
    }

    pub fn find_iter<'p, 's>(&'p self, subject: &'s [u8]) -> Matches<'p, 's> {
        Matches {
            re: self,
            data: &self.data,
            subject,
            last_end: 0,
            last_match: None,
        }
    }

    pub fn is_match(&self, subject: &[u8]) -> bool {
        self.find_at(subject, 0).is_ok()
    }
}

#[derive(Default, Debug)]
pub struct PCRE2Builder {
    options: u32,
}

impl PCRE2Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self, pattern: &str) -> Result<PCRE2> {
        // create pattern with compile options, default: 0x00000000
        let origin = pattern.to_string();
        let pattern = Pattern::new_with(pattern, self.options, CompileContext::new())?;
        let data = MatchData::new(&pattern);
        Ok(PCRE2 {
            options: self.options,
            origin,
            pattern,
            data,
        })
    }

    // don't check the option validity here
    pub fn options(mut self, options: u32) -> Self {
        self.options = options;
        self
    }

    pub fn add_option(mut self, option: u32) -> Self {
        self.options |= option;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_context_new() {
        let ctx = CompileContext::new();
        assert!(!ctx.0.is_null());
    }

    #[test]
    fn test_is_option_valid() {
        let valid = is_option_valid(0x20000000);
        assert!(valid);
        let valid = is_option_valid(0x10000000);
        assert!(!valid);
    }

    #[test]
    fn test_pattern_new() {
        let pattern = Pattern::new(r"*");
        assert!(pattern.is_err());
        let pattern = Pattern::new(r"(?<=\d{4})[^\d\s]{3,11}(?=\S)");
        assert!(pattern.is_ok());
    }
}
