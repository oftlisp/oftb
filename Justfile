all: check build doc test build-macro-expander
build:
	cargo build --all
build-macro-expander: build
	cargo run -- compile --std ministd macro-expander oftb-macro-expander
check:
	cargo check --all
doc:
	cargo doc --all
run-hello-world:
	cargo run -- run --std ministd examples/hello-world hello-world
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
todos:
	@rg 'TODO|unimplemented' -g '!Justfile'
