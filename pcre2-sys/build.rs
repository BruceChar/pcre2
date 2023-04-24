extern crate bindgen;
extern crate cc;
extern crate pkg_config;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use bindgen::CargoCallbacks;

// Files that PCRE2 needs to compile.
const FILES: &'static [&'static str] = &[
    "pcre2posix.c",
    "pcre2_auto_possess.c",
    "pcre2_compile.c",
    "pcre2_config.c",
    "pcre2_context.c",
    "pcre2_convert.c",
    "pcre2_dfa_match.c",
    "pcre2_error.c",
    "pcre2_extuni.c",
    "pcre2_find_bracket.c",
    "pcre2_jit_compile.c",
    "pcre2_maketables.c",
    "pcre2_match.c",
    "pcre2_match_data.c",
    "pcre2_newline.c",
    "pcre2_ord2utf.c",
    "pcre2_pattern_info.c",
    "pcre2_script_run.c",
    "pcre2_serialize.c",
    "pcre2_string_utils.c",
    "pcre2_study.c",
    "pcre2_substitute.c",
    "pcre2_substring.c",
    "pcre2_tables.c",
    "pcre2_ucd.c",
    "pcre2_valid_utf.c",
    "pcre2_xclass.c",
    "pcre2_chkdint.c", // recent addition missed in NON-AUTOTOOLS list
                       // "pcre2_jit_match.c",
                       // "pcre2_jit_misc.c",
];

fn main() {
    println!("cargo:rerun-if-env-changed=PCRE2_SYS_STATIC");

    // link search path
    println!("cargo:rustc-link-search=pcre2-sys/target/");

    let target = env::var("TARGET").unwrap();
    let out = PathBuf::from("./target");

    // Don't link to a system library if we want a static build.
    let want_static = use_pcre2_sys_static().unwrap_or(target.contains("musl"));
    if !want_static && pkg_config::probe_library("libpcre2-8").is_ok() {
        return;
    }

    // make sure our pcre2 submodule has been loaded.
    if has_git() && !Path::new("pcre2/.git").exists() {
        Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status()
            .unwrap();
    }

    let mut builder = cc::Build::new();
    builder
        .out_dir("./target")
        .define("PCRE2_CODE_UNIT_WIDTH", "8")
        .define("HAVE_STDLIB_H", "1")
        .define("HAVE_MEMMOVE", "1")
        .define("HEAP_LIMIT", "20000000")
        .define("LINK_SIZE", "2")
        .define("MATCH_LIMIT", "10000000")
        .define("MATCH_LIMIT_DEPTH", "10000000")
        .define("MAX_NAME_COUNT", "10000")
        .define("MAX_NAME_SIZE", "32")
        .define("NEWLINE_DEFAULT", "2")
        .define("PARENS_NEST_LIMIT", "250")
        .define("PCRE2_STATIC", "1")
        .define("STDC_HEADERS", "1")
        .define("SUPPORT_PCRE2_8", "1")
        .define("SUPPORT_UNICODE", "1")
        .define("PCRE2GREP_BUFSIZE", "20480")
        .define("PCRE2GREP_MAX_BUFSIZE", "1048576");

    if target.contains("windows") {
        builder.define("HAVE_WINDOWS_H", "1");
    }

    // Copy PCRE2 headers manually.
    let include = out.join("include");
    fs::create_dir_all(&include).unwrap();
    fs::copy("pcre2/src/config.h.generic", include.join("config.h")).unwrap();
    fs::copy("pcre2/src/pcre2.h.generic", include.join("pcre2.h")).unwrap();

    // Same deal for chartables. Just use the default.
    let src = out.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::copy(
        "pcre2/src/pcre2_chartables.c.dist",
        src.join("pcre2_chartables.c"),
    ).unwrap();

    // Build everything.
    builder
        .include("pcre2/src")
        .include(&include)
        .file(src.join("pcre2_chartables.c"));
    for file in FILES {
        builder.file(Path::new("pcre2/src").join(file));
    }

    if env::var("PCRE2_SYS_DEBUG").unwrap_or(String::new()) == "1" {
        builder.debug(true);
    }
    builder.compile("pcre2");

    binding();
}

fn has_git() -> bool {
    Command::new("git")
        .arg("--help")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn use_pcre2_sys_static() -> Option<bool> {
    match env::var("PCRE2_SYS_STATIC") {
        Err(_) => None,
        Ok(s) => {
            if s == "1" {
                Some(true)
            } else if s == "0" {
                Some(false)
            } else {
                None
            }
        }
    }
}

fn binding() {
    let lib_dir = PathBuf::from("./target");

    // head file path need to binding
    let header_path = lib_dir.join("include/pcre2.h");
    let header_path_str = header_path.to_str().expect("invalid head path");
   
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(header_path_str)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(CargoCallbacks))
        .raw_line("#![allow(clippy::all)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .derive_debug(true)
        .derive_eq(true)
        .ctypes_prefix("::libc")
        .allowlist_function("^pcre2_.*")
        .allowlist_type("^pcre2_.*")
        .allowlist_var("^PCRE2_.*")
        .blocklist_function("^.*_callout_.*")
        .blocklist_type("^.*_callout_.*")
        .clang_arg("-DPCRE2_CODE_UNIT_WIDTH=8")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the src/bindings.rs file.
    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}
