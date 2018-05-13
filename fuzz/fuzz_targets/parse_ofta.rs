#![no_main]

#[macro_use]
extern crate libfuzzer_sys;
extern crate oftb;

use oftb::flatanf::Program;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let _ = Program::deserialize_from(&mut Cursor::new(data));
});
