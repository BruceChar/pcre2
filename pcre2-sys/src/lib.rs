mod bindings;
pub use bindings::*;

// didn't auto binding
pub const PCRE2_UNSET: usize = ::std::usize::MAX;
pub const PCRE2_ZERO_TERMINATED: usize = ::std::usize::MAX;