#[macro_use]
extern crate clap;
extern crate semver;
extern crate toml_edit;

use std::fs;
use std::io::Write;

use clap::{App, Arg, ArgGroup, ArgMatches, SubCommand};
use semver::Version;
use toml_edit::{value, Document};

fn read_manifest(path: &str) -> Document {
    fs::read_to_string(path)
        .expect("Could not find Cargo.toml")
        .parse::<Document>()
        .expect("Invalid Cargo.toml")
}

fn read_version(manifest: &Document) -> Version {
    let version_str = manifest["package"]["version"].as_str().unwrap();

    Version::parse(version_str).expect(&format!(
        "Invalid package version: {} in Cargo.toml",
        version_str
    ))
}

fn read(manifest_path: &str, matches: &ArgMatches) {
    let manifest = read_manifest(manifest_path);
    let version = read_version(&manifest);

    let component = if matches.is_present("major") {
        version.major.to_string()
    } else if matches.is_present("minor") {
        version.minor.to_string()
    } else if matches.is_present("patch") {
        version.patch.to_string()
    } else if matches.is_present("pre") {
        version.pre.iter()
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join(".")
    } else if matches.is_present("build") {
        version.build.iter()
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join(".")
    } else if matches.is_present("version") {
        version.to_string()
    } else {
        panic!("Unreachable - at least one argument to bump must be specified.");
    };

    println!("{}", component);
}

fn bump(manifest_path: &str, matches: &ArgMatches) {
    let mut manifest = read_manifest(manifest_path);
    let mut version = read_version(&manifest);

    if matches.is_present("major") {
        version.increment_major();
    } else if matches.is_present("minor") {
        version.increment_minor();
    } else if matches.is_present("patch") {
        version.increment_patch();
    } else if let Some(pre) = matches.value_of("pre") {
        version.pre = Version::parse(&format!("0.0.0-{}", pre))
            .expect(&format!("Invalid pre-release given: {}", pre))
            .pre;
    } else if let Some(build) = matches.value_of("build") {
        version.build = Version::parse(&format!("0.0.0+{}", build))
            .expect(&format!("Invalid build given: {}", build))
            .build;
    } else if let Some(new_version_str) = matches.value_of("version") {
        version.clone_from(
            &Version::parse(new_version_str)
                .expect(&format!("Invalid new version given: {}", new_version_str)),
        );
    } else {
        panic!("Unreachable - at least one argument to bump must be specified.");
    };

    manifest["package"]["version"] = value(version.to_string());

    fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(manifest_path)
        .expect("Could not find Cargo.toml to write to.")
        .write_all(manifest.to_string().as_bytes())
        .expect("Failed to write updated manifest to Cargo.toml");
}

fn main() {
    let matches = App::new("version-bump")
        .version(crate_version!())
        .subcommand(
            SubCommand::with_name("read")
                .arg(
                    Arg::with_name("version")
                        .long("version")
                        .help("Print the VERSION set in the given manifest.")
                )
                .arg(
                    Arg::with_name("major")
                        .long("major")
                        .help("Print the MAJOR version of this package.")
                )
                .arg(
                    Arg::with_name("minor")
                        .long("minor")
                        .help("Print the MINOR version of this package.")
                )
                .arg(
                    Arg::with_name("patch")
                        .long("patch")
                        .help("Print the PATCH version of this package.")
                )
                .arg(
                    Arg::with_name("pre")
                        .long("pre")
                        .help("Print the PRE-RELEASE version of this package.")
                )
                .arg(
                    Arg::with_name("build")
                        .long("build")
                        .help("Print the BUILD version of this package.")
                )
                .group(
                    ArgGroup::with_name("read-args")
                        .args(&["version", "major", "minor", "patch", "pre", "build"])
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("bump")
                .arg(
                    Arg::with_name("major")
                        .long("major")
                        .help("Bump the MAJOR version."),
                )
                .arg(
                    Arg::with_name("minor")
                        .long("minor")
                        .help("Bump the MINOR version."),
                )
                .arg(
                    Arg::with_name("patch")
                        .long("patch")
                        .help("Bump the PATCH version.")
                )
                .arg(
                    Arg::with_name("pre")
                        .long("pre")
                        .help("Set the PRE-RELEASE version.")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("build")
                        .long("build")
                        .help("Set the BUILD metadata.")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("version")
                        .long("version")
                        .help("Set VERSION")
                        .takes_value(true),
                )
                .group(
                    ArgGroup::with_name("bump-args")
                        .args(&["version", "major", "minor", "patch", "pre", "build"])
                        .required(true),
                ),
        )
        .arg(
            Arg::with_name("manifest-path")
                .long("manifest-path")
                .help("Path to Cargo.toml")
                .takes_value(true)
                .default_value("Cargo.toml"),
        )
        .get_matches();

    match matches.subcommand() {
        ("bump", Some(bump_matches)) => { bump(matches.value_of("manifest-path").unwrap(), bump_matches) },
        ("read", Some(read_matches)) => { read(matches.value_of("manifest-path").unwrap(), read_matches) },
        (_, _) => { panic!("Unreachable - at least one subcommand must be specified.") }

    };
}
