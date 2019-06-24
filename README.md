# Table of Contents

1.  [`semvercli`](#org5312ed0)
    1.  [Overview:](#org841ba40)
    2.  [Installation:](#orgecf44db)
    3.  [Usage:](#org718da0e)


<a id="org5312ed0"></a>

# `semvercli`


<a id="org841ba40"></a>

## Overview:

   Command line utility for setting, bumping, and reading Rust pacakge versions.
This is an extremely thin layer over the [semver crate](https://crates.io/crates/semver) and meant to just serve as a
command line glue for tools such as [cargo-make](https://crates.io/crates/cargo-make).


<a id="orgecf44db"></a>

## Installation:

Via `cargo`:

    cargo install semvercli

From source:

    git clone git@github.com:kzvezdarov/semvercli
    cargo install -C semvercli/Cargo.toml


<a id="org718da0e"></a>

## Usage:

The command's interface is split into a `read` and `bump` subcommands:

    semvercli 0.0.1
    
    USAGE:
        semvercli [OPTIONS] <SUBCOMMAND>
    
    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information
    
    OPTIONS:
            --manifest-path <manifest-path>    Path to Cargo.toml [default: Cargo.toml]
    
    SUBCOMMANDS:
        bump    Bump or set a specific version component.
        help    Prints this message or the help of the given subcommand(s)
        read    Read and print a specific version component.

Bump has the following interface:

    semvercli-bump
    Bump or set a specific version component.
    
    USAGE:
        semvercli bump [OPTIONS] <--version <version>|--major|--minor|--patch|--pre <pre>|--build <build>>
    
    FLAGS:
        -h, --help     Prints help information
            --major    Bump the MAJOR version.
            --minor    Bump the MINOR version.
            --patch    Bump the PATCH version.
    
    OPTIONS:
            --build <build>        Set the BUILD metadata.
            --pre <pre>            Set the PRE-RELEASE version.
            --version <version>    Set the full VERSION

Note that `semvercli bump` will only take one flag or option at a time (i.e. it can only mutate the value of one
component per invocation).

It is used as such:

    semvercli bump --version 0.0.1
    semvercli bump --major
    semvercli bump --minor
    semvercli bump --patch
    semvercli bump --pre rc.1
    semvercli bump --build dev.amd64.linux

Read has the following interface:

    semvercli-read
    Read and print a specific version component.
    
    USAGE:
        semvercli read <--version|--major|--minor|--patch|--pre|--build>
    
    FLAGS:
            --build      Print the BUILD version of this package.
        -h, --help       Prints help information
            --major      Print the MAJOR version of this package.
            --minor      Print the MINOR version of this package.
            --patch      Print the PATCH version of this package.
            --pre        Print the PRE-RELEASE version of this package.
            --version    Print the VERSION set in the given manifest.

Note that `semvercli read` will only take one flag or option at a time(i.e. it can only read the value of one
component per invocation).

It is used as such:

    semvercli read --major
    1
    semvercli read --minor
    1
    semvercli read --patch
    1
    semvercli read --pre
    rc.1
    semvercli read --build
    dev.amd64.linux
    semvercli read --version
    1.1.1-rc.1+dev.amd64.linux
