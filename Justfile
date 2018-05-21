build:
	python3 build.py

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
