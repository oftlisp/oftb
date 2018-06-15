build:
	./build.py
watch:
	watchexec -cre oft -i env.oft -i prelude.oft -i \*.ofta -- ./build.py --no-oftb-build
