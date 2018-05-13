all: check build doc test bootstrap
bootstrap:
	@just macro-expand examples/do-notation list-monad
	@just interpret examples/do-notation/build/list-monad.ofta
hello-world:
	@just run examples/hello-world hello-world

build:
	cargo build --all
build-macro-expander:
	@just compile macro-expander oftb-macro-expander
check:
	cargo check --all
doc:
	cargo doc --all
test:
	cargo test --all
watch TARGETS="all":
	watchexec -cre rs,oft,oftd,pest,toml "just {{TARGETS}}"

compile PKG BIN:
	cargo run -- compile --std ministd {{PKG}} {{BIN}}
interpret OFTA ARGS="":
	cargo run -- interpret {{OFTA}} {{ARGS}}
macro-expand PKG BIN: build-macro-expander
	just interpret macro-expander/build/oftb-macro-expander.ofta "{{PKG}} {{BIN}}"
run PKG BIN ARGS="":
	cargo run -- run --std ministd {{PKG}} {{BIN}} {{ARGS}}

afl:
	cd fuzz/afl/oftb-fuzz-target && cargo afl build
	cd fuzz/afl/oftb-fuzz-target && cargo afl fuzz -i in -o out target/debug/oftb-fuzz-target
fuzz:
	cargo +nightly fuzz run parse_program

outdated-deps:
	cargo outdated -R
todos:
	@rg 'TODO|unimplemented' -g '!Justfile'
