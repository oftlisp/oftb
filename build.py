#!/usr/bin/env python3

from argparse import ArgumentParser
import filecmp
from os import chdir, makedirs, remove
from os.path import abspath, isdir, join, realpath
import shutil
import subprocess
import tarfile
from tempfile import NamedTemporaryFile


def command(cmd, redirect=None):
    if redirect is True:
        return subprocess.check_output(cmd, universal_newlines=True)
    elif redirect is None:
        subprocess.check_call(cmd)
    else:
        with NamedTemporaryFile() as f:
            subprocess.check_call(cmd, stdout=f)
            d = realpath(abspath(join(redirect, "..")))
            if not isdir(d):
                makedirs(d)
            shutil.copy(f.name, redirect)


def print_cyan(*args, indent=0):
    from sys import stdout
    stdout.write(" " * indent)
    stdout.write("\x1b[1;36m")
    print(*args, end="")
    stdout.write("\x1b[0m\n")


def cargo(subcmd, mode="debug"):
    flags = []
    if mode == "release":
        flags.append("--release")
    command(["cargo", "-q", subcmd] + flags)


oftb_exec = "target/release/oftb"


def oftb(args, redirect=None):
    return command([oftb_exec, "-v"] + args, redirect=redirect)


def compile(pkg_dir, bin_name):
    print_cyan("compile", bin_name)
    return oftb(["compile", "--std", "ministd", pkg_dir, bin_name])


def interpret(pkg_dir, bin_name, *args, redirect=None):
    print_cyan("interpret", bin_name, *args)
    bin_path = "{}/build/{}.ofta".format(pkg_dir, bin_name)
    return oftb(["interpret", bin_path] + list(args), redirect=redirect)


def run(pkg_dir, bin_name, *args, redirect=None):
    print_cyan("run", bin_name, *args)
    args = ["run", "--std", "ministd", pkg_dir, bin_name] + list(args)
    return oftb(args, redirect=redirect)


def build_oftb():
    print_cyan("build oftb")
    print_cyan("check oftb", indent=2)
    cargo("check")
    print_cyan("document oftb", indent=2)
    cargo("doc")
    print_cyan("compile oftb", indent=2)
    cargo("build", mode="release")


def test_macro_expander(use_prebuilt):
    def run_with_macros(pkg_dir, bin_name, *args, expected=None):
        if use_prebuilt:
            f = interpret
        else:
            f = run
        f("macro-expander", "oftb-macro-expander", "ministd", pkg_dir,
          bin_name, redirect="{}/build/{}.ofta".format(pkg_dir, bin_name))
        output = interpret(pkg_dir, bin_name, *args, redirect=True)
        if expected is not None:
            if output != expected:
                raise Exception(
                    "Assertion failed: {!r} == {!r}".format(
                        output, expected))
    run_with_macros("examples/structure", "structure",
                    expected="Got arguments: ()\nHello, world!\nhullo\nGoodbye, world!\n")
    run_with_macros("examples/structure", "structure", "foo", "bar",
                    expected="Got arguments: (\"foo\" \"bar\")\nHello, world!\nhullo\nGoodbye, world!\n(\"foo\" \"bar\")\n")


def triple_compile_macro_expander():
    # Compile 1: oftb -> oftb-macro-expander
    compile("macro-expander", "oftb-macro-expander")

    # Compile 2: oftb-macro-expander -> oftb-macro-expander-2
    interpret("macro-expander", "oftb-macro-expander", "ministd", "macro-expander",
              "oftb-macro-expander", redirect="macro-expander/build/oftb-macro-expander-2.ofta")
    # oftb-macro-expander and oftb-macro-expander-2 should be equivalent in
    # functionality, but are possibly not identical files, as oftb and
    # oftb-macro-expander may apply different optimizations, order things
    # differently, etc.

    # Compile 3: oftb-macro-expander-2 -> oftb-macro-expander-3
    interpret("macro-expander", "oftb-macro-expander-2", "ministd", "macro-expander",
              "oftb-macro-expander", redirect="macro-expander/build/oftb-macro-expander-3.ofta")

    # oftb-macro-expander-2 and oftb-macro-expander-3 should be identical,
    # given that the macro expander is deterministic (which it should be).
    if not filecmp.cmp("macro-expander/build/oftb-macro-expander-2.ofta",
                       "macro-expander/build/oftb-macro-expander-3.ofta"):
        raise Exception("oftb-macro-expander is not idempotent")

    shutil.copy("macro-expander/build/oftb-macro-expander-3.ofta",
                "macro-expander/build/oftb-macro-expander.ofta")
    remove("macro-expander/build/oftb-macro-expander-2.ofta")
    remove("macro-expander/build/oftb-macro-expander-3.ofta")


def bootstrap():
    run("macro-expander", "make-prelude", "ministd",
        redirect="ministd/src/prelude.oft")
    test_macro_expander(False)
    triple_compile_macro_expander()
    test_macro_expander(True)


def make_archive():
    with tarfile.open("oftb.tar.gz", "w:gz") as tar:
        tar.add(oftb_exec, arcname="oftb")
        tar.add("macro-expander")
        tar.add("ministd")


if __name__ == "__main__":
    chdir(abspath(join(__file__, "..")))
    parser = ArgumentParser()
    parser.add_argument("--no-oftb-build", action="store_true")
    parser.add_argument("--use-system-oftb", action="store_true")
    args = parser.parse_args()
    if args.use_system_oftb:
        oftb_exec = "oftb"
    elif not args.no_oftb_build:
        build_oftb()
    bootstrap()
    make_archive()
