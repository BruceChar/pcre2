#![allow(dead_code)]

mod pcre2;
pub use pcre2::*;


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Match<'s> {
    subject: &'s [u8],
    start: usize,
    end: usize,
}

impl<'s> Match<'s> {
    /// Creates a new match from the given subject string and byte offsets.
    pub fn new(subject: &'s [u8], start: usize, end: usize) -> Match<'s> {
        Match {
            subject,
            start,
            end,
        }
    }
    /// Returns the starting byte offset of the match in the subject.
    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the ending byte offset of the match in the subject.
    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    /// Returns the matched portion of the subject string.
    #[inline]
    pub fn as_bytes(&self) -> &'s [u8] {
        self.subject
    }

    pub fn to_string(&self) -> &str {
        std::str::from_utf8(self.subject).unwrap()
    }
}

pub trait Matcher {
    fn find(&self) ;
}