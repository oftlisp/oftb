extern crate afl;
extern crate oftb;

fn main() {
    afl::read_stdio_string(|src| {
        let _ = oftb::parse_program(&src);
    });
}
