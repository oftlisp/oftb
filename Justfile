all:
	cargo check --all
	cargo doc --all
	cargo test --all
	cargo run macro-expander oftb-macro-expander
watch:
	watchexec -cre rs,oft,oftd,pest,toml just

afl:
	cd fuzz/afl/oftb-fuzz-target && cargo afl build
	cd fuzz/afl/oftb-fuzz-target && cargo afl fuzz -i in -o out target/debug/oftb-fuzz-target
fuzz:
	cargo +nightly fuzz run parse_program
