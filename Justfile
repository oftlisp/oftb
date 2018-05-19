run_flags = "--release"

all: check build doc test bootstrap
bootstrap: make-prelude
	@just macro-expand examples/do-notation list-monad
	@just interpret examples/do-notation/build/list-monad.ofta
hello-world:
	@just run examples/hello-world hello-world
make-env:
	#!/usr/bin/env python3
	import shutil, subprocess, tempfile
	with tempfile.NamedTemporaryFile() as f:
		code = subprocess.call(["just", "run", "macro-expander",
		    "make-env", "ministd"], stdout=f)
		assert code == 0
		shutil.copy(f.name, "macro-expander/src/interpreter/env.oft")
make-prelude:
	#!/usr/bin/env python3
	import shutil, subprocess, tempfile
	with tempfile.NamedTemporaryFile() as f:
		code = subprocess.call(["just", "run", "macro-expander",
		    "make-prelude", "ministd"], stdout=f)
		assert code == 0
		shutil.copy(f.name, "ministd/src/prelude.oft")

build:
	cargo build --all
build-eval: make-env
	@just compile macro-expander eval-expr
build-macro-expander:
	@just compile macro-expander oftb-macro-expander
build-release:
	cargo build --all --release
check:
	cargo check --all
doc:
	cargo doc --all
test:
	cargo test --all
watch TARGETS="all":
	watchexec -cre rs,oft,oftd,pest,toml "just {{TARGETS}}"

compile PKG BIN:
	cargo run {{run_flags}} -- compile --std ministd {{PKG}} {{BIN}}
eval FILE: build-eval
	@just interpret macro-expander/build/eval-expr.ofta {{FILE}}
interpret OFTA ARGS="":
	cargo run {{run_flags}} -- -vv interpret {{OFTA}} {{ARGS}}
macro-expand PKG BIN: build-macro-expander
	@mkdir -p {{PKG}}/build
	just interpret macro-expander/build/oftb-macro-expander.ofta "{{PKG}} {{BIN}}"
	@# just interpret macro-expander/build/oftb-macro-expander.ofta "{{PKG}} {{BIN}}" > {{PKG}}/build/{{BIN}}.ofta
run PKG BIN ARGS="":
	cargo run {{run_flags}} -- -vv run --std ministd {{PKG}} {{BIN}} {{ARGS}}

afl:
	cd fuzz/afl/oftb-fuzz-target && cargo afl build
	cd fuzz/afl/oftb-fuzz-target && cargo afl fuzz -i in -o out target/debug/oftb-fuzz-target
fuzz-ofta-parser:
	cargo +nightly fuzz run parse_ofta
fuzz-program-parser:
	cargo +nightly fuzz run parse_program

outdated-deps:
	cargo outdated -R
todos:
	@rg 'TODO|unimplemented' -g '!Justfile'
