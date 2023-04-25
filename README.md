A simple `PCRE2` library usecase in rust with FFI binding.

## Run
Ooen a terminal run:
```bash
./receiver.sh
```
then
```
cargo run
```


## Issue
[Build error](https://github.com/PCRE2Project/pcre2/issues/241) with the [non autotools build guider](https://pcre2project.github.io/pcre2/doc/html/NON-AUTOTOOLS-BUILD.txt). The build source need add the `pcre2_chkdint.c` file.