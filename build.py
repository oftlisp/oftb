#!/usr/bin/env python3

from argparse import ArgumentParser
import filecmp
from os import chdir, makedirs, remove
from os.path import abspath, isdir, join, realpath
import shutil
import subprocess
import tarfile
from tempfile import NamedTemporaryFile


def command(cmd, redirect=None, verbose=False):
    if verbose:
        print(" ".join(cmd))
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


def cargo(subcmd, mode="debug", verbose=False):
    flags = ["cargo"]
    if not verbose:
        flags.append("-q")
    flags.append(subcmd)
    if mode == "release":
        flags.append("--release")
    return command(flags, verbose=verbose)


oftb_exec = "target/release/oftb"


def oftb(args, redirect=None, verbose=False):
    return command([oftb_exec, "-v"] + args,
                   redirect=redirect, verbose=verbose)


def compile(pkg_dir, bin_name, verbose=False):
    print_cyan("compile", bin_name)
    return oftb(["compile", "--std", "ministd",
                 pkg_dir, bin_name], verbose=verbose)


def interpret(pkg_dir, bin_name, *args, redirect=None, verbose=False):
    print_cyan("interpret", bin_name, *args)
    bin_path = "{}/build/{}.ofta".format(pkg_dir, bin_name)
    return oftb(["interpret", bin_path] + list(args),
                redirect=redirect, verbose=verbose)


def run(pkg_dir, bin_name, *args, redirect=None, verbose=False):
    print_cyan("run", bin_name, *args)
    args = ["run", "--std", "ministd", pkg_dir, bin_name] + list(args)
    return oftb(args, redirect=redirect, verbose=verbose)


def build_oftb(verbose=False):
    print_cyan("build oftb")
    print_cyan("check oftb", indent=2)
    cargo("check", verbose=verbose)
    print_cyan("compile oftb", indent=2)
    cargo("build", mode="release", verbose=verbose)


def test_macro_expander(use_prebuilt, verbose=False):
    def run_with_macros(pkg_dir, bin_name, *args, expected=None):
        if use_prebuilt:
            f = interpret
        else:
            f = run
        f("macro-expander", "oftb-stage2", "ministd", pkg_dir,
          bin_name, redirect="{}/build/{}.ofta".format(pkg_dir, bin_name), verbose=verbose)
        output = interpret(
            pkg_dir,
            bin_name,
            *args,
            redirect=True,
            verbose=verbose)
        if expected is not None:
            if output != expected:
                raise Exception(
                    "Assertion failed: {!r} == {!r}".format(
                        output, expected))
    run_with_macros("examples/structure", "structure",
                    expected="Got arguments: ()\nHello, world!\nhullo\nGoodbye, world!\n")
    run_with_macros("examples/structure", "structure", "foo", "bar",
                    expected="Got arguments: (\"foo\" \"bar\")\nHello, world!\nhullo\nGoodbye, world!\n(\"foo\" \"bar\")\n")


def triple_compile_macro_expander(verbose=False):
    # Compile 1: oftb -> oftb-stage2
    compile("macro-expander", "oftb-stage2", verbose=verbose)

    # Compile 2: oftb-stage2 -> oftb-stage2-2
    interpret("macro-expander", "oftb-stage2", "ministd", "macro-expander",
              "oftb-stage2", redirect="macro-expander/build/oftb-stage2-2.ofta",
              verbose=verbose)
    # oftb-stage2 and oftb-stage2-2 should be equivalent in
    # functionality, but are possibly not identical files, as oftb and
    # oftb-stage2 may apply different optimizations, order things
    # differently, etc.

    # Compile 3: oftb-stage2-2 -> oftb-stage2-3
    interpret("macro-expander", "oftb-stage2-2", "ministd", "macro-expander",
              "oftb-stage2", redirect="macro-expander/build/oftb-stage2-3.ofta",
              verbose=verbose)

    # oftb-stage2-2 and oftb-stage2-3 should be identical,
    # given that the macro expander is deterministic (which it should be).
    if not filecmp.cmp("macro-expander/build/oftb-stage2-2.ofta",
                       "macro-expander/build/oftb-stage2-3.ofta"):
        raise Exception("oftb-stage2 is not idempotent")

    shutil.copy("macro-expander/build/oftb-stage2-3.ofta",
                "macro-expander/build/oftb-stage2.ofta")
    remove("macro-expander/build/oftb-stage2-2.ofta")
    remove("macro-expander/build/oftb-stage2-3.ofta")


def compile_extras(verbose=False):
    compile("macro-expander", "oftb-expand", verbose=verbose)


def bootstrap(verbose=False):
    run("macro-expander", "make-prelude", "ministd",
        redirect="ministd/src/prelude.oft")
    test_macro_expander(False, verbose=verbose)
    triple_compile_macro_expander(verbose=verbose)
    test_macro_expander(True, verbose=verbose)
    compile_extras(verbose=verbose)


def make_archive():
    with tarfile.open("oftb.tar.gz", "w:gz") as tar:
        tar.add(oftb_exec, arcname="oftb")
        tar.add("macro-expander")
        tar.add("ministd")


if __name__ == "__main__":
    chdir(abspath(join(__file__, "..")))
    parser = ArgumentParser()
    parser.add_argument("--no-oftb-build", action="store_true")
    parser.add_argument("--rebuild-macro-expander", action="store_true")
    parser.add_argument("--use-system-oftb", action="store_true")
    parser.add_argument("--verbose", action="store_true")
    args = parser.parse_args()
    if args.use_system_oftb:
        oftb_exec = "oftb"
    elif not args.no_oftb_build:
        build_oftb(verbose=args.verbose)
    if args.rebuild_macro_expander:
        triple_compile_macro_expander(verbose=args.verbose)
    else:
        bootstrap(verbose=args.verbose)
        make_archive()
    print_cyan("done")
