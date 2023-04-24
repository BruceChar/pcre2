#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

mod bindings;
pub use bindings::*;

// STRANGE: didn't auto binding
pub const PCRE2_UNSET: usize = ::std::usize::MAX;
pub const PCRE2_ZERO_TERMINATED: usize = ::std::usize::MAX;
pub type PCRE2_SIZE = usize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfa_match() {
        unsafe {

        }
    }
}
