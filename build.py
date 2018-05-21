#!/usr/bin/env python3

import shutil, subprocess, tempfile

def command(cmd, redirect=None):
    if redirect == None:
        code = subprocess.call(cmd)
        assert code == 0
    else:
        with tempfile.NamedTemporaryFile() as f:
            code = subprocess.call(cmd, stdout=f)
            assert code == 0
            shutil.copy(f.name, redirect)

def print_cyan(*args):
    from sys import stdout
    stdout.write("\x1b[1;36m")
    print(*args, end="")
    stdout.write("\x1b[0m\n")

def cargo(subcmd, mode="debug"):
    flags = []
    if mode == "release":
        flags.append("--release")
    command(["cargo", "-q", subcmd] + flags)
def oftb(args, redirect=None):
    command(["target/release/oftb", "-v"] + args, redirect=redirect)

def compile(pkg_dir, bin_name):
    print_cyan("compile", bin_name)
    oftb(["compile", "--std", "ministd", pkg_dir, bin_name])
def interpret(pkg_dir, bin_name, args, redirect=None):
    print_cyan("interpret", bin_name, *args)
    args = ["interpret", "{}/build/{}.ofta".format(pkg_dir, bin_name)] + args
    oftb(args, redirect=redirect)
def macro_expand(pkg_dir, bin_name):
    interpret("macro-expander", "oftb-macro-expander",
        ["ministd", pkg_dir, bin_name])
def run(pkg_dir, bin_name, args, redirect=None):
    print_cyan("run", bin_name, *args)
    args = ["run", "--std", "ministd", pkg_dir, bin_name] + args
    oftb(args, redirect=redirect)

def build_and_self_test():
    print_cyan("Building oftb...")
    cargo("check")
    cargo("doc")
    cargo("build", mode="release")
def bootstrap():
    run("macro-expander", "make-env", ["ministd"],
        redirect="macro-expander/src/interpreter/env.oft")
    run("macro-expander", "make-prelude", ["ministd"],
        redirect="ministd/src/prelude.oft")
    compile("macro-expander", "oftb-macro-expander")
    macro_expand("examples/do-notation", "list-monad")
    interpret("examples/do-notation", "list-monad")

if __name__ == "__main__":
    build_and_self_test()
    bootstrap()
