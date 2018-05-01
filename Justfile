all: check build doc test run
build:
	cargo build --all
check:
	cargo check --all
doc:
	cargo doc --all
test:
	cargo test --all
run:
	cargo run -- -vvv macro-expander oftb-macro-expander --std ministd
watch TARGETS="all":
	watchexec -cre rs,oft,oftd,pest,toml "just {{TARGETS}}"

afl:
	cd fuzz/afl/oftb-fuzz-target && cargo afl build
	cd fuzz/afl/oftb-fuzz-target && cargo afl fuzz -i in -o out target/debug/oftb-fuzz-target
fuzz:
	cargo +nightly fuzz run parse_program
