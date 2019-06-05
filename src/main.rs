#[macro_use]
extern crate clap;
extern crate semver;
extern crate toml_edit;

use std::fs;
use std::io::Write;

use clap::{App, Arg, ArgGroup};
use semver::Version;
use toml_edit::{value, Document};

fn main() {
    let matches = App::new("version-bump")
        .version(crate_version!())
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
                .help("Bump the PATCH version."),
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
            ArgGroup::with_name("versions")
                .args(&["version", "major", "minor", "patch", "pre", "build"])
                .required(true),
        )
        .arg(
            Arg::with_name("manifest-path")
                .long("manifest-path")
                .help("Path to Cargo.toml")
                .takes_value(true)
                .default_value("Cargo.toml"),
        )
        .get_matches();

    let manifest_file = fs::read_to_string(matches.value_of("manifest-path").unwrap())
        .expect("Could not find Cargo.toml");
    let mut manifest_document = manifest_file
        .parse::<Document>()
        .expect("Invalid Cargo.toml");

    let version_str = manifest_document["package"]["version"].as_str().unwrap();
    let mut version = Version::parse(version_str).expect(&format!(
        "Invalid package version: {} in Cargo.toml",
        version_str
    ));

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
        panic!("Unreachable");
    };

    manifest_document["package"]["version"] = value(version.to_string());

    fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(matches.value_of("manifest-path").unwrap())
        .expect("Could not find Cargo.toml to write to.")
        .write_all(manifest_document.to_string().as_bytes())
        .expect("Failed to write updated manifest to Cargo.toml");
}
