#![no_main]

#[macro_use]
extern crate libfuzzer_sys;
extern crate oftb;

fuzz_target!(|data: &[u8]| {
    if let Ok(src) = std::str::from_utf8(data) {
        let _ = oftb::parse_program(src);
    }
});
