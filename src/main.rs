//! #Requirements:
//! 1. Call [pcre2](https://github.com/PhilipHazel/pcre2) bind the FFI manually without 3rd lib.
//! 2. Filter out the **result string** that meets the **filtering rules** from the **target text**.
//! 3. Send the **result string** to a bash script.
//! 4. Print the result received in script.
//! 5. Use the UDP to transmit.
//!
//! # Target Text
//! `a;jhgoqoghqoj0329 u0tyu10hg0h9Y0Y9827342482y(Y0y(G)_)lajf;lqjfgqhgpqjopjqa=)*(^!@#$%^&*())9999999`
//!
//! # Filtering rules
//! 1. No digit and any whitespace characters, length must be [3, 11]
//! 2. The string adjacent to the left of result string is 4 digits.
//! 2. The string adjacent to the right of result string is not empty.
//! 3. Fewer matches is better, use only regular expression as much as possible.
//!
mod matcher;
mod sender;

use matcher::{PCRE2};
use sender::{Sender, Udp};

fn main() {
    let udp = Udp::new("127.0.0.1:7878").unwrap();

    let target = r"a;jhgoqoghqoj0329 u0tyu10hg0h9Y0Y9827342482y(Y0y(G)_)lajf;lqjfgqhgpqjopjqa=)*(^!@#$%^&*())9999999";
    // Use Positive Lookbehind (?<=) and Positive Lookahead (?=)
    let pattern = r"(?<=\d{4})[^\d\s]{3,11}(?=\S)";
    // We use port 0 to let the operating system allocate an available port for us.

    let grep = PCRE2::new(pattern).unwrap();

    for m in grep.find_iter(target.as_bytes()) {
        match m {
            Ok(s) => {
                match udp.send(s.as_bytes()) {
                    Ok(len) => print!("send {:?} bytes: {:?}", len, s.to_string()),
                    Err(e) => print!("send error: {:?}", e)
                }
            }
            Err(e) => println!("match error: {:?}", e)
        }
    }
}
