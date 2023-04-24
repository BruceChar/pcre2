mod pcre2;
pub use pcre2::*;

pub trait Matcher {
    fn matchi(&self);
}