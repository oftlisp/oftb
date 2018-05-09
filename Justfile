all: check build doc test build-macro-expander
build:
	cargo build --all
build-macro-expander: build
	cargo run --bin oftbc -- -vvv macro-expander oftb-macro-expander --std ministd
check:
	cargo check --all
doc:
	cargo doc --all
run-hello-world: build
	cargo run --bin oftbc -- examples/hello-world hello-world --std ministd -v
	cargo run --bin oftbi -- hello-world.ofta -vvvv
test:
	cargo test --all
watch TARGETS="all":
	watchexec -cre rs,oft,oftd,pest,toml "just {{TARGETS}}"

afl:
	cd fuzz/afl/oftb-fuzz-target && cargo afl build
	cd fuzz/afl/oftb-fuzz-target && cargo afl fuzz -i in -o out target/debug/oftb-fuzz-target
fuzz:
	cargo +nightly fuzz run parse_program

outdated-deps:
	cargo outdated -R
