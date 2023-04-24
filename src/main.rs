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
use sender::{Sender, Udp};
use matcher::Regex;

fn main() {
    let udp = Udp::new("127.0.0.1:7878").unwrap();

    let target = r"a;jhgoqoghqoj0329 u0tyu10hg0h9Y0Y9827342482y(Y0y(G)_)lajf;lqjfgqhgpqjopjqa=)*(^!@#$%^&*())9999999";
    // Use Positive Lookbehind (?<=) and Positive Lookahead (?=)
    let pattern = r"(?<=\d{4})[^\d\s]{3,11}(?=\S)";
    // We use port 0 to let the operating system allocate an available port for us.

    for mat in Regex::new(pattern).unwrap().find_iter(target) {
        let matched = get_matched(target, mat.0, mat.1);
        println!("Matched: {matched}");
        // let len = udp.se
        //     .send_to(matched.as_bytes(), &(cli.addr.to_string(), cli.port))
        //     .expect("Could not send data to server");
        // println!("Send len: {len}");
        udp.send(matched.as_bytes()).unwrap();
    }
}

fn get_matched(target: &str, begin: usize, end: usize) -> String {
    String::from(&target[begin..end])
}
